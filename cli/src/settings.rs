// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The settings a command runs under, as the library's one value: the
//! command's defaults, then the JSON of `--settings FILE` over them, then
//! the flags given on the command line, each the spelling of one key.

use crate::argument_parsing::{
    CheckArgs, CoresArg, InteractArgs, LogicArgs, ProveArgs, SearchArgs,
};
use crate::io;
use anyhow::{Context, Result, bail};
use clap::ArgMatches;
use clap::parser::ValueSource;
use linlog::Settings;
use linlog::search::batch::{self, Cores};
use linlog::search::{Jobs, Options, Schedule};
use serde_json::Value;
use std::collections::HashSet;
use std::path::Path;
use std::time::Duration;

/// The flags given on the command line, by the names of their arguments.
#[derive(Clone, Debug, Default)]
pub(crate) struct Given(HashSet<String>);

impl Given {
    /// Returns the flags given to the command the matches lead to, its
    /// subcommands followed to the last.
    #[must_use]
    pub(crate) fn of(matches: &ArgMatches) -> Self {
        let mut leaf = matches;
        while let Some((_, sub)) = leaf.subcommand() {
            leaf = sub;
        }
        Self(
            leaf.ids()
                .filter(|id| leaf.value_source(id.as_str()) == Some(ValueSource::CommandLine))
                .map(|id| id.as_str().to_owned())
                .collect(),
        )
    }
}

/// Which flags a command applies over its settings: every one, since a
/// flag's default is the command's, when no file was read; else the
/// flags given, so that a file's value stands where no flag overrides it.
struct Apply<'a> {
    /// Whether a settings file was read.
    file: bool,
    /// The flags given.
    given: &'a Given,
}

impl Apply<'_> {
    /// Whether the flag of this argument sets its key.
    fn set(&self, id: &str) -> bool {
        !self.file || self.given.0.contains(id)
    }
}

/// Returns the command's defaults: the library's, but a batch has as many
/// workers as the machine runs threads and holds half the memory the
/// process may use.
fn defaults() -> Settings {
    let mut settings = Settings::default();
    settings.batch.workers = std::thread::available_parallelism()
        .map_or(1, std::num::NonZero::get)
        .clamp(1, Options::MAX_JOBS);
    settings.batch.total_memory_bytes = Some(
        crate::batch::machine_memory()
            .map_or(batch::Options::DEFAULT_TOTAL_MEMORY_BYTES, |m| m / 2),
    );
    settings
}

/// Returns the command's defaults with the settings file read over them,
/// if one is given: a key the file leaves out keeps the default.
///
/// # Errors
///
/// A file that cannot be read, is no JSON, or holds a key or a value the
/// settings do not have.
fn read(path: Option<&Path>) -> Result<Settings> {
    let base = defaults();
    let Some(path) = path else {
        return Ok(base);
    };
    let text = io::read_configuration(path, "settings file")?;
    let file: Value =
        serde_json::from_str(&text).with_context(|| format!("{} is not JSON", path.display()))?;
    // The flags alone say these, so a file's value would be ignored.
    for (key, flags) in [
        (
            "/batch/mode",
            "the mode flags (--intuitionistic, --affine, --mix)",
        ),
        ("/ordinary/logic", "--logic"),
    ] {
        if file.pointer(key).is_some() {
            bail!(
                "{} sets {}, which {flags} alone set: leave it out",
                path.display(),
                key[1..].replace('/', ".")
            );
        }
    }
    let mut value = serde_json::to_value(&base)?;
    over(&mut value, file);
    serde_json::from_value(value)
        .with_context(|| format!("{} holds settings linlog does not know", path.display()))
}

/// Writes `new` over `value`: an object key by key, anything else whole.
pub(crate) fn over(value: &mut Value, new: Value) {
    match (value, new) {
        (Value::Object(old), Value::Object(new)) => {
            for (key, new) in new {
                match old.get_mut(&key) {
                    Some(at) => over(at, new),
                    None => {
                        old.insert(key, new);
                    }
                }
            }
        }
        (value, new) => *value = new,
    }
}

/// Returns a duration as the settings' milliseconds, the most there are
/// for one past them.
fn millis(time: Duration) -> u64 {
    u64::try_from(time.as_millis()).unwrap_or(u64::MAX)
}

