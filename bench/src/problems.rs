// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The four sources of problems. The parent lists [`Reference`]s without
//! building a sequent from a file, so that an input that crashes the
//! parser crashes a child, not the run; the child [`load`]s the one it is
//! given.
//!
//! A problem file (`bench/problems/*.txt`) has one problem per line, `name;
//! mode; expected; copies; sequent`, blank lines and lines from `#` on
//! ignored: the mode is `classical`, `mix`, `affine`, `intuitionistic` or
//! `intuitionistic-affine`, `expected` is `provable`, `unprovable` or
//! `-`, `copies` a number or `-`, and the sequent is written as the
//! `linlog` command reads it. The part of the name before a `/` is the
//! family, else the file's stem.

use anyhow::{Context, Result, anyhow, bail};
use linlog::families::{FAMILIES, find};
use linlog::lltp::{self, Status};
use linlog::mist::{self, Safety};
use linlog::{Limits, Mode, Sequent};
use std::path::{Path, PathBuf};

/// A problem as the parent names it.
#[derive(Clone, Debug)]
pub struct Reference {
    /// Where it comes from: `family`, `lltp`, `spec` or `file`.
    pub source: &'static str,
    /// Its family: a generated family, an LLTP or `.spec` directory
    /// (relative to the parent of the path given), or a problem file's
    /// group.
    pub family: String,
    /// The size of a generated problem.
    pub size: Option<u32>,
    /// The index of a generated problem among those of its size.
    pub index: Option<u32>,
    /// Its name within the family.
    pub name: String,
    /// The mode it is meant for.
    pub mode: Mode,
    /// How the child is told which problem to load: `family:NAME:SIZE:INDEX`,
    /// `lltp:PATH`, `spec:PATH` or `file:PATH:LINE`.
    pub id: String,
}

/// A problem the child runs.
pub struct Problem {
    /// The sequent.
    pub sequent: Sequent,
    /// The mode it is meant for.
    pub mode: Mode,
    /// Whether it is provable, when the source says.
    pub expected: Option<bool>,
    /// The copy bound the source names.
    pub copies: Option<u32>,
}

/// Lists the instances of the families named (`NAME` or `NAME=SIZES`), or
/// of every family.
pub fn families(specs: &[String], all: bool) -> Result<Vec<Reference>> {
    let mut chosen = Vec::new();
    if all {
        chosen.extend(FAMILIES.iter().map(|f| (f, f.sizes.to_vec())));
    }
    for spec in specs {
        let (name, sizes) = spec.split_once('=').unwrap_or((spec, ""));
        let family = find(name).ok_or_else(|| anyhow!("no family `{name}`"))?;
        let sizes = if sizes.is_empty() {
            family.sizes.to_vec()
        } else {
            sizes
                .split(',')
                .map(|s| s.trim().parse().with_context(|| format!("size `{s}`")))
                .collect::<Result<_>>()?
        };
        chosen.push((family, sizes));
    }
    let mut references = Vec::new();
    for (family, sizes) in chosen {
        for size in sizes {
            for index in 0..family.instances {
                let instance = family.instance(size, index);
                references.push(Reference {
                    source: "family",
                    family: family.name.to_owned(),
                    size: Some(size),
                    index: Some(index),
                    name: instance.name,
                    mode: instance.mode,
                    id: format!("family:{}:{size}:{index}", family.name),
                });
            }
        }
    }
    Ok(references)
}

/// Lists the LLTP problems under the paths, in path order.
pub fn lltp(paths: &[PathBuf]) -> Result<Vec<Reference>> {
    listed(paths, "lltp", "p", lltp_mode)
}

/// Lists the coverability problems under the paths, in path order, each
/// for intuitionistic affine mode: the question whether a marking that
/// covers the target is reachable.
pub fn specs(paths: &[PathBuf]) -> Result<Vec<Reference>> {
    listed(paths, "spec", "spec", |_| {
        Mode::INTUITIONISTIC.with_affine()
    })
}

/// Lists the files of a source under the paths, in path order: each path
/// a file, or a directory searched for files with the extension; the
/// family of a file is its directory relative to the parent of the path
/// given.
fn listed(
    paths: &[PathBuf],
    source: &'static str,
    extension: &str,
    mode: fn(&Path) -> Mode,
) -> Result<Vec<Reference>> {
    let mut references = Vec::new();
    for root in paths {
        let mut files = Vec::new();
        collect(root, extension, &mut files)
            .with_context(|| format!("reading {}", root.display()))?;
        files.sort();
        let base = root.parent().unwrap_or(Path::new(""));
        for file in files {
            let directory = file.parent().unwrap_or(Path::new(""));
            let family = directory.strip_prefix(base).unwrap_or(directory);
            references.push(Reference {
                source,
                family: family.display().to_string(),
                size: None,
                index: None,
                name: file
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
                mode: mode(&file),
                id: format!("{source}:{}", file.display()),
            });
        }
    }
    Ok(references)
}

