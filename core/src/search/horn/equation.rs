// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The state equation of a net: a marking `M` reached from `M₀` by firing
//! each transition `t` some `xₜ` times is `M = M₀ + Σ xₜ·Cₜ`, `Cₜ` the
//! transition's effect, and a marking that covers the target is at least
//! the target. When no rational `x ≥ 0` solves that, Farkas' lemma gives a
//! weight per place, `y`, such that no transition raises the weighted
//! count of a marking (`y·Cₜ ≤ 0`) while the target asks it raised
//! (`y·(M − M₀) > 0`), and in affine mode no weight is negative: then no
//! firing sequence reaches, or covers, the target.
//!
//! A simplex in floating point proposes the weights; [`certify`] checks
//! them in exact integer arithmetic, and only a vector it accepts refutes,
//! so a rounding error of the simplex costs a refutation and never makes
//! a wrong one. The simplex runs beside a search, a slice at a time, its
//! work kept in step with the search's: most nets of practice are
//! decided by the search long before the tableau would be worth its
//! cost.

use super::Program;
use crate::search::Reason;
use crate::search::memory::{Account, Charged};

/// The entries of the simplex's basis and columns it may touch for each
/// unit of work the search beside it has done in affine mode: a unit is a
/// marking computed, an element compared or a lookup, which takes some
/// sixteen times an entry's update, so the simplex has about half the
/// time. A coverability problem from verification is mostly safe, and the
/// equation refutes most of those.
const AFFINE_ENTRIES_PER_UNIT: u64 = 16;

/// The same in linear mode, where a unit is a transition examined or a
/// marking written: about a fifth of the time, beside a search that on
/// the nets of practice mostly proves the goal.
const LINEAR_ENTRIES_PER_UNIT: u64 = 4;

/// What the tolerance of the simplex counts as zero. The tableau starts
/// with small integers, and what the simplex gets wrong by more than this
/// only costs a refutation, since the weights it proposes are checked
/// exactly.
const EPSILON: f64 = 1e-9;

/// The largest denominator a weight is read as, before the weights are
/// brought to integers: the weights of a vertex are ratios of
/// determinants of the tableau, small on nets of small weights.
const MOST_DENOMINATOR: i64 = 1 << 20;

/// The largest common denominator of the weights: a fraction `n/d` of
/// the weights' range has `|n| ≤ d`, so its integer `n · (common / d)` is
/// at most the common denominator, 2⁴⁰, which an `i64` holds.
const MOST_COMMON_DENOMINATOR: i64 = 1 << 40;

/// The state equation of a program, solved a slice at a time.
pub(super) struct Equation<'a> {
    /// Whether the target need only be covered.
    affine: bool,
    /// Where the simplex stands.
    state: State,
    /// The tableau's entries touched so far.
    spent: u64,
    /// The tableau's memory, charged to the search's account.
    charged: Charged<'a>,
    /// The weights that refute the target, once found and checked: a
    /// place and its weight, for the places of the tableau's rows.
    certificate: Option<Vec<(u32, i64)>>,
}

/// Where the simplex stands.
enum State {
    /// Not started.
    Waiting,
    /// Pivoting.
    Solving(Tableau),
    /// Its tableau given back to a search that ran out of room; it starts
    /// again only after the search, from [`Equation::finish`].
    Yielded,
    /// Done: refuted, solved, or given up.
    Done,
}

impl<'a> Equation<'a> {
    /// The state equation of a program not yet looked at, whose tableau
    /// will be charged to `account`.
    pub(super) fn new(affine: bool, account: &'a Account) -> Self {
        Self {
            affine,
            state: State::Waiting,
            spent: 0,
            charged: Charged::new(account),
            certificate: None,
        }
    }

    /// The entries the simplex may touch for the search's `work`.
    pub(super) fn budget(&self, work: u64) -> u64 {
        let per_unit = if self.affine {
            AFFINE_ENTRIES_PER_UNIT
        } else {
            LINEAR_ENTRIES_PER_UNIT
        };
        work.saturating_mul(per_unit)
    }

