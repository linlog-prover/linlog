// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The zone of a first-order search: its members are occurrences under
//! binders with the terms their binders were opened with, a fresh
//! eigenvariable for a `∀` and a metavariable for the witness of an `∃`,
//! and its identities join literals whose terms unify.
//!
//! **Members.** An occurrence outside every binder is its own member. An
//! instance of a binder's body is a *span* of member ids, one per
//! occurrence of the body's subtree, with the body's frame: the binder's
//! own frame and the new term. So a subformula's member is the member's
//! id moved by as much as the occurrence is, and no table is looked up
//! on the engine's paths but where an occurrence is read, which finds
//! the span by its first id. A binder below another is never reached
//! through its parent's span: the rules open it with a span of its own.
//!
//! **Terms** are shared, never built: a term is a term of the sequent's
//! arena read under a frame, whose bound variables are the frame's terms,
//! or a variable of the search, a metavariable that a binding may point
//! at another term, or an eigenvariable. Unification binds metavariables
//! to such terms with the occurs check, iteratively, each pair of terms
//! once.
//!
//! **Levels** keep the eigenvariable condition. Eigenvariables are
//! numbered in the order they are made, and an eigenvariable's level is
//! its number plus one; a metavariable's level is the number of
//! eigenvariables made before it. A metavariable is bound only to a term
//! whose eigenvariables are below its level, and the metavariables of
//! that term are lowered to its level, trailed. An eigenvariable made
//! after a metavariable lies in the subtree of a `∀` above the `∃` that
//! made the metavariable, or on another branch, where none of its
//! terms ever meets the metavariable but through metavariables of lower
//! levels; so a binding the levels allow puts no eigenvariable into the
//! conclusion of its own `∀`, and the checker's condition holds.
//!
//! **Undo.** Every binding and every lowering is on the trail, and the
//! spans, frames and variables only grow; a mark is where each stands,
//! and undoing to it pops the trail and truncates the rest. The engine
//! undoes exactly where it releases pending nodes, and where a binder
//! rule or an identity it tried failed.
//!
//! **Answers.** A rule of two premises whose second premise shares an
//! unbound metavariable with the first may need another answer of the
//! first: one that binds it otherwise. The first premise's answer is the
//! tuple of the shared metavariables' terms; when the second fails under
//! it, it fails under every instance of it (with metavariables no more
//! free and no eigenvariable of a higher level), so the answer is
//! recorded as a pattern and the first premise searched again with every
//! binding that makes the tuple an instance of a recorded pattern
//! refused. Bindings only make the tuple more specific, so the refusal at
//! the binding prunes exactly the proofs whose answer is such an
//! instance, and the search of the first premise finds every other
//! answer. It ends when the bindings before the first premise are an
//! instance themselves, or when the first premise has no proof left.

use super::choices::Choices;
use super::classes::Classes;
use super::context::Context;
use super::scratch::{Pooled, Pools};
use super::zone::{Cursors, Linear, Unrestricted, Zone};
use crate::Instances;
use crate::hash::{HashMap, HashSet};
use crate::occurrences::{Forest, FrameId, Member, OccId, OccSet};
use crate::proofs::Node;
use crate::search::memory::{Charged, bytes_of};
use crate::search::{Options, Reason};
use crate::sequents::fo::FirstOrder;
use crate::sequents::{Eigen, FoTerm, FoTermId, Symbol};
use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// The frame of no binder: an occurrence outside every binder, or a
/// closed term, which reads no frame.
const NONE: u32 = u32::MAX;

#[cfg(test)]
thread_local! {
    /// The first member id a test's search may not give, below the
    /// default's: the bound on members is reached only by a search far
    /// larger than a test runs.
    pub(super) static MEMBER_LIMIT: std::cell::Cell<u32> = const { std::cell::Cell::new(u32::MAX) };
}

/// The first member id a search may not give: `u32::MAX`, which stands
/// for no frame among the frames' parents, or in a test what its thread
/// set.
fn member_limit() -> u32 {
    #[cfg(test)]
    {
        MEMBER_LIMIT.with(std::cell::Cell::get)
    }
    #[cfg(not(test))]
    {
        u32::MAX
    }
}

/// A member of a first-order search: an occurrence of the forest below
/// its length, an occurrence of a span past it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) struct FMember(u32);

impl From<FMember> for Member {
    fn from(m: FMember) -> Self {
        Self::new(m.0)
    }
}

/// A term of a search: a term of the sequent's arena under a frame (an
/// id below the arena's length), or a variable of the search (the id past
/// it by the variable's index), whose frame is [`NONE`], as a closed
/// term's is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct TRef {
    /// The term or the variable.
    id: u32,
    /// The frame its bound variables are read in.
    frame: u32,
}

/// A variable of the search.
#[derive(Clone, Copy, Debug)]
enum Var {
    /// The witness of an `∃` step: unbound, or bound to a term, and its
    /// level, below which every eigenvariable of a term it is bound to
    /// lies.
    Meta {
        /// What it is bound to.
        binding: Option<TRef>,
        /// Its level.
        level: u32,
    },
    /// The eigenvariable of a `∀` step, by its number, one less than its
    /// level.
    Eigen(u32),
}

/// What undoing a step of the trail takes back.
#[derive(Clone, Copy, Debug)]
enum Undone {
    /// The binding of the metavariable of this index.
    Bound(u32),
    /// The lowering of the metavariable of this index from that level.
    Lowered(u32, u32),
    /// What the metavariable of this index was found to stand for.
    Named(u32),
}

/// What a bound metavariable was found to stand for, kept until an undo
/// takes back a binding it rests on.
#[derive(Clone, Copy, Debug)]
enum Named {
    /// Not looked at since it was bound.
    Unknown,
    /// The ground term of this id.
    Ground(u32),
    /// A term that holds the variable of this index, unbound or an
    /// eigenvariable: so while that variable stays unbound.
    Open(u32),
}

/// The head of an application in the table of ground terms: a symbol
/// before its first argument, else the application to the arguments
/// before.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Head {
    /// The symbol.
    Symbol(u32),
    /// The application of this id.
    Applied(u32),
}

/// The table of ground terms: a zone's own, or the one every engine of a
/// search on a pool shares, so that a ground term has one id in all their
/// keys and their memo is one.
#[derive(Clone, Debug)]
enum Grounds {
    /// The zone's own.
    Own(HashMap<(Head, u32), u32>),
    /// The pool's.
    #[cfg(feature = "parallel")]
    Shared(Arc<std::sync::Mutex<HashMap<(Head, u32), u32>>>),
}

impl Grounds {
    /// The name of the application of `symbol` to terms of the names
    /// `args`, as [`Framed::apply`] gives it.
    fn apply(&mut self, symbol: Symbol, args: &[Name]) -> Name {
        match self {
            Self::Own(table) => Framed::apply(table, symbol, args),
            #[cfg(feature = "parallel")]
            Self::Shared(table) => Framed::apply(&mut crate::search::lock(table), symbol, args),
        }
    }

    /// The slots of the table. A shared one is counted by every engine
    /// that reads it, more than it holds, which the memory bound errs on.
    fn capacity(&self) -> usize {
        match self {
            Self::Own(table) => table.capacity(),
            #[cfg(feature = "parallel")]
            Self::Shared(table) => crate::search::lock(table).capacity(),
        }
    }
}

/// A step of the walk that names a metavariable's term.
#[derive(Clone, Copy, Debug)]
enum Naming {
    /// Name a term.
    Term(TRef),
    /// Name a variable, by index.
    Var(u32),
    /// Apply the symbol to the last names found, as many as its arity.
    Apply(Symbol, u32),
    /// Keep the last name found as the metavariable's of this index.
    Keep(u32),
}

/// A name a walk found for a term.
#[derive(Clone, Copy, Debug)]
enum Name {
    /// The ground term of this id.
    Ground(u32),
    /// A term holding the variable of this index.
    Open(u32),
    /// A ground term whose id would pass a key word's value.
    Past,
}

/// The members of one instance of a binder's body: the ids from `base`
/// to `end`, one per occurrence from `first` on, under `frame`.
#[derive(Clone, Copy, Debug)]
struct Span {
    /// The first id.
    base: u32,
    /// The first id past the span.
    end: u32,
    /// The occurrence of the first id: the body's.
    first: u32,
    /// The frame of every member of the span.
    frame: u32,
}

