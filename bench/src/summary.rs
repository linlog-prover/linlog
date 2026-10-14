// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Markdown tables from the rows `run` wrote. A configuration is a CSV
//! file's name with its rows' mode, engine, threads and test period
//! (`base`), the portfolio of a file from before that column was removed,
//! and every option a row names that is not the default (`configuration`),
//! since every file of a baseline is one run of `run` with its own
//! options. A problem counts once per configuration: its first run gives
//! the verdict, the median over its runs the time. Rows of a forced engine
//! that does not apply (`refused`) are left out. `--strict` fails on a
//! row that a check must not pass (`Runs::failed`).

use anyhow::{Context, Result, bail};
use std::collections::HashMap;
use std::path::PathBuf;

/// One CSV row, as a map from column to value.
pub(crate) struct Row(HashMap<String, String>);

impl Row {
    /// The value of a column, empty when the column is absent.
    pub(crate) fn get(&self, column: &str) -> &str {
        self.0.get(column).map_or("", String::as_str)
    }
}

/// The runs of one problem in one configuration.
pub(crate) struct Runs<'a> {
    /// The first run.
    pub(crate) first: &'a Row,
    /// The time of every run, in milliseconds.
    pub(crate) times: Vec<f64>,
    /// Whether a sequential run waited for a CPU for more than a hundredth
    /// of its time and a millisecond: another process slowed it down.
    pub(crate) disturbed: bool,
}

/// Where a run without a decided verdict ended, as the tables count it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Ending {
    /// The time limit, or killed past it.
    Time,
    /// The copy bound.
    Bound,
    /// The recursion limit.
    Depth,
    /// The memory limit.
    Memory,
    /// An index past its width.
    Index,
    /// A context too wide to split, a reason of older files.
    Wide,
    /// Killed past the time limit and its grace, or while loading.
    Killed,
    /// The child crashed.
    Crash,
    /// An error, or a reason added since.
    Error,
}

impl Ending {
    /// Every ending, in the order the tables list them.
    pub(crate) const ALL: [Self; 9] = [
        Self::Time,
        Self::Bound,
        Self::Depth,
        Self::Memory,
        Self::Index,
        Self::Wide,
        Self::Killed,
        Self::Crash,
        Self::Error,
    ];

    /// Returns where the row's run ended, by its `reason` column.
    pub(crate) fn of(row: &Row) -> Self {
        match row.get("reason") {
            "timeout" => Self::Time,
            "copy_bound" => Self::Bound,
            "recursion_limit" => Self::Depth,
            "memory_limit" => Self::Memory,
            "index_limit" => Self::Index,
            "context_too_wide" => Self::Wide,
            "killed" | "killed while loading" => Self::Killed,
            reason if reason.starts_with("crash") => Self::Crash,
            _ => Self::Error,
        }
    }

    /// The ending in a word, as a table names it.
    pub(crate) const fn word(self) -> &'static str {
        match self {
            Self::Time => "time",
            Self::Bound => "bound",
            Self::Depth => "depth",
            Self::Memory => "memory",
            Self::Index => "index",
            Self::Wide => "wide",
            Self::Killed => "killed",
            Self::Crash => "crash",
            Self::Error => "error",
        }
    }
}

/// The counters of the engines, which a decided run on one thread has as
/// a function of its input.
pub(crate) const COUNTERS: [&str; 6] = [
    "nodes",
    "splits",
    "memo_hits",
    "memo_entries",
    "links",
    "tests",
];

/// Returns the part of a row's configuration that a comparison of
/// passes under other options keeps: its mode, requested engine and
/// threads, a portfolio and a test period.
pub(crate) fn base(row: &Row) -> String {
    let mut label = format!(
        "{} {} j{}",
        row.get("mode"),
        row.get("engine_requested"),
        row.get("jobs")
    );
    if row.get("portfolio") == "true" {
        label.push_str(" portfolio");
    }
    if !row.get("test_period").is_empty() {
        label.push_str(&format!(" period {}", row.get("test_period")));
    }
    label
}

