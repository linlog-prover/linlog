# Step 28, stage 0: requirements, baselines and gates

One session (`step-28`, Opus 5.5 at `high`) on 2026-10-08, unattended
and supervised by the planning session. It built what the audit and the
fixes are judged against, and what proves that a fix changed nothing it
should not. The step's checklist is in
`plan/reports/28-audit-and-refactor.md`.

## Outcome

- **The register of later requirements** (`plan/notes/requirements.md`)
  holds 242 requirements of steps 29 to 38, `plan/later.md` and the
  decisions D15 to D22, each with the steps and source lines that need
  it, the item of the API it concerns, how a later session knows it is
  met, and its state today with evidence: 13 met, 75 partly, 149 not
  met, 5 unknown.
- **The behaviour lock** pins the command's output and exit status on 73
  calls (`cli/tests/lock.rs`, files under `cli/tests/lock/out`) and 32
  JSON forms of the library (`core/tests/lock.rs`,
  `core/tests/lock/json.txt`).
- **Twenty journeys** (`bench/src/journeys.rs`) count the instructions
  of the hot paths under callgrind; the **ratchet** (`linlog-bench
  ratchet`, the devshell's `ratchet`, the flake's `ratchet` check) holds
  them under committed ceilings (`bench/ceilings.csv`).
- **Mutation testing** of the checker, the readers, the search's front
  door, the Horn engine's refutations and the ordinary layer's checker
  (`mutants/run.sh`) ran 1 826 mutants in 5.1 hours: 321 survive both
  passes, among them the ordinary layer's checker replaced by `Ok(())`
  and the checker's memory bound and saturation guards
  (`mutants/baseline/`).
- **Nine fuzz targets**, one per reader of untrusted input (`fuzz/`),
  ran 9.4 hours in all until their coverage stalled or for 90 minutes:
  one finding, a panic of the ordinary syntax's parser on the six bytes
  `|-z⊢` (`core/src/ordinary/parse.rs:104`).
- **The counts track time** for any change that removes much work (all
  twenty journeys), not for one of a few percent (four of twenty
  disagree there); provisional, since another project's tests loaded the
  other cores during half of the timing.
- **The gate** (`gate` in the devshell) is what every later commit that
  touches code passes: clippy, the tests with both locks, both cargo-hack
  runs and the ratchet, in about four minutes on four cores.

## The register of later requirements

A workflow of seventeen agents on Sonnet 5.5 at `high` (run
`wf_a3156457-6a2`, 2.8 million tokens, 48 minutes) read one source each:
the ten prompts `plan/29-*.md` to `plan/38-*.md`, `plan/later.md` in
three parts, and the decisions D15 to D23 of `plan/README.md`. They
extracted 356 requirements with the item of the API each concerns and a
first look at the code for its state. One agent merged them into 239,
a completeness critic reread every source against the merge, and a last
agent checked each of the critic's points against its source before
folding it in: three requirements were added (R240, the Rocq bridges and
their pinned inputs; R241, the names of the modes in one table of the
library; R242, a writer of first-order certificates), four were
extended (R53, R101, R162, R209), source lines were corrected, and six
entries moved from met to partly, since "met" was said of code that does
not exist yet.

| kind | requirements | met | partly | not met | unknown |
|---|--:|--:|--:|--:|--:|
| wire forms | 17 | 0 | 5 | 12 | 0 |
| bounds and stops | 23 | 0 | 9 | 13 | 1 |
| wasm portability | 9 | 1 | 4 | 3 | 1 |
| data model | 36 | 1 | 13 | 22 | 0 |
| engine interface | 32 | 1 | 8 | 23 | 0 |
| proof term and checker | 12 | 0 | 3 | 9 | 0 |
| errors | 10 | 0 | 5 | 4 | 1 |
| options | 13 | 1 | 5 | 7 | 0 |
| export | 21 | 3 | 3 | 15 | 0 |
| CLI | 12 | 1 | 2 | 9 | 0 |
| harness | 34 | 5 | 12 | 16 | 1 |
| docs | 15 | 0 | 4 | 11 | 0 |
| other | 8 | 0 | 2 | 5 | 1 |
| total | 242 | 13 | 75 | 149 | 5 |

What weighs most for the audit and the design, by the number of later
steps that lean on it: a wire form for every options value (R1, which
no options value of the search or the batch has; the export options
do), outcomes and refutations that read back (R3), a version on every
JSON form (R6), a stop and a bound on every long call (the bounds and
stops section, where the read-back, the derivation's check and the
checker's pass take no stop), one family of errors with a wire form (the
errors section), the engine interface and dispatch as the place of a new
calculus (the engine interface section), and the data model's room for
terms, binders, boxes and cuts (the data model section). The states are
the extractors' first look; the audit checks them.

The register's evidence names lines as they were on 2026-10-08. One
claim was checked here at once: `core/src/serialize/search.rs:189` ends
the conversion of a `Refutation` to its wire form with `_ =>
WhyNot::Exhausted` (R3), which the mutation run found too (below).

