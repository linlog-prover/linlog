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
use crate::errors::describe::{Limit, Names, cut};
use crate::errors::{Described, Owner, Subject};
use crate::sequents::notation::TEXT;
use crate::sequents::{Kind, Walk};
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
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ShapeError {
    /// No root formula can be the goal: with the sides known, the formula
    /// right of `⊢` cannot; without them, every one is input-shaped only, or
    /// the sequent is empty.
    NoGoal,
    /// Two root formulas can only be goals (the sides not known).
    #[non_exhaustive]
    SeveralGoals {
        /// The first.
        first: OccId,
        /// The second.
        second: OccId,
    },
    /// A subformula is neither an intuitionistic formula nor the negation of
    /// one, in any position.
    #[non_exhaustive]
    Formula {
        /// The subformula.
        occurrence: OccId,
    },
    /// The sequent has another number of formulas right of `⊢` than one.
    #[non_exhaustive]
    Succedents {
        /// How many formulas stand right of `⊢`.
        count: usize,
    },
    /// A formula written left of `⊢` (the first is 0) cannot be a
    /// hypothesis.
    #[non_exhaustive]
    Hypothesis {
        /// Its index among the formulas left of `⊢`.
        index: usize,
    },
    /// Two root formulas can each be the goal, every other root reading as
    /// a hypothesis either way, and the sequent does not say which was
    /// written right of `⊢`.
    #[non_exhaustive]
    Undetermined {
        /// The first root that can be the goal.
        first: OccId,
        /// The second.
        second: OccId,
    },
    /// In the Lambek calculus without empty antecedents, the sequent has
    /// no formula left of `⊢`.
    EmptyAntecedent,
    /// A cut formula is no intuitionistic formula: the root of a cut
    /// formula `A`, which a cut proves in one premise and uses as a
    /// hypothesis in the other, cannot be output, or its dual's cannot
    /// be input.
    #[non_exhaustive]
    CutFormula {
        /// The root of the cut formula `A`.
        occurrence: OccId,
    },
}

