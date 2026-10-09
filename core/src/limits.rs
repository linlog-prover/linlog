// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The resources a call may use, what a long call tells its stop, and why
//! a call was refused without an answer.
//!
//! Every long call of the crate takes a [`Limits`] and a stop, a closure
//! `FnMut(Progress) -> bool` that the call asks at a bounded interval of
//! work and that ends the call when it returns `true`. The crate reads no
//! clock: a front end with a deadline reads its own clock in the stop, as
//! often as the work done ([`Progress::work`]) says it should. A call
//! that a bound or the stop ends answers with a [`Refusal`], which is
//! never a verdict on its input.

use std::fmt::{self, Display, Formatter};

#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};

/// The resources a call may use, which the library enforces. `None` in an
/// optional field means no bound; the defaults are the named constants.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serialize",
    derive(Serialize, Deserialize),
    serde(default, deny_unknown_fields)
)]
pub struct Limits {
    /// The most bytes a call holds at once beyond its input: a search's
    /// growing structures, a check's pass, a derivation's making, a
    /// render's estimate. The sequent, its forest (bounded by
    /// [`occurrences`](Self::occurrences)), the proof returned, the
    /// threads' stacks and the allocator's own overhead are not counted.
    pub memory_bytes: Option<u64>,
    /// The most subformula occurrences a sequent may unfold to, on every
    /// reader and wherever a forest is built.
    pub occurrences: Option<u64>,
    /// The most bytes a derivation, or the drawing of a net or a sequent,
    /// may be estimated at; past it nothing is made.
    pub derivation_bytes: Option<u64>,
    /// The most units of work a search may do (each engine says what its
    /// unit is).
    pub work: Option<u64>,
    /// The deepest recursion on any one stack; every thread the library
    /// starts is sized for it ([`stack_bytes`](Self::stack_bytes)).
    pub recursion_depth: u32,
}

impl Limits {
    /// The default memory bound: one gibibyte.
    pub const DEFAULT_MEMORY_BYTES: u64 = 1 << 30;
    /// The default bound on occurrences: fifty million, above the largest
    /// problem of the LLTP library (27.8 million); a forest takes about
    /// 25 bytes an occurrence.
    pub const DEFAULT_OCCURRENCES: u64 = 50_000_000;
    /// The default bound on a derivation's estimated size: 64 mebibytes.
    pub const DEFAULT_DERIVATION_BYTES: u64 = 64 << 20;
    /// The default recursion depth: 2048 levels, which fit a main
    /// thread's stack of 8 MiB in an optimized build.
    pub const DEFAULT_RECURSION_DEPTH: u32 = 2048;

    /// No bound at all but the default recursion depth, which every stack
    /// needs.
    pub const UNBOUNDED: Self = Self {
        memory_bytes: None,
        occurrences: None,
        derivation_bytes: None,
        work: None,
        recursion_depth: Self::DEFAULT_RECURSION_DEPTH,
    };

    /// The stack one level of the search's recursion may take: twice the
    /// most it was measured to take, on a chain of tensors whose splits
    /// are searched and recursed into (5.6 KiB unoptimized, 1.1 KiB
    /// optimized, on x86-64).
    const PER_LEVEL: usize = if cfg!(debug_assertions) { 12288 } else { 2304 };

    /// What a thread needs below and beside the recursion: the frames of
    /// its caller, the search's set-up and its proof's checks.
    const RESERVE: usize = 1 << 20;

    /// Returns the stack a thread needs to recurse to
    /// [`recursion_depth`](Self::recursion_depth), at least a main
    /// thread's 8 MiB.
    #[must_use]
    pub const fn stack_bytes(&self) -> usize {
        let needed = (self.recursion_depth as usize)
            .saturating_mul(Self::PER_LEVEL)
            .saturating_add(Self::RESERVE);
        if needed > 8 << 20 { needed } else { 8 << 20 }
    }

    /// Returns the deepest recursion a stack of `bytes` holds, the inverse
    /// of [`stack_bytes`](Self::stack_bytes) above its floor.
    #[must_use]
    pub const fn recursion_depth_for_stack(bytes: usize) -> u32 {
        let levels = bytes.saturating_sub(Self::RESERVE) / Self::PER_LEVEL;
        if levels > u32::MAX as usize {
            u32::MAX
        } else {
            levels as u32
        }
    }

    /// Returns the limits with another memory bound.
    #[must_use]
    pub const fn with_memory_bytes(mut self, memory_bytes: Option<u64>) -> Self {
        self.memory_bytes = memory_bytes;
        self
    }

