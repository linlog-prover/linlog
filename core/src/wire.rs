// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The wire forms: how a value crosses a process boundary (JSON, or any
//! format serde writes), and the rule that keeps a stored document
//! readable by later releases. Needs the cargo feature `serialize` (on by
//! default).
//!
//! # The level
//!
//! Every top-level document starts with `"version": n`, the lowest wire
//! level whose reader understands all of it; [`LEVEL`] is the highest
//! this build reads and writes, and level 1 is this release's. The writer
//! computes the level from the content, so a value that a later release
//! represents as this one does keeps its bytes. A reader refuses a
//! document above its level by name ([`Error::Version`], code
//! `unsupported_version`), before the other keys when `version` comes
//! first, as the writer puts it; a document without `version` reads as
//! level 1. Nested values carry no `version`.
//!
//! What raises the level: a new tag, a new value of a closed enumeration
//! (a mode, a criterion), a key whose presence changes what the other keys
//! mean. What does not: a key an older reader may ignore without
//! misreading (a counter, an error's detail), and a new value of an open
//! enumeration with a meaning to fall back on (a refutation's kind reads as
//! `exhausted`).
//!
//! # Reading
//!
//! [`upgrade`] is the one reader of every form: it reads a document of any
//! level up to [`LEVEL`] as a value of this release, every older level
//! through one step per level that changed a form (at level 1, none), and
//! counts every sequent's occurrences against `limits.occurrences` before
//! anything unfolds. [`Within`] is the same as a serde seed, for a value
//! nested in a document of the caller's; the plain `Deserialize` impls read
//! within the default [`Limits`]. A data form (a sequent, a proof, a
//! structure, a session) ignores the keys it does not know, which is what
//! lets an outcome be read as a proof; an options form refuses them by
//! name. The keys every level-1 value has are required, so a document of
//! another shape is refused naming the key ([`Error::Json`]); the names
//! of the forms before this release (`ids`, `var_dict`, `proof`, a mode as
//! an object of flags) are not read.
//!
//! # The forms
//!
//! Enumerations are written by name, never by number; integers are exact
//! in JavaScript (a bound past 2⁵³ is written `null`, and a written count
//! that saturates is `u64::MAX`, meaning "at least"). With the running
//! example `A, A -o B |- B`, `⊢ ~A, A ⊗ ~B, B`:
//!
//! - **Sequent**: `{"version": 1, "terms": [{"D": 0}, {"V": 0}, {"D": 1},
//!   {"⊗": [1, 2]}, {"V": 1}], "roots": [0, 3, 4], "atoms": ["A", "B"],
//!   "antecedents": 2}`, see [`Sequent`](crate::Sequent).
//! - **Proof**: `{"version": 1, "sequent": {…}, "nodes": [{"ax": [0, 2]},
//!   {"ax": [3, 4]}, {"⊗": [1, 0, 1]}], "mode": "classical"}`, with `goal`
//!   for a proof of a goal off the roots, see [`Proof`](crate::Proof).
//! - **Proof structure**: `{"version": 1, "sequent": {…}, "mix": false,
//!   "links": [[0, 2], [3, 4]]}`, see
//!   [`ProofStructure`](crate::ProofStructure).
//! - **Session**: `{"version": 1, "sequent": {…}, "mode":
//!   "intuitionistic", "inferences": […], "history": […]}`, see
//!   `Interactive` (feature `interactive`).
//! - **Outcome** (written only): `{"version": 1, "linlog": "0.1.0",
//!   "verdict": "proved", "checked": true, "fragment": "MLL", "mode":
//!   "classical", "engine": "net", "statistics": {…}, "sequent": {…},
//!   "nodes": […]}`, a proof's keys for `proved` and a disproof's for
//!   `unprovable`, so that it reads back as a proof; see
//!   [`Outcome`](crate::Outcome).

use crate::Error;
use crate::limits::Limits;
use serde::de::{DeserializeSeed, Deserializer};
use std::cell::RefCell;
use std::marker::PhantomData;

/// The highest wire level this build reads and writes.
pub const LEVEL: u32 = 1;

/// A form [`upgrade`] and [`Within`] read: one whose reading counts a
/// sequent against the caller's limits. Implemented by the crate's data
/// forms; a type outside the crate may implement it to be read by the same
/// calls.
pub trait Readable: Sized {
    /// The form's name, as an error names it: `sequent`, `proof`, …
    const FORM: &'static str;

