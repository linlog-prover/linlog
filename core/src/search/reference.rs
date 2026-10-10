// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! A reference prover for the tests: it decides a small sequent by the
//! plain rules of the unfocused sequent calculus, trying every rule on
//! every formula, with nothing the engines rely on (no polarity, no focus,
//! no invertible phase, no count prune, no interchangeable occurrences,
//! no occurrence ids). The classical calculus is one-sided, the
//! intuitionistic one two-sided, each linear or affine, the classical one
//! with or without Mix. Contraction is the one rule that makes a sequent
//! larger, so it is bounded: each branch may contract at most `copies`
//! times, and a search that wanted one more says so, which keeps it from
//! answering "unprovable". It shares no code with the engines: formulas
//! are its own, read from the generator's trees or from a sequent's terms.
//!
//! Every rule but contraction makes the sequent smaller, and contraction
//! spends the budget, so the search ends; a memo keyed by the sequent (a
//! sorted multiset of formulas, and the goal two-sided) holds what each
//! sequent was found to be at which budget. The budget deepens from zero,
//! so that what the reference claims at every budget below the one it is
//! given is answered too.
//!
//! "Within the budget" counts contractions on a branch, which is one less
//! than the engines' copies of a formula (every use of it), so the two
//! bounds are not compared: only a proof and a refutation are. And the
//! reference cuts every failing sequent with a `?` member (one-sided) or a
//! `!` hypothesis (two-sided), at every budget, since it contracts down
//! to an empty budget there; so with exponentials it refutes only where
//! the failure lies on a premise without one (`⊢ 0 ⊗ ?a`), and answers
//! any of the three verdicts, each sound. It judges an engine's
//! "unprovable"; an engine's proof is the checker's to judge.

use super::generate::Tree;
use crate::Sequent;
use crate::fragment::Mode;
use crate::sequents::{Term, TermId};
use std::collections::HashMap;

/// What the reference found.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Answer {
    /// A proof exists within the budget of contractions per branch.
    Proved,
    /// No proof exists: no branch of the search wanted more contractions
    /// than the budget, and it did not give up.
    Unprovable,
    /// No proof within the budget, and some branch wanted a contraction
    /// more, or the search gave up after its bound on visits.
    Unknown,
}

/// The most sequents one decision computes, and context splits it tries,
/// before it gives up, so that no test runs away on a sequent too large
/// for an exhaustive search: a split of `n` formulas has `2ⁿ` ways, and
/// under Mix every sequent searched has as many.
pub(crate) const VISITS: usize = 1_000_000;

/// Decides the one-sided classical sequent of `formulas` (atoms
/// `Tree::Atom`, their negations `Tree::Dual`, no `Tree::Lolli`) under the
/// mode (its `affine` and `mix`), with at most `copies` contractions per
/// branch.
pub(crate) fn classical(formulas: &[Tree], mode: Mode, copies: u32) -> Answer {
    assert!(!mode.intuitionistic, "a one-sided sequent is classical");
    let mut prover = Prover::new(mode);
    let mut sequent: Vec<u32> = formulas.iter().map(|t| prover.tree(t)).collect();
    sequent.sort_unstable();
    prover.deepen(copies, |prover, budget| prover.one_sided(&sequent, budget))
}

/// Decides the two-sided intuitionistic sequent `hypotheses ⊢ goal`
/// (atoms, `1`, `⊤`, `0`, `⊗`, `⊸`, `&`, `⊕` and `!`) under the mode (its
/// `affine`), with at most `copies` contractions per branch.
pub(crate) fn intuitionistic(hypotheses: &[Tree], goal: &Tree, mode: Mode, copies: u32) -> Answer {
    assert!(
        mode.intuitionistic && !mode.mix,
        "a two-sided sequent is intuitionistic, without Mix"
    );
    let mut prover = Prover::new(mode);
    let mut left: Vec<u32> = hypotheses.iter().map(|t| prover.tree(t)).collect();
    left.sort_unstable();
    let goal = prover.tree(goal);
    prover.deepen(copies, |prover, budget| {
        prover.two_sided(&left, goal, budget)
    })
}

/// Decides a parsed sequent one-sided, as [`classical`] does.
pub(crate) fn sequent(sequent: &Sequent, mode: Mode, copies: u32) -> Answer {
    assert!(!mode.intuitionistic, "a one-sided sequent is classical");
    let mut prover = Prover::new(mode);
    let mut ids = HashMap::new();
    let mut formulas: Vec<u32> = sequent
        .roots()
        .iter()
        .map(|&root| prover.term(sequent, root, &mut ids))
        .collect();
    formulas.sort_unstable();
    prover.deepen(copies, |prover, budget| prover.one_sided(&formulas, budget))
}

/// A formula of the reference's own table, its subformulas by index.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum F {
    /// An atom.
    Atom(u32),
    /// A negated atom.
    Dual(u32),
    /// `1`
    One,
    /// `⊥`
    Bot,
    /// `⊤`
    Top,
    /// `0`
    Zero,
    /// `A ⊗ B`
    Tensor(u32, u32),
    /// `A ⅋ B`
    Par(u32, u32),
    /// `A & B`
    With(u32, u32),
    /// `A ⊕ B`
    Plus(u32, u32),
    /// `!A`
    Bang(u32),
    /// `?A`
    Quest(u32),
    /// `A ⊸ B`, two-sided only.
    Lolli(u32, u32),
}

