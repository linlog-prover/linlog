// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Ordinary propositional logic, classical, intuitionistic and minimal,
//! decided through its embeddings into linear logic: a sequent of
//! ordinary formulas ([`Sequent`]), its image
//! under a named [`Translation`]
//! ([`translate`], an
//! [`Image`] holding a linear
//! [`Sequent`](crate::Sequent) and the mode to prove it in), and the
//! linear proof read back as a derivation of LK or LJ
//! ([`Image::read_back`], a
//! [`Derivation`] that
//! [`Derivation::check`] checks by the
//! rules of the logic); [`decide`] does all of it
//! in one call.
//!
//! Classical logic goes into affine MALL without exponentials: the
//! one-sided sequent in negation normal form with `∧` as `&`, `∨` as `⅋`,
//! true as `⊤` and false as `⊥` is provable in affine mode exactly when
//! the sequent is classically valid, and the focused engine decides it
//! without a copy bound. Intuitionistic logic goes into intuitionistic
//! linear logic by one of three translations with exponentials; minimal
//! logic is the same with false translated as an atom. Deciding through
//! them is subject to the copy bound of the search, so an intuitionistic
//! sequent may be answered "unknown".
//!
#![cfg_attr(feature = "parse", doc = "```")]
#![cfg_attr(not(feature = "parse"), doc = "```ignore")]
//! use linlog::ordinary::{self, Logic, Sequent, Translation, Verdict, translate};
//! use linlog::{Limits, Options};
//!
//! let sequent: Sequent = "a -> b, b -> c |- a -> c".parse()?;
//! let image = translate(&sequent, Logic::Intuitionistic, Translation::CallByName)?;
//! assert_eq!(image.sequent().to_string(), "⊢ ?(!a ⊗ ~b), ?(!b ⊗ ~c), ?~a ⅋ c");
//! let options = ordinary::Options::default().with_logic(Logic::Intuitionistic);
//! let outcome =
//!     ordinary::decide(&sequent, &options, &Options::default(), &Limits::default(), |_| false)?;
//! let Verdict::Valid(derivation) = &outcome.verdict else {
//!     panic!("valid");
//! };
//! assert_eq!(derivation.inference(derivation.root()).rule().name(), "→R");
//! # Ok::<(), linlog::Error>(())
//! ```

/// Deciding an ordinary sequent in one call.
mod decide;
/// The derivations of LK and LJ, the read-back and its check.
mod derivation;
#[cfg(feature = "parse")]
mod parse;
/// The certificates over `Prop`.
#[cfg(feature = "rocq")]
pub(crate) mod rocq;
/// The translations into linear logic.
mod translate;

pub use decide::{Outcome, Verdict, decide};
pub use derivation::{Derivation, Inference, Rule, Side};
#[cfg(feature = "parse")]
pub use parse::{Problem, read_tptp};
pub use translate::{FALSE, Image, translate};

use crate::Error;
use crate::fragment::{Fragment, Mode};
use crate::hash::HashMap;
use crate::limits::{Refusal, Space};
use crate::sequents::{Visit, Walk};
use std::fmt::{Display, Formatter, Result as FmtResult};

/// An ordinary propositional logic.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serialize", serde(rename_all = "lowercase"))]
pub enum Logic {
    /// Classical logic, whose derivations are LK's.
    Classical,
    /// Intuitionistic logic, whose derivations are LJ's.
    #[default]
    Intuitionistic,
    /// Minimal logic: intuitionistic logic without ex falso, false being
    /// a proposition like any other.
    Minimal,
}

impl Logic {
    /// Returns the logic's name in lower case.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Classical => "classical",
            Self::Intuitionistic => "intuitionistic",
            Self::Minimal => "minimal",
        }
    }

    /// Returns the name of its sequent calculus: LK, or LJ for the other
    /// two.
    pub const fn calculus(self) -> &'static str {
        match self {
            Self::Classical => "LK",
            Self::Intuitionistic | Self::Minimal => "LJ",
        }
    }
}

impl Display for Logic {
    /// Writes the logic's [`name`](Self::name).
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(self.name())
    }
}

