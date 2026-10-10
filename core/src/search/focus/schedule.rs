// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The two searches of the default bias and how they share the cores:
//! which searches decide a goal ([`plan`]), one search on its own
//! ([`Rule::search`]), and the two ways they share one core, in turns
//! from their start without threads ([`turns`]) and alternating in
//! slices on two threads ([`alternate`]).

use super::arena::{Arena, Kept};
use super::classes::Classes;
use super::counts::Counts;
use super::memo::{Memo, Table};
use super::{Engine, Problem, Search, reason};
use crate::fragment::{Fragment, Mode};
use crate::limits::Limits;
use crate::occurrences::{Forest, OccId, Reading};
use crate::proofs::Node;
use crate::search::Bias;
use crate::search::memory::Account;
use crate::search::{Options, Reason, Statistics, Stop};
use crate::sequents::Kind;
#[cfg(feature = "parallel")]
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
#[cfg(feature = "parallel")]
use std::sync::{Condvar, Mutex};

/// Decides a goal by two searches that take turns on one thread, each
/// from its start with a memo and an arena of its own, on an amount of
/// work that grows from round to round, until one decides; a search that
/// ended without deciding takes no further turn, and the other then runs
/// to its own end. The counters are those of every turn together, the
/// memo's entries the most of one turn.
#[allow(clippy::too_many_arguments)]
pub(super) fn turns(
    forest: &Forest,
    goal: &[OccId],
    fragment: Fragment,
    mode: Mode,
    reading: Option<&Reading>,
    classes: &Classes,
    options: &Options,
    limits: &Limits,
    searches: [(Rule, &Counts, &Account); 2],
    stop: &mut dyn FnMut() -> bool,
) -> (Search, Vec<Node>, Statistics) {
    let mut ended: [Option<Reason>; 2] = [None, None];
    let mut statistics = Statistics::default();
    let mut turn = FIRST_TURN;
    loop {
        for (i, (rule, counts, account)) in searches.into_iter().enumerate() {
            if ended[i].is_some() {
                continue;
            }
            let stop = if ended[1 - i].is_some() {
                Stop::Closure(&mut *stop)
            } else if i == 0 {
                Stop::Turn(&mut *stop, turn)
            } else {
                Stop::Turn(&mut *stop, turn.saturating_mul(BACKWARD_SHARE))
            };
            let (result, nodes, run, over) = rule.search(
                forest,
                goal,
                fragment,
                mode,
                reading,
                (counts, classes),
                options,
                limits,
                // A turn's memo and arena go when it ends.
                &account.fork(),
                stop,
            );
            statistics.add(&run);
            match result {
                Err(Reason::Stopped) if over => {}
                Err(Reason::Stopped) => return (Err(Reason::Stopped), nodes, statistics),
                Err(reason) => ended[i] = Some(reason),
                decided => return (decided, nodes, statistics),
            }
        }
        if let [Some(_), Some(backward)] = ended {
            return (
                Err(reason(backward, options, limits)),
                Vec::new(),
                statistics,
            );
        }
        turn = turn.saturating_mul(TURN_GROWTH);
    }
}

/// The work each of the two searches of the default bias gets in the
/// first round of their turns, in steps of a split search, a stable
/// sequent counting [`NODE_WORK`] of them and more with its size: some
/// thousand stable sequents of a small problem.
const FIRST_TURN: u64 = 1 << 16;

/// How much work the backward search of the default bias gets for each
/// unit of the forward one: it is the search the default ran alone
/// before, so under a time limit what it decides alone in two thirds of
/// the limit is still decided, and what the forward search decides in a
/// third.
pub(crate) const BACKWARD_SHARE: u64 = 2;

/// The factor by which the turns of the two searches grow from round to
/// round. A search starts afresh in every turn, so with turns that grow
/// geometrically the turns that ended early cost a fraction of the one
/// that decides.
const TURN_GROWTH: u64 = 4;

