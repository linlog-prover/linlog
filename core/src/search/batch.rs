// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Many sequents decided in one call: problems in, results out, in the
//! order of the problems and each as soon as it and those before it are
//! decided, so that a stream of questions is answered one by one. The
//! machine's threads go across the sequents, one sequent per worker on
//! the sequential engines, or within one sequent at a time
//! ([`Cores`](crate::search::batch::Cores)); the memory bound of each search and the batch's own
//! bound decide how many workers run at once ([`Options::plan`](crate::search::batch::Options::plan)).
//! Without the `parallel` feature a batch runs on the caller's thread.
//!
//! # Examples
//!
#![cfg_attr(feature = "parse", doc = "```")]
#![cfg_attr(not(feature = "parse"), doc = "```ignore")]
//! use linlog::search::Verdict;
//! use linlog::search::batch::{Options, Problem, prove};
//!
//! let problems = ["A |- A", "A |- B"].map(|text| Problem {
//!     name: text.to_owned(),
//!     sequent: text.parse().unwrap(),
//!     mode: None,
//! });
//! let proved: Vec<bool> = prove(problems, &Options::default())
//!     .map(|answer| matches!(answer.outcome.unwrap().verdict, Verdict::Proved(_)))
//!     .collect();
//! assert_eq!(proved, [true, false]);
//! ```

use super::{Options as Search, Outcome};
use crate::{Error, Mode, Sequent};

/// How a batch spends the machine's threads.
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

/// The options of a batch: those of every sequent's search, and how the
/// batch shares the machine among the sequents.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Options {
    /// The options of each sequent's search; its memory limit is each
    /// search's bound.
    pub search: Search,
    /// The mode of a problem that names none.
    pub mode: Mode,
    /// Where the threads go.
    pub cores: Cores,
    /// The most sequents decided at once, across the sequents.
    pub workers: usize,
    /// The most memory all the searches of the batch may hold together,
    /// or `None` for no bound but each search's.
    pub memory_limit: Option<u64>,
}

impl Options {
    /// The default number of workers: one, since this crate does not ask
    /// the machine how many threads it runs; a front end sets it.
    pub const DEFAULT_WORKERS: usize = 1;
    /// The default bound of the whole batch, 4 GiB: four searches at the
    /// default bound of each.
    pub const DEFAULT_MEMORY_LIMIT: u64 = 4 << 30;

    /// Returns how the batch runs, `within` its sequents or across them:
    /// across, as many workers as the batch's bound holds searches at the
    /// bound of each (at least one, whose bound is then the batch's), each
    /// on one thread; within, one worker whose search may be two at once
    /// (a front end may race one thread against a pool of the others), so
    /// each holds at most half the batch's bound.
    pub fn plan(&self, within: bool) -> Plan {
        let mut search = self.search.clone();
        let total = self.memory_limit;
        if within {
            let share = total.map(|t| if search.jobs > 1 { t / 2 } else { t });
            search.memory_limit = smaller(search.memory_limit, share);
            return Plan { workers: 1, search };
        }
        let workers = match (search.memory_limit, total) {
            (Some(each), Some(total)) => usize::try_from(total / each.max(1)).unwrap_or(usize::MAX),
            (None, Some(total)) => {
                search.memory_limit = Some(total / self.workers.max(1) as u64);
                usize::MAX
            }
            (_, None) => usize::MAX,
        };
        search.memory_limit = smaller(search.memory_limit, total);
        Plan {
            workers: workers.clamp(1, self.workers.max(1)),
            search: search.jobs(1),
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
    /// The search's defaults, the classical mode,
    /// [`DEFAULT_WORKERS`](Self::DEFAULT_WORKERS) and
    /// [`DEFAULT_MEMORY_LIMIT`](Self::DEFAULT_MEMORY_LIMIT).
    fn default() -> Self {
        Self {
            search: Search::default(),
            mode: Mode::CLASSICAL,
            cores: Cores::Auto,
            workers: Self::DEFAULT_WORKERS,
            memory_limit: Some(Self::DEFAULT_MEMORY_LIMIT),
        }
    }
}

/// How a batch runs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Plan {
    /// The sequents decided at once.
    pub workers: usize,
    /// The options of each sequent's search.
    pub search: Search,
}

/// A problem of [`prove`].
#[derive(Clone, Debug)]
pub struct Problem {
    /// Its name, which its answer carries.
    pub name: String,
    /// The sequent.
    pub sequent: Sequent,
    /// Its mode, or `None` for the batch's.
    pub mode: Option<Mode>,
}

/// The answer to a problem of [`prove`].
#[derive(Debug)]
pub struct Answer {
    /// The problem's name.
    pub name: String,
    /// What the search returned.
    pub outcome: Result<Outcome, Error>,
}

/// Decides the problems, and returns their answers in order as they are
/// decided. Each search ends by the options' bounds; a front end that
/// needs a time limit or a stop of its own calls [`run`].
pub fn prove(
    problems: impl IntoIterator<Item = Problem, IntoIter: Send + 'static>,
    options: &Options,
) -> Results<Answer> {
    let mode = options.mode;
    run(problems, options, move |problem: Problem, search| Answer {
        outcome: super::prove(&problem.sequent, problem.mode.unwrap_or(mode), search),
        name: problem.name,
    })
}

/// Applies `work` to every problem with the options of its search, as the
/// plan says, and returns the results in the order of the problems, each
/// as soon as it and those before it are done. `Cores::Auto` reads up to
/// as many problems as there are workers before the first starts; a
/// stream whose next problem waits on an answer wants `Across` or
/// `Within`. A panic in `work` is resumed by the iterator.
pub fn run<P, R>(
    problems: impl IntoIterator<Item = P, IntoIter: Send + 'static>,
    options: &Options,
    work: impl Fn(P, &Search) -> R + Send + Sync + 'static,
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
            first.extend(problems.by_ref().take(options.workers.max(1)));
            first.len() < options.workers.max(1)
        }
    };
    let problems = first.into_iter().chain(problems);
    let plan = options.plan(within);
    #[cfg(feature = "parallel")]
    if plan.workers > 1 {
        return Results(Inner::Workers(workers::Workers::start(
            Box::new(problems),
            plan,
            work,
        )));
    }
    let search = plan.search;
    Results(Inner::Here(Box::new(
        problems.map(move |problem| work(problem, &search)),
    )))
}

