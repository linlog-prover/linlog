// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! A rule applied forward: its term evaluated over the sequents in its
//! holes, step by step, to the sequent it derives, or to nothing where a
//! step's side condition fails. A step derives what the checker derives of
//! that rule: a member a step takes may be absent from a premise that a
//! `⊤` lets absorb anything (the `any` flag of a sequent, here `top`), or,
//! in affine mode, weakened in below the step. With a node arena given, the
//! same evaluation writes the proof term, a node per step and a `Weaken`
//! per member weakened in, so that the proof the search returns is the one
//! it judged.

use super::rules::{Partner, Shape, Step};
use crate::occurrences::{Member, OccId};
use crate::proofs::{Node, NodeId};

/// A sequent as the search derives it: the linear zone sorted with its
/// repeats, the unrestricted zone sorted, whether a `⊤` above lets it
/// absorb any further linear member, how many members are in output
/// position, and the node proving it while a proof is written.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct Zone {
    /// The linear zone, sorted, with repeats.
    pub(super) gamma: Vec<OccId>,
    /// The unrestricted zone, sorted, without repeats.
    pub(super) theta: Vec<OccId>,
    /// Whether any further linear member is absorbed.
    pub(super) top: bool,
    /// The members of `gamma` in output position: a `u64`, since it never
    /// exceeds the occurrences the zones in memory hold together (each
    /// four bytes), below 2⁶² on any target, so no sum of two wraps.
    pub(super) outputs: u64,
    /// The node proving the sequent, while a proof is written.
    pub(super) node: Option<NodeId>,
}

/// Inserts into a sorted list, keeping repeats.
pub(super) fn insert(list: &mut Vec<OccId>, o: OccId) {
    let at = list.partition_point(|&x| x <= o);
    list.insert(at, o);
}

/// Removes one copy from a sorted list, returning whether there was one.
fn remove(list: &mut Vec<OccId>, o: OccId) -> bool {
    match list.binary_search(&o) {
        Ok(at) => {
            list.remove(at);
            true
        }
        Err(_) => false,
    }
}

/// Merges `from` into `into`, both sorted, keeping repeats: the sum of two
/// multisets.
fn sum(into: &mut Vec<OccId>, from: &[OccId]) {
    if from.is_empty() {
        return;
    }
    let mut merged = Vec::with_capacity(into.len() + from.len());
    let (mut i, mut j) = (0, 0);
    while i < into.len() && j < from.len() {
        if into[i] <= from[j] {
            merged.push(into[i]);
            i += 1;
        } else {
            merged.push(from[j]);
            j += 1;
        }
    }
    merged.extend_from_slice(&into[i..]);
    merged.extend_from_slice(&from[j..]);
    *into = merged;
}

/// Merges `from` into `into`, both sorted sets: their union.
fn union(into: &mut Vec<OccId>, from: &[OccId]) {
    for &o in from {
        if let Err(at) = into.binary_search(&o) {
            into.insert(at, o);
        }
    }
}

/// Returns the members of `of` beyond those of `within`, with repeats: what
/// `within` lacks to hold `of`, both sorted multisets.
pub(super) fn excess(of: &[OccId], within: &[OccId]) -> Vec<OccId> {
    let mut out = Vec::new();
    let mut j = 0;
    for &o in of {
        while j < within.len() && within[j] < o {
            j += 1;
        }
        if j < within.len() && within[j] == o {
            j += 1;
        } else {
            out.push(o);
        }
    }
    out
}

/// Returns whether one sorted multiset lies within another.
pub(super) fn within(small: &[OccId], large: &[OccId]) -> bool {
    let mut j = 0;
    for &o in small {
        while j < large.len() && large[j] < o {
            j += 1;
        }
        if j == large.len() || large[j] != o {
            return false;
        }
        j += 1;
    }
    true
}

/// The rules of the calculus the search derives in, read forward.
pub(super) struct Calculus<'s, 'a> {
    /// The goal's shape: the forest, the reading and the mode.
    pub(super) shape: &'s Shape<'a>,
}

impl Calculus<'_, '_> {
    /// Whether a zone is a sequent of the mode: under a reading, one
    /// member in output position, or none where `⊤` absorbs one.
    fn shaped(&self, z: &Zone) -> bool {
        self.shape.reading.is_none() || z.outputs == 1 || (z.outputs == 0 && z.top)
    }

    /// Adds a member to the linear zone.
    fn add(&self, z: &mut Zone, o: OccId) {
        insert(&mut z.gamma, o);
        z.outputs += u64::from(self.shape.output(o));
    }

    /// Weakens a member in below a premise's node: in affine mode, never
    /// an output under a reading.
    fn weaken(&self, z: &mut Zone, o: OccId, nodes: &mut Option<&mut Vec<Node>>) -> bool {
        if !self.shape.affine || self.shape.output(o) {
            return false;
        }
        if let (Some(nodes), Some(n)) = (nodes.as_deref_mut(), z.node) {
            z.node = Some(push(nodes, Node::Weaken(Member::from(o), n)));
        }
        true
    }

