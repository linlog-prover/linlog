// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Proof structures as formula trees under their axiom links: the
//! literals side by side along the top in occurrence order, which is left
//! to right in every tree, their edges and links meeting each at the
//! middle of its atom (not of a negated literal's whole `A⊥`), each
//! connective one layer below the lower of its premises, the conclusions
//! on the bottom layer, and every link a half-ellipse over the two
//! literals it joins, as high as it is wide times a fixed ratio up to
//! [`Style::link_cap`] and less beyond, so that the arcs of nested pairs
//! of literals still never cross; links over interleaved pairs cross.
//!
//! Drawn with its formulas, every occurrence is its formula's text, a
//! connective's with its own connective under the joint where the edges
//! from its premises meet, and the subtree of each premise as far to that
//! side of the joint as the premise stands in the text: every subformula
//! of a one-sided net is where it is in its conclusion, and since the
//! subtrees of a link's two premises stand on either side of its joint,
//! no edge crosses a formula. Compact, the literals are packed
//! `literal_gap` apart, every connective is a circle halfway between its
//! premises, and the conclusions are written under the edges hanging from
//! the bottom layer, as far apart as their texts need.
//!
//! A structure that is no proof net has its switching cycle, or every
//! part but the first that it falls into, in the highlight colour. Drawn
//! as an essential net, the same layout carries the reading's direction
//! on every edge, as an arrowhead halfway along it, and its polarity in
//! the colours.

use super::font::{AXIS, DEPTH, HEIGHT};
use super::{NOTATION, NetDrawing, PLAIN, RAISED_BOT, Style, document, escaped, lay, run, text};
use crate::nets::{NetError, ProofStructure};
use crate::occurrences::{OccId, Reading, Side};
use crate::sequents::Kind;
use std::fmt::Write;

/// How far below a literal's baseline its premise edge ends.
const FOOT: i64 = 100;

/// The height of the middle of `⅋` above the baseline: unlike `⊗`, it
/// stands on the baseline instead of being centred on the math axis.
const PAR_MIDDLE: i64 = 344;

/// Writes an arrowhead with its tip at `tip`, pointing along `towards`,
/// its wings `size` back from the tip.
fn arrowhead(out: &mut String, tip: (i64, i64), towards: (f64, f64), size: i64) {
    let length = towards.0.hypot(towards.1);
    let (dx, dy) = (towards.0 / length, towards.1 / length);
    let back = (
        tip.0 as f64 - dx * size as f64,
        tip.1 as f64 - dy * size as f64,
    );
    let side = 0.6 * size as f64;
    let wing = |sign: f64| {
        (
            (back.0 - sign * dy * side).round() as i64,
            (back.1 + sign * dx * side).round() as i64,
        )
    };
    let (a, b) = (wing(1.0), wing(-1.0));
    writeln!(
        out,
        r#"<path d="M{} {}L{} {}L{} {}"/>"#,
        a.0, a.1, tip.0, tip.1, b.0, b.1
    )
    .unwrap();
}

/// Writes an arrowhead halfway along the segment from `from` to `to`,
/// pointing at `to`.
fn arrow_along(out: &mut String, from: (i64, i64), to: (i64, i64), size: i64) {
    let towards = ((to.0 - from.0) as f64, (to.1 - from.1) as f64);
    let middle = ((from.0 + to.0) / 2, (from.1 + to.1) / 2);
    let tip = towards_by(middle, towards, size / 2);
    arrowhead(out, tip, towards, size);
}

/// Returns the point `by` from `from` in the direction `towards`.
fn towards_by(from: (i64, i64), towards: (f64, f64), by: i64) -> (i64, i64) {
    let scale = by as f64 / towards.0.hypot(towards.1);
    (
        from.0 + (towards.0 * scale).round() as i64,
        from.1 + (towards.1 * scale).round() as i64,
    )
}

/// Returns the point `by` along the segment from `from` towards `to`.
fn towards(from: (i64, i64), to: (i64, i64), by: i64) -> (i64, i64) {
    let (dx, dy) = ((to.0 - from.0) as f64, (to.1 - from.1) as f64);
    let scale = by as f64 / dx.hypot(dy);
    (
        from.0 + (dx * scale).round() as i64,
        from.1 + (dy * scale).round() as i64,
    )
}