    /// Whether the simplex would run within `budget` entries: it is
    /// neither done nor yielded and has spent less.
    pub(super) fn wants(&self, budget: u64) -> bool {
        !matches!(self.state, State::Done | State::Yielded) && self.spent < budget
    }

    /// Gives the tableau's memory back, for a search that has no room
    /// left: returns whether there was a tableau to give. The search never
    /// loses a decision to the simplex's memory.
    pub(super) fn release(&mut self) -> bool {
        if !matches!(self.state, State::Solving(_)) {
            return false;
        }
        self.state = State::Yielded;
        self.charged = Charged::new(self.charged.account());
        true
    }

    /// Runs the simplex to its end after a search that ran out of room and
    /// gave its memory back, starting afresh if it had yielded; returns
    /// what [`run`](Self::run) does.
    pub(super) fn finish(
        &mut self,
        program: &Program,
        stop: &mut dyn FnMut() -> bool,
    ) -> Result<bool, Reason> {
        if let State::Yielded = self.state {
            self.state = State::Waiting;
        }
        self.run(program, u64::MAX, stop)
    }

    /// Runs the simplex until it has touched `budget` entries in all, or
    /// is done; returns whether the weights it found refute the target,
    /// or that `stop` fired, which it polls at every pivot. A tableau the
    /// memory bound has no room for, a solution, and a pivot limit end it
    /// without a refutation.
    pub(super) fn run(
        &mut self,
        program: &Program,
        budget: u64,
        stop: &mut dyn FnMut() -> bool,
    ) -> Result<bool, Reason> {
        if let State::Waiting = self.state {
            // The set-up reads every arc and lays out a basis of up to a
            // row per place, and a pivot touches it twice: the simplex
            // waits until the search has done as much as both.
            let places = program.places as u64;
            let set_up = places
                .saturating_mul(places)
                .saturating_mul(3)
                .saturating_add(program.arcs.len() as u64);
            if budget < set_up {
                return Ok(false);
            }
            self.state = match Tableau::new(program, self.affine, &mut self.charged) {
                Some(tableau) => {
                    self.spent = self.spent.saturating_add(tableau.entries());
                    State::Solving(tableau)
                }
                // No room beside the search: it may have after the
                // search, from `finish`.
                None => State::Yielded,
            };
        }
        let State::Solving(tableau) = &mut self.state else {
            return Ok(false);
        };
        let weights = loop {
            // A pivot is made only within the budget.
            if self.spent.saturating_add(tableau.entries()) > budget {
                return Ok(false);
            }
            if stop() {
                return Err(Reason::Stopped);
            }
            match tableau.pivot() {
                Step::Pivoted => self.spent = self.spent.saturating_add(tableau.entries()),
                Step::Optimal => break tableau.weights(),
                Step::GiveUp => break None,
            }
        };
        // The tableau goes, and what it was charged with it.
        self.state = State::Done;
        self.charged = Charged::new(self.charged.account());
        self.certificate = weights
            .and_then(|weights| integers(&weights))
            .filter(|weights| certify(program, self.affine, weights));
        Ok(self.certificate.is_some())
    }

    /// The weights that refute the target, a place and its weight, the
    /// places not named weighing nothing, if the simplex found them and
    /// [`certify`] accepted them.
    pub(super) fn certificate(&self) -> Option<&[(u32, i64)]> {
        self.certificate.as_deref()
    }
}

/// What a pivot came to.
enum Step {
    /// The basis changed.
    Pivoted,
    /// No column improves the objective.
    Optimal,
    /// The pivots reached their limit.
    GiveUp,
}

