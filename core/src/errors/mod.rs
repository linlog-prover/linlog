// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

/// Errors printed with formulas for their occurrence ids.
pub(crate) mod describe;
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
    ///
    /// Needs the cargo feature `parse`.
    #[cfg(feature = "parse")]
    #[error("cannot parse the sequent: {0}")]
    Parse(Box<ParseError>),
    /// The input is not an LLTP problem, for the reason given.
    ///
    /// Needs the cargo feature `parse`.
    #[cfg(feature = "parse")]
    #[error("not an LLTP problem: {message}")]
    #[non_exhaustive]
    Lltp {
        /// What is wrong with it.
        message: String,
    },
    /// The input is not a coverability problem in Mist's `.spec` format,
    /// for the reason given.
    ///
    /// Needs the cargo feature `parse`.
    #[cfg(feature = "parse")]
    #[error("not a .spec problem: {message}")]
    #[non_exhaustive]
    Spec {
        /// What is wrong with it.
        message: String,
    },
    /// The input is not a TPTP problem of propositional logic, for the
    /// reason given.
    ///
    /// Needs the cargo feature `parse`.
    #[cfg(feature = "parse")]
    #[error("not a TPTP problem: {message}")]
    #[non_exhaustive]
    Tptp {
        /// What is wrong with it.
        message: String,
    },
    /// An LLTP or TPTP file has more than one conjecture (the name of the
    /// second given), which this crate does not read as one sequent.
    ///
    /// Needs the cargo feature `parse`.
    #[cfg(feature = "parse")]
    #[error("the file has a second conjecture, `{second}`, and is read with one only")]
    #[non_exhaustive]
    SeveralConjectures {
        /// The name of the second clause with the role `conjecture`.
        second: String,
    },
    /// A name that names none of the values of its kind, such as a mode
    /// or a fragment read from text.
    #[error("unknown {what} `{name}`: {}", or_list(.known))]
    #[non_exhaustive]
    UnknownName {
        /// What the name was to name: `mode`, `fragment`.
        what: &'static str,
        /// The name read.
        name: Box<str>,
        /// Every name of that kind.
        known: &'static [&'static str],
    },
    /// A generated family has no instance of the size asked for.
    ///
    /// Needs the cargo feature `parse`.
    #[cfg(feature = "parse")]
    #[error(
        "the family {family} has no instance of size {size}: its sizes are {}at least {least}",
        if *.powers_of_two { "the powers of two " } else { "" }
    )]
    #[non_exhaustive]
    FamilySize {
        /// The family's name.
        family: &'static str,
        /// The size asked for.
        size: u32,
        /// The least size it has.
        least: u32,
        /// Whether its sizes are the powers of two.
        powers_of_two: bool,
    },
    /// An option's value is not one the output can take: a Rocq lemma
    /// that is no identifier.
    #[error("{key}: {message}")]
    #[non_exhaustive]
    InvalidOption {
        /// The option, as the settings name it: `rocq.lemma`.
        key: &'static str,
        /// What is wrong with the value.
        message: String,
    },
    /// A document is not of the form it was read as: not JSON, a key
    /// missing or of another type.
    #[error("not a {form} of linlog's wire form: {message}")]
    #[non_exhaustive]
    Json {
        /// The form read: `sequent`, `proof`, …
        form: &'static str,
        /// What was wrong, as the reader says it.
        message: String,
    },
    /// A document is of a wire level above the one this build reads.
    #[error("the {form} is of wire level {found}, and this build reads levels up to {supported}")]
    #[non_exhaustive]
    Version {
        /// The form read.
        form: &'static str,
        /// The document's `version`.
        found: u64,
        /// The highest level this build reads, [`wire::LEVEL`](crate::wire::LEVEL).
        supported: u32,
    },
    /// An index of a value read names nothing: it is not below the length
    /// of its space.
    #[error("{}", out_of_bounds(*.space, *.index, *.len))]
    #[non_exhaustive]
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
    #[non_exhaustive]
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
    #[non_exhaustive]
    Antecedents {
        /// How many root formulas the sequent says stand left of `⊢`.
        antecedents: usize,
        /// How many root formulas it has.
        roots: usize,
    },
    /// An atom name read from a file or given to a constructor is no
    /// identifier of the text syntax, or is one of its keywords or a word
    /// reserved for a later version of it, so that the sequent written as
    /// text would not read back as itself.
    #[error("the atom name {name:?} {}", crate::sequents::name::fault(name))]
    #[non_exhaustive]
    AtomName {
        /// The name.
        name: String,
    },
    /// The parts of an interactive proof read back do not fit together, as
    /// the reason says.
    ///
    /// Needs the cargo feature `interactive`.
    #[cfg(feature = "interactive")]
    #[error("not a proof in progress: {reason}")]
    #[non_exhaustive]
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
    ///
    /// Needs the cargo feature `interactive`.
    #[cfg(feature = "interactive")]
    #[error(transparent)]
    Step(#[from] crate::proofs::StepError),
    /// An interactive proof still has this many open goals, so there is no
    /// proof term to make of it yet.
    ///
    /// Needs the cargo feature `interactive`.
    #[cfg(feature = "interactive")]
    #[error(
        "the proof is not finished: {}",
        if *.count == 1 { "1 goal is open".to_owned() } else { format!("{} goals are open", .count) }
    )]
    #[non_exhaustive]
    OpenGoals {
        /// How many goals are open.
        count: usize,
    },
    /// A proof given to an interactive proof to close a goal is over
    /// another sequent than the session's, whose goals name occurrences of
    /// the session's own.
    ///
    /// Needs the cargo feature `interactive`.
    #[cfg(feature = "interactive")]
    #[error("the proof is over another sequent than the session's")]
    ForeignProof,
    /// A proof given to close a goal of a session concludes another goal.
    ///
    /// Needs the cargo feature `interactive`.
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
    #[non_exhaustive]
    GoalOutputs {
        /// How many formulas stand right of `⊢`.
        count: usize,
    },
    /// An intuitionistic or minimal sequent has more than one formula
    /// right of `⊢`.
    #[error("an intuitionistic sequent has at most one formula right of ⊢, not {count}")]
    #[non_exhaustive]
    Succedents {
        /// How many formulas stand right of `⊢`.
        count: usize,
    },
    /// The sequent uses connectives outside the fragment the search options
    /// assert.
    #[error("the sequent lies in {detected}, outside the asserted fragment {asserted}")]
    #[non_exhaustive]
    FragmentMismatch {
        /// The fragment the options assert.
        asserted: Fragment,
        /// The fragment the sequent was detected to lie in.
        detected: Fragment,
    },
    /// The translation does not decide the logic: the affine one decides
    /// classical logic, the others intuitionistic and minimal logic.
    #[error("the {translation} translation does not decide {logic} logic")]
    #[non_exhaustive]
    Translation {
        /// The translation.
        translation: crate::ordinary::Translation,
        /// The logic.
        logic: crate::ordinary::Logic,
    },
    /// No engine handles the fragment in the mode yet.
    #[error("no engine for {fragment} in {mode} mode yet")]
    #[non_exhaustive]
    NoEngine {
        /// The fragment the search was asked for.
        fragment: Fragment,
        /// The mode the search was asked for.
        mode: Mode,
    },
    /// Proof nets exist for unit-free MLL only, and the sequent lies in a
    /// larger fragment.
    #[error("proof nets exist for MLL without units only, not for {fragment}")]
    #[non_exhaustive]
    NetFragment {
        /// The sequent's fragment.
        fragment: Fragment,
    },
    /// Proof nets exist in linear mode only, and the mode is affine.
    #[error("proof nets exist in linear mode only, with or without Mix, not in {mode} mode")]
    #[non_exhaustive]
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
    #[non_exhaustive]
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
    #[non_exhaustive]
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
    ///
    /// Needs the cargo feature `rocq`.
    #[cfg(feature = "rocq")]
    #[error(transparent)]
    Unsupported(#[from] crate::export::rocq::Unsupported),
    /// A bound, set or default, or the caller's stop refused the call
    /// before it had an answer; lifting the bound may answer.
    #[error("{0}")]
    Refused(Refusal),
    /// The threads of a parallel search could not be started.
    ///
    /// Needs the cargo feature `parallel`.
    #[cfg(feature = "parallel")]
    #[error("cannot start {threads} search threads: {message}")]
    #[non_exhaustive]
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
    ///
    /// Needs one of the cargo features `png` or `pdf`.
    #[cfg(any(feature = "png", feature = "pdf"))]
    #[error("not an SVG document the renderer reads: {message}")]
    #[non_exhaustive]
    NotSvg {
        /// What the reader of SVG said.
        message: String,
    },
    /// The renderer failed on a document it read, or the document does
    /// not conform to the standard asked for.
    ///
    /// Needs one of the cargo features `png` or `pdf`.
    #[cfg(any(feature = "png", feature = "pdf"))]
    #[error("the renderer failed: {message}")]
    #[non_exhaustive]
    RenderFailed {
        /// What the renderer said.
        message: String,
    },
    /// A PDF/A document needs the date it was made, and none was given.
    ///
    /// Needs the cargo feature `pdf`.
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
    #[non_exhaustive]
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
        "family_size",
        "invalid_option",
        "json",
        "unsupported_version",
        "index_out_of_bounds",
        "not_topological",
        "antecedents",
        "atom_name",
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
            Self::UnknownName { .. } | Self::InvalidOption { .. } | Self::Json { .. } => Malformed,
            #[cfg(feature = "parse")]
            Self::FamilySize { .. } => Malformed,
            Self::Version { .. } => Unsupported,
            Self::IndexOutOfBounds { .. }
            | Self::NotTopological { .. }
            | Self::Antecedents { .. }
            | Self::AtomName { .. } => Malformed,
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
            #[cfg(feature = "parse")]
            Self::FamilySize { .. } => "family_size",
            Self::InvalidOption { .. } => "invalid_option",
            Self::Json { .. } => "json",
            Self::Version { .. } => "unsupported_version",
            Self::IndexOutOfBounds { .. } => "index_out_of_bounds",
            Self::NotTopological { .. } => "not_topological",
            Self::Antecedents { .. } => "antecedents",
            Self::AtomName { .. } => "atom_name",
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

impl From<ShapeError> for Error {
    /// Wraps why the sequent has no intuitionistic reading.
    fn from(error: ShapeError) -> Self {
        Self::NotIntuitionistic(error)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    /// One error of every variant, and of every code a wrapped type
    /// gives.
    fn samples() -> Vec<Error> {
        use crate::limits::Phase;
        use crate::occurrences::{Member, OccId};
        use crate::proofs::{Fault, Invalid, Node, NodeId, Refused};
        let refusals = [
            Refusal::Stopped {
                phase: Phase::Search,
            },
            Refusal::Memory {
                phase: Phase::Search,
                limit_bytes: 1,
                needed_bytes: None,
            },
            Refusal::Occurrences {
                occurrences: 2,
                limit: 1,
            },
            Refusal::Output {
                what: "derivation",
                estimate_bytes: 2,
                limit_bytes: 1,
                least_bytes: None,
            },
            Refusal::Work { limit: 1 },
            Refusal::Pixels {
                pixels: 2,
                limit: 1,
            },
            Refusal::Index {
                what: Space::Node,
                count: 2,
                most: 1,
            },
        ];
        let invalid = CheckError::Invalid(Box::new(Invalid {
            node: NodeId::new(0),
            rule: Node::One(Member::from(OccId::new(0))),
            premises: Vec::new(),
            fault: Fault::NotDual,
        }));
        let message = String::new;
        let mut samples = vec![
            Error::UnknownName {
                what: "mode",
                name: "x".into(),
                known: &[],
            },
            Error::InvalidOption {
                key: "typst",
                message: message(),
            },
            Error::Json {
                form: "sequent",
                message: message(),
            },
            Error::Version {
                form: "sequent",
                found: 2,
                supported: 1,
            },
            Error::IndexOutOfBounds {
                space: Space::Term,
                index: 1,
                len: 1,
            },
            Error::NotTopological {
                space: Space::Term,
                index: 1,
                parent: 0,
            },
            Error::Antecedents {
                antecedents: 2,
                roots: 1,
            },
            Error::AtomName { name: "par".into() },
            Error::Check(invalid.clone()),
            Error::Check(CheckError::Refused(Refused {
                node: NodeId::new(0),
                refusal: refusals[0].clone(),
            })),
            Error::Net(Box::new(NetError::Empty)),
            Error::Net(Box::new(NetError::Fragment {
                fragment: Fragment::LL,
            })),
            Error::GoalProof,
            Error::NotIntuitionistic(ShapeError::NoGoal),
            Error::IntuitionisticMix,
            Error::GoalOutputs { count: 2 },
            Error::Succedents { count: 2 },
            Error::FragmentMismatch {
                asserted: Fragment::MLL,
                detected: Fragment::LL,
            },
            Error::Translation {
                translation: crate::ordinary::Translation::Affine,
                logic: crate::ordinary::Logic::Intuitionistic,
            },
            Error::NoEngine {
                fragment: Fragment::LL,
                mode: Mode::CLASSICAL,
            },
            Error::NetFragment {
                fragment: Fragment::LL,
            },
            Error::NetMode {
                mode: Mode::CLASSICAL,
            },
            Error::NetGoal,
            Error::EngineMode {
                engine: Engine::Net,
                mode: Mode::CLASSICAL,
            },
            Error::NotAdditive {
                fragment: Fragment::LL,
                roots: 3,
            },
            Error::NotHorn,
            Error::WriteFailed,
            Error::Rejected(Box::new(invalid)),
            Error::ReadBack {
                calculus: "LK",
                reason: message(),
            },
        ];
        samples.extend(refusals.into_iter().map(Error::Refused));
        #[cfg(feature = "parse")]
        samples.extend([
            Error::Parse(Box::new(ParseError::spanning("(", 1..1, &["a formula"]))),
            Error::Lltp { message: message() },
            Error::Spec { message: message() },
            Error::Tptp { message: message() },
            Error::SeveralConjectures { second: message() },
            Error::FamilySize {
                family: "chain",
                size: 0,
                least: 1,
                powers_of_two: false,
            },
        ]);
        #[cfg(feature = "interactive")]
        samples.extend([
            Error::InconsistentSession { reason: "" },
            Error::Step(crate::proofs::StepError::NoGoal {
                goal: crate::proofs::interactive::GoalId::new(0),
            }),
            Error::OpenGoals { count: 1 },
            Error::ForeignProof,
            Error::GoalMismatch,
        ]);
        #[cfg(feature = "rocq")]
        samples.push(Error::Unsupported(crate::export::rocq::Unsupported::Mix));
        #[cfg(feature = "parallel")]
        samples.push(Error::ThreadPool {
            threads: 2,
            message: message(),
        });
        #[cfg(any(feature = "png", feature = "pdf"))]
        samples.extend([
            Error::NotSvg { message: message() },
            Error::RenderFailed { message: message() },
        ]);
        #[cfg(feature = "pdf")]
        samples.push(Error::NoDate);
        samples
    }

    /// Names every variant without a wildcard, so that a variant added
    /// later fails to compile here until it has a sample above.
    fn variant(error: &Error) {
        match error {
            #[cfg(feature = "parse")]
            Error::Parse(_)
            | Error::Lltp { .. }
            | Error::Spec { .. }
            | Error::Tptp { .. }
            | Error::SeveralConjectures { .. }
            | Error::FamilySize { .. } => {}
            #[cfg(feature = "interactive")]
            Error::InconsistentSession { .. }
            | Error::Step(_)
            | Error::OpenGoals { .. }
            | Error::ForeignProof
            | Error::GoalMismatch => {}
            #[cfg(feature = "rocq")]
            Error::Unsupported(_) => {}
            #[cfg(feature = "parallel")]
            Error::ThreadPool { .. } => {}
            #[cfg(any(feature = "png", feature = "pdf"))]
            Error::NotSvg { .. } | Error::RenderFailed { .. } => {}
            #[cfg(feature = "pdf")]
            Error::NoDate => {}
            Error::UnknownName { .. }
            | Error::InvalidOption { .. }
            | Error::Json { .. }
            | Error::Version { .. }
            | Error::IndexOutOfBounds { .. }
            | Error::NotTopological { .. }
            | Error::Antecedents { .. }
            | Error::AtomName { .. }
            | Error::Check(_)
            | Error::Net(_)
            | Error::GoalProof
            | Error::NotIntuitionistic(_)
            | Error::IntuitionisticMix
            | Error::GoalOutputs { .. }
            | Error::Succedents { .. }
            | Error::FragmentMismatch { .. }
            | Error::Translation { .. }
            | Error::NoEngine { .. }
            | Error::NetFragment { .. }
            | Error::NetMode { .. }
            | Error::NetGoal
            | Error::EngineMode { .. }
            | Error::NotAdditive { .. }
            | Error::NotHorn
            | Error::Refused(_)
            | Error::WriteFailed
            | Error::Rejected(_)
            | Error::ReadBack { .. } => {}
        }
    }

    /// `CODES` lists exactly the codes the errors give, once each; a code
    /// of a feature this build lacks is listed without a sample.
    #[test]
    fn the_codes_are_listed() {
        let listed: BTreeSet<&str> = Error::CODES.iter().copied().collect();
        assert_eq!(listed.len(), Error::CODES.len(), "a code listed twice");
        let mut given = BTreeSet::new();
        for error in samples() {
            variant(&error);
            assert!(
                listed.contains(error.code()),
                "{} is not listed",
                error.code()
            );
            given.insert(error.code());
        }
        let parse = ["parse", "lltp", "spec", "tptp", "several_conjectures"];
        let interactive = [
            "inconsistent_session",
            "step",
            "open_goals",
            "foreign_proof",
            "goal_mismatch",
        ];
        let gated = |code: &str| {
            let render = cfg!(any(feature = "png", feature = "pdf"));
            (parse.contains(&code) || code == "family_size") && !cfg!(feature = "parse")
                || interactive.contains(&code) && !cfg!(feature = "interactive")
                || code == "no_certificate" && !cfg!(feature = "rocq")
                || code == "thread_pool" && !cfg!(feature = "parallel")
                || (code == "not_svg" || code == "render_failed") && !render
                || code == "no_date" && !cfg!(feature = "pdf")
        };
        for code in listed.difference(&given) {
            assert!(gated(code), "no error gives {code}");
        }
    }
}
