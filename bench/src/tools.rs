// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Provers side by side: every problem run by each tool of a list, linlog's
//! command among them, each as a process of its own measured by BenchExec's
//! `runexec` (wall time, CPU time and peak memory of the process and its
//! children, in a cgroup with the memory limit), with the same time and
//! memory limit for every tool. A tool is a [`Driver`]: whether it covers a
//! problem's fragment and mode, how the problem is written for it, its
//! command line, and how its verdict is read. A problem outside a tool's
//! fragment gets a row of its own, verdict `refused` and reason `fragment`,
//! and is not run.

use crate::problems::{self, Problem, Reference, mode_name};
use crate::run::HEADER;
use crate::translate;
use anyhow::{Context, Result, anyhow, bail};
use linlog::{Fragment, Mode};
use std::collections::HashSet;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

/// How a driver hands a problem to its tool.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Input {
    /// linlog's command: an LLTP or `.spec` file as it is, anything else
    /// as the sequent's JSON.
    Linlog,
    /// A file the translator writes, with this extension.
    Translated(&'static str),
}

/// One tool: its name in the CSV, the program on the path that runs it,
/// what it covers, how it reads a problem and how its answer is read.
pub struct Driver {
    /// The name of the `tool` column and of `--tools`.
    pub name: &'static str,
    /// The program, found on the path.
    pub program: &'static str,
    /// What the tool is, for the record of a run.
    pub summary: &'static str,
    /// Whether it runs a problem of this fragment in this mode.
    pub covers: fn(Fragment, Mode) -> bool,
    /// How the problem reaches it.
    pub input: Input,
    /// The problem in the tool's own syntax, for a translated input.
    pub translate: fn(&Problem) -> Result<String>,
    /// The arguments after the program, given the input file, the time
    /// limit in seconds and the memory limit in bytes.
    pub arguments: fn(&Path, &Problem, f64, u64) -> Vec<String>,
    /// The verdict, its reason and the engine, from the tool's output and
    /// exit status on the problem.
    pub read: fn(&str, i32, &Problem) -> Answer,
    /// Threads the tool runs: the `jobs` column.
    pub jobs: &'static str,
    /// Whether the tool checks a proof before it reports it.
    pub checks: bool,
}

/// A tool's answer on one problem.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Answer {
    /// `proved`, `unprovable`, `unknown` or `error`.
    pub verdict: &'static str,
    /// Why it is unknown or an error, else empty.
    pub reason: String,
    /// The engine that decided, where the tool names one.
    pub engine: String,
}

impl Answer {
    /// An answer with a verdict and nothing else.
    pub fn plain(verdict: &'static str, reason: &str) -> Self {
        Self {
            verdict,
            reason: reason.to_owned(),
            engine: String::new(),
        }
    }
}

/// Whether a mode is plain intuitionistic linear logic, neither affine nor
/// with Mix.
fn intuitionistic(mode: Mode) -> bool {
    mode.is_intuitionistic() && !mode.is_affine() && !mode.has_mix()
}

/// Whether a mode is plain classical linear logic.
fn classical(mode: Mode) -> bool {
    !mode.is_intuitionistic() && !mode.is_affine() && !mode.has_mix()
}

/// linlog covers every problem the harness has.
const fn everything(_: Fragment, _: Mode) -> bool {
    true
}

/// linlog reads its input itself.
fn untranslated(_: &Problem) -> Result<String> {
    Ok(String::new())
}

/// The arguments of `linlog prove` common to both of linlog's columns.
fn linlog_arguments(file: &Path, problem: &Problem, timeout: f64, memory: u64) -> Vec<String> {
    let format = match file.extension().and_then(|e| e.to_str()) {
        Some("p") => "lltp",
        Some("spec") => "spec",
        _ => "json",
    };
    let mut arguments: Vec<String> = ["prove", "--file"]
        .into_iter()
        .map(String::from)
        .chain([file.display().to_string()])
        .chain(["--input-format".into(), format.into()])
        .collect();
    let mode = problem.mode;
    for (on, flag) in [
        (mode.is_intuitionistic(), "--intuitionistic"),
        (mode.is_affine(), "--affine"),
        (mode.has_mix(), "--mix"),
    ] {
        if on {
            arguments.push(flag.into());
        }
    }
    arguments.extend([
        "--timeout".into(),
        format!("{timeout}s"),
        "--memory-limit".into(),
        format!("{memory}B"),
        "--quiet".into(),
    ]);
    arguments
}

