// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Essential-net search for unit-free IMLL: the net engine's linking
//! search under the directed criterion of essential nets instead of the
//! switchings. The sequent's reading orients every edge
//! ([`ProofStructure::is_essential`] says how), and the search keeps the
//! reflexive transitive closure of the directed graph as a bit matrix,
//! row `u` the occurrences `u` reaches, which every link widens and an
//! undo log narrows again when the link is taken back.
//!
//! A candidate link from an input literal `x` to an output literal `y` is
//! rejected when `y` reaches `x` (the link would close a directed cycle,
//! which every completion keeps), and when `x` lies in the antecedent of
//! an implication `l` by input premises alone, `y` reaches the goal and
//! does not reach `l` (the antecedent would reach the goal around `l`
//! along the link, in every completion). At the cadence of the net
//! engine's exact test, and on every complete linking, the dominator
//! condition is tested for each implication `l` in output position but
//! the goal, with antecedent `s`: it fails when `s` does not reach `l` but
//! reaches the goal or an occurrence `l` reaches, or when, for some input
//! `⅋` `p` (the one vertex with two edges out) that `s` reaches and that
//! reaches `l`, a premise `q` of `p` does not reach `l` but reaches the
//! goal or an occurrence `l` reaches. Each such failure is a path from
//! `s` around `l` to an occurrence that can never reach `l` again without
//! a cycle, so no completion is correct; on a complete acyclic linking,
//! where every occurrence reaches the goal, it is exactly the dominator
//! condition (a path that leaves the occurrences reaching `l` leaves them
//! at an input `⅋`, or at `s`). A complete linking that passes is an
//! essential net, which for an intuitionistic sequent is a proof net, and
//! is sequentialized like the net engine's.

use super::memory::{Account, Charged, bytes_of};
use super::net::{self, Correctness, Linker, Prunes};
use super::{Answer, Decide, Engine, NotTaken, Options, Reason, Statistics, Stop, Task, Work};
use crate::Error;
use crate::fragment::Fragment;
use crate::limits::Limits;
use crate::nets::essential::{fixed_successors, is_implication};
use crate::nets::{Criterion, ProofStructure, VertexId};
use crate::occurrences::{Forest, OccId, Reading, Side, Sign};
use crate::sequents::{Atom, Kind};

/// The raw index that stands for "no occurrence".
const NONE: u32 = u32::MAX;

/// The bytes per occurrence of the criterion's tables beside the closure:
/// its partner and binder (four each), an implication with its
/// antecedent at most (eight), and its share of the links and their marks,
/// one each per two literals (four each); the bits of the branching
/// connectives come on top, a word per 64 occurrences.
const LINEAR_BYTES: usize = 24;

/// The essential-net engine as the front door calls it.
pub(crate) struct Essentials;

impl Decide for Essentials {
    /// Refuses classical and affine mode, a fragment beyond unit-free
    /// MLL, which in intuitionistic mode is unit-free IMLL, and a goal
    /// other than the roots: an essential net's conclusions are the
    /// forest's roots.
    fn admits(&self, task: &Task<'_>) -> Result<(), Error> {
        let refused = |because| Error::EngineRefused {
            engine: Engine::Essential,
            because,
        };
        let mode = task.mode;
        if !mode.is_intuitionistic() || mode.is_affine() {
            return Err(refused(NotTaken::Mode { mode }));
        }
        if !task.fragment.has_nets() {
            return Err(refused(NotTaken::Fragment {
                decides: Fragment::MLL,
                goal: task.fragment,
            }));
        }
        if !task.roots {
            return Err(refused(NotTaken::Goal));
        }
        Ok(())
    }

    /// Links the roots on the calling thread, whatever
    /// [`Options::jobs`] says.
    fn decide(
        &self,
        task: &Task<'_>,
        options: &Options,
        limits: &Limits,
        account: &Account,
        _work: &Work,
        stop: &mut dyn FnMut(u64) -> bool,
    ) -> Result<Answer, Error> {
        let reading = task
            .reading
            .expect("intuitionistic mode reads the sequent first");
        Ok(search(task.forest, reading, options, limits, account, stop))
    }
}

