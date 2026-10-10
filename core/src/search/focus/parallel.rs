// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The focused engine on several threads: cube-and-conquer over the
//! choices near the root, and-parallel `&` premises, and one memo and one
//! proof arena shared by every worker.
//!
//! A choice among alternatives (the focus on a stable sequent, the side
//! of a `⊕`, the splits of a `⊗`) within the first [`LEVELS`] such
//! choices of a branch runs its alternatives as tasks of the pool: the
//! engine that meets the choice spawns a worker for each alternative but
//! the first, a copy of its own state at that point (the branch stack,
//! the nesting depth, the budget) with pools of its own, which is the
//! per-worker scratch, and runs the first alternative on one more worker
//! on its own thread; the first alternative to succeed raises the
//! choice's flag, which every worker of the choice polls, and the others
//! stop at their next stable sequent. Below those levels every worker runs the sequential
//! engine, so the tasks are few and large. The `&` rule within the same
//! levels runs its two premises the same way, the first to fail
//! cancelling the other. The levels of the copy bound never overlap:
//! `run` deepens on the root engine, which is alone until its first
//! choice, and every task of a level has ended when the level's answer is
//! read.

use super::arena::{Arena, Kept};
use super::classes::Classes;
use super::context::Context;
use super::counts::{Counts, Split};
use super::memo::{Key, Shared, Table};
use super::schedule::{Rule, merged};
use super::split::Join;
use super::{Alternative, Cuts, Engine, Found, Problem, Search, Step};
use crate::Error;
use crate::fragment::{Fragment, Mode};
use crate::limits::Limits;
use crate::occurrences::{Forest, OccId, OccSet, Reading};
use crate::proofs::{Node, NodeId};
use crate::search::memory::Account;
use crate::search::parallel::{Flags, Lent, RaiseOnPanic, Runtime};
use crate::search::{Options, Reason, Statistics, Stop, Work, set_up_stopped};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

/// How many choices of a branch, from the root, run their alternatives on
/// several threads.
const LEVELS: u32 = 2;

/// The most members of a context whose assignment a split fixes per
/// task when the splits run on several threads: 2⁶ tasks at most.
const MAX_FIXED: usize = 6;

/// Runs the focused engine on `Options::jobs` threads from a goal, as
/// [`super::search_goal`] does on one thread, polling `stop` on the
/// calling thread while the threads search. Returns the node proving the
/// goal, the arena it lives in and the statistics of every worker
/// together. Where the default bias takes two searches they run side by
/// side, each on a pool of its own with half the threads, the first to
/// decide stopping the other.
///
/// # Errors
///
/// [`Error::ThreadPool`] when a pool's threads cannot start.
#[allow(clippy::too_many_arguments)]
pub(crate) fn search_goal(
    forest: &Forest,
    goal: &[OccId],
    fragment: Fragment,
    mode: Mode,
    reading: Option<&Reading>,
    options: &Options,
    limits: &Limits,
    account: &Account,
    work: &Work,
    stop: &mut dyn FnMut(u64) -> bool,
) -> Result<(Search, Vec<Node>, Statistics), Error> {
    // On a large forest the passes of the set-up are followed by a poll;
    // from the pool's start the driver polls.
    let stopped = || Ok((Err(Reason::Stopped), Vec::new(), Statistics::default()));
    let classes = Classes::new(forest, reading);
    if set_up_stopped(forest, stop) {
        return stopped();
    }
    let stack = limits.stack_bytes();
    let (first, second) = super::plan(forest, goal, fragment, mode, options);
    if set_up_stopped(forest, stop) {
        return stopped();
    }
    let search = |rule: Rule, runtime: &Runtime, account: &Account, flags: Flags<'_>| {
        account.charge(classes.bytes());
        rule.search_on(
            forest, goal, fragment, mode, reading, &classes, options, limits, account, runtime,
            flags,
        )
    };
    let Some(second) = second else {
        let runtime = Lent::take(options.pool.as_ref(), options.threads(), stack)?;
        let (result, nodes, statistics) =
            runtime.drive(stop, work, |flags| search(first, &runtime, account, flags));
        let result = result.map_err(|r| super::reason(r, options, limits));
        return Ok((result, nodes, statistics));
    };
    // Two pools, so that no thread of one search is ever busy with a task
    // of the other when its own search has decided.
    let threads = options.threads() / 2;
    let runtimes = (
        Lent::take(options.pool.as_ref(), threads, stack)?,
        Lent::take(options.pool.as_ref(), options.threads() - threads, stack)?,
    );
    // Each search has half the memory, as each has its own memo.
    let accounts = (account.share(2), account.share(2));
    let (forward, backward) = crate::search::parallel::race(
        (&runtimes.0, &runtimes.1),
        stop,
        work,
        (
            |flags: Flags<'_>| search(first, &runtimes.0, &accounts.0, flags),
            |flags: Flags<'_>| search(second, &runtimes.1, &accounts.1, flags),
        ),
        |(result, _, _)| result.is_ok(),
    );
    Ok(merged(forward, backward, options, limits))
}

