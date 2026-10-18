// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The memory a search may hold: a count of the bytes its growing
//! structures have allocated, against the bound of the options. What
//! grows charges it where it allocates (the memo, the proof arena, the
//! counts, the buffers a level of recursion takes), by the capacity
//! allocated and not by what is in use, so the count follows the memory
//! the process holds; what was freed releases it. The count is atomic
//! because the workers of a parallel search share one account; one thread
//! pays an uncontended addition per allocation, which is rare by design.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// The bytes a search holds, and the most it may.
#[derive(Debug)]
pub(crate) struct Account {
    /// The bound, or `u64::MAX` for none.
    limit: u64,
    /// The bytes charged and not released. A sum of sizes of live
    /// allocations, each below `isize::MAX`, of which an address space
    /// holds fewer than 2⁶⁴ bytes' worth: it cannot wrap.
    used: AtomicU64,
    /// The account of the race this one is a part of, which every charge
    /// counts in too and which no part passes: the two searches of a race
    /// draw on one bound. `None` outside a race.
    whole: Option<Arc<Account>>,
    /// What this account held in the whole when it began: a fork starts
    /// from what its original holds, which the whole counts already.
    base: u64,
    /// What it holds for its whole life from the start, the input of the
    /// searches it bounds, which its parts leave room for.
    #[cfg_attr(
        not(feature = "parallel"),
        expect(dead_code, reason = "only a race's parts read it")
    )]
    input: u64,
}

impl Account {
    /// An account with nothing charged and the bound given, or none.
    #[cfg(any(test, feature = "interactive"))]
    pub(crate) fn new(limit: Option<u64>) -> Self {
        Self::holding(limit, 0)
    }

    /// An account with the bound given that holds `input` bytes from the
    /// start to its end: what the searches it bounds read, which they
    /// hold as long as they run.
    pub(crate) fn holding(limit: Option<u64>, input: u64) -> Self {
        Self {
            limit: limit.unwrap_or(u64::MAX),
            used: AtomicU64::new(input),
            whole: None,
            base: 0,
            input,
        }
    }

    /// An account for one search of a race, with the whole's bound less
    /// its input, whose charges count in the whole: what one search
    /// holds the other may not.
    #[cfg(feature = "parallel")]
    pub(crate) fn part_of(whole: &Arc<Account>) -> Self {
        Self {
            limit: if whole.limit == u64::MAX {
                u64::MAX
            } else {
                whole.limit.saturating_sub(whole.input)
            },
            used: AtomicU64::new(0),
            whole: Some(whole.clone()),
            base: 0,
            input: 0,
        }
    }

    /// An account for one of `parts` searches that run side by side: what
    /// the bound leaves beside what this account holds (its input, or a
    /// reading) divided evenly, so that neither search's memory decides
    /// what the other may keep; within a race, a part of its whole too.
    pub(crate) fn share(&self, parts: u64) -> Self {
        Self {
            limit: if self.limit == u64::MAX {
                u64::MAX
            } else {
                self.limit.saturating_sub(self.used()) / parts
            },
            used: AtomicU64::new(0),
            whole: self.whole.clone(),
            base: 0,
            input: 0,
        }
    }

    /// An account with the same bound that starts from what this one
    /// holds now: for a search that starts afresh beside what was set up
    /// for it, and whose own memory goes when it ends.
    pub(crate) fn fork(&self) -> Self {
        Self {
            limit: self.limit,
            used: AtomicU64::new(self.used()),
            whole: self.whole.clone(),
            base: self.used(),
            input: 0,
        }
    }

    /// Returns the bound, or `u64::MAX` for none.
    pub(crate) fn limit(&self) -> u64 {
        self.limit
    }

    /// Counts `bytes` more as held.
    pub(crate) fn charge(&self, bytes: usize) {
        self.used.fetch_add(bytes as u64, Ordering::Relaxed);
        if let Some(whole) = &self.whole {
            whole.charge(bytes);
        }
    }

    /// Counts `bytes` as given back, which must have been charged.
    pub(crate) fn release(&self, bytes: usize) {
        self.used.fetch_sub(bytes as u64, Ordering::Relaxed);
        if let Some(whole) = &self.whole {
            whole.release(bytes);
        }
    }

    /// Gives back everything charged since the account held `held` bytes:
    /// what a search that has ended left charged (tables of its set-up and
    /// a memo's reserve, which it never releases), which it no longer
    /// holds.
    pub(crate) fn rewind(&self, held: u64) {
        let since = self.used().saturating_sub(held);
        self.used.fetch_sub(since, Ordering::Relaxed);
        if let Some(whole) = &self.whole {
            whole
                .used
                .fetch_sub(since.min(whole.used()), Ordering::Relaxed);
        }
    }

