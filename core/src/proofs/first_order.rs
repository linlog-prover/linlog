// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! First-order proofs written by hand, the checker's verdict on each, and
//! the tests that run the checker, its first implementation, the size
//! estimate, the derivation and the JSON form on them. Each proof is
//! pinned as a file under `core/tests/first-order/`, which the
//! integration tests read.

use super::{Node, NodeId, Proof};
use crate::fragment::Mode;
use crate::occurrences::{Forest, FrameId, Instances, Member, OccId};
use crate::sequents::{Eigen, FoTerm, FoTermId, Sequent};

/// A proof and what the checker says of it in its mode.
pub(crate) struct Case {
    /// The proof's name, which its file is named after.
    pub(crate) name: &'static str,
    /// The proof.
    pub(crate) proof: Proof,
    /// The mode it is checked in.
    pub(crate) mode: Mode,
    /// The fault the checker finds, by its kind in the JSON form, and the
    /// node it finds it at; `None` for a proof.
    pub(crate) fault: Option<(&'static str, u32)>,
}

/// A proof under construction: the sequent's forest and the table of its
/// instances.
struct Build {
    /// The forest.
    forest: Forest,
    /// The instances.
    table: Instances,
}

impl Build {
    /// Starts a proof of `input`, with the cut formulas `cuts`.
    fn new(input: &str, cuts: &[&str]) -> Self {
        let mut sequent: Sequent = input.parse().unwrap();
        for cut in cuts {
            sequent
                .add_cut(format!("|- {cut}").parse().unwrap())
                .unwrap();
        }
        let forest = Forest::new(&sequent).unwrap();
        let table = Instances::new(&forest);
        Self { forest, table }
    }

    /// The root formula `i`, the cut formulas' trees after the roots.
    fn root(&self, i: usize) -> OccId {
        let roots: Vec<OccId> = self
            .forest
            .roots()
            .iter()
            .copied()
            .chain(self.forest.cut_pairs().flat_map(|(a, d)| [a, d]))
            .collect();
        roots[i]
    }

    /// The sequent's term written `text`, a constant or an application.
    fn term(&self, text: &str) -> FoTermId {
        let sequent = self.forest.sequent();
        let written = |t: usize| {
            let mut out = String::new();
            sequent
                .fo
                .write(
                    &mut out,
                    FoTermId::new(t as u32),
                    &mut |out: &mut String, name: &str| {
                        out.push_str(name);
                        Ok(())
                    },
                    &mut |_, _| Ok(()),
                )
                .unwrap();
            out
        };
        (0..sequent.fo_terms().len())
            .find(|&t| matches!(sequent.fo_terms()[t], FoTerm::App(..)) && written(t) == text)
            .map_or_else(|| panic!("no term {text}"), |t| FoTermId::new(t as u32))
    }

    /// The term of eigenvariable `a`.
    fn eigen(&mut self, a: u32) -> FoTermId {
        self.table.eigen(Eigen::new(a))
    }

    /// The frame of `terms`, the outermost binder's first.
    fn frame(&mut self, terms: &[FoTermId]) -> FrameId {
        let mut frame = None;
        for &t in terms {
            frame = Some(self.table.frame(frame, t));
        }
        frame.unwrap()
    }

    /// The member that is `o` under `frame`.
    fn at(&mut self, o: OccId, frame: FrameId) -> Member {
        self.table.member(o, frame)
    }

