// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The parallel runtime: a thread pool per search, built for it or lent
//! by a [`Pool`] the caller keeps across searches, an atomic stop flag
//! the workers poll, and the driver that polls the caller's stop
//! condition on the calling thread while the pool searches. The engines
//! own the rest (shared memo, cubes, per-worker state); nothing here is
//! global.

use crate::Error;
use rayon::{ThreadPool, ThreadPoolBuilder};
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::ops::Deref;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{RecvTimeoutError, channel};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

/// How long the driver waits for the result before it polls the caller's
/// stop condition again.
const POLL: Duration = Duration::from_millis(1);

/// The threads a search uses of the `jobs` its options name: no more than
/// the machine runs at once, where the platform tells how many that is.
pub(crate) fn threads(jobs: usize) -> usize {
    if jobs <= 1 {
        return jobs;
    }
    std::thread::available_parallelism().map_or(jobs, |machine| jobs.min(machine.get()))
}

/// The most runtimes a [`Pool`] keeps between searches: one search
/// holds two at most (the two searches of the default bias), and two
/// parallel searches running at once through one pool hold four. A kept
/// runtime's threads sleep, but each holds its stack's mapping and a
/// thread of the process, so a pool keeps no more than it can lend at
/// once.
const MAX_KEPT: usize = 4;

/// Thread pools kept across searches, for a caller that runs many
/// parallel searches one after another, such as a batch of sequents: a
/// search with this handle in its options
/// ([`Options::pool`](super::Options::pool)) borrows a kept pool of
/// exactly the threads it runs on, or builds one, and gives it back when
/// it ends, so that the threads start once rather than per search. The
/// search on a kept pool is the search on a fresh one; a kept pool is
/// lent to one search at a time, so searches running at once through
/// one handle each run on pools of their own. At most four pools are
/// kept; their threads end when the last clone of the handle is dropped
/// and no search holds them. Clones share the pools; nothing is global.
///
/// Needs the cargo feature `parallel` (off by default).
#[derive(Clone, Default)]
pub struct Pool {
    /// The shared state.
    inner: Arc<Kept>,
}

/// What the clones of a [`Pool`] share.
#[derive(Default)]
struct Kept {
    /// The runtimes no search holds, the most recently given back last.
    idle: Mutex<Vec<Runtime>>,
    /// How many runtimes the pool has built.
    #[cfg(test)]
    built: std::sync::atomic::AtomicUsize,
}

impl Pool {
    /// Returns an empty pool, which builds its thread pools as searches
    /// ask for them.
    pub fn new() -> Self {
        Self::default()
    }

    /// Locks the idle runtimes; a panic elsewhere leaves the list whole,
    /// since it is changed only by one push or one removal.
    fn idle(&self) -> MutexGuard<'_, Vec<Runtime>> {
        self.inner
            .idle
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Gives a runtime back for later searches, dropping the one given
    /// back longest ago when the pool is full.
    fn give_back(&self, runtime: Runtime) {
        let evicted = {
            let mut idle = self.idle();
            idle.push(runtime);
            (idle.len() > MAX_KEPT).then(|| idle.remove(0))
        };
        // Its threads are told to end outside the lock.
        drop(evicted);
    }
}

impl PartialEq for Pool {
    /// Whether the two handles are clones of one pool.
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}

impl Eq for Pool {}

impl Debug for Pool {
    /// Writes the pool's address and the thread counts of its idle pools.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let threads: Vec<usize> = self.idle().iter().map(Runtime::threads).collect();
        f.debug_struct("Pool")
            .field("at", &Arc::as_ptr(&self.inner))
            .field("idle", &threads)
            .finish()
    }
}

/// A runtime a search holds: lent by a [`Pool`], which takes it back when
/// this is dropped (on a return, an error or a panic, after every task
/// on it has ended), or built for the search alone and dropped with it.
pub(crate) struct Lent<'p> {
    /// The runtime, until it is given back.
    runtime: Option<Runtime>,
    /// The pool it goes back to, if any.
    pool: Option<&'p Pool>,
}

