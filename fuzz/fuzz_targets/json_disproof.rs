// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The JSON form of a disproof: no input panics, and a disproof that
//! deserializes writes back as a document that reads back as itself.
#![no_main]

use libfuzzer_sys::fuzz_target;
use linlog::Disproof;

fuzz_target!(|data: &[u8]| {
    if let Ok(disproof) = serde_json::from_slice::<Disproof>(data) {
        let written = serde_json::to_vec(&disproof).expect("a disproof writes");
        let again: Disproof =
            serde_json::from_slice(&written).expect("a written disproof reads back");
        assert_eq!(again, disproof);
    }
});
