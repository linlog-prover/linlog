// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The steps of a session: the rules a goal admits, a rule applied and
//! expanded into its premises, undo, and a goal closed by the search.

use super::quantified::Clash;
use super::{Applicable, Closed, GoalId, Interactive, Needs, Step, StepError, Witness};
use crate::Error;
use crate::limits::{Limits, Progress};
use crate::occurrences::instances::eigen_names;
use crate::occurrences::{Member, OccId, Reading, Side};
use crate::proofs::derivation::{Derivation, InfId, Inference, ViewOptions};
use crate::proofs::multiset::Multiset;
use crate::proofs::{Branch, CheckError, Named, Proof, Rule};
use crate::search::{self, Options, Verdict, focus};
use crate::sequents::Kind;

impl Interactive {
    /// Returns the sequent of an open goal, or the refusal.
    pub(super) fn open(&self, id: GoalId) -> Result<Vec<OccId>, StepError> {
        let goal = self.goal(id)?;
        Ok(goal.iter().map(|&m| self.occurrence(m)).collect())
    }

    /// Returns the rules that can act on the formula at `position` of the
    /// open goal in this mode, by the formula's connective and the mode
    /// alone: the axiom on a literal, `⊗ ⅋ 1 ⊥ & ⊕₁ ⊕₂ ⊤ !` on their
    /// connectives, `?d ?c ?w` on a `?` formula, `wk` on any other formula
    /// in affine mode, and Mix on any formula when the mode has it; in
    /// intuitionistic mode by their two-sided names. Nothing acts on `0`.
    /// Whether the goal's context lets a rule apply (the axiom's dual, the
    /// context of `1` and `!`, a valid split, one succedent) is what
    /// [`apply`](Self::apply) decides.
    ///
    /// # Errors
    ///
    /// [`StepError::NoGoal`] for a goal that is not open, and
    /// [`StepError::NoFormula`] for a position outside its sequent.
    pub fn rules(&self, goal: GoalId, position: usize) -> Result<Vec<Applicable>, StepError> {
        let sequent = self.open(goal)?;
        let o = formula_at(&sequent, position)?;
        use Rule::*;
        let mut rules = match self.forest.kind(o) {
            Kind::Atom | Kind::DualAtom => vec![Ax],
            Kind::Tensor => vec![Tensor],
            Kind::Par => vec![Par],
            Kind::One => vec![One],
            Kind::Bot => vec![Bot],
            Kind::With => vec![With],
            Kind::Plus => vec![PlusLeft, PlusRight],
            Kind::Top => vec![Top],
            Kind::Zero => vec![],
            Kind::Bang => vec![Promotion],
            Kind::Quest => vec![Dereliction, Contraction, Weakening],
            Kind::Forall => vec![Forall],
            Kind::Exists => vec![Exists],
        };
        if self.mode.affine && self.forest.kind(o) != Kind::Quest {
            rules.push(AffineWeakening);
        }
        if self.mode.mix {
            rules.push(Mix);
        }
        let reading = self.reading();
        Ok(rules
            .into_iter()
            .map(|rule| Applicable {
                rule: Named::applied(rule, reading.as_ref(), o),
                needs: match rule {
                    Tensor | Mix => Needs::Split,
                    Exists => Needs::Witness,
                    Ax | Par | One | Bot | With | PlusLeft | PlusRight | Top | Promotion
                    | Dereliction | Contraction | Weakening | AffineWeakening | Cut | Forall
                    | Open => Needs::Nothing,
                },
            })
            .collect())
    }