/// What a search of one sequent at one budget found.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Found {
    /// A proof.
    Proved,
    /// No proof, and `cut` when some branch wanted a contraction beyond
    /// its budget (or the search gave up), so that more budget might find
    /// one.
    Failed {
        /// Whether the failure may be the budget's.
        cut: bool,
    },
}

/// What the memo knows of a sequent, at every budget at once.
#[derive(Clone, Copy, Debug, Default)]
struct Known {
    /// The least budget a proof was found at: one is found at any larger.
    proved: Option<u32>,
    /// Whether a search failed without a cut: no budget proves it.
    refuted: bool,
    /// The largest budget a search failed at with a cut: no proof at that
    /// budget or less, unless the search gave up, after which the
    /// reference answers nothing but a proof.
    cut: Option<u32>,
}

impl Known {
    /// What the memo answers for the budget, if it knows.
    fn at(&self, budget: u32) -> Option<Found> {
        if self.proved.is_some_and(|b| b <= budget) {
            Some(Found::Proved)
        } else if self.refuted {
            Some(Found::Failed { cut: false })
        } else if self.cut.is_some_and(|b| b >= budget) {
            Some(Found::Failed { cut: true })
        } else {
            None
        }
    }

    /// Records what a search at the budget found.
    fn record(&mut self, budget: u32, found: Found) {
        match found {
            Found::Proved => self.proved = Some(self.proved.map_or(budget, |b| b.min(budget))),
            Found::Failed { cut: false } => self.refuted = true,
            Found::Failed { cut: true } => {
                self.cut = Some(self.cut.map_or(budget, |b| b.max(budget)))
            }
        }
    }
}

/// One decision: the formulas, the mode's rules, the memos and the count
/// of sequents computed.
struct Prover {
    /// The formulas, each once.
    formulas: Vec<F>,
    /// Each formula's index.
    index: HashMap<F, u32>,
    /// Weakening of any formula (two-sided: of any hypothesis).
    affine: bool,
    /// The Mix rule.
    mix: bool,
    /// What is known of one-sided sequents.
    one_sided: HashMap<Vec<u32>, Known>,
    /// What is known of two-sided sequents, the goal last.
    two_sided: HashMap<(Vec<u32>, u32), Known>,
    /// The sequents computed (memo hits not counted).
    visits: usize,
    /// Whether the search gave up at [`VISITS`].
    gave_up: bool,
}

impl Prover {
    /// A prover for the mode.
    fn new(mode: Mode) -> Self {
        Self {
            formulas: Vec::new(),
            index: HashMap::new(),
            affine: mode.affine,
            mix: mode.mix,
            one_sided: HashMap::new(),
            two_sided: HashMap::new(),
            visits: 0,
            gave_up: false,
        }
    }

    /// Decides the whole sequent at the budgets from 0 up to `copies`,
    /// one after the other with the memo kept, until one proves it or
    /// fails without a cut: so every budget's answer is the one given,
    /// and a wrong refutation at a small budget is not hidden behind a
    /// proof at a larger one.
    fn deepen(&mut self, copies: u32, mut search: impl FnMut(&mut Self, u32) -> Found) -> Answer {
        for budget in 0..=copies {
            if self.gave_up {
                break;
            }
            match search(self, budget) {
                Found::Proved => return Answer::Proved,
                Found::Failed { cut: false } if !self.gave_up => return Answer::Unprovable,
                Found::Failed { .. } => {}
            }
        }
        Answer::Unknown
    }

    /// The index of a formula, added to the table if new.
    fn add(&mut self, f: F) -> u32 {
        if let Some(&i) = self.index.get(&f) {
            return i;
        }
        let i = self.formulas.len() as u32;
        self.formulas.push(f);
        self.index.insert(f, i);
        i
    }

    /// The index of a generator's tree.
    fn tree(&mut self, t: &Tree) -> u32 {
        let f = match t {
            Tree::Atom(a) => F::Atom(u32::from(*a)),
            Tree::Dual(a) => F::Dual(u32::from(*a)),
            Tree::One => F::One,
            Tree::Bot => F::Bot,
            Tree::Top => F::Top,
            Tree::Zero => F::Zero,
            Tree::Tensor(l, r) => F::Tensor(self.tree(l), self.tree(r)),
            Tree::Par(l, r) => F::Par(self.tree(l), self.tree(r)),
            Tree::With(l, r) => F::With(self.tree(l), self.tree(r)),
            Tree::Plus(l, r) => F::Plus(self.tree(l), self.tree(r)),
            Tree::Bang(a) => F::Bang(self.tree(a)),
            Tree::Quest(a) => F::Quest(self.tree(a)),
            Tree::Lolli(l, r) => F::Lolli(self.tree(l), self.tree(r)),
        };
        self.add(f)
    }

