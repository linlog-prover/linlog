// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Running problems. The parent (`run`) starts this program again as a
//! child (`one`) for every run, so that a crash, a stack overflow or a run
//! that ignores its time limit costs that run only; the child times the
//! search alone, not its start or the parsing, and prints the tail of the
//! run's CSV row: once with the search's verdict, before it checks a
//! proof, and once more with the check's result. A child that takes too
//! long to load its problem, or outlives its time limit by more than the
//! grace period counted from the end of the load, is killed; one that
//! dies in its check leaves the row its verdict.

use crate::problems::{self, Reference, mode_name};
use crate::{OneArgs, RunArgs};
use anyhow::{Context, Result, anyhow};
use clap::ValueEnum;
use linlog::search::{Engine, Options, Reason, Verdict, prove_within};
use linlog::{Atom, Bias, Error, Forest, Limits, Mode, Sign};
use std::collections::HashSet;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::{Duration, Instant};

/// The columns the parent writes, then those the child prints. Files
/// from before the portfolio of worker orders was removed have a column
/// `portfolio` after `jobs`, which the summaries still read.
pub const HEADER: &str = "source,family,size,index,problem,mode,engine_requested,jobs,\
                          test_period,timeout_s,run,copies,expected,verdict,reason,checked,engine,\
                          fragment,occurrences,multiplicity,time_ms,nodes,memo_hits,memo_entries,\
                          splits,links,tests,recursion_limit,cpu_ms,wait_ms,bias,forward_copies,\
                          check_ms,memory_limit,copies_reached,pool_after,forward_copies_reached,work,\
                          tool,tool_version,translate_ms,peak_bytes,raw";

/// The columns the child prints.
const TAIL: usize = 32;

/// The line the child prints when its problem is loaded and its search
/// starts, from which the parent counts the time limit.
const LOADED: &str = "loaded";

/// Returns the index in the tail of the column `name`.
fn column(name: &str) -> usize {
    let columns: Vec<&str> = HEADER.split(',').collect();
    columns[columns.len() - TAIL..]
        .iter()
        .position(|c| *c == name)
        .expect("a column of the tail")
}

/// Which mode to run a problem in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum ModeChoice {
    /// The problem's own
    Given,
    /// The problem's own made classical
    Classical,
    /// The problem's own made intuitionistic (without Mix)
    Intuitionistic,
}

impl ModeChoice {
    /// Returns the mode to run a problem meant for `given` in.
    fn apply(self, given: Mode) -> Mode {
        match self {
            Self::Given => given,
            Self::Classical => {
                let mut mode = Mode::CLASSICAL;
                if given.is_affine() {
                    mode = mode.with_affine();
                }
                if given.has_mix() {
                    mode = mode.with_mix();
                }
                mode
            }
            Self::Intuitionistic if given.is_affine() => Mode::INTUITIONISTIC.with_affine(),
            Self::Intuitionistic => Mode::INTUITIONISTIC,
        }
    }
}

/// How the focused engines pick the positive literal of every atom.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum BiasChoice {
    /// The rarer literal in affine mode, by tensor factors in linear mode
    /// without exponentials, and in linear mode with exponentials both,
    /// each in a search of its own
    Auto,
    /// The rarer literal
    Rarer,
    /// By tensor factors
    Factors,
}

impl BiasChoice {
    /// Returns the rule of the library.
    fn bias(self) -> Bias {
        match self {
            Self::Auto => Bias::Auto,
            Self::Rarer => Bias::Rarer,
            Self::Factors => Bias::Factors,
        }
    }
}

/// One of N shards of the problems: the K-th takes the problems at the
/// places K − 1, K − 1 + N, K − 1 + 2N, … of the list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shard {
    /// Which shard, from 1.
    k: usize,
    /// How many.
    n: usize,
}

impl std::str::FromStr for Shard {
    type Err = String;