/// Returns the height of the arc of a link `rx` wide on either side of
/// its middle: `rx` times the style's ratio, and beyond the style's cap
/// `W` the ratio times `sqrt(W·rx)`, which meets it at `rx = W`.
///
/// Nested arcs never cross: the squared height of an arc over `(a, b)`
/// at `x` is `φ(rx)·(x−a)(b−x)`, which shrinks with the interval for a
/// constant `φ` (similar arcs) and for `φ ∝ 1/rx` (then it is a multiple
/// of the harmonic mean of `x−a` and `b−x`), and an arc with `rx ≤ W` lies
/// below that harmonic arc of its own interval.
fn height(rx: i64, style: &Style) -> i64 {
    let ratio = i64::from(style.link_height);
    match style.link_cap.map(i64::from) {
        Some(cap) if rx > cap => ratio * cap.saturating_mul(rx).isqrt() / 1000,
        _ => rx * ratio / 1000,
    }
}

/// Returns the links of a structure as pairs of the occurrences they join.
fn joined(net: &ProofStructure) -> impl Iterator<Item = (OccId, OccId)> {
    net.links().iter().map(|&(x, y)| (x.occ(), y.occ()))
}

/// Returns every occurrence's component in the switching of a structure
/// that keeps the left premise of every `⅋`, as the index of one
/// occurrence in it.
fn components(net: &ProofStructure) -> Vec<usize> {
    let forest = net.forest();
    let mut parent: Vec<usize> = (0..forest.len()).collect();
    let find = |parent: &mut Vec<usize>, mut o: usize| {
        while parent[o] != o {
            parent[o] = parent[parent[o]];
            o = parent[o];
        }
        o
    };
    let kept = forest.ids().flat_map(|o| {
        let cut = (forest.kind(o) == Kind::Par)
            .then(|| forest.right(o))
            .flatten();
        forest
            .children(o)
            .filter(move |&c| Some(c) != cut)
            .map(move |c| (o, c))
    });
    for (a, b) in kept.chain(joined(net)) {
        let (a, b) = (find(&mut parent, a.index()), find(&mut parent, b.index()));
        parent[a] = b;
    }
    (0..forest.len()).map(|o| find(&mut parent, o)).collect()
}

/// The bytes of a drawing's start, its groups and its end, beyond what
/// [`estimate`] counts per part.
const DOCUMENT: u64 = 1000;

/// The bytes of one `<text>` element besides its characters: its
/// coordinates and width of up to twelve digits, its size, an id and a
/// fill attribute's name.
const PIECE: u64 = 140;

/// The bytes of the group a formula's text is when it has several pieces,
/// besides its colour.
const GROUP: u64 = 40;

/// The bytes a connective adds to its formula's text: its symbol, the
/// spaces around it and the brackets around the formula.
const BINARY: u64 = 7;

/// The bytes of a connective: its circle, its symbol and its two premise
/// edges.
const CONNECTIVE: u64 = 400;

/// The bytes of the edge below a conclusion.
const ROOT: u64 = 60;

/// The bytes of a link's arc and the two lines down to its literals.
const LINK: u64 = 160;

/// The bytes an essential net's drawing adds per edge, conclusion and
/// link: an arrowhead of three points with coordinates of up to twelve
/// digits; per literal, its colour; and per drawing, its groups.
const ARROW: u64 = 100;

/// A writer that counts the bytes of what it is given escaped for XML,
/// and fails once they pass `most`.
struct Count {
    /// The bytes counted.
    bytes: u64,
    /// The most it counts before it fails.
    most: u64,
}

impl Write for Count {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        for c in s.chars() {
            let escaped = match c {
                '&' => "&amp;".len(),
                '<' | '>' => "&lt;".len(),
                '"' => "&quot;".len(),
                c => c.len_utf8(),
            };
            self.bytes = self.bytes.saturating_add(escaped as u64);
        }
        if self.bytes > self.most {
            return Err(std::fmt::Error);
        }
        Ok(())
    }
}

