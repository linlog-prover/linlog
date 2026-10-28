// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The classical refuter: a search for an assignment of truth values to
//! the atoms under which a goal's classical reading is false. The reading
//! erases `!` and `?`, reads `⊗` and `&` as "and", `⅋` and `⊕` as "or",
//! `1` and `⊤` as true, `⊥` and `0` as false and `~a` as "not a", and a
//! sequent as the disjunction of its formulas. Every rule of linear
//! logic, in every mode, derives a tautology from tautologies (Mix and
//! weakening add a disjunct, contraction repeats one), and an
//! intuitionistic sequent's one-sided form reads as its implication, so
//! an assignment that falsifies a goal's reading proves it unprovable.
//!
//! The search runs after a search that did not prove the goal: a
//! backtracking over the goal's atoms in the order of their indices,
//! false before true, that evaluates the reading in three values at every
//! step and backs off where it is already true. It is bounded by work,
//! one unit per term evaluated, and by the memory account.

use super::memory::{Account, Charged};
use super::refutation::Classical;
use crate::limits::{Phase, Progress};
use crate::occurrences::{Forest, OccId};
use crate::sequents::{Term, TermId};

/// A truth value of three: the reading under a partial assignment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Value {
    /// False under every extension of the assignment.
    False,
    /// True under every extension.
    True,
    /// Not yet decided.
    Open,
}

impl Value {
    /// Returns the value of a conjunction.
    const fn and(self, other: Self) -> Self {
        match (self, other) {
            (Self::False, _) | (_, Self::False) => Self::False,
            (Self::True, Self::True) => Self::True,
            _ => Self::Open,
        }
    }

    /// Returns the value of a disjunction.
    const fn or(self, other: Self) -> Self {
        match (self, other) {
            (Self::True, _) | (_, Self::True) => Self::True,
            (Self::False, Self::False) => Self::False,
            _ => Self::Open,
        }
    }
}

/// Returns an assignment under which the classical reading of the goal
/// (`goal`'s occurrences of `forest`) is false, one value per atom of the
/// sequent by index, or `None` when its reading is a tautology, when the
/// search would pass `work` units (one per term evaluated, so zero
/// switches it off), when its buffers do not fit `account`, or when
/// `stop` says so: asked before each evaluation with the units done.
pub(crate) fn refute(
    forest: &Forest,
    goal: &[OccId],
    work: u64,
    account: &Account,
    stop: &mut dyn FnMut(Progress) -> bool,
) -> Option<Classical> {
    let sequent = forest.sequent();
    let terms = sequent.terms();
    let members: Vec<TermId> = goal.iter().map(|&o| forest.term(o)).collect();
    let width = members.iter().map(|t| t.index() + 1).max().unwrap_or(0);
    let atoms = sequent.atom_names().len();
    // A byte per term of the arena below the goal for the marks and the
    // values, four per term reached, a slot per atom twice.
    let bytes = width * 2 + width * 4 + atoms * 2;
    if !account.fits(bytes) {
        return None;
    }
    let mut held = Charged::new(account);
    held.charge(bytes);
    // The terms the goal reaches, in the arena's order: every term names
    // only terms before it, so this order evaluates operands first.
    let mut reached = vec![false; width];
    for t in &members {
        reached[t.index()] = true;
    }
    let mut order = Vec::new();
    let mut occurs = vec![false; atoms];
    for i in (0..width).rev() {
        if !reached[i] {
            continue;
        }
        let term = terms[i];
        for sub in term.subterms() {
            reached[sub.index()] = true;
        }
        if let Some(atom) = term.atom() {
            occurs[atom.index()] = true;
        }
        order.push(i as u32);
    }
    order.reverse();
    let cost = (order.len() as u64).max(1);
    let decided: Vec<usize> = (0..atoms).filter(|&a| occurs[a]).collect();
    let mut assigned: Vec<Option<bool>> = vec![None; atoms];
    let mut values = vec![Value::Open; width];
    // How many of `decided` have a value, and per depth whether its atom
    // was tried true too.
    let mut depth = 0;
    let mut both = vec![false; decided.len()];
    let mut done: u64 = 0;
    loop {
        // The bound is on work done, so the sum cannot pass it by more
        // than one evaluation, and `work` is a `u64`.
        if done.saturating_add(cost) > work || stop(Progress::new(Phase::Refute, cost, done)) {
            return None;
        }
        done += cost;
        let value = evaluate(terms, &order, &members, &assigned, &mut values);
        let descend = match value {
            Value::False => {
                let assignment = assigned.iter().map(|v| v.unwrap_or(false)).collect();
                return Some(Classical { assignment });
            }
            Value::Open => true,
            Value::True => false,
        };
        if descend {
            // An open value has an atom without a value below it.
            assigned[decided[depth]] = Some(false);
            both[depth] = false;
            depth += 1;
            continue;
        }
        loop {
            if depth == 0 {
                return None;
            }
            depth -= 1;
            if both[depth] {
                assigned[decided[depth]] = None;
            } else {
                assigned[decided[depth]] = Some(true);
                both[depth] = true;
                depth += 1;
                break;
            }
        }
    }
}

