// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use super::{MellStructure, Parts, Placed, Placement};
use crate::Error;
use crate::fragment::Mode;
use crate::limits::{Limits, Refusal};
use crate::nets::{Criterion, NetError, ProofStructure, VertexId};
use crate::occurrences::{Forest, Member, OccId};
use crate::proofs::{Node, NodeId, Proof};
use crate::search::generate::{self, Rng, Rules};
use crate::search::{Engine, Options, Verdict};
use crate::sequents::{Kind, Sequent};

/// Wraps a raw vertex id.
const fn v(id: u32) -> VertexId {
    VertexId::new(id)
}

/// Wraps a raw occurrence id.
const fn o(id: u32) -> OccId {
    OccId::new(id)
}

/// The forest of a sequent.
fn forest(input: &str) -> Forest {
    let s: Sequent = input.parse().unwrap_or_else(|e| panic!("{input:?}: {e}"));
    Forest::new(&s).unwrap()
}

/// Builds a structure from its non-root vertices as (occurrence, parent,
/// box), its links and weakenings, or returns the error.
fn build(
    input: &str,
    vertices: &[(u32, u32, Option<u32>)],
    links: &[(u32, u32)],
    weakenings: &[(u32, u32)],
) -> Result<MellStructure, Error> {
    let parts = Parts {
        vertices: vertices
            .iter()
            .map(|&(oc, p, b)| Placed::new(o(oc), v(p), b.map(v)))
            .collect(),
        links: links.iter().map(|&(x, y)| (v(x), v(y))).collect(),
        weakenings: weakenings.iter().map(|&(x, y)| (v(x), v(y))).collect(),
    };
    MellStructure::from_parts(forest(input), Criterion::MLL, &parts, &Limits::default())
}

/// Checks that a structure is a net, sequentializes into a proof the
/// checker accepts, and desequentializes back into a net, the same up to
/// the links its weakenings hang on where no occurrence has two vertices:
/// otherwise the term does not say which instance a rule acts on.
fn round_trip(net: &MellStructure) -> Proof {
    assert_eq!(net.is_correct(|_| false), Ok(()));
    let proof = net.sequentialize(&Limits::default(), |_| false).unwrap();
    let mode = if net.criterion().mix {
        Mode::CLASSICAL.with_mix()
    } else {
        Mode::CLASSICAL
    };
    assert_eq!(proof.mode(), Some(mode));
    assert_eq!(proof.check(mode), Ok(()));
    let back =
        MellStructure::from_proof(&proof, net.criterion(), &Limits::default(), |_| false).unwrap();
    let mut seen = vec![false; net.forest().len()];
    let repeats = net.vertices().any(|x| {
        let o = net.occurrence(x).unwrap().index();
        std::mem::replace(&mut seen[o], true)
    });
    assert!(
        repeats || isomorphic(net, &back),
        "{net:?}\nback as\n{back:?}"
    );
    proof
}

/// One pair of vertices to match, or two lists of copies to match in some
/// order.
#[derive(Clone)]
enum Pending {
    /// These two vertices.
    Pair(u32, u32),
    /// These copies, in some order.
    Bag(Vec<u32>, Vec<u32>),
}

/// Returns whether two structures over one forest are the same up to the
/// order of each `?` vertex's copies, the numbering of the vertices and the
/// links the weakenings hang on: a bijection that keeps occurrences,
/// premises, boxes and links. A search over the copies' orders, for small
/// structures.
fn isomorphic(a: &MellStructure, b: &MellStructure) -> bool {
    if a.len() != b.len() || a.links().len() != b.links().len() {
        return false;
    }
    let roots = a.forest().roots().len() as u32;
    let pending = (0..roots).map(|r| Pending::Pair(r, r)).collect();
    matches(a, b, vec![None; a.len()], vec![None; b.len()], pending)
}

