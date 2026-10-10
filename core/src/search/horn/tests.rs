// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The Horn engine's tests.

#![cfg(feature = "parse")]

use super::equation::{Equation, certify};
use super::{Program, cover, proof, reach};
use crate::Error;
use crate::families::FAMILIES;
use crate::fragment::Mode;
use crate::occurrences::Forest;
use crate::search::memory::Account;
use crate::search::{Engine, Options, Reason, Refutation, Task, Verdict, prove};
use crate::sequents::{Atom, Sequent};

/// The options that force the engine.
fn horn() -> Options {
    Options::default().with_engine(Some(Engine::Horn))
}

/// The program of a sequent read in classical mode.
fn program(forest: &Forest) -> Program {
    let task = Task {
        forest,
        goal: forest.roots(),
        fragment: forest.sequent().fragment(),
        mode: Mode::CLASSICAL,
        reading: None,
        roots: true,
    };
    Program::read(&task).expect("a Horn program")
}

/// The engine decides the families that are Horn programs as they are
/// constructed, the counters also read intuitionistically, and every
/// proof passes the checker.
#[test]
fn decides_the_horn_families() {
    for (name, sizes) in [
        ("counter", &[2, 4, 8, 16, 64][..]),
        ("counter-over", &[2, 4, 8, 16, 64]),
        ("partition-yes", &[2, 4, 6]),
        ("partition-no", &[3, 4, 5]),
    ] {
        let family = FAMILIES.iter().find(|f| f.name == name).unwrap();
        for &size in sizes {
            let instance = family.instance(size, 0).unwrap();
            let mut modes = vec![instance.mode];
            if name.starts_with("counter") {
                modes.push(Mode::INTUITIONISTIC);
            }
            for mode in modes {
                let outcome = prove(&instance.sequent, mode, &horn()).unwrap();
                let proved = match outcome.verdict {
                    Verdict::Proved(_) => true,
                    Verdict::Unprovable(_) => false,
                    Verdict::Unknown(reason) => panic!("{}: {reason}", instance.name),
                };
                assert_eq!(
                    proved, instance.provable,
                    "{} in {mode} mode",
                    instance.name
                );
            }
        }
    }
}

/// A goal that is no Horn program, or lies beyond MELL, is refused with
/// the error a forced engine gets.
#[test]
fn refuses_what_it_does_not_decide() {
    for text in [
        "a & b |- a",
        "!(a -o a * a), a |- ?b",
        "|- a, ~a, b, ~b",
        "!(a -o b) |- a -o b",
    ] {
        let sequent: Sequent = text.parse().unwrap();
        let error = prove(&sequent, Mode::CLASSICAL, &horn()).unwrap_err();
        assert!(
            matches!(
                error,
                Error::EngineRefused {
                    engine: Engine::Horn,
                    because: crate::search::NotTaken::Shape
                        | crate::search::NotTaken::Fragment { .. },
                }
            ),
            "{text}: {error}"
        );
    }
}

/// The verdict of a sequent under the engine, as a word, with the
/// refutation's kind.
fn verdict(text: &str, mode: Mode) -> &'static str {
    let sequent: Sequent = text.parse().unwrap();
    match prove(&sequent, mode, &horn()).unwrap().verdict {
        Verdict::Proved(_) => "proved",
        Verdict::Unprovable(d) if matches!(d.refutation(), Refutation::StateEquation(_)) => {
            "state equation"
        }
        Verdict::Unprovable(_) => "unprovable",
        Verdict::Unknown(reason) => panic!("{text} in {mode} mode: {reason}"),
    }
}

/// The refutation the counts give before any search is handed back with
/// the answer, which the front door takes as it is rather than count
/// again.
#[test]
fn counted_refutations_are_handed_back() {
    use crate::search::{Decide, Unbalanced};
    let sequent: Sequent = "c, !c |- a".parse().unwrap();
    let forest = Forest::new(&sequent).unwrap();
    let task = Task {
        forest: &forest,
        goal: forest.roots(),
        fragment: sequent.fragment(),
        mode: Mode::CLASSICAL,
        reading: None,
        roots: true,
    };
    let limits = crate::Limits::default();
    let answer = super::Horn
        .decide(
            &task,
            &horn(),
            &limits,
            &Account::new(None),
            &crate::search::Work::new(None),
            &mut |_| false,
        )
        .unwrap();
    assert!(matches!(answer.result, Ok(None)));
    assert!(
        matches!(
            answer.refutation,
            Some(Refutation::Unbalanced(Unbalanced { .. }))
        ),
        "{:?}",
        answer.refutation
    );
}

