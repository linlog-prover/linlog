// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The planar criterion of the net engine's search, for cyclic MLL: the
//! Danos–Regnier criterion of the classical search, and no two links
//! crossing in the cyclic order of the literals, which is the order of
//! their occurrence ids. A structure of unit-free MLL is a proof net of
//! cyclic MLL exactly when it is a planar proof net
//! ([`ProofStructure::is_planar`]), so the search keeps planarity as a
//! rejection of every candidate link and leaves the rest to the classical
//! criterion it wraps.
//!
//! Between two links that do not cross, the literals form regions: a
//! literal's region is the innermost link around it in the linear order
//! of the literals, or none. A link crosses no link exactly when its two
//! literals lie in one region (one of them inside a link the other is
//! outside of makes that link cross it, and a link that crosses it has one
//! literal inside it, whose region then differs from the region of the
//! literal outside). The search keeps every literal's region, which a link
//! rewrites inside its span and an unlink restores.
//!
//! A link also leaves the literals strictly between its two ends to be
//! linked among themselves, since a link from inside to outside would
//! cross it; so every atom must have as many `a` as `~a` there, the linked
//! ones being pairs already. That is a function of the two ends alone,
//! compared through prefix sums of a hash of the signed atoms: equal
//! counts give equal sums, so the comparison never rejects a link a net
//! could hold, and two unequal counts that happen to give equal sums let
//! through a link that the search finds dead later.

use super::Reason;
use super::net::{Correctness, Switching};
use crate::fragment::Mode;
use crate::nets::VertexId;
use crate::occurrences::{Forest, OccId, Sign};
use crate::sequents::Atom;

/// The raw index that stands for "no literal": the region outside every
/// link, and an occurrence that is no literal.
const NONE: u32 = u32::MAX;

/// The classical criterion with planarity: the regions of the literals
/// between the links made, and the prefix sums that say which spans are
/// balanced.
pub(super) struct Planar {
    /// The Danos–Regnier criterion of the same structure.
    switching: Switching,
    /// Per occurrence, its place among the literals in id order, or
    /// `NONE` for a compound one. A place is below the number of
    /// literals, which is below the forest's length and fits a `u32`.
    place: Box<[u32]>,
    /// Per place, its region: the place of the left end of the innermost
    /// link around it, or `NONE`.
    region: Box<[u32]>,
    /// Per place `i`, the sum over the literals at places before `i` of
    /// their atom's weight, added for `a` and taken for `~a`, wrapping:
    /// a hash of the signed counts, compared and never read otherwise,
    /// so that a wrap costs no soundness.
    prefix: Box<[u64]>,
    /// The links made, as the places of their two ends, the left one
    /// first, with the region the left end was in.
    made: Vec<(u32, u32, u32)>,
}

impl Planar {
    /// The criterion over a forest of unit-free MLL without a link, in an
    /// ordered mode.
    pub(super) fn new(forest: &Forest, mode: Mode) -> Self {
        debug_assert!(mode.is_ordered(), "the planar criterion keeps an order");
        let switching = Switching::new(forest, mode);
        let mut place = vec![NONE; forest.len()].into_boxed_slice();
        let mut prefix = vec![0u64];
        let mut sum = 0u64;
        for o in forest.ids() {
            let (Some(atom), Some(sign)) = (forest.atom(o), forest.sign(o)) else {
                continue;
            };
            place[o.index()] = (prefix.len() - 1) as u32;
            let weight = weight(atom);
            sum = match sign {
                Sign::Atom => sum.wrapping_add(weight),
                Sign::Dual => sum.wrapping_sub(weight),
            };
            prefix.push(sum);
        }
        let literals = prefix.len() - 1;
        Self {
            switching,
            place,
            region: vec![NONE; literals].into_boxed_slice(),
            prefix: prefix.into_boxed_slice(),
            made: Vec::with_capacity(literals / 2),
        }
    }

    /// Returns the classical criterion, whose structure holds the links.
    pub(super) fn switching(self) -> Switching {
        self.switching
    }

    /// The places of two literals, the smaller first.
    fn ends(&self, x: OccId, y: OccId) -> (u32, u32) {
        let (p, q) = (self.place[x.index()], self.place[y.index()]);
        (p.min(q), p.max(q))
    }
}

