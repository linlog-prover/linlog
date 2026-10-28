// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Proof nets of unit-free MELL with `!`-boxes: proof structures whose
//! vertices are instances of the forest's occurrences, the correctness
//! criterion, and the conversions to and from proof terms.
//!
//! A [`MellStructure`] holds a vertex for every root formula and below
//! each `⊗`, `⅋` and `!` vertex one for each premise, as the forest's
//! trees do; below a `?` vertex it holds any number of *copies* of its
//! subformula, the generalised `?` node, which stands for dereliction
//! (one copy), contraction (several) and weakening (none) at once and
//! keeps neither their order nor the boxes they happen in. Each `!` vertex
//! opens a box: its premise and everything above it, up to the copies of
//! `?` vertices outside, lies in it, and a copy may lie in any box inside
//! its `?` vertex's own, the wire to it crossing each box border on the
//! way as an auxiliary door. A `?` vertex without copies hangs on an axiom
//! link inside its box, as a weakening formula is a conclusion of an axiom
//! (Guerrini and Masini, *Parsing MELL proof nets*, TCS 254, 2001): that
//! keeps every proof net connected, so that the Danos–Regnier criterion
//! decides it, which the count of components cannot when weakenings hang
//! free.

// A variant a later step adds must not fall into an existing arm.
#![deny(clippy::wildcard_enum_match_arm)]

/// From a proof term to its structure.
mod desequentialize;
/// The structure graph of every box depth with boxes contracted, and the
/// criterion's tests on it.
mod graph;
/// From a proof net to a proof term.
mod sequentialize;
#[cfg(all(test, feature = "parse"))]
pub(in crate::nets) mod tests;

use super::{Criterion, NONE, NetError, VertexId, afford};
use crate::Error;
use crate::fragment::{Fragment, Mode, Order};
use crate::limits::{Limits, Progress, Refusal, Space};
use crate::occurrences::{Forest, OccId, Reading, ShapeError};
use crate::sequents::Kind;
use graph::Graph;

/// The most vertices a structure has, the door vertices of its graph
/// included: a vertex has at most as many slots in the graph as it has
/// edges, every edge two, and the slots' offsets are `u32`, so the
/// structure refuses a graph past `u32::MAX` slots too.
const MOST: u64 = (u32::MAX / 2 - 1) as u64;

/// The bytes a structure holds per vertex beside its forest: the
/// occurrence, the parent, the box, the partner and the hanging (20), the
/// children's offset and slot (8), and the graph's level, offset, two
/// slots and flags (16).
const VERTEX_BYTES: u64 = 20 + 8 + 16;

/// The bytes the construction holds per vertex while it builds the
/// structure, beside it: the premises' counts, offsets and fill (16), the
/// box nesting's depths, children and intervals (40), and the graph's
/// builder: a parent, a stamp, a degree, a fill and two edges (40).
const BUILD_BYTES: u64 = 16 + 40 + 40;

/// The bytes a door vertex of the graph holds: its level, its `?` vertex,
/// its offset and slots (16).
const DOOR_BYTES: u64 = 16;

/// The bytes the criterion's working memory takes per vertex of the
/// graph: the search's four arrays (16), its bridge flag and the deleted
/// sets (2), a removed flag per slot (2) and the stack (8).
const SCRATCH_BYTES: u64 = 16 + 2 + 2 + 8;

/// A vertex of a [`MellStructure`] other than a root, as
/// [`MellStructure::from_parts`] takes it: the occurrence it stands for,
/// the vertex it is a premise of, or none for an end of a cut link, and
/// the `!` vertex of the innermost box it lies in, or `None` outside every
/// box.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Placed {
    /// The occurrence the vertex stands for.
    pub occurrence: OccId,
    /// The vertex it is a premise of: a `⊗` or `⅋` vertex for its left or
    /// right subformula, a `!` vertex for its subformula, or a `?` vertex
    /// for a copy of its subformula; `None` for an end of a cut link, an
    /// occurrence of a cut formula's tree or its dual.
    pub parent: Option<VertexId>,
    /// The `!` vertex whose box is the innermost one around the vertex, or
    /// `None` outside every box.
    pub inside: Option<VertexId>,
}

impl Placed {
    /// Returns the vertex of `occurrence` above `parent`, inside the box
    /// of `inside`.
    pub const fn new(occurrence: OccId, parent: VertexId, inside: Option<VertexId>) -> Self {
        Self {
            occurrence,
            parent: Some(parent),
            inside,
        }
    }

    /// Returns the vertex of `occurrence` at an end of a cut link, inside
    /// the box of `inside`.
    pub const fn cut(occurrence: OccId, inside: Option<VertexId>) -> Self {
        Self {
            occurrence,
            parent: None,
            inside,
        }
    }
}

