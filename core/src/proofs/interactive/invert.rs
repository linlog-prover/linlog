// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The invertible rules applied without being asked: a rule is invertible
//! when its premises are provable whenever its conclusion is, so applying
//! it never turns a provable goal into an unprovable one, and a session
//! may take such steps by itself, as the asynchronous phase of a focused
//! search does.

use super::expand::formula_at;
use super::{GoalId, Interactive, Step, StepError, Witness};
use crate::limits::{Limits, Phase, Refusal};
use crate::occurrences::{Member, OccId};
use crate::proofs::derivation::Inference;
use crate::proofs::multiset::Multiset;
use crate::proofs::{Named, Rule};
use crate::sequents::Kind;

/// How a session takes steps of its own: one plain value a front end
/// holds with its other settings.
///
/// # JSON
///
/// With the feature `serialize` `{"invertible": false}`; a missing field
/// takes its default and a misspelt one is refused.
///
/// Needs the cargo feature `interactive` (on by default).
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serialize",
    derive(serde::Serialize, serde::Deserialize),
    serde(default, deny_unknown_fields)
)]
pub struct Options {
    /// Whether a front end applies every invertible rule
    /// ([`Interactive::invert`]) to the goals each step opens and to the
    /// sequent a session starts from.
    pub invertible: bool,
}

impl Options {
    /// Off: every step a newcomer sees is one they took, until they ask
    /// for the invertible rules.
    pub const DEFAULT_INVERTIBLE: bool = false;

    /// Returns the options with another [`invertible`](Self::invertible).
    #[must_use]
    pub const fn with_invertible(self, invertible: bool) -> Self {
        Self { invertible }
    }
}

impl Default for Options {
    /// [`DEFAULT_INVERTIBLE`](Self::DEFAULT_INVERTIBLE).
    fn default() -> Self {
        Self {
            invertible: Self::DEFAULT_INVERTIBLE,
        }
    }
}

/// What [`Interactive::invert`] did.
///
/// Needs the cargo feature `interactive` (on by default).
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Inverted {
    /// How many steps it took, each an ordinary step of the session, which
    /// [`undo`](Interactive::undo) retracts one at a time, the last first.
    pub steps: usize,
    /// The goals left open where it went: each goal it was given that it
    /// took no step on, and each goal its steps opened that it took none
    /// on, from left to right.
    pub open: Vec<GoalId>,
    /// `Ok` when no invertible rule applies to any goal of
    /// [`open`](Self::open); otherwise the refusal of the memory bound
    /// that ended it, the steps taken standing.
    pub finished: Result<(), Refusal>,
}

impl Interactive {
    /// Applies to an open goal, and then to every goal a step of it opens,
    /// each rule invertible in the session's mode
    /// ([`is_invertible`](Self::is_invertible)) until none applies: a
    /// rule that closes a goal (`⊤`, the axiom, `1`) before one that opens
    /// goals, and otherwise the first formula's. Each application is an
    /// ordinary step. It ends, since every premise of these rules has fewer
    /// connectives and formulas than its conclusion; the inferences it
    /// adds are bounded by `limits.memory_bytes`, at which it stops, the
    /// steps taken standing ([`Inverted::finished`]).
    ///
    /// # Errors
    ///
    /// [`StepError::NoGoal`] for a goal that is not open.
    pub fn invert(&mut self, goal: GoalId, limits: &Limits) -> Result<Inverted, StepError> {
        self.goal(goal)?;
        Ok(self.invert_goals(vec![goal], limits))
    }

    /// Applies the invertible rules to every open goal, in turn, as
    /// [`invert`](Self::invert) does to one, all within one bound.
    pub fn invert_all(&mut self, limits: &Limits) -> Inverted {
        let goals = self.goals().collect();
        self.invert_goals(goals, limits)
    }