/// Extends a partial bijection over the pending pairs, backtracking over
/// the order of copies.
fn matches(
    a: &MellStructure,
    b: &MellStructure,
    mut map: Vec<Option<u32>>,
    mut back: Vec<Option<u32>>,
    mut pending: Vec<Pending>,
) -> bool {
    while let Some(next) = pending.pop() {
        match next {
            Pending::Pair(x, y) => {
                match (map[x as usize], back[y as usize]) {
                    (Some(m), _) if m == y => continue,
                    (None, None) => {}
                    _ => return false,
                }
                let (vx, vy) = (v(x), v(y));
                if a.occurrence(vx) != b.occurrence(vy) {
                    return false;
                }
                map[x as usize] = Some(y);
                back[y as usize] = Some(x);
                match (a.inside(vx), b.inside(vy)) {
                    (None, None) => {}
                    (Some(p), Some(q)) => pending.push(Pending::Pair(p.get(), q.get())),
                    _ => return false,
                }
                match (a.partner(vx), b.partner(vy)) {
                    (None, None) => {}
                    (Some(p), Some(q)) => pending.push(Pending::Pair(p.get(), q.get())),
                    _ => return false,
                }
                let px: Vec<u32> = a.premises(vx).map(VertexId::get).collect();
                let py: Vec<u32> = b.premises(vy).map(VertexId::get).collect();
                if px.len() != py.len() {
                    return false;
                }
                if a.forest().kind(a.occurrence(vx).unwrap()) == Kind::Quest {
                    pending.push(Pending::Bag(px, py));
                } else {
                    pending.extend(px.into_iter().zip(py).map(|(p, q)| Pending::Pair(p, q)));
                }
            }
            Pending::Bag(xs, ys) => {
                let Some((&first, rest)) = xs.split_first() else {
                    continue;
                };
                return (0..ys.len()).any(|j| {
                    let mut others = ys.clone();
                    let y = others.remove(j);
                    let mut pending = pending.clone();
                    pending.push(Pending::Bag(rest.to_vec(), others));
                    pending.push(Pending::Pair(first, y));
                    matches(a, b, map.clone(), back.clone(), pending)
                });
            }
        }
    }
    true
}

/// Proves a sequent with the focused engine within `copies` copies and
/// a bound of polls of its stop, which keeps a debug build's search on a
/// large generated sequent short; `None` unless it found a proof.
fn proof_of(sequent: &Sequent, mode: Mode, copies: u32) -> Option<Proof> {
    let options = Options::default()
        .with_copies(Some(copies))
        .with_engine(Some(Engine::Focus));
    let mut polls = 0u32;
    let outcome = crate::search::prove_within(sequent, mode, &options, &Limits::default(), |_| {
        polls += 1;
        polls > 20_000
    })
    .ok()?;
    match outcome.verdict {
        Verdict::Proved(proof) => Some(*proof),
        Verdict::Unprovable(_) | Verdict::Unknown(_) => None,
    }
}

