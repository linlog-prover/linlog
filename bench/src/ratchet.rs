// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The ratchet: every journey's instruction count under callgrind against
//! its committed ceiling in `bench/ceilings.csv`. A count above its
//! ceiling by more than [`TOLERANCE`] fails the check; `--lower` writes
//! the counts that went down as the new ceilings and never raises one: a
//! raise is an edit of the file in a commit that says why.
//!
//! Beside each count, the misses of a fixed model of the caches (the first
//! level's and the last's), against those recorded in `bench/misses.csv`:
//! they show what a count cannot see, a layout that touches more lines or a
//! working set that outgrows a cache, and are deterministic as the counts
//! are. They never fail a run: a move past [`MISS_NOTICE`] is named, as a
//! question for the journey's time pinned to one core.

use crate::journeys::JOURNEYS;
use anyhow::{Context, Result, bail};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use std::thread;

/// How far a count may pass its ceiling, as a fraction. The counts of one
/// build are exact, but a build in the shell and one in the flake's
/// sandbox, of the same sources in another directory, count up to 0.9 %
/// apart (`render-svg`; under 0.1 % elsewhere). The ceilings are the
/// larger of the two.
pub const TOLERANCE: f64 = 0.02;

/// The ceilings' file, relative to the workspace.
pub const CEILINGS: &str = "bench/ceilings.csv";

/// The recorded misses' file, relative to the workspace.
pub const MISSES: &str = "bench/misses.csv";

/// How far a journey's misses may move from the record, as a fraction,
/// before the run names the journey, if they moved by [`MISS_FLOOR`] too.
pub const MISS_NOTICE: f64 = 0.05;

/// How many misses a journey's may move by without notice, whatever the
/// fraction: a build in the shell and one in the flake's sandbox lay the
/// code out apart, which moved the first level's misses of `read-json` by
/// 8.6 % (3 758 of 43 599) and no journey's by more than 59 000 (0.9 % of
/// `search-additive-14`'s 6.5 million).
pub const MISS_FLOOR: u64 = 10_000;

/// The caches callgrind simulates: those of a performance core of the
/// machine the ratchet was made on (L1 instructions 64 KiB, L1 data 48
/// KiB, the last level 18 MiB, which callgrind takes as 18 ways of 16 384
/// sets), named so that a run counts the same misses on any host, CI's
/// included; left out, callgrind reads them off the host's processor.
const CACHES: [&str; 3] = [
    "--I1=65536,16,64",
    "--D1=49152,12,64",
    "--LL=18874368,18,64",
];

/// What callgrind counts of one journey.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Counts {
    /// The instructions executed.
    instructions: u64,
    /// The misses of the first level's caches, instructions and data.
    l1_misses: u64,
    /// The misses of the last level's cache, instructions and data.
    ll_misses: u64,
}

/// What the ratchet does with the counts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// Print them beside the ceilings.
    Show,
    /// Fail when one passes its ceiling, or a journey has none.
    Check,
    /// Write every count below its ceiling, and every journey's that has
    /// none, as the new ceiling.
    Lower,
}

