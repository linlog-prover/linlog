// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Proof nets for unit-free MLL, with or without Mix: proof structures over
//! the occurrence forest, the correctness criterion, and the conversions
//! to and from proof terms.
//!
//! A [`ProofStructure`] is the sequent's formula trees, which the forest
//! already holds, plus axiom links between dual literal occurrences. It is
//! complete when every literal is linked. A complete structure is a proof
//! net when it passes the Danos–Regnier criterion, which
//! [`is_correct`](ProofStructure::is_correct) decides independently of any
//! search and of the proof checker: no switching cycle, and without Mix
//! exactly one connected component under every switching. A proof net
//! sequentializes into a proof term that the checker accepts, and a proof
//! term of MLL desequentializes into the net of its axiom links; two
//! derivations that differ only by the order of their rules give the same
//! net.

// A variant a later step adds must not fall into an existing arm.
#![deny(clippy::wildcard_enum_match_arm)]

/// The coloured structure graph and the criterion's tests on it.
mod graph;
/// From a proof net to a proof term.
mod sequentialize;
/// The `⅋`-free skeleton as a union-find with undo.
mod skeleton;

pub use graph::Scratch;

use crate::Error;
use crate::errors::{Described, Owner, Subject};
use crate::fragment::{Fragment, Mode};
use crate::limits::{Limits, Phase, Progress, Refusal, Space};
use crate::occurrences::{Forest, OccId};
use crate::proofs::{Node, NodeId, Proof};
use crate::sequents::{Kind, Sequent, Term};
use graph::Graph;
use skeleton::Skeleton;
use std::fmt::{Display, Formatter, Result as FmtResult};

/// The raw index that stands for "no vertex".
const NONE: u32 = u32::MAX;

/// The most vertices a structure has: a vertex has up to three slots in
/// the graph (its parent, its children or its link), whose offsets are
/// `u32`, and the last offset is the total.
const MOST: u64 = (u32::MAX / 3 - 1) as u64;

/// Checks that proof structures exist for a sequent of `fragment` in
/// `mode`: unit-free MLL, in a linear mode, with or without Mix. A front
/// end asks it before it offers a net; the net engine's search answers the
/// same.
///
/// # Errors
///
/// [`Error::NetFragment`] for a larger fragment and [`Error::NetMode`] in
/// affine mode.
pub fn exist(fragment: Fragment, mode: Mode) -> Result<(), Error> {
    if !fragment.has_nets() {
        return Err(Error::NetFragment { fragment });
    }
    if mode.is_affine() {
        return Err(Error::NetMode { mode });
    }
    Ok(())
}

/// The bytes a structure holds per vertex, the copy of its forest
/// included: the forest's per-occurrence arrays (25), `partner` (4), the
/// graph's offset (4) and up to three slots (12), the skeleton's parent
/// and rank (5), and a link per two literals (4).
const STRUCTURE_BYTES: u64 = 25 + 4 + 4 + 12 + 5 + 4;

/// The bytes the criterion's working memory takes per vertex: a flag per
/// slot (3), the search's four arrays (16), its bridge flags (1) and its
/// stack (8), and the two sets of deleted vertices (1).
const SCRATCH_BYTES: u64 = 3 + 16 + 1 + 8 + 1;

/// The bytes a sequentialization holds per vertex beside its structure:
/// the proof's copy of the forest (25), at most one node (16), the
/// working memory of the criterion and the conclusions waiting to be
/// proved (8).
const SEQUENTIALIZE_BYTES: u64 = 25 + 16 + SCRATCH_BYTES + 8;

/// A vertex of a proof structure: in MLL the occurrence of the same
/// number, so vertex `i` of a structure is occurrence `i` of its forest.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct VertexId(u32);

impl VertexId {
    /// Wraps a raw vertex index.
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    /// Returns the raw vertex index.
    pub const fn get(self) -> u32 {
        self.0
    }

    /// Returns the index as a `usize`, for indexing per-vertex arrays.
    pub const fn index(self) -> usize {
        self.0 as usize
    }

    /// Returns the vertex of an occurrence of an MLL structure.
    pub(crate) const fn of(o: OccId) -> Self {
        Self(o.get())
    }

    /// Returns the occurrence of a vertex of an MLL structure.
    pub(crate) const fn occ(self) -> OccId {
        OccId::new(self.0)
    }
}

/// The rules a proof structure is checked under: whether Mix is allowed,
/// in which case a proof net need not be connected.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Criterion {
    /// Whether the Mix rule is allowed.
    pub mix: bool,
}

impl Criterion {
    /// Multiplicative linear logic without Mix.
    pub const MLL: Self = Self { mix: false };

    /// Returns the criterion with Mix allowed.
    #[must_use]
    pub const fn with_mix(self) -> Self {
        Self { mix: true }
    }

    /// Returns the criterion of a mode: Mix as the mode has it. Fails
    /// with [`NetError::Mode`] in affine mode, which has no proof nets; in
    /// intuitionistic mode the net is the one of the one-sided sequent.
    ///
    /// # Errors
    ///
    /// [`NetError::Mode`] in affine mode.
    pub fn of(mode: Mode) -> Result<Self, NetError> {
        if mode.is_affine() {
            return Err(NetError::Mode { mode });
        }
        Ok(Self {
            mix: mode.has_mix(),
        })
    }

    /// The mode a sequentialized proof holds in: classical, with Mix if
    /// the criterion allows it.
    const fn mode(self) -> Mode {
        if self.mix {
            Mode::CLASSICAL.with_mix()
        } else {
            Mode::CLASSICAL
        }
    }
}

