// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The calls of the bindings over JSON text: each reads its documents and
//! the page's settings, does one thing with the library, and writes what
//! the page shows. The page holds every document (the session, the
//! settings); nothing is kept here between calls, so a worker stopped
//! mid-call loses nothing.

use crate::failure::Failure;
use crate::tab;
use linlog::export::{latex, pdf, png, rocq, svg, typst};
use linlog::proofs::Compact;
use linlog::proofs::interactive::{Applicable, Needs, Split};
use linlog::{
    Criterion, Disproof, Error, GoalId, Interactive, Member, Mode, Named, Outcome, Progress,
    ProofStructure, Rule, Sequent, Settings, Side, Step, Verdict, nets, wire,
};
use serde::{Deserialize, Serialize};

/// The stop of every call: the page cancels by stopping the worker, so
/// only the bounds of the settings end a call early.
const fn never(_: Progress) -> bool {
    false
}

/// What the module is: the library's version and the highest wire level
/// it reads and writes.
pub fn about() -> String {
    serde_json::json!({
        "linlog": env!("CARGO_PKG_VERSION"),
        "level": wire::LEVEL,
        "stack_bytes": tab::STACK_BYTES,
    })
    .to_string()
}

/// The tab's settings, as the page shows and changes them.
pub fn defaults() -> String {
    serde_json::to_string(&tab::defaults()).unwrap_or_default()
}

/// What [`read`] found in a document.
#[derive(Serialize)]
struct Read<'a> {
    /// `sequent` for a sequent, as text or as `seq json` writes it, and
    /// `session` for a session as `save` writes it.
    kind: &'static str,
    /// The sequent as JSON.
    sequent: &'a Sequent,
    /// The sequent in the parser's Unicode syntax.
    text: String,
    /// The session's mode, for a session.
    #[serde(skip_serializing_if = "Option::is_none")]
    mode: Option<Mode>,
    /// The session, for a session.
    #[serde(skip_serializing_if = "Option::is_none")]
    session: Option<&'a Interactive>,
    /// The smallest fragment the sequent is in, by the names `seq
    /// fragment` prints, in each of the two modes.
    fragment: Fragments,
}

/// A sequent's fragment as named in classical and intuitionistic mode.
#[derive(Serialize)]
struct Fragments {
    /// In classical mode.
    classical: &'static str,
    /// In intuitionistic mode.
    intuitionistic: &'static str,
}

impl Fragments {
    /// Returns the names of a sequent's fragment.
    fn of(sequent: &Sequent) -> Self {
        let fragment = sequent.fragment();
        Self {
            classical: fragment.name_in(Mode::CLASSICAL),
            intuitionistic: fragment.name_in(Mode::INTUITIONISTIC),
        }
    }
}

/// The keys that tell a session document from a sequent's, read without
/// keeping anything else.
#[derive(Deserialize)]
struct Probe {
    /// A session's inferences.
    inferences: Option<serde::de::IgnoredAny>,
}

/// Reads a document the page was given: a sequent typed or as `seq json`
/// writes it, or a session as `save` writes it, and says which.
///
/// # Errors
///
/// The parser's error for text that is no sequent (its position is the
/// caret's), and the readers' for a document they refuse: an unknown
/// level, a malformed session, a bound passed.
pub fn read(input: &str, settings: &str) -> Result<String, Failure> {
    let settings = tab::read(settings)?;
    if !is_json(input) {
        let sequent = sequent_of(input, &settings)?;
        return written(&Read {
            kind: "sequent",
            text: sequent.to_string(),
            fragment: Fragments::of(&sequent),
            sequent: &sequent,
            mode: None,
            session: None,
        });
    }
    let probe: Probe = serde_json::from_str(input).map_err(Failure::json)?;
    if probe.inferences.is_some() {
        let state = session_of(input, &settings)?;
        written(&Read {
            kind: "session",
            text: state.sequent().to_string(),
            fragment: Fragments::of(state.sequent()),
            sequent: state.sequent(),
            mode: Some(state.mode()),
            session: Some(&state),
        })
    } else {
        let sequent = sequent_of(input, &settings)?;
        written(&Read {
            kind: "sequent",
            text: sequent.to_string(),
            fragment: Fragments::of(&sequent),
            sequent: &sequent,
            mode: None,
            session: None,
        })
    }
}

