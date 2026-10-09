// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The JSON form of an interactive session: no input panics, and a
//! complete session gives a proof the checker accepts.
#![no_main]

use libfuzzer_sys::fuzz_target;
use linlog::{Interactive, Limits};

fuzz_target!(|data: &[u8]| {
    if let Ok(session) = serde_json::from_slice::<Interactive>(data) {
        let _ = serde_json::to_string(&session).expect("a session serializes");
        if session.is_complete() {
            let proof = session
                .proof(&Limits::default(), |_| false)
                .expect("a complete session is a proof");
            proof.check(session.mode()).expect("its proof checks");
        }
    }
});
