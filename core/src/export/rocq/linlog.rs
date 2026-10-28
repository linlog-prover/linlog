// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The certificates for linlog's own Rocq library: a lemma stating the
//! sequent over the library's calculus, proved by its soundness theorem
//! from the derivation written as a proof term, which the library's
//! checker accepts by computation.
//!
//! A one-sided derivation becomes a term of the dyadic calculus the
//! checker runs (the library's `proof`): every rule is its constructor,
//! but the exponentials, whose `?` formulas the term keeps in an
//! unrestricted zone. A `Quest` moves a `?` formula there where it first
//! appears, at the root, as a subformula of the rule below or as a cut
//! formula or its dual in a cut's premise, so that a dereliction is a
//! `Copy` from the zone, and a contraction and the weakening of a `?`
//! formula are implicit in it; a promotion then finds its `?` context in
//! the zone, as the checker's `Bang` requires. A two-sided derivation
//! becomes a two-sided term (the library's `iproof`), one constructor per
//! two-sided rule, with the context of a `⊤` on the right and of a `0` on
//! the left written out.
//!
//! The library names the calculi without cut; a derivation with a cut
//! states its calculus as a record literal with the cut, proved by the
//! one-sided theorem for every calculus or by the two-sided one.
//!
//! Atoms are Rocq strings, which take any name, so no atom is renamed.

use super::{
    LINLOG_PRELUDE, LINLOG_REFUTATION_PRELUDE, LINLOG_TWO_SIDED_PRELUDE, Options, Unsupported,
};
use crate::Error;
use crate::export::Form;
use crate::export::notation::{Step, flush, walk};
use crate::occurrences::{Forest, Member, OccId, Reading, Side};
use crate::proofs::{Derivation, InfId, Rule};
use crate::search::{Classical, Disproof, Refutation};
use crate::sequents::{Stop, Stops, Term};
use std::fmt::Write;

/// The optional rules of the one-sided calculus a certificate states, and
/// the weakening and the cut of the two-sided one.
#[derive(Clone, Copy, Debug)]
pub(super) struct Calculus {
    /// Binary Mix.
    pub(super) mix: bool,
    /// The weakening of any formula.
    pub(super) weakening: bool,
    /// The cut rule.
    pub(super) cut: bool,
}

/// Returns the library's one-sided calculus with these optional rules as
/// a record literal, which a reader checks against `Calculus.v` alone.
fn literal(mix: bool, weakening: bool, cut: bool) -> String {
    format!("{{| mix := {mix}; weakening := {weakening}; cut := {cut} |}}")
}

