// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! LaTeX: formulas in math mode with the connectives of the `cmll` and
//! `amssymb` packages (`\otimes`, `\parr`, `\with`, `\oplus`, `\multimap`,
//! `\oc`, `\wn`, `\mathbf{1}`, `\bot`, `\top`, `0`, `A^\bot`, `\vdash`),
//! and derivations as proof trees of the `ebproof` package, one statement
//! per inference in its postfix order.
//!
//! A fragment is a sequent in `$…$` or a `prooftree` environment, for a
//! document that loads `amssymb`, `cmll` and `ebproof`; a standalone
//! document is of the `standalone` class, which crops the page to the
//! content, and compiles with pdfLaTeX. Neither chooses a font: the output
//! is set in the fonts of the document it goes into. The turnstiles of two-sided
//! sequents are aligned in the tree with ebproof's `&` unless
//! [`Options::align`] is off; one-sided sequents are centred.
//!
//! An atom named by one ASCII letter is written as it is, in math italic,
//! and one named by a Greek letter as its command (`\alpha`, `\Gamma`, or
//! the Latin letter of a capital that looks like one); any other name goes
//! into `\mathit{…}`, with `\ { } $ # % & _` escaped by a backslash, `^`,
//! `~` and `\` written as text, a space kept, a Greek letter as its
//! command in braces, and the `‿` and `·` of the names `lltp::read` makes
//! as `\smallsmile` and `\cdotp`. So names of ASCII and Greek letters and
//! those of the LLTP library compile with pdfLaTeX; any other character
//! is written as it is and needs a Unicode engine such as LuaLaTeX with
//! `unicode-math`, or a declaration of the character for pdfLaTeX.
//!
//! # Examples
//!
//!
//! Needs the cargo feature `latex` (on by default).
#![cfg_attr(feature = "parse", doc = "```")]
#![cfg_attr(not(feature = "parse"), doc = "```ignore")]
//! use linlog::export::latex;
//! use linlog::{Mode, Options, Sequent, Verdict, prove};
//!
//! let sequent: Sequent = "A, A -o B |- B".parse()?;
//! let options = latex::Options::default();
//! assert_eq!(latex::sequent(&sequent, &options), r"$\vdash A^\bot, A \otimes B^\bot, B$");
//!
//! let outcome = prove(&sequent, Mode::INTUITIONISTIC, &Options::default())?;
//! let Verdict::Proved(proof) = &outcome.verdict else {
//!     panic!("provable");
//! };
//! assert_eq!(
//!     latex::derivation(&proof.two_sided_derivation()?, &options),
//!     r"\begin{prooftree}
//! \infer0[$\mathrm{ax}$]{A &\vdash A}
//! \infer0[$\mathrm{ax}$]{B &\vdash B}
//! \infer2[$\multimap\mathrm{L}$]{A, A \multimap B &\vdash B}
//! \end{prooftree}"
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

/// The LaTeX spelling of formulas and sequents.
const NOTATION: Notation = Notation {
    tensor: r"\otimes",
    par: r"\parr",
    with: r"\with",
    plus: r"\oplus",
    lollipop: r"\multimap",
    bang: r"\oc ",
    quest: r"\wn ",
    one: r"\mathbf{1}",
    bot: r"\bot",
    top: r"\top",
    zero: "0",
    dual: r"^\bot",
    turnstile: r"\vdash",
    align: "&",
    atom,
};

/// The preamble of a standalone document: `amssymb` for `\multimap`,
/// `cmll` for `\parr`, `\with`, `\oc` and `\wn`.
const PREAMBLE: &str = r"\documentclass[border=5pt]{standalone}
\usepackage{amssymb}
\usepackage{cmll}
";

/// Returns the math-mode spelling of a Greek letter: its command, or
/// the Latin letter of a capital that looks like one.
fn greek(c: char) -> Option<&'static str> {
    Some(match c {
        'α' => r"\alpha",
        'β' => r"\beta",
        'γ' => r"\gamma",
        'δ' => r"\delta",
        'ε' => r"\varepsilon",
        'ϵ' => r"\epsilon",
        'ζ' => r"\zeta",
        'η' => r"\eta",
        'θ' => r"\theta",
        'ϑ' => r"\vartheta",
        'ι' => r"\iota",
        'κ' => r"\kappa",
        'λ' => r"\lambda",
        'μ' => r"\mu",
        'ν' => r"\nu",
        'ξ' => r"\xi",
        'ο' => "o",
        'π' => r"\pi",
        'ϖ' => r"\varpi",
        'ρ' => r"\rho",
        'ϱ' => r"\varrho",
        'σ' => r"\sigma",
        'ς' => r"\varsigma",
        'τ' => r"\tau",
        'υ' => r"\upsilon",
        'φ' => r"\varphi",
        'ϕ' => r"\phi",
        'χ' => r"\chi",
        'ψ' => r"\psi",
        'ω' => r"\omega",
        'Γ' => r"\Gamma",
        'Δ' => r"\Delta",
        'Θ' => r"\Theta",
        'Λ' => r"\Lambda",
        'Ξ' => r"\Xi",
        'Π' => r"\Pi",
        'Σ' => r"\Sigma",
        'Υ' => r"\Upsilon",
        'Φ' => r"\Phi",
        'Ψ' => r"\Psi",
        'Ω' => r"\Omega",
        'Α' => "A",
        'Β' => "B",
        'Ε' => "E",
        'Ζ' => "Z",
        'Η' => "H",
        'Ι' => "I",
        'Κ' => "K",
        'Μ' => "M",
        'Ν' => "N",
        'Ο' => "O",
        'Ρ' => "P",
        'Τ' => "T",
        'Χ' => "X",
        _ => return None,
    })
}

