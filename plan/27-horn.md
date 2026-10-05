# Step 27: Horn programs: an engine, coverability, and problems from practice

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing. The step takes two
sessions. Read before you start:

- `plan/reports/17-assessment.md`: 2.1, 2.5, 3.7, 3.15, 5.4.
- `plan/later.md`: "The !-Horn fragment through Petri-net reachability",
  "Problems from practice".
- `plan/reports/15-performance.md` ("The default bias"), `16-baseline.md`,
  `26-focused-engine.md`.
- `plan/README.md`: D8, D19.
- `.claude/rules/core.md` and the module files of the search:
  `core-search.md` (the engine interface, the dispatch table, the
  reference prover), `core-focus.md` (the Horn shape, the forward
  search), `core-parallel.md`; `bench.md` and `flake.md` for the suite
  and its runs, `cli.md` for `--engine`.

## Both sessions run unattended

The author starts each session and leaves it: nobody answers until the
planning session reviews it. The first ran overnight; the second runs
by day while the author is away (2026-10-05).

- **Never ask.** No question tool, no turn that ends waiting for an
  answer. Where a choice is open, take the most idiomatic, current
  best-practice option you can recommend, and record each with the
  alternatives you set aside in the report, under "Decided unattended".
  What the measurement decides (D19), it decides: the first session the
  default for Horn programs and whether the step ended there; the second
  whether coverability becomes the default in affine mode, and whether
  the Horn row keeps its scope or narrows.
- **How a turn ends.** A message without a tool call ends your turn, and
  then nothing runs until someone returns. So do not end one with a
  summary that announces the next step, an offer to go on, a list of
  decisions that block nothing, or because a milestone is done: put
  status notes in the same message as the next tool call. A turn ends
  when the session's part is done, or when something blocks every
  remaining item; then say what blocks it. The first session's part was
  reachability built, measured, decided and through its panels. The
  second's is coverability, the state equation and the qcover suite
  built, measured and decided, each change of the search through its
  panel, the closing runs done (the target set, the families, the
  library, the near-Horn programs of the review), and the report and
  its checklist committed. While a run, a workflow or a sub-agent you
  started is still going, wait for it rather than end.
- **The machine.** The measurements run detached on cores 2 to 5;
  builds, tests and your probes run capped on cores 6 to 11 (where
  `conduct.md` says 4 to 9); the panels' runs and their agents'
  programs on cores 12 to 15; cores 0 and 1 stay free. What
  `conduct.md` says to ask for first, a probe or a run the step does not
  name, you run without asking within these cores, capped, and detached
  when it takes more than a few minutes. `nix flake check` builds on any
  core, so never run it beside a timed run. Nothing outward-facing: no
  push, no `gh`.
- **Signing.** The author enters the passphrase just before starting
  you, and `max-cache-ttl` gives about two hours of signatures from
  then. Start the warm loop first, before any commit, under a name of
  the session's own (`step27b` for the second, `step27` for the first):

  ```sh
  systemd-run --user --unit=step27b-gpg-warm /run/current-system/sw/bin/bash -c 'while echo x | gpg --batch --pinentry-mode error --local-user flgrubm@grubmueller.dev --sign -o /dev/null 2>/dev/null; do sleep 240; done; touch /tmp/linlog-step27b-unsigned'
  ```

  It never opens a pinentry and leaves `/tmp/linlog-step27b-unsigned`
  once the cache is gone. Before every jj command that can write (a
  commit, a split, a describe, and `jj st` once files changed), check
  that the file is absent and that `echo x | gpg --batch --pinentry-mode
  error --local-user flgrubm@grubmueller.dev --sign -o /dev/null`
  succeeds (the agent's `KEYINFO` flag for the key is no guide: on
  2026-10-05 it read `-` while signing worked); from the first time
  either fails, run every jj command
  with `--config signing.behavior=drop`, which commits unsigned instead
  of waiting on a pinentry, and go on committing thematically. If a
  command hangs on signing all the same, run
  `/home/tux/.claude/hooks/unwedge-gpg-lock.sh` and repeat it with that
  option. End the report with the unsigned commits and the one command
  that signs them, `jj sign -r 'main@origin..@-'`.

## Goal

A sequent that is a Horn program (clauses under `!`, a marking, a goal:
a Petri net) is decided by an engine made for it, chosen by the dispatch
from the sequent's shape (D19): reachability by explicit-state search
with the proof built from the firing sequence, and in affine mode
coverability by the backward algorithm, which decides. The library's
1 594 nets that end at the time limit and the coverability suites of
software verification are what it is measured on.

## What is fixed now

1. **First session, reachability**: markings as count vectors,
   transitions indexed by their input places, a visited set, a proof
   term the checker accepts. It is the default for Horn programs only
   where it beats the forward focused search on the library's nets; if
   it does not, the step ends with its first session and says why.
2. **Second session, coverability**: the backward algorithm on upward-
   closed sets for affine mode, with its termination argument and a
   review by the panel that step 26's prompt describes (a workflow of
   three agents, each trying to refute it in one way); `Unprovable` from it is the suite's first
   refutation of an affine sequent with exponentials.
