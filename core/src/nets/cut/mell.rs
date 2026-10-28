// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use super::{Step, within};
use crate::Error;
use crate::limits::{Limits, Progress};
use crate::nets::{MellStructure, NONE, NetError, Parts, Placed, VertexId};
use crate::occurrences::OccId;
use crate::proofs::cut::{Case, Options};
use crate::sequents::Kind;

/// A structure with boxes taken apart for a step: its vertices, each
/// with its occurrence, parent and box, those the step erased marked, and
/// its cut links, links and weakenings, all by raw vertex ids.
struct Work {
    /// Per vertex, its occurrence.
    occurrence: Vec<OccId>,
    /// Per vertex, the vertex it is a premise of, or `NONE`.
    parent: Vec<u32>,
    /// Per vertex, the `!` vertex of its innermost box, or `NONE`.
    inside: Vec<u32>,
    /// Per vertex, whether the step erased it.
    erased: Vec<bool>,
    /// The cut links, each from its end in a cut formula's tree.
    cuts: Vec<(u32, u32)>,
    /// The links.
    links: Vec<(u32, u32)>,
    /// The weakenings and the literals they hang on.
    hangs: Vec<(u32, u32)>,
    /// The number of roots, vertices `0` to `roots − 1`.
    roots: usize,
}

impl Work {
    /// Takes a structure apart.
    fn of(net: &MellStructure) -> Self {
        let raw = |v: Option<VertexId>| v.map_or(NONE, VertexId::get);
        let parts = net.parts();
        let pair = |&(x, y): &(VertexId, VertexId)| (x.get(), y.get());
        Self {
            occurrence: net.vertices().map(|v| net.occurrence(v).unwrap()).collect(),
            parent: net.vertices().map(|v| raw(net.parent(v))).collect(),
            inside: net.vertices().map(|v| raw(net.inside(v))).collect(),
            erased: vec![false; net.len()],
            cuts: parts.cuts.iter().map(pair).collect(),
            links: parts.links.iter().map(pair).collect(),
            hangs: parts.weakenings.iter().map(pair).collect(),
            roots: net.forest().roots().len(),
        }
    }

    /// Returns the number of vertices the step left.
    fn vertices(&self) -> usize {
        self.erased.iter().filter(|&&e| !e).count()
    }

    /// Returns the premises of a vertex left, in vertex order.
    fn premises(&self, v: u32) -> Vec<u32> {
        (0..self.parent.len() as u32)
            .filter(|&w| self.parent[w as usize] == v && !self.erased[w as usize])
            .collect()
    }

    /// Returns whether the vertex lies in the box of `b`, at any depth.
    fn within(&self, v: u32, b: u32) -> bool {
        let mut c = self.inside[v as usize];
        while c != NONE {
            if c == b {
                return true;
            }
            c = self.inside[c as usize];
        }
        false
    }

    /// Returns the box of the `!` vertex `b`: `b` and every vertex inside
    /// it, at any depth, in vertex order.
    fn contents(&self, b: u32) -> Vec<u32> {
        (0..self.parent.len() as u32)
            .filter(|&v| !self.erased[v as usize] && (v == b || self.within(v, b)))
            .collect()
    }

    /// Adds a vertex and returns it.
    fn push(&mut self, occurrence: OccId, parent: u32, inside: u32) -> u32 {
        self.occurrence.push(occurrence);
        self.parent.push(parent);
        self.inside.push(inside);
        self.erased.push(false);
        (self.occurrence.len() - 1) as u32
    }

    /// Erases the box of `b` and every link, cut link and weakening in it.
    fn erase(&mut self, b: u32) {
        let gone = self.contents(b);
        for &v in &gone {
            self.erased[v as usize] = true;
        }
        let erased = &self.erased;
        let kept = |&(x, y): &(u32, u32)| !erased[x as usize] && !erased[y as usize];
        self.links.retain(kept);
        self.cuts.retain(kept);
        self.hangs.retain(|&(q, _)| !erased[q as usize]);
    }

