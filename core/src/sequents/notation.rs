// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The one printer of formulas and sequents: a table of spellings per
//! target, [`TEXT`] for `Display` and one per export, and the walks that
//! write a formula, a sequent one-sided or two-sided under a reading, and
//! a sequent of ordinary logic through it. A new connective is an arm here
//! and a spelling in each table, never a printer of its own.

use crate::occurrences::{Forest, Member, OccId, Reading, Side};
use crate::ordinary::{Formulas, NodeId, Symbols};
use crate::sequents::{Kind, Sequent, Term, TermId, Visit, Walk};
use std::fmt::{Result as FmtResult, Write};

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
    /// The mark before a negated atom's name.
    pub(crate) dual_prefix: &'static str,
    /// The mark after a negated atom's name.
    pub(crate) dual: &'static str,
    /// `⊢`
    pub(crate) turnstile: &'static str,
    /// What goes before the turnstile of a two-sided sequent in a proof
    /// tree so that the turnstiles of a unary inference line up; empty
    /// where the target cannot align them.
    pub(crate) align: &'static str,
    /// Writes an atom's name.
    pub(crate) atom: fn(&mut dyn Write, &str) -> FmtResult,
    /// The connectives and constants of ordinary formulas.
    pub(crate) ordinary: Symbols,
}

/// The spellings `Display` writes: the Unicode symbols, a negated atom as
/// `~a`, a name as it is.
pub(crate) const TEXT: Notation = Notation {
    tensor: "⊗",
    par: "⅋",
    with: "&",
    plus: "⊕",
    lollipop: "⊸",
    bang: "!",
    quest: "?",
    one: "1",
    bot: "⊥",
    top: "⊤",
    zero: "0",
    dual_prefix: "~",
    dual: "",
    turnstile: "⊢",
    align: "",
    atom: name,
    ordinary: Symbols::UNICODE,
};

/// Writes an atom's name as it is.
fn name(out: &mut dyn Write, name: &str) -> FmtResult {
    out.write_str(name)
}

