// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Proofs: terms over occurrence ids, the checker that validates a term
//! against its sequent independently of any engine, and the derivation view
//! for humans and exporters.
//!
//! A [`Proof`] is an arena of [`Node`]s, one per rule instance, over the
//! [`Forest`] of the sequent it proves. A node names the rule, the
//! occurrence the rule acts on and the nodes that prove its premises. The
//! rules are those of the dyadic sequent calculus every engine searches in:
//! the standard rules, with the `?` formulas handled through an unrestricted
//! zone `Θ` (a `?` step moves a formula into it, a copy uses it without
//! consuming it), plus weakening for affine mode and Mix. A node does not
//! record the sequent it proves: the checker derives that from the premises,
//! and the derivation view shows it.

/// The checker.
pub mod check;
/// The derivation view.
pub mod derivation;
/// Text rendering of derivations.
pub(crate) mod fmt;
/// Step-by-step proving.
#[cfg(feature = "interactive")]
pub mod interactive;
/// Multisets of occurrence ids.
mod multiset;
/// The checker's first implementation, which the tests compare it with.
#[cfg(test)]
mod oracle;
/// The size of a derivation, without building it.
/// The rules of the standard calculus, one-sided and named two-sided.
mod rule;
pub mod size;
/// The labels of rules, the shape of an open goal, and why a derivation was
/// not written whole, which every output that draws a derivation shares.
pub mod style;

pub use check::{CheckError, Dyadic, Fault, Invalid, Refused};
pub use derivation::{Compact, Derivation, InfId, Inference, ViewOptions};
pub use fmt::TextOptions;
#[cfg(feature = "interactive")]
pub use interactive::{Interactive, StepError};
pub use rule::{Named, Rule};
pub use size::Size;
pub use style::{Labels, OpenGoal};

use crate::Error;
use crate::fragment::Mode;
use crate::limits::{Limits, Progress};
use crate::occurrences::{Forest, Member, OccId};
use crate::sequents::Sequent;
use std::fmt::{Display, Formatter, Result as FmtResult};

/// A number of bytes, written in the largest binary unit that divides it:
/// `1 GiB`, `512 MiB`, `1500 B`.
pub(crate) struct Bytes(pub(crate) u64);

impl std::fmt::Display for Bytes {
    /// Writes the number with its unit.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let units = [(30, "GiB"), (20, "MiB"), (10, "KiB")];
        match units
            .into_iter()
            .find(|&(shift, _)| self.0 != 0 && self.0.is_multiple_of(1 << shift))
        {
            Some((shift, unit)) => write!(f, "{} {unit}", self.0 >> shift),
            None => write!(f, "{} B", self.0),
        }
    }
}

/// The index of a node in a proof's arena.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(u32);

impl NodeId {
    /// Wraps a raw arena index.
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    /// Returns the raw arena index.
    pub const fn get(self) -> u32 {
        self.0
    }

    /// Returns the index as a `usize`, for indexing the arena.
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

/// Which subformula of a binary connective a rule picks.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Branch {
    /// The left subformula.
    Left,
    /// The right subformula.
    Right,
}

