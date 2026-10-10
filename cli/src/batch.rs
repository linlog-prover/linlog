// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! `prove` over many sequents: the entries of the inputs in order, each
//! decided by a worker of the library's batch under its own time limit,
//! and one result per entry written as soon as it and those before it
//! are decided.

use crate::argument_parsing::{CoresArg, Format, InputFormat, ProveArgs, Threads, threads};
use crate::io::{self, admit, sequent_in};
use crate::limit::Deadline;
use crate::ordinary;
use crate::prove::{
    Ended, STEPS_PER_CLOCK, Show, Shown, alone_first, derivation, describe, net_into, nets_exist,
    on_large_stack, statistics, stopped, verdict_line,
};
use crate::{Status, catch_interrupt, interrupted};
use anyhow::{Context, Result, anyhow, bail};
use linlog::ordinary::Image;
use linlog::search::batch::{self, Cores};
use linlog::search::{Engine, Options, Outcome, Pool, Verdict, engine_for, prove_goal};
use linlog::{Forest, Mode};
use serde::de::{Deserialize, Deserializer, MapAccess, Visitor};
use serde_json::value::RawValue;
use std::collections::{HashSet, VecDeque};
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Whether the arguments ask for a batch: several files, a directory, a
/// list of paths, or a format of many sequents.
pub fn is_batch(args: &ProveArgs) -> bool {
    let input = &args.input;
    input.file.len() > 1
        || input.file.iter().any(|f| f.is_dir())
        || args.batch.files_from.is_some()
        || input.input_format.is_many()
        || args.batch.entry_name.is_some()
}

/// One sequent of a batch: its name, its mode if it names one, and where
/// it comes from.
struct Entry {
    /// The path as given, or the line's name.
    name: String,
    /// The mode a problem file's column or a record names.
    mode: Option<Mode>,
    /// Where the sequent is.
    source: Source,
}

/// Where an entry's sequent is.
enum Source {
    /// Text read already, in a format of one sequent.
    Text(String, InputFormat),
    /// A file, read when the entry is decided, in a format of one sequent.
    File(PathBuf, InputFormat),
    /// Why there is no sequent: an unreadable file, a malformed line.
    Bad(String),
}

/// What is still to be read, in order.
enum Pending {
    /// A path: a file, or a directory to walk.
    Path(PathBuf),
    /// Lines of sequents, from a file or standard input named `origin`.
    Lines {
        /// The lines.
        reader: Box<dyn BufRead + Send>,
        /// The name of their file, `-` for standard input.
        origin: String,
        /// Their format: lines, jsonl or problems.
        format: InputFormat,
        /// The number of the last line read.
        number: usize,
    },
    /// A list of paths, one per line or separated by NUL.
    List(Box<dyn BufRead + Send>, u8),
}

/// The entries of the inputs, in order, read as they are asked for.
struct Entries {
    /// What is left, first first.
    pending: VecDeque<Pending>,
    /// The format the flag names.
    format: InputFormat,
    /// Whether the inputs are ordinary logic, a `.p` file a TPTP problem.
    ordinary: bool,
    /// The directories walked, by their canonical paths, so that a link
    /// back up is walked once.
    walked: HashSet<PathBuf>,
}

/// Returns a reader of standard input, or of the file at `path`.
fn reader(path: &Path) -> std::io::Result<Box<dyn BufRead + Send>> {
    if path == Path::new("-") {
        return Ok(Box::new(BufReader::new(std::io::stdin())));
    }
    Ok(Box::new(BufReader::new(fs::File::open(path)?)))
}

impl Entries {
    /// The entries of the arguments: the files given, then the list.
    fn new(args: &ProveArgs) -> Result<Self> {
        let mut pending: VecDeque<Pending> =
            args.input.file.iter().cloned().map(Pending::Path).collect();
        if let Some(list) = &args.batch.files_from {
            let reader = reader(list).with_context(|| format!("cannot read {}", list.display()))?;
            pending.push_back(Pending::List(
                reader,
                if args.batch.null { 0 } else { b'\n' },
            ));
        }
        if pending.is_empty() {
            if std::io::IsTerminal::is_terminal(&std::io::stdin()) {
                bail!("no sequents given: pass --file, --files-from, or lines on standard input");
            }
            pending.push_back(Pending::Path("-".into()));
        }
        Ok(Self {
            pending,
            format: args.input.input_format,
            ordinary: args.logic.logic.is_some(),
            walked: HashSet::new(),
        })
    }