/// Where the trail and the tables of a first-order search stand.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Mark {
    /// The spans.
    spans: usize,
    /// The frames.
    frames: usize,
    /// The variables.
    vars: usize,
    /// The eigenvariables made.
    eigens: u32,
    /// The trail.
    trail: usize,
    /// The eigenvariables whose `∀` node is not built.
    introduced: usize,
}

/// What a unification found.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Unified {
    /// The terms unify, with the bindings made.
    Yes,
    /// They do not, or the bindings make an excluded answer.
    No,
    /// A binding would pass the witness depth: the branch is cut.
    Cut,
}

/// A node of a recorded answer's pattern.
#[derive(Clone, Copy, Debug)]
enum Pat {
    /// A metavariable of the answer, numbered within it.
    Var(u32),
    /// An eigenvariable, by number.
    Eigen(u32),
    /// A closed term of the sequent's arena.
    Closed(u32),
    /// A symbol applied to the patterns `kids[start..start + len]`.
    App(Symbol, u32, u32),
}

/// A recorded answer: per shared metavariable the pattern of its term,
/// and per metavariable of the patterns its level.
#[derive(Clone, Debug, Default)]
struct Pattern {
    /// The nodes.
    nodes: Vec<Pat>,
    /// The children of the applications.
    kids: Vec<u32>,
    /// Per shared metavariable, its term's node.
    roots: Vec<u32>,
    /// Per metavariable of the patterns, its level when recorded.
    levels: Vec<u32>,
}

/// The answers of one rule's first premise.
#[derive(Clone, Debug)]
struct Answers {
    /// The metavariables the second premise shares, by index.
    metas: Vec<u32>,
    /// The answers under which the second premise failed.
    patterns: Vec<Pattern>,
    /// Whether the first premise is being searched, when the patterns
    /// are excluded.
    first: bool,
}

/// The zone whose members are instances of occurrences under binders:
/// the spans, frames and variables, the trail, the answers excluded, the
/// first-order bounds, and the tables read off the sequent's terms.
pub(super) struct Framed {
    /// The forest's length: the first id of a span.
    occurrences: u32,
    /// The sequent's terms: the first id of a variable.
    base: u32,
    /// The spans, one per instance held: the forest's occurrences are
    /// members below them.
    spans: Vec<Span>,
    /// Per frame, its parent ([`NONE`] for none) and its term.
    frames: Vec<(u32, TRef)>,
    /// The variables.
    vars: Vec<Var>,
    /// How many eigenvariables were made: the next one's number.
    eigens: u32,
    /// The bindings and lowerings, to be undone.
    trail: Vec<Undone>,
    /// The eigenvariables of the `∀` steps whose nodes are not built.
    introduced: Vec<Eigen>,
    /// The answers of the rules open on the branch, the innermost last.
    answers: Vec<Answers>,
    /// The choices made while a rule may ask for another answer, and the
    /// replay that resumes its first premise.
    choices: Choices,
    /// The deepest term a metavariable may be bound to.
    witness_depth: u32,
    /// The most instances of binders the search may hold at once.
    instances: u32,
    /// The bound that cut a branch of the level, if one did.
    cut: Option<Reason>,
    /// Per term of the sequent's arena, whether no bound variable lies
    /// below it. This table and the two below never change once the zone
    /// is made: the forks of a pool share them.
    closed: Arc<[bool]>,
    /// Per term of the sequent's arena, its depth: a constant's is 0.
    depth: Arc<[u32]>,
    /// Per predicate and sign, the stamp of the marking that last found
    /// a literal of it among the members.
    marks: Vec<u64>,
    /// The stamp of the last marking.
    stamp: u64,
    /// The terms the walks visited since the engine last took them: the
    /// work a unification, a pattern or a match does, which grows with the
    /// terms' size and not with the search's steps. Atomic, so that the
    /// walks that read the zone count too; one thread uses it.
    steps: AtomicU64,
    /// The unifications of two literals tried.
    unifications: u64,
    /// The metavariables bound.
    bindings: u64,
    /// The bytes of the tables and the walks' buffers the engine was last
    /// told the zone holds.
    charged: usize,
    /// Scratch: pairs of terms to unify or compare.
    pairs: Vec<(TRef, TRef)>,
    /// Scratch: the pairs met in one walk.
    met: HashSet<(TRef, TRef)>,
    /// Scratch: terms to walk, and whether their children were.
    walk: Vec<(TRef, bool)>,
    /// Scratch: the depth of every term a binding's walk met.
    depths: HashMap<TRef, u32>,
    /// Scratch: the buffers the keys of stable sequents are written with.
    key: KeyScratch,
    /// The ground terms the keys name, by their heads and last arguments:
    /// each term met gets one id for the whole search, so equal ids are
    /// equal terms however they were reached.
    ground: Grounds,
    /// Per term of the sequent's arena, its id in `ground` if it is
    /// closed and its id below a key word's value, else [`NONE`].
    arena: Arc<[u32]>,
    /// Per variable, what it was found to stand for, on the trail.
    named: Vec<Named>,
    /// Scratch: the steps of a naming walk.
    naming: Vec<Naming>,
    /// Scratch: the names it found.
    names: Vec<Name>,
}

impl Framed {
    /// Counts a term visited by a walk.
    fn step(&self) {
        self.steps.fetch_add(1, Ordering::Relaxed);
    }

    /// The occurrence and frame of a member.
    fn instance(&self, m: FMember) -> (OccId, u32) {
        if m.0 < self.occurrences {
            return (OccId::new(m.0), NONE);
        }
        let span = self.spans[self.spans.partition_point(|s| s.base <= m.0) - 1];
        (OccId::new(span.first + (m.0 - span.base)), span.frame)
    }

    /// The term the bound variable `i` of `frame` stands for.
    fn frame_term(&self, mut frame: u32, i: u32) -> TRef {
        for _ in 0..i {
            frame = self.frames[frame as usize].0;
        }
        self.frames[frame as usize].1
    }

    /// The term `t` stands for: its bound variables read in its frame,
    /// its metavariables' bindings followed; a closed term without its
    /// frame.
    fn deref(&self, fo: &FirstOrder, mut t: TRef) -> TRef {
        loop {
            if t.id < self.base {
                if let FoTerm::Bound(i) = fo.terms[t.id as usize] {
                    t = self.frame_term(t.frame, i);
                    continue;
                }
                if self.closed[t.id as usize] {
                    t.frame = NONE;
                }
                return t;
            }
            match self.vars[(t.id - self.base) as usize] {
                Var::Meta {
                    binding: Some(b), ..
                } => t = b,
                _ => return t,
            }
        }
    }

    /// The variable a term is, by index, if it is one.
    fn var(&self, t: TRef) -> Option<u32> {
        t.id.checked_sub(self.base)
    }

    /// The level of a variable.
    fn level(&self, v: u32) -> u32 {
        match self.vars[v as usize] {
            Var::Meta { level, .. } => level,
            Var::Eigen(number) => number + 1,
        }
    }

    /// The symbol and the arguments of a term of the arena that is an
    /// application.
    fn app(fo: &FirstOrder, t: TRef) -> (Symbol, &[FoTermId]) {
        match fo.terms[t.id as usize] {
            FoTerm::App(symbol, args) => (symbol, fo.list(args)),
            term => unreachable!("{term:?} is no application after a deref"),
        }
    }

    /// Adds a variable and returns its term.
    fn push_var(&mut self, var: Var) -> Result<TRef, Reason> {
        let id = u32::try_from(self.vars.len())
            .ok()
            .and_then(|v| self.base.checked_add(v))
            .filter(|&id| id < u32::MAX)
            .ok_or(Reason::IndexLimit)?;
        self.vars.push(var);
        self.named.push(Named::Unknown);
        Ok(TRef { id, frame: NONE })
    }

    /// Opens the body of the binder `binder` with `term`: a span of the
    /// body's occurrences under the binder's frame extended by `term`.
    fn open(&mut self, forest: &Forest, binder: FMember, term: TRef) -> Result<FMember, Reason> {
        let (o, frame) = self.instance(binder);
        let frames = u32::try_from(self.frames.len())
            .ok()
            .filter(|&f| f < NONE)
            .ok_or(Reason::IndexLimit)?;
        let base = self.spans.last().map_or(self.occurrences, |s| s.end);
        // A binder's body is one occurrence at the least.
        let end = base
            .checked_add(forest.size(o) - 1)
            .filter(|&end| end <= member_limit())
            .ok_or(Reason::IndexLimit)?;
        self.frames.push((frame, term));
        self.spans.push(Span {
            base,
            end,
            first: o.get() + 1,
            frame: frames,
        });
        Ok(FMember(base))
    }