impl Rule {
    /// Runs the search of this rule on a pool, from one of its threads,
    /// with a memo and an arena of its own that its workers share, stopped
    /// by the flags; on a pool of one thread it is the sequential engine.
    /// Returns what [`Rule::search`] does.
    #[allow(clippy::too_many_arguments)]
    fn search_on(
        self,
        forest: &Forest,
        goal: &[OccId],
        fragment: Fragment,
        mode: Mode,
        reading: Option<&Reading>,
        classes: &Classes,
        options: &Options,
        limits: &Limits,
        account: &Account,
        runtime: &Runtime,
        flags: Flags<'_>,
    ) -> (Search, Vec<Node>, Statistics) {
        // The longest pass of the set-up reads the flags too.
        let counts = match Counts::new_until(forest, self.bias, account, &mut |_| flags.raised()) {
            Ok(counts) => counts,
            Err(reason) => return (Err(reason), Vec::new(), Statistics::default()),
        };
        let memo = Shared::new(options.memo_entries());
        let arena = Mutex::new(Vec::new());
        let (result, statistics) = {
            let problem = Problem::new(
                forest,
                reading,
                (&counts, classes),
                fragment,
                mode,
                options,
                limits,
                self.copies,
                account,
            );
            let mut engine = Engine::new(
                problem,
                Stop::Flags(flags),
                Table::Shared(&memo),
                Arena::new(Kept::Shared(&arena), account),
            );
            engine.runtime = (runtime.threads() > 1).then_some(runtime);
            let result = engine
                .run(goal)
                .and_then(|root| root.map(|root| engine.nodes.keep(0, root)).transpose());
            (result, engine.statistics())
        };
        let nodes = arena
            .into_inner()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        (result, nodes, statistics)
    }
}

/// What a worker starts from: the problem of the engine that spawns it at
/// a parallel choice, the shared parts by reference and the branch's by
/// copy.
struct Spawn<'s> {
    /// The problem.
    problem: Problem<'s>,
    /// The shared memo.
    memo: &'s Shared,
    /// The shared arena.
    arena: &'s Mutex<Vec<Node>>,
    /// The runtime.
    runtime: &'s Runtime,
    /// The spawning engine's stop flags, which the worker's chain to.
    flags: Flags<'s>,
    /// The branch at the choice: the spawning engine's ancestors and its
    /// live stack, with their hashes, read in place.
    branch: Vec<(&'s [Key], &'s [u64])>,
    /// The nesting of engine calls at the choice.
    depth: u32,
    /// The workers' levels of cube-and-conquer.
    or_depth: u32,
}

