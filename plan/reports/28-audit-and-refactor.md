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
| 0.1 register of later requirements, `plan/notes/requirements.md` | open | |
| 0.2 behaviour lock: fixtures and test | open | |
| 0.3 journeys under callgrind, validated against wall-clock | open | |
| 0.3 ratchet: ceilings, flake check, lowering command | open | |
| 0.4 mutation testing wired in (`new-tool`), scope and command committed | open | |
| 0.4 mutation run, batches per target file | open | |
| 0.4 fuzz targets wired in (`new-tool`), one per untrusted reader | open | |
| 0.4 fuzz runs, each until coverage stops growing | open | |
| 0.5 the gate, a devshell command | open | |
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
