// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! A goal of a first-order session handed to the search: written as a
//! sequent of its own, proved, and its derivation carried back onto the
//! session's instances.
//!
//! The goal's members are instances whose frames hold the session's
//! eigenvariables and open witnesses. The sequent the search gets is one
//! formula: the goal's formulas joined by `⅋` (the hypotheses first, so
//! that the intuitionistic reading takes each `⅋` as an implication and
//! the goal last), its open witnesses bound by `∃` and the eigenvariables
//! made after the first of them by `∀`, in the order they were made, so
//! that the search's eigenvariable condition is the session's levels; the
//! older eigenvariables and the constants are constants of their own. A
//! proof of it opens that prefix and splits that spine before anything
//! else, and those steps are left out when the derivation comes back: the
//! prefix's witnesses bind the open witnesses, its eigenvariables are the
//! session's, every other eigenvariable of the proof is a fresh one of
//! the session's (a constant where no `∀` step introduces it).

use super::{Closed, GoalId, Interactive};
use crate::Error;
use crate::hash::HashMap;
use crate::limits::{Limits, Progress};
use crate::occurrences::instances::{Terms, binders, eigen_names};
use crate::occurrences::{Forest, Member, OccId};
use crate::proofs::derivation::{Compact, Derivation, InfId, Inference, ViewOptions};
use crate::proofs::{CheckError, Named, Node, Proof, Rule};
use crate::search::{self, Options, Verdict};
use crate::sequents::fo::{FirstOrder, Interner};
use crate::sequents::{Atom, Eigen, FoTerm, FoTermId, Sequent, Symbol, Term, TermId};

/// What a variable of the goal is in the sequent written for the search.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Bound {
    /// The `∃` or `∀` of the prefix at this position, outermost first.
    Prefix(u32),
    /// A constant, by its symbol.
    Constant(Symbol),
}

/// The goal written as a sequent for the search, with what its
/// occurrences stand for in the session.
struct Written {
    /// The sequent.
    sequent: Sequent,
    /// The prefix: per binder outermost first, the open witness it
    /// quantifies existentially or the eigenvariable it quantifies
    /// universally.
    prefix: Vec<Result<u32, Eigen>>,
    /// The constant symbols that stand for eigenvariables.
    constants: HashMap<Symbol, Eigen>,
}

/// Where an occurrence of the written sequent stands in the session.
#[derive(Clone, Copy, Debug)]
enum Place {
    /// A binder of the prefix, above every formula of the goal.
    Prefix,
    /// The `⅋` joining the goal's formulas from this one on.
    Spine(usize),
    /// An occurrence inside the goal's formula of this index, at this
    /// offset from its root.
    Inside(usize, u32),
}

impl Interactive {
    /// Closes a goal of a first-order session by the search, as
    /// [`close`](Self::close) does: the outcome is the search's on the goal
    /// written as a sequent of its own, and a proof is carried back and
    /// grafted as one step, which binds the open witnesses it filled.
    pub(super) fn close_framed(
        &mut self,
        goal: GoalId,
        options: &Options,
        view: &ViewOptions,
        limits: &Limits,
        mut stop: impl FnMut(Progress) -> bool,
    ) -> Result<Closed, Error> {
        let members = self.goal(goal)?.to_vec();
        let written = self.written(&members);
        let outcome =
            search::prove_within(&written.sequent, self.mode, options, limits, &mut stop)?;
        let mut grafted = Ok(());
        if let Verdict::Proved(proof) = &outcome.verdict {
            grafted = match self.carry_back(goal, &members, &written, proof, view, limits, stop) {
                Ok(()) => Ok(()),
                Err(Error::Refused(refusal)) => Err(refusal),
                Err(Error::Check(CheckError::Refused(refused))) => Err(refused.refusal),
                Err(error) => return Err(error),
            };
        }
        Ok(Closed { outcome, grafted })
    }

