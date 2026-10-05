// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Reachability by explicit states: every marking expanded is kept once,
//! as its count vector written sparsely in a few bytes, and the frontier
//! holds the successors not yet expanded as the marking and the
//! transition they come from, nearest to the target first. A successor
//! is written out only when it is expanded, and dropped then if it was
//! kept before. A finite set of reachable markings is exhausted, which
//! refutes the goal; an infinite one runs until the stop or the memory
//! bound.

use super::Program;
use super::equation::{ENTRIES_PER_UNIT, Equation};
use crate::search::memory::{Account, Charged, bytes_of};
use crate::search::{Reason, Statistics};
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::hash::BuildHasher;

/// The most markings the search keeps: their indices are `u32`, and the
/// table stores an index plus one. Reaching it is
/// [`Reason::IndexLimit`]; with a memory bound the bound comes first,
/// since a marking kept takes more than thirty bytes.
pub(super) const MOST_MARKINGS: usize = u32::MAX as usize - 1;

/// The parent of the initial marking.
const ROOT: u32 = u32::MAX;

/// Searches the program's markings from the initial one for the target,
/// polling `stop` at every successor taken from the frontier, keeping at
/// most `most` markings and giving `equation` its share of the work, and
/// returns the transitions of a firing sequence that reaches it, `None`
/// when the reachable markings are exhausted or the state equation
/// refutes, or the reason it stopped; and the counters: the initial
/// marking and the successors taken from the frontier as `nodes`, those
/// of them kept already as `memo_hits`, the markings kept, each expanded
/// once, as `memo_entries`.
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
        nodes: search.parents.len() as u64 + search.repeated,
        memo_hits: search.repeated,
        memo_entries: search.parents.len(),
        ..Statistics::default()
    };
    (result, statistics)
}

/// An entry of the frontier: the distance of the successor to the
/// target above and its parent's index complemented below, so that the
/// nearest comes first and, among equals, a successor of the latest
/// marking expanded; and the transition that leads to it.
type Entry = Reverse<(u64, u32)>;

/// The state of a search: the markings kept, the frontier, the index of
/// the transitions by their first input place, the marking at hand, and
/// the counters.
struct Search<'a> {
    /// The program.
    program: &'a Program,
    /// The tokens of the target, the distance to it of the empty
    /// marking: at most the forest's occurrences.
    target_total: u64,
    /// The markings kept, one after the other, each as its marked places
    /// in increasing order, a place as its distance from the one before
    /// less one and then its count less one, both in LEB128: a function
    /// of the marking, so two are equal exactly when their bytes are.
    bytes: Vec<u8>,
    /// Where each marking's bytes end.
    ends: Vec<u64>,
    /// The hash of each marking's bytes.
    hashes: Vec<u64>,
    /// The marking each was reached from and the transition fired, as
    /// `parent << 32 | transition`; [`ROOT`] above for the initial one.
    parents: Vec<u64>,
    /// The table of markings: a power of two of slots, each zero or a
    /// marking's index plus one, at most half of them used.
    slots: Vec<u32>,
    /// The successors to expand.
    frontier: BinaryHeap<Entry>,
    /// The transitions whose first input is each place: those of place
    /// `p` are `by_place[starts[p]..starts[p + 1]]`.
    starts: Vec<u32>,
    /// The transitions by their first input place.
    by_place: Vec<u32>,
    /// The transitions without inputs, always enabled.
    free: Vec<u32>,
    /// The marking at hand, as a count per place: zero but on `marked`.
    counts: Vec<u32>,
    /// The places the marking at hand marks, in increasing order.
    marked: Vec<u32>,
    /// A buffer for the places of a successor.
    next: Vec<u32>,
    /// A buffer for a marking's bytes.
    written: Vec<u8>,
    /// What the buffers that grow hold.
    charged: Charged<'a>,
    /// The successors taken from the frontier that were kept already.
    repeated: u64,
    /// The work done, in transitions examined and markings written.
    work: u64,
}

