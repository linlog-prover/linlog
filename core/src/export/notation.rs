// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! What the export targets share: a symbol table per target, the printers
//! that write formulas and sequents through it with the bracketing of
//! `Display`, and the walk over a derivation that writes one inference at a
//! time.

use crate::occurrences::{Forest, OccId, Reading, Side};
use crate::ordinary::{Formulas, NodeId, Symbols};
use crate::proofs::style::Drawn;
use crate::proofs::{InfId, WriteError};
use crate::sequents::{Kind, Sequent, Term, TermId, Visit, Walk};
use std::fmt::Write;

/// How a target writes formulas and sequents: the spelling of every
/// connective, unit and of the turnstile, and how an atom's name is
/// escaped. Binary connectives are written between spaces, `!` and `?`
/// directly before their operand.
pub(crate) struct Notation {
    /// `⊗`
    pub(crate) tensor: &'static str,
    /// `⅋`
    pub(crate) par: &'static str,
    /// `&`
    pub(crate) with: &'static str,
    /// `⊕`
    pub(crate) plus: &'static str,
    /// `⊸`, which only a two-sided sequent shows.
    pub(crate) lollipop: &'static str,
    /// `!`
    pub(crate) bang: &'static str,
    /// `?`
    pub(crate) quest: &'static str,
    /// `1`
    pub(crate) one: &'static str,
    /// `⊥`
    pub(crate) bot: &'static str,
    /// `⊤`
    pub(crate) top: &'static str,
    /// `0`
    pub(crate) zero: &'static str,
    /// The mark after a negated atom's name.
    pub(crate) dual: &'static str,
    /// `⊢`
    pub(crate) turnstile: &'static str,
    /// What goes before the turnstile of a two-sided sequent in a proof
    /// tree so that the turnstiles of a unary inference line up; empty
    /// where the target cannot align them.
    pub(crate) align: &'static str,
    /// Writes an atom's name.
    pub(crate) atom: fn(&mut String, &str),
    /// The connectives and constants of ordinary formulas.
    pub(crate) ordinary: Symbols,
}

impl Notation {
    /// Writes the formula rooted at `id` of `sequent` one-sided, in
    /// brackets if it is binary and `brackets` is set.
    pub(crate) fn term(&self, out: &mut String, sequent: &Sequent, id: TermId, brackets: bool) {
        use Term::*;
        for visit in Walk::new(id, brackets, |k| sequent.term(k).operands()) {
            match visit {
                Visit::Enter(k, nested) => match sequent.term(k) {
                    Atom(a) => (self.atom)(out, sequent.atom_name(a)),
                    DualAtom(a) => {
                        (self.atom)(out, sequent.atom_name(a));
                        out.push_str(self.dual);
                    }
                    One => out.push_str(self.one),
                    Bot => out.push_str(self.bot),
                    Top => out.push_str(self.top),
                    Zero => out.push_str(self.zero),
                    Tensor(..) | Par(..) | With(..) | Plus(..) if nested => out.push('('),
                    Tensor(..) | Par(..) | With(..) | Plus(..) => {}
                    Bang(_) => out.push_str(self.bang),
                    Quest(_) => out.push_str(self.quest),
                },
                Visit::Between(k) => {
                    out.push(' ');
                    out.push_str(match sequent.term(k) {
                        Tensor(..) => self.tensor,
                        Par(..) => self.par,
                        With(..) => self.with,
                        _ => self.plus,
                    });
                    out.push(' ');
                }
                Visit::Exit(k, nested) => {
                    if nested && sequent.term(k).kind().arity() == 2 {
                        out.push(')');
                    }
                }
            }
        }
    }

    /// Writes the intuitionistic formula at `o` as the reading reads it
    /// (`⊸`, `1` for `⊥`, `⊤` for an input `0`, and so on), in brackets if
    /// it is binary and `brackets` is set.
    pub(crate) fn ill(&self, out: &mut String, reading: &Reading, o: OccId, brackets: bool) {
        use Kind::*;
        let forest = reading.forest();
        for visit in Walk::new(o, brackets, |o| reading.operands(o)) {
            match visit {
                Visit::Enter(o, nested) => match (forest.kind(o), reading.position(o)) {
                    (Atom | DualAtom, _) => {
                        (self.atom)(out, forest.sequent().atom_name(forest.atom(o).unwrap()));
                    }
                    (One | Bot, _) => out.push_str(self.one),
                    (Top, Side::Output) | (Zero, Side::Input) => out.push_str(self.top),
                    (Zero, Side::Output) | (Top, Side::Input) => out.push_str(self.zero),
                    (Tensor | Par | With | Plus, _) if nested => out.push('('),
                    (Tensor | Par | With | Plus, _) => {}
                    (Bang | Quest, _) => out.push_str(self.bang),
                },
                Visit::Between(o) => {
                    out.push(' ');
                    out.push_str(match (forest.kind(o), reading.position(o)) {
                        (Tensor, Side::Output) | (Par, Side::Input) => self.tensor,
                        (Tensor, Side::Input) | (Par, Side::Output) => self.lollipop,
                        (With, Side::Output) | (Plus, Side::Input) => self.with,
                        _ => self.plus,
                    });
                    out.push(' ');
                }
                Visit::Exit(o, nested) => {
                    if nested && forest.kind(o).arity() == 2 {
                        out.push(')');
                    }
                }
            }
        }
    }