    /// Reads a value of the form from `deserializer`, every sequent within
    /// `limits.occurrences`.
    ///
    /// # Errors
    ///
    /// The deserializer's error for a document of another shape or a value
    /// the form's checks refuse.
    fn read<'de, D: Deserializer<'de>>(deserializer: D, limits: &Limits) -> Result<Self, D::Error>;
}

/// Reads a value of the form `T` within limits, as a serde seed: the
/// value nested in a document of the caller's that [`upgrade`] reads
/// whole.
#[derive(Clone, Copy, Debug)]
pub struct Within<'a, T> {
    /// The limits.
    limits: &'a Limits,
    /// The form read.
    form: PhantomData<fn() -> T>,
}

impl<'a, T> Within<'a, T> {
    /// Returns the seed that reads within `limits`.
    pub const fn new(limits: &'a Limits) -> Self {
        Self {
            limits,
            form: PhantomData,
        }
    }
}

impl<'de, T: Readable> DeserializeSeed<'de> for Within<'_, T> {
    type Value = T;

    /// Reads the value within the seed's limits.
    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<T, D::Error> {
        T::read(deserializer, self.limits)
    }
}

/// Reads a document of any released level up to [`LEVEL`] and returns it
/// as a value of this release, an older level through each level's step
/// in turn (at level 1 there is none), every sequent within
/// `limits.occurrences`. It reads one value and leaves the deserializer
/// there: a caller whose input is that one document ends the
/// deserializer (`end()` in serde_json), which refuses what follows it.
///
/// # Errors
///
/// [`Error::Version`] for a document above [`LEVEL`]; the error of the
/// form's checks for a value they refuse (an index out of bounds, a
/// sequent past the bound, links that are no links); [`Error::Json`] for a
/// document of another shape, naming what is missing or wrong.
///
/// # Examples
///
#[cfg_attr(feature = "parse", doc = "```")]
#[cfg_attr(not(feature = "parse"), doc = "```ignore")]
/// use linlog::{Error, Limits, Sequent, wire};
///
/// let json = r#"{"version": 1, "terms": [{"V": 0}], "roots": [0], "atoms": ["A"]}"#;
/// let mut document = serde_json::Deserializer::from_str(json);
/// let sequent: Sequent = wire::upgrade(&mut document, &Limits::default())?;
/// document.end().expect("one document and nothing after it");
/// assert_eq!(sequent.to_string(), "⊢ A");
///
/// let newer = r#"{"version": 9, "terms": []}"#;
/// let mut document = serde_json::Deserializer::from_str(newer);
/// let error = wire::upgrade::<Sequent, _>(&mut document, &Limits::default()).unwrap_err();
/// assert!(matches!(error, Error::Version { found: 9, .. }));
/// # Ok::<(), Error>(())
/// ```
pub fn upgrade<'de, T: Readable, D: Deserializer<'de>>(
    document: D,
    limits: &Limits,
) -> Result<T, Error> {
    CAUSE.with_borrow_mut(|cause| *cause = None);
    let read = T::read(document, limits);
    let cause = CAUSE.with_borrow_mut(Option::take);
    read.map_err(|error| match cause {
        // The key names no form; the caller's is the one read.
        Some(Error::Version {
            found, supported, ..
        }) => Error::Version {
            form: T::FORM,
            found,
            supported,
        },
        Some(cause) => cause,
        None => Error::Json {
            form: T::FORM,
            message: error.to_string(),
        },
    })
}

thread_local! {
    /// The error of the library's own that ended the read [`upgrade`] runs
    /// on this thread, which a deserializer's error carries as text only.
    static CAUSE: RefCell<Option<Error>> = const { RefCell::new(None) };
}

/// Returns the deserializer's error for one of the library's own, which
/// [`upgrade`] then answers instead of [`Error::Json`].
pub(crate) fn fail<E: serde::de::Error>(error: Error) -> E {
    let message = error.to_string();
    CAUSE.with_borrow_mut(|cause| *cause = Some(error));
    E::custom(message)
}

/// Reads a `version` key: a level up to [`LEVEL`], refused above it.
pub(crate) fn version<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<u32>, D::Error> {
    let found = <u64 as serde::Deserialize>::deserialize(deserializer)?;
    match u32::try_from(found) {
        Ok(level) if level <= LEVEL => Ok(Some(level)),
        _ => Err(fail(Error::Version {
            form: "document",
            found,
            supported: LEVEL,
        })),
    }
}

/// The level a written document carries: this release's.
pub(crate) const fn level() -> Option<u32> {
    Some(LEVEL)
}
