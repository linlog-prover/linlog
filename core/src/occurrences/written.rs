// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! A sequent of classical linear logic as its author wrote it: the
//! one-sided forest every engine, checker and view works on, and beside
//! it, per occurrence, what the lowering to it merged. Classical syntax
//! has negation and both sides of `⊢`, so `Γ, A ⊢ Δ` and `Γ ⊢ A⊥, Δ`,
//! `~(A ⊗ B)` and `A⊥ ⅋ B⊥`, `A ⊸ B` and `A⊥ ⅋ B` lower alike; the
//! record keeps which was written, recorded where the text is read and
//! never inferred from the one-sided form.

use super::reading::Side;
use super::{Forest, OccId};
use crate::sequents::notation::TEXT;
use crate::sequents::{Kind, Sequent};
use crate::{Error, Limits};
use std::fmt::{Display, Formatter, Result as FmtResult};

/// How one occurrence of a classical sequent was written: the side of `⊢`
/// its connective stands on, and whether it is an implication.
///
/// The side is the one-sided occurrence's position once every negation
/// above it is passed: `Output` where the one-sided connective is the
/// written one, `Input` where the written one is its dual (`A ⊗ B` left
/// of `⊢` is the one-sided `A⊥ ⅋ B⊥` on the input side). A negation was
/// written before the occurrence exactly where its side differs from the
/// side its parent puts it on (its context), up to double negations; an
/// atom is `Output`, a negated atom `Input`. An implication is a `⅋` on
/// the output side or a `⊗` on the input side written as `A ⊸ B`, whose
/// left factor is then the antecedent, on the other side than the
/// implication's.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Written(u8);

impl Written {
    /// The bit of a connective standing on the input side.
    const INPUT: u8 = 1;
    /// The bit of an implication.
    const IMPLICATION: u8 = 2;

    /// Returns the record of a connective written on `side`, an
    /// implication or not.
    pub(crate) const fn new(side: Side, implication: bool) -> Self {
        let input = match side {
            Side::Input => Self::INPUT,
            Side::Output => 0,
        };
        Self(input | if implication { Self::IMPLICATION } else { 0 })
    }

    /// Returns the side of `⊢` the written connective stands on.
    pub const fn side(self) -> Side {
        if self.0 & Self::INPUT == 0 {
            Side::Output
        } else {
            Side::Input
        }
    }

    /// Returns whether the occurrence was written as an implication,
    /// `A ⊸ B`.
    pub const fn is_implication(self) -> bool {
        self.0 & Self::IMPLICATION != 0
    }
}

/// Returns the side of `⊢` an occurrence stands on as its parent writes
/// it: a root's side of the turnstile (a cut formula `A` right of it, as
/// the premise that proves it writes it, and its dual left of it), the
/// implication's other side for its antecedent, and its parent's side for
/// every other subformula.
pub(crate) fn context(forest: &Forest, writing: &[Written], o: OccId) -> Side {
    match forest.parent(o) {
        Some(parent) => {
            let written = writing[parent.index()];
            if written.is_implication() && forest.left(parent) == Some(o) {
                !written.side()
            } else {
                written.side()
            }
        }
        None if !forest.is_conclusion(o) => {
            if forest.is_cut_formula(o) {
                Side::Output
            } else {
                Side::Input
            }
        }
        None => {
            let roots = forest.roots();
            let antecedents = forest.sequent().antecedents().unwrap_or(0) as usize;
            // The roots are in ascending order, so a search finds one.
            match roots.binary_search(&o) {
                Ok(index) if index < antecedents => Side::Input,
                _ => Side::Output,
            }
        }
    }
}

/// Returns whether a negation stands before the occurrence as written:
/// its side differs from its context's.
pub(crate) fn negated(forest: &Forest, writing: &[Written], o: OccId) -> bool {
    writing[o.index()].side() != context(forest, writing, o)
}

/// Returns why a record cannot be one of the forest's, or `None` where it
/// can: one entry per occurrence, every atom on the output side and every
/// negated atom on the input side, and an implication only of a `⅋` on the
/// output side or a `⊗` on the input side.
pub(crate) fn fault(forest: &Forest, writing: &[Written]) -> Option<&'static str> {
    if writing.len() != forest.len() {
        return Some("the record has another number of occurrences than the sequent");
    }
    for o in forest.ids() {
        let (kind, written) = (forest.kind(o), writing[o.index()]);
        let side = written.side();
        if (kind == Kind::Atom && side == Side::Input)
            || (kind == Kind::DualAtom && side == Side::Output)
        {
            return Some("an atom is recorded on the wrong side");
        }
        let implication = (kind == Kind::Par && side == Side::Output)
            || (kind == Kind::Tensor && side == Side::Input);
        if written.is_implication() && !implication {
            return Some("an implication is recorded on what is no implication");
        }
    }
    None
}

