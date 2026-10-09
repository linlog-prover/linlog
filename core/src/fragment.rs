// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use crate::Error;
use crate::sequents::{Kind, Sequent};
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::ops::{BitAnd, BitOr, BitOrAssign};
use std::str::FromStr;

/// The connective classes a sequent uses, which name the fragment of
/// propositional linear logic it lives in. Fragments form a lattice under
/// [`union`](Self::union) and [`contains`](Self::contains); the named
/// constants are the usual fragments, and [`Sequent::fragment`] computes
/// the smallest one a sequent fits.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Fragment(u8);

impl Fragment {
    /// No connective at all: atoms only.
    pub const EMPTY: Self = Self(0);
    /// `⊗` and `⅋`.
    pub const MULTIPLICATIVES: Self = Self(1);
    /// `1` and `⊥`.
    pub const MULTIPLICATIVE_UNITS: Self = Self(2);
    /// `&` and `⊕`.
    pub const ADDITIVES: Self = Self(4);
    /// `⊤` and `0`.
    pub const ADDITIVE_UNITS: Self = Self(8);
    /// `!` and `?`.
    pub const EXPONENTIALS: Self = Self(16);

    /// Multiplicative linear logic without units: `⊗ ⅋`.
    pub const MLL: Self = Self::MULTIPLICATIVES;
    /// Multiplicative linear logic with units: `⊗ ⅋ 1 ⊥`.
    pub const MLL_WITH_UNITS: Self = Self::MLL.union(Self::MULTIPLICATIVE_UNITS);
    /// Additive linear logic, named `ALL`: `& ⊕ ⊤ 0`, and nothing
    /// multiplicative. Not to be confused with [`LL`](Self::LL), which has
    /// every connective.
    pub const ADDITIVE: Self = Self::ADDITIVES.union(Self::ADDITIVE_UNITS);
    /// Multiplicative-additive linear logic: `⊗ ⅋ 1 ⊥ & ⊕ ⊤ 0`.
    pub const MALL: Self = Self::MLL_WITH_UNITS.union(Self::ADDITIVE);
    /// Multiplicative-exponential linear logic: `⊗ ⅋ 1 ⊥ ! ?`.
    pub const MELL: Self = Self::MLL_WITH_UNITS.union(Self::EXPONENTIALS);
    /// Full propositional linear logic: every propositional connective.
    pub const LL: Self = Self::MALL.union(Self::EXPONENTIALS);

    /// The named fragments, in the order of [`NAMES`](Self::NAMES): each is
    /// the largest fragment of its name.
    pub const NAMED: [Self; 6] = [
        Self::MLL,
        Self::MLL_WITH_UNITS,
        Self::ADDITIVE,
        Self::MALL,
        Self::MELL,
        Self::LL,
    ];

    /// The names of the [`NAMED`](Self::NAMED) fragments, as
    /// [`name`](Self::name) writes them and [`FromStr`] reads them.
    pub const NAMES: &'static [&'static str] =
        &["MLL", "MLL with units", "ALL", "MALL", "MELL", "LL"];

    /// Returns the fragment with the connective classes of both.
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Returns the connective classes the two fragments share.
    pub const fn intersection(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    /// Returns whether every connective class of `other` is in this fragment,
    /// that is, whether a sequent of `other` is also a sequent of this
    /// fragment.
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    /// Returns whether no connective class is present: atoms only.
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Returns whether `⊗` or `⅋` occurs.
    pub const fn has_multiplicatives(self) -> bool {
        self.contains(Self::MULTIPLICATIVES)
    }

    /// Returns whether `1` or `⊥` occurs.
    pub const fn has_multiplicative_units(self) -> bool {
        self.contains(Self::MULTIPLICATIVE_UNITS)
    }

    /// Returns whether `&` or `⊕` occurs.
    pub const fn has_additives(self) -> bool {
        self.contains(Self::ADDITIVES)
    }

    /// Returns whether `⊤` or `0` occurs.
    pub const fn has_additive_units(self) -> bool {
        self.contains(Self::ADDITIVE_UNITS)
    }

    /// Returns whether `!` or `?` occurs.
    pub const fn has_exponentials(self) -> bool {
        self.contains(Self::EXPONENTIALS)
    }

    /// Returns whether a sequent of this fragment has proof structures:
    /// whether it is unit-free MLL.
    pub const fn has_nets(self) -> bool {
        Self::MLL.contains(self)
    }

    /// Returns the usual name of the smallest named fragment that contains
    /// this one: `MLL`, `MLL with units`, `ALL`, `MALL`, `MELL` or `LL`. The
    /// empty fragment is named `MLL`, and units only count for the name of a
    /// purely multiplicative fragment, where they decide which engine applies.
    pub const fn name(self) -> &'static str {
        let additive = self.has_additives() || self.has_additive_units();
        let multiplicative = self.has_multiplicatives() || self.has_multiplicative_units();
        match (self.has_exponentials(), additive, multiplicative) {
            (true, true, _) => "LL",
            (true, false, _) => "MELL",
            (false, true, true) => "MALL",
            (false, true, false) => "ALL",
            (false, false, _) if self.has_multiplicative_units() => "MLL with units",
            (false, false, _) => "MLL",
        }
    }

    /// Returns the fragment's name in a mode: the classical
    /// [`name`](Self::name), or in intuitionistic mode the intuitionistic
    /// one, `IMLL`, `IMLL with units`, `IALL`, `IMALL`, `IMELL` or `ILL`.
    pub const fn name_in(self, mode: Mode) -> &'static str {
        if !mode.is_intuitionistic() {
            return self.name();
        }
        let additive = self.has_additives() || self.has_additive_units();
        let multiplicative = self.has_multiplicatives() || self.has_multiplicative_units();
        match (self.has_exponentials(), additive, multiplicative) {
            (true, true, _) => "ILL",
            (true, false, _) => "IMELL",
            (false, true, true) => "IMALL",
            (false, true, false) => "IALL",
            (false, false, _) if self.has_multiplicative_units() => "IMLL with units",
            (false, false, _) => "IMLL",
        }
    }
}

