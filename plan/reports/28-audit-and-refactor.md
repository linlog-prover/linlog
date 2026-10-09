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
| 1.1 rubric, `plan/notes/audit-rubric.md` | done | ad4794af |
| 1.2 machine checks in `nix flake check` | done | 42d0f22d: `conventions`, `typos`, `shear` (modules/conventions.nix), clippy's `pedantic` and picked lints with a backlog in `Cargo.toml`, rustdoc lints; the gate passed, the checks built, planted faults caught; `nix flake check --keep-going` passed at 334c68b1 (06:15) |
| 1.3 audit workflow, rounds and critic | done | `wf_715f365e-ce0`, 03:52–06:02, 81 agents (17 reviewers, 3 merges, 41 examiners, 2 critics, 16 gap readers, 2 rechecks): 226 findings stand (56 must-fix, 170 should-fix), 6 refuted, 135 of taste; two gap rounds, the second still adding 23 |
| 1.4 `plan/reports/28-audit.md` | done | this commit: the report, the decision list (C1 to C3, T1 to T7), `plan/reports/28-audit-findings.json` |

### Stage 2: the design

| item | state | evidence |
|---|---|---|
| 2.1 three drafts | done | session `step-28c`: three Opus 5.5 agents at `xhigh` started 06:55 (A web and wire forms, B proof term, checker and Rocq, C engines, calculi and quantifiers), brief in the session's scratchpad; C done 07:31 (98 KB), B 07:35 (93 KB), A 07:37 (97 KB) |
| 2.2 judged and synthesised, `plan/notes/api.md` | done | two judges started 07:37 (Fable 5.1 at `high`, Opus 5.5 at `high`), independent, with two added questions (an interned-atom representation, H9/H10); Opus done 07:52 (ranks A 56, C 52, B 49; base A with grafts), Fable 08:00 (A 56, B 56, C 54, ranked B, A, C; the same synthesis); both chose C's sides rule and the interned atomic formula; drafts and judgements committed in `plan/notes/api-drafts/`; `plan/notes/api.md` synthesised 08:14 (sections 11.5 and 12 open) |
| 2.3 walk-through per later step | done | ten Sonnet 5.5 agents at `high`, 08:16 to 08:26, one per step 29 to 38, against ab0b27e5's `plan/notes/api.md`: 2 blocking, 70 friction, 59 notes, every one answered in `api.md` section 12 (most by a change of the sections above); reports in `plan/notes/api-drafts/walk-NN.md` |
| 2.4 quantifier spike, measured | done | M1 to M3 by the agent, M1d, M2d, M1b by it after the go, M3i finished by the session after the pause (10:20 to 10:55, the supervisor's go for each run): the design's data model (M1d) and the generic zone with one and two instances (M2d, M3i) pass the gates, the drafts' appended `Pred`/`DualPred` cost 6.6 %; `plan/notes/api.md` 11.5, `plan/notes/api-drafts/spike-report.md`; the workspace forgotten and removed |
| 2.4a the fresh review answered | done | the supervisor's review (Fable 5.1 at `high`, `plan/notes/api-drafts/review-fable.md`, 6c6a72d0): nothing blocking, ten items; all ten answered in `plan/notes/api.md` (1085288f, listed at the end of its section 12) and in `plan/reports/28-design.md`; the supervisor's follow-up on item 10 answered as decision 21 (5b5b3064) |
| 2.5 the author's sign-off | done | the author answered on 2026-10-09, through the supervisor: every recommended answer, decision 4 with a converter (`wire::upgrade`), T3 changed to link-time optimisation now in area 3.1; api.md section 14 marked answered (17711466); the answers as rules, a `## Decisions` section in eleven files under `.claude/rules/` (f25e2285) |

### Stage 3 and 4: the fixes and their check rounds

| area | state | evidence |
|---|---|---|
| 3.1 the library's API, data model and wire forms | running (session `step-28d`) | sub-items below |
| 3.2 the search | open | |
| 3.3 efficiency | open | |
| 3.4 the command, the harness, the flake and the documents | open | |
| last reader of the rustdoc front page and README | open | |

#### Area 3.1, item by item (session `step-28d`)

The first fixes come first (the wrong answers, then the panic), each
with the test that would have caught it; then the design's sections in
an order where each commit builds on the last. Runs that need the
supervisor's go are marked "go".

| item | state | evidence |
|---|---|---|
| a. F23: `close_with` refuses a foreign proof, another goal, a mode that forbids it | done | tszyrrtw (64cec218): `Error::ForeignProof`, the graft checked in the session's mode; test `close_with_refuses_a_foreign_proof` (both witnesses, the smaller forest's panic included); gate passed. Another goal is refused by the existing check against the goal's ids; `GoalMismatch` comes with the proof's recorded goal (item m) |
| b. H2, H3: several conjectures refused (TPTP, LLTP) | done | xwquzzos (94a10a3f), with d: `Error::SeveralConjectures { second }` in `lltp::clauses`, which both readers share; tests `refuses_several_conjectures` in `lltp.rs` and `ordinary/parse.rs` with the findings' files; gate passed |
| c. H4: a counter named twice in a `.spec` file's `init` refused | done | lwpolxzs (51ad26ac): the finding's file in `reads_problems`' refusals; gate passed |
| d. H7: an empty LLTP formula refused | done | xwquzzos (94a10a3f): two empty formulas in `reads_problems`' refusals |
| e. H17: a session's history checked against the arena as each step left it | done | wptxunzx (1a2fe79e): the finding's reordered history in `interactive_json_format_and_round_trip`, which fails without the fix (run with it reverted); gate passed |
| f. H9, H10 with C1: the written order and sides, the reading's rule (lock commit (2)); the target set after it (go) | done, the target set waiting for a go | rtlqploq (gate passed, after omsrptkl, a commit of its own before it, raised the two check journeys' ceilings by 3.9 % and 3.8 %: their proofs are found on the reordered sequents); the probe of the LLTP ILL files ended: 4 268 read by both binaries, 226 refused by both for size only (200 000 occurrences), none changed; `Sequent::antecedents`, roots unsorted, the reading by the written sides and the left factor; test `the_written_sides_decide` (both witnesses as text and without sides, `⊢ ⊤, a` read as `0 ⊢ a`, `⊢ 0, ⊤` undetermined); both locks reblessed (the JSON key, the written order, `seq-print-latex`'s sequent); README; a probe read every ILTP image (274 problems, three translations, two logics) with no refusal, and the LLTP ILL files are being read with the old and the new binary |
| g. T3: `lto = "fat"`, `codegen-units = 1`, the ceilings re-recorded (go) | waiting for the go | kept uncommitted until its ceilings can be recorded, so every gate passes. After the go: the target set on a build of C1's commit itself (a jj workspace at rtlqploq), its counters compared with `bench/targets/after-coverability.csv` (the current oracle; `after-bias.csv` no longer matches the engine, F160, F181), so the comparison is C1's whatever lands later; then the target set at the head under LTO as the new oracle |
| h. the renames (`Atom`, `Branch`, `Side`, …) and what becomes private (2.3, 2.4) | done, gate interrupted by the pause | ksnmyzmo; the rest of 2.3 (`Forest::from_owned` stays public: the command builds a forest of a sequent it owns without a clone) comes with its items |
| i. `Mode` and `Fragment` (3.5) | done | `Mode` with `pub(crate)` fields, getters, `with_affine`/`with_mix`, `NAMES`, `name`, `FromStr` and `check`, the one table of words the command's batch and the harness now read (`affine-mix`; the batch wrote `affine` for it and lost the Mix); `Error::UnknownName`; `Fragment::ADDITIVE` (named `ALL`), `NAMED`, `NAMES`, `FromStr` with or without the `I`, `has_nets` (`ProofStructure`, the command); the dispatch destructures the mode (F140); test `names_read_back`; the command's pinned batch message lists the six words. The mode's wire form by name is lock commit (1), item s |
| j. one error family (4) | done | yzywpzko: `Error` non-exhaustive with named fields, `ErrorKind`, `code()`, `CODES`, `setting()`; `limits` module (`Limits`, `Progress`, `Phase`, `Refusal`, `Space`); `CheckError::{Invalid, Refused}` with `Fault`; `ViewError`, `WriteError`, `RenderError`, `svg::TooLarge` folded; `StepError`; `Error::Parse`. ukxwvpnq: `ParseError` made in one place with `span_utf16`, `line`, `column` and `expected` (R129, F34, the library half of F29), test `error_places_and_expectations`; the ordinary parser's panic on a second `⊢` (a slice inside the character, found while there) fixed with its case in `reads_the_syntax`. Then: one `Described` and `Owner` for `Error`, `CheckError`, `NetError` and `ShapeError` (F48), `NetError` written once, `Debug` on `Results` and the command's `Output` with the lint on (F50), the written form of every error (4.3) with test `error_json_format`, `Send + Sync + 'static` asserted; `OccSet`'s methods narrowed (wyuzsnql), which the unreachable-pub check reported. Two lock commits: nkwtoxsu (F29: the command's parse error ends with what was expected; the lock's two parse errors, README's broken entry), and the counts agreeing with their nouns (F33: `1 goal is open`, the only pinned line that moved; the index, refusal and session messages likewise) |
| k. `Limits`, `Progress` and the stop on every long call (5) | first commit done, gate passed | the bounds out of `search::Options` and `ViewOptions` into `Limits` (`Forest::within`, `Sequent::parse_within`, `lltp::read`, `mist::read`, `check_within` with `work` and a stop every 4 096 nodes, `derivation_size_within`, `derivation_within`, `prove_within`, `prove_goal`, `Interactive::close*`, `batch::{prove, run}` with a `Plan` of limits, `linear_derivation`), every writer's stop a progress stop, the engines behind the shim `without_progress` until area 3.2's measured commit; the command and the harness map their flags onto it with their output unchanged (both locks and README pass unreblessed); test `core/tests/limits.rs`. Moved on: `race` with `Clock` (item l), `Within` (item s), the sequent writers' estimate (F5) and `png`/`pdf` memory (item t), the net calls (item p), the ordinary layer (item r), `Limits::work` in the engines and `Reason::WorkLimit` (area 3.2, with the polls) |
| l. the options, `Clock`, `Settings`, `Styles` (6) | done | tuooxrol: `search::Options` with public fields under `#[non_exhaustive]`, a `with_*` per field, serde (defaults, unknown keys refused, `"auto"`), `Jobs` and `Cadence` clamped where read, `Schedule::Turns` (F61, F64, F78, F79); tests `options_json_format` and the turns in `default_bias_takes_turns`. Then: `batch::Options` with builders and serde, its workers clamped to `MAX_JOBS` (F73), `Cancel` on `Results`, raised by `cancel`, `canceller` and a drop (F74), test `cancelled_results_stop_their_searches`; `Clock` and `Settings` (`Settings::default()` the command's behaviour), `export::Styles` from the command with a placeholder for a format the build lacks, test `settings_json_format`; the front page says why the command's verdicts differ from the default options' (F42). Moved on: `search::race` (F103, F104, filed under the search) and its callers to area 3.2, the command's `--settings FILE` (R183) to area 3.4, `run_local` to its caller (the web client) |
| m. `Member`, `Proof { goal, mode }`, `CheckError` (3.4, 3.7, 3.8) | done | nyxmnuxv: `Member` (a member of a sequent of its owner, the occurrence's number while no owner keeps a table), `Node`'s operands, `Dyadic` and `Fault` as members; the engines and the forest keep `OccId` and convert where a node is built or read, once per match arm. Then: `Proof` records its `goal` (set by `prove_goal` off the roots) and the `mode` it was found in, `new_of_goal`, `with_mode`, `occurrence`, `formula`; the checker, the size pass and the view conclude at the proof's own conclusion, so a goal proof checks, and `prove_goal` checks every proof it returns; `Error::GoalProof` (kind unsupported) from `from_proof`, the Rocq writer and `linlog check`, `Error::GoalMismatch` from `close_with` (F23's other half, F89); `Node::NAMES` and `TAGS` pinned by `name_lists_follow_the_variants`; `Dyadic` non-exhaustive; tests `a_goal_proof_records_its_goal` and the mismatch in `close_with_refuses_a_foreign_proof`. The keys `goal` and `mode` go on the wire with the wire level (item s) |
| n. `Rule` and `Named`, `Derivation`, `Inference`, `ViewOptions` (3.9) | done | xnszropr: `Rule` the sixteen one-sided rules, `Named { rule, side }` the rule a derivation shows (F60), names, spellings and labels unchanged, label tables indexed by `Named::index`, test `names_round_trip` in `rule.rs`. rnnrsvnv: `Inference` non-exhaustive with accessors and a sequent of members, `Derivation::occurrence`/`formula`. mnkunorl: `ViewOptions::sides` (one, two, or `Auto`, two-sided for a proof meant for intuitionistic mode) replaces the two-sided twins of `derivation`, `derivation_within` and `derivation_size`, whose view now carries the sides; `Proof::derivation_within` is the one public door (`Derivation::new` and `two_sided` crate-private) |
| o. `Interactive` (3.10) | done | `GoalId` apart from `InfId` (F69), `Step`/`Split`, `Applicable`/`Needs`, `apply(goal, &step)`, `split_passes(goal, &step)`, `Closed` from `close` (a refused graft keeps the goal open and the proof in the outcome), `close_all` with a result per goal (F68), `proof(limits, stop)`, `derivation()` fallible and `derivation_within`, `within`, `occurrence`/`formula`, `StepError::EmptyPremise` (F67); the command's session output unchanged (qmpstuqr). Then the reading kept in the state, O(1) (F66), and a linear read-back: two pointers for a split, a history walk that enters no later step's inferences (F21) |
| p. `ProofStructure`, `VertexId`, `Criterion`, `NetError` (3.11) | open | |
| q. `Refutation` and `Disproof` (3.12, lock commit (3)) | open | |
| r. the ordinary layer (3.13) | open | |
| s. the wire level, `wire::{Within, upgrade, LEVEL}` (7, lock commit (1)) | open | |
| t. the exports' one `write` (9) | open | |
| u. the area's other findings, `lib.rs`'s allowances, the docs | open | |
| v. check rounds (stage 4), at most three; the fresh-context reviewer | open | |

#### Area 3.1: decided unattended

- **`Forest::from_owned` stays public** (2.3 would have made it
  private): the command builds the forest of a sequent it owns, and the
  alternative, `Forest::within(&sequent)`, clones a sequent of up to
  fifty million occurrences.
- **The derivation's refusal carries bytes, not a `Size`**
  (`Refusal::Output { estimate_bytes, limit_bytes, least_bytes }`): the
  command's line about a derivation too large to build names its
  inferences and characters, so it asks `derivation_size` for them;
  set aside: a `Size` in the refusal, which would put a view's type into
  the one refusal every call shares.
- **The command reads a sequent's text without a bound and then admits
  it** (`io::admit`, `--occurrence-limit`), as before: reading within the
  bound (F16) changes which message a too-large input gets, which is the
  command's area (3.4) and would rebless the lock here.
- **A check refused for its memory is `Error::Check(CheckError::Refused)`**
  (`Error::Unchecked` is gone): one variant per specific error type;
  F139's change of the verdict a search answers then is the search's
  (area 3.2).
- **The forced-engine refusals keep their variants** (`NetFragment`,
  `NetMode`, `NetGoal`, `EngineMode`, `NotAdditive`, `NotHorn`), kind
  `unsupported`, until area 3.2 folds them into one `EngineRefused`
  with the engine's `NotTaken` reason; their codes are already final.
- **`Limits::work` binds the checker's pass alone so far**, and
  `Reason::WorkLimit` does not exist yet: a search counts its work at
  the polls that area 3.2 moves to the progress stop in one measured
  commit, and the shim hands the engines no work until then.
- **`Limits::stack_bytes` keeps the 8 MiB floor** that the design says
  the derivation's builder needed: the pool's workers run stolen tasks
  on top of their own frames and the command's search thread runs the
  writers too, so the floor stays until area 3.2 counts the depth per
  worker; it costs address space only.
- **`search::race` comes with `Clock` (item l)**: its `add_pool` reads
  the clock's `pool_after_ms`, and the command and the harness move to
  it in one change.
- **One `Described` holds an enum of the four error types**, not a
  generic `Described<'a, E>`: a generic one needs a public trait bound,
  and a crate-private one is a private bound in a public impl, which the
  lints refuse; the enum keeps the trait out of the API.
- **`Owner` is not sealed** (the design says sealed): a sealed
  supertrait is an unnameable public type, which the crate's
  `unnameable_types` check forbids; a method added later gets a default
  body, so implementing it outside the crate breaks nothing.
- **Errors give no `source()`** (the design asks for it): every wrapping
  variant's message already holds the inner error's, and the command
  and the harness print errors with anyhow's `{:#}`, which appends each
  source; a `source()` would print the inner message twice in the
  pinned output, and dropping it from the message would empty the
  `message` of the written form. API guideline C-GOOD-ERR allows either.
- **An error from `prove_within` carries no formula text**: the design
  would have `ShapeError` carry the subformula's text, since the caller
  holds no forest; the forest is a function of the sequent, so the
  caller rebuilds it (`Forest::new`) and describes, as the command does.
- **`memo_limit` is a `u32`**, as the design's table has it, so that a
  settings file reads alike on a 32-bit build; the command's
  `--memo-limit` now refuses a value past four billion, which no search
  could fill.
- **`batch::run_local` waits for its caller**, the web client, which
  needs a batch without `Send`; nothing in the tree would call it, and an
  item without a caller waits for one.
- **`Settings` is the library's value now, and the command adopts it in
  area 3.4** (`--settings FILE`, every flag the spelling of a key); here
  the command's timings come from `Clock`'s constants and its styles
  are the library's `Styles`.
- **The harness's CSV columns keep their names** (`memory_limit`,
  `recursion_limit`): the columns are its interface (`bench.md`), and
  only the flags behind them map onto `Limits`.

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


