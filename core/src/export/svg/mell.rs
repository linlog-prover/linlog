// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Proof structures with boxes drawn as the multiplicative ones are, with
//! every `!`-box a rectangle around its contents: the literals side by
//! side along the top under the axiom links' arcs, every vertex one row
//! below the lowest of what it depends on, the conclusions at the bottom.
//!
//! A box is laid out as one piece above its `!` vertex: the premise of the
//! `!` where it stands in the `!` formula's text, and beside it the copies
//! that lie in the box but belong to a `?` vertex outside it, those of a
//! `?` that comes earlier in the formula trees to its left, the others to
//! its right. Its rectangle encloses them and every box nested in them,
//! with its bottom border through the `!` vertex's joint (its principal
//! door) and through an auxiliary door for every `?` vertex outside with a
//! copy inside, where the wires from those copies meet and leave the box,
//! straight to the door of the box around it or to the `?` vertex itself.
//! A `?` vertex stands at least a row below every door its wires leave,
//! a link over a box rises above the box's top, and copies at their `?`
//! vertex's own depth stand on the row right above it. Wires and the
//! premise edges of a `?` vertex are switched as a `⅋`'s are and drawn
//! like them. A weakening (a `?` vertex without copies) stands alone; the
//! literal it hangs on is in the description. A door's wire may cross
//! other edges and formulas, where the trees do not let it go around.
//!
//! A vertex's formula, or a compact vertex's circle, has the id `o<n>`
//! when it is the only vertex of occurrence `n`, and `v<k>` (its vertex
//! number) when its occurrence has several; a link has `l<x>-<y>`, the
//! vertices it joins, and a box's rectangle `b<k>`, its `!` vertex.

use super::font::{AXIS, DEPTH, HEIGHT};
use super::net::{
    BINARY, CONNECTIVE, Count, DOCUMENT, GROUP, LINK, PAR_MIDDLE, PIECE, ROOT, height, towards,
};
use super::{
    Conclusions, NOTATION, NetDrawing, PLAIN, Premises, RAISED_BOT, Style, WhyNot, document,
    escaped, lay, run, text,
};
use crate::hash::HashMap;
use crate::nets::{MellStructure, NetError, VertexId};
use crate::occurrences::OccId;
use crate::sequents::Kind;
use std::fmt::Write;

/// No vertex: the box outside every box, or a root's parent.
const NONE: u32 = u32::MAX;

/// The bytes of a box's rectangle besides its doors' gaps.
const BOX: u64 = 120;

/// The bytes of a door: its symbol or circle, its gap in the border and
/// the wire that leaves it.
const DOOR: u64 = 400;

/// The bytes of a wire from a copy into a door.
const WIRE: u64 = 80;

/// The bytes of a node of the rules: its label, its circle and the edge
/// into it.
const RULE: u64 = 200;

/// The auxiliary doors of a structure: every pair of a box and a `?`
/// vertex outside it with a copy inside, or under [`WhyNot::Rules`] of a
/// box and such a copy, sorted, each with what feeds it and where its
/// wire goes.
struct Doors {
    /// The pairs (`!` vertex, `?` vertex or copy), sorted.
    pairs: Vec<(u32, u32)>,
}

impl Doors {
    /// Returns the doors of a structure: for each copy outside its `?`
    /// vertex's box, a door on every box from the copy's out to the
    /// `?`'s, a box's chain walked only until a door already found; one
    /// per copy, not per `?` vertex, when `apart`.
    fn of(net: &MellStructure, apart: bool) -> Self {
        let mut found: crate::hash::HashSet<(u32, u32)> = crate::hash::HashSet::default();
        for c in net.vertices() {
            let Some(q) = net.parent(c).filter(|&q| kind(net, q) == Kind::Quest) else {
                continue;
            };
            let home = raw(net.inside(q));
            let key = if apart { c.get() } else { q.get() };
            let mut b = raw(net.inside(c));
            while b != home && found.insert((b, key)) {
                b = raw(net.inside(VertexId::new(b)));
            }
        }
        let mut pairs: Vec<(u32, u32)> = found.into_iter().collect();
        pairs.sort_unstable();
        Self { pairs }
    }

    /// Returns the index of the door of box `b` for `key`, a `?` vertex
    /// or a copy.
    fn index(&self, b: u32, key: u32) -> usize {
        self.pairs
            .binary_search(&(b, key))
            .expect("a door on the chain of a copy")
    }
}

/// Where a wire ends under [`WhyNot::Rules`]: at a node of the rules or
/// at a formula.
#[derive(Clone, Copy)]
struct End {
    /// The node's centre, or the point of the formula it goes into.
    at: (i64, i64),
    /// How far from `at` it stops: the node's radius, or none.
    into: i64,
    /// The height of its leg when it comes down into `at` from above, or
    /// none when it comes in from the side.
    above: Option<i64>,
}

/// Returns the kind of a vertex's occurrence.
fn kind(net: &MellStructure, v: VertexId) -> Kind {
    net.forest().kind(net.occurrence(v).expect("a vertex"))
}

/// Returns a vertex as its raw index, `NONE` for none.
fn raw(v: Option<VertexId>) -> u32 {
    v.map_or(NONE, VertexId::get)
}

