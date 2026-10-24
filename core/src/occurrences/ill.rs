// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! A sequent of intuitionistic linear logic as its author wrote it: its
//! hypotheses in the order written and exactly one goal, built from the
//! connectives of intuitionistic linear logic alone, lowered once to the
//! one-sided forest every engine, checker and view works on, with the side
//! of every occurrence recorded at the lowering.

use super::reading::{Reading, Side};
use super::{Forest, OccId};
use crate::sequents::Sequent;
use crate::{Error, Limits};
use std::fmt::{Display, Formatter, Result as FmtResult};

/// A sequent of intuitionistic linear logic, `Γ ⊢ A`: its hypotheses in
/// the order written and exactly one goal, every formula built from `⊗`,
/// `⊸`, `&`, `⊕`, `!`, `1`, `⊤`, `0` and atoms.
///
/// It is built three ways: from text by [`parse_within`](Self::parse_within)
/// (or [`str::parse`]), which reads intuitionistic syntax alone and
/// refuses the symbols of classical linear logic with a hint; from its
/// JSON form (with the feature `serialize`); and by
/// [`read_one_sided`](Self::read_one_sided), the explicit reading of a
/// one-sided sequent as intuitionistic, which takes the sides from the
/// one-sided formulas where they decide them and refuses what they do not.
/// Each lowers the sequent once to the [`Forest`] the engines take and
/// records the side of every occurrence; [`reading`](Self::reading) lends
/// the [`Reading`] over those sides, which prints the sequent two-sided as
/// it was written and is the reading [`Reading::new`] gives of the same
/// forest.
///
/// # JSON
///
/// With the feature `serialize`, the document `{"version": 5, "terms":
/// […], "atoms": […], "hypotheses": […], "goal": n}`: an arena of
/// intuitionistic formulas, each a term `{"V": atom}`, `"1"`, `"0"`,
/// `"⊤"`, `{"!": t}` or `{"⊗": [t, u]}` with `⊗`, `⊸`, `&` or `⊕`, every
/// operand an index below its own, the atom names, and the roots by index.
/// A tag of classical linear logic (`D`, `⅋`, `?`, `⊥`) is refused as an
/// unknown variant.
///
/// # Examples
///
#[cfg_attr(feature = "parse", doc = "```")]
#[cfg_attr(not(feature = "parse"), doc = "```ignore")]
/// use linlog::{Error, IllSequent, Side};
///
/// let sequent: IllSequent = "A -o B, A |- B".parse()?;
/// assert_eq!(sequent.to_string(), "A ⊸ B, A ⊢ B");
/// assert_eq!(sequent.sequent().to_string(), "⊢ A ⊗ ~B, ~A, B");
/// let reading = sequent.reading();
/// assert_eq!(reading.position(sequent.goal()), Side::Output);
///
/// let Err(Error::Parse(error)) = "~(A * ~B), A |- B".parse::<IllSequent>() else {
///     panic!("a negation is no intuitionistic syntax");
/// };
/// assert_eq!(error.span, 0..1);
/// assert!(error.hint.is_some());
/// # Ok::<(), Error>(())
/// ```
#[derive(Clone, Debug)]
pub struct IllSequent {
    /// The one-sided lowering, laid out as the forest the engines take.
    forest: Forest,
    /// Per occurrence, the side it stands on as written.
    position: Box<[Side]>,
    /// The goal: the last root of a sequent written two-sided.
    goal: OccId,
}