/// One rule instance of a proof: the rule, the member it acts on, and the
/// nodes that prove its premises, which precede it in the arena. The rules
/// are the dyadic calculus's, so `⊢ Θ ; Γ` below is a sequent with the
/// unrestricted zone `Θ` and the linear zone `Γ`; `Θ` is empty until a
/// [`Quest`](Self::Quest) fills it.
///
/// The type is closed: a new rule is a new release of the calculus, with
/// its tag, its checker's arm, its rule of the derivation view and its
/// constructor wherever the type is mirrored ([`NAMES`](Self::NAMES)).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Node {
    /// The axiom `⊢ Θ ; a, ~a` on two literal occurrences of one atom with
    /// opposite signs, in either order.
    Ax(Member, Member),
    /// `⊢ Θ ; Γ, Δ, A ⊗ B` from `⊢ Θ ; Γ, A` and `⊢ Θ ; Δ, B`, on the
    /// occurrence of `A ⊗ B`; the premises prove its left and right
    /// subformula, in that order.
    Tensor(Member, NodeId, NodeId),
    /// `⊢ Θ ; Γ, A ⅋ B` from `⊢ Θ ; Γ, A, B`.
    Par(Member, NodeId),
    /// `⊢ Θ ; 1`.
    One(Member),
    /// `⊢ Θ ; Γ, ⊥` from `⊢ Θ ; Γ`.
    Bot(Member, NodeId),
    /// `⊢ Θ ; Γ, A & B` from `⊢ Θ ; Γ, A` and `⊢ Θ ; Γ, B`, the premises in
    /// that order.
    With(Member, NodeId, NodeId),
    /// `⊢ Θ ; Γ, A ⊕ B` from `⊢ Θ ; Γ, A` or from `⊢ Θ ; Γ, B`, as the side
    /// says.
    Plus(Member, Branch, NodeId),
    /// `⊢ Θ ; Γ, ⊤` for any `Γ`.
    Top(Member),
    /// Promotion: `⊢ Θ ; !A` from `⊢ Θ ; A`, with an empty linear zone.
    Bang(Member, NodeId),
    /// `⊢ Θ ; Γ, ?A` from `⊢ Θ, A ; Γ`: the subformula moves into the
    /// unrestricted zone.
    Quest(Member, NodeId),
    /// A copy: `⊢ Θ, A ; Γ` from `⊢ Θ, A ; Γ, A`, on the occurrence of `A`,
    /// which stays in `Θ`. The only rule that can repeat an occurrence in a
    /// branch, which is why the linear zone is a multiset.
    Copy(Member, NodeId),
    /// Weakening: `⊢ Θ ; Γ, A` from `⊢ Θ ; Γ`, in affine mode, or in any
    /// mode when `A` is a `?` formula (the standard `?w`; the dyadic form of
    /// it is a [`Quest`](Self::Quest) whose formula goes unused).
    Weaken(Member, NodeId),
    /// Mix, when the mode allows it: `⊢ Θ ; Γ, Δ` from `⊢ Θ ; Γ` and
    /// `⊢ Θ ; Δ`.
    Mix(NodeId, NodeId),
}

/// A node is a tag, an occurrence and two node indices at most.
const _: () = assert!(size_of::<Node>() == 16);

impl Node {
    /// The variants' names, in their order: what a mirror of the type in
    /// another language matches, one constructor each.
    pub const NAMES: &'static [&'static str] = &[
        "Ax", "Tensor", "Par", "One", "Bot", "With", "Plus", "Top", "Bang", "Quest", "Copy",
        "Weaken", "Mix",
    ];

    /// The rules' tags in a proof's JSON form and their names
    /// ([`name`](Self::name)): one per variant, and two for `Plus`, whose
    /// branch is in its tag.
    pub const TAGS: &'static [&'static str] = &[
        "ax", "⊗", "⅋", "1", "⊥", "&", "⊕₁", "⊕₂", "⊤", "!", "?", "copy", "wk", "mix",
    ];

    /// Returns the member the rule acts on: `None` for Mix, and the first
    /// literal of an axiom.
    pub const fn principal(self) -> Option<Member> {
        use Node::*;
        match self {
            Ax(o, _)
            | Tensor(o, ..)
            | Par(o, _)
            | One(o)
            | Bot(o, _)
            | With(o, ..)
            | Plus(o, ..)
            | Top(o)
            | Bang(o, _)
            | Quest(o, _)
            | Copy(o, _)
            | Weaken(o, _) => Some(o),
            Mix(..) => None,
        }
    }

    /// Returns the members the node names: one, or two for an axiom, or
    /// none for Mix.
    pub fn members(self) -> impl Iterator<Item = Member> {
        let second = match self {
            Node::Ax(_, b) => Some(b),
            _ => None,
        };
        self.principal().into_iter().chain(second)
    }

    /// Returns the nodes that prove the premises, in the rule's order: none
    /// for an axiom, `1` or `⊤`, two for `⊗`, `&` and Mix, one otherwise.
    pub fn premises(self) -> impl Iterator<Item = NodeId> {
        use Node::*;
        let (first, second) = match self {
            Ax(..) | One(_) | Top(_) => (None, None),
            Tensor(_, l, r) | With(_, l, r) | Mix(l, r) => (Some(l), Some(r)),
            Par(_, p)
            | Bot(_, p)
            | Plus(_, _, p)
            | Bang(_, p)
            | Quest(_, p)
            | Copy(_, p)
            | Weaken(_, p) => (Some(p), None),
        };
        first.into_iter().chain(second)
    }

    /// Returns the rule's name: `ax`, `⊗`, `⅋`, `1`, `⊥`, `&`, `⊕₁`, `⊕₂`,
    /// `⊤`, `!`, `?`, `copy`, `wk` or `mix`.
    pub const fn name(self) -> &'static str {
        use Node::*;
        match self {
            Ax(..) => "ax",
            Tensor(..) => "⊗",
            Par(..) => "⅋",
            One(_) => "1",
            Bot(..) => "⊥",
            With(..) => "&",
            Plus(_, Branch::Left, _) => "⊕₁",
            Plus(_, Branch::Right, _) => "⊕₂",
            Top(_) => "⊤",
            Bang(..) => "!",
            Quest(..) => "?",
            Copy(..) => "copy",
            Weaken(..) => "wk",
            Mix(..) => "mix",
        }
    }
}