    /// Duplicates the box of `b` and returns the duplicate's `!` vertex:
    /// every vertex in it, the links, cut links and weakenings inside, and
    /// for a copy in it of a `?` vertex outside, a copy of that vertex.
    fn duplicate(&mut self, b: u32) -> u32 {
        let inner = self.contents(b);
        let mut copy = vec![NONE; self.parent.len()];
        for &v in &inner {
            copy[v as usize] = self.push(self.occurrence[v as usize], NONE, NONE);
        }
        let image = |v: u32| match copy[v as usize] {
            NONE => v,
            c => c,
        };
        for &v in &inner {
            let c = copy[v as usize] as usize;
            let (p, i) = (self.parent[v as usize], self.inside[v as usize]);
            self.parent[c] = if p == NONE { NONE } else { image(p) };
            self.inside[c] = if i == NONE { NONE } else { image(i) };
        }
        let within = |&(x, y): &(u32, u32)| copy[x as usize] != NONE && copy[y as usize] != NONE;
        let twin = |&(x, y): &(u32, u32)| (image(x), image(y));
        let links: Vec<(u32, u32)> = self.links.iter().filter(|l| within(l)).map(twin).collect();
        let cuts: Vec<(u32, u32)> = self.cuts.iter().filter(|l| within(l)).map(twin).collect();
        let hangs: Vec<(u32, u32)> = self.hangs.iter().filter(|l| within(l)).map(twin).collect();
        self.links.extend(links);
        self.cuts.extend(cuts);
        self.hangs.extend(hangs);
        copy[b as usize]
    }

    /// Returns the parts of what the step left, renumbered: the roots,
    /// then depth first every root's tree and every cut link's two trees,
    /// a vertex's premises in vertex order.
    fn parts(&self) -> Parts {
        let n = self.parent.len();
        let mut premises = vec![Vec::new(); n];
        for v in 0..n {
            let p = self.parent[v];
            if p != NONE && !self.erased[v] {
                premises[p as usize].push(v as u32);
            }
        }
        let mut new = vec![NONE; n];
        let mut order: Vec<u32> = Vec::with_capacity(n);
        for r in 0..self.roots as u32 {
            new[r as usize] = r;
            order.push(r);
        }
        let ends = self.cuts.iter().flat_map(|&(x, y)| [x, y]);
        let mut stack = Vec::new();
        for top in (0..self.roots as u32).chain(ends) {
            stack.push(top);
            while let Some(v) = stack.pop() {
                if new[v as usize] == NONE {
                    new[v as usize] = order.len() as u32;
                    order.push(v);
                }
                stack.extend(premises[v as usize].iter().rev());
            }
        }
        let renamed = |v: u32| VertexId::new(new[v as usize]);
        let vertices = order[self.roots..]
            .iter()
            .map(|&v| {
                let (o, p, i) = (
                    self.occurrence[v as usize],
                    self.parent[v as usize],
                    self.inside[v as usize],
                );
                let inside = (i != NONE).then(|| renamed(i));
                match p {
                    NONE => Placed::cut(o, inside),
                    p => Placed::new(o, renamed(p), inside),
                }
            })
            .collect();
        let pairs = |list: &[(u32, u32)]| {
            list.iter()
                .map(|&(x, y)| (renamed(x), renamed(y)))
                .collect()
        };
        Parts {
            vertices,
            cuts: pairs(&self.cuts),
            links: pairs(&self.links),
            weakenings: pairs(&self.hangs),
        }
    }
}

impl crate::sealed::Net for MellStructure {
    fn cut_links(&self) -> &[(VertexId, VertexId)] {
        self.cuts()
    }

    fn check(&self, stop: &mut dyn FnMut(Progress) -> bool) -> Result<(), NetError> {
        self.is_correct(stop)
    }