/// Counts the journeys named (all when `only` is empty) under callgrind,
/// `jobs` at a time, keeping callgrind's files in `keep` if given, and
/// compares them with the ceilings in `ceilings` and their misses with
/// those in `misses`, which `record_misses` writes afresh.
pub fn ratchet(
    action: Action,
    ceilings: &Path,
    misses: &Path,
    record_misses: bool,
    only: &[String],
    jobs: usize,
    keep: Option<&Path>,
) -> Result<()> {
    let names: Vec<&str> = JOURNEYS
        .iter()
        .map(|j| j.name)
        .filter(|name| only.is_empty() || only.iter().any(|o| o == name))
        .collect();
    if let Some(unknown) = only.iter().find(|o| !names.contains(&o.as_str())) {
        bail!("no journey {unknown:?}; `linlog-bench journeys` lists them");
    }
    let scratch = match keep {
        Some(dir) => dir.to_owned(),
        None => std::env::temp_dir().join(format!("linlog-ratchet-{}", std::process::id())),
    };
    std::fs::create_dir_all(&scratch)?;
    let counts = count_all(&names, jobs, &scratch);
    if keep.is_none() {
        std::fs::remove_dir_all(&scratch).ok();
    }
    let counts = counts?;
    let mut table = read(ceilings)?;
    let mut recorded = read_misses(misses)?;
    let mut failed = Vec::new();
    let mut moved = Vec::new();
    println!(
        "| journey | instructions | ceiling | change | L1 misses | change | LL misses | change |\n\
         |---|--:|--:|--:|--:|--:|--:|--:|"
    );
    for name in &names {
        let Counts {
            instructions: count,
            l1_misses,
            ll_misses,
        } = counts[name];
        let ceiling = table.get(*name).copied();
        let shown = ceiling.map_or_else(|| "–".to_owned(), |c| c.to_string());
        let record = recorded.get(*name).copied();
        println!(
            "| {name} | {count} | {shown} | {} | {l1_misses} | {} | {ll_misses} | {} |",
            change(count, ceiling),
            change(l1_misses, record.map(|r| r.0)),
            change(ll_misses, record.map(|r| r.1)),
        );
        if record.is_some_and(|(l1, ll)| moves(l1_misses, l1) || moves(ll_misses, ll)) {
            moved.push(*name);
        }
        if record_misses {
            recorded.insert((*name).to_owned(), (l1_misses, ll_misses));
        }
        if fails(count, ceiling, action) {
            failed.push(*name);
        }
        if lowers(count, ceiling, action) {
            table.insert((*name).to_owned(), count);
        }
    }
    if !moved.is_empty() {
        println!(
            "\nmisses moved by more than {} % and {MISS_FLOOR} on {}: a question for their \
             time pinned to one core (`journey NAME --repeat N`, before and after), not a failure",
            MISS_NOTICE * 100.0,
            moved.join(", ")
        );
    }
    if record_misses {
        write_misses(misses, &recorded)?;
    }
    if only.is_empty() {
        for name in table.keys() {
            if !JOURNEYS.iter().any(|j| j.name == name) {
                bail!("{name} in {} is no journey", ceilings.display());
            }
        }
    }
    match action {
        Action::Lower => write(ceilings, &table)?,
        Action::Check if !failed.is_empty() => bail!(
            "over the ceiling by more than {} %, or without one: {}",
            TOLERANCE * 100.0,
            failed.join(", ")
        ),
        _ => {}
    }
    Ok(())
}

/// Counts every journey, `jobs` at a time.
fn count_all<'a>(
    names: &[&'a str],
    jobs: usize,
    scratch: &Path,
) -> Result<BTreeMap<&'a str, Counts>> {
    let exe = std::env::current_exe()?;
    let next = Mutex::new(names.iter());
    let counts = Mutex::new(BTreeMap::new());
    thread::scope(|scope| {
        let workers: Vec<_> = (0..jobs.max(1))
            .map(|_| {
                scope.spawn(|| -> Result<()> {
                    while let Some(&name) = next.lock().unwrap().next() {
                        let count = count(&exe, name, &scratch.join(format!("{name}.callgrind")))?;
                        counts.lock().unwrap().insert(name, count);
                    }
                    Ok(())
                })
            })
            .collect();
        workers.into_iter().try_for_each(|w| w.join().unwrap())
    })?;
    Ok(counts.into_inner().unwrap())
}

