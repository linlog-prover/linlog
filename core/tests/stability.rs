// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The wire forms of earlier releases and levels, read by this one.
//! `wire/level-1.txt` is the JSON lock (`lock/json.txt`) as version 0.1.0
//! wrote it, followed by the disproofs of every refutation and a proof and
//! a disproof of a goal off the roots, as 0.1.0 wrote them, one `NAME:
//! JSON` per line, `wire/level-2.txt` the forms level 2 added and
//! `wire/level-3.txt` those level 3 added, `wire/level-4.txt` those
//! level 4 added; none is ever rewritten: where
//! the lock moves on, these files stay. Every
//! line of a form that is read back reads through
//! `wire::upgrade`, and writes the same bytes again; an outcome reads back
//! as the proof or the disproof it holds. A release that fails here can no
//! longer read what an earlier one wrote at a level it claims, which its
//! wire level promises.
#![cfg(all(feature = "serialize", feature = "interactive"))]

use linlog::wire::{self, Readable};
use linlog::{Disproof, Interactive, Limits, MellStructure, Proof, ProofStructure, Sequent};

/// Reads one document of the form `T`.
fn read<T: Readable>(name: &str, json: &str) -> T {
    let mut document = serde_json::Deserializer::from_str(json);
    let value = wire::upgrade(&mut document, &Limits::default())
        .unwrap_or_else(|error| panic!("{name}: {error}"));
    document.end().unwrap();
    value
}

/// Reads one document of the form `T` and asserts that it writes the same
/// bytes again.
fn again<T: Readable + serde::Serialize>(name: &str, json: &str) {
    let value: T = read(name, json);
    assert_eq!(serde_json::to_string(&value).unwrap(), json, "{name}");
}

/// Every line of the first release's forms reads, and a form read back
/// writes what it read.
#[test]
fn the_first_release_reads() {
    let lines = include_str!("wire/level-1.txt");
    let mut read_back = 0;
    for line in lines.lines() {
        let (name, json) = line.split_once(": ").unwrap();
        let verdict = |v: &str| json.contains(&format!(r#""verdict":"{v}""#));
        match name.split('-').next().unwrap() {
            "sequent" => again::<Sequent>(name, json),
            "proof" => again::<Proof>(name, json),
            "disproof" => again::<Disproof>(name, json),
            "structure" => again::<ProofStructure>(name, json),
            "session" => again::<Interactive>(name, json),
            "outcome" if verdict("proved") => {
                let _: Proof = read(name, json);
            }
            "outcome" if verdict("unprovable") => {
                let _: Disproof = read(name, json);
            }
            // An unknown outcome is written only.
            "outcome" => continue,
            other => panic!("{name}: no form {other}"),
        }
        read_back += 1;
    }
    assert_eq!(read_back, 37, "every line but the three unknown outcomes");
}

/// The forms level 2 added, as the release that made it wrote them: the
/// classical refutation of a disproof and of an unprovable outcome. Never
/// rewritten, as `wire/level-1.txt` is not.
#[test]
fn the_second_level_reads() {
    let lines = include_str!("wire/level-2.txt");
    for line in lines.lines() {
        let (name, json) = line.split_once(": ").unwrap();
        match name.split('-').next().unwrap() {
            "disproof" => again::<Disproof>(name, json),
            "outcome" => {
                let _: Disproof = read(name, json);
            }
            other => panic!("{name}: no form {other}"),
        }
    }
    assert_eq!(lines.lines().count(), 2);
}

/// The forms level 3 added, as the release that made it wrote them: the
/// refutation by a saturated set of a disproof and of an unprovable
/// outcome. Never rewritten, as `wire/level-1.txt` is not.
#[test]
fn the_third_level_reads() {
    let lines = include_str!("wire/level-3.txt");
    for line in lines.lines() {
        let (name, json) = line.split_once(": ").unwrap();
        match name.split('-').next().unwrap() {
            "disproof" => again::<Disproof>(name, json),
            "outcome" => {
                let disproof: Disproof = read(name, json);
                assert_eq!(disproof.check(), Ok(()), "{name}");
            }
            other => panic!("{name}: no form {other}"),
        }
    }
    assert_eq!(lines.lines().count(), 2);
}

/// The forms level 4 added, as the release that made it wrote them: a
/// proof structure with boxes. Never rewritten, as `wire/level-1.txt` is
/// not.
#[test]
fn the_fourth_level_reads() {
    let lines = include_str!("wire/level-4.txt");
    for line in lines.lines() {
        let (name, json) = line.split_once(": ").unwrap();
        match name.split('-').next().unwrap() {
            "structure" => {
                again::<MellStructure>(name, json);
                let net: MellStructure = read(name, json);
                assert_eq!(net.is_correct(|_| false), Ok(()), "{name}");
            }
            other => panic!("{name}: no form {other}"),
        }
    }
    assert_eq!(lines.lines().count(), 1);
}
