// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The translation of a checked term into the inferences of its
//! derivation, and what it holds while it runs.

use super::view::{Compact, Derivation, InfId, Inference, ViewOptions};
use crate::Error;
use crate::fragment::Mode;
use crate::hash::HashMap;
use crate::limits::{Limits, Phase, Progress, Refusal, Space};
use crate::occurrences::{Forest, Member, OccId, Reading, Side};
use crate::proofs::check::{self, Allowance, Facts, Observer, State};
use crate::proofs::multiset::Multiset;
use crate::proofs::size::{self, Size};
use crate::proofs::{Branch, Node, NodeId, Proof, Rule};
use crate::sequents::Kind;

/// Unfolds a proof that concludes `goal` into the inferences of its
/// derivation, premises before conclusions and the root last: one pass of
/// the checker for the size, which must be within the bounds, a second for
/// what the translation reads, then the translation. Every pass holds no
/// more than the memory bound, and so does the derivation by its estimate.
/// Where the options ask for the compact view, or allow it and the whole
/// derivation is over a bound, the translation draws runs of a structural
/// rule as one inference and counts what it holds as it goes, giving up
/// with the whole derivation's error once that passes the bound.
pub(super) fn unfold(
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
pub(super) fn count(len: usize) -> u32 {
    u32::try_from(len).unwrap_or(u32::MAX)
}

/// The refusal of a derivation of more inferences than one holds.
pub(super) fn too_many(size: &Size) -> Refusal {
    Refusal::Index {
        what: Space::Inference,
        count: size.inferences,
        most: Derivation::MOST,
    }
}

/// The bound of a compact view, which is not measured before it is built.
pub(super) struct Budget {
    /// The most bytes the derivation may take by [`Size::bytes`]'s
    /// estimate, or `None` for no bound but [`Derivation::MOST`].
    pub(super) most: Option<u64>,
    /// The error to give up with: the whole derivation's.
    pub(super) error: Error,
}

/// What a compact view holds so far, by the estimate of [`Size::bytes`].
pub(super) struct Held {
    /// The characters of every occurrence's formula and its separator.
    pub(super) weights: Vec<u32>,
    /// The bytes of the inferences made so far; it saturates.
    pub(super) bytes: u64,
    /// The bound.
    pub(super) budget: Budget,
}

impl Held {
    /// Returns the bytes an inference with this sequent takes.
    pub(super) fn cost(&self, sequent: &[Member]) -> u64 {
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
pub(super) struct Record<'a> {
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
    pub(super) standard: HashMap<NodeId, Multiset>,
    /// For a `⊗` or Mix, the unrestricted occurrences both premises need,
    /// ascending, where there are any.
    pub(super) shared: HashMap<NodeId, Vec<OccId>>,
    /// The bytes of the sequents and lists kept so far, at
    /// [`ENTRY`](Self::ENTRY) each and eight for every member; it
    /// saturates.
    held: u64,
}

impl<'a> Record<'a> {
    /// What a kept sequent or list takes besides its members: its place in
    /// the table and its allocation.
    pub(super) const ENTRY: u64 = 96;

    /// Counts a list of `members` occurrences as kept: four bytes each,
    /// in a vector that may have room for as many again.
    fn keep(&mut self, members: usize) {
        let bytes = (members as u64).saturating_mul(8);
        self.held = self.held.saturating_add(Self::ENTRY).saturating_add(bytes);
    }

    /// The record of a proof the checker has yet to pass over.
    pub(super) fn new(proof: &'a Proof) -> Self {
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
pub(super) enum Task {
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
pub(super) struct Build<'a> {
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
            && let Some((o, rule)) = self.weakened(id)
        {
            return self.weaken_run(id, o, rule, actual);
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

    /// Returns the formula the node at `id` weakens and the rule, if it is
    /// a weakening: a `?` step whose formula is not used above or a
    /// weakening of a `?` formula (`?w`), or an affine weakening (`wk`).
    fn weakened(&self, id: NodeId) -> Option<(OccId, Rule)> {
        match self.proof.node(id) {
            Node::Quest(o, _) if !self.record.used[id.index()] => Some((o.occ(), Rule::Weakening)),
            Node::Weaken(o, _) if self.forest().kind(o.occ()) == Kind::Quest => {
                Some((o.occ(), Rule::Weakening))
            }
            Node::Weaken(o, _) => Some((o.occ(), Rule::AffineWeakening)),
            _ => None,
        }
    }

    /// Plans, in a compact view, the run of weakenings that starts at the
    /// node at `id`, which weakens `o` by `rule`, under the conclusion
    /// `actual`: one inference for every weakening of the same named rule
    /// down the chain of nodes, through the `?` steps whose formula is
    /// used above, which are no inference, without a sequent per step (a
    /// chain of `wk` took a sequent per node, quadratic in the chain).
    fn weaken_run(&mut self, id: NodeId, o: OccId, rule: Rule, actual: Multiset) {
        let named = |o: OccId, rule: Rule| match (rule, self.reading) {
            (Rule::Weakening, Some(reading)) => Rule::Weakening.on(reading.position(o)),
            (rule, _) => rule.into(),
        };
        let first = named(o, rule);
        let mut removed = vec![o];
        let mut next = self.proof.node(id).premises().next().unwrap();
        loop {
            match self.proof.node(next) {
                Node::Quest(_, p) if self.record.used[next.index()] => next = p,
                _ => match self.weakened(next) {
                    Some((o, rule)) if named(o, rule) == first => {
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
            rule,
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