    /// Applies a step to the open goal and returns the goals it opens, in
    /// the rule's order of premises; a closed leaf opens none. The split
    /// is what a `⊗` or Mix needs ([`Step::left`]), and no other rule takes
    /// one. In intuitionistic mode the rule may be given by its classical
    /// or its two-sided name and is recorded by the latter. Fails with what
    /// the rule needed, and changes nothing then.
    ///
    /// With quantifiers, `∀R` and `∃L` (`∀` one-sided) open the body with
    /// a fresh eigenvariable; `∃R` and `∀L` (`∃` one-sided) with the
    /// step's witness, or with a metavariable where it is open, which a
    /// later axiom or [`close`](Self::close) binds and [`undo`](Self::undo)
    /// of that step unbinds; and the axiom unifies its two literals' terms,
    /// binding open witnesses, with the occurs check and the eigenvariable
    /// condition: an open witness never takes an eigenvariable made after
    /// it.
    ///
    /// # Errors
    ///
    /// A [`StepError`] that names what the rule needed: an open goal, a
    /// formula at the position, a rule that acts on it in the mode, a split
    /// that fits, the dual of an axiom, an empty context for `1` and `!`,
    /// a witness only where the rule takes one, a term of the sequent's
    /// symbols and eigenvariables in scope.
    pub fn apply(&mut self, goal: GoalId, step: &Step) -> Result<Vec<GoalId>, StepError> {
        let sequent = self.open(goal)?;
        let (position, rule) = (step.position, step.rule);
        let left = step.left_positions(sequent.len())?;
        let left = &left[..];
        let reading = self.reading();
        let (rule, premises) = {
            let o = formula_at(&sequent, position)?;
            let rule = match &reading {
                Some(reading) => {
                    let named = Named::applied(rule.rule, Some(reading), o);
                    if rule.side.is_some() && rule != named {
                        return Err(StepError::Rule { rule, position });
                    }
                    named
                }
                None => rule,
            };
            (
                rule,
                self.expand(reading.as_ref(), &sequent, position, rule, left)?,
            )
        };
        if step.witness != Witness::Open && rule.rule != Rule::Exists {
            return Err(StepError::NoWitness { rule });
        }
        let premises = if self.fo.is_some() {
            self.framed_premises(goal, position, rule.rule, left, &step.witness)?
        } else {
            premises.into_iter().map(Multiset::into_members).collect()
        };
        Ok(self.record(goal, position, rule, premises))
    }

    /// Returns the premises of a rule on a goal of a first-order session
    /// as members, the checks on occurrences passed: the children of the
    /// principal formula under its frame, the body of a binder under its
    /// frame pushed with a fresh eigenvariable, the witness given or a
    /// fresh metavariable; and for the axiom none, its literals' terms
    /// unified.
    pub(super) fn framed_premises(
        &mut self,
        goal: GoalId,
        position: usize,
        rule: Rule,
        left: &[usize],
        witness: &Witness,
    ) -> Result<Vec<Vec<Member>>, StepError> {
        use Rule::*;
        let members = self.goal(goal)?.to_vec();
        let m = members[position];
        // The number the step will have, under which its bindings are
        // trailed: fewer steps than inferences, which an `InfId` numbers.
        let step = self.history.len() as u32;
        if rule == Ax {
            let [x, y] = members[..] else {
                unreachable!("the occurrences' check holds two literals");
            };
            let forest = &self.forest;
            let q = self.fo.as_mut().expect("a first-order session");
            let lx = q.literal(forest, x);
            let ly = q.literal(forest, y);
            let no_dual = StepError::NoDual { position };
            if forest.kind(q.instance(x).0) == forest.kind(q.instance(y).0) {
                return Err(no_dual);
            }
            match (lx, ly) {
                (Some((p, a)), Some((r, b))) if p == r => {
                    let pairs = a.into_iter().zip(b).collect();
                    match q.unify(forest, pairs, step) {
                        Ok(()) => {}
                        Err(Clash::Terms) => return Err(no_dual),
                        Err(Clash::Eigen(a)) => {
                            let names =
                                eigen_names(&forest.sequent().fo, &q.shown.eigenvariables());
                            return Err(StepError::Eigenvariable {
                                name: names[&a].clone(),
                            });
                        }
                    }
                }
                (None, None) if forest.atom(q.instance(x).0) == forest.atom(q.instance(y).0) => {}
                _ => return Err(no_dual),
            }
            self.show();
            return Ok(Vec::new());
        }
        let opened = match (rule, witness) {
            (Forall, _) => {
                let q = self.fo.as_mut().expect("a first-order session");
                let Some((_, a)) = q.fresh_eigen() else {
                    return Err(StepError::Witness {
                        reason: "the session has made every eigenvariable a u32 numbers".into(),
                    });
                };
                Some(a)
            }
            (Exists, Witness::Open) => {
                let q = self.fo.as_mut().expect("a first-order session");
                Some(q.fresh_meta())
            }
            (Exists, Witness::Term(nodes)) => Some(self.witness_term(goal, nodes)?),
            _ => None,
        };
        let forest = &self.forest;
        let q = self.fo.as_mut().expect("a first-order session");
        let (o, frame) = q.instance(m);
        // The instances the premises name, made where they are new.
        for c in [forest.left(o), forest.right(o)].into_iter().flatten() {
            if !forest.kind(o).is_binder() {
                q.member(c, frame);
            }
        }
        let body = opened.map(|t| {
            let pushed = q.frame(frame, t);
            q.member(forest.left(o).expect("a binder's body"), Some(pushed))
        });
        let premises = self
            .mirror(&members, position, rule, left, body)
            .expect("the premises' instances were made");
        self.show();
        Ok(premises)
    }