    /// Whether the search holds as many instances as it may: the next
    /// binder step cuts its branch.
    fn full(&mut self) -> bool {
        if self.spans.len() >= self.instances as usize {
            self.cut = Some(Reason::Instances {
                instances: self.instances,
            });
            return true;
        }
        false
    }

    /// Binds the metavariable `x` to `t`: refused where `x` occurs in
    /// `t` or `t` holds an eigenvariable at or past `x`'s level, cut where
    /// `t` is deeper than the witness depth; the metavariables of `t`
    /// above `x`'s level are lowered to it.
    fn bind(&mut self, fo: &FirstOrder, x: u32, t: TRef) -> Unified {
        let level = self.level(x);
        self.depths.clear();
        self.walk.clear();
        self.walk.push((t, false));
        while let Some((u, expanded)) = self.walk.pop() {
            self.step();
            let u = self.deref(fo, u);
            if !expanded && self.depths.contains_key(&u) {
                continue;
            }
            let depth = if let Some(v) = self.var(u) {
                match self.vars[v as usize] {
                    Var::Meta { level: l, .. } => {
                        if v == x {
                            return Unified::No;
                        }
                        if l > level {
                            self.trail.push(Undone::Lowered(v, l));
                            self.vars[v as usize] = Var::Meta {
                                binding: None,
                                level,
                            };
                        }
                    }
                    Var::Eigen(number) => {
                        if number + 1 > level {
                            return Unified::No;
                        }
                    }
                }
                0
            } else if self.closed[u.id as usize] {
                self.depth[u.id as usize]
            } else {
                let (_, args) = Self::app(fo, u);
                if !expanded {
                    self.walk.push((u, true));
                    self.walk.extend(args.iter().map(|&a| {
                        (
                            TRef {
                                id: a.get(),
                                frame: u.frame,
                            },
                            false,
                        )
                    }));
                    continue;
                }
                let deepest = args
                    .iter()
                    .map(|&a| {
                        let child = self.deref(
                            fo,
                            TRef {
                                id: a.get(),
                                frame: u.frame,
                            },
                        );
                        self.depths[&child]
                    })
                    .max();
                deepest.map_or(0, |d| d + 1)
            };
            if depth > self.witness_depth {
                self.cut = Some(Reason::WitnessDepth {
                    depth: self.witness_depth,
                });
                return Unified::Cut;
            }
            self.depths.insert(u, depth);
        }
        self.trail.push(Undone::Bound(x));
        self.bindings += 1;
        self.vars[x as usize] = Var::Meta {
            binding: Some(t),
            level,
        };
        Unified::Yes
    }

    /// Unifies the pairs of terms on `pairs`.
    fn unify_pairs(&mut self, fo: &FirstOrder) -> Unified {
        self.met.clear();
        while let Some((a, b)) = self.pairs.pop() {
            self.step();
            let (a, b) = (self.deref(fo, a), self.deref(fo, b));
            if a == b || !self.met.insert((a, b)) {
                continue;
            }
            let found = match (self.var(a), self.var(b)) {
                (Some(x), Some(y)) => match (self.vars[x as usize], self.vars[y as usize]) {
                    // The younger of two metavariables is bound to the
                    // older, which lowers nothing.
                    (Var::Meta { level: l, .. }, Var::Meta { level: m, .. }) => {
                        if l >= m {
                            self.bind(fo, x, b)
                        } else {
                            self.bind(fo, y, a)
                        }
                    }
                    (Var::Meta { .. }, Var::Eigen(_)) => self.bind(fo, x, b),
                    (Var::Eigen(_), Var::Meta { .. }) => self.bind(fo, y, a),
                    (Var::Eigen(_), Var::Eigen(_)) => Unified::No,
                },
                (Some(x), None) => match self.vars[x as usize] {
                    Var::Meta { .. } => self.bind(fo, x, b),
                    Var::Eigen(_) => Unified::No,
                },
                (None, Some(y)) => match self.vars[y as usize] {
                    Var::Meta { .. } => self.bind(fo, y, a),
                    Var::Eigen(_) => Unified::No,
                },
                (None, None) => {
                    let ((s, xs), (r, ys)) = (Self::app(fo, a), Self::app(fo, b));
                    if s != r {
                        Unified::No
                    } else {
                        self.pairs.extend(xs.iter().zip(ys).map(|(&x, &y)| {
                            (
                                TRef {
                                    id: x.get(),
                                    frame: a.frame,
                                },
                                TRef {
                                    id: y.get(),
                                    frame: b.frame,
                                },
                            )
                        }));
                        Unified::Yes
                    }
                }
            };
            if found != Unified::Yes {
                self.pairs.clear();
                return found;
            }
        }
        Unified::Yes
    }

    /// Whether two terms are equal, as they stand.
    fn equal(&self, fo: &FirstOrder, a: TRef, b: TRef) -> bool {
        let mut pairs = vec![(a, b)];
        let mut met = HashSet::default();
        while let Some((a, b)) = pairs.pop() {
            self.step();
            let (a, b) = (self.deref(fo, a), self.deref(fo, b));
            if a == b || !met.insert((a, b)) {
                continue;
            }
            if self.var(a).is_some() || self.var(b).is_some() {
                return false;
            }
            let ((s, xs), (r, ys)) = (Self::app(fo, a), Self::app(fo, b));
            if s != r {
                return false;
            }
            pairs.extend(xs.iter().zip(ys).map(|(&x, &y)| {
                (
                    TRef {
                        id: x.get(),
                        frame: a.frame,
                    },
                    TRef {
                        id: y.get(),
                        frame: b.frame,
                    },
                )
            }));
        }
        true
    }

    /// Whether every variable of `t` is below `level`, or at it for a
    /// metavariable: what a metavariable of that level could be bound
    /// to, and be read as.
    fn fits(&self, fo: &FirstOrder, t: TRef, level: u32) -> bool {
        let mut walk = vec![t];
        let mut met = HashSet::default();
        while let Some(u) = walk.pop() {
            self.step();
            let u = self.deref(fo, u);
            if !met.insert(u) {
                continue;
            }
            if let Some(v) = self.var(u) {
                let fits = match self.vars[v as usize] {
                    Var::Meta { level: l, .. } => l <= level,
                    Var::Eigen(number) => number < level,
                };
                if !fits {
                    return false;
                }
            } else if !self.closed[u.id as usize] {
                walk.extend(Self::app(fo, u).1.iter().map(|&a| TRef {
                    id: a.get(),
                    frame: u.frame,
                }));
            }
        }
        true
    }

    /// Whether the terms of the metavariables `metas` are an instance of
    /// `pattern`: its metavariables replaced by terms that fit their
    /// levels, consistently.
    fn instance_of(&self, fo: &FirstOrder, metas: &[u32], pattern: &Pattern) -> bool {
        let mut theta: Vec<Option<TRef>> = vec![None; pattern.levels.len()];
        let mut walk: Vec<(u32, TRef)> = metas
            .iter()
            .zip(&pattern.roots)
            .map(|(&m, &root)| {
                (
                    root,
                    TRef {
                        id: self.base + m,
                        frame: NONE,
                    },
                )
            })
            .collect();
        let mut met = HashSet::default();
        while let Some((node, t)) = walk.pop() {
            self.step();
            let t = self.deref(fo, t);
            if !met.insert((node, t)) {
                continue;
            }
            let fits = match pattern.nodes[node as usize] {
                Pat::Var(k) => match theta[k as usize] {
                    None => {
                        theta[k as usize] = Some(t);
                        self.fits(fo, t, pattern.levels[k as usize])
                    }
                    Some(u) => self.equal(fo, u, t),
                },
                Pat::Eigen(number) => self
                    .var(t)
                    .is_some_and(|v| matches!(self.vars[v as usize], Var::Eigen(n) if n == number)),
                Pat::Closed(id) => self.equal(fo, TRef { id, frame: NONE }, t),
                Pat::App(symbol, start, len) => {
                    if self.var(t).is_some() {
                        false
                    } else {
                        let (s, args) = Self::app(fo, t);
                        let kids = &pattern.kids[start as usize..(start + len) as usize];
                        walk.extend(kids.iter().zip(args).map(|(&kid, &a)| {
                            (
                                kid,
                                TRef {
                                    id: a.get(),
                                    frame: t.frame,
                                },
                            )
                        }));
                        s == symbol
                    }
                }
            };
            if !fits {
                return false;
            }
        }
        true
    }

