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
//!
//! A derivation of a sequent with a quantifier is written for the
//! library's first-order development: the same terms over first-order
//! formulas, a predicate with its arguments as terms, a bound variable a
//! de Bruijn index and an eigenvariable by its number, every instance
//! with the terms of its frame substituted for its loose bound
//! variables; a `∀` step names its body and its eigenvariable, an `∃`
//! step its body and its witness.

use super::{
    LINLOG_FIRST_ORDER_PRELUDE, LINLOG_FIRST_ORDER_TWO_SIDED_PRELUDE, LINLOG_PRELUDE,
    LINLOG_REFUTATION_PRELUDE, LINLOG_TWO_SIDED_PRELUDE, Options, Unsupported,
};
use crate::Error;
use crate::export::Form;
use crate::export::notation::{Step, flush, walk};
use crate::occurrences::instances::Framing;
use crate::occurrences::{Forest, FrameId, Member, OccId, Reading, Side};
use crate::proofs::{Derivation, InfId, Rule};
use crate::search::{Classical, Disproof, Refutation};
use crate::sequents::{Args, FoTerm, FoTermId, Stop, Stops, Term};
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

/// Returns the record literal of [`literal`] with its fields named by
/// their module, as the first-order development, which does not import
/// `Calculus`, writes it.
fn qualified(mix: bool, weakening: bool, cut: bool) -> String {
    format!(
        "{{| Calculus.mix := {mix}; Calculus.weakening := {weakening}; Calculus.cut := {cut} |}}"
    )
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
    // The first-order development names the propositional calculi's
    // optional rules by their modules.
    let first_order = term.fo.is_some();
    let (atom, ill_module) = if first_order {
        ("symbol", "Intuitionistic.")
    } else {
        ("atom", "")
    };
    if options.form == Form::Standalone {
        let own = match (reading, first_order) {
            (Some(_), false) => LINLOG_TWO_SIDED_PRELUDE,
            (None, false) => LINLOG_PRELUDE,
            (Some(_), true) => LINLOG_FIRST_ORDER_TWO_SIDED_PRELUDE,
            (None, true) => LINLOG_FIRST_ORDER_PRELUDE,
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
            (true, false) => format!("{ill_module}affine"),
            (false, false) => format!("{ill_module}intuitionistic"),
            (weakening, true) => {
                format!("{{| {ill_module}iweakening := {weakening}; {ill_module}icut := true |}}")
            }
        };
        write!(term.out, "ill ({atom} := string) {rules} [")?;
        // The root's members are the sequent's occurrences.
        let hypotheses = root
            .sequent
            .iter()
            .map(|m| m.occ())
            .filter(|&o| reading.position(o) == Side::Input);
        for (i, o) in hypotheses.enumerate() {
            if i > 0 {
                term.out.push_str("; ");
            }
            term.iformula((o, None), 0, false);
        }
        term.out.push_str("] ");
        let goal = goal(derivation, reading, &root.sequent);
        term.iformula(term.parts(goal), 0, true);
        format!("soundness_ill String.eqb String.eqb_spec {rules}")
    } else {
        let Calculus {
            mix,
            weakening,
            cut,
        } = calculus;
        let record = if first_order { qualified } else { literal };
        let (rules, theorem) = match (mix, weakening, cut) {
            (false, false, false) if first_order => {
                ("Calculus.classical".to_owned(), "soundness_classical")
            }
            (false, false, false) => ("classical".to_owned(), "soundness_classical"),
            (true, false, false) => (record(mix, weakening, cut), "soundness_mix"),
            (false, true, false) => (record(mix, weakening, cut), "soundness_affine"),
            _ => (record(mix, weakening, cut), "soundness"),
        };
        write!(term.out, "ll ({atom} := string) {rules} [")?;
        for (i, m) in root.sequent.iter().enumerate() {
            if i > 0 {
                term.out.push_str("; ");
            }
            term.formula((m.occ(), None), 0, false);
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
fn goal(derivation: &Derivation, reading: &Reading, sequent: &[Member]) -> Member {
    *sequent
        .iter()
        .find(|&&m| reading.position(derivation.occurrence(m)) == Side::Output)
        .expect("a two-sided sequent has its goal")
}

/// A formula a node names: an occurrence and the frame its loose bound
/// variables take their terms from, `None` for an occurrence under no
/// binder.
type Instance = (OccId, Option<FrameId>);

/// What writing a first-order formula needs: the forest, whose sequent
/// holds the symbols, the atoms' arguments and the terms, and the proof's
/// instances, where it has them.
#[derive(Clone, Copy)]
struct Fo<'a> {
    /// The forest the occurrences index.
    forest: &'a Forest,
    /// The proof's table and its extension of the terms.
    framing: Option<&'a Framing<'a>>,
}

impl Fo<'_> {
    /// Returns the instance a member stands for.
    fn parts(self, m: Member) -> Instance {
        match self.framing {
            Some(framing) if m.index() >= self.forest.len() => {
                let (o, frame) = framing
                    .table
                    .instance(m)
                    .expect("a member of the proof's table");
                (o, Some(frame))
            }
            _ => (m.occ(), None),
        }
    }

    /// Returns the terms of a frame, the innermost binder's first.
    fn frame(self, frame: Option<FrameId>) -> Vec<FoTermId> {
        match (frame, self.framing) {
            (Some(frame), Some(framing)) => framing.table.frame_terms(frame),
            _ => Vec::new(),
        }
    }

    /// Returns the term at `id` of the sequent's arena or the proof's.
    fn term(self, id: FoTermId) -> FoTerm {
        match self.framing {
            Some(framing) => framing.terms().term(id),
            None => self.forest.sequent().fo.terms[id.index()],
        }
    }

    /// Returns the terms of the argument list `args`.
    fn list(&self, args: Args) -> &[FoTermId] {
        match self.framing {
            Some(framing) => framing.list(args),
            None => self.forest.sequent().fo.list(args),
        }
    }

    /// Returns the term an `∀` or `∃` step's body is opened with: the
    /// innermost term of its instance's frame.
    fn opened_with(self, body: Instance) -> FoTermId {
        let framing = self.framing.expect("a quantifier step has instances");
        let frame = body.1.expect("a body's instance has a frame");
        framing.table.frame_parts(frame).1
    }

    /// Writes the term `id`, its bound variables from `depth` on taken from
    /// `frame`, in brackets if `argument` is set and it is an application;
    /// on an explicit stack, since a witness may be nested as deep as the
    /// search's bindings went.
    fn write_term(
        self,
        out: &mut String,
        id: FoTermId,
        depth: u32,
        frame: &[FoTermId],
        argument: bool,
    ) {
        /// What the walk has yet to write.
        enum Item {
            /// A term, and whether it is an argument of a constructor.
            Term(FoTermId, bool),
            /// Text closing an application.
            Text(&'static str),
        }
        let mut stack = vec![Item::Term(id, argument)];
        while let Some(item) = stack.pop() {
            let (t, argument) = match item {
                Item::Text(text) => {
                    out.push_str(text);
                    continue;
                }
                Item::Term(t, argument) => (t, argument),
            };
            let (open, close) = if argument { ("(", ")") } else { ("", "") };
            match self.term(t) {
                FoTerm::Bound(i) if i < depth => {
                    write!(out, "{open}bound {i}{close}").expect("a string takes the text");
                }
                // A frame holds closed terms.
                FoTerm::Bound(i) => stack.push(Item::Term(frame[(i - depth) as usize], argument)),
                FoTerm::Eigen(a) => {
                    write!(out, "{open}eigen {}{close}", a.get()).expect("a string takes the text");
                }
                FoTerm::App(f, args) => {
                    let name = &self.forest.sequent().fo.symbols[f.index()].0;
                    write!(out, "{open}tapp {} [", string(name)).expect("a string takes the text");
                    stack.push(Item::Text(if argument { "])" } else { "]" }));
                    let list = self.list(args);
                    for (k, &u) in list.iter().enumerate().rev() {
                        stack.push(Item::Term(u, false));
                        if k > 0 {
                            stack.push(Item::Text("; "));
                        }
                    }
                }
                FoTerm::Meta(_) => unreachable!("a proof's table holds no metavariable"),
            }
        }
    }

    /// Writes the formula of `instance`, read one-sided or, with a
    /// reading, on the side of `⊢` its position gives, under `depth`
    /// binders of its own already open, in brackets if `argument` is set
    /// and it is an application.
    fn write_formula(
        self,
        out: &mut String,
        reading: Option<&Reading>,
        instance: Instance,
        depth: u32,
        argument: bool,
    ) {
        let forest = self.forest;
        let frame = self.frame(instance.1);
        let mut depth = depth;
        for visit in Stops::new(instance.0, argument, |k| (forest.left(k), forest.right(k))) {
            match visit {
                Stop::Enter(k, argument) => {
                    let t = forest.sequent().term(forest.term(k));
                    let nullary = matches!(t, Term::One | Term::Bot | Term::Top | Term::Zero);
                    if argument && !nullary {
                        out.push('(');
                    }
                    let name = match reading {
                        None => one_sided(t),
                        Some(reading) => two_sided(t, reading.position(k) == Side::Output),
                    };
                    out.push_str(name);
                    match t {
                        Term::Atom(a) | Term::DualAtom(a) => {
                            let (symbol, args) = forest.sequent().fo.atoms[a.index()];
                            let name = &forest.sequent().fo.symbols[symbol.index()].0;
                            write!(out, "{} [", string(name)).expect("a string takes the text");
                            for (i, &t) in self.list(args).iter().enumerate() {
                                if i > 0 {
                                    out.push_str("; ");
                                }
                                self.write_term(out, t, depth, &frame, false);
                            }
                            out.push(']');
                            if argument {
                                out.push(')');
                            }
                        }
                        Term::Forall(_) | Term::Exists(_) => depth += 1,
                        Term::One
                        | Term::Bot
                        | Term::Top
                        | Term::Zero
                        | Term::Tensor(..)
                        | Term::Par(..)
                        | Term::With(..)
                        | Term::Plus(..)
                        | Term::Bang(_)
                        | Term::Quest(_) => {}
                    }
                }
                Stop::Between(_) => out.push(' '),
                Stop::Exit(k, argument) => {
                    let t = forest.sequent().term(forest.term(k));
                    if matches!(t, Term::Forall(_) | Term::Exists(_)) {
                        depth -= 1;
                    }
                    if argument {
                        out.push(')');
                    }
                }
            }
        }
    }
}

/// Returns the constructor of the one-sided formula `t` as the library
/// names it, with the space before its first argument.
const fn one_sided(t: Term) -> &'static str {
    match t {
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
        Term::Forall(_) => "fall ",
        Term::Exists(_) => "fex ",
    }
}

/// Returns the constructor of the intuitionistic formula the one-sided
/// `t` stands for in output position (`output`) or input position.
const fn two_sided(t: Term, output: bool) -> &'static str {
    match (t, output) {
        (Term::Atom(_) | Term::DualAtom(_), _) => "ivar ",
        (Term::One | Term::Bot, _) => "ione",
        (Term::Top, true) | (Term::Zero, false) => "itop",
        (Term::Zero, true) | (Term::Top, false) => "izero",
        (Term::Tensor(..), true) | (Term::Par(..), false) => "itens ",
        (Term::Par(..), true) | (Term::Tensor(..), false) => "ilmap ",
        (Term::With(..), true) | (Term::Plus(..), false) => "iwith ",
        (Term::Plus(..), true) | (Term::With(..), false) => "iplus ",
        (Term::Bang(_) | Term::Quest(_), _) => "ioc ",
        (Term::Forall(_), true) | (Term::Exists(_), false) => "iall ",
        (Term::Exists(_), true) | (Term::Forall(_), false) => "iex ",
    }
}

