// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use super::*;
use crate::occurrences::Instances;
use crate::proofs::{CutFormula, Interactive};
use crate::{Forest, InputOptions, Sequent, ViewOptions};
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
/// cut-free, as `eliminate` does in the same steps, giving its callback
/// each step with the proof it was taken on; and every case is met, in
/// every commutative mode.
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
        let (mut steps, mut taken) = (Vec::new(), Vec::new());
        while !current.is_cut_free() {
            taken.push(current.nodes().to_vec());
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
        let mut seen = Vec::new();
        let elimination = eliminate(
            proof,
            *mode,
            &Options::default(),
            &Limits::default(),
            never,
            |before, step| seen.push((before.nodes().to_vec(), *step)),
        )
        .unwrap();
        assert_eq!(elimination.end, End::CutFree);
        assert_eq!(elimination.steps, steps);
        assert_eq!(seen, taken.into_iter().zip(steps).collect::<Vec<_>>());
        assert_eq!(elimination.proof.nodes(), current.nodes());
    }
    // These proofs are propositional: the key case of `∀` and `∃` is met
    // by the first-order ones (`first_order_cuts_are_eliminated`).
    for &case in Case::ALL.iter().filter(|&&c| c != Case::ForallExists) {
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
    let ended = eliminate(&proof, mode, &one, &limits, never, |_, _| {}).unwrap();
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
    let ended = eliminate(&proof, mode, &small, &limits, never, |_, _| {}).unwrap();
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
        let stopped = eliminate(
            &proof,
            mode,
            &Options::default(),
            &limits,
            |_| {
                asked += 1;
                asked > after
            },
            |_, _| {},
        );
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

/// Builds a proof's nodes from its roots and cut pairs.
type Terms<'a> = dyn Fn(&[OccId], &[(OccId, OccId)]) -> Vec<Node> + 'a;

/// The axiom case renames the traced occurrence in one premise of a rule
/// of two premises that reads one subproof twice, not in both: a Mix of a
/// shared axiom, and an affine `⊗` of a shared absorbing subproof, each
/// with the cut formula's dual literal in it twice.
#[test]
fn the_axiom_case_renames_one_premise_of_a_shared_pair() {
    let o = |x: OccId| Member::from(x);
    let n = NodeId::new;
    let made = |text: &str, cuts: &[&str], nodes: &Terms<'_>| {
        let mut sequent: Sequent = text.parse().unwrap();
        for cut in cuts {
            sequent.add_cut(cut.parse().unwrap()).unwrap();
        }
        let forest = crate::Forest::new(&sequent).unwrap();
        let roots = forest.roots().to_vec();
        let pairs: Vec<(OccId, OccId)> = forest.cut_pairs().collect();
        let nodes = nodes(&roots, &pairs);
        let root = n(nodes.len() as u32 - 1);
        Proof::new(forest, nodes, root).unwrap()
    };
    let mix = made("|- ~x, ~x, x, x", &["|- x", "|- ~x"], &|r, p| {
        let ((a1, da1), (a2, da2)) = (p[0], p[1]);
        vec![
            Node::Ax(o(da1), o(da2)),
            Node::Mix(n(0), n(0)),
            Node::Ax(o(a1), o(r[0])),
            Node::Cut(o(a1), n(2), n(1)),
            Node::Ax(o(a1), o(r[1])),
            Node::Cut(o(a1), n(4), n(3)),
            Node::Ax(o(a2), o(r[2])),
            Node::Cut(o(a2), n(6), n(5)),
            Node::Ax(o(a2), o(r[3])),
            Node::Cut(o(a2), n(8), n(7)),
        ]
    });
    let tensor = made("|- ~x, ~x, p * q, 1, 1", &["|- x", "|- top"], &|r, p| {
        let ((a1, da1), (a4, da4)) = (p[0], p[1]);
        vec![
            Node::Top(o(a4)),
            Node::Weaken(o(da1), n(0)),
            Node::Tensor(o(r[2]), n(1), n(1)),
            Node::Ax(o(a1), o(r[0])),
            Node::Cut(o(a1), n(3), n(2)),
            Node::Ax(o(a1), o(r[1])),
            Node::Cut(o(a1), n(5), n(4)),
            Node::One(o(r[3])),
            Node::Weaken(o(da4), n(7)),
            Node::Cut(o(a4), n(6), n(8)),
            Node::One(o(r[4])),
            Node::Weaken(o(da4), n(10)),
            Node::Cut(o(a4), n(9), n(11)),
        ]
    });
    for (proof, mode) in [
        (mix, Mode::CLASSICAL.with_mix()),
        (tensor, Mode::CLASSICAL.with_affine()),
    ] {
        proof.check(mode).unwrap();
        let ended = eliminate(
            &proof,
            mode,
            &Options::default(),
            &Limits::default(),
            never,
            |_, _| {},
        )
        .unwrap();
        assert_eq!(ended.steps[0].case, Case::Axiom);
        assert_eq!(ended.end, End::CutFree);
    }
    // A node both `⊗`s under a `&` read, one taking the renamed literal
    // from it and the other from its other premise, in every mode.
    #[cfg(feature = "serialize")]
    {
        let proof: Proof = serde_json::from_str(
            r#"{"version":6,"sequent":{"terms":[{"D":0},{"?":0},{"V":0},{"⊗":[2,2]},{"V":0},{"⊕":[4,4]},{"!":5},{"&":[0,0]},{"?":7}],"roots":[0,1,3],"atoms":["a"],"antecedents":2,"cuts":[6]},"nodes":[{"ax":[12,4]},{"ax":[13,4]},{"ax":[12,5]},{"⊗":[3,0,2]},{"⊗":[3,1,2]},{"&":[11,3,4]},{"copy":[11,5]},{"ax":[8,0]},{"cut":[8,7,6]},{"?":[10,8]},{"ax":[8,2]},{"copy":[2,10]},{"⊕₁":[7,11]},{"!":[6,12]},{"cut":[6,13,9]},{"?":[1,14]}]}"#,
        )
        .unwrap();
        for mode in [
            Mode::CLASSICAL,
            Mode::CLASSICAL.with_affine(),
            Mode::CLASSICAL.with_mix(),
            Mode::CLASSICAL.with_affine().with_mix(),
            Mode::INTUITIONISTIC,
            Mode::INTUITIONISTIC.with_affine(),
        ] {
            proof.check(mode).unwrap();
            let ended = eliminate(
                &proof,
                mode,
                &Options::default(),
                &Limits::default(),
                never,
                |_, _| {},
            )
            .unwrap();
            assert_eq!(ended.end, End::CutFree, "{mode:?}");
        }
    }
}