/// linlog on one thread: its sequential engines, the counters a function
/// of the input.
fn linlog_one(file: &Path, problem: &Problem, timeout: f64, memory: u64) -> Vec<String> {
    let mut arguments = linlog_arguments(file, problem, timeout, memory);
    arguments.push("--deterministic".into());
    arguments
}

/// Reads linlog's exit status (0 proved, 1 unprovable, 3 unknown, 2 an
/// error) and the engine from its verdict line, `provable (MLL,
/// classical, focused engine)`.
fn linlog_read(output: &str, status: i32, _: &Problem) -> Answer {
    let line = output.lines().find(|l| !l.trim().is_empty()).unwrap_or("");
    let engine = line
        .rsplit_once(", ")
        .and_then(|(_, rest)| rest.split_once(" engine"))
        .map_or(String::new(), |(engine, _)| engine.to_owned());
    let (verdict, reason) = match status {
        0 => ("proved", String::new()),
        1 => ("unprovable", String::new()),
        3 => {
            // The verdict line says in words which bound stopped the search.
            let why = line.split_once("): ").map_or("", |(_, why)| why);
            let reason = [
                ("time limit", "timeout"),
                ("memory", "memory_limit"),
                ("copy bound", "copy_bound"),
                ("recursion", "recursion_limit"),
                ("work limit", "work_limit"),
                ("index", "index_limit"),
            ]
            .iter()
            .find(|(words, _)| why.contains(words))
            .map_or(why, |&(_, reason)| reason);
            ("unknown", reason.to_owned())
        }
        _ => ("error", line.to_owned()),
    };
    Answer {
        verdict,
        reason,
        engine,
    }
}