/// Runs the essential-net engine on the forest of a sequent of unit-free
/// IMLL read by `reading`, charging its closure to `account` and polling
/// `stop` once per literal chosen, and returns its answer, with the
/// proof net the proof was read off for a proved sequent.
pub(crate) fn search(
    forest: &Forest,
    reading: &Reading<'_>,
    options: &Options,
    limits: &Limits,
    account: &Account,
    stop: &mut dyn FnMut(u64) -> bool,
) -> Answer {
    // Intuitionistic mode has no Mix.
    if !net::counts_admit(forest, false) {
        return net::answer(Ok(false), Statistics::default(), None, limits, stop);
    }
    let (result, statistics, structure) = match Essential::new(forest, reading, account) {
        Err(reason) => (Err(reason), Statistics::default(), None),
        Ok(criterion) => {
            let mut engine = Linker::with(criterion, options, Stop::Closure(stop), Prunes::ALL);
            let result = engine.run();
            let structure = matches!(result, Ok(true)).then(|| engine.criterion.structure());
            (result, engine.statistics, structure)
        }
    };
    debug_assert!(
        structure
            .as_ref()
            .is_none_or(|net| net.is_essential(|_| false).is_ok()),
        "the search's test is the essential criterion"
    );
    net::answer(result, statistics, structure, limits, stop)
}

/// A change of the closure, which the undo log takes back: the row, the
/// word in it, and what the word held before.
#[derive(Clone, Copy, Debug)]
struct Change {
    /// The row: an occurrence, below the forest's length.
    row: u32,
    /// The word in the row, below the row's length in words.
    word: u32,
    /// The word's bits before the change.
    old: u64,
}

/// The essential criterion over a forest of unit-free IMLL: the links
/// made, the reflexive transitive closure of the essential net they make,
/// and its undo log.
pub(super) struct Essential<'a> {
    /// The forest linked.
    forest: &'a Forest,
    /// Per occurrence, the literal it is linked to, or `NONE`.
    partner: Box<[u32]>,
    /// The links made, in order, each as the search gave it.
    links: Vec<(VertexId, VertexId)>,
    /// The words of a row of the closure.
    words: usize,
    /// The closure, row by row: bit `v` of row `u` says that `u` reaches
    /// `v`. Every occurrence reaches itself.
    reach: Box<[u64]>,
    /// The words the links made have changed, in the order they changed.
    log: Vec<Change>,
    /// Per link made, the length of the log before it.
    marks: Vec<usize>,
    /// The bits of the input `⅋`s, the occurrences with two edges out, as
    /// a row.
    branching: Box<[u64]>,
    /// The implications in output position other than the goal, each with
    /// its antecedent.
    implications: Box<[(u32, u32)]>,
    /// Per occurrence in input position, the implication whose antecedent
    /// it lies in by input premises alone, or `NONE` for a hypothesis's
    /// or the goal's antecedent.
    binder: Box<[u32]>,
    /// The goal.
    goal: u32,
    /// What the closure and the log charge to the search's account.
    charged: Charged<'a>,
}

