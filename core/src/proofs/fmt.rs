// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The text rendering of a derivation: a tree of sequents with a bar and
//! the rule's name over each conclusion. The layout is computed in two
//! passes over the inferences, sizes from the leaves down and positions
//! from the root up, and written row by row, so that drawing a tree
//! takes time proportional to its text and memory proportional to its
//! inferences.

use super::derivation::{Derivation, InfId, Inference, Rule};
use super::style::{Labels, OpenGoal, RUN, WriteError, plain};
use crate::occurrences::{Forest, OccId, Position, Reading};
use std::fmt::{Display, Formatter, Result as FmtResult, Write};

/// What a user may vary in the text tree of a derivation.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serialize", serde(default, deny_unknown_fields))]
pub struct TextOptions {
    /// The labels of the rules.
    pub labels: Labels,
    /// How an open goal is drawn; a dashed line is a bar of `╌`.
    pub open: OpenGoal,
    /// The character of an inference's bar.
    pub bar: char,
    /// The columns between two premises, at most 65 535: the gaps are
    /// written as spaces, which the bound on a derivation's size does not
    /// count.
    pub gap: u16,
}

impl TextOptions {
    /// The default bar character.
    pub const BAR: char = '─';
    /// The default gap between premises, in columns.
    pub const GAP: u16 = 3;
}

impl Default for TextOptions {
    /// Returns upright labels, an open goal as its bare sequent, bars of
    /// [`BAR`](Self::BAR) and premises [`GAP`](Self::GAP) columns apart.
    fn default() -> Self {
        Self {
            labels: Labels::Upright,
            open: OpenGoal::Bare,
            bar: Self::BAR,
            gap: Self::GAP,
        }
    }
}

/// What the bar row of an inference holds: the bar's character, its
/// label as text, and whether the bar is the one character of vertical
/// dots over an open goal rather than as wide as the inference.
struct Bar {
    /// The character the bar is drawn with.
    line: char,
    /// The label after the bar, empty for none.
    label: String,
    /// Whether the bar is one character wide, centred.
    dots: bool,
}

/// Returns the bar row of every rule, in declaration order, under the
/// options, and then of every rule for a run of it ([`bar`]): `None` for
/// an open goal drawn bare.
fn bars(options: &TextOptions) -> Vec<Option<Bar>> {
    let runs = Rule::ALL.iter().map(|&rule| {
        Some(Bar {
            line: options.bar,
            label: options
                .labels
                .markup(rule)
                .map_or_else(String::new, |label| plain(label) + RUN),
            dots: false,
        })
    });
    Rule::ALL
        .iter()
        .map(|&rule| {
            let bar = |line, label: &str, dots| Bar {
                line,
                label: plain(label),
                dots,
            };
            if rule != Rule::Open {
                return Some(bar(
                    options.bar,
                    options.labels.markup(rule).unwrap_or(""),
                    false,
                ));
            }
            match &options.open {
                OpenGoal::Bare => None,
                OpenGoal::Dots => Some(bar('⋮', "", true)),
                OpenGoal::Dashed => Some(bar('╌', "", false)),
                OpenGoal::Mark(mark) => Some(bar(options.bar, mark, false)),
            }
        })
        .chain(runs)
        .collect()
}

/// Returns the bar row of an inference among those [`bars`] returns.
fn bar<'a>(bars: &'a [Option<Bar>], inference: &Inference) -> &'a Option<Bar> {
    let run = if inference.times > 1 {
        Rule::ALL.len()
    } else {
        0
    };
    &bars[inference.rule as usize + run]
}

/// Where an inference and the subtree above it go, within the box that
/// holds the subtree.
#[derive(Clone, Copy, Debug, Default)]
struct Place {
    /// The left end of the box: relative to the box of the inference
    /// below after the first pass, in the whole drawing after the second.
    x: usize,
    /// The width of the box in characters.
    width: usize,
    /// The lines of the box.
    height: usize,
    /// The inferences below it, down to the root.
    depth: usize,
    /// The column of the box where the conclusion starts.
    left: usize,
    /// The characters of the conclusion.
    conclusion: usize,
    /// The column of the box where the bar starts and the bar's length, or
    /// none for an open goal.
    bar: Option<(usize, usize)>,
}

/// A piece of one row of the drawing: an inference's conclusion or its
/// bar with the rule's name.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Piece {
    /// The row, from the top.
    row: usize,
    /// The column it starts in.
    column: usize,
    /// The inference.
    id: InfId,
    /// Whether it is the bar rather than the conclusion.
    bar: bool,
}

/// A writer that counts the characters written to it.
struct Count(usize);

impl Write for Count {
    fn write_str(&mut self, s: &str) -> FmtResult {
        self.0 += s.chars().count();
        Ok(())
    }
}

