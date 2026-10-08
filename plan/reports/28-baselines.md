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
  (`mutants/run.sh`) and **nine fuzz targets**, one per reader of
  untrusted input (`fuzz/`), are wired in and have run; their results
  are below.
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

<!-- RESULTS -->
