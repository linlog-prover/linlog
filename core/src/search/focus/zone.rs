// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! What the focused engine is generic over: the members of its zones, the
//! zones themselves, and how the literals among the members meet their
//! duals. [`Ground`] is the instance whose members are the forest's
//! occurrences, the propositional search; its methods are the engine's
//! own code, inlined into it.

use super::choices::Choices;
use super::classes::Classes;
use super::context::Context;
use super::framed::{Mark, Unified};
use super::scratch::{Pooled, Pools};
use crate::Instances;
use crate::occurrences::{Forest, Member, OccId, OccSet};
use crate::proofs::Node;
use crate::search::memory::Charged;
use crate::search::{Options, Reason};
use crate::sequents::{Eigen, FoTermId};
use std::fmt::Debug;
use std::hash::Hash;

/// The members of the engine's zones and the seam where its literals meet:
/// the initial rules, the forced splits' lookups of duals, the copy
/// heuristic, the classes, and what the memo and the loop check may key.
/// The forest is passed to every call that reads it, since a pool's
/// workers hold no borrow of their spawner's zone.
pub(super) trait Zone: Sized + Send + Sync {
    /// Whether the members are instances of occurrences under binders,
    /// with the terms the binders were opened with: the engine then runs
    /// the quantifier rules, closes its identities by unification and
    /// asks a premise for another answer where the next one fails. The
    /// hooks below that only such a zone has are never called otherwise.
    const FIRST_ORDER: bool = false;

    /// A member of a zone, which names its occurrence.
    type Member: Copy + Eq + Ord + Hash + Debug + Send + Sync + Into<Member> + 'static;
    /// The linear zone, a multiset of members.
    type Linear: Linear<Self::Member> + 'static;
    /// The unrestricted zone, a set of members.
    type Unrestricted: Unrestricted<Self::Member> + 'static;
    /// Where the marks of the members dual to a literal are kept.
    type Duals: Copy;

    /// The zone of a search of the forest under the options.
    fn new(forest: &Forest, options: &Options) -> Self;

    /// The zone of a worker of a pool, spawned from this one.
    #[cfg(feature = "parallel")]
    fn fork(&self) -> Self;

    /// The member a goal's occurrence stands for.
    fn root(&self, o: OccId) -> Self::Member;

    /// The occurrence of a member.
    fn occurrence(&self, m: Self::Member) -> OccId;

    /// The member of an occurrence below a member's occurrence.
    fn child(&self, m: Self::Member, child: OccId) -> Self::Member;

    /// The first member dual to the literal `literal` in the unrestricted
    /// zone `within`.
    fn dual_in(
        &self,
        forest: &Forest,
        literal: Self::Member,
        within: &Self::Unrestricted,
    ) -> Option<Self::Member>;

    /// The first member dual to `literal` in `rest`, the context of a
    /// chain of forced splits, read on from where the chain's last lookup
    /// of that literal ended.
    fn dual_from(
        &self,
        forest: &Forest,
        literal: Self::Member,
        rest: &Self::Linear,
        cursors: &mut Cursors,
    ) -> Option<Self::Member>;

    /// Marks the literals among the members of a stable sequent, under a
    /// stamp of their own, with the first member of each, charging what
    /// the marks allocate to `scratch`.
    fn mark_literals(&mut self, forest: &Forest, members: &[Self::Member], scratch: &mut Charged);

    /// Where the marks of the members dual to `literal` are, or `None`
    /// when it is no literal.
    fn duals(&self, forest: &Forest, literal: Self::Member) -> Option<Self::Duals>;

    /// The first dual member that the last marking marked.
    fn marked(&self, duals: Self::Duals) -> Option<Self::Member>;

    /// Whether the duals are yet to be looked up in `Θ` under the last
    /// marking; they count as looked up from then on.
    fn first_lookup(&mut self, duals: Self::Duals) -> bool;

    /// Whether a formula of `Θ` has a literal below it whose dual the
    /// last marking marked.
    fn meets(&self, forest: &Forest, formula: Self::Member) -> bool;

    /// The class of a member: the first member interchangeable with it,
    /// itself when it has none.
    fn class(&self, classes: &Classes, m: Self::Member) -> Self::Member;

