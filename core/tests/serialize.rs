// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The JSON forms of a sequent, of a proof, of a proof net and of a proof
//! in progress, through the public API. The formats are interchange
//! formats, so the strings pinned here must not change.

#![cfg(all(feature = "parse", feature = "serialize"))]

use linlog::search::{Engine, Options, prove, prove_within};
use linlog::{
    Branch, Criterion, Forest, Fragment, GoalId, Interactive, Limits, MellStructure, Member, Mode,
    Named, NetError, Node, NodeId, OccId, Order, Placement, Proof, ProofStructure, Rule, Sequent,
    Step, VertexId, ViewOptions,
};
use linlog::{CutFormula, InputOptions};

/// A sequent's JSON as another form nests it: without `version`.
fn nested(s: &Sequent) -> String {
    serde_json::to_string(s)
        .unwrap()
        .replacen(r#""version":1,"#, "", 1)
}

/// A sequent survives a trip through JSON unchanged.
#[test]
fn round_trip() {
    for input in [
        "|-",
        "A |- A",
        "A * B, C par D |- A & B, C + D",
        "!A, ?B |- ~A, B^",
        "A -o B -o C, (A -o B) -o C |- (A * B) + (0 & top)",
        "A, A -o B |- B, B * ~A",
    ] {
        let s: Sequent = input.parse().unwrap();
        let back: Sequent = serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
        assert_eq!(back, s, "{input:?}");
        assert_eq!(back.to_string(), s.to_string(), "{input:?}");
    }
}

/// A name the atom dictionary repeats is one atom: the sequent reads as the
/// one with the name once, and is proved like it.
#[test]
fn repeated_atom_name_is_one_atom() {
    let json = r#"{"version":1,"terms":[{"D":0},{"V":1}],"roots":[0,1],"atoms":["A","A"],"antecedents":1}"#;
    let s: Sequent = serde_json::from_str(json).unwrap();
    assert_eq!(s, "A |- A".parse().unwrap());
    let outcome = prove(&s, Mode::CLASSICAL, &Options::default()).unwrap();
    assert!(matches!(outcome.verdict, linlog::Verdict::Proved(_)));
}

/// An atom's name is a variable of the text syntax, so that the sequent
/// written as text reads back as itself: an atom named `top` printed as
/// the unit, and `a b` as no sequent.
#[test]
fn atom_names_are_variables_of_the_text() {
    for name in ["top", "par", "forall", "a b", "", "1", "a\u{7}"] {
        let json = format!(
            r#"{{"version":1,"terms":[{{"V":0}}],"roots":[0],"atoms":[{}]}}"#,
            serde_json::to_string(name).unwrap()
        );
        let read = linlog::wire::upgrade::<Sequent, _>(
            &mut serde_json::Deserializer::from_str(&json),
            &linlog::Limits::default(),
        );
        assert!(
            matches!(&read, Err(linlog::Error::AtomName { name: n, .. }) if n == name),
            "{name:?}: {read:?}"
        );
    }
}

/// A name is read in NFC, so a dictionary that spells one name composed
/// and decomposed has one atom.
#[test]
fn atom_names_are_composed() {
    let json =
        r#"{"version":1,"terms":[{"D":0},{"V":1}],"roots":[0,1],"atoms":["e\u0301","\u00e9"]}"#;
    let s: Sequent = serde_json::from_str(json).unwrap();
    assert_eq!(s.atom_names(), ["\u{e9}"]);
    assert_eq!(s.to_string(), "⊢ ~\u{e9}, \u{e9}");
}

/// JSON whose arena breaks an invariant is rejected on deserialization.
#[test]
fn broken_arena_is_rejected() {
    for json in [
        // A subterm index that does not precede its parent.
        r#"{"version":1,"terms":[{"⊗":[1,0]},"1"],"roots":[0],"atoms":[]}"#,
        // A root outside the arena.
        r#"{"version":1,"terms":["1"],"roots":[1],"atoms":[]}"#,
        // An atom outside the dictionary.
        r#"{"version":1,"terms":[{"V":0}],"roots":[0],"atoms":[]}"#,
        // More formulas left of `⊢` than the sequent has.
        r#"{"version":1,"terms":["1"],"roots":[0],"atoms":[],"antecedents":2}"#,
    ] {
        assert!(serde_json::from_str::<Sequent>(json).is_err(), "{json}");
    }
}

/// Builds a proof of the sequent `input` parses to, with the last node as
/// the root.
fn proof(input: &str, nodes: Vec<Node>) -> Proof {
    let s: Sequent = input.parse().unwrap();
    let root = NodeId::new(nodes.len() as u32 - 1);
    Proof::new(Forest::new(&s).unwrap(), nodes, root).unwrap()
}

/// A proof serializes as its sequent, in the sequent's own format, and its
/// nodes, one object per node with the rule's short tag.
#[test]
fn proof_json_format() {
    use Node::*;
    let (o, n) = (OccId::new, NodeId::new);
    for (input, nodes, expected) in [
        // ⊢ ~A, A ⊗ ~B, B: 0 ~A, 1 ⊗, 2 A, 3 ~B, 4 B
        (
            "A, A -o B |- B",
            vec![
                Ax(o(0).into(), o(2).into()),
                Ax(o(3).into(), o(4).into()),
                Tensor(Member::from(o(1)), n(0), n(1)),
            ],
            r#"[{"ax":[0,2]},{"ax":[3,4]},{"⊗":[1,0,1]}]"#,
        ),
        // ⊢ A & ⊤, ~A ⊕ ⊥: 0 &, 1 A, 2 ⊤, 3 ⊕, 4 ~A, 5 ⊥
        (
            "|- A & top, ~A + bot",
            vec![
                Ax(o(1).into(), o(4).into()),
                Plus(Member::from(o(3)), Branch::Left, n(0)),
                Top(Member::from(o(2))),
                With(Member::from(o(0)), n(1), n(2)),
            ],
            r#"[{"ax":[1,4]},{"⊕₁":[3,0]},{"⊤":2},{"&":[0,1,2]}]"#,
        ),
        // ⊢ ?~A, ?(A ⊗ ~B), !B: 0 ?, 1 ~A, 2 ?, 3 ⊗, 4 A, 5 ~B, 6 !, 7 B
        (
            "!A, !(A -o B) |- !B",
            vec![
                Ax(o(1).into(), o(4).into()),
                Copy(Member::from(o(1)), n(0)),
                Ax(o(5).into(), o(7).into()),
                Tensor(Member::from(o(3)), n(1), n(2)),
                Copy(Member::from(o(3)), n(3)),
                Bang(Member::from(o(6)), n(4)),
                Quest(Member::from(o(2)), n(5)),
                Quest(Member::from(o(0)), n(6)),
            ],
            r#"[{"ax":[1,4]},{"copy":[1,0]},{"ax":[5,7]},{"⊗":[3,1,2]},{"copy":[3,3]},{"!":[6,4]},{"?":[2,5]},{"?":[0,6]}]"#,
        ),
        // ⊢ ~A, ~B, A, B, 1, ⊥ with weakening and Mix: 0 ~A, 1 ~B, 2 A, 3 B,
        // 4 1, 5 ⊥
        (
            "A, B |- A, B, 1, bot",
            vec![
                Ax(o(0).into(), o(2).into()),
                Weaken(Member::from(o(1)), n(0)),
                One(Member::from(o(4))),
                Bot(Member::from(o(5)), n(2)),
                Mix(n(1), n(3)),
                Weaken(Member::from(o(3)), n(4)),
            ],
            r#"[{"ax":[0,2]},{"wk":[1,0]},{"1":4},{"⊥":[5,2]},{"mix":[1,3]},{"wk":[3,4]}]"#,
        ),
        // ⊢ A ⊕ B, ~A ⅋ ~B: 0 ⊕, 1 A, 2 B, 3 ⅋, 4 ~A, 5 ~B
        (
            "|- A + B, ~A par ~B",
            vec![
                Ax(o(2).into(), o(5).into()),
                Plus(Member::from(o(0)), Branch::Right, n(0)),
                Weaken(Member::from(o(4)), n(1)),
                Par(Member::from(o(3)), n(2)),
            ],
            r#"[{"ax":[2,5]},{"⊕₂":[0,0]},{"wk":[4,1]},{"⅋":[3,2]}]"#,
        ),
    ] {
        let p = proof(input, nodes);
        let sequent = nested(p.sequent());
        assert_eq!(
            serde_json::to_string(&p).unwrap(),
            format!(r#"{{"version":1,"sequent":{sequent},"nodes":{expected}}}"#),
            "{input:?}"
        );
    }
}

/// A proof survives a trip through JSON with its sequent and nodes intact,
/// and still checks.
#[test]
fn proof_round_trip() {
    use Node::*;
    let (o, n) = (OccId::new, NodeId::new);
    let p = proof(
        "!A, !(A -o B) |- !B, top",
        vec![
            Ax(o(1).into(), o(4).into()),
            Copy(Member::from(o(1)), n(0)),
            Ax(o(5).into(), o(7).into()),
            Tensor(Member::from(o(3)), n(1), n(2)),
            Copy(Member::from(o(3)), n(3)),
            Bang(Member::from(o(6)), n(4)),
            Quest(Member::from(o(2)), n(5)),
            Quest(Member::from(o(0)), n(6)),
            Top(Member::from(o(8))),
            Mix(n(7), n(8)),
        ],
    );
    let back: Proof = serde_json::from_str(&serde_json::to_string(&p).unwrap()).unwrap();
    assert_eq!(back.sequent(), p.sequent());
    assert_eq!(back.nodes(), p.nodes());
    assert_eq!(back.check(Mode::CLASSICAL.with_mix()), Ok(()));
    assert_eq!(
        back.derivation().unwrap().to_string(),
        p.derivation().unwrap().to_string()
    );
}

/// A proof with a cut is written at level 6: its sequent's cut formulas
/// as `cuts`, each the term of `A` (the arena holds its dual, which the
/// reader finds again), and the cut as `{"cut": [a, l, r]}`; it reads back
/// as it was and checks.
#[test]
fn proof_with_a_cut_json_format_and_round_trip() {
    use Node::*;
    let (o, n) = (|x| Member::from(OccId::new(x)), NodeId::new);
    // A ⊢ A by a cut on A: 0 ~A, 1 A, the cut formula 2 A and its dual 3 ~A.
    let mut s: Sequent = "A |- A".parse().unwrap();
    s.add_cut("|- A".parse().unwrap()).unwrap();
    let nodes = vec![Ax(o(0), o(2)), Ax(o(1), o(3)), Cut(o(2), n(0), n(1))];
    let p = Proof::new(Forest::new(&s).unwrap(), nodes, n(2)).unwrap();
    let json = serde_json::to_string(&p).unwrap();
    assert_eq!(
        json,
        r#"{"version":6,"sequent":{"terms":[{"D":0},{"V":0},{"V":0}],"roots":[0,1],"atoms":["A"],"antecedents":1,"cuts":[2]},"nodes":[{"ax":[0,2]},{"ax":[1,3]},{"cut":[2,0,1]}]}"#
    );
    let back: Proof = serde_json::from_str(&json).unwrap();
    assert_eq!(back.sequent(), p.sequent());
    assert_eq!(back.nodes(), p.nodes());
    assert_eq!(serde_json::to_string(&back).unwrap(), json);
    assert_eq!(back.check(Mode::INTUITIONISTIC), Ok(()));
    // Without its cut formulas the sequent reads at level 1, as before.
    assert!(nested(&"A |- A".parse().unwrap()).find("cuts").is_none());
}

/// A sequent with many cut formulas over a long arena reads in time of
/// the document's size: each term's dual is built once, however many cut
/// formulas reach it (20 000 cuts of a chain of 20 000 terms took seconds
/// when every cut formula went over the arena again).
#[test]
fn many_cut_formulas_read_in_linear_time() {
    const N: usize = 20_000;
    let mut terms = vec![r#"{"V":0}"#.to_owned()];
    terms.extend((1..N).map(|i| format!(r#"{{"⊗":[{},0]}}"#, i - 1)));
    let cuts = vec!["0"; N].join(",");
    let json = format!(
        r#"{{"version":6,"terms":[{}],"roots":[{}],"atoms":["a"],"cuts":[{cuts},{}]}}"#,
        terms.join(","),
        N - 1,
        N - 1
    );
    let sequent: Sequent = serde_json::from_str(&json).unwrap();
    assert_eq!(sequent.cuts().len(), N + 1);
    // The dual of the chain, once: one new term per term.
    assert_eq!(sequent.terms().len(), 2 * N);
}

/// JSON whose nodes point outside the forest or the arena, or at a later
/// node, is rejected on deserialization; whether the proof is correct is
/// left to the checker.
#[test]
fn broken_proof_is_rejected() {
    let sequent = r#"{"version":1,"terms":[{"D":0},{"V":0}],"roots":[0,1],"atoms":["A"]}"#;
    for proof in [
        // No node at all.
        r#"[]"#,
        // An occurrence outside the forest.
        r#"[{"ax":[0,2]}]"#,
        // A premise that is the node itself.
        r#"[{"⅋":[0,0]}]"#,
        // A premise after the node.
        r#"[{"ax":[0,1]},{"⅋":[0,2]}]"#,
        // An unknown rule.
        r#"[{"cut":[0,1]}]"#,
    ] {
        let json = format!(r#"{{"sequent":{sequent},"nodes":{proof}}}"#);
        assert!(serde_json::from_str::<Proof>(&json).is_err(), "{json}");
    }
    // A wrong proof deserializes and fails the checker.
    let json = format!(r#"{{"sequent":{sequent},"nodes":[{{"ax":[0,0]}}]}}"#);
    let p: Proof = serde_json::from_str(&json).unwrap();
    assert!(p.check(Mode::CLASSICAL).is_err());
}

/// A fragment serializes as its name and reads back as the named fragment,
/// which contains it; a mode serializes as its three flags.
#[test]
fn fragment_and_mode_json_format() {
    for (fragment, name) in [
        (Fragment::EMPTY, "MLL"),
        (Fragment::MLL, "MLL"),
        (Fragment::MULTIPLICATIVE_UNITS, "MLL with units"),
        (Fragment::ADDITIVES, "ALL"),
        (Fragment::MALL, "MALL"),
        (Fragment::MELL, "MELL"),
        (Fragment::LL, "LL"),
    ] {
        let json = serde_json::to_string(&fragment).unwrap();
        assert_eq!(json, format!("{name:?}"));
        let back: Fragment = serde_json::from_str(&json).unwrap();
        assert!(back.contains(fragment) && back.name() == name, "{name}");
    }
    assert!(serde_json::from_str::<Fragment>(r#""mll""#).is_err());
    // The intuitionistic names read back as the classical fragments.
    for (name, fragment) in [("IMLL", Fragment::MLL), ("ILL", Fragment::LL)] {
        let back: Fragment = serde_json::from_str(&format!("{name:?}")).unwrap();
        assert_eq!(back, fragment);
    }

    let mode = Mode::CLASSICAL.with_affine();
    let json = r#""affine""#;
    assert_eq!(serde_json::to_string(&mode).unwrap(), json);
    assert_eq!(serde_json::from_str::<Mode>(json).unwrap(), mode);
}

/// An outcome serializes as its verdict, the reason for `unknown`, the
/// fragment, mode and engine, the statistics, and for `proved` the proof's
/// own keys, so that it deserializes as the proof.
#[test]
fn outcome_json_format() {
    let s: Sequent = "A, A -o B |- B".parse().unwrap();
    let net = Options::default().with_engine(Some(Engine::Net));
    let outcome = prove(&s, Mode::CLASSICAL, &net).unwrap();
    let json = serde_json::to_string(&outcome).unwrap();
    let sequent = nested(&s);
    let head = r#"{"version":1,"linlog":"0.1.0","verdict":"proved","checked":true,"fragment":"MLL","mode":"classical","engine":"net","#;
    assert_eq!(
        json,
        format!(
            r#"{head}"statistics":{{"nodes":2,"memo_hits":0,"memo_entries":0,"splits":0,"links":2,"tests":2,"copies":0,"forward_copies":0,"work":2}},"sequent":{sequent},"nodes":[{{"ax":[0,2]}},{{"ax":[3,4]}},{{"⊗":[1,0,1]}}]}}"#
        )
    );
    let proof: Proof = serde_json::from_str(&json).unwrap();
    assert_eq!(proof.check(Mode::CLASSICAL), Ok(()));
    let focus = Options::default().with_engine(Some(Engine::Focus));
    let json = serde_json::to_string(&prove(&s, Mode::CLASSICAL, &focus).unwrap()).unwrap();
    let head = r#"{"version":1,"linlog":"0.1.0","verdict":"proved","checked":true,"fragment":"MLL","mode":"classical","engine":"focus","#;
    assert_eq!(
        json,
        format!(
            r#"{head}"statistics":{{"nodes":1,"memo_hits":0,"memo_entries":1,"splits":1,"links":0,"tests":0,"copies":0,"forward_copies":0,"work":16}},"sequent":{sequent},"nodes":[{{"ax":[2,0]}},{{"ax":[3,4]}},{{"⊗":[1,0,1]}}]}}"#
        )
    );

    let s: Sequent = "|- A par B, ~A, ~B".parse().unwrap();
    let outcome = prove(&s, Mode::CLASSICAL, &net).unwrap();
    let sequent = nested(&s);
    assert_eq!(
        serde_json::to_string(&outcome).unwrap(),
        format!(
            r#"{{"version":1,"linlog":"0.1.0","verdict":"unprovable","refutation":{{"kind":"equation","formulas":3,"needed":1,"tensors":0,"pars":1,"ones":0,"bottoms":0,"mix":false}},"fragment":"MLL","mode":"classical","engine":"net","statistics":{{"nodes":0,"memo_hits":0,"memo_entries":0,"splits":0,"links":0,"tests":0,"copies":0,"forward_copies":0,"work":0}},"sequent":{sequent}}}"#
        )
    );
    let outcome = prove_within(
        &s,
        Mode::CLASSICAL.with_mix(),
        &net,
        &linlog::Limits::default(),
        |_| true,
    )
    .unwrap();
    assert_eq!(
        serde_json::to_string(&outcome).unwrap(),
        r#"{"version":1,"linlog":"0.1.0","verdict":"unknown","reason":{"kind":"stopped"},"fragment":"MLL","mode":"mix","engine":"net","statistics":{"nodes":1,"memo_hits":0,"memo_entries":0,"splits":0,"links":0,"tests":0,"copies":0,"forward_copies":0,"work":1}}"#
    );
    // The focused engine, whose copy bound this is; the dispatch's Horn
    // engine has none and proves the sequent.
    let s: Sequent = "!A |- A".parse().unwrap();
    let options = Options::default()
        .with_engine(Some(Engine::Focus))
        .with_copies(Some(0))
        .with_forward_copies(0);
    let outcome = prove(&s, Mode::CLASSICAL, &options).unwrap();
    assert_eq!(
        serde_json::to_string(&outcome).unwrap(),
        r#"{"version":1,"linlog":"0.1.0","verdict":"unknown","reason":{"kind":"copy_bound","copies":0},"fragment":"MELL","mode":"classical","engine":"focus","statistics":{"nodes":1,"memo_hits":0,"memo_entries":1,"splits":0,"links":0,"tests":0,"copies":0,"forward_copies":0,"work":16}}"#
    );
}

/// A proof net serializes as its sequent, its Mix flag and its links as
/// pairs of vertex ids, and reads back with its links validated.
#[test]
fn net_json_format_and_round_trip() {
    let v = VertexId::new;
    let s: Sequent = "A, A -o B |- B".parse().unwrap();
    let net = ProofStructure::from_links(
        Forest::new(&s).unwrap(),
        Criterion::MLL,
        &[(v(0), v(2)), (v(3), v(4))],
    )
    .unwrap();
    let json = serde_json::to_string(&net).unwrap();
    let sequent = nested(&s);
    assert_eq!(
        json,
        format!(r#"{{"version":1,"sequent":{sequent},"mix":false,"links":[[0,2],[3,4]]}}"#)
    );
    let back: ProofStructure = serde_json::from_str(&json).unwrap();
    assert_eq!(back.links(), net.links());
    assert_eq!(back.to_string(), net.to_string());
    assert_eq!(back.criterion(), Criterion::MLL);

    // A link that is not between dual literals, or outside the forest, is
    // rejected on deserialization; a wrong net still reads.
    for links in ["[[0,3]]", "[[1,2]]", "[[0,2],[0,2]]", "[[0,9]]"] {
        let json = format!(r#"{{"sequent":{sequent},"mix":true,"links":{links}}}"#);
        assert!(
            serde_json::from_str::<ProofStructure>(&json).is_err(),
            "{links}"
        );
    }
    let json = format!(r#"{{"sequent":{sequent},"mix":true,"links":[[0,2]]}}"#);
    let partial: ProofStructure = serde_json::from_str(&json).unwrap();
    assert!(!partial.is_complete() && partial.criterion().mix);

    // The cyclic order is written at level 2, and its links read back
    // under it: these two cross, ~A[0] — A[3] and ~B[2] — B[4] of
    // ⊢ ~A, ~B ⊗ A, B.
    let s = Sequent::parse_ordered("A -o B, A |- B", Order::Cyclic, &Limits::default()).unwrap();
    let sequent = nested(&s);
    let cyclic = ProofStructure::from_links(
        Forest::new(&s).unwrap(),
        Criterion::CYCLIC,
        &[(v(0), v(3)), (v(2), v(4))],
    )
    .unwrap();
    let json = serde_json::to_string(&cyclic).unwrap();
    assert_eq!(
        json,
        format!(
            r#"{{"version":2,"sequent":{sequent},"mix":false,"order":"cyclic","links":[[0,3],[2,4]]}}"#
        )
    );
    let back: ProofStructure = serde_json::from_str(&json).unwrap();
    assert_eq!(back.criterion(), Criterion::CYCLIC);
    assert!(matches!(
        back.is_correct(|_| false),
        Err(NetError::Crossing { links: [(x, y), (u, w)], .. }) if [x, y, u, w] == [v(0), v(3), v(2), v(4)]
    ));
}

/// A proof structure with boxes serializes at level 4 as its sequent,
/// whether Mix is allowed, its vertices after the roots as `[occurrence,
/// parent, box]`, its links and its weakenings, and reads back the same
/// structure through `wire::upgrade`. A document that is no structure is
/// refused, as `from_parts` refuses its parts: a vertex outside its box or
/// naming nothing, a link out of its box, a graph past the memory bound;
/// and the reader of a proof structure without boxes refuses it rather
/// than read its links as occurrences.
#[test]
fn mell_structure_json_format_and_round_trip() {
    use linlog::{Error, wire};
    let s: Sequent = "!a, !b |- !a".parse().unwrap();
    let outcome = prove(&s, Mode::CLASSICAL, &Options::default()).unwrap();
    let proof = outcome.verdict.proof().unwrap();
    let net =
        MellStructure::from_proof(proof, Criterion::MLL, &Limits::default(), |_| false).unwrap();
    let json = serde_json::to_string(&net).unwrap();
    let sequent = nested(&s);
    assert_eq!(
        json,
        format!(
            r#"{{"version":4,"sequent":{sequent},"mix":false,"vertices":[[1,0,2],[5,2,2]],"links":[[4,3]],"weakenings":[[1,4]]}}"#
        )
    );
    let read = |json: &str, limits: &Limits| {
        wire::upgrade::<MellStructure, _>(&mut serde_json::Deserializer::from_str(json), limits)
    };
    let back = read(&json, &Limits::default()).unwrap();
    assert_eq!(serde_json::to_string(&back).unwrap(), json);
    assert_eq!(back.to_string(), net.to_string());
    assert_eq!(back.is_correct(|_| false), Ok(()));

    let placed = |vertices: &str| {
        let json = json.replace(r#"[[1,0,2],[5,2,2]]"#, vertices);
        match read(&json, &Limits::default()) {
            Err(Error::Net(error)) => match *error {
                NetError::Placement { fault, .. } => fault,
                other => panic!("{vertices}: {other:?}"),
            },
            other => panic!("{vertices}: {other:?}"),
        }
    };
    // The copy outside the box, so that its link leaves it; the premise of
    // `!a` outside its box; an occurrence and a parent that are none.
    assert_eq!(placed("[[1,0,null],[5,2,2]]"), Placement::Link);
    assert_eq!(placed("[[1,0,2],[5,2,null]]"), Placement::Box);
    assert_eq!(placed("[[99,0,2],[5,2,2]]"), Placement::Outside);
    assert_eq!(placed("[[1,4294967295,2],[5,2,2]]"), Placement::Outside);
    let tiny = Limits::default().with_memory_bytes(Some(64));
    assert!(matches!(
        read(&json, &tiny),
        Err(Error::Net(error)) if matches!(*error, NetError::Refused { .. })
    ));
    match wire::upgrade::<ProofStructure, _>(
        &mut serde_json::Deserializer::from_str(&json),
        &Limits::default(),
    ) {
        Err(Error::Json { message, .. }) => assert!(message.contains("with boxes"), "{message}"),
        other => panic!("{other:?}"),
    }
}

/// A session of a classical sequent as written keeps the record under
/// `written` (the occurrences on the input side and the implications),
/// which an older reader ignores, so the level stays 1; it reads back
/// whole, and a record that cannot be the sequent's is refused.
#[test]
fn a_session_keeps_how_its_sequent_was_written() {
    let written: linlog::ClassicalSequent = "A -o B, A |- ~~B".parse().unwrap();
    let state = Interactive::of_classical(&written, Mode::CLASSICAL, &Limits::default()).unwrap();
    let json = serde_json::to_string(&state).unwrap();
    assert!(json.starts_with(r#"{"version":1,"#), "{json}");
    assert!(
        json.ends_with(r#""written":{"inputs":[0,2,3],"implications":[0]}}"#),
        "{json}"
    );
    let back: Interactive = serde_json::from_str(&json).unwrap();
    assert_eq!(back.writing(), Some(written.writing()));
    assert_eq!(serde_json::to_string(&back).unwrap(), json);
    for broken in [r#""implications":[1]"#, r#""implications":[9]"#] {
        let broken = json.replace(r#""implications":[0]"#, broken);
        assert!(
            serde_json::from_str::<Interactive>(&broken).is_err(),
            "{broken}"
        );
    }
    let unsorted = json.replace("[0,2,3]", "[2,0,3]");
    assert!(serde_json::from_str::<Interactive>(&unsorted).is_err());
}

/// A proof in progress serializes as its sequent, mode, inferences (the
/// root first; an open goal is its sequent alone) and the goals its steps
/// closed, and reads back with the same goals, undo history and derivation;
/// an inference whose premises are not what its rule yields is rejected.
/// A session with a cut is written at level 6: its sequent's cut
/// formulas and the cut's `cut`, the occurrence of its formula; it reads
/// back as it was, and `undo` on the session read back takes the cut and
/// its formula away, back to the session as it began.
#[test]
fn a_session_with_a_cut_json_format_and_round_trip() {
    let limits = Limits::default();
    let s: Sequent = "A |- A".parse().unwrap();
    let mut state = Interactive::new(&s, Mode::INTUITIONISTIC).unwrap();
    let fresh = serde_json::to_string(&state).unwrap();
    let formula =
        CutFormula::parse_within("A", Mode::INTUITIONISTIC, &InputOptions::default(), &limits)
            .unwrap();
    let [first, _] = state.cut(GoalId::new(0), &formula, &[0], &limits).unwrap();
    state.apply(first, &Step::new(0, Rule::Ax)).unwrap();
    let json = serde_json::to_string(&state).unwrap();
    assert_eq!(
        json,
        r#"{"version":6,"sequent":{"terms":[{"D":0},{"V":0},{"V":0}],"roots":[0,1],"atoms":["A"],"antecedents":1,"cuts":[2]},"mode":"intuitionistic","inferences":[{"sequent":[0,1],"rule":"cut","premises":[1,2],"cut":2},{"sequent":[0,2],"rule":"ax"},{"sequent":[1,3]}],"history":[0,1]}"#
    );
    let mut back: Interactive = serde_json::from_str(&json).unwrap();
    assert_eq!(serde_json::to_string(&back).unwrap(), json);
    back.undo();
    back.undo();
    assert_eq!(serde_json::to_string(&back).unwrap(), fresh);
    // A cut formula no cut made, or a cut on another formula, is refused.
    for wrong in [
        json.replace(r#""cut":2"#, r#""cut":3"#),
        json.replace(r#""cuts":[2]"#, r#""cuts":[2,1]"#),
    ] {
        let error = serde_json::from_str::<Interactive>(&wrong).unwrap_err();
        assert!(error.to_string().contains("cut"), "{error}");
    }
}

#[test]
fn interactive_json_format_and_round_trip() {
    let s: Sequent = "A, A -o B |- B".parse().unwrap();
    let mut state = Interactive::new(&s, Mode::INTUITIONISTIC).unwrap();
    let root = GoalId::new(0);
    let goals = state
        .apply(
            root,
            &Step::new(1, "⊸L".parse::<Named>().unwrap()).left(&[0]),
        )
        .unwrap();
    state.apply(goals[0], &Step::new(0, Rule::Ax)).unwrap();
    let json = serde_json::to_string(&state).unwrap();
    let sequent = nested(&s);
    assert_eq!(
        json,
        format!(
            r#"{{"version":1,"sequent":{sequent},"mode":"intuitionistic","inferences":[{{"sequent":[0,1,4],"rule":"⊸L","principal":1,"premises":[1,2]}},{{"sequent":[0,2],"rule":"ax"}},{{"sequent":[3,4]}}],"history":[0,1]}}"#
        )
    );
    let back: Interactive = serde_json::from_str(&json).unwrap();
    assert_eq!(serde_json::to_string(&back).unwrap(), json);
    assert_eq!(back.goals().collect::<Vec<_>>(), vec![goals[1]]);
    assert_eq!(
        back.derivation().unwrap().to_string(),
        state.derivation().unwrap().to_string()
    );
    let mut back = back;
    assert_eq!(back.undo(), Some(goals[0]));
    assert_eq!(back.undo(), Some(root));
    assert_eq!(back.undo(), None);

    // The rule names read back from their ASCII spellings too; a premise
    // that the rule does not yield, a rule on the wrong connective, a
    // history naming an open goal, and an occurrence outside the forest are
    // rejected.
    let ok = format!(
        r#"{{"version":1,"sequent":{sequent},"mode":"intuitionistic","inferences":[{{"sequent":[0,1,4],"rule":"-oL","principal":1,"premises":[1,2]}},{{"sequent":[0,2]}},{{"sequent":[3,4]}}],"history":[0]}}"#
    );
    assert!(serde_json::from_str::<Interactive>(&ok).is_ok());
    for (broken, why) in [
        (
            ok.replace(r#""sequent":[0,2]}"#, r#""sequent":[0,4]}"#),
            "wrong premise",
        ),
        (
            ok.replace(r#""rule":"-oL""#, r#""rule":"&R""#),
            "wrong rule",
        ),
        (
            ok.replace(r#""history":[0]"#, r#""history":[1]"#),
            "open goal in the history",
        ),
        (
            ok.replace(r#""sequent":[3,4]}"#, r#""sequent":[3,9]}"#),
            "occurrence outside the forest",
        ),
        (
            ok.replace(r#""premises":[1,2]"#, r#""premises":[1,1]"#),
            "a premise used twice",
        ),
        (
            ok.replace(r#""history":[0]"#, r#""history":[0,0]"#),
            "a step twice in the history",
        ),
        (
            ok.replace(r#""principal":1,"#, ""),
            "a ⊗ without its principal formula",
        ),
    ] {
        assert!(
            serde_json::from_str::<Interactive>(&broken).is_err(),
            "{why}"
        );
    }
    // A history entry inside a later step's subtree did not exist when
    // that step was taken; `undo` would index past the arena.
    let closed = ok
        .replace(r#"{"sequent":[0,2]}"#, r#"{"sequent":[0,2],"rule":"ax"}"#)
        .replace(r#"{"sequent":[3,4]}"#, r#"{"sequent":[3,4],"rule":"ax"}"#);
    let finished = closed.replace(r#""history":[0]"#, r#""history":[0,1,2]"#);
    assert!(serde_json::from_str::<Interactive>(&finished).is_ok());
    let reordered = closed.replace(r#""history":[0]"#, r#""history":[1,0]"#);
    assert!(serde_json::from_str::<Interactive>(&reordered).is_err());

    // States with a chain of steps on one branch, a Mix, a search graft
    // and a repeated formula read back with their history.
    let mix = Mode::CLASSICAL.with_mix();
    let s: Sequent = "|- ~a par ~b, a * b, c, ~c, ?d".parse().unwrap();
    let mut state = Interactive::new(&s, mix).unwrap();
    // The position of the formula printed as `text` in an open goal.
    let at = |state: &Interactive, goal: GoalId, text: &str| {
        state
            .goal(goal)
            .unwrap()
            .iter()
            .position(|&m| state.formula(m).to_string() == text)
            .unwrap()
    };
    let g = state
        .apply(root, &Step::new(at(&state, root, "?d"), Rule::Weakening))
        .unwrap()[0];
    let g = state
        .apply(g, &Step::new(at(&state, g, "~a ⅋ ~b"), Rule::Par))
        .unwrap()[0];
    let goals = state
        .apply(
            g,
            &Step::new(at(&state, g, "c"), Rule::Mix).left(&[at(&state, g, "~c")]),
        )
        .unwrap();
    let g = goals[1];
    let goals = state
        .apply(
            g,
            &Step::new(at(&state, g, "a ⊗ b"), Rule::Tensor).left(&[at(&state, g, "~a")]),
        )
        .unwrap();
    state
        .close(
            goals[0],
            &Options::default(),
            &ViewOptions::default(),
            &linlog::Limits::default(),
            |_| false,
        )
        .unwrap();
    let json = serde_json::to_string(&state).unwrap();
    let mut back: Interactive = serde_json::from_str(&json).unwrap();
    assert_eq!(serde_json::to_string(&back).unwrap(), json);
    assert_eq!(back.steps(), 5);
    let closed = back.close_all(
        &Options::default(),
        &ViewOptions::default(),
        &linlog::Limits::default(),
        |_| false,
    );
    assert_eq!(closed.len(), 2);
    assert_eq!(
        back.proof(&linlog::Limits::default(), |_| false)
            .unwrap()
            .check(mix),
        Ok(())
    );
    while back.undo().is_some() {}
    assert_eq!(back.goals().collect::<Vec<_>>(), vec![root]);
    let s: Sequent = "|- ?(a par ~a)".parse().unwrap();
    let mut state = Interactive::new(&s, mix).unwrap();
    let g = state.apply(root, &Step::new(0, Rule::Contraction)).unwrap()[0];
    let g = state.apply(g, &Step::new(0, Rule::Dereliction)).unwrap()[0];
    let g = state.apply(g, &Step::new(0, Rule::Dereliction)).unwrap()[0];
    state.apply(g, &Step::new(1, Rule::Mix)).unwrap();
    let json = serde_json::to_string(&state).unwrap();
    let back: Interactive = serde_json::from_str(&json).unwrap();
    assert_eq!(serde_json::to_string(&back).unwrap(), json);
}

/// An error is written as a client reads it: the code it branches on, the
/// kind it shows, the message, the setting of a bound that refused the
/// call, and the variant's fields; described, the message has formulas.
#[test]
fn error_json_format() {
    let limits = linlog::Limits::default().with_occurrences(Some(2));
    let error = Sequent::parse_within("|- a * b", &limits).unwrap_err();
    assert_eq!(
        serde_json::to_string(&error).unwrap(),
        format!(
            "{{\"code\":\"too_many_occurrences\",\"kind\":\"limit\",\"message\":\"{error}\",\
             \"setting\":\"limits.occurrences\",\
             \"details\":{{\"kind\":\"occurrences\",\"occurrences\":3,\"limit\":2}}}}"
        )
    );
    let error = "|- a *".parse::<Sequent>().unwrap_err();
    assert_eq!(
        serde_json::to_string(&error).unwrap(),
        format!(
            "{{\"code\":\"parse\",\"kind\":\"malformed\",\"message\":\"{error}\",\
             \"details\":{{\"span\":{{\"start\":6,\"end\":6}},\"span_utf16\":{{\"start\":6,\"end\":6}},\
             \"line\":1,\"column\":7,\"found\":null,\"expected\":[\"a formula\"]}}}}"
        )
    );
    // A proof of `⊢ a, ~a` alone, which concludes less than the sequent.
    let s: Sequent = "|- a * b, ~a, ~b".parse().unwrap();
    let axiom = vec![Node::Ax(Member::new(1), Member::new(3))];
    let proof = Proof::new(Forest::new(&s).unwrap(), axiom, NodeId::new(0)).unwrap();
    let error = linlog::Error::from(proof.check(Mode::CLASSICAL).unwrap_err());
    let json = serde_json::to_string(&error).unwrap();
    assert!(
        json.starts_with(
            "{\"code\":\"invalid_proof\",\"kind\":\"invalid\",\"message\":\"invalid proof: node 0"
        ),
        "{json}"
    );
    assert!(
        json.contains("\"details\":{\"node\":0,\"rule\":{\"ax\":[1,3]}"),
        "{json}"
    );
    let described = serde_json::to_string(&error.describe(&proof)).unwrap();
    assert!(
        described.contains("\"message\":\"invalid proof: node 0 (ax on a, ~a)"),
        "{described}"
    );
    // A switching cycle writes its vertices' ids.
    let s: Sequent = "|- a * b, ~a * ~b".parse().unwrap();
    let links = [
        (VertexId::new(1), VertexId::new(4)),
        (VertexId::new(2), VertexId::new(5)),
    ];
    let net = ProofStructure::from_links(Forest::new(&s).unwrap(), Criterion::MLL, &links).unwrap();
    let error = net.is_correct(|_| false).unwrap_err();
    let linlog::nets::NetError::SwitchingCycle { cycle, .. } = &error else {
        panic!("{error}");
    };
    let ids: Vec<u32> = cycle.iter().map(|v| v.get()).collect();
    assert!(ids.len() > 1, "{ids:?}");
    let json = serde_json::to_value(linlog::Error::from(error.clone())).unwrap();
    assert_eq!(json["details"]["cycle"], serde_json::json!(ids));
}

/// The search options are an object of their fields: a missing key is
/// the default, a misspelt one refused, a choice left to the library
/// `"auto"`, and no bound `null`.
#[test]
fn options_json_format() {
    use linlog::search::{Bias, Cadence, Jobs, Schedule};
    let defaults = serde_json::to_string(&Options::default()).unwrap();
    assert_eq!(
        defaults,
        "{\"engine\":\"auto\",\"fragment\":\"auto\",\"bias\":\"auto\",\"copies\":3,\
         \"forward_copies\":30,\"memo_limit\":1048576,\"test_period\":\"auto\",\"jobs\":1,\
         \"schedule\":\"auto\",\"check\":true,\"classical_work\":67108864}"
    );
    let read: Options = serde_json::from_str("{}").unwrap();
    assert_eq!(read, Options::default());
    let read: Options = serde_json::from_str(
        "{\"engine\":\"two-sided\",\"fragment\":\"IMLL\",\"bias\":\"factors\",\"copies\":null,\
         \"test_period\":7,\"jobs\":\"auto\",\"schedule\":\"turns\"}",
    )
    .unwrap();
    assert_eq!(
        read,
        Options::default()
            .with_engine(Some(Engine::TwoSided))
            .with_fragment(Some(Fragment::MLL))
            .with_bias(Bias::Factors)
            .with_copies(None)
            .with_test_period(Cadence::Every(7))
            .with_jobs(Jobs::Auto)
            .with_schedule(Schedule::Turns)
    );
    let error = serde_json::from_str::<Options>("{\"copy\":3}").unwrap_err();
    assert!(
        error.to_string().contains("unknown field `copy`"),
        "{error}"
    );
    let error = serde_json::from_str::<Options>("{\"engine\":\"fast\"}").unwrap_err();
    assert!(
        error.to_string().contains("unknown engine `fast`"),
        "{error}"
    );
    let error = serde_json::from_str::<Options>("{\"jobs\":\"many\"}").unwrap_err();
    assert!(
        error.to_string().contains("expected a number or \"auto\""),
        "{error}"
    );
}

/// The settings a front end holds read back from their JSON, every value
/// at its default where a key is missing; a format this build lacks is
/// read and dropped, and an unknown one refused.
#[cfg(feature = "latex")]
#[test]
fn settings_json_format() {
    use linlog::Settings;
    let defaults = Settings::default();
    let json = serde_json::to_string(&defaults).unwrap();
    assert_eq!(serde_json::from_str::<Settings>(&json).unwrap(), defaults);
    assert_eq!(serde_json::from_str::<Settings>("{}").unwrap(), defaults);
    let read: Settings = serde_json::from_str(
        "{\"clock\":{\"time_limit_ms\":null},\"limits\":{\"memory_bytes\":1024},\
         \"search\":{\"copies\":5},\"styles\":{\"latex\":{\"form\":\"standalone\"}}}",
    )
    .unwrap();
    assert_eq!(read.clock.time_limit_ms, None);
    assert_eq!(read.clock.pool_after_ms, defaults.clock.pool_after_ms);
    assert_eq!(read.limits.memory_bytes, Some(1024));
    assert_eq!(read.search.copies, Some(5));
    assert_eq!(read.styles.latex.form, linlog::export::Form::Standalone);
    let error = serde_json::from_str::<Settings>("{\"styles\":{\"html\":{}}}").unwrap_err();
    assert!(
        error.to_string().contains("unknown field `html`"),
        "{error}"
    );
}

/// A disproof is a document of its own and reads back as itself, as an
/// unprovable outcome does; a refutation kind the reader does not know is
/// malformed, never read as another, and an atom or an occurrence the
/// sequent lacks is refused.
#[test]
fn disproof_json_format_and_round_trip() {
    use linlog::{Disproof, Error, ErrorKind, Limits, Verdict, wire};
    let s: Sequent = "|- a, a".parse().unwrap();
    let outcome = prove(&s, Mode::CLASSICAL, &Options::default()).unwrap();
    let Verdict::Unprovable(disproof) = &outcome.verdict else {
        panic!("|- a, a is unprovable");
    };
    let json = serde_json::to_string(disproof.as_ref()).unwrap();
    assert_eq!(
        json,
        "{\"version\":1,\"sequent\":{\"terms\":[{\"V\":0}],\"roots\":[0,0],\"atoms\":[\"a\"],\
         \"antecedents\":0},\"mode\":\"classical\",\"refutation\":{\"kind\":\"unbalanced\",\
         \"atom\":0,\"least\":2,\"most\":2}}"
    );
    assert_eq!(read_alike::<Disproof>(&json), json);
    let back: Disproof = serde_json::from_str(&json).unwrap();
    assert_eq!(&back, disproof.as_ref());
    let from_outcome: Disproof =
        serde_json::from_str(&serde_json::to_string(&outcome).unwrap()).unwrap();
    assert_eq!(&from_outcome, disproof.as_ref());
    let read = |json: &str| {
        wire::upgrade::<Disproof, _>(
            &mut serde_json::Deserializer::from_str(json),
            &Limits::default(),
        )
    };
    let unknown = read(&json.replace("unbalanced", "tableau")).unwrap_err();
    assert_eq!(unknown.kind(), ErrorKind::Malformed, "{unknown}");
    let outside = read(&json.replace("\"atom\":0", "\"atom\":3")).unwrap_err();
    assert!(
        matches!(outside, Error::IndexOutOfBounds { index: 3, .. }),
        "{outside}"
    );
}

/// Reads a document through `wire::upgrade`, through `Within` and through
/// plain serde, asserts that the three write back alike, and returns what
/// they write.
fn read_alike<T>(json: &str) -> String
where
    T: linlog::wire::Readable + serde::Serialize + serde::de::DeserializeOwned,
{
    use linlog::wire::{self, Within};
    use serde::de::DeserializeSeed;
    let limits = linlog::Limits::default();
    let document = || serde_json::Deserializer::from_str(json);
    let upgraded: T = wire::upgrade(&mut document(), &limits).unwrap();
    let within: T = Within::<T>::new(&limits)
        .deserialize(&mut document())
        .unwrap();
    let plain: T = serde_json::from_str(json).unwrap();
    let written = serde_json::to_string(&upgraded).unwrap();
    assert_eq!(serde_json::to_string(&within).unwrap(), written);
    assert_eq!(serde_json::to_string(&plain).unwrap(), written);
    written
}

/// The wire level: at level 1 `wire::upgrade` is the identity, each
/// form's document reading back as through `Within` and plain serde and
/// writing back byte for byte; a level above this build's is refused
/// naming it; the names from before the release are refused naming the
/// key they lack; a data form ignores a key it does not know and an
/// options form refuses it; a bound is exact in JavaScript or none; and
/// every reader counts a sequent against the occurrence bound.
#[test]
fn wire_levels() {
    use linlog::limits::Refusal;
    use linlog::{Error, Limits, wire};
    let s: Sequent = "A, A -o B |- B".parse().unwrap();
    let sequent = serde_json::to_string(&s).unwrap();
    let outcome = prove(&s, Mode::INTUITIONISTIC, &Options::default()).unwrap();
    let proof = serde_json::to_string(outcome.verdict.proof().unwrap()).unwrap();
    let v = VertexId::new;
    let net = ProofStructure::from_links(
        Forest::new(&s).unwrap(),
        Criterion::MLL,
        &[(v(0), v(2)), (v(3), v(4))],
    )
    .unwrap();
    let net = serde_json::to_string(&net).unwrap();
    let session = Interactive::new(&s, Mode::INTUITIONISTIC).unwrap();
    let session = serde_json::to_string(&session).unwrap();
    assert_eq!(read_alike::<Sequent>(&sequent), sequent);
    assert_eq!(read_alike::<Proof>(&proof), proof);
    assert_eq!(read_alike::<ProofStructure>(&net), net);
    assert_eq!(read_alike::<Interactive>(&session), session);
    assert!(proof.contains(r#""mode":"intuitionistic""#), "{proof}");

    let read = |json: &str, limits: &Limits| {
        wire::upgrade::<Sequent, _>(&mut serde_json::Deserializer::from_str(json), limits)
    };
    let above = wire::LEVEL + 1;
    let newer = sequent.replacen(r#""version":1"#, &format!(r#""version":{above}"#), 1);
    let error = read(&newer, &Limits::default()).unwrap_err();
    assert!(matches!(
        error,
        Error::Version {
            form: "sequent",
            found,
            supported: wire::LEVEL,
            ..
        } if found == u64::from(above)
    ));
    assert_eq!(error.code(), "unsupported_version");
    let old = r#"{"terms":[{"V":0}],"ids":[0],"var_dict":["A"]}"#;
    match read(old, &Limits::default()) {
        Err(Error::Json { form, message, .. }) => {
            assert_eq!(form, "sequent");
            assert!(message.contains("missing field `roots`"), "{message}");
        }
        other => panic!("{other:?}"),
    }
    let unknown = sequent.replacen('{', r#"{"later":true,"#, 1);
    assert_eq!(read(&unknown, &Limits::default()).unwrap(), s);
    assert!(serde_json::from_str::<Options>(r#"{"later":true}"#).is_err());
    // A bound JavaScript cannot read exactly is written as none, and read
    // as an error.
    let past = Limits::default().with_memory_bytes(Some(1 << 60));
    let written = serde_json::to_string(&past).unwrap();
    assert!(written.contains(r#""memory_bytes":null"#), "{written}");
    let error = serde_json::from_str::<Limits>(r#"{"memory_bytes":9007199254740993}"#);
    assert!(error.unwrap_err().to_string().contains("null lifts it"));
    let most = serde_json::from_str::<Limits>(r#"{"memory_bytes":9007199254740992}"#);
    assert_eq!(most.unwrap().memory_bytes, Some(1 << 53));

    let tiny = Limits::default().with_occurrences(Some(3));
    let refused = |result: Result<(), Error>| {
        matches!(
            result,
            Err(Error::Refused(Refusal::Occurrences { limit: 3, .. }))
        )
    };
    let document = serde_json::Deserializer::from_str;
    assert!(refused(read(&sequent, &tiny).map(drop)));
    let exact = Limits::default().with_occurrences(Some(s.occurrences()));
    assert!(read(&sequent, &exact).is_ok());
    assert!(refused(
        wire::upgrade::<Proof, _>(&mut document(&proof), &tiny).map(drop)
    ));
    assert!(refused(
        wire::upgrade::<ProofStructure, _>(&mut document(&net), &tiny).map(drop)
    ));
    assert!(refused(
        wire::upgrade::<Interactive, _>(&mut document(&session), &tiny).map(drop)
    ));
}

/// A forced engine's refusal is written with its reason, tagged by
/// `kind`, under the error's `details`.
#[test]
fn engine_refusals_json() {
    let net = Options::default().with_engine(Some(Engine::Net));
    let refused = |text: &str, mode: Mode| {
        let error = prove(&text.parse().unwrap(), mode, &net).unwrap_err();
        serde_json::to_value(&error).unwrap()["details"].to_string()
    };
    assert_eq!(
        refused("|- !a, ?~a", Mode::CLASSICAL),
        r#"{"because":{"decides":"MLL","goal":"MELL","kind":"fragment"},"engine":"net"}"#
    );
    assert_eq!(
        refused("|- a, ~a", Mode::CLASSICAL.with_affine()),
        r#"{"because":{"kind":"mode","mode":"affine"},"engine":"net"}"#
    );
}

/// A first-order sequent writes its symbols, its terms and its atoms as
/// predicates at wire level 7, and reads back as itself, one-sided and
/// intuitionistic alike; a ground one writes level 7 too, a propositional
/// one level 1 as before.
#[test]
fn first_order_round_trip() {
    for input in [
        "forall x. p(x, c) -o q(x), p(a, c) |- exists y. q(y)",
        "|- ∀x y. ~r(x, f(y)) ⅋ r(x, f(y)), ∀x. p(x) & ∃z. ~p(z)",
        "p(a) |- p(a)",
        "forall x. p(x), forall y. p(y) |- q",
    ] {
        let s: Sequent = input.parse().unwrap();
        let json = serde_json::to_string(&s).unwrap();
        assert!(json.starts_with(r#"{"version":7,"#), "{input:?}: {json}");
        let back: Sequent = serde_json::from_str(&json).unwrap();
        assert_eq!(back, s, "{input:?}");
        assert_eq!(back.to_string(), s.to_string(), "{input:?}");
    }
    let s: Sequent = "forall x. p(x, c) -o q(x), p(a, c) |- exists y. q(y)"
        .parse()
        .unwrap();
    assert_eq!(
        serde_json::to_string(&s).unwrap(),
        r#"{"version":7,"terms":[{"V":0},{"D":1},{"⊗":[0,1]},{"∃":2},{"D":2},{"V":1},{"∃":5}],"roots":[3,4,6],"atoms":[{"P":[1,[0,1]]},{"P":[2,[0]]},{"P":[1,[2,1]]}],"symbols":[["c",0],["p",2],["q",1],["a",0]],"fo":[{"B":0},{"F":[0,[]]},{"F":[3,[]]}],"binders":["x","y"],"antecedents":2}"#
    );
    let ill: linlog::IllSequent = "forall x. p(x, c) -o q(x), p(a, c) |- exists y. q(y)"
        .parse()
        .unwrap();
    let json = serde_json::to_string(&ill).unwrap();
    assert_eq!(
        json,
        r#"{"version":7,"terms":[{"V":0},{"V":1},{"⊸":[0,1]},{"∀":2},{"V":2},{"∃":1}],"atoms":[{"P":[1,[0,1]]},{"P":[2,[0]]},{"P":[1,[2,1]]}],"symbols":[["c",0],["p",2],["q",1],["a",0]],"fo":[{"B":0},{"F":[0,[]]},{"F":[3,[]]}],"binders":["x","y"],"hypotheses":[3,4],"goal":5}"#
    );
    let back: linlog::IllSequent = serde_json::from_str(&json).unwrap();
    assert_eq!(back.to_string(), ill.to_string());
    assert_eq!(back.sequent().to_string(), ill.sequent().to_string());
    // A propositional sequent's bytes are those of level 1.
    let s: Sequent = "A |- A".parse().unwrap();
    assert!(
        serde_json::to_string(&s)
            .unwrap()
            .starts_with(r#"{"version":1,"#)
    );
    // A binder over nullary atoms alone keeps its atoms as names, and the
    // document writes back as it was read, with or without the names.
    let json =
        r#"{"version":7,"terms":[{"V":0},{"∀":0}],"roots":[1],"atoms":["a"],"binders":["x"]}"#;
    let s: Sequent = serde_json::from_str(json).unwrap();
    assert_eq!(s.to_string(), "⊢ ∀x. a");
    assert_eq!(serde_json::to_string(&s).unwrap(), json);
    let json = r#"{"version":7,"terms":[{"V":0},{"∀":0}],"roots":[1],"atoms":["a"]}"#;
    let s: Sequent = serde_json::from_str(json).unwrap();
    assert_eq!(s.to_string(), "⊢ ∀x. a");
    // The terms an atom unfolds to are counted against the limit before
    // its key is built: shared terms could name a key of exponential
    // length from a short document.
    let mut fo = String::from(r#"{"F":[0,[]]}"#);
    for n in 0..40 {
        fo.push_str(&format!(r#",{{"F":[1,[{n},{n}]]}}"#));
    }
    let json = format!(
        r#"{{"version":7,"terms":[{{"V":0}}],"roots":[0],"atoms":[{{"P":[2,[40]]}}],"symbols":[["a",0],["f",2],["p",1]],"fo":[{fo}]}}"#
    );
    let mut document = serde_json::Deserializer::from_str(&json);
    let error =
        linlog::wire::upgrade::<Sequent, _>(&mut document, &linlog::Limits::default()).unwrap_err();
    assert_eq!(error.code(), "too_many_occurrences", "{error}");
    // What a file cannot hold: a loose variable, a symbol at two arities,
    // an argument past the terms, predicates for some atoms only, a name
    // as predicate and function.
    for (json, code) in [
        (
            r#"{"version":7,"terms":[{"V":0}],"roots":[0],"atoms":[{"P":[0,[5]]}],"symbols":[["p",1]],"fo":[]}"#,
            "index_out_of_bounds",
        ),
        (
            r#"{"version":7,"terms":[{"V":0}],"roots":[0],"atoms":[{"P":[0,[0]]}],"symbols":[["p",1]],"fo":[{"B":0}]}"#,
            "loose_variable",
        ),
        (
            r#"{"version":7,"terms":[{"V":0}],"roots":[0],"atoms":[{"P":[0,[0]]}],"symbols":[["p",2]],"fo":[{"F":[1,[]]}, {"B": 0}]}"#,
            "index_out_of_bounds",
        ),
        (
            r#"{"version":7,"terms":[{"V":0}],"roots":[0],"atoms":[{"P":[0,[0]]}],"symbols":[["p",2],["a",0]],"fo":[{"F":[1,[]]}]}"#,
            "symbol",
        ),
        (
            r#"{"version":7,"terms":[{"V":0}],"roots":[0],"atoms":[{"P":[0,[0]]}],"symbols":[["p",1]],"fo":[{"F":[0,[]]}]}"#,
            "symbol",
        ),
    ] {
        let error = serde_json::from_str::<Sequent>(json)
            .unwrap_err()
            .to_string();
        let mut document = serde_json::Deserializer::from_str(json);
        let error = linlog::wire::upgrade::<Sequent, _>(&mut document, &linlog::Limits::default())
            .map(|_| error.clone())
            .unwrap_err();
        assert_eq!(error.code(), code, "{json}: {error}");
    }
}
