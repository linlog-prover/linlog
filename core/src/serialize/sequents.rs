// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use crate::limits::{Limits, Refusal};
use crate::sequents::fo::{Args, FirstOrder, FoTerm, FoTermId, Symbol};
use crate::sequents::{Sequent as Seq, Term, TermId, name};
use crate::wire::{self, Readable};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::borrow::Cow;

/// The serialized form of a first-order term: a bound variable by its
/// index, or a function symbol applied to terms of the `fo` arena.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename = "T")]
pub(super) enum FoExpression {
    /// A bound variable: `0` is the innermost binder's.
    #[serde(rename = "B")]
    Bound(u32),
    /// A symbol, by its index in `symbols`, applied to terms by their
    /// index in `fo`, each before this one.
    #[serde(rename = "F")]
    App(u32, Vec<u32>),
    /// An eigenvariable by its number: only in a proof's extension of the
    /// arena, never in a sequent's.
    #[serde(rename = "E")]
    Eigen(u32),
}

/// The serialized form of an atom: its name alone in a propositional
/// sequent, or in a first-order one its predicate symbol by index in
/// `symbols` applied to terms by their index in `fo`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(untagged)]
pub(super) enum AtomForm {
    /// A nullary atom of a propositional sequent, by name.
    Name(String),
    /// A predicate applied to its arguments, boxed so that a list of
    /// names stays a list of strings in size.
    Predicate(Box<Predicate>),
}

impl<'de> Deserialize<'de> for AtomForm {
    /// Reads a string as a name and an object as a predicate, without
    /// buffering the value as an untagged enum would.
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        /// The two shapes an atom takes.
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = AtomForm;

            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("an atom's name, or a predicate applied to its arguments")
            }

            fn visit_str<E: serde::de::Error>(self, name: &str) -> Result<AtomForm, E> {
                Ok(AtomForm::Name(name.to_owned()))
            }

            fn visit_string<E: serde::de::Error>(self, name: String) -> Result<AtomForm, E> {
                Ok(AtomForm::Name(name))
            }

            fn visit_map<A: serde::de::MapAccess<'de>>(self, map: A) -> Result<AtomForm, A::Error> {
                Predicate::deserialize(serde::de::value::MapAccessDeserializer::new(map))
                    .map(|predicate| AtomForm::Predicate(Box::new(predicate)))
            }
        }
        deserializer.deserialize_any(Visitor)
    }
}

/// A predicate symbol applied to arguments, as `{"P": [symbol, [terms]]}`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename = "A")]
pub(super) enum Predicate {
    /// The symbol and its arguments.
    #[serde(rename = "P")]
    P(u32, Vec<u32>),
}

/// The atoms of a form: names alone, as a propositional sequent writes
/// them and reads them as it did before terms, or atoms of both shapes.
#[derive(Clone, Debug)]
pub(super) enum Atoms {
    /// Every atom by name.
    Names(Vec<String>),
    /// Atoms by name or as predicates.
    Mixed(Vec<AtomForm>),
}

impl Serialize for Atoms {
    /// Writes the atoms as the sequence they are.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Atoms::Names(names) => names.serialize(serializer),
            Atoms::Mixed(atoms) => atoms.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for Atoms {
    /// Reads the atoms as names while every one is a name, as predicates
    /// from the first predicate on.
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        /// The sequence of atoms.
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = Atoms;

            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("the atoms, by name or as predicates")
            }

