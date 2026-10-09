// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! README's examples against the binary: every command of every `console`
//! block of `README.md` is run in a directory of its block's own, through
//! `sh` so that pipes and redirections work, and its output (standard
//! output and standard error together) is compared with the lines that
//! follow it in the block. Lines of the block that start with `> ` are
//! the command's standard input, as in a transcript of `linlog interact`.
//!
//! A comment on the line before a block says how much of its output is
//! the machine's (`<!-- readme-check: WORD -->`):
//!
//! - none: the output is compared whole, every time a search took
//!   (after "after " and "time: ") read as `…`;
//! - `machine`: a time limit, a thread count or a copy bound reached
//!   under a time limit shows in it, so every number is read as `#`;
//! - `terminal`: it is what a terminal of a given size shows, so only the
//!   first line, the verdict, is compared;
//! - `skip`, then why: the block is not run.
//!
//! A file a command writes (`--output FILE`, every file under `--output
//! DIR` of a batch, or any file of a known kind in the block's directory
//! afterwards) must exist and be of the kind its extension names; its bytes are not compared, since a PDF carries the
//! date it was made.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// The README, read when the test is built.
const README: &str = include_str!("../../README.md");

/// The programs other than `linlog` that the README's examples call and
/// this test does not run: the harness has the flake's `bench` check, and
/// the LaTeX and Rocq output its `export` and `rocq` checks.
const OTHER_PROGRAMS: [&str; 3] = ["linlog-bench", "pdflatex", "rocq"];

/// The shell's own commands that the examples use to make or show a file.
const SHELL_COMMANDS: [&str; 2] = ["cat", "echo"];

/// How much of a block's output is compared.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Check {
    /// All of it, with the times read as `…`.
    Exact,
    /// Its shape: every number read as `#`.
    Machine,
    /// The first line.
    Terminal,
    /// Nothing: the block is not run.
    Skip,
}

/// One command of a block with the transcript that follows it.
#[derive(Debug)]
struct Example {
    /// The README's line of the command, counting from 1.
    line: usize,
    /// The command line, without the `$ `.
    command: String,
    /// The lines that start with `> `, without it, each with a newline.
    input: String,
    /// The other lines up to the next command, trailing empty ones left out.
    output: Vec<String>,
}

/// A `console` block of the README.
#[derive(Debug)]
struct Block {
    /// The README's line of the opening fence, counting from 1.
    line: usize,
    /// How much of the output is compared.
    check: Check,
    /// The commands in the order of the block.
    examples: Vec<Example>,
}

/// Returns the `console` blocks of the text with the check their comment
/// names.
fn blocks(text: &str) -> Vec<Block> {
    let lines: Vec<&str> = text.lines().collect();
    let mut blocks = Vec::new();
    let mut at = 0;
    while at < lines.len() {
        if lines[at] != "```console" {
            at += 1;
            continue;
        }
        let check = match at.checked_sub(1).map(|before| lines[before]) {
            Some(comment) if comment.starts_with("<!-- readme-check:") => {
                let word = comment
                    .trim_start_matches("<!-- readme-check:")
                    .trim_start()
                    .split(|c: char| !c.is_ascii_alphabetic())
                    .next();
                match word {
                    Some("machine") => Check::Machine,
                    Some("terminal") => Check::Terminal,
                    Some("skip") => Check::Skip,
                    _ => panic!("README.md:{at}: unknown check {comment}"),
                }
            }
            _ => Check::Exact,
        };
        let mut block = Block {
            line: at + 1,
            check,
            examples: Vec::new(),
        };
        at += 1;
        while at < lines.len() && lines[at] != "```" {
            let line = lines[at];
            if let Some(command) = line.strip_prefix("$ ") {
                block.examples.push(Example {
                    line: at + 1,
                    command: command.to_owned(),
                    input: String::new(),
                    output: Vec::new(),
                });
            } else {
                let example = block
                    .examples
                    .last_mut()
                    .unwrap_or_else(|| panic!("README.md:{}: output before a command", at + 1));
                match line.strip_prefix("> ") {
                    Some(input) => {
                        example.input.push_str(input);
                        example.input.push('\n');
                    }
                    None => example.output.push(line.to_owned()),
                }
            }
            at += 1;
        }
        for example in &mut block.examples {
            while example.output.last().is_some_and(|line| line.is_empty()) {
                example.output.pop();
            }
        }
        blocks.push(block);
        at += 1;
    }
    blocks
}

/// Returns the text with every time a search took, a number with its unit
/// after "after " or "time: ", and the crate's version an outcome names,
/// as `…`.
fn timeless(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = ["after ", "time: "]
        .iter()
        .filter_map(|prefix| rest.find(prefix).map(|found| found + prefix.len()))
        .min()
    {
        out.push_str(&rest[..at]);
        rest = &rest[at..];
        let number = rest
            .find(|c: char| !(c.is_ascii_digit() || c == '.'))
            .unwrap_or(rest.len());
        if number > 0 {
            let unit = rest[number..]
                .find(|c: char| !"nµms".contains(c))
                .map_or(rest.len(), |end| number + end);
            out.push('…');
            rest = &rest[unit..];
        }
    }
    out.push_str(rest);
    unversioned(&out)
}

/// Returns the text with the crate's version that an outcome names, in
/// `"linlog":"0.1.0"`, as `…`: a release moves it.
fn unversioned(text: &str) -> String {
    const KEY: &str = "\"linlog\":\"";
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find(KEY) {
        let (head, tail) = rest.split_at(at + KEY.len());
        out.push_str(head);
        out.push('…');
        rest = &tail[tail.find('"').unwrap_or(tail.len())..];
    }
    out.push_str(rest);
    out
}

