// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Markdown that compares two sets of rows problem by problem: two
//! baselines, each file with the file of the same name in the other
//! (`summary --before`), or the files of one baseline with one of them
//! (`summary --against`, as the passes under one atom bias against the
//! default). A problem's outcome in a configuration is its first run's
//! verdict and the median of its runs' times, as in the summary; a
//! verdict found after the time limit (the grace before the kill allows
//! it) counts as late and is kept apart.

use crate::summary::{COUNTERS, Ending, Row, Runs, base, configuration, read};
use anyhow::{Result, bail};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// The outcomes of one set of rows: by group (file, family and
/// configuration) and problem, in sorted order.
type Outcomes<'a> = BTreeMap<(String, String), Runs<'a>>;

/// Prints the comparison of every file with the file of the same name in
/// the directory `before`, followed by the outcomes of the files without
/// one beside each problem decided before and not after.
pub fn before(dir: &Path, files: &[PathBuf]) -> Result<()> {
    let (mut old, mut new, mut extra) = (Vec::new(), Vec::new(), Vec::new());
    for file in files {
        let counterpart = dir.join(file.file_name().unwrap_or_default());
        if counterpart.exists() {
            old.push(counterpart);
            new.push(file.clone());
        } else {
            extra.push(file.clone());
        }
    }
    if new.is_empty() {
        bail!("no file has a counterpart in {}", dir.display());
    }
    let (old, new, extra) = (read(&old)?, read(&new)?, read(&extra)?);
    let beside: Vec<(String, Outcomes)> = files_of(&extra)
        .into_iter()
        .map(|name| {
            let rows: Vec<&Row> = extra.iter().filter(|r| r.get("file") == name).collect();
            (name, outcomes(&rows, Grouping::Configurations))
        })
        .collect();
    let old: Vec<&Row> = old.iter().collect();
    let new: Vec<&Row> = new.iter().collect();
    compare(
        &outcomes(&old, Grouping::Files),
        &outcomes(&new, Grouping::Files),
        ("before", "after"),
        &beside,
        false,
        false,
    );
    Ok(())
}

/// Prints the comparison of every other file with `reference`, problem by
/// problem whatever the files' names, with every problem whose counters
/// differ where `counters` asks.
pub fn against(reference: &Path, files: &[PathBuf], counters: bool) -> Result<()> {
    let main = read(&[reference.to_path_buf()])?;
    let main: Vec<&Row> = main.iter().collect();
    let main = outcomes(&main, Grouping::Problems);
    let name = stem(reference);
    for file in files.iter().filter(|f| f.as_path() != reference) {
        let rows = read(std::slice::from_ref(file))?;
        let rows: Vec<&Row> = rows.iter().collect();
        println!("## {} against {name}\n", stem(file));
        compare(
            &outcomes(&rows, Grouping::Problems),
            &main,
            (&stem(file), &name),
            &[],
            counters,
            true,
        );
    }
    Ok(())
}

/// A file's name without its extension.
fn stem(file: &Path) -> String {
    file.file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}

/// The names of the files the rows come from, in the order first seen.
fn files_of(rows: &[Row]) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for row in rows {
        if !names.iter().any(|n| n == row.get("file")) {
            names.push(row.get("file").to_owned());
        }
    }
    names
}

/// How rows are grouped for a comparison.
#[derive(Clone, Copy)]
enum Grouping {
    /// By file, family and whole configuration: two baselines, file by
    /// file.
    Files,
    /// By family and whole configuration, whatever the file.
    Configurations,
    /// By family, mode, engine and threads alone: files of one baseline
    /// that differ in an option (a bias, say), problem by problem.
    Problems,
}

/// The group of a row: its file, family and configuration as `grouping`
/// takes them, as the columns of a table, separated by `|`.
fn group(row: &Row, grouping: Grouping) -> String {
    let (file, config) = match grouping {
        Grouping::Files => (row.get("file"), configuration(row)),
        Grouping::Configurations => ("", configuration(row)),
        Grouping::Problems => ("", base(row)),
    };
    format!("{file} | {} | {config}", row.get("family"))
}

/// The outcome of every problem in every group of the rows.
fn outcomes<'a>(rows: &[&'a Row], grouping: Grouping) -> Outcomes<'a> {
    let mut outcomes = Outcomes::new();
    for row in rows {
        let key = (group(row, grouping), row.get("problem").to_owned());
        let time: f64 = row.get("time_ms").parse().unwrap_or(0.0);
        outcomes
            .entry(key)
            .and_modify(|runs| runs.times.push(time))
            .or_insert(Runs {
                first: row,
                times: vec![time],
                disturbed: false,
            });
    }
    outcomes
}