/// The parts of a [`MellStructure`] beside its forest and criterion, as
/// [`MellStructure::from_parts`] takes them and
/// [`MellStructure::parts`] returns them. The root formulas are vertices
/// `0` to `r − 1`, in the forest's order; `vertices[i]` is vertex `r + i`.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Parts {
    /// The vertices after the roots.
    pub vertices: Vec<Placed>,
    /// The cut links, each between two vertices without a parent in one
    /// box, an occurrence of a cut formula's tree and its dual.
    pub cuts: Vec<(VertexId, VertexId)>,
    /// The axiom links, each between two dual literal vertices of one box.
    pub links: Vec<(VertexId, VertexId)>,
    /// The weakenings: a `?` vertex without copies and a literal vertex
    /// in its box or a box inside it, on whose axiom link it hangs.
    pub weakenings: Vec<(VertexId, VertexId)>,
}

/// Why a list of vertices, cut links, links and weakenings is not a
/// structure with boxes over its forest, or a list of cut links and links
/// no proof structure over its forest, which [`NetError::Placement`]
/// names with the vertex at fault.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Placement {
    /// The vertex names an occurrence outside the forest, or a parent or
    /// a box that is no vertex of the structure.
    Outside,
    /// The vertex's occurrence is no premise of its parent's: not the left
    /// or right subformula of a `⊗` or `⅋`, not the subformula of a `!` or
    /// a `?`.
    Premise,
    /// The vertex, a `⊗`, `⅋` or `!`, has a premise missing or twice.
    Premises,
    /// The vertex lies in another box than its parent allows: a root or a
    /// premise of a `⊗` or `⅋` lies where its parent does, the premise of a
    /// `!` in its box, a copy in its `?` vertex's box or one inside it; or
    /// the box it names is no `!` vertex's.
    Box,
    /// The boxes the vertex's box lies in come round to it again.
    Nesting,
    /// The link from this literal leaves its box.
    Link,
    /// The weakening of this vertex is none: the vertex is no `?` without
    /// copies, or hangs twice, or its literal is none, or lies outside its
    /// box.
    Weakening,
    /// The cut link at this vertex is none: it does not join an occurrence
    /// of a cut formula's tree to its dual, below neither of which lies a
    /// premise, in one box; or the vertex is the end of two cut links, or
    /// of none where it is no premise.
    Cut,
    /// The vertex lies in a cut formula's tree above no cut link, so it is
    /// no vertex of the structure, and a link names it.
    Uncut,
}

impl Placement {
    /// Returns what is wrong with the vertex, as a clause after its name.
    pub(crate) const fn clause(self) -> &'static str {
        match self {
            Self::Outside => "names an occurrence, a parent or a box outside the structure",
            Self::Premise => "stands for no premise of its parent",
            Self::Premises => "has a premise missing or twice",
            Self::Box => "lies in another box than its parent allows",
            Self::Nesting => "lies in boxes that nest in a cycle",
            Self::Link => "is linked out of its box",
            Self::Weakening => {
                "is no weakening: no ? without copies hanging once on a literal of its box"
            }
            Self::Cut => {
                "is no end of one cut link between a cut formula's occurrence and its dual in one \
                 box"
            }
            Self::Uncut => "lies in a cut formula above every cut link, and is linked",
        }
    }
}

/// A proof structure of unit-free MELL over the occurrence forest of its
/// sequent, with `!`-boxes and generalised `?` vertices (the module's
/// documentation says what they are), and the [`Criterion`] that decides
/// what [`is_correct`](Self::is_correct) requires. Vertices are numbered
/// from the root formulas, `0` to `r − 1` in the forest's order; vertex
/// `v` stands for occurrence [`occurrence(v)`](Self::occurrence), and a
/// box is named by its `!` vertex. Over a sequent with cut formulas, the
/// vertices without a parent but the roots are the ends of *cut links*,
/// each joining an instance of a cut formula's occurrence to one of its
/// dual in one box, which the criterion takes as a `⊗` without a
/// conclusion and [`cut`](crate::nets::cut) eliminates. A structure of an
/// MLL sequent has one
/// vertex per occurrence and no box.
///
/// The structure is built whole, from its [`Parts`] or from a proof term,
/// and stays as it is. A term names occurrences, not vertices, so it fixes
/// a net only up to which axiom link a weakening hangs on and, where a
/// sequent of the term holds two instances of one occurrence, which of
/// them a rule acts on (and so which of two `?` vertices of one occurrence
/// a copy belongs to): reading a term back may give another proof net of
/// the same sequent than the one it came from, and gives the same one up
/// to the weakenings' links when no occurrence has two vertices.
#[derive(Clone, Debug)]
pub struct MellStructure {
    /// The forest of the sequent.
    forest: Forest,
    /// The rules the structure is checked under.
    criterion: Criterion,
    /// Per vertex, the occurrence it stands for.
    occurrence: Box<[OccId]>,
    /// Per vertex, the vertex it is a premise of, or `NONE` for a root.
    parent: Box<[u32]>,
    /// Per vertex, the `!` vertex of its innermost box, or `NONE`.
    inside: Box<[u32]>,
    /// Per vertex, where its premises start in `premises`; one more entry
    /// holds the total, below the vertex count.
    start: Box<[u32]>,
    /// The premises of every vertex: of a `⊗` or `⅋` the left one first,
    /// of a `?` its copies in the order they were given.
    premises: Box<[u32]>,
    /// Per vertex, the literal it is linked to, or `NONE`.
    partner: Box<[u32]>,
    /// The links, as given.
    links: Vec<(VertexId, VertexId)>,
    /// The cut links, each from its end in a cut formula's tree, in the
    /// order given.
    cuts: Vec<(VertexId, VertexId)>,
    /// Per vertex, the literal a weakening hangs on, or `NONE`.
    hangs: Box<[u32]>,
    /// The graph of every box depth with boxes contracted.
    graph: Graph,
}