    /// Whether two members are interchangeable.
    fn same(&self, classes: &Classes, x: Self::Member, y: Self::Member) -> bool;

    /// Whether the memo and the loop check may key a stable sequent of
    /// these zones, which they key by its occurrences
    /// ([`Linear::ground`], [`Unrestricted::ground`]).
    fn memoizes(&self, theta: &Self::Unrestricted, gamma: &Self::Linear) -> bool;

    /// The pool the words of the memo's keys are taken from.
    fn word_lists(pools: &mut Pools<Self>) -> &mut Vec<Pooled<OccId>>;

    /// The members of a linear zone that is keyed by `words`, its members
    /// each once ([`Context::list_into`]), when they are its words.
    fn listed(words: &[OccId]) -> Option<&[Self::Member]>;

    /// Where the bindings, the instances and the eigenvariables stand,
    /// for [`Self::undo`].
    fn mark(&self) -> Mark {
        unreachable!("a zone without binders binds nothing")
    }

    /// Takes back every binding, instance and eigenvariable made since
    /// `mark`.
    fn undo(&mut self, _mark: Mark) {
        unreachable!("a zone without binders binds nothing")
    }

    /// Begins a level of the copy bound's deepening: no first-order
    /// bound has cut it yet.
    fn begin_level(&mut self) {}

    /// The first-order bound that cut a branch of the level, if one did.
    fn cut(&self) -> Option<Reason> {
        None
    }

    /// The `∀` rule on the binder `binder`: its body's member under a
    /// fresh eigenvariable, which [`Self::introduced`] hands back; `None`
    /// where a bound cuts the branch.
    ///
    /// # Errors
    ///
    /// [`Reason::IndexLimit`] where the members outgrow their ids.
    fn forall(
        &mut self,
        _forest: &Forest,
        _binder: Self::Member,
    ) -> Result<Option<Self::Member>, Reason> {
        unreachable!("a zone without binders opens none")
    }

    /// The eigenvariable of the latest `∀` rule whose node is not built.
    fn introduced(&mut self) -> Eigen {
        unreachable!("a zone without binders opens none")
    }

    /// The `∃` rule on the binder `binder`: its body's member under a
    /// fresh metavariable, with the metavariable's term as the witness;
    /// `None` where a bound cuts the branch.
    ///
    /// # Errors
    ///
    /// [`Reason::IndexLimit`] where the members outgrow their ids.
    fn exists(
        &mut self,
        _forest: &Forest,
        _binder: Self::Member,
    ) -> Result<Option<(Self::Member, FoTermId)>, Reason> {
        unreachable!("a zone without binders opens none")
    }

    /// Whether two members are literals of one predicate and of opposite
    /// signs, so that an axiom may join them where their terms unify.
    fn opposite(&self, _forest: &Forest, _p: Self::Member, _q: Self::Member) -> bool {
        unreachable!("a zone without binders unifies nothing")
    }

    /// Unifies the terms of two opposite literals, binding
    /// metavariables, and keeps the bindings where no answer excluded
    /// below fits them.
    fn unify(&mut self, _forest: &Forest, _p: Self::Member, _q: Self::Member) -> Unified {
        unreachable!("a zone without binders unifies nothing")
    }

    /// Opens the answers of a rule's first premise, whose second premise
    /// has the members given: `false` where they hold no unbound
    /// metavariable, so that no binding of the first changes the second;
    /// otherwise the answers found to fail are excluded while the first
    /// premise is searched, from [`Self::record`] on, until
    /// [`Self::close_answers`].
    fn open_answers(
        &mut self,
        _forest: &Forest,
        _members: &mut dyn Iterator<Item = Self::Member>,
    ) -> bool {
        unreachable!("a zone without binders has one answer")
    }

    /// Whether the answers last opened are excluded while the first
    /// premise is searched.
    fn searching_first(&mut self, _first: bool) {
        unreachable!("a zone without binders has one answer")
    }

    /// Records the answer the first premise gave last, under which the
    /// second failed, to be excluded from then on, before [`Self::undo`]
    /// takes it back; returns the bytes it holds, which the engine
    /// charges until [`Self::close_answers`].
    fn record(&mut self, _forest: &Forest) -> usize {
        unreachable!("a zone without binders has one answer")
    }

