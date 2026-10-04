// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! What every output that draws a derivation shares: the labels of the
//! rules, the shape of an open goal, and why a derivation was not written
//! whole.
//!
//! A label is written once per convention in a small markup that each
//! output sets in its own way: the connective and unit characters `⊗ ⅋ &
//! ⊕ ⊸ ! ? ⊤ ⊥ 0 1` are symbols, `_x` or `_{xy}` is a subscript, and every
//! other character is text set upright. `⊸L` is the upright `L` after
//! `⊸`, `&L_1` has the subscript `1`. A `*` is a symbol too, a superscript
//! where the output sets one: the mark of a run of a rule (`?w*`). The
//! connectives of ordinary logic `∧ ∨ → ¬ ↔` are symbols as well, for the
//! labels of LK and LJ (`→L`, `∧_{L1}`).
//!
//! Every output draws a derivation through the crate-private trait
//! `Drawn`, which the linear derivations and those of LK and LJ
//! implement.

use super::derivation::{Derivation, InfId, Rule};
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fmt::Write;
use thiserror::Error;

/// How the rules of a derivation are labelled.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serialize", serde(rename_all = "lowercase"))]
pub enum Labels {
    /// The connective with an upright `L` or `R` and the subscripts `1`
    /// and `2`: `⊸L`, `&L₁`, `?d`, as [`Rule::name`] spells them.
    #[default]
    Upright,
    /// The side and the letter of a structural rule as a subscript: `⊸_L`,
    /// `&_{L1}`, `?_d`.
    Subscript,
    /// No labels.
    Off,
    /// The user's labels in the markup of this module, by rule; a rule the
    /// table leaves out keeps its upright label.
    Table(BTreeMap<Rule, String>),
}

/// The labels of the upright convention, by rule in declaration order.
const UPRIGHT: [&str; Rule::ALL.len()] = [
    "ax", "⊗", "⅋", "1", "⊥", "&", "⊕_1", "⊕_2", "⊤", "!", "?d", "?c", "?w", "mix", "wk", "⊸L",
    "⊸R", "⊗L", "⊗R", "&L_1", "&L_2", "&R", "⊕L", "⊕R_1", "⊕R_2", "1L", "1R", "0L", "⊤R", "!L",
    "!R", "!c", "!w", "",
];

/// The labels of the subscript convention, by rule in declaration order.
const SUBSCRIPT: [&str; Rule::ALL.len()] = [
    "ax", "⊗", "⅋", "1", "⊥", "&", "⊕_1", "⊕_2", "⊤", "!", "?_d", "?_c", "?_w", "mix", "wk", "⊸_L",
    "⊸_R", "⊗_L", "⊗_R", "&_{L1}", "&_{L2}", "&_R", "⊕_L", "⊕_{R1}", "⊕_{R2}", "1_L", "1_R", "0_L",
    "⊤_R", "!_L", "!_R", "!_c", "!_w", "",
];

impl Labels {
    /// Returns the label of a rule in the markup of this module, or `None`
    /// where the rule has none: an open goal, or every rule when labels
    /// are off.
    pub fn markup(&self, rule: Rule) -> Option<&str> {
        let label = match self {
            Self::Upright => UPRIGHT[rule as usize],
            Self::Subscript => SUBSCRIPT[rule as usize],
            Self::Off => return None,
            Self::Table(table) => table
                .get(&rule)
                .map_or(UPRIGHT[rule as usize], String::as_str),
        };
        (rule != Rule::Open && !label.is_empty()).then_some(label)
    }
}

/// What the label of an inference that stands for a run of a structural
/// rule ends with: `?w*`.
pub(crate) const RUN: &str = "*";

