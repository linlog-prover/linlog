// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The calls of the bindings over JSON text: each reads its documents and
//! the page's settings, does one thing with the library, and writes what
//! the page shows. The page holds every document (the session, the
//! settings); nothing is kept here between calls, so a worker stopped
//! mid-call loses nothing.

use crate::failure::Failure;
use crate::tab;
use linlog::export::{Drawable, latex, pdf, png, rocq, svg, typst};
use linlog::proofs::Compact;
use linlog::proofs::interactive::{Applicable, Inverted, Needs, Split};
use linlog::proofs::{Sides, cut};
use linlog::{
    ClassicalRule, ClassicalSequent, ClassicalSides, Criterion, CutFormula, Disproof, Error,
    Fragment, GoalId, IllSequent, IllSyntax, InfId, Inference, Interactive, MellStructure, Member,
    Mode, Named, NetError, Node, NodeId, Outcome, Progress, Proof, ProofStructure, Reading, Rule,
    Sequent, Settings, Side, Step, TwoSided, Verdict, VertexId, nets, wire,
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

/// Reads a document as [`read`] does, for a session in `mode`: in an
/// intuitionistic mode of the commutative order a sequent typed is read by
/// the settings' `input.intuitionistic`, its own syntax alone by default
/// (a symbol of classical linear logic refused with a hint), a sequent in
/// JSON may be an intuitionistic sequent's document as well as `seq json`'s,
/// and `text` is the sequent two-sided; in any other mode, and for a
/// session, it is [`read`].
///
/// # Errors
///
/// As [`read`], an unknown mode, and a sequent with no intuitionistic
/// reading.
pub fn read_as(input: &str, mode: &str, settings: &str) -> Result<String, Failure> {
    let mode: Mode = mode.parse().map_err(Failure::from)?;
    let read_settings = tab::read(settings)?;
    if !commutative_intuitionistic(mode) {
        return read(input, settings);
    }
    if is_json(input) {
        let probe: Probe = serde_json::from_str(input).map_err(Failure::json)?;
        if probe.inferences.is_some() {
            return read(input, settings);
        }
    }
    let sequent = ill_of(input, &read_settings)?;
    written(&Read {
        kind: "sequent",
        text: sequent.to_string(),
        fragment: Fragments::of(sequent.sequent()),
        sequent: sequent.sequent(),
        mode: None,
        session: None,
    })
}

/// A formula of a goal as the page shows it.
#[derive(Serialize)]
struct Formula {
    /// The formula in the parser's Unicode syntax: in intuitionistic mode
    /// as the reading writes it in its position, as `reading` does.
    text: String,
    /// Its occurrence in the sequent's forest.
    occurrence: u32,
    /// In intuitionistic mode, `input` for a hypothesis and `output` for
    /// the goal, `null` in classical mode.
    side: Option<&'static str>,
    /// In intuitionistic mode, the formula as the reading writes it in its
    /// position (a hypothesis `A ⊸ B` as `A ⊸ B`, which the one-sided
    /// sequent holds as `A ⊗ ~B`), the same as `text`; absent in classical
    /// mode.
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
    /// The session's inferences that a step of an invertible rule closed
    /// (`Interactive::is_invertible`), whichever took it, in order.
    invertible: Vec<GoalId>,
    /// The session's cuts, each its inference and its cut formula as typed,
    /// in the mode's syntax, which [`cut`] reads again: what a replay of
    /// the session makes each cut with.
    cuts: Vec<CutMade>,
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
            invertible: invertible_steps(state),
            cuts: cuts_made(state),
            net: nets_exist(fragment, state.mode())
                .err()
                .and_then(|refusal| serde_json::to_value(Error::from(refusal)).ok()),
        }
    }
}

/// A cut of a session: its inference and its formula as typed.
#[derive(Serialize)]
struct CutMade {
    /// The cut's inference.
    inference: GoalId,
    /// The cut formula in the mode's syntax: under the reading in an
    /// intuitionistic mode, as written in a classical session that keeps
    /// how it was written, else one-sided.
    formula: String,
}

/// Returns a session's cuts with their formulas as typed.
fn cuts_made(state: &Interactive) -> Vec<CutMade> {
    let Ok(derivation) = state.derivation_within(
        &linlog::ViewOptions::default(),
        &linlog::Limits::default().with_derivation_bytes(None),
        never,
    ) else {
        return Vec::new();
    };
    (derivation.inferences().iter())
        .zip(state.derivation_ids())
        .filter_map(|(inference, goal)| {
            let a = inference.cut()?;
            let formula = match state.formula_as_written(a) {
                Some(written) if !state.mode().is_intuitionistic() => written.to_string(),
                _ => state.formula(a).to_string(),
            };
            Some(CutMade {
                inference: goal,
                formula,
            })
        })
        .collect()
}

