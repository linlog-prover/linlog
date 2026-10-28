// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The two-sided view of a derivation of classical linear logic: the
//! one-sided derivation read through the record of how its sequent was
//! written, every sequent split by side, every rule named by the side of
//! its principal formula, and a negation step wherever a negation was
//! written, which the one-sided calculus does not have.

use super::view::{Derivation, InfId};
use crate::Error;
use crate::limits::{Limits, Refusal, Space};
use crate::occurrences::written::{self, Written};
use crate::occurrences::{Forest, Member, OccId, Side};
use crate::proofs::style::{Drawn, Labels};
use crate::proofs::{Rule, Size, TextOptions};
use std::fmt::{Display, Formatter, Result as FmtResult, Write};

/// A rule of the two-sided sequent calculus of classical linear logic, as
/// the two-sided view ([`TwoSided`]) names it: the one-sided rule applied
/// to a formula on one side of `⊢`, or a negation step, which moves a
/// formula written with a negation to the other side of `⊢` without it.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ClassicalRule {
    /// `ax`: `A ⊢ A`.
    Axiom,
    /// `⊗L`
    TensorLeft,
    /// `⊗R`
    TensorRight,
    /// `⅋L`
    ParLeft,
    /// `⅋R`
    ParRight,
    /// `⊸L`
    LollipopLeft,
    /// `⊸R`
    LollipopRight,
    /// `1L`
    OneLeft,
    /// `1R`
    OneRight,
    /// `⊥L`
    BotLeft,
    /// `⊥R`
    BotRight,
    /// `&L₁`
    WithLeft1,
    /// `&L₂`
    WithLeft2,
    /// `&R`
    WithRight,
    /// `⊕L`
    PlusLeft,
    /// `⊕R₁`
    PlusRight1,
    /// `⊕R₂`
    PlusRight2,
    /// `⊤R`
    TopRight,
    /// `0L`
    ZeroLeft,
    /// `!L`, dereliction.
    BangLeft,
    /// `!R`, promotion.
    BangRight,
    /// `!c`, contraction of a `!` formula on the left.
    BangContraction,
    /// `!w`, weakening by a `!` formula on the left.
    BangWeakening,
    /// `?L`, promotion on the left.
    QuestLeft,
    /// `?R`, dereliction on the right.
    QuestRight,
    /// `?c`, contraction of a `?` formula on the right.
    QuestContraction,
    /// `?w`, weakening by a `?` formula on the right.
    QuestWeakening,
    /// `mix`
    Mix,
    /// `wk`, weakening by a formula that is no exponential, in affine
    /// mode.
    Weakening,
    /// `cut`: from `Γ ⊢ A, Δ` and `Γ', A ⊢ Δ'` to `Γ, Γ' ⊢ Δ, Δ'`.
    Cut,
    /// `~L`: from `Γ ⊢ A, Δ` to `Γ, A⊥ ⊢ Δ`.
    NegationLeft,
    /// `~R`: from `Γ, A ⊢ Δ` to `Γ ⊢ A⊥, Δ`.
    NegationRight,
    /// An open goal of a proof in progress.
    Open,
}

impl ClassicalRule {
    /// Every rule, in the order of declaration, by which the label tables
    /// are indexed.
    pub(crate) const ALL: [Self; 33] = {
        use ClassicalRule::*;
        [
            Axiom,
            TensorLeft,
            TensorRight,
            ParLeft,
            ParRight,
            LollipopLeft,
            LollipopRight,
            OneLeft,
            OneRight,
            BotLeft,
            BotRight,
            WithLeft1,
            WithLeft2,
            WithRight,
            PlusLeft,
            PlusRight1,
            PlusRight2,
            TopRight,
            ZeroLeft,
            BangLeft,
            BangRight,
            BangContraction,
            BangWeakening,
            QuestLeft,
            QuestRight,
            QuestContraction,
            QuestWeakening,
            Mix,
            Weakening,
            Cut,
            NegationLeft,
            NegationRight,
            Open,
        ]
    };

    /// The upright labels, in the order of [`ALL`](Self::ALL).
    const UPRIGHT: [&'static str; 33] = [
        "ax", "⊗L", "⊗R", "⅋L", "⅋R", "⊸L", "⊸R", "1L", "1R", "⊥L", "⊥R", "&L_1", "&L_2", "&R",
        "⊕L", "⊕R_1", "⊕R_2", "⊤R", "0L", "!L", "!R", "!c", "!w", "?L", "?R", "?c", "?w", "mix",
        "wk", "cut", "~L", "~R", "",
    ];

    /// The labels with the side as a subscript, in the order of
    /// [`ALL`](Self::ALL).
    const SUBSCRIPT: [&'static str; 33] = [
        "ax", "⊗_L", "⊗_R", "⅋_L", "⅋_R", "⊸_L", "⊸_R", "1_L", "1_R", "⊥_L", "⊥_R", "&_{L1}",
        "&_{L2}", "&_R", "⊕_L", "⊕_{R1}", "⊕_{R2}", "⊤_R", "0_L", "!_L", "!_R", "!_c", "!_w",
        "?_L", "?_R", "?_c", "?_w", "mix", "wk", "cut", "~_L", "~_R", "",
    ];

