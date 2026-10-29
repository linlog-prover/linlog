// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The checker's one pass, its bounds and its observers.

use super::error::{CheckError, Fault, Refused};
use super::frames::{Framed, Ground, Instancing};
use super::zones::{Bag, Eigens, State, Zone};
use crate::fragment::Mode;
use crate::limits::{Limits, Phase, Progress, Refusal};
use crate::occurrences::{Forest, Member, OccId, Reading, Side};
use crate::proofs::{Branch, Node, NodeId, Proof};
use crate::sequents::Kind;

/// Why the pass stopped at a node: a fault of the proof, or its bound.
#[derive(Debug)]
pub(super) enum Halt {
    /// The rule required what it did not get.
    Fault(Fault),
    /// The sequents the pass must hold at once take more than the bound.
    Memory {
        /// The bound, in bytes.
        limit: u64,
    },
    /// The pass has done as much work as its bound allows.
    Work {
        /// The bound, in units of work ([`Allowance`]).
        limit: u64,
    },
    /// The caller's stop ended the pass.
    Stopped,
}

/// How many nodes a pass visits between two polls of its stop, at most.
pub(super) const POLL: usize = 4096;

/// How many units of work a pass does between two polls of its stop, at
/// most: a node with large shared premises does many.
pub(super) const POLL_WORK: u64 = 1 << 16;

/// What a pass of the checker may spend, and whom it asks whether to go
/// on: the memory bound, the most work it may do, the phase a refusal
/// names, and the caller's stop, asked every [`POLL`] nodes or
/// [`POLL_WORK`] units of work, whichever comes first, with the work
/// done. A unit of work is a node, or an entry of a premise's sequent
/// that the node's rule is handed (and copies, pours or compares): a
/// node read by many others with a large sequent costs each of them its
/// size, so a file of a few megabytes can ask for 10¹⁰ units.
pub(crate) struct Allowance<'s> {
    /// The most bytes the pass may hold, or `None` for any number.
    pub(crate) memory: Option<u64>,
    /// The most units of work the pass may do, or `None` for any number.
    pub(crate) work: Option<u64>,
    /// What the pass is for.
    pub(crate) phase: Phase,
    /// The caller's stop.
    pub(crate) stop: &'s mut dyn FnMut(Progress) -> bool,
}

impl<'s> Allowance<'s> {
    /// Returns what `limits` allow a pass for `phase` that asks `stop`.
    pub(crate) fn new(
        limits: &Limits,
        phase: Phase,
        stop: &'s mut dyn FnMut(Progress) -> bool,
    ) -> Self {
        Self {
            memory: limits.memory_bytes,
            work: limits.work,
            phase,
            stop,
        }
    }
}

impl From<Fault> for Halt {
    /// The fault, as what stopped the pass.
    fn from(fault: Fault) -> Self {
        Self::Fault(fault)
    }
}

/// Checks that a proof proves its sequent: every node applies its rule to
/// what its premises derive, the mode allows the rule, and the root derives
/// exactly the sequent's formulas with nothing left in the unrestricted
/// zone; in intuitionistic mode also that the sequent has an intuitionistic
/// reading and every sequent of the proof one formula on the right of `⊢`.
/// Returns the first node that fails, in arena order, with what it needed.
///
/// The memory taken is that of the sequents some later node still reads,
/// which for a proof without shared subproofs is proportional to the
/// proof, and within [`Limits::DEFAULT_MEMORY_BYTES`]: see [`check_within`].
pub(crate) fn check(proof: &Proof, mode: Mode) -> Result<(), CheckError> {
    check_within(proof, mode, &Limits::default(), &mut |_| false)
}

/// Checks that a proof proves its sequent as [`check`] does, holding
/// `memory` bytes at most, or any number with `None`. A node that several
/// others read has its sequent copied for each, so a proof with shared
/// subproofs can take its nodes times its sequents; a check that would
/// pass the bound ends with an error that
/// [`Refused`](CheckError::Refused), which says nothing about the
/// proof.
///
/// What is counted is what the pass allocates beyond the proof and the
/// forest it is given: a count and a pointer for every node (twelve
/// bytes on a 64-bit target, eight on a 32-bit one), and every sequent it
/// holds at the size of its two tables of members, which is the least
/// they take and at least a quarter of what they do (a table that
/// members came and went from can have up to four times the slots of one
/// that only grew). A copy is counted before it is made. Not counted:
/// what one rule needs while it joins two sequents and the sequents of
/// an error's report, each within a small multiple of the largest
/// sequent counted, and the intuitionistic reading, which is a few bytes
/// for every occurrence of the forest.
pub(crate) fn check_within(
    proof: &Proof,
    mode: Mode,
    limits: &Limits,
    stop: &mut dyn FnMut(Progress) -> bool,
) -> Result<(), CheckError> {
    let reading = reading(proof, mode)?;
    examine(
        proof,
        &proof.conclusion(),
        mode,
        reading.as_ref(),
        Allowance::new(limits, Phase::Check, stop),
        &mut (),
    )
}

