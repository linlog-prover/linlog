# Step 35: engines for MLL and IMLL: net pruning, routing, essential nets

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing. The step takes two
sessions; this prompt was finished at the review of step 30
(2026-10-18), whose baseline it is measured against. It says what is
fixed.

Read `plan/later.md` ("Net-engine pruning and routing for repeated
literals", "Essential nets for IMLL"), `plan/reports/06-net-search.md`,
`08-intuitionistic.md`, `17-assessment.md` (2.4, 3.3, 3.5, the author's
answer 7), `26-focused-engine.md`, `30-baseline-release.md` (session
30c), `plan/README.md` (D8, D19), the spec's sections on MLL-Net and
IMLL-Net, `.claude/rules/core-nets.md`, `core-search.md` (the front
door and its dispatch) and `bench.md`, and `bench/TARGETS.md`.

## How the step runs (2026-10-18)

Two sessions, one after the other, each reviewed by `planning` before
the next starts, both Opus 5.5 at `high`. Step 31's sessions run beside
them on the same machine (see "The machine" below).

- **35a: the net engine's pruning and the routing** (items 1 and 2).
- **35b: essential nets for IMLL** (item 3), the engine and the
  drawing.

## What step 30 left you

- **The engine is the release's** (0.1.0, prepared by session 30c and
  published by the author): a change of a public item is an addition
  or waits for 0.2.0, as `CHANGELOG.md` ("What is promised") says, and
  each session adds its lines under `## [Unreleased]`. Which engine
  decides a sequent, and how fast, is not promised.
- **The third baseline** (`bench/results/2026-10-17/`, `bench/RESULTS.md`,
  `bench/COMPARISON.md`) is the measurement this step starts from. Its
  `engines` runs (`engines.csv`) put the focused engine ahead of the net
  engine on every row both decide: `wide-m1` to `wide-m4` at 256
  literals 2.8 to 4.0 ms against 9.0 to 9.8 ms, and at 2 048 literals
  184 to 211 ms against 586 to 652 ms (the recursion limit no longer
  stops the focused engine there); the Partition tables and the MLL
  3-Partition refutations microseconds against seconds or the 60 s
  limit. The dispatch still routes `wide-m1` and `wide-m2` to the net
  engine (`families.csv`, `engine` `net`), so today the net engine
  holds rows it loses. `parallel-net.csv` and `period-*.csv` are the
  cubes on many threads and the test period.
- **The oracle** of the net engine's target set is still
  `bench/targets/net-after-efficiency.csv`, and the focused engine's
  `bench/targets/after-efficiency.csv`.

## The machine (two steps at once)

Step 31 runs beside this step. The cores are split, and everything the
Nix daemon builds is serialised with every timed run through one lock
file, since the daemon builds on any core:

- **Timed runs** (`bench/targets.sh`, `bench/net-targets.sh`, the
  harness's runs for a dispatch row or the IMLL comparison) on cores 2
  and 3, the cores those scripts pin, each under the lock, held until
  the run's unit ends:
  `flock /tmp/linlog-machine.lock bash -c 'bench/net-targets.sh LABEL && while systemctl --user is-active -q linlog-net-targets; do sleep 30; done'`
  (in a background task; the harness's own runs likewise, in a capped
  scope with `taskset -c 2,3`). The runs this prompt names need no
  further word; any other probe is asked for through `planning`, with
  its purpose, duration and cores.
- **Builds, tests, the gate and clippy** on cores 4 to 7, in capped
  scopes (`systemd-run --user --scope -p MemoryMax=8G -p
  MemorySwapMax=0 taskset -c 4-7 …`, `CARGO_BUILD_JOBS=4`), never
  beside a timed run of your own on 2 and 3 that it could slow.
- **`nix flake check` and `nix build`** under the same lock:
  `flock /tmp/linlog-machine.lock nix flake check`. Step 31 holds it for
  its Rocq builds too, so a wait for it is expected.
- **Mutation runs** (`mutants/run.sh`, cores 6 to 11) only on
  `planning`'s word, which arranges the cores with step 31.
- **Signing**: the passphrase's cache lasts two hours at most. Test a
  signature before each commit; while it fails, commit with
  `--config signing.behavior=drop`, and `planning` signs later. When
  `planning` signs, it rewrites the commits under your workspace: if a
  jj command then says the working copy is stale, run `jj workspace
  update-stale` and go on.

## Goal

The author's aim from the start (D19): each fragment has the engine made
for it where that engine is the fastest, picked automatically. For
unit-free MLL that is the net engine wherever it wins, and for IMLL an
essential-net engine if it wins over the embedding.

## Fixed now

1. **The net engine's pruning**: the leaf symmetry break for pure `⊗`
   and `⅋` trees of equal literals, a per-atom balance over the
   skeleton's components, the sound symmetry break for equal compound
   conclusions; each with its argument and a differential review against
   brute-force enumeration, as step 6 had.
2. **The routing feature**: "no two equal literals under one pure tree"
   or whatever the harness shows separates the two engines, in the
   dispatch table of step 26, with the measurement beside the row. The
   honest starting point (step 17): the focused engine is within a
   factor of three of the net engine on its own case and ahead from 256
   literals on, so the net engine has to earn every row, and a row it
   does not earn stays the focused engine's.
3. **Essential nets for IMLL**: Lamarche's polarized structures over the
   reading, correctness by directed acyclicity and the dominator
   condition (Murawski and Ong), the incremental search with a
   transitive closure and an undo log (Moot, 2008), compared with the
   embedding on the baseline's IMLL rows before it becomes a default;
   and the drawing of an essential net, which teaching wants whatever
   the measurement says.

Opus 5.5 at `high`; every change of a search or a criterion is reviewed by
the panel step 26's prompt describes, its argument read on Fable 5.1 at
`high`. A change of the net engine's search changes the counters of
its target set by design: the verdicts must not change, and a new
oracle, like a change of the behaviour lock's `--stats` lines, is a
commit of its own that says why.

## Deliverables

Thematic jj commits in each session's own workspace
(`../linlog-step35a`, `../linlog-step35b`, from the head `planning`
names); `plan/reports/35-mll-engines.md`, one section and a checklist
row per session; README's "What exists and what is planned" and the
changelog's `[Unreleased]` kept true. Nothing is pushed.
