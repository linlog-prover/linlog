// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The derivation view's public face: its options, its inferences and
//! the queries a renderer asks.

use super::build::unfold;
use crate::Error;
use crate::fragment::Mode;
use crate::limits::{Limits, Progress};
use crate::occurrences::instances::Framing;
use crate::occurrences::{Forest, Member, OccId, Reading};
use crate::proofs::check;
use crate::proofs::{Named, NodeId, Proof};

/// The index of an inference in a derivation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize), serde(transparent))]
pub struct InfId(u32);

impl InfId {
    /// Wraps a raw index.
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    /// Returns the raw index.
    pub const fn get(self) -> u32 {
        self.0
    }

    /// Returns the index as a `usize`, for indexing the inferences.
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

/// How a derivation is shown: the one value that every path which builds
/// a derivation takes, whatever it then draws or writes. The bounds that
/// keep a call from building what the machine cannot hold are the
/// [`Limits`] beside it: `derivation_bytes`, the most bytes a derivation
/// may be estimated to take ([`Size::bytes`](crate::Size::bytes)) and still be built (a proof
/// stores a shared subproof once and its derivation repeats it, and every
/// inference carries its whole sequent, so a derivation can be larger than
/// its proof by any factor), and `memory_bytes`, the most the making of
/// one may hold at once, every pass of the checker on the way and the
/// derivation itself by the same estimate.
///
/// # JSON
///
/// With the feature `serialize` an object `{"compact": "auto", "sides":
/// "auto", "classical": "one"}`, `compact` one of `"auto"`, `"always"` and
/// `"never"`, `sides` one of `"auto"`, `"one"` and `"two"`, `classical` one
/// of `"one"` and `"two"`; a key left out keeps its default, and an unknown
/// one is refused, as for every options value.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serialize", serde(default, deny_unknown_fields))]
pub struct ViewOptions {
    /// Whether a run of one structural rule is drawn as one inference.
    pub compact: Compact,
    /// Whether the derivation is one-sided or two-sided.
    pub sides: Sides,
    /// Whether a front end that holds a classical sequent as it was
    /// written ([`ClassicalSequent`](crate::ClassicalSequent)) draws its
    /// derivation one-sided or two-sided ([`TwoSided`](crate::TwoSided)),
    /// one-sided by default ([`DEFAULT_CLASSICAL`](Self::DEFAULT_CLASSICAL)).
    /// The library's derivations are one-sided whatever it says: the
    /// two-sided view is made from one and the record of the sequent.
    pub classical: ClassicalSides,
}

/// Whether a derivation of classical linear logic is drawn one-sided, as
/// the calculus is, or two-sided, as its sequent was written.
///
/// # JSON
///
/// With the feature `serialize`, `"one"` or `"two"`.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serialize", serde(rename_all = "lowercase"))]
pub enum ClassicalSides {
    /// One-sided: every formula right of `⊢`.
    #[default]
    One,
    /// Two-sided: every inference's formulas on the sides of `⊢` they
    /// were written on, the rules named by side, and a negation step where
    /// a negation was written.
    Two,
}

/// Whether a derivation is drawn one-sided, as classical linear logic
/// writes it, or two-sided, as intuitionistic linear logic does.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serialize", serde(rename_all = "lowercase"))]
pub enum Sides {
    /// Two-sided exactly when the proof was found in intuitionistic mode
    /// ([`Proof::mode`]), one-sided otherwise.
    #[default]
    Auto,
    /// One-sided.
    One,
    /// Two-sided, under the sequent's intuitionistic reading.
    Two,
}

/// Whether a derivation draws a run of one structural rule, such as the
/// weakenings of every unused `?` formula, as one inference labelled with
/// a star.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serialize", serde(rename_all = "lowercase"))]
pub enum Compact {
    /// Every rule as its own inference, unless the derivation would then
    /// pass a bound of the options and the compact one would not.
    #[default]
    Auto,
    /// Every run as one inference.
    Always,
    /// Every rule as its own inference.
    Never,
}

impl ViewOptions {
    /// The default view of a classical derivation: one-sided, as the
    /// calculus is.
    pub const DEFAULT_CLASSICAL: ClassicalSides = ClassicalSides::One;

    /// Returns the options with another [`classical`](Self::classical).
    #[must_use]
    pub const fn with_classical(self, classical: ClassicalSides) -> Self {
        Self { classical, ..self }
    }

    /// Returns the options with another [`compact`](Self::compact).
    #[must_use]
    pub const fn with_compact(self, compact: Compact) -> Self {
        Self { compact, ..self }
    }

