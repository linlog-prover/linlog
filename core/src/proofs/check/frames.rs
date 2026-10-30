// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! What the checker's bottom-up pass asks of a proof's members: the
//! occurrence a member stands for, the member of a subformula under the
//! same frame, and whether two literals are an atom and its negation. A
//! proof without binders answers by occurrences alone ([`Ground`]); a
//! proof with an instance table answers from the frame pass ([`Framed`]),
//! which runs before the bottom-up pass.

use super::error::Fault;
use crate::hash::{HashMap, HashSet};
use crate::occurrences::instances::{Numbering, Terms, binders};
use crate::occurrences::{Forest, FrameId, Instances, Member, OccId};
use crate::proofs::{Node, NodeId, Proof};
use crate::sequents::{Eigen, FoTerm, FoTermId};

/// What the bottom-up pass needs to know of members beyond the forest.
pub(crate) trait Instancing {
    /// Whether the proof has an instance table, and the pass carries
    /// eigenvariables: false for the pass of a proof without binders,
    /// which compiles without the eigenvariables' code.
    const FRAMED: bool;

    /// Returns the occurrence `m` stands for.
    fn occ(&self, m: Member) -> OccId;

    /// Returns the member of `c`, a subformula of `m`'s occurrence that is
    /// no binder's body, under `m`'s frame.
    fn child(&self, m: Member, c: OccId) -> Member;

    /// Returns the member of `p`, the parent of `m`'s occurrence and no
    /// binder, under `m`'s frame, where the proof has it.
    fn parent(&self, m: Member, p: OccId) -> Member;

    /// Returns whether the literals `a` and `b` are an atom and its
    /// negation: one atom, or under frames one predicate applied to equal
    /// terms.
    fn dual_literals(&mut self, forest: &Forest, a: Member, b: Member) -> bool;

    /// Returns the member of the dual of `m`'s occurrence in its cut
    /// formula's pair, under `m`'s frame, if it has one.
    fn dual(&self, forest: &Forest, m: Member) -> Option<Member>;

    /// Returns the fault the frame pass found at `node`, if any: a member
    /// under binders named without its instance, an eigenvariable
    /// introduced a second time.
    fn fault(&self, node: NodeId) -> Option<Fault>;

    /// Returns the instance of the body of the binder that the `∀` or `∃`
    /// step `node` acts on, under the frame with its eigenvariable or
    /// witness pushed.
    fn body(&self, node: NodeId) -> Member;

    /// Adds to `into` the eigenvariables the `∃` step or the cut `node`
    /// brings in, those of its witness or of its member's frame, and
    /// returns the units of work that took: a term or a frame visited is
    /// one. Each shared term is visited once.
    fn bring(&self, node: Node, into: &mut Vec<Eigen>) -> u64;

    /// Returns whether a `∀` step of the proof introduces `a`: one no step
    /// introduces is a constant outside the signature, which any witness
    /// may name.
    fn introduces(&self, a: Eigen) -> bool;

    /// Returns how many terms the frame of `m` holds: the work of reading
    /// its atom's instance, `0` for an occurrence.
    fn frame_len(&self, m: Member) -> u64;
}

/// The members of a proof without binders: its occurrences.
pub(crate) struct Ground;

impl Instancing for Ground {
    const FRAMED: bool = false;

    #[inline(always)]
    fn occ(&self, m: Member) -> OccId {
        m.occ()
    }

    #[inline(always)]
    fn child(&self, _: Member, c: OccId) -> Member {
        Member::from(c)
    }

    #[inline(always)]
    fn parent(&self, _: Member, p: OccId) -> Member {
        Member::from(p)
    }

    #[inline(always)]
    fn dual_literals(&mut self, forest: &Forest, a: Member, b: Member) -> bool {
        forest.dual_literals(a.occ(), b.occ())
    }

    #[inline(always)]
    fn dual(&self, forest: &Forest, m: Member) -> Option<Member> {
        forest.dual(m.occ()).map(Member::from)
    }

    #[inline(always)]
    fn fault(&self, _: NodeId) -> Option<Fault> {
        None
    }

    fn body(&self, _: NodeId) -> Member {
        // A forest without binders has no occurrence of the kind a `∀` or
        // `∃` step acts on, which the pass tests first.
        unreachable!("a binder's step on a forest without binders fails its kind")
    }

    fn bring(&self, _: Node, _: &mut Vec<Eigen>) -> u64 {
        0
    }

