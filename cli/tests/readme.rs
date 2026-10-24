// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The examples of README and of the guide for AI models
//! (`skills/linlog/`) against the binary: every command of every `console`
//! block of `README.md`, `SKILL.md` and `reference.md` is run in a
//! directory of its block's own, through
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
//!
//! The guide is held to more: every command, flag, session command and
//! settings key it names exists, its Rust example is a doctest of the
//! library's, and its summary keeps within its budget.

use std::path::{Path, PathBuf};
use std::process::Command;

mod common;
use common::timeless;

/// The README, read when the test is built.
const README: &str = include_str!("../../README.md");

/// The guide's summary, an Agent Skill's `SKILL.md`.
const SUMMARY: &str = include_str!("../../skills/linlog/SKILL.md");

/// The guide's full reference beside it.
const REFERENCE: &str = include_str!("../../skills/linlog/reference.md");

/// The guide's documents by their names.
const GUIDE: [(&str, &str); 2] = [("SKILL.md", SUMMARY), ("reference.md", REFERENCE)];

/// The programs other than `linlog` that the README's examples call and
/// this test does not run: the harness has the flake's `bench` check, and
/// the LaTeX and Rocq output its `export` and `rocq` checks.
const OTHER_PROGRAMS: [&str; 3] = ["linlog-bench", "pdflatex", "rocq"];

/// The shell's own commands that the examples use to make or show a file.
const SHELL_COMMANDS: [&str; 3] = ["cat", "echo", "printf"];

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

/// Returns the `console` blocks of the text, the document `name`, with the
/// check their comment names.
fn blocks(name: &str, text: &str) -> Vec<Block> {
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
                    _ => panic!("{name}:{at}: unknown check {comment}"),
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
                    .unwrap_or_else(|| panic!("{name}:{}: output before a command", at + 1));
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
    let out = common::output(
        Command::new("sh")
            .arg("-c")
            .arg(format!("{{ {command}\n}} 2>&1"))
            .current_dir(directory)
            .env("PATH", path),
        input,
    );
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

/// Runs every example of the document `name`, in directories under
/// `root`, and returns how many commands ran and what failed.
fn examples(name: &str, text: &str, root: &Path) -> (usize, Vec<String>) {
    let mut failures = Vec::new();
    let mut ran = 0;
    for block in blocks(name, text) {
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
                "{name}:{}: no rule for the program {program}",
                example.line
            );
            ran += 1;
            let actual = timeless(run(&directory, &example.command, &example.input).trim_end());
            let expected = timeless(&example.output.join("\n"));
            let (actual, expected) = match block.check {
                Check::Exact => (actual, expected),
                // A machine of fewer threads than an example asks for
                // adds a note that it takes fewer, which only an example
                // of more threads than any machine shows.
                Check::Machine => {
                    let noted = expected.contains("note: --jobs ");
                    let actual: Vec<&str> = actual
                        .lines()
                        .filter(|l| noted || !l.starts_with("note: --jobs "))
                        .collect();
                    (shape(&actual.join("\n")), shape(&expected))
                }
                Check::Terminal => (
                    actual.lines().next().unwrap_or("").to_owned(),
                    expected.lines().next().unwrap_or("").to_owned(),
                ),
                Check::Skip => unreachable!(),
            };
            if actual != expected {
                failures.push(format!(
                    "{name}:{}: $ {}\n--- {name}\n{expected}\n--- linlog\n{actual}",
                    example.line, example.command
                ));
            }
            let mut words = example.command.split_whitespace();
            while let Some(word) = words.next() {
                if word == "--output" {
                    let output_name = words.next().unwrap_or("").trim_matches(['"', '\'']);
                    let output = directory.join(output_name);
                    if output.is_dir() {
                        let mut files = Vec::new();
                        walk(&output, &mut files);
                        failures.extend(
                            files
                                .iter()
                                .filter_map(|file| not_of_its_kind(file))
                                .map(|why| format!("{name}:{}: {why}", example.line)),
                        );
                        if files.is_empty() {
                            failures.push(format!(
                                "{name}:{}: $ {}\nno file in {output_name}",
                                example.line, example.command
                            ));
                        }
                    } else if !output.is_file() {
                        failures.push(format!(
                            "{name}:{}: $ {}\nno file {output_name}",
                            example.line, example.command
                        ));
                    }
                }
            }
        }
        for entry in std::fs::read_dir(&directory).unwrap() {
            if let Some(why) = not_of_its_kind(&entry.unwrap().path()) {
                failures.push(format!("{name}:{}: {why}", block.line));
            }
        }
    }
    (ran, failures)
}

