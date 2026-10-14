// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Every long call of the public API ends when its stop fires or a bound
//! of its `Limits` is reached, and says so as a refusal, never as a fault
//! of its input.

#![cfg(all(
    feature = "parse",
    feature = "interactive",
    feature = "latex",
    feature = "rocq",
    feature = "svg",
    feature = "typst"
))]

use linlog::export::{latex, rocq, svg, typst};
use linlog::proofs::TextOptions;
use linlog::{
    CheckError, Error, ErrorKind, Forest, Interactive, Limits, Mode, Options, Phase, Proof, Reason,
    Refusal, Sequent, Verdict, ViewOptions, prove, prove_within,
};

/// `⊢ ~a0 ⅋ … ⅋ ~a(n-1), a0 ⊗ … ⊗ a(n-1)`, whose proof has `3n − 2`
/// nodes.
fn wide(n: usize) -> Sequent {
    let atoms: Vec<String> = (0..n).map(|i| format!("a{i}")).collect();
    let duals: Vec<String> = atoms.iter().map(|a| format!("~{a}")).collect();
    format!("|- {}, {}", duals.join(" par "), atoms.join(" * "))
        .parse()
        .unwrap()
}

/// The proof of [`wide`] at `n`.
fn proof(n: usize) -> Proof {
    match prove(&wide(n), Mode::CLASSICAL, &Options::default())
        .unwrap()
        .verdict
    {
        Verdict::Proved(proof) => *proof,
        verdict => panic!("provable: {verdict:?}"),
    }
}

/// Whether `error` is a refusal of the kind `kind`.
fn refused(error: &Error, kind: ErrorKind) -> bool {
    error.is_refusal() && error.kind() == kind
}

#[test]
fn the_readers_refuse_past_the_occurrences() {
    let limits = Limits::default().with_occurrences(Some(3));
    let sequent = Sequent::parse_within("|- a * b, ~a, ~b", &limits);
    assert!(matches!(
        sequent,
        Err(Error::Refused(Refusal::Occurrences { .. }))
    ));
    let forest = Forest::within(&wide(2), &limits);
    assert!(matches!(
        forest,
        Err(Error::Refused(Refusal::Occurrences { .. }))
    ));
    // A run of `!` or `?` is counted as it is read: the text is refused
    // before the parse error after the run, not held until its operand.
    for prefix in ["!", "?"] {
        let text = format!("|- {}A B", prefix.repeat(1000));
        assert!(matches!(
            Sequent::parse_within(&text, &limits),
            Err(Error::Refused(Refusal::Occurrences { .. }))
        ));
    }
    // A `!` or `?` pays its term when it is reduced: `!A, !B` is four
    // terms, read at a bound of four and refused at three.
    let bound = |n| Limits::default().with_occurrences(Some(n));
    let refused = |text: &str, n| {
        matches!(
            Sequent::parse_within(text, &bound(n)),
            Err(Error::Refused(Refusal::Occurrences { .. }))
        )
    };
    for text in ["|- !A, !B", "|- ?A, ?B"] {
        assert!(Sequent::parse_within(text, &bound(4)).is_ok(), "{text}");
        assert!(refused(text, 3), "{text}");
    }
    // A `(` or `~` waits as a mark until its `)` or its operand gives it
    // back: six of them one after another fit a bound of six, and two
    // open at once fit a bound of two where three do not.
    for text in [
        "|- (A), (B), (C), (D), (E), (F)",
        "|- ~A, ~B, ~C, ~D, ~E, ~F",
    ] {
        assert!(Sequent::parse_within(text, &bound(6)).is_ok(), "{text}");
    }
    // Marks given back, not merely not added: groups that each open more
    // marks than they hold terms fit a bound of their terms.
    for text in ["|- ((A)), ((B)), ((C))", "|- ~(~A), ~(~B), ~(~C)"] {
        assert!(Sequent::parse_within(text, &bound(3)).is_ok(), "{text}");
    }
    assert!(Sequent::parse_within("|- ((A))", &bound(2)).is_ok());
    assert!(refused("|- (((A)))", 2));
    // A run of `(`, or of `~(`, makes no term and is refused past the
    // bound all the same.
    for run in ["(", "~("] {
        let text = format!("|- {}A B", run.repeat(1000));
        assert!(matches!(
            Sequent::parse_within(&text, &limits),
            Err(Error::Refused(Refusal::Occurrences { .. }))
        ));
    }
    // Two `~` cancel as they are read, so a run of them is one at most.
    let even = format!("|- {}A", "~".repeat(1000));
    assert_eq!(
        Sequent::parse_within(&even, &limits).unwrap(),
        "|- A".parse().unwrap()
    );
}

