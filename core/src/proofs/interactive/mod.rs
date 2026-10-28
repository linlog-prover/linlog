// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Step-by-step proving: a derivation of the standard sequent calculus
//! built top-down, with open goals as leaves, over the occurrence forest of
//! a sequent. A client names a goal, a formula in it and a rule; the rule
//! is validated against the formula, the mode and, in intuitionistic mode,
//! the one-succedent condition, and the goals it leaves are returned. The
//! search can close any goal, and a finished derivation translates back
//! into a proof term that the checker validates, so this layer is trusted
//! no more than an engine.
//!
//! The inferences are those of the derivation view, with the same sequent
//! representation (occurrence ids in ascending order, with repeats) and
//! the same rule names; an open goal is an inference with [`Rule::Open`]
//! and no premises. The arena is top-down: the root is inference 0 and a
//! step appends the goals it opens at the end, so a premise has a larger
//! index than its conclusion, the reverse of a [`Derivation`], which
//! [`Interactive::derivation`] renumbers.
//!
//! Needs the cargo feature `interactive` (on by default).

// A rule a later step adds must not fall into an existing arm.
#![deny(clippy::wildcard_enum_match_arm)]

mod cut;
mod expand;
mod invert;
#[cfg(feature = "serialize")]
mod replay;
mod terms;

pub use cut::CutFormula;
#[cfg(all(test, feature = "parse"))]
pub(crate) use cut::with_cuts;
pub use invert::{Inverted, Options};

use crate::Error;
use crate::fragment::Mode;
use crate::limits::{Limits, Phase, Progress, Refusal};
use crate::occurrences::{ClassicalSequent, Forest, Member, OccId, Reading, Side, Written};
use crate::proofs::derivation::{Derivation, InfId, Inference, ViewOptions};
use crate::proofs::size::Size;
use crate::proofs::{Named, Rule};
use crate::search::Outcome;
use crate::sequents::Sequent;
use std::fmt::{Display, Formatter, Result as FmtResult};

/// Why a rule does not apply to a goal as asked. Positions are those in the
/// goal's sequent, as the client named them.
///
/// Needs the cargo feature `interactive` (on by default).
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialize",
    derive(serde::Serialize),
    serde(tag = "kind", rename_all = "snake_case")
)]
#[non_exhaustive]
pub enum StepError {
    /// No open goal has this id.
    #[non_exhaustive]
    NoGoal {
        /// The id asked for.
        goal: GoalId,
    },
    /// The goal has no formula at the position; it has `len` formulas.
    #[non_exhaustive]
    NoFormula {
        /// The position asked for.
        position: usize,
        /// How many formulas the goal has.
        len: usize,
    },
    /// The rule does not act on the formula at the position: its connective
    /// is another, or in intuitionistic mode the rule is for the other
    /// side of `⊢`.
    #[non_exhaustive]
    Rule {
        /// The rule asked for.
        rule: Named,
        /// The position of the formula.
        position: usize,
    },
    /// The rule is not available in the mode: weakening of a formula that
    /// is not a `?` formula needs affine mode, Mix needs Mix.
    #[non_exhaustive]
    Mode {
        /// The rule asked for.
        rule: Named,
        /// The mode of the proof.
        mode: Mode,
    },
    /// The rule needs the formula alone in the goal (`1`), or alone with its
    /// dual (`ax`); in affine mode the other formulas can be weakened first.
    #[non_exhaustive]
    NotAlone {
        /// The rule asked for.
        rule: Named,
        /// The position of the formula.
        position: usize,
    },
    /// The axiom needs the goal to be the literal and its dual, and the
    /// other formula is not the dual.
    #[non_exhaustive]
    NoDual {
        /// The position of the literal.
        position: usize,
    },
    /// Promotion needs every other formula to be a `?` formula, and the one
    /// at the position is not.
    #[non_exhaustive]
    NotQuest {
        /// The position of the offending formula.
        position: usize,
    },
    /// A position of the split is out of range, repeated, or the formula the
    /// rule acts on.
    #[non_exhaustive]
    Split {
        /// The offending position.
        position: usize,
    },
    /// A split was given to a rule that takes none.
    #[non_exhaustive]
    NoSplit {
        /// The rule asked for.
        rule: Named,
    },
    /// In intuitionistic mode, a premise would have this many formulas on
    /// the right of `⊢` instead of one.
    #[non_exhaustive]
    Succedents {
        /// How many formulas would stand right of `⊢`.
        count: usize,
    },
    /// In intuitionistic mode, the formula on the right of `⊢` cannot be
    /// weakened.
    #[non_exhaustive]
    Output {
        /// The position of the formula.
        position: usize,
    },
    /// A Mix whose split leaves a premise without a formula: no rule
    /// concludes the empty sequent, so nothing could close it.
    EmptyPremise,
    /// In an ordered mode, the split is not the two arcs on either side of
    /// the `⊗`: going round from it, the formulas going right must come
    /// first and those going left after them, and any other split needs
    /// an exchange.
    #[non_exhaustive]
    Exchange {
        /// The position of a formula going left that comes before one
        /// going right.
        position: usize,
    },
    /// In the Lambek calculus without empty antecedents, a premise would
    /// have no formula left of `⊢`, which no proof of the calculus has.
    EmptyAntecedent,
}

impl Display for StepError {
    /// Writes the refusal as a phrase, naming rules and positions.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            StepError::NoGoal { goal: id } => write!(f, "there is no open goal {}", id.get()),
            StepError::NoFormula { position, len } => {
                write!(
                    f,
                    "the goal has no formula {position}: it has {}",
                    crate::errors::counted(*len, "formula", "formulas")
                )
            }
            StepError::Rule { rule, position } => {
                write!(f, "the rule {rule} does not act on formula {position}")
            }
            StepError::Mode { rule, mode } => {
                write!(f, "the rule {rule} is not available in {mode} mode")
            }
            StepError::NotAlone { rule, position } => write!(
                f,
                "the rule {rule} needs formula {position} without other formulas in the goal"
            ),
            StepError::NoDual { position } => write!(
                f,
                "the axiom needs formula {position} together with its dual literal and nothing else"
            ),
            StepError::NotQuest { position } => write!(
                f,
                "promotion needs every other formula to be a ? formula, and formula {position} is not"
            ),
            StepError::Split { position } => write!(
                f,
                "position {position} of the split is out of range, repeated, or the formula the rule acts on"
            ),
            StepError::NoSplit { rule } => write!(f, "the rule {rule} takes no split"),
            StepError::Succedents { count: n } => write!(
                f,
                "a premise would have {n} formulas on the right of ⊢ instead of one"
            ),
            StepError::Output { position } => write!(
                f,
                "formula {position} is the one on the right of ⊢ and cannot be weakened"
            ),
            StepError::EmptyPremise => {
                f.write_str("the split leaves a premise without a formula, which no rule concludes")
            }
            StepError::EmptyAntecedent => f.write_str(
                "a premise would have no formula left of ⊢, which the Lambek calculus without \
                 empty antecedents forbids",
            ),
            StepError::Exchange { position } => write!(
                f,
                "formula {position} goes left before a formula going right, which needs an \
                 exchange: the left premise takes the formulas just before the ⊗, going round"
            ),
        }
    }
}

impl std::error::Error for StepError {}

/// The number of an inference of a session, in the session's own order:
/// the root `0`, and every step's premises after it. It names the goals
/// that [`Interactive::apply`], [`close`](Interactive::close) and
/// [`undo`](Interactive::undo) take and return; a derivation of the
/// session numbers its inferences otherwise ([`InfId`]), and
/// [`Interactive::derivation_ids`] maps the one to the other.
///
/// Needs the cargo feature `interactive` (on by default).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize), serde(transparent))]
pub struct GoalId(u32);

impl GoalId {
    /// Wraps a raw goal number.
    pub const fn new(raw: u32) -> Self {
        Self(raw)
    }

