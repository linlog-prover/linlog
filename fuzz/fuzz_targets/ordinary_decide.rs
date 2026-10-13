// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Ordinary logic decided through its translation, as `prove --logic`
//! decides it: no input panics the translation, the search, the read-back
//! or the checker of LK and LJ, and a proof the search found always reads
//! back to a derivation that checks (an error of kind `defect` says it did
//! not). The first byte picks the logic and the translation, the rest is
//! the ordinary text.
#![no_main]

use libfuzzer_sys::fuzz_target;
use linlog::ordinary::{self, Logic, Sequent, Translation};
use linlog::search::Options;
use linlog::{ErrorKind, Limits};

fuzz_target!(|data: &[u8]| {
    let Some((&bits, text)) = data.split_first() else {
        return;
    };
    let Ok(text) = std::str::from_utf8(text) else {
        return;
    };
    let (logic, translation) = match bits % 7 {
        0 => (Logic::Classical, Translation::Affine),
        1 => (Logic::Intuitionistic, Translation::CallByName),
        2 => (Logic::Intuitionistic, Translation::CallByValue),
        3 => (Logic::Intuitionistic, Translation::ZeroOne),
        4 => (Logic::Minimal, Translation::CallByName),
        5 => (Logic::Minimal, Translation::CallByValue),
        _ => (Logic::Minimal, Translation::ZeroOne),
    };
    let limits = Limits::default()
        .with_occurrences(Some(4096))
        .with_memory_bytes(Some(1 << 26))
        .with_derivation_bytes(Some(1 << 22))
        .with_work(Some(20_000));
    let Ok(sequent) = Sequent::parse_within(text, &limits) else {
        return;
    };
    let options = ordinary::Options::default()
        .with_logic(logic)
        .with_translation(Some(translation));
    let search = Options::default().with_copies(Some(2)).with_jobs(1);
    if let Err(error) = ordinary::decide(&sequent, &options, &search, &limits, |_| false) {
        assert_ne!(error.kind(), ErrorKind::Defect, "{error}");
    }
});