/// A translation of ordinary logic into linear logic, by its name.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub enum Translation {
    /// Classical logic into affine MALL: the one-sided sequent in negation
    /// normal form, `∧` as `&`, `∨` as `⅋`, true as `⊤`, false as `⊥`, an
    /// implication `A → B` as `¬A ∨ B` and `A ↔ B` as `(A → B) ∧ (B → A)`.
    #[cfg_attr(feature = "serialize", serde(rename = "affine"))]
    Affine,
    /// Girard's call-by-name translation into ILL: `A → B` as `!A ⊸ B`,
    /// `∧` as `&`, `A ∨ B` as `!A ⊕ !B`, `¬A` as `!A ⊸ 0`, true as `⊤`,
    /// false as `0`, every hypothesis under `!`.
    #[cfg_attr(feature = "serialize", serde(rename = "cbn"))]
    CallByName,
    /// Girard's call-by-value translation into ILL: an atom `a` as `!a`,
    /// `A → B` as `!(A ⊸ B)`, `∧` as `⊗`, `∨` as `⊕`, `¬A` as `!(A ⊸ 0)`,
    /// true as `1`, false as `0`.
    #[cfg_attr(feature = "serialize", serde(rename = "cbv"))]
    CallByValue,
    /// Liang and Miller's 0/1 translation into ILL, as the LLTP library
    /// states it: hypotheses by the negative translation `t₀` under `!`,
    /// the goal by the positive one `t₁`, with `t₀(A ∧ B) = !t₀A & !t₀B`,
    /// `t₀(A → B) = !t₁A ⊸ !t₀B`, `t₁(A ∧ B) = !(t₁A & t₁B)`,
    /// `t₁(A → B) = !(!t₀A ⊸ t₁B)`, `A ∨ B` as `!A ⊕ !B` in both, true as
    /// `⊤` and `1`, false as `0`, and `¬A` as `A → false`.
    #[cfg_attr(feature = "serialize", serde(rename = "01"))]
    ZeroOne,
}

impl Translation {
    /// Every translation.
    pub const ALL: [Self; 4] = [
        Self::Affine,
        Self::CallByName,
        Self::CallByValue,
        Self::ZeroOne,
    ];

    /// The translation that decides intuitionistic and minimal logic by
    /// default, chosen by a run of the ILTP library's propositional
    /// problems.
    pub const DEFAULT_INTUITIONISTIC: Self = Self::CallByName;

    /// Returns the translation's short name: `affine`, `cbn`, `cbv` or
    /// `01`, as the command and the JSON spell it.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Affine => "affine",
            Self::CallByName => "cbn",
            Self::CallByValue => "cbv",
            Self::ZeroOne => "01",
        }
    }

    /// Returns the translation the logic is decided with by default:
    /// [`Affine`](Self::Affine) for classical logic,
    /// [`DEFAULT_INTUITIONISTIC`](Self::DEFAULT_INTUITIONISTIC) for the
    /// others.
    pub const fn default_for(logic: Logic) -> Self {
        match logic {
            Logic::Classical => Self::Affine,
            Logic::Intuitionistic | Logic::Minimal => Self::DEFAULT_INTUITIONISTIC,
        }
    }

    /// Returns whether the translation decides the logic: the affine one
    /// classical logic, the others intuitionistic and minimal logic.
    pub const fn decides(self, logic: Logic) -> bool {
        matches!(
            (self, logic),
            (Self::Affine, Logic::Classical)
                | (
                    Self::CallByName | Self::CallByValue | Self::ZeroOne,
                    Logic::Intuitionistic | Logic::Minimal
                )
        )
    }

    /// Returns where the image lies: MALL in classical affine mode, or LL
    /// in intuitionistic mode.
    pub const fn target(self) -> Target {
        match self {
            Self::Affine => Target {
                fragment: Fragment::MALL,
                mode: Mode::CLASSICAL.with_affine(),
            },
            Self::CallByName | Self::CallByValue | Self::ZeroOne => Target {
                fragment: Fragment::LL,
                mode: Mode::INTUITIONISTIC,
            },
        }
    }
}

/// Where a translation's images lie: the largest fragment and the mode
/// they are decided in.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Target {
    /// The fragment.
    pub fragment: Fragment,
    /// The mode.
    pub mode: Mode,
}

impl Display for Target {
    /// Writes the fragment's name in the mode, after `affine` in classical
    /// affine mode: `affine MALL`, `ILL`.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        if self.mode.is_affine() && !self.mode.is_intuitionistic() {
            f.write_str("affine ")?;
        }
        f.write_str(self.fragment.name_in(self.mode))
    }
}

