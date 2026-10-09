// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

/// Errors printed with formulas for their occurrence ids.
mod describe;
/// The error type for text that is not a sequent.
#[cfg(feature = "parse")]
mod parse;

pub(crate) use describe::Subject;
pub use describe::{Described, Owner};
#[cfg(feature = "parse")]
pub use parse::ParseError;

use crate::fragment::{Fragment, Mode};
use crate::limits::{Refusal, Space};
use crate::nets::NetError;
use crate::occurrences::ShapeError;
use crate::proofs::CheckError;
use crate::search::Engine;
use thiserror::Error;

/// Everything that can go wrong in a call of this crate, as one family:
/// [`kind`](Self::kind) says what sort of failure it is, so that a refusal
/// is never read as a fault, and [`code`](Self::code) is the stable reason
/// a program branches on. The specific types a caller may match on
/// ([`CheckError`], [`NetError`], [`ShapeError`], the session's
/// `StepError`, the parser's `ParseError`, Rocq's `Unsupported`) are
/// variants of it, converted without loss.
///
/// # JSON
///
/// With the feature `serialize` an error is written (never read) as
/// `{"code": …, "kind": …, "message": …, "setting": …, "details": …}`:
/// `code` is one of [`CODES`](Self::CODES), stable from the first release,
/// which a client branches on, accepting a code it does not know by its
/// `kind`; `kind` is the [`ErrorKind`] in snake case; `message` the
/// English text, with formulas when written through
/// [`describe`](Self::describe); `setting`, present only where a bound
/// refused the call, the settings key that lifts it
/// ([`setting`](Self::setting)); and `details` the variant's fields, or
/// the wrapped error's (a `ParseError` its span in bytes and in UTF-16
/// code units, its line and character and what was expected; a
/// [`CheckError`] the node, its rule as a proof writes it, the fault and
/// the premises' sequents; a [`Refusal`] its `kind` and fields).
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum Error {
    /// The input is not a sequent: where and why.
    #[cfg(feature = "parse")]
    #[error("cannot parse the sequent: {0}")]
    Parse(Box<ParseError>),
    /// The input is not an LLTP problem, for the reason given.
    #[cfg(feature = "parse")]
    #[error("not an LLTP problem: {message}")]
    Lltp {
        /// What is wrong with it.
        message: String,
    },
    /// The input is not a coverability problem in Mist's `.spec` format,
    /// for the reason given.
    #[cfg(feature = "parse")]
    #[error("not a .spec problem: {message}")]
    Spec {
        /// What is wrong with it.
        message: String,
    },
    /// The input is not a TPTP problem of propositional logic, for the
    /// reason given.
    #[cfg(feature = "parse")]
    #[error("not a TPTP problem: {message}")]
    Tptp {
        /// What is wrong with it.
        message: String,
    },
    /// An LLTP or TPTP file has more than one conjecture (the name of the
    /// second given), which this crate does not read as one sequent.
    #[cfg(feature = "parse")]
    #[error("the file has a second conjecture, `{second}`, and is read with one only")]
    SeveralConjectures {
        /// The name of the second clause with the role `conjecture`.
        second: String,
    },
    /// A name that names none of the values of its kind, such as a mode
    /// or a fragment read from text.
    #[error("unknown {what} `{name}`: {}", or_list(.known))]
    UnknownName {
        /// What the name was to name: `mode`, `fragment`.
        what: &'static str,
        /// The name read.
        name: Box<str>,
        /// Every name of that kind.
        known: &'static [&'static str],
    },
    /// An index of a value read names nothing: it is not below the length
    /// of its space.
    #[error("{}", out_of_bounds(*.space, *.index, *.len))]
    IndexOutOfBounds {
        /// What the index names.
        space: Space,
        /// The index.
        index: usize,
        /// How many there are.
        len: usize,
    },
    /// A term or a proof node names a part that does not come before it,
    /// where every part must precede what uses it.
    #[error("{}", not_topological(*.space, *.index, *.parent))]
    NotTopological {
        /// What the indices name.
        space: Space,
        /// The part named.
        index: usize,
        /// The one that names it.
        parent: usize,
    },
    /// A sequent says more of its root formulas stand left of `⊢`
    /// (`antecedents`) than it has (`roots`).
    #[error(
        "the sequent has {} left of ⊢ but {} in all",
        counted(*.antecedents, "formula", "formulas"),
        counted(*.roots, "formula", "formulas")
    )]
    Antecedents {
        /// How many root formulas the sequent says stand left of `⊢`.
        antecedents: usize,
        /// How many root formulas it has.
        roots: usize,
    },
    /// The parts of an interactive proof read back do not fit together, as
    /// the reason says.
    #[cfg(feature = "interactive")]
    #[error("not a proof in progress: {reason}")]
    InconsistentSession {
        /// What does not fit.
        reason: &'static str,
    },
    /// A proof does not prove its conclusion, or its check was refused.
    #[error("{}", check_message(.0))]
    Check(CheckError),
    /// A list of links is not a proof structure, or a structure is not a
    /// proof net.
    #[error("not a proof net: {0}")]
    Net(Box<NetError>),
    /// A step of an interactive proof does not apply as asked.
    #[cfg(feature = "interactive")]
    #[error(transparent)]
    Step(#[from] crate::proofs::StepError),
    /// An interactive proof still has this many open goals, so there is no
    /// proof term to make of it yet.
    #[cfg(feature = "interactive")]
    #[error(
        "the proof is not finished: {}",
        if *.count == 1 { "1 goal is open".to_owned() } else { format!("{} goals are open", .count) }
    )]
    OpenGoals {
        /// How many goals are open.
        count: usize,
    },
    /// A proof given to an interactive proof to close a goal is over
    /// another sequent than the session's, whose goals name occurrences of
    /// the session's own.
    #[cfg(feature = "interactive")]
    #[error("the proof is over another sequent than the session's")]
    ForeignProof,
    /// A proof given to close a goal of a session concludes another goal.
    #[cfg(feature = "interactive")]
    #[error("the proof concludes another goal than the one it is to close")]
    GoalMismatch,
    /// A proof of a goal, given where a proof of the sequent is needed: a
    /// proof of a goal is checked against the goal, so it is no wrong
    /// proof, only not one of the sequent.
    #[error("the proof concludes a goal, not the sequent")]
    GoalProof,
    /// The mode is intuitionistic and the sequent has no intuitionistic
    /// reading. The message names occurrences by id;
    /// [`ShapeError::describe`] names them by formula.
    #[error("not an intuitionistic sequent: {0}")]
    NotIntuitionistic(ShapeError),
    /// Mix was asked for in intuitionistic mode, where it has no form: a
    /// premise of a Mix would have no goal.
    #[error("Mix has no intuitionistic form: a premise of a Mix would have no goal")]
    IntuitionisticMix,
    /// An intuitionistic goal has this many formulas on the right of `⊢`,
    /// where a sequent of intuitionistic linear logic has exactly one.
    #[error("an intuitionistic goal has {count} formulas on the right of ⊢ instead of one")]
    GoalOutputs {
        /// How many formulas stand right of `⊢`.
        count: usize,
    },
    /// An intuitionistic or minimal sequent has more than one formula
    /// right of `⊢`.
    #[error("an intuitionistic sequent has at most one formula right of ⊢, not {count}")]
    Succedents {
        /// How many formulas stand right of `⊢`.
        count: usize,
    },
    /// The sequent uses connectives outside the fragment the search options
    /// assert.
    #[error("the sequent lies in {detected}, outside the asserted fragment {asserted}")]
    FragmentMismatch {
        /// The fragment the options assert.
        asserted: Fragment,
        /// The fragment the sequent was detected to lie in.
        detected: Fragment,
    },
    /// The translation does not decide the logic: the affine one decides
    /// classical logic, the others intuitionistic and minimal logic.
    #[error("the {translation} translation does not decide {logic} logic")]
    Translation {
        /// The translation.
        translation: crate::ordinary::Translation,
        /// The logic.
        logic: crate::ordinary::Logic,
    },
    /// No engine handles the fragment in the mode yet.
    #[error("no engine for {fragment} in {mode} mode yet")]
    NoEngine {
        /// The fragment the search was asked for.
        fragment: Fragment,
        /// The mode the search was asked for.
        mode: Mode,
    },
    /// Proof nets exist for unit-free MLL only, and the sequent lies in a
    /// larger fragment.
    #[error("proof nets exist for MLL without units only, not for {fragment}")]
    NetFragment {
        /// The sequent's fragment.
        fragment: Fragment,
    },
    /// Proof nets exist in linear mode only, and the mode is affine.
    #[error("proof nets exist in classical mode only, with or without Mix, not in {mode} mode")]
    NetMode {
        /// The mode asked for.
        mode: Mode,
    },
    /// The net engine, forced by the options, decides the sequent's roots
    /// only, not a goal deeper in the forest.
    #[error("the net engine decides the whole sequent only, not a goal within it")]
    NetGoal,
    /// The engine forced by the options does not search in the mode: the
    /// two-sided engine is for intuitionistic mode, the focus engine for
    /// classical mode.
    #[error("the {engine} engine does not search in {mode} mode")]
    EngineMode {
        /// The engine the options force.
        engine: Engine,
        /// The mode the search was asked for.
        mode: Mode,
    },
    /// The additive engine, forced by the options, decides only sequents of
    /// exactly two additive-only formulas.
    #[error(
        "the additive engine decides a sequent of two additive-only formulas, not {roots} formulas of {fragment}"
    )]
    NotAdditive {
        /// The fragment the sequent was searched in.
        fragment: Fragment,
        /// How many formulas the sequent has.
        roots: usize,
    },
    /// The Horn engine, forced by the options, decides only Horn programs.
    #[error(
        "the horn engine decides a Horn program only: atoms, implications between tensors of \
         atoms, such implications under !, and one goal that is a tensor of atoms"
    )]
    NotHorn,
    /// The output has no form for the derivation; nothing was written.
    #[cfg(feature = "rocq")]
    #[error(transparent)]
    Unsupported(#[from] crate::export::rocq::Unsupported),
    /// A bound, set or default, or the caller's stop refused the call
    /// before it had an answer; lifting the bound may answer.
    #[error("{0}")]
    Refused(Refusal),
    /// The threads of a parallel search could not be started.
    #[cfg(feature = "parallel")]
    #[error("cannot start {threads} search threads: {message}")]
    ThreadPool {
        /// How many threads were asked for.
        threads: usize,
        /// Why they could not be started.
        message: String,
    },
    /// The writer an output went to failed.
    #[error("the writer failed")]
    WriteFailed,
    /// The text given to a renderer is not an SVG document it reads.
    #[cfg(any(feature = "png", feature = "pdf"))]
    #[error("not an SVG document the renderer reads: {message}")]
    NotSvg {
        /// What the reader of SVG said.
        message: String,
    },
    /// The renderer failed on a document it read, or the document does
    /// not conform to the standard asked for.
    #[cfg(any(feature = "png", feature = "pdf"))]
    #[error("the renderer failed: {message}")]
    RenderFailed {
        /// What the renderer said.
        message: String,
    },
    /// A PDF/A document needs the date it was made, and none was given.
    #[cfg(feature = "pdf")]
    #[error("a PDF/A document needs the date it was made: give pdf::Options::date")]
    NoDate,
    /// The checker rejected the proof a search found: a defect of the
    /// engine that found it, and no verdict on the sequent.
    #[error("{}{}", describe::REJECTED, .0)]
    Rejected(Box<CheckError>),
    /// A derivation read back from a linear proof is not one of LK or LJ,
    /// for the reason given: a defect of this crate.
    #[error("the proof read back is not a derivation of {calculus}: {reason}")]
    ReadBack {
        /// LK or LJ.
        calculus: &'static str,
        /// What is wrong, and at which inference.
        reason: String,
    },
}