impl IllSequent {
    /// Lowers a sequent whose sides were written and whose formulas are
    /// intuitionistic as written: the roots after the first
    /// `antecedents` are the goal, of which there must be exactly one,
    /// and every occurrence is on its root's side but the antecedent of
    /// an implication, which is on the other. An implication is a `⅋` on
    /// the goal's side or a `⊗` on the hypotheses', whose left factor is
    /// the antecedent, since a written `A ⊸ B` is lowered to `A⊥ ⅋ B`
    /// and intuitionistic syntax writes no other `⅋`.
    #[cfg(any(feature = "parse", feature = "serialize"))]
    pub(crate) fn lowered(sequent: Sequent, limits: &Limits) -> Result<Self, Error> {
        use super::reading::ShapeError;
        use crate::sequents::Kind;
        let antecedents = sequent.antecedents().unwrap_or(0) as usize;
        let succedents = sequent.roots().len().saturating_sub(antecedents);
        if succedents != 1 {
            return Err(Error::NotIntuitionistic(ShapeError::Succedents {
                count: succedents,
            }));
        }
        let forest = Forest::from_owned(sequent, limits)?;
        let goal = forest.roots()[antecedents];
        let mut position = vec![Side::Input; forest.len()].into_boxed_slice();
        position[goal.index()] = Side::Output;
        // Preorder: a parent's side is known before its children's.
        for o in forest.ids() {
            let side = position[o.index()];
            let implication = matches!(
                (forest.kind(o), side),
                (Kind::Par, Side::Output) | (Kind::Tensor, Side::Input)
            );
            if let Some(left) = forest.left(o) {
                position[left.index()] = if implication { !side } else { side };
            }
            if let Some(right) = forest.right(o) {
                position[right.index()] = side;
            }
        }
        let lowered = Self {
            forest,
            position,
            goal,
        };
        debug_assert!(
            Reading::new(&lowered.forest).is_ok_and(|reading| reading.goal() == goal
                && lowered
                    .forest
                    .ids()
                    .all(|o| reading.position(o) == lowered.position[o.index()])),
            "the recorded sides are the reading's"
        );
        Ok(lowered)
    }

    /// Reads a one-sided sequent as intuitionistic, as [`Reading::new`]
    /// reads its forest: the goal is the formula written right of `⊢`
    /// where the sequent records its sides, else the one root that can be
    /// the goal, and every formula is read in the position the one-sided
    /// formula decides. `⊢ ~(A ⊗ ~B), ~A, B` (the text `~(A * ~B), A |-
    /// B`) is read as `A ⊸ B, A ⊢ B`.
    ///
    /// # Errors
    ///
    /// [`Error::NotIntuitionistic`] with the [`ShapeError`](crate::ShapeError) of a sequent
    /// that has no intuitionistic reading or more than one, and the
    /// errors of [`Forest::from_owned`] past the limits.
    pub fn read_one_sided(sequent: Sequent, limits: &Limits) -> Result<Self, Error> {
        let forest = Forest::from_owned(sequent, limits)?;
        let (position, goal) = Reading::new(&forest)
            .map_err(Error::NotIntuitionistic)?
            .into_parts();
        Ok(Self {
            forest,
            position,
            goal,
        })
    }

    /// Returns the one-sided sequent the engines take: the hypotheses
    /// negated, then the goal where the sequent was written two-sided,
    /// with `A ⊸ B` as `A⊥ ⅋ B`.
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

    /// Returns the goal's occurrence.
    pub fn goal(&self) -> OccId {
        self.goal
    }

    /// Returns the reading over the recorded sides, in constant time: the
    /// position of every occurrence, the formulas as written and the
    /// sequent printed two-sided.
    pub fn reading(&self) -> Reading<'_> {
        Reading::of_parts(&self.forest, &self.position, self.goal, false)
    }
}

impl PartialEq for IllSequent {
    /// Two sequents are equal when they are written alike: the same
    /// formulas on the same sides, however their arenas share subterms.
    fn eq(&self, other: &Self) -> bool {
        let (a, b) = (&self.forest, &other.forest);
        fn name(forest: &Forest, o: OccId) -> Option<&str> {
            forest.atom(o).map(|x| forest.sequent().atom_name(x))
        }
        a.len() == b.len()
            && a.roots() == b.roots()
            && self.goal == other.goal
            && self.position == other.position
            && a.ids()
                .all(|o| a.kind(o) == b.kind(o) && name(a, o) == name(b, o))
    }
}

impl Eq for IllSequent {}

impl Display for IllSequent {
    /// Writes the sequent two-sided, as it was written up to spacing and
    /// brackets: `A ⊸ B, A ⊢ B`.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.reading().fmt(f)
    }
}