impl Display for ShapeError {
    /// Writes the reason with occurrence ids, such as `subformula 3 has no
    /// intuitionistic reading`.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.write(f, None, None)
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
    pub(crate) fn write(
        &self,
        f: &mut Formatter<'_>,
        names: Option<Names<'_>>,
        limit: Limit,
    ) -> FmtResult {
        let occurrence = |f: &mut Formatter<'_>, o: OccId| cut(f, names, o, limit);
        let forest = names.map(|n| n.forest);
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
            ShapeError::SeveralGoals { first: a, second: b } => {
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
            ShapeError::EmptyAntecedent => f.write_str(
                "the sequent has no formula left of ⊢, which the Lambek calculus without empty antecedents needs",
            ),
            ShapeError::CutFormula { occurrence: o } => {
                f.write_str(if forest.is_some() {
                    "the cut formula "
                } else {
                    "cut formula "
                })?;
                occurrence(f, *o)?;
                f.write_str(" is no intuitionistic formula")
            }
            ShapeError::Formula { occurrence: o } => {
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
/// - **A cut formula** `A` ([`Sequent::cuts`](crate::Sequent::cuts)) is
///   proved by one premise of its cut and used as a hypothesis by the
///   other: its tree reads in output position and its dual's in input
///   position ([`ShapeError::CutFormula`] where either cannot).
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
    /// Whether the reading is the ordered one of the Lambek calculus.
    ordered: bool,
}

impl<'a> Reading<'a> {
    /// Reads the forest as an intuitionistic sequent, or says why it is
    /// none: a subformula with no reading in any position, a formula on a
    /// side of `⊢` it cannot stand on, or, with the sides unknown, no root
    /// that can be the goal or two that can.
    ///
    /// # Errors
    ///
    /// A [`ShapeError`]: [`Formula`](ShapeError::Formula) for a subformula
    /// with no reading, and for the roots
    /// [`Succedents`](ShapeError::Succedents), [`NoGoal`](ShapeError::NoGoal),
    /// [`Hypothesis`](ShapeError::Hypothesis),
    /// [`SeveralGoals`](ShapeError::SeveralGoals) or
    /// [`Undetermined`](ShapeError::Undetermined).
    pub fn new(forest: &'a Forest) -> Result<Self, ShapeError> {
        Self::read(forest, false)
    }

    /// Reads the forest as a sequent of `mode`: as [`new`](Self::new)
    /// does in a commutative mode, and in an ordered one as a sequent of
    /// the Lambek calculus, whose formulas stand in the cyclic order of
    /// [`Sequent::parse_ordered`](crate::Sequent::parse_ordered). There
    /// an implication is a division, its antecedent the factor in the
    /// other position from it, whichever that is: in output position
    /// `A⊥ ⅋ B` is `A \ B` and `B ⅋ A⊥` is `B / A`, and in input
    /// position, where the reversing dual turned every product round,
    /// the operands of `⊗` and `⅋` are written right one first, so that
    /// `B⊥ ⊗ A` is the hypothesis `A \ B` and `A ⊗ B⊥` is `B / A`. The
    /// hypotheses are read going round from the goal backwards, which is
    /// the order they were written in. Without empty antecedents
    /// ([`Mode::allows_empty_antecedents`]) a sequent needs a hypothesis.
    ///
    /// [`Mode::allows_empty_antecedents`]: crate::Mode::allows_empty_antecedents
    ///
    /// # Errors
    ///
    /// Those of [`new`](Self::new), and [`ShapeError::EmptyAntecedent`]
    /// for a sequent without a hypothesis in the Lambek calculus without
    /// empty antecedents.
    ///
    /// # Examples
    ///
    #[cfg_attr(feature = "parse", doc = "```")]
    #[cfg_attr(not(feature = "parse"), doc = "```ignore")]
    /// use linlog::{Forest, Limits, Mode, Order, Reading, Sequent};
    ///
    /// let text = "b / a, a |- b";
    /// let sequent = Sequent::parse_ordered(text, Order::Cyclic, &Limits::default())?;
    /// let forest = Forest::new(&sequent)?;
    /// let reading = Reading::of_mode(&forest, Mode::LAMBEK)?;
    /// assert_eq!(reading.to_string(), "b / a, a ⊢ b");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn of_mode(forest: &'a Forest, mode: crate::Mode) -> Result<Self, ShapeError> {
        let reading = Self::read(forest, mode.is_ordered())?;
        if !mode.allows_empty_antecedents() && forest.roots().len() < 2 {
            return Err(ShapeError::EmptyAntecedent);
        }
        Ok(reading)
    }

    /// Reads the forest, in the Lambek calculus's order when `ordered`.
    fn read(forest: &'a Forest, ordered: bool) -> Result<Self, ShapeError> {
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
                // `A⊥ ⅋ B`: the left factor is the antecedent, or in the
                // ordered reading either factor.
                Tensor => both(l_out && r_in || ordered && l_in && r_out, l_out && r_out),
                Par => both(l_in && r_in, l_in && r_out || ordered && l_out && r_in),
            };
            if can[o.index()] == 0 {
                return Err(ShapeError::Formula { occurrence: o });
            }
        }

        let roots = forest.roots();
        let goal = match forest.sequent().antecedents() {
            Some(k) => {
                let (left, right) = roots.split_at(k as usize);
                let &[goal] = right else {
                    return Err(ShapeError::Succedents { count: right.len() });
                };
                let refused = |o: &OccId| can[o.index()] & IN == 0;
                // The ordered parse reversed the formulas left of ⊢: the
                // first written is the last root before the goal.
                let index = if ordered {
                    left.iter().rposition(refused).map(|i| left.len() - 1 - i)
                } else {
                    left.iter().position(refused)
                };
                if let Some(index) = index {
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
                    (Some(a), Some(b)) => {
                        return Err(ShapeError::SeveralGoals {
                            first: a,
                            second: b,
                        });
                    }
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

        // A cut formula is the goal of one premise and a hypothesis of
        // the other: `A` output, `A⊥` input.
        for (a, dual) in forest.cut_pairs() {
            if can[a.index()] & OUT == 0 || can[dual.index()] & IN == 0 {
                return Err(ShapeError::CutFormula { occurrence: a });
            }
        }

        // The positions, parents before children.
        let mut position = vec![Side::Input; forest.len()].into_boxed_slice();
        position[goal.index()] = Side::Output;
        for (a, _) in forest.cut_pairs() {
            position[a.index()] = Side::Output;
        }
        for o in forest.ids() {
            let p = position[o.index()];
            let Some((l, r)) = forest.left(o).zip(forest.right(o)) else {
                if let Some(l) = forest.left(o) {
                    position[l.index()] = p;
                }
                continue;
            };
            // An implication's antecedent flips: its left factor, or in
            // the ordered reading the right one where only that can.
            let implication = matches!(
                (forest.kind(o), p),
                (Tensor, Side::Input) | (Par, Side::Output)
            );
            let takes = |k: OccId, side: Side| {
                can[k.index()] & if side == Side::Input { IN } else { OUT } != 0
            };
            let right_flips = implication && ordered && !(takes(l, !p) && takes(r, p));
            position[l.index()] = if implication && !right_flips { !p } else { p };
            position[r.index()] = if right_flips { !p } else { p };
        }
        Ok(Self {
            forest,
            position: Cow::Owned(position.into_vec()),
            goal,
            ordered,
        })
    }

    /// Returns the positions and the goal, for an owner of the forest to
    /// keep beside it.
    pub(crate) fn into_parts(self) -> (Box<[Side]>, OccId) {
        (self.position.into_owned().into_boxed_slice(), self.goal)
    }

    /// Returns the bytes the reading allocated itself: its positions, where
    /// it does not read a session's.
    pub(crate) fn held_bytes(&self) -> usize {
        match &self.position {
            Cow::Owned(position) => position.capacity() * size_of::<Side>(),
            Cow::Borrowed(_) => 0,
        }
    }

    /// Returns the reading of `forest` from the positions and the goal
    /// [`into_parts`](Self::into_parts) gave, in constant time.
    pub(crate) const fn of_parts(
        forest: &'a Forest,
        position: &'a [Side],
        goal: OccId,
        ordered: bool,
    ) -> Self {
        Self {
            forest,
            position: Cow::Borrowed(position),
            goal,
            ordered,
        }
    }

    /// Returns whether this is the ordered reading of the Lambek
    /// calculus ([`of_mode`](Self::of_mode)).
    pub fn is_ordered(&self) -> bool {
        self.ordered
    }

    /// Returns the forest read.
    pub fn forest(&self) -> &'a Forest {
        self.forest
    }

    /// Returns the position of an occurrence.
    ///
    /// # Panics
    ///
    /// For an id of another forest, past this one's end.
    pub fn position(&self, o: OccId) -> Side {
        self.position[o.index()]
    }

    /// Returns the goal: the one root in output position.
    pub fn goal(&self) -> OccId {
        self.goal
    }

    /// Returns the hypotheses: every root but the goal, in the order
    /// written. In the ordered reading that is going round from the goal
    /// backwards, the formulas before it from the last, then those after
    /// it from the last.
    pub fn hypotheses(&self) -> impl Iterator<Item = OccId> + '_ {
        let roots = self.forest.roots();
        // The goal is a root, and the roots ascend.
        let at = roots.partition_point(|&r| r < self.goal);
        let len = roots.len();
        let ordered = self.ordered;
        (0..len - 1).map(move |i| {
            roots[match (ordered, i < at) {
                (false, true) => i,
                (false, false) => i + 1,
                (true, true) => at - 1 - i,
                (true, false) => len - 1 - (i - at),
            }]
        })
    }

    /// Returns the positions in `sequent`, a sequent of the forest's
    /// occurrences in ascending order, of its hypotheses in the order a
    /// two-sided sequent writes them, then of its goal: in id order, or
    /// in the ordered reading going round from the goal backwards.
    pub(crate) fn written<'s>(
        &'s self,
        sequent: &'s [super::Member],
    ) -> impl Iterator<Item = usize> + 's {
        let output = move |p: usize| self.position(sequent[p].occ()) == Side::Output;
        let len = sequent.len();
        let ordered = self.ordered;
        let at = if ordered {
            (0..len).rfind(|&p| output(p)).unwrap_or(len)
        } else {
            0
        };
        let order = (0..len).map(move |i| match (ordered, i < at) {
            (false, _) => i,
            (true, true) => at - 1 - i,
            (true, false) => len - 1 - (i - at),
        });
        order
            .clone()
            .filter(move |&p| !output(p))
            .chain(order.filter(move |&p| output(p)))
    }

    /// Returns the antecedent and the consequent of an implication `A ⊸ B`:
    /// a `⅋` in output position or a `⊗` in input position, whose antecedent
    /// is the left factor, or in the ordered reading the factor in the
    /// other position. `None` for any other occurrence.
    pub fn implication(&self, o: OccId) -> Option<(OccId, OccId)> {
        match (self.forest.kind(o), self.position(o)) {
            (Kind::Par, Side::Output) | (Kind::Tensor, Side::Input) => {
                let (l, r) = (self.forest.left(o)?, self.forest.right(o)?);
                if self.ordered && self.position(r) != self.position(o) {
                    Some((r, l))
                } else {
                    Some((l, r))
                }
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

    /// Returns the connective, unit or atom that the occurrence `o` is as
    /// an intuitionistic formula in its position: an output `⅋` or an input
    /// `⊗` is an implication, in the ordered reading a division, `⊥` is
    /// `1`, an input `⊤` is `0`, and so on.
    pub fn connective(&self, o: OccId) -> IllConnective {
        use IllConnective as C;
        if self.ordered
            && let Some((antecedent, _)) = self.implication(o)
        {
            return if Some(antecedent) == self.operands(o).0 {
                C::Under
            } else {
                C::Over
            };
        }
        match (self.forest.kind(o), self.position(o)) {
            (Kind::Atom | Kind::DualAtom, _) => C::Atom,
            (Kind::One | Kind::Bot, _) => C::One,
            (Kind::Top, Side::Output) | (Kind::Zero, Side::Input) => C::Top,
            (Kind::Zero, Side::Output) | (Kind::Top, Side::Input) => C::Zero,
            (Kind::Bang | Kind::Quest, _) => C::Bang,
            (Kind::Tensor, Side::Output) | (Kind::Par, Side::Input) => C::Tensor,
            (Kind::Tensor, Side::Input) | (Kind::Par, Side::Output) => C::Lollipop,
            (Kind::With, Side::Output) | (Kind::Plus, Side::Input) => C::With,
            (Kind::Plus, Side::Output) | (Kind::With, Side::Input) => C::Plus,
        }
    }

    /// Returns the walk over the intuitionistic formula at `o`: the stops
    /// a printer of it in another syntax follows, each naming an
    /// occurrence whose [`connective`](Self::connective) says what it is,
    /// the operands of an implication its antecedent and then its
    /// consequent.
    #[cfg_attr(feature = "parse", doc = "```")]
    #[cfg_attr(not(feature = "parse"), doc = "```ignore")]
    /// use linlog::occurrences::IllConnective;
    /// use linlog::sequents::Visit;
    /// use linlog::{Forest, Reading, Sequent};
    ///
    /// let sequent: Sequent = "a |- !b -o a".parse()?;
    /// let forest = Forest::new(&sequent)?;
    /// let reading = Reading::new(&forest)?;
    /// let mut out = String::new();
    /// for visit in reading.walk(reading.goal()) {
    ///     match visit {
    ///         Visit::Enter(o) => match reading.connective(o) {
    ///             IllConnective::Atom => {
    ///                 let atom = forest.atom(o).unwrap();
    ///                 out += sequent.atom_name(atom);
    ///             }
    ///             IllConnective::Bang => out += "!",
    ///             _ => out += "(",
    ///         },
    ///         Visit::Between(..) => out += " -> ",
    ///         Visit::Exit(o) if reading.connective(o) == IllConnective::Bang => {}
    ///         Visit::Exit(_) => out += ")",
    ///     }
    /// }
    /// assert_eq!(out, "(!b -> a)");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn walk(&self, o: OccId) -> Walk<'_, OccId> {
        Walk::new(o, |o| self.operands(o))
    }

    /// Returns the first and the second subformula of the intuitionistic
    /// formula at `o`, each if there is one, in the order they are written:
    /// of an implication the antecedent, then the consequent. In the
    /// ordered reading the operands of a product in input position are
    /// written right one first, the reversing dual having turned them
    /// round, which puts the consequent of `B / A` first.
    pub(crate) fn operands(&self, o: OccId) -> (Option<OccId>, Option<OccId>) {
        let (l, r) = (self.forest.left(o), self.forest.right(o));
        if self.ordered {
            let reversed = self.position(o) == Side::Input
                && matches!(self.forest.kind(o), Kind::Tensor | Kind::Par);
            return if reversed { (r, l) } else { (l, r) };
        }
        match self.implication(o) {
            Some((antecedent, consequent)) => (Some(antecedent), Some(consequent)),
            None => (l, r),
        }
    }
}

impl Display for Reading<'_> {
    /// Writes the sequent two-sided: the hypotheses, `⊢`, the goal, with
    /// intuitionistic formulas.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        TEXT.reading(f, self)
    }
}

