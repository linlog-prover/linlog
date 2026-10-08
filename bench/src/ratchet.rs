// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The ratchet: every journey's instruction count under callgrind against
//! its committed ceiling in `bench/ceilings.csv`. A count above its
//! ceiling by more than [`TOLERANCE`] fails the check; `--lower` writes
//! the counts that went down as the new ceilings and never raises one: a
//! raise is an edit of the file in a commit that says why.

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
/// compares them with the ceilings in `ceilings`.
pub fn ratchet(
    action: Action,
    ceilings: &Path,
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
    let mut failed = Vec::new();
    println!("| journey | instructions | ceiling | change |\n|---|--:|--:|--:|");
    for name in &names {
        let count = counts[name];
        let ceiling = table.get(*name).copied();
        let change = ceiling.map_or_else(
            || "new".to_owned(),
            |c| format!("{:+.2} %", (count as f64 / c as f64 - 1.0) * 100.0),
        );
        let shown = ceiling.map_or_else(|| "–".to_owned(), |c| c.to_string());
        println!("| {name} | {count} | {shown} | {change} |");
        match ceiling {
            Some(c) if count as f64 > c as f64 * (1.0 + TOLERANCE) => failed.push(*name),
            None if action == Action::Check => failed.push(*name),
            _ => {}
        }
        if action == Action::Lower && ceiling.is_none_or(|c| count < c) {
            table.insert((*name).to_owned(), count);
        }
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
fn count_all<'a>(names: &[&'a str], jobs: usize, scratch: &Path) -> Result<BTreeMap<&'a str, u64>> {
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

/// Runs one journey under callgrind, collecting inside
/// [`measured`](crate::journeys::measured) only, and returns the
/// instructions counted.
fn count(exe: &Path, name: &str, out: &Path) -> Result<u64> {
    let valgrind = std::env::var_os("VALGRIND").unwrap_or_else(|| "valgrind".into());
    let status = Command::new(&valgrind)
        .args(["--tool=callgrind", "--collect-atstart=no", "--quiet"])
        .arg("--toggle-collect=*journeys::measured*")
        .arg(format!("--callgrind-out-file={}", out.display()))
        .arg(exe)
        .args(["journey", name])
        .status()
        .with_context(|| {
            format!(
                "running {} (VALGRIND names another)",
                PathBuf::from(&valgrind).display()
            )
        })?;
    if !status.success() {
        bail!("the journey {name} failed under callgrind: {status}");
    }
    let text = std::fs::read_to_string(out)?;
    text.lines()
        .find_map(|line| {
            line.strip_prefix("totals: ")
                .or_else(|| line.strip_prefix("summary: "))
        })
        .and_then(|rest| rest.split_whitespace().next()?.parse().ok())
        .with_context(|| format!("no total in {}", out.display()))
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