/// One part of a label's markup.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Part<'a> {
    /// A connective or unit character.
    Symbol(char),
    /// Text set upright.
    Text(&'a str),
    /// A subscript, its text set upright.
    Sub(&'a str),
}

/// Whether a character of a label is a symbol.
fn symbol(c: char) -> bool {
    "⊗⅋&⊕⊸!?⊤⊥01*∧∨→¬↔".contains(c)
}

/// Returns the parts of a label's markup in order. A `_` at the end, or
/// a `{` that is never closed, takes the rest as its subscript.
pub(crate) fn parts(markup: &str) -> Vec<Part<'_>> {
    let mut parts = Vec::new();
    let mut rest = markup;
    while let Some(c) = rest.chars().next() {
        if c == '_' {
            let after = &rest[1..];
            let (sub, next) = match after.strip_prefix('{') {
                Some(group) => group.split_once('}').unwrap_or((group, "")),
                None => after.split_at(after.chars().next().map_or(0, char::len_utf8)),
            };
            parts.push(Part::Sub(sub));
            rest = next;
        } else if symbol(c) {
            parts.push(Part::Symbol(c));
            rest = &rest[c.len_utf8()..];
        } else {
            let end = rest
                .find(|c: char| c == '_' || symbol(c))
                .unwrap_or(rest.len());
            parts.push(Part::Text(&rest[..end]));
            rest = &rest[end..];
        }
    }
    parts
}

/// Returns a label's markup as plain Unicode text: subscript digits as
/// their subscript characters, every other part as it is.
pub(crate) fn plain(markup: &str) -> String {
    let mut out = String::new();
    for part in parts(markup) {
        match part {
            Part::Symbol(c) => out.push(c),
            Part::Text(text) => out.push_str(text),
            Part::Sub(text) => out.extend(text.chars().map(|c| match c.to_digit(10) {
                Some(d) => char::from_u32(0x2080 + d).unwrap(),
                None => c,
            })),
        }
    }
    out
}

/// What an output needs of a derivation to draw it: the tree, the rule
/// of every inference and its label, and its sequent in the text tree's
/// characters or in a target's notation. The linear derivations and those
/// of LK and LJ implement it, so that every output draws both.
pub(crate) trait Drawn {
    /// How many rules the calculus has: a rule is a number below it.
    const RULES: usize;

    /// The rule of an open goal, if the calculus has one.
    const OPEN: Option<usize>;

    /// Returns the label of a rule in the markup of this module under
    /// `labels`, or `None` where it has none: an open goal, or every rule
    /// when labels are off.
    fn markup(rule: usize, labels: &Labels) -> Option<&str>;

    /// Returns the number of inferences.
    fn len(&self) -> usize;

    /// Returns the root, the last inference.
    fn root(&self) -> InfId;

    /// Returns the premises of an inference, in the rule's order; each
    /// precedes it.
    fn premises(&self, id: InfId) -> &[InfId];

    /// Returns the rule of an inference, a number below [`RULES`](Self::RULES).
    fn rule(&self, id: InfId) -> usize;

    /// Returns the name of an inference's rule, as the reading of the
    /// steps says it.
    fn name(&self, id: InfId) -> &'static str;

    /// Returns how many applications of its rule an inference stands for.
    fn times(&self, id: InfId) -> u32;

    /// Writes the sequent an inference concludes as the text tree shows
    /// it, in Unicode.
    fn write_sequent(&self, out: &mut impl Write, id: InfId) -> std::fmt::Result;

    /// Writes the sequent an inference concludes in a target's notation,
    /// as [`Notation::sequent`](crate::export::notation::Notation::sequent)
    /// does with `aligned` and `marks`.
    #[cfg(any(feature = "latex", feature = "typst", feature = "svg"))]
    fn sequent(
        &self,
        out: &mut String,
        notation: &crate::export::notation::Notation,
        id: InfId,
        aligned: bool,
        marks: bool,
    );

    /// Returns the positions of the formulas of an inference's sequent in
    /// the order [`sequent`](Self::sequent) writes them, hypotheses first.
    #[cfg(feature = "svg")]
    fn positions(&self, id: InfId) -> Vec<usize>;

    /// Returns whether an inference is an open goal.
    fn is_open(&self, id: InfId) -> bool {
        Self::OPEN == Some(self.rule(id))
    }

    /// Returns the label of an inference as [`markup`](Self::markup)
    /// does, followed by [`RUN`] when it stands for a run of its rule.
    fn label<'l>(&self, id: InfId, labels: &'l Labels) -> Option<Cow<'l, str>> {
        let markup = Self::markup(self.rule(id), labels)?;
        Some(match self.times(id) {
            1 => Cow::Borrowed(markup),
            _ => Cow::Owned(format!("{markup}{RUN}")),
        })
    }
}

