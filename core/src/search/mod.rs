// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Proof search: the front door `prove` with its options and outcome, the
//! dispatch on fragment and mode, the engines, which [`Engine`] lists and
//! describes (proof-net search for unit-free MLL, the focused sequent
//! engine for everything else, one-sided in classical mode and two-sided
//! in intuitionistic mode, and the additive fast path for two
//! additive-only formulas), and the [`batch`] of
//! many sequents.

/// The additive fast path.
pub(crate) mod additive;
pub mod batch;
/// The focused sequent engine.
pub(crate) mod focus;
/// Random provable sequents for the tests.
#[cfg(all(test, feature = "parse"))]
pub(crate) mod generate;
/// The Horn engine: reachability of markings.
pub(crate) mod horn;
/// The count of the bytes a search holds.
pub(crate) mod memory;
/// The proof-net engine.
pub(crate) mod net;
#[cfg(feature = "parallel")]
mod parallel;
/// A reference prover for the tests: the plain unfocused calculus.
#[cfg(all(test, feature = "parse"))]
pub(crate) mod reference;
/// Why a sequent is unprovable, and the disproof that says so.
mod refutation;

#[cfg(feature = "parallel")]
pub use parallel::Pool;
pub use refutation::{Disproof, Equation, Refutation, StateEquation, Unbalanced};

use crate::Error;
use crate::fragment::{Fragment, Mode};
use crate::limits::{Limits, Phase, Progress, Refusal};
use crate::nets::ProofStructure;
use crate::occurrences::{Forest, Member, OccId, Reading};
use crate::proofs::{Bytes, CheckError, Node, NodeId, Proof};
use crate::sequents::{Atom, Sequent};
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::num::NonZeroUsize;
use std::str::FromStr;

/// The work a search has done, in its engine's units, shared by every
/// thread of it, and the bound on it ([`Limits::work`]). The calling
/// thread counts its own units apart and adds them at the end, and other
/// threads add theirs in batches of [`Work::BATCH`]: an atomic addition
/// at every poll cost the additive path, which polls at every pair, 5 % of
/// its instructions, and would contend on a pool.
#[derive(Debug)]
pub(crate) struct Work {
    /// The units done.
    done: std::sync::atomic::AtomicU64,
    /// The most units allowed; `u64::MAX` for no bound, which no search
    /// reaches (a unit takes a nanosecond at the least).
    limit: u64,
}

impl Work {
    /// How many units a thread other than the calling one counts before it
    /// adds them: what the total and the bound see of it is late by at
    /// most this much per thread.
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

    /// Returns whether the work passed its bound.
    pub(crate) fn passed(&self) -> bool {
        self.done() > self.limit
    }
}

/// An engine's stop condition: the caller's in a sequential search, the
/// chain of stop flags of a worker in a parallel one. Each poll is told
/// the units of work done since the last.
pub(crate) enum Stop<'a> {
    /// The caller's condition.
    Closure(&'a mut dyn FnMut(u64) -> bool),
    /// The caller's condition, and an amount of work after which the
    /// search stops as well: one turn of a search that takes turns with
    /// another.
    Turn(&'a mut dyn FnMut(u64) -> bool, u64),
    /// A condition that is told, at every poll, the work since the last
    /// and whether a slice of work (the last field) has passed since it
    /// was last told so: one of two searches that alternate, which gives
    /// way to the other there.
    #[cfg_attr(
        not(feature = "parallel"),
        expect(
            dead_code,
            reason = "only threads alternate the two searches in slices"
        )
    )]
    Slice(&'a mut dyn FnMut(u64, bool) -> bool, u64, u64),
    /// A worker's flags, and the units it has not yet added to the
    /// search's work.
    #[cfg(feature = "parallel")]
    Flags(parallel::Flags<'a>, u64),
}

impl Stop<'_> {
    /// Polls the condition after `work` more units of work, in the unit
    /// the engine that polls counts its turns in.
    pub(crate) fn fired(&mut self, work: u64) -> bool {
        match self {
            Self::Closure(stop) => stop(work),
            Self::Turn(stop, left) => {
                *left = left.saturating_sub(work);
                *left == 0 || stop(work)
            }
            Self::Slice(stop, left, slice) => {
                *left = left.saturating_sub(work);
                let passed = *left == 0;
                if passed {
                    *left = *slice;
                }
                stop(work, passed)
            }
            #[cfg(feature = "parallel")]
            Self::Flags(flags, pending) => flags.fired(work, pending),
        }
    }
}

#[cfg(feature = "parallel")]
impl Drop for Stop<'_> {
    /// Adds a worker's last units, fewer than a batch, to the search's
    /// work, which would not count them otherwise.
    fn drop(&mut self) {
        if let Self::Flags(flags, pending) = self {
            flags.settle(*pending);
        }
    }
}

/// The occurrences a forest must have for a search to poll the stop
/// condition between the passes that set it up: a pass over fewer takes
/// under a millisecond, and a condition that counts its polls then sees
/// the engine's own and no others.
const SET_UP_POLL: usize = 1 << 16;

/// Polls the caller's stop condition between two passes over a forest
/// that set a search up, when the forest is large enough for a pass to
/// take time ([`SET_UP_POLL`]): on millions of occurrences the set-up
/// takes a second, and the engine's first poll comes after it.
pub(crate) fn set_up_stopped(forest: &Forest, stop: &mut dyn FnMut(u64) -> bool) -> bool {
    forest.len() >= SET_UP_POLL && stop(0)
}

/// Decides a sequent under a mode with the engine its fragment calls for,
/// and returns the outcome: the verdict with a proof if there is one, the
/// fragment detected, the engine used and the statistics of the run. The
/// search runs to completion; [`prove_within`] takes a stop condition.
///
/// # Errors
///
/// A sequent outside the fragment the options assert is refused
/// ([`Error::FragmentMismatch`]); in intuitionistic mode a sequent with no
/// intuitionistic reading ([`Error::NotIntuitionistic`]) and Mix
/// ([`Error::IntuitionisticMix`]); an engine the options force where it
/// does not search ([`Error::EngineRefused`], with the [`NotTaken`] that
/// says why: the net engine outside unit-free MLL and in affine mode, the
/// focus engine in intuitionistic mode and the two-sided engine in
/// classical mode, the additive engine on anything but two additive-only
/// formulas, the Horn engine on anything but a Horn program); and a sequent
/// that unfolds to more subformula occurrences than
/// `limits.occurrences` allows ([`Refusal::Occurrences`]). A proof that the checker rejects is
/// [`Error::Rejected`], a defect and no verdict; one whose check would
/// hold more than `limits.memory_bytes` makes the verdict
/// [`Reason::Unchecked`], one that `limits.work` or the stop gave up
/// [`Reason::WorkLimit`] or [`Reason::Stopped`]: every proof returned has
/// passed the checker, unless [`Options::check`] says otherwise. On
/// several threads, [`Error::ThreadPool`] when they cannot start.
///
/// # Examples
///
#[cfg_attr(feature = "parse", doc = "```")]
#[cfg_attr(not(feature = "parse"), doc = "```ignore")]
/// use linlog::search::{Engine, Options, Verdict, prove};
/// use linlog::{Fragment, Mode, Sequent};
///
/// let sequent: Sequent = "A, A -o B |- B".parse()?;
/// let outcome = prove(&sequent, Mode::CLASSICAL, &Options::default())?;
/// assert_eq!(outcome.fragment, Fragment::MLL);
/// assert_eq!(outcome.engine, Engine::Net);
/// assert!(outcome.net.is_some(), "the net engine returns the net it found");
/// let Verdict::Proved(proof) = outcome.verdict else {
///     panic!("provable");
/// };
/// assert_eq!(
///     proof.derivation()?.to_string(),
///     "─────── ax   ─────── ax\n\
///      ⊢ ~A, A      ⊢ ~B, B\n\
///      ──────────────────── ⊗\n\
///     \x20 ⊢ ~A, A ⊗ ~B, B"
/// );
///
/// let sequent: Sequent = "|- A par B, ~A, ~B".parse()?;
/// let outcome = prove(&sequent, Mode::CLASSICAL, &Options::default())?;
/// assert!(matches!(outcome.verdict, Verdict::Unprovable(_)));
/// let outcome = prove(&sequent, Mode::CLASSICAL.with_mix(), &Options::default())?;
/// assert!(matches!(outcome.verdict, Verdict::Proved(_)));
/// # Ok::<(), linlog::Error>(())
/// ```
pub fn prove(sequent: &Sequent, mode: Mode, options: &Options) -> Result<Outcome, Error> {
    prove_within(sequent, mode, options, &Limits::default(), |_| false)
}

/// Decides a sequent as [`prove`] does, within `limits` (its forest within
/// `limits.occurrences`, the search's structures and the check of its
/// proof within `limits.memory_bytes`, its recursion within
/// `limits.recursion_depth`), polling `stop` and giving up with
/// [`Reason::Stopped`] once it returns true. The condition is the
/// caller's: a deadline on a clock the caller has, a flag an interrupt
/// handler sets; this crate has no clock of its own. The engines poll at
/// every stable sequent or literal chosen and inside every loop that can
/// run long between two of them, and on a forest of tens of thousands of
/// occurrences also between the passes that set the search up; the check
/// of a proof found polls every 4 096 nodes. What is not polled is the
/// building of the forest before the search and single passes over the
/// forest, each linear in it. A condition that is cheap to ask is asked
/// often enough: one that reads a clock only every so many polls is late
/// by that many polls, and on a large sequent a poll can be many
/// milliseconds from the last.
///
/// # Errors
///
/// Those of [`prove`], under `limits`.
///
/// # Examples
///
#[cfg_attr(feature = "parse", doc = "```")]
#[cfg_attr(not(feature = "parse"), doc = "```ignore")]
/// use linlog::search::{Options, Reason, Verdict, prove_within};
/// use linlog::{Limits, Mode, Sequent};
/// use std::time::{Duration, Instant};
///
/// let sequent: Sequent = "|- (a & b) + (a & c), ~a par (~b & ~c)".parse()?;
/// let deadline = Instant::now() + Duration::from_secs(10);
/// let limits = Limits::default();
/// let outcome = prove_within(&sequent, Mode::CLASSICAL, &Options::default(), &limits, |_| {
///     Instant::now() >= deadline
/// })?;
/// assert!(!matches!(outcome.verdict, Verdict::Unknown(Reason::Stopped)));
/// # Ok::<(), linlog::Error>(())
/// ```
pub fn prove_within(
    sequent: &Sequent,
    mode: Mode,
    options: &Options,
    limits: &Limits,
    stop: impl FnMut(Progress) -> bool,
) -> Result<Outcome, Error> {
    let forest = Forest::within(sequent, limits)?;
    prove_goal(Goal::conclusion(&forest), mode, options, limits, stop)
}

/// Members of a forest that a search decides: its conclusion, the roots,
/// or a goal a session left open, any multiset of its occurrences. The
/// roots in any order are the conclusion.
#[derive(Clone, Copy, Debug)]
pub struct Goal<'a> {
    /// The forest.
    forest: &'a Forest,
    /// The members, as given; empty for the conclusion.
    members: &'a [Member],
    /// Whether the members are the roots.
    conclusion: bool,
}

impl<'a> Goal<'a> {
    /// The forest's conclusion: the sequent itself.
    pub fn conclusion(forest: &'a Forest) -> Self {
        Self {
            forest,
            members: &[],
            conclusion: true,
        }
    }

    /// The goal of the members given, in any order and with repeats.
    ///
    /// # Errors
    ///
    /// [`Error::IndexOutOfBounds`] for a member outside the forest.
    pub fn new(forest: &'a Forest, members: &'a [Member]) -> Result<Self, Error> {
        if let Some(m) = members.iter().find(|m| m.index() >= forest.len()) {
            return Err(Error::IndexOutOfBounds {
                space: crate::limits::Space::Occurrence,
                index: m.index(),
                len: forest.len(),
            });
        }
        let occurrences: Vec<OccId> = members.iter().map(|m| m.occ()).collect();
        Ok(Self {
            forest,
            members,
            conclusion: forest.is_roots(&occurrences),
        })
    }

    /// Returns whether the goal is the forest's conclusion.
    pub fn is_conclusion(&self) -> bool {
        self.conclusion
    }

    /// Returns the forest.
    pub fn forest(&self) -> &'a Forest {
        self.forest
    }

    /// Returns the goal's occurrences: the roots in the forest's order
    /// for the conclusion, the members as given otherwise.
    fn occurrences(&self) -> std::borrow::Cow<'a, [OccId]> {
        if self.conclusion {
            std::borrow::Cow::Borrowed(self.forest.roots())
        } else {
            std::borrow::Cow::Owned(self.members.iter().map(|m| m.occ()).collect())
        }
    }
}

/// Decides a [`Goal`]: a multiset of occurrences of a forest, given in
/// any order, which stands for the sequent of those subformulas, within
/// `limits` and until `stop` fires, as [`prove_within`] does. The roots are
/// the sequent itself, and [`prove_within`] is this function on them; any
/// other goal is what an interactive proof leaves open, and the engine that
/// decides it is the one its own fragment calls for, except that the net
/// engine works on the roots only. The proof of a goal other than the
/// roots is a [`Proof`] whose root concludes the goal, which it records
/// ([`Proof::goal`]) and [`Proof::check`] checks it against; it is meant
/// to be grafted onto the goal, as the interactive state does, and is
/// refused where a proof of the sequent is needed ([`Error::GoalProof`]).
/// Every proof records the mode it was found in ([`Proof::mode`]).
///
/// # Errors
///
/// Those of [`prove`], plus [`Error::GoalOutputs`] for an
/// intuitionistic goal without exactly one formula on the right of `⊢`,
/// and [`Error::EngineRefused`] with [`NotTaken::Goal`] for the net engine
/// forced on a goal other than the roots.
///
/// # Examples
///
#[cfg_attr(feature = "parse", doc = "```")]
#[cfg_attr(not(feature = "parse"), doc = "```ignore")]
/// use linlog::search::{Goal, Options, Verdict, prove_goal};
/// use linlog::{Forest, Limits, Member, Mode, Sequent};
///
/// // ⊢ ~A, A ⊗ ~B, B, with the occurrences 0: ~A, 1: A ⊗ ~B, 2: A,
/// // 3: ~B, 4: B. The goal ⊢ ~A, A is the left premise of the ⊗.
/// let sequent: Sequent = "A, A -o B |- B".parse()?;
/// let forest = Forest::new(&sequent)?;
/// let members = [Member::new(0), Member::new(2)];
/// let goal = Goal::new(&forest, &members)?;
/// let limits = Limits::default();
/// let outcome = prove_goal(goal, Mode::CLASSICAL, &Options::default(), &limits, |_| false)?;
/// assert!(matches!(outcome.verdict, Verdict::Proved(_)));
/// # Ok::<(), linlog::Error>(())
/// ```
pub fn prove_goal(
    goal: Goal<'_>,
    mode: Mode,
    options: &Options,
    limits: &Limits,
    mut stop: impl FnMut(Progress) -> bool,
) -> Result<Outcome, Error> {
    let account = memory::Account::new(limits.memory_bytes);
    let work = Work::new(limits.work);
    decide_goal(goal, mode, options, limits, &account, &work, &mut stop)
}