/// Returns the bytes of a structure's drawing, estimated from the
/// structure without drawing it, and at least what the drawing has; once
/// past `most`, some number past it; `read` when it is drawn as its
/// essential net. Saturating, so that no structure is estimated small.
pub(super) fn estimate(net: &ProofStructure, style: &Style, read: bool, most: u64) -> u64 {
    let forest = net.forest();
    let sequent = forest.sequent();
    let formulas = style.net == NetDrawing::Formulas;
    let colour = match read {
        true => escaped(&style.input).len() as u64 + 8,
        false => 0,
    };
    let mut bytes = DOCUMENT;
    // Per occurrence, the bytes of its formula's text with letters of four
    // bytes, and how many negated literals it has, each a raised `⊥` that
    // splits the text into two more pieces where the reading does not
    // write it as its atom: a formula is written once per subformula
    // drawn, so the drawing grows with the formulas' depth.
    let (mut chars, mut duals) = (vec![0u64; forest.len()], vec![0u64; forest.len()]);
    for o in forest.ids().rev() {
        let i = o.index();
        (chars[i], duals[i]) = match (forest.left(o), forest.right(o), forest.atom(o)) {
            (Some(l), Some(r), _) => (
                chars[l.index()]
                    .saturating_add(chars[r.index()])
                    .saturating_add(BINARY),
                duals[l.index()].saturating_add(duals[r.index()]),
            ),
            (_, _, atom) => {
                let name = atom.map_or(0, |a| sequent.atom_name(a).len() as u64);
                let dual = !read && forest.kind(o) == Kind::DualAtom;
                (name.saturating_mul(4).saturating_add(3), u64::from(dual))
            }
        };
        if formulas || forest.is_literal(o) || forest.parent(o).is_none() {
            let pieces = duals[i].saturating_mul(2).saturating_add(1);
            bytes = bytes
                .saturating_add(chars[i])
                .saturating_add(pieces.saturating_mul(PIECE + colour))
                .saturating_add(GROUP + colour);
        }
        bytes = bytes.saturating_add(match forest.atom(o) {
            // In the title, the name as it is, with a `⊥`.
            Some(atom) => (sequent.atom_name(atom).len() as u64).saturating_add(3),
            // Its circle or joint, symbol and edges, and in the title the
            // symbol with a space on either side and its brackets.
            None => CONNECTIVE + 9,
        });
    }
    let roots = forest.roots().len() as u64;
    let links = net.links().len() as u64;
    bytes = bytes
        .saturating_add(roots.saturating_mul(ROOT + 2))
        .saturating_add(links.saturating_mul(LINK));
    if read {
        // An arrowhead per edge (two per connective), conclusion and link,
        // a colour per literal and the drawing's two more groups.
        let edges = (forest.len() as u64)
            .saturating_mul(2)
            .saturating_add(roots)
            .saturating_add(links);
        bytes = bytes
            .saturating_add(edges.saturating_mul(ARROW))
            .saturating_add(DOCUMENT);
    }
    if style.description && bytes <= most {
        // The text form, as the drawing writes it: its sequent, its links
        // and the verdict, which names the occurrences of a switching
        // cycle or of the parts of a disconnected structure.
        let mut count = Count {
            bytes: 0,
            most: most - bytes,
        };
        let _ = write!(count, "{net}");
        bytes = bytes.saturating_add(count.bytes);
    }
    bytes
}