    /// Returns the options with another [`sides`](Self::sides).
    #[must_use]
    pub const fn with_sides(self, sides: Sides) -> Self {
        Self { sides, ..self }
    }

    /// Returns whether a derivation of `proof` is two-sided.
    pub(crate) fn two_sided(&self, proof: &Proof) -> bool {
        match self.sides {
            Sides::One => false,
            Sides::Two => true,
            Sides::Auto => proof.mode().is_some_and(Mode::is_intuitionistic),
        }
    }
}

/// One inference of a derivation: the sequent it concludes, the rule, and
/// the inferences of its premises. Two inferences are equal when they
/// show the same step, whichever proof term's node they unfold
/// ([`node`](Self::node)).
#[non_exhaustive]
#[derive(Clone, Debug, Eq)]
pub struct Inference {
    /// The sequent concluded, as members in ascending order, one repeated
    /// as often as the sequent holds it.
    pub(crate) sequent: Vec<Member>,
    /// The rule applied, as the derivation names it.
    pub(crate) rule: Named,
    /// The position in `sequent` of the formula the rule introduces or
    /// removes: `None` for an axiom, whose sequent is its two literals, and
    /// for Mix.
    pub(crate) principal: Option<usize>,
    /// The premises, in the rule's order.
    pub(crate) premises: Vec<InfId>,
    /// How many applications of the rule the inference stands for: one,
    /// or more for a run of a structural rule that a compact view draws
    /// as one inference ([`Compact`]), whose principal formula is then
    /// that of the lowest application.
    pub(crate) times: u32,
    /// For a cut, the occurrence of the cut formula `A` it cuts, which
    /// its first premise holds, its second holding the dual.
    pub(crate) cut: Option<Member>,
    /// In a derivation of a proof term, the node whose rule the inference
    /// shows.
    pub(crate) node: Option<NodeId>,
}

impl PartialEq for Inference {
    /// Compares every field but the node, so that the same step unfolded
    /// from terms whose nodes are numbered apart, or from a session, which
    /// names no node, is one inference.
    fn eq(&self, other: &Self) -> bool {
        // A field added later joins the comparison here.
        let Self {
            sequent,
            rule,
            principal,
            premises,
            times,
            cut,
            ..
        } = self;
        *sequent == other.sequent
            && *rule == other.rule
            && *principal == other.principal
            && *premises == other.premises
            && *times == other.times
            && *cut == other.cut
    }
}

impl Inference {
    /// Returns the sequent concluded, as members in ascending order, one
    /// repeated as often as the sequent holds it. In the cyclic mode that
    /// is the sequent's cyclic order read from its first occurrence,
    /// every sequent of the derivation a rotation of the order its rule
    /// reads, so that no exchange stands between two of them.
    pub fn sequent(&self) -> &[Member] {
        &self.sequent
    }

    /// Returns the rule applied, as the derivation names it.
    pub const fn rule(&self) -> Named {
        self.rule
    }

    /// Returns the position in [`sequent`](Self::sequent) of the formula
    /// the rule introduces or removes: `None` for an axiom, whose sequent
    /// is its two literals, for Mix and for an open goal.
    pub const fn principal(&self) -> Option<usize> {
        self.principal
    }

    /// Returns the premises, in the rule's order.
    pub fn premises(&self) -> &[InfId] {
        &self.premises
    }

    /// Returns how many applications of the rule the inference stands for:
    /// one, or more for a run of a structural rule that a compact view
    /// draws as one inference ([`Compact`]), whose principal formula is then
    /// that of the lowest application.
    pub const fn times(&self) -> u32 {
        self.times
    }

    /// Returns, for a cut, the member of its cut formula `A`, which the
    /// first premise's sequent holds, the second holding its dual
    /// ([`Forest::dual`](crate::Forest::dual)); `None` for every other
    /// rule.
    pub const fn cut(&self) -> Option<Member> {
        self.cut
    }

