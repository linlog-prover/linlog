// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use crate::Error;
use crate::occurrences::Side;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::str::FromStr;

/// A rule of the one-sided standard sequent calculus of linear logic, as
/// an inference of a derivation applies it. The type is closed: a new rule
/// is a new release of the calculus.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Rule {
    /// `ax`
    Ax,
    /// `⊗`
    Tensor,
    /// `⅋`
    Par,
    /// `1`
    One,
    /// `⊥`
    Bot,
    /// `&`
    With,
    /// `⊕₁`, the left introduction of `⊕`.
    PlusLeft,
    /// `⊕₂`, the right introduction of `⊕`.
    PlusRight,
    /// `⊤`
    Top,
    /// `!`, promotion.
    Promotion,
    /// `?d`, dereliction.
    Dereliction,
    /// `?c`, contraction.
    Contraction,
    /// `?w`, weakening of a `?` formula.
    Weakening,
    /// `mix`
    Mix,
    /// `wk`, weakening of a formula that is not a `?`, in affine mode.
    AffineWeakening,
    /// `cut`
    Cut,
    /// An open goal of a proof in progress: a leaf without a rule, which
    /// only the derivation of an interactive state contains.
    Open,
}

impl Rule {
    /// Every rule, in the order of declaration.
    pub const ALL: [Self; 17] = {
        use Rule::*;
        [
            Ax,
            Tensor,
            Par,
            One,
            Bot,
            With,
            PlusLeft,
            PlusRight,
            Top,
            Promotion,
            Dereliction,
            Contraction,
            Weakening,
            Mix,
            AffineWeakening,
            Cut,
            Open,
        ]
    };

    /// Returns the rule's usual spelling.
    pub const fn name(self) -> &'static str {
        use Rule::*;
        match self {
            Ax => "ax",
            Tensor => "⊗",
            Par => "⅋",
            One => "1",
            Bot => "⊥",
            With => "&",
            PlusLeft => "⊕₁",
            PlusRight => "⊕₂",
            Top => "⊤",
            Promotion => "!",
            Dereliction => "?d",
            Contraction => "?c",
            Weakening => "?w",
            Mix => "mix",
            AffineWeakening => "wk",
            Cut => "cut",
            Open => "open",
        }
    }

    /// Returns whether the rule is structural: a weakening or a
    /// contraction, which a compact view draws a run of as one inference.
    pub const fn is_structural(self) -> bool {
        matches!(
            self,
            Self::Contraction | Self::Weakening | Self::AffineWeakening
        )
    }

    /// Returns how many premises the rule has: none for the axiom, `1`,
    /// `⊤` and an open goal, two for `⊗`, `&`, Mix and the cut, one
    /// otherwise.
    pub const fn premises(self) -> u8 {
        use Rule::*;
        match self {
            Ax | One | Top | Open => 0,
            Tensor | With | Mix | Cut => 2,
            Par | Bot | PlusLeft | PlusRight | Promotion | Dereliction | Contraction
            | Weakening | AffineWeakening => 1,
        }
    }

    /// Returns whether the rule acts on one formula of its conclusion,
    /// its principal one: every rule but the axiom, Mix, the cut and an
    /// open goal.
    pub const fn has_principal(self) -> bool {
        !matches!(self, Self::Ax | Self::Mix | Self::Cut | Self::Open)
    }

    /// Returns the rule as a two-sided derivation names it, applied to a
    /// formula on `side` of `⊢`: `⊗` on a hypothesis is `⊸L`, on the goal
    /// `⊗R`, and so on. The axiom, Mix, affine weakening, the cut and an
    /// open goal keep their names, with no side.
    pub const fn on(self, side: Side) -> Named {
        Named::new(self, Some(side))
    }
}

impl Display for Rule {
    /// Writes the rule's [`name`](Self::name).
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(self.name())
    }
}

/// A rule as a derivation names it: the one-sided rule and, in a
/// two-sided derivation, the side of `⊢` its principal formula stands on
/// (`⊸L` is `⊗` on the input side, `⊗R` on the output side).
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Named {
    /// The one-sided rule.
    pub rule: Rule,
    /// The side of `⊢` the principal formula stands on, in a two-sided
    /// derivation; `None` one-sided, and for a rule whose name has no side
    /// (the axiom, Mix, affine weakening, the cut, an open goal).
    pub side: Option<Side>,
    /// In a two-sided derivation of the Lambek calculus, the division a
    /// rule of an implication acts on (`\L`, `\R`, `/L`, `/R`); `None`
    /// for every other rule and in every other derivation, where the
    /// implication is `⊸`.
    pub division: Option<Division>,
}