impl SearchArgs {
    /// Writes the flags that apply over the settings' search options,
    /// limits and clock.
    fn apply(&self, settings: &mut Settings, apply: &Apply) {
        let (search, limits, clock) = (
            &mut settings.search,
            &mut settings.limits,
            &mut settings.clock,
        );
        if apply.set("bias") {
            search.bias = self.bias.into();
        }
        if apply.set("copies") {
            search.copies = self.copies.0;
        }
        if apply.set("forward_copies") {
            search.forward_copies = self.forward_copies;
        }
        if apply.set("memo_limit") {
            search.memo_limit = self.memo_limit;
        }
        if apply.set("test_period") {
            search.test_period = self.test_period;
        }
        if apply.set("schedule") {
            search.schedule = self.schedule.into();
        }
        if apply.set("timeout") {
            clock.time_limit_ms = self.timeout.0.map(millis);
        }
        if apply.set("memory_limit") {
            limits.memory_bytes = self.memory_limit.0;
        }
        if apply.set("recursion_limit") {
            limits.recursion_depth = self.recursion_limit;
        }
        if apply.set("work_limit") {
            limits.work = self.work_limit.0;
        }
        // `--jobs N` searches with N threads from the start unless
        // `--pool-after` says otherwise, and `--deterministic` on one.
        if self.deterministic {
            search.jobs = Jobs::Count(1);
            search.schedule = Schedule::Turns;
            clock.pool_after_ms = None;
            return;
        }
        if let Some(jobs) = self.jobs {
            search.jobs = Jobs::Count(jobs);
            clock.pool_after_ms = None;
        }
        if let Some(after) = self.pool_after {
            clock.pool_after_ms = Some(millis(after));
        }
    }
}

impl ProveArgs {
    /// Returns the settings `prove` runs under.
    ///
    /// # Errors
    ///
    /// Those of reading the settings file.
    pub(crate) fn settings(&self, given: &Given) -> Result<Settings> {
        let mut settings = read(self.settings.as_deref())?;
        let apply = Apply {
            file: self.settings.is_some(),
            given,
        };
        self.search.apply(&mut settings, &apply);
        if apply.set("engine") {
            settings.search.engine = self.engine.0;
        }
        if apply.set("fragment") {
            settings.search.fragment = self.fragment.map(Into::into);
        }
        if apply.set("no_check") {
            settings.search.check = !self.no_check;
        }
        if apply.set("occurrence_limit") {
            settings.limits.occurrences = self.input.occurrence_limit.0;
        }
        if apply.set("derivation_limit") {
            settings.limits.derivation_bytes = self.output.derivation_limit.0;
        }
        if apply.set("compact") {
            settings.view.compact = self.output.compact.into();
        }
        let batch = &self.batch;
        if let Some(workers) = batch.workers {
            settings.batch.workers = workers.clamp(1, Options::MAX_JOBS);
        }
        if apply.set("cores") {
            settings.batch.cores = match batch.cores {
                CoresArg::Auto => Cores::Auto,
                CoresArg::Across => Cores::Across,
                CoresArg::Within => Cores::Within,
            };
        }
        if let Some(memory) = batch.batch_memory {
            settings.batch.total_memory_bytes = memory.0;
        }
        if apply.set("batch_timeout") {
            settings.clock.batch_time_limit_ms = batch.batch_timeout.0.map(millis);
        }
        Ok(settings)
    }
}

impl InteractArgs {
    /// Returns the settings a session runs under: its search's, a
    /// `close`'s clock, and the bounds of its derivations.
    ///
    /// # Errors
    ///
    /// Those of reading the settings file.
    pub(crate) fn settings(&self, given: &Given) -> Result<Settings> {
        let mut settings = read(self.settings.as_deref())?;
        let apply = Apply {
            file: self.settings.is_some(),
            given,
        };
        self.search.apply(&mut settings, &apply);
        if apply.set("occurrence_limit") {
            settings.limits.occurrences = self.input.occurrence_limit.0;
        }
        if apply.set("derivation_limit") {
            settings.limits.derivation_bytes = self.derivation_limit.0;
        }
        Ok(settings)
    }
}