impl<'s> Spawn<'s> {
    /// Starts a worker: a fresh engine on the spawn's problem that
    /// continues its branch, stopped by the spawn's flags or by `cancel`.
    fn worker<'w>(&'w self, cancel: &'w AtomicBool) -> Engine<'w> {
        let mut worker = Engine::new(
            self.problem,
            Stop::Flags(self.flags.child(cancel)),
            Table::Shared(self.memo),
            Arena::new(Kept::Shared(self.arena), self.problem.account),
        );
        worker.runtime = Some(self.runtime);
        // A task stolen by a thread that waits at a scope runs on that
        // scope's frames: its recursion counts on top of them.
        worker.depth = self.depth.max(self.runtime.depth_here());
        worker.or_depth = self.or_depth;
        worker.ancestors = self.branch.clone();
        worker
    }
}

/// What the workers of a choice among alternatives report.
struct Collected {
    /// The first proof found.
    proof: Option<NodeId>,
    /// The first error, a stop that a cancellation caused excepted, a
    /// stop giving way to any other reason.
    error: Option<Reason>,
    /// What the failed alternatives cut.
    cuts: Cuts,
    /// The workers' counters.
    statistics: Statistics,
}

impl Collected {
    /// Nothing reported yet.
    fn new() -> Self {
        Self {
            proof: None,
            error: None,
            cuts: Cuts::NONE,
            statistics: Statistics::default(),
        }
    }

    /// Takes a worker's result: a proof or an error settles the choice
    /// and raises its flag, a failure adds its cuts.
    fn take(&mut self, result: Step, worker: &Engine<'_>, cancel: &AtomicBool) {
        self.statistics.add_run(&worker.statistics);
        match result {
            Ok(Found::Proved(node)) => {
                self.proof.get_or_insert(node);
                cancel.store(true, Ordering::Relaxed);
            }
            Ok(Found::Failed(failed)) => self.cuts = self.cuts.and(failed),
            Err(Reason::Stopped) if cancel.load(Ordering::Relaxed) => {}
            Err(reason) => {
                if self.error.is_none_or(|old| old == Reason::Stopped) {
                    self.error = Some(reason);
                }
                cancel.store(true, Ordering::Relaxed);
            }
        }
    }

    /// Takes an alternative that was never started because a flag of its
    /// chain was raised: the choice's own, which is raised only once a
    /// proof or an error is recorded (a skip's stop included), or an
    /// ancestor's, a stop that the choice records as its error, since
    /// what was never searched cannot have failed.
    fn skip(&mut self, cancel: &AtomicBool) {
        if cancel.load(Ordering::Relaxed) {
            return;
        }
        if self.error.is_none() {
            self.error = Some(Reason::Stopped);
        }
        cancel.store(true, Ordering::Relaxed);
    }
}

/// A premise of a `&` searched on a worker.
struct Premise {
    /// What the search found, with its cuts.
    result: Step,
    /// The worker's counters.
    statistics: Statistics,
}

impl Premise {
    /// What a worker found on a premise, with its counters.
    fn of(worker: &Engine<'_>, result: Step) -> Self {
        Self {
            result,
            statistics: worker.statistics,
        }
    }
}

impl<'a> Engine<'a> {
    /// Whether the next choice of the branch runs on several threads.
    pub(super) fn cubes(&self) -> bool {
        self.runtime.is_some() && self.or_depth < LEVELS
    }

