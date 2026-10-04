// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use super::translate::{Core, Image, pattern};
use super::{Formulas, Logic, Node, NodeId, Symbols, Translation, write_sides};
use crate::occurrences::OccId;
use crate::proofs::style::Drawn;
use crate::proofs::{
    Compact, InfId, Labels, Rule as Linear, TextOptions, ViewError, ViewOptions, WriteError,
};
use crate::{Error, Proof};
use std::fmt::{Display, Formatter, Result as FmtResult, Write};

/// A side of the turnstile.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Side {
    /// Left of `⊢`: a hypothesis.
    Left,
    /// Right of `⊢`.
    Right,
}

impl Side {
    /// Returns the other side.
    const fn flip(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

/// A rule of LK or LJ, as a derivation read back names it.
///
/// The derivations are those of Gentzen's calculi with explicit weakening
/// and contraction: a rule with two premises takes its context either
/// whole into both (as `∧R` of LJ does) or split between them (as `→L`
/// of LJ does); an axiom `A ⊢ A` has no context, while `⊤R` and `⊥L` take
/// any. `¬A` is a connective of its own; in LJ its left rule has the form
/// of `→L` for `A → ⊥`, and in LK the one premise `Γ ⊢ A, Δ`. Every LJ
/// sequent has at most one formula right of `⊢`, and minimal logic has no
/// `⊥L`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Rule {
    /// `ax`: `A ⊢ A`.
    Axiom,
    /// `WL`, weakening of a hypothesis.
    WeakenLeft,
    /// `WR`, weakening right of `⊢`.
    WeakenRight,
    /// `CL`, contraction of a hypothesis.
    ContractLeft,
    /// `CR`, contraction right of `⊢`.
    ContractRight,
    /// `∧L`, both conjuncts.
    AndLeft,
    /// `∧L₁`, the first conjunct.
    AndLeft1,
    /// `∧L₂`, the second conjunct.
    AndLeft2,
    /// `∧R`
    AndRight,
    /// `∨L`
    OrLeft,
    /// `∨R`, both disjuncts (LK).
    OrRight,
    /// `∨R₁`
    OrRight1,
    /// `∨R₂`
    OrRight2,
    /// `→L`
    ImpliesLeft,
    /// `→R`
    ImpliesRight,
    /// `¬L`
    NotLeft,
    /// `¬R`
    NotRight,
    /// `↔L`, both implications.
    IffLeft,
    /// `↔L₁`, the implication from left to right.
    IffLeft1,
    /// `↔L₂`, the implication from right to left.
    IffLeft2,
    /// `↔R`
    IffRight,
    /// `⊥L`, ex falso.
    FalseLeft,
    /// `⊥R`, false dropped from the right.
    FalseRight,
    /// `⊤L`, true dropped from the hypotheses.
    TrueLeft,
    /// `⊤R`
    TrueRight,
}

impl Rule {
    /// Every rule, in the order of declaration, which is the order of
    /// `rule as usize`.
    pub(crate) const ALL: [Self; 25] = {
        use Rule::*;
        [
            Axiom,
            WeakenLeft,
            WeakenRight,
            ContractLeft,
            ContractRight,
            AndLeft,
            AndLeft1,
            AndLeft2,
            AndRight,
            OrLeft,
            OrRight,
            OrRight1,
            OrRight2,
            ImpliesLeft,
            ImpliesRight,
            NotLeft,
            NotRight,
            IffLeft,
            IffLeft1,
            IffLeft2,
            IffRight,
            FalseLeft,
            FalseRight,
            TrueLeft,
            TrueRight,
        ]
    };

    /// Returns the rule's usual spelling.
    pub const fn name(self) -> &'static str {
        use Rule::*;
        match self {
            Axiom => "ax",
            WeakenLeft => "WL",
            WeakenRight => "WR",
            ContractLeft => "CL",
            ContractRight => "CR",
            AndLeft => "∧L",
            AndLeft1 => "∧L₁",
            AndLeft2 => "∧L₂",
            AndRight => "∧R",
            OrLeft => "∨L",
            OrRight => "∨R",
            OrRight1 => "∨R₁",
            OrRight2 => "∨R₂",
            ImpliesLeft => "→L",
            ImpliesRight => "→R",
            NotLeft => "¬L",
            NotRight => "¬R",
            IffLeft => "↔L",
            IffLeft1 => "↔L₁",
            IffLeft2 => "↔L₂",
            IffRight => "↔R",
            FalseLeft => "⊥L",
            FalseRight => "⊥R",
            TrueLeft => "⊤L",
            TrueRight => "⊤R",
        }
    }