    /// Returns the rule's name: `⊗L`, `⅋R`, `⊸L`, `&L₁`, `~R`, and so on.
    pub const fn name(self) -> &'static str {
        use ClassicalRule::*;
        match self {
            Axiom => "ax",
            TensorLeft => "⊗L",
            TensorRight => "⊗R",
            ParLeft => "⅋L",
            ParRight => "⅋R",
            LollipopLeft => "⊸L",
            LollipopRight => "⊸R",
            OneLeft => "1L",
            OneRight => "1R",
            BotLeft => "⊥L",
            BotRight => "⊥R",
            WithLeft1 => "&L₁",
            WithLeft2 => "&L₂",
            WithRight => "&R",
            PlusLeft => "⊕L",
            PlusRight1 => "⊕R₁",
            PlusRight2 => "⊕R₂",
            TopRight => "⊤R",
            ZeroLeft => "0L",
            BangLeft => "!L",
            BangRight => "!R",
            BangContraction => "!c",
            BangWeakening => "!w",
            QuestLeft => "?L",
            QuestRight => "?R",
            QuestContraction => "?c",
            QuestWeakening => "?w",
            Mix => "mix",
            Weakening => "wk",
            Cut => "cut",
            NegationLeft => "~L",
            NegationRight => "~R",
            Open => "open",
        }
    }

    /// Returns the one-sided rule the rule is, or `None` for a negation
    /// step, which the one-sided calculus does not have: its premise and
    /// its conclusion are one one-sided sequent.
    pub const fn rule(self) -> Option<Rule> {
        use ClassicalRule::*;
        Some(match self {
            Axiom => Rule::Ax,
            TensorRight | ParLeft | LollipopLeft => Rule::Tensor,
            TensorLeft | ParRight | LollipopRight => Rule::Par,
            OneRight | BotLeft => Rule::One,
            OneLeft | BotRight => Rule::Bot,
            WithRight | PlusLeft => Rule::With,
            PlusRight1 | WithLeft1 => Rule::PlusLeft,
            PlusRight2 | WithLeft2 => Rule::PlusRight,
            TopRight | ZeroLeft => Rule::Top,
            BangRight | QuestLeft => Rule::Promotion,
            BangLeft | QuestRight => Rule::Dereliction,
            BangContraction | QuestContraction => Rule::Contraction,
            BangWeakening | QuestWeakening => Rule::Weakening,
            Mix => Rule::Mix,
            Weakening => Rule::AffineWeakening,
            Cut => Rule::Cut,
            Open => Rule::Open,
            NegationLeft | NegationRight => return None,
        })
    }

    /// Returns the side of `⊢` the principal formula stands on in the
    /// rule's conclusion, or `None` for the axiom, Mix, weakening, the cut
    /// and an open goal, whose names have no side.
    pub const fn side(self) -> Option<Side> {
        use ClassicalRule::*;
        match self {
            TensorLeft | ParLeft | LollipopLeft | OneLeft | BotLeft | WithLeft1 | WithLeft2
            | PlusLeft | ZeroLeft | BangLeft | BangContraction | BangWeakening | QuestLeft
            | NegationLeft => Some(Side::Input),
            TensorRight | ParRight | LollipopRight | OneRight | BotRight | WithRight
            | PlusRight1 | PlusRight2 | TopRight | BangRight | QuestRight | QuestContraction
            | QuestWeakening | NegationRight => Some(Side::Output),
            Axiom | Mix | Weakening | Cut | Open => None,
        }
    }

    /// Returns the one-sided rule `rule` applied to a formula written as
    /// `written`, named two-sided: a `⊗` on the input side is `⅋L`, or
    /// `⊸L` where an implication was written, and so on.
    fn applied(rule: Rule, written: Written) -> Self {
        use ClassicalRule::*;
        let left = written.side() == Side::Input;
        let pick = |input, output| if left { input } else { output };
        match rule {
            // The view refuses a sequent with binders.
            Rule::Forall | Rule::Exists => unreachable!("no binder in the classical view"),
            Rule::Ax => Axiom,
            Rule::Tensor if written.is_implication() => LollipopLeft,
            Rule::Tensor => pick(ParLeft, TensorRight),
            Rule::Par if written.is_implication() => LollipopRight,
            Rule::Par => pick(TensorLeft, ParRight),
            Rule::One => pick(BotLeft, OneRight),
            Rule::Bot => pick(OneLeft, BotRight),
            Rule::With => pick(PlusLeft, WithRight),
            Rule::PlusLeft => pick(WithLeft1, PlusRight1),
            Rule::PlusRight => pick(WithLeft2, PlusRight2),
            Rule::Top => pick(ZeroLeft, TopRight),
            Rule::Promotion => pick(QuestLeft, BangRight),
            Rule::Dereliction => pick(BangLeft, QuestRight),
            Rule::Contraction => pick(BangContraction, QuestContraction),
            Rule::Weakening => pick(BangWeakening, QuestWeakening),
            Rule::Mix => Mix,
            Rule::AffineWeakening => Weakening,
            Rule::Cut => Cut,
            Rule::Open => Open,
        }
    }

    /// Returns the negation step whose principal formula stands on `side`
    /// in its conclusion.
    const fn negation(side: Side) -> Self {
        match side {
            Side::Input => Self::NegationLeft,
            Side::Output => Self::NegationRight,
        }
    }

    /// Returns the rule's label in the markup of the labels, upright or
    /// with the side as a subscript; empty for an open goal.
    pub(crate) const fn markup(self, subscript: bool) -> &'static str {
        if subscript {
            Self::SUBSCRIPT[self as usize]
        } else {
            Self::UPRIGHT[self as usize]
        }
    }
}

impl Display for ClassicalRule {
    /// Writes the rule's [`name`](Self::name).
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(self.name())
    }
}

/// One inference of the two-sided view: the one-sided sequent it shows,
/// which of its formulas stand before their negation step, its rule and
/// its premises.
#[derive(Clone, Debug)]
struct Shown {
    /// The one-sided inference whose sequent this one shows.
    base: InfId,
    /// The positions in that sequent of the formulas shown as written
    /// inside their parent, negation and all, on the side their parent
    /// puts them: those whose negation step stands above.
    pending: Vec<usize>,
    /// The positions in that sequent of formulas it leaves out: the
    /// copies the upper half of a run of contractions has not made yet,
    /// or what the lower half of a run of weakenings weakens, where a run
    /// of the one-sided derivation is split by side.
    hidden: Vec<usize>,
    /// The rule.
    rule: ClassicalRule,
    /// The premises, in the rule's order.
    premises: Vec<InfId>,
    /// How many applications of the rule it stands for.
    times: u32,
}

