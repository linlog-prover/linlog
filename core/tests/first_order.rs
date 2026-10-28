// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! What a sequent with a quantifier is refused by, until a later version
//! proves and checks first-order linear logic: every engine by its
//! fragment, and the checker, interactive proving, the proof structures
//! and the refutations' checks by name. A ground first-order sequent is
//! propositional and refused by none.
#![cfg(all(feature = "parse", feature = "serialize", feature = "interactive"))]

use linlog::search::Engine;
use linlog::{
    CheckError, Criterion, CutFormula, Disproof, Error, Forest, InputOptions, Interactive, Limits,
    Mode, Options, Proof, ProofStructure, Sequent, Verdict,
};

/// Every engine refuses a quantified goal, the dispatch finding none,
/// and the ordered modes refuse it by the fragment.
#[test]
fn every_engine_refuses_a_quantified_goal() {
    let sequent: Sequent = "forall x. p(x) |- p(a)".parse().unwrap();
    assert_eq!(sequent.fragment().to_string(), "MLL1");
    for mode in [
        Mode::CLASSICAL,
        Mode::INTUITIONISTIC,
        Mode::CLASSICAL.with_mix(),
    ] {
        let error = linlog::prove(&sequent, mode, &Options::default()).unwrap_err();
        assert!(matches!(error, Error::NoEngine { .. }), "{mode}: {error}");
        for &engine in Engine::ALL {
            let options = Options::default().with_engine(Some(engine));
            let error = linlog::prove(&sequent, mode, &options).unwrap_err();
            assert!(
                matches!(error, Error::EngineRefused { .. }),
                "{engine} in {mode}: {error}"
            );
        }
    }
    let cyclic = Sequent::parse_ordered(
        "forall x. p(x) |- p(a)",
        linlog::Order::Cyclic,
        &Limits::default(),
    )
    .unwrap();
    let error = linlog::prove(&cyclic, Mode::CYCLIC, &Options::default()).unwrap_err();
    assert!(matches!(error, Error::OrderedFragment { .. }), "{error}");
    // A ground sequent is propositional and proved.
    let ground: Sequent = "p(a), p(a) -o q(f(a)) |- q(f(a))".parse().unwrap();
    let outcome = linlog::prove(&ground, Mode::INTUITIONISTIC, &Options::default()).unwrap();
    assert!(matches!(outcome.verdict, Verdict::Proved(_)));
    assert_eq!(outcome.fragment.to_string(), "MLL");
}

/// The checker, interactive proving, the proof structures and the
/// refutations' checks refuse a sequent with a quantifier by name.
#[test]
fn the_checker_and_the_views_refuse_a_quantified_sequent() {
    let sequent: Sequent = "forall x. p(x) |- p(a)".parse().unwrap();
    let json = format!(
        r#"{{"version":7,"sequent":{},"nodes":[{{"ax":[1,2]}}],"mode":"classical"}}"#,
        serde_json::to_string(&sequent).unwrap()
    );
    let proof: Proof = serde_json::from_str(&json).unwrap();
    assert_eq!(proof.check(Mode::CLASSICAL), Err(CheckError::Quantifiers));
    assert_eq!(Error::from(CheckError::Quantifiers).code(), "quantifiers");
    let error = Interactive::new(&sequent, Mode::INTUITIONISTIC).unwrap_err();
    assert!(
        matches!(
            error,
            Error::Quantifiers {
                what: "interactive proving",
                ..
            }
        ),
        "{error}"
    );
    let forest = Forest::new(&sequent).unwrap();
    let error = ProofStructure::new(forest, Criterion::default()).unwrap_err();
    assert!(matches!(error, Error::Net(_)), "{error}");
    let json = format!(
        r#"{{"version":7,"sequent":{},"mode":"classical","refutation":{{"kind":"exhausted"}}}}"#,
        serde_json::to_string(&sequent).unwrap()
    );
    let disproof: Disproof = serde_json::from_str(&json).unwrap();
    let error = disproof.check().unwrap_err();
    assert!(matches!(error, Error::Refutation(_)), "{error}");
}

/// A session read from JSON and a cut formula refuse a quantifier as the
/// session's start does, and a cut on a ground atom joins the formula's
/// symbols, so that the session writes and reads back.
#[test]
fn a_session_refuses_a_quantifier_on_every_path() {
    let sequent: Sequent = "forall x. p(x) |- p(a)".parse().unwrap();
    let json = format!(
        r#"{{"version":7,"sequent":{},"mode":"classical","inferences":[{{"sequent":[0,2]}}],"history":[]}}"#,
        serde_json::to_string(&sequent).unwrap()
    );
    let error = serde_json::from_str::<Interactive>(&json).unwrap_err();
    assert!(error.to_string().contains("quantifiers"), "{error}");
    let limits = Limits::default();
    let input = InputOptions::default();
    let mut session = Interactive::new(&"a |- a".parse().unwrap(), Mode::CLASSICAL).unwrap();
    let goal = session.goals().next().unwrap();
    let formula =
        CutFormula::parse_within("forall x. p(x)", Mode::CLASSICAL, &input, &limits).unwrap();
    let error = session.cut(goal, &formula, &[], &limits).unwrap_err();
    assert!(
        matches!(
            error,
            Error::Quantifiers {
                what: "interactive proving",
                ..
            }
        ),
        "{error}"
    );
    let formula = CutFormula::parse_within("p(b)", Mode::CLASSICAL, &input, &limits).unwrap();
    session.cut(goal, &formula, &[], &limits).unwrap();
    let json = serde_json::to_string(&session).unwrap();
    assert!(json.starts_with(r#"{"version":7,"#), "{json}");
    let back: Interactive = serde_json::from_str(&json).unwrap();
    assert_eq!(serde_json::to_string(&back).unwrap(), json);
}

/// Two sequents added keep their atoms paired and their symbols in their
/// roles: a nullary atom of a propositional side is the predicate of
/// that name, not a function symbol.
#[test]
fn adding_sequents_joins_their_tables() {
    let mut s: Sequent = "|- q, p(a)".parse().unwrap();
    s.add("|- ~q, p(f(a))".parse().unwrap());
    assert_eq!(s.to_string(), "⊢ q, p(a), ~q, p(f(a))");
    assert_eq!(s.atom_names(), ["q", "p(a)", "p(f(a))"]);
    let json = serde_json::to_string(&s).unwrap();
    assert_eq!(serde_json::from_str::<Sequent>(&json).unwrap(), s);
    let mut t: Sequent = "|- ~q".parse().unwrap();
    t.add("|- q, p(a)".parse().unwrap());
    assert_eq!(t.atom_names(), ["q", "p(a)"]);
    // A cyclic net of a ground sequent is a level-7 document.
    let cyclic =
        Sequent::parse_ordered("p(a) |- p(a)", linlog::Order::Cyclic, &Limits::default()).unwrap();
    let outcome = linlog::prove(&cyclic, Mode::CYCLIC, &Options::default()).unwrap();
    let net = outcome.net.expect("a net");
    assert!(
        serde_json::to_string(&net)
            .unwrap()
            .starts_with(r#"{"version":7,"#)
    );
}