    /// Returns the raw goal number.
    pub const fn get(self) -> u32 {
        self.0
    }

    /// Returns the number as a `usize`.
    pub const fn index(self) -> usize {
        self.0 as usize
    }

    /// Returns the inference of the session's arena the goal is.
    const fn inf(self) -> InfId {
        InfId::new(self.0)
    }

    /// Returns the goal an inference of the session's arena is.
    const fn of(id: InfId) -> Self {
        Self(id.get())
    }
}

/// A step of a session: the rule applied to the formula at a position of
/// a goal, and the split a `⊗` or Mix needs.
///
/// Needs the cargo feature `interactive` (on by default).
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Step {
    /// The position of the formula in the goal's sequent.
    pub position: usize,
    /// The rule, by its classical name or, in intuitionistic mode, by its
    /// two-sided one.
    pub rule: Named,
    /// How the context is split between two premises.
    pub split: Split,
}

impl Step {
    /// Returns the step of `rule` on the formula at `position`, without a
    /// split.
    pub fn new(position: usize, rule: impl Into<Named>) -> Self {
        Self {
            position,
            rule: rule.into(),
            split: Split::None,
        }
    }

    /// Returns the step with the formulas at `positions` sent to the left
    /// premise of a `⊗` or Mix, the rest to the right; for Mix the formula
    /// the step names goes left too.
    #[must_use]
    pub fn left(self, positions: &[usize]) -> Self {
        Self {
            split: Split::Left {
                positions: positions.to_vec(),
            },
            ..self
        }
    }

    /// Returns the step with the context split at a cut point of the
    /// goal's cyclic order, as an ordered mode reads it: going round from
    /// the formula the step names, the formulas before `position` go to
    /// the right premise and those from `position` on to the left one, so
    /// that `position` is the first formula going left, and the step's own
    /// position sends none left. In the cyclic mode these are the splits
    /// of a `⊗`; in a commutative one it is a split as any other.
    #[must_use]
    pub fn at(self, position: usize) -> Self {
        Self {
            split: Split::At { position },
            ..self
        }
    }

    /// Returns the positions going left in a goal of `len` formulas, none
    /// without a split, or the refusal of a cut point outside the goal.
    fn left_positions(&self, len: usize) -> Result<Vec<usize>, StepError> {
        match &self.split {
            Split::None => Ok(Vec::new()),
            Split::Left { positions } => Ok(positions.clone()),
            &Split::At { position } if position >= len => Err(StepError::Split { position }),
            // From the cut point round to the step's formula.
            &Split::At { position } => Ok((position..len)
                .chain(0..position)
                .take_while(|&p| p != self.position)
                .collect()),
        }
    }
}

/// How a step splits its goal's context between two premises.
///
/// Needs the cargo feature `interactive` (on by default).
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Split {
    /// No split: the rule has one premise or none.
    #[default]
    None,
    /// The formulas at these positions go to the left premise, the rest to
    /// the right.
    #[non_exhaustive]
    Left {
        /// The positions in the goal's sequent.
        positions: Vec<usize>,
    },
    /// The goal's formulas from this position round to the step's formula
    /// go to the left premise, the others to the right ([`Step::at`]).
    #[non_exhaustive]
    At {
        /// The position of the first formula going left.
        position: usize,
    },
}

/// A rule that can act on a formula of a goal, and what a step of it needs
/// beyond the formula's position.
///
/// Needs the cargo feature `interactive` (on by default).
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Applicable {
    /// The rule, named as the session records it.
    pub rule: Named,
    /// What the step needs.
    pub needs: Needs,
}

/// What [`Interactive::close`] did: the search's outcome, and whether the
/// derivation of the proof it found, if any, was grafted onto the goal.
///
/// Needs the cargo feature `interactive` (on by default).
#[non_exhaustive]
#[derive(Clone, Debug)]
pub struct Closed {
    /// The search's outcome; a proof in it is the proof of the goal
    /// alone, kept for export when its graft was refused.
    pub outcome: Outcome,
    /// `Ok` when the proof was grafted or there was none to graft; the
    /// refusal of the graft, by its size or the stop, otherwise, the goal
    /// then still open.
    pub grafted: Result<(), Refusal>,
}

/// What a step of a rule needs beyond the formula's position.
///
/// Needs the cargo feature `interactive` (on by default).
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Needs {
    /// Nothing.
    Nothing,
    /// A split of the context between the two premises: `⊗` and Mix.
    Split,
}

/// A proof in progress: the forest of a sequent, the mode, the inferences
/// made so far and the goals still open. It starts from a sequent as one
/// open goal; rules are applied to a formula of a goal by position, the
/// search closes goals, [`undo`](Self::undo) retracts the last step (or
/// the client keeps a clone), and once no goal is open
/// [`proof`](Self::proof) gives the checked term.
///
/// # Examples
///
#[cfg_attr(feature = "parse", doc = "```")]
#[cfg_attr(not(feature = "parse"), doc = "```ignore")]
/// use linlog::proofs::interactive::Needs;
/// use linlog::{Interactive, Limits, Mode, Named, Rule, Sequent, Step};
///
/// // ⊢ ~A, A ⊗ ~B, B: the goal's formulas are 0: ~A, 1: A ⊗ ~B, 2: B.
/// let sequent: Sequent = "A, A -o B |- B".parse()?;
/// let mut state = Interactive::new(&sequent, Mode::CLASSICAL)?;
/// let root = state.goals().next().unwrap();
/// let rules = state.rules(root, 1)?;
/// assert_eq!((rules[0].rule, rules[0].needs), (Named::from(Rule::Tensor), Needs::Split));
/// // ⊗ on formula 1, with formula 0 going to the left premise.
/// let goals = state.apply(root, &Step::new(1, Rule::Tensor).left(&[0]))?;
/// assert_eq!(goals.len(), 2);
/// state.apply(goals[0], &Step::new(0, Rule::Ax))?;
/// state.apply(goals[1], &Step::new(0, Rule::Ax))?;
/// let proof = state.proof(&Limits::default(), |_| false)?;
/// assert_eq!(proof.nodes().len(), 3);
/// # Ok::<(), linlog::Error>(())
/// ```
///
/// # JSON
///
/// With the feature `serialize` a session is `{"version": 1, "sequent":
/// …, "mode": "intuitionistic", "inferences": […], "history": […]}`
/// ([`wire`](crate::wire)), the mode by its name: the inferences in the
/// state's own
/// order, the conclusion first, each `{"sequent": [ids], "rule": name,
/// "principal": position, "premises": [indices]}` with the rule's
/// [`name`](Rule::name), an open goal as its sequent alone; and the
/// history as the inferences the steps closed, in order. A session that
/// keeps how its sequent was written ([`writing`](Self::writing)) has the
/// key `"written": {"inputs": [ids], "implications": [ids]}` too: the
/// occurrences written on the input side and those written as an
/// implication, ascending. Reading it back
/// replays every inference, so a state read is as sound as one built
/// through [`apply`](Self::apply). It is what a front end keeps between
/// requests, and what the command's session saves.
///
/// Needs the cargo feature `interactive` (on by default).
#[derive(Clone, Debug)]
pub struct Interactive {
    /// The forest of the sequent being proved.
    forest: Forest,
    /// The mode.
    mode: Mode,
    /// In intuitionistic mode, the reading's positions and goal, kept so
    /// that [`reading`](Self::reading) costs nothing.
    reading: Option<(Box<[Side]>, OccId)>,
    /// How the sequent was written, for a session started from a
    /// classical sequent as written.
    writing: Option<Box<[Written]>>,
    /// The inferences, top-down: the root first, then the goals each step
    /// opened, in order; an open goal has [`Rule::Open`].
    inferences: Vec<Inference>,
    /// The goals closed so far, in order, for undo.
    history: Vec<InfId>,
}