    /// Returns the premises of a rule on a first-order session's goal as
    /// members, as the rule yields them from the goal's: the principal
    /// formula's children under its frame, the binder's `body`; `None`
    /// where an instance they name was never made. The checks on
    /// occurrences are passed.
    pub(super) fn mirror(
        &self,
        members: &[Member],
        position: usize,
        rule: Rule,
        left: &[usize],
        body: Option<Member>,
    ) -> Option<Vec<Vec<Member>>> {
        use Rule::*;
        let forest = &self.forest;
        let q = self.fo.as_ref().expect("a first-order session");
        let m = members[position];
        let (o, frame) = q.instance(m);
        let child = |c: Option<OccId>| q.find(c.expect("a child of the connective"), frame);
        let l = matches!(
            rule,
            Tensor | Par | With | PlusLeft | Promotion | Dereliction
        )
        .then(|| child(forest.left(o)))
        .map(|c| c.ok_or(()));
        let r = matches!(rule, Tensor | Par | With | PlusRight)
            .then(|| child(forest.right(o)))
            .map(|c| c.ok_or(()));
        let (l, r) = (l.transpose().ok()?, r.transpose().ok()?);
        let mut rest: Vec<Member> = members.to_vec();
        rest.remove(position);
        let mut going_left: Vec<Member> = left.iter().map(|&p| members[p]).collect();
        let mut going_right = Multiset::of(rest.iter().copied())
            .difference(&Multiset::of(going_left.iter().copied()))
            .as_slice()
            .to_vec();
        let with = |added: &[Option<Member>]| {
            let mut premise = rest.clone();
            premise.extend(added.iter().flatten());
            premise
        };
        let mut premises = match rule {
            Ax | One | Top => vec![],
            Tensor => {
                going_left.push(l?);
                going_right.push(r?);
                vec![going_left, going_right]
            }
            Mix => {
                going_left.push(m);
                vec![going_left, going_right]
            }
            Par => vec![with(&[l, r])],
            Bot | Weakening | AffineWeakening => vec![with(&[])],
            With => vec![with(&[l]), with(&[r])],
            PlusLeft | Promotion | Dereliction => vec![with(&[l])],
            PlusRight => vec![with(&[r])],
            Contraction => vec![with(&[Some(m), Some(m)])],
            Forall | Exists => vec![with(&[Some(body?)])],
            Cut | Open => unreachable!("no step of the rule reaches here"),
        };
        for premise in &mut premises {
            premise.sort_unstable();
        }
        Some(premises)
    }

    /// Makes the shown instances of a first-order session again after a
    /// change.
    pub(super) fn show(&mut self) {
        if let Some(q) = self.fo.as_mut() {
            q.show(&self.forest);
        }
    }