/// Why a proof structure is not a proof net, why a list of links is not a
/// proof structure, or why there is no structure. Vertices print as ids;
/// [`describe`](Self::describe) prints them as formulas.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NetError {
    /// A link names a vertex outside the structure.
    #[non_exhaustive]
    NoVertex {
        /// The vertex named.
        vertex: u32,
        /// How many vertices the structure has.
        vertices: u32,
    },
    /// A link names a vertex that is not a literal.
    #[non_exhaustive]
    NotLiteral {
        /// The vertex.
        vertex: VertexId,
    },
    /// A link joins two literals that are not `a` and `~a` for one atom.
    #[non_exhaustive]
    NotDual {
        /// The first literal of the link.
        x: VertexId,
        /// The second literal of the link.
        y: VertexId,
    },
    /// A literal appears in two links.
    #[non_exhaustive]
    LinkedTwice {
        /// The literal.
        vertex: VertexId,
    },
    /// A literal has no link, so the structure is incomplete.
    #[non_exhaustive]
    Unlinked {
        /// The literal.
        vertex: VertexId,
    },
    /// The structure has no vertex at all: no rule concludes the empty
    /// sequent.
    Empty,
    /// A cycle survives some switching.
    #[non_exhaustive]
    SwitchingCycle {
        /// The vertices the cycle runs through, in order along it.
        cycle: Vec<VertexId>,
    },
    /// Every switching falls into several parts.
    #[non_exhaustive]
    Disconnected {
        /// The parts of the switching that keeps the left premise of
        /// every `⅋`, each given by the vertices in it that have no parent
        /// edge there: the roots and the right premises of `⅋` nodes.
        parts: Vec<Vec<VertexId>>,
    },
    /// Proof nets exist for unit-free MLL only, and the sequent lies in a
    /// larger fragment.
    #[non_exhaustive]
    Fragment {
        /// The sequent's fragment.
        fragment: Fragment,
    },
    /// Proof nets exist in linear mode only, and the mode is affine.
    #[non_exhaustive]
    Mode {
        /// The mode asked for.
        mode: Mode,
    },
    /// A node of the proof applies a rule that a proof net of unit-free
    /// MLL has no place for.
    #[non_exhaustive]
    Rule {
        /// The node.
        node: NodeId,
        /// The rule's name, as [`Node::name`] gives it.
        rule: &'static str,
    },
    /// A bound or the caller's stop ended the call without a verdict on
    /// the structure.
    #[non_exhaustive]
    Refused {
        /// The bound that refused it, or the stop.
        refusal: Refusal,
    },
}

impl NetError {
    /// Returns what sort of failure this is: a list of links that is no
    /// structure is malformed, a structure that is no net invalid, a
    /// sequent, a mode or a rule without nets unsupported, a refusal no
    /// verdict.
    pub fn kind(&self) -> crate::ErrorKind {
        use crate::ErrorKind::*;
        match self {
            Self::NoVertex { .. }
            | Self::NotLiteral { .. }
            | Self::NotDual { .. }
            | Self::LinkedTwice { .. } => Malformed,
            Self::Unlinked { .. }
            | Self::Empty
            | Self::SwitchingCycle { .. }
            | Self::Disconnected { .. } => Invalid,
            Self::Fragment { .. } | Self::Mode { .. } | Self::Rule { .. } => Unsupported,
            Self::Refused { refusal } => crate::errors::refusal_kind(refusal),
        }
    }

    /// Returns the stable code of the error: `no_nets` where no structure
    /// exists, the refusal's, or `invalid_net`.
    pub fn code(&self) -> &'static str {
        match self {
            Self::Fragment { .. } | Self::Mode { .. } | Self::Rule { .. } => "no_nets",
            Self::Refused { refusal } => refusal.code(),
            Self::NoVertex { .. }
            | Self::NotLiteral { .. }
            | Self::NotDual { .. }
            | Self::LinkedTwice { .. }
            | Self::Unlinked { .. }
            | Self::Empty
            | Self::SwitchingCycle { .. }
            | Self::Disconnected { .. } => "invalid_net",
        }
    }

    /// Returns the error for display with every vertex as its formula
    /// followed by its id in brackets, as in `~A[0]`, read from `owner`,
    /// the structure that failed or its forest.
    pub fn describe<'a>(&'a self, owner: &'a impl Owner) -> Described<'a> {
        Described::new(Subject::Net(self), owner)
    }

    /// Writes the error as [`Display`] does, with `formula[id]` in place of
    /// every vertex id when a forest is given.
    pub(crate) fn write(&self, f: &mut Formatter<'_>, forest: Option<&Forest>) -> FmtResult {
        let vertex = |v: &VertexId| match forest {
            Some(forest) if v.index() < forest.len() => {
                format!("{}[{}]", forest.formula(v.occ()), v.get())
            }
            _ => v.get().to_string(),
        };
        let list = |vs: &[VertexId]| vs.iter().map(vertex).collect::<Vec<_>>().join(", ");
        use NetError::*;
        match self {
            // A vertex outside the structure has no formula.
            NoVertex { vertex, vertices } => write!(
                f,
                "a link names vertex {vertex}, but the structure has {}",
                crate::errors::counted(*vertices, "vertex", "vertices")
            ),
            NotLiteral { vertex: v } => write!(f, "vertex {} is not a literal", vertex(v)),
            NotDual { x, y } => {
                write!(
                    f,
                    "the literals {} and {} are not dual",
                    vertex(x),
                    vertex(y)
                )
            }
            LinkedTwice { vertex: v } => write!(f, "literal {} is linked twice", vertex(v)),
            Unlinked { vertex: v } => write!(f, "literal {} has no axiom link", vertex(v)),
            Empty => f.write_str("the structure is empty, and no rule concludes the empty sequent"),
            SwitchingCycle { cycle } => write!(f, "a switching cycle runs through {}", list(cycle)),
            Disconnected { parts } => {
                let parts: Vec<String> = parts.iter().map(|p| format!("{{{}}}", list(p))).collect();
                write!(
                    f,
                    "every switching falls into {} parts; keeping every left premise, they are {}",
                    parts.len(),
                    parts.join(" and ")
                )
            }
            Fragment { fragment } => write!(
                f,
                "proof nets exist for MLL without units only, not for {fragment}"
            ),
            Mode { mode } => write!(
                f,
                "proof nets exist in linear mode only, with or without Mix, not in {mode} mode"
            ),
            Rule { node, rule } => write!(
                f,
                "node {} applies {rule}, which a proof net of MLL without units has no place for",
                node.get()
            ),
            Refused { refusal } => write!(f, "{refusal}"),
        }
    }
}