/// Writes the certificate of a derivation for linlog's library, one
/// inference at a time, asking `stop` after each; an open goal or a
/// compact run is refused before anything is written.
pub(super) fn write(
    derivation: &Derivation,
    options: &Options,
    calculus: Calculus,
    out: &mut impl Write,
    mut stop: impl FnMut() -> bool,
) -> Result<(), Error> {
    for inference in derivation.inferences() {
        if inference.times > 1 {
            return Err(Unsupported::Compact.into());
        }
        if inference.rule.rule == Rule::Open {
            return Err(Unsupported::Open.into());
        }
    }
    let mut term = Writer::new(derivation);
    let reading = derivation.reading();
    if options.form == Form::Standalone {
        let own = match reading {
            Some(_) => LINLOG_TWO_SIDED_PRELUDE,
            None => LINLOG_PRELUDE,
        };
        let prelude = options.prelude.as_deref().unwrap_or(own);
        write!(out, "{}\n\n", prelude.trim_end())?;
    }
    let root = derivation.inference(derivation.root());
    write!(term.out, "Lemma {} : ", options.lemma)?;
    let theorem = if let Some(reading) = reading {
        // The library names the calculi without cut; one with cut is a
        // record literal.
        let rules = match (calculus.weakening, calculus.cut) {
            (true, false) => "affine".to_owned(),
            (false, false) => "intuitionistic".to_owned(),
            (weakening, true) => format!("{{| iweakening := {weakening}; icut := true |}}"),
        };
        write!(term.out, "ill (atom := string) {rules} [")?;
        let hypotheses = root
            .sequent
            .iter()
            .map(|m| m.occ())
            .filter(|&o| reading.position(o) == Side::Input);
        for (i, o) in hypotheses.enumerate() {
            if i > 0 {
                term.out.push_str("; ");
            }
            term.iformula(o, false);
        }
        term.out.push_str("] ");
        term.iformula(goal(reading, &root.sequent), true);
        format!("soundness_ill String.eqb String.eqb_spec {rules}")
    } else {
        let Calculus {
            mix,
            weakening,
            cut,
        } = calculus;
        let (rules, theorem) = match (mix, weakening, cut) {
            (false, false, false) => ("classical".to_owned(), "soundness_classical"),
            (true, false, false) => (literal(mix, weakening, cut), "soundness_mix"),
            (false, true, false) => (literal(mix, weakening, cut), "soundness_affine"),
            _ => (literal(mix, weakening, cut), "soundness"),
        };
        write!(term.out, "ll (atom := string) {rules} [")?;
        for (i, m) in root.sequent.iter().enumerate() {
            if i > 0 {
                term.out.push_str("; ");
            }
            term.formula(m.occ(), false);
        }
        term.out.push(']');
        // The general theorem takes the calculus, its instances name it.
        if theorem == "soundness" {
            format!("{theorem} String.eqb String.eqb_spec {rules}")
        } else {
            format!("{theorem} String.eqb String.eqb_spec")
        }
    };
    write!(term.out, ".\nProof.\n  apply ({theorem}\n    ")?;
    walk(derivation, |step| {
        match step {
            Step::Enter(id, _) => term.enter(id),
            Step::Exit(id, _) => {
                for _ in 0..term.closes[id.index()] {
                    term.out.push(')');
                }
            }
        }
        flush(out, &mut term.out, &mut stop)
    })?;
    out.write_str(").\n  vm_compute. reflexivity.\nQed.")?;
    Ok(())
}

/// Returns the goal of a two-sided sequent: its one member in output
/// position.
fn goal(reading: &Reading, sequent: &[Member]) -> OccId {
    sequent
        .iter()
        .map(|m| m.occ())
        .find(|&o| reading.position(o) == Side::Output)
        .expect("a two-sided sequent has its goal")
}

/// The proof term in progress.
struct Writer<'a> {
    /// The derivation being written.
    derivation: &'a Derivation<'a>,
    /// The forest its occurrences index.
    forest: &'a Forest,
    /// The text not yet handed on.
    out: String,
    /// Per inference, the `?` formulas of its sequent that first appear
    /// there, at the root, as a subformula of the rule below or as a cut
    /// formula or its dual, which the one-sided term moves into the
    /// unrestricted zone.
    quests: Vec<Vec<OccId>>,
    /// Per inference, the brackets its term leaves open for its premises.
    closes: Vec<u32>,
    /// Whether the next subterm follows text it needs a space after.
    spaced: bool,
}

impl<'a> Writer<'a> {
    /// Returns the term of a derivation, nothing written yet.
    fn new(derivation: &'a Derivation<'a>) -> Self {
        let forest = derivation.forest();
        let count = derivation.inferences().len();
        let mut quests = vec![Vec::new(); count];
        if derivation.reading().is_none() {
            let is_quest =
                |o: OccId| matches!(forest.sequent().term(forest.term(o)), Term::Quest(_));
            let root = derivation.root();
            quests[root.index()] = derivation
                .inference(root)
                .sequent
                .iter()
                .map(|m| m.occ())
                .filter(|&o| is_quest(o))
                .collect();
            for inference in derivation.inferences() {
                // A cut formula and its dual first appear in the premises.
                if let Some(a) = inference.cut {
                    let a = a.occ();
                    let dual = forest.dual(a).expect("a cut formula has its dual");
                    for (&premise, o) in inference.premises.iter().zip([a, dual]) {
                        let sequent = &derivation.inference(premise).sequent;
                        if is_quest(o) && sequent.iter().any(|m| m.occ() == o) {
                            quests[premise.index()].push(o);
                        }
                    }
                }
                let Some(principal) = inference.principal else {
                    continue;
                };
                let o = inference.sequent[principal].occ();
                let children = [forest.left(o), forest.right(o)];
                for &premise in &inference.premises {
                    let sequent = &derivation.inference(premise).sequent;
                    for child in children.into_iter().flatten() {
                        if is_quest(child) && sequent.iter().any(|m| m.occ() == child) {
                            quests[premise.index()].push(child);
                        }
                    }
                }
            }
        }
        Self {
            derivation,
            forest,
            out: String::new(),
            quests,
            closes: vec![0; count],
            spaced: false,
        }
    }

