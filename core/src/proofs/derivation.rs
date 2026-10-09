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

use super::check::{self, Allowance, Facts, Observer, State};
use super::multiset::Multiset;
use super::size::{self, Size};
use super::{Branch, Named, Node, NodeId, Proof, Rule};
use crate::Error;
use crate::fragment::Mode;
use crate::hash::HashMap;
use crate::limits::{Limits, Phase, Progress, Refusal, Space};
use crate::occurrences::{Forest, Member, OccId, Reading, Side};
use crate::sequents::Kind;

/// The index of an inference in a derivation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize), serde(transparent))]
pub struct InfId(u32);

impl InfId {
    /// Wraps a raw index.
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    /// Returns the raw index.
    pub const fn get(self) -> u32 {
        self.0
    }

    /// Returns the index as a `usize`, for indexing the inferences.
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

/// How a derivation is shown: the one value that every path which builds
/// a derivation takes, whatever it then draws or writes. The bounds that
/// keep a call from building what the machine cannot hold are the
/// [`Limits`] beside it: `derivation_bytes`, the most bytes a derivation
/// may be estimated to take ([`Size::bytes`]) and still be built (a proof
/// stores a shared subproof once and its derivation repeats it, and every
/// inference carries its whole sequent, so a derivation can be larger than
/// its proof by any factor), and `memory_bytes`, the most the making of
/// one may hold at once, every pass of the checker on the way and the
/// derivation itself by the same estimate.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serialize", serde(default, deny_unknown_fields))]
pub struct ViewOptions {
    /// Whether a run of one structural rule is drawn as one inference.
    pub compact: Compact,
    /// Whether the derivation is one-sided or two-sided.
    pub sides: Sides,
}

/// Whether a derivation is drawn one-sided, as classical linear logic
/// writes it, or two-sided, as intuitionistic linear logic does.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serialize", serde(rename_all = "lowercase"))]
pub enum Sides {
    /// Two-sided exactly when the proof was found in intuitionistic mode
    /// ([`Proof::mode`]), one-sided otherwise.
    #[default]
    Auto,
    /// One-sided.
    One,
    /// Two-sided, under the sequent's intuitionistic reading.
    Two,
}

/// Whether a derivation draws a run of one structural rule, such as the
/// weakenings of every unused `?` formula, as one inference labelled with
/// a star.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serialize", serde(rename_all = "lowercase"))]
pub enum Compact {
    /// Every rule as its own inference, unless the derivation would then
    /// pass a bound of the options and the compact one would not.
    #[default]
    Auto,
    /// Every run as one inference.
    Always,
    /// Every rule as its own inference.
    Never,
}

impl ViewOptions {
    /// Returns the options with another [`compact`](Self::compact).
    #[must_use]
    pub const fn with_compact(self, compact: Compact) -> Self {
        Self { compact, ..self }
    }

    /// Returns the options with another [`sides`](Self::sides).
    #[must_use]
    pub const fn with_sides(self, sides: Sides) -> Self {
        Self { sides, ..self }
    }

    /// Returns whether a derivation of `proof` is two-sided.
    pub(crate) fn two_sided(&self, proof: &Proof) -> bool {
        match self.sides {
            Sides::One => false,
            Sides::Two => true,
            Sides::Auto => proof.mode().is_some_and(Mode::is_intuitionistic),
        }
    }
}

/// One inference of a derivation: the sequent it concludes, the rule, and
/// the inferences of its premises.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Inference {
    /// The sequent concluded, as members in ascending order, one repeated
    /// as often as the sequent holds it.
    pub(crate) sequent: Vec<Member>,
    /// The rule applied, as the derivation names it.
    pub(crate) rule: Named,
    /// The position in `sequent` of the formula the rule introduces or
    /// removes: `None` for an axiom, whose sequent is its two literals, and
    /// for Mix.
    pub(crate) principal: Option<usize>,
    /// The premises, in the rule's order.
    pub(crate) premises: Vec<InfId>,
    /// How many applications of the rule the inference stands for: one,
    /// or more for a run of a structural rule that a compact view draws
    /// as one inference ([`Compact`]), whose principal formula is then
    /// that of the lowest application.
    pub(crate) times: u32,
}

impl Inference {
    /// Returns the sequent concluded, as members in ascending order, one
    /// repeated as often as the sequent holds it.
    pub fn sequent(&self) -> &[Member] {
        &self.sequent
    }

    /// Returns the rule applied, as the derivation names it.
    pub const fn rule(&self) -> Named {
        self.rule
    }

    /// Returns the position in [`sequent`](Self::sequent) of the formula
    /// the rule introduces or removes: `None` for an axiom, whose sequent
    /// is its two literals, for Mix and for an open goal.
    pub const fn principal(&self) -> Option<usize> {
        self.principal
    }

    /// Returns the premises, in the rule's order.
    pub fn premises(&self) -> &[InfId] {
        &self.premises
    }

