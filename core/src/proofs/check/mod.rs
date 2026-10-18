// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The proof checker: it derives, node by node, the sequent a proof term
//! proves under the rules of the dyadic sequent calculus and accepts the
//! term only if the root derives the proof's sequent. It is the reference
//! for what a proof is, so it shares no code with any search engine and must
//! never: an engine's proofs are validated by something that cannot repeat
//! the engine's mistakes.
//!
//! The pass is bottom-up, one node at a time in arena order, and never
//! recurses. What a subproof proves is a `State`, a sequent kept as two
//! tables of its members; the one rule whose conclusion the premises do not
//! determine, `⊤`, is handled by a flag that says the subproof proves its
//! sequent under any further context. A node's sequent is built from its
//! premises' own: the last node to read a premise takes its sequent and
//! changes it in place, an earlier reader a copy, so what the pass holds at
//! any moment is the sequents some later node still reads. Those are
//! counted in bytes as they come and go, and a pass that would hold more
//! than its bound ends with a refusal, which is no verdict on the proof.
//!
//! No arithmetic here may wrap, in any build: a term from a file is
//! hostile input, and a counter that wrapped once made a term a proof
//! that was none. So every integer says at its declaration why it stays
//! in range, or saturates into a refusal.
//!
//! In intuitionistic mode the same pass also checks the one-succedent
//! condition against the sequent's intuitionistic reading: every derived
//! linear zone holds at most one formula in output position, exactly one
//! unless a `⊤` above absorbs the rest, a `⊤` never absorbs a second
//! output, weakening never discards the goal, and Mix has no premise with
//! a goal at all. Every intuitionistic rule is a classical rule on the
//! one-sided sequent, so nothing else is intuitionistic about a proof.

mod error;
mod pass;
mod zones;

pub use error::{CheckError, Dyadic, Fault, Invalid, Refused};
pub(crate) use pass::{Allowance, Facts, Observer, afford, check, check_within, examine, reading};
pub(crate) use zones::State;

#[cfg(all(test, feature = "parse"))]
mod tests {
    use super::pass::tables;
    use super::zones::{Bag, Zone, table_bytes};
    use super::*;
    use crate::fragment::Mode;
    use crate::limits::{Phase, Refusal};
    use crate::occurrences::{Forest, Member, OccId};
    use crate::proofs::oracle;
    use crate::proofs::{Branch, Node, NodeId, Proof};
    use crate::search::generate::Rng;
    use crate::{Error, Sequent};

    /// Wraps a raw member id.
    const fn o(id: u32) -> Member {
        Member::new(id)
    }

    /// Wraps a raw node id.
    const fn n(id: u32) -> NodeId {
        NodeId::new(id)
    }

    /// Builds a proof of the sequent `input` parses to, with the last node
    /// as the root. Occurrence ids are the preorder numbering, which each
    /// test lists in a comment.
    fn proof(input: &str, nodes: Vec<Node>) -> Proof {
        let s: Sequent = input.parse().unwrap_or_else(|e| panic!("{input:?}: {e}"));
        let root = n(nodes.len() as u32 - 1);
        Proof::new(Forest::new(&s).unwrap(), nodes, root).unwrap()
    }

    /// A proof with one node changed at random, or none when the change
    /// does not leave a proof term: another occurrence, another premise,
    /// another rule on the same operands, the premises swapped, or a
    /// weakening in the node's place.
    fn mutant(rng: &mut Rng, proof: &Proof) -> Option<Proof> {
        use Node::*;
        let mut nodes = proof.nodes().to_vec();
        let i = rng.below(nodes.len());
        let occurrences = proof.forest().len();
        let occ = |rng: &mut Rng| o(rng.below(occurrences) as u32);
        let earlier = |rng: &mut Rng| n(rng.below(i.max(1)) as u32);
        let unary = |k: usize, o: OccId, p: NodeId| match k % 8 {
            0 => Bot(Member::from(o), p),
            1 => Par(Member::from(o), p),
            2 => Bang(Member::from(o), p),
            3 => Quest(Member::from(o), p),
            4 => Copy(Member::from(o), p),
            5 => Weaken(Member::from(o), p),
            6 => Plus(Member::from(o), Branch::Left, p),
            _ => Plus(Member::from(o), Branch::Right, p),
        };
        let choice = rng.below(5);
        nodes[i] = match (nodes[i], choice) {
            (Ax(a, _), 0) => Ax(a, occ(rng)),
            (Ax(a, b), 1) => Ax(b, a),
            (Ax(a, _), 2) => One(a),
            (Ax(_, b), 3) => Top(b),
            (One(a) | Top(a), 0 | 1) => Ax(a, occ(rng)),
            (One(a), _) => Top(a),
            (Top(a), _) => One(a),
            (Tensor(_, l, r) | With(_, l, r), 0) => Tensor(occ(rng), l, r),
            (Tensor(a, _, r), 1) => Tensor(a, earlier(rng), r),
            (With(a, l, _), 1) => With(a, l, earlier(rng)),
            (Tensor(a, l, r), 2) => With(a, l, r),
            (With(a, l, r), 2) => Tensor(a, l, r),
            (Tensor(a, l, r), 3) => Tensor(a, r, l),
            (With(a, l, r), 3) => With(a, r, l),
            (Tensor(_, l, r) | With(_, l, r), _) => Mix(l, r),
            (Mix(l, r), 0 | 1) => Tensor(occ(rng), l, r),
            (Mix(l, r), 2) => With(occ(rng), l, r),
            (Mix(l, r), 3) => Mix(r, l),
            (Mix(l, _), _) => Mix(l, earlier(rng)),
            (node, 0) => unary(rng.below(8), occ(rng).occ(), node.premises().next()?),
            (node, 1) => unary(rng.below(8), (node.members().next()?).occ(), earlier(rng)),
            (node, 2) => unary(
                rng.below(8),
                (node.members().next()?).occ(),
                node.premises().next()?,
            ),
            (node, 3) => Weaken(occ(rng), node.premises().next()?),
            (Ax(a, _), _) => Ax(a, a),
            (node, _) => {
                let p = node.premises().next()?;
                Mix(p, earlier(rng))
            }
        };
        Proof::new(proof.forest().clone(), nodes, proof.root()).ok()
    }

