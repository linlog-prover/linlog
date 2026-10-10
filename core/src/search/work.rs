// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! A search's count of its work, which every thread of it adds to, the
//! bound on it, and the helpers that take what its threads share.

#[cfg(doc)]
use crate::limits::Limits;

/// The work a search has done, in its engine's units, shared by every
/// thread of it, and the bound on it ([`Limits::work`]). Each thread
/// counts its own units apart and adds them in batches of
/// [`Work::BATCH`], the rest when it ends: an atomic addition at every
/// poll cost the additive path 5 % of its instructions, and would contend
/// on a pool.
#[derive(Debug)]
pub(crate) struct Work {
    /// The units done.
    done: std::sync::atomic::AtomicU64,
    /// The most units allowed; `u64::MAX` for no bound, which no search
    /// reaches (a unit takes a nanosecond at the least).
    limit: u64,
}

impl Work {
    /// How many units a thread counts before it adds them: what the total
    /// and the bound see of it is late by at most this much per thread.
    #[cfg(feature = "parallel")]
    pub(crate) const BATCH: u64 = 1 << 12;

    /// No work done yet, under the bound `limit`.
    pub(crate) const fn new(limit: Option<u64>) -> Self {
        Self {
            done: std::sync::atomic::AtomicU64::new(0),
            limit: match limit {
                Some(limit) => limit,
                None => u64::MAX,
            },
        }
    }

    /// Adds `units` and returns the units done since the search began.
    pub(crate) fn add(&self, units: u64) -> u64 {
        self.done
            .fetch_add(units, std::sync::atomic::Ordering::Relaxed)
            .saturating_add(units)
    }

    /// Returns the units done since the search began.
    pub(crate) fn done(&self) -> u64 {
        self.done.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Returns the most units allowed.
    pub(crate) const fn limit(&self) -> u64 {
        self.limit
    }

    /// Returns whether the work passed its bound.
    pub(crate) fn passed(&self) -> bool {
        self.done() > self.limit
    }
}

/// Locks what a search's threads share. A panic in one of them leaves
/// it whole, since each change is one assignment or one push, so a
/// poisoned lock is taken as it is.
pub(crate) fn lock<T>(shared: &std::sync::Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    shared
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Returns what the threads shared, once they ended, poisoned or not, as
/// [`lock`] takes it.
#[cfg(feature = "parallel")]
pub(crate) fn taken<T>(shared: std::sync::Mutex<T>) -> T {
    shared
        .into_inner()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// A thread's count of the work it does: its units not yet added to the
/// search's [`Work`], which it adds in batches of [`Work::BATCH`] (with
/// `parallel`, where other threads read the count) and the rest when it
/// drops. Every thread of a search counts through one.
pub(crate) struct Counted<'a> {
    /// The search's work.
    work: &'a Work,
    /// The units not yet added to it.
    pending: u64,
}

impl<'a> Counted<'a> {
    /// No units counted yet towards `work`.
    pub(crate) const fn new(work: &'a Work) -> Self {
        Self { work, pending: 0 }
    }

    /// Counts `units` and returns the search's units in all, this
    /// thread's not yet added included.
    pub(crate) fn add(&mut self, units: u64) -> u64 {
        self.pending += units;
        #[cfg(feature = "parallel")]
        if self.pending >= Work::BATCH {
            self.work.add(self.pending);
            self.pending = 0;
        }
        self.done()
    }

    /// Returns the search's units in all, this thread's not yet added
    /// included.
    pub(crate) fn done(&self) -> u64 {
        self.work.done().saturating_add(self.pending)
    }

    /// Returns whether the search's work, this thread's included, passed
    /// its bound.
    pub(crate) fn passed(&self) -> bool {
        self.done() > self.work.limit
    }
}

impl Drop for Counted<'_> {
    /// Adds the units not yet added.
    fn drop(&mut self) {
        self.work.add(self.pending);
    }
}