/// A formula of a goal as the page shows it.
#[derive(Serialize)]
struct Formula {
    /// The formula in the parser's Unicode syntax.
    text: String,
    /// Its occurrence in the sequent's forest.
    occurrence: u32,
    /// In intuitionistic mode, `input` for a hypothesis and `output` for
    /// the goal, `null` in classical mode.
    side: Option<&'static str>,
    /// In intuitionistic mode, the formula as the reading writes it in its
    /// position (a hypothesis `A ⊸ B` as `A ⊸ B`, where `text` has its
    /// one-sided `A ⊗ ~B`); absent in classical mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    reading: Option<String>,
}

/// An open goal.
#[derive(Serialize)]
struct Goal {
    /// The goal's id, which every call on it takes.
    goal: GoalId,
    /// Its formulas in the order of their positions.
    formulas: Vec<Formula>,
}

/// What the page shows of a session.
#[derive(Serialize)]
struct View {
    /// The session's mode.
    mode: Mode,
    /// The smallest fragment of the sequent, named in the mode.
    fragment: &'static str,
    /// Whether no goal is open.
    complete: bool,
    /// The steps taken.
    steps: usize,
    /// The open goals.
    goals: Vec<Goal>,
    /// For each inference `n` of the drawing (`i<n>`), the goal it is in
    /// the session.
    inferences: Vec<GoalId>,
    /// `null` when the sequent has proof nets, and otherwise the refusal
    /// that says why not, which the page shows at the greyed net view.
    net: Option<serde_json::Value>,
}

impl View {
    /// Returns the view of a session.
    fn of(state: &Interactive) -> Self {
        let reading = state.reading();
        let fragment = state.sequent().fragment();
        let goals = state
            .goals()
            .map(|goal| Goal {
                goal,
                formulas: state
                    .goal(goal)
                    .unwrap_or_default()
                    .iter()
                    .map(|&member| {
                        let occurrence = state.occurrence(member);
                        Formula {
                            text: state.formula(member).to_string(),
                            occurrence: occurrence.get(),
                            side: reading.as_ref().map(|reading| {
                                match reading.position(occurrence) {
                                    Side::Input => "input",
                                    Side::Output => "output",
                                }
                            }),
                            reading: reading
                                .as_ref()
                                .map(|reading| reading.formula(occurrence).to_string()),
                        }
                    })
                    .collect(),
            })
            .collect();
        Self {
            mode: state.mode(),
            fragment: fragment.name_in(state.mode()),
            complete: state.is_complete(),
            steps: state.steps(),
            goals,
            inferences: state.derivation_ids(),
            net: nets::exist(fragment, state.mode())
                .err()
                .and_then(|refusal| serde_json::to_value(Error::from(refusal)).ok()),
        }
    }
}

/// A session after a call that changed it, with its view and what the
/// call says beyond it.
#[derive(Serialize)]
struct Changed<'a, T: Serialize> {
    /// The session, which the page keeps for the next call.
    session: &'a Interactive,
    /// What the page shows of it.
    view: View,
    /// What the call says beyond the session.
    #[serde(flatten)]
    said: T,
}

/// Returns a session's document with its view and what a call said.
fn changed(state: &Interactive, said: impl Serialize) -> Result<String, Failure> {
    written(&Changed {
        session: state,
        view: View::of(state),
        said,
    })
}