/// One search of a goal: the bias it runs under and its copy bound.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Rule {
    /// The bias.
    pub(super) bias: Bias,
    /// The most copies a branch may take; `u32::MAX` is no bound.
    pub(super) copies: u32,
}

impl Rule {
    /// Runs the search of this rule alone, with a memo and an arena of its
    /// own, until it ends or the stop condition fires. Returns what
    /// [`search_goal`] does, and whether it was a turn that ran out of
    /// its work.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn search<'a>(
        self,
        forest: &'a Forest,
        goal: &[OccId],
        fragment: Fragment,
        mode: Mode,
        reading: Option<&'a Reading<'a>>,
        (counts, classes): (&'a Counts, &'a Classes),
        options: &Options,
        limits: &Limits,
        account: &'a Account,
        stop: Stop<'a>,
    ) -> (Search, Vec<Node>, Statistics, bool) {
        let problem = Problem::new(
            forest,
            reading,
            (counts, classes),
            fragment,
            mode,
            options,
            limits,
            self.copies,
            account,
        );
        let mut engine = Engine::new(
            problem,
            stop,
            Table::Own(Memo::new(options.memo_entries())),
            Arena::new(Kept::Own(Vec::new()), account),
        );
        let result = engine
            .run(goal)
            .and_then(|root| root.map(|root| engine.nodes.keep(0, root)).transpose());
        let statistics = engine.statistics();
        let over = matches!(engine.stop, Stop::Turn(_, 0));
        let nodes = match engine.nodes.kept {
            Kept::Own(nodes) => nodes,
            Kept::Shared(_) => unreachable!("a sequential search owns its arena"),
        };
        (result, nodes, statistics, over)
    }
}

/// The searches that decide a goal under the options: one, under the bias
/// they name, where `Auto` is the factor rule without an exponential in
/// the sequent and the rarer literal with one or under weakening (nothing
/// forces a split there, so the factors have nothing to say). Or two, for
/// `Auto` on a goal with exponentials in linear mode: the forward search
/// first, under the factor rule, and the backward search, under the rarer
/// literal within [`Options::copies`], which is the one `Auto` ran alone
/// before; they share nothing, and the first to decide answers. The
/// forward search runs within `Options::copies` too, and where the
/// goal is a Horn program ([`chains`]) within the larger of that and
/// [`Options::forward_copies`]: there a copy is a step of a forward
/// chain, of which one branch takes as many as the chain is long.
/// Not under Mix, where every sequent of a chain that grows is tried in
/// every partition.
/// When the two rules give every atom the same literal the forward search
/// is the backward one continued, and runs alone.
pub(crate) fn plan(
    forest: &Forest,
    goal: &[OccId],
    fragment: Fragment,
    mode: Mode,
    options: &Options,
) -> (Rule, Option<Rule>) {
    let copies = options.copy_bound();
    let both = options.bias == Bias::Auto
        && !mode.affine
        && fragment.has_exponentials()
        && forest.sequent().fragment().has_exponentials();
    if !both {
        let bias = if options.bias == Bias::Auto && mode.affine {
            Bias::Rarer
        } else {
            options.bias
        };
        return (Rule { bias, copies }, None);
    }
    let forward = Rule {
        bias: Bias::Factors,
        copies: if !mode.mix && chains(forest, goal) {
            copies.max(options.forward_copies)
        } else {
            copies
        },
    };
    let backward = Rule {
        bias: Bias::Rarer,
        copies,
    };
    let same = super::bias::signs(forest, Bias::Factors) == super::bias::signs(forest, Bias::Rarer);
    (forward, (!same).then_some(backward))
}

