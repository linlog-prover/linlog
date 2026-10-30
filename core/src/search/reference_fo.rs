// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! A first-order reference prover for the tests: it decides a small
//! sequent with quantifiers by the plain rules of the sequent calculus,
//! sharing no code with the engines, one-sided in classical linear logic
//! and two-sided in intuitionistic linear logic. Its formulas and terms
//! are its own, read from a sequent's arenas (one-sided) or built by the
//! tests directly (two-sided); a witness of `∃` (and two-sided of `∀` on
//! the left) is an unknown that unification at the axioms binds, with the
//! occurs check, and the eigenvariable condition is checked as the
//! calculus states it, by freshness: once every goal is closed, the
//! eigenvariable of each `∀` step (two-sided: `∀R` and `∃L`) must occur
//! nowhere in that step's conclusion under the bindings made. A derivation
//! that fails it is no proof, and the search goes on to the next one, so
//! every derivation is tried.
//!
//! The goals still to prove are a stack, so a goal is proved under the
//! bindings its siblings left and a failure later backtracks into any
//! choice made before: the answers of a premise are all tried. The
//! invertible rules are applied first, on the first formula that has one,
//! which the inversion lemma of the calculus allows: one-sided `⅋`, `⊥`,
//! `⊤`, `&`, `∀`; every other rule is tried on every formula: the axiom on
//! two literals, `1` alone, every split of the rest for `⊗`, both sides of
//! `⊕`, an unknown for `∃`, `!` where the rest is all `?` formulas, and
//! for `?A` dereliction, weakening and, within the budget, contraction;
//! in affine mode the weakening of any formula, and with Mix every split
//! into two non-empty parts. Two-sided, the invertible rules are `⊸R`,
//! `&R`, `⊤R`, `∀R`, `⊗L`, `1L`, `⊕L`, `∃L` and `0L` ([`Prover::invert`]
//! says why each is), and the others are tried on the goal and on every
//! hypothesis: the axiom on one atom against the goal, `1R` alone, every
//! split for `⊗R` and `⊸L`, both sides of `⊕R` and `&L`, an unknown for
//! `∃R` and `∀L`, `!R` where every hypothesis is a `!` formula, and for a
//! `!` hypothesis dereliction, weakening and, within the budget,
//! contraction; in affine mode the weakening of any hypothesis, never of
//! the goal.
//!
//! Contraction is bounded per branch, as the propositional reference
//! bounds it, and a goal with a `?` formula (two-sided: a `!` hypothesis)
//! that fails with its budget spent keeps the answer from "unprovable".
//! The visits are bounded too. It judges an engine's "unprovable"; an
//! engine's proof is the checker's to judge.

use super::reference::Answer;
use crate::Sequent;
use crate::fragment::Mode;
use crate::sequents::{FoTerm, FoTermId, Term, TermId};

/// The most goals one decision takes up before it gives up.
pub(crate) const VISITS: usize = 400_000;

/// A term: a variable bound by a quantifier above (a de Bruijn index), a
/// function symbol applied to terms, an unknown of the search, or an
/// eigenvariable.
#[derive(Clone, Debug, PartialEq, Eq)]
enum T {
    /// A bound variable.
    Bound(u32),
    /// A symbol applied to terms.
    App(u32, Vec<T>),
    /// An unknown, by index.
    Var(usize),
    /// An eigenvariable, by number.
    Eigen(u32),
}

/// A formula of a one-sided sequent.
#[derive(Clone, Debug)]
enum F {
    /// A literal: whether it is the atom (not its negation), its
    /// predicate, and its arguments.
    Lit(bool, u32, Vec<T>),
    /// `1`.
    One,
    /// `⊥`.
    Bot,
    /// `⊤`.
    Top,
    /// `0`.
    Zero,
    /// `A ⊗ B`.
    Tensor(Box<F>, Box<F>),
    /// `A ⅋ B`.
    Par(Box<F>, Box<F>),
    /// `A & B`.
    With(Box<F>, Box<F>),
    /// `A ⊕ B`.
    Plus(Box<F>, Box<F>),
    /// `!A`.
    Bang(Box<F>),
    /// `?A`.
    Quest(Box<F>),
    /// `∀x.A`.
    Forall(Box<F>),
    /// `∃x.A`.
    Exists(Box<F>),
    /// `A ⊸ B`, two-sided only.
    Lolli(Box<F>, Box<F>),
}

/// A goal: a sequent and the contractions its branch may still make.
type Goal = (Vec<F>, u32);

/// A two-sided goal: the hypotheses, the goal formula and the
/// contractions its branch may still make.
type Judgement = (Vec<F>, F, u32);

/// Why a search stopped before its end.
struct GaveUp;

/// The state of a decision.
struct Prover {
    /// The mode: affine, Mix.
    mode: Mode,
    /// What each unknown is bound to.
    bound: Vec<Option<T>>,
    /// The unknowns bound, in order, for undoing.
    trail: Vec<usize>,
    /// The eigenvariables made: the next one's number.
    eigens: u32,
    /// The steps that introduce an eigenvariable (`∀`, two-sided `∀R`
    /// and `∃L`) on the derivation in the making: the eigenvariable and
    /// the formulas of the step's conclusion.
    foralls: Vec<(u32, Vec<F>)>,
    /// The goals taken up.
    visits: usize,
    /// Whether a goal with a `?` formula (two-sided: a `!` hypothesis)
    /// failed with no contraction left.
    wanted: bool,
}

/// Decides the one-sided classical sequent under the mode (its `affine`
/// and `mix`), with at most `copies` contractions per branch.
pub(crate) fn sequent(sequent: &Sequent, mode: Mode, copies: u32) -> Answer {
    assert!(!mode.intuitionistic, "a one-sided sequent is classical");
    let formulas: Vec<F> = sequent
        .roots()
        .iter()
        .map(|&root| formula(sequent, root))
        .collect();
    for budget in 0..=copies {
        let mut prover = Prover::new(mode);
        let mut stack = vec![(formulas.clone(), budget)];
        match prover.solve(&mut stack) {
            Ok(true) => return Answer::Proved,
            Ok(false) if !prover.wanted => return Answer::Unprovable,
            Ok(false) => {}
            Err(GaveUp) => return Answer::Unknown,
        }
    }
    Answer::Unknown
}

/// Decides the two-sided intuitionistic sequent `hypotheses ⊢ goal`
/// (atoms as positive literals, `1`, `⊤`, `0`, `⊗`, `⊸`, `&`, `⊕`, `!`,
/// `∀` and `∃`, closed) under the mode (its `affine`), with at most
/// `copies` contractions per branch.
fn intuitionistic(hypotheses: &[F], goal: &F, mode: Mode, copies: u32) -> Answer {
    assert!(
        mode.intuitionistic && !mode.mix,
        "a two-sided sequent is intuitionistic, without Mix"
    );
    for budget in 0..=copies {
        let mut prover = Prover::new(mode);
        let mut stack = vec![(hypotheses.to_vec(), goal.clone(), budget)];
        match prover.solve_two_sided(&mut stack) {
            Ok(true) => return Answer::Proved,
            Ok(false) if !prover.wanted => return Answer::Unprovable,
            Ok(false) => {}
            Err(GaveUp) => return Answer::Unknown,
        }
    }
    Answer::Unknown
}