impl Display for NetError {
    /// Writes the error with vertex ids.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.write(f, None)
    }
}

impl std::error::Error for NetError {}

/// Fails with [`Refusal::Memory`] when `bytes` are more than `limits`
/// allow.
fn afford(bytes: u64, limits: &Limits) -> Result<(), NetError> {
    match limits.memory_bytes {
        Some(limit) if bytes > limit => Err(NetError::Refused {
            refusal: Refusal::Memory {
                phase: Phase::Net,
                limit_bytes: limit,
                needed_bytes: Some(bytes),
            },
        }),
        _ => Ok(()),
    }
}

/// The refusal of a call that `stop` ended.
const STOPPED: NetError = NetError::Refused {
    refusal: Refusal::Stopped { phase: Phase::Net },
};

/// A proof structure of unit-free MLL over the occurrence forest of its
/// sequent: the formula trees, which the forest holds, plus axiom links
/// between dual literal vertices, and the [`Criterion`] that decides what
/// [`is_correct`](Self::is_correct) requires. Vertex `i` is occurrence `i`
/// of the forest ([`vertex`](Self::vertex), [`occurrence`](Self::occurrence)).
///
/// Links are added with [`link`](Self::link), which refuses what is not a
/// link, and taken back in the reverse order with [`unlink`](Self::unlink),
/// both in constant time, so that a search can try linkings by
/// backtracking. The structure keeps the `⅋`-free skeleton (the premise
/// edges of `⊗` nodes and the links) as a union-find, so that
/// [`same_component`](Self::same_component) answers in logarithmic time
/// whether a candidate link would close a cycle that no switching breaks.
///
/// # Examples
///
#[cfg_attr(feature = "parse", doc = "```")]
#[cfg_attr(not(feature = "parse"), doc = "```ignore")]
/// use linlog::{Criterion, Forest, ProofStructure, Sequent, VertexId};
///
/// // ⊢ ~A, A ⊗ ~B, B, with the vertices 0: ~A, 1: A ⊗ ~B, 2: A,
/// // 3: ~B, 4: B.
/// let sequent: Sequent = "A, A -o B |- B".parse()?;
/// let v = VertexId::new;
/// let links = [(v(0), v(2)), (v(3), v(4))];
/// let net = ProofStructure::from_links(Forest::new(&sequent)?, Criterion::MLL, &links)?;
/// assert!(net.is_complete());
/// assert_eq!(net.partner(v(2)), Some(v(0)));
/// assert_eq!(net.is_correct(|_| false), Ok(()));
/// # Ok::<(), linlog::Error>(())
/// ```
///
/// # JSON
///
/// With the feature `serialize` a structure is `{"version": 1, "sequent":
/// …, "mix": …, "links": [[x, y], …]}` ([`wire`](crate::wire)): its
/// sequent in [`Sequent`]'s form, the
/// criterion's fields (whether Mix is allowed), and its axiom links as
/// pairs of vertex ids in the order they were made. Reading validates the
/// links as [`from_links`](Self::from_links) does and takes a partial or
/// incorrect structure, since whether it is a net is
/// [`is_correct`](Self::is_correct)'s question.
#[derive(Clone, Debug)]
pub struct ProofStructure {
    /// The forest of the sequent.
    forest: Forest,
    /// The rules the structure is checked under.
    criterion: Criterion,
    /// Per vertex, the literal it is linked to, or `NONE`.
    partner: Box<[u32]>,
    /// The links in the order they were made, each as the pair given to
    /// [`link`](Self::link).
    links: Vec<(VertexId, VertexId)>,
    /// The coloured structure graph.
    graph: Graph,
    /// The `⅋`-free skeleton.
    skeleton: Skeleton,
}

impl ProofStructure {
    /// Returns the structure over the forest with no link yet. Fails with
    /// [`NetError::Fragment`] if the sequent lies outside unit-free MLL,
    /// where there are no proof nets, and with [`Refusal::Index`] past
    /// 1 431 655 763 vertices.
    ///
    /// # Errors
    ///
    /// [`NetError::Fragment`] for a sequent outside unit-free MLL, and
    /// [`Refusal::Index`] for a forest past the vertices a structure holds.
    pub fn new(forest: Forest, criterion: Criterion) -> Result<Self, Error> {
        let fragment = forest.sequent().fragment();
        if !fragment.has_nets() {
            return Err(NetError::Fragment { fragment }.into());
        }
        let n = forest.len();
        if n as u64 > MOST {
            return Err(Error::Refused(Refusal::Index {
                what: Space::Vertex,
                count: n as u64,
                most: MOST,
            }));
        }
        let mut skeleton = Skeleton::new(n);
        for o in forest.ids() {
            if let Some(p) = forest.parent(o)
                && forest.kind(p) == Kind::Tensor
            {
                skeleton.union(p.get(), o.get());
            }
        }
        let literals = forest.all_literals().len();
        skeleton.commit(literals / 2);
        Ok(Self {
            graph: Graph::new(&forest),
            forest,
            criterion,
            partner: vec![NONE; n].into_boxed_slice(),
            links: Vec::with_capacity(literals / 2),
            skeleton,
        })
    }