/// Returns a row's configuration: its mode, requested engine and threads,
/// and every option its file names that is not the default (a portfolio,
/// a test period, a bias, a forward copy bound, a memory limit, a pool
/// started after a while), as a label; `summary` puts the file's name
/// before it and `--before` the file and family.
pub(crate) fn configuration(row: &Row) -> String {
    let mut label = base(row);
    if !matches!(row.get("bias"), "" | "auto") {
        label.push_str(&format!(" bias {}", row.get("bias")));
    }
    let forward = row.get("forward_copies");
    if !forward.is_empty() && forward != linlog::search::Options::DEFAULT_FORWARD_COPIES.to_string()
    {
        label.push_str(&format!(" forward {forward}"));
    }
    // A file from before the bound has no such column, and ran without
    // one.
    let memory = row.get("memory_limit");
    if memory == "0" {
        label.push_str(" memory none");
    } else if !memory.is_empty() && memory != linlog::Limits::DEFAULT_MEMORY_BYTES.to_string() {
        label.push_str(&format!(" memory {memory}"));
    }
    // A file from before the column ran every pool from its start.
    if !row.get("pool_after").is_empty() {
        label.push_str(&format!(" pool after {}s", row.get("pool_after")));
    }
    label
}

impl Runs<'_> {
    /// The median of the times.
    pub(crate) fn median(&self) -> f64 {
        let mut times = self.times.clone();
        times.sort_by(f64::total_cmp);
        times[times.len() / 2]
    }

    /// Whether the verdict is decided, in time or after it.
    pub(crate) fn decided(&self) -> bool {
        matches!(self.first.get("verdict"), "proved" | "unprovable")
    }

    /// Whether a decided verdict came after the time limit, which the
    /// grace before the kill allows.
    pub(crate) fn late(&self) -> bool {
        let limit: f64 = self.first.get("timeout_s").parse().unwrap_or(f64::INFINITY);
        self.decided() && self.first.get("time_ms").parse::<f64>().unwrap_or(0.0) > limit * 1000.0
    }

    /// Whether the verdict is decided within the time limit.
    pub(crate) fn solved(&self) -> bool {
        self.decided() && !self.late()
    }

    /// Whether the verdict contradicts the expected one, or a proof failed
    /// the checker: a bug for a generated problem, a bug or a wrong header
    /// for an LLTP one.
    fn wrong(&self) -> bool {
        let (verdict, expected) = (self.first.get("verdict"), self.first.get("expected"));
        // A check its bound gave up (`unchecked: …`) is no verdict on the
        // proof.
        let checked = self.first.get("checked");
        (verdict == "proved"
            && (expected == "unprovable"
                || (checked != "ok" && !checked.starts_with("unchecked:"))))
            || (verdict == "unprovable" && expected == "provable")
    }

    /// Whether the run is a finding rather than a configuration's
    /// result: a verdict against the known one, an error, a crash or a
    /// kill.
    pub(crate) fn failed(&self) -> bool {
        self.wrong()
            || self.first.get("verdict") == "error"
            || matches!(Ending::of(self.first), Ending::Crash | Ending::Killed) && !self.decided()
    }

    /// The cell of a scaling table: the median time and a mark for the
    /// verdict.
    pub(crate) fn cell(&self) -> String {
        let row = self.first;
        let mark = match (row.get("verdict"), Ending::of(row)) {
            ("proved", _) => "✓",
            ("unprovable", _) => "✗",
            (_, Ending::Time | Ending::Killed) => {
                return format!("> {}", seconds(row.get("timeout_s")));
            }
            (_, Ending::Bound) => "? bound",
            (_, Ending::Wide) => "? wide",
            (_, Ending::Depth) => "? depth",
            (_, Ending::Memory) => "? memory",
            (verdict, _) => verdict,
        };
        let wrong = if self.wrong() { " MISMATCH" } else { "" };
        let disturbed = if self.disturbed { " †" } else { "" };
        format!("{} {mark}{wrong}{disturbed}", time(self.median()))
    }
}

/// The runs of one configuration on one family.
struct Group<'a> {
    /// The family.
    family: String,
    /// The configuration's label.
    config: String,
    /// The runs of every problem, in the order first seen.
    problems: Vec<(String, Runs<'a>)>,
}

