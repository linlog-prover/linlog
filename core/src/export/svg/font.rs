// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! What the layout knows of the font: the advance of every character the
//! drawings set, from the tables of four math fonts (Euler Math 0.75 by
//! default, STIX Two Math 2.12, Garamond-Math 2022-01-03 and Asana Math
//! 0.962, each 1000 units per em), and the vertical metrics of a line,
//! which are Euler Math's for every font. The advances were read from
//! each font's `hmtx` table once; nothing is measured at run time.

use std::collections::BTreeMap;

/// The font a drawing asks for and the advances its layout gives the
/// characters. In JSON (feature `serialize`) the family and the advances,
/// or the name of a [`MathFont`] alone (`"stix-two"`), which reads as
/// that font's [`MathFont::font`].
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
pub struct Font {
    /// The font families the document asks for, as CSS lists them.
    pub family: String,
    /// The advance of every character.
    pub advances: Advances,
}

impl Font {
    /// Euler Math, with Neo Euler and any serif font after it, and its
    /// own advances.
    pub fn euler() -> Self {
        Self {
            family: "'Euler Math', 'Neo Euler', serif".to_owned(),
            advances: Advances::Euler,
        }
    }

    /// Any monospace font, every character [`MONOSPACE`] thousandths of
    /// an em wide: for a viewer without a math font.
    pub fn monospace() -> Self {
        Self {
            family: "monospace".to_owned(),
            advances: Advances::Fixed(MONOSPACE),
        }
    }

    /// Returns the font with another [`family`](Self::family).
    #[must_use]
    pub fn with_family(self, family: String) -> Self {
        Self { family, ..self }
    }

    /// Returns the font with another [`advances`](Self::advances).
    #[must_use]
    pub fn with_advances(self, advances: Advances) -> Self {
        Self { advances, ..self }
    }
}

impl Default for Font {
    /// Returns [`Font::euler`].
    fn default() -> Self {
        Self::euler()
    }
}

#[cfg(feature = "serialize")]
impl<'de> serde::Deserialize<'de> for Font {
    /// Reads a font's family and advances, or a [`MathFont`]'s name.
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        /// A font's fields as JSON writes them, each left out the default
        /// font's.
        #[derive(serde::Deserialize)]
        #[serde(default, deny_unknown_fields)]
        struct Fields {
            /// [`Font::family`].
            family: String,
            /// [`Font::advances`].
            advances: Advances,
        }

        impl Default for Fields {
            /// Returns the default font's fields.
            fn default() -> Self {
                let Font { family, advances } = Font::default();
                Self { family, advances }
            }
        }

        /// Reads either form.
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = Font;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a font's family and advances, or the name of a math font")
            }

            fn visit_str<E: serde::de::Error>(self, name: &str) -> Result<Font, E> {
                use serde::Deserialize;
                use serde::de::IntoDeserializer;
                MathFont::deserialize(name.into_deserializer()).map(MathFont::font)
            }

            fn visit_map<A: serde::de::MapAccess<'de>>(self, map: A) -> Result<Font, A::Error> {
                use serde::Deserialize;
                let Fields { family, advances } =
                    Fields::deserialize(serde::de::value::MapAccessDeserializer::new(map))?;
                Ok(Font { family, advances })
            }
        }

        deserializer.deserialize_any(Visitor)
    }
}

/// A math font whose advances the layout knows, read from its file. In
/// JSON (feature `serialize`) `"euler"`, `"stix-two"`, `"garamond"` or
/// `"asana"`. Each sets every character the drawings set; a renderer
/// needs the font's file under its [`family`](Self::family) name.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serialize", serde(rename_all = "kebab-case"))]
pub enum MathFont {
    /// Euler Math 0.75, Hermann Zapf's upright Euler for the AMS as an
    /// OpenType math font, under the SIL Open Font License.
    #[default]
    Euler,
    /// STIX Two Math 2.12, a Times-like font of the STIX project, under
    /// the SIL Open Font License.
    StixTwo,
    /// Garamond-Math 2022-01-03, matching EB Garamond, under the SIL Open
    /// Font License.
    Garamond,
    /// Asana Math 0.962, Palatino-like, under the SIL Open Font License.
    Asana,
}

