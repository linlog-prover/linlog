// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The certificate of an ordinary sequent: a lemma over `Prop` whose
//! proof is a term made from a checked derivation of LK or LJ, one
//! construction per rule. An LJ sequent `Γ ⊢ C` is a term of type `C`
//! (`False` when nothing is right of `⊢`) with a variable per hypothesis;
//! an LK sequent `Γ ⊢ Δ` a term of type `False` with a variable per
//! hypothesis and one of type `~ D` per formula `D` of `Δ`, which the
//! rules that move a formula to the right close with the excluded middle
//! (`NNPP` of the standard library). The terms are written by a walk with
//! a stack of its own, so a derivation of any height is written.

use super::derivation::Side;
use super::{Derivation, Formulas, Logic, NodeId, Rule, Symbols};
use crate::Error;
use crate::export::Form;
use crate::export::rocq::{CLASSICAL_PRELUDE, Options, identifiers};
use crate::hash::HashMap;
use crate::proofs::InfId;
use std::fmt::Write;

/// The names the terms use, which an atom never gets.
const USED: &[&str] = &[
    "conj",
    "or_introl",
    "or_intror",
    "proj1",
    "proj2",
    "I",
    "True",
    "False",
    "False_ind",
    "NNPP",
    "iff",
    "and",
    "or",
    "not",
];

/// How Rocq spells the connectives.
const ROCQ: Symbols = Symbols {
    not: "~ ",
    and: "/\\",
    or: "\\/",
    implies: "->",
    iff: "<->",
    truth: "True",
    falsity: "False",
};

/// What the walk has yet to do.
enum Task {
    /// Write the term of an inference.
    Enter(InfId),
    /// Write text.
    Text(String),
    /// Give formulas names for the premise that follows.
    Bind(Vec<((NodeId, Side), String)>),
    /// Take back the names of the last `Bind` of this many.
    Unbind(Vec<(NodeId, Side)>),
}

/// The writer of one certificate.
struct Certificate<'a> {
    /// The derivation.
    derivation: &'a Derivation,
    /// The Rocq identifier of every atom.
    atoms: Vec<String>,
    /// The variables in scope, per formula and side, the innermost last.
    names: HashMap<(NodeId, Side), Vec<String>>,
    /// How many variables were made.
    fresh: u64,
}