    /// Counts a buffer whose allocation went from `before` to `after`
    /// bytes.
    pub(crate) fn resize(&self, before: usize, after: usize) {
        if after > before {
            self.charge(after - before);
        } else if before > after {
            self.release(before - after);
        }
    }

    /// Returns the bytes held.
    pub(crate) fn used(&self) -> u64 {
        self.used.load(Ordering::Relaxed)
    }

    /// Returns the bytes left under the bound, and under the whole's,
    /// or `None` where there is no bound.
    pub(crate) fn left(&self) -> Option<u64> {
        let own = (self.limit != u64::MAX).then(|| self.limit.saturating_sub(self.used()));
        match (own, self.whole.as_ref().and_then(|whole| whole.left())) {
            (Some(own), Some(whole)) => Some(own.min(whole)),
            (own, whole) => own.or(whole),
        }
    }

    /// Returns whether `bytes` more would still be within the bound.
    pub(crate) fn fits(&self, bytes: usize) -> bool {
        self.used().saturating_add(bytes as u64) <= self.limit
            && self.whole.as_ref().is_none_or(|whole| whole.fits(bytes))
    }

    /// Returns whether `bytes` more would leave an eighth of the bound
    /// free. The memo asks this before it grows: it would take whatever
    /// is left, and what cannot be emptied (the proofs kept, the buffers
    /// of the recursion) would then pass the bound with every node, each
    /// time at the price of the whole memo.
    pub(crate) fn spares(&self, bytes: usize) -> bool {
        self.used()
            .saturating_add(bytes as u64)
            .saturating_add(self.limit / 8)
            <= self.limit
            && self.whole.as_ref().is_none_or(|whole| whole.spares(bytes))
    }

    /// Returns whether more is held than the bound allows.
    pub(crate) fn over(&self) -> bool {
        self.used() > self.limit || self.whole.as_ref().is_some_and(|whole| whole.over())
    }
}

impl Drop for Account {
    /// Gives the whole back what this account still holds of its own.
    fn drop(&mut self) {
        if let Some(whole) = &self.whole {
            let own = self.used().saturating_sub(self.base);
            whole
                .used
                .fetch_sub(own.min(whole.used()), Ordering::Relaxed);
        }
    }
}

/// What one engine charged an account for buffers of its own, which go
/// when the engine does: dropping this releases them. The workers of a
/// parallel search come and go by the thousand, each with its buffers.
#[derive(Debug)]
pub(crate) struct Charged<'a> {
    /// The account.
    account: &'a Account,
    /// The bytes charged through this value and not released.
    bytes: usize,
}

impl<'a> Charged<'a> {
    /// Nothing charged to `account` yet.
    pub(crate) fn new(account: &'a Account) -> Self {
        Self { account, bytes: 0 }
    }

    /// Returns the account.
    pub(crate) fn account(&self) -> &'a Account {
        self.account
    }

    /// Counts `bytes` more as held.
    pub(crate) fn charge(&mut self, bytes: usize) {
        self.bytes += bytes;
        self.account.charge(bytes);
    }

    /// Counts a buffer whose allocation went from `before` to `after`
    /// bytes.
    pub(crate) fn resize(&mut self, before: usize, after: usize) {
        self.bytes = self.bytes + after - before;
        self.account.resize(before, after);
    }
}

impl Drop for Charged<'_> {
    fn drop(&mut self) {
        self.account.release(self.bytes);
    }
}

/// Returns the bytes a vector's allocation takes.
pub(crate) fn bytes_of<T>(vector: &Vec<T>) -> usize {
    vector.capacity() * size_of::<T>()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A charge counts, a release takes it back, and the bound is
    /// inclusive; a share has its part of what the bound leaves beside
    /// what its account holds, a race's input included, and a count of
    /// its own.
    #[test]
    fn accounting() {
        let account = Account::new(Some(100));
        account.charge(60);
        assert!(account.fits(40) && !account.fits(41) && !account.over());
        assert!(account.spares(28) && !account.spares(29));
        account.resize(60, 110);
        assert!(account.over());
        account.release(20);
        assert_eq!(account.used(), 90);
        assert_eq!(account.left(), Some(10));
        let half = account.share(2);
        assert!(half.fits(5) && !half.fits(6), "half of what is left");
        assert_eq!(Account::new(None).left(), None);
        assert!(Account::new(None).share(2).fits(usize::MAX));
        // A race's parts leave room for the input the whole holds.
        #[cfg(feature = "parallel")]
        {
            let whole = Arc::new(Account::holding(Some(100), 40));
            let half = Account::part_of(&whole).share(2);
            assert!(half.fits(30) && !half.fits(31));
        }
        let before = account.used();
        let mut own = Charged::new(&account);
        own.charge(7);
        own.resize(7, 3);
        assert_eq!(account.used(), before + 3);
        drop(own);
        assert_eq!(account.used(), before);
    }
}