impl FromStr for Fragment {
    type Err = Error;

    /// Reads a fragment's name, classical or with the intuitionistic `I`
    /// before it (`MALL`, `IMALL`), as the largest fragment of that name.
    fn from_str(name: &str) -> Result<Self, Error> {
        let classical = name.strip_prefix('I').unwrap_or(name);
        Self::NAMES
            .iter()
            .position(|n| *n == classical)
            .map(|i| Self::NAMED[i])
            .ok_or_else(|| Error::UnknownName {
                what: "fragment",
                name: name.into(),
                known: Self::NAMES,
            })
    }
}

impl BitOr for Fragment {
    type Output = Self;

    /// The union of the connective classes.
    fn bitor(self, other: Self) -> Self {
        self.union(other)
    }
}

impl BitOrAssign for Fragment {
    /// Adds the connective classes of `other`.
    fn bitor_assign(&mut self, other: Self) {
        *self = self.union(other);
    }
}

impl BitAnd for Fragment {
    type Output = Self;

    /// The connective classes both fragments have.
    fn bitand(self, other: Self) -> Self {
        self.intersection(other)
    }
}

impl Display for Fragment {
    /// Writes the fragment's [`name`](Self::name).
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(self.name())
    }
}

impl Kind {
    /// Returns the connective class of this kind, or the empty fragment for a
    /// literal.
    pub const fn fragment(self) -> Fragment {
        use Kind::*;
        match self {
            Atom | DualAtom => Fragment::EMPTY,
            Tensor | Par => Fragment::MULTIPLICATIVES,
            One | Bot => Fragment::MULTIPLICATIVE_UNITS,
            With | Plus => Fragment::ADDITIVES,
            Top | Zero => Fragment::ADDITIVE_UNITS,
            Bang | Quest => Fragment::EXPONENTIALS,
        }
    }
}

impl Sequent {
    /// Returns the smallest fragment the sequent lives in: the connective
    /// classes of the terms its root formulas reach.
    ///
    /// # Examples
    ///
    #[cfg_attr(feature = "parse", doc = "```")]
    #[cfg_attr(not(feature = "parse"), doc = "```ignore")]
    /// use linlog::{Fragment, Sequent};
    ///
    /// let mll: Sequent = "A, A -o B |- B".parse()?;
    /// assert_eq!(mll.fragment(), Fragment::MLL);
    /// assert_eq!(mll.fragment().to_string(), "MLL");
    ///
    /// let ll: Sequent = "|- (A & B) * !C".parse()?;
    /// assert!(ll.fragment().has_additives());
    /// assert!(Fragment::LL.contains(ll.fragment()));
    /// assert!(!Fragment::MALL.contains(ll.fragment()));
    /// assert_eq!(ll.fragment().to_string(), "LL");
    /// # Ok::<(), linlog::Error>(())
    /// ```
    pub fn fragment(&self) -> Fragment {
        let mut fragment = Fragment::EMPTY;
        // A subterm precedes its parent, so one pass from the top visits every
        // reachable term after the term that reaches it.
        let mut reachable = vec![false; self.terms.len()];
        for root in &self.roots {
            reachable[root.index()] = true;
        }
        for (n, term) in self.terms.iter().enumerate().rev() {
            if !reachable[n] {
                continue;
            }
            fragment |= term.kind().fragment();
            for k in term.subterms() {
                reachable[k.index()] = true;
            }
        }
        fragment
    }
}