/// What an occurrence is as a formula of intuitionistic linear logic
/// under its position: the table every printer of a reading follows.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum IllConnective {
    /// An atom `a`, written `a` in output position and `~a` in input.
    Atom,
    /// `1`, an output `1` or an input `⊥`.
    One,
    /// `0`, an output `0` or an input `⊤`.
    Zero,
    /// `⊤`, an output `⊤` or an input `0`.
    Top,
    /// `!A`, an output `!` or an input `?`.
    Bang,
    /// `A ⊗ B`, an output `⊗` or an input `⅋`.
    Tensor,
    /// `A & B`, an output `&` or an input `⊕`.
    With,
    /// `A ⊕ B`, an output `⊕` or an input `&`.
    Plus,
    /// `A ⊸ B`, an output `⅋` or an input `⊗`, whose antecedent is its
    /// first operand.
    Lollipop,
    /// `A \ B` in the Lambek calculus's ordered reading: an output `⅋`
    /// or an input `⊗` whose antecedent `A` is its first operand.
    Under,
    /// `B / A` in the Lambek calculus's ordered reading: an output `⅋` or
    /// an input `⊗` whose antecedent `A` is its second operand.
    Over,
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
        TEXT.ill(f, self.reading, self.id, false)
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

    /// One occurrence, anywhere below a root, is written as the reading
    /// reads it in its position: a hypothesis `A ⊸ B` (one-sided `A ⊗
    /// B⊥`) as `A ⊸ B`, the same formula under a hypothesis's `!` and
    /// under the goal's `!` alike, and a negated atom as its atom.
    #[test]
    fn one_occurrence_is_written_in_its_position() {
        let s: Sequent = "!(A -o B), A |- !(A -o B) * A".parse().unwrap();
        let forest = Forest::new(&s).unwrap();
        let reading = Reading::new(&forest).unwrap();
        let written: Vec<String> = forest
            .ids()
            .map(|o| reading.formula(o).to_string())
            .collect();
        let at = |text: &str| written.iter().filter(|w| *w == text).count();
        // Under each `!`, the implication; at the roots, the hypotheses and the goal.
        assert_eq!(at("A ⊸ B"), 2, "{written:?}");
        assert_eq!(at("!(A ⊸ B)"), 2, "{written:?}");
        assert_eq!(at("!(A ⊸ B) ⊗ A"), 1, "{written:?}");
        assert!(
            written.iter().all(|w| !w.contains('~') && !w.contains('⊥')),
            "{written:?}"
        );
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
    /// The ordered reading names the first formula written left of ⊢
    /// that cannot be a hypothesis, as the commutative one does.
    #[test]
    fn the_first_written_hypothesis_is_named() {
        let sequent = Sequent::parse_ordered(
            "~a, b, ~c |- d",
            crate::Order::Cyclic,
            &crate::Limits::default(),
        )
        .unwrap();
        let forest = Forest::new(&sequent).unwrap();
        assert_eq!(
            Reading::of_mode(&forest, crate::Mode::LAMBEK).unwrap_err(),
            ShapeError::Hypothesis { index: 0 }
        );
    }

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
            Err(ShapeError::SeveralGoals { first: a, second: b }) if a == OccId::new(0) && b == OccId::new(5)
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