    /// The checker agrees with its first implementation, which keeps every
    /// node's sequent, on the verdict and on the error: on the proofs the
    /// engines find of generated sequents, classical and intuitionistic,
    /// and of the smallest instance of every family, in the mode they were
    /// found in and in the others, and on mutants of them.
    #[test]
    fn agrees_with_the_first_implementation() {
        let classical = Mode::CLASSICAL;
        let proofs = oracle::proofs();
        assert!(proofs.len() > 1000, "{} proofs", proofs.len());

        let modes = [
            classical,
            classical.with_affine().with_mix(),
            Mode::INTUITIONISTIC,
            Mode::INTUITIONISTIC.with_affine(),
        ];
        let mut rng = Rng::new(7);
        let (mut mutants, mut rejected) = (0, 0);
        for (proof, mode) in &proofs {
            assert_eq!(check(proof, *mode), Ok(()), "{}", proof.sequent());
            for mode in modes {
                assert_eq!(
                    check(proof, mode),
                    oracle::check(proof, mode),
                    "{} in {mode} mode",
                    proof.sequent()
                );
            }
            for _ in 0..8 {
                let Some(mutant) = mutant(&mut rng, proof) else {
                    continue;
                };
                mutants += 1;
                for mode in [*mode, classical.with_affine().with_mix()] {
                    let result = check(&mutant, mode);
                    rejected += u32::from(result.is_err());
                    assert_eq!(
                        result,
                        oracle::check(&mutant, mode),
                        "{:?} of {} in {mode} mode",
                        mutant.nodes(),
                        mutant.sequent()
                    );
                }
            }
        }
        assert!(mutants > 5000, "{mutants} mutants");
        assert!(
            rejected > mutants,
            "{rejected} of {mutants} mutants rejected"
        );
    }