    /// Closes an open goal by a rule at a position with the premises
    /// [`expand`](Self::expand) computed, opens the premises as goals, and
    /// returns them.
    pub(super) fn record(
        &mut self,
        goal: GoalId,
        position: usize,
        rule: Named,
        premises: Vec<Vec<Member>>,
    ) -> Vec<GoalId> {
        let principal = rule.rule.has_principal().then_some(position);
        let base = self.inferences.len() as u32;
        let ids: Vec<InfId> = (0..premises.len() as u32)
            .map(|i| InfId::new(base + i))
            .collect();
        for premise in premises {
            self.inferences.push(Inference {
                sequent: premise,
                rule: Rule::Open.into(),
                principal: None,
                premises: vec![],
                times: 1,
                cut: None,
                node: None,
            });
        }
        let inference = &mut self.inferences[goal.index()];
        inference.rule = rule;
        inference.principal = principal;
        inference.premises = ids.clone();
        self.history.push(goal.inf());
        ids.into_iter().map(GoalId::of).collect()
    }

    /// Computes the premises of a rule on a sequent, in the rule's order, or
    /// refuses: the rule must act on the formula's connective, be available
    /// in the mode, find the context it needs, take a valid split, and in
    /// intuitionistic mode leave one formula on the right of `⊢` in every
    /// premise and never weaken that formula.
    pub(super) fn expand(
        &self,
        reading: Option<&Reading>,
        sequent: &[OccId],
        position: usize,
        rule: Named,
        left: &[usize],
    ) -> Result<Vec<Multiset>, StepError> {
        use Rule::*;
        let f = &self.forest;
        let o = formula_at(sequent, position)?;
        let kind = f.kind(o);
        let classical = rule.rule;
        // The rule acts on the connective, and on the right side of ⊢.
        let acts = match classical {
            Ax => kind.is_literal(),
            // A session holds no binder, so this is false.
            Forall => kind == Kind::Forall,
            Exists => kind == Kind::Exists,
            Tensor => kind == Kind::Tensor,
            Par => kind == Kind::Par,
            One => kind == Kind::One,
            Bot => kind == Kind::Bot,
            With => kind == Kind::With,
            PlusLeft | PlusRight => kind == Kind::Plus,
            Top => kind == Kind::Top,
            Promotion => kind == Kind::Bang,
            Dereliction | Contraction | Weakening => kind == Kind::Quest,
            AffineWeakening => kind != Kind::Quest,
            Mix => true,
            // A cut is no step on a formula of the goal: it is made by
            // naming its formula (`cut`).
            Cut | Open => false,
        };
        let named = Named::applied(classical, reading, o);
        if !acts || rule != named {
            return Err(StepError::Rule { rule, position });
        }
        let allowed = match classical {
            AffineWeakening => self.mode.affine,
            Mix => self.mode.mix,
            // The ordered calculus is unit-free MLL.
            One | Bot | With | PlusLeft | PlusRight | Top | Promotion | Dereliction
            | Contraction | Weakening => !self.mode.is_ordered(),
            // A session in an ordered mode holds no binder.
            Ax | Tensor | Par | Cut | Open | Forall | Exists => true,
        };
        if !allowed {
            return Err(StepError::Mode {
                rule,
                mode: self.mode,
            });
        }
        let splits = matches!(classical, Tensor | Mix);
        if !splits && !left.is_empty() {
            return Err(StepError::NoSplit { rule });
        }
        // The context without the formula, and the part of it going left.
        let mut rest = Multiset::of(sequent.iter().copied());
        rest.remove(o);
        let mut going_left = Multiset::new();
        if splits {
            let mut used = vec![false; sequent.len()];
            used[position] = true;
            for &p in left {
                if p >= sequent.len() || used[p] {
                    return Err(StepError::Split { position: p });
                }
                used[p] = true;
                going_left.insert(sequent[p]);
            }
            // Going round from the formula, the right premise's context
            // first, then the left one's: an arc on either side.
            if self.mode.is_ordered() {
                let round = (position + 1..sequent.len()).chain(0..position);
                let mut left_seen = None;
                for p in round {
                    match (used[p], left_seen) {
                        (true, None) => left_seen = Some(p),
                        (false, Some(first)) => {
                            return Err(StepError::Exchange { position: first });
                        }
                        _ => {}
                    }
                }
            }
        }
        let child = |side: Branch| match side {
            Branch::Left => f.left(o).unwrap(),
            Branch::Right => f.right(o).unwrap(),
        };
        let with = |added: &[OccId]| {
            let mut premise = rest.clone();
            for &a in added {
                premise.insert(a);
            }
            premise
        };
        let premises = match classical {
            Forall | Exists => vec![with(&[child(Branch::Left)])],
            Ax => {
                let [x, y] = sequent else {
                    return Err(StepError::NotAlone { rule, position });
                };
                let other = if position == 0 { *y } else { *x };
                // A first-order session's literals are instances, whose
                // terms the step unifies: here only their signs are read.
                let dual = if self.fo.is_some() {
                    f.kind(other).is_literal() && f.kind(other) != kind
                } else {
                    f.dual_literals(o, other)
                };
                if !dual {
                    return Err(StepError::NoDual { position });
                }
                vec![]
            }
            One => {
                if sequent.len() != 1 {
                    return Err(StepError::NotAlone { rule, position });
                }
                vec![]
            }
            Top => vec![],
            Tensor => {
                let mut right = rest.difference(&going_left);
                going_left.insert(child(Branch::Left));
                right.insert(child(Branch::Right));
                vec![going_left, right]
            }
            Mix => {
                let right = rest.difference(&going_left);
                // No rule concludes the empty sequent.
                if right.is_empty() {
                    return Err(StepError::EmptyPremise);
                }
                going_left.insert(o);
                vec![going_left, right]
            }
            Par => vec![with(&[child(Branch::Left), child(Branch::Right)])],
            Bot => vec![with(&[])],
            With => vec![with(&[child(Branch::Left)]), with(&[child(Branch::Right)])],
            PlusLeft => vec![with(&[child(Branch::Left)])],
            PlusRight => vec![with(&[child(Branch::Right)])],
            Promotion => {
                if let Some(p) = sequent
                    .iter()
                    .enumerate()
                    .position(|(i, &x)| i != position && f.kind(x) != Kind::Quest)
                {
                    return Err(StepError::NotQuest { position: p });
                }
                vec![with(&[child(Branch::Left)])]
            }
            Dereliction => vec![with(&[child(Branch::Left)])],
            Contraction => vec![with(&[o, o])],
            Weakening | AffineWeakening => {
                if let Some(reading) = reading
                    && reading.position(o) == Side::Output
                {
                    return Err(StepError::Output { position });
                }
                vec![with(&[])]
            }
            Cut | Open => unreachable!("refused above: no step names the rule"),
        };
        if let Some(reading) = reading {
            for premise in &premises {
                let outputs = reading.outputs(premise.as_slice().iter().copied());
                if outputs != 1 {
                    return Err(StepError::Succedents { count: outputs });
                }
                if !self.mode.allows_empty_antecedents() && premise.as_slice().len() <= outputs {
                    return Err(StepError::EmptyAntecedent);
                }
            }
        }
        Ok(premises)
    }