/// The two-sided view of a derivation of classical linear logic, as its
/// sequent was written: every inference's sequent its formulas split by
/// the side of `⊢` they stand on as written, every rule named two-sided
/// (`⊗L`, `⅋R`, `⊸L`, …, [`ClassicalRule`]), and a negation step (`~L`,
/// `~R`) inserted wherever a negation was written, which the one-sided
/// proof absorbed: from `Γ, A ⊢ Δ` to `Γ ⊢ A⊥, Δ`.
///
/// A one-sided proof has many two-sided derivations; the record of the
/// written sequent ([`ClassicalSequent::writing`](crate::ClassicalSequent::writing))
/// picks the one written. A formula is shown on the side its connective
/// was written on; one written with a negation is shown so, on the side
/// its parent puts it, only in the premise of the rule that exposes it
/// (or in the conclusion, for a formula of the sequent), and a negation
/// step right above moves it, so that every rule acts on formulas as they
/// were written. A run of weakenings or contractions that a compact view
/// draws as one inference and that acts on both sides of `⊢` is shown as
/// two, `!w*` below `?w*`, so that every inference is one rule. The
/// derivation of a goal off the roots (a proof of a goal) ends in the goal
/// as the rules above it read it, with no negation step below, as the goal
/// stands in a larger derivation. Read one-sided (every formula on the
/// left negated and moved right), every inference is the one-sided
/// inference it shows, a negation step its premise's sequent again.
///
/// [`Display`] draws the tree as [`Derivation`]'s does.
///
/// # Examples
///
#[cfg_attr(feature = "parse", doc = "```")]
#[cfg_attr(not(feature = "parse"), doc = "```ignore")]
/// use linlog::{ClassicalSequent, Limits, Mode, Options, TwoSided, prove};
///
/// let written: ClassicalSequent = "|- A^ par A".parse()?;
/// let outcome = prove(written.sequent(), Mode::CLASSICAL, &Options::default())?;
/// let proof = outcome.verdict.proof().unwrap();
/// let derivation = proof.derivation()?;
/// let view = TwoSided::new(&derivation, written.writing(), &Limits::default())?;
/// assert_eq!(
///     view.to_string(),
///     " ───── ax\n A ⊢ A\n─────── ~R\n⊢ ~A, A\n──────── ⅋R\n⊢ ~A ⅋ A"
/// );
/// # Ok::<(), linlog::Error>(())
/// ```
#[derive(Clone, Debug)]
pub struct TwoSided<'a> {
    /// The one-sided derivation shown.
    derivation: &'a Derivation<'a>,
    /// How every occurrence of its forest was written.
    writing: &'a [Written],
    /// The inferences, premises before conclusions, the root last.
    inferences: Vec<Shown>,
}

impl<'a> TwoSided<'a> {
    /// Returns the two-sided view of a one-sided derivation whose sequent
    /// was written as `writing` records, unless the view is estimated
    /// past `limits.derivation_bytes`: each negation step repeats a
    /// sequent, so the view holds at most twice as many inferences more
    /// than the derivation, and one per formula of the sequent.
    ///
    /// # Errors
    ///
    /// [`Refusal::Output`] past the bound and [`Refusal::Index`] for more
    /// inferences than an [`InfId`] counts, which only a call without a
    /// bound can reach, each with nothing built.
    ///
    /// # Panics
    ///
    /// For a record of another sequent than the derivation's: one entry
    /// per occurrence of its forest, as
    /// [`ClassicalSequent::writing`](crate::ClassicalSequent::writing) or
    /// a session's [`writing`](crate::Interactive::writing) gives it.
    pub fn new(
        derivation: &'a Derivation<'a>,
        writing: &'a [Written],
        limits: &Limits,
    ) -> Result<Self, Error> {
        let forest = derivation.forest();
        if let Some(fault) = written::fault(forest, writing) {
            panic!("the record is not one of the derivation's sequent: {fault}");
        }
        if forest.has_binders() {
            return Err(Error::Quantifiers {
                what: "the classical two-sided view",
            });
        }
        let view = Self {
            derivation,
            writing,
            inferences: Vec::new(),
        };
        // Built twice over the same walk: first counted, then made, so that
        // nothing past the bound is made.
        let mut counted = Counted::new(derivation);
        view.walk(&mut counted);
        if counted.inferences > Derivation::MOST {
            return Err(Error::Refused(Refusal::Index {
                what: Space::Inference,
                count: counted.inferences,
                most: Derivation::MOST,
            }));
        }
        if let Some(bound) = limits.derivation_bytes
            && counted.bytes > bound
        {
            return Err(Error::Refused(Refusal::Output {
                what: "derivation",
                estimate_bytes: counted.bytes,
                limit_bytes: bound,
                least_bytes: None,
            }));
        }
        // At most `Derivation::MOST`, which a `usize` holds where a
        // derivation of that many could be built.
        let mut made = Vec::with_capacity(usize::try_from(counted.inferences).unwrap_or(0));
        view.walk(&mut made);
        Ok(Self {
            inferences: made,
            ..view
        })
    }