    /// Every rule applied correctly is accepted: the multiplicatives and
    /// their units, the additives with `⊤` absorbing its context, the
    /// exponentials in dyadic form, weakening in affine mode and Mix.
    #[test]
    fn accepts_every_rule() {
        use Node::*;
        let classical = Mode::CLASSICAL;
        for (input, nodes, mode) in [
            // ⊢ ~A, A ⊗ ~B, B: 0 ~A, 1 ⊗, 2 A, 3 ~B, 4 B
            (
                "A, A -o B |- B",
                vec![Ax(o(0), o(2)), Ax(o(3), o(4)), Tensor(o(1), n(0), n(1))],
                classical,
            ),
            // ⊢ A ⅋ ~A: 0 ⅋, 1 A, 2 ~A
            (
                "|- A par ~A",
                vec![Ax(o(2), o(1)), Par(o(0), n(0))],
                classical,
            ),
            // ⊢ 1, ⊥
            ("|- 1, bot", vec![One(o(0)), Bot(o(1), n(0))], classical),
            // ⊢ A & B, ~A ⊕ ~B: 0 &, 1 A, 2 B, 3 ⊕, 4 ~A, 5 ~B
            (
                "|- A & B, ~A + ~B",
                vec![
                    Ax(o(1), o(4)),
                    Plus(o(3), Branch::Left, n(0)),
                    Ax(o(2), o(5)),
                    Plus(o(3), Branch::Right, n(2)),
                    With(o(0), n(1), n(3)),
                ],
                classical,
            ),
            // ⊢ ⊤, A: ⊤ absorbs A.
            ("|- top, A", vec![Top(o(0))], classical),
            // ⊢ A & ⊤, ~A: 0 &, 1 A, 2 ⊤, 3 ~A: the ⊤ side adapts to the
            // other.
            (
                "|- A & top, ~A",
                vec![Ax(o(1), o(3)), Top(o(2)), With(o(0), n(0), n(1))],
                classical,
            ),
            // ⊢ ⊤ ⊗ A, ~A, B: 0 ⊗, 1 ⊤, 2 A, 3 ~A, 4 B: the ⊤ side of the
            // split takes B.
            (
                "|- top * A, ~A, B",
                vec![Top(o(1)), Ax(o(2), o(3)), Tensor(o(0), n(0), n(1))],
                classical,
            ),
            // ⊢ ?~A, A: 0 ?, 1 ~A, 2 A: a copy, then the ? step.
            (
                "!A |- A",
                vec![Ax(o(1), o(2)), Copy(o(1), n(0)), Quest(o(0), n(1))],
                classical,
            ),
            // ⊢ ?~A, A ⊗ A: 0 ?, 1 ~A, 2 ⊗, 3 A, 4 A: two copies of one
            // occurrence.
            (
                "!A |- A * A",
                vec![
                    Ax(o(1), o(3)),
                    Copy(o(1), n(0)),
                    Ax(o(1), o(4)),
                    Copy(o(1), n(2)),
                    Tensor(o(2), n(1), n(3)),
                    Quest(o(0), n(4)),
                ],
                classical,
            ),
            // ⊢ ?~A, 1: 0 ?, 1 ~A, 2 1: an unused ? formula.
            ("!A |- 1", vec![One(o(2)), Quest(o(0), n(0))], classical),
            // ⊢ ?~A, ?(A ⊗ ~B), !B: 0 ?, 1 ~A, 2 ?, 3 ⊗, 4 A, 5 ~B, 6 !, 7 B:
            // promotion under two ? formulas.
            (
                "!A, !(A -o B) |- !B",
                vec![
                    Ax(o(1), o(4)),
                    Copy(o(1), n(0)),
                    Ax(o(5), o(7)),
                    Tensor(o(3), n(1), n(2)),
                    Copy(o(3), n(3)),
                    Bang(o(6), n(4)),
                    Quest(o(2), n(5)),
                    Quest(o(0), n(6)),
                ],
                classical,
            ),
            // ⊢ !(A ⅋ ~A): 0 !, 1 ⅋, 2 A, 3 ~A: promotion of a closed
            // formula.
            (
                "|- !(A par ~A)",
                vec![Ax(o(2), o(3)), Par(o(1), n(0)), Bang(o(0), n(1))],
                classical,
            ),
            // ⊢ ~A, ~B, A: weakening of ~B.
            (
                "A, B |- A",
                vec![Ax(o(0), o(2)), Weaken(o(1), n(0))],
                classical.with_affine(),
            ),
            // ⊢ 1, ?A: weakening of a ? formula needs no affine mode.
            ("|- 1, ?A", vec![One(o(0)), Weaken(o(1), n(0))], classical),
            // ⊢ ?⊤, !0: 0 ?, 1 ⊤, 2 !, 3 0: the ⊤ stands in for the 0 the
            // promotion consumes, and the zone is empty after it.
            (
                "|- ?top, !0",
                vec![
                    Top(o(1)),
                    Copy(o(1), n(0)),
                    Bang(o(2), n(1)),
                    Quest(o(0), n(2)),
                ],
                classical,
            ),
            // ⊢ ⊤ & ⊤, A: both sides absorb.
            (
                "|- top & top, A",
                vec![Top(o(1)), Top(o(2)), With(o(0), n(0), n(1))],
                classical,
            ),
            // ⊢ ⊤, ?A: 0 ⊤, 1 ?, 2 A: a copy the ⊤ stands in for.
            (
                "|- top, ?A",
                vec![Top(o(0)), Copy(o(2), n(0)), Quest(o(1), n(1))],
                classical,
            ),
            // ⊢ ?(?~A), A ⊗ A: 0 ?, 1 ?, 2 ~A, 3 ⊗, 4 A, 5 A: the inner ?
            // is copied twice and its ? step taken twice on one path.
            (
                "|- ?(?~A), A * A",
                vec![
                    Ax(o(2), o(4)),
                    Copy(o(2), n(0)),
                    Ax(o(2), o(5)),
                    Copy(o(2), n(2)),
                    Tensor(o(3), n(1), n(3)),
                    Quest(o(1), n(4)),
                    Copy(o(1), n(5)),
                    Quest(o(1), n(6)),
                    Copy(o(1), n(7)),
                    Quest(o(0), n(8)),
                ],
                classical,
            ),
            // ⊢ ?A, ?~A: 0 ?, 1 A, 2 ?, 3 ~A: one subproof shared by both
            // sides of a Mix.
            (
                "|- ?A, ?~A",
                vec![
                    Ax(o(1), o(3)),
                    Copy(o(1), n(0)),
                    Copy(o(3), n(1)),
                    Mix(n(2), n(2)),
                    Quest(o(2), n(3)),
                    Quest(o(0), n(4)),
                ],
                classical.with_mix(),
            ),
            // ⊢ ~A ⅋ ~B, A ⅋ B: 0 ⅋, 1 ~A, 2 ~B, 3 ⅋, 4 A, 5 B: needs Mix.
            (
                "A * B |- A par B",
                vec![
                    Ax(o(1), o(4)),
                    Ax(o(2), o(5)),
                    Mix(n(0), n(1)),
                    Par(o(0), n(2)),
                    Par(o(3), n(3)),
                ],
                classical.with_mix(),
            ),
        ] {
            let p = proof(input, nodes);
            assert_eq!(p.check(mode), Ok(()), "{input:?}");
        }
    }

