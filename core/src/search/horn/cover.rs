// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Coverability by the backward algorithm: the markings from which some
//! firing sequence covers the target form an upward-closed set, kept as
//! its minimal elements, the basis. It starts from the target, and every
//! element `m` and transition `t` give the least marking from which `t`
//! leads to a marking at least `m`, `max(m − out(t), 0) + in(t)`, which
//! joins the basis unless an element is already at most it. A marking at
//! most the initial one proves the target coverable, and its line of
//! descent is the firing sequence; a basis that no transition adds to is
//! the whole set, and the initial marking outside it refutes.
//!
//! It ends: every element added is at least no element added before it
//! (an element at most it would have kept it out, or one at most that
//! one, which was kept), and an infinite sequence of count vectors in
//! which no element is at least an earlier one does not exist (Dickson's
//! lemma). How long it takes, Dickson does not bound: the memory bound
//! and the caller's stop do.

use super::Program;
use super::equation::{ENTRIES_PER_UNIT, Equation};
use super::reach::room;
use crate::search::memory::{Account, Charged};
use crate::search::{Reason, Statistics};
use std::cmp::Reverse;
use std::collections::BinaryHeap;

/// The parent of the target, the first element.
const ROOT: u32 = u32::MAX;

/// Searches backward from the program's target for a marking at most the
/// initial one, polling `stop` at every element taken from the queue,
/// keeping at most `most` elements and giving `equation` its share of the
/// work; returns the transitions of a firing sequence from the initial
/// marking that covers the target, `None` when no marking reached
/// covers it or the state equation refutes, or the reason it stopped;
/// and the counters: the markings computed as `nodes`, those kept out by
/// an element at most them as `memo_hits`, the elements kept as
/// `memo_entries`.
pub(super) fn search(
    program: &Program,
    account: &Account,
    most: usize,
    equation: &mut Equation<'_>,
    stop: &mut dyn FnMut() -> bool,
) -> (Result<Option<Vec<u32>>, Reason>, Statistics) {
    let mut search = Search::new(program, account);
    let result = search.run(most, equation, stop);
    let statistics = Statistics {
        nodes: search.computed,
        memo_hits: search.covered,
        memo_entries: search.parents.len(),
        ..Statistics::default()
    };
    (result, statistics)
}

/// The state of a search: the elements kept, their index by their first
/// place, the queue, the index of the transitions by their output places,
/// and the counters.
struct Search<'a> {
    /// The program.
    program: &'a Program,
    /// The elements, one after the other, each as its places in
    /// increasing order with their counts, all above zero.
    entries: Vec<(u32, u32)>,
    /// Where each element's entries end.
    ends: Vec<usize>,
    /// The tokens of each element.
    sums: Vec<u64>,
    /// The element each came from and the transition, as
    /// `parent << 32 | transition`; [`ROOT`] above for the target.
    parents: Vec<u64>,
    /// The elements by their first place: an element at most a marking
    /// has its first place marked there.
    by_first: Vec<Vec<u32>>,
    /// The elements to take, fewest tokens first.
    queue: BinaryHeap<Reverse<(u64, u32)>>,
    /// The transitions with an output to each place: those of place `p`
    /// are `by_output[starts[p]..starts[p + 1]]`.
    starts: Vec<u32>,
    /// The transitions by their output places.
    by_output: Vec<u32>,
    /// The element each transition was last tried on, plus one.
    tried: Vec<u32>,
    /// A marking tested against the elements, as a count per place.
    probe: Vec<u32>,
    /// A buffer for a computed marking.
    next: Vec<(u32, u32)>,
    /// What the buffers that grow hold.
    charged: Charged<'a>,
    /// The markings computed.
    computed: u64,
    /// Those an element at most them kept out.
    covered: u64,
    /// The work done, in transitions tried and elements compared.
    work: u64,
}

