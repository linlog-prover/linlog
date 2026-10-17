# Step 30: the third baseline and the first release

## The checklist

| item | state | evidence |
|---|---|---|
| Session 30a: the tests pruned and CI cached (session `step-30a`, 2026-10-16) | done (2026-10-16 to 10-17) | Per-crate checks (voppnlzq) and the Cachix cache in CI (ptprktol): a change to `cli/` leaves the core crate's six Rust checks' derivations as they were, one to `plan/` every Rust check's. Tests pruned in wtpnsrlo and svrqukoy: four removed, one merged into `detection`, three shrunk; 448 tests to 443; `cargo test --workspace` in the dev profile 111.6 s to 50.3 s, the release tests with debug assertions 24.0 s to 13.4 s (cores 2 to 5). Every code commit gated. Mutation runs of every file whose tests changed (17 batches, 2 831 mutants at `main`, 2 830 after; `check` and `focus-28j` for step 28j's lists), at `main` and after: 235 and 236 surviving, the one more a timeout race that is caught alone on both sides; every timeout rerun alone; no new survivor. `mutants/step-28i/` lost (forgotten with its workspace); `mutants/step-28j/check.txt` short by ten false catches of a README test that failed unmutated at its revision. `nix flake check` passed at the session's last commit. |

## Session 30a: the tests pruned and CI cached

**Outcome.** CI pulls from the public Cachix cache `linlog-prover` in
every job and pushes from `main`, and the Rust checks are cut per crate,
each from the sources it needs, so a push reruns only what its change
touches and fetches the rest. The suite lost the tests whose behaviour
another test or a behaviour lock already pins and the part of its one
oversized sweep that bought nothing, and the debug suite takes less than
half its time; the mutation runs of every file whose tests changed, at
`main` and after, find no new survivor.

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

**No new survivor.** Every batch ran twice, at `main` (`../linlog-step30a-mut-before`)
and after the pruning (`../linlog-step30a-mut-after`, at svrqukoy), with
the two passes of `mutants/run.sh` (the mutants against the tests that
exercise the files, then the survivors against the whole suite), and
each mutant left a timeout was rerun alone. The two runs' lists are
equal batch for batch once the reruns are read; the committed lists are a
further reference, compared without line numbers since the code moved.

What each pruned test's assertion pins, and the batch that shows a loss:

