// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Sequents of linear logic, stored as compact arena DAGs in negation normal
//! form, with parsing, printing and serialization; the fragment a sequent
//! lives in and the mode proof search runs in; the occurrence forest of a
//! sequent, the numbering of its subformula occurrences that proof search,
//! proof checking and proof nets are built on; proofs as terms over those
//! occurrences, with the checker that validates them, the derivation view
//! that renders them, and their serialization; proof search, which
//! decides a sequent with the engine its fragment calls for and returns a
//! checked proof; and [`ordinary`] propositional logic, classical,
//! intuitionistic and minimal, decided through its embeddings into linear
//! logic, with the proof read back as a derivation of LK or LJ.
//!
//! # The common path
//!
//! Parse a sequent, see which fragment it lives in, prove it, print the
//! derivation and serialize the proof:
//!
#![cfg_attr(all(feature = "parse", feature = "serialize"), doc = "```")]
#![cfg_attr(not(all(feature = "parse", feature = "serialize")), doc = "```ignore")]
//! use linlog::{Fragment, Mode, Options, Sequent, Verdict, prove};
//!
//! // Two-sided input becomes one-sided: ⊢ ~A, A ⊗ ~B, B.
//! let sequent: Sequent = "A, A -o B |- B".parse()?;
//! assert_eq!(sequent.fragment(), Fragment::MLL);
//!
//! let outcome = prove(&sequent, Mode::CLASSICAL, &Options::default())?;
//! let Verdict::Proved(proof) = &outcome.verdict else {
//!     panic!("the sequent is provable");
//! };
//! assert_eq!(
//!     proof.derivation()?.to_string(),
//!     "─────── ax   ─────── ax\n\
//!      ⊢ ~A, A      ⊢ ~B, B\n\
//!      ──────────────────── ⊗\n\
//!     \x20 ⊢ ~A, A ⊗ ~B, B"
//! );
//!
//! let json = serde_json::to_string(&proof)?;
//! let back: linlog::Proof = serde_json::from_str(&json)?;
//! back.check(Mode::CLASSICAL)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! A [`Mode`] is the rules a logic allows, [`Options`] the knobs of a
//! search with their defaults, [`Limits`] the resources a call may use,
//! and [`prove_within`] takes a stop condition for a time limit or an
//! interruption, since this crate has no clock:
//!
#![cfg_attr(feature = "parse", doc = "```")]
#![cfg_attr(not(feature = "parse"), doc = "```ignore")]
//! use linlog::{Fragment, Limits, Mode, Options, Verdict, prove_within};
//! use std::time::{Duration, Instant};
//!
//! let mode = Mode::CLASSICAL.with_mix();
//! let options = Options::default().with_fragment(Some(Fragment::MALL));
//! let limits = Limits::default().with_memory_bytes(Some(64 << 20));
//! let deadline = Instant::now() + Duration::from_secs(10);
//! let sequent = "|- A par B, ~A, ~B".parse()?;
//! let outcome = prove_within(&sequent, mode, &options, &limits, |_| Instant::now() > deadline)?;
//! assert!(matches!(outcome.verdict, Verdict::Proved(_)));
//! assert_eq!(outcome.fragment, Fragment::MALL);
//! # Ok::<(), linlog::Error>(())
//! ```
//!
//! **The command's verdicts differ from `Options::default()`'s on purpose.**
//! A library call without a stop must end, so the default options bound
//! the copies of a `?` formula per branch at three; the `linlog` command
//! has a clock, so it deepens without a bound until its time limit of two
//! seconds, and after a tenth of a second it adds a pool of every other
//! thread beside the first. [`Settings::default()`](Settings) is that
//! behaviour as data: to decide what the command decides, search with its
//! `search` options (no copy bound) and stop at its `clock`'s time limit,
//! on one thread for the command's `--deterministic` answer.
//!
//! Errors are one family, [`Error`]: its [`kind`](Error::kind) says what
//! sort of failure it is, so that a call a bound or a stop refused
//! ([`ErrorKind::is_refusal`], with the [`Refusal`]) is never read as a
//! fault of its input, and its [`code`](Error::code) is the stable reason
//! a program branches on. The types a caller may match on are variants of
//! it, converted without loss: a proof that fails the checker gives a
//! [`CheckError`], [`Invalid`](CheckError::Invalid) or
//! [`Refused`](CheckError::Refused); a proof structure a [`NetError`]; a
//! sequent without an intuitionistic reading a [`ShapeError`]; a text that
//! is no sequent a `ParseError`. Their messages name occurrences by id,
//! and `describe` on any of them, given the forest or the value the ids
//! belong to (an [`Owner`]), prints them as formulas ([`Described`]).
//!
//! # Syntax and JSON
//!
//! [`Sequent`] says how a sequent is written, which is the text that
//! `"…".parse::<Sequent>()` reads and its `Display` writes, and what its
//! JSON is. The JSON forms are the interchange format of the command, of
//! front ends and of files; those of [`Proof`], [`Outcome`],
//! [`ProofStructure`] and `Interactive` are described on those types.
//! [`ordinary::Sequent`] says how a sequent of ordinary logic is written.
//!
//! # Features
//!
//! On by default: `parse` (the text syntaxes, the LLTP reader `lltp`, the
//! reader of coverability problems `mist`, the generated `families` and
//! the TPTP reader `ordinary::read_tptp`),
//! `serialize` (the JSON forms, through serde), `interactive`
//! (`Interactive`, proving step by step), and the exports
//! `latex`, `typst`, `svg` and `rocq` (under [`export`]). Off by default:
//! `parallel` (the search on a pool of [`Options::jobs`] threads, and a
//! pool kept across searches, `search::Pool`; never for WebAssembly), and
//! `png` and `pdf` (the SVG drawings rendered). An item that needs a
//! feature says so.