impl<'a> Search<'a> {
    /// A search of the program with nothing kept yet.
    fn new(program: &'a Program, account: &'a Account) -> Self {
        let width = program.places;
        let mut starts = vec![0u32; width + 1];
        for transition in &program.transitions {
            for &(p, _) in &program.arcs[transition.outputs as usize..transition.end as usize] {
                starts[p as usize + 1] += 1;
            }
        }
        for p in 0..width {
            starts[p + 1] += starts[p];
        }
        let mut next = starts.clone();
        let mut by_output = vec![0; starts[width] as usize];
        for (t, transition) in program.transitions.iter().enumerate() {
            for &(p, _) in &program.arcs[transition.outputs as usize..transition.end as usize] {
                by_output[next[p as usize] as usize] = t as u32;
                next[p as usize] += 1;
            }
        }
        Self {
            program,
            entries: Vec::new(),
            ends: Vec::new(),
            sums: Vec::new(),
            parents: Vec::new(),
            by_first: vec![Vec::new(); width],
            queue: BinaryHeap::new(),
            starts,
            by_output,
            tried: vec![0; program.transitions.len()],
            probe: vec![0; width],
            next: Vec::new(),
            charged: Charged::new(account),
            computed: 0,
            covered: 0,
            work: 0,
        }
    }

    /// The search itself.
    fn run(
        &mut self,
        most: usize,
        equation: &mut Equation<'_>,
        stop: &mut dyn FnMut() -> bool,
    ) -> Result<Option<Vec<u32>>, Reason> {
        let program = self.program;
        self.next.clear();
        for (p, &count) in program.target.iter().enumerate() {
            if count > 0 {
                self.next.push((p as u32, count));
            }
        }
        if self.below_initial() {
            return Ok(Some(Vec::new()));
        }
        self.keep(ROOT, 0, most)?;
        while let Some(Reverse((_, e))) = self.queue.pop() {
            if stop() {
                return Err(Reason::Stopped);
            }
            let budget = self.work.saturating_mul(ENTRIES_PER_UNIT);
            if equation.wants(budget) && equation.run(program, budget, stop)? {
                return Ok(None);
            }
            let (start, end) = self.span(e);
            // An element kept after this one and at most it covers what
            // this one would.
            write(&mut self.probe, &self.entries[start..end], false);
            let (superseded, compared) = self.dominated(&self.entries[start..end], e);
            write(&mut self.probe, &self.entries[start..end], true);
            self.work += compared;
            if superseded {
                continue;
            }
            for i in start..end {
                let p = self.entries[i].0 as usize;
                for k in self.starts[p]..self.starts[p + 1] {
                    let t = self.by_output[k as usize];
                    if self.tried[t as usize] == e + 1 {
                        continue;
                    }
                    self.tried[t as usize] = e + 1;
                    if stop() {
                        return Err(Reason::Stopped);
                    }
                    self.computed += 1;
                    self.before(e, t)?;
                    if self.below_initial() {
                        let mut firings = vec![t];
                        firings.extend(self.path(e));
                        return Ok(Some(firings));
                    }
                    write(&mut self.probe, &self.next, false);
                    let (dominated, compared) = self.dominated(&self.next, u32::MAX);
                    write(&mut self.probe, &self.next, true);
                    self.work += 1 + compared;
                    if dominated {
                        self.covered += 1;
                    } else {
                        self.keep(e, t, most)?;
                    }
                }
            }
        }
        Ok(None)
    }

    /// The span of an element's entries.
    fn span(&self, e: u32) -> (usize, usize) {
        let e = e as usize;
        (if e == 0 { 0 } else { self.ends[e - 1] }, self.ends[e])
    }

    /// Computes into `next` the least marking from which transition `t`
    /// leads to one at least element `e`: the element less the outputs,
    /// but not below zero, plus the inputs. A count past `u32::MAX` is
    /// [`Reason::IndexLimit`].
    fn before(&mut self, e: u32, t: u32) -> Result<(), Reason> {
        let program = self.program;
        let transition = &program.transitions[t as usize];
        let outputs = &program.arcs[transition.outputs as usize..transition.end as usize];
        let inputs = &program.arcs[transition.inputs as usize..transition.outputs as usize];
        let (start, end) = self.span(e);
        let element = &self.entries[start..end];
        self.next.clear();
        // The element, the outputs and the inputs are each sorted by
        // place: one merge.
        let (mut i, mut j, mut o) = (0, 0, 0);
        while i < element.len() || j < inputs.len() {
            let p = match (element.get(i), inputs.get(j)) {
                (Some(&(a, _)), Some(&(b, _))) => a.min(b),
                (Some(&(a, _)), None) => a,
                (None, Some(&(b, _))) => b,
                (None, None) => unreachable!("the loop's condition"),
            };
            let mut count = 0;
            if let Some(&(a, c)) = element.get(i)
                && a == p
            {
                i += 1;
                while o < outputs.len() && outputs[o].0 < p {
                    o += 1;
                }
                let given = match outputs.get(o) {
                    Some(&(q, w)) if q == p => w,
                    _ => 0,
                };
                count = c.saturating_sub(given);
            }
            if let Some(&(b, w)) = inputs.get(j)
                && b == p
            {
                j += 1;
                count = count.checked_add(w).ok_or(Reason::IndexLimit)?;
            }
            if count > 0 {
                self.next.push((p, count));
            }
        }
        Ok(())
    }

