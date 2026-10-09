// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! PDF: an SVG document of [`svg`](super::svg) as a PDF page of its size,
//! written by krilla and conforming to an archival standard: PDF/A-4 (PDF
//! 2.0) by default, PDF/A-2u (PDF 1.7) with [`Options::compatible`], and
//! PDF/A-2a with PDF/UA-1 (PDF 1.7) with [`Options::accessible`]. krilla
//! checks the document against them before it returns it. A pixel of the
//! drawing is three quarters of a point, as in CSS, so text of 16 pixels
//! is set at 12 points. Text stays text, in a subset of the font embedded
//! in the document, unless [`Options::embed_text`] says otherwise; it is
//! set in the fonts the caller gives and no other. The document carries
//! the date the caller gives and no random identifier, so the same
//! drawing and date give the same bytes.
//!
//! Needs the cargo feature `pdf` (off by default).

use super::{Measure, PDF, PDF_OUTLINES, parse, texts};
use crate::Error;
use krilla::configure::{Accessibility, Archival, ConfigurationBuilder, PdfVersion};
use krilla::destination::XyzDestination;
use krilla::geom::{Point, Size};
use krilla::metadata::{DateTime, Metadata};
use krilla::outline::{Outline, OutlineNode};
use krilla::page::PageSettings;
use krilla::tagging::{ContentTag, Tag, TagGroup, TagTree};
use krilla::{Document, SerializeSettings};
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
    /// Whether the document is PDF/A-2u, PDF 1.7, rather than PDF/A-4,
    /// PDF 2.0: for a tool or an archive that does not take PDF 2.0
    /// (Typst's PDF images, several national archives).
    pub compatible: bool,
    /// Whether the document is accessible, PDF/UA-1 with PDF/A-2a (PDF
    /// 1.7, whatever `compatible` says): tagged, its drawing one figure
    /// whose alternative text is the drawing's description, the numbered
    /// reading of a derivation or the links of a net. A screen reader
    /// reads that text, not the drawing, and a long proof read out line
    /// by line may still be hard to follow.
    pub accessible: bool,
    /// The document's title, or `None` for the drawing's title.
    pub title: Option<String>,
    /// The language of the document's text, as a language tag.
    pub language: String,
    /// The date the document was made, which every PDF/A part requires;
    /// the crate reads no clock, so the caller gives it.
    pub date: Option<Date>,
    /// The most bytes the render may take, by an estimate made before the
    /// SVG is parsed, or `None` for no bound. The estimate counts the
    /// glyphs of the drawing (some 1 300 bytes each embedded as text,
    /// 2 000 as outlines), its elements and the arcs usvg strokes to bound
    /// them; what takes time rather than memory is counted as the memory
    /// the renderer fills in that time, so that the bound keeps a render
    /// short as well: the default is about 8 s of rendering on a slow core
    /// at most.
    pub memory: Option<u64>,
}

impl Options {
    /// The default bound on what a render takes, 1 GiB, the crate's
    /// [`DEFAULT_MEMORY_LIMIT`](crate::DEFAULT_MEMORY_LIMIT).
    pub const DEFAULT_MEMORY: u64 = crate::DEFAULT_MEMORY_LIMIT;
}

impl Default for Options {
    /// Returns PDF/A-4 with text embedded as text, the drawing's title,
    /// English, no date, which a caller must still give, and
    /// [`DEFAULT_MEMORY`](Self::DEFAULT_MEMORY).
    fn default() -> Self {
        Self {
            embed_text: true,
            compatible: false,
            accessible: false,
            title: None,
            language: "en".to_owned(),
            date: None,
            memory: Some(Self::DEFAULT_MEMORY),
        }
    }
}

/// A moment in UTC, to the second.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
pub struct Date {
    /// The year.
    pub year: u16,
    /// The month, 1 to 12.
    pub month: u8,
    /// The day of the month, 1 to 31.
    pub day: u8,
    /// The hour, 0 to 23.
    pub hour: u8,
    /// The minute, 0 to 59.
    pub minute: u8,
    /// The second, 0 to 59.
    pub second: u8,
}

impl Date {
    /// Returns the moment `seconds` after the Unix epoch, in UTC, as the
    /// `SOURCE_DATE_EPOCH` of reproducible builds gives it; years past
    /// 65 535 are that year.
    pub fn from_unix(seconds: u64) -> Self {
        let (days, rest) = (seconds / 86_400, seconds % 86_400);
        // The civil date of a day count, after Howard Hinnant's
        // `civil_from_days`, for days on or after 1970-01-01.
        let z = days + 719_468;
        let era = z / 146_097;
        let doe = z - era * 146_097;
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let day = doy - (153 * mp + 2) / 5 + 1;
        let month = if mp < 10 { mp + 3 } else { mp - 9 };
        let year = yoe + era * 400 + u64::from(month <= 2);
        Self {
            year: u16::try_from(year).unwrap_or(u16::MAX),
            month: month as u8,
            day: day as u8,
            hour: (rest / 3600) as u8,
            minute: (rest % 3600 / 60) as u8,
            second: (rest % 60) as u8,
        }
    }
}

