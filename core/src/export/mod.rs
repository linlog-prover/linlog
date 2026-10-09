// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Export of sequents and derivations: to LaTeX with the `ebproof` package
//! ([`latex`], feature `latex`), to Typst with the
//! `curryst` package or a layout of linlog's own
//! ([`typst`], feature `typst`),
//! to SVG, with proof nets ([`svg`], feature `svg`),
//! and to Rocq as proof scripts for the NanoYalla kernel
//! ([`rocq`], feature `rocq`).
//! Every function is a pure function of its input and of one options
//! value per target, which holds everything a user may vary and has serde
//! behind `serialize`; the output is deterministic. Anything
//! [`Drawable`], a derivation of linear logic or
//! one of LK or LJ, is written into any [`std::fmt::Write`] (a `String`
//! among them) with a stop condition asked between two inferences, by one
//! `write` per target of the same signature as the text tree's
//! ([`Derivation::write_text`](crate::Derivation::write_text)). LaTeX,
//! Typst and Rocq come as a fragment to paste or as a standalone document
//! ([`Form`]), and SVG is always a whole document.
//! PNG ([`png`], feature `png`) and PDF
//! ([`pdf`], feature `pdf`) render an SVG document of
//! [`svg`] with the fonts the caller gives. [`Styles`]
//! holds the options of every format in one value.
//!
//! All targets write formulas with the bracketing of `Display` (every
//! binary subformula of a formula in brackets) and derivations as
//! `Display` draws them: one-sided, or two-sided with the hypotheses in id
//! order when the derivation has a reading.
//! They write one inference at a time and keep their own stack, so the
//! text grows with the derivation, not with its width, and a derivation of
//! any height fits. Rules are labelled as [`Labels`](crate::proofs::Labels) says,
//! an open goal is drawn as [`OpenGoal`](crate::proofs::OpenGoal) says, and an
//! inference of a compact derivation that stands for a run of a
//! structural rule ([`Compact`](crate::proofs::Compact)) has its label starred.

// A variant a later step adds must not fall into an existing arm.
#![deny(clippy::wildcard_enum_match_arm)]

#[cfg(feature = "latex")]
pub mod latex;
/// The symbol tables and the printers the targets share.
#[cfg(any(
    feature = "latex",
    feature = "typst",
    feature = "svg",
    feature = "rocq"
))]
pub(crate) mod notation;
#[cfg(feature = "pdf")]
pub mod pdf;
#[cfg(feature = "png")]
pub mod png;
#[cfg(feature = "rocq")]
pub mod rocq;
/// The options of every format in one value.
mod styles;
#[cfg(feature = "svg")]
pub mod svg;
#[cfg(feature = "typst")]
pub mod typst;

pub use styles::Styles;

/// Returns the forest of a sequent to print two-sided in `mode`, built
/// within the limits, or `None` to print it one-sided, after refusing a
/// text estimated past `limits.derivation_bytes` (`Refusal::Output`) at
/// `per_occurrence` bytes per occurrence and the longest atom name each,
/// before anything is laid out.
#[cfg(any(feature = "latex", feature = "typst", feature = "svg"))]
pub(crate) fn printed(
    sequent: &crate::Sequent,
    mode: crate::Mode,
    per_occurrence: u64,
    limits: &crate::Limits,
) -> Result<Option<crate::Forest>, crate::Error> {
    let name = sequent
        .atom_names()
        .iter()
        .map(String::len)
        .max()
        .unwrap_or(0);
    let estimate = sequent
        .occurrences()
        .saturating_mul(per_occurrence.saturating_add(name as u64));
    if let Some(limit) = limits.derivation_bytes
        && estimate > limit
    {
        return Err(crate::Error::Refused(crate::limits::Refusal::Output {
            what: "sequent",
            estimate_bytes: estimate,
            limit_bytes: limit,
            least_bytes: None,
        }));
    }
    if !mode.is_intuitionistic() {
        return Ok(None);
    }
    crate::Forest::within(sequent, limits).map(Some)
}