impl Display for Translation {
    /// Writes the translation's [`name`](Self::name).
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(self.name())
    }
}

/// How an ordinary sequent is decided: the logic, and the translation,
/// `None` for the logic's default ([`Translation::default_for`]).
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serialize", serde(default, deny_unknown_fields))]
pub struct Options {
    /// The logic.
    pub logic: Logic,
    /// The translation, or `None` for the logic's default.
    pub translation: Option<Translation>,
}

impl Options {
    /// Returns the options with the logic given.
    #[must_use]
    pub const fn with_logic(self, logic: Logic) -> Self {
        Self { logic, ..self }
    }

    /// Returns the options with the translation given, `None` for the
    /// logic's default.
    #[must_use]
    pub const fn with_translation(self, translation: Option<Translation>) -> Self {
        Self {
            translation,
            ..self
        }
    }

    /// Returns the translation these options choose.
    pub const fn translation(&self) -> Translation {
        match self.translation {
            Some(translation) => translation,
            None => Translation::default_for(self.logic),
        }
    }
}

/// The index of a formula in a [`Formulas`] arena.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(u32);

impl NodeId {
    /// Returns the index as a `usize`, for indexing the arena.
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

/// A node of an ordinary formula, which refers to its subformulas by
/// arena index.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Node {
    /// An atom, by its index among the arena's atom names.
    Atom(u32),
    /// `⊤`, true.
    True,
    /// `⊥`, false.
    False,
    /// `¬A`
    Not(NodeId),
    /// `A ∧ B`
    And(NodeId, NodeId),
    /// `A ∨ B`
    Or(NodeId, NodeId),
    /// `A → B`
    Implies(NodeId, NodeId),
    /// `A ↔ B`
    Iff(NodeId, NodeId),
}

impl Node {
    /// Returns the subformulas, in the order they are written.
    pub const fn operands(self) -> (Option<NodeId>, Option<NodeId>) {
        match self {
            Self::Atom(_) | Self::True | Self::False => (None, None),
            Self::Not(a) => (Some(a), None),
            Self::And(a, b) | Self::Or(a, b) | Self::Implies(a, b) | Self::Iff(a, b) => {
                (Some(a), Some(b))
            }
        }
    }
}

/// The formulas of ordinary sequents, as an arena in which every
/// subformula precedes the formulas built from it and equal formulas are
/// one node, so that two formulas are equal exactly when their ids are.
#[derive(Clone, Debug, Default)]
pub struct Formulas {
    /// The nodes, each after its subformulas.
    nodes: Vec<Node>,
    /// The atom names, by the index `Node::Atom` holds.
    atoms: Vec<String>,
    /// The index of every atom name.
    names: HashMap<String, u32>,
    /// The id of every node, for sharing.
    ids: HashMap<Node, NodeId>,
}

impl Formulas {
    /// The most nodes an arena holds: the ids are `u32`.
    pub const MOST: usize = u32::MAX as usize;

    /// Returns the node with the id given, which must belong to this arena.
    pub fn node(&self, id: NodeId) -> Node {
        self.nodes[id.index()]
    }

    /// Returns the number of nodes.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns whether the arena has no node.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Returns the name of the atom with the index given.
    pub fn atom_name(&self, atom: u32) -> &str {
        &self.atoms[atom as usize]
    }

    /// Returns the atom names, in the order of their indices.
    pub fn atom_names(&self) -> &[String] {
        &self.atoms
    }

    /// Returns the id of `node`, adding it unless the arena has it.
    ///
    /// # Errors
    ///
    /// [`Error::IndexOutOfBounds`] for a subformula that is no node of
    /// this arena (an id of another one) or an atom that is not one of its
    /// names, and [`Refusal::Index`] when the arena
    /// is full.
    pub fn add(&mut self, node: Node) -> Result<NodeId, Error> {
        if let Some(&id) = self.ids.get(&node) {
            return Ok(id);
        }
        let (a, b) = node.operands();
        for id in [a, b].into_iter().flatten() {
            self.holds(id)?;
        }
        if let Node::Atom(atom) = node
            && atom as usize >= self.atoms.len()
        {
            return Err(Error::IndexOutOfBounds {
                space: Space::Atom,
                index: atom as usize,
                len: self.atoms.len(),
            });
        }
        if self.nodes.len() >= Self::MOST {
            return Err(Error::Refused(Refusal::Index {
                what: Space::Formula,
                count: Self::MOST as u64 + 1,
                most: Self::MOST as u64,
            }));
        }
        let id = NodeId(self.nodes.len() as u32);
        self.nodes.push(node);
        self.ids.insert(node, id);
        Ok(id)
    }