    /// Writes a space unless the text so far ends where a subterm may
    /// start without one.
    fn separate(&mut self) {
        if self.spaced {
            self.out.push(' ');
            self.spaced = false;
        }
    }

    /// Writes the one-sided formula at `o`, in brackets if `argument` is
    /// set and it is an application.
    fn formula(&mut self, o: OccId, argument: bool) {
        formula(self.forest, &mut self.out, o, argument);
    }

    /// Writes the intuitionistic formula at `o`, in brackets if `argument`
    /// is set and it is an application.
    fn iformula(&mut self, o: OccId, argument: bool) {
        let reading = self.derivation.reading().expect("a two-sided derivation");
        iformula(self.forest, reading, &mut self.out, o, argument);
    }

    /// Writes a node's constructor and the formulas it names, opening a
    /// bracket that its premises fill, or the whole node for a leaf.
    fn node(&mut self, id: InfId, constructor: &str, formulas: &[OccId], two_sided: bool) {
        self.separate();
        let leaf = self.derivation.inference(id).premises.is_empty();
        if !leaf || !formulas.is_empty() {
            self.out.push('(');
        }
        self.out.push_str(constructor);
        for &f in formulas {
            self.out.push(' ');
            if two_sided {
                self.iformula(f, true);
            } else {
                self.formula(f, true);
            }
        }
        if leaf {
            if !formulas.is_empty() {
                self.out.push(')');
            }
        } else {
            self.closes[id.index()] += 1;
        }
        self.spaced = true;
    }

    /// Writes the start of the term of the inference `id`: the moves of
    /// its new `?` formulas and its constructor.
    fn enter(&mut self, id: InfId) {
        let derivation = self.derivation;
        let forest = self.forest;
        let inference = derivation.inference(id);
        for i in 0..self.quests[id.index()].len() {
            let q = self.quests[id.index()][i];
            self.separate();
            self.out.push_str("(Quest ");
            self.formula(forest.left(q).expect("a ? formula"), true);
            self.closes[id.index()] += 1;
            self.spaced = true;
        }
        let principal = inference.principal.map(|p| inference.sequent[p].occ());
        let children: Vec<OccId> = principal
            .into_iter()
            .flat_map(|o| [forest.left(o), forest.right(o)])
            .flatten()
            .collect();
        if let Some(reading) = derivation.reading() {
            self.two_sided(id, reading, principal, &children);
            return;
        }
        let is_quest = |o: OccId| matches!(forest.sequent().term(forest.term(o)), Term::Quest(_));
        match inference.rule.rule {
            Rule::Ax => {
                let literal = inference.sequent[0].occ();
                self.separate();
                self.out.push_str("(Ax ");
                atom(self.forest, &mut self.out, literal);
                self.out.push(')');
                self.spaced = true;
            }
            Rule::Tensor => self.node(id, "Tensor", &children, false),
            Rule::Par => self.node(id, "Par", &children, false),
            Rule::One => self.node(id, "One", &[], false),
            Rule::Bot => self.node(id, "Bot", &[], false),
            Rule::With => self.node(id, "With", &children, false),
            Rule::PlusLeft | Rule::PlusRight => {
                let o = principal.expect("a principal formula");
                let operands = [forest.left(o).unwrap(), forest.right(o).unwrap()];
                let side = if inference.rule.rule == Rule::PlusLeft {
                    "Left"
                } else {
                    "Right"
                };
                self.separate();
                self.out.push_str("(Plus");
                for f in operands {
                    self.out.push(' ');
                    self.formula(f, true);
                }
                self.out.push(' ');
                self.out.push_str(side);
                self.closes[id.index()] += 1;
                self.spaced = true;
            }
            Rule::Top => self.node(id, "Top", &[], false),
            Rule::Promotion => self.node(id, "Bang", &children, false),
            Rule::Dereliction => self.node(id, "Copy", &children, false),
            Rule::Mix => self.node(id, "Mix", &[], false),
            Rule::AffineWeakening => {
                let o = principal.expect("a principal formula");
                // A `?` formula weakened is one the zone has already.
                if !is_quest(o) {
                    self.node(id, "Weaken", &[o], false);
                }
            }
            // Implicit in the unrestricted zone.
            Rule::Contraction | Rule::Weakening => {}
            Rule::Cut => {
                let a = inference.cut.expect("a cut names its formula").occ();
                self.node(id, "Cut", &[a], false);
            }
            Rule::Open => unreachable!("refused before anything is written"),
        }
    }