/// The proof term in progress.
struct Writer<'a> {
    /// The derivation being written.
    derivation: &'a Derivation<'a>,
    /// The forest its occurrences index.
    forest: &'a Forest,
    /// For a derivation of a sequent with a quantifier, what its
    /// first-order formulas are written with.
    fo: Option<Fo<'a>>,
    /// The text not yet handed on.
    out: String,
    /// Per inference, the `?` formulas of its sequent that first appear
    /// there, at the root, as a subformula of the rule below or as a cut
    /// formula or its dual, which the one-sided term moves into the
    /// unrestricted zone.
    quests: Vec<Vec<Instance>>,
    /// Per inference, the brackets its term leaves open for its premises.
    closes: Vec<u32>,
    /// Whether the next subterm follows text it needs a space after.
    spaced: bool,
}

impl<'a> Writer<'a> {
    /// Returns the term of a derivation, nothing written yet.
    fn new(derivation: &'a Derivation<'a>) -> Self {
        let forest = derivation.forest();
        let fo = forest.has_binders().then(|| Fo {
            forest,
            framing: derivation.framing(),
        });
        let parts = |m: Member| fo.map_or((m.occ(), None), |fo| fo.parts(m));
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
                .map(|&m| parts(m))
                .filter(|&(o, _)| is_quest(o))
                .collect();
            for (i, inference) in derivation.inferences().iter().enumerate() {
                // A cut formula and its dual first appear in the premises.
                if let Some(a) = inference.cut {
                    let (a, frame) = parts(a);
                    let dual = forest.dual(a).expect("a cut formula has its dual");
                    for (&premise, o) in inference.premises.iter().zip([a, dual]) {
                        let sequent = &derivation.inference(premise).sequent;
                        if is_quest(o) && sequent.iter().any(|&m| parts(m) == (o, frame)) {
                            quests[premise.index()].push((o, frame));
                        }
                    }
                }
                let Some(principal) = inference.principal else {
                    continue;
                };
                let (o, frame) = parts(inference.sequent[principal]);
                // A quantifier's body is an instance of its own, opened
                // with the step's term.
                let children: Vec<Instance> =
                    if matches!(inference.rule.rule, Rule::Forall | Rule::Exists) {
                        vec![parts(body(derivation, InfId::new(i as u32)))]
                    } else {
                        [forest.left(o), forest.right(o)]
                            .into_iter()
                            .flatten()
                            .map(|child| (child, frame))
                            .collect()
                    };
                for &premise in &inference.premises {
                    let sequent = &derivation.inference(premise).sequent;
                    for &child in &children {
                        if is_quest(child.0) && sequent.iter().any(|&m| parts(m) == child) {
                            quests[premise.index()].push(child);
                        }
                    }
                }
            }
        }
        Self {
            derivation,
            forest,
            fo,
            out: String::new(),
            quests,
            closes: vec![0; count],
            spaced: false,
        }
    }

    /// Returns the instance a member of the derivation stands for.
    fn parts(&self, m: Member) -> Instance {
        self.fo.map_or((m.occ(), None), |fo| fo.parts(m))
    }

    /// Writes a space unless the text so far ends where a subterm may
    /// start without one.
    fn separate(&mut self) {
        if self.spaced {
            self.out.push(' ');
            self.spaced = false;
        }
    }

    /// Writes the one-sided formula of `instance` under `depth` binders of
    /// its own already open, in brackets if `argument` is set and it is an
    /// application.
    fn formula(&mut self, instance: Instance, depth: u32, argument: bool) {
        match self.fo {
            Some(fo) => fo.write_formula(&mut self.out, None, instance, depth, argument),
            None => formula(self.forest, &mut self.out, instance.0, argument),
        }
    }

    /// Writes the intuitionistic formula of `instance` under `depth`
    /// binders of its own already open, in brackets if `argument` is set
    /// and it is an application.
    fn iformula(&mut self, instance: Instance, depth: u32, argument: bool) {
        let reading = self.derivation.reading().expect("a two-sided derivation");
        match self.fo {
            Some(fo) => fo.write_formula(&mut self.out, Some(reading), instance, depth, argument),
            None => iformula(self.forest, reading, &mut self.out, instance.0, argument),
        }
    }

    /// Writes the literal of an axiom: its atom's name, and for a
    /// first-order one its arguments.
    fn literal(&mut self, m: Member) {
        let instance = self.parts(m);
        match self.fo {
            None => atom(self.forest, &mut self.out, instance.0),
            Some(fo) => {
                // The literal written as a formula is `var "p" [ts]`; the
                // axiom names what follows its constructor.
                let mut text = String::new();
                fo.write_formula(&mut text, None, instance, 0, false);
                let args = text
                    .strip_prefix("var ")
                    .or_else(|| text.strip_prefix("covar "))
                    .expect("a literal");
                self.out.push_str(args);
            }
        }
    }

    /// Writes a node's constructor and the formulas it names, opening a
    /// bracket that its premises fill, or the whole node for a leaf.
    fn node(&mut self, id: InfId, constructor: &str, formulas: &[Instance], two_sided: bool) {
        self.separate();
        let leaf = self.derivation.inference(id).premises.is_empty();
        if !leaf || !formulas.is_empty() {
            self.out.push('(');
        }
        self.out.push_str(constructor);
        for &f in formulas {
            self.out.push(' ');
            if two_sided {
                self.iformula(f, 0, true);
            } else {
                self.formula(f, 0, true);
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

    /// Writes the start of a quantifier step's node: its constructor, the
    /// body of its principal formula (its bound variable `0`) and the
    /// eigenvariable or the witness its body is opened with, opening a
    /// bracket that its premise fills.
    fn quantifier(&mut self, id: InfId, constructor: &str, principal: Instance, two_sided: bool) {
        let fo = self.fo.expect("a quantifier step is first-order");
        let opened = fo.parts(body(self.derivation, id));
        let term = fo.opened_with(opened);
        self.separate();
        self.out.push('(');
        self.out.push_str(constructor);
        self.out.push(' ');
        let body = (
            self.forest.left(principal.0).expect("a binder's body"),
            principal.1,
        );
        if two_sided {
            self.iformula(body, 1, true);
        } else {
            self.formula(body, 1, true);
        }
        self.out.push(' ');
        if constructor_introduces(constructor) {
            // The eigenvariable, by its number.
            let FoTerm::Eigen(a) = fo.term(term) else {
                unreachable!("a ∀ step's body is opened with its eigenvariable")
            };
            write!(self.out, "{}", a.get()).expect("a string takes the text");
        } else {
            fo.write_term(&mut self.out, term, 0, &[], true);
        }
        self.closes[id.index()] += 1;
        self.spaced = true;
    }

    /// Writes the start of the term of the inference `id`: the moves of
    /// its new `?` formulas and its constructor.
    fn enter(&mut self, id: InfId) {
        let derivation = self.derivation;
        let forest = self.forest;
        let inference = derivation.inference(id);
        for i in 0..self.quests[id.index()].len() {
            let (q, frame) = self.quests[id.index()][i];
            self.separate();
            self.out.push_str("(Quest ");
            self.formula((forest.left(q).expect("a ? formula"), frame), 0, true);
            self.closes[id.index()] += 1;
            self.spaced = true;
        }
        let principal = inference
            .principal
            .map(|p| self.parts(inference.sequent[p]));
        let children: Vec<Instance> = principal
            .into_iter()
            .flat_map(|(o, frame)| {
                [forest.left(o), forest.right(o)]
                    .into_iter()
                    .flatten()
                    .map(move |child| (child, frame))
            })
            .collect();
        if let Some(reading) = derivation.reading() {
            self.two_sided(id, reading, principal, &children);
            return;
        }
        let is_quest = |o: OccId| matches!(forest.sequent().term(forest.term(o)), Term::Quest(_));
        match inference.rule.rule {
            Rule::Forall => {
                let principal = principal.expect("a principal formula");
                self.quantifier(id, "Forall", principal, false);
            }
            Rule::Exists => {
                let principal = principal.expect("a principal formula");
                self.quantifier(id, "Exists", principal, false);
            }
            Rule::Ax => {
                let literal = inference.sequent[0];
                self.separate();
                self.out.push_str("(Ax ");
                self.literal(literal);
                self.out.push(')');
                self.spaced = true;
            }
            Rule::Tensor => self.node(id, "Tensor", &children, false),
            Rule::Par => self.node(id, "Par", &children, false),
            Rule::One => self.node(id, "One", &[], false),
            Rule::Bot => self.node(id, "Bot", &[], false),
            Rule::With => self.node(id, "With", &children, false),
            Rule::PlusLeft | Rule::PlusRight => {
                let side = if inference.rule.rule == Rule::PlusLeft {
                    "Left"
                } else {
                    "Right"
                };
                self.separate();
                self.out.push_str("(Plus");
                for &f in &children {
                    self.out.push(' ');
                    self.formula(f, 0, true);
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
                let principal = principal.expect("a principal formula");
                // A `?` formula weakened is one the zone has already.
                if !is_quest(principal.0) {
                    self.node(id, "Weaken", &[principal], false);
                }
            }
            // Implicit in the unrestricted zone.
            Rule::Contraction | Rule::Weakening => {}
            Rule::Cut => {
                let a = self.parts(inference.cut.expect("a cut names its formula"));
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
        principal: Option<Instance>,
        children: &[Instance],
    ) {
        let derivation = self.derivation;
        let inference = derivation.inference(id);
        let side = inference.rule.side;
        let constructor = match (inference.rule.rule, side) {
            (Rule::Forall, Some(Side::Output)) => {
                self.quantifier(id, "IAllR", principal.expect("a principal formula"), true);
                return;
            }
            (Rule::Forall, _) => {
                self.quantifier(id, "IExL", principal.expect("a principal formula"), true);
                return;
            }
            (Rule::Exists, Some(Side::Output)) => {
                self.quantifier(id, "IExR", principal.expect("a principal formula"), true);
                return;
            }
            (Rule::Exists, _) => {
                self.quantifier(id, "IAllL", principal.expect("a principal formula"), true);
                return;
            }
            (Rule::Ax, _) => {
                let literal = inference.sequent[0];
                self.separate();
                self.out.push_str("(IAx ");
                self.literal(literal);
                self.out.push(')');
                self.spaced = true;
                return;
            }
            (Rule::Top, _) => {
                let at = inference.principal.expect("a principal formula");
                let others: Vec<Member> = inference
                    .sequent
                    .iter()
                    .copied()
                    .filter(|&m| m != inference.sequent[at])
                    .collect();
                let right = side == Some(Side::Output);
                let goal = goal(derivation, reading, &inference.sequent);
                let (constructor, context): (_, Vec<Member>) = if right {
                    ("(ITopR [", others)
                } else {
                    (
                        "(IZeroL [",
                        others.into_iter().filter(|&m| m != goal).collect(),
                    )
                };
                self.separate();
                self.out.push_str(constructor);
                for (i, &m) in context.iter().enumerate() {
                    if i > 0 {
                        self.out.push_str("; ");
                    }
                    self.iformula(self.parts(m), 0, false);
                }
                self.out.push(']');
                if !right {
                    self.out.push(' ');
                    self.iformula(self.parts(goal), 0, true);
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
                let principal = principal.expect("a principal formula");
                self.node(id, "IWeaken", &[principal], true);
                return;
            }
            (Rule::Cut, _) => {
                // The cut formula as the premise that proves it has it:
                // the occurrence in output position, `A` or its dual.
                let (a, frame) = self.parts(inference.cut.expect("a cut names its formula"));
                let dual = self.forest.dual(a).expect("a cut formula has its dual");
                let proved = if reading.position(a) == Side::Output {
                    a
                } else {
                    dual
                };
                self.node(id, "ICut", &[(proved, frame)], true);
                return;
            }
            (Rule::Mix | Rule::Open, _) => {
                unreachable!("no two-sided rule, or refused before anything is written")
            }
        };
        self.node(id, constructor, children, true);
    }
}

/// Returns whether a quantifier step's constructor names the eigenvariable
/// it introduces, rather than a witness.
fn constructor_introduces(constructor: &str) -> bool {
    matches!(constructor, "Forall" | "IAllR" | "IExL")
}

/// Returns the member of the body a quantifier step's premise holds in
/// place of its principal formula: the one member of the premise's
/// sequent that its conclusion's lacks, where both are in ascending
/// order.
fn body(derivation: &Derivation, id: InfId) -> Member {
    let inference = derivation.inference(id);
    let premise = &derivation.inference(inference.premises[0]).sequent;
    let conclusion = &inference.sequent;
    let mut j = 0;
    for &m in premise {
        // The principal formula is the one member of the conclusion the
        // premise lacks.
        while j < conclusion.len() && conclusion[j] < m {
            j += 1;
        }
        if conclusion.get(j) == Some(&m) {
            j += 1;
        } else {
            return m;
        }
    }
    unreachable!("a quantifier step's premise holds its body")
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
                        unreachable!("a sequent with a binder is written by `Fo`")
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
                        unreachable!("a sequent with a binder is written by `Fo`")
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
    // The library's counts and equation are of propositional sequents.
    if forest.has_binders() {
        return Err(super::Unsupported::Quantifiers.into());
    }
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
        Refutation::UnbalancedSymbol(_) | Refutation::OneSidedSymbol(_) => {
            unreachable!("the writer refuses a sequent with a quantifier first")
        }
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