/// Fixed nets: a box with a copy inside it, its contraction outside, a
/// weakening, and the witnesses of what is no net.
#[test]
fn classic_nets() {
    // ⊢ ?~a, !a: occurrences 0 ?~a, 1 ~a, 2 !a, 3 a; vertices 0 ?~a, 1 !a
    // (the roots), 2 a in the box of 1, 3 the copy ~a in that box.
    let net = build(
        "|- ?~a, !a",
        &[(3, 1, Some(1)), (1, 0, Some(1))],
        &[(2, 3)],
        &[],
    )
    .unwrap();
    assert_eq!(net.auxiliary_doors(v(1)), [v(0)]);
    let proof = round_trip(&net);
    assert_eq!(
        proof.nodes(),
        [
            Node::Ax(Member::from(o(1)), Member::from(o(3))),
            Node::Copy(Member::from(o(1)), NodeId::new(0)),
            Node::Bang(Member::from(o(2)), NodeId::new(1)),
            Node::Quest(Member::from(o(0)), NodeId::new(2)),
        ]
    );
    // The copy outside the box links out of it.
    assert!(matches!(
        build("|- ?~a, !a", &[(3, 1, Some(1)), (1, 0, None)], &[(2, 3)], &[]),
        Err(Error::Net(e)) if *e == NetError::Placement { vertex: v(2), fault: Placement::Link }
    ));
    // ⊢ ?~a, !a ⊗ !a: two boxes, a copy in each, one contraction: 0 ?~a,
    // 1 ~a, 2 ⊗, 3 !a, 4 a, 5 !a, 6 a. Vertices 0 ?~a, 1 ⊗, 2 !a, 3 !a,
    // 4 a, 5 a, 6 and 7 the copies.
    let net = build(
        "|- ?~a, !a * !a",
        &[
            (3, 1, None),
            (5, 1, None),
            (4, 2, Some(2)),
            (6, 3, Some(3)),
            (1, 0, Some(2)),
            (1, 0, Some(3)),
        ],
        &[(4, 6), (5, 7)],
        &[],
    )
    .unwrap();
    round_trip(&net);
    // ⊢ ?b, a, ~a: the weakening hangs on the link.
    let weakened = build("|- ?b, a, ~a", &[], &[(1, 2)], &[(0, 1)]).unwrap();
    round_trip(&weakened);
    assert_eq!(
        build("|- ?b, a, ~a", &[], &[(1, 2)], &[])
            .unwrap()
            .is_correct(|_| false),
        Err(NetError::Unattached { vertex: v(0) })
    );
    // ⊢ ?c ⊗ (a ⅋ (b ⅋ ~b)), ~a, whose components the weakening count
    // takes for a net: wherever the weakening hangs, a switching cycle runs
    // through the ⊗. Occurrences 0 ⊗, 1 ?c, 2 c, 3 ⅋, 4 a, 5 ⅋, 6 b, 7 ~b,
    // 8 ~a; vertices 0 ⊗, 1 ~a, 2 ?c, 3 ⅋, 4 a, 5 ⅋, 6 b, 7 ~b.
    for literal in [1, 4, 6, 7] {
        let net = build(
            "|- ?c * (a par (b par ~b)), ~a",
            &[
                (1, 0, None),
                (3, 0, None),
                (4, 3, None),
                (5, 3, None),
                (6, 5, None),
                (7, 5, None),
            ],
            &[(4, 1), (6, 7)],
            &[(2, literal)],
        )
        .unwrap();
        assert!(
            matches!(
                net.is_correct(|_| false),
                Err(NetError::SwitchingCycle { .. })
            ),
            "hung on {literal}"
        );
    }
    // ⊢ !(a ⊗ b), ?~a, ?~b with both copies in the box is a net; with
    // ⊢ !(a ⅋ b) the box's contents fall apart. Occurrences 0 !, 1 ⊗ or
    // ⅋, 2 a, 3 b, 4 ?~a, 5 ~a, 6 ?~b, 7 ~b; vertices 0 !, 1 ?~a, 2 ?~b,
    // 3 the connective, 4 a, 5 b, 6 ~a, 7 ~b.
    let vertices = [
        (1, 0, Some(0)),
        (2, 3, Some(0)),
        (3, 3, Some(0)),
        (5, 1, Some(0)),
        (7, 2, Some(0)),
    ];
    let net = build("|- !(a * b), ?~a, ?~b", &vertices, &[(4, 6), (5, 7)], &[]).unwrap();
    round_trip(&net);
    let net = build("|- !(a par b), ?~a, ?~b", &vertices, &[(4, 6), (5, 7)], &[]).unwrap();
    let Err(NetError::InBox { door, error }) = net.is_correct(|_| false) else {
        panic!("a box whose contents fall apart is no net");
    };
    assert_eq!(door, v(0));
    assert!(matches!(*error, NetError::Disconnected { ref parts } if parts.len() == 2));
    assert_eq!(
        NetError::InBox { door, error }.describe(&net).to_string(),
        "inside the box of !(a ⅋ b)[0], every switching falls into 2 parts; keeping every left \
         premise, they are {a ⅋ b[3], ?~a[1]} and {b[5], ?~b[2]}"
    );
}