/// Starts a proof of a sequent, typed or as JSON, in a mode by its name
/// (`classical`, `affine`, `mix`, `affine-mix`, `intuitionistic`,
/// `intuitionistic-affine`), and returns the session with its view.
///
/// # Errors
///
/// As [`read`] for the sequent, an unknown mode, and a sequent the mode
/// does not read (an intuitionistic one needs a reading).
pub fn start(sequent: &str, mode: &str, settings: &str) -> Result<String, Failure> {
    let settings = tab::read(settings)?;
    let sequent = sequent_of(sequent, &settings)?;
    let mode: Mode = mode.parse().map_err(Failure::from)?;
    let state = Interactive::within(&sequent, mode, &settings.limits).map_err(Failure::from)?;
    changed(&state, serde_json::Map::new())
}

/// Returns a session's view.
///
/// # Errors
///
/// As the session's reader.
pub fn view(session: &str, settings: &str) -> Result<String, Failure> {
    let settings = tab::read(settings)?;
    let state = session_of(session, &settings)?;
    written(&View::of(&state))
}

/// A rule a formula takes, as the page lists it.
#[derive(Serialize)]
struct Offered {
    /// The rule's name.
    rule: Named,
    /// Whether a step of it needs a split: `⊗` and Mix.
    split: bool,
}

impl From<Applicable> for Offered {
    /// The rule and whether it needs a split.
    fn from(applicable: Applicable) -> Self {
        Self {
            rule: applicable.rule,
            split: matches!(applicable.needs, Needs::Split),
        }
    }
}

/// Returns the rules that act on the formula at a position of a goal, by
/// its connective and the mode: `[{"rule", "split"}]`. A rule listed may
/// still be refused by the context, with the reason.
///
/// # Errors
///
/// The step error for a goal that is not open or a position outside it.
pub fn rules(session: &str, goal: u32, position: u32, settings: &str) -> Result<String, Failure> {
    let settings = tab::read(settings)?;
    let state = session_of(session, &settings)?;
    let rules = state
        .rules(GoalId::new(goal), position_of(position))
        .map_err(|e| Failure::described(&Error::from(e), &state))?;
    written(&rules.into_iter().map(Offered::from).collect::<Vec<_>>())
}

/// A step as the page sends it: `{"position", "rule", "left"}`, `left` the
/// positions that go to the left premise of a `⊗` or Mix and left out for
/// every other rule.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StepForm {
    /// The position of the formula the rule acts on.
    pub position: usize,
    /// The rule's name.
    pub rule: String,
    /// The positions that go to the left premise.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub left: Option<Vec<usize>>,
}

impl StepForm {
    /// Returns the step it names.
    fn step(&self) -> Result<Step, Failure> {
        let rule: Named = self.rule.parse().map_err(Failure::from)?;
        let step = Step::new(self.position, rule);
        Ok(match &self.left {
            Some(left) => step.left(left),
            None => step,
        })
    }

    /// Returns the form of a step.
    fn of(step: &Step) -> Self {
        Self {
            position: step.position,
            rule: step.rule.to_string(),
            left: match &step.split {
                Split::Left { positions, .. } => Some(positions.clone()),
                _ => None,
            },
        }
    }
}

/// Reads a step's JSON.
fn step_of(step: &str) -> Result<Step, Failure> {
    serde_json::from_str::<StepForm>(step)
        .map_err(Failure::json)?
        .step()
}

/// Returns whether a split can pass the counts: `false` means the step
/// cannot close, `true` that it may.
///
/// # Errors
///
/// As [`rules`], and a step that is no step's JSON.
pub fn split_passes(session: &str, goal: u32, step: &str, settings: &str) -> Result<bool, Failure> {
    let settings = tab::read(settings)?;
    let state = session_of(session, &settings)?;
    state
        .split_passes(GoalId::new(goal), &step_of(step)?)
        .map_err(|e| Failure::described(&Error::from(e), &state))
}

/// What a step opened.
#[derive(Serialize)]
struct Opened {
    /// The goals the step opened, in the order of the rule's premises.
    opened: Vec<GoalId>,
}

