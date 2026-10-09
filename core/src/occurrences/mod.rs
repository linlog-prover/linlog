// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

// A variant a later step adds must not fall into an existing arm.
#![deny(clippy::wildcard_enum_match_arm)]

pub mod reading;
/// Bitsets over occurrence ids.
pub(crate) mod set;

pub use reading::{IllFormula, Reading, ShapeError, Side};
pub(crate) use set::OccSet;

use crate::Error;
use crate::limits::{Limits, Refusal, Space};
use crate::sequents::{Atom, Formula, Kind, Sequent, TermId};
use std::ops::Not;

/// The index of a subformula occurrence in a [`Forest`]: the position of the
/// occurrence in a depth-first preorder walk over the root formulas.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct OccId(u32);

impl OccId {
    /// Wraps a raw occurrence index.
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    /// Returns the raw occurrence index.
    pub const fn get(self) -> u32 {
        self.0
    }

    /// Returns the index as a `usize`, for indexing per-occurrence arrays.
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

/// A member of a sequent of its owner (a proof, a derivation, a session,
/// a goal): below the forest's length the occurrence with that id, and so
/// the same number in every owner. Every public list of a sequent's
/// members holds members; [`Forest`] and [`Reading`] speak of occurrences.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(
    feature = "serialize",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
pub struct Member(u32);

impl Member {
    /// Wraps a raw member index.
    pub const fn new(raw: u32) -> Self {
        Self(raw)
    }

    /// Returns the raw member index.
    pub const fn get(self) -> u32 {
        self.0
    }

    /// Returns the index as a `usize`.
    pub const fn index(self) -> usize {
        self.0 as usize
    }

    /// Returns the occurrence the member is, when it is one of the
    /// forest's own.
    pub fn occurrence(self, forest: &Forest) -> Option<OccId> {
        (self.index() < forest.len()).then_some(OccId(self.0))
    }

    /// Returns the occurrence the member is, for a member of an owner
    /// without a table of its own, which every member of a proposition is.
    pub(crate) const fn occ(self) -> OccId {
        OccId(self.0)
    }
}

impl From<OccId> for Member {
    /// The member that is the occurrence.
    fn from(o: OccId) -> Self {
        Self(o.0)
    }
}

const _: () = assert!(size_of::<Member>() == 4);

/// The raw index that stands for "no occurrence" in the forest's arrays.
const NONE: u32 = u32::MAX;

/// Which literal of an atom `a` an occurrence is: `a` itself or its negation
/// `~a`.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Sign {
    /// The atom `a`, an `Atom` term.
    Atom,
    /// The negation `~a`, a `DualAtom` term.
    Dual,
}

impl Not for Sign {
    type Output = Self;

    /// The other literal of the same atom.
    fn not(self) -> Self {
        match self {
            Sign::Atom => Sign::Dual,
            Sign::Dual => Sign::Atom,
        }
    }
}

/// The polarity of a formula in focused proof search: positive connectives
/// (`⊗ 1 ⊕ 0 !`) have non-invertible rules and are decomposed under focus,
/// negative ones (`⅋ ⊥ & ⊤ ?`) have invertible rules and are decomposed
/// eagerly. A literal's polarity is chosen per atom by the search, under
/// its [`Bias`](crate::search::Bias).
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Polarity {
    /// `⊗ 1 ⊕ 0 !`, and the literals the bias makes positive.
    Positive,
    /// `⅋ ⊥ & ⊤ ?`, and the literals the bias makes negative.
    Negative,
}

impl Not for Polarity {
    type Output = Self;

    /// The other polarity.
    fn not(self) -> Self {
        match self {
            Polarity::Positive => Polarity::Negative,
            Polarity::Negative => Polarity::Positive,
        }
    }
}

impl Kind {
    /// Returns the sign of a literal kind, or `None` for a connective.
    pub const fn sign(self) -> Option<Sign> {
        match self {
            Kind::Atom => Some(Sign::Atom),
            Kind::DualAtom => Some(Sign::Dual),
            Kind::One
            | Kind::Bot
            | Kind::Top
            | Kind::Zero
            | Kind::Tensor
            | Kind::Par
            | Kind::With
            | Kind::Plus
            | Kind::Bang
            | Kind::Quest => None,
        }
    }

