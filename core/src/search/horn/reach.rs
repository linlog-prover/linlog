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
//!
//! Once the forward search has done some work without deciding, a second
//! search runs beside it, a share of the work at a time: the same search
//! on the reversed net, from the target toward the initial marking, with
//! the places that no transition raises capped at their initial count
//! (no marking a firing sequence from the initial one passes has more).
//! Its finding the initial marking is a firing sequence read backward,
//! and its exhausting its markings refutes: nets whose markings grow
//! without end forward often have few backward.

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

/// The work the forward search does alone before the backward search
/// starts beside it: a net decided within it is searched exactly as
/// without the backward search, which most nets of practice are.
const BACKWARD_AFTER: u64 = 1 << 16;

/// The backward search's share: it may do one unit of work for every
/// this many of the forward search's.
const BACKWARD_SHARE: u64 = 4;

/// Searches the program's markings from the initial one for the target,
/// polling `stop` at every successor taken from the frontier, keeping at
/// most `most` markings in each direction and giving `equation` and the
/// backward search their shares of the work, and returns the transitions
/// of a firing sequence that reaches it, `None` when the markings of
/// either direction are exhausted or the state equation refutes, or the
/// reason it stopped; and the counters of both directions: the initial
/// marking and the successors taken from the frontier as `nodes`, those
/// of them kept already or above a cap as `memo_hits`, the markings kept,
/// each expanded once, as `memo_entries`.
pub(super) fn search(
    program: &Program,
    account: &Account,
    most: usize,
    equation: &mut Equation<'_>,
    stop: &mut dyn FnMut() -> bool,
) -> (Result<Option<Vec<u32>>, Reason>, Statistics) {
    let reversed = program.reversed();
    let mut forward = Search::new(program, account, Vec::new());
    let mut backward = Search::new(&reversed, account, program.caps());
    let result = both(&mut forward, &mut backward, most, equation, stop);
    let statistics = Statistics {
        nodes: forward.nodes() + backward.nodes(),
        memo_hits: forward.repeated + backward.repeated,
        memo_entries: forward.kept + backward.kept,
        ..Statistics::default()
    };
    (result, statistics)
}

/// The forward search with the state equation and the backward search
/// beside it, as [`search`] describes.
fn both(
    forward: &mut Search<'_>,
    backward: &mut Search<'_>,
    most: usize,
    equation: &mut Equation<'_>,
    stop: &mut dyn FnMut() -> bool,
) -> Result<Option<Vec<u32>>, Reason> {
    let program = forward.program;
    // Out of room, the forward search takes back the simplex's memory,
    // then the backward search's, and tries once more.
    if let Some(firings) =
        forward.start(most, &mut || equation.release() || backward.give_back())?
    {
        return Ok(Some(firings));
    }
    loop {
        if stop() {
            return Err(Reason::Stopped);
        }
        let work = forward.work;
        let budget = work.saturating_mul(ENTRIES_PER_UNIT);
        if equation.wants(budget) && equation.run(program, budget, stop)? {
            return Ok(None);
        }
        if work >= BACKWARD_AFTER && backward.alive {
            match backward.advance(work / BACKWARD_SHARE, most, stop) {
                Ok(Slice::Found(mut firings)) => {
                    firings.reverse();
                    return Ok(Some(firings));
                }
                Ok(Slice::Exhausted) => return Ok(None),
                Ok(Slice::Paused) => {}
                Err(Reason::MemoryLimit(_) | Reason::IndexLimit) => {
                    backward.give_back();
                }
                Err(reason) => return Err(reason),
            }
        }
        match forward.step(most, &mut || equation.release() || backward.give_back())? {
            Slice::Paused => {}
            Slice::Found(firings) => return Ok(Some(firings)),
            Slice::Exhausted => return Ok(None),
        }
    }
}

/// What a step or a slice of a search came to.
enum Slice {
    /// The target was reached, by these firings from the initial marking.
    Found(Vec<u32>),
    /// The frontier is empty: every reachable marking was expanded.
    Exhausted,
    /// Neither yet.
    Paused,
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
    /// The successors taken from the frontier that were kept already or
    /// above a cap.
    repeated: u64,
    /// The markings kept.
    kept: usize,
    /// The work done, in transitions examined and markings written.
    work: u64,
    /// The most tokens a marking may hold on each place, the marking
    /// above it dropped; empty for no caps.
    caps: Vec<u32>,
    /// Whether the search has started.
    started: bool,
    /// Whether the search still holds its markings, not having given its
    /// memory back.
    alive: bool,
}

