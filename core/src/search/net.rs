// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Proof-net search for unit-free MLL, with or without Mix. A cut-free
//! proof of MLL is determined by its axiom linking, so the search
//! enumerates linkings over a [`ProofStructure`] by backtracking and keeps
//! the first one the correctness criterion accepts.
//!
//! Before any link, the count equation `c = t − p + 2` (`c` conclusions,
//! `t` tensor and `p` par occurrences; `≥` with Mix) and the balance of
//! every atom reject most unprovable sequents at once. Then each step
//! chooses the unlinked literal with the fewest admissible partners and
//! tries them in order. A partner is admissible when it is unlinked and
//! dual, when the two literals do not hang under a common `⊗` of one
//! conclusion, when the `⅋`-free skeleton does not join them already, and
//! when the partners of interchangeable literals and subformulas (equal
//! conclusions, equal leaves of one tree of `⊗` alone or of `⅋` alone)
//! stay in the order of their ids, which breaks those symmetries. The exact acyclicity test runs
//! after every link on a small structure and after every fourth link on a
//! large one, and on every complete linking, which is then a proof net: the
//! count equation is the connectedness equation of a complete acyclic
//! linking. The net is sequentialized into the proof term returned.

use super::memory::Account;
use super::{
    Answer, Cadence, Decide, Engine, NotTaken, Options, Reason, Statistics, Stop, Task, Work,
};
use crate::Error;
use crate::fragment::{Fragment, Mode};
use crate::hash::HashMap;
use crate::limits::{Limits, Refusal};
use crate::nets::{Criterion, NetError, ProofStructure, Scratch, VertexId};
use crate::occurrences::{Forest, OccId, Sign};
use crate::sequents::{Atom, Kind};

/// The raw index that stands for "no occurrence".
const NONE: u32 = u32::MAX;

/// The proof-net engine as the front door calls it.
pub(crate) struct Nets;

impl Decide for Nets {
    /// Refuses a fragment beyond unit-free MLL, affine mode and a goal
    /// other than the roots: a structure's conclusions are the forest's
    /// roots.
    fn admits(&self, task: &Task<'_>) -> Result<(), Error> {
        let refused = |because| Error::EngineRefused {
            engine: Engine::Net,
            because,
        };
        crate::nets::exist(task.fragment, task.mode).map_err(|error| match error {
            NetError::Fragment { fragment } => refused(NotTaken::Fragment {
                decides: Fragment::MLL,
                goal: fragment,
            }),
            NetError::Mode { mode } => refused(NotTaken::Mode { mode }),
            other => other.into(),
        })?;
        if !task.roots {
            return Err(refused(NotTaken::Goal));
        }
        Ok(())
    }

    /// Links the roots on one thread, or with the feature `parallel` on
    /// [`Options::jobs`] threads.
    fn decide(
        &self,
        task: &Task<'_>,
        options: &Options,
        limits: &Limits,
        _account: &Account,
        work: &Work,
        stop: &mut dyn FnMut(u64) -> bool,
    ) -> Result<Answer, Error> {
        #[cfg(feature = "parallel")]
        if options.threads() > 1 {
            let runtime = super::parallel::Lent::take(
                options.pool.as_ref(),
                options.threads(),
                limits.stack_bytes(),
            )?;
            return Ok(parallel::search(
                task.forest,
                task.mode,
                options,
                limits,
                &runtime,
                work,
                stop,
            ));
        }
        #[cfg(not(feature = "parallel"))]
        let _ = work;
        Ok(search(task.forest, task.mode, options, limits, stop))
    }
}

/// Runs the proof-net engine on the forest of a sequent of unit-free MLL
/// under `mode`, polling `stop` once per literal chosen, and returns its
/// answer, with the proof net the proof was read off for a proved
/// sequent.
pub(crate) fn search(
    forest: &Forest,
    mode: Mode,
    options: &Options,
    limits: &Limits,
    stop: &mut dyn FnMut(u64) -> bool,
) -> Answer {
    if !counts_admit(forest, mode.mix) {
        return answer(Ok(false), Statistics::default(), None, limits, stop);
    }
    let (result, statistics, net) = {
        let mut engine = Linker::new(forest, mode, options, Stop::Closure(stop));
        let result = engine.run();
        let net = matches!(result, Ok(true)).then_some(engine.net);
        (result, engine.statistics, net)
    };
    answer(result, statistics, net, limits, stop)
}

/// The answer of a search that linked the roots (`Ok(true)`, with the net
/// found), found that no linking is a proof net (`Ok(false)`), or stopped:
/// the net sequentialized into the proof it is, within `limits` and until
/// `stop` fires, either of which makes the answer unknown.
fn answer(
    result: Result<bool, Reason>,
    statistics: Statistics,
    net: Option<ProofStructure>,
    limits: &Limits,
    stop: &mut dyn FnMut(u64) -> bool,
) -> Answer {
    let result = result.and_then(|linked| {
        if !linked {
            return Ok(None);
        }
        let net = net.as_ref().expect("a proof net was found");
        match net.sequentialize(limits, |_| stop(0)) {
            Ok(proof) => Ok(Some(proof)),
            Err(Error::Net(error)) => match *error {
                NetError::Refused {
                    refusal: Refusal::Memory { limit_bytes, .. },
                } => Err(Reason::MemoryLimit { limit_bytes }),
                NetError::Refused { .. } => Err(Reason::Stopped),
                error => {
                    panic!("a complete linking that passed the exact test is a proof net: {error}")
                }
            },
            Err(error) => panic!("a proof net sequentializes: {error}"),
        }
    });
    let net = net.filter(|_| matches!(result, Ok(Some(_))));
    Answer {
        result,
        statistics,
        net,
        refutation: None,
    }
}

/// Returns whether the counts admit a proof: `c = t − p + 2` for `c`
/// conclusions, `t` tensor and `p` par occurrences (`c ≥ t − p + 2` with
/// Mix), and as many `a` as `~a` for every atom. Both are necessary for a
/// proof net to exist, and the first is the connectedness equation of a
/// complete acyclic linking.
fn counts_admit(forest: &Forest, mix: bool) -> bool {
    let (mut tensors, mut pars) = (0i64, 0i64);
    for o in forest.ids() {
        match forest.kind(o) {
            Kind::Tensor => tensors += 1,
            Kind::Par => pars += 1,
            _ => {}
        }
    }
    let conclusions = forest.roots().len() as i64;
    let needed = tensors - pars + 2;
    if if mix {
        conclusions < needed
    } else {
        conclusions != needed
    } {
        return false;
    }
    atoms(forest)
        .all(|a| forest.literals(a, Sign::Atom).len() == forest.literals(a, Sign::Dual).len())
}

/// Returns every atom of the forest's sequent.
fn atoms(forest: &Forest) -> impl Iterator<Item = Atom> {
    (0..forest.sequent().atom_names().len()).map(|a| Atom::new(a as u32))
}