    /// Whether the bindings as they stand make no answer excluded while
    /// its first premise is searched.
    fn passes(&self, fo: &FirstOrder) -> bool {
        self.answers
            .iter()
            .filter(|answers| answers.first)
            .all(|answers| {
                answers
                    .patterns
                    .iter()
                    .all(|p| !self.instance_of(fo, &answers.metas, p))
            })
    }

    /// The pattern of the terms of `metas` as they stand.
    fn pattern(&self, fo: &FirstOrder, metas: &[u32]) -> Pattern {
        let mut pattern = Pattern::default();
        let mut nodes: HashMap<TRef, u32> = HashMap::default();
        let mut numbered: HashMap<u32, u32> = HashMap::default();
        for &m in metas {
            let root = TRef {
                id: self.base + m,
                frame: NONE,
            };
            let mut walk = vec![(root, false)];
            while let Some((u, expanded)) = walk.pop() {
                self.step();
                let u = self.deref(fo, u);
                if !expanded && nodes.contains_key(&u) {
                    continue;
                }
                let node = if let Some(v) = self.var(u) {
                    match self.vars[v as usize] {
                        Var::Meta { level, .. } => {
                            let k = *numbered.entry(v).or_insert_with(|| {
                                pattern.levels.push(level);
                                (pattern.levels.len() - 1) as u32
                            });
                            Pat::Var(k)
                        }
                        Var::Eigen(number) => Pat::Eigen(number),
                    }
                } else if self.closed[u.id as usize] {
                    Pat::Closed(u.id)
                } else {
                    let (symbol, args) = Self::app(fo, u);
                    let child = |a: &FoTermId| TRef {
                        id: a.get(),
                        frame: u.frame,
                    };
                    if !expanded {
                        walk.push((u, true));
                        walk.extend(args.iter().map(|a| (child(a), false)));
                        continue;
                    }
                    let start = pattern.kids.len() as u32;
                    for a in args {
                        let kid = nodes[&self.deref(fo, child(a))];
                        pattern.kids.push(kid);
                    }
                    Pat::App(symbol, start, args.len() as u32)
                };
                pattern.nodes.push(node);
                nodes.insert(u, (pattern.nodes.len() - 1) as u32);
            }
            pattern.roots.push(nodes[&self.deref(fo, root)]);
        }
        pattern
    }

    /// The marks' index of a literal's predicate, or of its atom where
    /// the sequent's atoms have no predicates, and sign.
    fn key(forest: &Forest, o: OccId, dual: bool) -> Option<usize> {
        let (atom, sign) = (forest.atom(o)?, forest.sign(o)?);
        let fo = &forest.sequent().fo;
        let index = match fo.atoms.get(atom.index()) {
            Some(&(symbol, _)) => symbol.index(),
            None => atom.index(),
        };
        Some(2 * index + (sign as usize ^ usize::from(dual)))
    }
}

// The keys of a first-order search's stable sequents, which its memo and
// loop check read.
//
// A member is written as its occurrence followed by the terms of its
// frame, innermost first, each in prefix order as the bindings resolve
// it: a ground term by its id in the search's table of ground terms
// (the arena's closed terms and every ground term a binding built, each
// once, so equal ids are equal terms however they were reached, and a
// witness built over many bindings is one word, not its length), an
// application holding a variable by its symbol followed by its
// arguments, a metavariable or an eigenvariable by a number. The
// occurrence fixes how many binders lie above it, and a symbol its
// arity, so the words of a member end where its last term does.
//
// **The instance's own key** names every variable as the search does,
// and every metavariable's level beside it: two stable sequents share it
// exactly when they are the same instance, the same members over the
// same unbound metavariables and eigenvariables. The loop check compares
// it: a sequent equal to an ancestor whose metavariables are still
// unbound is that ancestor, and a smallest proof of the ancestor never
// passes through it. A failure cut by a bound is recorded under it, as
// the propositional engine records one under the sequent's own key.
//
// **The canonical key** is the same sequent up to a renaming of its
// variables: the members sorted with every metavariable alike, the
// metavariables then numbered as they first appear, the eigenvariables
// ranked by number, and every metavariable's level written as how many
// of the sequent's eigenvariables lie below it, which is all of its
// level a search of the sequent reads: a binding there only ever meets
// the sequent's own eigenvariables and younger ones, which no
// metavariable of the sequent may take. Two sequents with one canonical
// key are one sequent up to a renaming that keeps the levels' order, so
// a complete failure of either is one of the other.

/// The bits of a term's word that say what it is.
const TAG: u32 = 3 << 30;
/// A ground term, by its id in the table of ground terms.
const GROUND: u32 = 0;
/// An application, by its symbol; its arguments follow.
const APP: u32 = 1 << 30;
/// An eigenvariable, by its number or rank.
const EIGEN: u32 = 2 << 30;
/// A metavariable, by its index or number.
const META: u32 = 3 << 30;
/// The bits of a term's word that hold its value. A value past them is
/// refused: such a sequent is decided without a key.
const VALUE: u32 = !TAG;

/// The most words the members of a key may take: a sequent whose terms,
/// written out, pass it (a witness shared many times over is written as
/// often as it is reached) is decided without the memo and the loop
/// check, which would spend more on its key than on most searches.
const KEY_WORDS: usize = 1 << 16;

/// The buffers the keys are written with, kept between stable sequents.
#[derive(Debug, Default)]
struct KeyScratch {
    /// The members' words, one member after another.
    words: Vec<u32>,
    /// Per member, where its words start, `Γ`'s first.
    starts: Vec<usize>,
    /// Per member, how many copies of it the zone holds.
    copies: Vec<u32>,
    /// The members' order in a key.
    order: Vec<usize>,
    /// The eigenvariables' numbers, sorted, each once.
    eigens: Vec<u32>,
    /// The metavariables by index, in the order a key names them.
    metas: Vec<u32>,
    /// The number of each metavariable in the canonical key.
    numbers: HashMap<u32, u32>,
    /// The terms to write.
    walk: Vec<TRef>,
}

impl KeyScratch {
    /// The bytes its buffers hold; a hash table's entries as its slots
    /// and a control byte each.
    fn bytes(&self) -> usize {
        bytes_of(&self.words)
            + bytes_of(&self.starts)
            + bytes_of(&self.copies)
            + bytes_of(&self.order)
            + bytes_of(&self.eigens)
            + bytes_of(&self.metas)
            + bytes_of(&self.walk)
            + self.numbers.capacity() * (size_of::<(u32, u32)>() + 1)
    }

    /// Where the words of member `i` lie.
    fn range(&self, i: usize) -> std::ops::Range<usize> {
        let end = self.starts.get(i + 1).copied().unwrap_or(self.words.len());
        self.starts[i]..end
    }

    /// The words of member `i`.
    fn member(&self, i: usize) -> &[u32] {
        &self.words[self.range(i)]
    }

    /// Orders members `x` and `y` as the canonical key does: by their
    /// words, a metavariable as any other and an eigenvariable by its
    /// rank.
    fn abstract_order(&self, x: usize, y: usize) -> std::cmp::Ordering {
        let (x, y) = (self.member(x), self.member(y));
        x[0].cmp(&y[0]).then_with(|| {
            x[1..]
                .iter()
                .map(|&w| self.abstracted(w))
                .cmp(y[1..].iter().map(|&w| self.abstracted(w)))
        })
    }

    /// The word as the canonical key's order reads it: a metavariable as
    /// any other, an eigenvariable by its rank.
    fn abstracted(&self, word: u32) -> u32 {
        match word & TAG {
            META => META,
            EIGEN => EIGEN | self.rank(word & VALUE),
            _ => word,
        }
    }

    /// The rank of an eigenvariable among the sequent's.
    fn rank(&self, number: u32) -> u32 {
        // Fewer eigenvariables than words.
        self.eigens.partition_point(|&n| n < number) as u32
    }
}