/// Returns a proof structure as an SVG document, titled with its sequent
/// in plain text; `verdict` is its [`ProofStructure::is_correct`], or with
/// the `reading` of its sequent, which draws it as its essential net, its
/// [`ProofStructure::is_essential`].
pub(super) fn draw(
    net: &ProofStructure,
    style: &Style,
    verdict: &Result<(), NetError>,
    reading: Option<&Reading<'_>>,
) -> String {
    let forest = net.forest();
    let n = forest.len();
    let formulas = style.net == NetDrawing::Formulas;
    let line_height = i64::from(style.line_height);
    let (gap, radius) = (i64::from(style.literal_gap), i64::from(style.node_radius));
    let (margin, stroke) = (i64::from(style.margin), i64::from(style.stroke_width));
    let label_size = i64::from(style.label_size);
    let input = |o: OccId| reading.is_some_and(|r| r.position(o) == Side::Input);
    let arrow = radius / 2;
    let advance = |c: char| i64::from(style.font.advances.advance(c));
    let width = |text: &str| lay(text, 1000, &style.font, |_| {});
    let root = |o: OccId| forest.parent(o).is_none();
    // Whether an occurrence's formula is written: a compact net writes
    // only its literals and its conclusions.
    let written = |o: OccId| formulas || forest.is_literal(o) || root(o);
    // An occurrence's formula as the drawing writes it: one-sided, or as
    // the reading reads it, which writes a literal as its atom.
    let write = |out: &mut String, o: OccId, brackets: bool| {
        match reading {
            Some(r) => NOTATION.ill(out, r, o, brackets),
            None => NOTATION.term(out, forest.sequent(), forest.term(o), brackets),
        }
        .unwrap();
    };
    // The connective at a link's joint or in a compact node's circle, and
    // the height of its middle above its baseline. An essential net's
    // links are the reading's connectives: `A⊥ ⅋ B` in output position and
    // `A ⊗ B⊥` in input position are `⊸`, a division of the Lambek
    // calculus too, whose side its conclusion's text tells (a `\` or `/` at
    // the joint would read as a piece of an edge).
    let symbol = |o: OccId| -> (&'static str, i64) {
        match reading {
            Some(_) if (forest.kind(o) == Kind::Par) != input(o) => (NOTATION.lollipop, AXIS),
            Some(_) => (NOTATION.tensor, AXIS),
            None if forest.kind(o) == Kind::Par => (NOTATION.par, PAR_MIDDLE),
            None => (NOTATION.tensor, AXIS),
        }
    };

    // Bottom-up, the extent of every subtree around its anchor (a literal's
    // atom's middle, not a negated literal's whole `A⊥`; a link's joint; a
    // compact node's centre) and each anchor's offset from its parent's;
    // each connective one layer below the lower of its premises.
    let mut formula = String::new();
    let mut from_left = vec![0; n];
    let mut extent = vec![(0, 0); n];
    let mut offset = vec![0; n];
    let mut layer = vec![0; n];
    for o in forest.ids().rev() {
        let i = o.index();
        formula.clear();
        let text_width = if written(o) {
            write(&mut formula, o, false);
            width(&formula)
        } else {
            0
        };
        let (Some(l), Some(r)) = (forest.left(o), forest.right(o)) else {
            let atom = match (forest.kind(o), reading) {
                (Kind::DualAtom, None) => width(formula.trim_end_matches(RAISED_BOT)),
                _ => text_width,
            };
            from_left[i] = atom / 2;
            extent[i] = (-(atom / 2), text_width - atom / 2);
            continue;
        };
        layer[i] = 1 + layer[l.index()].max(layer[r.index()]);
        let (left, right) = (extent[l.index()], extent[r.index()]);
        let own = if formulas {
            // The text stands with its connective under the joint, and each
            // premise's subtree as far from the joint as the premise stands
            // in the text: so every subformula of a one-sided net is drawn
            // where it is in its conclusion, and the subtrees of the two
            // premises stand on either side of the joint.
            let first = reading.and_then(|r| r.operands(o).0).unwrap_or(l);
            formula.clear();
            write(&mut formula, first, true);
            formula.push(' ');
            let half = symbol(o).0.chars().map(advance).sum::<i64>() / 2;
            from_left[i] = width(&formula) + half;
            let bracket = |child: OccId, c: char| match forest.is_literal(child) {
                true => 0,
                false => advance(c),
            };
            let space = advance(' ');
            offset[l.index()] = -(half + space + bracket(l, ')') + left.1);
            offset[r.index()] = half + space + bracket(r, '(') - right.0;
            (-from_left[i], text_width - from_left[i])
        } else {
            // The subtrees `literal_gap` apart and the node halfway between
            // their anchors; a conclusion's text centred under its circle.
            let span = left.1 + gap - right.0;
            offset[l.index()] = -(span / 2);
            offset[r.index()] = span - span / 2;
            from_left[i] = text_width / 2;
            (
                (-radius).min(-from_left[i]),
                radius.max(text_width - from_left[i]),
            )
        };
        extent[i] = (
            own.0.min(offset[l.index()] + left.0),
            own.1.max(offset[r.index()] + right.1),
        );
    }
    // The conclusions side by side on the bottom layer, `literal_gap`
    // apart, and top-down every anchor from its parent's.
    let bottom = forest.roots().iter().map(|r| layer[r.index()]).max();
    let bottom = bottom.unwrap_or(0);
    let mut x = vec![0; n];
    let mut right = margin - gap;
    for &r in forest.roots() {
        layer[r.index()] = bottom;
        x[r.index()] = right + gap - extent[r.index()].0;
        right = x[r.index()] + extent[r.index()].1;
    }
    for o in forest.ids() {
        if let Some(p) = forest.parent(o) {
            x[o.index()] = x[p.index()] + offset[o.index()];
        }
    }

    // The links' arcs above the literals decide where the literals'
    // baseline is.
    let arc = |(p, q): (OccId, OccId)| {
        let (a, b) = (
            x[p.index()].min(x[q.index()]),
            x[p.index()].max(x[q.index()]),
        );
        let rx = (b - a) / 2;
        (a, b, rx, height(rx, style))
    };
    let highest = joined(net).map(|l| arc(l).3).max();
    let baseline = margin + stroke + highest.unwrap_or(0) + HEIGHT;
    let top = baseline - HEIGHT;
    // Drawn with its formulas, every layer is a line of text, the next
    // `line_height` and a joint's room further down; compact, the nodes'
    // layers are `line_height` apart, the conclusions' edges end half a
    // layer below the lowest, and the conclusions are written under them.
    let pitch = line_height + radius + HEIGHT + DEPTH;
    let end = baseline - AXIS + bottom * line_height + line_height / 2;
    let under = end + HEIGHT;
    let text_y = |o: OccId| match (formulas, root(o)) {
        (true, _) => baseline + layer[o.index()] * pitch,
        (false, true) => under,
        (false, false) => baseline,
    };
    // A link's joint, half its room clear above its conclusion's text, or
    // a compact node's centre.
    let node = |o: OccId| match formulas {
        true => (x[o.index()], text_y(o) - HEIGHT - radius - radius / 2),
        false => (
            x[o.index()],
            baseline - AXIS + layer[o.index()] * line_height,
        ),
    };
    // Where a premise edge ends under a written formula, clear of its
    // descenders.
    let depth = style.font.advances.depth();
    let foot = |o: OccId| match formulas {
        true => (x[o.index()], text_y(o) + depth),
        false => (x[o.index()], text_y(o) + FOOT),
    };

    // A switching cycle, as its edges, or the parts of a disconnected
    // structure: every occurrence's component in the switching that keeps
    // each left premise, and the first part's, which stays unmarked.
    let (mut cycle, mut parts) = (Vec::new(), None);
    match verdict {
        Err(
            NetError::SwitchingCycle { cycle: vertices }
            | NetError::DirectedCycle { cycle: vertices },
        ) => {
            let next = vertices.iter().cycle().skip(1);
            cycle = vertices
                .iter()
                .zip(next)
                .map(|(&a, &b)| (a.min(b).occ(), a.max(b).occ()))
                .collect();
            cycle.sort_unstable();
        }
        Err(NetError::Disconnected { parts: tops }) => {
            let component = components(net);
            let first = component[tops[0][0].index()];
            parts = Some((component, first));
        }
        // The path, and the missing edge from the implication to its
        // antecedent that it goes around.
        Err(NetError::NoDominator { implication, path }) => {
            cycle = path
                .windows(2)
                .map(|w| (w[0].min(w[1]).occ(), w[0].max(w[1]).occ()))
                .collect();
            if let Some(antecedent) = forest.left(implication.occ()) {
                cycle.push((implication.occ(), antecedent));
            }
            cycle.sort_unstable();
        }
        _ => {}
    }
    let apart = |o: OccId| {
        parts
            .as_ref()
            .is_some_and(|(c, first)| c[o.index()] != *first)
    };
    let marks = |a: OccId, b: OccId| {
        let joined = parts
            .as_ref()
            .is_some_and(|(c, _)| c[a.index()] == c[b.index()]);
        cycle.binary_search(&(a.min(b), a.max(b))).is_ok() || (joined && apart(a))
    };

    let (mut lines, mut pars, mut links) = (String::new(), String::new(), String::new());
    let (mut marked, mut texts) = (String::new(), String::new());
    let (mut inputs, mut dominators) = (String::new(), String::new());
    let dash = format!("{} {}", 3 * stroke, 3 * stroke);
    let fill = |o: OccId| match input(o) {
        true => format!(r#" fill="{}""#, escaped(&style.input)),
        false => String::new(),
    };
    for o in forest.ids() {
        if written(o) {
            // A compact net's circle is its connective's element.
            let mut attributes = match formulas || forest.is_literal(o) {
                true => format!(r#" id="o{}""#, o.get()),
                false => String::new(),
            };
            attributes.push_str(&fill(o));
            formula.clear();
            write(&mut formula, o, false);
            let line = run(&formula, 1000, &style.font);
            text(
                &mut texts,
                x[o.index()] - from_left[o.index()],
                text_y(o),
                &line,
                &attributes,
                None,
            );
        }
        if forest.is_literal(o) {
            continue;
        }
        let centre = node(o);
        let (sign, middle) = symbol(o);
        let label = run(sign, label_size, &style.font);
        let mut size = format!(r#" font-size="{label_size}""#);
        if formulas {
            size.push_str(&fill(o));
        } else {
            writeln!(
                if input(o) { &mut inputs } else { &mut lines },
                r#"<circle id="o{}" cx="{}" cy="{}" r="{radius}"/>"#,
                o.get(),
                centre.0,
                centre.1
            )
            .unwrap();
        }
        text(
            &mut texts,
            centre.0 - label.width / 2,
            centre.1 + middle * label_size / 1000,
            &label,
            &size,
            None,
        );
        for child in forest.children(o) {
            let to = match formulas || forest.is_literal(child) {
                true => foot(child),
                false => towards(node(child), centre, radius),
            };
            let from = towards(centre, to, radius);
            if reading.is_some() {
                // Output premises lead down into their connective, input
                // connectives up into their input premises, and nothing
                // into the antecedent of an output implication.
                let (upward, antecedent) = (input(child), input(child) && !input(o));
                let out = match (marks(o, child), antecedent, upward) {
                    (true, _, _) => &mut marked,
                    (false, true, _) => &mut dominators,
                    (false, false, true) => &mut inputs,
                    (false, false, false) => &mut lines,
                };
                write!(out, r#"<path d="M{} {}L{} {}""#, from.0, from.1, to.0, to.1).unwrap();
                if antecedent {
                    write!(out, r#" stroke-dasharray="{stroke} {}""#, 2 * stroke).unwrap();
                    out.push_str("/>\n");
                } else {
                    out.push_str("/>\n");
                    let (start, end) = if upward { (from, to) } else { (to, from) };
                    arrow_along(out, start, end, arrow);
                }
                continue;
            }
            let par = forest.kind(o) == Kind::Par;
            let out = match (marks(o, child), par) {
                (true, _) => &mut marked,
                (false, true) => &mut pars,
                (false, false) => &mut lines,
            };
            write!(out, r#"<path d="M{} {}L{} {}""#, from.0, from.1, to.0, to.1).unwrap();
            if par && marks(o, child) {
                write!(out, r#" stroke-dasharray="{dash}""#).unwrap();
            }
            out.push_str("/>\n");
        }
        // A compact conclusion's edge hangs from its circle to its text;
        // the hypotheses come in from below, the goal goes out.
        if root(o) && !formulas {
            let (column, start) = (x[o.index()], centre.1 + radius);
            let out = match (apart(o), input(o)) {
                (true, _) => &mut marked,
                (false, true) => &mut inputs,
                (false, false) => &mut lines,
            };
            writeln!(out, r#"<path d="M{column} {start}V{end}"/>"#).unwrap();
            if reading.is_some() {
                let (from, to) = ((column, start), (column, end));
                let (from, to) = if input(o) { (to, from) } else { (from, to) };
                arrow_along(out, from, to, arrow);
            }
        }
    }
    for (p, q) in joined(net) {
        let (a, b, rx, ry) = arc((p, q));
        let out = if marks(p, q) { &mut marked } else { &mut links };
        // A literal that is a conclusion stands at the bottom, and its
        // link's arc reaches down to it.
        let (left, right) = if x[p.index()] < x[q.index()] {
            (p, q)
        } else {
            (q, p)
        };
        let (down_left, down_right) = (text_y(left) - HEIGHT, text_y(right) - HEIGHT);
        write!(
            out,
            r#"<path id="l{}-{}" d="M{a} {down_left}"#,
            p.min(q).get(),
            p.max(q).get()
        )
        .unwrap();
        if down_left != top {
            write!(out, "V{top}").unwrap();
        }
        write!(out, "A{rx} {ry} 0 0 1 {b} {top}").unwrap();
        if down_right != top {
            write!(out, "V{down_right}").unwrap();
        }
        out.push_str("\"/>\n");
        // From the input literal to the output one, at the arc's top.
        if reading.is_some() {
            let leftwards = left == if input(p) { q } else { p };
            let towards = (if leftwards { -1.0 } else { 1.0 }, 0.0);
            let tip = towards_by((a + rx, top - ry), towards, arrow / 2);
            arrowhead(out, tip, towards, arrow);
        }
    }

    let group = |out: &mut String, colour: &str, extra: &str, content: &str| {
        writeln!(
            out,
            "<g fill=\"none\" stroke=\"{}\" stroke-width=\"{stroke}\"{extra}>",
            escaped(colour)
        )
        .unwrap();
        out.push_str(content);
        out.push_str("</g>\n");
    };
    let body = |out: &mut String| {
        group(out, &style.line, "", &lines);
        if reading.is_some() {
            group(out, &style.input, "", &inputs);
            group(out, &style.dominator, "", &dominators);
        } else {
            group(
                out,
                &style.par,
                &format!(r#" stroke-dasharray="{dash}""#),
                &pars,
            );
        }
        group(out, &style.link, "", &links);
        if !marked.is_empty() {
            group(out, &style.highlight, "", &marked);
        }
        out.push_str(&texts);
    };
    let mut title = String::new();
    PLAIN.one_sided(&mut title, forest.sequent()).unwrap();
    let width = if forest.roots().is_empty() {
        2 * margin
    } else {
        right + margin
    };
    let height = match formulas {
        true => baseline + bottom * pitch + DEPTH + margin,
        false => under + DEPTH + margin,
    };
    let description = style.description.then(|| net.to_string());
    document(style, &title, description.as_deref(), (width, height), body)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The arc of a wide link stays under the arc of a wider link it is
    /// nested in, past the cap, where a plain cap on the height would
    /// make them cross.
    #[test]
    fn nested_arcs_do_not_cross() {
        let style = Style::default();
        // The height of the arc over `(a, b)` at `x`.
        let at = |(a, b): (i64, i64), x: i64| {
            let (rx, middle) = ((b - a) as f64 / 2.0, (a + b) as f64 / 2.0);
            let ry = height((b - a) / 2, &style) as f64;
            ry * (1.0 - ((x as f64 - middle) / rx).powi(2)).max(0.0).sqrt()
        };
        let (outer, inner) = ((0, 200_000), (2_000, 30_000));
        assert!(height(15_000, &style) < 15_000 * 600 / 1000);
        for x in [inner.0, inner.0 + 500, (inner.0 + inner.1) / 2, inner.1] {
            assert!(at(inner, x) < at(outer, x), "{x}");
        }
    }
}
