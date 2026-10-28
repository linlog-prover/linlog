// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The structure graph of a structure with boxes: the graphs of every box
//! depth side by side, each with the boxes inside it contracted to their
//! `!` vertex and the `?` vertices outside it replaced by its auxiliary
//! doors, and the correctness criterion's tests on it.
//!
//! The vertices are the structure's, each at the depth of its innermost
//! box, one *door vertex* for each `?` vertex and each box inside its
//! own that a wire of it enters: inside that box the door stands for the
//! `?` vertex, and after the doors one *cut vertex* per cut link, a `⊗`
//! without a conclusion whose premises are the link's two ends, so that
//! the graph stays simple where a wire of a `?` end enters the box of the
//! `!` end. A box's `!` vertex lies at the depth around the box and
//! takes the box's place there, so a wire entering the box ends at it.
//! The edges, each between two vertices of one depth: a `⊗` or `⅋` to its
//! premises; a `?` vertex or a door to each copy at its depth, to each
//! weakening literal there and to the `!` vertex of each box a wire
//! enters (once, however many wires do); a cut vertex to its two ends; and
//! the links. Slot 0 of every vertex is its *parent edge*, to the vertex
//! below it at its depth (a cut link's end to its cut vertex), or `NONE`
//! for a conclusion of its depth (a root, a box's premise, a door, a cut
//! vertex).
//!
//! The colouring is the one of the multiplicative graph, with `?` vertices
//! and doors switched like a `⅋` of any arity: their wires share a colour
//! of their own, every other edge has its own. Two wires of one vertex to
//! one other are one edge: they share a colour, so no properly coloured
//! cycle uses both, and a switching keeps one of them as it keeps either.
//! That leaves the graph simple, and the deletion condition of Yeo's
//! theorem the multiplicative one: a switched vertex is deletable when its
//! parent edge is absent or a bridge, any other when all its edges are.

use super::{DOOR_BYTES, MOST, MellStructure, Nesting};
use crate::Error;
use crate::limits::{Limits, Phase, Progress, Refusal, Space};
use crate::nets::{NONE, NetError, STOPPED, VertexId, afford};
use crate::sequents::Kind;

/// The graph of every box depth.
#[derive(Clone, Debug, Default)]
pub(super) struct Graph {
    /// Per vertex, where its slots start in `to`; one more entry holds the
    /// total, which the construction keeps below `u32::MAX`.
    start: Box<[u32]>,
    /// Per slot, the other end of the edge, or `NONE` for a parent edge
    /// that is absent.
    to: Box<[u32]>,
    /// Per vertex, whether its wires or premise edges are switched.
    switched: Box<[bool]>,
    /// Per vertex, its depth: the `!` vertex of its box, or `NONE`.
    level: Box<[u32]>,
    /// The structure's vertex count: vertex `n + i` is door `i`.
    n: u32,
    /// Per door, the `?` vertex it stands for.
    door_of: Box<[u32]>,
    /// Per cut vertex, which follow the doors, its two ends, the one in a
    /// cut formula's tree first.
    cut_of: Box<[(u32, u32)]>,
    /// The depths in the order the criterion checks them, outside every
    /// box first, then each box by its `!` vertex, with the vertices and
    /// the edges a switching keeps there.
    depths: Box<[Depth]>,
}

/// One box depth's counts.
#[derive(Clone, Copy, Debug)]
struct Depth {
    /// The `!` vertex of the box, or `NONE` outside every box.
    level: u32,
    /// Its vertices, doors included.
    vertices: u64,
    /// The edges every switching keeps there.
    kept: u64,
}

/// What the construction gathers before it lays the slots out.
struct Builder<'a> {
    /// The structure.
    s: &'a MellStructure,
    /// Per vertex, the other end of its parent edge, or `NONE`.
    parent: Vec<u32>,
    /// Every edge but the parent edges, by its two ends; a parent edge
    /// adds its lower end's slot here.
    extra: Vec<(u32, u32)>,
    /// Per door, its depth and its `?` vertex.
    doors: Vec<(u32, u32)>,
    /// Per box, the `?` vertex whose door it holds last, and that door.
    stamp: Vec<(u32, u32)>,
    /// The caller's bounds.
    limits: &'a Limits,
}