impl Display for Node {
    /// Writes the rule's name, then `on` the occurrences and `from` the
    /// premise nodes, as in `⊗ on 1 from 0, 2`.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(self.name())?;
        let mut occurrences = self.members().map(Member::get);
        if let Some(o) = occurrences.next() {
            write!(f, " on {o}")?;
            for o in occurrences {
                write!(f, ", {o}")?;
            }
        }
        let mut premises = self.premises().map(|p| p.get());
        if let Some(p) = premises.next() {
            write!(f, " from {p}")?;
            for p in premises {
                write!(f, ", {p}")?;
            }
        }
        Ok(())
    }
}

/// A proof of a sequent: an arena of rule instances over the sequent's
/// occurrence forest, which the proof owns. Every node refers only to nodes
/// before it, the last node is the root, and every node is reachable from
/// the root; a subproof that two nodes share is stored once.
///
/// A proof is built bottom-up, premises before conclusions, and is what a
/// search engine returns. Whether it proves its sequent is a separate
/// question, answered by [`check`](Self::check).
///
/// # Examples
///
#[cfg_attr(feature = "parse", doc = "```")]
#[cfg_attr(not(feature = "parse"), doc = "```ignore")]
/// use linlog::{Forest, Member, Mode, Node, NodeId, Proof, Sequent};
///
/// // ⊢ ~A, A ⊗ ~B, B, with the members 0: ~A, 1: A ⊗ ~B, 2: A,
/// // 3: ~B, 4: B.
/// let sequent: Sequent = "A, A -o B |- B".parse()?;
/// let forest = Forest::new(&sequent)?;
/// let (o, n) = (Member::new, NodeId::new);
/// let nodes = vec![
///     Node::Ax(o(0), o(2)),
///     Node::Ax(o(3), o(4)),
///     Node::Tensor(o(1), n(0), n(1)),
/// ];
/// let proof = Proof::new(forest, nodes, n(2))?;
/// proof.check(Mode::CLASSICAL)?;
/// assert_eq!(
///     proof.derivation()?.to_string(),
///     "─────── ax   ─────── ax\n\
///      ⊢ ~A, A      ⊢ ~B, B\n\
///      ──────────────────── ⊗\n\
///     \x20 ⊢ ~A, A ⊗ ~B, B"
/// );
/// # Ok::<(), linlog::Error>(())
/// ```
///
/// # JSON
///
/// With the feature `serialize` a proof is `{"sequent": …, "proof": […]}`:
/// its sequent in [`Sequent`]'s form, and its nodes, premises before
/// conclusions and the root last, each an object of one key, the rule:
/// `{"ax": [x, y]}` on two literals, `{"1": o}` and `{"⊤": o}` on one
/// occurrence, `{"⊗": [o, l, r]}` and `{"&": [o, l, r]}` with the indices
/// of two premises, `{"mix": [l, r]}`, and `{"⅋": [o, p]}` with one, as
/// `⊥`, `⊕₁`, `⊕₂`, `!`, `?`, `copy` and `wk` have it. An occurrence is
/// an id of the sequent's [`Forest`], a premise a node's index: a proof of
/// `A, B ⊢ A ⊗ B` is `[{"ax": [0, 3]}, {"ax": [1, 4]}, {"⊗": [2, 0, 1]}]`.
/// Reading rebuilds the forest and checks the indices and their order;
/// whether the proof is one is [`Proof::check`]'s question, since the
/// mode is not in it. The command's `check` reads this form, and the
/// output of `prove --format json` holds it.
#[derive(Clone, Debug)]
pub struct Proof {
    /// The forest of the sequent the proof is of.
    forest: Forest,
    /// The rule instances, premises before conclusions, the root last.
    nodes: Box<[Node]>,
    /// The members the root concludes, where they are not the sequent's
    /// root formulas.
    goal: Option<Box<[Member]>>,
    /// The mode the proof is meant for: a claim, which only a check tests.
    mode: Option<Mode>,
}

