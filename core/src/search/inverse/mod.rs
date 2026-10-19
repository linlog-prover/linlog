// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The focused inverse method: a forward search that derives stable
//! sequents from the axioms upward, by derived rules that are whole
//! focusing phases, and keeps each sequent unless one kept already
//! subsumes it, until a derived sequent proves a stable sequent of the
//! goal's asynchronous phase, every one of them, or nothing new comes.
//! Without exponentials every sequent of a proof holds each occurrence once
//! at most, so the sequents are finitely many and the search decides; with
//! them a sequent may hold copies, and the search proves what is provable
//! (it takes the smallest sequent first, so every sequent is reached) and
//! refutes only where the saturation ends.
//!
//! A derived sequent `Γ ; Θ` stands for exactly its linear zone `Γ` with
//! any unrestricted zone that holds `Θ`, or, where a `⊤` above absorbs
//! (`top`) or in affine mode, for any larger linear zone too, as the
//! checker reads a proof's sequents. One sequent subsumes another when it
//! stands for every sequent the other does: the same `Γ` and a smaller
//! `Θ`, or, absorbing or weakening, a smaller `Γ` as well.

/// The forward evaluation of a rule's term.
mod eval;
/// The index of the absorbing and weakening sequents.
mod index;
/// The derived rules, compiled from the forest.
mod rules;
#[cfg(all(test, feature = "parse"))]
mod tests;

use super::memory::{Account, Charged, bytes_of};
use super::{Answer, Decide, Engine, Finished, NotTaken, Options, Reason, Statistics, Task, Work};
use crate::Error;
use crate::fragment::Fragment;
use crate::hash::HashMap;
use crate::limits::Limits;
use crate::occurrences::OccId;
use crate::proofs::{Node, NodeId};
use eval::{Calculus, Zone, within};
use rules::{Rules, Shape};
use std::cmp::Reverse;
use std::collections::BinaryHeap;

/// The engine, as [`Engine::Inverse`] names it.
pub(crate) struct Inverse;

impl Decide for Inverse {
    /// Refuses a goal beyond full linear logic and the modes that keep the
    /// order of the formulas.
    fn admits(&self, task: &Task<'_>) -> Result<(), Error> {
        let because = if !Fragment::LL.contains(task.fragment) {
            NotTaken::Fragment {
                decides: Fragment::LL,
                goal: task.fragment,
            }
        } else if task.mode.is_ordered() {
            NotTaken::Mode { mode: task.mode }
        } else {
            return Ok(());
        };
        Err(Error::EngineRefused {
            engine: Engine::Inverse,
            because,
        })
    }

    /// Saturates on the calling thread, whatever [`Options::jobs`] says.
    fn decide(
        &self,
        task: &Task<'_>,
        options: &Options,
        limits: &Limits,
        account: &Account,
        _work: &Work,
        stop: &mut dyn FnMut(u64) -> bool,
    ) -> Result<Answer, Error> {
        let mut found = search(task, options, account, stop);
        found.result = found.result.map_err(|r| r.as_set(options, limits));
        Ok(Answer::of_arena(task.forest, found, stop))
    }
}

/// How many rule applications the search makes between two polls of its
/// stop, besides the poll at every sequent it takes up: some tens of
/// microseconds.
const APPLICATIONS_PER_POLL: u64 = 1 << 10;

/// How many kept sequents' terms the proof is written for between two
/// polls of the stop.
const BUILT_PER_POLL: usize = 1 << 12;

/// The tag of a record of Mix, beside the records of rules, which are the
/// rule's index.
const MIX: u32 = u32::MAX;