    /// Returns whether a step on a sequent of this session's forest (an
    /// open goal, an inference's conclusion) applies a rule invertible in
    /// the session's mode and is a valid step there:
    /// - in every mode `⅋`, `⊥`, `&` and `⊤`, under any name the
    ///   intuitionistic or the ordered reading gives them (`⊸R`, `⊗L`,
    ///   `&R`, `⊕L`, `1L`, `⊤R`, `0L`, `\R`, `/R`);
    /// - `!` when every other formula is a `?` formula (`!R` when every
    ///   hypothesis is a `!`);
    /// - `1` alone, the axiom on a literal and its dual alone, and `⊗`
    ///   alone, with no context to split.
    ///
    /// No `?` rule is among them: dereliction and weakening are not
    /// invertible, and contraction is but never ends.
    pub fn is_invertible(&self, sequent: &[Member], step: &Step) -> bool {
        let sequent: Vec<OccId> = sequent.iter().map(|&m| self.occurrence(m)).collect();
        let Ok(left) = step.left_positions(sequent.len()) else {
            return false;
        };
        let Ok(o) = formula_at(&sequent, step.position) else {
            return false;
        };
        let rule = self.invertible_rule(&sequent, self.non_quests(&sequent), step.position);
        if !left.is_empty() || rule != Some(step.rule.rule) {
            return false;
        }
        let reading = self.reading();
        let named = Named::applied(step.rule.rule, reading.as_ref(), o);
        (step.rule.side.is_none() || step.rule == named)
            && self
                .expand(reading.as_ref(), &sequent, step.position, named, &[])
                .is_ok()
    }

    /// Applies the invertible rules to the goals, depth first, within the
    /// memory bound.
    fn invert_goals(&mut self, goals: Vec<GoalId>, limits: &Limits) -> Inverted {
        let mut todo: Vec<GoalId> = goals.into_iter().rev().collect();
        let (mut steps, mut open) = (0, Vec::new());
        // The bytes of the inferences added, which a step adds at most
        // a sequent's length to per premise: it saturates, and the bound
        // stops it long before.
        let mut held = 0u64;
        while let Some(goal) = todo.pop() {
            let Some((position, rule, premises)) = self.invertible_step(goal) else {
                open.push(goal);
                continue;
            };
            let needed = premises.iter().fold(0u64, |sum, premise| {
                let members = premise.as_slice().len() * size_of::<Member>();
                sum.saturating_add((size_of::<Inference>() + members) as u64)
            });
            if let Some(limit) = limits.memory_bytes
                && held.saturating_add(needed) > limit
            {
                open.push(goal);
                open.extend(todo.into_iter().rev());
                return Inverted {
                    steps,
                    open,
                    finished: Err(Refusal::Memory {
                        phase: Phase::View,
                        limit_bytes: limit,
                        needed_bytes: Some(held.saturating_add(needed)),
                    }),
                };
            }
            held = held.saturating_add(needed);
            let premises = if self.fo.is_some() {
                match self.framed_premises(goal, position, rule.rule, &[], &Witness::Open) {
                    Ok(premises) => premises,
                    // Past the eigenvariables a `u32` numbers.
                    Err(_) => {
                        open.push(goal);
                        continue;
                    }
                }
            } else {
                premises.into_iter().map(Multiset::into_members).collect()
            };
            let opened = self.record(goal, position, rule, premises);
            steps += 1;
            todo.extend(opened.into_iter().rev());
        }
        Inverted {
            steps,
            open,
            finished: Ok(()),
        }
    }

    /// Returns the invertible step to take on an open goal, with its
    /// premises, or `None` when no invertible rule applies: one that
    /// closes the goal first, else the first position's.
    fn invertible_step(&self, goal: GoalId) -> Option<(usize, Named, Vec<Multiset>)> {
        let sequent = self.open(goal).ok()?;
        let reading = self.reading();
        let closes = |rule: Rule| matches!(rule, Rule::Top | Rule::Ax | Rule::One);
        let non_quests = self.non_quests(&sequent);
        for closing in [true, false] {
            for position in 0..sequent.len() {
                let Some(rule) = self.invertible_rule(&sequent, non_quests, position) else {
                    continue;
                };
                if closes(rule) != closing {
                    continue;
                }
                let named = Named::applied(rule, reading.as_ref(), sequent[position]);
                // The reading may still refuse it: a premise without one
                // formula on the right of ⊢, or without an antecedent.
                // A first-order axiom closes equal instances only: binding an
                // open witness is a choice, no invertible step.
                if rule == Rule::Ax
                    && let Some(q) = &self.fo
                    && let Ok(&[x, y]) = self.goal(goal)
                    && !q.same_atom(&self.forest, x, y)
                {
                    continue;
                }
                if let Ok(premises) = self.expand(reading.as_ref(), &sequent, position, named, &[])
                {
                    return Some((position, named, premises));
                }
            }
        }
        None
    }