    /// The proof of the nodes, the root last.
    fn proof(self, nodes: Vec<Node>) -> Proof {
        let root = NodeId::new(nodes.len() as u32 - 1);
        Proof::with_instances(self.forest, self.table, nodes, root).unwrap()
    }
}

/// The member that is the occurrence `o` itself.
fn bare(o: OccId) -> Member {
    Member::from(o)
}

/// The occurrence `k` places after `o`.
fn below(o: OccId, k: u32) -> OccId {
    OccId::new(o.get() + k)
}

/// Returns the proofs and their verdicts.
pub(crate) fn cases() -> Vec<Case> {
    use Node::*;
    let n = NodeId::new;
    let mut cases = vec![];
    let classical = Mode::CLASSICAL;
    let ill = Mode::INTUITIONISTIC;

    // `∀x. p(x) ⊢ p(f(a))`: the witness `f(a)`, a term the proof writes
    // anew and the sequent holds.
    let witness = |wrong: bool| {
        let mut b = Build::new("forall x. p(x) |- p(f(a))", &[]);
        let (ex, goal) = (b.root(0), b.root(1));
        let w = if wrong {
            b.term("a")
        } else {
            let a = b.term("a");
            let f = b
                .forest
                .sequent()
                .symbols()
                .iter()
                .position(|(s, _)| s == "f");
            b.table
                .apply(crate::sequents::Symbol::new(f.unwrap() as u32), &[a])
        };
        let frame = b.frame(&[w]);
        let body = b.at(below(ex, 1), frame);
        b.proof(vec![Ax(body, bare(goal)), Exists(bare(ex), w, n(0))])
    };
    cases.push(Case {
        name: "witness",
        proof: witness(false),
        mode: classical,
        fault: None,
    });
    cases.push(Case {
        name: "witness_ill",
        proof: witness(false).with_mode(ill),
        mode: ill,
        fault: None,
    });
    cases.push(Case {
        name: "wrong_witness",
        proof: witness(true),
        mode: classical,
        fault: Some(("not_dual", 0)),
    });

    // `∃x. ∀y. p(x, y) ⊢ ∀y. ∃x. p(x, y)`: each `∃` step's witness names
    // the eigenvariable of a `∀` step below it, which is the condition
    // met.
    let swap = || {
        let mut b = Build::new(
            "exists x. forall y. p(x, y) |- forall y. exists x. p(x, y)",
            &[],
        );
        // `∀x. ∃y. ~p(x, y)` and `∀y. ∃x. p(x, y)`.
        let (fx, fy) = (b.root(0), b.root(1));
        let (ta, tb) = (b.eigen(0), b.eigen(1));
        let (fa, fb) = (b.frame(&[ta]), b.frame(&[tb]));
        let (fab, fba) = (b.frame(&[ta, tb]), b.frame(&[tb, ta]));
        let ey = b.at(below(fx, 1), fa);
        let ex = b.at(below(fy, 1), fb);
        let left = b.at(below(fx, 2), fab);
        let right = b.at(below(fy, 2), fba);
        b.proof(vec![
            Ax(left, right),
            Exists(ex, ta, n(0)),
            Exists(ey, tb, n(1)),
            Forall(bare(fy), Eigen::new(1), n(2)),
            Forall(bare(fx), Eigen::new(0), n(3)),
        ])
    };
    cases.push(Case {
        name: "swap",
        proof: swap(),
        mode: classical,
        fault: None,
    });
    cases.push(Case {
        name: "swap_ill",
        proof: swap().with_mode(ill),
        mode: ill,
        fault: None,
    });

    // `∀y. ∃x. p(x, y) ⊢ ∃x. ∀y. p(x, y)`, which has no proof: the `∃`
    // steps below the `∀` steps name their eigenvariables, which reach
    // the root.
    {
        let mut b = Build::new(
            "forall y. exists x. p(x, y) |- exists x. forall y. p(x, y)",
            &[],
        );
        // `∃y. ∀x. ~p(x, y)` and `∃x. ∀y. p(x, y)`.
        let (ey, ex) = (b.root(0), b.root(1));
        let (ta, tb) = (b.eigen(0), b.eigen(1));
        let (fa, fb) = (b.frame(&[ta]), b.frame(&[tb]));
        let (fba, fab) = (b.frame(&[tb, ta]), b.frame(&[ta, tb]));
        let fx = b.at(below(ey, 1), fb);
        let fy = b.at(below(ex, 1), fa);
        let left = b.at(below(ey, 2), fba);
        let right = b.at(below(ex, 2), fab);
        cases.push(Case {
            name: "swap_back",
            proof: b.proof(vec![
                Ax(left, right),
                Forall(fy, Eigen::new(1), n(0)),
                Forall(fx, Eigen::new(0), n(1)),
                Exists(bare(ey), tb, n(2)),
                Exists(bare(ex), ta, n(3)),
            ]),
            mode: classical,
            fault: Some(("eigenvariable", 4)),
        });
    }

    // `⊢ ∀x. (~p(x) ⅋ p(x)), q & r`, in affine mode: both premises of the
    // `&` hold a `∀` step on the one member, each with its own
    // eigenvariable over its own subproof; one subproof shared under both
    // with one eigenvariable, which is introduced twice; and one shared
    // under both with two, whose second body the subproof lacks.
    let affine = classical.with_affine();
    for (name, shared, second, fault) in [
        ("apart", false, 1, None),
        ("shared", true, 0, Some(("reintroduced", 4))),
        ("shared_apart", true, 1, Some(("missing", 4))),
    ] {
        let mut b = Build::new("|- forall x. (~p(x) | p(x)), q & r", &[]);
        let (all, with) = (b.root(0), b.root(1));
        let (par, q, r) = (below(all, 1), below(with, 1), below(with, 2));
        let subproof = |b: &mut Build, a: u32| {
            let t = b.eigen(a);
            let frame = b.frame(&[t]);
            let (l, rr, p) = (
                b.at(below(par, 1), frame),
                b.at(below(par, 2), frame),
                b.at(par, frame),
            );
            (Ax(l, rr), p)
        };
        let (ax, p) = subproof(&mut b, 0);
        let mut nodes = vec![
            ax,
            Par(p, n(0)),
            Forall(bare(all), Eigen::new(0), n(1)),
            Weaken(bare(q), n(2)),
        ];
        if shared {
            nodes.push(Forall(bare(all), Eigen::new(second), n(1)));
            nodes.push(Weaken(bare(r), n(4)));
            nodes.push(With(bare(with), n(3), n(5)));
        } else {
            let (ax, p) = subproof(&mut b, second);
            nodes.extend([
                ax,
                Par(p, n(4)),
                Forall(bare(all), Eigen::new(second), n(5)),
                Weaken(bare(r), n(6)),
                With(bare(with), n(3), n(7)),
            ]);
        }
        cases.push(Case {
            name,
            proof: b.proof(nodes),
            mode: affine,
            fault,
        });
    }

    // `∀x. p(x) ⊢ p(a)`: an axiom on the open atom named bare, `∀x. p(x)
    // ⊢ ∃y. p(y)` with a witness no `∀` step introduces, a constant
    // outside the signature, and a `∀` step on the `∃` the hypothesis is
    // one-sided.
    {
        let b = Build::new("forall x. p(x) |- p(a)", &[]);
        let (ex, goal) = (b.root(0), b.root(1));
        cases.push(Case {
            name: "bare",
            proof: b.proof(vec![Ax(bare(below(ex, 1)), bare(goal))]),
            mode: classical,
            fault: Some(("instance", 0)),
        });
        let mut b = Build::new("forall x. p(x) |- exists y. p(y)", &[]);
        let (ex, ey) = (b.root(0), b.root(1));
        let c = b.eigen(2);
        let frame = b.frame(&[c]);
        let (l, r) = (b.at(below(ex, 1), frame), b.at(below(ey, 1), frame));
        // An eigenvariable no `∀` step introduces is a constant outside
        // the signature, a witness of its own.
        cases.push(Case {
            name: "unintroduced",
            proof: b.proof(vec![
                Ax(l, r),
                Exists(bare(ey), c, n(0)),
                Exists(bare(ex), c, n(1)),
            ]),
            mode: classical,
            fault: None,
        });
        let mut b = Build::new("forall x. p(x) |- p(a)", &[]);
        let (ex, goal) = (b.root(0), b.root(1));
        let a = b.term("a");
        let frame = b.frame(&[a]);
        let body = b.at(below(ex, 1), frame);
        cases.push(Case {
            name: "forall_on_exists",
            proof: b.proof(vec![
                Ax(body, bare(goal)),
                Forall(bare(ex), Eigen::new(0), n(0)),
            ]),
            mode: classical,
            fault: Some(("kind", 1)),
        });
    }

    // `∀x. p(x) ⊢ p(a)` with the cut formula `∀x. p(x)`: a cut on the
    // formula, its first premise the identity by a `∀` step and a `∃`
    // step, and a cut on its instance `p(a)`.
    let cut = |mode| {
        let mut b = Build::new("forall x. p(x) |- p(a)", &["forall x. p(x)"]);
        // `∃x. ~p(x)`, `p(a)`, the cut's `∀x. p(x)` and `∃x. ~p(x)`.
        let (ex, goal, all, dual) = (b.root(0), b.root(1), b.root(2), b.root(3));
        let e = b.eigen(0);
        let a = b.term("a");
        let (fe, fa) = (b.frame(&[e]), b.frame(&[a]));
        let (l, r) = (b.at(below(ex, 1), fe), b.at(below(all, 1), fe));
        let d = b.at(below(dual, 1), fa);
        b.proof(vec![
            Ax(l, r),
            Exists(bare(ex), e, n(0)),
            Forall(bare(all), Eigen::new(0), n(1)),
            Ax(d, bare(goal)),
            Exists(bare(dual), a, n(3)),
            Cut(bare(all), n(2), n(4)),
        ])
        .with_mode(mode)
    };
    cases.push(Case {
        name: "cut",
        proof: cut(classical),
        mode: classical,
        fault: None,
    });
    cases.push(Case {
        name: "cut_ill",
        proof: cut(ill),
        mode: ill,
        fault: None,
    });
    {
        let mut b = Build::new("forall x. p(x) |- p(a)", &["forall x. p(x)"]);
        let (ex, goal, all, dual) = (b.root(0), b.root(1), b.root(2), b.root(3));
        let a = b.term("a");
        let fa = b.frame(&[a]);
        let (l, inst) = (b.at(below(ex, 1), fa), b.at(below(all, 1), fa));
        let d = b.at(below(dual, 1), fa);
        cases.push(Case {
            name: "cut_instance",
            proof: b.proof(vec![
                Ax(l, inst),
                Exists(bare(ex), a, n(0)),
                Ax(d, bare(goal)),
                Cut(inst, n(1), n(2)),
            ]),
            mode: classical,
            fault: None,
        });
    }
    // `⊢ ∀x. p(x), ⊤` with the cut formula `∀x. p(x)`: a cut on the
    // instance `p(a)` of an eigenvariable, its second premise a `∀` step
    // introducing `a` beside `~p(a)`, which the cut's frame brings to the
    // root.
    {
        let mut b = Build::new("|- forall x. p(x), top", &["forall x. p(x)"]);
        let (all, top, cut, dual) = (b.root(0), b.root(1), b.root(2), b.root(3));
        let t = b.eigen(0);
        let fa = b.frame(&[t]);
        let (body, inst, d) = (
            b.at(below(all, 1), fa),
            b.at(below(cut, 1), fa),
            b.at(below(dual, 1), fa),
        );
        cases.push(Case {
            name: "cut_frame",
            proof: b.proof(vec![
                Top(bare(top)),
                Ax(body, d),
                Forall(bare(all), Eigen::new(0), n(1)),
                Cut(inst, n(0), n(2)),
            ]),
            mode: classical,
            fault: Some(("eigenvariable", 3)),
        });
    }
    cases
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Limits;
    use crate::proofs::check::{CheckError, Fault};
    use crate::proofs::oracle;
    use crate::proofs::{Sides, ViewOptions};

    /// Returns the kind of a fault as the JSON form writes it.
    fn kind(fault: &Fault) -> &'static str {
        match fault {
            Fault::Instance { .. } => "instance",
            Fault::Eigenvariable { .. } => "eigenvariable",
            Fault::Reintroduced { .. } => "reintroduced",
            Fault::Missing { .. } => "missing",
            Fault::NotDual => "not_dual",
            Fault::Kind { .. } => "kind",
            other => panic!("unexpected {other:?}"),
        }
    }