    /// Returns, in a derivation of a proof term, the node of the term
    /// whose rule the inference shows, which is what turns a node into
    /// the inferences drawn for it: one or more, a node the term shares
    /// unfolding once for every place that reads it. `None` for an
    /// inference the derivation adds below a node (a weakening of a `?`
    /// formula the subproof does not use, a contraction of the `?`
    /// formulas both premises of a `⊗`, Mix or cut use) and in a
    /// session's derivation. A run of a compact view has its lowest
    /// application's node.
    pub const fn node(&self) -> Option<NodeId> {
        self.node
    }
}

/// A derivation in the standard sequent calculus: the tree of inferences a
/// proof term stands for, over the proof's forest, one-sided for classical
/// linear logic or two-sided for intuitionistic linear logic. Premises
/// precede their conclusion and the root is the last inference.
///
/// [`Display`](std::fmt::Display) draws the tree, see [`Proof`] for an example.
///
/// A derivation has no JSON form of its own: the proof's form and the
/// [`ViewOptions`] it was made with make it again.
#[derive(Clone, Debug)]
pub struct Derivation<'a> {
    /// The forest the sequents' occurrences index.
    forest: &'a Forest,
    /// The intuitionistic reading, for a two-sided derivation.
    reading: Option<Reading<'a>>,
    /// The inferences, premises before conclusions, the root last.
    inferences: Vec<Inference>,
    /// Whether the root concludes a goal of a proof rather than the
    /// sequent.
    of_goal: bool,
    /// The mode the proof is meant for, where it records one.
    #[cfg_attr(
        not(feature = "rocq"),
        expect(dead_code, reason = "only a certificate asks")
    )]
    mode: Option<Mode>,
    /// The instances of the proof's members past the forest, where it
    /// has them.
    framing: Option<Framing<'a>>,
}

impl<'a> Derivation<'a> {
    /// Unfolds a proof into the one-sided derivation of classical linear
    /// logic, unless it is larger than `view` allows or `stop` fires on
    /// the way, once per inference. The proof must be correct, and the
    /// unfolding fails as [`check`](Proof::check) would if it is not. Mode
    /// is not a question here: a derivation shows every rule the proof
    /// uses.
    pub(crate) fn new(
        proof: &'a Proof,
        view: &ViewOptions,
        limits: &Limits,
        stop: impl FnMut(Progress) -> bool,
    ) -> Result<Self, Error> {
        Self::build(proof, Self::ONE_SIDED, None, view, limits, stop)
    }

    /// The most inferences a derivation holds: as many as an [`InfId`]
    /// counts.
    pub const MOST: u64 = u32::MAX as u64;

    /// The rules of a one-sided derivation: every rule a proof can use.
    pub(crate) const ONE_SIDED: Mode = Mode::CLASSICAL.with_affine().with_mix();

    /// The rules of a two-sided derivation, weakening among them so that
    /// it shows where used.
    pub(crate) const TWO_SIDED: Mode = Mode::INTUITIONISTIC.with_affine();

    /// The rules of a two-sided derivation of `proof`: of the Lambek
    /// calculus with empty antecedents for a proof meant for an ordered
    /// mode, read in its order, and [`TWO_SIDED`](Self::TWO_SIDED) for
    /// any other.
    pub(crate) fn two_sided_mode(proof: &Proof) -> Mode {
        if proof.mode().is_some_and(Mode::is_ordered) {
            Mode::LAMBEK.with_empty_antecedents()
        } else {
            Self::TWO_SIDED
        }
    }

    /// Unfolds a proof into the two-sided derivation of intuitionistic
    /// linear logic, `Γ ⊢ A` at every inference with the intuitionistic
    /// rule names, unless it is larger than `view` allows or `stop` fires
    /// on the way. The proof must pass the checker in intuitionistic mode
    /// (affine or not), or for a proof meant for an ordered mode in the
    /// Lambek calculus with empty antecedents, read in its order, and the
    /// unfolding fails as it would otherwise.
    pub(crate) fn two_sided(
        proof: &'a Proof,
        view: &ViewOptions,
        limits: &Limits,
        stop: impl FnMut(Progress) -> bool,
    ) -> Result<Self, Error> {
        let mode = Self::two_sided_mode(proof);
        let reading = check::reading(proof, mode)?;
        Self::build(proof, mode, reading, view, limits, stop)
    }

    /// Checks the proof in `mode` and unfolds it, two-sided when a reading
    /// is given.
    fn build(
        proof: &'a Proof,
        mode: Mode,
        reading: Option<Reading<'a>>,
        view: &ViewOptions,
        limits: &Limits,
        mut stop: impl FnMut(Progress) -> bool,
    ) -> Result<Self, Error> {
        let conclusion = proof.conclusion();
        let inferences = unfold(
            proof,
            &conclusion,
            mode,
            reading.as_ref(),
            view,
            limits,
            &mut stop,
        )?;
        Ok(Self {
            forest: proof.forest(),
            reading,
            inferences,
            of_goal: proof.goal().is_some(),
            mode: proof.mode(),
            framing: proof
                .instances()
                .map(|table| Framing::new(&proof.forest().sequent().fo, table)),
        })
    }

