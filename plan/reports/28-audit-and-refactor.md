# Step 28 report: the audit, and the code in order for the release

The step runs as one session per stage and per area of fixes, each
started and reviewed by the planning session. This file holds the
step's checklist, which every session keeps current, and what the
supervisor said to each session. The stage reports are
`plan/reports/28-baselines.md` (stage 0) and `plan/reports/28-audit.md`
(stage 1); the step's own report grows here once the fixes start.

## The checklist

States: `open`, `running`, `done`, `paused`, `blocked`. Each item names
its evidence (a commit, a file, a command's result) once it is done.

### Stage 0: requirements, baselines and gates (session `step-28`)

| item | state | evidence |
|---|---|---|
| 0.0 signing loop `step28-gpg-warm` | done | started 20:50, signatures work |
| 0.1 register of later requirements, `plan/notes/requirements.md` | running | workflow `wf_a3156457-6a2` (14 extractors, merge, critic, fold-in; Sonnet 5.5 at `high`) |
| 0.2 behaviour lock: fixtures and test | done | 908f91f7: `cli/tests/lock.rs` (73 calls), `core/tests/lock.rs` (36 JSON lines); stable over three runs |
| 0.3 journeys under callgrind, validated against wall-clock | running | journeys and counts in 1ae95c4c; the timed validation waits for a quiet machine (after the mutation and fuzz runs) |
| 0.3 ratchet: ceilings, flake check, lowering command | done | 1ae95c4c: 20 journeys, `bench/ceilings.csv`, `nix build .#checks.x86_64-linux.ratchet` passed |
| 0.4 mutation testing wired in (`new-tool`), scope and command committed | done | f602c7e9: `mutants/run.sh`, `.cargo/mutants.toml`, profile `mutants` |
| 0.4 mutation run, batches per target file | running | unit `step28-mutants` since 21:03 on cores 6–11; `check` done (33 surviving, 1 653 s) |
| 0.4 fuzz targets wired in (`new-tool`), one per untrusted reader | done | f602c7e9: nine targets under `fuzz/`, nightly 2026-09-25 in the `fuzz` devshell |
| 0.4 fuzz runs, each until coverage stops growing | running | unit `step28-fuzz` since 21:12 on cores 12–15; stall 900 s, cap 5 400 s |
| 0.5 the gate, a devshell command | done | 7ebc848e: `gate`, passed in 4 min 15 s on cores 2–5 |
| 0.6 `plan/reports/28-baselines.md` | open | |
| 0.7 last message to `planning` | open | |

### Stage 1: the audit (session `step-28b`)

| item | state | evidence |
|---|---|---|
| 1.1 rubric, `plan/notes/audit-rubric.md` | open | |
| 1.2 machine checks in `nix flake check` | open | |
| 1.3 audit workflow, rounds and critic | open | |
| 1.4 `plan/reports/28-audit.md` | open | |

### Stage 2: the design

| item | state | evidence |
|---|---|---|
| 2.1 three drafts | open | |
| 2.2 judged and synthesised, `plan/notes/api.md` | open | |
| 2.3 walk-through per later step | open | |
| 2.4 quantifier spike, measured | open | |
| 2.5 the author's sign-off | open | |

### Stage 3 and 4: the fixes and their check rounds

| area | state | evidence |
|---|---|---|
| 3.1 the library's API, data model and wire forms | open | |
| 3.2 the search | open | |
| 3.3 efficiency | open | |
| 3.4 the command, the harness, the flake and the documents | open | |
| last reader of the rustdoc front page and README | open | |

## From the supervisor

### Where you start, `step-28` (2026-10-08 20:51)

- You are `step-28`, the first session of step 28: stage 0 only
  (requirements, baselines and gates), Opus 5.5 at `high`. The session
  ends when stage 0 is done, committed, its checklist current and the
  last message sent to `planning`. The audit is the next session's.
- Start this file with the step's checklist and a section "From the
  supervisor", and record this note there first.
- The author entered the passphrase at 20:50; signatures work until
  about 22:50, then commit unsigned as the prompt says.
- The machine is idle and the step's from now on. The author may take
  it back by day: then a "Pause" message comes.
- The author's answers so far: none needed for this stage. The
  supervisor's review asks nothing yet.
- `main@origin` is a10d8c74; the plan commits after it are signed and
  stay as they are.