/// Nets whose markings grow without end are refuted by the state
/// equation where it has no solution, the transitions that can never fire
/// left out, or by a backward search whose markings run out; in affine
/// mode the backward coverability search decides, refuting what no
/// firing covers and proving what one does with the tokens and the
/// clauses left over weakened.
#[test]
fn decides_unbounded_nets() {
    for (text, linear, affine) in [
        // Nothing makes `c`.
        (
            "!d, !((c * d * d) -o 1) |- c * c",
            "state equation",
            "unprovable",
        ),
        // The tokens of `A` only grow, and the goal has fewer.
        (
            "!(A -o A * A), !(B * B -o C), A, B |- C",
            "state equation",
            "unprovable",
        ),
        // The clause that makes `b` needs a `c`, which is never marked:
        // without it, nothing makes `b`.
        (
            "!(a -o a * a), !(a * c -o b * c), a |- b",
            "state equation",
            "unprovable",
        ),
        // The equation has a solution, `a` and the clause used once each
        // fired once, but the clause needs two `a`; backward, the goal has
        // no predecessor.
        ("!(1 -o 1), !a, (a * a -o a) |- 1", "unprovable", "proved"),
        // Covered with an `a`, a `c` and a clause used once to spare.
        (
            "!(a -o a * a), a -o c, c -o b, !(a * a * a -o b), a |- b * b",
            "",
            "proved",
        ),
    ] {
        if !linear.is_empty() {
            assert_eq!(verdict(text, Mode::CLASSICAL), linear, "{text}");
            assert_eq!(verdict(text, Mode::INTUITIONISTIC), linear, "{text}");
        }
        // Which of the backward search and the equation refutes first is a
        // matter of their shares of the work.
        for mode in [
            Mode::CLASSICAL.with_affine(),
            Mode::INTUITIONISTIC.with_affine(),
        ] {
            let found = match verdict(text, mode) {
                "state equation" => "unprovable",
                found => found,
            };
            assert_eq!(found, affine, "{text} in {mode} mode");
        }
    }
}

/// A refutation by the state equation names every place's weight, the
/// tickets of clauses used once by clause, and the clauses that can never
/// fire, which the inequalities leave out.
#[test]
fn the_certificate_names_its_clauses() {
    let certificate = |text: &str| {
        let sequent: Sequent = text.parse().unwrap();
        let outcome = prove(&sequent, Mode::CLASSICAL, &horn()).unwrap();
        let Verdict::Unprovable(disproof) = outcome.verdict else {
            panic!("{text}: {:?}", outcome.verdict);
        };
        let Refutation::StateEquation(certificate) = disproof.refutation().clone() else {
            panic!("{text}: {disproof}");
        };
        (Forest::new(&sequent).unwrap(), certificate)
    };
    // The atoms `a`, `c`, `b`; the clause that needs a `c` is under the
    // second `!`.
    let (forest, dead) = certificate("!(a -o a * a), !(a * c -o b * c), a |- b");
    let needs_c = forest.children(forest.roots()[1]).next().unwrap();
    assert_eq!(dead.atoms, [(Atom::new(0), -1), (Atom::new(2), 1)]);
    assert_eq!((dead.clauses, dead.dropped), (vec![], vec![needs_c]));
    // One clause used once makes one `b`, and the goal asks two: its ticket
    // weighs as a `b` does.
    let (forest, once) = certificate("!(a -o a * a), !(b -o b), (a * a -o b), a |- b * b");
    assert_eq!(once.atoms, [(Atom::new(1), 1)]);
    assert_eq!(
        (once.clauses, once.dropped),
        (vec![(forest.roots()[2], 1)], vec![])
    );
}

/// The check of the weights is exact: weights whose products wrap in 64
/// bits, so that wrapping arithmetic would find the transition lowering
/// the weighted count, are no refutation.
#[test]
fn checks_the_weights_exactly() {
    let sequent: Sequent = "!(a -o b * b), a |- b * b * b".parse().unwrap();
    let forest = Forest::new(&sequent).unwrap();
    let net = program(&forest);
    let (a, b) = (net.place_of[0], net.place_of[1]);
    // `b` weighs `i64::MAX`: the transition raises the count by
    // `2 · i64::MAX − 1`, which wraps to `−3` in 64 bits, while the goal's
    // `3 · i64::MAX − 1` wraps to a number above zero.
    assert!(!certify(&net, false, &[(a, 1), (b, i64::MAX)]));
    // Not raised, but not asked raised either.
    assert!(!certify(&net, false, &[(a, 3), (b, 1)]));
    for affine in [false, true] {
        assert!(certify(&net, affine, &[(a, 2), (b, 1)]));
    }
}

