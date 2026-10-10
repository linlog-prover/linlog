// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Many sequents decided in one call: problems in, results out, in the
//! order of the problems and each as soon as it and those before it are
//! decided, so that a stream of questions is answered one by one. The
//! machine's threads go across the sequents, one sequent per worker on
//! the sequential engines, or within one sequent at a time
//! ([`Cores`]); the memory bound of each search and the batch's own
//! bound decide how many workers run at once (the [`Plan`] each work
//! call is handed).
//! The options of every sequent's search and the [`Limits`] each runs
//! within are the batch's arguments beside its own options.
//! Without the `parallel` feature a batch runs on the caller's thread.
//!
//! # Examples
//!
#![cfg_attr(feature = "parse", doc = "```")]
#![cfg_attr(not(feature = "parse"), doc = "```ignore")]
//! use linlog::Limits;
//! use linlog::search::{self, Verdict};
//! use linlog::search::batch::{Options, Problem, prove};
//!
//! let problems = ["A |- A", "A |- B"].map(|text| Problem::new(text, text.parse().unwrap(), None));
//! let search = search::Options::default();
//! let proved: Vec<bool> = prove(problems, &Options::default(), &search, &Limits::default())
//!     .map(|answer| matches!(answer.outcome.unwrap().verdict, Verdict::Proved(_)))
//!     .collect();
//! assert_eq!(proved, [true, false]);
//! ```

use super::{Options as Search, Outcome};
use crate::{Error, Limits, Mode, Sequent};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// How a batch spends the machine's threads. In JSON (feature
/// `serialize`) `"auto"`, `"across"` or `"within"`.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serialize", serde(rename_all = "lowercase"))]
pub enum Cores {
    /// Across the sequents when the batch has at least as many as there
    /// are workers, else within each.
    #[default]
    Auto,
    /// One sequent per worker, each searched by the sequential engines,
    /// whose outcome is a function of the input.
    Across,
    /// One sequent at a time, searched with the threads of the search
    /// options.
    Within,
}

/// How a batch shares the machine among its sequents. The options of each
/// sequent's search and the limits it runs within are given beside them;
/// `Limits::memory_bytes` is each search's bound.
///
/// # JSON
///
/// With the feature `serialize` an object of the fields, a missing one
/// taking its default and a misspelt one refused: `"mode"` a mode's name
/// (`"classical"`), `"cores"` one of `"auto"`, `"across"` and `"within"`,
/// `"workers"` a number, `"total_memory_bytes"` a number or `null` for no
/// bound.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialize",
    derive(serde::Serialize, serde::Deserialize),
    serde(default, deny_unknown_fields)
)]
pub struct Options {
    /// The mode of a problem that names none.
    pub mode: Mode,
    /// Where the threads go.
    pub cores: Cores,
    /// The most sequents decided at once, across the sequents: at least
    /// one, and more than [`MAX_JOBS`](Search::MAX_JOBS) are taken as that
    /// many.
    pub workers: usize,
    /// The most memory all the searches of the batch may hold together,
    /// or `None` for no bound but each search's.
    #[cfg_attr(feature = "serialize", serde(with = "crate::serialize::exact"))]
    pub total_memory_bytes: Option<u64>,
}

impl Options {
    /// The default number of workers: one, since this crate does not ask
    /// the machine how many threads it runs; a front end sets it.
    pub const DEFAULT_WORKERS: usize = 1;
    /// The default bound of the whole batch, 4 GiB: four searches at the
    /// default bound of each.
    pub const DEFAULT_TOTAL_MEMORY_BYTES: u64 = 4 << 30;

    /// Returns the options with another default [`mode`](Self::mode).
    #[must_use]
    pub const fn with_mode(self, mode: Mode) -> Self {
        Self { mode, ..self }
    }

    /// Returns the options with another use of the [`cores`](Self::cores).
    #[must_use]
    pub const fn with_cores(self, cores: Cores) -> Self {
        Self { cores, ..self }
    }

    /// Returns the options with another number of [`workers`](Self::workers).
    #[must_use]
    pub const fn with_workers(self, workers: usize) -> Self {
        Self { workers, ..self }
    }