impl Framed {
    /// Writes the keys of the stable sequent `⊢ Θ ; Γ`: its own and its
    /// canonical one, as said above [`KeyScratch`]. False where a variable, a term
    /// or a symbol passes a word's value or the words pass
    /// [`KEY_WORDS`]: the sequent is then decided without a key.
    fn write_keys(
        &mut self,
        forest: &Forest,
        theta: &Set,
        gamma: &Members,
        own: &mut Vec<OccId>,
        canonical: &mut Vec<OccId>,
    ) -> bool {
        let fo = &forest.sequent().fo;
        let mut s = std::mem::take(&mut self.key);
        s.words.clear();
        s.starts.clear();
        s.copies.clear();
        s.eigens.clear();
        // A member present more than once is written once, with its number
        // of copies: a clause copied a hundred times leaves a hundred
        // copies of one instance, which written out made every key as long.
        let written = gamma
            .0
            .chunk_by(|x, y| x == y)
            .chain(theta.0.chunks(1))
            .all(|copies| {
                u32::try_from(copies.len()).is_ok_and(|n| {
                    s.copies.push(n);
                    self.write_member(fo, copies[0], &mut s)
                })
            });
        if written {
            // Writing the keys reads every word once per key and sorts the
            // members: counted as the terms a walk visits.
            let sorting = usize::BITS - s.starts.len().leading_zeros();
            self.steps.fetch_add(
                (s.words.len() as u64).saturating_mul(2 * u64::from(1 + sorting)),
                Ordering::Relaxed,
            );
            s.eigens.sort_unstable();
            s.eigens.dedup();
            let n = gamma.0.chunk_by(|x, y| x == y).count();
            self.own_key(&mut s, n, own);
            self.canonical_key(&mut s, n, canonical);
        }
        self.key = s;
        written
    }

    /// Writes a member's words, collecting its eigenvariables.
    fn write_member(&mut self, fo: &FirstOrder, m: FMember, s: &mut KeyScratch) -> bool {
        let (o, mut frame) = self.instance(m);
        s.starts.push(s.words.len());
        s.words.push(o.get());
        while frame != NONE {
            let (parent, term) = self.frames[frame as usize];
            s.walk.clear();
            s.walk.push(term);
            while let Some(t) = s.walk.pop() {
                self.step();
                if s.words.len() >= KEY_WORDS {
                    return false;
                }
                let (tag, value) = match self.name(fo, t) {
                    Name::Ground(id) => (GROUND, id),
                    Name::Past => return false,
                    Name::Open(_) => {
                        let t = self.deref(fo, t);
                        if let Some(v) = self.var(t) {
                            match self.vars[v as usize] {
                                Var::Meta { .. } => (META, v),
                                Var::Eigen(number) => {
                                    s.eigens.push(number);
                                    (EIGEN, number)
                                }
                            }
                        } else {
                            let (symbol, args) = Self::app(fo, t);
                            s.walk.extend(args.iter().rev().map(|a| TRef {
                                id: a.get(),
                                frame: t.frame,
                            }));
                            (APP, symbol.get())
                        }
                    }
                };
                if value > VALUE {
                    return false;
                }
                s.words.push(tag | value);
            }
            frame = parent;
        }
        true
    }

    /// The name of the term `t` stands for: its id where it is ground, a
    /// variable it holds where it is not, or past a word's value. Every
    /// bound metavariable the walk passes keeps its name, on the trail,
    /// so a key names a witness built over many bindings in one step:
    /// a name rests only on bindings made before it, which an undo takes
    /// back after it, and an open name is asked again once its variable
    /// is bound.
    fn name(&mut self, fo: &FirstOrder, t: TRef) -> Name {
        // A frame's term is a variable: an unbound one, an eigenvariable
        // or a kept name needs no walk.
        if let Some(v) = self.var(t) {
            match self.vars[v as usize] {
                Var::Meta {
                    binding: Some(_), ..
                } => {
                    if let Some(name) = self.kept(v) {
                        return name;
                    }
                }
                _ => return Name::Open(v),
            }
        }
        let mut steps = std::mem::take(&mut self.naming);
        let mut names = std::mem::take(&mut self.names);
        steps.clear();
        names.clear();
        steps.push(Naming::Term(t));
        while let Some(step) = steps.pop() {
            self.step();
            match step {
                Naming::Term(mut t) => {
                    while t.id < self.base
                        && let FoTerm::Bound(i) = fo.terms[t.id as usize]
                    {
                        t = self.frame_term(t.frame, i);
                    }
                    if let Some(v) = self.var(t) {
                        steps.push(Naming::Var(v));
                    } else if self.closed[t.id as usize] {
                        names.push(match self.arena[t.id as usize] {
                            NONE => Name::Past,
                            id => Name::Ground(id),
                        });
                    } else {
                        let (symbol, args) = Self::app(fo, t);
                        // An arity is below a sequent's length.
                        steps.push(Naming::Apply(symbol, args.len() as u32));
                        steps.extend(args.iter().rev().map(|a| {
                            Naming::Term(TRef {
                                id: a.get(),
                                frame: t.frame,
                            })
                        }));
                    }
                }
                Naming::Var(v) => match self.vars[v as usize] {
                    Var::Meta {
                        binding: Some(b), ..
                    } => match self.kept(v) {
                        Some(name) => names.push(name),
                        None => {
                            steps.push(Naming::Keep(v));
                            steps.push(Naming::Term(b));
                        }
                    },
                    _ => names.push(Name::Open(v)),
                },
                Naming::Apply(symbol, arity) => {
                    let at = names.len() - arity as usize;
                    let name = self.ground.apply(symbol, &names[at..]);
                    names.truncate(at);
                    names.push(name);
                }
                Naming::Keep(v) => {
                    let kept = match names.last() {
                        Some(&Name::Ground(id)) => Named::Ground(id),
                        Some(&Name::Open(w)) => Named::Open(w),
                        _ => continue,
                    };
                    self.named[v as usize] = kept;
                    self.trail.push(Undone::Named(v));
                }
            }
        }
        let name = names.pop().expect("a walk names its term");
        self.naming = steps;
        self.names = names;
        name
    }

    /// The name the bound metavariable `v` keeps, where it still holds.
    fn kept(&self, v: u32) -> Option<Name> {
        match self.named[v as usize] {
            Named::Unknown => None,
            Named::Ground(id) => Some(Name::Ground(id)),
            Named::Open(w) => match self.vars[w as usize] {
                Var::Meta {
                    binding: Some(_), ..
                } => None,
                _ => Some(Name::Open(w)),
            },
        }
    }

    /// The name of the application of `symbol` to terms of the names
    /// `args`: where all are ground, the id of the term in `ground`, a
    /// new one for a new term, the arguments taken one at a time (a
    /// constant applied to none, [`NONE`]); else the first open one, or
    /// past.
    fn apply(ground: &mut HashMap<(Head, u32), u32>, symbol: Symbol, args: &[Name]) -> Name {
        let mut past = false;
        for &arg in args {
            match arg {
                Name::Open(v) => return Name::Open(v),
                Name::Past => past = true,
                Name::Ground(_) => {}
            }
        }
        if past {
            return Name::Past;
        }
        let ids = args.iter().map(|&arg| match arg {
            Name::Ground(id) => id,
            Name::Open(_) | Name::Past => unreachable!("every argument is ground"),
        });
        let mut head = Head::Symbol(symbol.get());
        let mut id = NONE;
        for arg in ids.chain(args.is_empty().then_some(NONE)) {
            let next = u32::try_from(ground.len())
                .ok()
                .filter(|&next| next <= VALUE);
            id = match (ground.entry((head, arg)), next) {
                (Entry::Occupied(entry), _) => *entry.get(),
                (Entry::Vacant(entry), Some(next)) => *entry.insert(next),
                (Entry::Vacant(_), None) => return Name::Past,
            };
            head = Head::Applied(id);
        }
        Name::Ground(id)
    }

    /// The level of a metavariable, by index.
    fn meta_level(&self, v: u32) -> u32 {
        match self.vars[v as usize] {
            Var::Meta { level, .. } => level,
            Var::Eigen(_) => unreachable!("a metavariable's word names a metavariable"),
        }
    }

    /// The instance's own key: the members of `Γ` (the first `n`, each
    /// once) and of `Θ`, each zone's sorted by their words, each member
    /// after its number of copies and each zone after its number of
    /// members, then the level of every metavariable in the order they
    /// appear.
    fn own_key(&self, s: &mut KeyScratch, n: usize, key: &mut Vec<OccId>) {
        let total = s.starts.len();
        s.order.clear();
        s.order.extend(0..total);
        let mut order = std::mem::take(&mut s.order);
        order[..n].sort_by(|&x, &y| s.member(x).cmp(s.member(y)));
        order[n..].sort_by(|&x, &y| s.member(x).cmp(s.member(y)));
        s.metas.clear();
        s.numbers.clear();
        key.clear();
        for members in [&order[..n], &order[n..]] {
            // A zone's distinct members are fewer than a word counts: each
            // is a member id, a `u32`.
            key.push(OccId::new(members.len() as u32));
            for &i in members {
                let range = s.range(i);
                key.push(OccId::new(s.copies[i]));
                key.push(OccId::new(s.words[range.start]));
                for &w in &s.words[range.start + 1..range.end] {
                    if w & TAG == META && s.numbers.insert(w & VALUE, 0).is_none() {
                        s.metas.push(w & VALUE);
                    }
                    key.push(OccId::new(w));
                }
            }
        }
        key.extend(s.metas.iter().map(|&v| OccId::new(self.meta_level(v))));
        s.order = order;
    }

