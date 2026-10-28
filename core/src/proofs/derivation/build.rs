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
use crate::proofs::check::{self, Allowance, Facts, Framed, Ground, Instancing, Observer, State};
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
    let made = Made {
        goal,
        mode,
        reading,
        limits,
        size: &size,
        compact,
    };
    match proof.instances() {
        None => made.unfold(proof, &Ground, stop),
        Some(table) => made.unfold(proof, &Framed::new(proof, table), stop),
    }
}

/// What [`unfold`] has settled before it unfolds the proof.
struct Made<'g> {
    /// The goal.
    goal: &'g [OccId],
    /// The mode.
    mode: Mode,
    /// The reading, for a two-sided derivation.
    reading: Option<&'g Reading<'g>>,
    /// The limits.
    limits: &'g Limits,
    /// The size the derivation was measured at.
    size: &'g Size,
    /// The bound of a compact view, or `None` for every rule shown.
    compact: Option<Budget>,
}

impl Made<'_> {
    /// Unfolds the proof, whose members `inst` knows.
    fn unfold<I: Instancing>(
        self,
        proof: &Proof,
        inst: &I,
        stop: &mut dyn FnMut(Progress) -> bool,
    ) -> Result<Vec<Inference>, Error> {
        let Self {
            goal,
            mode,
            reading,
            limits,
            size,
            compact,
        } = self;
        let mut record = Record::new(proof, inst);
        let allowance = Allowance::new(limits, Phase::View, stop);
        check::examine(proof, goal, mode, reading, allowance, &mut record)?;
        let mut build = Build {
            proof,
            inst,
            record: &record,
            reading,
            // As many as the size says, which fit a `usize` since they fit a
            // `u32` (`Derivation::MOST`), on a 32-bit target too; a compact
            // view has fewer.
            inferences: match compact {
                None => Vec::with_capacity(size.inferences as usize),
                Some(_) => Vec::new(),
            },
            tasks: Vec::new(),
            done: Vec::new(),
            stop,
            compact: compact.map(|budget| Held {
                weights: size::Weights::of(proof),
                bytes: 0,
                budget,
            }),
        };
        build.run(
            proof.root(),
            Multiset::of(goal.iter().copied().map(Member::from)),
        )?;
        Ok(build.inferences)
    }
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
    /// The characters of every member's formula and its separator.
    pub(super) weights: size::Weights,
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
                sum.saturating_add(u64::from(self.weights.get(*o)))
            })
            .saturating_mul(Size::BYTES_PER_CHARACTER)
            .saturating_add(Size::BYTES_PER_INFERENCE)
    }
}

/// What the translation reads off the checker's pass: for every node a
/// flag or two, and the sequent of the nodes where a context is split or
/// padded, which the derivation shows anyway.
pub(super) struct Record<'a, I> {
    /// The forest.
    forest: &'a Forest,
    /// What is known of the proof's members.
    inst: &'a I,
    /// Whether a `⊤` in a node's subproof absorbs any context.
    absorbs: Vec<bool>,
    /// For a `?` step, whether a copy above uses its formula; for a copy,
    /// whether another copy above uses the same occurrence.
    used: Vec<bool>,
    /// The nodes whose sequent is kept: every `⊗`, Mix and cut, and the
    /// premises of `⊗`, Mix, cut and `&`.
    kept: Vec<bool>,
    /// The standard sequents of the kept nodes: `?Θ` and `Γ` as the
    /// checker found them.
    pub(super) standard: HashMap<NodeId, Multiset<Member>>,
    /// For a `⊗`, Mix or cut, the unrestricted members both premises need,
    /// ascending, where there are any.
    pub(super) shared: HashMap<NodeId, Vec<Member>>,
    /// The bytes of the sequents and lists kept so far, at
    /// [`ENTRY`](Self::ENTRY) each and eight for every member; it
    /// saturates.
    held: u64,
}

impl<'a, I: Instancing> Record<'a, I> {
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
    pub(super) fn new(proof: &'a Proof, inst: &'a I) -> Self {
        let len = proof.nodes().len();
        let mut kept = vec![false; len];
        for id in proof.ids() {
            let node = proof.node(id);
            if matches!(node, Node::Tensor(..) | Node::Mix(..) | Node::Cut(..)) {
                kept[id.index()] = true;
            }
            if matches!(
                node,
                Node::Tensor(..) | Node::Mix(..) | Node::Cut(..) | Node::With(..)
            ) {
                for p in node.premises() {
                    kept[p.index()] = true;
                }
            }
        }
        Self {
            forest: proof.forest(),
            inst,
            absorbs: vec![false; len],
            used: vec![false; len],
            kept,
            standard: HashMap::default(),
            shared: HashMap::default(),
            held: 0,
        }
    }
}

