// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The intuitionistic reading of a sequent: which occurrences are
//! hypotheses and which one is the goal, so that a one-sided sequent in
//! negation normal form is read as a two-sided sequent `Γ ⊢ A` of
//! intuitionistic linear logic without a second representation.
//!
//! An intuitionistic formula stands in *output position* when it is the goal
//! or the antecedent of a hypothesis: it is built from `⊗ ⊕ & ! 1 ⊤ 0`,
//! atoms `a`, and `A ⊸ B`, which the one-sided arena stores as the par
//! `A⊥ ⅋ B`. In *input position*, as a hypothesis or as the antecedent of
//! the goal, a formula is the negation of such a formula: `⅋ & ⊕ ? ⊥ 0 ⊤`,
//! negated atoms `~a`, and `A ⊗ B⊥` for a hypothesis `A ⊸ B`. The position
//! flips at the antecedent of an implication and nowhere else, which is
//! Lamarche's polarization of intuitionistic proof structures. An
//! intuitionistic sequent is a one-sided sequent with exactly one
//! output-shaped root, the goal, and every other root input-shaped.

use super::{Forest, OccId};
use crate::errors::{Described, Owner, Subject};
use crate::sequents::{Kind, Visit, Walk};
use std::borrow::Cow;
use std::fmt::{Display, Formatter, Result as FmtResult};

/// The side of `⊢` an occurrence stands on under the intuitionistic reading
/// of its sequent: a hypothesis, or a subformula that behaves like one, is
/// input; the goal, or a subformula that behaves like it, is output.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Side {
    /// A hypothesis, or the antecedent of a formula in output position.
    Input,
    /// The goal, or the antecedent of a formula in input position.
    Output,
}

impl std::ops::Not for Side {
    type Output = Self;

    /// The other position.
    fn not(self) -> Self {
        match self {
            Side::Input => Side::Output,
            Side::Output => Side::Input,
        }
    }
}

/// Why a sequent has no intuitionistic reading.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ShapeError {
    /// No root formula can be the goal: with the sides known, the formula
    /// right of `⊢` cannot; without them, every one is input-shaped only, or
    /// the sequent is empty.
    NoGoal,
    /// Two root formulas can only be goals (the sides not known).
    SeveralGoals(OccId, OccId),
    /// A subformula is neither an intuitionistic formula nor the negation of
    /// one, in any position.
    Formula(OccId),
    /// The sequent has another number of formulas right of `⊢` than one.
    Succedents {
        /// How many formulas stand right of `⊢`.
        count: usize,
    },
    /// A formula written left of `⊢` (the first is 0) cannot be a
    /// hypothesis.
    Hypothesis {
        /// Its index among the formulas left of `⊢`.
        index: usize,
    },
    /// Two root formulas can each be the goal, every other root reading as
    /// a hypothesis either way, and the sequent does not say which was
    /// written right of `⊢`.
    Undetermined {
        /// The first root that can be the goal.
        first: OccId,
        /// The second.
        second: OccId,
    },
}

impl Display for ShapeError {
    /// Writes the reason with occurrence ids, such as `subformula 3 has no
    /// intuitionistic reading`.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.write(f, None)
    }
}

impl ShapeError {
    /// Returns the error for display with formulas instead of occurrence
    /// ids, read from `owner`, the forest the reading was attempted on.
    pub fn describe<'a>(&'a self, owner: &'a impl Owner) -> Described<'a> {
        Described::new(Subject::Shape(self), owner)
    }

