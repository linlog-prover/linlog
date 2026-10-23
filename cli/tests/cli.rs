// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The `linlog` binary end to end: verdicts, exit statuses, the JSON round
//! trip from `prove` to `check`, the interactive session, the `seq`
//! commands and the help text.

use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

mod common;
use common::timeless;

/// Runs `linlog` with the arguments and `stdin` as standard input, and
/// returns the exit status, standard output and standard error.
///
/// A machine of fewer threads than a call's `--jobs` adds a note that it
/// takes fewer, which is left out of standard error here
/// (`jobs_are_bounded` pins it).
fn linlog(args: &[&str], stdin: &str) -> (i32, String, String) {
    let (status, out, err) = linlog_as_is(args, stdin);
    let err: String = err
        .split_inclusive('\n')
        .filter(|line| !line.starts_with("note: --jobs "))
        .collect();
    (status, out, err)
}

/// Runs `linlog` as [`linlog`] does, and returns its standard error as
/// it is.
fn linlog_as_is(args: &[&str], stdin: &str) -> (i32, String, String) {
    let out = common::output(Command::new(env!("CARGO_BIN_EXE_linlog")).args(args), stdin);
    (
        out.status.code().unwrap(),
        String::from_utf8(out.stdout).unwrap(),
        String::from_utf8(out.stderr).unwrap(),
    )
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
/// status 3, on one thread and on a pool, which the classical refutation
/// after it then answers by default; and it counts from the start of
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
        // The search alone: every assignment falsifies the sequent.
        let args = [
            &["prove", "-a", "--timeout", "200ms", "--classical-work", "0"],
            threads,
            &[&sequent],
        ]
        .concat();
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
    // By default the classical refutation answers the search the time
    // limit ended, after it.
    let (status, out, _) = linlog(&["prove", "-a", "--timeout", "200ms", &sequent], "");
    assert_eq!(status, 1, "{out}");
    assert!(
        out.starts_with(
            "unprovable (MALL, classical affine, focus engine): the classical reading is false \
             where every atom is false"
        ),
        "{out}"
    );

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

/// A search that holds more than `--memory-limit` allows answers
/// "unknown" with the limit and the flag, exit status 3, and so does a
/// proof whose check would hold more, and a sequent
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

    // A proof found within the bound whose check would need more: a Horn
    // counter of 3 000 tokens, whose firing sequence the search holds
    // beside the sequent under bounds from about 920 KiB, and the check of
    // its proof beside the sequent only from about 1 040 KiB.
    let counter = format!("!(a -o a * b), a |- a * {}", vec!["b"; 3000].join(" * "));
    let (status, out, err) = linlog(&["prove", "-i", "--memory-limit", "960KiB", &counter], "");
    assert_eq!(
        (status, timeless(&out), err),
        (
            3,
            "unknown (IMELL, intuitionistic, horn engine): a proof was found, but its check \
             would hold more than the memory limit of 960 KiB after …; raise it with \
             --memory-limit SIZE\n"
                .to_owned(),
            String::new()
        )
    );

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
    assert_eq!(
        (status, err.as_str()),
        (
            2,
            "error: the sequent has more subformula occurrences than the limit of 2; raise it \
             with --occurrence-limit\n"
        )
    );
    assert_eq!(
        linlog(
            &["seq", "fragment", "--occurrence-limit", "none", "A |- A"],
            ""
        ),
        (0, "MLL\n".to_owned(), String::new())
    );
}