/// An affine erasure specialized for many places is bounded inside the
/// step: a cut that `d` nested `&`s read under `2^d` contexts of `g`
/// formulas each asks the stop, and is refused past the bound on nodes
/// and on memory before its weakenings are made.
#[test]
fn a_specialized_erasure_is_bounded_inside_the_step() {
    let (d, g) = (13, 10);
    let mode = Mode::CLASSICAL.with_affine();
    let mut text = "|- 1, top".to_owned();
    for _ in 0..d {
        text.push_str(", 1 & 1");
    }
    for _ in 0..g {
        text.push_str(", 1");
    }
    let mut sequent: Sequent = text.parse().unwrap();
    sequent.add_cut("|- p".parse().unwrap()).unwrap();
    let forest = crate::Forest::new(&sequent).unwrap();
    let roots = forest.roots().to_vec();
    let (p, _) = forest.cut_pairs().next().unwrap();
    let (o, n) = (|x: OccId| Member::from(x), NodeId::new);
    let mut nodes = vec![
        Node::One(o(roots[0])),
        Node::Weaken(o(p), n(0)),
        Node::Top(o(roots[1])),
        Node::Cut(o(p), n(1), n(2)),
    ];
    for j in 0..d {
        let below = n(nodes.len() as u32 - 1);
        nodes.push(Node::With(o(roots[2 + j]), below, below));
    }
    let root = n(nodes.len() as u32 - 1);
    let proof = Proof::new(forest, nodes, root).unwrap();
    proof.check(mode).unwrap();
    let unbounded = Options::default().with_nodes(None);
    let mut asked = 0;
    let stopped = step(&proof, mode, None, &unbounded, &Limits::UNBOUNDED, |_| {
        asked += 1;
        asked >= 2
    });
    assert!(
        matches!(stopped, Err(Error::Refused(Refusal::Stopped { .. }))),
        "{stopped:?}"
    );
    let few = Options::default().with_nodes(Some(20_000));
    let refused = step(&proof, mode, None, &few, &Limits::default(), never);
    assert!(
        matches!(refused, Err(Error::Refused(Refusal::Elimination { .. }))),
        "{refused:?}"
    );
    let small = Limits::default().with_memory_bytes(Some(1 << 20));
    let refused = step(&proof, mode, None, &unbounded, &small, never);
    assert!(
        matches!(
            refused,
            Err(Error::Refused(Refusal::Memory {
                phase: Phase::Eliminate,
                ..
            }))
        ),
        "{refused:?}"
    );
    let made = step(&proof, mode, None, &unbounded, &Limits::default(), never).unwrap();
    assert!(made.proof.is_cut_free());
}