/// Writes `width` spaces, a run at a time: a width in a format string
/// is at most 65 535, and a tree's columns are not.
fn spaces(out: &mut impl Write, width: usize) -> FmtResult {
    const RUN: &str = "                                                                ";
    let mut left = width;
    while left > 0 {
        let n = left.min(RUN.len());
        out.write_str(&RUN[..n])?;
        left -= n;
    }
    Ok(())
}

/// Writes a sequent: `⊢` and its formulas, comma-separated; two-sided
/// under a reading, the hypotheses before `⊢` and the goal after it.
fn write_sequent(
    out: &mut impl Write,
    forest: &Forest,
    reading: Option<&Reading>,
    sequent: &[OccId],
) -> FmtResult {
    let Some(reading) = reading else {
        out.write_char('⊢')?;
        for (i, &o) in sequent.iter().enumerate() {
            out.write_str(if i == 0 { " " } else { ", " })?;
            write!(out, "{}", forest.formula(o))?;
        }
        return Ok(());
    };
    let (mut goal, mut first) = (None, true);
    for &o in sequent {
        if reading.position(o) == Position::Output {
            goal = Some(o);
            continue;
        }
        if !first {
            out.write_str(", ")?;
        }
        first = false;
        write!(out, "{}", reading.formula(o))?;
    }
    if !first {
        out.write_char(' ')?;
    }
    out.write_char('⊢')?;
    if let Some(goal) = goal {
        write!(out, " {}", reading.formula(goal))?;
    }
    Ok(())
}

/// Returns the text of a sequent: `⊢` and its formulas, comma-separated;
/// two-sided under a reading, the hypotheses before `⊢` and the goal after
/// it.
pub(crate) fn sequent_text(
    forest: &Forest,
    reading: Option<&Reading>,
    sequent: &[OccId],
) -> String {
    let mut text = String::new();
    write_sequent(&mut text, forest, reading, sequent).unwrap();
    text
}

impl Derivation<'_> {
    /// Lays the tree out: the premises of an inference side by side,
    /// bottom-aligned and `gap` columns apart, a bar spanning their
    /// conclusions or the conclusion, whichever is wider, with the rule's
    /// label after it, and the conclusion centred under the bar. Returns
    /// every inference's place, its box positioned in the whole drawing.
    fn layout(&self, options: &TextOptions, bars: &[Option<Bar>]) -> Vec<Place> {
        let mut places = vec![Place::default(); self.inferences().len()];
        // Sizes: an inference comes after its premises.
        for (i, inference) in self.inferences().iter().enumerate() {
            let mut count = Count(0);
            write_sequent(
                &mut count,
                self.forest(),
                self.reading(),
                &inference.sequent,
            )
            .expect("counting never fails");
            let conclusion = count.0;
            let Some(bar) = bar(bars, inference) else {
                places[i] = Place {
                    width: conclusion,
                    height: 1,
                    conclusion,
                    ..Place::default()
                };
                continue;
            };
            // The premises in a row, and the span of their conclusions.
            let (mut row, mut height) = (0, 0);
            let (mut span_left, mut span_right) = (0, 0);
            for (k, p) in inference.premises.iter().enumerate() {
                let place = &mut places[p.index()];
                place.x = row + if k == 0 { 0 } else { usize::from(options.gap) };
                if k == 0 {
                    span_left = place.left;
                }
                span_right = place.x + place.left + place.conclusion;
                row = place.x + place.width;
                height = height.max(place.height);
            }
            // The bar covers the premises' conclusions and the conclusion,
            // whichever is wider, centred on the other; a bar that would
            // start left of the box moves the premises right instead.
            let span = span_right - span_left;
            let bar_width = conclusion.max(span);
            let overhang = (bar_width - span) / 2;
            let shift = overhang.saturating_sub(span_left);
            for p in &inference.premises {
                places[p.index()].x += shift;
            }
            let bar_left = span_left + shift - overhang;
            let left = bar_left + (bar_width - conclusion) / 2;
            let (bar_left, bar_width) = if bar.dots {
                (left + conclusion.saturating_sub(1) / 2, 1)
            } else {
                (bar_left, bar_width)
            };
            let label = match bar.label.chars().count() {
                0 => 0,
                n => n + 1,
            };
            let premises = if inference.premises.is_empty() {
                0
            } else {
                row + shift
            };
            places[i] = Place {
                x: 0,
                width: premises
                    .max(bar_left + bar_width + label)
                    .max(left + conclusion),
                height: height + 2,
                depth: 0,
                left,
                conclusion,
                bar: Some((bar_left, bar_width)),
            };
        }
        // Positions: an inference comes before its premises.
        for (i, inference) in self.inferences().iter().enumerate().rev() {
            let (x, depth) = (places[i].x, places[i].depth);
            for p in &inference.premises {
                places[p.index()].x += x;
                places[p.index()].depth = depth + 1;
            }
        }
        places
    }

    /// Returns the width in characters and the number of lines of the tree
    /// that [`write_text`](Self::write_text) draws under the options,
    /// without drawing it.
    pub fn text_size(&self, options: &TextOptions) -> (usize, usize) {
        let root = self.layout(options, &bars(options))[self.root().index()];
        (root.width, root.height)
    }

    /// Writes the derivation as a tree of sequents, one line per row,
    /// without a trailing newline, and asks `stop` before every piece of
    /// a row. The layout takes memory proportional to the inferences and
    /// the text is written as it is made.
    pub fn write_text(
        &self,
        options: &TextOptions,
        out: &mut impl Write,
        mut stop: impl FnMut() -> bool,
    ) -> Result<(), WriteError> {
        let bars = bars(options);
        let places = self.layout(options, &bars);
        let height = places[self.root().index()].height;
        // The root's conclusion is the last row, its bar the one above,
        // and every inference further up is two rows higher.
        let mut pieces = Vec::with_capacity(2 * places.len());
        for (i, place) in places.iter().enumerate() {
            let id = InfId::new(i as u32);
            let row = height - 1 - 2 * place.depth;
            pieces.push(Piece {
                row,
                column: place.x + place.left,
                id,
                bar: false,
            });
            if let Some((left, _)) = place.bar {
                pieces.push(Piece {
                    row: row - 1,
                    column: place.x + left,
                    id,
                    bar: true,
                });
            }
        }
        pieces.sort_unstable();
        let (mut row, mut column) = (0, 0);
        for piece in pieces {
            if stop() {
                return Err(WriteError::Stopped);
            }
            while row < piece.row {
                out.write_char('\n')?;
                (row, column) = (row + 1, 0);
            }
            spaces(out, piece.column - column)?;
            let (place, inference) = (&places[piece.id.index()], self.inference(piece.id));
            column = piece.column;
            match (place.bar, bar(&bars, inference)) {
                (Some((_, width)), Some(bar)) if piece.bar => {
                    for _ in 0..width {
                        out.write_char(bar.line)?;
                    }
                    column += width;
                    if !bar.label.is_empty() {
                        write!(out, " {}", bar.label)?;
                        column += 1 + bar.label.chars().count();
                    }
                }
                _ => {
                    write_sequent(out, self.forest(), self.reading(), &inference.sequent)?;
                    column += place.conclusion;
                }
            }
        }
        Ok(())
    }
}

