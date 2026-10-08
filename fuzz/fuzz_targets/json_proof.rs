// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! A proof to check, as the `check` command reads it: no input panics
//! the reader or the checker, and the checker holds its memory bound. The
//! first byte picks the mode, the rest is the JSON form.
#![no_main]

use libfuzzer_sys::fuzz_target;
use linlog::{Mode, Proof};

fuzz_target!(|data: &[u8]| {
    let Some((&bits, json)) = data.split_first() else {
        return;
    };
    let mode = Mode {
        intuitionistic: bits & 1 != 0,
        affine: bits & 2 != 0,
        mix: bits & 4 != 0,
    };
    if let Ok(proof) = serde_json::from_slice::<Proof>(json) {
        let _ = proof.check_within(mode, Some(1 << 26));
    }
});