impl MellStructure {
    /// Checks that structures with boxes exist for a sequent of `fragment`
    /// in `mode`: unit-free MELL in classical mode, with or without Mix,
    /// and in intuitionistic mode, where the net is the one of the
    /// one-sided sequent and the criterion without Mix decides it (every
    /// sequent of a classical proof of an intuitionistic sequent without
    /// units holds one formula in output position). Affine modes are left
    /// out: a proof there weakens formulas that are no `?`, which a
    /// structure has no vertex for.
    ///
    /// # Errors
    ///
    /// [`NetError::NotMell`] for any other fragment or mode.
    pub fn exist(fragment: Fragment, mode: Mode) -> Result<(), NetError> {
        let unit_free = Fragment::MLL.union(Fragment::EXPONENTIALS);
        if unit_free.contains(fragment) && !mode.is_affine() && !mode.is_ordered() {
            Ok(())
        } else {
            Err(NetError::NotMell { fragment, mode })
        }
    }

    /// Returns the structure of `parts` over the forest, validated: every
    /// vertex in its place in the formula trees and the boxes, every
    /// vertex without a parent but the roots at one end of one cut link,
    /// which joins an occurrence of a cut formula's tree to its dual in one
    /// box, every link
    /// between two dual literals of one box, every weakening a `?` vertex
    /// without copies on a literal of its box or one inside it. Its graph is
    /// held within `limits.memory_bytes`. Whether it is a proof net is
    /// [`is_correct`](Self::is_correct)'s question; a literal without a
    /// link and a weakening that hangs on nothing are allowed here.
    ///
    /// # Errors
    ///
    /// [`NetError::NotMell`] for a sequent outside unit-free MELL;
    /// [`NetError::Mode`] for a criterion of the cyclic order, which keeps
    /// no exponentials; [`NetError::Placement`] for a vertex out of place;
    /// [`NetError::NoVertex`], [`NetError::NotLiteral`],
    /// [`NetError::NotDual`] and [`NetError::LinkedTwice`] for a link that
    /// is none; [`Refusal::Index`] past the vertices a structure holds and
    /// [`NetError::Refused`] past the memory bound.
    pub fn from_parts(
        forest: Forest,
        criterion: Criterion,
        parts: &Parts,
        limits: &Limits,
    ) -> Result<Self, Error> {
        let mode = if criterion.mix {
            Mode::CLASSICAL.with_mix()
        } else {
            Mode::CLASSICAL
        };
        Self::exist(forest.sequent().fragment(), mode)?;
        if criterion.order != Order::Commutative {
            return Err(NetError::Mode { mode: Mode::CYCLIC }.into());
        }
        let roots = forest.roots().len();
        let n = roots as u64 + parts.vertices.len() as u64;
        if n > MOST {
            return Err(Error::Refused(Refusal::Index {
                what: Space::Vertex,
                count: n,
                most: MOST,
            }));
        }
        afford(n * (VERTEX_BYTES + BUILD_BYTES), limits)?;
        let n = n as usize;
        let mut occurrence = Vec::with_capacity(n);
        let mut parent = Vec::with_capacity(n);
        let mut inside = Vec::with_capacity(n);
        occurrence.extend(forest.roots().iter().copied());
        parent.resize(roots, NONE);
        inside.resize(roots, NONE);
        for (i, placed) in parts.vertices.iter().enumerate() {
            let v = VertexId::new((roots + i) as u32);
            let outside = |w: VertexId| w.index() >= n;
            if placed.occurrence.index() >= forest.len()
                || placed.parent.is_some_and(outside)
                || placed.inside.is_some_and(outside)
            {
                return Err(misplaced(v, Placement::Outside));
            }
            occurrence.push(placed.occurrence);
            parent.push(placed.parent.map_or(NONE, VertexId::get));
            inside.push(placed.inside.map_or(NONE, VertexId::get));
        }
        let (start, premises) = premises(&forest, &occurrence, &parent)?;
        let cuts = place_cuts(&forest, &occurrence, &parent, &inside, roots, &parts.cuts)?;
        check_boxes(&forest, &occurrence, &parent, &inside, roots)?;
        let mut structure = Self {
            criterion,
            occurrence: occurrence.into_boxed_slice(),
            parent: parent.into_boxed_slice(),
            inside: inside.into_boxed_slice(),
            start,
            premises,
            partner: vec![NONE; n].into_boxed_slice(),
            links: Vec::with_capacity(parts.links.len()),
            cuts,
            hangs: vec![NONE; n].into_boxed_slice(),
            graph: Graph::default(),
            forest,
        };
        let boxes = Nesting::new(&structure);
        structure.check_copies(&boxes)?;
        for &(x, y) in &parts.links {
            structure.link(x, y, &boxes)?;
        }
        for &(q, l) in &parts.weakenings {
            structure.hang(q, l, &boxes)?;
        }
        structure.graph = Graph::new(&structure, &boxes, limits)?;
        Ok(structure)
    }

