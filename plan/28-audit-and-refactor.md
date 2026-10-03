# Step 28: the audit, and the code in order for the release

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing. The step takes several
sessions: the audit, then one session per area of fixes, each followed by
a check round; every session starts from the reports before it. This
prompt is finished at the review of step 27. Read before you start:

- `plan/reports/17-assessment.md`: section 1 in full (1.1's stale
  claims, 1.2's refactoring list, 1.3's invariants, 1.4's tests), "The
  command, the harness and the flake as a maintainer finds them",
  section 4, 3.9, and the author's answers 4 and 5.
- `plan/README.md`: every decision, D15 to D18 and D23 above all; the
  reports of steps 18 to 27.
- `plan/later.md`: "Code audit and refactoring", "First-order linear
  logic"; `proof-search-specifications.md`, "First-order fragments".
- Every rules file under `.claude/rules/`.

## What the earlier steps left you

Twenty-seven steps by separate sessions built the library, the command
and the harness, each reviewed for its own correctness and none for the
whole; step 17 read it once as a maintainer would. The author decided on
2026-10-03 to put it in order once, here, right before the release,
rather than before the steps that add to it, so that the audit is not
paid twice (D23). Until the release the API may change freely (D18);
quantifiers will come, and the data model and the interfaces must have
their place without a second rewrite and without slowing the
propositional case (D17).

What the steps left for this one, besides section 1 of step 17's
report:

- **From steps 18 to 20**: the error family is scattered (`ShapeError`,
  `Unsupported`, `UnknownRule`, `NetError`, `ViewError`,
  `Error::Rejected`, `Error::Unchecked`, `Error::TooManyNodes`, and in
  the checker `Problem::Surplus`, a fault of the proof, beside
  `Problem::Memory`, a refusal and no verdict: a caller must never be
  able to read a refusal as "invalid"). A proof file and a session's
  state are read under the default occurrence limit whatever the caller
  asked, because `Deserialize` takes no options; the checker's pass
  takes a memory bound and no stop condition; the bounds of a search, a
  check and a view are three values a front end sets one by one.
- **From step 21**: `Verdict::Unprovable(Refutation)`, whose `Unbalanced`
  holds an atom and its name both, which the forest has;
  `Options::copies(Option<u32>)`; `Statistics::copies`, the larger of
  two searches' levels (30 under a bound of 3 on a Horn program);
  `Interactive::close_with`, which grafts any `Proof` that passes the
  checker against the goal's ids without asking whether it is a proof
  over the session's forest. In the command: the race of one thread
  and the pool is written twice (`cli/src/prove.rs`, `bench/src/run.rs`);
  its two searches each hold the whole `--memory-limit`, so a call
  without flags may hold twice it (2.05 GB measured on `SYJ206+1.016`
  in its `01` translation; half the bound each starved a wide memo, and
  one account both draw from is the question); `--jobs 2` with
  `--pool-after` runs three threads; the harness's `--pool-after` and
  `--timeout` panic on a negative or NaN value.
- **From step 26**: where a trail of bindings would go in the focused
  engine and which prunes assume ground atoms (its report).
- **From steps 22 to 27**: whatever their reports leave to "step 28" or
  to "the audit".

## Goal

At the release, the library is one a stranger can use from its
documentation, whose types say what they hold, whose errors are one
family and whose options are data with a wire form; the command, the
harness, the flake and every document say and do the same thing once;
and the code meets the author's rubric, agreed by reviewers that looked
for its faults independently.

## Stage 1: the audit (one session, Opus 5.5 at `high`)

1. **The rubric first**, `plan/notes/audit-rubric.md`: every criterion,
   each marked must-fix, should-fix or taste. The author's, of
   2026-10-03: very efficient and performant; an idiomatic project
   structure and code following current best practice; concise doc
   comments a human understands at first read; self-documenting data
   structures and functions; comments only where the code cannot say
   it; altogether, code that is pleasant to review. And the standing
   ones, from CLAUDE.md, `plan/conduct.md` and the decisions: bounded
   time and memory; every default a named option (D15, D16); a number
   soundness rests on argued at its declaration or refused at its
   limit; tests as necessary; dependencies that earn their place; no
   recursion over a formula; no comment naming the plan; licence
   headers; README true.
2. **What a machine can check is a check, not a reviewer's token**: the
   clippy lint groups chosen (and the ones not chosen, with the reason),
   rustdoc's lints, unused dependencies, a spell check, the greps for
   comments that name the plan and for licence headers, all in `nix
   flake check`, so that they never regress.
3. **The audit as a workflow** (the author's choice, 2026-10-03: use a
   workflow for it). Six reviewers, one lens each, each reading the
   whole repository through its lens against the rubric:

   | lens | model, effort |
   |---|---|
   | soundness, and the integers and limits it rests on | Fable 5.1, `high` |
   | performance (profiles read as text, never drawn) | Opus 5.5, `high` |
   | the API, idiom and structure | Opus 5.5, `high` |
   | doc comments, names, comments | Opus 5.5, `medium` |
   | tests: missing for a stated behaviour, or in excess | Sonnet 5.5, `high` |
   | coherence of README, CLAUDE.md, the rules, the help texts with the code | Sonnet 5.5, `high` |

   Round one is independent: a finding has a file and a line, the
   criterion, a severity, its evidence (a quote, a failing test, a
   measurement) and a proposed fix, as a JSON schema. Round two is a
   cross-examination, not an amendment: each finding goes to two
   reviewers other than its author, who try to refute it with evidence;
   it falls when both do. Each reviewer then gets the surviving list and
   adds what its own lens missed. Duplicates are merged by code, and
   near-duplicates by one agent. Matters of taste are not voted on: they
   go to the author as a short decision list. No reviewer runs at `max`.
   An agent that runs programs (the performance lens may run the release
   binary and callgrind on small rows; the soundness lens may run the
   tests) gets the rules of `plan/conduct.md` in its prompt: named
   cores, a memory-capped scope, bounded runs, nothing by day that the
   step does not name. Keep a run under 25 agents and log what a bound
   drops.
4. **`plan/reports/28-audit.md`**: the findings by area and severity,
   each with its evidence, the decision list, and what the run cost
   (agents, and tokens by model). The session ends there, and the author
   answers the decision list; the answers become rules under
   `.claude/rules/`, which later rounds judge against.

## Stage 2: the fixes (one session per area, in this order)

Each session takes the rubric, the decision list and its area's
findings, and the items below. Fixes are sequential in the one working
copy.

1. **The library's API and data model** (Opus 5.5 at `high`; Fable 5.1
   at `high` for the checker). A design note first,
   `plan/notes/api.md`: the public surface after this step, and how
   first-order logic enters it (atoms as predicates over terms, binders
   in the arena, a substitution beside the forest, witnesses in proofs),
   with what stays untouched for the propositional case and how that is
   measured; written as a workflow of three independent drafts from
   different angles, judged, and synthesised. Then: what is `pub` and
   need not be; names that differ between neighbours (the `Engine`
   structs beside the enum, the types called `Rule` or `Rules`);
   `#[non_exhaustive]` and builders applied evenly; `Mode` where a
   `bool` stands for it; one rule for taking or cloning a forest; one
   numbering of inferences or a map between the two; options with a
   wire form for every options value (the command, the batch mode and
   the web client are then callers of the same values, D15); one family
   of errors with a serializable form; one printer of formulas and of
   two-sided sequents over `Notation`; `Rule` as a classical rule and a
   position, in a file of its own; `proofs/interactive.rs` along its
   seams; the lint allowances of `core/src/lib.rs` removed; the
   invariants of section 1.3 as types, assertions or tests where one
   sentence of code does it.