### Where you start, `step-28b` (2026-10-09)

- You are `step-28b`, the second session of step 28: stage 1 only, the
  audit (Opus 5.5 at `high`; the workflow with the lenses and models the
  stage names). The session ends when stage 1 is done: the rubric, the
  machine checks in `nix flake check`, the audit workflow with its
  rounds and critic, `plan/reports/28-audit.md` with the decision list,
  all committed, the checklist current and the last message sent to
  `planning`. The design is the next session's.
- Read "From the review of stage 0" in the prompt first; the research
  notes it names are in `plan/notes/research/`. Record this note here
  first.
- The author's answers so far: none. The register's conflicts C1 to C3
  go into the decision list beside the audit's own matters of taste.
- Signing: the author is away and the passphrase's cache has lapsed.
  Test a signature before each commit and commit with
  `--config signing.behavior=drop` while it fails; the unsigned commits
  since 7c438f1b stay as they are, the supervisor signs them.
- The machine is the step's, but another project of the author's runs
  browser tests (headless Chrome and Node) now and then; this stage has
  no timed run. Builds run on cores 2 to 5, the agents' programs on 6
  to 15.
- Usage: the five-hour window reset at 01:50, the weekly one resets at
  05:00. If a limit stops the session, it writes where it stands into
  the checklist and ends its turn; the supervisor says when to go on.