    /// The index of a term of a sequent; `ids` remembers the terms read.
    fn term(&mut self, sequent: &Sequent, id: TermId, ids: &mut HashMap<TermId, u32>) -> u32 {
        if let Some(&i) = ids.get(&id) {
            return i;
        }
        let f = match sequent.term(id) {
            Term::Atom(a) => F::Atom(a.index() as u32),
            Term::DualAtom(a) => F::Dual(a.index() as u32),
            Term::One => F::One,
            Term::Bot => F::Bot,
            Term::Top => F::Top,
            Term::Zero => F::Zero,
            Term::Tensor(l, r) => F::Tensor(self.term(sequent, l, ids), self.term(sequent, r, ids)),
            Term::Par(l, r) => F::Par(self.term(sequent, l, ids), self.term(sequent, r, ids)),
            Term::With(l, r) => F::With(self.term(sequent, l, ids), self.term(sequent, r, ids)),
            Term::Plus(l, r) => F::Plus(self.term(sequent, l, ids), self.term(sequent, r, ids)),
            Term::Bang(a) => F::Bang(self.term(sequent, a, ids)),
            Term::Quest(a) => F::Quest(self.term(sequent, a, ids)),
        };
        let i = self.add(f);
        ids.insert(id, i);
        i
    }

    /// Counts a sequent computed, and says whether the search must give
    /// up instead.
    fn visit(&mut self) -> bool {
        self.visits += 1;
        if self.visits > VISITS {
            self.gave_up = true;
        }
        self.gave_up
    }

    // The one-sided calculus.

    /// Decides `⊢ sequent` (sorted) with `budget` contractions left on
    /// this branch.
    fn one_sided(&mut self, sequent: &[u32], budget: u32) -> Found {
        if let Some(found) = self.one_sided.get(sequent).and_then(|k| k.at(budget)) {
            return found;
        }
        if self.visit() {
            return Found::Failed { cut: true };
        }
        let found = self.one_sided_rules(sequent, budget);
        self.one_sided
            .entry(sequent.to_vec())
            .or_default()
            .record(budget, found);
        found
    }

    /// Every rule on every formula of `⊢ sequent`, and Mix: proved when
    /// one application has all its premises proved.
    fn one_sided_rules(&mut self, sequent: &[u32], budget: u32) -> Found {
        let mut cut = false;
        // Folds a failed application into the sequent's failure; true
        // when the application proved it.
        let mut tried = |found: Found| match found {
            Found::Proved => true,
            Found::Failed { cut: c } => {
                cut |= c;
                false
            }
        };
        for (i, &f) in sequent.iter().enumerate() {
            // Equal formulas give equal premises: the first of a run will do.
            if i > 0 && sequent[i - 1] == f {
                continue;
            }
            let rest = without(sequent, i);
            if self.affine && tried(self.one_sided(&rest, budget)) {
                return Found::Proved;
            }
            let found = match self.formulas[f as usize] {
                F::Atom(a) => {
                    let dual = self.index.get(&F::Dual(a)).copied();
                    if rest.len() == 1 && Some(rest[0]) == dual {
                        return Found::Proved;
                    }
                    continue;
                }
                F::Dual(_) | F::Zero => continue,
                F::One if rest.is_empty() => return Found::Proved,
                F::One => continue,
                F::Top => return Found::Proved,
                F::Bot => self.one_sided(&rest, budget),
                F::Par(a, b) => self.one_sided(&with(&rest, &[a, b]), budget),
                F::Plus(a, b) => {
                    if tried(self.one_sided(&with(&rest, &[a]), budget)) {
                        return Found::Proved;
                    }
                    self.one_sided(&with(&rest, &[b]), budget)
                }
                F::With(a, b) => {
                    self.both_one_sided(&with(&rest, &[a]), &with(&rest, &[b]), budget)
                }
                F::Tensor(a, b) => {
                    let mut found = Found::Failed { cut: false };
                    for (left, right) in splits(&rest) {
                        if self.visit() {
                            return Found::Failed { cut: true };
                        }
                        let attempt =
                            self.both_one_sided(&with(&left, &[a]), &with(&right, &[b]), budget);
                        if tried(attempt) {
                            return Found::Proved;
                        }
                        found = attempt;
                    }
                    found
                }
                F::Bang(a) => {
                    let all_quest = rest
                        .iter()
                        .all(|&g| matches!(self.formulas[g as usize], F::Quest(_)));
                    if !all_quest {
                        continue;
                    }
                    self.one_sided(&with(&rest, &[a]), budget)
                }
                F::Quest(a) => {
                    // Dereliction, weakening, and contraction at a unit of
                    // the budget.
                    if tried(self.one_sided(&with(&rest, &[a]), budget))
                        || tried(self.one_sided(&rest, budget))
                    {
                        return Found::Proved;
                    }
                    if budget == 0 {
                        Found::Failed { cut: true }
                    } else {
                        self.one_sided(&with(sequent, &[f]), budget - 1)
                    }
                }
                F::Lolli(..) => unreachable!("no ⊸ in a one-sided sequent"),
            };
            if tried(found) {
                return Found::Proved;
            }
        }
        if self.mix {
            for (left, right) in splits(sequent) {
                if self.visit() {
                    return Found::Failed { cut: true };
                }
                if left.is_empty() || right.is_empty() {
                    continue;
                }
                if tried(self.both_one_sided(&left, &right, budget)) {
                    return Found::Proved;
                }
            }
        }
        Found::Failed { cut }
    }

