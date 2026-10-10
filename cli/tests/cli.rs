// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The `linlog` binary end to end: verdicts, exit statuses, the JSON round
//! trip from `prove` to `check`, the interactive session, the `seq`
//! commands and the help text.

use std::io::Write;
use std::process::{Command, Stdio};

/// Runs `linlog` with the arguments and `stdin` as standard input, and
/// returns the exit status, standard output and standard error.
fn linlog(args: &[&str], stdin: &str) -> (i32, String, String) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_linlog"))
        .args(args)
        // A PDF's date, so that every run writes the same bytes.
        .env("SOURCE_DATE_EPOCH", "1791158399")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    // A call that does not read its input may exit before the write: a
    // closed pipe is then no failure of the test.
    if let Err(error) = child.stdin.take().unwrap().write_all(stdin.as_bytes()) {
        assert_eq!(error.kind(), std::io::ErrorKind::BrokenPipe, "{error}");
    }
    let out = child.wait_with_output().unwrap();
    (
        out.status.code().unwrap(),
        String::from_utf8(out.stdout).unwrap(),
        String::from_utf8(out.stderr).unwrap(),
    )
}

/// Returns the text with every time that follows "after " as `…`: how
/// long a search took is the machine's.
fn timeless(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(at) = rest.find("after ") {
        let (head, tail) = rest.split_at(at + "after ".len());
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

/// A parse error shows the line it is in with a caret under the character
/// that cannot go on a sequent. Of a long line it shows the part around
/// that character, cut between characters, and says which character it
/// is; in an input of several lines it says which line.
#[test]
fn parse_errors() {
    let error = |args: &[&str], stdin: &str| {
        let (status, out, err) = linlog(args, stdin);
        assert_eq!((status, out.as_str()), (2, ""), "{err}");
        err
    };
    assert_eq!(
        error(&["seq", "fragment", "|- A * )"], ""),
        "error: cannot parse the sequent\n  |- A * )\n         ^ unexpected \")\", expected a formula\n"
    );

    // The 70 001st character of one line, after 17 499 characters of
    // three bytes each.
    let head = format!("|- a{}", " ⊗ a".repeat(17_499));
    assert_eq!(head.chars().count(), 70_000);
    let input = format!("{head}${}", " ⊗ a".repeat(100));
    assert_eq!(
        error(&["seq", "fragment"], &input),
        format!(
            "error: cannot parse the sequent\n  …{}${} ⊗ …\n  {}^ unexpected \"$\" at \
             character 70001, expected a connective, `,` or the end\n",
            " ⊗ a".repeat(15),
            " ⊗ a".repeat(14),
            " ".repeat(61)
        )
    );

    assert_eq!(
        error(&["seq", "fragment"], "A,\n  B * |- C\n"),
        "error: cannot parse the sequent\n    B * |- C\n        ^ unexpected \"|\" at line 2, \
         character 7, expected a formula\n"
    );
}

/// `--timeout` ends a search that would run for hours (2⁴² splits that no
/// count cuts) with an unknown verdict, the limit in its line and exit
/// status 3, on one thread and on a pool; and it counts from the start of
/// the command, so a sequent that is not read in time is given up on with
/// a line that says so.
#[test]
fn timeout() {
    let literals: Vec<String> = (0..40).map(|i| format!("x{i}")).collect();
    let sequent = format!(
        "|- p * q, 0 * (~p par ~p), 0 * (~q par ~q), {}",
        literals.join(", ")
    );
    for threads in [["--deterministic"].as_slice(), &["--jobs", "2"]] {
        let args = [&["prove", "-a", "--timeout", "200ms"], threads, &[&sequent]].concat();
        let (status, out, err) = linlog(&args, "");
        assert_eq!(
            (status, out.as_str(), err.as_str()),
            (
                3,
                "unknown (MALL, classical affine, focus engine): the time limit of 200ms was \
                 reached; --timeout DURATION gives the search longer\n",
                ""
            )
        );
    }

    // Standard input stays open and empty until the command has answered.
    let mut child = Command::new(env!("CARGO_BIN_EXE_linlog"))
        .args(["prove", "--timeout", "100ms"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let open = child.stdin.take();
    let status = child.wait().unwrap();
    drop(open);
    let mut out = String::new();
    std::io::Read::read_to_string(&mut child.stdout.take().unwrap(), &mut out).unwrap();
    assert_eq!(
        (status.code(), out.as_str()),
        (
            Some(3),
            "unknown: the time limit of 100ms was reached while the sequent was read\n"
        )
    );
}

/// `--jobs` beyond what the machine runs at once is taken as that many,
/// and standard error says so; the verdict and the status are the
/// search's.
/// A search that holds more than `--memory-limit` allows answers
/// "unknown" with the limit and the flag, exit status 3, and a sequent
/// that unfolds beyond `--occurrence-limit` is an error before it is
/// unfolded: 25 doublings of one atom are some 430 bytes of JSON and 67 million
/// occurrences.
#[test]
fn limits_on_memory_and_occurrences() {
    let literals: Vec<String> = (0..40).map(|i| format!("x{i}")).collect();
    let sequent = format!(
        "|- p * q, 0 * (~p par ~p), 0 * (~q par ~q), {}",
        literals.join(", ")
    );
    for threads in [["--deterministic"].as_slice(), &["--jobs", "2"]] {
        let args = [
            &["prove", "-a", "--memory-limit", "1KiB"],
            threads,
            &[&sequent],
        ]
        .concat();
        let (status, out, err) = linlog(&args, "");
        assert_eq!(
            (status, timeless(&out), err),
            (
                3,
                "unknown (MALL, classical affine, focus engine): the memory limit of 1 KiB was \
                 reached after …; raise it with --memory-limit SIZE\n"
                    .to_owned(),
                String::new()
            )
        );
    }

    let doublings: Vec<String> = (0..25).map(|i| format!(r#"{{"⊗":[{i},{i}]}}"#)).collect();
    let shared = format!(
        r#"{{"terms":[{{"V":0}},{}],"roots":[25],"atoms":["A"]}}"#,
        doublings.join(",")
    );
    for command in [
        ["prove"].as_slice(),
        &["seq", "print"],
        &["seq", "fragment"],
    ] {
        let (status, out, err) = linlog(&[command, &["--input-format", "json"]].concat(), &shared);
        assert_eq!(
            (status, out.as_str(), err.as_str()),
            (
                2,
                "",
                "error: the sequent unfolds to 67108863 subformula occurrences, more than the \
                 limit of 50000000; raise it with --occurrence-limit\n"
            )
        );
    }
    let (status, _, err) = linlog(
        &["seq", "fragment", "--occurrence-limit", "2", "A |- A * A"],
        "",
    );
    assert!(
        status == 2 && err.contains("to 4 subformula occurrences"),
        "{err}"
    );
    assert_eq!(
        linlog(
            &["seq", "fragment", "--occurrence-limit", "none", "A |- A"],
            ""
        ),
        (0, "MLL\n".to_owned(), String::new())
    );
}

#[test]
fn jobs_are_bounded() {
    let (status, out, err) = linlog(&["prove", "--quiet", "--jobs", "100000", "A |- A"], "");
    assert_eq!(
        (status, out.as_str()),
        (0, "provable (MLL, classical, net engine)\n")
    );
    assert!(
        err.starts_with("note: --jobs 100000 is more than the ")
            && err.contains(" threads a search"),
        "{err:?}"
    );
}

/// A derivation estimated above `--derivation-limit` is not written: the
/// verdict stands with its exit status, standard error says how large the
/// derivation is and how to get it, and `none` lifts the limit;
/// `--tree never` leaves the tree out without a word, and into a pipe
/// `--tree auto` writes it whatever its size.
#[test]
fn derivation_limit_and_tree_switch() {
    let sequent = "A, A -o B |- B";
    let verdict = "provable (MLL, classical, net engine)\n";
    let (status, out, err) = linlog(&["prove", "--derivation-limit", "100", sequent], "");
    assert_eq!((status, out.as_str()), (0, verdict));
    assert_eq!(
        err,
        "the derivation is not written: its 3 inferences with 29 characters of sequents are \
         estimated at 616 B, over the limit of 100 B; --format json writes the proof itself, \
         --derivation-limit SIZE raises the limit and --derivation-limit none lifts it\n"
    );
    let (status, out, err) = linlog(
        &[
            "prove",
            "--format",
            "latex",
            "--derivation-limit",
            "1KiB",
            sequent,
        ],
        "",
    );
    assert_eq!(status, 0);
    assert!(
        out.contains("\\begin{prooftree}") && err.is_empty(),
        "{out}{err}"
    );

    let (_, whole, _) = linlog(&["prove", sequent], "");
    for when in ["always", "auto"] {
        let (status, out, err) = linlog(
            &[
                "prove",
                "--tree",
                when,
                "--derivation-limit",
                "none",
                sequent,
            ],
            "",
        );
        assert_eq!((status, out, err), (0, whole.clone(), String::new()));
    }
    let (status, out, err) = linlog(&["prove", "--tree", "never", sequent], "");
    assert_eq!((status, out.as_str(), err.as_str()), (0, verdict, ""));

    // `check` is under the same limit.
    let (_, proof, _) = linlog(&["prove", "--format", "json", sequent], "");
    let (status, out, err) = linlog(&["check", "--derivation-limit", "100"], &proof);
    assert_eq!(
        (status, out.as_str()),
        (0, "valid proof of ⊢ ~A, A ⊗ ~B, B (classical)\n")
    );
    assert!(
        err.starts_with("the derivation is not written: its 3 inferences"),
        "{err}"
    );
}

/// A drawing past a bound is left out with a line that names the flag,
/// and no file is made: a PNG or PDF render the memory bound refuses
/// before it starts, and a proof net's drawing past `--derivation-limit`;
/// `seq print`, whose drawing is its whole output, fails instead.
#[test]
fn bounded_drawings() {
    let file = scratch("bounded.pdf");
    let path = file.to_str().unwrap();
    let args = ["prove", "-o", path, "--memory-limit", "1000", "A |- A"];
    let (status, out, err) = linlog(&args, "");
    assert_eq!((status, out.as_str()), (0, ""));
    assert!(
        err.starts_with(
            "provable (MLL, classical, net engine)\nthe drawing is not rendered: the PDF is \
             estimated to take "
        ) && err.contains("over the memory limit of 1000 B; --memory-limit SIZE raises the limit"),
        "{err}"
    );
    assert!(!file.exists());

    let net = [
        "prove",
        "--net",
        "--format",
        "svg",
        "--derivation-limit",
        "100",
    ];
    let (status, out, err) = linlog(&[&net[..], &["A * B |- B * A"]].concat(), "");
    assert_eq!((status, out.as_str()), (0, ""));
    assert!(
        err.contains("the proof net is not drawn: its 6 occurrences and 2 links are estimated at ")
            && err.contains("--derivation-limit SIZE raises the limit"),
        "{err}"
    );

    let file = scratch("bounded.png");
    let args = [
        "seq",
        "print",
        "--format",
        "png",
        "-o",
        file.to_str().unwrap(),
    ];
    let (status, _, err) = linlog(
        &[&args[..], &["--memory-limit", "1KiB", "A |- A"]].concat(),
        "",
    );
    assert_eq!(status, 2);
    assert!(err.contains("the drawing is not rendered"), "{err}");
    assert!(!file.exists());
}

/// `prove` prints the verdict line and the derivation, and exits 0 for
/// provable, 1 for unprovable, 3 for unknown and 2 for an error.
#[test]
fn prove_verdicts_and_exit_statuses() {
    let (status, out, _) = linlog(&["prove", "A, A -o B |- B"], "");
    assert_eq!(status, 0);
    assert_eq!(
        out,
        "provable (MLL, classical, net engine)\n\
         ─────── ax   ─────── ax\n\
         ⊢ ~A, A      ⊢ ~B, B\n\
         ──────────────────── ⊗\n\
         \x20 ⊢ ~A, A ⊗ ~B, B\n"
    );

    for (args, status, line) in [
        (
            &["prove", "-q", "--mix", "|- A par B, ~A, ~B"][..],
            0,
            "provable (MLL, classical with Mix, net engine)",
        ),
        (
            &["prove", "|- A par B, ~A, ~B"],
            1,
            "unprovable (MLL, classical, net engine): the count equation fails: a provable \
             one-sided sequent of MLL has exactly #⊗ − #⅋ − #1 + #⊥ + 2 formulas, here 0 − 1 − 0 \
             + 0 + 2 = 1, and this one has 3",
        ),
        (
            &["prove", "-q", "--engine", "focus", "A * B |- A * B"],
            0,
            "provable (MLL, classical, focus engine)",
        ),
        (
            &[
                "prove",
                "--engine",
                "focus",
                "--recursion-limit",
                "1",
                "A * B |- A * B",
            ],
            3,
            "unknown (MLL, classical, focus engine): the recursion limit of 1 was reached \
             after …; raise it with --recursion-limit N",
        ),
        (
            &["prove", "-q", "--fragment", "mall", "A |- A"],
            0,
            "provable (MALL as asserted, classical, focus engine)",
        ),
        (
            &[
                "prove",
                "-q",
                "--engine",
                "focus",
                "--copies",
                "0",
                "--forward-copies",
                "0",
                "!A |- A",
            ],
            3,
            "unknown (MELL, classical, focus engine): the copy bound of 0 was reached after \
             …; raise it with --copies N, or lift it with --copies none to deepen it while the \
             time limit lasts",
        ),
        (
            &["prove", "-q", "-a", "A, B |- A"],
            0,
            "provable (MLL, classical affine, focus engine)",
        ),
    ] {
        let (code, out, err) = linlog(args, "");
        assert_eq!(
            (code, timeless(&out), err),
            (status, format!("{line}\n"), String::new())
        );
    }

    let (status, out, _) = linlog(
        &[
            "prove",
            "-q",
            "--stats",
            "--deterministic",
            "|- a * b, ~a par ~b",
        ],
        "",
    );
    assert_eq!(status, 0);
    assert!(
        out.starts_with(
            "provable (MLL, classical, net engine)\nliterals chosen: 2\nlinks tried: 2\n\
             exact tests run: 2\ntime: "
        ),
        "{out}"
    );

    for (args, error) in [
        (&["prove", "A * |- A"][..], "cannot parse the sequent"),
        (
            &["prove", "-i", "|- A par B"],
            "not an intuitionistic sequent",
        ),
        (
            &["prove", "--fragment", "mll", "A & B |- A"],
            "outside the asserted fragment MLL",
        ),
        (
            &["prove", "--engine", "net", "A & B |- A"],
            "proof nets exist for MLL without units only, not for ALL",
        ),
    ] {
        let (status, out, err) = linlog(args, "");
        assert_eq!((status, out.as_str()), (2, ""), "{args:?}");
        assert!(err.contains(error), "{args:?}: {err}");
    }
}

/// Intuitionistic mode: the verdict line names the intuitionistic fragment
/// and the engine, the derivation is two-sided, `check -i` reads what
/// `prove -i` writes and prints the sequent two-sided, and a sequent with
/// no intuitionistic reading is an error that names the subformula.
#[test]
fn intuitionistic_mode() {
    let (status, out, _) = linlog(&["prove", "-i", "A, A -o B |- B"], "");
    assert_eq!(status, 0);
    assert_eq!(
        out,
        "provable (IMLL, intuitionistic, net engine)\n\
         ───── ax   ───── ax\n\
         A ⊢ A      B ⊢ B\n\
         ──────────────── ⊸L\n\
         \x20 A, A ⊸ B ⊢ B\n"
    );
    for (args, status, line) in [
        (
            &["prove", "-i", "-q", "A & B |- A"][..],
            0,
            "provable (IALL, intuitionistic, additive engine)",
        ),
        (
            &["prove", "-i", "-q", "!A, !(A -o B) |- !B"],
            0,
            "provable (IMELL, intuitionistic, two-sided engine)",
        ),
        (
            &["prove", "-i", "-a", "-q", "A, B |- A"],
            0,
            "provable (IMLL, intuitionistic affine, two-sided engine)",
        ),
        (
            &["prove", "-i", "-q", "--engine", "two-sided", "A |- A"],
            0,
            "provable (IMLL, intuitionistic, two-sided engine)",
        ),
        (
            &["prove", "-i", "-q", "(A -o B) -o A |- A"],
            1,
            "unprovable (IMLL, intuitionistic, net engine): ~A occurs 1 more time than A in the \
             one-sided sequent, so they cannot all meet in axioms",
        ),
        (
            &["prove", "-q", "--engine", "additive", "|- A & B, ~A + ~B"],
            0,
            "provable (ALL, classical, additive engine)",
        ),
    ] {
        let (got, out, err) = linlog(args, "");
        assert_eq!(got, status, "{args:?}: {err}");
        assert_eq!(out.lines().next(), Some(line), "{args:?}");
    }
    for (args, error) in [
        (
            &["prove", "-i", "|- A par B"][..],
            "not an intuitionistic sequent: the subformula A ⅋ B is neither an intuitionistic \
             formula nor the negation of one",
        ),
        (
            &["prove", "-i", "A |- B, C"],
            "exactly one formula right of ⊢, not 2: write the hypotheses left of ⊢",
        ),
        (
            &["prove", "-i", "--engine", "focus", "A |- A"],
            "the focus engine does not search in intuitionistic mode",
        ),
        (
            &["prove", "-i", "--mix", "A |- A"],
            "Mix has no intuitionistic form",
        ),
        (
            &["prove", "--engine", "additive", "A, B |- A * B"],
            "the additive engine decides a sequent of two additive-only formulas, not 3 formulas of MLL",
        ),
    ] {
        let (status, out, err) = linlog(args, "");
        assert_eq!((status, out.as_str()), (2, ""), "{args:?}");
        assert!(err.contains(error), "{args:?}: {err}");
    }

    let (status, json, _) = linlog(&["prove", "-i", "--format", "json", "A & B |- B"], "");
    assert_eq!(status, 0);
    assert!(
        json.contains(r#""verdict":"proved","checked":true,"fragment":"IALL","#),
        "{json}"
    );
    let (status, out, _) = linlog(&["check", "-i"], &json);
    assert_eq!(status, 0);
    assert_eq!(
        out,
        "valid proof of A & B ⊢ B (intuitionistic)\n\
         \x20 ───── ax\n\
         \x20 B ⊢ B\n\
         ───────── &L₂\n\
         A & B ⊢ B\n"
    );
    // The classical proof of a sequent classical linear logic proves and
    // intuitionistic linear logic does not is an invalid proof under -i.
    let schellinx = "((A * top) & (B * top)) -o 0 |- (A -o C) + (B -o C)";
    let (status, json, _) = linlog(&["prove", "--format", "json", schellinx], "");
    assert_eq!(status, 0);
    let (status, out, _) = linlog(&["check", "-i", "-q"], &json);
    assert_eq!(status, 1);
    assert!(
        out.contains("a sequent of the rule has 2 formulas on the right of ⊢ instead of one"),
        "{out}"
    );

    let (status, out, _) = linlog(&["seq", "print", "-i", "A, A -o B |- B"], "");
    assert_eq!((status, out.as_str()), (0, "A, A ⊸ B ⊢ B\n"));
    let (status, _, err) = linlog(&["seq", "print", "-i", "A, B |-"], "");
    assert_eq!(status, 2);
    assert!(
        err.contains("exactly one formula right of ⊢, not 0"),
        "{err}"
    );
    let (status, out, _) = linlog(&["seq", "fragment", "-i", "!A |- A & 1"], "");
    assert_eq!((status, out.as_str()), (0, "ILL\n"));
}

/// `interact` reads commands from standard input: rules are listed and
/// applied by goal and position, refused with a reason, undone, goals are
/// closed by the search, the session is saved and resumed, and the finished
/// proof is checked, which decides the exit status.
#[test]
fn interactive_session() {
    let dir = std::env::temp_dir().join(format!("linlog-cli-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let session = dir.join("session.json");
    let proof = dir.join("proof.json");
    let script = format!(
        "goals\nrules 0 1\napply 0 1 par\napply 0 1 * 0\napply 1 0 ax\nundo\nsave {s}\nquit\n",
        s = session.display()
    );
    let (status, out, err) = linlog(&["interact", "A, A -o B |- B"], &script);
    assert_eq!(
        out,
        format!(
            "goal 0: ⊢ 0: ~A, 1: A ⊗ ~B, 2: B\n\
             ⊗ (with a split)\n\
             error: the rule ⅋ does not act on formula 1\n\
             opened goal 1: ⊢ 0: ~A, 1: A\n\
             opened goal 2: ⊢ 0: ~B, 1: B\n\
             closed\n\
             reopened goal 1: ⊢ 0: ~A, 1: A\n\
             session written to {}\n",
            session.display()
        )
    );
    assert_eq!((status, err.as_str()), (1, ""));
    let script = format!(
        "close 1\napply 2 1 ax\nshow\nproof {p}\n",
        p = proof.display()
    );
    let (status, out, err) = linlog(&["interact", "--state", session.to_str().unwrap()], &script);
    assert_eq!(
        out,
        format!(
            "goal 1: proved (MLL, classical, focus engine)\n\
             closed; no goal is open: `proof` checks the proof\n\
             ─────── ax   ─────── ax\n\
             ⊢ ~A, A      ⊢ ~B, B\n\
             ──────────────────── ⊗\n\
            \x20 ⊢ ~A, A ⊗ ~B, B\n\
             valid proof written to {}\n",
            proof.display()
        )
    );
    assert_eq!((status, err.as_str()), (0, ""));
    let (status, _, _) = linlog(&["check", "-q", proof.to_str().unwrap()], "");
    assert_eq!(status, 0);
    std::fs::remove_dir_all(&dir).unwrap();

    // Two-sided: the goals show both sides, the rules their ILL names, and
    // a split that leaves two goals on a side is refused.
    let script = "goals\nrules 0 1\napply 0 1 -oL 2\napply 0 1 -oL 0\nclose\nproof\n";
    let (status, out, err) = linlog(&["interact", "-i", "A, A -o B |- B"], script);
    assert_eq!(
        out,
        "goal 0: 0: A, 1: A ⊸ B ⊢ 2: B\n\
         ⊸L (with a split)\n\
         error: a premise would have 2 formulas on the right of ⊢ instead of one\n\
         opened goal 1: 0: A ⊢ 1: A\n\
         opened goal 2: 0: B ⊢ 1: B\n\
         goal 1: proved (IMLL, intuitionistic, two-sided engine)\n\
         goal 2: proved (IMLL, intuitionistic, two-sided engine)\n\
         no goal is open: `proof` checks the proof\n\
         valid proof (intuitionistic)\n\
         ───── ax   ───── ax\n\
         A ⊢ A      B ⊢ B\n\
         ──────────────── ⊸L\n\
        \x20 A, A ⊸ B ⊢ B\n"
    );
    assert_eq!((status, err.as_str()), (0, ""));
    let (status, _, err) = linlog(&["interact"], "");
    assert_eq!(status, 2);
    assert!(err.contains("no sequent given"), "{err}");
}

/// The JSON output of `prove` is a proof file that `check` accepts in the
/// mode it was found in and rejects in a stricter one.
#[test]
fn check_reads_what_prove_writes() {
    let (status, json, _) = linlog(
        &["prove", "--mix", "--format", "json", "|- A par B, ~A, ~B"],
        "",
    );
    assert_eq!(status, 0);
    assert!(json.contains(r#""verdict":"proved","checked":true,"fragment":"MLL","#));

    let (status, out, _) = linlog(&["check", "--mix", "-q"], &json);
    assert_eq!(
        (status, out.as_str()),
        (0, "valid proof of ⊢ A ⅋ B, ~A, ~B (classical with Mix)\n")
    );
    let (status, out, _) = linlog(&["check"], &json);
    assert_eq!(status, 1);
    assert!(
        out.starts_with("invalid proof of ⊢ A ⅋ B, ~A, ~B (classical): node 2 (mix from 0, 1)"),
        "{out}"
    );
}

/// `--format net` prints the proof net of a proof after the verdict line,
/// for `prove` and for `check`, and is refused outside unit-free MLL and
/// classical mode.
#[test]
fn net_format() {
    let expected = "⊢ ~A, A ⊗ ~B, B\n\
                    ~A[0] — A[2]\n\
                    ~B[3] — B[4]\n\
                    proof net\n";
    let (status, out, _) = linlog(&["prove", "--net", "A, A -o B |- B"], "");
    assert_eq!(status, 0);
    assert_eq!(
        out,
        format!("provable (MLL, classical, net engine)\n{expected}")
    );
    let (status, out, _) = linlog(
        &["prove", "--net", "--engine", "focus", "A, A -o B |- B"],
        "",
    );
    assert_eq!(status, 0);
    assert_eq!(
        out,
        format!("provable (MLL, classical, focus engine)\n{expected}")
    );
    let (_, json, _) = linlog(&["prove", "--format", "json", "A, A -o B |- B"], "");
    let (status, out, _) = linlog(&["check", "--net"], &json);
    assert_eq!(status, 0);
    assert!(out.ends_with("~B[3] — B[4]\nproof net\n"), "{out}");
    for (args, error) in [
        (
            &["prove", "--net", "A & B |- A"][..],
            "proof nets exist for MLL without units only, not for ALL",
        ),
        (
            &["prove", "--net", "--affine", "A |- A"],
            "proof nets exist in linear mode only",
        ),
    ] {
        let (status, out, err) = linlog(args, "");
        assert_eq!((status, out.as_str()), (2, ""), "{args:?}");
        assert!(err.contains(error), "{args:?}: {err}");
    }
}

/// `--format latex` and `--format typst` print the verdict as a comment and
/// the derivation as a proof tree, for `prove`, `check` and the session's
/// `show`, and the sequent for `seq print`; `--standalone` makes a document
/// and is refused for the other formats.
#[test]
fn latex_and_typst_formats() {
    let (status, out, _) = linlog(&["prove", "--format", "latex", "A, A -o B |- B"], "");
    assert_eq!(status, 0);
    assert_eq!(
        out,
        "% provable (MLL, classical, net engine)\n\
         \\begin{prooftree}\n\
         \\infer0[$\\mathrm{ax}$]{\\vdash A^\\bot, A}\n\
         \\infer0[$\\mathrm{ax}$]{\\vdash B^\\bot, B}\n\
         \\infer2[$\\otimes$]{\\vdash A^\\bot, A \\otimes B^\\bot, B}\n\
         \\end{prooftree}\n"
    );
    let (_, json, _) = linlog(&["prove", "-i", "--format", "json", "A, A -o B |- B"], "");
    let (status, out, _) = linlog(&["check", "-i", "--format", "typst", "--standalone"], &json);
    assert_eq!(status, 0);
    assert!(
        out.starts_with(
            "// valid proof of A, A ⊸ B ⊢ B (intuitionistic)\n\
             #import \"@preview/curryst:0.6.0\": prooftree, rule\n"
        ),
        "{out}"
    );
    let printed = linlog(
        &["seq", "print", "-i", "--format", "latex", "A, A -o B |- B"],
        "",
    );
    assert_eq!(
        printed,
        (0, "$A, A \\multimap B \\vdash B$\n".into(), String::new())
    );
    let (status, out, err) = linlog(&["prove", "--standalone", "A |- A"], "");
    assert_eq!((status, out.as_str()), (2, ""));
    assert!(err.contains("--standalone needs --format latex"), "{err}");
    let (status, out, _) = linlog(&["interact", "A |- A"], "show --latex\nquit\n");
    assert_eq!(status, 1);
    assert_eq!(
        out,
        "\\begin{prooftree}\n\
         \\hypo{\\vdots}\n\
         \\infer[no rule]1{\\vdash A^\\bot, A}\n\
         \\end{prooftree}\n"
    );
}

/// `--format rocq` prints the verdict as a Rocq comment and the
/// derivation as a lemma with its proof script for `prove` and `check`;
/// `--standalone` adds the import; a proof with Mix has no certificate.
#[test]
fn rocq_format() {
    let (status, out, _) = linlog(&["prove", "--format", "rocq", "A |- A"], "");
    assert_eq!(status, 0);
    assert_eq!(
        out,
        "(* provable (MLL, classical, net engine) *)\n\
         Lemma certificate (A : formula) : ll [dual A; A].\nProof.\nax_expansion.\nQed.\n"
    );
    let (_, json, _) = linlog(&["prove", "--format", "json", "A * B |- B * A"], "");
    let (status, out, _) = linlog(&["check", "--format", "rocq", "--standalone"], &json);
    assert_eq!(status, 0);
    assert!(
        out.starts_with(
            "(* valid proof of ⊢ ~A ⅋ ~B, B ⊗ A (classical) *)\n\
             From NanoYalla Require Import macroll.\n\nLemma certificate (A B : formula)"
        ),
        "{out}"
    );
    assert!(out.contains("apply (ex_perm_r [2; 0; 1] [dual B; tens B A; dual A]).\n"));
    let (status, out, err) = linlog(&["prove", "--mix", "--format", "rocq", "A, B |- A, B"], "");
    assert_eq!((status, out.as_str()), (2, ""));
    assert!(err.contains("NanoYalla has no Mix rule"), "{err}");
}

/// `--format svg`, with and without `--net`, prints the verdict as an XML
/// comment, with no `--` in it, before the derivation or the proof net as
/// an SVG document, and nothing without one, for `prove`, `check`, `seq print` and the session's
/// `show`; an SVG is always a document, so `--standalone` is refused.
#[test]
fn svg_formats() {
    let (status, out, _) = linlog(&["prove", "--format", "svg", "A |- A"], "");
    assert_eq!(status, 0);
    assert!(
        out.starts_with("<!-- provable (MLL, classical, net engine) -->\n<svg ")
            && out.ends_with("</svg>\n"),
        "{out}"
    );
    let (_, json, _) = linlog(&["prove", "--format", "json", "A * B |- B * A"], "");
    let (status, out, _) = linlog(&["check", "--net", "--format", "svg"], &json);
    assert_eq!(status, 0);
    assert!(out.contains("<circle id=\"o0\""), "{out}");
    let bounds = ["--copies", "0", "--forward-copies", "0"];
    let args = [&["prove", "--format", "svg"], &bounds[..], &["|- ?A"]].concat();
    // Without a drawing an SVG output is nothing, never a comment alone;
    // the verdict goes to standard error.
    let (status, out, err) = linlog(&args, "");
    assert_eq!((status, out.as_str()), (3, ""));
    assert!(err.contains("while the time limit lasts"), "{err}");
    let (status, out, _) = linlog(&["seq", "print", "--format", "svg", "A |- A"], "");
    assert_eq!(status, 0);
    assert!(out.contains("<title>⊢ A⊥, A</title>"), "{out}");
    let (status, _, err) = linlog(&["prove", "--format", "svg", "--standalone", "A |- A"], "");
    assert_eq!(status, 2);
    assert!(err.contains("--standalone needs --format latex"), "{err}");
    let (_, out, _) = linlog(&["interact", "A |- A"], "show --svg\nquit\n");
    assert!(
        out.starts_with("<svg ") && out.contains(">⋮</text>"),
        "{out}"
    );
}

/// `seq` prints a sequent one-sided, as JSON that it reads back, and its
/// fragment.
#[test]
fn seq_commands() {
    let (status, json, _) = linlog(&["seq", "json", "A, A -o B |- B"], "");
    assert_eq!(status, 0);
    assert_eq!(
        json,
        "{\"version\":1,\"terms\":[{\"D\":0},{\"V\":0},{\"D\":1},{\"⊗\":[1,2]},{\"V\":1}],\
         \"roots\":[0,3,4],\"atoms\":[\"A\",\"B\"],\"antecedents\":2}\n"
    );
    let printed = linlog(&["seq", "print", "--input-format", "json"], &json);
    assert_eq!(printed, (0, "⊢ ~A, A ⊗ ~B, B\n".into(), String::new()));
    let fragment = linlog(&["seq", "fragment", "--file", "-"], "A & B |- 1");
    assert_eq!(fragment, (0, "MALL\n".into(), String::new()));
}

/// The help lists every command.
#[test]
fn help_names_every_command() {
    let (status, out, _) = linlog(&["--help"], "");
    assert_eq!(status, 0);
    for command in ["prove", "check", "interact", "seq"] {
        assert!(out.contains(&format!("\n  {command} ")), "{command}: {out}");
    }
    let (_, out, _) = linlog(&["seq", "--help"], "");
    for command in ["print", "json", "fragment"] {
        assert!(out.contains(&format!("\n  {command} ")), "{command}: {out}");
    }
}

/// Returns the path of the file `name` in a directory of this test
/// process's own, apart from the session test's, which removes its own.
fn scratch(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("linlog-cli-files-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join(name)
}

/// PNG and PDF are drawn from the SVG, the derivation or with `--net` the
/// net, into a file whose extension names the format unless `--format`
/// does; the verdict goes to standard error. `--net` takes the drawn
/// formats and text only.
#[test]
fn binary_formats() {
    let (png, pdf) = (scratch("proof.png"), scratch("net.pdf"));
    let png_path = png.to_str().unwrap();
    let (status, out, err) = linlog(&["prove", "-o", png_path, "A * B |- B * A"], "");
    assert_eq!((status, out.as_str()), (0, ""), "{err}");
    assert!(err.starts_with("provable"), "{err}");
    assert!(std::fs::read(&png).unwrap().starts_with(b"\x89PNG"));
    let pdf_path = pdf.to_str().unwrap();
    let (status, _, _) = linlog(&["prove", "--net", "-o", pdf_path, "A * B |- B * A"], "");
    assert_eq!(status, 0);
    assert!(std::fs::read(&pdf).unwrap().starts_with(b"%PDF-"));
    // `--format` wins over the extension.
    let (status, _, _) = linlog(&["prove", "--format", "text", "-o", png_path, "A |- A"], "");
    assert_eq!(status, 0);
    assert!(
        std::fs::read_to_string(&png)
            .unwrap()
            .starts_with("provable")
    );
    // An unprovable sequent leaves no derivation, so no file but JSON's,
    // and its verdict on standard error.
    for name in ["none.pdf", "none.svg", "none.tex", "none.json"] {
        let file = scratch(name);
        let (status, out, err) = linlog(&["prove", "-o", file.to_str().unwrap(), "A |- B"], "");
        assert_eq!((status, out.as_str()), (1, ""), "{name}");
        assert_eq!(file.exists(), name.ends_with(".json"), "{name}");
        assert_eq!(
            err.starts_with("unprovable"),
            !name.ends_with(".json"),
            "{name}"
        );
    }
    // A session writes its derivation so far, open goals included, in
    // the format its file's extension names.
    let part = scratch("part.pdf");
    let commands = format!("apply 0 1 -oL 0\nshow {}\nquit\n", part.to_str().unwrap());
    let (_, out, _) = linlog(&["interact", "-i", "A, A -o B |- B"], &commands);
    assert!(out.contains("derivation so far written to"), "{out}");
    assert!(std::fs::read(&part).unwrap().starts_with(b"%PDF-2.0"));
    let (status, _, err) = linlog(&["prove", "--net", "--format", "latex", "A |- A"], "");
    assert_eq!(status, 2);
    assert!(
        err.contains("--net writes the formats text, svg, png and pdf"),
        "{err}"
    );
}

/// Every option of every format, set once with `--style KEY=VALUE` and
/// once from a `--style-file`, gives the same output, and one that differs
/// from the default's where the input shows the option.
#[test]
fn every_style_option() {
    let ill = ["-i", "A, A -o B |- B"];
    // The arguments, the format's key, the option, its value, and whether
    // the input shows it. Open goals exist in a session only.
    let rows: &[(&[&str], &str, &str, &str, bool)] = &[
        (&ill, "text", "labels", "\"off\"", true),
        (&ill, "text", "bar", "\"=\"", true),
        (&["A * B |- A * B"], "text", "gap", "7", true),
        (&ill, "latex", "form", "\"standalone\"", true),
        (&ill, "latex", "labels", "\"off\"", true),
        (&ill, "latex", "align", "false", true),
        (&ill, "latex", "ebproof", "\"center=false\"", true),
        (
            &["--standalone", "-i", "A |- A"],
            "latex",
            "preamble",
            "\"\\\\documentclass{article}\"",
            true,
        ),
        (&ill, "typst", "form", "\"standalone\"", true),
        (
            &ill,
            "typst",
            "labels",
            "{\"table\":{\"⊸L\":\"⊸_L\"}}",
            true,
        ),
        (
            &["--standalone", "A |- A"],
            "typst",
            "import",
            "\"#import \\\"x.typ\\\": *\"",
            true,
        ),
        (
            &["--standalone", "A |- A"],
            "typst",
            "page",
            "\"#set page(margin: 1cm)\"",
            true,
        ),
        (
            &ill,
            "svg",
            "font",
            "{\"family\":\"monospace\",\"advances\":{\"fixed\":600}}",
            true,
        ),
        (&ill, "typst", "layout", "\"linlog\"", true),
        (
            &["--style", "layout=linlog", "-i", "A, A -o B |- B"],
            "typst",
            "premise_gap",
            "\"2em\"",
            true,
        ),
        (
            &["--style", "layout=linlog", "-i", "A, A -o B |- B"],
            "typst",
            "label_gap",
            "\"1em\"",
            true,
        ),
        (
            &["--style", "layout=linlog", "-i", "A, A -o B |- B"],
            "typst",
            "band",
            "\"2em\"",
            true,
        ),
        (
            &["--style", "layout=linlog", "-i", "A, A -o B |- B"],
            "typst",
            "stroke",
            "\"1pt\"",
            true,
        ),
        (&ill, "svg", "labels", "\"subscript\"", true),
        (&ill, "svg", "ids", "true", true),
        (&ill, "svg", "font_size", "20", true),
        (&ill, "svg", "label_size", "700", true),
        (&ill, "svg", "line_height", "1700", true),
        (&ill, "svg", "premise_gap", "2000", true),
        (
            &["--net", "A * B |- B * A"],
            "svg",
            "literal_gap",
            "1200",
            true,
        ),
        (&ill, "svg", "label_gap", "300", true),
        (&ill, "svg", "margin", "500", true),
        (&ill, "svg", "stroke_width", "60", true),
        (
            &["--net", "A * B |- B * A"],
            "svg",
            "link_height",
            "700",
            true,
        ),
        (
            &["--net", "A * B |- B * A"],
            "svg",
            "link_cap",
            "1000",
            true,
        ),
        (
            &["--net", "A * B |- B * A"],
            "svg",
            "node_radius",
            "400",
            true,
        ),
        (&ill, "svg", "text", "\"navy\"", true),
        (&ill, "svg", "line", "\"navy\"", true),
        (&["--net", "A * B |- B * A"], "svg", "par", "\"navy\"", true),
        (
            &["--net", "A * B |- B * A"],
            "svg",
            "link",
            "\"navy\"",
            true,
        ),
        // Only a structure that is no net shows the highlight.
        (
            &["--net", "A * B |- B * A"],
            "svg",
            "highlight",
            "\"navy\"",
            false,
        ),
        (&ill, "svg", "background", "\"white\"", true),
        (&ill, "png", "scale", "1", true),
        (&ill, "png", "pixels", "100", true),
        (&ill, "png", "memory", "1000", true),
        (&ill, "pdf", "memory", "1000", true),
        (&ill, "pdf", "embed_text", "false", true),
        (&ill, "pdf", "compatible", "true", true),
        (&ill, "pdf", "accessible", "true", true),
        (&ill, "pdf", "title", "\"A proof\"", true),
        (&ill, "pdf", "language", "\"de\"", true),
        (
            &ill,
            "pdf",
            "date",
            "{\"year\":2027,\"month\":1,\"day\":2,\"hour\":3,\"minute\":4,\"second\":5}",
            true,
        ),
        (&ill, "svg", "description", "false", true),
        (&["A |- A"], "rocq", "form", "\"standalone\"", true),
        (&["A |- A"], "rocq", "lemma", "\"identity\"", true),
        (
            &["--standalone", "A |- A"],
            "rocq",
            "prelude",
            "\"Require Import x.\"",
            true,
        ),
    ];
    let run = |args: &[&str], format: &str, style: &[&str]| {
        let out = scratch(&format!("styled.{format}"));
        let _ = std::fs::remove_file(&out);
        let format = if format == "text" { "text" } else { format };
        let all = [
            &["prove", "--format", format, "-o", out.to_str().unwrap()],
            style,
            args,
        ]
        .concat();
        let (status, _, err) = linlog(&all, "");
        (
            status,
            std::fs::read(&out).unwrap_or_else(|_| err.into_bytes()),
        )
    };
    for (args, format, key, value, shows) in rows {
        let file = scratch("style.json");
        std::fs::write(&file, format!("{{\"{format}\":{{\"{key}\":{value}}}}}")).unwrap();
        let plain: String = match serde_json::from_str::<serde_json::Value>(value).unwrap() {
            serde_json::Value::String(text) => text,
            other => other.to_string(),
        };
        let flag = format!("{key}={plain}");
        let by_flag = run(args, format, &["--style", &flag]);
        let by_file = run(args, format, &["--style-file", file.to_str().unwrap()]);
        let default = run(args, format, &[]);
        assert_eq!(by_flag, by_file, "{format}.{key}");
        assert_eq!(by_flag != default, *shows, "{format}.{key}");
    }
    // In a session the key names its format, and the open goal's shape
    // is an option of every drawn format.
    for (format, show) in [
        ("text", "show"),
        ("latex", "show --latex"),
        ("typst", "show --typst"),
        ("svg", "show --svg"),
    ] {
        let commands = format!("apply 0 1 -oL 0\n{show}\nquit\n");
        let flag = format!("{format}.open=dashed");
        let file = scratch("open.json");
        std::fs::write(&file, format!("{{\"{format}\":{{\"open\":\"dashed\"}}}}")).unwrap();
        let session = |style: &[&str]| {
            let args = [&["interact", "-i"], style, &["A, A -o B |- B"]].concat();
            linlog(&args, &commands).1
        };
        let by_flag = session(&["--style", &flag]);
        assert_eq!(
            by_flag,
            session(&["--style-file", file.to_str().unwrap()]),
            "{format}"
        );
        assert_ne!(by_flag, session(&[]), "{format}");
    }
    let (status, _, err) = linlog(&["interact", "--style", "labels=off", "A |- A"], "quit\n");
    assert_eq!(status, 2);
    assert!(err.contains("name the format"), "{err}");
}

/// `--abbreviate` cuts the sequent of `check`'s verdict line and names its
/// formulas, `--quiet` included; `--no-verdict` leaves the line out.
#[test]
fn verdict_lines() {
    let (_, json, _) = linlog(&["prove", "--format", "json", "A, B, C |- A * B * C"], "");
    let (status, out, _) = linlog(&["check", "--quiet", "--abbreviate", "8"], &json);
    assert_eq!(status, 0);
    assert_eq!(out, "valid proof of ⊢ ~A, ~B… (4 formulas) (classical)\n");
    let (status, out, _) = linlog(
        &["prove", "--format", "latex", "--no-verdict", "A |- A"],
        "",
    );
    assert_eq!(status, 0);
    assert!(out.starts_with("\\begin{prooftree}\n"), "{out}");
}

/// `--compact` draws a run of weakenings as one starred inference, and
/// never in a certificate.
#[test]
fn compact_runs() {
    let input = "!A, !B, !C |- 1";
    let (status, out, _) = linlog(&["prove", "-i", "--compact", "always", input], "");
    assert_eq!(status, 0);
    assert!(out.contains("!w*"), "{out}");
    let (_, out, _) = linlog(&["prove", "-i", "--compact", "never", input], "");
    assert!(!out.contains('*'), "{out}");
    let (status, out, _) = linlog(
        &[
            "prove",
            "-i",
            "--compact",
            "always",
            "--format",
            "rocq",
            input,
        ],
        "",
    );
    assert_eq!(status, 0);
    assert!(out.contains("wk_r_ext"), "{out}");
}

/// A batch walks a directory in sorted order, links followed without
/// loops, for the files its format takes; reads a NUL-separated list and
/// problem lines; answers one line per entry in order, a malformed or
/// missing entry being that entry's error; ends with the worst status;
/// and answers the same in a child per entry.
#[test]
fn batch_inputs_and_exit_status() {
    let dir = scratch("batch");
    std::fs::create_dir_all(dir.join("b")).unwrap();
    std::fs::write(
        dir.join("b/x.p"),
        "fof(a, axiom, a).\nfof(c, conjecture, a).\n",
    )
    .unwrap();
    let (_, json, _) = linlog(&["seq", "json", "A |- B"], "");
    std::fs::write(dir.join("a.json"), json).unwrap();
    std::fs::write(dir.join("c.txt"), "not read").unwrap();
    #[cfg(unix)]
    let _ = std::os::unix::fs::symlink("..", dir.join("b/up"));
    let d = dir.display();
    let (status, out, _) = linlog(&["prove", "--file", &dir.to_string_lossy()], "");
    assert_eq!(
        out,
        format!(
            "{d}/a.json: unprovable (MLL, classical, net engine): ~A occurs 1 more time than A \
             in the one-sided sequent, so they cannot all meet in axioms\n\
             {d}/b/x.p: provable (MLL, classical, net engine)\n"
        )
    );
    assert_eq!(status, 1);
    let list = format!("{d}/b/x.p\0{d}/missing.p\0");
    let (status, out, _) = linlog(&["prove", "--files-from", "-", "--null", "-q"], &list);
    assert!(out.starts_with(&format!("{d}/b/x.p: provable")), "{out}");
    assert!(
        out.contains(&format!("{d}/missing.p: error: cannot read")),
        "{out}"
    );
    assert_eq!(status, 2);
    let problems = "one; intuitionistic; -; -; A, A -o B |- B\ntwo; nonsense; -; -; A |- A\n";
    for isolate in [&[][..], &["--isolate"]] {
        let args = [&["prove", "--input-format", "problems"][..], isolate].concat();
        let (status, out, _) = linlog(&args, problems);
        assert_eq!(
            out,
            "one: provable (IMLL, intuitionistic, net engine)\ntwo: error: unknown mode \
             `nonsense`: classical, affine, mix, affine-mix, intuitionistic or \
             intuitionistic-affine\n"
        );
        assert_eq!(status, 2);
    }
}

/// A child of `--isolate` decides the entry it is given, however the
/// command's flags are spelt: `-qf DIR` bundles the batch's input with
/// another flag.
#[test]
fn isolated_entries_with_bundled_flags() {
    let dir = scratch("bundled");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("a.txt"), "A |- A").unwrap();
    std::fs::write(dir.join("b.txt"), "A |- A * A").unwrap();
    let d = dir.to_string_lossy();
    let common = ["prove", "--deterministic", "--workers", "1"];
    let flags = ["--input-format", "text", "-qf", &d];
    let (status, out, _) = linlog(&[&common[..], &flags].concat(), "");
    let isolated = linlog(&[&common[..], &["--isolate"], &flags].concat(), "");
    assert_eq!((isolated.0, &isolated.1), (status, &out));
    assert_eq!(status, 1);
    assert!(out.contains(&format!("{d}/b.txt: unprovable")), "{out}");
}

/// A sequent typed as the argument is refused beside a batch's input,
/// which would otherwise decide its own entries, or none, and leave the
/// typed sequent unanswered.
#[test]
fn a_typed_sequent_is_no_batch_entry() {
    let list = scratch("typed.list");
    std::fs::write(&list, "").unwrap();
    for batch in [
        &["--input-format", "lines"][..],
        &["--files-from", &list.to_string_lossy()],
    ] {
        let (status, out, err) = linlog(&[&["prove", "A |- B"][..], batch].concat(), "");
        assert_eq!((status, out.as_str()), (2, ""), "{batch:?}");
        assert!(err.contains("not read in a batch"), "{err}");
    }
}

/// A batch that holds no entry is an error, not a batch of verdicts all
/// proved: a directory of files its format does not take, an input of
/// comments.
#[test]
fn an_empty_batch_is_an_error() {
    let dir = scratch("none");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("c.txt"), "A |- A").unwrap();
    for (args, input) in [
        (&["prove", "--file", &dir.to_string_lossy()][..], ""),
        (&["prove", "--input-format", "lines"], "# a comment\n\n"),
    ] {
        let (status, out, err) = linlog(args, input);
        assert_eq!((status, out.as_str()), (2, ""), "{args:?}");
        assert!(err.contains("the batch holds no sequent"), "{err}");
    }
}

/// An interrupted batch is unknown at best, whatever it decided before:
/// the interrupt ends its input, whose rest is never answered.
#[cfg(unix)]
#[test]
fn an_interrupted_batch_is_unknown() {
    use std::io::{BufRead, BufReader};
    let mut child = Command::new(env!("CARGO_BIN_EXE_linlog"))
        .args(["prove", "--input-format", "lines"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    stdin.write_all(b"a: A |- A\n").unwrap();
    let mut line = String::new();
    BufReader::new(child.stdout.as_mut().unwrap())
        .read_line(&mut line)
        .unwrap();
    assert!(line.starts_with("a: provable"), "{line}");
    let pid = child.id().to_string();
    assert!(
        Command::new("kill")
            .args(["-INT", &pid])
            .status()
            .unwrap()
            .success()
    );
    // The handler sets the flag on a thread of its own; the end of the
    // input then finds it set.
    std::thread::sleep(std::time::Duration::from_millis(300));
    drop(stdin);
    let out = child.wait_with_output().unwrap();
    assert_eq!(out.status.code(), Some(3));
    assert!(String::from_utf8_lossy(&out.stderr).contains("interrupted"));
}

/// `--logic` decides ordinary logic through the translation it names:
/// the exit statuses are the verdicts', the derivation is read back as LK
/// or LJ, `--linear` shows the image's proof instead, a `.p` file is a
/// TPTP problem, `seq print` prints the image, and the linear connectives
/// and a translation into the wrong logic are refused.
#[test]
fn ordinary_logic() {
    let decide = |args: &[&str]| {
        let mut all = vec!["prove", "--deterministic", "--copies", "6"];
        all.extend(args);
        linlog(&all, "")
    };
    let (status, out, _) = decide(&["--logic", "classical", "~~a -> a"]);
    assert_eq!(status, 0);
    assert!(out.starts_with("valid (classical logic by the affine translation into affine MALL"));
    assert!(out.contains("¬R") && out.contains("→R"), "{out}");
    let (status, out, _) = decide(&["--logic", "intuitionistic", "~~a -> a"]);
    assert_eq!(status, 1);
    assert!(out.starts_with("not valid (intuitionistic logic by the cbn translation into ILL"));
    let (status, _, _) = decide(&["--logic", "intuitionistic", "--quiet", "a |- b"]);
    assert_eq!(status, 1);
    let (status, out, _) = decide(&[
        "--logic",
        "minimal",
        "--translation",
        "cbv",
        "a, a -> b |- b",
    ]);
    assert_eq!(status, 0);
    assert!(out.contains("→L") && out.contains("a, a → b ⊢ b"), "{out}");
    let (_, out, _) = decide(&["--logic", "intuitionistic", "--linear", "a, a -> b |- b"]);
    assert!(out.contains("⊸L") && !out.contains("→L"), "{out}");

    let dir = std::env::temp_dir().join(format!("linlog-ordinary-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("SYN.p");
    std::fs::write(
        &file,
        "fof(a, axiom, p => q).\nfof(c, conjecture, ~q => ~p).\n",
    )
    .unwrap();
    let (status, _, _) = decide(&[
        "--logic",
        "intuitionistic",
        "--quiet",
        "--file",
        file.to_str().unwrap(),
    ]);
    assert_eq!(status, 0);
    std::fs::remove_dir_all(&dir).unwrap();

    let (status, out, _) = linlog(
        &["seq", "print", "--logic", "intuitionistic", "a -> b |- b"],
        "",
    );
    assert_eq!((status, out.trim_end()), (0, "!(!a ⊸ b) ⊢ b"));
    for refused in [
        &["--logic", "classical", "a & b"][..],
        &["--logic", "classical", "--translation", "cbn", "a"],
        &["--logic", "intuitionistic", "|- a, b"],
        &["--logic", "intuitionistic", "-i", "a"],
    ] {
        assert_eq!(decide(refused).0, 2, "{refused:?}");
    }
}