/// What sort of failure an [`Error`](enum@Error) is.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serialize",
    derive(serde::Serialize),
    serde(rename_all = "snake_case")
)]
pub enum ErrorKind {
    /// The input is not what it says: no sequent, indices that do not fit.
    Malformed,
    /// A verdict: the proof, net, step or session is not what it claims.
    Invalid,
    /// No verdict: outside what this build, engine, kernel or reader does.
    Unsupported,
    /// No verdict: a bound, set or default, refused the call; lifting it
    /// may answer.
    Limit,
    /// No verdict: the caller's stop ended the call.
    Stopped,
    /// The environment failed: threads, the writer, the renderer.
    Failed,
    /// A check of the library's own result failed: a defect to report.
    Defect,
}

impl ErrorKind {
    /// Returns whether the call was refused without a verdict: neither the
    /// input nor a claim about it is at fault.
    pub const fn is_refusal(self) -> bool {
        matches!(self, Self::Unsupported | Self::Limit | Self::Stopped)
    }

    /// Returns the kind's name, as the wire form writes it.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Malformed => "malformed",
            Self::Invalid => "invalid",
            Self::Unsupported => "unsupported",
            Self::Limit => "limit",
            Self::Stopped => "stopped",
            Self::Failed => "failed",
            Self::Defect => "defect",
        }
    }
}