/// Runs the search on a task, on the calling thread.
fn search(
    task: &Task<'_>,
    options: &Options,
    account: &Account,
    stop: &mut dyn FnMut(u64) -> bool,
) -> Finished {
    let signs = super::focus::bias::signs(task.forest, options.bias);
    let shape = Shape::new(
        task.forest,
        task.goal,
        signs,
        task.reading,
        task.mode.is_affine(),
    );
    let mut charged = Charged::new(account);
    let goal = match rules::goal(&shape, &mut charged, stop) {
        Ok(goal) => goal,
        Err(reason) => return Finished::gave_up(reason),
    };
    let derived = match rules::compile(&shape, &mut charged, stop) {
        Ok(rules) => rules,
        Err(reason) => return Finished::gave_up(reason),
    };
    let mut saturation = Saturation::new(
        &shape,
        &derived,
        &goal,
        task.mode.has_mix() && !task.mode.is_affine(),
        account,
        stop,
    );
    let result = saturation.run();
    let statistics = saturation.statistics;
    match result {
        Ok(Some(premises)) => match saturation.build(&premises) {
            Ok((root, nodes)) => Finished {
                result: Ok(Some(root)),
                nodes,
                statistics,
            },
            Err(reason) => Finished {
                result: Err(reason),
                nodes: Vec::new(),
                statistics,
            },
        },
        Ok(None) => Finished {
            result: Ok(None),
            nodes: Vec::new(),
            statistics,
        },
        Err(reason) => Finished {
            result: Err(reason),
            nodes: Vec::new(),
            statistics,
        },
    }
}

/// A sequent kept: where its zones lie in the store, whether it absorbs,
/// whether it was taken up (its rank among those taken up) or subsumed
/// since, and where its record lies.
#[derive(Clone, Copy, Debug)]
struct Kept {
    /// The start of its linear zone, then its unrestricted zone, in the
    /// store.
    start: usize,
    /// The members of its linear zone.
    gamma: u32,
    /// The members of its unrestricted zone.
    theta: u32,
    /// Whether a `⊤` above absorbs any further linear member.
    top: bool,
    /// The members of its linear zone in output position.
    outputs: u64,
    /// Its rank among the sequents taken up, or `u32::MAX` while it waits:
    /// below its id, which is below `u32::MAX` ([`Saturation::keep`]).
    rank: u32,
    /// Whether a sequent kept after it subsumes it.
    dead: bool,
    /// The start of its record: the rule and its premises and partners, or
    /// Mix and its two sequents.
    record: usize,
}

impl Kept {
    /// The number of members of both zones: the order in which sequents
    /// are taken up.
    fn size(&self) -> u64 {
        u64::from(self.gamma) + u64::from(self.theta)
    }
}

/// The search's state: the sequents kept, their indices, the queue of
/// those to take up, and the goal's stable sequents proved.
struct Saturation<'s, 'a> {
    /// The calculus the rules are read in.
    calculus: Calculus<'s, 'a>,
    /// The goal's shape.
    shape: &'s Shape<'a>,
    /// The derived rules.
    rules: &'s Rules,
    /// The goal's phase.
    goal: &'s Rules,
    /// Whether Mix joins two sequents.
    mix: bool,
    /// Every sequent kept, by its id.
    kept: Vec<Kept>,
    /// The kept sequents' zones.
    store: Vec<OccId>,
    /// The kept sequents' records.
    records: Vec<u32>,
    /// The sequents waiting to be taken up, smallest first, then oldest.
    queue: BinaryHeap<Reverse<(u64, u32)>>,
    /// The exact sequents kept, by their zones.
    exact: index::Trie,
    /// The absorbing or weakening sequents kept, by their linear zone.
    weak: index::Trie,
    /// The sequents taken up, by rank.
    active: Vec<u32>,
    /// The sequents taken up, by every member of their linear zone.
    postings: HashMap<OccId, Vec<u32>>,
    /// The absorbing or weakening sequents taken up.
    weak_active: Vec<u32>,
    /// The goal's holes by every member they take.
    goal_index: HashMap<OccId, Vec<u32>>,
    /// The sequent proving each of the goal's holes, once one is kept.
    filled: Vec<Option<u32>>,
    /// The goal's holes not yet proved.
    unfilled: usize,
    /// The bytes of the lists in the tables, which their tables' own
    /// capacity does not count.
    lists: usize,
    /// The charge for everything the search holds.
    charged: Charged<'s>,
    /// The bytes charged.
    held: usize,
    /// A stack for evaluating a rule's term.
    stack: Vec<Zone>,
    /// The work done since the last poll.
    unpolled: u64,
    /// The counters.
    statistics: Statistics,
    /// The caller's stop.
    stop: &'s mut dyn FnMut(u64) -> bool,
}

