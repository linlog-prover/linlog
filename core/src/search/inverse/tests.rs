// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The inverse method's tests.

use crate::search::generate::{self, IllRules, Rng, Rules};
use crate::search::reference::{self, Answer};
use crate::search::{Engine, Options, Verdict, prove_within};
use crate::{Limits, Mode, Sequent};

/// Decides a sequent with the inverse method within `polls` polls.
fn decide(text: &str, mode: Mode, polls: u64) -> Verdict {
    let sequent: Sequent = text.parse().unwrap();
    let options = Options::default().with_engine(Some(Engine::Inverse));
    let mut left = polls;
    let stop = |_| {
        left = left.saturating_sub(1);
        left == 0
    };
    prove_within(&sequent, mode, &options, &Limits::default(), stop)
        .unwrap()
        .verdict
}

/// A few sequents of each fragment, proved and refuted.
#[test]
fn decides_small_sequents() {
    for (text, provable) in [
        ("|- a, ~a", true),
        ("|- a * b, ~a, ~b", true),
        ("|- a * b, ~a", false),
        ("|- a + b, ~a", true),
        ("|- a & b, ~a", false),
        ("|- a & b, ~a + ~b", true),
        ("|- top, a", true),
        ("|- 1", true),
        ("|- bot", false),
        ("|- bot, 1", true),
        ("|- a par ~a", true),
        ("|- ?~a, !a", true),
        ("|- ?~a, a * a", true),
        // One kept sequent, the empty one, in both holes of one rule.
        ("|- ?a, ?~a, bot * bot", true),
    ] {
        let verdict = decide(text, Mode::CLASSICAL, 1 << 20);
        match (provable, &verdict) {
            (true, Verdict::Proved(_)) | (false, Verdict::Unprovable(_)) => {}
            _ => panic!("{text}: {verdict:?}"),
        }
    }
}

/// The polls of its stop the inverse method gets on a generated sequent:
/// each a sequent taken up or a thousand rule applications.
const POLLS: u64 = 1 << 10;

/// Whether two verdicts contradict each other: a proof against a
/// refutation.
fn contradict(a: &Verdict, b: &Verdict) -> bool {
    matches!(
        (a, b),
        (Verdict::Proved(_), Verdict::Unprovable(_)) | (Verdict::Unprovable(_), Verdict::Proved(_))
    )
}

/// Whether a verdict contradicts the reference's answer.
fn against(verdict: &Verdict, reference: Answer) -> bool {
    match verdict {
        Verdict::Proved(_) => reference == Answer::Unprovable,
        Verdict::Unprovable(_) => reference == Answer::Proved,
        Verdict::Unknown(_) => false,
    }
}

/// Judges one generated sequent: the inverse method against the focused
/// engine within the copies the generator gives and against the
/// reference, and without exponentials decisive and equal to the
/// reference where it decides. Returns whether the inverse method decided.
fn judge(text: &str, mode: Mode, copies: u32, reference: Answer, exponentials: bool) -> bool {
    let inverse = decide(text, mode, POLLS);
    let sequent: Sequent = text.parse().unwrap();
    let engine = if mode.is_intuitionistic() {
        Engine::TwoSided
    } else {
        Engine::Focus
    };
    let options = Options::default()
        .with_engine(Some(engine))
        .with_copies(Some(copies));
    let focused = prove_within(&sequent, mode, &options, &Limits::default(), |_| false)
        .unwrap()
        .verdict;
    assert!(
        !contradict(&inverse, &focused),
        "{text} in {mode} mode: inverse {inverse:?}, focused {focused:?}"
    );
    assert!(
        !against(&inverse, reference),
        "{text} in {mode} mode: inverse {inverse:?}, the reference {reference:?}"
    );
    let decided = !matches!(inverse, Verdict::Unknown(_));
    if !exponentials {
        assert!(decided, "{text} in {mode} mode: {inverse:?}");
    }
    decided
}