    /// Returns how many formulas of a sequent are no `?` formula.
    fn non_quests(&self, sequent: &[OccId]) -> usize {
        (sequent.iter())
            .filter(|&&o| self.forest.kind(o) != Kind::Quest)
            .count()
    }

    /// Returns the invertible rule that acts on the formula at a position
    /// of a sequent, `non_quests` of whose formulas are no `?` formula,
    /// under its condition, if any; whether the mode and the reading allow
    /// the step is [`expand`](Self::expand)'s to say.
    fn invertible_rule(
        &self,
        sequent: &[OccId],
        non_quests: usize,
        position: usize,
    ) -> Option<Rule> {
        let f = &self.forest;
        let o = sequent[position];
        let alone = sequent.len() == 1;
        match f.kind(o) {
            Kind::Par => Some(Rule::Par),
            Kind::Bot => Some(Rule::Bot),
            Kind::With => Some(Rule::With),
            Kind::Top => Some(Rule::Top),
            // The `!` itself is the one formula that is no `?`.
            Kind::Bang => (non_quests == 1).then_some(Rule::Promotion),
            Kind::One => alone.then_some(Rule::One),
            Kind::Tensor => alone.then_some(Rule::Tensor),
            Kind::Atom | Kind::DualAtom => match sequent {
                &[x, y] if f.dual_literals(x, y) => Some(Rule::Ax),
                _ => None,
            },
            Kind::Forall => Some(Rule::Forall),
            Kind::Plus | Kind::Zero | Kind::Quest | Kind::Exists => None,
        }
    }
}

#[cfg(all(test, feature = "parse"))]
mod tests {
    use super::*;
    use crate::fragment::Mode;
    use crate::search::generate::{self, IllRules, Rng, Rules};
    use crate::search::{self, Verdict};
    use crate::{Sequent, ViewOptions};

    /// Starts a session of `input`, read in the mode's order.
    fn start(input: &str, mode: Mode) -> Interactive {
        let sequent = Sequent::parse_ordered(input, mode.order(), &Limits::default())
            .unwrap_or_else(|e| panic!("{input:?}: {e}"));
        Interactive::new(&sequent, mode).unwrap()
    }

    /// The rules of the steps a session took, in the arena's order.
    fn taken(state: &Interactive) -> Vec<String> {
        (state.inferences.iter())
            .filter(|i| i.rule.rule != Rule::Open)
            .map(|i| i.rule.to_string())
            .collect()
    }

    /// Each invertible rule applies under its condition and only then, a
    /// rule that closes the goal goes first, each application is a step
    /// that undo retracts, and the reading names the rules.
    #[test]
    fn the_invertible_rules_under_their_conditions() {
        let classical = Mode::CLASSICAL;
        for (input, mode, rules) in [
            ("|- a par ~a", classical, &["⅋", "ax"][..]),
            ("|- bot, 1", classical, &["⊥", "1"]),
            ("|- a & b, top", classical, &["⊤"]),
            ("|- (a par ~a) & 1", classical, &["&", "⅋", "1", "ax"]),
            ("|- !(a par ~a), ?b", classical, &["!", "⅋"]),
            ("|- 1 * 1", classical, &["⊗", "1", "1"]),
            ("|- 1, a", classical, &[]),
            ("|- !a, b", classical, &[]),
            ("|- a * b, c", classical, &[]),
            ("|- a, ~a, b", classical.with_affine(), &[]),
            ("|- a, ?b", classical, &[]),
            ("|- a + b, ~a", classical.with_mix(), &[]),
            ("a * b |- b * a", Mode::INTUITIONISTIC, &["⊗L"]),
            ("!a |- !(a & a)", Mode::INTUITIONISTIC, &["!R", "&R"]),
            ("|- a -o (b -o a * b)", Mode::INTUITIONISTIC, &["⊸R", "⊸R"]),
            ("|- 1 * 1", Mode::INTUITIONISTIC, &["⊗R", "1R", "1R"]),
            ("b |- (a \\ a) \\ b", Mode::LAMBEK, &["\\R"]),
            (
                "|- (a \\ a) * (b \\ b)",
                Mode::LAMBEK.with_empty_antecedents(),
                &["⊗R", "\\R", "\\R", "ax", "ax"],
            ),
            (
                "|- (a \\ a) * (b \\ b)",
                Mode::CYCLIC,
                &["⊗", "⅋", "⅋", "ax", "ax"],
            ),
            ("a * b |- a * b", Mode::CYCLIC, &["⅋"]),
        ] {
            let mut state = start(input, mode);
            let before = state.derivation().unwrap().to_string();
            let inverted = state.invert_all(&Limits::default());
            assert_eq!(taken(&state), rules, "{input} in {mode}");
            assert_eq!(
                (inverted.steps, state.steps(), inverted.finished.clone()),
                (rules.len(), rules.len(), Ok(())),
                "{input} in {mode}"
            );
            let mut open = inverted.open.clone();
            open.sort_unstable();
            assert_eq!(open, state.goals().collect::<Vec<_>>());
            assert_eq!(state.invert_all(&Limits::default()).steps, 0);
            if state.is_complete() {
                state.proof(&Limits::default(), |_| false).unwrap();
            }
            for _ in 0..rules.len() {
                state.undo().unwrap();
            }
            assert_eq!(state.derivation().unwrap().to_string(), before);
        }
        let mut state = start("|- a par b", classical);
        state
            .apply(GoalId::new(0), &Step::new(0, Rule::Par))
            .unwrap();
        assert_eq!(
            state.invert(GoalId::new(0), &Limits::default()),
            Err(StepError::NoGoal {
                goal: GoalId::new(0)
            })
        );
    }

