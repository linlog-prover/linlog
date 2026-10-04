// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The Horn engine's tests.

#![cfg(feature = "parse")]

use super::{Program, proof, reach};
use crate::Error;
use crate::families::FAMILIES;
use crate::fragment::Mode;
use crate::occurrences::Forest;
use crate::search::memory::Account;
use crate::search::{Engine, Options, Reason, Task, Verdict, prove};
use crate::sequents::Sequent;

/// The options that force the engine.
fn horn() -> Options {
    Options::default().engine(Some(Engine::Horn))
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

/// A goal that is no Horn program, or affine mode, is refused with the
/// error a forced engine gets.
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
    let sequent: Sequent = "!(a -o b), a |- b".parse().unwrap();
    let error = prove(&sequent, Mode::CLASSICAL.affine(), &horn()).unwrap_err();
    assert!(matches!(error, Error::EngineMode { .. }), "{error}");
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
    let (found, _) = reach::search(&counter, &account, 2, &mut || false);
    assert_eq!(found, Err(Reason::IndexLimit));
    let (found, _) = reach::search(&counter, &account, reach::MOST_MARKINGS, &mut || false);
    let firings = found.unwrap().expect("the counter is provable");
    let built = proof::build(&forest, &counter, &firings, &account, 3);
    assert_eq!(built.err(), Some(Reason::IndexLimit));

    // A clause that adds almost 2³² tokens passes the count's bound at
    // its second firing.
    let sequent: Sequent = "!(a -o a * a), a |- b".parse().unwrap();
    let forest = Forest::new(&sequent).unwrap();
    let mut growing = program(&forest);
    let outputs = growing.transitions[0].outputs as usize;
    growing.arcs[outputs].1 = u32::MAX - 1;
    let (found, _) = reach::search(&growing, &account, reach::MOST_MARKINGS, &mut || false);
    assert_eq!(found, Err(Reason::IndexLimit));

    let outcome = prove(&sequent, Mode::CLASSICAL, &horn().memory_limit(Some(4096))).unwrap();
    assert!(
        matches!(outcome.verdict, Verdict::Unknown(Reason::MemoryLimit(4096))),
        "{:?}",
        outcome.verdict
    );
}