            fn visit_seq<A: serde::de::SeqAccess<'de>>(
                self,
                mut seq: A,
            ) -> Result<Atoms, A::Error> {
                let mut atoms = Atoms::Names(Vec::new());
                while seq.next_element_seed(Push(&mut atoms))?.is_some() {}
                Ok(atoms)
            }
        }
        /// Reads one atom into the list, which turns mixed at the first
        /// predicate.
        struct Push<'a>(&'a mut Atoms);
        impl<'de> serde::de::DeserializeSeed<'de> for Push<'_> {
            type Value = ();

            fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<(), D::Error> {
                deserializer.deserialize_any(self)
            }
        }
        impl<'de> serde::de::Visitor<'de> for Push<'_> {
            type Value = ();

            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("an atom's name, or a predicate applied to its arguments")
            }

            fn visit_str<E: serde::de::Error>(self, name: &str) -> Result<(), E> {
                match self.0 {
                    Atoms::Names(names) => names.push(name.to_owned()),
                    Atoms::Mixed(atoms) => atoms.push(AtomForm::Name(name.to_owned())),
                }
                Ok(())
            }

            fn visit_string<E: serde::de::Error>(self, name: String) -> Result<(), E> {
                match self.0 {
                    Atoms::Names(names) => names.push(name),
                    Atoms::Mixed(atoms) => atoms.push(AtomForm::Name(name)),
                }
                Ok(())
            }

            fn visit_map<A: serde::de::MapAccess<'de>>(self, map: A) -> Result<(), A::Error> {
                let predicate =
                    Predicate::deserialize(serde::de::value::MapAccessDeserializer::new(map))?;
                if let Atoms::Names(names) = self.0 {
                    let mixed = std::mem::take(names)
                        .into_iter()
                        .map(AtomForm::Name)
                        .collect();
                    *self.0 = Atoms::Mixed(mixed);
                }
                let Atoms::Mixed(atoms) = self.0 else {
                    unreachable!("mixed from the first predicate on");
                };
                atoms.push(AtomForm::Predicate(Box::new(predicate)));
                Ok(())
            }
        }
        deserializer.deserialize_seq(Visitor)
    }
}

/// The first-order tables of a form, as its `symbols`, `fo` and `atoms`
/// keys hold them, and what builds them from a sequent's.
pub(super) struct Tables {
    /// The symbols, each a name with its arity.
    pub(super) symbols: Vec<(String, u32)>,
    /// The first-order terms.
    pub(super) fo: Vec<FoExpression>,
    /// The atoms, by name or as predicates.
    pub(super) atoms: Atoms,
}

impl Tables {
    /// Returns the forms of `sequent`'s tables: the atoms by name where
    /// none is a predicate, else every one as a predicate.
    pub(super) fn of(sequent: &Seq) -> Self {
        let fo = &sequent.fo;
        if fo.atoms.is_empty() {
            return Self {
                symbols: vec![],
                fo: vec![],
                atoms: Atoms::Names(sequent.atoms.clone()),
            };
        }
        let list = |args: Args| fo.list(args).iter().map(|t| t.get()).collect();
        Self {
            symbols: fo.symbols.clone(),
            fo: fo
                .terms
                .iter()
                .map(|term| match *term {
                    FoTerm::Bound(i) => FoExpression::Bound(i),
                    FoTerm::App(f, args) => FoExpression::App(f.get(), list(args)),
                    FoTerm::Eigen(a) => FoExpression::Eigen(a.get()),
                })
                .collect(),
            atoms: Atoms::Mixed(
                fo.atoms
                    .iter()
                    .map(|&(p, args)| {
                        AtomForm::Predicate(Box::new(Predicate::P(p.get(), list(args))))
                    })
                    .collect(),
            ),
        }
    }

