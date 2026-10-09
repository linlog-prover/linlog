// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Typst: formulas in math mode, and derivations as proof trees, of the
//! `curryst` package, whose `rule(name: …, premises…, conclusion)` nests
//! the premises inside their conclusion, or of linlog's own layout.
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
//! tree with two premises on a branch more than [`CURRYST_HEIGHT`]
//! inferences high ("maximum show rule depth exceeded"). A higher tree is
//! therefore written by default in linlog's own layout ([`Layout`]): a
//! `#context` block that lists the inferences and measures, lays out and
//! places them in one box, the SVG drawing's layout with Typst's
//! measurements, so that no height reaches a limit and no package is
//! needed.
//!
//! An atom named by one letter is written as it is, in math italic; any
//! other name is a string in `italic(…)`, with `"` and `\` escaped, since
//! Typst would read a longer name as a variable of its own. Typst is
//! Unicode throughout, so names beyond ASCII need nothing else.
//!
//! # Examples
//!
//!
//! Needs the cargo feature `typst` (on by default).
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
use crate::Error;
use crate::occurrences::Reading;
use crate::ordinary::{self, Symbols};
use crate::proofs::style::{Drawn, Part, parts};
use crate::proofs::{Derivation, Labels, OpenGoal};
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
    ordinary: Symbols::UNICODE,
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

/// The highest tree, in inferences on its longest branch, that curryst
/// [`CURRYST`] sets: Typst refuses a higher one with two premises
/// anywhere on the branch ("maximum show rule depth exceeded").
pub const CURRYST_HEIGHT: usize = 9;

/// Which code sets a Typst proof tree.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serialize", serde(rename_all = "lowercase"))]
pub enum Layout {
    /// curryst for a tree of at most [`CURRYST_HEIGHT`] inferences on
    /// its longest branch, linlog's own layout for a higher one.
    #[default]
    Auto,
    /// curryst's `prooftree` and `rule`, which a document imports.
    Curryst,
    /// Typst code of linlog's own in the output, which measures every
    /// sequent and places it, with no package and no limit on the height.
    Linlog,
}

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
    /// Which code sets the tree.
    pub layout: Layout,
    /// The least space between two premises side by side in linlog's own
    /// layout, a Typst length.
    pub premise_gap: String,
    /// The space between an inference line and its rule's label in
    /// linlog's own layout, a Typst length.
    pub label_gap: String,
    /// The least height of the space an inference line stands in, between
    /// its premises and its conclusion, in linlog's own layout, a Typst
    /// length; a taller label makes it taller.
    pub band: String,
    /// The thickness of an inference line in linlog's own layout, a Typst
    /// length.
    pub stroke: String,
}