/// Returns the intuitionistic reading of the proof's sequent in
/// intuitionistic mode and none otherwise, or the error of a sequent that
/// has no reading.
pub(crate) fn reading(proof: &Proof, mode: Mode) -> Result<Option<Reading<'_>>, CheckError> {
    if !mode.intuitionistic {
        return Ok(None);
    }
    match Reading::of_mode(proof.forest(), mode) {
        Ok(reading) => Ok(Some(reading)),
        Err(e) => Err(CheckError::invalid(
            proof.root(),
            proof.node(proof.root()),
            vec![],
            Fault::Shape(e),
        )),
    }
}

/// Checks that a proof concludes `goal`, a multiset of occurrences in any
/// order (the roots, for a proof of the sequent), as [`check_within`]
/// does, with the one-succedent condition when a reading is given, and
/// shows every node's sequent to `observer` on the way.
pub(crate) fn examine<O: Observer>(
    proof: &Proof,
    goal: &[OccId],
    mode: Mode,
    reading: Option<&Reading>,
    allowance: Allowance<'_>,
    observer: &mut O,
) -> Result<(), CheckError> {
    match proof.instances() {
        None => examine_with(proof, goal, mode, reading, allowance, observer, || Ground),
        Some(table) => examine_with(proof, goal, mode, reading, allowance, observer, || {
            Framed::new(proof, table)
        }),
    }
}

/// Checks a proof as [`examine`] does, whose members `instancing` (made
/// anew for a second pass) knows.
fn examine_with<O: Observer, I: Instancing>(
    proof: &Proof,
    goal: &[OccId],
    mode: Mode,
    reading: Option<&Reading>,
    allowance: Allowance<'_>,
    observer: &mut O,
    instancing: impl Fn() -> I,
) -> Result<(), CheckError> {
    let (memory, phase) = (allowance.memory, allowance.phase);
    afford(proof, observer.bytes(), memory, phase)?;
    let mut pass = Pass::new(
        proof,
        goal.len(),
        mode,
        reading,
        allowance,
        observer,
        instancing(),
    );
    let end = proof.nodes().len();
    let (node, halt) = match pass.run(end) {
        Err(failure) => failure,
        Ok(()) => match pass.conclude(goal) {
            Ok(()) => return Ok(()),
            Err(halt) => (proof.root(), halt),
        },
    };
    drop(pass);
    let rule = proof.node(node);
    let fault = match halt {
        Halt::Fault(fault) => fault,
        // A refusal shows no premises: deriving them again would take the
        // memory that was refused.
        Halt::Memory { limit } => return Err(memory_refused(node, limit, phase)),
        Halt::Work { limit } => return Err(refused(node, Refusal::Work { limit })),
        Halt::Stopped => return Err(refused(node, Refusal::Stopped { phase })),
    };
    // The premises' sequents are gone, moved into the node that failed, so
    // the pass runs once more up to it. They are kept then, since the node
    // itself has yet to read them, and the pass does what the first did
    // up to there, with nothing of an observer's to count: it holds no
    // more, and cannot fail.
    let mut nobody = ();
    let mut never = |_| false;
    let again = Allowance {
        memory,
        work: None,
        phase,
        stop: &mut never,
    };
    let mut pass = Pass::new(
        proof,
        goal.len(),
        mode,
        reading,
        again,
        &mut nobody,
        instancing(),
    );
    let again = pass.run(node.index());
    debug_assert!(again.is_ok());
    let premises = rule
        .premises()
        .filter_map(|p| pass.live[p.index()].as_deref().map(State::to_dyadic))
        .collect();
    Err(CheckError::invalid(node, rule, premises, fault))
}

/// Returns whether the contexts `left` and `right` that a `⊗` at `o`
/// joins are the two arcs of the cyclic order of its conclusion on
/// either side of it: going round from `o` in occurrence order, every
/// member of `right` before every member of `left`.
///
/// A cyclic sequent's formulas stand in ascending occurrence order up to
/// rotation, and so do every subproof's: the conclusion `Γ, A ⊗ B, Δ`
/// read from the `⊗` is `Δ` then `Γ`, and `A` and `B` take the `⊗`'s
/// place in that order, since its subtree holds no other member. So the
/// premises `⊢ Γ, A` and `⊢ B, Δ` are each in the order the forest gives
/// them, and checking this at every `⊗` of a proof the commutative rules
/// accept makes it a proof of cyclic MLL; the axiom and `⅋` need nothing,
/// a link's two literals being neighbours in any order and the two
/// operands of a `⅋` neighbours in the forest's.
fn arcs(forest: &Forest, o: OccId, left: &Bag, right: &Bag, occ: impl Fn(Member) -> OccId) -> bool {
    // An occurrence's distance from `o` going round: below the forest's
    // length, which fits a `u32`, so no sum wraps in a `u64`.
    let n = forest.len() as u64;
    let from = |x: OccId| (u64::from(x.get()) + n - u64::from(o.get())) % n;
    let last_right = right.counts().map(|(x, _)| from(occ(x))).max();
    let first_left = left.counts().map(|(x, _)| from(occ(x))).min();
    // A member in both contexts belongs to no proof the commutative rules
    // accept, which concludes every occurrence once; it passes here as in
    // the oracle, which lists the right context first.
    match (last_right, first_left) {
        (Some(r), Some(l)) => r <= l,
        _ => true,
    }
}