impl Proof {
    /// Builds a proof from the nodes of an arena and its root, keeping the
    /// nodes the root reaches in their order. Fails if a node the root
    /// reaches names an occurrence outside the forest or a premise that does
    /// not precede it, if the root lies outside the arena, or if it reaches
    /// more nodes than a [`NodeId`] counts (2³² − 1).
    pub fn new(forest: Forest, nodes: Vec<Node>, root: NodeId) -> Result<Self, Error> {
        if root.index() >= nodes.len() {
            return Err(Error::IndexOutOfBounds {
                space: crate::limits::Space::Node,
                index: root.index(),
                len: nodes.len(),
            });
        }
        // A premise precedes its conclusion, so one pass from the root down
        // visits every reachable node after the node that reaches it. The
        // root's index is below the length of a vector, so one more is a
        // `usize`.
        let mut reachable = vec![false; root.index() + 1];
        reachable[root.index()] = true;
        for i in (0..=root.index()).rev() {
            if !reachable[i] {
                continue;
            }
            for o in nodes[i].members() {
                if o.index() >= forest.len() {
                    return Err(Error::IndexOutOfBounds {
                        space: crate::limits::Space::Occurrence,
                        index: o.index(),
                        len: forest.len(),
                    });
                }
            }
            for p in nodes[i].premises() {
                if p.index() >= i {
                    return Err(Error::NotTopological {
                        space: crate::limits::Space::Node,
                        index: p.index(),
                        parent: i,
                    });
                }
                reachable[p.index()] = true;
            }
        }
        // The number of nodes is a `u32` too, which the checker's counts
        // of a node's readers rely on; the root's index alone allows one
        // node more.
        let reached = reachable.iter().filter(|&&r| r).count();
        if u32::try_from(reached).is_err() {
            return Err(Error::Refused(crate::limits::Refusal::Index {
                what: crate::limits::Space::Node,
                count: reached as u64,
                most: u64::from(u32::MAX),
            }));
        }
        let mut new_index = vec![NodeId::new(u32::MAX); root.index() + 1];
        let mut kept = Vec::with_capacity(reached);
        for (i, node) in nodes.into_iter().enumerate().take(root.index() + 1) {
            if !reachable[i] {
                continue;
            }
            // Fewer than `reached` nodes are kept so far.
            new_index[i] = NodeId::new(kept.len() as u32);
            kept.push(node.map_premises(|p| new_index[p.index()]));
        }
        Ok(Self {
            forest,
            nodes: kept.into_boxed_slice(),
            goal: None,
            mode: None,
        })
    }

    /// Builds a proof of `goal`, members of the forest that stand for the
    /// sequent of those subformulas, as [`new`](Self::new) builds a proof
    /// of the sequent; its root concludes the goal. Fails as `new` does,
    /// or for a member outside the forest.
    pub fn new_of_goal(
        forest: Forest,
        goal: &[Member],
        nodes: Vec<Node>,
        root: NodeId,
    ) -> Result<Self, Error> {
        if let Some(m) = goal.iter().find(|m| m.occurrence(&forest).is_none()) {
            return Err(Error::IndexOutOfBounds {
                space: crate::limits::Space::Member,
                index: m.index(),
                len: forest.len(),
            });
        }
        let mut proof = Self::new(forest, nodes, root)?;
        proof.goal = Some(goal.into());
        Ok(proof)
    }