/// The formula of a term of the sequent.
fn formula(sequent: &Sequent, t: TermId) -> F {
    let sub = |t| Box::new(formula(sequent, t));
    match sequent.term(t) {
        Term::Atom(a) | Term::DualAtom(a) => {
            let positive = matches!(sequent.term(t), Term::Atom(_));
            match sequent.atom_predicate(a) {
                Some((symbol, args)) => F::Lit(
                    positive,
                    symbol.get(),
                    sequent
                        .arguments(args)
                        .iter()
                        .map(|&arg| term(sequent, arg))
                        .collect(),
                ),
                // Predicates past every symbol: a sequent's atoms have
                // fewer than 2³¹ names.
                None => F::Lit(positive, (1 << 31) | a.get(), Vec::new()),
            }
        }
        Term::One => F::One,
        Term::Bot => F::Bot,
        Term::Top => F::Top,
        Term::Zero => F::Zero,
        Term::Tensor(a, b) => F::Tensor(sub(a), sub(b)),
        Term::Par(a, b) => F::Par(sub(a), sub(b)),
        Term::With(a, b) => F::With(sub(a), sub(b)),
        Term::Plus(a, b) => F::Plus(sub(a), sub(b)),
        Term::Bang(a) => F::Bang(sub(a)),
        Term::Quest(a) => F::Quest(sub(a)),
        Term::Forall(a) => F::Forall(sub(a)),
        Term::Exists(a) => F::Exists(sub(a)),
    }
}

/// The term of a first-order term of the sequent.
fn term(sequent: &Sequent, t: FoTermId) -> T {
    match sequent.fo_terms()[t.index()] {
        FoTerm::Bound(i) => T::Bound(i),
        FoTerm::App(symbol, args) => T::App(
            symbol.get(),
            sequent
                .arguments(args)
                .iter()
                .map(|&a| term(sequent, a))
                .collect(),
        ),
        FoTerm::Eigen(_) | FoTerm::Meta(_) => {
            unreachable!("a sequent's arena holds no eigenvariable or metavariable")
        }
    }
}

/// The body of a quantifier with its variable replaced by `t`: the
/// bound variable as many binders deep as it lies.
fn open(f: &F, t: &T, depth: u32) -> F {
    let sub = |g: &F| Box::new(open(g, t, depth));
    match f {
        F::Lit(sign, p, args) => F::Lit(
            *sign,
            *p,
            args.iter().map(|a| open_term(a, t, depth)).collect(),
        ),
        F::One => F::One,
        F::Bot => F::Bot,
        F::Top => F::Top,
        F::Zero => F::Zero,
        F::Tensor(a, b) => F::Tensor(sub(a), sub(b)),
        F::Par(a, b) => F::Par(sub(a), sub(b)),
        F::With(a, b) => F::With(sub(a), sub(b)),
        F::Plus(a, b) => F::Plus(sub(a), sub(b)),
        F::Bang(a) => F::Bang(sub(a)),
        F::Quest(a) => F::Quest(sub(a)),
        F::Forall(a) => F::Forall(Box::new(open(a, t, depth + 1))),
        F::Exists(a) => F::Exists(Box::new(open(a, t, depth + 1))),
        F::Lolli(a, b) => F::Lolli(sub(a), sub(b)),
    }
}

/// A term with the variable `depth` binders up replaced by `t`.
fn open_term(term: &T, t: &T, depth: u32) -> T {
    match term {
        T::Bound(i) if *i == depth => t.clone(),
        T::App(s, args) => T::App(*s, args.iter().map(|a| open_term(a, t, depth)).collect()),
        other => other.clone(),
    }
}

impl Prover {
    /// A prover for the mode, with nothing bound yet.
    const fn new(mode: Mode) -> Self {
        Self {
            mode,
            bound: Vec::new(),
            trail: Vec::new(),
            eigens: 0,
            foralls: Vec::new(),
            visits: 0,
            wanted: false,
        }
    }

    /// A term with every bound unknown replaced by its binding.
    fn resolve(&self, t: &T) -> T {
        match t {
            T::Var(v) => match &self.bound[*v] {
                Some(b) => self.resolve(b),
                None => t.clone(),
            },
            T::App(s, args) => T::App(*s, args.iter().map(|a| self.resolve(a)).collect()),
            other => other.clone(),
        }
    }

    /// Whether the unknown `v` occurs in `t` as it stands.
    fn occurs(&self, v: usize, t: &T) -> bool {
        match t {
            T::Var(w) => *w == v || self.bound[*w].as_ref().is_some_and(|b| self.occurs(v, b)),
            T::App(_, args) => args.iter().any(|a| self.occurs(v, a)),
            T::Bound(_) | T::Eigen(_) => false,
        }
    }

    /// Unifies two terms, binding unknowns; the bindings made stay on the
    /// trail for the caller to undo on failure.
    fn unify(&mut self, a: &T, b: &T) -> bool {
        let (a, b) = (self.walk(a), self.walk(b));
        match (&a, &b) {
            (T::Var(x), T::Var(y)) if x == y => true,
            (T::Var(x), t) | (t, T::Var(x)) => {
                if self.occurs(*x, t) {
                    return false;
                }
                self.bound[*x] = Some(t.clone());
                self.trail.push(*x);
                true
            }
            (T::Eigen(x), T::Eigen(y)) => x == y,
            (T::App(s, xs), T::App(r, ys)) => {
                s == r && xs.len() == ys.len() && xs.iter().zip(ys).all(|(x, y)| self.unify(x, y))
            }
            _ => false,
        }
    }

    /// A term with the bindings of its outermost unknowns followed.
    fn walk(&self, t: &T) -> T {
        match t {
            T::Var(v) => match &self.bound[*v] {
                Some(b) => self.walk(b),
                None => t.clone(),
            },
            other => other.clone(),
        }
    }

    /// Whether the eigenvariable `e` occurs in the formula under the
    /// bindings.
    fn names(&self, f: &F, e: u32) -> bool {
        match f {
            F::Lit(_, _, args) => args.iter().any(|a| contains(&self.resolve(a), e)),
            F::One | F::Bot | F::Top | F::Zero => false,
            F::Tensor(a, b) | F::Par(a, b) | F::With(a, b) | F::Plus(a, b) | F::Lolli(a, b) => {
                self.names(a, e) || self.names(b, e)
            }
            F::Bang(a) | F::Quest(a) | F::Forall(a) | F::Exists(a) => self.names(a, e),
        }
    }

    /// Whether every `∀` step's eigenvariable is absent from its
    /// conclusion.
    fn fresh(&self) -> bool {
        self.foralls
            .iter()
            .all(|(e, conclusion)| !conclusion.iter().any(|f| self.names(f, *e)))
    }

    /// Takes back the bindings and unknowns made since the marks.
    fn undo(&mut self, (trail, unknowns, eigens): (usize, usize, u32)) {
        while self.trail.len() > trail {
            let v = self.trail.pop().expect("longer than the mark");
            self.bound[v] = None;
        }
        self.bound.truncate(unknowns);
        self.eigens = eigens;
    }

