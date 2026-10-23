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
use super::{NOTATION, NetDrawing, PLAIN, RAISED_BOT, Style, document, escaped, lay, run, text};
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

/// The auxiliary doors of a structure: every pair of a box and a `?`
/// vertex outside it with a copy inside, sorted, each with what feeds it
/// and where its wire goes.
struct Doors {
    /// The pairs (`!` vertex, `?` vertex), sorted.
    pairs: Vec<(u32, u32)>,
}

impl Doors {
    /// Returns the doors of a structure: for each copy outside its `?`
    /// vertex's box, a door on every box from the copy's out to the
    /// `?`'s, a box's chain walked only until a door already found.
    fn of(net: &MellStructure) -> Self {
        let mut found: crate::hash::HashSet<(u32, u32)> = crate::hash::HashSet::default();
        for c in net.vertices() {
            let Some(q) = net.parent(c).filter(|&q| kind(net, q) == Kind::Quest) else {
                continue;
            };
            let home = raw(net.inside(q));
            let mut b = raw(net.inside(c));
            while b != home && found.insert((b, q.get())) {
                b = raw(net.inside(VertexId::new(b)));
            }
        }
        let mut pairs: Vec<(u32, u32)> = found.into_iter().collect();
        pairs.sort_unstable();
        Self { pairs }
    }

    /// Returns the index of the door of box `b` for `q`.
    fn index(&self, b: u32, q: u32) -> usize {
        self.pairs
            .binary_search(&(b, q))
            .expect("a door on the chain of a copy")
    }
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
        }
    }
    let doors = Doors::of(net).pairs.len() as u64;
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
    let doors = Doors::of(net);
    let target = |d: usize| {
        let (b, q) = doors.pairs[d];
        let outer = inside(b);
        (outer != inside(q)).then(|| doors.index(outer, q))
    };
    let mut last: HashMap<u32, Vec<u32>> = HashMap::default();
    for (d, &(b, q)) in doors.pairs.iter().enumerate() {
        if target(d).is_none() {
            last.entry(q).or_default().push(b);
        }
    }

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
            let lowest = on.iter().map(|&w| layer[w as usize]).max();
            layer[v as usize] = match (lowest, forest.is_literal(occ(v))) {
                (_, true) => 0,
                (Some(l), false) => l + 1,
                (None, false) => 1,
            };
        } else {
            stack.push((v, true));
            stack.extend(on.into_iter().map(|w| (w, false)));
        }
    }
    // The conclusions on the bottom row, and the copies of a `?` vertex with
    // several at its own depth on the row above it, so that their edges
    // into it pass below every formula and box of the others; a `!` vertex with
    // auxiliary doors stays where it is, above the `?` vertices its wires
    // go to.
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
        } else if kind_of(p) == Kind::Quest && !boxed(v) && kids(p as usize).nth(1).is_some() {
            row[v as usize] = row[v as usize].max(row[p as usize] - 1);
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
    let baseline = margin + stroke + highest + HEIGHT;
    let top = baseline - HEIGHT;
    let pitch = line_height + radius + HEIGHT + DEPTH;
    let rows = row.iter().copied().max().unwrap_or(0);
    let end = baseline - AXIS + i64::from(rows) * line_height + line_height / 2;
    let under = end + HEIGHT;
    let text_y = |v: u32| match (formulas, root(v)) {
        (true, _) => baseline + i64::from(row[v as usize]) * pitch,
        (false, true) => under,
        (false, false) => baseline,
    };
    // A vertex's joint, or a compact vertex's centre.
    let node = |v: u32| match formulas {
        true => (x[v as usize], text_y(v) - HEIGHT - radius - radius / 2),
        false => (
            x[v as usize],
            baseline - AXIS + i64::from(row[v as usize]) * line_height,
        ),
    };
    let depth = style.font.advances.depth();
    let foot = |v: u32| (x[v as usize], text_y(v) + depth);
    // Where an edge from a vertex below ends at `v`: a written formula's
    // foot, or a compact circle's rim towards `to`.
    let end_at = |v: u32, to: (i64, i64)| match formulas || forest.is_literal(occ(v)) {
        true => foot(v),
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
            let d = doors.index(inside(c), parent(c));
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
        if forest.is_literal(occ(v)) {
            continue;
        }
        let centre = node(v);
        if !formulas {
            circle(&mut lines, centre, Some(&id(v)));
        }
        label(&mut texts, centre, symbol_of(v));
        let switched = matches!(kind_of(v), Kind::Par | Kind::Quest);
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
    // The wires: from each boxed copy into its door, and from each door to
    // the door of the box around it or to its `?` vertex.
    for c in 0..n as u32 {
        if !boxed(c) {
            continue;
        }
        let q = parent(c);
        let d = doors.index(inside(c), q);
        let to = point[d];
        let from = end_at(c, to);
        let out = if marks(q, c) { &mut marked } else { &mut pars };
        segment(out, from, towards(to, from, radius), marks(q, c));
    }
    for (d, &(b, q)) in doors.pairs.iter().enumerate() {
        let from = point[d];
        let to = target(d).map_or_else(|| node(q), |outer| point[outer]);
        let mark = marks(q, b);
        let out = if mark { &mut marked } else { &mut pars };
        // Straight down from the door, along twice the radius above the
        // row of what it goes to, and into it from the side it comes from,
        // so that it crosses no joint or circle on the way.
        let leg = to.1 - 2 * radius;
        if (from.0 - to.0).abs() <= 2 * radius || leg <= from.1 + radius {
            segment(
                out,
                towards(from, to, radius),
                towards(to, from, radius),
                mark,
            );
        } else {
            let side = (from.0 - to.0).signum() * 2 * radius;
            let corner = (to.0 + side, leg);
            let end = towards(to, corner, radius);
            write!(
                out,
                r#"<path d="M{} {}V{leg}H{}L{} {}""#,
                from.0,
                from.1 + radius,
                corner.0,
                end.0,
                end.1
            )
            .unwrap();
            if mark {
                write!(out, r#" stroke-dasharray="{dash}""#).unwrap();
            }
            out.push_str("/>\n");
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
        gaps.extend(
            doors.pairs[doors.pairs.partition_point(|&(c, _)| c < b)..]
                .iter()
                .take_while(|&&(c, _)| c == b)
                .map(|&(c, q)| point[doors.index(c, q)].0),
        );
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
        group(
            out,
            &style.par,
            &format!(r#" stroke-dasharray="{dash}""#),
            &pars,
        );
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