    /// Returns how many applications of the rule the inference stands for:
    /// one, or more for a run of a structural rule that a compact view
    /// draws as one inference ([`Compact`]), whose principal formula is then
    /// that of the lowest application.
    pub const fn times(&self) -> u32 {
        self.times
    }
}

/// A derivation in the standard sequent calculus: the tree of inferences a
/// proof term stands for, over the proof's forest, one-sided for classical
/// linear logic or two-sided for intuitionistic linear logic. Premises
/// precede their conclusion and the root is the last inference.
///
/// [`Display`](std::fmt::Display) draws the tree, see [`Proof`] for an example.
#[derive(Clone, Debug)]
pub struct Derivation<'a> {
    /// The forest the sequents' occurrences index.
    forest: &'a Forest,
    /// The intuitionistic reading, for a two-sided derivation.
    reading: Option<Reading<'a>>,
    /// The inferences, premises before conclusions, the root last.
    inferences: Vec<Inference>,
    /// Whether the root concludes a goal of a proof rather than the
    /// sequent.
    of_goal: bool,
}

impl<'a> Derivation<'a> {
    /// Unfolds a proof into the one-sided derivation of classical linear
    /// logic, unless it is larger than `view` allows or `stop` fires on
    /// the way, once per inference. The proof must be correct, and the
    /// unfolding fails as [`check`](Proof::check) would if it is not. Mode
    /// is not a question here: a derivation shows every rule the proof
    /// uses.
    pub(crate) fn new(
        proof: &'a Proof,
        view: &ViewOptions,
        limits: &Limits,
        stop: impl FnMut(Progress) -> bool,
    ) -> Result<Self, Error> {
        Self::build(proof, Self::ONE_SIDED, None, view, limits, stop)
    }

    /// The most inferences a derivation holds: as many as an [`InfId`]
    /// counts.
    pub const MOST: u64 = u32::MAX as u64;

    /// The rules of a one-sided derivation: every rule a proof can use.
    pub(crate) const ONE_SIDED: Mode = Mode::CLASSICAL.with_affine().with_mix();

    /// The rules of a two-sided derivation, weakening among them so that
    /// it shows where used.
    pub(crate) const TWO_SIDED: Mode = Mode::INTUITIONISTIC.with_affine();

    /// Unfolds a proof into the two-sided derivation of intuitionistic
    /// linear logic, `Γ ⊢ A` at every inference with the intuitionistic
    /// rule names, unless it is larger than `view` allows or `stop` fires
    /// on the way. The proof must pass the checker in intuitionistic mode
    /// (affine or not), and the unfolding fails as it would otherwise.
    pub(crate) fn two_sided(
        proof: &'a Proof,
        view: &ViewOptions,
        limits: &Limits,
        stop: impl FnMut(Progress) -> bool,
    ) -> Result<Self, Error> {
        let reading = check::reading(proof, Self::TWO_SIDED)?;
        Self::build(proof, Self::TWO_SIDED, reading, view, limits, stop)
    }

    /// Checks the proof in `mode` and unfolds it, two-sided when a reading
    /// is given.
    fn build(
        proof: &'a Proof,
        mode: Mode,
        reading: Option<Reading<'a>>,
        view: &ViewOptions,
        limits: &Limits,
        mut stop: impl FnMut(Progress) -> bool,
    ) -> Result<Self, Error> {
        let conclusion = proof.conclusion();
        let inferences = unfold(
            proof,
            &conclusion,
            mode,
            reading.as_ref(),
            view,
            limits,
            &mut stop,
        )?;
        Ok(Self {
            forest: proof.forest(),
            reading,
            inferences,
            of_goal: proof.goal().is_some(),
        })
    }

    /// Wraps inferences that already have the derivation's shape: premises
    /// before conclusions, the root last, two-sided under a reading.
    pub(crate) fn from_parts(
        forest: &'a Forest,
        reading: Option<Reading<'a>>,
        inferences: Vec<Inference>,
    ) -> Self {
        debug_assert!(!inferences.is_empty());
        Self {
            forest,
            reading,
            inferences,
            of_goal: false,
        }
    }

    /// Returns whether the root concludes a goal of its proof rather than
    /// the sequent.
    pub(crate) const fn is_of_goal(&self) -> bool {
        self.of_goal
    }

    /// Unfolds a proof whose root concludes `goal` rather than the roots,
    /// as the search from a goal returns it, into the inferences of its
    /// derivation, premises before conclusions and the root last, two-sided
    /// in intuitionistic mode, unless it is larger than `view` allows or
    /// `stop` fires on the way. Fails as the checker in `mode` would on a
    /// node that misapplies its rule and on a root that does not conclude
    /// the goal.
    pub(crate) fn of_goal(
        proof: &'a Proof,
        goal: &[OccId],
        mode: Mode,
        view: &ViewOptions,
        limits: &Limits,
        stop: &mut dyn FnMut(Progress) -> bool,
    ) -> Result<Vec<Inference>, Error> {
        let reading = check::reading(proof, mode)?;
        // A graft is read rule by rule, never drawn compact.
        let view = view.with_compact(Compact::Never);
        unfold(proof, goal, mode, reading.as_ref(), &view, limits, stop)
    }

