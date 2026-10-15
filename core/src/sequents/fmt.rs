// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use super::notation::TEXT;
use super::{Sequent, TermId};
use std::fmt::{Display, Formatter, Result as FmtResult};

/// A formula of a sequent, as a value that prints it in one-sided notation.
#[derive(Clone, Copy, Debug)]
pub struct Formula<'a> {
    /// The sequent whose arena and atom names the formula lives in.
    sequent: &'a Sequent,
    /// The formula's root term.
    id: TermId,
}

/// One stop of the crate's printers' walk over a formula, which carries
/// what they need to place brackets.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Stop<T> {
    /// A subformula is reached, before anything below it. The flag says
    /// whether it stands inside another formula, where a binary one is
    /// written in brackets; for the formula the walk started on it is the
    /// flag the walk was given.
    Enter(T, bool),
    /// The place between the two subformulas of a binary formula.
    Between(T),
    /// A formula with subformulas is left, after the last of them, with
    /// the flag it was entered with.
    Exit(T, bool),
}

/// What a walk has yet to do with a formula.
#[derive(Clone, Copy, Debug)]
enum Step<T> {
    /// Enter it.
    Enter(T, bool),
    /// Its first subformula is done: go on to the second.
    Second(T, bool),
    /// Its last subformula is done: leave it.
    Exit(T, bool),
}

/// How many pending steps a walk holds before it allocates: one for each
/// formula it is inside of, so an ordinary formula is written without an
/// allocation.
const INLINE: usize = 16;

/// The stops of a formula in the order it is written: every subformula
/// when it is reached, a binary one again between its two subformulas, and
/// a compound one once more when it is left. The walk keeps the formulas
/// it is inside of in a stack of its own, so a formula of any depth takes
/// no more of the caller's stack than a literal does.
pub(crate) struct Stops<T, F> {
    /// The first pending steps, the next one last.
    inline: [Step<T>; INLINE],
    /// How many entries of `inline` are pending.
    len: usize,
    /// The pending steps beyond `inline`, the next one last.
    spilled: Vec<Step<T>>,
    /// Returns the first and the second subformula of a formula, each if
    /// there is one, in the order they are written.
    operands: F,
}

impl<T: Copy, F: Fn(T) -> (Option<T>, Option<T>)> Stops<T, F> {
    /// Returns the walk over the formula `root`, whose subformulas are
    /// what `operands` returns; `nested` is the flag `root` is entered
    /// with.
    pub(crate) fn new(root: T, nested: bool, operands: F) -> Self {
        Self {
            inline: [Step::Enter(root, nested); INLINE],
            len: 1,
            spilled: Vec::new(),
            operands,
        }
    }

    /// Adds a step to take before the pending ones.
    fn push(&mut self, step: Step<T>) {
        if self.len < INLINE {
            self.inline[self.len] = step;
            self.len += 1;
        } else {
            self.spilled.push(step);
        }
    }
}

impl<T: Copy, F: Fn(T) -> (Option<T>, Option<T>)> Iterator for Stops<T, F> {
    type Item = Stop<T>;

    /// Returns the next stop, or `None` once the formula is left.
    fn next(&mut self) -> Option<Stop<T>> {
        let step = match self.spilled.pop() {
            Some(step) => step,
            None => {
                self.len = self.len.checked_sub(1)?;
                self.inline[self.len]
            }
        };
        Some(match step {
            Step::Enter(formula, nested) => {
                match (self.operands)(formula) {
                    (Some(first), Some(_)) => {
                        self.push(Step::Second(formula, nested));
                        self.push(Step::Enter(first, true));
                    }
                    (Some(only), None) | (None, Some(only)) => {
                        self.push(Step::Exit(formula, nested));
                        self.push(Step::Enter(only, true));
                    }
                    (None, None) => {}
                }
                Stop::Enter(formula, nested)
            }
            Step::Second(formula, nested) => {
                self.push(Step::Exit(formula, nested));
                if let (_, Some(second)) = (self.operands)(formula) {
                    self.push(Step::Enter(second, true));
                }
                Stop::Between(formula)
            }
            Step::Exit(formula, nested) => Stop::Exit(formula, nested),
        })
    }
}

