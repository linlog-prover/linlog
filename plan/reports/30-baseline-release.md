# Step 30: the third baseline and the first release

## The checklist

| item | state | evidence |
|---|---|---|
| Session 30a: the tests pruned and CI cached (session `step-30a`, 2026-10-16) | done (2026-10-16 to 10-17) | Per-crate checks (voppnlzq) and the Cachix cache in CI (ptprktol): a change to `cli/` leaves the core crate's six Rust checks' derivations as they were, one to `plan/` every Rust check's. Tests pruned in wtpnsrlo and svrqukoy: four removed, one merged into `detection`, three shrunk; 448 tests to 443; `cargo test --workspace` in the dev profile 111.6 s to 50.3 s, the release tests with debug assertions 24.0 s to 13.4 s (cores 2 to 5). Every code commit gated. Mutation runs of every file whose tests changed (17 batches, 2 831 mutants at `main`, 2 830 after; `check` and `focus-28j` for step 28j's lists), at `main` and after: 235 and 236 surviving, the one more a timeout race that is caught alone on both sides; every timeout rerun alone; no new survivor. `mutants/step-28i/` lost (forgotten with its workspace); `mutants/step-28j/check.txt` short by ten false catches of a README test that failed unmutated at its revision. `nix flake check` passed at the session's last commit. |
| Session 30b: the memory bound charged and the memo's cut entries fixed (session `step-30b`, 2026-10-17) | done (2026-10-17) | The memo's cut entries are read only within their generation (vpryqrqn, qwwpklrw): probe A1's four rows refuted with the memo on at levels 7, 5, 5 and 12 in 88 to 341 µs; `deepening_closes_on_a_finite_space` fails before and passes after. At 10 s on one thread, rows decided (stems): ILLTP-SYJ-01 (tuned) 57 to 103 (14 to 19), KLE 243 to 264 (83 to 88), SYN and LCL 51 to 63 (18 to 21), every `-cbv` collection 196 to 232 (115 to 128); all 103 gains refutations, none proved by any tool, no decided row changed, every proof checked. The memory bound counts the input, the readings and a race's verdict (ypskkpwl, rutmmotq, xtsvttyy; departs from the design); `the_memory_bound_holds_the_input` ends the default at a bound one byte short of the input. The default's six runs of SYJ206+1.020 and SYJ212+1.020 rerun under the comparison's settings (xovvltrl): one ends at the memory bound, five at the time limit, none killed by the cgroup. Target sets against `after-efficiency` and `net-after-efficiency`: no verdict differs; only the chain family's four rows differ in counters, as before the session; five rows without a copy bound added (ttlytoks). One reviewer, five should-fix, all fixed. Every code commit gated (orzmpoxm a doc link that `nix flake check` found); `nix flake check` passed at the session's last commit. |

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

## Session 30b: the memory bound charged and the memo's cut entries fixed

**Outcome.** Iterative deepening now closes on a finite space: a failure
cut by the copy budget is read only within its generation, and the four
rows of probe A1, undecided at 10 s at levels 0.5 to 3.3 million, are
refuted in under a millisecond at levels 5 to 12. On the sets not tuned
on, at 10 s on one thread, KLE gains 21 rows (243 to 264 of 264; stems
83 to 88), SYN and LCL 12 (51 to 63 of 63; stems 18 to 21) and the cbv
translation 36 over its four collections (196 to 232 of 361; stems 115
to 128), with the tuning set ILLTP-SYJ-01 from 57 to 103 rows, every new
verdict a refutation, none against a decided verdict, none of a problem
any tool proved. The memory bound now holds the input: the forest and
its sequent, each search's reading, and the check and refutation after a
race, which were the part of the process outside the account. Rerun
under the comparison's settings, the default's six runs of
`SYJ206+1.020` and `SYJ212+1.020` stay within the 4 GiB cgroup (peaks
2.8 to 3.5 GiB): `SYJ212+1.020` in 01 ends at linlog's own memory bound
after 7.5 s, the other five at the time limit; three of them were killed
by the cgroup in the published run. Every code commit passed the gate.

### The memo's cut entries