    /// Whether a file met in a directory walk is an input: its extension
    /// is the format's, or for `auto` `.p` or `.json`.
    fn takes(&self, path: &Path) -> bool {
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        match self.format {
            InputFormat::Auto if self.ordinary => extension == "p",
            InputFormat::Auto => matches!(extension, "p" | "json"),
            InputFormat::Lltp | InputFormat::Tptp => extension == "p",
            InputFormat::Spec => extension == "spec",
            InputFormat::Json => extension == "json",
            InputFormat::Jsonl => extension == "jsonl",
            InputFormat::Text | InputFormat::Lines | InputFormat::Problems => extension == "txt",
        }
    }

    /// Returns the entries of a path, or puts what it holds in front of
    /// what is pending.
    fn path(&mut self, path: PathBuf) -> Option<Entry> {
        let name = path.display().to_string();
        let bad = |why: String| {
            Some(Entry {
                name: name.clone(),
                mode: None,
                source: Source::Bad(why),
            })
        };
        if path != Path::new("-") && path.is_dir() {
            let canonical = fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
            if !self.walked.insert(canonical) {
                return None;
            }
            let mut inside: Vec<PathBuf> = match fs::read_dir(&path) {
                Ok(entries) => entries.filter_map(|e| e.ok().map(|e| e.path())).collect(),
                Err(e) => return bad(format!("cannot read the directory: {e}")),
            };
            inside.sort();
            inside.retain(|p| p.is_dir() || self.takes(p));
            for p in inside.into_iter().rev() {
                self.pending.push_front(Pending::Path(p));
            }
            return None;
        }
        let format = if path == Path::new("-") {
            self.format
        } else {
            self.format.of(&path, self.ordinary)
        };
        if !format.is_many() {
            return Some(Entry {
                name,
                mode: None,
                source: Source::File(path, format),
            });
        }
        match reader(&path) {
            Ok(reader) => {
                self.pending.push_front(Pending::Lines {
                    reader,
                    origin: name,
                    format,
                    number: 0,
                });
                None
            }
            Err(e) => bad(format!("cannot read it: {e}")),
        }
    }
}

impl Iterator for Entries {
    type Item = Entry;

    fn next(&mut self) -> Option<Entry> {
        while !interrupted() {
            let entry = match self.pending.front_mut()? {
                Pending::Path(_) => {
                    let Some(Pending::Path(path)) = self.pending.pop_front() else {
                        unreachable!("the front is a path")
                    };
                    self.path(path)
                }
                Pending::List(reader, separator) => {
                    let mut bytes = Vec::new();
                    match reader.read_until(*separator, &mut bytes) {
                        Ok(0) => {
                            self.pending.pop_front();
                            continue;
                        }
                        Ok(_) => {
                            if bytes.last() == Some(separator) {
                                bytes.pop();
                            }
                            if *separator == b'\n' && bytes.last() == Some(&b'\r') {
                                bytes.pop();
                            }
                            if bytes.is_empty() {
                                continue;
                            }
                            self.path(path_of(bytes))
                        }
                        Err(e) => {
                            self.pending.pop_front();
                            Some(Entry {
                                name: "--files-from".into(),
                                mode: None,
                                source: Source::Bad(format!("cannot read the list: {e}")),
                            })
                        }
                    }
                }
                Pending::Lines {
                    reader,
                    origin,
                    format,
                    number,
                } => {
                    let mut line = String::new();
                    match reader.read_line(&mut line) {
                        Ok(0) => {
                            self.pending.pop_front();
                            continue;
                        }
                        Ok(_) => {
                            *number += 1;
                            entry_of(&line, origin, *number, *format)
                        }
                        Err(e) => {
                            *number += 1;
                            let name = format!("{origin}:{number}");
                            self.pending.pop_front();
                            Some(Entry {
                                name,
                                mode: None,
                                source: Source::Bad(format!("cannot read the line: {e}")),
                            })
                        }
                    }
                }
            };
            if entry.is_some() {
                return entry;
            }
        }
        None
    }
}

/// Returns the path a list names, its bytes as they are on Unix.
fn path_of(bytes: Vec<u8>) -> PathBuf {
    #[cfg(unix)]
    {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;
        PathBuf::from(OsString::from_vec(bytes))
    }
    #[cfg(not(unix))]
    PathBuf::from(String::from_utf8_lossy(&bytes).into_owned())
}