impl Default for Options {
    /// Returns a fragment with upright labels, an open goal under
    /// vertical dots, the import of curryst [`CURRYST`], the page
    /// [`PAGE`], curryst up to its height and linlog's own layout above
    /// it, with curryst's spacing and line.
    fn default() -> Self {
        Self {
            form: Form::Fragment,
            labels: Labels::Upright,
            open: OpenGoal::Dots,
            import: format!("#import \"@preview/curryst:{CURRYST}\": prooftree, rule"),
            page: PAGE.to_owned(),
            layout: Layout::Auto,
            premise_gap: "1.5em".to_owned(),
            label_gap: "0.2em".to_owned(),
            band: "0.8em".to_owned(),
            stroke: "0.05em".to_owned(),
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
                '∧' => "∧",
                '∨' => "∨",
                '→' => "→",
                '¬' => "¬",
                '↔' => "↔",
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
        &forest.root_members(),
        false,
        false,
    );
    out.push('$');
    formed(out, options)
}

/// Returns a derivation as a proof tree, two-sided if the derivation is,
/// in the options' form: a curryst `#prooftree(…)` call, in which a rule
/// with premises spans several lines, indented by its depth, and a leaf is
/// one line; or a `#context` block of linlog's own layout
/// ([`Options::layout`]), the inferences listed first, from the root in
/// preorder, one line each, then the code that sets them.
pub fn derivation(derivation: &Derivation, options: &Options) -> String {
    let mut out = String::new();
    write(derivation, options, &mut out, |_| false).expect("a string takes any text");
    out
}

/// Writes a derivation as [`derivation`] returns it into `out`, one
/// inference at a time, and asks `stop` after each.
pub fn write(
    derivation: &Derivation,
    options: &Options,
    out: &mut impl Write,
    stop: impl FnMut(crate::limits::Progress) -> bool,
) -> Result<(), Error> {
    tree(
        derivation,
        options,
        out,
        crate::limits::counting(stop, crate::limits::Phase::Write),
    )
}

/// Writes a derivation of LK or LJ as a proof tree into `out`, two-sided,
/// in the options' form and layout, one inference at a time, and asks
/// `stop` after each: the connectives and constants are their Unicode
/// characters `¬ ∧ ∨ → ↔ ⊤ ⊥`, and the labels those of
/// [`ordinary::Rule`] (a label table of [`Labels::Table`] is keyed by the
/// linear rules, so it leaves them upright).
///
/// Needs the cargo feature `typst` (on by default).
pub fn ordinary(
    derivation: &ordinary::Derivation,
    options: &Options,
    out: &mut impl Write,
    stop: impl FnMut(crate::limits::Progress) -> bool,
) -> Result<(), Error> {
    tree(
        derivation,
        options,
        out,
        crate::limits::counting(stop, crate::limits::Phase::Write),
    )
}

/// Writes any derivation as [`write()`] does, in the layout the options
/// choose.
fn tree<T: Drawn>(
    derivation: &T,
    options: &Options,
    out: &mut impl Write,
    stop: impl FnMut() -> bool,
) -> Result<(), Error> {
    let own = match options.layout {
        Layout::Curryst => false,
        Layout::Linlog => true,
        Layout::Auto => height(derivation) > CURRYST_HEIGHT,
    };
    if own {
        laid_out(derivation, options, out, stop)
    } else {
        curryst(derivation, options, out, stop)
    }
}

/// Returns the inferences on the longest branch of a derivation.
fn height<T: Drawn>(derivation: &T) -> usize {
    // Premises come before their conclusions, the root last.
    let mut heights = vec![0; derivation.len()];
    for i in 0..heights.len() {
        let premises = derivation.premises(crate::InfId::new(i as u32));
        let above = premises.iter().map(|p| heights[p.index()]);
        heights[i] = 1 + above.max().unwrap_or(0);
    }
    heights.last().copied().unwrap_or(0)
}

/// The Typst code of linlog's own layout, which follows the inferences
/// (`nodes`: premises, label and conclusion, from the root in preorder;
/// `none` premises for an open goal drawn as `open` says) and the spacing
/// in a `#context` block. It measures every sequent and label, lays the
/// tree out as the SVG drawing does (premises side by side `gap` apart,
/// their conclusions centred over the conclusion, the line spanning both
/// with the label after it) and places every piece in one box: nothing is
/// nested per level, so no height reaches a limit of Typst's. A sequent's
/// ascent is its height less its depth, which a strut of `tall` above it
/// reveals, so that the sequents of a row share a baseline.
const LAYOUT: &str = r#"  let n = nodes.len()
  let tall = (3em).to-absolute()
  let metrics(c) = {
    let size = measure(box(c))
    let low = measure(box[#box(height: tall, width: 0pt)#c]).height - tall
    (size.width, size.height - low, low)
  }
  let (dots-width, dots-up, dots-down) = metrics(dots)
  let (width, up, down, wide, left, offset) = ((0pt,) * n,) * 6
  let bar = (none,) * n
  let names = (none,) * n
  let stack = ()
  for i in range(n - 1, -1, step: -1) {
    let (k, name, c) = nodes.at(i)
    let (w, a, d) = metrics(c)
    width.at(i) = w
    up.at(i) = a
    down.at(i) = d
    if k == none {
      let reach = if open == "dots" { dots-width } else { 0pt }
      wide.at(i) = calc.max(w, reach)
      left.at(i) = calc.max(reach - w, 0pt) / 2
      if open == "dashed" { bar.at(i) = (left.at(i), left.at(i) + w) }
      stack.push(i)
      continue
    }
    let kids = ()
    for _ in range(k) { kids.push(stack.pop()) }
    let x = 0pt
    let span = none
    for p in kids {
      offset.at(p) = x
      let s = x + left.at(p)
      span = (if span == none { s } else { span.at(0 ) }, s + width.at(p))
      x += wide.at(p) + gap
    }
    let (l, b) = if span == none { (0pt, (0pt, w)) } else {
      let l = (span.at(0) + span.at(1) - w) / 2
      (l, (calc.min(span.at(0), l), calc.max(span.at(1), l + w)))
    }
    let shift = calc.max(-l, 0pt)
    for p in kids { offset.at(p) += shift }
    let named = 0pt
    if name != none {
      let size = measure(box(name))
      names.at(i) = (size.width, size.height)
      named = name-gap + size.width
    }
    let row = if k == 0 { 0pt } else { x - gap }
    wide.at(i) = calc.max(row, b.at(1) + named) + shift
    left.at(i) = l + shift
    bar.at(i) = (b.at(0) + shift, b.at(1) + shift)
    stack.push(i)
  }
  let (x, depth) = ((0pt,) * n, (0,) * n)
  let parents = ()
  for i in range(n) {
    if parents.len() > 0 {
      let (p, r) = parents.last()
      x.at(i) = x.at(p) + offset.at(i)
      depth.at(i) = depth.at(p) + 1
      if r == 1 { let _ = parents.pop() } else { parents.at(-1) = (p, r - 1) }
    }
    let k = nodes.at(i).at(0)
    if k != none and k > 0 { parents.push((i, k)) }
  }
  let rows = calc.max(..depth) + 2
  let (row-up, row-down, between) = ((0pt,) * rows,) * 3
  for i in range(n) {
    let j = depth.at(i)
    row-up.at(j) = calc.max(row-up.at(j), up.at(i))
    row-down.at(j) = calc.max(row-down.at(j), down.at(i))
    if bar.at(i) != none {
      let tall = if names.at(i) == none { 0pt } else { names.at(i).at(1) }
      between.at(j) = calc.max(between.at(j), band, tall)
    }
    if nodes.at(i).at(0) == none and open == "dots" {
      between.at(j) = calc.max(between.at(j), band / 2)
      row-up.at(j + 1) = calc.max(row-up.at(j + 1), dots-up)
      row-down.at(j + 1) = calc.max(row-down.at(j + 1), dots-down)
    }
  }
  let base = (0pt,) * rows
  base.at(rows - 1) = row-up.at(rows - 1)
  for j in range(rows - 2, -1, step: -1) {
    base.at(j) = base.at(j + 1) + row-down.at(j + 1) + between.at(j) + row-up.at(j)
  }
  box(width: wide.at(0), height: base.at(0) + row-down.at(0), baseline: row-down.at(0), {
    for i in range(n) {
      let (k, name, c) = nodes.at(i)
      let (j, at) = (depth.at(i), x.at(i))
      place(dx: at + left.at(i), dy: base.at(j) - up.at(i), box(c))
      if k == none and open == "dots" {
        let dx = at + (wide.at(i) - dots-width) / 2
        place(dx: dx, dy: base.at(j + 1) - dots-up, box(dots))
      }
      if bar.at(i) != none {
        let (start, end) = bar.at(i)
        let y = base.at(j) - row-up.at(j) - between.at(j) / 2
        let dash = if k == none { "dashed" } else { none }
        place(dx: at + start, dy: y, line(length: end - start, stroke: (thickness: stroke, dash: dash)))
        if names.at(i) != none {
          let (w, h) = names.at(i)
          place(dx: at + end + name-gap, dy: y - h / 2, box(name))
        }
      }
    }
  })
}
"#;

/// Writes a derivation in linlog's own layout into `out`, one inference
/// at a time, and asks `stop` after each.
fn laid_out<T: Drawn>(
    derivation: &T,
    options: &Options,
    out: &mut impl Write,
    mut stop: impl FnMut() -> bool,
) -> Result<(), Error> {
    if options.form == Form::Standalone {
        write!(out, "{}\n\n", options.page.trim_end())?;
    }
    let open = match options.open {
        OpenGoal::Dots => "dots",
        OpenGoal::Dashed => "dashed",
        OpenGoal::Bare | OpenGoal::Mark(_) => "bare",
    };
    write!(
        out,
        "#context {{\n  let open = \"{open}\"\n  let gap = ({}).to-absolute()\n  \
         let name-gap = ({}).to-absolute()\n  let band = ({}).to-absolute()\n  \
         let stroke = {}\n  let dots = $dots.v$\n  let nodes = (\n",
        options.premise_gap, options.label_gap, options.band, options.stroke
    )?;
    let mut buffer = String::new();
    walk(derivation, |step| {
        let Step::Enter(id, _) = step else {
            return Ok(());
        };
        let markup = derivation.label(id, &options.labels);
        let (premises, name) = match (derivation.is_open(id), &options.open) {
            (true, OpenGoal::Mark(mark)) => ("0".to_owned(), Some(mark.as_str())),
            (true, _) => ("none".to_owned(), None),
            _ => (derivation.premises(id).len().to_string(), markup.as_deref()),
        };
        write!(buffer, "    ({premises}, ")?;
        match name {
            Some(name) => {
                buffer.push('$');
                label(&mut buffer, name);
                buffer.push('$');
            }
            None => buffer.push_str("none"),
        }
        buffer.push_str(", $");
        derivation.sequent(&mut buffer, &NOTATION, id, false, false);
        buffer.push_str("$),\n");
        flush(out, &mut buffer, &mut stop)
    })?;
    out.write_str("  )\n")?;
    out.write_str(LAYOUT.trim_end())?;
    Ok(())
}

/// Writes a derivation as a curryst tree into `out`, one inference at a
/// time, and asks `stop` after each.
fn curryst<T: Drawn>(
    derivation: &T,
    options: &Options,
    out: &mut impl Write,
    mut stop: impl FnMut() -> bool,
) -> Result<(), Error> {
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
        derivation.sequent(out, &NOTATION, id, false, false);
        out.push('$');
    };
    let mut buffer = String::new();
    walk(derivation, |step| {
        match step {
            Step::Enter(id, depth) => {
                let premises = derivation.premises(id);
                indent(&mut buffer, depth + 1);
                // A grid of one column is as wide as the sequent, and so
                // is a line across it.
                let open = |out: &mut String, grid: &str| {
                    out.push_str(grid);
                    conclusion(out, id);
                    out.push_str("),\n");
                };
                match (derivation.is_open(id), &options.open) {
                    (true, OpenGoal::Dots) => open(
                        &mut buffer,
                        "grid(align: center, row-gutter: 0.4em, $dots.v$, ",
                    ),
                    (true, OpenGoal::Dashed) => open(
                        &mut buffer,
                        "grid(align: center, inset: (top: 0.3em), \
                         grid.hline(stroke: (dash: \"dashed\")), ",
                    ),
                    (true, OpenGoal::Bare) => {
                        conclusion(&mut buffer, id);
                        buffer.push_str(",\n");
                    }
                    (true, OpenGoal::Mark(mark)) => {
                        buffer.push_str("rule(name: $");
                        label(&mut buffer, mark);
                        buffer.push_str("$, ");
                        conclusion(&mut buffer, id);
                        buffer.push_str("),\n");
                    }
                    _ => {
                        let markup = derivation.label(id, &options.labels);
                        let markup = markup.as_deref();
                        let name = |out: &mut String| {
                            if let Some(markup) = markup {
                                out.push_str("name: $");
                                label(out, markup);
                                out.push('$');
                            }
                        };
                        buffer.push_str("rule(");
                        if premises.is_empty() {
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
                if !derivation.premises(id).is_empty() {
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