    /// Returns the label in the markup the exports set (a subscript as
    /// `_1`), upright or with the side and the letter of a structural
    /// rule as a subscript.
    pub(crate) const fn markup(self, subscript: bool) -> &'static str {
        use Rule::*;
        match (self, subscript) {
            (WeakenLeft, false) => "WL",
            (WeakenLeft, true) => "W_L",
            (WeakenRight, false) => "WR",
            (WeakenRight, true) => "W_R",
            (ContractLeft, false) => "CL",
            (ContractLeft, true) => "C_L",
            (ContractRight, false) => "CR",
            (ContractRight, true) => "C_R",
            (AndLeft, false) => "∧L",
            (AndLeft, true) => "∧_L",
            (AndLeft1, false) => "∧L_1",
            (AndLeft1, true) => "∧_{L1}",
            (AndLeft2, false) => "∧L_2",
            (AndLeft2, true) => "∧_{L2}",
            (AndRight, false) => "∧R",
            (AndRight, true) => "∧_R",
            (OrLeft, false) => "∨L",
            (OrLeft, true) => "∨_L",
            (OrRight, false) => "∨R",
            (OrRight, true) => "∨_R",
            (OrRight1, false) => "∨R_1",
            (OrRight1, true) => "∨_{R1}",
            (OrRight2, false) => "∨R_2",
            (OrRight2, true) => "∨_{R2}",
            (ImpliesLeft, false) => "→L",
            (ImpliesLeft, true) => "→_L",
            (ImpliesRight, false) => "→R",
            (ImpliesRight, true) => "→_R",
            (NotLeft, false) => "¬L",
            (NotLeft, true) => "¬_L",
            (NotRight, false) => "¬R",
            (NotRight, true) => "¬_R",
            (IffLeft, false) => "↔L",
            (IffLeft, true) => "↔_L",
            (IffLeft1, false) => "↔L_1",
            (IffLeft1, true) => "↔_{L1}",
            (IffLeft2, false) => "↔L_2",
            (IffLeft2, true) => "↔_{L2}",
            (IffRight, false) => "↔R",
            (IffRight, true) => "↔_R",
            (FalseLeft, false) => "⊥L",
            (FalseLeft, true) => "⊥_L",
            (FalseRight, false) => "⊥R",
            (FalseRight, true) => "⊥_R",
            (TrueLeft, false) => "⊤L",
            (TrueLeft, true) => "⊤_L",
            (TrueRight, false) => "⊤R",
            (TrueRight, true) => "⊤_R",
            (Axiom, _) => "ax",
        }
    }
}

impl Display for Rule {
    /// Writes the rule's [`name`](Self::name).
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(self.name())
    }
}

/// One inference of a derivation of LK or LJ: the sequent it concludes,
/// the rule, the formula the rule introduces, and its premises.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Inference {
    /// The hypotheses of the sequent concluded, a formula repeated as often
    /// as the sequent holds it.
    pub left: Vec<NodeId>,
    /// The formulas right of `⊢`.
    pub right: Vec<NodeId>,
    /// The rule.
    pub rule: Rule,
    /// The side and the position there of the formula the rule
    /// introduces, removes or copies; `None` for an axiom.
    pub principal: Option<(Side, usize)>,
    /// The premises, in the rule's order.
    pub premises: Vec<InfId>,
}

impl Inference {
    /// Returns the formulas of one side.
    pub(crate) fn side(&self, side: Side) -> &[NodeId] {
        match side {
            Side::Left => &self.left,
            Side::Right => &self.right,
        }
    }
}

