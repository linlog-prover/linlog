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

use super::derivation::{Derivation, InfId, Inference, ViewOptions};
use super::multiset::Multiset;
use super::size::Size;
use super::{Branch, CheckError, Node, NodeId, Proof};
use super::{Named, Rule};
use crate::Error;
use crate::fragment::Mode;
use crate::limits::{Limits, Phase, Progress, Refusal};
use crate::occurrences::{Forest, Member, OccId, Reading, Side};
use crate::search::{self, Options, Outcome, Verdict, focus};
use crate::sequents::{Kind, Sequent};
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
    NoGoal {
        /// The id asked for.
        goal: GoalId,
    },
    /// The goal has no formula at the position; it has `len` formulas.
    NoFormula {
        /// The position asked for.
        position: usize,
        /// How many formulas the goal has.
        len: usize,
    },
    /// The rule does not act on the formula at the position: its connective
    /// is another, or in intuitionistic mode the rule is for the other
    /// side of `⊢`.
    Rule {
        /// The rule asked for.
        rule: Named,
        /// The position of the formula.
        position: usize,
    },
    /// The rule is not available in the mode: weakening of a formula that
    /// is not a `?` formula needs affine mode, Mix needs Mix.
    Mode {
        /// The rule asked for.
        rule: Named,
        /// The mode of the proof.
        mode: Mode,
    },
    /// The rule needs the formula alone in the goal (`1`), or alone with its
    /// dual (`ax`); in affine mode the other formulas can be weakened first.
    NotAlone {
        /// The rule asked for.
        rule: Named,
        /// The position of the formula.
        position: usize,
    },
    /// The axiom needs the goal to be the literal and its dual, and the
    /// other formula is not the dual.
    NoDual {
        /// The position of the literal.
        position: usize,
    },
    /// Promotion needs every other formula to be a `?` formula, and the one
    /// at the position is not.
    NotQuest {
        /// The position of the offending formula.
        position: usize,
    },
    /// A position of the split is out of range, repeated, or the formula the
    /// rule acts on.
    Split {
        /// The offending position.
        position: usize,
    },
    /// A split was given to a rule that takes none.
    NoSplit {
        /// The rule asked for.
        rule: Named,
    },
    /// In intuitionistic mode, a premise would have this many formulas on
    /// the right of `⊢` instead of one.
    Succedents {
        /// How many formulas would stand right of `⊢`.
        count: usize,
    },
    /// In intuitionistic mode, the formula on the right of `⊢` cannot be
    /// weakened.
    Output {
        /// The position of the formula.
        position: usize,
    },
    /// A Mix whose split leaves a premise without a formula: no rule
    /// concludes the empty sequent, so nothing could close it.
    EmptyPremise,
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

    /// Returns the positions going left, empty without a split.
    fn left_positions(&self) -> &[usize] {
        match &self.split {
            Split::None => &[],
            Split::Left { positions } => positions,
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
    Left {
        /// The positions in the goal's sequent.
        positions: Vec<usize>,
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
#[derive(Debug)]
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
/// history as the inferences the steps closed, in order. Reading it back
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
    pub fn new(sequent: &Sequent, mode: Mode) -> Result<Self, Error> {
        Self::within(sequent, mode, &Limits::default())
    }

    /// Starts a proof as [`new`](Self::new) does, of a sequent that
    /// unfolds to at most `limits.occurrences` occurrences
    /// ([`Refusal::Occurrences`] otherwise).
    pub fn within(sequent: &Sequent, mode: Mode, limits: &Limits) -> Result<Self, Error> {
        let forest = Forest::within(sequent, limits)?;
        Self::from_forest(forest, mode)
    }

    /// Starts a proof over a forest built already.
    fn from_forest(forest: Forest, mode: Mode) -> Result<Self, Error> {
        let reading = if mode.intuitionistic {
            if mode.mix {
                return Err(Error::IntuitionisticMix);
            }
            let reading = Reading::new(&forest).map_err(Error::NotIntuitionistic)?;
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
        };
        Ok(Self {
            forest,
            mode,
            reading,
            inferences: vec![root],
            history: vec![],
        })
    }

    /// Rebuilds a state from its parts, as deserialization does, checking
    /// that they are consistent: inference 0 concludes the sequent, every
    /// other inference is the premise of exactly one earlier inference,
    /// every closed inference's premises are what its rule on its sequent
    /// yields, and the history names distinct closed inferences whose
    /// premises were, in order, the last inferences added.
    pub(crate) fn from_parts(
        forest: Forest,
        mode: Mode,
        inferences: Vec<Inference>,
        history: Vec<InfId>,
    ) -> Result<Self, Error> {
        let mut state = Self::from_forest(forest, mode)?;
        let Some(root) = inferences.first() else {
            return Err(Error::InconsistentSession {
                reason: "there is no inference",
            });
        };
        if !root
            .sequent
            .iter()
            .map(|m| m.occ())
            .eq(state.forest.roots().iter().copied())
        {
            return Err(Error::InconsistentSession {
                reason: "inference 0 does not conclude the sequent",
            });
        }
        let n = inferences.len();
        let mut parent = vec![None; n];
        for (i, inference) in inferences.iter().enumerate() {
            for &o in &inference.sequent {
                if o.index() >= state.forest.len() {
                    return Err(Error::IndexOutOfBounds {
                        space: crate::limits::Space::Occurrence,
                        index: o.index(),
                        len: state.forest.len(),
                    });
                }
            }
            if !inference.sequent.is_sorted() {
                return Err(Error::InconsistentSession {
                    reason: "a sequent is not in ascending order",
                });
            }
            for &p in &inference.premises {
                if p.index() <= i || p.index() >= n || parent[p.index()].is_some() {
                    return Err(Error::InconsistentSession {
                        reason: "a premise must be a later inference and the premise of one inference only",
                    });
                }
                parent[p.index()] = Some(i);
            }
        }
        if parent.iter().skip(1).any(Option::is_none) {
            return Err(Error::InconsistentSession {
                reason: "an inference other than the root is nobody's premise",
            });
        }
        state.inferences = inferences;
        let reading = state.reading();
        for id in 0..n {
            let inference = &state.inferences[id];
            if inference.rule.rule == Rule::Open {
                if inference.principal.is_some() || !inference.premises.is_empty() {
                    return Err(Error::InconsistentSession {
                        reason: "an open goal has no principal formula and no premise",
                    });
                }
                continue;
            }
            state.replay(reading.as_ref(), InfId::new(id as u32))?;
        }
        // Seen from the last step back, the inferences a step added are
        // those of its subtree that exist at that point, and they are the
        // suffix of the arena then.
        let mut len = n;
        let mut seen = vec![false; n];
        for &goal in history.iter().rev() {
            // Below `len`, the arena as this step left it: an entry inside
            // a later step's subtree would make `undo` index past it.
            let closed = goal.index() < len
                && state.inferences[goal.index()].rule.rule != Rule::Open
                && !std::mem::replace(&mut seen[goal.index()], true);
            if !closed {
                return Err(Error::InconsistentSession {
                    reason: "the history names an inference that is not a closed one, or names one twice",
                });
            }
            // The inferences this step added: those of its subtree below
            // `len`. A premise has a larger index than its conclusion, so
            // the walk enters no inference a later step added, and every
            // inference is walked by one step only.
            let (mut added, mut lowest) = (0, len);
            let mut walk = state.inferences[goal.index()].premises.clone();
            while let Some(id) = walk.pop() {
                if id.index() >= len {
                    continue;
                }
                added += 1;
                lowest = lowest.min(id.index());
                walk.extend(state.inferences[id.index()].premises.iter().copied());
            }
            // Each inference is the premise of one other only, so the walk
            // counts each once: `added` of them below `len` and none below
            // `start` is exactly the suffix `start..len`.
            let start = len - added;
            if lowest < start {
                return Err(Error::InconsistentSession {
                    reason: "the history does not match the order of the inferences",
                });
            }
            len = start;
        }
        state.history = history;
        Ok(state)
    }

    /// Checks that a closed inference names a principal formula exactly
    /// when its rule has one and that its premises are what the rule
    /// yields on its sequent, recovering the split of a `⊗` or Mix from
    /// the left premise.
    fn replay(&self, reading: Option<&Reading>, id: InfId) -> Result<(), Error> {
        let inference = &self.inferences[id.index()];
        let sequent: &[OccId] = &inference
            .sequent
            .iter()
            .map(|m| m.occ())
            .collect::<Vec<_>>();
        let rule = inference.rule;
        let classical = rule.rule;
        let has_principal = classical.has_principal();
        if inference.principal.is_some() != has_principal {
            return Err(Error::InconsistentSession {
                reason: "a rule other than the axiom and Mix names its principal formula, those two none",
            });
        }
        let principal = inference.principal;
        if principal.is_some_and(|p| p >= sequent.len()) {
            return Err(Error::InconsistentSession {
                reason: "a principal position is out of range",
            });
        }
        // The formulas the left premise took from the context, as positions
        // in the sequent.
        let mut left = Vec::new();
        if matches!(classical, Rule::Tensor | Rule::Mix) {
            let Some(&first) = inference.premises.first() else {
                return Err(Error::InconsistentSession {
                    reason: "a split has no premise",
                });
            };
            let mut context = Multiset::of(
                self.inferences[first.index()]
                    .sequent
                    .iter()
                    .map(|m| m.occ()),
            );
            if let Some(p) = principal {
                context.remove(self.forest.left(sequent[p]).unwrap_or(sequent[p]));
            }
            // Both are sorted, so each member of the context is the next
            // equal one of the sequent, the principal formula skipped.
            let mut at = 0;
            for &o in context.as_slice() {
                while at < sequent.len() && (sequent[at] < o || Some(at) == principal) {
                    at += 1;
                }
                if at == sequent.len() || sequent[at] != o {
                    return Err(Error::InconsistentSession {
                        reason: "a premise holds a formula its conclusion lacks",
                    });
                }
                left.push(at);
                at += 1;
            }
        }
        let position = match principal {
            Some(p) => p,
            // Mix: the first formula of the left premise stands for the
            // position; the axiom has position 0, its sequent being the
            // two literals.
            None if classical == Rule::Mix => {
                if left.is_empty() {
                    return Err(Error::InconsistentSession {
                        reason: "a Mix has an empty left premise",
                    });
                }
                left.remove(0)
            }
            None => 0,
        };
        let expected = self.expand(reading, sequent, position, rule, &left)?;
        let same = expected.len() == inference.premises.len()
            && expected.iter().zip(&inference.premises).all(|(e, p)| {
                e.as_slice()
                    .iter()
                    .copied()
                    .map(Member::from)
                    .eq(self.inferences[p.index()].sequent.iter().copied())
            });
        if same {
            Ok(())
        } else {
            Err(Error::InconsistentSession {
                reason: "an inference's premises are not what its rule yields",
            })
        }
    }

    /// Returns the forest of the sequent being proved.
    pub fn forest(&self) -> &Forest {
        &self.forest
    }

    /// Returns the occurrence a member of the session stands for.
    pub fn occurrence(&self, member: Member) -> OccId {
        member.occ()
    }

    /// Returns the formula a member of the session stands for, as the
    /// one-sided sequent writes it.
    pub fn formula(&self, member: Member) -> impl Display + '_ {
        self.forest.formula(member.occ())
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
        Some(Reading::of_parts(&self.forest, position, *goal))
    }

    /// Returns every inference, the root first and then in the order the
    /// steps opened them; an open goal is an inference with [`Rule::Open`].
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

    /// Returns the sequent of an open goal, or `None` if the id is not one.
    pub fn goal(&self, id: GoalId) -> Option<&[Member]> {
        self.inferences
            .get(id.index())
            .filter(|inference| inference.rule.rule == Rule::Open)
            .map(|inference| inference.sequent.as_slice())
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
    pub(crate) fn history(&self) -> &[InfId] {
        &self.history
    }

    /// Returns the sequent of an open goal, or the refusal.
    fn open(&self, id: GoalId) -> Result<Vec<OccId>, StepError> {
        let goal = self.goal(id).ok_or(StepError::NoGoal { goal: id })?;
        Ok(goal.iter().map(|m| m.occ()).collect())
    }

    /// Returns the rules that can act on the formula at `position` of the
    /// open goal in this mode, by the formula's connective and the mode
    /// alone: the axiom on a literal, `⊗ ⅋ 1 ⊥ & ⊕₁ ⊕₂ ⊤ !` on their
    /// connectives, `?d ?c ?w` on a `?` formula, `wk` on any other formula
    /// in affine mode, and Mix on any formula when the mode has it; in
    /// intuitionistic mode by their two-sided names. Nothing acts on `0`.
    /// Whether the goal's context lets a rule apply (the axiom's dual, the
    /// context of `1` and `!`, a valid split, one succedent) is what
    /// [`apply`](Self::apply) decides.
    pub fn rules(&self, goal: GoalId, position: usize) -> Result<Vec<Applicable>, StepError> {
        let sequent = self.open(goal)?;
        let o = formula_at(&sequent, position)?;
        use Rule::*;
        let mut rules = match self.forest.kind(o) {
            Kind::Atom | Kind::DualAtom => vec![Ax],
            Kind::Tensor => vec![Tensor],
            Kind::Par => vec![Par],
            Kind::One => vec![One],
            Kind::Bot => vec![Bot],
            Kind::With => vec![With],
            Kind::Plus => vec![PlusLeft, PlusRight],
            Kind::Top => vec![Top],
            Kind::Zero => vec![],
            Kind::Bang => vec![Promotion],
            Kind::Quest => vec![Dereliction, Contraction, Weakening],
        };
        if self.mode.affine && self.forest.kind(o) != Kind::Quest {
            rules.push(AffineWeakening);
        }
        if self.mode.mix {
            rules.push(Mix);
        }
        let side = self.reading().map(|reading| reading.position(o));
        Ok(rules
            .into_iter()
            .map(|rule| Applicable {
                rule: Named::new(rule, side),
                needs: match rule {
                    Tensor | Mix => Needs::Split,
                    _ => Needs::Nothing,
                },
            })
            .collect())
    }

    /// Applies a step to the open goal and returns the goals it opens, in
    /// the rule's order of premises; a closed leaf opens none. The split
    /// is what a `⊗` or Mix needs ([`Step::left`]), and no other rule takes
    /// one. In intuitionistic mode the rule may be given by its classical
    /// or its two-sided name and is recorded by the latter. Fails with what
    /// the rule needed, and changes nothing then.
    pub fn apply(&mut self, goal: GoalId, step: &Step) -> Result<Vec<GoalId>, StepError> {
        let (position, rule, left) = (step.position, step.rule, step.left_positions());
        let sequent = self.open(goal)?;
        let reading = self.reading();
        let (rule, premises) = {
            let o = formula_at(&sequent, position)?;
            let rule = match &reading {
                Some(reading) => {
                    let named = rule.rule.on(reading.position(o));
                    if rule.side.is_some() && rule != named {
                        return Err(StepError::Rule { rule, position });
                    }
                    named
                }
                None => rule,
            };
            (
                rule,
                self.expand(reading.as_ref(), &sequent, position, rule, left)?,
            )
        };
        let principal = rule.rule.has_principal().then_some(position);
        let base = self.inferences.len() as u32;
        let ids: Vec<InfId> = (0..premises.len() as u32)
            .map(|i| InfId::new(base + i))
            .collect();
        for premise in premises {
            self.inferences.push(Inference {
                sequent: premise.into_members(),
                rule: Rule::Open.into(),
                principal: None,
                premises: vec![],
                times: 1,
            });
        }
        let inference = &mut self.inferences[goal.index()];
        inference.rule = rule;
        inference.principal = principal;
        inference.premises = ids.clone();
        self.history.push(goal.inf());
        Ok(ids.into_iter().map(GoalId::of).collect())
    }

    /// Computes the premises of a rule on a sequent, in the rule's order, or
    /// refuses: the rule must act on the formula's connective, be available
    /// in the mode, find the context it needs, take a valid split, and in
    /// intuitionistic mode leave one formula on the right of `⊢` in every
    /// premise and never weaken that formula.
    fn expand(
        &self,
        reading: Option<&Reading>,
        sequent: &[OccId],
        position: usize,
        rule: Named,
        left: &[usize],
    ) -> Result<Vec<Multiset>, StepError> {
        use Rule::*;
        let f = &self.forest;
        let o = formula_at(sequent, position)?;
        let kind = f.kind(o);
        let classical = rule.rule;
        // The rule acts on the connective, and on the right side of ⊢.
        let acts = match classical {
            Ax => kind.is_literal(),
            Tensor => kind == Kind::Tensor,
            Par => kind == Kind::Par,
            One => kind == Kind::One,
            Bot => kind == Kind::Bot,
            With => kind == Kind::With,
            PlusLeft | PlusRight => kind == Kind::Plus,
            Top => kind == Kind::Top,
            Promotion => kind == Kind::Bang,
            Dereliction | Contraction | Weakening => kind == Kind::Quest,
            AffineWeakening => kind != Kind::Quest,
            Mix => true,
            Open => false,
        };
        let named = Named::new(classical, reading.map(|reading| reading.position(o)));
        if !acts || rule != named {
            return Err(StepError::Rule { rule, position });
        }
        match classical {
            AffineWeakening if !self.mode.affine => {
                return Err(StepError::Mode {
                    rule,
                    mode: self.mode,
                });
            }
            Mix if !self.mode.mix => {
                return Err(StepError::Mode {
                    rule,
                    mode: self.mode,
                });
            }
            _ => {}
        }
        let splits = matches!(classical, Tensor | Mix);
        if !splits && !left.is_empty() {
            return Err(StepError::NoSplit { rule });
        }
        // The context without the formula, and the part of it going left.
        let mut rest = Multiset::of(sequent.iter().copied());
        rest.remove(o);
        let mut going_left = Multiset::new();
        if splits {
            let mut used = vec![false; sequent.len()];
            used[position] = true;
            for &p in left {
                if p >= sequent.len() || used[p] {
                    return Err(StepError::Split { position: p });
                }
                used[p] = true;
                going_left.insert(sequent[p]);
            }
        }
        let child = |side: Branch| match side {
            Branch::Left => f.left(o).unwrap(),
            Branch::Right => f.right(o).unwrap(),
        };
        let with = |added: &[OccId]| {
            let mut premise = rest.clone();
            for &a in added {
                premise.insert(a);
            }
            premise
        };
        let premises = match classical {
            Ax => {
                let [x, y] = sequent else {
                    return Err(StepError::NotAlone { rule, position });
                };
                let other = if position == 0 { *y } else { *x };
                if f.atom(other) != f.atom(o) || f.sign(other) == f.sign(o) {
                    return Err(StepError::NoDual { position });
                }
                vec![]
            }
            One => {
                if sequent.len() != 1 {
                    return Err(StepError::NotAlone { rule, position });
                }
                vec![]
            }
            Top => vec![],
            Tensor => {
                let mut right = rest.difference(&going_left);
                going_left.insert(child(Branch::Left));
                right.insert(child(Branch::Right));
                vec![going_left, right]
            }
            Mix => {
                let right = rest.difference(&going_left);
                // No rule concludes the empty sequent.
                if right.is_empty() {
                    return Err(StepError::EmptyPremise);
                }
                going_left.insert(o);
                vec![going_left, right]
            }
            Par => vec![with(&[child(Branch::Left), child(Branch::Right)])],
            Bot => vec![with(&[])],
            With => vec![with(&[child(Branch::Left)]), with(&[child(Branch::Right)])],
            PlusLeft => vec![with(&[child(Branch::Left)])],
            PlusRight => vec![with(&[child(Branch::Right)])],
            Promotion => {
                if let Some(p) = sequent
                    .iter()
                    .enumerate()
                    .position(|(i, &x)| i != position && f.kind(x) != Kind::Quest)
                {
                    return Err(StepError::NotQuest { position: p });
                }
                vec![with(&[child(Branch::Left)])]
            }
            Dereliction => vec![with(&[child(Branch::Left)])],
            Contraction => vec![with(&[o, o])],
            Weakening | AffineWeakening => {
                if let Some(reading) = reading
                    && reading.position(o) == Side::Output
                {
                    return Err(StepError::Output { position });
                }
                vec![with(&[])]
            }
            _ => unreachable!("handled above"),
        };
        if let Some(reading) = reading {
            for premise in &premises {
                let outputs = reading.outputs(premise.as_slice().iter().copied());
                if outputs != 1 {
                    return Err(StepError::Succedents { count: outputs });
                }
            }
        }
        Ok(premises)
    }

    /// Returns whether the split of a step of `⊗` (or a Mix, when the
    /// formula at the step's position is not a `⊗`) passes the count
    /// prunes of the focused engine, a cheap test that says "this split
    /// cannot close" before a client tries it; a split that passes may
    /// still fail. The step's rule is not read.
    pub fn split_passes(&self, goal: GoalId, step: &Step) -> Result<bool, StepError> {
        let (position, left) = (step.position, step.left_positions());
        let sequent = self.open(goal)?;
        let o = formula_at(&sequent, position)?;
        let rule = if self.forest.kind(o) == Kind::Tensor {
            Rule::Tensor
        } else {
            Rule::Mix
        };
        let reading = self.reading();
        let rule = Named::new(rule, reading.as_ref().map(|reading| reading.position(o)));
        let premises = self.expand(reading.as_ref(), &sequent, position, rule, left)?;
        let [l, r] = premises.as_slice() else {
            unreachable!("a split has two premises");
        };
        let fragment = search::goal_fragment(&self.forest, &sequent);
        Ok(focus::split_passes(
            &self.forest,
            fragment,
            self.mode,
            l.as_slice(),
            r.as_slice(),
        ))
    }

    /// Retracts the last step, a rule applied or a goal closed by the
    /// search, and returns the goal it reopened, or `None` when no step is
    /// left.
    pub fn undo(&mut self) -> Option<GoalId> {
        let goal = self.history.pop()?;
        // The step's inferences are the suffix of the arena.
        let first = self.subtree(goal).into_iter().min();
        if let Some(first) = first {
            self.inferences.truncate(first.index());
        }
        let inference = &mut self.inferences[goal.index()];
        inference.rule = Rule::Open.into();
        inference.principal = None;
        inference.premises.clear();
        Some(GoalId::of(goal))
    }

    /// Returns the inferences above `id`, excluding it, in no particular
    /// order.
    fn subtree(&self, id: InfId) -> Vec<InfId> {
        let mut ids = self.inferences[id.index()].premises.clone();
        let mut i = 0;
        while i < ids.len() {
            ids.extend(self.inferences[ids[i].index()].premises.iter().copied());
            i += 1;
        }
        ids
    }

    /// Runs the search on an open goal with the options and the stop
    /// condition, as [`prove_goal`](crate::search::prove_goal) does, and
    /// returns its outcome. When the goal is proved, the derivation of the
    /// proof found is grafted onto the goal as one step, which
    /// [`undo`](Self::undo) retracts whole; otherwise nothing changes. The
    /// outcome's proof, if any, is the proof of the goal alone.
    ///
    /// The search runs within `limits`, and the derivation grafted is
    /// within them too: a goal whose proof unfolds into a larger one, or
    /// whose unfolding `stop` ends, stays open, and [`Closed::grafted`]
    /// says why; its proof stays in the outcome. A proved goal is never
    /// lost.
    pub fn close(
        &mut self,
        goal: GoalId,
        options: &Options,
        view: &ViewOptions,
        limits: &Limits,
        mut stop: impl FnMut(Progress) -> bool,
    ) -> Result<Closed, Error> {
        let sequent = self.open(goal)?;
        let outcome = search::prove_goal(
            &self.forest,
            &sequent,
            self.mode,
            options,
            limits,
            &mut stop,
        )?;
        let mut grafted = Ok(());
        if let Verdict::Proved(proof) = &outcome.verdict {
            grafted = match self.close_with(goal, proof, view, limits, stop) {
                Ok(()) => Ok(()),
                Err(Error::Refused(refusal)) => Err(refusal),
                Err(Error::Check(CheckError::Refused(refused))) => Err(refused.refusal),
                Err(error) => return Err(error),
            };
        }
        Ok(Closed { outcome, grafted })
    }

    /// Closes an open goal with a proof of it that a search found, as
    /// [`close`](Self::close) does with its own: the proof must be over the
    /// session's sequent ([`Error::ForeignProof`] otherwise), its root must
    /// conclude the goal, as the proof of [`prove_goal`] on the goal's
    /// occurrences ([`goal`](Self::goal)) does, and it must pass the
    /// checker in the session's mode; its derivation is grafted within
    /// `limits` as one step. A caller that runs the search itself,
    /// several of them side by side for one, closes the goal with this.
    ///
    /// [`prove_goal`]: crate::search::prove_goal
    pub fn close_with(
        &mut self,
        goal: GoalId,
        proof: &Proof,
        view: &ViewOptions,
        limits: &Limits,
        mut stop: impl FnMut(Progress) -> bool,
    ) -> Result<(), Error> {
        let sequent = self.open(goal)?;
        // The goal's ids name occurrences of the session's forest, which
        // is a function of its sequent.
        if proof.sequent() != self.sequent() {
            return Err(Error::ForeignProof);
        }
        let mut concluded: Vec<OccId> = proof.conclusion();
        concluded.sort_unstable();
        if concluded != sequent {
            return Err(Error::GoalMismatch);
        }
        let found = Derivation::of_goal(proof, &sequent, self.mode, view, limits, &mut stop)?;
        self.graft(goal.inf(), found);
        self.history.push(goal.inf());
        Ok(())
    }

    /// Runs the search on every open goal in turn, as [`close`](Self::close)
    /// does, and returns each goal with what closing it did; a goal the
    /// search does not prove, or whose closing fails, stays open, and the
    /// run goes on with the next. The limits bound each goal's search.
    pub fn close_all(
        &mut self,
        options: &Options,
        view: &ViewOptions,
        limits: &Limits,
        mut stop: impl FnMut(Progress) -> bool,
    ) -> Vec<(GoalId, Result<Closed, Error>)> {
        let goals: Vec<GoalId> = self.goals().collect();
        goals
            .into_iter()
            .map(|goal| (goal, self.close(goal, options, view, limits, &mut stop)))
            .collect()
    }

    /// Replaces the open goal by the root of a derivation given premises
    /// before conclusions, appending the rest in reverse, so that every
    /// premise keeps a larger index than its conclusion.
    fn graft(&mut self, goal: InfId, mut found: Vec<Inference>) {
        let root = found.len() - 1;
        let base = self.inferences.len();
        let renumber = |id: InfId| {
            if id.index() == root {
                goal
            } else {
                InfId::new((base + root - 1 - id.index()) as u32)
            }
        };
        for inference in &mut found {
            for premise in &mut inference.premises {
                *premise = renumber(*premise);
            }
        }
        let root = found.pop().unwrap();
        self.inferences[goal.index()] = root;
        self.inferences.extend(found.into_iter().rev());
    }

    /// Returns the derivation so far, with the open goals as leaves of
    /// [`Rule::Open`], two-sided in intuitionistic mode; its inferences are
    /// renumbered premises before conclusions, so its ids are not this
    /// state's: [`derivation_ids`](Self::derivation_ids) maps them.
    pub fn derivation(&self) -> Result<Derivation<'_>, Error> {
        self.derivation_within(&ViewOptions::default(), &Limits::default(), |_| false)
    }

    /// Returns the derivation so far as [`derivation`](Self::derivation)
    /// does, refused past `limits.derivation_bytes` by its estimate, as a
    /// proof's derivation is, and until `stop` fires. Its inferences are
    /// the state's own, never compacted, whatever `view` says.
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
        ))
    }

    /// Returns, for every inference of [`derivation`](Self::derivation) by
    /// its id, the id of the same inference in this state: what turns the
    /// inference `n` of a drawing (the SVG's `i<n>`) into the goal that
    /// [`apply`](Self::apply) and [`close`](Self::close) take.
    pub fn derivation_ids(&self) -> Vec<GoalId> {
        self.postorder().into_iter().map(GoalId::of).collect()
    }

    /// Returns the arena's inferences in postorder, premises before
    /// conclusions, the order of [`derivation`](Self::derivation).
    fn postorder(&self) -> Vec<InfId> {
        let mut order = Vec::with_capacity(self.inferences.len());
        // Postorder without recursion: a frame is an inference and whether
        // its premises were visited.
        let mut stack = vec![(InfId::new(0), false)];
        while let Some((id, visited)) = stack.pop() {
            if visited {
                order.push(id);
            } else {
                stack.push((id, true));
                let premises = &self.inferences[id.index()].premises;
                stack.extend(premises.iter().rev().map(|&p| (p, false)));
            }
        }
        order
    }

    /// Translates the finished derivation into a proof term and checks it,
    /// returning the proof. Fails while goals are open
    /// ([`Error::OpenGoals`]) or if the checker rejects the term
    /// ([`Error::Check`]), which it never does for a derivation built
    /// through this interface, or gives the check up within `limits` or
    /// when `stop` fires ([`Error::Check`] with a
    /// [`CheckError::Refused`](crate::CheckError::Refused)).
    pub fn proof(
        &self,
        limits: &Limits,
        stop: impl FnMut(Progress) -> bool,
    ) -> Result<Proof, Error> {
        let open = self.goals().count();
        if open > 0 {
            return Err(Error::OpenGoals { count: open });
        }
        let mut terms = Terms {
            state: self,
            nodes: Vec::with_capacity(self.inferences.len()),
        };
        let mut root = terms.term(InfId::new(0));
        for &r in self.forest.roots() {
            if self.forest.kind(r) == Kind::Quest {
                root = terms.push(Node::Quest(Member::from(r), root));
            }
        }
        let proof = Proof::new(self.forest.clone(), terms.nodes, root)?.with_mode(self.mode);
        proof.check_within(self.mode, limits, stop)?;
        Ok(proof)
    }
}