    #[inline(always)]
    fn frame_len(&self, _: Member) -> u64 {
        0
    }

    fn introduces(&self, _: Eigen) -> bool {
        false
    }
}

/// What the frame pass found at a node.
#[derive(Debug, Default)]
struct Found {
    /// The node's fault, if any.
    fault: Option<Fault>,
    /// The body's instance of a `∀` or `∃` step.
    body: Option<Member>,
}

/// The members of a proof with an instance table, which
/// [`Proof::with_instances`] made canonical and closed under the members
/// the rules derive: equal instances are one member, so the bottom-up
/// pass compares them by number.
pub(crate) struct Framed<'a> {
    /// The proof's table.
    table: &'a Instances,
    /// The numbers of the terms, by which atoms' instances compare.
    numbering: Numbering<'a>,
    /// Every frame of the table, by its parent and term.
    frames: HashMap<(Option<FrameId>, FoTermId), FrameId>,
    /// Every member of the table, by its occurrence and frame.
    members: HashMap<(OccId, FrameId), Member>,
    /// What the frame pass found, by node.
    found: HashMap<NodeId, Found>,
    /// Per frame, how many terms it holds.
    lengths: Vec<u64>,
    /// The eigenvariables a `∀` step introduces.
    introduced: HashSet<Eigen>,
}

impl<'a> Framed<'a> {
    /// Runs the frame pass over `proof`, top-down in reverse arena order:
    /// every member a node names under binders must be an instance of
    /// the table; a `∀` step's body is its member's frame with its
    /// eigenvariable pushed, an `∃` step's with its witness, and the
    /// eigenvariables of the witness, as those of a cut's frame, come in
    /// there; every eigenvariable is introduced by one `∀` step, the
    /// first in arena order, and a second introduction is a fault of the
    /// later one. A frame is never derived here: every member names its
    /// own, and the bottom-up pass finds a member a premise lacks.
    pub(crate) fn new(proof: &'a Proof, table: &'a Instances) -> Self {
        let forest = proof.forest();
        let terms = Terms {
            fo: &forest.sequent().fo,
            table,
        };
        let mut this = Self {
            table,
            numbering: Numbering::new(terms),
            frames: table
                .frames
                .iter()
                .zip(0..)
                .map(|(&key, f)| (key, FrameId::new(f)))
                .collect(),
            members: table
                .members
                .iter()
                .zip(0..)
                .map(|(&key, n)| (key, Member::new(table.occurrences + n)))
                .collect(),
            found: HashMap::default(),
            lengths: Vec::with_capacity(table.frames.len()),
            introduced: HashSet::default(),
        };
        // A parent precedes its frames in a canonical table.
        for &(parent, _) in &table.frames {
            let length = parent.map_or(1, |p| this.lengths[p.index()].saturating_add(1));
            this.lengths.push(length);
        }
        let eigen_terms: HashMap<Eigen, FoTermId> = table
            .terms
            .iter()
            .zip(0..)
            .filter_map(|(t, n)| match *t {
                FoTerm::Eigen(a) => Some((a, FoTermId::new(table.base_terms + n))),
                FoTerm::Bound(_) | FoTerm::App(..) | FoTerm::Meta(_) => None,
            })
            .collect();
        let depth = binders(forest);
        let mut introduced: HashMap<Eigen, NodeId> = HashMap::default();
        for id in proof.ids().rev() {
            let node = proof.node(id);
            let mut found = Found::default();
            let bare = node
                .members()
                .find(|m| m.index() < forest.len() && depth[m.index()] > 0);
            if let Some(member) = bare {
                found.fault = Some(Fault::Instance { member });
            }
            match node {
                Node::Forall(m, a, _) if bare.is_none() => {
                    introduced
                        .entry(a)
                        .and_modify(|first| *first = (*first).min(id))
                        .or_insert(id);
                    let o = this.occ(m);
                    if forest.kind(o).is_binder() {
                        let term = eigen_terms[&a];
                        found.body = Some(this.pushed(m, o, term));
                    }
                }
                Node::Exists(m, w, _) if bare.is_none() => {
                    let o = this.occ(m);
                    if forest.kind(o).is_binder() {
                        found.body = Some(this.pushed(m, o, w));
                    }
                }
                _ => {}
            }
            if found.fault.is_some() || found.body.is_some() {
                this.found.insert(id, found);
            }
        }
        this.introduced = introduced.keys().copied().collect();
        for id in proof.ids() {
            if let Node::Forall(_, a, _) = proof.node(id)
                && introduced.get(&a).is_some_and(|&first| first != id)
            {
                let found = this.found.entry(id).or_default();
                found.fault.get_or_insert(Fault::Reintroduced { eigen: a });
            }
        }
        this
    }

