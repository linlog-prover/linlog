// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Gives the WebAssembly module the stack the search's recursion needs:
//! rustc links one of 1 MiB for `wasm32-unknown-unknown`.

/// The stack of the module, in bytes, which the crate reads back as
/// `STACK_BYTES`.
const STACK_BYTES: usize = 8 << 20;

fn main() {
    println!("cargo::rustc-env=LINLOG_WEB_STACK_BYTES={STACK_BYTES}");
    if std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() == Ok("wasm32") {
        println!("cargo::rustc-link-arg=-zstack-size={STACK_BYTES}");
    }
}