    /// Walks the view's inferences in their order, premises before
    /// conclusions, handing each to `sink`, which returns its id.
    fn walk(&self, sink: &mut impl Sink) {
        let derivation = self.derivation;
        let forest = derivation.forest();
        let mut ids = Vec::with_capacity(derivation.inferences().len());
        for (index, inference) in derivation.inferences().iter().enumerate() {
            let rule = inference.rule().rule;
            let principal = inference
                .principal()
                .map(|position| inference.sequent()[position].occ());
            let mut premises = Vec::with_capacity(inference.premises().len());
            for (j, &premise) in inference.premises().iter().enumerate() {
                let top: InfId = ids[premise.index()];
                // A cut puts its cut formula into its first premise and the
                // dual into its second, as roots are written.
                let exposed = match (inference.cut(), principal) {
                    (Some(a), _) => {
                        let a = a.occ();
                        [Some(if j == 0 { a } else { forest.dual(a).unwrap() }), None]
                    }
                    (None, Some(p)) => exposed(forest, rule, p, j),
                    (None, None) => [None, None],
                };
                let negated = exposed
                    .into_iter()
                    .flatten()
                    .filter(|&c| written::negated(forest, self.writing, c));
                premises.push(self.negations(sink, premise, negated, top));
            }
            let written = principal.map_or(Written::new(Side::Output, false), |p| {
                self.writing[p.index()]
            });
            let shown = Shown {
                base: InfId::new(index as u32),
                pending: Vec::new(),
                hidden: Vec::new(),
                rule: ClassicalRule::applied(rule, written),
                premises,
                times: inference.times(),
            };
            ids.push(self.by_side(sink, shown, rule, inference.premises()));
        }
        let root = derivation.root();
        if !derivation.is_of_goal() {
            // The formulas of the sequent written with a negation, as they
            // were written below the last rule.
            let sequent = derivation.inference(root).sequent();
            let roots = sequent.iter().map(|m| m.occ()).filter(|&o| {
                forest.parent(o).is_none() && written::negated(forest, self.writing, o)
            });
            self.negations(sink, root, roots, ids[root.index()]);
        }
    }

    /// Hands `sink` the inference `shown` of the one-sided rule `rule`, and
    /// returns its id: a run of weakenings or contractions of `?` formulas
    /// on both sides of `⊢`, which the one-sided derivation draws as one
    /// inference, as two, the formulas on the input side (`!w`, `!c`)
    /// below those on the output side (`?w`, `?c`), so that every
    /// inference is one two-sided rule.
    /// `below` is the one-sided inference's own premises.
    fn by_side(
        &self,
        sink: &mut impl Sink,
        mut shown: Shown,
        rule: Rule,
        below: &[InfId],
    ) -> InfId {
        let derivation = self.derivation;
        let (Rule::Weakening | Rule::Contraction, &[premise]) = (rule, below) else {
            return sink.take(shown);
        };
        let conclusion = derivation.inference(shown.base).sequent();
        let above = derivation.inference(premise).sequent();
        // What the run takes away or adds, by position in the larger
        // sequent: the weakened formulas, or the copies contracted.
        let (larger, extra) = if rule == Rule::Weakening {
            (shown.base, difference(conclusion, above))
        } else {
            (premise, difference(above, conclusion))
        };
        let larger_sequent = derivation.inference(larger).sequent();
        let side = |&p: &usize| self.writing[larger_sequent[p].occ().index()].side();
        let (input, output): (Vec<usize>, Vec<usize>) =
            extra.iter().partition(|p| side(p) == Side::Input);
        if input.is_empty() || output.is_empty() {
            return sink.take(shown);
        }
        let named = |side| ClassicalRule::applied(rule, Written::new(side, false));
        let count = |positions: &[usize]| u32::try_from(positions.len()).unwrap_or(u32::MAX);
        // The output side's half above, the input side's below; the upper
        // half shows the larger sequent less what the lower half adds.
        let upper = sink.take(Shown {
            base: larger,
            pending: Vec::new(),
            hidden: if rule == Rule::Weakening {
                input.clone()
            } else {
                output.clone()
            },
            rule: named(Side::Output),
            premises: shown.premises.clone(),
            times: count(&output),
        });
        shown.rule = named(Side::Input);
        shown.premises = vec![upper];
        shown.times = count(&input);
        sink.take(shown)
    }

    /// Hands `sink` a negation step below the inference `top`, which shows
    /// the one-sided inference `base`, for each of `negated` in turn, and
    /// returns the id of the last, or `top` for none: each step's
    /// conclusion shows one formula more as written, negation and all.
    fn negations(
        &self,
        sink: &mut impl Sink,
        base: InfId,
        negated: impl Iterator<Item = OccId>,
        mut top: InfId,
    ) -> InfId {
        let forest = self.derivation.forest();
        let sequent = self.derivation.inference(base).sequent();
        let mut pending = Vec::new();
        for c in negated {
            // A formula the sequent holds twice is either copy.
            let position = sequent.partition_point(|m| m.occ() < c);
            debug_assert_eq!(sequent[position].occ(), c);
            pending.push(position);
            let side = written::context(forest, self.writing, c);
            top = sink.take(Shown {
                base,
                pending: pending.clone(),
                hidden: Vec::new(),
                rule: ClassicalRule::negation(side),
                premises: vec![top],
                times: 1,
            });
        }
        top
    }

    /// Returns the one-sided derivation the view shows.
    pub fn derivation(&self) -> &'a Derivation<'a> {
        self.derivation
    }

    /// Returns the number of inferences.
    pub fn len(&self) -> usize {
        self.inferences.len()
    }

    /// Returns whether the view has no inference, which no view built
    /// from a derivation lacks.
    pub fn is_empty(&self) -> bool {
        self.inferences.is_empty()
    }

    /// Returns the root: the inference that concludes the sequent.
    pub fn root(&self) -> InfId {
        // At least one inference, and no more than `new` counted against
        // `Derivation::MOST`.
        InfId::new(self.inferences.len() as u32 - 1)
    }

    /// Returns the rule of the inference at `id`.
    ///
    /// # Panics
    ///
    /// For an id past the view's end.
    pub fn rule(&self, id: InfId) -> ClassicalRule {
        self.inferences[id.index()].rule
    }

    /// Returns the premises of the inference at `id`, in the rule's order.
    ///
    /// # Panics
    ///
    /// For an id past the view's end.
    pub fn premises(&self, id: InfId) -> &[InfId] {
        &self.inferences[id.index()].premises
    }

    /// Returns the inference of the one-sided derivation whose sequent the
    /// inference at `id` shows: its own, or for a negation step its
    /// premise's.
    ///
    /// # Panics
    ///
    /// For an id past the view's end.
    pub fn shows(&self, id: InfId) -> InfId {
        self.inferences[id.index()].base
    }