    fn reduce(
        &self,
        (x, y): (VertexId, VertexId),
        options: &Options,
        limits: &Limits,
    ) -> Result<(Self, Step), Error> {
        let forest = self.forest();
        let mut work = Work::of(self);
        let at = work
            .cuts
            .iter()
            .position(|&cut| cut == (x.get(), y.get()))
            .expect("a cut link of the net");
        let (x, y) = (x.get(), y.get());
        let kind = |work: &Work, v: u32| forest.kind(work.occurrence[v as usize]);
        let mut above = None;
        let case = match kind(&work, x) {
            Kind::Atom | Kind::DualAtom => {
                // A proof net links every literal, and never a cut's two
                // ends to each other.
                let partner = |v: u32| {
                    self.partner(VertexId::new(v))
                        .expect("a proof net links every literal")
                        .get()
                };
                let (px, py) = (partner(x), partner(y));
                let ends = |&(u, v): &(u32, u32)| [x, y].contains(&u) || [x, y].contains(&v);
                let first = work.links.iter().position(ends).expect("the literal's link");
                work.links[first] = (px, py);
                work.links.retain(|link| !ends(link));
                // A weakening hung on either literal hangs on the link made.
                for hang in &mut work.hangs {
                    if hang.1 == x || hang.1 == y {
                        hang.1 = px;
                    }
                }
                work.erased[x as usize] = true;
                work.erased[y as usize] = true;
                work.cuts.remove(at);
                Case::Axiom
            }
            Kind::Tensor | Kind::Par => {
                let (px, py) = (work.premises(x), work.premises(y));
                // The left premise first, as the structure keeps them.
                let left = |v: u32, list: &[u32]| {
                    let l = forest.left(work.occurrence[v as usize]).unwrap();
                    if work.occurrence[list[0] as usize] == l {
                        (list[0], list[1])
                    } else {
                        (list[1], list[0])
                    }
                };
                let ((xl, xr), (yl, yr)) = (left(x, &px), left(y, &py));
                for v in [xl, xr, yl, yr] {
                    work.parent[v as usize] = NONE;
                }
                work.erased[x as usize] = true;
                work.erased[y as usize] = true;
                work.cuts.splice(at..=at, [(xl, yl), (xr, yr)]);
                Case::TensorPar
            }
            Kind::Bang | Kind::Quest => {
                let (b, q) = if kind(&work, x) == Kind::Bang {
                    (x, y)
                } else {
                    (y, x)
                };
                let copies = work.premises(q);
                let depth = work.inside[q as usize];
                match copies[..] {
                    [] => {
                        let hung = self
                            .hangs_on(VertexId::new(q))
                            .expect("a proof net hangs every weakening")
                            .get();
                        work.cuts.remove(at);
                        work.erase(b);
                        debug_assert!(
                            !work.erased[hung as usize],
                            "a weakening cut against a box hangs outside it in a proof net"
                        );
                        work.erased[q as usize] = true;
                        work.hangs.retain(|&(w, _)| w != q);
                        // What lost its copies, or its literal, in the box
                        // hangs where the erased weakening hung.
                        for hang in &mut work.hangs {
                            if work.erased[hang.1 as usize] {
                                hang.1 = hung;
                            }
                        }
                        let bare: Vec<u32> = (0..work.parent.len() as u32)
                            .filter(|&r| {
                                !work.erased[r as usize]
                                    && kind(&work, r) == Kind::Quest
                                    && self.premises(VertexId::new(r)).next().is_some()
                                    && work.premises(r).is_empty()
                            })
                            .collect();
                        work.hangs.extend(bare.into_iter().map(|r| (r, hung)));
                        Case::Weakening
                    }
                    [c] if work.inside[c as usize] == depth => {
                        let premise = work.premises(b)[0];
                        for v in 0..work.inside.len() {
                            if work.inside[v] == b {
                                work.inside[v] = depth;
                            }
                        }
                        work.parent[premise as usize] = NONE;
                        work.parent[c as usize] = NONE;
                        work.erased[b as usize] = true;
                        work.erased[q as usize] = true;
                        work.cuts[at] = if b == x { (premise, c) } else { (c, premise) };
                        above = Some(c);
                        Case::Dereliction
                    }
                    [c] => {
                        // The box at the cut's depth around the copy.
                        let mut e = work.inside[c as usize];
                        while work.inside[e as usize] != depth {
                            e = work.inside[e as usize];
                        }
                        work.inside[b as usize] = e;
                        work.inside[q as usize] = e;
                        above = Some(e);
                        Case::BoxIntoBox
                    }
                    [c, ..] => {
                        let twin = work.duplicate(b);
                        let own = work.push(work.occurrence[q as usize], NONE, depth);
                        work.parent[c as usize] = own;
                        let cut = if b == x { (twin, own) } else { (own, twin) };
                        work.cuts.insert(at, cut);
                        above = Some(c);
                        Case::Contraction
                    }
                }
            }
            Kind::One | Kind::Bot | Kind::Top | Kind::Zero | Kind::With | Kind::Plus => {
                unreachable!("a structure with boxes is of unit-free MELL")
            }
        };
        within(work.vertices(), options)?;
        let parts = work.parts();
        let made = Self::from_parts(forest.clone(), self.criterion(), &parts, limits)?;
        let step = Step {
            cut: (VertexId::new(x), VertexId::new(y)),
            case,
            above: above.map(VertexId::new),
        };
        Ok((made, step))
    }
}