/// Returns the inferences of a session that a step of an invertible rule
/// closed: each closed inference whose rule, on its conclusion, is one.
fn invertible_steps(state: &Interactive) -> Vec<GoalId> {
    let Ok(derivation) = state.derivation_within(
        &linlog::ViewOptions::default(),
        &linlog::Limits::default().with_derivation_bytes(None),
        never,
    ) else {
        return Vec::new();
    };
    let mut found: Vec<GoalId> = (derivation.inferences().iter())
        .zip(state.derivation_ids())
        .filter(|(inference, _)| {
            let step = Step::new(inference.principal().unwrap_or(0), inference.rule());
            inference.rule().rule != Rule::Open && state.is_invertible(inference.sequent(), &step)
        })
        .map(|(_, goal)| goal)
        .collect();
    found.sort_unstable();
    found
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
/// `intuitionistic-affine`), and returns the session with its view; with
/// the settings' `interactive.invertible` on, every invertible rule
/// applied to it first.
///
/// # Errors
///
/// As [`read`] for the sequent, an unknown mode, and a sequent the mode
/// does not read (an intuitionistic one needs a reading).
pub fn start(sequent: &str, mode: &str, settings: &str) -> Result<String, Failure> {
    let settings = tab::read(settings)?;
    let mode: Mode = mode.parse().map_err(Failure::from)?;
    // A typed classical sequent keeps how it was written, for a drawing
    // two-sided at any point of the proof.
    let mut state = if !mode.is_intuitionistic() && !mode.is_ordered() && !is_json(sequent) {
        let written =
            ClassicalSequent::parse_within(sequent, &settings.limits).map_err(Failure::from)?;
        Interactive::of_classical(&written, mode, &settings.limits)
    } else {
        Interactive::within(
            &sequent_in(sequent, mode, &settings)?,
            mode,
            &settings.limits,
        )
    }
    .map_err(Failure::from)?;
    let mut automatic = Automatic::default();
    if settings.interactive.invertible {
        automatic.add(state.invert_all(&inverting(&settings)));
    }
    changed(&state, automatic)
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
    /// The goals the step opened, in the order of the rule's premises;
    /// with `interactive.invertible`, those its invertible steps left
    /// open, from left to right.
    opened: Vec<GoalId>,
    /// The invertible steps taken after it.
    #[serde(flatten)]
    automatic: Automatic,
}

/// What the invertible rules applied by themselves did.
#[derive(Default, Serialize)]
struct Automatic {
    /// How many invertible steps were taken: none unless the settings'
    /// `interactive.invertible` is on, or the call applies them anyway.
    inverted: usize,
    /// Left out, or the refusal of the bound that stopped the invertible
    /// steps, the steps taken standing.
    #[serde(skip_serializing_if = "Option::is_none")]
    inverted_refused: Option<serde_json::Value>,
}

impl Automatic {
    /// Counts a run of invertible steps and returns the goals it left open.
    fn add(&mut self, inverted: Inverted) -> Vec<GoalId> {
        self.inverted += inverted.steps;
        if let Err(refused) = inverted.finished {
            self.inverted_refused.get_or_insert(refusal(refused));
        }
        inverted.open
    }
}

/// Returns the bounds of a run of invertible steps: the settings' memory
/// bound, and no more than the size of a derivation the page draws, so
/// that the steps never make a session the page could not show.
fn inverting(settings: &Settings) -> linlog::Limits {
    let limits = settings.limits;
    let bound = match (limits.memory_bytes, limits.derivation_bytes) {
        (Some(memory), Some(derivation)) => Some(memory.min(derivation)),
        (memory, derivation) => memory.or(derivation),
    };
    limits.with_memory_bytes(bound)
}

/// Applies a step to a goal and returns the session with the goals it
/// opened. With the settings' `interactive.invertible` on, every
/// invertible rule is then applied to those goals
/// (`Interactive::invert`), each an ordinary step.
///
/// # Errors
///
/// The step's refusal with its reason (`kind`: `no_dual`, `not_alone`,
/// `not_quest`, …), the session unchanged.
pub fn apply(session: &str, goal: u32, step: &str, settings: &str) -> Result<String, Failure> {
    let settings = tab::read(settings)?;
    let mut state = session_of(session, &settings)?;
    let mut opened = state
        .apply(GoalId::new(goal), &step_of(step)?)
        .map_err(|e| Failure::described(&Error::from(e), &state))?;
    let mut automatic = Automatic::default();
    if settings.interactive.invertible {
        let limits = inverting(&settings);
        for goal in std::mem::take(&mut opened) {
            let inverted = state
                .invert(goal, &limits)
                .map_err(|e| Failure::described(&Error::from(e), &state))?;
            opened.extend(automatic.add(inverted));
        }
    }
    changed(&state, Opened { opened, automatic })
}

/// Cuts an open goal on a formula typed as text, written as the session's
/// mode writes formulas (intuitionistic as `input.intuitionistic` says,
/// else classical as written), and returns the session with the two goals
/// it opened: the first proves the formula, with the goal's formulas at
/// the positions `left` (a JSON array), and the second takes it as a
/// hypothesis, with the others (`Interactive::cut`). With the settings'
/// `interactive.invertible` on, every invertible rule is then applied to
/// those goals, as after `apply`; a cut itself is no invertible step.
///
/// # Errors
///
/// The parse error of a formula that does not read (its span in the
/// formula's text), `cut_formula` for text of another number of formulas
/// than one, and the step's refusal (`kind`: `split`, `succedents`,
/// `mode` in an ordered mode, …), the session unchanged.
pub fn cut(
    session: &str,
    goal: u32,
    formula: &str,
    left: &str,
    settings: &str,
) -> Result<String, Failure> {
    let settings = tab::read(settings)?;
    let mut state = session_of(session, &settings)?;
    let left: Vec<usize> = serde_json::from_str(left).map_err(Failure::json)?;
    let formula =
        CutFormula::parse_within(formula, state.mode(), &settings.input, &settings.limits)?;
    let mut opened = state
        .cut(GoalId::new(goal), &formula, &left, &settings.limits)
        .map_err(|e| Failure::described(&e, &state))?
        .to_vec();
    let mut automatic = Automatic::default();
    if settings.interactive.invertible {
        let limits = inverting(&settings);
        for goal in std::mem::take(&mut opened) {
            let inverted = state
                .invert(goal, &limits)
                .map_err(|e| Failure::described(&Error::from(e), &state))?;
            opened.extend(automatic.add(inverted));
        }
    }
    changed(&state, Opened { opened, automatic })
}

/// Applies every invertible rule to every open goal
/// (`Interactive::invert_all`), whatever the settings say, and returns
/// the session with how many steps that took: what turning the setting
/// on does to a session under way.
///
/// # Errors
///
/// As the session's reader.
pub fn invert_all(session: &str, settings: &str) -> Result<String, Failure> {
    let settings = tab::read(settings)?;
    let mut state = session_of(session, &settings)?;
    let mut automatic = Automatic::default();
    automatic.add(state.invert_all(&inverting(&settings)));
    changed(&state, automatic)
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
    /// Whether the last of the steps applies a rule invertible in the
    /// session's mode (`Interactive::is_invertible`); `null` without
    /// steps.
    invertible: Option<bool>,
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
    let first = match &outcome.verdict {
        Verdict::Proved(proof) => first_steps(&state, proof, &settings)?,
        _ => None,
    };
    let (steps, invertible) = first.unzip();
    written(&Hint {
        outcome,
        steps,
        invertible,
    })
}

/// Returns the steps a proof of a goal of a session begins with, as
/// [`Hint::steps`] says, with whether the last is invertible, or `None`
/// when the bounds refuse its derivation.
fn first_steps(
    state: &Interactive,
    proof: &linlog::Proof,
    settings: &Settings,
) -> Result<Option<(Vec<StepForm>, bool)>, Failure> {
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
            _ => return Ok(Some((steps, state.is_invertible(members, &step)))),
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
    let mode: Mode = mode.parse().map_err(Failure::from)?;
    let sequent = sequent_in(sequent, mode, &settings)?;
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

/// Answers where a session's proof has a net: for a sequent of unit-free
/// MLL, and with exponentials one with boxes, of unit-free MELL in
/// classical mode, with or without Mix.
fn nets_exist(fragment: Fragment, mode: Mode) -> Result<(), NetError> {
    if fragment.has_exponentials() {
        MellStructure::exist(fragment, mode)
    } else {
        nets::exist(fragment, mode)
    }
}

/// Returns the proof net of a finished session as an SVG drawing: for a
/// sequent of unit-free MLL, and with `styles.svg.essential` for one with
/// an intuitionistic reading, its essential net; for one of unit-free MELL
/// in classical or intuitionistic mode, its net with boxes, each `!`-box a
/// rectangle, with `styles.svg.essential` the essential net with boxes (ids
/// `o<n>` on the only vertex of occurrence `n`, `v<k>` on vertex `k` of an
/// occurrence with several, `b<k>` on the box of the `!` vertex `k`); every
/// subformula written, or with `styles.svg.net` `"compact"` only the
/// literals and the conclusions; in linlog's look, or with
/// `styles.svg.net_look` `"textbook"` (or its fields `premises`,
/// `conclusions` and `why_not`) the textbooks'.
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
    let mode = state.mode();
    nets_exist(proof.sequent().fragment(), mode).map_err(|e| Failure::from(Error::from(e)))?;
    let criterion = Criterion::of(mode).map_err(|e| Failure::from(Error::from(e)))?;
    // An intuitionistic net is drawn as its essential net, whose formulas
    // are the intuitionistic ones.
    let svg_style = &settings
        .styles
        .svg
        .clone()
        .with_essential(settings.styles.svg.essential || mode.is_intuitionistic());
    let limits = &settings.limits;
    if proof.sequent().fragment().has_exponentials() {
        let net =
            MellStructure::from_proof(&proof, criterion, limits, never).map_err(Failure::from)?;
        return svg::mell_net(&net, svg_style, limits, never).map_err(Failure::from);
    }
    let net =
        ProofStructure::from_proof(&proof, criterion, limits, never).map_err(Failure::from)?;
    svg::net(&net, svg_style, limits, never).map_err(Failure::from)
}

/// Returns a session's derivation in a format: `text` (the command's text
/// tree), `latex`, `typst` and `svg` of the session's tree, open goals
/// included, and `rocq`, the certificate of a finished session's checked
/// proof; each in the settings' style for it. With `view.classical`
/// `"two"`, a session of a commutative classical mode is drawn two-sided
/// as its sequent was written ([`TwoSided`]), or for a session that keeps
/// no record, one read from JSON, as its one-sided syntax writes it.
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
    let mode = state.mode();
    let as_written = settings.view.classical == ClassicalSides::Two
        && !mode.is_intuitionistic()
        && !mode.is_ordered();
    let one_sided;
    let writing = match state.writing() {
        Some(writing) => writing,
        None if as_written => {
            // The session's sequent passed the bounds already.
            let unbounded = linlog::Limits::default().with_occurrences(None);
            one_sided = ClassicalSequent::read_one_sided(state.sequent().clone(), &unbounded)
                .map_err(Failure::from)?;
            one_sided.writing()
        }
        None => &[],
    };
    let two_sided = if as_written {
        Some(TwoSided::new(&derivation, writing, limits).map_err(Failure::from)?)
    } else {
        None
    };
    let drawable: Drawable<'_> = match &two_sided {
        Some(view) => view.into(),
        None => (&derivation).into(),
    };
    match format {
        "text" => match &two_sided {
            Some(view) => view.write_text(&styles.text, &mut out, never),
            None => derivation.write_text(&styles.text, &mut out, never),
        },
        "latex" => latex::write(drawable, &styles.latex, &mut out, never),
        "typst" => typst::write(drawable, &styles.typst, &mut out, never),
        "svg" => svg::write(drawable, &styles.svg, &mut out, never),
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

/// Returns whether `mode` reads its sequents as intuitionistic sequents
/// of the commutative order, which the strict syntax is for.
fn commutative_intuitionistic(mode: Mode) -> bool {
    mode.is_intuitionistic() && !mode.is_ordered()
}

/// Reads an intuitionistic sequent: typed by the settings'
/// `input.intuitionistic`, or as JSON, an intuitionistic sequent's
/// document or a one-sided sequent read as intuitionistic.
fn ill_of(input: &str, settings: &Settings) -> Result<IllSequent, Failure> {
    let limits = &settings.limits;
    if is_json(input) {
        /// The key that tells an intuitionistic sequent's document.
        #[derive(Deserialize)]
        struct Ill {
            /// Its hypotheses.
            hypotheses: Option<serde::de::IgnoredAny>,
        }
        let probe: Ill = serde_json::from_str(input).map_err(Failure::json)?;
        if probe.hypotheses.is_some() {
            let mut json = serde_json::Deserializer::from_str(input);
            let sequent = wire::upgrade(&mut json, limits).map_err(Failure::from)?;
            json.end().map_err(Failure::json)?;
            return Ok(sequent);
        }
        let sequent = sequent_of(input, settings)?;
        return IllSequent::read_one_sided(sequent, limits).map_err(Failure::from);
    }
    match settings.input.intuitionistic {
        IllSyntax::OneSided => {
            let sequent = Sequent::parse_within(input, limits).map_err(Failure::from)?;
            IllSequent::read_one_sided(sequent, limits).map_err(Failure::from)
        }
        // A syntax a later version adds reads as the strict one here.
        _ => IllSequent::parse_within(input, limits).map_err(Failure::from),
    }
}

/// Reads a sequent for a session or a search in `mode`: in an
/// intuitionistic mode of the commutative order as [`ill_of`] reads it,
/// else typed or as JSON.
fn sequent_in(input: &str, mode: Mode, settings: &Settings) -> Result<Sequent, Failure> {
    if commutative_intuitionistic(mode) {
        return ill_of(input, settings).map(IllSequent::into_sequent);
    }
    sequent_of(input, settings)
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

/// A cut of a proof as the page shows it.
#[derive(Serialize)]
struct CutShown {
    /// The cut, a node of the proof, which [`cut_step`] takes as `at`.
    node: u32,
    /// The cut formula as the mode reads it.
    formula: String,
    /// The inferences of the drawing that show it (its `i<n>`): one, or
    /// one for every place the proof reads a shared cut at.
    inferences: Vec<u32>,
}

/// A proof of an elimination as the page shows it: the proof, its cuts
/// and its derivation drawn.
#[derive(Serialize)]
struct CutsView<'a> {
    /// The proof, which the page keeps for the next call.
    proof: &'a Proof,
    /// Its cuts, in the order of its arena.
    cuts: Vec<CutShown>,
    /// The cut the strategy reduces next, `null` for a cut-free proof.
    next: Option<u32>,
    /// The derivation as SVG, as [`draw`] draws a session's, or `null`
    /// where its bound refused it.
    drawing: Option<String>,
    /// The refusal of the drawing, where there is one.
    #[serde(skip_serializing_if = "Option::is_none")]
    refused: Option<serde_json::Value>,
}

/// A proof's derivation drawn as the page draws a session's, with the
/// node of the proof each drawn inference shows.
struct Drawn {
    /// The drawing.
    svg: String,
    /// For the inference `i<n>` of the drawing, at `n`, the node of the
    /// proof it shows; `None` for an inference the view adds.
    nodes: Vec<Option<NodeId>>,
}

impl Drawn {
    /// Returns the inferences of the drawing that show `node`.
    fn showing(&self, node: NodeId) -> Vec<u32> {
        (0u32..)
            .zip(&self.nodes)
            .filter(|&(_, &shown)| shown == Some(node))
            .map(|(n, _)| n)
            .collect()
    }

    /// Returns the ids of the drawing's elements that show `node`.
    fn ids(&self, node: Option<NodeId>) -> Vec<String> {
        node.map_or_else(Vec::new, |node| {
            let ids = self.showing(node).into_iter();
            ids.map(|n| format!("i{n}")).collect()
        })
    }
}

/// Draws the derivation of `proof`, a proof of `state`'s sequent, as
/// [`export`] draws the session's: two-sided in an intuitionistic mode,
/// and in a classical one as written with `view.classical` `"two"`.
fn drawn(proof: &Proof, state: &Interactive, settings: &Settings) -> Result<Drawn, Error> {
    let (mode, limits) = (state.mode(), &settings.limits);
    let view = if mode.is_intuitionistic() {
        settings.view.with_sides(Sides::Two)
    } else {
        settings.view.with_sides(Sides::One)
    };
    let derivation = proof.derivation_within(&view, limits, never)?;
    let mut svg = String::new();
    let as_written = settings.view.classical == ClassicalSides::Two
        && !mode.is_intuitionistic()
        && !mode.is_ordered();
    if !as_written {
        svg::write(
            Drawable::from(&derivation),
            &settings.styles.svg,
            &mut svg,
            never,
        )?;
        let nodes = derivation.inferences().iter().map(Inference::node);
        return Ok(Drawn {
            svg,
            nodes: nodes.collect(),
        });
    }
    let one_sided;
    let writing = match state.writing() {
        Some(writing) => writing,
        None => {
            // The session's sequent passed the bounds already.
            let unbounded = linlog::Limits::default().with_occurrences(None);
            one_sided = ClassicalSequent::read_one_sided(state.sequent().clone(), &unbounded)?;
            one_sided.writing()
        }
    };
    let two_sided = TwoSided::new(&derivation, writing, limits)?;
    svg::write(
        Drawable::from(&two_sided),
        &settings.styles.svg,
        &mut svg,
        never,
    )?;
    // The view's inferences are numbered from 0, fewer than 2³² as its
    // ids count them.
    let nodes = (0..two_sided.len() as u32)
        .map(InfId::new)
        .map(|id| match two_sided.rule(id) {
            ClassicalRule::NegationLeft | ClassicalRule::NegationRight => None,
            _ => derivation.inference(two_sided.shows(id)).node(),
        });
    Ok(Drawn {
        svg,
        nodes: nodes.collect(),
    })
}

/// Returns the text of an occurrence as `mode` reads it: under the
/// intuitionistic reading in an intuitionistic mode, else one-sided.
fn formula_in(forest: &linlog::Forest, occurrence: linlog::OccId, mode: Mode) -> String {
    match Reading::of_mode(forest, mode)
        .ok()
        .filter(|_| mode.is_intuitionistic())
    {
        Some(reading) => reading.formula(occurrence).to_string(),
        None => forest.formula(occurrence).to_string(),
    }
}

impl<'a> CutsView<'a> {
    /// Returns the view of `proof`, a proof of `state`'s sequent, drawn
    /// with the settings, with the drawing it made where it made one.
    fn of(proof: &'a Proof, state: &Interactive, settings: &Settings) -> (Self, Option<Drawn>) {
        let (drawing, refused) = match drawn(proof, state, settings) {
            Ok(drawn) => (Some(drawn), None),
            Err(error) => (None, serde_json::to_value(error).ok()),
        };
        let cuts = (0u32..)
            .zip(proof.nodes())
            .filter_map(|(id, &node)| match node {
                Node::Cut(a, ..) => Some(CutShown {
                    node: id,
                    formula: formula_in(proof.forest(), proof.occurrence(a), state.mode()),
                    inferences: drawing
                        .as_ref()
                        .map_or_else(Vec::new, |d| d.showing(NodeId::new(id))),
                }),
                _ => None,
            })
            .collect();
        let view = Self {
            proof,
            cuts,
            next: cut::Strategy::default().pick(proof).map(NodeId::get),
            drawing: drawing.as_ref().map(|d| d.svg.clone()),
            refused,
        };
        (view, drawing)
    }
}

/// Reads the proof of an elimination: a proof's document of the
/// session's sequent, or with an empty text the session's own proof.
fn proof_in(proof: &str, state: &Interactive, settings: &Settings) -> Result<Proof, Failure> {
    if proof.trim().is_empty() {
        return state
            .proof(&settings.limits, never)
            .map_err(|e| Failure::described(&e, state));
    }
    let mut json = serde_json::Deserializer::from_str(proof);
    let proof: Proof = wire::upgrade(&mut json, &settings.limits).map_err(Failure::from)?;
    json.end().map_err(Failure::json)?;
    if proof.sequent() != state.sequent() {
        return Err(Error::ForeignProof.into());
    }
    Ok(proof)
}

/// A step of an elimination as the page shows it.
#[derive(Serialize)]
struct Taken {
    /// The cut reduced, a node of the proof the step was taken on, or for
    /// a net its cut link, its end in a cut formula's tree first.
    cut: serde_json::Value,
    /// The cut formula, the occurrence of the cut's, as the mode reads it.
    formula: String,
    /// The case, in snake case (`axiom`, `tensor_par`, …).
    case: cut::Case,
    /// The case as a phrase: `the axiom case`, `contraction`, …
    name: &'static str,
    /// Where the step acted besides the cut, if anywhere: a node of the
    /// proof, or a vertex of the net, the step was taken on.
    above: Option<u32>,
    /// The ids of the elements of that proof's or net's drawing to
    /// highlight, `null` where it was not drawn.
    highlight: Option<Highlight>,
}

/// The ids of a drawing's elements a step acted on.
#[derive(Serialize)]
struct Highlight {
    /// What shows the cut: the inferences of a derivation (`i<n>`), or a
    /// net's cut link (`c<x>-<y>`).
    cut: Vec<String>,
    /// What shows where the step acted besides it: inferences, or a net's
    /// vertex (`o<n>` or `v<k>`) or box (`b<k>`).
    above: Vec<String>,
}

impl Taken {
    /// Returns a step of an elimination on a proof term, its ids those of
    /// `drawn`, the drawing of the proof it was taken on.
    fn term(step: &cut::Step, proof: &Proof, mode: Mode, drawn: Option<&Drawn>) -> Self {
        Self {
            cut: step.cut.get().into(),
            formula: formula_in(proof.forest(), proof.occurrence(step.formula), mode),
            case: step.case,
            name: step.case.name(),
            above: step.above.map(NodeId::get),
            highlight: drawn.map(|drawn| Highlight {
                cut: drawn.ids(Some(step.cut)),
                above: drawn.ids(step.above),
            }),
        }
    }
}

/// A step and what the page shows of the proof or net it made.
#[derive(Serialize)]
struct Stepped<T: Serialize> {
    /// The step.
    step: Taken,
    /// For a net, the net the step was taken on, drawn with what the step
    /// acted on in the highlight colour.
    #[serde(skip_serializing_if = "Option::is_none")]
    marked: Option<String>,
    /// The proof or net made.
    #[serde(flatten)]
    made: T,
}

/// A step of an elimination to the end, with the drawing of the proof or
/// net it was taken on.
#[derive(Serialize)]
struct Run {
    /// The step.
    #[serde(flatten)]
    step: Taken,
    /// The drawing, of a net with what the step acted on in the highlight
    /// colour; `null` for a step past `cut.drawn_bytes` or one whose
    /// drawing its bound refused.
    drawing: Option<String>,
    /// The refusal of that drawing, where there is one.
    #[serde(skip_serializing_if = "Option::is_none")]
    refused: Option<serde_json::Value>,
}

/// An elimination to the end and what the page shows of the proof or net
/// it reached.
#[derive(Serialize)]
struct Ran<T: Serialize> {
    /// The steps, in order.
    steps: Vec<Run>,
    /// `cut_free`, or `refused` when a bound ended the elimination.
    end: &'static str,
    /// The bound that ended it, in an error's form, for `refused`.
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<serde_json::Value>,
    /// The first step, counted from 1, whose drawing `cut.drawn_bytes`
    /// left out, with every step after it; `null` when every step is
    /// drawn.
    undrawn: Option<usize>,
    /// The proof or net reached.
    #[serde(flatten)]
    reached: T,
}

/// Returns the end of an elimination as `end` and `reason`.
fn ended(end: &cut::End) -> (&'static str, Option<serde_json::Value>) {
    match end {
        cut::End::CutFree => ("cut_free", None),
        cut::End::Refused(r) => ("refused", Some(refusal(r.clone()))),
        _ => ("refused", None),
    }
}

/// The drawings of an elimination's steps within `cut.drawn_bytes`: each
/// is kept while the bytes kept so far, its own included, stay within the
/// bound, and none after the first that would pass it.
struct Drawings {
    /// The bound, in bytes.
    limit: Option<u64>,
    /// The bytes kept so far.
    kept: u64,
    /// The first step left out, counted from 1.
    undrawn: Option<usize>,
}

impl Drawings {
    /// Returns whether the steps are still drawn.
    const fn open(&self) -> bool {
        self.undrawn.is_none()
    }

    /// Keeps step `k`'s drawing if it fits the bound, and returns it.
    fn keep(&mut self, k: usize, drawing: String) -> Option<String> {
        let total = self.kept.saturating_add(drawing.len() as u64);
        if self.limit.is_some_and(|limit| total > limit) {
            self.undrawn = Some(k);
            return None;
        }
        self.kept = total;
        Some(drawing)
    }
}

/// Returns the proof of an elimination with its cuts, drawn, and the cut
/// the strategy reduces next: `proof` a proof of the finished session's
/// sequent as this call or [`cut_step`] wrote it, or with an empty text
/// the session's own proof. Answers `{"proof", "cuts": [{"node",
/// "formula", "inferences"}], "next", "drawing"}`, `inferences` the
/// drawing's `i<n>` that show the cut.
///
/// # Errors
///
/// As the session's reader, `open_goals` for an unfinished session,
/// `foreign_proof` for a proof of another sequent, and the proof's reader;
/// a drawing past its bound is `null` with its refusal in `refused`.
pub fn cuts(session: &str, proof: &str, settings: &str) -> Result<String, Failure> {
    let settings = tab::read(settings)?;
    let state = session_of(session, &settings)?;
    let proof = proof_in(proof, &state, &settings)?;
    written(&CutsView::of(&proof, &state, &settings).0)
}

/// Takes one step of cut elimination on a proof of a finished session
/// (`proof` as [`cuts`] reads it): the cut at the node `at`, or with
/// `None` the one the strategy picks (`cut.strategy`), within the
/// settings' `cut` and `limits`. Answers the step, `{"cut", "formula",
/// "case", "name", "above", "highlight": {"cut", "above"}}` (the ids of
/// the drawing of the proof given that show the cut and where the step
/// acted), and the proof made as [`cuts`] answers.
///
/// # Errors
///
/// As [`cuts`] reads its documents; `no_cut` for an `at` that is no cut
/// and for a cut-free proof; `elimination_limit` for a proof made past
/// `cut.nodes`; and the checker's.
pub fn cut_step(
    session: &str,
    proof: &str,
    at: Option<u32>,
    settings: &str,
) -> Result<String, Failure> {
    let settings = tab::read(settings)?;
    let state = session_of(session, &settings)?;
    let proof = proof_in(proof, &state, &settings)?;
    let mode = state.mode();
    let at = at.map(NodeId::new);
    let reduction = cut::step(&proof, mode, at, &settings.cut, &settings.limits, never)
        .map_err(|e| Failure::described(&e, &state))?;
    let before = drawn(&proof, &state, &settings).ok();
    let step = Taken::term(&reduction.step, &proof, mode, before.as_ref());
    written(&Stepped {
        step,
        marked: None,
        made: CutsView::of(&reduction.proof, &state, &settings).0,
    })
}

/// Eliminates the cuts of a proof of a finished session (`proof` as
/// [`cuts`] reads it) by the strategy, within the settings' `cut` (`steps`,
/// `nodes`, and `drawn_bytes` for the drawings of the steps) and `limits`.
/// Answers `{"steps": [{…, "drawing"}], "end", "reason", "undrawn"}`, each
/// step as [`cut_step`] gives it with the drawing of the proof it was taken
/// on, and the proof reached as [`cuts`] answers.
///
/// # Errors
///
/// As [`cuts`] reads its documents, and the checker's for a proof that is
/// none; a bound that ends the elimination is its `end`, not an error.
pub fn eliminate(session: &str, proof: &str, settings: &str) -> Result<String, Failure> {
    let settings = tab::read(settings)?;
    let state = session_of(session, &settings)?;
    let proof = proof_in(proof, &state, &settings)?;
    let mode = state.mode();
    let mut drawings = Drawings {
        limit: settings.cut.drawn_bytes,
        kept: 0,
        undrawn: None,
    };
    let mut steps = Vec::new();
    let elimination = cut::eliminate(
        &proof,
        mode,
        &settings.cut,
        &settings.limits,
        never,
        |before, step| {
            let k = steps.len() + 1;
            let (mut drawing, mut refused, mut shown) = (None, None, None);
            if drawings.open() {
                match drawn(before, &state, &settings) {
                    Ok(made) => {
                        drawing = drawings.keep(k, made.svg.clone());
                        shown = drawing.is_some().then_some(made);
                    }
                    Err(error) => refused = serde_json::to_value(error).ok(),
                }
            }
            steps.push(Run {
                step: Taken::term(step, before, mode, shown.as_ref()),
                drawing,
                refused,
            });
        },
    )
    .map_err(|e| Failure::described(&e, &state))?;
    let (end, reason) = ended(&elimination.end);
    written(&Ran {
        steps,
        end,
        reason,
        undrawn: drawings.undrawn,
        reached: CutsView::of(&elimination.proof, &state, &settings).0,
    })
}

/// A net of an elimination: a proof structure, or one with boxes for a
/// sequent with exponentials and for a proof that cuts an occurrence
/// twice.
#[derive(Serialize)]
#[serde(untagged)]
enum Net {
    /// Without boxes, its vertices the forest's occurrences.
    Mll(ProofStructure),
    /// With boxes.
    Mell(MellStructure),
}

/// A net of an elimination, borrowed.
#[derive(Clone, Copy)]
enum NetRef<'a> {
    /// Without boxes.
    Mll(&'a ProofStructure),
    /// With boxes.
    Mell(&'a MellStructure),
}

/// A cut link of a net as the page shows it.
#[derive(Serialize)]
struct LinkShown {
    /// Its two ends, its end in a cut formula's tree first; either is the
    /// `at` of [`net_cut_step`].
    ends: [u32; 2],
    /// The id of its bar in the drawing, `c<x>-<y>`.
    id: String,
    /// The cut formula as the mode reads it.
    formula: String,
}

/// A net of an elimination as the page shows it.
#[derive(Serialize)]
struct NetView<'a> {
    /// The net, which the page keeps for the next call.
    net: &'a Net,
    /// Whether it has boxes (`MellStructure`'s form).
    boxes: bool,
    /// Its cut links, in the net's order.
    cuts: Vec<LinkShown>,
    /// The cut link the strategy reduces next, `null` for a net without
    /// cut links.
    next: Option<[u32; 2]>,
    /// The net as SVG, as [`draw_net`] draws a proof's, or `null` where
    /// its bound refused it.
    drawing: Option<String>,
    /// The refusal of the drawing, where there is one.
    #[serde(skip_serializing_if = "Option::is_none")]
    refused: Option<serde_json::Value>,
}

impl Net {
    /// Returns the net of a proof in `mode`, as the command makes it: with
    /// boxes for a sequent with exponentials, and for one without where
    /// the proof cuts an occurrence twice, which needs two vertices of one
    /// occurrence.
    fn of(proof: &Proof, mode: Mode, limits: &linlog::Limits) -> Result<Self, Error> {
        nets_exist(proof.sequent().fragment(), mode)?;
        let criterion = Criterion::of(mode)?;
        let boxed = || MellStructure::from_proof(proof, criterion, limits, never).map(Self::Mell);
        if proof.sequent().fragment().has_exponentials() {
            return boxed();
        }
        match ProofStructure::from_proof(proof, criterion, limits, never) {
            Err(Error::Net(error))
                if matches!(
                    *error,
                    NetError::Placement {
                        fault: linlog::Placement::Cut,
                        ..
                    }
                ) =>
            {
                boxed()
            }
            made => made.map(Self::Mll),
        }
    }

    /// Reads a net's document, with boxes when it has `vertices`.
    fn read(input: &str, settings: &Settings) -> Result<Self, Failure> {
        /// The key that tells a structure with boxes.
        #[derive(Deserialize)]
        struct Boxes {
            /// A structure with boxes' vertices.
            vertices: Option<serde::de::IgnoredAny>,
        }
        let probe: Boxes = serde_json::from_str(input).map_err(Failure::json)?;
        let mut json = serde_json::Deserializer::from_str(input);
        let net = if probe.vertices.is_some() {
            Self::Mell(wire::upgrade(&mut json, &settings.limits).map_err(Failure::from)?)
        } else {
            Self::Mll(wire::upgrade(&mut json, &settings.limits).map_err(Failure::from)?)
        };
        json.end().map_err(Failure::json)?;
        Ok(net)
    }

    /// Returns the net borrowed.
    const fn as_ref(&self) -> NetRef<'_> {
        match self {
            Self::Mll(net) => NetRef::Mll(net),
            Self::Mell(net) => NetRef::Mell(net),
        }
    }
}