/// Every driver, linlog's two columns first.
pub const DRIVERS: &[Driver] = &[
    Driver {
        name: "linlog",
        program: "linlog",
        summary: "linlog prove --deterministic: one thread",
        covers: everything,
        input: Input::Linlog,
        translate: untranslated,
        arguments: linlog_one,
        read: linlog_read,
        jobs: "1",
        checks: true,
    },
    Driver {
        name: "linlog-default",
        program: "linlog",
        summary: "linlog prove as a user runs it: one thread, then a pool beside it",
        covers: everything,
        input: Input::Linlog,
        translate: untranslated,
        arguments: linlog_arguments,
        read: linlog_read,
        jobs: "all",
        checks: true,
    },
    Driver {
        name: "click-and-collect",
        program: "click-and-collect",
        summary: "Click & coLLecT's automatic prover (focused search, exponential bound \
                  raised step by step), called as its web server's handler calls it",
        covers: |_, mode| classical(mode),
        input: Input::Translated("txt"),
        translate: translate::click_and_collect,
        arguments: |file, _, timeout, _| {
            vec![
                file.display().to_string(),
                "--seconds".into(),
                timeout.to_string(),
            ]
        },
        read: |output, status, _| match status {
            0 => Answer::plain("proved", ""),
            1 => Answer::plain("unprovable", ""),
            3 => Answer::plain("unknown", "timeout"),
            _ => Answer::plain("error", output.lines().next().unwrap_or("")),
        },
        jobs: "1",
        checks: false,
    },
    Driver {
        name: "maude-ill",
        program: "maude-ll",
        summary: "the Maude prover's focused system for ILL, its solve under the copy \
                  bound of its own script for the LLTP library",
        covers: |_, mode| intuitionistic(mode),
        input: Input::Translated("maude"),
        translate: translate::maude_ill,
        arguments: |file, _, _, _| vec!["ill".into(), file.display().to_string()],
        read: |output, _, problem| {
            if output.contains("result Bool: true") {
                Answer::plain("proved", "")
            } else if output.contains("result Bool: false") {
                // Exhaustive only where no `!` can be copied.
                if problem.sequent.fragment().has_exponentials() {
                    Answer::plain("unknown", "copy_bound")
                } else {
                    Answer::plain("unprovable", "")
                }
            } else {
                Answer::plain("error", maude_error(output))
            }
        },
        jobs: "1",
        checks: false,
    },
    Driver {
        name: "maude-cll",
        program: "maude-ll",
        summary: "the Maude prover's focused system for classical LL, a search for a \
                  proof without a bound",
        covers: |_, mode| classical(mode),
        input: Input::Translated("maude"),
        translate: translate::maude_cll,
        arguments: |file, _, _, _| vec!["cll".into(), file.display().to_string()],
        read: |output, _, _| {
            if output.contains("Solution 1") {
                Answer::plain("proved", "")
            } else if output.contains("No solution") {
                Answer::plain("unprovable", "")
            } else {
                Answer::plain("error", maude_error(output))
            }
        },
        jobs: "1",
        checks: false,
    },
    Driver {
        name: "llprover-cll",
        program: "llprover",
        summary: "llprover's classical system cll(full), its default threshold of 5 cuts \
                  and contractions per path",
        covers: |_, mode| classical(mode),
        input: Input::Translated("ll"),
        translate: translate::llprover_cll,
        arguments: prolog_arguments,
        read: llprover_read,
        jobs: "1",
        checks: false,
    },
    Driver {
        name: "llprover-ill",
        program: "llprover",
        summary: "llprover's intuitionistic system ill(full), its default threshold of 5 \
                  cuts and contractions per path",
        covers: |_, mode| intuitionistic(mode),
        input: Input::Translated("ll"),
        translate: translate::llprover_ill,
        arguments: prolog_arguments,
        read: llprover_read,
        jobs: "1",
        checks: false,
    },
    Driver {
        name: "lintap",
        program: "lintap",
        summary: "linTAP on the ⅋ of the sequent's formulas, its exponential limit raised \
                  without end as its prove/1 does",
        covers: |fragment, mode| {
            classical(mode) && !fragment.has_additives() && !fragment.has_additive_units()
        },
        input: Input::Translated("pl"),
        translate: translate::lintap,
        arguments: prolog_arguments,
        read: |output, status, _| match status {
            _ if prolog_out_of_memory(output) => Answer::plain("unknown", "out_of_memory"),
            0 => Answer::plain("proved", ""),
            1 => Answer::plain("unprovable", ""),
            _ => Answer::plain("error", output.lines().next().unwrap_or("")),
        },
        jobs: "1",
        checks: false,
    },
    Driver {
        name: "linearone",
        program: "linearone",
        summary: "LinearOne's first_proof on the sequent, multiplicative intuitionistic",
        covers: |fragment, mode| intuitionistic(mode) && Fragment::MLL.contains(fragment),
        input: Input::Translated("pl"),
        translate: translate::linearone,
        arguments: prolog_arguments,
        read: |output, status, _| match status {
            _ if prolog_out_of_memory(output) => Answer::plain("unknown", "out_of_memory"),
            0 => Answer::plain("proved", ""),
            1 => Answer::plain("unprovable", ""),
            _ => Answer::plain("error", output.lines().last().unwrap_or("")),
        },
        jobs: "1",
        checks: false,
    },
    Driver {
        name: "sympli",
        program: "sympli",
        summary: "Sympli's inverse method on (⊗Γ) ⊸ A, its proofs validated by its own \
                  checker",
        covers: |_, mode| intuitionistic(mode),
        input: Input::Translated("sym"),
        translate: translate::sympli,
        arguments: |file, _, _, _| vec![file.display().to_string()],
        read: |output, _, _| {
            if output.contains("certificate OK") {
                Answer::plain("proved", "")
            } else if output.contains("ABORT: Saturated") {
                Answer::plain("unprovable", "")
            } else {
                let error = output.lines().find(|l| l.to_lowercase().contains("error"));
                Answer::plain("error", error.unwrap_or(""))
            }
        },
        jobs: "1",
        checks: true,
    },
    Driver {
        name: "lolli",
        program: "lolli",
        summary: "lolli's focused search under its default depth of 100",
        covers: |_, mode| classical(mode),
        input: Input::Translated("txt"),
        translate: translate::lolli,
        arguments: |file, _, _, _| vec![file.display().to_string()],
        read: |output, _, _| {
            if output.contains("NOT PROVABLE") {
                Answer::plain("unknown", "depth")
            } else if output.contains("PROVABLE") {
                Answer::plain("proved", "")
            } else {
                Answer::plain("error", output.lines().next().unwrap_or(""))
            }
        },
        jobs: "1",
        checks: false,
    },
    Driver {
        name: "apll-ll",
        program: "apll",
        summary: "APLL's focused backward search for LL under its default bound of 3 \
                  contractions",
        covers: |_, mode| classical(mode),
        input: Input::Translated("txt"),
        translate: translate::apll_ll,
        arguments: |file, _, _, _| vec!["-t".into(), file.display().to_string()],
        read: apll_read,
        jobs: "1",
        checks: false,
    },
    Driver {
        name: "apll-ill",
        program: "apll",
        summary: "APLL's focused backward search for ILL under its default bound of 3 \
                  contractions",
        covers: |_, mode| intuitionistic(mode),
        input: Input::Translated("txt"),
        translate: translate::apll_ill,
        arguments: |file, _, _, _| vec!["-ill".into(), "-t".into(), file.display().to_string()],
        read: apll_read,
        jobs: "1",
        checks: false,
    },
];