/// Pushes onto a list in a table, returning how many bytes its allocation
/// grew by.
fn grow(list: &mut Vec<u32>, id: u32) -> usize {
    let before = bytes_of(list);
    list.push(id);
    bytes_of(list) - before
}

/// Returns the members of a sorted list without its repeats.
fn distinct(list: &[OccId]) -> impl Iterator<Item = OccId> + '_ {
    list.iter()
        .enumerate()
        .filter(|&(i, o)| i == 0 || list[i - 1] != *o)
        .map(|(_, &o)| o)
}

impl<'s, 'a> Saturation<'s, 'a> {
    /// A search with nothing kept.
    fn new(
        shape: &'s Shape<'a>,
        rules: &'s Rules,
        goal: &'s Rules,
        mix: bool,
        account: &'s Account,
        stop: &'s mut dyn FnMut(u64) -> bool,
    ) -> Self {
        let mut goal_index: HashMap<OccId, Vec<u32>> = HashMap::default();
        for (h, hole) in goal.holes.iter().enumerate() {
            let leaves = &goal.leaves[hole.leaves.start as usize..hole.leaves.end as usize];
            for o in distinct(leaves) {
                goal_index.entry(o).or_default().push(h as u32);
            }
        }
        Self {
            calculus: Calculus { shape },
            shape,
            rules,
            goal,
            mix,
            kept: Vec::new(),
            store: Vec::new(),
            records: Vec::new(),
            queue: BinaryHeap::new(),
            exact: index::Trie::default(),
            weak: index::Trie::default(),
            active: Vec::new(),
            postings: HashMap::default(),
            weak_active: Vec::new(),
            goal_index,
            filled: vec![None; goal.holes.len()],
            unfilled: goal.holes.len(),
            lists: 0,
            charged: Charged::new(account),
            held: 0,
            stack: Vec::new(),
            unpolled: 0,
            statistics: Statistics::default(),
            stop,
        }
    }

    /// Whether a sequent stands for larger linear zones too.
    fn weak(&self, top: bool) -> bool {
        top || self.shape.affine
    }

    /// The linear zone of a kept sequent.
    fn gamma(&self, id: u32) -> &[OccId] {
        let k = &self.kept[id as usize];
        &self.store[k.start..k.start + k.gamma as usize]
    }

    /// The unrestricted zone of a kept sequent, its globals left out.
    fn theta(&self, id: u32) -> &[OccId] {
        let k = &self.kept[id as usize];
        let at = k.start + k.gamma as usize;
        &self.store[at..at + k.theta as usize]
    }

    /// The zone of a kept sequent, with the node proving it.
    fn zone(&self, id: u32, node: Option<NodeId>, into: &mut Zone) {
        let k = self.kept[id as usize];
        into.gamma.clear();
        into.gamma.extend_from_slice(self.gamma(id));
        into.theta.clear();
        into.theta.extend_from_slice(self.theta(id));
        into.top = k.top;
        into.outputs = k.outputs;
        into.node = node;
    }