impl<'a> NetRef<'a> {
    /// Returns the cut links.
    fn cuts(self) -> &'a [(VertexId, VertexId)] {
        match self {
            Self::Mll(net) => net.cuts(),
            Self::Mell(net) => net.cuts(),
        }
    }

    /// Returns the formula of a vertex as `mode` reads it.
    fn formula(self, v: VertexId, mode: Mode) -> String {
        let (forest, o) = match self {
            Self::Mll(net) => (net.forest(), net.occurrence(v)),
            Self::Mell(net) => (net.forest(), net.occurrence(v)),
        };
        o.map_or_else(|| v.get().to_string(), |o| formula_in(forest, o, mode))
    }

    /// Returns the id of a vertex's element in the drawing: `o<n>` for the
    /// only vertex of occurrence `n`, `v<k>` for vertex `k` of an
    /// occurrence with several.
    fn id(self, v: VertexId) -> String {
        match self {
            Self::Mll(_) => format!("o{}", v.get()),
            Self::Mell(net) => {
                let o = net.occurrence(v);
                match net.vertices().filter(|&w| net.occurrence(w) == o).count() {
                    1 => o.map_or_else(|| format!("v{}", v.get()), |o| format!("o{}", o.get())),
                    _ => format!("v{}", v.get()),
                }
            }
        }
    }

    /// Draws the net as [`draw_net`] draws a proof's, with what `step`, a
    /// step taken on it, acted on in the highlight colour.
    fn draw(
        self,
        step: Option<&nets::cut::Step>,
        mode: Mode,
        settings: &Settings,
    ) -> Result<String, Error> {
        // An intuitionistic net is drawn as its essential net, with cut
        // links too, whose formulas are the intuitionistic ones.
        let style = &settings
            .styles
            .svg
            .clone()
            .with_essential(settings.styles.svg.essential || mode.is_intuitionistic());
        let limits = &settings.limits;
        match (self, step) {
            (Self::Mll(net), Some(step)) => svg::net_step(net, step, style, limits, never),
            (Self::Mll(net), None) => svg::net(net, style, limits, never),
            (Self::Mell(net), Some(step)) => svg::mell_net_step(net, step, style, limits, never),
            (Self::Mell(net), None) => svg::mell_net(net, style, limits, never),
        }
    }

    /// Returns a step taken on the net as the page shows it.
    fn taken(self, step: &nets::cut::Step, mode: Mode) -> Taken {
        let (x, y) = step.cut;
        let above = match (step.case, step.above) {
            (_, None) => Vec::new(),
            (cut::Case::BoxIntoBox, Some(door)) => vec![format!("b{}", door.get())],
            (_, Some(v)) => vec![self.id(v)],
        };
        Taken {
            cut: serde_json::json!([x.get(), y.get()]),
            formula: self.formula(x, mode),
            case: step.case,
            name: step.case.name(),
            above: step.above.map(VertexId::get),
            highlight: Some(Highlight {
                cut: vec![format!("c{}-{}", x.get(), y.get())],
                above,
            }),
        }
    }
}