/// The prunes of the search beyond the rejections every link passes;
/// a search applies all of them, and the tests switch each off to show
/// what it cuts.
#[derive(Clone, Copy, Debug)]
struct Prunes {
    /// Break the symmetry of equal literals under one tree.
    leaves: bool,
    /// Break the symmetry of equal compound subformulas.
    compounds: bool,
    /// Refuse a skeleton component that holds more literals of an atom
    /// than the atom has pairs left.
    balance: bool,
}

impl Prunes {
    /// Every prune.
    const ALL: Self = Self {
        leaves: true,
        compounds: true,
        balance: true,
    };
}

/// The symmetries of a sequent that the search breaks, as chains in id
/// order along which the partners must not descend.
///
/// Two kinds of exchange map proof nets to proof nets, and to linkings
/// that are not nets the linkings that are not. Exchanging the partners
/// of two equal literals that are conclusions, or leaves of one maximal
/// tree of `⊗` alone or of `⅋` alone (a literal class): the leaves of a
/// `⊗` tree hang from one vertex once the tree is contracted, which
/// keeps the cyclomatic number and the components of every switching,
/// and every switching of a `⅋` tree joins its root to one leaf and
/// leaves the others as paths, so its switchings join the root to each
/// leaf in turn. And exchanging two equal compound subformulas that are
/// conclusions or leaves of one such tree (a compound class), by the
/// same argument for whole subtrees. Among the linkings that differ by
/// such exchanges, one has ascending partners along every literal class
/// and keys that do not descend along every compound class whose copies
/// have a key literal: sort the copies of each compound class by key,
/// outer classes first, then take the lexicographically least linking
/// that exchanges within literal classes reach. A key literal is in no
/// literal class of two or more and inside no copy of a smaller broken
/// class, so neither later step moves it, and a partner's key, the
/// first literal of its class or none inside a broken copy, is the
/// same for every partner those exchanges move it to.
#[derive(Clone, Debug)]
struct Symmetry {
    /// Per occurrence, its neighbours along the chains.
    chains: Box<[Chains]>,
    /// Per literal, its key as a partner: the first literal of its class,
    /// or `NONE`, above every key, inside a copy of a broken class.
    key: Box<[u32]>,
}

/// The neighbours of a literal along the chains of [`Symmetry`], side by
/// side, since a candidate link reads all four.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Chains {
    /// The previous literal of its class, or `NONE`.
    before: u32,
    /// The next literal of its class, or `NONE`.
    after: u32,
    /// For the key literal of a copy of a broken compound class, the key
    /// literal of the previous copy, or `NONE`.
    key_before: u32,
    /// For the key literal of a copy of a broken compound class, the key
    /// literal of the next copy, or `NONE`.
    key_after: u32,
}

impl Chains {
    /// A literal on no chain.
    const NONE: Self = Self {
        before: NONE,
        after: NONE,
        key_before: NONE,
        key_after: NONE,
    };
}

impl Symmetry {
    /// Finds the classes of a forest of unit-free MLL: the literal
    /// classes of leaves only with `prunes.leaves`, and the compound
    /// classes only with `prunes.compounds`.
    fn new(forest: &Forest, prunes: Prunes) -> Self {
        let n = forest.len();
        // The anchor of an occurrence: the top of the maximal tree of its
        // parent's connective that holds the parent, or `n` for a root.
        let mut top = vec![NONE; n];
        let mut anchor = vec![n as u32; n];
        for o in forest.ids() {
            if let Some(p) = forest.parent(o) {
                anchor[o.index()] = top[p.index()];
            }
            if !forest.is_literal(o) {
                top[o.index()] = match forest.parent(o) {
                    Some(p) if forest.kind(p) == forest.kind(o) => top[p.index()],
                    _ => o.get(),
                };
            }
        }
        // Literal classes: one atom, sign and anchor, chained in id order.
        let chained = |l: &&OccId| prunes.leaves || anchor[l.index()] == n as u32;
        let (mut before, mut after) = (vec![NONE; n], vec![NONE; n]);
        let mut last = vec![NONE; n + 1];
        for a in atoms(forest) {
            for sign in [Sign::Atom, Sign::Dual] {
                let literals = forest.literals(a, sign);
                for &l in literals.iter().filter(chained) {
                    let previous = &mut last[anchor[l.index()] as usize];
                    if *previous != NONE {
                        after[*previous as usize] = l.get();
                        before[l.index()] = *previous;
                    }
                    *previous = l.get();
                }
                for &l in literals.iter().filter(chained) {
                    last[anchor[l.index()] as usize] = NONE;
                }
            }
        }
        // Compound classes: equal formulas with one anchor, of which no
        // ancestor below the anchor has the anchor's connective.
        let mut shape = vec![0u32; n];
        let mut shapes = HashMap::<(Kind, u32, u32), u32>::default();
        let mut compounds = Vec::new();
        for o in forest.ids().rev() {
            let key = match (forest.left(o), forest.right(o)) {
                (Some(l), Some(r)) => (forest.kind(o), shape[l.index()], shape[r.index()]),
                _ => (forest.kind(o), forest.atom(o).map_or(0, Atom::get), 0),
            };
            let next = shapes.len() as u32;
            shape[o.index()] = *shapes.entry(key).or_insert(next);
            let leaf = forest
                .parent(o)
                .is_none_or(|p| forest.kind(p) != forest.kind(o));
            if prunes.compounds && !forest.is_literal(o) && leaf {
                compounds.push((anchor[o.index()], shape[o.index()], o));
            }
        }
        compounds.sort_unstable();
        let mut classes: Vec<&[(u32, u32, OccId)]> = compounds
            .chunk_by(|x, y| (x.0, x.1) == (y.0, y.1))
            .filter(|class| class.len() > 1)
            .collect();
        // Inner classes first, so that a key literal avoids their copies.
        classes.sort_unstable_by_key(|class| std::cmp::Reverse(class[0].2));
        let (mut key_before, mut key_after) = (vec![NONE; n], vec![NONE; n]);
        let mut broken = vec![false; n];
        let mut keyed = Vec::new();
        for class in classes {
            let first = class[0].2;
            let Some(offset) = forest.subtree(first).position(|x| {
                forest.is_literal(x)
                    && before[x.index()] == NONE
                    && after[x.index()] == NONE
                    && !broken[x.index()]
            }) else {
                continue;
            };
            keyed.push((class, offset as u32));
            for &(_, _, copy) in class {
                for x in forest.subtree(copy) {
                    broken[x.index()] = true;
                }
            }
        }
        let mut key = vec![NONE; n];
        for o in forest.ids() {
            if forest.is_literal(o) && !broken[o.index()] {
                key[o.index()] = match before[o.index()] {
                    NONE => o.get(),
                    b => key[b as usize],
                };
            }
        }
        // A class whose key literals can only take partners keyed `NONE`
        // gets no chain: its order would hold whatever the partners.
        for (class, offset) in keyed {
            let l = OccId::new(class[0].2.get() + offset);
            let (atom, sign) = (forest.atom(l).unwrap(), forest.sign(l).unwrap());
            if forest
                .literals(atom, !sign)
                .iter()
                .all(|y| key[y.index()] == NONE)
            {
                continue;
            }
            let mut previous = NONE;
            for &(_, _, copy) in class {
                let l = copy.get() + offset;
                if previous != NONE {
                    key_after[previous as usize] = l;
                    key_before[l as usize] = previous;
                }
                previous = l;
            }
        }
        let chains = (0..n)
            .map(|i| Chains {
                before: before[i],
                after: after[i],
                key_before: key_before[i],
                key_after: key_after[i],
            })
            .collect();
        Self {
            chains,
            key: key.into_boxed_slice(),
        }
    }
}