    /// Proves every goal on the stack, the last first, under the bindings
    /// as they stand; leaves the stack as it found it where it fails.
    fn solve(&mut self, stack: &mut Vec<Goal>) -> Result<bool, GaveUp> {
        let Some((sequent, budget)) = stack.pop() else {
            return Ok(self.fresh());
        };
        self.visits += 1;
        if self.visits > VISITS {
            return Err(GaveUp);
        }
        let found = self.goal(stack, &sequent, budget);
        if !matches!(found, Ok(true)) {
            stack.push((sequent, budget));
        }
        found
    }

    /// Tries every rule on the goal `sequent`, the rest of the stack
    /// after it.
    fn goal(&mut self, stack: &mut Vec<Goal>, sequent: &[F], budget: u32) -> Result<bool, GaveUp> {
        let rest = |i: usize| -> Vec<F> {
            sequent
                .iter()
                .enumerate()
                .filter(|&(j, _)| j != i)
                .map(|(_, f)| f.clone())
                .collect()
        };
        let mark = (self.trail.len(), self.bound.len(), self.eigens);
        if let Some(i) = sequent
            .iter()
            .position(|f| matches!(f, F::Par(..) | F::Bot | F::Top | F::With(..) | F::Forall(_)))
        {
            return match &sequent[i] {
                F::Top => self.solve(stack),
                F::Bot => self.premises(stack, vec![(rest(i), budget)], mark),
                F::Par(a, b) => {
                    let mut r = rest(i);
                    r.push((**a).clone());
                    r.push((**b).clone());
                    self.premises(stack, vec![(r, budget)], mark)
                }
                F::With(a, b) => {
                    let (mut l, mut r) = (rest(i), rest(i));
                    l.push((**a).clone());
                    r.push((**b).clone());
                    self.premises(stack, vec![(l, budget), (r, budget)], mark)
                }
                F::Forall(a) => {
                    let e = self.eigens;
                    self.eigens += 1;
                    self.foralls.push((e, sequent.to_vec()));
                    let mut r = rest(i);
                    r.push(open(a, &T::Eigen(e), 0));
                    let found = self.premises(stack, vec![(r, budget)], mark);
                    if !matches!(found, Ok(true)) {
                        self.foralls.pop();
                    }
                    found
                }
                _ => unreachable!("an invertible formula"),
            };
        }
        // The axiom joins two literals alone; in affine mode the others
        // are weakened first, by the rule below.
        if let [F::Lit(s, p, xs), F::Lit(t, q, ys)] = sequent
            && s != t
            && p == q
            && xs.len() == ys.len()
        {
            if xs.iter().zip(ys).all(|(x, y)| self.unify(x, y)) && self.solve(stack)? {
                return Ok(true);
            }
            self.undo(mark);
        }
        if sequent.len() == 1 && matches!(sequent[0], F::One) && self.solve(stack)? {
            return Ok(true);
        }
        let mut quest_failed = false;
        for i in 0..sequent.len() {
            let others = rest(i);
            let mut alternatives: Vec<Vec<Goal>> = Vec::new();
            match &sequent[i] {
                F::Tensor(a, b) => {
                    for mask in 0..1u64 << others.len() {
                        let (mut l, mut r) = (Vec::new(), Vec::new());
                        for (k, f) in others.iter().enumerate() {
                            if mask >> k & 1 == 1 {
                                l.push(f.clone());
                            } else {
                                r.push(f.clone());
                            }
                        }
                        l.push((**a).clone());
                        r.push((**b).clone());
                        alternatives.push(vec![(l, budget), (r, budget)]);
                    }
                }
                F::Plus(a, b) => {
                    for side in [a, b] {
                        let mut r = others.clone();
                        r.push((**side).clone());
                        alternatives.push(vec![(r, budget)]);
                    }
                }
                F::Bang(a) if others.iter().all(|f| matches!(f, F::Quest(_))) => {
                    let mut r = others.clone();
                    r.push((**a).clone());
                    alternatives.push(vec![(r, budget)]);
                }
                F::Quest(a) => {
                    let mut derelict = others.clone();
                    derelict.push((**a).clone());
                    alternatives.push(vec![(derelict, budget)]);
                    alternatives.push(vec![(others.clone(), budget)]);
                    if budget > 0 {
                        let mut contracted = sequent.to_vec();
                        contracted.push(F::Quest(a.clone()));
                        alternatives.push(vec![(contracted, budget - 1)]);
                    } else {
                        quest_failed = true;
                    }
                }
                F::Exists(a) => {
                    let v = self.bound.len();
                    self.bound.push(None);
                    let mut r = others.clone();
                    r.push(open(a, &T::Var(v), 0));
                    if self.premises(stack, vec![(r, budget)], mark)? {
                        return Ok(true);
                    }
                }
                _ => {}
            }
            if self.mode.affine && sequent.len() > 1 {
                alternatives.push(vec![(others.clone(), budget)]);
            }
            for premises in alternatives {
                if self.premises(stack, premises, mark)? {
                    return Ok(true);
                }
            }
        }
        if self.mode.mix && sequent.len() > 1 {
            // The first formula on the left, so each partition comes once.
            for mask in 0..1u64 << (sequent.len() - 1) {
                let (mut l, mut r) = (vec![sequent[0].clone()], Vec::new());
                for (k, f) in sequent[1..].iter().enumerate() {
                    if mask >> k & 1 == 1 {
                        l.push(f.clone());
                    } else {
                        r.push(f.clone());
                    }
                }
                if !r.is_empty() && self.premises(stack, vec![(l, budget), (r, budget)], mark)? {
                    return Ok(true);
                }
            }
        }
        if quest_failed {
            self.wanted = true;
        }
        Ok(false)
    }

    /// Pushes the premises of a rule, the first last, so it is proved
    /// first, and proves the stack; on a failure takes them off again and
    /// undoes what was bound since `mark`.
    fn premises(
        &mut self,
        stack: &mut Vec<Goal>,
        premises: Vec<Goal>,
        mark: (usize, usize, u32),
    ) -> Result<bool, GaveUp> {
        let height = stack.len();
        stack.extend(premises.into_iter().rev());
        let found = self.solve(stack);
        if !matches!(found, Ok(true)) {
            stack.truncate(height);
            self.undo(mark);
        }
        found
    }

    // The two-sided calculus.

    /// Proves every two-sided goal on the stack, the last first, under
    /// the bindings as they stand; leaves the stack as it found it where
    /// it fails.
    fn solve_two_sided(&mut self, stack: &mut Vec<Judgement>) -> Result<bool, GaveUp> {
        let Some((hypotheses, goal, budget)) = stack.pop() else {
            return Ok(self.fresh());
        };
        self.visits += 1;
        if self.visits > VISITS {
            return Err(GaveUp);
        }
        let found = self.judgement(stack, &hypotheses, &goal, budget);
        if !matches!(found, Ok(true)) {
            stack.push((hypotheses, goal, budget));
        }
        found
    }