/// What the targets write: a derivation of linear logic, one- or
/// two-sided, or a derivation of LK or LJ read back from a proof of an
/// ordinary sequent's image. Each target's `write` takes either, as
/// `&derivation`.
#[non_exhaustive]
#[derive(Clone, Copy, Debug)]
pub enum Drawable<'a> {
    /// A derivation of linear logic.
    Linear(&'a crate::Derivation<'a>),
    /// A derivation of LK or LJ.
    Ordinary(&'a crate::ordinary::Derivation),
}

impl<'a, 'p: 'a> From<&'a crate::Derivation<'p>> for Drawable<'a> {
    /// Draws a derivation of linear logic.
    fn from(derivation: &'a crate::Derivation<'p>) -> Self {
        Self::Linear(derivation)
    }
}

impl<'a> From<&'a crate::ordinary::Derivation> for Drawable<'a> {
    /// Draws a derivation of LK or LJ.
    fn from(derivation: &'a crate::ordinary::Derivation) -> Self {
        Self::Ordinary(derivation)
    }
}

/// Whether an export is a fragment to paste into a document or a document
/// of its own.
///
/// Needs one of the cargo features `latex`, `typst` or `rocq`.
#[non_exhaustive]
#[cfg(any(feature = "latex", feature = "typst", feature = "rocq"))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serialize", serde(rename_all = "lowercase"))]
pub enum Form {
    /// The sequent or the proof tree alone, for a document that loads the
    /// packages it needs.
    #[default]
    Fragment,
    /// A complete document that compiles on its own, cropped to its
    /// content, or a whole Rocq file with its import.
    Standalone,
}

/// Returns the title and the description an SVG document of
/// [`svg`](crate::export::svg) carries, its accessible name and its
/// reading, where it has them.
#[cfg(any(feature = "png", feature = "pdf"))]
fn texts(svg: &str) -> (Option<String>, Option<String>) {
    let Ok(document) = resvg::usvg::roxmltree::Document::parse(svg) else {
        return (None, None);
    };
    let text = |name: &str| {
        let element = document
            .root_element()
            .children()
            .find(|node| node.has_tag_name(name))?;
        Some(element.text().unwrap_or_default().to_owned())
    };
    (text("title"), text("desc"))
}

/// Parses an SVG document with the fonts given and no other: what a text
/// is set in depends on nothing but the arguments.
#[cfg(any(feature = "png", feature = "pdf"))]
fn parse(svg: &str, fonts: &[&[u8]]) -> Result<resvg::usvg::Tree, crate::Error> {
    let mut options = resvg::usvg::Options::default();
    let database = options.fontdb_mut();
    for font in fonts {
        database.load_font_data(font.to_vec());
    }
    resvg::usvg::Tree::from_str(svg, &options).map_err(|e| crate::Error::NotSvg {
        message: e.to_string(),
    })
}

/// What a render costs per unit of a [`Measure`], in bytes: each the
/// larger of the memory it was measured to take and the time it was
/// measured to take at [`BYTES_PER_MICROSECOND`], a quarter more, so that
/// a bound on the estimate bounds both. Measured with usvg 0.47 and
/// krilla 0.8 in a release build on one slow core of an x86-64 desktop
/// (peak resident memory and wall time), on derivations of
/// `a0 ⊗ … ⊗ aN ⊢ a0 ⊗ … ⊗ aN` (N from 50 to 200, atom names of 2 to 4
/// and of 20 characters: 0.9 to 13 MB of SVG, up to
/// 1.2 GB and 5 s as a PDF), a sequent of 40 000 literals on one line
/// (160 million units wide), and nets of `a0 ⅋ … ⅋ aN, aN⊥ ⅋ … ⅋ a0⊥`
/// and of crossing links under Mix (500 to 6 000 links, up to 5.7 MB of
/// SVG and 15 s).
#[cfg(any(feature = "png", feature = "pdf"))]
#[derive(Clone, Copy, Debug)]
struct Costs {
    /// What every render takes whatever its drawing: the fonts, the
    /// renderer's own tables.
    base: u64,
    /// Per byte of the SVG text, which is read and parsed whole.
    byte: u64,
    /// Per element: 3 750 bytes measured in usvg's tree, 3 490 in the
    /// PDF.
    element: u64,
    /// Per glyph: usvg sets each as a path, 795 bytes and 4.7 µs; the
    /// PDF 1 035 bytes and 6.2 µs with text embedded, 1 600 bytes and
    /// 12 µs with text as outlines.
    glyph: u64,
    /// Per pixel of the image: 6.0 to 6.5 bytes measured, and 33 to 76 ns
    /// drawing and encoding it.
    pixel: u64,
    /// How often the arcs are stroked: once to bound them in the parse,
    /// and again when the image is drawn.
    arc_passes: u64,
}