    /// Returns whether the split of a step of `⊗` (or a Mix, when the
    /// formula at the step's position is not a `⊗`) passes the count
    /// prunes of the focused engine, a cheap test that says "this split
    /// cannot close" before a client tries it; a split that passes may
    /// still fail. The step's rule is not read.
    ///
    /// # Errors
    ///
    /// As [`rules`](Self::rules) does for the goal and the position.
    pub fn split_passes(&self, goal: GoalId, step: &Step) -> Result<bool, StepError> {
        let sequent = self.open(goal)?;
        let position = step.position;
        let left = step.left_positions(sequent.len())?;
        let left = &left[..];
        let o = formula_at(&sequent, position)?;
        let rule = if self.forest.kind(o) == Kind::Tensor {
            Rule::Tensor
        } else {
            Rule::Mix
        };
        let reading = self.reading();
        let rule = Named::applied(rule, reading.as_ref(), o);
        let premises = self.expand(reading.as_ref(), &sequent, position, rule, left)?;
        let [l, r] = premises.as_slice() else {
            unreachable!("a split has two premises");
        };
        let fragment = search::goal_fragment(&self.forest, &sequent);
        Ok(focus::counts::split_passes(
            &self.forest,
            fragment,
            self.mode,
            l.as_slice(),
            r.as_slice(),
        ))
    }