/// Every number the engine's answer rests on refuses at its limit
/// instead of wrapping: the markings kept, a token count, the nodes of
/// the proof, and the memory bound.
#[test]
fn refuses_at_its_limits() {
    let account = Account::new(None);
    let sequent: Sequent = FAMILIES
        .iter()
        .find(|f| f.name == "counter")
        .unwrap()
        .instance(8, 0)
        .unwrap()
        .sequent;
    let forest = Forest::new(&sequent).unwrap();
    let counter = program(&forest);
    let equation = || Equation::new(false, &account);
    let (found, _) = reach::search(&counter, &account, 2, &mut equation(), &mut |_| false);
    assert_eq!(found, Err(Reason::IndexLimit));
    let (found, _) = reach::search(
        &counter,
        &account,
        reach::MOST_MARKINGS,
        &mut equation(),
        &mut |_| false,
    );
    let firings = found.unwrap().expect("the counter is provable");
    let built = proof::build(&forest, &counter, &firings, false, &account, 3);
    assert_eq!(built.err(), Some(Reason::IndexLimit));
    let affine = || Equation::new(true, &account);
    let (found, _) = cover::search(&counter, &account, 1, &mut affine(), &mut |_| false);
    assert_eq!(found, Err(Reason::IndexLimit));

    // A clause that adds almost 2³² tokens passes the count's bound at
    // its second firing; the state equation, which would refute the goal,
    // has no room here.
    let sequent: Sequent = "!(a -o a * a), a |- b".parse().unwrap();
    let forest = Forest::new(&sequent).unwrap();
    let mut growing = program(&forest);
    let outputs = growing.transitions[0].outputs as usize;
    growing.arcs[outputs].1 = u32::MAX - 1;
    let no_room = Account::new(Some(0));
    let (found, _) = reach::search(
        &growing,
        &account,
        reach::MOST_MARKINGS,
        &mut Equation::new(false, &no_room),
        &mut |_| false,
    );
    assert_eq!(found, Err(Reason::IndexLimit));

    // Backward, a clause that takes almost 2³² tokens and gives one back
    // asks for more than 2³² before its own output; the state equation has
    // a solution, since `c` makes as many `a` as asked.
    let sequent: Sequent = "!(a * a -o a), !(a -o b), !(c -o c * a), c |- b"
        .parse()
        .unwrap();
    let forest = Forest::new(&sequent).unwrap();
    let mut taking = program(&forest);
    for t in 0..2 {
        let inputs = taking.transitions[t].inputs as usize;
        taking.arcs[inputs].1 = u32::MAX - 1;
    }
    let (found, _) = cover::search(
        &taking,
        &account,
        reach::MOST_MARKINGS,
        &mut affine(),
        &mut |_| false,
    );
    assert_eq!(found, Err(Reason::IndexLimit));

    // Nothing refutes this one but its markings, which grow without end in
    // both directions: the tokens `a` stay odd, so the goal is never
    // reached, but the state equation has a rational solution, and the
    // tokens `d` grow and shrink.
    let sequent: Sequent =
        "!(a -o a * a * a), !(a * a -o b), !(d -o d * d), !(d * d -o d), a, d |- b * d"
            .parse()
            .unwrap();
    let limits = crate::Limits::default().with_memory_bytes(Some(4096));
    let outcome =
        crate::search::prove_within(&sequent, Mode::CLASSICAL, &horn(), &limits, |_| false)
            .unwrap();
    assert!(
        matches!(
            outcome.verdict,
            Verdict::Unknown(Reason::MemoryLimit { limit_bytes: 4096 })
        ),
        "{:?}",
        outcome.verdict
    );
}

/// A sequent's verdict under the engine within a memory bound.
fn bounded(sequent: &Sequent, mode: Mode, bytes: u64) -> Verdict {
    let limits = crate::Limits::default().with_memory_bytes(Some(bytes));
    crate::search::prove_within(sequent, mode, &horn(), &limits, |_| false)
        .unwrap()
        .verdict
}