    /// Returns the polarity of a connective, or `None` for a literal, whose
    /// polarity is a per-atom choice.
    pub const fn polarity(self) -> Option<Polarity> {
        use Kind::*;
        match self {
            Atom | DualAtom => None,
            Tensor | One | Plus | Zero | Bang => Some(Polarity::Positive),
            Par | Bot | With | Top | Quest => Some(Polarity::Negative),
        }
    }
}

/// The subformula occurrences of a sequent, numbered so that every subtree is
/// a contiguous range of ids, with what proof search asks of each occurrence
/// and of each atom.
///
/// The numbering is a depth-first preorder walk: the roots in the order
/// [`Sequent::roots`] lists them, and below a connective its left subterm
/// before its right one. So a root comes before everything below it, the
/// left child of `o` is `o + 1`, the right child follows the left child's
/// subtree, and the subtree of `o` is exactly the ids `o .. o + size(o)`.
/// Two occurrences of one arena term are distinct ids that share a
/// [`TermId`].
///
/// A forest owns a copy of its sequent, so that an occurrence can be printed
/// as a formula, and every other per-occurrence datum is an array of `u32`
/// or smaller.
///
/// Every accessor that takes an [`OccId`] reads arrays per occurrence and
/// panics for an id past the forest's end, one of another forest:
/// [`Member::occurrence`] and [`len`](Self::len) are the checked way in.
///
/// # Examples
///
#[cfg_attr(feature = "parse", doc = "```")]
#[cfg_attr(not(feature = "parse"), doc = "```ignore")]
/// use linlog::{Forest, Kind, Sequent, Sign};
///
/// // ⊢ ~A, A ⊗ ~B, B
/// let sequent: Sequent = "A, A -o B |- B".parse()?;
/// let forest = Forest::new(&sequent)?;
/// assert_eq!(forest.len(), 5);
///
/// let tensor = forest.roots()[1];
/// assert_eq!(forest.kind(tensor), Kind::Tensor);
/// assert_eq!(forest.size(tensor), 3);
/// let (left, right) = (forest.left(tensor).unwrap(), forest.right(tensor).unwrap());
/// assert_eq!(forest.formula(left).to_string(), "A");
/// assert_eq!(forest.formula(right).to_string(), "~B");
/// assert_eq!(forest.parent(right), Some(tensor));
/// assert_eq!(forest.lca(left, right), Some(tensor));
///
/// // Every literal of an atom, by sign, in id order.
/// let a = sequent.atom("A").unwrap();
/// assert_eq!(forest.literals(a, Sign::Atom), [left]);
/// assert_eq!(forest.literals(a, Sign::Dual), [forest.roots()[0]]);
/// # Ok::<(), linlog::Error>(())
/// ```
#[derive(Clone, Debug)]
pub struct Forest {
    /// The sequent the occurrences are of.
    sequent: Sequent,
    /// The occurrence of each root formula, in the sequent's order.
    roots: Box<[OccId]>,
    /// Per occurrence, its arena term.
    term: Box<[TermId]>,
    /// Per occurrence, the kind of its term.
    kind: Box<[Kind]>,
    /// Per occurrence, its parent, or `NONE` for a root.
    parent: Box<[u32]>,
    /// Per occurrence, the root it lies below (itself for a root).
    root: Box<[OccId]>,
    /// Per occurrence, the number of occurrences in its subtree, itself
    /// included.
    size: Box<[u32]>,
    /// Per occurrence, the number of ancestors it has.
    depth: Box<[u32]>,
    /// Every literal occurrence, grouped by atom, then by sign (`Atom` first),
    /// in ascending id order within a group.
    literals: Box<[OccId]>,
    /// Where each group of `literals` starts: group `2a` holds the `Atom`
    /// literals of atom `a`, group `2a + 1` its `DualAtom` literals, and the
    /// last entry is the total.
    literal_start: Box<[u32]>,
}

impl Forest {
    /// The most occurrences a forest can hold: the ids are `u32`, and the
    /// last one stands for "no occurrence".
    pub(crate) const MOST: u64 = NONE as u64 - 1;

