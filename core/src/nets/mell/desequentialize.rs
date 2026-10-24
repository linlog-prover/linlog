// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Desequentialization: from a proof term of unit-free MELL to its
//! structure with boxes. The term is unfolded as a tree, a node that
//! several others share once per use, and read from the leaves: every
//! rule makes the vertex of the formula it introduces from the vertices
//! its premises left open, so the vertices of a formula are instances of
//! its occurrence. An axiom makes two literals and links them; a promotion
//! closes the box around everything its premise made; a copy moves an
//! instance from the linear zone to the unrestricted one; a `?` rule
//! takes every copy of its subformula its premise made and no `?` rule
//! above took, as the checker reads the unrestricted zone, and without
//! any it is a weakening that hangs on the first link above it.
//!
//! A term names occurrences, not instances, so where a sequent holds two
//! instances of one occurrence the rule acting on it takes the last one
//! made: any choice is a sequent proof, and its net a proof net.

use super::{BUILD_BYTES, MOST, MellStructure, Parts, Placed, VERTEX_BYTES};
use crate::Error;
use crate::fragment::Mode;
use crate::limits::{Limits, Phase, Progress, Refusal, Space};
use crate::nets::{Criterion, NONE, NetError, STOPPED, VertexId, afford};
use crate::occurrences::OccId;
use crate::proofs::{Node, NodeId, Proof};

/// The bytes the unfolding holds per vertex beside the structure: its
/// occurrence and box (8), its list of premises and the entry it takes in
/// its parent's (28), an entry in a list of open or waiting instances (8),
/// the renumbering's three arrays (12) and its vertex of the parts (16).
const UNFOLD_BYTES: u64 = 8 + 28 + 8 + 12 + 16;

impl MellStructure {
    /// Desequentializes a proof of unit-free MELL in classical mode, with
    /// Mix where the criterion allows it, or in intuitionistic mode, whose
    /// every proof is a classical one of the one-sided sequent: checks it
    /// in classical mode, unfolds it into its
    /// structure with boxes over a copy of the proof's forest, within
    /// `limits` (the check's bounds, and the unfolded vertices, counted
    /// before anything is made, within `limits.memory_bytes`) and until
    /// `stop` returns true, which the check asks and the unfolding asks per
    /// node. The vertices are numbered from the roots, then each root's
    /// formula tree depth first, a `?` vertex's copies in the order they
    /// were made. Every proof gives a proof net; two proofs that differ in
    /// the order of their rules give the same one up to the axiom links the
    /// weakenings hang on and, where a sequent holds two instances of one
    /// occurrence, up to the instance a rule takes (the last made).
    ///
    /// # Errors
    ///
    /// [`Error::GoalProof`] for a proof of a goal other than the sequent;
    /// [`NetError::NotMell`] outside unit-free MELL; [`NetError::Rule`]
    /// for a cut, which no structure links yet;
    /// the checker's [`CheckError`](crate::proofs::CheckError) for a term
    /// that is no proof; [`Refusal::Index`] for more vertices than a
    /// structure holds; [`NetError::Refused`] when the bound or the stop
    /// ended it.
    pub fn from_proof(
        proof: &Proof,
        criterion: Criterion,
        limits: &Limits,
        mut stop: impl FnMut(Progress) -> bool,
    ) -> Result<Self, Error> {
        if proof.goal().is_some() {
            return Err(Error::GoalProof);
        }
        let mode = if criterion.mix {
            Mode::CLASSICAL.with_mix()
        } else {
            Mode::CLASSICAL
        };
        Self::exist(proof.sequent().fragment(), mode)?;
        if let Some(i) = proof
            .nodes()
            .iter()
            .position(|n| matches!(n, Node::Cut(..)))
        {
            return Err(NetError::Rule {
                node: NodeId::new(i as u32),
                rule: "cut",
            }
            .into());
        }
        proof.check_within(mode, limits, &mut stop)?;
        let vertices = unfolded(proof);
        if vertices > MOST {
            return Err(Error::Refused(Refusal::Index {
                what: Space::Vertex,
                count: vertices,
                most: MOST,
            }));
        }
        let bytes = vertices.saturating_mul(VERTEX_BYTES + BUILD_BYTES + UNFOLD_BYTES);
        afford(bytes, limits)?;
        let mut unfolding = Unfolding::new(proof, vertices as usize);
        unfolding.run(&mut |done| {
            stop(Progress {
                held_bytes: bytes,
                ..Progress::new(Phase::Net, 1, done)
            })
        })?;
        let parts = unfolding.parts();
        let structure = Self::from_parts(proof.forest().clone(), criterion, &parts, limits)?;
        structure.is_correct(|progress| {
            stop(Progress {
                held_bytes: bytes,
                ..progress
            })
        })?;
        Ok(structure)
    }
}

