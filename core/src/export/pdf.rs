// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! PDF: an SVG document of [`svg`](super::svg) as a PDF page of its size,
//! written by krilla. A pixel of the drawing is three quarters of a point,
//! as in CSS, so text of 16 pixels is set at 12 points. Text stays text,
//! in a subset of the font embedded in the document, unless
//! [`Options::embed_text`] says otherwise; it is set in the fonts the
//! caller gives and no other. The document carries no date and no random
//! identifier, so the same drawing gives the same bytes.

use super::{RenderError, parse};
use krilla::Document;
use krilla::geom::Size;
use krilla::page::PageSettings;
use krilla_svg::{SurfaceExt, SvgSettings};

/// Points per pixel of the drawing, as CSS has them.
const POINTS_PER_PIXEL: f32 = 0.75;

/// What a user may vary in a PDF beyond the drawing's style.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serialize", serde(default, deny_unknown_fields))]
pub struct Options {
    /// Whether text is embedded as text, selectable with its font, rather
    /// than drawn as outlines.
    pub embed_text: bool,
}

impl Default for Options {
    /// Returns text embedded as text.
    fn default() -> Self {
        Self { embed_text: true }
    }
}

/// Returns an SVG document as a PDF document of one page, its text set in
/// `fonts` (the data of font files), or why it is not.
pub fn from_svg(svg: &str, fonts: &[&[u8]], options: &Options) -> Result<Vec<u8>, RenderError> {
    let tree = parse(svg, fonts)?;
    let failed = |what: &str| RenderError::Failed(what.to_owned());
    let size = Size::from_wh(
        tree.size().width() * POINTS_PER_PIXEL,
        tree.size().height() * POINTS_PER_PIXEL,
    )
    .ok_or_else(|| failed("the drawing has no size"))?;
    let mut document = Document::new();
    let mut page = document.start_page_with(PageSettings::new(size));
    let mut surface = page.surface();
    let settings = SvgSettings {
        embed_text: options.embed_text,
        ..SvgSettings::default()
    };
    let drawn = surface.draw_svg(&tree, size, settings);
    surface.finish();
    page.finish();
    drawn.ok_or_else(|| failed("the drawing could not be converted"))?;
    document
        .finish()
        .map_err(|e| RenderError::Failed(format!("{e:?}")))
}
