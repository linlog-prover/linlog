// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Derivations as proof trees, laid out bottom-up by subtree width: the
//! premises of an inference side by side, their conclusions centred over
//! its conclusion, an inference line spanning both with the rule's name to
//! its right, and the root at the bottom. Every row is one line height
//! above the row below it.

use super::font::{AXIS, DEPTH, HEIGHT};
use super::{
    Escaping, NOTATION, PLAIN, Run, Style, backdrop, drawn, escaped, head, label, run, text,
};
use crate::export::notation::{Step, flush, walk};
use crate::proofs::style::RUN;
use crate::proofs::{Derivation, Inference, OpenGoal, Rule, WriteError};
use std::fmt::Write;

/// Where an inference and the subtree above it go, within the box that
/// holds the subtree and the rule names in it. A conclusion is laid out
/// again when it is written, so that the layout holds a few numbers per
/// inference and no text.
#[derive(Clone, Copy, Debug, Default)]
struct Place {
    /// The left end of the box, relative to the left end of the box of the
    /// inference below; 0 for the root.
    offset: i64,
    /// The width of the box.
    width: i64,
    /// The width of the conclusion.
    conclusion: i64,
    /// The left end of the conclusion within the box.
    left: i64,
    /// The ends of the inference line within the box, or none for an
    /// open goal drawn without one.
    bar: Option<(i64, i64)>,
}

