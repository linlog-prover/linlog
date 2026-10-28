// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Cut elimination on proof terms: a step reduces one cut, the caller's
//! or the strategy's, and says which and by which case; [`eliminate`]
//! takes steps by the strategy until no cut is left or a bound is reached.
//!
//! Every proof a step makes lives over the forest of the proof it was
//! made from, and every cut it makes is on an occurrence of the cut
//! formula's tree: a key case cuts the subformulas of the formula cut,
//! and the other cases cut the occurrence they were given. A step
//! copies nothing in the forest: a contraction or a commutation over `&`
//! makes a second cut of a subproof that both cuts share, and a copy of
//! a box shares the box's subproof, as the arena shares any subproof.
//!
//! The exponentials are the dyadic calculus's (a `?` step moves its
//! formula into the unrestricted zone, a copy uses it from there), so a
//! box `!A` cut against a `?A⊥` that a `?` step took into the zone goes
//! up the other premise with that step: a copy of `A⊥` it meets is a
//! *dereliction* (the box's content cut against the copy's premise), or a
//! *contraction* where the premise still uses `A⊥` (the content cut and
//! the box kept for the uses above); a box it meets it enters; a
//! subproof that does not use `A⊥` *weakens* the box away.
//!
//! A step is checked, not trusted: [`step`] returns a proof the checker
//! accepts for the same conclusion in the mode it was given, and
//! [`eliminate`] checks every proof it reaches before its next step and
//! the last one before it returns.

use crate::Error;
use crate::fragment::Mode;
use crate::hash::{HashMap, HashSet};
use crate::limits::{Limits, Phase, Progress, Refusal};
use crate::occurrences::{Forest, Member, OccId, Reading, Side};
use crate::proofs::check::{self, Allowance, CheckError, Facts, Observer, State};
use crate::proofs::multiset::Multiset;
use crate::proofs::{Branch, Node, NodeId, Proof};

/// How an elimination proceeds and when it stops.
///
/// # JSON
///
/// With the feature `serialize` an object of the fields, the bounds a
/// number or `null`; a missing field takes its default and a misspelt one
/// is refused.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serialize",
    derive(serde::Serialize, serde::Deserialize),
    serde(default, deny_unknown_fields)
)]
pub struct Options {
    /// Which cut a step reduces when the caller names none.
    pub strategy: Strategy,
    /// The most steps [`eliminate`] takes.
    #[cfg_attr(feature = "serialize", serde(with = "crate::serialize::exact"))]
    pub steps: Option<u64>,
    /// The most nodes a proof a step makes may have; on a proof net
    /// ([`nets::cut`](crate::nets::cut)), the most vertices a net a step
    /// makes may have.
    #[cfg_attr(feature = "serialize", serde(with = "crate::serialize::exact"))]
    pub nodes: Option<u64>,
    /// For a front end that draws the net of every step of an elimination
    /// on a net, the most bytes those drawings take together; past it the
    /// steps are taken undrawn. The library draws none itself.
    #[cfg_attr(feature = "serialize", serde(with = "crate::serialize::exact"))]
    pub drawn_bytes: Option<u64>,
}

impl Options {
    /// The default bound on steps: ten thousand.
    pub const DEFAULT_STEPS: u64 = 10_000;
    /// The default bound on a proof's nodes: a hundred thousand, 1.6 MB
    /// of arena.
    pub const DEFAULT_NODES: u64 = 100_000;
    /// The default bound on the bytes the drawings of an elimination's
    /// steps take together: 256 MiB, four drawings at the default bound on
    /// one.
    pub const DEFAULT_DRAWN_BYTES: u64 = 256 << 20;

    /// Returns the options with another [`strategy`](Self::strategy).
    #[must_use]
    pub const fn with_strategy(self, strategy: Strategy) -> Self {
        Self { strategy, ..self }
    }

    /// Returns the options with another bound on [`steps`](Self::steps).
    #[must_use]
    pub const fn with_steps(self, steps: Option<u64>) -> Self {
        Self { steps, ..self }
    }

    /// Returns the options with another bound on [`nodes`](Self::nodes).
    #[must_use]
    pub const fn with_nodes(self, nodes: Option<u64>) -> Self {
        Self { nodes, ..self }
    }

    /// Returns the options with another bound on
    /// [`drawn_bytes`](Self::drawn_bytes).
    #[must_use]
    pub const fn with_drawn_bytes(self, drawn_bytes: Option<u64>) -> Self {
        Self {
            drawn_bytes,
            ..self
        }
    }
}

impl Default for Options {
    /// The strategy [`Strategy::Topmost`] within
    /// [`DEFAULT_STEPS`](Self::DEFAULT_STEPS),
    /// [`DEFAULT_NODES`](Self::DEFAULT_NODES) and
    /// [`DEFAULT_DRAWN_BYTES`](Self::DEFAULT_DRAWN_BYTES).
    fn default() -> Self {
        Self {
            strategy: Strategy::default(),
            steps: Some(Self::DEFAULT_STEPS),
            nodes: Some(Self::DEFAULT_NODES),
            drawn_bytes: Some(Self::DEFAULT_DRAWN_BYTES),
        }
    }
}

/// Which cut a step reduces when the caller names none.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serialize",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
pub enum Strategy {
    /// The cut that comes first in the proof's arena, so that no cut
    /// stands above it: a cut whose premises are cut-free is reduced by
    /// its own steps and those of the cuts they make, each on a smaller
    /// formula or between smaller premises, which is the classical
    /// argument that elimination ends.
    #[default]
    Topmost,
}

impl Strategy {
    /// Returns the cut of `proof` the strategy reduces next, or `None`
    /// for a cut-free proof.
    pub fn pick(self, proof: &Proof) -> Option<NodeId> {
        match self {
            Self::Topmost => proof
                .ids()
                .find(|&id| matches!(proof.node(id), Node::Cut(..))),
        }
    }
}