    /// Builds the forest of a sequent within the default
    /// [`Limits::occurrences`], keeping a copy of the sequent. A forest
    /// takes about 25 bytes per occurrence, and an arena that shares its
    /// subterms unfolds to exponentially more occurrences than it has
    /// terms, so a sequent of a few hundred bytes can ask for gigabytes;
    /// the bound refuses it before anything of that size is built.
    ///
    /// # Errors
    ///
    /// [`Refusal::Occurrences`] past the bound, and [`Refusal::Index`] for
    /// more occurrences than a forest indexes (2³² − 2), whatever the
    /// bound.
    pub fn new(sequent: &Sequent) -> Result<Self, Error> {
        Self::within(sequent, &Limits::default())
    }

    /// Builds the forest of a sequent of at most `limits.occurrences`
    /// subformula occurrences ([`Sequent::occurrences`] counts them),
    /// keeping a copy of the sequent, as [`new`](Self::new) does.
    ///
    /// # Errors
    ///
    /// As [`new`](Self::new)'s, under `limits`.
    pub fn within(sequent: &Sequent, limits: &Limits) -> Result<Self, Error> {
        let sizes = Self::measure(sequent, limits)?;
        Ok(Self::build(sequent.clone(), &sizes))
    }

    /// Builds the forest of a sequent as [`within`](Self::within) does,
    /// taking the sequent instead of copying it: of a sequent of millions
    /// of terms the copy is memory worth saving.
    ///
    /// # Errors
    ///
    /// As [`within`](Self::within)'s.
    pub fn from_owned(sequent: Sequent, limits: &Limits) -> Result<Self, Error> {
        let sizes = Self::measure(&sequent, limits)?;
        Ok(Self::build(sequent, &sizes))
    }

    /// Returns the number of occurrences below every arena term of a
    /// sequent, itself included, if the sequent has no more occurrences
    /// than the limits and a forest allow.
    fn measure(sequent: &Sequent, limits: &Limits) -> Result<Vec<u64>, Error> {
        let sizes = sequent.sizes();
        let occurrences = sequent
            .roots()
            .iter()
            .fold(0u64, |sum, r| sum.saturating_add(sizes[r.index()]));
        if let Some(limit) = limits.occurrences
            && occurrences > limit
        {
            return Err(Error::Refused(Refusal::Occurrences { occurrences, limit }));
        }
        if occurrences > Self::MOST {
            return Err(Error::Refused(Refusal::Index {
                what: Space::Occurrence,
                count: occurrences,
                most: Self::MOST,
            }));
        }
        Ok(sizes)
    }

    /// Returns the sequent the forest was built from.
    pub fn sequent(&self) -> &Sequent {
        &self.sequent
    }

    /// Returns the number of occurrences, which is the width of an
    /// occurrence set of this forest.
    pub fn len(&self) -> usize {
        self.term.len()
    }

    /// Returns whether the sequent has no formula.
    pub fn is_empty(&self) -> bool {
        self.term.is_empty()
    }

    /// Returns every occurrence id in ascending order.
    pub fn ids(&self) -> impl DoubleEndedIterator<Item = OccId> + ExactSizeIterator {
        (0..self.term.len() as u32).map(OccId::new)
    }

    /// Returns the roots as members, in the order the sequent lists them.
    #[cfg(any(feature = "latex", feature = "typst", feature = "svg"))]
    pub(crate) fn root_members(&self) -> Vec<Member> {
        self.roots().iter().copied().map(Member::from).collect()
    }

    /// Returns the occurrence of each root formula, in the order the sequent
    /// lists them.
    pub fn roots(&self) -> &[OccId] {
        &self.roots
    }

    /// Returns whether `goal` is the roots, in any order: the sequent
    /// itself.
    pub(crate) fn is_roots(&self, goal: &[OccId]) -> bool {
        if goal.len() != self.roots.len() {
            return false;
        }
        if *goal == *self.roots {
            return true;
        }
        let mut sorted = goal.to_vec();
        sorted.sort_unstable();
        let mut roots = self.roots.to_vec();
        roots.sort_unstable();
        sorted == roots
    }

    /// Returns the arena term of an occurrence.
    ///
    /// # Panics
    ///
    /// For an id of another forest, past this one's end.
    pub fn term(&self, o: OccId) -> TermId {
        self.term[o.index()]
    }

    /// Returns the kind of an occurrence's term.
    ///
    /// # Panics
    ///
    /// For an id of another forest, past this one's end.
    pub fn kind(&self, o: OccId) -> Kind {
        self.kind[o.index()]
    }

