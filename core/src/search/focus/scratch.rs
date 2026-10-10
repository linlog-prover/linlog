// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The engine's pools of scratch buffers: a step takes what it needs
//! from a pool and gives it back, so that no step allocates once the
//! pools are warm.

use super::Run;
use super::context::Context;
use super::counts::{Split, Tally};
use super::split::{Cursors, Frame};
use crate::occurrences::{OccId, OccSet};
use crate::proofs::{Branch, NodeId};
use crate::search::memory::{Charged, bytes_of};

impl Run<'_> {
    /// Takes an empty set from the pool.
    pub(super) fn take_set(&mut self) -> OccSet {
        match self.pools.sets.pop() {
            Some(mut set) => {
                set.clear();
                set
            }
            None => {
                self.scratch.charge(self.set_bytes());
                self.problem.forest.empty_set()
            }
        }
    }

    /// Returns a set to the pool.
    pub(super) fn give_set(&mut self, set: OccSet) {
        self.pools.sets.push(set);
    }

    /// Takes an empty linear zone from the pool.
    pub(super) fn take_context(&mut self) -> Context {
        match self.pools.contexts.pop() {
            Some(mut context) => {
                context.clear();
                context
            }
            None => {
                self.scratch.charge(self.set_bytes());
                Context::empty(self.problem.forest.len())
            }
        }
    }

    /// Takes a linear zone from the pool with any contents, for a caller
    /// that overwrites it whole: clearing it first would write the
    /// forest's width once more.
    pub(super) fn take_context_any(&mut self) -> Context {
        self.pools.contexts.pop().unwrap_or_else(|| {
            self.scratch.charge(self.set_bytes());
            Context::empty(self.problem.forest.len())
        })
    }

    /// Takes a linear zone from the pool that is a copy of `gamma`.
    pub(super) fn take_context_from(&mut self, gamma: &Context) -> Context {
        let mut context = self.take_context_any();
        context.clone_from(gamma);
        context
    }

    /// Returns a linear zone to the pool.
    pub(super) fn give_context(&mut self, context: Context) {
        self.pools.contexts.push(context);
    }

    /// Takes an empty list from the pool.
    pub(super) fn take_list(&mut self) -> Pooled<OccId> {
        let mut list = self.pools.lists.pop().unwrap_or_default();
        list.clear();
        list
    }

    /// Returns a list to the pool.
    pub(super) fn give_list(&mut self, mut list: Pooled<OccId>) {
        list.settle(&mut self.scratch);
        self.pools.lists.push(list);
    }

    /// Takes an empty tally from the pool.
    pub(super) fn take_tally(&mut self) -> Tally {
        match self.pools.tallies.pop() {
            Some(mut tally) => {
                tally.clear();
                tally
            }
            None => {
                self.scratch.charge(self.problem.counts.tally_bytes());
                self.problem.counts.tally()
            }
        }
    }

    /// Returns a tally to the pool.
    pub(super) fn give_tally(&mut self, tally: Tally) {
        self.pools.tallies.push(tally);
    }

    /// Takes the counts of a split with no member from the pool.
    pub(super) fn take_split(&mut self) -> Box<Split> {
        match self.pools.splits.pop() {
            Some(mut split) => {
                split.clear();
                split
            }
            None => {
                self.scratch.charge(self.problem.counts.split_bytes());
                Box::new(self.problem.counts.split())
            }
        }
    }

    /// Returns the counts of a split to the pool.
    pub(super) fn give_split(&mut self, split: Box<Split>) {
        self.pools.splits.push(split);
    }

    /// Takes an empty trail from the pool.
    pub(super) fn take_trail(&mut self) -> Pooled<Branch> {
        let mut trail = self.pools.trails.pop().unwrap_or_default();
        trail.clear();
        trail
    }

    /// Returns a trail to the pool.
    pub(super) fn give_trail(&mut self, mut trail: Pooled<Branch>) {
        trail.settle(&mut self.scratch);
        self.pools.trails.push(trail);
    }

    /// Takes an empty list of links from the pool.
    pub(super) fn take_links(&mut self) -> Pooled<(OccId, NodeId, bool)> {
        let mut links = self.pools.links.pop().unwrap_or_default();
        links.clear();
        links
    }

    /// Returns a list of links to the pool.
    pub(super) fn give_links(&mut self, mut links: Pooled<(OccId, NodeId, bool)>) {
        links.settle(&mut self.scratch);
        self.pools.links.push(links);
    }

    /// Takes an empty list of the frames of a chain of free splits from
    /// the pool.
    pub(super) fn take_frames(&mut self) -> Vec<Frame> {
        self.pools.frames.pop().unwrap_or_default()
    }

    /// Returns a list of frames to the pool, emptied.
    pub(super) fn give_frames(&mut self, frames: Vec<Frame>) {
        debug_assert!(frames.is_empty(), "the frames are closed");
        self.pools.frames.push(frames);
    }

    /// Takes cursors at the head of every list from the pool.
    pub(super) fn take_cursors(&mut self) -> Cursors {
        self.pools.cursors.pop().unwrap_or_else(|| {
            let lists = 2 * self.problem.forest.sequent().atom_names().len();
            self.scratch.charge(lists * size_of::<u32>());
            Cursors {
                passed: vec![0; lists],
                moved: Vec::new(),
            }
        })
    }

    /// Returns cursors to the pool, back at the head of every list.
    pub(super) fn give_cursors(&mut self, mut cursors: Cursors) {
        for list in cursors.moved.drain(..) {
            cursors.passed[list as usize] = 0;
        }
        self.pools.cursors.push(cursors);
    }
}