/// What a step did to the cut it reduced, by the usual name of the case.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serialize",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
pub enum Case {
    /// A premise is an axiom on the cut formula: the other premise takes
    /// the axiom's other literal in its place.
    Axiom,
    /// `A ⊗ B` against `A⊥ ⅋ B⊥`: two cuts, on `A` and on `B`.
    TensorPar,
    /// `1` against `⊥`: the `⊥` premise's own premise.
    OneBot,
    /// `A & B` against `A⊥ ⊕ B⊥`: one cut, on the subformula the `⊕`
    /// chose.
    WithPlus,
    /// A weakened formula: a box cut against a `?A⊥` that the rest of the
    /// subproof does not use, or that a `?w` brought in, is erased; in
    /// affine mode a weakened cut formula erases the other premise, whose
    /// formulas are weakened in.
    Weakening,
    /// A box cut against the last copy of `A⊥`: the box's content cut
    /// against the copy's premise.
    Dereliction,
    /// A box cut against a copy of `A⊥` whose premise uses `A⊥` again, or
    /// against a rule both of whose premises use it: the box's content cut
    /// against the copy's premise and the box kept for the other uses, or
    /// the box cut against each premise.
    Contraction,
    /// A box cut against a box whose content uses `A⊥`: the cut enters
    /// that box.
    BoxIntoBox,
    /// The cut moves above a rule of one premise that does not act on the
    /// cut formula, into the premise of it that holds the formula, or into
    /// both premises of `&`; above `⊤`, which absorbs the cut formula, the
    /// cut and the other premise are gone.
    Commutative,
}

impl Case {
    /// Every case, in the order of declaration.
    pub const ALL: &'static [Self] = &[
        Self::Axiom,
        Self::TensorPar,
        Self::OneBot,
        Self::WithPlus,
        Self::Weakening,
        Self::Dereliction,
        Self::Contraction,
        Self::BoxIntoBox,
        Self::Commutative,
    ];

    /// Returns the case's name as a phrase: `the axiom case`, `the key
    /// case of ⊗ and ⅋`, `weakening`, `a box entering a box`, …
    pub const fn name(self) -> &'static str {
        match self {
            Self::Axiom => "the axiom case",
            Self::TensorPar => "the key case of ⊗ and ⅋",
            Self::OneBot => "the key case of 1 and ⊥",
            Self::WithPlus => "the key case of & and ⊕",
            Self::Weakening => "weakening",
            Self::Dereliction => "dereliction",
            Self::Contraction => "contraction",
            Self::BoxIntoBox => "a box entering a box",
            Self::Commutative => "a commutative case",
        }
    }
}

/// One step of an elimination: which cut it reduced and how.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Step {
    /// The cut reduced, a node of the proof the step was taken on.
    pub cut: NodeId,
    /// The occurrence of the cut formula it cut.
    pub formula: Member,
    /// The case.
    pub case: Case,
    /// The node of the proof the step was taken on where the step acted
    /// besides the cut, if any: the rule a commutative case moved the cut
    /// above, the copy a dereliction or a contraction opened, the box a
    /// box entered.
    pub above: Option<NodeId>,
}

/// A step and the proof it made.
#[non_exhaustive]
#[derive(Clone, Debug)]
pub struct Reduction {
    /// The proof after the step, over the same forest and of the same
    /// conclusion, which the checker accepted in the step's mode.
    pub proof: Proof,
    /// What the step did.
    pub step: Step,
}

/// Why an elimination ended.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum End {
    /// No cut is left.
    CutFree,
    /// A bound or the stop ended it, with the proof reached so far.
    Refused(Refusal),
}

/// The outcome of an elimination: the last proof it reached, the steps
/// that led there and why it ended.
///
/// # JSON
///
/// With the feature `serialize`, written only: `{"version": 6, "proof":
/// {…}, "steps": [{"cut": 7, "formula": 9, "case": "tensor_par"}, …],
/// "end": "cut_free"}`, the proof in [`Proof`]'s form without `version`,
/// each step's `cut` a node of the proof it was taken on, `formula` the
/// occurrence it cut, `case` the [`Case`] in snake case and `above` the
/// node [`Step::above`] names where there is one; `end` is `cut_free`, or
/// `refused` with `reason`, the refusal in an error's form. The level is
/// the proof's, 6 for a sequent with cut formulas
/// ([`wire`](crate::wire)).
#[non_exhaustive]
#[derive(Clone, Debug)]
pub struct Elimination {
    /// The last proof reached, which the checker accepted: cut-free when
    /// the elimination ended at [`End::CutFree`].
    pub proof: Proof,
    /// The steps taken, in order, each on the proof the step before made.
    pub steps: Vec<Step>,
    /// Why the elimination ended.
    pub end: End,
}

/// Reduces one cut of `proof`, the one at `at`, or the one the strategy
/// of `options` picks: checks the proof in `mode`, takes the step and
/// checks the proof it made, each check within `limits` and asking
/// `stop`.
///
/// # Errors
///
/// The checker's error for a proof that is not one in `mode`;
/// [`Error::NoCut`] for an `at` that is no cut, or a proof without one;
/// [`Refusal::Elimination`] for a proof made
/// past `options.nodes`, and the refusals of the checks;
/// [`Error::Reduction`] where the checker refuses the proof made, a
/// defect of this crate.
pub fn step(
    proof: &Proof,
    mode: Mode,
    at: Option<NodeId>,
    options: &Options,
    limits: &Limits,
    mut stop: impl FnMut(Progress) -> bool,
) -> Result<Reduction, Error> {
    let record = Record::of(proof, mode, limits, &mut stop).map_err(Error::Check)?;
    let cut = match at {
        Some(at) if matches!(proof.nodes().get(at.index()), Some(Node::Cut(..))) => at,
        Some(at) => {
            return Err(Error::NoCut {
                node: Some(at.get()),
            });
        }
        None => options
            .strategy
            .pick(proof)
            .ok_or(Error::NoCut { node: None })?,
    };
    let (made, step) =
        reduce(proof, &record, mode, cut, options, limits, &mut stop).map_err(Error::Refused)?;
    let reading = check::reading(&made, mode).map_err(Error::Check)?;
    let allowance = Allowance::new(limits, Phase::Eliminate, &mut stop);
    match check::examine(
        &made,
        &made.conclusion(),
        mode,
        reading.as_ref(),
        allowance,
        &mut (),
    ) {
        Ok(()) => Ok(Reduction { proof: made, step }),
        Err(refused @ CheckError::Refused(_)) => Err(Error::Check(refused)),
        Err(invalid) => Err(Error::Reduction(Box::new(invalid))),
    }
}