impl Interactive {
    /// Starts a proof of the sequent in the mode, with the whole sequent as
    /// the one open goal. Fails in intuitionistic mode for a sequent with no
    /// intuitionistic reading or with Mix.
    ///
    /// # Errors
    ///
    /// [`Error::NotIntuitionistic`] in intuitionistic mode for a sequent
    /// without an intuitionistic reading, [`Error::IntuitionisticMix`] for
    /// intuitionistic mode with Mix, [`Error::OrderedMode`] for a mode
    /// [`Mode::check`] refuses otherwise, [`Error::OrderedFragment`] in an
    /// ordered mode for a sequent beyond unit-free MLL, and
    /// [`Refusal::Occurrences`] past the default
    /// bound.
    pub fn new(sequent: &Sequent, mode: Mode) -> Result<Self, Error> {
        Self::within(sequent, mode, &Limits::default())
    }

    /// Starts a proof as [`new`](Self::new) does, of a sequent that
    /// unfolds to at most `limits.occurrences` occurrences
    /// ([`Refusal::Occurrences`] otherwise).
    ///
    /// # Errors
    ///
    /// As [`new`](Self::new) does, the bound being `limits.occurrences`.
    pub fn within(sequent: &Sequent, mode: Mode, limits: &Limits) -> Result<Self, Error> {
        let forest = Forest::within(sequent, limits)?;
        Self::from_forest(forest, mode)
    }

    /// Starts a proof over a forest built already.
    fn from_forest(forest: Forest, mode: Mode) -> Result<Self, Error> {
        mode.check()?;
        let fragment = forest.sequent().fragment();
        if mode.is_ordered() && !crate::Fragment::MLL.contains(fragment) {
            return Err(Error::OrderedFragment { fragment, mode });
        }
        let reading = if mode.intuitionistic {
            let reading = Reading::of_mode(&forest, mode).map_err(Error::NotIntuitionistic)?;
            Some(reading.into_parts())
        } else {
            None
        };
        let root = Inference {
            sequent: forest.roots().iter().copied().map(Member::from).collect(),
            rule: Rule::Open.into(),
            principal: None,
            premises: vec![],
            times: 1,
            cut: None,
            node: None,
        };
        Ok(Self {
            forest,
            mode,
            reading,
            writing: None,
            inferences: vec![root],
            history: vec![],
        })
    }

    /// Starts a proof as [`within`](Self::within) does of a classical
    /// sequent as it was written, and keeps how it was written
    /// ([`writing`](Self::writing)), so that its derivation can be drawn
    /// two-sided ([`TwoSided`](crate::TwoSided)) at any point of the
    /// proof.
    ///
    /// # Errors
    ///
    /// As [`within`](Self::within) does.
    pub fn of_classical(
        sequent: &ClassicalSequent,
        mode: Mode,
        limits: &Limits,
    ) -> Result<Self, Error> {
        let mut state = Self::within(sequent.sequent(), mode, limits)?;
        state.writing = Some(sequent.writing().into());
        Ok(state)
    }

    /// Returns how the sequent was written, by occurrence, for a session
    /// started from a classical sequent as written, and `None` otherwise.
    pub fn writing(&self) -> Option<&[Written]> {
        self.writing.as_deref()
    }

    /// Keeps how the sequent was written, as a session read back carries
    /// it.
    #[cfg(feature = "serialize")]
    pub(crate) fn keep_writing(&mut self, writing: Box<[Written]>) -> Result<(), Error> {
        if let Some(reason) = crate::occurrences::written::fault(&self.forest, &writing) {
            return Err(Error::InconsistentSession { reason });
        }
        self.writing = Some(writing);
        Ok(())
    }

    /// Returns the forest of the sequent being proved.
    pub fn forest(&self) -> &Forest {
        &self.forest
    }

    /// Returns the occurrence a member of the session stands for.
    pub fn occurrence(&self, member: Member) -> OccId {
        member.occ()
    }

    /// Returns the formula a member of the session stands for: in an
    /// intuitionistic mode as the session's reading writes it, two-sided
    /// (`A ⊸ B` for a hypothesis the one-sided sequent holds as `A ⊗
    /// ~B`), else as the one-sided sequent writes it.
    pub fn formula(&self, member: Member) -> impl Display + '_ {
        match self.reading() {
            Some(reading) => reading.formula(member.occ()).to_string(),
            None => self.forest.formula(member.occ()).to_string(),
        }
    }

    /// Returns the sequent being proved.
    pub fn sequent(&self) -> &Sequent {
        self.forest.sequent()
    }

    /// Returns the mode.
    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// Returns the intuitionistic reading of the sequent in intuitionistic
    /// mode, which says on which side of `⊢` a client shows each formula,
    /// and `None` in classical mode.
    pub fn reading(&self) -> Option<Reading<'_>> {
        let (position, goal) = self.reading.as_ref()?;
        Some(Reading::of_parts(
            &self.forest,
            position,
            *goal,
            self.mode.is_ordered(),
        ))
    }

    /// Returns every inference, the root first and then in the order the
    /// steps opened them; an open goal is an inference with [`Rule::Open`].
    #[cfg(feature = "serialize")]
    pub(crate) fn inferences(&self) -> &[Inference] {
        &self.inferences
    }

    /// Returns the open goals, in the order they were opened.
    pub fn goals(&self) -> impl Iterator<Item = GoalId> + '_ {
        self.inferences
            .iter()
            .enumerate()
            .filter(|(_, inference)| inference.rule.rule == Rule::Open)
            .map(|(i, _)| GoalId(i as u32))
    }

    /// Returns the sequent of an open goal.
    ///
    /// # Errors
    ///
    /// [`StepError::NoGoal`] if the id is no open goal.
    pub fn goal(&self, id: GoalId) -> Result<&[Member], StepError> {
        self.inferences
            .get(id.index())
            .filter(|inference| inference.rule.rule == Rule::Open)
            .map(|inference| inference.sequent.as_slice())
            .ok_or(StepError::NoGoal { goal: id })
    }

    /// Returns whether no goal is open.
    pub fn is_complete(&self) -> bool {
        self.goals().next().is_none()
    }

    /// Returns how many steps were taken: rules applied and goals closed by
    /// the search, less those undone.
    pub fn steps(&self) -> usize {
        self.history.len()
    }

    /// Returns the goals the steps closed, in order.
    #[cfg(feature = "serialize")]
    pub(crate) fn history(&self) -> &[InfId] {
        &self.history
    }

    /// Returns the derivation so far, with the open goals as leaves of
    /// [`Rule::Open`], two-sided in intuitionistic mode; its inferences are
    /// renumbered premises before conclusions, so its ids are not this
    /// state's: [`derivation_ids`](Self::derivation_ids) maps them.
    ///
    /// # Errors
    ///
    /// [`Refusal::Output`] for a derivation past the
    /// default [`Limits::derivation_bytes`].
    pub fn derivation(&self) -> Result<Derivation<'_>, Error> {
        self.derivation_within(&ViewOptions::default(), &Limits::default(), |_| false)
    }

    /// Returns the derivation so far as [`derivation`](Self::derivation)
    /// does, refused past `limits.derivation_bytes` by its estimate, as a
    /// proof's derivation is, and until `stop` fires. Its inferences are
    /// the state's own, never compacted, whatever `view` says.
    ///
    /// # Errors
    ///
    /// [`Refusal::Output`] past
    /// `limits.derivation_bytes`, and [`Refusal::Stopped`]
    /// when `stop` fired.
    pub fn derivation_within(
        &self,
        view: &ViewOptions,
        limits: &Limits,
        mut stop: impl FnMut(Progress) -> bool,
    ) -> Result<Derivation<'_>, Error> {
        let _ = view;
        // What the derivation holds, by the size estimate's measure: its
        // inferences and the characters of their sequents.
        let weights = super::size::weights(&self.forest);
        let estimate = self.inferences.iter().fold(0u64, |sum, inference| {
            let characters = inference.sequent.iter().fold(0u64, |sum, m| {
                sum.saturating_add(u64::from(weights[m.index()]))
            });
            sum.saturating_add(Size::BYTES_PER_INFERENCE)
                .saturating_add(characters.saturating_mul(Size::BYTES_PER_CHARACTER))
        });
        if let Some(limit) = limits.derivation_bytes
            && estimate > limit
        {
            return Err(Error::Refused(Refusal::Output {
                what: "derivation",
                estimate_bytes: estimate,
                limit_bytes: limit,
                least_bytes: None,
            }));
        }
        if stop(Progress::new(Phase::View, 0, 0)) {
            return Err(Error::Refused(Refusal::Stopped { phase: Phase::View }));
        }
        let order = self.postorder();
        let mut new_id = vec![InfId::new(u32::MAX); self.inferences.len()];
        for (i, id) in order.iter().enumerate() {
            new_id[id.index()] = InfId::new(i as u32);
        }
        let inferences = order
            .iter()
            .map(|&id| {
                let inference = &self.inferences[id.index()];
                Inference {
                    sequent: inference.sequent.clone(),
                    rule: inference.rule,
                    principal: inference.principal,
                    cut: inference.cut,
                    node: None,
                    premises: inference
                        .premises
                        .iter()
                        .map(|&p| new_id[p.index()])
                        .collect(),
                    times: inference.times,
                }
            })
            .collect();
        Ok(Derivation::from_parts(
            &self.forest,
            self.reading(),
            inferences,
            self.mode,
        ))
    }

    /// Returns, for every inference of [`derivation`](Self::derivation) by
    /// its id, the id of the same inference in this state: what turns the
    /// inference `n` of a drawing (the SVG's `i<n>`) into the goal that
    /// [`apply`](Self::apply) and [`close`](Self::close) take.
    pub fn derivation_ids(&self) -> Vec<GoalId> {
        self.postorder().into_iter().map(GoalId::of).collect()
    }
}