    /// Returns the tables the forms hold and the atoms' names: a name's
    /// in NFC, a predicate's its canonical text. A name in a sequent with
    /// a symbol, a term or a predicate is the nullary symbol of that
    /// name, added where the table lacks it; a document of names alone
    /// keeps them as names, whatever its binders. The tables are checked
    /// (`FirstOrder::check`) before a key is built, and the terms every
    /// atom unfolds to are counted against `limits.occurrences` first:
    /// the terms are shared, so a short document could name a key of
    /// exponential length.
    pub(super) fn read(self, limits: &Limits) -> Result<(FirstOrder, Vec<String>), crate::Error> {
        let mut tables = FirstOrder::new();
        let first_order = !self.symbols.is_empty() || !self.fo.is_empty();
        let atoms = match self.atoms {
            Atoms::Names(mut names) if !first_order => {
                for name in &mut names {
                    if let Cow::Owned(composed) = name::normalized(name) {
                        *name = composed;
                    }
                    name::check(name)?;
                }
                return Ok((tables, names));
            }
            Atoms::Names(names) => names.into_iter().map(AtomForm::Name).collect(),
            Atoms::Mixed(atoms) => atoms,
        };
        let mut names = Vec::with_capacity(atoms.len());
        tables.symbols = self
            .symbols
            .into_iter()
            .map(|(name, arity)| Ok((normalized(name)?, arity)))
            .collect::<Result<_, crate::Error>>()?;
        // Equal lists are one entry, as the parser interns them.
        let mut known = crate::hash::HashMap::<Vec<u32>, Args>::default();
        let mut list = |tables: &mut FirstOrder, items: Vec<u32>| {
            if let Some(&args) = known.get(&items) {
                return args;
            }
            let args = Args::new(tables.lists.len() as u32);
            let start = tables.args.len() as u32;
            tables.args.extend(items.iter().map(|&t| FoTermId::new(t)));
            tables.lists.push((start, items.len() as u32));
            known.insert(items, args);
            args
        };
        for term in self.fo {
            let term = match term {
                FoExpression::Bound(i) => FoTerm::Bound(i),
                FoExpression::App(f, items) => {
                    let args = list(&mut tables, items);
                    FoTerm::App(Symbol::new(f), args)
                }
                FoExpression::Eigen(_) => {
                    return Err(crate::Error::Json {
                        form: "sequent",
                        message: "an eigenvariable stands only in a proof's terms".to_owned(),
                    });
                }
            };
            tables.terms.push(term);
        }
        for atom in atoms {
            let (symbol, args) = match atom {
                AtomForm::Predicate(predicate) => {
                    let Predicate::P(p, items) = *predicate;
                    (Symbol::new(p), list(&mut tables, items))
                }
                AtomForm::Name(name) => {
                    let name = normalized(name)?;
                    let symbol = match tables.symbols.iter().position(|(n, _)| *n == name) {
                        Some(i) => Symbol::new(i as u32),
                        None => {
                            tables.symbols.push((name, 0));
                            Symbol::new(tables.symbols.len() as u32 - 1)
                        }
                    };
                    (symbol, list(&mut tables, vec![]))
                }
            };
            tables.atoms.push((symbol, args));
        }
        // The keys, once every index the key reads is known to exist and
        // their unfolded size is within the limit.
        tables.check(tables.atoms.len())?;
        let unfolded = tables.unfolded();
        let terms = tables.atoms.iter().fold(0u64, |sum, &(_, args)| {
            tables
                .list(args)
                .iter()
                .fold(sum, |sum, t| sum.saturating_add(unfolded[t.index()]))
        });
        if let Some(limit) = limits.terms
            && terms > limit
        {
            return Err(crate::Error::Refused(Refusal::Terms { terms, limit }));
        }
        for &(p, args) in &tables.atoms {
            names.push(tables.atom_key(p, args));
        }
        Ok((tables, names))
    }
}

/// Returns `name` in NFC, checked to be an atom name.
fn normalized(name: String) -> Result<String, crate::Error> {
    let name = match name::normalized(&name) {
        Cow::Owned(composed) => composed,
        Cow::Borrowed(_) => name,
    };
    name::check(&name)?;
    Ok(name)
}

/// The serialized form of an arena term. Its tags are part of the interchange
/// format, so renaming one breaks it. A binder's variable is named in the
/// form's `binders`, beside the terms, so that a term stays three words.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
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
    /// `∀x.A`: the body; the variable's name stands in `binders`.
    #[serde(rename = "∀")]
    Forall(u32),
    /// `∃x.A`: the body; the variable's name stands in `binders`.
    #[serde(rename = "∃")]
    Exists(u32),
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
    /// The atoms: names, or predicates in a first-order sequent.
    atoms: Atoms,
    /// The symbols of a first-order sequent, each a name with its arity;
    /// absent in a propositional one.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    symbols: Vec<(String, u32)>,
    /// The first-order terms; absent in a propositional sequent.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    fo: Vec<FoExpression>,
    /// The names of the binders' variables, one per `∀` or `∃` term in
    /// the order of the terms, for printing; absent without binders, and
    /// `x` where a name is missing.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    binders: Vec<String>,
    /// How many of the root formulas, the first, stand left of `⊢`;
    /// absent where the sides are not known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    antecedents: Option<u32>,
    /// The cut formulas, as arena indices, in the order they were added;
    /// absent where there are none. Each one's dual is built on reading.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    cuts: Vec<u32>,
}