    /// Returns the formulas of the inference at `id`'s sequent as it
    /// shows them, in the order of the one-sided sequent: each member,
    /// the side of `⊢` it stands on, and whether it is shown with the
    /// negation it was written with.
    ///
    /// # Panics
    ///
    /// For an id past the view's end.
    pub fn sides(&self, id: InfId) -> impl Iterator<Item = (Member, Side, bool)> + '_ {
        let shown = &self.inferences[id.index()];
        let forest = self.derivation.forest();
        let sequent = self.derivation.inference(shown.base).sequent();
        let shown_at = move |&(position, _): &(usize, &Member)| !shown.hidden.contains(&position);
        sequent
            .iter()
            .enumerate()
            .filter(shown_at)
            .map(move |(position, &m)| {
                if shown.pending.contains(&position) {
                    (m, written::context(forest, self.writing, m.occ()), true)
                } else {
                    (m, self.writing[m.occ().index()].side(), false)
                }
            })
    }

    /// Returns the sequent of the inference at `id`, `Γ ⊢ Δ`, as the text
    /// tree writes it.
    ///
    /// # Panics
    ///
    /// For an id past the view's end.
    pub fn sequent(&self, id: InfId) -> impl Display + '_ {
        Sequent { view: self, id }
    }

    /// Writes the sequent of the inference at `id` in a notation.
    fn write(
        &self,
        out: &mut impl Write,
        notation: &crate::sequents::notation::Notation,
        id: InfId,
        aligned: bool,
        marks: bool,
    ) -> FmtResult {
        let shown = |side: Side| {
            self.sides(id)
                .filter(move |&(_, s, _)| s == side)
                .map(|(m, _, negated)| (m.occ(), negated))
        };
        notation.classical(
            out,
            self.derivation.forest(),
            self.writing,
            shown(Side::Input),
            shown(Side::Output),
            aligned,
            marks,
        )
    }

    /// Returns the width and the height of the text tree the options
    /// draw, as [`Derivation::text_size`] does.
    pub fn text_size(&self, options: &TextOptions) -> (usize, usize) {
        crate::proofs::fmt::text_size(self, options)
    }

    /// Writes the text tree into `out` under `options`, as
    /// [`Derivation::write_text`] does, asking `stop` after each row.
    ///
    /// # Errors
    ///
    /// [`Refusal::Stopped`] when `stop` fired, and [`Error::WriteFailed`]
    /// when `out` refused the text.
    pub fn write_text(
        &self,
        options: &TextOptions,
        out: &mut impl Write,
        stop: impl FnMut(crate::limits::Progress) -> bool,
    ) -> Result<(), Error> {
        crate::proofs::fmt::write_text(
            self,
            options,
            out,
            crate::limits::counting(stop, crate::limits::Phase::Write),
        )
    }
}

impl Drawn for TwoSided<'_> {
    const RULES: usize = ClassicalRule::ALL.len();
    const OPEN: Option<usize> = Some(ClassicalRule::Open as usize);

    fn markup(rule: usize, labels: &Labels) -> Option<&str> {
        let rule = ClassicalRule::ALL[rule];
        // A table names the rules of the one-sided calculus and of
        // intuitionistic linear logic, so these keep their upright labels.
        let label = match labels {
            Labels::Upright | Labels::Table(_) => rule.markup(false),
            Labels::Subscript => rule.markup(true),
            Labels::Off => return None,
        };
        (!label.is_empty()).then_some(label)
    }

    fn len(&self) -> usize {
        self.inferences.len()
    }

    fn root(&self) -> InfId {
        TwoSided::root(self)
    }

    fn premises(&self, id: InfId) -> &[InfId] {
        &self.inferences[id.index()].premises
    }

    fn rule(&self, id: InfId) -> usize {
        self.inferences[id.index()].rule as usize
    }

    fn name(&self, id: InfId) -> &'static str {
        self.inferences[id.index()].rule.name()
    }

    fn times(&self, id: InfId) -> u32 {
        self.inferences[id.index()].times
    }

    fn write_sequent(&self, out: &mut impl Write, id: InfId) -> FmtResult {
        self.write(out, &crate::sequents::notation::TEXT, id, false, false)
    }

    #[cfg(any(feature = "latex", feature = "typst", feature = "svg"))]
    fn sequent(
        &self,
        out: &mut String,
        notation: &crate::sequents::notation::Notation,
        id: InfId,
        aligned: bool,
        marks: bool,
    ) {
        // Writing into a `String` does not fail.
        self.write(out, notation, id, aligned, marks).unwrap();
    }

    #[cfg(feature = "svg")]
    fn positions(&self, id: InfId) -> Vec<usize> {
        let sides: Vec<Side> = self.sides(id).map(|(_, side, _)| side).collect();
        let (left, right): (Vec<usize>, Vec<usize>) =
            (0..sides.len()).partition(|&p| sides[p] == Side::Input);
        left.into_iter().chain(right).collect()
    }
}

/// The sequent of an inference of a view, as a value that writes it.
struct Sequent<'v, 'a> {
    /// The view.
    view: &'v TwoSided<'a>,
    /// The inference.
    id: InfId,
}

impl Display for Sequent<'_, '_> {
    /// Writes the sequent in Unicode.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let notation = &crate::sequents::notation::TEXT;
        self.view.write(f, notation, self.id, false, false)
    }
}

impl Display for TwoSided<'_> {
    /// Draws the view as a tree of sequents under the default
    /// [`TextOptions`], one line per row, without a trailing newline.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.write_text(&TextOptions::default(), f, |_| false)
            .map_err(|_| std::fmt::Error)
    }
}

/// Returns the positions in `larger` of the members `smaller` lacks, both
/// in ascending order and `smaller` a sub-multiset of `larger`.
fn difference(larger: &[Member], smaller: &[Member]) -> Vec<usize> {
    let mut extra = Vec::new();
    let mut k = 0;
    for (position, m) in larger.iter().enumerate() {
        if smaller.get(k) == Some(m) {
            k += 1;
        } else {
            extra.push(position);
        }
    }
    extra
}