/// Whether a goal is a Horn program: every member under a `?` a clause,
/// every other member a marking, a goal or a clause used once, as a
/// clause is in the linear zone that an interactive dereliction leaves. With the atoms of the
/// bodies written `a` (or all of them `~a`), a clause is a tensor of body
/// literals of which at most one factor is a head instead, a literal of
/// the other sign or a `⅋` of such, which is what `!(a ⊗ b ⊸ c ⊗ d)` is
/// on the right of `⊢`; a marking is a `⅋` of head literals, a goal a
/// tensor of body literals; `1` stands for an empty body or goal and `⊥`
/// for an empty head. A Petri net with a marking to reach is such a
/// program. There a copy of a clause rewrites the linear zone, so a
/// search within `n` copies visits the markings within `n` steps and
/// nothing else, which is what makes a bound of its own affordable.
pub(super) fn chains(forest: &Forest, goal: &[OccId]) -> bool {
    use crate::occurrences::Sign;
    [Sign::Atom, Sign::Dual].into_iter().any(|body| {
        // A tree of one connective and its unit over literals of one sign.
        let tree = |o: OccId, connective: Kind, unit: Kind, sign: Sign| {
            forest.subtree(o).all(|x| {
                let kind = forest.kind(x);
                kind == connective || kind == unit || forest.sign(x) == Some(sign)
            })
        };
        let head = |o: OccId| tree(o, Kind::Par, Kind::Bot, !body);
        // A tensor of body literals with at most one factor a head.
        let mut factors = Vec::new();
        let mut clause = |o: OccId| {
            let mut heads = 0;
            factors.clear();
            factors.push(o);
            while let Some(x) = factors.pop() {
                match forest.kind(x) {
                    Kind::Tensor => factors.extend(forest.children(x)),
                    Kind::One => {}
                    _ if forest.sign(x) == Some(body) => {}
                    _ => {
                        heads += 1;
                        if heads > 1 || !head(x) {
                            return false;
                        }
                    }
                }
            }
            true
        };
        goal.iter().all(|&member| {
            if forest.kind(member) == Kind::Quest {
                clause(forest.left(member).unwrap())
            } else {
                head(member) || clause(member)
            }
        })
    })
}

#[cfg(feature = "parallel")]
/// The work a search does between two points at which it gives way to
/// the other, when two alternate on one core: about a thousand stable
/// sequents of a small problem, against which a change of threads costs
/// little.
const SLICE: u64 = 1 << 16;

#[cfg(feature = "parallel")]
/// How long the calling thread waits for a search that runs on alone
/// before it polls the caller's stop condition again.
const POLL: std::time::Duration = std::time::Duration::from_millis(1);

#[cfg(feature = "parallel")]
/// Which of two alternating searches may run, and what both must know of
/// each other.
#[derive(Default)]
struct Baton {
    /// The state.
    state: Mutex<Turns>,
    /// Signalled whenever the state changes.
    changed: Condvar,
    /// The polls of the second search that the caller's stop condition
    /// has not been told of yet: it lives on the calling thread, which
    /// polls it once for each of them.
    polls: AtomicU64,
    /// Whether both must stop, for the second search to read at every
    /// poll without the lock.
    halt: AtomicBool,
}

#[cfg(feature = "parallel")]
/// The state two alternating searches share.
#[derive(Default)]
struct Turns {
    /// The search that may run.
    holder: usize,
    /// Whether each search has ended.
    ended: [bool; 2],
    /// Whether both must stop: one decided, the caller's stop condition
    /// fired, or a thread panicked.
    stop: bool,
}

