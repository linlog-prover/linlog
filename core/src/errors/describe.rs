// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use super::{Error, INVALID_PROOF};
use crate::Mode;
use crate::nets::{MellStructure, NetError, ProofStructure};
use crate::occurrences::instances::Framing;
use crate::occurrences::{Forest, Instances, Member, OccId, Reading, ShapeError};
use crate::proofs::{CheckError, Derivation, Proof};
use crate::sequents::notation::TEXT;
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

/// What an error report names occurrences by: the forest's formulas, and
/// under an intuitionistic reading the formulas as the reading writes
/// them, two-sided.
#[derive(Clone, Copy)]
pub(crate) struct Names<'a> {
    /// The forest the ids are occurrences of.
    pub(crate) forest: &'a Forest,
    /// The reading of an owner in an intuitionistic mode.
    pub(crate) reading: Option<&'a Reading<'a>>,
    /// The instances of an owner that has them.
    pub(crate) framing: Option<&'a Framing<'a>>,
}

/// The most characters of a term an error writes in an instance, past
/// which it is cut: a term of shared subterms in a short file can unfold
/// to more than any memory holds.
pub(crate) const TERM: usize = 1 << 12;

/// Writes a member as its formula when names are given, an instance as
/// its owner's table says, else as its id.
pub(crate) fn occurrence(
    f: &mut (impl Write + ?Sized),
    names: Option<Names<'_>>,
    o: Member,
) -> fmt::Result {
    match names {
        Some(Names {
            forest,
            reading,
            framing: Some(framing),
        }) if o.index() >= forest.len() => {
            let mut text = String::new();
            TEXT.instance(&mut text, forest, reading, framing, o, Some(TERM))?;
            f.write_str(&text)
        }
        // An instance whose owner's table the names lack, by its id.
        Some(Names { forest, .. }) if o.index() >= forest.len() => write!(f, "#{}", o.get()),
        Some(Names {
            reading: Some(reading),
            ..
        }) => write!(f, "{}", reading.formula(o.occ())),
        Some(Names { forest, .. }) => write!(f, "{}", forest.formula(o.occ())),
        None => write!(f, "{}", o.get()),
    }
}

/// Writes a member as [`occurrence`] does, cut after `limit` characters
/// with `…` after it.
pub(crate) fn cut(
    f: &mut (impl Write + ?Sized),
    names: Option<Names<'_>>,
    o: Member,
    limit: Limit,
) -> fmt::Result {
    let mut budget = Budget::new(f, limit);
    match occurrence(&mut budget, names, o) {
        Err(_) if budget.cut => budget.f.write_char('…'),
        written => written,
    }
}

/// What owns the occurrences an error names, whose formulas [`Described`]
/// prints: a forest, or a value built on one. Sealed: the crate's types
/// alone implement it.
pub trait Owner: crate::sealed::Owner {
    /// Returns the forest whose occurrences the error's ids name.
    fn forest(&self) -> &Forest;
}

impl crate::sealed::Owner for Forest {}

impl Owner for Forest {
    /// The forest itself.
    fn forest(&self) -> &Forest {
        self
    }
}

impl crate::sealed::Owner for Proof {
    /// The mode the proof records.
    fn mode(&self) -> Option<Mode> {
        Proof::mode(self)
    }

    /// The proof's instances.
    fn instances(&self) -> Option<&Instances> {
        Proof::instances(self)
    }
}

impl Owner for Proof {
    /// The proof's forest.
    fn forest(&self) -> &Forest {
        Proof::forest(self)
    }
}

impl crate::sealed::Owner for Derivation<'_> {
    /// An intuitionistic mode where the derivation is two-sided.
    fn mode(&self) -> Option<Mode> {
        self.reading().map(|reading| {
            if reading.is_ordered() {
                Mode::LAMBEK
            } else {
                Mode::INTUITIONISTIC
            }
        })
    }
}

impl Owner for Derivation<'_> {
    /// The forest of the proof the derivation unfolds.
    fn forest(&self) -> &Forest {
        Derivation::forest(self)
    }
}

impl crate::sealed::Owner for ProofStructure {}