/// What the user asks of proof search beyond the sequent: the calculus and
/// the structural rules in force. Built from [`CLASSICAL`](Self::CLASSICAL)
/// or [`INTUITIONISTIC`](Self::INTUITIONISTIC) with the builders, or read
/// from its [`name`](Self::name); [`check`](Self::check) refuses a
/// combination no calculus has.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct Mode {
    /// Intuitionistic linear logic, with one formula on the right, instead of
    /// classical one-sided sequents.
    pub(crate) intuitionistic: bool,
    /// Affine logic: weakening is allowed, so a formula may go unused.
    pub(crate) affine: bool,
    /// The Mix rule is allowed: `⊢ Γ` and `⊢ Δ` give `⊢ Γ, Δ`.
    pub(crate) mix: bool,
}

impl Mode {
    /// Classical linear logic without Mix, the default.
    pub const CLASSICAL: Self = Self {
        intuitionistic: false,
        affine: false,
        mix: false,
    };

    /// Intuitionistic linear logic without Mix.
    pub const INTUITIONISTIC: Self = Self {
        intuitionistic: true,
        affine: false,
        mix: false,
    };

    /// The names of the modes a calculus has, as [`name`](Self::name)
    /// writes them and [`FromStr`] reads them.
    pub const NAMES: &'static [&'static str] = &[
        "classical",
        "affine",
        "mix",
        "affine-mix",
        "intuitionistic",
        "intuitionistic-affine",
    ];

    /// Returns whether sequents are intuitionistic, with one formula on
    /// the right, rather than classical and one-sided.
    pub const fn is_intuitionistic(self) -> bool {
        self.intuitionistic
    }

    /// Returns whether weakening is allowed.
    pub const fn is_affine(self) -> bool {
        self.affine
    }

    /// Returns whether the Mix rule is allowed.
    pub const fn has_mix(self) -> bool {
        self.mix
    }

    /// Returns the mode with weakening allowed.
    #[must_use]
    pub const fn with_affine(self) -> Self {
        Self {
            affine: true,
            ..self
        }
    }

    /// Returns the mode with the Mix rule allowed.
    #[must_use]
    pub const fn with_mix(self) -> Self {
        Self { mix: true, ..self }
    }

    /// Returns the mode's name: one of [`NAMES`](Self::NAMES), or
    /// `intuitionistic-mix` and `intuitionistic-affine-mix` for the modes
    /// [`check`](Self::check) refuses.
    pub const fn name(self) -> &'static str {
        let Self {
            intuitionistic,
            affine,
            mix,
        } = self;
        match (intuitionistic, affine, mix) {
            (false, false, false) => "classical",
            (false, true, false) => "affine",
            (false, false, true) => "mix",
            (false, true, true) => "affine-mix",
            (true, false, false) => "intuitionistic",
            (true, true, false) => "intuitionistic-affine",
            (true, false, true) => "intuitionistic-mix",
            (true, true, true) => "intuitionistic-affine-mix",
        }
    }

    /// Refuses a combination no calculus has: intuitionistic with Mix.
    ///
    /// # Errors
    ///
    /// [`Error::IntuitionisticMix`] for an intuitionistic mode with Mix.
    pub const fn check(self) -> Result<(), Error> {
        if self.intuitionistic && self.mix {
            Err(Error::IntuitionisticMix)
        } else {
            Ok(())
        }
    }
}

impl FromStr for Mode {
    type Err = Error;

    /// Reads a mode's [`name`](Self::name), one of
    /// [`NAMES`](Self::NAMES).
    fn from_str(name: &str) -> Result<Self, Error> {
        let mode = match name {
            "classical" => Self::CLASSICAL,
            "affine" => Self::CLASSICAL.with_affine(),
            "mix" => Self::CLASSICAL.with_mix(),
            "affine-mix" => Self::CLASSICAL.with_affine().with_mix(),
            "intuitionistic" => Self::INTUITIONISTIC,
            "intuitionistic-affine" => Self::INTUITIONISTIC.with_affine(),
            _ => {
                return Err(Error::UnknownName {
                    what: "mode",
                    name: name.into(),
                    known: Self::NAMES,
                });
            }
        };
        Ok(mode)
    }
}