impl MathFont {
    /// Every math font, the default first.
    pub const ALL: [Self; 4] = [Self::Euler, Self::StixTwo, Self::Garamond, Self::Asana];

    /// Returns the family name in the font's file, which a renderer
    /// looks the font up by.
    pub const fn family(self) -> &'static str {
        match self {
            Self::Euler => "Euler Math",
            Self::StixTwo => "STIX Two Math",
            Self::Garamond => "Garamond-Math",
            Self::Asana => "Asana Math",
        }
    }

    /// Returns the column of the font's advances in the table.
    const fn column(self) -> usize {
        match self {
            Self::Euler => 0,
            Self::StixTwo => 1,
            Self::Garamond => 2,
            Self::Asana => 3,
        }
    }

    /// Returns the font a drawing is set in: this family, then any serif
    /// font (Euler Math with Neo Euler between, [`Font::euler`]), and its
    /// advances.
    pub fn font(self) -> Font {
        match self {
            Self::Euler => Font::euler(),
            Self::StixTwo => Font {
                family: "'STIX Two Math', serif".to_owned(),
                advances: Advances::StixTwo,
            },
            Self::Garamond => Font {
                family: "'Garamond-Math', serif".to_owned(),
                advances: Advances::Garamond,
            },
            Self::Asana => Font {
                family: "'Asana Math', serif".to_owned(),
                advances: Advances::Asana,
            },
        }
    }
}

/// The advance of a character in [`Font::monospace`], in thousandths of
/// an em: what most monospace fonts have.
pub const MONOSPACE: u32 = 600;

/// The advance widths a layout gives characters, in thousandths of an em.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serialize", serde(rename_all = "kebab-case"))]
pub enum Advances {
    /// The committed table of Euler Math, with a fallback of 650 outside
    /// it.
    Euler,
    /// The committed table of STIX Two Math, with the same fallback.
    StixTwo,
    /// The committed table of Garamond-Math, with the same fallback.
    Garamond,
    /// The committed table of Asana Math, with the same fallback.
    Asana,
    /// One advance for every character.
    Fixed(u32),
    /// A table of the user's, and the advance of a character outside it.
    #[non_exhaustive]
    Table {
        /// The advance of each character.
        table: BTreeMap<char, u32>,
        /// The advance of a character outside the table.
        fallback: u32,
    },
}

impl Advances {
    /// Returns the advances of a table of the caller's: each character's
    /// in `table`, and `fallback` for a character outside it.
    pub fn table(table: BTreeMap<char, u32>, fallback: u32) -> Self {
        Self::Table { table, fallback }
    }

    /// Returns the advance of a character, in thousandths of an em.
    pub fn advance(&self, c: char) -> u32 {
        match self {
            Self::Euler => advance(c, MathFont::Euler),
            Self::StixTwo => advance(c, MathFont::StixTwo),
            Self::Garamond => advance(c, MathFont::Garamond),
            Self::Asana => advance(c, MathFont::Asana),
            Self::Fixed(advance) => *advance,
            Self::Table { table, fallback } => table.get(&c).copied().unwrap_or(*fallback),
        }
    }
}