/// One of the two divisions of the Lambek calculus, the implications of
/// its ordered reading ([`Reading::of_mode`](crate::Reading::of_mode)).
/// The type is closed, as the calculus is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Division {
    /// `A \ B`, whose antecedent `A` is written first: `A, A \ B ⊢ B`.
    Under,
    /// `B / A`, whose antecedent `A` is written second: `B / A, A ⊢ B`.
    Over,
}

impl Named {
    /// Every named rule: the one-sided ones in the order of
    /// [`Rule::ALL`], an open goal last, and the two-sided ones between.
    pub const ALL: [Self; 35] = {
        use Rule::*;
        use Side::{Input, Output};
        [
            one_sided(Ax),
            one_sided(Tensor),
            one_sided(Par),
            one_sided(One),
            one_sided(Bot),
            one_sided(With),
            one_sided(PlusLeft),
            one_sided(PlusRight),
            one_sided(Top),
            one_sided(Promotion),
            one_sided(Dereliction),
            one_sided(Contraction),
            one_sided(Weakening),
            one_sided(Mix),
            one_sided(AffineWeakening),
            one_sided(Cut),
            two_sided(Tensor, Input),
            two_sided(Par, Output),
            two_sided(Par, Input),
            two_sided(Tensor, Output),
            two_sided(PlusLeft, Input),
            two_sided(PlusRight, Input),
            two_sided(With, Output),
            two_sided(With, Input),
            two_sided(PlusLeft, Output),
            two_sided(PlusRight, Output),
            two_sided(Bot, Input),
            two_sided(One, Output),
            two_sided(Top, Input),
            two_sided(Top, Output),
            two_sided(Dereliction, Input),
            two_sided(Promotion, Output),
            two_sided(Contraction, Input),
            two_sided(Weakening, Input),
            one_sided(Open),
        ]
    };

    /// Returns the rule named on `side`: one-sided for `None`, and for a
    /// rule whose name has no side. `1`, `⊥` and the exponential rules have
    /// one two-sided name each, which either side gives.
    pub const fn new(rule: Rule, side: Option<Side>) -> Self {
        use Rule::*;
        use Side::{Input, Output};
        let side = match (rule, side) {
            (Ax | Mix | AffineWeakening | Cut | Open, _) | (_, None) => None,
            (Bot | Dereliction | Contraction | Weakening, Some(_)) => Some(Input),
            (One | Promotion, Some(_)) => Some(Output),
            (_, side) => side,
        };
        Self {
            rule,
            side,
            division: None,
        }
    }

    /// The rules of the two divisions, `\L`, `\R`, `/L` and `/R`, which
    /// follow [`ALL`](Self::ALL) in the label tables.
    pub(crate) const DIVISIONS: [Self; 4] = [
        divided(Rule::Tensor, Side::Input, Division::Under),
        divided(Rule::Par, Side::Output, Division::Under),
        divided(Rule::Tensor, Side::Input, Division::Over),
        divided(Rule::Par, Side::Output, Division::Over),
    ];

    /// Returns the rule `rule` applied to `o` as a derivation under
    /// `reading` names it, or one-sided without a reading: on the side
    /// of `⊢` the formula stands on, and in the Lambek calculus's ordered
    /// reading with the division it is.
    pub(crate) fn applied(
        rule: Rule,
        reading: Option<&crate::Reading<'_>>,
        o: crate::OccId,
    ) -> Self {
        use crate::occurrences::IllConnective as C;
        let named = Self::new(rule, reading.map(|reading| reading.position(o)));
        let division = match reading.map(|reading| reading.connective(o)) {
            Some(C::Under) => Division::Under,
            Some(C::Over) => Division::Over,
            _ => return named,
        };
        match (named.rule, named.side) {
            (Rule::Tensor, Some(Side::Input)) | (Rule::Par, Some(Side::Output)) => Self {
                division: Some(division),
                ..named
            },
            _ => named,
        }
    }

    /// Returns the rule at `index` of the label tables: of
    /// [`ALL`](Self::ALL), then of the divisions.
    pub(crate) const fn at(index: usize) -> Self {
        if index < Self::ALL.len() {
            Self::ALL[index]
        } else {
            Self::DIVISIONS[index - Self::ALL.len()]
        }
    }