    /// The state a worker starts from, with `or_depth` levels of
    /// cube-and-conquer above it. The branch is read in place, so the
    /// engine's own stack stays as it is while the workers run.
    fn spawn<'s>(&'s self, or_depth: u32) -> Spawn<'s>
    where
        'a: 's,
    {
        let (Table::Shared(memo), Kept::Shared(arena), Stop::Flags(flags), Some(runtime)) =
            (&self.memo, &self.nodes.kept, &self.stop, self.runtime)
        else {
            unreachable!("a parallel choice is met on a worker of the pool")
        };
        Spawn {
            problem: self.problem(),
            memo,
            arena,
            runtime,
            flags: *flags,
            branch: self
                .ancestors
                .iter()
                .copied()
                .chain([(
                    &self.stack[..self.stack_len],
                    &self.hashes[..self.stack_len],
                )])
                .collect(),
            depth: self.depth,
            or_depth,
        }
    }

    /// A result as it leaves this engine for another: the proof's pending
    /// nodes kept, since a pending id means nothing outside its engine.
    fn exported(&mut self, result: Step) -> Step {
        match result? {
            Found::Proved(node) => Ok(Found::proved(self.nodes.keep(0, node)?)),
            failed => Ok(failed),
        }
    }

    /// Locks what the workers report.
    fn lock<T>(shared: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
        shared
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Searches one alternative of a choice on a worker of its own and
    /// returns what leaves it.
    fn run_alternative(
        &mut self,
        theta: &OccSet,
        gamma: &Context,
        alternative: Alternative<'_>,
        budget: u32,
    ) -> Step {
        let mut rest = None;
        let result = self.alternative(theta, gamma, alternative, &mut rest, budget);
        if let Some(rest) = rest {
            self.give_context(rest);
        }
        self.exported(result)
    }

    /// Decides a choice among alternatives on the pool: a worker per
    /// alternative, the first one on this thread, and the first to
    /// succeed or to fail with an error cancels the rest. Returns the
    /// proof, or nothing when every alternative failed, with their cuts,
    /// or the first error (a stop a cancellation caused is none). A proof
    /// found by any alternative wins over an error of another, so the
    /// pool may decide where one thread gives up.
    pub(super) fn choose_parallel(
        &mut self,
        theta: &OccSet,
        gamma: &Context,
        alternatives: &[Alternative<'_>],
        budget: u32,
    ) -> Step {
        let cancel = AtomicBool::new(false);
        let spawn = self.spawn(self.or_depth + 1);
        let collected = Mutex::new(Collected::new());
        let (&first, rest) = alternatives
            .split_first()
            .expect("a choice has an alternative");
        let waiting = spawn.runtime.waiting(self.depth);
        spawn.runtime.pool.in_place_scope(|scope| {
            for &alternative in rest {
                let (spawn, cancel, collected) = (&spawn, &cancel, &collected);
                scope.spawn(move |_| {
                    // A task the pool reaches after its choice was settled
                    // or a flag above it was raised (the caller's stop, a
                    // proof at an enclosing choice, the other premise of
                    // a `&`) ends before it builds a worker, which copies
                    // the branch's stack of keys: a stable sequent of a
                    // Petri net queues a task per transition, and each
                    // copy is the forest's width times the depth.
                    if spawn.flags.child(cancel).raised() {
                        Self::lock(collected).skip(cancel);
                        return;
                    }
                    let _cancel = RaiseOnPanic(cancel);
                    let mut worker = spawn.worker(cancel);
                    let result = worker.run_alternative(theta, gamma, alternative, budget);
                    Self::lock(collected).take(result, &worker, cancel);
                });
            }
            // The first alternative on this thread, on a worker of its
            // own so that it polls the choice's flag like the others; a
            // panic in any alternative cancels the rest.
            let _cancel = RaiseOnPanic(&cancel);
            let mut worker = spawn.worker(&cancel);
            let result = worker.run_alternative(theta, gamma, first, budget);
            Self::lock(&collected).take(result, &worker, &cancel);
        });
        drop(waiting);
        let collected = collected
            .into_inner()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        self.statistics.add_run(&collected.statistics);
        match (collected.proof, collected.error) {
            (Some(node), _) => Ok(Found::proved(node)),
            (None, Some(reason)) => Err(reason),
            (None, None) => Ok(Found::failed(collected.cuts)),
        }
    }

    /// The free splits of `F = A ⊗ B` on the pool: the assignments of the
    /// first few members of the context, as many as give the pool twice
    /// its threads in tasks (at most [`MAX_FIXED`]), as alternatives, each
    /// searching the assignments of the rest. The sides and counts given
    /// are the state before any member moved.
    pub(super) fn split_parallel(
        &mut self,
        theta: &OccSet,
        members: &[OccId],
        sides: (&Context, &Context),
        split: &Split,
        join: Join,
        budget: u32,
    ) -> Step {
        let threads = self.runtime.map_or(1, Runtime::threads);
        let bits = (2 * threads).next_power_of_two().trailing_zeros() as usize;
        let fixed = bits.clamp(1, MAX_FIXED).min(members.len());
        // Of interchangeable members the lowest ids go left, and they come
        // last among their like: a pattern that sends one left and the
        // next one right is none of the splits searched.
        let canonical = |pattern: u64| {
            (1..fixed).all(|i| {
                pattern >> (i - 1) & 1 == 0
                    || pattern >> i & 1 == 1
                    || !self.classes.same(members[i - 1], members[i])
            })
        };
        let alternatives: Vec<Alternative<'_>> = (0..1u64 << fixed)
            .filter(|&pattern| canonical(pattern))
            .map(|pattern| Alternative::Splits {
                join,
                members,
                fixed,
                pattern,
                sides,
                split,
            })
            .collect();
        self.choose_parallel(theta, sides.1, &alternatives, budget)
    }

    /// The `&` rule on the pool: the left premise on a worker on this
    /// thread, the right one on a worker of the pool, the first to fail
    /// cancelling the other, and the left one that gives up the right; a
    /// failure of either is the rule's, else a premise's reason for giving
    /// up, a stop giving way to any other.
    /// An engine that is stopped already starts neither.
    pub(super) fn with_parallel(
        &mut self,
        theta: &OccSet,
        gamma: &Context,
        list: &[OccId],
        o: OccId,
        budget: u32,
    ) -> Step {
        // The asynchronous phase polls nowhere else before its stable
        // sequents: without this poll a premise that is stopped already
        // would still start both premises of every `&` below it, two to
        // the number of them before the first stable sequent ends one.
        if self.stop.fired(0) {
            return Err(Reason::Stopped);
        }
        let cancel = AtomicBool::new(false);
        let spawn = self.spawn(self.or_depth + 1);
        let premise: Mutex<Option<Premise>> = Mutex::new(None);
        let (left_sub, right_sub) = (self.forest.left(o).unwrap(), self.forest.right(o).unwrap());
        // A premise on a worker, which cancels the other when it fails:
        // the rule fails then, whatever the other finds. A left premise
        // that gives up cancels the right one too, which one thread never
        // starts then; a right one that gives up leaves the left to run,
        // since its failure still decides the rule, as on one thread.
        let search = |worker: &mut Engine<'_>, sub: OccId, left: bool| {
            // A premise that panics cancels the other.
            let _cancel = RaiseOnPanic(&cancel);
            let result = worker.premise(theta, gamma, list, sub, budget);
            let result = worker.exported(result);
            if matches!(result, Ok(Found::Failed(_))) || (left && result.is_err()) {
                cancel.store(true, Ordering::Relaxed);
            }
            result
        };
        let waiting = spawn.runtime.waiting(self.depth);
        let left = spawn.runtime.pool.in_place_scope(|scope| {
            let (spawn, cancel, premise, search) = (&spawn, &cancel, &premise, &search);
            scope.spawn(move |_| {
                let mut worker = spawn.worker(cancel);
                let result = search(&mut worker, right_sub, false);
                *Self::lock(premise) = Some(Premise::of(&worker, result));
            });
            let mut worker = spawn.worker(cancel);
            let result = search(&mut worker, left_sub, true);
            Premise::of(&worker, result)
        });
        drop(waiting);
        let right = premise
            .into_inner()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .expect("the worker reports before the scope ends");
        self.statistics.add_run(&left.statistics);
        self.statistics.add_run(&right.statistics);
        match (left.result, right.result) {
            (Ok(Found::Proved(l)), Ok(right @ Found::Proved(_))) => {
                Ok(self.both(o, l, right, self.nodes.mark()))
            }
            (Ok(failed @ Found::Failed(_)), _) | (_, Ok(failed @ Found::Failed(_))) => Ok(failed),
            // A premise that the other's error cancelled reports a stop,
            // which is not the reason: the error is.
            (Err(Reason::Stopped), Err(reason)) | (Err(reason), _) | (Ok(_), Err(reason)) => {
                Err(reason)
            }
        }
    }
}