impl<'a> NetView<'a> {
    /// Returns the view of a net in `mode`, drawn with the settings.
    fn of(net: &'a Net, mode: Mode, settings: &Settings) -> Self {
        let borrowed = net.as_ref();
        let (drawing, refused) = match borrowed.draw(None, mode, settings) {
            Ok(drawing) => (Some(drawing), None),
            Err(error) => (None, serde_json::to_value(error).ok()),
        };
        let cuts = borrowed.cuts();
        Self {
            net,
            boxes: matches!(net, Net::Mell(_)),
            cuts: cuts
                .iter()
                .map(|&(x, y)| LinkShown {
                    ends: [x.get(), y.get()],
                    id: format!("c{}-{}", x.get(), y.get()),
                    formula: borrowed.formula(x, mode),
                })
                .collect(),
            next: cuts.first().map(|&(x, y)| [x.get(), y.get()]),
            drawing,
            refused,
        }
    }
}

/// Reads the net of an elimination: a net's document as [`net_cuts`]
/// writes it, or the net of a proof, a proof's document of the session's
/// sequent (told by its `nodes`) or with an empty text the finished
/// session's own.
fn net_in(net: &str, state: &Interactive, settings: &Settings) -> Result<Net, Failure> {
    /// The key that tells a proof's document.
    #[derive(Deserialize)]
    struct Nodes {
        /// A proof's nodes.
        nodes: Option<serde::de::IgnoredAny>,
    }
    let is_proof = net.trim().is_empty()
        || serde_json::from_str::<Nodes>(net)
            .map_err(Failure::json)?
            .nodes
            .is_some();
    if !is_proof {
        return Net::read(net, settings);
    }
    let proof = proof_in(net, state, settings)?;
    Net::of(&proof, state.mode(), &settings.limits).map_err(|e| Failure::described(&e, state))
}