    /// Writes a sequent one-sided: the turnstile, then its root formulas,
    /// comma-separated.
    pub(crate) fn one_sided(&self, out: &mut String, sequent: &Sequent) {
        out.push_str(self.turnstile);
        for (i, &id) in sequent.roots().iter().enumerate() {
            out.push_str(if i == 0 { " " } else { ", " });
            self.term(out, sequent, id, false);
        }
    }

    /// Writes a sequent of a derivation, occurrences in ascending order:
    /// one-sided, or two-sided under a reading with the hypotheses in id
    /// order before the turnstile and the goal after it, which `aligned`
    /// lines up with the turnstiles above and below. With `marks`, every
    /// formula stands between the two characters `'\u{2}'` and `'\u{3}'`.
    pub(crate) fn sequent(
        &self,
        out: &mut String,
        forest: &Forest,
        reading: Option<&Reading>,
        sequent: &[OccId],
        aligned: bool,
        marks: bool,
    ) {
        let (open, close) = if marks { ("\u{2}", "\u{3}") } else { ("", "") };
        let Some(reading) = reading else {
            out.push_str(self.turnstile);
            for (i, &o) in sequent.iter().enumerate() {
                out.push_str(if i == 0 { " " } else { ", " });
                out.push_str(open);
                self.term(out, forest.sequent(), forest.term(o), false);
                out.push_str(close);
            }
            return;
        };
        let mut goal = None;
        let mut hypotheses = 0;
        for &o in sequent {
            if reading.position(o) == Side::Output {
                goal = Some(o);
                continue;
            }
            if hypotheses > 0 {
                out.push_str(", ");
            }
            out.push_str(open);
            self.ill(out, reading, o, false);
            out.push_str(close);
            hypotheses += 1;
        }
        if hypotheses > 0 {
            out.push(' ');
        }
        if aligned {
            out.push_str(self.align);
        }
        out.push_str(self.turnstile);
        if let Some(goal) = goal {
            out.push(' ');
            out.push_str(open);
            self.ill(out, reading, goal, false);
            out.push_str(close);
        }
    }
}

impl Notation {
    /// Writes a sequent of ordinary formulas two-sided, `left ⊢ right`
    /// in the order given, with the turnstile lined up as
    /// [`sequent`](Self::sequent) does when `aligned` is set, and with
    /// `marks` every formula between `'\u{2}'` and `'\u{3}'`.
    pub(crate) fn ordinary(
        &self,
        out: &mut String,
        formulas: &Formulas,
        sides: (&[NodeId], &[NodeId]),
        aligned: bool,
        marks: bool,
    ) {
        let (open, close) = if marks { ("\u{2}", "\u{3}") } else { ("", "") };
        let formula = |out: &mut String, id: NodeId| {
            out.push_str(open);
            formulas.write(out, id, false, &self.ordinary, |o, a| {
                (self.atom)(o, formulas.atom_name(a));
            });
            out.push_str(close);
        };
        let (left, right) = sides;
        for (i, &id) in left.iter().enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            formula(out, id);
        }
        if !left.is_empty() {
            out.push(' ');
        }
        if aligned {
            out.push_str(self.align);
        }
        out.push_str(self.turnstile);
        for (i, &id) in right.iter().enumerate() {
            out.push_str(if i == 0 { " " } else { ", " });
            formula(out, id);
        }
    }
}

/// One step of the walk over a derivation.
#[derive(Clone, Copy)]
pub(crate) enum Step {
    /// An inference is reached, before its premises; the root is at depth 0.
    Enter(InfId, usize),
    /// An inference is left, after its premises.
    Exit(InfId, usize),
}

/// Walks a derivation depth-first from the root, premises in their order,
/// and hands `visit` every inference once on the way up and once on the
/// way down, until it fails. Exits alone come in postfix order. The walk
/// keeps its own stack, so a derivation of any height fits.
pub(crate) fn walk<T: Drawn, E>(
    derivation: &T,
    mut visit: impl FnMut(Step) -> Result<(), E>,
) -> Result<(), E> {
    let mut stack = vec![Step::Enter(derivation.root(), 0)];
    while let Some(step) = stack.pop() {
        visit(step)?;
        if let Step::Enter(id, depth) = step {
            stack.push(Step::Exit(id, depth));
            for &p in derivation.premises(id).iter().rev() {
                stack.push(Step::Enter(p, depth + 1));
            }
        }
    }
    Ok(())
}

/// Writes what `buffer` holds to `out`, empties it, and fails if `stop`
/// says so: the emitters make one inference at a time in a buffer and
/// hand it on, so that they hold one inference's text and can be stopped
/// between two.
pub(crate) fn flush(
    out: &mut impl Write,
    buffer: &mut String,
    stop: &mut impl FnMut() -> bool,
) -> Result<(), WriteError> {
    out.write_str(buffer)?;
    buffer.clear();
    if stop() {
        Err(WriteError::Stopped)
    } else {
        Ok(())
    }
}