/// The record of a one-sided sequent as its one-sided syntax writes it:
/// every formula on its root's side of `⊢` (a cut formula right of it,
/// its dual left of it), and an atom negated by a negation of its own
/// where its root's side is not its own.
pub(crate) fn one_sided(forest: &Forest) -> Box<[Written]> {
    let mut writing = vec![Written::new(Side::Output, false); forest.len()];
    let antecedents = forest.sequent().antecedents().unwrap_or(0) as usize;
    let roots = forest.roots().iter().enumerate().map(|(index, &root)| {
        let side = if index < antecedents {
            Side::Input
        } else {
            Side::Output
        };
        (root, side)
    });
    let cuts = forest
        .cut_pairs()
        .flat_map(|(a, dual)| [(a, Side::Output), (dual, Side::Input)]);
    for (root, side) in roots.chain(cuts) {
        let end = root.index() + forest.size(root) as usize;
        for (k, written) in writing.iter_mut().enumerate().take(end).skip(root.index()) {
            // A literal's side is its sign's.
            let side = match forest.kind(OccId::new(k as u32)) {
                Kind::Atom => Side::Output,
                Kind::DualAtom => Side::Input,
                Kind::One
                | Kind::Bot
                | Kind::Top
                | Kind::Zero
                | Kind::Tensor
                | Kind::Par
                | Kind::With
                | Kind::Plus
                | Kind::Bang
                | Kind::Quest => side,
            };
            *written = Written::new(side, false);
        }
    }
    writing.into_boxed_slice()
}

/// A sequent of classical linear logic, `Γ ⊢ Δ`, as its author wrote it:
/// the formulas in the order written on their sides of `⊢`, negations and
/// implications where they were written.
///
/// It is built from text by [`parse_within`](Self::parse_within) (or
/// [`str::parse`]), which reads the syntax [`Sequent`] reads and records
/// how every occurrence was written, and by
/// [`read_one_sided`](Self::read_one_sided), which takes a one-sided
/// sequent as its one-sided syntax writes it. Either lowers the sequent
/// once to the [`Forest`] the engines take; the record
/// ([`writing`](Self::writing), a byte per occurrence) is what printing
/// reads to write the sequent back as it was written, up to brackets and
/// double negations, and what the two-sided view of a derivation
/// ([`TwoSided`](crate::TwoSided)) reads to split its sequents by side.
///
/// # Examples
///
#[cfg_attr(feature = "parse", doc = "```")]
#[cfg_attr(not(feature = "parse"), doc = "```ignore")]
/// use linlog::{ClassicalSequent, Side};
///
/// let written: ClassicalSequent = "A -o B, A |- B".parse()?;
/// assert_eq!(written.to_string(), "A ⊸ B, A ⊢ B");
/// assert_eq!(written.sequent().to_string(), "⊢ A ⊗ ~B, ~A, B");
/// let moved: ClassicalSequent = "|- ~(A -o B), A^, B".parse()?;
/// assert_eq!(moved.to_string(), "⊢ ~(A ⊸ B), ~A, B");
/// assert_eq!(moved.sequent().to_string(), written.sequent().to_string());
/// let implication = moved.forest().roots()[0];
/// assert!(moved.is_negated(implication));
/// assert_eq!(moved.written(implication).side(), Side::Input);
/// # Ok::<(), linlog::Error>(())
/// ```
#[derive(Clone, Debug)]
pub struct ClassicalSequent {
    /// The one-sided lowering, laid out as the forest the engines take.
    forest: Forest,
    /// Per occurrence, how it was written.
    writing: Box<[Written]>,
}

impl ClassicalSequent {
    /// Reads a one-sided sequent as its one-sided syntax writes it: the
    /// formulas on the sides of `⊢` the sequent records
    /// ([`Sequent::antecedents`]), the right side where it records none,
    /// each with the connectives the one-sided formula has and a negated
    /// atom written as the negation of its atom. `⊢ ~a, a` stays `⊢ ~a,
    /// a`.
    ///
    /// # Errors
    ///
    /// The errors of [`Forest::from_owned`] past the limits.
    pub fn read_one_sided(sequent: Sequent, limits: &Limits) -> Result<Self, Error> {
        let forest = Forest::from_owned(sequent, limits)?;
        let writing = one_sided(&forest);
        Ok(Self { forest, writing })
    }