/// Returns the net of an elimination with its cut links, drawn, and the
/// cut link the strategy reduces next: `net` a net as this call or
/// [`net_cut_step`] wrote it, or the net of a proof of the session's
/// sequent (its document, or with an empty text the finished session's
/// own proof): a proof structure (with cut links), or one with boxes for
/// a sequent with exponentials and for a proof that cuts an occurrence
/// twice. Answers `{"net", "boxes", "cuts": [{"ends",
/// "id", "formula"}], "next", "drawing"}`, `id` the cut link's bar,
/// `c<x>-<y>`. The session gives the mode, whose reading writes the
/// formulas; the net is drawn as [`draw_net`] draws one, in an
/// intuitionistic mode as its essential net, cut links and all.
///
/// # Errors
///
/// As the session's reader, `open_goals` for an unfinished session, the
/// refusal [`view`]'s `net` gives for a sequent without nets, and the
/// net's reader; a drawing past its bound is `null` with its refusal in
/// `refused`.
pub fn net_cuts(session: &str, net: &str, settings: &str) -> Result<String, Failure> {
    let settings = tab::read(settings)?;
    let state = session_of(session, &settings)?;
    let net = net_in(net, &state, &settings)?;
    written(&NetView::of(&net, state.mode(), &settings))
}

/// Takes one step of cut elimination on a net (`net` as [`net_cuts`]
/// reads it): the cut link at the vertex `at`, either end, or with `None`
/// the strategy's, the first of the net's list, within the settings' `cut`
/// (`nodes` counting the net's vertices) and `limits`. Answers the step,
/// `{"cut": [x, y], "formula", "case", "name", "above", "highlight":
/// {"cut", "above"}}`, `marked`, the net the step was taken on drawn with
/// what it acted on in the highlight colour, and the net made as
/// [`net_cuts`] answers.
///
/// # Errors
///
/// As [`net_cuts`] reads its documents; `no_cut` for a vertex at no cut
/// link and for a net without one; `elimination_limit` for a net made past
/// `cut.nodes`; the criterion's for a structure that is no proof net.
pub fn net_cut_step(
    session: &str,
    net: &str,
    at: Option<u32>,
    settings: &str,
) -> Result<String, Failure> {
    let settings = tab::read(settings)?;
    let state = session_of(session, &settings)?;
    let net = net_in(net, &state, &settings)?;
    let mode = state.mode();
    let at = at.map(VertexId::new);
    let (options, limits) = (&settings.cut, &settings.limits);
    let (made, step) = match &net {
        Net::Mll(net) => nets::cut::step(net, at, options, limits, never)
            .map(|reduction| (Net::Mll(reduction.net), reduction.step)),
        Net::Mell(net) => nets::cut::step(net, at, options, limits, never)
            .map(|reduction| (Net::Mell(reduction.net), reduction.step)),
    }
    .map_err(|e| Failure::described(&e, &state))?;
    let before = net.as_ref();
    written(&Stepped {
        step: before.taken(&step, mode),
        marked: before.draw(Some(&step), mode, &settings).ok(),
        made: NetView::of(&made, mode, &settings),
    })
}