/// Every example of the README gives the output the README shows.
#[test]
fn readme_examples() {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join("readme");
    let (ran, failures) = examples("README.md", README, &root);
    assert!(ran > 40, "only {ran} of the README's commands ran");
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

/// Every example of the guide gives the output the guide shows.
#[test]
fn guide_examples() {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join("guide");
    let mut ran = 0;
    let mut failures = Vec::new();
    for (name, text) in GUIDE {
        let (each, failed) = examples(name, text, &root.join(name));
        ran += each;
        failures.extend(failed);
    }
    assert!(ran > 40, "only {ran} of the guide's commands ran");
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

/// The commands whose help the guide's flags are looked up in, as the
/// words after `linlog`.
const COMMANDS: [&[&str]; 6] = [
    &["prove"],
    &["check"],
    &["interact"],
    &["seq", "print"],
    &["seq", "json"],
    &["seq", "fragment"],
];

/// The programs other than `linlog` whose calls the guide shows, and
/// whose flags are theirs.
const TOOLS: [&str; 3] = ["cargo", "nix", "rocq"];

/// The top-level keys of the settings that a flag's help names a field of.
const SETTINGS: [&str; 8] = [
    "clock",
    "limits",
    "search",
    "view",
    "batch",
    "ordinary",
    "input",
    "interactive",
];

/// Returns what `linlog` writes for the arguments, standard output and
/// error together.
fn linlog(arguments: &[&str], input: &str) -> String {
    let out = common::output(
        Command::new(env!("CARGO_BIN_EXE_linlog")).args(arguments),
        input,
    );
    let mut text = String::from_utf8(out.stdout).unwrap();
    text.push_str(&String::from_utf8(out.stderr).unwrap());
    text
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

/// Returns the code of a document that names commands: the command lines
/// of its `console` blocks and its inline code spans (a table's `\|` read
/// as `|`, a span across a line break joined by a blank), each with the
/// line it starts on, and its prose outside them. Other code blocks are
/// left out.
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
                let line = line.replace("\\|", "|");
                for (index, piece) in line.split('`').enumerate() {
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

/// Returns the flags a piece of code or prose names: every word that is
/// `--` and a lowercase name, or `-` and one letter, with a value after
/// `=` cut off.
fn flags_in(text: &str) -> Vec<String> {
    text.split(|c: char| c.is_whitespace() || "`'\"(),;".contains(c))
        .filter_map(|word| {
            let word = word.split('=').next().unwrap_or("");
            let long = word.strip_prefix("--").is_some_and(|name| {
                name.starts_with(|c: char| c.is_ascii_lowercase())
                    && name.chars().all(|c| c.is_ascii_lowercase() || c == '-')
            });
            let short = word.len() == 2
                && word.starts_with('-')
                && word[1..].chars().all(|c| c.is_ascii_alphabetic());
            (long || short).then(|| word.to_owned())
        })
        .collect()
}

/// Returns the text without what its quotes hold, a sequent or a
/// script: their `-o` and `|` are no flag and no pipe.
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

/// Whether the help names the flag: a long one as a word, a short one
/// in front of its long name.
fn names_flag(help: &str, flag: &str) -> bool {
    if flag.starts_with("--") {
        holds_word(help, flag)
    } else {
        help.contains(&format!("{flag}, --"))
    }
}

/// Every command, flag, session command and settings key the guide names
/// exists: a flag after `linlog COMMAND` in the command's help, any other
/// in some command's help or the session's, a session command in the
/// list `help` prints in a session, and a settings key in the help of the
/// flag that sets it.
#[test]
fn guide_names_what_exists() {
    let helps: Vec<(&[&str], String)> = COMMANDS
        .iter()
        .map(|command| {
            let mut arguments = command.to_vec();
            arguments.push("--help");
            (*command, linlog(&arguments, ""))
        })
        .collect();
    let session = linlog(&["interact", "A |- A"], "help\n");
    let session_commands: Vec<&str> = session
        .lines()
        .filter(|line| line.starts_with(|c: char| c.is_ascii_lowercase()))
        .filter_map(|line| line.split_whitespace().next())
        .collect();
    let every_help = helps
        .iter()
        .map(|(_, help)| help.as_str())
        .collect::<String>()
        + &session;
    let mut failures = Vec::new();
    for (name, text) in GUIDE {
        let (code, prose) = code_of(text);
        for flag in flags_in(&prose) {
            if !names_flag(&every_help, &flag) {
                failures.push(format!("{name}: the flag {flag} is in no command's help"));
            }
        }
        for (line, piece) in &code {
            let piece = &unquoted(piece);
            let first = piece.split_whitespace().next().unwrap_or("");
            for flag in flags_in(piece)
                .into_iter()
                .filter(|_| !TOOLS.contains(&first))
            {
                if !names_flag(&every_help, &flag) {
                    failures.push(format!(
                        "{name}:{line}: the flag {flag} is in no command's help"
                    ));
                }
            }
            // Each `linlog COMMAND …` up to the next pipe, list or redirection.
            for call in piece.split(['|', ';', '>']).filter_map(|part| {
                let at = part.find("linlog ")?;
                Some(&part[at + "linlog ".len()..])
            }) {
                let mut words = call.split_whitespace();
                let Some(first) = words.next() else { continue };
                if !first.starts_with(|c: char| c.is_ascii_lowercase()) {
                    continue;
                }
                let mut command = vec![first];
                if first == "seq" {
                    let Some(second) = words.next() else { continue };
                    command.push(second);
                }
                let Some((_, help)) = helps.iter().find(|(known, _)| *known == command.as_slice())
                else {
                    failures.push(format!(
                        "{name}:{line}: no command linlog {}",
                        command.join(" ")
                    ));
                    continue;
                };
                for flag in flags_in(&words.collect::<Vec<_>>().join(" ")) {
                    if !names_flag(help, &flag) {
                        failures.push(format!(
                            "{name}:{line}: linlog {} has no flag {flag}",
                            command.join(" ")
                        ));
                    }
                }
            }
            // A settings key such as `search.copies`.
            if let Some((top, _)) = piece.split_once('.')
                && SETTINGS.contains(&top)
                && piece
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c == '.' || c == '_')
                && !holds_word(&every_help, piece)
            {
                failures.push(format!("{name}:{line}: no flag sets the key {piece}"));
            }
        }
        // The session commands: the first column of the reference's table
        // and the summary's paragraph on the session.
        let mut in_table = false;
        for line in text.lines() {
            in_table = line.starts_with("| session command") || in_table && line.starts_with('|');
            let spans: Vec<&str> = if in_table && line.starts_with("| `") {
                line.split('`').nth(1).into_iter().collect()
            } else if line.starts_with("**Step by step.**") {
                line.split('`').skip(1).step_by(2).collect()
            } else {
                continue;
            };
            for span in spans {
                let word = span.split_whitespace().next().unwrap_or("");
                let is_word = !word.is_empty() && word.chars().all(|c| c.is_ascii_lowercase());
                // A span may name a command of `linlog` beside the session's.
                let a_command = COMMANDS.iter().any(|command| command[0] == word);
                if is_word && !a_command && !session_commands.contains(&word) {
                    failures.push(format!("{name}: no session command {word}"));
                }
            }
        }
    }
    // The checks find what they look for.
    assert!(session_commands.contains(&"apply"), "{session}");
    assert!(names_flag(&helps[0].1, "--copies") && names_flag(&helps[0].1, "-q"));
    assert!(!names_flag(&every_help, "--copy") && !names_flag(&every_help, "-z"));
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// Every Rust example of the guide is one of the library's crate-level
/// doctests, which the library's tests compile and run: its lines, those
/// the doctest hides (`# `) left out, stand together in that doctest.
#[test]
fn guide_s_rust_is_a_doctest() {
    let library = include_str!("../../core/src/lib.rs");
    let docs: Vec<&str> = library
        .lines()
        .filter_map(|line| line.strip_prefix("//!"))
        .map(|line| line.strip_prefix(' ').unwrap_or(line))
        .filter(|line| !line.starts_with("# "))
        .collect();
    let mut found = 0;
    for (name, text) in GUIDE {
        let mut block: Option<Vec<&str>> = None;
        for line in text.lines() {
            match (&mut block, line) {
                (None, "```rust") => block = Some(Vec::new()),
                (Some(lines), "```") => {
                    assert!(
                        docs.windows(lines.len())
                            .any(|window| window == lines.as_slice()),
                        "{name}: a Rust example that is no doctest of core/src/lib.rs:\n{}",
                        lines.join("\n")
                    );
                    found += 1;
                    block = None;
                }
                (Some(lines), line) => lines.push(line),
                (None, _) => {}
            }
        }
    }
    assert!(found > 0, "the guide has no Rust example");
}

/// The summary is an Agent Skill within its budget: a `name` matching its
/// directory and a `description` of at most 1 024 characters in its
/// front matter, and a body of fewer than 100 lines and at most 8 000
/// bytes, about 2 000 tokens.
#[test]
fn the_summary_keeps_its_budget() {
    let (front, body) = SUMMARY
        .strip_prefix("---\n")
        .and_then(|rest| rest.split_once("\n---\n"))
        .expect("SKILL.md starts with its front matter");
    let field = |key: &str| {
        front
            .lines()
            .find_map(|line| line.strip_prefix(key)?.strip_prefix(": "))
            .unwrap_or_else(|| panic!("SKILL.md has no {key}"))
    };
    assert_eq!(field("name"), "linlog");
    assert!(field("description").chars().count() <= 1024);
    assert!(body.lines().count() < 100, "{} lines", body.lines().count());
    assert!(body.len() <= 8000, "{} bytes", body.len());
}

/// Returns the section of a command's help that documents the flag: from
/// its line to the next flag's.
fn help_of<'h>(help: &'h str, flag: &str) -> Option<&'h str> {
    let starts = |line: &str| {
        let line = line.trim_start();
        line.starts_with("--") || (line.starts_with('-') && line.get(2..4) == Some(", "))
    };
    let mut lines = help.lines();
    let at =
        lines.position(|line| starts(line) && line.split([' ', ',']).any(|word| word == flag))?;
    let start = help
        .lines()
        .take(at)
        .map(|line| line.len() + 1)
        .sum::<usize>();
    let rest = &help[start..];
    let first = rest.find('\n').map_or(rest.len(), |end| end + 1);
    let end = rest[first..]
        .lines()
        .take_while(|line| !starts(line))
        .map(|line| line.len() + 1)
        .sum::<usize>();
    Some(&rest[..(first + end).min(rest.len())])
}

/// Whether the flag's section of the help names `value` as its default.
fn defaults_to(section: &str, value: &str) -> bool {
    section.contains(&format!("[default: {value}]"))
        || section.contains(&format!("default is {value}"))
        || section.contains(&format!("{value} (by default)"))
}

/// Every default and number the guide states is the command's: a span
/// `--flag VALUE` in a sentence that says "default", and every row of the
/// reference's table of defaults, names the default the flag's `--help`
/// gives; the batch's order of exit statuses is the one `prove --help`
/// states; the library's copy bound is `Options::default()`'s. The exit
/// statuses themselves are shown by examples that print them
/// (`; echo "exit $?"`), which `guide_examples` runs.
#[test]
fn guide_states_the_defaults() {
    let helps: Vec<String> = COMMANDS
        .iter()
        .map(|command| {
            let mut arguments = command.to_vec();
            arguments.push("--help");
            linlog(&arguments, "")
        })
        .collect();
    let mut failures = Vec::new();
    let mut checked = 0;
    // The defaults stated, as the document, the flag and the value.
    let mut stated: Vec<(&str, String, String)> = Vec::new();
    for (name, text) in GUIDE {
        let mut fence = false;
        let mut prose = String::new();
        let mut in_table = false;
        for line in text.lines() {
            if line.starts_with("```") {
                fence = !fence;
                continue;
            }
            if fence {
                continue;
            }
            in_table = line.starts_with("| flag | default |") || in_table && line.starts_with('|');
            if in_table {
                let cells: Vec<&str> = line.split('|').map(str::trim).collect();
                let flag = cells.get(1).and_then(|cell| cell.split('`').nth(1));
                let value = cells.get(2).and_then(|cell| cell.split('`').nth(1));
                if let (Some(flag), Some(value)) = (flag, value) {
                    let flag = flag.split_whitespace().next().unwrap_or("");
                    stated.push((name, flag.to_owned(), value.to_owned()));
                }
            } else if !line.starts_with('|') {
                prose.push_str(line);
                prose.push('\n');
            }
        }
        let prose = prose.replace('\n', " ");
        for sentence in prose
            .split(". ")
            .filter(|sentence| sentence.contains("default"))
        {
            for span in sentence.split('`').skip(1).step_by(2) {
                let mut words = span.split_whitespace();
                let (Some(flag), Some(value), None) = (words.next(), words.next(), words.next())
                else {
                    continue;
                };
                let literal = !value.starts_with(|c: char| c.is_ascii_uppercase());
                if flag.starts_with("--") && literal {
                    stated.push((name, flag.to_owned(), value.to_owned()));
                }
            }
        }
        let order = "an error (2) before unknown (3) before unprovable (1) before proved (0)";
        if prose.contains("exits with its worst") || prose.contains("the exit status is the worst")
        {
            checked += 1;
            if !prose.contains(order) || !helps[0].contains(order) {
                failures.push(format!(
                    "{name}: a batch's order of exit statuses is not `prove --help`'s"
                ));
            }
        }
        for (at, _) in prose.match_indices("bounds the copies") {
            checked += 1;
            let after = &prose[at..];
            let number = after
                .split("at ")
                .nth(1)
                .and_then(|rest| rest.split(|c: char| !c.is_ascii_digit()).next())
                .and_then(|digits| digits.parse::<u32>().ok());
            if number != linlog::Options::default().copies {
                failures.push(format!(
                    "{name}: the library's default copy bound is {:?}, not {number:?}",
                    linlog::Options::default().copies
                ));
            }
        }
    }
    for (name, flag, value) in &stated {
        checked += 1;
        let mut sections = helps.iter().filter_map(|help| help_of(help, flag));
        if !sections.any(|section| defaults_to(section, value)) {
            failures.push(format!("{name}: the default of {flag} is not {value}"));
        }
    }
    assert!(checked > 15, "only {checked} defaults were checked");
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