/// Applies a step to a goal and returns the session with the goals it
/// opened.
///
/// # Errors
///
/// The step's refusal with its reason (`kind`: `no_dual`, `not_alone`,
/// `not_quest`, …), the session unchanged.
pub fn apply(session: &str, goal: u32, step: &str, settings: &str) -> Result<String, Failure> {
    let settings = tab::read(settings)?;
    let mut state = session_of(session, &settings)?;
    let opened = state
        .apply(GoalId::new(goal), &step_of(step)?)
        .map_err(|e| Failure::described(&Error::from(e), &state))?;
    changed(&state, Opened { opened })
}

/// Closes an open goal with the proof of another finished session, a
/// tab's of an equal sequent: the source's proof, checked in its own mode,
/// is carried over to the goal, its formulas matched up to their order
/// (`Proof::onto`), and grafted as one step, checked again in this
/// session's mode. Returns the session with its view and the goal
/// filled.
///
/// # Errors
///
/// A goal that is not open; `open_goals` for a source not finished;
/// `sequent_mismatch` for a source whose sequent is not the goal's up to
/// order, its `details` the formulas only the source has (`proof`) and
/// only the goal has (`goal`); and `invalid_proof` for a proof this
/// session's mode does not allow (a rule of another mode), naming the
/// rule; the session unchanged in each.
pub fn fill(session: &str, goal: u32, source: &str, settings: &str) -> Result<String, Failure> {
    let settings = tab::read(settings)?;
    let mut state = session_of(session, &settings)?;
    let from = session_of(source, &settings)?;
    let proof = from
        .proof(&settings.limits, never)
        .map_err(|e| Failure::described(&e, &from))?;
    state
        .fill(
            GoalId::new(goal),
            &proof,
            &settings.view,
            &settings.limits,
            never,
        )
        .map_err(|e| Failure::described(&e, &state))?;
    changed(
        &state,
        Filled {
            filled: GoalId::new(goal),
        },
    )
}

/// What a fill closed.
#[derive(Serialize)]
struct Filled {
    /// The goal the source's proof closed.
    filled: GoalId,
}

/// What an undo reopened.
#[derive(Serialize)]
struct Reopened {
    /// The goal the undo reopened, `null` when no step was left.
    reopened: Option<GoalId>,
}

/// Retracts the session's last step.
///
/// # Errors
///
/// As the session's reader.
pub fn undo(session: &str, settings: &str) -> Result<String, Failure> {
    let settings = tab::read(settings)?;
    let mut state = session_of(session, &settings)?;
    let reopened = state.undo();
    changed(&state, Reopened { reopened })
}

/// What closing a goal did.
#[derive(Serialize)]
struct Closing {
    /// The search's outcome, a proof in it the goal's alone.
    outcome: Outcome,
    /// `null` when the proof was grafted or there was none, otherwise the
    /// refusal of the graft (its size, the stop), the goal then still
    /// open.
    refused: Option<serde_json::Value>,
    /// For an unprovable goal, whether its refutation passed
    /// `Disproof::check`; `null` otherwise.
    refutation_checked: Option<bool>,
}

/// Runs the search on a goal within the settings' bounds, and grafts its
/// proof.
///
/// # Errors
///
/// A goal that is not open, and the search's errors.
pub fn close(session: &str, goal: u32, settings: &str) -> Result<String, Failure> {
    let settings = tab::read(settings)?;
    let mut state = session_of(session, &settings)?;
    let closed = state
        .close(
            GoalId::new(goal),
            &settings.search,
            &settings.view,
            &settings.limits,
            never,
        )
        .map_err(|e| Failure::described(&e, &state))?;
    let refused = closed.grafted.err().map(refusal);
    changed(
        &state,
        Closing {
            refutation_checked: refutation_checked(&closed.outcome),
            outcome: closed.outcome,
            refused,
        },
    )
}