/// Random proofs of unit-free MELL, with and without Mix, found by the
/// focused engine: each desequentializes into a net, which sequentializes
/// into a proof the checker accepts, which desequentializes into the same
/// net up to the links its weakenings hang on.
#[test]
fn round_trips() {
    let mut cases = (0, 0, 0);
    for (seed, mix) in [(11, false), (12, true)] {
        let mut rng = Rng::new(seed);
        for _ in 0..120 {
            let rules = Rules {
                units: false,
                additives: false,
                mix,
                exponentials: true,
            };
            let budget = 2 + rng.below(10);
            let provable = generate::provable(&mut rng, rules, 3, budget);
            let text = generate::sequent(&provable.formulas);
            let sequent: Sequent = text.parse().unwrap();
            let mode = if mix {
                Mode::CLASSICAL.with_mix()
            } else {
                Mode::CLASSICAL
            };
            let Some(proof) = proof_of(&sequent, mode, provable.copies) else {
                continue;
            };
            let criterion = Criterion {
                mix,
                ..Criterion::MLL
            };
            let net = MellStructure::from_proof(&proof, criterion, &Limits::default(), |_| false)
                .unwrap_or_else(|e| panic!("{text:?}: {e}"));
            round_trip(&net);
            cases.0 += 1;
            if net.boxes().next().is_some() {
                cases.1 += 1;
            }
            if net.vertices().any(|q| net.hangs_on(q).is_some()) {
                cases.2 += 1;
            }
        }
    }
    assert!(cases.0 > 100 && cases.1 > 30 && cases.2 > 10, "{cases:?}");
}

/// On MLL sequents the criterion is the one of [`ProofStructure`]: random
/// linkings of random sequents get the same verdict.
#[test]
fn agrees_with_the_multiplicative_criterion() {
    let mut rng = Rng::new(21);
    let mut verdicts = [0; 2];
    for i in 0..300 {
        let rules = Rules {
            units: false,
            additives: false,
            mix: i % 3 == 2,
            exponentials: false,
        };
        let budget = 2 + rng.below(9);
        let mut formulas = generate::provable(&mut rng, rules, 3, budget).formulas;
        if i % 2 == 1 {
            generate::mutate(&mut rng, &mut formulas, 3);
        }
        let s: Sequent = generate::sequent(&formulas).parse().unwrap();
        let forest = Forest::new(&s).unwrap();
        let mut links = Vec::new();
        for a in 0..s.atom_names().len() {
            let atom = crate::sequents::Atom::new(a as u32);
            let atoms = forest.literals(atom, crate::occurrences::Sign::Atom);
            let mut duals = forest
                .literals(atom, crate::occurrences::Sign::Dual)
                .to_vec();
            for k in (1..duals.len()).rev() {
                duals.swap(k, rng.below(k + 1));
            }
            links.extend(atoms.iter().zip(duals).map(|(&x, y)| (x, y)));
        }
        let mix = i % 4 == 0;
        let criterion = Criterion {
            mix,
            ..Criterion::MLL
        };
        let pairs: Vec<_> = links
            .iter()
            .map(|&(x, y)| (v(x.get()), v(y.get())))
            .collect();
        let mll = ProofStructure::from_links(forest.clone(), criterion, &pairs).unwrap();
        // The same trees with the roots first: vertex numbers by a map.
        let roots = forest.roots().len() as u32;
        let mut vertex = vec![0; forest.len()];
        for (i, &r) in forest.roots().iter().enumerate() {
            vertex[r.index()] = i as u32;
        }
        let mut next = roots;
        let mut vertices = Vec::new();
        for occurrence in forest.ids() {
            if let Some(p) = forest.parent(occurrence) {
                vertex[occurrence.index()] = next;
                next += 1;
                vertices.push(Placed::new(occurrence, v(vertex[p.index()]), None));
            }
        }
        let parts = Parts {
            vertices,
            links: links
                .iter()
                .map(|&(x, y)| (v(vertex[x.index()]), v(vertex[y.index()])))
                .collect(),
            weakenings: Vec::new(),
        };
        let net = MellStructure::from_parts(forest, criterion, &parts, &Limits::default()).unwrap();
        let expected = mll.is_correct(|_| false).is_ok();
        assert_eq!(
            net.is_correct(|_| false).is_ok(),
            expected,
            "{s} with {links:?}"
        );
        verdicts[usize::from(expected)] += 1;
        if expected {
            round_trip(&net);
        }
    }
    assert!(verdicts[0] > 50 && verdicts[1] > 50, "{verdicts:?}");
}