    /// Introduces an eigenvariable for the step whose conclusion is
    /// `hypotheses ⊢ goal` and whose premise is `premise` applied to it,
    /// and proves the stack with that premise on it; the step stays on
    /// record for the freshness check only while it is part of the
    /// derivation.
    fn eigen_step(
        &mut self,
        stack: &mut Vec<Judgement>,
        (hypotheses, goal): (&[F], &F),
        premise: impl FnOnce(T) -> Judgement,
        mark: (usize, usize, u32),
    ) -> Result<bool, GaveUp> {
        let e = self.eigens;
        self.eigens += 1;
        let mut conclusion = hypotheses.to_vec();
        conclusion.push(goal.clone());
        self.foralls.push((e, conclusion));
        let found = self.premises_two_sided(stack, vec![premise(T::Eigen(e))], mark);
        if !matches!(found, Ok(true)) {
            self.foralls.pop();
        }
        found
    }

    /// Applies the first invertible rule that `hypotheses ⊢ goal` has, if
    /// any: `Some` with what proving its premises found, the only rule
    /// tried then. Each is invertible in intuitionistic linear logic (its
    /// premises follow from its conclusion by a cut with a provable
    /// sequent, and the inversion keeps a cut-free derivation's height
    /// and its contractions per branch, so it holds at every budget):
    /// `⊸R` by `A ⊸ B, A ⊢ B`, `&R` by `A & B ⊢ A` and `A & B ⊢ B`, `∀R`
    /// by `∀x.A ⊢ A[e/x]`, `⊗L` by `A, B ⊢ A ⊗ B`, `1L` by `⊢ 1`, `⊕L` by
    /// `A ⊢ A ⊕ B` and `B ⊢ A ⊕ B`, `∃L` by `A[e/x] ⊢ ∃x.A`, and `⊤R` and
    /// `0L` have no premise to lose.
    fn invert(
        &mut self,
        stack: &mut Vec<Judgement>,
        (hypotheses, goal, budget): (&[F], &F, u32),
        mark: (usize, usize, u32),
    ) -> Option<Result<bool, GaveUp>> {
        let with = |i: Option<usize>, added: &[&F]| -> Vec<F> {
            let mut formulas: Vec<F> = hypotheses
                .iter()
                .enumerate()
                .filter(|&(j, _)| Some(j) != i)
                .map(|(_, f)| f.clone())
                .collect();
            formulas.extend(added.iter().map(|&f| f.clone()));
            formulas
        };
        match goal {
            F::Top => return Some(self.solve_two_sided(stack)),
            F::Lolli(a, b) => {
                let premise = (with(None, &[&**a]), (**b).clone(), budget);
                return Some(self.premises_two_sided(stack, vec![premise], mark));
            }
            F::With(a, b) => {
                let premises = vec![
                    (hypotheses.to_vec(), (**a).clone(), budget),
                    (hypotheses.to_vec(), (**b).clone(), budget),
                ];
                return Some(self.premises_two_sided(stack, premises, mark));
            }
            F::Forall(a) => {
                let premise = |e| (hypotheses.to_vec(), open(a, &e, 0), budget);
                return Some(self.eigen_step(stack, (hypotheses, goal), premise, mark));
            }
            _ => {}
        }
        let i = hypotheses.iter().position(|h| {
            matches!(
                h,
                F::Zero | F::One | F::Tensor(..) | F::Plus(..) | F::Exists(_)
            )
        })?;
        let premises = match &hypotheses[i] {
            F::Zero => return Some(self.solve_two_sided(stack)),
            F::One => vec![(with(Some(i), &[]), goal.clone(), budget)],
            F::Tensor(a, b) => vec![(with(Some(i), &[&**a, &**b]), goal.clone(), budget)],
            F::Plus(a, b) => vec![
                (with(Some(i), &[&**a]), goal.clone(), budget),
                (with(Some(i), &[&**b]), goal.clone(), budget),
            ],
            F::Exists(a) => {
                let premise = |e| (with(Some(i), &[&open(a, &e, 0)]), goal.clone(), budget);
                return Some(self.eigen_step(stack, (hypotheses, goal), premise, mark));
            }
            _ => unreachable!("an invertible hypothesis"),
        };
        Some(self.premises_two_sided(stack, premises, mark))
    }

    /// Tries every rule on the goal `hypotheses ⊢ goal`, the rest of the
    /// stack after it: the invertible one alone where there is one.
    fn judgement(
        &mut self,
        stack: &mut Vec<Judgement>,
        hypotheses: &[F],
        goal: &F,
        budget: u32,
    ) -> Result<bool, GaveUp> {
        let mark = (self.trail.len(), self.bound.len(), self.eigens);
        if let Some(found) = self.invert(stack, (hypotheses, goal, budget), mark) {
            return found;
        }
        let rest = |i: usize| -> Vec<F> {
            hypotheses
                .iter()
                .enumerate()
                .filter(|&(j, _)| j != i)
                .map(|(_, f)| f.clone())
                .collect()
        };
        // The axiom joins one atom to the goal alone; in affine mode the
        // other hypotheses are weakened first, by the rule below.
        if let ([F::Lit(_, p, xs)], F::Lit(_, q, ys)) = (hypotheses, goal)
            && p == q
            && xs.len() == ys.len()
        {
            if xs.iter().zip(ys).all(|(x, y)| self.unify(x, y)) && self.solve_two_sided(stack)? {
                return Ok(true);
            }
            self.undo(mark);
        }
        if hypotheses.is_empty() && matches!(goal, F::One) && self.solve_two_sided(stack)? {
            return Ok(true);
        }
        // The right rules.
        let mut alternatives: Vec<Vec<Judgement>> = Vec::new();
        match goal {
            F::Tensor(a, b) => {
                for (l, r) in splits(hypotheses) {
                    alternatives.push(vec![(l, (**a).clone(), budget), (r, (**b).clone(), budget)]);
                }
            }
            F::Plus(a, b) => {
                for side in [a, b] {
                    alternatives.push(vec![(hypotheses.to_vec(), (**side).clone(), budget)]);
                }
            }
            F::Bang(a) if hypotheses.iter().all(|h| matches!(h, F::Bang(_))) => {
                alternatives.push(vec![(hypotheses.to_vec(), (**a).clone(), budget)]);
            }
            F::Exists(a) => {
                let v = self.bound.len();
                self.bound.push(None);
                let premise = (hypotheses.to_vec(), open(a, &T::Var(v), 0), budget);
                if self.premises_two_sided(stack, vec![premise], mark)? {
                    return Ok(true);
                }
            }
            _ => {}
        }
        for premises in alternatives {
            if self.premises_two_sided(stack, premises, mark)? {
                return Ok(true);
            }
        }
        // The left rules.
        let mut bang_failed = false;
        for i in 0..hypotheses.len() {
            let others = rest(i);
            let mut alternatives: Vec<Vec<Judgement>> = Vec::new();
            let extended = |f: &F| {
                let mut formulas = others.clone();
                formulas.push(f.clone());
                formulas
            };
            match &hypotheses[i] {
                F::With(a, b) => {
                    for side in [a, b] {
                        alternatives.push(vec![(extended(side), goal.clone(), budget)]);
                    }
                }
                F::Lolli(a, b) => {
                    for (l, r) in splits(&others) {
                        let mut r = r;
                        r.push((**b).clone());
                        alternatives
                            .push(vec![(l, (**a).clone(), budget), (r, goal.clone(), budget)]);
                    }
                }
                F::Bang(a) => {
                    alternatives.push(vec![(extended(a), goal.clone(), budget)]);
                    alternatives.push(vec![(others.clone(), goal.clone(), budget)]);
                    if budget > 0 {
                        let mut contracted = hypotheses.to_vec();
                        contracted.push(hypotheses[i].clone());
                        alternatives.push(vec![(contracted, goal.clone(), budget - 1)]);
                    } else {
                        bang_failed = true;
                    }
                }
                F::Forall(a) => {
                    let v = self.bound.len();
                    self.bound.push(None);
                    let premise = (extended(&open(a, &T::Var(v), 0)), goal.clone(), budget);
                    if self.premises_two_sided(stack, vec![premise], mark)? {
                        return Ok(true);
                    }
                }
                _ => {}
            }
            if self.mode.affine {
                alternatives.push(vec![(others.clone(), goal.clone(), budget)]);
            }
            for premises in alternatives {
                if self.premises_two_sided(stack, premises, mark)? {
                    return Ok(true);
                }
            }
        }
        if bang_failed {
            self.wanted = true;
        }
        Ok(false)
    }