/// What closing one of the goals did, or why it failed.
#[derive(Serialize)]
struct Each {
    /// The goal.
    goal: GoalId,
    /// The search's outcome.
    #[serde(skip_serializing_if = "Option::is_none")]
    outcome: Option<Outcome>,
    /// The graft's refusal, as in [`close`].
    #[serde(skip_serializing_if = "Option::is_none")]
    refused: Option<serde_json::Value>,
    /// The error of a goal whose search failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<serde_json::Value>,
    /// The work its search did (`statistics.work`), which the budget
    /// counts.
    work: u64,
}

/// What closing every goal did.
#[derive(Serialize)]
struct All {
    /// Each goal open before the call, in order.
    results: Vec<Each>,
}

/// Runs the search on every open goal in turn within one work budget,
/// `limits.work`, for all their searches together: each goal's search
/// has what the searches before it left of the budget as its
/// `limits.work`, so a goal the budget runs out on is unknown with
/// `work_limit`, as after [`close`]. What a search spends is its
/// `statistics.work`; its proof's check is bounded by the same remainder,
/// and the classical refuter keeps its own bound
/// (`search.classical_work`). Every proof found is grafted; a goal not
/// proved stays open.
///
/// # Errors
///
/// As the session's reader.
pub fn close_all(session: &str, settings: &str) -> Result<String, Failure> {
    let settings = tab::read(settings)?;
    let mut state = session_of(session, &settings)?;
    let goals: Vec<GoalId> = state.goals().collect();
    let mut left = settings.limits.work;
    let mut results = Vec::with_capacity(goals.len());
    for goal in goals {
        let limits = settings.limits.with_work(left);
        let closed = state.close(goal, &settings.search, &settings.view, &limits, never);
        let work = closed
            .as_ref()
            .map_or(0, |closed| closed.outcome.statistics.work);
        left = left.map(|left| left.saturating_sub(work));
        results.push(match closed {
            Ok(closed) => Each {
                goal,
                refused: closed.grafted.err().map(refusal),
                outcome: Some(closed.outcome),
                error: None,
                work,
            },
            Err(error) => Each {
                goal,
                outcome: None,
                refused: None,
                error: serde_json::to_value(error.describe(&state)).ok(),
                work,
            },
        });
    }
    changed(&state, All { results })
}

/// What a hint found.
#[derive(Serialize)]
struct Hint {
    /// The search's outcome on the goal.
    outcome: Outcome,
    /// The first steps of the proof found, each on the goal the one
    /// before it opened, the first on the goal asked for: the structural
    /// rules at its root, and the first rule that is not one, which ends
    /// the list. `null` when no proof was found or its derivation was
    /// refused.
    steps: Option<Vec<StepForm>>,
}

/// Runs the search on a goal, the session left as it was, and returns
/// its outcome with the first steps of the proof it found, which the page
/// reveals rung by rung and, at the last rung, applies in turn. A
/// contraction at the root comes with the rule that uses both copies, so
/// that following the hints proves the goal: a search on the contracted
/// goal alone may weaken the copy again.
///
/// # Errors
///
/// A goal that is not open, and the search's errors.
pub fn hint(session: &str, goal: u32, settings: &str) -> Result<String, Failure> {
    let settings = tab::read(settings)?;
    let state = session_of(session, &settings)?;
    let members = state
        .goal(GoalId::new(goal))
        .map_err(|e| Failure::described(&Error::from(e), &state))?;
    let target = linlog::Goal::new(state.forest(), members).map_err(Failure::from)?;
    let outcome = linlog::prove_goal(
        target,
        state.mode(),
        &settings.search,
        &settings.limits,
        never,
    )
    .map_err(|e| Failure::described(&e, &state))?;
    let steps = match &outcome.verdict {
        Verdict::Proved(proof) => first_steps(proof, &settings)?,
        _ => None,
    };
    written(&Hint { outcome, steps })
}

