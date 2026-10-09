// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The TPTP reader of ordinary problems: no input panics.
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|text: &str| {
    let _ = linlog::ordinary::read_tptp(text, &linlog::Limits::default());
});