    /// Links two dual literals of one box, refusing what is no link.
    fn link(&mut self, x: VertexId, y: VertexId, boxes: &Nesting) -> Result<(), NetError> {
        let n = self.len();
        for v in [x, y] {
            if v.index() >= n {
                return Err(NetError::NoVertex {
                    vertex: v.get(),
                    vertices: n as u32,
                });
            }
            if !self.forest.is_literal(self.occurrence[v.index()]) {
                return Err(NetError::NotLiteral { vertex: v });
            }
        }
        let (ox, oy) = (self.occurrence[x.index()], self.occurrence[y.index()]);
        if !self.forest.dual_literals(ox, oy) {
            return Err(NetError::NotDual { x, y });
        }
        for v in [x, y] {
            if self.partner[v.index()] != NONE {
                return Err(NetError::LinkedTwice { vertex: v });
            }
        }
        if boxes.of(x) != boxes.of(y) {
            return Err(NetError::Placement {
                vertex: x,
                fault: Placement::Link,
            });
        }
        self.partner[x.index()] = y.get();
        self.partner[y.index()] = x.get();
        self.links.push((x, y));
        Ok(())
    }

    /// Hangs the weakening `q` on the literal `l`, refusing what is none.
    fn hang(&mut self, q: VertexId, l: VertexId, boxes: &Nesting) -> Result<(), NetError> {
        let n = self.len();
        let fault = NetError::Placement {
            vertex: q,
            fault: Placement::Weakening,
        };
        if q.index() >= n || l.index() >= n {
            return Err(fault);
        }
        let weakening = self.forest.kind(self.occurrence[q.index()]) == Kind::Quest
            && self.premises(q).next().is_none()
            && self.hangs[q.index()] == NONE;
        let literal = self.forest.is_literal(self.occurrence[l.index()]);
        if !weakening || !literal || !boxes.within(boxes.of(l), boxes.of(q)) {
            return Err(fault);
        }
        self.hangs[q.index()] = l.get();
        Ok(())
    }

    /// Returns the forest of the sequent.
    pub fn forest(&self) -> &Forest {
        &self.forest
    }

    /// Returns the rules the structure is checked under.
    pub fn criterion(&self) -> Criterion {
        self.criterion
    }