    /// Returns the options with another bound on the whole batch's memory
    /// ([`total_memory_bytes`](Self::total_memory_bytes)).
    #[must_use]
    pub const fn with_total_memory_bytes(self, total_memory_bytes: Option<u64>) -> Self {
        Self {
            total_memory_bytes,
            ..self
        }
    }

    /// Returns how the batch runs, `within` its sequents or across them:
    /// across, as many workers as the batch's bound holds searches at the
    /// bound of each (at least one, whose bound is then the batch's), each
    /// on one thread; within, one worker whose search may be two at once
    /// (a front end may race one thread against a pool of the others), so
    /// each holds at most half the batch's bound.
    pub(crate) fn plan(&self, within: bool, search: &Search, limits: &Limits) -> Plan {
        let search = search.clone();
        let mut limits = *limits;
        let total = self.total_memory_bytes;
        if within {
            let share = total.map(|t| if search.threads() > 1 { t / 2 } else { t });
            limits.memory_bytes = smaller(limits.memory_bytes, share);
            return Plan {
                workers: 1,
                search,
                limits,
            };
        }
        let most = self.workers.clamp(1, Search::MAX_JOBS);
        let workers = match (limits.memory_bytes, total) {
            (Some(each), Some(total)) => usize::try_from(total / each.max(1)).unwrap_or(usize::MAX),
            (None, Some(total)) => {
                limits.memory_bytes = Some(total / u64::try_from(most).unwrap_or(u64::MAX));
                usize::MAX
            }
            (_, None) => usize::MAX,
        };
        limits.memory_bytes = smaller(limits.memory_bytes, total);
        Plan {
            workers: workers.clamp(1, most),
            search: search.with_jobs(1),
            limits,
        }
    }
}

/// Returns the smaller of two bounds, `None` being none.
fn smaller(a: Option<u64>, b: Option<u64>) -> Option<u64> {
    match (a, b) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (a, None) => a,
        (None, b) => b,
    }
}

impl Default for Options {
    /// The classical mode, [`DEFAULT_WORKERS`](Self::DEFAULT_WORKERS) and
    /// [`DEFAULT_TOTAL_MEMORY_BYTES`](Self::DEFAULT_TOTAL_MEMORY_BYTES).
    fn default() -> Self {
        Self {
            mode: Mode::CLASSICAL,
            cores: Cores::Auto,
            workers: Self::DEFAULT_WORKERS,
            total_memory_bytes: Some(Self::DEFAULT_TOTAL_MEMORY_BYTES),
        }
    }
}

/// How a batch runs.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Plan {
    /// The sequents decided at once.
    pub workers: usize,
    /// The options of each sequent's search.
    pub search: Search,
    /// The limits each search runs within.
    pub limits: Limits,
}

impl Plan {
    /// Returns the plan of one sequent decided alone, with these options
    /// and limits.
    pub const fn alone(search: Search, limits: Limits) -> Self {
        Self {
            workers: 1,
            search,
            limits,
        }
    }
}

/// A problem of [`prove`].
#[non_exhaustive]
#[derive(Clone, Debug)]
pub struct Problem {
    /// Its name, which its answer carries.
    pub name: String,
    /// The sequent.
    pub sequent: Sequent,
    /// Its mode, or `None` for the batch's.
    pub mode: Option<Mode>,
}

impl Problem {
    /// Returns the problem `name` of deciding `sequent`, in `mode` or the
    /// batch's.
    pub fn new(name: impl Into<String>, sequent: Sequent, mode: Option<Mode>) -> Self {
        Self {
            name: name.into(),
            sequent,
            mode,
        }
    }
}

/// The answer to a problem of [`prove`].
#[non_exhaustive]
#[derive(Clone, Debug)]
pub struct Answer {
    /// The problem's name.
    pub name: String,
    /// What the search returned.
    pub outcome: Result<Outcome, Error>,
}

/// Decides the problems, and returns their answers in order as they are
/// decided. Each search ends by the limits or when the results are
/// cancelled ([`Results::cancel`], or dropped); a front end that needs a
/// time limit of its own calls [`run`].
pub fn prove(
    problems: impl IntoIterator<Item = Problem, IntoIter: Send + 'static>,
    options: &Options,
    search: &Search,
    limits: &Limits,
) -> Results<Answer> {
    let mode = options.mode;
    run(
        problems,
        options,
        search,
        limits,
        move |problem: Problem, plan: &Plan, cancel: &Cancel| {
            let mode = problem.mode.unwrap_or(mode);
            let outcome =
                super::prove_within(&problem.sequent, mode, &plan.search, &plan.limits, |_| {
                    cancel.is_cancelled()
                });
            Answer {
                outcome,
                name: problem.name,
            }
        },
    )
}

