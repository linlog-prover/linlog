// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Cut elimination on proofs with instances: the members of a step's
//! proof past the forest, the substitution of a witness for an
//! eigenvariable, and the renaming apart that keeps every eigenvariable
//! introduced by one `∀` step.
//!
//! A step reads the proof's members through its instance table and adds
//! to a copy of it what it makes: a binder's body under a witness, and the
//! terms, frames and members of a subproof it substitutes into. A step
//! copies a subproof only where it changes, so a `∀` step it remakes may
//! still be read elsewhere, by a subproof that two places share; two
//! `∀` steps with one eigenvariable are no proof, so the copy is given an
//! eigenvariable of its own ([`Rewrite::apart`]).

use super::Rewrite;
use crate::hash::HashMap;
use crate::limits::{Phase, Progress, Refusal, Space};
use crate::occurrences::instances::Terms;
use crate::occurrences::{FrameId, Instances, Member, OccId};
use crate::proofs::{Node, NodeId, Proof};
use crate::sequents::{Eigen, FoTerm, FoTermId};

/// What a step knows of the members of a proof with instances: the
/// proof's table, extended by what the step makes, and the lookups that
/// find an instance's member, a frame and an eigenvariable's term by what
/// they hold.
pub(super) struct Table {
    /// The proof's table and what the step added to it.
    pub(super) instances: Instances,
    /// Every member of the table, by its occurrence and frame.
    members: HashMap<(OccId, FrameId), Member>,
    /// Every frame of the table, by its parent and term.
    frames: HashMap<(Option<FrameId>, FoTermId), FrameId>,
    /// The term of every eigenvariable the table holds.
    eigens: HashMap<Eigen, FoTermId>,
    /// The next eigenvariable that no term and no `∀` step names, or
    /// `None` past the last a number names.
    next: Option<u32>,
}

impl Table {
    /// Returns the table of `proof`, or `None` for a proof without one.
    pub(super) fn of(proof: &Proof) -> Option<Self> {
        let instances = proof.instances()?.clone();
        let members = instances
            .members
            .iter()
            .enumerate()
            // The table's ids are `u32`s, which `Proof::with_instances`
            // checked.
            .map(|(n, &key)| (key, Member::new(instances.occurrences + n as u32)))
            .collect();
        let frames = instances
            .frames
            .iter()
            .enumerate()
            .map(|(n, &key)| (key, FrameId::new(n as u32)))
            .collect();
        let mut eigens = HashMap::default();
        let mut most: Option<u32> = None;
        for (n, term) in instances.terms.iter().enumerate() {
            if let FoTerm::Eigen(a) = *term {
                eigens
                    .entry(a)
                    .or_insert(FoTermId::new(instances.base_terms + n as u32));
                most = most.max(Some(a.get()));
            }
        }
        for node in proof.nodes() {
            if let Node::Forall(_, a, _) = *node {
                most = most.max(Some(a.get()));
            }
        }
        let next = most.map_or(Some(0), |a| a.checked_add(1));
        Some(Self {
            instances,
            members,
            frames,
            eigens,
            next,
        })
    }

    /// Returns the occurrence and frame of a member, `None` for one of
    /// the forest's own occurrences.
    pub(super) fn resolve(&self, m: Member) -> (OccId, Option<FrameId>) {
        match self.instances.instance(m) {
            Some((o, frame)) => (o, Some(frame)),
            None => (m.occ(), None),
        }
    }

    /// Returns the member that is `o` under `frame`, added where the
    /// table lacks it; the occurrence itself without a frame.
    pub(super) fn member(&mut self, o: OccId, frame: Option<FrameId>) -> Member {
        let Some(frame) = frame else {
            return Member::from(o);
        };
        *self
            .members
            .entry((o, frame))
            .or_insert_with(|| self.instances.member(o, frame))
    }

    /// Returns the member that is `o` under the frame of `like`, a member
    /// of an occurrence under as many binders.
    pub(super) fn beside(&mut self, o: OccId, like: Member) -> Member {
        let (_, frame) = self.resolve(like);
        self.member(o, frame)
    }