impl Derivation<'_> {
    /// Writes the derivation as a numbered list of its inferences, one per
    /// line, premises before their conclusion and the conclusion last:
    /// `3. A, A ⊸ B ⊢ B, by ⊸L from 1 and 2.` An open goal is `open`, a
    /// run of a structural rule `by ?w 3 times`.
    /// This is the reading of a derivation for a screen reader, which
    /// cannot follow a tree. Asks `stop` before every line.
    pub fn write_steps(
        &self,
        out: &mut impl Write,
        mut stop: impl FnMut() -> bool,
    ) -> Result<(), WriteError> {
        for (i, inference) in self.inferences().iter().enumerate() {
            if stop() {
                return Err(WriteError::Stopped);
            }
            if i > 0 {
                out.write_char('\n')?;
            }
            write!(out, "{}. ", i + 1)?;
            write_sequent(out, self.forest(), self.reading(), &inference.sequent)?;
            if inference.rule == Rule::Open {
                out.write_str(", open.")?;
                continue;
            }
            write!(out, ", by {}", inference.rule.name())?;
            if inference.times > 1 {
                write!(out, " {} times", inference.times)?;
            }
            for (k, p) in inference.premises.iter().enumerate() {
                let joint = match k {
                    0 => " from ",
                    _ if k + 1 == inference.premises.len() => " and ",
                    _ => ", ",
                };
                write!(out, "{joint}{}", p.index() + 1)?;
            }
            out.write_char('.')?;
        }
        Ok(())
    }
}

impl Display for Derivation<'_> {
    /// Draws the derivation as a tree of sequents under the default
    /// [`TextOptions`], one line per row, without a trailing newline.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.write_text(&TextOptions::default(), f, || false)
            .map_err(|_| std::fmt::Error)
    }
}

#[cfg(all(test, feature = "parse"))]
mod tests {
    use crate::{Mode, Options, Sequent, Verdict, prove};

    /// A tree whose premises start beyond the widest column a format
    /// string pads to is drawn.
    #[test]
    fn wider_than_a_format_width() {
        let name = "a".repeat(70_000);
        let sequent: Sequent = format!("|- {name}, ~{name} * ~b, b").parse().unwrap();
        let outcome = prove(&sequent, Mode::CLASSICAL, &Options::default()).unwrap();
        let Verdict::Proved(proof) = outcome.verdict else {
            panic!("provable");
        };
        let tree = proof.derivation().unwrap().to_string();
        // The premises' bars, then their conclusions.
        let premises = tree.lines().nth(1).unwrap();
        assert!(premises.len() > 140_000, "{}", premises.len());
        assert!(
            premises.ends_with("⊢ ~b, b"),
            "{}",
            &premises[premises.len() - 40..]
        );
    }
}
