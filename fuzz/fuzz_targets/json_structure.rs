// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The JSON form of a proof structure: no input panics, and a structure
//! the criterion calls correct sequentializes to a proof the checker
//! accepts.
#![no_main]

use libfuzzer_sys::fuzz_target;
use linlog::{Limits, Mode, ProofStructure};

fuzz_target!(|data: &[u8]| {
    if let Ok(structure) = serde_json::from_slice::<ProofStructure>(data) {
        if structure.is_correct(|_| false).is_ok() {
            let proof = structure
                .sequentialize(&Limits::default(), |_| false)
                .expect("a correct structure sequentializes");
            let mode = if structure.criterion().mix {
                Mode::CLASSICAL.with_mix()
            } else {
                Mode::CLASSICAL
            };
            proof.check(mode).expect("its proof checks");
        }
    }
});
