// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The focused sequent engine: backward search over dyadic sequents of
//! occurrence ids with a memo of stable sequents, as the `MALL-Seq` and
//! `MELL-Seq` specifications state it.
//!
//! A sequent inside the search is `⊢ Θ ; Γ`: the *unrestricted zone* `Θ`, a
//! set of the formulas that arrived under a `?` and may be copied any number
//! of times, and the *linear zone* `Γ`, a multiset of occurrences each of
//! which is used exactly once. The *asynchronous phase* decomposes the
//! negative formulas of a sequent without choice (`⅋` opens, `⊥` drops, `⊤`
//! closes, `&` branches into two premises with the same context, `?` moves
//! its subformula into `Θ`) until a *stable* sequent remains: `Γ` holds
//! positive formulas and negative literals only. `prove` decides a stable
//! sequent by looking it up in the memo, by the immediate tests (a `0`, an
//! unbalanced atom, a dual pair, a literal whose dual lies in `Θ`), and
//! otherwise by choosing a positive formula to *focus* on: one of `Γ`, which
//! it consumes, or one of `Θ`, which stays there (a *copy*, the one step
//! that can repeat, bounded per branch). `focus` applies the positive rules
//! along that formula: `⊕` picks a side, `⊗` splits `Γ`, `1` needs it empty,
//! `!` needs it empty and continues with its subformula, a positive literal
//! needs exactly its dual, in `Γ` or in `Θ`; and when the formula turns
//! negative it is *released* into the asynchronous phase. Every stable
//! sequent proved or refuted goes into the memo, so a `&` that duplicates
//! the context, or a split that revisits a part, never pays twice. With
//! Mix, a stable sequent no focus proves is split into two provable parts.
//! In affine mode a leaf weakens whatever is left over.
//!
//! The copies are bounded per branch and the bound deepens iteratively:
//! `Unprovable` is answered only after a level that never hit its bound.
//!
//! The same engine searches intuitionistic linear logic two-sided, given
//! the sequent's intuitionistic reading: every rule of the two-sided
//! focused calculus is a rule of this one on the one-sided sequent, and
//! the one-succedent condition holds on every branch by itself except at
//! the split of a hypothesis `A ⊸ B` (a `⊗` in input position), where the
//! goal must stay on the consequent's side; that is the one place the
//! reading is consulted.
//! The counts of `counts.rs` prune: a stable sequent or a side of a split
//! whose intervals exclude zero for some atom is refuted without search,
//! and in the multiplicative fragments the count equation as well.

/// The proof arena: pending and kept nodes.
mod arena;
/// The atom bias.
mod bias;
/// Interchangeable occurrences.
mod classes;
/// The linear zone as a multiset.
mod context;
/// The count invariants the engine prunes with.
mod counts;
/// The memo of stable sequents.
mod memo;
#[cfg(feature = "parallel")]
pub(crate) mod parallel;
/// The two searches of the default bias and how they share the cores.
mod schedule;
/// The pools of scratch buffers.
mod scratch;
/// The `⊗` rule and Mix: forced and searched splits.
mod split;
#[cfg(all(test, feature = "parse"))]
mod tests;

use self::arena::Arena;
use self::classes::Classes;
use self::context::Context;
use self::counts::{Counts, Split, Tally};
use self::memo::{Entry, Failure, Inserted, Key, Table, Zones};
use self::schedule::{plan, turns};
use self::scratch::{Pooled, Pools};
use self::split::Join;
use super::Bias;
use super::memory::{Account, Charged};
use super::{Answer, Decide, Options, Reason, Refutation, Statistics, Stop, Task, set_up_stopped};
use crate::Error;
use crate::fragment::{Fragment, Mode};
use crate::occurrences::{Forest, OccId, OccSet, Reading};
use crate::proofs::{Node, NodeId, Side};
use crate::sequents::Kind;

/// The stack depth that stands for "no pruned sequent depends on an
/// ancestor".
const NO_DEPENDENCY: u32 = u32::MAX;

/// How many steps of its searches for the splits of a `⊗` or a Mix the
/// engine takes between two polls of the stop condition.
const SPLITS_PER_POLL: u64 = 4096;

/// How many forced splits, and literals of the tensors they close in
/// place, a chain of them takes between two polls of the stop condition.
const FORCED_PER_POLL: u64 = 4096;

/// The work of a stable sequent in steps of a split search, the unit a
/// turn is counted in, before what depends on its size: the bookkeeping
/// of a visit costs some ten times a step, which moves one member and
/// tests its row.
const NODE_WORK: u64 = 16;

/// How many occurrences of the forest make a unit of a stable sequent's
/// work: both zones are hashed, compared and copied as bitsets of the
/// forest's width.
const OCCURRENCES_PER_WORK: usize = 128;

/// How many comparisons of a member of `Γ` with a literal of a formula of
/// `Θ` make a unit of work: the copies are ranked by whether they meet a
/// member, which on a Petri net with its hundreds of transitions and
/// tokens is most of what a stable sequent costs.
const MEETS_PER_WORK: usize = 16;

/// How many occurrences of the forest make a unit of the work of a split
/// whose premises are tried: its two sides are built as zones of the
/// forest's width.
const OCCURRENCES_PER_LEAF: usize = 512;

/// The focused engine as the front door calls it, one-sided or two-sided.
pub(crate) struct Focused {
    /// Whether it is the two-sided engine of intuitionistic mode.
    two_sided: bool,
}

/// The focused engine of classical mode.
pub(crate) const ONE_SIDED: Focused = Focused { two_sided: false };

/// The focused engine of intuitionistic mode.
pub(crate) const TWO_SIDED: Focused = Focused { two_sided: true };

impl Decide for Focused {
    /// Refuses the one-sided engine in intuitionistic mode and the
    /// two-sided one in classical mode.
    fn admits(&self, task: &Task<'_>) -> Result<(), Error> {
        if task.mode.intuitionistic == self.two_sided {
            return Ok(());
        }
        let engine = if self.two_sided {
            super::Engine::TwoSided
        } else {
            super::Engine::Focus
        };
        Err(Error::EngineMode {
            engine,
            mode: task.mode,
        })
    }

    /// Searches the goal on one thread, or with the feature `parallel` on
    /// [`Options::jobs`] threads.
    fn decide(
        &self,
        task: &Task<'_>,
        options: &Options,
        account: &Account,
        stop: &mut dyn FnMut() -> bool,
    ) -> Result<Answer, Error> {
        let Task {
            forest,
            goal,
            fragment,
            mode,
            reading,
            ..
        } = *task;
        #[cfg(feature = "parallel")]
        if options.job_count() > 1 {
            let found = parallel::search_goal(
                forest, goal, fragment, mode, reading, options, account, stop,
            )?;
            return Ok(Answer::of_arena(forest, found));
        }
        let found = search_goal(
            forest, goal, fragment, mode, reading, options, account, stop,
        );
        Ok(Answer::of_arena(forest, found))
    }
}