/// Reads APLL's answer, the line before its time: `P` provable, `N`
/// refuted by a search that never met its bound, `-` the bound met.
fn apll_read(output: &str, _: i32, _: &Problem) -> Answer {
    let mut lines = output.lines().rev().filter(|l| !l.trim().is_empty());
    let _time = lines.next();
    match lines.next().map(str::trim) {
        Some("P") => Answer::plain("proved", ""),
        Some("N") => Answer::plain("unprovable", ""),
        Some("-") => Answer::plain("unknown", "bound"),
        _ => Answer::plain("error", output.lines().next().unwrap_or("")),
    }
}

/// The arguments of a prover on SWI-Prolog: the file, and the memory
/// limit as its stacks' limit, which is 1 GiB unless a run names another.
fn prolog_arguments(file: &Path, _: &Problem, _: f64, memory: u64) -> Vec<String> {
    vec![file.display().to_string(), memory.to_string()]
}

/// Whether SWI-Prolog stopped at its stacks' limit.
fn prolog_out_of_memory(output: &str) -> bool {
    output.contains("Stack limit") || output.contains("resource_error")
}

/// Reads llprover's verdict line: a failure is a refutation only where no
/// `!` or `?` could be contracted beyond its threshold.
fn llprover_read(output: &str, _: i32, problem: &Problem) -> Answer {
    if prolog_out_of_memory(output) {
        Answer::plain("unknown", "out_of_memory")
    } else if output.contains("Succeed in proving") {
        Answer::plain("proved", "")
    } else if output.contains("Fail to prove") {
        if problem.sequent.fragment().has_exponentials() {
            Answer::plain("unknown", "threshold")
        } else {
            Answer::plain("unprovable", "")
        }
    } else {
        let error = output.lines().find(|l| l.to_lowercase().contains("error"));
        Answer::plain("error", error.unwrap_or(""))
    }
}

/// Maude's first warning or error line, or its output's first line.
fn maude_error(output: &str) -> &str {
    let mut lines = output.lines().filter(|l| !l.trim().is_empty());
    output
        .lines()
        .find(|l| l.starts_with("Warning") || l.starts_with("Error"))
        .or_else(|| lines.next())
        .unwrap_or("")
}