    /// Returns the forest the sequents' members index.
    pub fn forest(&self) -> &'a Forest {
        self.forest
    }

    /// Returns the occurrence a member of the derivation stands for.
    pub fn occurrence(&self, member: Member) -> OccId {
        member.occ()
    }

    /// Returns the formula a member of the derivation stands for, as the
    /// one-sided sequent writes it.
    pub fn formula(&self, member: Member) -> impl std::fmt::Display + 'a {
        self.forest.formula(member.occ())
    }

    /// Returns the intuitionistic reading of a two-sided derivation, or
    /// `None` for a one-sided one.
    pub fn reading(&self) -> Option<&Reading<'a>> {
        self.reading.as_ref()
    }

    /// Returns every inference, premises before conclusions, the root last.
    pub fn inferences(&self) -> &[Inference] {
        &self.inferences
    }

    /// Returns the inference at `id`, which must belong to this derivation.
    ///
    /// # Panics
    ///
    /// For an id of another one, past its end.
    pub fn inference(&self, id: InfId) -> &Inference {
        &self.inferences[id.index()]
    }

    /// Returns the root: the inference that concludes the sequent.
    pub fn root(&self) -> InfId {
        // A derivation has at least one inference and [`MOST`](Self::MOST)
        // at most.
        InfId::new(self.inferences.len() as u32 - 1)
    }
}

/// Unfolds a proof that concludes `goal` into the inferences of its
/// derivation, premises before conclusions and the root last: one pass of
/// the checker for the size, which must be within the bounds, a second for
/// what the translation reads, then the translation. Every pass holds no
/// more than the memory bound, and so does the derivation by its estimate.
/// Where the options ask for the compact view, or allow it and the whole
/// derivation is over a bound, the translation draws runs of a structural
/// rule as one inference and counts what it holds as it goes, giving up
/// with the whole derivation's error once that passes the bound.
fn unfold(
    proof: &Proof,
    goal: &[OccId],
    mode: Mode,
    reading: Option<&Reading>,
    view: &ViewOptions,
    limits: &Limits,
    stop: &mut dyn FnMut(Progress) -> bool,
) -> Result<Vec<Inference>, Error> {
    let allowance = Allowance::new(limits, Phase::View, stop);
    let (size, firm) = size::measured(proof, goal, mode, reading, allowance)?;
    // A view that may compact says what a compact one takes at least.
    let least_bytes = (view.compact != Compact::Never).then_some(firm.bytes);
    let over = if let Some(limit) = limits.derivation_bytes
        && size.bytes() > limit
    {
        Some(Refusal::Output {
            what: "derivation",
            estimate_bytes: size.bytes(),
            limit_bytes: limit,
            least_bytes,
        })
    } else if let Some(limit) = limits.memory_bytes
        && size.bytes() > limit
    {
        Some(Refusal::Memory {
            phase: Phase::View,
            limit_bytes: limit,
            needed_bytes: Some(size.bytes()),
        })
    } else if size.inferences > Derivation::MOST {
        Some(too_many(&size))
    } else {
        None
    };
    let compact = match (view.compact, over.map(Error::Refused)) {
        (Compact::Never, Some(error)) => return Err(error),
        (Compact::Never | Compact::Auto, None) => None,
        (Compact::Always, None) => Some(Budget {
            most: None,
            error: Error::Refused(too_many(&size)),
        }),
        (compact, Some(error)) => {
            let most = [limits.derivation_bytes, limits.memory_bytes]
                .into_iter()
                .flatten()
                .min();
            // What a compact view cannot do without is known before it is
            // built. Without a bound in bytes it is tried only when asked
            // for, since nothing would then end the attempt but the count
            // of inferences.
            if (compact == Compact::Auto && most.is_none())
                || most.is_some_and(|most| firm.bytes > most)
                || firm.inferences > Derivation::MOST
            {
                return Err(error);
            }
            Some(Budget { most, error })
        }
    };
    let mut record = Record::new(proof);
    let allowance = Allowance::new(limits, Phase::View, stop);
    check::examine(proof, goal, mode, reading, allowance, &mut record)?;
    let mut build = Build {
        proof,
        record: &record,
        reading,
        // As many as the size says, which fit a `usize` since they fit a
        // `u32`; a compact view has fewer.
        inferences: match compact {
            None => Vec::with_capacity(size.inferences as usize),
            Some(_) => Vec::new(),
        },
        tasks: Vec::new(),
        done: Vec::new(),
        stop,
        compact: compact.map(|budget| Held {
            weights: size::weights(proof.forest()),
            bytes: 0,
            budget,
        }),
    };
    build.run(proof.root(), Multiset::of(goal.iter().copied()))?;
    Ok(build.inferences)
}