impl crate::sealed::Owner for MellStructure {
    /// The occurrence each vertex stands for.
    fn vertices(&self) -> Option<&[OccId]> {
        Some(self.occurrences())
    }
}

impl Owner for MellStructure {
    /// The structure's forest.
    fn forest(&self) -> &Forest {
        MellStructure::forest(self)
    }
}

impl Owner for ProofStructure {
    /// The structure's forest.
    fn forest(&self) -> &Forest {
        ProofStructure::forest(self)
    }
}

#[cfg(feature = "interactive")]
impl crate::sealed::Owner for crate::Interactive {
    /// The session's mode.
    fn mode(&self) -> Option<Mode> {
        Some(crate::Interactive::mode(self))
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
    /// The occurrence of every vertex, where vertices are no occurrences.
    vertices: Option<&'a [OccId]>,
    /// The most characters of a formula or a list of formulas, or `None`.
    limit: Option<usize>,
    /// The mode whose reading writes the formulas, where it is an
    /// intuitionistic one.
    mode: Option<Mode>,
    /// The owner's instances, where it has them.
    instances: Option<&'a Instances>,
}

impl<'a> Described<'a> {
    /// Returns `error` to be printed with the formulas of `owner`.
    pub(crate) fn new(error: Subject<'a>, owner: &'a impl Owner) -> Self {
        Self {
            error,
            forest: owner.forest(),
            vertices: owner.vertices(),
            limit: None,
            mode: owner.mode(),
            instances: owner.instances(),
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

    /// Returns the report with the formulas written as `mode` reads them:
    /// in an intuitionistic mode two-sided, under the sequent's reading
    /// (`A ⊸ B` where the one-sided sequent has `A ⊗ ~B` as a
    /// hypothesis), where the sequent has one; in a classical mode, and
    /// for a sequent without a reading, as the one-sided sequent writes
    /// them. A proof that records its mode, a derivation and a session
    /// give theirs already; this is for an owner that records none, such
    /// as a forest or a proof structure.
    #[must_use]
    pub const fn in_mode(self, mode: Mode) -> Self {
        Self {
            mode: Some(mode),
            ..self
        }
    }
}

impl Display for Described<'_> {
    /// Writes the error with formulas instead of occurrence ids.
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let reading = self
            .mode
            .filter(|mode| mode.is_intuitionistic())
            .and_then(|mode| Reading::of_mode(self.forest, mode.with_empty_antecedents()).ok());
        let framing = self
            .instances
            .map(|table| Framing::new(&self.forest.sequent().fo, table));
        let forest = Some(Names {
            forest: self.forest,
            reading: reading.as_ref(),
            framing: framing.as_ref(),
        });
        match self.error {
            Subject::Error(Error::Net(e)) => {
                f.write_str("not a proof net: ")?;
                e.write_over(f, forest, self.vertices, self.limit)
            }
            Subject::Net(e) => e.write_over(f, forest, self.vertices, self.limit),
            Subject::Error(e) => e.write(f, forest, self.limit),
            Subject::Check(e) => e.write(f, forest, self.limit),
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
        names: Option<Names<'_>>,
        limit: Option<usize>,
    ) -> fmt::Result {
        if names.is_none() {
            return Display::fmt(self, f);
        }
        match self {
            Self::Check(e) => {
                if let CheckError::Invalid(_) = e {
                    f.write_str(INVALID_PROOF)?;
                }
                e.write(f, names, limit)
            }
            Self::Rejected(e) => {
                f.write_str(REJECTED)?;
                e.write(f, names, limit)
            }
            Self::Net(e) => {
                f.write_str("not a proof net: ")?;
                e.write(f, names, limit)
            }
            Self::NotIntuitionistic(e) => {
                f.write_str("not an intuitionistic sequent: ")?;
                e.write(f, names, limit)
            }
            _ => Display::fmt(self, f),
        }
    }
}

/// What [`Error::Rejected`] says before the checker's error.
pub(super) const REJECTED: &str = "the proof the search found does not pass the checker, which \
                                   is a defect of the engine and no verdict on the sequent: ";