impl CheckArgs {
    /// Returns the settings `check` runs under: the bounds of the check
    /// and of its derivation, and how that is shown.
    ///
    /// # Errors
    ///
    /// Those of reading the settings file.
    pub(crate) fn settings(&self, given: &Given) -> Result<Settings> {
        let mut settings = read(self.settings.as_deref())?;
        let apply = Apply {
            file: self.settings.is_some(),
            given,
        };
        if apply.set("memory_limit") {
            settings.limits.memory_bytes = self.memory_limit.0;
        }
        if apply.set("timeout") {
            settings.clock.time_limit_ms = self.timeout.0.map(millis);
        }
        if apply.set("work_limit") {
            settings.limits.work = self.work_limit.0;
        }
        if apply.set("occurrence_limit") {
            settings.limits.occurrences = self.occurrence_limit.0;
        }
        if apply.set("derivation_limit") {
            settings.limits.derivation_bytes = self.output.derivation_limit.0;
        }
        if apply.set("compact") {
            settings.view.compact = self.output.compact.into();
        }
        Ok(settings)
    }
}

/// Returns the settings `seq print` runs under: the bound of a render
/// and the occurrences of the sequent.
///
/// # Errors
///
/// Those of reading the settings file.
pub(crate) fn of_print(
    path: Option<&Path>,
    memory: Option<u64>,
    occurrences: Option<u64>,
    given: &Given,
) -> Result<Settings> {
    let mut settings = read(path)?;
    let apply = Apply {
        file: path.is_some(),
        given,
    };
    if apply.set("memory_limit") {
        settings.limits.memory_bytes = memory;
    }
    if apply.set("occurrence_limit") {
        settings.limits.occurrences = occurrences;
    }
    Ok(settings)
}

/// The threads a search gets, and how long one thread searches before
/// they join it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Threads {
    /// The threads, at most those the machine runs at once.
    pub(crate) jobs: usize,
    /// How long one thread searches first, when the threads are more than
    /// one and do not start at once.
    pub(crate) alone: Option<Duration>,
}

impl Threads {
    /// Returns the threads the settings give: every thread the machine
    /// runs at once for `Jobs::Auto`, else the count, at most what the
    /// machine runs at once (with a note on standard error when the count
    /// is more) and at most the library's bound; one where the machine
    /// does not tell and the count is not given.
    pub(crate) fn of(settings: &Settings) -> Self {
        let machine = std::thread::available_parallelism()
            .map(std::num::NonZero::get)
            .ok();
        let most = machine.map_or(Options::MAX_JOBS, |m| m.min(Options::MAX_JOBS));
        let jobs = match settings.search.jobs {
            Jobs::Count(asked) => {
                if asked > most {
                    eprintln!(
                        "note: --jobs {asked} is more than this machine runs at once; a search \
                         uses {most}"
                    );
                }
                asked.clamp(1, most)
            }
            _ => machine.map_or(1, |m| m.min(Options::MAX_JOBS)),
        };
        Self {
            jobs,
            alone: settings
                .clock
                .pool_after_ms
                .map(Duration::from_millis)
                .filter(|t| jobs > 1 && !t.is_zero()),
        }
    }
}

/// Returns the options of the ordinary logic `--logic` names, if it names
/// one, with the translation `--translation` names, else the settings'.
pub(crate) fn ordinary(
    settings: &Settings,
    logic: &LogicArgs,
) -> Option<linlog::ordinary::Options> {
    let options = settings.ordinary.with_logic(logic.logic?.into());
    Some(match logic.translation {
        Some(translation) => options.with_translation(Some(translation.into())),
        None => options,
    })
}

/// Returns the most occurrences the settings give a sequent.
pub(crate) fn most(settings: &Settings) -> u64 {
    settings.limits.occurrences.unwrap_or(u64::MAX)
}