/// Reads a mode's name as problem files and records write it.
fn mode_named(name: &str) -> Result<Mode> {
    Ok(name.parse()?)
}

/// Returns the entry a line holds, or `None` for a blank line or a
/// comment; a line that is none of the format's is that line's error.
fn entry_of(line: &str, origin: &str, number: usize, format: InputFormat) -> Option<Entry> {
    let place = format!("{origin}:{number}");
    let code = match format {
        InputFormat::Jsonl => line.trim(),
        _ => line.split_once('#').map_or(line, |(code, _)| code).trim(),
    };
    if code.is_empty() {
        return None;
    }
    let entry = |name: Option<&str>, mode, source| Entry {
        name: name
            .filter(|n| names_a_file(n))
            .unwrap_or(&place)
            .to_owned(),
        mode,
        source,
    };
    let text = |s: &str| Source::Text(s.to_owned(), InputFormat::Text);
    Some(match format {
        InputFormat::Problems => {
            let fields: Vec<&str> = code.splitn(5, ';').map(str::trim).collect();
            let [name, mode, _, _, sequent] = fields[..] else {
                let why = "not a problem line `NAME; MODE; EXPECTED; COPIES; SEQUENT`".into();
                return Some(entry(None, None, Source::Bad(why)));
            };
            match mode_named(mode) {
                Ok(mode) => entry(Some(name), Some(mode), text(sequent)),
                Err(e) => entry(Some(name), None, Source::Bad(e.to_string())),
            }
        }
        InputFormat::Jsonl => match record(code) {
            Ok((name, mode, source)) => entry(name.as_deref(), mode, source),
            Err(e) => entry(None, None, Source::Bad(format!("{e:#}"))),
        },
        _ => match code.split_once(':') {
            Some((name, sequent)) => entry(Some(name.trim()), None, text(sequent)),
            None => entry(None, None, text(code)),
        },
    })
}

/// A JSON object's members in the order written, a key written twice
/// kept twice, each value as its text.
struct Members(Vec<(String, Box<RawValue>)>);

impl<'de> Deserialize<'de> for Members {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        /// Collects the members of an object.
        struct Collect;
        impl<'de> Visitor<'de> for Collect {
            type Value = Members;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("a JSON object")
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Members, A::Error> {
                let mut members = Vec::new();
                while let Some(member) = map.next_entry()? {
                    members.push(member);
                }
                Ok(Members(members))
            }
        }
        deserializer.deserialize_map(Collect)
    }
}

/// Reads a JSON line: a sequent, or a record of a `sequent` in JSON or
/// as text with a `name` and a `mode` if it names them. A line with any
/// of the record's keys is a record, whose other keys and repeated keys
/// are refused; its sequent is read as `--input-format json` reads one.
fn record(line: &str) -> Result<(Option<String>, Option<Mode>, Source)> {
    let Members(members) = serde_json::from_str(line).context("not a JSON object")?;
    let keys = ["name", "mode", "sequent"];
    if !members.iter().any(|(key, _)| keys.contains(&key.as_str())) {
        return Ok((None, None, Source::Text(line.to_owned(), InputFormat::Json)));
    }
    let [mut name, mut mode, mut sequent] = [None, None, None];
    for (key, value) in members {
        let slot = match key.as_str() {
            "name" => &mut name,
            "mode" => &mut mode,
            "sequent" => &mut sequent,
            _ => bail!("a record's keys are name, mode and sequent, not `{key}`"),
        };
        if slot.replace(value).is_some() {
            bail!("the record names `{key}` twice");
        }
    }
    let sequent = sequent.context("the record has no `sequent`")?;
    let string = |value: &RawValue, what: &str| {
        serde_json::from_str::<String>(value.get())
            .with_context(|| format!("the record's {what} is not a string"))
    };
    let name = name.map(|name| string(&name, "name")).transpose()?;
    let mode = mode
        .map(|mode| mode_named(&string(&mode, "mode")?))
        .transpose()?;
    let source = if sequent.get().starts_with('"') {
        Source::Text(string(&sequent, "sequent")?, InputFormat::Text)
    } else {
        Source::Text(sequent.get().to_owned(), InputFormat::Json)
    };
    Ok((name, mode, source))
}

/// The answer to one entry: what is written for it, and its verdict.
struct Done {
    /// The line, or lines, written for it.
    text: String,
    /// Its verdict.
    status: Status,
}

