// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The instance table of a proof: what its members past the forest's
//! length stand for. An occurrence under binders is a formula with loose
//! bound variables; a member of a sequent of the proof is such an
//! occurrence with a *frame*, the terms its binders were opened with, and
//! the table numbers each one. The terms a frame holds are the sequent's
//! closed terms or the proof's own, which extend the sequent's arena
//! past its end: the witnesses of `∃` steps and the eigenvariables of
//! `∀` steps.

use crate::Error;
use crate::hash::HashMap;
use crate::limits::Space;
use crate::occurrences::{Forest, Member, OccId};
use crate::proofs::Node;
use crate::sequents::fo::FirstOrder;
use crate::sequents::{Args, Eigen, FoTerm, FoTermId, Symbol};
use std::fmt::{Result as FmtResult, Write};

/// A frame of a proof's instance table: a term pushed onto a parent
/// frame, the innermost binder's term first. Frames are cons cells, so
/// the members of a tower of binders share their frames' tails.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FrameId(u32);

impl FrameId {
    /// Wraps a raw table index.
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    /// Returns the raw table index.
    pub const fn get(self) -> u32 {
        self.0
    }

    /// Returns the index as a `usize`, for indexing the table.
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

/// What the members of a proof past its forest's length stand for: each
/// an occurrence under `n` binders with a frame of `n` terms, the
/// innermost binder's first; the frames as cons cells; and the proof's
/// extension of its sequent's term arena, the witnesses and
/// eigenvariables the frames hold, numbered on from the sequent's own
/// terms and argument lists.
///
/// A table is built for one forest ([`new`](Self::new)) and handed to
/// [`Proof::with_instances`](crate::Proof::with_instances), which checks
/// it and makes it canonical: equal terms, frames and members one entry
/// each, so that the checker compares instances by their numbers.
///
/// # Examples
///
#[cfg_attr(feature = "parse", doc = "```")]
#[cfg_attr(not(feature = "parse"), doc = "```ignore")]
/// use linlog::{Eigen, Forest, Instances, Member, Mode, Node, NodeId, OccId, Proof, Sequent};
///
/// // ⊢ ∀x. (~p(x) ⅋ p(x)): occurrence 0 the binder, 1 the `⅋`, 2 and 3
/// // the literals.
/// let sequent: Sequent = "|- forall x. p(x) -o p(x)".parse()?;
/// let forest = Forest::new(&sequent)?;
/// let mut instances = Instances::new(&forest);
/// let a = instances.eigen(Eigen::new(0));
/// let frame = instances.frame(None, a);
/// let [par, l, r] = [1, 2, 3].map(|o| instances.member(OccId::new(o), frame));
/// let nodes = vec![
///     Node::Ax(l, r),
///     Node::Par(par, NodeId::new(0)),
///     Node::Forall(Member::new(0), Eigen::new(0), NodeId::new(1)),
/// ];
/// let proof = Proof::with_instances(forest, instances, nodes, NodeId::new(2))?;
/// proof.check(Mode::CLASSICAL)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Instances {
    /// The forest's length: the first member the table numbers.
    pub(crate) occurrences: u32,
    /// The terms of the sequent's arena: the first term of the extension.
    pub(crate) base_terms: u32,
    /// The argument lists of the sequent: the first list of the extension.
    pub(crate) base_lists: u32,
    /// Per member past the forest, its occurrence and frame.
    pub(crate) members: Vec<(OccId, FrameId)>,
    /// Per frame, its parent and the term it pushes.
    pub(crate) frames: Vec<(Option<FrameId>, FoTermId)>,
    /// The extension of the term arena: closed terms, each referring to
    /// terms before it.
    pub(crate) terms: Vec<FoTerm>,
    /// The arguments of every list of the extension, one after the other.
    pub(crate) args: Vec<FoTermId>,
    /// The extension's argument lists, each a start and a length into
    /// `args`.
    pub(crate) lists: Vec<(u32, u32)>,
}

impl Instances {
    /// Returns the empty table of a proof over `forest`.
    pub fn new(forest: &Forest) -> Self {
        let fo = &forest.sequent().fo;
        Self {
            // A forest has fewer than 2³² occurrences and a sequent's
            // arenas no more entries than its text or document.
            occurrences: forest.len() as u32,
            base_terms: fo.terms.len() as u32,
            base_lists: fo.lists.len() as u32,
            ..Self::default()
        }
    }