**The mechanism**, confirmed first: probe A1 at `mypyslzq` (core 12,
one thread, 10 s) left `SYJ208+1.001` in cbn, `LCL181+1` in cbv,
`KLE017+1` in cbv and `KLE069+1` in 01 undecided at levels 1 811 730,
3 210 756, 3 339 257 and 470 095, with 13 to 61 memo entries. A failure
cut by the copy budget (`Failure::Exhausted`) rests on the branch it was
found in as much as on the sequent, since a search of the same sequent
below other ancestors may prune as a repeat what was cut there. Read at
the next level, it answered "cut" where a search would have ended in
repeats, and the level recorded the cut one copy higher, so no level
ever closed. Probe A4's bisection was not needed: the mechanism was
plain from A1 and the code.

**The fix** (vpryqrqn, and qwwpklrw after the review): each cut entry
carries a stamp of its *generation* in the high bits of its record's
entry word (`Memo::begin`, `Entry::code`; 30 bits, the memo emptied
when the stamp comes round). A generation is a run of deepening levels
each of which recorded a key new to the memo (`Memo::added`) and
emptied no part of it (`Table::emptied`: the memo, a shard of a pool's,
the table of zones), and the first that did not. After a level that
ended cut with no new key, or with an emptying, the next level starts
a new generation: it reads the proofs and complete failures of every
level and only its own cuts. `Run::run` makes the choice between
levels; a pool's workers never start a generation, since they live
within one level.

**Why deepening now closes on a finite space.** Let every branch of the
reachable stable sequents end in a repeat or a fact within a budget
`L`. Each level either starts a new generation or records a key not
recorded since the last emptying; there are finitely many keys, so
generations keep starting, at worst every other level. A fresh
generation at a level above `L` meets no genuine cut (none is reached
on a branch that ends within `L`), reads no old one, and so ends
without a cut: `Unprovable`.

**Why no verdict can turn wrong.** The fix only makes a lookup ignore a
cut entry, which is what the memo already does when it is emptied or
switched off (`--memo-limit 0`). A cut entry never refutes anything:
it makes a sequent's failure *cut*, and only an uncut level answers
`Unprovable`. Ignoring it makes the search look again, under the same
loop check and waiting failures, as a memo-free search would. No stale
entry can be read as complete (`Memo::refuted` reads only `Complete`
and `Hereditary`, which carry no stamp, and a `Complete` never rests on
a cut). Every proof is still checked by `conclude`.

**Decision: generations, not "cuts only in their own level".** The
note weighed both. Reading cuts only within their own level (built
first, "fix 1") also refutes the four rows, but it re-explores every
level of a deep proof: on the target set `chain/256` took 1 430 917
stable sequents for 65 413, `chain/16` 477 for 253, and on the tuning
set it lost a proof at 10 s (`SYJ203+1.010` in cbv). Generations move no
counter of a decided row of the target set, nor of
`a_close_settles_by_every_rule`'s cases, which fix 1 moved in all
seven.

**Tests.** `deepening_closes_on_a_finite_space` (focus/tests.rs): the
four sequents, unbounded, under a work bound of a million, on one
thread and on a pool of two with a memo of 64 entries. With the stamps
switched off in the tree (the old behaviour) the first stops at the work
limit at level 1 263 on one thread, and without the emptying trigger at
level 798 on the pool; with the fix all four are refuted at levels 5 to
12 within 28 258 units. It passed 50
runs of 50 in the release build. `cuts_hold_in_their_generation`
(memo.rs) pins the stamp, the replacement of a stale cut and the clear
at the wrap (removing the clear fails it).

### The memory bound