#[cfg(feature = "parallel")]
impl Baton {
    /// Locks the state; it is plain data, so a panic of the other thread
    /// leaves it usable.
    fn lock(&self) -> std::sync::MutexGuard<'_, Turns> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Waits until search `me` may run, and returns whether it must stop
    /// instead.
    fn wait(&self, me: usize) -> bool {
        let mut turns = self.lock();
        while turns.holder != me && !turns.stop {
            turns = self
                .changed
                .wait(turns)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
        turns.stop
    }

    /// Gives way to the other search, unless it has ended, and waits for
    /// the turn of search `me` to come again. Returns whether it must stop.
    fn pass(&self, me: usize) -> bool {
        {
            let mut turns = self.lock();
            if turns.stop || turns.ended[1 - me] {
                return turns.stop;
            }
            turns.holder = 1 - me;
        }
        self.changed.notify_all();
        self.wait(me)
    }

    /// Gives way to the second search from the first, which runs on the
    /// calling thread, and waits for its turn to come again, unless the
    /// second has ended. While it waits it wakes once every [`POLL`] and
    /// polls the caller's stop condition for the polls the second search
    /// made meanwhile, so that a slice of the second search, which in
    /// seconds may be long, does not keep the condition waiting. Returns
    /// whether both must stop.
    fn pass_polling(&self, stop: &mut dyn FnMut() -> bool) -> bool {
        {
            let mut turns = self.lock();
            if turns.stop || turns.ended[1] {
                return turns.stop;
            }
            turns.holder = 1;
        }
        self.changed.notify_all();
        loop {
            {
                let turns = self.lock();
                if turns.stop {
                    return true;
                }
                if turns.holder == 0 {
                    break;
                }
                let waited = self.changed.wait_timeout(turns, POLL);
                drop(waited.unwrap_or_else(|poisoned| poisoned.into_inner()));
            }
            // The caller's condition runs without the lock.
            if self.caught_up(stop) {
                self.stop();
                return true;
            }
        }
        self.caught_up(stop)
    }

    /// Tells both searches to stop.
    fn stop(&self) {
        self.halt.store(true, Ordering::Relaxed);
        self.lock().stop = true;
        self.changed.notify_all();
    }

    /// Polls the caller's stop condition once for every poll the second
    /// search made since it was last done, so that a condition that
    /// counts its polls, or looks at a clock every so many, sees the two
    /// searches as it sees one. Returns whether it fired.
    fn caught_up(&self, stop: &mut dyn FnMut() -> bool) -> bool {
        (0..self.polls.swap(0, Ordering::Relaxed)).any(|_| stop())
    }
}

#[cfg(feature = "parallel")]
/// Stops both searches when the calling thread unwinds, so that a panic
/// of the caller's stop condition does not leave the second search
/// running.
struct StopOnPanic<'a>(&'a Baton);

#[cfg(feature = "parallel")]
impl Drop for StopOnPanic<'_> {
    fn drop(&mut self) {
        if std::thread::panicking() {
            self.0.stop();
        }
    }
}

#[cfg(feature = "parallel")]
/// Marks a search as ended when its thread leaves it, by a return or by a
/// panic, so that the other never waits for it.
struct Ended<'a>(&'a Baton, usize);

#[cfg(feature = "parallel")]
impl Drop for Ended<'_> {
    fn drop(&mut self) {
        let Ended(baton, me) = *self;
        {
            let mut turns = baton.lock();
            turns.ended[me] = true;
            turns.holder = 1 - me;
            turns.stop |= std::thread::panicking();
            if turns.stop {
                baton.halt.store(true, Ordering::Relaxed);
            }
        }
        baton.changed.notify_all();
    }
}