/// Prints the tables for the CSV files; with `strict`, fails after them
/// when a verdict contradicts the known one, a run ended in an error, a
/// crash or a kill, or there is no run.
pub fn summary(files: &[PathBuf], strict: bool) -> Result<()> {
    let rows = read(files)?;

    // Runs grouped by configuration and problem, in the order first seen.
    let config = |row: &Row| format!("{}: {}", row.get("file"), configuration(row));
    let mut groups: Vec<Group> = Vec::new();
    for row in &rows {
        let (family, label, problem) = (row.get("family"), config(row), row.get("problem"));
        let at = groups
            .iter()
            .position(|g| g.family == family && g.config == label);
        let at = at.unwrap_or_else(|| {
            groups.push(Group {
                family: family.to_owned(),
                config: label,
                problems: Vec::new(),
            });
            groups.len() - 1
        });
        let problems = &mut groups[at].problems;
        let time: f64 = row.get("time_ms").parse().unwrap_or(0.0);
        let wait: f64 = row.get("wait_ms").parse().unwrap_or(0.0);
        let disturbed = row.get("jobs") == "1" && wait > (time / 100.0).max(1.0);
        match problems.iter_mut().find(|(p, _)| p == problem) {
            Some((_, runs)) => {
                runs.times.push(time);
                runs.disturbed |= disturbed;
            }
            None => problems.push((
                problem.to_owned(),
                Runs {
                    first: row,
                    times: vec![time],
                    disturbed,
                },
            )),
        }
    }

    let disturbed = groups
        .iter()
        .flat_map(|g| &g.problems)
        .filter(|(_, r)| r.disturbed)
        .count();
    println!("## Solved within the time limit\n");
    println!(
        "Sequential problems that waited for a CPU for over 1 % of their time \
         (another process slowed them down; `†` in the tables below): {disturbed}.\n"
    );
    println!(
        "| family | configuration | engines | problems | solved | proved | refuted | late | timeout | bound | other | mismatch | median solved | total solved |"
    );
    println!("|---|---|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|");
    for Group {
        family,
        config: label,
        problems,
    } in &groups
    {
        let count = |test: &dyn Fn(&Runs) -> bool| problems.iter().filter(|(_, r)| test(r)).count();
        // Decided in time, by its verdict.
        let verdict = |v: &'static str| move |r: &Runs| r.solved() && r.first.get("verdict") == v;
        let ended =
            |e: &'static [Ending]| move |r: &Runs| !r.decided() && e.contains(&Ending::of(r.first));
        let late = count(&|r: &Runs| r.late());
        let timeouts = count(&ended(&[Ending::Time, Ending::Killed]));
        let bound = count(&ended(&[Ending::Bound]));
        let solved: Vec<f64> = problems
            .iter()
            .filter(|(_, r)| r.solved())
            .map(|(_, r)| r.median())
            .collect();
        let mut engines: Vec<&str> = problems
            .iter()
            .map(|(_, r)| r.first.get("engine"))
            .collect();
        engines.sort_unstable();
        engines.dedup();
        engines.retain(|e| !e.is_empty());
        let mut sorted = solved.clone();
        sorted.sort_by(f64::total_cmp);
        println!(
            "| {family} | {label} | {} | {} | {} | {} | {} | {late} | {timeouts} | {bound} | {} | {} | {} | {} |",
            engines.join(", "),
            problems.len(),
            solved.len(),
            count(&verdict("proved")),
            count(&verdict("unprovable")),
            problems.len() - solved.len() - late - timeouts - bound,
            count(&|r: &Runs| r.wrong()),
            sorted
                .get(sorted.len() / 2)
                .map_or("–".to_owned(), |&t| time(t)),
            if solved.is_empty() {
                "–".to_owned()
            } else {
                time(solved.iter().sum())
            },
        );
    }

    // The engines' counters summed over the problems decided in time, the
    // first run of each, which on one thread are a function of the input.
    println!("\n## Counters of the decided problems\n");
    println!(
        "| family | configuration | decided | {} |",
        COUNTERS.join(" | ")
    );
    println!("|---|---|--:|{}", "--:|".repeat(COUNTERS.len()));
    for group in &groups {
        let decided: Vec<&Runs> = group
            .problems
            .iter()
            .map(|(_, r)| r)
            .filter(|r| r.solved())
            .collect();
        let sums: Vec<String> = COUNTERS
            .iter()
            .map(|c| {
                let sum: u64 = decided
                    .iter()
                    .map(|r| r.first.get(c).parse::<u64>().unwrap_or(0))
                    .sum();
                sum.to_string()
            })
            .collect();
        println!(
            "| {} | {} | {} | {} |",
            group.family,
            group.config,
            decided.len(),
            sums.join(" | ")
        );
    }

    // One table per family of generated or listed problems: a row per
    // problem, a column per configuration.
    let mut families: Vec<&str> = Vec::new();
    for row in rows.iter().filter(|r| r.get("source") != "lltp") {
        if !families.contains(&row.get("family")) {
            families.push(row.get("family"));
        }
    }
    for family in families {
        let columns: Vec<&Group> = groups.iter().filter(|g| g.family == family).collect();
        let mut problems: Vec<&str> = Vec::new();
        for group in &columns {
            for (problem, _) in &group.problems {
                if !problems.contains(&problem.as_str()) {
                    problems.push(problem);
                }
            }
        }
        println!("\n## {family}\n");
        let labels: Vec<&str> = columns.iter().map(|g| g.config.as_str()).collect();
        println!("| problem | {} |", labels.join(" | "));
        println!("|---|{}", "--:|".repeat(labels.len()));
        for problem in problems {
            let cells: Vec<String> = columns
                .iter()
                .map(|group| {
                    group
                        .problems
                        .iter()
                        .find(|(p, _)| p == problem)
                        .map_or("".to_owned(), |(_, r)| r.cell())
                })
                .collect();
            println!("| {problem} | {} |", cells.join(" | "));
        }
    }
    if strict {
        let failed: Vec<String> = groups
            .iter()
            .flat_map(|g| &g.problems)
            .filter(|(_, r)| r.failed())
            .map(|(problem, r)| {
                format!(
                    "{problem} ({}: {} {})",
                    r.first.get("file"),
                    r.first.get("verdict"),
                    r.first.get("reason")
                )
            })
            .collect();
        if rows.is_empty() {
            bail!("the files hold no run");
        }
        if !failed.is_empty() {
            bail!("{} runs failed: {}", failed.len(), failed.join(", "));
        }
    }
    Ok(())
}