/// Returns the steps a proof of a goal begins with, as [`Hint::steps`]
/// says, or `None` when the bounds refuse its derivation.
fn first_steps(
    proof: &linlog::Proof,
    settings: &Settings,
) -> Result<Option<Vec<StepForm>>, Failure> {
    let view = settings.view.with_compact(Compact::Never);
    let derivation = match proof.derivation_within(&view, &settings.limits, never) {
        Ok(derivation) => derivation,
        Err(error) if error.is_refusal() => return Ok(None),
        Err(error) => return Err(Failure::from(error)),
    };
    let mut steps = Vec::new();
    let mut at = derivation.root();
    loop {
        let inference = derivation.inference(at);
        let members = inference.sequent();
        let rule = inference.rule();
        let premise = |i: usize| derivation.inference(inference.premises()[i]).sequent();
        let step = match (rule.rule, inference.principal()) {
            (Rule::Mix, _) => {
                let mut taken = vec![false; members.len()];
                let positions = positions_of(premise(0), members, &mut taken);
                let Some((&first, rest)) = positions.split_first() else {
                    return Ok(None);
                };
                Step::new(first, rule).left(rest)
            }
            (Rule::Tensor, Some(position)) => {
                let forest = derivation.forest();
                let child = forest.left(derivation.occurrence(members[position]));
                let mut left = premise(0).to_vec();
                if let Some(i) = left
                    .iter()
                    .position(|&m| Some(derivation.occurrence(m)) == child)
                {
                    left.remove(i);
                }
                let mut taken = vec![false; members.len()];
                taken[position] = true;
                Step::new(position, rule).left(&positions_of(&left, members, &mut taken))
            }
            (_, Some(position)) => Step::new(position, rule),
            (_, None) => Step::new(0, rule),
        };
        steps.push(StepForm::of(&step));
        match inference.premises() {
            [premise] if rule.is_structural() => at = *premise,
            _ => return Ok(Some(steps)),
        }
    }
}

/// Returns, for each member of `part`, a position of the goal `members`
/// that holds it and is not `taken`, and takes it.
fn positions_of(part: &[Member], members: &[Member], taken: &mut [bool]) -> Vec<usize> {
    part.iter()
        .filter_map(|member| {
            let at = (0..members.len()).find(|&j| !taken[j] && members[j] == *member)?;
            taken[at] = true;
            Some(at)
        })
        .collect()
}

/// Returns the checked proof of a finished session as JSON.
///
/// # Errors
///
/// `open_goals` while a goal is open, and a check the bounds refuse.
pub fn proof(session: &str, settings: &str) -> Result<String, Failure> {
    let settings = tab::read(settings)?;
    let state = session_of(session, &settings)?;
    let proof = state
        .proof(&settings.limits, never)
        .map_err(|e| Failure::described(&e, &state))?;
    written(&proof)
}

/// A search's outcome, with what the page shows beside a refutation.
#[derive(Serialize)]
struct Decided {
    /// The outcome, as `prove --json` writes it.
    outcome: Outcome,
    /// For an unprovable sequent, whether the refutation passed
    /// `Disproof::check`, its own checker; `null` otherwise.
    refutation_checked: Option<bool>,
}

/// Decides a sequent in a mode within the settings' bounds, and checks a
/// refutation it gives.
///
/// # Errors
///
/// As [`start`] for the sequent and the mode, and the search's.
pub fn prove(sequent: &str, mode: &str, settings: &str) -> Result<String, Failure> {
    let settings = tab::read(settings)?;
    let sequent = sequent_of(sequent, &settings)?;
    let mode: Mode = mode.parse().map_err(Failure::from)?;
    let outcome = linlog::prove_within(&sequent, mode, &settings.search, &settings.limits, never)
        .map_err(Failure::from)?;
    written(&Decided {
        refutation_checked: refutation_checked(&outcome),
        outcome,
    })
}

