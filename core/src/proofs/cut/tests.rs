// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use super::*;
use crate::proofs::{CutFormula, Interactive};
use crate::{InputOptions, Sequent, ViewOptions};
use std::collections::BTreeMap;

/// The most steps a test takes on one proof before it calls the
/// elimination unfinished.
const MOST_STEPS: usize = 5_000;

/// Never stops.
fn never(_: Progress) -> bool {
    false
}

/// Returns a proof of the sequent `text` in `mode` with a cut on
/// `formula`, typed as the mode writes formulas, whose first premise
/// takes the members at `left`; both premises closed by the search.
fn cut_proof(text: &str, mode: Mode, formula: &str, left: &[usize]) -> Proof {
    let limits = Limits::default();
    let sequent: Sequent = text.parse().unwrap();
    let mut session = Interactive::new(&sequent, mode).unwrap();
    let goal = session.goals().next().unwrap();
    let formula =
        CutFormula::parse_within(formula, mode, &InputOptions::default(), &limits).unwrap();
    session.cut(goal, &formula, left, &limits).unwrap();
    let closed = session.close_all(
        &crate::Options::default(),
        &ViewOptions::default(),
        &limits,
        never,
    );
    assert!(
        closed
            .iter()
            .all(|(_, c)| c.as_ref().is_ok_and(|c| c.grafted.is_ok())),
        "{text}: {closed:?}"
    );
    session.proof(&limits, never).unwrap()
}

/// The proofs with cuts the tests eliminate: the generated ones of
/// every commutative mode and those made here for the exponential cases,
/// each with its mode.
fn proofs() -> Vec<(Proof, Mode)> {
    let mut proofs: Vec<(Proof, Mode)> = crate::proofs::oracle::proofs()
        .into_iter()
        .filter(|(proof, _)| !proof.is_cut_free())
        .collect();
    for mode in [Mode::INTUITIONISTIC, Mode::INTUITIONISTIC.with_affine()] {
        // Two copies of the box's formula: a contraction, then a
        // dereliction; a box's content using it: a box entering a box; a
        // formula the premise never uses: weakening.
        for text in [
            "!a |- a * a",
            "!a |- !a",
            "!a, b |- b",
            "!a, !a -o b |- b * a",
        ] {
            proofs.push((cut_proof(text, mode, "!a", &[0]), mode));
        }
    }
    for mode in [
        Mode::CLASSICAL,
        Mode::CLASSICAL.with_affine(),
        Mode::CLASSICAL.with_mix(),
    ] {
        for (text, rest) in [
            ("|- ?~a, a * a", &[1][..]),
            ("|- ?~a, !a", &[1]),
            ("|- ?~a, ~b, b", &[1, 2]),
        ] {
            proofs.push((cut_proof(text, mode, "!a", &[0]), mode));
            proofs.push((cut_proof(text, mode, "?~a", rest), mode));
        }
    }
    proofs
}

/// Every step of every proof with cuts, taken one at a time by the
/// strategy, makes a proof over the same sequent, cut formulas included,
/// with the same conclusion, that the checker accepts in the proof's
/// mode, every cut of it on a cut formula's tree; each elimination ends
/// cut-free, as `eliminate` does in the same steps; and every case is
/// met, in every commutative mode.
#[test]
fn every_step_keeps_the_conclusion() {
    let proofs = proofs();
    let mut cases: BTreeMap<&str, usize> = BTreeMap::new();
    let mut modes: BTreeMap<String, usize> = BTreeMap::new();
    let (mut nested, mut exponential) = (0, 0);
    for (proof, mode) in &proofs {
        let forest = proof.forest();
        let cuts = proof
            .nodes()
            .iter()
            .filter(|n| matches!(n, Node::Cut(..)))
            .count();
        nested += usize::from(cuts > 1);
        exponential += usize::from(proof.nodes().iter().any(|&node| {
            matches!(node, Node::Cut(a, ..)
                if matches!(forest.kind(a.occ()), crate::Kind::Bang | crate::Kind::Quest))
        }));
        *modes.entry(format!("{mode:?}")).or_default() += 1;
        let mut current = proof.clone();
        let mut steps = Vec::new();
        while !current.is_cut_free() {
            assert!(steps.len() < MOST_STEPS, "no end in {mode:?}: {proof:?}");
            let reduction = step(
                &current,
                *mode,
                None,
                &Options::default(),
                &Limits::default(),
                never,
            )
            .unwrap_or_else(|e| panic!("{e} in {mode:?} at {current:?}"));
            let made = &reduction.proof;
            made.check(*mode).unwrap();
            assert_eq!(made.sequent(), proof.sequent());
            assert_eq!(made.conclusion(), proof.conclusion());
            for node in made.nodes() {
                if let Node::Cut(a, ..) = node {
                    assert!(made.forest().is_cut_formula(a.occ()));
                }
            }
            *cases.entry(reduction.step.case.name()).or_default() += 1;
            steps.push(reduction.step);
            current = reduction.proof;
        }
        let elimination =
            eliminate(proof, *mode, &Options::default(), &Limits::default(), never).unwrap();
        assert_eq!(elimination.end, End::CutFree);
        assert_eq!(elimination.steps, steps);
        assert_eq!(elimination.proof.nodes(), current.nodes());
    }
    for &case in Case::ALL {
        assert!(
            cases.contains_key(case.name()),
            "{} never met: {cases:?}",
            case.name()
        );
    }
    for mode in [
        Mode::CLASSICAL,
        Mode::CLASSICAL.with_affine(),
        Mode::CLASSICAL.with_mix(),
        Mode::INTUITIONISTIC,
        Mode::INTUITIONISTIC.with_affine(),
    ] {
        assert!(
            modes.get(&format!("{mode:?}")).is_some_and(|&n| n >= 10),
            "{modes:?}"
        );
    }
    assert!(nested >= 50 && exponential >= 20, "{nested} {exponential}");
}