impl std::fmt::Display for ErrorKind {
    /// Writes the kind's name.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

impl Error {
    /// Every code an error of this crate may have.
    pub const CODES: &'static [&'static str] = &[
        "parse",
        "lltp",
        "spec",
        "tptp",
        "several_conjectures",
        "unknown_name",
        "index_out_of_bounds",
        "not_topological",
        "antecedents",
        "inconsistent_session",
        "invalid_proof",
        "invalid_net",
        "no_nets",
        "step",
        "open_goals",
        "foreign_proof",
        "goal_mismatch",
        "goal_proof",
        "not_intuitionistic",
        "intuitionistic_mix",
        "goal_outputs",
        "succedents",
        "fragment_mismatch",
        "translation",
        "no_engine",
        "engine_refused",
        "no_certificate",
        "stopped",
        "memory_limit",
        "too_many_occurrences",
        "output_too_large",
        "work_limit",
        "too_many_pixels",
        "index_limit",
        "thread_pool",
        "write_failed",
        "not_svg",
        "render_failed",
        "no_date",
        "rejected",
        "read_back",
    ];

    /// Returns what sort of failure this is; a wrapped error answers with
    /// its own kind, so a refused check is a refusal.
    pub fn kind(&self) -> ErrorKind {
        use ErrorKind::*;
        match self {
            #[cfg(feature = "parse")]
            Self::Parse(_)
            | Self::Lltp { .. }
            | Self::Spec { .. }
            | Self::Tptp { .. }
            | Self::SeveralConjectures { .. } => Malformed,
            Self::UnknownName { .. } => Malformed,
            Self::IndexOutOfBounds { .. }
            | Self::NotTopological { .. }
            | Self::Antecedents { .. } => Malformed,
            #[cfg(feature = "interactive")]
            Self::InconsistentSession { .. } => Malformed,
            Self::Check(CheckError::Invalid(_)) => Invalid,
            Self::Check(CheckError::Refused(refused)) => refusal_kind(&refused.refusal),
            Self::Net(error) => error.kind(),
            #[cfg(feature = "interactive")]
            Self::Step(_) | Self::OpenGoals { .. } | Self::ForeignProof | Self::GoalMismatch => {
                Invalid
            }
            Self::GoalProof => Unsupported,
            Self::NotIntuitionistic(_)
            | Self::IntuitionisticMix
            | Self::GoalOutputs { .. }
            | Self::Succedents { .. }
            | Self::FragmentMismatch { .. }
            | Self::Translation { .. }
            | Self::NoEngine { .. }
            | Self::NetFragment { .. }
            | Self::NetMode { .. }
            | Self::NetGoal
            | Self::EngineMode { .. }
            | Self::NotAdditive { .. }
            | Self::NotHorn => Unsupported,
            #[cfg(feature = "rocq")]
            Self::Unsupported(_) => Unsupported,
            Self::Refused(refusal) => refusal_kind(refusal),
            #[cfg(feature = "parallel")]
            Self::ThreadPool { .. } => Failed,
            Self::WriteFailed => Failed,
            #[cfg(any(feature = "png", feature = "pdf"))]
            Self::NotSvg { .. } => Malformed,
            #[cfg(any(feature = "png", feature = "pdf"))]
            Self::RenderFailed { .. } => Failed,
            #[cfg(feature = "pdf")]
            Self::NoDate => Malformed,
            Self::Rejected(_) | Self::ReadBack { .. } => Defect,
        }
    }