    /// Returns the term of the eigenvariable `a`, added to the extension.
    pub fn eigen(&mut self, a: Eigen) -> FoTermId {
        self.push_term(FoTerm::Eigen(a))
    }

    /// Returns the term `symbol(args…)`, added to the extension with its
    /// argument list; `args` are terms of the sequent or of the extension.
    pub fn apply(&mut self, symbol: Symbol, args: &[FoTermId]) -> FoTermId {
        // The ids saturate: a table past what a `u32` counts is refused
        // when a proof takes it (`canonical`), before any id is read.
        let start = saturated(self.args.len());
        self.args.extend_from_slice(args);
        self.lists.push((start, saturated(args.len())));
        let list = Args::new(
            self.base_lists
                .saturating_add(saturated(self.lists.len() - 1)),
        );
        self.push_term(FoTerm::App(symbol, list))
    }

    /// Returns the frame that pushes `term` onto `parent`, the frame of
    /// the binders further out, or onto none for an outermost binder.
    pub fn frame(&mut self, parent: Option<FrameId>, term: FoTermId) -> FrameId {
        self.frames.push((parent, term));
        FrameId::new(saturated(self.frames.len() - 1))
    }

    /// Returns the member that stands for `occurrence` under `frame`,
    /// which holds a term for every binder above the occurrence.
    pub fn member(&mut self, occurrence: OccId, frame: FrameId) -> Member {
        self.members.push((occurrence, frame));
        Member::new(
            self.occurrences
                .saturating_add(saturated(self.members.len() - 1)),
        )
    }

    /// Returns the number of members the table numbers.
    pub fn len(&self) -> usize {
        self.members.len()
    }

    /// Returns whether the table numbers no member.
    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }

    /// Returns the occurrence and frame `member` stands for: `None` for a
    /// member that is an occurrence of the forest, or that the table does
    /// not number.
    pub fn instance(&self, member: Member) -> Option<(OccId, FrameId)> {
        let at = member.get().checked_sub(self.occurrences)?;
        self.members.get(at as usize).copied()
    }

    /// Returns the parent frame and the term of `frame`.
    ///
    /// # Panics
    ///
    /// For a frame the table does not hold.
    pub fn frame_parts(&self, frame: FrameId) -> (Option<FrameId>, FoTermId) {
        self.frames[frame.index()]
    }

    /// Returns the terms of `frame`, the innermost binder's first.
    ///
    /// # Panics
    ///
    /// For a frame the table does not hold.
    pub fn frame_terms(&self, frame: FrameId) -> Vec<FoTermId> {
        let mut terms = Vec::new();
        let mut next = Some(frame);
        while let Some(f) = next {
            let (parent, term) = self.frames[f.index()];
            terms.push(term);
            next = parent;
        }
        terms
    }

    /// Returns the extension's terms, numbered on from the sequent's.
    pub fn terms(&self) -> &[FoTerm] {
        &self.terms
    }

    /// Returns the first term of the extension: the number of the
    /// sequent's own terms.
    pub fn first_term(&self) -> FoTermId {
        FoTermId::new(self.base_terms)
    }

    /// Returns the eigenvariables the table's terms name, ascending and
    /// each once.
    pub fn eigenvariables(&self) -> Vec<Eigen> {
        let mut eigens: Vec<Eigen> = self
            .terms
            .iter()
            .filter_map(|t| match *t {
                FoTerm::Eigen(a) => Some(a),
                FoTerm::Bound(_) | FoTerm::App(..) => None,
            })
            .collect();
        eigens.sort_unstable();
        eigens.dedup();
        eigens
    }

    /// Adds `term` to the extension and returns its id.
    pub(crate) fn push_term(&mut self, term: FoTerm) -> FoTermId {
        self.terms.push(term);
        FoTermId::new(
            self.base_terms
                .saturating_add(saturated(self.terms.len() - 1)),
        )
    }
}

/// Returns `n` as a `u32`, or `u32::MAX` where it is more.
fn saturated(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

/// The terms of a proof: its sequent's arena and the proof's extension
/// of it, one space of ids.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Terms<'a> {
    /// The sequent's tables.
    pub(crate) fo: &'a FirstOrder,
    /// The proof's table.
    pub(crate) table: &'a Instances,
}