/// A decision of the search: the literal chosen, and how far along the
/// literals of the other sign of its atom the search for a partner is.
#[derive(Clone, Copy, Debug)]
struct Frame {
    /// The literal being linked.
    literal: OccId,
    /// The position in the list of literals of the other sign at which the
    /// search for the next partner resumes.
    next: u32,
    /// Whether the literal had one admissible partner when it was chosen:
    /// its link is no choice.
    forced: bool,
}

/// What the choice of the next literal found.
#[derive(Clone, Copy, Debug)]
enum Choice {
    /// Every literal is linked.
    Complete,
    /// Some unlinked literal has no admissible partner.
    DeadEnd,
    /// The unlinked literal with the fewest admissible partners, and
    /// whether it has only one.
    Literal(OccId, bool),
}

/// The state of one run: the structure being linked, the working memory
/// of the exact test, the counts the choice of the next literal reads, the
/// stack of decisions and the counters. Nothing allocates once the run has
/// started.
struct Linker<'a> {
    /// The proof structure being linked.
    net: ProofStructure,
    /// Working memory for the exact test.
    scratch: Scratch,
    /// Per atom, the pairs of its literals still to link: as many `a` as
    /// `~a` are unlinked at every moment.
    remaining: Box<[u32]>,
    /// The symmetries the search breaks.
    symmetry: Symmetry,
    /// Whether the search refuses unbalanced skeleton components.
    balance: bool,
    /// Per vertex, a count the balance prune keeps for the component the
    /// vertex names, zero between two uses.
    tally: Box<[u32]>,
    /// The decisions made so far. Every frame but the top has its current
    /// link made; the top frame is looking for one.
    stack: Vec<Frame>,
    /// How many frames of the stack are choices: not forced.
    choices: usize,
    /// How many links go between two exact tests.
    period: u32,
    /// The counters.
    statistics: Statistics,
    /// The stop condition.
    stop: Stop<'a>,
}

impl<'a> Linker<'a> {
    /// Prepares a run on the forest, which must be one of unit-free MLL.
    fn new(forest: &Forest, mode: Mode, options: &Options, stop: Stop<'a>) -> Self {
        Self::with_prunes(forest, mode, options, stop, Prunes::ALL)
    }

    /// Prepares a run that applies only the given prunes.
    fn with_prunes(
        forest: &Forest,
        mode: Mode,
        options: &Options,
        stop: Stop<'a>,
        prunes: Prunes,
    ) -> Self {
        let net = ProofStructure::new(forest.clone(), Criterion { mix: mode.mix })
            .expect("the dispatch routes unit-free MLL only");
        let scratch = net.scratch();
        let remaining = atoms(forest)
            .map(|a| forest.literals(a, Sign::Atom).len() as u32)
            .collect();
        let n = forest.len();
        let period = match options.test_period {
            Cadence::Every(period) => period.max(1),
            Cadence::Auto if n <= Cadence::AUTO_SMALL => 1,
            Cadence::Auto => Cadence::AUTO_PERIOD,
        };
        Self {
            scratch,
            remaining,
            symmetry: Symmetry::new(forest, prunes),
            balance: prunes.balance,
            tally: vec![0; n].into_boxed_slice(),
            stack: Vec::with_capacity(forest.all_literals().len() / 2 + 1),
            choices: 0,
            period,
            statistics: Statistics::default(),
            stop,
            net,
        }
    }

    /// Runs the search and returns whether a proof net was found, or the
    /// reason the search stopped. With links seeded, the search is the
    /// branch below them and ends when it backtracks up to them.
    fn run(&mut self) -> Result<bool, Reason> {
        self.explore(None, &mut Vec::new())
    }

    /// The search, and the enumeration of its cubes: with a `limit`, a
    /// branch that reaches that many choices (links of literals that had
    /// more than one admissible partner; a forced link is none) is
    /// recorded in `cubes` as the links made, and taken back instead of
    /// followed, so that the cubes are the branches of the first `limit`
    /// choices that the tests do not reject; a proof net found within the
    /// limit ends the search as usual. Without a limit this is the whole
    /// search. The stop condition is polled at every literal chosen and
    /// at every exact test that fails, so a run of failures, which
    /// chooses no literal, is polled too.
    fn explore(
        &mut self,
        limit: Option<usize>,
        cubes: &mut Vec<Vec<(VertexId, VertexId)>>,
    ) -> Result<bool, Reason> {
        // A dead end before any link is a refutation; a complete structure
        // without a link would have no literal, which the counts exclude.
        let Choice::Literal(first, forced) = self.decide()? else {
            return Ok(false);
        };
        self.push(first, forced);
        loop {
            let Some(y) = self.next_partner() else {
                self.pop();
                if self.stack.is_empty() {
                    return Ok(false);
                }
                self.unlink();
                continue;
            };
            let x = self.stack.last().unwrap().literal;
            self.link(x, y);
            let complete = self.net.is_complete();
            if complete || (self.net.links().len() as u32).is_multiple_of(self.period) {
                self.statistics.tests += 1;
                if !self.net.is_acyclic(&mut self.scratch) {
                    self.unlink();
                    if self.stop.fired(1) {
                        return Err(Reason::Stopped);
                    }
                    continue;
                }
            }
            if complete {
                return Ok(true);
            }
            if limit == Some(self.choices) {
                cubes.push(self.net.links().to_vec());
                self.unlink();
                continue;
            }
            match self.decide()? {
                Choice::Literal(literal, forced) => self.push(literal, forced),
                Choice::DeadEnd => self.unlink(),
                Choice::Complete => unreachable!("an incomplete structure has an unlinked literal"),
            }
        }
    }

    /// Opens a decision on a literal.
    fn push(&mut self, literal: OccId, forced: bool) {
        self.choices += usize::from(!forced);
        self.stack.push(Frame {
            literal,
            next: 0,
            forced,
        });
    }

    /// Closes the top decision, whose partners are used up.
    fn pop(&mut self) {
        let frame = self.stack.pop().expect("a decision to close");
        self.choices -= usize::from(!frame.forced);
    }

    /// Polls the stop condition, counts a node, and chooses the next
    /// literal.
    fn decide(&mut self) -> Result<Choice, Reason> {
        self.statistics.nodes += 1;
        if self.stop.fired(1) {
            return Err(Reason::Stopped);
        }
        Ok(self.choose())
    }

    /// Makes the links of a cube, on an engine without links, so that
    /// `run` searches the branch below them.
    #[cfg(feature = "parallel")]
    fn seed(&mut self, links: &[(VertexId, VertexId)]) {
        debug_assert!(
            self.net.links().is_empty(),
            "a seed goes on an empty structure"
        );
        for &(x, y) in links {
            self.link(x.occ(), y.occ());
        }
    }