impl<'p> Lent<'p> {
    /// Takes a runtime of `threads` workers (at least one) whose stacks
    /// have at least `stack_size` bytes: an idle one of `pool`, or a new
    /// one, as [`Runtime::new`] builds it.
    ///
    /// # Errors
    ///
    /// [`Error::ThreadPool`] when a new pool's threads cannot start.
    pub(crate) fn take(
        pool: Option<&'p Pool>,
        threads: usize,
        stack_size: usize,
    ) -> Result<Self, Error> {
        let threads = threads.max(1);
        let kept = pool.and_then(|pool| {
            let mut idle = pool.idle();
            let at = idle
                .iter()
                .rposition(|r| r.threads == threads && r.stack_size >= stack_size)?;
            Some(idle.remove(at))
        });
        let runtime = match kept {
            Some(runtime) => runtime,
            None => {
                let runtime = Runtime::new(threads, stack_size)?;
                #[cfg(test)]
                if let Some(pool) = pool {
                    pool.inner.built.fetch_add(1, Ordering::Relaxed);
                }
                runtime
            }
        };
        Ok(Self {
            runtime: Some(runtime),
            pool,
        })
    }
}

impl Deref for Lent<'_> {
    type Target = Runtime;

    /// The runtime held.
    fn deref(&self) -> &Runtime {
        self.runtime
            .as_ref()
            .expect("a lent runtime is taken only when it is dropped")
    }
}

impl Drop for Lent<'_> {
    /// Gives the runtime back to its pool, if it came from one. Every
    /// task on it has ended, since a search's scopes end before it does.
    fn drop(&mut self) {
        if let (Some(runtime), Some(pool)) = (self.runtime.take(), self.pool) {
            pool.give_back(runtime);
        }
    }
}

/// The pool a parallel search runs on, built for one search and dropped
/// with it or kept by a [`Pool`]: `threads` workers whose stacks fit the
/// recursion limit.
pub(crate) struct Runtime {
    /// The pool.
    pub(crate) pool: ThreadPool,
    /// How many workers it has.
    threads: usize,
    /// The size of each worker's stack, in bytes.
    stack_size: usize,
}

impl Runtime {
    /// Starts a pool of `threads` workers with stacks of `stack_size`
    /// bytes, or says why it cannot ([`Error::ThreadPool`]).
    pub(crate) fn new(threads: usize, stack_size: usize) -> Result<Self, Error> {
        let threads = threads.max(1);
        let pool = ThreadPoolBuilder::new()
            .num_threads(threads)
            .stack_size(stack_size)
            .thread_name(|i| format!("linlog-search-{i}"))
            .build()
            .map_err(|e| Error::ThreadPool {
                threads,
                message: e.to_string(),
            })?;
        Ok(Self {
            pool,
            threads,
            stack_size,
        })
    }

    /// Returns how many workers the pool has.
    pub(crate) fn threads(&self) -> usize {
        self.threads
    }