/// The refusal of a pass at `node` that would hold more than `limit`
/// bytes.
pub(super) fn memory_refused(node: NodeId, limit: u64, phase: Phase) -> CheckError {
    refused(
        node,
        Refusal::Memory {
            phase,
            limit_bytes: limit,
            needed_bytes: None,
        },
    )
}

/// The refusal of a pass at `node`.
pub(super) fn refused(node: NodeId, refusal: Refusal) -> CheckError {
    CheckError::Refused(Refused { node, refusal })
}

/// Returns the bytes of the two tables a pass over `proof` keeps: a count
/// of readers and a place for a sequent, a `u32` and a pointer, for each
/// of fewer than 2³² nodes: below 2³⁶ on any target.
pub(super) fn tables(proof: &Proof) -> u64 {
    proof.nodes().len() as u64 * (size_of::<u32>() + size_of::<Option<Box<State>>>()) as u64
}

/// Refuses a pass over `proof` that may hold `memory` bytes when its own
/// tables and the `observer` bytes of its observer's are more than that
/// already. An observer asks before it allocates its own.
pub(crate) fn afford(
    proof: &Proof,
    observer: u64,
    memory: Option<u64>,
    phase: Phase,
) -> Result<(), CheckError> {
    match memory {
        Some(limit) if tables(proof).saturating_add(observer) > limit => {
            Err(memory_refused(NodeId::new(0), limit, phase))
        }
        _ => Ok(()),
    }
}

/// What a caller of [`examine`] learns about every node of a correct
/// proof, in arena order.
pub(crate) trait Observer {
    /// Returns the weight of a member `m`, which stands for the occurrence
    /// `o`, that a [`State`] adds up over its sequent. A `u32`, so that the
    /// sum over a zone fits a `u64`.
    fn weight(&self, m: Member, o: OccId) -> u32 {
        let _ = (m, o);
        0
    }

    /// Returns the bytes the observer holds for the pass by now, which
    /// count against the pass's bound.
    fn bytes(&self) -> u64 {
        0
    }