/// Returns a session's derivation, open goals as leaves, as an SVG
/// drawing in the settings' style (`styles.svg`), with an id on every
/// conclusion (`i<n>`) and, with `ids`, on every formula (`i<n>-<p>`).
///
/// # Errors
///
/// The derivation's refusal past `limits.derivation_bytes`, and a style
/// the drawing refuses.
pub fn draw(session: &str, settings: &str) -> Result<String, Failure> {
    export(session, "svg", settings)
}

/// Returns the proof net of a finished session as an SVG drawing: for a
/// sequent of unit-free MLL, and with `styles.svg.essential` for one with
/// an intuitionistic reading, its essential net; every subformula written,
/// or with `styles.svg.net` `"compact"` only the literals and the
/// conclusions.
///
/// # Errors
///
/// While a goal is open, for a sequent without proof nets (the refusal
/// [`view`] gives too), and past the drawing's bound.
pub fn draw_net(session: &str, settings: &str) -> Result<String, Failure> {
    let settings = tab::read(settings)?;
    let state = session_of(session, &settings)?;
    let proof = state
        .proof(&settings.limits, never)
        .map_err(|e| Failure::described(&e, &state))?;
    let criterion = Criterion::of(state.mode()).map_err(|e| Failure::from(Error::from(e)))?;
    let net = ProofStructure::from_proof(&proof, criterion, &settings.limits, never)
        .map_err(Failure::from)?;
    svg::net(&net, &settings.styles.svg, &settings.limits, never).map_err(Failure::from)
}

/// Returns a session's derivation in a format: `text` (the command's text
/// tree), `latex`, `typst` and `svg` of the session's tree, open goals
/// included, and `rocq`, the certificate of a finished session's checked
/// proof; each in the settings' style for it.
///
/// # Errors
///
/// An unknown format, a derivation past its bound, `open_goals` for a
/// certificate of an unfinished proof, and what the format refuses.
pub fn export(session: &str, format: &str, settings: &str) -> Result<String, Failure> {
    let settings = tab::read(settings)?;
    let state = session_of(session, &settings)?;
    let (limits, styles) = (&settings.limits, &settings.styles);
    let mut out = String::new();
    if format == "rocq" {
        let proof = state
            .proof(limits, never)
            .map_err(|e| Failure::described(&e, &state))?;
        let view = settings.view.with_compact(Compact::Never);
        let derivation = proof
            .derivation_within(&view, limits, never)
            .map_err(Failure::from)?;
        rocq::write(&derivation, &styles.rocq, &mut out, never).map_err(Failure::from)?;
        return Ok(out);
    }
    let derivation = state
        .derivation_within(&settings.view, limits, never)
        .map_err(Failure::from)?;
    match format {
        "text" => derivation.write_text(&styles.text, &mut out, never),
        "latex" => latex::write(&derivation, &styles.latex, &mut out, never),
        "typst" => typst::write(&derivation, &styles.typst, &mut out, never),
        "svg" => svg::write(&derivation, &styles.svg, &mut out, never),
        _ => {
            return Err(Failure::refused(
                "unknown_name",
                "malformed",
                "the formats are text, latex, typst, svg and rocq",
            ));
        }
    }
    .map_err(Failure::from)?;
    Ok(out)
}

/// Renders an SVG drawing of the bindings as `png` or `pdf`, with the
/// font the drawing names (`styles.svg.font`, by default Euler Math) as
/// `font`, the bytes of its file: no other font sets its text.
/// A PDF needs `styles.pdf.date`, which the page takes from its clock.
///
/// # Errors
///
/// An unknown format, a render past `limits.memory_bytes` or the pixel
/// bound, a PDF without a date, and a text that is no drawing of the
/// bindings.
pub fn render(svg: &str, format: &str, font: &[u8], settings: &str) -> Result<Vec<u8>, Failure> {
    let settings = tab::read(settings)?;
    let (limits, styles) = (&settings.limits, &settings.styles);
    match format {
        "png" => png::from_svg(svg, &[font], &styles.png, limits),
        "pdf" => pdf::from_svg(svg, &[font], &styles.pdf, limits),
        _ => {
            return Err(Failure::refused(
                "unknown_name",
                "malformed",
                "the renders are png and pdf",
            ));
        }
    }
    .map_err(Failure::from)
}

