// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! What takes a sequent with a quantifier and what refuses it: the
//! focused engine in the classical modes and the two-sided one in the
//! intuitionistic modes prove and refute it, every other engine refuses
//! it by its fragment, as interactive proving, the
//! proof structures and the propositional refutations' checks do by name.
//! A ground first-order sequent is propositional and refused by none.
#![cfg(all(feature = "parse", feature = "serialize", feature = "interactive"))]

use linlog::proofs::Fault;
use linlog::proofs::interactive::{GoalId, Needs, StepError, TermNode, Witness};
use linlog::search::Engine;
use linlog::{
    CheckError, Criterion, CutFormula, Disproof, Eigen, Error, Forest, InputOptions, Interactive,
    Limits, Member, Mode, Options, Proof, ProofStructure, Rule, Sequent, Step, Verdict,
};

/// The focused engine decides a quantified goal in the classical modes,
/// Mix included, and the two-sided engine in the intuitionistic modes, by
/// the dispatch and forced; every other engine refuses it; the ordered
/// modes refuse it by the fragment.
#[test]
fn the_focused_engine_takes_a_quantified_goal_and_no_other() {
    let sequent: Sequent = "forall x. p(x) |- p(a)".parse().unwrap();
    assert_eq!(sequent.fragment().to_string(), "MLL1");
    for mode in [
        Mode::CLASSICAL,
        Mode::INTUITIONISTIC,
        Mode::CLASSICAL.with_mix(),
        Mode::INTUITIONISTIC.with_affine(),
    ] {
        let focused = if mode.is_intuitionistic() {
            Engine::TwoSided
        } else {
            Engine::Focus
        };
        let outcome = linlog::prove(&sequent, mode, &Options::default()).unwrap();
        assert!(matches!(outcome.verdict, Verdict::Proved(_)), "{mode}");
        assert_eq!(outcome.engine, focused);
        for &engine in Engine::ALL {
            let options = Options::default().with_engine(Some(engine));
            match linlog::prove(&sequent, mode, &options) {
                Ok(outcome) if engine == focused => {
                    assert!(matches!(outcome.verdict, Verdict::Proved(_)), "{mode}");
                }
                Err(Error::EngineRefused { .. }) => {}
                other => panic!("{engine} in {mode}: {other:?}"),
            }
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

/// The proof structures and the refutations' checks refuse a sequent
/// with a quantifier by name; the checker gives a verdict, here on an
/// axiom that names an open atom without its instance.
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

/// A session read from JSON names an occurrence under a quantifier only
/// by its instance; a cut formula with a quantifier, and any cut in a
/// session with one, are refused; and a cut on a ground atom joins the
/// formula's symbols, so that the session writes and reads back.
#[test]
fn a_session_refuses_a_quantifier_on_every_path() {
    let sequent: Sequent = "forall x. p(x) |- p(a)".parse().unwrap();
    let json = format!(
        r#"{{"version":7,"sequent":{},"mode":"classical","inferences":[{{"sequent":[0,2],"rule":"∃","principal":0,"premises":[1]}},{{"sequent":[1,2]}}],"history":[0]}}"#,
        serde_json::to_string(&sequent).unwrap()
    );
    let error = serde_json::from_str::<Interactive>(&json).unwrap_err();
    assert!(error.to_string().contains("instance"), "{error}");
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
                what: "a cut in interactive proving",
                ..
            }
        ),
        "{error}"
    );
    let mut quantified = Interactive::new(&sequent, Mode::CLASSICAL).unwrap();
    let root = quantified.goals().next().unwrap();
    let ground = CutFormula::parse_within("q(b)", Mode::CLASSICAL, &input, &limits).unwrap();
    let error = quantified.cut(root, &ground, &[], &limits).unwrap_err();
    assert!(matches!(error, Error::Quantifiers { .. }), "{error}");
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
        "cut_deep",
        "cut_deep_ill",
        "cut_ill",
        "cut_instance",
        "cut_shared",
        "cut_shared_ill",
        "cut_tensor",
        "cut_tensor_ill",
        "swap",
        "swap_ill",
        "unintroduced",
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
    assert_eq!(read, 22);
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

/// The proofs with cuts written by hand under `tests/first-order/`, read
/// through the wire, are eliminated in the mode they record (classical
/// where they record none): every step's
/// proof passes the checker and the last is cut-free; among them a cut on
/// a first-order formula (`cut`, `cut_deep`), one on an instance
/// (`cut_instance`, `cut_tensor`), a subproof shared under two cuts on a
/// `∀` formula (`cut_shared`), and an eigenvariable in a witness two rules
/// up its `∀` premise (`cut_deep`). The proof structures refuse them.
#[test]
fn the_written_cuts_are_eliminated() {
    use linlog::proofs::cut::{self, Case, End};
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/first-order");
    let mut cases: Vec<Case> = Vec::new();
    for name in [
        "cut",
        "cut_ill",
        "cut_instance",
        "cut_shared",
        "cut_shared_ill",
        "cut_tensor",
        "cut_tensor_ill",
        "cut_deep",
        "cut_deep_ill",
    ] {
        let json = std::fs::read_to_string(dir.join(format!("{name}.json"))).unwrap();
        let proof: Proof = serde_json::from_str(&json).unwrap();
        assert!(!proof.is_cut_free(), "{name}");
        let mode = proof.mode().unwrap_or(Mode::CLASSICAL);
        let mut taken = 0;
        let elimination = cut::eliminate(
            &proof,
            mode,
            &cut::Options::default(),
            &Limits::default(),
            |_| false,
            |before, step| {
                assert!(before.check(mode).is_ok(), "{name}: step {taken}");
                cases.push(step.case);
                taken += 1;
            },
        )
        .unwrap();
        assert_eq!(elimination.end, End::CutFree, "{name}");
        assert_eq!(elimination.steps.len(), taken, "{name}");
        assert!(elimination.proof.is_cut_free(), "{name}");
        assert_eq!(elimination.proof.check(mode), Ok(()), "{name}");
        assert_eq!(
            elimination.proof.sequent().to_string(),
            proof.sequent().to_string()
        );
        let net =
            ProofStructure::from_proof(&proof, Criterion::default(), &Limits::default(), |_| false);
        assert!(net.is_err(), "{name}");
    }
    for case in [
        Case::ForallExists,
        Case::TensorPar,
        Case::Axiom,
        Case::Commutative,
    ] {
        assert!(cases.contains(&case), "{case:?} not met");
    }
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

/// A cut formula of a document is closed as a root is: one with a loose
/// bound variable is refused when read, not left for the checker, which
/// would look past its instance's frame.
#[test]
fn a_cut_formula_with_a_loose_variable_is_refused() {
    let json = r#"{"version":7,"sequent":{"terms":[{"V":0},{"∀":0},"⊤",{"D":0},{"∃":3}],"roots":[2],"atoms":[{"P":[0,[0]]}],"symbols":[["p",1]],"fo":[{"B":1}],"binders":["x","x"],"antecedents":0,"cuts":[1]},"nodes":[{"ax":[5,6]}],"instances":{"members":[[2,0],[4,0]],"frames":[[null,1]],"terms":[{"E":0}]}}"#;
    let error = serde_json::from_str::<Proof>(json).unwrap_err();
    assert!(error.to_string().contains("bound variable"), "{error}");
}

/// What the verdict of the default search of `text` in `mode` is: `Some`
/// whether it is proved, its proof checked and holding an instance
/// table, or `None` for unknown.
fn decided(text: &str, mode: Mode, options: &Options) -> Option<bool> {
    let sequent: Sequent = text.parse().unwrap();
    let outcome = linlog::prove(&sequent, mode, options).unwrap();
    let engine = if mode.is_intuitionistic() {
        Engine::TwoSided
    } else {
        Engine::Focus
    };
    assert_eq!(outcome.engine, engine, "{text}");
    match outcome.verdict {
        Verdict::Proved(proof) => {
            assert_eq!(proof.check(mode), Ok(()), "{text}");
            assert!(proof.instances().is_some(), "{text}");
            Some(true)
        }
        Verdict::Unprovable(_) => Some(false),
        Verdict::Unknown(_) => None,
    }
}

/// The memo and the loop check key a first-order stable sequent by its
/// instance: a clause that leads back to the sequent it started from is
/// pruned as a repeat, the failures below are reused, and the search
/// ends exhaustive where without the memo every copy level is cut by the
/// budget; a sequent proved with the memo has its proof checked.
#[test]
fn the_memo_and_the_loop_check_key_first_order_sequents() {
    let looping = [
        "!(forall x. (p(x) -o p(x))), !(q -o r), !(r -o q), p(e) |- r",
        "!(a -o a), !(b -o c), !(c -o b), a |- forall z. c",
    ];
    let options = Options::default().with_copies(Some(6));
    for text in looping {
        let outcome =
            linlog::prove(&text.parse().unwrap(), Mode::INTUITIONISTIC, &options).unwrap();
        assert!(matches!(outcome.verdict, Verdict::Unprovable(_)), "{text}");
        assert!(outcome.statistics.memo_hits > 0, "{text}");
        let unkeyed = options.clone().with_memo_limit(0);
        let outcome =
            linlog::prove(&text.parse().unwrap(), Mode::INTUITIONISTIC, &unkeyed).unwrap();
        assert!(matches!(outcome.verdict, Verdict::Unknown(_)), "{text}");
    }
    let proved = "!(forall x. (p(x) -o p(x))), !(forall x. (p(x) -o q(x))), p(e) |- q(e)";
    assert_eq!(decided(proved, Mode::INTUITIONISTIC, &options), Some(true));
}

/// The focused engine decides first-order sequents in every classical
/// mode: the order of the quantifiers (the eigenvariable condition by
/// levels), a witness shared by both factors of a `⊗` whose first answer
/// fails (another answer asked for), the occurs check, a witness left
/// open (closed by a fresh constant), copies of a quantified hypothesis,
/// Mix, and weakening; every proof checked.
#[test]
fn the_focused_engine_decides_first_order_sequents() {
    let (classical, mix, affine) = (
        Mode::CLASSICAL,
        Mode::CLASSICAL.with_mix(),
        Mode::CLASSICAL.with_affine(),
    );
    let options = Options::default();
    for (text, mode, expected) in [
        ("forall x. p(x) |- p(a)", classical, true),
        ("forall x. p(x) |- exists y. p(y)", classical, true),
        ("exists x. p(x) |- forall y. p(y)", classical, false),
        (
            "exists y. forall x. r(x, y) |- forall x. exists y. r(x, y)",
            classical,
            true,
        ),
        (
            "forall x. exists y. r(x, y) |- exists y. forall x. r(x, y)",
            classical,
            false,
        ),
        (
            "p(a) & p(b), q(b) |- exists x. (p(x) * q(x))",
            classical,
            true,
        ),
        (
            "p(b) & p(a), q(b) |- exists x. (p(x) * q(x))",
            classical,
            true,
        ),
        (
            "p(a) & p(b), q(c) |- exists x. (p(x) * q(x))",
            classical,
            false,
        ),
        ("|- exists x. (~p(x) par p(f(x)))", classical, false),
        ("|- forall y. exists x. (~p(x) par p(y))", classical, true),
        ("|- exists x. forall y. (~p(x) par p(y))", classical, false),
        ("forall x. (p(x) & q(x)) |- p(a) & q(a)", classical, true),
        ("forall x. p(x) |- p(a) * p(b)", classical, false),
        ("!forall x. p(x) |- p(a) * p(b)", classical, true),
        (
            "p(z), !forall x. (p(x) -o p(s(x))) |- p(s(s(s(z))))",
            classical,
            true,
        ),
        (
            "|- exists x. (p(x) * q(x)), ~p(a), ~q(a), r, ~r",
            classical,
            false,
        ),
        ("|- exists x. (p(x) * q(x)), ~p(a), ~q(a), r, ~r", mix, true),
        ("forall x. p(x), q(b) |- p(a)", affine, true),
        ("forall x. p(x) |- q(a)", affine, false),
    ] {
        assert_eq!(
            decided(text, mode, &options),
            Some(expected),
            "{text} in {mode}"
        );
    }
}

/// The two-sided engine decides first-order sequents in the
/// intuitionistic modes: the order of the quantifiers, a `∃` on the left
/// opened by an eigenvariable, a quantified implication split by `⊸L`
/// with the goal on its consequent's side, a witness that the second
/// factor of a `⊗` on the right rejects (another answer asked for),
/// copies of a quantified hypothesis under `!`, a quantifier shifted
/// across `⊸` one way and refuted the other, and weakening; every proof
/// checked.
#[test]
fn the_two_sided_engine_decides_first_order_sequents() {
    let (linear, affine) = (Mode::INTUITIONISTIC, Mode::INTUITIONISTIC.with_affine());
    let options = Options::default();
    for (text, mode, expected) in [
        ("forall x. p(x) |- exists y. p(y)", linear, true),
        ("exists x. p(x) |- p(a)", linear, false),
        (
            "exists y. forall x. r(x, y) |- forall x. exists y. r(x, y)",
            linear,
            true,
        ),
        (
            "forall x. exists y. r(x, y) |- exists y. forall x. r(x, y)",
            linear,
            false,
        ),
        (
            "exists x. (p(x) * q(x)) |- exists y. (q(y) * p(y))",
            linear,
            true,
        ),
        ("forall x. (p(x) -o q(x)), p(a) |- q(a)", linear, true),
        ("forall x. (p(x) -o q(x)), p(a) |- q(b)", linear, false),
        ("p(a) & p(b), q(b) |- exists x. (p(x) * q(x))", linear, true),
        (
            "p(a) & p(b), q(c) |- exists x. (p(x) * q(x))",
            linear,
            false,
        ),
        (
            "p(z), !forall x. (p(x) -o p(s(x))) |- p(s(s(z)))",
            linear,
            true,
        ),
        (
            "exists x. (p(x) -o q) |- (forall x. p(x)) -o q",
            linear,
            true,
        ),
        (
            "(forall x. p(x)) -o q |- exists x. (p(x) -o q)",
            linear,
            false,
        ),
        ("forall x. p(x), q(b) |- p(a)", affine, true),
        ("forall x. p(x) |- q(a)", affine, false),
    ] {
        assert_eq!(
            decided(text, mode, &options),
            Some(expected),
            "{text} in {mode}"
        );
    }
}

/// A premise whose answer fails the other is asked for its next one:
/// the witness that a later factor needs is the last of its premise's
/// answers, provable with every proof checked, and refuted where no
/// answer serves, in both engines.
#[test]
fn a_premise_gives_its_answers_in_turn() {
    let options = Options::default();
    for (goal, provable) in [("q(a4)", true), ("q(b)", false)] {
        let ill = format!(
            "p(a1), p(a2), p(a3), p(a4), {goal} |- exists x. (p(x) * q(x)) * exists x. p(x) * exists x. p(x) * exists x. p(x)"
        );
        assert_eq!(
            decided(&ill, Mode::INTUITIONISTIC, &options),
            Some(provable),
            "{ill}"
        );
        let one_sided = format!(
            "|- ~p(a1), ~p(a2), ~p(a3), ~p(a4), ~{goal}, exists x. (p(x) * q(x)) * exists x. p(x) * exists x. p(x) * exists x. p(x)"
        );
        assert_eq!(
            decided(&one_sided, Mode::CLASSICAL, &options),
            Some(provable),
            "{one_sided}"
        );
    }
}

/// The first-order families with verdicts by construction: a witness
/// among `n` alternatives (`p(a₁) & … & p(aₙ), q(aₖ) ⊢ ∃x.(p(x) ⊗ q(x))`,
/// unprovable with `q(b)`), and a chain of `n` steps of a quantified
/// clause under `!`, provable within `n` copies and not within fewer
/// (`p(z), !∀x.(p(x) ⊸ p(s(x))) ⊢ p(sⁿ(z))`).
#[test]
fn the_first_order_families_have_their_verdicts() {
    for n in 1..=5 {
        let alternatives: Vec<String> = (1..=n).map(|i| format!("p(a{i})")).collect();
        let choice = alternatives.join(" & ");
        for k in 1..=n {
            let text = format!("{choice}, q(a{k}) |- exists x. (p(x) * q(x))");
            assert_eq!(
                decided(&text, Mode::CLASSICAL, &Options::default()),
                Some(true),
                "{text}"
            );
        }
        let text = format!("{choice}, q(b) |- exists x. (p(x) * q(x))");
        assert_eq!(
            decided(&text, Mode::CLASSICAL, &Options::default()),
            Some(false),
            "{text}"
        );
    }
    for n in 1..=4u32 {
        let mut goal = "z".to_owned();
        for _ in 0..n {
            goal = format!("s({goal})");
        }
        let text = format!("p(z), !forall x. (p(x) -o p(s(x))) |- p({goal})");
        let options = |copies| Options::default().with_copies(Some(copies));
        assert_eq!(
            decided(&text, Mode::CLASSICAL, &options(n)),
            Some(true),
            "{text}"
        );
        assert_eq!(
            decided(&text, Mode::CLASSICAL, &options(n - 1)),
            None,
            "{text}"
        );
    }
}

/// A first-order search counts the unifications of two literals it tried
/// and the metavariables it bound, one-sided and two-sided; a
/// propositional search counts neither, and its JSON leaves them out.
#[test]
fn a_first_order_search_counts_its_unifications() {
    let counted = |text: &str, mode: Mode| {
        let sequent: Sequent = text.parse().unwrap();
        linlog::prove(&sequent, mode, &Options::default())
            .unwrap()
            .statistics
    };
    for mode in [Mode::CLASSICAL, Mode::INTUITIONISTIC] {
        let statistics = counted("p(a), p(b) & q(c) |- exists x. (p(x) * p(b))", mode);
        assert!(statistics.unifications >= 2, "{mode}: {statistics:?}");
        assert!(statistics.bindings >= 1, "{mode}: {statistics:?}");
    }
    let statistics = counted("a, b |- a * b", Mode::CLASSICAL);
    assert_eq!((statistics.unifications, statistics.bindings), (0, 0));
    let written = serde_json::to_value(statistics).unwrap();
    assert!(written.get("unifications").is_none() && written.get("bindings").is_none());
}

/// Each first-order bound stops a search that reaches it, with its own
/// reason, and one more lets the search through: a witness deeper than
/// the witness depth, an instance past the bound on instances.
#[test]
fn the_first_order_bounds_stop_the_search() {
    use linlog::Reason;
    let deep = "|- exists x. p(x), ~p(f(f(f(a))))";
    let outcome = |text: &str, options: &Options| {
        linlog::prove(&text.parse().unwrap(), Mode::CLASSICAL, options)
            .unwrap()
            .verdict
    };
    let depth = |d| Options::default().with_witness_depth(d);
    let Verdict::Unknown(reason) = outcome(deep, &depth(2)) else {
        panic!("{deep} past the witness depth");
    };
    assert!(matches!(reason, Reason::WitnessDepth { depth: 2, .. }));
    assert_eq!(reason.setting(), Some("search.witness_depth"));
    assert!(matches!(outcome(deep, &depth(3)), Verdict::Proved(_)));
    let two = "|- exists x. exists y. (p(x) * p(y)), ~p(a), ~p(b)";
    let instances = |n| Options::default().with_instances(n);
    let Verdict::Unknown(reason) = outcome(two, &instances(1)) else {
        panic!("{two} past the instances");
    };
    assert!(matches!(reason, Reason::Instances { instances: 1, .. }));
    assert_eq!(reason.setting(), Some("search.instances"));
    assert!(matches!(outcome(two, &instances(2)), Verdict::Proved(_)));
    // Every copy level cut at once by the instances, none bounding the
    // copies: the levels are work, and the work bound ends them, in
    // either mode.
    let cut = "exists x. p(x), exists x. !p(x) |- q";
    let limits = Limits::default().with_work(Some(1 << 20));
    for mode in [Mode::CLASSICAL, Mode::INTUITIONISTIC] {
        let options = instances(1).with_copies(None);
        let outcome =
            linlog::prove_within(&cut.parse().unwrap(), mode, &options, &limits, |_| false)
                .unwrap();
        assert!(matches!(outcome.verdict, Verdict::Unknown(_)), "{mode}");
    }
}

/// The counts carry over to quantifiers: a predicate with an open atom is
/// counted over all its atoms, and refuted so by a kind of its own, which
/// the check accepts and the JSON reads back; an atom of a predicate with
/// only closed atoms is counted alone, as before; the count equation
/// counts a quantifier as its body; the classical refutation, which has
/// no domain, gives nothing.
#[test]
fn the_counts_refute_first_order_sequents() {
    use linlog::Refutation;
    let refuted = |text: &str| {
        let sequent: Sequent = text.parse().unwrap();
        match linlog::prove(&sequent, Mode::CLASSICAL, &Options::default())
            .unwrap()
            .verdict
        {
            Verdict::Unprovable(disproof) => *disproof,
            other => panic!("{text}: {other:?}"),
        }
    };
    let symbol = refuted("forall x. p(x), p(a) |- p(b)");
    assert!(
        matches!(symbol.refutation(), Refutation::UnbalancedSymbol(_)),
        "{symbol:?}"
    );
    assert_eq!(symbol.check(), Ok(()));
    assert!(
        symbol
            .to_string()
            .contains("~p(…) occurs 1 more time than p(…)"),
        "{symbol}"
    );
    let json = serde_json::to_string(&symbol).unwrap();
    assert!(json.contains("\"kind\":\"unbalanced_symbol\""), "{json}");
    let read: Disproof = serde_json::from_str(&json).unwrap();
    assert_eq!(read.check(), Ok(()));
    let atom = refuted("forall x. p(x) |- p(a) * q(b)");
    assert!(
        matches!(atom.refutation(), Refutation::Unbalanced(_)),
        "{atom:?}"
    );
    assert_eq!(atom.check(), Ok(()));
    let equation = refuted("forall x. (p(x) * q(x)) |- p(a) par q(a)");
    assert!(
        matches!(equation.refutation(), Refutation::Equation(_)),
        "{equation:?}"
    );
    assert_eq!(equation.check(), Ok(()));
    // An undecided search stays so: no assignment refutes a quantifier.
    let open = refuted("|- exists x. forall y. (~p(x) par p(y))");
    assert!(
        matches!(open.refutation(), Refutation::Exhausted),
        "{open:?}"
    );
}

/// A witness as deep as its terms go, here a hundred thousand
/// applications around an eigenvariable, is closed into the proof's
/// instance table and checked without a recursion per level, which ran
/// out of the search's stack.
#[test]
fn a_deep_witness_closes_without_recursion() {
    let n = 100_000;
    let text = format!(
        "|- forall z. (~p({}z{}) par (exists y. p(y)))",
        "f(".repeat(n),
        ")".repeat(n)
    );
    let options = Options::default().with_witness_depth(u32::MAX);
    assert_eq!(decided(&text, Mode::CLASSICAL, &options), Some(true));
}

/// Unifying deep terms is work the search counts and polls its stop on:
/// one split search whose every step unifies terms two thousand deep ends
/// at a bound on work, where it used to count none of it and run to its
/// end between two polls.
#[test]
fn a_deep_unification_is_counted_work() {
    use linlog::Reason;
    let deep = format!("{}z{}", "f(".repeat(2000), ")".repeat(2000));
    let literals: Vec<String> = (0..6).map(|i| format!("~p(g{i}({deep}))")).collect();
    let text = format!(
        "|- forall z. ({} par ~q(a) par (exists y. (p(y) * q(y))))",
        literals.join(" par ")
    );
    let sequent: Sequent = text.parse().unwrap();
    let options = Options::default().with_witness_depth(u32::MAX);
    let limits = Limits::default().with_work(Some(20_000));
    let outcome = linlog::prove_within(
        &sequent,
        Mode::CLASSICAL.with_affine(),
        &options,
        &limits,
        |_| false,
    )
    .unwrap();
    assert!(
        matches!(outcome.verdict, Verdict::Unknown(Reason::WorkLimit { .. })),
        "{:?}",
        outcome.verdict
    );
}

/// The position in a goal of the formula written `text`.
fn at(state: &Interactive, goal: GoalId, text: &str) -> usize {
    let goal = state.goal(goal).unwrap();
    goal.iter()
        .position(|&m| state.formula(m).to_string() == text)
        .unwrap_or_else(|| {
            let shown: Vec<String> = goal.iter().map(|&m| state.formula(m).to_string()).collect();
            panic!("no {text} among {shown:?}")
        })
}

/// Applies a step on the formula written `text` of the session's only
/// open goal and returns the goals it opens.
fn step(state: &mut Interactive, text: &str, rule: Rule, witness: Option<&str>) -> Vec<GoalId> {
    let goal = state.goals().next().expect("an open goal");
    let mut step = Step::new(at(state, goal, text), rule);
    if let Some(witness) = witness {
        step = step.with_witness(state.witness(witness).unwrap());
    }
    state.apply(goal, &step).unwrap()
}

/// A session proves a first-order sequent with the four quantifier rules:
/// `∃L` and `∀R` with fresh eigenvariables, `∀L` with a witness left open
/// and `∃R` with one given as text; the axiom binds the open witness,
/// undo unbinds it, the session's JSON reads back as itself before and
/// after, and the finished proof passes the checker.
#[test]
fn a_session_proves_with_the_quantifier_rules() {
    let sequent: Sequent = "exists y. forall x. r(x, y) |- forall x. exists y. r(x, y)"
        .parse()
        .unwrap();
    let mode = Mode::INTUITIONISTIC;
    let mut state = Interactive::new(&sequent, mode).unwrap();
    let root = state.goals().next().unwrap();
    let rules = state
        .rules(root, at(&state, root, "∃y. (∀x. r(x, y))"))
        .unwrap();
    assert_eq!(rules[0].rule.to_string(), "∃L");
    // A witness only where the rule takes one.
    let given = Witness::Term(vec![TermNode::Eigen(Eigen::new(0))]);
    let error = state
        .apply(
            root,
            &Step::new(at(&state, root, "∃y. (∀x. r(x, y))"), Rule::Forall).with_witness(given),
        )
        .unwrap_err();
    assert!(matches!(error, StepError::NoWitness { .. }), "{error}");
    step(&mut state, "∃y. (∀x. r(x, y))", Rule::Forall, None);
    step(&mut state, "∀x. (∃y. r(x, y))", Rule::Forall, None);
    let goal = state.goals().next().unwrap();
    let rules = state.rules(goal, at(&state, goal, "∀x. r(x, a)")).unwrap();
    assert_eq!(
        (rules[0].rule.to_string(), rules[0].needs),
        ("∀L".to_owned(), Needs::Witness)
    );
    // A witness of the sequent's terms.
    assert!(matches!(
        state.witness("r(a)"),
        Err(StepError::Witness { .. })
    ));
    assert!(matches!(state.witness("c"), Err(StepError::Witness { .. })));
    step(&mut state, "∀x. r(x, a)", Rule::Exists, None);
    step(&mut state, "∃y. r(b, y)", Rule::Exists, Some("a"));
    let goal = state.goals().next().unwrap();
    let open: Vec<String> = state
        .goal(goal)
        .unwrap()
        .iter()
        .map(|&m| state.formula(m).to_string())
        .collect();
    assert_eq!(open, ["r($1, a)", "r(b, a)"]);
    // An open witness's name reads back as nothing, in either surface.
    for text in ["|- r($1, a)", "|- $1", "r($1, a) |- r(b, a)"] {
        assert!(text.parse::<Sequent>().is_err(), "{text}");
        assert!(text.parse::<linlog::IllSequent>().is_err(), "{text}");
    }
    let json = serde_json::to_string(&state).unwrap();
    let back: Interactive = serde_json::from_str(&json).unwrap();
    assert_eq!(serde_json::to_string(&back).unwrap(), json);
    state.apply(goal, &Step::new(0, Rule::Ax)).unwrap();
    assert!(state.is_complete());
    let derivation = state.derivation().unwrap().to_string();
    assert!(derivation.contains("r(b, a) ⊢ r(b, a)"), "{derivation}");
    let json = serde_json::to_string(&state).unwrap();
    assert!(json.contains(r#""bindings":"#), "{json}");
    let back: Interactive = serde_json::from_str(&json).unwrap();
    assert_eq!(serde_json::to_string(&back).unwrap(), json);
    let proof = back.proof(&Limits::default(), |_| false).unwrap();
    assert_eq!(proof.check(mode), Ok(()));
    assert_eq!(state.undo(), Some(goal));
    let open: Vec<String> = state
        .goal(goal)
        .unwrap()
        .iter()
        .map(|&m| state.formula(m).to_string())
        .collect();
    assert_eq!(open, ["r($1, a)", "r(b, a)"]);
    state.apply(goal, &Step::new(0, Rule::Ax)).unwrap();
    let proof = state.proof(&Limits::default(), |_| false).unwrap();
    assert_eq!(proof.check(mode), Ok(()));
}

/// The eigenvariable condition holds in a session: an open witness made
/// before a `∀` step's eigenvariable is never bound to it, and a witness
/// given names only the eigenvariables of the `∀` steps below its goal.
#[test]
fn a_session_keeps_the_eigenvariable_condition() {
    let sequent: Sequent = "forall x. exists y. r(x, y) |- exists y. forall x. r(x, y)"
        .parse()
        .unwrap();
    let mut state = Interactive::new(&sequent, Mode::INTUITIONISTIC).unwrap();
    step(&mut state, "∃y. (∀x. r(x, y))", Rule::Exists, None);
    step(&mut state, "∀x. r(x, $1)", Rule::Forall, None);
    step(&mut state, "∀x. (∃y. r(x, y))", Rule::Exists, Some("a"));
    step(&mut state, "∃y. r(a, y)", Rule::Forall, None);
    let goal = state.goals().next().unwrap();
    let error = state.apply(goal, &Step::new(0, Rule::Ax)).unwrap_err();
    assert!(
        matches!(&error, StepError::Eigenvariable { name, .. } if name == "b"),
        "{error}"
    );
    let sequent: Sequent = "forall x. p(x), forall x. q(x) |- (forall x. p(x)) * (forall x. q(x))"
        .parse()
        .unwrap();
    let mut state = Interactive::new(&sequent, Mode::INTUITIONISTIC).unwrap();
    let root = state.goals().next().unwrap();
    let tensor = at(&state, root, "(∀x. p(x)) ⊗ (∀x. q(x))");
    let left = at(&state, root, "∀x. p(x)");
    let goals = state
        .apply(root, &Step::new(tensor, Rule::Tensor).left(&[left]))
        .unwrap();
    // The factor, after the hypothesis of the same formula.
    let factor = state.goal(goals[0]).unwrap().len() - 1;
    let opened = state
        .apply(goals[0], &Step::new(factor, Rule::Forall))
        .unwrap();
    assert_eq!(opened.len(), 1);
    // `a` is the left premise's: the right one is not in its scope.
    let error = state
        .apply(
            goals[1],
            &Step::new(at(&state, goals[1], "∀x. q(x)"), Rule::Exists)
                .with_witness(state.witness("a").unwrap()),
        )
        .unwrap_err();
    assert!(
        matches!(&error, StepError::Eigenvariable { name, .. } if name == "a"),
        "{error}"
    );
}

/// The search closes a goal of a session: an open witness it fills is
/// bound, as a later axiom would bind it, and undo unbinds it; a goal
/// with eigenvariables is proved with them as the session's; a goal whose
/// open witness was made before an eigenvariable it would need is not
/// proved; every finished proof passes the checker.
#[test]
fn the_search_closes_a_session_goal() {
    let (options, view, limits) = (
        Options::default(),
        linlog::ViewOptions::default(),
        Limits::default(),
    );
    let mode = Mode::INTUITIONISTIC;
    let sequent: Sequent = "forall x. p(x) |- p(f(a))".parse().unwrap();
    let mut state = Interactive::new(&sequent, mode).unwrap();
    step(&mut state, "∀x. p(x)", Rule::Exists, None);
    let goal = state.goals().next().unwrap();
    let closed = state
        .close(goal, &options, &view, &limits, |_| false)
        .unwrap();
    assert!(matches!(closed.outcome.verdict, Verdict::Proved(_)));
    assert_eq!(closed.grafted, Ok(()));
    assert!(state.is_complete());
    let derivation = state.derivation().unwrap().to_string();
    assert!(derivation.contains("p(f(a)) ⊢ p(f(a))"), "{derivation}");
    let json = serde_json::to_string(&state).unwrap();
    let back: Interactive = serde_json::from_str(&json).unwrap();
    assert_eq!(serde_json::to_string(&back).unwrap(), json);
    assert_eq!(state.proof(&limits, |_| false).unwrap().check(mode), Ok(()));
    assert_eq!(state.undo(), Some(goal));
    let open: Vec<String> = state
        .goal(goal)
        .unwrap()
        .iter()
        .map(|&m| state.formula(m).to_string())
        .collect();
    assert_eq!(open, ["p(f(a))", "p($1)"]);

    let sequent: Sequent = "exists y. forall x. r(x, y) |- forall x. exists y. r(x, y)"
        .parse()
        .unwrap();
    let mut state = Interactive::new(&sequent, mode).unwrap();
    step(&mut state, "∃y. (∀x. r(x, y))", Rule::Forall, None);
    step(&mut state, "∀x. (∃y. r(x, y))", Rule::Forall, None);
    let goal = state.goals().next().unwrap();
    let closed = state
        .close(goal, &options, &view, &limits, |_| false)
        .unwrap();
    assert!(matches!(closed.outcome.verdict, Verdict::Proved(_)));
    assert!(state.is_complete());
    assert_eq!(state.proof(&limits, |_| false).unwrap().check(mode), Ok(()));

    let sequent: Sequent = "forall x. exists y. r(x, y) |- exists y. forall x. r(x, y)"
        .parse()
        .unwrap();
    let mut state = Interactive::new(&sequent, mode).unwrap();
    step(&mut state, "∃y. (∀x. r(x, y))", Rule::Exists, None);
    step(&mut state, "∀x. r(x, $1)", Rule::Forall, None);
    let goal = state.goals().next().unwrap();
    let closed = state
        .close(goal, &options, &view, &limits, |_| false)
        .unwrap();
    assert!(
        matches!(closed.outcome.verdict, Verdict::Unprovable(_)),
        "{:?}",
        closed.outcome.verdict
    );
    assert!(!state.is_complete());

    // A goal without a binder or an open witness left, and the empty goal
    // of a classical mode, are searched as sequents of their own too.
    let sequent: Sequent = "forall x. p(x) |- p(c)".parse().unwrap();
    let mut state = Interactive::new(&sequent, mode).unwrap();
    step(&mut state, "∀x. p(x)", Rule::Exists, Some("c"));
    let goal = state.goals().next().unwrap();
    let closed = state
        .close(goal, &options, &view, &limits, |_| false)
        .unwrap();
    assert!(matches!(closed.outcome.verdict, Verdict::Proved(_)));
    assert_eq!(state.proof(&limits, |_| false).unwrap().check(mode), Ok(()));
    let sequent: Sequent = "|- (exists x. p(x)) * bot".parse().unwrap();
    let mut state = Interactive::new(&sequent, Mode::CLASSICAL).unwrap();
    let goals = step(&mut state, "(∃x. p(x)) ⊗ ⊥", Rule::Tensor, None);
    let bot = state.apply(goals[1], &Step::new(0, Rule::Bot)).unwrap();
    let closed = state
        .close(bot[0], &options, &view, &limits, |_| false)
        .unwrap();
    assert!(matches!(closed.outcome.verdict, Verdict::Unprovable(_)));
}
/// A witness nested 100 000 deep goes through a session without
/// recursion: built flat, printed, written for the search and decided;
/// and a session read from JSON holds no more table entries than
/// `limits.terms`.
#[test]
fn a_session_holds_deep_witnesses_and_bounds_its_table() {
    let sequent: Sequent = "forall x. p(x) |- p(f(c))".parse().unwrap();
    let symbol = |name: &str| {
        let index = sequent
            .symbols()
            .iter()
            .position(|(n, _)| n == name)
            .unwrap();
        linlog::sequents::Symbol::new(u32::try_from(index).unwrap())
    };
    let mut nodes = vec![TermNode::App(symbol("c"), vec![])];
    for i in 0..100_000u32 {
        nodes.push(TermNode::App(symbol("f"), vec![i]));
    }
    let mut state = Interactive::new(&sequent, Mode::INTUITIONISTIC).unwrap();
    let root = state.goals().next().unwrap();
    let position = at(&state, root, "∀x. p(x)");
    let deep = Step::new(position, Rule::Exists).with_witness(Witness::Term(nodes));
    let goals = state.apply(root, &deep).unwrap();
    let deep = state.formula(state.goal(goals[0]).unwrap()[1]).to_string();
    // `p(`, a hundred thousand `f(`, `c` and as many brackets and one.
    assert_eq!(deep.len(), 2 + 200_000 + 1 + 100_001);
    assert!(deep.starts_with("p(f(f(") && deep.contains("f(f(c))"));
    let view = linlog::ViewOptions::default();
    let closed = state
        .close(
            goals[0],
            &Options::default(),
            &view,
            &Limits::default(),
            |_| false,
        )
        .unwrap();
    assert!(matches!(closed.outcome.verdict, Verdict::Unprovable(_)));
    assert_eq!(state.undo(), Some(root));
    step(&mut state, "∀x. p(x)", Rule::Exists, Some("f(c)"));
    let json = serde_json::to_string(&state).unwrap();
    // A member and a frame: undo dropped the deep witness's entries.
    let limits = Limits::default().with_terms(Some(1));
    let error = <Interactive as linlog::wire::Readable>::read(
        &mut serde_json::Deserializer::from_str(&json),
        &limits,
    )
    .unwrap_err();
    assert!(error.to_string().contains("terms"), "{error}");
}

/// `undo` takes back what its step made: the open witnesses `close`
/// bound, and the eigenvariables, constants and entries the step added,
/// so that the session reads as it did before the step.
#[test]
fn undo_drops_what_its_step_made() {
    let sequent: Sequent = "|- exists x. exists y. (p(x) -o p(y))".parse().unwrap();
    let mut state = Interactive::new(&sequent, Mode::CLASSICAL).unwrap();
    step(&mut state, "∃x. (∃y. (~p(x) ⅋ p(y)))", Rule::Exists, None);
    let before = serde_json::to_string(&state).unwrap();
    let goal = state.goals().next().unwrap();
    let (options, view, limits) = (
        Options::default(),
        linlog::ViewOptions::default(),
        Limits::default(),
    );
    for _ in 0..3 {
        let closed = state
            .close(goal, &options, &view, &limits, |_| false)
            .unwrap();
        assert!(matches!(closed.outcome.verdict, Verdict::Proved(_)));
        assert_eq!(state.undo(), Some(goal));
    }
    assert_eq!(serde_json::to_string(&state).unwrap(), before);
    assert!(matches!(state.witness("a"), Err(StepError::Witness { .. })));
}

/// A session refuses what none of its steps could make: a witness that
/// unfolds past `limits.terms` (a term of shared halves, a hundred
/// thousand nodes written out to 2⁴⁰), and in its JSON a binding at a
/// step that closes no goal and an open witness whose level admits an
/// eigenvariable made after it.
#[test]
fn a_session_refuses_what_no_step_could_make() {
    let sequent: Sequent = "forall x. p(x) |- p(g(c, c))".parse().unwrap();
    let index = |name: &str| {
        let i = sequent
            .symbols()
            .iter()
            .position(|(n, _)| n == name)
            .unwrap();
        linlog::sequents::Symbol::new(u32::try_from(i).unwrap())
    };
    let mut nodes = vec![TermNode::App(index("c"), vec![])];
    for i in 0..40u32 {
        nodes.push(TermNode::App(index("g"), vec![i, i]));
    }
    let mut state = Interactive::new(&sequent, Mode::INTUITIONISTIC).unwrap();
    let root = state.goals().next().unwrap();
    let position = at(&state, root, "∀x. p(x)");
    let error = state
        .apply(
            root,
            &Step::new(position, Rule::Exists).with_witness(Witness::Term(nodes)),
        )
        .unwrap_err();
    assert!(
        matches!(&error, StepError::Witness { reason, .. } if reason.contains("unfold")),
        "{error}"
    );
    assert_eq!(state.steps(), 0);

    // The session measures what its reader measures: under a limit of 4
    // a witness of 11 terms is refused, and a session within the limit
    // reads back under it.
    let sequent: Sequent = "|- exists x. p(x), ~p(a), q(g(b))".parse().unwrap();
    let limits = Limits::default().with_terms(Some(4));
    let mut state = Interactive::within(&sequent, Mode::CLASSICAL, &limits).unwrap();
    let root = state.goals().next().unwrap();
    let deep = state.witness("g(g(g(g(g(g(g(g(g(g(a))))))))))").unwrap();
    let error = state
        .apply(root, &Step::new(0, Rule::Exists).with_witness(deep))
        .unwrap_err();
    assert!(matches!(error, StepError::Witness { .. }), "{error}");
    let a = state.witness("a").unwrap();
    state
        .apply(root, &Step::new(0, Rule::Exists).with_witness(a))
        .unwrap();
    let json = serde_json::to_string(&state).unwrap();
    <Interactive as linlog::wire::Readable>::read(
        &mut serde_json::Deserializer::from_str(&json),
        &limits,
    )
    .unwrap();

    // A binding moved onto the `∀L` step that opened its witness.
    let sequent: Sequent = "forall x. p(x) |- p(c)".parse().unwrap();
    let mut state = Interactive::new(&sequent, Mode::INTUITIONISTIC).unwrap();
    let goals = step(&mut state, "∀x. p(x)", Rule::Exists, None);
    state.apply(goals[0], &Step::new(0, Rule::Ax)).unwrap();
    let json = serde_json::to_string(&state).unwrap();
    let forged = json.replace(r#""bindings":[[0,1,1]]"#, r#""bindings":[[0,1,0]]"#);
    assert_ne!(forged, json, "{json}");
    assert!(serde_json::from_str::<Interactive>(&json).is_ok());
    let error = serde_json::from_str::<Interactive>(&forged).unwrap_err();
    assert!(error.to_string().contains("binding"), "{error}");

    // An open witness made before the eigenvariables `a` and `b`, given
    // a level that admits them.
    let sequent: Sequent = "forall x. exists y. r(x, y) |- exists y. forall x. r(x, y)"
        .parse()
        .unwrap();
    let mut state = Interactive::new(&sequent, Mode::INTUITIONISTIC).unwrap();
    step(&mut state, "∃y. (∀x. r(x, y))", Rule::Exists, None);
    step(&mut state, "∀x. r(x, $1)", Rule::Forall, None);
    step(&mut state, "∀x. (∃y. r(x, y))", Rule::Exists, Some("a"));
    step(&mut state, "∃y. r(a, y)", Rule::Forall, None);
    let json = serde_json::to_string(&state).unwrap();
    let forged = json.replace(r#""metas":[0]"#, r#""metas":[2]"#);
    assert_ne!(forged, json, "{json}");
    assert!(serde_json::from_str::<Interactive>(&json).is_ok());
    let error = serde_json::from_str::<Interactive>(&forged).unwrap_err();
    assert!(error.to_string().contains("level"), "{error}");
}