    /// Both premises of a rule, each with the branch's whole budget: the
    /// failure of the first that fails, or a proof.
    fn both_one_sided(&mut self, first: &[u32], second: &[u32], budget: u32) -> Found {
        match self.one_sided(first, budget) {
            Found::Proved => self.one_sided(second, budget),
            failed => failed,
        }
    }

    // The two-sided calculus.

    /// Decides `hypotheses ⊢ goal` (the hypotheses sorted) with `budget`
    /// contractions left on this branch.
    fn two_sided(&mut self, hypotheses: &[u32], goal: u32, budget: u32) -> Found {
        let key = (hypotheses.to_vec(), goal);
        if let Some(found) = self.two_sided.get(&key).and_then(|k| k.at(budget)) {
            return found;
        }
        if self.visit() {
            return Found::Failed { cut: true };
        }
        let found = self.two_sided_rules(hypotheses, goal, budget);
        self.two_sided.entry(key).or_default().record(budget, found);
        found
    }

    /// Every right rule on the goal, every left rule on every hypothesis:
    /// proved when one application has all its premises proved.
    fn two_sided_rules(&mut self, hypotheses: &[u32], goal: u32, budget: u32) -> Found {
        let mut cut = false;
        let mut tried = |found: Found| match found {
            Found::Proved => true,
            Found::Failed { cut: c } => {
                cut |= c;
                false
            }
        };
        // The identity on an atom, and the right rules.
        let found = match self.formulas[goal as usize] {
            F::Atom(_) => {
                if hypotheses == [goal] {
                    return Found::Proved;
                }
                Found::Failed { cut: false }
            }
            F::One if hypotheses.is_empty() => return Found::Proved,
            F::One | F::Zero => Found::Failed { cut: false },
            F::Top => return Found::Proved,
            F::Tensor(a, b) => {
                let mut found = Found::Failed { cut: false };
                for (left, right) in splits(hypotheses) {
                    if self.visit() {
                        return Found::Failed { cut: true };
                    }
                    let attempt = self.both_two_sided((&left, a), (&right, b), budget);
                    if tried(attempt) {
                        return Found::Proved;
                    }
                    found = attempt;
                }
                found
            }
            F::Lolli(a, b) => self.two_sided(&with(hypotheses, &[a]), b, budget),
            F::With(a, b) => self.both_two_sided((hypotheses, a), (hypotheses, b), budget),
            F::Plus(a, b) => {
                if tried(self.two_sided(hypotheses, a, budget)) {
                    return Found::Proved;
                }
                self.two_sided(hypotheses, b, budget)
            }
            F::Bang(a) => {
                let all_bang = hypotheses
                    .iter()
                    .all(|&h| matches!(self.formulas[h as usize], F::Bang(_)));
                if all_bang {
                    self.two_sided(hypotheses, a, budget)
                } else {
                    Found::Failed { cut: false }
                }
            }
            f => unreachable!("{f:?} is no intuitionistic goal"),
        };
        if tried(found) {
            return Found::Proved;
        }
        // The left rules.
        for (i, &h) in hypotheses.iter().enumerate() {
            if i > 0 && hypotheses[i - 1] == h {
                continue;
            }
            let rest = without(hypotheses, i);
            if self.affine && tried(self.two_sided(&rest, goal, budget)) {
                return Found::Proved;
            }
            let found = match self.formulas[h as usize] {
                F::Atom(_) | F::Top => continue,
                F::Zero => return Found::Proved,
                F::One => self.two_sided(&rest, goal, budget),
                F::Tensor(a, b) => self.two_sided(&with(&rest, &[a, b]), goal, budget),
                F::With(a, b) => {
                    if tried(self.two_sided(&with(&rest, &[a]), goal, budget)) {
                        return Found::Proved;
                    }
                    self.two_sided(&with(&rest, &[b]), goal, budget)
                }
                F::Plus(a, b) => self.both_two_sided(
                    (&with(&rest, &[a]), goal),
                    (&with(&rest, &[b]), goal),
                    budget,
                ),
                F::Lolli(a, b) => {
                    let mut found = Found::Failed { cut: false };
                    for (left, right) in splits(&rest) {
                        if self.visit() {
                            return Found::Failed { cut: true };
                        }
                        let attempt =
                            self.both_two_sided((&left, a), (&with(&right, &[b]), goal), budget);
                        if tried(attempt) {
                            return Found::Proved;
                        }
                        found = attempt;
                    }
                    found
                }
                F::Bang(a) => {
                    // Dereliction, weakening, and contraction at a unit of
                    // the budget.
                    if tried(self.two_sided(&with(&rest, &[a]), goal, budget))
                        || tried(self.two_sided(&rest, goal, budget))
                    {
                        return Found::Proved;
                    }
                    if budget == 0 {
                        Found::Failed { cut: true }
                    } else {
                        self.two_sided(&with(hypotheses, &[h]), goal, budget - 1)
                    }
                }
                f => unreachable!("{f:?} is no intuitionistic hypothesis"),
            };
            if tried(found) {
                return Found::Proved;
            }
        }
        Found::Failed { cut }
    }