- What was done: recorded at the session's start; signing failed
  throughout (no pinentry), so the session's commits are unsigned. Two
  confirmed wrong answers (F165, F23) were sent to `planning` at 06:05,
  as the prompt asks for a soundness fault.

### Where you start, `step-28c` (2026-10-09)

- You are `step-28c`, the third session of step 28: stage 2 only, the
  design (Opus 5.5 at `xhigh`; the drafts, judges, walk-through and
  spike with the models the stage names). The session ends when
  `plan/notes/api.md` is written, judged, synthesised, walked through
  and its spike measured, with the decisions it needs from the author at
  its end, committed, the checklist current and the last message sent
  to `planning`. The supervisor then has the design reviewed and puts
  it to the author for the sign-off; the fixes are later sessions'.
- Read "From the review of stage 0" and "From the review of stage 1" in
  the prompt first, then `plan/reports/28-audit.md` (its decision list,
  "For the design and the fix sessions" and "From the review"). Record
  this note here first.
- The author's answers so far: none. The design starts on the
  recommended answers to C1 to C3, T1 to T7 and HD1 to HD5, all
  provisional; where an answer would change the design, it says how.
- The findings the design must answer are those of the audit report's
  "For the design and the fix sessions", plus H9 and H10 with C1 and
  H18 with the bounds. The findings file (about 1 MB) is searched by
  `fid` with a script, never read whole.