    /// Writes the reason as [`Display`] does, with formulas instead of ids
    /// when a forest is given.
    pub(crate) fn write(&self, f: &mut Formatter<'_>, forest: Option<&Forest>) -> FmtResult {
        let occurrence = |f: &mut Formatter<'_>, o: OccId| match forest {
            Some(forest) => write!(f, "{}", forest.formula(o)),
            None => write!(f, "{}", o.get()),
        };
        match self {
            ShapeError::NoGoal => f.write_str(
                "no formula can be the goal: an intuitionistic sequent has exactly one formula on the right of ⊢",
            ),
            ShapeError::Succedents { count } => {
                write!(
                    f,
                    "an intuitionistic sequent has exactly one formula right of ⊢, not {count}"
                )?;
                if *count > 1 {
                    f.write_str(": write the hypotheses left of ⊢")?;
                }
                Ok(())
            }
            ShapeError::Hypothesis { index } => write!(
                f,
                "formula {} left of ⊢ cannot be a hypothesis of an intuitionistic sequent",
                index + 1
            ),
            ShapeError::Undetermined { first, second } => {
                f.write_str(if forest.is_some() {
                    "both "
                } else {
                    "both formula "
                })?;
                occurrence(f, *first)?;
                f.write_str(" and ")?;
                occurrence(f, *second)?;
                f.write_str(
                    " can be the goal, and the sequent does not say which stands right of ⊢: give its sides",
                )
            }
            ShapeError::SeveralGoals(a, b) => {
                f.write_str(if forest.is_some() {
                    "both "
                } else {
                    "both formula "
                })?;
                occurrence(f, *a)?;
                f.write_str(" and ")?;
                occurrence(f, *b)?;
                f.write_str(" can only be the goal, but an intuitionistic sequent has one")
            }
            ShapeError::Formula(o) => {
                f.write_str(if forest.is_some() {
                    "the subformula "
                } else {
                    "subformula "
                })?;
                occurrence(f, *o)?;
                if let Some(forest) = forest
                    && forest.root(*o) != *o
                {
                    write!(f, " (in {})", forest.formula(forest.root(*o)))?;
                }
                f.write_str(
                    " is neither an intuitionistic formula nor the negation of one (⅋ only as A ⊸ B, that is ~A ⅋ B, and ? only under a negation)",
                )
            }
        }
    }
}

impl std::error::Error for ShapeError {}

/// An occurrence's possible positions, as two bits.
const IN: u8 = 1;
/// See [`IN`].
const OUT: u8 = 2;

/// The intuitionistic reading of a sequent: the position of every
/// occurrence of its forest and which root is the goal. It prints the
/// sequent two-sided, `Γ ⊢ A`, with intuitionistic formulas.
///
/// The reading is what was written and guesses nothing:
///
/// - **An implication's antecedent is its left factor**, where the
///   lowering of `A ⊸ B` puts it: `⅋` in output position is `A ⊸ B` for
///   `A⊥ ⅋ B`, and `⊗` in input position `A ⊗ B⊥`. `b ⅋ ~a` has no
///   reading in output position.
/// - **With the sides known** ([`Sequent::antecedents`]), the formulas
///   right of `⊢` must be exactly one, the goal
///   ([`ShapeError::Succedents`]), which must read in output position
///   ([`ShapeError::NoGoal`]), and every formula left of it a hypothesis
///   in input position ([`ShapeError::Hypothesis`]).
/// - **With the sides unknown**, the goal is the one root that can be it
///   with every other root a hypothesis; where two can, which only
///   formulas built from `⊤` and `0` allow (`⊢ 0, ⊤`), the reading refuses
///   ([`ShapeError::Undetermined`]) and asks for the sides. `⊢ ⊤, a` has
///   one reading, `0 ⊢ a`.
///
/// [`Sequent::antecedents`]: crate::Sequent::antecedents
///
/// # Examples
///
#[cfg_attr(feature = "parse", doc = "```")]
#[cfg_attr(not(feature = "parse"), doc = "```ignore")]
/// use linlog::{Forest, Side, Reading, Sequent};
///
/// // ⊢ ~A, A ⊗ ~B, B
/// let sequent: Sequent = "A, A -o B |- B".parse()?;
/// let forest = Forest::new(&sequent)?;
/// let reading = Reading::new(&forest)?;
/// assert_eq!(reading.to_string(), "A, A ⊸ B ⊢ B");
/// assert_eq!(reading.goal(), forest.roots()[2]);
/// let hypothesis = forest.roots()[1];
/// assert_eq!(reading.position(hypothesis), Side::Input);
/// assert_eq!(reading.position(forest.left(hypothesis).unwrap()), Side::Output);
/// assert_eq!(reading.formula(hypothesis).to_string(), "A ⊸ B");
///
/// let classical: Sequent = "|- A par B".parse()?;
/// assert!(Reading::new(&Forest::new(&classical)?).is_err());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug)]
pub struct Reading<'a> {
    /// The forest read.
    forest: &'a Forest,
    /// Per occurrence, its position: the reading's own, or a session's
    /// kept beside its forest.
    position: Cow<'a, [Side]>,
    /// The goal.
    goal: OccId,
}