#[cfg(all(test, feature = "parse"))]
mod tests {
    use super::*;
    use crate::proofs::{Node, NodeId, Proof};
    use crate::search::{self, Options, Reason, Verdict};

    /// Parses `input`.
    fn sequent(input: &str) -> Sequent {
        input.parse().unwrap_or_else(|e| panic!("{input:?}: {e}"))
    }

    /// Returns the rules of applicable steps.
    fn names(rules: Vec<Applicable>) -> Vec<Named> {
        rules.into_iter().map(|a| a.rule).collect()
    }

    /// Returns the rule a client names `name`.
    fn named(name: &str) -> Named {
        name.parse().unwrap()
    }

    /// Starts a proof of `input` in `mode` and returns it with its goal.
    fn start(input: &str, mode: Mode) -> (Interactive, GoalId) {
        let state = Interactive::new(&sequent(input), mode).unwrap();
        let goal = state.goals().next().unwrap();
        (state, goal)
    }

    /// A derivation forty thousand rules high is translated into its
    /// proof term, and the term checked, on a stack that a recursion over
    /// its height would overflow.
    #[test]
    fn a_high_derivation_needs_no_stack() {
        const DEPTH: usize = 20_000;
        let text = format!("|- {}1{}", "bot | (".repeat(DEPTH), ")".repeat(DEPTH));
        let proved = std::thread::Builder::new()
            .stack_size(256 * 1024)
            .spawn(move || {
                let (mut state, mut goal) = start(&text, Mode::CLASSICAL);
                for _ in 0..DEPTH {
                    goal = state.apply(goal, &Step::new(0, Rule::Par)).unwrap()[0];
                    goal = state.apply(goal, &Step::new(0, Rule::Bot)).unwrap()[0];
                }
                assert_eq!(state.apply(goal, &Step::new(0, Rule::One)).unwrap(), []);
                state
                    .proof(&Limits::default(), |_| false)
                    .unwrap()
                    .nodes()
                    .len()
            })
            .unwrap()
            .join()
            .unwrap();
        assert_eq!(proved, 2 * DEPTH + 1);
    }

    /// Returns the position in the goal of the first formula printed as
    /// `text`.
    fn at(state: &Interactive, goal: GoalId, text: &str) -> usize {
        let sequent = state.goal(goal).unwrap();
        sequent
            .iter()
            .position(|&o| state.forest().formula(o.occ()).to_string() == text)
            .unwrap_or_else(|| panic!("no {text} in goal {}", goal.get()))
    }

    /// Applies `rule` to the formula printed as `text` and returns the
    /// goals opened.
    fn step(
        state: &mut Interactive,
        goal: GoalId,
        text: &str,
        rule: Rule,
        left: &[&str],
    ) -> Vec<GoalId> {
        let position = at(state, goal, text);
        let left: Vec<usize> = left.iter().map(|t| at(state, goal, t)).collect();
        state
            .apply(goal, &Step::new(position, rule).left(&left))
            .unwrap_or_else(|e| panic!("{rule} on {text}: {e}"))
    }

    /// Every rule applied once through the interface, in a proof the
    /// checker accepts, per fragment and mode.
    #[test]
    fn every_rule_once() {
        use Rule::*;
        let classical = Mode::CLASSICAL;
        // ⅋, ⊗ with a split, ax.
        let (mut s, g) = start("|- ~a par ~b, a * b", classical);
        let [g] = step(&mut s, g, "~a ⅋ ~b", Par, &[])[..] else {
            panic!()
        };
        let [l, r] = step(&mut s, g, "a ⊗ b", Tensor, &["~a"])[..] else {
            panic!()
        };
        assert_eq!(s.goals().collect::<Vec<_>>(), [l, r]);
        step(&mut s, l, "a", Ax, &[]);
        assert_eq!(names(s.rules(r, 0).unwrap()), [Named::from(Ax)]);
        step(&mut s, r, "b", Ax, &[]);
        assert!(s.is_complete());
        let proof = s.proof(&Limits::default(), |_| false).unwrap();
        assert_eq!(proof.nodes().len(), 4);
        assert_eq!(
            s.derivation().unwrap().to_string(),
            proof.derivation().unwrap().to_string()
        );
        // ⊥ and 1.
        let (mut s, g) = start("|- bot, 1", classical);
        let [g] = step(&mut s, g, "⊥", Bot, &[])[..] else {
            panic!()
        };
        step(&mut s, g, "1", One, &[]);
        s.proof(&Limits::default(), |_| false).unwrap();
        // &, ⊕₁, ⊕₂ and ⊤.
        let (mut s, g) = start("|- a & b, ~a + ~b", classical);
        let [l, r] = step(&mut s, g, "a & b", With, &[])[..] else {
            panic!()
        };
        let [l] = step(&mut s, l, "~a ⊕ ~b", PlusLeft, &[])[..] else {
            panic!()
        };
        step(&mut s, l, "a", Ax, &[]);
        let [r] = step(&mut s, r, "~a ⊕ ~b", PlusRight, &[])[..] else {
            panic!()
        };
        step(&mut s, r, "~b", Ax, &[]);
        s.proof(&Limits::default(), |_| false).unwrap();
        let (mut s, g) = start("|- top, 0", classical);
        assert_eq!(names(s.rules(g, at(&s, g, "0")).unwrap()), []);
        step(&mut s, g, "⊤", Top, &[]);
        s.proof(&Limits::default(), |_| false).unwrap();
        // ?c, ?d, ! and ?w.
        let (mut s, g) = start("!a |- a * a", classical);
        let [g] = step(&mut s, g, "?~a", Contraction, &[])[..] else {
            panic!()
        };
        let [l, r] = step(&mut s, g, "a ⊗ a", Tensor, &["?~a"])[..] else {
            panic!()
        };
        for g in [l, r] {
            let [g] = step(&mut s, g, "?~a", Dereliction, &[])[..] else {
                panic!()
            };
            step(&mut s, g, "~a", Ax, &[]);
        }
        let proof = s.proof(&Limits::default(), |_| false).unwrap();
        assert_eq!(
            proof.derivation().unwrap().to_string(),
            "─────── ax    ─────── ax\n\
             ⊢ ~a, a       ⊢ ~a, a\n\
             ──────── ?d   ──────── ?d\n\
             ⊢ ?~a, a      ⊢ ?~a, a\n\
             ────────────────────── ⊗\n\
            \x20 ⊢ ?~a, ?~a, a ⊗ a\n\
            \x20 ───────────────── ?c\n\
            \x20   ⊢ ?~a, a ⊗ a"
        );
        let (mut s, g) = start("!a |- !a", classical);
        let [g] = step(&mut s, g, "!a", Promotion, &[])[..] else {
            panic!()
        };
        let [g] = step(&mut s, g, "?~a", Dereliction, &[])[..] else {
            panic!()
        };
        step(&mut s, g, "~a", Ax, &[]);
        s.proof(&Limits::default(), |_| false).unwrap();
        let (mut s, g) = start("|- ?a, 1", classical);
        let [g] = step(&mut s, g, "?a", Weakening, &[])[..] else {
            panic!()
        };
        step(&mut s, g, "1", One, &[]);
        s.proof(&Limits::default(), |_| false).unwrap();
        // wk in affine mode, and Mix.
        let (mut s, g) = start("a, b |- a", classical.with_affine());
        assert_eq!(
            names(s.rules(g, at(&s, g, "~b")).unwrap()),
            [Named::from(Ax), Named::from(AffineWeakening)]
        );
        let [g] = step(&mut s, g, "~b", AffineWeakening, &[])[..] else {
            panic!()
        };
        step(&mut s, g, "~a", Ax, &[]);
        s.proof(&Limits::default(), |_| false).unwrap();
        let (mut s, g) = start("|- a, ~a, b, ~b", classical.with_mix());
        assert_eq!(
            names(s.rules(g, 0).unwrap()),
            [Named::from(Ax), Named::from(Mix)]
        );
        let [l, r] = step(&mut s, g, "a", Mix, &["~a"])[..] else {
            panic!()
        };
        step(&mut s, l, "a", Ax, &[]);
        step(&mut s, r, "b", Ax, &[]);
        assert_eq!(s.steps(), 3);
        s.proof(&Limits::default(), |_| false).unwrap();
    }