- Signing: the passphrase's cache has lapsed and the author is away;
  test a signature before each commit and commit with
  `--config signing.behavior=drop` while it fails.
- The machine is the step's; another project of the author's runs
  browser tests now and then. The spike's measurement counts
  instructions (callgrind and the target set's counters); note the load
  if anything is timed. Builds on cores 2 to 5, agents' programs on 6
  to 15.
- Usage: the five-hour window resets at about 06:50, the weekly one on
  2026-10-16. If a limit stops the session, it writes where it stands
  into the checklist and ends its turn.

### Message from `planning` (2026-10-09, about 09:00): Pause

- "Pause: the author needs the machine now." What was done: the spike's
  agent was stopped (it had finished M1d, M2d and M1b's journeys and was
  starting M3i), the unit `linlog-targets` (M1b's target set, in the
  spike workspace) was stopped; no unit, scope, build or agent of this
  session runs. The design and the walk-through are committed (5c12e0ed);
  `plan/reports/28-design.md` is written but not committed.
- **How each part resumes**: (1) M1b's target set: `cd ../linlog-spike`,
  `bench/targets.sh spike-m1b` in a capped scope on cores 12 to 15 (it
  resumes its CSV, `--append --resume`), then `cmp.py` against
  `spike-base.csv`; (2) M3i: build, `cargo test -p linlog` and the lock,
  the journeys and the target set as the spike's brief says, compared with
  M2d; (3) fill `api.md` 11.5 with M1 to M3i, and decision 1's evidence;
  (4) copy the spike's report into `plan/notes/api-drafts/`, finish
  `plan/reports/28-design.md` (outcome, cost), commit, last message to
  `planning`; (5) `jj workspace forget spike` and abandon the spike's
  commits once the numbers are recorded.

### Message from `planning` (2026-10-09): Resume

- "Resume: go on from your checklist. The author uses the machine during
  the day, with no limit on your cores. One condition: before you start
  any benchmark or measurement run (M1b's target set, M3i's callgrind
  counts, anything else timed or counted), message planning with what you
  are about to run, on which cores, and for about how long, then end your
  turn. I check the machine and answer go or wait. Builds and writing
  need no check."
- What was done: recorded; M3i built and tested, then the measurement
  runs asked for (below).

### Message from `planning` (2026-10-09, about 14:00): amend the design

- "A fresh-context review by Fable 5.1 at high found nothing blocking,
  but ten items. It is committed as
  plan/notes/api-drafts/review-fable.md (6c6a72d0); read it whole. Fix
  them in api.md, then the report and checklist, as commits of their
  own, testing a signature before each": items 1, 2 and 7 before the
  sign-off (the fourth lock change ordered or named, the stop's shim
  named, `checked` added; the race at `--jobs 2` and below; the
  exception to P3's named fields stated and the rest converted); items
  3, 4 and 5 as well (no reading of version-less documents in the
  pre-release names, "the author's standing rule is no aliases before
  the release, so this is not a decision for the author", commit (1)
  regenerating the fixtures; the text parser refusing `forall` and
  `exists` from step 28; the prune row of 10.10 (f) and the open-atom
  count in decision 1's costs); item 6 (decision 20 adopted on counts,
  provisional on a pinned-time run at step 37's lift, in 14.2 and the
  stage report); items 8 and 9 (the mismatches fixed, `GoalProof`'s kind
  settled, both in "Decided unattended"); item 10 (the H9 and H10 tests
  text-only on purpose). "No measurement is needed. End as before, with
  your last message to planning listing what changed, by item."
- What was done: all ten answered in `plan/notes/api.md` (1085288f), the
  stage report and this checklist amended; nothing was run.

### Message from `planning` (2026-10-09, afternoon): decision 21

- With the sides unknown, today's choice keeps H9's and H10's wrong
  answers reachable through JSON and the library. "Write this as
  decision 21 in §14.2": recommended, the reading takes today's choice
  only where it is the only intuitionistic reading, else refuses asking
  for the sides; set aside, today's guess. "Check its cost before you
  write it" (lock entries, fixtures, tests, README blocks with `-i` and
  JSON; whether `Interactive`, `Derivation` or the Horn engine rely on
  the guess), "then fix §3.6's wording", commit as before, one short
  message to planning.
- What was done: the cost checked in the code (nothing moves beyond
  commit (2) of 7.5; nothing downstream of the reading guesses);
  decision 21 and 3.6 written (5b5b3064), with the factor rule sharpened
  to the left factor in both cases, since H9's one-sided form has one
  reading by the symmetric rule and only the left-factor rule refuses
  it; the stage report amended.

### Message from `planning` (2026-10-09, afternoon): the author's sign-off

- "The author has signed off the design (step-28c), answering every
  decision in this session by their own choice." All recommended answers
  are taken except these. **Decision 4**: one global wire level plus a
  converter, from the first version bump on, of a document of an older
  level to the current one where possible; its skeleton now, "one
  upgrade entry point (in the library, behind `serialize`) that takes a
  document of any known level and returns it at the current level", the
  identity at level 1 with one test, in §7.1 and area 3.1's plan;
  between released levels only, so no alias. **Decision 3**: the author
  likes a progress value for front ends and leaves the form to the
  recommendation; keep the closure over `Progress`. **T3**: LTO now,
  `lto = "fat"` and `codegen-units = 1` in `[profile.release]`, a commit
  of its own early in area 3.1 that re-records the ratchet's ceilings,
  says why and notes the gate's longer release build.
- Mark §14 answered, update the stage report and the checklist (2.5
  done), and make the answers rules under `.claude/rules/` (a decisions
  file loaded for `core/**` and `cli/**`, or bullets in the modules'
  existing files), each bullet the rule and its reason. Commit as
  before; one message to planning with the commits and where the rules
  went.
- What was done: §14 answered and `wire::upgrade` in 7.1 and 7.5
  (17711466); the rules as a `## Decisions` section in eleven existing
  files (`core.md`, `core-sequents.md`, `core-forest.md`,
  `core-proofs.md`, `core-search.md`, `core-focus.md`, `core-inputs.md`,
  `core-ordinary.md`, `cli.md`, `bench.md`, `claude-infra.md`), so each
  loads with the code it governs and CLAUDE.md's table is unchanged
  (f25e2285); the stage report's area plan has the converter and the LTO
  commit.

### Where you start, `step-28d` (2026-10-09)

- You are `step-28d`, the fourth session of step 28: stage 3, area 3.1
  only, the library's API, data model and wire forms (Opus 5.5 at
  `xhigh`). The session ends with the area's check rounds (stage 4,
  three at most) and the fresh-context reviewer the stage names, all
  committed, with the checklist current and the last message sent to
  `planning`. The search, efficiency and command areas are later
  sessions'.