    /// Returns the frame that pushes `term` onto `parent`, added where the
    /// table lacks it.
    pub(super) fn frame(&mut self, parent: Option<FrameId>, term: FoTermId) -> FrameId {
        *self
            .frames
            .entry((parent, term))
            .or_insert_with(|| self.instances.frame(parent, term))
    }

    /// Returns the term of the eigenvariable `a`, added where the table
    /// lacks it.
    pub(super) fn eigen(&mut self, a: Eigen) -> FoTermId {
        *self
            .eigens
            .entry(a)
            .or_insert_with(|| self.instances.eigen(a))
    }

    /// Returns the member of the body of the binder `m` stands for, under
    /// `m`'s frame with `term` pushed.
    pub(super) fn body(&mut self, m: Member, term: FoTermId) -> Member {
        let (o, frame) = self.resolve(m);
        let frame = self.frame(frame, term);
        self.member(OccId::new(o.get() + 1), Some(frame))
    }

    /// Returns a new eigenvariable that no term and no `∀` step of the
    /// proof names, or the refusal past the last a number names.
    pub(super) fn fresh(&mut self) -> Result<Eigen, Refusal> {
        let a = self.next.ok_or(Refusal::Index {
            what: Space::Term,
            count: u64::from(u32::MAX) + 1,
            most: u64::from(u32::MAX),
        })?;
        self.next = a.checked_add(1);
        Ok(Eigen::new(a))
    }

    /// Returns the terms of the sequent and of the table, one space of
    /// ids.
    fn terms<'t>(&'t self, proof: &'t Proof) -> Terms<'t> {
        Terms {
            fo: &proof.forest().sequent().fo,
            table: &self.instances,
        }
    }
}

/// Which terms and frames name an eigenvariable a substitution maps, so
/// that one shared by many nodes is walked once.
#[derive(Default)]
struct Named {
    /// The terms, by their ids.
    terms: HashMap<FoTermId, bool>,
    /// The frames.
    frames: HashMap<FrameId, bool>,
}

/// What a substitution replaced so far, so that a term, frame or member
/// shared by many nodes is walked once.
#[derive(Default)]
struct Replaced {
    /// The terms, by their ids before.
    terms: HashMap<FoTermId, FoTermId>,
    /// The frames.
    frames: HashMap<FrameId, FrameId>,
    /// The members.
    members: HashMap<Member, Member>,
}