/// Adds `path` if it is a file, and the files with the extension below it
/// if it is a directory.
fn collect(path: &Path, extension: &str, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if !path.is_dir() {
        files.push(path.to_owned());
        return Ok(());
    }
    for entry in std::fs::read_dir(path)? {
        let path = entry?.path();
        if path.is_dir() {
            collect(&path, extension, files)?;
        } else if path.extension().is_some_and(|e| e == extension) {
            files.push(path);
        }
    }
    Ok(())
}

/// The mode of an LLTP problem: intuitionistic under the library's `ILL`
/// directory, classical otherwise.
fn lltp_mode(path: &Path) -> Mode {
    if path.components().any(|c| c.as_os_str() == "ILL") {
        Mode::INTUITIONISTIC
    } else {
        Mode::CLASSICAL
    }
}

/// Lists the problems of problem files.
pub fn files(paths: &[PathBuf]) -> Result<Vec<Reference>> {
    let mut references = Vec::new();
    for path in paths {
        let text =
            std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
        let stem = path.file_stem().unwrap_or_default().to_string_lossy();
        for (number, line) in text.lines().enumerate() {
            let Some([name, mode, ..]) = fields(line) else {
                continue;
            };
            let place = format!("{}:{}", path.display(), number + 1);
            let (family, _) = name.split_once('/').unwrap_or((&stem, name));
            references.push(Reference {
                source: "file",
                family: family.to_owned(),
                size: None,
                index: None,
                name: name.to_owned(),
                mode: parse_mode(mode).with_context(|| place.clone())?,
                id: format!("file:{place}"),
            });
        }
    }
    Ok(references)
}

/// The five fields of a problem line, or `None` for a comment or a blank
/// line.
fn fields(line: &str) -> Option<[&str; 5]> {
    let line = line.split_once('#').map_or(line, |(code, _)| code).trim();
    if line.is_empty() {
        return None;
    }
    let mut fields = line.splitn(5, ';').map(str::trim);
    Some(std::array::from_fn(|_| fields.next().unwrap_or("")))
}

/// Reads a mode's name as problem files and the CSV write it.
fn parse_mode(name: &str) -> Result<Mode> {
    Ok(name.parse()?)
}

/// Names a mode as problem files and the CSV write it.
pub fn mode_name(mode: Mode) -> &'static str {
    mode.name()
}

/// Loads the problem a [`Reference::id`] names.
pub fn load(id: &str) -> Result<Problem> {
    let (source, rest) = id.split_once(':').unwrap_or((id, ""));
    match source {
        "family" => {
            let mut parts = rest.split(':');
            let (Some(name), Some(size), Some(index)) = (parts.next(), parts.next(), parts.next())
            else {
                bail!("not a family problem: `{id}`");
            };
            let family = find(name).ok_or_else(|| anyhow!("no family `{name}`"))?;
            let instance = family.instance(size.parse()?, index.parse()?);
            Ok(Problem {
                sequent: instance.sequent,
                mode: instance.mode,
                expected: Some(instance.provable),
                copies: instance.copies,
            })
        }
        "lltp" => {
            let text = std::fs::read_to_string(rest).with_context(|| format!("reading {rest}"))?;
            let problem = lltp::read(&text, &Limits::default())?;
            Ok(Problem {
                sequent: problem.sequent,
                mode: lltp_mode(Path::new(rest)),
                expected: problem.status.map(|s| s == Status::Theorem),
                copies: None,
            })
        }
        "spec" => {
            let text = std::fs::read_to_string(rest).with_context(|| format!("reading {rest}"))?;
            let problem = mist::read(&text, &Limits::default())?;
            Ok(Problem {
                sequent: problem.sequent,
                mode: Mode::INTUITIONISTIC.with_affine(),
                expected: problem.expected.map(|s| s == Safety::Unsafe),
                copies: None,
            })
        }
        "file" => {
            let (path, line) = rest
                .rsplit_once(':')
                .ok_or_else(|| anyhow!("not a file problem: `{id}`"))?;
            let text = std::fs::read_to_string(path).with_context(|| format!("reading {path}"))?;
            let line: usize = line.parse()?;
            let [_, mode, expected, copies, sequent] = text
                .lines()
                .nth(line - 1)
                .and_then(fields)
                .ok_or_else(|| anyhow!("no problem on line {line} of {path}"))?;
            Ok(Problem {
                sequent: sequent.parse()?,
                mode: parse_mode(mode)?,
                expected: match expected {
                    "provable" => Some(true),
                    "unprovable" => Some(false),
                    _ => None,
                },
                copies: copies.parse().ok(),
            })
        }
        _ => bail!("unknown problem source in `{id}`"),
    }
}