#[cfg(feature = "parallel")]
/// Decides a goal by two searches that alternate on one core: the first
/// on the calling thread, the second on a thread of its own, one of them
/// running at a time for a slice of work ([`SLICE`], the second search
/// [`BACKWARD_SHARE`](BACKWARD_SHARE) of them), so that each is the
/// search it would be alone, none starts again, and the run is a function
/// of the input. The first to decide stops the other at the end of its
/// slice; a search that ended without deciding leaves the other to run
/// on. The caller's stop condition is polled by the first search at its
/// own polls, and once for every poll of the second, within a millisecond
/// of it: the calling thread wakes that often while the second search
/// has its turn or runs on alone. Returns what
/// [`search_goal`](super::search_goal) does, the counters of both searches together,
/// or `None` when the thread cannot start.
#[allow(clippy::too_many_arguments)]
pub(super) fn alternate(
    forest: &Forest,
    goal: &[OccId],
    fragment: Fragment,
    mode: Mode,
    reading: Option<&Reading>,
    classes: &Classes,
    options: &Options,
    limits: &Limits,
    [
        (first, first_counts, first_account),
        (second, second_counts, second_account),
    ]: [(Rule, &Counts, &Account); 2],
    stop: &mut dyn FnMut() -> bool,
) -> Option<(Search, Vec<Node>, Statistics)> {
    let baton = Baton::default();
    let (forward, backward) = std::thread::scope(|scope| {
        let baton = &baton;
        let thread = std::thread::Builder::new()
            .name("linlog-search".to_owned())
            .stack_size(limits.stack_bytes())
            .spawn_scoped(scope, move || {
                let _ended = Ended(baton, 1);
                if baton.wait(1) {
                    return (Err(Reason::Stopped), Vec::new(), Statistics::default());
                }
                let mut give_way = |passed: bool| {
                    baton.polls.fetch_add(1, Ordering::Relaxed);
                    baton.halt.load(Ordering::Relaxed) || (passed && baton.pass(1))
                };
                let slice = SLICE * BACKWARD_SHARE;
                let (result, nodes, statistics, _) = second.search(
                    forest,
                    goal,
                    fragment,
                    mode,
                    reading,
                    (second_counts, classes),
                    options,
                    limits,
                    second_account,
                    Stop::Slice(&mut give_way, slice, slice),
                );
                if result.is_ok() {
                    baton.stop();
                }
                (result, nodes, statistics)
            })
            .ok()?;
        let _stop = StopOnPanic(baton);
        let forward = {
            let _ended = Ended(baton, 0);
            let mut give_way = |passed: bool| stop() || (passed && baton.pass_polling(stop));
            let (result, nodes, statistics, _) = first.search(
                forest,
                goal,
                fragment,
                mode,
                reading,
                (first_counts, classes),
                options,
                limits,
                first_account,
                Stop::Slice(&mut give_way, SLICE, SLICE),
            );
            if !matches!(result, Err(reason) if reason != Reason::Stopped) {
                baton.stop();
            }
            (result, nodes, statistics)
        };
        // The second search runs on alone, or is about to end.
        loop {
            let turns = baton.lock();
            if turns.ended[1] {
                break;
            }
            let waited = baton.changed.wait_timeout(turns, POLL);
            drop(waited.unwrap_or_else(|poisoned| poisoned.into_inner()));
            if baton.caught_up(stop) {
                baton.stop();
            }
        }
        let backward = thread
            .join()
            .unwrap_or_else(|panic| std::panic::resume_unwind(panic));
        Some((forward, backward))
    })?;
    Some(merged(forward, backward, options, limits))
}

#[cfg(feature = "parallel")]
/// The answer of two searches of one goal that ran together, from what
/// each returned: a verdict of either; else the stop, which without a
/// verdict is the caller's; else the second search's reason. The counters
/// are both searches' together.
pub(super) fn merged(
    first: (Search, Vec<Node>, Statistics),
    second: (Search, Vec<Node>, Statistics),
    options: &Options,
    limits: &Limits,
) -> (Search, Vec<Node>, Statistics) {
    let mut statistics = first.2;
    // The two memos were held at once.
    statistics.add_run(&second.2);
    statistics.memo_hits += second.2.memo_hits;
    statistics.memo_entries += second.2.memo_entries;
    debug_assert!(
        !matches!((&first.0, &second.0), (Ok(a), Ok(b)) if a.is_some() != b.is_some()),
        "the two searches contradict each other"
    );
    let (result, nodes) = match (first.0, second.0) {
        (Ok(root), _) => (Ok(root), first.1),
        (_, Ok(root)) => (Ok(root), second.1),
        (Err(Reason::Stopped), _) | (_, Err(Reason::Stopped)) => (Err(Reason::Stopped), Vec::new()),
        (Err(_), Err(reason)) => (Err(super::reason(reason, options, limits)), Vec::new()),
    };
    (result, nodes, statistics)
}