impl Builder<'_> {
    /// Returns the door vertex of the `?` vertex `q` inside box `b`, and
    /// whether it is new. Wires are added a `?` vertex at a time, so a
    /// box's stamp says whether its door for `q` exists.
    fn door(&mut self, q: u32, b: u32) -> Result<(u32, bool), Error> {
        let (stamped, door) = self.stamp[b as usize];
        if stamped == q {
            return Ok((door, false));
        }
        let id = self.s.len() as u64 + self.doors.len() as u64;
        if id >= MOST {
            return Err(Error::Refused(Refusal::Index {
                what: Space::Vertex,
                count: id + 1,
                most: MOST,
            }));
        }
        let bytes = (self.doors.len() as u64 + 1) * DOOR_BYTES + self.extra.len() as u64 * 8;
        afford(bytes, self.limits)?;
        self.doors.push((b, q));
        self.parent.push(NONE);
        self.stamp[b as usize] = (q, id as u32);
        Ok((id as u32, true))
    }

    /// Adds the wire of the `?` vertex `q` to `end`, a copy of it or the
    /// literal it hangs on: an edge at the end's depth from `q` or from its
    /// door there, and up the boxes around the end to `q`'s depth, an edge
    /// from the door or `q` itself to the `!` vertex of each box entered,
    /// unless it is there already.
    fn wire(&mut self, q: u32, end: u32, copy: bool, boxes: &Nesting) -> Result<(), Error> {
        let home = self.s.inside[q as usize];
        let mut current = boxes.of(VertexId::new(end));
        let source = if current == home {
            q
        } else {
            let (door, fresh) = self.door(q, current)?;
            if !fresh {
                // The chain above was laid by an earlier wire of `q`.
                self.attach(door, end, copy);
                return Ok(());
            }
            door
        };
        self.attach(source, end, copy);
        while current != home {
            let bang = current;
            let outer = self.s.inside[bang as usize];
            let (source, fresh) = if outer == home {
                (q, true)
            } else {
                self.door(q, outer)?
            };
            // A copy of `q` that is itself the `!` vertex has this edge as
            // its parent edge already.
            if self.s.parent[bang as usize] != q {
                self.extra.push((source, bang));
            }
            if !fresh {
                break;
            }
            current = outer;
        }
        Ok(())
    }

    /// Adds an edge from a `?` vertex or door to the end of a wire: the
    /// copy's parent edge, or an edge to the literal it hangs on.
    fn attach(&mut self, source: u32, end: u32, copy: bool) {
        if copy {
            self.parent[end as usize] = source;
        } else {
            self.extra.push((source, end));
        }
    }
}