/// Returns the driver of a tool's name.
pub fn driver(name: &str) -> Result<&'static Driver> {
    DRIVERS.iter().find(|d| d.name == name).ok_or_else(|| {
        let names: Vec<&str> = DRIVERS.iter().map(|d| d.name).collect();
        anyhow!("no tool `{name}`; the tools are {}", names.join(", "))
    })
}

/// What `runexec` measured of one run.
#[derive(Debug, Default)]
struct Measured {
    /// Wall time in milliseconds.
    wall_ms: f64,
    /// CPU time in milliseconds, children included.
    cpu_ms: f64,
    /// The cgroup's peak memory in bytes.
    peak_bytes: u64,
    /// The exit status, or the signal that ended it as 128 and more.
    status: i32,
    /// Why runexec ended it: `walltime`, `memory`, …, empty when it ended
    /// by itself.
    terminated: String,
}

/// Reads `runexec`'s `key=value` lines.
fn measured(stdout: &str) -> Measured {
    let mut m = Measured::default();
    // runexec writes durations as seconds with an `s`, memory with a `B`.
    let number = |v: &str| v.trim_end_matches(['s', 'B']).parse::<f64>().unwrap_or(0.0);
    for (key, value) in stdout.lines().filter_map(|l| l.split_once('=')) {
        match key {
            "walltime" => m.wall_ms = number(value) * 1000.0,
            "cputime" => m.cpu_ms = number(value) * 1000.0,
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "a count of bytes runexec wrote as an integer"
            )]
            "memory" => m.peak_bytes = number(value) as u64,
            "returnvalue" => m.status = value.parse().unwrap_or(-1),
            "exitsignal" => m.status = 128 + value.parse::<i32>().unwrap_or(0),
            "terminationreason" => value.clone_into(&mut m.terminated),
            _ => {}
        }
    }
    m
}

/// The tool's own output in runexec's log: what follows the command line
/// and the line of dashes runexec writes first.
fn tool_output(log: &str) -> &str {
    let dashes = "-".repeat(80);
    log.split_once(dashes.as_str())
        .map_or(log, |(_, rest)| rest.trim_start_matches('\n'))
}

/// The settings of a comparison run that every row and the record name.
pub struct Settings<'a> {
    /// The tools, in the order each problem runs them.
    pub drivers: Vec<&'static Driver>,
    /// The time limit in seconds, the same for every tool.
    pub timeout: f64,
    /// The memory limit in bytes, the same for every tool.
    pub memory: u64,
    /// Seconds a tool may run past the limit before runexec ends it, so
    /// that linlog's own stop answers first.
    pub grace: f64,
    /// Where the inputs and logs of a run are written, emptied after it.
    pub work: &'a Path,
    /// Each tool's version, from the record of the tools the run was
    /// given.
    pub versions: std::collections::HashMap<String, String>,
}

/// Runs every problem with every tool and writes a row per pair into
/// `out`; the keys in `done` (source, family, problem, mode, tool) are
/// skipped.
pub fn run(
    references: &[Reference],
    settings: &Settings,
    done: &HashSet<String>,
    out: &mut dyn Write,
) -> Result<()> {
    std::fs::create_dir_all(settings.work)?;
    let total = references.len() * settings.drivers.len();
    let start = Instant::now();
    let (mut count, mut ran) = (0, 0);
    for reference in references {
        let key = |tool: &str| {
            [
                reference.source,
                &reference.family,
                &reference.name,
                mode_name(reference.mode),
                tool,
            ]
            .join(",")
        };
        if settings.drivers.iter().all(|d| done.contains(&key(d.name))) {
            count += settings.drivers.len();
            continue;
        }
        let problem = problems::load(&reference.id);
        for driver in &settings.drivers {
            count += 1;
            if done.contains(&key(driver.name)) {
                continue;
            }
            ran += 1;
            let row = match &problem {
                Ok(problem) => one(reference, problem, driver, settings)?,
                Err(error) => vec![
                    ("verdict", "error".into()),
                    ("reason", crate::run::clean(&format!("{error:#}"))),
                ],
            };
            let line = full_row(reference, driver, settings, &row);
            writeln!(out, "{line}")?;
            out.flush()?;
            let get = |name: &str| {
                row.iter()
                    .find(|(n, _)| *n == name)
                    .map_or("", |(_, v)| v.as_str())
            };
            let left = start.elapsed().as_secs_f64() / f64::from(ran) * (total - count) as f64;
            eprintln!(
                "[{count}/{total}] {}/{} {}: {} {} {} ms, about {:.0} min left",
                reference.family,
                reference.name,
                driver.name,
                get("verdict"),
                get("reason"),
                get("time_ms"),
                left / 60.0,
            );
        }
    }
    Ok(())
}