/// Returns the Rocq certificate of non-derivability of a refuted sequent
/// or goal: `document` is a disproof, or the outcome of [`prove`] or
/// [`close`] for an unprovable one, which reads as its disproof.
///
/// # Errors
///
/// A document that is no disproof, and a refutation the certificate does
/// not cover (an exhausted search) or the kernel `nanoyalla`.
pub fn certificate(document: &str, settings: &str) -> Result<String, Failure> {
    let settings = tab::read(settings)?;
    let mut json = serde_json::Deserializer::from_str(document);
    let disproof: Disproof = wire::upgrade(&mut json, &settings.limits).map_err(Failure::from)?;
    json.end().map_err(Failure::json)?;
    let mut out = String::new();
    rocq::write_disproof(&disproof, &settings.styles.rocq, &mut out).map_err(Failure::from)?;
    Ok(out)
}

/// Returns whether a text is a JSON document rather than a typed sequent.
fn is_json(input: &str) -> bool {
    input.trim_start().starts_with('{')
}

/// Reads a sequent, typed or as JSON.
fn sequent_of(input: &str, settings: &Settings) -> Result<Sequent, Failure> {
    if is_json(input) {
        let mut json = serde_json::Deserializer::from_str(input);
        let sequent = wire::upgrade(&mut json, &settings.limits).map_err(Failure::from)?;
        json.end().map_err(Failure::json)?;
        Ok(sequent)
    } else {
        Sequent::parse_within(input, &settings.limits).map_err(Failure::from)
    }
}

/// Reads a session document.
fn session_of(input: &str, settings: &Settings) -> Result<Interactive, Failure> {
    let mut json = serde_json::Deserializer::from_str(input);
    let state = wire::upgrade(&mut json, &settings.limits).map_err(Failure::from)?;
    json.end().map_err(Failure::json)?;
    Ok(state)
}

/// Returns the position a page gives as a number: a `u32` widens to a
/// `usize` on every target the crate builds for, 32 bits included.
fn position_of(position: u32) -> usize {
    position as usize
}

/// Returns a goal or a position the page gives as a JavaScript number,
/// which must be an integer from 0 to `u32::MAX`; the exports take it as a
/// number, since wasm-bindgen would convert any number to a `u32` modulo
/// 2³², so that `2 ** 32 + 3` would be 3 and `1.5` would be 1.
///
/// # Errors
///
/// A number that is not such an integer (`NaN` included), named as `what`.
pub fn index(number: f64, what: &str) -> Result<u32, Failure> {
    if number.fract() == 0.0 && (0.0..=f64::from(u32::MAX)).contains(&number) {
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "an integer from 0 to u32::MAX, as just checked"
        )]
        Ok(number as u32)
    } else {
        Err(Failure::refused(
            "json",
            "malformed",
            &format!(
                "{what} must be an integer from 0 to {}, not {number}",
                u32::MAX
            ),
        ))
    }
}

/// Returns, for an unprovable outcome, whether its refutation passes its
/// own checker.
fn refutation_checked(outcome: &Outcome) -> Option<bool> {
    match &outcome.verdict {
        Verdict::Unprovable(disproof) => Some(disproof.check().is_ok()),
        _ => None,
    }
}

/// Returns a refusal in its written form, as an error.
fn refusal(refusal: linlog::Refusal) -> serde_json::Value {
    serde_json::to_value(Error::Refused(refusal)).unwrap_or_default()
}

/// Writes a value as JSON.
fn written(value: &impl Serialize) -> Result<String, Failure> {
    serde_json::to_string(value).map_err(Failure::json)
}