    /// Charges what the tables hold, and answers the memory bound.
    fn settle(&mut self) -> Result<(), Reason> {
        /// The bytes a table's slot takes beside its entry.
        const CONTROL: usize = 1;
        let held = bytes_of(&self.kept)
            + bytes_of(&self.store)
            + bytes_of(&self.records)
            + self.queue.capacity() * size_of::<Reverse<(u64, u32)>>()
            + self.exact.bytes()
            + self.weak.bytes()
            + bytes_of(&self.active)
            + self.postings.capacity() * (size_of::<(OccId, Vec<u32>)>() + CONTROL)
            + bytes_of(&self.weak_active)
            + self.lists;
        self.charged.resize(self.held, held);
        self.held = held;
        if self.charged.account().over() {
            return Err(Reason::MemoryLimit {
                limit_bytes: self.charged.account().limit(),
            });
        }
        Ok(())
    }

    /// Counts work, and polls the stop once enough has been done or when
    /// `now`.
    fn work(&mut self, units: u64, now: bool) -> Result<(), Reason> {
        self.unpolled += units;
        if (now || self.unpolled >= APPLICATIONS_PER_POLL)
            && (self.stop)(std::mem::take(&mut self.unpolled))
        {
            return Err(Reason::Stopped);
        }
        Ok(())
    }

    /// Whether a kept sequent other than `except` subsumes the zone (its
    /// globals left out).
    fn subsumed(&self, z: &Zone, except: Option<u32>) -> bool {
        let alive = |t: u32| Some(t) != except && !self.kept[t as usize].dead;
        (!self.weak(z.top) && self.exact.same_within(&z.gamma, &z.theta, alive))
            || self.weak.within(&z.gamma, &z.theta, alive)
    }

    /// Whether every two members of a zone can stand in one sequent of a
    /// proof of the goal: those consecutive in the order of the forest
    /// (whose common ancestors are every two members' common ancestors)
    /// are coherent.
    fn coherent(&self, z: &Zone) -> bool {
        let mut all: Vec<OccId> = z.gamma.iter().chain(&z.theta).copied().collect();
        all.sort_unstable();
        all.windows(2).all(|w| self.shape.coherent(w[0], w[1]))
    }

    /// Keeps a derived sequent unless a kept one subsumes it or it cannot
    /// stand in a proof of the goal, with its record; fills the goal's
    /// holes it proves.
    fn keep(&mut self, mut z: Zone, record: &[u32]) -> Result<(), Reason> {
        z.theta
            .retain(|o| self.shape.globals.binary_search(o).is_err());
        if !self.coherent(&z) {
            return Ok(());
        }
        if self.subsumed(&z, None) {
            self.statistics.memo_hits += 1;
            return Ok(());
        }
        // Ids, ranks and the lengths of zones are `u32`s: a search without a
        // memory bound that passes them ends here.
        let id = u32::try_from(self.kept.len())
            .ok()
            .filter(|&id| id < u32::MAX)
            .ok_or(Reason::IndexLimit)?;
        let (Ok(gamma), Ok(theta)) = (u32::try_from(z.gamma.len()), u32::try_from(z.theta.len()))
        else {
            return Err(Reason::IndexLimit);
        };
        let start = self.store.len();
        self.store.extend_from_slice(&z.gamma);
        self.store.extend_from_slice(&z.theta);
        let at = self.records.len();
        self.records.extend_from_slice(record);
        let kept = Kept {
            start,
            gamma,
            theta,
            top: z.top,
            outputs: z.outputs,
            rank: u32::MAX,
            dead: false,
            record: at,
        };
        self.kept.push(kept);
        let index = if self.weak(z.top) {
            &mut self.weak
        } else {
            &mut self.exact
        };
        if !index.insert(&z.gamma, &z.theta, id) {
            return Err(Reason::IndexLimit);
        }
        self.queue.push(Reverse((kept.size(), id)));
        self.statistics.memo_entries = self.kept.len() as u64;
        self.fill(id);
        self.settle()
    }

