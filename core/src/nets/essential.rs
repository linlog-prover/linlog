// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Essential nets: a proof structure of an intuitionistic sequent read as
//! Lamarche's polarized structure, a directed graph, and decided by
//! directed acyclicity and the dominator condition (Lamarche; Murawski
//! and Ong), independently of the switchings.
//!
//! Every occurrence is input or output by the sequent's [`Reading`]. The
//! edges run from an output occurrence to its parent, from an input
//! connective to each of its input premises, and along every link from its
//! input literal (`~a`) to its output literal (`a`). The one premise with
//! no edge is the antecedent of an implication in output position,
//! `A ⊸ B`, stored as `A⊥ ⅋ B`: its input premise `A⊥` is where the
//! hypothesis `A` enters, and nothing leads into it. So every path runs
//! from the hypotheses and the antecedents toward the goal, the one
//! occurrence without an edge out once every literal is linked.
//!
//! A linking is an essential net when the graph has no directed cycle and
//! every implication in output position, other than the goal itself,
//! dominates its antecedent: every path from the antecedent to the goal
//! passes through the implication, so the hypothesis it binds is used
//! inside its body only. For the structure of an intuitionistic sequent
//! this is the same verdict as the Danos–Regnier criterion without Mix.

use super::{NetError, ProofStructure, STOPPED, VertexId};
use crate::limits::{Phase, Progress};
use crate::occurrences::{Forest, OccId, Reading, ShapeError, Side};
use crate::sequents::Kind;

/// The raw index that stands for "no occurrence".
const NONE: u32 = u32::MAX;

/// The bytes per vertex `is_essential` holds: two successors, the
/// marks of the search for a cycle and of the searches around each
/// implication, and the stack and predecessor of each.
const ESSENTIAL_BYTES: u64 = 8 + 4 + 1 + 8 + 4;

/// Returns the edges of the essential net that leave `o` whatever is
/// linked: to its parent for an output occurrence that is no root, to
/// each input premise for an input connective, and none for a literal in
/// input position, whose edge is its link.
pub(crate) fn fixed_successors(
    forest: &Forest,
    reading: &Reading<'_>,
    o: OccId,
) -> impl Iterator<Item = OccId> {
    let output = reading.position(o) == Side::Output;
    let parent = forest.parent(o).filter(|_| output);
    let premises = forest
        .children(o)
        .filter(move |&c| !output && reading.position(c) == Side::Input);
    parent.into_iter().chain(premises)
}

/// Returns whether `o` is an implication in output position, `A ⊸ B`
/// stored as `A⊥ ⅋ B`, whose left premise is the antecedent.
pub(crate) fn is_implication(forest: &Forest, reading: &Reading<'_>, o: OccId) -> bool {
    forest.kind(o) == Kind::Par && reading.position(o) == Side::Output
}