impl Expression {
    /// Returns the serialized form of the arena term `e`.
    fn of(e: Term) -> Self {
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
            Forall(k) => E::Forall(k.get()),
            Exists(k) => E::Exists(k.get()),
        }
    }

    /// Returns the arena term of the form.
    fn into_term(self) -> Term {
        use Expression::*;
        use Term as E;
        let t = TermId::new;
        match self {
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
            Forall(k) => E::Forall(t(k)),
            Exists(k) => E::Exists(t(k)),
        }
    }
}

impl From<&Seq> for Sequent {
    /// Converts a sequent into its serialized form nested in another.
    fn from(s: &Seq) -> Sequent {
        let tables = Tables::of(s);
        Sequent {
            version: None,
            terms: s.terms.iter().copied().map(Expression::of).collect(),
            binders: s.fo.binders.iter().map(|(_, name)| name.clone()).collect(),
            roots: s.roots.iter().map(|k| k.get()).collect(),
            atoms: tables.atoms,
            symbols: tables.symbols,
            fo: tables.fo,
            antecedents: s.antecedents,
            cuts: s.cuts.iter().map(|[a, _]| a.get()).collect(),
        }
    }
}

impl Sequent {
    /// Returns the sequent the form holds, checked, within
    /// `limits.occurrences`.
    pub(super) fn within(self, limits: &Limits) -> Result<Seq, crate::Error> {
        let sequent = Seq::read_within(self, limits)?;
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
    /// atom. A name the dictionary repeats, in NFC, is one atom, as it is
    /// in a parsed sequent.
    fn try_from(s: Sequent) -> Result<Seq, Self::Error> {
        Self::read_within(s, &Limits::default())
    }
}

impl Seq {
    /// Converts a deserialized sequent back as `TryFrom` does, the terms
    /// of its atoms counted against `limits.occurrences`.
    fn read_within(s: Sequent, limits: &Limits) -> Result<Seq, crate::Error> {
        let (mut fo, atoms) = Tables {
            symbols: s.symbols,
            fo: s.fo,
            atoms: s.atoms,
        }
        .read(limits)?;
        let cuts = s.cuts;
        let terms: Vec<Term> = s.terms.into_iter().map(Expression::into_term).collect();
        if !s.binders.is_empty() {
            let mut names = s.binders.into_iter();
            for (n, term) in terms.iter().enumerate() {
                if term.kind().is_binder() {
                    let name = names.next().unwrap_or_else(|| "x".to_owned());
                    fo.binders.push((TermId::new(n as u32), normalized(name)?));
                }
            }
        }
        let mut s = Seq {
            terms,
            roots: s.roots.into_iter().map(TermId::new).collect(),
            atoms,
            antecedents: s.antecedents,
            cuts: vec![],
            fo,
        };
        s.check()?;
        let len = s.terms.len();
        if let Some(&a) = cuts.iter().find(|&&a| a as usize >= len) {
            return Err(crate::Error::IndexOutOfBounds {
                space: crate::limits::Space::Term,
                index: a as usize,
                len,
            });
        }
        let cuts: Vec<TermId> = cuts.into_iter().map(TermId::new).collect();
        s.push_cuts(&cuts);
        // A cut formula is closed as a root is, which the check sees only
        // once the cuts are the sequent's.
        if !cuts.is_empty() && !s.fo.is_empty() {
            s.check()?;
        }
        s.merge_atoms();
        Ok(s)
    }
}

impl serde::Serialize for Seq {
    /// Serializes the sequent as a document: its level, arena, root term
    /// indices and atom names.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut proxy = Sequent::from(self);
        proxy.version = Some(wire::level_of_sequent(self));
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

impl crate::sealed::Readable for Seq {}

impl Readable for Seq {
    const FORM: &'static str = "sequent";

    /// Reads a sequent and checks that its arena keeps the invariants and
    /// that it unfolds to no more occurrences than the limits allow.
    fn read<'de, D: Deserializer<'de>>(deserializer: D, limits: &Limits) -> Result<Self, D::Error> {
        let proxy = Sequent::deserialize(deserializer)?;
        proxy.within(limits).map_err(wire::fail)
    }
}
