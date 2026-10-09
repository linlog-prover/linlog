// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! PNG: an SVG document of [`svg`](super::svg) rendered to an image with
//! resvg, its text set in the fonts the caller gives and no other, so that
//! the image depends on nothing but the arguments. A transparent drawing
//! stays transparent; [`Style::background`](super::svg::Style) fills it.
//! The image declares its colours as sRGB and its density as the scale
//! times 96 pixels per inch, so that a viewer shows it at the drawing's
//! size, and carries the drawing's title and description as its `Title`
//! and `Description`.
//!
//! Needs the cargo feature `png` (off by default).

use super::{Measure, PNG, declared_size, parse, texts};
use crate::Error;
use resvg::tiny_skia::{Pixmap, Transform};

/// What a user may vary in a PNG beyond the drawing's style.
/// Made from its `default()` with the fields set that differ, since a
/// later field breaks no caller that way.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serialize", serde(default, deny_unknown_fields))]
pub struct Options {
    /// The pixels of the image per pixel of the SVG document.
    pub scale: u32,
    /// The most pixels the image may have, or `None` for no bound: an
    /// image takes up to eight bytes per pixel while it is drawn and
    /// encoded. The bound is compared before the SVG is parsed, with the
    /// size its root declares, and again after.
    pub pixels: Option<u64>,
}

impl Options {
    /// The default scale, sharp on a screen of twice the usual density.
    pub const DEFAULT_SCALE: u32 = 2;
    /// The default bound on the pixels, 64 million: about 512 MiB while
    /// drawn.
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
/// `fonts` (the data of font files), within `limits.memory_bytes`.
///
/// The memory is an estimate made before the SVG is parsed: it counts the
/// glyphs of the drawing (usvg sets each as a path, some 1 000 bytes),
/// its elements, the arcs usvg strokes to bound them and the pixels of
/// the image; what takes time rather than memory is counted as the memory
/// the renderer fills in that time, so that the bound keeps a render short
/// as well: the default is about 8 s of rendering on a slow core at most.
///
/// # Errors
///
/// [`Refusal::Pixels`](crate::Refusal::Pixels) past
/// [`Options::pixels`] and [`Refusal::Memory`](crate::Refusal::Memory)
/// past the memory bound (both compared before the SVG is parsed, the
/// pixels again after), and [`Error::NotSvg`] for a text that is no
/// document the renderer reads.
pub fn from_svg(
    svg: &str,
    fonts: &[&[u8]],
    options: &Options,
    limits: &crate::Limits,
) -> Result<Vec<u8>, Error> {
    let scale = options.scale.max(1);
    let pixels_of = |(width, height): (u64, u64)| {
        let side = |n: u64| n.saturating_mul(u64::from(scale));
        (
            side(width),
            side(height),
            side(width).saturating_mul(side(height)),
        )
    };
    let too_large = |pixels| {
        Error::Refused(crate::limits::Refusal::Pixels {
            pixels,
            limit: options.pixels.unwrap_or(u64::MAX),
        })
    };
    let measure = Measure::of(svg);
    // Within the bounds, or why not, for an image of `pixels`.
    let check = |pixels: u64| {
        if options.pixels.is_some_and(|limit| pixels > limit) {
            return Err(too_large(pixels));
        }
        let estimate = measure.estimate(&PNG, pixels);
        match limits.memory_bytes {
            Some(limit) if estimate > limit => {
                Err(Error::Refused(crate::limits::Refusal::Memory {
                    phase: crate::limits::Phase::Render,
                    limit_bytes: limit,
                    needed_bytes: Some(estimate),
                }))
            }
            _ => Ok(()),
        }
    };
    // usvg rounds the declared size up to whole pixels.
    let declared = declared_size(svg).map(|(w, h)| pixels_of((w.ceil() as u64, h.ceil() as u64)));
    check(declared.map_or(0, |(_, _, pixels)| pixels))?;
    let tree = parse(svg, fonts)?;
    let size = tree.size().to_int_size();
    let (width, height, pixels) = pixels_of((u64::from(size.width()), u64::from(size.height())));
    check(pixels)?;
    // tiny-skia refuses what its own sizes cannot hold.
    let fits = |n: u64| u32::try_from(n).ok();
    let mut pixmap = fits(width)
        .zip(fits(height))
        .and_then(|(w, h)| Pixmap::new(w, h))
        .ok_or_else(|| too_large(pixels))?;
    let factor = scale as f32;
    resvg::render(
        &tree,
        Transform::from_scale(factor, factor),
        &mut pixmap.as_mut(),
    );
    let (title, description) = texts(svg);
    encode(&pixmap, scale, title, description).map_err(|e| Error::RenderFailed {
        message: e.to_string(),
    })
}

/// Returns a pixmap as PNG bytes with the image's colour space, density,
/// title and description.
fn encode(
    pixmap: &Pixmap,
    scale: u32,
    title: Option<String>,
    description: Option<String>,
) -> Result<Vec<u8>, png::EncodingError> {
    let mut data = Vec::with_capacity(pixmap.data().len());
    for pixel in pixmap.pixels() {
        let colour = pixel.demultiply();
        data.extend([colour.red(), colour.green(), colour.blue(), colour.alpha()]);
    }
    let mut bytes = Vec::new();
    let mut encoder = png::Encoder::new(&mut bytes, pixmap.width(), pixmap.height());
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_source_srgb(png::SrgbRenderingIntent::Perceptual);
    // 96 pixels per inch at scale 1, per metre and rounded.
    let density = u32::try_from((u64::from(scale) * 960_000 + 127) / 254).unwrap_or(u32::MAX);
    encoder.set_pixel_dims(Some(png::PixelDimensions {
        xppu: density,
        yppu: density,
        unit: png::Unit::Meter,
    }));
    for (keyword, text) in [("Title", title), ("Description", description)] {
        if let Some(text) = text {
            encoder.add_itxt_chunk(keyword.to_owned(), text)?;
        }
    }
    let mut writer = encoder.write_header()?;
    writer.write_image_data(&data)?;
    writer.finish()?;
    Ok(bytes)
}