/// The backward search tries every transition on every element it takes.
/// Elements leave the queue by their tokens, not by their index, so an
/// element kept later may be taken earlier: a transition already tried on
/// that one is still to be tried on this one, here the step that covers
/// the goal.
#[test]
fn tries_every_transition_on_every_element() {
    let text = "(c * b -o c * c * b), (1 -o b * c * b), !(c * b * c -o c * a * a) |- a";
    for mode in [
        Mode::CLASSICAL.with_affine(),
        Mode::INTUITIONISTIC.with_affine(),
    ] {
        assert_eq!(verdict(text, mode), "proved", "{mode}");
    }
}

/// The backward search caps a place at its initial count unless some
/// clause gives it more tokens than it takes, which no marking on the way
/// from the initial one passes: a place only taken from, or given back as
/// many as taken, is capped, the place of a class of clauses used once
/// among them, and one that a clause raises is not.
#[test]
fn caps_the_places_no_clause_raises() {
    // The places `a`, `d`, `g`, `b`, `c` and the class of the clause used
    // once. The clause under `!` gives `d` back after taking an `a`, a
    // place before it; the clause used once raises `c` and takes `b` and
    // its ticket, a place before it and one after.
    let sequent: Sequent = "!(a -o a * a), !(a * d -o d * g), a, b, b, b, d, (a * b -o c) |- c"
        .parse()
        .unwrap();
    let forest = Forest::new(&sequent).unwrap();
    let open = u32::MAX;
    assert_eq!(program(&forest).caps(), [open, 1, open, 3, open, 1]);
}

/// Once the forward search has kept its share of markings without
/// deciding, the backward search runs beside it and may find the firing
/// sequence first. Forward, every marking of fewer than 200 tokens comes
/// before the clause used once can fire; backward, its 200 tokens are
/// walked down to one, with its ticket at its cap, the initial count.
#[test]
fn the_backward_search_finds_a_long_firing_sequence() {
    let tokens = vec!["a"; 200].join(" * ");
    let sequent: Sequent = format!("!(a -o a * a), !(a -o b), !(b -o a), ({tokens} -o c), a |- c")
        .parse()
        .unwrap();
    let outcome = prove(&sequent, Mode::CLASSICAL, &horn()).unwrap();
    assert!(
        matches!(outcome.verdict, Verdict::Proved(_)),
        "{:?}",
        outcome.verdict
    );
    // More markings than the forward search keeps alone.
    assert!(
        outcome.statistics.memo_entries > 1 << 14,
        "{:?}",
        outcome.statistics
    );
}

/// The distance to the target after a firing merges the clause's inputs
/// and outputs by place, so a place both touch is counted once even when
/// an output on a place before it comes first: the firing that reaches the
/// target is seen to.
#[test]
fn sees_the_firing_that_reaches_the_target() {
    for text in [
        "!(a -o b * a), !(c * c -o b * c), c, c |- b * c",
        "(b -o a), !(a * c -o b * a), b, c |- b * a",
    ] {
        assert_eq!(verdict(text, Mode::CLASSICAL), "proved", "{text}");
    }
}

/// A marking kept with a count past a byte reads back as written: 130
/// tokens on one place, moved one by one to another.
#[test]
fn keeps_counts_past_a_byte() {
    let given = vec!["a"; 130].join(", ");
    let goal = vec!["b"; 130].join(" * ");
    let text = format!("!(a -o b), {given} |- {goal}");
    assert_eq!(verdict(&text, Mode::CLASSICAL), "proved");
}

/// Out of room, a search takes the simplex's tableau back, and the
/// forward search the backward one's markings, and tries once more: the
/// simplex never costs it a decision. At these bounds the tableau is held
/// when the search's buffers next grow, and without it the search has
/// room to prove the goal.
#[test]
fn the_simplex_gives_its_memory_back() {
    let sequent: Sequent = "!(c * a -o b), !(1 -o b * d), !(b * b * d -o a), d |- a"
        .parse()
        .unwrap();
    let verdict = bounded(&sequent, Mode::CLASSICAL.with_affine(), 1500);
    assert!(matches!(verdict, Verdict::Proved(_)), "{verdict:?}");
    let partition = FAMILIES
        .iter()
        .find(|f| f.name == "partition-yes")
        .unwrap()
        .instance(4, 0)
        .unwrap()
        .sequent;
    let verdict = bounded(&partition, Mode::CLASSICAL, 8400);
    assert!(matches!(verdict, Verdict::Proved(_)), "{verdict:?}");
}