/// Returns the vertices the unfolded term makes: the uses of every node,
/// counted from the root down and saturating, times the vertices its rule
/// makes.
fn unfolded(proof: &Proof) -> u64 {
    let nodes = proof.nodes();
    let mut uses = vec![0u64; nodes.len()];
    let root = proof.root().index();
    uses[root] = 1;
    let mut total = 0u64;
    for i in (0..=root).rev() {
        let made = match nodes[i] {
            Node::Ax(..) => 2,
            // A cut was refused before.
            Node::Copy(..) | Node::Mix(..) | Node::Cut(..) => 0,
            Node::Tensor(..)
            | Node::Par(..)
            | Node::Bang(..)
            | Node::Quest(..)
            | Node::Weaken(..)
            | Node::One(_)
            | Node::Bot(..)
            | Node::With(..)
            | Node::Plus(..)
            | Node::Top(_) => 1,
        };
        total = total.saturating_add(uses[i].saturating_mul(made));
        for p in nodes[i].premises() {
            uses[p.index()] = uses[p.index()].saturating_add(uses[i]);
        }
    }
    total
}

/// A node of the unfolded term being read: its premises are read first,
/// each into the vertices from `start` and `middle` on.
struct Frame {
    /// The node.
    node: NodeId,
    /// The premises read so far.
    read: u8,
    /// The vertex count when the node was entered.
    start: u32,
    /// The vertex count when its first premise was read; for a `?` rule
    /// the copies of its subformula waiting when it was entered.
    middle: u32,
    /// The link count when the node was entered.
    links: u32,
}

/// An unfolding in progress.
struct Unfolding<'a> {
    /// The proof.
    proof: &'a Proof,
    /// Per vertex, its occurrence.
    occurrence: Vec<OccId>,
    /// Per vertex, its box, as an index into `doors`, or `NONE`.
    boxed: Vec<u32>,
    /// Per vertex, its premises as they were made.
    premises: Vec<Vec<u32>>,
    /// Per box, its `!` vertex once its promotion is read.
    doors: Vec<u32>,
    /// The boxes the node being read lies in, the innermost last.
    open_boxes: Vec<u32>,
    /// Per occurrence, its instances in a linear zone, in the order made.
    open: Vec<Vec<u32>>,
    /// Per occurrence under a `?`, its copies no `?` rule took yet.
    waiting: Vec<Vec<u32>>,
    /// The links, in the order made.
    links: Vec<(u32, u32)>,
    /// The weakenings and the literal each hangs on.
    weakenings: Vec<(u32, u32)>,
}

impl<'a> Unfolding<'a> {
    /// Returns an unfolding of a proof that makes `vertices` vertices.
    fn new(proof: &'a Proof, vertices: usize) -> Self {
        let n = proof.forest().len();
        Self {
            proof,
            occurrence: Vec::with_capacity(vertices),
            boxed: Vec::with_capacity(vertices),
            premises: Vec::with_capacity(vertices),
            doors: Vec::new(),
            open_boxes: Vec::new(),
            open: vec![Vec::new(); n],
            waiting: vec![Vec::new(); n],
            links: Vec::new(),
            weakenings: Vec::new(),
        }
    }

    /// Makes a vertex in the current box and returns it.
    fn make(&mut self, o: OccId, premises: Vec<u32>) -> u32 {
        let v = self.occurrence.len() as u32;
        self.occurrence.push(o);
        self.boxed
            .push(self.open_boxes.last().copied().unwrap_or(NONE));
        self.premises.push(premises);
        self.open[o.index()].push(v);
        v
    }

    /// Takes the last instance of `o` made from vertex `from` on and before
    /// vertex `to` out of the linear zone: one a checked proof's premise
    /// holds.
    fn take(&mut self, o: OccId, from: u32, to: u32) -> u32 {
        let list = &mut self.open[o.index()];
        let i = list.partition_point(|&v| v < to);
        assert!(
            i > 0 && list[i - 1] >= from,
            "a checked premise holds its rule's subformula"
        );
        list.remove(i - 1)
    }

    /// Reads the whole term, asking `stop` with the nodes read so far
    /// before each.
    fn run(&mut self, stop: &mut dyn FnMut(u64) -> bool) -> Result<(), NetError> {
        let nodes = self.proof.nodes();
        let mut frames = vec![self.enter(self.proof.root())];
        let mut read = 0u64;
        while let Some(frame) = frames.last_mut() {
            let node = nodes[frame.node.index()];
            let premise = node.premises().nth(frame.read as usize);
            if let Some(p) = premise {
                if frame.read == 1 {
                    frame.middle = self.occurrence.len() as u32;
                }
                frame.read += 1;
                read += 1;
                if stop(read) {
                    return Err(STOPPED);
                }
                let entered = self.enter(p);
                frames.push(entered);
                continue;
            }
            let frame = frames.pop().expect("the frame just looked at");
            self.exit(node, &frame);
        }
        Ok(())
    }