/// Returns the members of the subformulas of `m`, an instance of a
/// connective (not a binder) under its frame, in `table`.
fn children_of(forest: &Forest, table: &mut Instances, m: Member) -> Vec<Member> {
    let (o, frame) = match table.instance(m) {
        Some((o, frame)) => (o, Some(frame)),
        None => (m.occ(), None),
    };
    forest
        .children(o)
        .map(|c| match frame {
            Some(frame) => table.member(c, frame),
            None => Member::from(c),
        })
        .collect()
}

/// Appends to `nodes` a proof of `⊢ x, y`, two members of dual formulas
/// under equal frames, by the identity expanded rule by rule, each
/// `∀` step's eigenvariable the next of `eigen`, and returns its root.
fn identity(
    nodes: &mut Vec<Node>,
    table: &mut Instances,
    forest: &Forest,
    (x, y): (Member, Member),
    eigen: &mut u32,
) -> NodeId {
    use crate::Kind;
    let push = |nodes: &mut Vec<Node>, node| {
        nodes.push(node);
        NodeId::new(nodes.len() as u32 - 1)
    };
    let occurrence = |table: &Instances, m: Member| table.instance(m).map_or(m.occ(), |(o, _)| o);
    let kind = forest.kind(occurrence(table, x));
    let (cx, cy) = (children_of(forest, table, x), children_of(forest, table, y));
    match kind {
        Kind::Atom | Kind::DualAtom => push(nodes, Node::Ax(x, y)),
        Kind::Tensor => {
            let l = identity(nodes, table, forest, (cx[0], cy[0]), eigen);
            let r = identity(nodes, table, forest, (cx[1], cy[1]), eigen);
            let t = push(nodes, Node::Tensor(x, l, r));
            push(nodes, Node::Par(y, t))
        }
        Kind::One => {
            let one = push(nodes, Node::One(x));
            push(nodes, Node::Bot(y, one))
        }
        Kind::Top => push(nodes, Node::Top(x)),
        Kind::With => {
            let l = identity(nodes, table, forest, (cx[0], cy[0]), eigen);
            let l = push(nodes, Node::Plus(y, Branch::Left, l));
            let r = identity(nodes, table, forest, (cx[1], cy[1]), eigen);
            let r = push(nodes, Node::Plus(y, Branch::Right, r));
            push(nodes, Node::With(x, l, r))
        }
        Kind::Bang => {
            let inner = identity(nodes, table, forest, (cx[0], cy[0]), eigen);
            let copy = push(nodes, Node::Copy(cy[0], inner));
            let bang = push(nodes, Node::Bang(x, copy));
            push(nodes, Node::Quest(y, bang))
        }
        Kind::Forall => {
            let a = crate::sequents::Eigen::new(*eigen);
            *eigen += 1;
            let term = table.eigen(a);
            let mut body = |m: Member| {
                let (o, frame) = match table.instance(m) {
                    Some((o, frame)) => (o, Some(frame)),
                    None => (m.occ(), None),
                };
                let frame = table.frame(frame, term);
                table.member(OccId::new(o.get() + 1), frame)
            };
            let (bx, by) = (body(x), body(y));
            let inner = identity(nodes, table, forest, (bx, by), eigen);
            let exists = push(nodes, Node::Exists(y, term, inner));
            push(nodes, Node::Forall(x, a, exists))
        }
        Kind::Par | Kind::Bot | Kind::Zero | Kind::Plus | Kind::Quest | Kind::Exists => {
            identity(nodes, table, forest, (y, x), eigen)
        }
    }
}

