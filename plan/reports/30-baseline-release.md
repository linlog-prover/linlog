# Step 30: the third baseline and the first release

## The checklist

| item | state | evidence |
|---|---|---|
| Session 30a: the tests pruned and CI cached (session `step-30a`, 2026-10-16) | paused for the author (2026-10-16 22:38): the mutation runs are to resume, see "Where the runs stand" | Per-crate checks (voppnlzq) and the Cachix cache in CI (ptprktol): a change to `cli/` leaves the core crate's six Rust checks' derivations as they were, one to `plan/` every Rust check's. Tests pruned in wtpnsrlo and svrqukoy: four removed, one merged into `detection`, three shrunk; 448 tests to 443; `cargo test --workspace` in the dev profile 111.6 s to 50.3 s, the release tests with debug assertions 24.0 s to 13.4 s (cores 2 to 5). Every code commit gated. The mutation runs and the last `nix flake check` are to resume. |

## Session 30a: the tests pruned and CI cached

**Outcome.** CI pulls from the public Cachix cache `linlog-prover` in
every job and pushes from `main`, and the Rust checks are cut per crate,
each from the sources it needs, so a push reruns only what its change
touches and fetches the rest. The suite lost the tests whose behaviour
another test or a behaviour lock already pins and the part of its one
oversized sweep that bought nothing, and the debug suite takes less than
half its time.

### CI cached, with the checks split per crate

- **The split** (voppnlzq, `modules/workspace.nix`, `modules/checks.nix`).
  `workspace.nix` cuts the workspace into parts (`core`, `core-tests`,
  `cli`, `cli-tests`, `bench`, `bench-tests`: a crate's `src/` with its
  manifest, its `tests/` with what they read) and `argsOf PARTS` gives
  the crane arguments of a source of those parts. Every source keeps the
  four manifests and the lock, since cargo loads every member; a member
  left out gets an empty file for each target its manifest names (the
  `postPatch` `stubMembers`), which nothing builds. `clippy`, `test` and
  `test-debug-assertions` became `clippy-CRATE`, `test-CRATE` and
  `test-debug-assertions-CRATE` for `core`, `cli` and `bench`. The
  packages leave the tests out, so a test's edit reruns neither
  `export`, `rocq`, `bench` nor `ratchet`; `doc` and `doc-private` read
  the core crate and the command, `wasm` and `features` the core crate
  alone, `deny` and `shear` everything Rust.
- **Clippy split too.** A change to the command or the harness reruns
  their clippy alone (which checks the core crate's library again, not
  its tests); a change to the core crate reruns all three, each checking
  the library again, which costs more than the one workspace-wide run
  did (not measured). Changes confined to the command, the harness or
  the tests are frequent enough that it pays.
- **Feature selections.** cargo resolves the dependencies' features for
  the packages a command selects, so the core crate alone builds `serde`,
  `syn`, `resvg` and some thirty more otherwise than the workspace does
  (the command's `serde_json` features, the harness without `png` and
  `pdf`): the first build of the split rebuilt them in every core and
  harness check. One dependency build now covers each crate's selection
  (`workspace.selections`, `depsOnly`), and the checks with debug
  assertions get one of their own (`cargoArtifactsDebugAssertions`), since
  the flags are in every artifact. The core crate is tested with
  `--all-features`, the set `cargo test --workspace` unifies to, so
  `parallel`, `png` and `pdf` stay tested.
- **Checks install no build tree** (`doInstallCargoArtifacts = false`):
  their outputs are empty, so the cache holds the dependency builds
  (185 MiB each) and the packages, not a target directory per check.
- **The proof** (`nix eval` of every check's `drvPath` at svrqukoy, then
  with a line appended to `cli/src/main.rs`, then to `plan/README.md`,
  each undone): the cli change changed `build`, `clippy-cli`, `test-cli`,
  `test-debug-assertions-cli`, `doc`, `doc-private`, `export`, `rocq`,
  `deny`, `shear` and the prose checks, and left `test-core`,
  `test-debug-assertions-core`, `clippy-core`, `wasm`, `features`, the
  harness's three, `bench` and `ratchet` as they were; the plan change
  changed `treefmt` alone. A line appended to `core/tests/parse.rs`
  changed the core crate's three checks, `deny`, `shear` and the prose
  checks only.