/// What every worker shares.
struct Shared {
    /// The arguments.
    args: Arc<ProveArgs>,
    /// The output's options, with a directory as the output if there is
    /// one.
    show: Show,
    /// The batch's own time limit.
    batch: Deadline,
    /// The threads of a sequent searched within the cores.
    threads: Threads,
    /// The directory the derivations go to.
    directory: Option<PathBuf>,
}

/// The extension of a file a derivation is written to in a format.
fn extension(format: Format) -> &'static str {
    match format {
        Format::Text => "txt",
        Format::Json => "json",
        Format::Latex => "tex",
        Format::Typst => "typ",
        Format::Svg => "svg",
        Format::Png => "png",
        Format::Pdf => "pdf",
        Format::Rocq => "v",
    }
}

/// Returns whether a line's name names a file of `file_for`: whether it
/// has a component other than a root or `.`, so that its file lies inside
/// the directory. A name without one (empty, `.`, `/`) is no name.
fn names_a_file(name: &str) -> bool {
    Path::new(name)
        .components()
        .any(|c| matches!(c, Component::Normal(_) | Component::ParentDir))
}

/// Returns the file in `directory` for an entry named `name`: the name as
/// a relative path (`..` written `__`, a root dropped) with the format's
/// extension added, so that entries of the same file name in different
/// directories, or of different kinds, get different files. Every name
/// of an entry names a file (a path given, or `names_a_file`), so the
/// file lies inside `directory`.
fn file_for(directory: &Path, name: &str, format: Format) -> PathBuf {
    let mut path = directory.to_owned();
    for component in Path::new(name).components() {
        match component {
            Component::Normal(part) => path.push(part),
            Component::ParentDir => path.push("__"),
            _ => {}
        }
    }
    let mut file = path.into_os_string();
    file.push(".");
    file.push(extension(format));
    file.into()
}

/// Returns the JSON of a record: the wire level, the entry's name, then
/// the fields of `body`, a JSON object, but its own `version`.
fn record_json(name: &str, body: &str) -> String {
    let name = serde_json::to_string(name).expect("a string is JSON");
    let head = format!("{{\"version\":{},\"name\":{name}", linlog::wire::LEVEL);
    let rest = body.strip_prefix('{').expect("the body is an object");
    let rest = match rest.strip_prefix("\"version\":") {
        Some(versioned) => versioned.trim_start_matches(|c: char| c.is_ascii_digit()),
        None => rest,
    };
    match rest.strip_prefix(',') {
        Some(fields) => format!("{head},{fields}"),
        None if rest == "}" => format!("{head}}}"),
        None => format!("{head},{rest}"),
    }
}

impl Shared {
    /// Returns what is written for an entry: in JSON a record, else
    /// `NAME: LINE`.
    fn line(&self, name: &str, kind: &str, line: &str) -> String {
        if self.json_lines() {
            let body = serde_json::json!({ kind: line }).to_string();
            record_json(name, &body)
        } else if kind == "error" {
            format!("{name}: error: {line}")
        } else {
            format!("{name}: {line}")
        }
    }

    /// Whether the results are JSON Lines: JSON without a directory.
    fn json_lines(&self) -> bool {
        self.show.format == Format::Json && self.directory.is_none()
    }

    /// Decides an entry, writes its derivation if one is asked for, and
    /// returns what is written for it.
    fn answer(&self, entry: Entry, plan: &batch::Plan) -> Done {
        let name = entry.name.clone();
        let failed = |e: anyhow::Error| Done {
            text: self.line(&name, "error", &format!("{e:#}")),
            status: Status::Error,
        };
        if self.batch.passed() || interrupted() {
            let why = match self.batch.limit() {
                _ if interrupted() => "interrupted before it was searched".to_owned(),
                Some(t) => {
                    format!("the batch's time limit of {t:?} was reached before it was searched")
                }
                None => unreachable!("only a limit passes"),
            };
            return Done {
                text: self.line(&name, "unknown", &format!("unknown: {why}")),
                status: Status::Unknown,
            };
        }
        if self.args.batch.isolate && self.args.batch.entry_name.is_none() {
            return self.isolated(entry).unwrap_or_else(failed);
        }
        self.decide(entry, plan).unwrap_or_else(failed)
    }