    /// Both premises of a two-sided rule, each a pair of hypotheses and a
    /// goal, each with the branch's whole budget.
    fn both_two_sided(
        &mut self,
        first: (&[u32], u32),
        second: (&[u32], u32),
        budget: u32,
    ) -> Found {
        match self.two_sided(first.0, first.1, budget) {
            Found::Proved => self.two_sided(second.0, second.1, budget),
            failed => failed,
        }
    }
}

/// The multiset without its element at `i`.
fn without(sequent: &[u32], i: usize) -> Vec<u32> {
    let mut rest = sequent.to_vec();
    rest.remove(i);
    rest
}

/// The multiset with the formulas added, sorted.
fn with(sequent: &[u32], added: &[u32]) -> Vec<u32> {
    let mut larger = sequent.to_vec();
    larger.extend_from_slice(added);
    larger.sort_unstable();
    larger
}

/// Every way to split a sorted multiset into two, each as a sorted
/// multiset, one after the other: of every run of equal formulas, every
/// number of them on the left.
fn splits(sequent: &[u32]) -> Splits {
    let mut runs: Vec<(u32, usize)> = Vec::new();
    for &f in sequent {
        match runs.last_mut() {
            Some((g, n)) if *g == f => *n += 1,
            _ => runs.push((f, 1)),
        }
    }
    Splits {
        taken: vec![0; runs.len()],
        runs,
        done: false,
    }
}

/// The splits of a multiset not given out yet, as an odometer over how
/// many of each run of equal formulas go left.
struct Splits {
    /// The runs of equal formulas, each with its length.
    runs: Vec<(u32, usize)>,
    /// How many of each run the next split puts on the left.
    taken: Vec<usize>,
    /// Whether every split was given out.
    done: bool,
}

impl Iterator for Splits {
    type Item = (Vec<u32>, Vec<u32>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        let (mut left, mut right) = (Vec::new(), Vec::new());
        for (&(f, n), &k) in self.runs.iter().zip(&self.taken) {
            left.extend(std::iter::repeat_n(f, k));
            right.extend(std::iter::repeat_n(f, n - k));
        }
        // The next count: the first run not full goes up by one, those
        // before it back to none; past the last, every split was given.
        self.done = true;
        for (k, &(_, n)) in self.taken.iter_mut().zip(&self.runs) {
            if *k < n {
                *k += 1;
                self.done = false;
                break;
            }
            *k = 0;
        }
        Some((left, right))
    }
}

#[cfg(feature = "parse")]
mod tests {
    use super::*;
    use crate::Error;
    use crate::families::FAMILIES;
    use crate::search::Bias;
    use crate::search::generate::{self, IllRules, Rng, Rules};
    use crate::search::{Engine, Options, Verdict, prove_within};

    /// The configurations of the engines that the reference judges: the
    /// dispatch, each engine forced, and the focused engines under each
    /// bias; with the feature `parallel` also the dispatch on two
    /// threads. An engine that refuses a sequent is skipped on it.
    fn configurations(copies: u32) -> Vec<(String, Options)> {
        let base = Options::default().with_copies(Some(copies));
        let mut all = vec![("dispatch".to_owned(), base.clone())];
        for engine in [Engine::Net, Engine::Additive] {
            all.push((format!("{engine}"), base.clone().with_engine(Some(engine))));
        }
        all.push((
            "horn".to_owned(),
            base.clone().with_engine(Some(Engine::Horn)),
        ));
        for engine in [Engine::Focus, Engine::TwoSided] {
            for bias in [Bias::Auto, Bias::Rarer, Bias::Factors] {
                all.push((
                    format!("{engine} {bias:?}"),
                    base.clone().with_engine(Some(engine)).with_bias(bias),
                ));
            }
        }
        #[cfg(feature = "parallel")]
        all.push((
            "dispatch on two threads".to_owned(),
            base.clone().with_jobs(2),
        ));
        all
    }

    /// What the cases of one comparison came to.
    #[derive(Default)]
    struct Tally {
        /// The sequents compared.
        cases: usize,
        /// Those the reference decided.
        decided: usize,
        /// The engines' runs that answered beside a reference that decided.
        judged: usize,
    }

    /// Runs every configuration on `text` under `mode` and asserts what
    /// the contract allows: no proof where the reference refutes, no
    /// refutation where the reference proves. Every proof of the roots is
    /// checked by `prove` itself.
    fn judge(text: &str, mode: Mode, copies: u32, reference: Answer, tally: &mut Tally) {
        judge_within(text, mode, copies, reference, tally, u64::MAX);
    }