/// Returns `proof`, a first-order proof of a sequent's roots, cut on its
/// root `root`: the sequent with that root's formula `A` as a cut
/// formula, the proof with `A` moved onto the cut formula's tree, cut
/// against the identity of `A` (`⊢ A⊥, A`, expanded rule by rule) on the
/// root itself; `None` where the sequent's tables or roots move with the
/// cut formula.
fn identity_cut(proof: &Proof, root: usize) -> Option<Proof> {
    let forest = proof.forest();
    let table = proof.instances()?;
    let r = forest.roots()[root];
    let mut sequent = proof.sequent().clone();
    sequent
        .add_cut(format!("|- {}", forest.formula(r)).parse().ok()?)
        .ok()?;
    let cut_forest = Forest::new(&sequent).ok()?;
    let made_for = Instances::new(&cut_forest);
    if cut_forest.roots() != forest.roots()
        || (made_for.base_terms, made_for.base_lists) != (table.base_terms, table.base_lists)
    {
        return None;
    }
    let (c, d) = cut_forest.cut_pairs().next()?;
    let size = forest.size(r);
    let moved = |o: OccId| {
        if (r.get()..r.get() + size).contains(&o.get()) {
            OccId::new(c.get() + o.get() - r.get())
        } else {
            o
        }
    };
    let shift = (cut_forest.len() - forest.len()) as u32;
    let mut instances = Instances {
        occurrences: made_for.occurrences,
        members: table.members.iter().map(|&(o, f)| (moved(o), f)).collect(),
        ..table.clone()
    };
    let mut nodes: Vec<Node> = proof
        .nodes()
        .iter()
        .map(|node| {
            node.renamed(|m| {
                Ok::<_, std::convert::Infallible>(if m.index() < forest.len() {
                    Member::from(moved(m.occ()))
                } else {
                    Member::new(m.get() + shift)
                })
            })
            .unwrap_or_else(|never| match never {})
        })
        .collect();
    let left = NodeId::new(nodes.len() as u32 - 1);
    let mut eigen = proof
        .nodes()
        .iter()
        .filter_map(|n| match n {
            Node::Forall(_, a, _) => Some(a.get() + 1),
            _ => None,
        })
        .chain(instances.eigenvariables().iter().map(|a| a.get() + 1))
        .max()
        .unwrap_or(0);
    let right = identity(
        &mut nodes,
        &mut instances,
        &cut_forest,
        (Member::from(d), Member::from(r)),
        &mut eigen,
    );
    nodes.push(Node::Cut(Member::from(c), left, right));
    let root = NodeId::new(nodes.len() as u32 - 1);
    Proof::with_instances(cut_forest, instances, nodes, root).ok()
}