/// Every character the drawings set, sorted: printable ASCII, the
/// mathematical italic Latin letters, Greek, and the connectives, units,
/// turnstile and vertical dots. Kept apart from the advances, so that the
/// binary search reads the characters alone.
const CHARACTERS: [char; 208] = [
    ' ', '!', '"', '#', '$', '%', '&', '\'', '(', ')', '*', '+', ',', '-', '.', '/', '0', '1', '2',
    '3', '4', '5', '6', '7', '8', '9', ':', ';', '<', '=', '>', '?', '@', 'A', 'B', 'C', 'D', 'E',
    'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X',
    'Y', 'Z', '[', '\\', ']', '^', '_', '`', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k',
    'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '{', '|', '}', '~',
    '¬', 'Α', 'Β', 'Γ', 'Δ', 'Ε', 'Ζ', 'Η', 'Θ', 'Ι', 'Κ', 'Λ', 'Μ', 'Ν', 'Ξ', 'Ο', 'Π', 'Ρ', 'Σ',
    'Τ', 'Υ', 'Φ', 'Χ', 'Ψ', 'Ω', 'α', 'β', 'γ', 'δ', 'ε', 'ζ', 'η', 'θ', 'ι', 'κ', 'λ', 'μ', 'ν',
    'ξ', 'ο', 'π', 'ρ', 'σ', 'τ', 'υ', 'φ', 'χ', 'ψ', 'ω', 'ℎ', '⅋', '→', '↔', '∧', '∨', '⊕', '⊗',
    '⊢', '⊤', '⊥', '⊸', '⋮', '𝐴', '𝐵', '𝐶', '𝐷', '𝐸', '𝐹', '𝐺', '𝐻', '𝐼', '𝐽', '𝐾', '𝐿', '𝑀', '𝑁',
    '𝑂', '𝑃', '𝑄', '𝑅', '𝑆', '𝑇', '𝑈', '𝑉', '𝑊', '𝑋', '𝑌', '𝑍', '𝑎', '𝑏', '𝑐', '𝑑', '𝑒', '𝑓', '𝑔',
    '𝑖', '𝑗', '𝑘', '𝑙', '𝑚', '𝑛', '𝑜', '𝑝', '𝑞', '𝑟', '𝑠', '𝑡', '𝑢', '𝑣', '𝑤', '𝑥', '𝑦', '𝑧',
];