/// The first phase of the revised simplex for `A·x (= or ≥) b`, `x ≥ 0`:
/// one row per place the net touches, each signed so that its right-hand
/// side is not negative; a column per distinct effect of a transition, in
/// affine mode a surplus column per row, and an artificial column per row,
/// whose sum is the objective, zero exactly when the equation has a
/// solution. The basis is kept as its dense inverse and the columns
/// sparse, so a pivot costs the square of the rows and a pass over the
/// columns' entries, not the rows times the columns: a net of practice has
/// hundreds of places and tens of thousands of transitions.
struct Tableau {
    /// The columns of the transitions' effects, each its rows and entries,
    /// signed as the rows are.
    columns: Vec<Vec<(u32, f64)>>,
    /// The place of each row, and its sign.
    places: Vec<(u32, f64)>,
    /// Whether each row has a surplus column, for `≥`.
    surplus: bool,
    /// The inverse of the basis, row after row.
    inverse: Vec<f64>,
    /// The values of the basic variables, one per row.
    values: Vec<f64>,
    /// The basic column of each row: a transition's below the number of
    /// columns, then the surplus columns, then the artificial ones.
    basis: Vec<usize>,
    /// The prices of the rows, the objective's coefficients of the basis
    /// times its inverse: the weights, at the optimum.
    prices: Vec<f64>,
    /// The entering column times the inverse.
    column: Vec<f64>,
    /// The pivots made.
    pivots: u64,
    /// The most pivots before the simplex gives up: in floating point a
    /// simplex that has not ended after this many is taken to cycle on
    /// rounding.
    most_pivots: u64,
    /// The pivots in a row that did not move the objective; above
    /// [`STALLED`], Bland's rule chooses the columns, which ends in exact
    /// arithmetic.
    stalled: u32,
}

/// The pivots in a row without progress after which the simplex chooses
/// by Bland's rule (the first improving column) instead of the steepest
/// one, which may cycle.
const STALLED: u32 = 64;

impl Tableau {
    /// The simplex of a program's state equation, or `None` when the memory
    /// bound has no room for its basis; charged to `charged`.
    fn new(program: &Program, affine: bool, charged: &mut Charged<'_>) -> Option<Self> {
        let places = program.places;
        // The effect of each transition on each place it changes, each
        // distinct effect once.
        let mut effects: Vec<Vec<(u32, i64)>> = Vec::new();
        let mut seen = crate::hash::HashSet::default();
        let mut touched = vec![false; places];
        for transition in &program.transitions {
            // The inputs and the outputs are each sorted by place: one merge.
            let inputs = &program.arcs[transition.inputs as usize..transition.outputs as usize];
            let outputs = &program.arcs[transition.outputs as usize..transition.end as usize];
            let mut effect: Vec<(u32, i64)> = Vec::with_capacity(inputs.len() + outputs.len());
            let (mut i, mut o) = (0, 0);
            while i < inputs.len() || o < outputs.len() {
                let (p, e) = match (inputs.get(i), outputs.get(o)) {
                    (Some(&(a, w)), Some(&(b, v))) if a == b => {
                        (i, o) = (i + 1, o + 1);
                        (a, i64::from(v) - i64::from(w))
                    }
                    (Some(&(a, w)), Some(&(b, _))) if a < b => {
                        i += 1;
                        (a, -i64::from(w))
                    }
                    (Some(&(a, w)), None) => {
                        i += 1;
                        (a, -i64::from(w))
                    }
                    (_, Some(&(b, v))) => {
                        o += 1;
                        (b, i64::from(v))
                    }
                    (None, None) => unreachable!("the loop's condition"),
                };
                if e != 0 {
                    effect.push((p, e));
                }
            }
            if !effect.is_empty() && seen.insert(effect.clone()) {
                for &(p, _) in &effect {
                    touched[p as usize] = true;
                }
                effects.push(effect);
            }
        }
        let goal = |p: usize| i64::from(program.target[p]) - i64::from(program.initial[p]);
        let rows: Vec<u32> = (0..places)
            .filter(|&p| touched[p] || goal(p) != 0)
            .map(|p| p as u32)
            .collect();
        let mut row_of = vec![u32::MAX; places];
        for (r, &p) in rows.iter().enumerate() {
            row_of[p as usize] = r as u32;
        }
        let height = rows.len();
        let entries: usize = effects.iter().map(Vec::len).sum();
        let bytes = height
            .checked_mul(height)?
            .checked_add(height.checked_mul(4)?)?
            .checked_mul(size_of::<f64>())?
            .checked_add(entries.checked_mul(size_of::<(u32, f64)>())?)?;
        if !charged.account().fits(bytes) {
            return None;
        }
        charged.charge(bytes);
        let places: Vec<(u32, f64)> = rows
            .iter()
            .map(|&p| (p, if goal(p as usize) < 0 { -1.0 } else { 1.0 }))
            .collect();
        let columns = effects
            .into_iter()
            .map(|effect| {
                effect
                    .into_iter()
                    .map(|(p, e)| {
                        let r = row_of[p as usize];
                        (r, places[r as usize].1 * e as f64)
                    })
                    .collect()
            })
            .collect::<Vec<Vec<(u32, f64)>>>();
        let mut inverse = vec![0.0; height * height];
        for r in 0..height {
            inverse[r * height + r] = 1.0;
        }
        let structural = columns.len() + if affine { height } else { 0 };
        Some(Self {
            values: places
                .iter()
                .map(|&(p, sign)| sign * goal(p as usize) as f64)
                .collect(),
            basis: (0..height).map(|r| structural + r).collect(),
            prices: vec![1.0; height],
            column: vec![0.0; height],
            most_pivots: 64 * (structural as u64 + 2 * height as u64),
            columns,
            places,
            surplus: affine,
            inverse,
            pivots: 0,
            stalled: 0,
        })
    }

