// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The witnesses of a session's `∃` steps: read from text, built into its
//! table, and held to the eigenvariables a goal is in the scope of.

use super::{GoalId, Interactive, StepError, TermNode, Witness};
use crate::occurrences::Member;
use crate::occurrences::instances::eigen_names;
use crate::proofs::Rule;
use crate::proofs::derivation::InfId;
use crate::proofs::multiset::Multiset;
use crate::sequents::{Eigen, FoTerm, FoTermId, Symbol};

impl Interactive {
    /// Reads a witness from text: a closed term of the sequent's function
    /// symbols and constants and the session's eigenvariables by the
    /// letters the session writes them with, `f(a, g(b))`. Whether the
    /// goal is in the scope of its eigenvariables is what
    /// [`apply`](Self::apply) decides.
    ///
    /// # Errors
    ///
    /// [`StepError::Witness`] for text that is no such term: an unknown
    /// name, a predicate, a symbol given the wrong number of arguments, or
    /// a stray bracket or comma.
    pub fn witness(&self, text: &str) -> Result<Witness, StepError> {
        let fault = |reason: String| StepError::Witness { reason };
        let fo = &self.forest.sequent().fo;
        let eigens = self
            .fo
            .as_ref()
            .map(|q| eigen_names(fo, &q.shown.eigenvariables()))
            .unwrap_or_default();
        let predicates: Vec<Symbol> = fo.atoms.iter().map(|&(p, _)| p).collect();
        // The tokens: names, brackets and commas.
        let mut tokens = Vec::new();
        let mut name = String::new();
        for c in text.chars().chain([' ']) {
            if c.is_whitespace() || matches!(c, '(' | ')' | ',') {
                if !name.is_empty() {
                    tokens.push(std::mem::take(&mut name));
                }
                if !c.is_whitespace() {
                    tokens.push(c.to_string());
                }
            } else {
                name.push(c);
            }
        }
        // A name is a node, applied to the terms in the brackets after it;
        // the open applications wait on a stack with their arguments.
        let mut nodes: Vec<TermNode> = Vec::new();
        let mut open: Vec<(Symbol, Vec<u32>)> = Vec::new();
        let mut done: Option<u32> = None;
        let mut tokens = tokens.into_iter().peekable();
        while let Some(token) = tokens.next() {
            match token.as_str() {
                "(" | ")" | "," if done.is_none() && token != "(" => {
                    return Err(fault(format!("a term is missing before {token:?}")));
                }
                "(" => return Err(fault("a bracket follows no function symbol".into())),
                "," | ")" => {
                    let (Some((_, args)), Some(arg)) = (open.last_mut(), done.take()) else {
                        return Err(fault(format!("{token:?} closes no application")));
                    };
                    args.push(arg);
                    if token == ")"
                        && let Some((f, args)) = open.pop()
                    {
                        let arity = fo.symbols[f.index()].1 as usize;
                        if args.len() != arity {
                            return Err(fault(format!(
                                "{} takes {arity} arguments, not {}",
                                fo.symbols[f.index()].0,
                                args.len()
                            )));
                        }
                        done = Some(push(&mut nodes, TermNode::App(f, args)));
                    }
                }
                _ if done.is_some() => {
                    return Err(fault(format!("{token:?} follows a term without a comma")));
                }
                name => {
                    let applied = tokens.peek().is_some_and(|t| t == "(");
                    if let Some((&a, _)) = eigens.iter().find(|(_, n)| n.as_str() == name) {
                        if applied {
                            return Err(fault(format!(
                                "the eigenvariable {name} takes no arguments"
                            )));
                        }
                        done = Some(push(&mut nodes, TermNode::Eigen(a)));
                        continue;
                    }
                    let Some(f) = fo.symbols.iter().position(|(s, _)| s == name) else {
                        return Err(fault(format!("{name} is no symbol of the sequent")));
                    };
                    // Fewer symbols than a `u32` counts.
                    let f = Symbol::new(f as u32);
                    if predicates.contains(&f) {
                        return Err(fault(format!("{name} is a predicate, not a term")));
                    }
                    let arity = fo.symbols[f.index()].1;
                    match (applied, arity) {
                        (true, _) => {
                            tokens.next();
                            open.push((f, Vec::new()));
                        }
                        (false, 0) => done = Some(push(&mut nodes, TermNode::App(f, Vec::new()))),
                        (false, _) => {
                            return Err(fault(format!("{name} takes {arity} arguments")));
                        }
                    }
                }
            }
        }
        if !open.is_empty() {
            return Err(fault("a bracket is not closed".into()));
        }
        if done.is_none() {
            return Err(fault("there is no term".into()));
        }
        Ok(Witness::Term(nodes))
    }