/// The undecided outcomes counted by where they ended, as `12 time, 3
/// bound`, or `–`.
fn endings<'a>(outcomes: impl Iterator<Item = &'a Runs<'a>>) -> String {
    let mut counts: BTreeMap<Ending, usize> = BTreeMap::new();
    for runs in outcomes.filter(|r| !r.decided()) {
        *counts.entry(Ending::of(runs.first)).or_default() += 1;
    }
    let parts: Vec<String> = Ending::ALL
        .iter()
        .filter_map(|e| counts.get(e).map(|n| format!("{n} {}", e.word())))
        .collect();
    if parts.is_empty() {
        "–".to_owned()
    } else {
        parts.join(", ")
    }
}

/// A count with the late ones among them in parentheses.
fn decided<'a>(outcomes: impl Iterator<Item = &'a Runs<'a>>) -> String {
    let (mut all, mut later) = (0, 0);
    for runs in outcomes.filter(|r| r.decided()) {
        all += 1;
        later += usize::from(runs.late());
    }
    if later > 0 {
        format!("{all} ({later} late)")
    } else {
        all.to_string()
    }
}

/// The median of the ratios, as `1.23× (n)`, or `–`.
fn median_ratio(mut ratios: Vec<f64>) -> String {
    if ratios.is_empty() {
        return "–".to_owned();
    }
    ratios.sort_by(f64::total_cmp);
    format!("{} ({})", ratio(ratios[ratios.len() / 2]), ratios.len())
}

/// A ratio of two times, in scientific notation below a tenth.
fn ratio(value: f64) -> String {
    if value >= 0.1 {
        format!("{value:.2}×")
    } else {
        format!("{value:.1e}×")
    }
}