impl<'a> Reading<'a> {
    /// Reads the forest as an intuitionistic sequent, or says why it is
    /// none: a subformula with no reading in any position, a formula on a
    /// side of `⊢` it cannot stand on, or, with the sides unknown, no root
    /// that can be the goal or two that can.
    pub fn new(forest: &'a Forest) -> Result<Self, ShapeError> {
        use Kind::*;
        // Which positions each occurrence can take, children before parents.
        let mut can = vec![0u8; forest.len()];
        for o in forest.ids().rev() {
            let child = |k: Option<OccId>| k.map_or(0, |k| can[k.index()]);
            let (l, r) = (child(forest.left(o)), child(forest.right(o)));
            let (l_in, l_out, r_in, r_out) = (l & IN != 0, l & OUT != 0, r & IN != 0, r & OUT != 0);
            let both = |i: bool, u: bool| (u8::from(i) * IN) | (u8::from(u) * OUT);
            can[o.index()] = match forest.kind(o) {
                Atom | One | Bang => both(false, l_out || forest.kind(o) != Bang),
                DualAtom | Bot | Quest => both(l_in || forest.kind(o) != Quest, false),
                Top | Zero => IN | OUT,
                With | Plus => both(l_in && r_in, l_out && r_out),
                // In input position `A ⊗ B⊥` is `A ⊸ B`, in output position
                // `A⊥ ⅋ B`: the left factor is the antecedent.
                Tensor => both(l_out && r_in, l_out && r_out),
                Par => both(l_in && r_in, l_in && r_out),
            };
            if can[o.index()] == 0 {
                return Err(ShapeError::Formula(o));
            }
        }

        let roots = forest.roots();
        let goal = match forest.sequent().antecedents() {
            Some(k) => {
                let (left, right) = roots.split_at(k as usize);
                let &[goal] = right else {
                    return Err(ShapeError::Succedents { count: right.len() });
                };
                if let Some(index) = left.iter().position(|&o| can[o.index()] & IN == 0) {
                    return Err(ShapeError::Hypothesis { index });
                }
                if can[goal.index()] & OUT == 0 {
                    return Err(ShapeError::NoGoal);
                }
                goal
            }
            None => {
                // A root that cannot be input is the goal; else the one
                // root that can be output, every other one being input.
                let mut only_goals = roots.iter().copied().filter(|&o| can[o.index()] == OUT);
                match (only_goals.next(), only_goals.next()) {
                    (Some(a), Some(b)) => return Err(ShapeError::SeveralGoals(a, b)),
                    (Some(goal), None) => goal,
                    (None, _) => {
                        let mut goals =
                            roots.iter().copied().filter(|&o| can[o.index()] & OUT != 0);
                        match (goals.next(), goals.next()) {
                            (Some(first), Some(second)) => {
                                return Err(ShapeError::Undetermined { first, second });
                            }
                            (Some(goal), None) => goal,
                            (None, _) => return Err(ShapeError::NoGoal),
                        }
                    }
                }
            }
        };

        // The positions, parents before children.
        let mut position = vec![Side::Input; forest.len()].into_boxed_slice();
        position[goal.index()] = Side::Output;
        for o in forest.ids() {
            let p = position[o.index()];
            let Some((l, r)) = forest.left(o).zip(forest.right(o)) else {
                if let Some(l) = forest.left(o) {
                    position[l.index()] = p;
                }
                continue;
            };
            // An implication's antecedent, its left factor, flips.
            let implication = matches!(
                (forest.kind(o), p),
                (Tensor, Side::Input) | (Par, Side::Output)
            );
            position[l.index()] = if implication { !p } else { p };
            position[r.index()] = p;
        }
        Ok(Self {
            forest,
            position: Cow::Owned(position.into_vec()),
            goal,
        })
    }