/// A derivation of LK or LJ (see [`Rule`]), read back from a linear proof
/// of an image by [`Image::read_back`]: premises precede their conclusion,
/// the root is the last inference.
#[derive(Clone, Debug)]
pub struct Derivation {
    /// The logic.
    logic: Logic,
    /// The formulas, those an equivalence or a negation needs included.
    formulas: Formulas,
    /// The sequent derived.
    left: Vec<NodeId>,
    /// The sequent derived, right of `⊢`.
    right: Vec<NodeId>,
    /// The inferences, premises before conclusions.
    inferences: Vec<Inference>,
}

impl Derivation {
    /// Returns the logic.
    pub fn logic(&self) -> Logic {
        self.logic
    }

    /// Returns the formulas the inferences refer to.
    pub fn formulas(&self) -> &Formulas {
        &self.formulas
    }

    /// Returns every inference, premises before conclusions, the root last.
    pub fn inferences(&self) -> &[Inference] {
        &self.inferences
    }

    /// Returns the inference at `id`, which must belong to this derivation.
    pub fn inference(&self, id: InfId) -> &Inference {
        &self.inferences[id.index()]
    }

    /// Returns the root.
    pub fn root(&self) -> InfId {
        // A derivation read back has an inference for every leaf.
        InfId::new(self.inferences.len() as u32 - 1)
    }

    /// Checks the derivation by the rules of its logic, as [`Rule`]
    /// describes them, and that its root concludes the sequent it was read
    /// back for: each inference on its own, from the formulas as they
    /// stand, sharing nothing with the linear proof or the read-back.
    ///
    /// # Errors
    ///
    /// [`Error::ReadBack`] naming the first inference that breaks a rule.
    pub fn check(&self) -> Result<(), Error> {
        let calculus = self.logic.calculus();
        let fail = |n: usize, why: String| Error::ReadBack {
            calculus,
            reason: format!("inference {n} ({}): {why}", self.inferences[n].rule),
        };
        let root = self
            .inferences
            .len()
            .checked_sub(1)
            .ok_or(Error::ReadBack {
                calculus,
                reason: "no inference".to_owned(),
            })?;
        let root_inference = &self.inferences[root];
        if !same(&root_inference.left, &self.left) || !same(&root_inference.right, &self.right) {
            return Err(fail(
                root,
                "the root does not conclude the sequent".to_owned(),
            ));
        }
        for (n, inference) in self.inferences.iter().enumerate() {
            if inference.premises.iter().any(|p| p.index() >= n) {
                return Err(fail(n, "a premise does not precede it".to_owned()));
            }
            if self.logic != Logic::Classical && inference.right.len() > 1 {
                return Err(fail(n, "more than one formula right of ⊢".to_owned()));
            }
            self.check_one(inference).map_err(|why| fail(n, why))?;
        }
        Ok(())
    }