    /// The canonical key: the members of `Γ` (the first `n`, each once,
    /// after its number of copies) and of `Θ`, each zone's sorted with every metavariable alike and every
    /// eigenvariable by its rank, the metavariables numbered as they
    /// first appear in that order, then per metavariable how many of the
    /// sequent's eigenvariables lie below its level.
    fn canonical_key(&self, s: &mut KeyScratch, n: usize, key: &mut Vec<OccId>) {
        let mut order = std::mem::take(&mut s.order);
        order[..n].sort_by(|&x, &y| s.abstract_order(x, y));
        order[n..].sort_by(|&x, &y| s.abstract_order(x, y));
        s.metas.clear();
        s.numbers.clear();
        key.clear();
        for members in [&order[..n], &order[n..]] {
            key.push(OccId::new(members.len() as u32));
            for &i in members {
                let range = s.range(i);
                key.push(OccId::new(s.copies[i]));
                key.push(OccId::new(s.words[range.start]));
                for j in range.start + 1..range.end {
                    let w = s.words[j];
                    let word = match w & TAG {
                        META => {
                            let v = w & VALUE;
                            // Fewer metavariables than words.
                            let next = s.metas.len() as u32;
                            let number = *s.numbers.entry(v).or_insert(next);
                            if number == next {
                                s.metas.push(v);
                            }
                            META | number
                        }
                        EIGEN => EIGEN | s.rank(w & VALUE),
                        _ => w,
                    };
                    key.push(OccId::new(word));
                }
            }
        }
        key.extend(
            s.metas
                .iter()
                .map(|&v| OccId::new(s.rank(self.meta_level(v)))),
        );
        s.order = order;
    }
}

impl Zone for Framed {
    const FIRST_ORDER: bool = true;

    type Member = FMember;
    type Linear = Members;
    type Unrestricted = Set;
    type Duals = usize;

    fn new(forest: &Forest, options: &Options) -> Self {
        let fo = &forest.sequent().fo;
        let mut closed = vec![true; fo.terms.len()];
        let mut depth = vec![0; fo.terms.len()];
        // Every term names only terms before it.
        for (i, term) in fo.terms.iter().enumerate() {
            (closed[i], depth[i]) = match *term {
                FoTerm::Bound(_) => (false, 0),
                FoTerm::App(_, args) => {
                    let args = fo.list(args);
                    (
                        args.iter().all(|a| closed[a.index()]),
                        args.iter().map(|a| depth[a.index()] + 1).max().unwrap_or(0),
                    )
                }
                FoTerm::Eigen(_) | FoTerm::Meta(_) => {
                    unreachable!("a sequent's arena holds no eigenvariable or metavariable")
                }
            };
        }
        // A forest has fewer than 2³² occurrences and a sequent fewer
        // terms than its text or document.
        let occurrences = forest.len() as u32;
        let mut ground = Grounds::Own(HashMap::default());
        let mut arena = vec![NONE; fo.terms.len()];
        // Every term names only terms before it.
        let mut names = Vec::new();
        for (i, term) in fo.terms.iter().enumerate() {
            if let FoTerm::App(symbol, args) = *term
                && closed[i]
            {
                names.clear();
                names.extend(fo.list(args).iter().map(|a| match arena[a.index()] {
                    NONE => Name::Past,
                    id => Name::Ground(id),
                }));
                if let Name::Ground(id) = ground.apply(symbol, &names) {
                    arena[i] = id;
                }
            }
        }
        Self {
            occurrences,
            base: fo.terms.len() as u32,
            spans: Vec::new(),
            frames: Vec::new(),
            vars: Vec::new(),
            eigens: 0,
            trail: Vec::new(),
            introduced: Vec::new(),
            answers: Vec::new(),
            choices: Choices::default(),
            witness_depth: options.witness_depth,
            instances: options.instances,
            cut: None,
            closed: closed.into(),
            depth: depth.into(),
            marks: Vec::new(),
            stamp: 0,
            steps: AtomicU64::new(0),
            unifications: 0,
            bindings: 0,
            charged: 0,
            pairs: Vec::new(),
            met: HashSet::default(),
            walk: Vec::new(),
            depths: HashMap::default(),
            key: KeyScratch::default(),
            ground,
            arena: arena.into(),
            named: Vec::new(),
            naming: Vec::new(),
            names: Vec::new(),
        }
    }

    /// A copy of the spans, frames, variables, bindings and trail, with
    /// the sequent's tables and the table of ground terms the pool shares
    /// ([`Zone::share`]), so
    /// that the worker's keys name what every engine's do; the counters
    /// at zero and the scratch empty. No rule
    /// is open that asks for another answer ([`Zone::divides`]), so there
    /// is no answer and no choice to log.
    #[cfg(feature = "parallel")]
    fn fork(&self) -> Self {
        Self {
            occurrences: self.occurrences,
            base: self.base,
            spans: self.spans.clone(),
            frames: self.frames.clone(),
            vars: self.vars.clone(),
            eigens: self.eigens,
            trail: self.trail.clone(),
            introduced: self.introduced.clone(),
            answers: Vec::new(),
            choices: Choices::default(),
            witness_depth: self.witness_depth,
            instances: self.instances,
            cut: None,
            closed: Arc::clone(&self.closed),
            depth: Arc::clone(&self.depth),
            marks: Vec::new(),
            stamp: 0,
            steps: AtomicU64::new(0),
            unifications: 0,
            bindings: 0,
            charged: 0,
            pairs: Vec::new(),
            met: HashSet::default(),
            walk: Vec::new(),
            depths: HashMap::default(),
            key: KeyScratch::default(),
            ground: self.ground.clone(),
            arena: Arc::clone(&self.arena),
            named: self.named.clone(),
            naming: Vec::new(),
            names: Vec::new(),
        }
    }

    #[cfg(feature = "parallel")]
    fn share(&mut self) {
        if let Grounds::Own(table) = &mut self.ground {
            let table = std::mem::take(table);
            self.ground = Grounds::Shared(Arc::new(std::sync::Mutex::new(table)));
        }
    }

    /// While a rule may ask a premise for another answer, its choices are
    /// logged and replayed in order, and a failure may rest on an answer
    /// excluded: only a choice outside every such rule is divided.
    #[cfg(feature = "parallel")]
    fn divides(&self) -> bool {
        self.answers.is_empty()
    }

    #[cfg(feature = "parallel")]
    fn adopt(&mut self, worker: Self) {
        let (unifications, bindings, cut, charged) =
            (self.unifications, self.bindings, self.cut, self.charged);
        let steps = *self.steps.get_mut();
        *self = Self {
            unifications,
            bindings,
            cut,
            charged,
            steps: AtomicU64::new(steps),
            ..worker
        };
    }

    #[cfg(feature = "parallel")]
    fn cut_by(&mut self, reason: Reason) {
        self.cut.get_or_insert(reason);
    }

    fn root(&self, o: OccId) -> FMember {
        FMember(o.get())
    }

    fn occurrence(&self, m: FMember) -> OccId {
        self.instance(m).0
    }

    fn child(&self, m: FMember, child: OccId) -> FMember {
        FMember(m.0 + (child.get() - self.occurrence(m).get()))
    }

    /// A literal of a predicate whose atoms are all closed, so that its
    /// dual is its atom's.
    fn dual_in(&self, forest: &Forest, literal: FMember, within: &Set) -> Option<FMember> {
        let o = self.occurrence(literal);
        let (atom, sign) = (forest.atom(o)?, forest.sign(o)?);
        within.iter().find(|&d| {
            let d = self.occurrence(d);
            forest.atom(d) == Some(atom) && forest.sign(d) == Some(!sign)
        })
    }