/// The rows of the CSV files, each with its file's name in the column
/// `file`, without those of a forced engine that does not apply.
pub(crate) fn read(files: &[PathBuf]) -> Result<Vec<Row>> {
    let mut rows = Vec::new();
    for file in files {
        let text =
            std::fs::read_to_string(file).with_context(|| format!("reading {}", file.display()))?;
        let mut lines = text.lines();
        let Some(header) = lines.next() else { continue };
        let columns: Vec<&str> = header.split(',').collect();
        let stem = file.file_stem().unwrap_or_default().to_string_lossy();
        for line in lines.filter(|l| !l.is_empty() && !l.starts_with("source,")) {
            let values: Vec<&str> = line.split(',').collect();
            if values.len() != columns.len() {
                bail!(
                    "{}: a row of {} fields: {line}",
                    file.display(),
                    values.len()
                );
            }
            let row = columns
                .iter()
                .zip(values)
                .map(|(c, v)| ((*c).to_owned(), v.to_owned()));
            let mut row: HashMap<String, String> = row.collect();
            row.insert("file".to_owned(), stem.to_string());
            rows.push(Row(row));
        }
    }
    rows.retain(|row| row.get("verdict") != "refused");
    Ok(rows)
}

/// Formats milliseconds for a table: microseconds below one millisecond,
/// milliseconds below a second, seconds above.
pub(crate) fn time(ms: f64) -> String {
    if ms < 1.0 {
        format!("{:.0} µs", ms * 1000.0)
    } else if ms < 1000.0 {
        format!("{ms:.1} ms")
    } else {
        format!("{:.2} s", ms / 1000.0)
    }
}