/// Returns the bytes of a structure's drawing, estimated without drawing
/// it, and at least what the drawing has; once past `most`, some number
/// past it. Saturating, so that no structure is estimated small.
pub(super) fn estimate(net: &MellStructure, style: &Style, most: u64) -> u64 {
    let forest = net.forest();
    let sequent = forest.sequent();
    let formulas = style.net == NetDrawing::Formulas;
    let rules = style.net_look.why_not == WhyNot::Rules;
    let bottom = style.net_look.conclusions == Conclusions::Bottom;
    let mut bytes = DOCUMENT;
    // Per occurrence, the bytes of its formula's text with letters of four
    // bytes and its negated literals, each a raised `⊥` of two more
    // pieces; and in the title its symbols or its name.
    let (mut chars, mut duals) = (vec![0u64; forest.len()], vec![0u64; forest.len()]);
    for o in forest.ids().rev() {
        let i = o.index();
        let below: Vec<OccId> = forest.children(o).collect();
        (chars[i], duals[i]) = match (below.as_slice(), forest.atom(o)) {
            ([], atom) => {
                let name = atom.map_or(0, |a| sequent.atom_name(a).len() as u64);
                let dual = u64::from(forest.kind(o) == Kind::DualAtom);
                (name.saturating_mul(4).saturating_add(3), dual)
            }
            (children, _) => children.iter().fold((BINARY, 0u64), |(c, d), &k| {
                (
                    c.saturating_add(chars[k.index()]),
                    d.saturating_add(duals[k.index()]),
                )
            }),
        };
        bytes = bytes.saturating_add(match forest.atom(o) {
            Some(atom) => (sequent.atom_name(atom).len() as u64).saturating_add(3),
            None => 9,
        });
    }
    for v in net.vertices() {
        let o = net.occurrence(v).expect("a vertex");
        let root = net.parent(v).is_none();
        if formulas || forest.is_literal(o) || root {
            let pieces = duals[o.index()].saturating_mul(2).saturating_add(1);
            bytes = bytes
                .saturating_add(chars[o.index()])
                .saturating_add(pieces.saturating_mul(PIECE))
                .saturating_add(GROUP);
        }
        // Its symbol and circle or joint and the edges into it, its wire
        // into a door, and a box's rectangle.
        bytes = bytes.saturating_add(CONNECTIVE + WIRE);
        if kind(net, v) == Kind::Bang {
            bytes = bytes.saturating_add(BOX);
        }
        if root {
            bytes = bytes.saturating_add(ROOT);
            // And the edge down from a raised `!` to the bottom row.
            if bottom && kind(net, v) == Kind::Bang {
                bytes = bytes.saturating_add(ROOT);
            }
        }
        // Under the rules, a copy's dereliction and the contraction it
        // goes into, and a `?` vertex's weakening.
        if rules {
            let copy = net.parent(v).is_some_and(|q| kind(net, q) == Kind::Quest);
            if copy {
                bytes = bytes.saturating_add(2 * RULE);
            }
            if kind(net, v) == Kind::Quest {
                bytes = bytes.saturating_add(RULE);
            }
        }
    }
    let doors = Doors::of(net, rules).pairs.len() as u64;
    let links = net.links().len() as u64;
    bytes = bytes
        .saturating_add(doors.saturating_mul(DOOR))
        .saturating_add(links.saturating_mul(LINK));
    if style.description && bytes <= most {
        let mut count = Count::new(most - bytes);
        let _ = write!(count, "{net}");
        bytes = bytes.saturating_add(count.bytes());
    }
    bytes
}

