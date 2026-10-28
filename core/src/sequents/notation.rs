// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The one printer of formulas and sequents: a table of spellings per
//! target, [`TEXT`](crate::sequents::notation::TEXT) for `Display` and one per export, and the walks that
//! write a formula, a sequent one-sided or two-sided under a reading, and
//! a sequent of ordinary logic through it. A new connective is an arm here
//! and a spelling in each table, never a printer of its own.

use crate::hash::HashMap;
use crate::occurrences::written::{self, Written};
use crate::occurrences::{Forest, IllConnective, Member, OccId, Reading, Side};
use crate::ordinary::{Formulas, NodeId, Symbols};
use crate::sequents::{Atom, Kind, Sequent, Stop, Stops, Term, TermId};
use std::borrow::Cow;
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
    /// `\`, the left division, which only a two-sided sequent of the
    /// Lambek calculus shows.
    pub(crate) under: &'static str,
    /// `/`, the right division, as `under`.
    pub(crate) over: &'static str,
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
    /// How an atom's name is written.
    pub(crate) atom: Atoms,
    /// `∀`, with what stands between it and the variable's name.
    pub(crate) forall: &'static str,
    /// `∃`, as `forall`.
    pub(crate) exists: &'static str,
    /// What stands between a binder's variable and its body: the dot and
    /// a space.
    pub(crate) dot: &'static str,
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
    under: "\\",
    over: "/",
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
    atom: Atoms::Plain,
    forall: "∀",
    exists: "∃",
    dot: ". ",
    ordinary: Symbols::UNICODE,
};

/// How a target writes an atom's name: a closed set rather than a function
/// pointer, so that every writer is compiled for the output it writes
/// into (through `dyn Write` one call per character made the Typst
/// export a fifth slower).
#[derive(Clone, Copy, Debug)]
pub(crate) enum Atoms {
    /// As it is.
    Plain,
    /// In LaTeX's math mode.
    #[cfg(feature = "latex")]
    Latex,
    /// In Typst's math mode.
    #[cfg(feature = "typst")]
    Typst,
    /// With its Latin letters as mathematical italic characters.
    #[cfg(feature = "svg")]
    Italic,
}

impl Atoms {
    /// Writes an atom's name into `out`.
    pub(crate) fn write<W: Write + ?Sized>(self, out: &mut W, name: &str) -> FmtResult {
        match self {
            Self::Plain => out.write_str(name),
            #[cfg(feature = "latex")]
            Self::Latex => crate::export::latex::atom(out, name),
            #[cfg(feature = "typst")]
            Self::Typst => crate::export::typst::atom(out, name),
            #[cfg(feature = "svg")]
            Self::Italic => crate::export::svg::italic(out, name),
        }
    }
}

/// The binders a printer's walk is inside of: their variables' names as
/// written, innermost last, and how many binders of each name are open,
/// so that a bound variable is written as its binder's name and a new
/// binder's name is checked in constant time. Empty, and allocating
/// nothing, while the walk meets no binder.
#[derive(Default)]
pub(crate) struct Scope<'a> {
    /// The names, innermost last.
    names: Vec<Cow<'a, str>>,
    /// How many open binders have each name.
    open: HashMap<Cow<'a, str>, u32>,
    /// The sequent's symbol names, filled at the first binder.
    symbols: Option<std::collections::HashSet<&'a str>>,
}