    /// Returns the structure with the given links. Fails as
    /// [`new`](Self::new) does, and with the [`NetError`] of
    /// [`link`](Self::link) if a link names a vertex outside the
    /// structure, does not join two dual literals or links a literal
    /// twice.
    ///
    /// # Errors
    ///
    /// As [`new`](Self::new) does, and the [`NetError`] of
    /// [`link`](Self::link) for a link that is none.
    pub fn from_links(
        forest: Forest,
        criterion: Criterion,
        links: &[(VertexId, VertexId)],
    ) -> Result<Self, Error> {
        let mut net = Self::new(forest, criterion)?;
        for &(x, y) in links {
            net.link(x, y)?;
        }
        Ok(net)
    }

    /// Desequentializes a proof of MLL: reads the axiom links off its `Ax`
    /// nodes and builds their structure over a copy of the proof's
    /// forest, within `limits.memory_bytes` (the structure and the
    /// criterion's working memory, estimated before anything is made) and
    /// until `stop` returns true, which is asked once per node and per
    /// round of the criterion. Every proof the checker accepts gives a
    /// proof net; two proofs that differ only in the order of their rules
    /// give the same one. The checker is not run: the net of a term the
    /// checker would reject can still be a proof net, and is returned as
    /// one.
    ///
    /// # Errors
    ///
    /// [`Error::GoalProof`] for a proof of a goal other than the sequent;
    /// [`NetError::Rule`] for a node a net has no place for (any but `ax`,
    /// `⊗`, `⅋` and Mix); otherwise as [`from_links`](Self::from_links)
    /// does, and with the criterion's error if the links are not a proof
    /// net, which happens only for a proof the checker rejects or one that
    /// uses Mix where the criterion forbids it; [`NetError::Refused`] when
    /// the bound or the stop ended it.
    pub fn from_proof(
        proof: &Proof,
        criterion: Criterion,
        limits: &Limits,
        mut stop: impl FnMut(Progress) -> bool,
    ) -> Result<Self, Error> {
        if proof.goal().is_some() {
            return Err(Error::GoalProof);
        }
        let forest = proof.forest();
        afford(Self::bytes(forest), limits)?;
        let mut links = Vec::with_capacity(forest.all_literals().len() / 2);
        for (i, &node) in proof.nodes().iter().enumerate() {
            let done = i as u64 + 1;
            if stop(Progress::new(Phase::Net, 1, done)) {
                return Err(STOPPED.into());
            }
            match node {
                Node::Ax(x, y) => links.push((VertexId::of(x.occ()), VertexId::of(y.occ()))),
                Node::Tensor(..) | Node::Par(..) | Node::Mix(..) => {}
                Node::One(_)
                | Node::Bot(..)
                | Node::With(..)
                | Node::Plus(..)
                | Node::Top(_)
                | Node::Bang(..)
                | Node::Quest(..)
                | Node::Copy(..)
                | Node::Weaken(..) => {
                    return Err(NetError::Rule {
                        node: NodeId::new(i as u32),
                        rule: node.name(),
                    }
                    .into());
                }
            }
        }
        let net = Self::from_links(forest.clone(), criterion, &links)?;
        net.is_correct(stop)?;
        Ok(net)
    }

    /// Returns the bytes a structure over the forest holds with the
    /// criterion's working memory, the copy of its forest included.
    fn bytes(forest: &Forest) -> u64 {
        let arena = forest.sequent().terms().len() as u64 * size_of::<Term>() as u64;
        (forest.len() as u64 * (STRUCTURE_BYTES + SCRATCH_BYTES)).saturating_add(arena)
    }

    /// Fails if `x` and `y` are not two unlinked dual literals of the
    /// structure; the ids may be any, also none of the structure's.
    fn check_link(&self, x: VertexId, y: VertexId) -> Result<(), NetError> {
        let f = &self.forest;
        for v in [x, y] {
            if v.index() >= f.len() {
                return Err(NetError::NoVertex {
                    vertex: v.get(),
                    vertices: f.len() as u32,
                });
            }
            if !f.is_literal(v.occ()) {
                return Err(NetError::NotLiteral { vertex: v });
            }
        }
        if !self.dual(x, y) {
            return Err(NetError::NotDual { x, y });
        }
        for v in [x, y] {
            if self.partner(v).is_some() {
                return Err(NetError::LinkedTwice { vertex: v });
            }
        }
        Ok(())
    }

    /// Returns whether two literal vertices are `a` and `~a` for one atom.
    fn dual(&self, x: VertexId, y: VertexId) -> bool {
        self.forest.dual_literals(x.occ(), y.occ())
    }

    /// Returns the forest of the sequent.
    pub fn forest(&self) -> &Forest {
        &self.forest
    }

    /// Returns the sequent.
    pub fn sequent(&self) -> &Sequent {
        self.forest.sequent()
    }

    /// Returns the rules the structure is checked under.
    pub fn criterion(&self) -> Criterion {
        self.criterion
    }

    /// Returns the vertex of an occurrence of the forest, `None` for an
    /// id outside it.
    pub fn vertex(&self, o: OccId) -> Option<VertexId> {
        (o.index() < self.forest.len()).then_some(VertexId::of(o))
    }

    /// Returns the occurrence a vertex stands for, `None` for a vertex
    /// outside the structure.
    pub fn occurrence(&self, v: VertexId) -> Option<OccId> {
        (v.index() < self.forest.len()).then_some(v.occ())
    }

    /// Returns the literal a vertex is linked to, or `None` for an
    /// unlinked literal, a connective or a vertex outside the structure.
    pub fn partner(&self, v: VertexId) -> Option<VertexId> {
        match self.partner.get(v.index()) {
            None | Some(&NONE) => None,
            Some(&p) => Some(VertexId(p)),
        }
    }

    /// Returns [`partner`](Self::partner) of a vertex the caller knows to
    /// be the structure's, as a search does of its candidates: one
    /// comparison, which the search makes for every candidate link.
    pub(crate) fn mate(&self, v: VertexId) -> Option<VertexId> {
        match self.partner[v.index()] {
            NONE => None,
            p => Some(VertexId(p)),
        }
    }