/// Returns the formula at a position of a sequent, or the refusal.
fn formula_at(sequent: &[OccId], position: usize) -> Result<OccId, StepError> {
    sequent.get(position).copied().ok_or(StepError::NoFormula {
        position,
        len: sequent.len(),
    })
}

/// The translation of a finished derivation into a proof term, bottom-up
/// over the tree: every rule is its node; a dereliction is a copy, a
/// contraction and a `?w` are nothing, since a `?` formula lives in the
/// unrestricted zone from the point where it enters the derivation, which
/// is where its `?` node goes: below the rule that introduces it as a
/// subformula, or below the root for a `?` formula of the sequent.
struct Terms<'a> {
    /// The finished derivation.
    state: &'a Interactive,
    /// The arena being built, premises before conclusions.
    nodes: Vec<Node>,
}

impl Terms<'_> {
    /// Appends a node and returns its id.
    fn push(&mut self, node: Node) -> NodeId {
        self.nodes.push(node);
        NodeId::new(self.nodes.len() as u32 - 1)
    }

    /// Builds the term of the subtree at `root`, without recursion: a
    /// derivation read from a file can be as high as its formulas are
    /// deep. The steps still to take wait on a stack and the terms of the
    /// subtrees done on another, in the order a recursion would take
    /// them, premises left to right and a rule after its premises.
    fn term(&mut self, root: InfId) -> NodeId {
        use Rule::*;
        /// What is left to do.
        enum Step {
            /// Build the term of the subtree at an inference.
            Visit(InfId),
            /// Put the `?` nodes of the `?` formulas among these, which
            /// the rule below introduced, on the term just built.
            Quests([Option<OccId>; 2]),
            /// Put an inference's own node on the terms of its premises.
            Build(InfId),
        }
        let state = self.state;
        let f = &state.forest;
        let mut steps = vec![Step::Visit(root)];
        let mut done: Vec<NodeId> = Vec::new();
        while let Some(step) = steps.pop() {
            let id = match step {
                Step::Visit(id) | Step::Build(id) => id,
                Step::Quests(introduced) => {
                    let mut node = done.pop().expect("the premise's term");
                    for x in introduced.into_iter().flatten() {
                        if f.kind(x) == Kind::Quest {
                            node = self.push(Node::Quest(Member::from(x), node));
                        }
                    }
                    done.push(node);
                    continue;
                }
            };
            let inference = &state.inferences[id.index()];
            let sequent = &inference.sequent;
            let premises = &inference.premises;
            let o = || {
                sequent[inference
                    .principal
                    .expect("every rule but ax and mix has one")]
            };
            let a = || f.left(o().occ()).expect("a connective with a subformula");
            let b = || f.right(o().occ()).expect("a binary connective");
            let rule = inference.rule.rule;
            if matches!(step, Step::Visit(_)) {
                // What each premise's sequent gains from the rule.
                let introduced: [[Option<OccId>; 2]; 2] = match rule {
                    Ax | One | Top => {
                        let leaf = match rule {
                            Ax => Node::Ax(sequent[0], sequent[1]),
                            One => Node::One(o()),
                            _ => Node::Top(o()),
                        };
                        let leaf = self.push(leaf);
                        done.push(leaf);
                        continue;
                    }
                    // Neither is a node: the subtree's term is the
                    // premise's.
                    Contraction | Weakening => {
                        steps.push(Step::Visit(premises[0]));
                        continue;
                    }
                    Par => [[Some(a()), Some(b())], [None; 2]],
                    Bot | AffineWeakening | Mix => [[None; 2]; 2],
                    PlusLeft | Promotion | Dereliction => [[Some(a()), None], [None; 2]],
                    PlusRight => [[Some(b()), None], [None; 2]],
                    With | Tensor => [[Some(a()), None], [Some(b()), None]],
                    Open => unreachable!("no goal is open"),
                };
                steps.push(Step::Build(id));
                for (&premise, introduced) in premises.iter().zip(introduced).rev() {
                    steps.push(Step::Quests(introduced));
                    steps.push(Step::Visit(premise));
                }
                continue;
            }
            let mut premise = || done.pop().expect("a premise's term");
            let node = match rule {
                Par => Node::Par(o(), premise()),
                Bot => Node::Bot(o(), premise()),
                PlusLeft => Node::Plus(o(), Branch::Left, premise()),
                PlusRight => Node::Plus(o(), Branch::Right, premise()),
                Promotion => Node::Bang(o(), premise()),
                Dereliction => Node::Copy(Member::from(a()), premise()),
                AffineWeakening => Node::Weaken(o(), premise()),
                With | Tensor | Mix => {
                    let (r, l) = (premise(), premise());
                    match rule {
                        With => Node::With(o(), l, r),
                        Tensor => Node::Tensor(o(), l, r),
                        _ => Node::Mix(l, r),
                    }
                }
                _ => unreachable!("a rule with premises"),
            };
            let node = self.push(node);
            done.push(node);
        }
        done.pop().expect("the root's term")
    }
}

#[cfg(all(test, feature = "parse"))]
mod tests {
    use super::*;
    use crate::search::Reason;

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
                assert!(
                    state
                        .apply(goal, &Step::new(0, Rule::One))
                        .unwrap()
                        .is_empty()
                );
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

    /// A goal is closed only by a proof over the session's sequent that
    /// passes the checker in the session's mode: a proof of another
    /// sequent, larger or smaller, and a weakening in a linear session are
    /// refused, and the session stays as it was.
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
        let goal: Vec<OccId> = s.goal(g).unwrap().iter().map(|m| m.occ()).collect();
        let affine = search::prove_goal(
            s.forest(),
            &goal,
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
        assert_eq!(s.goal(l).map(<[Member]>::len), Some(1));
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