#[cfg(feature = "parse")]
impl IllSequent {
    /// Parses a sequent of intuitionistic linear logic, `A -o B, A |- B`,
    /// in the syntax [`Sequent`] reads without the symbols of classical
    /// linear logic: a `~`, `^`, `⅋` (`par`, `|`), `?` or `⊥` (`bot`) is
    /// a parse error whose [`hint`](crate::ParseError::hint) says what to
    /// write instead. Exactly one formula stands right of `|-`; the
    /// hypotheses keep the order written. Refused at the first term past
    /// `limits.occurrences`, as [`Sequent::parse_within`] refuses.
    ///
    /// Needs the cargo feature `parse` (on by default).
    ///
    /// # Errors
    ///
    /// [`Error::Parse`] for text that is no intuitionistic sequent, at the
    /// first place that cannot go on one; [`Error::NotIntuitionistic`]
    /// with [`ShapeError::Succedents`](crate::ShapeError::Succedents) for another number of formulas
    /// right of `|-` than one; and the refusals of
    /// [`Sequent::parse_within`].
    pub fn parse_within(text: &str, limits: &Limits) -> Result<Self, Error> {
        let sequent = match Sequent::parse_within(text, limits) {
            Ok(sequent) => sequent,
            // A classical symbol before the place the parser stopped at
            // is the first thing wrong.
            Err(Error::Parse(error)) => {
                return Err(crate::parse::classical(&text[..error.span.start])
                    .map_or(Error::Parse(error), Error::from));
            }
            Err(error) => return Err(error),
        };
        if let Some(error) = crate::parse::classical(text) {
            return Err(error.into());
        }
        Self::lowered(sequent, limits)
    }
}

#[cfg(feature = "parse")]
impl std::str::FromStr for IllSequent {
    type Err = Error;

    /// Parses a sequent as [`IllSequent::parse_within`] does, within the
    /// default [`Limits`].
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse_within(s, &Limits::default())
    }
}

/// How a front end reads the text of a sequent in an intuitionistic mode.
///
/// # JSON
///
/// With the feature `serialize`, its name: `"strict"` or `"one-sided"`.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serialize",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "kebab-case")
)]
pub enum IllSyntax {
    /// Intuitionistic syntax alone ([`IllSequent::parse_within`]): a
    /// symbol of classical linear logic is refused with a hint.
    #[default]
    Strict,
    /// The syntax of classical linear logic, read as the one-sided
    /// sequent it lowers to and that read as intuitionistic
    /// ([`IllSequent::read_one_sided`]): `~(A * ~B), A |- B` is `A ⊸ B,
    /// A ⊢ B`.
    OneSided,
}

/// How a front end reads its input.
///
/// # JSON
///
/// With the feature `serialize`, an object with a key per field, a
/// missing one taking its default and a misspelt one refused.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serialize",
    derive(serde::Serialize, serde::Deserialize),
    serde(default, deny_unknown_fields)
)]
pub struct InputOptions {
    /// How the text of an intuitionistic sequent is read.
    pub intuitionistic: IllSyntax,
}

impl InputOptions {
    /// The default reading of an intuitionistic sequent's text: its own
    /// syntax alone.
    pub const DEFAULT_INTUITIONISTIC: IllSyntax = IllSyntax::Strict;

    /// Returns the options with another
    /// [`intuitionistic`](Self::intuitionistic).
    #[must_use]
    pub const fn with_intuitionistic(self, intuitionistic: IllSyntax) -> Self {
        Self { intuitionistic }
    }
}

impl Default for InputOptions {
    /// The defaults named by the constants.
    fn default() -> Self {
        Self {
            intuitionistic: Self::DEFAULT_INTUITIONISTIC,
        }
    }
}

#[cfg(all(test, feature = "parse"))]
mod tests {
    use super::*;
    use crate::errors::ParseError;
    use crate::occurrences::ShapeError;
    use crate::search::generate::{self, IllRules, Rng};
    use crate::sequents::notation::TEXT;
    use crate::{Mode, search};

    /// The text of generated provable intuitionistic sequents, of every
    /// rule set, `per` of each.
    fn generated(seed: u64, per: usize) -> Vec<String> {
        let mut texts = Vec::new();
        for (i, rules) in IllRules::ALL.into_iter().enumerate() {
            let mut rng = Rng::new(seed + i as u64);
            for _ in 0..per {
                let budget = 2 + rng.below(10);
                let ill = generate::ill(&mut rng, rules, 3, budget);
                texts.push(generate::two_sided(&ill.hypotheses, &ill.goal));
            }
        }
        texts
    }

    /// Returns the parse error of `text`, which the strict parser refuses.
    fn refused(text: &str) -> ParseError {
        match text.parse::<IllSequent>() {
            Err(Error::Parse(error)) => *error,
            other => panic!("{text:?}: {other:?}"),
        }
    }

