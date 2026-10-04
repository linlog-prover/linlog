// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Typst: formulas in math mode, and derivations as proof trees of the
//! `curryst` package, whose `rule(name: …, premises…, conclusion)` nests
//! the premises inside their conclusion.
//!
//! The connectives are written as their Unicode characters (`⊗ ⅋ ⊕ ⊸ ⊢
//! ⊥ ⊤`), which Typst's math reads with their Unicode math class and which
//! no renaming of Typst's symbol names can break; `&` is escaped and set
//! as a binary operator, `?` set as an ordinary symbol so that no
//! punctuation space follows it, and `1` bold. A fragment is a sequent in
//! `$…$` or a `#prooftree(…)` call, for a document that imports `rule`
//! and `prooftree` from curryst [`CURRYST`]; a standalone document imports
//! them itself and sizes the page to its content. Neither chooses a font:
//! the output is set in the fonts of the document it goes into. curryst cannot align
//! turnstiles, so every sequent is centred. An open goal of a proof in
//! progress is its sequent under vertical dots, with no inference line.
//! curryst nests its layout at every level of the tree, so Typst refuses a
//! tree more than about eleven inferences high ("maximum show rule depth
//! exceeded"); the LaTeX export has no such limit.
//!
//! An atom named by one letter is written as it is, in math italic; any
//! other name is a string in `italic(…)`, with `"` and `\` escaped, since
//! Typst would read a longer name as a variable of its own. Typst is
//! Unicode throughout, so names beyond ASCII need nothing else.
//!
//! # Examples
//!
#![cfg_attr(feature = "parse", doc = "```")]
#![cfg_attr(not(feature = "parse"), doc = "```ignore")]
//! use linlog::export::typst;
//! use linlog::{Mode, Options, Sequent, Verdict, prove};
//!
//! let sequent: Sequent = "A, A -o B |- B".parse()?;
//! let options = typst::Options::default();
//! assert_eq!(typst::sequent(&sequent, &options), "$⊢ A^⊥, A ⊗ B^⊥, B$");
//!
//! let outcome = prove(&sequent, Mode::INTUITIONISTIC, &Options::default())?;
//! let Verdict::Proved(proof) = &outcome.verdict else {
//!     panic!("provable");
//! };
//! assert_eq!(
//!     typst::derivation(&proof.two_sided_derivation()?, &options),
//!     r#"#prooftree(
//!   rule(
//!     name: $⊸ upright(L)$,
//!     rule(name: $"ax"$, $A ⊢ A$),
//!     rule(name: $"ax"$, $B ⊢ B$),
//!     $A, A ⊸ B ⊢ B$,
//!   ),
//! )"#
//! );
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use super::Form;
use super::notation::{Notation, Step, flush, walk};
use crate::occurrences::Reading;
use crate::proofs::style::{Part, parts};
use crate::proofs::{Derivation, Labels, OpenGoal, Rule, WriteError};
use crate::sequents::Sequent;
use std::fmt::Write;

/// The version of curryst a standalone document imports, the one the
/// output is written for. The flake's `export` check compiles the output
/// with this version from nixpkgs, so the two change together.
pub const CURRYST: &str = "0.6.0";

/// The Typst spelling of formulas and sequents.
const NOTATION: Notation = Notation {
    tensor: "⊗",
    par: "⅋",
    with: r#"class("binary", \&)"#,
    plus: "⊕",
    lollipop: "⊸",
    bang: "!",
    quest: r#"class("normal", ?)"#,
    one: "bold(1)",
    bot: "⊥",
    top: "⊤",
    zero: "0",
    dual: "^⊥",
    turnstile: "⊢",
    align: "",
    atom,
};

/// Writes an atom's name in math mode.
fn atom(out: &mut String, name: &str) {
    let mut chars = name.chars();
    if let (Some(c), None) = (chars.next(), chars.next())
        && c.is_alphabetic()
    {
        out.push(c);
        return;
    }
    out.push_str("italic(\"");
    for c in name.chars() {
        if c == '"' || c == '\\' {
            out.push('\\');
        }
        out.push(c);
    }
    out.push_str("\")");
}