| test | its assertion lands in | batch |
|---|---|---|
| `json_format` | `serialize/*.rs` | `serialize` |
| `kinds_have_classes` | `Kind::fragment`, `fragment.rs` | `fragment` (this session's) |
| `forest_sets` | `Forest::empty_set`, `occurrences/set.rs` | `occurrences` (this session's) |
| `search::tests::stop` | the stop's verdict, `search/mod.rs`; the net engine's poll, `search/net.rs` | `search`; `net` (this session's) |
| `help_names_every_command` | clap's derive, `cli/src/argument_parsing.rs` | `cli-arguments` (this session's) |
| `engines_agree_on_horn_programs`, shrunk | the Horn engine and the dispatch; the reference it judges with; the focused engine and the checker it also runs | `horn-mod`, `horn-equation`, `horn-cover`, `horn-reach`, `search`; `reference` (this session's); `focus-28j` (step 28j's changed lines of the focused engine, rebuilt from its diff, 327 mutants where step 28j had 329), `check` |
| the depth tests, shrunk | every walk of the public API | `parse`, `serialize`, `ordinary-parse`, `ordinary-derivation` (their filtersets hold `binary_id(linlog::depth)`) |

`cli-settings` (`cli/src/settings.rs`) was batched for `jobs_are_bounded`
before the review restored that test; it stays as a check of the
command's batch driver. The depth tests' point, that nothing recurses
over a formula, is a stack bound no mutant can change (cargo-mutants never
turns a loop into a recursion); the batches show that their value
assertions catch at 50 000 what they caught at 100 000.

| batch | mutants | unviable | surviving at `main` | after | timeouts after | minutes at `main` / after |
|---|--:|--:|--:|--:|--:|--:|
| `search` | 163 | 32 | 8 | 8 | 2 | 38 / 27 |
| `horn-mod` | 162 | 21 | 1 | 1 | 0 | 19 / 22 |
| `horn-equation` | 264 | 1 | 54 | 54 | 3 | 40 / 39 |
| `horn-cover` | 172 | 2 | 17 | 17 | 4 | 21 / 19 |
| `horn-reach` | 269 | 3 | 18 | 18 | 7 | 33 / 35 |
| `serialize` | 106 | 56 | 6 | 6 | 0 | 10 / 11 |
| `parse` | 145 | 16 | 4 | 4 | 0 | 18 / 18 |
| `ordinary-parse` | 100 | 10 | 0 | 0 | 0 | 6 / 7 |
| `ordinary-derivation` | 173 | 24 | 12 | 12 | 0 | 19 / 22 |
| `fragment` | 71 | 8 | 1 | 1 | 0 | 11 / 14 |
| `occurrences` | 241 / 240 | 39 | 4 | 4 | 0 | 26 / 28 |
| `cli-settings` | 35 | 1 | 6 | 6 | 1 | 10 / 10 |
| `cli-arguments` | 72 | 11 | 4 | 4 | 0 | 9 / 9 |
| `net` | 124 | 16 | 18 | 19 | 2 | 17 / 15 |
| `reference` | 98 | 15 | 16 | 16 | 0 | 15 / 13 |
| `check` | 309 | 59 | 17 | 17 | 0 | 50 / 44 |
| `focus-28j` | 327 | 28 | 49 | 49 | 9 | 89 / 69 |
| all | 2 831 / 2 830 | 342 | 235 | 236 | 28 | |

`occurrences` has one mutant fewer after: `Forest::root_set`'s, removed
with its test. The runs took three jobs on seven cores at night (2 to 8
and 9 to 15), two for `check` and `focus-28j`; `ordinary-parse`'s first
pass caught every viable mutant, so its second had none and its empty
lists were written from the first.

**The two differences, read.**

- `net`, `net.rs:466:64` (`+` for `*` in `Linker::next_partner`, which
  hands the same partner out again forever): a timeout in the first pass
  of both runs; in the second, at `main`, the unit's memory cap killed the
  two `focus::parallel` verdict sweeps (SIGKILL at 31 and 68 s, a failure,
  so caught) before the 60 s timeout, and after the pruning the timeout
  came first (one SIGKILL at 37 s, the run cut at 60 s). Rerun alone (one
  job, the whole suite, 300 s) it is caught on both sides by the same two
  tests, killed by the OOM killer (the journal says so) at 28 s at `main`
  and at 36 and 92 s after. A race between two ways of dying, not a lost
  test.
- `reference`, `tests::contractions` at line 978 at `main` and 982
  after: the same mutant of a test helper, moved four lines by the
  comment of the shrunk sweep.

`reference`'s `Prover::visit` (`+=` for `*=`, and `false`) were timeouts
at `main` and plain misses after, and missed alone at `main` too: with
100 programs the sweep ends within the timeout. They are on both lists.

**The timeouts, rerun alone** (one job, the whole suite, 300 s, no
baseline: `$S/rerun.sh`). At `main` 20, after the pruning 19 of the
fifteen groups, and the 9 of `focus-28j` on each side. `net.rs:520`
(`Linker::link` doing nothing) and `net.rs:466:64` are caught alone, on
both sides. The rest stay timeouts on both sides, the same mutants; read
by their `FAIL` lines (the rule in `.claude/rules/evidence.md`), four
have a test that fails before the hang and passes unmutated, so are
caught: `reach.rs:602:37` (`agrees_with_the_first_implementation`),
`few_equal_literals` always true (`search::tests::dispatch`),
`time_limit` none (`check_s_verdicts`) and `DEFAULT_MEMO_LIMIT` shifted
to zero (`mod.rs:1808`, `readme_examples`). Thirteen hang the whole suite with no
failure first, all in the Horn engine's arithmetic (`cover.rs`'s
`Cover::before`, four; `equation.rs`'s `fraction` and `gcd`, three;
`reach.rs`'s `find`, `keep` and `place`, six), neither caught nor missed,
as step 28j filed its hangs.
Of `focus-28j`'s nine, the same on both sides, `Table::get` none fails
`interchangeable_members_share_an_entry` before it hangs (caught), and
eight hang with no failure first (`Classes::canonical`'s run count;
`place`'s probe, three; `Memo::insert_hashed`'s record count;
`Memo::reserve`'s growth, three), the hangs step 28j listed.

**Against the committed lists.** Every survivor after the pruning that an
older list lacks survives at `main` too, so none comes from the
pruning: against `area-3.4`, the latest of each, one in `search`, two in
`horn-equation`, one in `horn-reach`, two in `check`; more against
`area-3.1` and `baseline`, whose code is older. `focus-28j`'s 49 are all
in `mutants/step-28j/focus-diff-2.txt` (51; twtorrxl killed two since).
`check` has 10 survivors that `mutants/step-28j/check.txt` (8) lacks,
at `main` as after; rerun alone at pwpppszv, step 28j's revision (the
workspace `../linlog-step30a-at-28j`), each is caught only by
`readme_examples`, which fails there with no mutant at all (three runs of
three: README's example at its line 244 reaches the recursion limit on
this machine before the time limit it shows, which nrpxxsrl fixed
after), so these ten were false catches in step 28j's
run and survive at its revision too. The checker's code is the same at
pwpppszv and at `main`, and its tests only grew.

**`mutants/step-28i/` is lost.** Step 28i's lists were never committed
and lived only in the workspace `../linlog-step28i-mutants`, which was
forgotten on the morning of 2026-10-16 before this session compared with
it (operation f6649f127aa6). The run at `main` beside the run after the
pruning stands in for it; the hangs step 28 names from it in prose
(`few_equal_literals` always true and `DEFAULT_MEMO_LIMIT` shifted to
zero, in `search`) hang here too, on both sides, each after a test
fails, so both count as caught.

### The runs

The scratch directory is this session's,
`/tmp/claude-1000/-home-tux-Projects-own-linlog/76df6421-3668-4499-a1b7-27e6e69ef0b6/scratchpad`
(`$S`): the driver `$S/mutate.sh` (the batch table of `mutants/run.sh`
plus `$S/extra-batches.txt`; a file `diff:PATH` mutates the lines a diff
changes), the lists under `$S/mutants/before/` and `$S/mutants/after/`,
the comparison `$S/compare.sh`, the reruns `$S/rerun.sh` and
`$S/reruns/`, the gate driver `$S/gate-seq.sh` and its logs, the timings
under `$S/times/`. The runs were paused three times for the author
(2026-10-16 09:16 to 10:10, 10:44 to 20:27, 22:38 to 01:19); a batch cut
midway reran from its start.

### Left open

- The first CI run on `main` fills the cache; whether a second run
  fetches the unchanged checks is to be seen on GitHub.
- The focused engine's other lines were not mutated: `focus-28j` holds
  the lines step 28j changed, and the engine keeps its own 73 tests.