    /// Whether the marking in `next` is at most the initial marking.
    fn below_initial(&self) -> bool {
        self.next
            .iter()
            .all(|&(p, c)| c <= self.program.initial[p as usize])
    }

    /// Whether an element other than `except` is at most `marking`, which
    /// the probe holds, and the elements compared: only an element whose
    /// first place the marking marks can be, and only one of no more
    /// tokens.
    fn dominated(&self, marking: &[(u32, u32)], except: u32) -> (bool, u64) {
        let tokens: u64 = marking.iter().map(|&(_, c)| u64::from(c)).sum();
        let mut compared = 0;
        for &(p, _) in marking {
            for &b in &self.by_first[p as usize] {
                if b == except || self.sums[b as usize] > tokens {
                    continue;
                }
                compared += 1;
                let (start, end) = self.span(b);
                if self.entries[start..end]
                    .iter()
                    .all(|&(q, c)| self.probe[q as usize] >= c)
                {
                    return (true, compared);
                }
            }
        }
        (false, compared)
    }

    /// Keeps the marking in `next` as an element, computed from `parent`
    /// by `transition`; or says why it cannot be kept.
    fn keep(&mut self, parent: u32, transition: u32, most: usize) -> Result<(), Reason> {
        let index = self.parents.len();
        if index >= most {
            return Err(Reason::IndexLimit);
        }
        let first = self.next.first().map_or(0, |&(p, _)| p as usize);
        let more = self.next.len();
        if !room(&mut self.entries, more, &mut self.charged)
            || !room(&mut self.ends, 1, &mut self.charged)
            || !room(&mut self.sums, 1, &mut self.charged)
            || !room(&mut self.parents, 1, &mut self.charged)
            || !room(&mut self.by_first[first], 1, &mut self.charged)
            || !self.room_in_queue()
        {
            return Err(Reason::MemoryLimit(self.charged.account().limit()));
        }
        self.entries.extend_from_slice(&self.next);
        self.ends.push(self.entries.len());
        let tokens = self.next.iter().map(|&(_, c)| u64::from(c)).sum();
        self.sums.push(tokens);
        self.parents
            .push(u64::from(parent) << 32 | u64::from(transition));
        let index = index as u32;
        self.by_first[first].push(index);
        self.queue.push(Reverse((tokens, index)));
        Ok(())
    }

    /// Makes room in the queue for one more element, doubling it, and
    /// charges what it grew by; returns false when the bound has no room.
    fn room_in_queue(&mut self) -> bool {
        let size = size_of::<Reverse<(u64, u32)>>();
        if self.queue.len() < self.queue.capacity() {
            return true;
        }
        let more = self.queue.capacity().max(16);
        let fits = more
            .checked_mul(size)
            .is_some_and(|bytes| self.charged.account().fits(bytes));
        if !fits {
            return false;
        }
        let before = self.queue.capacity() * size;
        self.queue.reserve_exact(more);
        self.charged.resize(before, self.queue.capacity() * size);
        true
    }

    /// The transitions fired from a marking at least element `e` to one
    /// that covers the target: `e`'s transition, then its parent's, up to
    /// the target.
    fn path(&self, mut e: u32) -> Vec<u32> {
        let mut firings = Vec::new();
        loop {
            let parent = self.parents[e as usize];
            if (parent >> 32) as u32 == ROOT {
                return firings;
            }
            firings.push(parent as u32);
            e = (parent >> 32) as u32;
        }
    }
}

/// Writes a marking's counts into a dense vector of counts, or clears
/// them.
fn write(dense: &mut [u32], marking: &[(u32, u32)], clear: bool) {
    for &(p, c) in marking {
        dense[p as usize] = if clear { 0 } else { c };
    }
}
