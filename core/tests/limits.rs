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
}

#[test]
fn the_checker_stops_and_keeps_its_bounds() {
    // Past the 4 096 nodes between two polls of the stop.
    let proof = proof(1500);
    let mode = Mode::CLASSICAL;
    let stopped = proof.check_within(mode, &Limits::default(), |_| true);
    assert!(matches!(
        stopped,
        Err(CheckError::Refused(r)) if r.refusal == Refusal::Stopped { phase: Phase::Check }
    ));
    let work = proof.check_within(mode, &Limits::default().with_work(Some(10)), |_| false);
    assert!(matches!(
        work,
        Err(CheckError::Refused(r)) if r.refusal == Refusal::Work { limit: 10 }
    ));
    let memory = proof.check_within(mode, &Limits::default().with_memory_bytes(Some(0)), |_| {
        false
    });
    assert!(matches!(
        memory,
        Err(CheckError::Refused(r)) if matches!(r.refusal, Refusal::Memory { .. })
    ));
    let size = proof.derivation_size_within(false, &Limits::default(), |_| true);
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
                phase: Phase::Write
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
        outcome.unwrap().verdict,
        Verdict::Unknown(Reason::Stopped)
    ));
    assert!(session.goals().eq([goal]));
}
