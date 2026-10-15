# Step 29: linlog beside the other provers

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing. The step needs the
machine for a night, which the author gives; this prompt was finished at
the review of step 28 (2026-10-14). Read before you start:

- `plan/notes/comparison.md` (the tools, the method, where to run it).
- `plan/reports/16-baseline.md`, `17-assessment.md` (2.2: linlog against
  the Maude prover on the 1 342 problems its files cover),
  `21-defaults.md`, `24-batch.md`, `28-audit-and-refactor.md` (its
  "What steps 29 to 31 must know" and "Open questions for the author"
  first).
- `plan/notes/requirements.md`: the entries whose steps name 29 (R2,
  R18, R74, R75, R85, R151, R172, R184 to R193, R217, R245), each with
  its "State after step 28" and what is missing.
- `.claude/rules/bench.md`, `.claude/rules/ci.md`, `bench/baseline.sh`.

## Goal

A comparison a sceptical reader trusts: what each comparable tool can do,
as a matrix, and what each decides on the same problems under the same
limits, measured on the author's machine and reproducible by anyone,
with the numbers where linlog loses as plain as those where it wins.

## What step 28 left you

- **The library's side of a driver** is there:
  - the outcome's JSON: `version`, `verdict` (a name from
    `Verdict::NAMES`), `engine`, `reason`, `statistics` and `linlog`;
    read `engine` and `reason` as open sets;
  - the one-thread column, `--deterministic`, and the default column,
    `search::race`, which the command and the harness share;
  - one memory account for both searches of a race;
  - "outside the fragment" is an error (`Unsupported`, `EngineRefused`),
    never unknown;
  - a `proved` verdict is checked, while an `unprovable` one carries a
    `Disproof` that nothing checks before step 31: list a contradiction
    against one as unchecked.
- **First, the public walk** (R245, `api.md` 2.4 and 3.6), which step
  28 did not make public: `sequents::fmt::{Walk, Visit}` are still
  crate-private and `Reading::walk` does not exist. Make them public
  before the translators, as a commit of its own with the gate and the
  behaviour lock. The translators print through it, never recursing on
  a term, as R245 and R74's "Met when" say.
- **The harness's side is yours**: R185 (a `tool` column and a driver),
  R186 (peak memory per run), R188 (shards), R191 (the plots), R184 (the
  time each tool is charged), R189 (the CI job). Keep its columns read
  by name. `summary --against … --counters` and `summary --strict` check
  a run. The focused engine's oracle is `bench/targets/after-efficiency.csv`,
  the net engine's `net-after-efficiency.csv`.
- **A memory limit equal for every tool** (R18) holds for linlog's
  search, with one known gap. A proof of `&` rules closed by `⊤` can
  have `2ⁿ` branches, and the proof is not counted against
  `--memory-limit`: 26 pairs peaked at 4.6 GiB under 1 GiB. Under
  BenchExec such a run ends at the cgroup's limit, not as unknown. If
  the author has not decided the question by then (the step report's
  open question 5), report any such run as linlog's, out of memory.

## What is fixed now

1. **The feature matrix**: a row per feature (the fragments, the modes,
   automatic or interactive, the proofs output, proof nets, a checker,
   certificates, the exports, limits, threads, input formats, licence,
   activity), a column per tool of the note, each cell a tick, a cross
   or a short text, with its source and the date, and "unverified" where
   the tool could not be run. A short form in README, the whole in
   `bench/TOOLS.md`.
2. **The tools that run in batch**, each built by the flake at a pinned
   version as the LLTP library is (never vendored: several are GPL or
   carry no licence), with a translator from the LLTP syntax into its
   own and a driver in the harness: a command line, how its verdict is
   read, and its own time and memory limits the same as linlog's. A tool
   that cannot be built or run is a row of the matrix and not a column
   of the tables; the report says why.
