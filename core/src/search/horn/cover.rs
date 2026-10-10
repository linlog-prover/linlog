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
//! (that one, which stays in the index, would have kept it out), and an
//! infinite sequence of count vectors in
//! which no element is at least an earlier one does not exist (Dickson's
//! lemma). How long it takes, Dickson does not bound: the memory bound
//! and the caller's stop do.

use super::Program;
use super::equation::Equation;
use super::reach::room;
use crate::hash::HashMap;
use crate::search::memory::{Account, Charged};
use crate::search::{Reason, Statistics};
use std::cell::RefCell;
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
        memo_entries: search.parents.len() as u64,
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
    /// The index of the elements by their places, a trie: the child of a
    /// node by a place, the root being node zero and a path following
    /// places in increasing order. An element is listed at the node its
    /// places lead to, so the elements at most a marking are listed at
    /// nodes that some of the marking's places lead to.
    edges: HashMap<(u32, u32), u32>,
    /// The first element listed at each node, plus one; zero for none.
    listed: Vec<u32>,
    /// The first child of each node, plus one; zero for none.
    first_child: Vec<u32>,
    /// The next child of each node's parent after it, plus one.
    sibling: Vec<u32>,
    /// The place that leads to each node from its parent.
    place_of_node: Vec<u32>,
    /// The children of each node.
    children: Vec<u32>,
    /// The element listed after each at its node, plus one.
    after: Vec<u32>,
    /// The nodes a query has still to visit, with where in the marking's
    /// places their children start.
    stack: RefCell<Vec<(u32, u32)>>,
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
    /// The index of each place among the probe's marked places, plus one;
    /// zero for a place it does not mark.
    position: Vec<u32>,
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
            edges: HashMap::default(),
            listed: vec![0],
            first_child: vec![0],
            sibling: vec![0],
            place_of_node: vec![0],
            children: vec![0],
            after: Vec::new(),
            stack: RefCell::new(Vec::new()),
            queue: BinaryHeap::new(),
            starts,
            by_output,
            tried: vec![0; program.transitions.len()],
            probe: vec![0; width],
            position: vec![0; width],
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
            let budget = equation.budget(self.work);
            if equation.wants(budget) && equation.run(program, budget, stop)? {
                return Ok(None);
            }
            let (start, end) = self.span(e);
            // An element kept after this one and at most it covers what
            // this one would.
            load(
                &mut self.probe,
                &mut self.position,
                &self.entries[start..end],
                false,
            );
            let (superseded, compared) = self.dominated(&self.entries[start..end], e);
            load(
                &mut self.probe,
                &mut self.position,
                &self.entries[start..end],
                true,
            );
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
                    load(&mut self.probe, &mut self.position, &self.next, false);
                    let (dominated, compared) = self.dominated(&self.next, u32::MAX);
                    load(&mut self.probe, &mut self.position, &self.next, true);
                    self.work += 1 + compared;
                    if dominated {
                        self.covered += 1;
                    } else {
                        // Out of room, the search takes back the simplex's
                        // memory and tries once more.
                        match self.keep(e, t, most) {
                            Err(Reason::MemoryLimit { .. }) if equation.release() => {
                                self.keep(e, t, most)?;
                            }
                            kept => kept?,
                        }
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
    /// the probe holds, and the nodes and elements looked at: the index's
    /// nodes that the marking's places lead to, and of the elements listed
    /// there, whose places the marking all marks, those of no more tokens.
    fn dominated(&self, marking: &[(u32, u32)], except: u32) -> (bool, u64) {
        let tokens: u64 = marking.iter().map(|&(_, c)| u64::from(c)).sum();
        let mut looked = 0;
        let mut stack = self.stack.borrow_mut();
        stack.clear();
        stack.push((0, 0));
        while let Some((node, from)) = stack.pop() {
            looked += 1;
            let mut listed = self.listed[node as usize];
            while listed != 0 {
                let b = listed - 1;
                if b != except && self.sums[b as usize] <= tokens {
                    looked += 1;
                    let (start, end) = self.span(b);
                    if self.entries[start..end]
                        .iter()
                        .all(|&(q, c)| self.probe[q as usize] >= c)
                    {
                        return (true, looked);
                    }
                }
                listed = self.after[b as usize];
            }
            // The node's children that the rest of the marking's places
            // lead to: from the side with fewer, the children or the
            // places, each a step that costs about a comparison.
            let rest = marking.len() - from as usize;
            let children = self.children[node as usize] as usize;
            if children <= rest {
                looked += children as u64;
                let mut next = self.first_child[node as usize];
                while next != 0 {
                    let child = next - 1;
                    let at = self.position[self.place_of_node[child as usize] as usize];
                    if at > from {
                        stack.push((child, at));
                    }
                    next = self.sibling[child as usize];
                }
            } else {
                looked += rest as u64;
                for (i, &(p, _)) in marking.iter().enumerate().skip(from as usize) {
                    if let Some(&child) = self.edges.get(&(node, p)) {
                        stack.push((child, i as u32 + 1));
                    }
                }
            }
        }
        (false, looked)
    }

    /// Keeps the marking in `next` as an element, computed from `parent`
    /// by `transition`; or says why it cannot be kept.
    fn keep(&mut self, parent: u32, transition: u32, most: usize) -> Result<(), Reason> {
        let index = self.parents.len();
        if index >= most {
            return Err(Reason::IndexLimit);
        }
        let more = self.next.len();
        // A node per place at most: the nodes are `u32`.
        if self.listed.len() + more >= u32::MAX as usize {
            return Err(Reason::IndexLimit);
        }
        if !room(&mut self.entries, more, &mut self.charged)
            || !room(&mut self.ends, 1, &mut self.charged)
            || !room(&mut self.sums, 1, &mut self.charged)
            || !room(&mut self.parents, 1, &mut self.charged)
            || !room(&mut self.listed, more, &mut self.charged)
            || !room(&mut self.first_child, more, &mut self.charged)
            || !room(&mut self.sibling, more, &mut self.charged)
            || !room(&mut self.place_of_node, more, &mut self.charged)
            || !room(&mut self.children, more, &mut self.charged)
            || !room(&mut self.after, 1, &mut self.charged)
            || !self.room_in_edges(more)
            || !self.room_in_queue()
        {
            return Err(Reason::MemoryLimit {
                limit_bytes: self.charged.account().limit(),
            });
        }
        self.entries.extend_from_slice(&self.next);
        self.ends.push(self.entries.len());
        let tokens = self.next.iter().map(|&(_, c)| u64::from(c)).sum();
        self.sums.push(tokens);
        self.parents
            .push(u64::from(parent) << 32 | u64::from(transition));
        let index = index as u32;
        let mut node = 0;
        for &(p, _) in &self.next {
            node = match self.edges.get(&(node, p)) {
                Some(&child) => child,
                None => {
                    let child = self.listed.len() as u32;
                    self.edges.insert((node, p), child);
                    self.listed.push(0);
                    self.first_child.push(0);
                    self.sibling.push(self.first_child[node as usize]);
                    self.first_child[node as usize] = child + 1;
                    self.place_of_node.push(p);
                    self.children.push(0);
                    self.children[node as usize] += 1;
                    child
                }
            };
        }
        self.after.push(self.listed[node as usize]);
        self.listed[node as usize] = index + 1;
        self.queue.push(Reverse((tokens, index)));
        Ok(())
    }

    /// Makes room in the index's edges for `more`, and charges what the
    /// table grew by (its slots and a control byte each); returns false
    /// when the bound has no room.
    fn room_in_edges(&mut self, more: usize) -> bool {
        let slot = size_of::<((u32, u32), u32)>() + 1;
        if self.edges.capacity() - self.edges.len() >= more {
            return true;
        }
        let before = self.edges.capacity() * slot;
        let wanted = self
            .edges
            .len()
            .checked_add(more)
            .map(|needed| needed.max(2 * self.edges.capacity()));
        let fits = wanted
            .and_then(|wanted| wanted.checked_mul(2 * slot))
            .is_some_and(|bytes| self.charged.account().fits(bytes.saturating_sub(before)));
        let (true, Some(wanted)) = (fits, wanted) else {
            return false;
        };
        self.edges.reserve(wanted - self.edges.len());
        self.charged.resize(before, self.edges.capacity() * slot);
        true
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

/// Writes a marking's counts into a dense vector of counts and each
/// place's index among its marked places, plus one, into another, or
/// clears both.
fn load(counts: &mut [u32], position: &mut [u32], marking: &[(u32, u32)], clear: bool) {
    for (i, &(p, c)) in marking.iter().enumerate() {
        (counts[p as usize], position[p as usize]) = if clear { (0, 0) } else { (c, i as u32 + 1) };
    }
}