/// Decides a goal as [`prove_goal`] does, on at most `threads` threads,
/// the calling one counted. From three: one thread first and, once
/// `add_pool` says so at one of its polls, a pool of the other
/// `threads − 1` beside it, which starts the search afresh; the first to
/// decide answers and stops the other, and both draw on one memory bound
/// and one bound on work. The single search goes on beside the pool, since
/// a pool may search worse than one thread. With two threads a pool of two
/// from the start, with one the calling thread alone, and so on an engine
/// that searches on one thread ([`Engine::parallel`]); `add_pool` is then
/// never asked. The single search runs on the calling thread, whose stack
/// must hold the recursion ([`Limits::stack_bytes`]). `add_pool` is asked
/// with the single search's progress, so a front end decides with its own
/// clock; `stop` is asked by both searches, from two threads. The outcome
/// is the deciding search's, with both searches' counters
/// ([`Statistics::add`]).
///
/// Needs the cargo feature `parallel` (off by default).
///
/// # Errors
///
/// Those of [`prove_goal`], and [`Error::ThreadPool`] when the pool's
/// threads cannot start.
#[cfg(feature = "parallel")]
pub fn race(
    goal: Goal<'_>,
    mode: Mode,
    options: &Options,
    limits: &Limits,
    threads: usize,
    mut add_pool: impl FnMut(Progress) -> bool,
    stop: impl Fn(Progress) -> bool + Sync,
) -> Result<Outcome, Error> {
    use std::sync::atomic::{AtomicBool, Ordering};
    let threads = threads.clamp(1, Options::MAX_JOBS);
    let whole = std::sync::Arc::new(memory::Account::new(limits.memory_bytes));
    let work = Work::new(limits.work);
    if threads < 3 || !engine_for(goal, mode, options)?.parallel() {
        let options = options.clone().with_jobs(threads);
        let account = memory::Account::part_of(&whole);
        return decide_goal(goal, mode, &options, limits, &account, &work, &mut |p| {
            stop(p)
        });
    }
    let decided = AtomicBool::new(false);
    let settled = |outcome: &Result<Outcome, Error>| matches!(outcome, Ok(o) if !matches!(o.verdict, Verdict::Unknown(_)));
    let (single, pooled) = (
        options.clone().with_jobs(1),
        options.clone().with_jobs(threads - 1),
    );
    std::thread::scope(|scope| {
        let (whole, work, stop, decided) = (&whole, &work, &stop, &decided);
        let pooled = &pooled;
        let mut pool = None;
        let account = memory::Account::part_of(whole);
        let first = decide_goal(
            goal,
            mode,
            &single,
            limits,
            &account,
            work,
            &mut |progress| {
                if pool.is_none() && add_pool(progress) {
                    // A pool whose thread cannot start leaves the single
                    // search alone.
                    pool = Some(
                        std::thread::Builder::new()
                            .name("linlog-race".into())
                            .stack_size(limits.stack_bytes())
                            .spawn_scoped(scope, move || {
                                let account = memory::Account::part_of(whole);
                                let outcome = decide_goal(
                                    goal,
                                    mode,
                                    pooled,
                                    limits,
                                    &account,
                                    work,
                                    &mut |p| stop(p) || decided.load(Ordering::Relaxed),
                                );
                                if settled(&outcome) {
                                    decided.store(true, Ordering::Relaxed);
                                }
                                outcome
                            })
                            .ok(),
                    );
                }
                stop(progress) || decided.load(Ordering::Relaxed)
            },
        );
        if settled(&first) {
            decided.store(true, Ordering::Relaxed);
        }
        let Some(Some(handle)) = pool else {
            return first;
        };
        let second = handle
            .join()
            .unwrap_or_else(|panic| std::panic::resume_unwind(panic));
        let (mut outcome, other) = if settled(&first) {
            (first?, second?)
        } else {
            (second?, first?)
        };
        outcome.statistics.add(&other.statistics);
        outcome.statistics.work = work.done();
        Ok(outcome)
    })
}

/// Decides a goal as [`prove_goal`] does, charging what the search holds
/// to `account` and counting its work in `work`, which a race's two
/// searches share.
fn decide_goal(
    goal: Goal<'_>,
    mode: Mode,
    options: &Options,
    limits: &Limits,
    account: &memory::Account,
    work: &Work,
    stop: &mut impl FnMut(Progress) -> bool,
) -> Result<Outcome, Error> {
    let forest = goal.forest;
    let occurrences = goal.occurrences();
    let fragment = fragment_of(forest, &occurrences, options)?;
    let reading = read(forest, mode)?;
    let (task, engine) = prepare(
        goal,
        &occurrences,
        mode,
        fragment,
        options,
        reading.as_ref(),
    )?;
    let implementation = engine.implementation();
    // Every poll on this thread counts its units of work, and tells the
    // caller's stop those and the units of every thread of the search;
    // past the bound on work the search ends.
    let mut mine = 0u64;
    let mut polled = |units: u64| {
        mine += units;
        let done = work.done().saturating_add(mine);
        done > work.limit || stop(Progress::new(Phase::Search, units, done))
    };
    // The fragment, the reading and the dispatch were passes over the
    // forest: the caller's condition is asked before the engine's own.
    if set_up_stopped(forest, &mut polled) {
        return Ok(Outcome {
            verdict: Verdict::Unknown(Reason::Stopped),
            fragment,
            mode,
            engine,
            statistics: Statistics::default(),
            net: None,
            checked: false,
        });
    }
    // What the search allocates is counted against the bound.
    #[cfg(feature = "parallel")]
    let options = &options
        .clone()
        .with_jobs(parallel::threads(options.threads()));
    let answer = implementation.decide(&task, options, limits, account, work, &mut polled);
    work.add(mine);
    let mut answer = answer?;
    answer.statistics.work = work.done();
    if work.passed() && matches!(answer.result, Err(Reason::Stopped)) {
        answer.result = Err(Reason::WorkLimit { limit: work.limit });
    }
    let verdict = conclude(
        answer.result,
        answer.refutation,
        &task,
        goal.members,
        options,
        limits,
        stop,
    )?;
    let checked = options.check && matches!(verdict, Verdict::Proved(_));
    Ok(Outcome {
        verdict,
        fragment,
        mode,
        engine,
        statistics: answer.statistics,
        net: answer.net,
        checked,
    })
}

/// Returns the verdict of an engine's answer, the one place it is made:
/// a proof that passed the checker and records the goal it concludes and
/// the mode, an exhausted search with the engine's refutation or the
/// counts', or why the search stopped. No engine is trusted with its own
/// proof: the checker has the last word on every proof, of the sequent or
/// of a goal, in every build, and a check that a bound or the stop gave up
/// is no verdict on the proof, so no proof is returned unchecked.
///
/// # Errors
///
/// [`Error::Rejected`] for a proof the checker rejects, a defect.
fn conclude(
    result: Result<Option<Proof>, Reason>,
    refutation: Option<Refutation>,
    task: &Task<'_>,
    members: &[Member],
    options: &Options,
    limits: &Limits,
    stop: &mut dyn FnMut(Progress) -> bool,
) -> Result<Verdict, Error> {
    let (forest, mode) = (task.forest, task.mode);
    let verdict = match result {
        Ok(Some(proof)) if task.roots => Verdict::Proved(Box::new(proof.with_mode(mode))),
        Ok(Some(proof)) => Verdict::Proved(Box::new(proof.concluding(task.goal).with_mode(mode))),
        Ok(None) => {
            // What the counts of the goal rule out, under the same limits
            // as the search, where the engine has no refutation of its own.
            let refutation = refutation.unwrap_or_else(|| {
                let account = memory::Account::new(limits.memory_bytes);
                let mut polled = |_| stop(Progress::new(Phase::Refute, 0, 0));
                focus::counts::refutation(
                    forest,
                    task.goal,
                    task.fragment,
                    mode,
                    &account,
                    &mut polled,
                )
            });
            let disproof = Disproof::new(forest.sequent().clone(), mode, refutation);
            Verdict::Unprovable(Box::new(if task.roots {
                disproof
            } else {
                disproof.of_goal(members.into())
            }))
        }
        Err(reason) => Verdict::Unknown(reason),
    };
    let Verdict::Proved(proof) = &verdict else {
        return Ok(verdict);
    };
    if !options.check {
        debug_assert_eq!(proof.check(mode), Ok(()), "an engine's proof");
        return Ok(verdict);
    }
    match proof.check_within(mode, limits, stop) {
        Ok(()) => Ok(verdict),
        Err(e @ CheckError::Invalid(_)) => Err(Error::Rejected(Box::new(e))),
        Err(CheckError::Refused(refused)) => Ok(Verdict::Unknown(unchecked(refused.refusal)?)),
    }
}

/// Returns why a search whose proof's check was given up is unknown: the
/// stop, the work bound, or the memory bound the check would have passed;
/// a refusal no check makes stays the error it is.
fn unchecked(refusal: Refusal) -> Result<Reason, Error> {
    match refusal {
        Refusal::Stopped { .. } => Ok(Reason::Stopped),
        Refusal::Memory { limit_bytes, .. } => Ok(Reason::Unchecked { limit_bytes }),
        Refusal::Work { limit } => Ok(Reason::WorkLimit { limit }),
        refusal => Err(Error::Refused(refusal)),
    }
}

/// Returns the engine [`prove_goal`] runs on a goal under the options:
/// the one they force, or the one the dispatch picks, or the error the
/// search would answer before it starts. It costs what `prove_goal` does
/// before it searches: a pass over the goal for its fragment, in
/// intuitionistic mode the reading of the forest, and the dispatch's
/// test of the goal's shape. A front end that runs a search on one thread
/// first and adds a pool when it takes long asks this before it adds one,
/// since an engine that runs on one thread
/// ([`Engine::parallel`]) would only search again.
///
/// # Errors
///
/// Those of [`prove_goal`] that come before the search.
pub fn engine_for(goal: Goal<'_>, mode: Mode, options: &Options) -> Result<Engine, Error> {
    let occurrences = goal.occurrences();
    let fragment = fragment_of(goal.forest, &occurrences, options)?;
    let reading = read(goal.forest, mode)?;
    prepare(
        goal,
        &occurrences,
        mode,
        fragment,
        options,
        reading.as_ref(),
    )
    .map(|(_, engine)| engine)
}

/// The fragment a goal is searched in, the options' or its own.
fn fragment_of(forest: &Forest, goal: &[OccId], options: &Options) -> Result<Fragment, Error> {
    let detected = goal_fragment(forest, goal);
    match options.fragment {
        Some(asserted) if !asserted.contains(detected) => {
            Err(Error::FragmentMismatch { asserted, detected })
        }
        Some(asserted) => Ok(asserted),
        None => Ok(detected),
    }
}

/// The forest's intuitionistic reading in intuitionistic mode, which has
/// no Mix.
fn read(forest: &Forest, mode: Mode) -> Result<Option<Reading<'_>>, Error> {
    // Mix has no intuitionistic form.
    if mode.intuitionistic && mode.mix {
        return Err(Error::IntuitionisticMix);
    }
    if !mode.intuitionistic {
        return Ok(None);
    }
    Reading::new(forest)
        .map(Some)
        .map_err(Error::NotIntuitionistic)
}

/// The task of a goal searched in `fragment` and the engine that decides
/// it, which admits it; in intuitionistic mode the goal has one output
/// under the reading given.
fn prepare<'a>(
    goal: Goal<'a>,
    occurrences: &'a [OccId],
    mode: Mode,
    fragment: Fragment,
    options: &Options,
    reading: Option<&'a Reading<'a>>,
) -> Result<(Task<'a>, Engine), Error> {
    if let Some(reading) = reading {
        let outputs = reading.outputs(occurrences.iter().copied());
        if outputs != 1 {
            return Err(Error::GoalOutputs { count: outputs });
        }
    }
    // The roots in any order are the sequent itself, which the engines
    // are handed in the forest's order.
    let task = Task {
        forest: goal.forest,
        goal: occurrences,
        fragment,
        mode,
        reading,
        roots: goal.conclusion,
    };
    let engine = match options.engine {
        Some(engine) => engine,
        None => dispatch(&task)?,
    };
    engine.implementation().admits(&task)?;
    Ok((task, engine))
}

/// A goal as the engines are handed it: the occurrences of the forest it
/// consists of, the fragment and the mode it is searched in, and in
/// intuitionistic mode the forest's reading.
pub(crate) struct Task<'a> {
    /// The forest.
    pub(crate) forest: &'a Forest,
    /// The goal's occurrences; the roots in the forest's order when the
    /// goal is the sequent itself.
    pub(crate) goal: &'a [OccId],
    /// The fragment searched in: the goal's own, or the one the options
    /// assert.
    pub(crate) fragment: Fragment,
    /// The mode.
    pub(crate) mode: Mode,
    /// The intuitionistic reading, in intuitionistic mode.
    pub(crate) reading: Option<&'a Reading<'a>>,
    /// Whether the goal is the sequent's roots.
    pub(crate) roots: bool,
}

/// What a search that keeps its proofs in an arena ended with: the node
/// proving the goal, `None` when the search was exhaustive, or the reason
/// it stopped; the arena; and the counters.
pub(crate) struct Finished {
    /// The node proving the goal, `None` for an unprovable goal, or why
    /// the search stopped.
    pub(crate) result: Result<Option<NodeId>, Reason>,
    /// The arena the node lives in.
    pub(crate) nodes: Vec<Node>,
    /// The counters.
    pub(crate) statistics: Statistics,
}

impl Finished {
    /// A search that gave up for `reason` before it found anything.
    pub(crate) fn gave_up(reason: Reason) -> Self {
        Self {
            result: Err(reason),
            nodes: Vec::new(),
            statistics: Statistics::default(),
        }
    }
}

/// What an engine's search ended with: a proof of the goal, `None` when
/// the search was exhaustive, or the reason it stopped; its counters; and
/// the proof net the net engine read its proof off.
pub(crate) struct Answer {
    /// The proof, `None` for an unprovable goal, or why the search
    /// stopped.
    pub(crate) result: Result<Option<Proof>, Reason>,
    /// The counters.
    pub(crate) statistics: Statistics,
    /// The net found, from the net engine.
    pub(crate) net: Option<ProofStructure>,
    /// For an unprovable goal, why, where the engine knows more than that
    /// its search was exhaustive.
    pub(crate) refutation: Option<Refutation>,
}