    /// Returns the name: the rule's, or two-sided `⊸L`, `⊸R`, `⊗L`, `⊗R`,
    /// `&L₁`, `&L₂`, `&R`, `⊕L`, `⊕R₁`, `⊕R₂`, `1L`, `1R`, `0L`, `⊤R`, `!L`,
    /// `!R`, `!c` or `!w`, and in the Lambek calculus `\L`, `\R`, `/L` or
    /// `/R`.
    pub const fn name(self) -> &'static str {
        use Rule::*;
        use Side::{Input, Output};
        match (self.rule, self.side, self.division) {
            (Tensor, Some(Input), Some(Division::Under)) => return "\\L",
            (Par, Some(Output), Some(Division::Under)) => return "\\R",
            (Tensor, Some(Input), Some(Division::Over)) => return "/L",
            (Par, Some(Output), Some(Division::Over)) => return "/R",
            _ => {}
        }
        match (self.rule, self.side) {
            (Tensor, Some(Input)) => "⊸L",
            (Par, Some(Output)) => "⊸R",
            (Par, Some(Input)) => "⊗L",
            (Tensor, Some(Output)) => "⊗R",
            (PlusLeft, Some(Input)) => "&L₁",
            (PlusRight, Some(Input)) => "&L₂",
            (With, Some(Output)) => "&R",
            (With, Some(Input)) => "⊕L",
            (PlusLeft, Some(Output)) => "⊕R₁",
            (PlusRight, Some(Output)) => "⊕R₂",
            (Bot, Some(_)) => "1L",
            (One, Some(_)) => "1R",
            (Top, Some(Input)) => "0L",
            (Top, Some(Output)) => "⊤R",
            (Dereliction, Some(_)) => "!L",
            (Promotion, Some(_)) => "!R",
            (Contraction, Some(_)) => "!c",
            (Weakening, Some(_)) => "!w",
            (rule, _) => rule.name(),
        }
    }

    /// Returns whether the rule is structural: see [`Rule::is_structural`].
    pub const fn is_structural(self) -> bool {
        self.rule.is_structural()
    }

    /// Returns the rule's place in [`ALL`](Self::ALL), or after it among
    /// the divisions, by which the label tables are indexed.
    pub(crate) const fn index(self) -> usize {
        if let Some(division) = self.division {
            let left = matches!(self.rule, Rule::Tensor);
            return Self::ALL.len() + 2 * (division as usize) + if left { 0 } else { 1 };
        }
        let mut i = 0;
        while i < Self::ALL.len() {
            let named = Self::ALL[i];
            if named.rule as u8 == self.rule as u8 && same_side(named.side, self.side) {
                return i;
            }
            i += 1;
        }
        // Every value `new` makes is in the list; a literal with an unused
        // side reads as the one-sided rule.
        self.rule as usize
    }
}

/// Returns the rule named one-sided, for [`Named::ALL`].
const fn one_sided(rule: Rule) -> Named {
    Named {
        rule,
        side: None,
        division: None,
    }
}

/// Returns the rule named on `side`, for [`Named::ALL`].
const fn two_sided(rule: Rule, side: Side) -> Named {
    Named {
        rule,
        side: Some(side),
        division: None,
    }
}

/// Returns the rule of a division named on `side`, for
/// [`Named::DIVISIONS`].
const fn divided(rule: Rule, side: Side, division: Division) -> Named {
    Named {
        rule,
        side: Some(side),
        division: Some(division),
    }
}

/// Returns whether two sides are equal, as a `const fn` can compare them.
const fn same_side(a: Option<Side>, b: Option<Side>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => a as u8 == b as u8,
        _ => false,
    }
}

impl From<Rule> for Named {
    /// The rule named one-sided.
    fn from(rule: Rule) -> Self {
        one_sided(rule)
    }
}

impl Display for Named {
    /// Writes the rule's [`name`](Self::name).
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(self.name())
    }
}

impl FromStr for Named {
    type Err = Error;