/// Writes a derivation as an SVG document of its proof tree into `out`,
/// titled with its conclusion in plain text, and asks `stop` after the
/// elements of each inference.
pub(super) fn draw(
    derivation: &Derivation,
    style: &Style,
    out: &mut impl Write,
    mut stop: impl FnMut() -> bool,
) -> Result<(), WriteError> {
    let (forest, reading) = (derivation.forest(), derivation.reading());
    let font = &style.font;
    let line_height = i64::from(style.line_height);
    let label_size = i64::from(style.label_size);
    let (gap, label_gap) = (i64::from(style.premise_gap), i64::from(style.label_gap));
    let (margin, stroke) = (i64::from(style.margin), i64::from(style.stroke_width));
    // An inference line lies halfway between the lowest reach of the
    // premises' text and the highest reach of the conclusion's; a rule's
    // name has its math axis on the line; the dots over an open goal stand
    // where its premises would.
    let rise = (line_height + HEIGHT - DEPTH) / 2;
    let dots = run("⋮", 1000, font);
    let dots_rise = line_height - DEPTH;
    // The label of every rule, and of an open goal its mark, laid out
    // once; then the label of every rule for a run of it.
    let runs = Rule::ALL
        .iter()
        .map(|&rule| style.labels.markup(rule).map(|m| format!("{m}{RUN}")));
    let labels: Vec<Option<Run>> = Rule::ALL
        .iter()
        .map(|&rule| match (rule, &style.open) {
            (Rule::Open, OpenGoal::Mark(mark)) => Some(mark.clone()),
            _ => style.labels.markup(rule).map(str::to_owned),
        })
        .chain(runs)
        .map(|markup| markup.map(|m| run(&label(&m), label_size, font)))
        .collect();
    let empty = Run::default();
    let label_of = |inference: &Inference| {
        let run = if inference.times > 1 {
            Rule::ALL.len()
        } else {
            0
        };
        labels[inference.rule as usize + run]
            .as_ref()
            .unwrap_or(&empty)
    };
    let bare = matches!(style.open, OpenGoal::Bare);
    let dotted = matches!(style.open, OpenGoal::Dots);
    let marks = style.ids;
    let conclusion = |sequent: &mut String, id| {
        sequent.clear();
        let inference = derivation.inference(id);
        NOTATION.sequent(sequent, forest, reading, &inference.sequent, false, marks);
        run(sequent, 1000, font)
    };

    // Up the tree: sizes, the premises' offsets, and the highest point of
    // the drawing relative to the root's baseline.
    let mut places = vec![Place::default(); derivation.inferences().len()];
    let mut top = -HEIGHT;
    let mut sequent = String::new();
    walk(derivation, |step| {
        let Step::Exit(id, depth) = step else {
            return Ok::<_, WriteError>(());
        };
        let inference = derivation.inference(id);
        let width = conclusion(&mut sequent, id).width;
        let baseline = -(depth as i64) * line_height;
        top = top.min(baseline - HEIGHT);
        if inference.rule == Rule::Open && (bare || dotted) {
            if dotted {
                top = top.min(baseline - dots_rise - HEIGHT);
            }
            let reach = if dotted { dots.width } else { 0 };
            places[id.index()] = Place {
                width: width.max(reach),
                conclusion: width,
                left: (reach - width).max(0) / 2,
                ..Place::default()
            };
            return Ok(());
        }
        let label = label_of(inference);
        top = top.min(baseline - rise - stroke / 2);
        if !label.pieces.is_empty() {
            top = top.min(baseline - rise + (AXIS - HEIGHT) * label_size / 1000);
        }

        // The premises side by side from 0, and the span of their
        // conclusions.
        let (mut x, mut span) = (0, None);
        for &p in &inference.premises {
            let place = &mut places[p.index()];
            place.offset = x;
            let (start, end) = (x + place.left, x + place.left + place.conclusion);
            span = Some(span.map_or((start, end), |(s, _)| (s, end)));
            x += place.width + gap;
        }
        let row = (x - gap).max(0);
        let (left, bar) = match span {
            Some((start, end)) => {
                let left = (start + end) / 2 - width / 2;
                (left, (start.min(left), end.max(left + width)))
            }
            None => (0, (0, width)),
        };
        let shift = -left.min(0);
        for &p in &inference.premises {
            places[p.index()].offset += shift;
        }
        let named = if label.pieces.is_empty() {
            0
        } else {
            label_gap + label.width
        };
        let right = row.max(bar.1 + named);
        places[id.index()] = Place {
            offset: 0,
            width: right + shift,
            conclusion: width,
            left: left + shift,
            bar: Some((bar.0 + shift, bar.1 + shift)),
        };
        Ok(())
    })?;

    // Down the tree: every box at its place.
    let mut xs = vec![0; places.len()];
    xs[derivation.root().index()] = margin;
    walk(derivation, |step| {
        if let Step::Enter(id, _) = step {
            for &p in &derivation.inference(id).premises {
                xs[p.index()] = xs[id.index()] + places[p.index()].offset;
            }
        }
        Ok::<_, WriteError>(())
    })?;
    let y = |depth: usize| margin - top - depth as i64 * line_height;

    let root = derivation.root();
    let mut title = String::new();
    let root_sequent = &derivation.inference(root).sequent;
    PLAIN.sequent(&mut title, forest, reading, root_sequent, false, false);
    let width = places[root.index()].width + 2 * margin;
    let size = (width, DEPTH - top + 2 * margin);
    head(out, style, &title, size)?;
    if style.description {
        out.write_str("<desc>")?;
        derivation.write_steps(&mut Escaping(out), &mut stop)?;
        out.write_str("</desc>\n")?;
    }
    backdrop(out, style, size)?;

    // The lines, then the texts, each in the order of the walk.
    writeln!(
        out,
        "<g fill=\"none\" stroke=\"{}\" stroke-width=\"{stroke}\">",
        escaped(&style.line)
    )?;
    let mut buffer = String::new();
    let dashed = matches!(style.open, OpenGoal::Dashed);
    walk(derivation, |step| {
        let Step::Enter(id, depth) = step else {
            return Ok(());
        };
        let (place, x) = (&places[id.index()], xs[id.index()]);
        if let Some((start, end)) = place.bar {
            let y = y(depth) - rise;
            write!(buffer, r#"<path d="M{} {y}H{}""#, x + start, x + end)?;
            if dashed && derivation.inference(id).rule == Rule::Open {
                write!(buffer, r#" stroke-dasharray="{0} {0}""#, 3 * stroke)?;
            }
            buffer.push_str("/>\n");
        }
        flush(out, &mut buffer, &mut stop)
    })?;
    out.write_str("</g>\n")?;
    let size = format!(r#" font-size="{label_size}""#);
    let mut positions = Vec::new();
    walk(derivation, |step| {
        let Step::Enter(id, depth) = step else {
            return Ok(());
        };
        let (place, x, y) = (&places[id.index()], xs[id.index()], y(depth));
        let inference = derivation.inference(id);
        let attributes = format!(r#" id="i{}""#, id.get());
        let ids = marks.then(|| {
            positions = drawn(reading, &inference.sequent);
            (id.get(), positions.as_slice())
        });
        let line = conclusion(&mut sequent, id);
        text(&mut buffer, x + place.left, y, &line, &attributes, ids);
        match place.bar {
            None if dotted => {
                let dots_x = x + (place.width - dots.width) / 2;
                text(&mut buffer, dots_x, y - dots_rise, &dots, "", None);
            }
            None => {}
            Some((_, end)) => {
                let label = label_of(inference);
                if !label.pieces.is_empty() {
                    let baseline = y - rise + AXIS * label_size / 1000;
                    text(
                        &mut buffer,
                        x + end + label_gap,
                        baseline,
                        label,
                        &size,
                        None,
                    );
                }
            }
        }
        flush(out, &mut buffer, &mut stop)
    })?;
    out.write_str("</svg>")?;
    Ok(())
}