    /// Checks one inference against its rule.
    fn check_one(&self, inference: &Inference) -> Result<(), String> {
        use Rule::*;
        use Side::{Left, Right};
        let f = &self.formulas;
        let premises: Vec<&Inference> = inference
            .premises
            .iter()
            .map(|&p| &self.inferences[p.index()])
            .collect();
        let count = |n: usize| {
            if premises.len() == n {
                Ok(())
            } else {
                Err(format!("{} premises instead of {n}", premises.len()))
            }
        };
        if inference.rule == Axiom {
            count(0)?;
            return match (&inference.left[..], &inference.right[..]) {
                ([a], [b]) if a == b && inference.principal.is_none() => Ok(()),
                _ => Err("not a sequent A ⊢ A".to_owned()),
            };
        }
        let Some((side, at)) = inference.principal else {
            return Err("no principal formula".to_owned());
        };
        let principal = *inference
            .side(side)
            .get(at)
            .ok_or("the principal position lies outside the sequent")?;
        let node = f.node(principal);
        // The sequent without the principal formula.
        let mut context = (inference.left.clone(), inference.right.clone());
        match side {
            Left => context.0.remove(at),
            Right => context.1.remove(at),
        };
        let with = |added: &[(Side, NodeId)]| {
            let mut sequent = context.clone();
            for &(side, id) in added {
                match side {
                    Left => sequent.0.push(id),
                    Right => sequent.1.push(id),
                }
            }
            sequent
        };
        // One premise that is the context with the formulas given added.
        let unary = |added: &[(Side, NodeId)]| {
            count(1)?;
            if is(premises[0], &with(added)) {
                Ok(())
            } else {
                Err("the premise is not what the rule makes of the conclusion".to_owned())
            }
        };
        // Two premises, each the context, whole or a part of a split, with
        // the formulas given added.
        let binary = |first: &[(Side, NodeId)], second: &[(Side, NodeId)]| {
            count(2)?;
            if is(premises[0], &with(first)) && is(premises[1], &with(second)) {
                return Ok(());
            }
            // Split: what each premise holds beyond its formulas is a part
            // of the context, the two parts together all of it.
            let rest = |p: &Inference, added: &[(Side, NodeId)]| {
                let mut left = p.left.clone();
                let mut right = p.right.clone();
                for &(side, id) in added {
                    let list = if side == Left { &mut left } else { &mut right };
                    let i = list.iter().position(|&x| x == id)?;
                    list.remove(i);
                }
                Some((left, right))
            };
            if let (Some(mut a), Some(b)) = (rest(premises[0], first), rest(premises[1], second)) {
                a.0.extend(b.0);
                a.1.extend(b.1);
                if same(&a.0, &context.0) && same(&a.1, &context.1) {
                    return Ok(());
                }
            }
            Err("the premises do not share or split the context".to_owned())
        };
        let leaf = || count(0);
        match (inference.rule, side, node) {
            (WeakenLeft, Left, _) | (WeakenRight, Right, _) => unary(&[]),
            (ContractLeft, Left, _) => unary(&[(Left, principal), (Left, principal)]),
            (ContractRight, Right, _) => unary(&[(Right, principal), (Right, principal)]),
            (AndLeft, Left, Node::And(a, b)) => unary(&[(Left, a), (Left, b)]),
            (AndLeft1, Left, Node::And(a, _)) => unary(&[(Left, a)]),
            (AndLeft2, Left, Node::And(_, b)) => unary(&[(Left, b)]),
            (AndRight, Right, Node::And(a, b)) => binary(&[(Right, a)], &[(Right, b)]),
            (OrLeft, Left, Node::Or(a, b)) => binary(&[(Left, a)], &[(Left, b)]),
            (OrRight, Right, Node::Or(a, b)) => unary(&[(Right, a), (Right, b)]),
            (OrRight1, Right, Node::Or(a, _)) => unary(&[(Right, a)]),
            (OrRight2, Right, Node::Or(_, b)) => unary(&[(Right, b)]),
            (ImpliesLeft, Left, Node::Implies(a, b)) => binary(&[(Right, a)], &[(Left, b)]),
            (ImpliesRight, Right, Node::Implies(a, b)) => unary(&[(Left, a), (Right, b)]),
            // `¬A` as itself, or as `A → ⊥` (LJ).
            (NotRight, Right, Node::Not(a)) => unary(&[(Left, a)]).or_else(|_| {
                let falsity = self.formulas.ids.get(&Node::False).copied();
                unary(&[
                    (Left, a),
                    (Right, falsity.ok_or("no ⊥ for the form of →R")?),
                ])
            }),
            (NotLeft, Left, Node::Not(a)) if premises.len() == 1 => unary(&[(Right, a)]),
            (NotLeft, Left, Node::Not(a)) => {
                let falsity = self.formulas.ids.get(&Node::False).copied();
                let falsity = falsity.ok_or("no ⊥ for the form of →L")?;
                binary(&[(Right, a)], &[(Left, falsity)])
            }
            (IffLeft | IffLeft1 | IffLeft2 | IffRight, _, Node::Iff(a, b)) => {
                let forward = self.formulas.ids.get(&Node::Implies(a, b)).copied();
                let backward = self.formulas.ids.get(&Node::Implies(b, a)).copied();
                let (Some(forward), Some(backward)) = (forward, backward) else {
                    return Err("the implications of the equivalence are missing".to_owned());
                };
                match (inference.rule, side) {
                    (IffLeft, Left) => unary(&[(Left, forward), (Left, backward)]),
                    (IffLeft1, Left) => unary(&[(Left, forward)]),
                    (IffLeft2, Left) => unary(&[(Left, backward)]),
                    (IffRight, Right) => binary(&[(Right, forward)], &[(Right, backward)]),
                    _ => Err("the rule does not apply on this side".to_owned()),
                }
            }
            (FalseLeft, Left, Node::False) if self.logic != Logic::Minimal => leaf(),
            (FalseLeft, ..) if self.logic == Logic::Minimal => {
                Err("minimal logic has no ex falso".to_owned())
            }
            (FalseRight, Right, Node::False) | (TrueLeft, Left, Node::True) => unary(&[]),
            (TrueRight, Right, Node::True) => leaf(),
            _ => Err("the rule does not apply to the principal formula".to_owned()),
        }
    }
}