impl Graph {
    /// Builds the graph of a structure whose vertices, links and
    /// weakenings are in place, within `limits.memory_bytes`.
    pub(super) fn new(s: &MellStructure, boxes: &Nesting, limits: &Limits) -> Result<Self, Error> {
        let n = s.len();
        let mut b = Builder {
            s,
            parent: vec![NONE; n],
            extra: Vec::new(),
            doors: Vec::new(),
            stamp: vec![(NONE, NONE); n],
            limits,
        };
        for v in 0..n as u32 {
            let p = s.parent[v as usize];
            if p == NONE {
                continue;
            }
            match s.kind(p) {
                Kind::Tensor | Kind::Par => b.parent[v as usize] = p,
                Kind::Bang | Kind::Quest => {}
                Kind::Atom
                | Kind::DualAtom
                | Kind::One
                | Kind::Bot
                | Kind::Top
                | Kind::Zero
                | Kind::With
                | Kind::Plus => unreachable!("only the structure's connectives have premises"),
            }
        }
        for q in 0..n as u32 {
            if s.kind(q) != Kind::Quest {
                continue;
            }
            for c in s.premises(VertexId::new(q)) {
                b.wire(q, c.get(), true, boxes)?;
            }
            if let Some(l) = s.hangs_on(VertexId::new(q)) {
                b.wire(q, l.get(), false, boxes)?;
            }
        }
        for &(x, y) in s.links() {
            b.extra.push((x.get(), y.get()));
        }
        // A vertex per cut link after the doors, below its two ends.
        let first_cut = n as u64 + b.doors.len() as u64;
        if first_cut + s.cuts.len() as u64 > MOST {
            return Err(Error::Refused(Refusal::Index {
                what: Space::Vertex,
                count: first_cut + s.cuts.len() as u64,
                most: MOST,
            }));
        }
        for (i, &(x, y)) in s.cuts.iter().enumerate() {
            let c = (first_cut + i as u64) as u32;
            b.parent.push(NONE);
            b.parent[x.index()] = c;
            b.parent[y.index()] = c;
        }
        let total = n + b.doors.len() + s.cuts.len();
        for v in 0..total {
            if b.parent[v] != NONE {
                b.extra.push((b.parent[v], v as u32));
            }
        }
        // Slots: one parent slot per vertex and two per other edge.
        let slots = total as u64 + 2 * b.extra.len() as u64
            - (b.parent.iter().filter(|&&p| p != NONE).count() as u64);
        if slots >= u64::from(u32::MAX) {
            return Err(Error::Refused(Refusal::Index {
                what: Space::Vertex,
                count: slots,
                most: u64::from(u32::MAX) - 1,
            }));
        }
        afford(slots * 4 + total as u64 * super::VERTEX_BYTES, limits)?;
        let mut degree = vec![1u32; total];
        for &(x, y) in &b.extra {
            // A parent edge takes its upper end's parent slot.
            if b.parent[y as usize] != x {
                degree[y as usize] += 1;
            }
            degree[x as usize] += 1;
        }
        let mut start = Vec::with_capacity(total + 1);
        let mut sum = 0u32;
        for &d in &degree {
            start.push(sum);
            sum += d;
        }
        start.push(sum);
        let mut to = vec![NONE; sum as usize];
        let mut fill: Vec<u32> = start[..total].iter().map(|&s| s + 1).collect();
        for &(x, y) in &b.extra {
            if b.parent[y as usize] == x {
                to[start[y as usize] as usize] = x;
            } else {
                to[fill[y as usize] as usize] = x;
                fill[y as usize] += 1;
            }
            to[fill[x as usize] as usize] = y;
            fill[x as usize] += 1;
        }
        let mut switched = Vec::with_capacity(total);
        let mut level = Vec::with_capacity(total);
        for v in 0..n as u32 {
            switched.push(matches!(s.kind(v), Kind::Par | Kind::Quest));
            level.push(s.inside[v as usize]);
        }
        let mut door_of = Vec::with_capacity(b.doors.len());
        for &(depth, q) in &b.doors {
            switched.push(true);
            level.push(depth);
            door_of.push(q);
        }
        for &(x, _) in &s.cuts {
            switched.push(false);
            level.push(s.inside[x.index()]);
        }
        let mut graph = Self {
            start: start.into_boxed_slice(),
            to: to.into_boxed_slice(),
            switched: switched.into_boxed_slice(),
            level: level.into_boxed_slice(),
            n: n as u32,
            door_of: door_of.into_boxed_slice(),
            cut_of: s.cuts.iter().map(|&(x, y)| (x.get(), y.get())).collect(),
            depths: Box::default(),
        };
        graph.depths = graph.count(s);
        Ok(graph)
    }