/// A node of a box depth in [`enumerate`]: a vertex, or the door of a `?`
/// vertex.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Place {
    /// A vertex of the depth.
    Vertex(u32),
    /// The door of this `?` vertex.
    Door(u32),
}

/// An edge of a box depth in [`enumerate`]: its two ends and the vertex
/// whose switching keeps it, if any.
type Edge = (Place, Place, Option<Place>);

/// Returns the representative of `x` in a union-find without ranks.
fn find(parent: &[usize], mut x: usize) -> usize {
    while parent[x] != x {
        x = parent[x];
    }
    x
}

/// The criterion by brute force, written from the definition without the
/// structure's graph: at every box depth, its vertices, the `!` vertices
/// of the boxes in it, a door for every `?` vertex outside with a wire
/// into it; every switching of every depth acyclic, and connected
/// without Mix. `None` when a depth has more than 4 096 switchings.
fn enumerate(net: &MellStructure) -> Option<bool> {
    let f = net.forest();
    let kind = |x: VertexId| f.kind(net.occurrence(x).unwrap());
    let mut depths: Vec<Option<VertexId>> = vec![None];
    depths.extend(net.boxes().map(Some));
    for depth in depths {
        let mut nodes: Vec<Place> = net
            .vertices()
            .filter(|&x| net.inside(x) == depth)
            .map(|x| Place::Vertex(x.get()))
            .collect();
        // Edges: (a, b, switch), the switch the vertex whose choice keeps
        // the edge, or `None`.
        let mut edges: Vec<Edge> = Vec::new();
        let add = |edges: &mut Vec<Edge>, e: Edge| {
            if !edges.contains(&e) {
                edges.push(e);
            }
        };
        for x in net.vertices().filter(|&x| net.inside(x) == depth) {
            if let Some(p) = net.partner(x)
                && p > x
            {
                add(
                    &mut edges,
                    (Place::Vertex(x.get()), Place::Vertex(p.get()), None),
                );
            }
            if matches!(kind(x), Kind::Tensor | Kind::Par) {
                for c in net.premises(x) {
                    let switch = (kind(x) == Kind::Par).then_some(Place::Vertex(x.get()));
                    add(
                        &mut edges,
                        (Place::Vertex(x.get()), Place::Vertex(c.get()), switch),
                    );
                }
            }
        }
        // Wires of every `?` vertex: the end at its own depth, each box
        // border crossed on the way out.
        for q in net.vertices().filter(|&q| kind(q) == Kind::Quest) {
            let ends: Vec<VertexId> = net.premises(q).chain(net.hangs_on(q)).collect();
            for end in ends {
                let mut at = end;
                let mut b = net.inside(end);
                loop {
                    let here = b;
                    let source = if here == net.inside(q) {
                        Place::Vertex(q.get())
                    } else {
                        Place::Door(q.get())
                    };
                    if here == depth {
                        if source != Place::Vertex(q.get()) && !nodes.contains(&source) {
                            nodes.push(source);
                        }
                        add(&mut edges, (source, Place::Vertex(at.get()), Some(source)));
                    }
                    if here == net.inside(q) {
                        break;
                    }
                    let bang = here.unwrap();
                    at = bang;
                    b = net.inside(bang);
                }
            }
        }
        let switched: Vec<Place> = {
            let mut s: Vec<Place> = edges.iter().filter_map(|e| e.2).collect();
            s.sort_by_key(|p| format!("{p:?}"));
            s.dedup();
            s
        };
        let choices: Vec<Vec<usize>> = switched
            .iter()
            .map(|&s| {
                (0..edges.len())
                    .filter(|&i| edges[i].2 == Some(s))
                    .collect()
            })
            .collect();
        let count: usize = choices.iter().map(Vec::len).product();
        if count > 4096 {
            return None;
        }
        for mut k in 0..count {
            let mut kept: Vec<usize> = (0..edges.len()).filter(|&i| edges[i].2.is_none()).collect();
            for c in &choices {
                kept.push(c[k % c.len()]);
                k /= c.len();
            }
            // Union-find over the nodes.
            let index = |p: Place| nodes.iter().position(|&q| q == p).unwrap();
            let mut parent: Vec<usize> = (0..nodes.len()).collect();
            for &i in &kept {
                let (a, b) = (
                    find(&parent, index(edges[i].0)),
                    find(&parent, index(edges[i].1)),
                );
                if a == b {
                    return Some(false);
                }
                parent[a] = b;
            }
            if !net.criterion().mix && kept.len() + 1 != nodes.len() {
                return Some(false);
            }
        }
    }
    Some(true)
}