    /// Every rule misapplied is rejected at the node that misapplies it,
    /// with the problem named; a rule the mode forbids is rejected too, and
    /// so is a root that concludes something else.
    #[test]
    fn rejects_every_misuse() {
        use Fault::*;
        use Node::*;
        let classical = Mode::CLASSICAL;
        for (input, nodes, mode, node, problem) in [
            // Axiom on two copies of one literal.
            ("|- A, A", vec![Ax(o(0), o(1))], classical, 0, NotDual),
            // Axiom on a connective: 0 ⊗, 1 A, 2 B, 3 ~A.
            (
                "|- A * B, ~A",
                vec![Ax(o(0), o(3))],
                classical,
                0,
                Kind { member: o(0) },
            ),
            // ⊗ with its premises swapped: 0 ~A, 1 ⊗, 2 A, 3 ~B, 4 B.
            (
                "A, A -o B |- B",
                vec![Ax(o(0), o(2)), Ax(o(3), o(4)), Tensor(o(1), n(1), n(0))],
                classical,
                2,
                Missing {
                    premise: 0,
                    member: o(2),
                },
            ),
            // ⅋ on a ⊗ occurrence.
            (
                "A, A -o B |- B",
                vec![Ax(o(0), o(2)), Par(o(1), n(0))],
                classical,
                1,
                Kind { member: o(1) },
            ),
            // ⅋ whose premise lacks a subformula: 0 A, 1 ⅋, 2 ~A, 3 B.
            (
                "|- A, ~A par B",
                vec![Ax(o(0), o(2)), Par(o(1), n(0))],
                classical,
                1,
                Missing {
                    premise: 0,
                    member: o(3),
                },
            ),
            // 1 on ⊥.
            (
                "|- bot",
                vec![One(o(0))],
                classical,
                0,
                Kind { member: o(0) },
            ),
            // ⊥ on 1.
            (
                "|- 1, bot",
                vec![One(o(0)), Bot(o(0), n(0))],
                classical,
                1,
                Kind { member: o(0) },
            ),
            // & whose premises consume different contexts: 0 &, 1 A, 2 B,
            // 3 ~A, 4 ~B.
            (
                "|- A & B, ~A, ~B",
                vec![Ax(o(1), o(3)), Ax(o(2), o(4)), With(o(0), n(0), n(1))],
                classical,
                2,
                Differ,
            ),
            // & whose ⊤ side needs more than the other side has: the root
            // then concludes ⊢ A & ⊤, ~A without ~B.
            (
                "|- A & top, ~A, ~B",
                vec![Ax(o(1), o(3)), Top(o(2)), With(o(0), n(0), n(1))],
                classical,
                2,
                Conclusion {
                    derived: Dyadic {
                        theta: vec![],
                        gamma: vec![o(0), o(3)],
                        any: false,
                    },
                },
            ),
            // ⊕ on the side the premise does not prove: 0 ⊕, 1 A, 2 B, 3 ~A.
            (
                "|- A + B, ~A",
                vec![Ax(o(1), o(3)), Plus(o(0), Branch::Right, n(0))],
                classical,
                1,
                Missing {
                    premise: 0,
                    member: o(2),
                },
            ),
            // ⊤ on 0.
            ("|- 0", vec![Top(o(0))], classical, 0, Kind { member: o(0) }),
            // Promotion with a linear formula in the context: 0 ~A, 1 !, 2 A.
            (
                "A |- !A",
                vec![Ax(o(0), o(2)), Bang(o(1), n(0))],
                classical,
                1,
                NotEmpty,
            ),
            // A copy of a formula under !: 0 !, 1 ~A, 2 A.
            (
                "?A |- A",
                vec![Ax(o(1), o(2)), Copy(o(1), n(0))],
                classical,
                1,
                NotUnderQuest { member: o(1) },
            ),
            // A copy of a formula the premise does not hold: 0 ?, 1 ~A, 2 1.
            (
                "!A |- 1",
                vec![One(o(2)), Copy(o(1), n(0))],
                classical,
                1,
                Missing {
                    premise: 0,
                    member: o(1),
                },
            ),
            // A copy without the ? step below it: the root still needs ~A
            // in the unrestricted zone.
            (
                "!A |- A",
                vec![Ax(o(1), o(2)), Copy(o(1), n(0))],
                classical,
                1,
                Conclusion {
                    derived: Dyadic {
                        theta: vec![o(1)],
                        gamma: vec![o(2)],
                        any: false,
                    },
                },
            ),
            // ? on a ! occurrence.
            (
                "|- !(A par ~A)",
                vec![Ax(o(2), o(3)), Par(o(1), n(0)), Quest(o(0), n(1))],
                classical,
                2,
                Kind { member: o(0) },
            ),
            // Weakening outside affine mode.
            (
                "A, B |- A",
                vec![Ax(o(0), o(2)), Weaken(o(1), n(0))],
                classical,
                1,
                Forbidden,
            ),
            // Mix without Mix.
            (
                "|- A, ~A, B, ~B",
                vec![Ax(o(0), o(1)), Ax(o(2), o(3)), Mix(n(0), n(1))],
                classical,
                2,
                Forbidden,
            ),
            // A proof of a smaller sequent.
            (
                "|- A, ~A, 1",
                vec![Ax(o(0), o(1))],
                classical,
                0,
                Conclusion {
                    derived: Dyadic {
                        theta: vec![],
                        gamma: vec![o(0), o(1)],
                        any: false,
                    },
                },
            ),
            // Intuitionistic mode needs an intuitionistic sequent, whatever
            // the proof.
            (
                "|- A par B",
                vec![Ax(o(1), o(2))],
                Mode::INTUITIONISTIC,
                0,
                Shape(crate::occurrences::ShapeError::Formula {
                    occurrence: o(0).occ(),
                }),
            ),
        ] {
            let p = proof(input, nodes);
            let e = p.check(mode).unwrap_err();
            let message = e.to_string();
            assert_eq!(
                (e.node(), e.fault()),
                (n(node), Some(&problem)),
                "{input:?}: {message}"
            );
        }
    }

    /// A term that doubles a zone at every node is refused where the zone
    /// outgrows what the rest of the proof can consume, long before a
    /// counter is full: this one mixes 2⁶⁴ copies of `⊢ 1`, promotes `⊥`
    /// over them and mixes one more in, and `⊢ !⊥, 1` has no proof.
    #[test]
    fn refuses_a_zone_too_large_to_conclude() {
        use Node::*;
        // ⊢ !⊥, 1: 0 !, 1 ⊥, 2 1
        let mut nodes = vec![One(o(2))];
        for i in 0..63 {
            nodes.push(Mix(n(i), n(i)));
        }
        let mut all = n(0);
        for i in 1..64 {
            nodes.push(Mix(all, n(i)));
            all = n(nodes.len() as u32 - 1);
        }
        nodes.push(Mix(all, n(0)));
        nodes.push(Bot(o(1), n(nodes.len() as u32 - 1)));
        nodes.push(Bang(o(0), n(nodes.len() as u32 - 1)));
        nodes.push(Mix(n(nodes.len() as u32 - 1), n(0)));
        let p = proof("|- !bot, 1", nodes);
        let mode = Mode::CLASSICAL.with_mix();
        let e = p.check(mode).unwrap_err();
        // Node 8 holds 256 copies with 122 nodes to come.
        assert_eq!((e.node(), e.fault()), (n(8), Some(&Fault::Surplus)));
        assert_eq!(Err(e.clone()), oracle::check(&p, mode));
        // The pass that adds up weights for the size refuses it there too.
        assert_eq!(p.size_of(false), Err(e));
    }