2. **The search** (Fable 5.1 at `high`): the audit's findings under
   `core/src/search/`. A commit that claims no change of the search
   keeps the target set's counters; one that changes it is reviewed by
   the panel step 26's prompt describes.
3. **The command, the harness, the flake and the documents** (Opus 5.5
   at `high`): `prove` and `interact` share their search flags, the
   building of `Options`, the stop closure and the verdict line; a new
   output format is one place; the help texts that are wrong are right;
   `interact`'s words are read with quoting, its ASCII rule names are in
   its help, and `--file -` is refused; a closed pipe keeps the verdict;
   `--output` into a missing directory fails before the search; exit
   statuses beyond 0 to 3 are documented or removed; a session's sequent
   is read under the limit as `prove`'s is. Step 21's leftovers above.
   Tests for the stated behaviours that have none (`--timeout`, Ctrl-C,
   `--jobs`, `--bias`, `--memo-limit`, `--output`). The harness: tests
   for `summary` and the comparison, the `bench` check failing on an
   `error` or `crash` row, one configuration label, a table of the
   counters in `summary`, a late verdict kept apart, the columns read by
   name, `baseline.sh`'s hard-coded second baseline as parameters,
   `reruns.txt` and the repeated experiments retired or regenerated. The
   flake and CI: `cargo doc --document-private-items` builds, the
   fragments the command prints by default compile in the `export`
   check, the declared systems are those CI builds, the constants that
   name pinned versions are checked against the pins. Every stale claim
   of section 1.1 that step 23 did not correct, and the spec's two wrong
   statements moved to its errata.

## Stage 3: the check rounds

After each fix session, a workflow of the same lenses one effort level
lower: each finding of that area goes back to the lens that raised it,
which answers fixed, partly fixed, not fixed or regressed, with
evidence; and the lenses read the diff since the audit, and only it,
for new findings. The step is done when no confirmed must-fix or
should-fix finding is open and the decision list is answered, after at
most three rounds; what remains goes to `plan/later.md` with its reason.

## Constraints

- The command's behaviour, every JSON form and every snapshot stay as
  they are unless a finding requires a change; each such change is a
  commit of its own and listed in the report.
- A commit that claims no change of the search keeps `nodes`, `splits`,
  `memo_hits` and `memo_entries` of every decided row of
  `bench/targets.sh` identical, and pinned CPU time on its rows over a
  second within two percent.
- The CSV columns of the baselines stay readable.
- Dependencies only where they earn their place (D10).
- No quantifiers are added: the note says what the first step of adding
  them is. The Rocq library (step 31) comes after this step and takes
  the proof term as it leaves it.

## Verification

Every check after every commit that touches code: clippy, the tests,
both `cargo hack` runs; `bench/targets.sh` after any commit that touches
`search/`; `linlog-bench run --all-families --timeout 5`; `summary
--before` on the committed baselines reproducing `bench/COMPARISON.md`;
`nix flake check`; `nix build .#doc`, and the rustdoc front page read as
a stranger would read it.

## Deliverables

- Thematic jj commits, each of which builds and passes alone.
- `plan/notes/audit-rubric.md`, `plan/notes/api.md`.
- `plan/reports/28-audit.md` (stage 1), and
  `plan/reports/28-audit-and-refactor.md`: the surface before and after,
  what was renamed or removed (a table a later session can search), the
  first-order plan in a page, the check rounds, decisions, deviations,
  open questions, what steps 29 to 31 must know.
