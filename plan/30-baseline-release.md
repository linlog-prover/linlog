# Step 30: the third baseline, and the first release

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing. The step needs the
machine for a night, which the author gives; this prompt was finished
at the review of step 29 (2026-10-16). Read before you start:

- `plan/reports/14-benchmarks.md` ("What step 16 must repeat"),
  `16-baseline.md`, `17-assessment.md` (5.6, the author's answer 9 and
  what follows it), `21-defaults.md`, `27`, `28` ("What steps 29 to 31
  must know"), `29` ("What later steps must know"), and this step's own
  report, `30-baseline-release.md`, which sessions 30a and 30b open.
- `plan/notes/distribution.md` (how and where to publish, what each
  registry asks, the repositories) and `plan/notes/lltp-headers.md`.
- `plan/notes/requirements.md`: the entries whose steps name 30 (R6,
  R19, R41, R50, R162, R194 to R197, R206, R215, R218 to R224, R232 to
  R236, R252, R253), each with its "State after step 28".
- `.claude/rules/bench.md`, `bench/baseline.sh`.

## How the step runs (2026-10-16)

Three sessions, one after another, each reviewed by `planning` before
the next starts. All are Opus 5.5 at `high`.

- **30a: the tests pruned, and CI cached** (the author's requests of
  2026-10-15, which they said to carry out without asking them again):
  only the necessary tests kept, with mutation and coverage evidence
  that nothing needed was lost; CI cached through the public Cachix
  cache `linlog-prover`, the test checks split per crate so that a
  push reruns only what it touches. Its brief is the supervisor's.
- **30b: the memory bound charged** (`plan/later.md`, "The memory bound
  is passed on wide intuitionistic searches by default"). Before the
  baseline, because the pass under the default must measure a default
  that keeps its bound. Find what the default's race allocates outside
  the bound's account (the second search's structures, the pool's
  workers, or a proof of `&` rules outside the focused engine's
  charge), charge it, and pin it with a test that runs under a bound
  small enough to be reached quickly. The gate; the target sets if the
  search changes, against their oracles. Step 29's published counts
  show linlog's default out of memory on `SYJ206+1.020` and
  `SYJ212+1.020`: the report says whether they now end at the bound or
  at the time limit. The author gave their word (2026-10-17): after
  the fix, rerun linlog's default on those two rows of the comparison,
  under the comparison's own settings, and update the published counts
  and every place that cites them (README's "Beside the other provers",
  `bench/TOOLS.md`, `bench/comparison/2026-10-15/`) in a commit of its
  own that says why. The second night of the comparison is still the
  author's to ask for.

  **And the memo's cut entries** (the author, 2026-10-17: fixed in
  30b, before the baseline). `plan/notes/research/comparison-gaps.md`
  (2.2, R1, probe A1): with the memo, `SYJ208+1.001` in cbn, `LCL181+1`
  in cbv, `KLE017+1` in cbv and `KLE069+1` in 01 stay undecided at 10 s
  at copy levels in the millions; with `--memo-limit 0` each is refuted
  in milliseconds at levels 6 to 16, and the engine of 2026-10-04
  refuted two of them. A cut entry, a failure within a budget in the
  branch and level it was found in, is read as a cut in another branch
  or at the next level, so a level never closes even on a finite space.
  Fix it so that iterative deepening closes on a finite space (the note
  weighs not carrying cut entries across levels against starting a
  fresh level when one adds no memo entry); find the commit that lost
  the four (probe A4) if that helps the fix. Pin it with tests on small
  sequents of these shapes, no copy bound, under a work bound
  (deterministic), and add a few such rows to the target set without a
  copy bound. Judge it, against overfitting, on what was not tuned on:
  tune on `ILLTP-SYJ-01`, and report the rows and the distinct formula
  stems decided at 10 s on one thread on KLE, SYN/LCL and the cbv
  translation, before and after; both target sets against their
  oracles (verdicts must not move to wrong, and every change of
  counters explained). No verdict may change from decided to wrong,
  and every proof stays checked. The comparison's published counts
  change only with the SYJ206/212 rerun above; the rest waits for a
  second night of the comparison, which is the author's to ask for.
- **30c: the baseline and the release**, the rest of this prompt.
  The night or nights need the author's go, which they gave
  (2026-10-17), and then changed the timing the same day: no slot and
  no timer. The baseline starts as soon as 30c has it ready, detached
  (`bench/baseline.sh --detach`, not `--arm`), at any hour; the author
  says when the machine is not free, and the graceful-stop skill pauses
  it. A stopped one resumes from where it stopped.

## What step 29 left you

- **The comparison is public** (pushed 2026-10-16, the author's
  choice): README's "Beside the other provers", `bench/TOOLS.md` and
  `bench/comparison/2026-10-15/` cite it. The release's README keeps
  that section and its links true; the drafts to the tools' authors
  (`plan/notes/comparison-messages.md`) are the author's to send.
- **The CI job `Comparison`** (`.github/workflows/comparison.yml`,
  `workflow_dispatch` only) has never run; its first run is the
  author's. Whether BenchExec gets its cgroup on GitHub's runners is
  untested: the release notes do not promise it.
- **The baseline and the comparison are separate**: the baseline
  compares linlog with itself across steps (`bench/RESULTS.md`,
  `bench/COMPARISON.md`); the comparison compares it with other tools
  (`linlog-bench ranking`). Don't merge them.

## Goal

The third baseline, comparable with the second under the flags the
second ran under and with a pass under the new defaults; and everything
a release needs prepared, so that the author's own acts (the tag, the
publication) are the last and smallest part.

## What is fixed now

1. **The baseline**, by `.claude/rules/bench.md` and the lessons of
   steps 14 and 16: the estimate by arithmetic, a word to the
   supervisor with the estimate before the start, no probes beyond
   those named.
   `bench/COMPARISON.md` gains the third column. The pass under the
   default (`lltp-default`: every core, `--pool-after 0.1 --timeout 2`)
   answers what step 21 left open. On two cores the race of one thread
   and the pool lost one row of the old default
   (`AutoFlight_afcs_06_b_10_1`, three threads sharing the cores); the
   question is whether it keeps them all when the pool has fifteen
   threads beside the single one. Read its latest stops as well: at
   step 21's review the pool's choices spent up to 15 s on queued tasks
   after a stop on Petri nets, which the review fixed. The script now
   estimates eleven and a half hours and resumes a stopped baseline.
2. **The release, prepared**: version 0.1.0 in the manifests, a
   changelog, `CITATION.cff`, the crates' metadata and what docs.rs
   builds, a dry run of publishing, the release workflow, the Rocq
   library's opam file, as `plan/notes/distribution.md` lays out. The
   owner is the organization `linlog-prover` (D22): every address
   written is `github.com/linlog-prover/linlog`, where the repository
   has been since 2026-10-03. Pushing, tagging and publishing are the
   author's.
3. **The reminder** (the author, 2026-10-03): the final message of this
   step tells the author that `plan/notes/lltp-headers.md` is ready to
   send to the LLTP library's maintainers, and asks whether to send it.
   It also reminds the author that a rule on `main` against force
   pushes and deletion was to be added once a release is out (the
   author, 2026-10-03); the planning session adds it through `gh` on
   the author's word.
4. **README** as the face of a released tool: installation, a first
   proof, where the documentation and the benchmarks are.

## Deliverables

Thematic jj commits; `bench/results/DAY/`; `plan/reports/30-baseline-release.md`.