    /// Returns the positions and the goal, for an owner of the forest to
    /// keep beside it.
    pub(crate) fn into_parts(self) -> (Box<[Side]>, OccId) {
        (self.position.into_owned().into_boxed_slice(), self.goal)
    }

    /// Returns the reading of `forest` from the positions and the goal
    /// [`into_parts`](Self::into_parts) gave, in constant time.
    pub(crate) const fn of_parts(forest: &'a Forest, position: &'a [Side], goal: OccId) -> Self {
        Self {
            forest,
            position: Cow::Borrowed(position),
            goal,
        }
    }

    /// Returns the forest read.
    pub fn forest(&self) -> &'a Forest {
        self.forest
    }

    /// Returns the position of an occurrence.
    pub fn position(&self, o: OccId) -> Side {
        self.position[o.index()]
    }

    /// Returns the goal: the one root in output position.
    pub fn goal(&self) -> OccId {
        self.goal
    }

    /// Returns the hypotheses: every root but the goal, in the order
    /// written.
    pub fn hypotheses(&self) -> impl Iterator<Item = OccId> + '_ {
        self.forest
            .roots()
            .iter()
            .copied()
            .filter(move |&o| o != self.goal)
    }

    /// Returns the antecedent and the consequent of an implication `A ⊸ B`:
    /// a `⅋` in output position or a `⊗` in input position, whose antecedent
    /// is the left factor. `None` for any other occurrence.
    pub fn implication(&self, o: OccId) -> Option<(OccId, OccId)> {
        match (self.forest.kind(o), self.position(o)) {
            (Kind::Par, Side::Output) | (Kind::Tensor, Side::Input) => {
                Some((self.forest.left(o)?, self.forest.right(o)?))
            }
            _ => None,
        }
    }

    /// Counts the occurrences in output position among `ids`: an
    /// intuitionistic sequent has exactly one.
    pub fn outputs(&self, ids: impl IntoIterator<Item = OccId>) -> usize {
        ids.into_iter()
            .filter(|&o| self.position(o) == Side::Output)
            .count()
    }

    /// Returns the intuitionistic formula at `o`, as a value that prints it
    /// with `⊸`, `1` for `⊥`, `⊤` for an input `0` and so on, according to
    /// the position of `o`.
    pub fn formula(&self, o: OccId) -> IllFormula<'_> {
        IllFormula {
            reading: self,
            id: o,
        }
    }

    /// Returns the first and the second subformula of the intuitionistic
    /// formula at `o`, each if there is one, in the order they are written:
    /// of an implication the antecedent, then the consequent.
    pub(crate) fn operands(&self, o: OccId) -> (Option<OccId>, Option<OccId>) {
        match self.implication(o) {
            Some((antecedent, consequent)) => (Some(antecedent), Some(consequent)),
            None => (self.forest.left(o), self.forest.right(o)),
        }
    }

    /// Writes the formula at `o`, in brackets if it is binary and `brackets`
    /// is set.
    fn fmt_formula(&self, o: OccId, f: &mut Formatter<'_>, brackets: bool) -> FmtResult {
        use Kind::*;
        let forest = self.forest;
        for visit in Walk::new(o, brackets, |o| self.operands(o)) {
            match visit {
                Visit::Enter(o, nested) => match (forest.kind(o), self.position(o)) {
                    (Atom | DualAtom, _) => {
                        f.write_str(forest.sequent().atom_name(forest.atom(o).unwrap()))?;
                    }
                    (One | Bot, _) => f.write_str("1")?,
                    (Top, Side::Output) | (Zero, Side::Input) => f.write_str("⊤")?,
                    (Zero, Side::Output) | (Top, Side::Input) => f.write_str("0")?,
                    (Tensor | Par | With | Plus, _) if nested => f.write_str("(")?,
                    (Tensor | Par | With | Plus, _) => {}
                    (Bang | Quest, _) => f.write_str("!")?,
                },
                Visit::Between(o) => f.write_str(match (forest.kind(o), self.position(o)) {
                    (Tensor, Side::Output) | (Par, Side::Input) => " ⊗ ",
                    (Tensor, Side::Input) | (Par, Side::Output) => " ⊸ ",
                    (With, Side::Output) | (Plus, Side::Input) => " & ",
                    _ => " ⊕ ",
                })?,
                Visit::Exit(o, nested) => {
                    if nested && forest.kind(o).arity() == 2 {
                        f.write_str(")")?;
                    }
                }
            }
        }
        Ok(())
    }
}