- **The cache** (ptprktol). `cachix/cachix-action` v17 (commit 38b0826,
  resolved with `nix flake prefetch`, `flake metadata` fails on a
  repository without a flake) after `install-nix-action` in every job of
  `ci.yml`, `docs.yml` and `comparison.yml`, with `authToken` from the
  secret `CACHIX_AUTH_TOKEN` and `skipPush` unless the ref is `main`. The
  action at that commit reads an empty token as "no push": a pull
  request from a fork logs that pushing is disabled and goes on. It
  pushes in daemon mode, what a job built and nothing it fetched.
- **No `nixConfig`**, as the supervisor recommended: it would make every
  local `nix` command ask about the substituter. README's new section
  "Development" and CLAUDE.md say how a contributor adds the cache
  (`nix run nixpkgs#cachix -- use linlog-prover`, or the substituter and
  key in `nix.conf`). `.claude/rules/ci.md` and `flake.md` record the
  cache, its key, what is pushed and when, and the split.
- **Not verified here**: the workflows on GitHub. actionlint passes; the
  first push to `main` fills the cache, which the supervisor watches.

### The tests pruned

**Inventory.** Every test was timed under nextest (dev profile and release
with debug assertions, cores 2 to 5) and sorted into a kind by four
reading sub-agents (Sonnet), one per group: the focused engine's 73
tests, the rest of the search's 85, the other 214 of the core crate (130
unit, 56 integration, 28 doc tests), the command's 63 and the harness's
13. Of 448 tests, 2 are the behaviour locks and 1 README's examples; the
readers proposed 9 removals, 2 merges and 20 shrinks, and I took the ones
below after reading each. One test dominated: `engines_agree_on_horn_programs`
took 81 s of the dev suite's 88 under nextest, 13.8 s of the release
suite's 14.7.

**Removed or merged, one line each** (wtpnsrlo):

- `core/tests/serialize.rs` `json_format`: removed; the JSON lock's
  `sequent-empty`, `sequent-units`, `sequent-mll`, `sequent-ll` and
  `sequent-repeated` pin the same wire form, every tag it used among
  them; its helper `json` went with it.
- `fragment::tests::kinds_have_classes`: merged into `tests::detection`,
  which asserts the fragment of sequents holding every connective and,
  through the dualised hypotheses, its dual; the reviewer found that `&`
  and `!` occurred there only beside a connective of the same class, so
  `detection` gained `|- A & B` and `|- !A`.
- `occurrences::set::tests::forest_sets`: removed; it tested two getters,
  and `Forest::root_set`, a test-only helper it alone used, went with it.
- `search::tests::stop`: removed; the lock's `outcome-stopped`
  (`prove_within` with a stop that fires at once) pins the verdict,
  `net::tests::stop` and `net::tests::stops` the net engine's stop, and
  `reasons_name_their_setting` the text.
- `cli/tests/cli.rs` `help_names_every_command`: removed; the command
  list is clap's derive of the enum, the lock's error without a command
  lists the four commands, and the lock runs every subcommand of `seq`.

**Shrunk, each with its bound in a comment** (svrqukoy):

- `search::reference::tests::engines_agree_on_horn_programs`: 300 random
  Horn programs to 100, the first 100 of the same seed. A probe of each
  program's time (release build): the programs that reach their goal by
  construction cost 0.17 s in all, the others 16.0 s, about a tenth of a
  second each, half the reference at 3 copies and half the engines up to
  their 5 000 polls, spread over 44 programs with no outlier; so only the
  count could shrink. Every clause shape and all five modes recur many
  times in 100, and the test's ratio of decided programs still holds.
- `core/tests/depth.rs` `DEPTH` and `core/tests/parse.rs`
  `depth_costs_no_stack`'s `N`: 100 000 levels to 50 000. On the stack
  of 256 KiB that is about five bytes a level, below any call frame even
  where inlining lets one frame serve two levels; the readers proposed
  25 000, and the reviewer's point about inlining set 50 000.