#[test]
fn the_checker_stops_and_keeps_its_bounds() {
    // Past the 4 096 nodes between two polls of the stop.
    let proof = proof(1500);
    let mode = Mode::CLASSICAL;
    // The pass tells its stop what it holds by its count.
    let mut held = 0;
    proof
        .check_within(mode, &Limits::default(), |p| {
            held = held.max(p.held_bytes);
            false
        })
        .unwrap();
    assert!(held > 0);
    let stopped = proof.check_within(mode, &Limits::default(), |_| true);
    assert!(matches!(
        stopped,
        Err(CheckError::Refused(r)) if matches!(r.refusal, Refusal::Stopped { phase: Phase::Check, .. })
    ));
    let work = proof.check_within(mode, &Limits::default().with_work(Some(10)), |_| false);
    assert!(matches!(
        work,
        Err(CheckError::Refused(r)) if matches!(r.refusal, Refusal::Work { limit: 10, .. })
    ));
    let memory = proof.check_within(mode, &Limits::default().with_memory_bytes(Some(0)), |_| {
        false
    });
    assert!(matches!(
        memory,
        Err(CheckError::Refused(r)) if matches!(r.refusal, Refusal::Memory { .. })
    ));
    let size = proof.derivation_size(&ViewOptions::default(), &Limits::default(), |_| true);
    assert!(matches!(size, Err(CheckError::Refused(r)) if r.refusal.is_stop()));
}

#[test]
fn a_derivation_and_its_writers_stop() {
    let large = proof(1500);
    let view = ViewOptions::default();
    let built = large.derivation_within(&view, &Limits::default(), |_| true);
    assert!(refused(&built.unwrap_err(), ErrorKind::Stopped));
    let bounded = Limits::default().with_derivation_bytes(Some(1));
    let built = large.derivation_within(&view, &bounded, |_| false);
    assert!(matches!(built, Err(Error::Refused(Refusal::Output { .. }))));

    let small = proof(8);
    let derivation = small.derivation().unwrap();
    let stopped = |result: Result<(), Error>| {
        matches!(
            result,
            Err(Error::Refused(Refusal::Stopped {
                phase: Phase::Write,
                ..
            }))
        )
    };
    let mut out = String::new();
    assert!(stopped(derivation.write_text(
        &TextOptions::default(),
        &mut out,
        |_| true
    )));
    assert!(stopped(latex::write(
        &derivation,
        &latex::Options::default(),
        &mut out,
        |_| true
    )));
    assert!(stopped(typst::write(
        &derivation,
        &typst::Options::default(),
        &mut out,
        |_| true
    )));
    assert!(stopped(svg::write(
        &derivation,
        &svg::Style::default(),
        &mut out,
        |_| true
    )));
    assert!(stopped(rocq::write(
        &derivation,
        &rocq::Options::default(),
        &mut out,
        |_| true
    )));
}

#[test]
fn a_search_and_a_session_stop() {
    let sequent = wide(8);
    let outcome = prove_within(
        &sequent,
        Mode::CLASSICAL,
        &Options::default(),
        &Limits::default(),
        |_| true,
    );
    assert!(matches!(
        outcome.unwrap().verdict,
        Verdict::Unknown(Reason::Stopped)
    ));

    let mut session = Interactive::new(&sequent, Mode::CLASSICAL).unwrap();
    let goal = session.goals().next().unwrap();
    let outcome = session.close(
        goal,
        &Options::default(),
        &ViewOptions::default(),
        &Limits::default(),
        |_| true,
    );
    assert!(matches!(
        outcome.unwrap().outcome.verdict,
        Verdict::Unknown(Reason::Stopped)
    ));
    assert!(session.goals().eq([goal]));
}