/// Runs one journey under callgrind with its model of the caches,
/// collecting inside [`measured`](crate::journeys::measured) only, and
/// returns what it counted.
fn count(exe: &Path, name: &str, out: &Path) -> Result<Counts> {
    let valgrind = std::env::var_os("VALGRIND").unwrap_or_else(|| "valgrind".into());
    // valgrind warns on every run that it read the host's caches, even
    // where CACHES replaces them, so its errors are shown on a failure only.
    let output = Command::new(&valgrind)
        .args(["--tool=callgrind", "--collect-atstart=no", "--quiet"])
        .arg("--cache-sim=yes")
        .args(CACHES)
        .arg("--toggle-collect=*journeys::measured*")
        .arg(format!("--callgrind-out-file={}", out.display()))
        .arg(exe)
        .args(["journey", name])
        .output()
        .with_context(|| {
            format!(
                "running {} (VALGRIND names another)",
                PathBuf::from(&valgrind).display()
            )
        })?;
    if !output.status.success() {
        bail!(
            "the journey {name} failed under callgrind: {}\n{}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let text = std::fs::read_to_string(out)?;
    totals(&text).with_context(|| format!("no totals of the events in {}", out.display()))
}

/// Reads the totals off a callgrind file: the line naming its events and
/// the line of their sums, the instructions `Ir`, the first level's misses
/// `I1mr`, `D1mr` and `D1mw`, the last level's `ILmr`, `DLmr` and `DLmw`;
/// an event named without a sum counted zero.
fn totals(text: &str) -> Option<Counts> {
    let events: Vec<&str> = text
        .lines()
        .find_map(|line| line.strip_prefix("events: "))?
        .split_whitespace()
        .collect();
    let sums: Vec<u64> = text
        .lines()
        .find_map(|line| {
            line.strip_prefix("totals: ")
                .or_else(|| line.strip_prefix("summary: "))
        })?
        .split_whitespace()
        .map(str::parse)
        .collect::<Result<_, _>>()
        .ok()?;
    // callgrind leaves trailing zeros off the line of sums.
    let sum = |names: &[&str]| -> Option<u64> {
        names
            .iter()
            .map(|name| {
                let at = events.iter().position(|e| e == name)?;
                Some(sums.get(at).copied().unwrap_or(0))
            })
            .sum()
    };
    Some(Counts {
        instructions: sum(&["Ir"])?,
        l1_misses: sum(&["I1mr", "D1mr", "D1mw"])?,
        ll_misses: sum(&["ILmr", "DLmr", "DLmw"])?,
    })
}

/// A value's change from a reference, in percent, or `new` without one.
fn change(value: u64, reference: Option<u64>) -> String {
    reference.map_or_else(
        || "new".to_owned(),
        |r| format!("{:+.2} %", (value as f64 / r.max(1) as f64 - 1.0) * 100.0),
    )
}

/// Whether misses moved from their record, either way, by more than
/// [`MISS_NOTICE`] of it and by more than [`MISS_FLOOR`].
fn moves(misses: u64, recorded: u64) -> bool {
    let moved = misses.abs_diff(recorded);
    moved > MISS_FLOOR && moved as f64 > recorded as f64 * MISS_NOTICE
}

/// Whether a journey's count fails the run: above its ceiling by more
/// than [`TOLERANCE`], or, for a check, without a ceiling.
fn fails(count: u64, ceiling: Option<u64>, action: Action) -> bool {
    match ceiling {
        Some(c) => count as f64 > c as f64 * (1.0 + TOLERANCE),
        None => action == Action::Check,
    }
}

/// Whether `--lower` writes the count as the journey's new ceiling: below
/// the ceiling, or where there is none; never above one.
fn lowers(count: u64, ceiling: Option<u64>, action: Action) -> bool {
    action == Action::Lower && ceiling.is_none_or(|c| count < c)
}

/// Reads the ceilings: a header, then `journey,instructions` per line.
fn read(path: &Path) -> Result<BTreeMap<String, u64>> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(BTreeMap::new()),
        Err(error) => return Err(error).context(path.display().to_string()),
    };
    text.lines()
        .skip(1)
        .filter(|line| !line.is_empty())
        .map(|line| {
            let (name, count) = line.split_once(',').with_context(|| format!("{line:?}"))?;
            Ok((
                name.to_owned(),
                count.parse().with_context(|| format!("{line:?}"))?,
            ))
        })
        .collect()
}

