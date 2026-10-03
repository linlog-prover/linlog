// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! SVG: a sequent as one line of text, and a whole SVG document for each
//! drawing.
//!
//! The layout is computed here, not by the viewer: every width is the sum
//! of the advances of the style's [`Font`], by default the Euler Math
//! font's from a table committed with the code, so the output is
//! deterministic and needs no font at run time. The document asks for the
//! font's families (`"Euler Math", "Neo Euler", serif`) and does not embed
//! the font; every `<text>` carries its computed width as `textLength`, so
//! a viewer that sets it in another font still fits the layout. Letters of
//! atom names are the mathematical italic characters, which a math font
//! sets as math italic, and rule names stay upright; the `⊥` of a negated
//! atom is a superscript, the `₁` and `₂` of a rule name are subscripts.
//! All coordinates are integers in thousandths of an em of formula text
//! ([`Style::font_size`] pixels), so the text is selectable and the
//! drawing scales without loss.
//!
//! # Examples
//!
#![cfg_attr(feature = "parse", doc = "```")]
#![cfg_attr(not(feature = "parse"), doc = "```ignore")]
//! use linlog::Sequent;
//! use linlog::export::svg::{self, Style};
//!
//! let sequent: Sequent = "A |- A".parse()?;
//! let svg = svg::sequent(&sequent, &Style::default());
//! assert!(svg.starts_with("<svg xmlns=\"http://www.w3.org/2000/svg\""));
//! assert!(svg.contains("<title>⊢ A⊥, A</title>"));
//! # Ok::<(), linlog::Error>(())
//! ```

/// The metrics of the font.
mod font;
/// Proof structures as formula trees under their axiom links.
mod net;
/// Derivations as proof trees.
mod tree;

use super::notation::Notation;
use crate::nets::ProofStructure;
use crate::occurrences::{OccId, Position, Reading};
use crate::proofs::style::{Part, parts};
use crate::proofs::{Derivation, Labels, OpenGoal, WriteError};
use crate::sequents::Sequent;
pub use font::{Advances, Font, MONOSPACE};
use font::{DEPTH, HEIGHT, LOWER, RAISE, SCRIPT};
use std::fmt::Write;

/// The sizes, distances and colours of a drawing. Lengths are in
/// thousandths of an em of formula text, whose size in pixels is
/// [`font_size`](Self::font_size); colours are CSS colours.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serialize", serde(default, deny_unknown_fields))]
pub struct Style {
    /// The font the drawing asks for and the advances it is laid out with.
    pub font: Font,
    /// The labels of the rules.
    pub labels: Labels,
    /// How an open goal is drawn.
    pub open: OpenGoal,
    /// Whether every formula of a conclusion is a group of its own, with
    /// the id `i<n>-<p>` for position `p` of the sequent of inference `n`.
    pub ids: bool,
    /// Whether a drawing of a derivation or a net carries a `<desc>` that
    /// reads it in order, which a screen reader reads out: the numbered
    /// inferences of [`Derivation::write_steps`], or the net's links as
    /// its text form gives them.
    pub description: bool,
    /// The size of formula text in pixels, which scales the whole drawing.
    pub font_size: u32,
    /// The size of rule names and of the connectives in a proof net's
    /// nodes, in thousandths of the size of formula text.
    pub label_size: u32,
    /// The distance from the baseline of a conclusion to the baseline of
    /// its premises, and from one layer of a proof net to the next.
    pub line_height: u32,
    /// The space between two premises side by side.
    pub premise_gap: u32,
    /// The space between two literals of a proof net.
    pub literal_gap: u32,
    /// The space between an inference line and its rule name.
    pub label_gap: u32,
    /// The empty border around the drawing.
    pub margin: u32,
    /// The thickness of every line.
    pub stroke_width: u32,
    /// The height of an axiom link's arc, in thousandths of half its
    /// width, so that the arc over a pair of literals nested between two
    /// others stays below theirs.
    pub link_height: u32,
    /// The radius of the circle of a `⊗` or `⅋` node of a proof net.
    pub node_radius: u32,
    /// The colour of text.
    pub text: String,
    /// The colour of inference lines, of a proof net's nodes, of the
    /// premise edges of its `⊗` nodes and of its conclusions.
    pub line: String,
    /// The colour of the premise edges of a `⅋` node, which are also
    /// dashed: a switching keeps one of the two.
    pub par: String,
    /// The colour of axiom links.
    pub link: String,
    /// The colour of the edges of a switching cycle, when a proof
    /// structure has one.
    pub highlight: String,
    /// The colour the drawing is filled with, or none for a transparent
    /// background.
    pub background: Option<String>,
}

