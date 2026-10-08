// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The behaviour lock: what the command prints and the status it exits
//! with, on a fixed corpus of calls (every command, every output format,
//! every fragment and mode, ordinary logic, the problem files, the batch
//! and the errors), pinned in `tests/lock/out/NAME.txt`. A change of what
//! the command does shows here as a changed file; such a change is a
//! commit of its own that says why. `BLESS=1 cargo test --test lock`
//! rewrites the files.
//!
//! The calls run in `tests/lock/inputs`, which holds the files they read.
//! A call whose output carries the search's counters names
//! `--deterministic`, since a pool's counters add every thread's; the
//! others decide in microseconds, long before the pool would start. PNG
//! and PDF output is pinned by its length and an FNV-1a hash.

use std::fmt::Write as _;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// A call of the command: the name of its file, its arguments and its
/// standard input.
struct Case {
    /// The file's name, without `.txt`.
    name: &'static str,
    /// The arguments.
    args: &'static [&'static str],
    /// The standard input.
    stdin: &'static str,
}

/// A call with no standard input.
const fn call(name: &'static str, args: &'static [&'static str]) -> Case {
    Case {
        name,
        args,
        stdin: "",
    }
}

/// A call that reads standard input.
const fn piped(name: &'static str, args: &'static [&'static str], stdin: &'static str) -> Case {
    Case { name, args, stdin }
}

/// A proof of `A * B |- B * A`, as `prove --format json` writes it.
const PROOF: &str = r#"{"verdict":"proved","fragment":"MLL","mode":{"intuitionistic":false,"affine":false,"mix":false},"engine":"net","statistics":{"nodes":2,"memo_hits":0,"memo_entries":0,"splits":0,"links":2,"tests":2,"copies":0},"sequent":{"terms":[{"D":0},{"D":1},{"⅋":[0,1]},{"V":1},{"V":0},{"⊗":[3,4]}],"ids":[2,5],"var_dict":["A","B"]},"proof":[{"ax":[2,4]},{"ax":[1,5]},{"⊗":[3,0,1]},{"⅋":[0,2]}]}"#;

/// The same proof with its axioms crossed: no proof of the sequent.
const WRONG_PROOF: &str = r#"{"sequent":{"terms":[{"D":0},{"D":1},{"⅋":[0,1]},{"V":1},{"V":0},{"⊗":[3,4]}],"ids":[2,5],"var_dict":["A","B"]},"proof":[{"ax":[1,4]},{"ax":[2,5]},{"⊗":[3,0,1]},{"⅋":[0,2]}]}"#;

/// Every call the lock pins.
const CASES: &[Case] = &[
    // Fragments and modes, as text.
    call("prove-mll", &["prove", "A * B |- B * A"]),
    call("prove-mll-units", &["prove", "1, bot |- 1 * bot"]),
    call("prove-mall", &["prove", "A & B |- A + B"]),
    call("prove-mall-units", &["prove", "|- T, 0 + 1"]),
    call("prove-mell", &["prove", "!A |- A * !A"]),
    call("prove-ll", &["prove", "!(A & B) |- !A * ?B"]),
    call("prove-intuitionistic", &["prove", "-i", "A, A -o B |- B"]),
    call(
        "prove-intuitionistic-unprovable",
        &["prove", "-i", "A -o B |- B -o A"],
    ),
    call("prove-affine", &["prove", "-a", "A, B |- A"]),
    call("prove-mix", &["prove", "--mix", "A, B |- A, B"]),
    call("prove-horn", &["prove", "!(a -o a * b), a |- a * b * b"]),
    // Every refutation, as text and as JSON.
    call("refute-unbalanced", &["prove", "A |- A * A"]),
    call("refute-equation", &["prove", "|- A * ~A"]),
    call("refute-exhausted", &["prove", "A & B |- A * B"]),
    call(
        "refute-state-equation",
        &["prove", "!(A -o A * A), !(B * B -o C), A, B |- C"],
    ),
    call(
        "json-unbalanced",
        &["prove", "--deterministic", "--format", "json", "A |- A * A"],
    ),
    call(
        "json-equation",
        &["prove", "--deterministic", "--format", "json", "|- A * ~A"],
    ),
    call(
        "json-exhausted",
        &[
            "prove",
            "--deterministic",
            "--format",
            "json",
            "A & B |- A * B",
        ],
    ),
    call(
        "json-state-equation",
        &[
            "prove",
            "--deterministic",
            "--format",
            "json",
            "!(A -o A * A), !(B * B -o C), A, B |- C",
        ],
    ),
    // Every output format of a proof.
    call(
        "format-json",
        &[
            "prove",
            "--deterministic",
            "--format",
            "json",
            "!A |- A * !A",
        ],
    ),
    call(
        "format-json-intuitionistic",
        &[
            "prove",
            "--deterministic",
            "-i",
            "--format",
            "json",
            "A, A -o B |- B",
        ],
    ),
    call(
        "format-latex",
        &["prove", "--format", "latex", "A & B |- A + B"],
    ),
    call(
        "format-latex-standalone",
        &[
            "prove",
            "-i",
            "--format",
            "latex",
            "--standalone",
            "A, A -o B |- B",
        ],
    ),
    call(
        "format-typst",
        &["prove", "--format", "typst", "A & B |- A + B"],
    ),
    call(
        "format-svg",
        &["prove", "--format", "svg", "A * B |- B * A"],
    ),
    call(
        "format-png",
        &["prove", "--format", "png", "A * B |- B * A"],
    ),
    call(
        "format-pdf",
        &["prove", "--format", "pdf", "A * B |- B * A"],
    ),
    call(
        "format-rocq",
        &["prove", "--format", "rocq", "A * B |- B * A"],
    ),
    call(
        "format-rocq-standalone",
        &[
            "prove",
            "--format",
            "rocq",
            "--standalone",
            "--lemma",
            "swap",
            "!A |- A * !A",
        ],
    ),
    call("format-net", &["prove", "--net", "A * B |- B * A"]),
    call(
        "format-net-svg",
        &["prove", "--net", "--format", "svg", "A * B |- B * A"],
    ),
    call(
        "format-stats",
        &["prove", "--deterministic", "--stats", "!(A & B) |- !A * ?B"],
    ),
    call("format-quiet", &["prove", "--quiet", "A |- B"]),
    call("format-no-verdict", &["prove", "--no-verdict", "A |- A"]),
    call("format-tree-never", &["prove", "--tree", "never", "A |- A"]),
    call(
        "format-compact",
        &["prove", "--compact", "always", "A, B, C |- A * (B * C)"],
    ),
    call(
        "format-style",
        &[
            "prove",
            "--format",
            "latex",
            "--style",
            "labels=subscript",
            "--style",
            "align=false",
            "A, A -o B |- B",
        ],
    ),
    call(
        "format-derivation-limit",
        &["prove", "--derivation-limit", "100", "A * B |- B * A"],
    ),
    // Ordinary logic.
    call(
        "ordinary-classical",
        &["prove", "--logic", "classical", "a \\/ ~a"],
    ),
    call(
        "ordinary-intuitionistic",
        &["prove", "--logic", "intuitionistic", "a \\/ ~a"],
    ),
    call(
        "ordinary-minimal",
        &["prove", "--logic", "minimal", "a, a -> b |- b"],
    ),
    call(
        "ordinary-translation",
        &[
            "prove",
            "--logic",
            "intuitionistic",
            "--translation",
            "cbv",
            "(a /\\ b) -> (b /\\ a)",
        ],
    ),
    call(
        "ordinary-linear",
        &["prove", "--logic", "intuitionistic", "--linear", "a -> a"],
    ),
    call(
        "ordinary-json",
        &[
            "prove",
            "--deterministic",
            "--logic",
            "classical",
            "--format",
            "json",
            "~~a -> a",
        ],
    ),
    call(
        "ordinary-rocq",
        &[
            "prove",
            "--logic",
            "classical",
            "--format",
            "rocq",
            "a \\/ ~a",
        ],
    ),
    // Problem files.
    call("file-lltp", &["prove", "--file", "double.p"]),
    call(
        "file-tptp",
        &["prove", "--logic", "classical", "--file", "peirce.p"],
    ),
    call(
        "file-tptp-intuitionistic",
        &["prove", "--logic", "intuitionistic", "--file", "peirce.p"],
    ),
    call("file-spec", &["prove", "--file", "mutex.spec"]),
    call("file-spec-affine", &["prove", "-a", "--file", "mutex.spec"]),
    piped(
        "input-json",
        &["prove", "--input-format", "json"],
        r#"{"terms":[{"D":0},{"V":0}],"ids":[0,1],"var_dict":["A"]}"#,
    ),
    // The batch.
    call(
        "batch-lines",
        &["prove", "--input-format", "lines", "--file", "many.txt"],
    ),
    piped(
        "batch-jsonl",
        &["prove", "--input-format", "jsonl", "--file", "-"],
        "{\"terms\":[{\"D\":0},{\"V\":0}],\"ids\":[0,1],\"var_dict\":[\"A\"]}\n",
    ),
    // check.
    piped("check-valid", &["check"], PROOF),
    piped("check-invalid", &["check"], WRONG_PROOF),
    piped("check-not-json", &["check"], "A |- A"),
    // interact.
    piped(
        "interact-session",
        &["interact", "A * B |- B * A"],
        "goals\nrules 0 0\napply 0 0 par\nundo\napply 0 0 par\ngoals\nrules 1 2\nclose\nshow\nproof --json\nquit\n",
    ),
    piped(
        "interact-errors",
        &["interact", "A |- A"],
        "apply 9 0 ax\nrules 0 7\nfrobnicate\nproof\nquit\n",
    ),
    piped("interact-help", &["interact", "A |- A"], "help\n"),
    // seq.
    call("seq-print", &["seq", "print", "A -o B, !C |- ?D & 1"]),
    call(
        "seq-print-latex",
        &[
            "seq",
            "print",
            "-i",
            "--format",
            "latex",
            "A * B -o C |- ~C -o ~(A * B)",
        ],
    ),
    call("seq-json", &["seq", "json", "A, A -o B |- B"]),
    call("seq-fragment", &["seq", "fragment", "!A & B |- ?C"]),
    call(
        "seq-fragment-intuitionistic",
        &["seq", "fragment", "-i", "A & B |- A"],
    ),
    // Errors.
    call("error-parse", &["prove", "A * |- B"]),
    call(
        "error-parse-ordinary",
        &["prove", "--logic", "classical", "a /\\"],
    ),
    call("error-not-intuitionistic", &["prove", "-i", "|- ?A, !~A"]),
    call("error-engine", &["prove", "--engine", "net", "A & B |- A"]),
    call(
        "error-occurrence-limit",
        &["prove", "--occurrence-limit", "2", "A * B |- B * A"],
    ),
    call(
        "error-memory-limit",
        &["prove", "--memory-limit", "100", "!(A & B) |- !A * ?B"],
    ),
    call("error-missing-file", &["prove", "--file", "missing.p"]),
    call("error-unknown-flag", &["prove", "--frobnicate", "A |- A"]),
    call("error-no-command", &[]),
];

/// Runs one call in the inputs directory and returns what the lock pins:
/// the command line, the exit status, standard output and standard error.
fn run(case: &Case, inputs: &Path) -> String {
    let mut child = Command::new(env!("CARGO_BIN_EXE_linlog"))
        .args(case.args)
        .current_dir(inputs)
        // A PDF's date, so that every run writes the same bytes.
        .env("SOURCE_DATE_EPOCH", "1791158399")
        .env_remove("NO_COLOR")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    // A call that does not read its input may exit before the write: a
    // closed pipe is then no failure of the test.
    if let Err(error) = child.stdin.take().unwrap().write_all(case.stdin.as_bytes()) {
        assert_eq!(error.kind(), std::io::ErrorKind::BrokenPipe, "{error}");
    }
    let out = child.wait_with_output().unwrap();
    let mut text = String::from("$ linlog");
    for arg in case.args {
        if arg.is_empty()
            || arg.contains(|c: char| c.is_whitespace() || "\"'\\|*&!?~()<>;$".contains(c))
        {
            write!(text, " '{}'", arg.replace('\'', "'\\''")).unwrap();
        } else {
            write!(text, " {arg}").unwrap();
        }
    }
    if !case.stdin.is_empty() {
        text.push_str("\n--- stdin\n");
        text.push_str(case.stdin);
        if !case.stdin.ends_with('\n') {
            text.push('\n');
        }
    }
    writeln!(text, "\n--- status {}", out.status.code().unwrap_or(-1)).unwrap();
    text.push_str("--- stdout\n");
    match String::from_utf8(out.stdout) {
        Ok(stdout) => text.push_str(&timeless(&stdout)),
        Err(bytes) => {
            let bytes = bytes.into_bytes();
            writeln!(text, "{} bytes, FNV-1a {:016x}", bytes.len(), fnv(&bytes)).unwrap();
        }
    }
    text.push_str("--- stderr\n");
    text.push_str(&timeless(&String::from_utf8(out.stderr).unwrap()));
    text
}

/// Returns the text with every time that follows "after " or "time: " as
/// `…`: how long a search took is the machine's.
fn timeless(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(at) = ["after ", "time: "]
        .iter()
        .filter_map(|word| rest.find(word).map(|at| at + word.len()))
        .min()
    {
        let (head, tail) = rest.split_at(at);
        out.push_str(head);
        let time = tail
            .find(|c: char| !(c.is_ascii_digit() || "._µnms".contains(c)))
            .unwrap_or(tail.len());
        if time > 0 && tail.starts_with(|c: char| c.is_ascii_digit()) {
            out.push('…');
            rest = &tail[time..];
        } else {
            rest = tail;
        }
    }
    out.push_str(rest);
    out
}

/// The 64-bit FNV-1a hash of the bytes, which pins binary output without a
/// dependency.
fn fnv(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, &b| {
        (hash ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
    })
}

/// Every call of the corpus prints what its file pins, and every file in
/// `out/` is a call of the corpus.
#[test]
fn behaviour_lock() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/lock");
    let inputs = dir.join("inputs");
    let out = dir.join("out");
    let bless = std::env::var_os("BLESS").is_some();
    let mut changed = Vec::new();
    for case in CASES {
        let got = run(case, &inputs);
        let path = out.join(format!("{}.txt", case.name));
        let pinned = std::fs::read_to_string(&path).unwrap_or_default();
        if got != pinned {
            if bless {
                std::fs::write(&path, &got).unwrap();
            } else {
                let line = got
                    .lines()
                    .zip(pinned.lines())
                    .position(|(a, b)| a != b)
                    .unwrap_or_else(|| got.lines().count().min(pinned.lines().count()));
                changed.push(format!("{} (from line {})", case.name, line + 1));
            }
        }
    }
    for entry in std::fs::read_dir(&out).unwrap() {
        let name = entry.unwrap().file_name().into_string().unwrap();
        let stem = name.trim_end_matches(".txt");
        assert!(
            CASES.iter().any(|case| case.name == stem),
            "{name} in tests/lock/out is no call of the corpus"
        );
    }
    assert!(
        changed.is_empty(),
        "the command's behaviour changed (BLESS=1 rewrites the files): {}",
        changed.join(", ")
    );
}