    /// Fails unless `id` is a node of this arena.
    fn holds(&self, id: NodeId) -> Result<(), Error> {
        if id.index() < self.nodes.len() {
            Ok(())
        } else {
            Err(Error::IndexOutOfBounds {
                space: Space::Formula,
                index: id.index(),
                len: self.nodes.len(),
            })
        }
    }

    /// Returns the atom called `name`, adding it unless the arena has it.
    ///
    /// # Errors
    ///
    /// [`Refusal::Index`] when the arena is full.
    pub fn atom(&mut self, name: &str) -> Result<NodeId, Error> {
        let index = match self.names.get(name) {
            Some(&index) => index,
            None => {
                let index = self.atoms.len() as u32;
                self.atoms.push(name.to_owned());
                self.names.insert(name.to_owned(), index);
                index
            }
        };
        self.add(Node::Atom(index))
    }

    /// Returns the formula at `id` as a value that prints it.
    pub fn formula(&self, id: NodeId) -> Formula<'_> {
        Formula { formulas: self, id }
    }

    /// Writes the formula at `id` with the spellings given, every binary
    /// subformula in brackets, and the whole in brackets too if it is
    /// binary and `brackets` is set.
    pub(crate) fn write(
        &self,
        out: &mut String,
        id: NodeId,
        brackets: bool,
        symbols: &Symbols,
        atom: impl Fn(&mut String, u32),
    ) {
        for visit in Walk::new(id, brackets, |k| self.node(k).operands()) {
            match visit {
                Visit::Enter(k, nested) => match self.node(k) {
                    Node::Atom(a) => atom(out, a),
                    Node::True => out.push_str(symbols.truth),
                    Node::False => out.push_str(symbols.falsity),
                    Node::Not(_) => out.push_str(symbols.not),
                    _ if nested => out.push('('),
                    _ => {}
                },
                Visit::Between(k) => {
                    out.push(' ');
                    out.push_str(match self.node(k) {
                        Node::And(..) => symbols.and,
                        Node::Or(..) => symbols.or,
                        Node::Implies(..) => symbols.implies,
                        _ => symbols.iff,
                    });
                    out.push(' ');
                }
                Visit::Exit(k, nested) => {
                    if nested && self.node(k).operands().1.is_some() {
                        out.push(')');
                    }
                }
            }
        }
    }
}

/// How a target spells the connectives and constants of ordinary
/// formulas.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Symbols {
    /// `¬`, written directly before its operand.
    pub(crate) not: &'static str,
    /// `∧`
    pub(crate) and: &'static str,
    /// `∨`
    pub(crate) or: &'static str,
    /// `→`
    pub(crate) implies: &'static str,
    /// `↔`
    pub(crate) iff: &'static str,
    /// `⊤`
    pub(crate) truth: &'static str,
    /// `⊥`
    pub(crate) falsity: &'static str,
}

impl Symbols {
    /// The Unicode symbols, which `Display` writes.
    pub(crate) const UNICODE: Self = Self {
        not: "¬",
        and: "∧",
        or: "∨",
        implies: "→",
        iff: "↔",
        truth: "⊤",
        falsity: "⊥",
    };
}

/// A formula of an arena, as a value that prints it with the Unicode
/// symbols, every binary subformula in brackets.
#[derive(Clone, Copy, Debug)]
pub struct Formula<'a> {
    /// The arena.
    formulas: &'a Formulas,
    /// The formula.
    id: NodeId,
}

impl Display for Formula<'_> {
    /// Writes the formula: `(a ∧ b) → ¬c`.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let mut out = String::new();
        self.formulas
            .write(&mut out, self.id, false, &Symbols::UNICODE, |o, a| {
                o.push_str(self.formulas.atom_name(a))
            });
        f.write_str(&out)
    }
}