impl Certificate<'_> {
    /// Returns a formula in Rocq's syntax, in brackets if binary or a
    /// negation, so that it stands as an argument.
    fn formula(&self, id: NodeId) -> String {
        let mut out = String::new();
        let formulas: &Formulas = self.derivation.formulas();
        // A negation is an application, which an argument needs in
        // brackets as much as a binary formula.
        let negation = matches!(formulas.node(id), super::Node::Not(_));
        if negation {
            out.push('(');
        }
        formulas.write(&mut out, id, true, &ROCQ, |o, a| {
            o.push_str(&self.atoms[a as usize])
        });
        if negation {
            out.push(')');
        }
        out
    }

    /// Returns a new variable's name, with a `'` no atom has.
    fn name(&mut self, letter: char) -> String {
        self.fresh += 1;
        format!("{letter}'{}", self.fresh)
    }

    /// Returns the variable of a formula on a side.
    fn of(&self, node: NodeId, side: Side) -> String {
        self.names
            .get(&(node, side))
            .and_then(|v| v.last())
            .cloned()
            .expect("a checked derivation names only formulas in scope")
    }

    /// Returns the type of an LJ sequent's term: its goal, or `False`.
    fn goal(&self, id: InfId) -> String {
        match self.derivation.inference(id).right.first() {
            Some(&c) => self.formula(c),
            None => "False".to_owned(),
        }
    }

    /// Returns the tasks that write the term of an inference: text and
    /// premises, each premise between the names it binds and their end.
    fn tasks(&mut self, id: InfId) -> Vec<Task> {
        use Rule::*;
        use Side::{Left, Right};
        let d = self.derivation;
        let inference = d.inference(id);
        let classical = d.logic() == Logic::Classical;
        let f = d.formulas();
        let principal = inference
            .principal
            .map(|(side, at)| inference.side(side)[at]);
        let p = principal.unwrap_or(NodeId(0));
        let operands = f.node(p).operands();
        let (a, b) = (operands.0.unwrap_or(p), operands.1.unwrap_or(p));
        let premises = &inference.premises;
        let text = |s: String| Task::Text(s);
        // A premise with the formulas given bound to new names.
        let premise = |n: usize, binds: Vec<((NodeId, Side), String)>| {
            let keys = binds.iter().map(|(k, _)| *k).collect();
            [
                Task::Bind(binds),
                Task::Enter(premises[n]),
                Task::Unbind(keys),
            ]
        };
        let fa = self.formula(a);
        let fb = self.formula(b);
        let mut tasks = Vec::new();
        // The term of a premise whose goal moves to the right, classically:
        // `NNPP A (fun k : ~ A => premise)`.
        let excluded = |this: &mut Self, n: usize, node: NodeId, tasks: &mut Vec<Task>| {
            let k = this.name('k');
            let shown = this.formula(node);
            tasks.push(text(format!("(NNPP {shown} (fun {k} : ~ {shown} => ")));
            tasks.extend(premise(n, vec![((node, Right), k)]));
            tasks.push(text("))".to_owned()));
        };
        match inference.rule {
            Axiom => {
                let (x, y) = (inference.left[0], inference.right[0]);
                let h = self.of(x, Left);
                tasks.push(text(if classical {
                    format!("({} {h})", self.of(y, Right))
                } else {
                    h
                }));
            }
            WeakenLeft | ContractLeft | ContractRight | TrueLeft | FalseRight => {
                tasks.extend(premise(0, vec![]));
            }
            WeakenRight if classical => tasks.extend(premise(0, vec![])),
            WeakenRight => {
                tasks.push(text(format!("(False_ind {} ", self.goal(id))));
                tasks.extend(premise(0, vec![]));
                tasks.push(text(")".to_owned()));
            }
            TrueRight if classical => tasks.push(text(format!("({} I)", self.of(p, Right)))),
            TrueRight => tasks.push(text("I".to_owned())),
            FalseLeft if classical => tasks.push(text(self.of(p, Left))),
            FalseLeft => tasks.push(text(format!(
                "(False_ind {} {})",
                self.goal(id),
                self.of(p, Left)
            ))),
            AndLeft | AndLeft1 | AndLeft2 | IffLeft | IffLeft1 | IffLeft2 => {
                let h = self.of(p, Left);
                let iff = matches!(inference.rule, IffLeft | IffLeft1 | IffLeft2);
                let (x, y) = if iff { implications(f, a, b) } else { (a, b) };
                let pair = if iff {
                    format!("({h} : {} /\\ {})", self.formula(x), self.formula(y))
                } else {
                    h
                };
                let mut binds = Vec::new();
                let mut lets = String::from("(");
                for (keep, node, projection) in [
                    (!matches!(inference.rule, AndLeft2 | IffLeft2), x, "proj1"),
                    (!matches!(inference.rule, AndLeft1 | IffLeft1), y, "proj2"),
                ] {
                    if keep {
                        let n = self.name('h');
                        write!(
                            lets,
                            "let {n} : {} := {projection} {pair} in ",
                            self.formula(node)
                        )
                        .unwrap();
                        binds.push(((node, Left), n));
                    }
                }
                tasks.push(text(lets));
                tasks.extend(premise(0, binds));
                tasks.push(text(")".to_owned()));
            }
            AndRight | IffRight => {
                let iff = inference.rule == IffRight;
                let (x, y) = if iff { implications(f, a, b) } else { (a, b) };
                let cast = format!(" : {} /\\ {}", self.formula(x), self.formula(y));
                if classical {
                    tasks.push(text(format!("({} (conj ", self.of(p, Right))));
                    excluded(self, 0, x, &mut tasks);
                    tasks.push(text(" ".to_owned()));
                    excluded(self, 1, y, &mut tasks);
                    tasks.push(text(format!("{cast}))")));
                } else {
                    tasks.push(text("(conj ".to_owned()));
                    tasks.extend(premise(0, vec![]));
                    tasks.push(text(" ".to_owned()));
                    tasks.extend(premise(1, vec![]));
                    tasks.push(text(format!("{cast})")));
                }
            }
            OrLeft => {
                let h = self.of(p, Left);
                let (h1, h2) = (self.name('h'), self.name('h'));
                let goal = if classical {
                    "False".to_owned()
                } else {
                    self.goal(id)
                };
                tasks.push(text(format!(
                    "(match {h} return {goal} with or_introl {h1} => "
                )));
                tasks.extend(premise(0, vec![((a, Left), h1)]));
                tasks.push(text(format!(" | or_intror {h2} => ")));
                tasks.extend(premise(1, vec![((b, Left), h2)]));
                tasks.push(text(" end)".to_owned()));
            }
            OrRight => {
                let k = self.of(p, Right);
                let (k1, k2) = (self.name('k'), self.name('k'));
                tasks.push(text(format!(
                    "(let {k1} : ~ {fa} := fun x' => {k} (or_introl x') in \
                     let {k2} : ~ {fb} := fun x' => {k} (or_intror x') in "
                )));
                tasks.extend(premise(0, vec![((a, Right), k1), ((b, Right), k2)]));
                tasks.push(text(")".to_owned()));
            }
            OrRight1 | OrRight2 => {
                let (constructor, x) = if inference.rule == OrRight1 {
                    ("or_introl", a)
                } else {
                    ("or_intror", b)
                };
                if classical {
                    tasks.push(text(format!("({} ({constructor} ", self.of(p, Right))));
                    excluded(self, 0, x, &mut tasks);
                    tasks.push(text("))".to_owned()));
                } else {
                    tasks.push(text(format!("({constructor} ")));
                    tasks.extend(premise(0, vec![]));
                    tasks.push(text(")".to_owned()));
                }
            }
            ImpliesRight | NotRight => {
                let h = self.name('h');
                if classical {
                    tasks.push(text(format!("({} (fun {h} : {fa} => ", self.of(p, Right))));
                } else {
                    tasks.push(text(format!("(fun {h} : {fa} => ")));
                }
                if classical && inference.rule == ImpliesRight {
                    let k = self.name('k');
                    tasks.push(text(format!("NNPP {fb} (fun {k} : ~ {fb} => ")));
                    tasks.extend(premise(0, vec![((a, Left), h), ((b, Right), k)]));
                    tasks.push(text(")".to_owned()));
                } else {
                    tasks.extend(premise(0, vec![((a, Left), h)]));
                }
                tasks.push(text(if classical { "))" } else { ")" }.to_owned()));
            }
            NotLeft if premises.len() == 1 => {
                let k = self.name('k');
                tasks.push(text(format!(
                    "(let {k} : ~ {fa} := {} in ",
                    self.of(p, Left)
                )));
                tasks.extend(premise(0, vec![((a, Right), k)]));
                tasks.push(text(")".to_owned()));
            }
            ImpliesLeft | NotLeft => {
                let h = self.of(p, Left);
                let (consequent, shown) = if inference.rule == NotLeft {
                    let falsity = f.ids.get(&super::Node::False).copied();
                    (falsity.expect("a checked ¬L has ⊥"), "False".to_owned())
                } else {
                    (b, fb.clone())
                };
                let h2 = self.name('h');
                tasks.push(text(format!("(let {h2} : {shown} := {h} ")));
                if classical {
                    excluded(self, 0, a, &mut tasks);
                } else {
                    tasks.push(text("(".to_owned()));
                    tasks.extend(premise(0, vec![]));
                    tasks.push(text(")".to_owned()));
                }
                tasks.push(text(" in ".to_owned()));
                tasks.extend(premise(1, vec![((consequent, Left), h2)]));
                tasks.push(text(")".to_owned()));
            }
        }
        tasks
    }
}

