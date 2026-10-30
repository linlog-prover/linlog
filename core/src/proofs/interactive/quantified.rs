// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The first-order part of a session: the instances its goals hold past
//! the forest's occurrences, the eigenvariables its `∀` steps made, and
//! the witnesses its `∃` steps left open as metavariables, with what the
//! axioms and the search bound them to.
//!
//! The table only grows: an undone step leaves its entries behind, unused,
//! and takes back the bindings it made, which the trail records with the
//! step's number. Equal instances, frames and closed terms are one entry
//! each, so that a member stands for one instance and a step done again
//! names the members it named before.

use crate::Instances;
use crate::hash::HashMap;
use crate::occurrences::instances::{FrameId, Terms};
use crate::occurrences::{Forest, Member, OccId};
use crate::sequents::{Eigen, FoTerm, FoTermId, Symbol};

/// A metavariable: the witness of an `∃` step left open.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Meta {
    /// Its term in the table.
    pub(super) term: FoTermId,
    /// How many eigenvariables were made before it, lowered by every
    /// binding of a metavariable it occurs in to that one's: a binding may
    /// name only eigenvariables numbered below, or constants. An
    /// eigenvariable made later belongs to a `∀` step above the `∃` step
    /// that made the metavariable, or on another branch, and in its
    /// witness would stand in that `∀` step's conclusion or outside its
    /// subtree.
    pub(super) level: u32,
    /// Its level when it was made, before any binding lowered it.
    pub(super) made: u32,
    /// What a later axiom or the search bound it to.
    pub(super) bound: Option<FoTermId>,
}

/// What a step did that its undo takes back.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Trailed {
    /// A metavariable was bound.
    Bound(u32),
    /// A metavariable's level was lowered from the one given.
    Lowered(u32, u32),
}

/// Why two terms do not unify.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Clash {
    /// The symbols differ, or a metavariable would occur in its own
    /// binding.
    Terms,
    /// A binding would name an eigenvariable made after the metavariable.
    Eigen(Eigen),
}

/// A closed term by what it is, for hash-consing.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum Key {
    /// A symbol applied to closed terms.
    App(Symbol, Box<[FoTermId]>),
    /// An eigenvariable.
    Eigen(Eigen),
}

/// The first-order part of a session.
#[derive(Clone, Debug)]
pub(super) struct Quantified {
    /// The instances as made: members, frames and terms, metavariables
    /// among the terms.
    pub(super) table: Instances,
    /// The same instances with every bound metavariable replaced by what
    /// it is bound to: what the views print and the proof is made of.
    /// Members and frames keep their numbers. Made again ([`show`](Self::show))
    /// after every change of the session, before anything reads it.
    pub(super) shown: Instances,
    /// Every member past the forest by its occurrence and frame.
    members: HashMap<(OccId, FrameId), Member>,
    /// Every frame by its parent and term.
    frames: HashMap<(Option<FrameId>, FoTermId), FrameId>,
    /// Every closed term without a metavariable by what it is, the
    /// sequent's own closed terms among them.
    terms: HashMap<Key, FoTermId>,
    /// The metavariables, by number.
    pub(super) metas: Vec<Meta>,
    /// How many eigenvariables were made: the next one's number. Each is
    /// a term of the table, as is each metavariable, so both together are
    /// fewer than the table's terms, which a `u32` numbers.
    pub(super) eigens: u32,
    /// The eigenvariables that are constants: made by the search for a
    /// witness nothing constrained, introduced by no `∀` step, and so
    /// free to stand in any binding.
    pub(super) constants: Vec<Eigen>,
    /// The bindings and lowerings made, in order, each with the number of
    /// the step that made it.
    trail: Vec<(u32, Trailed)>,
    /// Where the table, the metavariables, the eigenvariables and the
    /// constants stood before each step that made entries, with the step's
    /// number: what its undo truncates to. A session read from JSON has
    /// none for its steps, whose undo leaves their entries unused.
    marks: Vec<(u32, Mark)>,
    /// The most terms a term of the session may unfold to: `limits.terms`
    /// of the session's start.
    pub(super) limit: Option<u64>,
    /// The most terms a shown term unfolds to, or the table's entries
    /// (members, frames and terms) where they are more, as
    /// [`show`](Self::show) last counted: what the reader bounds too.
    pub(super) largest: u64,
}

