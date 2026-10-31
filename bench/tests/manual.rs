// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The manual's chapters on the harness name only what exists: in
//! `docs/src/benchmarks.md` and every chapter under `docs/src/results/`,
//! each `linlog-bench` subcommand a command line or an inline code span
//! names is one the harness has, each flag after it is in that
//! subcommand's help, each flag of a script `bench/*.sh` is one the script
//! reads, and each other flag, alone or after a subcommand's name, is in
//! the harness's help or a script's. The calls of `linlog` are the
//! command's tests' (`cli/tests/readme.rs`).

use std::path::{Path, PathBuf};
use std::process::Command;

/// The repository's root.
const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/..");

/// Returns what `linlog-bench` prints for the arguments.
fn bench(arguments: &[&str]) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_linlog-bench"))
        .args(arguments)
        .output()
        .unwrap();
    String::from_utf8(out.stdout).unwrap() + &String::from_utf8(out.stderr).unwrap()
}

/// Returns the subcommands of the harness, as its help lists them.
fn subcommands() -> Vec<String> {
    bench(&["--help"])
        .lines()
        .skip_while(|line| !line.starts_with("Commands:"))
        .skip(1)
        .take_while(|line| line.starts_with("  "))
        .filter_map(|line| line.split_whitespace().next())
        .filter(|name| *name != "help")
        .map(str::to_owned)
        .collect()
}

/// Whether `text` holds `word` with no letter, digit, `-` or `_` right
/// before or after it.
fn holds_word(text: &str, word: &str) -> bool {
    let part = |c: char| c.is_ascii_alphanumeric() || c == '-' || c == '_';
    text.match_indices(word).any(|(at, _)| {
        !text[..at].chars().next_back().is_some_and(part)
            && !text[at + word.len()..].chars().next().is_some_and(part)
    })
}

/// Returns the flags a piece of text names: every word that is `--` and
/// a lowercase name, with a value after `=` cut off.
fn flags_in(text: &str) -> Vec<String> {
    text.split(|c: char| c.is_whitespace() || "`'\"(),;".contains(c))
        .filter_map(|word| {
            let word = word.split('=').next().unwrap_or("");
            let name = word.strip_prefix("--")?;
            (name.starts_with(|c: char| c.is_ascii_lowercase())
                && name.chars().all(|c| c.is_ascii_lowercase() || c == '-'))
            .then(|| word.to_owned())
        })
        .collect()
}

/// Returns the text without what its quotes hold.
fn unquoted(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut quote = None;
    for c in text.chars() {
        match quote {
            Some(open) if c == open => quote = None,
            Some(_) => {}
            None if c == '\'' || c == '"' => quote = Some(c),
            None => out.push(c),
        }
    }
    out
}

/// Returns the code of a chapter: the command lines of its `console`
/// blocks and its inline code spans (a span across a line break joined by
/// a blank), each with the line it starts on, and its prose outside them.
fn code_of(text: &str) -> (Vec<(usize, String)>, String) {
    let mut code = Vec::new();
    let mut prose = String::new();
    let mut fence: Option<&str> = None;
    let mut span: Option<(usize, String)> = None;
    for (at, line) in text.lines().enumerate() {
        if let Some(rest) = line.strip_prefix("```") {
            fence = if fence.is_some() { None } else { Some(rest) };
            continue;
        }
        match fence {
            Some("console") => {
                if let Some(command) = line.strip_prefix("$ ") {
                    code.push((at + 1, command.to_owned()));
                }
            }
            Some(_) => {}
            None => {
                for (index, piece) in line.replace("\\|", "|").split('`').enumerate() {
                    if index > 0 {
                        span = match span.take() {
                            Some(done) => {
                                code.push(done);
                                None
                            }
                            None => Some((at + 1, String::new())),
                        };
                    }
                    match &mut span {
                        Some((_, text)) => text.push_str(piece),
                        None => prose.push_str(piece),
                    }
                }
                match &mut span {
                    Some((_, text)) => text.push(' '),
                    None => prose.push('\n'),
                }
            }
        }
    }
    (code, prose)
}