impl<'a> Essential<'a> {
    /// The criterion over a forest of unit-free IMLL without a link, its
    /// closure that of the edges every structure has, or
    /// [`Reason::MemoryLimit`] when the account cannot hold the closure,
    /// which is charged before it is allocated.
    fn new(
        forest: &'a Forest,
        reading: &Reading<'_>,
        account: &'a Account,
    ) -> Result<Self, Reason> {
        let n = forest.len();
        let words = n.div_ceil(64);
        // In `u64` first, since a matrix of more than 4 GB does not fit a
        // 32-bit `usize`: the forest has fewer than 2³² occurrences, so the
        // product is below 2⁵⁹, and once the whole fits `usize` so does
        // every row index `u · words + w` below it. Every step is checked,
        // and a charge that does not fit is the memory limit.
        let cells = n as u64 * words as u64;
        let held = usize::try_from(cells)
            .ok()
            .and_then(|cells| cells.checked_mul(size_of::<u64>()))
            .and_then(|matrix| matrix.checked_add(n.checked_mul(LINEAR_BYTES)?))
            .and_then(|bytes| bytes.checked_add(words * size_of::<u64>()))
            .filter(|&bytes| bytes <= isize::MAX as usize && account.fits(bytes))
            .ok_or(Reason::MemoryLimit {
                limit_bytes: account.limit(),
            })?;
        let mut charged = Charged::new(account);
        charged.charge(held);
        // `cells` fits `usize`, since `held` does.
        let mut reach = vec![0u64; cells as usize].into_boxed_slice();

        let position = |o: OccId| reading.position(o);
        let set = |row: &mut [u64], v: usize| row[v / 64] |= 1 << (v % 64);
        // The fixed edges go from an input connective to its input
        // premises, which come after it, and from an output occurrence to
        // its parent, which comes before it: the inputs from the last
        // up, then the outputs from the first down.
        for o in forest.ids().rev().filter(|&o| position(o) == Side::Input) {
            let (u, row) = (o.index(), o.index() * words);
            set(&mut reach[row..row + words], u);
            for s in fixed_successors(forest, reading, o) {
                let from = s.index() * words;
                for w in 0..words {
                    reach[row + w] |= reach[from + w];
                }
            }
        }
        for o in forest.ids().filter(|&o| position(o) == Side::Output) {
            let (u, row) = (o.index(), o.index() * words);
            set(&mut reach[row..row + words], u);
            for s in fixed_successors(forest, reading, o) {
                let from = s.index() * words;
                for w in 0..words {
                    reach[row + w] |= reach[from + w];
                }
            }
        }

        let goal = reading.goal();
        let mut branching = vec![0u64; words].into_boxed_slice();
        let mut implications = Vec::new();
        let mut binder = vec![NONE; n].into_boxed_slice();
        for o in forest.ids() {
            if position(o) == Side::Input {
                if forest.kind(o) == Kind::Par {
                    set(&mut branching, o.index());
                }
                binder[o.index()] = match forest.parent(o) {
                    // An input occurrence under an output one is the
                    // antecedent of an implication.
                    Some(p) if position(p) == Side::Output && p != goal => p.get(),
                    Some(p) if position(p) == Side::Input => binder[p.index()],
                    _ => NONE,
                };
            } else if is_implication(forest, reading, o) && o != goal {
                implications.push((o.get(), forest.left(o).unwrap().get()));
            }
        }
        Ok(Self {
            forest,
            partner: vec![NONE; n].into_boxed_slice(),
            links: Vec::with_capacity(forest.all_literals().len() / 2),
            words,
            reach,
            log: Vec::new(),
            marks: Vec::with_capacity(forest.all_literals().len() / 2),
            branching,
            implications: implications.into_boxed_slice(),
            binder,
            goal: goal.get(),
            charged,
        })
    }

    /// Returns whether `u` reaches `v`.
    fn reaches(&self, u: u32, v: u32) -> bool {
        let (u, v) = (u as usize, v as usize);
        self.reach[u * self.words + v / 64] >> (v % 64) & 1 == 1
    }

    /// Returns the row of `u`: the occurrences it reaches.
    fn row(&self, u: u32) -> &[u64] {
        let start = u as usize * self.words;
        &self.reach[start..start + self.words]
    }

    /// Returns whether `q`, which does not reach `l`, reaches the goal or
    /// an occurrence that `l` reaches: a path from `q` that never comes
    /// back to `l` in any acyclic completion, where it ends at the goal.
    fn escapes(&self, q: u32, l: u32) -> bool {
        self.reaches(q, self.goal) || self.row(q).iter().zip(self.row(l)).any(|(a, b)| a & b != 0)
    }

    /// Returns whether no path from the antecedent `s` of the implication
    /// `l` is known to pass around it: see the module's documentation.
    fn dominated(&self, l: u32, s: u32) -> bool {
        if !self.reaches(s, l) {
            return !self.escapes(s, l);
        }
        for (w, (&reached, &branching)) in self.row(s).iter().zip(&self.branching).enumerate() {
            let mut bits = reached & branching;
            while bits != 0 {
                let p = (w * 64) as u32 + bits.trailing_zeros();
                bits &= bits - 1;
                if !self.reaches(p, l) {
                    continue;
                }
                let p = OccId::new(p);
                for q in [self.forest.left(p), self.forest.right(p)] {
                    let q = q.expect("a `⅋` has two premises").get();
                    if !self.reaches(q, l) && self.escapes(q, l) {
                        return false;
                    }
                }
            }
        }
        true
    }

    /// Returns the link's literals as the edge runs: from the input
    /// literal `~a` to the output literal `a`.
    fn oriented(&self, x: OccId, y: OccId) -> (u32, u32) {
        match self.forest.sign(x) {
            Some(Sign::Dual) => (x.get(), y.get()),
            _ => (y.get(), x.get()),
        }
    }