    /// Returns the links in the order they were made.
    pub fn links(&self) -> &[(VertexId, VertexId)] {
        &self.links
    }

    /// Returns whether every literal is linked.
    pub fn is_complete(&self) -> bool {
        2 * self.links.len() == self.forest.all_literals().len()
    }

    /// Returns the literals without a link, grouped by atom and sign as
    /// [`Forest::all_literals`] lists them.
    pub fn unlinked(&self) -> impl Iterator<Item = VertexId> {
        self.forest
            .all_literals()
            .iter()
            .filter(|&&l| self.partner[l.index()] == NONE)
            .map(|&l| VertexId::of(l))
    }

    /// Links two unlinked dual literals, in constant time.
    ///
    /// # Errors
    ///
    /// Fails, and leaves the structure as it is, if one of the two is no
    /// vertex of the structure ([`NetError::NoVertex`]) or no literal
    /// ([`NetError::NotLiteral`]), if they are not dual
    /// ([`NetError::NotDual`]), or if one has a link already
    /// ([`NetError::LinkedTwice`]).
    pub fn link(&mut self, x: VertexId, y: VertexId) -> Result<(), NetError> {
        self.check_link(x, y)?;
        self.link_unchecked(x, y);
        Ok(())
    }

    /// Links two literals that the caller knows to be unlinked dual
    /// literals of the structure, as a search does of the candidates it
    /// enumerates; a debug build checks. Anything else corrupts the
    /// structure.
    pub(crate) fn link_unchecked(&mut self, x: VertexId, y: VertexId) {
        debug_assert!(
            self.check_link(x, y).is_ok(),
            "{x:?} and {y:?} cannot be linked"
        );
        self.partner[x.index()] = y.get();
        self.partner[y.index()] = x.get();
        self.links.push((x, y));
        self.skeleton.union(x.get(), y.get());
        self.graph.link(x.get(), y.get());
    }

    /// Takes back the last link made and returns it, or `None` if there is
    /// no link. Links are undone in the reverse order of their making, as a
    /// backtracking search does.
    pub fn unlink(&mut self) -> Option<(VertexId, VertexId)> {
        let (x, y) = self.links.pop()?;
        self.partner[x.index()] = NONE;
        self.partner[y.index()] = NONE;
        self.skeleton.undo();
        self.graph.unlink(x.get(), y.get());
        Some((x, y))
    }

    /// Returns whether two vertices are joined by a path that uses no
    /// premise edge of a `⅋`: `⊗` premise edges and links only. Linking two
    /// such literals closes a cycle that every switching keeps. A vertex
    /// outside the structure is joined to nothing.
    pub fn same_component(&self, x: VertexId, y: VertexId) -> bool {
        let n = self.forest.len();
        x.index() < n && y.index() < n && self.joined(x, y)
    }

    /// Returns [`same_component`](Self::same_component) of two vertices
    /// the caller knows to be the structure's, as a search does of its
    /// candidates, without comparing them with its length: the search
    /// asks it for every candidate link.
    pub(crate) fn joined(&self, x: VertexId, y: VertexId) -> bool {
        self.skeleton.same(x.get(), y.get())
    }

    /// Returns working memory for the tests of the criterion, sized for
    /// this structure; one serves any number of calls.
    pub fn scratch(&self) -> Scratch {
        self.graph.scratch()
    }

    /// Returns whether no cycle survives any switching, by the deletion
    /// procedure of Yeo's theorem: a vertex that every part of the graph
    /// without it meets in edges of one colour only lies on no switching
    /// cycle and goes; when none is left to delete, what remains carries a
    /// switching cycle, or nothing remains. Meaningful on a partial
    /// structure too: an unlinked literal lies on no cycle. Allocates
    /// nothing with a scratch from [`scratch`](Self::scratch); a scratch
    /// sized for another structure is replaced by one for this one.
    pub fn is_acyclic(&self, scratch: &mut Scratch) -> bool {
        if !scratch.fits(&self.graph) {
            *scratch = self.scratch();
        }
        scratch.restore();
        self.graph
            .acyclic(&self.forest, scratch, &mut |_| false)
            .unwrap_or(false)
    }

    /// Decides whether the structure is a proof net, independently of any
    /// search and of the proof checker: every literal is linked, no cycle
    /// survives any switching, and, unless the criterion allows Mix, every
    /// switching is connected, which given acyclicity is the count of
    /// edges a switching keeps being one less than the number of vertices.
    /// It holds working memory linear in the structure and asks `stop`
    /// once per round of the deletion procedure, of which a witness for a
    /// cycle runs one per edge.
    ///
    /// # Errors
    ///
    /// [`NetError::Empty`], [`NetError::Unlinked`],
    /// [`NetError::SwitchingCycle`] or [`NetError::Disconnected`] with a
    /// witness; [`NetError::Refused`] when `stop` ended it.
    pub fn is_correct(&self, mut stop: impl FnMut(Progress) -> bool) -> Result<(), NetError> {
        if self.forest.is_empty() {
            return Err(NetError::Empty);
        }
        if let Some(l) = self.unlinked().next() {
            return Err(NetError::Unlinked { vertex: l });
        }
        let mut scratch = self.scratch();
        let mut rounds = 0;
        let mut stop = |round: u64| {
            rounds += round;
            stop(Progress::new(Phase::Net, round, rounds))
        };
        match self.graph.acyclic(&self.forest, &mut scratch, &mut stop) {
            None => return Err(STOPPED),
            Some(false) => {
                return match self.graph.cycle(&self.forest, &mut scratch, &mut stop) {
                    Some(cycle) => Err(NetError::SwitchingCycle { cycle }),
                    None => Err(STOPPED),
                };
            }
            Some(true) => {}
        }
        if !self.criterion.mix
            && self.graph.switched_edges(self.links.len()) + 1 != self.forest.len()
        {
            return Err(NetError::Disconnected {
                parts: self.graph.parts(&self.forest, &mut scratch),
            });
        }
        Ok(())
    }
}

