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
            let instance = family.instance(size, 0);
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

/// A goal that is no Horn program is refused with the error a forced
/// engine gets.
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
        assert!(matches!(error, Error::NotHorn), "{text}: {error}");
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
        .sequent;
    let forest = Forest::new(&sequent).unwrap();
    let counter = program(&forest);
    let equation = || Equation::new(false, &account);
    let (found, _) = reach::search(&counter, &account, 2, &mut equation(), &mut || false);
    assert_eq!(found, Err(Reason::IndexLimit));
    let (found, _) = reach::search(
        &counter,
        &account,
        reach::MOST_MARKINGS,
        &mut equation(),
        &mut || false,
    );
    let firings = found.unwrap().expect("the counter is provable");
    let built = proof::build(&forest, &counter, &firings, false, &account, 3);
    assert_eq!(built.err(), Some(Reason::IndexLimit));
    let affine = || Equation::new(true, &account);
    let (found, _) = cover::search(&counter, &account, 1, &mut affine(), &mut || false);
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
        &mut || false,
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
        &mut || false,
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
        matches!(outcome.verdict, Verdict::Unknown(Reason::MemoryLimit(4096))),
        "{:?}",
        outcome.verdict
    );
}
