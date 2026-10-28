// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The derivation view: a proof term unfolded into the tree of explicit
//! sequents and rule names of the standard one-sided sequent calculus, with
//! the term's dyadic bookkeeping expanded into dereliction, contraction and
//! weakening. Renderers and exporters read this view, never the term.
//!
//! The translation keeps the derivation small: the standard sequent of a
//! subproof is `⊢ ?Θ, Γ` for the unrestricted zone `Θ` it needs (see
//! [`check`](crate::proofs::check)), not for the zone in force. So a copy is a
//! dereliction, and a contraction as well when the copied formula is used
//! again above; the `?` step is nothing when its formula is used above and
//! a weakening otherwise; a `⊗` or Mix contracts the `?` formulas both
//! premises use, below the rule; and a `&` weakens, above each premise, the
//! `?` formulas only the other premise uses. A `⊤` absorbs whatever context
//! reaches it, so nothing is weakened above it.
//!
//! An intuitionistic derivation is the same tree over the same term, read
//! two-sided: every sequent has one goal, the inferences carry the
//! intuitionistic rule names (`⊸L` for a `⊗` on a hypothesis, `⊗L` for a
//! `⅋` on one, `!L` for a dereliction, and so on), and what a `⊤` absorbs
//! is distributed so that each premise keeps exactly one goal.
//!
//! A classical derivation is drawn two-sided by [`TwoSided`], a view over
//! the one-sided derivation and the record of how its sequent was
//! written: the same inferences with their formulas split by side and
//! their rules named two-sided, and a negation step wherever a negation
//! was written.

mod build;
mod classical;
mod view;

pub use classical::{ClassicalRule, TwoSided};
pub use view::{ClassicalSides, Compact, Derivation, InfId, Inference, Sides, ViewOptions};

#[cfg(all(test, feature = "parse"))]
mod tests {
    use super::build::{Budget, Held, Record};
    use super::view::{Compact, Derivation, InfId, Inference, ViewOptions};
    use crate::Error;
    use crate::Sequent;
    use crate::fragment::Mode;
    use crate::limits::{Limits, Phase, Refusal, Space};
    use crate::occurrences::{Forest, Member, Side};
    use crate::proofs::check::{self, Allowance, Observer};
    use crate::proofs::size;
    use crate::proofs::{Branch, Named, Node, NodeId, Proof, Rule};

    /// Wraps a raw member id.
    const fn o(id: u32) -> Member {
        Member::new(id)
    }

    /// Wraps a raw node id.
    const fn n(id: u32) -> NodeId {
        NodeId::new(id)
    }

    /// Builds a proof of the sequent `input` parses to, with the last node
    /// as the root; the occurrence ids are the preorder numbering.
    fn proof(input: &str, nodes: Vec<Node>) -> Proof {
        let s: Sequent = input.parse().unwrap_or_else(|e| panic!("{input:?}: {e}"));
        let root = n(nodes.len() as u32 - 1);
        Proof::new(Forest::new(&s).unwrap(), nodes, root).unwrap()
    }

    /// Renders the derivation of a proof.
    fn render(input: &str, nodes: Vec<Node>) -> String {
        proof(input, nodes).derivation().unwrap().to_string()
    }

    /// Multiplicative rules and their units draw as a tree with the rule
    /// on the bar and the conclusion centred under it.
    #[test]
    fn mll_with_units() {
        use Node::*;
        // ⊢ (1 ⊗ A) ⅋ ~A, ⊥: 0 ⅋, 1 ⊗, 2 1, 3 A, 4 ~A, 5 ⊥
        assert_eq!(
            render(
                "|- (1 * A) par ~A, bot",
                vec![
                    One(o(2)),
                    Ax(o(3), o(4)),
                    Tensor(o(1), n(0), n(1)),
                    Par(o(0), n(2)),
                    Bot(o(5), n(3)),
                ]
            ),
            [
                "─── 1   ─────── ax",
                "⊢ 1     ⊢ A, ~A",
                "─────────────── ⊗",
                "  ⊢ 1 ⊗ A, ~A",
                " ────────────── ⅋",
                " ⊢ (1 ⊗ A) ⅋ ~A",
                "───────────────── ⊥",
                "⊢ (1 ⊗ A) ⅋ ~A, ⊥",
            ]
            .join("\n")
        );
    }