impl Default for Style {
    /// Returns Euler Math, upright labels, an open goal under vertical
    /// dots, no ids per formula, a description, and black lines and text at 16 pixels to
    /// the em on a transparent background, with `⅋` edges blue and a
    /// switching cycle orange.
    fn default() -> Self {
        Self {
            font: Font::euler(),
            labels: Labels::Upright,
            open: OpenGoal::Dots,
            ids: false,
            description: true,
            font_size: 16,
            label_size: 800,
            line_height: 1500,
            premise_gap: 1500,
            literal_gap: 1000,
            label_gap: 200,
            margin: 300,
            stroke_width: 40,
            link_height: 600,
            node_radius: 380,
            text: "black".into(),
            line: "black".into(),
            par: "#0072b2".into(),
            link: "black".into(),
            highlight: "#d55e00".into(),
            background: None,
        }
    }
}

impl Style {
    /// Returns the default style in light colours on a dark background.
    pub fn dark() -> Self {
        Self {
            text: "#e6e6e6".into(),
            line: "#e6e6e6".into(),
            par: "#56b4e9".into(),
            link: "#e6e6e6".into(),
            highlight: "#e69f00".into(),
            background: Some("#1e1e1e".into()),
            ..Self::default()
        }
    }

    /// Returns the default style set in a monospace font
    /// ([`Font::monospace`]).
    pub fn monospace() -> Self {
        Self {
            font: Font::monospace(),
            ..Self::default()
        }
    }
}

/// The characters that open and close a formula of a sequent in the text
/// the notation writes, which [`run`] makes pieces of their own.
const FORMULA: (char, char) = ('\u{2}', '\u{3}');

/// The characters that open and close a subscript of a rule's label in
/// the text [`label`] writes.
const SUBSCRIPT: (char, char) = ('\u{4}', '\u{5}');

/// Writes a label's markup as text for [`run`]: symbols and text as they
/// are, a subscript between the [`SUBSCRIPT`] marks.
fn label(markup: &str) -> String {
    let mut out = String::new();
    for part in parts(markup) {
        match part {
            Part::Symbol(c) => out.push(c),
            Part::Text(text) => out.push_str(text),
            Part::Sub(text) => {
                out.push(SUBSCRIPT.0);
                out.push_str(text);
                out.push(SUBSCRIPT.1);
            }
        }
    }
    out
}

/// Returns the positions of a sequent's formulas in the order the
/// notation writes them: as they are one-sided, the goal last two-sided.
fn drawn(reading: Option<&Reading>, sequent: &[OccId]) -> Vec<usize> {
    let mut positions: Vec<usize> = (0..sequent.len()).collect();
    if let Some(reading) = reading {
        positions.sort_by_key(|&p| reading.position(sequent[p]) == Position::Output);
    }
    positions
}

/// The character that stands for the raised `⊥` of a negated atom in the
/// text the notation writes, a control character that no atom name keeps.
const RAISED_BOT: char = '\u{1}';

/// The spelling of formulas and sequents as they are drawn: Unicode
/// connectives and mathematical italic letters, which [`run`] lays out.
const NOTATION: Notation = Notation {
    tensor: "⊗",
    par: "⅋",
    with: "&",
    plus: "⊕",
    lollipop: "⊸",
    bang: "!",
    quest: "?",
    one: "1",
    bot: "⊥",
    top: "⊤",
    zero: "0",
    dual: "\u{1}",
    turnstile: "⊢",
    align: "",
    atom: italic,
};

/// The spelling of formulas and sequents in a drawing's title: plain
/// Unicode text.
const PLAIN: Notation = Notation {
    dual: "⊥",
    atom: plain,
    ..NOTATION
};

/// Writes an atom's name with its Latin letters as mathematical italic
/// characters and its control characters, which XML cannot carry, as
/// `�`.
fn italic(out: &mut String, name: &str) {
    for c in name.chars() {
        out.push(match c {
            'h' => 'ℎ',
            'A'..='Z' => char::from_u32(0x1D434 + (c as u32 - 'A' as u32)).unwrap(),
            'a'..='z' => char::from_u32(0x1D44E + (c as u32 - 'a' as u32)).unwrap(),
            c if c.is_control() => '\u{FFFD}',
            c => c,
        });
    }
}