    /// Takes every link back and forgets the decisions, keeping the
    /// counters, so that the engine can run another cube.
    #[cfg(feature = "parallel")]
    fn reset(&mut self) {
        self.stack.clear();
        self.choices = 0;
        while !self.net.links().is_empty() {
            self.unlink();
        }
    }

    /// Chooses the unlinked literal with the fewest admissible partners,
    /// the first in atom order, `a` before `~a`, then id order among
    /// equals; or reports that some literal has none, or that every
    /// literal is linked. Every unlinked literal is looked at, so a dead
    /// end anywhere is found now rather than after more links; so is an
    /// atom unbalanced over the skeleton's components, where the exact
    /// test runs.
    fn choose(&mut self) -> Choice {
        let balance = self.balance && (self.net.links().len() as u32).is_multiple_of(self.period);
        let mut best: Option<(usize, OccId)> = None;
        for a in 0..self.remaining.len() {
            let remaining = self.remaining[a];
            if remaining == 0 {
                continue;
            }
            let atom = Atom::new(a as u32);
            // A component that breaks the balance while each of its
            // literals keeps an admissible partner holds two of each sign
            // and leaves one of each outside: three pairs at least. With
            // fewer, the count of partners below finds the literal left
            // without one. The check runs where the exact test does,
            // which finds every such branch dead some links later.
            if balance && remaining > 2 && !self.balanced(atom, remaining) {
                return Choice::DeadEnd;
            }
            let forest = self.net.forest();
            for sign in [Sign::Atom, Sign::Dual] {
                let partners = forest.literals(atom, !sign);
                for &x in forest.literals(atom, sign) {
                    if self.net.mate(VertexId::of(x)).is_some() {
                        continue;
                    }
                    // Counting stops once the literal cannot beat the best.
                    let bound = best.map_or(usize::MAX, |(count, _)| count);
                    let mut count = 0;
                    for &y in partners {
                        if self.net.mate(VertexId::of(y)).is_none() && self.admissible(x, y) {
                            count += 1;
                            if count >= bound {
                                break;
                            }
                        }
                    }
                    if count == 0 {
                        return Choice::DeadEnd;
                    }
                    if count < bound {
                        best = Some((count, x));
                    }
                }
            }
        }
        match best {
            Some((count, literal)) => Choice::Literal(literal, count == 1),
            None => Choice::Complete,
        }
    }

    /// Returns whether no component of the `⅋`-free skeleton holds more
    /// unlinked literals of `atom` than its `remaining` pairs. No literal
    /// links into its own component (that closes a cycle every switching
    /// keeps), so each `a` of a component needs a `~a` of its own outside
    /// the component, and each `~a` an `a`: the component's `a` are at
    /// most the atom's unlinked `~a` less the component's, which is the
    /// bound. A count is at most the atom's literals, below the forest's
    /// length.
    fn balanced(&mut self, atom: Atom, remaining: u32) -> bool {
        let (net, tally) = (&self.net, &mut self.tally);
        let forest = net.forest();
        let unlinked = || {
            [Sign::Atom, Sign::Dual]
                .into_iter()
                .flat_map(move |sign| forest.literals(atom, sign))
                .filter(|&&x| net.mate(VertexId::of(x)).is_none())
                .map(|&x| net.component(VertexId::of(x)) as usize)
        };
        let mut balanced = true;
        for c in unlinked() {
            tally[c] += 1;
            balanced &= tally[c] <= remaining;
        }
        for c in unlinked() {
            tally[c] = 0;
        }
        balanced
    }

    /// Finds the next admissible partner of the top frame's literal, moves
    /// the frame past it and returns it, or `None` when none is left.
    fn next_partner(&mut self) -> Option<OccId> {
        let Frame {
            literal: x, next, ..
        } = *self.stack.last().unwrap();
        let forest = self.net.forest();
        let partners = forest.literals(forest.atom(x).unwrap(), !forest.sign(x).unwrap());
        for (i, &y) in partners.iter().enumerate().skip(next as usize) {
            if self.net.mate(VertexId::of(y)).is_none() && self.admissible(x, y) {
                self.stack.last_mut().unwrap().next = i as u32 + 1;
                return Some(y);
            }
        }
        None
    }

    /// Returns whether linking two unlinked dual literals passes the
    /// constant-time rejections: they do not hang under a common `⊗` of
    /// one conclusion (the tree path and the link would be a cycle that
    /// the switchings keeping the path's `⅋` premises keep, and it stays
    /// one whatever is linked later), the `⅋`-free skeleton does not join
    /// them already (the path and the link would be a cycle every
    /// switching keeps), and the link keeps the order the symmetry break
    /// asks along the literal and compound classes of both.
    fn admissible(&self, x: OccId, y: OccId) -> bool {
        let forest = self.net.forest();
        if let Some(a) = forest.lca(x, y)
            && forest.kind(a) == Kind::Tensor
        {
            return false;
        }
        if self.net.joined(VertexId::of(x), VertexId::of(y)) {
            return false;
        }
        let chains = &self.symmetry.chains;
        let (cx, cy) = (chains[x.index()], chains[y.index()]);
        (cx == Chains::NONE || self.ordered(cx, y)) && (cy == Chains::NONE || self.ordered(cy, x))
    }

    /// Returns whether linking `y` to the literal whose chains are given
    /// keeps the partners ascending along its literal class and, for the
    /// key literal of a copy of a broken compound class, the keys of the
    /// partners from descending along the copies ([`Symmetry`]).
    fn ordered(&self, chains: Chains, y: OccId) -> bool {
        let partner = |l: u32| {
            (l != NONE)
                .then(|| self.net.mate(VertexId::new(l)))
                .flatten()
        };
        let key = &self.symmetry.key;
        if partner(chains.before).is_some_and(|p| p.occ() > y)
            || partner(chains.after).is_some_and(|q| q.occ() < y)
        {
            return false;
        }
        !(partner(chains.key_before).is_some_and(|p| key[p.index()] > key[y.index()])
            || partner(chains.key_after).is_some_and(|q| key[y.index()] > key[q.index()]))
    }

    /// Makes a link between two unlinked dual literals and keeps the counts
    /// current.
    fn link(&mut self, x: OccId, y: OccId) {
        self.net.link_unchecked(VertexId::of(x), VertexId::of(y));
        let atom = self.net.forest().atom(x).unwrap();
        self.remaining[atom.index()] -= 1;
        self.statistics.links += 1;
    }

    /// Takes back the last link and keeps the counts current.
    fn unlink(&mut self) {
        let (x, _) = self.net.unlink().expect("a link to take back");
        let atom = self.net.forest().atom(x.occ()).unwrap();
        self.remaining[atom.index()] += 1;
    }
}

#[cfg(all(test, feature = "parse"))]
pub(super) mod tests {
    use super::*;
    use crate::search::generate::{self, Rng, Rules, Tree};
    use crate::search::{Engine, Verdict, prove};
    use crate::sequents::Sequent;