/// Runs one problem with one tool and returns the row's fields that
/// depend on the run.
fn one(
    reference: &Reference,
    problem: &Problem,
    driver: &Driver,
    settings: &Settings,
) -> Result<Vec<(&'static str, String)>> {
    let fragment = problem.sequent.fragment();
    let expected = match problem.expected {
        Some(true) => "provable",
        Some(false) => "unprovable",
        None => "",
    };
    let mut row = vec![
        ("expected", expected.to_owned()),
        ("fragment", fragment.name_in(problem.mode).to_string()),
    ];
    row.push(("occurrences", problem.sequent.occurrences().to_string()));
    if !(driver.covers)(fragment, problem.mode) {
        row.extend([("verdict", "refused".into()), ("reason", "fragment".into())]);
        return Ok(row);
    }
    let dir = settings.work.join(driver.name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir)?;
    }
    std::fs::create_dir_all(&dir)?;
    let started = Instant::now();
    let input = match driver.input {
        Input::Linlog if matches!(reference.source, "lltp" | "spec") => {
            PathBuf::from(reference.id.split_once(':').map_or("", |(_, path)| path))
        }
        Input::Linlog => {
            let file = dir.join("problem.json");
            std::fs::write(&file, serde_json::to_string(&problem.sequent)?)?;
            file
        }
        Input::Translated(extension) => match (driver.translate)(problem) {
            Ok(text) => {
                let file = dir.join(format!("problem.{extension}"));
                std::fs::write(&file, text)?;
                file
            }
            Err(error) => {
                row.extend([
                    ("verdict", "refused".into()),
                    (
                        "reason",
                        crate::run::clean(&format!("translation: {error:#}")),
                    ),
                ]);
                return Ok(row);
            }
        },
    };
    let translate_ms = started.elapsed().as_secs_f64() * 1000.0;
    let input = std::path::absolute(&input)?;
    let arguments = (driver.arguments)(&input, problem, settings.timeout, settings.memory);
    let log = dir.join("output.log");
    let result = Command::new("runexec")
        .args(["--no-container", "--quiet", "--walltimelimit"])
        .arg(format!("{}", (settings.timeout + settings.grace).ceil()))
        .arg("--memlimit")
        .arg(settings.memory.to_string())
        .arg("--output")
        .arg(&log)
        .arg("--dir")
        .arg(&dir)
        .arg("--")
        .arg(driver.program)
        .args(&arguments)
        .output()
        .context("starting runexec, which BenchExec provides")?;
    let m = measured(&String::from_utf8_lossy(&result.stdout));
    let log = std::fs::read_to_string(&log).unwrap_or_default();
    if !result.status.success() || m.wall_ms == 0.0 && m.terminated.is_empty() {
        bail!(
            "runexec failed on {} with {}: {}",
            reference.id,
            driver.name,
            String::from_utf8_lossy(&result.stderr).trim()
        );
    }
    let answer = match m.terminated.as_str() {
        "" => (driver.read)(tool_output(&log), m.status, problem),
        "memory" => Answer::plain("unknown", "out_of_memory"),
        "walltime" | "cputime" => Answer::plain("unknown", "timeout"),
        other => Answer::plain("unknown", other),
    };
    let raw = tool_output(&log)
        .lines()
        .rev()
        .find(|l| !l.trim().is_empty())
        .unwrap_or("");
    let checked = match (answer.verdict, driver.checks) {
        ("proved", true) => "ok",
        ("proved", false) => "unchecked: no checker",
        _ => "",
    };
    row.extend([
        ("verdict", answer.verdict.into()),
        ("reason", crate::run::clean(&answer.reason)),
        ("checked", checked.into()),
        ("engine", answer.engine),
        ("time_ms", format!("{:.1}", m.wall_ms)),
        ("cpu_ms", format!("{:.1}", m.cpu_ms)),
        ("translate_ms", format!("{translate_ms:.1}")),
        ("peak_bytes", m.peak_bytes.to_string()),
        (
            "raw",
            crate::run::clean(raw.chars().take(120).collect::<String>().trim()),
        ),
    ]);
    std::fs::remove_dir_all(&dir)?;
    Ok(row)
}