/// Applies `work` to every problem with the options of its search, as the
/// plan says, and returns the results in the order of the problems, each
/// as soon as it and those before it are done. `Cores::Auto` reads up to
/// as many problems as there are workers before the first starts; a
/// stream whose next problem waits on an answer wants `Across` or
/// `Within`. `work` is handed the batch's [`Cancel`], which its stop asks.
/// A panic in `work` is resumed by the iterator at its problem's place,
/// after the results before it; the batch then takes no more problems
/// and is cancelled, so the other workers end. Workers that cannot
/// start (the stack the limits ask for is not to be had) leave the batch
/// to those that did, or to the caller's thread.
pub fn run<P, R>(
    problems: impl IntoIterator<Item = P, IntoIter: Send + 'static>,
    options: &Options,
    search: &Search,
    limits: &Limits,
    work: impl Fn(P, &Plan, &Cancel) -> R + Send + Sync + 'static,
) -> Results<R>
where
    P: Send + 'static,
    R: Send + 'static,
{
    let mut problems = problems.into_iter();
    let mut first = Vec::new();
    let within = match options.cores {
        Cores::Across => false,
        Cores::Within => true,
        Cores::Auto => {
            let workers = options.workers.clamp(1, Search::MAX_JOBS);
            first.extend(problems.by_ref().take(workers));
            first.len() < workers
        }
    };
    let problems: Box<dyn Iterator<Item = P> + Send> = Box::new(first.into_iter().chain(problems));
    let plan = options.plan(within, search, limits);
    let cancel = Cancel::default();
    #[cfg(feature = "parallel")]
    let (problems, work) = if plan.workers > 1 {
        match workers::Workers::start(problems, plan.clone(), cancel.clone(), work) {
            Ok(workers) => {
                return Results {
                    inner: Inner::Workers(workers),
                    cancel,
                };
            }
            Err(unstarted) => unstarted,
        }
    } else {
        (problems, work)
    };
    let canceller = cancel.clone();
    Results {
        inner: Inner::Here(Box::new(
            problems.map(move |problem| work(problem, &plan, &canceller)),
        )),
        cancel,
    }
}

/// Asks the work of a batch to end: [`Results::cancel`] raises it, and so
/// does dropping the results; the work's stop asks it, so a search in
/// flight ends as `Unknown(Reason::Stopped)`.
#[derive(Clone, Debug, Default)]
pub struct Cancel(Arc<AtomicBool>);

impl Cancel {
    /// Returns whether the batch was cancelled.
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }

    /// Cancels the batch.
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Relaxed);
    }
}

/// The results of a batch, in the order of its problems. Dropping them
/// cancels the work still running.
pub struct Results<R> {
    /// Where the work runs.
    inner: Inner<R>,
    /// What the work asks whether to end.
    cancel: Cancel,
}

impl<R> Results<R> {
    /// Cancels the work: every search in flight ends at its next poll, and
    /// the problems not yet begun are answered as their work answers a
    /// cancelled batch.
    pub fn cancel(&self) {
        self.cancel.cancel();
    }

    /// Returns a handle that cancels the work from elsewhere, such as a
    /// handler of an interruption.
    pub fn canceller(&self) -> Cancel {
        self.cancel.clone()
    }
}

impl<R> Drop for Results<R> {
    /// Cancels the work still running.
    fn drop(&mut self) {
        self.cancel.cancel();
    }
}

impl<R> std::fmt::Debug for Results<R> {
    /// Names the type: what it holds is the workers' state.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Results").finish_non_exhaustive()
    }
}

/// Where a batch runs.
enum Inner<R> {
    /// On the caller's thread, one problem per result asked for.
    Here(Box<dyn Iterator<Item = R> + Send>),
    /// On workers of its own.
    #[cfg(feature = "parallel")]
    Workers(workers::Workers<R>),
}