    /// Returns the intuitionistic reading of the structure's sequent: the
    /// side of `⊢` every occurrence, and so every vertex, stands on. It is
    /// recomputed on each call, in time linear in the forest.
    ///
    /// # Errors
    ///
    /// The [`ShapeError`] of a sequent with no intuitionistic reading.
    pub fn reading(&self) -> Result<Reading<'_>, ShapeError> {
        Reading::new(&self.forest)
    }

    /// Returns the number of vertices.
    pub fn len(&self) -> usize {
        self.occurrence.len()
    }

    /// Returns whether the structure has no vertex, which only the empty
    /// sequent gives.
    pub fn is_empty(&self) -> bool {
        self.occurrence.is_empty()
    }

    /// Returns every vertex, in order.
    pub fn vertices(&self) -> impl ExactSizeIterator<Item = VertexId> {
        (0..self.len() as u32).map(VertexId::new)
    }

    /// Returns the occurrence a vertex stands for, `None` for a vertex
    /// outside the structure.
    pub fn occurrence(&self, v: VertexId) -> Option<OccId> {
        self.occurrence.get(v.index()).copied()
    }

    /// Returns the occurrence of every vertex, in vertex order.
    pub(crate) fn occurrences(&self) -> &[OccId] {
        &self.occurrence
    }

    /// Returns the vertex a vertex is a premise of, `None` for a root, an
    /// end of a cut link and a vertex outside the structure.
    pub fn parent(&self, v: VertexId) -> Option<VertexId> {
        some(*self.parent.get(v.index())?)
    }

    /// Returns the premises of a vertex: of a `⊗` or `⅋` its left and its
    /// right one, of a `!` its subformula's, of a `?` its copies; none for
    /// a literal and for a vertex outside the structure.
    pub fn premises(&self, v: VertexId) -> impl Iterator<Item = VertexId> + '_ {
        let range = if v.index() < self.len() {
            self.start[v.index()] as usize..self.start[v.index() + 1] as usize
        } else {
            0..0
        };
        self.premises[range].iter().map(|&w| VertexId::new(w))
    }

    /// Returns the `!` vertex of the innermost box a vertex lies in, `None`
    /// outside every box and for a vertex outside the structure.
    pub fn inside(&self, v: VertexId) -> Option<VertexId> {
        some(*self.inside.get(v.index())?)
    }

    /// Returns the `!` vertices, each the principal door of its box, in
    /// order.
    pub fn boxes(&self) -> impl Iterator<Item = VertexId> + '_ {
        self.vertices()
            .filter(|&v| self.forest.kind(self.occurrence[v.index()]) == Kind::Bang)
    }

    /// Returns the `?` vertices outside the box of the `!` vertex `door`
    /// with a copy or a weakening inside it, each of which has an
    /// auxiliary door on its border, in order; none for a vertex that is
    /// no `!`.
    pub fn auxiliary_doors(&self, door: VertexId) -> Vec<VertexId> {
        self.graph.doors(door)
    }

    /// Returns the literal a vertex is linked to, or `None`.
    pub fn partner(&self, v: VertexId) -> Option<VertexId> {
        some(*self.partner.get(v.index())?)
    }

    /// Returns the links, as given.
    pub fn links(&self) -> &[(VertexId, VertexId)] {
        &self.links
    }

    /// Returns the cut links, each from its end in a cut formula's tree to
    /// its dual end, in the order given.
    pub fn cuts(&self) -> &[(VertexId, VertexId)] {
        &self.cuts
    }

    /// Returns the other end of the cut link at a vertex, `None` for a
    /// vertex at no cut link.
    pub fn cut_partner(&self, v: VertexId) -> Option<VertexId> {
        self.cuts.iter().find_map(|&(x, y)| match v {
            v if v == x => Some(y),
            v if v == y => Some(x),
            _ => None,
        })
    }

    /// Returns whether the structure has no cut link.
    pub fn is_cut_free(&self) -> bool {
        self.cuts.is_empty()
    }

    /// Returns the literal whose axiom link the weakening `v` hangs on, or
    /// `None`.
    pub fn hangs_on(&self, v: VertexId) -> Option<VertexId> {
        some(*self.hangs.get(v.index())?)
    }

    /// Returns the structure's parts, from which
    /// [`from_parts`](Self::from_parts) builds it again.
    pub fn parts(&self) -> Parts {
        let roots = self.forest.roots().len();
        let vertices = (roots..self.len())
            .map(|v| Placed {
                occurrence: self.occurrence[v],
                parent: some(self.parent[v]),
                inside: some(self.inside[v]),
            })
            .collect();
        let weakenings = self
            .vertices()
            .filter_map(|q| Some((q, self.hangs_on(q)?)))
            .collect();
        Parts {
            vertices,
            cuts: self.cuts.clone(),
            links: self.links.clone(),
            weakenings,
        }
    }

    /// Returns the kind of a vertex's occurrence.
    fn kind(&self, v: u32) -> Kind {
        self.forest.kind(self.occurrence[v as usize])
    }

    /// Decides whether the structure is a proof net, independently of any
    /// search and of the proof checker: every literal is linked, every
    /// weakening hangs on a link, and at every box depth (outside every
    /// box, and inside each box), with the boxes inside contracted to
    /// their `!` vertex and the `?` vertices outside it to their auxiliary
    /// doors, no cycle survives any switching and, unless the criterion
    /// allows Mix, every switching is connected. A switching keeps one
    /// premise edge of every `⅋`, one wire of every `?` vertex and door.
    /// It holds working memory linear in the graph and asks `stop` once
    /// per round of the deletion procedure, of which a witness for a cycle
    /// runs one per edge.
    ///
    /// # Errors
    ///
    /// [`NetError::Empty`], [`NetError::Unlinked`],
    /// [`NetError::Unattached`], and [`NetError::SwitchingCycle`] or
    /// [`NetError::Disconnected`] with a witness, wrapped in
    /// [`NetError::InBox`] when it lies inside a box;
    /// [`NetError::Refused`] when `stop` ended it.
    pub fn is_correct(&self, stop: impl FnMut(Progress) -> bool) -> Result<(), NetError> {
        if self.is_empty() {
            return Err(NetError::Empty);
        }
        for v in self.vertices() {
            let kind = self.kind(v.get());
            if matches!(kind, Kind::Atom | Kind::DualAtom) && self.partner[v.index()] == NONE {
                return Err(NetError::Unlinked { vertex: v });
            }
            if kind == Kind::Quest
                && self.premises(v).next().is_none()
                && self.hangs[v.index()] == NONE
            {
                return Err(NetError::Unattached { vertex: v });
            }
        }
        self.graph.correct(self.criterion.mix, stop)
    }
}