#[cfg(all(test, feature = "parse"))]
mod tests {
    use crate::fragment::Mode;
    use crate::search::generate::{self, IllRules, Rng, Rules};
    use crate::search::{Engine, Options, Reason, Verdict, prove, prove_within};
    use crate::sequents::Sequent;

    /// The verdict of `input` under `mode` with `options` as a three-way
    /// value: provable, unprovable, or undecided within the copy bound;
    /// every proof checked.
    fn decided(input: &str, mode: Mode, options: &Options) -> Option<bool> {
        let sequent: Sequent = input.parse().unwrap();
        let outcome = prove(&sequent, mode, options).unwrap();
        match outcome.verdict {
            Verdict::Proved(proof) => {
                assert_eq!(proof.check(mode), Ok(()), "{input:?} with {options:?}");
                Some(true)
            }
            Verdict::Unprovable(_) => Some(false),
            Verdict::Unknown(Reason::CopyBound { .. }) => None,
            Verdict::Unknown(reason) => panic!("{input:?}: {reason}"),
        }
    }

    /// The verdicts on two and four threads never contradict the
    /// sequential one: a pool searches the same levels of the copy bound
    /// to their end, so what one proves the other does not refute. Which
    /// of the two decides within the bound may differ either way, as it
    /// does with the memo's contents.
    fn agree(text: &str, mode: Mode, options: &Options) {
        let sequential = decided(text, mode, options);
        for jobs in [2, 4] {
            let options = options.clone().with_jobs(jobs);
            let parallel = decided(text, mode, &options);
            assert!(
                sequential.is_none() || parallel.is_none() || sequential == parallel,
                "{text:?} in {mode} mode: {sequential:?} on one thread, {parallel:?} with {options:?}"
            );
        }
    }