    /// Takes what node `id` derived and how its rule applied.
    fn derived(&mut self, id: NodeId, state: &State, facts: &Facts<'_>);
}

impl Observer for () {
    fn derived(&mut self, _: NodeId, _: &State, _: &Facts<'_>) {}
}

/// How a node's rule applied to its premises, beyond the sequent derived.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Facts<'a> {
    /// For a `?` step, whether a copy above uses its formula; for a copy,
    /// whether another copy above uses the same occurrence.
    pub(crate) used: bool,
    /// For `⊗` and Mix, the unrestricted members both premises need, in
    /// no particular order.
    pub(crate) shared: &'a [Member],
    /// Whether a subformula the rule consumes was missing from its premise
    /// and absorbed by a `⊤` above: the left and the right one of `⅋`, `⊗`
    /// and `&`, and the first for a rule that consumes one.
    pub(crate) absent: [bool; 2],
    /// For `⊗`, `&` and Mix, how many unrestricted occurrences each
    /// premise needs: the sizes of two sets of occurrences.
    pub(crate) needs: [usize; 2],
}

/// The pass over a proof's nodes.
pub(super) struct Pass<'a, 's, O, I> {
    /// The proof.
    proof: &'a Proof,
    /// Its forest.
    forest: &'a Forest,
    /// How many formulas the proof is to conclude: the length of a slice,
    /// only ever added to with saturation, and then taken to
    /// [`Bag::MOST`], below `usize::MAX` on a 32-bit target too.
    goal: usize,
    /// The rules in force.
    mode: Mode,
    /// The intuitionistic reading, in intuitionistic mode.
    reading: Option<&'a Reading<'a>>,
    /// How many later nodes still read each node's sequent. A proof of `n`
    /// nodes has `n` references to one node at most: every node after it
    /// has two premises at most, and every one of them but the root is
    /// itself the premise of a later node, or the proof would not hold
    /// it. And `n` fits a `u32`, as [`Proof::new`] sees to, whatever the
    /// width of a `usize`.
    readers: Vec<u32>,
    /// The sequents of the nodes that a later node still reads.
    live: Vec<Option<Box<State>>>,
    /// Who is shown every node.
    observer: &'a mut O,
    /// The unrestricted members both premises of the current node need.
    shared: Vec<Member>,
    /// What the pass knows of the proof's members.
    inst: I,
    /// Per node a later node still reads, the eigenvariables its sequent
    /// brings to the root ([`Eigens`]); empty for a proof without binders.
    eigens: Vec<Option<Box<Eigens>>>,
    /// The most bytes the pass may hold, or none for any number.
    memory: Option<u64>,
    /// The most units of work the pass may do, or none for any number.
    work: Option<u64>,
    /// The units of work done: nodes visited and the entries of the
    /// premises handed to their rules, each premise's fewer than 2³³ and
    /// at most two premises a node, of fewer than 2³² nodes: below 2⁶⁶,
    /// so the count saturates. It is a `u64` on every target, and each
    /// premise's entries are summed in `u64` ([`State::entries`]).
    done: u64,
    /// The work done when the stop was last asked.
    polled: u64,
    /// What the pass is for, which its progress names.
    phase: Phase,
    /// The caller's stop, asked every [`POLL`] nodes.
    stop: &'s mut dyn FnMut(Progress) -> bool,
    /// The bytes the pass holds itself: its two tables, and every sequent
    /// in `live` or in the hands of the current rule, each as
    /// [`State::bytes`] counts it. Every part is allocated, and counted
    /// at no more than it takes, so the sum is below what the process
    /// holds and far from 2⁶⁴; it saturates all the same, so that no
    /// estimate, however wrong, could wrap it. What the observer holds
    /// is added whenever the sum is compared with the bound.
    held: u64,
    /// The bytes among them of the sequents the current rule was handed.
    passed: u64,
}

impl<'a, 's, O: Observer, I: Instancing> Pass<'a, 's, O, I> {
    /// The pass before its first node, which [`afford`] has allowed its
    /// tables.
    fn new(
        proof: &'a Proof,
        goal: usize,
        mode: Mode,
        reading: Option<&'a Reading<'a>>,
        allowance: Allowance<'s>,
        observer: &'a mut O,
        inst: I,
    ) -> Self {
        let mut readers = vec![0; proof.nodes().len()];
        for node in proof.nodes() {
            for p in node.premises() {
                readers[p.index()] += 1;
            }
        }
        let mut live = Vec::new();
        live.resize_with(readers.len(), || None);
        let held = tables(proof);
        Self {
            proof,
            forest: proof.forest(),
            goal,
            mode,
            reading,
            readers,
            live,
            observer,
            shared: Vec::new(),
            eigens: if I::FRAMED {
                let mut eigens = Vec::new();
                eigens.resize_with(proof.nodes().len(), || None);
                eigens
            } else {
                Vec::new()
            },
            inst,
            memory: allowance.memory,
            work: allowance.work,
            phase: allowance.phase,
            stop: allowance.stop,
            held,
            passed: 0,
            done: 0,
            polled: 0,
        }
    }

    /// Counts `bytes` more as held, or refuses them when the pass and its
    /// observer would hold more than the bound.
    fn charge(&mut self, bytes: u64) -> Result<(), Halt> {
        self.held = self.held.saturating_add(bytes);
        match self.memory {
            Some(limit) if self.held.saturating_add(self.observer.bytes()) > limit => {
                Err(Halt::Memory { limit })
            }
            _ => Ok(()),
        }
    }

    /// Derives the nodes before `end` in arena order, or returns the first
    /// that misapplies its rule, uses one the mode forbids, derives more
    /// than the proof can conclude or breaks the one-succedent condition,
    /// with the problem; or the node at which the pass would hold more
    /// than its bound.
    fn run(&mut self, end: usize) -> Result<(), (NodeId, Halt)> {
        for id in self.proof.ids().take(end) {
            if let Some(limit) = self.work
                && self.done >= limit
            {
                return Err((id, Halt::Work { limit }));
            }
            self.done = self.done.saturating_add(1);
            if (id.index() + 1).is_multiple_of(POLL) || self.done - self.polled >= POLL_WORK {
                let since = self.done - self.polled;
                self.polled = self.done;
                let progress = Progress {
                    held_bytes: self.held,
                    ..Progress::new(self.phase, since, self.done)
                };
                if (self.stop)(progress) {
                    return Err((id, Halt::Stopped));
                }
            }
            let mut facts = Facts::default();
            self.passed = 0;
            let state = self
                .rule(id, self.proof.node(id), &mut facts)
                .and_then(|state| self.within(id, state))
                .and_then(|state| self.one_succedent(state))
                .map_err(|problem| (id, problem))?;
            if I::FRAMED {
                self.flow(id).map_err(|problem| (id, problem))?;
            }
            facts.shared = &self.shared;
            self.observer.derived(id, &state, &facts);
            // The premises' sequents are gone into the node's own. Each was
            // counted when it was kept or copied, so the difference is exact
            // as long as the count never saturated.
            self.held = self.held.saturating_sub(self.passed);
            // The root has no reader and is read at the end.
            if self.readers[id.index()] > 0 || id == self.proof.root() {
                self.charge(state.bytes())
                    .map_err(|problem| (id, problem))?;
                self.live[id.index()] = Some(state);
            }
        }
        Ok(())
    }