/// Runs the focused engine from a goal: a multiset of occurrences of the
/// forest, the sequent's roots for a search of the sequent itself, or any
/// other with an empty unrestricted zone, as an interactive prover hands
/// over an open goal (with exactly one occurrence in output position when
/// a reading is given). Returns the node proving the goal (`None` when it
/// is unprovable, or the reason the search gave up), the arena the node
/// lives in, and the statistics. What the search allocates is charged to
/// `account`; two searches that decide the goal together have half its
/// bound each.
#[allow(clippy::too_many_arguments)]
pub(crate) fn search_goal(
    forest: &Forest,
    goal: &[OccId],
    fragment: Fragment,
    mode: Mode,
    reading: Option<&Reading>,
    options: &Options,
    account: &Account,
    stop: &mut dyn FnMut() -> bool,
) -> (Search, Vec<Node>, Statistics) {
    // On a large forest every pass of the set-up is followed by a poll.
    let gave_up = |r| (Err(reason(r, options)), Vec::new(), Statistics::default());
    let classes = Classes::new(forest, reading);
    if set_up_stopped(forest, stop) {
        return gave_up(Reason::Stopped);
    }
    let (first, second) = plan(forest, goal, fragment, mode, options);
    if set_up_stopped(forest, stop) {
        return gave_up(Reason::Stopped);
    }
    let Some(second) = second else {
        account.charge(classes.bytes());
        let counts = match Counts::new_until(forest, first.bias, account, stop) {
            Ok(counts) => counts,
            Err(reason) => return gave_up(reason),
        };
        let (result, nodes, statistics, _) = first.search(
            forest,
            goal,
            fragment,
            mode,
            reading,
            (&counts, &classes),
            options,
            account,
            Stop::Closure(stop),
        );
        return (result.map_err(|r| reason(r, options)), nodes, statistics);
    };
    // Each search is charged what it reads, the classes included.
    let accounts = [account.share(2), account.share(2)];
    for account in &accounts {
        account.charge(classes.bytes());
    }
    let first_counts = match Counts::new_until(forest, first.bias, &accounts[0], stop) {
        Ok(counts) => counts,
        Err(reason) => return gave_up(reason),
    };
    let second_counts = match Counts::new_until(forest, second.bias, &accounts[1], stop) {
        Ok(counts) => counts,
        Err(reason) => return gave_up(reason),
    };
    let searches = [
        (first, &first_counts, &accounts[0]),
        (second, &second_counts, &accounts[1]),
    ];
    // With threads the two searches alternate in slices and none starts
    // again; without them, or when a thread cannot start, they take
    // turns from their start.
    #[cfg(feature = "parallel")]
    if let Some(result) = schedule::alternate(
        forest, goal, fragment, mode, reading, &classes, options, searches, stop,
    ) {
        return result;
    }
    turns(
        forest, goal, fragment, mode, reading, &classes, options, searches, stop,
    )
}

/// Returns why a goal that the search refuted is unprovable, as far as its
/// counts tell: an atom whose literals cannot pair up, or the count
/// equation that fails, each under the rules the engine searched the
/// fragment and the mode with; else that the search was exhaustive. The
/// asynchronous phase keeps the sums of the goal's members (a `⅋` or `⊥`
/// keeps the count equation, a premise of `&` lies within its hull), so
/// a goal that fails a test makes every stable sequent above it fail it,
/// where the engine checks it. Counting is a pass over the forest under
/// the search's limits; a pass given up says only what the verdict does.
pub(crate) fn refutation(
    forest: &Forest,
    goal: &[OccId],
    fragment: Fragment,
    mode: Mode,
    account: &Account,
    stop: &mut dyn FnMut() -> bool,
) -> Refutation {
    let Ok(counts) = Counts::new_until(forest, Bias::Rarer, account, stop) else {
        return Refutation::Exhausted;
    };
    let rules = Rules::new(fragment, mode, &counts);
    let mut tally = counts.tally();
    for &o in goal {
        tally.add(&counts, o);
    }
    if rules.intervals
        && let Some((atom, least, most)) = tally.unbalanced(&counts)
    {
        return Refutation::Unbalanced {
            atom,
            name: forest.sequent().atom_names()[atom.index()].clone(),
            least,
            most,
        };
    }
    if rules.equation && !tally.equation(rules.mix) {
        let (mut tensors, mut pars, mut ones, mut bottoms) = (0, 0, 0, 0);
        for &o in goal {
            for x in forest.subtree(o) {
                match forest.kind(x) {
                    Kind::Tensor => tensors += 1,
                    Kind::Par => pars += 1,
                    Kind::One => ones += 1,
                    Kind::Bot => bottoms += 1,
                    _ => {}
                }
            }
        }
        return Refutation::Equation {
            formulas: goal.len() as u64,
            tensors,
            pars,
            ones,
            bottoms,
            mix: rules.mix,
        };
    }
    Refutation::Exhausted
}

/// The reason a search of the options gives up with, given the reason one
/// of its searches did: a copy bound is [`Options::copies`], the bound
/// every search ran within at the least, and a memory limit is
/// [`Options::memory_limit`], of which a search may have had a part.
pub(crate) fn reason(reason: Reason, options: &Options) -> Reason {
    match reason {
        Reason::CopyBound(_) => Reason::CopyBound(options.copy_bound()),
        Reason::MemoryLimit(bytes) => Reason::MemoryLimit(options.memory_limit.unwrap_or(bytes)),
        reason => reason,
    }
}

/// Whether a split of a goal into the two premises of a `⊗`, each given
/// with its subformula of the `⊗`, or of a Mix passes the count prunes the
/// engine applies to every split: the interval check per atom and, in the
/// multiplicative fragments, the count equation. A split that fails cannot
/// close; one that passes may still fail. `fragment` is the goal's.
pub(crate) fn split_passes(
    forest: &Forest,
    fragment: Fragment,
    mode: Mode,
    left: &[OccId],
    right: &[OccId],
) -> bool {
    let counts = Counts::new(forest, Bias::Auto);
    let rules = Rules::new(fragment, mode, &counts);
    let mut split = counts.split();
    for (members, side) in [(left, Side::Left), (right, Side::Right)] {
        for &m in members {
            split.place(&counts, m, side);
        }
    }
    split.feasible(rules.intervals, rules.equation, rules.mix)
}

/// The rules in force beyond the core ones, switched by fragment and mode.
#[derive(Clone, Copy, Debug)]
struct Rules {
    /// The Mix rule, tried last on a stable sequent; off under weakening,
    /// which makes it admissible.
    mix: bool,
    /// The `MLL` count equation as a prune: only without additives and
    /// exponentials anywhere in the problem, and never with weakening.
    equation: bool,
    /// The interval check as a prune: never with weakening, which can
    /// discard any imbalance.
    intervals: bool,
    /// The exponential rules: `?` into the unrestricted zone, copies from it
    /// under the bound, `!` in focus.
    exponentials: bool,
    /// Weakening: a leaf discards what is left over.
    affine: bool,
    /// The stack of the branch's stable sequents, for the loop check when
    /// copies can repeat a sequent.
    stack: bool,
}

impl Rules {
    /// The rules for a fragment and a mode: the count equation only in the
    /// multiplicative fragments without weakening, the interval check
    /// unless weakening or a `⊤` under an exponential defeats it, and Mix
    /// only without weakening.
    fn new(fragment: Fragment, mode: Mode, counts: &Counts) -> Self {
        let exponentials = fragment.has_exponentials();
        Self {
            // With weakening Mix proves nothing new: the proof of one part
            // proves the whole with the other part weakened at its leaves,
            // with the same copies on every branch.
            mix: mode.mix && !mode.affine,
            equation: !mode.affine
                && !fragment.has_additives()
                && !fragment.has_additive_units()
                && !exponentials,
            intervals: !mode.affine && !counts.absorbs_from_copies(),
            exponentials,
            affine: mode.affine,
            stack: exponentials,
        }
    }
}

/// The result of a search: the node proving the goal, `None` when it is
/// unprovable, or the reason the search stopped.
pub(crate) type Search = Result<Option<NodeId>, Reason>;

/// What a failure below a step rests on besides the sequents it searched:
/// a branch cut by the copy budget, or a prune by the loop check, which
/// is a fact about the branch and not about the sequent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Cuts {
    /// Whether some branch failed because its copy budget was spent: a
    /// level whose search failed with this is not `Unprovable`.
    exhausted: bool,
    /// The shallowest stack depth of an ancestor that a pruned sequent
    /// repeated, or [`NO_DEPENDENCY`]: a failure that rests on such a
    /// prune is not memoized.
    dependency: u32,
}

impl Cuts {
    /// Nothing cut, nothing pruned.
    const NONE: Self = Self {
        exhausted: false,
        dependency: NO_DEPENDENCY,
    };