/// `check` names `--occurrence-limit` only where that limit refused the
/// proof file: a sequent of more occurrences than a forest can number,
/// 32 doublings of one atom, is refused with no advice under `none`,
/// since no setting lifts that.
#[test]
fn check_advises_only_the_limit_that_refused() {
    let doublings: Vec<String> = (0..32).map(|i| format!(r#"{{"⊗":[{i},{i}]}}"#)).collect();
    let proof = format!(
        r#"{{"version":1,"sequent":{{"terms":[{{"V":0}},{}],"roots":[32],"atoms":["A"]}},"nodes":[]}}"#,
        doublings.join(",")
    );
    let (status, _, err) = linlog(&["check", "--occurrence-limit", "none"], &proof);
    assert_eq!(status, 2, "{err}");
    assert!(!err.contains("--occurrence-limit"), "{err}");
    let (status, _, err) = linlog(&["check"], &proof);
    assert_eq!(status, 2, "{err}");
    assert!(
        err.ends_with("; raise it with --occurrence-limit\n"),
        "{err}"
    );
}

/// `--jobs` beyond what the machine runs at once is taken as that many,
/// and standard error says so; the verdict and the status are the
/// search's.
#[test]
fn jobs_are_bounded() {
    let (status, out, err) = linlog_as_is(&["prove", "--quiet", "--jobs", "100000", "A |- A"], "");
    assert_eq!(
        (status, out.as_str()),
        (0, "provable (MLL, classical, focus engine)\n")
    );
    assert!(
        err.starts_with(
            "note: --jobs 100000 is more than this machine runs at once; a search uses "
        ),
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
    let verdict = "provable (MLL, classical, focus engine)\n";
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
            "provable (MLL, classical, focus engine)\nthe drawing is not rendered: the PDF is \
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
        "provable (MLL, classical, focus engine)\n\
         ─────── ax   ─────── ax\n\
         ⊢ ~A, A      ⊢ ~B, B\n\
         ──────────────────── ⊗\n\
         \x20 ⊢ ~A, A ⊗ ~B, B\n"
    );

    for (args, status, line) in [
        (
            &["prove", "-q", "--mix", "|- A par B, ~A, ~B"][..],
            0,
            "provable (MLL, classical with Mix, focus engine)",
        ),
        (
            &["prove", "|- A par B, ~A, ~B"],
            1,
            "unprovable (MLL, classical, focus engine): the count equation fails: a provable \
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
            "--engine",
            "net",
            "|- a * b, ~a par ~b",
        ],
        "",
    );
    assert_eq!(status, 0);
    assert!(
        out.starts_with(
            "provable (MLL, classical, net engine)\nliterals chosen: 2\nlinks tried: 2\n\
             exact tests run: 2\nunits of work: 2\ntime: "
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
        "provable (IMLL, intuitionistic, two-sided engine)\n\
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
            "unprovable (IMLL, intuitionistic, two-sided engine): ~A occurs 1 more time than A in the \
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
            "the additive engine decides a sequent of two additive-only formulas, not one of MLL",
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
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("cli-session");
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
            "goal 1: provable (MLL, classical, focus engine)\n\
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
         goal 1: provable (IMLL, intuitionistic, two-sided engine)\n\
         goal 2: provable (IMLL, intuitionistic, two-sided engine)\n\
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

/// A proof of the Lambek calculus with an empty antecedent passes `check`
/// only where empty antecedents are allowed, and without them no net of
/// it is drawn as correct: the verdict is the checker's, with its rule.
#[test]
fn check_keeps_the_lambek_calculus_s_antecedents() {
    let sequent = "e |- e * (a \\ a)";
    let (status, json, _) = linlog(
        &[
            "prove",
            "--lambek",
            "--empty-antecedents",
            "--format",
            "json",
            sequent,
        ],
        "",
    );
    assert_eq!(status, 0);
    let (status, out, _) = linlog(&["check", "--lambek", "--empty-antecedents", "-q"], &json);
    assert_eq!(
        (status, out.as_str()),
        (
            0,
            "valid proof of e ⊢ e ⊗ (a \\ a) (Lambek with empty antecedents)\n"
        )
    );
    for flags in [&["check", "--lambek"][..], &["check", "--lambek", "--net"]] {
        let (status, out, _) = linlog(flags, &json);
        assert_eq!(status, 1, "{flags:?}");
        assert!(
            out.starts_with("invalid proof of e ⊢ e ⊗ (a \\ a) (Lambek): ")
                && out.ends_with(
                    "the sequent has no formula left of ⊢, which the Lambek calculus without \
                     empty antecedents forbids\n"
                ),
            "{out}"
        );
    }
    let (status, _, _) = linlog(&["prove", "--lambek", sequent], "");
    assert_eq!(status, 1);
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
        format!("provable (MLL, classical, focus engine)\n{expected}")
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

/// A proof with exponentials gets its net with boxes wherever `--net`
/// shows an MLL net: its text form after the verdict line, for `prove` and
/// for `check`; its document with `--format json`, which the library reads
/// back as a proof net; its drawing, each box a rectangle, as SVG, PNG and
/// PDF. Intuitionistic mode has no such nets and is refused before the
/// search.
#[test]
fn mell_nets() {
    let sequent = "!a, !b |- !a";
    let expected = "⊢ ?~a, ?~b, !a\n\
                    ?~a[0]: ~a[3] in !a[2]\n\
                    ?~b[1]: no copy, hangs on a[4]\n\
                    ~a[3] — a[4]\n\
                    proof net\n";
    let (status, out, _) = linlog(&["prove", "--net", sequent], "");
    assert_eq!(status, 0);
    assert_eq!(
        out,
        format!("provable (MELL, classical, focus engine)\n{expected}")
    );
    let (_, proof, _) = linlog(&["prove", "--format", "json", sequent], "");
    let (status, out, _) = linlog(&["check", "--net"], &proof);
    assert_eq!(status, 0);
    assert!(out.ends_with(expected), "{out}");

    for args in [
        &["prove", "--net", "--format", "json", sequent][..],
        &["check", "--net", "--format", "json"],
    ] {
        let (status, json, _) = linlog(args, &proof);
        assert_eq!(status, 0, "{args:?}");
        assert!(json.starts_with(r#"{"version":4,"#), "{json}");
        let mut document = serde_json::Deserializer::from_str(&json);
        let net: linlog::MellStructure =
            linlog::wire::upgrade(&mut document, &linlog::Limits::default()).unwrap();
        assert_eq!(net.is_correct(|_| false), Ok(()));
        assert_eq!(format!("{net}\n"), expected);
    }

    let (status, svg, _) = linlog(&["prove", "--net", "--format", "svg", sequent], "");
    assert_eq!(status, 0);
    assert!(svg.contains(r#"<path id="b2" d="M"#), "{svg}");
    for (name, magic) in [("boxes.png", &b"\x89PNG"[..]), ("boxes.pdf", b"%PDF")] {
        let path = scratch(name);
        let (status, _, _) = linlog(
            &["prove", "--net", "-o", path.to_str().unwrap(), sequent],
            "",
        );
        assert_eq!(status, 0, "{name}");
        assert!(std::fs::read(&path).unwrap().starts_with(magic), "{name}");
    }

    let (status, out, err) = linlog(&["prove", "--net", "-i", "!a |- !a"], "");
    assert_eq!((status, out.as_str()), (2, ""));
    assert!(
        err.contains("proof nets with boxes exist for MELL"),
        "{err}"
    );
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
        "% provable (MLL, classical, focus engine)\n\
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
/// `--standalone` adds the import; a proof with Mix has a certificate for
/// linlog's library, which the mode chooses, and none for NanoYalla.
#[test]
fn rocq_format() {
    let (status, out, _) = linlog(&["prove", "--format", "rocq", "A |- A"], "");
    assert_eq!(status, 0);
    assert_eq!(
        out,
        "(* provable (MLL, classical, focus engine) *)\n\
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
    let mix = ["prove", "--mix", "--format", "rocq", "A, B |- A, B"];
    let (status, out, _) = linlog(&mix, "");
    assert_eq!(status, 0);
    assert!(
        out.contains(
            "Lemma certificate : ll (atom := string) {| mix := true; weakening := false |}"
        ),
        "{out}"
    );
    let (status, out, err) = linlog(&[&mix[..], &["--kernel", "nanoyalla"]].concat(), "");
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
        out.starts_with("<!-- provable (MLL, classical, focus engine) -->\n<svg ")
            && out.ends_with("</svg>\n"),
        "{out}"
    );
    let (_, json, _) = linlog(&["prove", "--format", "json", "A * B |- B * A"], "");
    let (status, out, _) = linlog(&["check", "--net", "--format", "svg"], &json);
    assert_eq!(status, 0);
    assert!(out.contains("<g id=\"o0\">"), "{out}");
    let bounds = [
        "--copies",
        "0",
        "--forward-copies",
        "0",
        "--classical-work",
        "0",
    ];
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
    let printed = linlog(&["seq", "print", "--input-format", "json"], &json);
    assert_eq!(printed, (0, "⊢ ~A, A ⊗ ~B, B\n".into(), String::new()));
    let fragment = linlog(&["seq", "fragment", "--file", "-"], "A & B |- 1");
    assert_eq!(fragment, (0, "MALL\n".into(), String::new()));
}

/// Returns the path of the file `name` in a directory of these tests
/// under the target's temporary directory, apart from the session test's,
/// which removes its own.
fn scratch(name: &str) -> std::path::PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("cli-files");
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
        err.contains("--net writes the formats text, json, svg, png and pdf"),
        "{err}"
    );
}

/// `--font` sets a drawing in one of the math fonts the command embeds:
/// the SVG asks for its family and is laid out in its advances, as
/// `--style svg.font=NAME` sets it, and the PNG is rendered with its file.
#[test]
fn fonts() {
    let svg = |args: &[&str]| {
        let (status, out, err) = linlog(
            &[&["prove", "--format", "svg"], args, &["A * B |- B * A"]].concat(),
            "",
        );
        assert_eq!(status, 0, "{err}");
        out
    };
    let euler = svg(&[]);
    assert!(euler.contains("font-family=\"'Euler Math', 'Neo Euler', serif\""));
    assert_eq!(svg(&["--font", "euler"]), euler);
    for (name, family) in [
        ("stix-two", "'STIX Two Math', serif"),
        ("garamond", "'Garamond-Math', serif"),
        ("asana", "'Asana Math', serif"),
        ("new-computer-modern", "'NewComputerModernMath', serif"),
    ] {
        let set = svg(&["--font", name]);
        assert!(set.contains(&format!("font-family=\"{family}\"")), "{set}");
        assert_eq!(svg(&["--style", &format!("svg.font={name}")]), set);
        let laid = |text: &str| {
            let lines = text.lines().filter(|l| l.contains("textLength"));
            lines.map(str::to_owned).collect::<Vec<_>>()
        };
        assert_ne!(laid(&set), laid(&euler), "{name}");
    }
    let png = |args: &[&str]| {
        let file = scratch(&format!("font{}.png", args.len()));
        let path = file.to_str().unwrap();
        let (status, _, err) = linlog(
            &[&["prove", "-o", path], args, &["A * B |- B * A"]].concat(),
            "",
        );
        assert_eq!(status, 0, "{err}");
        std::fs::read(&file).unwrap()
    };
    assert_ne!(png(&["--font", "stix-two"]), png(&[]));
    let (status, _, err) = linlog(&["prove", "--font", "stix", "A |- A"], "");
    assert_eq!(status, 2, "{err}");
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
        (&ill, "svg", "font", "\"stix-two\"", true),
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
        (
            &["--net", "-i", "A -o B, B -o C |- A -o C"],
            "svg",
            "essential",
            "true",
            true,
        ),
        (
            &[
                "--style",
                "svg.essential=true",
                "--net",
                "-i",
                "A -o B, B -o C |- A -o C",
            ],
            "svg",
            "input",
            "\"navy\"",
            true,
        ),
        (
            &[
                "--style",
                "svg.essential=true",
                "--net",
                "-i",
                "A -o B, B -o C |- A -o C",
            ],
            "svg",
            "dominator",
            "\"navy\"",
            true,
        ),
        (
            &["--net", "A * B |- B * A"],
            "svg",
            "net",
            "\"compact\"",
            true,
        ),
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
        (&["A |- A"], "rocq", "kernel", "\"linlog\"", true),
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
/// never in a certificate of either kernel.
#[test]
fn compact_runs() {
    let input = "!A, !B, !C |- 1";
    let (status, out, _) = linlog(&["prove", "-i", "--compact", "always", input], "");
    assert_eq!(status, 0);
    assert!(out.contains("!w*"), "{out}");
    let (_, out, _) = linlog(&["prove", "-i", "--compact", "never", input], "");
    assert!(!out.contains('*'), "{out}");
    let rocq = ["prove", "-i", "--compact", "always", "--format", "rocq"];
    let (status, out, _) = linlog(&[&rocq[..], &["--kernel", "nanoyalla", input]].concat(), "");
    assert_eq!(status, 0);
    assert!(out.contains("wk_r_ext"), "{out}");
    // linlog's library, the mode's kernel, has a term per weakening.
    let (status, out, _) = linlog(&[&rocq[..], &[input]].concat(), "");
    assert_eq!(status, 0);
    assert_eq!(out.matches("(IWk ").count(), 3, "{out}");
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
            "{d}/a.json: unprovable (MLL, classical, focus engine): ~A occurs 1 more time than A \
             in the one-sided sequent, so they cannot all meet in axioms\n\
             {d}/b/x.p: provable (MLL, classical, focus engine)\n"
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
            "one: provable (IMLL, intuitionistic, two-sided engine)\ntwo: error: unknown mode \
             `nonsense`: classical, affine, mix, affine-mix, intuitionistic, \
             intuitionistic-affine, cyclic, lambek or lambek-star\n"
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

/// `--prelude` goes into a standalone certificate, and is refused where
/// it would do nothing.
#[test]
fn a_prelude_needs_a_standalone_certificate() {
    let prelude = ["prove", "--format", "rocq", "--prelude", "(* mine *)"];
    let (status, out, err) = linlog(&[&prelude[..], &["A |- A"]].concat(), "");
    assert_eq!((status, out.as_str()), (2, ""));
    assert!(
        err.contains("--prelude goes into a standalone certificate"),
        "{err}"
    );
    let (status, out, _) = linlog(&[&prelude[..], &["--standalone", "A |- A"]].concat(), "");
    assert_eq!(status, 0);
    assert!(out.contains("(* mine *)"), "{out}");
}

/// An entry whose name starts with `-` gets its verdict under
/// `--isolate` too: the child reads the name as a value, not a flag.
#[test]
fn isolated_entries_named_like_flags() {
    let common = [
        "prove",
        "--deterministic",
        "--workers",
        "1",
        "--input-format",
        "lines",
    ];
    let input = "-x: A |- A\n--y: A |- B\n";
    let (status, out, _) = linlog(&common, input);
    let isolated = linlog(&[&common[..], &["--isolate"]].concat(), input);
    assert_eq!((isolated.0, &isolated.1), (status, &out));
    assert_eq!(status, 1);
    assert!(out.starts_with("-x: provable"), "{out}");
    assert!(out.contains("--y: unprovable"), "{out}");
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

/// `load` resumes a session of the same sequent and mode only: the exit
/// status answers for the session's own question.
#[test]
fn load_keeps_the_question() {
    let saved = scratch("affine-session.json");
    let saved = saved.to_str().unwrap();
    let finished = format!("close\nsave {saved}\n");
    let (status, _, _) = linlog(&["interact", "-a", "a, b |- a"], &finished);
    assert_eq!(status, 0);
    let load = format!("load {saved}\n");
    let (status, _, _) = linlog(&["interact", "-a", "a, b |- a"], &load);
    assert_eq!(status, 0);
    for (args, why) in [
        (&["interact", "|- 1, 1"][..], "a session of another sequent"),
        (
            &["interact", "a, b |- a"],
            "in classical affine mode, this one is in classical",
        ),
    ] {
        let (status, out, _) = linlog(args, &load);
        assert_eq!(status, 1, "{args:?}");
        assert!(out.starts_with("error: ") && out.contains(why), "{out}");
    }
}

/// A `.spec` file is a coverability question, decided in affine mode
/// without `--affine`, alone and in a batch: here a marking covers the
/// target that no marking reaches exactly.
#[test]
fn a_spec_file_is_affine() {
    let dir = scratch("cover");
    std::fs::create_dir_all(&dir).unwrap();
    let spec = dir.join("cov.spec");
    std::fs::write(
        &spec,
        "vars x y\nrules\n  x >= 1 -> x' = x - 1, y' = y + 1;\ninit x = 2\ntarget y >= 1\n",
    )
    .unwrap();
    let (status, out, _) = linlog(&["prove", "-q", "--file", spec.to_str().unwrap()], "");
    assert_eq!(
        (status, out.as_str()),
        (0, "provable (MELL, classical affine, horn engine)\n")
    );
    let (status, out, _) = linlog(
        &[
            "prove",
            "-q",
            "--file",
            dir.to_str().unwrap(),
            "--input-format",
            "spec",
        ],
        "",
    );
    assert_eq!(status, 0, "{out}");
    assert!(
        out.ends_with("cov.spec: provable (MELL, classical affine, horn engine)\n"),
        "{out}"
    );
}

/// A JSON Lines record is read key by key: a line with a record's keys
/// and no `sequent`, a key written twice and a sequent with a key written
/// twice are that line's errors, never a sequent read in another mode.
#[test]
fn json_lines_records_are_whole() {
    let lines = r#"{"name":"flat","mode":"affine","terms":[{"D":0},{"D":1},{"V":0}],"roots":[0,1,2],"atoms":["A","B"]}
{"name":"rec","mode":"affine","sequent":{"terms":[{"D":0},{"D":1},{"V":0}],"roots":[0,1,2],"atoms":["A","B"]}}
{"name":"dup","mode":"affine","mode":"classical","sequent":"A, B |- A"}
{"name":"ids","sequent":{"terms":[{"D":0},{"V":0}],"roots":[0,1],"roots":[1],"atoms":["A"]}}
"#;
    let (status, out, _) = linlog(
        &["prove", "--deterministic", "--input-format", "jsonl"],
        lines,
    );
    let out: Vec<&str> = out.lines().collect();
    assert_eq!(status, 2);
    assert_eq!(out.len(), 4, "{out:?}");
    assert!(
        out[0].starts_with("-:1: error: a record's keys are"),
        "{}",
        out[0]
    );
    assert_eq!(
        out[1],
        "rec: provable (MLL, classical affine, focus engine)"
    );
    assert_eq!(out[2], "-:3: error: the record names `mode` twice");
    assert!(
        out[3].starts_with("ids: error: not a sequent in JSON: duplicate field"),
        "{}",
        out[3]
    );
}

/// A line named `/` or `.` is named by its place, so that its file lies
/// inside `--output DIR`, never beside it.
#[test]
fn every_entry_file_lies_inside_the_directory() {
    let dir = scratch("inside");
    std::fs::create_dir_all(&dir).unwrap();
    let out = dir.join("out");
    let args = [
        "prove",
        "--input-format",
        "lines",
        "--format",
        "json",
        "--output",
    ];
    let (status, lines, _) = linlog(
        &[&args[..], &[out.to_str().unwrap()]].concat(),
        "/: A |- A\n.: A |- A\n",
    );
    assert_eq!(status, 0);
    assert!(
        lines.starts_with("-:1: provable") && lines.contains("\n-:2: provable"),
        "{lines}"
    );
    assert!(!dir.join("out.json").exists());
    assert!(out.join("-:1.json").is_file() && out.join("-:2.json").is_file());
}

/// A PNG's drawing is held as SVG text only within the memory bound: past
/// it the drawing is left out with a line that names the flag, before the
/// render's estimate is asked.
#[test]
fn a_drawing_is_held_within_the_memory_bound() {
    let atoms: Vec<String> = (0..60).map(|i| format!("a{i}")).collect();
    let sequent = format!("{} |- {}", atoms.join(", "), atoms.join(" * "));
    let png = scratch("held").with_extension("png");
    let (status, _, err) = linlog(
        &[
            "prove",
            "--deterministic",
            "--memory-limit",
            "256KiB",
            "--output",
            png.to_str().unwrap(),
            &sequent,
        ],
        "",
    );
    assert!(
        status == 0
            && err.contains("its SVG passes the memory limit of 256.0 KiB")
            && !png.exists(),
        "{err}"
    );
}

/// A reader that closes standard output keeps the verdict's exit status:
/// what is written after is dropped, where it once ended in an error, exit
/// status 2.
#[test]
fn a_closed_pipe_keeps_the_verdict() {
    for (sequent, status) in [("|- a * b * c * d * e, ~a, ~b, ~c, ~d, ~e", 0), ("|- a", 1)] {
        let mut child = Command::new(env!("CARGO_BIN_EXE_linlog"))
            .args(["prove", "--tree", "always", sequent])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        drop(child.stdout.take());
        let out = child.wait_with_output().unwrap();
        assert_eq!(
            out.status.code(),
            Some(status),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

/// `check` gives up at its limits with an unknown verdict, exit status 3,
/// which its JSON says too, its time limit counting the read; an outcome that holds no proof is refused
/// with what it is; an invalid proof is `invalid` in JSON with the error.
#[test]
fn check_s_verdicts() {
    let (_, proof, _) = linlog(
        &[
            "prove",
            "--format",
            "json",
            "a * b, c * d |- (a * b) * (c * d)",
        ],
        "",
    );
    let (status, out, _) = linlog(&["check", "--work-limit", "2"], &proof);
    assert!(
        status == 3 && out.starts_with("unknown: the proof of") && out.contains("--work-limit"),
        "{out}"
    );
    let (status, out, _) = linlog(&["check", "--work-limit", "2", "--format", "json"], &proof);
    let checked: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!((status, &checked["verdict"]), (3, &"unknown".into()));
    assert_eq!(checked["error"]["code"], "work_limit");
    // The time limit counts from the start: the proof is read within it.
    let (status, out, _) = linlog(&["check", "--timeout", "0ms", "--format", "json"], &proof);
    let checked: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!((status, &checked["verdict"]), (3, &"unknown".into()));
    assert!(
        checked["message"]
            .as_str()
            .unwrap()
            .contains("was reached while the proof was read"),
        "{out}"
    );
    let (_, classical, _) = linlog(&["prove", "--format", "json", "|- a, ~a"], "");
    let (status, out, _) = linlog(&["check", "-i", "--format", "json"], &classical);
    let checked: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!((status, &checked["verdict"]), (1, &"invalid".into()));
    let (_, outcome, _) = linlog(&["prove", "--format", "json", "a |- b"], "");
    let (status, _, err) = linlog(&["check"], &outcome);
    assert_eq!(
        (status, err.as_str()),
        (
            2,
            "error: the file is the outcome of a sequent the search found unprovable: it holds \
             no proof to check\n"
        )
    );
}

/// Every JSON Lines record of a batch has a name and a verdict: an entry
/// that could not be read is `error` with the error's message (and the
/// library's error where the error is the library's), one the batch's
/// time limit stopped is `unknown`; in text a name's control characters
/// and reordering marks are escaped, so it cannot read as another
/// entry's line; and two entries
/// of one file name in `--output DIR` are refused, the second an error.
#[test]
fn a_batch_s_records_names_and_files() {
    let (status, lines, _) = linlog(
        &["prove", "--input-format", "jsonl", "--format", "json"],
        "{\"name\": \"bad\", \"sequent\": \"A |-|- B\"}\n\
         {\"name\": \"big\", \"sequent\": {\"terms\": [{\"V\": 0}], \"roots\": [0, 0, 0], \"var_dict\": [\"a\"]}}\n",
    );
    assert_eq!(status, 2);
    let records: Vec<serde_json::Value> = lines
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(records.len(), 2, "{lines}");
    for record in &records {
        assert_eq!(record["verdict"], "error", "{record}");
        assert!(record["message"].is_string(), "{record}");
    }
    // The parse error is the command's line with a caret, and the
    // library's error object beside it.
    assert_eq!(records[0]["error"]["code"], "parse", "{}", records[0]);
    let (status, lines, _) = linlog(
        &[
            "prove",
            "--input-format",
            "lines",
            "--format",
            "json",
            "--batch-timeout",
            "0ms",
        ],
        "a: A |- A\n",
    );
    let record: serde_json::Value = serde_json::from_str(lines.trim()).unwrap();
    assert_eq!((status, &record["verdict"]), (3, &"unknown".into()));
    let (_, lines, _) = linlog(
        &["prove", "--input-format", "jsonl"],
        "{\"name\": \"x\\ny: proved\", \"sequent\": \"A |- A\"}\n",
    );
    assert!(lines.starts_with("x\\ny: proved: provable"), "{lines}");
    // A line past the memory limit is an error of its own, skipped without
    // being held, and a line that is no UTF-8 too; the batch goes on.
    let long = format!("a: A |- A\nb: {}A |- A\nc: A |- A\n", "A * ".repeat(600));
    let (status, lines, _) = linlog(
        &["prove", "--input-format", "lines", "--memory-limit", "2KiB"],
        &long,
    );
    assert_eq!(status, 2);
    assert_eq!(
        lines.lines().collect::<Vec<_>>(),
        [
            "a: provable (MLL, classical, focus engine)",
            "-:2: error: the line is longer than the memory limit of 2.0 KiB; raise it with \
             --memory-limit SIZE",
            "c: provable (MLL, classical, focus engine)",
        ]
    );
    // A mark that reorders the text after it is escaped too.
    let (_, lines, _) = linlog(
        &["prove", "--input-format", "jsonl"],
        "{\"name\": \"a\\u202eb\", \"sequent\": \"A |- A\"}\n",
    );
    assert!(lines.starts_with("a\\u{202e}b: provable"), "{lines}");
    let dir = scratch("collide");
    let (status, lines, _) = linlog(
        &[
            "prove",
            "--input-format",
            "lines",
            "--format",
            "json",
            "--output",
            dir.to_str().unwrap(),
        ],
        "x: A |- A\nx: B |- B\n",
    );
    assert_eq!(status, 2);
    assert!(
        lines.starts_with("x: provable") && lines.contains("\nx: error: its file"),
        "{lines}"
    );
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

    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("cli-ordinary");
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

/// The search's flags change what they say: `--memo-limit 0` switches the
/// memo off and 1 holds one entry, `--deterministic` wins over `--jobs`,
/// and `--no-check` is taken with the verdict unchanged.
#[test]
fn search_flags() {
    let sequent = "!(a*a -o b), !(b*b -o c), !(c*c -o d), a,a,a,a,a,a,a,a |- d";
    let stats = |extra: &[&str]| {
        let args = [
            &[
                "prove",
                "-q",
                "--deterministic",
                "--stats",
                "--engine",
                "focus",
            ],
            extra,
            &[sequent],
        ]
        .concat();
        let (status, out, _) = linlog(&args, "");
        assert_eq!(status, 0, "{out}");
        timeless(&out)
    };
    let memo = |out: &str| -> u64 {
        let line = out
            .lines()
            .find(|l| l.starts_with("memo entries at most: "))
            .unwrap();
        line["memo entries at most: ".len()..].parse().unwrap()
    };
    let default = stats(&[]);
    assert!(memo(&default) > 1, "{default}");
    assert_eq!(memo(&stats(&["--memo-limit", "0"])), 0);
    assert_eq!(memo(&stats(&["--memo-limit", "1"])), 1);
    assert_eq!(stats(&["--jobs", "8"]), default);
    assert_eq!(stats(&["--no-check"]), default);
    // The outcome says whether its proof was checked.
    for (flags, checked) in [
        (&["--format", "json"][..], true),
        (&["--format", "json", "--no-check"], false),
    ] {
        let (_, json, _) = linlog(&[&["prove"], flags, &["A |- A"]].concat(), "");
        let outcome: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(outcome["checked"], checked, "{flags:?}");
    }
}

/// Ctrl-C stops a single call's search with an unknown verdict, exit
/// status 3, and says so; in a batch it stops the entry searched and ends
/// the input, an entry read but not begun saying it was not searched.
#[test]
fn interrupt() {
    use std::io::{BufRead, BufReader};
    let literals: Vec<String> = (0..40).map(|i| format!("x{i}")).collect();
    let hard = format!(
        "|- p * q, 0 * (~p par ~p), 0 * (~q par ~q), {}",
        literals.join(", ")
    );
    let interrupt = |child: &std::process::Child| {
        std::thread::sleep(std::time::Duration::from_millis(300));
        let pid = child.id().to_string();
        assert!(
            Command::new("kill")
                .args(["-INT", &pid])
                .status()
                .unwrap()
                .success()
        );
    };
    let child = Command::new(env!("CARGO_BIN_EXE_linlog"))
        .args(["prove", "-a", "--timeout", "none", "--deterministic", &hard])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    interrupt(&child);
    let out = child.wait_with_output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(out.status.code(), Some(3), "{stdout}");
    assert!(
        stdout.starts_with("unknown (") && stdout.contains("interrupted after"),
        "{stdout}"
    );
    let mut child = Command::new(env!("CARGO_BIN_EXE_linlog"))
        .args([
            "prove",
            "-a",
            "--input-format",
            "lines",
            "--workers",
            "1",
            "--timeout",
            "none",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    writeln!(stdin, "hard: {hard}\nnext: A |- A").unwrap();
    drop(stdin);
    interrupt(&child);
    let mut lines = BufReader::new(child.stdout.take().unwrap()).lines();
    let first = lines.next().unwrap().unwrap();
    assert!(
        first.starts_with("hard: unknown") && first.contains("interrupted after"),
        "{first}"
    );
    // The interrupt ends the input: an entry never read gets no line, one
    // read and not begun says it was not searched, none is answered.
    // Whether `next` was read when the interrupt came is the reader
    // thread's race, so its line is there or not, and nothing else is.
    let rest: Vec<String> = lines.map(Result::unwrap).collect();
    assert!(
        rest.len() <= 1
            && rest
                .iter()
                .all(|line| line == "next: unknown: interrupted before it was searched"),
        "{rest:?}"
    );
    assert_eq!(child.wait().unwrap().code(), Some(3));
}

/// An `--output` file holds a whole output or is left as it was: a
/// derivation given up while it is built, after the search proved the
/// sequent, leaves an earlier run's file alone, and so does a verdict
/// without a derivation. The work limit gives up the derivation's check
/// where a time limit could not tell the search from the build.
#[test]
fn output_is_whole_or_untouched() {
    // 2^12 occurrences of A: the search's 8 191 units of work, the
    // derivation's check past the 20 000 the limit allows.
    let doublings: Vec<String> = (0..12).map(|i| format!(r#"{{"&":[{i},{i}]}}"#)).collect();
    let amp = format!(
        r#"{{"terms":[{{"V":0}},{},{{"D":0}}],"roots":[12,13],"atoms":["A"]}}"#,
        doublings.join(",")
    );
    let keep = Path::new(env!("CARGO_TARGET_TMPDIR")).join("cli-keep.tex");
    let given_up = [
        "prove",
        "--input-format",
        "json",
        "--deterministic",
        "--no-check",
        "--work-limit",
        "20000",
        "--derivation-limit",
        "none",
    ];
    for (args, stdin, status) in [
        (given_up.to_vec(), amp.as_str(), 0),
        (vec!["prove", "A |- B"], "", 1),
    ] {
        std::fs::write(&keep, "OLD").unwrap();
        let args = [&args[..], &["--output", keep.to_str().unwrap()]].concat();
        let (got, _, err) = linlog(&args, stdin);
        assert_eq!(got, status, "{err}");
        if status == 0 {
            assert!(err.contains("the derivation is not written"), "{err}");
        }
        assert_eq!(std::fs::read_to_string(&keep).unwrap(), "OLD");
    }
    // A directory is refused before the search, which it would lose.
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR"));
    let (status, _, err) = linlog(
        &["prove", "--output", directory.to_str().unwrap(), "A |- A"],
        "",
    );
    assert_eq!(status, 2);
    assert!(err.ends_with("it is a directory\n"), "{err}");
}

/// A batch's own flags: an unknown entry outranks an unprovable one in
/// the exit status; `--batch-timeout` answers the entries not begun as
/// unknown without searching them; `--cores across` and `within` and
/// `--batch-memory` give the verdicts single calls give; `--isolate`
/// answers a hard entry as a single call does.
#[test]
fn batch_flags() {
    let literals: Vec<String> = (0..40).map(|i| format!("x{i}")).collect();
    let hard = format!(
        "|- p * q, 0 * (~p par ~p), 0 * (~q par ~q), {}",
        literals.join(", ")
    );
    let lines = format!("hard: {hard}\nno: A |- B\n");
    // The search alone: every assignment falsifies the hard entry.
    let batch = |extra: &[&str], input: &str| {
        let args = [
            &[
                "prove",
                "-a",
                "--input-format",
                "lines",
                "--classical-work",
                "0",
            ],
            extra,
        ]
        .concat();
        linlog(&args, input)
    };
    let (status, out, err) = batch(&["--timeout", "100ms"], &lines);
    assert_eq!(status, 3, "{out}{err}");
    assert!(
        out.starts_with("hard: unknown") && out.contains("\nno: unprovable"),
        "{out}"
    );
    let (status, out, _) = batch(
        &[
            "--workers",
            "1",
            "--batch-timeout",
            "300ms",
            "--timeout",
            "none",
        ],
        &format!("hard1: {hard}\nhard2: {hard}\neasy: A |- A\n"),
    );
    assert_eq!(status, 3, "{out}");
    assert!(
        out.contains(
            "hard2: unknown: the batch's time limit of 300ms was reached before it was searched"
        ) && out.contains("easy: unknown: the batch's time limit"),
        "{out}"
    );
    // The entry the batch's limit stopped in its search names that limit.
    let hard1 = out.lines().find(|l| l.starts_with("hard1: ")).unwrap();
    assert!(
        hard1.contains("the batch's time limit of 300ms was reached")
            && hard1.contains("--batch-timeout DURATION"),
        "{out}"
    );
    let easy = "a: A |- A\nb: A |- B\n";
    for flags in [
        &["--cores", "across"][..],
        &["--cores", "within"],
        &["--batch-memory", "64MiB"],
    ] {
        let (status, out, _) = batch(&[flags, &["--timeout", "100ms"]].concat(), easy);
        assert_eq!(
            (status, timeless(&out)),
            (
                1,
                "a: provable (MLL, classical affine, focus engine)\nb: unprovable (MLL, classical \
                 affine, focus engine): the search was exhaustive\n"
                    .to_owned()
            ),
            "{flags:?}"
        );
    }
    let (status, out, _) = batch(
        &["--isolate", "--timeout", "100ms"],
        &format!("hard: {hard}\n"),
    );
    assert_eq!(status, 3, "{out}");
    assert!(
        out.starts_with("hard: unknown") && out.contains("the time limit of 100ms"),
        "{out}"
    );
    // A child of --isolate is stopped by the batch's limit as a search in
    // this process is, its entry unknown.
    let (status, out, _) = batch(
        &[
            "--isolate",
            "--workers",
            "1",
            "--timeout",
            "none",
            "--batch-timeout",
            "300ms",
        ],
        &format!(
            "hard: {hard}
"
        ),
    );
    assert_eq!(
        (status, out.as_str()),
        (
            3,
            "hard: unknown: the batch's time limit of 300ms was reached while it was searched\n"
        )
    );
}

/// A batch on standard input is a stream however the input is named: a
/// line is answered while the input stays open, so a program can ask and
/// wait for the answer.
#[test]
fn standard_input_is_answered_line_by_line() {
    use std::io::{BufRead, BufReader, Write};
    for named in [&[][..], &["--file", "-"]] {
        let mut child = Command::new(env!("CARGO_BIN_EXE_linlog"))
            .args([&["prove", "--input-format", "lines"][..], named].concat())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let mut input = child.stdin.take().unwrap();
        writeln!(input, "a: A |- A").unwrap();
        input.flush().unwrap();
        let mut answers = BufReader::new(child.stdout.take().unwrap());
        let mut line = String::new();
        answers.read_line(&mut line).unwrap();
        assert_eq!(
            line, "a: provable (MLL, classical, focus engine)\n",
            "{named:?}"
        );
        drop(input);
        assert_eq!(child.wait().unwrap().code(), Some(0));
    }
}

/// An `--output` that is a pipe gets the output once: the check that the
/// output can be made before the search opens only a file, since a pipe
/// opened and closed would end its reader before the output comes.
#[test]
fn an_output_pipe_gets_the_output() {
    let fifo = Path::new(env!("CARGO_TARGET_TMPDIR")).join("cli-fifo");
    let _ = std::fs::remove_file(&fifo);
    assert!(
        Command::new("mkfifo")
            .arg(&fifo)
            .status()
            .unwrap()
            .success()
    );
    let reader = {
        let fifo = fifo.clone();
        std::thread::spawn(move || std::fs::read_to_string(fifo).unwrap())
    };
    let (status, _, err) = linlog(&["prove", "--output", fifo.to_str().unwrap(), "A |- A"], "");
    assert_eq!(status, 0, "{err}");
    assert!(reader.join().unwrap().starts_with("provable (MLL"));
    std::fs::remove_file(&fifo).unwrap();
}
