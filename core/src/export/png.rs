// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! PNG: an SVG document of [`svg`](super::svg) rendered to an image with
//! resvg, its text set in the fonts the caller gives and no other, so that
//! the image depends on nothing but the arguments. A transparent drawing
//! stays transparent; [`Style::background`](super::svg::Style) fills it.

use super::{RenderError, parse};
use resvg::tiny_skia::{Pixmap, Transform};

/// What a user may vary in a PNG beyond the drawing's style.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serialize", serde(default, deny_unknown_fields))]
pub struct Options {
    /// The pixels of the image per pixel of the SVG document.
    pub scale: u32,
    /// The most pixels the image may have, or `None` for no bound: an
    /// image takes four bytes per pixel while it is drawn.
    pub pixels: Option<u64>,
}

impl Options {
    /// The default scale, sharp on a screen of twice the usual density.
    pub const DEFAULT_SCALE: u32 = 2;
    /// The default bound on the pixels, 64 million: 256 MiB while drawn.
    pub const DEFAULT_PIXELS: u64 = 1 << 26;
}

impl Default for Options {
    /// Returns [`DEFAULT_SCALE`](Self::DEFAULT_SCALE) and
    /// [`DEFAULT_PIXELS`](Self::DEFAULT_PIXELS).
    fn default() -> Self {
        Self {
            scale: Self::DEFAULT_SCALE,
            pixels: Some(Self::DEFAULT_PIXELS),
        }
    }
}

/// Returns an SVG document rendered as a PNG image, its text set in
/// `fonts` (the data of font files), or why it is not: the text is no
/// document the renderer reads, or the image would pass the bound.
pub fn from_svg(svg: &str, fonts: &[&[u8]], options: &Options) -> Result<Vec<u8>, RenderError> {
    let tree = parse(svg, fonts)?;
    let scale = options.scale.max(1);
    let size = tree.size().to_int_size();
    let (width, height) = (
        u64::from(size.width()) * u64::from(scale),
        u64::from(size.height()) * u64::from(scale),
    );
    let pixels = width * height;
    let too_large = || RenderError::TooLarge {
        pixels,
        limit: options.pixels.unwrap_or(u64::MAX),
    };
    if options.pixels.is_some_and(|limit| pixels > limit) {
        return Err(too_large());
    }
    // tiny-skia refuses what its own sizes cannot hold.
    let fits = |n: u64| u32::try_from(n).ok();
    let mut pixmap = fits(width)
        .zip(fits(height))
        .and_then(|(w, h)| Pixmap::new(w, h))
        .ok_or_else(too_large)?;
    let factor = scale as f32;
    resvg::render(
        &tree,
        Transform::from_scale(factor, factor),
        &mut pixmap.as_mut(),
    );
    pixmap
        .encode_png()
        .map_err(|e| RenderError::Failed(e.to_string()))
}