impl Terms<'_> {
    /// Returns the number of terms of both arenas.
    pub(crate) fn len(&self) -> usize {
        self.fo.terms.len() + self.table.terms.len()
    }

    /// Returns the term `id`.
    pub(crate) fn term(&self, id: FoTermId) -> FoTerm {
        match id.index().checked_sub(self.fo.terms.len()) {
            None => self.fo.terms[id.index()],
            Some(at) => self.table.terms[at],
        }
    }

    /// Returns the terms of the argument list `args`.
    pub(crate) fn list(&self, args: Args) -> &[FoTermId] {
        match args.index().checked_sub(self.fo.lists.len()) {
            None => self.fo.list(args),
            Some(at) => {
                let (start, len) = self.table.lists[at];
                &self.table.args[start as usize..(start + len) as usize]
            }
        }
    }

    /// Returns the number of argument lists of both tables.
    fn lists(&self) -> usize {
        self.fo.lists.len() + self.table.lists.len()
    }
}

/// A term as hash-consing keys it: its tag with its children's numbers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Key {
    /// A bound variable.
    Bound(u32),
    /// A symbol applied to the list of that number.
    App(Symbol, u64),
    /// An eigenvariable.
    Eigen(Eigen),
}

/// Numbers first-order terms and argument lists so that equal ones have
/// one number: every term is numbered by its symbol and its arguments'
/// numbers, every list by its items'. A number is the count of distinct
/// terms or lists numbered before it, each held in a table: a `u64`, which
/// no memory holds as many entries as it counts.
#[derive(Default)]
pub(crate) struct Hashcons {
    /// Every term numbered, by its key.
    terms: HashMap<Key, u64>,
    /// Every list numbered, by its items.
    lists: HashMap<Box<[u64]>, u64>,
}

impl Hashcons {
    /// Returns the number of a term and whether it is new.
    fn term(&mut self, key: Key) -> (u64, bool) {
        let next = self.terms.len() as u64;
        match self.terms.entry(key) {
            std::collections::hash_map::Entry::Occupied(e) => (*e.get(), false),
            std::collections::hash_map::Entry::Vacant(e) => (*e.insert(next), true),
        }
    }

    /// Returns the number of a list of term numbers and whether it is new.
    fn list(&mut self, items: &[u64]) -> (u64, bool) {
        if let Some(&n) = self.lists.get(items) {
            return (n, false);
        }
        let n = self.lists.len() as u64;
        self.lists.insert(items.into(), n);
        (n, true)
    }
}

/// The numbers of a proof's terms and the means to number the instances
/// of its atoms: what the checker compares an axiom's two literals by.
pub(crate) struct Numbering<'a> {
    /// The proof's terms.
    terms: Terms<'a>,
    /// The hash-consing.
    cons: Hashcons,
    /// Per term of both arenas, its number.
    numbers: Vec<u64>,
    /// Per term, one more than its largest loose bound index: `0` for a
    /// closed term.
    loose: Vec<u32>,
    /// Per argument list, its number once computed.
    list_numbers: Vec<Option<u64>>,
}

impl<'a> Numbering<'a> {
    /// Numbers every term of both arenas, which are topological.
    pub(crate) fn new(terms: Terms<'a>) -> Self {
        let mut this = Self {
            terms,
            cons: Hashcons::default(),
            numbers: Vec::with_capacity(terms.len()),
            loose: Vec::with_capacity(terms.len()),
            list_numbers: vec![None; terms.lists()],
        };
        for id in 0..terms.len() {
            let id = FoTermId::new(id as u32);
            let (key, loose) = match terms.term(id) {
                FoTerm::Bound(i) => (Key::Bound(i), i.saturating_add(1)),
                FoTerm::Eigen(a) => (Key::Eigen(a), 0),
                FoTerm::App(f, args) => {
                    let loose = terms
                        .list(args)
                        .iter()
                        .map(|t| this.loose[t.index()])
                        .max()
                        .unwrap_or(0);
                    (Key::App(f, this.list_number(args)), loose)
                }
            };
            this.numbers.push(this.cons.term(key).0);
            this.loose.push(loose);
        }
        this
    }