impl Answer {
    /// The answer of an engine that keeps its proofs as nodes of an arena:
    /// the result as a node, the arena and the counters.
    pub(crate) fn of_arena(
        forest: &Forest,
        Finished {
            result,
            nodes,
            statistics,
        }: Finished,
    ) -> Self {
        let result = result.map(|root| {
            root.map(|root| {
                Proof::new(forest.clone(), nodes, root)
                    .expect("the engine pushes premises before conclusions")
            })
        });
        Self {
            result,
            statistics,
            net: None,
            refutation: None,
        }
    }
}

/// What every engine implements: which goals it takes, and how it decides
/// one. An engine reads the options its variant of [`Engine`] names and
/// no others.
pub(crate) trait Decide {
    /// Refuses a goal the engine does not decide, with the error the
    /// options get when they force it there.
    fn admits(&self, task: &Task<'_>) -> Result<(), Error>;

    /// Decides the goal under the options, polling `stop` with the units
    /// of work done since the last poll (which `work` adds up for every
    /// thread of the search), and charges what the search holds to
    /// `account`.
    ///
    /// # Errors
    ///
    /// [`Error::ThreadPool`] when the threads of a parallel search cannot
    /// start.
    fn decide(
        &self,
        task: &Task<'_>,
        options: &Options,
        limits: &Limits,
        account: &memory::Account,
        work: &Work,
        stop: &mut dyn FnMut(u64) -> bool,
    ) -> Result<Answer, Error>;
}

/// The engine the dispatch picks for a goal: that of the first row of
/// [`DISPATCH`] that takes it, or [`Error::NoEngine`] when none does,
/// which a mode or a fragment no row names would meet.
fn dispatch(task: &Task<'_>) -> Result<Engine, Error> {
    DISPATCH
        .iter()
        .find(|row| row.takes(task))
        .map(|row| row.engine)
        .ok_or(Error::NoEngine {
            fragment: task.fragment,
            mode: task.mode,
        })
}

/// The dispatch, read from the first row down: for each fragment, mode and
/// feature of a goal, the engine a measurement shows fastest there, as the
/// documentation of [`Engine`] tabulates with the measurements. The last
/// two rows take every goal, each in its own modes.
const DISPATCH: [Row; 5] = [
    Row {
        fragment: Fragment::ADDITIVE,
        modes: Modes::Any,
        feature: Feature::TwoFormulas,
        engine: Engine::Additive,
    },
    Row {
        fragment: Fragment::MLL,
        modes: Modes::Linear,
        feature: Feature::FewEqualLiterals,
        engine: Engine::Net,
    },
    Row {
        fragment: Fragment::MELL,
        modes: Modes::Any,
        feature: Feature::PetriNet,
        engine: Engine::Horn,
    },
    Row {
        fragment: Fragment::LL,
        modes: Modes::Intuitionistic,
        feature: Feature::Any,
        engine: Engine::TwoSided,
    },
    Row {
        fragment: Fragment::LL,
        modes: Modes::Classical,
        feature: Feature::Any,
        engine: Engine::Focus,
    },
];

/// A row of the dispatch: a goal whose fragment lies within `fragment`,
/// searched in one of `modes`, with `feature`, goes to `engine`.
#[derive(Clone, Copy, Debug)]
struct Row {
    /// The largest fragment the row takes.
    fragment: Fragment,
    /// The modes it takes.
    modes: Modes,
    /// What it asks of a goal besides.
    feature: Feature,
    /// The engine its goals go to.
    engine: Engine,
}

impl Row {
    /// Whether the row takes the goal.
    fn takes(&self, task: &Task<'_>) -> bool {
        self.fragment.contains(task.fragment) && self.modes.take(task.mode) && self.feature.of(task)
    }
}

/// The modes a row of the dispatch takes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Modes {
    /// Every mode.
    Any,
    /// Every mode without weakening.
    Linear,
    /// Classical mode, with or without weakening and Mix.
    Classical,
    /// Intuitionistic mode, with or without weakening.
    Intuitionistic,
}

impl Modes {
    /// Whether a mode is one of these.
    fn take(self, mode: Mode) -> bool {
        #[expect(
            clippy::unneeded_field_pattern,
            reason = "every field named, so that a new one must be placed here"
        )]
        let Mode {
            intuitionistic,
            affine,
            mix: _,
        } = mode;
        match self {
            Modes::Any => true,
            Modes::Linear => !affine,
            Modes::Classical => !intuitionistic,
            Modes::Intuitionistic => intuitionistic,
        }
    }
}

/// What a row of the dispatch asks of a goal beyond its fragment and mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Feature {
    /// Nothing more.
    Any,
    /// Exactly two formulas, with an additive connective or unit among
    /// them (two literals are no additive problem).
    TwoFormulas,
    /// The sequent itself, with no literal occurring more than
    /// [`NET_MULTIPLICITY`] times.
    FewEqualLiterals,
    /// A Horn program with a clause under `!`: a Petri net with a marking
    /// to reach.
    PetriNet,
}

impl Feature {
    /// Whether the goal has the feature.
    fn of(self, task: &Task<'_>) -> bool {
        match self {
            Feature::Any => true,
            Feature::TwoFormulas => task.goal.len() == 2 && !task.fragment.is_empty(),
            Feature::FewEqualLiterals => task.roots && few_equal_literals(task.forest),
            // The fragment, which the options may assert larger than the
            // goal's, only spares a non-Horn goal the reading.
            Feature::PetriNet => task.fragment.has_exponentials() && horn::is_net(task),
        }
    }
}

/// Returns the smallest fragment the goal's subformulas live in: the
/// sequent's own fragment for the roots, and possibly a smaller one for an
/// open goal deeper in the forest.
pub(crate) fn goal_fragment(forest: &Forest, goal: &[OccId]) -> Fragment {
    let mut fragment = Fragment::EMPTY;
    for &o in goal {
        for x in forest.subtree(o) {
            fragment |= forest.kind(x).fragment();
        }
    }
    fragment
}

/// The most occurrences of one literal, `a` or `~a`, a sequent may have for
/// the net engine to be the default on it.
const NET_MULTIPLICITY: usize = 2;

/// Whether no literal of the sequent occurs more than [`NET_MULTIPLICITY`]
/// times. Equal literals under one connective are interchangeable
/// partners, so the linking search explores every permutation of a wrong
/// choice before a cycle shows, and the focused engine, whose count prunes
/// see the mistake at once, wins by orders of magnitude on such sequents;
/// with distinct atoms the linking is nearly forced.
fn few_equal_literals(forest: &Forest) -> bool {
    use crate::occurrences::Sign;
    let atoms = forest.sequent().atom_names().len() as u32;
    (0..atoms).all(|a| {
        let atom = Atom::new(a);
        forest.literals(atom, Sign::Atom).len() <= NET_MULTIPLICITY
            && forest.literals(atom, Sign::Dual).len() <= NET_MULTIPLICITY
    })
}

/// The engines, by which an outcome names the one that ran and the options
/// force one.
///
/// # Which engine decides a goal
///
/// Unless [`Options::engine`] forces one, a goal goes to the engine of the
/// first row it fits, by the fragment it lies in (detected, or asserted by
/// [`Options::fragment`]), its mode and one more feature; each row is the
/// engine that measured fastest there, or the one that decides there at
/// all:
///
/// | fragment | mode | feature | engine | measured (one thread) |
/// |---|---|---|---|---|
/// | additives only | any | two formulas, an additive connective or unit among them | [`Additive`](Engine::Additive) | `A ⊢ A` for `A` a complete tree of `&` and `⊕`: depth 8 in 0.5 ms against 1.4 ms on the focused engine, depth 14 in 0.1 to 0.2 s against 2.7 s, depth 16 in 0.3 s against over 20 s, in either mode |
/// | unit-free MLL | linear, classical or intuitionistic | the sequent itself, no literal more than twice | [`Net`](Engine::Net) | as fast as the focused engines up to three times slower (`wide` sequents of 8 to 1 024 literals; intuitionistic wide and curried sequents of 1 024 and 4 096 atoms 7 to 18 times slower), but the only engine that decides a long chain within the default recursion limit: `wide` with 2 048 literals in 0.64 s, a chain of 1 024 implications `a₀, a₀ ⊸ a₁, … ⊢ a₁₀₂₄` in 71 ms, where the focused engines recurse once per link |
/// | MELL | any, classical or intuitionistic, with or without Mix | a Horn program with a clause under `!`: a Petri net | [`Horn`](Engine::Horn) | in affine mode, the 176 coverability problems of the qcover suite at 5 s, intuitionistic: decides 162 (59 provable, 103 not, 155 within a second) where the default before it, the two-sided engine, decides 8, no verdict against another and every proof checked; 10 464 random affine Horn programs of one to four atoms and clauses, classical and intuitionistic: decides all, at most 0.2 ms each, where the focused engines leave 886 undecided at 1 s and are slower by more than a millisecond on 467. In linear mode, the library's 3 137 Petri nets at 5 s, intuitionistic, against the forward focused search (the factor bias within 30 copies, the better of the focused engine's two searches there): decides 3 026 nets against 1 628 (1 400 only by the Horn engine, 2 only by the forward search, no verdict against the other), in 0.23 ms against 1.2 ms in the median of the 1 626 both decide, faster on 1 013 of them; 2 670 within 10 ms against 1 103; the counter with 64 tokens proved in 0.07 ms and with the unreachable goal refuted in 0.6 ms, where the focused engine reaches its limit after 10 s; with the state equation, the dead transitions dropped and the backward search beside it, 3 071 of the nets; and of 7 000 random programs near the Horn shape it decides 4 303 of the 4 318 each linear mode sends it at 1 s, every one the focused engines decide and 548 to 580 more. Horn programs without `!` stay with the focused engines, whose counts decide the Partition encodings up to a hundred times faster (12 items: 15 ms against 2.0 s) |
/// | any | intuitionistic | | [`TwoSided`](Engine::TwoSided) | the general engine; on equal literals, as in the Horn encodings of Partition, 10 to 10⁵ times faster than the net engine, which is not the default there for that reason |
/// | any | classical | | [`Focus`](Engine::Focus) | the general engine, the same on the one-sided sequent |
///
/// The bias of the focused engines, [`Bias::Auto`], is chosen per goal as
/// well, by the measurements its documentation names.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Engine {
    /// The focused sequent engine, one-sided: classical mode, every
    /// fragment. It searches backward over dyadic sequents `⊢ Θ ; Γ`, `Θ`
    /// the formulas that came under a `?` and may be copied, `Γ` the
    /// formulas each used once. The asynchronous phase decomposes the
    /// negative formulas without a choice until a stable sequent remains,
    /// which a memo of stable sequents decides, or the counts of each
    /// atom's literals refute without a search, or a focus on a positive
    /// formula of `Γ`, or on a copy from `Θ`, proves. The copies are bounded
    /// per branch and the bound deepens: `Unprovable` comes only from a
    /// level that never met its bound. With Mix a stable sequent may be
    /// split in two, and in affine mode a leaf weakens what is left. It
    /// reads every option but [`Options::test_period`], the net engine's.
    /// Its unit of work ([`Progress::work`], [`Limits::work`]) is a stable
    /// sequent, weighed by the size of its zones, members and copies, and
    /// a step of a split search; forced splits count none.
    Focus,
    /// The proof-net engine, for unit-free MLL with or without Mix; in
    /// intuitionistic mode it decides IMLL through the embedding into MLL.
    /// A cut-free proof of MLL is its axiom linking, so the engine links
    /// dual literals by backtracking over a [`ProofStructure`] and keeps
    /// the first linking the correctness criterion accepts; the count
    /// equation and each atom's balance refuse most unprovable sequents
    /// before any link, and the net found is sequentialized into the proof
    /// returned. Of the options it reads [`Options::jobs`] and the pool,
    /// [`Options::test_period`], [`Options::check`] and the occurrence
    /// limit: a unit-free MLL sequent has no copies to bound, and the
    /// engine keeps no memo and no recursion, so the copy bounds, the
    /// bias, the memo limit and the recursion limit have nothing to bound
    /// here, and its structure and scratch, linear in the forest, are not
    /// counted under [`Limits::memory_bytes`](crate::Limits::memory_bytes).
    /// Its unit of work is a literal chosen and an exact test that fails.
    Net,
    /// The focused sequent engine two-sided: intuitionistic mode. It is
    /// the search of `Focus` on the one-sided sequent, which keeps one goal
    /// on every branch by itself except at the split of a hypothesis
    /// `A ⊸ B`, where the goal must go with `B`: the one place it reads the
    /// sequent's [`Reading`]. It reads the options `Focus` does, and
    /// counts its work as `Focus` does.
    TwoSided,
    /// The fast path for a sequent of two additive-only formulas, in every
    /// mode: a recursion on pairs of subformula occurrences, one below each
    /// root, memoized on the pair, in time proportional to the product of
    /// the two formulas' sizes. It reads [`Options::memo_limit`] and
    /// [`Options::check`], and the limits' recursion depth and memory
    /// bound, and runs on the calling thread whatever
    /// [`Options::jobs`] says; two additive formulas have no copies and no
    /// atoms to bias. Its unit of work is a pair decided, which it counts
    /// at its polls, every 1 024 pairs.
    Additive,
    /// The engine for Horn programs: clauses under `!` that may be used
    /// any number of times, implications used once, atoms, and one goal
    /// that is a tensor of atoms, which is a Petri net with a marking to
    /// reach (`!(a ⊗ b ⊸ c), a, b ⊢ c`: the clause is a transition, the
    /// atoms on the left the tokens, the goal the marking). It searches
    /// the markings instead of sequents: each reached once, kept as a
    /// count of tokens per atom, and expanded nearest to the goal first,
    /// the distance being the tokens by which the two differ; an
    /// implication used once is a transition that takes a token of its
    /// own. A firing sequence that reaches the goal is the proof: every
    /// firing a copy of its clause, whose body takes its tokens by axioms
    /// and whose head adds its atoms to the context. Every marking
    /// reached having been expanded without reaching the goal is
    /// `Unprovable`, since a proof of a Horn program is a firing sequence
    /// read upward; so is, before any search, a goal whose atom counts
    /// cannot balance, as for every engine. Beside the search, a slice at
    /// a time and never more work than the search has done, a simplex
    /// solves the net's state equation, whether the goal's tokens are the
    /// start's plus some number of firings of each clause: where it has no
    /// solution, Farkas' lemma gives each atom a weight under which no
    /// clause raises the weighted count of the tokens while the goal asks
    /// it raised, which an exact check in integers confirms, and the goal
    /// is `Unprovable` with that [`Refutation::StateEquation`], however
    /// many markings the net has. A net whose markings grow without end
    /// and whose equation has a solution is searched until the stop or
    /// [`Limits::memory_bytes`](crate::Limits::memory_bytes), which counts the markings kept and the
    /// simplex's basis. Two more refutations reach such nets: a clause
    /// with an atom that no reachable marking holds never fires and is
    /// left out, and once the search has done some work, the same search
    /// runs backward from the goal beside it, a quarter of the work, whose
    /// exhausting its markings refutes and whose firing sequence, read
    /// backward, proves. In affine mode, where the tokens and the clauses
    /// a firing sequence leaves are weakened, the question is whether a
    /// marking that covers the goal is reachable, and the engine decides
    /// it backward: from the goal, the least markings from which a firing
    /// leads to one that covers it, each kept unless a smaller one is,
    /// until one lies below the start (the proof) or no new one comes
    /// (`Unprovable`), which Dickson's lemma says happens; the state
    /// equation runs beside it as in linear mode. Classical or
    /// intuitionistic, with or without Mix (which no proof of such a goal
    /// can use). It reads
    /// [`Options::check`] and the limits' memory bound, runs on the
    /// calling thread whatever [`Options::jobs`] says, and needs no copy
    /// bound, memo limit or recursion limit: it keeps every marking once
    /// and recurses nowhere. Its unit of work is a marking taken from the
    /// frontier or an element from the queue, a marking computed backward,
    /// and a pivot of the state equation.
    Horn,
}