    /// Records the log's growth past its capacity on the account, or
    /// refuses it with [`Reason::MemoryLimit`].
    fn reserve(&mut self) -> Result<(), Reason> {
        if self.log.len() < self.log.capacity() {
            return Ok(());
        }
        let before = bytes_of(&self.log);
        let more = self.log.capacity().max(1024);
        if !self
            .charged
            .account()
            .fits(more.saturating_mul(size_of::<Change>()))
        {
            return Err(Reason::MemoryLimit {
                limit_bytes: self.charged.account().limit(),
            });
        }
        self.log.reserve_exact(more);
        self.charged.resize(before, bytes_of(&self.log));
        Ok(())
    }

    /// Returns the proof structure of the links made.
    pub(super) fn structure(&self) -> ProofStructure {
        let mut net = ProofStructure::new(self.forest.clone(), Criterion::MLL)
            .expect("the engine admits unit-free IMLL only");
        for &(x, y) in &self.links {
            net.link_unchecked(x, y);
        }
        net
    }
}

impl Correctness for Essential<'_> {
    fn forest(&self) -> &Forest {
        self.forest
    }

    fn mate(&self, x: OccId) -> Option<OccId> {
        let p = self.partner[x.index()];
        (p != NONE).then(|| OccId::new(p))
    }

    fn links(&self) -> &[(VertexId, VertexId)] {
        &self.links
    }

    fn is_complete(&self) -> bool {
        2 * self.links.len() == self.forest.all_literals().len()
    }

    /// Rejects a link whose output literal reaches its input literal (a
    /// directed cycle), and one whose input literal lies in the
    /// antecedent of an implication by input premises alone while its
    /// output literal reaches the goal and not the implication.
    fn admits(&self, x: OccId, y: OccId) -> bool {
        let (i, o) = self.oriented(x, y);
        if self.reaches(o, i) {
            return false;
        }
        let l = self.binder[i as usize];
        l == NONE || self.reaches(o, l) || !self.reaches(o, self.goal)
    }

    /// Adds the edge from the input literal to the output literal, and to
    /// every row that reaches the input literal the output literal's row.
    /// No row reaches both ways, since the link closes no cycle, so the
    /// row read is never one written.
    fn link(&mut self, x: OccId, y: OccId) -> Result<(), Reason> {
        let (i, o) = self.oriented(x, y);
        debug_assert!(!self.reaches(o, i), "a link closes no cycle");
        let mark = self.log.len();
        let words = self.words;
        let (column, bit) = (i as usize / 64, i % 64);
        let from = o as usize * words;
        for u in 0..self.forest.len() {
            let row = u * words;
            if self.reach[row + column] >> bit & 1 == 0 {
                continue;
            }
            for w in 0..words {
                let (old, new) = (
                    self.reach[row + w],
                    self.reach[row + w] | self.reach[from + w],
                );
                if old != new {
                    if let Err(reason) = self.reserve() {
                        // Leave the structure as it was.
                        while self.log.len() > mark {
                            let change = self.log.pop().unwrap();
                            self.reach[change.row as usize * words + change.word as usize] =
                                change.old;
                        }
                        return Err(reason);
                    }
                    // `u` is below the forest's length and `w` below the
                    // words of a row, both below 2³².
                    self.log.push(Change {
                        row: u as u32,
                        word: w as u32,
                        old,
                    });
                    self.reach[row + w] = new;
                }
            }
        }
        self.marks.push(mark);
        self.partner[x.index()] = y.get();
        self.partner[y.index()] = x.get();
        self.links.push((VertexId::of(x), VertexId::of(y)));
        Ok(())
    }

    fn unlink(&mut self) -> OccId {
        let (x, y) = self.links.pop().expect("a link to take back");
        let mark = self.marks.pop().expect("a mark per link");
        while self.log.len() > mark {
            let change = self.log.pop().unwrap();
            self.reach[change.row as usize * self.words + change.word as usize] = change.old;
        }
        self.partner[x.index()] = NONE;
        self.partner[y.index()] = NONE;
        x.occ()
    }

    /// The dominator condition of every implication but the goal; the
    /// closure keeps the graph acyclic by itself.
    fn test(&mut self) -> bool {
        self.implications.iter().all(|&(l, s)| self.dominated(l, s))
    }

    /// The criterion keeps no skeleton.
    fn balanced(&self, _atom: Atom, _remaining: u32, _tally: &mut [u32]) -> bool {
        true
    }
}