/// Returns the subformulas of `p` that the one-sided rule `rule` on it
/// puts into its premise `j`: the left and the right one of a `⅋`, one
/// factor of a `⊗` or `&` each, the operand of a `⊕` rule, `!` and `?d`,
/// and none for every other rule.
fn exposed(forest: &Forest, rule: Rule, p: OccId, j: usize) -> [Option<OccId>; 2] {
    match (rule, j) {
        // The view refuses a sequent with binders.
        (Rule::Forall | Rule::Exists, _) => unreachable!("no binder in the classical view"),
        (Rule::Par, _) => [forest.left(p), forest.right(p)],
        (Rule::Tensor | Rule::With, 1) | (Rule::PlusRight, _) => [forest.right(p), None],
        (Rule::Tensor | Rule::With | Rule::PlusLeft | Rule::Promotion | Rule::Dereliction, _) => {
            [forest.left(p), None]
        }
        (
            Rule::Ax
            | Rule::One
            | Rule::Bot
            | Rule::Top
            | Rule::Contraction
            | Rule::Weakening
            | Rule::Mix
            | Rule::AffineWeakening
            | Rule::Cut
            | Rule::Open,
            _,
        ) => [None, None],
    }
}

/// What the walk over a view's inferences hands them to.
trait Sink {
    /// Takes the next inference and returns its id.
    fn take(&mut self, shown: Shown) -> InfId;
}

impl Sink for Vec<Shown> {
    fn take(&mut self, shown: Shown) -> InfId {
        self.push(shown);
        // As many as the counting pass allowed, fewer than `u32::MAX`.
        InfId::new(self.len() as u32 - 1)
    }
}

/// The counting pass: the inferences and the bytes of the view by the
/// derivation size's estimate ([`Size::bytes`]), every sum saturating.
struct Counted<'d> {
    /// The derivation the view shows.
    derivation: &'d Derivation<'d>,
    /// The characters of every occurrence's formula and its separator.
    weights: Vec<u32>,
    /// The characters of each one-sided inference's sequent, by its id,
    /// once computed: a chain of negation steps shows one sequent again.
    characters: Vec<Option<u64>>,
    /// The inferences so far.
    inferences: u64,
    /// The estimate so far.
    bytes: u64,
}

impl<'d> Counted<'d> {
    /// Returns a count of nothing yet, of a view of `derivation`.
    fn new(derivation: &'d Derivation<'d>) -> Self {
        Self {
            derivation,
            weights: crate::proofs::size::weights(derivation.forest()),
            characters: vec![None; derivation.inferences().len()],
            inferences: 0,
            bytes: 0,
        }
    }
}

impl Sink for Counted<'_> {
    fn take(&mut self, shown: Shown) -> InfId {
        let characters = *self.characters[shown.base.index()].get_or_insert_with(|| {
            let sequent = self.derivation.inference(shown.base).sequent();
            sequent.iter().fold(0u64, |sum, m| {
                sum.saturating_add(self.weights[m.occ().index()].into())
            })
        });
        self.inferences = self.inferences.saturating_add(1);
        self.bytes = self.bytes.saturating_add(
            characters
                .saturating_mul(Size::BYTES_PER_CHARACTER)
                .saturating_add(Size::BYTES_PER_INFERENCE),
        );
        // The id is a placeholder the count never reads back.
        InfId::new(u32::try_from(self.inferences).unwrap_or(u32::MAX))
    }
}

#[cfg(all(test, feature = "parse"))]
mod tests {
    use super::*;
    use crate::occurrences::ClassicalSequent;
    use crate::proofs::{Compact, ViewOptions};
    use crate::search::generate::{self, Rng, Rules};
    use crate::{Mode, Options, Proof};

    /// Returns a proof of a written sequent in `mode`.
    fn proved(written: &ClassicalSequent, mode: Mode) -> Proof {
        let outcome = crate::prove(written.sequent(), mode, &Options::default()).unwrap();
        outcome.verdict.proof().expect("provable").clone()
    }

    /// Returns a proof with a cut of a written sequent in `mode` and the
    /// record of its session: the sequent's session cut on one of its
    /// formulas, typed as it was written (the others going to the first
    /// premise, or the formula alone where it stands left of `⊢`), both
    /// goals closed by the search; `None` where the search leaves one.
    #[cfg(feature = "interactive")]
    fn cut_written(
        written: &ClassicalSequent,
        mode: Mode,
        rng: &mut Rng,
    ) -> Option<(Proof, Vec<Written>)> {
        use crate::{CutFormula, InputOptions, Interactive};
        let limits = Limits::default();
        let mut session = Interactive::of_classical(written, mode, &limits).unwrap();
        let root = session.goals().next().unwrap();
        let roots = written.forest().roots();
        let p = rng.below(roots.len());
        let text = written.formula(roots[p]).to_string();
        let antecedents = written.sequent().antecedents().unwrap_or(0) as usize;
        let left: Vec<usize> = if p < antecedents {
            vec![p]
        } else {
            (0..roots.len()).filter(|&q| q != p).collect()
        };
        let formula =
            CutFormula::parse_within(&text, mode, &InputOptions::default(), &limits).unwrap();
        session.cut(root, &formula, &left, &limits).unwrap();
        let options = Options::default();
        let view = ViewOptions::default();
        for (_, closed) in session.close_all(&options, &view, &limits, |_| false) {
            closed.ok()?.grafted.ok()?;
        }
        let proof = session.proof(&limits, |_| false).ok()?;
        Some((proof, session.writing().unwrap().to_vec()))
    }

    /// Returns no proof with a cut: the sessions that make one need the
    /// feature `interactive`.
    #[cfg(not(feature = "interactive"))]
    fn cut_written(_: &ClassicalSequent, _: Mode, _: &mut Rng) -> Option<(Proof, Vec<Written>)> {
        None
    }