/// A sequent of ordinary propositional logic, `Γ ⊢ Δ`: formulas left of
/// the turnstile, the hypotheses, and right of it, read as their
/// disjunction classically and as the one goal intuitionistically, which
/// is false when there is none.
///
/// # Syntax
///
/// With the feature `parse`, `"…".parse::<ordinary::Sequent>()` reads a
/// sequent written `formulas |- formulas` (or `⊢`), each side a list
/// separated by commas and possibly empty; a text without a turnstile is
/// one formula to prove. From the loosest binding to the tightest, a
/// formula is built with `<->`/`↔` (to the right), `->`/`→` (to the
/// right), `\/`/`∨`, `/\`/`∧` (these to the left) and the prefix `~`/`¬`,
/// over the constants `true`/`⊤` and `false`/`⊥`, atoms and parentheses.
/// An atom is a Unicode identifier other than `true` and `false`. The
/// symbols of linear logic (`&`, `|`, `*`, `-o`, …) are not read here, so
/// that a linear connective is never taken for an ordinary one.
///
/// `Display` writes the Unicode symbols with every nested binary formula
/// in brackets, which this syntax reads back.
///
#[cfg_attr(feature = "parse", doc = "```")]
#[cfg_attr(not(feature = "parse"), doc = "```ignore")]
/// let sequent: linlog::ordinary::Sequent = "a /\\ b -> c |- ~c -> ~a \\/ ~b".parse()?;
/// assert_eq!(sequent.to_string(), "(a ∧ b) → c ⊢ ¬c → (¬a ∨ ¬b)");
/// # Ok::<(), linlog::Error>(())
/// ```
#[derive(Clone, Debug)]
pub struct Sequent {
    /// The formulas.
    formulas: Formulas,
    /// The hypotheses, in the order written.
    left: Vec<NodeId>,
    /// The formulas right of the turnstile, in the order written.
    right: Vec<NodeId>,
}

impl Sequent {
    /// Returns the sequent `left ⊢ right` over the formulas given.
    ///
    /// # Errors
    ///
    /// [`Error::IndexOutOfBounds`] for an id of the two lists that is no
    /// node of `formulas`.
    pub fn new(formulas: Formulas, left: Vec<NodeId>, right: Vec<NodeId>) -> Result<Self, Error> {
        for &id in left.iter().chain(&right) {
            formulas.holds(id)?;
        }
        Ok(Self {
            formulas,
            left,
            right,
        })
    }

    /// Returns the arena of the formulas.
    pub fn formulas(&self) -> &Formulas {
        &self.formulas
    }

    /// Returns the formulas left of the turnstile, in the order written.
    pub fn left(&self) -> &[NodeId] {
        &self.left
    }

    /// Returns the formulas right of the turnstile, in the order written.
    pub fn right(&self) -> &[NodeId] {
        &self.right
    }
}

impl Display for Sequent {
    /// Writes `Γ ⊢ Δ` with the Unicode symbols.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let mut out = String::new();
        write_sides(
            &mut out,
            &self.formulas,
            &self.left,
            &self.right,
            &Symbols::UNICODE,
            "⊢",
        );
        f.write_str(&out)
    }
}

/// Writes `left ⊢ right` with the turnstile and symbols given.
pub(crate) fn write_sides(
    out: &mut String,
    formulas: &Formulas,
    left: &[NodeId],
    right: &[NodeId],
    symbols: &Symbols,
    turnstile: &str,
) {
    for (i, &id) in left.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        formulas.write(out, id, false, symbols, |o, a| {
            o.push_str(formulas.atom_name(a))
        });
    }
    if !left.is_empty() {
        out.push(' ');
    }
    out.push_str(turnstile);
    for (i, &id) in right.iter().enumerate() {
        out.push_str(if i == 0 { " " } else { ", " });
        formulas.write(out, id, false, symbols, |o, a| {
            o.push_str(formulas.atom_name(a))
        });
    }
}

#[cfg(all(test, feature = "parse"))]
mod tests {
    use super::*;
    use crate::{Options as Search, Verdict, prove};

    /// Decides `text` in `logic` by `translation` with a copy bound of
    /// eight, and returns whether it was proved, with the derivation read
    /// back checked.
    fn decide(text: &str, logic: Logic, translation: Translation) -> Option<bool> {
        let sequent: Sequent = text.parse().unwrap();
        let image = translate(&sequent, logic, translation).unwrap();
        let search = Search::default().with_copies(Some(8));
        let outcome = prove(image.sequent(), image.mode(), &search).unwrap();
        match &outcome.verdict {
            Verdict::Proved(proof) => {
                let unbounded = crate::Limits::default();
                let derivation = image.read_back(proof, &unbounded, |_| false).unwrap();
                derivation
                    .check(&unbounded, |_| false)
                    .unwrap_or_else(|e| panic!("{text} ({translation}): {e}"));
                Some(true)
            }
            Verdict::Unprovable(_) => Some(false),
            Verdict::Unknown(_) => None,
        }
    }