/// The page setup of a standalone document by default: as large as its
/// content.
pub const PAGE: &str = "#set page(width: auto, height: auto, margin: 5pt)";

/// What a user may vary in the Typst output.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serialize", serde(default, deny_unknown_fields))]
pub struct Options {
    /// A fragment to paste, or a document that compiles on its own.
    pub form: Form,
    /// The labels of the rules.
    pub labels: Labels,
    /// How an open goal is drawn: under `dots.v`, bare, as a leaf rule
    /// named by a mark, or under a dashed line.
    pub open: OpenGoal,
    /// The line of a standalone document that imports `prooftree` and
    /// `rule` from curryst.
    pub import: String,
    /// The page setup of a standalone document.
    pub page: String,
}

impl Default for Options {
    /// Returns a fragment with upright labels, an open goal under
    /// vertical dots, the import of curryst [`CURRYST`] and the page
    /// [`PAGE`].
    fn default() -> Self {
        Self {
            form: Form::Fragment,
            labels: Labels::Upright,
            open: OpenGoal::Dots,
            import: format!("#import \"@preview/curryst:{CURRYST}\": prooftree, rule"),
            page: PAGE.to_owned(),
        }
    }
}

/// Writes a label's markup in math mode: symbols as their characters,
/// one letter as `upright(L)`, longer text as a string, subscripts after
/// `_`, and a space before text that follows something.
fn label(out: &mut String, markup: &str) {
    let text = |out: &mut String, text: &str| {
        let mut chars = text.chars();
        if let (Some(c), None) = (chars.next(), chars.next())
            && c.is_alphabetic()
        {
            out.push_str("upright(");
            out.push(c);
            out.push(')');
            return;
        }
        out.push('"');
        for c in text.chars() {
            if c == '"' || c == '\\' {
                out.push('\\');
            }
            out.push(c);
        }
        out.push('"');
    };
    let start = out.len();
    for part in parts(markup) {
        match part {
            Part::Symbol(c) => out.push_str(match c {
                '&' => r"\&",
                '?' => NOTATION.quest,
                '1' => NOTATION.one,
                '⊗' => "⊗",
                '⅋' => "⅋",
                '⊕' => "⊕",
                '⊸' => "⊸",
                '!' => "!",
                '⊤' => "⊤",
                '⊥' => "⊥",
                '*' => "^*",
                _ => "0",
            }),
            Part::Text(t) => {
                if out.len() > start {
                    out.push(' ');
                }
                text(out, t);
            }
            Part::Sub(t) if t.len() == 1 && t.chars().all(|c| c.is_ascii_digit()) => {
                out.push('_');
                out.push_str(t);
            }
            Part::Sub(t) => {
                out.push_str("_(");
                text(out, t);
                out.push(')');
            }
        }
    }
}

/// Returns `math` in the options' form: as it is, or after the page
/// setup.
fn formed(math: String, options: &Options) -> String {
    match options.form {
        Form::Fragment => math,
        Form::Standalone => format!("{}\n\n{math}", options.page.trim_end()),
    }
}

/// Returns a sequent one-sided, `$⊢ A^⊥, A$`, in the options' form.
pub fn sequent(sequent: &Sequent, options: &Options) -> String {
    let mut out = String::from("$");
    NOTATION.one_sided(&mut out, sequent);
    out.push('$');
    formed(out, options)
}

/// Returns the sequent of an intuitionistic reading two-sided,
/// `$A, A ⊸ B ⊢ B$`, in the options' form.
pub fn two_sided(reading: &Reading, options: &Options) -> String {
    let forest = reading.forest();
    let mut out = String::from("$");
    NOTATION.sequent(
        &mut out,
        forest,
        Some(reading),
        forest.roots(),
        false,
        false,
    );
    out.push('$');
    formed(out, options)
}

/// Returns a derivation as a curryst `#prooftree(…)` call, two-sided if
/// the derivation is, in the options' form. A rule with premises spans
/// several lines, indented by its depth; a leaf is one line.
pub fn derivation(derivation: &Derivation, options: &Options) -> String {
    let mut out = String::new();
    write(derivation, options, &mut out, || false).expect("a string takes any text");
    out
}