impl Engine {
    /// Whether the engine searches on several threads when
    /// [`Options::jobs`] asks for them: the focused engines and the net
    /// engine do; the additive and the Horn engine run on the calling
    /// thread whatever it says.
    pub const fn parallel(self) -> bool {
        match self {
            Engine::Focus | Engine::Net | Engine::TwoSided => true,
            Engine::Additive | Engine::Horn => false,
        }
    }

    /// The implementation of the engine.
    fn implementation(self) -> &'static dyn Decide {
        match self {
            Engine::Focus => &focus::ONE_SIDED,
            Engine::TwoSided => &focus::TWO_SIDED,
            Engine::Net => &net::Nets,
            Engine::Additive => &additive::Additive,
            Engine::Horn => &horn::Horn,
        }
    }
}

impl Display for Engine {
    /// Writes the engine's name: `focus`, `net`, `two-sided`, `additive` or
    /// `horn`.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(self.name())
    }
}

/// A counter of [`Statistics`] as an engine fills it
/// ([`Engine::counters`]).
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Counter {
    /// The field's name in the statistics' JSON form.
    pub key: &'static str,
    /// A short label, as the command's `--stats` prints it.
    pub label: &'static str,
    /// What it counts for the engine.
    pub meaning: &'static str,
}

impl Counter {
    /// A counter of the key, the label and the meaning given.
    const fn new(key: &'static str, label: &'static str, meaning: &'static str) -> Self {
        Self {
            key,
            label,
            meaning,
        }
    }
}

/// Why an engine the options force does not take a goal, which each
/// engine checks in this order: the largest fragment it decides, its
/// modes, whether the goal is the sequent itself, and the goal's shape.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NotTaken {
    /// The goal lies beyond the largest fragment the engine decides.
    #[non_exhaustive]
    Fragment {
        /// The largest fragment the engine decides.
        decides: Fragment,
        /// The fragment the goal was searched in.
        goal: Fragment,
    },
    /// The engine does not search in the mode.
    #[non_exhaustive]
    Mode {
        /// The mode asked for.
        mode: Mode,
    },
    /// The goal is not the sequent itself, the only goal the engine takes.
    Goal,
    /// The goal is not of the engine's shape: two additive-only formulas,
    /// or a Horn program.
    Shape,
}

impl NotTaken {
    /// Returns the sentence that says why `engine` does not take the goal.
    pub fn explained(self, engine: Engine) -> impl Display {
        Explained(engine, self)
    }
}

/// The sentence of a refusal of a forced engine.
struct Explained(Engine, NotTaken);

impl Display for Explained {
    /// Writes why the engine does not take the goal, in the engine's
    /// words where it has them.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let Explained(engine, because) = *self;
        match (engine, because) {
            (Engine::Net, NotTaken::Fragment { goal, .. }) => {
                write!(
                    f,
                    "proof nets exist for MLL without units only, not for {goal}"
                )
            }
            (Engine::Net, NotTaken::Mode { mode }) => write!(
                f,
                "proof nets exist in linear mode only, with or without Mix, not in {mode} mode"
            ),
            (_, NotTaken::Goal) => write!(
                f,
                "the {engine} engine decides the whole sequent only, not a goal within it"
            ),
            (_, NotTaken::Mode { mode }) => {
                write!(f, "the {engine} engine does not search in {mode} mode")
            }
            (Engine::Additive, NotTaken::Fragment { goal, .. }) => write!(
                f,
                "the additive engine decides a sequent of two additive-only formulas, not one of \
                 {goal}"
            ),
            (Engine::Additive, NotTaken::Shape) => f.write_str(
                "the additive engine decides a sequent of two additive-only formulas only",
            ),
            (Engine::Horn, NotTaken::Fragment { .. } | NotTaken::Shape) => f.write_str(
                "the horn engine decides a Horn program only: atoms, implications between \
                 tensors of atoms, such implications under !, and one goal that is a tensor of \
                 atoms",
            ),
            (_, NotTaken::Fragment { decides, goal }) => {
                write!(
                    f,
                    "the {engine} engine decides {decides} at most, not {goal}"
                )
            }
            (_, NotTaken::Shape) => {
                write!(f, "the {engine} engine does not take a goal of this shape")
            }
        }
    }
}

/// How the focused engine chooses the positive literal of every atom.
/// Focusing is complete for every choice, so the rules differ in speed
/// and, with exponentials, in the copies a branch of the proofs they lead
/// to needs: never in what is provable.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Bias {
    /// [`Factors`](Self::Factors) for a sequent without exponentials and
    /// [`Rarer`](Self::Rarer) for a search with weakening. For a sequent
    /// with a `!` or a `?` in linear mode the search runs a search under
    /// each rule and answers with the first that decides, so it decides
    /// whatever either does: the backward one within the copy bound, the
    /// forward one within a bound of its own where the goal is a Horn
    /// program. Each choice is what measured best there: without
    /// exponentials the factor rule visits up to a third of the stable
    /// sequents of the rarer one (unsolvable 3-Partition with bins of
    /// five 971 against 3 373, a random QBF of 20 variables 60 883
    /// against 105 667); under weakening nothing forces a split, and the
    /// factor rule visited up to 700 times the stable sequents of the
    /// rarer one on random affine sequents; with exponentials neither
    /// rule wins (of the Petri nets of the LLTP library within 5 s, the
    /// pair decides 1 520, the backward search run alone 442 and the
    /// forward one run alone 1 576, since under a time limit each of the
    /// pair has its share of the time, and the pair costs 1.3 times the
    /// backward search and 1.6 times the forward one where each decides).
    /// That the pair decides whatever either does holds without a stop.
    #[default]
    Auto,
    /// The literal with fewer occurrences in the sequent is positive, `Atom`
    /// when both have the same number. With Horn-like hypotheses this
    /// mostly chains backward from the goal, which keeps the copies per
    /// branch low.
    Rarer,
    /// The literal that is more often a direct factor of a `⊗` is
    /// positive, an occurrence counting half for every `&` or `⊕` above
    /// it; the rarer literal on a tie. A `⊗` with a positive literal for a
    /// factor takes exactly the dual literal for it, so its split needs no
    /// search. With Horn-like hypotheses this chains forward from the
    /// facts, one copy per step on a single branch: fast, and in need of a
    /// copy bound as large as the number of steps.
    Factors,
}

impl Engine {
    /// Every engine, in the order of [`NAMES`](Self::NAMES).
    pub const ALL: &'static [Self] = &[
        Self::Focus,
        Self::Net,
        Self::TwoSided,
        Self::Additive,
        Self::Horn,
    ];

    /// The engines' names, as [`name`](Self::name) writes them and
    /// [`FromStr`] reads them.
    pub const NAMES: &'static [&'static str] = &["focus", "net", "two-sided", "additive", "horn"];

    /// Returns the counters of [`Statistics`] the engine fills, each with
    /// its key in the statistics' JSON form, a label and what it counts
    /// for this engine; the others stay zero. A front end shows these.
    pub const fn counters(self) -> &'static [Counter] {
        /// The unit of work, which every engine counts.
        const WORK: Counter = Counter::new("work", "units of work", "the engine's unit of work");
        /// The focused engines' counters.
        const FOCUSED: &[Counter] = &[
            Counter::new(
                "nodes",
                "stable sequents visited",
                "stable sequents visited, memo hits included",
            ),
            Counter::new(
                "memo_hits",
                "from the memo",
                "stable sequents the memo answered",
            ),
            Counter::new(
                "memo_entries",
                "memo entries at most",
                "the most entries the memo held at once",
            ),
            Counter::new(
                "splits",
                "splits examined",
                "steps of the split searches and forced splits",
            ),
            Counter::new(
                "copies",
                "copy bound reached",
                "the copy bound of the last level begun",
            ),
            Counter::new(
                "forward_copies",
                "forward search's copy bound reached",
                "the forward search's level under the default bias",
            ),
            WORK,
        ];
        /// The net engine's counters.
        const NET: &[Counter] = &[
            Counter::new(
                "nodes",
                "literals chosen",
                "literals the search chose a partner for",
            ),
            Counter::new("links", "links tried", "axiom links made"),
            Counter::new("tests", "exact tests run", "exact acyclicity tests"),
            WORK,
        ];
        /// The additive path's counters.
        const ADDITIVE: &[Counter] = &[
            Counter::new(
                "nodes",
                "pairs of subformulas visited",
                "pairs decided, memo hits included",
            ),
            Counter::new("memo_hits", "from the memo", "pairs the memo answered"),
            Counter::new(
                "memo_entries",
                "memo entries",
                "the most pairs the memo held at once",
            ),
            WORK,
        ];
        /// The Horn engine's counters.
        const HORN: &[Counter] = &[
            Counter::new(
                "nodes",
                "markings reached",
                "markings reached, or in affine mode computed backward",
            ),
            Counter::new(
                "memo_hits",
                "markings met again",
                "markings kept already, or covered already",
            ),
            Counter::new("memo_entries", "markings kept", "markings kept"),
            WORK,
        ];
        match self {
            Self::Focus | Self::TwoSided => FOCUSED,
            Self::Net => NET,
            Self::Additive => ADDITIVE,
            Self::Horn => HORN,
        }
    }

    /// Returns the engine's name: `focus`, `net`, `two-sided`, `additive`
    /// or `horn`.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Focus => "focus",
            Self::Net => "net",
            Self::TwoSided => "two-sided",
            Self::Additive => "additive",
            Self::Horn => "horn",
        }
    }
}

impl FromStr for Engine {
    type Err = Error;

    /// Reads an engine's name, one of [`NAMES`](Self::NAMES).
    fn from_str(name: &str) -> Result<Self, Error> {
        Self::ALL
            .iter()
            .copied()
            .find(|engine| engine.name() == name)
            .ok_or_else(|| Error::UnknownName {
                what: "engine",
                name: name.into(),
                known: Self::NAMES,
            })
    }
}

impl Bias {
    /// Every rule, in the order of [`NAMES`](Self::NAMES).
    pub const ALL: &'static [Self] = &[Self::Auto, Self::Rarer, Self::Factors];

    /// The rules' names, as [`name`](Self::name) writes them and
    /// [`FromStr`] reads them.
    pub const NAMES: &'static [&'static str] = &["auto", "rarer", "factors"];

    /// Returns the rule's name: `auto`, `rarer` or `factors`.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Rarer => "rarer",
            Self::Factors => "factors",
        }
    }
}

impl Display for Bias {
    /// Writes the rule's [`name`](Self::name).
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(self.name())
    }
}

impl FromStr for Bias {
    type Err = Error;

    /// Reads a rule's name, one of [`NAMES`](Self::NAMES).
    fn from_str(name: &str) -> Result<Self, Error> {
        Self::ALL
            .iter()
            .copied()
            .find(|bias| bias.name() == name)
            .ok_or_else(|| Error::UnknownName {
                what: "bias",
                name: name.into(),
                known: Self::NAMES,
            })
    }
}

/// How many threads a search may use: written `"auto"` or the number.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Jobs {
    /// Every thread the machine runs at once, as the platform tells, or one
    /// where it does not; a front end resolves it once to the count it
    /// shows.
    Auto,
    /// This many. One runs the sequential engines, whose proof is a
    /// function of the input; zero counts as one, and more than
    /// [`Options::MAX_JOBS`] as that many.
    Count(usize),
}

impl Default for Jobs {
    /// One thread.
    fn default() -> Self {
        Self::Count(1)
    }
}

impl From<usize> for Jobs {
    /// That many threads.
    fn from(count: usize) -> Self {
        Self::Count(count)
    }
}

impl Jobs {
    /// Returns the number of threads: the count asked for, or the
    /// machine's for [`Auto`](Self::Auto), at least one and at most
    /// [`Options::MAX_JOBS`].
    pub fn count(self) -> usize {
        let count = match self {
            Self::Auto => std::thread::available_parallelism().map_or(1, NonZeroUsize::get),
            Self::Count(count) => count,
        };
        count.clamp(1, Options::MAX_JOBS)
    }
}

/// How often the net engine runs its exact test: written `"auto"` or the
/// number of links between two tests.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Cadence {
    /// After every link on a structure of at most
    /// [`AUTO_SMALL`](Self::AUTO_SMALL) occurrences, every
    /// [`AUTO_PERIOD`](Self::AUTO_PERIOD)th link on a larger one.
    #[default]
    Auto,
    /// After this many links; zero counts as one.
    Every(u32),
}

impl Cadence {
    /// The most occurrences a structure has on which [`Auto`](Self::Auto)
    /// tests after every link.
    pub const AUTO_SMALL: usize = 200;

    /// How many links go between two tests of [`Auto`](Self::Auto) on a
    /// larger structure.
    pub const AUTO_PERIOD: u32 = 4;
}

impl From<u32> for Cadence {
    /// Every so many links.
    fn from(links: u32) -> Self {
        Self::Every(links)
    }
}

impl From<Option<u32>> for Cadence {
    /// Every so many links, or the default for `None`.
    fn from(links: Option<u32>) -> Self {
        links.map_or(Self::Auto, Self::Every)
    }
}