    /// Additive rules: `&` copies the context, `⊕` names its side, and a `⊤`
    /// shows the context it absorbs, also through a `⊗` split.
    #[test]
    fn mall() {
        use Node::*;
        // ⊢ A & B, ~A ⊕ ~B: 0 &, 1 A, 2 B, 3 ⊕, 4 ~A, 5 ~B
        assert_eq!(
            render(
                "|- A & B, ~A + ~B",
                vec![
                    Ax(o(1), o(4)),
                    Plus(o(3), Branch::Left, n(0)),
                    Ax(o(2), o(5)),
                    Plus(o(3), Branch::Right, n(2)),
                    With(o(0), n(1), n(3)),
                ]
            ),
            [
                "  ─────── ax        ─────── ax",
                "  ⊢ A, ~A           ⊢ B, ~B",
                "──────────── ⊕₁   ──────────── ⊕₂",
                "⊢ A, ~A ⊕ ~B      ⊢ B, ~A ⊕ ~B",
                "────────────────────────────── &",
                "       ⊢ A & B, ~A ⊕ ~B",
            ]
            .join("\n")
        );
        // ⊢ ⊤ ⊗ A, ~A, B: 0 ⊗, 1 ⊤, 2 A, 3 ~A, 4 B
        assert_eq!(
            render(
                "|- top * A, ~A, B",
                vec![Top(o(1)), Ax(o(2), o(3)), Tensor(o(0), n(0), n(1))]
            ),
            [
                "────── ⊤   ─────── ax",
                "⊢ ⊤, B     ⊢ A, ~A",
                "────────────────── ⊗",
                "  ⊢ ⊤ ⊗ A, ~A, B",
            ]
            .join("\n")
        );
    }