    /// Writes the constructor of the two-sided inference `id`.
    fn two_sided(
        &mut self,
        id: InfId,
        reading: &Reading,
        principal: Option<OccId>,
        children: &[OccId],
    ) {
        let inference = self.derivation.inference(id);
        let side = inference.rule.side;
        let constructor = match (inference.rule.rule, side) {
            (Rule::Ax, _) => {
                let literal = inference.sequent[0].occ();
                self.separate();
                self.out.push_str("(IAx ");
                atom(self.forest, &mut self.out, literal);
                self.out.push(')');
                self.spaced = true;
                return;
            }
            (Rule::Top, _) => {
                let principal = principal.expect("a principal formula");
                let others: Vec<OccId> = inference
                    .sequent
                    .iter()
                    .map(|m| m.occ())
                    .filter(|&o| o != principal)
                    .collect();
                let right = side == Some(Side::Output);
                let (constructor, context): (_, Vec<OccId>) = if right {
                    ("(ITopR [", others)
                } else {
                    let goal = goal(reading, &inference.sequent);
                    (
                        "(IZeroL [",
                        others.into_iter().filter(|&o| o != goal).collect(),
                    )
                };
                self.separate();
                self.out.push_str(constructor);
                for (i, &o) in context.iter().enumerate() {
                    if i > 0 {
                        self.out.push_str("; ");
                    }
                    self.iformula(o, false);
                }
                self.out.push(']');
                if !right {
                    self.out.push(' ');
                    self.iformula(goal(reading, &inference.sequent), true);
                }
                self.out.push(')');
                self.spaced = true;
                return;
            }
            (Rule::One, _) => "IOneR",
            (Rule::Bot, _) => "IOneL",
            (Rule::Tensor, Some(Side::Output)) => "ITensR",
            (Rule::Tensor, _) => "ILmapL",
            (Rule::Par, Some(Side::Output)) => "ILmapR",
            (Rule::Par, _) => "ITensL",
            (Rule::With, Some(Side::Output)) => "IWithR",
            (Rule::With, _) => "IPlusL",
            (Rule::PlusLeft, Some(Side::Output)) => "IPlusR1",
            (Rule::PlusLeft, _) => "IWithL1",
            (Rule::PlusRight, Some(Side::Output)) => "IPlusR2",
            (Rule::PlusRight, _) => "IWithL2",
            (Rule::Promotion, _) => "IOcR",
            (Rule::Dereliction, _) => "IDe",
            (Rule::Contraction, _) => "ICo",
            (Rule::Weakening, _) => "IWk",
            (Rule::AffineWeakening, _) => {
                let o = principal.expect("a principal formula");
                self.node(id, "IWeaken", &[o], true);
                return;
            }
            (Rule::Cut, _) => {
                // The cut formula as the premise that proves it has it:
                // the occurrence in output position, `A` or its dual.
                let a = inference.cut.expect("a cut names its formula").occ();
                let dual = self.forest.dual(a).expect("a cut formula has its dual");
                let proved = if reading.position(a) == Side::Output {
                    a
                } else {
                    dual
                };
                self.node(id, "ICut", &[proved], true);
                return;
            }
            (Rule::Mix | Rule::Open, _) => {
                unreachable!("no two-sided rule, or refused before anything is written")
            }
        };
        self.node(id, constructor, children, true);
    }
}