    /// Weakens a member into a zone, writing the `Weaken` node: in affine
    /// mode, never an output under a reading.
    pub(super) fn weaken_in(&self, z: &mut Zone, o: OccId, nodes: &mut Vec<Node>) -> bool {
        if !self.weaken(z, o, &mut Some(nodes)) {
            return false;
        }
        self.add(z, o);
        true
    }

    /// Takes a principal's subformula out of a premise's linear zone: it
    /// is there, or the premise absorbs it (under a reading, an output only
    /// when the premise has none), or it is weakened in.
    fn take(&self, z: &mut Zone, o: OccId, nodes: &mut Option<&mut Vec<Node>>) -> bool {
        if remove(&mut z.gamma, o) {
            z.outputs -= u64::from(self.shape.output(o));
            return true;
        }
        if z.top {
            return !(self.shape.output(o) && z.outputs > 0);
        }
        self.weaken(z, o, nodes)
    }

    /// Joins the two premises of a `&`, their subformulas taken: equal
    /// zones, or an absorbing zone within an exact one (the exact one
    /// stays), or both absorbing (the larger count of each member); in
    /// affine mode the exact sides are weakened to what both hold. Returns
    /// the joined zone, and the two premises' nodes while a proof is
    /// written.
    fn join(
        &self,
        mut l: Zone,
        mut r: Zone,
        nodes: &mut Option<&mut Vec<Node>>,
    ) -> Option<(Zone, Option<(NodeId, NodeId)>)> {
        let mut most = l.gamma.clone();
        sum(&mut most, &excess(&r.gamma, &l.gamma));
        let gamma = match (l.top, r.top) {
            (true, true) => most,
            (false, false) if l.gamma == r.gamma => most,
            (true, false) if within(&l.gamma, &r.gamma) => most,
            (false, true) if within(&r.gamma, &l.gamma) => most,
            _ if self.shape.affine => {
                for side in [&mut l, &mut r] {
                    if !side.top {
                        for o in excess(&most, &side.gamma) {
                            if !self.weaken(side, o, nodes) {
                                return None;
                            }
                        }
                    }
                }
                most
            }
            _ => return None,
        };
        let mut theta = l.theta;
        union(&mut theta, &r.theta);
        let outputs = gamma.iter().map(|&o| u64::from(self.shape.output(o))).sum();
        let zone = Zone {
            gamma,
            theta,
            top: l.top && r.top,
            outputs,
            node: None,
        };
        Some((zone, l.node.zip(r.node)))
    }