    /// Writes a goal of a first-order session as a sequent of its own.
    fn written(&self, members: &[Member]) -> Written {
        let q = self.fo.as_ref().expect("a first-order session");
        let forest = &self.forest;
        let fo = &forest.sequent().fo;
        // The goal's formulas, the hypotheses first in an intuitionistic
        // mode, and the variables their frames hold under the bindings.
        let mut order: Vec<Member> = members.to_vec();
        if let Some(reading) = self.reading() {
            order.sort_by_key(|&m| reading.position(self.occurrence(m)) == crate::Side::Output);
        }
        let (mut eigens, mut metas) = (Vec::new(), Vec::new());
        for &m in &order {
            for t in q.frame_terms(q.instance(m).1) {
                let (e, k) = q.variables(forest, t);
                eigens.extend(e);
                metas.extend(k);
            }
        }
        eigens.sort_unstable();
        eigens.dedup();
        metas.sort_unstable_by_key(|&k| (q.metas[k as usize].level, k));
        metas.dedup();
        // The prefix in the order the variables were made: an
        // eigenvariable at or past the first open witness's level is
        // bound by `∀` right before the first open witness of a higher
        // level; one below it, and a constant, is a constant.
        let first = metas.first().map(|&k| q.metas[k as usize].level);
        let mut prefix: Vec<Result<u32, Eigen>> = Vec::new();
        let mut placed: Vec<Eigen> = Vec::new();
        for &k in &metas {
            let level = q.metas[k as usize].level;
            for &a in &eigens {
                if a.get() < level
                    && first.is_some_and(|f| a.get() >= f)
                    && !q.constants.contains(&a)
                    && !placed.contains(&a)
                {
                    prefix.push(Err(a));
                    placed.push(a);
                }
            }
            prefix.push(Ok(k));
        }
        for &a in &eigens {
            if first.is_some_and(|f| a.get() >= f)
                && !q.constants.contains(&a)
                && !placed.contains(&a)
            {
                prefix.push(Err(a));
                placed.push(a);
            }
        }
        // The symbols: the sequent's, in their order and roles, then one
        // constant per eigenvariable that is no binder of the prefix.
        let predicates: Vec<Symbol> = fo.atoms.iter().map(|&(p, _)| p).collect();
        let mut interner = Interner::new();
        for (i, (name, arity)) in fo.symbols.iter().enumerate() {
            let predicate = predicates.contains(&Symbol::new(i as u32));
            interner
                .symbol(name, *arity, predicate)
                .expect("the sequent's own symbols fit together");
        }
        let names = eigen_names(fo, &q.shown.eigenvariables());
        let mut bound: HashMap<Eigen, Bound> = HashMap::default();
        let mut constants: HashMap<Symbol, Eigen> = HashMap::default();
        for (j, p) in prefix.iter().enumerate() {
            if let &Err(a) = p {
                bound.insert(a, Bound::Prefix(j as u32));
            }
        }
        for &a in &eigens {
            if let std::collections::hash_map::Entry::Vacant(entry) = bound.entry(a) {
                let symbol = interner
                    .symbol(&names[&a], 0, false)
                    .expect("an eigenvariable's name is no symbol's");
                entry.insert(Bound::Constant(symbol));
                constants.insert(symbol, a);
            }
        }
        let metas_at: HashMap<u32, u32> = prefix
            .iter()
            .enumerate()
            .filter_map(|(j, p)| p.ok().map(|k| (k, j as u32)))
            .collect();
        let depth = prefix.len() as u32;
        let mut copy = Copy {
            forest,
            q,
            interner,
            terms: Vec::new(),
            atoms: Vec::new(),
            atom_of: HashMap::default(),
            fo_atoms: Vec::new(),
            binders: Vec::new(),
            bound,
            metas_at,
            depth,
            closed: HashMap::default(),
        };
        let mut formulas: Vec<TermId> = order.iter().map(|&m| copy.formula(m)).collect();
        // The spine: the hypotheses' `⅋` the goal's formula last.
        let mut spine = formulas.pop().expect("a goal has a formula");
        while let Some(f) = formulas.pop() {
            spine = copy.push(Term::Par(f, spine));
        }
        let mut root = spine;
        for (j, p) in prefix.iter().enumerate().rev() {
            root = copy.push(match p {
                Ok(_) => Term::Exists(root),
                Err(_) => Term::Forall(root),
            });
            let name = match p {
                Ok(k) => format!("w_{k}"),
                Err(a) => names[a].clone(),
            };
            let _ = j;
            copy.binders.push((root, name));
        }
        copy.binders.sort_unstable_by_key(|&(t, _)| t);
        let mut tables: FirstOrder = copy.interner.tables;
        tables.atoms = copy.fo_atoms;
        tables.binders = copy.binders;
        let sequent = Sequent {
            terms: copy.terms,
            roots: vec![root],
            atoms: copy.atoms,
            antecedents: Some(0),
            cuts: vec![],
            fo: tables,
        };
        #[cfg(any(test, feature = "parse", feature = "serialize"))]
        debug_assert!(sequent.check().is_ok(), "{:?}", sequent.check());
        Written {
            sequent,
            prefix,
            constants,
        }
    }