3. **The method of the note**: measured with BenchExec (or, if it cannot
   run here, the harness with the same guarantees argued); problems
   solved within the limit as the ranking; cactus plots and pairwise
   scatter plots drawn from the CSV by a script; linlog on one thread as
   the main column, since the others are sequential, and its default as
   a second; problems outside a tool's fragment apart; translation time
   stated; contradictions between tools listed. The set: the LLTP
   library's intuitionistic and classical problems, the families, and
   what step 27 added from practice.
4. **The published numbers** come from a night on the author's machine,
   asked for with its estimate as a baseline is (`.claude/rules/bench.md`,
   the slot), and are committed like everything else. The hardware, the
   versions and the commands are on the page beside them.
5. **A CI job anyone can trigger** (`workflow_dispatch`): the same script
   on GitHub's hosted runners, split by problems so that every tool runs
   every problem of a job on that job's machine, uploading the rows,
   tables and figures as an artifact. It commits nothing. Actions pinned
   by SHA, as `.claude/rules/ci.md` says.
6. **A message to each tool's authors**, drafted in `plan/notes/`, with
   the matrix and the configuration their tool ran in, for the author to
   send before the comparison is published.

## Deliverables

Thematic jj commits; `bench/TOOLS.md` with its figures; the README
section; `plan/reports/29-comparison.md`.

## From the review (2026-10-16)

- **Verified** by the supervisor:
  - `nix flake check --keep-going` passed at szvmwors, the step's head
    with its report;
  - the ranking, counted again from `bench/comparison/2026-10-15/runs.csv`
    by a script of the supervisor's own (62 543 rows, 4 811 problems):
    the problems solved within 10 s per tool equal `ranking.md`'s for
    all thirteen columns, once each tool's wrong refutations are taken
    out (APLL's ILL system 1 908 less 1 296, its LL system and Click &
    coLLecT 43 less 12);
  - linlog has no wrong verdict: its nine contradictions of a known
    verdict are LLTP headers that say Non-Theorem where linlog proves
    with a checked proof (`KLE065+1`, `SYN001+1` and `SYJ212+1.001` in
    their three translations), items 1 to 9 of
    `plan/notes/lltp-headers.md`, whose proofs are recorded there;
  - APLL's `N` is its own claim of unprovability: in its source
    (`src/prover.ml`, `src/foc_ill_bwd.ml`, `prove_sequent`) `N` is
    printed only when the search ends without reaching its copy bound,
    and `-` when it reaches it. On
    `ILL/petri-nets/MCC/SmallOperatingSystem-MT0032DC0008_50_1.p` its
    own LLTP reader (`apll -ill -lltp FILE -t`) answers `N` in 0.097
    ms, while `linlog prove -i` proves it with a checked proof (the
    Horn engine), and the run's rows say the same. Counting its 1 296
    such answers wrong is right;
  - the public pages say what the limit costs: `bench/TOOLS.md` has the
    10 s budget and its effect on late tools, the Maude prover's 2018
    results at 300 s beside its results here, and linlog's default on
    the slower low-power cores; the author's pause of the run (21:54 to
    22:07) is in the step report only, as the author asked;
  - `.github/workflows/comparison.yml` is `workflow_dispatch` only, with
    `contents: read` and every action pinned by commit.
- **Not pushed**, on purpose: the comparison stays local until the
  author has sent the drafts in `plan/notes/comparison-messages.md` to
  the tools' authors, or decides to publish anyway. `main@origin` is
  step 28's end (nvwkumls); the step's nine commits are signed.
- **For the author**: the drafts; the first run of the job
  `Comparison`; a second night at 60 s on the 452 problems no tool
  solved (6 to 8 hours), which is their call.
- **Step 30 starts with the author's requests of 2026-10-15**, before
  its baseline: the tests pruned to the necessary ones, with mutation
  and coverage evidence, and CI cached through the public Cachix cache
  `linlog-prover` (the secret `CACHIX_AUTH_TOKEN` is set) with the test
  checks split per crate. The work moves to the author's other account
  with a handoff prompt from this planning session.