impl Derivation {
    /// Returns the width in characters and the number of lines of the tree
    /// that [`write_text`](Self::write_text) draws under the options,
    /// without drawing it.
    pub fn text_size(&self, options: &TextOptions) -> (usize, usize) {
        crate::proofs::fmt::text_size(self, options)
    }

    /// Writes the derivation as a tree of sequents `Γ ⊢ Δ`, one line per
    /// row, without a trailing newline, as
    /// [`crate::Derivation::write_text`] draws a linear one, and asks
    /// `stop` before every piece of a row. The labels are those of
    /// [`Rule`] (a label table of [`Labels::Table`] is keyed by the
    /// linear rules, so it leaves them upright).
    pub fn write_text(
        &self,
        options: &TextOptions,
        out: &mut impl Write,
        stop: impl FnMut() -> bool,
    ) -> Result<(), WriteError> {
        crate::proofs::fmt::write_text(self, options, out, stop)
    }
}

impl Display for Derivation {
    /// Draws the derivation as a tree of sequents under the default
    /// [`TextOptions`], one line per row, without a trailing newline.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.write_text(&TextOptions::default(), f, || false)
            .map_err(|_| std::fmt::Error)
    }
}

impl Drawn for Derivation {
    const RULES: usize = Rule::ALL.len();
    const OPEN: Option<usize> = None;

    fn markup(rule: usize, labels: &Labels) -> Option<&str> {
        let rule = Rule::ALL[rule];
        match labels {
            Labels::Upright | Labels::Table(_) => Some(rule.markup(false)),
            Labels::Subscript => Some(rule.markup(true)),
            Labels::Off => None,
        }
    }

    fn len(&self) -> usize {
        self.inferences.len()
    }

    fn root(&self) -> InfId {
        Derivation::root(self)
    }

    fn premises(&self, id: InfId) -> &[InfId] {
        &self.inference(id).premises
    }

    fn rule(&self, id: InfId) -> usize {
        self.inference(id).rule as usize
    }

    fn name(&self, id: InfId) -> &'static str {
        self.inference(id).rule.name()
    }

    fn times(&self, _: InfId) -> u32 {
        1
    }

    fn write_sequent(&self, out: &mut impl Write, id: InfId) -> FmtResult {
        let inference = self.inference(id);
        let mut text = String::new();
        let sides = (&inference.left[..], &inference.right[..]);
        write_sides(
            &mut text,
            &self.formulas,
            sides.0,
            sides.1,
            &Symbols::UNICODE,
            "⊢",
        );
        out.write_str(&text)
    }

    #[cfg(any(feature = "latex", feature = "typst", feature = "svg"))]
    fn sequent(
        &self,
        out: &mut String,
        notation: &crate::export::notation::Notation,
        id: InfId,
        aligned: bool,
        marks: bool,
    ) {
        let inference = self.inference(id);
        let sides = (&inference.left[..], &inference.right[..]);
        notation.ordinary(out, &self.formulas, sides, aligned, marks);
    }

    #[cfg(feature = "svg")]
    fn positions(&self, id: InfId) -> Vec<usize> {
        let inference = self.inference(id);
        (0..inference.left.len() + inference.right.len()).collect()
    }
}

/// Returns whether two lists are the same multiset.
fn same(a: &[NodeId], b: &[NodeId]) -> bool {
    let (mut a, mut b) = (a.to_vec(), b.to_vec());
    a.sort_unstable();
    b.sort_unstable();
    a == b
}

/// Returns whether an inference concludes the sequent given, as multisets.
fn is(inference: &Inference, sequent: &(Vec<NodeId>, Vec<NodeId>)) -> bool {
    same(&inference.left, &sequent.0) && same(&inference.right, &sequent.1)
}