    /// A proof of `⊢ 1, ⊥, …, ⊥, T` with `width` formulas `⊥` and `T` a
    /// balanced tree of `&` over `2^depth` leaves `⊥`: the `⊥` formulas are
    /// introduced once, into one sequent that every leaf of the tree then
    /// reads, and all the leaves come before the first `&` that joins two
    /// of them. So a pass holds a copy of that sequent for every leaf.
    fn shared(width: u32, depth: u32) -> Proof {
        use crate::sequents::{Term, TermId};
        use Node::*;
        // Terms: 0 is 1, 1 is ⊥, and 2 + j the tree of depth j + 1 over
        // two of depth j, so the arena has the tree once per depth.
        let mut terms = vec![Term::One, Term::Bot];
        for j in 0..depth {
            terms.push(Term::With(TermId::new(1 + j), TermId::new(1 + j)));
        }
        let mut roots = vec![TermId::new(0)];
        roots.extend((0..width).map(|_| TermId::new(1)));
        roots.push(TermId::new(1 + depth));
        let sequent = Sequent {
            terms,
            roots,
            atoms: vec![],
            antecedents: None,
        };
        // Occurrences: 0 is 1, 1 to `width` the ⊥ formulas, then the tree
        // in preorder.
        let mut nodes = vec![One(o(0))];
        for i in 1..=width {
            nodes.push(Bot(o(i), n(i - 1)));
        }
        let context = n(width);
        // The tree's occurrences with their depths, in preorder: a tree of
        // depth j has 2^(j + 1) − 1 of them.
        let mut tree = vec![];
        let mut stack = vec![(width + 1, depth)];
        while let Some((at, d)) = stack.pop() {
            tree.push((at, d));
            if d > 0 {
                stack.push((at + (1 << d), d - 1));
                stack.push((at + 1, d - 1));
            }
        }
        let mut proved = crate::hash::HashMap::default();
        for &(at, d) in &tree {
            if d == 0 {
                proved.insert(at, n(nodes.len() as u32));
                nodes.push(Bot(o(at), context));
            }
        }
        // A subtree's occurrences follow its root, so the reverse of the
        // preorder has both subformulas before their `&`.
        for &(at, d) in tree.iter().rev() {
            if d > 0 {
                let (l, r) = (proved[&(at + 1)], proved[&(at + (1 << d))]);
                proved.insert(at, n(nodes.len() as u32));
                nodes.push(With(o(at), l, r));
            }
        }
        let root = n(nodes.len() as u32 - 1);
        Proof::new(Forest::new(&sequent).unwrap(), nodes, root).unwrap()
    }

    /// A zone's count of its bytes stays what its table takes when a
    /// member that is there already is added again: the table is full at
    /// fourteen members in sixteen slots, and would double to make room
    /// before it looked the member up.
    #[test]
    fn a_member_twice_grows_no_table() {
        let mut zone = Zone::default();
        for id in 0..14 {
            assert!(zone.insert(o(id).occ()));
        }
        let (bytes, room) = (zone.bytes(), zone.members.capacity());
        assert_eq!((bytes, room), (16 * 5 + 16, 14));
        for id in 0..14 {
            assert!(!zone.insert(o(id).occ()));
        }
        assert_eq!((zone.bytes(), zone.members.capacity()), (bytes, room));
    }

    /// The pass's own tables are asked for before it judges anything: a
    /// term whose first node is no rule is refused, not judged, under a
    /// bound of a byte less than its tables, and judged at the tables'
    /// size exactly.
    #[test]
    fn refuses_its_tables_before_it_judges() {
        let (o, mode) = (|i| Member::new(i), Mode::CLASSICAL);
        let crossed = proof("|- a, a", vec![Node::Ax(o(0), o(1))]);
        // A count of four bytes and a pointer for the one node.
        let bytes = tables(&crossed);
        assert_eq!(bytes, 4 + size_of::<usize>() as u64);
        let within = |bytes| {
            let limits = crate::Limits::default().with_memory_bytes(Some(bytes));
            crossed.check_within(mode, &limits, |_| false)
        };
        match within(bytes) {
            Err(CheckError::Invalid(invalid)) => assert_eq!(invalid.fault, Fault::NotDual),
            other => panic!("{other:?}"),
        }
        match within(bytes - 1) {
            Err(CheckError::Refused(refused)) => assert!(
                matches!(
                    refused.refusal,
                    Refusal::Memory { limit_bytes, .. } if limit_bytes == bytes - 1
                ),
                "{refused:?}"
            ),
            other => panic!("{other:?}"),
        }
    }

    /// A proof of `⊢ ⊤, ⊥, …, ⊥, ⊥ & ⊥, …, ⊥ & ⊥` with `width` formulas
    /// `⊥` and `length` of `⊥ & ⊥`: the `⊥` formulas introduced one by one
    /// over the `⊤`, then a chain of `&` nodes each of whose right premises
    /// is that one node, which absorbs both of its subformulas. The chain
    /// is a proof file linear in its length whose pass handles the
    /// node's sequent at every link.
    fn chained(width: u32, length: u32) -> Proof {
        use crate::sequents::{Term, TermId};
        use Node::*;
        let mut roots = vec![TermId::new(0)];
        roots.extend((0..width).map(|_| TermId::new(1)));
        roots.extend((0..length).map(|_| TermId::new(2)));
        let sequent = Sequent {
            terms: vec![
                Term::Top,
                Term::Bot,
                Term::With(TermId::new(1), TermId::new(1)),
            ],
            roots,
            atoms: vec![],
            antecedents: None,
        };
        // Occurrences: 0 is ⊤, 1 to `width` the ⊥ formulas, then each
        // `⊥ & ⊥` and its two subformulas.
        let mut nodes = vec![Top(o(0))];
        for i in 1..=width {
            nodes.push(Bot(o(i), n(i - 1)));
        }
        let context = n(width);
        let mut previous = context;
        for j in 0..length {
            nodes.push(With(o(width + 1 + 3 * j), previous, context));
            previous = n(nodes.len() as u32 - 1);
        }
        Proof::new(Forest::new(&sequent).unwrap(), nodes, previous).unwrap()
    }