/// The advance width of each of the [`CHARACTERS`], in thousandths of an
/// em, in the same order: one table per [`MathFont`], in the order of
/// [`MathFont::ALL`].
#[rustfmt::skip]
const ADVANCES: [[u16; 208]; 4] = [
    [
        333, 295, 214, 500, 557, 840, 737, 212, 328, 328, 277, 755,
        277, 544, 277, 501, 499, 499, 499, 499, 499, 499, 499, 499,
        499, 499, 216, 216, 800, 756, 800, 362, 744, 770, 655, 714,
        828, 604, 499, 765, 783, 394, 402, 668, 559, 1044, 829, 803,
        576, 828, 609, 557, 514, 774, 646, 986, 666, 555, 666, 290,
        501, 290, 499, 450, 201, 609, 588, 486, 603, 499, 419, 568,
        621, 360, 331, 555, 365, 915, 664, 563, 589, 605, 432, 455,
        416, 642, 495, 812, 526, 593, 470, 345, 213, 345, 551, 773,
        770, 655, 428, 713, 604, 666, 783, 757, 394, 668, 770, 1044,
        829, 596, 803, 722, 576, 646, 492, 716, 833, 666, 703, 875,
        658, 662, 608, 501, 486, 512, 560, 554, 334, 555, 541, 617,
        599, 553, 563, 609, 548, 605, 513, 587, 763, 576, 754, 851,
        621, 737, 1000, 1000, 775, 775, 668, 668, 698, 874, 874, 1010,
        768, 770, 655, 714, 828, 604, 499, 765, 783, 394, 402, 668,
        559, 1044, 829, 803, 576, 828, 609, 557, 492, 774, 646, 986,
        666, 555, 666, 609, 588, 486, 603, 499, 419, 568, 360, 331,
        555, 365, 915, 664, 563, 589, 605, 432, 455, 416, 642, 495,
        812, 526, 593, 470,
    ],
    [
        235, 283, 334, 702, 495, 692, 736, 187, 357, 357, 410, 720,
        245, 315, 245, 386, 495, 495, 495, 495, 495, 495, 495, 495,
        495, 495, 245, 245, 720, 720, 720, 411, 905, 718, 618, 670,
        730, 626, 588, 720, 777, 357, 396, 702, 585, 895, 742, 738,
        569, 737, 659, 504, 603, 752, 680, 944, 664, 647, 594, 349,
        385, 349, 480, 512, 222, 480, 522, 446, 539, 464, 307, 484,
        577, 285, 263, 542, 283, 852, 576, 514, 537, 521, 380, 395,
        315, 570, 481, 749, 479, 483, 458, 327, 267, 327, 557, 720,
        718, 618, 569, 667, 626, 594, 777, 738, 357, 702, 710, 895,
        742, 648, 738, 759, 569, 604, 603, 647, 718, 664, 720, 777,
        561, 530, 466, 479, 381, 386, 510, 514, 259, 495, 506, 552,
        453, 390, 475, 591, 515, 545, 392, 491, 603, 462, 620, 660,
        569, 736, 948, 948, 636, 636, 862, 862, 720, 720, 720, 869,
        519, 681, 593, 656, 702, 614, 583, 697, 757, 343, 398, 655,
        569, 869, 714, 707, 569, 707, 620, 534, 610, 731, 640, 913,
        637, 607, 606, 555, 518, 414, 556, 424, 577, 503, 274, 431,
        533, 278, 830, 565, 486, 598, 517, 412, 386, 338, 565, 504,
        727, 559, 510, 493,
    ],
    [
        200, 250, 330, 468, 441, 650, 756, 200, 375, 375, 340, 600,
        230, 274, 230, 489, 489, 489, 489, 489, 489, 489, 489, 489,
        489, 489, 248, 228, 554, 566, 554, 381, 749, 692, 587, 710,
        754, 564, 512, 729, 810, 340, 340, 681, 584, 901, 792, 763,
        552, 763, 713, 465, 670, 738, 672, 916, 707, 578, 603, 341,
        489, 341, 500, 500, 200, 399, 515, 407, 506, 390, 318, 435,
        515, 245, 226, 469, 240, 778, 528, 495, 519, 522, 334, 323,
        314, 527, 438, 685, 430, 438, 377, 416, 352, 416, 500, 600,
        692, 587, 517, 676, 564, 603, 810, 773, 340, 681, 692, 901,
        792, 640, 763, 828, 552, 603, 670, 739, 745, 707, 745, 830,
        452, 510, 430, 418, 334, 329, 460, 430, 238, 468, 469, 537,
        405, 374, 400, 521, 426, 478, 410, 434, 537, 438, 560, 562,
        500, 756, 1000, 1000, 572, 572, 706, 706, 553, 554, 554, 685,
        210, 707, 623, 671, 780, 626, 603, 699, 821, 424, 483, 767,
        552, 1062, 858, 727, 627, 751, 647, 557, 635, 774, 693, 1065,
        752, 614, 661, 490, 462, 353, 570, 379, 577, 485, 340, 420,
        473, 339, 758, 509, 416, 522, 515, 422, 327, 347, 494, 431,
        642, 498, 451, 431,
    ],
    [
        249, 277, 370, 499, 499, 839, 777, 277, 332, 332, 388, 668,
        249, 332, 249, 286, 499, 443, 499, 499, 499, 499, 499, 499,
        499, 499, 249, 249, 668, 668, 668, 443, 746, 777, 610, 708,
        793, 610, 555, 762, 831, 336, 332, 725, 610, 945, 830, 785,
        603, 785, 667, 524, 612, 777, 721, 1000, 666, 666, 666, 332,
        286, 332, 605, 499, 277, 499, 552, 443, 610, 478, 332, 555,
        581, 290, 233, 555, 290, 882, 581, 545, 600, 600, 394, 423,
        325, 602, 564, 833, 515, 555, 499, 332, 210, 332, 605, 673,
        777, 610, 555, 688, 610, 666, 831, 785, 336, 725, 753, 945,
        830, 692, 785, 812, 603, 684, 612, 733, 755, 666, 870, 824,
        594, 512, 581, 497, 480, 504, 531, 613, 310, 571, 618, 610,
        497, 547, 545, 653, 566, 552, 519, 547, 681, 608, 695, 715,
        499, 777, 1013, 1013, 687, 687, 668, 668, 748, 748, 748, 1016,
        249, 751, 610, 687, 777, 610, 555, 721, 777, 332, 332, 666,
        555, 953, 777, 777, 610, 777, 666, 555, 610, 777, 721, 943,
        721, 666, 666, 477, 462, 406, 499, 388, 550, 499, 277, 277,
        468, 277, 777, 555, 443, 499, 462, 388, 388, 332, 555, 499,
        721, 499, 499, 443,
    ],
];