impl Display for Mode {
    /// Writes the mode in words: `classical` or `intuitionistic`, then
    /// `affine` and `with Mix` where they apply.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(if self.intuitionistic {
            "intuitionistic"
        } else {
            "classical"
        })?;
        if self.affine {
            f.write_str(" affine")?;
        }
        if self.mix {
            f.write_str(" with Mix")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The named fragments nest as the names say.
    #[test]
    fn named_fragments_nest() {
        use Fragment as F;
        assert!(F::MLL_WITH_UNITS.contains(F::MLL));
        assert!(F::MALL.contains(F::MLL_WITH_UNITS));
        assert!(F::MALL.contains(F::ADDITIVE));
        assert!(F::MELL.contains(F::MLL_WITH_UNITS));
        assert!(F::LL.contains(F::MALL));
        assert!(F::LL.contains(F::MELL));
        assert!(!F::MALL.contains(F::MELL));
        assert!(!F::MELL.contains(F::ADDITIVE));
        assert!(!F::MLL.contains(F::MULTIPLICATIVE_UNITS));
        assert!(F::EMPTY.is_empty());
        assert!(!F::MLL.is_empty());
        assert_eq!(F::MALL & F::MELL, F::MLL_WITH_UNITS);
        assert_eq!(F::ADDITIVE | F::EXPONENTIALS | F::MLL_WITH_UNITS, F::LL);
    }

    /// Every combination of connective classes gets the name of the smallest
    /// named fragment containing it.
    #[test]
    fn names() {
        use Fragment as F;
        for (fragment, name) in [
            (F::EMPTY, "MLL"),
            (F::MLL, "MLL"),
            (F::MULTIPLICATIVE_UNITS, "MLL with units"),
            (F::MLL_WITH_UNITS, "MLL with units"),
            (F::ADDITIVES, "ALL"),
            (F::ADDITIVE_UNITS, "ALL"),
            (F::ADDITIVE, "ALL"),
            (F::MLL | F::ADDITIVES, "MALL"),
            (F::MULTIPLICATIVE_UNITS | F::ADDITIVE_UNITS, "MALL"),
            (F::MALL, "MALL"),
            (F::EXPONENTIALS, "MELL"),
            (F::MLL | F::EXPONENTIALS, "MELL"),
            (F::MELL, "MELL"),
            (F::ADDITIVES | F::EXPONENTIALS, "LL"),
            (F::MELL | F::ADDITIVE_UNITS, "LL"),
            (F::LL, "LL"),
        ] {
            assert_eq!(fragment.to_string(), name, "{fragment:?}");
        }
    }

    /// The intuitionistic names put an `I` in front of the classical ones.
    #[test]
    fn intuitionistic_names() {
        use Fragment as F;
        for (fragment, name) in [
            (F::EMPTY, "IMLL"),
            (F::MLL, "IMLL"),
            (F::MLL_WITH_UNITS, "IMLL with units"),
            (F::ADDITIVE, "IALL"),
            (F::MALL, "IMALL"),
            (F::MELL, "IMELL"),
            (F::LL, "ILL"),
        ] {
            assert_eq!(fragment.name_in(Mode::INTUITIONISTIC), name, "{fragment:?}");
            assert_eq!(
                fragment.name_in(Mode::INTUITIONISTIC.with_affine()),
                name,
                "{fragment:?}"
            );
            assert_eq!(
                fragment.name_in(Mode::CLASSICAL),
                fragment.name(),
                "{fragment:?}"
            );
        }
    }

    /// Every mode a calculus has, and every named fragment, reads back
    /// from its name; an unknown name is refused naming the known ones.
    #[test]
    fn names_read_back() {
        let modes = [
            Mode::CLASSICAL,
            Mode::CLASSICAL.with_affine(),
            Mode::CLASSICAL.with_mix(),
            Mode::CLASSICAL.with_affine().with_mix(),
            Mode::INTUITIONISTIC,
            Mode::INTUITIONISTIC.with_affine(),
        ];
        assert_eq!(modes.map(Mode::name), Mode::NAMES);
        for mode in modes {
            assert_eq!(mode.name().parse::<Mode>().unwrap(), mode);
            assert!(mode.check().is_ok(), "{mode}");
        }
        assert!(Mode::INTUITIONISTIC.with_mix().check().is_err());
        assert!(matches!(
            "lambek".parse::<Mode>(),
            Err(Error::UnknownName {
                what: "mode",
                known: Mode::NAMES,
                ..
            })
        ));
        for (fragment, name) in Fragment::NAMED.into_iter().zip(Fragment::NAMES) {
            assert_eq!(fragment.name(), *name);
            assert_eq!(name.parse::<Fragment>().unwrap(), fragment);
            assert_eq!(format!("I{name}").parse::<Fragment>().unwrap(), fragment);
        }
        assert!("MLLL".parse::<Fragment>().is_err());
    }

    /// Each kind belongs to the connective class of its symbol.
    #[test]
    fn kinds_have_classes() {
        use Fragment as F;
        use Kind::*;
        for (kind, fragment) in [
            (Atom, F::EMPTY),
            (DualAtom, F::EMPTY),
            (Tensor, F::MULTIPLICATIVES),
            (Par, F::MULTIPLICATIVES),
            (One, F::MULTIPLICATIVE_UNITS),
            (Bot, F::MULTIPLICATIVE_UNITS),
            (With, F::ADDITIVES),
            (Plus, F::ADDITIVES),
            (Top, F::ADDITIVE_UNITS),
            (Zero, F::ADDITIVE_UNITS),
            (Bang, F::EXPONENTIALS),
            (Quest, F::EXPONENTIALS),
        ] {
            assert_eq!(kind.fragment(), fragment, "{kind:?}");
            assert_eq!(kind.dual().fragment(), fragment, "{kind:?}");
        }
    }

    /// Detection sees exactly the connectives of a sequent, dualised
    /// hypotheses included.
    #[cfg(feature = "parse")]
    #[test]
    fn detection() {
        use Fragment as F;
        for (input, fragment) in [
            ("|-", F::EMPTY),
            ("A |- A", F::EMPTY),
            ("A, A -o B |- B", F::MLL),
            ("A * B |- A par B", F::MLL),
            ("|- 1", F::MULTIPLICATIVE_UNITS),
            ("1 |-", F::MULTIPLICATIVE_UNITS),
            ("A * 1 |- A", F::MLL_WITH_UNITS),
            ("A & B |- A", F::ADDITIVES),
            ("|- top", F::ADDITIVE_UNITS),
            ("0 |- A", F::ADDITIVE_UNITS),
            ("A & B |- A + 0", F::ADDITIVE),
            ("A & B |- A * B", F::MLL | F::ADDITIVES),
            ("A * B, C par D |- A & B, C + D, 0, top, 1, bot", F::MALL),
            ("!A |- A", F::EXPONENTIALS),
            ("!A, !(A -o B) |- !B", F::MLL | F::EXPONENTIALS),
            ("!A, !(A -o B) |- !B, 1", F::MELL),
            ("!(A & B) |- ?(A + B)", F::ADDITIVES | F::EXPONENTIALS),
            ("!(A & B) * 1 |- ?(A + B), top", F::LL),
        ] {
            let s: Sequent = input.parse().unwrap();
            assert_eq!(s.fragment(), fragment, "{input:?}");
        }
    }

    /// Terms no root reaches do not count.
    #[test]
    fn detection_ignores_unreachable_terms() {
        use crate::sequents::{Term, TermId};
        let s = Sequent {
            terms: vec![Term::One, Term::Bang(TermId::new(0)), Term::Top],
            roots: vec![TermId::new(1)],
            atoms: vec![],
            antecedents: None,
        };
        assert_eq!(
            s.fragment(),
            Fragment::MULTIPLICATIVE_UNITS | Fragment::EXPONENTIALS
        );
    }

    /// The modes print in words.
    #[test]
    fn mode_names() {
        assert_eq!(Mode::default(), Mode::CLASSICAL);
        assert_eq!(Mode::CLASSICAL.to_string(), "classical");
        assert_eq!(
            Mode::INTUITIONISTIC.with_affine().to_string(),
            "intuitionistic affine"
        );
        assert_eq!(
            Mode::CLASSICAL.with_affine().with_mix().to_string(),
            "classical affine with Mix"
        );
        assert_eq!(
            Mode::INTUITIONISTIC.with_mix().to_string(),
            "intuitionistic with Mix"
        );
    }
}