impl Rewrite<'_> {
    /// Returns the table of a proof with instances.
    fn table(&mut self) -> &mut Table {
        self.table
            .as_mut()
            .expect("a proof with instances has a table")
    }

    /// Returns the subproof at `top` with the eigenvariables `sigma` maps
    /// replaced by their terms: in the frames of the members its nodes
    /// name, in the witnesses of its `∃` steps, and as the eigenvariable of
    /// a `∀` step, where the term is an eigenvariable (a renaming). A node
    /// whose subproof names none of them is kept, shared with whatever
    /// else reads it; every other is made anew. Terms are closed, so no
    /// variable is captured. The nodes made anew are counted and charged
    /// ([`afford`](Self::afford)) before any is made: a copy has at most
    /// the nodes `top` reaches.
    pub(super) fn substitute(
        &mut self,
        top: NodeId,
        sigma: &HashMap<Eigen, FoTermId>,
    ) -> Result<NodeId, Refusal> {
        let reach = reached(&self.nodes, top, None);
        let mut named = Named::default();
        let mut changes = vec![false; top.index() + 1];
        for id in (0..=top.index()).filter(|&i| reach[i]) {
            let node = self.nodes[id];
            changes[id] = node.premises().any(|p| changes[p.index()])
                || node
                    .members()
                    .any(|m| self.names_member(m, sigma, &mut named))
                || match node {
                    Node::Exists(_, w, _) => self.names_term(w, sigma, &mut named),
                    Node::Forall(_, a, _) => sigma.contains_key(&a),
                    _ => false,
                };
        }
        self.afford(changes.iter().filter(|&&c| c).count())?;
        let mut replaced = Replaced::default();
        let mut made: HashMap<NodeId, NodeId> = HashMap::default();
        for id in (0..=top.index()).filter(|&i| changes[i]) {
            let node = self.nodes[id];
            let mut pairs: Vec<(Member, Member)> = Vec::with_capacity(2);
            for m in node.members() {
                pairs.push((m, self.substituted_member(m, sigma, &mut replaced)));
            }
            let renamed = match with_members(node, &pairs) {
                Node::Exists(m, w, p) => {
                    Node::Exists(m, self.substituted_term(w, sigma, &mut replaced), p)
                }
                Node::Forall(m, a, p) => match sigma.get(&a) {
                    Some(&term) => {
                        let FoTerm::Eigen(b) = self.term(term) else {
                            unreachable!("a `∀` step's eigenvariable is renamed to one")
                        };
                        Node::Forall(m, b, p)
                    }
                    None => Node::Forall(m, a, p),
                },
                node => node,
            };
            let renamed = renamed.map_premises(|p| made.get(&p).copied().unwrap_or(p));
            let new = self.push(renamed);
            made.insert(NodeId::new(id as u32), new);
        }
        Ok(made.get(&top).copied().unwrap_or(top))
    }

    /// Refuses `more` nodes added to the arena past a step's bounds: an
    /// arena, the proof's nodes and those the step made, of more than
    /// `most` nodes, or of more bytes than `memory` at sixteen a node. The
    /// proof made has at most the arena's nodes, so a step within the
    /// bound makes a proof within it; one whose copies take the place of
    /// subproofs it no longer reads may be refused though the proof it
    /// would make is smaller.
    pub(super) fn afford(&self, more: usize) -> Result<(), Refusal> {
        self.within_ids()?;
        if self.nodes.len().saturating_add(more) > self.most {
            return Err(Refusal::Elimination {
                what: crate::limits::Bound::Nodes,
                limit: self.most as u64,
            });
        }
        let bytes =
            (self.nodes.len().saturating_add(more) as u64).saturating_mul(size_of::<Node>() as u64);
        match self.memory {
            Some(limit) if bytes > limit => Err(Refusal::Memory {
                phase: Phase::Eliminate,
                limit_bytes: limit,
                needed_bytes: Some(bytes),
            }),
            _ => Ok(()),
        }
    }

    /// Returns whether the frame of `m` names an eigenvariable `sigma`
    /// maps.
    fn names_member(&self, m: Member, sigma: &HashMap<Eigen, FoTermId>, named: &mut Named) -> bool {
        let table = self.table.as_ref().expect("a proof with instances");
        let Some((_, frame)) = table.instances.instance(m) else {
            return false;
        };
        let mut chain = Vec::new();
        let mut next = Some(frame);
        let mut found = false;
        while let Some(f) = next {
            if let Some(&known) = named.frames.get(&f) {
                found = known;
                break;
            }
            chain.push(f);
            next = table.instances.frame_parts(f).0;
        }
        for f in chain.into_iter().rev() {
            found = found || self.names_term(table.instances.frame_parts(f).1, sigma, named);
            named.frames.insert(f, found);
        }
        found
    }

    /// Returns whether the term `id` names an eigenvariable `sigma` maps,
    /// by a walk with a stack of its own.
    fn names_term(
        &self,
        id: FoTermId,
        sigma: &HashMap<Eigen, FoTermId>,
        named: &mut Named,
    ) -> bool {
        let table = self.table.as_ref().expect("a proof with instances");
        let terms = table.terms(self.proof);
        let base = table.instances.base_terms as usize;
        let mut walk = vec![(id, false)];
        while let Some((t, expanded)) = walk.pop() {
            if t.index() < base || named.terms.contains_key(&t) {
                continue;
            }
            let found = match terms.term(t) {
                FoTerm::Eigen(a) => sigma.contains_key(&a),
                FoTerm::App(_, args) => {
                    let list = terms.list(args);
                    if !expanded {
                        walk.push((t, true));
                        walk.extend(list.iter().map(|&a| (a, false)));
                        continue;
                    }
                    list.iter()
                        .any(|a| named.terms.get(a).copied().unwrap_or(false))
                }
                FoTerm::Bound(_) | FoTerm::Meta(_) => false,
            };
            named.terms.insert(t, found);
        }
        named.terms.get(&id).copied().unwrap_or(false)
    }

    /// Returns the term `id` of the sequent or the table.
    fn term(&self, id: FoTermId) -> FoTerm {
        let table = self.table.as_ref().expect("a proof with instances");
        table.terms(self.proof).term(id)
    }

    /// Returns `m` with `sigma` applied to its frame.
    fn substituted_member(
        &mut self,
        m: Member,
        sigma: &HashMap<Eigen, FoTermId>,
        replaced: &mut Replaced,
    ) -> Member {
        if let Some(&done) = replaced.members.get(&m) {
            return done;
        }
        let (o, frame) = self.table().resolve(m);
        let made = match frame {
            None => m,
            Some(frame) => {
                let new = self.substituted_frame(frame, sigma, replaced);
                if new == frame {
                    m
                } else {
                    self.table().member(o, Some(new))
                }
            }
        };
        replaced.members.insert(m, made);
        made
    }

    /// Returns `frame` with `sigma` applied to its terms: the cells up to
    /// the first one replaced before, then made downwards.
    fn substituted_frame(
        &mut self,
        frame: FrameId,
        sigma: &HashMap<Eigen, FoTermId>,
        replaced: &mut Replaced,
    ) -> FrameId {
        let mut chain = Vec::new();
        let mut next = Some(frame);
        let mut below: Option<FrameId> = None;
        while let Some(f) = next {
            if let Some(&done) = replaced.frames.get(&f) {
                below = Some(done);
                break;
            }
            chain.push(f);
            next = self.table().instances.frame_parts(f).0;
        }
        for f in chain.into_iter().rev() {
            let (parent, term) = self.table().instances.frame_parts(f);
            let term_made = self.substituted_term(term, sigma, replaced);
            let made = if below == parent && term_made == term {
                f
            } else {
                self.table().frame(below, term_made)
            };
            replaced.frames.insert(f, made);
            below = Some(made);
        }
        below.expect("a frame holds a term")
    }

    /// Returns the term `id` with `sigma` applied, by a walk with a stack
    /// of its own, since a witness may be nested as deep as a search's
    /// bindings went. The sequent's own terms name no eigenvariable.
    fn substituted_term(
        &mut self,
        id: FoTermId,
        sigma: &HashMap<Eigen, FoTermId>,
        replaced: &mut Replaced,
    ) -> FoTermId {
        let base = self.table().instances.base_terms as usize;
        let mut walk = vec![(id, false)];
        while let Some((t, expanded)) = walk.pop() {
            if t.index() < base || replaced.terms.contains_key(&t) {
                continue;
            }
            let made = match self.term(t) {
                FoTerm::Eigen(a) => sigma.get(&a).copied().unwrap_or(t),
                FoTerm::App(f, args) => {
                    let table = self.table.as_ref().expect("a proof with instances");
                    let list: Vec<FoTermId> = table.terms(self.proof).list(args).to_vec();
                    if !expanded {
                        walk.push((t, true));
                        walk.extend(list.iter().map(|&a| (a, false)));
                        continue;
                    }
                    let args: Vec<FoTermId> = list
                        .iter()
                        .map(|a| replaced.terms.get(a).copied().unwrap_or(*a))
                        .collect();
                    if args == list {
                        t
                    } else {
                        self.table().instances.apply(f, &args)
                    }
                }
                FoTerm::Bound(_) | FoTerm::Meta(_) => t,
            };
            replaced.terms.insert(t, made);
        }
        replaced.terms.get(&id).copied().unwrap_or(id)
    }

    /// Gives every eigenvariable that two `∀` steps of the proof at `root`
    /// introduce a step of its own, and returns the root. A step remakes
    /// a `∀` step that a subproof it changes holds, while the original may
    /// still be read elsewhere; the later of two such steps, or the
    /// earlier where it lies above the later, is then made anew with a
    /// fresh eigenvariable throughout its subproof, and so is every `∀`
    /// step of that subproof that the proof still reads elsewhere. Each
    /// round takes one introduction away from an eigenvariable two steps
    /// introduce, and gives none to another. Every node a round makes is
    /// charged before it is made ([`afford`](Self::afford)), and the stop
    /// is asked each round.
    pub(super) fn apart(
        &mut self,
        mut root: NodeId,
        stop: &mut dyn FnMut(Progress) -> bool,
    ) -> Result<NodeId, Refusal> {
        let mut rounds: u64 = 0;
        loop {
            let reach = reached(&self.nodes, root, None);
            let mut first: HashMap<Eigen, NodeId> = HashMap::default();
            let mut twins = None;
            for (i, _) in reach.iter().enumerate().filter(|&(_, &r)| r) {
                if let Node::Forall(_, a, _) = self.nodes[i] {
                    let id = NodeId::new(i as u32);
                    if let Some(&earlier) = first.get(&a) {
                        twins = Some((earlier, id));
                        break;
                    }
                    first.insert(a, id);
                }
            }
            let Some((earlier, later)) = twins else {
                return Ok(root);
            };
            rounds += 1;
            if stop(Progress::new(Phase::Eliminate, 1, rounds)) {
                return Err(Refusal::Stopped {
                    phase: Phase::Eliminate,
                });
            }
            let Node::Forall(_, _, premise) = self.nodes[later.index()] else {
                unreachable!("a twin is a `∀` step")
            };
            let renamed = if reached(&self.nodes, premise, None)[earlier.index()] {
                earlier
            } else {
                later
            };
            let Node::Forall(m, a, premise) = self.nodes[renamed.index()] else {
                unreachable!("a twin is a `∀` step")
            };
            // The `∀` steps above that the proof reads besides through the
            // one renamed keep their eigenvariables there, so their copies
            // take fresh ones.
            let elsewhere = reached(&self.nodes, root, Some(renamed));
            let above = reached(&self.nodes, premise, None);
            let mut sigma: HashMap<Eigen, FoTermId> = HashMap::default();
            let fresh = self.table().fresh()?;
            let fresh_term = self.table().eigen(fresh);
            sigma.insert(a, fresh_term);
            for (i, _) in above.iter().enumerate().filter(|&(_, &r)| r) {
                if let Node::Forall(_, b, _) = self.nodes[i]
                    && elsewhere[i]
                {
                    let fresh = self.table().fresh()?;
                    let term = self.table().eigen(fresh);
                    sigma.insert(b, term);
                }
            }
            let premise = self.substitute(premise, &sigma)?;
            // The step renamed, and every node below it that reaches it.
            let mut below = vec![false; root.index() + 1];
            below[renamed.index()] = true;
            for i in renamed.index() + 1..=root.index() {
                below[i] = self.nodes[i].premises().any(|p| below[p.index()]);
            }
            self.afford(below.iter().filter(|&&b| b).count())?;
            let made = self.push(Node::Forall(m, fresh, premise));
            root = self.replace(renamed, made, root);
        }
    }

    /// Returns a table that holds only what the nodes `root` reaches name,
    /// with those nodes' members and witnesses renumbered into it: a
    /// step's table extends the proof's, and what the step left unread is
    /// not carried to the next.
    pub(super) fn compact(&mut self, root: NodeId) -> Instances {
        let table = self.table.as_ref().expect("a proof with instances");
        let mut copy = Transfer {
            terms: table.terms(self.proof),
            table,
            out: Instances::new(self.forest),
            term_at: HashMap::default(),
            frame_at: HashMap::default(),
            member_at: HashMap::default(),
        };
        let reach = reached(&self.nodes, root, None);
        for (i, _) in reach.iter().enumerate().filter(|&(_, &r)| r) {
            let node = self.nodes[i];
            let pairs = node.members().map(|m| (m, copy.member(m)));
            let pairs: Vec<(Member, Member)> = pairs.collect();
            let node = with_members(node, &pairs);
            self.nodes[i] = match node {
                Node::Exists(m, w, p) => Node::Exists(m, copy.term(w), p),
                node => node,
            };
        }
        copy.out
    }
}