    /// Fills the goal's holes the kept sequent proves: its linear zone
    /// within the hole's members (all of them unless it absorbs or
    /// weakens) and its unrestricted zone within the hole's.
    fn fill(&mut self, id: u32) {
        let candidates: Vec<u32> = match self.gamma(id).first() {
            Some(o) => self.goal_index.get(o).cloned().unwrap_or_default(),
            None => (0..self.goal.holes.len() as u32).collect(),
        };
        let weak = self.weak(self.kept[id as usize].top);
        for h in candidates {
            if self.filled[h as usize].is_some() {
                continue;
            }
            let hole = &self.goal.holes[h as usize];
            let leaves = &self.goal.leaves[hole.leaves.start as usize..hole.leaves.end as usize];
            let theta = &self.goal.leaves[hole.theta.start as usize..hole.theta.end as usize];
            let gamma = self.gamma(id);
            // A member left out is absorbed, or weakened in where it is no
            // output.
            let top = self.kept[id as usize].top;
            if within(gamma, leaves)
                && (weak || gamma.len() == leaves.len())
                && (top
                    || eval::excess(leaves, gamma)
                        .iter()
                        .all(|&o| !self.shape.output(o)))
                && within(self.theta(id), theta)
            {
                self.filled[h as usize] = Some(id);
                self.unfilled -= 1;
            }
        }
    }

    /// Whether a zone derived by the goal's phase is the goal.
    fn concludes(&self, z: &Zone) -> bool {
        z.theta.is_empty()
            && (z.gamma == self.shape.goal || (z.top && within(&z.gamma, &self.shape.goal)))
    }

    /// Runs the search: the rules without premises first, then the
    /// sequents kept, smallest first. Returns the sequents proving the
    /// goal's holes, `None` when the saturation ended without them.
    fn run(&mut self) -> Result<Option<Vec<u32>>, Reason> {
        let rules = self.rules;
        for r in 0..rules.all.len() as u32 {
            if self.unfilled == 0 {
                break;
            }
            if rules.all[r as usize].holes.is_empty() {
                self.apply(r, None)?;
            }
        }
        while self.unfilled > 0 {
            let Some(Reverse((_, id))) = self.queue.pop() else {
                return Ok(None);
            };
            if self.kept[id as usize].dead {
                continue;
            }
            let mut z = Zone::default();
            self.zone(id, None, &mut z);
            if self.subsumed(&z, Some(id)) {
                self.kept[id as usize].dead = true;
                continue;
            }
            self.activate(id)?;
        }
        Ok(Some(
            self.filled
                .iter()
                .map(|f| f.expect("every hole is filled"))
                .collect(),
        ))
    }

    /// Takes a sequent up: indexes it among those taken up and applies
    /// every rule with a premise it can be, the others taken up before it,
    /// and Mix with every sequent taken up.
    fn activate(&mut self, id: u32) -> Result<(), Reason> {
        self.work(1, true)?;
        self.statistics.nodes += 1;
        let rank = self.active.len() as u32;
        self.kept[id as usize].rank = rank;
        self.lists += grow(&mut self.active, id);
        let members: Vec<OccId> = distinct(self.gamma(id)).collect();
        for &o in &members {
            self.lists += grow(self.postings.entry(o).or_default(), id);
        }
        let weak = self.weak(self.kept[id as usize].top);
        if weak {
            self.lists += grow(&mut self.weak_active, id);
        }
        self.settle()?;
        let holes: Vec<u32> = if weak {
            (0..self.rules.holes.len() as u32).collect()
        } else {
            members
                .iter()
                .filter_map(|o| self.rules.by_leaf.get(o))
                .flatten()
                .chain(&self.rules.open)
                .copied()
                .collect()
        };
        for h in holes {
            let hole = &self.rules.holes[h as usize];
            let leaves = &self.rules.leaves[hole.leaves.start as usize..hole.leaves.end as usize];
            if !weak && !within(leaves, self.gamma(id)) {
                continue;
            }
            self.apply(hole.rule, Some((hole.index, id)))?;
            if self.unfilled == 0 {
                return Ok(());
            }
        }
        if self.mix {
            for rank in 0..=rank {
                let t = self.active[rank as usize];
                if self.kept[t as usize].dead {
                    continue;
                }
                self.work(1, false)?;
                let (mut l, mut r) = (Zone::default(), Zone::default());
                self.zone(t, None, &mut l);
                self.zone(id, None, &mut r);
                if let Some(z) = self.calculus.mix(&l, &r, None) {
                    self.keep(z, &[MIX, t, id])?;
                }
                if self.unfilled == 0 {
                    return Ok(());
                }
            }
        }
        Ok(())
    }