/// The rate at which the measured renderers fill memory, 128 MiB a
/// second (usvg and krilla filled 167 MB a second on glyphs): a cost
/// that takes time and little memory, an arc or a pixel, is charged the
/// bytes they would fill in that time.
#[cfg(any(feature = "png", feature = "pdf"))]
const BYTES_PER_MICROSECOND: u64 = 134;

/// The costs of a PNG: usvg's parse, then the image.
#[cfg(feature = "png")]
const PNG: Costs = Costs {
    base: 32 << 20,
    byte: 2,
    element: 4700,
    glyph: 1000,
    pixel: 14,
    arc_passes: 2,
};

/// The costs of a PDF with its text embedded as text: usvg's parse, then
/// krilla.
#[cfg(feature = "pdf")]
const PDF: Costs = Costs {
    base: 32 << 20,
    byte: 2,
    element: 4400,
    glyph: 1300,
    pixel: 0,
    arc_passes: 1,
};

/// The costs of a PDF with its text as outlines.
#[cfg(feature = "pdf")]
const PDF_OUTLINES: Costs = Costs { glyph: 2000, ..PDF };

/// The microseconds usvg takes at least to stroke an arc, when bounding
/// it in the parse.
#[cfg(any(feature = "png", feature = "pdf"))]
const ARC_LEAST: u64 = 200;

/// The microseconds an arc takes beyond [`ARC_LEAST`] per million units
/// its start point lies from the origin ([`arc_micros`]).
#[cfg(any(feature = "png", feature = "pdf"))]
const ARC_PER_MILLION: u64 = 2000;

/// The microseconds a wide arc takes beyond the others, wherever it
/// starts: 1.4 ms measured for a half-width of 10 million at the origin.
#[cfg(any(feature = "png", feature = "pdf"))]
const ARC_WIDE: u64 = 1500;

/// The half-width from which an arc is wide.
#[cfg(any(feature = "png", feature = "pdf"))]
const WIDE: f64 = 5e5;

/// The most microseconds an arc was measured to take, 35 ms, a seventh
/// more.
#[cfg(any(feature = "png", feature = "pdf"))]
const ARC_MOST: u64 = 40_000;

/// Returns the microseconds usvg may take to stroke an arc that starts
/// `start` units from the origin and has the larger radius `radius`.
/// usvg computes in single precision, and strokes an arc that starts
/// millions of units out in many pieces: arcs of half-width 100 000
/// starting at 8 million units took 11 ms each, at 30 million 24 ms, at
/// 50 to 200 million 29 to 35 ms, at a billion almost nothing again; a
/// wide arc near the origin took 1.4 ms. The line through `ARC_LEAST` and
/// `ARC_PER_MILLION`, `ARC_WIDE` above it for a wide arc, capped at
/// `ARC_MOST`, lies above every point of a grid of starts from 0 to 10
/// billion units and half-widths from 1 000 to 30 million, 50 arcs each.
#[cfg(any(feature = "png", feature = "pdf"))]
fn arc_micros(start: f64, radius: f64) -> u64 {
    if !start.is_finite() || !radius.is_finite() {
        return ARC_MOST;
    }
    let out = (start.abs() / 1e6 * ARC_PER_MILLION as f64) as u64;
    let wide = if radius.abs() >= WIDE { ARC_WIDE } else { 0 };
    ARC_LEAST
        .saturating_add(out)
        .saturating_add(wide)
        .min(ARC_MOST)
}