/// Writes a derivation as [`derivation`] returns it into `out`, one
/// inference at a time, and asks `stop` after each.
pub fn write(
    derivation: &Derivation,
    options: &Options,
    out: &mut impl Write,
    mut stop: impl FnMut() -> bool,
) -> Result<(), WriteError> {
    let (forest, reading) = (derivation.forest(), derivation.reading());
    if options.form == Form::Standalone {
        write!(
            out,
            "{}\n{}\n\n",
            options.import.trim_end(),
            options.page.trim_end()
        )?;
    }
    out.write_str("#prooftree(\n")?;
    let indent = |out: &mut String, depth: usize| {
        for _ in 0..depth {
            out.push_str("  ");
        }
    };
    let conclusion = |out: &mut String, id| {
        out.push('$');
        let sequent = &derivation.inference(id).sequent;
        NOTATION.sequent(out, forest, reading, sequent, false, false);
        out.push('$');
    };
    let mut buffer = String::new();
    walk(derivation, |step| {
        match step {
            Step::Enter(id, depth) => {
                let inference = derivation.inference(id);
                indent(&mut buffer, depth + 1);
                // A grid of one column is as wide as the sequent, and so
                // is a line across it.
                let open = |out: &mut String, grid: &str| {
                    out.push_str(grid);
                    conclusion(out, id);
                    out.push_str("),\n");
                };
                match (inference.rule, &options.open) {
                    (Rule::Open, OpenGoal::Dots) => open(
                        &mut buffer,
                        "grid(align: center, row-gutter: 0.4em, $dots.v$, ",
                    ),
                    (Rule::Open, OpenGoal::Dashed) => open(
                        &mut buffer,
                        "grid(align: center, inset: (top: 0.3em), \
                         grid.hline(stroke: (dash: \"dashed\")), ",
                    ),
                    (Rule::Open, OpenGoal::Bare) => {
                        conclusion(&mut buffer, id);
                        buffer.push_str(",\n");
                    }
                    (Rule::Open, OpenGoal::Mark(mark)) => {
                        buffer.push_str("rule(name: $");
                        label(&mut buffer, mark);
                        buffer.push_str("$, ");
                        conclusion(&mut buffer, id);
                        buffer.push_str("),\n");
                    }
                    _ => {
                        let markup = options.labels.of(inference);
                        let markup = markup.as_deref();
                        let name = |out: &mut String| {
                            if let Some(markup) = markup {
                                out.push_str("name: $");
                                label(out, markup);
                                out.push('$');
                            }
                        };
                        buffer.push_str("rule(");
                        if inference.premises.is_empty() {
                            name(&mut buffer);
                            if markup.is_some() {
                                buffer.push_str(", ");
                            }
                            conclusion(&mut buffer, id);
                            buffer.push_str("),\n");
                        } else if markup.is_some() {
                            buffer.push('\n');
                            indent(&mut buffer, depth + 2);
                            name(&mut buffer);
                            buffer.push_str(",\n");
                        } else {
                            buffer.push('\n');
                        }
                    }
                }
            }
            Step::Exit(id, depth) => {
                if !derivation.inference(id).premises.is_empty() {
                    indent(&mut buffer, depth + 2);
                    conclusion(&mut buffer, id);
                    buffer.push_str(",\n");
                    indent(&mut buffer, depth + 1);
                    buffer.push_str("),\n");
                }
            }
        }
        flush(out, &mut buffer, &mut stop)
    })?;
    out.write_char(')')?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Returns an atom's name as the Typst export writes it.
    fn escaped(name: &str) -> String {
        let mut out = String::new();
        atom(&mut out, name);
        out
    }

    /// One letter stays as it is; every other name is an italic string
    /// with quotes and backslashes escaped.
    #[test]
    fn names_are_escaped() {
        for (name, expected) in [
            ("A", "A"),
            ("α", "α"),
            ("foo", r#"italic("foo")"#),
            ("x_1", r#"italic("x_1")"#),
            (r#"a"b\c"#, r#"italic("a\"b\\c")"#),
        ] {
            assert_eq!(escaped(name), expected, "{name:?}");
        }
    }
}