impl<'a> Search<'a> {
    /// A search of the program with nothing kept yet.
    fn new(program: &'a Program, account: &'a Account) -> Self {
        let width = program.places;
        let mut starts = vec![0u32; width + 1];
        let mut free = Vec::new();
        for (t, transition) in program.transitions.iter().enumerate() {
            if transition.inputs == transition.outputs {
                free.push(t as u32);
            } else {
                starts[program.arcs[transition.inputs as usize].0 as usize + 1] += 1;
            }
        }
        for p in 0..width {
            starts[p + 1] += starts[p];
        }
        let mut next = starts.clone();
        let mut by_place = vec![0; starts[width] as usize];
        for (t, transition) in program.transitions.iter().enumerate() {
            if transition.inputs != transition.outputs {
                let p = program.arcs[transition.inputs as usize].0 as usize;
                by_place[next[p] as usize] = t as u32;
                next[p] += 1;
            }
        }
        Self {
            program,
            target_total: program.target.iter().map(|&t| u64::from(t)).sum(),
            bytes: Vec::new(),
            ends: Vec::new(),
            hashes: Vec::new(),
            parents: Vec::new(),
            slots: Vec::new(),
            frontier: BinaryHeap::new(),
            starts,
            by_place,
            free,
            counts: vec![0; width],
            marked: Vec::new(),
            next: Vec::new(),
            written: Vec::new(),
            charged: Charged::new(account),
            repeated: 0,
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
        for (p, &count) in program.initial.iter().enumerate() {
            if count > 0 {
                self.counts[p] = count;
                self.marked.push(p as u32);
            }
        }
        if self.distance() == 0 {
            return Ok(Some(Vec::new()));
        }
        let mut parent = ROOT;
        let mut transition = 0;
        loop {
            self.write();
            if self.find().is_some() {
                self.repeated += 1;
            } else {
                // Out of room, the search takes back the simplex's memory
                // and tries once more.
                let index = match self.keep(parent, transition, most) {
                    Err(Reason::MemoryLimit(_)) if equation.release() => {
                        self.keep(parent, transition, most)?
                    }
                    kept => kept?,
                };
                let found = match self.expand(index) {
                    Err(Reason::MemoryLimit(_)) if equation.release() => self.expand(index)?,
                    expanded => expanded?,
                };
                if let Some(t) = found {
                    let mut firings = self.path(index);
                    firings.push(t);
                    return Ok(Some(firings));
                }
            }
            self.clear();
            let Some(Reverse((key, t))) = self.frontier.pop() else {
                return Ok(None);
            };
            if stop() {
                return Err(Reason::Stopped);
            }
            let budget = self.work.saturating_mul(ENTRIES_PER_UNIT);
            if equation.wants(budget) && equation.run(program, budget, stop)? {
                return Ok(None);
            }
            (parent, transition) = (u32::MAX - key as u32, t);
            self.successor(parent, transition)?;
        }
    }

    /// The distance of the marking at hand to the target: the tokens by
    /// which the two differ.
    fn distance(&self) -> u64 {
        let target = &self.program.target;
        let unmarked: u64 = self.target_total
            - self
                .marked
                .iter()
                .map(|&p| u64::from(target[p as usize]))
                .sum::<u64>();
        unmarked
            + self
                .marked
                .iter()
                .map(|&p| u64::from(self.counts[p as usize].abs_diff(target[p as usize])))
                .sum::<u64>()
    }

    /// Puts every successor of the marking at hand, kept as `index`, on
    /// the frontier; returns a transition that reaches the target from
    /// it, if one does, or why the frontier has no room.
    fn expand(&mut self, index: u32) -> Result<Option<u32>, Reason> {
        let program = self.program;
        let distance = self.distance();
        let marked = std::mem::take(&mut self.marked);
        let candidates = self
            .free
            .iter()
            .copied()
            .chain(marked.iter().flat_map(|&p| {
                self.by_place
                    [self.starts[p as usize] as usize..self.starts[p as usize + 1] as usize]
                    .iter()
                    .copied()
            }));
        let counts = &self.counts;
        let mut successors = Vec::new();
        let mut examined = 0;
        for t in candidates {
            examined += 1;
            let transition = &program.transitions[t as usize];
            let inputs = &program.arcs[transition.inputs as usize..transition.outputs as usize];
            if inputs.iter().any(|&(p, w)| counts[p as usize] < w) {
                continue;
            }
            let outputs = &program.arcs[transition.outputs as usize..transition.end as usize];
            let d = moved(distance, inputs, outputs, counts, &program.target);
            if d == 0 {
                self.marked = marked;
                return Ok(Some(t));
            }
            // A distance of 2³² or more ranks with the farthest.
            let rank = d.min(u64::from(u32::MAX));
            successors.push(Reverse((rank << 32 | u64::from(u32::MAX - index), t)));
        }
        self.work += examined;
        self.marked = marked;
        let before = self.frontier.capacity() * size_of::<Entry>();
        if self.frontier.capacity() - self.frontier.len() < successors.len() {
            let more = successors.len().max(self.frontier.capacity());
            let fits = more
                .checked_mul(size_of::<Entry>())
                .is_some_and(|bytes| self.charged.account().fits(bytes));
            if !fits {
                return Err(Reason::MemoryLimit(self.charged.account().limit()));
            }
            self.frontier.reserve_exact(more);
        }
        self.charged
            .resize(before, self.frontier.capacity() * size_of::<Entry>());
        self.frontier.extend(successors);
        Ok(None)
    }

    /// Makes the marking kept as `parent` with `transition` fired the
    /// marking at hand, or says that a count would pass `u32::MAX`.
    fn successor(&mut self, parent: u32, transition: u32) -> Result<(), Reason> {
        let start = match parent {
            0 => 0,
            _ => self.ends[parent as usize - 1] as usize,
        };
        let bytes = &self.bytes[start..self.ends[parent as usize] as usize];
        let (mut at, mut place) = (0, 0u32);
        while at < bytes.len() {
            place += get(bytes, &mut at);
            self.counts[place as usize] = get(bytes, &mut at) + 1;
            self.marked.push(place);
            place += 1;
        }
        let program = self.program;
        let transition = &program.transitions[transition as usize];
        for &(p, w) in &program.arcs[transition.inputs as usize..transition.outputs as usize] {
            self.counts[p as usize] -= w;
        }
        // The places marked before and those the transition marks, in
        // order, without those it empties.
        self.next.clear();
        let outputs = &program.arcs[transition.outputs as usize..transition.end as usize];
        let (mut i, mut o) = (0, 0);
        while i < self.marked.len() || o < outputs.len() {
            let p = match (self.marked.get(i), outputs.get(o)) {
                (Some(&a), Some(&(b, _))) if a == b => {
                    (i, o) = (i + 1, o + 1);
                    a
                }
                (Some(&a), Some(&(b, _))) if a < b => {
                    i += 1;
                    a
                }
                (Some(&a), None) => {
                    i += 1;
                    a
                }
                (_, Some(&(b, _))) => {
                    o += 1;
                    b
                }
                (None, None) => unreachable!("the loop's condition"),
            };
            self.next.push(p);
        }
        for &(p, w) in outputs {
            let Some(count) = self.counts[p as usize].checked_add(w) else {
                return Err(Reason::IndexLimit);
            };
            self.counts[p as usize] = count;
        }
        let counts = &self.counts;
        self.next.retain(|&p| counts[p as usize] > 0);
        std::mem::swap(&mut self.marked, &mut self.next);
        Ok(())
    }

    /// Writes the marking at hand into `written`.
    fn write(&mut self) {
        self.work += 1;
        self.written.clear();
        let mut from = 0;
        for &p in &self.marked {
            put(&mut self.written, p - from);
            put(&mut self.written, self.counts[p as usize] - 1);
            from = p + 1;
        }
    }

    /// Clears the marking at hand.
    fn clear(&mut self) {
        for &p in &self.marked {
            self.counts[p as usize] = 0;
        }
        self.marked.clear();
    }

    /// The hash of the marking written.
    fn hash(&self) -> u64 {
        crate::hash::BuildHasher::default().hash_one(&self.written)
    }

    /// The index of the marking written, if it is kept.
    fn find(&self) -> Option<u32> {
        if self.slots.is_empty() {
            return None;
        }
        let hash = self.hash();
        let mask = self.slots.len() - 1;
        let mut slot = hash as usize & mask;
        loop {
            let e = self.slots[slot].checked_sub(1)? as usize;
            let start = if e == 0 { 0 } else { self.ends[e - 1] as usize };
            if self.hashes[e] == hash && self.bytes[start..self.ends[e] as usize] == *self.written {
                return Some(e as u32);
            }
            slot = (slot + 1) & mask;
        }
    }

    /// Keeps the marking written, reached from `parent` by `transition`;
    /// returns its index, or why it cannot be kept.
    fn keep(&mut self, parent: u32, transition: u32, most: usize) -> Result<u32, Reason> {
        let index = self.parents.len();
        if index >= most {
            return Err(Reason::IndexLimit);
        }
        let more = self.written.len();
        if (self.slots.len() / 2 <= index && !self.grow_table())
            || !room(&mut self.bytes, more, &mut self.charged)
            || !room(&mut self.ends, 1, &mut self.charged)
            || !room(&mut self.hashes, 1, &mut self.charged)
            || !room(&mut self.parents, 1, &mut self.charged)
        {
            return Err(Reason::MemoryLimit(self.charged.account().limit()));
        }
        let hash = self.hash();
        self.bytes.extend_from_slice(&self.written);
        self.ends.push(self.bytes.len() as u64);
        self.hashes.push(hash);
        self.parents
            .push(u64::from(parent) << 32 | u64::from(transition));
        let index = index as u32;
        self.place(index, hash);
        Ok(index)
    }

    /// Puts a marking's index into the table.
    fn place(&mut self, index: u32, hash: u64) {
        let mask = self.slots.len() - 1;
        let mut slot = hash as usize & mask;
        while self.slots[slot] != 0 {
            slot = (slot + 1) & mask;
        }
        self.slots[slot] = index + 1;
    }

    /// Doubles the table, or returns false when the bound has no room for
    /// it.
    fn grow_table(&mut self) -> bool {
        let Some(size) = self.slots.len().checked_mul(2).map(|size| size.max(16)) else {
            return false;
        };
        let before = bytes_of(&self.slots);
        let fits = size
            .checked_mul(size_of::<u32>())
            .is_some_and(|bytes| self.charged.account().fits(bytes.saturating_sub(before)));
        if !fits {
            return false;
        }
        self.slots = vec![0; size];
        self.charged.resize(before, bytes_of(&self.slots));
        for index in 0..self.hashes.len() {
            self.place(index as u32, self.hashes[index]);
        }
        true
    }

    /// The transitions fired from the initial marking to marking `j`.
    fn path(&self, mut j: u32) -> Vec<u32> {
        let mut firings = Vec::new();
        loop {
            let parent = self.parents[j as usize];
            if (parent >> 32) as u32 == ROOT {
                break;
            }
            firings.push(parent as u32);
            j = (parent >> 32) as u32;
        }
        firings.reverse();
        firings
    }
}

/// The distance to the target after a transition with these arcs fires
/// on `counts`, at `distance` before. Both lists of arcs are sorted, so
/// a place both touch is counted once.
fn moved(
    distance: u64,
    inputs: &[(u32, u32)],
    outputs: &[(u32, u32)],
    counts: &[u32],
    target: &[u32],
) -> u64 {
    let (mut i, mut o) = (0, 0);
    let mut distance = distance;
    while i < inputs.len() || o < outputs.len() {
        let (p, taken, given) = match (inputs.get(i), outputs.get(o)) {
            (Some(&(a, w)), Some(&(b, v))) if a == b => {
                (i, o) = (i + 1, o + 1);
                (a, w, v)
            }
            (Some(&(a, w)), Some(&(b, _))) if a < b => {
                i += 1;
                (a, w, 0)
            }
            (Some(&(a, w)), None) => {
                i += 1;
                (a, w, 0)
            }
            (_, Some(&(b, v))) => {
                o += 1;
                (b, 0, v)
            }
            (None, None) => unreachable!("the loop's condition"),
        };
        let (count, goal) = (u64::from(counts[p as usize]), u64::from(target[p as usize]));
        // In u64 a count past u32::MAX is still exact; the successor
        // refuses it when it is written out.
        let after = count - u64::from(taken) + u64::from(given);
        distance = distance - count.abs_diff(goal) + after.abs_diff(goal);
    }
    distance
}

/// Appends a number in LEB128: seven bits a byte, the lowest first, the
/// top bit set on every byte but the last.
fn put(bytes: &mut Vec<u8>, mut value: u32) {
    while value >= 0x80 {
        bytes.push(value as u8 | 0x80);
        value >>= 7;
    }
    bytes.push(value as u8);
}

/// Reads a number [`put`] wrote at `at`, and moves `at` past it.
fn get(bytes: &[u8], at: &mut usize) -> u32 {
    let (mut value, mut shift) = (0, 0);
    loop {
        let byte = bytes[*at];
        *at += 1;
        value |= u32::from(byte & 0x7f) << shift;
        if byte < 0x80 {
            return value;
        }
        shift += 7;
    }
}

/// Makes room in a buffer for `more` elements, doubling it, and charges
/// what it grew by; returns false when the bound has no room.
pub(super) fn room<T>(buffer: &mut Vec<T>, more: usize, charged: &mut Charged<'_>) -> bool {
    if buffer.capacity() - buffer.len() >= more {
        return true;
    }
    let before = bytes_of(buffer);
    // On a target of 32 bits these could pass `usize::MAX`: that is a
    // buffer the bound cannot have room for.
    let capacity = buffer
        .len()
        .checked_add(more)
        .zip(buffer.capacity().checked_mul(2))
        .map(|(needed, doubled)| needed.max(doubled).max(16));
    let fits = capacity
        .and_then(|capacity| capacity.checked_mul(size_of::<T>()))
        .is_some_and(|bytes| charged.account().fits(bytes.saturating_sub(before)));
    let (true, Some(capacity)) = (fits, capacity) else {
        return false;
    };
    buffer.reserve_exact(capacity - buffer.len());
    charged.resize(before, bytes_of(buffer));
    true
}