    /// Returns the proof with the mode it is meant for recorded.
    #[must_use]
    pub fn with_mode(self, mode: Mode) -> Self {
        Self {
            mode: Some(mode),
            ..self
        }
    }

    /// Returns the members the root concludes, or `None` for a proof of
    /// the sequent, whose root concludes its root formulas.
    pub fn goal(&self) -> Option<&[Member]> {
        self.goal.as_deref()
    }

    /// Returns the mode the proof is meant for, as the search that found
    /// it records: a claim, which [`check`](Self::check) tests in the mode
    /// it is given.
    pub fn mode(&self) -> Option<Mode> {
        self.mode
    }

    /// Returns the occurrence a member of the proof stands for.
    pub fn occurrence(&self, member: Member) -> OccId {
        member.occ()
    }

    /// Returns the formula a member of the proof stands for.
    pub fn formula(&self, member: Member) -> impl Display + '_ {
        self.forest.formula(member.occ())
    }

    /// Returns the proof with `goal` recorded as what its root concludes.
    pub(crate) fn concluding(self, goal: &[OccId]) -> Self {
        Self {
            goal: Some(goal.iter().copied().map(Member::from).collect()),
            ..self
        }
    }

    /// Returns the occurrences the root concludes: the goal's, or the
    /// sequent's roots.
    pub(crate) fn conclusion(&self) -> Vec<OccId> {
        match &self.goal {
            Some(goal) => goal.iter().map(|m| m.occ()).collect(),
            None => self.forest.roots().to_vec(),
        }
    }

    /// Returns the forest of the sequent the proof is of.
    pub fn forest(&self) -> &Forest {
        &self.forest
    }

    /// Returns the sequent the proof is of.
    pub fn sequent(&self) -> &Sequent {
        self.forest.sequent()
    }

    /// Returns the arena: every node, premises before conclusions, the root
    /// last.
    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    /// Returns the node at `id`, which must belong to this proof.
    pub fn node(&self, id: NodeId) -> Node {
        self.nodes[id.index()]
    }

    /// Returns every node id in arena order.
    pub fn ids(&self) -> impl DoubleEndedIterator<Item = NodeId> + ExactSizeIterator {
        // A proof has at least one node and fewer than 2³².
        (0..self.nodes.len() as u32).map(NodeId::new)
    }

    /// Returns the root: the node that concludes the sequent, the last one.
    pub fn root(&self) -> NodeId {
        // A proof has at least one node and fewer than 2³².
        NodeId::new(self.nodes.len() as u32 - 1)
    }

    /// Checks that the proof proves its sequent under the rules the mode
    /// allows, within the default [`Limits`]: every node
    /// applies its rule to what its premises derive, the mode allows the
    /// rule, and the root derives exactly the sequent's formulas with
    /// nothing left in the unrestricted zone; in intuitionistic mode also
    /// that the sequent has an intuitionistic reading and every sequent of
    /// the proof one formula on the right of `⊢`. The error names the first
    /// node that fails, in arena order, with what it needed.
    pub fn check(&self, mode: Mode) -> Result<(), CheckError> {
        check::check(self, mode)
    }

    /// Checks the proof as [`check`](Self::check) does, holding
    /// `limits.memory_bytes` at most, visiting `limits.work` nodes at
    /// most, and asking `stop` every 4 096 nodes. A check that a bound or
    /// the stop ends answers [`CheckError::Refused`]: the proof is then
    /// neither valid nor invalid. What is counted is what the pass holds
    /// beyond the proof and its forest: twelve bytes for every node, and
    /// every sequent it keeps for a later node at the size of its tables
    /// of members, a sequent that several nodes read once for each.
    pub fn check_within(
        &self,
        mode: Mode,
        limits: &Limits,
        mut stop: impl FnMut(Progress) -> bool,
    ) -> Result<(), CheckError> {
        check::check_within(self, mode, limits, &mut stop)
    }

    /// Unfolds the proof into the derivation of the standard sequent
    /// calculus it stands for, within the default [`Limits`], or reports
    /// why it is not a proof or why the derivation was not built; see
    /// [`Derivation`], and [`derivation_within`](Self::derivation_within)
    /// for other options.
    pub fn derivation(&self) -> Result<Derivation<'_>, Error> {
        self.derivation_within(&ViewOptions::default(), &Limits::default(), |_| false)
    }

    /// Unfolds the proof as [`derivation`](Self::derivation) does, shown
    /// as `view` says, within `limits.derivation_bytes` and
    /// `limits.memory_bytes`, and until `stop` returns true, which is
    /// asked every 4 096 nodes of the checker's passes and once per node
    /// unfolded.
    pub fn derivation_within(
        &self,
        view: &ViewOptions,
        limits: &Limits,
        stop: impl FnMut(Progress) -> bool,
    ) -> Result<Derivation<'_>, Error> {
        Derivation::new(self, view, limits, stop)
    }

    /// Returns the two-sided derivation of intuitionistic linear logic the
    /// proof stands for, or the checker's complaint in intuitionistic
    /// mode, or why the derivation was not built. See
    /// [`Derivation::two_sided`].
    pub fn two_sided_derivation(&self) -> Result<Derivation<'_>, Error> {
        self.two_sided_derivation_within(&ViewOptions::default(), &Limits::default(), |_| false)
    }

    /// Unfolds the proof as
    /// [`two_sided_derivation`](Self::two_sided_derivation) does, as
    /// [`derivation_within`](Self::derivation_within) does.
    pub fn two_sided_derivation_within(
        &self,
        view: &ViewOptions,
        limits: &Limits,
        stop: impl FnMut(Progress) -> bool,
    ) -> Result<Derivation<'_>, Error> {
        Derivation::two_sided(self, view, limits, stop)
    }
}