/// A formula followed through its classical negations.
struct Unwound {
    /// The negations passed, outermost first, each with its side.
    chain: Vec<(NodeId, Side)>,
    /// The formula under them, its translation function and its side.
    core: (NodeId, u8, Side),
}

/// What a linear occurrence is the image of.
#[derive(Clone, Copy, Debug)]
struct Tag {
    /// The ordinary formula, as the sequents show it.
    node: NodeId,
    /// Its translation function.
    function: u8,
    /// The `!`s above the pattern's connective still to pass.
    bangs: u8,
    /// Its side.
    side: Side,
}

impl Image {
    /// Unfolds a proof of the image into the derivation the read-back
    /// reads: two-sided for an image in ILL, one-sided otherwise, never
    /// compact; `view` bounds it and `stop` ends it as for
    /// [`Proof::derivation_with`].
    ///
    /// # Errors
    ///
    /// As [`Proof::derivation_with`].
    pub fn linear_derivation<'p>(
        &self,
        proof: &'p Proof,
        view: &ViewOptions,
        stop: impl FnMut() -> bool,
    ) -> Result<crate::Derivation<'p>, ViewError> {
        let view = view.compact(Compact::Never);
        if self.mode.intuitionistic {
            proof.two_sided_derivation_with(&view, stop)
        } else {
            proof.derivation_with(&view, stop)
        }
    }

    /// Reads a derivation of the image back as one of LK (classical
    /// logic) or LJ (intuitionistic and minimal logic) of the ordinary
    /// sequent: a linear rule on an image is the rule of its connective, a
    /// dereliction or a promotion is no inference, a contraction or a
    /// weakening is one of the ordinary formula, and a classical negation,
    /// which has no image of its own, is a `¬` rule where its operand is
    /// taken apart. The derivation is from
    /// [`linear_derivation`](Self::linear_derivation);
    /// [`Derivation::check`] checks the result.
    ///
    /// # Errors
    ///
    /// [`Error::ReadBack`] for a derivation that is not of this image, or
    /// a rule with no reading.
    pub fn read_back(&self, linear: &crate::Derivation) -> Result<Derivation, Error> {
        let calculus = self.logic.calculus();
        let fail = |reason: String| Error::ReadBack { calculus, reason };
        let forest = linear.forest();
        if forest.sequent() != &self.sequent {
            return Err(fail("the proof is not of the image".to_owned()));
        }
        let formulas = &self.ordinary.formulas;
        // Every occurrence's tag, parents before children.
        let mut tags: Vec<Option<Tag>> = vec![None; forest.len()];
        for (&root, info) in forest.roots().iter().zip(&self.roots) {
            tags[root.index()] = Some(Tag {
                node: info.node,
                function: info.function,
                bangs: info.bangs,
                side: info.side,
            });
        }
        for o in forest.ids() {
            let tag = tags[o.index()].expect("a parent is tagged before its children");
            if tag.bangs > 0 {
                if let Some(child) = forest.left(o) {
                    tags[child.index()] = Some(Tag {
                        bangs: tag.bangs - 1,
                        ..tag
                    });
                }
                continue;
            }
            let (node, function, side) = self.unwind(tag.node, tag.function, tag.side).core;
            let p = pattern(
                formulas,
                &self.extra,
                self.translation,
                self.logic,
                node,
                function,
            );
            for (k, child) in forest.children(o).enumerate() {
                let Some((x, fx, extra)) = p.operands[k] else {
                    return Err(fail(format!("occurrence {} has no operand {k}", o.index())));
                };
                let px = pattern(formulas, &self.extra, self.translation, self.logic, x, fx);
                let flips = match p.core {
                    Core::Lollipop => k == 0,
                    _ => false,
                };
                // The classical implication's antecedent is on the other
                // side through its function.
                let side = if self.translation == Translation::Affine {
                    if fx == 0 { Side::Right } else { Side::Left }
                } else if flips {
                    side.flip()
                } else {
                    side
                };
                tags[child.index()] = Some(Tag {
                    node: x,
                    function: fx,
                    bangs: px.bangs + extra,
                    side,
                });
            }
        }
        let tag = |o: OccId| tags[o.index()].expect("every occurrence is tagged");

        let mut out = Vec::new();
        // Per linear inference, the ordinary inference that stands for it.
        let mut mapped: Vec<InfId> = Vec::with_capacity(linear.inferences().len());
        for inference in linear.inferences() {
            let members: Vec<(NodeId, Side)> = inference
                .sequent
                .iter()
                .map(|&o| (tag(o).node, tag(o).side))
                .collect();
            let premises: Vec<InfId> = inference
                .premises
                .iter()
                .map(|p| mapped[p.index()])
                .collect();
            let rule = inference.rule.classical();
            let next = match rule {
                Linear::Promotion | Linear::Dereliction => premises[0],
                Linear::Contraction | Linear::Weakening | Linear::AffineWeakening => {
                    let at = inference
                        .principal
                        .expect("a structural rule has a principal formula");
                    let (_, side) = members[at];
                    let rule = match (rule, side) {
                        (Linear::Contraction, Side::Left) => Rule::ContractLeft,
                        (Linear::Contraction, Side::Right) => Rule::ContractRight,
                        (_, Side::Left) => Rule::WeakenLeft,
                        (_, Side::Right) => Rule::WeakenRight,
                    };
                    push(&mut out, &members, at, rule, premises)
                }
                Linear::Ax => {
                    let mut members = members;
                    let chains: Vec<_> = (0..2)
                        .map(|at| {
                            let (node, side) = members[at];
                            let Unwound { chain, core } = self.unwind(node, 0, side);
                            members[at] = (core.0, core.2);
                            (at, chain)
                        })
                        .collect();
                    let mut id = push(&mut out, &members, usize::MAX, Rule::Axiom, premises);
                    for (at, chain) in chains {
                        id = negations(&mut out, &mut members, at, chain, id);
                    }
                    id
                }
                rule => {
                    let at = inference
                        .principal
                        .ok_or_else(|| fail("no principal formula".to_owned()))?;
                    let mut members = members;
                    let (node, side) = members[at];
                    let Unwound { chain, core } = self.unwind(node, 0, side);
                    members[at] = (core.0, core.2);
                    let ordinary = self
                        .rule_of(formulas.node(core.0), core.2, rule)
                        .ok_or_else(|| {
                            fail(format!(
                                "the linear rule {rule} on {} has no reading",
                                formulas.formula(core.0)
                            ))
                        })?;
                    let id = push(&mut out, &members, at, ordinary, premises);
                    negations(&mut out, &mut members, at, chain, id)
                }
            };
            mapped.push(next);
        }
        Ok(Derivation {
            logic: self.logic,
            formulas: self.ordinary.formulas.clone(),
            left: self.ordinary.left.clone(),
            right: self.ordinary.right.clone(),
            inferences: out,
        })
    }

    /// Follows a formula through the classical negations, which have no
    /// image of their own, to the formula whose connective its image has:
    /// returns the negations passed and that formula. Other translations
    /// pass none.
    fn unwind(&self, mut node: NodeId, mut function: u8, mut side: Side) -> Unwound {
        let mut chain = Vec::new();
        if self.translation == Translation::Affine {
            while let Node::Not(a) = self.ordinary.formulas.node(node) {
                chain.push((node, side));
                node = a;
                side = side.flip();
                function = 1 - function;
            }
        }
        Unwound {
            chain,
            core: (node, function, side),
        }
    }

    /// Returns the ordinary rule a classical linear rule is on the image of
    /// `node` standing on `side`.
    fn rule_of(&self, node: Node, side: Side, rule: Linear) -> Option<Rule> {
        use Linear as L;
        use Side::{Left, Right};
        Some(match (node, side, rule) {
            (Node::And(..), Right, L::With | L::Tensor) => Rule::AndRight,
            (Node::And(..), Left, L::Par) => Rule::AndLeft,
            (Node::And(..), Left, L::PlusLeft) => Rule::AndLeft1,
            (Node::And(..), Left, L::PlusRight) => Rule::AndLeft2,
            (Node::Or(..), Right, L::Par) => Rule::OrRight,
            (Node::Or(..), Right, L::PlusLeft) => Rule::OrRight1,
            (Node::Or(..), Right, L::PlusRight) => Rule::OrRight2,
            (Node::Or(..), Left, L::With) => Rule::OrLeft,
            (Node::Implies(..), Right, L::Par) => Rule::ImpliesRight,
            (Node::Implies(..), Left, L::Tensor | L::With) => Rule::ImpliesLeft,
            (Node::Not(_), Right, L::Par) => Rule::NotRight,
            (Node::Not(_), Left, L::Tensor) => Rule::NotLeft,
            (Node::Iff(..), Right, L::With | L::Tensor) => Rule::IffRight,
            (Node::Iff(..), Left, L::Par) => Rule::IffLeft,
            (Node::Iff(..), Left, L::PlusLeft) => Rule::IffLeft1,
            (Node::Iff(..), Left, L::PlusRight) => Rule::IffLeft2,
            (Node::True, Right, L::Top | L::One) => Rule::TrueRight,
            (Node::True, Left, L::Bot) => Rule::TrueLeft,
            (Node::False, Left, L::Top) => Rule::FalseLeft,
            (Node::False, Right, L::Bot) => Rule::FalseRight,
            _ => return None,
        })
    }
}