    /// Returns the occurrence as a value that prints its formula.
    ///
    /// # Panics
    ///
    /// For an id of another forest, past this one's end.
    pub fn formula(&self, o: OccId) -> Formula<'_> {
        self.sequent.formula(self.term(o))
    }

    /// Returns the parent of an occurrence, or `None` for a root.
    ///
    /// # Panics
    ///
    /// For an id of another forest, past this one's end.
    pub fn parent(&self, o: OccId) -> Option<OccId> {
        match self.parent[o.index()] {
            NONE => None,
            p => Some(OccId::new(p)),
        }
    }

    /// Returns the root formula an occurrence lies below, or itself if it is
    /// a root.
    ///
    /// # Panics
    ///
    /// For an id of another forest, past this one's end.
    pub fn root(&self, o: OccId) -> OccId {
        self.root[o.index()]
    }

    /// Returns whether two occurrences lie below the same root formula.
    ///
    /// # Panics
    ///
    /// For an id of another forest, past this one's end.
    pub fn same_root(&self, x: OccId, y: OccId) -> bool {
        self.root(x) == self.root(y)
    }

    /// Returns the number of occurrences in the subtree of `o`, itself
    /// included.
    ///
    /// # Panics
    ///
    /// For an id of another forest, past this one's end.
    pub fn size(&self, o: OccId) -> u32 {
        self.size[o.index()]
    }

    /// Returns the number of ancestors of an occurrence: 0 for a root.
    ///
    /// # Panics
    ///
    /// For an id of another forest, past this one's end.
    pub fn depth(&self, o: OccId) -> u32 {
        self.depth[o.index()]
    }

    /// Returns whether the occurrence is a literal, `a` or `~a`.
    ///
    /// # Panics
    ///
    /// For an id of another forest, past this one's end.
    pub fn is_literal(&self, o: OccId) -> bool {
        self.kind(o).is_literal()
    }

    /// Returns the atom of a literal occurrence, or `None` for a connective.
    ///
    /// # Panics
    ///
    /// For an id of another forest, past this one's end.
    pub fn atom(&self, o: OccId) -> Option<Atom> {
        self.sequent.term(self.term(o)).atom()
    }

    /// Returns the sign of a literal occurrence, or `None` for a connective.
    ///
    /// # Panics
    ///
    /// For an id of another forest, past this one's end.
    pub fn sign(&self, o: OccId) -> Option<Sign> {
        self.kind(o).sign()
    }

    /// Returns whether `x` and `y` are an atom and its negation, which an
    /// axiom pairs: the one test of axiom partners, which atoms with
    /// arguments will make a comparison of instances.
    pub(crate) fn dual_literals(&self, x: OccId, y: OccId) -> bool {
        self.is_literal(x) && self.atom(x) == self.atom(y) && self.sign(x) != self.sign(y)
    }

    /// Returns the first child: the only subformula of `!` and `?`, the left
    /// one of a binary connective, and `None` for a literal or a unit.
    ///
    /// # Panics
    ///
    /// For an id of another forest, past this one's end.
    pub fn left(&self, o: OccId) -> Option<OccId> {
        (self.kind(o).arity() >= 1).then(|| OccId::new(o.0 + 1))
    }

    /// Returns the right child of a binary connective, and `None` otherwise.
    ///
    /// # Panics
    ///
    /// For an id of another forest, past this one's end.
    pub fn right(&self, o: OccId) -> Option<OccId> {
        (self.kind(o).arity() == 2).then(|| {
            let left = o.0 + 1;
            OccId::new(left + self.size[left as usize])
        })
    }

    /// Returns the children of an occurrence in order, left before right.
    ///
    /// # Panics
    ///
    /// For an id of another forest, past this one's end.
    pub fn children(&self, o: OccId) -> impl Iterator<Item = OccId> {
        self.left(o).into_iter().chain(self.right(o))
    }

    /// Returns the subtree of `o` in preorder: `o` itself first, then every
    /// occurrence below it, which are the ids `o .. o + size(o)`.
    ///
    /// # Panics
    ///
    /// For an id of another forest, past this one's end.
    pub fn subtree(&self, o: OccId) -> impl DoubleEndedIterator<Item = OccId> + ExactSizeIterator {
        (o.0..o.0 + self.size(o)).map(OccId::new)
    }

    /// Returns whether `o` is `ancestor` itself or lies below it.
    ///
    /// # Panics
    ///
    /// For an id of another forest, past this one's end.
    pub fn is_below(&self, o: OccId, ancestor: OccId) -> bool {
        ancestor.0 <= o.0 && o.0 < ancestor.0 + self.size(ancestor)
    }

    /// Returns the lowest common ancestor of two occurrences, or `None` if
    /// they lie below different roots. Walks up from `x`, so it costs the
    /// depth of `x` at most.
    ///
    /// # Panics
    ///
    /// For an id of another forest, past this one's end.
    pub fn lca(&self, x: OccId, y: OccId) -> Option<OccId> {
        if !self.same_root(x, y) {
            return None;
        }
        let mut a = x;
        while !self.is_below(y, a) {
            a = self.parent(a)?;
        }
        Some(a)
    }

    /// Returns the occurrences of one literal of an atom, `a` or `~a`, in
    /// ascending id order: the candidate partners in an axiom of a literal
    /// of the other sign, which `dual_literals` decides.
    pub fn literals(&self, atom: Atom, sign: Sign) -> &[OccId] {
        let group = 2 * atom.index() + sign as usize;
        let (start, end) = (self.literal_start[group], self.literal_start[group + 1]);
        &self.literals[start as usize..end as usize]
    }

    /// Returns every literal occurrence, grouped by atom, `a` before `~a`
    /// within an atom, and in ascending id order within a group.
    pub fn all_literals(&self) -> &[OccId] {
        &self.literals
    }
}