/// Eliminates the cut links of a net (`net` as [`net_cuts`] reads it) by
/// the strategy, within the settings' `cut` (`steps`, `nodes` counting
/// vertices, and `drawn_bytes` for the drawings of the steps) and
/// `limits`. Answers `{"steps": [{…, "drawing"}], "end", "reason",
/// "undrawn"}`, each step as [`net_cut_step`] gives it with the net it
/// was taken on drawn with its marks, and the net reached as [`net_cuts`]
/// answers.
///
/// # Errors
///
/// As [`net_cuts`] reads its documents, and the criterion's for a
/// structure that is no proof net; a bound that ends the elimination is
/// its `end`, not an error.
pub fn net_eliminate(session: &str, net: &str, settings: &str) -> Result<String, Failure> {
    let settings = tab::read(settings)?;
    let state = session_of(session, &settings)?;
    let net = net_in(net, &state, &settings)?;
    let mode = state.mode();
    let mut drawings = Drawings {
        limit: settings.cut.drawn_bytes,
        kept: 0,
        undrawn: None,
    };
    let mut steps = Vec::new();
    let mut seen = |before: NetRef<'_>, step: &nets::cut::Step| {
        let k = steps.len() + 1;
        let (mut drawing, mut refused) = (None, None);
        if drawings.open() {
            match before.draw(Some(step), mode, &settings) {
                Ok(made) => drawing = drawings.keep(k, made),
                Err(error) => refused = serde_json::to_value(error).ok(),
            }
        }
        steps.push(Run {
            step: before.taken(step, mode),
            drawing,
            refused,
        });
    };
    let (options, limits) = (&settings.cut, &settings.limits);
    let (reached, end) = match &net {
        Net::Mll(net) => nets::cut::eliminate(net, options, limits, never, |before, step| {
            seen(NetRef::Mll(before), step);
        })
        .map(|e| (Net::Mll(e.net), e.end)),
        Net::Mell(net) => nets::cut::eliminate(net, options, limits, never, |before, step| {
            seen(NetRef::Mell(before), step);
        })
        .map(|e| (Net::Mell(e.net), e.end)),
    }
    .map_err(|e| Failure::described(&e, &state))?;
    let (end, reason) = ended(&end);
    written(&Ran {
        steps,
        end,
        reason,
        undrawn: drawings.undrawn,
        reached: NetView::of(&reached, mode, &settings),
    })
}