/// Returns the settings' time limit as a duration.
pub(crate) fn time_limit(settings: &Settings) -> Option<Duration> {
    settings.clock.time_limit_ms.map(Duration::from_millis)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A settings file is read over the defaults: a key it leaves out
    /// keeps the default, and one it holds replaces it.
    #[test]
    fn a_file_is_read_over_the_defaults() {
        let mut value = serde_json::to_value(defaults()).unwrap();
        over(
            &mut value,
            serde_json::json!({"search": {"copies": 3}, "limits": {"memory_bytes": null}}),
        );
        let settings: Settings = serde_json::from_value(value).unwrap();
        assert_eq!(settings.search.copies, Some(3));
        assert_eq!(settings.limits.memory_bytes, None);
        assert_eq!(settings.search.memo_limit, Options::DEFAULT_MEMO_LIMIT);
        assert_eq!(settings.batch.workers, defaults().batch.workers);
    }

    /// A settings file's key stands where no flag is given, and a flag
    /// given beside the file overrides it; a flag's default never does.
    #[test]
    fn flags_given_override_the_file() {
        use crate::argument_parsing::{Cli, Command};
        use clap::{CommandFactory, FromArgMatches};
        let file =
            std::env::temp_dir().join(format!("linlog-settings-{}.json", std::process::id()));
        std::fs::write(&file, r#"{"search": {"copies": 0}, "limits": {"work": 7}}"#).unwrap();
        let settings = |args: &[&str]| {
            let line = [
                &["linlog", "prove", "--settings", file.to_str().unwrap()],
                args,
                &["A |- A"],
            ]
            .concat();
            let matches = Cli::command().try_get_matches_from(line).unwrap();
            let Command::Prove(prove) = Cli::from_arg_matches(&matches).unwrap().command else {
                unreachable!("a prove command")
            };
            prove.settings(&Given::of(&matches)).unwrap()
        };
        let read = settings(&["--memo-limit", "5"]);
        assert_eq!(
            (read.search.copies, read.limits.work, read.search.memo_limit),
            (Some(0), Some(7), 5)
        );
        let overridden = settings(&["--copies", "3", "--work-limit", "none"]);
        assert_eq!(
            (overridden.search.copies, overridden.limits.work),
            (Some(3), None)
        );
        std::fs::remove_file(&file).unwrap();
    }

    /// A settings file may not set what the flags alone say, nor be
    /// standard input, which carries the input.
    #[test]
    fn a_file_keeps_to_its_keys() {
        let file =
            std::env::temp_dir().join(format!("linlog-settings-keys-{}.json", std::process::id()));
        for (json, key) in [
            (r#"{"batch": {"mode": "affine"}}"#, "batch.mode"),
            (r#"{"ordinary": {"logic": "minimal"}}"#, "ordinary.logic"),
        ] {
            std::fs::write(&file, json).unwrap();
            let error = read(Some(&file)).unwrap_err().to_string();
            assert!(error.contains(key), "{error}");
        }
        std::fs::remove_file(&file).unwrap();
        let error = read(Some(Path::new("-"))).unwrap_err().to_string();
        assert!(error.contains("standard input"), "{error}");
    }

    /// The search flags set their keys: `--deterministic` wins over
    /// `--jobs` and `--pool-after`, `--jobs` alone starts its threads at
    /// once, and `--pool-after` beside it makes them wait.
    #[test]
    fn search_flags_set_their_keys() {
        use crate::argument_parsing::{Cli, Command};
        use clap::Parser;
        let settings = |args: &[&str]| {
            let line = [&["linlog", "prove"], args, &["A |- A"]].concat();
            let Command::Prove(prove) = Cli::try_parse_from(line).unwrap().command else {
                unreachable!("a prove command")
            };
            prove.settings(&Given::default()).unwrap()
        };
        let one = settings(&["--deterministic", "--jobs", "8", "--pool-after", "1s"]);
        assert_eq!(
            (
                one.search.jobs,
                one.search.schedule,
                one.clock.pool_after_ms
            ),
            (Jobs::Count(1), Schedule::Turns, None)
        );
        let eight = settings(&["--jobs", "8"]);
        assert_eq!(
            (eight.search.jobs, eight.clock.pool_after_ms),
            (Jobs::Count(8), None)
        );
        let later = settings(&["--jobs", "8", "--pool-after", "250ms"]);
        assert_eq!(later.clock.pool_after_ms, Some(250));
        let default = settings(&[]);
        assert_eq!(default.search.jobs, Settings::default().search.jobs);
        assert_eq!(
            default.clock.pool_after_ms,
            Some(linlog::Clock::DEFAULT_POOL_AFTER_MS)
        );
    }

    /// The threads a search gets: one when deterministic, a count from
    /// the start, and every thread after the default wait by default.
    #[test]
    fn threads_follow_the_settings() {
        let mut one = Settings::default().with_search(Options::default().with_jobs(1));
        one.clock.pool_after_ms = None;
        assert_eq!(
            Threads::of(&one),
            Threads {
                jobs: 1,
                alone: None
            }
        );
        let auto = Threads::of(&Settings::default());
        let machine = std::thread::available_parallelism().map_or(1, std::num::NonZero::get);
        assert_eq!(auto.jobs, machine.min(Options::MAX_JOBS));
        assert_eq!(
            auto.alone,
            (auto.jobs > 1).then(|| Duration::from_millis(linlog::Clock::DEFAULT_POOL_AFTER_MS))
        );
    }
}
