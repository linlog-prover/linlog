// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The settings a command runs under, as the library's one value: the
//! command's defaults, then the JSON of `--settings FILE` over them, then
//! the flags given on the command line, each the spelling of one key.

use crate::argument_parsing::{
    CheckArgs, CoresArg, EliminateArgs, InteractArgs, LogicArgs, ProveArgs, SearchArgs,
};
use crate::io;
use anyhow::{Context, Result};
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
    /// Returns whether the flag of argument `id` was given on the command
    /// line.
    pub(crate) fn has(&self, id: &str) -> bool {
        self.0.contains(id)
    }

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

#[cfg(test)]
thread_local! {
    /// The ids `Apply::set` was asked about, for the test that each is an
    /// argument of its command: a misspelt id would never be given.
    static ASKED: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
}

impl Apply<'_> {
    /// Whether the flag of this argument sets its key.
    fn set(&self, id: &str) -> bool {
        #[cfg(test)]
        ASKED.with_borrow_mut(|asked| asked.push(id.to_owned()));
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
        if apply.set("classical_work") {
            search.classical_work = self.classical_work;
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
        // The mode flags together are the batch's mode; each is asked, so
        // that one given overrides the file's.
        if ["intuitionistic", "affine", "mix"]
            .map(|id| apply.set(id))
            .contains(&true)
        {
            settings.batch.mode = self.mode.mode();
        }
        if apply.set("occurrence_limit") {
            settings.limits.occurrences = self.input.occurrence_limit.0;
        }
        if apply.set("ill_syntax") {
            settings.input.intuitionistic = self.input.ill_syntax.into();
        }
        if apply.set("derivation_limit") {
            settings.limits.derivation_bytes = self.output.derivation_limit.0;
        }
        if apply.set("compact") {
            settings.view.compact = self.output.compact.into();
        }
        if apply.set("classical_sides") {
            settings.view.classical = self.output.classical_sides.into();
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
        if apply.set("ill_syntax") {
            settings.input.intuitionistic = self.input.ill_syntax.into();
        }
        if apply.set("derivation_limit") {
            settings.limits.derivation_bytes = self.derivation_limit.0;
        }
        if apply.set("invertible") {
            settings.interactive.invertible = self.invertible;
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
        if apply.set("classical_sides") {
            settings.view.classical = self.output.classical_sides.into();
        }
        Ok(settings)
    }
}

impl EliminateArgs {
    /// Returns the settings `eliminate` runs under: `check`'s, and the
    /// bounds of the elimination.
    ///
    /// # Errors
    ///
    /// Those of reading the settings file.
    pub(crate) fn settings(&self, given: &Given) -> Result<Settings> {
        let mut settings = self.check.settings(given)?;
        let apply = Apply {
            file: self.check.settings.is_some(),
            given,
        };
        if apply.set("steps") {
            settings.cut.steps = self.steps.0;
        }
        if apply.set("node_limit") {
            settings.cut.nodes = self.node_limit.0;
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
    syntax: linlog::IllSyntax,
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
    if apply.set("ill_syntax") {
        settings.input.intuitionistic = syntax;
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
    use crate::argument_parsing::{Cli, Command};
    use clap::{CommandFactory, FromArgMatches};

    /// Returns the settings `prove` runs under with `args` before its
    /// sequent, and `--settings FILE` first where a file is given.
    fn prove_settings(file: Option<&Path>, args: &[&str]) -> Settings {
        let mut line = vec!["linlog", "prove"];
        if let Some(file) = file {
            line.extend(["--settings", file.to_str().unwrap()]);
        }
        line.extend(args);
        line.push("A |- A");
        let matches = Cli::command().try_get_matches_from(line).unwrap();
        let Command::Prove(prove) = Cli::from_arg_matches(&matches).unwrap().command else {
            unreachable!("a prove command")
        };
        prove.settings(&Given::of(&matches)).unwrap()
    }

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
        let file =
            std::env::temp_dir().join(format!("linlog-settings-{}.json", std::process::id()));
        std::fs::write(&file, r#"{"search": {"copies": 0}, "limits": {"work": 7}}"#).unwrap();
        let settings = |args: &[&str]| prove_settings(Some(&file), args);
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

    /// Returns the settings a whole command line runs under, as `main`
    /// makes them.
    fn settings_of(line: &[&str]) -> Settings {
        let matches = Cli::command().try_get_matches_from(line).unwrap();
        let given = Given::of(&matches);
        match Cli::from_arg_matches(&matches).unwrap().command {
            Command::Prove(args) => args.settings(&given).unwrap(),
            Command::Interact(args) => args.settings(&given).unwrap(),
            Command::Check(args) => args.settings(&given).unwrap(),
            Command::Eliminate(args) => args.settings(&given).unwrap(),
            Command::Seq {
                command:
                    crate::argument_parsing::SeqCommand::Print {
                        settings,
                        memory_limit,
                        input,
                        ..
                    },
            } => of_print(
                settings.as_deref(),
                memory_limit.0,
                input.occurrence_limit.0,
                input.ill_syntax.into(),
                &given,
            )
            .unwrap(),
            Command::Seq { .. } => unreachable!("only `seq print` reads settings"),
        }
    }

    /// Every key a flag sets, on every command that reads settings: the
    /// file's value stands where the flag is not given and the flag's
    /// where it is, so that an argument's id and the name `Apply::set`
    /// asks about cannot drift apart unseen.
    #[test]
    fn each_flag_overrides_its_key_in_a_file() {
        // A line per key: the flag with its value, the key's place in the
        // settings' JSON, then the file's value and the flag's in JSON.
        const SEARCH: &str = r#"
            --bias factors          /search/bias               "rarer"           "factors"
            --copies 4              /search/copies             2                 4
            --forward-copies 7      /search/forward_copies     5                 7
            --memo-limit 9          /search/memo_limit         8                 9
            --classical-work 4      /search/classical_work     3                 4
            --test-period 3         /search/test_period        2                 3
            --schedule auto         /search/schedule           "turns"           "auto"
            --jobs 3                /search/jobs               2                 3
            --jobs 3                /clock/pool_after_ms       250               null
            --pool-after 1s         /clock/pool_after_ms       250               1000
            --deterministic         /search/jobs               2                 1
            --deterministic         /clock/pool_after_ms       250               null
            --timeout 3s            /clock/time_limit_ms       5000              3000
            --memory-limit 2MiB     /limits/memory_bytes       1048576           2097152
            --recursion-limit 99    /limits/recursion_depth    77                99
            --work-limit 6          /limits/work               5                 6
            --occurrence-limit 12   /limits/occurrences        11                12
            --derivation-limit 3KiB /limits/derivation_bytes   1024              3072"#;
        const PROVE: &str = r#"
            --engine focus          /search/engine             "two-sided"       "focus"
            --fragment mall         /search/fragment           "LL"              "MALL"
            --no-check              /search/check              true              false
            --affine                /batch/mode                "intuitionistic"  "affine"
            --mix                   /batch/mode                "intuitionistic"  "mix"
            -i --affine             /batch/mode                "intuitionistic"  "intuitionistic-affine"
            --workers 3             /batch/workers             2                 3
            --batch-memory 2MiB     /batch/total_memory_bytes  1048576           2097152
            --compact never         /view/compact              "always"          "never"
            --cores within          /batch/cores               "across"          "within"
            --ill-syntax one-sided  /input/intuitionistic      "strict"          "one-sided"
            --batch-timeout 2s      /clock/batch_time_limit_ms 1000              2000"#;
        const CHECK: &str = r#"
            --memory-limit 2MiB     /limits/memory_bytes       1048576           2097152
            --timeout 3s            /clock/time_limit_ms       5000              3000
            --work-limit 6          /limits/work               5                 6
            --occurrence-limit 12   /limits/occurrences        11                12
            --derivation-limit 3KiB /limits/derivation_bytes   1024              3072
            --compact never         /view/compact              "always"          "never""#;
        const ELIMINATE: &str = r#"
            --steps 4               /cut/steps                 3                 4
            --node-limit 9          /cut/nodes                 8                 9"#;
        const INTERACT: &str = r#"
            --invertible            /interactive/invertible    false             true
            --ill-syntax one-sided  /input/intuitionistic      "strict"          "one-sided""#;
        const PRINT: &str = r#"
            --memory-limit 2MiB     /limits/memory_bytes       1048576           2097152
            --occurrence-limit 12   /limits/occurrences        11                12
            --ill-syntax one-sided  /input/intuitionistic      "strict"          "one-sided""#;
        let file =
            std::env::temp_dir().join(format!("linlog-settings-flags-{}.json", std::process::id()));
        let path = file.to_str().unwrap();
        for (command, operand, table) in [
            (&["prove"][..], "A |- A", [SEARCH, PROVE].concat()),
            (&["interact"], "A |- A", [SEARCH, INTERACT].concat()),
            (&["check"], "proof.json", CHECK.to_owned()),
            (&["eliminate"], "proof.json", [CHECK, ELIMINATE].concat()),
            (&["seq", "print"], "A |- A", PRINT.to_owned()),
        ] {
            let cases: Vec<(Vec<&str>, &str, Value, Value)> = table
                .lines()
                .filter(|line| !line.trim().is_empty())
                .map(|line| {
                    let words: Vec<&str> = line.split_whitespace().collect();
                    let at = words.iter().position(|w| w.starts_with('/')).unwrap();
                    let json = |w: &str| serde_json::from_str(w).unwrap();
                    (
                        words[..at].to_vec(),
                        words[at],
                        json(words[at + 1]),
                        json(words[at + 2]),
                    )
                })
                .collect();
            let mut written = Value::Object(serde_json::Map::new());
            for (_, key, value, _) in &cases {
                let (object, field) = key.rsplit_once('/').unwrap();
                let mut at = &mut written;
                for part in object.split('/').skip(1) {
                    at = at
                        .as_object_mut()
                        .unwrap()
                        .entry(part)
                        .or_insert(Value::Object(serde_json::Map::new()));
                }
                at[field] = value.clone();
            }
            std::fs::write(&file, written.to_string()).unwrap();
            let read = |flag: &[&str]| {
                let line = [
                    &["linlog"],
                    command,
                    &["--settings", path],
                    flag,
                    &[operand],
                ]
                .concat();
                serde_json::to_value(settings_of(&line)).unwrap()
            };
            let unflagged = read(&[]);
            for (flag, key, in_file, by_flag) in &cases {
                let without = format!("{command:?} without {flag:?}");
                assert_eq!(unflagged.pointer(key), Some(in_file), "{without}");
                assert_eq!(
                    read(flag).pointer(key),
                    Some(by_flag),
                    "{command:?} {flag:?}"
                );
            }
        }
        std::fs::remove_file(&file).unwrap();
    }

    /// A settings file's `ordinary` applies under `--logic` alone, which
    /// names the logic: the file's `ordinary.logic` is not read, since the
    /// whole settings a front end keeps always hold one, and its
    /// translation is taken unless `--translation` names another.
    #[test]
    fn ordinary_applies_under_logic_alone() {
        use linlog::ordinary::{Logic, Translation};
        let file = std::env::temp_dir().join(format!(
            "linlog-settings-ordinary-{}.json",
            std::process::id()
        ));
        std::fs::write(
            &file,
            r#"{"ordinary": {"logic": "classical", "translation": "cbv"}}"#,
        )
        .unwrap();
        let ordinary_of = |args: &[&str]| {
            let line = [
                &["linlog", "prove", "--settings", file.to_str().unwrap()],
                args,
                &["a -> a"],
            ]
            .concat();
            let matches = Cli::command().try_get_matches_from(line).unwrap();
            let Command::Prove(prove) = Cli::from_arg_matches(&matches).unwrap().command else {
                unreachable!("a prove command")
            };
            let settings = prove.settings(&Given::of(&matches)).unwrap();
            ordinary(&settings, &prove.logic).map(|o| (o.logic, o.translation))
        };
        assert_eq!(ordinary_of(&[]), None);
        assert_eq!(
            ordinary_of(&["--logic", "intuitionistic"]),
            Some((Logic::Intuitionistic, Some(Translation::CallByValue)))
        );
        assert_eq!(
            ordinary_of(&["--logic", "intuitionistic", "--translation", "01"]),
            Some((Logic::Intuitionistic, Some(Translation::ZeroOne)))
        );
        std::fs::remove_file(&file).unwrap();
    }

    /// Every id a command asks `Apply::set` about is an argument of that
    /// command, so that a flag given on the command line can be found.
    #[test]
    fn every_id_asked_is_an_argument() {
        use clap::Parser;
        let cli = Cli::command();
        for line in [
            &["linlog", "prove", "A |- A"][..],
            &["linlog", "interact", "A |- A"],
            &["linlog", "check", "proof.json"],
            &["linlog", "eliminate", "proof.json"],
            &["linlog", "seq", "print", "A |- A"],
        ] {
            ASKED.with_borrow_mut(Vec::clear);
            let given = Given::default();
            match Cli::try_parse_from(line).unwrap().command {
                Command::Prove(args) => drop(args.settings(&given).unwrap()),
                Command::Interact(args) => drop(args.settings(&given).unwrap()),
                Command::Check(args) => drop(args.settings(&given).unwrap()),
                Command::Eliminate(args) => drop(args.settings(&given).unwrap()),
                Command::Seq { .. } => {
                    drop(of_print(None, None, None, linlog::IllSyntax::Strict, &given).unwrap())
                }
            }
            let mut leaf = &cli;
            for word in &line[1..] {
                if let Some(sub) = leaf.find_subcommand(word) {
                    leaf = sub;
                }
            }
            let ids: HashSet<&str> = leaf.get_arguments().map(|a| a.get_id().as_str()).collect();
            let asked = ASKED.with_borrow(Clone::clone);
            assert!(!asked.is_empty(), "{line:?}");
            for id in asked {
                assert!(ids.contains(id.as_str()), "{id} is no argument of {line:?}");
            }
        }
    }

    /// The library's own settings, as a front end keeps them, read as a
    /// file; its `batch.mode` is the batch's unless a mode flag is given;
    /// and the file is never standard input, which carries the input.
    #[test]
    fn a_file_is_the_library_s_settings() {
        let file =
            std::env::temp_dir().join(format!("linlog-settings-keys-{}.json", std::process::id()));
        let mut kept = Settings::default();
        kept.batch.mode = linlog::Mode::CLASSICAL.with_affine();
        std::fs::write(&file, serde_json::to_string(&kept).unwrap()).unwrap();
        let mode = |args: &[&str]| prove_settings(Some(&file), args).batch.mode;
        assert_eq!(mode(&[]), kept.batch.mode);
        assert_eq!(mode(&["--intuitionistic"]), linlog::Mode::INTUITIONISTIC);
        std::fs::remove_file(&file).unwrap();
        let error = read(Some(Path::new("-"))).unwrap_err().to_string();
        assert!(error.contains("standard input"), "{error}");
    }

    /// The search flags set their keys: `--deterministic` wins over
    /// `--jobs` and `--pool-after`, `--jobs` alone starts its threads at
    /// once, and `--pool-after` beside it makes them wait.
    #[test]
    fn search_flags_set_their_keys() {
        let settings = |args: &[&str]| prove_settings(None, args);
        let one = settings(&["--deterministic", "--jobs", "8", "--pool-after", "1s"]);
        assert_eq!(
            (
                one.search.jobs,
                one.search.schedule,
                one.clock.pool_after_ms
            ),
            (Jobs::Count(1), Schedule::Turns, None)
        );
        // The batch's flags set their keys, so a flag ignored fails here
        // where a batch's verdicts could not tell.
        let batch = settings(&[
            "--cores",
            "within",
            "--batch-memory",
            "1GiB",
            "--workers",
            "3",
        ]);
        assert_eq!(
            (
                batch.batch.cores,
                batch.batch.total_memory_bytes,
                batch.batch.workers
            ),
            (Cores::Within, Some(1 << 30), 3)
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