/// Eliminates the cuts of `proof` by the strategy of `options`, step by
/// step, until none is left or a bound of `options` or `limits` or the
/// stop ends it: the proof reached is then returned with its cuts and the
/// refusal that ended it. Every proof reached is checked in `mode` before
/// the next step is taken on it, and the last before it is returned.
/// `limits` bound each check; the stop is asked between steps and by the
/// checks.
///
/// # Errors
///
/// The checker's error for a proof that is not one in `mode`, or its
/// refusal of the first check; [`Error::Reduction`] where the checker
/// refuses a proof a step made, a defect of this crate.
pub fn eliminate(
    proof: &Proof,
    mode: Mode,
    options: &Options,
    limits: &Limits,
    mut stop: impl FnMut(Progress) -> bool,
) -> Result<Elimination, Error> {
    let mut steps: Vec<Step> = Vec::new();
    // The proof reached last and the one before it, which is the last one
    // checked while the latest waits for its check.
    let mut current = proof.clone();
    let mut checked: Option<Proof> = None;
    loop {
        let ended = |proof: Proof, steps: Vec<Step>, refusal: Refusal| Elimination {
            proof,
            steps,
            end: End::Refused(refusal),
        };
        let record = match Record::of(&current, mode, limits, &mut stop) {
            Ok(record) => record,
            Err(CheckError::Refused(refused)) => {
                let Some(checked) = checked else {
                    return Err(Error::Check(CheckError::Refused(refused)));
                };
                // The latest proof is unchecked: the one before it is the
                // proof reached.
                steps.pop();
                return Ok(ended(checked, steps, refused.refusal));
            }
            Err(invalid) if checked.is_none() => return Err(Error::Check(invalid)),
            Err(invalid) => return Err(Error::Reduction(Box::new(invalid))),
        };
        let Some(cut) = options.strategy.pick(&current) else {
            return Ok(Elimination {
                proof: current,
                steps,
                end: End::CutFree,
            });
        };
        if let Some(limit) = options.steps
            && steps.len() as u64 >= limit
        {
            let refusal = Refusal::Elimination {
                what: crate::limits::Bound::Steps,
                limit,
            };
            return Ok(ended(current, steps, refusal));
        }
        if stop(Progress::new(Phase::Eliminate, 1, steps.len() as u64)) {
            let refusal = Refusal::Stopped {
                phase: Phase::Eliminate,
            };
            return Ok(ended(current, steps, refusal));
        }
        match reduce(&current, &record, mode, cut, options, limits, &mut stop) {
            Ok((made, step)) => {
                steps.push(step);
                checked = Some(std::mem::replace(&mut current, made));
            }
            Err(refusal) => return Ok(ended(current, steps, refusal)),
        }
    }
}

/// What the checker derived of a proof that a step reads: whether each
/// node's subproof absorbs any context through a `⊤`, and the linear zone
/// of every premise of a `⊗`, a Mix or a cut, as the checker derived it.
struct Record {
    /// Whether a `⊤` in a node's subproof absorbs any context.
    absorbs: Vec<bool>,
    /// Whether a node is a premise of a `⊗`, a Mix or a cut.
    kept: Vec<bool>,
    /// The linear zones of the kept nodes.
    zones: HashMap<NodeId, Multiset>,
    /// The bytes of the zones kept so far; it saturates.
    held: u64,
}

impl Record {
    /// What a kept zone takes besides its members.
    const ENTRY: u64 = 96;

    /// Checks `proof` in `mode` within `limits` and returns what the check
    /// derived, or the checker's error.
    fn of(
        proof: &Proof,
        mode: Mode,
        limits: &Limits,
        stop: &mut dyn FnMut(Progress) -> bool,
    ) -> Result<Self, CheckError> {
        let len = proof.nodes().len();
        let mut kept = vec![false; len];
        for id in proof.ids() {
            if let node @ (Node::Tensor(..) | Node::Mix(..) | Node::Cut(..)) = proof.node(id) {
                for p in node.premises() {
                    kept[p.index()] = true;
                }
            }
        }
        let mut record = Self {
            absorbs: vec![false; len],
            kept,
            zones: HashMap::default(),
            held: 0,
        };
        let reading = check::reading(proof, mode)?;
        check::examine(
            proof,
            &proof.conclusion(),
            mode,
            reading.as_ref(),
            Allowance::new(limits, Phase::Eliminate, stop),
            &mut record,
        )?;
        Ok(record)
    }

    /// Returns the linear zone the checker derived for a premise of a
    /// `⊗`, a Mix or a cut.
    fn zone(&self, id: NodeId) -> &Multiset {
        &self.zones[&id]
    }
}

impl Observer for Record {
    fn bytes(&self) -> u64 {
        // Two flags for each of fewer than 2³² nodes, and the zones.
        self.held.saturating_add(self.kept.len() as u64 * 2)
    }

    fn derived(&mut self, id: NodeId, state: &State, _: &Facts<'_>) {
        self.absorbs[id.index()] = state.absorbs();
        if self.kept[id.index()] {
            let zone = Multiset::of(
                state
                    .linear()
                    .flat_map(|(o, n)| std::iter::repeat_n(o, n as usize)),
            );
            let bytes = (zone.as_slice().len() as u64).saturating_mul(4);
            self.held = self.held.saturating_add(Self::ENTRY).saturating_add(bytes);
            self.zones.insert(id, zone);
        }
    }
}