#![allow(dead_code)]
#![allow(unused_variables)]

/// The error types of this crate.
mod errors;
/// Export of sequents and derivations to LaTeX, Typst and SVG, and of
/// proof nets to SVG.
pub mod export;
/// Problem families with known verdicts, for benchmarks and tests.
#[cfg(feature = "parse")]
pub mod families;
/// Fragments of linear logic, modes of proof search, and fragment detection.
pub mod fragment;
/// The hash tables of this crate.
mod hash;
/// The resources a call may use, its progress, and why it was refused.
pub mod limits;
/// Problems of the LLTP benchmark library.
#[cfg(feature = "parse")]
pub mod lltp;
/// Coverability problems in the `.spec` format of the Mist tool.
#[cfg(feature = "parse")]
pub mod mist;
/// Proof nets.
pub mod nets;
/// The occurrence forest of a sequent and sets over it.
pub mod occurrences;
/// Ordinary propositional logic through its embeddings into linear logic.
pub mod ordinary;
/// Parsing sequents from text.
#[cfg(feature = "parse")]
mod parse;
/// Proof terms, the checker and derivations.
pub mod proofs;
/// Proof search.
pub mod search;
/// Sequents and the terms they are built from.
pub mod sequents;
/// Serde support for sequents and proofs.
#[cfg(feature = "serialize")]
mod serialize;
pub mod settings;

#[cfg(feature = "parse")]
pub use errors::ParseError;

pub use errors::{Described, Error, ErrorKind, Owner};
pub use fragment::{Fragment, Mode};
pub use limits::{Limits, Phase, Progress, Refusal};
pub use nets::{Criterion, NetError, ProofStructure, VertexId};
pub use occurrences::{Forest, Member, OccId, Reading, ShapeError, Side, Sign};
pub use proofs::{
    Branch, CheckError, Derivation, InfId, Inference, Named, Node, NodeId, Proof, Rule, Size,
    ViewOptions,
};
#[cfg(feature = "interactive")]
pub use proofs::{GoalId, Interactive, Step, StepError};
pub use search::{
    Bias, Engine, Options, Outcome, Reason, Refutation, Statistics, Verdict, prove, prove_goal,
    prove_within,
};
pub use sequents::{Atom, Formula, Kind, Sequent, Term, TermId};
pub use settings::{Clock, Settings};