/// In intuitionistic mode a marking of several atoms is read whole on the
/// left of `⊢`, each literal lying within the marking's extent, here
/// where the marking is the forest's first occurrence.
#[test]
fn reads_a_marking_of_several_atoms_intuitionistically() {
    let text = "a * c, !(a -o a * c * c) |- 1";
    assert_eq!(verdict(text, Mode::INTUITIONISTIC), "state equation");
    assert_eq!(verdict(text, Mode::INTUITIONISTIC.with_affine()), "proved");
}

/// In intuitionistic mode the reading must put on the left of `⊢` exactly
/// the occurrences that hold a head or lie in one, and not what follows
/// the head within its member. The reading is given by hand, since only
/// so can a clause be written with its head before a body literal:
/// `a, a ⊸ b ⊢ b` with the clause `b⊥ ⊗ a`.
#[cfg(feature = "interactive")]
#[test]
fn the_reading_puts_exactly_the_heads_on_the_left() {
    use crate::occurrences::{Reading, Side};
    use Side::{Input, Output};
    let sequent: Sequent = "|- ~b * a, ~a, b".parse().unwrap();
    let forest = Forest::new(&sequent).unwrap();
    let reads = |sides: &[Side]| {
        let reading = Reading::of_parts(&forest, sides, forest.roots()[2]);
        let task = Task {
            forest: &forest,
            goal: forest.roots(),
            fragment: sequent.fragment(),
            mode: Mode::INTUITIONISTIC,
            reading: Some(&reading),
            roots: true,
        };
        Program::read(&task).is_some()
    };
    // The clause, its head, its body literal, the marking and the goal.
    assert!(reads(&[Input, Input, Output, Input, Output]));
    assert!(!reads(&[Input, Input, Input, Input, Output]));
}

/// The simplex polls the stop before every pivot: a stop that fires at
/// once ends it with the reason the caller gets, where it would refute.
#[test]
fn the_simplex_polls_the_stop() {
    let sequent: Sequent = "!(A -o A * A), !(B * B -o C), A, B |- C".parse().unwrap();
    let forest = Forest::new(&sequent).unwrap();
    let net = program(&forest);
    let account = Account::new(None);
    let mut stopped = Equation::new(false, &account);
    assert_eq!(
        stopped.run(&net, u64::MAX, &mut |_| true),
        Err(Reason::Stopped)
    );
    let mut equation = Equation::new(false, &account);
    assert_eq!(equation.run(&net, u64::MAX, &mut |_| false), Ok(true));
}

/// The searches charge every buffer they grow, at its capacity: each
/// decides these nets exactly from the least bound below and refuses one
/// byte under it. The bounds are the account's, so they move when what is
/// charged does; the simplex has no room here. Forward, the markings of
/// the partition run out; backward, its elements; and the third net is
/// refuted by the backward search beside the forward one, which keeps
/// 2¹⁴ markings first.
#[test]
fn charges_what_it_grows() {
    let partition: Sequent = FAMILIES
        .iter()
        .find(|f| f.name == "partition-no")
        .unwrap()
        .instance(4, 0)
        .unwrap()
        .sequent;
    let both: Sequent = "!(c * a -o b), !(1 -o b * d), !(b * b * d -o a), d |- a"
        .parse()
        .unwrap();
    let no_room = Account::new(Some(0));
    for (sequent, affine, least) in [
        (&partition, false, 7536),
        (&partition, true, 116_204),
        (&both, false, 659_984),
    ] {
        let forest = Forest::new(sequent).unwrap();
        let net = program(&forest);
        let within = |bytes| {
            let account = Account::new(Some(bytes));
            let mut equation = Equation::new(affine, &no_room);
            let search = if affine { cover::search } else { reach::search };
            search(
                &net,
                &account,
                reach::MOST_MARKINGS,
                &mut equation,
                &mut |_| false,
            )
            .0
        };
        assert_eq!(within(least), Ok(None), "{sequent} within {least}");
        assert_eq!(
            within(least - 1),
            Err(Reason::MemoryLimit {
                limit_bytes: least - 1
            }),
            "{sequent}"
        );
    }
}

