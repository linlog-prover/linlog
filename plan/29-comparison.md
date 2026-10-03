# Step 29: linlog beside the other provers

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing. The step needs the
machine for a night, which the author gives; this prompt is finished at
the review of step 28. Read before you start:

- `plan/notes/comparison.md` (the tools, the method, where to run it).
- `plan/reports/16-baseline.md`, `17-assessment.md` (2.2: linlog against
  the Maude prover on the 1 342 problems its files cover),
  `21-defaults.md`, `24-batch.md`, `28-audit-and-refactor.md`.
- `.claude/rules/bench.md`, `.claude/rules/ci.md`, `bench/baseline.sh`.

## Goal

A comparison a sceptical reader trusts: what each comparable tool can do,
as a matrix, and what each decides on the same problems under the same
limits, measured on the author's machine and reproducible by anyone,
with the numbers where linlog loses as plain as those where it wins.

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