/// Returns the text with every run of digits as `#`.
fn shape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        if !c.is_ascii_digit() {
            out.push(c);
        } else if !out.ends_with('#') {
            out.push('#');
        }
    }
    out
}

/// Runs the command line with `sh` in the directory, with the directory
/// of the `linlog` under test first on the path, and returns what it
/// wrote to standard output and standard error, in the order it wrote it.
fn run(directory: &Path, command: &str, input: &str) -> String {
    let binary = PathBuf::from(env!("CARGO_BIN_EXE_linlog"));
    let mut path = binary.parent().unwrap().as_os_str().to_owned();
    if let Some(inherited) = std::env::var_os("PATH") {
        path.push(":");
        path.push(inherited);
    }
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(format!("{{ {command}\n}} 2>&1"))
        .current_dir(directory)
        .env("PATH", path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    // A command that does not read its input may exit before the write: a
    // closed pipe is then no failure of the test.
    if let Err(error) =
        std::io::Write::write_all(&mut child.stdin.take().unwrap(), input.as_bytes())
    {
        assert_eq!(error.kind(), std::io::ErrorKind::BrokenPipe, "{error}");
    }
    let out = child.wait_with_output().unwrap();
    String::from_utf8(out.stdout).unwrap()
}

/// Returns why the file is not of the kind its extension names, or `None`
/// when it is or its extension names no kind this test knows.
fn not_of_its_kind(file: &Path) -> Option<String> {
    let bytes = std::fs::read(file).ok()?;
    let text = String::from_utf8_lossy(&bytes);
    let extension = file.extension()?.to_str()?;
    let fine = match extension {
        "pdf" => bytes.starts_with(b"%PDF-"),
        "png" => bytes.starts_with(b"\x89PNG\r\n\x1a\n"),
        "svg" => text.contains("<svg ") && text.trim_end().ends_with("</svg>"),
        "tex" => text.contains("\\end{prooftree}"),
        "v" => text.contains("\nQed."),
        "json" => serde_json::from_slice::<serde_json::Value>(&bytes).is_ok(),
        _ => return None,
    };
    (!fine).then(|| format!("{} is no {extension} file", file.display()))
}

/// Adds the files under a directory a batch wrote its derivations into.
fn walk(directory: &Path, files: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            walk(&path, files);
        } else {
            files.push(path);
        }
    }
}

/// Writes the files the examples read and the README does not show how
/// to make into the directory: `shared.json`, a sequent of 438 bytes that
/// doubles one atom 25 times.
fn fixtures(directory: &Path) {
    let mut terms = vec![r#"{"V":0}"#.to_owned()];
    terms.extend((0..25).map(|below| format!(r#"{{"⊗":[{below},{below}]}}"#)));
    let shared = format!(
        r#"{{"version":1,"terms":[{}],"roots":[25],"atoms":["A"]}}"#,
        terms.join(",")
    );
    assert_eq!(shared.len(), 438);
    std::fs::write(directory.join("shared.json"), shared).unwrap();
}

/// Every example of the README gives the output the README shows.
#[test]
fn readme_examples() {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join("readme");
    let mut failures = Vec::new();
    let mut ran = 0;
    for block in blocks(README) {
        if block.check == Check::Skip {
            continue;
        }
        let directory = root.join(block.line.to_string());
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).unwrap();
        fixtures(&directory);
        for example in &block.examples {
            let program = example.command.split_whitespace().next().unwrap_or("");
            if OTHER_PROGRAMS.contains(&program) {
                continue;
            }
            assert!(
                program == "linlog" || SHELL_COMMANDS.contains(&program),
                "README.md:{}: no rule for the program {program}",
                example.line
            );
            ran += 1;
            let actual = timeless(run(&directory, &example.command, &example.input).trim_end());
            let expected = timeless(&example.output.join("\n"));
            let (actual, expected) = match block.check {
                Check::Exact => (actual, expected),
                Check::Machine => (shape(&actual), shape(&expected)),
                Check::Terminal => (
                    actual.lines().next().unwrap_or("").to_owned(),
                    expected.lines().next().unwrap_or("").to_owned(),
                ),
                Check::Skip => unreachable!(),
            };
            if actual != expected {
                failures.push(format!(
                    "README.md:{}: $ {}\n--- README\n{expected}\n--- linlog\n{actual}",
                    example.line, example.command
                ));
            }
            let mut words = example.command.split_whitespace();
            while let Some(word) = words.next() {
                if word == "--output" {
                    let name = words.next().unwrap_or("").trim_matches(['"', '\'']);
                    let output = directory.join(name);
                    if output.is_dir() {
                        let mut files = Vec::new();
                        walk(&output, &mut files);
                        failures.extend(
                            files
                                .iter()
                                .filter_map(|file| not_of_its_kind(file))
                                .map(|why| format!("README.md:{}: {why}", example.line)),
                        );
                        if files.is_empty() {
                            failures.push(format!(
                                "README.md:{}: $ {}\nno file in {name}",
                                example.line, example.command
                            ));
                        }
                    } else if !output.is_file() {
                        failures.push(format!(
                            "README.md:{}: $ {}\nno file {name}",
                            example.line, example.command
                        ));
                    }
                }
            }
        }
        for entry in std::fs::read_dir(&directory).unwrap() {
            if let Some(why) = not_of_its_kind(&entry.unwrap().path()) {
                failures.push(format!("README.md:{}: {why}", block.line));
            }
        }
    }
    assert!(ran > 40, "only {ran} of the README's commands ran");
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}