    /// The sequents taken up that can go into a hole of a rule beside the
    /// sequent of rank `rank`: before it in the rule's holes, those taken
    /// up before it; after it, it too.
    fn candidates(&self, hole: u32, rank: u32, before: bool) -> Vec<u32> {
        let hole = &self.rules.holes[hole as usize];
        let leaves = &self.rules.leaves[hole.leaves.start as usize..hole.leaves.end as usize];
        let fits = |t: &u32| {
            let k = &self.kept[*t as usize];
            !k.dead
                && if before {
                    k.rank < rank
                } else {
                    k.rank <= rank
                }
        };
        if leaves.is_empty() {
            return self.active.iter().copied().filter(fits).collect();
        }
        let shortest = distinct(leaves)
            .map(|o| self.postings.get(&o).map_or(&[][..], Vec::as_slice))
            .min_by_key(|list| list.len())
            .unwrap_or(&[]);
        let mut out: Vec<u32> = shortest
            .iter()
            .copied()
            .filter(|t| fits(t) && !self.weak(self.kept[*t as usize].top))
            .filter(|&t| within(leaves, self.gamma(t)))
            .collect();
        out.extend(self.weak_active.iter().copied().filter(fits));
        out
    }

    /// Applies a rule: with `fixed`, the sequent given goes into the hole
    /// at that place and the sequents taken up into the others; every
    /// choice of premises and partners is evaluated and what it derives
    /// kept. A choice of premises is given up as soon as its evaluation
    /// must fail ([`Saturation::fits`]).
    fn apply(&mut self, r: u32, fixed: Option<(u32, u32)>) -> Result<(), Reason> {
        let rules = self.rules;
        let rule = &rules.all[r as usize];
        let holes = rule.holes.len();
        let rank = fixed.map_or(u32::MAX, |(_, id)| self.kept[id as usize].rank);
        let mut lists: Vec<Vec<Premise>> = Vec::with_capacity(holes);
        for (j, h) in rule.holes.clone().enumerate() {
            let ids = match fixed {
                Some((i, id)) if i as usize == j => vec![id],
                Some((i, _)) => self.candidates(h, rank, (j as u32) < i),
                None => unreachable!("a rule with holes is applied with a premise"),
            };
            if ids.is_empty() {
                return Ok(());
            }
            lists.push(ids.into_iter().map(|id| self.premise(h, id)).collect());
        }
        let groups: Vec<u32> = rule
            .holes
            .clone()
            .map(|h| rules.holes[h as usize].group)
            .collect();
        let slots = &rules.slots[rule.slots.start as usize..rule.slots.end as usize];
        let steps = &rules.steps[rule.steps.start as usize..rule.steps.end as usize];
        let mut at = vec![0usize; holes];
        let mut partners = vec![0u32; slots.len()];
        let mut chosen = vec![0u32; holes];
        let mut record = Vec::with_capacity(1 + holes + slots.len());
        let mut j = 0;
        loop {
            if j == holes {
                for (c, (&p, list)) in chosen.iter_mut().zip(at.iter().zip(&lists)) {
                    *c = list[p].id;
                }
                loop {
                    self.work(1, false)?;
                    let mut stack = std::mem::take(&mut self.stack);
                    let derived = {
                        let this = &*self;
                        this.calculus.derive(
                            steps,
                            &mut |h, z| this.zone(chosen[h as usize], None, z),
                            &|s| {
                                rules.partners
                                    [(slots[s as usize].start + partners[s as usize]) as usize]
                            },
                            None,
                            &mut stack,
                        )
                    };
                    self.stack = stack;
                    if let Some(z) = derived {
                        record.clear();
                        record.push(r);
                        record.extend_from_slice(&chosen);
                        record.extend_from_slice(&partners);
                        self.keep(z, &record)?;
                        if self.unfilled == 0 {
                            return Ok(());
                        }
                    }
                    if !advance(&mut partners, |s| slots[s].len() as u32) {
                        break;
                    }
                }
                if holes == 0 {
                    return Ok(());
                }
                j = holes - 1;
                at[j] += 1;
            }
            // The next premise at place `j` that fits beside those before.
            while at[j] < lists[j].len() && !self.fits(&lists, &groups, &at, j) {
                self.work(1, false)?;
                at[j] += 1;
            }
            if at[j] < lists[j].len() {
                j += 1;
            } else if j == 0 {
                return Ok(());
            } else {
                at[j] = 0;
                j -= 1;
                at[j] += 1;
            }
        }
    }