/// How the two searches of the default bias share one thread: written
/// `"auto"` or `"turns"`.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serialize",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
pub enum Schedule {
    /// In slices on two threads, where the feature `parallel` starts a
    /// second one, and in turns otherwise.
    #[default]
    Auto,
    /// In turns on the calling thread, each turn from the search's start:
    /// the same polls and the same answer on every build, the web's
    /// included.
    Turns,
}

/// The knobs of a search: how much to remember, how deep to go, and which
/// fragment and engine to use instead of the detected ones. The defaults
/// suit a sequent of a few hundred occurrences on a thread with the usual
/// stack; the bounds on memory, occurrences and recursion are the
/// [`Limits`] a search is given, whose bound on work binds the check of
/// the proof found.
///
/// Every field has a builder, so that the options chain from the default.
///
/// # JSON
///
/// With the feature `serialize` the options are an object of the fields
/// below, a missing one taking its default and a misspelt one refused:
/// `"engine"` and `"fragment"` a name or `"auto"`, `"bias"` and
/// `"schedule"` a name, `"copies"` a number or `null` for no bound,
/// `"forward_copies"` and `"memo_limit"` numbers, `"test_period"` and
/// `"jobs"` a number or `"auto"`, `"check"` a boolean. The pool is never
/// written.
///
/// # Examples
///
/// ```
/// use linlog::Fragment;
/// use linlog::search::Options;
///
/// let options = Options::default()
///     .with_memo_limit(1 << 16)
///     .with_fragment(Some(Fragment::MALL));
/// ```
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialize",
    derive(serde::Serialize, serde::Deserialize),
    serde(default, deny_unknown_fields)
)]
pub struct Options {
    /// The engine to use, or `None` for the one the detected fragment and
    /// the mode call for.
    #[cfg_attr(feature = "serialize", serde(with = "crate::serialize::auto"))]
    pub engine: Option<Engine>,
    /// The fragment to search in, or `None` for the detected one. A
    /// sequent outside it is refused; a fragment larger than the detected
    /// one switches off the prunes that only hold in the smaller one.
    #[cfg_attr(feature = "serialize", serde(with = "crate::serialize::auto"))]
    pub fragment: Option<Fragment>,
    /// How the focused engine picks the positive literal of every atom.
    /// No choice changes what is provable. [`Bias::Rarer`] mostly chains
    /// backward from the goal and [`Bias::Factors`] forward from the
    /// hypotheses; on a sequent with exponentials whose hypotheses are
    /// Horn clauses, a Petri net for one, the forward search often decides
    /// in milliseconds what the backward one does not decide at all, but
    /// its proofs take one copy per step on a single branch. The default,
    /// [`Bias::Auto`], is `Factors` without exponentials and `Rarer` under
    /// weakening; on a sequent with exponentials in linear mode it runs
    /// both searches, the backward one within [`copies`](Self::copies) and
    /// the forward one within [`forward_copies`](Self::forward_copies),
    /// and answers with the first that decides. Either of the two named
    /// explicitly is that search alone, within `copies`. Only the focused
    /// engine reads it.
    pub bias: Bias,
    /// The most copies of `?` formulas one branch of a proof may take, or
    /// `None` for no bound. The search deepens the bound from zero, one
    /// level after another; a sequent that has no proof within the bound
    /// is [`Reason::CopyBound`], unless some level finished without ever
    /// reaching its bound, which makes the sequent [`Verdict::Unprovable`].
    /// Without a bound the search goes on to the next level until it
    /// decides, its stop condition fires or a limit binds, and
    /// [`Statistics::copies`] says how far it got: the choice of a caller
    /// with a time limit, as the command is, since no bound is too small
    /// for some provable sequent and full linear logic is undecidable.
    /// Without exponentials the bound has no effect. A proof found at some
    /// level may reuse a memoized subproof found with more copies left, so
    /// the bound limits the search, not the proof returned. Only the
    /// focused engine reads it: the net and the additive engine's goals
    /// have no exponentials, and the Horn engine keeps every marking once
    /// and needs no bound.
    pub copies: Option<u32>,
    /// The most copies of `?` formulas one branch may take in the forward
    /// search that [`Bias::Auto`] runs beside the backward one on a Horn
    /// program: clauses `!(a ⊗ b ⊸ c ⊗ d)` (or one used once), a marking
    /// and a goal of atoms, as a Petri net is. A forward chain takes a
    /// copy per step, all on one branch, so it wants a larger bound than
    /// [`copies`](Self::copies), which this is when it is the larger of
    /// the two; the forward search never runs within less than `copies`.
    /// On any other goal, under another bias, under Mix, under weakening
    /// and without a copy bound it has no effect: unbounded, the forward
    /// search deepens as far as its share of the work takes it. Only the
    /// focused engine reads it.
    pub forward_copies: u32,
    /// The most stable sequents the memo holds at once; when the memo is
    /// full it is emptied, which costs time but not correctness. Zero
    /// switches the memo off. The two searches that [`Bias::Auto`] runs on
    /// a sequent with exponentials hold a memo of this size each. This is
    /// the finer knob beside the memory bound
    /// ([`Limits::memory_bytes`](crate::Limits::memory_bytes)), which
    /// bounds the memo in bytes: a table that fits the processor's cache
    /// can be faster than one that fits the memory. The focused engine and
    /// the additive path read it; the net engine keeps no memo.
    pub memo_limit: u32,
    /// How often the net engine runs its exact acyclicity test: the test
    /// also runs on every complete linking, so the cadence trades time per
    /// link against how long a doomed branch is followed. Only the net
    /// engine reads it.
    pub test_period: Cadence,
    /// How many threads the search may use. One, the default, runs the
    /// sequential engines, whose proof is a function of the input. More
    /// than one, with the `parallel` feature, runs the focused engine and
    /// the net engine on that many threads of a pool of their own, which
    /// may find a different proof but never contradict the sequential
    /// verdict: one of the two may decide where the other stops at the
    /// copy bound or the recursion limit, and which one depends on how the
    /// threads interleave; without the feature, or for the additive path
    /// and the Horn engine, the search stays sequential. A search starts no
    /// more threads than the machine runs at once
    /// ([`std::thread::available_parallelism`], where the platform tells):
    /// threads beyond that only take turns on the same processors.
    pub jobs: Jobs,
    /// How the two searches of the default bias share one thread.
    pub schedule: Schedule,
    /// Whether a proof of the sequent passes the checker
    /// ([`Proof::check`], which shares no code with the engines) before
    /// the search returns it; one that does not is [`Error::Rejected`],
    /// never a verdict. Without the check the proof is the engine's word,
    /// which a caller that checks it itself, or times the search alone,
    /// may prefer. A proof of a goal is checked against the goal.
    pub check: bool,
    /// The thread pools a parallel search borrows instead of starting
    /// threads of its own, or `None`, the default, for a pool built for
    /// the search and dropped with it. A pool changes nothing a search
    /// does or finds, only that its threads are started once for many
    /// searches. Options are equal only if they name clones of the same
    /// pool or both name none. Never written.
    ///
    /// Needs the cargo feature `parallel` (off by default).
    #[cfg(feature = "parallel")]
    #[cfg_attr(feature = "serialize", serde(skip))]
    pub pool: Option<Pool>,
}

impl Default for Options {
    /// The engine and fragment chosen by detection, the default bias, a
    /// copy bound of [`DEFAULT_COPIES`](Self::DEFAULT_COPIES), one of
    /// [`DEFAULT_FORWARD_COPIES`](Self::DEFAULT_FORWARD_COPIES) for the
    /// forward search of the default bias, a memo of at most
    /// [`DEFAULT_MEMO_LIMIT`](Self::DEFAULT_MEMO_LIMIT) stable sequents,
    /// the net engine's exact test at its default cadence, one thread, the
    /// default schedule, and every proof checked
    /// ([`DEFAULT_CHECK`](Self::DEFAULT_CHECK)).
    fn default() -> Self {
        Self {
            engine: None,
            fragment: None,
            bias: Bias::Auto,
            copies: Some(Self::DEFAULT_COPIES),
            forward_copies: Self::DEFAULT_FORWARD_COPIES,
            memo_limit: Self::DEFAULT_MEMO_LIMIT,
            test_period: Cadence::Auto,
            jobs: Jobs::Count(1),
            schedule: Schedule::Auto,
            check: Self::DEFAULT_CHECK,
            #[cfg(feature = "parallel")]
            pool: None,
        }
    }
}

impl Options {
    /// The memo limit of the default options: 2²⁰ stable sequents.
    pub const DEFAULT_MEMO_LIMIT: u32 = 1 << 20;

    /// The copy bound of the default options: three copies per branch, the
    /// bound llprover searches with by default. The default options keep a
    /// bound because a search without one ends only when it decides or its
    /// stop condition fires, and [`prove`] has none; a front end with a
    /// clock lifts it ([`copies`](Self::copies)).
    pub const DEFAULT_COPIES: u32 = 3;

    /// The forward search's copy bound of the default options: thirty
    /// steps of a forward chain. A deeper bound decided nothing more
    /// within seconds on Petri nets from practice, and the price of the
    /// bound is what an undecided search to that depth costs: mostly
    /// milliseconds on a small Horn program, a second and more on one
    /// whose markings grow in several places at once.
    pub const DEFAULT_FORWARD_COPIES: u32 = 30;

    /// Whether the default options check a proof before returning it: they
    /// do. The check is one pass over the proof, in memory proportional to
    /// it.
    pub const DEFAULT_CHECK: bool = true;

    /// The most threads the options name: more are taken as this many.
    /// A pool beyond it has found no use, and a count without a bound
    /// starts whatever it is given (ten thousand threads on `A ⊢ A` cost
    /// three minutes of processor time).
    pub const MAX_JOBS: usize = 256;

    /// Returns the options with another [`engine`](Self::engine).
    #[must_use]
    pub fn with_engine(self, engine: Option<Engine>) -> Self {
        Self { engine, ..self }
    }

    /// Returns the options with another [`fragment`](Self::fragment).
    #[must_use]
    pub fn with_fragment(self, fragment: Option<Fragment>) -> Self {
        Self { fragment, ..self }
    }

    /// Returns the options with another [`bias`](Self::bias).
    #[must_use]
    pub fn with_bias(self, bias: Bias) -> Self {
        Self { bias, ..self }
    }

    /// Returns the options with another copy bound
    /// ([`copies`](Self::copies)).
    #[must_use]
    pub fn with_copies(self, copies: Option<u32>) -> Self {
        Self { copies, ..self }
    }

    /// Returns the options with another copy bound for the forward search
    /// ([`forward_copies`](Self::forward_copies)).
    #[must_use]
    pub fn with_forward_copies(self, forward_copies: u32) -> Self {
        Self {
            forward_copies,
            ..self
        }
    }

    /// Returns the options with another [`memo_limit`](Self::memo_limit).
    #[must_use]
    pub fn with_memo_limit(self, memo_limit: u32) -> Self {
        Self { memo_limit, ..self }
    }

    /// Returns the options with another cadence of the net engine's test
    /// ([`test_period`](Self::test_period)).
    #[must_use]
    pub fn with_test_period(self, test_period: impl Into<Cadence>) -> Self {
        Self {
            test_period: test_period.into(),
            ..self
        }
    }

    /// Returns the options with another number of threads
    /// ([`jobs`](Self::jobs)).
    #[must_use]
    pub fn with_jobs(self, jobs: impl Into<Jobs>) -> Self {
        Self {
            jobs: jobs.into(),
            ..self
        }
    }

    /// Returns the options with another [`schedule`](Self::schedule).
    #[must_use]
    pub fn with_schedule(self, schedule: Schedule) -> Self {
        Self { schedule, ..self }
    }

    /// Returns the options with or without the [`check`](Self::check) of
    /// a proof.
    #[must_use]
    pub fn with_check(self, check: bool) -> Self {
        Self { check, ..self }
    }

    /// Returns the options with another [`pool`](Self::pool).
    ///
    /// Needs the cargo feature `parallel` (off by default).
    #[cfg(feature = "parallel")]
    #[must_use]
    pub fn with_pool(self, pool: Option<Pool>) -> Self {
        Self { pool, ..self }
    }

    /// Returns the copy bound as the engines count it: no bound is the
    /// largest, which no search reaches, since every level visits a stable
    /// sequent and polls at it, and four billion levels take hours at the
    /// least.
    pub(crate) fn copy_bound(&self) -> u32 {
        self.copies.unwrap_or(u32::MAX)
    }

    /// Returns the threads the search may use: [`Jobs::count`].
    pub(crate) fn threads(&self) -> usize {
        self.jobs.count()
    }

    /// Returns the most stable sequents a memo holds.
    pub(crate) fn memo_entries(&self) -> usize {
        usize::try_from(self.memo_limit).unwrap_or(usize::MAX)
    }
}

/// What a search returned: the verdict, and how it was reached.
///
/// # JSON
///
/// With the feature `serialize` an outcome is written, never read, as one
/// object ([`wire`](crate::wire)): `version`, `linlog` (the crate's
/// version, which wrote it), `verdict` (`"proved"`, `"unprovable"` or
/// `"unknown"`), with `checked` for a proved sequent, `refutation` for an
/// unprovable one (in the form [`Refutation`] gives) and `reason` for an
/// unknown one (tagged by `kind`: `{"kind": "stopped"}`, `{"kind":
/// "copy_bound", "copies": 3}`, `{"kind": "memory_limit", "limit_bytes":
/// n}`, `{"kind": "recursion_limit", "depth": n}`, `{"kind":
/// "index_limit"}`),
/// `fragment` (its name in the mode, as [`Fragment::name_in`] gives it),
/// `mode` (its name), `engine`, `statistics`, for a proved sequent the
/// proof's own keys `sequent`, `nodes` and `goal`, so that the outcome
/// reads back as a [`Proof`], and for an unprovable one the
/// [`Disproof`]'s `sequent` and `goal`. The command's `prove --format
/// json` writes it.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct Outcome {
    /// The verdict, with the proof if there is one.
    pub verdict: Verdict,
    /// The fragment searched in: the detected one, or the one the options
    /// asserted.
    pub fragment: Fragment,
    /// The mode searched in.
    pub mode: Mode,
    /// The engine that ran.
    pub engine: Engine,
    /// What the search cost.
    pub statistics: Statistics,
    /// The proof net the proof was read off, when the net engine found
    /// one; `None` for the other engines and for any other verdict.
    pub net: Option<ProofStructure>,
    /// Whether the proof passed the checker before it was returned:
    /// false only where [`Options::check`] switched the check off, and
    /// for every verdict but a proof.
    pub checked: bool,
}