impl std::fmt::Display for MellStructure {
    /// Writes the sequent, then a line per `?` vertex with its copies
    /// (`?A⊥[0]: A⊥[3] in !A[1], A⊥[5]`, a copy's box named where it is
    /// another than the `?` vertex's, by its `!` vertex), or `no copy` and
    /// the literal it hangs on, then one line per cut link as `cut !A[4] —
    /// ?A⊥[7]` in their order, then one line per link as `A⊥[3] — A[2]`,
    /// the links ordered by their first id, then the verdict of the
    /// criterion: `proof net`, `proof net with Mix`, or `not a proof net: `
    /// and the reason with formulas. Every vertex is written as its
    /// formula with its id. No trailing newline.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.write(f, None)
    }
}

impl MellStructure {
    /// Returns the structure for display as `mode` reads it: as
    /// [`Display`](std::fmt::Display) writes it, and in an intuitionistic
    /// mode, where the sequent has a reading, with the sequent two-sided
    /// and every formula as the reading writes it (`!A[0]: A[3] in
    /// !A[1]`).
    pub fn written_in(&self, mode: crate::Mode) -> impl std::fmt::Display + '_ {
        crate::nets::In {
            structure: self,
            mode,
        }
    }

    /// Writes the structure as its `Display` does, or as `mode` reads
    /// it: in an intuitionistic mode every formula as the reading writes
    /// it, where the sequent has one.
    pub(crate) fn write(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        mode: Option<crate::Mode>,
    ) -> std::fmt::Result {
        let reading = mode
            .filter(|mode| mode.is_intuitionistic())
            .and_then(|_| self.reading().ok());
        let vertex = |f: &mut std::fmt::Formatter<'_>, v: VertexId| {
            let o = self.occurrence[v.index()];
            match &reading {
                Some(reading) => write!(f, "{}[{}]", reading.formula(o), v.get()),
                None => write!(f, "{}[{}]", self.forest.formula(o), v.get()),
            }
        };
        match &reading {
            Some(reading) => writeln!(f, "{reading}")?,
            None => writeln!(f, "{}", self.forest.sequent())?,
        }

        for q in self
            .vertices()
            .filter(|&q| self.kind(q.get()) == Kind::Quest)
        {
            vertex(f, q)?;
            f.write_str(":")?;
            let mut copies = self.premises(q).peekable();
            if copies.peek().is_none() {
                f.write_str(" no copy")?;
                if let Some(l) = self.hangs_on(q) {
                    f.write_str(", hangs on ")?;
                    vertex(f, l)?;
                }
            }
            let mut first = true;
            for c in copies {
                f.write_str(if first { " " } else { ", " })?;
                first = false;
                vertex(f, c)?;
                if let Some(d) = self.inside(c).filter(|&d| Some(d) != self.inside(q)) {
                    f.write_str(" in ")?;
                    vertex(f, d)?;
                }
            }
            f.write_str("\n")?;
        }
        for &(x, y) in &self.cuts {
            f.write_str("cut ")?;
            vertex(f, x)?;
            f.write_str(" — ")?;
            vertex(f, y)?;
            f.write_str("\n")?;
        }
        let mut links: Vec<(VertexId, VertexId)> = self
            .links
            .iter()
            .map(|&(x, y)| (x.min(y), x.max(y)))
            .collect();
        links.sort_unstable();
        for (x, y) in links {
            vertex(f, x)?;
            f.write_str(" — ")?;
            vertex(f, y)?;
            f.write_str("\n")?;
        }
        match self.is_correct(|_| false) {
            Ok(()) if self.criterion.mix => f.write_str("proof net with Mix"),
            Ok(()) => f.write_str("proof net"),
            Err(e) => match mode {
                Some(mode) => write!(f, "not a proof net: {}", e.describe(self).in_mode(mode)),
                None => write!(f, "not a proof net: {}", e.describe(self)),
            },
        }
    }
}