    /// Retracts the last step, a rule applied or a goal closed by the
    /// search, and returns the goal it reopened, or `None` when no step is
    /// left. The open witnesses the step bound are open again.
    pub fn undo(&mut self) -> Option<GoalId> {
        let goal = self.history.pop()?;
        // The step's inferences are the suffix of the arena.
        let first = self.subtree(goal).into_iter().min();
        if let Some(first) = first {
            self.inferences.truncate(first.index());
        }
        let inference = &mut self.inferences[goal.index()];
        let cut = inference.rule.rule == Rule::Cut;
        inference.rule = Rule::Open.into();
        inference.principal = None;
        inference.premises.clear();
        inference.cut = None;
        if cut {
            self.uncut();
        }
        // The step's number, under which its bindings were trailed: fewer
        // steps than inferences, which an `InfId` numbers.
        let step = self.history.len() as u32;
        if let Some(q) = self.fo.as_mut() {
            q.undo(step);
        }
        self.show();
        Some(GoalId::of(goal))
    }

    /// Returns the inferences above `id`, excluding it, in no particular
    /// order.
    pub(super) fn subtree(&self, id: InfId) -> Vec<InfId> {
        let mut ids = self.inferences[id.index()].premises.clone();
        let mut i = 0;
        while i < ids.len() {
            ids.extend(self.inferences[ids[i].index()].premises.iter().copied());
            i += 1;
        }
        ids
    }

    /// Runs the search on an open goal with the options and the stop
    /// condition, as [`prove_goal`](crate::search::prove_goal) does, and
    /// returns its outcome. When the goal is proved, the derivation of the
    /// proof found is grafted onto the goal as one step, which
    /// [`undo`](Self::undo) retracts whole; otherwise nothing changes. The
    /// outcome's proof, if any, is the proof of the goal alone.
    ///
    /// The search runs within `limits`, and the derivation grafted is
    /// within them too: a goal whose proof unfolds into a larger one, or
    /// whose unfolding `stop` ends, stays open, and [`Closed::grafted`]
    /// says why; its proof stays in the outcome. A proved goal is never
    /// lost.
    ///
    /// # Errors
    ///
    /// [`Error::Step`] for a goal that is not open, and the errors of the
    /// search ([`prove_goal`](crate::search::prove_goal)); a graft the limits
    /// or `stop` refuse is no error but [`Closed::grafted`].
    pub fn close(
        &mut self,
        goal: GoalId,
        options: &Options,
        view: &ViewOptions,
        limits: &Limits,
        mut stop: impl FnMut(Progress) -> bool,
    ) -> Result<Closed, Error> {
        if self.fo.is_some() {
            return self.close_framed(goal, options, view, limits, stop);
        }
        let members = self.goal(goal)?;
        let outcome = search::prove_goal(
            search::Goal::new(&self.forest, members)?,
            self.mode,
            options,
            limits,
            &mut stop,
        )?;
        let mut grafted = Ok(());
        if let Verdict::Proved(proof) = &outcome.verdict {
            grafted = match self.close_with(goal, proof, view, limits, stop) {
                Ok(()) => Ok(()),
                Err(Error::Refused(refusal)) => Err(refusal),
                Err(Error::Check(CheckError::Refused(refused))) => Err(refused.refusal),
                Err(error) => return Err(error),
            };
        }
        Ok(Closed { outcome, grafted })
    }