impl<I: Instancing> Observer for Record<'_, I> {
    fn bytes(&self) -> u64 {
        // Three flags for each of fewer than 2³² nodes, and what is kept.
        self.held.saturating_add(self.kept.len() as u64 * 3)
    }

    fn derived(&mut self, id: NodeId, state: &State, facts: &Facts<'_>) {
        self.absorbs[id.index()] = state.absorbs();
        self.used[id.index()] = facts.used;
        if self.kept[id.index()] {
            let quests = state
                .unrestricted()
                .map(|a| quest(self.forest, self.inst, a));
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
    Unfold(NodeId, Multiset<Member>),
    /// Unfold the subproof at a node under a conclusion that may hold `?`
    /// formulas the subproof does not use, weakening them above it unless
    /// a `⊤` in it absorbs them.
    Pad(NodeId, Multiset<Member>),
    /// Conclude a sequent by a rule from the subtrees finished last, as
    /// many as the rule has premises.
    Infer {
        /// The conclusion.
        sequent: Multiset<Member>,
        /// The rule.
        rule: Rule,
        /// The formula it introduces.
        principal: Option<Member>,
        /// How many premises it has.
        premises: usize,
        /// How many applications of the rule it stands for.
        times: u32,
        /// For a cut, the member of the cut formula it cuts.
        cut: Option<Member>,
        /// The node whose rule it shows.
        node: NodeId,
    },
    /// Weaken `?` formulas one by one below the subtree finished last.
    Weaken {
        /// The conclusion of the subtree.
        sequent: Multiset<Member>,
        /// The formulas to add, in ascending order.
        unused: Multiset<Member>,
    },
    /// Contract, below the subtree finished last, the `?` formulas that
    /// both premises of the `⊗` or Mix at a node use.
    Contract {
        /// The conclusion of the subtree, which holds each of them twice.
        sequent: Multiset<Member>,
        /// The node.
        id: NodeId,
    },
}

/// The translation in progress.
pub(super) struct Build<'a, I> {
    /// The proof being unfolded.
    proof: &'a Proof,
    /// What is known of the proof's members.
    inst: &'a I,
    /// What the checker derived for the nodes.
    record: &'a Record<'a, I>,
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

impl<'a, I: Instancing> Build<'a, I> {
    /// The forest.
    fn forest(&self) -> &'a Forest {
        self.proof.forest()
    }

    /// The `?` formula an unrestricted-zone member stands for.
    fn quest(&self, a: Member) -> Member {
        quest(self.forest(), self.inst, a)
    }

    /// The occurrence a member stands for.
    fn occ(&self, m: Member) -> OccId {
        self.inst.occ(m)
    }

    /// The standard sequent a subproof derives: `?Θ` and `Γ` as the checker
    /// found them.
    fn standard(&self, id: NodeId) -> Multiset<Member> {
        self.record.standard[&id].clone()
    }

    /// Whether a `⊤` in the subproof absorbs any context.
    fn absorbs(&self, id: NodeId) -> bool {
        self.record.absorbs[id.index()]
    }

    /// The unrestricted occurrences both premises of the `⊗` or Mix at
    /// `id` need, ascending.
    fn shared(&self, id: NodeId) -> &'a [Member] {
        self.record.shared.get(&id).map_or(&[], Vec::as_slice)
    }

    /// Unfolds the subproof at `root`, whose conclusion is `conclusion`,
    /// into the inferences, unless the caller's condition stops it.
    fn run(&mut self, root: NodeId, conclusion: Multiset<Member>) -> Result<(), Error> {
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
                    cut,
                    node,
                } => {
                    let premises = self.done.split_off(self.done.len() - premises);
                    let inference = self.infer(sequent, rule, principal, premises, times);
                    let made = &mut self.inferences[inference.index()];
                    made.cut = cut;
                    // A run's node is its lowest application's, as its
                    // principal formula is.
                    made.node = Some(node);
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
        sequent: Multiset<Member>,
        rule: Rule,
        principal: Option<Member>,
        premises: Vec<InfId>,
        times: u32,
    ) -> InfId {
        let rule = match (self.reading, principal) {
            (Some(reading), Some(o)) => {
                crate::proofs::Named::applied(rule, Some(reading), self.occ(o))
            }
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
                cut: None,
                node: None,
            });
        }
        self.push(Inference {
            sequent: sequent.into_members(),
            rule,
            principal,
            premises,
            times,
            cut: None,
            node: None,
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
    fn leaf(
        &mut self,
        id: NodeId,
        sequent: Multiset<Member>,
        rule: Rule,
        principal: Option<Member>,
    ) {
        let inference = self.infer(sequent, rule, principal, vec![], 1);
        self.inferences[inference.index()].node = Some(id);
        self.done.push(inference);
    }

    /// Adds an inference of `times` applications on `principal` below the
    /// subtree finished last.
    fn below(&mut self, sequent: Multiset<Member>, rule: Rule, principal: Member, times: u32) {
        let premise = self.done.pop().expect("a subtree is finished");
        let inference = self.infer(sequent, rule, Some(principal), vec![premise], times);
        self.done.push(inference);
    }

    /// Plans an inference that concludes `sequent` on `principal` from the
    /// subproof at `p` under the conclusion `up`.
    fn from(
        &mut self,
        id: NodeId,
        sequent: Multiset<Member>,
        rule: Rule,
        principal: Member,
        p: NodeId,
        up: Multiset<Member>,
    ) {
        self.tasks.push(Task::Infer {
            sequent,
            rule,
            principal: Some(principal),
            premises: 1,
            times: 1,
            cut: None,
            node: id,
        });
        self.tasks.push(Task::Unfold(p, up));
    }

    /// Unfolds the subproof at `id`, whose conclusion is `actual`: the
    /// standard sequent it derives plus whatever a `⊤` in it absorbs. A
    /// leaf is concluded at once; any other rule leaves its premises and
    /// then its own inference as steps to take.
    fn unfold(&mut self, id: NodeId, actual: Multiset<Member>) {
        use Node::*;
        let f = self.forest();
        let inst = self.inst;
        let (left, right) = (
            |m: Member| inst.child(m, f.left(inst.occ(m)).unwrap()),
            |m: Member| inst.child(m, f.right(inst.occ(m)).unwrap()),
        );
        // The premise's conclusion when the rule adds or removes formulas.
        let above = |actual: &Multiset<Member>, removed: &[Member], added: &[Member]| {
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
            Ax(..) => self.leaf(id, actual, Rule::Ax, None),
            One(o) => self.leaf(id, actual, Rule::One, Some(o)),
            Top(o) => self.leaf(id, actual, Rule::Top, Some(o)),
            Bot(o, p) => {
                let up = above(&actual, &[o], &[]);
                self.from(id, actual, Rule::Bot, o, p, up);
            }
            Par(o, p) => {
                let up = above(&actual, &[o], &[left(o), right(o)]);
                self.from(id, actual, Rule::Par, o, p, up);
            }
            Plus(o, side, p) => {
                let (chosen, rule) = match side {
                    Branch::Left => (left(o), Rule::PlusLeft),
                    Branch::Right => (right(o), Rule::PlusRight),
                };
                let up = above(&actual, &[o], &[chosen]);
                self.from(id, actual, rule, o, p, up);
            }
            Bang(o, p) => {
                let up = above(&actual, &[o], &[left(o)]);
                self.from(id, actual, Rule::Promotion, o, p, up);
            }
            Weaken(o, p) => {
                let rule = if f.kind(inst.occ(o)) == Kind::Quest {
                    Rule::Weakening
                } else {
                    Rule::AffineWeakening
                };
                let up = above(&actual, &[o], &[]);
                self.from(id, actual, rule, o, p, up);
            }
            Quest(o, p) => {
                if self.record.used[id.index()] {
                    // Used above: `?A` in Γ becomes `?A` in Θ, which the
                    // standard sequent does not distinguish.
                    self.tasks.push(Task::Unfold(p, actual));
                } else {
                    let up = above(&actual, &[o], &[]);
                    self.from(id, actual, Rule::Weakening, o, p, up);
                }
            }
            Copy(a, p) => {
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
                        cut: None,
                        node: id,
                    });
                    self.from(id, derelicted, Rule::Dereliction, q, p, up);
                } else {
                    let up = above(&actual, &[q], &[a]);
                    self.from(id, actual, Rule::Dereliction, q, p, up);
                }
            }
            Tensor(o, l, r) => self.split(id, actual, Join::Tensor(o, left(o), right(o)), l, r),
            Mix(l, r) => self.split(id, actual, Join::Mix, l, r),
            Cut(a, l, r) => {
                // The checker took the cut on an occurrence of a cut
                // formula, which has its dual.
                let dual = inst.dual(f, a).unwrap();
                self.split(id, actual, Join::Cut(a, dual), l, r);
            }
            With(o, l, r) => {
                let up_l = above(&actual, &[o], &[left(o)]);
                let up_r = above(&actual, &[o], &[right(o)]);
                self.tasks.push(Task::Infer {
                    sequent: actual,
                    rule: Rule::With,
                    principal: Some(o),
                    premises: 2,
                    times: 1,
                    cut: None,
                    node: id,
                });
                self.tasks.push(Task::Pad(r, up_r));
                self.tasks.push(Task::Pad(l, up_l));
            }
            Forall(o, _, p) => {
                let up = above(&actual, &[o], &[inst.body(id)]);
                self.from(id, actual, Rule::Forall, o, p, up);
            }
            Exists(o, _, p) => {
                let up = above(&actual, &[o], &[inst.body(id)]);
                self.from(id, actual, Rule::Exists, o, p, up);
            }
        }
    }

    /// Plans a `⊗`, a Mix or a cut at `id` with premises `l` and `r`: the
    /// context is split as the premises derived it, an absorbing premise
    /// taking what the `⊤` absorbs, and the `?` formulas both premises use
    /// are contracted below the rule.
    fn split(&mut self, id: NodeId, actual: Multiset<Member>, join: Join, l: NodeId, r: NodeId) {
        let extra = actual.difference(&self.standard(id));
        let (mut up_l, mut up_r) = (self.standard(l), self.standard(r));
        if let Join::Tensor(_, a, b) | Join::Cut(a, b) = join {
            up_l.ensure(a);
            up_r.ensure(b);
        }
        if let Some(reading) = self.reading
            && self.absorbs(l)
            && self.absorbs(r)
        {
            // Two-sided: the goal among the absorbed formulas goes to the
            // premise that has none, the hypotheses to the left one.
            let outputs = reading.outputs(up_l.as_slice().iter().map(|&m| self.occ(m)));
            let left_has_goal = outputs > 0;
            for &o in extra.as_slice() {
                if reading.position(self.occ(o)) == Side::Output && left_has_goal {
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
        let (rule, principal, sequent) = match join {
            Join::Tensor(o, a, b) => {
                let (mut rest_l, mut rest_r) = (up_l.clone(), up_r.clone());
                rest_l.remove(a);
                rest_r.remove(b);
                let mut sequent = rest_l.sum(&rest_r);
                sequent.insert(o);
                (Rule::Tensor, Some(o), sequent)
            }
            Join::Cut(a, dual) => {
                let (mut rest_l, mut rest_r) = (up_l.clone(), up_r.clone());
                rest_l.remove(a);
                rest_r.remove(dual);
                (Rule::Cut, None, rest_l.sum(&rest_r))
            }
            Join::Mix => (Rule::Mix, None, up_l.sum(&up_r)),
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
            cut: match join {
                Join::Cut(a, _) => Some(a),
                Join::Tensor(..) | Join::Mix => None,
            },
            node: id,
        });
        self.tasks.push(Task::Unfold(r, up_r));
        self.tasks.push(Task::Unfold(l, up_l));
    }

    /// Returns the formula the node at `id` weakens and the rule, if it is
    /// a weakening: a `?` step whose formula is not used above or a
    /// weakening of a `?` formula (`?w`), or an affine weakening (`wk`).
    fn weakened(&self, id: NodeId) -> Option<(Member, Rule)> {
        match self.proof.node(id) {
            Node::Quest(o, _) if !self.record.used[id.index()] => Some((o, Rule::Weakening)),
            Node::Weaken(o, _) if self.forest().kind(self.occ(o)) == Kind::Quest => {
                Some((o, Rule::Weakening))
            }
            Node::Weaken(o, _) => Some((o, Rule::AffineWeakening)),
            _ => None,
        }
    }

    /// Plans, in a compact view, the run of weakenings that starts at the
    /// node at `id`, which weakens `o` by `rule`, under the conclusion
    /// `actual`: one inference for every weakening of the same named rule
    /// down the chain of nodes, through the `?` steps whose formula is
    /// used above, which are no inference, without a sequent per step (a
    /// chain of `wk` took a sequent per node, quadratic in the chain).
    fn weaken_run(&mut self, id: NodeId, o: Member, rule: Rule, actual: Multiset<Member>) {
        let named = |o: Member, rule: Rule| match (rule, self.reading) {
            (Rule::Weakening, Some(reading)) => Rule::Weakening.on(reading.position(self.occ(o))),
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
            cut: None,
            node: id,
        });
        self.tasks.push(Task::Unfold(next, up));
    }

    /// Plans the subproof at `id` under a conclusion that may hold `?`
    /// formulas the subproof does not use, weakening them above it unless
    /// a `⊤` in it absorbs them.
    fn pad(&mut self, id: NodeId, actual: Multiset<Member>) {
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

/// What a rule of two premises that [`Build::split`] plans joins them by.
#[derive(Clone, Copy, Debug)]
enum Join {
    /// A `⊗` on its member, with its left and right subformula.
    Tensor(Member, Member, Member),
    /// Mix.
    Mix,
    /// A cut on a member of a cut formula, with its dual.
    Cut(Member, Member),
}

/// Returns the `?` formula the unrestricted-zone member `a` stands for.
fn quest(forest: &Forest, inst: &impl Instancing, a: Member) -> Member {
    let q = forest.parent(inst.occ(a)).unwrap();
    inst.parent(a, q)
}