/// The results of a batch, in the order of its problems.
pub struct Results<R>(Inner<R>);

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
        match &mut self.0 {
            Inner::Here(results) => results.next(),
            #[cfg(feature = "parallel")]
            Inner::Workers(workers) => workers.next(),
        }
    }
}

/// The workers of a batch, on threads of their own.
#[cfg(feature = "parallel")]
mod workers {
    use super::{Plan, Search};
    use std::collections::BTreeMap;
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
        /// The results as they are done, with their places.
        done: Receiver<(usize, R)>,
        /// Results done before those ahead of them.
        held: BTreeMap<usize, R>,
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
        /// Starts the workers the plan names on the problems.
        pub(super) fn start<P: Send + 'static>(
            problems: Box<dyn Iterator<Item = P> + Send>,
            plan: Plan,
            work: impl Fn(P, &Search) -> R + Send + Sync + 'static,
        ) -> Self {
            let shared = Arc::new(Shared {
                queue: Mutex::new((problems, 0, false)),
                given: AtomicUsize::new(0),
                turn: Condvar::new(),
            });
            let work = Arc::new(work);
            let (sender, done) = channel();
            let ahead = AHEAD * plan.workers;
            let threads = (0..plan.workers)
                .map(|_| {
                    let (shared, work, sender) = (shared.clone(), work.clone(), sender.clone());
                    let search = plan.search.clone();
                    std::thread::Builder::new()
                        .name("batch".into())
                        .stack_size(search.stack_size())
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
                                if sender.send((place, work(problem, &search))).is_err() {
                                    return;
                                }
                            }
                        })
                        .expect("a batch's worker starts")
                })
                .collect();
            let (raise, end) = (shared.clone(), shared);
            Self {
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
            }
        }
    }

    impl<R> Workers<R> {
        /// Returns the next result in order, waiting for it; at the end,
        /// resumes a worker's panic.
        pub(super) fn next(&mut self) -> Option<R> {
            loop {
                if let Some(result) = self.held.remove(&self.next) {
                    self.next += 1;
                    (self.given)(self.next);
                    return Some(result);
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
    use crate::search::Verdict;

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
                .map(|t| Problem {
                    name: t.clone(),
                    sequent: t.parse().unwrap(),
                    mode: None,
                })
                .collect();
            let options = Options {
                workers,
                cores,
                ..Options::default()
            };
            let answers: Vec<(String, bool)> = prove(problems, &options)
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
            search: Search::default().memory_limit(Some(1 << 30)).jobs(4),
            workers: 16,
            memory_limit: Some(5 << 30),
            ..Options::default()
        };
        let across = options.plan(false);
        assert_eq!(
            (across.workers, across.search.memory_limit),
            (5, Some(1 << 30))
        );
        assert_eq!(across.search.jobs, 1);
        let within = Options {
            memory_limit: Some(1 << 30),
            ..options.clone()
        }
        .plan(true);
        assert_eq!(
            (within.workers, within.search.memory_limit),
            (1, Some(1 << 29))
        );
    }
}