/// First-order proofs with a cut, from the search's proofs of generated
/// sequents (classical) and of written ones (intuitionistic), each root
/// cut against its identity, with every mode beside the search's that
/// takes it: affine, Mix.
fn first_order_cuts() -> Vec<(Proof, Mode)> {
    use crate::search::generate::Rng;
    use crate::search::reference_fo::tests::random_sequent;
    let options = crate::Options::default().with_copies(Some(1));
    let mut texts: Vec<(String, Mode)> = Vec::new();
    for (seed, rules) in [
        [false, false, false],
        [true, true, false],
        [false, true, true],
        [true, true, true],
    ]
    .into_iter()
    .enumerate()
    {
        let mut rng = Rng::new(4400 + seed as u64);
        for _ in 0..150 {
            texts.push((random_sequent(&mut rng, rules), Mode::CLASSICAL));
        }
    }
    for text in [
        "forall x. p(x) |- exists y. p(y)",
        "exists y. forall x. r(x, y) |- forall x. exists y. r(x, y)",
        "exists x. (p(x) * q(x)) |- exists y. (q(y) * p(y))",
        "forall x. (p(x) -o q(x)), p(a) |- q(a)",
        "p(a) & p(b), q(b) |- exists x. (p(x) * q(x))",
        "p(z), !forall x. (p(x) -o p(s(x))) |- p(s(s(z)))",
        "exists x. (p(x) -o q) |- (forall x. p(x)) -o q",
        "forall x. (p(x) & q(x)) |- forall y. (q(y) & p(y))",
        "!forall x. p(x) |- p(a) * p(b)",
        "forall x. forall y. r(x, y) |- forall y. forall x. r(x, y)",
    ] {
        texts.push((text.to_owned(), Mode::INTUITIONISTIC));
    }
    let mut proofs = Vec::new();
    for (text, mode) in texts {
        let Ok(sequent) = text.parse::<Sequent>() else {
            continue;
        };
        let Ok(outcome) = crate::prove(&sequent, mode, &options) else {
            continue;
        };
        let crate::Verdict::Proved(proof) = outcome.verdict else {
            continue;
        };
        let modes = if mode.is_intuitionistic() {
            vec![mode, mode.with_affine()]
        } else {
            vec![mode, mode.with_mix(), mode.with_affine()]
        };
        for root in 0..proof.forest().roots().len() {
            let Some(cut) = identity_cut(&proof, root) else {
                continue;
            };
            for &mode in &modes {
                if cut.check(mode).is_ok() {
                    proofs.push((cut.clone().with_mode(mode), mode));
                }
            }
        }
    }
    proofs
}

/// Every step of the elimination of first-order proofs with cuts, each
/// a search's proof cut against an identity, makes a proof over the same
/// sequent with the same conclusion that the checker accepts, every cut
/// of it on a cut formula's tree, and ends cut-free as `eliminate` does;
/// any cut of a proof on the way may be reduced first;
/// in the classical modes and the intuitionistic ones; the key case of
/// `∀` and `∃` is met on both sides of `⊢`.
#[test]
fn first_order_cuts_are_eliminated() {
    let proofs = first_order_cuts();
    let mut cases: BTreeMap<&str, usize> = BTreeMap::new();
    let mut modes: BTreeMap<String, usize> = BTreeMap::new();
    let mut steps_taken = 0;
    for (proof, mode) in &proofs {
        *modes.entry(format!("{mode:?}")).or_default() += 1;
        let mut current = proof.clone();
        let mut steps = Vec::new();
        while !current.is_cut_free() {
            assert!(steps.len() < MOST_STEPS, "no end in {mode:?}: {proof:?}");
            // Any cut may be reduced, not only the strategy's.
            for id in current
                .ids()
                .filter(|&id| matches!(current.node(id), Node::Cut(..)))
            {
                let options = Options::default();
                let reduction = step(
                    &current,
                    *mode,
                    Some(id),
                    &options,
                    &Limits::default(),
                    never,
                )
                .unwrap_or_else(|e| panic!("{e} at {id:?} in {mode:?} of {current:?}"));
                reduction.proof.check(*mode).unwrap();
            }
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
                    assert!(made.forest().is_cut_formula(made.occurrence(*a)));
                }
            }
            *cases.entry(reduction.step.case.name()).or_default() += 1;
            steps.push(reduction.step);
            current = reduction.proof;
        }
        steps_taken += steps.len();
        let elimination = eliminate(
            proof,
            *mode,
            &Options::default(),
            &Limits::default(),
            never,
            |_, _| {},
        )
        .unwrap();
        assert_eq!(elimination.end, End::CutFree);
        assert_eq!(elimination.steps, steps);
    }
    println!(
        "{} proofs, {steps_taken} steps, {cases:?}, {modes:?}",
        proofs.len()
    );
    assert!(
        cases
            .get("the key case of ∀ and ∃")
            .is_some_and(|&n| n >= 100)
    );
    let intuitionistic = proofs.iter().filter(|(_, m)| m.is_intuitionistic()).count();
    assert!(
        intuitionistic >= 10 && proofs.len() - intuitionistic >= 200,
        "{modes:?}"
    );
}