/// Where a session's first-order part stood before a step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Mark {
    /// The members.
    members: usize,
    /// The frames.
    frames: usize,
    /// The terms past the sequent's.
    terms: usize,
    /// The arguments of the table's lists.
    args: usize,
    /// The table's lists.
    lists: usize,
    /// The metavariables.
    metas: usize,
    /// The eigenvariables made.
    eigens: u32,
    /// The constants.
    constants: usize,
    /// The trail.
    trail: usize,
}

impl Quantified {
    /// Returns the empty first-order part of a session over `forest`.
    pub(super) fn new(forest: &Forest) -> Self {
        let table = Instances::new(forest);
        let fo = &forest.sequent().fo;
        let loose = fo.loose();
        let mut terms = HashMap::default();
        for (n, term) in fo.terms.iter().enumerate() {
            if let FoTerm::App(f, args) = *term
                && loose[n] == 0
            {
                // The sequent's arena has fewer terms than a `u32` counts.
                let id = FoTermId::new(n as u32);
                terms.insert(Key::App(f, fo.list(args).into()), id);
            }
        }
        Self {
            shown: table.clone(),
            table,
            members: HashMap::default(),
            frames: HashMap::default(),
            terms,
            metas: Vec::new(),
            eigens: 0,
            constants: Vec::new(),
            trail: Vec::new(),
            marks: Vec::new(),
            limit: Some(crate::Limits::DEFAULT_TERMS),
            largest: 0,
        }
    }

    /// Returns where everything stands now, for a step to go back to.
    pub(super) fn mark(&self) -> Mark {
        Mark {
            members: self.table.members.len(),
            frames: self.table.frames.len(),
            terms: self.table.terms.len(),
            args: self.table.args.len(),
            lists: self.table.lists.len(),
            metas: self.metas.len(),
            eigens: self.eigens,
            constants: self.constants.len(),
            trail: self.trail.len(),
        }
    }

    /// Goes back to `mark`: what the trail holds past it taken back, and
    /// every entry, metavariable, eigenvariable and constant made since
    /// dropped, none of which a member, frame or term before it names.
    pub(super) fn back_to(&mut self, mark: Mark) {
        self.undo_to(mark.trail);
        let (occurrences, base) = (self.table.occurrences, self.table.base_terms);
        self.table.members.truncate(mark.members);
        self.table.frames.truncate(mark.frames);
        self.table.terms.truncate(mark.terms);
        self.table.args.truncate(mark.args);
        self.table.lists.truncate(mark.lists);
        self.metas.truncate(mark.metas);
        self.eigens = mark.eigens;
        self.constants.truncate(mark.constants);
        // The ids below the mark fit a `u32`, as the table made them.
        self.members
            .retain(|_, m| (m.get() - occurrences) < mark.members as u32);
        self.frames.retain(|_, f| f.get() < mark.frames as u32);
        self.terms
            .retain(|_, t| t.get() < base || t.get() - base < mark.terms as u32);
    }

    /// Records that the step numbered `step` made entries from `mark` on,
    /// for its undo.
    pub(super) fn made(&mut self, step: u32, mark: Mark) {
        self.marks.push((step, mark));
    }

    /// Returns the occurrence and frame a member stands for: no frame for
    /// an occurrence of the forest.
    pub(super) fn instance(&self, m: Member) -> (OccId, Option<FrameId>) {
        self.table
            .instance(m)
            .map_or((m.occ(), None), |(o, f)| (o, Some(f)))
    }

    /// Returns the member of occurrence `o` under `frame`: the occurrence
    /// itself without one.
    pub(super) fn member(&mut self, o: OccId, frame: Option<FrameId>) -> Member {
        let Some(frame) = frame else {
            return Member::from(o);
        };
        if let Some(&m) = self.members.get(&(o, frame)) {
            return m;
        }
        let m = self.table.member(o, frame);
        self.members.insert((o, frame), m);
        m
    }

    /// Returns the member of occurrence `o` under `frame` where the table
    /// has it: the occurrence itself without a frame.
    pub(super) fn find(&self, o: OccId, frame: Option<FrameId>) -> Option<Member> {
        match frame {
            None => Some(Member::from(o)),
            Some(frame) => self.members.get(&(o, frame)).copied(),
        }
    }