/// Writes an atom's name as it is, with its control characters as `�`.
fn plain(out: &mut String, name: &str) {
    out.extend(
        name.chars()
            .map(|c| if c.is_control() { '\u{FFFD}' } else { c }),
    );
}

/// Writes a character escaped for XML text and attribute values.
fn escape(out: &mut String, c: char) {
    match c {
        '&' => out.push_str("&amp;"),
        '<' => out.push_str("&lt;"),
        '>' => out.push_str("&gt;"),
        '"' => out.push_str("&quot;"),
        c => out.push(c),
    }
}

/// Returns a text escaped for XML.
fn escaped(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    text.chars().for_each(|c| escape(&mut out, c));
    out
}

/// A piece of a line of text on one baseline and at one size, which is
/// one `<text>` element: renderers disagree on how `textLength` spreads
/// over a `<tspan>` that shifts the baseline, and agree on a text of its
/// own.
#[derive(Clone, Debug, Default)]
struct Piece {
    /// The escaped text, without spaces at either end.
    text: String,
    /// Where the piece starts, from the start of the line.
    offset: i64,
    /// How far its baseline is above the line's.
    raise: i64,
    /// Its font size, or none for the line's.
    size: Option<i64>,
    /// Its width.
    width: i64,
    /// The formula of the sequent it belongs to, counted in the order of
    /// the text, or none.
    formula: Option<usize>,
}

/// A line of text laid out, in thousandths of an em of formula text.
#[derive(Clone, Debug, Default)]
struct Run {
    /// The pieces, left to right.
    pieces: Vec<Piece>,
    /// The width.
    width: i64,
}

/// Lays out a line the notation or a rule's label wrote, at `size`
/// thousandths of an em in the font: [`RAISED_BOT`] becomes a
/// superscript `⊥`, what stands between the [`SUBSCRIPT`] marks a
/// subscript, and what stands between the [`FORMULA`] marks pieces of
/// that formula. Spaces separate pieces rather than start or end one.
fn run(text: &str, size: i64, font: &Font) -> Run {
    // Widths in millionths of an em of `size`, which `scale` turns into
    // thousandths of an em of formula text.
    let scale = |millionths: i64| millionths * size / 1_000_000;
    let mut pieces: Vec<Piece> = Vec::new();
    let (mut position, mut start, mut end) = (0, 0, 0);
    let mut current = (0, 1000);
    let (mut lowered, mut formula, mut formulas) = (false, None, 0);
    let mut piece = String::new();
    let close =
        |pieces: &mut Vec<Piece>, piece: &mut String, shape: (i64, i64), formula, start, end| {
            piece.truncate(piece.trim_end_matches(' ').len());
            if !piece.is_empty() {
                pieces.push(Piece {
                    text: std::mem::take(piece),
                    offset: scale(start),
                    raise: shape.0,
                    size: (shape.1 != 1000).then_some(size * shape.1 / 1000),
                    width: scale(end) - scale(start),
                    formula,
                });
            }
        };
    for c in text.chars() {
        if c == FORMULA.0 || c == FORMULA.1 {
            close(&mut pieces, &mut piece, current, formula, start, end);
            formula = (c == FORMULA.0).then_some(formulas);
            formulas += usize::from(c == FORMULA.0);
            continue;
        }
        if c == SUBSCRIPT.0 || c == SUBSCRIPT.1 {
            lowered = c == SUBSCRIPT.0;
            continue;
        }
        let (c, shape) = match c {
            RAISED_BOT => ('⊥', (RAISE * size / 1000, SCRIPT)),
            c if lowered => (c, (-LOWER * size / 1000, SCRIPT)),
            c => (c, (0, 1000)),
        };
        if shape != current {
            close(&mut pieces, &mut piece, current, formula, start, end);
            current = shape;
        }
        let advance = i64::from(font.advances.advance(c)) * shape.1;
        if c == ' ' {
            if !piece.is_empty() {
                piece.push(' ');
            }
            position += advance;
            continue;
        }
        if piece.is_empty() {
            start = position;
        }
        position += advance;
        end = position;
        escape(&mut piece, c);
    }
    close(&mut pieces, &mut piece, current, formula, start, end);
    Run {
        pieces,
        width: scale(position),
    }
}