    /// Counts every depth's vertices and the edges a switching keeps
    /// there: every edge, less all but one wire or premise edge of each
    /// switched vertex.
    fn count(&self, s: &MellStructure) -> Box<[Depth]> {
        let n = self.n as usize;
        // Slot of a depth: `n` outside every box, else its `!` vertex.
        let slot = |level: u32| if level == NONE { n } else { level as usize };
        let mut vertices = vec![0u64; n + 1];
        let mut ends = vec![0u64; n + 1];
        let mut dropped = vec![0u64; n + 1];
        for v in 0..self.len() {
            let d = slot(self.level[v]);
            vertices[d] += 1;
            let present = self.slots(v as u32).filter(|&i| self.to[i] != NONE).count() as u64;
            ends[d] += present;
            if self.switched[v] {
                let class = self.slots(v as u32).skip(1).count() as u64;
                dropped[d] += class.saturating_sub(1);
            }
        }
        // Each edge has two ends, and the classes of the switched vertices
        // are disjoint sets of edges, so fewer are dropped than there are.
        let kept = |d: usize| ends[d] / 2 - dropped[d];
        let mut depths = vec![Depth {
            level: NONE,
            vertices: vertices[n],
            kept: kept(n),
        }];
        depths.extend(s.boxes().map(|d| Depth {
            level: d.get(),
            vertices: vertices[d.index()],
            kept: kept(d.index()),
        }));
        depths.into_boxed_slice()
    }

    /// Returns the number of vertices, doors included.
    fn len(&self) -> usize {
        self.switched.len()
    }

    /// Returns the number of vertices, doors included.
    pub(super) fn vertex_count(&self) -> usize {
        self.len()
    }

    /// Returns the first door vertex: the structure's vertex count.
    pub(super) fn vertex_offset(&self) -> u32 {
        self.n
    }