/// Returns the length of a list of distinct nodes or occurrences, which
/// fits a `u32` as their ids do.
fn count(len: usize) -> u32 {
    u32::try_from(len).unwrap_or(u32::MAX)
}

/// The refusal of a derivation of more inferences than one holds.
fn too_many(size: &Size) -> Refusal {
    Refusal::Index {
        what: Space::Inference,
        count: size.inferences,
        most: Derivation::MOST,
    }
}

/// The bound of a compact view, which is not measured before it is built.
struct Budget {
    /// The most bytes the derivation may take by [`Size::bytes`]'s
    /// estimate, or `None` for no bound but [`Derivation::MOST`].
    most: Option<u64>,
    /// The error to give up with: the whole derivation's.
    error: Error,
}

/// What a compact view holds so far, by the estimate of [`Size::bytes`].
struct Held {
    /// The characters of every occurrence's formula and its separator.
    weights: Vec<u32>,
    /// The bytes of the inferences made so far; it saturates.
    bytes: u64,
    /// The bound.
    budget: Budget,
}

impl Held {
    /// Returns the bytes an inference with this sequent takes.
    fn cost(&self, sequent: &[Member]) -> u64 {
        sequent
            .iter()
            .fold(0u64, |sum, o| {
                sum.saturating_add(u64::from(self.weights[o.index()]))
            })
            .saturating_mul(Size::BYTES_PER_CHARACTER)
            .saturating_add(Size::BYTES_PER_INFERENCE)
    }
}

/// What the translation reads off the checker's pass: for every node a
/// flag or two, and the sequent of the nodes where a context is split or
/// padded, which the derivation shows anyway.
struct Record<'a> {
    /// The forest.
    forest: &'a Forest,
    /// Whether a `⊤` in a node's subproof absorbs any context.
    absorbs: Vec<bool>,
    /// For a `?` step, whether a copy above uses its formula; for a copy,
    /// whether another copy above uses the same occurrence.
    used: Vec<bool>,
    /// The nodes whose sequent is kept: every `⊗` and Mix, and the
    /// premises of `⊗`, Mix and `&`.
    kept: Vec<bool>,
    /// The standard sequents of the kept nodes: `?Θ` and `Γ` as the
    /// checker found them.
    standard: HashMap<NodeId, Multiset>,
    /// For a `⊗` or Mix, the unrestricted occurrences both premises need,
    /// ascending, where there are any.
    shared: HashMap<NodeId, Vec<OccId>>,
    /// The bytes of the sequents and lists kept so far, at
    /// [`ENTRY`](Self::ENTRY) each and eight for every member; it
    /// saturates.
    held: u64,
}

impl<'a> Record<'a> {
    /// What a kept sequent or list takes besides its members: its place in
    /// the table and its allocation.
    const ENTRY: u64 = 96;

    /// Counts a list of `members` occurrences as kept: four bytes each,
    /// in a vector that may have room for as many again.
    fn keep(&mut self, members: usize) {
        let bytes = (members as u64).saturating_mul(8);
        self.held = self.held.saturating_add(Self::ENTRY).saturating_add(bytes);
    }

    /// The record of a proof the checker has yet to pass over.
    fn new(proof: &'a Proof) -> Self {
        let len = proof.nodes().len();
        let mut kept = vec![false; len];
        for id in proof.ids() {
            let node = proof.node(id);
            if matches!(node, Node::Tensor(..) | Node::Mix(..)) {
                kept[id.index()] = true;
            }
            if matches!(node, Node::Tensor(..) | Node::Mix(..) | Node::With(..)) {
                for p in node.premises() {
                    kept[p.index()] = true;
                }
            }
        }
        Self {
            forest: proof.forest(),
            absorbs: vec![false; len],
            used: vec![false; len],
            kept,
            standard: HashMap::default(),
            shared: HashMap::default(),
            held: 0,
        }
    }
}

impl Observer for Record<'_> {
    fn bytes(&self) -> u64 {
        // Three flags for each of fewer than 2³² nodes, and what is kept.
        self.held.saturating_add(self.kept.len() as u64 * 3)
    }

    fn derived(&mut self, id: NodeId, state: &State, facts: &Facts<'_>) {
        self.absorbs[id.index()] = state.absorbs();
        self.used[id.index()] = facts.used;
        if self.kept[id.index()] {
            let quests = state.unrestricted().map(|a| self.forest.parent(a).unwrap());
            let linear = state
                .linear()
                .flat_map(|(o, n)| std::iter::repeat_n(o, n as usize));
            let standard = Multiset::of(quests.chain(linear));
            self.keep(standard.as_slice().len());
            self.standard.insert(id, standard);
        }
        if !facts.shared.is_empty() {
            let mut shared = facts.shared.to_vec();
            shared.sort_unstable();
            self.keep(shared.len());
            self.shared.insert(id, shared);
        }
    }
}