    /// Runs the net engine on `input` under `mode` with `options`, checks
    /// the proof if there is one against the checker and against the net,
    /// and returns the verdict and the statistics.
    fn run(input: &str, mode: Mode, options: &Options) -> (Verdict, Statistics) {
        let s: Sequent = input.parse().unwrap_or_else(|e| panic!("{input:?}: {e}"));
        let options = options.clone().with_engine(Some(Engine::Net));
        let outcome = prove(&s, mode, &options).unwrap_or_else(|e| panic!("{input:?}: {e}"));
        let (verdict, statistics, net) = (outcome.verdict, outcome.statistics, outcome.net);
        match (&verdict, &net) {
            (Verdict::Proved(proof), Some(net)) => {
                assert_eq!(proof.sequent(), &s);
                proof
                    .check(mode)
                    .unwrap_or_else(|e| panic!("{input:?}: the proof is wrong: {e}"));
                assert_eq!(net.is_correct(|_| false), Ok(()), "{input:?}");
                let criterion = Criterion { mix: mode.mix };
                let again =
                    ProofStructure::from_proof(proof, criterion, &Limits::default(), |_| false)
                        .unwrap();
                assert_eq!(sorted(&again), sorted(net), "{input:?}: the proof's net");
            }
            (Verdict::Proved(_), None) | (_, Some(_)) => {
                panic!("{input:?}: a net exactly when proved")
            }
            _ => {}
        }
        (verdict, statistics)
    }

    /// The links of a structure as sorted pairs.
    fn sorted(net: &ProofStructure) -> Vec<(VertexId, VertexId)> {
        let mut links: Vec<(VertexId, VertexId)> = net
            .links()
            .iter()
            .map(|&(x, y)| (x.min(y), x.max(y)))
            .collect();
        links.sort();
        links
    }

    /// Whether `input` is provable under `mode`, panicking on `Unknown`.
    fn provable(input: &str, mode: Mode) -> bool {
        match run(input, mode, &Options::default()).0 {
            Verdict::Proved(_) => true,
            Verdict::Unprovable(_) => false,
            Verdict::Unknown(reason) => panic!("{input:?}: {reason}"),
        }
    }

    /// The mode a rule set of the generator stands for.
    fn mode_for(mix: bool) -> Mode {
        if mix {
            Mode::CLASSICAL.with_mix()
        } else {
            Mode::CLASSICAL
        }
    }

    /// The classic small sequents get the verdicts the textbooks give
    /// them, with and without Mix.
    #[test]
    fn classic_sequents() {
        let m = Mode::CLASSICAL;
        for (input, expected) in [
            ("|- ~a, a", true),
            ("a |- a", true),
            ("|- a, a", false),
            ("|-", false),
            ("|- a, ~a, a, ~a", false),
            ("a * b |- a * b", true),
            ("|- a * b, ~a par ~b", true),
            ("|- a * b, ~a, ~b", true),
            ("|- a par b, ~a, ~b", false),
            ("|- (a * b) par (~a * ~b)", false),
            ("|- a * ~a", false),
            ("|- a par ~a", true),
            ("|- a * b, ~a * ~b", false),
            ("|- a * b, c * (~a par ~b), ~c", true),
            ("a, a -o b, b -o c |- c", true),
            ("a -o b, b -o c |- a -o c", true),
            ("a -o b |- b -o a", false),
            ("|- (a par ~a) par (b par ~b)", false),
            ("|- a, a, ~a * ~a", true),
            ("|- a, a, a, ~a * (~a * ~a)", true),
            ("|- a, ~a, a, ~a", false),
        ] {
            assert_eq!(provable(input, m), expected, "{input:?}");
        }
        let mix = m.with_mix();
        for (input, expected) in [
            ("|-", false),
            ("|- a, ~a, a, ~a", true),
            ("|- a par b, ~a, ~b", true),
            ("|- a * b, ~a * ~b", false),
            ("|- a * ~a", false),
            ("|- (a par ~a) par (b par ~b)", true),
            ("|- a, a, ~a * ~a", true),
            ("|- a, a, ~a, ~a, b par c, ~b, ~c", true),
        ] {
            assert_eq!(provable(input, mix), expected, "{input:?} with Mix");
        }
    }

    /// A sequent whose atoms are all distinct is decided without any
    /// backtracking: one node and one link per atom, and one exact test
    /// per link on a small structure.
    #[test]
    fn distinct_atoms_need_no_backtracking() {
        let k = 40;
        let atoms: Vec<String> = (0..k).map(|i| format!("a{i}")).collect();
        let tensors = atoms.join(" * ");
        let pars: Vec<String> = atoms.iter().map(|a| format!("~{a}")).collect();
        let input = format!("|- {tensors}, {}", pars.join(" par "));
        let (verdict, statistics) = run(&input, Mode::CLASSICAL, &Options::default());
        assert!(verdict.proof().is_some());
        assert_eq!(statistics.nodes, k as u64);
        assert_eq!(statistics.links, k as u64);
        assert_eq!(statistics.tests, k as u64);
        // A larger structure is tested every fourth link and at the end.
        let (verdict, statistics) = run(
            &input,
            Mode::CLASSICAL,
            &Options::default().with_test_period(Some(7)),
        );
        assert!(verdict.proof().is_some());
        assert_eq!(statistics.links, k as u64);
        assert_eq!(statistics.tests, (k / 7 + 1) as u64);
        let long: Vec<String> = (0..120).map(|i| format!("b{i}")).collect();
        let input = format!(
            "|- {}, {}",
            long.join(" * "),
            long.iter()
                .map(|a| format!("~{a}"))
                .collect::<Vec<_>>()
                .join(" par ")
        );
        let (verdict, statistics) = run(&input, Mode::CLASSICAL, &Options::default());
        assert!(verdict.proof().is_some());
        assert_eq!(statistics.links, 120);
        assert_eq!(statistics.tests, 30);
    }

    /// Equal literal conclusions are linked in order: a refutation with
    /// repeated conclusions tries each set of partners once rather than in
    /// every order, and a provable sequent with repeated conclusions is
    /// still proved.
    #[test]
    fn symmetry_of_equal_conclusions() {
        // Four `a` conclusions, and a switching cycle through `b` and `c`
        // that only the exact test sees. With the test postponed to the
        // complete linking, the four `a` are paired before it fails: once
        // in order, not in all 24 orders.
        let input = "|- a, a, a, a, ((~a * ~a) * (~a * ~a)) * (b * c), \
                     (~b par d) * (~c par f), ~d * ~f";
        let postponed = Options::default().with_test_period(Some(100));
        let (verdict, statistics) = run(input, Mode::CLASSICAL, &postponed);
        assert!(matches!(verdict, Verdict::Unprovable(_)));
        assert_eq!(statistics.tests, 1);
        assert!(statistics.links < 40, "{statistics:?}");
        let (verdict, statistics) = run(input, Mode::CLASSICAL, &Options::default());
        assert!(matches!(verdict, Verdict::Unprovable(_)));
        assert_eq!(statistics.links, 2, "the cycle is seen at the second link");
        // An exact test that fails is a poll of its own, beside the
        // literals chosen: a run of failures chooses no literal.
        assert!(statistics.work > statistics.nodes, "{statistics:?}");
        let provable_twin = "|- a, a, a, a, ((~a * ~a) * (~a * ~a)) * (b * c), \
                             (~b par ~c) * (d * f), ~d, ~f";
        assert!(provable(provable_twin, Mode::CLASSICAL));
    }

