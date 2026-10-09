// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use crate::limits::{Limits, Refusal};
use crate::sequents::{Sequent as Seq, Term, TermId};
use crate::wire::{self, Readable};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// The serialized form of an arena term. Its tags are part of the interchange
/// format, so renaming one breaks it.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(rename = "E")]
enum Expression {
    /// A variable, by dictionary index.
    #[serde(rename = "V")]
    Atom(u32),
    /// A negated variable, by dictionary index.
    #[serde(rename = "D")]
    DualAtom(u32),
    /// `1`
    #[serde(rename = "1")]
    One,
    /// `⊥`
    #[serde(rename = "⊥")]
    Bot,
    /// `⊤`
    #[serde(rename = "⊤")]
    Top,
    /// `0`
    #[serde(rename = "0")]
    Zero,
    /// `A ⊗ B`
    #[serde(rename = "⊗")]
    Tensor(u32, u32),
    /// `A ⅋ B`
    #[serde(rename = "⅋")]
    Par(u32, u32),
    /// `A & B`
    #[serde(rename = "&")]
    With(u32, u32),
    /// `A ⊕ B`
    #[serde(rename = "⊕")]
    Plus(u32, u32),
    /// `!A`
    #[serde(rename = "!")]
    Bang(u32),
    /// `?A`
    #[serde(rename = "?")]
    Quest(u32),
}

/// The serialized form of a sequent: with `version` at the top of a
/// document, without it nested in another form.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(super) struct Sequent {
    /// The wire level, written only at the top of a document.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "wire::version"
    )]
    version: Option<u32>,
    /// The arena.
    terms: Vec<Expression>,
    /// The root formulas, as arena indices.
    roots: Vec<u32>,
    /// The atom names.
    atoms: Vec<String>,
    /// How many of the root formulas, the first, stand left of `⊢`;
    /// absent where the sides are not known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    antecedents: Option<u32>,
}

impl From<Term> for Expression {
    /// Converts an arena term into its serialized form.
    fn from(e: Term) -> Self {
        use Expression as E;
        use Term::*;
        match e {
            Atom(a) => E::Atom(a.get()),
            DualAtom(a) => E::DualAtom(a.get()),
            One => E::One,
            Bot => E::Bot,
            Top => E::Top,
            Zero => E::Zero,
            Tensor(k, l) => E::Tensor(k.get(), l.get()),
            Par(k, l) => E::Par(k.get(), l.get()),
            With(k, l) => E::With(k.get(), l.get()),
            Plus(k, l) => E::Plus(k.get(), l.get()),
            Bang(k) => E::Bang(k.get()),
            Quest(k) => E::Quest(k.get()),
        }
    }
}

impl From<Expression> for Term {
    /// Converts a serialized expression back into an arena term.
    fn from(e: Expression) -> Self {
        use Expression::*;
        use Term as E;
        let t = TermId::new;
        match e {
            Atom(a) => E::Atom(crate::sequents::Atom::new(a)),
            DualAtom(a) => E::DualAtom(crate::sequents::Atom::new(a)),
            One => E::One,
            Bot => E::Bot,
            Top => E::Top,
            Zero => E::Zero,
            Tensor(k, l) => E::Tensor(t(k), t(l)),
            Par(k, l) => E::Par(t(k), t(l)),
            With(k, l) => E::With(t(k), t(l)),
            Plus(k, l) => E::Plus(t(k), t(l)),
            Bang(k) => E::Bang(t(k)),
            Quest(k) => E::Quest(t(k)),
        }
    }
}

impl From<&Seq> for Sequent {
    /// Converts a sequent into its serialized form nested in another.
    fn from(s: &Seq) -> Sequent {
        Sequent {
            version: None,
            terms: s.terms.iter().copied().map(Expression::from).collect(),
            roots: s.roots.iter().map(|k| k.get()).collect(),
            atoms: s.atoms.clone(),
            antecedents: s.antecedents,
        }
    }
}

impl Sequent {
    /// Returns the sequent the form holds, checked, within
    /// `limits.occurrences`.
    pub(super) fn within(self, limits: &Limits) -> Result<Seq, crate::Error> {
        let sequent = Seq::try_from(self)?;
        let occurrences = sequent.occurrences();
        match limits.occurrences {
            Some(limit) if occurrences > limit => {
                Err(crate::Error::Refused(Refusal::Occurrences {
                    occurrences,
                    limit,
                }))
            }
            _ => Ok(sequent),
        }
    }
}

impl TryFrom<Sequent> for Seq {
    type Error = crate::Error;

    /// Converts a deserialized sequent back, failing if its arena breaks the
    /// invariants or an atom's name is none the text syntax reads as that
    /// atom. A name the dictionary repeats is one atom, as it is in a
    /// parsed sequent.
    fn try_from(s: Sequent) -> Result<Seq, Self::Error> {
        for name in &s.atoms {
            crate::sequents::name::check(name)?;
        }
        let mut s = Seq {
            terms: s.terms.into_iter().map(Term::from).collect(),
            roots: s.roots.into_iter().map(TermId::new).collect(),
            atoms: s.atoms,
            antecedents: s.antecedents,
        };
        s.check()?;
        s.merge_atoms();
        Ok(s)
    }
}

impl serde::Serialize for Seq {
    /// Serializes the sequent as a document: its level, arena, root term
    /// indices and atom names.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut proxy = Sequent::from(self);
        proxy.version = wire::level();
        proxy.serialize(serializer)
    }
}

impl<'a> serde::Deserialize<'a> for Seq {
    /// Deserializes a sequent within the default limits, as
    /// [`wire::upgrade`] does.
    fn deserialize<D: Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        Self::read(deserializer, &Limits::default())
    }
}

impl Readable for Seq {
    const FORM: &'static str = "sequent";

    /// Reads a sequent and checks that its arena keeps the invariants and
    /// that it unfolds to no more occurrences than the limits allow.
    fn read<'de, D: Deserializer<'de>>(deserializer: D, limits: &Limits) -> Result<Self, D::Error> {
        let proxy = Sequent::deserialize(deserializer)?;
        proxy.within(limits).map_err(wire::fail)
    }
}