/// Returns a raw index as a vertex, `None` for `NONE`.
fn some(raw: u32) -> Option<VertexId> {
    (raw != NONE).then_some(VertexId::new(raw))
}

/// The error of a vertex out of place.
fn misplaced(vertex: VertexId, fault: Placement) -> Error {
    NetError::Placement { vertex, fault }.into()
}

/// Offsets per vertex, with the total after them, and the list they index.
type Csr = (Box<[u32]>, Box<[u32]>);

/// Checks that every vertex stands for a premise of its parent's
/// occurrence and that every `⊗`, `⅋` and `!` has each premise once, and
/// returns the premises in CSR layout. A parent's occurrence is the parent
/// of its premise's, one level nearer the root, so parents never go round
/// in a cycle.
fn premises(forest: &Forest, occurrence: &[OccId], parent: &[u32]) -> Result<Csr, Error> {
    let n = occurrence.len();
    let mut count = vec![0u32; n + 1];
    for (v, &p) in parent.iter().enumerate() {
        if p == NONE {
            continue;
        }
        let o = occurrence[v];
        if forest.parent(o) != Some(occurrence[p as usize]) {
            return Err(misplaced(VertexId::new(v as u32), Placement::Premise));
        }
        count[p as usize] += 1;
    }
    // Offsets: a vertex's premises are fewer than the vertices.
    let mut start = Vec::with_capacity(n + 1);
    let mut total = 0;
    for &c in &count[..n] {
        start.push(total);
        total += c;
    }
    start.push(total);
    let mut fill = start.clone();
    let mut list = vec![0u32; total as usize];
    for (v, &p) in parent.iter().enumerate() {
        if p != NONE {
            list[fill[p as usize] as usize] = v as u32;
            fill[p as usize] += 1;
        }
    }
    for p in 0..n {
        let o = occurrence[p];
        let premises = &mut list[start[p] as usize..start[p + 1] as usize];
        let expected = match forest.kind(o) {
            Kind::Tensor | Kind::Par => 2,
            Kind::Bang => 1,
            Kind::Quest | Kind::Atom | Kind::DualAtom => continue,
            Kind::One
            | Kind::Bot
            | Kind::Top
            | Kind::Zero
            | Kind::With
            | Kind::Plus
            | Kind::Forall
            | Kind::Exists => {
                unreachable!("a unit-free MELL sequent has no such occurrence")
            }
        };
        // The left premise first; each premise once.
        if expected == 2
            && premises.len() == 2
            && occurrence[premises[0] as usize] != forest.left(o).unwrap()
        {
            premises.swap(0, 1);
        }
        let distinct = expected == 1
            || premises.len() != 2
            || occurrence[premises[0] as usize] != occurrence[premises[1] as usize];
        if premises.len() != expected || !distinct {
            return Err(misplaced(VertexId::new(p as u32), Placement::Premises));
        }
    }
    Ok((start.into_boxed_slice(), list.into_boxed_slice()))
}

/// Checks that the cut links join, in one box, vertices without a parent
/// that are no roots, an occurrence of a cut formula's tree and its dual,
/// that each such vertex is the end of one, and returns them each from its
/// end in a cut formula's tree.
fn place_cuts(
    forest: &Forest,
    occurrence: &[OccId],
    parent: &[u32],
    inside: &[u32],
    roots: usize,
    cuts: &[(VertexId, VertexId)],
) -> Result<Vec<(VertexId, VertexId)>, Error> {
    let n = occurrence.len();
    let mut ended = vec![false; n];
    let mut placed = Vec::with_capacity(cuts.len());
    for &(x, y) in cuts {
        let end = |v: VertexId| v.index() < n && v.index() >= roots && parent[v.index()] == NONE;
        if !end(x) || !end(y) || inside[x.index()] != inside[y.index()] {
            return Err(misplaced(x, Placement::Cut));
        }
        let (ox, oy) = (occurrence[x.index()], occurrence[y.index()]);
        let (a, d) = match (forest.dual(ox), forest.dual(oy)) {
            (Some(dx), _) if dx == oy && ox < oy => (x, y),
            (_, Some(dy)) if dy == ox && oy < ox => (y, x),
            _ => return Err(misplaced(x, Placement::Cut)),
        };
        for v in [a, d] {
            if std::mem::replace(&mut ended[v.index()], true) {
                return Err(misplaced(v, Placement::Cut));
            }
        }
        placed.push((a, d));
    }
    if let Some(v) = (roots..n).find(|&v| parent[v] == NONE && !ended[v]) {
        return Err(misplaced(VertexId::new(v as u32), Placement::Cut));
    }
    Ok(placed)
}

/// The depth of a box whose chain of enclosing boxes is being walked.
const VISITING: u32 = NONE - 1;