/// The elements a drawing of [`svg`] has, the only ones a render reads:
/// the measure counts what they cost, and an element that refers to
/// another (`<use>`) or brings a cost of its own (a filter, a pattern, an
/// image) would make a short text cost what the measure cannot see.
#[cfg(any(feature = "png", feature = "pdf"))]
const ELEMENTS: [&str; 8] = [
    "svg", "title", "desc", "g", "text", "path", "rect", "circle",
];

/// What a drawing holds that decides what a render takes, read off its
/// SVG text without parsing it: usvg sets every glyph as a path and
/// strokes every arc to bound it before anything can be compared with a
/// bound.
#[cfg(any(feature = "png", feature = "pdf"))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Measure {
    /// The bytes of the text.
    bytes: u64,
    /// The elements.
    elements: u64,
    /// The characters of the text content but its white space, outside
    /// `<title>` and `<desc>`: the glyphs of the drawing.
    glyphs: u64,
    /// The microseconds stroking the arcs of every path may take
    /// ([`arc_micros`]).
    arcs: u64,
}

#[cfg(any(feature = "png", feature = "pdf"))]
impl Measure {
    /// Returns the measure of an SVG document. Every count saturates, so
    /// that no text, however large, makes the estimate small.
    ///
    /// # Errors
    ///
    /// [`Error::NotSvg`] for a document with an element a drawing of
    /// [`svg`] does not have ([`ELEMENTS`]), a reference (`href`) or a
    /// document type, whose cost the measure cannot see: six levels of
    /// `<use>` in 1 287 bytes took 227 MB and 4.3 s under a bound of
    /// 64 MiB.
    fn of(svg: &str) -> Result<Self, crate::Error> {
        let refused = |what: String| {
            Err(crate::Error::NotSvg {
                message: format!("{what}, which no drawing of linlog has"),
            })
        };
        let count = |text: &str| text.chars().filter(|c| !c.is_whitespace()).count() as u64;
        let mut measure = Self {
            bytes: svg.len() as u64,
            ..Self::default()
        };
        let mut rest = svg;
        while let Some(start) = rest.find('<') {
            measure.glyphs = measure.glyphs.saturating_add(count(&rest[..start]));
            rest = &rest[start..];
            // What the markup ends with, and the end of it.
            // A `<title>` or `<desc>` reaches to its end tag, unless its
            // start tag closes it (`<title/>`): its text is no glyph.
            let start_tag = &rest[..rest.find('>').map_or(rest.len(), |i| i + 1)];
            let empty = start_tag.ends_with("/>");
            let end = if rest.starts_with("<!--") {
                "-->"
            } else if rest.starts_with("<title") && !empty {
                "</title>"
            } else if rest.starts_with("<desc") && !empty {
                "</desc>"
            } else {
                ">"
            };
            let close = rest.find(end).map_or(rest.len(), |i| i + end.len());
            let tag = &rest[..close];
            if tag.starts_with("<!") && !tag.starts_with("<!--") {
                return refused("a document type".to_owned());
            }
            if tag.as_bytes().get(1).is_some_and(u8::is_ascii_alphabetic) {
                let name = tag[1..]
                    .split(|c: char| c.is_whitespace() || c == '>' || c == '/')
                    .next()
                    .unwrap_or_default();
                if !ELEMENTS.contains(&name) {
                    return refused(format!("the element <{name}>"));
                }
                if end == ">" && tag.contains("href") {
                    return refused("a reference".to_owned());
                }
                measure.elements = measure.elements.saturating_add(1);
                if end == ">" {
                    measure.arcs = measure.arcs.saturating_add(arcs(tag));
                }
            }
            rest = &rest[close..];
        }
        measure.glyphs = measure.glyphs.saturating_add(count(rest));
        Ok(measure)
    }