/// The weight of an atom in the prefix sums: a mix of its index, odd, so
/// that different atoms' weights are unlikely to cancel.
fn weight(atom: Atom) -> u64 {
    let mut z = u64::from(atom.get()).wrapping_add(0x9e37_79b9_7f4a_7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    (z ^ (z >> 31)) | 1
}

impl Correctness for Planar {
    fn forest(&self) -> &Forest {
        self.switching.forest()
    }

    fn mate(&self, x: OccId) -> Option<OccId> {
        self.switching.mate(x)
    }

    fn links(&self) -> &[(VertexId, VertexId)] {
        self.switching.links()
    }

    fn is_complete(&self) -> bool {
        self.switching.is_complete()
    }

    /// Rejects a link whose two literals lie in different regions (it
    /// would cross a link made, in every completion), or between which
    /// some atom has more `a` than `~a` or fewer (those would have to
    /// link across it), then asks the classical criterion.
    fn admits(&self, x: OccId, y: OccId) -> bool {
        let (left, right) = self.ends(x, y);
        self.region[left as usize] == self.region[right as usize]
            && self.prefix[left as usize + 1] == self.prefix[right as usize]
            && self.switching.admits(x, y)
    }

    /// Links the literals and moves the places between them that were in
    /// their region into the link's own.
    fn link(&mut self, x: OccId, y: OccId) -> Result<(), Reason> {
        let (left, right) = self.ends(x, y);
        let outer = self.region[left as usize];
        for region in &mut self.region[left as usize + 1..right as usize] {
            if *region == outer {
                *region = left;
            }
        }
        self.made.push((left, right, outer));
        self.switching.link(x, y)
    }

    /// Takes the last link back, and with it its region.
    fn unlink(&mut self) -> OccId {
        let (left, right, outer) = self.made.pop().expect("a link to take back");
        for region in &mut self.region[left as usize + 1..right as usize] {
            if *region == left {
                *region = outer;
            }
        }
        self.switching.unlink()
    }

    /// The classical criterion's exact test: every link made crosses no
    /// other, so a complete linking that passes is a planar proof net.
    fn test(&mut self) -> bool {
        let correct = self.switching.test();
        debug_assert!(
            !self.is_complete() || self.switching.structure().is_planar().is_ok(),
            "every link was admitted in its region"
        );
        correct
    }

    /// The classical balance: a planar proof net is a proof net, so a
    /// component that holds more literals of an atom than the atom has
    /// pairs left has no planar completion either.
    fn balanced(&self, atom: Atom, remaining: u32, tally: &mut [u32]) -> bool {
        self.switching.balanced(atom, remaining, tally)
    }
}

#[cfg(all(test, feature = "parse"))]
mod tests {
    use crate::fragment::Mode;
    use crate::nets::{Criterion, ProofStructure, VertexId};
    use crate::occurrences::{Forest, OccId, Sign};
    use crate::search::generate::{self, Rng, Rules, Tree};
    use crate::search::{Engine, Options, Verdict, prove};
    use crate::sequents::{Atom, Sequent};

    /// The formulas of a random derivation of cyclic MLL in their cyclic
    /// order, with up to `budget` rules over `atoms` atoms: an axiom, a
    /// `⅋` of two neighbours, or a `⊗` of the last formula of one
    /// derivation and the first of another, each sequence turned at
    /// random as the cyclic calculus allows.
    fn cyclic(rng: &mut Rng, atoms: u8, budget: usize) -> Vec<Tree> {
        let turned = |rng: &mut Rng, mut formulas: Vec<Tree>| {
            let by = rng.below(formulas.len());
            formulas.rotate_left(by);
            formulas
        };
        let mut formulas = if budget <= 1 || rng.below(4) == 0 {
            let a = rng.below(usize::from(atoms)) as u8;
            vec![Tree::Atom(a), Tree::Dual(a)]
        } else if rng.below(2) == 0 {
            let mut left = cyclic(rng, atoms, budget / 2);
            let mut right = cyclic(rng, atoms, budget - budget / 2);
            let a = left.pop().unwrap();
            let b = right.remove(0);
            left.push(Tree::Tensor(Box::new(a), Box::new(b)));
            left.extend(right);
            left
        } else {
            let mut formulas = cyclic(rng, atoms, budget - 1);
            if formulas.len() > 2 {
                let b = formulas.pop().unwrap();
                let a = formulas.pop().unwrap();
                formulas.push(Tree::Par(Box::new(a), Box::new(b)));
            }
            formulas
        };
        formulas = turned(rng, formulas);
        formulas
    }

    /// Whether some planar linking of the literals of `positives` to
    /// unlinked partners completes `net` into a proof net of the
    /// commutative criterion: every linking is tried that has no two
    /// links whose ends alternate, compared pairwise.
    fn some_planar_net(net: &mut ProofStructure, positives: &[OccId]) -> bool {
        let Some((&x, rest)) = positives.split_first() else {
            return net.is_correct(|_| false).is_ok();
        };
        let forest = net.forest().clone();
        for &y in forest.literals(forest.atom(x).unwrap(), Sign::Dual) {
            let (u, v) = (x.min(y), x.max(y));
            let crosses = net.links().iter().any(|&(p, q)| {
                let (p, q) = (p.occ().min(q.occ()), p.occ().max(q.occ()));
                (u < p && p < v && v < q) || (p < u && u < q && q < v)
            });
            if net.partner(VertexId::of(y)).is_none() && !crosses {
                net.link(VertexId::of(x), VertexId::of(y)).unwrap();
                let found = some_planar_net(net, rest);
                net.unlink();
                if found {
                    return true;
                }
            }
        }
        false
    }

    /// Whether some planar linking of the sequent is a proof net, or
    /// `None` when an atom has more than six pairs or its counts differ.
    fn every_planar_linking(sequent: &Sequent) -> Option<bool> {
        let forest = Forest::new(sequent).unwrap();
        let mut positives = Vec::new();
        for a in 0..sequent.atom_names().len() as u32 {
            let a = Atom::new(a);
            let (p, q) = (
                forest.literals(a, Sign::Atom),
                forest.literals(a, Sign::Dual),
            );
            if p.len() != q.len() || p.len() > 6 {
                return None;
            }
            positives.extend_from_slice(p);
        }
        let mut net = ProofStructure::new(forest, Criterion::MLL).unwrap();
        Some(some_planar_net(&mut net, &positives))
    }

    /// The planar engine, the default of the cyclic mode, agrees with the
    /// enumeration of every planar linking on random derivations of the
    /// cyclic calculus, their mutants and random balanced sequents, every
    /// proof through the checker in the cyclic mode and its net planar;
    /// on the same sequents the classical verdict is never below it.
    #[test]
    fn agrees_with_every_planar_linking() {
        let rules = Rules {
            units: false,
            additives: false,
            mix: false,
            exponentials: false,
        };
        let mut rng = Rng::new(3_600);
        let (mut decided, mut provable, mut only_classically) = (0, 0, 0);
        for round in 0..900 {
            let formulas = match round % 3 {
                0 => {
                    let (atoms, budget) = (1 + rng.below(3) as u8, 1 + rng.below(9));
                    cyclic(&mut rng, atoms, budget)
                }
                1 => {
                    let (atoms, budget) = (1 + rng.below(3) as u8, 1 + rng.below(9));
                    let mut formulas = cyclic(&mut rng, atoms, budget);
                    generate::mutate(&mut rng, &mut formulas, 3);
                    formulas
                }
                _ => {
                    let (atoms, pairs) = (1 + rng.below(2) as u8, 1 + rng.below(6));
                    let mut formulas = generate::balanced(&mut rng, atoms, pairs, false);
                    if rng.below(2) == 0 {
                        let budget = 2 + rng.below(6);
                        formulas = generate::provable(&mut rng, rules, 2, budget).formulas;
                    }
                    formulas
                }
            };
            let text = generate::sequent(&formulas);
            let sequent: Sequent = text.parse().unwrap();
            let Some(expected) = every_planar_linking(&sequent) else {
                continue;
            };
            let outcome = prove(&sequent, Mode::CYCLIC, &Options::default()).unwrap();
            assert_eq!(outcome.engine, Engine::Net, "{text}");
            let found = match outcome.verdict {
                Verdict::Proved(proof) => {
                    assert_eq!(proof.check(Mode::CYCLIC), Ok(()), "{text}");
                    let net = outcome.net.expect("the net engine's net");
                    assert_eq!(net.is_correct(|_| false), Ok(()), "{text}");
                    assert_eq!(net.criterion(), Criterion::CYCLIC, "{text}");
                    true
                }
                Verdict::Unprovable(_) => false,
                Verdict::Unknown(reason) => panic!("{text}: {reason}"),
            };
            assert_eq!(found, expected, "{text}");
            let classical = prove(&sequent, Mode::CLASSICAL, &Options::default()).unwrap();
            let classically = matches!(classical.verdict, Verdict::Proved(_));
            assert!(classically || !found, "{text}");
            decided += 1;
            provable += usize::from(found);
            only_classically += usize::from(classically && !found);
        }
        assert!(
            decided > 600 && provable > 200 && only_classically > 50,
            "{decided} decided, {provable} provable, {only_classically} only classically"
        );
    }
}