    /// An arena refuses a subformula of another arena and an atom
    /// outside its names, and a sequent an id outside its arena.
    #[test]
    fn foreign_ids_are_refused() {
        let mut other = Formulas::default();
        let (a, b) = (other.atom("a").unwrap(), other.atom("b").unwrap());
        let and = other.add(Node::And(a, b)).unwrap();
        let mut formulas = Formulas::default();
        let x = formulas.atom("x").unwrap();
        let refused = |result: Result<_, Error>, space| matches!(result, Err(Error::IndexOutOfBounds { space: s, .. }) if s == space);
        assert!(refused(formulas.add(Node::Not(and)), Space::Formula));
        assert!(refused(formulas.add(Node::Atom(5)), Space::Atom));
        let sequent = Sequent::new(formulas.clone(), vec![x], vec![and]);
        assert!(refused(sequent.map(|_| x), Space::Formula));
        assert!(Sequent::new(formulas, vec![], vec![x]).is_ok());
    }

    /// In minimal logic false is an atom of its own, also beside an atom
    /// that a TPTP problem names `false`.
    #[test]
    fn false_is_no_atom_of_the_sequent() {
        let problem =
            read_tptp("fof(a, axiom, p). fof(b, axiom, ~p). fof(c, conjecture, false).").unwrap();
        for translation in [
            Translation::CallByName,
            Translation::CallByValue,
            Translation::ZeroOne,
        ] {
            let image = translate(&problem.sequent, Logic::Minimal, translation).unwrap();
            let outcome = prove(image.sequent(), image.mode(), &Search::default()).unwrap();
            assert!(
                !matches!(outcome.verdict, Verdict::Proved(_)),
                "{translation}"
            );
        }
    }

    /// Every translation decides the sequents of its logic as the logic
    /// does, and every proof reads back as a derivation its checker
    /// accepts: the connectives on both sides, nested negations, the
    /// constants, contraction, and the sequents that separate the logics.
    #[test]
    fn decides_and_reads_back() {
        // (sequent, classical, intuitionistic, minimal)
        let cases = [
            ("a -> a", true, true, true),
            ("a \\/ ~a", true, false, false),
            ("~~a -> a", true, false, false),
            ("a -> ~~a", true, true, true),
            ("((a -> b) -> a) -> a", true, false, false),
            ("false -> a", true, true, false),
            ("~a -> a -> b", true, true, false),
            ("~a -> a -> ~b", true, true, true),
            ("a /\\ b <-> b /\\ a", true, true, true),
            ("a \\/ b, ~a |- b", true, true, false),
            ("(a -> b) /\\ (a -> c) |- a -> b /\\ c", true, true, true),
            ("a -> b, b -> c |- a -> c", true, true, true),
            ("~(a \\/ b) <-> ~a /\\ ~b", true, true, true),
            ("~(a /\\ b) -> ~a \\/ ~b", true, false, false),
            ("true, a |- a /\\ true", true, true, true),
            ("a -> (a -> b) -> b", true, true, true),
            ("(a <-> b) -> (b <-> a)", true, true, true),
            ("a |- b", false, false, false),
            ("a \\/ b |- a /\\ b", false, false, false),
            ("~~(a \\/ ~a)", true, true, true),
        ];
        for (text, classical, intuitionistic, minimal) in cases {
            assert_eq!(
                decide(text, Logic::Classical, Translation::Affine),
                Some(classical),
                "{text} classically"
            );
            for translation in [
                Translation::CallByName,
                Translation::CallByValue,
                Translation::ZeroOne,
            ] {
                for (logic, valid) in [
                    (Logic::Intuitionistic, intuitionistic),
                    (Logic::Minimal, minimal),
                ] {
                    let verdict = decide(text, logic, translation);
                    assert!(
                        verdict.is_none() || verdict == Some(valid),
                        "{text} in {logic} logic by {translation}: {verdict:?}"
                    );
                    if valid {
                        assert_eq!(
                            verdict,
                            Some(true),
                            "{text} in {logic} logic by {translation}"
                        );
                    }
                }
            }
        }
    }
}