    /// Returns the stable reason a program branches on, never the English
    /// text: one of [`CODES`](Self::CODES).
    pub fn code(&self) -> &'static str {
        match self {
            #[cfg(feature = "parse")]
            Self::Parse(_) => "parse",
            #[cfg(feature = "parse")]
            Self::Lltp { .. } => "lltp",
            #[cfg(feature = "parse")]
            Self::Spec { .. } => "spec",
            #[cfg(feature = "parse")]
            Self::Tptp { .. } => "tptp",
            #[cfg(feature = "parse")]
            Self::SeveralConjectures { .. } => "several_conjectures",
            Self::UnknownName { .. } => "unknown_name",
            Self::IndexOutOfBounds { .. } => "index_out_of_bounds",
            Self::NotTopological { .. } => "not_topological",
            Self::Antecedents { .. } => "antecedents",
            #[cfg(feature = "interactive")]
            Self::InconsistentSession { .. } => "inconsistent_session",
            Self::Check(CheckError::Invalid(_)) => "invalid_proof",
            Self::Check(CheckError::Refused(refused)) => refused.refusal.code(),
            Self::Net(error) => error.code(),
            #[cfg(feature = "interactive")]
            Self::Step(_) => "step",
            #[cfg(feature = "interactive")]
            Self::OpenGoals { .. } => "open_goals",
            #[cfg(feature = "interactive")]
            Self::ForeignProof => "foreign_proof",
            #[cfg(feature = "interactive")]
            Self::GoalMismatch => "goal_mismatch",
            Self::GoalProof => "goal_proof",
            Self::NotIntuitionistic(_) => "not_intuitionistic",
            Self::IntuitionisticMix => "intuitionistic_mix",
            Self::GoalOutputs { .. } => "goal_outputs",
            Self::Succedents { .. } => "succedents",
            Self::FragmentMismatch { .. } => "fragment_mismatch",
            Self::Translation { .. } => "translation",
            Self::NoEngine { .. } => "no_engine",
            Self::NetFragment { .. }
            | Self::NetMode { .. }
            | Self::NetGoal
            | Self::EngineMode { .. }
            | Self::NotAdditive { .. }
            | Self::NotHorn => "engine_refused",
            #[cfg(feature = "rocq")]
            Self::Unsupported(_) => "no_certificate",
            Self::Refused(refusal) => refusal.code(),
            #[cfg(feature = "parallel")]
            Self::ThreadPool { .. } => "thread_pool",
            Self::WriteFailed => "write_failed",
            #[cfg(any(feature = "png", feature = "pdf"))]
            Self::NotSvg { .. } => "not_svg",
            #[cfg(any(feature = "png", feature = "pdf"))]
            Self::RenderFailed { .. } => "render_failed",
            #[cfg(feature = "pdf")]
            Self::NoDate => "no_date",
            Self::Rejected(_) => "rejected",
            Self::ReadBack { .. } => "read_back",
        }
    }

    /// Returns the settings key whose bound refused the call, which a front
    /// end maps to its flag: `None` for every error a bound did not cause.
    pub fn setting(&self) -> Option<&'static str> {
        match self {
            Self::Refused(refusal) => refusal.setting(),
            Self::Check(CheckError::Refused(refused)) => refused.refusal.setting(),
            Self::Net(error) => match error.as_ref() {
                NetError::Refused { refusal } => refusal.setting(),
                _ => None,
            },
            _ => None,
        }
    }

    /// Returns whether the call was refused without a verdict, as
    /// [`ErrorKind::is_refusal`] says of the error's kind.
    pub fn is_refusal(&self) -> bool {
        self.kind().is_refusal()
    }
}