**What was outside the account.** A sub-agent's diagnosis (Opus, an
instrumented copy of `Account` against `/proc/self/statm`, cores 13 to
15) found nothing on the search's path that grows uncharged: on every
run the resident memory above the input stayed under the account's
peak, the account over-counting by 15 to 20 % (zeroed bitsets, spare
capacity). What passed the cgroup was **the input**, which the bound
left to the caller by design: the forest with its sequent, about 27
bytes an occurrence, 644 MiB for `SYJ206+1.020` in 01, 812 MiB for
`SYJ212+1.020` in 01, 575 MiB in cbn. The comparison gave linlog a
`--memory-limit` equal to the cgroup's (4 GiB), so the race, whose four
searches reach their bound within 5 to 10 s, held input plus bound:
4 275 to 4 436 MiB resident on `SYJ212+1.020` in 01 on three cores
with the account at about 4 GiB. One thread does not reach the bound
within 10 s (the comparison's one-thread peaks were 2.4 to 3.2 GB).
Two smaller gaps were real too: each search's intuitionistic reading
(a byte an occurrence, two or three per race, 46 to 58 MiB here), and
the verdict's check and refutation counts, which each side of a race
ran under a fresh bound of its own while the other side still
searched, so a race could hold two bounds.

**The fix** (ypskkpwl, xtsvttyy after the review; departs from the
design, which left the forest and the sequent to the caller):
`Forest::held_bytes` is charged once, by `prove_goal` and by `race`
(`Account::holding`: an account that holds its input for its life,
whose race parts leave room for it), each search charges its reading
(`Reading::held_bytes`), shares divide what the bound leaves, and a
race's two sides only search (`answer_goal`); the verdict is made once
both have ended (`Answered::concluded`), its check under what the bound
leaves (`Account::left`), a refused check naming the option's bound.
`--memory-limit`'s help and README say that the sequent counts (rutmmotq
re-blesses the behaviour lock's two help outputs, nothing else).

**What it costs.** An input near `Limits::occurrences` (fifty million)
leaves little for the search under the default gibibyte; no problem of
the library above a million occurrences (27 MiB of input) was decided
in the second baseline, so no decided row is at stake.

**Tests.** `the_memory_bound_holds_the_input` (search/mod.rs): `⊢ a, ~a`
with an atom's name of a mebibyte, through `prove_goal` and through
`race` (the command's default): one byte short of the input it ends at
the bound (with the input's charge switched off it proves), with half the input more it
proves, so the input counts once and the check has room beside it. The
accounting test pins `left`, the shares of what is left and a race's
parts. Moved for the input, each with its reason: the Horn tests'
`bounded` helper gives its bound beside the input, and the command's
test of a refused check moves its window from 704 to 832 KiB to about
920 to 1 040 KiB (960 KiB, inside both measurements of it).

### Evidence against overfitting

`linlog-bench run --copies none --timeout 10`, one thread (the default
bias's two searches alternating on it), each set before (`mypyslzq`)
and after (the session's code, built at xtsvttyy's tree but for a
`cfg` fix that changes no behaviour) on the same core of the same class
(8 to 11; by day, beside the target sets and the gates). A *stem* is a
problem's name before `+`, so that one family's twenty sizes count
once. Tuned on `ILLTP-SYJ-01` only; the rest was not looked at until
the fix was chosen.

| set | rows (stems) | rows decided, before | after | stems decided, before | after |
|---|--:|--:|--:|--:|--:|
| tuning: ILLTP-SYJ-01 | 252 (19) | 57 | 103 | 14 | 19 |
| KLE (01, cbn, cbv) | 264 (88) | 243 | 264 | 83 | 88 |
| SYN and LCL (three translations each) | 63 (21) | 51 | 63 | 18 | 21 |
| cbv: ILLTP-SYJ-cbv | 252 (19) | 99 | 123 | 18 | 19 |
| cbv: every `-cbv` collection (SYJ, SYN, LCL, KLE) | 361 (128) | 196 | 232 | 115 | 128 |

- **Every change is a refutation gained**: 46, 21, 12 and 24 rows (36
  of the cbv union); no row decided before is lost or changes verdict,
  and every proof is checked (`checked` is `ok` on every proved row).
  Fix 2 alone and the final engine decide the same rows.
- **No new refutation is wrong as far as anything can tell.** Of the
  103, none is proved by any tool in the comparison's `runs.csv`
  (Sympli refuted most of them too); 16 no other tool decided. 79 agree
  with their header. The 24 that contradict a Theorem header are
  `SYJ103+1`, `SYJ105+1.003`, `.004` in 01 and `KLE015`, `017`, `069`,
  `074`, `075`, `078`, `083`, `086`, `088` across the KLE translations:
  21 have a classical countermodel (the classical reading: `!` erased,
  `⊸` as implication, `⊗` and `&` as and, `⊕` as or; a script of the
  session's scratchpad), so they are no theorems and their headers are
  wrong, most of them already listed in `plan/notes/lltp-headers.md`.
  The other three are `KLE075+1` in its three translations, which is
  classically valid: Sympli refutes it too, and the engine at
  `mypyslzq` with the memo off refutes it in cbn (level 11, 130 ms), an
  independent path to the same verdict. The gap note left it to be
  decided by hand; two engines now agree that the header is wrong.
- **Fix 1** (cuts only in their own level), measured on the tuning set
  and cbv before the choice: the same 103 on SYJ-01, but 122 on
  SYJ-cbv, where it lost a proof (`SYJ203+1.010`, proved in 3.7 s
  before and in 3.8 s after the chosen fix, at the time limit under fix
  1) to its re-exploration.

### The target sets

Both at the session's code (xtsvttyy), on cores 6 and 7, committed in
ttlytoks as `bench/targets/after-generations.csv` and
`net-after-generations.csv`; `linlog-bench summary --against … --counters`:

- **The focused engine's set** against `after-efficiency.csv`: no
  verdict differs, nothing decided by one and not the other; of the 135
  problems both decide alike on one thread, 4 differ in counters, the
  chain family's (`chain/16` to `chain/256`, one memo entry fewer and
  one hit more). That difference predates this session: step 28i
  recorded it for the count of one sign, and the engine at `mypyslzq`
  gives the same counters on all four sizes, as does fix 2 alone.
  Among the undecided rows one net, `JoinFreeModules_joinFree-10_50_1`
  (the Horn engine), now ends at the time limit where the oracle ended
  at the memory limit; the engine at `mypyslzq` ends it at the time
  limit too. The five rows without a copy bound (A5) are refuted in
  every run, within 3.6 ms, at levels 5 (`LCL181+1` in cbv, `KLE017+1`
  in cbv), 7 (`SYJ208+1.001` in cbn), 12 (`KLE069+1` in 01) and 16
  (`SYN393+1` in 01). `SYJ212+1.002` in 01, first among them, was
  replaced: it is in the set already at copy bound 3, and the harness's
  `--resume` took that row for the run without a bound.
- **The net engine's set** against `net-after-efficiency.csv`: the 64
  rows equal in verdict, `links` and `tests`.
- Fix 1, measured on the way, kept the verdicts too but moved the chain
  rows twenty-fold (above); the decision rests on that.

The families at 5 s (`run --all-families`, core 6, `summary --strict`):
123 runs, no mismatch, 62 proved, 37 unprovable, 24 unknown, the same
as the engine at `mypyslzq` on the same core.

### The rerun of the comparison's two rows

Run as `bench/comparison.sh` runs the default's stream (`linlog-bench
run --tools linlog-default`, runexec 3.27, 10 s, 4 GiB as both the
cgroup's limit and `--memory-limit`, cores 12 to 15, `PATH` from
`.#comparison` built from the workspace at ttlytoks's tree,
`LINLOG_COMMIT` e0215978), after planning's `nix flake check` had
ended, nothing else timed running:

| file | published (60226b2c) | now (e0215978) | peak now |
|---|---|---|---|
| `ILLTP-SYJ-01/SYJ206+1.020` | out of memory (cgroup), 6.7 s | time limit, copy bound 4 | 3 574 MiB |
| `ILLTP-SYJ-01/SYJ212+1.020` | out of memory (cgroup), 9.6 s | linlog's memory bound, 7.5 s | 3 526 MiB |
| `ILLTP-SYJ-cbn/SYJ206+1.020` | time limit | time limit, copy bound 4 | 3 425 MiB |
| `ILLTP-SYJ-cbn/SYJ212+1.020` | out of memory (cgroup), 5.7 s | time limit, copy bound 4 | 3 493 MiB |
| `ILLTP-SYJ-cbv/SYJ206+1.020` | time limit | time limit, copy bound 2 | 2 888 MiB |
| `ILLTP-SYJ-cbv/SYJ212+1.020` | time limit | time limit, copy bound 2 | 2 836 MiB |

So the answer to the step's question: one ends at the bound, the other
five at the time limit. The brief's "both translations the comparison
published" was read as every translation the comparison published of the
two files, all three; the three killed ones are among them. Committed in
xovvltrl, "Rerun the default's runs…": `runs.csv` takes the six rows
(the family set, which a file given alone lacks), `ranking.md` is drawn
again (the default's "time" 545 to 547, "memory" 3 to 1, nothing solved
changes), `bench/TOOLS.md`'s table and its paragraph say so,
`default-rerun.env.json` records the run. README's "Beside the other
provers" cites only solved counts, which did not move. The figures are
not drawn again: `plots.py` draws an unsolved run at the limit whatever
ended it, and a new drawing changes every SVG by matplotlib's ids alone
(25 000 lines for no datum). The comparison's other rows are not rerun.

### The review

One fresh-context reviewer (Opus 5.5 at high, read-only, probes on
cores 13 to 15) read both fixes against the memo's contract
(`core-focus.md`) and the memory bound's (`core-search.md`), at
vpryqrqn and ypskkpwl. No must-fix: no wrong verdict, crash or broken
bound. Its findings, each answered:

1. *Deepening still never closed where the memo empties itself within
   every level* (should-fix; a pool of two with `--memo-limit 64`,
   one entry a shard: level 4 231 at five million units). Fixed in
   qwwpklrw: an emptying of the memo, a shard or the table of zones
   also starts a generation (`Table::emptied`), and
   `deepening_closes_on_a_finite_space` runs that pool too (it fails
   without the trigger at level 798). The witness and its variants
   (`-j 2` and `-j 3`, memo limits 4 and 64, inputs 1 and 3) all
   refute now, where fix 2 alone reached levels 4 236 to 12 047.
2. *A race's shares ignored the input* (should-fix: inside a race the
   input sat in the whole, so each share was half the bound less the
   reading, where only the bound less the input was left). Fixed in
   xtsvttyy: `Account::holding`, whose parts are bounded by what the
   input leaves; the accounting test pins it.
3. *The test of the wrap did not pin the clear* (should-fix). Fixed in
   qwwpklrw: a cut of generation 0 must be gone at generation 2³⁰;
   removing the clear fails the test.
4. *The input test did not pin "charged once"* (should-fix). Fixed in
   xtsvttyy: it proves under half the input more, so a double charge
   fails it.
5. *The command test's window numbers* (should-fix): the comment and
   ypskkpwl's message now say about 920 to 1 040 KiB, as both
   measurements agree (928 and 920 KiB the lowest that fit).
6. Minor, taken: a comment at the wrap's clear (the kept proofs wait
   for the next collection); the race's two moved edge cases (a proof
   settles the race before its check; a rejected proof is no longer
   hidden by the other side) and what the bound still leaves out (the
   command's `Image` under `--logic`, the goal's occurrences kept for
   the verdict) in `core-search.md`. Not taken: `Forest::held_bytes`
   counting atom names by length; it counts their capacity already.