impl Forest {
    /// Builds the forest of a sequent from the number of occurrences below
    /// each of its arena terms, itself included, which add up to no more
    /// than a forest can hold.
    fn build(sequent: Sequent, term_size: &[u64]) -> Self {
        let terms = sequent.terms();
        let n = sequent
            .roots()
            .iter()
            .map(|r| term_size[r.index()] as usize)
            .sum();

        let mut roots = Vec::with_capacity(sequent.roots().len());
        let mut term = Vec::with_capacity(n);
        let mut kind = Vec::with_capacity(n);
        let mut parent = Vec::with_capacity(n);
        let mut root = Vec::with_capacity(n);
        let mut size = Vec::with_capacity(n);
        let mut depth = Vec::with_capacity(n);

        // Preorder: the right child is pushed first so the left one is
        // visited, with its whole subtree, before it.
        let mut stack: Vec<(TermId, u32, u32)> = Vec::new();
        for &r in sequent.roots() {
            let root_id = OccId::new(term.len() as u32);
            roots.push(root_id);
            stack.push((r, NONE, 0));
            while let Some((t, p, d)) = stack.pop() {
                let o = term.len() as u32;
                let node = terms[t.index()];
                term.push(t);
                kind.push(node.kind());
                parent.push(p);
                root.push(root_id);
                size.push(term_size[t.index()] as u32);
                depth.push(d);
                let subterms: Vec<TermId> = node.subterms().collect();
                for &k in subterms.iter().rev() {
                    stack.push((k, o, d + 1));
                }
            }
        }
        debug_assert_eq!(term.len(), n);

        // Literals: count per atom and sign, then lay the groups out by a
        // counting sort.
        let num_atoms = sequent.atom_names().len();
        let mut literal_start = vec![0u32; 2 * num_atoms + 1];
        for (o, k) in kind.iter().enumerate() {
            if let Some(s) = k.sign() {
                let a = terms[term[o].index()].atom().unwrap();
                literal_start[2 * a.index() + s as usize + 1] += 1;
            }
        }
        for g in 1..literal_start.len() {
            literal_start[g] += literal_start[g - 1];
        }
        let mut next = literal_start.clone();
        let mut literals = vec![OccId::new(NONE); literal_start[2 * num_atoms] as usize];
        for (o, k) in kind.iter().enumerate() {
            if let Some(s) = k.sign() {
                let a = terms[term[o].index()].atom().unwrap();
                let group = 2 * a.index() + s as usize;
                literals[next[group] as usize] = OccId::new(o as u32);
                next[group] += 1;
            }
        }

        Self {
            sequent,
            roots: roots.into_boxed_slice(),
            term: term.into_boxed_slice(),
            kind: kind.into_boxed_slice(),
            parent: parent.into_boxed_slice(),
            root: root.into_boxed_slice(),
            size: size.into_boxed_slice(),
            depth: depth.into_boxed_slice(),
            literals: literals.into_boxed_slice(),
            literal_start: literal_start.into_boxed_slice(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sequents::Term;

    /// Wraps a raw id.
    const fn o(id: u32) -> OccId {
        OccId::new(id)
    }

    /// Builds the forest of `input`, or panics with the parse error.
    #[cfg(feature = "parse")]
    fn forest(input: &str) -> Forest {
        let s: Sequent = input.parse().unwrap_or_else(|e| panic!("{input:?}: {e}"));
        Forest::new(&s).unwrap()
    }

    /// Checks the invariants that hold in every forest: preorder ranges,
    /// parents against children, depths, roots, sizes, and the literal
    /// tables against the occurrences.
    fn check_invariants(f: &Forest) {
        let n = f.len() as u32;
        assert_eq!(f.ids().count(), n as usize);
        assert_eq!(
            f.roots().iter().map(|&r| f.size(r)).sum::<u32>(),
            n,
            "the roots' subtrees partition the ids"
        );
        for (i, &r) in f.roots().iter().enumerate() {
            let expected = f.roots()[..i].iter().map(|&q| f.size(q)).sum::<u32>();
            assert_eq!(r, o(expected), "root {i} starts after the roots before it");
            assert_eq!(f.parent(r), None);
            assert_eq!(f.depth(r), 0);
            assert_eq!(f.root(r), r);
            assert_eq!(f.term(r), f.sequent().roots()[i]);
        }
        for x in f.ids() {
            let node = f.sequent().term(f.term(x));
            assert_eq!(f.kind(x), node.kind());
            let children: Vec<OccId> = f.children(x).collect();
            assert_eq!(children.len(), node.kind().arity() as usize);
            assert_eq!(children.first().copied(), f.left(x));
            assert_eq!(children.get(1).copied(), f.right(x));
            let subterms: Vec<TermId> = node.subterms().collect();
            for (&c, &t) in children.iter().zip(&subterms) {
                assert_eq!(f.parent(c), Some(x), "the child's parent is the node");
                assert_eq!(f.depth(c), f.depth(x) + 1);
                assert_eq!(f.root(c), f.root(x));
                assert_eq!(f.term(c), t, "children come in the term's order");
                assert!(f.is_below(c, x));
            }
            assert_eq!(
                f.size(x),
                1 + children.iter().map(|&c| f.size(c)).sum::<u32>()
            );
            if let Some(l) = f.left(x) {
                assert_eq!(l, o(x.get() + 1), "the left child follows its parent");
            }
            if let (Some(l), Some(r)) = (f.left(x), f.right(x)) {
                assert_eq!(
                    r,
                    o(l.get() + f.size(l)),
                    "the right child follows the left subtree"
                );
            }
            // Every id in the range descends from x through parents, and no
            // id outside does.
            for y in f.ids() {
                let mut a = Some(y);
                while let Some(b) = a
                    && b != x
                {
                    a = f.parent(b);
                }
                assert_eq!(a == Some(x), f.is_below(y, x), "{y:?} below {x:?}");
                assert_eq!(f.subtree(x).any(|z| z == y), f.is_below(y, x));
            }
            match f.parent(x) {
                Some(p) => assert_eq!(f.depth(x), f.depth(p) + 1),
                None => assert!(f.roots().contains(&x)),
            }
            // Atom and sign agree with the term.
            assert_eq!(f.atom(x), node.atom());
            assert_eq!(f.sign(x), node.kind().sign());
            assert_eq!(f.is_literal(x), node.kind().is_literal());
        }
        // The literal tables list exactly the literal occurrences, sorted.
        let mut all: Vec<OccId> = Vec::new();
        for a in 0..f.sequent().atom_names().len() {
            let atom = Atom::new(a as u32);
            for sign in [Sign::Atom, Sign::Dual] {
                let list = f.literals(atom, sign);
                assert!(list.windows(2).all(|w| w[0] < w[1]), "sorted");
                for &l in list {
                    assert_eq!(f.atom(l), Some(atom));
                    assert_eq!(f.sign(l), Some(sign));
                }
                all.extend_from_slice(list);
            }
        }
        assert_eq!(all, f.all_literals());
        let mut literal_ids: Vec<OccId> = f.ids().filter(|&x| f.is_literal(x)).collect();
        all.sort();
        literal_ids.sort();
        assert_eq!(all, literal_ids);
    }

    /// The forest of the empty sequent has nothing in it.
    #[test]
    fn empty() {
        let f = Forest::new(&Sequent::new()).unwrap();
        assert!(f.is_empty());
        assert_eq!(f.len(), 0);
        assert!(f.roots().is_empty());
        assert!(f.all_literals().is_empty());
        check_invariants(&f);
    }

    /// The numbering of `⊢ ~A, A ⊗ ~B, B` is preorder with the left subterm
    /// first, and the queries return what the layout says.
    #[cfg(feature = "parse")]
    #[test]
    fn layout() {
        let f = forest("A, A -o B |- B");
        check_invariants(&f);
        assert_eq!(f.len(), 5);
        assert_eq!(f.roots(), [o(0), o(1), o(4)]);
        let kinds: Vec<Kind> = f.ids().map(|x| f.kind(x)).collect();
        use Kind::*;
        assert_eq!(kinds, [DualAtom, Tensor, Atom, DualAtom, Atom]);
        assert_eq!(f.formula(o(1)).to_string(), "A ⊗ ~B");
        assert_eq!(f.left(o(1)), Some(o(2)));
        assert_eq!(f.right(o(1)), Some(o(3)));
        assert_eq!(f.subtree(o(1)).collect::<Vec<_>>(), [o(1), o(2), o(3)]);
        assert_eq!(f.size(o(1)), 3);
        assert_eq!(f.depth(o(3)), 1);
        assert_eq!(f.root(o(3)), o(1));
        assert!(f.same_root(o(2), o(3)));
        assert!(!f.same_root(o(0), o(2)));
    }

    /// Nested and repeated formulas keep every invariant, including shared
    /// subterms and repeated roots.
    #[cfg(feature = "parse")]
    #[test]
    fn invariants_on_a_table() {
        for input in [
            "|- A",
            "|- A, A",
            "|- A * A",
            "|- (A * B), (A * B)",
            "|- !(A & B) + ?(A * B), 1, bot, top, 0",
            "A -o B -o C, (A -o B) -o C |- (A * B) + (0 & top)",
            "!A, ?B, A * B, C par D |- ~A, B^, A & B, C + D",
            "|- ((A * B) * (A * B)) * ((A * B) * (A * B))",
        ] {
            check_invariants(&forest(input));
        }
    }

    /// The lowest common ancestor is the deepest node above both, and
    /// nothing across roots.
    #[cfg(feature = "parse")]
    #[test]
    fn lowest_common_ancestor() {
        // ⊢ (A ⊗ B) ⅋ (C ⊗ D), E
        let f = forest("|- (A * B) par (C * D), E");
        check_invariants(&f);
        assert_eq!(f.lca(o(2), o(3)), Some(o(1)));
        assert_eq!(f.lca(o(2), o(5)), Some(o(0)));
        assert_eq!(f.lca(o(3), o(6)), Some(o(0)));
        assert_eq!(f.lca(o(1), o(4)), Some(o(0)));
        assert_eq!(f.lca(o(0), o(6)), Some(o(0)));
        assert_eq!(f.lca(o(6), o(0)), Some(o(0)));
        assert_eq!(f.lca(o(5), o(5)), Some(o(5)));
        assert_eq!(f.lca(o(2), o(7)), None);
        assert_eq!(f.lca(o(7), o(7)), Some(o(7)));
    }

    /// Every literal occurrence is listed once, by atom and sign.
    #[cfg(feature = "parse")]
    #[test]
    fn literals() {
        // ⊢ ~A, ~A, A, B, ~C ⅋ C, ~C ⅋ C
        let f = forest("A, A |- A, B, C -o C, C -o C");
        check_invariants(&f);
        let s = f.sequent();
        let (a, b, c) = (
            s.atom("A").unwrap(),
            s.atom("B").unwrap(),
            s.atom("C").unwrap(),
        );
        assert_eq!(f.literals(a, Sign::Atom), [o(2)]);
        assert_eq!(f.literals(a, Sign::Dual), [o(0), o(1)]);
        assert_eq!(f.literals(b, Sign::Atom), [o(3)]);
        assert_eq!(f.literals(b, Sign::Dual), []);
        assert_eq!(f.literals(c, Sign::Atom), [o(6), o(9)]);
        assert_eq!(f.literals(c, Sign::Dual), [o(5), o(8)]);
    }

    /// Connectives have their fixed polarities.
    #[cfg(feature = "parse")]
    #[test]
    fn connective_polarities() {
        let f = forest("|- (A * B) par (A & B) + (!A par ?B), 1, bot, top, 0");
        check_invariants(&f);
        let by_kind = |k: Kind| {
            f.ids()
                .filter(|&x| f.kind(x) == k)
                .map(|x| f.kind(x).polarity().unwrap())
                .collect::<Vec<_>>()
        };
        use Kind::*;
        use Polarity::*;
        for (k, p) in [
            (Tensor, Positive),
            (Plus, Positive),
            (One, Positive),
            (Zero, Positive),
            (Bang, Positive),
            (Par, Negative),
            (With, Negative),
            (Bot, Negative),
            (Top, Negative),
            (Quest, Negative),
        ] {
            assert!(!by_kind(k).is_empty(), "{k:?} occurs");
            assert!(by_kind(k).iter().all(|&q| q == p), "{k:?} is {p:?}");
        }
    }

    /// A shared subterm is walked once per occurrence, and a deeply shared
    /// arena is refused rather than mis-numbered.
    #[test]
    fn sharing() {
        // Atom(0) shared by two roots and inside a tensor of itself.
        let s = Sequent {
            terms: vec![
                Term::Atom(Atom::new(0)),
                Term::Tensor(TermId::new(0), TermId::new(0)),
            ],
            roots: vec![TermId::new(1), TermId::new(0), TermId::new(1)],
            atoms: vec!["A".into()],
            antecedents: None,
        };
        let f = Forest::new(&s).unwrap();
        check_invariants(&f);
        assert_eq!(f.len(), 7);
        assert_eq!(f.roots(), [o(0), o(3), o(4)]);
        assert_eq!(
            f.literals(Atom::new(0), Sign::Atom),
            [o(1), o(2), o(3), o(5), o(6)]
        );

        // A limit admits a sequent of exactly that many occurrences.
        assert_eq!(s.occurrences(), 7);
        let limit = |n| Limits::default().with_occurrences(n);
        assert_eq!(Forest::within(&s, &limit(Some(7))).unwrap().len(), 7);
        assert!(matches!(
            Forest::within(&s, &limit(Some(6))),
            Err(Error::Refused(Refusal::Occurrences {
                occurrences: 7,
                limit: 6
            }))
        ));

        // No limit admits more occurrences than a `u32` numbers.
        let s = doubling(40);
        assert_eq!(s.occurrences(), (1 << 41) - 1);
        assert!(matches!(
            Forest::within(&s, &limit(None)),
            Err(Error::Refused(Refusal::Index { what: Space::Occurrence, count, most }))
                if count == (1 << 41) - 1 && most == (1 << 32) - 2
        ));
        assert_eq!(doubling(100).occurrences(), u64::MAX, "the count saturates");
    }

    /// Returns `⊢ 1` under `levels` tensors, each of the term below with
    /// itself: `levels + 1` terms that unfold to `2^(levels + 1) − 1`
    /// occurrences.
    fn doubling(levels: u32) -> Sequent {
        let mut terms = vec![Term::One];
        for k in 0..levels {
            terms.push(Term::Tensor(TermId::new(k), TermId::new(k)));
        }
        Sequent {
            terms,
            roots: vec![TermId::new(levels)],
            atoms: vec![],
            antecedents: None,
        }
    }

    /// A small arena that unfolds past the default limit is refused before
    /// anything of the unfolding's size is built, borrowed or owned: the
    /// forest of this one would take gigabytes and seconds.
    #[test]
    fn refuses_an_unfolding_past_the_default_limit() {
        let s = doubling(26);
        let refused = |result: Result<Forest, Error>| {
            matches!(
                result,
                Err(Error::Refused(crate::limits::Refusal::Occurrences { occurrences, limit }))
                    if occurrences == (1 << 27) - 1 && limit == Limits::DEFAULT_OCCURRENCES
            )
        };
        let start = std::time::Instant::now();
        assert!(refused(Forest::new(&s)));
        assert!(refused(Forest::from_owned(s, &Limits::default())));
        assert!(start.elapsed() < std::time::Duration::from_secs(1));
    }
}