    /// Returns the limits with another bound on occurrences.
    #[must_use]
    pub const fn with_occurrences(mut self, occurrences: Option<u64>) -> Self {
        self.occurrences = occurrences;
        self
    }

    /// Returns the limits with another bound on a derivation's size.
    #[must_use]
    pub const fn with_derivation_bytes(mut self, derivation_bytes: Option<u64>) -> Self {
        self.derivation_bytes = derivation_bytes;
        self
    }

    /// Returns the limits with another bound on a search's work.
    #[must_use]
    pub const fn with_work(mut self, work: Option<u64>) -> Self {
        self.work = work;
        self
    }

    /// Returns the limits with another recursion depth.
    #[must_use]
    pub const fn with_recursion_depth(mut self, recursion_depth: u32) -> Self {
        self.recursion_depth = recursion_depth;
        self
    }
}

impl Default for Limits {
    /// Returns the default bounds, each a named constant.
    fn default() -> Self {
        Self {
            memory_bytes: Some(Self::DEFAULT_MEMORY_BYTES),
            occurrences: Some(Self::DEFAULT_OCCURRENCES),
            derivation_bytes: Some(Self::DEFAULT_DERIVATION_BYTES),
            work: None,
            recursion_depth: Self::DEFAULT_RECURSION_DEPTH,
        }
    }
}

/// What a long call tells its stop each time it asks it.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
pub struct Progress {
    /// The units of work done since the previous poll, the unit of
    /// [`Limits::work`].
    pub work: u64,
    /// The units of work done since the call began.
    pub done: u64,
    /// The bytes the call's account holds now.
    pub held_bytes: u64,
    /// What the call is doing.
    pub phase: Phase,
    /// The item of a call that runs several: a goal of a session's
    /// `close_all`, a problem of a batch.
    pub item: u32,
}

impl Progress {
    /// Returns the progress of a call in `phase` that has done `done`
    /// units of work, `work` of them since the previous poll.
    pub(crate) const fn new(phase: Phase, work: u64, done: u64) -> Self {
        Self {
            work,
            done,
            held_bytes: 0,
            phase,
            item: 0,
        }
    }
}

/// What a long call is doing when it asks its stop or is refused.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serialize",
    derive(Serialize),
    serde(rename_all = "snake_case")
)]
pub enum Phase {
    /// Reading an input.
    #[default]
    Read,
    /// Searching for a proof.
    Search,
    /// Refuting a sequent after a search.
    Refute,
    /// Checking a proof.
    Check,
    /// Making a derivation.
    View,
    /// Writing a derivation in a format.
    Write,
    /// Rendering a drawing.
    Render,
    /// Building or judging a proof net.
    Net,
    /// Reading a linear proof back as one of ordinary logic.
    ReadBack,
}

impl Phase {
    /// Returns the phase as a lowercase noun phrase, as messages use it.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Read => "reading",
            Self::Search => "search",
            Self::Refute => "refutation",
            Self::Check => "check",
            Self::View => "derivation",
            Self::Write => "writing",
            Self::Render => "render",
            Self::Net => "net",
            Self::ReadBack => "read-back",
        }
    }
}

/// What a count of [`Refusal::Index`] counts: the spaces whose indices
/// have a fixed width.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serialize",
    derive(Serialize),
    serde(rename_all = "snake_case")
)]
pub enum Space {
    /// The atoms of a sequent.
    Atom,
    /// The terms of a sequent's arena.
    Term,
    /// The nodes of a proof.
    Node,
    /// The occurrences of a forest.
    Occurrence,
    /// The members of a sequent of a proof, a derivation or a session.
    Member,
    /// The vertices of a proof structure.
    Vertex,
    /// The inferences of a derivation.
    Inference,
}

impl Space {
    /// Returns the plural noun that counts members of the space.
    pub const fn plural(self) -> &'static str {
        match self {
            Self::Atom => "atoms",
            Self::Term => "terms",
            Self::Node => "nodes",
            Self::Occurrence => "occurrences",
            Self::Member => "members",
            Self::Vertex => "vertices",
            Self::Inference => "inferences",
        }
    }
}