    /// Returns the shown table with every open witness a fresh constant:
    /// the metavariable `k` the eigenvariable numbered `eigens + k`, which
    /// no `∀` step introduces. Refused past the numbers a `u32` holds.
    pub(super) fn closed_table(&self) -> Result<Instances, crate::Error> {
        let mut table = self.shown.clone();
        for term in &mut table.terms {
            if let FoTerm::Meta(k) = *term {
                let Some(n) = self.eigens.checked_add(k) else {
                    return Err(crate::Error::Refused(crate::Refusal::Index {
                        what: crate::limits::Space::Term,
                        count: u64::from(self.eigens) + u64::from(k),
                        most: u64::from(u32::MAX),
                    }));
                };
                *term = FoTerm::Eigen(Eigen::new(n));
            }
        }
        Ok(table)
    }

    /// Returns the frame that pushes `term` onto `parent`.
    pub(super) fn frame(&mut self, parent: Option<FrameId>, term: FoTermId) -> FrameId {
        if let Some(&f) = self.frames.get(&(parent, term)) {
            return f;
        }
        let f = self.table.frame(parent, term);
        self.frames.insert((parent, term), f);
        f
    }

    /// Returns the closed term `key`, made once.
    fn closed(&mut self, key: Key) -> FoTermId {
        if let Some(&t) = self.terms.get(&key) {
            return t;
        }
        let t = match &key {
            Key::App(f, args) => self.table.apply(*f, args),
            &Key::Eigen(a) => self.table.eigen(a),
        };
        self.terms.insert(key, t);
        t
    }

    /// Returns the term of symbol `f` applied to `args`, closed terms.
    pub(super) fn apply(&mut self, f: Symbol, args: &[FoTermId]) -> FoTermId {
        self.closed(Key::App(f, args.into()))
    }

    /// Returns the term of the eigenvariable `a`.
    pub(super) fn eigen_term(&mut self, a: Eigen) -> FoTermId {
        self.closed(Key::Eigen(a))
    }

    /// Makes a fresh eigenvariable and returns it with its term, or `None`
    /// past the numbers a `u32` holds.
    pub(super) fn fresh_eigen(&mut self) -> Option<(Eigen, FoTermId)> {
        let a = Eigen::new(self.eigens);
        self.eigens = self.eigens.checked_add(1)?;
        Some((a, self.eigen_term(a)))
    }

    /// Makes a fresh metavariable at the level of the eigenvariables made
    /// so far and returns its term.
    pub(super) fn fresh_meta(&mut self) -> FoTermId {
        // Fewer metavariables than the table's terms, which a `u32` numbers.
        let k = self.metas.len() as u32;
        let term = self.table.push_term(FoTerm::Meta(k));
        self.metas.push(Meta {
            term,
            level: self.eigens,
            made: self.eigens,
            bound: None,
        });
        term
    }

    /// Returns the terms of both arenas.
    pub(super) fn terms<'a>(&'a self, forest: &'a Forest) -> Terms<'a> {
        Terms {
            fo: &forest.sequent().fo,
            table: &self.table,
        }
    }

    /// Returns `t` with the bindings of the metavariables at its top
    /// followed.
    pub(super) fn resolve(&self, forest: &Forest, mut t: FoTermId) -> FoTermId {
        let terms = self.terms(forest);
        while let FoTerm::Meta(k) = terms.term(t)
            && let Some(b) = self.metas[k as usize].bound
        {
            t = b;
        }
        t
    }

    /// Returns the frame's terms, the innermost binder's first; none for no
    /// frame.
    pub(super) fn frame_terms(&self, frame: Option<FrameId>) -> Vec<FoTermId> {
        frame.map_or_else(Vec::new, |f| self.table.frame_terms(f))
    }

    /// Returns the term of the sequent's arena `t`, whose loose bound
    /// variables the frame's terms (innermost first) close, as a closed
    /// term of both arenas, made where it is not one already.
    pub(super) fn close(&mut self, forest: &Forest, t: FoTermId, frame: &[FoTermId]) -> FoTermId {
        let fo = &forest.sequent().fo;
        // A walk with a stack of its own, children before their term.
        let mut done: HashMap<FoTermId, FoTermId> = HashMap::default();
        let mut stack = vec![(t, false)];
        while let Some((u, expanded)) = stack.pop() {
            if done.contains_key(&u) {
                continue;
            }
            match fo.terms[u.index()] {
                FoTerm::Bound(i) => {
                    // The sequent's check keeps every bound index below
                    // its occurrence's binders, and a frame holds a term
                    // for each.
                    done.insert(u, frame[i as usize]);
                }
                FoTerm::App(f, args) if expanded => {
                    let items: Vec<FoTermId> = fo.list(args).iter().map(|c| done[c]).collect();
                    let closed = self.apply(f, &items);
                    done.insert(u, closed);
                }
                FoTerm::App(_, args) => {
                    stack.push((u, true));
                    stack.extend(fo.list(args).iter().map(|&c| (c, false)));
                }
                FoTerm::Eigen(_) | FoTerm::Meta(_) => {
                    unreachable!("a sequent's arena holds no eigenvariable or metavariable")
                }
            }
        }
        done[&t]
    }