/// A step the translation has yet to take. The steps wait on a stack of
/// the translation's own, so a derivation of any height is built on a call
/// stack of any size.
enum Task {
    /// Unfold the subproof at a node under a conclusion: the standard
    /// sequent it derives plus whatever a `⊤` in it absorbs.
    Unfold(NodeId, Multiset),
    /// Unfold the subproof at a node under a conclusion that may hold `?`
    /// formulas the subproof does not use, weakening them above it unless
    /// a `⊤` in it absorbs them.
    Pad(NodeId, Multiset),
    /// Conclude a sequent by a rule from the subtrees finished last, as
    /// many as the rule has premises.
    Infer {
        /// The conclusion.
        sequent: Multiset,
        /// The rule.
        rule: Rule,
        /// The formula it introduces.
        principal: Option<OccId>,
        /// How many premises it has.
        premises: usize,
        /// How many applications of the rule it stands for.
        times: u32,
    },
    /// Weaken `?` formulas one by one below the subtree finished last.
    Weaken {
        /// The conclusion of the subtree.
        sequent: Multiset,
        /// The formulas to add, in ascending order.
        unused: Multiset,
    },
    /// Contract, below the subtree finished last, the `?` formulas that
    /// both premises of the `⊗` or Mix at a node use.
    Contract {
        /// The conclusion of the subtree, which holds each of them twice.
        sequent: Multiset,
        /// The node.
        id: NodeId,
    },
}

/// The translation in progress.
struct Build<'a> {
    /// The proof being unfolded.
    proof: &'a Proof,
    /// What the checker derived for the nodes.
    record: &'a Record<'a>,
    /// The intuitionistic reading, for a two-sided derivation.
    reading: Option<&'a Reading<'a>>,
    /// The inferences made so far.
    inferences: Vec<Inference>,
    /// The steps yet to take, the next one last.
    tasks: Vec<Task>,
    /// The roots of the subtrees that are finished and wait for the
    /// inference below them, the latest last.
    done: Vec<InfId>,
    /// The caller's stop condition, polled once per node.
    stop: &'a mut dyn FnMut(Progress) -> bool,
    /// What a compact view holds and may hold, or `None` for a derivation
    /// that shows every rule.
    compact: Option<Held>,
}