- Read the prompt's three "From the review" sections first; record this
  note here first.
- The design is signed off: `plan/notes/api.md` §14 records the
  author's answers, and the `## Decisions` sections of eleven rules
  files hold them as rules that win over older bullets. The area's plan
  is in `plan/reports/28-design.md` (area 3.1): sections 2 to 7 and 9 of
  the design, the lock commits (1) to (3) of §7.5, `wire::upgrade`, and
  the T3 commit (`lto = "fat"`, `codegen-units = 1`, the ceilings
  re-recorded).
- The findings: `area == "library"` in
  `plan/reports/28-audit-findings.json` (1 MB, selected with
  `python3 -I`, never read whole): the F findings and the held-back H2
  to H4, H6 to H10, H17 and H19. First fixes, each with the test that
  would have caught it: the wrong answers F23, H2, H3, H4, H7, H9 and
  H10 (H9 and H10 through the written sides, decisions 2 and 21), then
  H17's panic.
- More findings come as H22 and up from the supervisor's probe workflow;
  the supervisor messages those in this area (expected: a read-back of a
  compact derivation that fails its own checker, panics of
  `ProofStructure` calls on foreign scratch or ids, a JSON Lines line
  without `sequent` losing its name and mode).
- No benchmarks until the supervisor says the machine is idle: no
  `bench/targets.sh`, baseline, `linlog-bench run` or callgrind
  measurement outside the gate, no timed run, no mutation or fuzz run.
  The gate on each code commit is allowed (capped, cores 2 to 5). Runs
  that need a go (the T3 commit's ceilings, the target set after C1's
  commit, the check rounds' mutants and fuzzing) are asked for by
  message, batched where the order allows.
- The check rounds probe: a lens that judges a guard, a refusal or an
  exit status runs a witness, and an H finding goes to the soundness
  lens, which reruns its witness.
- Signing works (`step28-gpg-warm`); test a signature before each
  commit anyway. Builds on cores 2 to 5, agents' programs on 6 to 15;
  the author uses the machine by day. The five-hour window resets at
  16:50, the weekly one on 2026-10-16; if a limit stops the session, it
  writes where it stands into the checklist and ends its turn.

### Messages from `planning` to `step-28d` (2026-10-09, afternoon)

- The go for the batched runs (T3's ceilings, the target set after C1,
  qcover, the families) waits until the author says the machine is idle;
  meanwhile work that needs no run, T3 uncommitted, gates per commit as
  before; C1's target-set comparison must stay attributable to C1. What
  was done: acknowledged; the target set will run on a build of C1's
  commit in a workspace of its own (item g).
- A correction: C1's target set compares with
  `bench/targets/after-coverability.csv`, not `after-bias.csv`, which no
  longer matches the engine since the Horn engine took the programs with
  `!` (F160, F181, area 3.4 fixes the names). What was done: recorded in
  item g.

### Message from `planning` (2026-10-09, evening): Pause

- "Pause: the author needs the machine now." What was done: the gate
  unit `step28-gate` was stopped while it ran the queue `rtlqploq
  ksnmyzmo yzywpzko` (rtlqploq passed; ksnmyzmo was in its feature
  checks, yzywpzko not begun); no unit, scope, build, probe or agent of
  this session runs (`step28-gpg-warm` is the supervisor's and runs on).
- **Where it stands**: committed and unsigned-or-signed as they came:
  tszyrrtw (F23), xwquzzos (H2, H3, H7), lwpolxzs (H4), wptxunzx (H17),
  omsrptkl (the two check ceilings), rtlqploq (C1, H9, H10), ksnmyzmo
  (renames), yzywpzko (the error family). In the working copy above
  them, uncommitted: item k half done. Done there: `Forest::{new,
  within, from_owned}` take `&Limits` (`Forest::DEFAULT_LIMIT` gone),
  `Sequent::parse_within`, `mist::read(text, &Limits)` (no
  `read_within`), the checker's pass with an `Allowance` (memory, work,
  phase, the progress stop every 4 096 nodes; `Halt::{Work, Stopped}`),
  `Proof::check_within(mode, &Limits, stop)`,
  `derivation_size_within(two_sided, &Limits, stop)`, `ViewOptions` with
  `compact` alone and `derivation_within` / `two_sided_derivation_within
  (&view, &Limits, stop)`, `proofs::DEFAULT_MEMORY_LIMIT` removed,
  `search::Options` without its three bounds, `prove_within` and
  `prove_goal(…, &Limits, stop: FnMut(Progress))` with the shim
  `search::without_progress`, `Decide::decide` taking `limits`. Not
  done: the engines' `decide` implementations and the functions that
  read the bounds (focus `search_goal`, `Problem::new`, `reason`,
  `schedule::alternate`'s stack, focus `parallel::search_goal`'s stack,
  the additive path's recursion, the net engine's stack), `lltp::read`,
  the exports' and `write_text`'s stops, `Interactive`'s `close*`, the
  ordinary layer's `linear_derivation`, the png/pdf memory default, the
  command, the harness, the fuzz targets, the tests, the rules files.
  The crate does not build in this state.
- **How it resumes**: `jj workspace update-stale` in `../linlog-gate`
  and the gate queue `ksnmyzmo yzywpzko` again (scratchpad `gate.sh`);
  then finish item k from the list above, compiling with the capped
  `cap.sh`, and commit it with its own gate.

### Message from `planning` (2026-10-09, evening): Resume

- Go on from the pause entry, gates first; benchmarks still held. Order
  the rest so that everything without a run comes first: the fixes, then
  the check rounds' reading and witness runs and their fixes, then the
  fresh-context reviewer; last one batch for the go (T3 with its
  ceilings, C1's target set on its own build, the head's target set,
  qcover, the families, the check rounds' mutants and fuzzing), sent to
  `planning` with times and cores. What was done: the gate queue
  `ksnmyzmo yzywpzko` restarted, the signing loop restarted, item k
  resumed.
