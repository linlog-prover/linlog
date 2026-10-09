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
| 0.1 register of later requirements, `plan/notes/requirements.md` | done | 5bf43829 (242 entries), the supervisor's review folded in by adbf12b6 (254 entries, conflicts C1 to C3) |
| 0.2 behaviour lock: fixtures and test | done | 908f91f7: `cli/tests/lock.rs` (73 calls), `core/tests/lock.rs` (32 JSON lines); stable over three runs |
| 0.3 journeys under callgrind, validated against wall-clock | done, provisional | 73789993: `bench/counts-against-time.py`, run 03:04–03:37 on core 2; a large drop in count is a drop in time on 20 of 20 journeys, a few percent on 16 of 20; other cores loaded by another project's tests, to repeat in a quiet window |
| 0.3 ratchet: ceilings, flake check, lowering command | done | 1ae95c4c: 20 journeys, `bench/ceilings.csv`, `nix build .#checks.x86_64-linux.ratchet` passed |
| 0.4 mutation testing wired in (`new-tool`), scope and command committed | done | f602c7e9: `mutants/run.sh`, `.cargo/mutants.toml`, profile `mutants` |
| 0.4 mutation run, batches per target file | done | 21:03–02:10, 5.1 h; `check`, `parse`, `lltp` rerun 02:11–02:56 on the fixed tests; 1 826 mutants, 321 surviving (2b332aa1, `mutants/baseline/`); labelled script 9bf4181e |
| 0.4 fuzz targets wired in (`new-tool`), one per untrusted reader | done | f602c7e9: nine targets under `fuzz/`, nightly 2026-09-25 in the `fuzz` devshell |
| 0.4 fuzz runs, each until coverage stops growing | done | ended 00:15; `target/fuzz/summary.tsv`: eight targets without a find (two at the 5 400 s cap, six stalled); `ordinary_text` 3 312 crash files, one panic: `core/src/ordinary/parse.rs:104:31` slices inside a multi-byte character (input `\|-z⊢`, "start byte index 5 is not a char boundary"), a panic on untrusted input, not fixed here, for the audit |
| 0.5 the gate, a devshell command | done | 7ebc848e: `gate`, passed in 4 min 15 s on cores 2–5 |
| 0.6 `plan/reports/28-baselines.md` | done | this commit |
| 0.7 last message to `planning` | done | sent with this commit's id |
| 0.8 the supervisor's 30 register issues | done (adbf12b6); was paused | workflow `wf_6d613a43-832` stopped at 23:16 when the usage limit was reached (six Sonnet verifiers, one Opus writer, none finished); resume with `Workflow({scriptPath: ".../step28-register-review-wf_6d613a43-832.js", resumeFromRunId: "wf_6d613a43-832"})` |

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

### Message from `planning` (2026-10-08, about 21:48)

- A test failed once and passed on its rerun (Claude Code flagged it on
  the screen): find which, check whether a flaky test can make the
  mutation run count a missed mutant as caught or a caught one as
  missed, and record it in the baselines report; nothing needs to stop.
- What was done: see "A flaky test and the mutation run" in
  `plan/reports/28-baselines.md`.

### Message from `planning` (2026-10-08, about 22:20)

- Go on: the turn ended while stage 0 was open. Wait on the runs in
  pieces of at most ten minutes and carry on; the turn ends only when the
  mutation run and the fuzzers have ended, the batches with flaky-only
  catches are rerun, the two changes of the mutation tooling are made,
  the counts are checked against wall-clock time on a quiet machine, `nix
  flake check` has run, `plan/reports/28-baselines.md` and the checklist
  are complete, and the last message is sent.
- What was done: acknowledged, and the session went on as asked.

### Message from `planning` (2026-10-08, about 23:05)

- Thirty issues with the register as committed in 5bf43829, from a
  review (Fable 5.1 for gaps, Opus 5.5 checking each against its
  sources) against the later prompts, `plan/later.md`, the decisions and
  research notes of the night (to be cited as `plan/notes/research/…`):
  17 missing requirements, 5 wrong readings, 5 conflicts, 3 vague
  entries. Verify each against the register and its sources, fold in
  those that hold in a commit of its own, resolve the conflicts or list
  them as decisions for the author, and record in the baselines report
  what was taken, changed or rejected and why. Part of item 0.1.
- What was done: see "The supervisor's review of the register" in
  `plan/reports/28-baselines.md`.

### Where stage 0 stood at the usage limit (2026-10-08 23:16; every item below was done by 03:45 on 2026-10-09)

- Committed: the register (5bf43829), the tools (f602c7e9), the
  behaviour lock (908f91f7), the journeys and ratchet (1ae95c4c), the
  gate (7ebc848e), the flaky-test fix (9556616f) and its rule (9c5ccb4d).
- Still running, detached and costing no quota: `step28-mutants` (7 of
  12 batches done; `mutants/baseline/`), `step28-fuzz` (5 of 9 targets
  done, no finds; `target/fuzz/summary.tsv`).
- Left: rerun the batches `check`, `parse` and `lltp`, whose only catches
  in some cases came from the flaky test (delete their lists, run
  `mutants/run.sh check parse lltp`); then swap in the labelled script
  (scratchpad `run.sh.new`: `--label`, `--compare`, outputs under
  `target/mutation/LABEL/`) and add `exclude_re` for test modules to
  `.cargo/mutants.toml`; the timed validation of counts against
  wall-clock time with nothing heavy beside it (scratchpad
  `validate.py`, frozen builds in `bin/`; the opt1, opt2 and opt3 counts
  are in `counts-opt*.txt`); the register review (0.8); `nix flake
  check`; the rest of `plan/reports/28-baselines.md`; the last message.

### Message from `planning` (2026-10-09, about 01:52)

- The usage limit reset at 01:50: go on with stage 0 (resume the
  register review, then the checklist's open items in order); commit
  unsigned while signing fails. A read-only workflow of the supervisor's
  runs beside, on no cores of this session's.
- What was done: resumed at 01:51.
- `nix flake check` passed at 2b332aa1 (03:00, `--keep-going`, exit 0)
  after a first run failed on a fuzz target's formatting (8f26bd36).