    /// Every symbol of classical linear logic is refused where it stands,
    /// with a hint of what to write instead; a syntax error before it is
    /// reported first; names that only contain a keyword are atoms; and
    /// another number of goals than one is refused.
    #[test]
    fn the_strict_parser_refuses_classical_symbols() {
        for (text, at, hint) in [
            ("~(A * ~B), A |- B", 0..1, "no negation"),
            ("A^ |- B", 1..2, "no negation"),
            ("|- A par B", 5..8, "no par"),
            ("|- A ⅋ B", 5..8, "no par"),
            ("|- A | B", 5..6, "no par"),
            ("?A |- B", 0..1, "no ?"),
            ("|- bot", 3..6, "no bot"),
            ("⊥ |- A", 0..3, "no bot"),
        ] {
            let error = refused(text);
            assert_eq!(error.span, at, "{text:?}");
            assert!(
                error.hint.is_some_and(|h| h.contains(hint)),
                "{text:?}: {error}"
            );
            assert!(error.to_string().contains(hint), "{error}");
        }
        // The parser's own error comes first where it is earlier.
        let error = refused("A * , ~B |- C");
        assert_eq!((error.span, error.hint), (4..5, None));
        // The `~` before the parser's error is the first thing wrong.
        assert_eq!(refused("~A * , B |- C").span, 0..1);
        let sequent: IllSequent = "bottom, parity |- bottom ⊗ parity".parse().unwrap();
        assert_eq!(sequent.to_string(), "bottom, parity ⊢ bottom ⊗ parity");
        for (text, count) in [("A |- B, C", 2), ("A |-", 0)] {
            assert!(
                matches!(
                    text.parse::<IllSequent>(),
                    Err(Error::NotIntuitionistic(ShapeError::Succedents { count: c })) if c == count
                ),
                "{text:?}"
            );
        }
    }

    /// The explicit reading of a one-sided sequent reads what the
    /// one-sided formulas decide and refuses the rest.
    #[test]
    fn a_one_sided_sequent_is_read_explicitly() {
        let limits = Limits::default();
        let read = |text: &str| {
            let sequent: Sequent = text.parse().unwrap();
            IllSequent::read_one_sided(sequent, &limits)
        };
        assert_eq!(
            read("~(A * ~B), A |- B").unwrap().to_string(),
            "A ⊸ B, A ⊢ B"
        );
        assert!(matches!(
            read("|- A par B"),
            Err(Error::NotIntuitionistic(ShapeError::Formula { .. }))
        ));
        let mut unsided: Sequent = "|- 0, top".parse().unwrap();
        unsided.antecedents = None;
        assert!(matches!(
            IllSequent::read_one_sided(unsided, &limits),
            Err(Error::NotIntuitionistic(ShapeError::Undetermined { .. }))
        ));
    }

    /// The sides recorded at lowering are the reading's, on every
    /// sequent lowered: the generated ones and those whose units stand on
    /// either side, where the one-sided form alone does not say.
    #[test]
    fn the_recorded_sides_are_the_readings() {
        let units = [
            "top |- 0",
            "0 |- top",
            "top, 0 |- top & 0",
            "(top -o 0) -o top |- 0 + top",
            "!(0 -o top), 1 |- (1 -o top) * 0",
        ];
        let texts = generated(4000, 20);
        for text in units
            .iter()
            .copied()
            .chain(texts.iter().map(String::as_str))
        {
            let sequent: IllSequent = text.parse().unwrap_or_else(|e| panic!("{text:?}: {e}"));
            let reading = Reading::new(sequent.forest()).unwrap();
            assert_eq!(reading.goal(), sequent.goal(), "{text:?}");
            let recorded = sequent.reading();
            for o in sequent.forest().ids() {
                assert_eq!(
                    reading.position(o),
                    recorded.position(o),
                    "{text:?} at {o:?}"
                );
            }
        }
    }

    /// A parsed sequent prints back to itself: the print parses to the
    /// same sequent, sides included.
    #[test]
    fn a_parsed_sequent_prints_back_to_itself() {
        for text in generated(4100, 20) {
            let sequent: IllSequent = text.parse().unwrap_or_else(|e| panic!("{text:?}: {e}"));
            let printed = sequent.to_string();
            let back: IllSequent = printed
                .parse()
                .unwrap_or_else(|e| panic!("{text:?} printed as {printed:?}: {e}"));
            assert_eq!(back, sequent, "{text:?} printed as {printed:?}");
        }
    }