/// Adds the inference concluding `members` by `rule`, with the formula at
/// `at` principal (none for an axiom), and returns its id.
fn push(
    out: &mut Vec<Inference>,
    members: &[(NodeId, Side)],
    at: usize,
    rule: Rule,
    premises: Vec<InfId>,
) -> InfId {
    let (mut left, mut right) = (Vec::new(), Vec::new());
    let mut principal = None;
    for (i, &(node, side)) in members.iter().enumerate() {
        let list = if side == Side::Left {
            &mut left
        } else {
            &mut right
        };
        if i == at {
            principal = Some((side, list.len()));
        }
        list.push(node);
    }
    out.push(Inference {
        left,
        right,
        rule,
        principal,
        premises,
    });
    // A read-back has an inference for every linear inference and every
    // negation passed, as many as the linear derivation's `InfId` counts
    // times the depth of a formula, far below `u32::MAX` for any proof a
    // search returns within its memory bound.
    InfId::new(out.len() as u32 - 1)
}

/// Adds below `id` the `¬` inferences that take the formula at `at` from
/// its core back to the negations of `chain`, the innermost first, and
/// returns the last one.
fn negations(
    out: &mut Vec<Inference>,
    members: &mut [(NodeId, Side)],
    at: usize,
    chain: Vec<(NodeId, Side)>,
    mut id: InfId,
) -> InfId {
    for (node, side) in chain.into_iter().rev() {
        members[at] = (node, side);
        let rule = match side {
            Side::Left => Rule::NotLeft,
            Side::Right => Rule::NotRight,
        };
        id = push(out, members, at, rule, vec![id]);
    }
    id
}