impl ProofStructure {
    /// Returns the intuitionistic reading of the structure's sequent: the
    /// side of `⊢` every vertex stands on, which orients its essential
    /// net. It is recomputed on each call, in time linear in the forest.
    ///
    /// # Errors
    ///
    /// The [`ShapeError`] of a sequent with no intuitionistic reading.
    pub fn reading(&self) -> Result<Reading<'_>, ShapeError> {
        Reading::new(&self.forest)
    }

    /// Decides whether the structure is an essential net of its
    /// sequent's intuitionistic reading, independently of any search and
    /// of the proof checker: every literal is linked, the directed graph
    /// of the essential net has no cycle, and every implication in output
    /// position but the goal dominates its antecedent. For a structure of
    /// an intuitionistic sequent the verdict is that of
    /// [`is_correct`](Self::is_correct) without Mix, whatever the
    /// structure's criterion. It holds working memory linear in the
    /// structure, takes time linear in it for the cycle and for each
    /// implication, and asks `stop` once per implication.
    ///
    /// # Errors
    ///
    /// [`NetError::Empty`], [`NetError::Unlinked`],
    /// [`NetError::NotIntuitionistic`] for a sequent without a reading,
    /// [`NetError::DirectedCycle`] or [`NetError::NoDominator`] with a
    /// witness; [`NetError::Refused`] when `stop` ended it.
    pub fn is_essential(&self, mut stop: impl FnMut(Progress) -> bool) -> Result<(), NetError> {
        let forest = &self.forest;
        if forest.is_empty() {
            return Err(NetError::Empty);
        }
        if let Some(l) = self.unlinked().next() {
            return Err(NetError::Unlinked { vertex: l });
        }
        let reading = self
            .reading()
            .map_err(|shape| NetError::NotIntuitionistic { shape })?;
        let n = forest.len();
        // Every vertex has at most two successors: an input `⅋` its two
        // premises, any other vertex one.
        let mut successors = vec![[NONE; 2]; n];
        for o in forest.ids() {
            let out = &mut successors[o.index()];
            let fixed = fixed_successors(forest, &reading, o);
            let link = self.mate(VertexId::of(o)).map(VertexId::occ);
            let link = link.filter(|_| reading.position(o) == Side::Input);
            for (slot, s) in out.iter_mut().zip(fixed.chain(link)) {
                *slot = s.get();
            }
        }
        let next = |v: u32| successors[v as usize].into_iter().filter(|&s| s != NONE);
        let held_bytes = n as u64 * ESSENTIAL_BYTES;
        let mut rounds = 0;
        let mut poll = |stop: &mut dyn FnMut(Progress) -> bool| {
            rounds += 1;
            stop(Progress {
                held_bytes,
                ..Progress::new(Phase::Net, 1, rounds)
            })
        };

        // A depth-first search over every vertex, its stack the path from
        // where it started: an edge back into the stack closes a cycle.
        // Per vertex, `UNSEEN`, `NONE` once its search is done, or its
        // place on the stack, which is below the vertices' count.
        const UNSEEN: u32 = NONE - 1;
        let mut state = vec![UNSEEN; n];
        let mut stack: Vec<(u32, usize)> = Vec::new();
        for start in 0..n as u32 {
            if state[start as usize] != UNSEEN {
                continue;
            }
            state[start as usize] = 0;
            stack.push((start, 0));
            while let Some(&mut (v, ref mut i)) = stack.last_mut() {
                let Some(w) = next(v).nth(*i) else {
                    state[v as usize] = NONE;
                    stack.pop();
                    continue;
                };
                *i += 1;
                match state[w as usize] {
                    UNSEEN => {
                        state[w as usize] = stack.len() as u32;
                        stack.push((w, 0));
                    }
                    NONE => {}
                    place => {
                        let cycle = stack[place as usize..]
                            .iter()
                            .map(|&(u, _)| VertexId::new(u))
                            .collect();
                        return Err(NetError::DirectedCycle { cycle });
                    }
                }
            }
        }
        if poll(&mut stop) {
            return Err(STOPPED);
        }

        // For each implication, a search from its antecedent that does not
        // enter the implication: reaching the goal is a path around it.
        let goal = reading.goal();
        let mut seen = vec![0u32; n];
        let mut from = vec![NONE; n];
        let mut round = 0u32;
        let mut queue = Vec::new();
        for l in forest.ids() {
            let Some(antecedent) = forest.left(l).filter(|_| l != goal) else {
                continue;
            };
            if !is_implication(forest, &reading, l) {
                continue;
            }
            if poll(&mut stop) {
                return Err(STOPPED);
            }
            // One round per implication, of which there are fewer than
            // the vertices, so the stamp never wraps.
            round += 1;
            let antecedent = antecedent.get();
            seen[l.index()] = round;
            seen[antecedent as usize] = round;
            queue.clear();
            queue.push(antecedent);
            let mut i = 0;
            while let Some(&v) = queue.get(i) {
                i += 1;
                if v == goal.get() {
                    let mut path = vec![VertexId::new(v)];
                    let mut u = v;
                    while u != antecedent {
                        u = from[u as usize];
                        path.push(VertexId::new(u));
                    }
                    path.reverse();
                    return Err(NetError::NoDominator {
                        implication: VertexId::of(l),
                        path,
                    });
                }
                for w in next(v) {
                    if seen[w as usize] != round {
                        seen[w as usize] = round;
                        from[w as usize] = v;
                        queue.push(w);
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(all(test, feature = "parse"))]
mod tests {
    use super::super::{Criterion, NetError, ProofStructure, VertexId};
    use super::{fixed_successors, is_implication};
    use crate::occurrences::{Forest, OccId, Side, Sign};
    use crate::search::generate::{self, IllRules, Rng};
    use crate::sequents::{Atom, Sequent};

    /// Unit-free IMLL.
    const IMLL: IllRules = IllRules {
        units: false,
        additives: false,
        zero: false,
        exponentials: false,
    };

    /// Builds the forest of `input`, or panics.
    fn forest(input: &str) -> Forest {
        let s: Sequent = input.parse().unwrap_or_else(|e| panic!("{input:?}: {e}"));
        Forest::new(&s).unwrap()
    }

    /// Links, each as a pair of vertices.
    type Links = [(VertexId, VertexId)];

    /// Calls `each` with every complete linking of the forest, each atom's
    /// literals paired in every way, unless there are more than `most`.
    fn linkings(forest: &Forest, most: usize, each: &mut dyn FnMut(&Links)) {
        let atoms = forest.sequent().atom_names().len();
        let mut count = 1usize;
        for a in 0..atoms {
            let n = forest.literals(Atom::new(a as u32), Sign::Atom).len();
            if n != forest.literals(Atom::new(a as u32), Sign::Dual).len() {
                return;
            }
            count = (1..=n).fold(count, |c, k| c.saturating_mul(k));
        }
        if count > most {
            return;
        }
        let duals: Vec<OccId> = forest
            .all_literals()
            .iter()
            .copied()
            .filter(|&x| forest.sign(x) == Some(Sign::Dual))
            .collect();
        let mut links = Vec::new();
        let mut used = vec![false; forest.len()];
        fn go(
            forest: &Forest,
            duals: &[OccId],
            used: &mut [bool],
            links: &mut Vec<(VertexId, VertexId)>,
            each: &mut dyn FnMut(&Links),
        ) {
            let Some((&x, rest)) = duals.split_first() else {
                each(links);
                return;
            };
            for &y in forest.literals(forest.atom(x).unwrap(), Sign::Atom) {
                if !used[y.index()] {
                    used[y.index()] = true;
                    links.push((VertexId::of(x), VertexId::of(y)));
                    go(forest, rest, used, links, each);
                    links.pop();
                    used[y.index()] = false;
                }
            }
        }
        go(forest, &duals, &mut used, &mut links, each);
    }

    /// The successors of a vertex in the essential net of a complete
    /// structure.
    fn successors(net: &ProofStructure, v: OccId) -> Vec<OccId> {
        let forest = net.forest();
        let reading = net.reading().unwrap();
        let mut out: Vec<OccId> = fixed_successors(forest, &reading, v).collect();
        if reading.position(v) == Side::Input
            && let Some(w) = net.partner(VertexId::of(v))
        {
            out.push(w.occ());
        }
        out
    }

    /// The dominator condition read off its definition: every simple
    /// path from the antecedent of every implication but the goal to the
    /// goal passes through the implication, all paths enumerated.
    fn dominated_by_every_path(net: &ProofStructure) -> bool {
        let forest = net.forest();
        let reading = net.reading().unwrap();
        let goal = reading.goal();
        let mut paths = 0u32;
        fn around(
            net: &ProofStructure,
            v: OccId,
            l: OccId,
            goal: OccId,
            on: &mut Vec<OccId>,
            paths: &mut u32,
        ) -> bool {
            *paths += 1;
            assert!(*paths < 1 << 20, "too many paths to enumerate");
            if v == goal {
                return !on.contains(&l);
            }
            on.push(v);
            let found = successors(net, v)
                .into_iter()
                .any(|w| !on.contains(&w) && around(net, w, l, goal, on, paths));
            on.pop();
            found
        }
        forest
            .ids()
            .filter(|&l| is_implication(forest, &reading, l) && l != goal)
            .all(|l| {
                !around(
                    net,
                    forest.left(l).unwrap(),
                    l,
                    goal,
                    &mut Vec::new(),
                    &mut paths,
                )
            })
    }

    /// Checks a witness against the structure: a directed cycle follows
    /// edges and closes; a path around an implication starts at its
    /// antecedent, follows edges, avoids it and ends at the goal.
    #[expect(
        clippy::wildcard_enum_match_arm,
        reason = "the criterion fails with these two witnesses only"
    )]
    fn check_witness(net: &ProofStructure, error: &NetError) {
        let edge = |a: VertexId, b: VertexId| successors(net, a.occ()).contains(&b.occ());
        match error {
            NetError::DirectedCycle { cycle } => {
                assert!(!cycle.is_empty());
                for (i, &a) in cycle.iter().enumerate() {
                    assert!(edge(a, cycle[(i + 1) % cycle.len()]), "{cycle:?}");
                }
            }
            NetError::NoDominator { implication, path } => {
                let forest = net.forest();
                let reading = net.reading().unwrap();
                assert_eq!(path[0].occ(), forest.left(implication.occ()).unwrap());
                assert_eq!(path.last().unwrap().occ(), reading.goal());
                assert!(!path.contains(implication));
                assert!(path.windows(2).all(|w| edge(w[0], w[1])), "{path:?}");
            }
            other => panic!("not a witness of the essential criterion: {other}"),
        }
    }

    /// On every linking of generated intuitionistic sequents and their
    /// mutants, the essential criterion and the Danos–Regnier criterion
    /// without Mix agree, the dominator condition agrees with its reading
    /// over every path, and every witness is one.
    #[test]
    #[expect(
        clippy::wildcard_enum_match_arm,
        reason = "the criterion fails with these two witnesses only"
    )]
    fn agrees_with_the_switchings() {
        let mut rng = Rng::new(35);
        let (mut structures, mut nets, mut cycles, mut dominators) = (0, 0, 0, 0);
        for _ in 0..400 {
            let budget = 2 + rng.below(9);
            let generate::Ill {
                mut hypotheses,
                mut goal,
                ..
            } = generate::ill(&mut rng, IMLL, 3, budget);
            for mutated in [false, true] {
                if mutated && !generate::mutate_two_sided(&mut rng, &mut hypotheses, &mut goal, 3) {
                    continue;
                }
                let text = generate::two_sided(&hypotheses, &goal);
                let forest = forest(&text);
                linkings(&forest, 120, &mut |links| {
                    let net =
                        ProofStructure::from_links(forest.clone(), Criterion::MLL, links).unwrap();
                    let switchings = net.is_correct(|_| false);
                    let essential = net.is_essential(|_| false);
                    assert_eq!(
                        switchings.is_ok(),
                        essential.is_ok(),
                        "{text:?} {links:?}: {switchings:?} against {essential:?}"
                    );
                    match &essential {
                        Ok(()) => nets += 1,
                        Err(error) => {
                            check_witness(&net, error);
                            match error {
                                NetError::DirectedCycle { .. } => cycles += 1,
                                _ => {
                                    dominators += 1;
                                    assert!(!dominated_by_every_path(&net), "{text:?} {links:?}");
                                }
                            }
                        }
                    }
                    if !matches!(essential, Err(NetError::DirectedCycle { .. })) {
                        assert_eq!(essential.is_ok(), dominated_by_every_path(&net));
                    }
                    structures += 1;
                });
            }
        }
        assert!(
            nets > 300 && cycles > 300 && dominators > 100,
            "{structures} structures: {nets} nets, {cycles} cycles, {dominators} without a dominator"
        );
    }

    /// A sequent without an intuitionistic reading has no essential net,
    /// and the witnesses of the two ways to fail are the expected ones.
    #[test]
    fn witnesses() {
        let links = |pairs: &[(u32, u32)]| -> Vec<(VertexId, VertexId)> {
            pairs
                .iter()
                .map(|&(x, y)| (VertexId::new(x), VertexId::new(y)))
                .collect()
        };
        let classical = ProofStructure::from_links(
            forest("|- a par b, ~a, ~b"),
            Criterion::MLL,
            &links(&[(1, 3), (2, 4)]),
        )
        .unwrap();
        assert!(matches!(
            classical.is_essential(|_| false),
            Err(NetError::NotIntuitionistic { .. })
        ));
        // `a ⊸ a` used as its own argument: `a ⊗ ~a` linked inside.
        let cycle = ProofStructure::from_links(
            forest("a -o a |- b -o b"),
            Criterion::MLL,
            &links(&[(2, 1), (4, 5)]),
        )
        .unwrap();
        let Err(error) = cycle.is_essential(|_| false) else {
            panic!("a cycle is no net");
        };
        assert_eq!(
            error.to_string(),
            "a directed cycle of the essential net runs through 0, 2, 1"
        );
        // `⊢ (a ⊸ b) ⊗ (b ⊸ a)` with each antecedent linked into the other
        // factor.
        let around = ProofStructure::from_links(
            forest("|- (a -o b) * (b -o a)"),
            Criterion::MLL,
            &links(&[(2, 6), (5, 3)]),
        )
        .unwrap();
        let Err(error) = around.is_essential(|_| false) else {
            panic!("a hypothesis used outside its body is no net");
        };
        assert_eq!(
            error.describe(&around).to_string(),
            "the implication ~a ⅋ b[1] does not dominate its antecedent: the path ~a[2], a[6], ~b ⅋ a[4], (~a ⅋ b) ⊗ (~b ⅋ a)[0] reaches the goal around it"
        );
        assert!(matches!(
            cycle.is_essential(|_| true),
            Err(NetError::DirectedCycle { .. })
        ));
        let net = ProofStructure::from_links(
            forest("a -o b, a |- b"),
            Criterion::MLL,
            &links(&[(3, 1), (2, 4)]),
        )
        .unwrap();
        assert_eq!(net.is_essential(|_| false), Ok(()));
    }
}