#[cfg(all(test, feature = "parse"))]
mod tests {
    use super::Essential;
    use crate::fragment::Mode;
    use crate::limits::Limits;
    use crate::nets::{Criterion, ProofStructure};
    use crate::occurrences::{Forest, Reading};
    use crate::search::generate::{self, IllRules, Rng};
    use crate::search::memory::Account;
    use crate::search::net::Correctness;
    use crate::search::{Engine, Error, NotTaken, Options, Outcome, Reason, Verdict, prove};
    use crate::search::{Statistics, prove_within};
    use crate::sequents::Sequent;

    /// Unit-free IMLL.
    const IMLL: IllRules = IllRules {
        units: false,
        additives: false,
        zero: false,
        exponentials: false,
    };

    /// Parses `input`.
    fn sequent(input: &str) -> Sequent {
        input.parse().unwrap_or_else(|e| panic!("{input:?}: {e}"))
    }

    /// Options that force the engine.
    fn essential() -> Options {
        Options::default().with_engine(Some(Engine::Essential))
    }

    /// The verdict of an outcome as provable or not, after checking a
    /// proof against the checker and, for the essential engine, its net
    /// against the proof: the links the proof's axioms make are the net's.
    fn verdict(text: &str, outcome: Outcome) -> bool {
        let i = Mode::INTUITIONISTIC;
        match outcome.verdict {
            Verdict::Proved(proof) => {
                assert!(outcome.checked);
                assert_eq!(proof.check(i), Ok(()), "{text:?} by {}", outcome.engine);
                if outcome.engine == Engine::Essential {
                    let net = outcome.net.expect("the net found");
                    let read = ProofStructure::from_proof(
                        &proof,
                        Criterion::MLL,
                        &Limits::default(),
                        |_| false,
                    )
                    .unwrap();
                    let sorted = |net: &ProofStructure| {
                        let mut links: Vec<_> = net
                            .links()
                            .iter()
                            .map(|&(x, y)| (x.min(y), x.max(y)))
                            .collect();
                        links.sort_unstable();
                        links
                    };
                    assert_eq!(sorted(&read), sorted(&net), "{text:?}");
                    assert_eq!(net.is_essential(|_| false), Ok(()), "{text:?}");
                }
                true
            }
            Verdict::Unprovable(_) => false,
            Verdict::Unknown(reason) => panic!("{text:?} by {}: {reason}", outcome.engine),
        }
    }

    /// The most copies of one literal (atom and sign) in the sequent.
    fn most_copies(s: &Sequent) -> usize {
        let forest = Forest::new(s).unwrap();
        forest
            .all_literals()
            .chunk_by(|x, y| {
                forest.atom(*x) == forest.atom(*y) && forest.sign(*x) == forest.sign(*y)
            })
            .map(<[_]>::len)
            .max()
            .unwrap_or(0)
    }

    /// On generated intuitionistic sequents and their mutants with no
    /// literal more than three times, the essential-net engine gives the
    /// verdict of the two-sided engine, and of the net engine on the
    /// embedding where none is there more than twice, every
    /// proof passes the checker, and the net it returns is the net of
    /// its proof.
    #[test]
    fn agrees_with_the_two_sided_engine() {
        let i = Mode::INTUITIONISTIC;
        let mut rng = Rng::new(35);
        let (mut compared, mut provable, mut embedded) = (0, 0, 0);
        for _ in 0..300 {
            let budget = 2 + rng.below(10);
            let generate::Ill {
                mut hypotheses,
                mut goal,
                ..
            } = generate::ill(&mut rng, IMLL, 3, budget);
            for mutated in [false, true] {
                if mutated && !generate::mutate_two_sided(&mut rng, &mut hypotheses, &mut goal, 3) {
                    continue;
                }
                let text = generate::two_sided(&hypotheses, &goal);
                let s = sequent(&text);
                // Linking searches are exponential in a literal's copies:
                // the essential engine is asked where none has more than
                // three, the net engine where none has more than two, which
                // a debug build decides quickly.
                let copies = most_copies(&s);
                if copies > 3 {
                    continue;
                }
                let by_essential = prove(&s, i, &essential()).unwrap();
                assert_eq!(by_essential.engine, Engine::Essential);
                let two_sided = Options::default().with_engine(Some(Engine::TwoSided));
                let by_two_sided = prove(&s, i, &two_sided).unwrap();
                let (e, t) = (verdict(&text, by_essential), verdict(&text, by_two_sided));
                assert_eq!(e, t, "{text:?}: essential {e}, two-sided {t}");
                if copies <= 2 {
                    let net = Options::default().with_engine(Some(Engine::Net));
                    let n = verdict(&text, prove(&s, i, &net).unwrap());
                    assert_eq!(e, n, "{text:?}: essential {e}, net {n}");
                    embedded += 1;
                }
                assert!(mutated || e, "{text:?} is provable");
                compared += 1;
                provable += usize::from(e);
            }
        }
        assert!(
            compared > 300 && provable > 150 && provable < compared - 100 && embedded > 200,
            "{provable} of {compared}, {embedded} on the embedding"
        );
    }

