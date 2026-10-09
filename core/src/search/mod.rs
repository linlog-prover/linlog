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
use crate::limits::{Limits, Phase, Progress};
use crate::nets::ProofStructure;
use crate::occurrences::{Forest, Member, OccId, Reading};
use crate::proofs::{Bytes, CheckError, Node, NodeId, Proof};
use crate::sequents::{Atom, Sequent};
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::num::NonZeroUsize;
use std::str::FromStr;

/// An engine's stop condition: the caller's closure in a sequential
/// search, the chain of stop flags of a worker in a parallel one.
pub(crate) enum Stop<'a> {
    /// The caller's condition.
    Closure(&'a mut dyn FnMut() -> bool),
    /// The caller's condition, and an amount of work after which the
    /// search stops as well: one turn of a search that takes turns with
    /// another.
    Turn(&'a mut dyn FnMut() -> bool, u64),
    /// A condition that is told, at every poll, whether a slice of work
    /// (the last field) has passed since it was last told so: one of two
    /// searches that alternate, which gives way to the other there.
    #[cfg_attr(
        not(feature = "parallel"),
        expect(
            dead_code,
            reason = "only threads alternate the two searches in slices"
        )
    )]
    Slice(&'a mut dyn FnMut(bool) -> bool, u64, u64),
    /// A worker's flags.
    #[cfg(feature = "parallel")]
    Flags(parallel::Flags<'a>),
}

impl Stop<'_> {
    /// Polls the condition after `work` more units of work, in the unit
    /// the engine that polls counts its turns in.
    pub(crate) fn fired(&mut self, work: u64) -> bool {
        match self {
            Self::Closure(stop) => stop(),
            Self::Turn(stop, left) => {
                *left = left.saturating_sub(work);
                *left == 0 || stop()
            }
            Self::Slice(stop, left, slice) => {
                *left = left.saturating_sub(work);
                let passed = *left == 0;
                if passed {
                    *left = *slice;
                }
                stop(passed)
            }
            #[cfg(feature = "parallel")]
            Self::Flags(flags) => flags.raised(),
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
pub(crate) fn set_up_stopped(forest: &Forest, stop: &mut dyn FnMut() -> bool) -> bool {
    forest.len() >= SET_UP_POLL && stop()
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
/// ([`Error::IntuitionisticMix`]); the net engine outside unit-free MLL
/// ([`Error::NetFragment`]) and in affine mode ([`Error::NetMode`]); the
/// focus engine in intuitionistic mode and the two-sided engine in
/// classical mode ([`Error::EngineMode`]); the additive engine on anything
/// but two additive-only formulas ([`Error::NotAdditive`]); and a sequent
/// that unfolds to more subformula occurrences than
/// `limits.occurrences` allows ([`Refusal::Occurrences`](crate::Refusal::Occurrences)). A proof that the checker rejects is
/// [`Error::Rejected`], and one whose check would hold more than
/// `limits.memory_bytes` is [`Error::Check`] with a [`CheckError::Refused`]: every proof returned
/// has passed the checker, unless [`Options::check`] says otherwise.
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
    prove_goal(&forest, forest.roots(), mode, options, limits, stop)
}

/// Asks the caller's stop as the engines poll it: in the search's phase,
/// with no work counted yet.
pub(crate) fn without_progress(
    stop: &mut impl FnMut(Progress) -> bool,
) -> impl FnMut() -> bool + '_ {
    move || stop(Progress::new(Phase::Search, 0, 0))
}

/// Decides a goal: a multiset of occurrences of a forest, given in any
/// order, which stands for the sequent of those subformulas, within
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
/// Those of [`prove`], plus [`Error::IndexOutOfBounds`] for an
/// occurrence outside the forest, [`Error::GoalOutputs`] for an
/// intuitionistic goal without exactly one formula on the right of `⊢`,
/// and [`Error::NetGoal`] for the net engine forced on a goal other than
/// the roots.
///
/// # Examples
///
#[cfg_attr(feature = "parse", doc = "```")]
#[cfg_attr(not(feature = "parse"), doc = "```ignore")]
/// use linlog::search::{Options, Verdict, prove_goal};
/// use linlog::{Forest, Limits, Mode, OccId, Sequent};
///
/// // ⊢ ~A, A ⊗ ~B, B, with the occurrences 0: ~A, 1: A ⊗ ~B, 2: A,
/// // 3: ~B, 4: B. The goal ⊢ ~A, A is the left premise of the ⊗.
/// let sequent: Sequent = "A, A -o B |- B".parse()?;
/// let forest = Forest::new(&sequent)?;
/// let goal = [OccId::new(0), OccId::new(2)];
/// let limits = Limits::default();
/// let outcome = prove_goal(&forest, &goal, Mode::CLASSICAL, &Options::default(), &limits, |_| false)?;
/// assert!(matches!(outcome.verdict, Verdict::Proved(_)));
/// # Ok::<(), linlog::Error>(())
/// ```
pub fn prove_goal(
    forest: &Forest,
    goal: &[OccId],
    mode: Mode,
    options: &Options,
    limits: &Limits,
    mut stop: impl FnMut(Progress) -> bool,
) -> Result<Outcome, Error> {
    let fragment = fragment_of(forest, goal, options)?;
    let reading = read(forest, mode)?;
    let (task, engine) = prepare(forest, goal, mode, fragment, options, reading.as_ref())?;
    let roots = task.roots;
    let implementation = engine.implementation();
    // The engines poll a condition without progress, which asks the
    // caller's with the search's phase.
    let mut polled = without_progress(&mut stop);
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
    let account = memory::Account::new(limits.memory_bytes);
    #[cfg(feature = "parallel")]
    let options = &options
        .clone()
        .with_jobs(parallel::threads(options.threads()));
    let answer = implementation.decide(&task, options, limits, &account, &mut polled)?;
    // The one place an engine's answer becomes a verdict. A refutation
    // says what the counts of the goal rule out, under the same limits as
    // the search.
    let verdict = match answer.result {
        // A proof records what it concludes and the mode it was found in.
        Ok(Some(proof)) if roots => Verdict::Proved(Box::new(proof.with_mode(mode))),
        Ok(Some(proof)) => Verdict::Proved(Box::new(proof.concluding(goal).with_mode(mode))),
        Ok(None) => {
            let refutation = match answer.refutation {
                Some(refutation) => refutation,
                None => {
                    let account = memory::Account::new(limits.memory_bytes);
                    focus::refutation(forest, task.goal, fragment, mode, &account, &mut polled)
                }
            };
            let disproof = Disproof::new(forest.sequent().clone(), mode, refutation);
            Verdict::Unprovable(Box::new(if roots {
                disproof
            } else {
                disproof.of_goal(goal.iter().map(|&o| Member::from(o)).collect())
            }))
        }
        Err(reason) => Verdict::Unknown(reason),
    };
    drop(polled);
    // No engine is trusted with its own proof: the checker has the last
    // word on every proof, of the sequent or of a goal, in every build.
    if let Verdict::Proved(proof) = &verdict {
        if options.check {
            proof
                .check_within(mode, limits, &mut stop)
                .map_err(|e| match e {
                    // A check given up is no verdict on the proof.
                    CheckError::Refused(_) => Error::Check(e),
                    CheckError::Invalid(_) => Error::Rejected(Box::new(e)),
                })?;
        } else {
            debug_assert_eq!(proof.check(mode), Ok(()), "the {engine} engine's proof");
        }
    }
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
pub fn engine_for(
    forest: &Forest,
    goal: &[OccId],
    mode: Mode,
    options: &Options,
) -> Result<Engine, Error> {
    let fragment = fragment_of(forest, goal, options)?;
    let reading = read(forest, mode)?;
    prepare(forest, goal, mode, fragment, options, reading.as_ref()).map(|(_, engine)| engine)
}

/// The fragment a goal is searched in, the options' or its own, once its
/// occurrences are checked.
fn fragment_of(forest: &Forest, goal: &[OccId], options: &Options) -> Result<Fragment, Error> {
    if let Some(o) = goal.iter().find(|o| o.index() >= forest.len()) {
        return Err(Error::IndexOutOfBounds {
            space: crate::limits::Space::Occurrence,
            index: o.index(),
            len: forest.len(),
        });
    }
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
    forest: &'a Forest,
    goal: &'a [OccId],
    mode: Mode,
    fragment: Fragment,
    options: &Options,
    reading: Option<&'a Reading<'a>>,
) -> Result<(Task<'a>, Engine), Error> {
    if let Some(reading) = reading {
        let outputs = reading.outputs(goal.iter().copied());
        if outputs != 1 {
            return Err(Error::GoalOutputs { count: outputs });
        }
    }
    // The roots in any order are the sequent itself, which the engines
    // are handed in the forest's order.
    let roots = is_roots(forest, goal);
    let task = Task {
        forest,
        goal: if roots { forest.roots() } else { goal },
        fragment,
        mode,
        reading,
        roots,
    };
    let engine = options.engine.unwrap_or_else(|| dispatch(&task));
    engine.implementation().admits(&task)?;
    Ok((task, engine))
}

/// Whether a goal is the forest's roots, in any order.
fn is_roots(forest: &Forest, goal: &[OccId]) -> bool {
    let roots = forest.roots();
    if goal.len() != roots.len() {
        return false;
    }
    if goal == roots {
        return true;
    }
    let mut sorted = goal.to_vec();
    sorted.sort_unstable();
    let mut roots = roots.to_vec();
    roots.sort_unstable();
    sorted == roots
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
        (result, nodes, statistics): (Result<Option<NodeId>, Reason>, Vec<Node>, Statistics),
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

    /// Decides the goal under the options, polling `stop`, and charges
    /// what the search holds to `account`.
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
        stop: &mut dyn FnMut() -> bool,
    ) -> Result<Answer, Error>;
}

/// The engine the dispatch picks for a goal: that of the first row of
/// [`DISPATCH`] that takes it.
fn dispatch(task: &Task<'_>) -> Engine {
    DISPATCH
        .iter()
        .find(|row| row.takes(task))
        .map(|row| row.engine)
        .expect("the last two rows take every goal")
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
    Net,
    /// The focused sequent engine two-sided: intuitionistic mode. It is
    /// the search of `Focus` on the one-sided sequent, which keeps one goal
    /// on every branch by itself except at the split of a hypothesis
    /// `A ⊸ B`, where the goal must go with `B`: the one place it reads the
    /// sequent's [`Reading`]. It reads the options `Focus` does.
    TwoSided,
    /// The fast path for a sequent of two additive-only formulas, in every
    /// mode: a recursion on pairs of subformula occurrences, one below each
    /// root, memoized on the pair, in time proportional to the product of
    /// the two formulas' sizes. It reads [`Options::memo_limit`] and
    /// [`Options::check`], and the limits' recursion depth and memory
    /// bound, and runs on the calling thread whatever
    /// [`Options::jobs`] says; two additive formulas have no copies and no
    /// atoms to bias.
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
    /// and recurses nowhere.
    Horn,
}

impl Engine {
    /// Whether the engine searches on several threads when
    /// [`Options::jobs`] asks for them: the focused engines and the net
    /// engine do; the additive and the Horn engine run on the calling
    /// thread whatever it says.
    pub fn parallel(self) -> bool {
        !matches!(self, Engine::Additive | Engine::Horn)
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
    /// two searches together decide 1 520, the backward one alone 442,
    /// the forward one alone 1 576, and the pair costs 1.3 times the
    /// backward search and 1.6 times the forward one where each decides).
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
    pub const ALL: [Self; 5] = [
        Self::Focus,
        Self::Net,
        Self::TwoSided,
        Self::Additive,
        Self::Horn,
    ];

    /// The engines' names, as [`name`](Self::name) writes them and
    /// [`FromStr`] reads them.
    pub const NAMES: &'static [&'static str] = &["focus", "net", "two-sided", "additive", "horn"];

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
            .into_iter()
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
    pub const ALL: [Self; 3] = [Self::Auto, Self::Rarer, Self::Factors];

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
            .into_iter()
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
/// stack; the bounds on memory, occurrences, recursion and work are the
/// [`Limits`] a search is given.
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
    /// focused engine reads it: the goals of the others have no
    /// exponentials.
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
/// n}`, `{"kind": "recursion_limit"}`, `{"kind": "index_limit"}`),
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Reason {
    /// The caller's stop condition fired: a time limit or an interruption.
    Stopped,
    /// The nesting of engine calls reached
    /// [`Limits::recursion_depth`](crate::Limits::recursion_depth).
    RecursionLimit,
    /// Every level up to the bound [`Options::copies`] set, which is this
    /// value, hit its bound on some branch, so a proof with more copies of
    /// a `?` formula per branch may exist.
    CopyBound(u32),
    /// The search held [`Limits::memory_bytes`](crate::Limits::memory_bytes) bytes, which is this
    /// value, with its memo already emptied, or had no room left for a
    /// memo at all.
    MemoryLimit(u64),
    /// A structure of the search outgrew what its indices address: the
    /// proof arena at 2³¹ nodes, the count invariants at 2³² row entries,
    /// the Horn engine's markings at 2³² or a count of its tokens at 2³².
    /// Only a search without a memory bound gets this far, but for the
    /// tokens, which a clause that adds thousands of them at each firing
    /// can pass in a few hundred thousand markings.
    IndexLimit,
}

impl Display for Reason {
    /// Writes the reason as a phrase, such as `the time limit was reached`.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Reason::Stopped => f.write_str("the search was stopped"),
            Reason::RecursionLimit => f.write_str("the recursion limit was reached"),
            Reason::CopyBound(n) => {
                write!(f, "the copy bound of {n} was reached")
            }
            Reason::MemoryLimit(bytes) => {
                write!(f, "the memory limit of {} was reached", Bytes(*bytes))
            }
            Reason::IndexLimit => f.write_str("the search outgrew what its indices address"),
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
    /// The most stable sequents the memo held at once; of two searches
    /// that ran together, the two memos' together, and of two that took
    /// turns from their start, the most of one turn; of the Horn engine,
    /// the markings it kept.
    pub memo_entries: usize,
    /// The context splits examined for `⊗` and Mix, most of them rejected by
    /// the counts.
    pub splits: u64,
    /// The axiom links the net engine tried: each was made, and taken back
    /// again unless it is part of the net found.
    pub links: u64,
    /// The exact acyclicity tests the net engine ran.
    pub tests: u64,
    /// The largest copy bound a level of the focused engine's deepening
    /// began under, of two searches the larger: how far a search without
    /// a bound got before it was stopped. Zero without exponentials and
    /// for the other engines.
    pub copies: u32,
}

impl Statistics {
    /// Adds another engine's counters to these, the memo's excepted: they
    /// describe a table, not a run, and a parallel search reads them off
    /// the one table its workers share.
    pub(crate) fn add(&mut self, other: &Statistics) {
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

    /// The listed names are the names written, in order, and each reads
    /// back as its value.
    #[test]
    fn names_are_listed() {
        assert_eq!(Engine::ALL.map(Engine::name), Engine::NAMES);
        assert_eq!(Bias::ALL.map(Bias::name), Bias::NAMES);
        for engine in Engine::ALL {
            assert_eq!(engine.to_string().parse::<Engine>().unwrap(), engine);
        }
        for bias in Bias::ALL {
            assert_eq!(bias.to_string().parse::<Bias>().unwrap(), bias);
        }
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
        let mut goal = forest.roots().to_vec();
        goal.reverse();
        assert_ne!(goal, forest.roots());
        let classical = Mode::CLASSICAL;
        let outcome = prove_goal(
            &forest,
            &goal,
            classical,
            &Options::default(),
            &Limits::default(),
            |_| false,
        )
        .unwrap();
        assert_eq!(outcome.engine, Engine::Net);
        assert!(outcome.verdict.proof().is_some());
        let net = Options::default().with_engine(Some(Engine::Net));
        let outcome = prove_goal(&forest, &goal, classical, &net, &Limits::default(), |_| {
            false
        })
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
                matches!(error, Error::NetFragment { .. }),
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
        assert!(matches!(error, Error::NetMode { .. }));
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
        assert!(matches!(error, Error::EngineMode { .. }));
        let outcome = prove(&sequent("a, a -o b |- b"), i, &two_sided).unwrap();
        assert_eq!(outcome.engine, Engine::TwoSided);
        assert!(outcome.verdict.proof().is_some());
        let additive = Options::default().with_engine(Some(Engine::Additive));
        let error = prove(&sequent("a & b, c |- a"), i, &additive).unwrap_err();
        assert!(matches!(error, Error::NotAdditive { .. }));
        assert_eq!(
            error.to_string(),
            "the additive engine decides a sequent of two additive-only formulas, not 3 formulas of ALL"
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
        use crate::occurrences::OccId;
        let o = |ids: &[u32]| ids.iter().map(|&i| OccId::new(i)).collect::<Vec<_>>();
        // 0: ~a, 1: (a ⊗ ~b) ⊗ ?c, 2: a ⊗ ~b, 3: a, 4: ~b, 5: ?c, 6: c, 7: b.
        let s = sequent("|- ~a, (a * ~b) * ?c, b");
        let forest = Forest::new(&s).unwrap();
        let options = Options::default();
        let goal = o(&[0, 3]);
        let outcome = prove_goal(
            &forest,
            &goal,
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
            &forest,
            &o(&[0, 2, 7]),
            Mode::CLASSICAL,
            &options,
            &Limits::default(),
            |_| false,
        )
        .unwrap();
        assert!(outcome.verdict.proof().is_some());
        assert_eq!(outcome.fragment, Fragment::MLL);
        let outcome = prove_goal(
            &forest,
            &o(&[4, 5]),
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
            &forest,
            &goal,
            Mode::CLASSICAL,
            &net,
            &Limits::default(),
            |_| false,
        )
        .unwrap_err();
        assert!(matches!(error, Error::NetGoal), "{error}");
        let error = prove_goal(
            &forest,
            &o(&[9]),
            Mode::CLASSICAL,
            &options,
            &Limits::default(),
            |_| false,
        )
        .unwrap_err();
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
            &forest,
            &o(&[1, 4]),
            Mode::CLASSICAL,
            &options,
            &Limits::default(),
            |_| false,
        )
        .unwrap();
        assert_eq!(outcome.engine, Engine::Additive);
        assert!(matches!(outcome.verdict, Verdict::Unprovable(_)));
        let outcome = prove_goal(
            &forest,
            &o(&[5, 2]),
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
            &forest,
            &o(&[0, 2]),
            i,
            &options,
            &Limits::default(),
            |_| false,
        )
        .unwrap();
        assert_eq!(outcome.engine, Engine::TwoSided);
        assert!(outcome.verdict.proof().is_some());
        for (goal, outputs) in [(o(&[0]), 0), (o(&[2, 4]), 2)] {
            let error =
                prove_goal(&forest, &goal, i, &options, &Limits::default(), |_| false).unwrap_err();
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
        let goal = [OccId::new(6), OccId::new(5)];
        let mode = Mode::CLASSICAL;
        let outcome = prove_goal(
            &forest,
            &goal,
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