impl<'a> Build<'a> {
    /// The forest.
    fn forest(&self) -> &'a Forest {
        self.proof.forest()
    }

    /// The `?` formula an unrestricted-zone occurrence stands for.
    fn quest(&self, a: OccId) -> OccId {
        self.forest().parent(a).unwrap()
    }

    /// The standard sequent a subproof derives: `?Θ` and `Γ` as the checker
    /// found them.
    fn standard(&self, id: NodeId) -> Multiset {
        self.record.standard[&id].clone()
    }

    /// Whether a `⊤` in the subproof absorbs any context.
    fn absorbs(&self, id: NodeId) -> bool {
        self.record.absorbs[id.index()]
    }

    /// The unrestricted occurrences both premises of the `⊗` or Mix at
    /// `id` need, ascending.
    fn shared(&self, id: NodeId) -> &'a [OccId] {
        self.record.shared.get(&id).map_or(&[], Vec::as_slice)
    }

    /// Unfolds the subproof at `root`, whose conclusion is `conclusion`,
    /// into the inferences, unless the caller's condition stops it.
    fn run(&mut self, root: NodeId, conclusion: Multiset) -> Result<(), Error> {
        self.tasks.push(Task::Unfold(root, conclusion));
        while let Some(task) = self.tasks.pop() {
            // A task adds one inference at most to a compact view, so a
            // view that stops here holds no more than one beyond its
            // bound, and never more than an id counts.
            if let Some(held) = &self.compact {
                let most = held.budget.most.unwrap_or(u64::MAX);
                if held.bytes > most || self.inferences.len() as u64 >= Derivation::MOST {
                    return Err(held.budget.error.clone());
                }
            }
            match task {
                Task::Unfold(id, actual) => {
                    // A node unfolds into an inference or a few.
                    let done = self.inferences.len() as u64;
                    if (self.stop)(Progress::new(Phase::View, 1, done)) {
                        return Err(Error::Refused(Refusal::Stopped { phase: Phase::View }));
                    }
                    self.unfold(id, actual);
                }
                Task::Pad(id, actual) => self.pad(id, actual),
                Task::Infer {
                    sequent,
                    rule,
                    principal,
                    premises,
                    times,
                } => {
                    let premises = self.done.split_off(self.done.len() - premises);
                    let inference = self.infer(sequent, rule, principal, premises, times);
                    self.done.push(inference);
                }
                Task::Weaken { sequent, unused } if self.compact.is_some() => {
                    if let Some(&lowest) = unused.as_slice().last() {
                        let times = count(unused.as_slice().len());
                        self.below(sequent.sum(&unused), Rule::Weakening, lowest, times);
                    }
                }
                Task::Weaken {
                    mut sequent,
                    unused,
                } => {
                    for &q in unused.as_slice() {
                        sequent.insert(q);
                        self.below(sequent.clone(), Rule::Weakening, q, 1);
                    }
                }
                Task::Contract { sequent, id } if self.compact.is_some() => {
                    let shared = self.shared(id);
                    if let Some(&lowest) = shared.last() {
                        let quests = Multiset::of(shared.iter().map(|&a| self.quest(a)));
                        let times = count(shared.len());
                        let sequent = sequent.difference(&quests);
                        self.below(sequent, Rule::Contraction, self.quest(lowest), times);
                    }
                }
                Task::Contract { mut sequent, id } => {
                    for &a in self.shared(id) {
                        let q = self.quest(a);
                        sequent.remove(q);
                        self.below(sequent.clone(), Rule::Contraction, q, 1);
                    }
                }
            }
        }
        Ok(())
    }

    /// Adds an inference that stands for `times` applications of its rule
    /// and returns its id; in a two-sided derivation the rule gets its
    /// intuitionistic name. In a compact view, a structural rule whose
    /// premise is a run of the same rule extends that run instead.
    fn infer(
        &mut self,
        sequent: Multiset,
        rule: Rule,
        principal: Option<OccId>,
        premises: Vec<InfId>,
        times: u32,
    ) -> InfId {
        let rule = match (self.reading, principal) {
            (Some(reading), Some(o)) => rule.on(reading.position(o)),
            _ => rule.into(),
        };
        let principal = principal.map(|o| sequent.position(o).unwrap());
        if let Some(held) = &mut self.compact {
            let sequent = sequent.into_members();
            let cost = held.cost(&sequent);
            held.bytes = held.bytes.saturating_add(cost);
            if let [p] = premises[..]
                && rule.is_structural()
                && self.inferences[p.index()].rule == rule
            {
                // The premise of a rule with one premise is the subtree
                // finished last, so its root is the last inference.
                debug_assert_eq!(p.index() + 1, self.inferences.len());
                let run = &mut self.inferences[p.index()];
                held.bytes = held.bytes.saturating_sub(held.cost(&run.sequent));
                run.sequent = sequent;
                run.principal = principal;
                run.times = run.times.saturating_add(times);
                return p;
            }
            return self.push(Inference {
                sequent,
                rule,
                principal,
                premises,
                times,
            });
        }
        self.push(Inference {
            sequent: sequent.into_members(),
            rule,
            principal,
            premises,
            times,
        })
    }

    /// Adds an inference and returns its id.
    fn push(&mut self, inference: Inference) -> InfId {
        // The size of the derivation was counted before it was begun, and
        // one of more inferences than an id counts was refused.
        let id = u32::try_from(self.inferences.len())
            .ok()
            .filter(|&id| u64::from(id) < Derivation::MOST)
            .expect("a derivation has no more inferences than its size says");
        self.inferences.push(inference);
        InfId::new(id)
    }

    /// Adds an inference without premises: a subtree of its own.
    fn leaf(&mut self, sequent: Multiset, rule: Rule, principal: Option<OccId>) {
        let inference = self.infer(sequent, rule, principal, vec![], 1);
        self.done.push(inference);
    }

    /// Adds an inference of `times` applications on `principal` below the
    /// subtree finished last.
    fn below(&mut self, sequent: Multiset, rule: Rule, principal: OccId, times: u32) {
        let premise = self.done.pop().expect("a subtree is finished");
        let inference = self.infer(sequent, rule, Some(principal), vec![premise], times);
        self.done.push(inference);
    }

    /// Plans an inference that concludes `sequent` on `principal` from the
    /// subproof at `p` under the conclusion `up`.
    fn from(&mut self, sequent: Multiset, rule: Rule, principal: OccId, p: NodeId, up: Multiset) {
        self.tasks.push(Task::Infer {
            sequent,
            rule,
            principal: Some(principal),
            premises: 1,
            times: 1,
        });
        self.tasks.push(Task::Unfold(p, up));
    }

    /// Unfolds the subproof at `id`, whose conclusion is `actual`: the
    /// standard sequent it derives plus whatever a `⊤` in it absorbs. A
    /// leaf is concluded at once; any other rule leaves its premises and
    /// then its own inference as steps to take.
    fn unfold(&mut self, id: NodeId, actual: Multiset) {
        use Node::*;
        let f = self.forest();
        let (left, right) = (
            |o: OccId| f.left(o).unwrap(),
            |o: OccId| f.right(o).unwrap(),
        );
        // The premise's conclusion when the rule adds or removes formulas.
        let above = |actual: &Multiset, removed: &[OccId], added: &[OccId]| {
            let mut up = actual.clone();
            for &o in removed {
                let present = up.remove(o);
                debug_assert!(present);
            }
            for &o in added {
                up.insert(o);
            }
            up
        };
        if self.compact.is_some()
            && let Some(o) = self.weakened(id)
        {
            return self.weaken_run(id, o, actual);
        }
        match self.proof.node(id) {
            Ax(..) => self.leaf(actual, Rule::Ax, None),
            One(o) => self.leaf(actual, Rule::One, Some(o.occ())),
            Top(o) => self.leaf(actual, Rule::Top, Some(o.occ())),
            Bot(o, p) => {
                let o = o.occ();
                let up = above(&actual, &[o], &[]);
                self.from(actual, Rule::Bot, o, p, up);
            }
            Par(o, p) => {
                let o = o.occ();
                let up = above(&actual, &[o], &[left(o), right(o)]);
                self.from(actual, Rule::Par, o, p, up);
            }
            Plus(o, side, p) => {
                let o = o.occ();
                let (chosen, rule) = match side {
                    Branch::Left => (left(o), Rule::PlusLeft),
                    Branch::Right => (right(o), Rule::PlusRight),
                };
                let up = above(&actual, &[o], &[chosen]);
                self.from(actual, rule, o, p, up);
            }
            Bang(o, p) => {
                let o = o.occ();
                let up = above(&actual, &[o], &[left(o)]);
                self.from(actual, Rule::Promotion, o, p, up);
            }
            Weaken(o, p) => {
                let o = o.occ();
                let rule = if f.kind(o) == Kind::Quest {
                    Rule::Weakening
                } else {
                    Rule::AffineWeakening
                };
                let up = above(&actual, &[o], &[]);
                self.from(actual, rule, o, p, up);
            }
            Quest(o, p) => {
                let o = o.occ();
                if self.record.used[id.index()] {
                    // Used above: `?A` in Γ becomes `?A` in Θ, which the
                    // standard sequent does not distinguish.
                    self.tasks.push(Task::Unfold(p, actual));
                } else {
                    let up = above(&actual, &[o], &[]);
                    self.from(actual, Rule::Weakening, o, p, up);
                }
            }
            Copy(a, p) => {
                let a = a.occ();
                let q = self.quest(a);
                if self.record.used[id.index()] {
                    // Used again above: derelict the copy, then contract it
                    // with the `?A` that stays.
                    let up = above(&actual, &[], &[a]);
                    let derelicted = above(&actual, &[], &[q]);
                    self.tasks.push(Task::Infer {
                        sequent: actual,
                        rule: Rule::Contraction,
                        principal: Some(q),
                        premises: 1,
                        times: 1,
                    });
                    self.from(derelicted, Rule::Dereliction, q, p, up);
                } else {
                    let up = above(&actual, &[q], &[a]);
                    self.from(actual, Rule::Dereliction, q, p, up);
                }
            }
            Tensor(o, l, r) => self.split(
                id,
                actual,
                Some((o.occ(), left(o.occ()), right(o.occ()))),
                l,
                r,
            ),
            Mix(l, r) => self.split(id, actual, None, l, r),
            With(o, l, r) => {
                let o = o.occ();
                let up_l = above(&actual, &[o], &[left(o)]);
                let up_r = above(&actual, &[o], &[right(o)]);
                self.tasks.push(Task::Infer {
                    sequent: actual,
                    rule: Rule::With,
                    principal: Some(o),
                    premises: 2,
                    times: 1,
                });
                self.tasks.push(Task::Pad(r, up_r));
                self.tasks.push(Task::Pad(l, up_l));
            }
        }
    }

    /// Plans a `⊗` on `o` with subformulas `a` and `b`, or a Mix, at `id`
    /// with premises `l` and `r`: the context is split as the premises
    /// derived it, an absorbing premise taking what the `⊤` absorbs, and
    /// the `?` formulas both premises use are contracted below the rule.
    fn split(
        &mut self,
        id: NodeId,
        actual: Multiset,
        tensor: Option<(OccId, OccId, OccId)>,
        l: NodeId,
        r: NodeId,
    ) {
        let extra = actual.difference(&self.standard(id));
        let (mut up_l, mut up_r) = (self.standard(l), self.standard(r));
        if let Some((_, a, b)) = tensor {
            up_l.ensure(a);
            up_r.ensure(b);
        }
        if let Some(reading) = self.reading
            && self.absorbs(l)
            && self.absorbs(r)
        {
            // Two-sided: the goal among the absorbed formulas goes to the
            // premise that has none, the hypotheses to the left one.
            let left_has_goal = reading.outputs(up_l.as_slice().iter().copied()) > 0;
            for &o in extra.as_slice() {
                if reading.position(o) == Side::Output && left_has_goal {
                    up_r.insert(o);
                } else {
                    up_l.insert(o);
                }
            }
        } else if self.absorbs(l) {
            up_l = up_l.sum(&extra);
        } else {
            debug_assert!(extra.is_empty() || self.absorbs(r));
            up_r = up_r.sum(&extra);
        }
        // The rule's conclusion: both contexts, and the `⊗` for its
        // subformulas.
        let (rule, principal, sequent) = match tensor {
            Some((o, a, b)) => {
                let (mut rest_l, mut rest_r) = (up_l.clone(), up_r.clone());
                rest_l.remove(a);
                rest_r.remove(b);
                let mut sequent = rest_l.sum(&rest_r);
                sequent.insert(o);
                (Rule::Tensor, Some(o), sequent)
            }
            None => (Rule::Mix, None, up_l.sum(&up_r)),
        };
        let shared = self.shared(id);
        debug_assert_eq!(
            {
                let mut contracted = sequent.clone();
                for &a in shared {
                    contracted.remove(self.quest(a));
                }
                contracted
            },
            actual
        );
        if !shared.is_empty() {
            self.tasks.push(Task::Contract {
                sequent: sequent.clone(),
                id,
            });
        }
        self.tasks.push(Task::Infer {
            sequent,
            rule,
            principal,
            premises: 2,
            times: 1,
        });
        self.tasks.push(Task::Unfold(r, up_r));
        self.tasks.push(Task::Unfold(l, up_l));
    }

    /// Returns the `?` formula the node at `id` weakens, if it is a
    /// weakening: a `?` step whose formula is not used above, or a
    /// weakening of a `?` formula.
    fn weakened(&self, id: NodeId) -> Option<OccId> {
        match self.proof.node(id) {
            Node::Quest(o, _) if !self.record.used[id.index()] => Some(o.occ()),
            Node::Weaken(o, _) if self.forest().kind(o.occ()) == Kind::Quest => Some(o.occ()),
            _ => None,
        }
    }

    /// Plans, in a compact view, the run of weakenings that starts at the
    /// node at `id`, which weakens `o`, under the conclusion `actual`: one
    /// inference for every weakening of the same rule down the chain of
    /// nodes, through the `?` steps whose formula is used above, which
    /// are no inference, without a sequent per step.
    fn weaken_run(&mut self, id: NodeId, o: OccId, actual: Multiset) {
        let rule = |o: OccId| match self.reading {
            Some(reading) => Rule::Weakening.on(reading.position(o)),
            None => Rule::Weakening.into(),
        };
        let first = rule(o);
        let mut removed = vec![o];
        let mut next = self.proof.node(id).premises().next().unwrap();
        loop {
            match self.proof.node(next) {
                Node::Quest(_, p) if self.record.used[next.index()] => next = p,
                _ => match self.weakened(next) {
                    Some(o) if rule(o) == first => {
                        removed.push(o);
                        next = self.proof.node(next).premises().next().unwrap();
                    }
                    _ => break,
                },
            }
        }
        let up = actual.difference(&Multiset::of(removed.iter().copied()));
        self.tasks.push(Task::Infer {
            sequent: actual,
            rule: Rule::Weakening,
            principal: Some(o),
            premises: 1,
            // Distinct nodes, fewer than 2³².
            times: count(removed.len()),
        });
        self.tasks.push(Task::Unfold(next, up));
    }

    /// Plans the subproof at `id` under a conclusion that may hold `?`
    /// formulas the subproof does not use, weakening them above it unless
    /// a `⊤` in it absorbs them.
    fn pad(&mut self, id: NodeId, actual: Multiset) {
        if self.absorbs(id) {
            self.tasks.push(Task::Unfold(id, actual));
            return;
        }
        let sequent = self.standard(id);
        debug_assert!(sequent.is_subset(&actual));
        let unused = actual.difference(&sequent);
        if !unused.is_empty() {
            self.tasks.push(Task::Weaken {
                sequent: sequent.clone(),
                unused,
            });
        }
        self.tasks.push(Task::Unfold(id, sequent));
    }
}