    /// Closes an open goal with a proof of it that a search found, as
    /// [`close`](Self::close) does with its own: the proof must be over the
    /// session's sequent ([`Error::ForeignProof`] otherwise), its root must
    /// conclude the goal, as the proof of [`prove_goal`] on the goal's
    /// occurrences ([`goal`](Self::goal)) does, and it must pass the
    /// checker in the session's mode; its derivation is grafted within
    /// `limits` as one step. A caller that runs the search itself,
    /// several of them side by side for one, closes the goal with this.
    ///
    /// [`prove_goal`]: crate::search::prove_goal
    ///
    /// # Errors
    ///
    /// [`Error::Step`] for a goal that is not open, [`Error::ForeignProof`]
    /// for a proof of another sequent, [`Error::GoalMismatch`] for one of
    /// another goal, [`Error::Check`] with [`CheckError::Invalid`] for one
    /// the checker rejects in the session's mode; and, leaving the goal
    /// open, [`Error::Check`] with [`CheckError::Refused`] when a bound or
    /// `stop` ends the check, and [`Error::Refused`] when they refuse the
    /// graft. [`Error::is_refusal`] tells the refusals from the rejection.
    pub fn close_with(
        &mut self,
        goal: GoalId,
        proof: &Proof,
        view: &ViewOptions,
        limits: &Limits,
        mut stop: impl FnMut(Progress) -> bool,
    ) -> Result<(), Error> {
        let sequent = self.open(goal)?;
        // The goal's ids name occurrences of the session's forest, which
        // is a function of its sequent.
        if proof.sequent() != self.sequent() {
            return Err(Error::ForeignProof);
        }
        let mut concluded: Vec<OccId> = proof.conclusion();
        concluded.sort_unstable();
        if concluded != sequent {
            return Err(Error::GoalMismatch);
        }
        let found = Derivation::of_goal(proof, &sequent, self.mode, view, limits, &mut stop)?;
        self.graft(goal.inf(), found);
        self.history.push(goal.inf());
        Ok(())
    }

    /// Closes an open goal with a proof of an equal sequent made elsewhere,
    /// another session's say: the proof is carried over to the goal
    /// ([`Proof::onto`]), its conclusions matched to the goal's formulas up
    /// to their order, and closes the goal as [`close_with`](Self::close_with)
    /// does, checked in the session's mode, as one step.
    ///
    /// # Errors
    ///
    /// [`Error::Step`] for a goal that is not open,
    /// [`Error::SequentMismatch`] for a proof whose conclusions are not the
    /// goal's formulas, and [`close_with`](Self::close_with)'s: above all
    /// [`Error::Check`] with [`CheckError::Invalid`] for a proof the
    /// checker rejects in the session's mode, one of another mode's rules.
    pub fn fill(
        &mut self,
        goal: GoalId,
        proof: &Proof,
        view: &ViewOptions,
        limits: &Limits,
        stop: impl FnMut(Progress) -> bool,
    ) -> Result<(), Error> {
        let members = self.goal(goal)?.to_vec();
        let carried = proof.onto(self.forest.clone(), &members)?;
        self.close_with(goal, &carried, view, limits, stop)
    }

    /// Runs the search on every open goal in turn, as [`close`](Self::close)
    /// does, and returns each goal with what closing it did; a goal the
    /// search does not prove, or whose closing fails, stays open, and the
    /// run goes on with the next. The limits bound each goal's search.
    pub fn close_all(
        &mut self,
        options: &Options,
        view: &ViewOptions,
        limits: &Limits,
        mut stop: impl FnMut(Progress) -> bool,
    ) -> Vec<(GoalId, Result<Closed, Error>)> {
        let goals: Vec<GoalId> = self.goals().collect();
        goals
            .into_iter()
            .enumerate()
            .map(|(i, goal)| {
                // Each goal's polls name it by its place among the goals.
                let item = u32::try_from(i).unwrap_or(u32::MAX);
                let stop = |progress: Progress| stop(Progress { item, ..progress });
                (goal, self.close(goal, options, view, limits, stop))
            })
            .collect()
    }

    /// Replaces the open goal by the root of a derivation given premises
    /// before conclusions, appending the rest in reverse, so that every
    /// premise keeps a larger index than its conclusion.
    pub(super) fn graft(&mut self, goal: InfId, mut found: Vec<Inference>) {
        let root = found.len() - 1;
        let base = self.inferences.len();
        let renumber = |id: InfId| {
            if id.index() == root {
                goal
            } else {
                InfId::new((base + root - 1 - id.index()) as u32)
            }
        };
        for inference in &mut found {
            for premise in &mut inference.premises {
                *premise = renumber(*premise);
            }
        }
        let root = found.pop().unwrap();
        self.inferences[goal.index()] = root;
        self.inferences.extend(found.into_iter().rev());
    }
}

/// Returns the formula at a position of a sequent, or the refusal.
pub(super) fn formula_at(sequent: &[OccId], position: usize) -> Result<OccId, StepError> {
    sequent.get(position).copied().ok_or(StepError::NoFormula {
        position,
        len: sequent.len(),
    })
}