impl Node {
    /// Returns the node with every premise index replaced by what `f` maps
    /// it to.
    pub(crate) fn map_premises(self, mut f: impl FnMut(NodeId) -> NodeId) -> Self {
        use Node::*;
        match self {
            Tensor(o, l, r) => Tensor(o, f(l), f(r)),
            Par(o, p) => Par(o, f(p)),
            Bot(o, p) => Bot(o, f(p)),
            With(o, l, r) => With(o, f(l), f(r)),
            Plus(o, s, p) => Plus(o, s, f(p)),
            Bang(o, p) => Bang(o, f(p)),
            Quest(o, p) => Quest(o, f(p)),
            Copy(o, p) => Copy(o, f(p)),
            Weaken(o, p) => Weaken(o, f(p)),
            Mix(l, r) => Mix(f(l), f(r)),
            leaf @ (Ax(..) | One(_) | Top(_)) => leaf,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The name lists follow the variants: `NAMES` one each in order, and
    /// `TAGS` every rule's name, in order, `Plus` twice.
    #[test]
    fn name_lists_follow_the_variants() {
        let (m, n) = (Member::new(0), NodeId::new(0));
        let nodes = [
            Node::Ax(m, m),
            Node::Tensor(m, n, n),
            Node::Par(m, n),
            Node::One(m),
            Node::Bot(m, n),
            Node::With(m, n, n),
            Node::Plus(m, Branch::Left, n),
            Node::Top(m),
            Node::Bang(m, n),
            Node::Quest(m, n),
            Node::Copy(m, n),
            Node::Weaken(m, n),
            Node::Mix(n, n),
        ];
        let names: Vec<String> = nodes
            .iter()
            .map(|node| format!("{node:?}").split('(').next().unwrap().to_owned())
            .collect();
        assert_eq!(names, Node::NAMES);
        let mut tags: Vec<&str> = nodes.iter().map(|node| node.name()).collect();
        tags.insert(7, Node::Plus(m, Branch::Right, n).name());
        assert_eq!(tags, Node::TAGS);
    }
}