/// Prints the comparison of the outcomes `first` and `second`, named
/// `names`: the verdicts that differ, the problems decided by the first
/// and not by the second (with the outcomes `beside` of the same problem),
/// and with `both_ways` those decided by the second and not the first,
/// one row per group, and every problem that is not from LLTP side by
/// side; how many decided runs on one thread differ in their counters,
/// and with `counters` which (a set of rows that must not change the
/// search, against the one it is compared with, as the target set is).
fn compare(
    first: &Outcomes,
    second: &Outcomes,
    names: (&str, &str),
    beside: &[(String, Outcomes)],
    counters: bool,
    both_ways: bool,
) {
    let (one, two) = names;
    let both: Vec<(&(String, String), &Runs, &Runs)> = first
        .iter()
        .filter_map(|(key, a)| second.get(key).map(|b| (key, a, b)))
        .collect();
    let verdict = |r: &Runs| r.first.get("verdict").to_owned();

    println!("### Verdicts that differ\n");
    let differ: Vec<_> = both
        .iter()
        .filter(|(_, a, b)| a.decided() && b.decided() && verdict(a) != verdict(b))
        .collect();
    if differ.is_empty() {
        println!("None.\n");
    } else {
        println!(
            "| file | family | configuration | problem | {one} | {two} |\n|---|---|---|---|--:|--:|"
        );
        for ((group, problem), a, b) in differ {
            println!("| {group} | {problem} | {} | {} |", a.cell(), b.cell());
        }
        println!();
    }

    // On one thread the counters of a run decided alike are a function of
    // its input: where they differ, the search did.
    println!("### Counters that differ\n");
    let counted: Vec<_> = both
        .iter()
        .filter(|(_, a, b)| {
            a.decided()
                && b.decided()
                && verdict(a) == verdict(b)
                && a.first.get("jobs") == "1"
                && b.first.get("jobs") == "1"
        })
        .collect();
    let differ: Vec<_> = counted
        .iter()
        .filter(|(_, a, b)| COUNTERS.iter().any(|c| a.first.get(c) != b.first.get(c)))
        .collect();
    println!(
        "Of the {} problems both decide alike on one thread, {} differ in {}.\n",
        counted.len(),
        differ.len(),
        COUNTERS.join(", ")
    );
    if counters && !differ.is_empty() {
        println!(
            "| file | family | configuration | problem | counter | {one} | {two} |\n|---|---|---|---|---|--:|--:|"
        );
        for ((group, problem), a, b) in differ {
            for c in COUNTERS.iter().filter(|c| a.first.get(c) != b.first.get(c)) {
                println!(
                    "| {group} | {problem} | {c} | {} | {} |",
                    a.first.get(c),
                    b.first.get(c)
                );
            }
        }
        println!();
    }

    // `--against` names the problems its reference decides and the
    // other file does not, the losses, as well as the gains.
    let directions: &[bool] = if both_ways { &[false, true] } else { &[false] };
    for &reverse in directions {
        let (decider, other) = if reverse { (two, one) } else { (one, two) };
        println!("### Decided by {decider}, not by {other}\n");
        let lost: Vec<_> = both
            .iter()
            .filter(|(_, a, b)| {
                if reverse {
                    b.decided() && !a.decided()
                } else {
                    a.decided() && !b.decided()
                }
            })
            .collect();
        if lost.is_empty() {
            println!("None.\n");
        } else {
            let extra: Vec<&str> = beside.iter().map(|(n, _)| n.as_str()).collect();
            let columns = if extra.is_empty() {
                String::new()
            } else {
                format!(" {} |", extra.join(" | "))
            };
            println!(
                "| file | family | configuration | problem | {one} | {two} |{columns}\n|---|---|---|---|--:|--:|{}",
                "--:|".repeat(extra.len())
            );
            for ((group, problem), a, b) in lost {
                // The same problem in the files without a counterpart, which
                // group it without a file.
                let unfiled = group.split_once(" | ").map_or("", |(_, rest)| rest);
                let cells: String = beside
                    .iter()
                    .map(|(_, o)| {
                        let key = (format!(" | {unfiled}"), problem.clone());
                        format!(" {} |", o.get(&key).map_or(String::new(), Runs::cell))
                    })
                    .collect();
                println!(
                    "| {group} | {problem} | {} | {} |{cells}",
                    a.cell(),
                    b.cell()
                );
            }
            println!();
        }
    }

    println!("### By group\n");
    println!(
        "Problems in both; decided (late: after the time limit); decided by one \
         and not the other; where the undecided ended (`time` the time limit or \
         `killed` past it, `bound` the copy bound, `depth` the recursion limit, \
         `memory` the memory limit, `index` an index past its width, `wide` a \
         context too wide to split, `crash` out of memory); the median \
         ratio of {two}'s time to {one}'s on the problems both decide within the \
         limit (how many); and the problems only {two} has.\n"
    );
    println!(
        "| file | family | configuration | problems | {one} decided | {two} decided | only {one} | only {two} | {one} ended | {two} ended | {two}/{one} | new |"
    );
    println!("|---|---|---|--:|--:|--:|--:|--:|---|---|--:|--:|");
    let mut groups: Vec<&String> = second.keys().map(|(g, _)| g).collect();
    groups.dedup();
    for group in groups {
        let pairs: Vec<_> = both.iter().filter(|((g, _), _, _)| g == group).collect();
        let new = second
            .keys()
            .filter(|(g, p)| g == group && !first.contains_key(&(g.clone(), p.clone())))
            .count();
        if pairs.is_empty() {
            println!("| {group} | 0 | | | | | | | | {new} |");
            continue;
        }
        // Decided by the first and not the second (`true`), or the reverse.
        let only = |by_first: bool| {
            pairs
                .iter()
                .filter(|(_, a, b)| a.decided() == by_first && b.decided() != by_first)
                .count()
        };
        let ratios = pairs
            .iter()
            .filter(|(_, a, b)| a.solved() && b.solved())
            .map(|(_, a, b)| b.median() / a.median().max(1e-6))
            .collect();
        println!(
            "| {group} | {} | {} | {} | {} | {} | {} | {} | {} | {new} |",
            pairs.len(),
            decided(pairs.iter().map(|(_, a, _)| *a)),
            decided(pairs.iter().map(|(_, _, b)| *b)),
            only(true),
            only(false),
            endings(pairs.iter().map(|(_, a, _)| *a)),
            endings(pairs.iter().map(|(_, _, b)| *b)),
            median_ratio(ratios),
        );
    }
    println!();

    // Generated and listed problems, side by side, whichever side has them.
    let mut keys: Vec<&(String, String)> = first.keys().chain(second.keys()).collect();
    keys.sort();
    keys.dedup();
    keys.retain(|key| {
        let row = first
            .get(*key)
            .or_else(|| second.get(*key))
            .map(|r| r.first);
        row.is_some_and(|r| r.get("source") != "lltp")
    });
    if keys.is_empty() {
        return;
    }
    println!("### Generated and listed problems\n");
    println!(
        "| file | family | configuration | problem | {one} | {two} | {two}/{one} |\n|---|---|---|---|--:|--:|--:|"
    );
    for key in keys {
        let (a, b) = (first.get(key), second.get(key));
        let ratio = match (a, b) {
            (Some(a), Some(b)) if a.decided() && b.decided() => {
                ratio(b.median() / a.median().max(1e-6))
            }
            _ => String::new(),
        };
        println!(
            "| {} | {} | {} | {} | {ratio} |",
            key.0,
            key.1,
            a.map_or(String::new(), Runs::cell),
            b.map_or(String::new(), Runs::cell)
        );
    }
    println!();
}