    /// The stop condition ends the search with `Unknown` at the next node.
    #[test]
    fn stop() {
        let s: Sequent = "|- a * b, ~a, ~b".parse().unwrap();
        let forest = Forest::new(&s).unwrap();
        let mut polls = 0;
        let (options, limits) = (Options::default(), Limits::default());
        let answer = search(&forest, Mode::CLASSICAL, &options, &limits, &mut |_| {
            polls += 1;
            polls == 2
        });
        assert_eq!(answer.result.err(), Some(Reason::Stopped));
        assert!(answer.net.is_none());
        assert_eq!(answer.statistics.nodes, 2);
        assert_eq!(answer.statistics.links, 1);
    }

    /// Decides `input` with the focused engine.
    fn focus_verdict(s: &Sequent, mode: Mode) -> bool {
        let options = Options::default().with_engine(Some(Engine::Focus));
        match prove(s, mode, &options).unwrap().verdict {
            Verdict::Proved(proof) => {
                assert_eq!(proof.check(mode), Ok(()));
                true
            }
            Verdict::Unprovable(_) => false,
            Verdict::Unknown(reason) => panic!("{s}: the focused engine says {reason}"),
        }
    }

    /// Decides `input` with the net engine, checking as `run` does.
    fn net_verdict(text: &str, mode: Mode, options: &Options) -> bool {
        match run(text, mode, options).0 {
            Verdict::Proved(_) => true,
            Verdict::Unprovable(_) => false,
            Verdict::Unknown(reason) => panic!("{text}: {reason}"),
        }
    }

    /// The sample the two engines are compared on: generated provable
    /// sequents, mutants of them, doubled sequents (equal conclusions),
    /// and random sequents that pass the counts; with and without Mix.
    /// Returns the texts with their modes.
    pub(super) fn sample(samples: usize, budget: usize, pairs: usize) -> Vec<(String, Mode)> {
        let mut texts = Vec::new();
        for mix in [false, true] {
            let mode = mode_for(mix);
            let rules = Rules {
                units: false,
                additives: false,
                mix,
                exponentials: false,
            };
            let mut rng = Rng::new(u64::from(mix) + 10);
            for _ in 0..samples {
                let budget = 2 + rng.below(budget - 1);
                let mut formulas = generate::provable(&mut rng, rules, 3, budget).formulas;
                texts.push((generate::sequent(&formulas), mode));
                let doubled: Vec<_> = formulas.iter().chain(&formulas).cloned().collect();
                texts.push((generate::sequent(&doubled), mode));
                if generate::mutate(&mut rng, &mut formulas, 3) {
                    texts.push((generate::sequent(&formulas), mode));
                }
                let pairs = 1 + rng.below(pairs);
                let atoms = 1 + rng.below(3) as u8;
                let balanced = generate::balanced(&mut rng, atoms, pairs, mix);
                texts.push((generate::sequent(&balanced), mode));
            }
        }
        texts
    }

    /// Compares the two engines on a sample and returns how many sequents
    /// were decided and how many were provable.
    fn compare(sample: &[(String, Mode)], period: Option<u32>) -> (usize, usize) {
        let options = Options::default().with_test_period(period);
        let mut provable = 0;
        for (text, mode) in sample {
            let s: Sequent = text.parse().unwrap();
            let by_focus = focus_verdict(&s, *mode);
            let by_net = net_verdict(text, *mode, &options);
            assert_eq!(
                by_net, by_focus,
                "{text:?} in {mode} mode: net {by_net}, focus {by_focus}"
            );
            provable += usize::from(by_net);
        }
        (sample.len(), provable)
    }

    /// The net engine agrees with the focused engine on every sequent of
    /// the sample, at the default cadence of the exact test and when it
    /// runs only every third link.
    #[test]
    fn agrees_with_the_focused_engine() {
        let sample = sample(60, 12, 8);
        let (decided, provable) = compare(&sample, None);
        assert_eq!(decided, sample.len());
        assert!(
            provable > 100 && provable < decided - 100,
            "{provable} of {decided}"
        );
        compare(&sample, Some(3));
    }

    /// Whether some linking of the literals of `text` is a proof net, with
    /// or without Mix, by trying every pairing of each atom's `a` with its
    /// `~a` and the criterion on each complete linking; `None` when there
    /// are more than `limit` pairings.
    fn every_linking(text: &str, mix: bool, limit: u64) -> Option<bool> {
        let sequent: Sequent = text.parse().unwrap();
        let forest = Forest::new(&sequent).unwrap();
        let mut pairings = 1u64;
        let mut positives = Vec::new();
        for a in atoms(&forest) {
            let (p, q) = (
                forest.literals(a, Sign::Atom),
                forest.literals(a, Sign::Dual),
            );
            if p.len() != q.len() {
                return Some(false);
            }
            pairings = pairings.saturating_mul((1..=p.len() as u64).product());
            if pairings > limit {
                return None;
            }
            positives.extend_from_slice(p);
        }
        let mut net = ProofStructure::new(forest, Criterion { mix }).unwrap();
        Some(some_net(&mut net, &positives))
    }

    /// Whether linking the literals of `positives` to unlinked partners,
    /// in every way, completes `net` into a proof net.
    fn some_net(net: &mut ProofStructure, positives: &[OccId]) -> bool {
        let Some((&x, rest)) = positives.split_first() else {
            return net.is_correct(|_| false).is_ok();
        };
        let forest = net.forest().clone();
        for &y in forest.literals(forest.atom(x).unwrap(), Sign::Dual) {
            if net.partner(VertexId::of(y)).is_none() {
                net.link(VertexId::of(x), VertexId::of(y)).unwrap();
                let found = some_net(net, rest);
                net.unlink();
                if found {
                    return true;
                }
            }
        }
        false
    }

    /// The dual of a formula of literals, `⊗` and `⅋`.
    fn dual(formula: &Tree) -> Tree {
        match formula {
            Tree::Atom(a) => Tree::Dual(*a),
            Tree::Dual(a) => Tree::Atom(*a),
            Tree::Tensor(l, r) => Tree::Par(Box::new(dual(l)), Box::new(dual(r))),
            Tree::Par(l, r) => Tree::Tensor(Box::new(dual(l)), Box::new(dual(r))),
            other => unreachable!("{other:?} is not unit-free MLL"),
        }
    }