    /// Reads `K/N` with 1 ≤ K ≤ N.
    fn from_str(text: &str) -> Result<Self, String> {
        let (k, n) = text
            .split_once('/')
            .and_then(|(k, n)| Some((k.parse().ok()?, n.parse().ok()?)))
            .ok_or_else(|| format!("{text:?} is not a shard K/N"))?;
        if k == 0 || k > n {
            return Err(format!("shard {text}: K must be 1 to N"));
        }
        Ok(Self { k, n })
    }
}

impl Shard {
    /// Returns this shard's problems of the list, in their order.
    pub fn of<T>(self, list: Vec<T>) -> Vec<T> {
        list.into_iter().skip(self.k - 1).step_by(self.n).collect()
    }
}

/// The memory bound of a run in bytes: the one asked for, none for 0, or
/// the library's default.
fn memory_limit(asked: Option<u64>) -> Option<u64> {
    match asked {
        None => Some(Limits::DEFAULT_MEMORY_BYTES),
        Some(0) => None,
        bound => bound,
    }
}

/// The memory bound of a run as its column has it: the bytes, or 0 for
/// none.
fn memory_column(asked: Option<u64>) -> String {
    memory_limit(asked).unwrap_or(0).to_string()
}

/// A copy bound as `--copies` reads it: a number, or `none` for a search
/// that deepens until it decides or its time limit passes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bound(pub Option<u32>);

impl std::str::FromStr for Bound {
    type Err = String;

    /// Reads a number, or `none`.
    fn from_str(text: &str) -> Result<Self, String> {
        match text {
            "none" => Ok(Self(None)),
            n => n
                .parse()
                .map(|n| Self(Some(n)))
                .map_err(|_| format!("{n:?} is not a number of copies, or `none`")),
        }
    }
}

impl std::fmt::Display for Bound {
    /// Writes the bound as `--copies` reads it and its column has it.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            Some(n) => write!(f, "{n}"),
            None => f.write_str("none"),
        }
    }
}

/// The column of how long one thread searched before the pool: the
/// seconds, or empty for a pool from the start.
fn pool_column(asked: Option<f64>) -> String {
    asked.map_or(String::new(), |t| t.to_string())
}

/// The forward search's copy bound of a run: the one asked for, or the
/// library's default.
fn forward_copies(asked: Option<u32>) -> u32 {
    asked.unwrap_or(Options::DEFAULT_FORWARD_COPIES)
}

/// Which engine to run: one the library names, or `None` for the one the
/// fragment and the mode call for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EngineChoice(pub Option<Engine>);

impl EngineChoice {
    /// Reads an engine's name, as the library spells it, or `auto`.
    pub fn parser() -> impl clap::builder::TypedValueParser<Value = Self> {
        use clap::builder::TypedValueParser as _;
        clap::builder::PossibleValuesParser::new(
            std::iter::once("auto").chain(Engine::NAMES.iter().copied()),
        )
        .map(|name| Self(name.parse().ok()))
    }

    /// Returns the engine's name, `auto` for the automatic choice.
    fn name(self) -> &'static str {
        self.0.map_or("auto", Engine::name)
    }
}

/// The name of a value as its argument spells it.
fn name(value: &impl ValueEnum) -> String {
    value
        .to_possible_value()
        .map_or_else(String::new, |v| v.get_name().to_owned())
}