    /// What a kept sequent brings into a hole: its context, the members the
    /// hole's phase does not take, and how many of them are outputs.
    fn premise(&self, hole: u32, id: u32) -> Premise {
        let hole = &self.rules.holes[hole as usize];
        let leaves = &self.rules.leaves[hole.leaves.start as usize..hole.leaves.end as usize];
        let context = eval::excess(self.gamma(id), leaves);
        let outputs = context.iter().filter(|&&o| self.shape.output(o)).count() as u64;
        Premise {
            id,
            exact: !self.weak(self.kept[id as usize].top),
            context,
            outputs,
        }
    }

    /// Whether the premise chosen at place `j` can stand beside those at
    /// the places before it: the contexts of one release's exact premises
    /// are equal, since its `&` steps join them and nothing else changes
    /// them (without weakening); and under a reading the releases bring
    /// one output at most, since the conclusion holds every context and
    /// takes the most outputs of each release's.
    fn fits(&self, lists: &[Vec<Premise>], groups: &[u32], at: &[usize], j: usize) -> bool {
        let p = &lists[j][at[j]];
        if !self.shape.affine && p.exact {
            for i in 0..j {
                let q = &lists[i][at[i]];
                if groups[i] == groups[j] && q.exact && q.context != p.context {
                    return false;
                }
            }
        }
        if self.shape.reading.is_some() {
            let mut most: Vec<(u32, u64)> = Vec::new();
            for i in 0..=j {
                let (g, out) = (groups[i], lists[i][at[i]].outputs);
                match most.iter_mut().find(|(h, _)| *h == g) {
                    Some((_, m)) => *m = (*m).max(out),
                    None => most.push((g, out)),
                }
            }
            if most.iter().map(|&(_, m)| m).sum::<u64>() > 1 {
                return false;
            }
        }
        true
    }

