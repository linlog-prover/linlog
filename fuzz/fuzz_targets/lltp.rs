// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The LLTP reader: no input panics.
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|text: &str| {
    let _ = linlog::lltp::read(text, &linlog::Limits::default());
});