impl Display for Reading<'_> {
    /// Writes the sequent two-sided: the hypotheses, `⊢`, the goal, with
    /// intuitionistic formulas.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        for (i, h) in self.hypotheses().enumerate() {
            if i > 0 {
                f.write_str(", ")?;
            }
            self.fmt_formula(h, f, false)?;
        }
        if self.hypotheses().next().is_some() {
            f.write_str(" ")?;
        }
        f.write_str("⊢ ")?;
        self.fmt_formula(self.goal, f, false)
    }
}

/// An intuitionistic formula of a read sequent, as a value that prints it.
#[derive(Clone, Copy, Debug)]
pub struct IllFormula<'a> {
    /// The reading the formula's position comes from.
    reading: &'a Reading<'a>,
    /// The formula's occurrence.
    id: OccId,
}

impl Display for IllFormula<'_> {
    /// Writes the formula with brackets around every binary subformula.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.reading.fmt_formula(self.id, f, false)
    }
}

#[cfg(all(test, feature = "parse"))]
mod tests {
    use super::*;
    use crate::Sequent;
    use crate::search::generate::Rng;

    /// Parses `input` and reads it.
    fn read(input: &str) -> Result<String, String> {
        let s: Sequent = input.parse().unwrap_or_else(|e| panic!("{input:?}: {e}"));
        let forest = Forest::new(&s).unwrap();
        Reading::new(&forest)
            .map(|r| r.to_string())
            .map_err(|e| e.describe(&forest).to_string())
    }

    /// Intuitionistic sequents print back two-sided with `⊸`, `1`, `⊤` and
    /// `0` recovered from their one-sided forms, the goal being the formula
    /// written right of `⊢`.
    #[test]
    fn shapes() {
        for (input, two_sided) in [
            ("A, A -o B |- B", "A, A ⊸ B ⊢ B"),
            ("|- A", "⊢ A"),
            ("A -o B -o C |- (A * B) -o C", "A ⊸ (B ⊸ C) ⊢ (A ⊗ B) ⊸ C"),
            ("|- A & B -o A + B", "⊢ (A & B) ⊸ (A ⊕ B)"),
            ("A & B |- A + B", "A & B ⊢ A ⊕ B"),
            ("!A, !(A -o B) |- !B", "!A, !(A ⊸ B) ⊢ !B"),
            ("(A -o B) -o C |- D", "(A ⊸ B) ⊸ C ⊢ D"),
            // `0` and `⊤` stand on the side they were written on.
            ("0 |- top", "0 ⊢ ⊤"),
            ("top |- top * top", "⊤ ⊢ ⊤ ⊗ ⊤"),
            ("A * top |- A", "A ⊗ ⊤ ⊢ A"),
            ("1, top, 0 |- 1 * top * 0", "1, ⊤, 0 ⊢ (1 ⊗ ⊤) ⊗ 0"),
        ] {
            assert_eq!(read(input).as_deref(), Ok(two_sided), "{input:?}");
        }
        for (input, message) in [
            (
                "|-",
                "an intuitionistic sequent has exactly one formula right of ⊢, not 0",
            ),
            (
                "A, B |-",
                "an intuitionistic sequent has exactly one formula right of ⊢, not 0",
            ),
            (
                "A |- B, C",
                "an intuitionistic sequent has exactly one formula right of ⊢, not 2: write the hypotheses left of ⊢",
            ),
            (
                "~A |- B",
                "formula 1 left of ⊢ cannot be a hypothesis of an intuitionistic sequent",
            ),
            (
                "|- B par ~A",
                "the subformula B ⅋ ~A is neither an intuitionistic formula nor the negation of one (⅋ only as A ⊸ B, that is ~A ⅋ B, and ? only under a negation)",
            ),
            (
                "|- A par B",
                "the subformula A ⅋ B is neither an intuitionistic formula nor the negation of one (⅋ only as A ⊸ B, that is ~A ⅋ B, and ? only under a negation)",
            ),
            (
                "A * (B par C) |- D",
                "the subformula ~B ⊗ ~C (in ~A ⅋ (~B ⊗ ~C)) is neither an intuitionistic formula nor the negation of one (⅋ only as A ⊸ B, that is ~A ⅋ B, and ? only under a negation)",
            ),
            (
                "|- ?A",
                "the subformula ?A is neither an intuitionistic formula nor the negation of one (⅋ only as A ⊸ B, that is ~A ⅋ B, and ? only under a negation)",
            ),
            (
                "|- ~A",
                "no formula can be the goal: an intuitionistic sequent has exactly one formula on the right of ⊢",
            ),
        ] {
            assert_eq!(
                read(input).as_ref().map_err(String::as_str),
                Err(message),
                "{input:?}"
            );
        }
        let s: Sequent = "|- A par B".parse().unwrap();
        let forest = Forest::new(&s).unwrap();
        assert_eq!(
            Reading::new(&forest).unwrap_err().to_string(),
            "subformula 0 is neither an intuitionistic formula nor the negation of one (⅋ only as A ⊸ B, that is ~A ⅋ B, and ? only under a negation)"
        );
    }