    /// Carries a proof of the written goal back onto the session: its
    /// derivation, the prefix's and the spine's steps left out, every
    /// member an instance of the session's, grafted onto the goal as one
    /// step, and the open witnesses the prefix's `∃` steps filled bound.
    #[expect(clippy::too_many_arguments, reason = "close's own, and what it wrote")]
    fn carry_back(
        &mut self,
        goal: GoalId,
        members: &[Member],
        written: &Written,
        proof: &Proof,
        view: &ViewOptions,
        limits: &Limits,
        mut stop: impl FnMut(Progress) -> bool,
    ) -> Result<(), Error> {
        let bad = || Error::GoalMismatch;
        let found = proof.forest();
        let table = proof.instances().ok_or_else(bad)?;
        let terms = Terms {
            fo: &found.sequent().fo,
            table,
        };
        // The goal's formulas in the order written.
        let mut order: Vec<Member> = members.to_vec();
        if let Some(reading) = self.reading() {
            order.sort_by_key(|&m| reading.position(self.occurrence(m)) == crate::Side::Output);
        }
        let places = places(found, written.prefix.len(), order.len());
        let depth = binders(found);
        // The proof's eigenvariables: the prefix's are the session's, the
        // others fresh ones, constants where no `∀` step introduces them.
        let mut eigen_at: HashMap<Eigen, Eigen> = HashMap::default();
        let mut witnesses: Vec<(u32, FoTermId)> = Vec::new();
        let mut introduced: Vec<Eigen> = Vec::new();
        for node in proof.nodes() {
            if let Node::Forall(m, a, _) = *node {
                introduced.push(a);
                if let Place::Prefix = places[proof_occ(proof, m).index()] {
                    let j = depth[proof_occ(proof, m).index()] as usize;
                    if let Err(e) = written.prefix[j] {
                        eigen_at.insert(a, e);
                    }
                }
            } else if let Node::Exists(m, w, _) = *node
                && let Place::Prefix = places[proof_occ(proof, m).index()]
                && let Ok(k) = written.prefix[depth[proof_occ(proof, m).index()] as usize]
            {
                witnesses.push((k, w));
            }
        }
        let view = view.with_compact(Compact::Never);
        let derivation = Derivation::new(proof, &view, limits, &mut stop)?;
        // The reading by the fields, beside the first-order part borrowed
        // for the carrying.
        let reading = self.reading.as_ref().map(|(position, goal)| {
            crate::Reading::of_parts(&self.forest, position, *goal, self.mode.is_ordered())
        });
        let q = self.fo.as_mut().expect("a first-order session");
        let forest = &self.forest;
        let mut back = Back {
            q,
            terms,
            constants: &written.constants,
            eigen_at,
            introduced,
            done: HashMap::default(),
        };
        // Each of the proof's members as the session's: one instance, or
        // for the prefix and the spine the goal's formulas they join.
        let mut carried: Vec<Inference> = Vec::new();
        let mut at: Vec<Option<InfId>> = Vec::with_capacity(derivation.inferences().len());
        let mut members_of: HashMap<Member, Vec<Member>> = HashMap::default();
        for inference in derivation.inferences() {
            let mut map = |m: Member| -> Result<Vec<Member>, Error> {
                if let Some(ms) = members_of.get(&m) {
                    return Ok(ms.clone());
                }
                let o = proof_occ(proof, m);
                let ms = match places[o.index()] {
                    Place::Prefix => order.clone(),
                    Place::Spine(i) => order[i..].to_vec(),
                    Place::Inside(i, k) => {
                        let (base, frame) = back.q.instance(order[i]);
                        // A formula's root stands right under the prefix.
                        let inner = depth[o.index()] as usize - written.prefix.len();
                        let mut frame = frame;
                        if let Some((_, f)) = table.instance(m) {
                            let ts = table.frame_terms(f);
                            for &t in ts[..inner].iter().rev() {
                                let t = back.term(forest, t)?;
                                frame = Some(back.q.frame(frame, t));
                            }
                        }
                        vec![back.q.member(OccId::new(base.get() + k), frame)]
                    }
                };
                members_of.insert(m, ms.clone());
                Ok(ms)
            };
            let principal = inference.principal.map(|p| inference.sequent[p]);
            // The prefix's and the spine's steps are left out: the
            // premise's carried inference stands for theirs.
            if let Some(p) = principal
                && !matches!(places[proof_occ(proof, p).index()], Place::Inside(..))
            {
                let rule = inference.rule.rule;
                if !matches!(rule, Rule::Forall | Rule::Exists | Rule::Par) {
                    return Err(bad());
                }
                at.push(at[inference.premises[0].index()]);
                continue;
            }
            let mut sequent: Vec<Member> = Vec::new();
            for &m in &inference.sequent {
                sequent.extend(map(m)?);
            }
            sequent.sort_unstable();
            let principal = match principal {
                Some(p) => {
                    let [m] = map(p)?[..] else {
                        return Err(bad());
                    };
                    Some(sequent.binary_search(&m).map_err(|_| bad())?)
                }
                None => None,
            };
            let rule = match principal {
                Some(p) => Named::applied(
                    inference.rule.rule,
                    reading.as_ref(),
                    back.q.instance(sequent[p]).0,
                ),
                None => inference.rule,
            };
            let premises = inference
                .premises
                .iter()
                .map(|p| at[p.index()].ok_or_else(bad))
                .collect::<Result<Vec<_>, _>>()?;
            at.push(Some(InfId::new(carried.len() as u32)));
            carried.push(Inference {
                sequent,
                rule,
                principal,
                premises,
                times: 1,
                cut: None,
                node: None,
            });
        }
        let mut goal_members = members.to_vec();
        goal_members.sort_unstable();
        if carried.last().map(|root| &root.sequent) != Some(&goal_members) {
            return Err(bad());
        }
        // The prefix's witnesses, as the session's terms.
        let mut bindings = Vec::new();
        for (k, w) in witnesses {
            bindings.push((k, back.term(forest, w)?));
        }
        // Fewer steps than inferences, which an `InfId` numbers.
        let step = self.history.len() as u32;
        let q = self.fo.as_mut().expect("a first-order session");
        for (k, t) in bindings {
            if q.bind(forest, k, t, step).is_err() {
                q.undo(step);
                return Err(bad());
            }
        }
        self.graft(goal.inf(), carried);
        self.history.push(goal.inf());
        self.show();
        Ok(())
    }
}