    /// Decides `text` on one thread with only the given prunes and returns
    /// the verdict and the literals chosen.
    fn decide_with(text: &str, mode: Mode, prunes: Prunes) -> (bool, u64) {
        let sequent: Sequent = text.parse().unwrap();
        let forest = Forest::new(&sequent).unwrap();
        if !counts_admit(&forest, mode.mix) {
            return (false, 0);
        }
        let options = Options::default();
        let mut never = |_| false;
        let mut linker =
            Linker::with_prunes(&forest, mode, &options, Stop::Closure(&mut never), prunes);
        (linker.run().unwrap(), linker.statistics.nodes)
    }

    /// The prunes switched off one at a time, and all of them.
    const WITHOUT_ONE: [Prunes; 4] = [
        Prunes {
            leaves: false,
            ..Prunes::ALL
        },
        Prunes {
            compounds: false,
            ..Prunes::ALL
        },
        Prunes {
            balance: false,
            ..Prunes::ALL
        },
        Prunes {
            leaves: false,
            compounds: false,
            balance: false,
        },
    ];

    /// The engine decides as the enumeration of every linking does, with
    /// every prune and with each switched off, and each prune cuts nodes:
    /// on random balanced sequents over one or two atoms, which put equal
    /// literals under one tree, on `F₁ ⊗ F₁, F₂, F₂, …` made of them, which
    /// keeps the counts and has equal compound conclusions and equal
    /// compound leaves of one `⊗`, and on `F, F, ~F ⊗ y, ~y ⊗ ~F` and
    /// `F ⊗ F, ~F ⊗ y, ~F, ~y` and their mutants, whose copies of `F` link
    /// into subformulas no exchange moves; with and without Mix.
    #[test]
    fn agrees_with_every_linking() {
        let mut rng = Rng::new(35);
        let (mut decided, mut proved) = (0, 0);
        let (mut nodes, mut without) = (0, [0; 4]);
        for round in 0..800 {
            let mix = round % 2 == 1;
            let atoms = 1 + rng.below(2) as u8;
            let pairs = 1 + rng.below(5);
            let formulas = generate::balanced(&mut rng, atoms, pairs, mix);
            let mut texts = vec![generate::sequent(&formulas)];
            let first = formulas[0].clone();
            let mut doubled = vec![Tree::Tensor(Box::new(first.clone()), Box::new(first))];
            for f in &formulas[1..] {
                doubled.extend([f.clone(), f.clone()]);
            }
            texts.push(generate::sequent(&doubled));
            // Two copies of a compound formula, consumed in different
            // trees, so that their keys are partners no exchange moves.
            let f = &formulas[0];
            if matches!(f, Tree::Tensor(..) | Tree::Par(..)) {
                let (y, not_y) = (Tree::Atom(7), Tree::Dual(7));
                let tensor =
                    |l: &Tree, r: &Tree| Tree::Tensor(Box::new(l.clone()), Box::new(r.clone()));
                let g = dual(f);
                for mut copies in [
                    vec![f.clone(), f.clone(), tensor(&g, &y), tensor(&not_y, &g)],
                    vec![tensor(f, f), tensor(&g, &y), g.clone(), not_y.clone()],
                ] {
                    texts.push(generate::sequent(&copies));
                    if generate::mutate(&mut rng, &mut copies, atoms) {
                        texts.push(generate::sequent(&copies));
                    }
                }
            }
            for text in texts {
                let Some(expected) = every_linking(&text, mix, 5_040) else {
                    continue;
                };
                let mode = mode_for(mix);
                let found = net_verdict(&text, mode, &Options::default());
                assert_eq!(found, expected, "{text:?} in {mode} mode");
                nodes += decide_with(&text, mode, Prunes::ALL).1;
                for (prunes, sum) in WITHOUT_ONE.iter().zip(&mut without) {
                    let (found, n) = decide_with(&text, mode, *prunes);
                    assert_eq!(found, expected, "{text:?} in {mode} mode, {prunes:?}");
                    *sum += n;
                }
                decided += 1;
                proved += usize::from(expected);
            }
        }
        assert!(
            decided > 1_000 && proved > 100 && decided - proved > 100,
            "{proved} of {decided}"
        );
        assert!(without.iter().all(|&n| n > nodes), "{nodes} {without:?}");
    }

    /// Small Partition instances are decided as the instance says, by both
    /// engines. The equal literals inside `b ⊗ b ⊗ …` and `~b ⅋ ~b` are
    /// interchangeable, which the symmetry breaks take out of the search,
    /// but which items go where is still found by cycles after the links,
    /// where the focused engine's counts see it at once.
    #[test]
    fn partition_instances() {
        for (sizes, expected) in [
            (&[1, 1][..], true),
            (&[1, 3], false),
            (&[2, 1, 1], true),
            (&[1, 1, 4], false),
        ] {
            let s = crate::families::partition(sizes);
            assert_eq!(
                provable(&s.to_string(), Mode::CLASSICAL),
                expected,
                "{sizes:?}"
            );
            assert_eq!(focus_verdict(&s, Mode::CLASSICAL), expected, "{sizes:?}");
        }
    }

    /// The engine is deterministic: the same input gives the same proof
    /// and the same counters.
    #[test]
    fn deterministic() {
        let input = "|- a, a, ~a * (b par ~b), ~a * (c * ~c) par d, ~d";
        let (first, stats) = run(input, Mode::CLASSICAL.with_mix(), &Options::default());
        let (second, again) = run(input, Mode::CLASSICAL.with_mix(), &Options::default());
        assert_eq!(stats, again);
        assert_eq!(
            first.proof().map(|p| p.nodes().to_vec()),
            second.proof().map(|p| p.nodes().to_vec())
        );
        assert_eq!(Engine::Net.to_string(), "net");
    }
}

/// The net engine on several threads: cube-and-conquer over the first
/// links.
#[cfg(feature = "parallel")]
pub(crate) mod parallel {
    use super::{Linker, counts_admit};
    use crate::fragment::Mode;
    use crate::limits::Limits;
    use crate::nets::{ProofStructure, VertexId};
    use crate::occurrences::Forest;
    use crate::search::parallel::{RaiseOnPanic, Runtime, record};
    use crate::search::{Answer, Options, Reason, Statistics, Stop, Work};
    use crate::search::{lock, taken};
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

    /// How many cubes per thread the splitting aims for, so that the
    /// cubes that turn out large are shared out among many small ones.
    const CUBES_PER_THREAD: usize = 16;

    /// A cube: the first links of a branch of the search, in the order
    /// they were made.
    type Cube = Vec<(VertexId, VertexId)>;

    /// What the workers of a cube-and-conquer run report.
    struct Collected {
        /// The first proof net found.
        net: Option<ProofStructure>,
        /// The first error, a stop that a cancellation caused excepted,
        /// a stop giving way to any other reason.
        error: Option<Reason>,
        /// The counters of every engine that ran.
        statistics: Statistics,
    }