    /// Returns the one-sided sequent the engines take: the formulas left
    /// of `⊢` negated, negations pushed to the atoms, `A ⊸ B` as `A⊥ ⅋ B`.
    pub fn sequent(&self) -> &Sequent {
        self.forest.sequent()
    }

    /// Returns the forest of the one-sided sequent.
    pub fn forest(&self) -> &Forest {
        &self.forest
    }

    /// Returns the one-sided sequent, for a caller that hands it on.
    pub fn into_sequent(self) -> Sequent {
        self.forest.sequent().clone()
    }

    /// Returns how every occurrence was written, by occurrence id.
    pub fn writing(&self) -> &[Written] {
        &self.writing
    }

    /// Returns how the occurrence `o` was written.
    ///
    /// # Panics
    ///
    /// For an id of another forest, past this one's end.
    pub fn written(&self, o: OccId) -> Written {
        self.writing[o.index()]
    }

    /// Returns whether a negation was written before the occurrence `o`,
    /// up to double negations.
    ///
    /// # Panics
    ///
    /// For an id of another forest, past this one's end.
    pub fn is_negated(&self, o: OccId) -> bool {
        negated(&self.forest, &self.writing, o)
    }

    /// Returns the formula at `o` as it was written, its negation
    /// included: `~(A ⊗ B)`, `A ⊸ B`.
    ///
    /// # Panics
    ///
    /// For an id of another forest, past this one's end.
    pub fn formula(&self, o: OccId) -> impl Display + '_ {
        Shown {
            forest: &self.forest,
            writing: &self.writing,
            o,
            negated: self.is_negated(o),
        }
    }
}

/// An occurrence written as its author wrote it, with a negation before
/// it or without.
pub(crate) struct Shown<'a> {
    /// The forest the occurrence is of.
    pub(crate) forest: &'a Forest,
    /// How every occurrence was written.
    pub(crate) writing: &'a [Written],
    /// The occurrence.
    pub(crate) o: OccId,
    /// Whether a negation stands before it.
    pub(crate) negated: bool,
}

impl Display for Shown<'_> {
    /// Writes the formula in Unicode.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        TEXT.written(f, self.forest, self.writing, self.o, self.negated)
    }
}

impl PartialEq for ClassicalSequent {
    /// Two sequents are equal when they are written alike: the same
    /// formulas on the same sides, the same negations and implications,
    /// however their arenas share subterms.
    fn eq(&self, other: &Self) -> bool {
        let (a, b) = (&self.forest, &other.forest);
        fn name(forest: &Forest, o: OccId) -> Option<&str> {
            forest.atom(o).map(|x| forest.sequent().atom_name(x))
        }
        let antecedents = |forest: &Forest| forest.sequent().antecedents().unwrap_or(0);
        a.len() == b.len()
            && a.roots() == b.roots()
            && antecedents(a) == antecedents(b)
            && self.writing == other.writing
            && a.ids()
                .all(|o| a.kind(o) == b.kind(o) && name(a, o) == name(b, o))
    }
}

impl Eq for ClassicalSequent {}

impl Display for ClassicalSequent {
    /// Writes the sequent as it was written, up to spacing, brackets and
    /// double negations: `A ⊸ B, A ⊢ B`.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let (forest, writing) = (&self.forest, &self.writing[..]);
        let antecedents = forest.sequent().antecedents().unwrap_or(0) as usize;
        let shown = |&o: &OccId| (o, negated(forest, writing, o));
        let roots = forest.roots();
        let (left, right) = roots.split_at(antecedents.min(roots.len()));
        TEXT.classical(
            f,
            forest,
            writing,
            left.iter().map(shown),
            right.iter().map(shown),
            false,
            false,
        )
    }
}

#[cfg(feature = "parse")]
impl ClassicalSequent {
    /// Parses a sequent of classical linear logic, `A -o B, A |- B`, in the
    /// syntax [`Sequent`] reads, and records how every occurrence was
    /// written. Refused at the first term past `limits.occurrences`, as
    /// [`Sequent::parse_within`] refuses; its one-sided
    /// [`sequent`](Self::sequent) is the one that call returns.
    ///
    /// Needs the cargo feature `parse` (on by default).
    ///
    /// # Errors
    ///
    /// Those of [`Sequent::parse_within`].
    pub fn parse_within(text: &str, limits: &Limits) -> Result<Self, Error> {
        let (sequent, writing) = crate::parse::written(text, limits)?;
        let forest = Forest::from_owned(sequent, limits)?;
        debug_assert_eq!(fault(&forest, &writing), None, "a parsed record fits");
        Ok(Self {
            forest,
            writing: writing.into_boxed_slice(),
        })
    }
}