/// Returns the implications `A → B` and `B → A` an equivalence of `a` and
/// `b` stands for, which a checked derivation's arena holds.
fn implications(formulas: &Formulas, a: NodeId, b: NodeId) -> (NodeId, NodeId) {
    let id = |node| {
        formulas
            .ids
            .get(&node)
            .copied()
            .expect("a checked derivation holds the implications of its equivalences")
    };
    (
        id(super::Node::Implies(a, b)),
        id(super::Node::Implies(b, a)),
    )
}

/// Writes the certificate of a derivation, which must have passed
/// [`Derivation::check`]: the lemma `options.lemma` stating the sequent
/// over `Prop`, with a variable for every atom, and its proof term;
/// standalone, it starts with `options.prelude`, by default the import of
/// the standard library's excluded middle for a classical certificate and
/// nothing for one of LJ. Asks `stop` after each inference.
pub(crate) fn write(
    derivation: &Derivation,
    options: &Options,
    out: &mut impl Write,
    mut stop: impl FnMut() -> bool,
) -> Result<(), Error> {
    let classical = derivation.logic() == Logic::Classical;
    let formulas = derivation.formulas();
    let mut atoms = identifiers(formulas.atom_names(), options.lemma.as_str());
    for atom in &mut atoms {
        while USED.contains(&atom.as_str()) {
            atom.push('\'');
        }
    }
    let mut certificate = Certificate {
        derivation,
        atoms,
        names: HashMap::default(),
        fresh: 0,
    };
    if options.form == Form::Standalone {
        let prelude = match &options.prelude {
            Some(prelude) => Some(prelude.as_str()),
            None if classical => Some(CLASSICAL_PRELUDE),
            None => None,
        };
        if let Some(prelude) = prelude {
            write!(out, "{}\n\n", prelude.trim_end())?;
        }
    }
    let root = derivation.inference(derivation.root());
    let mut statement = String::new();
    let mut binders = String::new();
    if !certificate.atoms.is_empty() {
        let all = certificate.atoms.join(" ");
        write!(statement, "forall {all} : Prop, ").unwrap();
        write!(binders, "({all} : Prop) ").unwrap();
    }
    for &h in &root.left {
        let name = certificate.name('h');
        let shown = certificate.formula(h);
        write!(statement, "{shown} -> ").unwrap();
        write!(binders, "({name} : {shown}) ").unwrap();
        certificate
            .names
            .entry((h, Side::Left))
            .or_default()
            .push(name);
    }
    let mut body = String::new();
    let mut closing = String::new();
    let right: Vec<String> = root.right.iter().map(|&d| certificate.formula(d)).collect();
    match right.len() {
        0 => statement.push_str("False"),
        _ if !classical => statement.push_str(&right[0]),
        n => {
            let disjunction = right.join(" \\/ ");
            statement.push_str(&disjunction);
            if n == 1 {
                let k = certificate.name('k');
                write!(body, "NNPP {} (fun {k} : ~ {} => ", right[0], right[0]).unwrap();
                certificate
                    .names
                    .entry((root.right[0], Side::Right))
                    .or_default()
                    .push(k);
            } else {
                let k = certificate.name('k');
                write!(
                    body,
                    "NNPP ({disjunction}) (fun {k} : ~ ({disjunction}) => "
                )
                .unwrap();
                for (i, &d) in root.right.iter().enumerate() {
                    let ki = certificate.name('k');
                    let mut injection = "x'".to_owned();
                    if i + 1 < n {
                        injection = format!("(or_introl {injection})");
                    }
                    for _ in 0..i {
                        injection = format!("(or_intror {injection})");
                    }
                    write!(
                        body,
                        "let {ki} : ~ {} := fun x' => {k} {injection} in ",
                        right[i]
                    )
                    .unwrap();
                    certificate
                        .names
                        .entry((d, Side::Right))
                        .or_default()
                        .push(ki);
                }
            }
            closing.push(')');
        }
    }
    // A statement without atoms or hypotheses binds nothing, and `fun =>`
    // is no term.
    let lambda = if binders.is_empty() {
        String::new()
    } else {
        format!("fun {binders}=> ")
    };
    write!(
        out,
        "Lemma {} : {statement}.\nProof.\n  exact ({lambda}{body}",
        options.lemma
    )?;
    let mut stack = vec![Task::Enter(derivation.root())];
    while let Some(task) = stack.pop() {
        match task {
            Task::Enter(id) => {
                let tasks = certificate.tasks(id);
                stack.extend(tasks.into_iter().rev());
                if stop() {
                    return Err(Error::Refused(crate::limits::Refusal::Stopped {
                        phase: crate::limits::Phase::Write,
                    }));
                }
            }
            Task::Text(text) => out.write_str(&text)?,
            Task::Bind(binds) => {
                for (key, name) in binds {
                    certificate.names.entry(key).or_default().push(name);
                }
            }
            Task::Unbind(keys) => {
                for key in keys {
                    if let Some(names) = certificate.names.get_mut(&key) {
                        names.pop();
                    }
                }
            }
        }
    }
    write!(out, "{closing}).\nQed.\n")?;
    Ok(())
}