    /// Returns the number of the argument list `args`, whose items are
    /// numbered.
    fn list_number(&mut self, args: Args) -> u64 {
        if let Some(n) = self.list_numbers[args.index()] {
            return n;
        }
        let items: Vec<u64> = self
            .terms
            .list(args)
            .iter()
            .map(|t| self.numbers[t.index()])
            .collect();
        let n = self.cons.list(&items).0;
        self.list_numbers[args.index()] = Some(n);
        n
    }

    /// Returns the terms numbered.
    pub(crate) fn terms(&self) -> Terms<'a> {
        self.terms
    }

    /// Returns the number of the term `id`.
    pub(crate) fn number(&self, id: FoTermId) -> u64 {
        self.numbers[id.index()]
    }

    /// Returns whether the term `id` is closed: no bound variable of it
    /// is loose.
    pub(crate) fn is_closed(&self, id: FoTermId) -> bool {
        self.loose[id.index()] == 0
    }

    /// Returns the number of the argument list `args` with every loose
    /// bound variable `i` replaced by the term numbered `frame[i]`: the
    /// arguments of an atom's instance. Each shared subterm is visited
    /// once, on an explicit stack.
    pub(crate) fn instance(&mut self, args: Args, frame: &[u64]) -> u64 {
        let mut memo: HashMap<FoTermId, u64> = HashMap::default();
        let terms = self.terms;
        let list = terms.list(args);
        let mut items = Vec::with_capacity(list.len());
        for &t in list {
            items.push(self.substitute(t, frame, &mut memo));
        }
        self.cons.list(&items).0
    }

    /// Returns the number of the term `t` under `frame`, as
    /// [`instance`](Self::instance) takes its arguments.
    fn substitute(&mut self, t: FoTermId, frame: &[u64], memo: &mut HashMap<FoTermId, u64>) -> u64 {
        if self.loose[t.index()] == 0 {
            return self.numbers[t.index()];
        }
        let mut stack = vec![(t, false)];
        while let Some((u, expanded)) = stack.pop() {
            if memo.contains_key(&u) {
                continue;
            }
            match self.terms.term(u) {
                // The sequent's check holds every bound index of an atom
                // below its occurrence's binders, and the frame has a term
                // for each.
                FoTerm::Bound(i) => {
                    memo.insert(u, frame[i as usize]);
                }
                FoTerm::App(f, args) if expanded => {
                    let items: Vec<u64> = self
                        .terms
                        .list(args)
                        .iter()
                        .map(|&c| {
                            if self.loose[c.index()] == 0 {
                                self.numbers[c.index()]
                            } else {
                                memo[&c]
                            }
                        })
                        .collect();
                    let list = self.cons.list(&items).0;
                    memo.insert(u, self.cons.term(Key::App(f, list)).0);
                }
                FoTerm::App(_, args) => {
                    stack.push((u, true));
                    for &c in self.terms.list(args) {
                        if self.loose[c.index()] > 0 && !memo.contains_key(&c) {
                            stack.push((c, false));
                        }
                    }
                }
                // An eigenvariable is closed, numbered above.
                FoTerm::Eigen(_) => unreachable!("an eigenvariable is a closed term"),
            }
        }
        memo[&t]
    }
}

/// Returns, per occurrence of `forest`, how many binders stand above it.
pub(crate) fn binders(forest: &Forest) -> Vec<u32> {
    let mut depth = vec![0u32; forest.len()];
    for o in forest.ids() {
        if let Some(p) = forest.parent(o) {
            // At most one per ancestor: below the forest's length.
            depth[o.index()] = depth[p.index()] + u32::from(forest.kind(p).is_binder());
        }
    }
    depth
}

/// Returns the error of a malformed instance table.
fn malformed(message: String) -> Error {
    Error::Json {
        form: "instance table",
        message,
    }
}