    /// The entries one pivot touches: the inverse twice, and every column
    /// once to price it.
    fn entries(&self) -> u64 {
        let height = self.places.len() as u64;
        let priced: usize = self.columns.iter().map(Vec::len).sum();
        2 * height * height + priced as u64 + height
    }

    /// The columns before the artificial ones.
    fn structural(&self) -> usize {
        self.columns.len() + if self.surplus { self.places.len() } else { 0 }
    }

    /// The reduced cost of a column: its cost, one for an artificial
    /// column, less the prices times its entries.
    fn reduced(&self, j: usize) -> f64 {
        let (n, structural) = (self.columns.len(), self.structural());
        if j < n {
            -self.columns[j]
                .iter()
                .map(|&(r, a)| self.prices[r as usize] * a)
                .sum::<f64>()
        } else if j < structural {
            let r = j - n;
            self.prices[r] * self.places[r].1
        } else {
            1.0 - self.prices[j - structural]
        }
    }

    /// Makes one pivot: the column of the most negative reduced cost
    /// enters, or the first such column after [`STALLED`] pivots without
    /// progress, and among the rows that bound it the one whose basic
    /// column comes first leaves.
    fn pivot(&mut self) -> Step {
        let height = self.places.len();
        let columns = self.structural() + height;
        let entering = if self.stalled < STALLED {
            (0..columns)
                .map(|j| (j, self.reduced(j)))
                .filter(|&(_, d)| d < -EPSILON)
                .min_by(|a, b| a.1.total_cmp(&b.1))
                .map(|(j, _)| j)
        } else {
            (0..columns).find(|&j| self.reduced(j) < -EPSILON)
        };
        let Some(q) = entering else {
            return Step::Optimal;
        };
        if self.pivots >= self.most_pivots {
            return Step::GiveUp;
        }
        // The entering column in the basis' terms.
        let (n, structural) = (self.columns.len(), self.structural());
        for (i, x) in self.column.iter_mut().enumerate() {
            let row = &self.inverse[i * height..(i + 1) * height];
            *x = if q < n {
                self.columns[q]
                    .iter()
                    .map(|&(r, a)| row[r as usize] * a)
                    .sum()
            } else if q < structural {
                let r = q - n;
                -row[r] * self.places[r].1
            } else {
                row[q - structural]
            };
        }
        let mut leaving: Option<(usize, f64)> = None;
        for i in 0..height {
            let a = self.column[i];
            if a > EPSILON {
                let ratio = self.values[i] / a;
                let better = match leaving {
                    None => true,
                    Some((best, least)) => {
                        ratio < least - EPSILON
                            || (ratio <= least + EPSILON && self.basis[i] < self.basis[best])
                    }
                };
                if better {
                    leaving = Some((i, ratio));
                }
            }
        }
        // The objective is bounded below by zero, so a column that
        // improves it is bounded by some row, but for rounding.
        let Some((p, ratio)) = leaving else {
            return Step::GiveUp;
        };
        self.pivots += 1;
        self.stalled = if ratio > EPSILON { 0 } else { self.stalled + 1 };
        let pivot = self.column[p];
        for x in &mut self.inverse[p * height..(p + 1) * height] {
            *x /= pivot;
        }
        self.values[p] /= pivot;
        let (before, rest) = self.inverse.split_at_mut(p * height);
        let (pivot_row, after) = rest.split_at_mut(height);
        let value = self.values[p];
        for (i, row) in before
            .chunks_exact_mut(height)
            .chain(after.chunks_exact_mut(height))
            .enumerate()
        {
            let i = if i < p { i } else { i + 1 };
            let factor = self.column[i];
            if factor != 0.0 {
                for (x, &y) in row.iter_mut().zip(pivot_row.iter()) {
                    *x -= factor * y;
                }
                self.values[i] -= factor * value;
            }
        }
        self.basis[p] = q;
        // The prices: the rows of the inverse whose basic column is
        // artificial, summed.
        self.prices.fill(0.0);
        for i in 0..height {
            if self.basis[i] >= structural {
                let row = &self.inverse[i * height..(i + 1) * height];
                for (y, &x) in self.prices.iter_mut().zip(row) {
                    *y += x;
                }
            }
        }
        Step::Pivoted
    }