    /// Checks that the root derived `goal` and needs no unrestricted
    /// occurrence, and under a reading that the goal is an intuitionistic
    /// sequent: one output, as every sequent of the derivation has.
    fn conclude(&self, goal: &[OccId]) -> Result<(), Halt> {
        if let Some(reading) = self.reading {
            let count = reading.outputs(goal.iter().copied());
            if count != 1 {
                return Err(Halt::Fault(Fault::Succedents { count }));
            }
        }
        let root = self.live[self.proof.root().index()]
            .as_deref()
            .expect("the root's sequent is kept");
        // A root's eigenvariable remained: a witness or a cut's frame
        // names it outside the `∀` step that introduces it. One that no
        // step introduces is a constant outside the signature.
        let remained = self
            .eigens
            .get(self.proof.root().index())
            .and_then(Option::as_deref);
        let escaped = remained.and_then(|e| {
            e.set
                .iter()
                .copied()
                .filter(|&a| self.inst.introduces(a))
                .min()
        });
        if let Some(eigen) = escaped {
            return Err(Halt::Fault(Fault::Eigenvariable { eigen }));
        }
        let goal = Bag::of(goal.iter().copied().map(Member::from));
        let concludes = if root.any {
            root.gamma.is_subset(&goal)
        } else {
            root.gamma == goal
        };
        if root.theta.is_empty() && concludes {
            Ok(())
        } else {
            Err(Halt::Fault(Fault::Conclusion {
                derived: root.to_dyadic(),
            }))
        }
    }

    /// What premise `p` derived: the sequent itself for its last reader, a
    /// copy for the others, unless the pass would hold more than its bound
    /// with the copy.
    fn premise(&mut self, p: NodeId) -> Result<Box<State>, Halt> {
        // The count includes this reader, so it is one at least.
        self.readers[p.index()] -= 1;
        let last = self.readers[p.index()] == 0;
        let kept = "a premise's sequent is kept until its last reader";
        let state = self.live[p.index()].as_deref().expect(kept);
        let bytes = state.bytes();
        self.done = self.done.saturating_add(state.entries());
        // A copy takes what its original does, and is counted before it is
        // made; the original is counted already.
        if !last {
            self.charge(bytes)?;
        }
        self.passed = self.passed.saturating_add(bytes);
        let slot = &mut self.live[p.index()];
        Ok(if last { slot.take() } else { slot.clone() }.expect(kept))
    }

    /// Whether `o` is in output position, under a reading.
    fn is_output(&self, o: OccId) -> bool {
        self.reading.is_some_and(|r| r.position(o) == Side::Output)
    }

    /// Fails unless the member `m`, of the occurrence `o`, has the kind
    /// the rule acts on.
    fn expect(&self, m: Member, o: OccId, kind: Kind) -> Result<(), Halt> {
        if self.forest.kind(o) == kind {
            Ok(())
        } else {
            Err(Halt::Fault(Fault::Kind { member: m }))
        }
    }

    /// Adds one copy of `m` to the linear zone of `d`.
    fn put(&self, d: &mut State, m: Member) {
        let o = self.inst.occ(m);
        d.gamma.insert(m);
        let weight = u64::from(self.observer.weight(m, o));
        d.linear = d.linear.saturating_add(weight);
        if self.is_output(o) {
            d.outputs = d.outputs.saturating_add(1);
            d.goal = d.goal.saturating_add(weight);
        }
    }

    /// Consumes one copy of `m` from the linear zone of `d`, the derived
    /// sequent of premise `premise`, and returns whether there was none: a
    /// `⊤` above stands in for a missing one, but never for a second goal.
    fn take(&self, d: &mut State, m: Member, premise: usize) -> Result<bool, Halt> {
        let o = self.inst.occ(m);
        if d.gamma.remove(m) {
            // The premise's sums are exact and count `m`, which was a
            // member: nothing goes below zero.
            let weight = u64::from(self.observer.weight(m, o));
            d.linear -= weight;
            if self.is_output(o) {
                d.outputs -= 1;
                d.goal -= weight;
            }
            return Ok(false);
        }
        if !d.any {
            return Err(Halt::Fault(Fault::Missing { premise, member: m }));
        }
        // The premise's sequent holds `m` besides its zone: one goal at
        // most.
        if self.is_output(o) && d.outputs > 0 {
            return Err(Halt::Fault(Fault::Succedents { count: 2 }));
        }
        Ok(true)
    }

    /// The weight of the `?` formula an unrestricted member stands for.
    fn quest_weight(&self, a: Member) -> u64 {
        let q = self
            .forest
            .parent(self.inst.occ(a))
            .expect("an unrestricted occurrence is under a ?");
        u64::from(self.observer.weight(self.inst.parent(a, q), q))
    }