/// Writes an atom's name in math mode.
fn atom(out: &mut String, name: &str) {
    let mut chars = name.chars();
    if let (Some(c), None) = (chars.next(), chars.next()) {
        if c.is_ascii_alphabetic() {
            out.push(c);
            return;
        }
        if let Some(letter) = greek(c) {
            out.push_str(letter);
            return;
        }
    }
    out.push_str(r"\mathit{");
    for c in name.chars() {
        escape(out, c);
    }
    out.push('}');
}

/// Writes a character of text in math mode, escaped.
fn escape(out: &mut String, c: char) {
    match c {
        '\\' => out.push_str(r"\mbox{\textbackslash}"),
        '{' | '}' | '$' | '#' | '%' | '&' | '_' => {
            out.push('\\');
            out.push(c);
        }
        '^' => out.push_str(r"\mbox{\textasciicircum}"),
        '~' => out.push_str(r"\mbox{\textasciitilde}"),
        ' ' => out.push_str(r"\ "),
        // `lltp::HYPHEN` and `lltp::DOT`, spelt out since `lltp` needs the
        // `parse` feature.
        '‿' => out.push_str(r"{\smallsmile}"),
        '·' => out.push_str(r"{\cdotp}"),
        // Braces keep a letter that follows from joining the command.
        c => match greek(c) {
            Some(letter) if letter.starts_with('\\') => {
                out.push('{');
                out.push_str(letter);
                out.push('}');
            }
            Some(letter) => out.push_str(letter),
            None => out.push(c),
        },
    }
}

/// What a user may vary in the LaTeX output.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serialize", serde(default, deny_unknown_fields))]
pub struct Options {
    /// A fragment to paste, or a document that compiles on its own.
    pub form: Form,
    /// The labels of the rules.
    pub labels: Labels,
    /// How an open goal is drawn: under `\vdots`, bare (`\hypo`), under a
    /// line with a mark, or under ebproof's dashed line.
    pub open: OpenGoal,
    /// Whether the turnstiles of a two-sided tree line up (ebproof's `&`).
    pub align: bool,
    /// The options of every `prooftree` environment, as ebproof reads
    /// them between brackets, or empty for none.
    pub ebproof: String,
    /// What a standalone document starts with, before
    /// `\begin{document}`, or `None` for a `standalone` class cropped to
    /// the content with `amssymb`, `cmll` and, for a tree, `ebproof`. A
    /// preamble of the user's must load those packages itself.
    pub preamble: Option<String>,
}

impl Default for Options {
    /// Returns a fragment with upright labels, an open goal under
    /// vertical dots, aligned turnstiles, no ebproof options and the
    /// `standalone` preamble.
    fn default() -> Self {
        Self {
            form: Form::Fragment,
            labels: Labels::Upright,
            open: OpenGoal::Dots,
            align: true,
            ebproof: String::new(),
            preamble: None,
        }
    }
}

/// Writes a label's markup in math mode: symbols as the connectives,
/// text in `\mathrm`, subscripts after `_`.
fn label(out: &mut String, markup: &str) {
    let text = |out: &mut String, text: &str| {
        out.push_str(r"\mathrm{");
        for c in text.chars() {
            escape(out, c);
        }
        out.push('}');
    };
    for part in parts(markup) {
        match part {
            Part::Symbol(c) => out.push_str(match c {
                '⊗' => NOTATION.tensor,
                '⅋' => NOTATION.par,
                '&' => NOTATION.with,
                '⊕' => NOTATION.plus,
                '⊸' => NOTATION.lollipop,
                '!' => r"\oc",
                '?' => r"\wn",
                '⊤' => NOTATION.top,
                '⊥' => NOTATION.bot,
                '1' => NOTATION.one,
                '*' => "^{*}",
                _ => NOTATION.zero,
            }),
            Part::Text(t) => text(out, t),
            Part::Sub(t) if t.len() == 1 && t.chars().all(|c| c.is_ascii_digit()) => {
                out.push('_');
                out.push_str(t);
            }
            Part::Sub(t) => {
                out.push_str("_{");
                text(out, t);
                out.push('}');
            }
        }
    }
}

