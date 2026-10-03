# Step 23: what every session reads, short and true

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing. Read before you start:

- `plan/reports/17-assessment.md`: section 1.1 (the stale claims; those
  in the rules files and CLAUDE.md are this step's, the others step
  28's), 1.2's item on the rules files.
- `plan/README.md`: the Status entry of 2026-10-03 on the new order.
- `.claude/rules/claude-infra.md` (what loads when), every rules file,
  CLAUDE.md, README.md.

## What the earlier steps left you

`.claude/rules/core.md` is 2 463 lines, about 27 000 words, and loads
whole into every session and sub-agent that reads any file under
`core/`: a session that touches the SVG export reads the focused
engine's six hundred lines first. CLAUDE.md, which every session reads,
is 3 500 words, much of it a tour of the API that the rules files say
again. And every step since step 4 has run README's examples by hand
against the binary, because nothing runs them. The thorough audit and
refactor of the code comes once, in step 28, before the release; this
step does only what saves every session until then its tokens and its
wrong turns.

## Goal

A session reads what it needs for the files it touches and nothing
else, and what it reads is true.

## What to build

1. **The core rules file split by module path**: one file per area
   (for example the sequents with parsing and serialization, the
   forest and its reading, the proofs with the checker, the derivation
   view and interactive proving, the search's front door and memory
   bound, the focused engine, the parallel runtime, the net engine and
   proof nets, the exports, the benchmark inputs), each with the
   `paths:` of its module, so that a file loads the rules of its own
   module. Text is moved, not rewritten, except where it is wrong; a
   point that two areas need lives in one file and the other names it.
   Show in the report that nothing was lost: every paragraph of the old
   file is in exactly one new one.
2. **CLAUDE.md keeps what every session needs**: the commands, the hard
   rules, the conventions, pointers. The tour of the API moves to the
   rules files where they do not already say it.
   `.claude/rules/claude-infra.md` lists the new files.
3. **The stale claims** of section 1.1 that are in the rules files and
   CLAUDE.md, and any others found on the way, corrected against the
   code.
4. **README's examples run by a check**: a test or a flake check runs
   every console block of README against the binary and compares the
   output, so that no step runs them by hand again. A block whose
   output depends on the machine (a time, a copy bound reached under a
   time limit, a thread count) is marked as such and checked for its
   verdict line's shape only; pinned blocks name `--copies`,
   `--timeout` and `--deterministic` as they need.

## Constraints

- No change to any code's behaviour; the only code is the check of
  item 4.
- Every invariant the rules state survives the move.

## Verification

The checks of CLAUDE.md's table, `nix flake check`, and a table in the
report: the words a session loads when it reads one file of each
module, before and after.

## Deliverables

- Thematic jj commits.
- `plan/reports/23-session-docs.md`: the new files and what each
  covers, the table, the claims corrected.