    /// As [`Self::dual_in`], over the members of `rest` in their order.
    fn dual_from(
        &self,
        forest: &Forest,
        literal: FMember,
        rest: &Members,
        _cursors: &mut Cursors,
    ) -> Option<FMember> {
        let o = self.occurrence(literal);
        let (atom, sign) = (forest.atom(o)?, forest.sign(o)?);
        rest.iter().find(|&d| {
            let d = self.occurrence(d);
            forest.atom(d) == Some(atom) && forest.sign(d) == Some(!sign)
        })
    }

    /// The predicates and signs of the members' literals.
    fn mark_literals(&mut self, forest: &Forest, members: &[FMember], scratch: &mut Charged) {
        if self.marks.is_empty() {
            let fo = &forest.sequent().fo;
            let keys = 2 * fo.symbols.len().max(forest.sequent().atom_names().len());
            scratch.charge(keys * size_of::<u64>());
            self.marks = vec![0; keys];
        }
        self.stamp += 1;
        for &m in members {
            if let Some(key) = Self::key(forest, self.occurrence(m), false) {
                self.marks[key] = self.stamp;
            }
        }
    }

    fn duals(&self, _: &Forest, _: FMember) -> Option<usize> {
        None
    }

    fn marked(&self, _: usize) -> Option<FMember> {
        None
    }

    fn first_lookup(&mut self, _: usize) -> bool {
        false
    }

    /// Whether a literal below the formula has a predicate whose dual
    /// literal the last marking found.
    fn meets(&self, forest: &Forest, formula: FMember) -> bool {
        forest
            .subtree(self.occurrence(formula))
            .filter_map(|l| Self::key(forest, l, true))
            .any(|key| self.marks[key] == self.stamp)
    }

    /// An occurrence's class; an instance is its own.
    fn class(&self, classes: &Classes, m: FMember) -> FMember {
        if m.0 < self.occurrences {
            FMember(classes.of(OccId::new(m.0)).get())
        } else {
            m
        }
    }

    fn same(&self, classes: &Classes, x: FMember, y: FMember) -> bool {
        if x.0 < self.occurrences && y.0 < self.occurrences {
            classes.same(OccId::new(x.0), OccId::new(y.0))
        } else {
            x == y
        }
    }

    /// A first-order search keys its sequents by instance
    /// ([`Zone::keys`]), not by occurrence.
    fn memoizes(&self, _: &Set, _: &Members) -> bool {
        false
    }

    /// A pool of its own: the members are no words.
    fn word_lists(pools: &mut Pools<Self>) -> &mut Vec<Pooled<OccId>> {
        &mut pools.words
    }

    fn listed(_: &[OccId]) -> Option<&[FMember]> {
        None
    }

    fn mark(&self) -> Mark {
        Mark {
            spans: self.spans.len(),
            frames: self.frames.len(),
            vars: self.vars.len(),
            eigens: self.eigens,
            trail: self.trail.len(),
            introduced: self.introduced.len(),
        }
    }

    fn undo(&mut self, mark: Mark) {
        while self.trail.len() > mark.trail {
            match self.trail.pop().expect("longer than the mark") {
                Undone::Bound(v) => {
                    if let Var::Meta { binding, .. } = &mut self.vars[v as usize] {
                        *binding = None;
                    }
                }
                Undone::Lowered(v, old) => {
                    if let Var::Meta { level, .. } = &mut self.vars[v as usize] {
                        *level = old;
                    }
                }
                Undone::Named(v) => self.named[v as usize] = Named::Unknown,
            }
        }
        self.spans.truncate(mark.spans);
        self.frames.truncate(mark.frames);
        self.vars.truncate(mark.vars);
        self.named.truncate(mark.vars);
        self.eigens = mark.eigens;
        // The nodes built since took theirs already.
        self.introduced.truncate(mark.introduced);
    }

    fn begin_level(&mut self) {
        self.cut = None;
    }

    fn cut(&self) -> Option<Reason> {
        self.cut
    }

    fn forall(&mut self, forest: &Forest, binder: FMember) -> Result<Option<FMember>, Reason> {
        if self.full() {
            return Ok(None);
        }
        let number = self.eigens;
        let term = self.push_var(Var::Eigen(number))?;
        self.eigens += 1;
        self.introduced.push(Eigen::new(number));
        self.open(forest, binder, term).map(Some)
    }

    fn introduced(&mut self) -> Eigen {
        self.introduced
            .pop()
            .expect("an eigenvariable per `∀` step")
    }

    fn exists(
        &mut self,
        forest: &Forest,
        binder: FMember,
    ) -> Result<Option<(FMember, FoTermId)>, Reason> {
        if self.full() {
            return Ok(None);
        }
        let term = self.push_var(Var::Meta {
            binding: None,
            level: self.eigens,
        })?;
        let body = self.open(forest, binder, term)?;
        Ok(Some((body, FoTermId::new(term.id))))
    }

    fn opposite(&self, forest: &Forest, p: FMember, q: FMember) -> bool {
        let (p, q) = (self.occurrence(p), self.occurrence(q));
        let (Some(a), Some(s), Some(b), Some(t)) = (
            forest.atom(p),
            forest.sign(p),
            forest.atom(q),
            forest.sign(q),
        ) else {
            return false;
        };
        if s == t {
            return false;
        }
        let fo = &forest.sequent().fo;
        match (fo.atoms.get(a.index()), fo.atoms.get(b.index())) {
            (Some(&(x, _)), Some(&(y, _))) => x == y,
            _ => a == b,
        }
    }

    fn unify(&mut self, forest: &Forest, p: FMember, q: FMember) -> Unified {
        self.unifications += 1;
        let fo = &forest.sequent().fo;
        let ((o, f), (r, g)) = (self.instance(p), self.instance(q));
        let (Some(a), Some(b)) = (forest.atom(o), forest.atom(r)) else {
            return Unified::No;
        };
        if let (Some(&(_, xs)), Some(&(_, ys))) = (fo.atoms.get(a.index()), fo.atoms.get(b.index()))
        {
            self.pairs.clear();
            self.pairs
                .extend(fo.list(xs).iter().zip(fo.list(ys)).map(|(&x, &y)| {
                    (
                        TRef {
                            id: x.get(),
                            frame: f,
                        },
                        TRef {
                            id: y.get(),
                            frame: g,
                        },
                    )
                }));
            let found = self.unify_pairs(fo);
            if found != Unified::Yes {
                return found;
            }
        } else if a != b {
            return Unified::No;
        }
        // A replay takes the alternatives the first search took, which
        // passed the answers excluded then.
        if self.choices.replaying() || self.passes(fo) {
            Unified::Yes
        } else {
            Unified::No
        }
    }

    fn open_answers(
        &mut self,
        forest: &Forest,
        members: &mut dyn Iterator<Item = FMember>,
    ) -> bool {
        let fo = &forest.sequent().fo;
        let mut metas = Vec::new();
        let mut frames = HashSet::default();
        let mut met = HashSet::default();
        let mut walk = Vec::new();
        for m in members {
            let mut frame = self.instance(m).1;
            while frame != NONE && frames.insert(frame) {
                let (parent, term) = self.frames[frame as usize];
                walk.push(term);
                frame = parent;
            }
        }
        while let Some(t) = walk.pop() {
            self.step();
            let t = self.deref(fo, t);
            if !met.insert(t) {
                continue;
            }
            match self.var(t) {
                Some(v) => {
                    if matches!(self.vars[v as usize], Var::Meta { .. }) {
                        metas.push(v);
                    }
                }
                None if !self.closed[t.id as usize] => {
                    walk.extend(Self::app(fo, t).1.iter().map(|&a| TRef {
                        id: a.get(),
                        frame: t.frame,
                    }));
                }
                None => {}
            }
        }
        if metas.is_empty() {
            return false;
        }
        metas.sort_unstable();
        self.answers.push(Answers {
            metas,
            patterns: Vec::new(),
            first: true,
        });
        true
    }

    fn searching_first(&mut self, first: bool) {
        self.answers.last_mut().expect("answers open").first = first;
    }

    fn record(&mut self, forest: &Forest) -> usize {
        let fo = &forest.sequent().fo;
        let answers = self.answers.last().expect("answers open");
        let pattern = self.pattern(fo, &answers.metas);
        let bytes = size_of::<Pattern>()
            + pattern.nodes.len() * size_of::<Pat>()
            + (pattern.kids.len() + pattern.roots.len() + pattern.levels.len()) * size_of::<u32>();
        self.answers
            .last_mut()
            .expect("answers open")
            .patterns
            .push(pattern);
        bytes
    }