    /// Generated classical sequents and their mutants, in every rule set,
    /// with the engine the dispatch picks and with the focused engine
    /// forced, so that unit-free MLL goes to both engines' pools.
    #[test]
    fn classical_verdicts_agree() {
        for (i, rules) in Rules::ALL.into_iter().enumerate() {
            let mode = if rules.mix {
                Mode::CLASSICAL.with_mix()
            } else {
                Mode::CLASSICAL
            };
            let mut rng = Rng::new(100 + i as u64);
            for _ in 0..10 {
                let budget = 2 + rng.below(9);
                let generate::Provable {
                    mut formulas,
                    copies,
                } = generate::provable(&mut rng, rules, 3, budget);
                let texts = [
                    generate::sequent(&formulas),
                    generate::sequent(if generate::mutate(&mut rng, &mut formulas, 3) {
                        &formulas
                    } else {
                        &[]
                    }),
                ];
                for text in texts.iter().filter(|t| !t.is_empty()) {
                    for engine in [None, Some(Engine::Focus)] {
                        let options = Options::default()
                            .with_copies(Some(copies))
                            .with_engine(engine);
                        agree(text, mode, &options);
                    }
                }
            }
        }
    }

    /// Generated intuitionistic sequents and their mutants, in every rule
    /// set, linear and affine.
    #[test]
    fn intuitionistic_verdicts_agree() {
        for (i, rules) in IllRules::ALL.into_iter().enumerate() {
            let mut rng = Rng::new(200 + i as u64);
            for _ in 0..8 {
                let budget = 2 + rng.below(9);
                let generate::Ill {
                    mut hypotheses,
                    goal,
                    copies,
                } = generate::ill(&mut rng, rules, 3, budget);
                let mut texts = vec![generate::two_sided(&hypotheses, &goal)];
                if generate::mutate(&mut rng, &mut hypotheses, 3) {
                    texts.push(generate::two_sided(&hypotheses, &goal));
                }
                for text in &texts {
                    let options = Options::default().with_copies(Some(copies));
                    agree(text, Mode::INTUITIONISTIC, &options);
                    agree(text, Mode::INTUITIONISTIC.with_affine(), &options);
                }
            }
        }
    }

