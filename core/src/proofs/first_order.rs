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

    /// The term `name(args…)` the proof writes anew, `name` a function
    /// symbol of the sequent.
    fn apply(&mut self, name: &str, args: &[FoTermId]) -> FoTermId {
        let symbols = self.forest.sequent().symbols();
        let f = symbols.iter().position(|(s, _)| s == name).unwrap();
        self.table
            .apply(crate::sequents::Symbol::new(f as u32), args)
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
    // `∀x. ∀y. p(x, y) ⊢ (∀y. p(a, y)) & (∀y. p(a, y))` with the cut
    // formula `∀x. ∀y. p(x, y)`, whose dual both branches of the `&` use:
    // the cut moves into both, which then share its first premise, two
    // `∀` steps; the key case of `∀` and `∃` substitutes `a` into a copy
    // of that shared subproof, whose inner `∀` step the other cut still
    // reads.
    let cut_shared = |mode| {
        let mut b = Build::new(
            "forall x. forall y. p(x, y) |- (forall y. p(a, y)) & (forall y. p(a, y))",
            &["forall x. forall y. p(x, y)"],
        );
        let (h, g, c, d) = (b.root(0), b.root(1), b.root(2), b.root(3));
        let a = b.term("a");
        let (e0, e1, e2, e3) = (b.eigen(0), b.eigen(1), b.eigen(2), b.eigen(3));
        let (f0, f01) = (b.frame(&[e0]), b.frame(&[e0, e1]));
        let (fa, fa2, fa3) = (b.frame(&[a]), b.frame(&[a, e2]), b.frame(&[a, e3]));
        let (f2, f3) = (b.frame(&[e2]), b.frame(&[e3]));
        let nodes = vec![
            Ax(b.at(below(h, 2), f01), b.at(below(c, 2), f01)),
            Exists(b.at(below(h, 1), f0), e1, n(0)),
            Exists(bare(h), e0, n(1)),
            Forall(b.at(below(c, 1), f0), Eigen::new(1), n(2)),
            Forall(bare(c), Eigen::new(0), n(3)),
            Ax(b.at(below(d, 2), fa2), b.at(below(g, 2), f2)),
            Exists(b.at(below(d, 1), fa), e2, n(5)),
            Exists(bare(d), a, n(6)),
            Forall(bare(below(g, 1)), Eigen::new(2), n(7)),
            Ax(b.at(below(d, 2), fa3), b.at(below(g, 4), f3)),
            Exists(b.at(below(d, 1), fa), e3, n(9)),
            Exists(bare(d), a, n(10)),
            Forall(bare(below(g, 3)), Eigen::new(3), n(11)),
            With(bare(g), n(8), n(12)),
            Cut(bare(c), n(4), n(13)),
        ];
        b.proof(nodes).with_mode(mode)
    };
    cases.push(Case {
        name: "cut_shared",
        proof: cut_shared(classical),
        mode: classical,
        fault: None,
    });
    cases.push(Case {
        name: "cut_shared_ill",
        proof: cut_shared(ill),
        mode: ill,
        fault: None,
    });
    // `∀x. (p(x) ⊗ q(x)) ⊢ ∀x. (p(x) ⊗ q(x))` with the cut formula
    // `∀x. (p(x) ⊗ q(x))`, cut on its instance `p(a) ⊗ q(a)` of the
    // eigenvariable the goal's `∀` step introduces below the cut.
    let cut_tensor = |mode| {
        let mut b = Build::new(
            "forall x. p(x) * q(x) |- forall x. p(x) * q(x)",
            &["forall x. p(x) * q(x)"],
        );
        let (h, g, c, d) = (b.root(0), b.root(1), b.root(2), b.root(3));
        let e0 = b.eigen(0);
        let f0 = b.frame(&[e0]);
        let [h1, h2, h3, c1, c2, c3, d1, d2, d3, g1, g2, g3] = [
            (h, 1),
            (h, 2),
            (h, 3),
            (c, 1),
            (c, 2),
            (c, 3),
            (d, 1),
            (d, 2),
            (d, 3),
            (g, 1),
            (g, 2),
            (g, 3),
        ]
        .map(|(o, k)| b.at(below(o, k), f0));
        let nodes = vec![
            Ax(h2, c2),
            Ax(h3, c3),
            Tensor(c1, n(0), n(1)),
            Par(h1, n(2)),
            Exists(bare(h), e0, n(3)),
            Ax(d2, g2),
            Ax(d3, g3),
            Tensor(g1, n(5), n(6)),
            Par(d1, n(7)),
            Cut(c1, n(4), n(8)),
            Forall(bare(g), Eigen::new(0), n(9)),
        ];
        b.proof(nodes).with_mode(mode)
    };
    cases.push(Case {
        name: "cut_tensor",
        proof: cut_tensor(classical),
        mode: classical,
        fault: None,
    });
    cases.push(Case {
        name: "cut_tensor_ill",
        proof: cut_tensor(ill),
        mode: ill,
        fault: None,
    });
    // `∀z. p(z), q ⊢ p(f(g(a))) ⊗ q` with the cut formula
    // `∀x. (p(f(g(x))) ⊗ q)`: the `∀` premise's eigenvariable `b` stands
    // in the witness `f(g(b))` of a `∃` step two rules up, which the key
    // case of `∀` and `∃` replaces by `f(g(a))`.
    let cut_deep = |mode| {
        let mut b = Build::new(
            "forall z. p(z), q |- p(f(g(a))) * q",
            &["forall x. p(f(g(x))) * q"],
        );
        let (h, nq, goal, c, d) = (b.root(0), b.root(1), b.root(2), b.root(3), b.root(4));
        let a = b.term("a");
        let e0 = b.eigen(0);
        let ge0 = b.apply("g", &[e0]);
        let w = b.apply("f", &[ge0]);
        let (f0, fw, fa) = (b.frame(&[e0]), b.frame(&[w]), b.frame(&[a]));
        let nodes = vec![
            Ax(b.at(below(h, 1), fw), b.at(below(c, 2), f0)),
            Exists(bare(h), w, n(0)),
            Ax(bare(nq), b.at(below(c, 3), f0)),
            Tensor(b.at(below(c, 1), f0), n(1), n(2)),
            Forall(bare(c), Eigen::new(0), n(3)),
            Ax(b.at(below(d, 2), fa), bare(below(goal, 1))),
            Ax(b.at(below(d, 3), fa), bare(below(goal, 2))),
            Tensor(bare(goal), n(5), n(6)),
            Par(b.at(below(d, 1), fa), n(7)),
            Exists(bare(d), a, n(8)),
            Cut(bare(c), n(4), n(9)),
        ];
        b.proof(nodes).with_mode(mode)
    };
    cases.push(Case {
        name: "cut_deep",
        proof: cut_deep(classical),
        mode: classical,
        fault: None,
    });
    cases.push(Case {
        name: "cut_deep_ill",
        proof: cut_deep(ill),
        mode: ill,
        fault: None,
    });
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

    /// Every written proof with a cut is eliminated by the strategy, step
    /// by step, in its mode and in every mode beside it that takes it
    /// (affine, Mix): every step's proof passes the checker, over the same
    /// sequent with the same conclusion, the last is cut-free, and
    /// `eliminate` takes the same steps; the key case of `∀` and `∃` is
    /// met, and where it substitutes into a copy of a subproof another cut
    /// still reads, the copy's `∀` step has an eigenvariable of its own.
    #[test]
    fn the_cuts_are_eliminated() {
        use crate::proofs::cut::{self, End};
        let (options, limits) = (cut::Options::default(), Limits::default());
        let mut met: std::collections::BTreeMap<&str, usize> = Default::default();
        let mut eliminated = 0;
        for case in cases()
            .into_iter()
            .filter(|c| c.fault.is_none() && !c.proof.is_cut_free())
        {
            let modes = if case.mode.is_intuitionistic() {
                vec![Mode::INTUITIONISTIC, Mode::INTUITIONISTIC.with_affine()]
            } else {
                vec![
                    Mode::CLASSICAL,
                    Mode::CLASSICAL.with_mix(),
                    Mode::CLASSICAL.with_affine(),
                    Mode::CLASSICAL.with_affine().with_mix(),
                ]
            };
            for mode in modes {
                case.proof.check(mode).unwrap();
                let mut current = case.proof.clone();
                let (mut steps, mut renamed) = (Vec::new(), false);
                while !current.is_cut_free() {
                    assert!(steps.len() < 100, "{} in {mode:?}", case.name);
                    let reduction = cut::step(&current, mode, None, &options, &limits, |_| false)
                        .unwrap_or_else(|e| panic!("{} in {mode:?}: {e}", case.name));
                    let made = &reduction.proof;
                    made.check(mode).unwrap_or_else(|e| {
                        panic!("{} in {mode:?}: {}", case.name, e.describe(made))
                    });
                    assert_eq!(made.sequent(), case.proof.sequent(), "{}", case.name);
                    assert_eq!(made.conclusion(), case.proof.conclusion(), "{}", case.name);
                    *met.entry(reduction.step.case.name()).or_default() += 1;
                    renamed |= made
                        .nodes()
                        .iter()
                        .any(|node| matches!(node, Node::Forall(_, a, _) if a.get() > 3));
                    steps.push(reduction.step);
                    current = reduction.proof;
                }
                let elimination =
                    cut::eliminate(&case.proof, mode, &options, &limits, |_| false, |_, _| {})
                        .unwrap();
                assert_eq!(elimination.end, End::CutFree, "{}", case.name);
                assert_eq!(elimination.steps, steps, "{} in {mode:?}", case.name);
                elimination.proof.check(mode).unwrap();
                // The proofs name the eigenvariables 0 to 3.
                assert_eq!(
                    renamed,
                    case.name.starts_with("cut_shared"),
                    "{}",
                    case.name
                );
                eliminated += 1;
            }
        }
        assert_eq!(eliminated, 28);
        for case in [
            "the key case of ∀ and ∃",
            "the key case of ⊗ and ⅋",
            "the axiom case",
            "a commutative case",
        ] {
            assert!(met.contains_key(case), "{case} not met: {met:?}");
        }
    }

    /// `∀x. ∀y. ∀z. p(x, y, z) ⊢ G`, `G` eight `∀y. ∀z. p(a, y, z)` under
    /// three levels of `&`, with the cut formula `∀x. ∀y. ∀z. p(x, y, z)`:
    /// the cut moves into every branch, the eight cuts share their first
    /// premise, and each key case of `∀` and `∃` copies it while the
    /// others still read it.
    fn shared_eight() -> Proof {
        use Node::*;
        let leaf = "(forall y. forall z. p(a, y, z))";
        let mut goal = leaf.to_owned();
        for _ in 0..3 {
            goal = format!("({goal} & {goal})");
        }
        let mut b = Build::new(
            &format!("forall x. forall y. forall z. p(x, y, z) |- {goal}"),
            &["forall x. forall y. forall z. p(x, y, z)"],
        );
        let (h, g, c, d) = (b.root(0), b.root(1), b.root(2), b.root(3));
        let a = b.term("a");
        let mut nodes: Vec<Node> = Vec::new();
        let push = |nodes: &mut Vec<Node>, node| {
            nodes.push(node);
            NodeId::new(nodes.len() as u32 - 1)
        };
        let (e0, e1, e2) = (b.eigen(0), b.eigen(1), b.eigen(2));
        let (f0, f01, f012) = (b.frame(&[e0]), b.frame(&[e0, e1]), b.frame(&[e0, e1, e2]));
        let (hx, cx) = (b.at(below(h, 3), f012), b.at(below(c, 3), f012));
        let mut top = push(&mut nodes, Ax(hx, cx));
        top = push(&mut nodes, Exists(b.at(below(h, 2), f01), e2, top));
        top = push(&mut nodes, Exists(b.at(below(h, 1), f0), e1, top));
        top = push(&mut nodes, Exists(bare(h), e0, top));
        top = push(
            &mut nodes,
            Forall(b.at(below(c, 2), f01), Eigen::new(2), top),
        );
        top = push(
            &mut nodes,
            Forall(b.at(below(c, 1), f0), Eigen::new(1), top),
        );
        let shared = push(&mut nodes, Forall(bare(c), Eigen::new(0), top));
        // The goal's tree: a `&` above two subtrees, or a leaf, each
        // closed by its own two eigenvariables.
        let mut next = 3;
        let mut stack = vec![(g, false)];
        let mut made: Vec<NodeId> = Vec::new();
        while let Some((o, expanded)) = stack.pop() {
            if b.forest.kind(o) == crate::Kind::With {
                if expanded {
                    let right = made.pop().unwrap();
                    let left = made.pop().unwrap();
                    made.push(push(&mut nodes, With(bare(o), left, right)));
                } else {
                    let (l, r) = (b.forest.left(o).unwrap(), b.forest.right(o).unwrap());
                    stack.extend([(o, true), (r, false), (l, false)]);
                }
                continue;
            }
            let (ey, ez) = (b.eigen(next), b.eigen(next + 1));
            let (fy, fyz) = (b.frame(&[ey]), b.frame(&[ey, ez]));
            let (fa, fay, fayz) = (b.frame(&[a]), b.frame(&[a, ey]), b.frame(&[a, ey, ez]));
            let (dx, gx) = (b.at(below(d, 3), fayz), b.at(below(o, 2), fyz));
            let mut top = push(&mut nodes, Ax(dx, gx));
            top = push(&mut nodes, Exists(b.at(below(d, 2), fay), ez, top));
            top = push(&mut nodes, Exists(b.at(below(d, 1), fa), ey, top));
            top = push(&mut nodes, Exists(bare(d), a, top));
            top = push(
                &mut nodes,
                Forall(b.at(below(o, 1), fy), Eigen::new(next + 1), top),
            );
            made.push(push(&mut nodes, Forall(bare(o), Eigen::new(next), top)));
            next += 2;
        }
        let right = made.pop().unwrap();
        push(&mut nodes, Cut(bare(c), shared, right));
        b.proof(nodes)
    }

    /// The copies a step makes of a subproof are charged against the bound
    /// on nodes before they are made: the proof of
    /// [`shared_eight`] grows by copies alone, and under a bound between its
    /// nodes and the most the elimination reaches, the elimination stops
    /// with its last checked proof, cuts and all, and the bound's refusal;
    /// the step it stops at is a key case of `∀` and `∃`.
    #[test]
    fn copies_are_charged_against_the_bounds() {
        use crate::limits::{Bound, Refusal};
        use crate::proofs::cut::{self, End, Strategy};
        let proof = shared_eight();
        let mode = Mode::CLASSICAL;
        proof.check(mode).unwrap();
        let unbounded = cut::Options::default().with_nodes(None);
        let mut largest = proof.nodes().len();
        let elimination =
            cut::eliminate(&proof, mode, &unbounded, &Limits::default(), |_| false, {
                |before, _| largest = largest.max(before.nodes().len())
            })
            .unwrap();
        assert_eq!(elimination.end, End::CutFree);
        assert!(
            largest >= proof.nodes().len() + 40,
            "{largest} of {}",
            proof.nodes().len()
        );
        let bound = (proof.nodes().len() + largest) / 2;
        let at_bound = |options: &cut::Options, limits: &Limits| {
            let ended =
                cut::eliminate(&proof, mode, options, limits, |_| false, |_, _| {}).unwrap();
            ended.proof.check(mode).unwrap();
            assert!(!ended.proof.is_cut_free());
            let cut = Strategy::Topmost.pick(&ended.proof).unwrap();
            let Node::Cut(_, l, r) = ended.proof.node(cut) else {
                unreachable!("a cut")
            };
            assert!(matches!(
                (ended.proof.node(l), ended.proof.node(r)),
                (Forall(..), Exists(..)) | (Exists(..), Forall(..))
            ));
            ended.end
        };
        use Node::{Exists, Forall};
        let options = cut::Options::default().with_nodes(Some(bound as u64));
        assert_eq!(
            at_bound(&options, &Limits::default()),
            End::Refused(Refusal::Elimination {
                what: Bound::Nodes,
                limit: bound as u64
            })
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