/// Checks `table` against `forest` and the proof's `nodes` (already
/// within their bounds), and returns it canonical, with the nodes'
/// members and witnesses renumbered into it: equal terms, lists, frames
/// and members one entry each, the sequent's own terms kept where they
/// are, and every member the checker derives from a member a node names
/// added (the subformulas of a named member under its frame, the body of
/// a `∀` or `∃` step under its frame with the eigenvariable or witness
/// pushed, the dual of a cut's member), so that every member a check or a
/// derivation of the proof speaks of has a number.
///
/// # Errors
///
/// [`Error::Json`] (form `instance table`) for a table made for another
/// forest, an index out of its range, an entry referring to a later one,
/// an extension term with a bound variable, a symbol applied with another
/// number of arguments or a predicate applied as a function, a frame
/// term with a loose bound variable, and a member whose frame has not
/// one term for each binder above its occurrence;
/// and a witness out of range or with a loose bound variable;
/// [`Error::IndexOutOfBounds`] for a member past the table, and
/// [`Refusal::Index`](crate::Refusal::Index) for a table whose ids would
/// pass what a `u32` counts.
pub(crate) fn canonical(
    forest: &Forest,
    table: &Instances,
    nodes: &mut [Node],
) -> Result<Instances, Error> {
    let fo = &forest.sequent().fo;
    let made_for = Instances::new(forest);
    if (table.occurrences, table.base_terms, table.base_lists)
        != (
            made_for.occurrences,
            made_for.base_terms,
            made_for.base_lists,
        )
    {
        return Err(malformed("the table was made for another sequent".into()));
    }
    // Every id the table made gives, with the members, frames and terms
    // the closure adds (three members, one frame and one term a node at
    // most), must be a `u32`: past that the table is refused.
    let count = nodes.len() as u64;
    let most = u64::from(u32::MAX);
    let ids = [
        (
            Space::Member,
            u64::from(table.occurrences) + table.members.len() as u64 + 3 * count,
        ),
        (
            Space::Term,
            u64::from(table.base_terms) + table.terms.len() as u64 + count,
        ),
        (
            Space::Term,
            u64::from(table.base_lists) + table.lists.len() as u64 + count,
        ),
        (Space::Term, table.args.len() as u64),
        (Space::Term, table.frames.len() as u64 + count),
    ];
    if let Some(&(what, count)) = ids.iter().find(|&&(_, count)| count > most) {
        return Err(Error::Refused(crate::limits::Refusal::Index {
            what,
            count,
            most,
        }));
    }
    let terms = Terms { fo, table };
    let all_terms = terms.len();
    let all_lists = terms.lists();
    let mut predicate = vec![false; fo.symbols.len()];
    for &(p, _) in &fo.atoms {
        predicate[p.index()] = true;
    }
    for (&(start, len), n) in table.lists.iter().zip(0..) {
        if (start as usize).saturating_add(len as usize) > table.args.len() {
            return Err(malformed(format!("list {n} runs past the arguments")));
        }
    }
    for (i, &term) in table.terms.iter().enumerate() {
        let id = fo.terms.len() + i;
        match term {
            FoTerm::Bound(_) => {
                return Err(malformed(format!("term {id} is a bound variable")));
            }
            FoTerm::Eigen(_) => {}
            FoTerm::App(f, args) => {
                let Some(&(_, arity)) = fo.symbols.get(f.index()) else {
                    return Err(malformed(format!("term {id} applies no symbol")));
                };
                if predicate[f.index()] {
                    return Err(malformed(format!("term {id} applies a predicate")));
                }
                if args.index() >= all_lists {
                    return Err(malformed(format!("term {id} has no argument list")));
                }
                let list = terms.list(args);
                if list.len() != arity as usize {
                    return Err(malformed(format!(
                        "term {id} applies a symbol of {arity} arguments to {}",
                        list.len()
                    )));
                }
                if list.iter().any(|t| t.index() >= id) {
                    return Err(malformed(format!("term {id} refers to a later term")));
                }
            }
        }
    }
    let numbering = Numbering::new(terms);
    // Every extension term is closed (no bound variable, and arguments
    // before it), so only the sequent's terms may be open.
    let mut lengths: Vec<u32> = Vec::with_capacity(table.frames.len());
    for (n, &(parent, term)) in table.frames.iter().enumerate() {
        if term.index() >= all_terms || !numbering.is_closed(term) {
            return Err(malformed(format!("frame {n} pushes no closed term")));
        }
        let length = match parent {
            None => 1,
            Some(p) if p.index() < n => lengths[p.index()] + 1,
            Some(_) => return Err(malformed(format!("frame {n} refers to a later frame"))),
        };
        lengths.push(length);
    }
    let depth = binders(forest);
    for (n, &(o, frame)) in table.members.iter().enumerate() {
        let member = u64::from(table.occurrences) + n as u64;
        if o.index() >= forest.len() || frame.index() >= table.frames.len() {
            return Err(malformed(format!("member {member} names no instance")));
        }
        if lengths[frame.index()] != depth[o.index()] {
            return Err(malformed(format!(
                "member {member} stands under {} binders with a frame of {} terms",
                depth[o.index()],
                lengths[frame.index()]
            )));
        }
    }
    for node in nodes.iter() {
        if let Node::Exists(_, w, _) = *node {
            if w.index() >= all_terms {
                return Err(malformed(format!(
                    "the witness {} is past the {all_terms} terms",
                    w.get()
                )));
            }
            if !numbering.is_closed(w) {
                return Err(malformed(format!(
                    "the witness {} holds a loose bound variable",
                    w.get()
                )));
            }
        }
    }
    Canonical::new(forest, table, numbering).finish(nodes)
}

