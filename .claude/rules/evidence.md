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
  round) takes a label of its own, `mutants/run.sh --label round-1
  [BATCH…]`, into `mutants/round-1/`, and `mutants/run.sh --compare
  round-1` prints each batch's survivors before and now and the new ones:
  a file the round changed must not have more survivors than before.
  cargo-mutants' own files are under `target/mutation/LABEL/BATCH/`, not
  `target/mutants/`, where the cargo profile of that name builds.
- `exclude_re` leaves out mutants of a `tests` module's own helpers,
  which say nothing of the code under test (42 of the checker's 319).
- The baseline of 2026-10-08/09 took 5.1 hours on cores 6 to 11, three
  mutants at a time; `plan/reports/28-baselines.md` has each batch's.
- The `mutants` cargo profile (opt-level 1, no debug info, the debug
  assertions and overflow checks of `dev`) runs the whole suite in about
  25 s where `dev` takes two minutes, because
  `search::reference::tests::engines_agree_on_horn_programs` alone takes
  114 s unoptimized; an incremental rebuild of the core crate stays a few
  seconds.
- A mutant that panics on an overflow or a debug assertion counts as
  caught: that is the build the tests run in debug, not the release
  build, where the same mutant may wrap silently.
- A flaky test turns a missed mutant into a false catch, never the
  reverse (a timeout counts as surviving). The first run found one:
  `every_style_option` failed on a broken pipe when a call exited before
  the test wrote its input, in about one whole-suite run in four under
  load, and 9 of the 9 catches of the checker's whole-suite pass were
  that alone; the CLI tests now take a closed pipe as no failure. After
  a run, read the failing tests of every caught mutant's log
  (`mutants.out/log/`, and `mutants.out.old/` for the first pass) and
  rerun a batch with a catch by one test that also fails unmutated.

## Fuzzing (`fuzz/`, `modules/fuzz.nix`)

- `fuzz/` is a workspace of its own on the nightly compiler of the
  `fuzz` devshell, pinned by date in `modules/fuzz.nix`; the default
  shell, every check and `rust-toolchain.toml` stay on stable. Its
  `Cargo.lock` is its own.
- One target per reader of untrusted input: the text syntax
  (`sequent_text`), the ordinary syntax (`ordinary_text`), TPTP (`tptp`),
  LLTP (`lltp`), Mist's `.spec` (`spec`), and the JSON forms of a
  sequent, a proof (read and checked, the first byte the mode), a proof
  structure (the criterion, then sequentialization and the checker), a
  session and a disproof (written back and read again; the seeds are
  unprovable outcomes, which read as disproofs). `Outcome` has no reader
  of its own: it reads as a proof or a disproof.
- Besides "no panic", a target asserts what must hold of what it read:
  a printed sequent parses back, a correct structure sequentializes to a
  proof the checker accepts, a complete session's proof checks, a
  disproof writes back as a document that reads back as itself.
- `fuzz/seed.sh` seeds the corpora from `bench/problems`, the command's
  own JSON and the LLTP, ILTP and qcover libraries; without seeds a JSON
  target rarely gets past the arena's consistency checks.
- `fuzz/run.sh` builds with debug assertions (so overflow checks are on)
  and runs libFuzzer in fork mode, which goes on past a crash, a timeout
  or an out-of-memory and keeps each under `fuzz/artifacts/TARGET/`; a
  target stops when no new edge came for `STALL` seconds or at `CAP`.
  `target/fuzz/summary.tsv` has a line per finished target, and the
  corpora under `fuzz/corpus/` stay for the next run.