    /// A branch cut by the copy budget.
    const BUDGET: Self = Self {
        exhausted: true,
        dependency: NO_DEPENDENCY,
    };

    /// A sequent pruned as a repeat of the ancestor at `depth` on the
    /// branch stack.
    fn repeat(depth: u32) -> Self {
        Self {
            exhausted: false,
            dependency: depth,
        }
    }

    /// Both together: a cut if either was, the shallower dependency.
    fn and(self, other: Self) -> Self {
        Self {
            exhausted: self.exhausted || other.exhausted,
            dependency: self.dependency.min(other.dependency),
        }
    }
}

/// What a step of the search found: a proof of its sequent, or that
/// there is none with the cuts of every step it ran. A failure rests on
/// the failures below it and on nothing else, so a step that runs others
/// adds the cuts of those that failed to its own; a proof is a fact that
/// rests on nothing, so it carries no cuts, and a failure of a rule whose
/// other premise was proved rests on its own premise's cuts alone. A
/// stable sequent reads off its decision's result what its memo entry may
/// say.
#[derive(Clone, Copy, Debug)]
enum Found {
    /// A proof, by this node.
    Proved(NodeId),
    /// No proof, with the cuts of the steps run.
    Failed(Cuts),
}

impl Found {
    /// Nothing proved and nothing cut: a failure that is a fact.
    const NOTHING: Self = Self::Failed(Cuts::NONE);

    /// A proof.
    fn proved(node: NodeId) -> Self {
        Self::Proved(node)
    }

    /// A failure with these cuts.
    fn failed(cuts: Cuts) -> Self {
        Self::Failed(cuts)
    }

    /// The node of the proof, if there is one.
    fn node(self) -> Option<NodeId> {
        match self {
            Self::Proved(node) => Some(node),
            Self::Failed(_) => None,
        }
    }

    /// The same result after steps with the cuts given failed first: a
    /// failure rests on them too, a proof on nothing.
    fn after(self, cuts: Cuts) -> Self {
        match self {
            Self::Proved(node) => Self::Proved(node),
            Self::Failed(own) => Self::Failed(cuts.and(own)),
        }
    }

    /// The same result with the proof, if there is one, wrapped by `wrap`.
    fn map(self, wrap: impl FnOnce(NodeId) -> NodeId) -> Self {
        match self {
            Self::Proved(node) => Self::Proved(wrap(node)),
            failed => failed,
        }
    }
}

/// The result of a step of the search: what it found, or the reason the
/// whole search stops.
type Step = Result<Found, Reason>;

/// An alternative of a choice: what one thread tries in its order and a
/// pool's search runs as a task of its own.
#[derive(Clone, Copy)]
enum Alternative<'m> {
    /// A focus on a member of `Γ`, which leaves the context.
    Focus(OccId),
    /// A copy of a member of `Θ` in focus, at one unit less of the budget.
    Copy(OccId),
    /// A side of a `⊕` in focus, with the subformula on that side.
    Side(OccId, Side, OccId),
    /// The free splits of a `⊗` that assign the first `fixed` members as
    /// the bits of `pattern` say, one for the left premise, from the sides
    /// and counts before any member moved.
    Splits {
        /// The `⊗` rule the sides are the premises of.
        join: Join,
        /// The members of the context that the search assigns.
        members: &'m [OccId],
        /// How many of them the pattern assigns.
        fixed: usize,
        /// Their assignment.
        pattern: u64,
        /// The sides before any member moved.
        sides: (&'m Context, &'m Context),
        /// Their counts, the members open.
        split: &'m Split,
    },
}

/// What every engine of one search reads and none changes: the problem,
/// the rules it is searched under, the account its memory is charged to,
/// and its limits. A pool's workers are built from the same one.
#[derive(Clone, Copy)]
struct Problem<'a> {
    /// The forest.
    forest: &'a Forest,
    /// Its intuitionistic reading, for a two-sided search.
    reading: Option<&'a Reading<'a>>,
    /// Its count invariants.
    counts: &'a Counts,
    /// Its classes of interchangeable occurrences.
    classes: &'a Classes,
    /// The rules in force.
    rules: Rules,
    /// The search's account, which the memo, the arena and the buffers are
    /// charged to.
    account: &'a Account,
    /// Whether the memo takes entries: a proof it refers to is kept.
    memoizes: bool,
    /// The deepest nesting allowed.
    recursion_limit: u32,
    /// The most copies a branch may take at the last deepening level.
    copies: u32,
}

impl<'a> Problem<'a> {
    /// The problem of a search of the forest under the fragment and the
    /// mode, given its reading, counts and classes, with the limits of
    /// the options and a copy bound of `copies`, charging `account`.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        forest: &'a Forest,
        reading: Option<&'a Reading<'a>>,
        (counts, classes): (&'a Counts, &'a Classes),
        fragment: Fragment,
        mode: Mode,
        options: &Options,
        copies: u32,
        account: &'a Account,
    ) -> Self {
        Self {
            forest,
            reading,
            counts,
            classes,
            rules: Rules::new(fragment, mode, counts),
            account,
            memoizes: options.memo_limit != 0,
            recursion_limit: options.recursion_limit,
            copies,
        }
    }
}

/// What the members marked last have on one list of a literal's
/// occurrences.
#[derive(Clone, Copy, Debug)]
struct Marks {
    /// The stamp of the marking that last found a member on the list.
    stamp: u64,
    /// The first member that marking found there.
    first: OccId,
    /// The stamp of the marking whose initial rules last looked for the
    /// list's literal in `Θ`.
    tried: u64,
}

/// The members of an unrestricted zone in the order of their classes.
struct ByClass {
    /// The zone; `None` before the first.
    zone: Option<OccSet>,
    /// Its members with their classes, by class and then by id.
    members: Pooled<(OccId, OccId)>,
}

/// The state of one run: the problem, the memo, the proof arena, the
/// counters, the branch, and pools of scratch buffers so that no step
/// allocates once the pools are warm.
struct Engine<'a> {
    /// The problem.
    forest: &'a Forest,
    /// Its intuitionistic reading, for a two-sided search.
    reading: Option<&'a Reading<'a>>,
    /// Its count invariants.
    counts: &'a Counts,
    /// Its classes of interchangeable occurrences.
    classes: &'a Classes,
    /// The rules in force.
    rules: Rules,
    /// The memo of stable sequents.
    memo: Table<'a>,
    /// The proof arena.
    nodes: Arena<'a>,
    /// The search's account, which the memo is charged to.
    account: &'a Account,
    /// What the engine's own buffers were charged: the branch stack and
    /// the pools, which go with the engine.
    scratch: Charged<'a>,
    /// Whether the memo takes entries: a proof it refers to is kept.
    memoizes: bool,
    /// The counters.
    statistics: Statistics,
    /// The steps of split searches since the stop condition was last
    /// polled there.
    steps: u64,
    /// The forced splits, and the literals of the tensors they closed in
    /// place, since the stop condition was last polled in a chain of them.
    forced: u64,
    /// The work done since the stop condition was last polled, as far as
    /// it grows with the size of the sequents and is no step of a split
    /// search: the unit of a turn.
    work: u64,
    /// The nesting of engine calls right now.
    depth: u32,
    /// The deepest nesting allowed.
    recursion_limit: u32,
    /// The most copies a branch may take at the last deepening level.
    copies: u32,
    /// The stop condition.
    stop: Stop<'a>,
    /// The runtime, when the search runs on several threads.
    #[cfg(feature = "parallel")]
    runtime: Option<&'a super::parallel::Runtime>,
    /// How many choices among alternatives on this branch ran on several
    /// threads: the levels of cube-and-conquer above the current sequent.
    or_depth: u32,
    /// The stable sequents of the current branch, the root end first; only
    /// the first `stack_len` are live, the rest are spare buffers.
    stack: Vec<Key>,
    /// The hash of every entry of the stack, live or spare: the loop check
    /// compares hashes before sequents.
    hashes: Vec<u64>,
    /// How many entries of the stack are live.
    stack_len: usize,
    /// The stable sequents of the branch above the stack, the root end
    /// first, with their hashes: the parts of the branch stacks of the
    /// engines this worker of a pool was spawned from, read in place. The
    /// depth of an entry of `stack` counts these first.
    ancestors: Vec<(&'a [Key], &'a [u64])>,
    /// The spare buffers.
    pools: Pools,
    /// Per list of a literal's occurrences, twice the atom plus the sign
    /// as the forest numbers them: what the members marked last have on
    /// it. Empty until the first marking.
    lists: Vec<Marks>,
    /// The stamp of the members marked last. A search that marks a
    /// billion sets a second takes five centuries to wrap it.
    stamp: u64,
    /// The unrestricted zone of the stable sequent whose copies were
    /// listed last, with its members by class.
    by_class: ByClass,
}