/// Returns the harness's chapters, each with its path under the root.
fn chapters() -> Vec<(String, String)> {
    let root = Path::new(ROOT);
    let mut paths = vec![PathBuf::from("docs/src/benchmarks.md")];
    let mut results: Vec<PathBuf> = std::fs::read_dir(root.join("docs/src/results"))
        .unwrap()
        .map(|entry| entry.unwrap().path().strip_prefix(root).unwrap().to_owned())
        .filter(|path| path.extension().is_some_and(|extension| extension == "md"))
        .collect();
    results.sort();
    paths.extend(results);
    paths
        .into_iter()
        .map(|path| {
            let text = std::fs::read_to_string(root.join(&path)).unwrap();
            (path.to_str().unwrap().to_owned(), text)
        })
        .collect()
}

/// Every subcommand and flag of the harness and of its scripts that the
/// harness's chapters of the manual name exists.
#[test]
fn the_manual_names_what_exists() {
    let names = subcommands();
    assert!(names.iter().any(|name| name == "run"), "{names:?}");
    let helps: Vec<(String, String)> = names
        .iter()
        .map(|name| (name.clone(), bench(&[name, "--help"])))
        .collect();
    let mut scripts = Vec::new();
    for entry in std::fs::read_dir(Path::new(ROOT).join("bench")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|extension| extension == "sh") {
            let name = format!("bench/{}", path.file_name().unwrap().to_str().unwrap());
            scripts.push((name, std::fs::read_to_string(path).unwrap()));
        }
    }
    // What any flag of the harness or a script may be found in.
    let everything = helps
        .iter()
        .map(|(_, help)| help.as_str())
        .collect::<String>()
        + &bench(&["--help"])
        + &scripts
            .iter()
            .map(|(_, text)| text.as_str())
            .collect::<String>();
    let mut failures = Vec::new();
    let mut checked = 0;
    for (chapter, text) in chapters() {
        let (code, prose) = code_of(&text);
        for flag in flags_in(&prose) {
            checked += 1;
            if !holds_word(&everything, &flag) {
                failures.push(format!(
                    "{chapter}: the flag {flag} is the harness's nowhere"
                ));
            }
        }
        for (line, piece) in &code {
            for part in unquoted(piece).split(['|', ';', '&']) {
                let mut words = part.split_whitespace();
                let Some(first) = words.next() else { continue };
                let rest: Vec<&str> = words.collect();
                // The text a flag of this call must be found in.
                let (known, call) = if first == "linlog-bench" {
                    let Some(name) = rest.first().filter(|word| !word.starts_with('-')) else {
                        continue;
                    };
                    match helps.iter().find(|(known, _)| known == name) {
                        Some((_, help)) => (help.as_str(), format!("linlog-bench {name}")),
                        None => {
                            failures.push(format!(
                                "{chapter}:{line}: no subcommand linlog-bench {name}"
                            ));
                            continue;
                        }
                    }
                } else if let Some((name, script)) = scripts.iter().find(|(name, _)| name == first)
                {
                    (script.as_str(), name.clone())
                } else if let Some((name, help)) = helps.iter().find(|(name, _)| name == first) {
                    (help.as_str(), format!("linlog-bench {name}"))
                } else if first.starts_with("--") {
                    (everything.as_str(), "the harness".to_owned())
                } else {
                    // Another program's call, `linlog`'s among them.
                    continue;
                };
                for flag in flags_in(&format!("{first} {}", rest.join(" "))) {
                    checked += 1;
                    if !holds_word(known, &flag) {
                        failures.push(format!("{chapter}:{line}: {call} has no flag {flag}"));
                    }
                }
            }
        }
    }
    // The check finds what it looks for.
    assert!(holds_word(&everything, "--family") && !holds_word(&everything, "--familly"));
    assert!(checked > 20, "only {checked} flags were checked");
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
