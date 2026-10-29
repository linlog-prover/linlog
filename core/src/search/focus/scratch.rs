// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The engine's pools of scratch buffers: a step takes what it needs
//! from a pool and gives it back, so that no step allocates once the
//! pools are warm.

use super::Run;
use super::counts::{Split, Tally};
use super::split::Frame;
use super::zone::{Cursors, Linear as _, Unrestricted as _, Zone};
use crate::occurrences::OccId;
use crate::proofs::{Branch, NodeId};
use crate::search::memory::{Charged, bytes_of};

impl<Z: Zone> Run<'_, Z> {
    /// Takes an empty set from the pool.
    pub(super) fn take_set(&mut self) -> Z::Unrestricted {
        match self.pools.sets.pop() {
            Some(mut set) => {
                set.clear();
                set
            }
            None => {
                self.scratch.charge(self.set_bytes());
                Z::Unrestricted::empty(self.problem.forest)
            }
        }
    }

    /// Returns a set to the pool.
    pub(super) fn give_set(&mut self, set: Z::Unrestricted) {
        self.pools.sets.push(set);
    }

    /// Takes an empty linear zone from the pool.
    pub(super) fn take_context(&mut self) -> Z::Linear {
        match self.pools.contexts.pop() {
            Some(mut context) => {
                context.clear();
                context
            }
            None => {
                self.scratch.charge(self.set_bytes());
                Z::Linear::empty(self.problem.forest)
            }
        }
    }

    /// Takes a linear zone from the pool with any contents, for a caller
    /// that overwrites it whole: clearing it first would write the
    /// forest's width once more.
    pub(super) fn take_context_any(&mut self) -> Z::Linear {
        self.pools.contexts.pop().unwrap_or_else(|| {
            self.scratch.charge(self.set_bytes());
            Z::Linear::empty(self.problem.forest)
        })
    }

    /// Takes a linear zone from the pool that is a copy of `gamma`.
    pub(super) fn take_context_from(&mut self, gamma: &Z::Linear) -> Z::Linear {
        let mut context = self.take_context_any();
        context.copy_from(gamma);
        context
    }

    /// Returns a linear zone to the pool.
    pub(super) fn give_context(&mut self, context: Z::Linear) {
        self.pools.contexts.push(context);
    }

    /// Takes an empty list from the pool.
    pub(super) fn take_list(&mut self) -> Pooled<Z::Member> {
        let mut list = self.pools.lists.pop().unwrap_or_default();
        list.clear();
        list
    }

    /// Returns a list to the pool.
    pub(super) fn give_list(&mut self, mut list: Pooled<Z::Member>) {
        list.settle(&mut self.scratch);
        self.pools.lists.push(list);
    }

    /// Takes an empty list of the words of a memo's key from the pool.
    pub(super) fn take_words(&mut self) -> Pooled<OccId> {
        let mut words = Z::word_lists(&mut self.pools).pop().unwrap_or_default();
        words.clear();
        words
    }

    /// Returns a list of words to the pool.
    pub(super) fn give_words(&mut self, mut words: Pooled<OccId>) {
        words.settle(&mut self.scratch);
        Z::word_lists(&mut self.pools).push(words);
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
    #[expect(
        clippy::unnecessary_box_returns,
        reason = "a split search's frame holds its counts, and the box keeps that frame a pointer wide on every level of the recursion"
    )]
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
    pub(super) fn take_links(&mut self) -> Pooled<(Z::Member, NodeId, Branch)> {
        let mut links = self.pools.links.pop().unwrap_or_default();
        links.clear();
        links
    }

    /// Returns a list of links to the pool.
    pub(super) fn give_links(&mut self, mut links: Pooled<(Z::Member, NodeId, Branch)>) {
        links.settle(&mut self.scratch);
        self.pools.links.push(links);
    }

    /// Takes an empty list of the frames of a chain of free splits from
    /// the pool.
    pub(super) fn take_frames(&mut self) -> Vec<Frame<Z>> {
        self.pools.frames.pop().unwrap_or_default()
    }

    /// Returns a list of frames to the pool, emptied.
    pub(super) fn give_frames(&mut self, frames: Vec<Frame<Z>>) {
        debug_assert!(frames.is_empty(), "the frames are closed");
        self.pools.frames.push(frames);
    }

    /// Takes cursors at the head of every list from the pool.
    pub(super) fn take_cursors(&mut self) -> Cursors {
        self.pools.cursors.pop().unwrap_or_else(|| {
            let lists = self.problem.forest.lists();
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
pub(super) struct Pools<Z: Zone> {
    /// Spare unrestricted zones of the forest's width.
    sets: Vec<Z::Unrestricted>,
    /// Spare linear zones of the forest's width.
    contexts: Vec<Z::Linear>,
    /// Spare lists of members.
    pub(super) lists: Vec<Pooled<Z::Member>>,
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
    links: Vec<Pooled<(Z::Member, NodeId, Branch)>>,
    /// Spare cursors of a chain of forced splits.
    cursors: Vec<Cursors>,
    /// Spare lists of the frames of a chain of free splits.
    frames: Vec<Vec<Frame<Z>>>,
}

impl<Z: Zone> Default for Pools<Z> {
    fn default() -> Self {
        Self {
            sets: Vec::new(),
            contexts: Vec::new(),
            lists: Vec::new(),
            tallies: Vec::new(),
            splits: Vec::new(),
            trails: Vec::new(),
            links: Vec::new(),
            cursors: Vec::new(),
            frames: Vec::new(),
        }
    }
}

/// A list from one of the engine's pools, which knows how much of its
/// allocation the search's account was charged: a list grows while a rule
/// uses it, and the growth is charged when the rule gives it back, so the
/// lists a branch has taken count as far as they had grown when they
/// were last returned.
#[derive(Debug)]
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

    /// Returns the bytes of its allocation that were charged.
    pub(super) fn charged(&self) -> usize {
        self.charged
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