    /// Checks that the linear zone node `id` derived is one the proof can
    /// still conclude its goal from. A rule consumes two members of a
    /// premise's zone at most and passes the others on, and the root's zone
    /// lies within the goal, so a zone with more members than the goal has
    /// formulas plus two for every later node belongs to no proof. This is
    /// what keeps a zone's counters exact: a term may double a zone at
    /// every node (a Mix of a subproof with itself), which no counter of a
    /// fixed width follows for long.
    fn within(&self, id: NodeId, d: Box<State>) -> Result<Box<State>, Halt> {
        // A node's index is below the number of nodes.
        let later = self.proof.nodes().len() - 1 - id.index();
        let room = self
            .goal
            .saturating_add(later.saturating_mul(2))
            .min(Bag::MOST);
        if d.gamma.len > room {
            return Err(Halt::Fault(Fault::Surplus));
        }
        Ok(d)
    }

    /// Checks the one-succedent condition on what a node derived: one
    /// output at most, and exactly one unless a `⊤` above supplies it;
    /// in the Lambek calculus without empty antecedents, an input besides.
    fn one_succedent(&self, d: Box<State>) -> Result<Box<State>, Halt> {
        if self.reading.is_some() && (d.outputs > 1 || (d.outputs == 0 && !d.any)) {
            return Err(Halt::Fault(Fault::Succedents { count: d.outputs }));
        }
        // The zone's count is exact: `within` held it to the bound.
        if !self.mode.allows_empty_antecedents() && d.gamma.len <= d.outputs {
            return Err(Halt::Fault(Fault::EmptyAntecedent));
        }
        Ok(d)
    }

    /// The member of the left subformula of `m`, of the occurrence `o`,
    /// which must have one.
    fn left(&self, m: Member, o: OccId) -> Member {
        self.inst.child(m, self.forest.left(o).unwrap())
    }

    /// The member of the right subformula of `m`, of the occurrence `o`,
    /// which must have one.
    fn right(&self, m: Member, o: OccId) -> Member {
        self.inst.child(m, self.forest.right(o).unwrap())
    }

    /// The sequent with only the given members in its linear zone.
    fn just(&self, ids: impl IntoIterator<Item = Member>, any: bool) -> Box<State> {
        let mut d = Box::new(State {
            theta: Zone::default(),
            gamma: Bag::default(),
            outputs: 0,
            linear: 0,
            unrestricted: 0,
            goal: 0,
            any,
        });
        for m in ids {
            self.put(&mut d, m);
        }
        d
    }

    /// Unites the unrestricted zone of `r` into that of `l`, the smaller
    /// set into the larger, and notes the members both hold.
    fn unite(&mut self, l: &mut State, r: &mut State) {
        if r.theta.len() > l.theta.len() {
            std::mem::swap(&mut l.theta, &mut r.theta);
            std::mem::swap(&mut l.unrestricted, &mut r.unrestricted);
        }
        for a in r.theta.members.drain() {
            if l.theta.insert(a) {
                l.unrestricted += self.quest_weight(a);
            } else {
                self.shared.push(a);
            }
        }
    }

    /// The sequent both premises of a two-premise rule give together: the
    /// unrestricted zones united, the linear zones summed, absorbing if
    /// either is.
    fn join(&mut self, mut l: Box<State>, mut r: Box<State>) -> Box<State> {
        self.unite(&mut l, &mut r);
        l.gamma.add(std::mem::take(&mut r.gamma));
        l.outputs = l.outputs.saturating_add(r.outputs);
        l.linear = l.linear.saturating_add(r.linear);
        l.goal = l.goal.saturating_add(r.goal);
        l.any |= r.any;
        l
    }

    /// Carries the eigenvariables to node `id` from its premises, whose
    /// sets go with their last reader and are copied for the others:
    /// those a `∀` step introduces stay behind, those an `∃` step's
    /// witness or a cut's frame names come in.
    fn flow(&mut self, id: NodeId) -> Result<(), Halt> {
        let node = self.proof.node(id);
        let mut set: Option<Box<Eigens>> = None;
        for p in node.premises() {
            let theirs = if self.readers[p.index()] == 0 {
                self.eigens[p.index()].take()
            } else {
                let copy = self.eigens[p.index()].clone();
                if let Some(copy) = &copy {
                    self.charge(copy.bytes())?;
                }
                copy
            };
            match (&mut set, theirs) {
                (Some(ours), Some(theirs)) => {
                    // Both were counted; what is left is the one they make.
                    let before = ours.bytes().saturating_add(theirs.bytes());
                    ours.add(*theirs);
                    let after = ours.bytes();
                    if after > before {
                        self.charge(after - before)?;
                    } else {
                        self.held = self.held.saturating_sub(before - after);
                    }
                }
                (None, theirs) => set = theirs,
                (Some(_), None) => {}
            }
        }
        if let Node::Forall(_, a, _) = node
            && let Some(set) = &mut set
        {
            set.set.remove(&a);
        }
        // The walk of the terms is work like any other, counted and
        // polled for: a chain of witnesses each holding the last makes it
        // quadratic in the proof.
        let mut brought = Vec::new();
        let units = self.inst.bring(node, &mut brought);
        self.done = self.done.saturating_add(units);
        if !brought.is_empty() {
            let set = set.get_or_insert_with(|| Box::new(Eigens::default()));
            let before = set.bytes();
            for a in brought {
                set.insert(a);
            }
            let after = set.bytes();
            self.charge(after.saturating_sub(before))?;
        }
        if self.readers[id.index()] > 0 || id == self.proof.root() {
            self.eigens[id.index()] = set;
        } else if let Some(set) = set {
            self.held = self.held.saturating_sub(set.bytes());
        }
        Ok(())
    }