impl<'a> Scope<'a> {
    /// Returns the name a binder's variable of `name` is written with:
    /// `name` itself, or with `_` and a number after it where a symbol of
    /// `sequent` or an enclosing binder has the name, so that the text
    /// reads back as the formula it writes (`∀x. p(x, x)` with a constant
    /// `x` would not); then opens the binder.
    #[cold]
    #[inline(never)]
    fn enter(&mut self, sequent: &'a Sequent, name: &'a str) -> &str {
        let symbols = self
            .symbols
            .get_or_insert_with(|| sequent.fo.symbols.iter().map(|(s, _)| s.as_str()).collect());
        let clashes = |open: &HashMap<Cow<'a, str>, u32>, candidate: &str| {
            symbols.contains(candidate) || open.contains_key(candidate)
        };
        let mut written = Cow::Borrowed(name);
        let mut n = 0u32;
        while clashes(&self.open, &written) {
            n += 1;
            written = Cow::Owned(format!("{name}_{n}"));
        }
        *self.open.entry(written.clone()).or_insert(0) += 1;
        self.names.push(written);
        self.names.last().expect("just pushed")
    }

    /// Closes the innermost binder.
    #[cold]
    #[inline(never)]
    fn exit(&mut self) {
        if let Some(name) = self.names.pop()
            && let Some(count) = self.open.get_mut(&name)
        {
            *count -= 1;
            if *count == 0 {
                self.open.remove(&name);
            }
        }
    }

    /// Returns the name of the bound variable `index` binders up, if the
    /// scope reaches it.
    fn name(&self, index: u32) -> Option<&str> {
        self.names
            .len()
            .checked_sub(index as usize + 1)
            .map(|at| self.names[at].as_ref())
    }
}

/// Returns whether a formula of `kind` is written in brackets where it
/// stands inside another: a binary one, and a binder, whose scope would
/// otherwise run on.
fn bracketed(kind: Kind) -> bool {
    kind.arity() == 2 || kind.is_binder()
}

/// The place a walk keeps its scope: made at the first binder, and
/// dropped by [`close`] at the walk's end, so that a walk without a
/// binder, which is every propositional one, pays no drop.
type Slot<'a> = std::mem::ManuallyDrop<Option<Box<Scope<'a>>>>;

/// Drops the scope of `slot`, if one was made.
#[inline(always)]
fn close(slot: Slot<'_>) {
    if let Some(scope) = std::mem::ManuallyDrop::into_inner(slot) {
        drop(scope);
    }
}

impl Notation {
    /// Writes the atom `atom` of `sequent`: its name, or a predicate with
    /// its arguments, a bound variable by the name of its binder in
    /// `scope`, and by its index after `#` where the scope does not reach
    /// it (an occurrence written below its binder).
    #[inline(always)]
    pub(crate) fn write_atom<W: Write + ?Sized>(
        &self,
        out: &mut W,
        sequent: &Sequent,
        atom: Atom,
        predicates: bool,
        scope: Option<&Scope<'_>>,
    ) -> FmtResult {
        if predicates {
            self.write_predicate(out, sequent, atom, scope)
        } else {
            self.atom.write(out, sequent.atom_name(atom))
        }
    }

    /// Writes the atom `atom` of a first-order `sequent` as
    /// [`write_atom`](Self::write_atom) does.
    #[cold]
    #[inline(never)]
    fn write_predicate<W: Write + ?Sized>(
        &self,
        out: &mut W,
        sequent: &Sequent,
        atom: Atom,
        scope: Option<&Scope<'_>>,
    ) -> FmtResult {
        let (symbol, args) = sequent.fo.atoms[atom.index()];
        let fo = &sequent.fo;
        self.atom.write(out, &fo.symbols[symbol.index()].0)?;
        let list = fo.list(args);
        if list.is_empty() {
            return Ok(());
        }
        out.write_char('(')?;
        for (i, &t) in list.iter().enumerate() {
            if i > 0 {
                out.write_str(", ")?;
            }
            fo.write(
                out,
                t,
                &mut |out, name| self.atom.write(out, name),
                &mut |out, index| match scope.and_then(|scope| scope.name(index)) {
                    Some(name) => self.atom.write(out, name),
                    None => write!(out, "#{index}"),
                },
            )?;
        }
        out.write_char(')')
    }