/// The canonical table [`canonical`] builds.
struct Canonical<'a> {
    /// The forest.
    forest: &'a Forest,
    /// The table read.
    table: &'a Instances,
    /// The numbers of its terms.
    numbering: Numbering<'a>,
    /// The table made.
    out: Instances,
    /// Per term number, its id in the table made.
    term_at: HashMap<u64, FoTermId>,
    /// Per list number, its id in the table made.
    list_at: HashMap<u64, Args>,
    /// Per frame of the table made, by its parent and term.
    frame_at: HashMap<(Option<FrameId>, FoTermId), FrameId>,
    /// Per frame of the table read, its frame in the table made once
    /// made: a tower of frames is made once, not once per member.
    made: Vec<Option<FrameId>>,
    /// Per member of the table made, by its occurrence and frame.
    member_at: HashMap<(OccId, FrameId), Member>,
}

impl<'a> Canonical<'a> {
    /// The empty table, the sequent's terms and lists in place.
    fn new(forest: &'a Forest, table: &'a Instances, numbering: Numbering<'a>) -> Self {
        let fo = &forest.sequent().fo;
        let mut this = Self {
            forest,
            table,
            numbering,
            out: Instances::new(forest),
            term_at: HashMap::default(),
            list_at: HashMap::default(),
            frame_at: HashMap::default(),
            made: vec![None; table.frames.len()],
            member_at: HashMap::default(),
        };
        for id in 0..fo.terms.len() {
            let id = FoTermId::new(id as u32);
            this.term_at.entry(this.numbering.number(id)).or_insert(id);
        }
        for args in 0..fo.lists.len() {
            let args = Args::new(args as u32);
            let n = this.numbering.list_number(args);
            this.list_at.entry(n).or_insert(args);
        }
        this
    }

    /// Returns the id in the table made of the closed term `id` of the
    /// table read, adding it and its arguments where they are new.
    fn term(&mut self, id: FoTermId) -> FoTermId {
        let n = self.numbering.number(id);
        if let Some(&at) = self.term_at.get(&n) {
            return at;
        }
        // Not among the sequent's terms: an extension term, whose
        // arguments precede it.
        let term = match self.numbering.terms.term(id) {
            FoTerm::App(f, args) => {
                let ln = self.numbering.list_number(args);
                let list = match self.list_at.get(&ln) {
                    Some(&list) => list,
                    None => {
                        let items: Vec<FoTermId> = self.numbering.terms.list(args).to_vec();
                        let items: Vec<FoTermId> =
                            items.into_iter().map(|t| self.term(t)).collect();
                        let start = self.out.args.len() as u32;
                        self.out.args.extend_from_slice(&items);
                        self.out.lists.push((start, items.len() as u32));
                        let list = Args::new(self.out.base_lists + self.out.lists.len() as u32 - 1);
                        self.list_at.insert(ln, list);
                        list
                    }
                };
                FoTerm::App(f, list)
            }
            other @ (FoTerm::Eigen(_) | FoTerm::Bound(_)) => other,
        };
        let at = self.out.push_term(term);
        self.term_at.insert(n, at);
        at
    }