    /// Returns the bytes a render of the drawing is estimated to take at
    /// `costs`, with an image of `pixels`; saturating, so that a huge
    /// drawing is refused rather than estimated small.
    fn estimate(&self, costs: &Costs, pixels: u64) -> u64 {
        [
            (self.bytes, costs.byte),
            (self.elements, costs.element),
            (self.glyphs, costs.glyph),
            (pixels, costs.pixel),
            (
                self.arcs,
                BYTES_PER_MICROSECOND.saturating_mul(costs.arc_passes),
            ),
        ]
        .into_iter()
        .fold(costs.base, |sum, (n, cost)| {
            sum.saturating_add(n.saturating_mul(cost))
        })
    }
}

/// Returns the microseconds stroking the arcs of a start tag's path data
/// (its `d` attribute) may take, by [`arc_micros`] of each arc's start
/// point and radii; saturating. The start point is followed through absolute moves
/// and lines; an arc after any other command, or in data that does not
/// read, is taken at [`ARC_MOST`].
#[cfg(any(feature = "png", feature = "pdf"))]
fn arcs(tag: &str) -> u64 {
    let Some((start, quote)) = [" d=\"", " d='"]
        .into_iter()
        .find_map(|key| tag.find(key).map(|i| (i + key.len(), &key[3..])))
    else {
        return 0;
    };
    let data = &tag[start..];
    let data = &data[..data.find(quote).unwrap_or(data.len())];
    let mut point = Some((0.0f64, 0.0f64));
    let mut micros = 0u64;
    let mut rest = data;
    while let Some(letter) = rest.chars().next() {
        let end = rest[letter.len_utf8()..]
            .find(|c: char| c.is_ascii_alphabetic())
            .map_or(rest.len(), |i| i + letter.len_utf8());
        let numbers: Option<Vec<f64>> = rest[letter.len_utf8()..end]
            .split([' ', ','])
            .filter(|n| !n.is_empty())
            .map(|n| n.parse().ok())
            .collect();
        let numbers = numbers.unwrap_or_default();
        point = match (letter, numbers.as_slice(), point) {
            ('M' | 'L', [.., x, y], _) => Some((*x, *y)),
            ('H', [.., x], Some((_, y))) => Some((*x, y)),
            ('V', [.., y], Some((x, _))) => Some((x, *y)),
            ('A', arcs, mut at) if !arcs.is_empty() && arcs.len() % 7 == 0 => {
                for arc in arcs.chunks(7) {
                    micros = micros.saturating_add(match at {
                        Some((x, y)) => {
                            arc_micros(x.abs().max(y.abs()), arc[0].abs().max(arc[1].abs()))
                        }
                        None => ARC_MOST,
                    });
                    at = Some((arc[5], arc[6]));
                }
                at
            }
            ('A' | 'a', ..) => {
                micros = micros.saturating_add(ARC_MOST);
                None
            }
            _ => None,
        };
        rest = &rest[end..];
    }
    micros
}

/// Returns the width and height an SVG document's root declares, in
/// pixels, read off its start tag without parsing the document: plain
/// numbers or numbers of `px`, as every drawing of
/// [`svg`](crate::export::svg) has them; `None` for anything else.
#[cfg(feature = "png")]
fn declared_size(svg: &str) -> Option<(f64, f64)> {
    let root = &svg[svg.find("<svg")?..];
    let root = &root[..root.find('>')?];
    let attribute = |name: &str| -> Option<f64> {
        let value = &root[root.find(&format!(" {name}=\""))? + name.len() + 3..];
        let value = &value[..value.find('"')?];
        let number: f64 = value.strip_suffix("px").unwrap_or(value).parse().ok()?;
        (number.is_finite() && number > 0.0).then_some(number)
    };
    Some((attribute("width")?, attribute("height")?))
}