    /// Pushes the premises of a two-sided rule, the first last, so it is
    /// proved first, and proves the stack; on a failure takes them off
    /// again and undoes what was bound since `mark`.
    fn premises_two_sided(
        &mut self,
        stack: &mut Vec<Judgement>,
        premises: Vec<Judgement>,
        mark: (usize, usize, u32),
    ) -> Result<bool, GaveUp> {
        let height = stack.len();
        stack.extend(premises.into_iter().rev());
        let found = self.solve_two_sided(stack);
        if !matches!(found, Ok(true)) {
            stack.truncate(height);
            self.undo(mark);
        }
        found
    }
}

/// Every way to split the formulas into two lists, in order.
fn splits(formulas: &[F]) -> impl Iterator<Item = (Vec<F>, Vec<F>)> + '_ {
    (0..1u64 << formulas.len()).map(move |mask| {
        let (mut l, mut r) = (Vec::new(), Vec::new());
        for (k, f) in formulas.iter().enumerate() {
            if mask >> k & 1 == 1 {
                l.push(f.clone());
            } else {
                r.push(f.clone());
            }
        }
        (l, r)
    })
}

/// Whether the eigenvariable `e` occurs in a resolved term.
fn contains(t: &T, e: u32) -> bool {
    match t {
        T::Eigen(x) => *x == e,
        T::App(_, args) => args.iter().any(|a| contains(a, e)),
        T::Bound(_) | T::Var(_) => false,
    }
}

#[cfg(feature = "parse")]
pub(crate) mod tests {
    use super::*;
    use crate::Error;
    use crate::search::generate::Rng;
    use crate::search::{Engine, Options, Verdict, prove_within};

    /// Decides `text` in `mode` within `copies`.
    fn decide(text: &str, mode: Mode, copies: u32) -> Answer {
        let s: Sequent = text.parse().unwrap_or_else(|e| panic!("{text:?}: {e}"));
        sequent(&s, mode, copies)
    }

    /// The reference proves what the calculus proves and refutes what it
    /// does not, the eigenvariable condition included: the order of the
    /// quantifiers, a witness that must be the same on both sides of a
    /// `⊗`, a premise asked for its second answer, and the occurs check.
    #[test]
    fn decides_the_textbook_cases() {
        let classical = Mode::CLASSICAL;
        for (text, expected) in [
            ("forall x. p(x) |- p(a)", Answer::Proved),
            ("p(a) |- exists x. p(x)", Answer::Proved),
            ("forall x. p(x) |- exists x. p(x)", Answer::Proved),
            ("exists x. p(x) |- forall x. p(x)", Answer::Unprovable),
            (
                "exists y. forall x. r(x, y) |- forall x. exists y. r(x, y)",
                Answer::Proved,
            ),
            (
                "forall x. exists y. r(x, y) |- exists y. forall x. r(x, y)",
                Answer::Unprovable,
            ),
            (
                "p(a) & p(b), q(b) |- exists x. (p(x) * q(x))",
                Answer::Proved,
            ),
            (
                "p(a) & p(b), q(c) |- exists x. (p(x) * q(x))",
                Answer::Unprovable,
            ),
            ("|- exists x. (~p(x) par p(f(x)))", Answer::Unprovable),
            ("|- forall x. exists y. (~p(x) par p(y))", Answer::Proved),
            ("forall x. p(x) |- p(a) * p(b)", Answer::Unprovable),
        ] {
            assert_eq!(decide(text, classical, 0), expected, "{text}");
        }
        assert_eq!(
            decide("!forall x. p(x) |- p(a) * p(b)", classical, 1),
            Answer::Proved
        );
        assert_eq!(
            decide(
                "p(a), !forall x. (p(x) -o p(s(x))) |- p(s(s(a)))",
                classical,
                1
            ),
            Answer::Proved
        );
    }

    /// A random term over the signature: a variable in scope, a constant,
    /// or `f` applied to a term.
    fn random_term(rng: &mut Rng, scope: &[String], depth: u32) -> String {
        match rng.below(4) {
            0 | 1 if !scope.is_empty() => scope[rng.below(scope.len())].clone(),
            2 if depth > 0 => format!("f({})", random_term(rng, scope, depth - 1)),
            _ => ["a", "b"][rng.below(2)].to_owned(),
        }
    }

    /// A random formula of a one-sided sequent over the predicates `p`
    /// (unary) and `q` (binary), of at most `depth` connectives deep,
    /// with the units, additives and exponentials as `rules` allow.
    fn random_formula(
        rng: &mut Rng,
        scope: &mut Vec<String>,
        depth: u32,
        rules: [bool; 3],
    ) -> String {
        let [units, additives, exponentials] = rules;
        let atom = |rng: &mut Rng, scope: &[String]| {
            let sign = if rng.one_in(2) { "~" } else { "" };
            if rng.one_in(2) {
                format!("{sign}p({})", random_term(rng, scope, 1))
            } else {
                format!(
                    "{sign}q({}, {})",
                    random_term(rng, scope, 1),
                    random_term(rng, scope, 1)
                )
            }
        };
        if depth == 0 {
            return atom(rng, scope);
        }
        match rng.below(10) {
            0 | 1 => atom(rng, scope),
            2 => format!(
                "({} * {})",
                random_formula(rng, scope, depth - 1, rules),
                random_formula(rng, scope, depth - 1, rules)
            ),
            3 => format!(
                "({} par {})",
                random_formula(rng, scope, depth - 1, rules),
                random_formula(rng, scope, depth - 1, rules)
            ),
            4 | 5 => {
                let name = format!("x{}", scope.len());
                scope.push(name.clone());
                let body = random_formula(rng, scope, depth - 1, rules);
                scope.pop();
                let q = if rng.one_in(2) { "forall" } else { "exists" };
                format!("({q} {name}. {body})")
            }
            6 if additives => format!(
                "({} & {})",
                random_formula(rng, scope, depth - 1, rules),
                random_formula(rng, scope, depth - 1, rules)
            ),
            7 if additives => format!(
                "({} + {})",
                random_formula(rng, scope, depth - 1, rules),
                random_formula(rng, scope, depth - 1, rules)
            ),
            8 if exponentials => {
                let e = if rng.one_in(2) { "!" } else { "?" };
                format!("{e}{}", random_formula(rng, scope, depth - 1, rules))
            }
            9 if units => ["1", "bot", "top", "0"][rng.below(4)].to_owned(),
            _ => atom(rng, scope),
        }
    }