    /// A rule that does not apply is refused with what it needed, and the
    /// state is unchanged.
    #[test]
    fn rejections() {
        use Rule::*;
        let (mut s, g) = start("|- ~a par ~b, a * b, 1, !c, ?d, top", Mode::CLASSICAL);
        let before = s.clone();
        let p = |s: &Interactive, text: &str| at(s, g, text);
        let cases: Vec<(&str, Named, Vec<usize>, StepError)> = vec![
            (
                "~a ⅋ ~b",
                Named::from(Tensor),
                vec![],
                StepError::Rule {
                    rule: Named::from(Tensor),
                    position: p(&s, "~a ⅋ ~b"),
                },
            ),
            (
                "1",
                Named::from(AffineWeakening),
                vec![],
                StepError::Mode {
                    rule: Named::from(AffineWeakening),
                    mode: Mode::CLASSICAL,
                },
            ),
            (
                "1",
                Named::from(Mix),
                vec![],
                StepError::Mode {
                    rule: Named::from(Mix),
                    mode: Mode::CLASSICAL,
                },
            ),
            (
                "1",
                Named::from(One),
                vec![],
                StepError::NotAlone {
                    rule: Named::from(One),
                    position: p(&s, "1"),
                },
            ),
            (
                "!c",
                Named::from(Promotion),
                vec![],
                StepError::NotQuest { position: 0 },
            ),
            (
                "~a ⅋ ~b",
                Named::from(Par),
                vec![1],
                StepError::NoSplit {
                    rule: Named::from(Par),
                },
            ),
            (
                "a ⊗ b",
                Named::from(Tensor),
                vec![9],
                StepError::Split { position: 9 },
            ),
            (
                "a ⊗ b",
                Named::from(Tensor),
                vec![0, 0],
                StepError::Split { position: 0 },
            ),
            (
                "a ⊗ b",
                Named::from(Tensor),
                vec![p(&s, "a ⊗ b")],
                StepError::Split {
                    position: p(&s, "a ⊗ b"),
                },
            ),
            (
                "1",
                named("⊸L"),
                vec![],
                StepError::Rule {
                    rule: named("⊸L"),
                    position: p(&s, "1"),
                },
            ),
        ];
        for (text, rule, left, refusal) in cases {
            let position = p(&s, text);
            assert_eq!(
                s.apply(g, &Step::new(position, rule).left(&left)),
                Err(refusal),
                "{rule} on {text}"
            );
        }
        assert_eq!(
            s.apply(g, &Step::new(99, Par)),
            Err(StepError::NoFormula {
                position: 99,
                len: 6
            })
        );
        assert_eq!(
            s.apply(GoalId::new(7), &Step::new(0, Par)),
            Err(StepError::NoGoal {
                goal: GoalId::new(7)
            })
        );
        assert_eq!(
            s.rules(GoalId::new(7), 0),
            Err(StepError::NoGoal {
                goal: GoalId::new(7)
            })
        );
        assert_eq!(s.inferences(), before.inferences());
        assert_eq!(s.steps(), 0);
        // The axiom needs the dual and nothing else.
        let (mut s, g) = start("|- a, ~a, b", Mode::CLASSICAL);
        assert_eq!(
            s.apply(g, &Step::new(0, Ax)),
            Err(StepError::NotAlone {
                rule: Named::from(Ax),
                position: 0
            })
        );
        let (mut s, g) = start("|- a, b", Mode::CLASSICAL);
        assert_eq!(
            s.apply(g, &Step::new(0, Ax)),
            Err(StepError::NoDual { position: 0 })
        );
        assert_eq!(
            s.apply(g, &Step::new(0, Ax)).unwrap_err().to_string(),
            "the axiom needs formula 0 together with its dual literal and nothing else"
        );
        // Intuitionistic mode: one goal per premise, the goal never weakened,
        // and the two-sided names.
        let i = Mode::INTUITIONISTIC;
        let (mut s, g) = start("a, a -o b |- b", i);
        let imp = at(&s, g, "a ⊗ ~b");
        assert_eq!(names(s.rules(g, imp).unwrap()), [named("⊸L")]);
        assert_eq!(
            s.apply(g, &Step::new(imp, named("⊗R"))),
            Err(StepError::Rule {
                rule: named("⊗R"),
                position: imp
            })
        );
        // The goal b with the antecedent a: two on the right.
        let b = at(&s, g, "b");
        assert_eq!(
            s.apply(g, &Step::new(imp, named("⊸L")).left(&[b])),
            Err(StepError::Succedents { count: 2 })
        );
        let (mut s, g) = start("a, b |- a", i.with_affine());
        let goal = at(&s, g, "a");
        assert_eq!(
            s.apply(g, &Step::new(goal, AffineWeakening)),
            Err(StepError::Output { position: goal })
        );
        assert!(Interactive::new(&sequent("|- a par b"), i).is_err());
        assert!(matches!(
            Interactive::new(&sequent("a |- a"), i.with_mix()),
            Err(Error::IntuitionisticMix)
        ));
    }