    /// At the optimum, the weights the prices give each place when the
    /// objective is above zero, that is when the equation has no solution,
    /// with the row's sign. `None` when the equation has a solution.
    fn weights(&self) -> Option<Vec<(u32, f64)>> {
        let structural = self.structural();
        let objective: f64 = (0..self.places.len())
            .filter(|&i| self.basis[i] >= structural)
            .map(|i| self.values[i])
            .sum();
        if objective <= 1e3 * EPSILON {
            return None;
        }
        Some(
            self.places
                .iter()
                .zip(&self.prices)
                .map(|(&(p, sign), &y)| (p, sign * y))
                .collect(),
        )
    }
}

/// Brings weights in floating point to integers of the same ratios, each
/// read as the nearest fraction of a denominator up to
/// [`MOST_DENOMINATOR`] and all multiplied by the least common multiple of
/// the denominators; `None` when that passes [`MOST_COMMON_DENOMINATOR`].
/// The result is checked by [`certify`], so this need only be right where
/// it can.
fn integers(weights: &[(u32, f64)]) -> Option<Vec<(u32, i64)>> {
    // A weight that is no number would keep the continued fraction going.
    if weights.iter().any(|&(_, w)| !w.is_finite()) {
        return None;
    }
    let largest = weights.iter().map(|&(_, w)| w.abs()).fold(0.0, f64::max);
    if largest <= EPSILON {
        return None;
    }
    let fractions: Vec<(u32, i64, i64)> = weights
        .iter()
        .map(|&(p, w)| {
            let (n, d) = fraction(w / largest);
            (p, n, d)
        })
        .collect();
    let mut common = 1i64;
    for &(_, _, d) in &fractions {
        common = common
            .checked_mul(d / gcd(common, d))
            .filter(|&c| c <= MOST_COMMON_DENOMINATOR)?;
    }
    let mut integers: Vec<(u32, i64)> = fractions
        .into_iter()
        .map(|(p, n, d)| (p, n * (common / d)))
        .collect();
    let divisor = integers.iter().fold(0, |g, &(_, w)| gcd(g, w));
    if divisor > 1 {
        for (_, w) in &mut integers {
            *w /= divisor;
        }
    }
    Some(integers)
}