/// The kind of an error that a refusal makes.
pub(crate) const fn refusal_kind(refusal: &Refusal) -> ErrorKind {
    match refusal {
        Refusal::Stopped { .. } => ErrorKind::Stopped,
        _ => ErrorKind::Limit,
    }
}

/// Lists names as prose: `a, b or c`.
fn or_list(names: &[&str]) -> String {
    match names.split_last() {
        None => String::new(),
        Some((last, [])) => (*last).to_owned(),
        Some((last, rest)) => format!("{} or {last}", rest.join(", ")),
    }
}

/// The message of an index that names nothing.
fn out_of_bounds(space: Space, index: usize, len: usize) -> String {
    match space {
        Space::Atom => format!(
            "a term refers to atom {index}, but the atom list has {}",
            counted(len, "name", "names")
        ),
        Space::Term => format!(
            "a root formula is term {index}, but the arena has {}",
            counted(len, "term", "terms")
        ),
        Space::Node => format!(
            "a proof refers to node {index}, but it has {}",
            counted(len, "node", "nodes")
        ),
        Space::Occurrence => format!(
            "a proof node refers to occurrence {index}, but the sequent has {}",
            counted(len, "occurrence", "occurrences")
        ),
        _ => format!(
            "an index names {index}, but there {} {}",
            if len == 1 { "is" } else { "are" },
            counted(len, space.singular(), space.plural())
        ),
    }
}

