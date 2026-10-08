---
paths:
  - "mutants/**"
  - "fuzz/**"
  - ".cargo/mutants.toml"
---

# Mutation testing and fuzzing

Loaded when a file of the mutation run or the fuzz targets is read. Both
are evidence no reviewer supplies: a mutant no test notices is a
behaviour no test pins, and a fuzzer feeds the readers the input a
browser or a stranger's file would.

## Mutation testing (`mutants/run.sh`, `.cargo/mutants.toml`)

- The scope is the code soundness rests on: the checker
  (`proofs/check.rs`, `proofs/mod.rs`), the readers (`parse/`,
  `serialize/`, `lltp.rs`, `mist.rs`, `ordinary/parse.rs`), the search's
  front door (`search/mod.rs`), the Horn engine's refutations
  (`search/horn/{mod,equation,cover,reach}.rs`) and the ordinary layer's
  checker (`ordinary/derivation.rs`). A batch is one group of files; the
  table at the top of the script is the scope, and a change of it is a
  commit that says why.
- Two passes per batch: the mutants against the tests that exercise the
  file (a nextest filterset over test paths and binary ids such as
  `linlog::serialize` or `linlog-cli::cli`), then only the survivors
  against the whole workspace (`--iterate` skips what the first pass
  caught). The whole suite for every mutant would cost hours more and
  tell little more.
- `mutants/baseline/BATCH.txt` is what survived both passes, missed or
  timed out; a batch whose file exists is done. A later run (a check
  round) moves the old lists aside, runs again and compares: a file it
  changed must not have more survivors than before.
- The `mutants` cargo profile (opt-level 1, no debug info, the debug
  assertions and overflow checks of `dev`) runs the whole suite in about
  25 s where `dev` takes two minutes, because
  `search::reference::tests::engines_agree_on_horn_programs` alone takes
  114 s unoptimized; an incremental rebuild of the core crate stays a few
  seconds.
- A mutant that panics on an overflow or a debug assertion counts as
  caught: that is the build the tests run in debug, not the release
  build, where the same mutant may wrap silently.
- The `cli` binary's `timeout` test measures wall-clock time and once
  failed under the load of a parallel build; a mutant it alone catches
  may be a false catch.

## Fuzzing (`fuzz/`, `modules/fuzz.nix`)

- `fuzz/` is a workspace of its own on the nightly compiler of the
  `fuzz` devshell, pinned by date in `modules/fuzz.nix`; the default
  shell, every check and `rust-toolchain.toml` stay on stable. Its
  `Cargo.lock` is its own.
- One target per reader of untrusted input: the text syntax
  (`sequent_text`), the ordinary syntax (`ordinary_text`), TPTP (`tptp`),
  LLTP (`lltp`), Mist's `.spec` (`spec`), and the JSON forms of a
  sequent, a proof (read and checked, the first byte the mode), a proof
  structure (the criterion, then sequentialization and the checker) and
  a session. `Outcome` and `Refutation` have no reader: they are written,
  never read.
- Besides "no panic", a target asserts what must hold of what it read:
  a printed sequent parses back, a correct structure sequentializes to a
  proof the checker accepts, a complete session's proof checks.
- `fuzz/seed.sh` seeds the corpora from `bench/problems`, the command's
  own JSON and the LLTP, ILTP and qcover libraries; without seeds a JSON
  target rarely gets past the arena's consistency checks.
- `fuzz/run.sh` builds with debug assertions (so overflow checks are on)
  and runs libFuzzer in fork mode, which goes on past a crash, a timeout
  or an out-of-memory and keeps each under `fuzz/artifacts/TARGET/`; a
  target stops when no new edge came for `STALL` seconds or at `CAP`.
  `target/fuzz/summary.tsv` has a line per finished target, and the
  corpora under `fuzz/corpus/` stay for the next run.