/// A step may reduce any cut, not only the strategy's: every cut of every
/// proof, reduced first, makes a proof the checker accepts.
#[test]
fn any_cut_may_be_reduced() {
    for (proof, mode) in proofs() {
        for id in proof
            .ids()
            .filter(|&id| matches!(proof.node(id), Node::Cut(..)))
        {
            let reduction = step(
                &proof,
                mode,
                Some(id),
                &Options::default(),
                &Limits::default(),
                never,
            )
            .unwrap_or_else(|e| panic!("{e} at {id:?} in {mode:?} of {proof:?}"));
            assert_eq!(reduction.step.cut, id);
            reduction.proof.check(mode).unwrap();
        }
        let free = proof
            .ids()
            .find(|&id| !matches!(proof.node(id), Node::Cut(..)));
        let error = step(
            &proof,
            mode,
            free,
            &Options::default(),
            &Limits::default(),
            never,
        );
        assert!(matches!(error, Err(Error::NoCut { .. })));
    }
}

/// In affine mode a weakened cut formula erases the other premise; where
/// that premise absorbs the context through a `⊤` and the place the cut is
/// read at gives it more than the checker derived, each place gets the
/// weakenings of its own context: here a cut that both premises of a `&`
/// read, absorbing `c` in one and `d` in the other.
#[test]
fn an_erasure_weakens_what_each_place_gives() {
    let mode = Mode::CLASSICAL.with_affine();
    let mut sequent: Sequent = "|- ~a, a, top, c & d".parse().unwrap();
    sequent.add_cut("|- b".parse().unwrap()).unwrap();
    let forest = crate::Forest::new(&sequent).unwrap();
    let roots = forest.roots().to_vec();
    let (b, _) = forest.cut_pairs().next().unwrap();
    let o = |x: OccId| Member::from(x);
    let n = NodeId::new;
    let nodes = vec![
        Node::Ax(o(roots[0]), o(roots[1])),
        Node::Weaken(o(b), n(0)),
        Node::Top(o(roots[2])),
        Node::Cut(o(b), n(1), n(2)),
        Node::With(o(roots[3]), n(3), n(3)),
    ];
    let proof = Proof::new(forest, nodes, n(4)).unwrap();
    proof.check(mode).unwrap();
    let reduction = step(
        &proof,
        mode,
        None,
        &Options::default(),
        &Limits::default(),
        never,
    )
    .unwrap();
    assert_eq!(reduction.step.case, Case::Weakening);
    let made = reduction.proof;
    made.check(mode).unwrap();
    assert!(made.is_cut_free());
    let weakened: Vec<OccId> = made
        .nodes()
        .iter()
        .filter_map(|node| match node {
            Node::Weaken(w, _) => Some(w.occ()),
            _ => None,
        })
        .collect();
    let (c, d) = (
        made.forest().left(roots[3]).unwrap(),
        made.forest().right(roots[3]).unwrap(),
    );
    assert!(weakened.contains(&c) && weakened.contains(&d), "{made:?}");
}

/// Each bound ends an elimination with its refusal and the last proof
/// reached, which the checker accepts and still has cuts; a stop ends it
/// the same way, or refuses before anything is checked.
#[test]
fn a_bound_stops_with_a_proof() {
    let mode = Mode::INTUITIONISTIC;
    let proof = cut_proof("!a |- a * a", mode, "!a", &[0]);
    let limits = Limits::default();
    let one = Options::default().with_steps(Some(1));
    let ended = eliminate(&proof, mode, &one, &limits, never).unwrap();
    assert_eq!(
        ended.end,
        End::Refused(Refusal::Elimination {
            what: crate::limits::Bound::Steps,
            limit: 1
        })
    );
    assert_eq!(ended.steps.len(), 1);
    ended.proof.check(mode).unwrap();
    assert!(!ended.proof.is_cut_free());

    // A bound one below the largest proof the elimination reaches.
    let mut largest = 0;
    let mut current = proof.clone();
    while !current.is_cut_free() {
        current = step(&current, mode, None, &Options::default(), &limits, never)
            .unwrap()
            .proof;
        largest = largest.max(current.nodes().len() as u64);
    }
    let size = largest - 1;
    let small = Options::default().with_nodes(Some(size));
    let ended = eliminate(&proof, mode, &small, &limits, never).unwrap();
    let End::Refused(Refusal::Elimination {
        what: crate::limits::Bound::Nodes,
        limit,
    }) = ended.end
    else {
        panic!("{:?}", ended.end)
    };
    assert_eq!(limit, size);
    // Every proof a step made is within the bound; the one given need not be.
    assert!(ended.proof.nodes().len() as u64 <= size || ended.steps.is_empty());
    ended.proof.check(mode).unwrap();
    assert!(!ended.proof.is_cut_free());
    let refused = step(&ended.proof, mode, None, &small, &limits, never);
    assert!(matches!(
        refused,
        Err(Error::Refused(Refusal::Elimination { .. }))
    ));

    for after in 1..40 {
        let mut asked = 0;
        let stopped = eliminate(&proof, mode, &Options::default(), &limits, |_| {
            asked += 1;
            asked > after
        });
        match stopped {
            Ok(elimination) => {
                elimination.proof.check(mode).unwrap();
                if let End::Refused(refusal) = elimination.end {
                    assert!(refusal.is_stop());
                } else {
                    assert!(elimination.proof.is_cut_free());
                }
            }
            Err(error) => assert_eq!(error.kind(), crate::ErrorKind::Stopped),
        }
    }
}