    /// Returns the two-sided tree of a sequent's proof in classical mode,
    /// every run of a structural rule drawn as one inference.
    fn drawn(text: &str) -> String {
        let written: ClassicalSequent = text.parse().unwrap();
        let proof = proved(&written, Mode::CLASSICAL);
        let options = ViewOptions::default().with_compact(Compact::Always);
        let limits = Limits::default();
        let derivation = proof
            .derivation_within(&options, &limits, |_| false)
            .unwrap();
        TwoSided::new(&derivation, written.writing(), &limits)
            .unwrap()
            .to_string()
    }

    /// Rules are named by the side and the writing of their principal
    /// formula, and a written negation is a step of its own, right above
    /// the rule that exposes the formula or below the last rule for one
    /// of the sequent.
    #[test]
    fn rules_by_side_and_negation_steps() {
        let cases = [
            (
                "A -o B, A |- B",
                "───── ax   ───── ax\n\
                 A ⊢ A      B ⊢ B\n\
                 ──────────────── ⊸L\n  \
                 A ⊸ B, A ⊢ B",
            ),
            (
                "|- ~(A * B), A * B",
                "───── ax   ───── ax\n\
                 A ⊢ A      B ⊢ B\n\
                 ──────────────── ⊗R\n  \
                 A, B ⊢ A ⊗ B\n  \
                 ───────────── ⊗L\n  \
                 A ⊗ B ⊢ A ⊗ B\n\
                 ───────────────── ~R\n\
                 ⊢ ~(A ⊗ B), A ⊗ B",
            ),
            (
                "A par B |- ~(~A * ~B)",
                "───── ax   ───── ax\n\
                 A ⊢ A      B ⊢ B\n\
                 ──────────────── ⅋L\n  \
                 A ⅋ B ⊢ A, B\n  \
                 ───────────── ~L\n  \
                 A ⅋ B, ~A ⊢ B\n \
                 ─────────────── ~L\n \
                 A ⅋ B, ~A, ~B ⊢\n \
                 ──────────────── ⊗L\n \
                 A ⅋ B, ~A ⊗ ~B ⊢\n\
                 ────────────────── ~R\n\
                 A ⅋ B ⊢ ~(~A ⊗ ~B)",
            ),
            (
                // A run of weakenings on both sides, drawn compact, is
                // split by side.
                "!b, a |- a, ?c",
                "    ───── ax\n    \
                 a ⊢ a\n  \
                 ───────── ?w\n  \
                 a ⊢ a, ?c\n\
                 ───────────── !w\n\
                 !b, a ⊢ a, ?c",
            ),
            (
                "!A |- ?(A + B) * 1",
                "   ───── ax\n   \
                 A ⊢ A\n \
                 ───────── ⊕R₁\n \
                 A ⊢ A ⊕ B\n\
                 ──────────── ?R\n\
                 A ⊢ ?(A ⊕ B)\n\
                 ───────────── !L   ─── 1R\n\
                 !A ⊢ ?(A ⊕ B)      ⊢ 1\n\
                 ────────────────────── ⊗R\n  \
                 !A ⊢ ?(A ⊕ B) ⊗ 1",
            ),
        ];
        for (text, tree) in cases {
            assert_eq!(drawn(text), tree, "{text}");
        }
    }

    /// A view estimated past the bound on derivations is refused, with
    /// nothing built; the derivation it would show is within it.
    #[test]
    fn a_view_past_the_bound_is_refused() {
        let written: ClassicalSequent = "A par B |- ~(~A * ~B)".parse().unwrap();
        let proof = proved(&written, Mode::CLASSICAL);
        let derivation = proof.derivation().unwrap();
        let shown = derivation.inferences().len() as u64 * Size::BYTES_PER_INFERENCE;
        let limits = Limits::default().with_derivation_bytes(Some(shown));
        let refused = TwoSided::new(&derivation, written.writing(), &limits).unwrap_err();
        assert!(matches!(
            refused,
            Error::Refused(Refusal::Output { estimate_bytes, limit_bytes, .. })
                if estimate_bytes > limit_bytes && limit_bytes == shown
        ));
    }

    /// Returns the formulas of an inference of a view as it shows them,
    /// those left of `⊢` first, each in the order of the one-sided
    /// sequent.
    fn shown(view: &TwoSided<'_>, id: InfId) -> Vec<(Member, Side, bool)> {
        let all: Vec<_> = view.sides(id).collect();
        let on = |side| all.iter().copied().filter(move |&(_, s, _)| s == side);
        on(Side::Input).chain(on(Side::Output)).collect()
    }

