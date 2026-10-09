// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use std::fmt;
use std::ops::Range;

/// Where and why parsing failed, owned so that it outlives the input: the
/// place as a byte range, as a range of UTF-16 code units and as a line
/// and a character, what was found there and what could have stood there.
///
/// Needs the cargo feature `parse` (on by default).
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
pub struct ParseError {
    /// The byte range of the input where parsing failed: the character
    /// found there, or the empty range at the end of the input.
    pub span: Range<usize>,
    /// The same range in UTF-16 code units, as an editor in JavaScript
    /// indexes the text.
    pub span_utf16: Range<usize>,
    /// The line the place is on, counting from 1.
    pub line: usize,
    /// The character of that line the place is, counting from 1.
    pub column: usize,
    /// The character found there, or `None` at the end of the input.
    pub found: Option<String>,
    /// What could have stood there, in words.
    pub expected: &'static [&'static str],
    /// Whether what was found is a word reserved for a later version of
    /// the syntax (`forall`, `exists`), which names no atom.
    #[cfg_attr(
        feature = "serialize",
        serde(skip_serializing_if = "std::ops::Not::not")
    )]
    pub reserved: bool,
}

impl ParseError {
    /// Returns the error for the character of `input` at byte `at`, or for
    /// the end of the input there.
    pub(crate) fn new(input: &str, at: usize, expected: &'static [&'static str]) -> Self {
        let end = at + input[at..].chars().next().map_or(0, char::len_utf8);
        Self::spanning(input, at..end, expected)
    }

    /// Returns the error for the reserved word of `input` that spans
    /// `span`, where `expected` could have stood.
    pub(crate) fn reserved(
        input: &str,
        span: Range<usize>,
        expected: &'static [&'static str],
    ) -> Self {
        Self {
            reserved: true,
            ..Self::spanning(input, span, expected)
        }
    }

    /// Returns the error for the text of `input` that spans `span`, the
    /// empty span at the end of the input for its end.
    pub(crate) fn spanning(
        input: &str,
        span: Range<usize>,
        expected: &'static [&'static str],
    ) -> Self {
        let before = &input[..span.start];
        let found = &input[span.clone()];
        let start_of_line = before.rfind('\n').map_or(0, |n| n + 1);
        let utf16 = |text: &str| text.chars().map(char::len_utf16).sum::<usize>();
        let start_utf16 = utf16(before);
        Self {
            span_utf16: start_utf16..start_utf16 + utf16(found),
            line: before.matches('\n').count() + 1,
            column: before[start_of_line..].chars().count() + 1,
            found: (!found.is_empty()).then(|| found.to_owned()),
            span,
            expected,
            reserved: false,
        }
    }
}

impl fmt::Display for ParseError {
    /// Writes what was found where parsing failed and what could have
    /// stood there, as in `unexpected "|" at line 1, character 5, expected
    /// a formula`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.found {
            Some(word) if self.reserved => {
                write!(
                    f,
                    "{word:?}, a word reserved for a later version of the syntax,"
                )?;
            }
            Some(token) => write!(f, "unexpected {token:?}")?,
            None => f.write_str("unexpected end of input")?,
        }
        write!(f, " at line {}, character {}", self.line, self.column)?;
        if !self.expected.is_empty() {
            write!(f, ", expected {}", super::or_list(self.expected))?;
        }
        Ok(())
    }
}

impl std::error::Error for ParseError {}