/// The whole CSV row of one run: the problem's columns, the driver's and
/// the run's.
fn full_row(
    reference: &Reference,
    driver: &Driver,
    settings: &Settings,
    filled: &[(&str, String)],
) -> String {
    let problem = [
        ("source", reference.source.to_owned()),
        ("family", reference.family.clone()),
        (
            "size",
            reference.size.map_or(String::new(), |s| s.to_string()),
        ),
        (
            "index",
            reference.index.map_or(String::new(), |i| i.to_string()),
        ),
        ("problem", reference.name.clone()),
        ("mode", mode_name(reference.mode).to_owned()),
        ("jobs", driver.jobs.to_owned()),
        ("timeout_s", settings.timeout.to_string()),
        ("run", "0".into()),
        ("memory_limit", settings.memory.to_string()),
        ("tool", driver.name.to_owned()),
        (
            "tool_version",
            settings
                .versions
                .get(driver.name)
                .cloned()
                .unwrap_or_default(),
        ),
    ];
    HEADER
        .split(',')
        .map(|column| {
            filled
                .iter()
                .chain(problem.iter())
                .find(|(name, _)| *name == column)
                .map_or("", |(_, value)| value.as_str())
        })
        .collect::<Vec<_>>()
        .join(",")
}

/// The memory limit of every tool when `--memory-limit` names none.
pub const DEFAULT_MEMORY: u64 = 4 << 30;

/// Seconds past the time limit before runexec ends a run: linlog stops
/// itself at the limit and needs a moment to say so.
const GRACE: f64 = 1.0;

/// The environment variable naming the record of the tools' versions, a
/// JSON object from each tool's name to its version, source and licence,
/// which the flake writes beside the tools.
const TOOLS_RECORD: &str = "LINLOG_COMPARISON_TOOLS";

/// `run --tools`: runs the problems listed with every tool named, writes
/// the rows to the output (appending and resuming as `run` does) and the
/// record of the run beside it, `NAME.env.json`.
pub fn main(args: &crate::RunArgs, references: Vec<Reference>) -> Result<()> {
    let drivers = args
        .tools
        .iter()
        .map(|name| driver(name))
        .collect::<Result<Vec<_>>>()?;
    let tools: serde_json::Value = match std::env::var_os(TOOLS_RECORD) {
        Some(path) => serde_json::from_str(&std::fs::read_to_string(&path)?)
            .with_context(|| format!("reading {}", Path::new(&path).display()))?,
        None => serde_json::Value::Null,
    };
    let versions = drivers
        .iter()
        .filter_map(|d| {
            let version = tools.get(d.name)?.get("version")?.as_str()?;
            Some((d.name.to_owned(), version.to_owned()))
        })
        .collect();
    let output = args
        .output
        .clone()
        .unwrap_or_else(|| PathBuf::from("tools.csv"));
    let work = args
        .work
        .clone()
        .unwrap_or_else(|| output.with_extension("work"));
    let settings = Settings {
        drivers,
        timeout: args.timeout,
        memory: args
            .memory_limit
            .filter(|&m| m > 0)
            .unwrap_or(DEFAULT_MEMORY),
        grace: GRACE,
        work: &work,
        versions,
    };
    let done = if args.resume {
        done(&output)?
    } else {
        HashSet::new()
    };
    let file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(args.append)
        .truncate(!args.append)
        .open(&output)
        .with_context(|| format!("opening {}", output.display()))?;
    let empty = file.metadata()?.len() == 0;
    let mut out = std::io::BufWriter::new(file);
    if empty {
        writeln!(out, "{HEADER}")?;
    }
    record(&output.with_extension("env.json"), &settings, &tools)?;
    let result = run(&references, &settings, &done, &mut out);
    std::fs::remove_dir_all(&work).ok();
    result
}

