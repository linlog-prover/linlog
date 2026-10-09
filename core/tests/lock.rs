// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The behaviour lock of the JSON forms: a sequent, a proof, an outcome
//! with every verdict and refutation, a proof structure and a session,
//! each written as the library writes it and pinned in
//! `tests/lock/json.txt`, one `NAME: JSON` per line. A change of a wire
//! form shows here as a changed line; such a change is a commit of its
//! own that says why. `BLESS=1 cargo test --test lock` rewrites the file.
//! Every search runs on one thread, so its counters are a function of the
//! input.
#![cfg(all(feature = "parse", feature = "serialize", feature = "interactive"))]

use linlog::search::Engine;
use linlog::{
    Forest, Interactive, Mode, Named, OccId, Options, ProofStructure, Sequent, Step, ViewOptions,
};
use std::fmt::Write as _;

/// Parses a sequent of the corpus.
fn sequent(text: &str) -> Sequent {
    text.parse().unwrap()
}

/// The JSON of a value.
fn json<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string(value).unwrap()
}

/// Every line of the lock, in order.
fn lines() -> String {
    let mut out = String::new();
    let mut pin = |name: &str, json: String| writeln!(out, "{name}: {json}").unwrap();
    let classical = Mode::CLASSICAL;
    let intuitionistic = Mode::INTUITIONISTIC;
    let affine = Mode::CLASSICAL.with_affine();
    let mix = Mode::CLASSICAL.with_mix();
    let options = Options::default();

    for (name, text) in [
        ("sequent-mll", "A * B |- B * A"),
        ("sequent-units", "1, bot |- top, 0"),
        ("sequent-ll", "!(A & B), ?C |- A -o ~B + C"),
        ("sequent-empty", "|-"),
        ("sequent-repeated", "A, A |- A * A"),
    ] {
        pin(name, json(&sequent(text)));
    }

    for (name, text, mode) in [
        ("outcome-mll", "A * B |- B * A", classical),
        ("outcome-mall", "A & B |- A + B", classical),
        ("outcome-mell", "!A |- A * !A", classical),
        ("outcome-units", "1, bot |- 1 * bot", classical),
        ("outcome-intuitionistic", "A, A -o B |- B", intuitionistic),
        ("outcome-affine", "A, B |- A", affine),
        ("outcome-mix", "A, B |- A, B", mix),
        ("outcome-horn", "!(a -o a * b), a |- a * b * b", classical),
        ("outcome-unbalanced", "A |- A * A", classical),
        ("outcome-equation", "|- A * ~A", classical),
        ("outcome-exhausted", "A & B |- A * B", classical),
        (
            "outcome-state-equation",
            "!(A -o A * A), !(B * B -o C), A, B |- C",
            classical,
        ),
    ] {
        let outcome = linlog::prove(&sequent(text), mode, &options).unwrap();
        pin(name, json(&outcome));
    }

    let hard = sequent("!(A & B) |- !A * ?B");
    let tiny = linlog::Limits::default().with_memory_bytes(Some(100));
    pin(
        "outcome-memory-limit",
        json(&linlog::prove_within(&hard, classical, &options, &tiny, |_| false).unwrap()),
    );
    let stopped = linlog::prove_within(
        &hard,
        classical,
        &options,
        &linlog::Limits::default(),
        |_| true,
    )
    .unwrap();
    pin("outcome-stopped", json(&stopped));
    let bounded = options.clone().with_copies(Some(0));
    pin(
        "outcome-copy-bound",
        json(&linlog::prove(&hard, classical, &bounded).unwrap()),
    );
    let forced = options.clone().with_engine(Some(Engine::Focus));
    pin(
        "outcome-forced-engine",
        json(&linlog::prove(&sequent("A * B |- B * A"), classical, &forced).unwrap()),
    );

    let proved = |text: &str, mode| match linlog::prove(&sequent(text), mode, &options)
        .unwrap()
        .verdict
    {
        linlog::Verdict::Proved(proof) => proof,
        other => panic!("{text}: {other:?}"),
    };
    for (name, text, mode) in [
        ("proof-mll", "A * B |- B * A", classical),
        ("proof-ll", "!(A & B) |- !A * ?B", classical),
        ("proof-intuitionistic", "A, A -o B |- B", intuitionistic),
        ("proof-affine", "A, B |- A", affine),
        ("proof-mix", "A, B |- A, B", mix),
    ] {
        pin(name, json(&proved(text, mode)));
    }

    let swap = proved("A * B |- B * A", classical);
    pin(
        "structure-from-proof",
        json(&ProofStructure::from_proof(&swap, false).unwrap()),
    );
    let modus = sequent("A, A -o B |- B");
    let partial = ProofStructure::from_links(
        Forest::new(&modus).unwrap(),
        true,
        &[(OccId::new(0), OccId::new(2))],
    )
    .unwrap();
    pin("structure-partial", json(&partial));

    let mut session = Interactive::new(&sequent("A * B |- B * A"), classical).unwrap();
    pin("session-open", json(&session));
    let root = session.goals().next().unwrap();
    session
        .apply(root, &Step::new(0, "par".parse::<Named>().unwrap()))
        .unwrap();
    pin("session-step", json(&session));
    session.close_all(
        &options,
        &ViewOptions::default(),
        &linlog::Limits::default(),
        |_| false,
    );
    pin("session-closed", json(&session));
    let mut session = Interactive::new(&sequent("A, A -o B |- B"), intuitionistic).unwrap();
    session.close_all(
        &options,
        &ViewOptions::default(),
        &linlog::Limits::default(),
        |_| false,
    );
    pin("session-intuitionistic", json(&session));

    out
}

/// Every JSON form of the corpus is the line the lock pins.
#[test]
fn json_lock() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/lock/json.txt");
    let got = lines();
    if std::env::var_os("BLESS").is_some() {
        std::fs::write(&path, &got).unwrap();
        return;
    }
    let pinned = std::fs::read_to_string(&path).unwrap_or_default();
    let changed: Vec<&str> = got
        .lines()
        .zip(pinned.lines())
        .filter(|(a, b)| a != b)
        .map(|(a, _)| a.split(':').next().unwrap())
        .collect();
    assert!(
        changed.is_empty() && got.lines().count() == pinned.lines().count(),
        "a JSON form changed (BLESS=1 rewrites the file): {changed:?}, {} lines where {} are pinned",
        got.lines().count(),
        pinned.lines().count()
    );
}