    /// Reads `input` as a sequent whose sides are not known.
    fn read_unsided(input: &str) -> Result<String, String> {
        let mut s: Sequent = input.parse().unwrap_or_else(|e| panic!("{input:?}: {e}"));
        s.antecedents = None;
        let forest = Forest::new(&s).unwrap();
        Reading::new(&forest)
            .map(|r| r.to_string())
            .map_err(|e| e.describe(&forest).to_string())
    }

    /// The reading takes the goal from the written sides and an
    /// implication's antecedent from its left factor, and never answers a
    /// question the input did not ask: `|- top, a` (two formulas right of
    /// `⊢`, read as `0 ⊢ a` by a guessed goal) and `(A -o bot) -o bot |- A`
    /// (read as `1 ⊸ (A ⊗ 1) ⊢ A` by a symmetric implication) are refused,
    /// with the sides known and without them, where without them a
    /// sequent with one reading is read and one with two is refused.
    #[test]
    fn the_written_sides_decide() {
        let several = "an intuitionistic sequent has exactly one formula right of ⊢, not 2: write the hypotheses left of ⊢";
        assert_eq!(read("|- top, a"), Err(several.to_owned()));
        assert_eq!(read("|- a, top"), Err(several.to_owned()));
        let h9 = "(A -o bot) -o bot |- A";
        assert!(
            read(h9)
                .unwrap_err()
                .contains("neither an intuitionistic formula")
        );
        assert!(
            read_unsided(h9)
                .unwrap_err()
                .contains("neither an intuitionistic formula")
        );
        // Without the sides: `a` cannot be a hypothesis, so `⊤` is one.
        assert_eq!(read_unsided("|- top, a").as_deref(), Ok("0 ⊢ a"));
        assert_eq!(
            read_unsided("|- 0, top"),
            Err(
                "both 0 and ⊤ can be the goal, and the sequent does not say which stands right of ⊢: give its sides".to_owned()
            )
        );
        assert_eq!(
            read_unsided("A, A -o B |- B").as_deref(),
            Ok("A, A ⊸ B ⊢ B")
        );
    }