    /// A node that many others read costs each of them its sequent: the
    /// pass counts that work, asks the stop by it and refuses past a bound
    /// on it. Counted by nodes, an 8.6 MB file of this shape took 83 s,
    /// between two polls a few seconds apart.
    #[test]
    fn counts_the_work_of_shared_premises() {
        let (proof, mode) = (chained(2000, 2000), Mode::CLASSICAL);
        let mut polls = 0;
        let checked = proof.check_within(mode, &crate::Limits::default(), |_| {
            polls += 1;
            false
        });
        assert_eq!(checked, Ok(()));
        // 4 001 nodes, one poll by their count; each link handles two
        // sequents of thousands of formulas: 176 polls, one per 65 536
        // units, not one per node.
        assert!((100..300).contains(&polls), "{polls} polls");
        let enough = crate::Limits::default().with_work(Some(1 << 40));
        assert_eq!(proof.check_within(mode, &enough, |_| false), Ok(()));
        let limits = crate::Limits::default().with_work(Some(40_010));
        let e = proof.check_within(mode, &limits, |_| false).unwrap_err();
        assert!(
            matches!(&e, CheckError::Refused(r) if r.refusal == Refusal::Work { limit: 40_010 }),
            "{e}"
        );
    }

    /// The bound is exact: the check of `⊢ 1` holds its tables and the
    /// root's sequent, the value and a table of one member, passes at
    /// exactly their bytes and is refused one byte below; and a zone's
    /// bound leaves its counters room.
    #[test]
    fn the_bound_is_exact() {
        let p = proof("|- 1", vec![Node::One(o(0))]);
        let root = size_of::<State>() as u64 + 16 + table_bytes(1, size_of::<(OccId, u32)>());
        let exact = tables(&p) + root;
        let within = |bytes| {
            let limits = crate::Limits::default().with_memory_bytes(Some(bytes));
            p.check_within(Mode::CLASSICAL, &limits, |_| false)
        };
        assert_eq!(within(exact), Ok(()));
        assert!(matches!(within(exact - 1), Err(CheckError::Refused(_))));
        assert!(Bag::MOST < u32::MAX as usize);
    }

    /// A check holds no more than it is allowed: a proof file of a megabyte
    /// whose pass would hold 16 384 copies of a sequent of 14 001 formulas,
    /// over two gigabytes, is refused after a few hundred of them, and the
    /// refusal is no verdict. So are its size and its derivation. The same
    /// proof at a size that fits is valid.
    #[test]
    fn holds_no_more_than_its_bound() {
        let mode = Mode::CLASSICAL;
        let small = shared(200, 6);
        assert_eq!(
            small.check_within(
                mode,
                &crate::Limits::default().with_memory_bytes(None),
                |_| false
            ),
            Ok(())
        );
        assert_eq!(small.check(mode), oracle::check(&small, mode));
        // 201 formulas take 256 slots of nine bytes: under 3 KB a copy, 64
        // copies and as many nodes within 256 KB, and not within 64 KB.
        assert_eq!(
            small.check_within(
                mode,
                &crate::Limits::default().with_memory_bytes(Some(256 << 10)),
                |_| false
            ),
            Ok(())
        );
        let e = small
            .check_within(
                mode,
                &crate::Limits::default().with_memory_bytes(Some(64 << 10)),
                |_| false,
            )
            .unwrap_err();
        assert!(matches!(e, CheckError::Refused(_)), "{e}");
        // The pass's own tables, a count and a pointer for each of 328
        // nodes, are refused before the first node a byte short of them.
        let short = tables(&small) - 1;
        let e = small
            .check_within(
                mode,
                &crate::Limits::default().with_memory_bytes(Some(short)),
                |_| false,
            )
            .unwrap_err();
        assert!(
            matches!(e, CheckError::Refused(_)) && e.node() == n(0),
            "{e}"
        );

        let (width, depth) = (14_000, 14);
        let large = shared(width, depth);
        #[cfg(feature = "serialize")]
        {
            let file = serde_json::to_string(&large).unwrap().len();
            assert!((900_000..1_200_000).contains(&file), "{file} bytes");
        }
        let limit = 64 << 20;
        let e = large
            .check_within(
                mode,
                &crate::Limits::default().with_memory_bytes(Some(limit)),
                |_| false,
            )
            .unwrap_err();
        let CheckError::Refused(refused) = &e else {
            panic!("{e}")
        };
        assert_eq!(
            refused.refusal,
            Refusal::Memory {
                phase: Phase::Check,
                limit_bytes: limit,
                needed_bytes: None
            }
        );
        // A copy takes 16 384 slots of nine bytes, 144 KiB: the bound is
        // reached within 512 leaves of the tree.
        let leaf = e.node().get() - width - 1;
        assert!((256..512).contains(&leaf), "leaf {leaf}");
        assert_eq!(
            e.to_string(),
            format!(
                "the check was given up at node {}, without a verdict on the proof: the check \
                 takes more than the memory limit of {limit} bytes",
                e.node().get(),
            )
        );
        // The size and the derivation are under the same bound, and their
        // refusal is no invalid proof either.
        assert!(matches!(
            large.derivation_size(
                &crate::proofs::ViewOptions::default(),
                &crate::Limits::default().with_memory_bytes(Some(limit)),
                |_| false
            ),
            Err(CheckError::Refused(_))
        ));
        let view = crate::proofs::ViewOptions::default();
        let limits = crate::Limits::UNBOUNDED.with_memory_bytes(Some(limit));
        let refused = large
            .derivation_within(&view, &limits, |_| false)
            .unwrap_err();
        assert!(matches!(
            refused,
            crate::Error::Check(CheckError::Refused(_))
        ));
        assert!(refused.is_refusal());
    }