    /// In intuitionistic mode the rules carry their two-sided names, the
    /// classical name is accepted, and the derivation is the two-sided one.
    #[test]
    fn intuitionistic() {
        use Rule::*;
        let i = Mode::INTUITIONISTIC;
        let (mut s, g) = start("a, a -o b |- b", i);
        let imp = at(&s, g, "a ⊗ ~b");
        let a = at(&s, g, "~a");
        let [l, r] = s.apply(g, &Step::new(imp, Tensor).left(&[a])).unwrap()[..] else {
            panic!()
        };
        assert_eq!(s.inferences()[g.index()].rule, named("⊸L"));
        assert_eq!(
            s.derivation().unwrap().to_string(),
            "a ⊢ a   b ⊢ b\n\
             ───────────── ⊸L\n\
             a, a ⊸ b ⊢ b"
        );
        s.apply(l, &Step::new(0, Ax)).unwrap();
        s.apply(r, &Step::new(0, Ax)).unwrap();
        let proof = s.proof(&Limits::default(), |_| false).unwrap();
        assert_eq!(proof.check(i), Ok(()));
        assert_eq!(
            s.derivation().unwrap().to_string(),
            proof.two_sided().unwrap().to_string()
        );
        // !L, !c and !R by their two-sided names.
        let (mut s, g) = start("!a |- !(a * a)", i);
        let [g] = s
            .apply(g, &Step::new(at(&s, g, "!(a ⊗ a)"), named("!R")))
            .unwrap()[..]
        else {
            panic!()
        };
        let [g] = s
            .apply(g, &Step::new(at(&s, g, "?~a"), named("!c")))
            .unwrap()[..]
        else {
            panic!()
        };
        assert_eq!(
            names(s.rules(g, at(&s, g, "?~a")).unwrap()),
            [named("!L"), named("!c"), named("!w")]
        );
        let [l, r] = s
            .apply(
                g,
                &Step::new(at(&s, g, "a ⊗ a"), named("⊗R")).left(&[at(&s, g, "?~a")]),
            )
            .unwrap()[..]
        else {
            panic!()
        };
        for g in [l, r] {
            let [g] = s
                .apply(g, &Step::new(at(&s, g, "?~a"), named("!L")))
                .unwrap()[..]
            else {
                panic!()
            };
            s.apply(g, &Step::new(0, Ax)).unwrap();
        }
        assert_eq!(
            s.proof(&Limits::default(), |_| false).unwrap().check(i),
            Ok(())
        );
    }

    /// The search closes a goal that repeats an occurrence, as a
    /// contraction leaves it, and in affine mode goals that need a
    /// weakening; every proof the session ends with passes the checker.
    #[test]
    fn search_closes_repeated_and_affine_goals() {
        use Rule::*;
        let close_all = |s: &mut Interactive| {
            let outcomes = s.close_all(
                &Options::default(),
                &ViewOptions::default(),
                &Limits::default(),
                |_| false,
            );
            assert!(s.is_complete(), "{outcomes:?}");
            s.proof(&Limits::default(), |_| false).unwrap()
        };
        let (mut s, g) = start("|- ?(a par ~a)", Mode::CLASSICAL);
        let [repeated] = step(&mut s, g, "?(a ⅋ ~a)", Contraction, &[])[..] else {
            panic!()
        };
        assert_eq!(s.goal(repeated).unwrap().len(), 2);
        assert_eq!(close_all(&mut s).check(Mode::CLASSICAL), Ok(()));
        let affine = Mode::CLASSICAL.with_affine();
        let (mut s, g) = start("|- a * b, ~a, ~b, c", affine);
        step(&mut s, g, "a ⊗ b", Tensor, &["~a"]);
        assert_eq!(close_all(&mut s).check(affine), Ok(()));
    }

    /// The search closes one goal or every goal, grafting its derivation,
    /// and the result passes the checker; an unprovable goal stays open.
    #[test]
    fn search_closes_goals() {
        use Rule::*;
        let options = Options::default();
        let (mut s, g) = start("|- (a & b) * c, ~a + ~b, ~c", Mode::CLASSICAL);
        let t = at(&s, g, "(a & b) ⊗ c");
        let [l, r] = s
            .apply(g, &Step::new(t, Tensor).left(&[at(&s, g, "~a ⊕ ~b")]))
            .unwrap()[..]
        else {
            panic!()
        };
        let outcome = s
            .close(
                l,
                &options,
                &ViewOptions::default(),
                &crate::Limits::default(),
                |_| false,
            )
            .unwrap()
            .outcome;
        assert!(matches!(outcome.verdict, Verdict::Proved(_)));
        assert_eq!(outcome.engine, crate::search::Engine::Additive);
        assert_eq!(s.goals().collect::<Vec<_>>(), [r]);
        assert_eq!(s.steps(), 2);
        let outcomes = s.close_all(
            &options,
            &ViewOptions::default(),
            &crate::Limits::default(),
            |_| false,
        );
        assert_eq!(outcomes.len(), 1);
        assert_eq!(outcomes[0].0, r);
        assert!(s.is_complete());
        let proof = s.proof(&Limits::default(), |_| false).unwrap();
        assert_eq!(proof.check(Mode::CLASSICAL), Ok(()));
        assert!(s.derivation().unwrap().to_string().contains("⊕₂"));

        // An unprovable goal stays open, a stopped search too.
        let (mut s, g) = start("|- a * b, ~a, ~b", Mode::CLASSICAL);
        let [l, r] = s
            .apply(
                g,
                &Step::new(at(&s, g, "a ⊗ b"), Tensor).left(&[at(&s, g, "~b")]),
            )
            .unwrap()[..]
        else {
            panic!()
        };
        let outcome = s
            .close(
                l,
                &options,
                &ViewOptions::default(),
                &crate::Limits::default(),
                |_| false,
            )
            .unwrap()
            .outcome;
        assert!(matches!(outcome.verdict, Verdict::Unprovable(_)));
        let outcome = s
            .close(
                r,
                &options,
                &ViewOptions::default(),
                &crate::Limits::default(),
                |_| true,
            )
            .unwrap()
            .outcome;
        assert!(matches!(outcome.verdict, Verdict::Unknown(Reason::Stopped)));
        assert_eq!(s.goals().collect::<Vec<_>>(), [l, r]);
        assert!(matches!(
            s.proof(&Limits::default(), |_| false),
            Err(Error::OpenGoals { count: 2 })
        ));
        assert!(matches!(
            s.close(
                g,
                &options,
                &ViewOptions::default(),
                &crate::Limits::default(),
                |_| false
            ),
            Err(Error::Step(StepError::NoGoal { .. }))
        ));

        // Two-sided: the grafted derivation carries the two-sided names.
        let (mut s, g) = start("a & b, !(a -o c) |- c", Mode::INTUITIONISTIC);
        let [g] = s
            .apply(g, &Step::new(at(&s, g, "~a ⊕ ~b"), named("&L₁")))
            .unwrap()[..]
        else {
            panic!()
        };
        let outcome = s
            .close(
                g,
                &options,
                &ViewOptions::default(),
                &crate::Limits::default(),
                |_| false,
            )
            .unwrap()
            .outcome;
        assert!(matches!(outcome.verdict, Verdict::Proved(_)));
        assert!(s.derivation().unwrap().to_string().contains("!L"));
        assert_eq!(
            s.proof(&Limits::default(), |_| false)
                .unwrap()
                .check(Mode::INTUITIONISTIC),
            Ok(())
        );
    }