    /// Runs `work` on the pool, given the root of the stop flags, and
    /// returns its result. Meanwhile the calling thread polls `stop` once
    /// a millisecond and raises the flag when it fires, so the caller's
    /// condition needs to be neither `Send` nor fast. A panic in `work`
    /// propagates to the caller.
    pub(crate) fn drive<T: Send>(
        &self,
        stop: &mut dyn FnMut() -> bool,
        work: impl FnOnce(Flags<'_>) -> T + Send,
    ) -> T {
        let flag = AtomicBool::new(false);
        let (sender, receiver) = channel();
        let result = self.pool.in_place_scope(|scope| {
            let flag = &flag;
            scope.spawn(move |_| {
                let result = work(Flags::root(flag));
                // The receiver is gone only when the caller's loop ended,
                // which it does only on a result.
                let _ = sender.send(result);
            });
            loop {
                match receiver.recv_timeout(POLL) {
                    Ok(result) => return Some(result),
                    Err(RecvTimeoutError::Timeout) => {
                        if stop() {
                            flag.store(true, Ordering::Relaxed);
                        }
                    }
                    // The task ended without a result: it panicked, and the
                    // scope resumes the panic once the loop lets it end.
                    Err(RecvTimeoutError::Disconnected) => return None,
                }
            }
        });
        result.expect("a task that ends without a result panicked, which the scope propagates")
    }
}

/// Runs two pieces of work at once, each on a pool of its own and given
/// the root of its own stop flags, and returns both results. The calling
/// thread polls `stop` once a millisecond, as [`Runtime::drive`] does, and
/// raises both flags when it fires; a result that `settles` the matter
/// raises the other work's flag, so the other returns as soon as it polls.
/// A panic in either propagates to the caller once both have ended.
pub(crate) fn race<T: Send>(
    runtimes: (&Runtime, &Runtime),
    stop: &mut dyn FnMut() -> bool,
    work: (
        impl FnOnce(Flags<'_>) -> T + Send,
        impl FnOnce(Flags<'_>) -> T + Send,
    ),
    settles: impl Fn(&T) -> bool,
) -> (T, T) {
    let flags = [AtomicBool::new(false), AtomicBool::new(false)];
    let (sender, receiver) = channel();
    let mut results = (None, None);
    runtimes.0.pool.in_place_scope(|first| {
        runtimes.1.pool.in_place_scope(|second| {
            let flags = &flags;
            let other = sender.clone();
            first.spawn(move |_| {
                let _ = sender.send((0, work.0(Flags::root(&flags[0]))));
            });
            second.spawn(move |_| {
                let _ = other.send((1, work.1(Flags::root(&flags[1]))));
            });
            while results.0.is_none() || results.1.is_none() {
                match receiver.recv_timeout(POLL) {
                    Ok((i, result)) => {
                        if settles(&result) {
                            flags[1 - i].store(true, Ordering::Relaxed);
                        }
                        if i == 0 {
                            results.0 = Some(result);
                        } else {
                            results.1 = Some(result);
                        }
                    }
                    Err(RecvTimeoutError::Timeout) => {
                        if stop() {
                            flags[0].store(true, Ordering::Relaxed);
                            flags[1].store(true, Ordering::Relaxed);
                        }
                    }
                    // Both tasks ended and one without a result: it
                    // panicked, and its scope resumes the panic.
                    Err(RecvTimeoutError::Disconnected) => break,
                }
            }
        });
    });
    match results {
        (Some(first), Some(second)) => (first, second),
        _ => unreachable!("a task that ends without a result panicked, which its scope propagates"),
    }
}

/// The stop flags a worker polls: its own, which a sibling raises to
/// cancel it once their common alternative is settled, and its
/// ancestors', up to the root flag the driver raises for the caller's stop
/// condition. A chain is as long as the nesting of parallel choices, a
/// few links at most.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Flags<'a> {
    /// This level's flag.
    flag: &'a AtomicBool,
    /// The enclosing level's flags.
    parent: Option<&'a Flags<'a>>,
}

impl<'a> Flags<'a> {
    /// The root of a chain: the driver's flag.
    pub(crate) fn root(flag: &'a AtomicBool) -> Self {
        Self { flag, parent: None }
    }

    /// A level below this one, with a flag of its own.
    pub(crate) fn child<'b>(&'b self, flag: &'b AtomicBool) -> Flags<'b>
    where
        'a: 'b,
    {
        Flags {
            flag,
            parent: Some(self),
        }
    }

    /// Whether any flag of the chain is raised.
    pub(crate) fn raised(&self) -> bool {
        let mut level = Some(self);
        while let Some(flags) = level {
            if flags.flag.load(Ordering::Relaxed) {
                return true;
            }
            level = flags.parent;
        }
        false
    }

    /// Whether this level's own flag is raised, whatever the ancestors'.
    pub(crate) fn own(&self) -> bool {
        self.flag.load(Ordering::Relaxed)
    }
}

#[cfg(all(test, feature = "parse"))]
mod tests {
    use super::{Pool, threads};
    use crate::fragment::Mode;
    use crate::search::{Engine, Options, Verdict, prove};
    use std::sync::atomic::Ordering;

    /// Searches through one pool, on the net engine and on the focused
    /// one, share the one runtime the first built, and each verdict is
    /// that of a search on a runtime of its own.
    #[test]
    fn searches_share_a_kept_runtime() {
        let pool = Pool::new();
        let alone = Options::default().jobs(2);
        let pooled = alone.clone().pool(Some(pool.clone()));
        // A machine of one processor runs every search sequentially.
        let built = usize::from(threads(2) > 1);
        let cases = [
            ("a & b, c |- c * (b & a)", Engine::Focus, true),
            ("a * b |- b * a", Engine::Net, true),
            ("a + b, c |- c * (a & b)", Engine::Focus, false),
        ];
        for (text, engine, provable) in cases {
            let sequent = text.parse().unwrap();
            let fresh = prove(&sequent, Mode::CLASSICAL, &alone).unwrap();
            let kept = prove(&sequent, Mode::CLASSICAL, &pooled).unwrap();
            assert_eq!(kept.engine, engine, "{text}");
            for outcome in [&fresh, &kept] {
                let verdict = &outcome.verdict;
                assert_eq!(matches!(verdict, Verdict::Proved(_)), provable, "{text}");
                assert_eq!(
                    matches!(verdict, Verdict::Unprovable(_)),
                    !provable,
                    "{text}"
                );
            }
            assert_eq!(pool.inner.built.load(Ordering::Relaxed), built, "{text}");
            assert_eq!(pool.idle().len(), built, "{text}");
        }
    }
}