/// The problems and tools a file of `run --tools` has a row for, each as
/// its source, family, problem, mode and tool joined by commas.
fn done(path: &Path) -> Result<HashSet<String>> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Ok(HashSet::new());
    };
    let mut lines = text.lines();
    if lines.next().unwrap_or("") != HEADER {
        bail!(
            "{} has other columns than this version writes; resume into a new file",
            path.display()
        );
    }
    let columns: Vec<&str> = HEADER.split(',').collect();
    let key: Vec<usize> = ["source", "family", "problem", "mode", "tool"]
        .iter()
        .filter_map(|name| columns.iter().position(|c| c == name))
        .collect();
    Ok(lines
        .map(|line| {
            let fields: Vec<&str> = line.split(',').collect();
            key.iter()
                .map(|&i| fields.get(i).copied().unwrap_or(""))
                .collect::<Vec<_>>()
                .join(",")
        })
        .collect())
}

/// Writes the record of a run: the command line, the limits, every tool
/// with its program, version and source, linlog's commit where the
/// environment names it (`LINLOG_COMMIT`), and the machine (processor,
/// cores this process may use, memory, kernel), as the run sees them.
fn record(path: &Path, settings: &Settings, tools: &serde_json::Value) -> Result<()> {
    let read = |file: &str| std::fs::read_to_string(file).unwrap_or_default();
    let cpuinfo = read("/proc/cpuinfo");
    let processor = cpuinfo
        .lines()
        .find_map(|l| l.strip_prefix("model name"))
        .map_or("", |l| l.trim_start_matches([' ', '\t', ':']));
    let memory = read("/proc/meminfo")
        .lines()
        .find_map(|l| l.strip_prefix("MemTotal:"))
        .map_or(String::new(), |m| m.trim().to_owned());
    let allowed = read("/proc/self/status")
        .lines()
        .find_map(|l| l.strip_prefix("Cpus_allowed_list:"))
        .map_or(String::new(), |c| c.trim().to_owned());
    let version = |program: &str| {
        Command::new(program)
            .arg("--version")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned())
            .unwrap_or_default()
    };
    let record = serde_json::json!({
        "command": std::env::args().collect::<Vec<_>>(),
        "timeout_s": settings.timeout,
        "memory_limit_bytes": settings.memory,
        "grace_s": settings.grace,
        "measured_by": format!("runexec {}", version("runexec")),
        "linlog": version("linlog"),
        "linlog_commit": std::env::var("LINLOG_COMMIT").unwrap_or_default(),
        "tools": settings.drivers.iter().map(|d| serde_json::json!({
            "name": d.name,
            "program": d.program,
            "summary": d.summary,
            "source": tools.get(d.name).cloned().unwrap_or_default(),
        })).collect::<Vec<_>>(),
        "machine": {
            "processor": processor,
            "cpus_allowed": allowed,
            "memory": memory,
            "kernel": read("/proc/sys/kernel/osrelease").trim(),
        },
    });
    std::fs::write(path, serde_json::to_string_pretty(&record)? + "\n")
        .with_context(|| format!("writing {}", path.display()))
}