    /// Returns how many connectives, units and literals a sequent's
    /// formulas have.
    fn weight(forest: &crate::Forest, sequent: &[Member]) -> usize {
        let mut todo: Vec<OccId> = sequent.iter().map(|m| m.occ()).collect();
        let mut weight = 0;
        while let Some(o) = todo.pop() {
            weight += 1;
            todo.extend(forest.left(o));
            todo.extend(forest.right(o));
        }
        weight
    }

    /// On generated provable sequents of every mode, the search proves
    /// every goal the invertible rules leave, the steps end with every
    /// premise lighter than its conclusion and no invertible rule left to
    /// apply, and the search closing the rest makes a proof the checker
    /// accepts.
    #[test]
    fn inverting_keeps_every_proof_and_ends() {
        let mut cases: Vec<(String, Mode)> = Vec::new();
        for (i, rules) in Rules::ALL.into_iter().enumerate() {
            let mut rng = Rng::new(3800 + i as u64);
            let base = if rules.mix {
                Mode::CLASSICAL.with_mix()
            } else {
                Mode::CLASSICAL
            };
            for _ in 0..8 {
                let budget = 2 + rng.below(10);
                let text =
                    generate::sequent(&generate::provable(&mut rng, rules, 3, budget).formulas);
                cases.push((text.clone(), base));
                cases.push((text, base.with_affine()));
            }
        }
        for (i, rules) in IllRules::ALL.into_iter().enumerate() {
            let mut rng = Rng::new(3900 + i as u64);
            for _ in 0..8 {
                let budget = 2 + rng.below(10);
                let ill = generate::ill(&mut rng, rules, 3, budget);
                let text = generate::two_sided(&ill.hypotheses, &ill.goal);
                cases.push((text.clone(), Mode::INTUITIONISTIC));
                cases.push((text, Mode::INTUITIONISTIC.with_affine()));
            }
        }
        let mut rng = Rng::new(3999);
        for k in 0..60 {
            let star = k % 2 == 1;
            let budget = 2 + rng.below(8);
            let (hypotheses, goal) = generate::lambek(&mut rng, 3, budget, star);
            let text = generate::lambek_text(&hypotheses, &goal);
            let lambek = if star {
                Mode::LAMBEK.with_empty_antecedents()
            } else {
                Mode::LAMBEK
            };
            cases.push((text.clone(), lambek));
            cases.push((text, Mode::CYCLIC));
        }
        let options = search::Options::default();
        let limits = Limits::default();
        let (mut proved, mut steps) = (0, 0);
        let mut used = std::collections::BTreeMap::<String, usize>::new();
        for (text, mode) in cases {
            let sequent = Sequent::parse_ordered(&text, mode.order(), &limits).unwrap();
            let Ok(outcome) = search::prove(&sequent, mode, &options) else {
                continue;
            };
            if !matches!(outcome.verdict, Verdict::Proved(_)) {
                continue;
            }
            proved += 1;
            let mut state = Interactive::new(&sequent, mode).unwrap();
            let inverted = state.invert_all(&limits);
            assert_eq!(inverted.finished, Ok(()), "{text} in {mode}");
            assert_eq!(state.invert_all(&limits).steps, 0, "{text} in {mode}");
            steps += inverted.steps;
            for inference in state
                .inferences
                .iter()
                .filter(|i| i.rule.rule != Rule::Open)
            {
                *used
                    .entry(format!("{mode}: {}", inference.rule))
                    .or_default() += 1;
                let conclusion = weight(&state.forest, &inference.sequent);
                for premise in &inference.premises {
                    let premise = &state.inferences[premise.index()].sequent;
                    assert!(
                        weight(&state.forest, premise) < conclusion,
                        "{text} in {mode}"
                    );
                }
            }
            for goal in inverted.open {
                let members = state.goal(goal).unwrap();
                let target = search::Goal::new(state.forest(), members).unwrap();
                let verdict = search::prove_goal(target, mode, &options, &limits, |_| false)
                    .unwrap()
                    .verdict;
                assert!(
                    matches!(verdict, Verdict::Proved(_)),
                    "{text} in {mode}: goal {} is {verdict:?}",
                    goal.get()
                );
            }
            for (goal, closed) in
                state.close_all(&options, &ViewOptions::default(), &limits, |_| false)
            {
                let closed = closed.unwrap();
                assert!(
                    closed.grafted.is_ok(),
                    "{text} in {mode}: goal {}",
                    goal.get()
                );
            }
            assert!(state.is_complete(), "{text} in {mode}");
            state.proof(&limits, |_| false).unwrap();
        }
        assert!(
            proved >= 500 && steps >= 600,
            "{proved} proved, {steps} steps"
        );
        // Every rule fired in every classical mode, and the multiplicatives
        // in every other.
        for (mode, rules) in [
            ("classical", &["⅋", "⊥", "&", "⊤", "!", "1", "⊗", "ax"][..]),
            (
                "classical with Mix",
                &["⅋", "⊥", "&", "⊤", "!", "1", "⊗", "ax"],
            ),
            (
                "classical affine",
                &["⅋", "⊥", "&", "⊤", "!", "1", "⊗", "ax"],
            ),
            (
                "intuitionistic",
                &["⊸R", "⊗L", "&R", "⊕L", "1L", "⊤R", "0L", "!R", "1R", "ax"],
            ),
            (
                "intuitionistic affine",
                &["⊸R", "⊗L", "&R", "⊕L", "1L", "⊤R", "0L", "!R", "1R", "ax"],
            ),
            ("Lambek", &["\\R", "/R", "⊗L", "ax"]),
            ("Lambek with empty antecedents", &["\\R", "/R", "⊗L", "ax"]),
            ("cyclic", &["⅋", "ax"]),
        ] {
            for rule in rules {
                assert!(
                    used.contains_key(&format!("{mode}: {rule}")),
                    "{mode}: {rule}"
                );
            }
        }
    }

    /// A run past the memory bound stops there, its steps ordinary ones
    /// that undo retracts, and the same run without the bound ends.
    #[test]
    fn inverting_stops_at_the_memory_bound() {
        let input = "|- a & a, a & a, a & a, a & a, a & a, a & a";
        let mut state = start(input, Mode::CLASSICAL);
        let bounded = Limits::default().with_memory_bytes(Some(4096));
        let inverted = state.invert_all(&bounded);
        assert!(
            matches!(
                inverted.finished,
                Err(Refusal::Memory {
                    limit_bytes: 4096,
                    needed_bytes: Some(needed),
                    ..
                }) if needed > 4096
            ),
            "{inverted:?}"
        );
        assert!(0 < inverted.steps && inverted.steps < 63, "{inverted:?}");
        let mut open = inverted.open.clone();
        open.sort_unstable();
        assert_eq!(open, state.goals().collect::<Vec<_>>());
        while state.undo().is_some() {}
        let inverted = state.invert_all(&Limits::default().with_memory_bytes(None));
        assert_eq!((inverted.steps, inverted.open.len()), (63, 64));
    }
}
