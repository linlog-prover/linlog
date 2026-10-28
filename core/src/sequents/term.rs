// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

#[cfg(any(test, feature = "parse", feature = "serialize"))]
use crate::Error;

/// The index of a term in a sequent's arena.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TermId(u32);

impl TermId {
    /// Wraps a raw arena index.
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    /// Returns the raw arena index.
    pub const fn get(self) -> u32 {
        self.0
    }

    /// Returns the index as a `usize`, for indexing the arena.
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

/// An atomic formula, by its index in a sequent's table of atoms: a
/// predicate symbol applied to its arguments, interned, so that two
/// literals of one atom are two occurrences of one atomic formula. Every
/// atom of a propositional sequent is nullary, and a ground atom (`p(a)`)
/// is propositional too: only an atom under a binder has a bound variable
/// among its arguments ([`Sequent::atom_is_open`](super::Sequent::atom_is_open)).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Atom(u32);

impl Atom {
    /// Wraps a raw dictionary index.
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    /// Returns the raw dictionary index.
    pub const fn get(self) -> u32 {
        self.0
    }

    /// Returns the index as a `usize`, for indexing the dictionary.
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

/// A node of a formula in negation normal form, which refers to its subterms
/// by arena index.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Term {
    /// An atom `a`.
    Atom(Atom),
    /// The negation `~a` of an atom.
    DualAtom(Atom),
    /// `1`, the unit of `⊗`.
    One,
    /// `⊥`, the unit of `⅋`.
    Bot,
    /// `⊤`, the unit of `&`.
    Top,
    /// `0`, the unit of `⊕`.
    Zero,
    /// `A ⊗ B`
    Tensor(TermId, TermId),
    /// `A ⅋ B`
    Par(TermId, TermId),
    /// `A & B`
    With(TermId, TermId),
    /// `A ⊕ B`
    Plus(TermId, TermId),
    /// `!A`
    Bang(TermId),
    /// `?A`
    Quest(TermId),
    /// `∀x.A`: a binder and its body, in which the variable it binds is
    /// the bound variable of index 0 and the variables of the binders
    /// above it are shifted by one (locally nameless, so two binders that
    /// differ in their variable's name alone are one term). The name is
    /// kept beside the arena for printing.
    Forall(TermId),
    /// `∃x.A`, a binder as [`Forall`](Self::Forall) is.
    Exists(TermId),
}

/// A term is a tag and two arena indices at most: every arena and every
/// table keyed by terms grows with it.
const _: () = assert!(size_of::<Term>() == 12);

impl std::hash::Hash for Term {
    /// Hashes the term as one word: its two indices, the second zero
    /// where there is none, and its kind in the top byte, over the second
    /// index's top bits, which only an arena past 2²⁴ terms fills; one
    /// write whatever the variant, where the derived form writes the
    /// discriminant and each field.
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        use Term::*;
        let (a, b) = match *self {
            Atom(a) | DualAtom(a) => (a.0, 0),
            One | Bot | Top | Zero => (0, 0),
            Tensor(k, l) | Par(k, l) | With(k, l) | Plus(k, l) => (k.0, l.0),
            Bang(k) | Quest(k) | Forall(k) | Exists(k) => (k.0, 0),
        };
        let kind = u64::from(self.kind() as u8) << 56;
        state.write_u64(u64::from(a) | (u64::from(b) << 32) ^ kind);
    }
}

/// The kind of a term: its connective, or which literal it is.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Kind {
    /// An atom `a`.
    Atom,
    /// The negation `~a` of an atom.
    DualAtom,
    /// `1`
    One,
    /// `⊥`
    Bot,
    /// `⊤`
    Top,
    /// `0`
    Zero,
    /// `⊗`
    Tensor,
    /// `⅋`
    Par,
    /// `&`
    With,
    /// `⊕`
    Plus,
    /// `!`
    Bang,
    /// `?`
    Quest,
    /// `∀`
    Forall,
    /// `∃`
    Exists,
}

impl Kind {
    /// Returns the kind of the dual term: `⊗` for `⅋`, `Atom` for `DualAtom`, …
    #[must_use]
    pub const fn dual(self) -> Self {
        use Kind::*;
        match self {
            Atom => DualAtom,
            DualAtom => Atom,
            One => Bot,
            Bot => One,
            Top => Zero,
            Zero => Top,
            Tensor => Par,
            Par => Tensor,
            With => Plus,
            Plus => With,
            Bang => Quest,
            Quest => Bang,
            Forall => Exists,
            Exists => Forall,
        }
    }

    /// Returns how many subterms a term of this kind has: 0, 1 or 2.
    pub const fn arity(self) -> u8 {
        use Kind::*;
        match self {
            Atom | DualAtom | One | Bot | Top | Zero => 0,
            Bang | Quest | Forall | Exists => 1,
            Tensor | Par | With | Plus => 2,
        }
    }

    /// Returns whether this is a literal, `a` or `~a`.
    pub const fn is_literal(self) -> bool {
        matches!(self, Kind::Atom | Kind::DualAtom)
    }

    /// Returns whether this is a binder, `∀` or `∃`.
    pub const fn is_binder(self) -> bool {
        matches!(self, Kind::Forall | Kind::Exists)
    }
}