**Kept against a reader's proposal**: `cli/tests/cli.rs` `jobs_are_bounded`
(README's `-j 10000` example merges standard error into standard output,
so only this test pins that the note goes to standard error), and
`occurrences::tests::empty` (the table it would have joined needs `parse`,
and it is the one test of `Forest::is_empty`). The focused engine's seven
sweeps and the reference's three smaller ones stay whole: their sizes are
set by the thresholds they assert (that the counts fire, that the bias
compares more than a thousand verdicts), they take under a second each in
the release build, and once the Horn sweep shrank they no longer set the
suite's wall time; halving them would have bought a few seconds of the
dev suite for a weaker sweep. `limits`' two tests that build the same
proof stay apart: they pin different stops.

**The review.** A fresh-context reviewer (Sonnet at high) checked each
claimed overlap against the code: it found the `detection` gap, the lost
stream of the `--jobs` note, the parse-only table, the inlining margin and
four rules notes that named removed items or old sizes; all were fixed
before the commits were gated. Its remaining minor point, that nothing
pins `seq --help`'s list or `--help`'s exit status 0, is clap's own
behaviour and was accepted.

**Numbers** (cores 2 to 5, the same load: the two mutation runs on 6 to
15):

| | `main` | after |
|---|--:|--:|
| tests (nextest, plus 28 doc tests) | 420 + 28 | 415 + 28 |
| `cargo test --workspace`, dev profile | 111.6 s | 50.3 s |
| `cargo test --workspace --release --all-targets`, debug assertions | 24.0 s | 13.4 s |
| nextest, dev profile | 97.5 s | 39.2 s |
| nextest, release with debug assertions | 17.8 s | 8.5 s |
| the gate's core unit tests (`cargo test`, dev), at wtpnsrlo and at svrqukoy | 103.2 s | 37.1 s |

**Coverage was not measured.** Neither `cargo-llvm-cov` nor the
toolchain's `llvm-tools` component is at hand; adding the component
changes `rust-toolchain.toml` and so every check's toolchain, for a
measure the mutation runs below give more sharply on the files that
changed.

### The mutation runs

Not finished: see "Where the runs stand". Each batch runs twice, at
`main` and after the pruning, with the same two passes as
`mutants/run.sh` (the mutants against the tests that exercise the file,
then the survivors against the whole suite), from a jj workspace of its
own; the comparison that counts is the two runs' lists, the committed
lists (`mutants/baseline/`, `area-3.*`, `step-28j/`) a further reference
without line numbers, since the code moved since.

What each pruned test's assertion pins, and the batch that shows a loss:

| test | its assertion lands in | batch |
|---|---|---|
| `json_format` | `serialize/*.rs` | `serialize` |
| `kinds_have_classes` | `Kind::fragment`, `fragment.rs` | `fragment` (this session's) |
| `forest_sets` | `Forest::empty_set`, `occurrences/set.rs` | `occurrences` (this session's) |
| `search::tests::stop` | the stop's verdict, `search/mod.rs`; the net engine's poll, `search/net.rs` | `search`; `net` (this session's) |
| `help_names_every_command` | clap's derive, `cli/src/argument_parsing.rs` | `cli-arguments` (this session's) |
| `engines_agree_on_horn_programs`, shrunk | the Horn engine and the dispatch; the reference it judges with | `horn-mod`, `horn-equation`, `horn-cover`, `horn-reach`, `search`; `reference` (this session's) |
| the depth tests, shrunk | every walk of the public API | `parse`, `serialize`, `ordinary-parse`, `ordinary-derivation` (their filtersets hold `binary_id(linlog::depth)`) |

The depth tests' point, that nothing recurses over a formula, is a stack
bound no mutant can change: cargo-mutants never turns a loop into a
recursion. The batches show only that their value assertions (the
deepest level, the printed length) catch at 50 000 what they caught at
100 000; the bound itself rests on the argument above. The Horn sweep
also judges the focused engine on Horn programs; its files are not
batched (thousands of mutants), and it keeps its own 73 tests.
`cli-settings` (`cli/src/settings.rs`) was batched for `jobs_are_bounded`
before the review restored that test; it stays as a check of the
command's batch driver.

### Where the runs stand

The scratch directory is this session's,
`/tmp/claude-1000/-home-tux-Projects-own-linlog/76df6421-3668-4499-a1b7-27e6e69ef0b6/scratchpad`
(`$S`): the driver `$S/mutate.sh` (the batch table of `mutants/run.sh`
plus `$S/extra-batches.txt`, two jobs, its lists under
`$S/mutants/LABEL/`; a batch whose list exists is skipped), the gate
driver `$S/gate-seq.sh`, the timings under `$S/times/`. The runs were
paused twice for the author (09:16, resumed 10:10; and 10:44); a batch
cut midway reruns from its start.

- **At `main`** (`W=/home/tux/Projects/own/linlog-step30a-mut-before`,
  cores 2 to 8, three jobs). Resume:
  `systemd-run --user --unit step30-mut-before -p MemoryMax=12G -p MemorySwapMax=0 -p OOMPolicy=continue -p LimitCORE=0 -E JOBS=3 --working-directory=$W taskset -c 2-8 nix develop -c bash -c "$S/mutate.sh $W 2-8 before search horn-mod horn-equation horn-cover horn-reach serialize parse ordinary-parse ordinary-derivation fragment occurrences cli-settings cli-arguments net reference >> $S/mut-before.log 2>&1"`
- **After the pruning** (`W=/home/tux/Projects/own/linlog-step30a-mut-after`,
  at svrqukoy, cores 9 to 15): the same with the unit
  `step30-mut-after`, cores `9-15` and the label `after`
  (`$S/mut-after.log`). The cores are those planning freed for the
  night; by day they are 6 to 10 and 11 to 15, two jobs.
- **Where each stood at the pause (22:38)**: at `main`, `search`,
  `horn-mod`, `horn-equation`, `horn-cover`, `horn-reach` and
  `serialize` done, `parse` cut in its first pass; after the pruning,
  `search`, `horn-mod`, `horn-equation` and `horn-cover` done,
  `horn-reach` cut in its first pass. The lists are under
  `$S/mutants/before/` and `$S/mutants/after/`; the driver skips them.
- **So far no batch differs**: `search` 8 survivors at `main` and
  after, `horn-mod` 1 and 1, `horn-equation` 54 and 54, `horn-cover` 17
  and 17, the same mutants (`$S/compare.sh`). Some survive that
  older committed lists lack (against `area-3.4`, the latest of each:
  one in `search`, two in `horn-equation`, none in `horn-mod` or
  `horn-cover`); `main` has them too, so they come from code changed
  since, not the pruning.
  Timeouts are still to be rerun alone, at the end.
- **Done and kept**: the gates of all four code commits (passed, logs
  `$S/gate-voppnlzq.log`, `-ptprktol`, `-wtpnsrlo`, `-svrqukoy`), the
  timings, the drvPath proof, the review, and `nix flake check` at
  svrqukoy (passed, exit 0, 10:10 to 10:25, `$S/flake-2.log`).

### Left open

- **`mutants/step-28i/` is lost.** Step 28i's lists were never committed
  and lived only in the workspace `../linlog-step28i-mutants`, which was
  forgotten on the morning of 2026-10-16 before this session compared
  with it (operation f6649f127aa6). The comparison that counts is the
  run of the same batches at `main` beside the run after the pruning;
  step 28i's survivors that the report of step 28 names in prose (in
  `search`, `few_equal_literals` always true and `DEFAULT_MEMO_LIMIT`
  shifted to zero, which hang) are compared there.
- The first CI run on `main` fills the cache; whether a second run
  fetches the unchanged checks is to be seen on GitHub.
- The focused engine's files were not mutated for the Horn sweep's
  shrink: the sweep's aim is the Horn engine, and the focused engine
  keeps its own 73 tests.