    /// Reads a rule from its [`name`](Self::name) or an ASCII spelling of
    /// it: `*` for `⊗`, `par` or `|` for `⅋`, `+1` and `+2` for `⊕₁` and
    /// `⊕₂`, `bot` and `top` for `⊥` and `⊤`, `-oL` for `⊸L`, `*L` for
    /// `⊗L`, `&L1` for `&L₁`, `+L` for `⊕L`, `+R1` for `⊕R₁`, `topR` for
    /// `⊤R`, and so on.
    fn from_str(text: &str) -> Result<Self, Error> {
        use Rule::*;
        use Side::{Input, Output};
        if let Some(&named) = Self::DIVISIONS.iter().find(|named| named.name() == text) {
            return Ok(named);
        }
        let (rule, side) = match text {
            "ax" => (Ax, None),
            "⊗" | "*" => (Tensor, None),
            "⅋" | "par" | "|" => (Par, None),
            "1" => (One, None),
            "⊥" | "bot" => (Bot, None),
            "&" => (With, None),
            "⊕₁" | "+1" => (PlusLeft, None),
            "⊕₂" | "+2" => (PlusRight, None),
            "⊤" | "top" => (Top, None),
            "!" => (Promotion, None),
            "?d" => (Dereliction, None),
            "?c" => (Contraction, None),
            "?w" => (Weakening, None),
            "mix" => (Mix, None),
            "wk" => (AffineWeakening, None),
            "cut" => (Cut, None),
            "⊸L" | "-oL" => (Tensor, Some(Input)),
            "⊸R" | "-oR" => (Par, Some(Output)),
            "⊗L" | "*L" => (Par, Some(Input)),
            "⊗R" | "*R" => (Tensor, Some(Output)),
            "&L₁" | "&L1" => (PlusLeft, Some(Input)),
            "&L₂" | "&L2" => (PlusRight, Some(Input)),
            "&R" => (With, Some(Output)),
            "⊕L" | "+L" => (With, Some(Input)),
            "⊕R₁" | "+R1" => (PlusLeft, Some(Output)),
            "⊕R₂" | "+R2" => (PlusRight, Some(Output)),
            "1L" => (Bot, Some(Input)),
            "1R" => (One, Some(Output)),
            "0L" => (Top, Some(Input)),
            "⊤R" | "topR" => (Top, Some(Output)),
            "!L" => (Dereliction, Some(Input)),
            "!R" => (Promotion, Some(Output)),
            "!c" => (Contraction, Some(Input)),
            "!w" => (Weakening, Some(Input)),
            "open" => (Open, None),
            _ => {
                return Err(Error::UnknownName {
                    what: "rule",
                    name: text.into(),
                    known: KNOWN,
                });
            }
        };
        Ok(Self::new(rule, side))
    }
}

/// The names of [`Named::ALL`], in its order.
#[cfg(test)]
const NAMES: &[&str] = KNOWN.split_at(Named::ALL.len()).0;

/// The names of [`Named::ALL`], in its order, then those of the
/// divisions.
const KNOWN: &[&str] = &[
    "ax", "⊗", "⅋", "1", "⊥", "&", "⊕₁", "⊕₂", "⊤", "!", "?d", "?c", "?w", "mix", "wk", "cut",
    "⊸L", "⊸R", "⊗L", "⊗R", "&L₁", "&L₂", "&R", "⊕L", "⊕R₁", "⊕R₂", "1L", "1R", "0L", "⊤R", "!L",
    "!R", "!c", "!w", "open", "\\L", "\\R", "/L", "/R",
];

#[cfg(test)]
mod tests {
    use super::*;

    /// Every named rule reads back from its name, which [`NAMES`] lists in
    /// the order of [`Named::ALL`], and sits at its index; `new` gives
    /// every side of a rule a name of the list.
    #[test]
    fn names_round_trip() {
        for (i, named) in Named::ALL.into_iter().enumerate() {
            assert_eq!(named.name(), NAMES[i]);
            assert_eq!(named.name().parse::<Named>().unwrap(), named, "{named:?}");
            assert_eq!(named.index(), i);
        }
        for rule in Rule::ALL {
            for side in [None, Some(Side::Input), Some(Side::Output)] {
                let named = Named::new(rule, side);
                assert_eq!(Named::ALL[named.index()], named, "{named:?}");
            }
        }
        for (i, named) in Named::DIVISIONS.into_iter().enumerate() {
            let index = Named::ALL.len() + i;
            assert_eq!(named.name(), KNOWN[index]);
            assert_eq!(named.name().parse::<Named>().unwrap(), named, "{named:?}");
            assert_eq!((named.index(), Named::at(index)), (index, named));
        }
        assert!("?x".parse::<Named>().is_err());
    }
}