    /// Whether the bindings as they stand, before the first premise once
    /// [`Self::undo`] restored them, are excluded already: then every
    /// answer would be, and none is left to search for.
    fn exhausted(&mut self, _forest: &Forest) -> bool {
        unreachable!("a zone without binders has one answer")
    }

    /// Closes the answers last opened.
    fn close_answers(&mut self) {
        unreachable!("a zone without binders has one answer")
    }

    /// Writes the keys of the stable sequent `⊢ Θ ; Γ` by instance: its
    /// own, which names its variables as the search does, and its
    /// canonical one, the same up to a renaming of them; false where the
    /// sequent is to be decided without a key.
    fn keys(
        &mut self,
        _forest: &Forest,
        _theta: &Self::Unrestricted,
        _gamma: &Self::Linear,
        _own: &mut Vec<OccId>,
        _canonical: &mut Vec<OccId>,
    ) -> bool {
        unreachable!("a zone without binders keys its sequents by occurrence")
    }

    /// Whether some answer is excluded while its first premise is
    /// searched: a failure found then may rest on the exclusion, so it is
    /// no fact about its sequent.
    fn excluding(&self) -> bool {
        unreachable!("a zone without binders has one answer")
    }

    /// The terms the zone's walks visited since this was last asked:
    /// work that grows with the size of the terms, which the engine counts
    /// and polls its stop on.
    fn take_steps(&mut self) -> u64 {
        unreachable!("a zone without binders walks no term")
    }

    /// The unifications of two literals the zone tried and the
    /// metavariables it bound since it was made.
    fn counts(&self) -> (u64, u64) {
        unreachable!("a zone without binders unifies nothing")
    }

    /// The bytes the zone's tables held when last asked and hold now,
    /// which the engine charges to its account.
    fn held(&mut self) -> (usize, usize) {
        unreachable!("a zone without binders holds no table")
    }

    /// Closes the proof found, renaming its members and witnesses to an
    /// instance table's, every metavariable left unbound a fresh
    /// constant.
    ///
    /// # Errors
    ///
    /// [`Reason::IndexLimit`] where the table outgrows its ids.
    fn close(&self, _forest: &Forest, _nodes: &mut [Node]) -> Result<Instances, Reason> {
        unreachable!("a zone without binders keeps its proofs as they are")
    }

    /// The choices made while a rule may ask its first premise for
    /// another answer, by which that premise's search is resumed.
    fn choices(&mut self) -> &mut Choices {
        unreachable!("a zone without binders has one answer")
    }
}

/// A linear zone: a multiset of members.
pub(super) trait Linear<M>: Debug + Send + Sync {
    /// The empty zone of a forest.
    fn empty(forest: &Forest) -> Self;
    /// Adds one copy of `m`.
    fn insert(&mut self, m: M);
    /// Removes one copy of `m`, which must be a member.
    fn remove(&mut self, m: M);
    /// Whether `m` is a member.
    fn contains(&self, m: M) -> bool;
    /// Removes every member.
    fn clear(&mut self);
    /// Makes this zone a copy of `other`, in its own buffers.
    fn copy_from(&mut self, other: &Self);
    /// Whether the zone has no member.
    fn is_empty(&self) -> bool;
    /// The members in ascending order, with repeats.
    fn iter(&self) -> impl Iterator<Item = M> + '_;
    /// Appends the members in ascending order, with repeats.
    fn list_into(&self, list: &mut Vec<M>);
    /// The zone as the occurrences the memo keys, when the zone
    /// [memoizes](Zone::memoizes).
    fn ground(&self) -> &Context;
}

/// An unrestricted zone: a set of members.
pub(super) trait Unrestricted<M>: Debug + Send + Sync {
    /// The empty zone of a forest.
    fn empty(forest: &Forest) -> Self;
    /// Adds `m`.
    fn insert(&mut self, m: M);
    /// Whether `m` is a member.
    fn contains(&self, m: M) -> bool;
    /// Removes every member.
    fn clear(&mut self);
    /// Makes this zone a copy of `other`, in its own buffers.
    fn copy_from(&mut self, other: &Self);
    /// The members in ascending order.
    fn iter(&self) -> impl Iterator<Item = M> + '_;
    /// The zone as the occurrences the memo keys, when the zone
    /// [memoizes](Zone::memoizes).
    fn ground(&self) -> &OccSet;
}

