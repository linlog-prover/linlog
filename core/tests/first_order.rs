// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! What a sequent with a quantifier is refused by, until a later version
//! proves and checks first-order linear logic: every engine by its
//! fragment, and the checker, interactive proving, the proof structures
//! and the refutations' checks by name. A ground first-order sequent is
//! propositional and refused by none.
#![cfg(all(feature = "parse", feature = "serialize", feature = "interactive"))]

use linlog::proofs::Fault;
use linlog::search::Engine;
use linlog::{
    CheckError, Criterion, CutFormula, Disproof, Error, Forest, InputOptions, Interactive, Limits,
    Member, Mode, Options, Proof, ProofStructure, Sequent, Verdict,
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

/// Interactive proving, the proof structures and the refutations' checks
/// refuse a sequent with a quantifier by name; the checker gives a
/// verdict, here on an axiom that names an open atom without its instance.
#[test]
fn the_views_refuse_a_quantified_sequent_and_the_checker_judges_it() {
    let sequent: Sequent = "forall x. p(x) |- p(a)".parse().unwrap();
    let json = format!(
        r#"{{"version":7,"sequent":{},"nodes":[{{"ax":[1,2]}}],"mode":"classical"}}"#,
        serde_json::to_string(&sequent).unwrap()
    );
    let proof: Proof = serde_json::from_str(&json).unwrap();
    let Err(CheckError::Invalid(invalid)) = proof.check(Mode::CLASSICAL) else {
        panic!("an open atom named bare is no instance");
    };
    assert!(
        matches!(invalid.fault, Fault::Instance { member, .. } if member == Member::new(1)),
        "{invalid:?}"
    );
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

/// The proofs written by hand under `tests/first-order/`, read through
/// the wire: those the library's own tests accept are proofs in the mode
/// they record, and the others are no proofs, with a fault whose message
/// names formulas and instances.
#[test]
fn the_written_proofs_read_back_and_are_judged() {
    let accepted = [
        "apart",
        "cut",
        "cut_ill",
        "cut_instance",
        "swap",
        "swap_ill",
        "witness",
        "witness_ill",
    ];
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/first-order");
    let mut read = 0;
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_stem().unwrap().to_str().unwrap().to_owned();
        let json = std::fs::read_to_string(&path).unwrap();
        let proof: Proof = serde_json::from_str(&json).unwrap();
        let mode = proof.mode().unwrap_or(Mode::CLASSICAL);
        let mode = if name.starts_with("apart") || name.starts_with("shared") {
            mode.with_affine()
        } else {
            mode
        };
        let result = proof.check(mode);
        if accepted.contains(&name.as_str()) {
            assert_eq!(result, Ok(()), "{name}");
        } else {
            let error = result.expect_err(&name);
            assert!(matches!(error, CheckError::Invalid(_)), "{name}: {error}");
            let message = error.describe(&proof).to_string();
            assert!(message.starts_with("node "), "{name}: {message}");
        }
        read += 1;
    }
    assert_eq!(read, 16);
    // The eigenvariable a `∃` step below its `∀` step names, by name.
    let json = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/first-order/swap_back.json"),
    )
    .unwrap();
    let proof: Proof = serde_json::from_str(&json).unwrap();
    let error = proof.check(Mode::CLASSICAL).unwrap_err();
    assert_eq!(
        error.describe(&proof).to_string(),
        "node 4 (∃ on ∃x. (∀y. p(x, y)) from 3) with premises ⊢ ∃y. (∀x. ~p(x, y)), \
         ∀y. p(a, y): eigenvariable a occurs outside the ∀ step that introduces it"
    );
}

/// The first-order terms a sequent's atoms unfold to are bounded on every
/// reader by `limits.terms`: the text, the sequent's JSON and a proof's
/// instance table, each refused past it with its own code.
#[test]
fn the_term_limit_refuses_at_every_reader() {
    let text = "|- p(f(f(a)))";
    let sequent: Sequent = text.parse().unwrap();
    assert_eq!(sequent.fo_size(), 3);
    let tight = Limits::default().with_terms(Some(2));
    let error = Sequent::parse_within(text, &tight).unwrap_err();
    assert_eq!(error.code(), "too_many_terms", "{error}");
    assert!(Sequent::parse_within(text, &Limits::default().with_terms(Some(3))).is_ok());
    let json = serde_json::to_string(&sequent).unwrap();
    let mut document = serde_json::Deserializer::from_str(&json);
    let error = linlog::wire::upgrade::<Sequent, _>(&mut document, &tight).unwrap_err();
    assert_eq!(error.code(), "too_many_terms", "{error}");
    // A proof whose sequent is within the bound and whose instance table
    // is not.
    let json = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/first-order/swap.json"),
    )
    .unwrap();
    let proof: Proof = serde_json::from_str(&json).unwrap();
    // Four members, four frames and two terms.
    assert_eq!(proof.instances().unwrap().len(), 4);
    assert_eq!(proof.sequent().fo_size(), 4);
    let limits = Limits::default().with_terms(Some(5));
    let mut document = serde_json::Deserializer::from_str(&json);
    let error = linlog::wire::upgrade::<Proof, _>(&mut document, &limits).unwrap_err();
    assert_eq!(error.code(), "too_many_terms", "{error}");
    assert_eq!(error.setting(), Some("limits.terms"));
}