    /// Runs the net engine on the runtime's pool, as [`super::search`]
    /// does on one thread, polling `stop` on the calling thread while the
    /// pool searches. The root engine splits the search into cubes: it
    /// starts from the one cube without a link and, pass by pass over the
    /// cubes in the order of the search, replaces every cube by the
    /// branches of its next choice, the links that choice forces
    /// included, until a pass leaves [`CUBES_PER_THREAD`] cubes per thread
    /// or the splitting decided the sequent by itself. A cube is never searched twice,
    /// where every link is forced the first cube's one branch is the
    /// whole sequential search, and the cubes stay in the order in which
    /// one thread would reach them. Then every worker takes cubes from a
    /// shared counter in that order, one engine of its own reset per
    /// cube, until a proof net is found, which stops the others, or the
    /// cubes run out. `Unprovable` needs every cube to have been searched
    /// to its end.
    pub(crate) fn search(
        forest: &Forest,
        mode: Mode,
        options: &Options,
        limits: &Limits,
        runtime: &Runtime,
        work: &Work,
        stop: &mut dyn FnMut(u64) -> bool,
    ) -> Answer {
        if !counts_admit(forest, mode.mix) {
            return super::answer(Ok(false), Statistics::default(), None, limits, stop);
        }
        let threads = runtime.threads();
        let (result, statistics, net) = runtime.drive(stop, work, |flags| {
            let mut root = Linker::new(forest, mode, options, Stop::flags(flags));
            // The cubes are what is left of the search at every moment,
            // each a branch nobody has followed yet, in the order of the
            // search: the branches of a cube take its place.
            let mut cubes = vec![Cube::new()];
            while cubes.len() < CUBES_PER_THREAD * threads {
                let mut split = Vec::new();
                for cube in &cubes {
                    root.reset();
                    root.seed(cube);
                    match root.explore(Some(1), &mut split) {
                        Ok(true) => return (Ok(true), root.statistics, Some(root.net)),
                        Ok(false) => {}
                        Err(reason) => return (Err(reason), root.statistics, None),
                    }
                }
                if split.is_empty() {
                    return (Ok(false), root.statistics, None);
                }
                cubes = split;
            }
            let next = AtomicUsize::new(0);
            let found = AtomicBool::new(false);
            let collected = Mutex::new(Collected {
                net: None,
                error: None,
                statistics: root.statistics,
            });
            runtime.pool.scope(|scope| {
                for _ in 0..threads {
                    let (cubes, next, found, collected, flags) =
                        (&cubes, &next, &found, &collected, &flags);
                    scope.spawn(move |_| {
                        // A worker that panics stops the others.
                        let _found = RaiseOnPanic(found);
                        let mut engine =
                            Linker::new(forest, mode, options, Stop::flags(flags.child(found)));
                        loop {
                            let i = next.fetch_add(1, Ordering::Relaxed);
                            let Some(cube) = cubes.get(i) else {
                                break;
                            };
                            engine.reset();
                            engine.seed(cube);
                            match engine.run() {
                                Ok(false) => continue,
                                Ok(true) => {
                                    let mut collected = lock(collected);
                                    if collected.net.is_none() {
                                        collected.net = Some(engine.net.clone());
                                    }
                                    found.store(true, Ordering::Relaxed);
                                }
                                Err(Reason::Stopped) if found.load(Ordering::Relaxed) => {}
                                Err(reason) => {
                                    record(&mut lock(collected).error, reason);
                                    found.store(true, Ordering::Relaxed);
                                }
                            }
                            break;
                        }
                        lock(collected).statistics.add_run(&engine.statistics);
                    });
                }
            });
            let collected = taken(collected);
            match (collected.net, collected.error) {
                (Some(net), _) => (Ok(true), collected.statistics, Some(net)),
                (None, Some(reason)) => (Err(reason), collected.statistics, None),
                (None, None) => (Ok(false), collected.statistics, None),
            }
        });
        super::answer(result, statistics, net, limits, stop)
    }
}

#[cfg(all(test, feature = "parse", feature = "parallel"))]
mod parallel_tests {
    use super::tests::sample;
    use crate::fragment::Mode;
    use crate::search::{Engine, Options, Reason, Verdict, prove, prove_within};
    use crate::sequents::Sequent;

    /// The verdict of the net engine on `jobs` threads, the proof and the
    /// net checked.
    fn verdict(text: &str, mode: Mode, jobs: usize) -> Verdict {
        let sequent: Sequent = text.parse().unwrap();
        let options = Options::default()
            .with_engine(Some(Engine::Net))
            .with_jobs(jobs);
        let outcome = prove(&sequent, mode, &options).unwrap();
        if let Verdict::Proved(proof) = &outcome.verdict {
            assert_eq!(proof.check(mode), Ok(()), "{text:?}");
            let net = outcome.net.as_ref().expect("the net found");
            assert_eq!(net.is_correct(|_| false), Ok(()), "{text:?}");
        }
        outcome.verdict
    }

    /// The net engine on two and four threads agrees with itself on one
    /// on generated sequents, provable and balanced ones, in both modes.
    #[test]
    fn cubes_agree_with_the_sequential_search() {
        for (text, mode) in sample(20, 6, 6) {
            let sequential = verdict(&text, mode, 1);
            for jobs in [2, 4] {
                let parallel = verdict(&text, mode, jobs);
                assert_eq!(
                    matches!(sequential, Verdict::Proved(_)),
                    matches!(parallel, Verdict::Proved(_)),
                    "{text:?} in {mode} mode: {sequential:?} on one thread, {parallel:?} on {jobs}"
                );
                assert_eq!(
                    matches!(sequential, Verdict::Unprovable(_)),
                    matches!(parallel, Verdict::Unprovable(_)),
                    "{text:?} in {mode} mode: {sequential:?} on one thread, {parallel:?} on {jobs}"
                );
            }
        }
    }

    /// Where every link is forced, the pool's splitting into cubes is the
    /// sequential search: `wide(64, 1)` is proved with the links and the
    /// literals chosen that one thread takes, none of them a second time.
    #[test]
    fn forced_links_are_made_once() {
        let sequent = crate::families::wide(64, 1);
        let options = Options::default().with_engine(Some(Engine::Net));
        let sequential = prove(&sequent, Mode::CLASSICAL, &options).unwrap();
        assert!(matches!(sequential.verdict, Verdict::Proved(_)));
        let parallel = prove(&sequent, Mode::CLASSICAL, &options.with_jobs(2)).unwrap();
        assert!(matches!(parallel.verdict, Verdict::Proved(_)));
        assert_eq!(parallel.statistics, sequential.statistics);
    }

    /// A stop condition that fires at once stops the pool: the driver
    /// raises the flag at its first poll, a millisecond in, long before
    /// the refutation of this Partition instance of nine items ends (more
    /// than a minute on one thread).
    #[test]
    fn stops() {
        let family = crate::families::find("partition-no").unwrap();
        let sequent = family.instance(9, 0).unwrap().sequent;
        let options = Options::default()
            .with_engine(Some(Engine::Net))
            .with_jobs(2);
        let outcome = prove_within(
            &sequent,
            Mode::CLASSICAL,
            &options,
            &crate::Limits::default(),
            |_| true,
        )
        .unwrap();
        assert!(
            matches!(outcome.verdict, Verdict::Unknown(Reason::Stopped)),
            "{:?}",
            outcome.verdict
        );
    }
}
