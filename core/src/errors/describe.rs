// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use super::{Error, INVALID_PROOF};
use crate::nets::{NetError, ProofStructure};
use crate::occurrences::OccId;
use crate::occurrences::{Forest, ShapeError};
use crate::proofs::{CheckError, Derivation, Proof};
use std::fmt::{self, Display, Formatter, Write};

/// The most characters a formula or a list of formulas of an error report
/// takes, or `None` for no bound.
pub(crate) type Limit = Option<usize>;

/// A writer that passes on at most `left` characters and then fails,
/// setting `cut`, so that a formula or a list of formulas of an error
/// report stops where its bound is reached.
pub(crate) struct Budget<'a, W: Write + ?Sized> {
    /// Where the text goes.
    pub(crate) f: &'a mut W,
    /// The characters it still passes on.
    pub(crate) left: usize,
    /// Whether text was held back.
    pub(crate) cut: bool,
}

impl<'a, W: Write + ?Sized> Budget<'a, W> {
    /// Returns a writer into `f` that passes on `limit` characters.
    pub(crate) fn new(f: &'a mut W, limit: Limit) -> Self {
        Self {
            f,
            left: limit.unwrap_or(usize::MAX),
            cut: false,
        }
    }
}

impl<W: Write + ?Sized> Write for Budget<'_, W> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let n = s.chars().count();
        if n <= self.left {
            self.left -= n;
            return self.f.write_str(s);
        }
        let head: String = s.chars().take(self.left).collect();
        self.f.write_str(&head)?;
        (self.left, self.cut) = (0, true);
        Err(fmt::Error)
    }
}

/// Writes an occurrence as its formula when a forest is given, else as
/// its id.
pub(crate) fn occurrence(
    f: &mut (impl Write + ?Sized),
    forest: Option<&Forest>,
    o: OccId,
) -> fmt::Result {
    match forest {
        Some(forest) => write!(f, "{}", forest.formula(o)),
        None => write!(f, "{}", o.get()),
    }
}

/// Writes an occurrence as [`occurrence`] does, cut after `limit`
/// characters with `…` after it.
pub(crate) fn cut(
    f: &mut (impl Write + ?Sized),
    forest: Option<&Forest>,
    o: OccId,
    limit: Limit,
) -> fmt::Result {
    let mut budget = Budget::new(f, limit);
    match occurrence(&mut budget, forest, o) {
        Err(_) if budget.cut => budget.f.write_char('…'),
        written => written,
    }
}

/// What owns the occurrences an error names, whose formulas [`Described`]
/// prints: a forest, or a value built on one.
pub trait Owner {
    /// Returns the forest whose occurrences the error's ids name.
    fn forest(&self) -> &Forest;
}

impl Owner for Forest {
    /// The forest itself.
    fn forest(&self) -> &Forest {
        self
    }
}

impl Owner for Proof {
    /// The proof's forest.
    fn forest(&self) -> &Forest {
        Proof::forest(self)
    }
}

impl Owner for Derivation<'_> {
    /// The forest of the proof the derivation unfolds.
    fn forest(&self) -> &Forest {
        Derivation::forest(self)
    }
}

impl Owner for ProofStructure {
    /// The structure's forest.
    fn forest(&self) -> &Forest {
        ProofStructure::forest(self)
    }
}

#[cfg(feature = "interactive")]
impl Owner for crate::Interactive {
    /// The session's forest.
    fn forest(&self) -> &Forest {
        crate::Interactive::forest(self)
    }
}

/// The error a [`Described`] prints.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Subject<'a> {
    /// Any error of the crate.
    Error(&'a Error),
    /// The checker's.
    Check(&'a CheckError),
    /// A proof structure's.
    Net(&'a NetError),
    /// The intuitionistic reading's.
    Shape(&'a ShapeError),
}

/// An error displayed with formulas instead of occurrence ids, read from
/// the owner of the ids, as `describe` on [`Error`](enum@Error), [`CheckError`],
/// [`NetError`] and [`ShapeError`] returns it; nodes and inferences keep
/// their numbers.
#[derive(Clone, Copy, Debug)]
pub struct Described<'a> {
    /// The error.
    error: Subject<'a>,
    /// The forest the ids are occurrences of.
    forest: &'a Forest,
    /// The most characters of a formula or a list of formulas, or `None`.
    limit: Option<usize>,
}

impl<'a> Described<'a> {
    /// Returns `error` to be printed with the formulas of `owner`.
    pub(crate) fn new(error: Subject<'a>, owner: &'a impl Owner) -> Self {
        Self {
            error,
            forest: owner.forest(),
            limit: None,
        }
    }

    /// Returns the report with every formula and every list of formulas
    /// of a proof's error cut after `limit` characters, `…` and the
    /// number of formulas of a list after it, or whole with `None`: a
    /// zone may hold a formula per node of the proof.
    #[must_use]
    pub const fn abbreviated(self, limit: Option<usize>) -> Self {
        Self { limit, ..self }
    }
}

impl Display for Described<'_> {
    /// Writes the error with formulas instead of occurrence ids.
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let forest = Some(self.forest);
        match self.error {
            Subject::Error(e) => e.write(f, forest, self.limit),
            Subject::Check(e) => e.write(f, forest, self.limit),
            Subject::Net(e) => e.write(f, forest, self.limit),
            Subject::Shape(e) => e.write(f, forest, self.limit),
        }
    }
}

#[cfg(feature = "serialize")]
impl serde::Serialize for Described<'_> {
    /// Writes the error's form with the message in formulas.
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let owned;
        let error = match self.error {
            Subject::Error(e) => e,
            Subject::Check(e) => {
                owned = Error::Check(e.clone());
                &owned
            }
            Subject::Net(e) => {
                owned = Error::Net(Box::new(e.clone()));
                &owned
            }
            Subject::Shape(e) => {
                owned = Error::NotIntuitionistic(e.clone());
                &owned
            }
        };
        crate::serialize::envelope(error, &self.to_string(), serializer)
    }
}

impl Error {
    /// Returns the error for display with formulas instead of occurrence
    /// ids, read from `owner`, the value whose occurrences the error names
    /// (the forest of the sequent, or the proof, derivation, structure or
    /// session that failed). An error that names no occurrence prints as
    /// [`Display`] does.
    pub fn describe<'a>(&'a self, owner: &'a impl Owner) -> Described<'a> {
        Described::new(Subject::Error(self), owner)
    }

    /// Writes the error as [`Display`] does, with formulas instead of ids
    /// when a forest is given.
    pub(crate) fn write(
        &self,
        f: &mut Formatter<'_>,
        forest: Option<&Forest>,
        limit: Option<usize>,
    ) -> fmt::Result {
        if forest.is_none() {
            return Display::fmt(self, f);
        }
        match self {
            Self::Check(e) => {
                if let CheckError::Invalid(_) = e {
                    f.write_str(INVALID_PROOF)?;
                }
                e.write(f, forest, limit)
            }
            Self::Rejected(e) => {
                f.write_str(REJECTED)?;
                e.write(f, forest, limit)
            }
            Self::Net(e) => {
                f.write_str("not a proof net: ")?;
                e.write(f, forest, limit)
            }
            Self::NotIntuitionistic(e) => {
                f.write_str("not an intuitionistic sequent: ")?;
                e.write(f, forest, limit)
            }
            _ => Display::fmt(self, f),
        }
    }
}

/// What [`Error::Rejected`] says before the checker's error.
pub(super) const REJECTED: &str = "the proof the search found does not pass the checker, which \
                                   is a defect of the engine and no verdict on the sequent: ";