/// Reduces the cut at `cut` of a checked proof whose record is `record`
/// and returns the proof made, not yet checked, and the step; or the
/// refusal of a proof past the bounds, or of the stop, which the one
/// rewrite that may make more nodes than the proof has asks.
fn reduce(
    proof: &Proof,
    record: &Record,
    mode: Mode,
    cut: NodeId,
    options: &Options,
    limits: &Limits,
    stop: &mut dyn FnMut(Progress) -> bool,
) -> Result<(Proof, Step), Refusal> {
    // The proof was checked in `mode`, so it has a reading there.
    let reading = check::reading(proof, mode).unwrap_or_default();
    let mut rewrite = Rewrite {
        proof,
        forest: proof.forest(),
        record,
        reading,
        nodes: proof.nodes().to_vec(),
        overflowed: false,
    };
    // Every rewrite but the specialized erasure adds fewer nodes than the
    // proof has for each of its passes, so only that one is bounded on the
    // way, by the nodes, the memory and the stop; the proof made is
    // measured after.
    let most = options
        .nodes
        .map_or(usize::MAX, |n| usize::try_from(n).unwrap_or(usize::MAX));
    let (made, step) = rewrite.reduce(cut, mode);
    let root = match made {
        Made::Subproof(replacement) => rewrite.replace(cut, replacement),
        Made::Erasure { kept, proved } => {
            rewrite.specialize(cut, kept, &proved, most, limits.memory_bytes, stop)?
        }
    };
    if rewrite.overflowed {
        return Err(Refusal::Index {
            what: crate::limits::Space::Node,
            count: rewrite.nodes.len() as u64,
            most: u64::from(u32::MAX),
        });
    }
    let nodes = rewrite.nodes;
    let made = Proof::new(proof.forest.clone(), nodes, root).map_err(|error| match error {
        Error::Refused(refusal) => refusal,
        error => unreachable!("a rewrite keeps the arena's order: {error}"),
    })?;
    let count = made.nodes().len() as u64;
    if let Some(limit) = options.nodes
        && count > limit
    {
        return Err(Refusal::Elimination {
            what: crate::limits::Bound::Nodes,
            limit,
        });
    }
    let bytes = count.saturating_mul(size_of::<Node>() as u64);
    if let Some(limit) = limits.memory_bytes
        && bytes > limit
    {
        return Err(Refusal::Memory {
            phase: Phase::Eliminate,
            limit_bytes: limit,
            needed_bytes: Some(bytes),
        });
    }
    let made = Proof {
        goal: proof.goal.clone(),
        mode: Some(mode),
        ..made
    };
    Ok((made, step))
}

/// What a reduction makes of its cut.
enum Made {
    /// A subproof of the cut's conclusion, which takes the cut's place
    /// wherever the proof reads it.
    Subproof(NodeId),
    /// The weakenings of the context every place the proof reads the cut
    /// at gives it, over `kept`, which proves `proved` of that context.
    Erasure {
        /// The subproof kept.
        kept: NodeId,
        /// What it proves.
        proved: Multiset,
    },
}

/// A proof's arena as a step rewrites it: the proof's nodes, and after
/// them the nodes the step adds. A node is never changed in place, since
/// others may share it; a changed node is a new one.
struct Rewrite<'a> {
    /// The proof the step is taken on.
    proof: &'a Proof,
    /// Its forest.
    forest: &'a Forest,
    /// What the checker derived of it.
    record: &'a Record,
    /// The intuitionistic reading, in an intuitionistic mode.
    reading: Option<Reading<'a>>,
    /// The arena.
    nodes: Vec<Node>,
    /// Whether the arena passed what a node id names, so that the step is
    /// refused rather than made of ids that alias.
    overflowed: bool,
}

