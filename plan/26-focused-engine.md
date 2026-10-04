# Step 26: the focused engine in order

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing. Read before you start:

- `plan/reports/17-assessment.md`: section 1.2 (the refactoring's items
  1 to 4, 9, 13 and 14, D8), 1.3, 1.4, and the author's answers 4, 5
  and 7.
- `plan/reports/15-performance.md` ("Constant factors", "The reviews"),
  `19`, `20`, `21`.
- `plan/later.md`: "Follow-ups: the focused engine".
- `plan/reports/24-batch.md` (the batch, the pool kept across
  searches) and `25-ordinary-logic.md` (the ILTP run, its section on
  termination, and "From the review").
- `plan/README.md`: D7, D8, D17, D18, D19.
- `.claude/rules/core.md` (the index and the crate-wide rules) and the
  module files of the search it names: `core-search.md`,
  `core-focus.md`, `core-parallel.md`, `core-nets.md`, `core-batch.md`;
  and `core/src/search/**` in full.

## Goal

The focused engine as one reader can hold it, with nothing that a
refactoring can break in silence; and a dispatch that is a table: for
each fragment, mode and feature of a sequent the engine that a
measurement shows fastest (D19), with one interface every engine
implements, so that steps 27, 35 and 37 add a row and not a special
case.

## What is fixed now

1. **The cut and dependency flags become values** a step of the search
   returns; with them each rule is written once for one thread and for
   the pool, and the worker is built in one place.
2. **The file along its seams**: the scheduling of the two searches
   (out of the parallel module), the arena, the split search, the
   scratch pools, the tests.
3. **One engine interface** and one place that builds a `Verdict`;
   options an engine does not honour are refused or documented, not
   ignored; `prove_goal` takes its goal in any order, as it says.
4. **The dispatch as data**, with the feature that picks each row named
   and the measurement behind it cited; the atom bias out of the forest.
5. **The hot spots, from a profile taken anew.** Step 15's profile
   named three (the memo key hashed once, an insert without an
   allocation, the canonical key only where a member is renamed), but
   steps 19 and 20 changed what it measured: the ranking of the copies
   is linear now, and the memo's keys were laid out again. So the step
   begins with a sampling profile (perf from the flake's nixpkgs, a
   release build with debug symbols set through the environment, pinned
   to cores of one speed and capped, as step 15 took it; read as text,
   by function and by call path: no flame graph or other drawing is
   wanted, the author does not need one) of every row of the target set
   that takes over a second, and works from its ranking; it ends with the same profile, and the report shows the
   two side by side. For a change worth a few percent, times by day are
   too noisy on this machine (one-thread rows whose code did not change
   moved by −6 to +4 % between two runs of step 19): compare instruction
   counts instead (valgrind's callgrind, through the `new-tool` skill)
   on a row small enough to run under it, about fifty times slower.
6. **From the follow-ups**: the Mix prune (`n·2^n` for `3^n`), a loop
   for chains of free splits, the Horn test on the goal's members. And
   from step 19's review of the pool, which item 1 is the place for: a
   premise's failure at a `&` cancels the other only once its worker has
   left its nested scopes, so a stolen task of the sibling runs on
   uncancelled (seen once as an answer that came only with the caller's
   stop); an error of one premise cancels the other, so a pool answers
   "recursion limit" where the other premise would have failed and one
   thread answers "unprovable"; and the differential run of the pool
   against one thread is repeated with recursion limits of 4 to 16,
   since limits of 24 to 63 were reached by 10 of 369 057 random
   sequents. From step 21 and its review: a worker copies the branch's
   stack of keys and hashes every key again (`Spawn::worker`), the
   forest's width times the depth for every task of a choice. The
   review made a task poll before it builds one (a stop on a Petri net
   came 15 s late), but a task that runs still pays it. On
   `SYJ204+1.014` in its `01` translation a pool of two visits 66
   million stable sequents without a proof, where one thread proves it
   in 16 million. `LCL181+1` in the ILLTP library is never decided by
   one thread: it deepens past a million levels in two seconds, every
   one of them cut. A pool refutes it at level 32 to 565, so it is the
   order of the memo's cut entries against the loop check that one
   thread loses on. Step 22's review met a second: `SYN393+1` in `cbn`
   deepens to 2.6 million levels in 10 s on one thread, and two threads
   do no better, where a pool of four from the start refutes it in
   0.7 ms; the default on four cores (one thread, then a pool of three
   beside it) does not within its 2 s, the default on eight does in
   0.1 s, and the step-21 sweep on four cores had it refuted in 0.24 s,
   so which pool decides it is a matter of the interleaving. Without a copy bound the backward search of the
   default bias keeps its share of the work for good, where a bound
   used to end it and hand the core to the forward one
   (`SYJ212+1.014` in `cbn`). Each of these is a measurement to take,
   with the profile of item 5, before anything changes.
7. **Ready for quantifiers** (D17): the report says where a trail of
   bindings would go and which prunes assume ground atoms, for the design
   note of step 28 (`plan/notes/api.md`) to take up.

## What the earlier steps left you

Steps 22 to 25 did not change the search, but they changed what calls
it and what reads its proofs.

- **The batch** (step 24, `core-batch.md`): `search::batch` decides many
  sequents under one `Options`, across the cores with one thread per
  sequent (as `--deterministic`), or within them on a `search::Pool`
  kept across searches. The one interface of item 3 is what the batch
  calls, and a pool built once stays usable from one search to the next.
- **Ordinary logic** (step 25, `core-ordinary.md`): `linlog::ordinary`
  decides classical logic through affine MALL (the focused engine, no
  copy bound) and intuitionistic and minimal logic through ILL (the
  two-sided engine, deepening), and reads each proof back as LK or LJ
  from its derivation. A change of a proof's shape (which inferences a
  step builds, which premise keeps the goal) is caught by
  `ordinary::tests::decides_and_reads_back` and by the ILTP run with
  `--output`, which reads back and checks every proof; the counters do
  not see it. The ILTP images are a workload of their own for item 4's
  measurement: 274 problems per translation (`nix build .#iltp`, then
  `linlog prove --logic intuitionistic --translation cbn|cbv|01 --file
  bench/iltp/Problems`, and `--logic classical`), with step 25's counts
  as the before.