impl<'a> Search<'a> {
    /// A search of the program with nothing kept yet, with caps per place
    /// or none.
    fn new(program: &'a Program, account: &'a Account, caps: Vec<u32>) -> Self {
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
            kept: 0,
            work: 0,
            caps,
            started: false,
            alive: true,
        }
    }

    /// The markings taken: the kept and the dropped.
    fn nodes(&self) -> u64 {
        self.kept as u64 + self.repeated
    }

    /// Gives back the memory the markings hold, for a search beside this
    /// one that has no room left: returns whether there was any to give.
    /// This search then ends without deciding.
    fn give_back(&mut self) -> bool {
        if !self.alive {
            return false;
        }
        self.alive = false;
        self.bytes = Vec::new();
        self.ends = Vec::new();
        self.hashes = Vec::new();
        self.parents = Vec::new();
        self.slots = Vec::new();
        self.frontier = BinaryHeap::new();
        self.charged = Charged::new(self.charged.account());
        true
    }

    /// Starts at the initial marking: returns the empty firing sequence
    /// if it is the target, the firing that reaches the target from it if
    /// one does, and else nothing, its successors on the frontier.
    fn start(
        &mut self,
        most: usize,
        release: &mut dyn FnMut() -> bool,
    ) -> Result<Option<Vec<u32>>, Reason> {
        self.started = true;
        for (p, &count) in self.program.initial.iter().enumerate() {
            if count > 0 {
                self.counts[p] = count;
                self.marked.push(p as u32);
            }
        }
        if self.distance() == 0 {
            return Ok(Some(Vec::new()));
        }
        self.take(ROOT, 0, most, release)
    }

    /// Takes the next successor from the frontier.
    fn step(&mut self, most: usize, release: &mut dyn FnMut() -> bool) -> Result<Slice, Reason> {
        let Some(Reverse((key, t))) = self.frontier.pop() else {
            return Ok(Slice::Exhausted);
        };
        let parent = u32::MAX - key as u32;
        self.successor(parent, t)?;
        Ok(match self.take(parent, t, most, release)? {
            Some(firings) => Slice::Found(firings),
            None => Slice::Paused,
        })
    }

    /// Runs until the work reaches `until`, polling `stop` at every
    /// successor; out of room, it ends.
    fn advance(
        &mut self,
        until: u64,
        most: usize,
        stop: &mut dyn FnMut() -> bool,
    ) -> Result<Slice, Reason> {
        if !self.started
            && let Some(firings) = self.start(most, &mut || false)?
        {
            return Ok(Slice::Found(firings));
        }
        while self.work < until {
            if stop() {
                return Err(Reason::Stopped);
            }
            match self.step(most, &mut || false)? {
                Slice::Paused => {}
                decided => return Ok(decided),
            }
        }
        Ok(Slice::Paused)
    }

    /// Takes the marking at hand, reached from `parent` by `transition`:
    /// drops it if it is kept already or above a cap, and else keeps and
    /// expands it; returns the firing sequence when one of its successors
    /// is the target. Out of room, it calls `release` and tries once more
    /// if that gave memory back.
    fn take(
        &mut self,
        parent: u32,
        transition: u32,
        most: usize,
        release: &mut dyn FnMut() -> bool,
    ) -> Result<Option<Vec<u32>>, Reason> {
        self.write();
        let capped = !self.caps.is_empty()
            && self
                .marked
                .iter()
                .any(|&p| self.counts[p as usize] > self.caps[p as usize]);
        if capped || self.find().is_some() {
            self.repeated += 1;
        } else {
            let index = match self.keep(parent, transition, most) {
                Err(Reason::MemoryLimit(_)) if release() => self.keep(parent, transition, most)?,
                kept => kept?,
            };
            let found = match self.expand(index) {
                Err(Reason::MemoryLimit(_)) if release() => self.expand(index)?,
                expanded => expanded?,
            };
            if let Some(t) = found {
                let mut firings = self.path(index);
                firings.push(t);
                return Ok(Some(firings));
            }
        }
        self.clear();
        Ok(None)
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
        self.kept += 1;
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