    /// Opens the binder `term` of `sequent` in `scope` and writes it, `∀x.
    /// ` up to where its body starts, with the name its variable is
    /// written by (`x` where none is recorded).
    #[cold]
    #[inline(never)]
    fn binder<'a, W: Write + ?Sized>(
        &self,
        out: &mut W,
        sequent: &'a Sequent,
        term: TermId,
        kind: Kind,
        scope: &mut Option<Box<Scope<'a>>>,
    ) -> FmtResult {
        out.write_str(if kind == Kind::Forall {
            self.forall
        } else {
            self.exists
        })?;
        let scope = scope.get_or_insert_with(Box::default);
        let name = scope.enter(sequent, sequent.binder_name(term).unwrap_or("x"));
        self.atom.write(out, name)?;
        out.write_str(self.dot)
    }

    /// Writes the formula rooted at `id` of `sequent` one-sided, in
    /// brackets if it is binary or a binder and `brackets` is set.
    pub(crate) fn term<W: Write>(
        &self,
        out: &mut W,
        sequent: &Sequent,
        id: TermId,
        brackets: bool,
    ) -> FmtResult {
        let mut slot: Slot<'_> = std::mem::ManuallyDrop::new(None);
        let written = self.term_in(out, sequent, id, brackets, &mut slot);
        close(slot);
        written
    }

    /// Writes the formula as [`term`](Self::term) does, its scope in
    /// `scope`.
    #[inline(always)]
    fn term_in<'a, W: Write>(
        &self,
        out: &mut W,
        sequent: &'a Sequent,
        id: TermId,
        brackets: bool,
        scope: &mut Option<Box<Scope<'a>>>,
    ) -> FmtResult {
        use Term::*;
        debug_assert!(id.index() < sequent.terms.len());
        let predicates = !sequent.fo.atoms.is_empty();
        for visit in Stops::new(id, brackets, |k| sequent.term(k).operands()) {
            match visit {
                Stop::Enter(k, nested) => {
                    let term = sequent.term(k);
                    debug_assert!(term.subterms().all(|sub| sub < k));
                    match term {
                        Atom(a) => {
                            self.write_atom(out, sequent, a, predicates, scope.as_deref())?
                        }
                        DualAtom(a) => {
                            if !self.dual_prefix.is_empty() {
                                out.write_str(self.dual_prefix)?;
                            }
                            self.write_atom(out, sequent, a, predicates, scope.as_deref())?;
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
                        Forall(_) | Exists(_) => {
                            if nested {
                                out.write_char('(')?;
                            }
                            self.binder(out, sequent, k, term.kind(), scope)?;
                        }
                    }
                }
                Stop::Between(k) => {
                    out.write_char(' ')?;
                    out.write_str(match sequent.term(k) {
                        Tensor(..) => self.tensor,
                        Par(..) => self.par,
                        With(..) => self.with,
                        Atom(_) | DualAtom(_) | One | Bot | Top | Zero | Plus(..) | Bang(_)
                        | Quest(_) | Forall(_) | Exists(_) => self.plus,
                    })?;
                    out.write_char(' ')?;
                }
                Stop::Exit(k, nested) => {
                    let kind = sequent.term(k).kind();
                    if kind.is_binder()
                        && let Some(scope) = scope
                    {
                        scope.exit();
                    }
                    if nested && bracketed(kind) {
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
        let mut slot: Slot<'_> = std::mem::ManuallyDrop::new(None);
        let written = self.ill_in(out, reading, o, brackets, &mut slot);
        close(slot);
        written
    }

    /// Writes the intuitionistic formula as [`ill`](Self::ill) does, its
    /// scope in `scope`.
    #[inline(always)]
    fn ill_in<'a, W: Write>(
        &self,
        out: &mut W,
        reading: &'a Reading,
        o: OccId,
        brackets: bool,
        scope: &mut Option<Box<Scope<'a>>>,
    ) -> FmtResult {
        use IllConnective as C;
        let forest = reading.forest();
        let sequent = forest.sequent();
        let predicates = !sequent.fo.atoms.is_empty();
        for visit in Stops::new(o, brackets, |o| reading.operands(o)) {
            match visit {
                Stop::Enter(o, nested) => match reading.connective(o) {
                    C::Atom => {
                        let atom = forest.atom(o).expect("a literal has an atom");
                        self.write_atom(out, sequent, atom, predicates, scope.as_deref())?;
                    }
                    C::One => out.write_str(self.one)?,
                    C::Top => out.write_str(self.top)?,
                    C::Zero => out.write_str(self.zero)?,
                    C::Tensor | C::With | C::Plus | C::Lollipop | C::Under | C::Over if nested => {
                        out.write_char('(')?;
                    }
                    C::Tensor | C::With | C::Plus | C::Lollipop | C::Under | C::Over => {}
                    C::Bang => out.write_str(self.bang)?,
                    connective @ (C::Forall | C::Exists) => {
                        if nested {
                            out.write_char('(')?;
                        }
                        let kind = if connective == C::Forall {
                            Kind::Forall
                        } else {
                            Kind::Exists
                        };
                        self.binder(out, sequent, forest.term(o), kind, scope)?;
                    }
                },
                Stop::Between(o) => {
                    out.write_char(' ')?;
                    out.write_str(match reading.connective(o) {
                        C::Tensor => self.tensor,
                        C::Lollipop => self.lollipop,
                        C::Under => self.under,
                        C::Over => self.over,
                        C::With => self.with,
                        C::Atom
                        | C::One
                        | C::Top
                        | C::Zero
                        | C::Bang
                        | C::Plus
                        | C::Forall
                        | C::Exists => self.plus,
                    })?;
                    out.write_char(' ')?;
                }
                Stop::Exit(o, nested) => {
                    let kind = forest.kind(o);
                    if kind.is_binder()
                        && let Some(scope) = scope
                    {
                        scope.exit();
                    }
                    if nested && bracketed(kind) {
                        out.write_char(')')?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Writes the formula at `o` as its author wrote it (`writing`): each
    /// connective on the side it stands on, `⊸` where an implication was
    /// written, a negation before every subformula written with one, and
    /// before `o` itself where `negated` says. A negated binary formula,
    /// and where the negation follows its operand a negated `!` or `?`, is
    /// in brackets, as is a binary one inside another.
    pub(crate) fn written<W: Write>(
        &self,
        out: &mut W,
        forest: &Forest,
        writing: &[Written],
        o: OccId,
        negated: bool,
    ) -> FmtResult {
        let mut slot: Slot<'_> = std::mem::ManuallyDrop::new(None);
        let written = self.written_in(out, forest, writing, o, negated, &mut slot);
        close(slot);
        written
    }

    /// Writes the formula as [`written`](Self::written) does, its scope
    /// in `scope`.
    #[inline(always)]
    fn written_in<'a, W: Write>(
        &self,
        out: &mut W,
        forest: &'a Forest,
        writing: &[Written],
        o: OccId,
        negated: bool,
        scope: &mut Option<Box<Scope<'a>>>,
    ) -> FmtResult {
        let top = o;
        let negation = |k: OccId| {
            if k == top {
                negated
            } else {
                written::negated(forest, writing, k)
            }
        };
        // The connective as written: on the input side the dual of the
        // one-sided one.
        let core = |k: OccId| match writing[k.index()].side() {
            Side::Output => forest.kind(k),
            Side::Input => forest.kind(k).dual(),
        };
        // Where the negation follows its operand, a prefix operator under
        // it needs brackets to stay under it.
        let postfix = !self.dual.is_empty();
        let operands = |k: OccId| (forest.left(k), forest.right(k));
        let sequent = forest.sequent();
        let predicates = !sequent.fo.atoms.is_empty();
        for visit in Stops::new(o, false, operands) {
            match visit {
                Stop::Enter(k, nested) => {
                    let negated = negation(k);
                    if negated {
                        out.write_str(self.dual_prefix)?;
                    }
                    match core(k) {
                        Kind::Atom => {
                            let atom = forest.atom(k).expect("a literal has an atom");
                            self.write_atom(out, sequent, atom, predicates, scope.as_deref())?;
                        }
                        Kind::DualAtom => {
                            let atom = forest.atom(k).expect("a literal has an atom");
                            out.write_str(self.dual_prefix)?;
                            self.write_atom(out, sequent, atom, predicates, scope.as_deref())?;
                            out.write_str(self.dual)?;
                        }
                        Kind::One => out.write_str(self.one)?,
                        Kind::Bot => out.write_str(self.bot)?,
                        Kind::Top => out.write_str(self.top)?,
                        Kind::Zero => out.write_str(self.zero)?,
                        Kind::Tensor | Kind::Par | Kind::With | Kind::Plus => {
                            if nested || negated {
                                out.write_char('(')?;
                            }
                        }
                        kind @ (Kind::Bang | Kind::Quest) => {
                            if negated && postfix {
                                out.write_char('(')?;
                            }
                            out.write_str(if kind == Kind::Bang {
                                self.bang
                            } else {
                                self.quest
                            })?;
                        }
                        kind @ (Kind::Forall | Kind::Exists) => {
                            if nested || negated {
                                out.write_char('(')?;
                            }
                            self.binder(out, sequent, forest.term(k), kind, scope)?;
                        }
                    }
                    if negated && forest.kind(k).arity() == 0 {
                        out.write_str(self.dual)?;
                    }
                }
                Stop::Between(k) => {
                    out.write_char(' ')?;
                    out.write_str(if writing[k.index()].is_implication() {
                        self.lollipop
                    } else {
                        match core(k) {
                            Kind::Tensor => self.tensor,
                            Kind::Par => self.par,
                            Kind::With => self.with,
                            Kind::Plus
                            | Kind::Atom
                            | Kind::DualAtom
                            | Kind::One
                            | Kind::Bot
                            | Kind::Top
                            | Kind::Zero
                            | Kind::Bang
                            | Kind::Quest
                            | Kind::Forall
                            | Kind::Exists => self.plus,
                        }
                    })?;
                    out.write_char(' ')?;
                }
                Stop::Exit(k, nested) => {
                    let negated = negation(k);
                    let kind = forest.kind(k);
                    if kind.is_binder()
                        && let Some(scope) = scope
                    {
                        scope.exit();
                    }
                    let arity = kind.arity();
                    if (bracketed(kind) && (nested || negated))
                        || (arity == 1 && !kind.is_binder() && negated && postfix)
                    {
                        out.write_char(')')?;
                    }
                    if negated {
                        out.write_str(self.dual)?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Writes a classical sequent two-sided, `left ⊢ right`, each formula
    /// an occurrence as its author wrote it with a negation before it or
    /// without ([`written`](Self::written)), the turnstile lined up as
    /// [`sequent`](Self::sequent) does when `aligned` is set, and with
    /// `marks` every formula between `'\u{2}'` and `'\u{3}'`.
    #[expect(
        clippy::too_many_arguments,
        reason = "the sides and the two switches are what every target passes"
    )]
    pub(crate) fn classical<W: Write>(
        &self,
        out: &mut W,
        forest: &Forest,
        writing: &[Written],
        left: impl Iterator<Item = (OccId, bool)>,
        right: impl Iterator<Item = (OccId, bool)>,
        aligned: bool,
        marks: bool,
    ) -> FmtResult {
        let (open, close) = if marks { ("\u{2}", "\u{3}") } else { ("", "") };
        let mut any = false;
        for (o, negated) in left {
            if any {
                out.write_str(", ")?;
            }
            out.write_str(open)?;
            self.written(out, forest, writing, o, negated)?;
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
        for (i, (o, negated)) in right.enumerate() {
            out.write_str(if i == 0 { " " } else { ", " })?;
            out.write_str(open)?;
            self.written(out, forest, writing, o, negated)?;
            out.write_str(close)?;
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
    /// one-sided, or two-sided under a reading with the hypotheses in the
    /// order it writes them before the turnstile and the goal after it, which `aligned`
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
        let output = |o: OccId| reading.position(o) == Side::Output;
        let goal = sequent.iter().map(|m| m.occ()).rfind(|&o| output(o));
        let hypotheses = reading
            .written(sequent)
            .map(|p| sequent[p].occ())
            .filter(|&o| !output(o));
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
                self.atom.write(o, formulas.atom_name(a))
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