    /// Decides an entry in this process.
    fn decide(&self, entry: Entry, plan: &batch::Plan) -> Result<Done> {
        let (search, limits) = (&plan.search, &plan.limits);
        let args = &self.args;
        let deadline = Deadline::start(args.timeout.0, Instant::now())?;
        let most = args.input.most();
        let format = match &entry.source {
            Source::Text(_, format) | Source::File(_, format) => Some(*format),
            Source::Bad(_) => None,
        };
        let source = entry.source;
        let logic = args.logic.clone();
        let loaded = deadline.within(move || -> Result<(Forest, Option<Image>)> {
            let (text, format) = match source {
                Source::Bad(why) => return Err(anyhow!(why)),
                Source::Text(text, format) => (text, format),
                Source::File(path, format) => (io::read(Some(&path), "sequent")?, format),
            };
            let (sequent, image) = if logic.logic.is_some() {
                let image = ordinary::image(&logic, &ordinary::sequent_in(&text, format)?)?;
                (image.sequent().clone(), Some(image))
            } else if format == InputFormat::Tptp {
                bail!("--input-format tptp holds ordinary logic, which --logic reads");
            } else {
                (sequent_in(&text, format, most)?, None)
            };
            Ok((
                Forest::from_owned(admit(sequent, most)?, &io::bound(most))?,
                image,
            ))
        })?;
        let Some(forest) = loaded else {
            let limit = deadline.limit().expect("only a limit passes");
            let line = format!(
                "unknown: the time limit of {limit:?} was reached while the sequent was read"
            );
            return Ok(Done {
                text: self.line(&entry.name, "unknown", &line),
                status: Status::Unknown,
            });
        };
        let (forest, image) = forest?;
        let sequent = forest.sequent();
        let mode = match &image {
            Some(image) => image.mode(),
            None => {
                let mode = entry.mode.unwrap_or(args.mode.mode());
                format.map_or(mode, |format| io::affine_for(format, mode))
            }
        };
        if self.show.net {
            nets_exist(sequent, mode)?;
        }
        let start = Instant::now();
        let halt = || interrupted() || deadline.passed() || self.batch.passed();
        let outcome = if search.jobs.count() > 1 {
            let parallel =
                || engine_for(&forest, forest.roots(), mode, search).is_ok_and(Engine::parallel);
            let stack = limits.stack_bytes();
            alone_first(
                search,
                stack,
                self.threads,
                &halt,
                parallel,
                |options, halt| {
                    prove_goal(&forest, forest.roots(), mode, options, limits, |_| halt())
                },
            )
        } else {
            prove_goal(&forest, forest.roots(), mode, search, limits, |_| halt())
        }
        .map_err(|e| describe(e, sequent))?;
        let elapsed = start.elapsed();
        let stop = stopped(&deadline).or_else(|| stopped(&self.batch));
        let status = match outcome.verdict {
            Verdict::Proved(_) => Status::Yes,
            Verdict::Unprovable(_) => Status::No,
            Verdict::Unknown(_) => Status::Unknown,
        };
        let ended = Ended { stop, elapsed };
        let line = match &image {
            Some(image) => ordinary::verdict_line(&outcome, image, &ended),
            None => verdict_line(&outcome, args.fragment.is_some(), &ended),
        };
        let mut text = if self.json_lines() {
            let mut body = serde_json::to_string(&outcome)?;
            if args.stats {
                body.pop();
                body.push_str(&format!(",\"seconds\":{}}}", elapsed.as_secs_f64()));
            }
            record_json(&entry.name, &body)
        } else {
            format!("{}: {line}", entry.name)
        };
        let over = || interrupted() || deadline.passed() || self.batch.passed();
        if let Some(directory) = &self.directory
            && let Some(note) = self.write(
                &entry.name,
                directory,
                &outcome,
                (mode, image.as_ref()),
                &line,
                over,
            )?
        {
            text.push_str(&format!("\n  {note}"));
        }
        if args.stats && !self.json_lines() {
            for counter in statistics(&outcome, elapsed).lines() {
                text.push_str(&format!("\n  {counter}"));
            }
        }
        Ok(Done { text, status })
    }