    /// Every written proof is judged as its case says, and every fault of
    /// the quantifier rules and of the eigenvariable condition is reached.
    #[test]
    fn the_checker_judges_the_written_proofs() {
        let mut reached = std::collections::BTreeSet::new();
        for case in cases() {
            let result = case.proof.check(case.mode);
            match (case.fault, result) {
                (None, Ok(())) => {}
                (Some((expected, node)), Err(error @ CheckError::Invalid(_))) => {
                    let CheckError::Invalid(invalid) = &error else {
                        unreachable!()
                    };
                    assert_eq!(
                        (kind(&invalid.fault), invalid.node.get()),
                        (expected, node),
                        "{}: {}",
                        case.name,
                        error.describe(&case.proof)
                    );
                    reached.insert(expected);
                }
                (expected, result) => panic!("{}: {expected:?} but {result:?}", case.name),
            }
        }
        let all = [
            "eigenvariable",
            "instance",
            "kind",
            "missing",
            "not_dual",
            "reintroduced",
        ];
        assert_eq!(reached.into_iter().collect::<Vec<_>>(), all);
    }

    /// The checker's first implementation, which has the quantifier rules
    /// of its own, gives the same verdict and error on every written proof
    /// in several modes, and on its mutants: every eigenvariable, every
    /// closed term as a witness, and every member of the table in place of
    /// a member a node names.
    #[test]
    fn the_first_implementation_agrees() {
        let modes = [
            Mode::CLASSICAL,
            Mode::CLASSICAL.with_affine().with_mix(),
            Mode::INTUITIONISTIC,
            Mode::INTUITIONISTIC.with_affine(),
            Mode::CYCLIC,
        ];
        let (mut mutants, mut rejected) = (0, 0);
        for case in cases() {
            let proof = &case.proof;
            for mode in modes.into_iter().chain([case.mode]) {
                assert_eq!(
                    proof.check(mode),
                    oracle::check(proof, mode),
                    "{} in {mode} mode",
                    case.name
                );
            }
            let table = proof.instances().unwrap();
            let first = proof.forest().len() as u32;
            let members: Vec<Member> = (0..first + table.len() as u32).map(Member::new).collect();
            let terms = proof.forest().sequent().fo_terms().len() + table.terms().len();
            for i in 0..proof.nodes().len() {
                let mut variants = vec![];
                match proof.nodes()[i] {
                    Node::Forall(m, _, p) => {
                        variants.extend((0..3).map(|a| Node::Forall(m, Eigen::new(a), p)));
                        variants.extend(
                            (0..terms).map(|w| Node::Exists(m, FoTermId::new(w as u32), p)),
                        );
                    }
                    Node::Exists(m, _, p) => {
                        variants.extend(
                            (0..terms).map(|w| Node::Exists(m, FoTermId::new(w as u32), p)),
                        );
                        variants.extend((0..3).map(|a| Node::Forall(m, Eigen::new(a), p)));
                    }
                    _ => {}
                }
                let node = proof.nodes()[i];
                for &m in &members {
                    if let Ok(renamed) = node.renamed(|_| Ok::<_, ()>(m)) {
                        variants.push(renamed);
                    }
                }
                for variant in variants {
                    let mut nodes = proof.nodes().to_vec();
                    nodes[i] = variant;
                    let Ok(mutant) = Proof::with_instances(
                        proof.forest().clone(),
                        table.clone(),
                        nodes,
                        proof.root(),
                    ) else {
                        continue;
                    };
                    mutants += 1;
                    for mode in [case.mode, Mode::CLASSICAL.with_affine()] {
                        let result = mutant.check(mode);
                        rejected += u32::from(result.is_err());
                        assert_eq!(
                            result,
                            oracle::check(&mutant, mode),
                            "{:?} of {} in {mode} mode",
                            mutant.nodes(),
                            case.name
                        );
                    }
                }
            }
        }
        assert!(mutants > 500, "{mutants} mutants");
        assert!(
            rejected > mutants,
            "{rejected} of {mutants} mutants rejected"
        );
    }