impl Display for ProofStructure {
    /// Writes the sequent, then one line per link as `~A[0] — A[2]`, each
    /// literal with its vertex id, the links ordered by their first id,
    /// then the verdict of the criterion: `proof net`, `proof net with
    /// Mix`, or `not a proof net: ` and the reason with formulas. No
    /// trailing newline.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        writeln!(f, "{}", self.sequent())?;
        let mut links: Vec<(VertexId, VertexId)> = self
            .links
            .iter()
            .map(|&(x, y)| (x.min(y), x.max(y)))
            .collect();
        links.sort_unstable();
        for (x, y) in links {
            let (fx, fy) = (self.forest.formula(x.occ()), self.forest.formula(y.occ()));
            writeln!(f, "{fx}[{}] — {fy}[{}]", x.get(), y.get())?;
        }
        match self.is_correct(|_| false) {
            Ok(()) if self.criterion.mix => f.write_str("proof net with Mix"),
            Ok(()) => f.write_str("proof net"),
            Err(e) => write!(f, "not a proof net: {}", e.describe(&self.forest)),
        }
    }
}

#[cfg(all(test, feature = "parse"))]
mod tests {
    use super::*;
    #[cfg(feature = "parse")]
    use crate::fragment::Mode;
    #[cfg(feature = "parse")]
    use crate::occurrences::Member;
    #[cfg(feature = "parse")]
    use crate::proofs::NodeId;

    /// Wraps a raw occurrence id.
    const fn o(id: u32) -> OccId {
        OccId::new(id)
    }

    /// Wraps a raw vertex id.
    const fn v(id: u32) -> VertexId {
        VertexId::new(id)
    }

    /// The criterion with Mix allowed or not.
    const fn criterion(mix: bool) -> Criterion {
        Criterion { mix }
    }

    /// Builds the forest of `input`, or panics with the parse error.
    #[cfg(feature = "parse")]
    fn forest(input: &str) -> Forest {
        let s: Sequent = input.parse().unwrap_or_else(|e| panic!("{input:?}: {e}"));
        Forest::new(&s).unwrap()
    }

    /// Links are made and taken back in stack order, the partners and the
    /// skeleton follow, and completeness is every literal linked.
    #[cfg(feature = "parse")]
    #[test]
    fn link_and_unlink() {
        // ⊢ ~A, A ⊗ ~B, B: 0 ~A, 1 ⊗, 2 A, 3 ~B, 4 B.
        let mut net = ProofStructure::new(forest("A, A -o B |- B"), criterion(false)).unwrap();
        assert!(!net.is_complete());
        assert_eq!(net.unlinked().collect::<Vec<_>>(), [v(2), v(0), v(4), v(3)]);
        assert!(net.same_component(v(2), v(3)), "the ⊗ joins its premises");
        assert!(!net.same_component(v(0), v(2)));

        net.link(v(0), v(2)).unwrap();
        assert_eq!(net.partner(v(2)), Some(v(0)));
        assert_eq!(net.partner(v(1)), None);
        assert!(net.same_component(v(0), v(3)), "through the link and the ⊗");
        net.link(v(4), v(3)).unwrap();
        assert!(net.is_complete());
        assert_eq!(net.links(), [(v(0), v(2)), (v(4), v(3))]);

        assert_eq!(net.unlink(), Some((v(4), v(3))));
        assert_eq!(net.partner(v(4)), None);
        assert!(!net.same_component(v(4), v(0)));
        assert_eq!(net.unlink(), Some((v(0), v(2))));
        assert!(!net.same_component(v(0), v(3)));
        assert_eq!(net.unlink(), None);
        assert_eq!(net.unlinked().count(), 4);
    }