/// Returns the occurrence a member of a proof stands for.
fn proof_occ(proof: &Proof, m: Member) -> OccId {
    proof
        .instances()
        .and_then(|t| t.instance(m))
        .map_or(m.occ(), |(o, _)| o)
}

/// Returns, per occurrence of the written sequent, where it stands: the
/// prefix's binders, the spine's `⅋`, and each formula's occurrences by
/// their offset.
fn places(forest: &Forest, prefix: usize, formulas: usize) -> Vec<Place> {
    let mut places = vec![Place::Prefix; forest.len()];
    let mut at = OccId::new(prefix as u32);
    for i in 0..formulas {
        let start = if i + 1 < formulas {
            places[at.index()] = Place::Spine(i);
            let left = forest.left(at).expect("a ⅋ of the spine");
            let next = forest.right(at).expect("a ⅋ of the spine");
            let start = left;
            at = next;
            start
        } else {
            at
        };
        let size = forest.size(start);
        for k in 0..size {
            places[(start.get() + k) as usize] = Place::Inside(i, k);
        }
    }
    places
}

/// The copy of a goal's formulas into the written sequent's arenas.
struct Copy<'a> {
    /// The session's forest.
    forest: &'a Forest,
    /// The session's first-order part.
    q: &'a super::quantified::Quantified,
    /// The written sequent's first-order terms.
    interner: Interner,
    /// The written sequent's formula terms.
    terms: Vec<Term>,
    /// The written sequent's atom names.
    atoms: Vec<String>,
    /// Every atom by its name.
    atom_of: HashMap<String, Atom>,
    /// Per atom, its predicate and arguments.
    fo_atoms: Vec<(Symbol, crate::sequents::Args)>,
    /// The binders' names.
    binders: Vec<(TermId, String)>,
    /// What each eigenvariable of the goal is.
    bound: HashMap<Eigen, Bound>,
    /// Each open witness's position in the prefix.
    metas_at: HashMap<u32, u32>,
    /// How many binders the prefix has.
    depth: u32,
    /// Every closed session term written under so many inner binders.
    closed: HashMap<(FoTermId, u32), FoTermId>,
}