#[cfg(all(test, feature = "parse"))]
mod tests {
    use super::*;
    use crate::Sequent;
    use crate::occurrences::Member;

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
        let mut runs = vec![];
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
    /// principal formula's position and the premises, premises first.
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
        let inference = |sequent: &[u32], rule, principal, premises: &[InfId]| Inference {
            sequent: sequent.iter().map(|&x| o(x)).collect(),
            rule,
            principal,
            premises: premises.to_vec(),
            times: 1,
        };
        assert_eq!(
            d.inferences(),
            [
                inference(&[1, 3], Named::from(Rule::Ax), None, &[]),
                inference(&[0, 3], Named::from(Rule::Dereliction), Some(0), &[i(0)]),
                inference(&[1, 4], Named::from(Rule::Ax), None, &[]),
                inference(&[0, 4], Named::from(Rule::Dereliction), Some(0), &[i(2)]),
                inference(
                    &[0, 0, 2],
                    Named::from(Rule::Tensor),
                    Some(2),
                    &[i(1), i(3)]
                ),
                inference(&[0, 2], Named::from(Rule::Contraction), Some(0), &[i(4)]),
            ]
        );
        assert_eq!(d.root(), i(5));
        assert_eq!(d.inference(i(5)).rule.to_string(), "?c");
        // An invalid proof has no derivation.
        let p = proof("!A |- A * A", vec![Ax(o(1), o(3)), Copy(o(1), n(0))]);
        assert!(p.derivation().is_err());
    }
}