/// The advance of a character outside the table, in thousandths of an em:
/// a viewer sets it in some other font, and `textLength` makes the line
/// fit whatever that font's width is.
const FALLBACK: u32 = 650;

/// How far the highest character of a line reaches above the baseline, a
/// raised `⊥` of a negated atom, in thousandths of an em.
pub(super) const HEIGHT: i64 = 890;

/// How far the lowest character of a line reaches below the baseline, the
/// comma, in thousandths of an em.
pub(super) const DEPTH: i64 = 210;

/// The height of the math axis above the baseline, where `⊗`, `⊸` and
/// the inference lines sit, in thousandths of an em.
pub(super) const AXIS: i64 = 250;

/// The size of a superscript or subscript, in thousandths of the size of
/// the text it belongs to.
pub(super) const SCRIPT: i64 = 700;

/// How far a superscript's baseline is raised, in thousandths of an em of
/// the text it belongs to.
pub(super) const RAISE: i64 = 400;

/// How far a subscript's baseline is lowered, in thousandths of an em of
/// the text it belongs to.
pub(super) const LOWER: i64 = 150;

/// Returns the advance of a character in a math font, in thousandths of
/// an em.
fn advance(c: char, font: MathFont) -> u32 {
    match CHARACTERS.binary_search(&c) {
        Ok(i) => u32::from(ADVANCES[font.column()][i]),
        Err(_) => FALLBACK,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use skrifa::prelude::{FontRef, LocationRef, MetadataProvider, Size};

    /// The table is sorted, so that the binary search finds every entry.
    #[test]
    fn advances_are_sorted() {
        assert!(CHARACTERS.windows(2).all(|w| w[0] < w[1]));
        assert_eq!(advance('⊗', MathFont::Euler), 668);
        assert_eq!(advance('\u{FFFD}', MathFont::StixTwo), FALLBACK);
        assert!(
            MathFont::ALL
                .iter()
                .enumerate()
                .all(|(i, font)| font.column() == i)
        );
    }

    /// Each font's column is the advances of its file, the command's copy
    /// in `cli/fonts`, and the file has a glyph for every character of
    /// the table, so no character a drawing sets falls back to another
    /// font.
    #[test]
    fn every_table_is_its_font_s() {
        let files = [
            "Euler-Math.otf",
            "STIXTwoMath-Regular.otf",
            "Garamond-Math.otf",
            "Asana-Math.otf",
        ];
        for (font, file) in MathFont::ALL.into_iter().zip(files) {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../cli/fonts")
                .join(file);
            let data = std::fs::read(&path).unwrap();
            let face = FontRef::new(&data).unwrap();
            assert_eq!(
                face.metrics(Size::unscaled(), LocationRef::default())
                    .units_per_em,
                1000
            );
            let (charmap, metrics) = (
                face.charmap(),
                face.glyph_metrics(Size::unscaled(), LocationRef::default()),
            );
            for (&c, &advance) in CHARACTERS.iter().zip(&ADVANCES[font.column()]) {
                let glyph = charmap.map(c).filter(|g| g.to_u32() != 0);
                let glyph = glyph.unwrap_or_else(|| panic!("{file} has no glyph for {c:?}"));
                let width = metrics.advance_width(glyph).unwrap();
                assert_eq!(width, f32::from(advance), "{file}: {c:?}");
            }
        }
    }
}