    /// [`judge`] with every run that may reach the Horn engine stopped
    /// after `polls` polls of its stop condition.
    fn judge_within(
        text: &str,
        mode: Mode,
        copies: u32,
        reference: Answer,
        tally: &mut Tally,
        polls: u64,
    ) {
        let sequent: Sequent = text.parse().unwrap_or_else(|e| panic!("{text:?}: {e}"));
        tally.cases += 1;
        if reference != Answer::Unknown {
            tally.decided += 1;
        }
        for (name, options) in configurations(copies) {
            // The bound is for the Horn engine, forced or by the dispatch;
            // the focused engines end within their copies.
            let mut left = match options.engine {
                None | Some(Engine::Horn) => polls,
                Some(_) => u64::MAX,
            };
            let stop = |_| {
                left = left.saturating_sub(1);
                left == 0
            };
            let limits = crate::Limits::default();
            let outcome = match prove_within(&sequent, mode, &options, &limits, stop) {
                Ok(outcome) => outcome,
                Err(Error::EngineRefused { .. }) => continue,
                Err(e) => panic!("{text:?} in {mode} mode, {name}: {e}"),
            };
            let contradiction = match outcome.verdict {
                Verdict::Proved(_) => reference == Answer::Unprovable,
                Verdict::Unprovable(_) => reference == Answer::Proved,
                Verdict::Unknown(_) => false,
            };
            assert!(
                !contradiction,
                "{text:?} in {mode} mode within {copies} copies: {name} answers {:?}, the reference {reference:?}",
                outcome.verdict
            );
            if reference != Answer::Unknown {
                tally.judged += 1;
            }
        }
    }

    /// The reference agrees with every verdict a family knows by
    /// construction, at the sizes it decides: it proves the provable
    /// instances within their copies, refutes the unprovable ones without
    /// exponentials, and proves none of the others.
    #[test]
    fn agrees_with_the_families() {
        for (name, sizes) in [
            ("3-partition-mll-yes", &[4][..]),
            ("3-partition-mll-no", &[4]),
            ("partition-yes", &[2]),
            ("qbf", &[3, 4]),
            ("wide-m1", &[2, 3, 4]),
            ("wide-m2", &[2, 4]),
            ("wide-m3", &[3]),
            ("wide-m4", &[4]),
            ("mix", &[1, 2, 3]),
            ("counter", &[2, 4]),
            ("counter-over", &[2, 4]),
            ("growing", &[1, 2]),
            ("chain", &[1, 2, 3]),
            ("additive", &[1, 2, 3]),
        ] {
            let family = FAMILIES.iter().find(|f| f.name == name).unwrap();
            for &size in sizes {
                for index in 0..family.instances {
                    let instance = family.instance(size, index).unwrap();
                    let copies = instance.copies.unwrap_or(0);
                    let answer = super::sequent(&instance.sequent, instance.mode, copies);
                    let exponentials = instance.sequent.fragment().has_exponentials();
                    let expected = match (instance.provable, exponentials) {
                        (true, _) => answer == Answer::Proved,
                        (false, false) => answer == Answer::Unprovable,
                        (false, true) => answer != Answer::Proved,
                    };
                    assert!(expected, "{}: {answer:?}", instance.name);
                }
            }
        }
    }

    /// Generated classical sequents and their mutants, in every rule set,
    /// linear and affine: no engine contradicts the reference.
    #[test]
    fn classical_engines_agree() {
        let mut tally = Tally::default();
        for (i, rules) in Rules::ALL.into_iter().enumerate() {
            let mut rng = Rng::new(300 + i as u64);
            for _ in 0..8 {
                let budget = 2 + rng.below(8);
                let generate::Provable {
                    mut formulas,
                    copies,
                } = generate::provable(&mut rng, rules, 3, budget);
                let mut cases = vec![formulas.clone()];
                if generate::mutate(&mut rng, &mut formulas, 3) {
                    cases.push(formulas);
                }
                for formulas in &cases {
                    let text = generate::sequent(formulas);
                    for affine in [false, true] {
                        let mut mode = if rules.mix {
                            Mode::CLASSICAL.with_mix()
                        } else {
                            Mode::CLASSICAL
                        };
                        if affine {
                            mode = mode.with_affine();
                        }
                        let reference = classical(formulas, mode, copies);
                        judge(&text, mode, copies, reference, &mut tally);
                    }
                }
            }
        }
        assert!(
            4 * tally.decided >= 3 * tally.cases,
            "the reference decided {} of {} sequents ({} runs judged)",
            tally.decided,
            tally.cases,
            tally.judged
        );
    }