/// Why a call was given up without an answer: a bound the caller set, or
/// a default, refused it, or the caller's stop ended it. A refusal says
/// nothing of the input; lifting the bound may answer.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(
    feature = "serialize",
    derive(Serialize),
    serde(tag = "kind", rename_all = "snake_case")
)]
pub enum Refusal {
    /// The caller's stop ended the call.
    Stopped {
        /// What the call was doing.
        phase: Phase,
    },
    /// The call would have held more memory than its bound.
    Memory {
        /// What the call was doing.
        phase: Phase,
        /// The bound, [`Limits::memory_bytes`].
        limit_bytes: u64,
        /// What the call would have needed, where it was estimated before
        /// anything was made.
        needed_bytes: Option<u64>,
    },
    /// A sequent unfolds to more occurrences than the bound: the count, or
    /// a number of them at which the count stopped.
    Occurrences {
        /// How many occurrences, or a number past the bound.
        occurrences: u64,
        /// The bound, [`Limits::occurrences`].
        limit: u64,
    },
    /// An output is estimated past its bound; nothing was made.
    Output {
        /// What was to be made: a derivation, a drawing.
        what: &'static str,
        /// The estimate.
        estimate_bytes: u64,
        /// The bound, [`Limits::derivation_bytes`].
        limit_bytes: u64,
        /// The least a smaller form of the output would take, where one
        /// was tried: a compact derivation's lower bound.
        least_bytes: Option<u64>,
    },
    /// A search did the most work its bound allows.
    Work {
        /// The bound, [`Limits::work`].
        limit: u64,
    },
    /// An image would have more pixels than its options allow.
    Pixels {
        /// The image's pixels.
        pixels: u64,
        /// The bound of the image's options.
        limit: u64,
    },
    /// A count passes what a representation's indices can name, whatever
    /// a bound says.
    Index {
        /// What is counted.
        what: Space,
        /// The count, or a number past the most.
        count: u64,
        /// The most the representation holds.
        most: u64,
    },
}

impl Refusal {
    /// Returns the stable code a program branches on.
    pub const fn code(&self) -> &'static str {
        match self {
            Self::Stopped { .. } => "stopped",
            Self::Memory { .. } => "memory_limit",
            Self::Occurrences { .. } => "too_many_occurrences",
            Self::Output { .. } => "output_too_large",
            Self::Work { .. } => "work_limit",
            Self::Pixels { .. } => "too_many_pixels",
            Self::Index { .. } => "index_limit",
        }
    }

    /// Returns the settings key whose bound refused the call; `None` for
    /// the stop and for a representation's width, which no setting lifts.
    pub const fn setting(&self) -> Option<&'static str> {
        match self {
            Self::Memory { .. } => Some("limits.memory_bytes"),
            Self::Occurrences { .. } => Some("limits.occurrences"),
            Self::Output { .. } => Some("limits.derivation_bytes"),
            Self::Work { .. } => Some("limits.work"),
            Self::Pixels { .. } => Some("styles.png.pixels"),
            Self::Stopped { .. } | Self::Index { .. } => None,
        }
    }

    /// Returns whether the caller's stop ended the call, rather than a
    /// bound.
    pub const fn is_stop(&self) -> bool {
        matches!(self, Self::Stopped { .. })
    }
}

impl Display for Refusal {
    /// Writes the reason in a sentence without a full stop.
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Stopped { phase } => write!(f, "the {} was stopped", phase.name()),
            Self::Memory {
                phase,
                limit_bytes,
                needed_bytes: Some(needed),
            } => write!(
                f,
                "the {} would take {needed} bytes, more than the memory limit of {limit_bytes} bytes",
                phase.name()
            ),
            Self::Memory {
                phase,
                limit_bytes,
                needed_bytes: None,
            } => write!(
                f,
                "the {} takes more than the memory limit of {limit_bytes} bytes",
                phase.name()
            ),
            Self::Occurrences { occurrences, limit } => write!(
                f,
                "the sequent unfolds to at least {occurrences} subformula occurrences, more \
                 than the limit of {limit}"
            ),
            Self::Output {
                what,
                estimate_bytes,
                limit_bytes,
                least_bytes,
            } => {
                write!(
                    f,
                    "the {what} is estimated at {estimate_bytes} bytes, more than the limit of \
                     {limit_bytes}"
                )?;
                match least_bytes {
                    Some(least) => write!(f, ", and a compact one at {least} bytes at least"),
                    None => Ok(()),
                }
            }
            Self::Work { limit } => {
                write!(f, "the search did the most work allowed, {limit} units")
            }
            Self::Pixels { pixels, limit } => write!(
                f,
                "the image would have {pixels} pixels, more than the limit of {limit}"
            ),
            Self::Index { what, count, most } => write!(
                f,
                "{count} {} are more than this representation can index ({most})",
                what.plural()
            ),
        }
    }
}

/// Calls `stop` with the progress of a call in `phase` that is not
/// counted in work, as the polls do whose work is not counted yet.
pub(crate) fn poll(stop: &mut impl FnMut(Progress) -> bool, phase: Phase) -> bool {
    stop(Progress::new(phase, 0, 0))
}