impl Linear<OccId> for Context {
    #[inline]
    fn empty(forest: &Forest) -> Self {
        Self::empty(forest.len())
    }

    #[inline]
    fn insert(&mut self, m: OccId) {
        Self::insert(self, m);
    }

    #[inline]
    fn remove(&mut self, m: OccId) {
        Self::remove(self, m);
    }

    #[inline]
    fn contains(&self, m: OccId) -> bool {
        Self::contains(self, m)
    }

    #[inline]
    fn clear(&mut self) {
        Self::clear(self);
    }

    #[inline]
    fn copy_from(&mut self, other: &Self) {
        self.clone_from(other);
    }

    #[inline]
    fn is_empty(&self) -> bool {
        Self::is_empty(self)
    }

    #[inline]
    fn iter(&self) -> impl Iterator<Item = OccId> + '_ {
        Self::iter(self)
    }

    #[inline]
    fn list_into(&self, list: &mut Vec<OccId>) {
        Self::list_into(self, list);
    }

    #[inline]
    fn ground(&self) -> &Context {
        self
    }
}

impl Unrestricted<OccId> for OccSet {
    #[inline]
    fn empty(forest: &Forest) -> Self {
        forest.empty_set()
    }

    #[inline]
    fn insert(&mut self, m: OccId) {
        Self::insert(self, m);
    }

    #[inline]
    fn contains(&self, m: OccId) -> bool {
        Self::contains(self, m)
    }

    #[inline]
    fn clear(&mut self) {
        Self::clear(self);
    }

    #[inline]
    fn copy_from(&mut self, other: &Self) {
        self.clone_from(other);
    }

    #[inline]
    fn iter(&self) -> impl Iterator<Item = OccId> + '_ {
        Self::iter(self)
    }

    #[inline]
    fn ground(&self) -> &OccSet {
        self
    }
}

/// How far a chain of forced splits has read the forest's lists of the
/// occurrences of each literal in its search for duals.
pub(super) struct Cursors {
    /// Per list, twice the atom plus the sign as the forest numbers them:
    /// how many occurrences at its head are no longer in the chain's
    /// context.
    pub(super) passed: Vec<u32>,
    /// The lists whose entry is not zero.
    pub(super) moved: Vec<u32>,
}

/// What the members marked last have on one list of a literal's
/// occurrences.
#[derive(Clone, Copy, Debug)]
struct Marks {
    /// The stamp of the marking that last found a member on the list.
    stamp: u64,
    /// The first member that marking found there.
    first: OccId,
    /// The stamp of the marking whose initial rules last looked for the
    /// list's literal in `Θ`.
    tried: u64,
}

/// The zone whose members are the forest's occurrences: its literals meet
/// through the forest's lists of each literal's occurrences, and the memo
/// keys every sequent.
pub(super) struct Ground {
    /// Per list of a literal's occurrences, twice the atom plus the sign
    /// as the forest numbers them: what the members marked last have on
    /// it. Empty until the first marking.
    lists: Vec<Marks>,
    /// The stamp of the members marked last. A search that marks a
    /// billion sets a second takes five centuries to wrap it.
    stamp: u64,
}

impl Zone for Ground {
    type Member = OccId;
    type Linear = Context;
    type Unrestricted = OccSet;
    type Duals = usize;

    #[inline]
    fn new(_: &Forest, _: &Options) -> Self {
        Self {
            lists: Vec::new(),
            stamp: 0,
        }
    }

    /// A worker marks on lists of its own.
    #[cfg(feature = "parallel")]
    #[inline]
    fn fork(&self) -> Self {
        Self {
            lists: Vec::new(),
            stamp: 0,
        }
    }

    #[inline]
    fn root(&self, o: OccId) -> OccId {
        o
    }

    #[inline]
    fn occurrence(&self, m: OccId) -> OccId {
        m
    }

    #[inline]
    fn child(&self, _: OccId, child: OccId) -> OccId {
        child
    }