/// Runs every problem in every configuration asked for and writes the rows.
pub fn run(args: &RunArgs) -> Result<()> {
    let mut references = problems::families(&args.family, args.all_families)?;
    references.extend(problems::lltp(&args.lltp)?);
    references.extend(problems::specs(&args.spec)?);
    references.extend(problems::files(&args.problems)?);
    if !args.only.is_empty() {
        references.retain(|r| {
            let path = format!("{}/{}", r.family, r.name);
            args.only.iter().any(|only| path.contains(only.as_str()))
        });
    }
    if let Some(shard) = args.shard {
        references = shard.of(references);
    }
    if !args.tools.is_empty() {
        return crate::tools::main(args, references);
    }
    let jobs = args
        .jobs
        .iter()
        .map(|j| match j.as_str() {
            "all" => Ok(thread::available_parallelism().map_or(1, |n| n.get())),
            n => n.parse().with_context(|| format!("jobs `{n}`")),
        })
        .collect::<Result<Vec<usize>>>()?;
    // A search uses no more threads than the machine runs at once, so a
    // row with a larger count would name threads that never ran.
    if let Ok(machine) = thread::available_parallelism()
        && let Some(n) = jobs.iter().find(|&&n| n > machine.get())
    {
        anyhow::bail!(
            "--jobs {n}: this process may run {machine} threads at once, and a search uses no more"
        );
    }

    let done_before = match &args.output {
        Some(path) if args.resume => finished(path)?,
        _ => HashSet::new(),
    };
    let mut out: Box<dyn Write> = match &args.output {
        Some(path) => {
            let file = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .append(args.append)
                .truncate(!args.append)
                .open(path)
                .with_context(|| format!("opening {}", path.display()))?;
            let empty = file.metadata()?.len() == 0;
            let mut file = Box::new(std::io::BufWriter::new(file));
            if empty {
                writeln!(file, "{HEADER}")?;
            }
            file
        }
        None => {
            println!("{HEADER}");
            Box::new(std::io::stdout())
        }
    };

    let exe = std::env::current_exe()?;
    if args.reverse {
        references.reverse();
    }
    let total = references.len() * args.modes.len() * args.engines.len() * jobs.len();
    let mut done = 0;
    // The configurations run by this invocation, for the estimate of the
    // time left: those a resumed run skips cost nothing.
    let (start, mut ran) = (Instant::now(), 0);
    for reference in &references {
        let mut modes: Vec<(ModeChoice, Mode)> = Vec::new();
        for &choice in &args.modes {
            let mode = choice.apply(reference.mode);
            if !modes.iter().any(|&(_, m)| m == mode) {
                modes.push((choice, mode));
            }
        }
        for &(choice, mode) in &modes {
            for &engine in &args.engines {
                for &threads in &jobs {
                    done += 1;
                    let key = [
                        reference.source,
                        &reference.family,
                        &reference.name,
                        mode_name(mode),
                        engine.name(),
                        &threads.to_string(),
                        &args.test_period.map_or(String::new(), |p| p.to_string()),
                        &name(&args.bias),
                        &forward_copies(args.forward_copies).to_string(),
                        &memory_column(args.memory_limit),
                        &pool_column(args.pool_after),
                    ]
                    .join(",");
                    if done_before.contains(&key) {
                        continue;
                    }
                    ran += 1;
                    for run in 0..args.repeat {
                        let tail = child(&exe, reference, choice, engine, threads, args)?;
                        let fields: Vec<&str> = tail.split(',').collect();
                        let (verdict, time) = (
                            fields[column("verdict")],
                            fields[column("time_ms")].parse().unwrap_or(0.0),
                        );
                        writeln!(
                            out,
                            "{},{},{},{},{},{},{},{threads},{},{},{run},{tail}",
                            reference.source,
                            reference.family,
                            reference.size.map_or(String::new(), |s| s.to_string()),
                            reference.index.map_or(String::new(), |i| i.to_string()),
                            reference.name,
                            mode_name(mode),
                            engine.name().to_owned(),
                            args.test_period.map_or(String::new(), |p| p.to_string()),
                            args.timeout,
                        )?;
                        out.flush()?;
                        if run == 0 {
                            // The configurations left at this run's mean.
                            let left = start.elapsed().as_secs_f64() / f64::from(ran)
                                * (total - done) as f64
                                / 60.0;
                            eprintln!(
                                "[{done}/{total}] {} {} {} j{threads}: {verdict} {} {time:.3} ms, \
                                 about {left:.0} min left",
                                reference.name,
                                mode_name(mode),
                                engine.name().to_owned(),
                                fields[column("reason")],
                            );
                        }
                        if time >= args.repeat_under * 1000.0 || verdict == "refused" {
                            break;
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

/// The problems and configurations a CSV file has a row for, each as its
/// source, family, problem, mode, requested engine, jobs, test period,
/// bias and forward copy bound joined by commas; a file from before
/// the bias had a column ran them all with `auto`, and one from before the
/// forward bound had one ran the default bias as one search, which no
/// bound names, so every row of it is run again. A file with other
/// columns than [`HEADER`] is refused: the rows this run appends would
/// not fit it.
fn finished(path: &Path) -> Result<HashSet<String>> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Ok(HashSet::new());
    };
    let mut lines = text.lines();
    let header = lines.next().unwrap_or("");
    if header != HEADER {
        anyhow::bail!(
            "{} has other columns than this version writes; resume into a new file",
            path.display()
        );
    }
    let columns: Vec<&str> = header.split(',').collect();
    let at = |column: &str| columns.iter().position(|c| *c == column);
    let key = [
        "source",
        "family",
        "problem",
        "mode",
        "engine_requested",
        "jobs",
        "test_period",
    ]
    .map(at);
    let Some(key) = key.into_iter().collect::<Option<Vec<usize>>>() else {
        anyhow::bail!("{} is not a CSV file of `run`", path.display());
    };
    let bias = at("bias");
    let forward = at("forward_copies");
    // A file from before the bound ran without one.
    let memory = at("memory_limit");
    // A file from before the column ran every pool from its start.
    let pool = at("pool_after");
    Ok(lines
        .map(|line| {
            let fields: Vec<&str> = line.split(',').collect();
            let field = |i: usize| fields.get(i).copied().unwrap_or("");
            let bias = bias.map(field).filter(|b| !b.is_empty()).unwrap_or("auto");
            key.iter()
                .map(|&i| field(i))
                .chain([
                    bias,
                    forward.map_or("", field),
                    memory.map(field).filter(|m| !m.is_empty()).unwrap_or("0"),
                    pool.map_or("", field),
                ])
                .collect::<Vec<_>>()
                .join(",")
        })
        .collect())
}

/// Runs one problem in a child process and returns the tail of its row:
/// the child's own, or one that says how it died.
fn child(
    exe: &Path,
    reference: &Reference,
    mode: ModeChoice,
    engine: EngineChoice,
    jobs: usize,
    args: &RunArgs,
) -> Result<String> {
    let mut command = Command::new(exe);
    command
        .args(["one", "--problem", &reference.id])
        .args(["--mode", &name(&mode), "--engine", engine.name()])
        .args([
            "--jobs",
            &jobs.to_string(),
            "--timeout",
            &args.timeout.to_string(),
        ]);
    if let Some(copies) = args.copies {
        command.args(["--copies", &copies.to_string()]);
    }
    command.args(["--bias", &name(&args.bias)]);
    if let Some(copies) = args.forward_copies {
        command.args(["--forward-copies", &copies.to_string()]);
    }
    if let Some(work) = args.classical_work {
        command.args(["--classical-work", &work.to_string()]);
    }
    if let Some(bytes) = args.memory_limit {
        command.args(["--memory-limit", &bytes.to_string()]);
    }
    if let Some(limit) = args.recursion_limit {
        command.args(["--recursion-limit", &limit.to_string()]);
    }
    if let Some(period) = args.test_period {
        command.args(["--test-period", &period.to_string()]);
    }
    if let Some(alone) = args.pool_after {
        command.args(["--pool-after", &alone.to_string()]);
    }
    let mut process = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("starting a child")?;
    // The child's rows, and the moment it said its problem was loaded.
    let (loaded_at, loaded) = mpsc::channel();
    let pipe = process.stdout.take().expect("piped");
    let stdout = thread::spawn(move || {
        let mut rows = Vec::new();
        for line in BufReader::new(pipe).lines().map_while(Result::ok) {
            if line == LOADED {
                let _ = loaded_at.send(Instant::now());
            } else {
                rows.push(line);
            }
        }
        rows
    });
    let mut pipe = process.stderr.take().expect("piped");
    let stderr = thread::spawn(move || {
        let mut text = String::new();
        let _ = pipe.read_to_string(&mut text);
        text
    });

    let spawned = Instant::now();
    let mut searching = None;
    // Time for the pool's teardown and the proof check.
    let grace = args.grace.unwrap_or(args.timeout * 0.1 + 5.0);
    // A limit and grace past what a duration holds never kill.
    let limit = Duration::try_from_secs_f64(args.timeout + grace).unwrap_or(Duration::MAX);
    let load_limit = Duration::from_secs_f64(args.load_limit);
    let status = loop {
        if searching.is_none() {
            searching = loaded.try_recv().ok();
        }
        if let Some(status) = process.try_wait()? {
            break Some(status);
        }
        let over = match searching {
            Some(since) => since.elapsed() > limit,
            None => spawned.elapsed() > load_limit,
        };
        if over {
            process.kill()?;
            process.wait()?;
            break None;
        }
        thread::sleep(Duration::from_millis(2));
    };
    let rows = stdout.join().map_err(|_| anyhow!("reading a child"))?;
    let stderr = stderr.join().map_err(|_| anyhow!("reading a child"))?;
    let row = rows.iter().rev().find(|row| row.split(',').count() == TAIL);
    // A child that died after it printed its verdict died in the check of
    // its proof: the row keeps the verdict and says what became of the
    // check. One that died before leaves a row of its own.
    let died = |reason: String| match row {
        Some(row) => {
            let mut fields: Vec<&str> = row.split(',').collect();
            let checked = format!("failed: {reason}");
            fields[column("checked")] = &checked;
            fields.join(",")
        }
        None => {
            let mut fields = vec![String::new(); TAIL];
            fields[column("verdict")] = "unknown".to_owned();
            fields[column("reason")] = reason;
            fields[column("bias")] = name(&args.bias);
            fields[column("forward_copies")] = forward_copies(args.forward_copies).to_string();
            fields[column("memory_limit")] = memory_column(args.memory_limit);
            fields[column("pool_after")] = pool_column(args.pool_after);
            let since = searching.unwrap_or(spawned);
            fields[column("time_ms")] = format!("{:.3}", since.elapsed().as_secs_f64() * 1000.0);
            fields.join(",")
        }
    };
    Ok(match (status, row) {
        (Some(status), Some(row)) if status.success() => row.clone(),
        (None, _) if searching.is_none() => died("killed while loading".to_owned()),
        (None, _) => died("killed".to_owned()),
        (Some(status), _) => {
            // The row keeps the line that says why; the log gets all of
            // the error output, a panic's place and backtrace with it.
            eprintln!(
                "{} crashed ({status}); its error output:\n{}",
                reference.name,
                stderr.trim_end()
            );
            died(clean(&format!("crash ({status}): {}", cause(&stderr))))
        }
    })
}

/// Returns the line of a dead child's error output that says why it died:
/// the last one that is neither empty nor a note, such as the hint about
/// backtraces that follows a panic's message.
fn cause(stderr: &str) -> &str {
    stderr
        .lines()
        .rev()
        .map(str::trim)
        .find(|line| !line.is_empty() && !line.starts_with("note:"))
        .unwrap_or("")
}

/// Makes text a CSV field: no commas, no line breaks.
pub(crate) fn clean(text: &str) -> String {
    text.replace(',', ";").replace(['\n', '\r'], " ")
}

/// Runs one problem and prints the tail of its row, on a thread with a
/// stack large enough for the search at its recursion limit and for the
/// parser on a huge input.
pub fn one(args: OneArgs) -> Result<()> {
    let stack = Limits::default().stack_bytes().max(1 << 30);
    let line = thread::Builder::new()
        .stack_size(stack)
        .spawn(move || tail(&args))?
        .join()
        .map_err(|_| anyhow!("the child panicked"))?;
    say(&line);
    Ok(())
}

/// Prints a line for the parent at once.
fn say(line: &str) {
    let mut out = std::io::stdout().lock();
    let _ = writeln!(out, "{line}");
    let _ = out.flush();
}

/// Loads and runs one problem and returns the tail of its row; on the
/// way it says when the problem is loaded, and prints the tail of a
/// proved problem once before the proof is checked.
fn tail(args: &OneArgs) -> String {
    let problem = match problems::load(&args.problem) {
        Ok(problem) => problem,
        Err(error) => {
            return row(&[
                ("verdict", "error"),
                ("reason", &clean(&format!("{error:#}"))),
            ]);
        }
    };
    let mode = args.mode.apply(problem.mode);
    let copies = match args.copies {
        Some(Bound(bound)) => bound,
        None => Some(problem.copies.unwrap_or(Options::DEFAULT_COPIES)),
    };
    let bound = Bound(copies).to_string();
    let recursion = args
        .recursion_limit
        .unwrap_or(Limits::DEFAULT_RECURSION_DEPTH);
    // A known verdict holds in the problem's own mode only: classical
    // linear logic proves more than intuitionistic, affine more than linear.
    let expected = match problem.expected.filter(|_| mode == problem.mode) {
        Some(true) => "provable",
        Some(false) => "unprovable",
        None => "",
    };
    let (occurrences, multiplicity) = match Forest::new(&problem.sequent) {
        Ok(forest) => (forest.len(), multiplicity(&forest)),
        Err(_) => (0, 0),
    };
    let options = Options::default()
        .with_engine(args.engine.0)
        .with_jobs(args.jobs)
        .with_copies(copies)
        .with_bias(args.bias.bias())
        .with_forward_copies(forward_copies(args.forward_copies))
        .with_test_period(args.test_period)
        .with_classical_work(
            args.classical_work
                .unwrap_or(Options::DEFAULT_CLASSICAL_WORK),
        )
        // The check is the child's own, outside the time measured.
        .with_check(false);
    let limits = Limits::default()
        .with_memory_bytes(memory_limit(args.memory_limit))
        .with_recursion_depth(recursion);
    say(LOADED);

    // The limit is a flag that a thread of its own raises, so that a poll
    // costs one load whatever the engine's cadence: a clock read every so
    // many polls is late by that many, which on a large net, where a poll
    // comes many milliseconds after the last, was seconds.
    let expired = Arc::new(AtomicBool::new(false));
    let limit = Duration::from_secs_f64(args.timeout);
    let (cpu_before, wait_before) = (cpu_ms(), wait_ms());
    let start = Instant::now();
    {
        let expired = Arc::clone(&expired);
        thread::spawn(move || {
            thread::sleep(limit);
            expired.store(true, Ordering::Relaxed);
        });
    }
    let outcome = decide(&problem.sequent, mode, &options, &limits, args, &expired);
    let time = start.elapsed().as_secs_f64() * 1000.0;
    // The time the search took on the CPUs, and the time its thread was
    // ready but waited for one: a run another process slowed down has the
    // latter well above zero.
    let since = |after: Option<f64>, before: Option<f64>| {
        after
            .zip(before)
            .map_or(String::new(), |(a, b)| format!("{:.3}", a - b))
    };
    let (cpu, wait) = (since(cpu_ms(), cpu_before), since(wait_ms(), wait_before));

    let outcome = match outcome {
        Ok(outcome) => outcome,
        Err(error) => {
            let verdict = match error {
                Error::EngineRefused { .. } | Error::IntuitionisticMix => "refused",
                _ => "error",
            };
            return row(&[
                ("copies", &bound),
                ("expected", expected),
                ("verdict", verdict),
                ("reason", &clean(&error.to_string())),
                ("occurrences", &occurrences.to_string()),
                ("multiplicity", &multiplicity.to_string()),
                ("recursion_limit", &recursion.to_string()),
                ("bias", &name(&args.bias)),
                (
                    "forward_copies",
                    &forward_copies(args.forward_copies).to_string(),
                ),
                ("memory_limit", &memory_column(args.memory_limit)),
                ("pool_after", &pool_column(args.pool_after)),
            ]);
        }
    };
    let (verdict, reason) = match &outcome.verdict {
        Verdict::Proved(_) => ("proved", ""),
        Verdict::Unprovable(_) => ("unprovable", ""),
        Verdict::Unknown(reason) => {
            // The harness's stop is its time limit.
            let reason = match reason {
                Reason::Stopped => "timeout",
                reason => reason.name(),
            };
            ("unknown", reason)
        }
    };
    let s = outcome.statistics;
    let tail = |checked: &str, check_ms: &str| {
        [
            bound.clone(),
            expected.to_owned(),
            verdict.to_owned(),
            reason.to_owned(),
            checked.to_owned(),
            outcome.engine.to_string(),
            outcome.fragment.name_in(mode).to_owned(),
            occurrences.to_string(),
            multiplicity.to_string(),
            format!("{time:.3}"),
            s.nodes.to_string(),
            s.memo_hits.to_string(),
            s.memo_entries.to_string(),
            s.splits.to_string(),
            s.links.to_string(),
            s.tests.to_string(),
            recursion.to_string(),
            cpu.clone(),
            wait.clone(),
            name(&args.bias),
            forward_copies(args.forward_copies).to_string(),
            check_ms.to_owned(),
            memory_column(args.memory_limit),
            s.copies.to_string(),
            pool_column(args.pool_after),
            s.forward_copies.to_string(),
            s.work.to_string(),
        ]
        .into_iter()
        // The columns of a row of `run --tools`: tool, tool_version,
        // translate_ms, peak_bytes and raw.
        .chain(std::iter::repeat_n(String::new(), 5))
        .collect::<Vec<_>>()
        .join(",")
    };
    let proof = match &outcome.verdict {
        Verdict::Proved(proof) => proof,
        // A refutation with a certificate is checked as a proof is; one
        // that rests on the search has nothing to check.
        Verdict::Unprovable(disproof) => {
            return match disproof.check() {
                Ok(()) => tail("ok", ""),
                Err(error) if error.kind() == linlog::ErrorKind::Unsupported => tail("", ""),
                Err(error) => tail(&clean(&format!("failed: {error}")), ""),
            };
        }
        Verdict::Unknown(_) => return tail("", ""),
    };
    // The verdict is out before the check, which a proof can outgrow.
    say(&tail("", ""));
    let start = Instant::now();
    let checked = match proof.check_within(mode, &limits, |_| false) {
        Ok(()) => "ok".to_owned(),
        // Given up within the run's memory bound: no verdict on the proof.
        Err(linlog::CheckError::Refused(_)) => "unchecked: memory limit".to_owned(),
        Err(error) => clean(&format!("failed: {error}")),
    };
    let check = start.elapsed().as_secs_f64() * 1000.0;
    tail(&checked, &format!("{check:.3}"))
}

/// Decides a problem under the options until the flag is raised: with
/// the options' threads from the start, or with `--pool-after` the
/// library's race, one thread first and, once the time has passed, a pool
/// of the other threads beside it, the first to decide answering, both
/// within one memory bound, as the command does by default. The outcome
/// of two searches has the counters of both.
fn decide(
    sequent: &linlog::Sequent,
    mode: linlog::Mode,
    options: &Options,
    limits: &Limits,
    args: &OneArgs,
    expired: &AtomicBool,
) -> Result<linlog::search::Outcome, Error> {
    // The refutation after the search is bounded by its own work and
    // answers a search the time limit ended, as the command's does.
    let stop = |progress: linlog::Progress| {
        progress.phase != linlog::Phase::Refute && expired.load(Ordering::Relaxed)
    };
    let Some(alone) = args.pool_after.filter(|_| args.jobs > 1) else {
        return prove_within(sequent, mode, options, limits, stop);
    };
    let forest = Forest::within(sequent, limits)?;
    let alone = Duration::from_secs_f64(alone);
    let start = Instant::now();
    linlog::search::race(
        linlog::Goal::conclusion(&forest),
        mode,
        options,
        limits,
        args.jobs,
        |_| start.elapsed() >= alone,
        stop,
    )
}

/// A tail with the given fields filled in and the others empty.
fn row(filled: &[(&str, &str)]) -> String {
    let mut fields = vec![""; TAIL];
    for &(name, value) in filled {
        fields[column(name)] = value;
    }
    fields.join(",")
}

/// The CPU time of the process so far, in milliseconds: the user and system
/// time of all its threads, dead ones included, from `/proc/self/stat`, in
/// the kernel's ticks of 10 ms; `None` where there is no such file.
fn cpu_ms() -> Option<f64> {
    let stat = std::fs::read_to_string("/proc/self/stat").ok()?;
    // After the command in parentheses, utime and stime are the 12th and
    // 13th fields.
    let mut fields = stat.rsplit_once(')')?.1.split_whitespace().skip(11);
    let user: f64 = fields.next()?.parse().ok()?;
    let system: f64 = fields.next()?.parse().ok()?;
    Some((user + system) * 10.0)
}

/// How long the calling thread has been ready to run but waiting for a CPU,
/// in milliseconds, from `/proc/thread-self/schedstat`; `None` where there
/// is no such file.
fn wait_ms() -> Option<f64> {
    let stat = std::fs::read_to_string("/proc/thread-self/schedstat").ok()?;
    let nanoseconds: f64 = stat.split_whitespace().nth(1)?.parse().ok()?;
    Some(nanoseconds / 1e6)
}

/// The most occurrences of one literal, `a` or `~a`, in the forest: what
/// the dispatch compares with its threshold for the net engine.
fn multiplicity(forest: &Forest) -> usize {
    (0..forest.sequent().atom_names().len() as u32)
        .map(|a| {
            let atom = Atom::new(a);
            forest
                .literals(atom, Sign::Atom)
                .len()
                .max(forest.literals(atom, Sign::Dual).len())
        })
        .max()
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::{HEADER, Shard, finished};

    /// Every shard of a list, joined, is the list, each problem once, and
    /// each keeps the list's order; a shard outside 1 to N is refused.
    #[test]
    fn shards_partition_the_problems_in_order() {
        let problems: Vec<u32> = (0..23).collect();
        for n in [1, 2, 5, 23, 30] {
            let mut joined: Vec<u32> = (1..=n)
                .flat_map(|k| {
                    format!("{k}/{n}")
                        .parse::<Shard>()
                        .unwrap()
                        .of(problems.clone())
                })
                .collect();
            let first = format!("1/{n}")
                .parse::<Shard>()
                .unwrap()
                .of(problems.clone());
            assert!(
                first.windows(2).all(|w| w[0] < w[1]),
                "a shard keeps the order"
            );
            joined.sort_unstable();
            assert_eq!(joined, problems, "{n} shards take every problem once");
        }
        for bad in ["0/3", "4/3", "3", "a/b"] {
            assert!(bad.parse::<Shard>().is_err(), "{bad}");
        }
    }

    /// `--resume` takes no file of other columns, and a run's key from a
    /// file of its own: what it ran and the options a later column added,
    /// at their defaults where empty.
    #[test]
    fn resume_reads_its_own_files_only() {
        let dir = std::env::temp_dir().join(format!("linlog-resume-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        assert!(finished(&dir.join("absent.csv")).unwrap().is_empty());
        let foreign = dir.join("foreign.csv");
        std::fs::write(&foreign, "source,family,problem\nf,a,b\n").unwrap();
        let error = finished(&foreign).unwrap_err().to_string();
        assert!(error.contains("other columns"), "{error}");
        let row: Vec<&str> = HEADER
            .split(',')
            .map(|column| match column {
                "source" => "family",
                "family" => "chain",
                "problem" => "chain/3",
                "mode" => "classical",
                "engine_requested" => "auto",
                "jobs" => "1",
                _ => "",
            })
            .collect();
        let own = dir.join("own.csv");
        std::fs::write(&own, format!("{HEADER}\n{}\n", row.join(","))).unwrap();
        let keys = finished(&own).unwrap();
        assert_eq!(
            keys.into_iter().collect::<Vec<_>>(),
            ["family,chain,chain/3,classical,auto,1,,auto,,0,"]
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