    /// Returns the predicate and the closed arguments of the literal a
    /// member stands for; `None` for a nullary atom of a sequent whose
    /// atoms have no tables, whose identity is its atom.
    pub(super) fn literal(
        &mut self,
        forest: &Forest,
        m: Member,
    ) -> Option<(Symbol, Vec<FoTermId>)> {
        let (o, frame) = self.instance(m);
        let atom = forest.atom(o).expect("a literal has an atom");
        let fo = &forest.sequent().fo;
        let &(symbol, args) = fo.atoms.get(atom.index())?;
        let frame = self.frame_terms(frame);
        let args: Vec<FoTermId> = fo.list(args).to_vec();
        Some((
            symbol,
            args.into_iter()
                .map(|t| self.close(forest, t, &frame))
                .collect(),
        ))
    }

    /// Unifies the pairs of closed terms, binding metavariables with the
    /// occurs check and the levels; the bindings are trailed under `step`.
    /// On a clash nothing is left bound.
    pub(super) fn unify(
        &mut self,
        forest: &Forest,
        pairs: Vec<(FoTermId, FoTermId)>,
        step: u32,
    ) -> Result<(), Clash> {
        let mark = self.trail.len();
        let result = self.unify_pairs(forest, pairs, step);
        if result.is_err() {
            self.undo_to(mark);
        }
        result
    }

    /// Unifies the pairs, leaving what it bound before a clash.
    fn unify_pairs(
        &mut self,
        forest: &Forest,
        mut pairs: Vec<(FoTermId, FoTermId)>,
        step: u32,
    ) -> Result<(), Clash> {
        // A pair met before is unified already: shared terms are walked
        // once, not once per path to them.
        let mut met: std::collections::HashSet<(FoTermId, FoTermId)> = Default::default();
        while let Some((x, y)) = pairs.pop() {
            let (x, y) = (self.resolve(forest, x), self.resolve(forest, y));
            if x == y || !met.insert((x, y)) {
                continue;
            }
            let terms = self.terms(forest);
            match (terms.term(x), terms.term(y)) {
                (FoTerm::Meta(k), _) => self.bind(forest, k, y, step)?,
                (_, FoTerm::Meta(k)) => self.bind(forest, k, x, step)?,
                (FoTerm::App(f, a), FoTerm::App(g, b)) if f == g => {
                    pairs.extend(
                        terms
                            .list(a)
                            .iter()
                            .copied()
                            .zip(terms.list(b).iter().copied()),
                    );
                }
                // Distinct eigenvariables, an eigenvariable against an
                // application, or two symbols.
                _ => return Err(Clash::Terms),
            }
        }
        Ok(())
    }

    /// Binds the unbound metavariable `k` to `t`, refusing a `t` that holds
    /// `k` or an eigenvariable at or above `k`'s level that is no
    /// constant, and lowering the levels of the metavariables `t` holds to
    /// `k`'s.
    pub(super) fn bind(
        &mut self,
        forest: &Forest,
        k: u32,
        t: FoTermId,
        step: u32,
    ) -> Result<(), Clash> {
        let level = self.metas[k as usize].level;
        let mut lowered = Vec::new();
        {
            let terms = self.terms(forest);
            let mut seen: std::collections::HashSet<FoTermId> = Default::default();
            let mut stack = vec![t];
            while let Some(u) = stack.pop() {
                let u = self.resolve(forest, u);
                if !seen.insert(u) {
                    continue;
                }
                match terms.term(u) {
                    FoTerm::Meta(j) if j == k => return Err(Clash::Terms),
                    FoTerm::Meta(j) => {
                        if self.metas[j as usize].level > level {
                            lowered.push(j);
                        }
                    }
                    FoTerm::Eigen(a) => {
                        if a.get() >= level && !self.constants.contains(&a) {
                            return Err(Clash::Eigen(a));
                        }
                    }
                    FoTerm::App(_, args) => stack.extend_from_slice(terms.list(args)),
                    FoTerm::Bound(_) => unreachable!("a closed term has no bound variable"),
                }
            }
        }
        for j in lowered {
            let meta = &mut self.metas[j as usize];
            self.trail.push((step, Trailed::Lowered(j, meta.level)));
            meta.level = level;
        }
        self.metas[k as usize].bound = Some(t);
        self.trail.push((step, Trailed::Bound(k)));
        Ok(())
    }

