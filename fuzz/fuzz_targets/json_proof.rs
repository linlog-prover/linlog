// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! A proof to check, as the `check` command reads it: no input panics
//! the reader or the checker, and the checker holds its memory bound. The
//! first byte picks the mode, the rest is the JSON form.
#![no_main]

use libfuzzer_sys::fuzz_target;
use linlog::{Limits, Mode, Proof};

fuzz_target!(|data: &[u8]| {
    let Some((&bits, json)) = data.split_first() else {
        return;
    };
    let mut mode = if bits & 1 != 0 {
        Mode::INTUITIONISTIC
    } else {
        Mode::CLASSICAL
    };
    if bits & 2 != 0 {
        mode = mode.with_affine();
    }
    if bits & 4 != 0 {
        mode = mode.with_mix();
    }
    if let Ok(proof) = serde_json::from_slice::<Proof>(json) {
        let limits = Limits::default().with_memory_bytes(Some(1 << 26));
        let _ = proof.check_within(mode, &limits, |_| false);
    }
});