/// Writes an atom's name as a Rocq string, `"` doubled.
fn atom(forest: &Forest, out: &mut String, o: OccId) {
    let sequent = forest.sequent();
    let (Term::Atom(a) | Term::DualAtom(a)) = sequent.term(forest.term(o)) else {
        unreachable!("a literal")
    };
    out.push_str(&string(&sequent.atom_names()[a.index()]));
}

/// Writes the one-sided formula at `o` in the library's constructors,
/// in brackets if `argument` is set and it is an application.
fn formula(forest: &Forest, out: &mut String, o: OccId, argument: bool) {
    for visit in Stops::new(o, argument, |k| (forest.left(k), forest.right(k))) {
        match visit {
            Stop::Enter(k, argument) => {
                let t = forest.sequent().term(forest.term(k));
                let nullary = matches!(t, Term::One | Term::Bot | Term::Top | Term::Zero);
                if argument && !nullary {
                    out.push('(');
                }
                let name = match t {
                    Term::Atom(_) => "var ",
                    Term::DualAtom(_) => "covar ",
                    Term::One => "one",
                    Term::Bot => "bot",
                    Term::Top => "top",
                    Term::Zero => "zero",
                    Term::Tensor(..) => "tens ",
                    Term::Par(..) => "parr ",
                    Term::With(..) => "awith ",
                    Term::Plus(..) => "aplus ",
                    Term::Bang(_) => "oc ",
                    Term::Quest(_) => "wn ",
                    Term::Forall(_) | Term::Exists(_) => {
                        unreachable!("the Rocq export refuses a sequent with a binder")
                    }
                };
                out.push_str(name);
                if matches!(t, Term::Atom(_) | Term::DualAtom(_)) {
                    atom(forest, out, k);
                    if argument {
                        out.push(')');
                    }
                }
            }
            Stop::Between(_) => out.push(' '),
            Stop::Exit(_, argument) => {
                if argument {
                    out.push(')');
                }
            }
        }
    }
}

/// Writes the intuitionistic formula at `o`, read on the side of `⊢`
/// its position gives, in brackets if `argument` is set and it is an
/// application.
fn iformula(forest: &Forest, reading: &Reading, out: &mut String, o: OccId, argument: bool) {
    for visit in Stops::new(o, argument, |k| (forest.left(k), forest.right(k))) {
        match visit {
            Stop::Enter(k, argument) => {
                let t = forest.sequent().term(forest.term(k));
                let output = reading.position(k) == Side::Output;
                let nullary = matches!(t, Term::One | Term::Bot | Term::Top | Term::Zero);
                if argument && !nullary {
                    out.push('(');
                }
                let name = match (t, output) {
                    (Term::Atom(_) | Term::DualAtom(_), _) => "ivar ",
                    (Term::One | Term::Bot, _) => "ione",
                    (Term::Top, true) | (Term::Zero, false) => "itop",
                    (Term::Zero, true) | (Term::Top, false) => "izero",
                    (Term::Tensor(..), true) | (Term::Par(..), false) => "itens ",
                    (Term::Par(..), true) | (Term::Tensor(..), false) => "ilmap ",
                    (Term::With(..), true) | (Term::Plus(..), false) => "iwith ",
                    (Term::Plus(..), true) | (Term::With(..), false) => "iplus ",
                    (Term::Bang(_) | Term::Quest(_), _) => "ioc ",
                    (Term::Forall(_) | Term::Exists(_), _) => {
                        unreachable!("the Rocq export refuses a sequent with a binder")
                    }
                };
                out.push_str(name);
                if matches!(t, Term::Atom(_) | Term::DualAtom(_)) {
                    atom(forest, out, k);
                    if argument {
                        out.push(')');
                    }
                }
            }
            Stop::Between(_) => out.push(' '),
            Stop::Exit(_, argument) => {
                if argument {
                    out.push(')');
                }
            }
        }
    }
}