/// Generated sequents and their mutants, classical in every rule set
/// (linear and affine, with Mix where the rules use it) and intuitionistic
/// in every rule set, linear and affine: the inverse method never
/// contradicts the focused engine or the reference, decides every sequent
/// without exponentials, and with them decides most.
#[test]
fn agrees_with_the_focused_engine_and_the_reference() {
    let (mut cases, mut decided) = (0, 0);
    for (i, rules) in Rules::ALL.into_iter().enumerate() {
        let mut rng = Rng::new(500 + i as u64);
        for _ in 0..6 {
            let budget = 2 + rng.below(8);
            let generate::Provable {
                mut formulas,
                copies,
            } = generate::provable(&mut rng, rules, 3, budget);
            let mut all = vec![formulas.clone()];
            if generate::mutate(&mut rng, &mut formulas, 3) {
                all.push(formulas);
            }
            for formulas in &all {
                let text = generate::sequent(formulas);
                let exponentials = text
                    .parse::<Sequent>()
                    .unwrap()
                    .fragment()
                    .has_exponentials();
                for affine in [false, true] {
                    let mut mode = if rules.mix {
                        Mode::CLASSICAL.with_mix()
                    } else {
                        Mode::CLASSICAL
                    };
                    if affine {
                        mode = mode.with_affine();
                    }
                    let answer = reference::classical(formulas, mode, copies);
                    cases += 1;
                    decided += usize::from(judge(&text, mode, copies, answer, exponentials));
                }
            }
        }
    }
    for (i, rules) in IllRules::ALL.into_iter().enumerate() {
        let mut rng = Rng::new(600 + i as u64);
        for _ in 0..6 {
            let budget = 2 + rng.below(8);
            let generate::Ill {
                mut hypotheses,
                mut goal,
                copies,
                ..
            } = generate::ill(&mut rng, rules, 3, budget);
            let mut all = vec![(hypotheses.clone(), goal.clone())];
            if generate::mutate_two_sided(&mut rng, &mut hypotheses, &mut goal, 3) {
                all.push((hypotheses, goal));
            }
            for (hypotheses, goal) in &all {
                let text = generate::two_sided(hypotheses, goal);
                let exponentials = text
                    .parse::<Sequent>()
                    .unwrap()
                    .fragment()
                    .has_exponentials();
                for mode in [Mode::INTUITIONISTIC, Mode::INTUITIONISTIC.with_affine()] {
                    let answer = reference::intuitionistic(hypotheses, goal, mode, copies);
                    cases += 1;
                    decided += usize::from(judge(&text, mode, copies, answer, exponentials));
                }
            }
        }
    }
    eprintln!("decided {decided} of {cases}");
    assert!(2 * decided >= cases, "decided {decided} of {cases}");
}

/// A goal within the sequent two of whose members no sequent of a proof
/// holds together is refused, since the search drops every sequent that
/// holds such a pair; a goal whose members can stand together is decided.
#[test]
fn refuses_an_incoherent_goal() {
    let options = Options::default().with_engine(Some(Engine::Inverse));
    let goal_of = |text: &str, members: &[u32], mode: Mode| {
        let forest = crate::occurrences::Forest::new(&text.parse().unwrap()).unwrap();
        let members: Vec<_> = members.iter().map(|&m| crate::Member::new(m)).collect();
        let goal = crate::search::Goal::new(&forest, &members).unwrap();
        crate::search::prove_goal(goal, mode, &options, &Limits::default(), |_| false)
            .map(|outcome| outcome.verdict)
    };
    // The literals a ⊗ separates, and two of the same tensor's factors.
    for (text, members) in [("|- a * ~a", &[1, 2][..]), ("|- a & ~a", &[1, 2])] {
        assert!(matches!(
            goal_of(text, members, Mode::CLASSICAL),
            Err(crate::Error::EngineRefused {
                because: crate::search::NotTaken::Shape,
                ..
            })
        ));
    }
    // `a` of the tensor and `~a` of another root stand together.
    let verdict = goal_of("|- a * b, ~a, ~b", &[1, 3], Mode::CLASSICAL).unwrap();
    assert!(matches!(verdict, Verdict::Proved(_)), "{verdict:?}");
}

/// Choices of `⊕` sides that a `0` ends are work the stop is told of,
/// however many there are: `0 ⊗ ((a₀ ⊕ b₀) ⊗ …)` has 2¹⁶ of them, none a
/// rule.
#[test]
fn choices_a_zero_ends_are_polled() {
    let mut factors = "(a0 + b0)".to_owned();
    for i in 1..16 {
        factors = format!("((a{i} + b{i}) * {factors})");
    }
    let sequent: Sequent = format!("|- 0 * {factors}").parse().unwrap();
    let options = Options::default().with_engine(Some(Engine::Inverse));
    let mut work = 0;
    let stop = |progress: crate::Progress| {
        work += progress.work;
        false
    };
    prove_within(
        &sequent,
        Mode::CLASSICAL,
        &options,
        &Limits::default(),
        stop,
    )
    .unwrap();
    assert!(work >= 1 << 16, "{work} units");
}