    /// Returns the frame of `m`, if it is an instance.
    fn frame(&self, m: Member) -> Option<FrameId> {
        self.table.instance(m).map(|(_, f)| f)
    }

    /// Returns the instance of the body of `m`'s occurrence `o`, a
    /// binder, under `m`'s frame with `term` pushed: a member of the table,
    /// which holds every one a node derives.
    fn pushed(&self, m: Member, o: OccId, term: FoTermId) -> Member {
        let frame = self.frames[&(self.frame(m), term)];
        self.members[&(OccId::new(o.get() + 1), frame)]
    }
}

impl Instancing for Framed<'_> {
    const FRAMED: bool = true;

    fn occ(&self, m: Member) -> OccId {
        self.table.instance(m).map_or(m.occ(), |(o, _)| o)
    }

    fn child(&self, m: Member, c: OccId) -> Member {
        match self.frame(m) {
            None => Member::from(c),
            // The table holds every subformula's instance of a member a
            // node names.
            Some(frame) => self.members[&(c, frame)],
        }
    }

    fn parent(&self, m: Member, p: OccId) -> Member {
        match self.frame(m) {
            None => Member::from(p),
            Some(frame) => self
                .members
                .get(&(p, frame))
                .copied()
                .unwrap_or(Member::from(p)),
        }
    }

    fn dual_literals(&mut self, forest: &Forest, a: Member, b: Member) -> bool {
        let (x, y) = (self.occ(a), self.occ(b));
        let (Some(sx), Some(sy)) = (forest.sign(x), forest.sign(y)) else {
            return false;
        };
        if sx == sy {
            return false;
        }
        let sequent = forest.sequent();
        let atoms = (forest.atom(x), forest.atom(y));
        let (Some(ax), Some(ay)) = atoms else {
            return false;
        };
        let predicates = (sequent.atom_predicate(ax), sequent.atom_predicate(ay));
        let (Some((px, lx)), Some((py, ly))) = predicates else {
            // Nullary atoms of a sequent with binders but no predicates
            // are compared by name.
            return ax == ay;
        };
        if px != py {
            return false;
        }
        let mut args = |m: Member, list| {
            let frame: Vec<u64> = self.frame(m).map_or_else(Vec::new, |f| {
                self.table
                    .frame_terms(f)
                    .into_iter()
                    .map(|t| self.numbering.number(t))
                    .collect()
            });
            self.numbering.instance(list, &frame)
        };
        args(a, lx) == args(b, ly)
    }

    fn dual(&self, forest: &Forest, m: Member) -> Option<Member> {
        let dual = forest.dual(self.occ(m))?;
        Some(self.child(m, dual))
    }

    fn fault(&self, node: NodeId) -> Option<Fault> {
        self.found.get(&node).and_then(|f| f.fault.clone())
    }

    fn body(&self, node: NodeId) -> Member {
        self.found[&node]
            .body
            .expect("the kind test passed: the step acts on a binder")
    }

    fn frame_len(&self, m: Member) -> u64 {
        self.frame(m).map_or(0, |f| self.lengths[f.index()])
    }

    fn introduces(&self, a: Eigen) -> bool {
        self.introduced.contains(&a)
    }

    fn bring(&self, node: Node, into: &mut Vec<Eigen>) -> u64 {
        let mut stack = match node {
            Node::Exists(_, w, _) => vec![w],
            Node::Cut(m, ..) => match self.frame(m) {
                Some(frame) => self.table.frame_terms(frame),
                None => return 0,
            },
            _ => return 0,
        };
        // The frame's cells, each one unit.
        let mut units = stack.len() as u64;
        let terms = self.numbering.terms();
        let mut seen = HashSet::default();
        while let Some(t) = stack.pop() {
            if !seen.insert(t) {
                continue;
            }
            units = units.saturating_add(1);
            match terms.term(t) {
                FoTerm::Eigen(a) => into.push(a),
                FoTerm::App(_, args) => stack.extend_from_slice(terms.list(args)),
                FoTerm::Bound(_) | FoTerm::Meta(_) => {}
            }
        }
        units
    }
}