impl<'a> Engine<'a> {
    /// Prepares a run on the problem with the stop condition, the memo and
    /// the arena to use, at the root of its branch; a pool's worker is one
    /// that is then given the branch it continues.
    fn new(problem: Problem<'a>, stop: Stop<'a>, memo: Table<'a>, nodes: Arena<'a>) -> Self {
        let Problem {
            forest,
            reading,
            counts,
            classes,
            rules,
            account,
            memoizes,
            recursion_limit,
            copies,
        } = problem;
        Self {
            forest,
            reading,
            counts,
            classes,
            rules,
            memo,
            account,
            scratch: Charged::new(account),
            nodes,
            memoizes,
            statistics: Statistics::default(),
            steps: 0,
            forced: 0,
            work: 0,
            depth: 0,
            recursion_limit,
            copies,
            stop,
            #[cfg(feature = "parallel")]
            runtime: None,
            or_depth: 0,
            stack: Vec::new(),
            hashes: Vec::new(),
            stack_len: 0,
            ancestors: Vec::new(),
            pools: Pools::default(),
            lists: Vec::new(),
            stamp: 0,
            by_class: ByClass {
                zone: None,
                members: Pooled::default(),
            },
        }
    }

    /// The problem this engine searches, as it was given.
    fn problem(&self) -> Problem<'a> {
        Problem {
            forest: self.forest,
            reading: self.reading,
            counts: self.counts,
            classes: self.classes,
            rules: self.rules,
            account: self.account,
            memoizes: self.memoizes,
            recursion_limit: self.recursion_limit,
            copies: self.copies,
        }
    }

    /// Returns the counters, with the memo's.
    fn statistics(&self) -> Statistics {
        Statistics {
            memo_hits: self.memo.hits(),
            memo_entries: self.memo.peak(),
            ..self.statistics
        }
    }

    /// Searches the goal with an empty unrestricted zone: the asynchronous
    /// phase on its formulas, once per copy bound from zero up to the
    /// configured one, until a level proves it or fails without ever
    /// spending its budget. Without exponentials there is one level.
    fn run(&mut self, goal: &[OccId]) -> Search {
        let levels = if self.rules.exponentials {
            self.copies
        } else {
            0
        };
        let theta = self.take_set();
        // Up to `u32::MAX` inclusive without a wrap; that bound stands for
        // none and is never reached (`Options::copy_bound`).
        for budget in 0..=levels {
            self.statistics.copies = budget;
            let mut gamma = self.take_context();
            let mut list = self.take_list();
            // A stack: the first formula is decomposed first.
            list.extend(goal.iter().rev());
            let result = self.asynchronous(&theta, &mut gamma, &mut list, budget);
            self.give_context(gamma);
            self.give_list(list);
            match result {
                Ok(Found::Failed(cuts)) if cuts.exhausted => {}
                decided => {
                    self.give_set(theta);
                    return decided.map(Found::node);
                }
            }
        }
        self.give_set(theta);
        Err(Reason::CopyBound(levels))
    }

    // The phases.

    /// The asynchronous phase on `⊢ Θ ; Γ ⇑ L`: decomposes the negative
    /// formulas of the list until the sequent is stable, then proves it.
    /// The list is a stack; the order does not matter for completeness.
    fn asynchronous(
        &mut self,
        theta: &OccSet,
        gamma: &mut Context,
        list: &mut Vec<OccId>,
        budget: u32,
    ) -> Step {
        self.enter()?;
        let result = self.decompose(theta, gamma, list, budget);
        self.leave();
        result
    }

    /// The body of the asynchronous phase. The `?` rules are applied in the
    /// loop like `⅋` and `⊥`, on a copy of the unrestricted zone that
    /// grows, so that a sequent of thousands of `?` formulas costs one
    /// level of recursion and not one each.
    fn decompose(
        &mut self,
        theta: &OccSet,
        gamma: &mut Context,
        list: &mut Vec<OccId>,
        budget: u32,
    ) -> Step {
        // The `⅋`, `⊥` and `?` rules applied, to wrap around the proof of
        // what remains; the first applied is the lowest.
        let mut applied = self.take_list();
        // The unrestricted zone once a `?` added a formula to it.
        let mut grown: Option<OccSet> = None;
        let result = loop {
            let Some(o) = list.pop() else {
                break self.prove(grown.as_ref().unwrap_or(theta), gamma, budget);
            };
            match self.forest.kind(o) {
                Kind::Par => {
                    list.push(self.forest.right(o).unwrap());
                    list.push(self.forest.left(o).unwrap());
                    applied.push(o);
                }
                Kind::Bot => applied.push(o),
                Kind::Top => break Ok(Found::proved(self.push(Node::Top(o)))),
                Kind::With => {
                    break self.with(grown.as_ref().unwrap_or(theta), gamma, list, o, budget);
                }
                Kind::Quest => {
                    // The subformula joins the unrestricted zone, which is
                    // a set: a formula already there changes nothing.
                    let a = self.forest.left(o).unwrap();
                    if !grown.as_ref().unwrap_or(theta).contains(a) {
                        if grown.is_none() {
                            let mut larger = self.take_set();
                            larger.clone_from(theta);
                            grown = Some(larger);
                        }
                        grown.as_mut().unwrap().insert(a);
                    }
                    applied.push(o);
                }
                _ => {
                    // A positive formula or a literal: part of the stable
                    // sequent.
                    gamma.insert(o);
                }
            }
        };
        if let Some(larger) = grown {
            self.give_set(larger);
        }
        let result = result.map(|found| {
            found.map(|mut node| {
                for &o in applied.iter().rev() {
                    node = self.push(match self.forest.kind(o) {
                        Kind::Par => Node::Par(o, node),
                        Kind::Quest => Node::Quest(o, node),
                        _ => Node::Bot(o, node),
                    });
                }
                node
            })
        });
        self.give_list(applied);
        result
    }

    /// The `&` rule: both premises with the same context, each keeping the
    /// whole copy budget; the right one is searched only once the left one
    /// is proved. On several threads both are searched at once.
    fn with(
        &mut self,
        theta: &OccSet,
        gamma: &mut Context,
        list: &mut Vec<OccId>,
        o: OccId,
        budget: u32,
    ) -> Step {
        #[cfg(feature = "parallel")]
        if self.cubes() {
            return self.with_parallel(theta, gamma, list, o, budget);
        }
        let mark = self.nodes.mark();
        let left = self.premise(theta, gamma, list, self.forest.left(o).unwrap(), budget)?;
        let Found::Proved(left_node) = left else {
            return Ok(left);
        };
        // The right premise on the state itself, which the rule leaves.
        list.push(self.forest.right(o).unwrap());
        let held = self.nodes.hold(left_node);
        let right = self.asynchronous(theta, gamma, list, budget)?;
        let left_node = self.nodes.unhold(held);
        Ok(self.both(o, left_node, right, mark))
    }

    /// A premise of the `&` rule on copies of the state: the context and
    /// the list with the subformula on top.
    fn premise(
        &mut self,
        theta: &OccSet,
        gamma: &Context,
        list: &[OccId],
        sub: OccId,
        budget: u32,
    ) -> Step {
        let mut premise_gamma = self.take_context_from(gamma);
        let mut premise_list = self.take_list();
        premise_list.extend_from_slice(list);
        premise_list.push(sub);
        let result = self.asynchronous(theta, &mut premise_gamma, &mut premise_list, budget);
        self.give_context(premise_gamma);
        self.give_list(premise_list);
        result
    }

    /// The `&` node on `o` from the proof of its left premise and what the
    /// search of the right one found; the right one's failure, with the
    /// nodes pending since `mark` released, when it failed.
    fn both(&mut self, o: OccId, left: NodeId, right: Found, mark: usize) -> Found {
        let Found::Proved(right) = right else {
            self.nodes.release(mark);
            return right;
        };
        Found::proved(self.push(Node::With(o, left, right)))
    }

    /// `prove(Θ ; Γ)` for a stable `Γ`: the memo, the loop check and the
    /// affine prune, the immediate tests, then a focus on each positive
    /// formula in turn, first from `Γ`, then copied from `Θ`, then Mix.
    fn prove(&mut self, theta: &OccSet, gamma: &Context, budget: u32) -> Step {
        Ok(self.prove_part(theta, gamma, budget)?.0)
    }

    /// [`Self::prove`], and whether a failure is hereditary: no non-empty
    /// sub-multiset of `Γ` is provable either.
    fn prove_part(
        &mut self,
        theta: &OccSet,
        gamma: &Context,
        budget: u32,
    ) -> Result<(Found, bool), Reason> {
        self.enter()?;
        let result = self.prove_stable(theta, gamma, budget);
        self.leave();
        result
    }

    /// The body of `prove`: the memo and the branch stack around the
    /// decision. Says whether a failure is hereditary.
    fn prove_stable(
        &mut self,
        theta: &OccSet,
        gamma: &Context,
        budget: u32,
    ) -> Result<(Found, bool), Reason> {
        self.statistics.nodes += 1;
        let work = NODE_WORK
            + (self.forest.len() / OCCURRENCES_PER_WORK) as u64
            + std::mem::take(&mut self.work);
        if self.stop.fired(work) {
            return Err(Reason::Stopped);
        }
        if self.account.over() {
            self.relieve()?;
        }
        // The memo and the loop check read the zones where they are, under
        // one hash.
        let key = Zones { theta, gamma };
        let hash = if self.memoizes || self.rules.stack {
            key.hash()
        } else {
            0
        };
        // Complete failures are recorded up to interchangeable members:
        // under the canonical key, where it differs from the sequent's own.
        let mut canonical = self.take_context_any();
        let renamed = !self.classes.distinct() && canonical.canonical_from(gamma, self.classes);
        let canonical_key = Zones {
            theta,
            gamma: &canonical,
        };
        let canonical_hash = if renamed && self.memoizes {
            canonical_key.hash()
        } else {
            0
        };
        // A proof or a complete failure from the memo settles it; a
        // failure cut by the budget waits for the loop check, which may
        // give the stronger answer that the branch is redundant.
        let refuted = if renamed {
            self.memo.refuted(canonical_key, canonical_hash)
        } else {
            None
        };
        let entry = match refuted {
            Some(failure) => Some(Entry::Failed(failure)),
            None => self.memo.get(key, hash, budget),
        };
        match entry {
            Some(Entry::Proved(node)) => {
                self.give_context(canonical);
                return Ok((Found::proved(node), false));
            }
            Some(Entry::Failed(failure @ (Failure::Complete | Failure::Hereditary))) => {
                self.give_context(canonical);
                return Ok((Found::NOTHING, failure == Failure::Hereditary));
            }
            Some(Entry::Failed(Failure::Exhausted(_))) | None => {}
        }
        // A sequent that repeats an ancestor on its branch is pruned: a
        // smallest proof of the ancestor never passes through it. The
        // failure this causes above is a fact about the branch, so it
        // carries a dependency on the ancestor's depth, which keeps the
        // sequents between them out of the memo, until the ancestor itself
        // is decided. (A sequent that merely contains an ancestor is not
        // redundant, with or without weakening: a proof of the larger
        // sequent proves nothing about the smaller one, and ⊢ ?(a ⅋ ~a) is
        // proved only through ⊢ a ⅋ ~a ; a, ~a.)
        if self.rules.stack
            && let Some(depth) = self.repeated(key, hash)
        {
            self.give_context(canonical);
            // A depth of the branch, which is as deep as the recursion
            // limit allows.
            return Ok((Found::failed(Cuts::repeat(depth as u32)), false));
        }
        if entry.is_some() {
            // Cut by the budget at this or a larger remaining budget.
            self.give_context(canonical);
            return Ok((Found::failed(Cuts::BUDGET), false));
        }
        if self.rules.stack {
            if self.stack_len < self.stack.len() {
                // Into the entry's own buffers: a derived `clone_from`
                // would allocate both zones anew.
                self.stack[self.stack_len].assign(key);
                self.hashes[self.stack_len] = hash;
            } else {
                self.scratch.charge(self.key_bytes() + size_of::<u64>());
                self.stack.push(Key {
                    theta: theta.clone(),
                    gamma: gamma.clone(),
                });
                self.hashes.push(hash);
            }
            self.stack_len += 1;
        }
        let mark = self.nodes.mark();
        let mut hereditary = false;
        let result = self.decide(theta, gamma, budget, &mut hereditary);
        let own_depth = if self.rules.stack {
            self.stack_len -= 1;
            (self.above() + self.stack_len) as u32
        } else {
            NO_DEPENDENCY
        };
        let found = match result {
            Ok(found) => found,
            Err(reason) => {
                self.give_context(canonical);
                return Err(reason);
            }
        };
        // What a failure rests on, as the steps it ran returned it. A
        // dependency on this very sequent is settled with it: the pruned
        // descendants could only have proved what this sequent proves.
        let (node, cuts) = match found {
            Found::Proved(node) => (Some(node), Cuts::NONE),
            Found::Failed(cuts) => (
                None,
                Cuts {
                    exhausted: cuts.exhausted,
                    dependency: if cuts.dependency >= own_depth {
                        NO_DEPENDENCY
                    } else {
                        cuts.dependency
                    },
                },
            ),
        };
        let canonical_key = Zones {
            theta,
            gamma: &canonical,
        };
        let recorded = match node {
            Some(proof) if self.memoizes => {
                // The entry outlives the branch, so the proof is kept.
                self.nodes.keep(mark, proof).and_then(|proof| {
                    match self.remember(key, hash, Entry::Proved(proof))? {
                        Entry::Proved(proof) => Ok(Some(proof)),
                        Entry::Failed(_) => unreachable!("the entry of a proof"),
                    }
                })
            }
            None => {
                self.nodes.release(mark);
                let mut recorded = Ok(None);
                if cuts.dependency == NO_DEPENDENCY {
                    // A complete failure answers for every relative; one
                    // cut by the budget stays the sequent's own.
                    let complete = if hereditary {
                        Failure::Hereditary
                    } else {
                        Failure::Complete
                    };
                    recorded = if cuts.exhausted {
                        self.remember(key, hash, Entry::Failed(Failure::Exhausted(budget)))
                    } else if renamed {
                        self.remember(canonical_key, canonical_hash, Entry::Failed(complete))
                    } else {
                        self.remember(key, hash, Entry::Failed(complete))
                    }
                    .map(|_| None);
                }
                recorded
            }
            Some(proof) => Ok(Some(proof)),
        };
        self.give_context(canonical);
        Ok(match recorded? {
            Some(node) => (Found::proved(node), false),
            None => (Found::failed(cuts), hereditary),
        })
    }

    /// The depth of the stable sequent of the branch that `key`, of hash
    /// `hash`, repeats, if any: the ancestors spawning engines hold first,
    /// then the stack's live entries.
    fn repeated(&self, key: Zones<'_>, hash: u64) -> Option<usize> {
        let own = (
            &self.stack[..self.stack_len],
            &self.hashes[..self.stack_len],
        );
        self.ancestors
            .iter()
            .chain([&own])
            .flat_map(|&(keys, hashes)| keys.iter().zip(hashes))
            .position(|(entry, &h)| h == hash && key.of(entry))
    }

    /// How many stable sequents of the branch lie above this engine's
    /// stack.
    fn above(&self) -> usize {
        self.ancestors.iter().map(|(keys, _)| keys.len()).sum()
    }

    /// Records what the search found out about a stable sequent. A memo
    /// of the engine's own that has no room is emptied first, and the
    /// kept proofs that only its entries referred to go with it; the
    /// entry comes back as recorded, a proof's node under the id it has
    /// after that. Fails when even the emptied memo has no room: the
    /// search's memory is at its bound without it.
    fn remember(&mut self, key: Zones<'_>, hash: u64, entry: Entry) -> Result<Entry, Reason> {
        if self.memo.insert(key, hash, entry, self.account) == Inserted::Done {
            return Ok(entry);
        }
        self.memo.clear();
        let entry = match entry {
            Entry::Proved(node) => Entry::Proved(
                self.nodes
                    .collect(Some(node), false)
                    .expect("the root given"),
            ),
            failed => {
                self.nodes.collect(None, false);
                failed
            }
        };
        if self.memo.insert(key, hash, entry, self.account) == Inserted::Done {
            return Ok(entry);
        }
        // Not even the first entry fits: what the proofs dropped leave
        // free is the last memory there is.
        let entry = match entry {
            Entry::Proved(node) => Entry::Proved(
                self.nodes
                    .collect(Some(node), true)
                    .expect("the root given"),
            ),
            failed => {
                self.nodes.collect(None, true);
                failed
            }
        };
        match self.memo.insert(key, hash, entry, self.account) {
            Inserted::Done => Ok(entry),
            _ => Err(Reason::MemoryLimit(self.account.limit())),
        }
    }

    /// Makes room when the search holds more than its bound: the memo is
    /// emptied, and the kept proofs that only its entries referred to are
    /// dropped; if that is not enough, the memo's own memory is given
    /// back, which it takes again as far as the rest leaves room. Fails
    /// when what is left, the branch's own buffers and proofs as they are
    /// allocated, is still over the bound: squeezing those would be undone
    /// by the next node.
    fn relieve(&mut self) -> Result<(), Reason> {
        self.memo.clear();
        self.nodes.collect(None, false);
        if !self.account.over() {
            return Ok(());
        }
        self.memo.release(self.account);
        if self.account.over() {
            Err(Reason::MemoryLimit(self.account.limit()))
        } else {
            Ok(())
        }
    }

    /// The bytes a memo key of the forest's width allocates.
    fn key_bytes(&self) -> usize {
        2 * self.set_bytes()
    }

    /// The bytes a set of the forest's width allocates.
    fn set_bytes(&self) -> usize {
        self.forest.len().div_ceil(64) * size_of::<u64>()
    }

    /// Decides a stable sequent the memo does not know, and sets
    /// `hereditary`, under Mix, when it fails and no non-empty part of `Γ`
    /// is provable either: `Γ` has one member, or Mix found each part with
    /// one member less hereditarily failed.
    fn decide(
        &mut self,
        theta: &OccSet,
        gamma: &Context,
        budget: u32,
        hereditary: &mut bool,
    ) -> Step {
        let mut members = self.take_list();
        members.extend(gamma.iter());
        let mut tally = self.take_tally();
        let mut candidates = self.take_list();
        let mut copies = self.take_list();
        let result = self.decide_with(
            theta,
            gamma,
            &members,
            &mut tally,
            &mut candidates,
            &mut copies,
            budget,
            hereditary,
        );
        if self.rules.mix && members.len() == 1 && matches!(result, Ok(Found::Failed(_))) {
            *hereditary = true;
        }
        self.give_list(members);
        self.give_tally(tally);
        self.give_list(candidates);
        self.give_list(copies);
        result
    }

    /// The body of `decide`, with its scratch buffers.
    #[allow(clippy::too_many_arguments)]
    fn decide_with(
        &mut self,
        theta: &OccSet,
        gamma: &Context,
        members: &[OccId],
        tally: &mut Tally,
        candidates: &mut Vec<OccId>,
        copies: &mut Vec<OccId>,
        budget: u32,
        hereditary: &mut bool,
    ) -> Step {
        let affine = self.rules.affine;
        // One pass over the members: the counts, a `0`, and the positive
        // formulas worth focusing on. `1` and `!` only when alone, since
        // they need an empty context (any context, with weakening); a
        // literal never, since a focus on it succeeds only in the initial
        // cases tested below.
        self.work += members.len() as u64;
        let mut zero = false;
        for &o in members {
            tally.add(self.counts, o);
            match self.forest.kind(o) {
                Kind::Zero => zero = true,
                Kind::Tensor | Kind::Plus => candidates.push(o),
                Kind::One | Kind::Bang if members.len() == 1 || affine => candidates.push(o),
                Kind::One | Kind::Bang | Kind::Var | Kind::DualVar => {}
                kind => unreachable!("{kind:?} in a stable sequent"),
            }
        }
        // A `0` has no rule, so only a `⊤` below some member of `Γ`, or
        // below a member of `Θ` that a copy can bring in, can prove the
        // sequent: ⊢ 0, ⊤ ⊕ b is provable. (The spec calls a `0` in a
        // stable sequent fatal, which overlooks this.) With weakening the
        // `0` is discarded at a leaf like anything else.
        if zero && !affine && !tally.absorbs() && !theta.iter().any(|a| self.counts.absorbs(a)) {
            return Ok(Found::NOTHING);
        }
        let mut cuts = match self.initial(theta, gamma, members, budget)? {
            Found::Failed(cuts) => cuts,
            proved => return Ok(proved),
        };
        if self.rules.intervals && !tally.balanced() {
            return Ok(Found::failed(cuts));
        }
        if self.rules.equation && !tally.equation(self.rules.mix) {
            return Ok(Found::failed(cuts));
        }

        // Of interchangeable candidates one is enough: the focus on either
        // leaves the same sequent up to a renaming.
        self.one_of_each(candidates);
        // Forced splits first, then `⊕`, then the free splits; by id within
        // a class, so that the run is deterministic.
        candidates.sort_by_key(|&o| (self.focus_class(o), o));
        // After them the copies from `Θ`, a formula with an unconsumed copy
        // in `Γ` skipped (a second copy cannot help before the first is
        // used), those that can meet a literal of `Γ` first, then by id.
        if self.rules.exponentials {
            let unconsumed = self.one_of_each_copy(theta, gamma, copies);
            // Each is looked up in `Γ`, compared with its class and sorted.
            self.work += 2 * unconsumed as u64;
            if budget == 0 {
                // A branch cut by the bound: the level cannot claim
                // completeness, unless nothing was there to copy.
                if !copies.is_empty() {
                    cuts = cuts.and(Cuts::BUDGET);
                }
                copies.clear();
            } else {
                // By id, those that meet a member first: the heuristic is
                // asked once per formula, not once per comparison.
                copies.sort_unstable();
                self.work += (copies.len() * members.len() / MEETS_PER_WORK) as u64;
                self.mark_literals(members);
                let mut others = self.take_list();
                let mut met = 0;
                for i in 0..copies.len() {
                    let a = copies[i];
                    if self.meets(a) {
                        copies[met] = a;
                        met += 1;
                    } else {
                        others.push(a);
                    }
                }
                copies.truncate(met);
                copies.extend_from_slice(&others);
                self.give_list(others);
            }
        }
        let alternatives = candidates
            .iter()
            .map(|&f| Alternative::Focus(f))
            .chain(copies.iter().map(|&a| Alternative::Copy(a)));
        let cuts = match self.choose(theta, gamma, alternatives, budget)?.after(cuts) {
            Found::Failed(cuts) => cuts,
            proved => return Ok(proved),
        };
        Ok(self
            .last_resort(theta, gamma, members, tally, budget, hereditary)?
            .after(cuts))
    }

    /// Decides a choice among alternatives: in their order on this thread,
    /// the first that proves the sequent ending it, or within the first
    /// levels of a pool's search on its threads. The cuts are those of
    /// the alternatives searched.
    fn choose<'m>(
        &mut self,
        theta: &OccSet,
        gamma: &Context,
        alternatives: impl Iterator<Item = Alternative<'m>>,
        budget: u32,
    ) -> Step {
        #[cfg(feature = "parallel")]
        if self.cubes() {
            let alternatives: Vec<Alternative<'m>> = alternatives.collect();
            if alternatives.len() >= 2 {
                return self.choose_parallel(theta, gamma, &alternatives, budget);
            }
            return self.choose_here(theta, gamma, alternatives.into_iter(), budget);
        }
        self.choose_here(theta, gamma, alternatives, budget)
    }

    /// Decides a choice among alternatives on this engine, in their order.
    fn choose_here<'m>(
        &mut self,
        theta: &OccSet,
        gamma: &Context,
        alternatives: impl Iterator<Item = Alternative<'m>>,
        budget: u32,
    ) -> Step {
        let mut cuts = Cuts::NONE;
        let mut rest = None;
        for alternative in alternatives {
            match self.alternative(theta, gamma, alternative, &mut rest, budget)? {
                Found::Failed(failed) => cuts = cuts.and(failed),
                proved => {
                    if let Some(rest) = rest {
                        self.give_context(rest);
                    }
                    return Ok(proved);
                }
            }
        }
        if let Some(rest) = rest {
            self.give_context(rest);
        }
        Ok(Found::failed(cuts))
    }

    /// Searches one alternative of a choice on this engine. `rest` is the
    /// context a focus on a member of `Γ` leaves, taken from the pool the
    /// first time one needs it.
    fn alternative(
        &mut self,
        theta: &OccSet,
        gamma: &Context,
        alternative: Alternative<'_>,
        rest: &mut Option<Context>,
        budget: u32,
    ) -> Step {
        match alternative {
            Alternative::Focus(f) => {
                if rest.is_none() {
                    *rest = Some(self.take_context_any());
                }
                let rest = rest.as_mut().expect("taken");
                rest.clone_from(gamma);
                rest.remove(f);
                self.focus(theta, rest, f, budget)
            }
            Alternative::Copy(a) => Ok(self
                .focus(theta, gamma, a, budget - 1)?
                .map(|node| self.push(Node::Copy(a, node)))),
            Alternative::Side(f, side, sub) => Ok(self
                .focus(theta, gamma, sub, budget)?
                .map(|node| self.push(Node::Plus(f, side, node)))),
            Alternative::Splits {
                join,
                members,
                fixed,
                pattern,
                sides,
                split,
            } => {
                let mut left = self.take_context_from(sides.0);
                let mut right = self.take_context_from(sides.1);
                let mut counts = self.take_split();
                (*counts).clone_from(split);
                for (i, &m) in members.iter().enumerate().take(fixed) {
                    if pattern >> i & 1 == 1 {
                        right.remove(m);
                        left.insert(m);
                        counts.assign(self.counts, m, Side::Left);
                    } else {
                        counts.assign(self.counts, m, Side::Right);
                    }
                }
                let result = self.search_splits(
                    theta,
                    members,
                    (fixed, pattern),
                    (&mut left, &mut right),
                    &mut counts,
                    join,
                    budget,
                );
                self.give_context(left);
                self.give_context(right);
                self.give_split(counts);
                result
            }
        }
    }

    /// Lists in `copies`, of the members of `Θ` without a copy in `Γ`, one
    /// of each class, the lowest id, in the order of their classes, which
    /// is [`Self::one_of_each`] on them; returns how many members `Θ` has
    /// without a copy in `Γ`. `Θ`'s members by class are kept from the
    /// stable sequent before when it had the same `Θ`, as along a branch
    /// it mostly has: sorting them anew for every stable sequent was most
    /// of what one cost on a net of thousands of transitions.
    fn one_of_each_copy(
        &mut self,
        theta: &OccSet,
        gamma: &Context,
        copies: &mut Vec<OccId>,
    ) -> usize {
        let by_class = &mut self.by_class;
        if by_class.zone.as_ref() != Some(theta) {
            match &mut by_class.zone {
                Some(zone) => zone.clone_from(theta),
                None => {
                    self.scratch
                        .charge(self.forest.len().div_ceil(64) * size_of::<u64>());
                    by_class.zone = Some(theta.clone());
                }
            }
            by_class.members.clear();
            by_class
                .members
                .extend(theta.iter().map(|a| (self.classes.of(a), a)));
            by_class.members.sort_unstable();
            by_class.members.settle(&mut self.scratch);
        }
        let mut unconsumed = 0;
        let mut last = None;
        for &(class, a) in self.by_class.members.iter() {
            if gamma.contains(a) {
                continue;
            }
            unconsumed += 1;
            if last != Some(class) {
                copies.push(a);
                last = Some(class);
            }
        }
        unconsumed
    }

    /// Keeps, of the occurrences that are interchangeable, the one with
    /// the lowest id.
    fn one_of_each(&self, occurrences: &mut Vec<OccId>) {
        occurrences.sort_unstable_by_key(|&o| (self.classes.of(o), o));
        occurrences.dedup_by_key(|o| self.classes.of(*o));
    }

    /// What is tried on a stable sequent after every focus failed: Mix,
    /// when the rules have it, which sets `hereditary` when it fails
    /// because no part of `Γ` is provable.
    fn last_resort(
        &mut self,
        theta: &OccSet,
        gamma: &Context,
        members: &[OccId],
        tally: &Tally,
        budget: u32,
        hereditary: &mut bool,
    ) -> Step {
        if self.rules.mix {
            return self.mix(theta, gamma, members, tally, budget, hereditary);
        }
        Ok(Found::NOTHING)
    }

    /// The initial rules on a stable sequent: a dual pair in `Γ`, or a
    /// literal of `Γ` whose dual lies in `Θ`, which is copied and counts
    /// against the budget; exactly that in linear mode, with anything else
    /// in `Γ` weakened away in affine mode. The first literal of the
    /// members, in their order, with a dual among them pairs with the first
    /// such dual, else the first with a dual in `Θ` is copied. One pass
    /// marks the lists of the members' literals, so that the test is
    /// linear in the members: comparing every pair was quadratic under
    /// weakening, where any number of members may close a sequent.
    fn initial(&mut self, theta: &OccSet, gamma: &Context, members: &[OccId], budget: u32) -> Step {
        let affine = self.rules.affine;
        if !affine && members.len() > 2 {
            return Ok(Found::NOTHING);
        }
        self.mark_literals(members);
        let f = self.forest;
        let mut cuts = Cuts::NONE;
        for &p in members {
            let (Some(atom), Some(sign)) = (f.atom(p), f.sign(p)) else {
                continue;
            };
            let dual = 2 * atom.index() + (!sign) as usize;
            // No member before `p` has a dual among the members, else the
            // pair would have closed the sequent there, so `p`'s first dual
            // comes after it.
            if (affine || members.len() == 2) && self.lists[dual].stamp == self.stamp {
                let q = self.lists[dual].first;
                let ax = self.push(Node::Ax(p, q));
                if !affine {
                    return Ok(Found::proved(ax));
                }
                let mut rest = self.take_context_from(gamma);
                rest.remove(p);
                rest.remove(q);
                let node = self.weakened(&rest, ax);
                self.give_context(rest);
                return Ok(Found::proved(node));
            }
            // Without exponentials `Θ` is empty; with them, a literal's
            // dual is looked up in `Θ` once, since another copy of it finds
            // what the first found.
            if !(affine || members.len() == 1)
                || !self.rules.exponentials
                || self.lists[dual].tried == self.stamp
            {
                continue;
            }
            self.lists[dual].tried = self.stamp;
            if let Some(d) = self.dual_in(p, |d| theta.contains(d)) {
                if budget == 0 {
                    // Another pair may still close the sequent without a
                    // copy.
                    cuts = Cuts::BUDGET;
                    continue;
                }
                let ax = self.push(Node::Ax(p, d));
                let copy = self.push(Node::Copy(d, ax));
                if !affine {
                    return Ok(Found::proved(copy));
                }
                let mut rest = self.take_context_from(gamma);
                rest.remove(p);
                let node = self.weakened(&rest, copy);
                self.give_context(rest);
                return Ok(Found::proved(node));
            }
        }
        Ok(Found::failed(cuts))
    }

    /// The first occurrence of the literal dual to `literal` that is `within`
    /// a zone, found through the forest's list of that literal's
    /// occurrences rather than through the zone's members.
    fn dual_in(&self, literal: OccId, within: impl Fn(OccId) -> bool) -> Option<OccId> {
        let f = self.forest;
        let (atom, sign) = (f.atom(literal)?, f.sign(literal)?);
        f.literals(atom, !sign).iter().copied().find(|&d| within(d))
    }

    /// Marks the lists of the literals among the members of a stable
    /// sequent, under a stamp of their own, with the first member on each,
    /// for [`Self::initial`] and [`Self::meets`]. One pass over the
    /// members, so that ranking the copies costs the members and the
    /// formulas of `Θ` once each and not their product, which on a
    /// marking of thousands of tokens under clauses of thousands of
    /// literals was a quarter of a second per stable sequent.
    fn mark_literals(&mut self, members: &[OccId]) {
        let f = self.forest;
        if self.lists.is_empty() {
            let lists = 2 * f.sequent().atom_names().len();
            self.scratch.charge(lists * size_of::<Marks>());
            self.lists = vec![
                Marks {
                    stamp: 0,
                    first: OccId::new(0),
                    tried: 0,
                };
                lists
            ];
        }
        self.stamp += 1;
        for &m in members {
            if let (Some(atom), Some(sign)) = (f.atom(m), f.sign(m)) {
                let list = &mut self.lists[2 * atom.index() + sign as usize];
                if list.stamp != self.stamp {
                    list.stamp = self.stamp;
                    list.first = m;
                }
            }
        }
    }

    /// Whether a formula of `Θ` has a literal below it whose dual is a
    /// member of the stable sequent marked last: the copy heuristic's
    /// notion of a copy that can meet something.
    fn meets(&self, a: OccId) -> bool {
        let f = self.forest;
        f.subtree(a).any(|l| match (f.atom(l), f.sign(l)) {
            (Some(atom), Some(sign)) => {
                self.lists[2 * atom.index() + (!sign) as usize].stamp == self.stamp
            }
            _ => false,
        })
    }

    /// Orders the focus candidates: `1` and `!`, then a `⊗` whose split is
    /// forced by a factor (a positive literal, `1`, `!`; or `0`, which
    /// fails at once), then `⊕`, then a `⊗` whose split must be searched.
    fn focus_class(&self, o: OccId) -> u8 {
        match self.forest.kind(o) {
            Kind::One | Kind::Bang => 0,
            Kind::Tensor => {
                let forced = self
                    .forest
                    .children(o)
                    .any(|c| self.forced_side(c).is_some());
                if forced { 1 } else { 3 }
            }
            _ => 2,
        }
    }

    /// `focus(Θ ; Γ ⇓ F)` with `F` taken out of `Γ` (or copied from `Θ`):
    /// the positive rules along `F`, and a release into the asynchronous
    /// phase once `F` is negative.
    fn focus(&mut self, theta: &OccSet, gamma: &Context, f: OccId, budget: u32) -> Step {
        self.enter()?;
        let result = self.focus_on(theta, gamma, f, budget);
        self.leave();
        result
    }

    /// The body of `focus`.
    fn focus_on(&mut self, theta: &OccSet, gamma: &Context, f: OccId, budget: u32) -> Step {
        match self.forest.kind(f) {
            Kind::Plus => {
                let sides = [
                    Alternative::Side(f, Side::Left, self.forest.left(f).unwrap()),
                    Alternative::Side(f, Side::Right, self.forest.right(f).unwrap()),
                ];
                self.choose(theta, gamma, sides.into_iter(), budget)
            }
            Kind::Tensor => self.split(theta, gamma, f, budget),
            Kind::One if self.leftover(gamma) => {
                let one = self.push(Node::One(f));
                Ok(Found::proved(self.weakened(gamma, one)))
            }
            Kind::One => Ok(Found::NOTHING),
            Kind::Bang => {
                // Promotion: the subformula is released into an empty
                // linear zone, and what was there is weakened below.
                if !self.leftover(gamma) {
                    return Ok(Found::NOTHING);
                }
                let mut released = self.take_context();
                let mut list = self.take_list();
                list.push(self.forest.left(f).unwrap());
                let result = self.asynchronous(theta, &mut released, &mut list, budget);
                self.give_context(released);
                self.give_list(list);
                Ok(result?.map(|node| {
                    let bang = self.push(Node::Bang(f, node));
                    self.weakened(gamma, bang)
                }))
            }
            Kind::Zero => Ok(Found::NOTHING),
            Kind::Var | Kind::DualVar if self.counts.positive(self.forest, f) => {
                // The initial rules: the context is the dual literal, or
                // nothing and the dual lies in `Θ`.
                let mut members = self.take_list();
                members.push(f);
                members.extend(gamma.iter());
                let mut whole = self.take_context_from(gamma);
                whole.insert(f);
                let result = self.initial(theta, &whole, &members, budget);
                self.give_list(members);
                self.give_context(whole);
                result
            }
            _ => {
                // Negative: release.
                let mut released = self.take_context_from(gamma);
                let mut list = self.take_list();
                list.push(f);
                let result = self.asynchronous(theta, &mut released, &mut list, budget);
                self.give_context(released);
                self.give_list(list);
                result
            }
        }
    }

    /// Whether a rule that needs an empty linear zone can be applied with
    /// this one: it is empty, or weakening will discard it.
    fn leftover(&self, gamma: &Context) -> bool {
        gamma.is_empty() || self.rules.affine
    }

    /// Weakens every member of `gamma` below `node`, one `Weaken` per copy;
    /// nothing when it is empty.
    fn weakened(&mut self, gamma: &Context, mut node: NodeId) -> NodeId {
        debug_assert!(
            gamma.is_empty() || self.rules.affine,
            "weakening needs affine mode"
        );
        debug_assert!(
            self.reading.is_none_or(|r| r.outputs(gamma.iter()) == 0),
            "a leaf is the goal, so only hypotheses are left over"
        );
        for o in gamma.iter() {
            node = self.push(Node::Weaken(o, node));
        }
        node
    }

    // Bookkeeping.

    /// Appends a node to the pending ones and returns its id.
    fn push(&mut self, node: Node) -> NodeId {
        self.nodes.push(node)
    }

    /// Enters a nested engine call, unless the nesting is at its limit.
    fn enter(&mut self) -> Result<(), Reason> {
        if self.depth >= self.recursion_limit {
            return Err(Reason::RecursionLimit);
        }
        self.depth += 1;
        Ok(())
    }

    /// Leaves a nested engine call.
    fn leave(&mut self) {
        self.depth -= 1;
    }
}