impl Drawn for Derivation<'_> {
    const RULES: usize = Rule::ALL.len();
    const OPEN: Option<usize> = Some(Rule::Open as usize);

    fn markup(rule: usize, labels: &Labels) -> Option<&str> {
        labels.markup(Rule::ALL[rule])
    }

    fn len(&self) -> usize {
        self.inferences().len()
    }

    fn root(&self) -> InfId {
        Derivation::root(self)
    }

    fn premises(&self, id: InfId) -> &[InfId] {
        &self.inference(id).premises
    }

    fn rule(&self, id: InfId) -> usize {
        self.inference(id).rule as usize
    }

    fn name(&self, id: InfId) -> &'static str {
        self.inference(id).rule.name()
    }

    fn times(&self, id: InfId) -> u32 {
        self.inference(id).times
    }

    fn write_sequent(&self, out: &mut impl Write, id: InfId) -> std::fmt::Result {
        super::fmt::write_sequent(
            out,
            self.forest(),
            self.reading(),
            &self.inference(id).sequent,
        )
    }

    #[cfg(any(feature = "latex", feature = "typst", feature = "svg"))]
    fn sequent(
        &self,
        out: &mut String,
        notation: &crate::export::notation::Notation,
        id: InfId,
        aligned: bool,
        marks: bool,
    ) {
        let sequent = &self.inference(id).sequent;
        notation.sequent(out, self.forest(), self.reading(), sequent, aligned, marks);
    }

    #[cfg(feature = "svg")]
    fn positions(&self, id: InfId) -> Vec<usize> {
        let sequent = &self.inference(id).sequent;
        let mut positions: Vec<usize> = (0..sequent.len()).collect();
        if let Some(reading) = self.reading() {
            positions.sort_by_key(|&p| reading.position(sequent[p]) == crate::Position::Output);
        }
        positions
    }
}

/// How an open goal of a proof in progress is drawn.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serialize", serde(rename_all = "lowercase"))]
pub enum OpenGoal {
    /// Its sequent under vertical dots.
    Dots,
    /// Its sequent alone.
    Bare,
    /// Its sequent under an inference line labelled with this text, in the
    /// markup of the labels: `?`, or a name.
    Mark(String),
    /// Its sequent under a dashed line, where the output can draw one, and
    /// under vertical dots where it cannot.
    Dashed,
}

/// Why a derivation was not written whole: what was written before stays
/// written.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
pub enum WriteError {
    /// The caller's stop condition fired.
    #[error("stopped")]
    Stopped,
    /// The writer failed.
    #[error("the writer failed")]
    Failed,
    /// The output has no form for the derivation; nothing was written.
    #[cfg(feature = "rocq")]
    #[error(transparent)]
    Unsupported(#[from] crate::export::rocq::Unsupported),
}

impl From<std::fmt::Error> for WriteError {
    /// The writer failed.
    fn from(_: std::fmt::Error) -> Self {
        Self::Failed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every rule's name reads back as the rule, and the upright labels as
    /// plain text are the names.
    #[test]
    fn names_round_trip() {
        for rule in Rule::ALL {
            assert_eq!(rule.name().parse::<Rule>(), Ok(rule), "{rule:?}");
            if rule != Rule::Open {
                let label = Labels::Upright.markup(rule).unwrap();
                assert_eq!(plain(label), rule.name(), "{rule:?}");
            }
        }
    }

    /// Markup splits into symbols, upright text and subscripts.
    #[test]
    fn markup_parts() {
        use Part::*;
        assert_eq!(
            parts("&L_1"),
            [Symbol('&'), Text("L"), Sub("1")],
            "a subscript of one character"
        );
        assert_eq!(parts("⊕_{R2}x"), [Symbol('⊕'), Sub("R2"), Text("x")]);
        assert_eq!(parts("ax"), [Text("ax")]);
        assert_eq!(parts("a_"), [Text("a"), Sub("")]);
    }
}