    /// Intuitionistic mode accepts the classical terms of intuitionistic
    /// proofs and rejects a sequent with two goals or none: a `⊸L` split
    /// that keeps the goal on the antecedent's side, a weakened goal, Mix.
    #[test]
    fn intuitionistic() {
        use Fault::*;
        use Node::*;
        let m = Mode::INTUITIONISTIC;
        for (input, nodes) in [
            // A, A ⊸ B ⊢ B: 0 ~A, 1 ⊗, 2 A, 3 ~B, 4 B
            (
                "A, A -o B |- B",
                vec![Ax(o(0), o(2)), Ax(o(3), o(4)), Tensor(o(1), n(0), n(1))],
            ),
            // A & B ⊢ A & B: 0 ⊕, 1 ~A, 2 ~B, 3 &, 4 A, 5 B
            (
                "A & B |- A & B",
                vec![
                    Ax(o(1), o(4)),
                    Plus(o(0), Branch::Left, n(0)),
                    Ax(o(2), o(5)),
                    Plus(o(0), Branch::Right, n(2)),
                    With(o(3), n(1), n(3)),
                ],
            ),
            // A ⊸ 0, A ⊢ B: 0 ⊗, 1 A, 2 ⊤, 3 ~A, 4 B: the `0` on the left
            // absorbs the goal.
            (
                "A -o 0, A |- B",
                vec![Ax(o(1), o(3)), Top(o(2)), Tensor(o(0), n(0), n(1))],
            ),
            // 0 ⊢ ⊤ ⊗ ⊤: 0 ⊤, 1 ⊗, 2 ⊤, 3 ⊤; the hypothesis `0` is absorbed
            // by one `⊤R`.
            (
                "0 |- top * top",
                vec![Top(o(2)), Top(o(3)), Tensor(o(1), n(0), n(1))],
            ),
            // !A, !(A ⊸ B) ⊢ !B: 0 ?, 1 ~A, 2 ?, 3 ⊗, 4 A, 5 ~B, 6 !, 7 B
            (
                "!A, !(A -o B) |- !B",
                vec![
                    Ax(o(1), o(4)),
                    Copy(o(1), n(0)),
                    Ax(o(5), o(7)),
                    Tensor(o(3), n(1), n(2)),
                    Copy(o(3), n(3)),
                    Bang(o(6), n(4)),
                    Quest(o(2), n(5)),
                    Quest(o(0), n(6)),
                ],
            ),
        ] {
            let p = proof(input, nodes);
            p.check(m)
                .unwrap_or_else(|e| panic!("{input:?}: {}", e.describe(p.forest())));
        }
        // A, B ⊢ A in affine mode: weakening the hypothesis is fine, the
        // goal never.
        let p = proof("A, B |- A", vec![Ax(o(0), o(2)), Weaken(o(1), n(0))]);
        assert_eq!(p.check(m.with_affine()), Ok(()));
        // A, 0 ⊢ B: 0 ~A, 1 ⊤, 2 B
        let p = proof("A, 0 |- B", vec![Top(o(1)), Weaken(o(2), n(0))]);
        assert_eq!(
            p.check(m.with_affine()).unwrap_err().fault(),
            Some(&Succedents { count: 0 })
        );
        // ((A ⊗ ⊤) & (B ⊗ ⊤)) ⊸ 0 ⊢ (A ⊸ C) ⊕ (B ⊸ C), which classical
        // linear logic proves and intuitionistic linear logic does not:
        // 0 ⊗, 1 &, 2 ⊗, 3 A, 4 ⊤, 5 ⊗, 6 B, 7 ⊤, 8 ⊤, 9 ⊕, 10 ⅋, 11 ~A,
        // 12 C, 13 ⅋, 14 ~B, 15 C. The classical proof splits the `⊸L`
        // with the goal on the antecedent's side, where a `⊤` absorbs it.
        let p = proof(
            "((A * top) & (B * top)) -o 0 |- (A -o C) + (B -o C)",
            vec![
                Ax(o(3), o(11)),
                Top(o(4)),
                Tensor(o(2), n(0), n(1)),
                Par(o(10), n(2)),
                Plus(o(9), Branch::Left, n(3)),
                Ax(o(6), o(14)),
                Top(o(7)),
                Tensor(o(5), n(5), n(6)),
                Par(o(13), n(7)),
                Plus(o(9), Branch::Right, n(8)),
                With(o(1), n(4), n(9)),
                Top(o(8)),
                Tensor(o(0), n(10), n(11)),
            ],
        );
        assert_eq!(p.check(Mode::CLASSICAL), Ok(()));
        let e = p.check(m).unwrap_err();
        assert_eq!(
            (e.node(), e.fault()),
            (n(3), Some(&Succedents { count: 2 }))
        );
        assert_eq!(
            e.describe(p.forest()).to_string(),
            "node 3 (⅋ on ~A ⅋ C from 2) with premises ⊢ A ⊗ ⊤, ~A, …: \
             a sequent of the rule has 2 formulas on the right of ⊢ instead of one"
        );
        // A ⊕ B, (1 ⊗ ⊤) ⊸ X, 0 ⊢ 1 ⊗ ⊤: 0 &, 1 ~A, 2 ~B, 3 ⊗, 4 ⊗, 5 1,
        // 6 ⊤, 7 ~X, 8 ⊤, 9 ⊗, 10 1, 11 ⊤. Both premises of the `&L` absorb
        // with a `⊤`, the left holding the antecedent `1 ⊗ ⊤` and the
        // right the goal, so the `&` joins two succedents, which only the
        // rule on every node's sequent sees: the `⊸L` below takes the
        // antecedent away again, and the root's sequent is the goal's.
        let p = proof(
            "A + B, (1 * top) -o X, 0 |- 1 * top",
            vec![
                One(o(5)),
                Top(o(6)),
                Tensor(o(4), n(0), n(1)),
                One(o(10)),
                Top(o(11)),
                Tensor(o(9), n(3), n(4)),
                With(o(0), n(2), n(5)),
                Top(o(8)),
                Tensor(o(3), n(6), n(7)),
            ],
        );
        assert_eq!(p.check(Mode::CLASSICAL), Ok(()));
        let e = p.check(m).unwrap_err();
        assert_eq!(
            (e.node(), e.fault()),
            (n(6), Some(&Succedents { count: 2 }))
        );
        // Mix is never intuitionistic. 0 ⊗, 1 A, 2 ~B, 3 ~A, 4 B: A ⊸ B, A
        // ⊢ B with a Mix of the axioms.
        let p = proof(
            "A -o B, A |- B",
            vec![Ax(o(1), o(3)), Ax(o(2), o(4)), Mix(n(0), n(1))],
        );
        assert_eq!(p.check(m.with_mix()).unwrap_err().fault(), Some(&Forbidden));
    }