## The behaviour lock

- **The command** (`cli/tests/lock.rs`): 73 calls, each pinned as one
  file under `cli/tests/lock/out` with the command line, the standard
  input, the exit status, standard output and standard error. The corpus:
  every fragment and mode as text (MLL, MLL with units, MALL with units,
  MELL, LL, intuitionistic, affine, Mix, a Horn program); every
  refutation as text and as JSON (unbalanced, the count equation,
  exhausted, the state equation); every output format of a proof (JSON
  classical and intuitionistic, LaTeX with and without `--standalone`,
  Typst, SVG, PNG and PDF by length and FNV-1a hash, Rocq with and
  without `--standalone`, the net as text and SVG, `--stats`, `--quiet`,
  `--no-verdict`, `--tree never`, `--compact always`, `--style`, a
  derivation over `--derivation-limit`); ordinary logic (classical,
  intuitionistic, minimal, a translation, `--linear`, JSON, Rocq); the
  problem files of `cli/tests/lock/inputs` (an LLTP file, a TPTP file in
  two logics, a `.spec` file in linear and affine mode, a JSON sequent on
  standard input); the batch (lines and JSON lines); `check` of a valid
  proof, a wrong one and text that is no JSON; an interactive session, a
  session's errors and its help; the four `seq` commands; and eight
  errors (a parse error of each syntax, a sequent that is not
  intuitionistic, a forced engine that does not apply, the occurrence
  and memory limits, a missing file, an unknown flag, no command).
- **The JSON forms** (`core/tests/lock.rs`): 32 lines of
  `core/tests/lock/json.txt`, written by the library itself: five
  sequents, outcomes of every fragment and mode and of every refutation,
  the reasons `memory_limit`, `stopped` and `copy_bound`, a forced
  engine, proofs of five modes, a proof structure from a proof and a
  partial one with Mix, and sessions open, after a step, closed, and
  intuitionistic.
- **Determinism**: a call whose output carries the search's counters
  names `--deterministic`; times after "after " and "time: " read as `…`.
  Both tests passed three times in a row unchanged, and in the gate.
- **The search's lock** is as before: the target set, the families,
  ILTP, qcover and the LLTP batch's verdicts.
- `BLESS=1` rewrites the files; a change of either lock is a commit of
  its own that says why (`.claude/rules/cli.md`, `core-sequents.md`).
- Two inputs of the first draft were corrected before the files were
  committed: `T` is an atom, not `⊤` (the syntax spells it `top`), and
  `!(a ⊸ b ⊗ b), a ⊢ b⁴` is unprovable, so the Horn call became
  `!(a ⊸ a ⊗ b), a ⊢ a ⊗ b ⊗ b`.

## The journeys and the ratchet

Twenty journeys (`linlog-bench journeys`), each preparing its input and
then doing the measured work inside `journeys::measured`, which
callgrind counts alone (`--toggle-collect=*journeys::measured*`):

| journey | what it runs | instructions (ceiling) | wall-clock, measured part |
|---|---|--:|--:|
| `search-qbf-20-2` | qbf/20#2, a target-set row, the focused engine | 661 564 923 | 40 ms |
| `search-chain-128` | chain/128, a target-set row, many copies | 1 216 594 953 | 50 ms |
| `search-partition-no-5` | partition-no/5, a target-set row, a refutation | 8 523 805 | 0.5 ms |
| `search-wide-m2-256` | wide-m2/256, the net engine | 274 070 486 | 11 ms |
| `search-spec-chain` | a chain of 2 000 transitions, the Horn engine, affine | 40 115 539 | 1.7 ms |
| `search-chain-64-intuitionistic` | chain/64 intuitionistic, the two-sided engine | 154 301 282 | 6.5 ms |
| `search-additive-14` | additive/14, the additive path | 608 840 614 | 70 ms |
| `read-text` | wide-m1/2048 printed and parsed | 18 548 925 | 1.1 ms |
| `read-json` | the same sequent from JSON | 12 515 726 | 0.4 ms |
| `read-lltp` | an LLTP problem of 2 000 clauses | 32 990 483 | 1.8 ms |
| `read-tptp` | a TPTP problem of 2 000 formulas | 30 658 823 | 1.5 ms |
| `read-spec` | a `.spec` problem of 2 000 rules | 22 458 173 | 1.2 ms |
| `check-qbf-20-2` | the checker on qbf/20#2's proof | 135 217 544 | 9.7 ms |
| `check-wide-m1-2048` | the checker on wide-m1/2048's proof | 8 772 182 | 0.6 ms |
| `derivation-chain-64` | chain/64's derivation, built and written as text | 17 006 453 | 0.9 ms |
| `render-latex` | LaTeX of that derivation | 6 585 706 | 0.3 ms |
| `render-typst` | Typst of it | 5 779 945 | 0.3 ms |
| `render-svg` | SVG of it | 86 837 537 | 4.9 ms |
| `batch-families` | the smallest instance of every family as one batch | 81 563 494 | 4.0 ms |
| `ordinary-pigeons` | three pigeons in two holes: translated, proved, read back to LK, checked | 39 298 004 | 2.4 ms |