/// Returns the reading of the goal whose terms are `members` under the
/// partial assignment `assigned`, evaluating the terms of `order` (the
/// terms the goal reaches, operands first) into `values`.
fn evaluate(
    terms: &[Term],
    order: &[u32],
    members: &[TermId],
    assigned: &[Option<bool>],
    values: &mut [Value],
) -> Value {
    let of = |v: Option<bool>| match v {
        Some(true) => Value::True,
        Some(false) => Value::False,
        None => Value::Open,
    };
    for &i in order {
        let i = i as usize;
        values[i] = match terms[i] {
            Term::Atom(a) => of(assigned[a.index()]),
            Term::DualAtom(a) => of(assigned[a.index()].map(|v| !v)),
            Term::One | Term::Top => Value::True,
            Term::Bot | Term::Zero => Value::False,
            Term::Tensor(k, l) | Term::With(k, l) => values[k.index()].and(values[l.index()]),
            Term::Par(k, l) | Term::Plus(k, l) => values[k.index()].or(values[l.index()]),
            Term::Bang(k) | Term::Quest(k) => values[k.index()],
            Term::Forall(_) | Term::Exists(_) => {
                unreachable!("the classical refuter refuses a sequent with a binder")
            }
        };
    }
    members
        .iter()
        .fold(Value::False, |reading, t| reading.or(values[t.index()]))
}

#[cfg(all(test, feature = "parse"))]
mod tests {
    use super::*;
    use crate::Sequent;

    /// Returns what the refuter finds for the roots of `input`.
    fn refuted(input: &str, work: u64) -> Option<Vec<bool>> {
        let sequent: Sequent = input.parse().unwrap();
        let forest = Forest::new(&sequent).unwrap();
        let account = Account::new(None);
        refute(&forest, forest.roots(), work, &account, &mut |_| false).map(|c| c.assignment)
    }

    /// A falsifying assignment is found where one exists, false tried
    /// first, a tautology is answered with none, and the bound and the
    /// stop give up.
    #[test]
    fn finds_an_assignment_within_its_bound() {
        assert_eq!(refuted("a |- b", 100), Some(vec![true, false]));
        assert_eq!(refuted("|- a, ~a", 100), None);
        assert_eq!(refuted("|- !(a -o b) -o ?a -o b", 100), None);
        assert_eq!(refuted("|- a * b, bot", 100), Some(vec![false, false]));
        assert_eq!(refuted("|- a * b, bot", 0), None);
        let sequent: Sequent = "a |- b".parse().unwrap();
        let forest = Forest::new(&sequent).unwrap();
        let account = Account::new(None);
        assert!(refute(&forest, forest.roots(), 100, &account, &mut |_| true).is_none());
    }

    /// The reading against checked proofs: every generated sequent the
    /// search proves, its proof checked, in every rule set and mode,
    /// classical, with Mix, affine and intuitionistic, reads as a
    /// tautology, so the refuter finds no assignment; and of the mutants,
    /// every one it refutes passes [`Disproof::check`] and none has a
    /// proof.
    #[test]
    fn checked_proofs_read_as_tautologies() {
        use crate::Mode;
        use crate::search::generate::{self, IllRules, Rng, Rules};
        use crate::search::{Options, Refutation, Verdict, prove};
        let (mut proved, mut refuted) = (0, 0);
        let mut judge = |text: &str, mode: Mode, provable: bool| {
            let sequent: Sequent = text.parse().unwrap();
            let outcome = prove(&sequent, mode, &Options::default()).unwrap();
            let forest = Forest::new(&sequent).unwrap();
            let found = refute(
                &forest,
                forest.roots(),
                1 << 20,
                &Account::new(None),
                &mut |_| false,
            );
            match outcome.verdict {
                Verdict::Proved(_) => {
                    assert!(outcome.checked, "{text}");
                    assert_eq!(found, None, "{text} in {mode}: a checked proof");
                    proved += 1;
                }
                Verdict::Unprovable(disproof) => {
                    assert!(!provable, "{text} in {mode}: built from a proof");
                    if let Refutation::Classical(_) = disproof.refutation() {
                        assert_eq!(disproof.check(), Ok(()), "{text} in {mode}");
                        refuted += 1;
                    }
                }
                Verdict::Unknown(_) => assert_eq!(found, None, "{text} in {mode}"),
            }
        };
        for (i, rules) in Rules::ALL.into_iter().enumerate() {
            let mut rng = Rng::new(3100 + i as u64);
            for _ in 0..12 {
                let budget = 2 + rng.below(10);
                let mut formulas = generate::provable(&mut rng, rules, 3, budget).formulas;
                let base = if rules.mix {
                    Mode::CLASSICAL.with_mix()
                } else {
                    Mode::CLASSICAL
                };
                for mode in [base, base.with_affine()] {
                    judge(&generate::sequent(&formulas), mode, true);
                }
                if generate::mutate(&mut rng, &mut formulas, 3) {
                    judge(&generate::sequent(&formulas), base, false);
                }
            }
        }
        for (i, rules) in IllRules::ALL.into_iter().enumerate() {
            let mut rng = Rng::new(3200 + i as u64);
            for _ in 0..12 {
                let budget = 2 + rng.below(10);
                let ill = generate::ill(&mut rng, rules, 3, budget);
                let text = generate::two_sided(&ill.hypotheses, &ill.goal);
                for mode in [Mode::INTUITIONISTIC, Mode::INTUITIONISTIC.with_affine()] {
                    judge(&text, mode, true);
                }
            }
        }
        assert!(
            proved >= 400 && refuted > 0,
            "{proved} proved, {refuted} refuted"
        );
    }
}