/// Reads the recorded misses: a header, then
/// `journey,l1_misses,ll_misses` per line; none when the file is absent.
fn read_misses(path: &Path) -> Result<BTreeMap<String, (u64, u64)>> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(BTreeMap::new()),
        Err(error) => return Err(error).context(path.display().to_string()),
    };
    text.lines()
        .skip(1)
        .filter(|line| !line.is_empty())
        .map(|line| {
            let mut fields = line.split(',');
            let (Some(name), Some(l1), Some(ll), None) =
                (fields.next(), fields.next(), fields.next(), fields.next())
            else {
                bail!("{line:?} is not journey,l1_misses,ll_misses");
            };
            Ok((
                name.to_owned(),
                (
                    l1.parse().with_context(|| format!("{line:?}"))?,
                    ll.parse().with_context(|| format!("{line:?}"))?,
                ),
            ))
        })
        .collect()
}

/// Writes the recorded misses in the journeys' order.
fn write_misses(path: &Path, table: &BTreeMap<String, (u64, u64)>) -> Result<()> {
    let mut text = String::from("journey,l1_misses,ll_misses\n");
    for journey in JOURNEYS {
        if let Some((l1, ll)) = table.get(journey.name) {
            text.push_str(&format!("{},{l1},{ll}\n", journey.name));
        }
    }
    std::fs::write(path, text).context(path.display().to_string())
}

/// Writes the ceilings in the journeys' order.
fn write(path: &Path, table: &BTreeMap<String, u64>) -> Result<()> {
    let mut text = String::from("journey,instructions\n");
    for journey in JOURNEYS {
        if let Some(count) = table.get(journey.name) {
            text.push_str(&format!("{},{count}\n", journey.name));
        }
    }
    std::fs::write(path, text).context(path.display().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A count fails within its tolerance never, past it always, and
    /// without a ceiling only under a check; `--lower` writes a count
    /// below its ceiling or without one, and never raises one.
    #[test]
    fn the_ratchet_s_decision() {
        assert!(!fails(1_020, Some(1_000), Action::Check));
        assert!(fails(1_021, Some(1_000), Action::Check));
        assert!(fails(5, None, Action::Check));
        assert!(!fails(5, None, Action::Lower));
        assert!(lowers(999, Some(1_000), Action::Lower));
        assert!(lowers(999, None, Action::Lower));
        assert!(!lowers(1_001, Some(1_000), Action::Lower));
        assert!(!lowers(999, Some(1_000), Action::Check));
    }

    /// The totals are read by the events' names, whatever their order, a
    /// missing sum as zero, and misses are named past both the notice and
    /// the floor, either way, never within one.
    #[test]
    fn the_totals_and_the_misses_notice() {
        let text = "events: Ir Dr Dw I1mr D1mr D1mw ILmr DLmr DLmw\nfl=x\n\
                    summary: 100 7 8 1 2 3 4 5 6\n";
        let expected = Counts {
            instructions: 100,
            l1_misses: 6,
            ll_misses: 15,
        };
        assert_eq!(totals(text), Some(expected));
        let reordered = "events: DLmw Ir I1mr D1mr D1mw ILmr DLmr\ntotals: 6 100 1 2 3 4 5\n";
        assert_eq!(totals(reordered), Some(expected));
        assert_eq!(totals("events: Ir\nsummary: 100\n"), None);
        let trailing = "events: Ir I1mr D1mr D1mw ILmr DLmr DLmw\nsummary: 100 1 2 3 4\n";
        let short = Counts {
            instructions: 100,
            l1_misses: 6,
            ll_misses: 4,
        };
        assert_eq!(totals(trailing), Some(short));
        assert!(!moves(105_000, 100_000));
        assert!(!moves(106_000, 100_000));
        assert!(moves(110_001, 100_000));
        assert!(moves(1_000_000, 1_060_001));
        assert!(!moves(43_599, 39_841));
        assert!(!moves(20_000, 10_001));
        assert!(moves(20_002, 10_001));
    }
}