/// The searches' counters on these runs, the simplex given no room so
/// that the search alone decides: the markings taken (`nodes`), those
/// dropped as kept already, covered or above a cap (`memo_hits`), those
/// kept (`memo_entries`), and the units of work polled. The numbers are
/// the search's own, a function of the net: they change only when the
/// search does. Forward and backward on the partitions, and the net the
/// backward reachability search refutes beside the forward one.
#[test]
fn counts_its_work() {
    let family = |name: &str| -> Sequent {
        FAMILIES
            .iter()
            .find(|f| f.name == name)
            .unwrap()
            .instance(4, 0)
            .unwrap()
            .sequent
    };
    let (yes, no) = (family("partition-yes"), family("partition-no"));
    let both: Sequent = "!(c * a -o b), !(1 -o b * d), !(b * b * d -o a), d |- a"
        .parse()
        .unwrap();
    let no_room = Account::new(Some(0));
    for (sequent, affine, proved, counters) in [
        (&yes, false, true, [75, 27, 48, 74]),
        (&no, false, false, [217, 136, 81, 217]),
        (&yes, true, true, [126, 44, 82, 145]),
        (&no, true, false, [2289, 1848, 442, 2731]),
        (&both, false, false, [32_233, 15_846, 16_387, 32_233]),
    ] {
        let forest = Forest::new(sequent).unwrap();
        let net = program(&forest);
        let account = Account::new(None);
        let mut equation = Equation::new(affine, &no_room);
        let mut work = 0;
        let mut stop = |units| {
            work += units;
            false
        };
        let search = if affine { cover::search } else { reach::search };
        let (found, statistics) = search(
            &net,
            &account,
            reach::MOST_MARKINGS,
            &mut equation,
            &mut stop,
        );
        assert_eq!(found.map(|f| f.is_some()), Ok(proved), "{sequent}");
        let counted = [
            statistics.nodes,
            statistics.memo_hits,
            statistics.memo_entries,
            work,
        ];
        assert_eq!(counted, counters, "{sequent} affine {affine}");
    }
}

/// The simplex's weights for nets whose state equation has no solution,
/// a place and its weight each, the places numbered as the goal's atoms
/// are met, and the units it polls, one before each pivot and one at the
/// optimum. In affine mode it enters the surplus columns, and no weight is
/// negative. The tokens of `z` grow without end, so only the equation
/// refutes these nets.
#[test]
fn the_simplex_finds_these_weights() {
    for (text, linear, affine) in [
        (
            "!(A -o A * A), !(B * B -o C), A, B |- C",
            (vec![(0, -2), (1, 1), (2, 2)], 2),
            (vec![(0, 0), (1, 1), (2, 2)], 3),
        ),
        (
            "!(a * a -o b), !(b -o a * a), !(b * b -o c), !(c -o b * b), !(z -o z * z), z, a |- c",
            (vec![(0, 1), (1, 2), (2, 4), (3, -4)], 3),
            (vec![(0, 1), (1, 2), (2, 4), (3, 0)], 4),
        ),
        (
            "!(a -o b * b), !(b * b * b -o c), !(z -o z * z), z, a, a |- c * c",
            (vec![(0, 2), (1, 1), (2, 3), (3, -3)], 3),
            (vec![(0, 2), (1, 1), (2, 3), (3, 0)], 4),
        ),
        (
            "!(a * b -o c), !(c -o a * a), !(c -o b * b), a, b, !(z -o z * z), z |- c * c",
            (vec![(0, 1), (1, 1), (2, 2), (3, -2)], 3),
            (vec![(0, 1), (1, 1), (2, 2), (3, 0)], 4),
        ),
    ] {
        let sequent: Sequent = text.parse().unwrap();
        let forest = Forest::new(&sequent).unwrap();
        let net = program(&forest);
        for (affine, (weights, polls)) in [(false, linear), (true, affine)] {
            let account = Account::new(None);
            let mut equation = Equation::new(affine, &account);
            let mut polled = 0;
            let refuted = equation.run(&net, u64::MAX, &mut |units| {
                polled += units;
                false
            });
            assert_eq!(refuted, Ok(true), "{text} affine {affine}");
            assert_eq!(
                (equation.certificate(), polled),
                (Some(&weights[..]), polls),
                "{text} affine {affine}"
            );
        }
    }
}

/// A search that runs out of room gives its memory back, and the simplex
/// then runs to its end with the rest of the bound: at this bound the
/// search cannot keep its first marking, while the tableau fits.
#[test]
fn the_simplex_refutes_after_the_search() {
    let sequent: Sequent = "!(A -o A * A), !(B * B -o C), A, B |- C".parse().unwrap();
    let verdict = bounded(&sequent, Mode::CLASSICAL, 300);
    assert!(
        matches!(&verdict, Verdict::Unprovable(d) if matches!(d.refutation(), Refutation::StateEquation(_))),
        "{verdict:?}"
    );
}