    /// The positions: the antecedent of an implication flips, the rest
    /// inherits, and `implication` names the antecedent first.
    #[test]
    fn positions() {
        // ⊢ ~A ⅋ (~B ⅋ C), D: two formulas right of `⊢`, of which, the sides
        // unknown, both can only be the goal.
        let mut s: Sequent = "|- A -o (B -o C), D".parse().unwrap();
        let forest = Forest::new(&s).unwrap();
        assert_eq!(
            Reading::new(&forest).unwrap_err(),
            ShapeError::Succedents { count: 2 }
        );
        s.antecedents = None;
        let forest = Forest::new(&s).unwrap();
        assert!(matches!(
            Reading::new(&forest),
            Err(ShapeError::SeveralGoals(a, b)) if a == OccId::new(0) && b == OccId::new(5)
        ));
        // ⊢ A ⊗ (B ⊗ ~C), D: hypothesis A ⊸ (B ⊸ C): 0 ⊗, 1 A, 2 ⊗, 3 B, 4 ~C, 5 D
        let s: Sequent = "A -o (B -o C) |- D".parse().unwrap();
        let forest = Forest::new(&s).unwrap();
        let r = Reading::new(&forest).unwrap();
        let o = OccId::new;
        assert_eq!(r.goal(), o(5));
        assert_eq!(r.hypotheses().collect::<Vec<_>>(), [o(0)]);
        let positions: Vec<Side> = forest.ids().map(|i| r.position(i)).collect();
        use Side::*;
        assert_eq!(positions, [Input, Output, Input, Output, Input, Output]);
        assert_eq!(r.implication(o(0)), Some((o(1), o(2))));
        assert_eq!(r.implication(o(2)), Some((o(3), o(4))));
        assert_eq!(r.implication(o(1)), None);
        assert_eq!(r.outputs(forest.roots().iter().copied()), 1);
        assert_eq!(r.formula(o(2)).to_string(), "B ⊸ C");
        assert_eq!(r.formula(o(3)).to_string(), "B");
    }

    /// A random intuitionistic formula in the parser's syntax.
    fn formula(rng: &mut Rng, depth: usize) -> String {
        if depth == 0 || rng.one_in(3) {
            return match rng.below(6) {
                0 => "1".into(),
                1 => "top".into(),
                2 => "0".into(),
                _ => ["a", "b", "c"][rng.below(3)].into(),
            };
        }
        let sub = |rng: &mut Rng| formula(rng, depth - 1);
        match rng.below(6) {
            0 => format!("({} * {})", sub(rng), sub(rng)),
            1 => format!("({} -o {})", sub(rng), sub(rng)),
            2 => format!("({} & {})", sub(rng), sub(rng)),
            3 => format!("({} + {})", sub(rng), sub(rng)),
            4 => format!("!{}", sub(rng)),
            _ => format!("({} -o {})", sub(rng), sub(rng)),
        }
    }

    /// Parsing a random intuitionistic sequent, printing it two-sided and
    /// parsing the print gives the same sequent.
    #[test]
    fn print_parse_round_trip() {
        let mut rng = Rng::new(8);
        for _ in 0..500 {
            let hypotheses: Vec<String> = (0..rng.below(4)).map(|_| formula(&mut rng, 3)).collect();
            let input = format!("{} |- {}", hypotheses.join(", "), formula(&mut rng, 3));
            let s: Sequent = input.parse().unwrap_or_else(|e| panic!("{input:?}: {e}"));
            let forest = Forest::new(&s).unwrap();
            let printed = Reading::new(&forest)
                .unwrap_or_else(|e| panic!("{input:?}: {}", e.describe(&forest)))
                .to_string();
            let back: Sequent = printed
                .parse()
                .unwrap_or_else(|e| panic!("{input:?} printed as {printed:?}: {e}"));
            // The print is canonical (the antecedent of an implication comes
            // first), so its own reading prints the same, over the same
            // fragment and as many formulas.
            assert_eq!(
                back.fragment(),
                s.fragment(),
                "{input:?} printed as {printed:?}"
            );
            assert_eq!(
                back.roots().len(),
                s.roots().len(),
                "{input:?} printed as {printed:?}"
            );
            let forest = Forest::new(&back).unwrap();
            assert_eq!(Reading::new(&forest).unwrap().to_string(), printed);
        }
    }
}
