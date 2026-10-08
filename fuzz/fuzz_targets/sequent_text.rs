// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The text syntax of sequents: no input panics, and a sequent that
//! parses prints as text that parses back to it.
#![no_main]

use libfuzzer_sys::fuzz_target;
use linlog::{Forest, Sequent};

fuzz_target!(|text: &str| {
    if let Ok(sequent) = text.parse::<Sequent>() {
        let printed = sequent.to_string();
        let again: Sequent = printed.parse().expect("a printed sequent parses");
        assert_eq!(again.to_string(), printed);
        let _ = sequent.fragment();
        let _ = Forest::new(&sequent);
    }
});