/// Returns `node` with each member it names replaced by its pair's second
/// member.
fn with_members(node: Node, pairs: &[(Member, Member)]) -> Node {
    node.renamed(|m| {
        Ok::<_, std::convert::Infallible>(
            pairs
                .iter()
                .find(|&&(from, _)| from == m)
                .map_or(m, |&(_, to)| to),
        )
    })
    .unwrap_or_else(|never| match never {})
}

/// Returns, for every node of `nodes` up to `top`, whether `top` reaches
/// it, not passing beyond `avoid` where one is given.
fn reached(nodes: &[Node], top: NodeId, avoid: Option<NodeId>) -> Vec<bool> {
    let mut reached = vec![false; top.index() + 1];
    reached[top.index()] = true;
    for i in (0..=top.index()).rev() {
        if reached[i] && Some(NodeId::new(i as u32)) != avoid {
            for p in nodes[i].premises() {
                reached[p.index()] = true;
            }
        }
    }
    reached
}

/// The copy of what a proof's nodes name into a table of their own.
struct Transfer<'t> {
    /// The terms of the table copied from, with the sequent's.
    terms: Terms<'t>,
    /// The table copied from.
    table: &'t Table,
    /// The table made.
    out: Instances,
    /// Each term copied, by its id before.
    term_at: HashMap<FoTermId, FoTermId>,
    /// Each frame copied.
    frame_at: HashMap<FrameId, FrameId>,
    /// Each member copied.
    member_at: HashMap<Member, Member>,
}