    fn exhausted(&mut self, forest: &Forest) -> bool {
        let fo = &forest.sequent().fo;
        let answers = self.answers.last().expect("answers open");
        answers
            .patterns
            .iter()
            .any(|p| self.instance_of(fo, &answers.metas, p))
    }

    fn close_answers(&mut self) {
        self.answers.pop();
    }

    fn keys(
        &mut self,
        forest: &Forest,
        theta: &Set,
        gamma: &Members,
        own: &mut Vec<OccId>,
        canonical: &mut Vec<OccId>,
    ) -> bool {
        self.write_keys(forest, theta, gamma, own, canonical)
    }

    /// Whether an answer is excluded while its first premise is searched.
    fn excluding(&self) -> bool {
        self.answers
            .iter()
            .any(|answers| answers.first && !answers.patterns.is_empty())
    }

    fn take_steps(&mut self) -> u64 {
        std::mem::take(self.steps.get_mut())
    }

    fn counts(&self) -> (u64, u64) {
        (self.unifications, self.bindings)
    }

    /// The tables and the walks' buffers by their capacities; a hash
    /// table's entries as its slots and a control byte each.
    fn held(&mut self) -> (usize, usize) {
        let entries = |slots: usize, size: usize| slots * (size + 1);
        let held = bytes_of(&self.spans)
            + bytes_of(&self.frames)
            + bytes_of(&self.vars)
            + bytes_of(&self.trail)
            + bytes_of(&self.introduced)
            + bytes_of(&self.pairs)
            + bytes_of(&self.walk)
            + self.key.bytes()
            + entries(self.met.capacity(), size_of::<(TRef, TRef)>())
            + entries(self.depths.capacity(), size_of::<(TRef, u32)>())
            + entries(self.ground.capacity(), size_of::<((Head, u32), u32)>())
            + bytes_of(&self.named)
            + bytes_of(&self.naming)
            + bytes_of(&self.names)
            + self.choices.bytes();
        (std::mem::replace(&mut self.charged, held), held)
    }

    fn close(&self, forest: &Forest, nodes: &mut [Node]) -> Result<Instances, Reason> {
        let closing = RefCell::new(Closing {
            framed: self,
            fo: &forest.sequent().fo,
            table: Instances::new(forest),
            terms: HashMap::default(),
            frames: HashMap::default(),
            members: HashMap::default(),
            fresh: HashMap::default(),
        });
        for node in nodes.iter_mut() {
            let renamed = node.renamed(|m| -> Result<Member, Reason> {
                Ok(closing.borrow_mut().member(FMember(m.get())))
            })?;
            *node = match renamed {
                Node::Exists(m, w, p) => Node::Exists(
                    m,
                    closing.borrow_mut().term(TRef {
                        id: w.get(),
                        frame: NONE,
                    }),
                    p,
                ),
                node => node,
            };
        }
        Ok(closing.into_inner().table)
    }

    fn choices(&mut self) -> &mut Choices {
        &mut self.choices
    }
}

/// The closing of a proof: its instance table as it is built, and what
/// it gave each term, frame, member and unbound metavariable.
struct Closing<'a> {
    /// The search.
    framed: &'a Framed,
    /// The sequent's tables.
    fo: &'a FirstOrder,
    /// The table.
    table: Instances,
    /// What each term became.
    terms: HashMap<TRef, FoTermId>,
    /// What each frame became.
    frames: HashMap<u32, FrameId>,
    /// What each member past the forest became.
    members: HashMap<u32, Member>,
    /// The fresh constant of each unbound metavariable.
    fresh: HashMap<u32, Eigen>,
}

impl Closing<'_> {
    /// The table's term for a term as the search's bindings left it.
    fn term(&mut self, t: TRef) -> FoTermId {
        let framed = self.framed;
        let mut walk = vec![(t, false)];
        while let Some((u, expanded)) = walk.pop() {
            let u = framed.deref(self.fo, u);
            if !expanded && self.terms.contains_key(&u) {
                continue;
            }
            let id = if let Some(v) = framed.var(u) {
                let eigen = match framed.vars[v as usize] {
                    Var::Eigen(number) => Eigen::new(number),
                    Var::Meta { .. } => {
                        // A constant outside the signature that no `∀`
                        // step introduces: the eigenvariables past every
                        // one the search made.
                        let next = framed.eigens.saturating_add(self.fresh.len() as u32);
                        *self.fresh.entry(v).or_insert(Eigen::new(next))
                    }
                };
                self.table.eigen(eigen)
            } else if framed.closed[u.id as usize] {
                FoTermId::new(u.id)
            } else {
                let (symbol, args) = Framed::app(self.fo, u);
                let child = |a: &FoTermId| TRef {
                    id: a.get(),
                    frame: u.frame,
                };
                if !expanded {
                    walk.push((u, true));
                    walk.extend(args.iter().map(|a| (child(a), false)));
                    continue;
                }
                let ids: Vec<FoTermId> = args
                    .iter()
                    .map(|a| self.terms[&framed.deref(self.fo, child(a))])
                    .collect();
                self.table.apply(symbol, &ids)
            };
            self.terms.insert(u, id);
        }
        self.terms[&framed.deref(self.fo, t)]
    }

    /// The table's frame for a frame of the search.
    fn frame(&mut self, frame: u32) -> FrameId {
        let mut chain = Vec::new();
        let mut f = frame;
        while f != NONE && !self.frames.contains_key(&f) {
            chain.push(f);
            f = self.framed.frames[f as usize].0;
        }
        for &f in chain.iter().rev() {
            let (parent, term) = self.framed.frames[f as usize];
            let parent = (parent != NONE).then(|| self.frames[&parent]);
            let term = self.term(term);
            let id = self.table.frame(parent, term);
            self.frames.insert(f, id);
        }
        self.frames[&frame]
    }

    /// The table's member for a member of the search.
    fn member(&mut self, m: FMember) -> Member {
        if m.0 < self.framed.occurrences {
            return Member::new(m.0);
        }
        if let Some(&member) = self.members.get(&m.0) {
            return member;
        }
        let (o, frame) = self.framed.instance(m);
        let frame = self.frame(frame);
        let member = self.table.member(o, frame);
        self.members.insert(m.0, member);
        member
    }
}

/// A linear zone of a first-order search: its members in ascending
/// order, with repeats.
#[derive(Clone, Debug, Default)]
pub(super) struct Members(Vec<FMember>);

impl Linear<FMember> for Members {
    fn empty(_: &Forest) -> Self {
        Self(Vec::new())
    }

    fn insert(&mut self, m: FMember) {
        let at = self.0.partition_point(|&x| x <= m);
        self.0.insert(at, m);
    }

    fn remove(&mut self, m: FMember) {
        let at = self.0.binary_search(&m).expect("a member");
        self.0.remove(at);
    }

    fn contains(&self, m: FMember) -> bool {
        self.0.binary_search(&m).is_ok()
    }

    fn clear(&mut self) {
        self.0.clear();
    }

    fn copy_from(&mut self, other: &Self) {
        self.0.clone_from(&other.0);
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn iter(&self) -> impl Iterator<Item = FMember> + '_ {
        self.0.iter().copied()
    }

    fn list_into(&self, list: &mut Vec<FMember>) {
        list.extend_from_slice(&self.0);
    }

    fn ground(&self) -> &Context {
        unreachable!("a first-order search keys no sequent by occurrence")
    }
}

/// An unrestricted zone of a first-order search: its members in
/// ascending order, each once, and the empty set of occurrences its
/// interning reads.
#[derive(Clone, Debug)]
pub(super) struct Set(Vec<FMember>, OccSet);

impl Unrestricted<FMember> for Set {
    fn empty(_: &Forest) -> Self {
        Self(Vec::new(), OccSet::empty(0))
    }

    fn insert(&mut self, m: FMember) {
        if let Err(at) = self.0.binary_search(&m) {
            self.0.insert(at, m);
        }
    }

    fn contains(&self, m: FMember) -> bool {
        self.0.binary_search(&m).is_ok()
    }

    fn clear(&mut self) {
        self.0.clear();
    }

    fn copy_from(&mut self, other: &Self) {
        self.0.clone_from(&other.0);
    }

    fn iter(&self) -> impl Iterator<Item = FMember> + '_ {
        self.0.iter().copied()
    }

    /// The empty set: a first-order search keys its sequents by instance,
    /// and its table of zones keeps none, so it gives every zone a new id
    /// without reading it.
    fn ground(&self) -> &OccSet {
        &self.1
    }
}