    /// Returns the id in the table made of the eigenvariable `a`'s term.
    fn eigen(&mut self, a: Eigen) -> FoTermId {
        let (n, _) = self.numbering.cons.term(Key::Eigen(a));
        if let Some(&at) = self.term_at.get(&n) {
            return at;
        }
        let at = self.out.push_term(FoTerm::Eigen(a));
        self.term_at.insert(n, at);
        at
    }

    /// Returns the frame of the table made that pushes `term`, an id of
    /// the table made, onto `parent`.
    fn push(&mut self, parent: Option<FrameId>, term: FoTermId) -> FrameId {
        *self.frame_at.entry((parent, term)).or_insert_with(|| {
            self.out.frames.push((parent, term));
            // Within what `canonical` checked first.
            FrameId::new(self.out.frames.len() as u32 - 1)
        })
    }

    /// Returns the frame of the table made for the frame `f` of the table
    /// read.
    fn frame(&mut self, f: FrameId) -> FrameId {
        // The frames up to the first made, then made downwards.
        let mut chain = Vec::new();
        let mut next = Some(f);
        let mut below = None;
        while let Some(f) = next {
            if let Some(made) = self.made[f.index()] {
                below = Some(made);
                break;
            }
            chain.push(f);
            next = self.table.frames[f.index()].0;
        }
        for f in chain.into_iter().rev() {
            let term = self.term(self.table.frames[f.index()].1);
            let made = self.push(below, term);
            self.made[f.index()] = Some(made);
            below = Some(made);
        }
        below.expect("a frame holds a term")
    }

    /// Returns the member of the table made for `o` under `frame`, a
    /// frame of the table made, or the occurrence itself without one.
    fn member(&mut self, o: OccId, frame: Option<FrameId>) -> Member {
        let Some(frame) = frame else {
            return Member::from(o);
        };
        *self.member_at.entry((o, frame)).or_insert_with(|| {
            self.out.members.push((o, frame));
            Member::new(self.out.occurrences + self.out.members.len() as u32 - 1)
        })
    }

    /// Returns the occurrence and frame of a member of the table made.
    fn resolve(&self, m: Member) -> (OccId, Option<FrameId>) {
        match self.out.instance(m) {
            Some((o, f)) => (o, Some(f)),
            None => (m.occ(), None),
        }
    }

    /// Renumbers the nodes' members and witnesses into the table made,
    /// adds the members the checker derives from them, and returns it.
    fn finish(mut self, nodes: &mut [Node]) -> Result<Instances, Error> {
        let read: Vec<Member> = (0..self.table.members.len())
            .map(|n| {
                let (o, f) = self.table.members[n];
                let f = self.frame(f);
                self.member(o, Some(f))
            })
            .collect();
        let occurrences = self.table.occurrences;
        let renumber = |m: Member| -> Result<Member, Error> {
            match m.get().checked_sub(occurrences) {
                None => Ok(m),
                Some(at) => read
                    .get(at as usize)
                    .copied()
                    .ok_or(Error::IndexOutOfBounds {
                        space: Space::Member,
                        index: m.index(),
                        len: occurrences as usize + read.len(),
                    }),
            }
        };
        for node in nodes.iter_mut() {
            *node = node.renamed(renumber)?;
            if let Node::Exists(m, w, p) = *node {
                *node = Node::Exists(m, self.term(w), p);
            }
        }
        let forest = self.forest;
        for node in nodes.iter() {
            for m in node.members() {
                let (o, frame) = self.resolve(m);
                if frame.is_some() && !forest.kind(o).is_binder() {
                    for c in forest.children(o) {
                        self.member(c, frame);
                    }
                }
            }
            match *node {
                Node::Forall(m, a, _) => {
                    let (o, frame) = self.resolve(m);
                    if forest.kind(o).is_binder() {
                        let term = self.eigen(a);
                        let frame = self.push(frame, term);
                        self.member(OccId::new(o.get() + 1), Some(frame));
                    }
                }
                Node::Exists(m, w, _) => {
                    let (o, frame) = self.resolve(m);
                    if forest.kind(o).is_binder() {
                        let frame = self.push(frame, w);
                        self.member(OccId::new(o.get() + 1), Some(frame));
                    }
                }
                Node::Cut(m, ..) => {
                    let (o, frame) = self.resolve(m);
                    if let Some(dual) = forest.dual(o) {
                        self.member(dual, frame);
                    }
                }
                Node::Ax(..)
                | Node::Tensor(..)
                | Node::Par(..)
                | Node::One(_)
                | Node::Bot(..)
                | Node::With(..)
                | Node::Plus(..)
                | Node::Top(_)
                | Node::Bang(..)
                | Node::Quest(..)
                | Node::Copy(..)
                | Node::Weaken(..)
                | Node::Mix(..) => {}
            }
        }
        Ok(self.out)
    }
}