/// What a search found: a proof, that there is none, or that it could not
/// tell.
#[derive(Clone, Debug)]
pub enum Verdict {
    /// The sequent is provable, and here is a proof, boxed because a proof
    /// carries its forest.
    Proved(Box<Proof>),
    /// The sequent is not provable: the search was exhaustive, and here is
    /// what can be said of why, boxed as a proof is.
    Unprovable(Box<Disproof>),
    /// The search stopped before it could decide, for the reason given.
    Unknown(Reason),
}

impl Verdict {
    /// Returns the proof, if the sequent was proved.
    pub fn proof(&self) -> Option<&Proof> {
        match self {
            Verdict::Proved(proof) => Some(proof),
            _ => None,
        }
    }
}

/// Why a search stopped without deciding.
///
/// In JSON (feature `serialize`) a reason is tagged by `kind`:
/// `{"kind": "stopped"}`, `{"kind": "recursion_limit", "depth": 2048}`,
/// `{"kind": "copy_bound", "copies": 3}`, `{"kind": "memory_limit",
/// "limit_bytes": 1073741824}`, `{"kind": "index_limit"}`, `{"kind":
/// "work_limit", "limit": 1000000}` or `{"kind": "unchecked",
/// "limit_bytes": 1073741824}`; a reader refuses a kind it does not know.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Reason {
    /// The caller's stop condition fired: a time limit or an interruption.
    Stopped,
    /// The nesting of engine calls reached
    /// [`Limits::recursion_depth`](crate::Limits::recursion_depth).
    #[non_exhaustive]
    RecursionLimit {
        /// The recursion depth the limits allowed.
        depth: u32,
    },
    /// Every level up to the bound [`Options::copies`] set hit its bound
    /// on some branch, so a proof with more copies of a `?` formula per
    /// branch may exist.
    #[non_exhaustive]
    CopyBound {
        /// The copy bound.
        copies: u32,
    },
    /// The search would have held more than
    /// [`Limits::memory_bytes`](crate::Limits::memory_bytes): with its memo
    /// emptied, where it keeps one, or with no room left for a memo, the
    /// Horn engine's markings or a proof's nodes.
    #[non_exhaustive]
    MemoryLimit {
        /// The memory bound, in bytes.
        limit_bytes: u64,
    },
    /// A structure of the search outgrew what its indices address: the
    /// proof arena at 2³¹ nodes, the additive path's at 2³² nodes, the
    /// count invariants at 2³² row entries, the Horn engine's markings at
    /// 2³² or a count of its tokens at 2³².
    /// Only a search without a memory bound gets this far, but for the
    /// tokens, which a clause that adds thousands of them at each firing
    /// can pass in a few hundred thousand markings.
    IndexLimit,
    /// The search did the most work [`Limits::work`](crate::Limits::work)
    /// allows, or the check of the proof it found would have done more.
    #[non_exhaustive]
    WorkLimit {
        /// The bound on work.
        limit: u64,
    },
    /// The search found a proof whose check would have held more than
    /// [`Limits::memory_bytes`](crate::Limits::memory_bytes): no proof is
    /// returned that has not passed the checker.
    #[non_exhaustive]
    Unchecked {
        /// The memory bound, in bytes.
        limit_bytes: u64,
    },
}

impl Reason {
    /// The reason a search of the options gives up with, given the reason
    /// one of its searches did: a copy bound is [`Options::copies`], the
    /// bound every search ran within at the least, and a memory limit is
    /// [`Limits::memory_bytes`], of which a search may have had a part.
    pub(crate) fn as_set(self, options: &Options, limits: &Limits) -> Self {
        match self {
            Self::CopyBound { .. } => Self::CopyBound {
                copies: options.copy_bound(),
            },
            Self::MemoryLimit { limit_bytes: bytes } => Self::MemoryLimit {
                limit_bytes: limits.memory_bytes.unwrap_or(bytes),
            },
            reason => reason,
        }
    }

    /// Returns the settings key whose bound the search reached, which a
    /// caller raises to search further; `None` for the stop and the
    /// indices, which no setting lifts.
    pub const fn setting(&self) -> Option<&'static str> {
        match self {
            Self::Stopped | Self::IndexLimit => None,
            Self::RecursionLimit { .. } => Some("limits.recursion_depth"),
            Self::CopyBound { .. } => Some("search.copies"),
            Self::MemoryLimit { .. } | Self::Unchecked { .. } => Some("limits.memory_bytes"),
            Self::WorkLimit { .. } => Some("limits.work"),
        }
    }
}

impl Display for Reason {
    /// Writes the reason as a phrase, such as `the copy bound of 3 was
    /// reached`.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Reason::Stopped => f.write_str("the search was stopped"),
            Reason::RecursionLimit { depth } => {
                write!(f, "the recursion limit of {depth} was reached")
            }
            Reason::CopyBound { copies } => {
                write!(f, "the copy bound of {copies} was reached")
            }
            Reason::MemoryLimit { limit_bytes } => {
                write!(f, "the memory limit of {} was reached", Bytes(*limit_bytes))
            }
            Reason::IndexLimit => f.write_str("the search outgrew what its indices address"),
            Reason::WorkLimit { limit } => write!(f, "the work limit of {limit} was reached"),
            Reason::Unchecked { limit_bytes } => write!(
                f,
                "a proof was found, but its check would hold more than the memory limit of {}",
                Bytes(*limit_bytes)
            ),
        }
    }
}

/// What a search cost. The focused engine counts stable sequents, memo use
/// and splits; the net engine counts literals chosen, links and exact
/// tests; the additive path counts pairs of subformulas; the Horn engine
/// counts markings in the first three; the other counters stay zero.
///
/// In JSON (feature `serialize`) an object of the counters by name, read
/// back with a missing one as zero.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serialize", serde(default))]
#[non_exhaustive]
pub struct Statistics {
    /// The nodes of the search: the stable sequents the focused engine
    /// visited, memo hits included, the literals the net engine chose a
    /// partner for, the pairs of subformulas the additive path decided, or
    /// the markings the Horn engine reached, the initial one included, and
    /// in affine mode the markings it computed backward from the goal.
    pub nodes: u64,
    /// The visits answered from the memo; of the Horn engine, the markings
    /// it had kept already, and in affine mode those a marking kept
    /// already covers.
    pub memo_hits: u64,
    /// The most stable sequents the memo held at once; on a pool, whose
    /// memo is shards each emptied by itself, the sum of the shards'
    /// peaks, which bounds it from above; of two searches that ran
    /// together, the two memos' together, and of two that took turns from
    /// their start, the most of one turn; of the Horn engine, the markings
    /// it kept. A `u64` on every target, as every counter is.
    pub memo_entries: u64,
    /// The context splits examined for `⊗` and Mix, most of them rejected by
    /// the counts.
    pub splits: u64,
    /// The axiom links the net engine tried: each was made, and taken back
    /// again unless it is part of the net found.
    pub links: u64,
    /// The exact acyclicity tests the net engine ran.
    pub tests: u64,
    /// The copy bound the last level of the focused engine's deepening
    /// began under: of the search that decided, else of the one under
    /// [`Options::copies`] (the backward one of the default bias's two),
    /// so how far a search without a bound got before it was stopped.
    /// Zero without exponentials and for the other engines.
    pub copies: u32,
    /// The copy bound the last level of the forward search of
    /// [`Bias::Auto`] began under, which runs within
    /// [`Options::forward_copies`] on a Horn program; zero where it did
    /// not run.
    pub forward_copies: u32,
    /// The units of work the search did, in its engine's unit (each
    /// [`Engine`] variant says which): on one thread what the
    /// [`Progress::work`] of its polls add up to.
    pub work: u64,
}

impl Statistics {
    /// Adds the counters of another search, one that ran beside this one
    /// or after it, to these: the work done adds up, and the memo's peak
    /// and the copy bound reached are the larger of the two, since each
    /// search had a memo and a deepening of its own.
    pub fn add(&mut self, other: &Statistics) {
        self.add_run(other);
        self.memo_hits += other.memo_hits;
        self.memo_entries = self.memo_entries.max(other.memo_entries);
        self.forward_copies = self.forward_copies.max(other.forward_copies);
        self.work += other.work;
    }

    /// Adds another engine's counters to these, the memo's excepted: they
    /// describe a table, not a run, and a parallel search reads them off
    /// the one table its workers share.
    pub(crate) fn add_run(&mut self, other: &Statistics) {
        self.nodes += other.nodes;
        self.splits += other.splits;
        self.links += other.links;
        self.tests += other.tests;
        self.copies = self.copies.max(other.copies);
    }
}

#[cfg(all(test, feature = "parse"))]
mod tests {
    use super::*;

    /// Parses `input`.
    fn sequent(input: &str) -> Sequent {
        input.parse().unwrap_or_else(|e| panic!("{input:?}: {e}"))
    }

    /// An outcome is checked exactly when it is a proof and the options
    /// ask for the check; a cadence of links is a number.
    #[test]
    fn checked_when_proved_and_asked() {
        let tautology = sequent("|- a, ~a");
        let proved = prove(&tautology, Mode::CLASSICAL, &Options::default()).unwrap();
        assert!(proved.checked);
        let unchecked = Options::default().with_check(false);
        assert!(
            !prove(&tautology, Mode::CLASSICAL, &unchecked)
                .unwrap()
                .checked
        );
        let refuted = prove(&sequent("|- a, a"), Mode::CLASSICAL, &Options::default()).unwrap();
        assert!(matches!(refuted.verdict, Verdict::Unprovable(_)) && !refuted.checked);
        assert_eq!(Cadence::from(7), Cadence::Every(7));
    }

    /// Every counter an engine lists is a field of the statistics' JSON
    /// form, once.
    #[cfg(feature = "serialize")]
    #[test]
    fn counters_are_fields() {
        let written = serde_json::to_value(Statistics::default()).unwrap();
        for &engine in Engine::ALL {
            let keys: Vec<&str> = engine.counters().iter().map(|c| c.key).collect();
            for key in &keys {
                assert!(written.get(key).is_some(), "{engine}: {key}");
            }
            let mut unique = keys.clone();
            unique.dedup();
            assert_eq!(unique, keys, "{engine}");
        }
    }

    /// The listed names are the names written, in order, and each reads
    /// back as its value.
    #[test]
    fn names_are_listed() {
        let engines: Vec<_> = Engine::ALL.iter().map(|e| e.name()).collect();
        let biases: Vec<_> = Bias::ALL.iter().map(|b| b.name()).collect();
        assert_eq!(engines, Engine::NAMES);
        assert_eq!(biases, Bias::NAMES);
        for &engine in Engine::ALL {
            assert_eq!(engine.to_string().parse::<Engine>().unwrap(), engine);
        }
        for &bias in Bias::ALL {
            assert_eq!(bias.to_string().parse::<Bias>().unwrap(), bias);
        }
    }

    /// An engine's proof that the checker rejects is a defect, never a
    /// verdict: a weakening in classical mode, as an affine search would
    /// leave it.
    #[test]
    fn a_rejected_proof_is_an_error() {
        let s = sequent("|- a, b, ~a");
        let forest = Forest::new(&s).unwrap();
        let (m, n) = (Member::new, NodeId::new);
        let nodes = vec![Node::Ax(m(0), m(2)), Node::Weaken(m(1), n(0))];
        let proof = Proof::new(forest.clone(), nodes, n(1)).unwrap();
        let task = Task {
            forest: &forest,
            goal: forest.roots(),
            fragment: s.fragment(),
            mode: Mode::CLASSICAL,
            reading: None,
            roots: true,
        };
        let verdict = conclude(
            Ok(Some(proof)),
            None,
            &task,
            &[],
            &Options::default(),
            &Limits::default(),
            &mut |_| false,
        );
        assert!(matches!(verdict, Err(Error::Rejected(_))), "{verdict:?}");
    }

    /// A proof whose check a bound gives up is no proof returned: the
    /// verdict is unknown, by the bound that refused the check.
    #[test]
    fn a_refused_check_is_unknown() {
        let refusals = [
            Refusal::Stopped {
                phase: Phase::Check,
            },
            Refusal::Memory {
                phase: Phase::Check,
                limit_bytes: 64,
                needed_bytes: None,
            },
            Refusal::Work { limit: 1 },
        ];
        let reasons = refusals.map(|r| unchecked(r).unwrap());
        assert_eq!(
            reasons,
            [
                Reason::Stopped,
                Reason::Unchecked { limit_bytes: 64 },
                Reason::WorkLimit { limit: 1 }
            ]
        );
        // The check counts its work, the search's proof found within one
        // unit of it is unknown.
        let work = Limits::default().with_work(Some(1));
        let outcome = prove_within(
            &sequent("a, b |- a * b"),
            Mode::CLASSICAL,
            &Options::default(),
            &work,
            |_| false,
        )
        .unwrap();
        assert!(
            matches!(
                outcome.verdict,
                Verdict::Unknown(Reason::WorkLimit { limit: 1 })
            ),
            "{:?}",
            outcome.verdict
        );
        assert!(!outcome.checked);
    }

    /// Every poll tells the stop the work done since the last and in all,
    /// in the engine's units, and the bound on work ends a search there:
    /// an unknown verdict that names the bound.
    #[test]
    fn the_search_counts_its_work() {
        let partition = crate::families::FAMILIES
            .iter()
            .find(|f| f.name == "partition-no")
            .unwrap()
            .instance(3, 0)
            .unwrap();
        let atoms: Vec<String> = (0..8).map(|i| format!("a{i}")).collect();
        let tensor = sequent(&format!("{} |- {}", atoms.join(", "), atoms.join(" * ")));
        for (sequent, mode, engine) in [
            (&partition.sequent, partition.mode, Engine::Focus),
            (&tensor, Mode::CLASSICAL, Engine::Net),
        ] {
            let options = Options::default().with_engine(Some(engine));
            let (mut sum, mut last, mut polls) = (0, 0, 0);
            let outcome = prove_within(sequent, mode, &options, &Limits::default(), |progress| {
                assert_eq!(progress.phase, Phase::Search);
                sum += progress.work;
                last = progress.done;
                polls += 1;
                false
            })
            .unwrap();
            assert!(!matches!(outcome.verdict, Verdict::Unknown(_)), "{engine}");
            assert!(
                polls > 1 && last > 1 && sum == last,
                "{engine}: {sum} {last}"
            );
            assert_eq!(outcome.statistics.work, last, "{engine}");
            let bound = Limits::default().with_work(Some(last / 2));
            let outcome = prove_within(sequent, mode, &options, &bound, |_| false).unwrap();
            assert!(
                matches!(outcome.verdict, Verdict::Unknown(Reason::WorkLimit { limit }) if limit == last / 2),
                "{engine}: {:?}",
                outcome.verdict
            );
        }
    }