    /// Random Horn programs, Petri nets with a marking to reach or, in
    /// affine mode, to cover, written two-sided for intuitionistic mode
    /// and one-sided for the classical modes: no engine contradicts the
    /// reference. Where a clause is
    /// under `!` the reference proves within the copies or decides
    /// nothing, and without one it refutes as well.
    #[test]
    fn engines_agree_on_horn_programs() {
        use Tree::{Atom, Bang, Bot, Dual, Lolli, One, Par, Quest, Tensor};
        let b = Box::new;
        /// A tensor of atoms, `1` for none, or a `⅋` of their negations,
        /// `⊥` for none.
        fn join(atoms: &[u8], negated: bool) -> Tree {
            let atom = |&a: &u8| if negated { Dual(a) } else { Atom(a) };
            let unit = if negated { Bot } else { One };
            atoms
                .iter()
                .map(atom)
                .reduce(|l, r| {
                    if negated {
                        Par(Box::new(l), Box::new(r))
                    } else {
                        Tensor(Box::new(l), Box::new(r))
                    }
                })
                .unwrap_or(unit)
        }
        let mut rng = Rng::new(2_700);
        let atoms = |rng: &mut Rng, most: usize| -> Vec<u8> {
            (0..rng.below(most + 1))
                .map(|_| rng.below(3) as u8)
                .collect()
        };
        let mut tally = Tally::default();
        for case in 0..300 {
            let (mut hypotheses, mut formulas) = (Vec::new(), Vec::new());
            // Every other program reaches its goal by construction: the
            // marking after a few firings of its clauses under `!`.
            let walk = case % 2 == 0;
            let mut clauses = Vec::new();
            for reusable in [true, true, false, false] {
                if rng.below(2) == 0 || (walk && !reusable) {
                    continue;
                }
                let (body, head) = (atoms(&mut rng, 2), atoms(&mut rng, 2));
                let clause = Lolli(b(join(&body, false)), b(join(&head, false)));
                let negated = Tensor(b(join(&body, false)), b(join(&head, true)));
                if reusable {
                    hypotheses.push(Bang(b(clause)));
                    formulas.push(Quest(b(negated)));
                } else {
                    hypotheses.push(clause);
                    formulas.push(negated);
                }
                clauses.push((body, head));
            }
            let mut marking = atoms(&mut rng, 3);
            for &a in &marking {
                hypotheses.push(Atom(a));
                formulas.push(Dual(a));
            }
            let goal = if walk {
                for _ in 0..rng.below(3) {
                    let enabled: Vec<_> = clauses
                        .iter()
                        .filter(|(body, _)| {
                            body.iter().all(|a| {
                                body.iter().filter(|&b| b == a).count()
                                    <= marking.iter().filter(|&b| b == a).count()
                            })
                        })
                        .collect();
                    if enabled.is_empty() {
                        break;
                    }
                    let (body, head) = enabled[rng.below(enabled.len())];
                    for a in body {
                        let at = marking.iter().position(|b| b == a).unwrap();
                        marking.remove(at);
                    }
                    marking.extend(head);
                }
                join(&marking, false)
            } else {
                join(&atoms(&mut rng, 2), false)
            };
            formulas.push(goal.clone());
            let copies = 3;
            // The Horn engine, forced or by the dispatch, searches a net
            // whose markings grow without end until it is stopped.
            let polls = 5_000;
            let text = generate::two_sided(&hypotheses, &goal);
            for mode in [Mode::INTUITIONISTIC, Mode::INTUITIONISTIC.with_affine()] {
                let reference = intuitionistic(&hypotheses, &goal, mode, copies);
                judge_within(&text, mode, copies, reference, &mut tally, polls);
            }
            let text = generate::sequent(&formulas);
            for mode in [
                Mode::CLASSICAL,
                Mode::CLASSICAL.with_mix(),
                Mode::CLASSICAL.with_affine(),
            ] {
                let reference = classical(&formulas, mode, copies);
                judge_within(&text, mode, copies, reference, &mut tally, polls);
            }
        }
        assert!(
            2 * tally.decided >= tally.cases,
            "the reference decided {} of {} programs ({} runs judged)",
            tally.decided,
            tally.cases,
            tally.judged
        );
    }

    /// A one-sided sequent and its copies.
    type OneSided = (Vec<Tree>, u32);

    /// A two-sided sequent, its hypotheses and its goal, and its copies.
    type TwoSided = (Vec<Tree>, Tree, u32);

    /// Sequents whose proofs need one or two contractions on a branch,
    /// which the generators make seldom: the reference's answers at the
    /// budgets below are where a fault of its bound shows. Each as text
    /// for the parser and as trees for the reference, with its copies.
    fn contractions() -> (Vec<OneSided>, Vec<TwoSided>) {
        use Tree::{Atom, Bang, Dual, Lolli, Quest, Tensor};
        let b = Box::new;
        let (a, c) = (|| Atom(0), || Atom(1));
        let classical = vec![
            (vec![Quest(b(Dual(0))), Tensor(b(a()), b(a()))], 2),
            (
                vec![Quest(b(Dual(0))), Tensor(b(Tensor(b(a()), b(a()))), b(a()))],
                3,
            ),
            (
                vec![
                    Quest(b(Dual(0))),
                    Quest(b(Dual(1))),
                    Tensor(b(a()), b(Tensor(b(a()), b(c())))),
                ],
                3,
            ),
        ];
        let intuitionistic = vec![
            (vec![Bang(b(a()))], Tensor(b(a()), b(a())), 2),
            (
                vec![Bang(b(a()))],
                Tensor(b(Tensor(b(a()), b(a()))), b(a())),
                3,
            ),
            (
                vec![Bang(b(Lolli(b(a()), b(c())))), a(), a()],
                Tensor(b(c()), b(c())),
                2,
            ),
            (vec![Bang(b(a()))], Bang(b(Tensor(b(a()), b(a())))), 2),
        ];
        (classical, intuitionistic)
    }