The review also confirmed, by reading and probes: a stale cut is
ignored exactly as an absent entry (no hit counted, `Waiting::find` may
reuse a waiting failure); `Memo::refuted` reads no stamped entry; a
`Complete` never rests on a cut; the encoding fills the word exactly;
a restarted turn of the default bias replays the same generations; the
input is charged once per call on every front door and released with
it; the race's restructure keeps its order of settling, statistics,
work and panics. It did not verify the target-set claims, which need
`bench/*.sh`; those are this session's runs above. Its fixes failed the
gate once (xtsvttyy: `Account::new` and `Account::input` unused in
`cargo hack`'s builds without `interactive` or `parallel`); fixed in
the same commit, which then passed.

**`nix flake check`** at the report's commit (kqwmkxkv) failed on
`doc-private` alone: the memo module's doc linked `[`Memo::begin`]`,
which the private documentation cannot resolve from a module's own doc
comment. orzmpoxm spells out the path, as the module's other links do
(gated); the check then passed at orzmpoxm, and again at the session's
last commit.

### Options and front ends

No option was added. The generations have no knob: the memo's existing
`Options::memo_limit` (`--memo-limit`) still switches the memo off, and
the stamp is internal. The memory bound keeps `Limits::memory_bytes`
(`--memory-limit`, the JSON key `limits.memory_bytes`); what changed is
what it counts, and every front end that sets it gets the new meaning
with no change of its own (the command's help and README say so).

### Left open

- **The comparison's other counts** wait for the second night the
  author may ask for: the fix decides 103 rows of the intuitionistic
  library that the published run left undecided (on one thread at
  10 s), which the published ranking does not show.
- **KLE075+1**: now refuted by two engines against its header and with
  no classical countermodel. Worth a line in the header report
  (`plan/notes/lltp-headers.md`), whose sending is the author's.
- **The race's default at the default bound on the largest inputs**:
  the diagnosis found the default's race giving up by memory after
  about a second on `SYJ206+1.020` under the default gibibyte, before
  any search, since each of its four searches charges its own set-up
  (the counts, 10 bytes an occurrence, and the classes); one thread runs
  to its time limit there. Charging the input makes that earlier
  still. Sharing the set-up between a race's searches would be the
  fix; not done here (it is the set-up's layout, not the bound's
  account).
- **Two warnings in the test build without default features**
  (`distinct` and `without_one_sided` never used), in code this session
  did not touch; the gate does not build that configuration.
- **Signing** stopped working mid-session (the passphrase's cache
  expired and the unit that kept it warm had ended): the commits after
  ttlytoks are made with `signing.behavior=drop`, for the supervisor to
  sign.