/// Checks that every vertex lies in the box its parent allows and that
/// boxes nest without a cycle.
fn check_boxes(
    forest: &Forest,
    occurrence: &[OccId],
    parent: &[u32],
    inside: &[u32],
    roots: usize,
) -> Result<(), Error> {
    let n = occurrence.len();
    let is_bang = |b: u32| b == NONE || forest.kind(occurrence[b as usize]) == Kind::Bang;
    for v in 0..n {
        let (p, b) = (parent[v], inside[v]);
        let fits = match p {
            // The roots lie outside every box, a cut link in any.
            NONE if v < roots => b == NONE,
            NONE => is_bang(b),
            p => match forest.kind(occurrence[p as usize]) {
                Kind::Bang => b == p,
                // A copy's box is checked once the boxes are known to nest.
                Kind::Quest => is_bang(b),
                Kind::Tensor | Kind::Par => b == inside[p as usize],
                Kind::Atom
                | Kind::DualAtom
                | Kind::One
                | Kind::Bot
                | Kind::Top
                | Kind::Zero
                | Kind::With
                | Kind::Plus
                | Kind::Forall
                | Kind::Exists => false,
            },
        };
        if !fits {
            return Err(misplaced(VertexId::new(v as u32), Placement::Box));
        }
    }
    // Every box's chain of enclosing boxes ends: a walk from each `!`
    // vertex, marking the boxes on the way, and the depth of those behind.
    let mut depth = vec![NONE; n];
    let mut path = Vec::new();
    for d in 0..n as u32 {
        if !is_bang(d) || depth[d as usize] != NONE {
            continue;
        }
        let mut b = d;
        while b != NONE && depth[b as usize] == NONE {
            depth[b as usize] = VISITING;
            path.push(b);
            b = inside[b as usize];
        }
        if b != NONE && depth[b as usize] == VISITING {
            return Err(misplaced(VertexId::new(d), Placement::Nesting));
        }
        let below = if b == NONE { 0 } else { depth[b as usize] + 1 };
        for (d, &c) in (below..).zip(path.iter().rev()) {
            depth[c as usize] = d;
        }
        path.clear();
    }
    Ok(())
}

/// The boxes of a structure as a tree, each with the interval of its
/// subtree in a depth-first order, so that whether one box lies inside
/// another is two comparisons. The box outside every box is `NONE`.
pub(super) struct Nesting {
    /// Per vertex, the `!` vertex of its innermost box, or `NONE`.
    inside: Box<[u32]>,
    /// Per `!` vertex, when the walk entered its box and when it left it.
    interval: Box<[(u32, u32)]>,
}

impl Nesting {
    /// Returns the nesting of a structure whose boxes nest.
    fn new(s: &MellStructure) -> Self {
        let n = s.len();
        let mut children = vec![Vec::new(); n + 1];
        for d in s.boxes() {
            let outer = s.inside[d.index()];
            let slot = if outer == NONE { n } else { outer as usize };
            children[slot].push(d.get());
        }
        let mut interval = vec![(0, 0); n];
        let mut time = 0;
        let mut stack = vec![(n as u32, 0usize)];
        while let Some(&mut (b, ref mut next)) = stack.last_mut() {
            let slot = if b as usize == n { n } else { b as usize };
            if let Some(&c) = children[slot].get(*next) {
                *next += 1;
                interval[c as usize].0 = time;
                time += 1;
                stack.push((c, 0));
            } else {
                if (b as usize) < n {
                    interval[b as usize].1 = time;
                }
                stack.pop();
            }
        }
        Self {
            inside: s.inside.clone(),
            interval: interval.into_boxed_slice(),
        }
    }

    /// Returns the box a vertex lies in.
    pub(super) fn of(&self, v: VertexId) -> u32 {
        self.inside[v.index()]
    }

    /// Returns whether box `b` lies in box `outer` or is it.
    pub(super) fn within(&self, b: u32, outer: u32) -> bool {
        if outer == NONE || b == outer {
            return true;
        }
        if b == NONE {
            return false;
        }
        let (i, o) = (self.interval[b as usize], self.interval[outer as usize]);
        o.0 < i.0 && i.0 < o.1
    }
}

impl MellStructure {
    /// Checks the copies' boxes, which [`check_boxes`] leaves for when
    /// the boxes are known to nest.
    fn check_copies(&self, boxes: &Nesting) -> Result<(), Error> {
        for v in self.vertices() {
            let p = self.parent[v.index()];
            if p != NONE
                && self.kind(p) == Kind::Quest
                && !boxes.within(self.inside[v.index()], self.inside[p as usize])
            {
                return Err(misplaced(v, Placement::Box));
            }
        }
        Ok(())
    }
}