    /// Evaluates a rule's steps over the premises (`premise` gives the
    /// zone of a hole) and the partners chosen (`partner` gives the
    /// partner of a slot), and returns the sequent derived, or `None`
    /// where a step's condition fails, adding to `units` one for each step
    /// and each linear member of the zone it yields, which the step copied
    /// or merged. With `nodes`, writes the proof term into it, the
    /// premises' zones carrying their nodes.
    pub(super) fn derive(
        &self,
        steps: &[Step],
        premise: &mut dyn FnMut(u32, &mut Zone),
        partner: &dyn Fn(u32) -> Partner,
        mut nodes: Option<&mut Vec<Node>>,
        stack: &mut Vec<Zone>,
        units: &mut u64,
    ) -> Option<Zone> {
        let forest = self.shape.forest;
        stack.clear();
        for &step in steps {
            let z = match step {
                Step::Axiom(p, slot) => {
                    let Partner { x, theta } = partner(slot);
                    let mut z = Zone::default();
                    self.add(&mut z, p);
                    let ax = nodes
                        .as_deref_mut()
                        .map(|n| push(n, Node::Ax(Member::from(p), Member::from(x))));
                    if theta {
                        z.theta.push(x);
                        z.node = ax
                            .zip(nodes.as_deref_mut())
                            .map(|(a, n)| push(n, Node::Copy(Member::from(x), a)));
                    } else {
                        self.add(&mut z, x);
                        z.node = ax;
                    }
                    z
                }
                Step::One(o) => {
                    let mut z = Zone::default();
                    self.add(&mut z, o);
                    z.node = nodes
                        .as_deref_mut()
                        .map(|n| push(n, Node::One(Member::from(o))));
                    z
                }
                Step::Top(o) => {
                    let mut z = Zone::default();
                    self.add(&mut z, o);
                    z.top = true;
                    z.node = nodes
                        .as_deref_mut()
                        .map(|n| push(n, Node::Top(Member::from(o))));
                    z
                }
                Step::Hole(h) => {
                    let mut z = Zone::default();
                    premise(h, &mut z);
                    z
                }
                Step::Tensor(o) => {
                    let mut r = stack.pop()?;
                    let mut l = stack.pop()?;
                    let (a, b) = (forest.left(o)?, forest.right(o)?);
                    if !self.take(&mut l, a, &mut nodes) || !self.take(&mut r, b, &mut nodes) {
                        return None;
                    }
                    sum(&mut l.gamma, &r.gamma);
                    union(&mut l.theta, &r.theta);
                    l.top |= r.top;
                    l.outputs += r.outputs;
                    l.node = match (nodes.as_deref_mut(), l.node, r.node) {
                        (Some(n), Some(x), Some(y)) => {
                            Some(push(n, Node::Tensor(Member::from(o), x, y)))
                        }
                        _ => None,
                    };
                    self.add(&mut l, o);
                    l
                }
                Step::Plus(o, side) => {
                    let mut z = stack.pop()?;
                    let child = match side {
                        crate::proofs::Branch::Left => forest.left(o)?,
                        crate::proofs::Branch::Right => forest.right(o)?,
                    };
                    if !self.take(&mut z, child, &mut nodes) {
                        return None;
                    }
                    z.node = wrap(&mut nodes, z.node, |n| Node::Plus(Member::from(o), side, n));
                    self.add(&mut z, o);
                    z
                }
                Step::Bang(o) => {
                    let mut z = stack.pop()?;
                    if !self.take(&mut z, forest.left(o)?, &mut nodes) || !z.gamma.is_empty() {
                        return None;
                    }
                    z.top = false;
                    z.outputs = 0;
                    z.node = wrap(&mut nodes, z.node, |n| Node::Bang(Member::from(o), n));
                    self.add(&mut z, o);
                    z
                }
                Step::Par(o) => {
                    let mut z = stack.pop()?;
                    let (a, b) = (forest.left(o)?, forest.right(o)?);
                    if !self.take(&mut z, a, &mut nodes) || !self.take(&mut z, b, &mut nodes) {
                        return None;
                    }
                    z.node = wrap(&mut nodes, z.node, |n| Node::Par(Member::from(o), n));
                    self.add(&mut z, o);
                    z
                }
                Step::Bot(o) => {
                    let mut z = stack.pop()?;
                    z.node = wrap(&mut nodes, z.node, |n| Node::Bot(Member::from(o), n));
                    self.add(&mut z, o);
                    z
                }
                Step::With(o) => {
                    let mut r = stack.pop()?;
                    let mut l = stack.pop()?;
                    let (a, b) = (forest.left(o)?, forest.right(o)?);
                    if !self.take(&mut l, a, &mut nodes) || !self.take(&mut r, b, &mut nodes) {
                        return None;
                    }
                    let (mut z, premises) = self.join(l, r, &mut nodes)?;
                    z.node = match (nodes.as_deref_mut(), premises) {
                        (Some(n), Some((x, y))) => Some(push(n, Node::With(Member::from(o), x, y))),
                        _ => None,
                    };
                    self.add(&mut z, o);
                    z
                }
                Step::Quest(o) => {
                    let mut z = stack.pop()?;
                    let a = forest.left(o)?;
                    if let Ok(at) = z.theta.binary_search(&a) {
                        z.theta.remove(at);
                    }
                    z.node = wrap(&mut nodes, z.node, |n| Node::Quest(Member::from(o), n));
                    self.add(&mut z, o);
                    z
                }
                Step::Copy(o) => {
                    let mut z = stack.pop()?;
                    if !self.take(&mut z, o, &mut nodes) {
                        return None;
                    }
                    if let Err(at) = z.theta.binary_search(&o) {
                        z.theta.insert(at, o);
                    }
                    z.node = wrap(&mut nodes, z.node, |n| Node::Copy(Member::from(o), n));
                    z
                }
            };
            *units += 1 + z.gamma.len() as u64;
            if !self.shaped(&z) {
                return None;
            }
            stack.push(z);
        }
        let z = stack.pop()?;
        debug_assert!(stack.is_empty(), "a rule's term has one root");
        Some(z)
    }

    /// Mix: the two sequents side by side.
    pub(super) fn mix(&self, l: &Zone, r: &Zone, nodes: Option<&mut Vec<Node>>) -> Option<Zone> {
        let mut z = l.clone();
        sum(&mut z.gamma, &r.gamma);
        union(&mut z.theta, &r.theta);
        z.top |= r.top;
        z.outputs += r.outputs;
        z.node = match (nodes, l.node, r.node) {
            (Some(n), Some(a), Some(b)) => Some(push(n, Node::Mix(a, b))),
            _ => None,
        };
        Some(z)
    }
}

/// Pushes a node, returning its id.
fn push(nodes: &mut Vec<Node>, node: Node) -> NodeId {
    let id = NodeId::new(u32::try_from(nodes.len()).expect("a proof has fewer than 2³² nodes"));
    nodes.push(node);
    id
}

/// Wraps a premise's node in a unary rule's node, while a proof is written.
fn wrap(
    nodes: &mut Option<&mut Vec<Node>>,
    premise: Option<NodeId>,
    rule: impl FnOnce(NodeId) -> Node,
) -> Option<NodeId> {
    match (nodes.as_deref_mut(), premise) {
        (Some(n), Some(p)) => Some(push(n, rule(p))),
        _ => None,
    }
}