/// The fraction `n/d`, `0 < d ≤` [`MOST_DENOMINATOR`], nearest to `x` in
/// `[-1, 1]`, by its continued fraction; numbers within the tolerance of
/// zero are zero.
fn fraction(x: f64) -> (i64, i64) {
    if x.abs() <= EPSILON {
        return (0, 1);
    }
    let (sign, x) = if x < 0.0 { (-1, -x) } else { (1, x) };
    // The convergents h/k of the continued fraction of x.
    let (mut h, mut h_before, mut k, mut k_before) = (1i64, 0i64, 0i64, 1i64);
    let mut rest = x;
    loop {
        let a = rest.floor();
        // Both numerators and denominators stay at most the bound, and
        // `a` at most its reciprocal of the tolerance: no product wraps.
        let a = a as i64;
        let (h_next, k_next) = (a * h + h_before, a * k + k_before);
        if k_next > MOST_DENOMINATOR {
            break;
        }
        (h_before, h, k_before, k) = (h, h_next, k, k_next);
        let part = rest - a as f64;
        if part <= EPSILON || (x - h as f64 / k as f64).abs() <= EPSILON {
            break;
        }
        rest = 1.0 / part;
        if rest > 1.0 / EPSILON {
            break;
        }
    }
    if k == 0 { (0, 1) } else { (sign * h, k) }
}

/// The greatest common divisor of two numbers, not negative.
fn gcd(a: i64, b: i64) -> i64 {
    let (mut a, mut b) = (a.unsigned_abs(), b.unsigned_abs());
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a as i64
}

/// Whether integer weights, one for each place they name and zero for
/// the others, refute the target: no transition raises the weighted count
/// of a marking, the target's lies above the initial marking's, and in
/// affine mode no weight is negative. Exact: a weight is below 2⁶³ and an
/// arc's weight below 2³², so a product lies below 2⁹⁵, and a sum of the
/// fewer than 2³² terms of a transition or of the places lies below 2¹²⁷;
/// the arithmetic is checked all the same, and a sum it cannot hold is no
/// refutation.
pub(super) fn certify(program: &Program, affine: bool, weights: &[(u32, i64)]) -> bool {
    let mut y = vec![0i64; program.places];
    for &(p, w) in weights {
        y[p as usize] = w;
    }
    if affine && y.iter().any(|&w| w < 0) {
        return false;
    }
    let weighted = |arcs: &[(u32, u32)]| {
        arcs.iter().try_fold(0i128, |sum, &(p, w)| {
            i128::from(y[p as usize])
                .checked_mul(i128::from(w))
                .and_then(|term| sum.checked_add(term))
        })
    };
    for transition in &program.transitions {
        let inputs = &program.arcs[transition.inputs as usize..transition.outputs as usize];
        let outputs = &program.arcs[transition.outputs as usize..transition.end as usize];
        let raised = weighted(outputs)
            .zip(weighted(inputs))
            .and_then(|(o, i)| o.checked_sub(i));
        if !raised.is_some_and(|r| r <= 0) {
            return false;
        }
    }
    let asked = (0..program.places).try_fold(0i128, |sum, p| {
        let change = i128::from(program.target[p]) - i128::from(program.initial[p]);
        i128::from(y[p])
            .checked_mul(change)
            .and_then(|term| sum.checked_add(term))
    });
    asked.is_some_and(|a| a > 0)
}