    /// The race counts every thread: below three it asks nothing and
    /// searches as one call with that many threads would; from three it
    /// starts the pool once, when asked to, and answers as one thread
    /// does.
    #[cfg(feature = "parallel")]
    #[test]
    fn the_race_counts_every_thread() {
        let s = sequent("!(a -o b), !(b -o c), a |- c * !(d -o d)");
        let forest = Forest::new(&s).unwrap();
        let goal = Goal::conclusion(&forest);
        let (options, limits) = (Options::default(), Limits::default());
        for threads in [1, 2] {
            let outcome = race(
                goal,
                Mode::CLASSICAL,
                &options,
                &limits,
                threads,
                |_| panic!("no pool is added below three threads"),
                |_| false,
            )
            .unwrap();
            assert!(outcome.verdict.proof().is_some(), "{threads}");
        }
        let alone = prove(&s, Mode::CLASSICAL, &options).unwrap();
        let mut asked = 0;
        let raced = race(
            goal,
            Mode::CLASSICAL,
            &options,
            &limits,
            3,
            |_| {
                asked += 1;
                true
            },
            |_| false,
        )
        .unwrap();
        assert_eq!(asked, 1);
        assert!(raced.verdict.proof().is_some());
        assert_eq!(raced.engine, alone.engine);
    }

    /// The front door's plain parts: a sequent past the bound on
    /// occurrences is refused before anything unfolds, the engines that
    /// search on a pool are the focused and the net engine, the default
    /// options are their documented constants, and each refutation says
    /// its counts in words, with the singular, a range, the minus sign and
    /// Mix's inequality where they apply.
    #[test]
    fn the_front_door_s_plain_parts() {
        let tight = Limits::default().with_occurrences(Some(2));
        let error = prove_within(
            &sequent("|- a, ~a, b"),
            Mode::CLASSICAL,
            &Options::default(),
            &tight,
            |_| false,
        )
        .unwrap_err();
        assert!(
            matches!(error, Error::Refused(Refusal::Occurrences { limit: 2, .. })),
            "{error}"
        );
        let parallel: Vec<bool> = Engine::ALL.iter().map(|e| e.parallel()).collect();
        assert_eq!(parallel, [true, true, true, false, false]);
        let defaults = Options::default();
        assert_eq!(
            (
                defaults.copies,
                defaults.forward_copies,
                defaults.memo_limit,
                defaults.check
            ),
            (
                Some(Options::DEFAULT_COPIES),
                Options::DEFAULT_FORWARD_COPIES,
                Options::DEFAULT_MEMO_LIMIT,
                Options::DEFAULT_CHECK
            )
        );
        assert_eq!(
            (Options::DEFAULT_COPIES, Options::DEFAULT_MEMO_LIMIT),
            (3, 1 << 20)
        );
        let atom = Atom::new(0);
        for (refutation, says) in [
            (
                Refutation::Unbalanced(Unbalanced {
                    atom,
                    least: 1,
                    most: 1,
                }),
                "#0 occurs 1 more time than ~#0 in the one-sided sequent, so",
            ),
            (
                Refutation::Unbalanced(Unbalanced {
                    atom,
                    least: -3,
                    most: -2,
                }),
                "~#0 occurs 2 to 3 more times than #0 in the one-sided sequent, whichever",
            ),
            (
                Refutation::Equation(Equation {
                    formulas: 3,
                    needed: -1,
                    tensors: 0,
                    pars: 3,
                    ones: 0,
                    bottoms: 0,
                    mix: true,
                }),
                "has at least #⊗ − #⅋ − #1 + #⊥ + 2 formulas, here 0 − 3 − 0 + 0 + 2 = −1, and this \
                 one has 3",
            ),
            (
                Refutation::StateEquation(StateEquation {
                    atoms: vec![(atom, -2), (Atom::new(1), 1)],
                    clauses: vec![(OccId::new(4), 1)],
                    dropped: Vec::new(),
                }),
                "weighting each #0 by −2, #1 by 1 and the clauses used once by weights of their \
                 own, no clause",
            ),
        ] {
            let text = refutation.to_string();
            assert!(text.contains(says), "{text}");
        }
    }

    /// A reason names the settings key a caller raises to search further,
    /// and says the bound it reached.
    #[test]
    fn reasons_name_their_setting() {
        for (reason, setting, text) in [
            (Reason::Stopped, None, "the search was stopped"),
            (
                Reason::RecursionLimit { depth: 7 },
                Some("limits.recursion_depth"),
                "the recursion limit of 7 was reached",
            ),
            (
                Reason::CopyBound { copies: 3 },
                Some("search.copies"),
                "the copy bound of 3 was reached",
            ),
            (
                Reason::MemoryLimit {
                    limit_bytes: 1 << 20,
                },
                Some("limits.memory_bytes"),
                "the memory limit of 1 MiB was reached",
            ),
            (
                Reason::IndexLimit,
                None,
                "the search outgrew what its indices address",
            ),
            (
                Reason::WorkLimit { limit: 9 },
                Some("limits.work"),
                "the work limit of 9 was reached",
            ),
            (
                Reason::Unchecked { limit_bytes: 1024 },
                Some("limits.memory_bytes"),
                "a proof was found, but its check would hold more than the memory limit of 1 KiB",
            ),
        ] {
            assert_eq!(
                (reason.setting(), reason.to_string().as_str()),
                (setting, text)
            );
        }
    }

    /// Two searches' counters add up, but the memo's peak and the copy
    /// bound reached, which are the larger of the two.
    #[test]
    fn statistics_add_up() {
        let mut first = Statistics {
            nodes: 3,
            memo_hits: 1,
            memo_entries: 7,
            splits: 2,
            links: 1,
            tests: 4,
            copies: 2,
            forward_copies: 9,
            work: 10,
        };
        let second = Statistics {
            nodes: 5,
            memo_hits: 2,
            memo_entries: 4,
            splits: 1,
            links: 3,
            tests: 1,
            copies: 5,
            forward_copies: 0,
            work: 20,
        };
        first.add(&second);
        assert_eq!(
            first,
            Statistics {
                nodes: 8,
                memo_hits: 3,
                memo_entries: 7,
                splits: 3,
                links: 4,
                tests: 5,
                copies: 5,
                forward_copies: 9,
                work: 30,
            }
        );
    }

    /// Unit-free MLL reaches the net engine unless a literal occurs more
    /// than twice, every other classical input without exponentials the
    /// focused engine, and the outcome says which fragment it was searched
    /// in.
    #[test]
    fn dispatch() {
        for (input, fragment, engine) in [
            ("|- a, ~a", Fragment::EMPTY, Engine::Net),
            ("a, a -o b |- b", Fragment::MLL, Engine::Net),
            ("a * a |- a * a", Fragment::MLL, Engine::Net),
            ("a * a * a |- a * a * a", Fragment::MLL, Engine::Focus),
            ("|- 1, bot", Fragment::MULTIPLICATIVE_UNITS, Engine::Focus),
            ("|- a & b, ~a + ~b", Fragment::ADDITIVES, Engine::Additive),
            ("|- top, 0", Fragment::ADDITIVE_UNITS, Engine::Additive),
            ("|- top, a, b", Fragment::ADDITIVE_UNITS, Engine::Focus),
            ("|- (a * top) + 1, ~a, bot", Fragment::MALL, Engine::Focus),
        ] {
            let outcome = prove(&sequent(input), Mode::CLASSICAL, &Options::default()).unwrap();
            assert_eq!(outcome.fragment, fragment, "{input:?}");
            assert_eq!(outcome.engine, engine, "{input:?}");
            assert_eq!(outcome.net.is_some(), engine == Engine::Net, "{input:?}");
            assert!(outcome.verdict.proof().is_some(), "{input:?}");
        }
    }

    /// An unprovable sequent says why where its counts tell: an atom whose
    /// literals cannot pair up, also across additive alternatives, or the
    /// count equation; else that the search was exhaustive.
    #[test]
    fn refutations() {
        let refuted = |input: &str, mode: Mode| {
            let outcome = prove(&sequent(input), mode, &Options::default()).unwrap();
            let Verdict::Unprovable(disproof) = outcome.verdict else {
                panic!("{input:?}: {:?}", outcome.verdict);
            };
            assert_eq!(
                (disproof.sequent(), disproof.mode()),
                (&sequent(input), mode)
            );
            *disproof
        };
        let unbalanced = |least, most| {
            Refutation::Unbalanced(Unbalanced {
                atom: Atom::new(0),
                least,
                most,
            })
        };
        let classical = Mode::CLASSICAL;
        assert_eq!(
            refuted("|- a, a", classical).refutation(),
            &unbalanced(2, 2)
        );
        assert_eq!(
            refuted("a |- b", Mode::INTUITIONISTIC).refutation(),
            &unbalanced(-1, -1)
        );
        let hull = refuted("|- (a * a) + (a * a * a), ~a", classical);
        assert_eq!(hull.refutation(), &unbalanced(1, 2));
        assert_eq!(
            hull.to_string(),
            "a occurs 1 to 2 more times than ~a in the one-sided sequent, whichever additive \
             alternatives a proof takes, so they cannot all meet in axioms"
        );
        assert!(
            hull.refutation()
                .to_string()
                .starts_with("#0 occurs 1 to 2 more times than ~#0")
        );
        let equation = refuted("|- a par b, ~a, ~b", classical);
        assert!(matches!(
            equation.refutation(),
            Refutation::Equation(Equation { needed: 1, .. })
        ));
        assert_eq!(
            equation.to_string(),
            "the count equation fails: a provable one-sided sequent of MLL has exactly #⊗ − #⅋ − \
             #1 + #⊥ + 2 formulas, here 0 − 1 − 0 + 0 + 2 = 1, and this one has 3"
        );
        for input in ["|- a par ~a, b * ~b", "|- a & b, ~a"] {
            assert_eq!(
                refuted(input, classical).refutation(),
                &Refutation::Exhausted,
                "{input:?}"
            );
        }
    }

    /// A goal is a multiset: the roots in another order are the sequent
    /// itself, which the net engine takes, forced or by default.
    #[test]
    fn goal_in_any_order() {
        let s = sequent("a, a -o b |- b");
        let forest = Forest::new(&s).unwrap();
        let goal: Vec<Member> = forest
            .roots()
            .iter()
            .rev()
            .map(|&o| Member::from(o))
            .collect();
        assert_ne!(goal[0], Member::from(forest.roots()[0]));
        assert!(Goal::new(&forest, &goal).unwrap().is_conclusion());
        let classical = Mode::CLASSICAL;
        let outcome = prove_goal(
            Goal::new(&forest, &goal).unwrap(),
            classical,
            &Options::default(),
            &Limits::default(),
            |_| false,
        )
        .unwrap();
        assert_eq!(outcome.engine, Engine::Net);
        assert!(outcome.verdict.proof().is_some());
        let net = Options::default().with_engine(Some(Engine::Net));
        let outcome = prove_goal(
            Goal::new(&forest, &goal).unwrap(),
            classical,
            &net,
            &Limits::default(),
            |_| false,
        )
        .unwrap();
        assert!(outcome.verdict.proof().is_some());
    }

    /// `Options::engine` forces an engine: the focused engine on MLL, the
    /// net engine on MLL with Mix, and the net engine outside MLL is an
    /// error.
    #[test]
    fn engine_override() {
        let s = sequent("|- a * b, ~a par ~b");
        let focus = Options::default().with_engine(Some(Engine::Focus));
        let outcome = prove(&s, Mode::CLASSICAL, &focus).unwrap();
        assert_eq!(outcome.engine, Engine::Focus);
        assert!(outcome.verdict.proof().is_some());
        assert!(outcome.net.is_none());
        let net = Options::default().with_engine(Some(Engine::Net));
        let outcome = prove(
            &sequent("|- a, ~a, b, ~b"),
            Mode::CLASSICAL.with_mix(),
            &net,
        )
        .unwrap();
        assert_eq!(outcome.engine, Engine::Net);
        assert!(outcome.verdict.proof().is_some());
        for (input, options) in [
            ("|- 1", net.clone()),
            ("|- a & b, ~a", net.clone()),
            ("|- a, ~a", net.clone().with_fragment(Some(Fragment::MALL))),
        ] {
            let error = prove(&sequent(input), Mode::CLASSICAL, &options).unwrap_err();
            assert!(
                matches!(
                    error,
                    Error::EngineRefused {
                        engine: Engine::Net,
                        because: NotTaken::Fragment { .. },
                        ..
                    }
                ),
                "{input:?}: {error}"
            );
        }
        assert_eq!(
            prove(&sequent("|- 1"), Mode::CLASSICAL, &net)
                .unwrap_err()
                .to_string(),
            "proof nets exist for MLL without units only, not for MLL with units"
        );
    }

    /// The asserted fragment must contain the sequent's, and is what the
    /// search runs in, which also picks the engine.
    #[test]
    fn fragment_override() {
        let s = sequent("|- a * b, ~a, ~b");
        let outcome = prove(
            &s,
            Mode::CLASSICAL,
            &Options::default().with_fragment(Some(Fragment::MALL)),
        )
        .unwrap();
        assert_eq!(outcome.fragment, Fragment::MALL);
        assert_eq!(outcome.engine, Engine::Focus);
        assert!(outcome.verdict.proof().is_some());
        let error = prove(
            &sequent("|- a & b, ~a"),
            Mode::CLASSICAL,
            &Options::default().with_fragment(Some(Fragment::MLL)),
        )
        .unwrap_err();
        assert!(matches!(
            error,
            Error::FragmentMismatch {
                asserted: Fragment::MLL,
                detected: Fragment::ADDITIVES,
            }
        ));
        assert_eq!(
            error.to_string(),
            "the sequent lies in ALL, outside the asserted fragment MLL"
        );
    }

    /// A Horn program with a clause under `!` goes to the Horn engine in
    /// every mode, with Mix and in affine mode too; the other sequents
    /// with exponentials to the focused engine; and the net engine is
    /// refused in affine mode.
    #[test]
    fn dispatch_by_mode() {
        for (input, mode, fragment, engine) in [
            (
                "a, b |- a",
                Mode::CLASSICAL.with_affine(),
                Fragment::EMPTY,
                Engine::Focus,
            ),
            (
                "!a |- a",
                Mode::CLASSICAL,
                Fragment::EXPONENTIALS,
                Engine::Horn,
            ),
            (
                "!(a -o a * b), a |- a * b * b",
                Mode::CLASSICAL.with_mix(),
                Fragment::MLL | Fragment::EXPONENTIALS,
                Engine::Horn,
            ),
            (
                "a |- !a -o a",
                Mode::CLASSICAL,
                Fragment::MLL | Fragment::EXPONENTIALS,
                Engine::Focus,
            ),
            (
                "!a |- a & a",
                Mode::CLASSICAL.with_mix(),
                Fragment::ADDITIVES | Fragment::EXPONENTIALS,
                Engine::Focus,
            ),
            (
                "!a, b |- a",
                Mode::CLASSICAL.with_affine(),
                Fragment::EXPONENTIALS,
                Engine::Horn,
            ),
            (
                "!a, b |- a & a",
                Mode::CLASSICAL.with_affine(),
                Fragment::ADDITIVES | Fragment::EXPONENTIALS,
                Engine::Focus,
            ),
        ] {
            let outcome = prove(&sequent(input), mode, &Options::default()).unwrap();
            assert_eq!(outcome.engine, engine, "{input:?}");
            assert_eq!(outcome.fragment, fragment, "{input:?}");
            assert!(outcome.verdict.proof().is_some(), "{input:?}");
        }
        let net = Options::default().with_engine(Some(Engine::Net));
        let error = prove(&sequent("a, b |- a"), Mode::CLASSICAL.with_affine(), &net).unwrap_err();
        assert!(matches!(
            error,
            Error::EngineRefused {
                because: NotTaken::Mode { .. },
                ..
            }
        ));
        assert_eq!(
            error.to_string(),
            "proof nets exist in linear mode only, with or without Mix, not in classical affine mode"
        );
    }

