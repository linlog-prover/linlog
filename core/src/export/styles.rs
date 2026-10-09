// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use crate::proofs::TextOptions;

/// The options of every output format in one value, which a front end
/// keeps and changes key by key: the text tree's, LaTeX's, Typst's, the
/// SVG drawing's (which PNG and PDF render), PNG's and PDF's beyond it,
/// and the Rocq certificate's.
///
/// # JSON
///
/// With the feature `serialize` an object with a key per format, each the
/// JSON of that format's options; a missing key, or a missing field of
/// one, takes its default, and a misspelt one is refused. A format this
/// build lacks reads any value and writes nothing, so that one settings
/// file serves every build.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(
    feature = "serialize",
    derive(serde::Serialize, serde::Deserialize),
    serde(default, deny_unknown_fields)
)]
pub struct Styles {
    /// The text tree's.
    pub text: TextOptions,
    /// LaTeX's.
    ///
    /// Needs the cargo feature `latex` (on by default).
    #[cfg(feature = "latex")]
    pub latex: super::latex::Options,
    /// LaTeX's, which this build does not write.
    #[cfg(not(feature = "latex"))]
    #[cfg_attr(feature = "serialize", serde(skip_serializing))]
    latex: Absent,
    /// Typst's.
    ///
    /// Needs the cargo feature `typst` (on by default).
    #[cfg(feature = "typst")]
    pub typst: super::typst::Options,
    /// Typst's, which this build does not write.
    #[cfg(not(feature = "typst"))]
    #[cfg_attr(feature = "serialize", serde(skip_serializing))]
    typst: Absent,
    /// SVG's, for derivations, sequents and nets, and for the drawings
    /// that PNG and PDF render.
    ///
    /// Needs the cargo feature `svg` (on by default).
    #[cfg(feature = "svg")]
    pub svg: super::svg::Style,
    /// SVG's, which this build does not draw.
    #[cfg(not(feature = "svg"))]
    #[cfg_attr(feature = "serialize", serde(skip_serializing))]
    svg: Absent,
    /// PNG's, beyond the SVG's.
    ///
    /// Needs the cargo feature `png` (off by default).
    #[cfg(feature = "png")]
    pub png: super::png::Options,
    /// PNG's, which this build does not render.
    #[cfg(not(feature = "png"))]
    #[cfg_attr(feature = "serialize", serde(skip_serializing))]
    png: Absent,
    /// PDF's, beyond the SVG's.
    ///
    /// Needs the cargo feature `pdf` (off by default).
    #[cfg(feature = "pdf")]
    pub pdf: super::pdf::Options,
    /// PDF's, which this build does not render.
    #[cfg(not(feature = "pdf"))]
    #[cfg_attr(feature = "serialize", serde(skip_serializing))]
    pdf: Absent,
    /// The Rocq certificate's.
    ///
    /// Needs the cargo feature `rocq` (on by default).
    #[cfg(feature = "rocq")]
    pub rocq: super::rocq::Options,
    /// Rocq's, which this build does not write.
    #[cfg(not(feature = "rocq"))]
    #[cfg_attr(feature = "serialize", serde(skip_serializing))]
    rocq: Absent,
}

impl Styles {
    /// Returns the styles with another [`text`](Self::text).
    #[must_use]
    pub fn with_text(self, text: TextOptions) -> Self {
        Self { text, ..self }
    }

    /// Returns the styles with another [`latex`](Self::latex).
    ///
    /// Needs the cargo feature `latex` (on by default).
    #[cfg(feature = "latex")]
    #[must_use]
    pub fn with_latex(self, latex: super::latex::Options) -> Self {
        Self { latex, ..self }
    }

    /// Returns the styles with another [`typst`](Self::typst).
    ///
    /// Needs the cargo feature `typst` (on by default).
    #[cfg(feature = "typst")]
    #[must_use]
    pub fn with_typst(self, typst: super::typst::Options) -> Self {
        Self { typst, ..self }
    }

    /// Returns the styles with another [`svg`](Self::svg).
    ///
    /// Needs the cargo feature `svg` (on by default).
    #[cfg(feature = "svg")]
    #[must_use]
    pub fn with_svg(self, svg: super::svg::Style) -> Self {
        Self { svg, ..self }
    }

    /// Returns the styles with another [`png`](Self::png).
    ///
    /// Needs the cargo feature `png` (off by default).
    #[cfg(feature = "png")]
    #[must_use]
    pub fn with_png(self, png: super::png::Options) -> Self {
        Self { png, ..self }
    }

    /// Returns the styles with another [`pdf`](Self::pdf).
    ///
    /// Needs the cargo feature `pdf` (off by default).
    #[cfg(feature = "pdf")]
    #[must_use]
    pub fn with_pdf(self, pdf: super::pdf::Options) -> Self {
        Self { pdf, ..self }
    }

    /// Returns the styles with another [`rocq`](Self::rocq).
    ///
    /// Needs the cargo feature `rocq` (on by default).
    #[cfg(feature = "rocq")]
    #[must_use]
    pub fn with_rocq(self, rocq: super::rocq::Options) -> Self {
        Self { rocq, ..self }
    }
}

/// The options of a format this build lacks: any value reads as it, and
/// nothing is written.
#[cfg(not(all(
    feature = "latex",
    feature = "typst",
    feature = "svg",
    feature = "png",
    feature = "pdf",
    feature = "rocq"
)))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Absent;

#[cfg(all(
    feature = "serialize",
    not(all(
        feature = "latex",
        feature = "typst",
        feature = "svg",
        feature = "png",
        feature = "pdf",
        feature = "rocq"
    ))
))]
impl<'a> serde::Deserialize<'a> for Absent {
    /// Reads any value and keeps nothing of it.
    fn deserialize<D: serde::Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        serde::de::IgnoredAny::deserialize(deserializer).map(|_| Self)
    }
}