    /// In the cyclic mode a `⊗` splits its goal into the two arcs on
    /// either side of it, at a cut point or by positions that make arcs,
    /// any other split needing an exchange; the search closes a goal off
    /// the roots; and a sequent beyond unit-free MLL has no session.
    #[test]
    fn cyclic() {
        let cyclic = Mode::CYCLIC;
        // 0 ⊗, 1 a ⊗ b, 2 a, 3 b, 4 c, 5 ~c, 6 ~b, 7 ~a.
        let (mut s, g) = start("|- (a * b) * c, ~c, ~b, ~a", cyclic);
        assert_eq!(
            s.apply(g, &Step::new(0, Rule::Tensor).left(&[1])),
            Err(StepError::Exchange { position: 1 })
        );
        let goals = s.apply(g, &Step::new(0, Rule::Tensor).at(2)).unwrap();
        let (left, right) = (goals[0], goals[1]);
        let members = |s: &Interactive, g| {
            s.goal(g)
                .unwrap()
                .iter()
                .map(|m| m.get())
                .collect::<Vec<_>>()
        };
        assert_eq!(
            (members(&s, left), members(&s, right)),
            (vec![1, 6, 7], vec![4, 5])
        );
        s.apply(right, &Step::new(0, Rule::Ax)).unwrap();
        let options = Options::default();
        let closed = s
            .close(
                left,
                &options,
                &ViewOptions::default(),
                &Limits::default(),
                |_| false,
            )
            .unwrap();
        assert!(matches!(closed.outcome.verdict, Verdict::Proved(_)) && closed.grafted.is_ok());
        assert_eq!(s.goals().count(), 0);
        let proof = s.proof(&Limits::default(), |_| false).unwrap();
        assert_eq!(proof.check(cyclic), Ok(()));
        // The step's own position as the cut point sends nothing left.
        let (mut s, g) = start("|- a * (~a par b), ~b", cyclic);
        let goals = s.apply(g, &Step::new(0, Rule::Tensor).at(0)).unwrap();
        assert_eq!(members(&s, goals[0]), [1]);
        assert!(matches!(
            Interactive::new(&sequent("|- a & b, ~a"), cyclic),
            Err(Error::OrderedFragment { .. })
        ));
    }

    /// In the Lambek calculus a rule on a division is named by it, a split
    /// that would leave a premise without a hypothesis is refused unless
    /// empty antecedents are allowed, and the search closes a goal off the
    /// roots in the order.
    #[test]
    fn lambek() {
        let ordered = |input: &str| {
            Sequent::parse_ordered(input, crate::Order::Cyclic, &Limits::default()).unwrap()
        };
        let lambek = Mode::LAMBEK;
        // 0 ~a, 1 a ⊗ ~b, 2 a, 3 ~b, 4 b: b / a, a ⊢ b.
        let mut s = Interactive::new(&ordered("b / a, a |- b"), lambek).unwrap();
        let g = s.goals().next().unwrap();
        let slash = "/L".parse::<Named>().unwrap();
        assert_eq!(names(s.rules(g, 1).unwrap()), [slash]);
        assert!(matches!(
            s.apply(g, &Step::new(1, "⊸L".parse::<Named>().unwrap()).at(0)),
            Err(StepError::Rule { .. })
        ));
        let goals = s.apply(g, &Step::new(1, slash).at(0)).unwrap();
        assert_eq!(
            goals
                .iter()
                .map(|&g| s.goal(g).unwrap().len())
                .collect::<Vec<_>>(),
            [2, 2]
        );
        s.apply(goals[0], &Step::new(0, Rule::Ax)).unwrap();
        let options = Options::default();
        let closed = s
            .close(
                goals[1],
                &options,
                &ViewOptions::default(),
                &Limits::default(),
                |_| false,
            )
            .unwrap();
        assert!(matches!(closed.outcome.verdict, Verdict::Proved(_)) && closed.grafted.is_ok());
        let proof = s.proof(&Limits::default(), |_| false).unwrap();
        assert_eq!(proof.check(lambek), Ok(()));
        assert_eq!(
            s.derivation().unwrap().to_string(),
            "───── ax   ───── ax\n\
             a ⊢ a      b ⊢ b\n\
             ──────────────── /L\n  \
             b / a, a ⊢ b"
        );
        // 0 ~e, 1 e ⊗ (~a ⅋ a), 2 e, 3 ~a ⅋ a, 4 ~a, 5 a: e ⊢ e ⊗ (a \ a),
        // whose `⊗R` leaves ⊢ a \ a without a hypothesis.
        let sequent = ordered("e |- e * (a \\ a)");
        let step = Step::new(1, Rule::Tensor).at(0);
        let (mut s, mut star) = (
            Interactive::new(&sequent, lambek).unwrap(),
            Interactive::new(&sequent, lambek.with_empty_antecedents()).unwrap(),
        );
        let (g, h) = (s.goals().next().unwrap(), star.goals().next().unwrap());
        assert_eq!(s.apply(g, &step), Err(StepError::EmptyAntecedent));
        assert_eq!(star.apply(h, &step).map(|goals| goals.len()), Ok(2));
        assert!(matches!(
            Interactive::new(&ordered("|- a \\ a"), lambek),
            Err(Error::NotIntuitionistic(_))
        ));
    }

    /// A goal is closed only by a proof over the session's sequent that
    /// passes the checker in the session's mode: a proof of another
    /// sequent, larger or smaller, and a weakening in a linear session are
    /// refused, and the session stays as it was.
    /// A proof of an equal sequent made elsewhere fills a goal, its
    /// conclusions in another order; one of other formulas is refused,
    /// saying which on each side, one of another mode's rules is the
    /// checker's to refuse, and a goal outside the forest is refused, each
    /// leaving the goal open.
    #[test]
    fn fill_carries_a_proof_of_an_equal_sequent() {
        let (view, limits) = (ViewOptions::default(), Limits::default());
        let proof_of = |input: &str, mode: Mode| {
            let outcome = search::prove(&sequent(input), mode, &Options::default()).unwrap();
            let Verdict::Proved(proof) = outcome.verdict else {
                panic!("{input}: {:?}", outcome.verdict)
            };
            proof
        };
        let after_par = || {
            let (mut s, g) = start("A * B |- B * A", Mode::CLASSICAL);
            let par = at(&s, g, "~A ⅋ ~B");
            let goal = s.apply(g, &Step::new(par, Rule::Par)).unwrap()[0];
            (s, goal)
        };
        let (mut s, goal) = after_par();
        let elsewhere = proof_of("|- B * A, ~A, ~B", Mode::CLASSICAL);
        s.fill(goal, &elsewhere, &view, &limits, |_| false).unwrap();
        assert!(s.is_complete());
        s.proof(&limits, |_| false).unwrap();

        let (mut s, goal) = after_par();
        let other = proof_of("|- A * B, ~A, ~B", Mode::CLASSICAL);
        let refused = s.fill(goal, &other, &view, &limits, |_| false).unwrap_err();
        let Error::SequentMismatch { proof, goal: held } = &refused else {
            panic!("{refused:?}")
        };
        assert_eq!(
            (proof, held),
            (&vec!["A ⊗ B".to_owned()], &vec!["B ⊗ A".to_owned()])
        );
        assert_eq!(refused.code(), "sequent_mismatch");
        let fewer = proof_of("|- ~A, A", Mode::CLASSICAL);
        let refused = s.fill(goal, &fewer, &view, &limits, |_| false).unwrap_err();
        let Error::SequentMismatch { proof, goal: held } = &refused else {
            panic!("{refused:?}")
        };
        assert_eq!((proof.len(), held.len()), (1, 2), "{refused}");
        assert_eq!(s.goals().collect::<Vec<_>>(), [goal]);

        let (mut s, g) = start("|- A, ~A, B, ~B", Mode::CLASSICAL);
        let mixed = proof_of("|- B, ~B, A, ~A", Mode::CLASSICAL.with_mix());
        assert!(matches!(
            s.fill(g, &mixed, &view, &limits, |_| false),
            Err(Error::Check(crate::proofs::CheckError::Invalid(_)))
        ));
        assert_eq!(s.goals().collect::<Vec<_>>(), [g]);
        let past = Member::new(u32::try_from(s.forest().len()).unwrap());
        assert!(matches!(
            mixed.onto(s.forest().clone(), &[past]),
            Err(Error::IndexOutOfBounds { .. })
        ));
    }