impl<R> Iterator for Results<R> {
    type Item = R;

    fn next(&mut self) -> Option<R> {
        match &mut self.inner {
            Inner::Here(results) => results.next(),
            #[cfg(feature = "parallel")]
            Inner::Workers(workers) => workers.next(),
        }
    }
}

/// The workers of a batch, on threads of their own.
#[cfg(feature = "parallel")]
mod workers {
    use super::{Cancel, Plan};
    use std::collections::BTreeMap;
    use std::panic::AssertUnwindSafe;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::mpsc::{Receiver, channel};
    use std::sync::{Arc, Condvar, Mutex};
    use std::thread::JoinHandle;
    use std::time::Duration;

    /// How many problems per worker may be taken beyond the first result
    /// not yet given out, so that the results held for their turn stay
    /// few while a slow problem is decided.
    const AHEAD: usize = 4;

    /// How long a worker waits for its turn before it looks again.
    const WAIT: Duration = Duration::from_millis(10);

    /// What the workers share.
    struct Shared<P> {
        /// The problems not yet taken, how many were taken, and whether
        /// they ended; locked only to take one.
        queue: Mutex<(Box<dyn Iterator<Item = P> + Send>, usize, bool)>,
        /// How many results were given out, which the iterator raises
        /// without the lock: a worker that waits on a stream for the next
        /// problem holds the lock, and the answer it waits for must not.
        given: AtomicUsize,
        /// Woken when a result is given out.
        turn: Condvar,
    }

    /// The workers and the results they sent before their turn.
    pub(super) struct Workers<R> {
        /// The results as they are done, with their places, or the panic
        /// of the work at that place.
        done: Receiver<(usize, std::thread::Result<R>)>,
        /// Results done before those ahead of them.
        held: BTreeMap<usize, std::thread::Result<R>>,
        /// The place of the next result to give out.
        next: usize,
        /// Raises `given` and wakes the workers.
        given: Box<dyn Fn(usize) + Send>,
        /// Ends the queue, so that no worker takes another problem.
        end: Box<dyn Fn() + Send>,
        /// The threads.
        threads: Vec<JoinHandle<()>>,
    }