/// Writes a line of text with its left end at `x` and its baseline at
/// `y`, every piece a `<text>` stretched or squeezed to the width the
/// layout gave it; `attributes` go into the start tag of the one `<text>`
/// or of the group of several, as they are. With `ids`, the id of an
/// inference's conclusion and the positions of its formulas in the order
/// written, the line is a group and every formula a group in it.
fn text(
    out: &mut String,
    x: i64,
    y: i64,
    run: &Run,
    attributes: &str,
    ids: Option<(u32, &[usize])>,
) {
    let group = run.pieces.len() > 1 || ids.is_some();
    if group {
        writeln!(out, "<g{attributes}>").unwrap();
    }
    let mut open = None;
    for piece in &run.pieces {
        if let Some((id, positions)) = ids
            && piece.formula != open
        {
            if open.is_some() {
                out.push_str("</g>\n");
            }
            if let Some(k) = piece.formula {
                writeln!(out, r#"<g id="i{id}-{}">"#, positions[k]).unwrap();
            }
            open = piece.formula;
        }
        write!(
            out,
            r#"<text x="{}" y="{}" textLength="{}" lengthAdjust="spacing""#,
            x + piece.offset,
            y - piece.raise,
            piece.width
        )
        .unwrap();
        if let Some(size) = piece.size {
            write!(out, r#" font-size="{size}""#).unwrap();
        }
        if !group {
            out.push_str(attributes);
        }
        writeln!(out, ">{}</text>", piece.text).unwrap();
    }
    if open.is_some() {
        out.push_str("</g>\n");
    }
    if group {
        out.push_str("</g>\n");
    }
}

/// Returns thousandths as a decimal number, without trailing zeros.
fn decimal(thousandths: i64) -> String {
    let text = format!("{}.{:03}", thousandths / 1000, thousandths % 1000);
    text.trim_end_matches('0').trim_end_matches('.').to_owned()
}

/// Writes the start of an SVG document of `width` by `height`
/// thousandths of an em with the given title, in the style's font and
/// colours, up to where its description goes: one image, whose title is
/// its accessible name.
fn head(out: &mut impl Write, style: &Style, title: &str, size: (i64, i64)) -> std::fmt::Result {
    let (width, height) = size;
    let px = |length: i64| decimal(length * i64::from(style.font_size));
    write!(
        out,
        r#"<svg xmlns="http://www.w3.org/2000/svg" role="img" width="{}" height="{}" viewBox="0 0 {width} {height}" font-family="{}" font-size="1000" fill="{}">"#,
        px(width),
        px(height),
        escaped(&style.font.family),
        escaped(&style.text),
    )?;
    write!(out, "\n<title>{}</title>\n", escaped(title))
}

/// A writer that escapes what it passes on for XML text.
struct Escaping<'a, W>(&'a mut W);

impl<W: Write> Write for Escaping<'_, W> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        let mut text = String::with_capacity(s.len());
        s.chars().for_each(|c| escape(&mut text, c));
        self.0.write_str(&text)
    }
}

/// Writes the background of a document of `width` by `height`
/// thousandths of an em, if the style has one: what comes after the head
/// and the description.
fn backdrop(out: &mut impl Write, style: &Style, size: (i64, i64)) -> std::fmt::Result {
    let (width, height) = size;
    if let Some(background) = &style.background {
        writeln!(
            out,
            r#"<rect width="{width}" height="{height}" fill="{}"/>"#,
            escaped(background)
        )?;
    }
    Ok(())
}

/// Returns an SVG document of `width` by `height` thousandths of an em
/// with the given title, description and body, in the style's font and
/// colours.
fn document(
    style: &Style,
    title: &str,
    description: Option<&str>,
    size: (i64, i64),
    body: &str,
) -> String {
    let mut out = String::new();
    head(&mut out, style, title, size).unwrap();
    if let Some(description) = description {
        writeln!(out, "<desc>{}</desc>", escaped(description)).unwrap();
    }
    backdrop(&mut out, style, size).unwrap();
    out.push_str(body);
    out.push_str("</svg>");
    out
}

/// Returns a document that shows one line of text, the notation's `content`,
/// titled `title`.
fn line(style: &Style, title: &str, content: &str) -> String {
    let run = run(content, 1000, &style.font);
    let margin = i64::from(style.margin);
    let mut body = String::new();
    text(&mut body, margin, margin + HEIGHT, &run, "", None);
    document(
        style,
        title,
        None,
        (run.width + 2 * margin, HEIGHT + DEPTH + 2 * margin),
        &body,
    )
}

/// Returns a sequent one-sided, `⊢ A⊥, A`, as an SVG document of one line
/// of text, titled with the sequent in plain text.
pub fn sequent(sequent: &Sequent, style: &Style) -> String {
    let (mut drawn, mut title) = (String::new(), String::new());
    NOTATION.one_sided(&mut drawn, sequent);
    PLAIN.one_sided(&mut title, sequent);
    line(style, &title, &drawn)
}