/// Returns a proof structure with boxes as an SVG document, titled with
/// its sequent in plain text; `verdict` is its
/// [`MellStructure::is_correct`], whose switching cycle is drawn in the
/// highlight colour.
#[expect(
    clippy::too_many_lines,
    reason = "one layout in its passes, as the multiplicative drawing is"
)]
pub(super) fn draw(net: &MellStructure, style: &Style, verdict: &Result<(), NetError>) -> String {
    let forest = net.forest();
    let n = net.len();
    let formulas = style.net == NetDrawing::Formulas;
    let dashed = style.net_look.premises == Premises::Dashed;
    let rules = style.net_look.why_not == WhyNot::Rules;
    let line_height = i64::from(style.line_height);
    let (gap, radius) = (i64::from(style.literal_gap), i64::from(style.node_radius));
    let (margin, stroke) = (i64::from(style.margin), i64::from(style.stroke_width));
    let label_size = i64::from(style.label_size);
    let advance = |c: char| i64::from(style.font.advances.advance(c));
    let width = |text: &str| lay(text, 1000, &style.font, |_| {});
    let occ = |v: u32| net.occurrence(VertexId::new(v)).expect("a vertex");
    let kind_of = |v: u32| forest.kind(occ(v));
    let parent = |v: u32| raw(net.parent(VertexId::new(v)));
    let inside = |v: u32| raw(net.inside(VertexId::new(v)));
    let root = |v: u32| parent(v) == NONE;
    let written = |v: u32| formulas || forest.is_literal(occ(v)) || root(v);
    let binary = |v: u32| matches!(kind_of(v), Kind::Tensor | Kind::Par);
    let write = |out: &mut String, v: u32, brackets: bool| {
        NOTATION
            .term(out, forest.sequent(), forest.term(occ(v)), brackets)
            .unwrap();
    };
    // Whether a copy lies in another box than its `?` vertex.
    let boxed = |c: u32| {
        let q = parent(c);
        q != NONE && kind_of(q) == Kind::Quest && inside(c) != inside(q)
    };
    // How many vertices each occurrence has, for the ids.
    let mut instances = vec![0u32; forest.len()];
    for v in 0..n as u32 {
        instances[occ(v).index()] += 1;
    }
    let id = |v: u32| match instances[occ(v).index()] {
        1 => format!("o{}", occ(v).get()),
        _ => format!("v{v}"),
    };
    // What a boxed copy's doors are of, its `?` vertex or under the rules
    // the copy itself, and the `?` vertex of a door's key.
    let key_of = |c: u32| if rules { c } else { parent(c) };
    let owner = |key: u32| if rules { parent(key) } else { key };

    // The layout tree: a vertex's premises, a `!` vertex's box (node
    // `n + b`), a `?` vertex's copies at its own depth, and a box's
    // premise and its boxed copies, ordered by where their `?` vertex
    // comes in the formula trees against the `!` vertex.
    let mut pre = vec![0u32; n];
    let mut walk: Vec<u32> = (0..forest.roots().len() as u32).rev().collect();
    let mut time = 0;
    while let Some(v) = walk.pop() {
        pre[v as usize] = time;
        time += 1;
        let premises: Vec<u32> = net.premises(VertexId::new(v)).map(VertexId::get).collect();
        walk.extend(premises.into_iter().rev());
    }
    let nodes = 2 * n;
    let mut edges: Vec<(u32, u32, u32)> = Vec::with_capacity(nodes);
    for v in 0..n as u32 {
        if kind_of(v) == Kind::Bang {
            edges.push((v, 0, n as u32 + v));
            let premise = net.premises(VertexId::new(v)).next().expect("a premise");
            edges.push((n as u32 + v, pre[v as usize], premise.get()));
            continue;
        }
        for (k, c) in net.premises(VertexId::new(v)).enumerate() {
            if boxed(c.get()) {
                let unit = n as u32 + inside(c.get());
                edges.push((unit, pre[v as usize], c.get()));
            } else {
                edges.push((v, k as u32, c.get()));
            }
        }
    }
    edges.sort_unstable();
    let mut start = vec![0usize; nodes + 1];
    for &(node, _, _) in &edges {
        start[node as usize + 1] += 1;
    }
    for i in 0..nodes {
        start[i + 1] += start[i];
    }
    let kids = |node: usize| edges[start[node]..start[node + 1]].iter().map(|e| e.2);
    let unit = |b: u32| n + b as usize;

    // The doors, and per `?` vertex the boxes whose doors its wires leave
    // last, which it must stand below.
    let doors = Doors::of(net, rules);
    let target = |d: usize| {
        let (b, key) = doors.pairs[d];
        let outer = inside(b);
        (outer != inside(owner(key))).then(|| doors.index(outer, key))
    };
    let mut last: HashMap<u32, Vec<u32>> = HashMap::default();
    for (d, &(b, key)) in doors.pairs.iter().enumerate() {
        if target(d).is_none() {
            last.entry(owner(key)).or_default().push(b);
        }
    }
    // Under the rules, per `?` vertex its premises (a copy at its depth or
    // a wire), its nodes' rows (one for the derelictions of the copies at
    // its depth, or with wires alone one the first wire comes down
    // through, a contraction's for every premise past the first, or a
    // weakening's), and the rows they take above it: all of them where
    // its formula is written below the last, all but the last where that
    // is its circle.
    let (mut inputs, mut levels, mut extra) = (vec![0u32; n], vec![0u32; n], vec![0u32; n]);
    if rules {
        for c in 0..n as u32 {
            if parent(c) != NONE && kind_of(parent(c)) == Kind::Quest {
                inputs[parent(c) as usize] += 1;
            }
        }
        for q in (0..n as u32).filter(|&q| kind_of(q) == Kind::Quest) {
            let (k, own) = (inputs[q as usize], kids(q as usize).next().is_some());
            levels[q as usize] = u32::from(own || k >= 2) + k.saturating_sub(1) + u32::from(k == 0);
            extra[q as usize] = match formulas {
                true => levels[q as usize],
                false => levels[q as usize].saturating_sub(1),
            };
        }
    }
    // A `?` vertex whose one premise is a wire, which is its conclusion.
    let bare = |q: u32| rules && kind_of(q) == Kind::Quest && levels[q as usize] == 0;

    // Rows: every vertex one below the lowest of what it depends on (its
    // layout children, a box's contents, the boxes its wires leave), in a
    // walk that finishes what a vertex depends on first.
    let depends = |v: u32| -> Vec<u32> {
        let mut on: Vec<u32> = if kind_of(v) == Kind::Bang {
            kids(unit(v)).collect()
        } else {
            kids(v as usize).collect()
        };
        if let Some(boxes) = last.get(&v) {
            on.extend(boxes);
        }
        on
    };
    let mut layer = vec![u32::MAX; n];
    let mut stack: Vec<(u32, bool)> = (0..n as u32).rev().map(|v| (v, false)).collect();
    while let Some((v, ready)) = stack.pop() {
        if layer[v as usize] != u32::MAX {
            continue;
        }
        let on = depends(v);
        if ready || on.iter().all(|&w| layer[w as usize] != u32::MAX) {
            // Under the rules, a copy in a box has its dereliction on the
            // row below it, inside the box.
            let below = |w: u32| u32::from(rules && kind_of(v) == Kind::Bang && boxed(w));
            let lowest = on.iter().map(|&w| layer[w as usize] + below(w)).max();
            layer[v as usize] = match (lowest, forest.is_literal(occ(v))) {
                (_, true) => 0,
                (Some(l), false) => l + 1 + extra[v as usize],
                (None, false) => 1 + extra[v as usize],
            };
        } else {
            stack.push((v, true));
            stack.extend(on.into_iter().map(|w| (w, false)));
        }
    }
    // The conclusions on the bottom row, and the copies of a `?` vertex with
    // several at its own depth on the row above it, so that their edges
    // into it pass below every formula and box of the others (under the
    // rules every copy at its depth, above its nodes' rows); a `!` vertex
    // with auxiliary doors stays where it is, above the `?` vertices its
    // wires go to.
    let bottom = (0..forest.roots().len())
        .map(|r| layer[r])
        .max()
        .unwrap_or(0);
    let has_doors: Vec<bool> = {
        let mut has = vec![false; n];
        for &(b, _) in &doors.pairs {
            has[b as usize] = true;
        }
        has
    };
    let mut row = layer.clone();
    let mut order: Vec<u32> = (0..n as u32).collect();
    order.sort_unstable_by_key(|&v| pre[v as usize]);
    for &v in &order {
        let fixed = kind_of(v) == Kind::Bang && has_doors[v as usize];
        let p = parent(v);
        if fixed {
            continue;
        }
        if p == NONE {
            row[v as usize] = bottom;
        } else if kind_of(p) == Kind::Quest
            && !boxed(v)
            && (rules || kids(p as usize).nth(1).is_some())
        {
            row[v as usize] = row[v as usize].max(row[p as usize] - 1 - extra[p as usize]);
        }
    }

    // Bottom-up over the layout tree, the extent of every subtree around
    // its anchor and each child's offset from its parent's anchor; a box
    // (node `n + b`) is anchored at its `!`'s premise.
    let mut formula = String::new();
    let mut from_left = vec![0i64; n];
    let mut extent = vec![(0i64, 0i64); nodes];
    let mut offset = vec![0i64; nodes];
    let mut post: Vec<usize> = Vec::with_capacity(nodes);
    let mut walk: Vec<(usize, bool)> = (0..forest.roots().len())
        .rev()
        .map(|r| (r, false))
        .collect();
    while let Some((node, done)) = walk.pop() {
        if done {
            post.push(node);
            continue;
        }
        walk.push((node, true));
        walk.extend(
            kids(node)
                .map(|k| (k as usize, false))
                .collect::<Vec<_>>()
                .into_iter()
                .rev(),
        );
    }
    // Lays children side by side `gap` apart, the `fixed`-th at offset
    // `at`, and returns the extent they cover.
    let side_by_side =
        |children: &[u32], fixed: usize, at: i64, offset: &mut [i64], extent: &[(i64, i64)]| {
            let e = |c: u32| extent[c as usize];
            offset[children[fixed] as usize] = at;
            let mut cursor = at + e(children[fixed]).0;
            for &c in children[..fixed].iter().rev() {
                offset[c as usize] = cursor - gap - e(c).1;
                cursor = offset[c as usize] + e(c).0;
            }
            let left = cursor;
            let mut cursor = at + e(children[fixed]).1;
            for &c in &children[fixed + 1..] {
                offset[c as usize] = cursor + gap - e(c).0;
                cursor = offset[c as usize] + e(c).1;
            }
            (left, cursor)
        };
    for &node in &post {
        let children: Vec<u32> = kids(node).collect();
        if node >= n {
            // A box: its premise at its anchor, the copies beside it.
            let b = (node - n) as u32;
            let premise = net
                .premises(VertexId::new(b))
                .next()
                .expect("a premise")
                .get();
            let fixed = children
                .iter()
                .position(|&c| c == premise)
                .expect("the premise");
            let covered = side_by_side(&children, fixed, 0, &mut offset, &extent);
            extent[node] = (covered.0 - radius, covered.1 + radius);
            continue;
        }
        let v = node as u32;
        formula.clear();
        let text_width = if written(v) {
            write(&mut formula, v, false);
            width(&formula)
        } else {
            0
        };
        if forest.is_literal(occ(v)) {
            let atom = if kind_of(v) == Kind::DualAtom {
                width(formula.trim_end_matches(RAISED_BOT))
            } else {
                text_width
            };
            from_left[node] = atom / 2;
            extent[node] = (-(atom / 2), text_width - atom / 2);
            continue;
        }
        let bracket = |c: u32, side: char| if binary(c) { advance(side) } else { 0 };
        let own = match kind_of(v) {
            Kind::Tensor | Kind::Par => {
                let (l, r) = (children[0] as usize, children[1] as usize);
                let (left, right) = (extent[l], extent[r]);
                if formulas {
                    formula.clear();
                    write(&mut formula, l as u32, true);
                    formula.push(' ');
                    let symbol = if kind_of(v) == Kind::Par {
                        NOTATION.par
                    } else {
                        NOTATION.tensor
                    };
                    let half = symbol.chars().map(advance).sum::<i64>() / 2;
                    from_left[node] = width(&formula) + half;
                    let space = advance(' ');
                    offset[l] = -(half + space + bracket(l as u32, ')') + left.1);
                    offset[r] = half + space + bracket(r as u32, '(') - right.0;
                    (-from_left[node], text_width - from_left[node])
                } else {
                    // The circle in the gap between the two subtrees, as a
                    // joint stands between them: halfway between the
                    // anchors, a box of one could reach over it, and the
                    // edge to the other premise would rise through the box.
                    offset[l] = -(gap / 2) - left.1;
                    offset[r] = gap - gap / 2 - right.0;
                    from_left[node] = text_width / 2;
                    (
                        (-radius).min(-from_left[node]),
                        radius.max(text_width - from_left[node]),
                    )
                }
            }
            Kind::Bang | Kind::Quest => {
                let symbol = if kind_of(v) == Kind::Bang { '!' } else { '?' };
                // The subformula as it stands after the symbol: the premise,
                // or the first copy.
                let first = if kind_of(v) == Kind::Bang {
                    net.premises(VertexId::new(v)).next().map(VertexId::get)
                } else {
                    children.first().copied()
                };
                if formulas {
                    from_left[node] = advance(symbol) / 2;
                    if let Some(f) = first {
                        let at = advance(symbol) + bracket(f, '(') + from_left[f as usize]
                            - from_left[node];
                        if kind_of(v) == Kind::Bang {
                            offset[unit(v)] = at;
                        } else {
                            side_by_side(&children, 0, at, &mut offset, &extent);
                        }
                    }
                } else {
                    from_left[node] = text_width / 2;
                    if !children.is_empty() {
                        side_by_side(&children, 0, 0, &mut offset, &extent);
                        // The circle halfway between the first and the last
                        // anchor.
                        let middle = offset[*children.last().unwrap() as usize] / 2;
                        for &c in &children {
                            offset[c as usize] -= middle;
                        }
                    }
                }
                if kind_of(v) == Kind::Bang {
                    // The box reaches out to its `!` door.
                    let u = unit(v);
                    let joint = -offset[u];
                    extent[u] = (
                        extent[u].0.min(joint - 2 * radius),
                        extent[u].1.max(joint + 2 * radius),
                    );
                }
                (
                    (-radius).min(-from_left[node]),
                    radius.max(text_width - from_left[node]),
                )
            }
            Kind::Atom
            | Kind::DualAtom
            | Kind::One
            | Kind::Bot
            | Kind::Top
            | Kind::Zero
            | Kind::With
            | Kind::Plus => {
                unreachable!("a literal is handled above and a MELL net has no units or additives")
            }
        };
        extent[node] = children.iter().fold(own, |(lo, hi), &c| {
            (
                lo.min(offset[c as usize] + extent[c as usize].0),
                hi.max(offset[c as usize] + extent[c as usize].1),
            )
        });
    }

    // The conclusions side by side, `literal_gap` apart, and top-down every
    // anchor from its parent's.
    let mut x = vec![0i64; nodes];
    let mut right = margin - gap;
    for r in 0..forest.roots().len() {
        x[r] = right + gap - extent[r].0;
        right = x[r] + extent[r].1;
    }
    for &node in post.iter().rev() {
        for c in kids(node) {
            x[c as usize] = x[node] + offset[c as usize];
        }
    }
    // A box's rectangle, left and right.
    let sides = |b: u32| {
        let u = unit(b);
        (x[u] + extent[u].0, x[u] + extent[u].1)
    };

    // The links and the boxes of every depth: a box is as high above the
    // literals as what is in it, plus a margin, and a link over a box rises
    // above it, so that no arc enters a box it does not belong to. Boxes
    // innermost first.
    let arc = |(p, q): (u32, u32)| {
        let (a, b) = (
            x[p as usize].min(x[q as usize]),
            x[p as usize].max(x[q as usize]),
        );
        let rx = (b - a) / 2;
        (a, b, rx, height(rx, style))
    };
    // Each box's depth, its chain of boxes walked up only to a box whose
    // depth is known, so that every box is walked once.
    let mut nesting = vec![u32::MAX; n];
    let mut boxes: Vec<u32> = net.boxes().map(VertexId::get).collect();
    let mut chain = Vec::new();
    for &b in &boxes {
        let mut c = b;
        while c != NONE && nesting[c as usize] == u32::MAX {
            chain.push(c);
            c = inside(c);
        }
        let below = if c == NONE {
            0
        } else {
            nesting[c as usize] + 1
        };
        for (d, &c) in (below..).zip(chain.iter().rev()) {
            nesting[c as usize] = d;
        }
        chain.clear();
    }
    boxes.sort_unstable_by_key(|&b| std::cmp::Reverse(nesting[b as usize]));
    // Per depth (a box's `!` vertex, or `NONE`), its boxes by their left
    // side, and its links.
    let mut children_of: HashMap<u32, Vec<u32>> = HashMap::default();
    for &b in &boxes {
        children_of.entry(inside(b)).or_default().push(b);
    }
    for list in children_of.values_mut() {
        list.sort_unstable_by_key(|&b| sides(b).0);
    }
    let mut links_of: HashMap<u32, Vec<(u32, u32)>> = HashMap::default();
    for &(p, q) in net.links() {
        links_of
            .entry(inside(p.get()))
            .or_default()
            .push((p.get(), q.get()));
    }
    let mut lid = vec![0i64; n];
    let mut lift: HashMap<(u32, u32), i64> = HashMap::default();
    let mut depths: Vec<u32> = boxes.clone();
    depths.push(NONE);
    let mut highest = 0;
    for &depth in &depths {
        let inner = children_of.get(&depth).map_or(&[][..], Vec::as_slice);
        let lids: Vec<i64> = inner.iter().map(|&c| lid[c as usize]).collect();
        let mut tallest = lids.iter().copied().max().unwrap_or(0);
        let table = runs(&lids);
        for &(p, q) in links_of.get(&depth).map_or(&[][..], Vec::as_slice) {
            let (a, b, _, ry) = arc((p, q));
            // The boxes of this depth under the link, a run of the list,
            // since they lie apart.
            let from = inner.partition_point(|&c| sides(c).0 < a);
            let to = inner.partition_point(|&c| sides(c).1 <= b);
            let raised = most(&table, from, to);
            lift.insert((p, q), raised);
            tallest = tallest.max(raised + ry);
        }
        if depth == NONE {
            highest = tallest;
        } else {
            lid[depth as usize] = tallest + radius;
        }
    }
    // The rows as far apart as the most wires that run level between the
    // same two rows need, each half the radius from the next at least, so
    // that none reads as part of another: by default a wire runs above the
    // row of its `?` vertex; under the rules above the row of the node it
    // comes into, the first of a `?` without copies at its depth above its
    // landing row, and a lone wire above its formula's row.
    let line_height = {
        let mut level: HashMap<u32, i64> = HashMap::default();
        for (d, &(_, key)) in doors.pairs.iter().enumerate() {
            if !rules && target(d).is_none() {
                *level.entry(row[owner(key) as usize]).or_default() += 1;
            }
        }
        for q in (0..n as u32).filter(|&q| rules && kind_of(q) == Kind::Quest) {
            let (k, own) = (inputs[q as usize], kids(q as usize).count() as u32);
            let first = row[q as usize] + u32::from(!formulas) - levels[q as usize];
            let last = row[q as usize] - u32::from(formulas);
            for i in own..k {
                let band = match (k, i) {
                    (1, _) => row[q as usize],
                    (_, 0) => first,
                    _ if i.max(1) + 1 == k => last,
                    _ => first + i.max(1),
                };
                *level.entry(band).or_default() += 1;
            }
        }
        let most = level.values().copied().max().unwrap_or(0);
        let beside = if formulas { radius + HEIGHT + DEPTH } else { 0 };
        line_height.max(most * (radius / 2) + 3 * radius - beside)
    };
    let baseline = margin + stroke + highest + HEIGHT;
    let top = baseline - HEIGHT;
    let pitch = line_height + radius + HEIGHT + DEPTH;
    let rows = row.iter().copied().max().unwrap_or(0);
    let end = baseline - AXIS + i64::from(rows) * line_height + line_height / 2;
    let under = end + HEIGHT;
    // A `!` conclusion with auxiliary doors written on the bottom row,
    // under an edge down from its door.
    let lowered = |v: u32| {
        style.net_look.conclusions == Conclusions::Bottom
            && root(v)
            && kind_of(v) == Kind::Bang
            && has_doors[v as usize]
    };
    let text_y = |v: u32| match (formulas, root(v)) {
        (true, _) if lowered(v) => baseline + i64::from(bottom) * pitch,
        (true, _) => baseline + i64::from(row[v as usize]) * pitch,
        (false, true) => under,
        (false, false) => baseline,
    };
    // A vertex's joint, or a compact vertex's centre.
    let node = |v: u32| match formulas {
        true => (
            x[v as usize],
            baseline + i64::from(row[v as usize]) * pitch - HEIGHT - radius - radius / 2,
        ),
        false => (
            x[v as usize],
            baseline - AXIS + i64::from(row[v as usize]) * line_height,
        ),
    };
    // The centre of a node of the rules on row `r`: where the joints of
    // the row stand, or a compact circle of the row.
    let centre_at = |r: u32| match formulas {
        true => baseline + i64::from(r) * pitch - HEIGHT - radius - radius / 2,
        false => baseline - AXIS + i64::from(r) * line_height,
    };
    let depth = style.font.advances.depth();
    let foot = |v: u32| (x[v as usize], text_y(v) + depth);
    // Where an edge from a vertex below ends at `v`: a written formula's
    // foot, or a compact circle's rim towards `to`.
    let end_at = |v: u32, to: (i64, i64)| match formulas || forest.is_literal(occ(v)) {
        true => foot(v),
        false if bare(v) => node(v),
        false => towards(node(v), to, radius),
    };
    // The doors' points, innermost first: each halfway between what feeds
    // it, on its box's bottom border.
    let mut feeders: Vec<Vec<i64>> = vec![Vec::new(); doors.pairs.len()];
    let mut point = vec![(0i64, 0i64); doors.pairs.len()];
    let mut by_depth: Vec<usize> = (0..doors.pairs.len()).collect();
    by_depth.sort_unstable_by_key(|&d| std::cmp::Reverse(nesting[doors.pairs[d].0 as usize]));
    for c in 0..n as u32 {
        if boxed(c) {
            let d = doors.index(inside(c), key_of(c));
            feeders[d].push(x[c as usize]);
        }
    }
    for &d in &by_depth {
        let (b, _) = doors.pairs[d];
        let xs = &feeders[d];
        let at = xs.iter().sum::<i64>() / xs.len().max(1) as i64;
        point[d] = (at, node(b).1);
        if let Some(outer) = target(d) {
            feeders[outer].push(at);
        }
    }

    // A switching cycle, as the pairs of vertices its edges join; a wire's
    // edge joins its `?` vertex and the copy, or the box, it comes from.
    let mut cycle: Vec<(u32, u32)> = Vec::new();
    let mut witness = verdict.as_ref().err();
    while let Some(NetError::InBox { error, .. }) = witness {
        witness = Some(error);
    }
    if let Some(NetError::SwitchingCycle { cycle: vertices }) = witness {
        let next = vertices.iter().cycle().skip(1);
        cycle = vertices
            .iter()
            .zip(next)
            .map(|(&a, &b)| (a.get().min(b.get()), a.get().max(b.get())))
            .collect();
        cycle.sort_unstable();
    }
    let marks = |a: u32, b: u32| cycle.binary_search(&(a.min(b), a.max(b))).is_ok();

    let (mut lines, mut pars, mut links) = (String::new(), String::new(), String::new());
    let (mut marked, mut texts) = (String::new(), String::new());
    let dash = format!("{} {}", 3 * stroke, 3 * stroke);
    let segment = |out: &mut String, from: (i64, i64), to: (i64, i64), dashed: bool| {
        write!(out, r#"<path d="M{} {}L{} {}""#, from.0, from.1, to.0, to.1).unwrap();
        if dashed {
            write!(out, r#" stroke-dasharray="{dash}""#).unwrap();
        }
        out.push_str("/>\n");
    };
    // A symbol at a joint, a door or in a circle.
    let symbol_of = |v: u32| -> (&'static str, i64) {
        match kind_of(v) {
            Kind::Par => (NOTATION.par, PAR_MIDDLE),
            Kind::Tensor => (NOTATION.tensor, AXIS),
            Kind::Bang => (NOTATION.bang, PAR_MIDDLE),
            Kind::Quest => (NOTATION.quest, PAR_MIDDLE),
            Kind::Atom
            | Kind::DualAtom
            | Kind::One
            | Kind::Bot
            | Kind::Top
            | Kind::Zero
            | Kind::With
            | Kind::Plus => unreachable!("a literal has no symbol, a MELL net no unit or additive"),
        }
    };
    let label = |texts: &mut String, centre: (i64, i64), (sign, middle): (&str, i64)| {
        let label = run(sign, label_size, &style.font);
        text(
            texts,
            centre.0 - label.width / 2,
            centre.1 + middle * label_size / 1000,
            &label,
            &format!(r#" font-size="{label_size}""#),
            None,
        );
    };
    let circle = |out: &mut String, centre: (i64, i64), id: Option<&str>| {
        let id = id.map_or(String::new(), |id| format!(r#" id="{id}""#));
        writeln!(
            out,
            r#"<circle{id} cx="{}" cy="{}" r="{radius}"/>"#,
            centre.0, centre.1
        )
        .unwrap();
    };
    for v in 0..n as u32 {
        if written(v) {
            let named = formulas || forest.is_literal(occ(v));
            let attributes = if named {
                format!(r#" id="{}""#, id(v))
            } else {
                String::new()
            };
            formula.clear();
            write(&mut formula, v, false);
            let line = run(&formula, 1000, &style.font);
            text(
                &mut texts,
                x[v as usize] - from_left[v as usize],
                text_y(v),
                &line,
                &attributes,
                None,
            );
        }
        // Under the rules a `?` vertex is its nodes, drawn below.
        if forest.is_literal(occ(v)) || (rules && kind_of(v) == Kind::Quest) {
            continue;
        }
        let centre = node(v);
        if !formulas {
            circle(&mut lines, centre, Some(&id(v)));
        }
        label(&mut texts, centre, symbol_of(v));
        let switched = matches!(kind_of(v), Kind::Par | Kind::Quest) && dashed;
        let children: Vec<u32> = if kind_of(v) == Kind::Bang {
            net.premises(VertexId::new(v)).map(VertexId::get).collect()
        } else {
            kids(v as usize).collect()
        };
        for c in children {
            let to = end_at(c, centre);
            let from = towards(centre, to, radius);
            let out = match (marks(v, c), switched) {
                (true, _) => &mut marked,
                (false, true) => &mut pars,
                (false, false) => &mut lines,
            };
            segment(out, from, to, switched && marks(v, c));
        }
        if formulas && lowered(v) {
            let foot = (centre.0, text_y(v) - HEIGHT);
            segment(&mut lines, (centre.0, centre.1 + radius), foot, false);
        }
        // A compact conclusion's edge hangs from its circle to its text.
        if root(v) && !formulas {
            writeln!(
                lines,
                r#"<path d="M{} {}V{end}"/>"#,
                centre.0,
                centre.1 + radius
            )
            .unwrap();
        }
    }
    // A node of the rules: its label at `centre`, narrowed to fit inside
    // the node's radius, in a circle when compact and `circled` (the last
    // node of a `?` vertex has its own, with the vertex's id).
    let rule_node =
        |lines: &mut String, texts: &mut String, centre: (i64, i64), sign: &str, circled: bool| {
            if circled && !formulas {
                circle(lines, centre, None);
            }
            let room = 2 * radius - 4 * stroke;
            let full = run(sign, label_size, &style.font).width;
            let size = if full > room {
                label_size * room / full
            } else {
                label_size
            };
            let label = run(sign, size, &style.font);
            text(
                texts,
                centre.0 - label.width / 2,
                centre.1 + PAR_MIDDLE * size / 1000,
                &label,
                &format!(r#" font-size="{size}""#),
                None,
            );
        };
    let (dereliction, contraction, weakening) = (
        format!("{}d", NOTATION.quest),
        format!("{}c", NOTATION.quest),
        format!("{}w", NOTATION.quest),
    );
    // Under the rules, every `?` vertex's nodes, and where each of its
    // wires ends (the node, how far from it, and whether from above): a
    // dereliction under each copy at its depth on the first of its rows,
    // the contractions in a column over its formula, one a row, joining
    // the copies first and then the wires, each the nearest to the column
    // first, and the last node over its formula, or its circle when
    // compact; a weakening alone, and a lone wire into its formula. A wire
    // runs level between two rows, as the wires of the one `?` node do:
    // into a contraction from the side along twice the radius above it,
    // and the first one, with wires alone, along twice the radius above
    // the row over it and down the column, so that no two share a leg.
    let mut entry: HashMap<usize, End> = HashMap::default();
    for q in (0..n as u32).filter(|&q| rules && kind_of(q) == Kind::Quest) {
        let k = inputs[q as usize];
        let wires = doors
            .pairs
            .iter()
            .enumerate()
            .filter(|&(d, &(_, key))| owner(key) == q && target(d).is_none());
        let down = parent(q) != NONE && marks(parent(q), q);
        // A compact conclusion's edge, from `from` down to its text.
        let hanging = |lines: &mut String, marked: &mut String, from: i64| {
            let target = if down { marked } else { lines };
            writeln!(target, r#"<path d="M{} {from}V{end}"/>"#, x[q as usize]).unwrap();
        };
        if bare(q) {
            let into = match formulas {
                true => (x[q as usize], text_y(q) - HEIGHT),
                false => node(q),
            };
            for (d, _) in wires {
                let above = Some(node(q).1 - 2 * radius);
                entry.insert(
                    d,
                    End {
                        at: into,
                        into: 0,
                        above,
                    },
                );
            }
            if !formulas && root(q) {
                hanging(&mut lines, &mut marked, into.1);
            }
            continue;
        }
        let last = match formulas {
            true => (x[q as usize], centre_at(row[q as usize] - 1)),
            false => node(q),
        };
        if formulas {
            let into = (last.0, text_y(q) - HEIGHT);
            let target = if down { &mut marked } else { &mut lines };
            segment(target, (last.0, last.1 + radius), into, false);
        } else {
            writeln!(
                lines,
                r#"<circle id="{}" cx="{}" cy="{}" r="{radius}"/>"#,
                id(q),
                last.0,
                last.1
            )
            .unwrap();
            if root(q) {
                hanging(&mut lines, &mut marked, last.1 + radius);
            }
        }
        if k == 0 {
            rule_node(&mut lines, &mut texts, last, &weakening, false);
            continue;
        }
        // The premises by their distance from the column, each with its
        // place and whether the cycle runs through it.
        let mut premises: Vec<(i64, Option<u32>, Option<usize>, bool)> = kids(q as usize)
            .map(|c| (x[c as usize], Some(c), None, marks(q, c)))
            .chain(wires.map(|(d, &(b, _))| (point[d].0, None, Some(d), marks(q, b))))
            .collect();
        let column = x[q as usize];
        premises.sort_unstable_by_key(|&(at, copy, ..)| (copy.is_none(), (at - column).abs(), at));
        let first = row[q as usize] + u32::from(!formulas) - levels[q as usize];
        let k = k as usize;
        // Contraction `j` (from 1) joins premise `j`, and the first two.
        let contraction_at = |j: usize| match j + 1 == k {
            true => last,
            false => (column, centre_at(first + j as u32)),
        };
        // The contractions on the cycle's way: from the first premise on it
        // to the last, or down to the formula.
        let on: Vec<usize> = premises
            .iter()
            .enumerate()
            .filter(|p| p.1.3)
            .map(|(i, _)| i.max(1))
            .collect();
        let from_on = on.iter().copied().min().unwrap_or(k - 1);
        let to_on = if down {
            k - 1
        } else {
            on.iter().copied().max().unwrap_or(0)
        };
        for (i, &(_, copy, wire, mark)) in premises.iter().enumerate() {
            let into = contraction_at(i.max(1));
            if let Some(d) = wire {
                let above = (i == 0).then(|| centre_at(first) - 2 * radius);
                entry.insert(
                    d,
                    End {
                        at: into,
                        into: radius,
                        above,
                    },
                );
                continue;
            }
            let c = copy.expect("a copy or a wire");
            let at = match k {
                1 => last,
                _ => (x[c as usize], centre_at(first)),
            };
            rule_node(&mut lines, &mut texts, at, &dereliction, k > 1);
            let from = end_at(c, at);
            let target = if mark { &mut marked } else { &mut lines };
            segment(target, from, towards(at, from, radius), false);
            if k > 1 {
                let target = match (mark, dashed) {
                    (true, _) => &mut marked,
                    (false, true) => &mut pars,
                    (false, false) => &mut lines,
                };
                segment(
                    target,
                    towards(at, into, radius),
                    towards(into, at, radius),
                    mark && dashed,
                );
            }
        }
        for j in 1..k {
            let at = contraction_at(j);
            rule_node(&mut lines, &mut texts, at, &contraction, j + 1 < k);
            if j + 1 < k {
                let below = contraction_at(j + 1);
                let mark = from_on <= j && j < to_on;
                let target = match (mark, dashed) {
                    (true, _) => &mut marked,
                    (false, true) => &mut pars,
                    (false, false) => &mut lines,
                };
                segment(
                    target,
                    (column, at.1 + radius),
                    (column, below.1 - radius),
                    mark && dashed,
                );
            }
        }
    }
    // The wires: from each boxed copy into its door (under the rules from
    // its dereliction, a row below it in its box), and from each door to
    // the door of the box around it or to its `?` vertex. Under the rules
    // a wire runs through its doors, which have no symbol of their own.
    let door_radius = if rules { 0 } else { radius };
    for c in 0..n as u32 {
        if !boxed(c) {
            continue;
        }
        let q = parent(c);
        let d = doors.index(inside(c), key_of(c));
        let to = point[d];
        let switched = dashed && !bare(q);
        let from = if rules {
            let at = (x[c as usize], centre_at(row[c as usize] + 1));
            let from = end_at(c, at);
            let target = if marks(q, c) { &mut marked } else { &mut lines };
            segment(target, from, towards(at, from, radius), false);
            rule_node(&mut lines, &mut texts, at, &dereliction, true);
            towards(at, to, radius)
        } else {
            end_at(c, to)
        };
        let target = match (marks(q, c), switched) {
            (true, _) => &mut marked,
            (false, true) => &mut pars,
            (false, false) => &mut lines,
        };
        segment(
            target,
            from,
            towards(to, from, door_radius),
            marks(q, c) && switched,
        );
    }
    // The legs of the wires that run level between the same two rows
    // apart, so that no two share one, wires bound for different `?`
    // vertices included: going left, the wire whose door is further left
    // higher, and going right the one further right, so that where their
    // spans overlap the way down of one does not cross the other's leg.
    // They stay clear of the circles above.
    let mut legs: HashMap<usize, i64> = HashMap::default();
    {
        let mut bands: HashMap<i64, Vec<(usize, i64, i64)>> = HashMap::default();
        for d in (0..doors.pairs.len()).filter(|&d| target(d).is_none()) {
            let (to, band) = if rules {
                let End { at: to, above, .. } = entry[&d];
                (to, above.unwrap_or(to.1 - 2 * radius))
            } else {
                let to = node(owner(doors.pairs[d].1));
                (to, to.1 - 2 * radius)
            };
            bands.entry(band).or_default().push((d, point[d].0, to.0));
        }
        let room = if formulas { pitch } else { line_height } - 3 * radius;
        for (band, mut wires) in bands {
            wires.sort_unstable_by_key(|&(_, from, to)| match from > to {
                true => (false, from),
                false => (true, -from),
            });
            let step = radius.min(room / wires.len() as i64);
            for (i, &(d, ..)) in wires.iter().enumerate() {
                legs.insert(d, band - (wires.len() - 1 - i) as i64 * step);
            }
        }
    }
    for (d, &(b, key)) in doors.pairs.iter().enumerate() {
        let q = owner(key);
        let from = point[d];
        let (to, into, above) = match target(d) {
            Some(outer) => (point[outer], door_radius, None),
            None if rules => {
                let End {
                    at: to,
                    into,
                    above,
                } = entry[&d];
                (to, into, Some(above))
            }
            None => (node(q), radius, None),
        };
        let mark = marks(q, b);
        let switched = dashed && !bare(q);
        let out = match (mark, switched) {
            (true, _) => &mut marked,
            (false, true) => &mut pars,
            (false, false) => &mut lines,
        };
        let dashing = mark && switched;
        if let Some(Some(_)) = above {
            // Down from the door, along its leg to the column and down into
            // a node of the rules or a formula.
            let leg = legs[&d];
            write!(out, r#"<path d="M{} {}"#, from.0, from.1).unwrap();
            if from.0 != to.0 {
                write!(out, "V{leg}H{}", to.0).unwrap();
            }
            write!(out, "V{}", to.1 - into).unwrap();
            out.push('"');
            if dashing {
                write!(out, r#" stroke-dasharray="{dash}""#).unwrap();
            }
            out.push_str("/>\n");
            continue;
        }
        // Straight down from the door, along twice the radius above the
        // row of what it goes to, and into it from the side it comes from,
        // so that it crosses no joint or circle on the way.
        let leg = legs.get(&d).copied().unwrap_or(to.1 - 2 * radius);
        if (from.0 - to.0).abs() <= 2 * radius || leg <= from.1 + radius {
            segment(
                out,
                towards(from, to, door_radius),
                towards(to, from, into),
                dashing,
            );
        } else {
            let side = (from.0 - to.0).signum() * 2 * radius;
            let corner = (to.0 + side, leg);
            let end = towards(to, corner, into);
            write!(
                out,
                r#"<path d="M{} {}V{leg}H{}L{} {}""#,
                from.0,
                from.1 + door_radius,
                corner.0,
                end.0,
                end.1
            )
            .unwrap();
            if dashing {
                write!(out, r#" stroke-dasharray="{dash}""#).unwrap();
            }
            out.push_str("/>\n");
        }
        if rules {
            continue;
        }
        if !formulas {
            circle(&mut lines, from, None);
        }
        label(&mut texts, from, (NOTATION.quest, PAR_MIDDLE));
    }
    // Each box's rectangle, its bottom border broken at its doors.
    for b in net.boxes().map(VertexId::get) {
        let (left, right) = sides(b);
        let bottom = node(b).1;
        let ceiling = top - lid[b as usize];
        let mut gaps: Vec<i64> = vec![x[b as usize]];
        // Under the rules a wire crosses the border, which has no door.
        if !rules {
            gaps.extend(
                doors.pairs[doors.pairs.partition_point(|&(c, _)| c < b)..]
                    .iter()
                    .take_while(|&&(c, _)| c == b)
                    .map(|&(c, q)| point[doors.index(c, q)].0),
            );
        }
        gaps.sort_unstable();
        let mut at = gaps[0] + radius;
        write!(lines, r#"<path id="b{b}" d="M{at} {bottom}"#).unwrap();
        for &door in &gaps[1..] {
            if door - radius > at {
                write!(lines, "H{}", door - radius).unwrap();
            }
            at = at.max(door + radius);
            write!(lines, "M{at} {bottom}").unwrap();
        }
        writeln!(
            lines,
            r#"H{right}V{ceiling}H{left}V{bottom}H{}"/>"#,
            gaps[0] - radius
        )
        .unwrap();
    }
    for &(p, q) in net.links() {
        let (p, q) = (p.get(), q.get());
        let (a, b, rx, ry) = arc((p, q));
        let rise = lift.get(&(p, q)).copied().unwrap_or(0);
        let out = if marks(p, q) { &mut marked } else { &mut links };
        let (left, right) = if x[p as usize] < x[q as usize] {
            (p, q)
        } else {
            (q, p)
        };
        let (down_left, down_right) = (text_y(left) - HEIGHT, text_y(right) - HEIGHT);
        let level = top - rise;
        write!(
            out,
            r#"<path id="l{}-{}" d="M{a} {down_left}"#,
            p.min(q),
            p.max(q)
        )
        .unwrap();
        if down_left != level {
            write!(out, "V{level}").unwrap();
        }
        write!(out, "A{rx} {ry} 0 0 1 {b} {level}").unwrap();
        if down_right != level {
            write!(out, "V{down_right}").unwrap();
        }
        out.push_str("\"/>\n");
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
        if dashed {
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
        true => baseline + i64::from(rows) * pitch + DEPTH + margin,
        false => under + DEPTH + margin,
    };
    let description = style.description.then(|| net.to_string());
    document(style, &title, description.as_deref(), (width, height), body)
}

/// Returns the most of every run of `values` whose length is a power of
/// two, by length: `table[k][i]` is the most of `values[i..i + 2^k]`.
fn runs(values: &[i64]) -> Vec<Vec<i64>> {
    let mut table = vec![values.to_vec()];
    let mut span = 1;
    while 2 * span <= values.len() {
        let last = &table[table.len() - 1];
        let next = (0..=values.len() - 2 * span)
            .map(|i| last[i].max(last[i + span]))
            .collect();
        table.push(next);
        span *= 2;
    }
    table
}

/// Returns the most of `values[from..to]` from its [`runs`], or 0 for an
/// empty range.
fn most(table: &[Vec<i64>], from: usize, to: usize) -> i64 {
    if from >= to {
        return 0;
    }
    let k = (to - from).ilog2() as usize;
    table[k][from].max(table[k][to - (1 << k)])
}