/// One stop of a [`Walk`] over a formula, in the order the formula is
/// written. A literal or a unit is one `Enter`; a formula with operands is
/// an `Enter`, its first operand, a `Between` before every further one and
/// an `Exit` after the last.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Visit<T> {
    /// A formula is reached, before anything below it.
    Enter(T),
    /// The place before the operand of the formula whose index (from 0) is
    /// the number: 1 for the second operand of a binary formula.
    Between(T, u32),
    /// A formula with operands is left, after the last of them.
    Exit(T),
}

/// The stops of a formula in the order it is written (see [`Visit`]),
/// returned by [`Sequent::walk`] and
/// [`Reading::walk`](crate::Reading::walk). The walk keeps the formulas it
/// is inside of in a stack of its own, so a printer that follows it takes
/// no more of its caller's stack for a formula of any depth than for a
/// literal.
pub struct Walk<'a, T> {
    /// The stops with the flags the crate's printers place brackets by.
    stops: Stops<T, Operands<'a, T>>,
}

/// Returns the first and the second operand of a formula, each if there is
/// one.
type Operands<'a, T> = Box<dyn Fn(T) -> (Option<T>, Option<T>) + 'a>;

impl<'a, T: Copy> Walk<'a, T> {
    /// Returns the walk over the formula `root`, whose operands are what
    /// `operands` returns.
    pub(crate) fn new(root: T, operands: impl Fn(T) -> (Option<T>, Option<T>) + 'a) -> Self {
        Self {
            stops: Stops::new(root, false, Box::new(operands)),
        }
    }
}

impl<T: Copy> Iterator for Walk<'_, T> {
    type Item = Visit<T>;

    /// Returns the next stop, or `None` once the formula is left.
    fn next(&mut self) -> Option<Visit<T>> {
        Some(match self.stops.next()? {
            Stop::Enter(formula, _) => Visit::Enter(formula),
            Stop::Between(formula) => Visit::Between(formula, 1),
            Stop::Exit(formula, _) => Visit::Exit(formula),
        })
    }
}

impl<T> std::fmt::Debug for Walk<'_, T> {
    /// Writes the walk's name alone: its stack is no part of what it is.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str("Walk")
    }
}

impl Sequent {
    /// Returns the walk over the formula rooted at `id`, which must belong to
    /// this sequent: the stops a printer of the formula in another syntax
    /// follows, each naming a term that [`Sequent::term`] returns.
    ///
    /// ```
    /// use linlog::sequents::Visit;
    /// use linlog::{Sequent, Term};
    ///
    /// let sequent: Sequent = "⊢ a ⊗ !b".parse().unwrap();
    /// let mut out = String::new();
    /// for visit in sequent.walk(sequent.roots()[0]) {
    ///     match visit {
    ///         Visit::Enter(id) => match sequent.term(id) {
    ///             Term::Atom(a) => out += sequent.atom_name(a),
    ///             Term::Tensor(..) => out += "(",
    ///             Term::Bang(_) => out += "!",
    ///             _ => unreachable!(),
    ///         },
    ///         Visit::Between(..) => out += " * ",
    ///         Visit::Exit(id) if sequent.term(id).kind().arity() == 2 => out += ")",
    ///         Visit::Exit(_) => {}
    ///     }
    /// }
    /// assert_eq!(out, "(a * !b)");
    /// ```
    ///
    /// # Panics
    ///
    /// For an id of another one, past its end.
    pub fn walk(&self, id: TermId) -> Walk<'_, TermId> {
        debug_assert!(id.index() < self.terms.len());
        Walk::new(id, |k| self.term(k).operands())
    }

    /// Returns the formula rooted at `id`, which must belong to this sequent,
    /// as a value that prints it.
    ///
    /// # Panics
    ///
    /// For an id of another one, past its end.
    pub fn formula(&self, id: TermId) -> Formula<'_> {
        debug_assert!(id.index() < self.terms.len());
        Formula { sequent: self, id }
    }
}

impl Display for Formula<'_> {
    /// Writes the formula with brackets around every binary subformula.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        TEXT.term(f, self.sequent, self.id, false)
    }
}

impl Display for Sequent {
    /// Writes the sequent one-sided: `⊢` followed by its formulas, separated by
    /// commas.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        TEXT.one_sided(f, self)
    }
}