    /// Taking links back restores the closure word for word, in stack
    /// order, after links that widen many rows.
    #[test]
    fn unlinking_restores_the_closure() {
        let s = sequent("a -o b, b -o c, c -o d, d -o e |- a -o e");
        let forest = Forest::new(&s).unwrap();
        let reading = Reading::new(&forest).unwrap();
        let account = Account::new(None);
        let mut criterion = Essential::new(&forest, &reading, &account).unwrap();
        let mut snapshots = vec![criterion.reach.clone()];
        let mut linked = 0;
        loop {
            // The first admissible pair, input literal first.
            let next = forest.all_literals().iter().find_map(|&x| {
                let free = |o| criterion.mate(o).is_none();
                (free(x) && forest.sign(x) == Some(crate::occurrences::Sign::Dual))
                    .then(|| {
                        forest
                            .literals(forest.atom(x).unwrap(), crate::occurrences::Sign::Atom)
                            .iter()
                            .copied()
                            .find(|&y| free(y) && criterion.admits(x, y))
                            .map(|y| (x, y))
                    })
                    .flatten()
            });
            let Some((x, y)) = next else { break };
            criterion.link(x, y).unwrap();
            linked += 1;
            snapshots.push(criterion.reach.clone());
        }
        assert!(criterion.is_complete() && linked == 5, "{linked} links");
        assert!(criterion.test(), "the composition is a net");
        assert!(criterion.log.len() > 10, "{} changes", criterion.log.len());
        while !criterion.links().is_empty() {
            snapshots.pop();
            criterion.unlink();
            assert_eq!(&criterion.reach, snapshots.last().unwrap());
        }
        assert!(criterion.log.is_empty() && criterion.marks.is_empty());
    }

    /// The closure is charged before it is allocated: under a bound it
    /// does not fit, the engine answers unknown at the memory limit, and
    /// a bound that holds it decides.
    #[test]
    fn the_closure_is_charged() {
        let s = sequent(&format!(
            "a0{} |- a40",
            (0..40)
                .map(|k| format!(", a{k} -o a{}", k + 1))
                .collect::<String>()
        ));
        let i = Mode::INTUITIONISTIC;
        let within = |bytes| {
            let limits = Limits {
                memory_bytes: Some(bytes),
                ..Limits::default()
            };
            prove_within(&s, i, &essential(), &limits, |_| false).unwrap()
        };
        // 162 occurrences: 162 rows of three words.
        let outcome = within(2_000);
        assert!(
            matches!(
                outcome.verdict,
                Verdict::Unknown(Reason::MemoryLimit { limit_bytes: 2_000 })
            ),
            "{:?}",
            outcome.verdict
        );
        assert_eq!(outcome.statistics, Statistics::default());
        assert!(matches!(within(1 << 20).verdict, Verdict::Proved(_)));
    }

    /// The engine refuses what has no essential net: classical and affine
    /// mode, units, and a goal other than the roots.
    #[test]
    fn refusals() {
        let refused = |text: &str, mode: Mode| match prove(&sequent(text), mode, &essential()) {
            Err(Error::EngineRefused {
                engine: Engine::Essential,
                because,
            }) => because,
            other => panic!("{text:?}: {other:?}"),
        };
        assert!(matches!(
            refused("|- a, ~a", Mode::CLASSICAL),
            NotTaken::Mode { .. }
        ));
        let affine = Mode::INTUITIONISTIC.with_affine();
        assert!(matches!(refused("a |- a", affine), NotTaken::Mode { .. }));
        let units = refused("a * 1 |- a", Mode::INTUITIONISTIC);
        assert_eq!(
            units.explained(Engine::Essential).to_string(),
            "essential nets exist for IMLL without units only, not for MLL with units"
        );
    }
}