/// Returns an SVG document as a PDF document of one page, its text set in
/// `fonts` (the data of font files), or why it is not: no date is given,
/// the render would pass the bound on its memory (compared before the SVG
/// is parsed), the text is no document the renderer reads, or the
/// document does not conform.
pub fn from_svg(svg: &str, fonts: &[&[u8]], options: &Options) -> Result<Vec<u8>, Error> {
    let date = options.date.ok_or(Error::NoDate)?;
    let costs = if options.embed_text {
        PDF
    } else {
        PDF_OUTLINES
    };
    let estimate = Measure::of(svg).estimate(&costs, 0);
    if let Some(limit) = options.memory.filter(|&limit| estimate > limit) {
        return Err(Error::Refused(crate::limits::Refusal::Memory {
            phase: crate::limits::Phase::Render,
            limit_bytes: limit,
            needed_bytes: Some(estimate),
        }));
    }
    let tree = parse(svg, fonts)?;
    let failed = |what: String| Error::RenderFailed { message: what };
    let size = Size::from_wh(
        tree.size().width() * POINTS_PER_PIXEL,
        tree.size().height() * POINTS_PER_PIXEL,
    )
    .ok_or_else(|| failed("the drawing has no size".to_owned()))?;
    let (drawn_title, description) = texts(svg);
    let title = options.title.clone().or(drawn_title).unwrap_or_default();

    let builder = match (options.accessible, options.compatible) {
        (true, _) => ConfigurationBuilder::new()
            .with_version(PdfVersion::Pdf17)
            .with_archival_validator(Archival::A2_A)
            .with_accessibility_validator(Accessibility::UA1),
        (false, true) => ConfigurationBuilder::new()
            .with_version(PdfVersion::Pdf17)
            .with_archival_validator(Archival::A2_U),
        (false, false) => ConfigurationBuilder::new()
            .with_version(PdfVersion::Pdf20)
            .with_archival_validator(Archival::A4),
    };
    let configuration = builder.finish().map_err(|e| failed(format!("{e:?}")))?;
    let mut document = Document::new_with(SerializeSettings {
        configuration,
        enable_tagging: options.accessible,
        ..SerializeSettings::default()
    });
    let created = DateTime::new(date.year)
        .month(date.month)
        .day(date.day)
        .hour(date.hour)
        .minute(date.minute)
        .second(date.second)
        .utc_offset_hour(0);
    document.set_metadata(
        Metadata::new()
            .title(title.clone())
            .language(options.language.clone())
            .creation_date(created),
    );

    let mut page = document.start_page_with(PageSettings::new(size));
    let mut surface = page.surface();
    let settings = SvgSettings {
        embed_text: options.embed_text,
        ..SvgSettings::default()
    };
    let figure = options
        .accessible
        .then(|| surface.start_tagged(ContentTag::Other));
    let drawn = surface.draw_svg(&tree, size, settings);
    if figure.is_some() {
        surface.end_tagged();
    }
    surface.finish();
    page.finish();
    drawn.ok_or_else(|| failed("the drawing could not be converted".to_owned()))?;
    if let Some(figure) = figure {
        let mut group = TagGroup::new(Tag::Figure(Some(description.unwrap_or(title.clone()))));
        group.push(figure);
        let mut tags = TagTree::new();
        tags.push(group);
        document.set_tag_tree(tags);
        let mut outline = Outline::new();
        outline.push_child(OutlineNode::new(
            title,
            XyzDestination::new(0, Point::from_xy(0.0, 0.0)),
        ));
        document.set_outline(outline);
    }
    document.finish().map_err(|e| failed(format!("{e:?}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The Unix epoch, a leap day and a moment past 2100 come out as their
    /// calendar dates.
    #[test]
    fn dates_from_unix_seconds() {
        let date = |s| {
            let d = Date::from_unix(s);
            (d.year, d.month, d.day, d.hour, d.minute, d.second)
        };
        assert_eq!(date(0), (1970, 1, 1, 0, 0, 0));
        assert_eq!(date(951_782_400), (2000, 2, 29, 0, 0, 0));
        assert_eq!(date(1_791_158_399), (2026, 10, 4, 23, 59, 59));
        assert_eq!(date(4_107_542_400), (2100, 3, 1, 0, 0, 0));
    }
}