impl Copy<'_> {
    /// Appends a formula term.
    fn push(&mut self, term: Term) -> TermId {
        self.terms.push(term);
        TermId::new(self.terms.len() as u32 - 1)
    }

    /// Copies the formula a member stands for, its loose variables its
    /// frame's terms.
    fn formula(&mut self, m: Member) -> TermId {
        let (o, frame) = self.q.instance(m);
        let frame = self.q.frame_terms(frame);
        let sequent = self.forest.sequent();
        let fo = &sequent.fo;
        // A walk with a stack of its own: a formula term with how many of
        // its own binders stand above it, and whether its children are
        // copied.
        let mut stack = vec![(self.forest.term(o), 0u32, false)];
        let mut done: Vec<TermId> = Vec::new();
        while let Some((t, inner, expanded)) = stack.pop() {
            let term = sequent.term(t);
            let unary = |t: TermId| [Some(t), None];
            let children: [Option<TermId>; 2] = match term {
                Term::Tensor(a, b) | Term::Par(a, b) | Term::With(a, b) | Term::Plus(a, b) => {
                    [Some(a), Some(b)]
                }
                Term::Bang(a) | Term::Quest(a) | Term::Forall(a) | Term::Exists(a) => unary(a),
                Term::Atom(_)
                | Term::DualAtom(_)
                | Term::One
                | Term::Bot
                | Term::Top
                | Term::Zero => [None; 2],
            };
            if !expanded && children[0].is_some() {
                stack.push((t, inner, true));
                let below = inner + u32::from(matches!(term, Term::Forall(_) | Term::Exists(_)));
                for c in children.into_iter().flatten().rev() {
                    stack.push((c, below, false));
                }
                continue;
            }
            let made = match term {
                Term::Atom(a) | Term::DualAtom(a) => {
                    let atom = self.atom(a, inner, &frame);
                    self.push(if matches!(term, Term::Atom(_)) {
                        Term::Atom(atom)
                    } else {
                        Term::DualAtom(atom)
                    })
                }
                Term::One | Term::Bot | Term::Top | Term::Zero => self.push(term),
                Term::Tensor(..) | Term::Par(..) | Term::With(..) | Term::Plus(..) => {
                    let b = done.pop().expect("the right child");
                    let a = done.pop().expect("the left child");
                    let mut copies = [a, b].into_iter();
                    self.push(term.map_subterms(|_| copies.next().expect("two children")))
                }
                Term::Bang(_) | Term::Quest(_) | Term::Forall(_) | Term::Exists(_) => {
                    let a = done.pop().expect("the child");
                    let made = self.push(term.map_subterms(|_| a));
                    if let Some(name) = fo.binder_name(t) {
                        self.binders.push((made, name.to_owned()));
                    }
                    made
                }
            };
            done.push(made);
        }
        done.pop().expect("the formula")
    }

    /// Returns the written sequent's atom for the session's atom `a` under
    /// `inner` binders of its formula and the member's frame.
    fn atom(&mut self, a: Atom, inner: u32, frame: &[FoTermId]) -> Atom {
        let fo = &self.forest.sequent().fo;
        let key = match fo.atoms.get(a.index()) {
            Some(&(p, args)) => {
                let items: Vec<FoTermId> = fo
                    .list(args)
                    .to_vec()
                    .into_iter()
                    .map(|t| self.open(t, inner, frame))
                    .collect();
                let list = self.interner.list(&items);
                Some((p, list))
            }
            None => None,
        };
        let name = match key {
            Some((p, list)) => self.interner.tables.atom_key(p, list),
            None => self.forest.sequent().atom_name(a).to_owned(),
        };
        if let Some(&atom) = self.atom_of.get(&name) {
            return atom;
        }
        let atom = Atom::new(self.atoms.len() as u32);
        self.atoms.push(name.clone());
        self.atom_of.insert(name, atom);
        if let Some(key) = key {
            self.fo_atoms.push(key);
        }
        atom
    }

    /// Copies a term of the sequent's arena under `inner` binders of its
    /// formula, its loose variables past them the frame's terms.
    fn open(&mut self, t: FoTermId, inner: u32, frame: &[FoTermId]) -> FoTermId {
        let fo = &self.forest.sequent().fo;
        match fo.terms[t.index()] {
            FoTerm::Bound(i) if i < inner => self.interner.term(FoTerm::Bound(i)),
            FoTerm::Bound(i) => self.closed(frame[(i - inner) as usize], inner),
            FoTerm::App(f, args) => {
                let items: Vec<FoTermId> = fo
                    .list(args)
                    .to_vec()
                    .into_iter()
                    .map(|c| self.open(c, inner, frame))
                    .collect();
                let list = self.interner.list(&items);
                self.interner.term(FoTerm::App(f, list))
            }
            FoTerm::Eigen(_) | FoTerm::Meta(_) => {
                unreachable!("a sequent's arena holds no eigenvariable or metavariable")
            }
        }
    }

    /// Copies a closed term of the session under `inner` binders of its
    /// formula, below the prefix: an eigenvariable or an open witness the
    /// prefix binds is its bound variable, any other eigenvariable its
    /// constant.
    fn closed(&mut self, t: FoTermId, inner: u32) -> FoTermId {
        let t = self.q.resolve(self.forest, t);
        if let Some(&made) = self.closed.get(&(t, inner)) {
            return made;
        }
        let made = match self.q.terms(self.forest).term(t) {
            FoTerm::App(f, args) => {
                let items: Vec<FoTermId> = self
                    .q
                    .terms(self.forest)
                    .list(args)
                    .to_vec()
                    .into_iter()
                    .map(|c| self.closed(c, inner))
                    .collect();
                let list = self.interner.list(&items);
                self.interner.term(FoTerm::App(f, list))
            }
            FoTerm::Eigen(a) => match self.bound[&a] {
                Bound::Constant(symbol) => {
                    let list = self.interner.list(&[]);
                    self.interner.term(FoTerm::App(symbol, list))
                }
                Bound::Prefix(j) => self.prefixed(j, inner),
            },
            FoTerm::Meta(k) => {
                let j = self.metas_at[&k];
                self.prefixed(j, inner)
            }
            FoTerm::Bound(_) => unreachable!("a closed term has no bound variable"),
        };
        self.closed.insert((t, inner), made);
        made
    }

    /// Returns the bound variable of the prefix's binder `j`, seen from
    /// under `inner` binders of a formula.
    fn prefixed(&mut self, j: u32, inner: u32) -> FoTermId {
        self.interner
            .term(FoTerm::Bound(self.depth - 1 - j + inner))
    }
}