- **From step 25's review**, for item 5: `focus::Engine::initial` in
  affine mode compares every pair of literals, and polls no stop.
  `a₀, …, aₙ ⊢ aₙ` with `-a` takes 32 ms with 10 000 atoms, 0.37 s with
  30 000 and 3.05 s with 100 000, past a 2 s time limit, on both threads
  of the race. The forest's lists of each literal's occurrences, which
  `dual_in` reads, would make it linear.
- **The rules are per module** (step 23): what a later session must know
  of the engine goes into `core-focus.md`, `core-search.md` or
  `core-parallel.md`, and a file this step splits off gets its paths
  there and, as a new module file, a row in `core.md`'s table.
- **The rustdoc is the manual** (`core.md`): each engine is described on
  its `Engine` variant, so item 4's table is said there too, with the
  measurement behind each row.
- **Termination on dyadic sequents is not this step's.** Step 25's
  report measured what a loop check that refutes would decide (35 ILTP
  problems, all Non-Theorems), and it is a step of its own if the author
  wants one (`plan/later.md`). Item 6's `LCL181+1` and `SYN393+1` are
  where the engine's cut entries meet its loop check: measure them, as
  item 6 says.

## The oracle

Every commit that claims no change of the search leaves `nodes`,
`splits`, `memo_hits` and `memo_entries` of every decided row of
`bench/targets.sh` identical; a commit that changes the search says so,
is measured, and is reviewed by a panel before it is called done. Use a
workflow for each such review (the author's choice, 2026-10-03): three
agents, each in a fresh context that holds the diff, the claim and the
code but not your reasoning, each trying to refute the change in one
way; the change stands when none does.

- **Counterexamples** (Opus 5.5 at `high`): both references (below) and
  the generators against the changed engine, on more sequents and
  larger ones than the committed test runs, within the scope's bounds.
- **The argument** (Opus 5.5 at `xhigh`): the invariant the change
  relies on, stated from the code before your argument is read, then
  checked line by line at every site that relies on it.
- **The integers and limits** it rests on (Claude Code's `sonnet`, Sonnet
  5 on 2026-10-04, at `high`), each
  driven to its limit by an input.

A refutation carries its witness (the sequent with both verdicts, or the
input and the line), and you reproduce it before acting on it. The three
are all Anthropic models, whose errors correlate (models of one
developer more than others), and a model judges what it wrote more
kindly: so the panel's independence comes from its methods, executable
witnesses, two references and an argument derived before it is
compared, not from its members. An agent that runs programs gets the
rules of `plan/conduct.md` in its prompt: named cores, a memory-capped
scope, bounded runs. Pinned CPU time does not rise.

The counters say that a search is the same search; they say nothing of
a verdict that was wrong before and after. So this step also leaves a
**reference prover in the repository** (the author, 2026-10-03, on the
planning session's recommendation), before it changes anything: a
test-only module, as `core/src/proofs/oracle.rs` is for the checker,
that decides a small sequent by the plain rules of the unfocused
calculus, exhaustively, with no polarity, no bias, no count prune and
no memo keyed as the engines key theirs, classical and two-sided,
linear and affine, with and without Mix, and with a copy bound of its
own for the exponentials. It shares no code with any engine. A
committed test compares it with every engine on the sequents
`search/generate.rs` makes, provable ones and mutants, at sizes it
finishes in the time the neighbouring tests take, and asserts what the
contract allows: never "proved" against "unprovable", in either
direction, and no "unprovable" from an engine where the reference finds
a proof within its bound. Until now such a reference was written anew
by a step's reviewer and thrown away (the unfocused two-sided prover
that step 8's engine was compared with on 60 000 sequents is the one
the rules file names), so none of those runs can be repeated on the
engine as it is today; the checker guards a
wrong "proved" in every build, and only the engines themselves guard a
wrong "unprovable". Steps 27, 35 and 37 add their engines to this test.
Before the engines are compared with it, a fresh-context reviewer (a
sub-agent on your model) reads the reference against the calculus in
the rules files and decides a dozen small sequents with it by hand: the
oracle is checked before it judges. Since that reviewer, you and the
planning session that reviews you are one model, whose blind spots are
shared, two checks rest on no model's reading: the reference agrees
with every verdict the families know by construction
(`linlog::families`, at the sizes it finishes); and a handful of
deliberate faults made in a scratch copy of it (a rule dropped, a side
condition flipped, a copy bound off by one) each make the committed
comparison test fail, the report listing which. The engines it is
compared with were written, nearly all of them, by sessions on Fable
5.1, so the comparison itself sets two models' work side by side. And in the session that changes the
search, a fresh-context agent (Opus 5.5 at `high`) writes a second
reference of its own from the calculus, sharing no code with the
committed one, and keeps it outside the repository; every panel's
counterexample agent runs both. The committed one is for repeating, the
second for independence.

## How the step runs

Two sessions, both on Opus 5.5 at `xhigh`: Fable's share of the weekly
allowance is spent (2026-10-04), and `xhigh` is the level Anthropic
names for agentic coding of more than half an hour.

- **The first session** builds the reference prover and its test before
  anything else changes, has it reviewed as above, takes the profile at
  the start (item 5) and the measurements of item 6, and does items 1
  to 3, the refactoring that claims no change: every commit keeps the
  target set's counters. It ends with the report's first part, and the
  planning session reviews it before the second session starts.
- **The second session** starts from the report and its review: first
  the pool against one thread at recursion limits 4 to 16 again, on the
  code as it stands (it ran before item 1 rewrote how a pool runs a
  choice), then items 4 to 6 (the dispatch as data, the hot spots, every
  change of the search with its panel), the profile at the end, and
  item 7.

Keep the step's items as a checklist in the report as you go, each with
its state and its evidence. A session ends when its part of the list is
done or something on it is blocked, not at a milestone to report it:
status notes go with the next tool call.

## What waits

Whether the engine's recursion becomes an explicit stack: only if the
web client (step 32, after the release) shows that it needs a search
that can be suspended.

## Verification

The checks of CLAUDE.md's table on every commit, and `nix flake check`.
The target set (`bench/targets.sh LABEL`, two cores, detached) against
`bench/targets/after-bias.csv` for every commit that claims no change
of the search; for every one that changes it, the measurement and its
panel. The reference prover's test. The pool against one thread with
recursion limits of 4 to 16 (item 6). The profile at the start and at
the end (item 5). The ILTP library per translation and classically
through the command's batch with `--output`, set beside step 25's
table. The library through the batch by default on four cores, set
beside step 24's review (4 512 answers, 2 193 proved, 142 refuted).
Every run in a memory-capped scope on named cores, the long ones
detached.

## Deliverables

Thematic jj commits, each building and passing alone; the reference
prover and its test; the rules files of the search brought up to date
(and `core.md`'s table where a file is added); README where the
command's behaviour changes; `plan/reports/26-focused-engine.md` with
the two profiles side by side, the dispatch table and the measurement
behind each row, each panel's verdict, the target set's counters before
and after, the ILTP and library runs, and item 7's note.
