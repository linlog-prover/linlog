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
//! when the partners of equal literal conclusions stay in the order of the
//! conclusions, which breaks that symmetry. The exact acyclicity test runs
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
    let mut engine = Linker::new(forest, mode, options, Stop::Closure(stop));
    let result = engine.run();
    let statistics = engine.statistics;
    let net = matches!(result, Ok(true)).then_some(engine.net);
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
    /// Per literal that is a conclusion, the previous conclusion that is
    /// the same literal, or `NONE`.
    copy_before: Box<[u32]>,
    /// Per literal that is a conclusion, the next conclusion that is the
    /// same literal, or `NONE`.
    copy_after: Box<[u32]>,
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
        let net = ProofStructure::new(forest.clone(), Criterion { mix: mode.mix })
            .expect("the dispatch routes unit-free MLL only");
        let scratch = net.scratch();
        let remaining = atoms(forest)
            .map(|a| forest.literals(a, Sign::Atom).len() as u32)
            .collect();
        // Equal literal conclusions, chained in id order.
        let n = forest.len();
        let mut copy_before = vec![NONE; n];
        let mut copy_after = vec![NONE; n];
        for a in atoms(forest) {
            for sign in [Sign::Atom, Sign::Dual] {
                let mut previous = NONE;
                for &l in forest.literals(a, sign) {
                    if forest.parent(l).is_some() {
                        continue;
                    }
                    if previous != NONE {
                        copy_after[previous as usize] = l.get();
                        copy_before[l.index()] = previous;
                    }
                    previous = l.get();
                }
            }
        }
        let period = match options.test_period {
            Cadence::Every(period) => period.max(1),
            Cadence::Auto if n <= Cadence::AUTO_SMALL => 1,
            Cadence::Auto => Cadence::AUTO_PERIOD,
        };
        Self {
            scratch,
            remaining,
            copy_before: copy_before.into_boxed_slice(),
            copy_after: copy_after.into_boxed_slice(),
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
    /// end anywhere is found now rather than after more links.
    fn choose(&self) -> Choice {
        let forest = self.net.forest();
        let mut best: Option<(usize, OccId)> = None;
        for (a, &remaining) in self.remaining.iter().enumerate() {
            if remaining == 0 {
                continue;
            }
            let atom = Atom::new(a as u32);
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
    /// switching keeps), and the partners of equal literal conclusions
    /// stay in the order of the conclusions.
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
        self.ordered(x, y) && self.ordered(y, x)
    }

    /// Returns whether linking `x` to `y` keeps the partners of the
    /// conclusions that are the same literal as `x` in the order of those
    /// conclusions. Swapping two such conclusions maps proof nets to proof
    /// nets, so among the linkings that differ only by such swaps the one
    /// with ascending partners is enough to look for.
    fn ordered(&self, x: OccId, y: OccId) -> bool {
        let before = self.copy_before[x.index()];
        if before != NONE
            && let Some(p) = self.net.mate(VertexId::new(before))
            && p.occ() > y
        {
            return false;
        }
        let after = self.copy_after[x.index()];
        if after != NONE
            && let Some(q) = self.net.mate(VertexId::new(after))
            && q.occ() < y
        {
            return false;
        }
        true
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
    use crate::search::generate::{self, Rng, Rules};
    use crate::search::{Engine, Verdict, prove};
    use crate::sequents::Sequent;
    use std::time::{Duration, Instant};

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
    /// were decided, how many were provable, and the time each engine
    /// took.
    fn compare(
        sample: &[(String, Mode)],
        period: Option<u32>,
    ) -> (usize, usize, Duration, Duration) {
        let options = Options::default().with_test_period(period);
        let (mut provable, mut net_time, mut focus_time) = (0, Duration::ZERO, Duration::ZERO);
        for (text, mode) in sample {
            let s: Sequent = text.parse().unwrap();
            let start = Instant::now();
            let by_focus = focus_verdict(&s, *mode);
            focus_time += start.elapsed();
            let start = Instant::now();
            let by_net = net_verdict(text, *mode, &options);
            net_time += start.elapsed();
            assert_eq!(
                by_net, by_focus,
                "{text:?} in {mode} mode: net {by_net}, focus {by_focus}"
            );
            provable += usize::from(by_net);
        }
        (sample.len(), provable, net_time, focus_time)
    }

    /// The net engine agrees with the focused engine on every sequent of
    /// the sample, at the default cadence of the exact test and when it
    /// runs only every third link.
    #[test]
    fn agrees_with_the_focused_engine() {
        let sample = sample(60, 12, 8);
        let (decided, provable, _, _) = compare(&sample, None);
        assert_eq!(decided, sample.len());
        assert!(
            provable > 100 && provable < decided - 100,
            "{provable} of {decided}"
        );
        compare(&sample, Some(3));
    }

    /// The same on a larger sample, with the time each engine took; run it
    /// in release mode and read the numbers it prints.
    #[test]
    #[ignore = "a larger sample; run with --release -- --ignored --nocapture"]
    fn net_versus_focus_timing() {
        let sample = sample(150, 16, 10);
        let (decided, provable, net_time, focus_time) = compare(&sample, None);
        println!(
            "{decided} sequents decided alike, {provable} provable; net {net_time:.2?}, focus {focus_time:.2?}"
        );
        let (_, _, net_time, _) = compare(&sample, Some(4));
        println!("net with the exact test every fourth link: {net_time:.2?}");
    }

    /// Small Partition instances are decided as the instance says, by both
    /// engines. The equal literals inside `b ⊗ b ⊗ …` and `~b ⅋ ~b` are
    /// interchangeable, so the net engine visits a symmetric subtree for
    /// every wrong choice and falls far behind the focused engine here.
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
    use crate::search::parallel::{RaiseOnPanic, Runtime, lock, record, taken};
    use crate::search::{Answer, Options, Reason, Statistics, Stop, Work};
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
            let mut root = Linker::new(forest, mode, options, Stop::Flags(flags));
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
                            Linker::new(forest, mode, options, Stop::Flags(flags.child(found)));
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
    /// the refutation of this Partition instance ends (seconds on one
    /// thread, half a second on eight).
    #[test]
    fn stops() {
        let sequent = crate::families::partition(&[1, 2, 5]);
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