/// Writes the start of a standalone document, with `ebproof` loaded if
/// `tree` is set.
fn begin(out: &mut impl Write, options: &Options, tree: bool) -> std::fmt::Result {
    match &options.preamble {
        Some(preamble) => writeln!(out, "{}", preamble.trim_end())?,
        None => {
            out.write_str(PREAMBLE)?;
            if tree {
                out.write_str("\\usepackage{ebproof}\n")?;
            }
        }
    }
    out.write_str("\\begin{document}\n")
}

/// Returns `math` in the options' form: as it is, or in a standalone
/// document.
fn formed(math: String, options: &Options) -> String {
    match options.form {
        Form::Fragment => math,
        Form::Standalone => {
            let mut out = String::new();
            begin(&mut out, options, false).expect("a string takes any text");
            out.push_str(&math);
            out.push_str("\n\\end{document}");
            out
        }
    }
}

/// Returns a sequent one-sided, `$\vdash A^\bot, A$`, in the options' form.
pub fn sequent(sequent: &Sequent, options: &Options) -> String {
    let mut out = String::from("$");
    NOTATION.one_sided(&mut out, sequent);
    out.push('$');
    formed(out, options)
}

/// Returns the sequent of an intuitionistic reading two-sided,
/// `$A, A \multimap B \vdash B$`, in the options' form.
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

/// Returns a derivation as an ebproof `prooftree` environment, two-sided
/// if the derivation is, in the options' form.
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
    let standalone = options.form == Form::Standalone;
    if standalone {
        begin(out, options, true)?;
    }
    out.write_str("\\begin{prooftree}")?;
    if !options.ebproof.is_empty() {
        write!(out, "[{}]", options.ebproof)?;
    }
    out.write_char('\n')?;
    let mut buffer = String::new();
    walk(derivation, |step| {
        let Step::Exit(id, _) = step else {
            return Ok(());
        };
        let inference = derivation.inference(id);
        let markup = options.labels.of(inference);
        let markup = markup.as_deref();
        match (inference.rule, &options.open) {
            (Rule::Open, OpenGoal::Dots) => buffer.push_str("\\hypo{\\vdots}\n\\infer[no rule]1"),
            (Rule::Open, OpenGoal::Bare) => buffer.push_str("\\hypo"),
            (Rule::Open, OpenGoal::Dashed) => buffer.push_str("\\infer[dashed]0"),
            (Rule::Open, OpenGoal::Mark(mark)) => {
                buffer.push_str("\\infer0[$");
                label(&mut buffer, mark);
                buffer.push_str("$]");
            }
            _ => {
                buffer.push_str("\\infer");
                buffer.push_str(&inference.premises.len().to_string());
                if let Some(markup) = markup {
                    buffer.push_str("[$");
                    label(&mut buffer, markup);
                    buffer.push_str("$]");
                }
            }
        }
        buffer.push('{');
        NOTATION.sequent(
            &mut buffer,
            forest,
            reading,
            &inference.sequent,
            options.align,
            false,
        );
        buffer.push_str("}\n");
        flush(out, &mut buffer, &mut stop)
    })?;
    out.write_str("\\end{prooftree}")?;
    if standalone {
        out.write_str("\n\\end{document}")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Returns an atom's name as the LaTeX export writes it.
    fn escaped(name: &str) -> String {
        let mut out = String::new();
        atom(&mut out, name);
        out
    }

    /// One ASCII letter stays as it is and one Greek letter is its
    /// command; every other name is set in `\mathit` with the special
    /// characters escaped and Greek letters and `lltp`'s marks as commands
    /// in braces.
    #[test]
    fn names_are_escaped() {
        for (name, expected) in [
            ("A", "A"),
            ("p", "p"),
            ("foo", r"\mathit{foo}"),
            ("x_1", r"\mathit{x\_1}"),
            ("α", r"\alpha"),
            ("Γ", r"\Gamma"),
            ("Α", "A"),
            ("αβ", r"\mathit{{\alpha}{\beta}}"),
            ("Οx", r"\mathit{Ox}"),
            ("P‿a·b", r"\mathit{P{\smallsmile}a{\cdotp}b}"),
            ("é", r"\mathit{é}"),
            (
                r"a{b}$#%&^~\ c",
                r"\mathit{a\{b\}\$\#\%\&\mbox{\textasciicircum}\mbox{\textasciitilde}\mbox{\textbackslash}\ c}",
            ),
        ] {
            assert_eq!(escaped(name), expected, "{name:?}");
        }
    }
}
