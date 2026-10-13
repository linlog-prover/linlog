// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! A proof file's views, as `check` draws one: no input panics the
//! reader, the checker, the size estimate, the derivation or its text
//! tree, and each holds its bounds. The first byte picks the mode and the
//! view, the rest is the JSON form.
#![no_main]

use libfuzzer_sys::fuzz_target;
use linlog::proofs::{Compact, Sides, TextOptions};
use linlog::{Limits, Mode, Proof, ViewOptions};

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
    let view = ViewOptions::default()
        .with_compact(match (bits >> 3) % 3 {
            0 => Compact::Auto,
            1 => Compact::Always,
            _ => Compact::Never,
        })
        .with_sides(if bits & 32 != 0 {
            Sides::Two
        } else {
            Sides::One
        });
    let limits = Limits::default()
        .with_memory_bytes(Some(1 << 26))
        .with_derivation_bytes(Some(1 << 22))
        .with_work(Some(1 << 22));
    let Ok(proof) = serde_json::from_slice::<Proof>(json) else {
        return;
    };
    if proof.check_within(mode, &limits, |_| false).is_err() {
        return;
    }
    let _ = proof.derivation_size(&view, &limits, |_| false);
    if let Ok(derivation) = proof.derivation_within(&view, &limits, |_| false) {
        let mut text = String::new();
        let _ = derivation.write_text(&TextOptions::default(), &mut text, |_| false);
    }
});