/// Writes the certificate of a disproof for linlog's library: the lemma
/// that the calculus of the disproof's mode does not derive its goal,
/// one-sided over `ll` or, in intuitionistic mode, two-sided over `ill`,
/// proved by the theorem of the refutation's invariant from the atom or
/// the assignment, which the library's test of the invariant accepts by
/// computation. The disproof is checked first, so only a refutation that
/// holds is written.
pub(super) fn disproof(
    disproof: &Disproof,
    options: &Options,
    out: &mut impl Write,
) -> Result<(), Error> {
    disproof.check()?;
    let forest = Forest::new(disproof.sequent())?;
    let goal: Vec<OccId> = match disproof.goal() {
        None => forest.roots().to_vec(),
        // The check found every member in the forest.
        Some(members) => members
            .iter()
            .filter_map(|m| m.occurrence(&forest))
            .collect(),
    };
    let mode = disproof.mode();
    let reading = if mode.is_intuitionistic() {
        Some(Reading::new(&forest).map_err(Error::NotIntuitionistic)?)
    } else {
        None
    };
    // A two-sided statement has one conclusion, which a search's goal has
    // and a disproof read from a file may lack.
    if let Some(reading) = &reading {
        let count = goal
            .iter()
            .filter(|&&o| reading.position(o) == Side::Output)
            .count();
        if count != 1 {
            return Err(Error::GoalOutputs { count });
        }
    }
    if options.form == Form::Standalone {
        let prelude = options
            .prelude
            .as_deref()
            .unwrap_or(LINLOG_REFUTATION_PRELUDE);
        write!(out, "{}\n\n", prelude.trim_end())?;
    }
    let mut text = format!("Lemma {} : ~ ", options.lemma);
    let side = if let Some(reading) = &reading {
        let rules = if mode.is_affine() {
            "affine"
        } else {
            "intuitionistic"
        };
        text.push_str("ill (atom := string) ");
        text.push_str(rules);
        text.push_str(" [");
        let hypotheses = goal
            .iter()
            .copied()
            .filter(|&o| reading.position(o) == Side::Input);
        for (i, o) in hypotheses.enumerate() {
            if i > 0 {
                text.push_str("; ");
            }
            iformula(&forest, reading, &mut text, o, false);
        }
        text.push_str("] ");
        let conclusion = goal
            .iter()
            .copied()
            .find(|&o| reading.position(o) == Side::Output)
            .expect("the goal has one conclusion");
        iformula(&forest, reading, &mut text, conclusion, true);
        "i"
    } else {
        // The refutations hold of the calculi without cut.
        let rules = match (mode.has_mix(), mode.is_affine()) {
            (false, false) => "classical".to_owned(),
            (mix, weakening) => literal(mix, weakening, false),
        };
        text.push_str("ll (atom := string) ");
        text.push_str(&rules);
        text.push_str(" [");
        for (i, &o) in goal.iter().enumerate() {
            if i > 0 {
                text.push_str("; ");
            }
            formula(&forest, &mut text, o, false);
        }
        text.push(']');
        ""
    };
    let names = forest.sequent().atom_names();
    let theorem = match disproof.refutation() {
        Refutation::Classical(Classical { assignment }) => {
            let truths: Vec<String> = (0..names.len())
                .filter(|&a| assignment[a])
                .map(|a| string(&names[a]))
                .collect();
            format!(
                "{side}classical_refutation _ (fun a => existsb (String.eqb a) [{}])",
                truths.join("; ")
            )
        }
        Refutation::Unbalanced(unbalanced) => format!(
            "{side}unbalanced_refutation String.eqb {} _",
            string(&names[unbalanced.atom.index()])
        ),
        Refutation::Equation(_) => format!("{side}equation_refutation _"),
        Refutation::Exhausted | Refutation::OneSided(_) | Refutation::StateEquation(_) => {
            unreachable!("the check certifies three refutations alone")
        }
        Refutation::Saturated(_) => unreachable!("the writer refuses a saturated set first"),
    };
    write!(
        text,
        ".\nProof.\n  apply ({theorem}).\n  vm_compute. reflexivity.\nQed."
    )
    .expect("a string takes the text");
    out.write_str(&text)?;
    Ok(())
}

/// Returns an atom's name as a Rocq string, `"` doubled.
fn string(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}