/// The criterion against [`enumerate`] on random structures: nets of
/// random proofs, and their mutants, in which a link, a copy's box or a
/// weakening's literal moves where the structure allows.
#[test]
fn agrees_with_enumeration() {
    let mut cases = [0; 2];
    for (seed, mix) in [(31, false), (32, true)] {
        let mut rng = Rng::new(seed);
        for _ in 0..150 {
            let rules = Rules {
                units: false,
                additives: false,
                mix,
                exponentials: true,
            };
            let budget = 2 + rng.below(8);
            let provable = generate::provable(&mut rng, rules, 2, budget);
            let sequent: Sequent = generate::sequent(&provable.formulas).parse().unwrap();
            let mode = if mix {
                Mode::CLASSICAL.with_mix()
            } else {
                Mode::CLASSICAL
            };
            let Some(proof) = proof_of(&sequent, mode, provable.copies) else {
                continue;
            };
            let criterion = Criterion {
                mix,
                ..Criterion::MLL
            };
            let net = MellStructure::from_proof(&proof, criterion, &Limits::default(), |_| false)
                .unwrap();
            for round in 0..8 {
                let mut parts = net.parts();
                if round > 0 {
                    mutate(&net, &mut parts, &mut rng);
                }
                let Ok(mutant) = MellStructure::from_parts(
                    net.forest().clone(),
                    criterion,
                    &parts,
                    &Limits::default(),
                ) else {
                    continue;
                };
                let Some(expected) = enumerate(&mutant) else {
                    continue;
                };
                let verdict = mutant.is_correct(|_| false);
                assert_eq!(verdict.is_ok(), expected, "{mutant:?}: {verdict:?}");
                cases[usize::from(expected)] += 1;
                if expected {
                    round_trip(&mutant);
                }
            }
        }
    }
    assert!(cases[0] > 30 && cases[1] > 100, "{cases:?}");
}