    /// Enters a node: a promotion opens its box, a `?` rule notes the
    /// copies waiting.
    fn enter(&mut self, id: NodeId) -> Frame {
        let node = self.proof.node(id);
        let mut middle = self.occurrence.len() as u32;
        match node {
            Node::Bang(..) => {
                self.open_boxes.push(self.doors.len() as u32);
                self.doors.push(NONE);
            }
            Node::Quest(q, _) => {
                let a = self.child(q.occ());
                middle = self.waiting[a.index()].len() as u32;
            }
            Node::Ax(..)
            | Node::Tensor(..)
            | Node::Par(..)
            | Node::One(_)
            | Node::Bot(..)
            | Node::With(..)
            | Node::Plus(..)
            | Node::Top(_)
            | Node::Copy(..)
            | Node::Weaken(..)
            | Node::Mix(..)
            | Node::Cut(..) => {}
        }
        Frame {
            node: id,
            read: 0,
            start: self.occurrence.len() as u32,
            middle,
            links: self.links.len() as u32,
        }
    }

    /// Returns the subformula of a unary connective's occurrence.
    fn child(&self, o: OccId) -> OccId {
        self.proof
            .forest()
            .left(o)
            .expect("a unary connective has a subformula")
    }

    /// Reads a node whose premises are read: makes the vertex of the
    /// formula it introduces.
    fn exit(&mut self, node: Node, frame: &Frame) {
        let f = self.proof.forest();
        let end = self.occurrence.len() as u32;
        match node {
            Node::Ax(x, y) => {
                let vx = self.make(x.occ(), Vec::new());
                let vy = self.make(y.occ(), Vec::new());
                self.links.push((vx, vy));
            }
            Node::Tensor(o, _, _) => {
                let o = o.occ();
                let l = self.take(f.left(o).unwrap(), frame.start, frame.middle);
                let r = self.take(f.right(o).unwrap(), frame.middle, end);
                self.make(o, vec![l, r]);
            }
            Node::Par(o, _) => {
                let o = o.occ();
                let l = self.take(f.left(o).unwrap(), frame.start, end);
                let r = self.take(f.right(o).unwrap(), frame.start, end);
                self.make(o, vec![l, r]);
            }
            Node::Bang(o, _) => {
                let o = o.occ();
                let a = self.take(self.child(o), frame.start, end);
                let b = self.open_boxes.pop().expect("the box its entry opened");
                let door = self.make(o, vec![a]);
                self.doors[b as usize] = door;
            }
            Node::Quest(o, _) => {
                let o = o.occ();
                let a = self.child(o);
                let copies = self.waiting[a.index()].split_off(frame.middle as usize);
                let q = self.make(o, copies);
                if self.premises[q as usize].is_empty() {
                    self.weakenings
                        .push((q, self.links[frame.links as usize].0));
                }
            }
            Node::Weaken(o, _) => {
                let q = self.make(o.occ(), Vec::new());
                self.weakenings
                    .push((q, self.links[frame.links as usize].0));
            }
            Node::Copy(a, _) => {
                let a = a.occ();
                let copy = self.take(a, frame.start, end);
                self.waiting[a.index()].push(copy);
            }
            Node::Mix(..) => {}
            Node::One(_) | Node::Bot(..) | Node::With(..) | Node::Plus(..) | Node::Top(_) => {
                unreachable!("a checked proof of unit-free MELL has no such rule")
            }
            Node::Cut(..) => unreachable!("a proof with a cut is refused before"),
        }
    }

    /// Returns the parts of the structure read, the vertices renumbered
    /// from the roots, then each root's tree depth first.
    fn parts(self) -> Parts {
        let f = self.proof.forest();
        let roots: Vec<u32> = f
            .roots()
            .iter()
            .map(|r| {
                let list = &self.open[r.index()];
                debug_assert_eq!(list.len(), 1, "a proof concludes each root once");
                list[0]
            })
            .collect();
        let n = self.occurrence.len();
        let mut new = vec![NONE; n];
        let mut order = Vec::with_capacity(n);
        for &r in &roots {
            new[r as usize] = order.len() as u32;
            order.push(r);
        }
        let mut stack = Vec::new();
        for &r in &roots {
            stack.push(r);
            while let Some(v) = stack.pop() {
                if new[v as usize] == NONE {
                    new[v as usize] = order.len() as u32;
                    order.push(v);
                }
                stack.extend(self.premises[v as usize].iter().rev());
            }
        }
        let mut parent = vec![NONE; n];
        for v in 0..n {
            for &w in &self.premises[v] {
                parent[w as usize] = v as u32;
            }
        }
        let renamed = |v: u32| VertexId::new(new[v as usize]);
        let vertices = order[roots.len()..]
            .iter()
            .map(|&v| {
                let b = self.boxed[v as usize];
                let inside = (b != NONE).then(|| renamed(self.doors[b as usize]));
                Placed::new(
                    self.occurrence[v as usize],
                    renamed(parent[v as usize]),
                    inside,
                )
            })
            .collect();
        Parts {
            vertices,
            links: self
                .links
                .iter()
                .map(|&(x, y)| (renamed(x), renamed(y)))
                .collect(),
            weakenings: self
                .weakenings
                .iter()
                .map(|&(q, l)| (renamed(q), renamed(l)))
                .collect(),
        }
    }
}
