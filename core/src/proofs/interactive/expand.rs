// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The steps of a session: the rules a goal admits, a rule applied and
//! expanded into its premises, undo, and a goal closed by the search.

use super::{Applicable, Closed, GoalId, Interactive, Needs, Step, StepError};
use crate::Error;
use crate::limits::{Limits, Progress};
use crate::occurrences::{OccId, Reading, Side};
use crate::proofs::derivation::{Derivation, InfId, Inference, ViewOptions};
use crate::proofs::multiset::Multiset;
use crate::proofs::{Branch, CheckError, Named, Proof, Rule};
use crate::search::{self, Options, Verdict, focus};
use crate::sequents::Kind;

impl Interactive {
    /// Returns the sequent of an open goal, or the refusal.
    pub(super) fn open(&self, id: GoalId) -> Result<Vec<OccId>, StepError> {
        let goal = self.goal(id)?;
        Ok(goal.iter().map(|m| m.occ()).collect())
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
            Kind::Forall | Kind::Exists => {
                unreachable!("interactive proving refuses a sequent with a binder")
            }
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
                    Ax | Par | One | Bot | With | PlusLeft | PlusRight | Top | Promotion
                    | Dereliction | Contraction | Weakening | AffineWeakening | Cut | Forall
                    | Exists | Open => Needs::Nothing,
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
    /// # Errors
    ///
    /// A [`StepError`] that names what the rule needed: an open goal, a
    /// formula at the position, a rule that acts on it in the mode, a split
    /// that fits, the dual of an axiom, an empty context for `1` and `!`.
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
        Ok(self.record(goal, position, rule, premises))
    }

    /// Closes an open goal by a rule at a position with the premises
    /// [`expand`](Self::expand) computed, opens the premises as goals, and
    /// returns them.
    pub(super) fn record(
        &mut self,
        goal: GoalId,
        position: usize,
        rule: Named,
        premises: Vec<Multiset>,
    ) -> Vec<GoalId> {
        let principal = rule.rule.has_principal().then_some(position);
        let base = self.inferences.len() as u32;
        let ids: Vec<InfId> = (0..premises.len() as u32)
            .map(|i| InfId::new(base + i))
            .collect();
        for premise in premises {
            self.inferences.push(Inference {
                sequent: premise.into_members(),
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
            Ax | Tensor | Par | Cut | Open => true,
            // A session holds no binder: its rule did not act above.
            Forall | Exists => false,
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
            // A session holds no binder: its rule did not act above.
            Forall | Exists => unreachable!("no binder in a session"),
            Ax => {
                let [x, y] = sequent else {
                    return Err(StepError::NotAlone { rule, position });
                };
                let other = if position == 0 { *y } else { *x };
                if !f.dual_literals(o, other) {
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
    /// left.
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