    /// A random one-sided sequent of two or three formulas, or a formula
    /// against its own negation with one quantifier, constant or literal
    /// changed, which keeps many of them provable.
    pub(crate) fn random_sequent(rng: &mut Rng, rules: [bool; 3]) -> String {
        if rng.one_in(2) {
            let f = random_formula(rng, &mut Vec::new(), 3, rules);
            // A change of one character keeps the text well formed.
            let mut g = f.clone();
            match rng.below(5) {
                0 => g = g.replacen("forall", "exists", 1),
                1 => g = g.replacen("exists", "forall", 1),
                2 => g = g.replacen("a)", "b)", 1),
                3 => g = g.replacen("p(", "q(a, ", 1),
                _ => {}
            }
            return format!("{f} |- {g}");
        }
        let n = 2 + rng.below(2);
        let formulas: Vec<String> = (0..n)
            .map(|_| random_formula(rng, &mut Vec::new(), 2, rules))
            .collect();
        format!("|- {}", formulas.join(", "))
    }

    /// The engine on generated first-order sequents, in the classical
    /// modes, with and without Mix, linear and affine, never contradicts
    /// the reference, and the reference decides most of them.
    #[test]
    fn the_focused_engine_agrees() {
        const COPIES: u32 = 1;
        let (mut cases, mut decided, mut judged) = (0, 0, 0);
        let (mut proved, mut engine_proved) = (0, 0);
        for (seed, rules) in [
            [false, false, false],
            [true, false, false],
            [false, true, false],
            [true, true, false],
            [false, false, true],
            [true, true, true],
        ]
        .into_iter()
        .enumerate()
        {
            let mut rng = Rng::new(4200 + seed as u64);
            for _ in 0..60 {
                let text = random_sequent(&mut rng, rules);
                let Ok(s) = text.parse::<Sequent>() else {
                    continue;
                };
                for mode in [
                    Mode::CLASSICAL,
                    Mode::CLASSICAL.with_mix(),
                    Mode::CLASSICAL.with_affine(),
                ] {
                    cases += 1;
                    let reference = sequent(&s, mode, COPIES);
                    if reference == Answer::Unknown {
                        continue;
                    }
                    decided += 1;
                    proved += usize::from(reference == Answer::Proved);
                    for options in [
                        Options::default().with_copies(Some(COPIES)),
                        Options::default()
                            .with_copies(Some(COPIES))
                            .with_engine(Some(Engine::Focus)),
                    ] {
                        let mut left = 1u64 << 14;
                        let stop = |_| {
                            left = left.saturating_sub(1);
                            left == 0
                        };
                        let outcome =
                            match prove_within(&s, mode, &options, &crate::Limits::default(), stop)
                            {
                                Ok(outcome) => outcome,
                                Err(Error::EngineRefused { .. } | Error::NoEngine { .. }) => {
                                    continue;
                                }
                                Err(e) => panic!("{text:?} in {mode} mode: {e}"),
                            };
                        let contradiction = match &outcome.verdict {
                            Verdict::Proved(_) => reference == Answer::Unprovable,
                            Verdict::Unprovable(_) => reference == Answer::Proved,
                            Verdict::Unknown(_) => false,
                        };
                        assert!(
                            !contradiction,
                            "{text:?} in {mode} mode: the engine answers {:?}, the reference {reference:?}",
                            outcome.verdict
                        );
                        judged += 1;
                        engine_proved += usize::from(matches!(outcome.verdict, Verdict::Proved(_)));
                    }
                }
            }
        }
        assert!(decided * 3 > cases * 2, "{decided} of {cases} decided");
        // Both answers occur, and the engine proves what it is given.
        assert!(
            proved * 10 > decided && (decided - proved) * 10 > decided && engine_proved > 0,
            "{proved} of {decided} proved, {engine_proved} of {judged} by the engine"
        );
    }

    /// The predicates of the two-sided tests, by number: name and arity.
    const PREDICATES: [(&str, usize); 3] = [("p", 1), ("q", 0), ("r", 2)];

    /// The function symbols of the two-sided tests, by number: name and
    /// arity; the generator takes the first three.
    const FUNCTIONS: [(&str, usize); 6] =
        [("a", 0), ("b", 0), ("f", 1), ("c", 0), ("s", 1), ("z", 0)];

    /// The names of the variables, by the number of binders outside
    /// theirs: the generator nests two quantifiers at most.
    const VARIABLES: [&str; 2] = ["x", "y"];

    /// A predicate (by its number) applied to terms.
    fn atom(p: u32, args: &[T]) -> F {
        F::Lit(true, p, args.to_vec())
    }

    /// A function symbol (by its number) applied to terms.
    fn app(s: u32, args: &[T]) -> T {
        T::App(s, args.to_vec())
    }

    /// `A ⊸ B`.
    fn lolli(a: F, b: F) -> F {
        F::Lolli(Box::new(a), Box::new(b))
    }

    /// `A & B`.
    fn with(a: F, b: F) -> F {
        F::With(Box::new(a), Box::new(b))
    }

    /// `∀x.A`.
    fn all(a: F) -> F {
        F::Forall(Box::new(a))
    }

    /// `∃x.A`.
    fn ex(a: F) -> F {
        F::Exists(Box::new(a))
    }