    /// The error names the node, its premises and the problem.
    #[test]
    fn error_message() {
        use Node::*;
        // ⊢ ~A, A ⊗ ~B, B with the ⊗ premises swapped.
        let p = proof(
            "A, A -o B |- B",
            vec![Ax(o(0), o(2)), Ax(o(3), o(4)), Tensor(o(1), n(1), n(0))],
        );
        assert_eq!(
            p.check(Mode::CLASSICAL).unwrap_err().to_string(),
            "node 2 (⊗ on 1 from 1, 0) with premises ⊢ 3, 4 and ⊢ 0, 2: \
             premise 0 lacks occurrence 2"
        );
        assert_eq!(
            p.check(Mode::CLASSICAL)
                .unwrap_err()
                .describe(p.forest())
                .to_string(),
            "node 2 (⊗ on A ⊗ ~B from 1, 0) with premises ⊢ ~B, B and ⊢ ~A, A: \
             premise 0 lacks A"
        );
        // Cut to three characters a formula, as a long report is.
        assert_eq!(
            p.check(Mode::CLASSICAL)
                .unwrap_err()
                .describe(p.forest())
                .abbreviated(Some(3))
                .to_string(),
            "node 2 (⊗ on A ⊗… from 1, 0) with premises ⊢ ~B… (2 formulas) and ⊢ ~A… \
             (2 formulas): premise 0 lacks A"
        );
        // The fault's formula is cut too: an axiom on a formula a proof
        // file can make as long as it likes.
        let p = proof("|- a * (b * c), ~a", vec![Ax(o(0), o(5))]);
        let report = |limit| {
            p.check(Mode::CLASSICAL)
                .unwrap_err()
                .describe(p.forest())
                .abbreviated(limit)
                .to_string()
        };
        assert_eq!(
            report(Some(3)),
            "node 0 (ax on a ⊗…, ~a): a ⊗… is not what the rule acts on"
        );
        assert_eq!(
            report(None),
            "node 0 (ax on a ⊗ (b ⊗ c), ~a): a ⊗ (b ⊗ c) is not what the rule acts on"
        );
        // ⊢ ?~A, A with the ? step missing: a dyadic sequent with Θ.
        let p = proof("!A |- A", vec![Ax(o(1), o(2)), Copy(o(1), n(0))]);
        assert_eq!(
            p.check(Mode::CLASSICAL).unwrap_err().to_string(),
            "node 1 (copy on 1 from 0) with premises ⊢ 1, 2: \
             the proof concludes ⊢ 1 ; 2, not the sequent"
        );
        assert_eq!(
            Dyadic {
                theta: vec![],
                gamma: vec![o(0)],
                any: true
            }
            .to_string(),
            "⊢ 0, …"
        );
    }

    /// A proof keeps only the nodes its root reaches, renumbered in order,
    /// and refuses a node that points outside the arena or the forest, or
    /// at a later node.
    #[test]
    fn construction() {
        use Node::*;
        let s: Sequent = "A, A -o B |- B".parse().unwrap();
        let f = Forest::new(&s).unwrap();
        // Node 1 is unused.
        let p = Proof::new(
            f.clone(),
            vec![
                Ax(o(0), o(2)),
                One(o(4)),
                Ax(o(3), o(4)),
                Tensor(o(1), n(0), n(2)),
            ],
            n(3),
        )
        .unwrap();
        assert_eq!(
            p.nodes(),
            [Ax(o(0), o(2)), Ax(o(3), o(4)), Tensor(o(1), n(0), n(1))]
        );
        assert_eq!(p.root(), n(2));
        assert_eq!(p.check(Mode::CLASSICAL), Ok(()));
        // The root is beyond the arena.
        assert!(matches!(
            Proof::new(f.clone(), vec![Ax(o(0), o(2))], n(1)),
            Err(Error::IndexOutOfBounds {
                space: crate::limits::Space::Node,
                index: 1,
                len: 1
            })
        ));
        // A premise does not precede its conclusion.
        assert!(matches!(
            Proof::new(f.clone(), vec![Par(o(1), n(0))], n(0)),
            Err(Error::NotTopological {
                space: crate::limits::Space::Node,
                index: 0,
                parent: 0
            })
        ));
        // An occurrence beyond the forest.
        assert!(matches!(
            Proof::new(f, vec![Ax(o(0), o(5))], n(0)),
            Err(Error::IndexOutOfBounds {
                space: crate::limits::Space::Occurrence,
                index: 5,
                len: 5
            })
        ));
    }
}