    /// Eleven tensor pairs under Mix, whose refutation takes minutes: the
    /// caller's stop condition, polled on the calling thread, stops every
    /// worker, inside a search for splits too.
    #[test]
    fn stops() {
        let sequent = crate::families::mix(11);
        let options = Options::default().with_jobs(4);
        let mut polls = 0;
        let outcome = prove_within(
            &sequent,
            Mode::CLASSICAL.with_mix(),
            &options,
            &crate::Limits::default(),
            |_| {
                polls += 1;
                polls > 20
            },
        )
        .unwrap();
        assert!(
            matches!(outcome.verdict, Verdict::Unknown(Reason::Stopped)),
            "{:?}",
            outcome.verdict
        );
        assert!(outcome.statistics.nodes > 0);

        // A split search that visits no stable sequent stops as well.
        let input = format!(
            "|- p * q, 0 * (~p par ~p), 0 * (~q par ~q), {}",
            crate::search::focus::tests::wide_context()
        );
        let sequent: Sequent = input.parse().unwrap();
        let mut polls = 0;
        let outcome = prove_within(
            &sequent,
            Mode::CLASSICAL.with_affine(),
            &options,
            &crate::Limits::default(),
            |_| {
                polls += 1;
                polls > 20
            },
        )
        .unwrap();
        assert!(
            matches!(outcome.verdict, Verdict::Unknown(Reason::Stopped)),
            "{:?}",
            outcome.verdict
        );
    }

    /// A premise of a `&` that ends at the recursion limit cancels
    /// nothing: the other premise's refutation decides the rule, as on one
    /// thread, which searches the left premise first, and the answer is
    /// neither the limit nor a stop nobody asked for. The right premise's
    /// chain of `⊕` is forty deep under a limit of 24, while the left
    /// one's refutation under Mix takes a hundred times as long.
    #[test]
    fn a_premise_that_gives_up_cancels_no_other() {
        let chain = format!("{}0{}", "(0 + ".repeat(40), ")".repeat(40));
        let pairs: Vec<String> = (0..6)
            .map(|i| format!("(a{i} * b{i}) + 0, (~a{i} * ~b{i}) + 0"))
            .collect();
        let sequent: Sequent = format!("|- bot & {chain}, {}", pairs.join(", "))
            .parse()
            .unwrap();
        for jobs in [1, 2, 4] {
            let options = Options::default().with_jobs(jobs);
            let limits = crate::Limits::default().with_recursion_depth(24);
            let mode = Mode::CLASSICAL.with_mix();
            let outcome = prove_within(&sequent, mode, &options, &limits, |_| false).unwrap();
            assert!(
                matches!(outcome.verdict, Verdict::Unprovable(_)),
                "{:?} on {jobs} threads",
                outcome.verdict
            );
        }
    }

    /// Premises that give up cancel no more than one thread skips: below
    /// sixteen `&`, one after the other in the asynchronous phase, every
    /// leaf reaches the recursion limit on a chain of `⊕` forty deep, and
    /// premises that cancelled nothing when they gave up searched every one
    /// of the 2¹⁶ leaves.
    #[test]
    fn nested_premises_that_give_up_search_no_tree() {
        let chain = format!("{}0{}", "(0 + ".repeat(40), ")".repeat(40));
        let withs = vec!["bot & bot"; 16].join(", ");
        let sequent: Sequent = format!("|- {withs}, {chain}").parse().unwrap();
        for jobs in [2, 4] {
            let options = Options::default().with_jobs(jobs);
            let limits = crate::Limits::default().with_recursion_depth(36);
            let outcome =
                prove_within(&sequent, Mode::CLASSICAL, &options, &limits, |_| false).unwrap();
            assert!(
                matches!(
                    outcome.verdict,
                    Verdict::Unknown(Reason::RecursionLimit { .. })
                ),
                "{:?} on {jobs} threads",
                outcome.verdict
            );
            assert!(
                outcome.statistics.nodes < 1_000,
                "{} stable sequents on {jobs} threads",
                outcome.statistics.nodes
            );
        }
    }

