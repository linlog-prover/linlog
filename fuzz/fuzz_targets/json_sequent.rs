// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The JSON form of a sequent: no input panics, and a sequent that
//! deserializes serializes back to the same value.
#![no_main]

use libfuzzer_sys::fuzz_target;
use linlog::{Forest, Sequent};

fuzz_target!(|data: &[u8]| {
    if let Ok(sequent) = serde_json::from_slice::<Sequent>(data) {
        let json = serde_json::to_string(&sequent).expect("a sequent serializes");
        let again: Sequent = serde_json::from_str(&json).expect("its JSON reads back");
        assert_eq!(again.to_string(), sequent.to_string());
        let _ = sequent.fragment();
        let _ = Forest::new(&sequent);
    }
});