3. **The suite**: the 176 coverability instances of `blondimi/qcover`
   (Mist's `.spec` format; five suites; expected results in the files),
   fetched by the flake at a pinned commit as LLTP is, not committed,
   since the files carry no licence statement of their own; a reader for
   the format; parametric initial markings as `!p`.
4. **What it retires**: the focused follow-ups that only served nets
   (the unit of work, the restart from the frontier), if the engine
   takes the nets.

## What the earlier steps left you

- **An engine is a row** (step 26): every engine implements
  `search::Decide` (`admits`, `decide` returning an `Answer`),
  `prove_goal` alone builds a `Verdict`, and `search::DISPATCH` is a
  table of rows (fragment, modes, `Feature`, engine), each documented on
  `Engine` with the measurement behind it. The Horn engine is a row with
  a `Feature` of its own: the Horn shape as `schedule::chains` reads it
  from the goal's members (a clause used once counts as a step since
  step 26's panel). `Engine` is public, so the new variant is described
  on itself and named by `--engine`, and README says so.
- **Today a Horn program goes to the focused engine's forward search**
  (the factor bias, chosen in `schedule::plan`). `lltp-forward.csv`, the
  second baseline's run of that configuration (`--bias factors --copies
  30`), is an older binary: step 26 changed the nets' throughput (the
  zone's range: NeoElection twelve times the stable sequents, TCPcondis
  15 % fewer), so the comparison below runs both engines on this step's
  binary and cores.
- **The references cannot judge a refutation with `!`.** The committed
  reference prover (`search::reference`) and step 26's second one
  refute nothing whose search can contract, so they judge the new
  engines' proofs against their own and nothing else; the checker judges
  every proof. Add the engines to the reference test's
  `configurations`. The new engines' refutations (an exhausted finite
  set of markings in linear mode, the backward algorithm's answer in
  affine mode) have three judges instead: the qcover suite's expected
  results, the families whose verdicts are known (`counter`,
  `counter-over`, the Horn encodings of Partition and 3-Partition), and
  an independent check that the panel's counterexample agent writes for
  the purpose (a Karp–Miller coverability tree; for a bounded net, its
  markings enumerated), sharing no code with the engine.
- **Every refutation is a change of the search**, reviewed by the panel
  step 26's prompt describes, with the same models (the integers on
  Sonnet 5.5: name `sonnet` in the workflow, which
  `.claude/settings.json` maps to `claude-sonnet-5-5`; step 26's panels
  ran Sonnet 5 before that mapping).
- **The target set**: rows the dispatch sends to the new engine change
  engine and counters; the focused engine's own rows, forced, keep
  theirs, which `bench/targets.sh` shows.

## Measurement

By day, detached, in capped scopes on four pinned cores (2 to 5), on
this step's binary: the library's 3 137 Petri nets (`ILL/petri-nets`)
at 5 s, once with the new engine forced and once with the forward
focused search as `lltp-forward.csv` configured it, so that the two are
paired (about two hours for both); the 176 qcover instances with both
at the same limit; and the families. The default moves to the new
engine only where these numbers show it faster or deciding more (D19),
and the report gives the table.

## How the step runs

Two sessions on Opus 5.5 at `xhigh`, as the step table says, with the
planning session's review between them: the first ends with
reachability, its measurement and its decision about the default (or
with the reason the step ends there), and the second starts from that
report and its review. Keep the step's items as a checklist in the
report, each with its state and its evidence.

## From the review of the first session

- **The row costs refutations of unbounded nets.** The library's nets
  are all reachable, so the measurement saw only proofs. The review ran
  7 000 small random programs near the Horn shape at 1 s, the default
  against the focused engine forced: one to four atoms and clauses,
  clauses used once and under `!`, goals reached by a replayed firing
  sequence or random, with mutations at the shape's edge. There was no
  contradiction in any mode. But of the 3 751 the row sends to the Horn
  engine, 84 (intuitionistic), 80 (classical) and 53 (with Mix) that the
  focused engine refutes stay unknown, against 1 or 2 gained. They are
  nets whose markings grow without end while the goal is unreachable:
  in `!d, !((c * d * d) -o 1) |- (c * c)` nothing makes `c`.
- **So before this session ends the row loses none of them, or it
  narrows (D19).** The canonical refutation beyond exhaustion is the
  net's state equation: the target less the initial marking must be a
  non-negative combination of the transitions' effects, the clauses
  used once at most once each. Its infeasibility is certified by a
  Farkas vector. Whatever proposes the vector (a linear-programming
  solver in floating point, or exact elimination), checking it in exact
  integer arithmetic keeps the refutation sound. It goes beside the
  coverability work, which needs the same kind of argument in affine
  mode, and a panel reviews it. Measure it on such programs, generated
  by a test or a script of the session's, and on the families and the
  library.

## Verification

The checks of CLAUDE.md's table on every commit, and `nix flake check`;
the reference test with the new engines in it; the target set; a panel
for every refutation the new engines give; the measurement above.

## Deliverables

Thematic jj commits; the engines and their row of the dispatch; the
suite's flake package and its reader; a rules file for the new module,
with its paths and a row in `core.md`'s table; README for the engine and
the suite; `plan/reports/27-horn.md`.