    /// Returns the other ends of a vertex's edges but its parent edge: of
    /// a `?` vertex or door its wires.
    pub(super) fn wires(&self, v: u32) -> impl Iterator<Item = u32> + '_ {
        self.slots(v).skip(1).map(|s| self.to[s])
    }

    /// Returns the other end of a vertex's parent edge, or `NONE`.
    pub(super) fn parent_edge(&self, v: u32) -> u32 {
        self.to[self.start[v as usize] as usize]
    }

    /// Returns the door vertices inside the box of `bang`.
    pub(super) fn doors_in(&self, bang: u32) -> impl Iterator<Item = u32> + '_ {
        (self.n..self.first_cut()).filter(move |&g| self.level[g as usize] == bang)
    }

    /// Returns the cut vertices at the depth of `level`, a `!` vertex or
    /// `NONE` outside every box.
    pub(super) fn cuts_in(&self, level: u32) -> impl Iterator<Item = u32> + '_ {
        (self.first_cut()..self.len() as u32).filter(move |&g| self.level[g as usize] == level)
    }

    /// Returns the first cut vertex: the doors end there.
    pub(super) fn first_cut(&self) -> u32 {
        (self.len() - self.cut_of.len()) as u32
    }

    /// Returns the two ends of a cut vertex, the one in a cut formula's
    /// tree first.
    pub(super) fn cut_ends(&self, c: u32) -> (u32, u32) {
        self.cut_of[(c - self.first_cut()) as usize]
    }

    /// Returns the slots of a vertex.
    fn slots(&self, v: u32) -> std::ops::Range<usize> {
        self.start[v as usize] as usize..self.start[v as usize + 1] as usize
    }

    /// Returns the `?` vertices with a door in the box of `bang`.
    pub(super) fn doors(&self, bang: VertexId) -> Vec<VertexId> {
        let n = self.n as usize;
        let mut doors: Vec<VertexId> = (n..self.first_cut() as usize)
            .filter(|&g| self.level[g] == bang.get())
            .map(|g| VertexId::new(self.door_of[g - n]))
            .collect();
        doors.sort_unstable();
        doors
    }

    /// Returns the structure vertex a graph vertex stands for: itself, a
    /// door's `?` vertex, or a cut vertex's end in a cut formula's tree.
    fn named(&self, g: u32) -> VertexId {
        if g < self.n {
            VertexId::new(g)
        } else if g < self.first_cut() {
            VertexId::new(self.door_of[(g - self.n) as usize])
        } else {
            VertexId::new(self.cut_ends(g).0)
        }
    }

    /// Returns the error of a depth: itself outside every box, else wrapped
    /// with the box's `!` vertex.
    fn at(level: u32, error: NetError) -> NetError {
        if level == NONE {
            error
        } else {
            NetError::InBox {
                door: VertexId::new(level),
                error: Box::new(error),
            }
        }
    }

    /// Decides the criterion on a complete structure: no switching cycle
    /// at any depth, and with `mix` false every depth connected under every
    /// switching, which given acyclicity is its kept edges being one fewer
    /// than its vertices.
    pub(super) fn correct(
        &self,
        mix: bool,
        mut stop: impl FnMut(Progress) -> bool,
    ) -> Result<(), NetError> {
        let mut scratch = Scratch::new(self);
        let held_bytes = self.len() as u64 * super::SCRATCH_BYTES;
        let mut rounds = 0;
        let mut stop = |round: u64| {
            rounds += round;
            stop(Progress {
                held_bytes,
                ..Progress::new(Phase::Net, round, rounds)
            })
        };
        match self.acyclic(&mut scratch, &mut stop) {
            None => return Err(STOPPED),
            Some(false) => {
                let Some(cycle) = self.cycle(&mut scratch, &mut stop) else {
                    return Err(STOPPED);
                };
                let level = self.level[cycle[0] as usize];
                // A cut vertex stands between its two ends, which the
                // cycle names one after the other.
                let first_cut = self.first_cut();
                let cycle = cycle
                    .into_iter()
                    .filter(|&g| g < first_cut)
                    .map(|g| self.named(g))
                    .collect();
                return Err(Self::at(level, NetError::SwitchingCycle { cycle }));
            }
            Some(true) => {}
        }
        if mix {
            return Ok(());
        }
        for depth in &self.depths {
            if depth.kept + 1 != depth.vertices {
                let parts = self.parts(&mut scratch, depth.level);
                return Err(Self::at(depth.level, NetError::Disconnected { parts }));
            }
        }
        Ok(())
    }

    /// Searches depth-first from every vertex not deleted, over the edges
    /// not removed, computing for every vertex its discovery time, its
    /// search-tree parent, the root that reached it, and whether its
    /// parent edge is a bridge.
    pub(super) fn search(
        &self,
        scratch: &mut Scratch,
        roots: impl IntoIterator<Item = u32>,
    ) -> u32 {
        scratch.disc.fill(NONE);
        let mut time = 0;
        let mut trees = 0;
        for r in roots {
            if scratch.deleted[r as usize] || scratch.disc[r as usize] != NONE {
                continue;
            }
            trees += 1;
            self.discover(scratch, r, NONE, r, &mut time);
            while let Some(&(v, next)) = scratch.stack.last() {
                if next < self.start[v as usize + 1] {
                    scratch.stack.last_mut().unwrap().1 = next + 1;
                    let w = self.to[next as usize];
                    if w == NONE
                        || scratch.removed[next as usize]
                        || scratch.deleted[w as usize]
                        || w == scratch.dfs_parent[v as usize]
                    {
                        continue;
                    }
                    if scratch.disc[w as usize] == NONE {
                        self.discover(scratch, w, v, r, &mut time);
                    } else {
                        let (lv, dw) = (scratch.low[v as usize], scratch.disc[w as usize]);
                        scratch.low[v as usize] = lv.min(dw);
                    }
                } else {
                    scratch.stack.pop();
                    let p = scratch.dfs_parent[v as usize];
                    if p != NONE {
                        let (lp, lv) = (scratch.low[p as usize], scratch.low[v as usize]);
                        scratch.low[p as usize] = lp.min(lv);
                        scratch.bridge[v as usize] = lv > scratch.disc[p as usize];
                    }
                }
            }
        }
        trees
    }

    /// Enters a vertex in the search: from `parent`, in the tree of `root`.
    fn discover(&self, scratch: &mut Scratch, v: u32, parent: u32, root: u32, time: &mut u32) {
        scratch.disc[v as usize] = *time;
        scratch.low[v as usize] = *time;
        *time += 1;
        scratch.dfs_parent[v as usize] = parent;
        scratch.component[v as usize] = root;
        scratch.bridge[v as usize] = false;
        scratch.stack.push((v, self.start[v as usize]));
    }

    /// Returns whether the slot's edge is there: its other end exists and
    /// is not deleted, and the edge is not removed.
    fn present(&self, scratch: &Scratch, slot: usize) -> bool {
        let w = self.to[slot];
        w != NONE && !scratch.removed[slot] && !scratch.deleted[w as usize]
    }

    /// Returns whether a vertex may be deleted according to the last
    /// search: it is switched and its parent edge is absent or a bridge, or
    /// it is not and all its edges are bridges.
    fn deletable(&self, scratch: &Scratch, v: u32) -> bool {
        let mut slots = self.slots(v);
        if self.switched[v as usize] {
            slots.end = slots.start + 1;
        }
        slots.all(|i| !self.present(scratch, i) || scratch.is_bridge(v, self.to[i]))
    }

    /// Runs the deletion procedure on the vertices not yet deleted, asking
    /// `stop` before each round. Returns whether every vertex went;
    /// otherwise a switching cycle runs among those left. `None` when
    /// `stop` fired.
    fn acyclic(&self, scratch: &mut Scratch, stop: &mut dyn FnMut(u64) -> bool) -> Option<bool> {
        let n = self.len();
        let mut remaining = scratch.deleted.iter().filter(|&&d| !d).count();
        while remaining > 0 {
            if stop(1) {
                return None;
            }
            self.search(scratch, 0..n as u32);
            let before = remaining;
            for v in 0..n as u32 {
                if !scratch.deleted[v as usize] && self.deletable(scratch, v) {
                    scratch.deleted[v as usize] = true;
                    remaining -= 1;
                }
            }
            if remaining == before {
                return Some(false);
            }
        }
        Some(true)
    }

    /// Returns the slot at `w` of the edge to `v`.
    fn twin(&self, v: u32, w: u32) -> usize {
        self.slots(w)
            .find(|&s| self.to[s] == v)
            .expect("the edge exists at both ends")
    }

    /// Isolates one switching cycle among the vertices
    /// [`acyclic`](Self::acyclic) got stuck on, which it must have just
    /// done, and returns its vertices in order along it: every edge among
    /// them is taken out in turn and kept out when a cycle survives
    /// without it, which leaves exactly one. `None` when `stop` fired.
    fn cycle(&self, scratch: &mut Scratch, stop: &mut dyn FnMut(u64) -> bool) -> Option<Vec<u32>> {
        let n = self.len();
        scratch.stuck.clone_from(&scratch.deleted);
        for v in 0..n as u32 {
            for s in self.slots(v) {
                let w = self.to[s];
                if w == NONE
                    || w < v
                    || scratch.removed[s]
                    || scratch.stuck[v as usize]
                    || scratch.stuck[w as usize]
                {
                    continue;
                }
                let t = self.twin(v, w);
                scratch.removed[s] = true;
                scratch.removed[t] = true;
                scratch.deleted.clone_from(&scratch.stuck);
                let acyclic = self.acyclic(scratch, stop)?;
                if acyclic {
                    scratch.removed[s] = false;
                    scratch.removed[t] = false;
                } else {
                    scratch.stuck.clone_from(&scratch.deleted);
                }
            }
        }
        let first = (0..n as u32)
            .find(|&v| !scratch.stuck[v as usize])
            .expect("a switching cycle is left");
        let mut cycle = Vec::new();
        let (mut previous, mut current) = (NONE, first);
        loop {
            cycle.push(current);
            let next = self
                .slots(current)
                .map(|s| (s, self.to[s]))
                .find(|&(s, w)| {
                    w != NONE && w != previous && !scratch.removed[s] && !scratch.stuck[w as usize]
                })
                .map(|(_, w)| w)
                .expect("every vertex of the cycle has two edges on it");
            if next == first {
                break;
            }
            (previous, current) = (current, next);
        }
        Some(cycle)
    }

    /// Returns the connected components of one depth under the switching
    /// that keeps the first wire or premise edge of every switched vertex,
    /// each as the vertices in it without a parent edge there: the
    /// depth's conclusions and the wires and premises the switching drops,
    /// doors as their `?` vertex. Components come in the order of their
    /// smallest vertex.
    fn parts(&self, scratch: &mut Scratch, level: u32) -> Vec<Vec<VertexId>> {
        let n = self.len();
        scratch.deleted.fill(false);
        scratch.removed.fill(false);
        for v in 0..n as u32 {
            if self.switched[v as usize] {
                for s in self.slots(v).skip(2) {
                    let w = self.to[s];
                    scratch.removed[s] = true;
                    scratch.removed[self.twin(v, w)] = true;
                }
            }
        }
        let at = (0..n as u32).filter(|&v| self.level[v as usize] == level);
        self.search(scratch, at.clone());
        // Per component's root, the index of its part, or `NONE`.
        let mut index = vec![NONE; n];
        let mut parts: Vec<(u32, Vec<VertexId>)> = Vec::new();
        for v in at {
            let first = self.start[v as usize] as usize;
            if self.to[first] != NONE && !scratch.removed[first] {
                continue;
            }
            let label = scratch.component[v as usize];
            if index[label as usize] == NONE {
                index[label as usize] = parts.len() as u32;
                parts.push((label, Vec::new()));
            }
            parts[index[label as usize] as usize].1.push(self.named(v));
        }
        scratch.removed.fill(false);
        parts.sort_by_key(|(label, _)| *label);
        parts.into_iter().map(|(_, part)| part).collect()
    }
}

