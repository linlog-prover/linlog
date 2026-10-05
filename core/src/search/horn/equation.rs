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

/// The tableau's entries the simplex may touch for each unit of work the
/// search beside it has done: a unit is a transition examined or a
/// marking written, which takes some sixteen times an entry's update.
pub(super) const ENTRIES_PER_UNIT: u64 = 16;

/// What the tolerance of the simplex counts as zero. The tableau starts
/// with small integers, and what the simplex gets wrong by more than this
/// only costs a refutation, since the weights it proposes are checked
/// exactly.
const EPSILON: f64 = 1e-9;

/// The largest denominator a weight is read as, before the weights are
/// brought to integers: the weights of a vertex are ratios of
/// determinants of the tableau, small on nets of small weights.
const MOST_DENOMINATOR: i64 = 1 << 20;

/// The largest common denominator of the weights: their integers then
/// stay below 2⁴⁰ · 2²⁰ = 2⁶⁰, which an `i64` holds.
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

    /// Whether the simplex would run within `budget` entries: it is not
    /// done and has spent less.
    pub(super) fn wants(&self, budget: u64) -> bool {
        !matches!(self.state, State::Done) && self.spent < budget
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
            self.state = match Tableau::new(program, self.affine, &mut self.charged) {
                Some(tableau) => {
                    self.spent = self.spent.saturating_add(tableau.entries());
                    State::Solving(tableau)
                }
                None => State::Done,
            };
        }
        let State::Solving(tableau) = &mut self.state else {
            return Ok(false);
        };
        let weights = loop {
            if self.spent >= budget {
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

/// The tableau of the first phase of the simplex for `A·x (= or ≥) b`,
/// `x ≥ 0`, one row per place the net touches, each signed so that its
/// right-hand side is not negative: the columns of the transitions'
/// effects, in affine mode a surplus column per row, an artificial column
/// per row, and the right-hand side. The objective is the sum of the
/// artificial variables, zero exactly when the equation has a solution.
struct Tableau {
    /// The rows, one after the other, `width` entries each.
    rows: Vec<f64>,
    /// The reduced cost of every column, and the objective's value negated
    /// last.
    cost: Vec<f64>,
    /// The entries of a row: the columns and the right-hand side.
    width: usize,
    /// The columns of the transitions and the surplus columns, before the
    /// artificial ones.
    structural: usize,
    /// The place of each row, and its sign.
    places: Vec<(u32, f64)>,
    /// The basic column of each row.
    basis: Vec<usize>,
    /// The pivots made.
    pivots: u64,
    /// The most pivots before the simplex gives up: Bland's rule ends in
    /// exact arithmetic, and a floating-point tableau that has not ended
    /// after this many is taken to cycle on rounding.
    most_pivots: u64,
}

impl Tableau {
    /// The tableau of a program's state equation, or `None` when the
    /// memory bound has no room for it; charged to `charged`.
    fn new(program: &Program, affine: bool, charged: &mut Charged<'_>) -> Option<Self> {
        let places = program.places;
        // The effect of each transition on each place it changes.
        let mut effects: Vec<Vec<(u32, i64)>> = Vec::new();
        let mut touched = vec![false; places];
        for transition in &program.transitions {
            let mut effect: Vec<(u32, i64)> = Vec::new();
            for &(p, w) in &program.arcs[transition.inputs as usize..transition.outputs as usize] {
                effect.push((p, -i64::from(w)));
            }
            for &(p, w) in &program.arcs[transition.outputs as usize..transition.end as usize] {
                match effect.iter_mut().find(|(q, _)| *q == p) {
                    Some((_, e)) => *e += i64::from(w),
                    None => effect.push((p, i64::from(w))),
                }
            }
            effect.retain(|&(_, e)| e != 0);
            if !effect.is_empty() {
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
        let structural = effects.len() + if affine { height } else { 0 };
        let width = structural.checked_add(height)?.checked_add(1)?;
        let bytes = height
            .checked_add(1)?
            .checked_mul(width)?
            .checked_mul(size_of::<f64>())?;
        if !charged.account().fits(bytes) {
            return None;
        }
        charged.charge(bytes);
        let mut table = vec![0.0; height * width];
        let mut signs = Vec::with_capacity(height);
        for (r, &p) in rows.iter().enumerate() {
            let sign = if goal(p as usize) < 0 { -1.0 } else { 1.0 };
            signs.push((p, sign));
            let row = &mut table[r * width..(r + 1) * width];
            row[width - 1] = sign * goal(p as usize) as f64;
            if affine {
                row[effects.len() + r] = -sign;
            }
            row[structural + r] = 1.0;
        }
        for (t, effect) in effects.iter().enumerate() {
            for &(p, e) in effect {
                let r = row_of[p as usize] as usize;
                table[r * width + t] = signs[r].1 * e as f64;
            }
        }
        // The reduced costs of the sum of the artificial variables over
        // the basis of the artificial columns: minus the column sums.
        let mut cost = vec![0.0; width];
        for r in 0..height {
            for (j, c) in cost.iter_mut().enumerate() {
                if j < structural || j == width - 1 {
                    *c -= table[r * width + j];
                }
            }
        }
        Some(Self {
            rows: table,
            cost,
            width,
            structural,
            places: signs,
            basis: (0..height).map(|r| structural + r).collect(),
            pivots: 0,
            most_pivots: 64 * (width as u64 + height as u64),
        })
    }

    /// The entries one pivot touches.
    fn entries(&self) -> u64 {
        (self.basis.len() as u64 + 1) * self.width as u64
    }

    /// Makes one pivot by Bland's rule, the first column that improves the
    /// objective entering and, among the rows that bound it, the one whose
    /// basic column comes first leaving.
    fn pivot(&mut self) -> Step {
        let width = self.width;
        let Some(column) = (0..width - 1).find(|&j| self.cost[j] < -EPSILON) else {
            return Step::Optimal;
        };
        if self.pivots >= self.most_pivots {
            return Step::GiveUp;
        }
        let mut leaving: Option<(usize, f64)> = None;
        for r in 0..self.basis.len() {
            let a = self.rows[r * width + column];
            if a > EPSILON {
                let ratio = self.rows[r * width + width - 1] / a;
                let better = match leaving {
                    None => true,
                    Some((best, least)) => {
                        ratio < least - EPSILON
                            || (ratio <= least + EPSILON && self.basis[r] < self.basis[best])
                    }
                };
                if better {
                    leaving = Some((r, ratio));
                }
            }
        }
        // The objective is bounded below by zero, so a column that
        // improves it is bounded by some row, but for rounding.
        let Some((row, _)) = leaving else {
            return Step::GiveUp;
        };
        self.pivots += 1;
        let pivot = self.rows[row * width + column];
        for x in &mut self.rows[row * width..(row + 1) * width] {
            *x /= pivot;
        }
        let (before, rest) = self.rows.split_at_mut(row * width);
        let (pivot_row, after) = rest.split_at_mut(width);
        for other in before
            .chunks_exact_mut(width)
            .chain(after.chunks_exact_mut(width))
            .chain(std::iter::once(&mut self.cost[..]))
        {
            let factor = other[column];
            if factor != 0.0 {
                for (x, &p) in other.iter_mut().zip(pivot_row.iter()) {
                    *x -= factor * p;
                }
            }
        }
        self.basis[row] = column;
        Step::Pivoted
    }

    /// At the optimum, the weights the dual gives each place when the
    /// objective is above zero, that is when the equation has no solution:
    /// one minus the reduced cost of the row's artificial column, with the
    /// row's sign. `None` when the equation has a solution.
    fn weights(&self) -> Option<Vec<(u32, f64)>> {
        let objective = -self.cost[self.width - 1];
        if objective <= 1e3 * EPSILON {
            return None;
        }
        Some(
            self.places
                .iter()
                .enumerate()
                .map(|(r, &(p, sign))| (p, sign * (1.0 - self.cost[self.structural + r])))
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