    /// Writes the outcome, or the proof's derivation or net, into its file
    /// in `directory`, and returns the line that says why nothing was
    /// written when a proof's file is not.
    fn write(
        &self,
        name: &str,
        directory: &Path,
        outcome: &Outcome,
        (mode, image): (Mode, Option<&Image>),
        line: &str,
        over: impl Fn() -> bool,
    ) -> Result<Option<String>> {
        let show = &self.show;
        let path = file_for(directory, name, show.format);
        let open = || -> Result<io::Output> {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("cannot make {}", parent.display()))?;
            }
            io::Output::open(Some(&path), show.format.is_binary())
        };
        if show.format == Format::Json {
            let mut out = open()?;
            serde_json::to_writer(out.stream(), outcome)?;
            out.finish()?;
            return Ok(None);
        }
        let (Verdict::Proved(proof), false) = (&outcome.verdict, self.args.output.quiet) else {
            return Ok(None);
        };
        let mut out = open()?;
        let prefix = (show.verdict && !show.format.is_binary())
            .then(|| format!("{}\n", crate::prove::note(show.format, line)));
        let why = || match self.args.timeout.0 {
            _ if interrupted() => "interrupted".to_owned(),
            _ if self.batch.passed() => "the batch's time limit was reached".to_owned(),
            Some(t) => format!("the time limit of {t:?} was reached"),
            None => "stopped".to_owned(),
        };
        let shown = if show.net {
            let found = outcome.net.as_ref();
            net_into(
                found,
                proof,
                mode,
                show,
                &over,
                &why,
                prefix.as_deref(),
                &mut out,
            )?
        } else {
            let mut steps = 0u32;
            let halt = || {
                steps = steps.wrapping_add(1);
                steps.is_multiple_of(STEPS_PER_CLOCK) && over()
            };
            match (image, self.args.linear) {
                (Some(image), false) => crate::ordinary::derivation(
                    image,
                    proof,
                    show,
                    halt,
                    why,
                    prefix.as_deref(),
                    &mut out,
                )?,
                _ => derivation(proof, mode, show, halt, why, prefix.as_deref(), &mut out)?,
            }
        };
        match shown {
            Shown::Written => out.finish()?,
            Shown::Rendered(bytes) => {
                out.stream().write_all(&bytes)?;
                out.finish()?;
            }
            Shown::LeftOut(reason) => return Ok(Some(reason)),
            Shown::Cut(reason) => return Ok(Some(format!("{reason}; no file is written"))),
            Shown::Nothing => {}
        }
        Ok(None)
    }

    /// Decides an entry in a child process of this command, which is given
    /// the command's own arguments and the entry, and answers as a batch
    /// of one.
    fn isolated(&self, entry: Entry) -> Result<Done> {
        let exe = std::env::current_exe().context("cannot find this program to start a child")?;
        // The entry's flags go right after the subcommand, before any `--`
        // of the command's own; the child reads no input of the batch's,
        // so the command line is passed whole, however its flags are
        // spelt.
        let mut arguments = std::env::args_os().skip(1);
        let mut command = Command::new(exe);
        command
            .args(arguments.next())
            .arg("--entry-name")
            .arg(&entry.name);
        if let Some(mode) = entry.mode {
            command.arg("--entry-mode").arg(mode.name());
        }
        let mut stdin = None;
        match entry.source {
            Source::Bad(why) => bail!(why),
            Source::File(path, format) => {
                command
                    .arg("--entry-file")
                    .arg(path)
                    .arg("--entry-format")
                    .arg(format_name(format));
            }
            Source::Text(text, format) => {
                command.arg("--entry-format").arg(format_name(format));
                stdin = Some(text);
            }
        }
        command.args(arguments);
        let mut child = command
            .stdin(if stdin.is_some() {
                Stdio::piped()
            } else {
                Stdio::null()
            })
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("cannot start a child process")?;
        if let (Some(text), Some(mut pipe)) = (stdin, child.stdin.take()) {
            pipe.write_all(text.as_bytes())
                .context("cannot write to the child")?;
        }
        // The child keeps its own time limit; a child that outlives it by
        // the grace is stopped.
        let end = self
            .args
            .timeout
            .0
            .map(|t| Instant::now() + t + CHILD_GRACE);
        let (mut out, mut err) = (child.stdout.take(), child.stderr.take());
        let reading = std::thread::spawn(move || {
            let mut text = String::new();
            if let Some(out) = out.as_mut() {
                let _ = out.read_to_string(&mut text);
            }
            let mut error = String::new();
            if let Some(err) = err.as_mut() {
                let _ = err.read_to_string(&mut error);
            }
            (text, error)
        });
        let status = loop {
            if let Some(status) = child.try_wait()? {
                break status;
            }
            if end.is_some_and(|end| Instant::now() > end) || interrupted() {
                let _ = child.kill();
                let _ = child.wait();
                let why = if interrupted() {
                    "interrupted"
                } else {
                    "outlived its time limit"
                };
                bail!("the child process {why} and was stopped");
            }
            std::thread::sleep(CHILD_POLL);
        };
        let (text, error) = reading.join().unwrap_or_default();
        let status = match status.code() {
            Some(0) => Status::Yes,
            Some(1) => Status::No,
            Some(3) => Status::Unknown,
            Some(_) if !text.trim().is_empty() => Status::Error,
            code => {
                let why = error
                    .lines()
                    .last()
                    .unwrap_or("")
                    .trim_start_matches("error: ");
                match code {
                    Some(code) => bail!("the child process ended with status {code}: {why}"),
                    None => bail!("the child process was killed{}", killed_by(&status)),
                }
            }
        };
        Ok(Done {
            text: text.trim_end().to_owned(),
            status,
        })
    }
}