#[cfg(all(test, feature = "parse"))]
mod tests {
    use super::super::{Sequent, translate};
    use super::*;
    use crate::{Options, Verdict, prove};

    /// The text tree draws `Γ ⊢ Δ` with the rules of LJ, and its size is
    /// that of the text.
    #[test]
    fn text_tree() {
        let sequent: Sequent = "a -> b, b -> c |- a -> c".parse().unwrap();
        let image = translate(&sequent, Logic::Intuitionistic, Translation::CallByName).unwrap();
        let outcome = prove(image.sequent(), image.mode(), &Options::default()).unwrap();
        let Verdict::Proved(proof) = &outcome.verdict else {
            panic!("provable");
        };
        let linear = image
            .linear_derivation(proof, &ViewOptions::default(), || false)
            .unwrap();
        let derivation = image.read_back(&linear).unwrap();
        let tree = derivation.to_string();
        assert_eq!(
            tree,
            "───── ax   ───── ax\n\
             a ⊢ a      b ⊢ b\n\
             ──────────────── →L   ───── ax\n\
            \x20 a, a → b ⊢ b        c ⊢ c\n\
            \x20 ───────────────────────── →L\n\
            \x20    a, a → b, b → c ⊢ c\n\
            \x20    ──────────────────── →R\n\
            \x20    a → b, b → c ⊢ a → c"
        );
        let width = tree.lines().map(|l| l.chars().count()).max();
        let size = derivation.text_size(&crate::TextOptions::default());
        assert_eq!(Some(size), width.map(|w| (w, tree.lines().count())));
    }
}
