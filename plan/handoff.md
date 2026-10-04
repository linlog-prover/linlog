# Handoff: the planning session

You are taking over the planning and review session of the linlog
repository from an earlier Claude Code session that ran from 2026-09-29 to
2026-09-30. The author (Fabian, GitHub `flgrubm`, the repository under the organization `linlog-prover`) runs a stepped plan in
which every step is its own Claude Code session started from a prompt file
under `plan/`; this session is the one that writes those prompts, reviews
each step when the author reports it finished, amends the later prompts,
and keeps `plan/README.md` true. Nothing below is a task to start on your
own: read, then wait for the author to say what happened.

## Read first

1. `plan/README.md` from its first line to its last: the step table, how
   the steps are numbered, "Why these models and efforts", the commands,
   "Review protocol", the design decisions D1 to D22, and the Status log,
   which is the detailed history of every review so far.
2. `plan/conduct.md` (appended to every step's prompt), the prompts of
   the next steps (`18-bounded-proofs.md` on), `plan/later.md`, and the
   latest reports (`plan/reports/16-baseline.md`, `17-assessment.md`).
3. `CLAUDE.md` and your memory index. The memory directory of this
   project carries the author's standing instructions as notes written by
   the earlier session (commit identity and signing, testing, fonts,
   configurable output, the shared machine, jj habits); they apply to you
   unchanged.

If you need a detail of what was said or done before, the earlier
session's transcript is
`~/.claude/projects/-home-tux-Projects-own-linlog/3cc66a40-813d-48b7-ae1a-9c7c5553db7b.jsonl`
(large: search it, do not read it whole).

## What has happened

- The plan was written on 2026-09-29 from `proof-search-specifications.md`
  and the author's requirements: the best proof search per fragment of
  linear logic with the fragment detected and overridable, intuitionistic
  and affine modes, proofs in a checkable form, exports to LaTeX, Typst
  and SVG, Rocq certificates, proof nets as a first-class representation,
  interactive proving in the library behind cargo features, and prompts
  written for the model and effort that run them.
- Steps 1 to 13 are done, reviewed, accepted and pushed: the core data
  model, proof terms with an independent checker, the focused engine, the
  library API and CLI, proof nets and net search, exponentials,
  intuitionistic mode, interactive proving, LaTeX and Typst export, SVG
  export, Rocq certificates for NanoYalla, and parallel search.
- Step 14 (benchmarks) built its code (an LLTP reader, seventeen problem
  families, the `linlog-bench` harness, a baseline script) in a first
  session that never took its baseline (a reviewer's scratch program
  took 62 GB and the kernel's OOM killer took the terminal with it), and
  took it in a second session on the night of 2026-09-30
  (`bench/results/2026-09-30/`, `bench/RESULTS.md`), with a supplement
  of reruns on the night of 2026-10-01. The Status log of
  `plan/README.md` has the details.
- Since 2026-09-30 the author runs the step sessions from a second
  Claude account whose configuration directory is `/home/tux/.claude-2`
  (the same Unix user, so the working copy, the jj identity, the gpg
  agent and the repository's `.claude/` are shared, but not this
  account's memory notes: anything a step session must know goes into
  the repository, never only into memory). While a step session is
  running, do not edit the shared working copy: your edits would land in
  its `@`.
- On 2026-09-30 the plan was renumbered into whole numbers and
  restructured on the author's wishes: step 15 is the performance pass on
  the focused engine, step 16 the baseline again with the comparison,
  step 17 an assessment that puts the author's decisions to the author
  and then plans the steps from 18 with their prompts. `plan/later.md`
  holds unnumbered candidates (among them a code audit with refactoring,
  added at the author's request) and follow-up lists by area.
- `main` is pushed after every review; on 2026-10-03 it is at "Plan:
  review step 17".

## Where things stand

As of 2026-10-04 steps 1 to 24 are finished, reviewed and pushed. Step 17 assessed the project (`plan/reports/17-assessment.md`)
and, on the author's answers, planned steps 18 to 37; at the review of
step 21 the steps after 22 were put in a new order and renumbered to 38
(D23: a lean step 23 that saves every later session its tokens, the
audit and refactor once at step 28 right before the release at step 30,
a comparison with the other provers at 29, the Rocq library and the web
client after the release). The step table, the decisions D16 to D23 and
the commands are in `plan/README.md`, the prompts are `plan/18-…md` to
`plan/38-…md` (25 and 28 written in full; each later one says
what is fixed and is finished by you at the review its row names), `plan/later.md` says where every candidate and follow-up
went, and `plan/notes/distribution.md` has the facts on releases,
repositories and the organization. Step 18 rewrote the checker for
linear memory, made every proof pass it in every build and bounded what
any front end builds of a derivation; its review found the new checker's
counters wrapping (a crafted file passed as a proof of an unprovable
sequent) and fixed that itself. Step 19 made the time limit hold to
within half a second on one thread and on a pool, counted from the
command's start; what is left of it is the freeing of a full memo, which
step 20 took with the memory bound. Step 20 gave a search a bound in
bytes (a memo of records in chunks, the arena collected, "unknown" by
the bound as the last answer), the sequent a bound on its occurrences,
the check and the derivation the same memory bound, and replaced the
parser by a hand-written one without recursion (chumsky is gone); its
review guarded one more counter. Step 21 (run with Opus 5.5 at xhigh,
the author's choice) set what a call without flags does: the copy bound
deepens without end while a time limit of 2 s lasts, one thread searches
for 100 ms before a pool of the other cores joins it, "unknown" names
the limit, the time and the copy bound, and "unprovable" says why where
the counts tell. Its review found the pool honouring a stop up to 15 s
late on Petri nets (queued tasks of a choice built their workers before
their first poll), which the default met on every large net, and fixed
it. Step 22 (two sessions, Opus 5.5) gave every output one options
value with serde and the command a style surface, streamed the
derivation after the verdict, took the font out of LaTeX and Typst,
set Typst trees of any height in a layout of linlog's own, added a
compact view, and at the author's requests PNG and archival PDF. Its
review found that nothing bounds a render (a PDF of a derivation the
default bound admits was killed at 4 GiB; assigned to step 24) and
fixed five smaller defects, among them a premise gap that wrote 23.6 GB
past the time limit. Step 23 split the core rules into an index and
ten module files (`.claude/rules/core-*.md`, loaded per module), cut
CLAUDE.md to what every session needs, and made README's examples a
test (`cli/tests/readme.rs`); it ran unattended at night with
`--permission-mode auto` and a prompt section that told it to decide
instead of asking (the recipe is under the commands in
`plan/README.md`). Step 24 made `linlog prove` a batch (files,
directories, lists, streams, LLTP files; `search::batch` in the
library), bounded renders before they parse, and drafted
`plan/notes/lltp-headers.md`, whose countermodels and proofs the review
checked independently. Its review fixed the batch's memory default,
which ignored a control group's limit; before it, on the author's
request, the rustdoc was made to show the whole public API ("The
rustdoc is the library's manual" in `.claude/rules/core.md`). The next
command is step 25:

```nu
claude --model claude-opus-5-5 --effort high --name step-25 ((open --raw plan/25-ordinary-logic.md) + "\n" + (open --raw plan/conduct.md))
```

At its review, finish `plan/26-focused-engine.md`.

Models and efforts were re-evaluated on 2026-10-03 ("Why these models
and efforts" in `plan/README.md`): the author runs off a Max 20x plan,
where Fable may use at most half of the weekly limit; Fable 5.1 and
Opus 5.5 at `high`, `xhigh` only where the row says, never `max`.
Workflows (multi-agent orchestration) are used where a prompt says so
in as many words, which is the author's opt-in: the panels for changes
of the search (step 26 on) and the audit of step 28.

Profiling (the author asked on 2026-10-03, the planning session
recommended, the prompts say): a heap profile at the start of step 20,
a sampling profile at the start and the end of step 26, instruction
counts where a change is worth less than the day's noise; never a
standing requirement of every step. The profiles are read as text and
reported as tables; the author needs no flame graph.

What the author decided on 2026-10-03, in a line each: sensible
defaults, every one an option (D16); quantifiers will come and the
propositional case must not pay (D17); the API is free until the first
release (D18); the fastest engine per fragment and feature, by
measurement (D19); the Rocq library is `linlog` under `rocq/` (D20);
research and teaching are equal, the command first (D21); one
workspace, the web client in a repository of its own, everything under
the GitHub organization `linlog-prover` (D22), where the repository
has been since 2026-10-03 (`origin` is
`git@github.com:linlog-prover/linlog.git`). Remind the author at step 30
that the report of the wrong LLTP headers is ready to send, and that a
rule on `main` against force pushes and deletion was to follow the
release. The GitHub CLI is logged in with the author's rights over the
organization: ask before every administrative task through it
(CLAUDE.md), as the author asked on 2026-10-03.

The lesson of steps 16 and 17, for every review: run the command on the
largest problems yourself. Rows and reports hid a checker that takes
gigabytes, time limits missed by minutes and a wrong verdict at the
JSON boundary; each was found by a call, not by reading. The lesson of
step 18: where a step replaces a structure by counters or indices, ask
what each does at its limit and feed it a file that gets there. The
step's own differential test and its reviewer's random terms could not
reach 2³² copies; a proof file of 131 nodes did. The lesson of step 21:
run a default with the thread count a user's machine has. The step
measured on two cores, where the pool runs the two searches side by side
and no cubes; on four, its default missed the time limit by 15 s on
Petri nets, and the review's own earlier sweeps of the pool had left
the nets out. Sweep the whole library under the default on four cores
per run. The lesson of step 22: a bound on what is built is not a bound
on what is written or rendered. The derivation's bound admitted an SVG
that a PDF renderer took 4 GiB for, and a style value (a gap of four
billion columns) multiplied a tiny proof into 23.6 GB past its time
limit. Feed every new option and every new output its largest value
and its largest admitted input. Whenever the command's output path
changed, sweep the library through the command, not only the harness:
until step 24 lets the command read LLTP files, a scratch converter to
JSON sequents does it, as at step 22's review.

## What a review is

When the author says a step has finished (or was interrupted), do what
the earlier session did every time:

1. Read the step's report in full, then `jj log` and `jj diff --stat`
   from the last plan commit, and the commit messages and authors.
2. Read the code that matters, not only the report: the diff of every
   soundness-relevant file in full, the rest by its shape. Check that no
   comment or doc comment mentions the plan, its steps, the sessions or
   the prompts, and that new Rust, Nix, TOML and shell files carry the
   licence header while Markdown, JSON and the Claude files do not.
3. Run the checks yourself, in the background while you read:
   `cargo clippy --workspace --all-targets -- --deny warnings`,
   `cargo test --workspace`, `cargo hack check --each-feature -p linlog`,
   `cargo hack check --feature-powerset --depth 2 -p linlog`,
   `cargo deny check`, and `nix flake check` (`jj st` first, so nix sees
   new files).
4. Exercise what was built by hand on cases the tests do not pin (the
   CLI, rendered output looked at as images, certificates compiled
   against the kernel, a timing in release), within the rules for a
   shared machine.
5. Judge it against the plan and the decisions. Small, clear defects
   (a doubled error message, a stale reference, an unused import) are
   fixed here as commits of their own and named in the review; anything
   larger goes into a later prompt.
6. Amend the later prompts with what the step left (a "What step N left
   you" section is the usual form), amend decisions if one moved, append
   a dated entry to "Status" in `plan/README.md`, commit as
   "Plan: review step NN", then `jj bookmark set main -r @-` and
   `jj git push --bookmark main`. Pushing `main` after a review is
   standing practice in this thread.
7. Tell the author what was delivered, what you checked, what you
   accepted or changed and why, and give the next command.

When asked to re-evaluate models and efforts, check the model docs on the
day (the page names are in "Why these models and efforts") and argue from
what the steps so far showed.

## What the author has asked for, standing

- When you give the command for the next step, recap briefly what that
  step does first: its goal, its main items, anything notable (several
  sessions, a night of the machine). Asked on 2026-10-04.
- jj only, never git. Every commit is authored, committed and signed as
  `flgrubm@grubmueller.dev`; the gpg agent is used for signing and for
  nothing else. The earlier session kept the agent's cache warm with a
  background loop that signs a throwaway string every five minutes; that
  loop dies with it, so start your own as the memory note on the commit
  identity describes, while the author is present for the first
  pinentry.
- Licence headers only in Rust, Nix, TOML, shell, Python and YAML files,
  never in Markdown, JSON, the lock files or the Claude files.
- Documentation is self-contained: no comment or doc comment mentions the
  Claude Code sessions, the prompts, the plan or its steps.
- "Add tests and checks, but only test as necessary."
- README is kept true after every step; the rustdoc of both crates is one
  tree.
- Optional layers sit behind cargo features (D14).
- Whatever a user might vary in an output is configured through the
  library by one options value designed for the CLI, the web front end
  and other wrappers at once (D15). LaTeX and Typst output never sets a
  font; Euler math is the font where linlog draws itself (SVG, the web).
- The machine is shared: scratch programs in memory-capped scopes, and
  `cargo test`, `clippy` and `hack` too (a bare test run of step 22 froze
  the machine for ten minutes until the author forced it off), long
  runs detached as systemd user units, nothing on every core by day, no
  polling through a night.
- Prompts are written for the model and effort that will run them, say
  what is wanted and why, and leave the design to the session.

## Things that bit the earlier session

- A step's session can die with its terminal. If the author asks what
  happened to one, look at `journalctl` around the time, at the step's
  transcript under `/home/tux/.claude-2/projects/-home-tux-Projects-own-linlog/`
  (the second account; `~/.claude/projects/…` for sessions of this one),
  and at what it left in its scratch directory under `/tmp`, which is
  volatile: rescue data from there before it is lost.
- The gpg cache's two-hour maximum cannot be extended by the signing
  loop. To restart the clock while the author is present:
  `gpg-connect-agent 'CLEAR_PASSPHRASE --mode=normal <keygrip>' /bye`,
  then sign once (the memory note on the commit identity has the
  keygrip); without `--mode=normal` the command returns OK and clears
  nothing.
- A report can say more than was done. Check that files a report or a
  documentation file cites exist (`bench/RESULTS.md` was cited before any
  baseline had been taken).
- Edits made through the shell skip the formatter hook: run `nix fmt` on
  the files before committing. `jj squash` into a described commit needs
  `--use-destination-message`. The author may commit in the same working
  copy while you work: `jj st` before every commit.
- Relabelling anything in the plan touches many files: the prompts, the
  rules files, `plan/later.md`, the memory notes. The Status log and the
  reports keep the labels they were written with; the mapping is under
  the step table.