/// How long a child may outlive its time limit before it is stopped: what
/// it may still write and render after the search.
const CHILD_GRACE: Duration = Duration::from_secs(5);

/// How often the parent looks whether a child has ended.
const CHILD_POLL: Duration = Duration::from_millis(5);

/// Says which signal ended a process, where the platform tells.
fn killed_by(status: &std::process::ExitStatus) -> String {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(signal) = status.signal() {
            return format!(" by signal {signal}");
        }
    }
    String::new()
}

/// Names a format as `--input-format` reads it.
fn format_name(format: InputFormat) -> String {
    clap::ValueEnum::to_possible_value(&format)
        .expect("no value is skipped")
        .get_name()
        .to_owned()
}

/// Returns the memory the process may use where the system says: the
/// machine's (`MemTotal`), or less where a control group bounds it, as a
/// container, a systemd scope or a CI runner does. Half of it is the
/// default of `--batch-memory`.
pub fn machine_memory() -> Option<u64> {
    let total = fs::read_to_string("/proc/meminfo").ok().and_then(|info| {
        let line = info.lines().find(|l| l.starts_with("MemTotal:"))?;
        let kib: u64 = line.split_whitespace().nth(1)?.parse().ok()?;
        Some(kib.saturating_mul(1024))
    });
    [total, cgroup_memory()].into_iter().flatten().min()
}

/// Returns the least memory limit of the control groups the process is
/// in, if one is set: cgroup v2's `memory.max` of its group and of every
/// group above it, else cgroup v1's `memory.limit_in_bytes`. A limit of
/// `max` is none.
fn cgroup_memory() -> Option<u64> {
    let groups = fs::read_to_string("/proc/self/cgroup").ok()?;
    let limit = |file: PathBuf| fs::read_to_string(file).ok()?.trim().parse::<u64>().ok();
    let root = Path::new("/sys/fs/cgroup");
    if let Some(group) = groups.lines().find_map(|line| line.strip_prefix("0::")) {
        let mut directory = root.join(group.trim_start_matches('/'));
        let mut least = None;
        loop {
            if let Some(bytes) = limit(directory.join("memory.max")) {
                least = Some(least.map_or(bytes, |least: u64| least.min(bytes)));
            }
            if directory == root || !directory.pop() {
                return least;
            }
        }
    }
    let group = groups.lines().find_map(|line| {
        let mut fields = line.splitn(3, ':');
        let controllers = fields.nth(1)?;
        let path = fields.next()?;
        controllers
            .split(',')
            .any(|c| c == "memory")
            .then_some(path)
    })?;
    limit(
        root.join("memory")
            .join(group.trim_start_matches('/'))
            .join("memory.limit_in_bytes"),
    )
}