    /// Builds a witness given flat into the table for a step on `goal`,
    /// holding it to the sequent's function symbols, to arities, and to
    /// the eigenvariables the goal is in the scope of.
    pub(super) fn witness_term(
        &mut self,
        goal: GoalId,
        nodes: &[TermNode],
    ) -> Result<FoTermId, StepError> {
        let fault = |reason: String| StepError::Witness { reason };
        if nodes.is_empty() {
            return Err(fault("there is no term".into()));
        }
        let scope = self.scope(goal.inf());
        let fo = &self.forest.sequent().fo;
        let q = self.fo.as_mut().expect("a first-order session");
        let names = eigen_names(fo, &q.shown.eigenvariables());
        let predicates: Vec<Symbol> = fo.atoms.iter().map(|&(p, _)| p).collect();
        let mut built: Vec<FoTermId> = Vec::with_capacity(nodes.len());
        for (i, node) in nodes.iter().enumerate() {
            let term = match node {
                TermNode::Eigen(a) => {
                    if !scope.contains(a) {
                        return Err(StepError::Eigenvariable {
                            name: names
                                .get(a)
                                .cloned()
                                .unwrap_or_else(|| format!("#{}", a.get())),
                        });
                    }
                    q.eigen_term(*a)
                }
                TermNode::App(f, args) => {
                    let Some(&(ref name, arity)) = fo.symbols.get(f.index()) else {
                        return Err(fault(format!("symbol {} is none of the sequent", f.get())));
                    };
                    if predicates.contains(f) {
                        return Err(fault(format!("{name} is a predicate, not a term")));
                    }
                    if args.len() != arity as usize {
                        return Err(fault(format!(
                            "{name} takes {arity} arguments, not {}",
                            args.len()
                        )));
                    }
                    let mut items = Vec::with_capacity(args.len());
                    for &arg in args {
                        let Some(&t) = built.get(arg as usize).filter(|_| (arg as usize) < i)
                        else {
                            return Err(fault(format!(
                                "node {i} names node {arg}, which is not before it"
                            )));
                        };
                        items.push(t);
                    }
                    q.apply(*f, &items)
                }
            };
            built.push(term);
        }
        Ok(*built.last().expect("not empty"))
    }

    /// Returns the eigenvariables a goal is in the scope of: those the
    /// `∀` steps below it introduced, and the constants.
    pub(super) fn scope(&self, goal: InfId) -> Vec<Eigen> {
        let mut parent = vec![None; self.inferences.len()];
        for (i, inference) in self.inferences.iter().enumerate() {
            for p in &inference.premises {
                parent[p.index()] = Some(InfId::new(i as u32));
            }
        }
        let mut scope = self
            .fo
            .as_ref()
            .map(|q| q.constants.clone())
            .unwrap_or_default();
        let mut at = parent[goal.index()];
        while let Some(id) = at {
            if self.inferences[id.index()].rule.rule == Rule::Forall
                && let Some(a) = self.introduced(id)
            {
                scope.push(a);
            }
            at = parent[id.index()];
        }
        scope
    }

    /// Returns the body member a closed `∀` or `∃` inference opened: the
    /// one member its premise has that its conclusion less the principal
    /// formula lacks.
    pub(super) fn body(&self, id: InfId) -> Option<Member> {
        let inference = &self.inferences[id.index()];
        let premise = &self.inferences[inference.premises.first()?.index()].sequent;
        let mut rest = Multiset::of(inference.sequent.iter().copied());
        rest.remove(inference.sequent[inference.principal?]);
        Multiset::of(premise.iter().copied())
            .difference(&rest)
            .as_slice()
            .first()
            .copied()
    }

    /// Returns the term a closed `∀` or `∃` inference put for its bound
    /// variable, as the table holds it: the innermost term of its body
    /// member's frame.
    pub(super) fn opened_with(&self, id: InfId) -> Option<FoTermId> {
        let q = self.fo.as_ref()?;
        let (_, frame) = q.instance(self.body(id)?);
        Some(q.table.frame_parts(frame?).1)
    }

    /// Returns the term a closed `∃` inference put for its bound variable
    /// as the shown table numbers it, its bindings followed.
    pub(super) fn shown_witness(&self, id: InfId) -> Option<FoTermId> {
        let q = self.fo.as_ref()?;
        let (_, frame) = q.instance(self.body(id)?);
        Some(q.shown.frame_parts(frame?).1)
    }

    /// Returns the member of the subformula `c` of the formula `m` stands
    /// for, under `m`'s frame.
    ///
    /// # Panics
    ///
    /// For a subformula whose instance the session never made, which no
    /// premise of a step on `m` lacks.
    pub(super) fn child(&self, m: Member, c: crate::occurrences::OccId) -> Member {
        match &self.fo {
            Some(q) => q
                .find(c, q.instance(m).1)
                .expect("a step made its premises' instances"),
            None => Member::from(c),
        }
    }

    /// Returns the eigenvariable a closed `∀` inference introduced.
    pub(super) fn introduced(&self, id: InfId) -> Option<Eigen> {
        let q = self.fo.as_ref()?;
        match q.terms(&self.forest).term(self.opened_with(id)?) {
            FoTerm::Eigen(a) => Some(a),
            FoTerm::Bound(_) | FoTerm::App(..) | FoTerm::Meta(_) => None,
        }
    }
}

/// Appends a node and returns its index; a witness of more nodes than a
/// `u32` counts is more text than any reader holds.
fn push(nodes: &mut Vec<TermNode>, node: TermNode) -> u32 {
    nodes.push(node);
    u32::try_from(nodes.len() - 1).expect("fewer nodes than a u32 counts")
}