    /// Applies node `id`'s rule to what its premises derived, noting in
    /// `facts` how it applied.
    fn rule(&mut self, id: NodeId, node: Node, facts: &mut Facts<'_>) -> Result<Box<State>, Halt> {
        use Node::*;
        let f = self.forest;
        self.shared.clear();
        // The ordered calculus is unit-free MLL.
        if self.mode.is_ordered() && !matches!(node, Ax(..) | Par(..) | Tensor(..)) {
            return Err(Halt::Fault(Fault::Forbidden));
        }
        if let Some(fault) = self.inst.fault(id) {
            return Err(Halt::Fault(fault));
        }
        match node {
            Ax(a, b) => {
                for m in [a, b] {
                    if !f.is_literal(self.inst.occ(m)) {
                        return Err(Halt::Fault(Fault::Kind { member: m }));
                    }
                }
                if I::FRAMED {
                    // Reading an instance's atom walks its frame.
                    let frames = self
                        .inst
                        .frame_len(a)
                        .saturating_add(self.inst.frame_len(b));
                    self.done = self.done.saturating_add(frames);
                }
                if !self.inst.dual_literals(f, a, b) {
                    return Err(Halt::Fault(Fault::NotDual));
                }
                Ok(self.just([a, b], false))
            }
            One(m) => {
                self.expect(m, self.inst.occ(m), Kind::One)?;
                Ok(self.just([m], false))
            }
            Top(m) => {
                self.expect(m, self.inst.occ(m), Kind::Top)?;
                Ok(self.just([m], true))
            }
            Bot(m, p) => {
                self.expect(m, self.inst.occ(m), Kind::Bot)?;
                let mut d = self.premise(p)?;
                self.put(&mut d, m);
                Ok(d)
            }
            Par(m, p) => {
                let o = self.inst.occ(m);
                self.expect(m, o, Kind::Par)?;
                let mut d = self.premise(p)?;
                facts.absent[0] = self.take(&mut d, self.left(m, o), 0)?;
                facts.absent[1] = self.take(&mut d, self.right(m, o), 0)?;
                self.put(&mut d, m);
                Ok(d)
            }
            Tensor(m, l, r) => {
                let o = self.inst.occ(m);
                self.expect(m, o, Kind::Tensor)?;
                let (mut dl, mut dr) = (self.premise(l)?, self.premise(r)?);
                facts.needs = [dl.theta.len(), dr.theta.len()];
                facts.absent[0] = self.take(&mut dl, self.left(m, o), 0)?;
                facts.absent[1] = self.take(&mut dr, self.right(m, o), 1)?;
                if self.mode.is_ordered()
                    && !arcs(self.forest, o, &dl.gamma, &dr.gamma, |x| self.inst.occ(x))
                {
                    return Err(Halt::Fault(Fault::Order));
                }
                let mut d = self.join(dl, dr);
                self.put(&mut d, m);
                Ok(d)
            }
            With(m, l, r) => {
                let o = self.inst.occ(m);
                self.expect(m, o, Kind::With)?;
                let (mut dl, mut dr) = (self.premise(l)?, self.premise(r)?);
                facts.needs = [dl.theta.len(), dr.theta.len()];
                facts.absent[0] = self.take(&mut dl, self.left(m, o), 0)?;
                facts.absent[1] = self.take(&mut dr, self.right(m, o), 1)?;
                // The conclusion's context is what both premises can prove
                // it under: an absorbing premise adapts to the other one.
                let mut d = match (dl.any, dr.any) {
                    (false, false) if dl.gamma == dr.gamma => self.over(dl, dr),
                    (true, false) if dl.gamma.is_subset(&dr.gamma) => self.over(dr, dl),
                    (false, true) if dr.gamma.is_subset(&dl.gamma) => self.over(dl, dr),
                    (true, true) => {
                        let gamma = std::mem::take(&mut dr.gamma);
                        dl.gamma.unite(gamma);
                        let mut d = self.over(dl, dr);
                        d.any = true;
                        self.recount(&mut d);
                        d
                    }
                    _ => return Err(Halt::Fault(Fault::Differ)),
                };
                self.shared.clear();
                self.put(&mut d, m);
                Ok(d)
            }
            Plus(m, side, p) => {
                let o = self.inst.occ(m);
                self.expect(m, o, Kind::Plus)?;
                let mut d = self.premise(p)?;
                let chosen = match side {
                    Branch::Left => self.left(m, o),
                    Branch::Right => self.right(m, o),
                };
                facts.absent[0] = self.take(&mut d, chosen, 0)?;
                self.put(&mut d, m);
                Ok(d)
            }
            Bang(m, p) => {
                let o = self.inst.occ(m);
                self.expect(m, o, Kind::Bang)?;
                let mut d = self.premise(p)?;
                facts.absent[0] = self.take(&mut d, self.left(m, o), 0)?;
                if !d.gamma.is_empty() {
                    return Err(Halt::Fault(Fault::NotEmpty));
                }
                // Promotion fixes the linear zone: a ⊤ above cannot absorb
                // past it.
                d.any = false;
                self.put(&mut d, m);
                Ok(d)
            }
            Quest(m, p) => {
                let o = self.inst.occ(m);
                self.expect(m, o, Kind::Quest)?;
                let mut d = self.premise(p)?;
                if d.theta.remove(self.left(m, o)) {
                    // What the member added when it came in.
                    facts.used = true;
                    d.unrestricted -= u64::from(self.observer.weight(m, o));
                }
                self.put(&mut d, m);
                Ok(d)
            }
            Copy(a, p) => {
                let o = self.inst.occ(a);
                if f.parent(o).map(|q| f.kind(q)) != Some(Kind::Quest) {
                    return Err(Halt::Fault(Fault::NotUnderQuest { member: a }));
                }
                let mut d = self.premise(p)?;
                facts.absent[0] = self.take(&mut d, a, 0)?;
                if d.theta.insert(a) {
                    d.unrestricted += self.quest_weight(a);
                } else {
                    facts.used = true;
                }
                Ok(d)
            }
            Weaken(m, p) => {
                let o = self.inst.occ(m);
                // Weakening a `?` formula is a rule of every mode; the goal
                // is never weakened.
                if !self.mode.affine && f.kind(o) != Kind::Quest {
                    return Err(Halt::Fault(Fault::Forbidden));
                }
                if self.is_output(o) {
                    return Err(Halt::Fault(Fault::Succedents { count: 0 }));
                }
                let mut d = self.premise(p)?;
                self.put(&mut d, m);
                Ok(d)
            }
            Mix(l, r) => {
                // Mix has no intuitionistic form: a premise would lack the
                // goal.
                if !self.mode.mix || self.mode.intuitionistic {
                    return Err(Halt::Fault(Fault::Forbidden));
                }
                let (dl, dr) = (self.premise(l)?, self.premise(r)?);
                facts.needs = [dl.theta.len(), dr.theta.len()];
                Ok(self.join(dl, dr))
            }
            Cut(a, l, r) => {
                // The ordered modes refused it above: the dual of an
                // ordered cut formula is not of one shape with it.
                let o = self.inst.occ(a);
                let dual = match self.inst.dual(f, a) {
                    Some(dual) if o < self.inst.occ(dual) => dual,
                    _ => return Err(Halt::Fault(Fault::NotACut { member: a })),
                };
                let (mut dl, mut dr) = (self.premise(l)?, self.premise(r)?);
                facts.needs = [dl.theta.len(), dr.theta.len()];
                facts.absent[0] = self.take(&mut dl, a, 0)?;
                facts.absent[1] = self.take(&mut dr, dual, 1)?;
                // A cut on an instance chooses its frame's terms, as an `∃`
                // step its witness: the frame pass found them, and `flow`
                // brings them in.
                Ok(self.join(dl, dr))
            }
            Forall(m, _, p) => {
                self.expect(m, self.inst.occ(m), Kind::Forall)?;
                let mut d = self.premise(p)?;
                facts.absent[0] = self.take(&mut d, self.inst.body(id), 0)?;
                // Every witness above that names its eigenvariable is
                // inside this step, which `flow` sees to.
                self.put(&mut d, m);
                Ok(d)
            }
            Exists(m, _, p) => {
                self.expect(m, self.inst.occ(m), Kind::Exists)?;
                let mut d = self.premise(p)?;
                facts.absent[0] = self.take(&mut d, self.inst.body(id), 0)?;
                self.put(&mut d, m);
                Ok(d)
            }
        }
    }

    /// The conclusion of a `&` whose linear zone is that of `keep`: its
    /// sequent with the unrestricted zone of `other` united into it.
    fn over(&mut self, mut keep: Box<State>, mut other: Box<State>) -> Box<State> {
        self.unite(&mut keep, &mut other);
        keep
    }

    /// Counts the outputs and the weights of the linear zone of `d` anew,
    /// after it was replaced.
    fn recount(&self, d: &mut State) {
        let (mut outputs, mut linear, mut goal) = (0usize, 0u64, 0u64);
        for (o, n) in d.gamma.counts() {
            // Two factors below 2³².
            let occ = self.inst.occ(o);
            let weight = u64::from(self.observer.weight(o, occ)) * u64::from(n);
            linear = linear.saturating_add(weight);
            if self.is_output(occ) {
                outputs = outputs.saturating_add(n as usize);
                goal = goal.saturating_add(weight);
            }
        }
        (d.outputs, d.linear, d.goal) = (outputs, linear, goal);
    }
}