/// Working memory for the criterion's tests and the sequentialization's
/// searches on one graph.
pub(super) struct Scratch {
    /// The vertices the deletion procedure has removed.
    deleted: Vec<bool>,
    /// The vertices deleted when the procedure got stuck, kept while a
    /// cycle is isolated among the others.
    stuck: Vec<bool>,
    /// Per slot, whether the edge is taken out of the graph.
    removed: Vec<bool>,
    /// Per vertex, its discovery time in the last search, or `NONE`.
    disc: Vec<u32>,
    /// Per vertex, the least discovery time it reaches through its
    /// search-tree descendants and one back edge.
    low: Vec<u32>,
    /// Per vertex, its parent in the search tree, or `NONE`.
    dfs_parent: Vec<u32>,
    /// Per vertex, whether the edge to its search-tree parent is a bridge.
    bridge: Vec<bool>,
    /// Per vertex, the root of the search tree that reached it.
    component: Vec<u32>,
    /// The search stack: a vertex and its next slot.
    stack: Vec<(u32, u32)>,
}

impl Scratch {
    /// Returns working memory for a graph.
    pub(super) fn new(graph: &Graph) -> Self {
        let n = graph.len();
        Self {
            deleted: vec![false; n],
            stuck: vec![false; n],
            removed: vec![false; graph.to.len()],
            disc: vec![NONE; n],
            low: vec![NONE; n],
            dfs_parent: vec![NONE; n],
            bridge: vec![false; n],
            component: vec![NONE; n],
            stack: Vec::with_capacity(n),
        }
    }

    /// Takes a vertex out of the graph.
    pub(super) fn delete(&mut self, v: u32) {
        self.deleted[v as usize] = true;
    }

    /// Returns whether the last search reached the vertex.
    pub(super) fn reached(&self, v: u32) -> bool {
        self.disc[v as usize] != NONE
    }

    /// Returns the root of the search tree that reached the vertex in the
    /// last search.
    pub(super) fn component(&self, v: u32) -> u32 {
        self.component[v as usize]
    }

    /// Returns whether the edge between two adjacent vertices is a bridge
    /// according to the last search, which reached both.
    pub(super) fn is_bridge(&self, v: u32, w: u32) -> bool {
        (self.dfs_parent[w as usize] == v && self.bridge[w as usize])
            || (self.dfs_parent[v as usize] == w && self.bridge[v as usize])
    }
}