    /// Forty `&` in one asynchronous phase, whose first stable sequent
    /// fails: the failed premise cancels the other one of every `&`, and
    /// a cancelled premise starts none of the 2⁴⁰ below it.
    #[test]
    fn a_cancelled_premise_starts_no_other() {
        let roots: Vec<String> = (0..40).map(|i| format!("a{i} & b{i}")).collect();
        let sequent: Sequent = format!("|- {}", roots.join(", ")).parse().unwrap();
        for jobs in [2, 4] {
            let options = Options::default().with_jobs(jobs);
            let outcome = prove(&sequent, Mode::CLASSICAL, &options).unwrap();
            assert!(matches!(outcome.verdict, Verdict::Unprovable(_)));
            assert!(
                outcome.statistics.nodes < 100_000,
                "{} stable sequents on {jobs} threads",
                outcome.statistics.nodes
            );
        }
    }

    /// Under a raised recursion limit the pool's threads hold the recursion
    /// the limit allows: two copies of a tower of 2 900 nested `&`, at a
    /// limit of 3 000 on two threads, overflowed a worker's stack when
    /// every `&` of the tower forked on the pool.
    #[test]
    fn a_raised_limit_holds_on_the_pool() {
        let n = 2_900;
        let tower = |i: usize| format!("{}c{i}{}", "b & (".repeat(n), ")".repeat(n));
        let sequent: Sequent = format!("|- ?({}), ?({}), ~b", tower(0), tower(1))
            .parse()
            .unwrap();
        let limits = crate::Limits::default().with_recursion_depth(3_000);
        let options = Options::default().with_jobs(2);
        let outcome = prove_within(
            &sequent,
            Mode::CLASSICAL.with_affine(),
            &options,
            &limits,
            |_| false,
        )
        .unwrap();
        assert!(
            matches!(
                outcome.verdict,
                Verdict::Unknown(Reason::RecursionLimit { .. } | Reason::CopyBound { .. })
            ),
            "{:?}",
            outcome.verdict
        );
    }

    /// A stop condition that panics ends the search on the pool at once:
    /// the panic raises the root flag, so the workers stop at their next
    /// poll and the scope that waits for them lets the panic go on, where
    /// they ran a search of minutes to its end before.
    #[test]
    fn a_panicking_stop_ends_every_worker() {
        let (sender, ended) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let sequent = crate::families::mix(11);
            let options = Options::default().with_jobs(2);
            let mut polls = 0;
            let panicked = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                prove_within(
                    &sequent,
                    Mode::CLASSICAL.with_mix(),
                    &options,
                    &crate::Limits::default(),
                    |_| {
                        polls += 1;
                        assert!(polls < 20, "the stop condition panics");
                        false
                    },
                )
            }));
            sender.send(panicked.is_err()).unwrap();
        });
        let panicked = ended.recv_timeout(std::time::Duration::from_secs(20));
        assert_eq!(panicked, Ok(true));
    }

    /// An alternative that a choice skips because an ancestor's flag was
    /// raised is a stop, never a failure: the choice cannot answer that
    /// every alternative failed, and its siblings are cancelled.
    #[test]
    fn a_skipped_alternative_is_no_failure() {
        use std::sync::atomic::{AtomicBool, Ordering};
        let cancel = AtomicBool::new(false);
        let mut collected = super::Collected::new();
        collected.skip(&cancel);
        assert_eq!(collected.error, Some(Reason::Stopped));
        assert!(cancel.load(Ordering::Relaxed));
    }
}