impl Rewrite<'_> {
    /// Adds a node and returns its id. The arena grows by a few times the
    /// proof's nodes at most, which [`reduce`] measures after; past what a
    /// node id names the id returned is a placeholder and the step is
    /// refused.
    fn push(&mut self, node: Node) -> NodeId {
        let id = match u32::try_from(self.nodes.len()) {
            Ok(id) if id < u32::MAX => id,
            _ => {
                self.overflowed = true;
                u32::MAX
            }
        };
        self.nodes.push(node);
        NodeId::new(id)
    }

    /// Returns the node at `id`.
    fn node(&self, id: NodeId) -> Node {
        self.nodes[id.index()]
    }

    /// Adds the cut of `a` from `first` and `second`.
    fn cut(&mut self, a: OccId, first: NodeId, second: NodeId) -> NodeId {
        self.push(Node::Cut(a.into(), first, second))
    }

    /// Returns whether the derived zone of `id`, a premise of a `⊗`, a Mix
    /// or a cut, holds `x` beyond the occurrence `consumed` its rule takes
    /// from it.
    fn holds(&self, id: NodeId, x: OccId, consumed: Option<OccId>) -> bool {
        let zone = self.record.zone(id).as_slice();
        let count = zone.partition_point(|&o| o <= x) - zone.partition_point(|&o| o < x);
        count > usize::from(consumed == Some(x))
    }

    /// Returns the premise of a two-premise rule, `first` or `second`, that
    /// an occurrence `x` of its conclusion comes from: the one whose zone
    /// holds it beyond what the rule takes from it, else one whose `⊤`
    /// absorbs it ([`absorber`](Self::absorber)).
    fn holder(
        &self,
        x: OccId,
        first: (NodeId, Option<OccId>),
        second: (NodeId, Option<OccId>),
    ) -> Branch {
        if self.holds(first.0, x, first.1) {
            Branch::Left
        } else if self.holds(second.0, x, second.1) {
            Branch::Right
        } else {
            self.absorber(self.is_output(x), first, second)
        }
    }

    /// Returns the premise of a two-premise rule whose `⊤` takes formulas
    /// its zone lacks: the first that absorbs any, unless they include a
    /// formula on the right of `⊢` (`output`), which a premise with one
    /// there already cannot take.
    fn absorber(
        &self,
        output: bool,
        first: (NodeId, Option<OccId>),
        second: (NodeId, Option<OccId>),
    ) -> Branch {
        let takes = |(id, taken): (NodeId, Option<OccId>)| {
            self.record.absorbs[id.index()] && !(output && self.has_output(id, taken))
        };
        if takes(first) || !takes(second) {
            Branch::Left
        } else {
            Branch::Right
        }
    }

    /// Returns whether an occurrence stands right of `⊢` under the reading.
    fn is_output(&self, x: OccId) -> bool {
        self.reading
            .as_ref()
            .is_some_and(|reading| reading.position(x) == Side::Output)
    }

    /// Returns whether the sequent of `id`, a premise of a `⊗`, a Mix or a
    /// cut whose rule takes `taken` from it, has its formula right of `⊢`:
    /// in its derived zone, or as the formula taken where a `⊤` absorbed
    /// it.
    fn has_output(&self, id: NodeId, taken: Option<OccId>) -> bool {
        self.reading.is_some()
            && (self
                .record
                .zone(id)
                .as_slice()
                .iter()
                .any(|&o| self.is_output(o))
                || taken.is_some_and(|t| self.is_output(t)))
    }

    /// Returns the two premises of a two-premise node and what its rule
    /// takes from each: `None` for any other node.
    fn split(&self, node: Node) -> Option<[(NodeId, Option<OccId>); 2]> {
        let f = self.forest;
        match node {
            Node::Tensor(o, p, q) => Some([(p, f.left(o.occ())), (q, f.right(o.occ()))]),
            Node::Cut(b, p, q) => Some([(p, Some(b.occ())), (q, f.dual(b.occ()))]),
            Node::Mix(p, q) => Some([(p, None), (q, None)]),
            _ => None,
        }
    }

    /// Reduces the cut at `cut` and returns what it makes and the step.
    fn reduce(&mut self, cut: NodeId, mode: Mode) -> (Made, Step) {
        let Node::Cut(member, l, r) = self.node(cut) else {
            unreachable!("the node reduced is a cut")
        };
        let f = self.forest;
        let a = member.occ();
        let da = f.dual(a).expect("a cut is on a cut formula's tree");
        let (nl, nr) = (self.node(l), self.node(r));
        let step = |case, above| Step {
            cut,
            formula: member,
            case,
            above,
        };
        if let Node::Ax(x, y) = nl {
            let other = if x.occ() == a { y } else { x };
            let made = self.rename(r, da, other.occ());
            return (Made::Subproof(made), step(Case::Axiom, None));
        }
        if let Node::Ax(x, y) = nr {
            let other = if x.occ() == da { y } else { x };
            let made = self.rename(l, a, other.occ());
            return (Made::Subproof(made), step(Case::Axiom, None));
        }
        if mode.affine {
            if let Node::Weaken(w, k) = nl
                && w.occ() == a
            {
                return (self.erase(k, l, a, r, da), step(Case::Weakening, None));
            }
            if let Node::Weaken(w, k) = nr
                && w.occ() == da
            {
                return (self.erase(k, r, da, l, a), step(Case::Weakening, None));
            }
        }
        if !principal(nl, a) {
            let made = self.commute(cut, Branch::Left);
            return (Made::Subproof(made), step(Case::Commutative, Some(l)));
        }
        if !principal(nr, da) {
            let made = self.commute(cut, Branch::Right);
            return (Made::Subproof(made), step(Case::Commutative, Some(r)));
        }
        let (left, right) = (
            |o: OccId| f.left(o).unwrap(),
            |o: OccId| f.right(o).unwrap(),
        );
        let child = |o: OccId, side: Branch| match side {
            Branch::Left => left(o),
            Branch::Right => right(o),
        };
        let key = |made, case| (Made::Subproof(made), step(case, None));
        match (nl, nr) {
            (Node::Tensor(_, l1, l2), Node::Par(_, r1)) => {
                let inner = self.cut(right(a), l2, r1);
                key(self.cut(left(a), l1, inner), Case::TensorPar)
            }
            (Node::Par(_, l1), Node::Tensor(_, r1, r2)) => {
                let inner = self.cut(right(a), l1, r2);
                key(self.cut(left(a), inner, r1), Case::TensorPar)
            }
            (Node::One(_), Node::Bot(_, r1)) => key(r1, Case::OneBot),
            (Node::Bot(_, l1), Node::One(_)) => key(l1, Case::OneBot),
            (Node::With(_, l1, l2), Node::Plus(_, side, r1)) => {
                let premise = if side == Branch::Left { l1 } else { l2 };
                key(self.cut(child(a, side), premise, r1), Case::WithPlus)
            }
            (Node::Plus(_, side, l1), Node::With(_, r1, r2)) => {
                let premise = if side == Branch::Left { r1 } else { r2 };
                key(self.cut(child(a, side), l1, premise), Case::WithPlus)
            }
            (Node::Bang(..), Node::Weaken(_, r1)) => key(r1, Case::Weakening),
            (Node::Weaken(_, l1), Node::Bang(..)) => key(l1, Case::Weakening),
            (Node::Bang(_, l1), Node::Quest(_, r1)) => {
                let (made, case, above) = self.exponential(cut, Branch::Left, l1, r1);
                (Made::Subproof(made), step(case, above))
            }
            (Node::Quest(_, l1), Node::Bang(_, r1)) => {
                let (made, case, above) = self.exponential(cut, Branch::Right, r1, l1);
                (Made::Subproof(made), step(case, above))
            }
            (nl, nr) => unreachable!("no rule of a checked proof gives {nl} against {nr}"),
        }
    }

    /// Moves the cut at `cut` above the last rule of its premise `side`,
    /// which does not act on the cut formula, and returns the subproof
    /// made.
    fn commute(&mut self, cut: NodeId, side: Branch) -> NodeId {
        let Node::Cut(a, l, r) = self.node(cut) else {
            unreachable!("the node reduced is a cut")
        };
        let (premise, x) = match side {
            Branch::Left => (l, a.occ()),
            Branch::Right => (r, self.forest.dual(a.occ()).unwrap()),
        };
        let cut_with = |this: &mut Self, p: NodeId| match side {
            Branch::Left => this.cut(a.occ(), p, r),
            Branch::Right => this.cut(a.occ(), l, p),
        };
        let node = self.node(premise);
        if let Node::Top(_) = node {
            // The `⊤` absorbs the cut formula, and with it the other
            // premise's context.
            return premise;
        }
        if let Node::With(o, p, q) = node {
            let (p, q) = (cut_with(self, p), cut_with(self, q));
            return self.push(Node::With(o, p, q));
        }
        if let Some([(p, tp), (q, tq)]) = self.split(node) {
            let made = match self.holder(x, (p, tp), (q, tq)) {
                Branch::Left => {
                    let p = cut_with(self, p);
                    node.with_premises(p, q)
                }
                Branch::Right => {
                    let q = cut_with(self, q);
                    node.with_premises(p, q)
                }
            };
            return self.push(made);
        }
        let [p] = node.premises().collect::<Vec<_>>()[..] else {
            unreachable!("{node} acts on the cut formula or has one premise")
        };
        let p = cut_with(self, p);
        self.push(node.map_premises(|_| p))
    }

    /// The exponential cases of a box at `cut`'s premise `boxed` (`Left`
    /// for `!A` in the first premise) whose content is `content`, against
    /// the `?` step of the other premise, whose premise is `quest`. Returns
    /// the subproof made, the case and the node acted at.
    fn exponential(
        &mut self,
        cut: NodeId,
        boxed: Branch,
        content: NodeId,
        quest: NodeId,
    ) -> (NodeId, Case, Option<NodeId>) {
        let Node::Cut(a, l, r) = self.node(cut) else {
            unreachable!("the node reduced is a cut")
        };
        let f = self.forest;
        let a = a.occ();
        // The box's content proves `inside`; the other premise uses its
        // dual `x` from the unrestricted zone.
        let inside = f.left(a).unwrap();
        let x = match boxed {
            Branch::Left => f.dual(inside).unwrap(),
            Branch::Right => inside,
        };
        let uses = self.uses(quest, x);
        // The cut of the box against the `?` step over `p`, which keeps the
        // box for the uses of `x` in `p`.
        let wrap = |this: &mut Self, p: NodeId| {
            if !uses[p.index()] {
                return p;
            }
            match boxed {
                Branch::Left => {
                    let Node::Quest(q, _) = this.node(r) else {
                        unreachable!("the other premise is a ? step")
                    };
                    let p = this.push(Node::Quest(q, p));
                    this.cut(a, l, p)
                }
                Branch::Right => {
                    let Node::Quest(q, _) = this.node(l) else {
                        unreachable!("the other premise is a ? step")
                    };
                    let p = this.push(Node::Quest(q, p));
                    this.cut(a, p, r)
                }
            }
        };
        // The cut of the box's content against a premise `p` that holds `x`.
        let open = |this: &mut Self, p: NodeId| match boxed {
            Branch::Left => this.cut(inside, content, p),
            Branch::Right => this.cut(inside, p, content),
        };
        if !uses[quest.index()] {
            return (quest, Case::Weakening, None);
        }
        let node = self.node(quest);
        match node {
            Node::Copy(o, p) if o.occ() == x => {
                if uses[p.index()] {
                    let kept = wrap(self, p);
                    (open(self, kept), Case::Contraction, Some(quest))
                } else {
                    (open(self, p), Case::Dereliction, Some(quest))
                }
            }
            Node::Bang(o, p) => {
                let p = wrap(self, p);
                (self.push(Node::Bang(o, p)), Case::BoxIntoBox, Some(quest))
            }
            Node::With(o, p, q) => {
                let (p, q) = (wrap(self, p), wrap(self, q));
                (
                    self.push(Node::With(o, p, q)),
                    Case::Commutative,
                    Some(quest),
                )
            }
            Node::Tensor(_, p, q) | Node::Mix(p, q) | Node::Cut(_, p, q) => {
                let case = if uses[p.index()] && uses[q.index()] {
                    Case::Contraction
                } else {
                    Case::Commutative
                };
                let (p, q) = (wrap(self, p), wrap(self, q));
                (self.push(node.with_premises(p, q)), case, Some(quest))
            }
            node => {
                let [p] = node.premises().collect::<Vec<_>>()[..] else {
                    unreachable!("a node that uses the zone has premises")
                };
                let p = wrap(self, p);
                (
                    self.push(node.map_premises(|_| p)),
                    Case::Commutative,
                    Some(quest),
                )
            }
        }
    }

    /// Returns, for every node up to `top`, whether its subproof uses the
    /// unrestricted occurrence `x`: a copy of it that no `?` step of its
    /// formula below discharges, as the checker's least unrestricted zone
    /// holds it.
    fn uses(&self, top: NodeId, x: OccId) -> Vec<bool> {
        let quest = self.forest.parent(x);
        let mut uses = vec![false; top.index() + 1];
        for i in 0..=top.index() {
            uses[i] = match self.nodes[i] {
                Node::Copy(o, p) => o.occ() == x || uses[p.index()],
                Node::Quest(o, p) => Some(o.occ()) != quest && uses[p.index()],
                node => node.premises().any(|p| uses[p.index()]),
            };
        }
        uses
    }

    /// Returns a subproof that is the one at `top` with the occurrence of
    /// `from` in its conclusion renamed to `to`, a literal of the same
    /// atom and sign: the occurrence is followed up to the axiom (or the
    /// affine weakening) that introduces it, or the `⊤` that absorbs it,
    /// and every node on the way is made anew.
    fn rename(&mut self, top: NodeId, from: OccId, to: OccId) -> NodeId {
        let mut seen: HashSet<NodeId> = HashSet::default();
        // The nodes on the way, and for a `⊗`, a Mix or a cut the premise
        // the occurrence comes from: only that one is made anew, even
        // where both premises are one shared subproof.
        let mut on: Vec<(NodeId, Option<Branch>)> = Vec::new();
        let mut stack = vec![top];
        while let Some(id) = stack.pop() {
            if !seen.insert(id) {
                continue;
            }
            let node = self.node(id);
            let mut entered = None;
            match node {
                Node::Ax(..) | Node::One(_) | Node::Top(_) => {}
                Node::Weaken(o, _) if o.occ() == from => {}
                Node::With(_, p, q) => stack.extend([p, q]),
                node => match self.split(node) {
                    Some([first, second]) => {
                        let side = self.holder(from, first, second);
                        entered = Some(side);
                        stack.push(match side {
                            Branch::Left => first.0,
                            Branch::Right => second.0,
                        });
                    }
                    None => stack.extend(node.premises()),
                },
            }
            on.push((id, entered));
        }
        on.sort_unstable_by_key(|&(id, _)| id);
        let mut made: HashMap<NodeId, NodeId> = HashMap::default();
        let to = Member::from(to);
        for (id, entered) in on {
            let old = self.node(id);
            let new = |p: NodeId| made.get(&p).copied().unwrap_or(p);
            let node = match (old, entered) {
                (Node::Ax(x, y), _) if x.occ() == from => Node::Ax(to, y),
                (Node::Ax(x, y), _) if y.occ() == from => Node::Ax(x, to),
                (Node::Weaken(o, p), _) if o.occ() == from => Node::Weaken(to, p),
                (node, Some(side)) => {
                    let [first, second] = self.split(node).expect("a rule of two premises");
                    match side {
                        Branch::Left => node.with_premises(new(first.0), second.0),
                        Branch::Right => node.with_premises(first.0, new(second.0)),
                    }
                }
                (node, None) => node.map_premises(new),
            };
            if node != old {
                let new = self.push(node);
                made.insert(id, new);
            }
        }
        made.get(&top).copied().unwrap_or(top)
    }

    /// The erasure of the premise `erased` of the cut, whose formula there
    /// is `theirs`, by the affine weakening at `weakened` of `ours` over
    /// `kept`: the weakenings of the erased premise's formulas over `kept`.
    /// Where the erased premise absorbs any context through a `⊤` and the
    /// kept one does not, those formulas are the ones each place the proof
    /// reads the cut at gives it, not only those the checker derived.
    fn erase(
        &mut self,
        kept: NodeId,
        weakened: NodeId,
        ours: OccId,
        erased: NodeId,
        theirs: OccId,
    ) -> Made {
        let mut proved = self.record.zone(weakened).clone();
        proved.remove(ours);
        if self.record.absorbs[erased.index()] && !self.record.absorbs[kept.index()] {
            return Made::Erasure { kept, proved };
        }
        let mut context = self.record.zone(erased).clone();
        context.remove(theirs);
        Made::Subproof(self.weaken(kept, &context))
    }

    /// Returns the subproof that weakens every member of `context` over
    /// the one at `kept`.
    fn weaken(&mut self, kept: NodeId, context: &Multiset) -> NodeId {
        context
            .as_slice()
            .iter()
            .fold(kept, |p, &o| self.push(Node::Weaken(o.into(), p)))
    }

    /// Puts `replacement` in the place of the node at `cut` wherever the
    /// proof reads it, making anew every node below it, and returns the
    /// root.
    fn replace(&mut self, cut: NodeId, replacement: NodeId) -> NodeId {
        let root = self.proof.root();
        let mut made: Vec<Option<NodeId>> = vec![None; root.index() + 1];
        made[cut.index()] = Some(replacement);
        for i in cut.index() + 1..=root.index() {
            let node = self.nodes[i];
            if node.premises().any(|p| made[p.index()].is_some()) {
                let node = node.map_premises(|p| made[p.index()].unwrap_or(p));
                made[i] = Some(self.push(node));
            }
        }
        made[root.index()].unwrap_or(root)
    }

    /// Erases a cut that only the place the proof reads it at can say the
    /// context of: every node from the root to the cut is made anew for
    /// each exact conclusion it has on the way there, and each place the
    /// cut is read at gets the weakenings of its own conclusion's formulas
    /// beyond `proved` over `kept`. Returns the root, or the refusal of
    /// more than `most` nodes made, of more than `memory` bytes held by the
    /// conclusions it keeps and the nodes it makes, or of the stop, which
    /// it asks every 4 096 pairs.
    fn specialize(
        &mut self,
        cut: NodeId,
        kept: NodeId,
        proved: &Multiset,
        most: usize,
        memory: Option<u64>,
        stop: &mut dyn FnMut(Progress) -> bool,
    ) -> Result<NodeId, Refusal> {
        /// How many pairs pass between two polls of the stop.
        const POLL: usize = 4096;
        /// A node with an exact conclusion, and the pairs of its premises
        /// that lead to the cut.
        struct Pair {
            /// The node.
            id: NodeId,
            /// Its exact conclusion.
            zone: Multiset,
            /// The pairs of its premises, in the rule's order, where they
            /// lead to the cut.
            premises: [Option<usize>; 2],
        }
        let root = self.proof.root();
        let mut reaches = vec![false; root.index() + 1];
        reaches[cut.index()] = true;
        for i in cut.index() + 1..=root.index() {
            reaches[i] = self.nodes[i].premises().any(|p| reaches[p.index()]);
        }
        let mut pairs = vec![Pair {
            id: root,
            zone: Multiset::of(self.proof.conclusion()),
            premises: [None; 2],
        }];
        let mut index: HashMap<(NodeId, Multiset), usize> = HashMap::default();
        let mut stack = vec![0];
        let refused = || Refusal::Elimination {
            what: crate::limits::Bound::Nodes,
            limit: most as u64,
        };
        // The bytes the pairs hold, each conclusion twice (the pair's and
        // the index's key) at four bytes a member beside a node's sixteen
        // and a table entry, and the nodes made; they saturate.
        let mut held: u64 = 0;
        let mut hold = |members: usize, nodes: usize| -> Result<(), Refusal> {
            held = held
                .saturating_add((members as u64).saturating_mul(8))
                .saturating_add((nodes as u64).saturating_mul(64));
            match memory {
                Some(limit) if held > limit => Err(Refusal::Memory {
                    phase: Phase::Eliminate,
                    limit_bytes: limit,
                    needed_bytes: None,
                }),
                _ => Ok(()),
            }
        };
        let mut popped: usize = 0;
        while let Some(k) = stack.pop() {
            // Each pair is popped once, and there are at most `most` of them.
            popped += 1;
            if popped.is_multiple_of(POLL)
                && stop(Progress::new(Phase::Eliminate, POLL as u64, popped as u64))
            {
                return Err(Refusal::Stopped {
                    phase: Phase::Eliminate,
                });
            }
            let id = pairs[k].id;
            if id == cut {
                continue;
            }
            let node = self.nodes[id.index()];
            let above = self.above(node, &pairs[k].zone);
            for (j, (p, zone)) in node.premises().zip(above).enumerate() {
                if !reaches[p.index()] {
                    continue;
                }
                let at = match index.get(&(p, zone.clone())) {
                    Some(&at) => at,
                    None => {
                        if pairs.len() >= most {
                            return Err(refused());
                        }
                        hold(zone.as_slice().len(), 1)?;
                        pairs.push(Pair {
                            id: p,
                            zone: zone.clone(),
                            premises: [None; 2],
                        });
                        index.insert((p, zone), pairs.len() - 1);
                        stack.push(pairs.len() - 1);
                        pairs.len() - 1
                    }
                };
                pairs[k].premises[j] = Some(at);
            }
        }
        // Each pair makes a node, and each place the cut is read at the
        // weakenings of its context: counted before any is made.
        let weakenings = pairs
            .iter()
            .filter(|pair| pair.id == cut)
            .fold(0usize, |sum, pair| {
                sum.saturating_add(pair.zone.difference(proved).as_slice().len())
            });
        if pairs.len().saturating_add(weakenings) > most {
            return Err(refused());
        }
        hold(0, weakenings)?;
        let mut order: Vec<usize> = (0..pairs.len()).collect();
        order.sort_unstable_by_key(|&k| pairs[k].id);
        let mut made = vec![NodeId::new(0); pairs.len()];
        for k in order {
            let pair = &pairs[k];
            made[k] = if pair.id == cut {
                let context = pair.zone.difference(proved);
                self.weaken(kept, &context)
            } else {
                let node = self.nodes[pair.id.index()];
                let mut j = 0;
                let premises = pair.premises;
                let node = node.map_premises(|p| {
                    let at = premises[j];
                    j += 1;
                    at.map_or(p, |at| made[at])
                });
                self.push(node)
            };
        }
        Ok(made[0])
    }

    /// Returns the exact conclusions of the premises of `node`, in the
    /// rule's order, when its own is `zone`: what the rule takes away
    /// and adds, and for a rule of two premises each premise's derived
    /// zone, the formula the rule takes from it where a `⊤` absorbed it,
    /// and the rest of `zone` given to a premise that absorbs it.
    fn above(&self, node: Node, zone: &Multiset) -> Vec<Multiset> {
        let f = self.forest;
        let change = |removed: Option<OccId>, added: &[OccId]| {
            let mut up = zone.clone();
            if let Some(o) = removed {
                up.remove(o);
            }
            for &o in added {
                up.insert(o);
            }
            up
        };
        if let Some([(p, tp), (q, tq)]) = self.split(node) {
            let removed = match node {
                Node::Tensor(o, ..) => Some(o.occ()),
                _ => None,
            };
            let mut rest = change(removed, &[]);
            let derived = |id: NodeId, taken: Option<OccId>| {
                let mut up = self.record.zone(id).clone();
                if let Some(t) = taken
                    && !up.contains(t)
                {
                    up.insert(t);
                }
                up
            };
            let (mut up_p, mut up_q) = (derived(p, tp), derived(q, tq));
            for (up, taken) in [(&up_p, tp), (&up_q, tq)] {
                let mut own = up.clone();
                if let Some(t) = taken {
                    own.remove(t);
                }
                rest = rest.difference(&own);
            }
            let output = rest.as_slice().iter().any(|&o| self.is_output(o));
            match self.absorber(output, (p, tp), (q, tq)) {
                Branch::Left => up_p = up_p.sum(&rest),
                Branch::Right => up_q = up_q.sum(&rest),
            }
            return vec![up_p, up_q];
        }
        let o = node.principal().map(Member::occ);
        let children: Vec<OccId> = o.map(|o| f.children(o).collect()).unwrap_or_default();
        match node {
            Node::Par(..) => vec![change(o, &children)],
            Node::Bot(..) | Node::Quest(..) | Node::Weaken(..) => vec![change(o, &[])],
            Node::Bang(..) => vec![change(o, &children)],
            Node::Plus(_, side, _) => {
                let child = if side == Branch::Left {
                    children[0]
                } else {
                    children[1]
                };
                vec![change(o, &[child])]
            }
            Node::With(..) => vec![change(o, &children[..1]), change(o, &children[1..])],
            Node::Copy(..) => vec![change(None, &[o.unwrap()])],
            _ => vec![],
        }
    }
}

/// Returns whether the last rule of a premise acts on its occurrence `x`
/// of the cut formula: introduces it, or for `⊤` is it. A copy never
/// does, since it adds to the linear zone rather than introducing what
/// is there.
fn principal(node: Node, x: OccId) -> bool {
    match node {
        Node::Ax(p, q) => p.occ() == x || q.occ() == x,
        Node::Copy(..) | Node::Mix(..) | Node::Cut(..) => false,
        node => node.principal().is_some_and(|o| o.occ() == x),
    }
}

impl Node {
    /// Returns the node of two premises with `first` and `second` for its
    /// premises.
    fn with_premises(self, first: NodeId, second: NodeId) -> Self {
        match self {
            Self::Tensor(o, ..) => Self::Tensor(o, first, second),
            Self::With(o, ..) => Self::With(o, first, second),
            Self::Mix(..) => Self::Mix(first, second),
            Self::Cut(o, ..) => Self::Cut(o, first, second),
            node => unreachable!("{node} has two premises"),
        }
    }
}

#[cfg(all(test, feature = "parse", feature = "interactive"))]
mod tests;