    impl<R: Send + 'static> Workers<R> {
        /// Starts the workers the plan names on the problems, as many as
        /// can start, or gives the problems and the work back when none
        /// can.
        pub(super) fn start<P: Send + 'static, W>(
            problems: Box<dyn Iterator<Item = P> + Send>,
            plan: Plan,
            cancel: Cancel,
            work: W,
        ) -> Result<Self, (Box<dyn Iterator<Item = P> + Send>, W)>
        where
            W: Fn(P, &Plan, &Cancel) -> R + Send + Sync + 'static,
        {
            let shared = Arc::new(Shared {
                queue: Mutex::new((problems, 0, false)),
                given: AtomicUsize::new(0),
                turn: Condvar::new(),
            });
            let work = Arc::new(work);
            let (sender, done) = channel();
            let ahead = AHEAD * plan.workers;
            let mut threads = Vec::with_capacity(plan.workers);
            for _ in 0..plan.workers {
                let (shared, work, sender) = (shared.clone(), work.clone(), sender.clone());
                let (plan, cancel) = (plan.clone(), cancel.clone());
                let started = std::thread::Builder::new()
                    .name("batch".into())
                    .stack_size(plan.limits.stack_bytes())
                    .spawn(move || {
                        loop {
                            let (place, problem) = {
                                let mut queue = shared.queue.lock().expect("no panic holds it");
                                while !queue.2
                                    && queue.1 >= shared.given.load(Ordering::Acquire) + ahead
                                {
                                    queue = shared
                                        .turn
                                        .wait_timeout(queue, WAIT)
                                        .expect("no panic holds it")
                                        .0;
                                }
                                if queue.2 {
                                    return;
                                }
                                let Some(problem) = queue.0.next() else {
                                    queue.2 = true;
                                    return;
                                };
                                queue.1 += 1;
                                (queue.1 - 1, problem)
                            };
                            let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
                                work(problem, &plan, &cancel)
                            }));
                            let panicked = result.is_err();
                            if sender.send((place, result)).is_err() || panicked {
                                if panicked {
                                    // The batch ends at the panic: no worker
                                    // takes another problem, and the work in
                                    // flight is asked to stop. A worker that
                                    // holds the queue waits on a stream, and
                                    // ends at its next send.
                                    cancel.cancel();
                                    if let Ok(mut queue) = shared.queue.try_lock() {
                                        queue.2 = true;
                                    }
                                    shared.turn.notify_all();
                                }
                                return;
                            }
                        }
                    });
                match started {
                    Ok(thread) => threads.push(thread),
                    Err(_) => break,
                }
            }
            if threads.is_empty() {
                // Nothing else holds the queue or the work now.
                let queue = Arc::try_unwrap(shared).ok().map(|s| s.queue);
                let work = Arc::try_unwrap(work).ok();
                if let (Some(queue), Some(work)) = (queue, work) {
                    let (problems, ..) = queue.into_inner().unwrap_or_else(|e| e.into_inner());
                    return Err((problems, work));
                }
                unreachable!("no worker started, so none holds the queue or the work");
            }
            let (raise, end) = (shared.clone(), shared);
            Ok(Self {
                done,
                held: BTreeMap::new(),
                next: 0,
                given: Box::new(move |n| {
                    raise.given.store(n, Ordering::Release);
                    raise.turn.notify_all();
                }),
                end: Box::new(move || {
                    if let Ok(mut queue) = end.queue.try_lock() {
                        queue.2 = true;
                    }
                }),
                threads,
            })
        }
    }

    impl<R> Workers<R> {
        /// Returns the next result in order, waiting for it; resumes the
        /// panic of the work at its place, and at the end a worker's own.
        pub(super) fn next(&mut self) -> Option<R> {
            loop {
                if let Some(result) = self.held.remove(&self.next) {
                    self.next += 1;
                    (self.given)(self.next);
                    match result {
                        Ok(result) => return Some(result),
                        Err(panic) => std::panic::resume_unwind(panic),
                    }
                }
                match self.done.recv() {
                    Ok((place, result)) => {
                        self.held.insert(place, result);
                    }
                    Err(_) => {
                        for thread in self.threads.drain(..) {
                            if let Err(panic) = thread.join() {
                                std::panic::resume_unwind(panic);
                            }
                        }
                        return None;
                    }
                }
            }
        }
    }

    impl<R> Drop for Workers<R> {
        /// Lets the workers finish the problems they hold and take no more.
        fn drop(&mut self) {
            (self.end)();
        }
    }
}

#[cfg(all(test, feature = "parse"))]
mod tests {
    use super::*;
    use crate::search::{Jobs, Reason, Verdict};

    /// The answers come in the order of the problems, on one worker and on
    /// several, whatever order the workers finish in.
    #[test]
    fn answers_keep_the_problems_order() {
        let texts: Vec<String> = (0..40)
            .map(|i| match i % 3 {
                0 => format!("a{i} |- a{i}"),
                1 => format!("a{i} |- b{i}"),
                _ => format!("a{i}, a{i} -o b{i} |- b{i}"),
            })
            .collect();
        for (workers, cores) in [(1, Cores::Auto), (4, Cores::Across), (4, Cores::Auto)] {
            let problems: Vec<Problem> = texts
                .iter()
                .map(|t| Problem::new(t.clone(), t.parse().unwrap(), None))
                .collect();
            let options = Options {
                workers,
                cores,
                ..Options::default()
            };
            let search = Search::default();
            let answers: Vec<(String, bool)> =
                prove(problems, &options, &search, &Limits::default())
                    .map(|a| {
                        let proved = matches!(a.outcome.unwrap().verdict, Verdict::Proved(_));
                        (a.name, proved)
                    })
                    .collect();
            let expected: Vec<(String, bool)> = texts
                .iter()
                .enumerate()
                .map(|(i, t)| (t.clone(), i % 3 != 1))
                .collect();
            assert_eq!(answers, expected);
        }
    }