    /// Wraps inferences that already have the derivation's shape: premises
    /// before conclusions, the root last, two-sided under a reading.
    #[cfg(feature = "interactive")]
    pub(crate) fn from_parts(
        forest: &'a Forest,
        reading: Option<Reading<'a>>,
        inferences: Vec<Inference>,
        mode: Mode,
    ) -> Self {
        debug_assert_ne!(inferences, []);
        Self {
            forest,
            reading,
            inferences,
            of_goal: false,
            mode: Some(mode),
            framing: None,
        }
    }

    /// Returns whether the root concludes a goal of its proof rather than
    /// the sequent.
    pub(crate) const fn is_of_goal(&self) -> bool {
        self.of_goal
    }

    /// Returns the mode the proof is meant for, where it records one.
    #[cfg(feature = "rocq")]
    pub(crate) const fn mode(&self) -> Option<Mode> {
        self.mode
    }

    /// Unfolds a proof whose root concludes `goal` rather than the roots,
    /// as the search from a goal returns it, into the inferences of its
    /// derivation, premises before conclusions and the root last, two-sided
    /// in intuitionistic mode, unless it is larger than `view` allows or
    /// `stop` fires on the way. Fails as the checker in `mode` would on a
    /// node that misapplies its rule and on a root that does not conclude
    /// the goal.
    #[cfg(feature = "interactive")]
    pub(crate) fn of_goal(
        proof: &'a Proof,
        goal: &[OccId],
        mode: Mode,
        view: &ViewOptions,
        limits: &Limits,
        stop: &mut dyn FnMut(Progress) -> bool,
    ) -> Result<Vec<Inference>, Error> {
        let reading = check::reading(proof, mode)?;
        // A graft is read rule by rule, never drawn compact.
        let view = view.with_compact(Compact::Never);
        let mut grafted = unfold(proof, goal, mode, reading.as_ref(), &view, limits, stop)?;
        // A session keeps no proof term whose nodes these would name.
        for inference in &mut grafted {
            inference.node = None;
        }
        Ok(grafted)
    }

    /// Returns the forest the sequents' members index.
    pub fn forest(&self) -> &'a Forest {
        self.forest
    }

    /// Returns the occurrence a member of the derivation stands for.
    pub fn occurrence(&self, member: Member) -> OccId {
        self.framing
            .as_ref()
            .map_or(member.occ(), |framing| framing.occurrence(member))
    }

    /// Returns the formula a member of the derivation stands for: in a
    /// two-sided derivation as its reading writes it, else as the
    /// one-sided sequent writes it; an instance with the terms of its
    /// frame for its loose bound variables.
    ///
    /// # Panics
    ///
    /// For a member past the forest that the proof's table does not
    /// number, which is no member of the derivation.
    pub fn formula(&self, member: Member) -> impl std::fmt::Display + 'a {
        if let Some(framing) = &self.framing
            && member.index() >= self.forest.len()
        {
            let mut text = String::new();
            crate::sequents::notation::TEXT
                .instance(
                    &mut text,
                    self.forest,
                    self.reading.as_ref(),
                    framing,
                    member,
                )
                .expect("writing into a string cannot fail");
            return text;
        }
        match &self.reading {
            Some(reading) => reading.formula(member.occ()).to_string(),
            None => self.forest.formula(member.occ()).to_string(),
        }
    }

    /// Returns the instances of the proof's members past the forest,
    /// where it has them.
    pub(crate) fn framing(&self) -> Option<&Framing<'a>> {
        self.framing.as_ref()
    }

    /// Returns the intuitionistic reading of a two-sided derivation, or
    /// `None` for a one-sided one.
    pub fn reading(&self) -> Option<&Reading<'a>> {
        self.reading.as_ref()
    }

    /// Returns every inference, premises before conclusions, the root last.
    pub fn inferences(&self) -> &[Inference] {
        &self.inferences
    }

    /// Returns the inference at `id`, which must belong to this derivation.
    ///
    /// # Panics
    ///
    /// For an id of another one, past its end.
    pub fn inference(&self, id: InfId) -> &Inference {
        &self.inferences[id.index()]
    }

    /// Returns the root: the inference that concludes the sequent.
    pub fn root(&self) -> InfId {
        // A derivation has at least one inference and [`MOST`](Self::MOST)
        // at most.
        InfId::new(self.inferences.len() as u32 - 1)
    }
}