The times are the median of three runs on core 2 by the release build,
with the mutation run and the fuzzers beside it on other cores: an
orientation, not a measurement (the timed validation below is).

- **Deterministic**: every search runs on one thread, and a sequent with
  exponentials takes the rarer literal's search alone (`Bias::Rarer`),
  because the default bias runs two searches in turns whose waiting
  thread wakes on a clock, so its count would depend on the time. Two
  runs of the same binary gave the same count on every journey but
  `read-spec`, which moved by 267 instructions (10⁻⁵); the environment
  of a run (variables of other lengths) changed no count.
- **Every journey checks its verdict**: a search that stops deciding
  fails the journey instead of counting something else.
- **The shell's build and the flake's count apart**: the release build
  in the shell and the crane build in the sandbox, same sources and
  toolchain, differ by under 0.1 % on nineteen journeys and by 0.9 % on
  `render-svg`; a rebuild after an edit that only moved lines
  (rustfmt) moved `check-qbf-20-2` by 0.6 %. So the ceilings are the
  larger of the two builds' counts, and the tolerance of `ratchet
  --check` is 2 % (`ratchet::TOLERANCE`). A before-and-after comparison
  for a change of speed takes both counts from one build, where they are
  exact, and judges far finer than the ratchet.
- **The ratchet** is `linlog-bench ratchet` (the devshell's `ratchet`):
  `--check` fails above a ceiling by more than the tolerance or on a
  journey without one, `--lower` writes every count that went down and
  never raises one, `--keep DIR` keeps callgrind's files for
  `callgrind_annotate`. The flake's `ratchet` check runs `--check` on
  the crane build; its first run passed, every journey within 0.1 % of
  the shell's counts but `render-svg`.
- **A caution for CI**: glibc picks `memcpy` and `memset` by the CPU's
  features, and valgrind passes some of them through, so a runner with
  another CPU may count otherwise; the first CI run of the `ratchet`
  check is the test of that, and a failure there with every local count
  under its ceiling is that and no regression.

## The supervisor's review of the register

On 2026-10-08 at 23:05 the supervisor sent thirty issues with the register
as committed in 5bf43829, from a review against the later prompts,
`plan/later.md`, the decisions and research notes written that night
(Fable 5.1 looked for gaps, Opus 5.5 checked each against its sources):
17 missing requirements, 5 wrong readings, 5 conflicts, 3 vague entries.
A workflow (`wf_6d613a43-832`, stopped at the usage limit and resumed at
01:51) had six agents on Sonnet 5.5 at `high`, another model than the
review's checker, judge five issues each against the register, the
sources, the research notes and the code, without the review's own
verdict as a premise; an agent on Opus 5.5 at `high` then folded the
results in (adbf12b6). Verdicts: take 5, change 23, reject 0, decision 2;
the writer turned one more into a conflict (C3). None was rejected: the
verifiers found every issue to hold at least in part, and most proposals
over-specified a design (a trait's name, a type's shape) that the
register now leaves to the design and records as a need.

| issue | kind | verdict | what the register got |
|---|---|---|---|
| I01 | missing | take | new entry R243 (Bounds and stops) |
| I02 | missing | change | new entry R244 (Data model); amended R230 (api.md also records the member type and the zone and frame types). R50 already lists Dyadic, so it needed no change |
| I03 | missing | change | new entry R245 (Data model); amended R170 (printing an occurrence below a binder with the binders' names) |
| I04 | wrong | change | amended R52, R63 and R135 |
| I05 | missing | change | amended R107 |
| I06 | conflict | decision | conflict C1; R53, R54 and R230 now point to it |
| I07 | conflict | change | amended R145, R104 and R123; added a bullet to "Not merged" |
| I08 | conflict | change | amended R57, R58 and R119; added a bullet to "Not merged" |
| I09 | missing | take | new entry R246 (Proof term and checker) |
| I10 | wrong | change | amended R36 (new requirement text and sources) |
| I11 | wrong | change | amended in place (the writer reported "already held"; the diff shows the change) |
| I12 | vague | change | amended in place (the writer reported "already held"; the diff shows the change) |
| I13 | vague | change | amended R141 and R144 (one sentence each). R50 already held the proposed text |
| I14 | conflict | change | amended in place (the writer reported "already held"; the diff shows the change) |
| I15 | conflict | take | amended R153 (it now names R20 as the source of the assignment). R20 already held the proposed text |
| I16 | missing | decision | conflict C2; R20, R72, R113, R153 and R154 now point to it |
| I17 | missing | change | amended R70 (steps, sources, requirement, met when, state now) and R124 |
| I18 | wrong | change | amended R78 (requirement, met when, sources, state now), R201, R8, R94 and R117; conflict C3 (nullary Mix) for the decision the result embeds, and R78, R94 and R117 point to it |
| I19 | wrong | take | amended in place (the writer reported "already held"; the diff shows the change) |
| I20 | missing | change | new entry R247 (Wire forms); amended R190 (steps, sources, requirement, met when) |
| I21 | missing | change | new entry R248 (Data model) |
| I22 | missing | change | new entry R249 (Data model) |
| I23 | missing | change | new entry R250 (Bounds and stops) |
| I24 | missing | take | new entry R251 (Data model) |
| I25 | missing | change | amended R163 |
| I26 | vague | change | amended in place (the writer reported "already held"; the diff shows the change) |
| I27 | missing | change | amended R218; new entries R252 (minimum toolchain, in Other) and R253 (status of linlog_cli, in Docs) |
| I28 | missing | change | amended R149 |
| I29 | missing | change | new entry R254 (Proof term and checker) |
| I30 | missing | change | amended R11, R62 and R67 |

The register now has 254 requirements (13 met, 75 partly, 161 not met,
5 unknown) and a section "Conflicts for the author" with three decisions,
each with the entries concerned, the options and a recommended answer:
C1, when the written order of a sequent's roots becomes its canonical
form (recommended: at step 28, before 0.1.0); C2, where the search for a
falsifying assignment lives and whether a refuter may decide an unknown
(recommended: in the library as refuters, the run after an unknown off
by default); C3, whether nullary Mix exists (recommended: not for 0.1.0
and step 33). The writer's account said five issues (I11, I12, I14, I19,
I26) needed no change; the diff shows their entries amended all the
same, and those amendments were kept.

## Fuzzing

Nine targets (`fuzz/fuzz_targets/`), one per reader of untrusted input,
built with cargo-fuzz 0.13.2, libfuzzer-sys 0.4.13 and the nightly
rustc 1.100.0 of 2026-09-25 (the `fuzz` devshell), with debug assertions
and overflow checks on, seeded by `fuzz/seed.sh` from `bench/problems`,
the command's own JSON (proofs, sessions) and the LLTP, ILTP and qcover
libraries. `fuzz/run.sh` ran them as the unit `step28-fuzz` from 21:12
to 00:15 on cores 12 to 15, one target per core, each in a scope of
4 GiB, libFuzzer in fork mode going on past a crash, a timeout (20 s for
one input) or an out-of-memory, and stopped a target after 15 minutes
without a new edge or at 90 minutes.

| target | reads | time | ended | edges | corpus | finds |
|---|---|--:|---|--:|--:|--:|
| `json_proof` | a proof file, then the checker in the mode of its first byte | 5 402 s | cap | 3 599 | 3 440 | 0 |
| `json_session` | a session, and a complete one's proof checked | 5 402 s | cap | 4 227 | 3 516 | 0 |
| `json_structure` | a proof structure, the criterion, sequentialization and the checker | 4 261 s | stalled | 3 394 | 2 959 | 0 |
| `json_sequent` | a sequent's JSON, round trip | 3 931 s | stalled | 2 855 | 2 656 | 0 |
| `lltp` | an LLTP problem | 3 871 s | stalled | 1 525 | 1 444 | 0 |
| `spec` | a `.spec` problem within a bound of 2²⁰ | 3 481 s | stalled | 1 678 | 1 043 | 0 |
| `ordinary_text` | the ordinary syntax, round trip | 2 911 s | stalled | 957 | 881 | 3 312 |
| `tptp` | a TPTP problem | 2 521 s | stalled | 1 141 | 1 150 | 0 |
| `sequent_text` | the text syntax, round trip | 2 100 s | stalled | 1 432 | 943 | 0 |

- **One finding, for the audit**: the ordinary syntax's parser panics on
  untrusted input. `core/src/ordinary/parse.rs:104:31` slices a string
  inside a multi-byte character; the six bytes `|-z⊢` give "start byte
  index 5 is not a char boundary; it is inside '⊢' (bytes 3..6 of
  string)". The 3 312 files under `fuzz/artifacts/ordinary_text/` are
  this one panic (reproduced by hand from the smallest). The command
  reads that syntax under `--logic`, and the web client will feed it
  text from a browser: a panic where an error is owed. Not fixed in
  stage 0.
- **No other crash, timeout or out-of-memory** in 8.6 hours of fuzzing
  over the eight other targets, no assertion of a round trip broken, and
  no proof structure or session that the checker refused after the
  library accepted it.
- Two targets reached the cap with coverage still creeping up:
  `json_proof` went from 3 566 edges at 61 minutes to 3 599 at the end,
  `json_session` from 4 208 to 4 227; they are the readers with the most
  behind them. A later round gives them longer;
  their corpora stay under `fuzz/corpus/` for it.

## Mutation testing

cargo-mutants 27.1.0 with cargo-nextest 0.9.146 (devshell), on the scope
in `mutants/run.sh`: the checker, the readers, the search's front door,
the Horn engine's refutations and the ordinary layer's checker. Each
batch runs twice: every mutant against the tests that exercise its file
(a nextest filterset; each one's in the script), then what survived
against the whole workspace suite (`--iterate`). The `mutants` cargo
profile (opt-level 1, debug assertions and overflow checks on) runs the
whole suite in 25 s where `dev` takes two minutes; an incremental
rebuild is 1 to 9 s. The run went as the unit `step28-mutants` on cores
6 to 11, three mutants at a time, from 21:03 to 02:10, 5.1 hours in all
(each batch's time below); the three batches the flaky test had
distorted ran again from 02:11 to 02:56 on the fixed tests, and with
mutants of tests' own helpers left out (`exclude_re`), which removed 42
of the checker's 319.

| batch | files | mutants | caught | unviable | surviving | time | where they survive |
|---|---|--:|--:|--:|--:|--:|---|
| `check` | `proofs/check.rs`, `proofs/mod.rs` | 277 | 192 | 47 | 37 | 1 467 s | `Pass::rule` 6, `Bag::unite` 4, `Pass::unite` 3, `afford` 4, `State::bytes` 2, `Observer` 3 |
| `parse` | `parse/mod.rs` | 107 | 92 | 8 | 5 | 674 s | `Parser::operator` 3, `Parser::operand` 2 |
| `serialize` | `serialize/*.rs` | 33 | 7 | 24 | 0 | 362 s | |
| `lltp` | `lltp.rs` | 55 | 40 | 2 | 13 | 594 s | `clauses` 11 |
| `mist` | `mist.rs` | 135 | 94 | 13 | 24 | 1 508 s | `Reader::rule` 9, `read_within` 5 |
| `ordinary-parse` | `ordinary/parse.rs` | 77 | 65 | 6 | 5 | 675 s | `from_str` 2 |
| `ordinary-derivation` | `ordinary/derivation.rs` | 149 | 106 | 16 | 21 | 1 249 s | `Derivation::check_one` 9 |
| `search` | `search/mod.rs` | 140 | 94 | 16 | 19 | 1 854 s | `Refutation`'s `Display` 4, `Refutation::needed` 3, `Statistics::add` 3 |
| `horn-mod` | `search/horn/mod.rs` | 136 | 96 | 19 | 15 | 1 160 s | `Program::caps` 7 |
| `horn-equation` | `search/horn/equation.rs` | 264 | 162 | 1 | 69 | 3 463 s | `Tableau::pivot` 33, `fraction` 16, `integers` 7 |
| `horn-cover` | `search/horn/cover.rs` | 178 | 112 | 2 | 61 | 2 361 s | `Search::keep` 15, `room_in_edges` 12, `dominated` 10 |
| `horn-reach` | `search/horn/reach.rs` | 275 | 197 | 3 | 52 | 2 846 s | `Search::expand` 8, `take` 7, `keep` 6 |
| total | | 1 826 | 1 349 | 157 | 321 | 5.1 h | |

"Surviving" is missed or timed out in the second pass; the lists are
`mutants/baseline/BATCH.txt`, cargo-mutants' files
`target/mutation/baseline/BATCH/` (the first pass in `mutants.out.old`).
What the audit should read first, from the lists:

- **The ordinary layer's checker accepts everything unnoticed**:
  `Derivation::check_one -> Ok(())` survives the whole suite, with eight
  more of its guards (`a == b && principal.is_none()` replaced by `true`,
  the arm of `ContractRight` deleted, the minimal logic's guards). No
  test gives that checker a wrong derivation of LK or LJ; it is only ever
  shown the read-back's own output. A must-fix by the rubric's standard
  of a checker.
- **The checker's bounds and accounting are untested at their edges**:
  `afford -> Ok(())` (the refusal of tables too large to allocate),
  `State::bytes` with `+` turned into `-` or `*`, `Observer::bytes -> 1`,
  `Pass::charge` with `>` as `>=`, and the guards of `Bag::add` and
  `Bag::unite` (`>` replaced by `<`, `==` or `>=`), which are what keeps
  a zone's counts from passing `Bag::MOST`. A memory bound and a
  saturation guard that no test reaches are the kind of number the
  conduct's rule says to test by an input that gets there. The soundness
  rule itself (`Problem::Surplus`) has its test; these guards are its
  arithmetic.
- **The state equation's simplex** (`horn-equation`: `Tableau::pivot` 33,
  `fraction` 16): a refutation stands on the exact check in integers
  after the simplex, so a mutant that weakens the simplex mostly loses
  refutations rather than making a wrong one; but `integers` has 7
  survivors, and that check is what soundness rests on.
- **Coverability's pruning** (`horn-cover`, 61): `keep`, `dominated` and
  `room_in_edges` decide which markings are kept; their survivors either
  change only the work done or no test tells a wrong prune from a right
  one, which the qcover suite's known answers may.
- **The readers** survive little (parse 5, ordinary parse 5, serialize
  0): the JSON lock of stage 0 now catches the deleted arms of the
  refutation's wire form, which the first trial run found missed. LLTP's
  `clauses` (11) and Mist's `Reader::rule` (9) have more.
- The front door's survivors are mostly in printing (`Refutation`'s
  `Display`) and in `Statistics::add`, whose sums no test compares.
- `mist` had 7 timeouts and 3 failures in its first pass; a timeout
  counts as surviving here (a mutant that makes a reader loop for ever is
  a finding, but the per-mutant limit also catches a slow test under
  load).

### A flaky test and the mutation run

The supervisor saw a test fail once and pass on its rerun (2026-10-08,
21:48). It was `linlog-cli::cli every_style_option`, and the cause was
in the CLI tests' helper: it spawned the command, then wrote standard
input and unwrapped the write, and a call that does not read its input
can exit first, so the write failed with a broken pipe
(`cli/tests/cli.rs:28`). Under the mutation run's load it failed in 2 of
6 and then 1 of 8 whole-suite runs on cores 2 to 5; with the write
taking a closed pipe as no failure (9556616f, also in `readme.rs` and
the lock's own helper) it passed 12 of 12, and the gate.

A flaky test can only make a mutant look caught, never the reverse: a
mutant is caught when some test fails, and a timeout counts as
surviving in the lists here. Reading the failing tests of every caught
mutant's log (`flaky_catches`, a scratch script): in the checker's batch
all 9 catches of the second pass and 5 of 230 in the first were by that
test alone, in `parse` 1 and in `lltp` 3 (both in the second pass);
every later batch ran on the fixed tests and had none. The three batches
ran again: the checker's survivors went from 27 (33 less the 6 in test
helpers) to 37, `parse` from 4 to 5, `lltp` from 10 to 13. The lists
above are the reruns'. `.claude/rules/evidence.md` says how a later run
looks for the same.

## The gate

`gate` in the devshell runs, with `--locked`: `cargo clippy --workspace
--all-targets -- --deny warnings`, `cargo test --workspace` (the
behaviour lock among the tests), `cargo hack check --each-feature -p
linlog`, `cargo hack check --feature-powerset --depth 2 -p linlog` and
`linlog-bench ratchet --check --jobs 4`, and prints `gate: passed`. It
passed at 7ebc848e in 4 min 15 s, and again on the test fix 9556616f,
both in a scope on cores 2 to 5 (`systemd-run --user --scope -p
MemoryMax=8G -p MemorySwapMax=0 taskset -c 2-5 nix develop -c gate`,
which CLAUDE.md's verification table now names for every commit that
touches code). A session may set the same command as its `/goal`
condition. The target set (`bench/targets.sh`) stays the search's own
check after a commit that touches `search/`, and `nix flake check` the
whole one: it passed at 2b332aa1 (2026-10-09, 03:00, every check with
`--keep-going`, the `ratchet` check among them) after a first run failed
on the formatting of a fuzz target written from the shell (8f26bd36).

## The tools, at the versions the locks pin

| tool | version | where |
|---|---|---|
| cargo-mutants | 27.1.0 | nixpkgs, devshell |
| cargo-nextest | 0.9.146 | nixpkgs, devshell |
| valgrind (callgrind) | 3.27.1 | nixpkgs, devshell (`valgrind.out`) and the `ratchet` check |
| cargo-fuzz | 0.13.2 | nixpkgs, `fuzz` devshell |
| rustc nightly | 1.100.0-nightly of 2026-09-24 (the overlay's 2026-09-25) | rust-overlay, `fuzz` devshell only |
| libfuzzer-sys | 0.4.13 | `fuzz/Cargo.lock` |

What contradicted expectation at these versions: cargo-mutants mutates
the helpers of a `#[cfg(test)]` module (`in tests::…`), so the scope
leaves them out by `exclude_re`; its `--iterate` keeps the first pass's
outcomes in `mutants.out.old`, which is where a first pass's logs are
read; valgrind's default output in nixpkgs is its manual, so the flake
names `valgrind.out`.

## Decided unattended

- **Mutation testing with two passes and nextest's filtersets**, as the
  prompt asks; nextest rather than cargo's test runner because its
  filtersets pick tests by binary and path in one expression and it runs
  each test in a process of its own. Set aside: the whole suite for
  every mutant (5 hours more), a test filter by name only.
- **A cargo profile `mutants`** at opt-level 1 with debug assertions:
  set aside, `dev` (the suite takes two minutes for one test) and
  `release` (overflow checks off, so an overflowing mutant passes).
- **Fuzzing in fork mode, past crashes, with a stall and a cap per
  target** (15 minutes without a new edge, 90 minutes at most); set
  aside, one fixed time for every target, and stopping at the first
  crash. The fuzz crate is a workspace of its own so that nothing stable
  sees the nightly compiler.
- **The behaviour lock as plain files compared byte for byte, with
  `BLESS=1`**, as `core/tests/export.rs` already does for its snapshots;
  set aside, a snapshot crate (a dependency for what thirty lines do).
  PNG and PDF by length and an FNV-1a hash, so no dependency for a hash.
- **Journeys inside the harness** (`linlog-bench journey`, `journeys`,
  `ratchet`), counting only a `measured` part with callgrind's
  `--toggle-collect`, so that the set-up a journey needs does not count;
  set aside, the command as the journey (its start-up and clap's parsing
  would be most of a small journey) and a crate for callgrind's client
  requests.
- **The ceilings as the larger of the shell's and the flake's counts,
  with a tolerance of 2 %**; set aside, a tolerance per journey (none
  was needed) and ceilings from the flake's build alone (`--lower` could
  then not be run in the shell).
- **The gate as a devshell command**, not a script under `bench/`: it is
  what a session runs before every commit, beside `check` and `tests`.
- **The flaky test fixed rather than left out of the mutation run**: the
  fault was in the test's helper, and the same pattern was in the lock's
  and README's helpers; leaving the test out would have hidden the style
  options from every mutant.
- **The register's second review**: verified by Sonnet 5.5 (another
  model than the review's checker), folded in by Opus 5.5; its three
  conflicts are recorded as decisions for the author with a recommended
  answer, the entries keeping their present text until the answer.

## Do the counts track time?

Before any count is trusted, the same journeys were timed for three
builds of the same code whose counts differ: the release profile at
opt-level 1, 2 and 3 (`CARGO_PROFILE_RELEASE_OPT_LEVEL`), counted under
callgrind as the ratchet counts. The timing ran from 03:04 to 03:37 on
2026-10-09, pinned on core 2 (`taskset`, a performance core), the
driver on core 3, after the mutation run and the fuzzers had ended and
`nix flake check` had finished, with no timer due (`logrotate` at 03:00
is trivial; `nix-optimise` was due at 03:54). Per journey five rounds,
each running the three builds once in an order that rotates; a run is a
process doing the journey 5 to 200 times (about 0.3 s of measured work),
and its time is the median of the measured parts. Recorded per run: the
process's user plus system CPU time against its wall-clock time, and the
busy share of the other fifteen cores from `/proc/stat`. Another
project's browser tests (headless Chrome and Node, the author's, which
the supervisor would not stop) loaded the other cores during the first
half: up to 56 % on the small `search-partition-no-5`, 7 to 34 % on the
other searches and readers, 1 % from the derivation on.

Each row gives the count and the median time of opt-level 1 and 2
relative to opt-level 3, the largest spread of a build's five rounds
((max − min) / median), the process's CPU share of its wall-clock time,
and whether the three builds order alike by count and by time:

| journey | count opt1 | time opt1 | count opt2 | time opt2 | spread | CPU/wall | others busy | follows |
|---|--:|--:|--:|--:|--:|--:|--:|---|
| `search-qbf-20-2` | +12.7% | +11.1% | +4.8% | +1.0% | 6.0% | 0.996 | 10% | yes |
| `search-chain-128` | +13.2% | +18.4% | +2.4% | +3.1% | 1.9% | 0.996 | 12% | yes |
| `search-partition-no-5` | +14.6% | +15.4% | +3.5% | +3.7% | 10.6% | 0.935 | 56% | yes |
| `search-wide-m2-256` | +10.2% | +6.8% | +4.3% | +8.9% | 6.4% | 0.992 | 34% | no |
| `search-spec-chain` | +18.0% | +19.3% | +2.6% | +1.7% | 5.7% | 0.996 | 20% | yes |
| `search-chain-64-intuitionistic` | +13.4% | +18.2% | +2.5% | +3.1% | 2.7% | 0.996 | 28% | yes |
| `search-additive-14` | +40.4% | +23.2% | +3.6% | +2.2% | 6.4% | 0.995 | 20% | yes |
| `read-text` | +32.8% | +27.2% | +7.4% | +8.6% | 7.0% | 0.996 | 18% | yes |
| `read-json` | +27.6% | +50.6% | +2.0% | +2.5% | 7.1% | 0.996 | 15% | yes |
| `read-lltp` | +33.4% | +29.8% | +3.5% | +2.8% | 3.2% | 0.996 | 7% | yes |
| `read-tptp` | +59.8% | +59.6% | +7.4% | +7.9% | 3.1% | 0.997 | 9% | yes |
| `read-spec` | +27.4% | +24.5% | +2.5% | +2.8% | 3.4% | 0.996 | 8% | yes |
| `check-qbf-20-2` | +33.9% | +13.0% | +7.2% | -3.9% | 12.2% | 0.993 | 22% | no |
| `check-wide-m1-2048` | +30.5% | +7.9% | +4.8% | -3.7% | 12.1% | 0.998 | 8% | no |
| `derivation-chain-64` | +8.3% | +13.7% | -0.1% | +2.5% | 5.4% | 0.998 | 1% | no |
| `render-latex` | +19.1% | +33.3% | +0.7% | +1.5% | 1.3% | 0.998 | 1% | yes |
| `render-typst` | +22.8% | +38.6% | +0.8% | +1.2% | 1.5% | 0.998 | 1% | yes |
| `render-svg` | +15.2% | +16.7% | +1.6% | +1.5% | 1.8% | 0.998 | 1% | yes |
| `batch-families` | +27.1% | +15.2% | +9.5% | +3.7% | 0.4% | 0.997 | 1% | yes |
| `ordinary-pigeons` | +19.3% | +17.2% | +1.9% | +1.5% | 2.9% | 0.997 | 1% | yes |

16 of 20 journeys order the three builds alike by count and by time.

What this supports:

- **A large drop in count is a drop in time on every journey.** Opt-level
  1 counts 8 to 60 % more than opt-level 3 and is slower on all twenty,
  by 7 to 60 %. The processes ran on their core nearly undisturbed
  (CPU over wall-clock 0.993 to 0.998, `search-partition-no-5` 0.935).
- **A change of a few percent is not settled by a count alone.** Opt-level
  2 counts 0 to 10 % more than opt-level 3; on sixteen journeys its time
  moves the same way, but on four it does not: `check-qbf-20-2` (+7.2 %
  instructions, −3.9 % time) and `check-wide-m1-2048` (+4.8 %, −3.7 %),
  where the checker's hash tables make the time the memory's rather than
  the instructions'; `search-wide-m2-256`, whose builds order alike in
  time but with opt-level 2 slower than 1; and `derivation-chain-64`,
  whose counts are equal within 0.1 %. The rounds' spread was 0.4 to
  12 %, widest on the checker's journeys and where the other cores were
  busiest.
- **So**: the ratchet's counts are trusted as a guard against
  regressions of more than its 2 % tolerance and as a measure of a
  change that removes work; a change claimed to save a few percent, on
  the checker's journeys above all, is shown by pinned wall-clock time
  beside the count, as `bench.md` already asks for clears (`memset`
  counted per byte).
- **Provisional**: the first half of the timing ran beside another
  project's tests, and the small differences are within that noise. The
  validation is to be repeated in a quiet window (the supervisor names
  one): `bench/counts-against-time.py` (its header has the commands)
  runs it and prints the table above; this run's rows and counts are in
  `bench/counts-against-time/2026-10-09*`.

## For the audit and the sessions after it

- **Findings of stage 0 for the audit**: the ordinary parser's panic on
  untrusted input (`ordinary/parse.rs:104`); the ordinary layer's
  checker, which no test shows a wrong derivation (`check_one -> Ok(())`
  survives); the checker's memory bound and the saturation guards of its
  counts, which no test reaches (`afford`, `State::bytes`, `Bag::add`,
  `Bag::unite`); the state equation's exact check in integers (7
  survivors in `integers`); coverability's prunes (61 survivors in
  `horn-cover`); the wildcard arm of the refutation's wire form
  (`serialize/search.rs:189`, register R3); the CLI tests' stdin writes
  that failed on a broken pipe, fixed here (9556616f).
- **The register's three conflicts** (C1 to C3) are decisions for the
  author, with recommended answers.
- **Every later commit that touches code passes `gate`**; a change of a
  lock's files or of a ceiling is a commit of its own that says why.
- **A check round** runs `mutants/run.sh --label round-N` for the
  batches its area touched and `--compare round-N` against the baseline,
  `fuzz/run.sh` for the readers it touched (the corpora stay), and the
  ratchet; `.claude/rules/evidence.md` says how to read a run for flaky
  catches.
- **Still provisional**: the timing of counts against time, to be
  repeated in a quiet window with `bench/counts-against-time.py`; and the
  `ratchet` check on CI, whose runners may pick other `memcpy` variants.
- **Unsigned commits**: from 7c438f1b on (signing failed after 22:50),
  for the supervisor to sign.


## From the review (2026-10-09)

Read in full: this report, the checklist, the commit list (19 commits,
all by the author's identity, from 7c438f1b on unsigned) and the shape
of every changed file; the ratchet's comparison (`bench/src/ratchet.rs`)
and its flake check; the scripts of the mutation and fuzz runs. Run
again: clippy, the workspace tests with both locks, the core tests, both
cargo-hack runs and cargo-deny on 71cc1b40, and `nix flake check`.
Reproduced by hand: the ordinary parser's panic on `|-z⊢` (exit 101,
where an error with status 2 is owed). No new file lacks its licence
header, and no comment names the plan.

- **Accepted**: the register with the second review folded in, the two
  locks, the journeys and the ratchet, the mutation and fuzz baselines,
  the gate. The report says what each rests on and what is provisional.
- **Fixed in the review**: the scripts named their systemd units after
  the step (`step28-mutants`, `step28-fuzz`); a committed name names no
  step, so they are `linlog-mutants` and `linlog-fuzz`.
- **For later stages** (prompt 28, "From the review of stage 0"): the
  research notes the supervisor's workflows wrote the same night, now in
  `plan/notes/research/`; the timed validation repeated by the
  efficiency area on a quiet machine; how to wait on a run; the
  register's conflicts to the author with the audit's decision list.
- **Open**: the `ratchet` check's first run in CI, at the next push.