    /// The size computed of a proof with instances is that of its
    /// derivation: the inferences and the height exactly, and at least
    /// the characters of every sequent as written.
    #[test]
    fn the_size_bounds_the_derivation() {
        for case in cases().into_iter().filter(|c| c.fault.is_none()) {
            let sides = if case.mode.is_intuitionistic() {
                Sides::Two
            } else {
                Sides::One
            };
            let view = ViewOptions::default().with_sides(sides);
            let size = case
                .proof
                .derivation_size(&view, &Limits::default(), |_| false)
                .unwrap();
            let derivation = case
                .proof
                .derivation_within(&view, &Limits::default(), |_| false)
                .unwrap();
            let inferences = derivation.inferences();
            let characters: u64 = inferences
                .iter()
                .map(|inference| {
                    inference
                        .sequent()
                        .iter()
                        .map(|&m| derivation.formula(m).to_string().chars().count() as u64 + 2)
                        .sum::<u64>()
                })
                .sum();
            assert_eq!(size.inferences, inferences.len() as u64, "{}", case.name);
            assert!(size.characters >= characters, "{}: {size:?}", case.name);
            assert!(!size.exact, "{}", case.name);
        }
    }

    /// The derivation of a proof with instances writes every instance with
    /// its frame's terms, the eigenvariables named after letters no
    /// symbol takes, one-sided and two-sided.
    #[test]
    fn a_derivation_writes_instances() {
        let derivations: Vec<(&str, String)> = cases()
            .into_iter()
            .filter(|c| ["swap", "swap_ill", "witness_ill", "cut"].contains(&c.name))
            .map(|c| {
                let sides = if c.mode.is_intuitionistic() {
                    Sides::Two
                } else {
                    Sides::One
                };
                let view = ViewOptions::default().with_sides(sides);
                let derivation = c
                    .proof
                    .derivation_within(&view, &Limits::default(), |_| false)
                    .unwrap();
                (c.name, derivation.to_string())
            })
            .collect();
        for (name, text) in &derivations {
            println!("{name}:\n{text}\n");
        }
        let text = |name: &str| &derivations.iter().find(|(n, _)| *n == name).unwrap().1;
        assert!(
            text("swap").contains("⊢ ~p(a, b), p(a, b)"),
            "{}",
            text("swap")
        );
        assert!(text("swap").ends_with("⊢ ∀x. (∃y. ~p(x, y)), ∀y. (∃x. p(x, y))"));
        assert!(text("swap_ill").contains("∃L"), "{}", text("swap_ill"));
        assert!(
            text("swap_ill").contains("p(a, b) ⊢ p(a, b)"),
            "{}",
            text("swap_ill")
        );
        assert!(
            text("witness_ill").contains("p(f(a)) ⊢ p(f(a))"),
            "{}",
            text("witness_ill")
        );
        assert!(
            text("witness_ill").contains("∀L"),
            "{}",
            text("witness_ill")
        );
    }