    #[test]
    fn close_with_refuses_a_foreign_proof() {
        let view = ViewOptions::default();
        let (mut s, g) = start("A, A -o B |- B", Mode::CLASSICAL);
        for other in ["|- A, B, C, D, top", "|- top"] {
            let forest = Forest::new(&sequent(other)).unwrap();
            let top = *forest.roots().last().unwrap();
            let proof =
                Proof::new(forest, vec![Node::Top(Member::from(top))], NodeId::new(0)).unwrap();
            assert!(matches!(
                s.close_with(g, &proof, &view, &crate::Limits::default(), |_| false),
                Err(Error::ForeignProof)
            ));
        }
        assert_eq!(s.goals().collect::<Vec<_>>(), [g]);

        let (mut s, g) = start("A, B |- A", Mode::CLASSICAL);
        let affine = search::prove_goal(
            search::Goal::new(s.forest(), s.goal(g).unwrap()).unwrap(),
            Mode::CLASSICAL.with_affine(),
            &Options::default(),
            &Limits::default(),
            |_| false,
        )
        .unwrap();
        let Verdict::Proved(proof) = affine.verdict else {
            panic!("{:?}", affine.verdict)
        };
        assert!(matches!(
            s.close_with(g, &proof, &view, &crate::Limits::default(), |_| false),
            Err(Error::Check(crate::proofs::CheckError::Invalid(_)))
        ));
        assert_eq!(s.goals().collect::<Vec<_>>(), [g]);

        // A proof of the sequent closes the root's goal and no other.
        let (mut s, g) = start("A, B |- A * B", Mode::CLASSICAL);
        let outcome = search::prove(s.sequent(), Mode::CLASSICAL, &Options::default()).unwrap();
        let Verdict::Proved(whole) = outcome.verdict else {
            panic!("{:?}", outcome.verdict)
        };
        let tensor = at(&s, g, "A ⊗ B");
        let premises = s
            .apply(g, &Step::new(tensor, Rule::Tensor).left(&[at(&s, g, "~A")]))
            .unwrap();
        assert!(matches!(
            s.close_with(premises[0], &whole, &view, &Limits::default(), |_| false),
            Err(Error::GoalMismatch)
        ));
    }

    /// The split helper says which splits the counts refuse.
    #[test]
    fn split_helper() {
        let (s, g) = start("|- a * b, ~a, ~b", Mode::CLASSICAL);
        let t = at(&s, g, "a ⊗ b");
        assert!(
            s.split_passes(g, &Step::new(t, Rule::Tensor).left(&[at(&s, g, "~a")]))
                .unwrap()
        );
        assert!(
            !s.split_passes(g, &Step::new(t, Rule::Tensor).left(&[at(&s, g, "~b")]))
                .unwrap()
        );
        assert!(
            !s.split_passes(g, &Step::new(t, Rule::Tensor).left(&[]))
                .unwrap()
        );
        assert_eq!(
            s.split_passes(g, &Step::new(t, Rule::Tensor).left(&[t])),
            Err(StepError::Split { position: t })
        );
        let (s, g) = start("|- a, ~a, b, ~b", Mode::CLASSICAL.with_mix());
        assert!(
            s.split_passes(g, &Step::new(0, Rule::Tensor).left(&[at(&s, g, "~a")]))
                .unwrap()
        );
        assert!(
            !s.split_passes(g, &Step::new(0, Rule::Tensor).left(&[at(&s, g, "b")]))
                .unwrap()
        );
    }

    /// Undo retracts a rule application and a grafted search alike,
    /// restoring the goals, and there is nothing to undo at the start.
    #[test]
    fn undo_restores_goals() {
        use Rule::*;
        let (mut s, g) = start("|- ~a par ~b, a * b", Mode::CLASSICAL);
        assert_eq!(s.undo(), None);
        let start_state = s.clone();
        let [g1] = s.apply(g, &Step::new(at(&s, g, "~a ⅋ ~b"), Par)).unwrap()[..] else {
            panic!()
        };
        let after_par = s.clone();
        let [l, _] = s
            .apply(
                g1,
                &Step::new(at(&s, g1, "a ⊗ b"), Tensor).left(&[at(&s, g1, "~a")]),
            )
            .unwrap()[..]
        else {
            panic!()
        };
        s.close(
            l,
            &Options::default(),
            &ViewOptions::default(),
            &crate::Limits::default(),
            |_| false,
        )
        .unwrap();
        assert_eq!(s.undo(), Some(l));
        assert_eq!(s.goals().count(), 2);
        assert_eq!(s.undo(), Some(g1));
        assert_eq!(s.inferences(), after_par.inferences());
        assert_eq!(s.undo(), Some(g));
        assert_eq!(s.inferences(), start_state.inferences());
        assert_eq!(s.undo(), None);
        let outcome = s
            .close(
                g,
                &Options::default(),
                &ViewOptions::default(),
                &crate::Limits::default(),
                |_| false,
            )
            .unwrap()
            .outcome;
        assert!(matches!(outcome.verdict, Verdict::Proved(_)));
        assert!(s.is_complete());
        assert_eq!(s.undo(), Some(g));
        assert_eq!(s.inferences(), start_state.inferences());
    }

    /// The derivation of a proof in progress draws open goals as bare
    /// sequents.
    #[test]
    fn open_goals_render() {
        use Rule::*;
        let (mut s, g) = start("|- ~a par ~b, a * b", Mode::CLASSICAL);
        let [g] = s.apply(g, &Step::new(at(&s, g, "~a ⅋ ~b"), Par)).unwrap()[..] else {
            panic!()
        };
        s.apply(
            g,
            &Step::new(at(&s, g, "a ⊗ b"), Tensor).left(&[at(&s, g, "~a")]),
        )
        .unwrap();
        assert_eq!(
            s.derivation().unwrap().to_string(),
            "⊢ ~a, a   ⊢ ~b, b\n\
             ───────────────── ⊗\n\
            \x20⊢ ~a, ~b, a ⊗ b\n\
            \x20──────────────── ⅋\n\
            \x20⊢ ~a ⅋ ~b, a ⊗ b"
        );
        let d = s.derivation().unwrap();
        assert_eq!(d.inference(d.root()).rule, Named::from(Par));
        assert_eq!(d.inference(InfId::new(0)).rule, Named::from(Open));
    }

    /// Mix on a goal that repeats the formula it is applied at keeps both
    /// copies.
    #[test]
    fn mix_keeps_repeated_formulas() {
        use Rule::*;
        let (mut s, g) = start("|- ?(a par ~a)", Mode::CLASSICAL.with_mix());
        let [g] = step(&mut s, g, "?(a ⅋ ~a)", Contraction, &[])[..] else {
            panic!()
        };
        let [g] = step(&mut s, g, "?(a ⅋ ~a)", Dereliction, &[])[..] else {
            panic!()
        };
        let [g] = step(&mut s, g, "?(a ⅋ ~a)", Dereliction, &[])[..] else {
            panic!()
        };
        let [l, r] = step(&mut s, g, "a ⅋ ~a", Mix, &[])[..] else {
            panic!()
        };
        assert_eq!(s.goal(l), s.goal(r));
        // A Mix of the goal's one formula: the right premise would be
        // empty, which nothing closes.
        assert_eq!(s.goal(l).map(<[Member]>::len), Ok(1));
        assert_eq!(s.apply(l, &Step::new(0, Mix)), Err(StepError::EmptyPremise));
        // Each goal gets its own result.
        let closed = s.close_all(
            &Options::default(),
            &ViewOptions::default(),
            &crate::Limits::default(),
            |_| false,
        );
        assert_eq!(closed.iter().map(|(g, _)| *g).collect::<Vec<_>>(), [l, r]);
        assert!(
            closed
                .iter()
                .all(|(_, c)| c.as_ref().is_ok_and(|c| c.grafted.is_ok()))
        );
        assert!(s.is_complete());
    }
}