    /// Two derivations that differ only in the order of their rules give
    /// the same net, and a proof that is not one of MLL is refused.
    #[cfg(feature = "parse")]
    #[test]
    fn desequentialize() {
        use Node::*;
        let (n, mode) = (NodeId::new, Mode::CLASSICAL);
        // ⊢ A ⊗ B, ~A, ~B ⊗ C, ~C: 0 ⊗, 1 A, 2 B, 3 ~A, 4 ⊗, 5 ~B, 6 C, 7 ~C.
        let f = forest("|- A * B, ~A, ~B * C, ~C");
        let first_tensor_first = Proof::new(
            f.clone(),
            vec![
                Ax(o(1).into(), o(3).into()),
                Ax(o(2).into(), o(5).into()),
                Ax(o(6).into(), o(7).into()),
                Tensor(Member::from(o(4)), n(1), n(2)),
                Tensor(Member::from(o(0)), n(0), n(3)),
            ],
            n(4),
        )
        .unwrap();
        let second_tensor_first = Proof::new(
            f.clone(),
            vec![
                Ax(o(1).into(), o(3).into()),
                Ax(o(2).into(), o(5).into()),
                Tensor(Member::from(o(0)), n(0), n(1)),
                Ax(o(6).into(), o(7).into()),
                Tensor(Member::from(o(4)), n(2), n(3)),
            ],
            n(4),
        )
        .unwrap();
        assert_eq!(first_tensor_first.check(mode), Ok(()));
        assert_eq!(second_tensor_first.check(mode), Ok(()));
        let a = ProofStructure::from_proof(
            &first_tensor_first,
            criterion(false),
            &Limits::default(),
            |_| false,
        )
        .unwrap();
        let b = ProofStructure::from_proof(
            &second_tensor_first,
            criterion(false),
            &Limits::default(),
            |_| false,
        )
        .unwrap();
        assert_eq!(links(&a), links(&b));
        assert_eq!(links(&a), [(v(1), v(3)), (v(2), v(5)), (v(6), v(7))]);

        // A proof with Mix desequentializes only when Mix is allowed.
        let f = forest("|- A par B, ~A, ~B");
        let with_mix = Proof::new(
            f,
            vec![
                Ax(o(1).into(), o(3).into()),
                Ax(o(2).into(), o(4).into()),
                Mix(n(0), n(1)),
                Par(Member::from(o(0)), n(2)),
            ],
            n(3),
        )
        .unwrap();
        assert!(
            ProofStructure::from_proof(&with_mix, criterion(true), &Limits::default(), |_| false)
                .is_ok()
        );
        assert!(matches!(
            ProofStructure::from_proof(&with_mix, criterion(false), &Limits::default(), |_| false),
            Err(Error::Net(e)) if matches!(*e, NetError::Disconnected { .. })
        ));
        // Outside MLL there is no net, and no node a net has no place for
        // is passed over.
        let f = forest("|- A & A, ~A");
        let additive = Proof::new(
            f,
            vec![
                Ax(o(1).into(), o(3).into()),
                Ax(o(2).into(), o(3).into()),
                With(Member::from(o(0)), n(0), n(1)),
            ],
            n(2),
        )
        .unwrap();
        assert!(matches!(
            ProofStructure::from_proof(&additive, criterion(false), &Limits::default(), |_| false),
            Err(Error::Net(e)) if matches!(*e, NetError::Rule { node, rule: "&" } if node == n(2))
        ));
        // Affine weakening on a sequent of MLL.
        let f = forest("|- A, ~A, B");
        let weakened = Proof::new(
            f,
            vec![Ax(o(0).into(), o(1).into()), Weaken(o(2).into(), n(0))],
            n(1),
        )
        .unwrap();
        assert!(matches!(
            ProofStructure::from_proof(&weakened, criterion(false), &Limits::default(), |_| false),
            Err(Error::Net(e)) if matches!(*e, NetError::Rule { rule: "wk", .. })
        ));
    }

    /// Random derivations of MLL, with and without Mix, round-trip through
    /// their net: the net is correct, the sequentialized proof passes the
    /// checker, and it has the same net as the derivation it came from.
    #[cfg(feature = "parse")]
    #[test]
    fn round_trip_random_derivations() {
        use crate::search::generate::{self, Rng, Rules};
        use crate::search::{Options, prove};
        for (seed, mix) in [(3, false), (4, true)] {
            let rules = Rules {
                units: false,
                additives: false,
                mix,
                exponentials: false,
            };
            let mode = if mix {
                Mode::CLASSICAL.with_mix()
            } else {
                Mode::CLASSICAL
            };
            let mut rng = Rng::new(seed);
            for _ in 0..60 {
                let budget = 2 + rng.below(14);
                let formulas = generate::provable(&mut rng, rules, 3, budget).formulas;
                let text = generate::sequent(&formulas);
                let s: Sequent = text.parse().unwrap();
                let outcome = prove(&s, mode, &Options::default()).unwrap();
                let proof = outcome.verdict.proof().expect("the sequent is provable");
                assert_eq!(proof.check(mode), Ok(()), "{text:?}");
                let net =
                    ProofStructure::from_proof(proof, criterion(mix), &Limits::default(), |_| {
                        false
                    })
                    .unwrap_or_else(|e| panic!("{text:?}: {e}"));
                let back = net
                    .sequentialize(&Limits::default(), |_| false)
                    .unwrap_or_else(|e| panic!("{text:?}: {e}"));
                assert_eq!(back.check(mode), Ok(()), "{text:?}");
                let again =
                    ProofStructure::from_proof(&back, criterion(mix), &Limits::default(), |_| {
                        false
                    })
                    .unwrap();
                assert_eq!(links(&again), links(&net), "{text:?}");
            }
        }
    }

    /// The text form lists the sequent, the links with their positions and
    /// the verdict, with formulas in the reason when there is one.
    #[cfg(feature = "parse")]
    #[test]
    fn text_form() {
        let net = ProofStructure::from_links(
            forest("A, A -o B |- B"),
            criterion(false),
            &[(v(2), v(0)), (v(3), v(4))],
        )
        .unwrap();
        assert_eq!(
            net.to_string(),
            "⊢ ~A, A ⊗ ~B, B\n~A[0] — A[2]\n~B[3] — B[4]\nproof net"
        );
        let mut net = ProofStructure::new(forest("|- A par B, ~A, ~B"), criterion(true)).unwrap();
        net.link(v(1), v(3)).unwrap();
        assert_eq!(
            net.to_string(),
            "⊢ A ⅋ B, ~A, ~B\nA[1] — ~A[3]\nnot a proof net: literal B[2] has no axiom link"
        );
        net.link(v(2), v(4)).unwrap();
        assert!(net.to_string().ends_with("\nproof net with Mix"));
        let net =
            ProofStructure::from_links(forest("|- A * ~A"), criterion(false), &[(v(1), v(2))])
                .unwrap();
        assert!(
            net.to_string().ends_with(
                "not a proof net: a switching cycle runs through A ⊗ ~A[0], A[1], ~A[2]"
            )
        );
        let net = ProofStructure::from_links(
            forest("|- A par B, ~A, ~B"),
            criterion(false),
            &[(v(1), v(3)), (v(2), v(4))],
        )
        .unwrap();
        assert!(net.to_string().ends_with(
            "not a proof net: every switching falls into 2 parts; keeping every left premise, \
             they are {A ⅋ B[0], ~A[3]} and {B[2], ~B[4]}"
        ));
    }

    /// The links of a structure as sorted pairs, for comparing nets.
    fn links(net: &ProofStructure) -> Vec<(VertexId, VertexId)> {
        let mut links: Vec<(VertexId, VertexId)> = net
            .links()
            .iter()
            .map(|&(x, y)| (x.min(y), x.max(y)))
            .collect();
        links.sort();
        links
    }