/// The spare buffers of an engine, one pool of each kind.
#[derive(Default)]
pub(super) struct Pools {
    /// Spare occurrence sets of the forest's width.
    sets: Vec<OccSet>,
    /// Spare linear zones of the forest's width.
    contexts: Vec<Context>,
    /// Spare lists of occurrences.
    lists: Vec<Pooled<OccId>>,
    /// Spare tallies, a column per atom that has rows.
    tallies: Vec<Tally>,
    /// Spare counts of splits, boxed so that a split search holds a
    /// pointer on the stack and not the counts: a level of recursion
    /// through a searched split took 1.5 KiB with them in the frame.
    #[allow(clippy::vec_box)]
    splits: Vec<Box<Split>>,
    /// Spare trails of split searches.
    trails: Vec<Pooled<Branch>>,
    /// Spare lists of the links of a chain of forced splits.
    links: Vec<Pooled<(OccId, NodeId, bool)>>,
    /// Spare cursors of a chain of forced splits.
    cursors: Vec<Cursors>,
    /// Spare lists of the frames of a chain of free splits.
    frames: Vec<Vec<Frame>>,
}

/// A list from one of the engine's pools, which knows how much of its
/// allocation the search's account was charged: a list grows while a rule
/// uses it, and the growth is charged when the rule gives it back, so the
/// lists a branch has taken count as far as they had grown when they
/// were last returned.
pub(super) struct Pooled<T> {
    /// The list.
    items: Vec<T>,
    /// The bytes of its allocation that were charged.
    charged: usize,
}

impl<T> Default for Pooled<T> {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            charged: 0,
        }
    }
}

impl<T> Pooled<T> {
    /// Charges the account what the list's allocation grew by since it
    /// was last charged.
    pub(super) fn settle(&mut self, account: &mut Charged<'_>) {
        let bytes = bytes_of(&self.items);
        if bytes != self.charged {
            account.resize(self.charged, bytes);
            self.charged = bytes;
        }
    }
}

impl<T> std::ops::Deref for Pooled<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Vec<T> {
        &self.items
    }
}

impl<T> std::ops::DerefMut for Pooled<T> {
    fn deref_mut(&mut self) -> &mut Vec<T> {
        &mut self.items
    }
}