/// Returns the names the eigenvariables `eigens` are written with: `a`,
/// `b`, … `z`, then `a_1`, `b_1`, …, skipping every name a symbol of the
/// sequent has, so that a sequent of a derivation reads as a sequent with
/// those constants.
pub(crate) fn eigen_names(fo: &FirstOrder, eigens: &[Eigen]) -> HashMap<Eigen, String> {
    let taken: std::collections::HashSet<&str> =
        fo.symbols.iter().map(|(s, _)| s.as_str()).collect();
    let mut names = HashMap::default();
    let mut candidates = (0u64..).map(|n| {
        let letter = char::from(b'a' + (n % 26) as u8);
        match n / 26 {
            0 => letter.to_string(),
            k => format!("{letter}_{k}"),
        }
    });
    for &a in eigens {
        let name = candidates
            .by_ref()
            .find(|c| !taken.contains(c.as_str()))
            .expect("the candidates never end");
        names.insert(a, name);
    }
    names
}

/// What a printer needs to write a proof's instances: the table, the
/// terms of both arenas and the eigenvariables' names.
#[derive(Clone, Debug)]
pub(crate) struct Framing<'a> {
    /// The proof's table.
    pub(crate) table: &'a Instances,
    /// The terms of both arenas.
    terms: Terms<'a>,
    /// The name of every eigenvariable the table holds.
    names: HashMap<Eigen, String>,
}

impl<'a> Framing<'a> {
    /// Returns the framing of `table`, a proof's over a sequent of the
    /// tables `fo`.
    pub(crate) fn new(fo: &'a FirstOrder, table: &'a Instances) -> Self {
        Self {
            table,
            terms: Terms { fo, table },
            names: eigen_names(fo, &table.eigenvariables()),
        }
    }

    /// Returns the occurrence `m` stands for.
    pub(crate) fn occurrence(&self, m: Member) -> OccId {
        self.table.instance(m).map_or(m.occ(), |(o, _)| o)
    }

    /// Returns the eigenvariables' names.
    pub(crate) fn names(&self) -> impl Iterator<Item = &str> {
        self.names.values().map(String::as_str)
    }

    /// Returns the name eigenvariable `a` is written with.
    pub(crate) fn name(&self, a: Eigen) -> &str {
        self.names.get(&a).map_or("?", String::as_str)
    }

    /// Writes the closed term `term` as `f(t, …)`, every symbol's and
    /// eigenvariable's name through `atom`, on an explicit stack.
    pub(crate) fn write_term<W: Write + ?Sized>(
        &self,
        out: &mut W,
        term: FoTermId,
        atom: &mut impl FnMut(&mut W, &str) -> FmtResult,
    ) -> FmtResult {
        let mut open: Vec<(Args, u32)> = Vec::new();
        let mut next = Some(term);
        loop {
            if let Some(t) = next.take() {
                match self.terms.term(t) {
                    FoTerm::App(f, args) => {
                        atom(out, &self.terms.fo.symbols[f.index()].0)?;
                        if !self.terms.list(args).is_empty() {
                            out.write_char('(')?;
                            open.push((args, 0));
                        }
                    }
                    FoTerm::Eigen(a) => atom(out, self.name(a))?,
                    // A frame and a witness hold closed terms.
                    FoTerm::Bound(index) => write!(out, "#{index}")?,
                }
            }
            let Some(&mut (args, ref mut i)) = open.last_mut() else {
                return Ok(());
            };
            let list = self.terms.list(args);
            if (*i as usize) < list.len() {
                if *i > 0 {
                    out.write_str(", ")?;
                }
                next = Some(list[*i as usize]);
                *i += 1;
            } else {
                out.write_char(')')?;
                open.pop();
            }
        }
    }
}
