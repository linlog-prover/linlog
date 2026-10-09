// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The reader of coverability problems in Mist's `.spec` format: no input
//! panics, and none builds more than the bound it reads under.
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|text: &str| {
    let limits = linlog::Limits::default().with_occurrences(Some(1 << 20));
    let _ = linlog::mist::read(text, &limits);
});