    /// The two-sided reference proves what intuitionistic first-order
    /// linear logic proves and refutes what it does not: the quantifiers
    /// shifted across `⊸` one way and not the other, their order, a
    /// witness that both premises of `&R` share, a premise asked for its
    /// second answer, the occurs check, a witness left open, copies of a
    /// quantified hypothesis under `!`, and weakening of hypotheses alone
    /// in affine mode.
    #[test]
    fn decides_the_intuitionistic_textbook_cases() {
        use Answer::{Proved, Unknown, Unprovable};
        let (a, b, z) = (app(0, &[]), app(1, &[]), app(5, &[]));
        let (f, s) = (
            |t: &T| app(2, std::slice::from_ref(t)),
            |t: &T| app(4, std::slice::from_ref(t)),
        );
        let (x, y) = (T::Bound(0), T::Bound(1));
        let (linear, affine) = (Mode::INTUITIONISTIC, Mode::INTUITIONISTIC.with_affine());
        let p = |t: &T| atom(0, std::slice::from_ref(t));
        let q = || atom(1, &[]);
        let r = |t: &T, u: &T| atom(2, &[t.clone(), u.clone()]);
        let (bang, tensor) = (
            |a: F| F::Bang(Box::new(a)),
            |a: F, b: F| F::Tensor(Box::new(a), Box::new(b)),
        );
        let step = || bang(all(lolli(p(&x), p(&s(&x)))));
        for (hypotheses, goal, mode, copies, expected) in [
            // ∃x.(p(x) ⊸ q) ⊢ (∀x.p(x)) ⊸ q, and not its converse.
            (
                vec![ex(lolli(p(&x), q()))],
                lolli(all(p(&x)), q()),
                linear,
                0,
                Proved,
            ),
            (
                vec![lolli(all(p(&x)), q())],
                ex(lolli(p(&x), q())),
                linear,
                0,
                Unprovable,
            ),
            // ∃y.∀x.r(x, y) ⊢ ∀x.∃y.r(x, y), and not its converse.
            (
                vec![ex(all(r(&x, &y)))],
                all(ex(r(&y, &x))),
                linear,
                0,
                Proved,
            ),
            (
                vec![all(ex(r(&y, &x)))],
                ex(all(r(&x, &y))),
                linear,
                0,
                Unprovable,
            ),
            // A witness that both premises of `&R` share.
            (
                vec![with(p(&a), r(&a, &a))],
                ex(with(p(&x), r(&x, &x))),
                linear,
                0,
                Proved,
            ),
            (
                vec![with(p(&a), r(&b, &b))],
                ex(with(p(&x), r(&x, &x))),
                linear,
                0,
                Unprovable,
            ),
            (
                vec![with(p(&a), r(&b, &b))],
                with(ex(p(&x)), ex(r(&x, &x))),
                linear,
                0,
                Proved,
            ),
            // The first answer of `p(a) & p(b) ⊢ p(x)`, `a`, fails `r(x, x)`.
            (
                vec![with(p(&a), p(&b)), r(&b, &b)],
                ex(tensor(p(&x), r(&x, &x))),
                linear,
                0,
                Proved,
            ),
            // The occurs check.
            (
                vec![all(r(&x, &f(&x)))],
                ex(r(&x, &x)),
                linear,
                0,
                Unprovable,
            ),
            (vec![all(p(&x))], ex(p(&x)), linear, 0, Proved),
            (vec![all(p(&x))], all(p(&x)), linear, 0, Proved),
            (vec![ex(p(&x))], all(p(&x)), linear, 0, Unprovable),
            (vec![ex(p(&x))], p(&a), linear, 0, Unprovable),
            (
                vec![all(with(p(&x), r(&x, &x)))],
                with(p(&a), r(&b, &b)),
                linear,
                0,
                Proved,
            ),
            // !∀x.(p(x) ⊸ p(s(x))), p(z) ⊢ p(s(s(z))) takes one contraction.
            (vec![step(), p(&z)], p(&s(&s(&z))), linear, 1, Proved),
            (vec![step(), p(&z)], p(&s(&s(&z))), linear, 0, Unknown),
            (
                vec![all(p(&x))],
                tensor(p(&a), p(&b)),
                linear,
                2,
                Unprovable,
            ),
            // `!R` where every hypothesis is a `!` formula, and nowhere else.
            (vec![bang(all(p(&x)))], bang(p(&a)), linear, 0, Proved),
            (vec![all(p(&x))], bang(p(&a)), linear, 0, Unprovable),
            // Affine: the hypotheses are weakened, the goal never.
            (vec![all(p(&x)), q()], p(&a), linear, 0, Unprovable),
            (vec![all(p(&x)), q()], p(&a), affine, 0, Proved),
            (vec![p(&a)], F::One, affine, 0, Proved),
            (vec![p(&a)], q(), affine, 0, Unprovable),
        ] {
            assert_eq!(
                intuitionistic(&hypotheses, &goal, mode, copies),
                expected,
                "{} in {mode} within {copies}",
                ill_text(&hypotheses, &goal)
            );
        }
    }

    /// The text of a closed term under `scope` binders.
    fn term_text(t: &T, scope: usize) -> String {
        match t {
            T::Bound(i) => VARIABLES[scope - 1 - *i as usize].to_owned(),
            T::App(s, args) => applied(FUNCTIONS[*s as usize].0, args, scope),
            T::Var(_) | T::Eigen(_) => unreachable!("a written term is closed"),
        }
    }

    /// A symbol applied to terms, in the text syntax: bare when there
    /// are none.
    fn applied(name: &str, args: &[T], scope: usize) -> String {
        if args.is_empty() {
            return name.to_owned();
        }
        let args: Vec<String> = args.iter().map(|t| term_text(t, scope)).collect();
        format!("{name}({})", args.join(", "))
    }

    /// The intuitionistic text of a formula under `scope` binders, every
    /// connective but `!` in brackets.
    fn ill_formula_text(f: &F, scope: usize) -> String {
        let binary = |a: &F, op: &str, b: &F| {
            format!(
                "({} {op} {})",
                ill_formula_text(a, scope),
                ill_formula_text(b, scope)
            )
        };
        match f {
            F::Lit(_, p, args) => applied(PREDICATES[*p as usize].0, args, scope),
            F::One => "1".to_owned(),
            F::Top => "top".to_owned(),
            F::Zero => "0".to_owned(),
            F::Tensor(a, b) => binary(a, "*", b),
            F::Lolli(a, b) => binary(a, "-o", b),
            F::With(a, b) => binary(a, "&", b),
            F::Plus(a, b) => binary(a, "+", b),
            F::Bang(a) => format!("!{}", ill_formula_text(a, scope)),
            F::Forall(a) | F::Exists(a) => {
                let q = if matches!(f, F::Forall(_)) {
                    "forall"
                } else {
                    "exists"
                };
                format!(
                    "({q} {}. {})",
                    VARIABLES[scope],
                    ill_formula_text(a, scope + 1)
                )
            }
            F::Bot | F::Par(..) | F::Quest(_) => unreachable!("no intuitionistic connective"),
        }
    }

    /// The intuitionistic text of `hypotheses ⊢ goal`.
    fn ill_text(hypotheses: &[F], goal: &F) -> String {
        let hypotheses: Vec<String> = hypotheses.iter().map(|h| ill_formula_text(h, 0)).collect();
        format!("{} |- {}", hypotheses.join(", "), ill_formula_text(goal, 0))
    }

    /// A random closed term under `scope` binders: a variable in scope,
    /// `a`, `b`, or `f` applied to a term, `f` at most `depth` deep.
    fn random_closed_term(rng: &mut Rng, scope: usize, depth: u32) -> T {
        match rng.below(5) {
            0..=2 if scope > 0 => T::Bound(rng.below(scope) as u32),
            3 if depth > 0 => app(2, &[random_closed_term(rng, scope, depth - 1)]),
            _ => app(rng.below(2) as u32, &[]),
        }
    }