    /// Across the sequents, the batch's bound decides how many searches
    /// run at once at the bound of each; within, each of the two searches
    /// of a sequent holds at most half of it.
    #[test]
    fn the_batch_bound_shares_out_the_memory() {
        let options = Options {
            workers: 16,
            total_memory_bytes: Some(5 << 30),
            ..Options::default()
        };
        let search = Search::default().with_jobs(4);
        let limits = Limits::default().with_memory_bytes(Some(1 << 30));
        let across = options.plan(false, &search, &limits);
        assert_eq!(
            (across.workers, across.limits.memory_bytes),
            (5, Some(1 << 30))
        );
        assert_eq!(across.search.jobs, Jobs::Count(1));
        let within = Options {
            total_memory_bytes: Some(1 << 30),
            ..options.clone()
        }
        .plan(true, &search, &limits);
        assert_eq!(
            (within.workers, within.limits.memory_bytes),
            (1, Some(1 << 29))
        );
        // Without a bound the workers are what was asked, at most as many
        // as the options take.
        let unbounded = options.with_workers(100_000).with_total_memory_bytes(None);
        let plan = unbounded.plan(false, &search, &Limits::UNBOUNDED);
        assert_eq!(plan.workers, Search::MAX_JOBS);
    }

    /// Cancelled results end the searches in flight and those not begun,
    /// on one worker and on several.
    #[test]
    fn cancelled_results_stop_their_searches() {
        // 2⁴² splits that no count cuts under weakening, which no search
        // gets through.
        let literals: Vec<String> = (0..40).map(|i| format!("x{i}")).collect();
        let text = format!(
            "|- p * q, 0 * (~p par ~p), 0 * (~q par ~q), {}",
            literals.join(", ")
        );
        let sequent: Sequent = text.parse().unwrap();
        for workers in [1, 2] {
            let problems: Vec<Problem> = (0..4)
                .map(|i| Problem::new(i.to_string(), sequent.clone(), None))
                .collect();
            let options = Options::default()
                .with_mode(Mode::CLASSICAL.with_affine())
                .with_workers(workers)
                .with_cores(Cores::Across);
            let results = prove(problems, &options, &Search::default(), &Limits::default());
            results.cancel();
            for answer in results {
                let verdict = answer.outcome.unwrap().verdict;
                assert!(
                    matches!(verdict, Verdict::Unknown(Reason::Stopped)),
                    "{verdict:?}"
                );
            }
        }
    }

    /// Workers whose stack cannot be had leave the batch to the caller's
    /// thread: a recursion limit read from a file asks for terabytes of
    /// stack, which panicked a batch of two workers.
    #[test]
    fn workers_that_cannot_start_leave_the_batch_here() {
        let sequent: Sequent = "|- a, ~a".parse().unwrap();
        let problems: Vec<Problem> = (0..3)
            .map(|i| Problem::new(i.to_string(), sequent.clone(), None))
            .collect();
        let options = Options::default().with_workers(2).with_cores(Cores::Across);
        let limits = Limits::default().with_recursion_depth(u32::MAX);
        for answer in prove(problems, &options, &Search::default(), &limits) {
            let verdict = answer.outcome.unwrap().verdict;
            assert!(matches!(verdict, Verdict::Proved(_)), "{verdict:?}");
        }
        // Reading ahead takes no more problems than workers can run.
        let options = Options::default()
            .with_workers(usize::MAX)
            .with_cores(Cores::Auto);
        let endless =
            std::iter::repeat_with(move || Problem::new("p".to_owned(), sequent.clone(), None));
        let first = prove(endless, &options, &Search::default(), &Limits::default()).next();
        assert!(matches!(
            first.unwrap().outcome.unwrap().verdict,
            Verdict::Proved(_)
        ));
    }

    /// A panic in the work is resumed at its problem's place, after the
    /// results before it, however many problems follow it: the other
    /// workers stop taking problems, so the batch never waits for good.
    #[test]
    fn a_panic_ends_the_batch_at_its_place() {
        let (sender, received) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let options = Options::default().with_workers(2).with_cores(Cores::Across);
            let mut results = run(
                0..100usize,
                &options,
                &Search::default(),
                &Limits::default(),
                |i, _, _| {
                    assert_ne!(i, 5, "the work panics on problem 5");
                    i
                },
            );
            let before: Vec<usize> = results.by_ref().take(5).collect();
            let panicked =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| results.next())).is_err();
            sender.send((before, panicked)).unwrap();
        });
        let answer = received.recv_timeout(std::time::Duration::from_secs(20));
        assert_eq!(answer, Ok(((0..5).collect(), true)));
    }
}