    /// Over the proofs of generated sequents written at random, every
    /// inference of the two-sided view prints as a sequent that parses
    /// back to the same: its formulas on the same sides, each written as
    /// the sequent's own occurrence was, with the negation shown where
    /// the view shows one. Read one-sided, each is the inference it shows:
    /// a rule of the term with its principal formula as written on the
    /// side its name says, or a negation step that moves one formula,
    /// written with its negation in the conclusion, without it to the
    /// other side in the premise. The root is the sequent as written. The
    /// same holds of a proof with a cut on a formula typed as written,
    /// whose two occurrences the session records on their sides.
    #[test]
    fn every_inference_parses_back_and_reads_as_its_term() {
        let limits = Limits::default();
        let (mut inferences, mut negations, mut split, mut cuts) = (0, 0, 0, 0);
        let mut cut_rng = Rng::new(41);
        for (i, rules) in Rules::ALL.into_iter().enumerate() {
            let mode = if rules.mix {
                Mode::CLASSICAL.with_mix()
            } else {
                Mode::CLASSICAL
            };
            let mut rng = Rng::new(400 + i as u64);
            for _ in 0..10 {
                let budget = 2 + rng.below(10);
                let provable = generate::provable(&mut rng, rules, 3, budget);
                let (text, _) = generate::written(&mut rng, &provable.formulas);
                let written: ClassicalSequent = text.parse().unwrap();
                let plain = (proved(&written, mode), written.writing().to_vec());
                let cut = cut_written(&written, mode, &mut cut_rng);
                for (proof, writing) in [Some(plain), cut].into_iter().flatten() {
                    for compact in [Compact::Never, Compact::Always] {
                        let options = ViewOptions::default().with_compact(compact);
                        let derivation = proof
                            .derivation_within(&options, &limits, |_| false)
                            .unwrap();
                        let view = TwoSided::new(&derivation, &writing, &limits).unwrap();
                        let forest = derivation.forest();
                        let root = view.root();
                        assert_eq!(
                            view.sequent(root).to_string(),
                            written.to_string(),
                            "{text}"
                        );
                        for k in 0..view.len() {
                            let id = InfId::new(k as u32);
                            let printed = view.sequent(id).to_string();
                            let parsed: ClassicalSequent = printed.parse().unwrap();
                            let members = shown(&view, id);
                            let left = members.iter().filter(|m| m.1 == Side::Input).count();
                            let (pf, pw) = (parsed.forest(), parsed.writing());
                            assert_eq!(
                                parsed.sequent().antecedents(),
                                Some(left as u32),
                                "{printed}"
                            );
                            assert_eq!(pf.roots().len(), members.len(), "{printed}");
                            for (&r, &(m, _, negated)) in pf.roots().iter().zip(&members) {
                                let o = m.occ();
                                assert_eq!(pf.size(r), forest.size(o), "{printed}");
                                for d in 0..forest.size(o) {
                                    let (p, q) = (OccId::new(r.get() + d), OccId::new(o.get() + d));
                                    assert_eq!(pf.kind(p), forest.kind(q), "{printed}");
                                    assert_eq!(pw[p.index()], writing[q.index()], "{printed}");
                                    let name = |f: &Forest, x| {
                                        f.atom(x).map(|a| f.sequent().atom_name(a).to_owned())
                                    };
                                    assert_eq!(name(pf, p), name(forest, q), "{printed}");
                                }
                                assert_eq!(parsed.is_negated(r), negated, "{printed}");
                            }
                            let base = derivation.inference(view.shows(id));
                            let rule = view.rule(id);
                            match rule.rule() {
                                None => {
                                    let &[premise] = view.premises(id) else {
                                        panic!("a negation step has one premise");
                                    };
                                    assert_eq!(view.shows(premise), view.shows(id));
                                    let above: Vec<_> = view.sides(premise).collect();
                                    let below: Vec<_> = view.sides(id).collect();
                                    let moved: Vec<_> = (0..below.len())
                                        .filter(|&p| above[p] != below[p])
                                        .collect();
                                    let &[p] = &moved[..] else {
                                        panic!("a negation step moves one formula: {printed}");
                                    };
                                    let (m, side, negated) = below[p];
                                    assert!(negated && !above[p].2, "{printed}");
                                    assert_eq!(above[p].1, writing[m.occ().index()].side());
                                    assert_eq!(Some(side), rule.side(), "{printed}");
                                    assert_ne!(side, above[p].1, "{printed}");
                                    negations += 1;
                                }
                                Some(one_sided) if one_sided.is_structural() => {
                                    // A run: what it adds or takes away stands on
                                    // its name's side, as many as it counts.
                                    assert_eq!(one_sided, base.rule().rule, "{printed}");
                                    let &[premise] = view.premises(id) else {
                                        panic!("a structural rule has one premise");
                                    };
                                    let key =
                                        |s: &(Member, Side, bool)| (s.0, s.1 == Side::Input, s.2);
                                    let mut below: Vec<_> =
                                        view.sides(id).map(|s| key(&s)).collect();
                                    let mut above: Vec<_> =
                                        view.sides(premise).map(|s| key(&s)).collect();
                                    below.sort_unstable();
                                    above.sort_unstable();
                                    let (larger, smaller) = if one_sided == Rule::Contraction {
                                        (&above, &below)
                                    } else {
                                        (&below, &above)
                                    };
                                    let mut moved = larger.clone();
                                    for s in smaller {
                                        let at = moved
                                            .iter()
                                            .position(|m| m == s)
                                            .expect("a sub-multiset");
                                        moved.remove(at);
                                    }
                                    assert_eq!(
                                        moved.len() as u32,
                                        view.inferences[k].times,
                                        "{printed}"
                                    );
                                    if let Some(named) = rule.side() {
                                        let input = named == Side::Input;
                                        assert!(
                                            moved.iter().all(|m| m.1 == input),
                                            "{printed} by {rule}"
                                        );
                                    }
                                }
                                Some(one_sided) => {
                                    assert_eq!(one_sided, base.rule().rule, "{printed}");
                                    assert_eq!(view.premises(id).len(), base.premises().len());
                                    if let Some(position) = base.principal() {
                                        let (m, side, negated) =
                                            view.sides(id).nth(position).unwrap();
                                        assert!(!negated, "a rule acts on a formula as written");
                                        if let Some(named) = rule.side() {
                                            assert_eq!(side, named, "{printed} by {rule}");
                                        }
                                        let implication = writing[m.occ().index()].is_implication();
                                        let lollipop = matches!(
                                            rule,
                                            ClassicalRule::LollipopLeft
                                                | ClassicalRule::LollipopRight
                                        );
                                        assert_eq!(implication, lollipop, "{printed} by {rule}");
                                    }
                                }
                            }
                            split += usize::from(!view.inferences[k].hidden.is_empty());
                            cuts += usize::from(rule == ClassicalRule::Cut);
                            inferences += 1;
                        }
                    }
                }
            }
        }
        assert!(
            inferences > 1000 && negations > 100 && split > 0,
            "{inferences} {negations} {split}"
        );
        #[cfg(feature = "interactive")]
        assert!(cuts > 50, "{cuts} cuts");
    }
}
