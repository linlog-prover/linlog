---
paths:
  - "web/**"
  - "modules/web.nix"
---

# linlog-web: the WebAssembly bindings

Loaded when a file of the bindings (`web/`) or their flake module is read
or changed. The plan of the web front end is `plan/web/README.md`; the
client is a repository of its own, which takes this repository's flake
as a pinned input and builds nothing of Rust.

## Layout

`web/src/calls.rs` the calls, plain Rust over JSON text, each reading
its documents and the settings and writing what the page shows;
`lib.rs` the `#[wasm_bindgen]` exports, one line each over `calls`;
`tab.rs` the tab's defaults and the overlay of the page's settings on
them; `failure.rs` the error's JSON a call throws; `build.rs` the stack.
`web/tests/calls.rs` one test per behaviour the page relies on, each
`#[wasm_bindgen_test(unsupported = test)]`, so the same test runs
natively (`cargo test`, the gate) and under Node (`web-wasm-tests`).
`about.toml` and `about.hbs` are cargo-about's configuration and template
for `THIRD-PARTY-NOTICES.txt`.

## What the code relies on

- **Nothing is kept between calls.** The page holds the session
  document and the settings and sends them with every call; a worker
  the page stops (its only cancel: the module reads no clock, and every
  call's stop is `never`) loses nothing. A handle kept in the module
  would save the read of the session per call (linear, and checked by
  `wire::upgrade` as a loaded session always is), and would have to be
  rebuilt after every stop.
- **The settings are read over the tab's** (`tab::read`): the page's
  JSON is laid over `tab::defaults()` key by key, objects merged and
  every other value replaced, then read as `Settings` with its
  `deny_unknown_fields`. So `{}` is the tab's settings, never the
  command's (whose memory bound is a gibibyte), and a misspelt key is
  refused as the command refuses it.
- **Errors are the library's JSON**, `Error`'s written form or
  `describe`'s with the session's formulas; the bindings' own refusals
  (`Failure::json`, `Failure::refused`) take the shape and a `code` the
  library uses (`json`, `unknown_name`). A new call returns `Failure`,
  never a message of its own.
- **`Step` and `Applicable` have no wire form in the library**: `StepForm`
  (`{"position", "rule", "left"}`) and `Offered` (`{"rule", "split"}`)
  are the bindings', the rule by its `Named` name. When the library
  gives them one (at level 1), the bindings take it.
- **A hint is read off a graft**: `hint` closes a copy of the goal and
  turns the grafted inference at that goal into a step (`first_step`):
  the principal's position, and for `⊗` the left premise's members less
  the left subformula, for Mix the left premise's members, matched to
  positions of the goal. `a_hint_is_a_step_that_applies` pins that the
  step applies to the original.
- **The integers at the boundary**: a goal and a position come from
  JavaScript as `u32`, which widens to `usize` on every target; a
  position inside a step's JSON is a `usize` read by serde_json, which
  refuses a number past `usize::MAX` on wasm32 rather than wrapping it
  (`a_position_past_32_bits_is_refused`). Every count the library
  writes is a `u64`, exact in JavaScript to 2⁵³ (`wire`'s rule).
- **The stack**: `build.rs` links the module with an 8 MiB stack
  (`-zstack-size`, read back as `tab::STACK_BYTES`), but the host's own
  stack, which no page can set (about 1 MiB in Node and in a browser),
  runs out first: each wasm frame is a native frame too. Measured in Node
  on the deepest levels, a Mix of pairs: the optimized module reached
  768 levels and not 1 024, the debug one 512 and not 768. Hence
  `tab::RECURSION_DEPTH` 384, half the optimized depth, and
  `a_search_at_the_recursion_limit_stops_there` runs the three paths
  with the largest levels (copies of a `!`, nested `&`, a Mix of pairs)
  at the limit under Node, in the debug build the check uses. A change
  that grows a level's frames shows there first.
- **`unsafe_code` stays forbidden**: the lint does not see the unsafe
  code `#[wasm_bindgen]` generates, so the workspace's `forbid` holds for
  the bindings too.

## The build (`modules/web.nix`)

- The `wasm-bindgen` CLI is nixpkgs' `wasm-bindgen-cli_X_Y_Z` for the
  version of the `wasm-bindgen` crate in `Cargo.lock`, which the
  manifest pins with `=` (and `wasm-bindgen-test` with it, which pins
  the crate exactly): a module the CLI of another version writes does
  not load. `web-bindgen` compares the CLI's `--version` with the lock.
  A bump of the crate needs the nixpkgs in `flake.lock` to have its CLI;
  otherwise the flake fails at evaluation, naming it.
- The package is `cargo build --release` for wasm32, `wasm-bindgen
  --target web`, then `wasm-opt -O3`, with `LICENSE` and
  `THIRD-PARTY-NOTICES.txt` (cargo-about, `--frozen`, the target's
  crates only, from `about.toml`'s accepted licences, which are
  `deny.toml`'s). A crate whose licence is not listed fails the package.
- `web-wasm-tests` runs `cargo test --target wasm32-unknown-unknown`
  with `wasm-bindgen-test-runner` in Node, which only finds
  `#[wasm_bindgen_test]` functions; `web-core-wasm32` runs the library's
  own unit tests of the checker, the size estimate, the memo and the
  memory account on `wasm32-wasip1` under wasmtime, since plain
  `#[test]` functions have no runner on the web target. On WASI a panic
  aborts the run: a failing test there is found by running each alone.
- Locally: `nix develop` has the CLI, `wasm-opt`, Node and wasmtime;
  `CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER=wasm-bindgen-test-runner
  cargo test -p linlog-web --target wasm32-unknown-unknown` and
  `CARGO_TARGET_WASM32_WASIP1_RUNNER="wasmtime run --" cargo test -p
  linlog --lib --target wasm32-wasip1 -- proofs::check` run the two.

## Extension points

- A new call: a function of `calls` over `&str` returning
  `Result<String, Failure>`, one export in `lib.rs`, a row in
  `plan/web/README.md`'s table, and a test in `web/tests/calls.rs` if
  the page relies on a behaviour of it.
- A new default for a tab: a constant of `tab` with the reason, set in
  `tab::defaults()`, and the table of `plan/web/README.md`.