/// The carrying back of a proof's terms into the session's.
struct Back<'a> {
    /// The session's first-order part.
    q: &'a mut super::quantified::Quantified,
    /// The proof's terms.
    terms: Terms<'a>,
    /// The written sequent's constants that stand for eigenvariables.
    constants: &'a HashMap<Symbol, Eigen>,
    /// The proof's eigenvariables as the session's.
    eigen_at: HashMap<Eigen, Eigen>,
    /// The proof's eigenvariables that a `∀` step introduces.
    introduced: Vec<Eigen>,
    /// Every term carried back.
    done: HashMap<FoTermId, FoTermId>,
}

impl Back<'_> {
    /// Returns the proof's closed term `t` as the session's.
    fn term(&mut self, forest: &Forest, t: FoTermId) -> Result<FoTermId, Error> {
        // A walk with a stack of its own, arguments first.
        let mut stack = vec![(t, false)];
        while let Some((u, expanded)) = stack.pop() {
            if self.done.contains_key(&u) {
                continue;
            }
            let made = match self.terms.term(u) {
                FoTerm::App(f, _) if self.constants.contains_key(&f) => {
                    self.q.eigen_term(self.constants[&f])
                }
                FoTerm::App(f, args) if expanded => {
                    let items: Vec<FoTermId> =
                        self.terms.list(args).iter().map(|c| self.done[c]).collect();
                    self.q.apply(f, &items)
                }
                FoTerm::App(_, args) => {
                    stack.push((u, true));
                    stack.extend(self.terms.list(args).iter().map(|&c| (c, false)));
                    continue;
                }
                FoTerm::Eigen(a) => {
                    let e = match self.eigen_at.get(&a) {
                        Some(&e) => e,
                        None => {
                            let (e, _) = self.q.fresh_eigen().ok_or(Error::GoalMismatch)?;
                            if !self.introduced.contains(&a) {
                                self.q.constants.push(e);
                            }
                            self.eigen_at.insert(a, e);
                            e
                        }
                    };
                    self.q.eigen_term(e)
                }
                FoTerm::Bound(_) | FoTerm::Meta(_) => return Err(Error::GoalMismatch),
            };
            self.done.insert(u, made);
        }
        let _ = forest;
        Ok(self.done[&t])
    }
}