    /// Every inference of every proof found prints as an intuitionistic
    /// sequent, which parses back to the same formulas on the same sides.
    #[test]
    fn every_inference_prints_as_an_intuitionistic_sequent() {
        let options = search::Options::default();
        let mut inferences = 0;
        for text in generated(4200, 6) {
            let sequent: IllSequent = text.parse().unwrap();
            let outcome = search::prove(sequent.sequent(), Mode::INTUITIONISTIC, &options).unwrap();
            let Some(proof) = outcome.verdict.proof() else {
                continue;
            };
            let derivation = proof.derivation().unwrap();
            let reading = derivation
                .reading()
                .expect("an intuitionistic proof is two-sided");
            for inference in derivation.inferences() {
                let mut printed = String::new();
                TEXT.sequent(
                    &mut printed,
                    proof.forest(),
                    Some(reading),
                    inference.sequent(),
                    false,
                    false,
                )
                .unwrap();
                let back: IllSequent = printed
                    .parse()
                    .unwrap_or_else(|e| panic!("{text:?}: {printed:?}: {e}"));
                let formulas = |reading: &Reading, members: &mut dyn Iterator<Item = OccId>| {
                    members
                        .map(|o| reading.formula(o).to_string())
                        .collect::<Vec<_>>()
                };
                let written: Vec<OccId> = reading
                    .written(inference.sequent())
                    .map(|p| inference.sequent()[p].occ())
                    .collect();
                let (goals, hypotheses): (Vec<OccId>, Vec<OccId>) = written
                    .into_iter()
                    .partition(|&o| reading.position(o) == Side::Output);
                assert_eq!(goals.len(), 1, "{text:?}: {printed:?}");
                let read = back.reading();
                assert_eq!(
                    formulas(&read, &mut read.hypotheses()),
                    formulas(reading, &mut hypotheses.into_iter()),
                    "{text:?}: {printed:?}"
                );
                assert_eq!(
                    read.formula(read.goal()).to_string(),
                    reading.formula(goals[0]).to_string(),
                    "{text:?}: {printed:?}"
                );
                inferences += 1;
            }
        }
        assert!(inferences > 200, "{inferences} inferences");
    }

    /// The JSON form writes the sequent as its reading reads it and reads
    /// back the same sequent; a classical tag, an index out of bounds or
    /// out of order, and a level above the build's are refused.
    #[cfg(feature = "serialize")]
    #[test]
    fn the_json_form() {
        let sequent: IllSequent = "A -o B, top |- 0 & B".parse().unwrap();
        let json = serde_json::to_string(&sequent).unwrap();
        assert_eq!(
            json,
            r#"{"version":5,"terms":[{"V":0},{"V":1},{"⊸":[0,1]},"⊤","0",{"&":[4,1]}],"atoms":["A","B"],"hypotheses":[2,3],"goal":5}"#
        );
        let back: IllSequent = serde_json::from_str(&json).unwrap();
        assert_eq!(back, sequent);
        for text in generated(4300, 10) {
            let sequent: IllSequent = text.parse().unwrap();
            let json = serde_json::to_string(&sequent).unwrap();
            let back: IllSequent = serde_json::from_str(&json).unwrap();
            assert_eq!(back.to_string(), sequent.to_string(), "{json}");
            let reading = Reading::new(back.forest()).unwrap();
            assert!(
                back.forest()
                    .ids()
                    .all(|o| reading.position(o) == back.reading().position(o))
            );
        }
        let read = |json: &str| {
            let mut document = serde_json::Deserializer::from_str(json);
            crate::wire::upgrade::<IllSequent, _>(&mut document, &Limits::default())
        };
        for tag in [r#"{"D":0}"#, r#"{"⅋":[0,0]}"#, r#"{"?":0}"#, r#""⊥""#] {
            let json =
                format!(r#"{{"terms":[{{"V":0}},{tag}],"atoms":["A"],"hypotheses":[],"goal":1}}"#);
            assert!(
                matches!(read(&json), Err(Error::Json { message, .. }) if message.contains("unknown variant")),
                "{json}"
            );
        }
        assert!(matches!(
            read(r#"{"terms":[{"⊸":[1,1]},{"V":0}],"atoms":["A"],"hypotheses":[],"goal":0}"#),
            Err(Error::NotTopological { .. })
        ));
        assert!(matches!(
            read(r#"{"terms":[{"V":0}],"atoms":["A"],"hypotheses":[3],"goal":0}"#),
            Err(Error::IndexOutOfBounds { index: 3, .. })
        ));
        assert!(matches!(
            read(r#"{"version":6,"terms":[{"V":0}],"atoms":["A"],"hypotheses":[],"goal":0}"#),
            Err(Error::Version { found: 6, .. })
        ));
    }
}