    /// A list of links is validated: literals only, dual, each at most
    /// once, inside the forest; and only unit-free MLL has structures. A
    /// single link is validated alike, and one refused changes nothing.
    #[cfg(feature = "parse")]
    #[test]
    fn validation() {
        // ⊢ ~A, A ⊗ ~B, B, A: 0 ~A, 1 ⊗, 2 A, 3 ~B, 4 B, 5 A.
        let f = || forest("A, A -o B |- B, A");
        let net = ProofStructure::from_links(f(), criterion(false), &[(v(0), v(5))]).unwrap();
        assert!(!net.is_complete());
        for (links, error) in [
            (
                vec![(v(0), v(9))],
                NetError::NoVertex {
                    vertex: 9,
                    vertices: 6,
                },
            ),
            (vec![(v(1), v(0))], NetError::NotLiteral { vertex: v(1) }),
            (vec![(v(0), v(3))], NetError::NotDual { x: v(0), y: v(3) }),
            (vec![(v(5), v(2))], NetError::NotDual { x: v(5), y: v(2) }),
            (
                vec![(v(0), v(5)), (v(0), v(2))],
                NetError::LinkedTwice { vertex: v(0) },
            ),
        ] {
            match ProofStructure::from_links(f(), criterion(false), &links) {
                Err(Error::Net(e)) => assert_eq!(*e, error, "{links:?}"),
                other => panic!("{links:?}: {other:?}"),
            }
            let mut net = ProofStructure::new(f(), criterion(false)).unwrap();
            let (&(x, y), made) = links.split_last().unwrap();
            for &(x, y) in made {
                net.link(x, y).unwrap();
            }
            assert_eq!(net.link(x, y), Err(error), "{links:?}");
            assert_eq!(net.links(), made, "{links:?}");
            assert_eq!(net.partner(v(2)), None, "{links:?}");
        }
        let outside = NetError::NoVertex {
            vertex: u32::MAX,
            vertices: 6,
        };
        assert_eq!(
            outside.describe(net.forest()).to_string(),
            "a link names vertex 4294967295, but the structure has 6 vertices"
        );
        for input in ["|- 1", "|- A & B", "|- !A"] {
            assert!(
                matches!(
                    ProofStructure::new(forest(input), criterion(true)),
                    Err(Error::Net(e)) if matches!(*e, NetError::Fragment { .. })
                ),
                "{input}"
            );
        }
        assert!(ProofStructure::new(forest("|-"), criterion(true)).is_ok());
    }

    /// An id the structure does not hold is answered, never a panic: no
    /// partner, joined to nothing, no occurrence; and a scratch sized for
    /// another structure is replaced, so the test answers as with its own.
    #[cfg(feature = "parse")]
    #[test]
    fn foreign_ids_and_scratches() {
        let small = ProofStructure::from_links(forest("a |- a"), criterion(false), &[(v(0), v(1))])
            .unwrap();
        assert_eq!(small.partner(v(99)), None);
        assert!(!small.same_component(v(0), v(99)));
        assert_eq!(small.occurrence(v(2)), None);
        assert_eq!(small.vertex(o(1)), Some(v(1)));
        // ⊢ A ⊗ ~A, B, ~B with both links has a switching cycle.
        let cyclic = ProofStructure::from_links(
            forest("|- A * ~A, B, ~B"),
            criterion(true),
            &[(v(1), v(2)), (v(3), v(4))],
        )
        .unwrap();
        let mut scratch = small.scratch();
        assert!(!cyclic.is_acyclic(&mut scratch));
        assert!(small.is_acyclic(&mut scratch));
    }

    /// The bound and the stop refuse building a net from a proof, judging
    /// it and sequentializing it, with no verdict.
    #[cfg(feature = "parse")]
    #[test]
    #[expect(
        clippy::wildcard_enum_match_arm,
        reason = "a test's other arm is its failure"
    )]
    fn refusals() {
        use crate::search::{Options, prove};
        let s: Sequent = "A * B |- B * A".parse().unwrap();
        let outcome = prove(&s, Mode::CLASSICAL, &Options::default()).unwrap();
        let proof = outcome.verdict.proof().unwrap();
        let tiny = Limits::default().with_memory_bytes(Some(100));
        let refused = |error: Error| match error {
            Error::Net(e) => match *e {
                NetError::Refused { refusal } => refusal,
                other => panic!("{other:?}"),
            },
            other => panic!("{other:?}"),
        };
        let memory = refused(
            ProofStructure::from_proof(proof, Criterion::MLL, &tiny, |_| false).unwrap_err(),
        );
        assert!(matches!(
            memory,
            Refusal::Memory {
                phase: Phase::Net,
                ..
            }
        ));
        let stopped = Refusal::Stopped { phase: Phase::Net };
        let error = ProofStructure::from_proof(proof, Criterion::MLL, &Limits::default(), |_| true);
        assert_eq!(refused(error.unwrap_err()), stopped);
        let net = ProofStructure::from_proof(proof, Criterion::MLL, &Limits::default(), |_| false)
            .unwrap();
        assert_eq!(
            net.is_correct(|_| true),
            Err(NetError::Refused {
                refusal: stopped.clone()
            })
        );
        assert_eq!(
            refused(net.sequentialize(&Limits::default(), |_| true).unwrap_err()),
            stopped
        );
        let memory = refused(net.sequentialize(&tiny, |_| false).unwrap_err());
        assert!(matches!(
            memory,
            Refusal::Memory {
                phase: Phase::Net,
                ..
            }
        ));
        let back = net.sequentialize(&Limits::default(), |_| false).unwrap();
        assert_eq!(back.mode(), Some(Mode::CLASSICAL));
    }
}