    /// The sequents of [`contractions`], in every mode they have: no
    /// engine contradicts the reference.
    #[test]
    fn engines_agree_where_contractions_are_needed() {
        let mut tally = Tally::default();
        let (classical_cases, intuitionistic_cases) = contractions();
        for (formulas, copies) in &classical_cases {
            let text = generate::sequent(formulas);
            for mode in [
                Mode::CLASSICAL,
                Mode::CLASSICAL.with_affine(),
                Mode::CLASSICAL.with_mix(),
            ] {
                let reference = classical(formulas, mode, *copies);
                assert_eq!(reference, Answer::Proved, "{text:?} in {mode} mode");
                judge(&text, mode, *copies, reference, &mut tally);
            }
        }
        for (hypotheses, goal, copies) in &intuitionistic_cases {
            let text = generate::two_sided(hypotheses, goal);
            for mode in [Mode::INTUITIONISTIC, Mode::INTUITIONISTIC.with_affine()] {
                let reference = intuitionistic(hypotheses, goal, mode, *copies);
                assert_eq!(reference, Answer::Proved, "{text:?} in {mode} mode");
                judge(&text, mode, *copies, reference, &mut tally);
            }
        }
    }

    /// Generated intuitionistic sequents and their mutants, in every rule
    /// set, linear and affine: no engine contradicts the reference.
    #[test]
    fn intuitionistic_engines_agree() {
        let mut tally = Tally::default();
        for (i, rules) in IllRules::ALL.into_iter().enumerate() {
            let mut rng = Rng::new(400 + i as u64);
            for _ in 0..8 {
                let budget = 2 + rng.below(8);
                let generate::Ill {
                    mut hypotheses,
                    goal,
                    copies,
                } = generate::ill(&mut rng, rules, 3, budget);
                let mut cases = vec![hypotheses.clone()];
                if generate::mutate(&mut rng, &mut hypotheses, 3) {
                    cases.push(hypotheses);
                }
                for hypotheses in &cases {
                    let text = generate::two_sided(hypotheses, &goal);
                    for mode in [Mode::INTUITIONISTIC, Mode::INTUITIONISTIC.with_affine()] {
                        let reference = intuitionistic(hypotheses, &goal, mode, copies);
                        judge(&text, mode, copies, reference, &mut tally);
                    }
                }
            }
        }
        assert!(
            4 * tally.decided >= 3 * tally.cases,
            "the reference decided {} of {} sequents ({} runs judged)",
            tally.decided,
            tally.cases,
            tally.judged
        );
    }

    /// A random formula of the additive fragment over three atoms, at most
    /// `depth` connectives deep; with `negated` its atoms may be negated.
    fn additive(rng: &mut Rng, depth: usize, negated: bool) -> Tree {
        let pick = if depth == 0 {
            4 + rng.below(4)
        } else {
            rng.below(8)
        };
        let boxed = Box::new;
        match pick {
            0 | 1 => Tree::With(
                boxed(additive(rng, depth - 1, negated)),
                boxed(additive(rng, depth - 1, negated)),
            ),
            2 | 3 => Tree::Plus(
                boxed(additive(rng, depth - 1, negated)),
                boxed(additive(rng, depth - 1, negated)),
            ),
            4 if rng.one_in(3) => Tree::Top,
            4 if rng.one_in(2) => Tree::Zero,
            5 if negated => Tree::Dual(rng.below(3) as u8),
            _ => Tree::Atom(rng.below(3) as u8),
        }
    }

    /// Two additive formulas, the additive path's sequents, classical and
    /// intuitionistic: no engine contradicts the reference.
    #[test]
    fn additive_engines_agree() {
        let mut tally = Tally::default();
        let mut rng = Rng::new(500);
        for _ in 0..120 {
            let (a, b) = (additive(&mut rng, 3, true), additive(&mut rng, 3, true));
            let formulas = [a, b];
            let text = generate::sequent(&formulas);
            let reference = classical(&formulas, Mode::CLASSICAL, 0);
            judge(&text, Mode::CLASSICAL, 0, reference, &mut tally);
            let (a, b) = (additive(&mut rng, 3, false), additive(&mut rng, 3, false));
            let text = generate::two_sided(std::slice::from_ref(&a), &b);
            for mode in [Mode::INTUITIONISTIC, Mode::INTUITIONISTIC.with_affine()] {
                let reference = intuitionistic(std::slice::from_ref(&a), &b, mode, 0);
                judge(&text, mode, 0, reference, &mut tally);
            }
        }
        assert_eq!(
            tally.decided, tally.cases,
            "the additive fragment is decided"
        );
    }
}