/// Moves one link, one copy's box or one weakening's literal of a
/// structure's parts at random.
fn mutate(net: &MellStructure, parts: &mut Parts, rng: &mut Rng) {
    let roots = net.forest().roots().len();
    match rng.below(3) {
        0 if parts.links.len() >= 2 => {
            let (i, j) = (rng.below(parts.links.len()), rng.below(parts.links.len()));
            let ((a, b), (c, d)) = (parts.links[i], parts.links[j]);
            if net
                .forest()
                .dual_literals(net.occurrence(a).unwrap(), net.occurrence(d).unwrap())
            {
                parts.links[i] = (a, d);
                parts.links[j] = (c, b);
            }
        }
        1 => {
            let copies: Vec<usize> = (0..parts.vertices.len())
                .filter(|&i| {
                    let p = parts.vertices[i].parent;
                    net.forest().kind(net.occurrence(p).unwrap()) == Kind::Quest
                })
                .collect();
            let boxes: Vec<VertexId> = net.boxes().collect();
            if let (false, false) = (copies.is_empty(), boxes.is_empty()) {
                let i = copies[rng.below(copies.len())];
                let b = boxes[rng.below(boxes.len())];
                let inside = if rng.one_in(3) { None } else { Some(b) };
                // The copy's tree moves with it, but for what lies in
                // boxes of its own.
                let moved = VertexId::new((roots + i) as u32);
                let old = parts.vertices[i].inside;
                for k in 0..parts.vertices.len() {
                    let mut x = Some(VertexId::new((roots + k) as u32));
                    while let Some(y) = x
                        && y != moved
                    {
                        x = net.parent(y);
                    }
                    if x.is_some() && parts.vertices[k].inside == old {
                        parts.vertices[k].inside = inside;
                    }
                }
            }
        }
        _ if !parts.weakenings.is_empty() => {
            let i = rng.below(parts.weakenings.len());
            let literals: Vec<VertexId> = net
                .vertices()
                .filter(|&x| net.forest().is_literal(net.occurrence(x).unwrap()))
                .collect();
            parts.weakenings[i].1 = literals[rng.below(literals.len())];
        }
        _ => {}
    }
}

/// A term of a few dozen nodes that unfolds into more vertices than a
/// structure holds is refused before anything is made: each level uses
/// the one below twice, so the 48th makes 3 · 2⁴⁸ vertices.
#[test]
fn an_unfolding_past_the_vertices_is_refused() {
    // ⊢ ?(a ⅋ ~a): 0 ?, 1 ⅋, 2 a, 3 ~a.
    let f = forest("|- ?(a par ~a)");
    let m = |id| Member::from(o(id));
    let mut nodes = vec![Node::Ax(m(2), m(3)), Node::Par(m(1), NodeId::new(0))];
    for _ in 0..48 {
        let last = NodeId::new(nodes.len() as u32 - 1);
        nodes.push(Node::Mix(last, last));
        nodes.push(Node::Copy(m(1), NodeId::new(nodes.len() as u32 - 1)));
    }
    nodes.push(Node::Copy(m(1), NodeId::new(nodes.len() as u32 - 1)));
    nodes.push(Node::Quest(m(0), NodeId::new(nodes.len() as u32 - 1)));
    let root = NodeId::new(nodes.len() as u32 - 1);
    let proof = Proof::new(f, nodes, root).unwrap();
    let mix = Mode::CLASSICAL.with_mix();
    assert_eq!(proof.check(mix), Ok(()));
    let criterion = Criterion::MLL.with_mix();
    let refused = MellStructure::from_proof(&proof, criterion, &Limits::default(), |_| false);
    assert!(
        matches!(refused, Err(Error::Refused(Refusal::Index { count, .. })) if count > 1 << 49),
        "{refused:?}"
    );
}

/// The text form: the sequent, each `?` vertex's copies with the box of a
/// copy outside the `?`'s, or the literal a weakening hangs on, the links
/// by their first id, and the verdict.
#[test]
fn text_form() {
    let sequent: Sequent = "!a, !b |- !a".parse().unwrap();
    let outcome = crate::search::prove(&sequent, Mode::CLASSICAL, &Options::default()).unwrap();
    let Verdict::Proved(proof) = &outcome.verdict else {
        panic!("{:?}", outcome.verdict);
    };
    let net =
        MellStructure::from_proof(proof, Criterion::MLL, &Limits::default(), |_| false).unwrap();
    assert_eq!(
        net.to_string(),
        "⊢ ?~a, ?~b, !a\n?~a[0]: ~a[3] in !a[2]\n?~b[1]: no copy, hangs on a[4]\n~a[3] — a[4]\nproof net"
    );
}