/// Returns the sequent of an intuitionistic reading two-sided,
/// `A, A ⊸ B ⊢ B`, as an SVG document of one line of text, titled with
/// the sequent in plain text.
pub fn two_sided(reading: &Reading, style: &Style) -> String {
    let forest = reading.forest();
    let (mut drawn, mut title) = (String::new(), String::new());
    NOTATION.sequent(
        &mut drawn,
        forest,
        Some(reading),
        forest.roots(),
        false,
        false,
    );
    PLAIN.sequent(
        &mut title,
        forest,
        Some(reading),
        forest.roots(),
        false,
        false,
    );
    line(style, &title, &drawn)
}

/// Returns a derivation as an SVG document of its proof tree, two-sided if
/// the derivation is, titled with its conclusion in plain text.
///
/// The tree grows upwards from its conclusion: the premises of an
/// inference stand side by side, their conclusions centred over its own,
/// under an inference line that spans both with the rule's name to its
/// right. An open goal of a proof in progress is its sequent under
/// vertical dots, with no inference line, unless the style says
/// otherwise. The conclusion of inference `n` is the `<text>` or the group
/// with the id `i<n>`, and with [`Style::ids`] its formula at position `p`
/// the group `i<n>-<p>`. The layout is one pass up and one down the tree,
/// each with a stack of its own, so a derivation of any height fits and
/// drawing it again after every step of an interactive proof stays cheap.
pub fn derivation(derivation: &Derivation, style: &Style) -> String {
    let mut out = String::new();
    write(derivation, style, &mut out, || false).expect("a string takes any text");
    out
}

/// Writes a derivation as [`derivation`] returns it into `out`, the
/// elements of one inference at a time, and asks `stop` after each. What
/// it holds besides is a few numbers per inference.
pub fn write(
    derivation: &Derivation,
    style: &Style,
    out: &mut impl Write,
    stop: impl FnMut() -> bool,
) -> Result<(), WriteError> {
    tree::draw(derivation, style, out, stop)
}

/// Returns a proof structure, a proof net or not, complete or not, as an
/// SVG document, titled with its sequent in plain text.
///
/// The conclusions stand in a row at the bottom and the formula trees
/// grow upwards from them: every `⊗` and `⅋` is a small labelled circle,
/// the edges to the two premises of a `⅋` are dashed and share a colour
/// of their own (a switching keeps one of them), and the literals stand
/// side by side along the top, each axiom link an arc over the two it
/// joins; Mix leaves no trace. A structure with a switching cycle has the
/// cycle's edges in the highlight colour. Literals and connectives are the
/// elements with the id `o<n>` for occurrence `n`, and a link is `l<m>-<n>`.
pub fn net(net: &ProofStructure, style: &Style) -> String {
    net::draw(net, style)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A raised `⊥` and a subscript are pieces of their own, smaller and
    /// shifted, and spaces separate pieces; the widths count every piece at
    /// its size.
    #[test]
    fn scripts_are_pieces() {
        let pieces = |r: &Run| {
            r.pieces
                .iter()
                .map(|p| (p.text.clone(), p.offset, p.raise, p.size, p.width))
                .collect::<Vec<_>>()
        };
        let (a, bot, comma, space) = (770, 611, 277, 333);
        let r = run("A\u{1} ⊗ A", 1000, &Font::euler());
        assert_eq!(
            pieces(&r),
            [
                ("A".into(), 0, 0, None, a),
                ("⊥".into(), a, 400, Some(700), bot),
                ("⊗ A".into(), a + bot + space, 0, None, 668 + space + a),
            ]
        );
        assert_eq!(r.width, a + bot + space + 668 + space + a);

        let r = run("&L\u{4}1\u{5}, ", 800, &Font::euler());
        assert_eq!(
            pieces(&r),
            [
                ("&amp;L".into(), 0, 0, None, 1036),
                ("1".into(), 1036, -120, Some(560), 280),
                (",".into(), 1316, 0, None, comma * 8 / 10),
            ]
        );
    }

    /// Atom names keep their letters as math italic and lose their control
    /// characters.
    #[test]
    fn names_are_italic() {
        let mut out = String::new();
        italic(&mut out, "Ah_1α\u{1}");
        assert_eq!(out, "𝐴ℎ_1α\u{FFFD}");
    }
}