impl Notation {
    /// Writes the formula rooted at `id` of `sequent` one-sided, in
    /// brackets if it is binary and `brackets` is set.
    pub(crate) fn term<W: Write>(
        &self,
        out: &mut W,
        sequent: &Sequent,
        id: TermId,
        brackets: bool,
    ) -> FmtResult {
        use Term::*;
        debug_assert!(id.index() < sequent.terms.len());
        for visit in Walk::new(id, brackets, |k| sequent.term(k).operands()) {
            match visit {
                Visit::Enter(k, nested) => {
                    let term = sequent.term(k);
                    debug_assert!(term.subterms().all(|sub| sub < k));
                    match term {
                        Atom(a) => (self.atom)(out, sequent.atom_name(a))?,
                        DualAtom(a) => {
                            out.write_str(self.dual_prefix)?;
                            (self.atom)(out, sequent.atom_name(a))?;
                            out.write_str(self.dual)?;
                        }
                        One => out.write_str(self.one)?,
                        Bot => out.write_str(self.bot)?,
                        Top => out.write_str(self.top)?,
                        Zero => out.write_str(self.zero)?,
                        Tensor(..) | Par(..) | With(..) | Plus(..) if nested => {
                            out.write_char('(')?;
                        }
                        Tensor(..) | Par(..) | With(..) | Plus(..) => {}
                        Bang(_) => out.write_str(self.bang)?,
                        Quest(_) => out.write_str(self.quest)?,
                    }
                }
                Visit::Between(k) => {
                    out.write_char(' ')?;
                    out.write_str(match sequent.term(k) {
                        Tensor(..) => self.tensor,
                        Par(..) => self.par,
                        With(..) => self.with,
                        Atom(_) | DualAtom(_) | One | Bot | Top | Zero | Plus(..) | Bang(_)
                        | Quest(_) => self.plus,
                    })?;
                    out.write_char(' ')?;
                }
                Visit::Exit(k, nested) => {
                    if nested && sequent.term(k).kind().arity() == 2 {
                        out.write_char(')')?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Writes the intuitionistic formula at `o` as the reading reads it
    /// (`⊸`, `1` for `⊥`, `⊤` for an input `0`, and so on), in brackets if
    /// it is binary and `brackets` is set.
    pub(crate) fn ill<W: Write>(
        &self,
        out: &mut W,
        reading: &Reading,
        o: OccId,
        brackets: bool,
    ) -> FmtResult {
        use Kind::*;
        let forest = reading.forest();
        for visit in Walk::new(o, brackets, |o| reading.operands(o)) {
            match visit {
                Visit::Enter(o, nested) => match (forest.kind(o), reading.position(o)) {
                    (Atom | DualAtom, _) => {
                        let atom = forest.atom(o).expect("a literal has an atom");
                        (self.atom)(out, forest.sequent().atom_name(atom))?;
                    }
                    (One | Bot, _) => out.write_str(self.one)?,
                    (Top, Side::Output) | (Zero, Side::Input) => out.write_str(self.top)?,
                    (Zero, Side::Output) | (Top, Side::Input) => out.write_str(self.zero)?,
                    (Tensor | Par | With | Plus, _) if nested => out.write_char('(')?,
                    (Tensor | Par | With | Plus, _) => {}
                    (Bang | Quest, _) => out.write_str(self.bang)?,
                },
                Visit::Between(o) => {
                    out.write_char(' ')?;
                    out.write_str(match (forest.kind(o), reading.position(o)) {
                        (Tensor, Side::Output) | (Par, Side::Input) => self.tensor,
                        (Tensor, Side::Input) | (Par, Side::Output) => self.lollipop,
                        (With, Side::Output) | (Plus, Side::Input) => self.with,
                        _ => self.plus,
                    })?;
                    out.write_char(' ')?;
                }
                Visit::Exit(o, nested) => {
                    if nested && forest.kind(o).arity() == 2 {
                        out.write_char(')')?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Writes a sequent one-sided: the turnstile, then its root formulas,
    /// comma-separated.
    pub(crate) fn one_sided<W: Write>(&self, out: &mut W, sequent: &Sequent) -> FmtResult {
        out.write_str(self.turnstile)?;
        for (i, &id) in sequent.roots().iter().enumerate() {
            out.write_str(if i == 0 { " " } else { ", " })?;
            self.term(out, sequent, id, false)?;
        }
        Ok(())
    }

    /// Writes a sequent of a derivation, occurrences in ascending order:
    /// one-sided, or two-sided under a reading with the hypotheses in id
    /// order before the turnstile and the goal after it, which `aligned`
    /// lines up with the turnstiles above and below. With `marks`, every
    /// formula stands between the two characters `'\u{2}'` and `'\u{3}'`.
    pub(crate) fn sequent<W: Write>(
        &self,
        out: &mut W,
        forest: &Forest,
        reading: Option<&Reading>,
        sequent: &[Member],
        aligned: bool,
        marks: bool,
    ) -> FmtResult {
        let (open, close) = if marks { ("\u{2}", "\u{3}") } else { ("", "") };
        let Some(reading) = reading else {
            out.write_str(self.turnstile)?;
            for (i, o) in sequent.iter().map(|m| m.occ()).enumerate() {
                out.write_str(if i == 0 { " " } else { ", " })?;
                out.write_str(open)?;
                self.term(out, forest.sequent(), forest.term(o), false)?;
                out.write_str(close)?;
            }
            return Ok(());
        };
        let members = || sequent.iter().map(|m| m.occ());
        let output = |o: OccId| reading.position(o) == Side::Output;
        let goal = members().rfind(|&o| output(o));
        let hypotheses = members().filter(|&o| !output(o));
        self.two_sided(out, reading, hypotheses, goal, aligned, (open, close))
    }

    /// Writes the sequent a reading reads two-sided: its hypotheses in the
    /// order written, `⊢` and its goal.
    pub(crate) fn reading<W: Write>(&self, out: &mut W, reading: &Reading) -> FmtResult {
        let goal = Some(reading.goal());
        self.two_sided(out, reading, reading.hypotheses(), goal, false, ("", ""))
    }

    /// Writes `hypotheses ⊢ goal` under a reading, the turnstile lined up
    /// where `aligned`, every formula between the `marks`.
    fn two_sided<W: Write>(
        &self,
        out: &mut W,
        reading: &Reading,
        hypotheses: impl Iterator<Item = OccId>,
        goal: Option<OccId>,
        aligned: bool,
        marks: (&str, &str),
    ) -> FmtResult {
        let (open, close) = marks;
        let mut any = false;
        for o in hypotheses {
            if any {
                out.write_str(", ")?;
            }
            out.write_str(open)?;
            self.ill(out, reading, o, false)?;
            out.write_str(close)?;
            any = true;
        }
        if any {
            out.write_char(' ')?;
        }
        if aligned {
            out.write_str(self.align)?;
        }
        out.write_str(self.turnstile)?;
        if let Some(goal) = goal {
            out.write_char(' ')?;
            out.write_str(open)?;
            self.ill(out, reading, goal, false)?;
            out.write_str(close)?;
        }
        Ok(())
    }

    /// Writes a sequent of ordinary formulas two-sided, `left ⊢ right`
    /// in the order given, with the turnstile lined up as
    /// [`sequent`](Self::sequent) does when `aligned` is set, and with
    /// `marks` every formula between `'\u{2}'` and `'\u{3}'`.
    pub(crate) fn ordinary<W: Write>(
        &self,
        out: &mut W,
        formulas: &Formulas,
        sides: (&[NodeId], &[NodeId]),
        aligned: bool,
        marks: bool,
    ) -> FmtResult {
        let (open, close) = if marks { ("\u{2}", "\u{3}") } else { ("", "") };
        let formula = |out: &mut W, id: NodeId| {
            out.write_str(open)?;
            formulas.write(out, id, false, &self.ordinary, |o, a| {
                (self.atom)(o, formulas.atom_name(a))
            })?;
            out.write_str(close)
        };
        let (left, right) = sides;
        for (i, &id) in left.iter().enumerate() {
            if i > 0 {
                out.write_str(", ")?;
            }
            formula(out, id)?;
        }
        if !left.is_empty() {
            out.write_char(' ')?;
        }
        if aligned {
            out.write_str(self.align)?;
        }
        out.write_str(self.turnstile)?;
        for (i, &id) in right.iter().enumerate() {
            out.write_str(if i == 0 { " " } else { ", " })?;
            formula(out, id)?;
        }
        Ok(())
    }
}