    /// Writes the proof: the kept sequents' terms the goal's holes rest on,
    /// each once, then the goal's phase over them. Returns the root and the
    /// arena.
    fn build(&mut self, premises: &[u32]) -> Result<(NodeId, Vec<Node>), Reason> {
        let mut nodes: Vec<Node> = Vec::new();
        let mut done: HashMap<u32, NodeId> = HashMap::default();
        let mut charged = Charged::new(self.charged.account());
        let mut todo: Vec<u32> = premises.to_vec();
        while let Some(&t) = todo.last() {
            if done.contains_key(&t) {
                todo.pop();
                continue;
            }
            let k = self.kept[t as usize];
            let tag = self.records[k.record];
            let needs: Vec<u32> = if tag == MIX {
                self.records[k.record + 1..k.record + 3].to_vec()
            } else {
                let rule = &self.rules.all[tag as usize];
                let n = rule.holes.len();
                self.records[k.record + 1..k.record + 1 + n].to_vec()
            };
            let missing: Vec<u32> = needs
                .iter()
                .copied()
                .filter(|p| !done.contains_key(p))
                .collect();
            if !missing.is_empty() {
                todo.extend(missing);
                continue;
            }
            todo.pop();
            let before = bytes_of(&nodes);
            let z = if tag == MIX {
                let (mut l, mut r) = (Zone::default(), Zone::default());
                self.zone(needs[0], Some(done[&needs[0]]), &mut l);
                self.zone(needs[1], Some(done[&needs[1]]), &mut r);
                self.calculus.mix(&l, &r, Some(&mut nodes))
            } else {
                let rules = self.rules;
                let rule = &rules.all[tag as usize];
                let slots = &rules.slots[rule.slots.start as usize..rule.slots.end as usize];
                let n = rule.holes.len();
                let chosen = &self.records[k.record + 1 + n..k.record + 1 + n + slots.len()];
                let steps = &rules.steps[rule.steps.start as usize..rule.steps.end as usize];
                let mut stack = Vec::new();
                self.calculus.derive(
                    steps,
                    &mut |h, z| self.zone(needs[h as usize], Some(done[&needs[h as usize]]), z),
                    &|s| rules.partners[(slots[s as usize].start + chosen[s as usize]) as usize],
                    Some(&mut nodes),
                    &mut stack,
                )
            };
            let z = z.expect("a kept sequent's rule derives it again");
            debug_assert!(
                z.gamma == self.gamma(t) && z.top == k.top,
                "the proof of a kept sequent derives its zones"
            );
            done.insert(t, z.node.expect("a node is written"));
            charged.resize(before, bytes_of(&nodes));
            if charged.account().over() {
                return Err(Reason::MemoryLimit {
                    limit_bytes: charged.account().limit(),
                });
            }
            if done.len().is_multiple_of(BUILT_PER_POLL) && (self.stop)(0) {
                return Err(Reason::Stopped);
            }
        }
        let goal = self.goal;
        let steps = &goal.steps[..];
        let mut stack = Vec::new();
        let z = self
            .calculus
            .derive(
                steps,
                &mut |h, z| {
                    let t = premises[h as usize];
                    self.zone(t, Some(done[&t]), z);
                },
                &|_| unreachable!("the goal's phase has no axioms"),
                Some(&mut nodes),
                &mut stack,
            )
            .expect("the goal's phase derives the goal again");
        let mut z = z;
        let goal = &self.shape.goal;
        if self.shape.affine && !z.top && within(&z.gamma, goal) {
            // What the premises weakened away, weakened in below the root.
            for o in eval::excess(goal, &z.gamma) {
                let weakened = self.calculus.weaken_in(&mut z, o, &mut nodes);
                debug_assert!(weakened, "only inputs go unused under a reading");
            }
        }
        debug_assert!(self.concludes(&z), "the proof concludes the goal");
        Ok((z.node.expect("a node is written"), nodes))
    }
}

/// A kept sequent as the premise of a hole, with what the evaluation of
/// the rule asks of it.
struct Premise {
    /// The kept sequent.
    id: u32,
    /// Whether it stands for its linear zone alone.
    exact: bool,
    /// Its linear zone less the hole's members.
    context: Vec<OccId>,
    /// The members of its context in output position.
    outputs: u64,
}

/// Advances an odometer of choices, the first place fastest, each place
/// below its bound; returns false after the last choice.
fn advance(places: &mut [u32], bound: impl Fn(usize) -> u32) -> bool {
    for (i, p) in places.iter_mut().enumerate() {
        *p += 1;
        if *p < bound(i) {
            return true;
        }
        *p = 0;
    }
    false
}