/// Formats a timeout in seconds, as the CSV holds it.
fn seconds(value: &str) -> String {
    format!("{} s", value.parse::<f64>().unwrap_or(0.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A row of the columns given, the others empty.
    pub(crate) fn row(columns: &[(&str, &str)]) -> Row {
        Row(columns
            .iter()
            .map(|(c, v)| ((*c).to_owned(), (*v).to_owned()))
            .collect())
    }

    /// The runs of rows, the first one's verdict standing for them.
    fn runs(rows: &[Row]) -> Runs<'_> {
        Runs {
            first: &rows[0],
            times: rows
                .iter()
                .map(|r| r.get("time_ms").parse().unwrap())
                .collect(),
            disturbed: false,
        }
    }

    /// A problem's verdict is its first run's and its time the median of
    /// its runs'; one decided after its time limit is decided but late,
    /// not solved; a verdict against the known one, a failed check, an
    /// error and a crash are failures, a run stopped at its limit is not.
    /// `--strict` fails after the tables on a run against its known
    /// verdict and on files without a run, and passes the others.
    #[test]
    fn strict_fails_on_a_failed_run_or_none() {
        let dir = std::env::temp_dir().join(format!("linlog-strict-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let header = "family,problem,verdict,expected,reason,time_ms,timeout_s,checked";
        let file = |name: &str, rows: &str| {
            let path = dir.join(name);
            std::fs::write(&path, format!("{header}\n{rows}")).unwrap();
            path
        };
        let right = file("right.csv", "f,p,proved,proved,,1,1,ok\n");
        let wrong = file("wrong.csv", "f,p,proved,unprovable,,1,1,ok\n");
        let empty = file("empty.csv", "");
        assert!(summary(std::slice::from_ref(&right), true).is_ok());
        let failed = summary(std::slice::from_ref(&wrong), true).unwrap_err();
        assert!(
            failed.to_string().starts_with("1 runs failed: p"),
            "{failed}"
        );
        assert!(summary(std::slice::from_ref(&wrong), false).is_ok());
        let none = summary(&[empty], true).unwrap_err();
        assert_eq!(none.to_string(), "the files hold no run");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn runs_verdicts_and_endings() {
        let timed = |verdict: &str, ms: &str| {
            row(&[
                ("verdict", verdict),
                ("time_ms", ms),
                ("timeout_s", "1"),
                ("checked", "ok"),
            ])
        };
        let rows = [
            timed("proved", "30"),
            timed("unknown", "10"),
            timed("proved", "20"),
        ];
        let three = runs(&rows);
        assert!(three.solved() && !three.late());
        assert!((three.median() - 20.0).abs() < 1e-9);
        let late = [timed("unprovable", "1500")];
        assert!(runs(&late).decided() && runs(&late).late() && !runs(&late).solved());
        let wrong = [row(&[
            ("verdict", "proved"),
            ("expected", "unprovable"),
            ("time_ms", "1"),
        ])];
        assert!(runs(&wrong).failed());
        let unchecked = [row(&[
            ("verdict", "proved"),
            ("checked", "failed: killed"),
            ("time_ms", "1"),
        ])];
        assert!(runs(&unchecked).failed());
        let refused = [row(&[
            ("verdict", "proved"),
            ("checked", "unchecked: memory limit"),
            ("time_ms", "1"),
        ])];
        assert!(!runs(&refused).failed());
        for (reason, ending, failed) in [
            ("timeout", Ending::Time, false),
            ("copy_bound", Ending::Bound, false),
            ("memory_limit", Ending::Memory, false),
            ("index_limit", Ending::Index, false),
            ("killed", Ending::Killed, true),
            ("crash (signal 6): out of memory", Ending::Crash, true),
        ] {
            let stopped = [row(&[
                ("verdict", "unknown"),
                ("reason", reason),
                ("time_ms", "1"),
            ])];
            assert_eq!(Ending::of(&stopped[0]), ending, "{reason}");
            assert_eq!(runs(&stopped).failed(), failed, "{reason}");
        }
        let error = [row(&[
            ("verdict", "error"),
            ("reason", "no such file"),
            ("time_ms", "1"),
        ])];
        assert!(runs(&error).failed());
    }

    /// A configuration names every option a row's file names that is not
    /// the default, and the base the options a comparison of passes keeps.
    #[test]
    fn configurations() {
        let plain = row(&[
            ("mode", "classical"),
            ("engine_requested", "auto"),
            ("jobs", "1"),
        ]);
        assert_eq!(configuration(&plain), "classical auto j1");
        let rarer = row(&[
            ("mode", "classical"),
            ("engine_requested", "auto"),
            ("jobs", "4"),
            ("bias", "rarer"),
            ("memory_limit", "0"),
            ("pool_after", "0.1"),
        ]);
        assert_eq!(
            configuration(&rarer),
            "classical auto j4 bias rarer memory none pool after 0.1s"
        );
        assert_eq!(base(&rarer), "classical auto j4");
    }

    /// A CSV file reads into rows by its header's names, with its name
    /// in `file` and its refused rows left out; a row of another width is
    /// an error.
    #[test]
    fn files_read_by_their_header() {
        let dir = std::env::temp_dir().join(format!("linlog-bench-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("pass.csv");
        std::fs::write(&file, "problem,verdict\na,proved\nb,refused\n").unwrap();
        let rows = read(std::slice::from_ref(&file)).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!((rows[0].get("problem"), rows[0].get("file")), ("a", "pass"));
        std::fs::write(&file, "problem,verdict\na\n").unwrap();
        assert!(read(std::slice::from_ref(&file)).is_err());
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