    /// The walk of the witnesses is work the pass counts: a tower of `∃`
    /// steps each of whose witnesses holds the last, `f(t, a)`, is quadratic
    /// in the walk, and a work limit above its nodes and below the walk
    /// refuses it, without a verdict; without one it is a proof.
    #[test]
    fn the_walk_of_witnesses_is_counted_work() {
        let n = 200u32;
        let vars: Vec<String> = (0..n).map(|i| format!("x{i}")).collect();
        let mut b = Build::new(
            &format!("|- exists {}. top, p(f(a, a))", vars.join(" ")),
            &[],
        );
        let outer = b.root(0);
        let f = b
            .forest
            .sequent()
            .symbols()
            .iter()
            .position(|(s, _)| s == "f");
        let f = crate::sequents::Symbol::new(f.unwrap() as u32);
        let a = b.term("a");
        let mut witness = a;
        let mut frame: Option<FrameId> = None;
        let mut members = vec![];
        for i in 0..n {
            let member = match frame {
                None => bare(below(outer, i)),
                Some(frame) => b.at(below(outer, i), frame),
            };
            members.push((member, witness));
            frame = Some(b.table.frame(frame, witness));
            witness = b.table.apply(f, &[witness, a]);
        }
        let top = b.at(below(outer, n), frame.unwrap());
        let mut nodes = vec![Node::Top(top)];
        for (k, &(member, witness)) in members.iter().rev().enumerate() {
            nodes.push(Node::Exists(member, witness, NodeId::new(k as u32)));
        }
        let proof = b.proof(nodes);
        assert_eq!(proof.check(Mode::CLASSICAL), Ok(()));
        let limits = Limits::default().with_work(Some(u64::from(n * n / 4)));
        let result = proof.check_within(Mode::CLASSICAL, &limits, |_| false);
        assert!(
            matches!(
                result,
                Err(CheckError::Refused(crate::proofs::Refused {
                    refusal: crate::Refusal::Work { .. },
                    ..
                }))
            ),
            "{result:?}"
        );
    }