#[cfg(feature = "parse")]
impl std::str::FromStr for ClassicalSequent {
    type Err = Error;

    /// Parses a sequent as [`ClassicalSequent::parse_within`] does, within
    /// the default [`Limits`].
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse_within(s, &Limits::default())
    }
}

#[cfg(all(test, feature = "parse"))]
mod tests {
    use super::*;
    use crate::search::generate::{self, Rng, Rules};

    /// Returns a record as a letter per occurrence: `o` or `i` for the
    /// side, upper case for an implication.
    fn letters(writing: &[Written]) -> String {
        let letter = |w: &Written| match (w.side(), w.is_implication()) {
            (Side::Output, false) => 'o',
            (Side::Input, false) => 'i',
            (Side::Output, true) => 'O',
            (Side::Input, true) => 'I',
        };
        writing.iter().map(letter).collect()
    }

    /// The parser records each occurrence's side and implications as
    /// written, a double negation recording nothing, and printing writes
    /// the sequent back.
    #[test]
    fn the_parser_records_what_was_written() {
        let cases = [
            // `A ⊗ ~B` on the input side, written as an implication.
            ("A -o B |- ~(C * D), ~~E", "Ioiiiio", "A ⊸ B ⊢ ~(C ⊗ D), E"),
            // `~A ⅋ B` on the output side, once each way.
            ("|- A -o B, A^ par B", "Oiooio", "⊢ A ⊸ B, ~A ⅋ B"),
            (
                "bot, 1 |- top, ~0, !A, ?~B",
                "iioioooi",
                "⊥, 1 ⊢ ⊤, ~0, !A, ?~B",
            ),
            ("~(A -o B) |- ~!A", "Oioii", "~(A ⊸ B) ⊢ ~!A"),
        ];
        for (text, record, printed) in cases {
            let written: ClassicalSequent = text.parse().unwrap();
            assert_eq!(letters(written.writing()), record, "{text}");
            assert_eq!(written.to_string(), printed, "{text}");
            let one_sided: Sequent = text.parse().unwrap();
            assert_eq!(written.sequent(), &one_sided, "{text}");
        }
    }

    /// A one-sided sequent read as written keeps its one-sided syntax, its
    /// formulas on the sides it records.
    #[test]
    fn a_one_sided_sequent_is_written_one_sided() {
        let limits = Limits::default();
        for (text, printed) in [("|- ~a, a", "⊢ ~a, a"), ("a |- a", "a ⊢ a")] {
            let sequent: Sequent = text.parse().unwrap();
            let written = ClassicalSequent::read_one_sided(sequent, &limits).unwrap();
            assert_eq!(fault(written.forest(), written.writing()), None);
            assert_eq!(written.to_string(), printed);
        }
    }

    /// Over generated sequents written at random (sides, negations,
    /// double negations and implications), the parser's record is the one
    /// the writing made, the one-sided sequent is the generated one, and
    /// printing the sequent gives back what was written: it parses to an
    /// equal sequent, record and all, and prints the same again.
    #[test]
    fn a_written_sequent_prints_back_to_itself() {
        let mut tried = 0;
        for (i, rules) in Rules::ALL.into_iter().enumerate() {
            let mut rng = Rng::new(40 + i as u64);
            for _ in 0..12 {
                let budget = 2 + rng.below(10);
                let provable = generate::provable(&mut rng, rules, 3, budget);
                let one_sided: Sequent = generate::sequent(&provable.formulas).parse().unwrap();
                for _ in 0..3 {
                    let (text, record) = generate::written(&mut rng, &provable.formulas);
                    let written: ClassicalSequent = text.parse().unwrap();
                    assert_eq!(written.writing(), &record[..], "{text}");
                    assert_eq!(fault(written.forest(), written.writing()), None);
                    assert_eq!(
                        written.sequent().to_string(),
                        one_sided.to_string(),
                        "{text}"
                    );
                    let printed = written.to_string();
                    let again: ClassicalSequent = printed.parse().unwrap();
                    assert_eq!(again, written, "{text} printed as {printed}");
                    assert_eq!(again.to_string(), printed, "{text}");
                    tried += 1;
                }
            }
        }
        assert_eq!(tried, 16 * 12 * 3);
    }
}