    /// Found through the forest's list of the dual literal's occurrences
    /// rather than through the zone's members. Always inlined: called on
    /// a set rather than a closure, it was left out of line once a second
    /// zone shared the callers, a call per lookup on a chain of copies.
    #[inline(always)]
    fn dual_in(&self, f: &Forest, literal: OccId, within: &OccSet) -> Option<OccId> {
        let (atom, sign) = (f.atom(literal)?, f.sign(literal)?);
        f.literals(atom, !sign)
            .iter()
            .copied()
            .find(|&d| within.contains(d))
    }

    /// What [`Self::dual_in`] finds in `rest`, read from where the chain's
    /// last lookup of that literal ended: the context of a chain only
    /// loses members, so the occurrences passed over once are in it no
    /// more. A chain's lookups of one literal together cost its list once
    /// and not once each, which on a marking of thousands of equal tokens
    /// was most of a search's time.
    #[inline]
    fn dual_from(
        &self,
        f: &Forest,
        literal: OccId,
        rest: &Context,
        cursors: &mut Cursors,
    ) -> Option<OccId> {
        let (atom, sign) = (f.atom(literal)?, f.sign(literal)?);
        let duals = f.literals(atom, !sign);
        let list = Forest::list(atom, !sign);
        let start = cursors.passed[list] as usize;
        let found = duals[start..].iter().position(|&d| rest.contains(d));
        let passed = found.map_or(duals.len(), |at| start + at);
        if passed != start {
            if start == 0 {
                cursors.moved.push(list as u32);
            }
            // A list is a part of the forest's occurrences, which a `u32`
            // indexes.
            cursors.passed[list] = passed as u32;
        }
        found.map(|at| duals[start + at])
    }

    /// One pass over the members, so that ranking the copies costs the
    /// members and the formulas of `Θ` once each and not their product,
    /// which on a marking of thousands of tokens under clauses of
    /// thousands of literals was a quarter of a second per stable
    /// sequent.
    #[inline]
    fn mark_literals(&mut self, f: &Forest, members: &[OccId], scratch: &mut Charged) {
        if self.lists.is_empty() {
            let lists = f.lists();
            scratch.charge(lists * size_of::<Marks>());
            self.lists = vec![
                Marks {
                    stamp: 0,
                    first: OccId::new(0),
                    tried: 0,
                };
                lists
            ];
        }
        self.stamp += 1;
        for &m in members {
            if let (Some(atom), Some(sign)) = (f.atom(m), f.sign(m)) {
                let list = &mut self.lists[Forest::list(atom, sign)];
                if list.stamp != self.stamp {
                    list.stamp = self.stamp;
                    list.first = m;
                }
            }
        }
    }

    /// The list of the dual literal's occurrences.
    #[inline]
    fn duals(&self, f: &Forest, literal: OccId) -> Option<usize> {
        let (atom, sign) = (f.atom(literal)?, f.sign(literal)?);
        Some(Forest::list(atom, !sign))
    }

    #[inline]
    fn marked(&self, dual: usize) -> Option<OccId> {
        let list = &self.lists[dual];
        (list.stamp == self.stamp).then_some(list.first)
    }

    #[inline]
    fn first_lookup(&mut self, dual: usize) -> bool {
        let list = &mut self.lists[dual];
        if list.tried == self.stamp {
            return false;
        }
        list.tried = self.stamp;
        true
    }

    /// The copy heuristic's notion of a copy that can meet something.
    #[inline]
    fn meets(&self, f: &Forest, a: OccId) -> bool {
        f.subtree(a).any(|l| match (f.atom(l), f.sign(l)) {
            (Some(atom), Some(sign)) => self.lists[Forest::list(atom, !sign)].stamp == self.stamp,
            _ => false,
        })
    }

    #[inline]
    fn class(&self, classes: &Classes, m: OccId) -> OccId {
        classes.of(m)
    }

    #[inline]
    fn same(&self, classes: &Classes, x: OccId, y: OccId) -> bool {
        classes.same(x, y)
    }

    #[inline]
    fn memoizes(&self, _: &OccSet, _: &Context) -> bool {
        true
    }

    /// The lists of members, which are occurrences.
    #[inline]
    fn word_lists(pools: &mut Pools<Self>) -> &mut Vec<Pooled<OccId>> {
        &mut pools.lists
    }

    #[inline]
    fn listed(words: &[OccId]) -> Option<&[OccId]> {
        Some(words)
    }
}