impl Term {
    /// Returns the kind of this term.
    pub const fn kind(self) -> Kind {
        use Term::*;
        match self {
            Atom(_) => Kind::Atom,
            DualAtom(_) => Kind::DualAtom,
            One => Kind::One,
            Bot => Kind::Bot,
            Top => Kind::Top,
            Zero => Kind::Zero,
            Tensor(..) => Kind::Tensor,
            Par(..) => Kind::Par,
            With(..) => Kind::With,
            Plus(..) => Kind::Plus,
            Bang(_) => Kind::Bang,
            Quest(_) => Kind::Quest,
            Forall(_) => Kind::Forall,
            Exists(_) => Kind::Exists,
        }
    }

    /// Returns the atom of a literal, or `None` for any other term.
    pub const fn atom(self) -> Option<Atom> {
        match self {
            Term::Atom(a) | Term::DualAtom(a) => Some(a),
            Term::One
            | Term::Bot
            | Term::Top
            | Term::Zero
            | Term::Tensor(..)
            | Term::Par(..)
            | Term::With(..)
            | Term::Plus(..)
            | Term::Bang(_)
            | Term::Quest(_)
            | Term::Forall(_)
            | Term::Exists(_) => None,
        }
    }

    /// Returns the term with its top node dualised (`⊗` to `⅋`, `a` to `~a`,
    /// …) and the same subterm indices. Dualising a whole formula dualises
    /// every node, which `Sequent` does when it lowers a two-sided sequent.
    #[must_use]
    pub const fn dual(self) -> Self {
        use Term::*;
        match self {
            Atom(a) => DualAtom(a),
            DualAtom(a) => Atom(a),
            One => Bot,
            Bot => One,
            Top => Zero,
            Zero => Top,
            Tensor(k, l) => Par(k, l),
            Par(k, l) => Tensor(k, l),
            With(k, l) => Plus(k, l),
            Plus(k, l) => With(k, l),
            Bang(k) => Quest(k),
            Quest(k) => Bang(k),
            Forall(k) => Exists(k),
            Exists(k) => Forall(k),
        }
    }

    /// Returns the subterms in order: none for a literal or a unit, one for
    /// `!`, `?` and a binder (its body), the left one then the right one for a binary connective.
    pub fn subterms(self) -> impl Iterator<Item = TermId> {
        let (first, second) = self.operands();
        first.into_iter().chain(second)
    }

    /// Returns the first and the second subterm, each if there is one.
    pub(crate) const fn operands(self) -> (Option<TermId>, Option<TermId>) {
        use Term::*;
        match self {
            Tensor(k, l) | Par(k, l) | With(k, l) | Plus(k, l) => (Some(k), Some(l)),
            Bang(k) | Quest(k) | Forall(k) | Exists(k) => (Some(k), None),
            Atom(_) | DualAtom(_) | One | Bot | Top | Zero => (None, None),
        }
    }

    /// Returns the term with every subterm index replaced by what `f` maps it
    /// to.
    pub(crate) fn map_subterms(self, mut f: impl FnMut(TermId) -> TermId) -> Self {
        use Term::*;
        match self {
            Tensor(k, l) => Tensor(f(k), f(l)),
            Par(k, l) => Par(f(k), f(l)),
            With(k, l) => With(f(k), f(l)),
            Plus(k, l) => Plus(f(k), f(l)),
            Bang(k) => Bang(f(k)),
            Quest(k) => Quest(f(k)),
            Forall(k) => Forall(f(k)),
            Exists(k) => Exists(f(k)),
            leaf @ Atom(_)
            | leaf @ DualAtom(_)
            | leaf @ One
            | leaf @ Bot
            | leaf @ Top
            | leaf @ Zero => leaf,
        }
    }

    /// Shifts the atom and subterm indices by the given offsets.
    pub(crate) fn offset(self, atoms: u32, terms: u32) -> Self {
        use Term::*;
        match self {
            Atom(a) => Atom(self::Atom(a.0 + atoms)),
            DualAtom(a) => DualAtom(self::Atom(a.0 + atoms)),
            other @ One
            | other @ Bot
            | other @ Top
            | other @ Zero
            | other @ Tensor(..)
            | other @ Par(..)
            | other @ With(..)
            | other @ Plus(..)
            | other @ Bang(_)
            | other @ Quest(_)
            | other @ Forall(_)
            | other @ Exists(_) => other.map_subterms(|k| TermId(k.0 + terms)),
        }
    }

    /// Checks that the term refers only to atoms below `atom_bound` and to
    /// subterms below `term_bound`.
    #[cfg(any(test, feature = "parse", feature = "serialize"))]
    pub(crate) fn check_bounds(self, atom_bound: u32, term_bound: u32) -> Result<(), Error> {
        if let Some(a) = self.atom()
            && a.0 >= atom_bound
        {
            return Err(Error::IndexOutOfBounds {
                space: crate::limits::Space::Atom,
                index: a.index(),
                len: atom_bound as usize,
            });
        }
        for k in self.subterms() {
            if k.0 >= term_bound {
                return Err(Error::NotTopological {
                    space: crate::limits::Space::Term,
                    index: k.index(),
                    parent: term_bound as usize,
                });
            }
        }
        Ok(())
    }
}