/// Writes a count with the noun that agrees with it: `1 goal`, `2 goals`.
pub(crate) fn counted<N>(count: N, one: &str, many: &str) -> String
where
    N: Copy + std::fmt::Display + TryInto<u64>,
{
    let noun = if count.try_into().ok() == Some(1) {
        one
    } else {
        many
    };
    format!("{count} {noun}")
}

/// The message of a part that names a part after it.
fn not_topological(space: Space, index: usize, parent: usize) -> String {
    match space {
        Space::Node => format!(
            "proof node {parent} has premise {index}, but a premise must come before the nodes \
             that use it"
        ),
        _ => format!(
            "term {parent} refers to term {index}, but a subterm must come before the terms that \
             use it"
        ),
    }
}

/// The message of a checker's error: a fault of the proof says so, a
/// refusal is its own.
fn check_message(error: &CheckError) -> String {
    match error {
        CheckError::Invalid(_) => format!("{INVALID_PROOF}{error}"),
        CheckError::Refused(_) => error.to_string(),
    }
}

impl From<CheckError> for Error {
    /// Wraps the checker's error, a fault and a refusal alike.
    fn from(error: CheckError) -> Self {
        Self::Check(error)
    }
}

impl From<NetError> for Error {
    /// Wraps the structure's error.
    fn from(error: NetError) -> Self {
        Self::Net(Box::new(error))
    }
}

impl From<Refusal> for Error {
    /// Wraps the refusal.
    fn from(refusal: Refusal) -> Self {
        Self::Refused(refusal)
    }
}

impl From<std::fmt::Error> for Error {
    /// The writer failed.
    fn from(_: std::fmt::Error) -> Self {
        Self::WriteFailed
    }
}

#[cfg(feature = "parse")]
impl From<ParseError> for Error {
    /// Wraps where and why the text is not a sequent.
    fn from(error: ParseError) -> Self {
        Self::Parse(Box::new(error))
    }
}

/// What [`Error::Check`] says before the checker's fault.
const INVALID_PROOF: &str = "invalid proof: ";

/// Errors stay small enough to pass by value in a `Result`.
const _: () = assert!(size_of::<Error>() <= 64);

/// Errors cross threads and outlive the call that made them.
const _: fn() = || {
    /// Compiles only for a type that is `Send`, `Sync` and `'static`.
    const fn shared<T: Send + Sync + 'static>() {}
    shared::<Error>();
};