    /// Takes back what the steps numbered `step` and later bound and
    /// lowered, and drops what they made where their marks say.
    pub(super) fn undo(&mut self, step: u32) {
        let keep = self.trail.partition_point(|&(s, _)| s < step);
        self.undo_to(keep);
        while let Some(&(s, mark)) = self.marks.last()
            && s >= step
        {
            self.marks.pop();
            self.back_to(mark);
        }
    }

    /// Takes back the trail past its first `len` entries, last first.
    fn undo_to(&mut self, len: usize) {
        while self.trail.len() > len {
            let (_, trailed) = self.trail.pop().expect("longer than len");
            match trailed {
                Trailed::Bound(k) => self.metas[k as usize].bound = None,
                Trailed::Lowered(k, level) => self.metas[k as usize].level = level,
            }
        }
    }

    /// Returns the bindings in force in the order made, each the
    /// metavariable, its term and the step that made it.
    #[cfg(feature = "serialize")]
    pub(super) fn bindings(&self) -> impl Iterator<Item = (u32, FoTermId, u32)> + '_ {
        self.trail
            .iter()
            .filter_map(|&(step, trailed)| match trailed {
                Trailed::Bound(k) => Some((
                    k,
                    self.metas[k as usize]
                        .bound
                        .expect("a trailed binding holds"),
                    step,
                )),
                Trailed::Lowered(..) => None,
            })
    }

    /// Rebuilds the first-order part of a session over `forest` from its
    /// written form: the table's members, frames and terms, the level
    /// each metavariable was made at, the eigenvariables made and the
    /// constants among them; and replays the bindings in the order made.
    /// Refuses a table a session could not have made: an entry naming one
    /// after it or out of range, a frame of another length than its
    /// occurrence's binders, an entry made twice, a term of a predicate or
    /// of the wrong arity, a binding the occurs check or the levels
    /// refuse, a step out of order.
    #[cfg(feature = "serialize")]
    pub(super) fn from_parts(
        forest: &Forest,
        parts: Parts,
        steps: usize,
        limit: Option<u64>,
    ) -> Result<Self, crate::Error> {
        let bad = |reason: &'static str| crate::Error::InconsistentSession { reason };
        let mut q = Self::new(forest);
        q.eigens = parts.eigens;
        q.limit = limit;
        let fo = &forest.sequent().fo;
        let loose = fo.loose();
        let predicates: Vec<Symbol> = fo.atoms.iter().map(|&(p, _)| p).collect();
        let base = q.table.base_terms as usize;
        let closed = |t: u32, before: usize| {
            (t as usize) < before && (t as usize >= base || loose.get(t as usize) == Some(&0))
        };
        for (n, term) in parts.terms.iter().enumerate() {
            let id = base + n;
            match *term {
                PartTerm::App(f, ref args) => {
                    let Some(&(_, arity)) = fo.symbols.get(f as usize) else {
                        return Err(bad("a term names no symbol of the sequent"));
                    };
                    let f = Symbol::new(f);
                    if predicates.contains(&f) || arity as usize != args.len() {
                        return Err(bad(
                            "a term applies a predicate, or a symbol to the wrong number of arguments",
                        ));
                    }
                    if !args.iter().all(|&a| closed(a, id)) {
                        return Err(bad("a term's argument is not a closed term before it"));
                    }
                    let args: Vec<FoTermId> = args.iter().map(|&a| FoTermId::new(a)).collect();
                    if q.terms.contains_key(&Key::App(f, args.clone().into())) {
                        return Err(bad("a term repeats another"));
                    }
                    let t = q.table.apply(f, &args);
                    q.terms.insert(Key::App(f, args.into()), t);
                }
                PartTerm::Eigen(a) => {
                    let a = Eigen::new(a);
                    if a.get() >= q.eigens || q.terms.contains_key(&Key::Eigen(a)) {
                        return Err(bad("an eigenvariable repeats or is past those made"));
                    }
                    let t = q.table.eigen(a);
                    q.terms.insert(Key::Eigen(a), t);
                }
                PartTerm::Meta(k) => {
                    let Some(&made) = parts.metas.get(q.metas.len()) else {
                        return Err(bad("a metavariable has no level"));
                    };
                    if k as usize != q.metas.len() || made > q.eigens {
                        return Err(bad(
                            "the metavariables are not numbered in order, or a level is past the eigenvariables made",
                        ));
                    }
                    let term = q.table.push_term(FoTerm::Meta(k));
                    q.metas.push(Meta {
                        term,
                        level: made,
                        made,
                        bound: None,
                    });
                }
            }
        }
        if q.metas.len() != parts.metas.len() {
            return Err(bad("a level has no metavariable"));
        }
        let all = base + parts.terms.len();
        let mut lengths: Vec<u32> = Vec::with_capacity(parts.frames.len());
        for (n, &(parent, t)) in parts.frames.iter().enumerate() {
            if parent.is_some_and(|p| p as usize >= n) || !closed(t, all) {
                return Err(bad("a frame names a frame after it or a term out of range"));
            }
            let parent = parent.map(FrameId::new);
            let t = FoTermId::new(t);
            if q.frames.contains_key(&(parent, t)) {
                return Err(bad("a frame repeats another"));
            }
            let f = q.table.frame(parent, t);
            q.frames.insert((parent, t), f);
            lengths.push(parent.map_or(1, |p| lengths[p.index()] + 1));
        }
        let depth = crate::occurrences::instances::binders(forest);
        for &(o, f) in &parts.members {
            let fits = depth
                .get(o as usize)
                .zip(lengths.get(f as usize))
                .is_some_and(|(d, l)| d == l);
            if !fits {
                return Err(bad(
                    "a member's frame is not as long as its occurrence's binders",
                ));
            }
            let (o, f) = (OccId::new(o), FrameId::new(f));
            if q.members.contains_key(&(o, f)) {
                return Err(bad("a member repeats another"));
            }
            let m = q.table.member(o, f);
            q.members.insert((o, f), m);
        }
        for &a in &parts.constants {
            if a >= q.eigens {
                return Err(bad("a constant is past the eigenvariables made"));
            }
            q.constants.push(Eigen::new(a));
        }
        let mut last = 0;
        for &(k, t, step) in &parts.bindings {
            // A step binds a metavariable to a term whose top is no bound
            // metavariable: unification follows the bindings first.
            let ok = step >= last
                && (step as usize) < steps
                && q.metas.get(k as usize).is_some_and(|m| m.bound.is_none())
                && closed(t, all)
                && q.resolve(forest, FoTermId::new(t)) == FoTermId::new(t);
            if !ok || q.bind(forest, k, FoTermId::new(t), step).is_err() {
                return Err(bad(
                    "a binding is out of order, of no open metavariable, or one the occurs check or the levels refuse",
                ));
            }
            last = step;
        }
        q.show(forest);
        if let Some(limit) = q.limit
            && q.too_large()
        {
            return Err(crate::Error::Refused(crate::Refusal::Terms {
                terms: q.largest,
                limit,
            }));
        }
        Ok(q)
    }

    /// Returns the written form of the first-order part.
    #[cfg(feature = "serialize")]
    pub(super) fn parts(&self, forest: &Forest) -> Parts {
        let terms = self.terms(forest);
        Parts {
            members: self
                .table
                .members
                .iter()
                .map(|&(o, f)| (o.get(), f.get()))
                .collect(),
            frames: self
                .table
                .frames
                .iter()
                .map(|&(p, t)| (p.map(FrameId::get), t.get()))
                .collect(),
            terms: self
                .table
                .terms
                .iter()
                .map(|&t| match t {
                    FoTerm::App(f, args) => {
                        PartTerm::App(f.get(), terms.list(args).iter().map(|a| a.get()).collect())
                    }
                    FoTerm::Eigen(a) => PartTerm::Eigen(a.get()),
                    FoTerm::Meta(k) => PartTerm::Meta(k),
                    FoTerm::Bound(_) => unreachable!("a table's term is closed"),
                })
                .collect(),
            metas: self.metas.iter().map(|m| m.made).collect(),
            eigens: self.eigens,
            constants: self.constants.iter().map(|a| a.get()).collect(),
            bindings: self
                .bindings()
                .map(|(k, t, step)| (k, t.get(), step))
                .collect(),
        }
    }

    /// Returns whether two literals stand for the same atom under the
    /// bindings, as instances: one predicate, equal arguments; read in the
    /// shown table, where the bindings are followed already, building
    /// nothing, binding nothing.
    pub(super) fn same_atom(&self, forest: &Forest, x: Member, y: Member) -> bool {
        let fo = &forest.sequent().fo;
        let shown = |m: Member| {
            self.shown
                .instance(m)
                .map_or((m.occ(), None), |(o, f)| (o, Some(f)))
        };
        let ((ox, fx), (oy, fy)) = (shown(x), shown(y));
        let (ax, ay) = (forest.atom(ox), forest.atom(oy));
        let (Some(&(p, a)), Some(&(r, b))) = (
            ax.and_then(|a| fo.atoms.get(a.index())),
            ay.and_then(|a| fo.atoms.get(a.index())),
        ) else {
            return ax == ay;
        };
        if p != r {
            return false;
        }
        let frame = |f: Option<FrameId>| f.map_or_else(Vec::new, |f| self.shown.frame_terms(f));
        let (fx, fy) = (frame(fx), frame(fy));
        let loose = fo.loose();
        let terms = Terms {
            fo,
            table: &self.shown,
        };
        // A term is the sequent's under its side's frame, or closed.
        let side = |t: FoTermId, frame: &[FoTermId], open: bool| match fo.terms.get(t.index()) {
            Some(&FoTerm::Bound(i)) if open => (frame[i as usize], false),
            _ if open && loose[t.index()] == 0 => (t, false),
            _ => (t, open),
        };
        let term = |u: FoTermId, open: bool| {
            if open {
                fo.terms[u.index()]
            } else {
                terms.term(u)
            }
        };
        let list = |args, open: bool| {
            if open {
                fo.list(args)
            } else {
                terms.list(args)
            }
        };
        let mut pairs: Vec<((FoTermId, bool), (FoTermId, bool))> = fo
            .list(a)
            .iter()
            .zip(fo.list(b))
            .map(|(&s, &t)| ((s, true), (t, true)))
            .collect();
        // A pair met before is equal already: shared terms are compared
        // once, not once per path to them.
        let mut met: std::collections::HashSet<((FoTermId, bool), (FoTermId, bool))> =
            Default::default();
        while let Some(((s, so), (t, to))) = pairs.pop() {
            let (s, t) = (side(s, &fx, so), side(t, &fy, to));
            if (!s.1 && !t.1 && s.0 == t.0) || !met.insert((s, t)) {
                continue;
            }
            match (term(s.0, s.1), term(t.0, t.1)) {
                (FoTerm::App(f, a), FoTerm::App(g, b)) if f == g => pairs.extend(
                    list(a, s.1)
                        .iter()
                        .zip(list(b, t.1))
                        .map(|(&u, &v)| ((u, s.1), (v, t.1))),
                ),
                _ => return false,
            }
        }
        true
    }

    /// Returns the eigenvariables and the unbound metavariables a closed
    /// term holds under the bindings.
    pub(super) fn variables(&self, forest: &Forest, t: FoTermId) -> (Vec<Eigen>, Vec<u32>) {
        let terms = self.terms(forest);
        let (mut eigens, mut metas) = (Vec::new(), Vec::new());
        let mut seen: std::collections::HashSet<FoTermId> = Default::default();
        let mut stack = vec![t];
        while let Some(u) = stack.pop() {
            let u = self.resolve(forest, u);
            if !seen.insert(u) {
                continue;
            }
            match terms.term(u) {
                FoTerm::Eigen(a) => eigens.push(a),
                FoTerm::Meta(k) => metas.push(k),
                FoTerm::App(_, args) => stack.extend_from_slice(terms.list(args)),
                FoTerm::Bound(_) => unreachable!("a closed term has no bound variable"),
            }
        }
        (eigens, metas)
    }

    /// Makes the shown table again from the table and the bindings in
    /// force: every extension term with its bound metavariables replaced,
    /// the frames over those terms, the members as they are.
    pub(super) fn show(&mut self, forest: &Forest) {
        let base = self.table.base_terms;
        let mut shown = Instances::new(forest);
        let mut at: HashMap<FoTermId, FoTermId> = HashMap::default();
        let terms = self.terms(forest);
        for n in 0..self.table.terms.len() {
            // The table's ids are below `u32::MAX`, as `Instances` makes
            // them.
            let t = FoTermId::new(base + n as u32);
            // A walk with a stack of its own: a binding names terms made
            // after the metavariable.
            let mut stack = vec![(t, false)];
            while let Some((u, expanded)) = stack.pop() {
                if u.get() < base || at.contains_key(&u) {
                    continue;
                }
                let made = match terms.term(u) {
                    FoTerm::Meta(k) => match self.metas[k as usize].bound {
                        Some(b) if expanded => self.mapped(b, &at),
                        Some(b) => {
                            stack.push((u, true));
                            stack.push((b, false));
                            continue;
                        }
                        None => shown.push_term(FoTerm::Meta(k)),
                    },
                    FoTerm::App(f, args) if expanded => {
                        let items: Vec<FoTermId> = terms
                            .list(args)
                            .iter()
                            .map(|&c| self.mapped(c, &at))
                            .collect();
                        shown.apply(f, &items)
                    }
                    FoTerm::App(_, args) => {
                        stack.push((u, true));
                        stack.extend(terms.list(args).iter().map(|&c| (c, false)));
                        continue;
                    }
                    FoTerm::Eigen(a) => shown.eigen(a),
                    FoTerm::Bound(_) => unreachable!("a table's term is closed"),
                };
                at.insert(u, made);
            }
        }
        shown.frames = self
            .table
            .frames
            .iter()
            .map(|&(parent, t)| (parent, self.mapped(t, &at)))
            .collect();
        shown.members.clone_from(&self.table.members);
        // How many terms each shown term unfolds to, arguments first.
        let unfolded = forest.sequent().fo.unfolded();
        let mut sizes: Vec<u64> = Vec::with_capacity(shown.terms.len());
        let size = |sizes: &[u64], t: FoTermId| {
            if t.get() < base {
                unfolded[t.index()]
            } else {
                sizes[(t.get() - base) as usize]
            }
        };
        for &term in &shown.terms {
            let n = match term {
                FoTerm::App(_, args) => {
                    let start = (args.get() - shown.base_lists) as usize;
                    let (from, len) = shown.lists[start];
                    shown.args[from as usize..(from + len) as usize]
                        .iter()
                        .fold(1u64, |sum, &a| sum.saturating_add(size(&sizes, a)))
                }
                FoTerm::Eigen(_) | FoTerm::Meta(_) | FoTerm::Bound(_) => 1,
            };
            sizes.push(n);
        }
        let entries = (shown.members.len() as u64)
            .saturating_add(shown.frames.len() as u64)
            .saturating_add(shown.terms.len() as u64);
        self.largest = sizes.iter().copied().max().unwrap_or(0).max(entries);
        self.shown = shown;
    }

    /// Returns whether a shown term unfolds past the session's limit.
    pub(super) fn too_large(&self) -> bool {
        self.limit.is_some_and(|limit| self.largest > limit)
    }

    /// Returns the shown term of a term of either arena: the sequent's own
    /// as it is, the table's by `at`.
    fn mapped(&self, t: FoTermId, at: &HashMap<FoTermId, FoTermId>) -> FoTermId {
        if t.get() < self.table.base_terms {
            t
        } else {
            at[&t]
        }
    }
}

/// A term of a session's table as its written form holds it.
#[cfg(feature = "serialize")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PartTerm {
    /// A symbol applied to closed terms, by their ids.
    App(u32, Vec<u32>),
    /// An eigenvariable by its number.
    Eigen(u32),
    /// A metavariable by its number.
    Meta(u32),
}

/// The written form of a session's first-order part.
#[cfg(feature = "serialize")]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Parts {
    /// The members past the forest: occurrence and frame.
    pub(crate) members: Vec<(u32, u32)>,
    /// The frames: parent and term.
    pub(crate) frames: Vec<(Option<u32>, u32)>,
    /// The terms past the sequent's.
    pub(crate) terms: Vec<PartTerm>,
    /// The level each metavariable was made at.
    pub(crate) metas: Vec<u32>,
    /// How many eigenvariables were made.
    pub(crate) eigens: u32,
    /// The eigenvariables that are constants.
    pub(crate) constants: Vec<u32>,
    /// The bindings in the order made: metavariable, term, step.
    pub(crate) bindings: Vec<(u32, u32, u32)>,
}