    /// An error writes a frame's term cut after a few thousand characters:
    /// a witness of forty doubly shared levels, `f(t, t)`, which a file of
    /// a kilobyte names and no memory holds written out, is reported in a
    /// short message.
    #[test]
    fn an_error_cuts_a_frame_term() {
        let mut b = Build::new("forall x. p(x) |- p(f(a, a))", &[]);
        let (ex, goal) = (b.root(0), b.root(1));
        let f = b
            .forest
            .sequent()
            .symbols()
            .iter()
            .position(|(s, _)| s == "f");
        let f = crate::sequents::Symbol::new(f.unwrap() as u32);
        let mut t = b.term("a");
        for _ in 0..40 {
            t = b.table.apply(f, &[t, t]);
        }
        let frame = b.frame(&[t]);
        let body = b.at(below(ex, 1), frame);
        let proof = b.proof(vec![
            Node::Ax(body, bare(goal)),
            Node::Exists(bare(ex), t, NodeId::new(0)),
        ]);
        let error = proof.check(Mode::CLASSICAL).unwrap_err();
        let message = error.describe(&proof).to_string();
        assert!(message.contains("…"), "{message}");
        assert!(message.len() < 3 * (1 << 13), "{} bytes", message.len());
    }

    /// Every written proof is pinned as a JSON file under
    /// `core/tests/first-order/`, at wire level 7, and reads back as the
    /// proof it is, which writes the same bytes again.
    #[cfg(feature = "serialize")]
    #[test]
    fn the_proofs_are_pinned_and_read_back() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/first-order");
        for case in cases() {
            let json = serde_json::to_string_pretty(&case.proof).unwrap();
            assert!(json.contains("\"version\": 7"), "{}", case.name);
            let path = dir.join(format!("{}.json", case.name));
            if std::env::var_os("BLESS").is_some() {
                std::fs::create_dir_all(&dir).unwrap();
                std::fs::write(&path, format!("{json}\n")).unwrap();
            }
            let pinned = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("{}: {e}; run with BLESS=1", path.display()));
            assert_eq!(json, pinned.trim_end(), "{}", case.name);
            let read: Proof = serde_json::from_str(&pinned).unwrap();
            assert_eq!(read.nodes(), case.proof.nodes(), "{}", case.name);
            assert_eq!(read.instances(), case.proof.instances(), "{}", case.name);
            // The reader numbers the argument lists in another order than
            // the parser, so the sequents compare by their text.
            assert_eq!(
                read.sequent().to_string(),
                case.proof.sequent().to_string(),
                "{}",
                case.name
            );
            assert_eq!(read.mode(), case.proof.mode(), "{}", case.name);
            assert_eq!(serde_json::to_string_pretty(&read).unwrap(), json);
        }
    }
}