    /// Exponentials: a copy is a dereliction, a second copy of the same
    /// formula adds a contraction below the `⊗` that joins them, an unused
    /// `?` formula is weakened, a `&` premise that does not use one weakens
    /// it above, and promotion keeps the `?` context.
    #[test]
    fn mell() {
        use Node::*;
        // ⊢ ?~A, A ⊗ A: 0 ?, 1 ~A, 2 ⊗, 3 A, 4 A
        assert_eq!(
            render(
                "!A |- A * A",
                vec![
                    Ax(o(1), o(3)),
                    Copy(o(1), n(0)),
                    Ax(o(1), o(4)),
                    Copy(o(1), n(2)),
                    Tensor(o(2), n(1), n(3)),
                    Quest(o(0), n(4)),
                ]
            ),
            [
                "─────── ax    ─────── ax",
                "⊢ ~A, A       ⊢ ~A, A",
                "──────── ?d   ──────── ?d",
                "⊢ ?~A, A      ⊢ ?~A, A",
                "────────────────────── ⊗",
                "  ⊢ ?~A, ?~A, A ⊗ A",
                "  ───────────────── ?c",
                "    ⊢ ?~A, A ⊗ A",
            ]
            .join("\n")
        );
        // ⊢ ?~A, A & 1: 0 ?, 1 ~A, 2 &, 3 A, 4 1
        assert_eq!(
            render(
                "!A |- A & 1",
                vec![
                    Ax(o(1), o(3)),
                    Copy(o(1), n(0)),
                    One(o(4)),
                    With(o(2), n(1), n(2)),
                    Quest(o(0), n(3)),
                ]
            ),
            [
                "─────── ax      ─── 1",
                "⊢ ~A, A         ⊢ 1",
                "──────── ?d   ──────── ?w",
                "⊢ ?~A, A      ⊢ ?~A, 1",
                "────────────────────── &",
                "     ⊢ ?~A, A & 1",
            ]
            .join("\n")
        );
        // ⊢ ?~A, ?(A ⊗ ~B), !B: 0 ?, 1 ~A, 2 ?, 3 ⊗, 4 A, 5 ~B, 6 !, 7 B
        assert_eq!(
            render(
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
                ]
            ),
            [
                "─────── ax",
                "⊢ ~A, A",
                "──────── ?d   ─────── ax",
                "⊢ ?~A, A      ⊢ ~B, B",
                "───────────────────── ⊗",
                "  ⊢ ?~A, A ⊗ ~B, B",
                " ─────────────────── ?d",
                " ⊢ ?~A, ?(A ⊗ ~B), B",
                " ──────────────────── !",
                " ⊢ ?~A, ?(A ⊗ ~B), !B",
            ]
            .join("\n")
        );
    }

    /// Affine weakening and Mix have their own rule names.
    #[test]
    fn affine_and_mix() {
        use Node::*;
        // ⊢ ~A, ~B, A
        assert_eq!(
            render("A, B |- A", vec![Ax(o(0), o(2)), Weaken(o(1), n(0))]),
            ["  ─────── ax", "  ⊢ ~A, A", "─────────── wk", "⊢ ~A, ~B, A",].join("\n")
        );
        // ⊢ 1, ?A: weakening a ? formula is ?w.
        assert_eq!(
            render("|- 1, ?A", vec![One(o(0)), Weaken(o(1), n(0))]),
            ["  ─── 1", "  ⊢ 1", "─────── ?w", "⊢ 1, ?A"].join("\n")
        );
        // ⊢ ~A ⅋ ~B, A ⅋ B: 0 ⅋, 1 ~A, 2 ~B, 3 ⅋, 4 A, 5 B
        assert_eq!(
            render(
                "A * B |- A par B",
                vec![
                    Ax(o(1), o(4)),
                    Ax(o(2), o(5)),
                    Mix(n(0), n(1)),
                    Par(o(0), n(2)),
                    Par(o(3), n(3)),
                ]
            ),
            [
                "─────── ax   ─────── ax",
                "⊢ ~A, A      ⊢ ~B, B",
                "──────────────────── mix",
                "   ⊢ ~A, ~B, A, B",
                "   ─────────────── ⅋",
                "   ⊢ ~A ⅋ ~B, A, B",
                "   ──────────────── ⅋",
                "   ⊢ ~A ⅋ ~B, A ⅋ B",
            ]
            .join("\n")
        );
    }

    /// A two-sided derivation shows `Γ ⊢ A` at every inference with the
    /// intuitionistic rule names, gives the `0` premise of a `⊸L` the goal
    /// its `⊤` absorbs, and refuses a proof that is not intuitionistic.
    #[test]
    fn two_sided() {
        use Node::*;
        let render = |input: &str, nodes| proof(input, nodes).two_sided().unwrap().to_string();
        // A, A ⊸ B ⊢ B: 0 ~A, 1 ⊗, 2 A, 3 ~B, 4 B
        assert_eq!(
            render(
                "A, A -o B |- B",
                vec![Ax(o(0), o(2)), Ax(o(3), o(4)), Tensor(o(1), n(0), n(1))]
            ),
            [
                "───── ax   ───── ax",
                "A ⊢ A      B ⊢ B",
                "──────────────── ⊸L",
                "  A, A ⊸ B ⊢ B",
            ]
            .join("\n")
        );
        // A & B ⊢ (A ⊕ 1) & B: 0 ⊕, 1 ~A, 2 ~B, 3 &, 4 ⊕, 5 A, 6 1, 7 B
        assert_eq!(
            render(
                "A & B |- (A + 1) & B",
                vec![
                    Ax(o(1), o(5)),
                    Plus(o(4), Branch::Left, n(0)),
                    Plus(o(0), Branch::Left, n(1)),
                    Ax(o(2), o(7)),
                    Plus(o(0), Branch::Right, n(3)),
                    With(o(3), n(2), n(4)),
                ]
            ),
            [
                "    ───── ax",
                "    A ⊢ A",
                "  ───────── ⊕R₁       ───── ax",
                "  A ⊢ A ⊕ 1           B ⊢ B",
                "───────────── &L₁   ───────── &L₂",
                "A & B ⊢ A ⊕ 1       A & B ⊢ B",
                "───────────────────────────── &R",
                "     A & B ⊢ (A ⊕ 1) & B",
            ]
            .join("\n")
        );
        // A ⊸ 0, A ⊢ B ⊗ ⊤: 0 ⊗, 1 A, 2 ⊤, 3 ~A, 4 ⊗, 5 B, 6 ⊤: the goal
        // `B ⊗ ⊤` is split first, then the `0` absorbs `B`.
        assert_eq!(
            render(
                "A -o 0, A |- B * top",
                vec![
                    Ax(o(1), o(3)),
                    Top(o(2)),
                    Tensor(o(0), n(0), n(1)),
                    Top(o(6)),
                    Tensor(o(4), n(2), n(3)),
                ]
            ),
            [
                "───── ax   ───── 0L",
                "A ⊢ A      0 ⊢ B",
                "──────────────── ⊸L   ─── ⊤R",
                "  A ⊸ 0, A ⊢ B        ⊢ ⊤",
                "  ─────────────────────── ⊗R",
                "     A ⊸ 0, A ⊢ B ⊗ ⊤",
            ]
            .join("\n")
        );
        // !A, !(A ⊸ B) ⊢ !B & 1: 0 ?, 1 ~A, 2 ?, 3 ⊗, 4 A, 5 ~B, 6 &, 7 !,
        // 8 B, 9 1
        assert_eq!(
            render(
                "!A, !(A -o B) |- !B & 1",
                vec![
                    Ax(o(1), o(4)),
                    Copy(o(1), n(0)),
                    Ax(o(5), o(8)),
                    Tensor(o(3), n(1), n(2)),
                    Copy(o(3), n(3)),
                    Bang(o(7), n(4)),
                    One(o(9)),
                    With(o(6), n(5), n(6)),
                    Quest(o(2), n(7)),
                    Quest(o(0), n(8)),
                ]
            ),
            [
                "───── ax",
                "A ⊢ A",
                "────── !L   ───── ax",
                "!A ⊢ A      B ⊢ B",
                "───────────────── ⊸L          ─── 1R",
                "  !A, A ⊸ B ⊢ B               ⊢ 1",
                " ──────────────── !L         ────── !w",
                " !A, !(A ⊸ B) ⊢ B            !A ⊢ 1",
                " ───────────────── !R   ──────────────── !w",
                " !A, !(A ⊸ B) ⊢ !B      !A, !(A ⊸ B) ⊢ 1",
                " ─────────────────────────────────────── &R",
                "          !A, !(A ⊸ B) ⊢ !B & 1",
            ]
            .join("\n")
        );
        // Affine weakening keeps its name. A classical proof that is not
        // intuitionistic has no two-sided derivation, and a classical
        // sequent none at all.
        let p = proof("A, B |- A", vec![Ax(o(0), o(2)), Weaken(o(1), n(0))]);
        assert_eq!(
            p.two_sided().unwrap().to_string(),
            [" ───── ax", " A ⊢ A", "──────── wk", "A, B ⊢ A"].join("\n")
        );
        let p = proof("|- A par B", vec![Ax(o(1), o(2))]);
        assert!(p.two_sided().is_err());
        let p = proof("A, B |- A", vec![Ax(o(0), o(2)), Weaken(o(2), n(0))]);
        assert!(p.two_sided().is_err());
    }

    /// A compact view draws a run of one structural rule as one starred
    /// inference: always when asked, and by default only where the whole
    /// derivation passes a bound that the compact one does not.
    #[test]
    fn compact_runs() {
        let s: Sequent = "|- ?a, ?b, ?c, ?d, 1".parse().unwrap();
        let options = crate::Options::default();
        let outcome = crate::prove(&s, Mode::default(), &options).unwrap();
        let crate::Verdict::Proved(p) = outcome.verdict else {
            panic!("provable");
        };
        let view = |compact| ViewOptions::default().with_compact(compact);
        let limit = |bytes| crate::Limits::default().with_derivation_bytes(bytes);
        let full = p.derivation_within(&view(Compact::Never), &limit(None), |_| false);
        assert_eq!(full.unwrap().inferences().len(), 5);
        let compact = p.derivation_within(&view(Compact::Always), &limit(None), |_| false);
        let compact = compact.unwrap();
        assert_eq!(compact.inferences().len(), 2);
        assert_eq!(compact.inference(compact.root()).times, 4);
        assert_eq!(
            compact.to_string(),
            "        ─── 1\n        ⊢ 1\n─────────────────── ?w*\n⊢ ?a, ?b, ?c, ?d, 1"
        );
        let bytes = p.size_of(false).unwrap().bytes();
        let tight = Some(bytes - 1);
        let auto = p.derivation_within(&view(Compact::Auto), &limit(tight), |_| false);
        assert_eq!(auto.unwrap().inferences().len(), 2, "over the bound");
        let auto = p.derivation_within(&view(Compact::Auto), &limit(Some(bytes)), |_| false);
        assert_eq!(auto.unwrap().inferences().len(), 5, "within the bound");
        let never = p.derivation_within(&view(Compact::Never), &limit(tight), |_| false);
        assert!(matches!(
            never,
            Err(Error::Refused(Refusal::Output {
                least_bytes: None,
                ..
            }))
        ));
        let none = p.derivation_within(&view(Compact::Auto), &limit(Some(10)), |_| false);
        // The compact view was considered, and its least size says so.
        assert!(
            matches!(
                none,
                Err(Error::Refused(Refusal::Output {
                    least_bytes: Some(_),
                    ..
                }))
            ),
            "both over"
        );
    }

    /// The compact view is the whole derivation with every run of one
    /// structural rule drawn as its lowest inference, standing for the run
    /// and resting on the premises of its highest, on the proofs of the
    /// checker's differential test, one-sided and two-sided.
    #[test]
    fn compact_is_the_whole_with_its_runs_merged() {
        /// An inference as the test compares it: its sequent, rule,
        /// principal, times and number of premises.
        type Shape = (Vec<Member>, Named, Option<usize>, u32, usize);
        /// Returns the tree of a derivation in preorder; with `merge`, a
        /// run of one structural rule as its lowest inference.
        fn shape(d: &Derivation, merge: bool) -> Vec<Shape> {
            let (mut out, mut stack) = (vec![], vec![d.root()]);
            while let Some(id) = stack.pop() {
                let low = d.inference(id);
                let (mut high, mut times) = (low, low.times);
                while merge && low.rule.is_structural() && high.premises.len() == 1 {
                    let above = d.inference(high.premises[0]);
                    if above.rule != low.rule {
                        break;
                    }
                    (high, times) = (above, times + above.times);
                }
                let premises = high.premises.len();
                out.push((
                    low.sequent.clone(),
                    low.rule,
                    low.principal,
                    times,
                    premises,
                ));
                stack.extend(high.premises.iter().rev().copied());
            }
            out
        }
        let mut proofs = crate::proofs::oracle::proofs();
        // Two `?` formulas that both premises of a `⊗` use, which no sample
        // has: a run of contractions below it.
        let sequent: Sequent = "!a, !b |- (a * b) * (a * b)".parse().unwrap();
        for (mode, engine) in [
            (Mode::CLASSICAL, crate::search::Engine::Focus),
            (Mode::INTUITIONISTIC, crate::search::Engine::TwoSided),
        ] {
            // The focused engines, whose proof copies both `?` formulas on
            // both premises; the dispatch's Horn engine copies each where
            // it fires.
            let options = crate::Options::default().with_engine(Some(engine));
            let outcome = crate::prove(&sequent, mode, &options).unwrap();
            let crate::Verdict::Proved(proof) = outcome.verdict else {
                panic!("provable");
            };
            proofs.push((*proof, mode));
        }
        let (mut runs, mut exact) = (vec![], 0);
        for (proof, mode) in proofs {
            let sides: &[bool] = if mode.intuitionistic {
                &[false, true]
            } else {
                &[false]
            };
            for &two_sided in sides {
                let build = |compact| {
                    let view = ViewOptions::default().with_compact(compact);
                    let limits = &crate::Limits::default().with_derivation_bytes(None);
                    let built = if two_sided {
                        Derivation::two_sided(&proof, &view, limits, |_| false)
                    } else {
                        Derivation::new(&proof, &view, limits, |_| false)
                    };
                    built.unwrap()
                };
                let (whole, compact) = (build(Compact::Never), build(Compact::Always));
                let context = format!("{} ({mode}, two-sided: {two_sided})", proof.sequent());
                assert_eq!(shape(&compact, false), shape(&whole, true), "{context}");
                // What the size pass says a compact view holds at least,
                // which decides whether `Auto` tries one, is no more than
                // the compact view built holds, by the builder's own count.
                let (view_mode, reading) = if two_sided {
                    let view_mode = Derivation::two_sided_mode(&proof);
                    (view_mode, check::reading(&proof, view_mode).unwrap())
                } else {
                    (Derivation::ONE_SIDED, None)
                };
                let mut stop = |_| false;
                let limits = crate::Limits::default();
                let allowance = Allowance::new(&limits, Phase::View, &mut stop);
                let conclusion = proof.conclusion();
                let (_, firm) =
                    size::measured(&proof, &conclusion, view_mode, reading.as_ref(), allowance)
                        .unwrap();
                let held = Held {
                    weights: size::weights(proof.forest()),
                    bytes: 0,
                    budget: Budget {
                        most: None,
                        error: Error::Refused(Refusal::Stopped { phase: Phase::View }),
                    },
                };
                let bytes = compact
                    .inferences()
                    .iter()
                    .map(|i| held.cost(&i.sequent))
                    .sum();
                let inferences = compact.inferences().len() as u64;
                assert!(firm.inferences <= inferences, "{context}");
                assert!(firm.bytes <= bytes, "{context}");
                // Without a structural rule and a `⊤`, whose context the
                // bound leaves out, the bound is the view.
                let plain = |i: &Inference| !i.rule.is_structural() && i.rule.rule != Rule::Top;
                if whole.inferences().iter().all(plain) {
                    assert_eq!(
                        (firm.inferences, firm.bytes),
                        (inferences, bytes),
                        "{context}"
                    );
                    exact += 1;
                }
                runs.extend(
                    compact
                        .inferences()
                        .iter()
                        .filter(|i| i.times > 1)
                        .map(|i| i.rule),
                );
            }
        }
        for rule in [
            Named::from(Rule::Weakening),
            Named::from(Rule::Contraction),
            Rule::Contraction.on(Side::Input),
        ] {
            assert!(runs.contains(&rule), "no run of {rule}");
        }
        assert!(exact > 0, "no sample without a structural rule");
    }

    /// Every inference of a proof's derivation names the node whose rule
    /// it shows, on the proofs of the checker's differential test (cuts
    /// among them), one-sided and two-sided: a cut inference its cut
    /// node, on that node's formula, every node but a `?` step shown by
    /// one inference at least, and an inference without a node a
    /// structural rule the derivation adds.
    #[test]
    fn every_inference_names_its_node() {
        let mut cuts = 0;
        let limits = crate::Limits::default().with_derivation_bytes(None);
        let view = ViewOptions::default().with_compact(Compact::Never);
        for (proof, mode) in crate::proofs::oracle::proofs() {
            for two_sided in [false, true] {
                let derivation = match two_sided {
                    false => Derivation::new(&proof, &view, &limits, |_| false),
                    true if mode.intuitionistic => {
                        Derivation::two_sided(&proof, &view, &limits, |_| false)
                    }
                    true => continue,
                }
                .unwrap();
                let context = format!("{} ({mode}, two-sided: {two_sided})", proof.sequent());
                let mut shown = vec![false; proof.nodes().len()];
                for inference in derivation.inferences() {
                    let Some(node) = inference.node() else {
                        assert!(inference.rule().is_structural(), "{context}");
                        continue;
                    };
                    shown[node.index()] = true;
                    match proof.node(node) {
                        Node::Cut(a, ..) => {
                            assert_eq!(inference.cut(), Some(a), "{context}");
                            cuts += 1;
                        }
                        _ => assert_eq!(inference.cut(), None, "{context}"),
                    }
                }
                for (id, node) in proof.nodes().iter().enumerate() {
                    assert!(
                        shown[id] || matches!(node, Node::Quest(..)),
                        "{context}: node {id} is not shown"
                    );
                }
            }
        }
        assert!(cuts > 100, "{cuts} cut inferences");
    }

    /// A chain of affine weakenings is one run of a compact view, planned
    /// in one step: the builder asks its stop once per step, so a chain
    /// walked node by node, with a sequent per node, asked it per node.
    #[test]
    fn a_chain_of_weakenings_is_one_step() {
        use Node::*;
        const LENGTH: u32 = 1_000;
        let literals: Vec<String> = (0..LENGTH).map(|i| format!("b{i}")).collect();
        let sequent: Sequent = format!("|- a, ~a, {}", literals.join(", "))
            .parse()
            .unwrap();
        let mut nodes = vec![Ax(o(0), o(1))];
        for i in 0..LENGTH {
            nodes.push(Weaken(o(2 + i), n(i)));
        }
        let proof = Proof::new(crate::Forest::new(&sequent).unwrap(), nodes, n(LENGTH)).unwrap();
        let view = ViewOptions::default().with_compact(Compact::Always);
        let mut polls = 0;
        let compact = Derivation::new(&proof, &view, &crate::Limits::default(), |p| {
            polls += u64::from(p.phase == Phase::View);
            false
        })
        .unwrap();
        let run = compact.inference(compact.root());
        assert_eq!(
            (run.rule, run.times),
            (Named::from(Rule::AffineWeakening), LENGTH)
        );
        // The checker's passes ask it too, by their work: some fifteen.
        assert!(polls < 100, "{polls} polls");
    }

    /// A derivation of any height is built on a small stack: 120 000
    /// inferences, one above the other, on a thread with 256 KiB.
    #[test]
    fn any_height_on_a_small_stack() {
        use crate::sequents::{Term, TermId};
        use Node::*;
        const ROUNDS: u32 = 40_000;
        // ⊢ ?⊥, 1: 0 ?, 1 ⊥, 2 1. Every round puts the ⊥ under the ? into
        // the linear zone and copies it out again: a ⊥ rule, a
        // dereliction and, but for the topmost, a contraction.
        let sequent = Sequent {
            terms: vec![Term::Bot, Term::Quest(TermId::new(0)), Term::One],
            roots: vec![TermId::new(1), TermId::new(2)],
            atoms: vec![],
            antecedents: None,
            cuts: vec![],
        };
        let mut nodes = vec![One(o(2))];
        for _ in 0..ROUNDS {
            nodes.push(Bot(o(1), n(nodes.len() as u32 - 1)));
            nodes.push(Copy(o(1), n(nodes.len() as u32 - 1)));
        }
        nodes.push(Quest(o(0), n(nodes.len() as u32 - 1)));
        let root = n(nodes.len() as u32 - 1);
        let build = move || {
            let p = Proof::new(Forest::new(&sequent).unwrap(), nodes, root).unwrap();
            let d = p.derivation().unwrap();
            let inferences = d.inferences();
            assert_eq!(inferences.len(), 3 * ROUNDS as usize);
            // One branch: every inference is over the one before it.
            for (i, inference) in inferences.iter().enumerate().skip(1) {
                assert_eq!(inference.premises, [InfId::new(i as u32 - 1)]);
            }
            let size = p.size_of(false).unwrap();
            assert_eq!(size.height, u64::from(3 * ROUNDS));
            assert_eq!(inferences[0].rule, Named::from(Rule::One));
            assert_eq!(d.inference(d.root()).rule, Named::from(Rule::Contraction));
        };
        std::thread::Builder::new()
            .stack_size(256 << 10)
            .spawn(build)
            .unwrap()
            .join()
            .unwrap();
    }

    /// A proof of `⊢ 1, ⊥ & ⊥, …` with `levels` formulas `⊥ & ⊥`, whose
    /// two premises at every level are one subproof: 3 · `levels` + 1
    /// nodes that unfold into more than 2^`levels` inferences.
    fn tower(levels: u32) -> Proof {
        use crate::sequents::{Term, TermId};
        use Node::*;
        // Occurrences: 0 is 1, then &, ⊥, ⊥ for every level.
        let both = Term::With(TermId::new(1), TermId::new(1));
        let mut roots = vec![TermId::new(0)];
        roots.extend((0..levels).map(|_| TermId::new(2)));
        let sequent = Sequent {
            terms: vec![Term::One, Term::Bot, both],
            roots,
            atoms: vec![],
            antecedents: None,
            cuts: vec![],
        };
        let mut nodes = vec![One(o(0))];
        for level in 0..levels {
            let (with, below) = (1 + 3 * level, n(nodes.len() as u32 - 1));
            nodes.push(Bot(o(with + 1), below));
            nodes.push(Bot(o(with + 2), below));
            let last = nodes.len() as u32;
            nodes.push(With(o(with), n(last - 2), n(last - 1)));
        }
        let root = n(nodes.len() as u32 - 1);
        Proof::new(Forest::new(&sequent).unwrap(), nodes, root).unwrap()
    }

    /// A derivation past a bound is not built, and the error says which
    /// bound: the size limit, the memory limit when that is the one it
    /// passes, or the number of inferences a derivation holds when both
    /// are lifted. A size that no `u64` holds saturates.
    #[test]
    fn refuses_a_derivation_past_its_bounds() {
        let never = || false;
        // 2²⁷ − 3 inferences, each with a sequent of up to 26 formulas.
        let p = tower(25);
        let size = p.size_of(false).unwrap();
        assert_eq!(size.inferences, (1 << 27) - 3);
        assert_eq!(size.height, 51);
        assert!(size.bytes() > Limits::DEFAULT_MEMORY_BYTES);
        let limit = crate::Limits::DEFAULT_DERIVATION_BYTES;
        let too_large = p.derivation().unwrap_err();
        let Error::Refused(Refusal::Output {
            estimate_bytes,
            limit_bytes,
            ..
        }) = too_large
        else {
            panic!("{too_large}")
        };
        assert_eq!((estimate_bytes, limit_bytes), (size.bytes(), limit));
        assert_eq!(too_large.setting(), Some("limits.derivation_bytes"));
        let memory = p
            .derivation_within(
                &ViewOptions::default(),
                &crate::Limits::default().with_derivation_bytes(None),
                |_| never(),
            )
            .unwrap_err();
        assert_eq!(
            memory,
            Error::Refused(Refusal::Memory {
                phase: Phase::View,
                limit_bytes: Limits::DEFAULT_MEMORY_BYTES,
                needed_bytes: Some(size.bytes())
            })
        );
        // Options read without a field have its default, and a field
        // they do not have is refused.
        #[cfg(feature = "serialize")]
        {
            let view = serde_json::from_str::<ViewOptions>(r#"{}"#).unwrap();
            assert_eq!(view, ViewOptions::default());
            assert!(serde_json::from_str::<ViewOptions>(r#"{"limit":null}"#).is_err());
        }

        // More than 2⁷⁰ inferences.
        let p = tower(70);
        let size = p.size_of(false).unwrap();
        assert_eq!((size.inferences, size.characters), (u64::MAX, u64::MAX));
        assert_eq!((size.bytes(), size.height), (u64::MAX, 141));
        assert!(!size.exact, "a saturated count is no exact one");
        let unbounded = crate::Limits::UNBOUNDED;
        let too_many = p
            .derivation_within(&ViewOptions::default(), &unbounded, |_| never())
            .unwrap_err();
        assert_eq!(
            too_many,
            Error::Refused(Refusal::Index {
                what: Space::Inference,
                count: u64::MAX,
                most: Derivation::MOST
            })
        );
    }

    /// What the record says it holds covers what it keeps: the sequents of
    /// a `⊗` and of its premises, and the occurrences both premises use.
    #[test]
    fn record_counts_what_it_keeps() {
        use Node::*;
        // ⊢ ?~A, A ⊗ A: 0 ?, 1 ~A, 2 ⊗, 3 A, 4 A
        let p = proof(
            "!A |- A * A",
            vec![
                Ax(o(1), o(3)),
                Copy(o(1), n(0)),
                Ax(o(1), o(4)),
                Copy(o(1), n(2)),
                Tensor(o(2), n(1), n(3)),
                Quest(o(0), n(4)),
            ],
        );
        let mut record = Record::new(&p);
        let roots = p.forest().roots();
        let mut never = |_| false;
        let allowance = check::Allowance::new(&crate::Limits::UNBOUNDED, Phase::View, &mut never);
        check::examine(
            &p,
            roots,
            Derivation::ONE_SIDED,
            None,
            allowance,
            &mut record,
        )
        .unwrap();
        // ⊢ ?~A, A twice and ⊢ ?~A, A ⊗ A, and the one shared ~A.
        let sequents: Vec<usize> = [1, 3, 4]
            .map(|id| record.standard[&n(id)].as_slice().len())
            .into();
        assert_eq!((sequents, record.standard.len()), (vec![2, 2, 2], 3));
        assert_eq!(record.shared[&n(4)], [o(1).occ()]);
        // Three flags for each of six nodes, four lists of seven members.
        assert_eq!(record.bytes(), 6 * 3 + 4 * Record::ENTRY + 7 * 8);
    }

    /// The inferences carry the sequents as ids with repeats, the rule, the
    /// principal formula's position, the premises, premises first, and
    /// the node each shows (none for the contraction the derivation adds
    /// below the `⊗`).
    #[test]
    fn inferences() {
        use Node::*;
        // ⊢ ?~A, A ⊗ A: 0 ?, 1 ~A, 2 ⊗, 3 A, 4 A
        let p = proof(
            "!A |- A * A",
            vec![
                Ax(o(1), o(3)),
                Copy(o(1), n(0)),
                Ax(o(1), o(4)),
                Copy(o(1), n(2)),
                Tensor(o(2), n(1), n(3)),
                Quest(o(0), n(4)),
            ],
        );
        let d = p.derivation().unwrap();
        let i = InfId::new;
        let inference =
            |sequent: &[u32], rule, principal, premises: &[InfId], node: Option<u32>| Inference {
                sequent: sequent.iter().map(|&x| o(x)).collect(),
                rule,
                principal,
                premises: premises.to_vec(),
                times: 1,
                cut: None,
                node: node.map(n),
            };
        assert_eq!(
            d.inferences(),
            [
                inference(&[1, 3], Named::from(Rule::Ax), None, &[], Some(0)),
                inference(
                    &[0, 3],
                    Named::from(Rule::Dereliction),
                    Some(0),
                    &[i(0)],
                    Some(1)
                ),
                inference(&[1, 4], Named::from(Rule::Ax), None, &[], Some(2)),
                inference(
                    &[0, 4],
                    Named::from(Rule::Dereliction),
                    Some(0),
                    &[i(2)],
                    Some(3)
                ),
                inference(
                    &[0, 0, 2],
                    Named::from(Rule::Tensor),
                    Some(2),
                    &[i(1), i(3)],
                    Some(4)
                ),
                inference(
                    &[0, 2],
                    Named::from(Rule::Contraction),
                    Some(0),
                    &[i(4)],
                    None
                ),
            ]
        );
        let nodes: Vec<_> = d.inferences().iter().map(Inference::node).collect();
        assert_eq!(
            nodes,
            [Some(0), Some(1), Some(2), Some(3), Some(4), None].map(|n| n.map(NodeId::new))
        );
        assert_eq!(d.root(), i(5));
        assert_eq!(d.inference(i(5)).rule.to_string(), "?c");
        // An invalid proof has no derivation.
        let p = proof("!A |- A * A", vec![Ax(o(1), o(3)), Copy(o(1), n(0))]);
        assert!(p.derivation().is_err());
    }
}