    /// Intuitionistic mode: IMLL without units goes to the net engine by
    /// the embedding, a Horn program with a clause under `!` to the Horn
    /// engine, everything else to the two-sided engine; a sequent
    /// with no intuitionistic reading, Mix, and an engine forced for the
    /// other mode are errors.
    #[test]
    fn dispatch_intuitionistic() {
        let i = Mode::INTUITIONISTIC;
        for (input, fragment, engine) in [
            ("a, a -o b |- b", Fragment::MLL, Engine::Net),
            ("a * a * a |- a * a * a", Fragment::MLL, Engine::TwoSided),
            ("1 |- 1", Fragment::MULTIPLICATIVE_UNITS, Engine::TwoSided),
            ("a & b |- a", Fragment::ADDITIVES, Engine::Additive),
            ("a & b, 0 |- a", Fragment::ADDITIVE, Engine::TwoSided),
            (
                "!a |- a * a",
                Fragment::MLL | Fragment::EXPONENTIALS,
                Engine::Horn,
            ),
            ("!a |- !a", Fragment::EXPONENTIALS, Engine::TwoSided),
        ] {
            let outcome = prove(&sequent(input), i, &Options::default()).unwrap();
            assert_eq!(outcome.fragment, fragment, "{input:?}");
            assert_eq!(outcome.engine, engine, "{input:?}");
            let proof = outcome
                .verdict
                .proof()
                .unwrap_or_else(|| panic!("{input:?}"));
            assert_eq!(proof.check(i), Ok(()), "{input:?}");
        }
        let outcome = prove(&sequent("a, b |- a"), i.with_affine(), &Options::default()).unwrap();
        assert_eq!(outcome.engine, Engine::TwoSided);
        assert!(outcome.verdict.proof().is_some());

        let error = prove(&sequent("|- a par b"), i, &Options::default()).unwrap_err();
        assert!(matches!(error, Error::NotIntuitionistic(_)));
        assert_eq!(
            error.to_string(),
            "not an intuitionistic sequent: subformula 0 is neither an intuitionistic formula nor \
             the negation of one (⅋ only as A ⊸ B, that is ~A ⅋ B, and ? only under a negation)"
        );
        let error = prove(&sequent("a |- a"), i.with_mix(), &Options::default()).unwrap_err();
        assert_eq!(
            error.to_string(),
            "Mix has no intuitionistic form: a premise of a Mix would have no goal"
        );
        let focus = Options::default().with_engine(Some(Engine::Focus));
        let error = prove(&sequent("a |- a"), i, &focus).unwrap_err();
        assert_eq!(
            error.to_string(),
            "the focus engine does not search in intuitionistic mode"
        );
        let two_sided = Options::default().with_engine(Some(Engine::TwoSided));
        let error = prove(&sequent("a |- a"), Mode::CLASSICAL, &two_sided).unwrap_err();
        assert!(matches!(
            error,
            Error::EngineRefused {
                because: NotTaken::Mode { .. },
                ..
            }
        ));
        let outcome = prove(&sequent("a, a -o b |- b"), i, &two_sided).unwrap();
        assert_eq!(outcome.engine, Engine::TwoSided);
        assert!(outcome.verdict.proof().is_some());
        let additive = Options::default().with_engine(Some(Engine::Additive));
        let error = prove(&sequent("a & b, c |- a"), i, &additive).unwrap_err();
        assert!(matches!(
            error,
            Error::EngineRefused {
                engine: Engine::Additive,
                ..
            }
        ));
        assert_eq!(
            error.to_string(),
            "the additive engine decides a sequent of two additive-only formulas only"
        );
    }

    /// IMLL by embedding: on generated intuitionistic sequents over `⊗` and
    /// `⊸` and their mutants, where the dispatch picks the net engine on the
    /// one-sided sequent, the two-sided engine gives the same verdict, and
    /// every net-engine proof passes the intuitionistic checker (every
    /// sequent of a cut-free MLL proof of an intuitionistic sequent has one
    /// goal).
    #[test]
    fn embedding_agrees_with_the_two_sided_engine() {
        use crate::search::generate::{self, IllRules, Rng};
        let i = Mode::INTUITIONISTIC;
        let rules = IllRules {
            units: false,
            additives: false,
            zero: false,
            exponentials: false,
        };
        let mut rng = Rng::new(7);
        let (mut compared, mut provable) = (0, 0);
        for _ in 0..200 {
            let budget = 2 + rng.below(8);
            let generate::Ill {
                mut hypotheses,
                goal,
                ..
            } = generate::ill(&mut rng, rules, 3, budget);
            hypotheses.push(goal);
            for mutated in [false, true] {
                if mutated && !generate::mutate(&mut rng, &mut hypotheses, 3) {
                    continue;
                }
                let goal = hypotheses.last().unwrap();
                let text = generate::two_sided(&hypotheses[..hypotheses.len() - 1], goal);
                let s = sequent(&text);
                let by_net = prove(&s, i, &Options::default()).unwrap();
                if by_net.engine != Engine::Net {
                    // Repeated literals: the dispatch keeps the net engine
                    // off them.
                    continue;
                }
                let two_sided = Options::default().with_engine(Some(Engine::TwoSided));
                let by_two_sided = prove(&s, i, &two_sided).unwrap();
                assert_eq!(by_two_sided.engine, Engine::TwoSided);
                let verdict = |outcome: Outcome| match outcome.verdict {
                    Verdict::Proved(proof) => {
                        assert_eq!(proof.check(i), Ok(()), "{text:?} by {}", outcome.engine);
                        assert_eq!(outcome.net.is_some(), outcome.engine == Engine::Net);
                        true
                    }
                    Verdict::Unprovable(_) => false,
                    Verdict::Unknown(reason) => panic!("{text:?} by {}: {reason}", outcome.engine),
                };
                let (net, focus) = (verdict(by_net), verdict(by_two_sided));
                assert_eq!(net, focus, "{text:?}: net {net}, two-sided {focus}");
                assert!(mutated || net, "{text:?} is provable");
                compared += 1;
                provable += usize::from(net);
            }
        }
        assert!(
            compared > 100 && provable > 50 && provable < compared,
            "{provable} of {compared}"
        );
    }

    /// A goal below the roots is decided in its own fragment by the focused
    /// engine, two additive-only occurrences by the additive path, and the
    /// two-sided engine in intuitionistic mode; the net engine is refused
    /// off the roots, and an intuitionistic goal must have one output.
    #[test]
    fn goals() {
        let o = |ids: &[u32]| ids.iter().map(|&i| Member::new(i)).collect::<Vec<_>>();
        // 0: ~a, 1: (a ⊗ ~b) ⊗ ?c, 2: a ⊗ ~b, 3: a, 4: ~b, 5: ?c, 6: c, 7: b.
        let s = sequent("|- ~a, (a * ~b) * ?c, b");
        let forest = Forest::new(&s).unwrap();
        let options = Options::default();
        let goal = o(&[0, 3]);
        let outcome = prove_goal(
            Goal::new(&forest, &goal).unwrap(),
            Mode::CLASSICAL,
            &options,
            &Limits::default(),
            |_| false,
        )
        .unwrap();
        assert_eq!(outcome.engine, Engine::Focus);
        assert_eq!(outcome.fragment, Fragment::EMPTY);
        let proof = outcome.verdict.proof().unwrap();
        // A goal proof is checked against the goal it records.
        assert!(proof.goal().is_some());
        assert_eq!(proof.check(Mode::CLASSICAL), Ok(()));
        let outcome = prove_goal(
            Goal::new(&forest, &o(&[0, 2, 7])).unwrap(),
            Mode::CLASSICAL,
            &options,
            &Limits::default(),
            |_| false,
        )
        .unwrap();
        assert!(outcome.verdict.proof().is_some());
        assert_eq!(outcome.fragment, Fragment::MLL);
        let outcome = prove_goal(
            Goal::new(&forest, &o(&[4, 5])).unwrap(),
            Mode::CLASSICAL,
            &options,
            &Limits::default(),
            |_| false,
        )
        .unwrap();
        assert!(
            matches!(outcome.verdict, Verdict::Unprovable(_)),
            "{:?}",
            outcome.verdict
        );
        assert_eq!(outcome.fragment, Fragment::EXPONENTIALS);
        let net = Options::default().with_engine(Some(Engine::Net));
        let error = prove_goal(
            Goal::new(&forest, &goal).unwrap(),
            Mode::CLASSICAL,
            &net,
            &Limits::default(),
            |_| false,
        )
        .unwrap_err();
        assert!(
            matches!(
                error,
                Error::EngineRefused {
                    because: NotTaken::Goal,
                    ..
                }
            ),
            "{error}"
        );
        let error = Goal::new(&forest, &o(&[9])).unwrap_err();
        assert!(
            matches!(
                error,
                Error::IndexOutOfBounds {
                    space: crate::limits::Space::Occurrence,
                    index: 9,
                    len: 8
                }
            ),
            "{error}"
        );

        // 0: (~a & ~b) ⅋ (a & b), 1: ~a & ~b, 2: ~a, 3: ~b, 4: a & b, 5: a,
        // 6: b: the additive path on the pair below the ⅋.
        let s = sequent("|- (~a & ~b) par (a & b)");
        let forest = Forest::new(&s).unwrap();
        let outcome = prove_goal(
            Goal::new(&forest, &o(&[1, 4])).unwrap(),
            Mode::CLASSICAL,
            &options,
            &Limits::default(),
            |_| false,
        )
        .unwrap();
        assert_eq!(outcome.engine, Engine::Additive);
        assert!(matches!(outcome.verdict, Verdict::Unprovable(_)));
        let outcome = prove_goal(
            Goal::new(&forest, &o(&[5, 2])).unwrap(),
            Mode::CLASSICAL,
            &options,
            &Limits::default(),
            |_| false,
        )
        .unwrap();
        assert_eq!(outcome.engine, Engine::Focus);
        assert!(outcome.verdict.proof().is_some());

        // 0: ~a, 1: a ⊗ ~b, 2: a, 3: ~b, 4: b, read as a, a ⊸ b ⊢ b: the
        // goal ⊢ ~a, a is a ⊢ a two-sided; ⊢ ~a alone has no output and
        // ⊢ a, b two.
        let s = sequent("a, a -o b |- b");
        let forest = Forest::new(&s).unwrap();
        let i = Mode::INTUITIONISTIC;
        let outcome = prove_goal(
            Goal::new(&forest, &o(&[0, 2])).unwrap(),
            i,
            &options,
            &Limits::default(),
            |_| false,
        )
        .unwrap();
        assert_eq!(outcome.engine, Engine::TwoSided);
        assert!(outcome.verdict.proof().is_some());
        for (goal, outputs) in [(o(&[0]), 0), (o(&[2, 4]), 2)] {
            let error = prove_goal(
                Goal::new(&forest, &goal).unwrap(),
                i,
                &options,
                &Limits::default(),
                |_| false,
            )
            .unwrap_err();
            assert!(
                matches!(error, Error::GoalOutputs { count: n } if n == outputs),
                "{error}"
            );
        }
    }

    /// A proof of a goal records the goal and the mode it was found in,
    /// passes the checker against the goal, and is refused where a proof
    /// of the sequent is needed.
    #[test]
    fn a_goal_proof_records_its_goal() {
        // 0: a ⊗ b, 1: a, 2: b, 3: ~a, 4: ~b, 5: c, 6: ~c
        let sequent: Sequent = "|- a * b, ~a, ~b, c, ~c".parse().unwrap();
        let forest = Forest::new(&sequent).unwrap();
        let goal = [Member::new(6), Member::new(5)];
        let mode = Mode::CLASSICAL;
        let outcome = prove_goal(
            Goal::new(&forest, &goal).unwrap(),
            mode,
            &Options::default(),
            &Limits::default(),
            |_| false,
        );
        let Verdict::Proved(proof) = outcome.unwrap().verdict else {
            panic!("⊢ ~c, c is provable");
        };
        let members = [crate::Member::new(6), crate::Member::new(5)];
        assert_eq!(
            (proof.goal(), proof.mode()),
            (Some(&members[..]), Some(mode))
        );
        assert_eq!(proof.check(mode), Ok(()));
        assert!(matches!(
            ProofStructure::from_proof(&proof, crate::Criterion::MLL, &Limits::default(), |_| {
                false
            }),
            Err(Error::GoalProof)
        ));
        #[cfg(feature = "rocq")]
        {
            let derivation = proof.derivation().unwrap();
            let options = crate::export::rocq::Options::default();
            let written =
                crate::export::rocq::write(&derivation, &options, &mut String::new(), |_| false);
            assert!(matches!(written, Err(Error::GoalProof)));
        }
    }

    /// A thread count is taken as at least one and at most
    /// [`Options::MAX_JOBS`], whatever is asked for.
    #[test]
    fn jobs_are_bounded() {
        assert_eq!(Jobs::Count(usize::MAX).count(), Options::MAX_JOBS);
        assert_eq!(Jobs::Count(0).count(), 1);
        assert!((1..=Options::MAX_JOBS).contains(&Jobs::Auto.count()));
        assert_eq!(Options::default().with_jobs(2).threads(), 2);
    }

    /// The stop condition ends the search with `Unknown`.
    #[test]
    fn stop() {
        let outcome = prove_within(
            &sequent("|- a * b, ~a, ~b"),
            Mode::CLASSICAL,
            &Options::default(),
            &Limits::default(),
            |_| true,
        )
        .unwrap();
        assert!(matches!(outcome.verdict, Verdict::Unknown(Reason::Stopped)));
        assert_eq!(Reason::Stopped.to_string(), "the search was stopped");
    }
}
