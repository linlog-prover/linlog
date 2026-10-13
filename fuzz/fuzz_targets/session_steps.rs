// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! A session driven step by step, as a client drives one: no step panics,
//! a refused one leaves the session as it was, and a session that ends
//! complete gives a proof the checker accepts. The first byte picks the
//! mode, the text up to the first newline is the sequent, and every three
//! bytes after it are a step: what to do, which goal, which formula.
#![no_main]

use libfuzzer_sys::fuzz_target;
use linlog::proofs::interactive::Needs;
use linlog::search::Options;
use linlog::{Interactive, Limits, Mode, Sequent, Step, ViewOptions};

fuzz_target!(|data: &[u8]| {
    let Some((&bits, rest)) = data.split_first() else {
        return;
    };
    let mode = match bits % 4 {
        0 => Mode::CLASSICAL,
        1 => Mode::INTUITIONISTIC,
        2 => Mode::CLASSICAL.with_affine(),
        _ => Mode::CLASSICAL.with_mix(),
    };
    let end = rest.iter().position(|&b| b == b'\n').unwrap_or(rest.len());
    let Ok(text) = std::str::from_utf8(&rest[..end]) else {
        return;
    };
    let limits = Limits::default()
        .with_occurrences(Some(4096))
        .with_memory_bytes(Some(1 << 26))
        .with_work(Some(20_000));
    let Ok(sequent) = Sequent::parse_within(text, &limits) else {
        return;
    };
    let Ok(mut session) = Interactive::within(&sequent, mode, &limits) else {
        return;
    };
    let options = Options::default().with_copies(Some(1)).with_jobs(1);
    for step in rest.get(end + 1..).unwrap_or_default().chunks_exact(3) {
        let goals: Vec<_> = session.goals().collect();
        let Some(&goal) = goals.get(usize::from(step[1]) % goals.len().max(1)) else {
            break;
        };
        let formulas = session.goal(goal).expect("an open goal").len();
        let position = usize::from(step[2]) % formulas.max(1);
        match step[0] % 4 {
            0 | 1 => {
                let Ok(rules) = session.rules(goal, position) else {
                    continue;
                };
                let Some(applicable) = rules.get(usize::from(step[0] / 4) % rules.len().max(1))
                else {
                    continue;
                };
                let mut next = Step::new(position, applicable.rule);
                if applicable.needs == Needs::Split {
                    let left: Vec<usize> = (0..formulas)
                        .filter(|&i| i != position && (step[2] >> (i % 8)) & 1 == 1)
                        .collect();
                    next = next.left(&left);
                }
                let before = session.clone();
                if session.apply(goal, &next).is_err() {
                    assert_eq!(
                        serde_json::to_string(&session).expect("a session serializes"),
                        serde_json::to_string(&before).expect("a session serializes"),
                        "a refused step changes nothing"
                    );
                }
            }
            2 => {
                let _ = session.undo();
            }
            _ => {
                let _ = session.close(goal, &options, &ViewOptions::default(), &limits, |_| false);
            }
        }
    }
    if session.is_complete()
        && let Ok(proof) = session.proof(&limits, |_| false)
    {
        let check = proof.check_within(session.mode(), &limits, |_| false);
        assert!(
            !matches!(check, Err(linlog::CheckError::Invalid(_))),
            "a finished session's proof checks: {check:?}"
        );
    }
});