    /// A random intuitionistic formula under `scope` binders, at most
    /// `depth` connectives deep and two quantifiers nested, with the
    /// additives (and `⊤`, `0`) and `!` as `rules` allow.
    fn random_ill(rng: &mut Rng, scope: usize, depth: u32, rules: [bool; 2]) -> F {
        let [additives, exponentials] = rules;
        let atom = |rng: &mut Rng| {
            // `q`, which has no argument, less often.
            let p = [0, 0, 1, 2, 2][rng.below(5)];
            let args: Vec<T> = (0..PREDICATES[p].1)
                .map(|_| random_closed_term(rng, scope, 1))
                .collect();
            F::Lit(true, p as u32, args)
        };
        if depth == 0 {
            return atom(rng);
        }
        let sub = |rng: &mut Rng| Box::new(random_ill(rng, scope, depth - 1, rules));
        match rng.below(12) {
            2 => F::Tensor(sub(rng), sub(rng)),
            3 | 4 => F::Lolli(sub(rng), sub(rng)),
            5 | 6 if scope < VARIABLES.len() => {
                let body = Box::new(random_ill(rng, scope + 1, depth - 1, rules));
                if rng.one_in(2) {
                    F::Forall(body)
                } else {
                    F::Exists(body)
                }
            }
            7 if additives => F::With(sub(rng), sub(rng)),
            8 if additives => F::Plus(sub(rng), sub(rng)),
            9 if exponentials => F::Bang(sub(rng)),
            10 if additives => [F::One, F::Top, F::Zero][rng.below(3)].clone(),
            10 => F::One,
            _ => atom(rng),
        }
    }

    /// The formula with its first site of a kind changed, once `done` is
    /// not yet set: a `∀` made `∃` (kind 0), an `∃` made `∀` (1), the
    /// constant `a` made `b` (2), a `⊗` made `⊸` (3); any other kind
    /// changes nothing.
    fn changed(f: &F, kind: usize, done: &mut bool) -> F {
        if *done {
            return f.clone();
        }
        let mut sub = |g: &F| Box::new(changed(g, kind, done));
        match (f, kind) {
            (F::Forall(a), 0) | (F::Exists(a), 1) => {
                *done = true;
                if kind == 0 {
                    F::Exists(a.clone())
                } else {
                    F::Forall(a.clone())
                }
            }
            (F::Tensor(a, b), 3) => {
                *done = true;
                F::Lolli(a.clone(), b.clone())
            }
            (F::Lit(sign, p, args), 2) => {
                let args = args.iter().map(|t| changed_term(t, done)).collect();
                F::Lit(*sign, *p, args)
            }
            (F::Tensor(a, b), _) => {
                let a = sub(a);
                F::Tensor(a, sub(b))
            }
            (F::Lolli(a, b), _) => {
                let a = sub(a);
                F::Lolli(a, sub(b))
            }
            (F::With(a, b), _) => {
                let a = sub(a);
                F::With(a, sub(b))
            }
            (F::Plus(a, b), _) => {
                let a = sub(a);
                F::Plus(a, sub(b))
            }
            (F::Bang(a), _) => F::Bang(sub(a)),
            (F::Forall(a), _) => F::Forall(sub(a)),
            (F::Exists(a), _) => F::Exists(sub(a)),
            _ => f.clone(),
        }
    }

    /// The term with its first `a` made `b`, once `done` is not yet set.
    fn changed_term(t: &T, done: &mut bool) -> T {
        match t {
            T::App(0, args) if args.is_empty() && !*done => {
                *done = true;
                app(1, &[])
            }
            T::App(s, args) => T::App(*s, args.iter().map(|a| changed_term(a, done)).collect()),
            other => other.clone(),
        }
    }

    /// A random two-sided intuitionistic sequent: a formula against
    /// itself with one site changed, `A, A ⊸ B ⊢ B` with one site of the
    /// goal changed (both keep many of them provable), or one to three
    /// random hypotheses and a random goal.
    /// A random two-sided sequent as [`random_ill_sequent`] makes one, in
    /// the text syntax.
    pub(crate) fn random_ill_text(rng: &mut Rng, rules: [bool; 2]) -> String {
        let (hypotheses, goal) = random_ill_sequent(rng, rules);
        ill_text(&hypotheses, &goal)
    }

    fn random_ill_sequent(rng: &mut Rng, rules: [bool; 2]) -> (Vec<F>, F) {
        match rng.below(3) {
            0 => {
                let f = random_ill(rng, 0, 3, rules);
                let goal = changed(&f, rng.below(5), &mut false);
                (vec![f], goal)
            }
            1 => {
                let a = random_ill(rng, 0, 2, rules);
                let b = random_ill(rng, 0, 2, rules);
                let goal = changed(&b, rng.below(5), &mut false);
                (vec![a.clone(), lolli(a, b)], goal)
            }
            _ => {
                let n = 1 + rng.below(3);
                let hypotheses = (0..n).map(|_| random_ill(rng, 0, 2, rules)).collect();
                (hypotheses, random_ill(rng, 0, 2, rules))
            }
        }
    }

    /// The two-sided engine on generated first-order intuitionistic
    /// sequents, linear and affine, by the dispatch and forced, never
    /// contradicts the two-sided reference, every proof it returns passes
    /// the checker, and the reference decides most of them.
    #[test]
    fn the_two_sided_engine_agrees() {
        const COPIES: u32 = 1;
        let (mut cases, mut decided, mut proved) = (0, 0, 0);
        let (mut engine_proved, mut engine_refuted) = (0, 0);
        for (seed, rules) in [[false, false], [true, false], [false, true], [true, true]]
            .into_iter()
            .enumerate()
        {
            let mut rng = Rng::new(4300 + seed as u64);
            for _ in 0..50 {
                let (hypotheses, goal) = random_ill_sequent(&mut rng, rules);
                let text = ill_text(&hypotheses, &goal);
                let s: Sequent = text.parse().unwrap_or_else(|e| panic!("{text:?}: {e}"));
                for mode in [Mode::INTUITIONISTIC, Mode::INTUITIONISTIC.with_affine()] {
                    cases += 1;
                    let reference = intuitionistic(&hypotheses, &goal, mode, COPIES);
                    decided += usize::from(reference != Answer::Unknown);
                    proved += usize::from(reference == Answer::Proved);
                    for engine in [None, Some(Engine::TwoSided)] {
                        let options = Options::default()
                            .with_copies(Some(COPIES))
                            .with_engine(engine)
                            .with_jobs(1);
                        let mut left = 1u64 << 14;
                        let stop = |_| {
                            left = left.saturating_sub(1);
                            left == 0
                        };
                        let outcome =
                            prove_within(&s, mode, &options, &crate::Limits::default(), stop)
                                .unwrap_or_else(|e| panic!("{text:?} in {mode} mode: {e}"));
                        let contradiction = match &outcome.verdict {
                            Verdict::Proved(proof) => {
                                assert_eq!(proof.check(mode), Ok(()), "{text:?} in {mode} mode");
                                engine_proved += 1;
                                reference == Answer::Unprovable
                            }
                            Verdict::Unprovable(_) => {
                                engine_refuted += 1;
                                reference == Answer::Proved
                            }
                            Verdict::Unknown(_) => false,
                        };
                        assert!(
                            !contradiction,
                            "{text:?} in {mode} mode by {:?}: the engine answers {:?}, the reference {reference:?}",
                            outcome.engine, outcome.verdict
                        );
                    }
                }
            }
        }
        assert!(decided * 3 > cases * 2, "{decided} of {cases} decided");
        // Both answers occur, and the engine proves and refutes.
        assert!(
            proved * 10 > decided
                && (decided - proved) * 10 > decided
                && engine_proved > 0
                && engine_refuted > 0,
            "{proved} of {decided} proved; the engine proved {engine_proved}, refuted {engine_refuted}"
        );
    }
}