/// Runs a batch: reads the entries, decides them on the library's batch,
/// and writes one result per entry in order; the status is the worst
/// verdict, an error before unknown before unprovable before proved, and
/// unknown at best after an interrupt.
///
/// # Errors
///
/// A sequent given as an argument, an input that cannot be read, a format
/// of one sequent, or inputs that hold no entry; an entry's own error is
/// its line, never an error of the batch.
pub fn run(args: &ProveArgs) -> Result<Status> {
    if args.input.sequent.is_some() {
        bail!(
            "a sequent given as an argument is not read in a batch, whose sequents come from \
             --files-from or a format of many sequents"
        );
    }
    crate::prove::ordinary_mode(args)?;
    let start = Instant::now();
    let format = args.output.format();
    let directory = args.output.output.clone();
    if directory.is_none() && !matches!(format, Format::Text | Format::Json) {
        bail!(
            "a batch writes each {} into a directory: give it with --output DIR",
            format.title()
        );
    }
    if let Some(directory) = &directory {
        fs::create_dir_all(directory)
            .with_context(|| format!("cannot make the directory {}", directory.display()))?;
    }
    let show = Show::new(&args.output)?.within(args.memory_limit.0);
    let threads = threads(args.jobs, args.pool_after, args.deterministic);
    let search = Options::default()
        .with_memo_limit(args.memo_limit)
        .with_engine(args.engine.into())
        .with_fragment(args.fragment.map(Into::into))
        .with_copies(args.copies.0)
        .with_bias(args.bias.into())
        .with_forward_copies(args.forward_copies)
        .with_check(!args.no_check)
        .with_jobs(threads.jobs)
        .with_pool(Some(Pool::new()));
    let limits = args.limits();
    let stdin_stream = args.input.file.is_empty() && args.batch.files_from.is_none();
    let cores = match args.batch.cores {
        // A program that writes a question and waits for its answer
        // cannot be read ahead of.
        CoresArg::Auto if stdin_stream => Cores::Within,
        CoresArg::Auto => Cores::Auto,
        CoresArg::Across => Cores::Across,
        CoresArg::Within => Cores::Within,
    };
    let machine = std::thread::available_parallelism().map_or(1, |n| n.get());
    let options = batch::Options::default()
        .with_mode(args.mode.mode())
        .with_cores(cores)
        .with_workers(
            args.batch
                .workers
                .unwrap_or(machine)
                .clamp(1, Options::MAX_JOBS),
        )
        .with_total_memory_bytes(match args.batch.batch_memory {
            Some(limit) => limit.0,
            None => {
                Some(machine_memory().map_or(batch::Options::DEFAULT_TOTAL_MEMORY_BYTES, |m| m / 2))
            }
        });
    if let Some(name) = &args.batch.entry_name {
        // A child of `--isolate`: the batch's inputs are the parent's, and
        // the one entry is the hidden flags' or standard input.
        let entry = Entry {
            name: name.clone(),
            mode: args
                .batch
                .entry_mode
                .as_deref()
                .map(mode_named)
                .transpose()?,
            source: Source::File(
                args.batch.entry_file.clone().unwrap_or_else(|| "-".into()),
                args.batch.entry_format.unwrap_or(InputFormat::Text),
            ),
        };
        let plan = batch::Plan::alone(search, limits);
        return one(args, show, threads, directory, &plan, entry);
    }
    let entries = Entries::new(args)?;
    catch_interrupt();
    let shared = Arc::new(Shared {
        args: Arc::new(args.clone()),
        show,
        batch: Deadline::start(args.batch.batch_timeout.0, start)?,
        threads,
        directory,
    });
    let worker = shared.clone();
    let stack = limits.stack_bytes();
    on_large_stack(stack, move || {
        // The command's interruption ends the searches; the batch's own
        // cancel is never raised here.
        let results = batch::run(
            entries,
            &options,
            &search,
            &limits,
            move |entry, plan, _| worker.answer(entry, plan),
        );
        let mut worst: Option<Status> = None;
        let mut stdout = std::io::stdout().lock();
        for done in results {
            writeln!(stdout, "{}", done.text)
                .and_then(|()| stdout.flush())
                .context("cannot write to standard output")?;
            worst = Some(worst.map_or(done.status, |worst| worst.worse(done.status)));
        }
        if interrupted() {
            // The interrupt ended the input: what was not read is not
            // answered, so the batch is unknown at best.
            eprintln!("interrupted: the rest of the input was not read");
            return Ok(worst.map_or(Status::Unknown, |worst| worst.worse(Status::Unknown)));
        }
        worst.ok_or_else(|| {
            anyhow!(
                "the batch holds no sequent: a directory is walked for the files of the input \
                 format's extension (.p and .json for auto), and blank lines and comments are \
                 no entries"
            )
        })
    })?
}

/// Decides the one entry of a child's batch on this thread's stack and
/// writes its result.
fn one(
    args: &ProveArgs,
    show: Show,
    threads: Threads,
    directory: Option<PathBuf>,
    plan: &batch::Plan,
    entry: Entry,
) -> Result<Status> {
    let shared = Shared {
        args: Arc::new(args.clone()),
        show,
        batch: Deadline::start(None, Instant::now())?,
        threads,
        directory,
    };
    let done = on_large_stack(plan.limits.stack_bytes(), || shared.answer(entry, plan))?;
    println!("{}", done.text);
    Ok(done.status)
}