impl Transfer<'_> {
    /// Returns the id in the table made of the term `id`, by a walk with a
    /// stack of its own; the sequent's terms keep their ids.
    fn term(&mut self, id: FoTermId) -> FoTermId {
        let base = self.out.base_terms as usize;
        let mut walk = vec![(id, false)];
        while let Some((t, expanded)) = walk.pop() {
            if t.index() < base || self.term_at.contains_key(&t) {
                continue;
            }
            let made = match self.terms.term(t) {
                FoTerm::App(f, args) => {
                    let list = self.terms.list(args);
                    if !expanded {
                        walk.push((t, true));
                        walk.extend(list.iter().map(|&a| (a, false)));
                        continue;
                    }
                    let args: Vec<FoTermId> = list
                        .iter()
                        .map(|a| self.term_at.get(a).copied().unwrap_or(*a))
                        .collect();
                    self.out.apply(f, &args)
                }
                term @ (FoTerm::Eigen(_) | FoTerm::Bound(_) | FoTerm::Meta(_)) => {
                    self.out.push_term(term)
                }
            };
            self.term_at.insert(t, made);
        }
        self.term_at.get(&id).copied().unwrap_or(id)
    }

    /// Returns the frame of the table made for `frame`: the cells up to
    /// the first copied before, then made downwards.
    fn frame(&mut self, frame: FrameId) -> FrameId {
        let mut chain = Vec::new();
        let mut next = Some(frame);
        let mut below = None;
        while let Some(f) = next {
            if let Some(&done) = self.frame_at.get(&f) {
                below = Some(done);
                break;
            }
            chain.push(f);
            next = self.table.instances.frame_parts(f).0;
        }
        for f in chain.into_iter().rev() {
            let term = self.term(self.table.instances.frame_parts(f).1);
            let made = self.out.frame(below, term);
            self.frame_at.insert(f, made);
            below = Some(made);
        }
        below.expect("a frame holds a term")
    }

    /// Returns the member of the table made for `m`.
    fn member(&mut self, m: Member) -> Member {
        let (o, Some(frame)) = self.table.resolve(m) else {
            return m;
        };
        if let Some(&done) = self.member_at.get(&m) {
            return done;
        }
        let frame = self.frame(frame);
        let made = self.out.member(o, frame);
        self.member_at.insert(m, made);
        made
    }
}
