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
| 3.1 the library's API, data model and wire forms | done (session `step-28d`, 2026-10-09 to 10-10; reviewed by the supervisor) | sub-items below; `nix flake check` passed at kktkswpq |
| 3.2 the search | done (session `step-28e`, 2026-10-10) | sub-items below; every code commit gated; the target sets equal; three check rounds, the fresh-context reviewer (no finding), the mutation run; `nix flake check` passed at vuuroryz |
| 3.3 efficiency | done (session `step-28f`, 2026-10-13; reviewed by the supervisor) | sub-items below; every code commit gated; the target sets equal; two check rounds and the fresh-context reviewer, every finding fixed or decided; mutants and fuzzing of what changed; F148 a decision for `planning`; `nix flake check` passed at twzoumll |
| 3.4 the command, the harness, the flake and the documents | done (session `step-28g`, 2026-10-13 to 10-14; reviewed by the supervisor) | sub-items below; every code commit gated; the families and the target set equal; three check rounds, the fresh-context reviewer and the ultra reviews' U1, every finding fixed or decided; every mutation batch and fuzz target again; `nix flake check` passed at nvxqnwxr |
| last reader of the rustdoc front page and README | done (Fable 5.1 at `high`, in `step-28g`) | area 3.4's row l: two must-fix obstacles answered in koqypwoq and `plan/later.md`, the rest in `plan/later.md`, "a first user's obstacles" |
| closing: the report's sections and the register | done (session `step-28h`, 2026-10-14) | tsuoupvz: README states `check`'s default time limit (README's test passed); qlplwuru: every register entry's "State after step 28" (13 read-only agents, one per kind; 31 met, 107 partly, 116 not met; a sample of the met claims checked here, R194 moved to partly); the closing sections below ("The step's outcome" to "The register after step 28"), the renamed table checked by an agent against the code (184 rows, 183 confirmed); `nix flake check --keep-going` passed at szmktwow (unit `step28-flake-check`, cores 2 to 5; its features check logged six unused imports of `proofs/interactive/replay.rs` without `serialize`, open question 8); the supervisor's three fixes: rsvlqous (`replay.rs` gated on `serialize`), krxomqoz (warnings denied in the gate's `cargo hack` runs and the `features` check), txykwztn (the three doc points of open question 7), each with the gate passed; the mutants' timeouts recorded in area 3.4's row k (all 24 hang alone); `nix flake check --keep-going` passed again at txykwztn with the records (unit `step28-flake-check2`, no unused import logged) |
| follow-up: F122, F123 and the engines on their sequents | done (session `step-28i`, 2026-10-14) | the section "Follow-up: F122, F123 and the engines on their sequents, item by item" and its "decided unattended". Both sequents unprovable; F123's Mix mutant was slow because failures that rest on the loop check were never memoized: they wait on their ancestor (qoksszzp, ytvrknnp), copy bound 3 in 17 179 stable sequents instead of 76.5 million. The generators landed (wzynqlqo, ypvxqstw); U2 to U7 (vzxtsvrx, oxlvywnp, mlkkmqmu, xwoononu); open questions 3 and 5 (yqsorumv, vvwkkuzx, vnkywvnv); open question 11, the count of one sign, built at the author's answer (kvpyrzuv, ymmluvqt, qlnmtlzq, uyrrnvwy, pqnxxtsx, lyowmwos): F122's mutant refuted at once; on LLTP's 1 663 problems with such an atom one more refuted at copy bound 3 (SYN389+1 in 01) and two more proved without a bound (SYJ205+1.009 in cbn 9.56 s to 0.78 s, .010 from over 30 s to 2.22 s, timed alone), none lost, none against the status. Target sets after each search change (pnmsymur, kqtkltmp, snrttxvw, voukzpot): no verdict differs from the oracles, only the chain family's four rows with one memo entry fewer and one hit more; net sets equal; families and qcover `--strict`. Panel on the waiting failures (argument, counterexamples, integers) and on the count (counterexamples: 8 209 built sequents, 2 740 reaching the count, 0 disagreements with the reference prover or head; argument; integers): no wrong verdict; their findings fixed (ytvrknnp, zmqntlyl, qlnmtlzq, uyrrnvwy, pqnxxtsx). Two fresh-context reviewers (pxsxlnrl..vvwkkuzx, kvpyrzuv..qlnmtlzq): no soundness bug; fixed in zmqntlyl, ovolwupp, srtmwmwo, yurnpots, spmrupso, lyowmwos. Every code commit gated. Mutants: check 4, search 3 (two that hang alone), horn-mod 0, serialize 2 survivors, each fewer than area 3.4's; the focused engine's changed lines 224 mutants, the survivors pinned in vtypozyz or explained. `nix flake check` passed at pqnxxtsx |

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
| f. H9, H10 with C1: the written order and sides, the reading's rule (lock commit (2)); the target set after it (go) | done | the target set on a build of rtlqploq itself (workspace `../linlog-at-c1`, `bench/targets/after-written-order.csv`, urnmzrnw) against `after-coverability.csv`: all 265 rows the same verdict (167 proved, 58 unprovable, 40 unknown), the 225 decided rows equal in `nodes`, `splits`, `memo_hits` and `memo_entries`; 432.7 s against 425.5 s summed, indicative only (a gate shared cores 2 and 3). rtlqploq (gate passed, after omsrptkl, a commit of its own before it, raised the two check journeys' ceilings by 3.9 % and 3.8 %: their proofs are found on the reordered sequents); the probe of the LLTP ILL files ended: 4 268 read by both binaries, 226 refused by both for size only (200 000 occurrences), none changed; `Sequent::antecedents`, roots unsorted, the reading by the written sides and the left factor; test `the_written_sides_decide` (both witnesses as text and without sides, `⊢ ⊤, a` read as `0 ⊢ a`, `⊢ 0, ⊤` undetermined); both locks reblessed (the JSON key, the written order, `seq-print-latex`'s sequent); README; a probe read every ILTP image (274 problems, three translations, two logics) with no refusal, and the LLTP ILL files are being read with the old and the new binary |
| g. T3: `lto = "fat"`, `codegen-units = 1`, the ceilings re-recorded (go) | done | wunyrqsz, after the go: every ceiling the larger of the shell's `ratchet` and the flake's `ratchet` check (within 0.75 % of each other), every count down, from 1.1 % (search-chain-128) to 43 % (derivation-chain-64; render-typst 33 %, render-latex 21 %, the searches 1 to 5 %); the release build took 42 s in the shell. At the area's end on T3's build (workspace `../linlog-at-t3`): the target set `bench/targets/head-lto.csv` (cores 2 and 3, no gate beside it), the same 265 verdicts as C1's and the 225 decided rows equal in `nodes`, `splits`, `memo_hits` and `memo_entries`, 119.1 s against C1's 122.4 s summed on them, so the new oracle under LTO; qcover (177 `.spec` files, intuitionistic affine as the files give it, `--engines horn,auto`, 5 s, core 4): 59 proved, 103 unprovable, 15 unknown on both rows, no verdict against `bench/defaults/horn-qcover.csv`, none gained or lost, none of the 12 stated answers contradicted; the families (`--all-families --timeout 5`, core 0 by planning's leave): 123 runs, no mismatch, 62 proved, 38 unprovable, 23 unknown, the unknowns those of the 10-02 baseline |
| h. the renames (`Atom`, `Branch`, `Side`, …) and what becomes private (2.3, 2.4) | done, gate interrupted by the pause | ksnmyzmo; the rest of 2.3 (`Forest::from_owned` stays public: the command builds a forest of a sequent it owns without a clone) comes with its items |
| i. `Mode` and `Fragment` (3.5) | done | `Mode` with `pub(crate)` fields, getters, `with_affine`/`with_mix`, `NAMES`, `name`, `FromStr` and `check`, the one table of words the command's batch and the harness now read (`affine-mix`; the batch wrote `affine` for it and lost the Mix); `Error::UnknownName`; `Fragment::ADDITIVE` (named `ALL`), `NAMED`, `NAMES`, `FromStr` with or without the `I`, `has_nets` (`ProofStructure`, the command); the dispatch destructures the mode (F140); test `names_read_back`; the command's pinned batch message lists the six words. The mode's wire form by name is lock commit (1), item s |
| j. one error family (4) | done | yzywpzko: `Error` non-exhaustive with named fields, `ErrorKind`, `code()`, `CODES`, `setting()`; `limits` module (`Limits`, `Progress`, `Phase`, `Refusal`, `Space`); `CheckError::{Invalid, Refused}` with `Fault`; `ViewError`, `WriteError`, `RenderError`, `svg::TooLarge` folded; `StepError`; `Error::Parse`. ukxwvpnq: `ParseError` made in one place with `span_utf16`, `line`, `column` and `expected` (R129, F34, the library half of F29), test `error_places_and_expectations`; the ordinary parser's panic on a second `⊢` (a slice inside the character, found while there) fixed with its case in `reads_the_syntax`. Then: one `Described` and `Owner` for `Error`, `CheckError`, `NetError` and `ShapeError` (F48), `NetError` written once, `Debug` on `Results` and the command's `Output` with the lint on (F50), the written form of every error (4.3) with test `error_json_format`, `Send + Sync + 'static` asserted; `OccSet`'s methods narrowed (wyuzsnql), which the unreachable-pub check reported. Two lock commits: nkwtoxsu (F29: the command's parse error ends with what was expected; the lock's two parse errors, README's broken entry), and the counts agreeing with their nouns (F33: `1 goal is open`, the only pinned line that moved; the index, refusal and session messages likewise) |
| k. `Limits`, `Progress` and the stop on every long call (5) | first commit done, gate passed | the bounds out of `search::Options` and `ViewOptions` into `Limits` (`Forest::within`, `Sequent::parse_within`, `lltp::read`, `mist::read`, `check_within` with `work` and a stop every 4 096 nodes, `derivation_size_within`, `derivation_within`, `prove_within`, `prove_goal`, `Interactive::close*`, `batch::{prove, run}` with a `Plan` of limits, `linear_derivation`), every writer's stop a progress stop, the engines behind the shim `without_progress` until area 3.2's measured commit; the command and the harness map their flags onto it with their output unchanged (both locks and README pass unreblessed); test `core/tests/limits.rs`. Moved on: `race` with `Clock` (item l), `Within` (item s), the sequent writers' estimate (F5) and `png`/`pdf` memory (item t), the net calls (item p), the ordinary layer (item r), `Limits::work` in the engines and `Reason::WorkLimit` (area 3.2, with the polls) |
| l. the options, `Clock`, `Settings`, `Styles` (6) | done | tuooxrol: `search::Options` with public fields under `#[non_exhaustive]`, a `with_*` per field, serde (defaults, unknown keys refused, `"auto"`), `Jobs` and `Cadence` clamped where read, `Schedule::Turns` (F61, F64, F78, F79); tests `options_json_format` and the turns in `default_bias_takes_turns`. Then: `batch::Options` with builders and serde, its workers clamped to `MAX_JOBS` (F73), `Cancel` on `Results`, raised by `cancel`, `canceller` and a drop (F74), test `cancelled_results_stop_their_searches`; `Clock` and `Settings` (`Settings::default()` the command's behaviour), `export::Styles` from the command with a placeholder for a format the build lacks, test `settings_json_format`; the front page says why the command's verdicts differ from the default options' (F42). Moved on: `search::race` (F103, F104, filed under the search) and its callers to area 3.2, the command's `--settings FILE` (R183) to area 3.4, `run_local` to its caller (the web client) |
| m. `Member`, `Proof { goal, mode }`, `CheckError` (3.4, 3.7, 3.8) | done | nyxmnuxv: `Member` (a member of a sequent of its owner, the occurrence's number while no owner keeps a table), `Node`'s operands, `Dyadic` and `Fault` as members; the engines and the forest keep `OccId` and convert where a node is built or read, once per match arm. Then: `Proof` records its `goal` (set by `prove_goal` off the roots) and the `mode` it was found in, `new_of_goal`, `with_mode`, `occurrence`, `formula`; the checker, the size pass and the view conclude at the proof's own conclusion, so a goal proof checks, and `prove_goal` checks every proof it returns; `Error::GoalProof` (kind unsupported) from `from_proof`, the Rocq writer and `linlog check`, `Error::GoalMismatch` from `close_with` (F23's other half, F89); `Node::NAMES` and `TAGS` pinned by `name_lists_follow_the_variants`; `Dyadic` non-exhaustive; tests `a_goal_proof_records_its_goal` and the mismatch in `close_with_refuses_a_foreign_proof`. The keys `goal` and `mode` go on the wire with the wire level (item s) |
| n. `Rule` and `Named`, `Derivation`, `Inference`, `ViewOptions` (3.9) | done | xnszropr: `Rule` the sixteen one-sided rules, `Named { rule, side }` the rule a derivation shows (F60), names, spellings and labels unchanged, label tables indexed by `Named::index`, test `names_round_trip` in `rule.rs`. rnnrsvnv: `Inference` non-exhaustive with accessors and a sequent of members, `Derivation::occurrence`/`formula`. mnkunorl: `ViewOptions::sides` (one, two, or `Auto`, two-sided for a proof meant for intuitionistic mode) replaces the two-sided twins of `derivation`, `derivation_within` and `derivation_size`, whose view now carries the sides; `Proof::derivation_within` is the one public door (`Derivation::new` and `two_sided` crate-private) |
| o. `Interactive` (3.10) | done | `GoalId` apart from `InfId` (F69), `Step`/`Split`, `Applicable`/`Needs`, `apply(goal, &step)`, `split_passes(goal, &step)`, `Closed` from `close` (a refused graft keeps the goal open and the proof in the outcome), `close_all` with a result per goal (F68), `proof(limits, stop)`, `derivation()` fallible and `derivation_within`, `within`, `occurrence`/`formula`, `StepError::EmptyPremise` (F67); the command's session output unchanged (qmpstuqr). Then the reading kept in the state, O(1) (F66), and a linear read-back: two pointers for a split, a history walk that enters no later step's inferences (F21) |
| p. `ProofStructure`, `VertexId`, `Criterion`, `NetError` (3.11) | done | the net target set at kwxnmzvw (`net-before-retype.csv`) and at the retype kzwvyknq (`net-after-retype.csv`), committed in twlqlloz: the 64 rows all decided (49 proved, 15 unprovable) with equal `verdict`, `links` and `tests`; 242.8 s against 240.1 s summed, indicative. kwxnmzvw: `bench/net-targets.sh`, the net engine's target set (the decided rows of the second baseline's `engines` and `period-N` passes, forced onto the net engine on one thread), to run at kwxnmzvw and at the retype after the go. Then the retype: `VertexId` (vertex `i` is occurrence `i` in MLL), `Criterion` (`MLL`, `with_mix`, `of(mode)`), `NetError` with named fields and `Fragment`, `Mode`, `Rule` (kind unsupported, code `no_nets`); `new` refuses past `u32::MAX / 3 − 1` vertices (F9); `from_proof(proof, criterion, limits, stop)` matches every node (F49) within the memory bound; `is_correct(stop)`, `sequentialize(limits, stop)` (F12's net half; the proof records its mode); an id outside the structure answered, never a panic (H24), a scratch of another structure replaced (H23); the net engine sequentializes under the search's limits and stop; tests `foreign_ids_and_scratches`, `refusals` and the `wk` and `&` refusals in `desequentialize`. H24's `OccSet` half: the set is crate-private since wyuzsnql, so no public call panics |
| q. `Refutation` and `Disproof` (3.12, lock commit (3)) | done | `search/refutation.rs`: `Refutation` over the structs `Unbalanced` (the atom, no name: F80), `Equation` (with `needed`) and `StateEquation` (every place's weight: the atoms', the clauses used once by occurrence, and the clauses `live` dropped, a dead transition's with every clause under `?` sharing its arcs: R70, `once` gone); `Disproof` (sequent, goal off the roots, mode, refutation) in `Verdict::Unprovable(Box<Disproof>)`, built in `prove_goal`; `Display` of a refutation writes atoms as `#0`, the disproof's by name, which the command prints (its text lines unchanged). Lock commit (3): the unprovable outcome writes the disproof's `sequent` (and `goal`), the unbalanced atom by index, the state equation's new payload; both locks reblessed (four JSON outputs of the command, four core lines). Tests `refutations` (the disproof's sequent, mode and both displays) and `the_certificate_names_its_clauses` |
| r. the ordinary layer (3.13) | done | `Image::read_back(&proof, &limits, stop)` unfolds the proof itself, never compact, so a compact derivation cannot reach it (H22; `linear_derivation` crate-private), and asks the stop per inference; `Derivation::check(&limits, stop)` likewise, within `limits.work` inferences (F12's ordinary half); `Formulas::add` and `Sequent::new` refuse foreign ids (F13, `Space::Formula`); `Translation::target() -> Target` with the `Display` the command prints; `Logic`, `Translation`, `Options` (builders), `Rule` and `Inference` (private fields, accessors) non-exhaustive; `ordinary::decide` with `Outcome` and `Verdict` (R113), which the ordinary journey now calls; a certificate without atoms or hypotheses binds nothing (H29), compiled by the flake's `rocq` check too. Tests `the_checker_refuses_each_break` (one break per guard, each asserting its guard's reason: F11), `foreign_ids_are_refused`, `ordinary_certificate_without_binders`. F15 (the ordinary certificate's import as `prelude`) goes with the export options, item t; F52 (the read-back's allocations) with item u |
| s. the wire level, `wire::{Within, upgrade, LEVEL}` (7, lock commit (1)) | done | `wire` (public, `serialize`): `LEVEL` 1, `Readable` (sequent, proof, structure, session), `Within` and `upgrade` (the identity at level 1), `Error::Json` and `Error::Version` (`unsupported_version`, named before the other keys); every top-level document starts with `version`, nested values without; `roots`, `atoms`, `nodes` for `ids`, `var_dict`, `proof`, which are no longer read (D18); the mode by name; the proof's `mode` and `goal` keys (F89's wire half); the outcome's `version`, `linlog` and `checked`, its reasons and refutations tagged by `kind` with named fields; every reader counts against `limits.occurrences` (F27's library half); bounds past 2⁵³ written `null` and refused on reading; the batch record's level and name first. Lock commit (1): both locks reblessed (13 command outputs, the core lines), README's four JSON examples, every fixture in the old names regenerated (the tests' inline documents, the LLTP header report's nine proofs, each read back by `linlog check -i`); the command's lock and README's test read the crate's version as `…`. Test `wire_levels` (7.5's list). Ceilings raised in commits of their own placed before what needs them: suorpuok (read-json to 12 945 304: every JSON reader counts the occurrences, F27) and roxtvnkn (read-tptp to 31 574 017 at pkkmlunw, whose reader is unchanged: the codegen units' layout) |
| t. the exports' one `write` (9) | done | knkmnqxz: `export::Drawable` (the linear derivation or one of LK or LJ, by `From`) and one `write` per target, LaTeX, Typst, SVG and Rocq; the `derivation(…) -> String` twins and the `ordinary` functions are gone (F36), a `String` being an `fmt::Write`; every snapshot unchanged. Then `sequent(&sequent, mode, &options, &limits)` in LaTeX, Typst and SVG, two-sided by mode and refused past `limits.derivation_bytes` before it is laid out (F5's bound; the streaming SVG line it also asks for, and F82's one printer, are item u's). Then `svg::net(&net, &style, &limits, stop)`; `png`/`pdf::from_svg(…, &limits)` within `limits.memory_bytes`, their `memory` option gone (F63; the command passes `--memory-limit`, and `--style png.memory` is no key now). Then Rocq's `Options` non-exhaustive with builders, `lemma: Identifier` (lexical form and Rocq's keywords, `Error::InvalidOption`, F38's Rocq half) and `prelude: Option` (`None` each certificate's own: NanoYalla's import, the excluded middle's for a classical certificate over `Prop`, nothing for LJ; F15), every snapshot unchanged; tests in `certificates` and `ordinary_certificate_without_binders`. To item u: F4 (the SVG lengths' bounds) and F38's Typst lengths |
| u. the area's other findings, `lib.rs`'s allowances, the docs | done but F59 (placed in area 3.4 by the author) | rxyloqup: `Error::FamilySize` before a generator runs (F6). qruwnxlr: nets exist in linear mode only, said where asked. slmkyspp: Rocq names checked by a set, a spec's names hashed with the crate's hasher, a term's size pinned. nunzkxvw, yunpwlkw, rxluqnuk, lpmqyqro: every module page's first sentence and links, `must_use` on every changed copy, `# Errors` and `# Panics` on every public call (the lints `missing_errors_doc`, `missing_panics_doc`, `return_self_not_must_use` on). kzxonryx: `wildcard_enum_match_arm` denied in `sequents`, `occurrences`, `nets`, `export`, `ordinary` (F85). nxuroptl: tests of the checker's refusal before it judges (F17), the parser's guards (F54), the abbreviated report (F55), a saturated size not exact (F70), the options' JSON on their types (F18, F61). uqsrzlvo: atom names are identifiers that no keyword (`par`, `top`, `bot`, ordinary `true`, `false`) or reserved word (`forall`, `exists`) takes, `Error::AtomName` on JSON, `.spec` and ordinary atoms, a parse error naming a reserved word in the text (H19, HD5, §3.1). tomxvyyy: LLTP's `a-ob` refused (H6; no file of LLTP or ILTP has one). mqxyvmkm: `unicode-normalization`, every name read in NFC (H8, HD3). ormrprlp: `Style::check` with stated maxima and the layout's products in `i128` (F4), `typst::Length` (F38's Typst half), `escape` the one door into XML (F39). lstpykon, lszkzunv: the seven options values and the open types of §2.5 `#[non_exhaustive]`. nmvmpoqk: `lib.rs`'s `dead_code`/`unused_variables` gone, test-only and feature-only items behind their `cfg`, every feature combination and the test builds without default features free of warnings (F43). qvrkvzqt: the checker's work counts the entries its rules handle, its stop polled by them (H25's library half). xooqntxz: `Firm` pinned against every compact view (F71), the size's goal sum and recount pinned by samples that kill their mutants, the goal flag that no reading sets false removed (F57). ltxrqrpn: LLTP and TPTP parse errors placed in the file (F45). qlmtwmpu: a render refuses what no drawing has (F37). mnqzltkk: `Cadence::AUTO_SMALL`, `AUTO_PERIOD` (F81's library half). rqxzxxtm: `Forest::dual_literals` (F58). wlywsswv: the readers' rare paths and messages (F46). nxrzvvsn: the ordinary read-back compares without allocating (F52). psrqvmlm: a sequent's and a net's drawing written into the document once (F5). qrwnnuoy: one `Notation` for every target and `Display` (F82). rqsynupm: the ordinary labels one table, tested against the names (F72). Not done: F59 (below). Gates: rxyloqup to kzxonryx and nxuroptl passed; uqsrzlvo to rqsynupm passed all but the ratchet, and pass it under the raises row v lists |
| v. check rounds (stage 4), at most three; the fresh-context reviewer | done: three rounds and the reviewer, every finding fixed or decided | **Round 1** (the audit's nine lenses one effort lower, over the area's 105 findings and the diff yvzmnnmk..rqsynupm, with witness runs): 71 fixed, 28 partly, 6 not; 24 new (N1 to N24, two must-fix: N16, a batch that panicked when a worker could not start, and N17, a fault's formula written whole in an abbreviated report, 134 MB from 594 bytes). Fixed since: F1, F8, F30, N9, N24 (kmwqxrzt: every struct-like variant of a marked enum non-exhaustive, `ShapeError`'s named fields and `From`); F48, N17 (pnvqvulr); F11, F55, N22, N23 (qynportt); N1 (nzyrxlkw); F10, F73, N16 (srtwltlm); F87, F88 but `memo_entries`, N11, N12 (vkppkovl); F76, N18 (luzxwzwl); F37 (lzwyrkkn); F14 (tslrxmvs); F6 (nmoqmnmv); F20 (wzympysn); F3 (lzqxnsmx); N2, N3, N4, N6, N13 (yvylnprl); F44, N14, N19, N20 (pvryxory); F84, F85 (znqztktq); F51 (ypwwvlnq); N10 (yvlxqlms); F12's table (twnuptzl); F53's lines (wnyqmyoo); F78 (zwzwqyvw: a `with_*` per field on the export options, `svg::Style`, `TextOptions`, `Styles`, `Settings`); F36's twin writers (mxywqyto) and the ordinary `write_steps` (tuuywsqw). N5 and N7 were fixed in kmwqxrzt (`ordinary::Rule` closed again); N8, N21 and F32 were true at the round's head. Decided below: N15, F31, F36's rest, F72, F90, `memo_entries` (3.2), F56's engine names (3.2), and the command's halves of F5, F16, F27, H25 and F65 (3.4). Found while measuring the ratchet: the one printer (qrwnnuoy) took Typst 24 % and LaTeX and SVG 10 % more instructions through a `dyn` atom writer and an `i128` division; qnwmxkwm and musowqsy take them to +2.9 %, +2.5 % and +3.3 %. **Ceilings**: five raises inserted before the commits that need them (klsnlozv before uqsrzlvo: read-json; klmlrxkr before mqxyvmkm: search-additive-14; ryolrmpw before ormrprlp: render-svg, read-text; mvwmtwtm before qvrkvzqt: the two check journeys; twykwnwo before qrwnnuoy: the printing journeys), each to the most the gates measured from there on; the gate logs of uqsrzlvo to rqsynupm, whose code is unchanged, pass under them (re-read by script), and the commits from kmwqxrzt on are gated again. **Round 2** (the same lenses over round 1's open findings and the fixes rqsynupm..wunyrqsz): 44 fixed, 14 partly, none regressed; 8 new, all should-fix, all fixed: R2-1, a font's advance table no caller could build (tqpnlwnt: `Advances::table`, `Font::with_*`); R2-2, three records of this checklist (corrected below); R2-3 and R2-6, `nets::exist` answering another error and code than the net constructors (xkzksksx: `NetError::{Fragment, Mode}`, code `no_nets`, the engine mapping it to its own refusal); R2-4, `batch::Answer` not `Clone` (uspooysu); R2-5, the ordinary readers letting a repeated formula past the occurrence bound (sllwwuqu: every formula of the sequent counts); R2-7, the lists of names tied by no test (kqvuwqmr: `Engine` and `Bias` read through `ALL`, the four lists pinned); R2-8 with F11's and N22's rest, the contraction and split guards of the ordinary checker (opsnyxrl: four breaks, each failing inference pinned, each killing its mutant, checked by hand). Of the partly fixed: N2's net half (mvpnwrym), N6 and N14 (wuqkrwzq), N10 (pzvnplqp: every net error and refusal sampled and matched without a wildcard), F37 (slvzwmsm: a tag ends at its first unquoted `>`); the rest are decisions below (H25, F31, F36, F59, F72, F87, F88, N2's batch half) and F55's optional sample of a `?` step's Θ, not taken. Found beside the round: the TPTP fuzz target no longer built after `read_tptp` took limits (fixed in luzxwzwl, the fuzz lock's new crate in mqxyvmkm; tntyzwnz says so in `core.md`). **Round 3** (the lenses over round 2's open findings and the fixes wunyrqsz..pxmlmvoq): 12 fixed, 10 partly, all but two the recorded decisions and the command's halves (H25, F31, F36, F59, F72, F87, F88, N2's batch half); 5 new, one must-fix, all fixed: R3-3, `ALL` as an array on the open enumerations, whose length a new value changes (vrnnzurm: slices, `Fragment::NAMED` too, the rule in `core.md`); R3-1, R3-2 and R3-4, one finding thrice, the criterion telling its stop no bytes held (onlokywx); R3-5, the occurrence refusal written twice in the ordinary parser (vznvomvq); and the two partly fixed left, N6's last doc (qzkvvspq) and F37's end tag with white space (lwnslzsk). Three rounds is the most; the area's last word is the fresh-context reviewer's. **The fresh-context reviewer** (Fable 5.1 at high effort, soundness first, by witness programs: an independent two-sided IMALL prover against `prove_within` on 1 500 random sequents in intuitionistic and intuitionistic affine mode, no disagreement; truth tables and G4ip against the ordinary layer on 400 sequents in each logic and translation; nets against the focused engine on 400 MLL sequents; the Horn engine against the focused engines on 120 programs in five modes; 300 random sessions through JSON; adversarial documents, readers and limits under `catch_unwind`, no panic, every bound a refusal): one must-fix and four should-fix, all fixed. Must-fix: minimal logic under cbv refuted valid sequents that mention `false` (`false |- ~true`), since its `false` atom got no `!` there, every other atom one; the line predates the area and is the register's held-back H1, area 3.2's (orszuxqx, with both witnesses in the ordinary test's table for every translation, which fail without the fix; yxxuumxn adds H1's own witness; H21, the agreement test over generated sequents, is not covered and stays 3.2's). Should-fix: a goal proof whose goal has no output passed the intuitionistic check, and the roots in another order were a goal for `new_of_goal` but the sequent for `prove_goal` (upzxzltw: `Fault::Succedents` at the root under a reading, `Forest::is_roots` shared); the forest's and the reading's accessors had no `# Panics` (wnklszlm, seventeen of them); `Sequent`'s doc promised a text that reads back as itself, which `Display`'s one-sided text does not do for the sides (lvxvtynv, the decision below). **Mutants** (`mutants/run.sh --label area-3.1`, the nine batches whose files the area changed, cores 6 to 11, in the workspace `../linlog-mutants` at xznklvpo after a first start from the main checkout failed on a hand-made mutant there, my fault): 95 survivors against the baseline's 139 in the same batches, every file below its baseline (check 18 against 37, lltp 3 against 13, mist 18 against 24, ordinary-derivation 18 against 21, ordinary-parse 4 against 5, parse 3 against 5, search 12 against 19, horn-mod 13 against 15) but serialize, 6 against 0, all in the wire code the area added. Compared without line numbers, 35 are new; the tests of skrpywxo (the checker's work bound that suffices and its poll cadence), otvprpqq (serialize's six), nzmlxtos (the ordinary accessors, the ordinary check's work bound at its edge, what a parse expects inside a parenthesis), xnsuqqxp (when an outcome is checked, `with_check`, `Cadence::from`) and opknqnoz (a `.spec` rule's tokens at the bound, an LLTP error's place in a conjecture alone) kill 26 of them, each checked by hand on its mutant; the rest are decided below. **Fuzz** (`fuzz/run.sh`, every target after `fuzz/seed.sh` with the wire's new forms, cores 12 to 15; the earlier run kept in `target/fuzz-baseline`): no find in any target; the 3 312 artifacts of `ordinary_text` are the ones of 10-08 (the second turnstile's panic, fixed in ukxwvpnq), none newer; coverage above the baseline's in eight targets (json_proof 4 549 against 3 599, json_session 4 866 against 4 227, json_structure 3 892 against 3 394, json_sequent 3 558 against 2 855, sequent_text 2 105 against 1 432, ordinary_text 1 760 against 957, lltp 1 785 against 1 525, tptp 1 596 against 1 141), and below it in spec (1 476 against 1 678, stalled after 42 min, the reader refusing more since lwpolxzs and uqsrzlvo). **Gates**: every commit of the area that touches code passed (uqsrzlvo to rqsynupm by their logs under the inserted raises, every later one run again, musowqsy twice). **After the area, on the author's answers** (2026-10-10, through `planning`): the readers of design 7.2's table that the area had deferred, each a commit with its gate and its round trip test, F87 closed (kvrrwumq `Reason`, ywyytxwu `Refutation`, myllqqsn `Disproof` and the unprovable outcome read as one, utzltllk the `json_disproof` fuzz target); `Sequent`'s `Display` one-sided, answered; F59 placed in area 3.4 |

#### Area 3.1: decided unattended

- **`Forest::from_owned` stays public** (2.3 would have made it
  private): the command builds the forest of a sequent it owns, and the
  alternative, `Forest::within(&sequent)`, clones a sequent of up to
  fifty million occurrences. **Answered by the author (2026-10-10): kept.**
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
  body, so implementing it outside the crate breaks nothing. **Answered by the author (2026-10-10): sealed (pysomtmo, with `#[expect(unnameable_types)]` at the supertrait).**
- **Errors give no `source()`** (the design asks for it): every wrapping
  variant's message already holds the inner error's, and the command
  and the harness print errors with anyhow's `{:#}`, which appends each
  source; a `source()` would print the inner message twice in the
  pinned output, and dropping it from the message would empty the
  `message` of the written form. API guideline C-GOOD-ERR allows either. **Answered by the author (2026-10-10): kept.**
- **An error from `prove_within` carries no formula text**: the design
  would have `ShapeError` carry the subformula's text, since the caller
  holds no forest; the forest is a function of the sequent, so the
  caller rebuilds it (`Forest::new`) and describes, as the command does. **Answered by the author (2026-10-10): kept.**
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
- **Every struct-like variant of a marked enum is `#[non_exhaustive]`**
  (P3; the first round of the area's check found that named fields alone
  do not let a field be added, since a pattern naming every field
  breaks): `Error`, `Refusal`, `Fault`, `StepError`, `ShapeError` (whose
  two positional variants got named fields), `NetError`, `Split`,
  `Advances`. The command built two library errors itself; it now asks
  the library (`nets::exist`, `Interactive::goal` answering
  `StepError::NoGoal`).
- **`ProofStructure::occurrence` answers `Option<OccId>`** (the design
  has `OccId`), like `vertex` and `Member::occurrence`: H24 asks that an
  id outside the structure be answered, not panicked on or mistaken. **Answered by the author (2026-10-10): kept.**
- **A node `from_proof` cannot read is `NetError::Rule { node, rule }`**,
  a variant the design's list lacks (F49 asks for one naming the
  kind), with `Fragment` and `Mode` under the new code `no_nets`; the
  malformed and the invalid keep `invalid_net`.
- **`svg::net` takes the stop since item t**; what stays is the
  command's own `nets_exist`, which keeps its check of affine mode
  before it asks `nets::exist`, until the command's area (F28).
- **`Equation::needed` is an `i64`** (the design has `u64`): the count
  equation asks `#⊗ − #⅋ − #1 + #⊥ + 2` formulas, below zero where the
  `⅋` and the `1` outnumber the rest, as the message already writes it.
- **`Verdict::Unprovable` boxes its `Disproof`**, as `Proved` boxes its
  proof: a disproof holds a sequent and a refutation, and the verdict
  stays a pointer wide.
- **A `Disproof`, a `Reason` and a `Refutation` read back** (the
  author's answer, 2026-10-10, overruling the deferral to step 31 that
  this list first held: design 7.2's table stands): kvrrwumq (a reason
  by its `kind`), ywyytxwu (a refutation by its `kind`), myllqqsn (a
  disproof as a document of its own, `{version, sequent, mode,
  refutation, goal}`, every atom and occurrence it names checked against
  its sequent, and an unprovable outcome read as its disproof), utzltllk
  (its fuzz target, `json_disproof`, seeded with unprovable outcomes).
  A kind the reader does not know is refused (`Error::Json`, kind
  malformed), never read as another, which the wire module's rule of
  levels now says (a new kind raises the level). Each has its round
  trip test; F87 is closed.
- **A refutation's own `Display` writes atoms by number** (`#0`), as
  the errors' `Display` writes ids, and the disproof's writes names, as
  `describe` does: the refutation no longer carries a name.
- **`ordinary::decide` answers a refused read-back as that refusal**,
  not as "valid" without a derivation: `Verdict::Valid` carries the
  derivation, as the design has it, and a caller that wants the verdict
  alone reads `outcome.linear` after a refusal it can tell by its kind.
  The command keeps its own steps (its race, its output) until its area. **Answered by the author (2026-10-10): kept.**
- **The ordinary checker's bound is `limits.work`** (inferences
  checked), as the linear checker's is its nodes: the pass holds one
  inference's copies at a time, so a memory bound would bound nothing.
- **`wire::Readable` is a public, unsealed trait** (the design's
  `sealed::Readable`): a sealed one is unnameable, which the crate's lint
  check forbids, as for `Owner`; an outside implementation only reads its
  own type through the same calls. **Answered by the author (2026-10-10): sealed (pysomtmo).**
- **`upgrade` learns the library's own error through a thread-local**: a
  deserializer's error is text, so `wire::fail` keeps the refusal, the
  index error or the version that ended a read beside it, and `upgrade`
  answers that instead of `Error::Json`. No JSON value type is needed,
  so the readers stay format-agnostic.
- **The command keeps its readers of proof files and sessions** (plain
  serde within the default limits) until its area adopts `upgrade` with
  `--occurrence-limit` (3.4, F27's command half); its JSON sequent goes
  through `Within` without a bound and `admit` after, as its text does.
- **The ordinary forms and the linear derivation's written form** (7.3,
  F53, R4) wait for their first caller, the web client: a new form is no
  new level, so they come at level 1 whenever they come. **Answered by the author (2026-10-10): they wait for the web client.**
- **`recursion_limit` carries no `depth`**: `Reason::RecursionLimit` has
  none to write; the search area may add it with its reasons (F144).
- **A keyword names no atom in the text either**: `|- par` is refused
  where it read as the atom `par`, since the decision lists `par` among
  the keywords; a JSON, `.spec` or ordinary name is `AtomName`, a text
  one a parse error (a reserved word's with `reserved` set and the
  word's span), and the LLTP reader, which assembles text, answers the
  parse error with its place in the file. **Answered by the author (2026-10-10): kept.**
- **Ordinary names refuse `true` and `false` as well**, the ordinary
  syntax's constants, and the linear keywords, since an atom keeps its
  name in the image; a linear name may be `false`, minimal logic's
  atom. The `_` appended to that atom's name is gone with the case. **Answered by the author (2026-10-10): kept.**
- **`unicode-ident` and `unicode-normalization` are dependencies of
  every build**, not of `parse` (the design's place): names enter through
  JSON and `ordinary::Formulas::atom` in builds without it. **Answered by the author (2026-10-10): kept.**
- **`typst::Length` is refused under the key `typst`**: a length does not
  know its field, and the message names the value.
- **The SVG maxima are checked by every drawing, not by serde**: a front
  end makes a `Style` in code as well; `Style::check` is public for one
  that checks early.
- **The checker's unit of work is a node or an entry it handles** (H25):
  `limits.work` for a check now counts more than nodes, and the stop is
  asked every 65 536 units as well as every 4 096 nodes. A check stays
  quadratic on such a proof (comparing the sequents is the work); the
  command's `check`, which has no time limit, is area 3.4's.
- **F65 is the library's already**: `Refusal::Output::least_bytes`
  carries the compact view's lower bound and its `Display` says it; the
  command's `too_large` line prints the whole size (3.4), and the view's
  `Refusal::Memory` has none to carry.
- **F81's library half only**: `Cadence::AUTO_SMALL` and `AUTO_PERIOD`
  are public; `--test-period` is the command's (3.4), and
  `NET_MULTIPLICITY` stays private by the decision that dispatch
  thresholds are not options.
- **F37 refuses foreign SVG** (`Error::NotSvg`) rather than taking a
  drawing type: the renders' callers hold text, and the estimate knows
  the cost of the drawings' elements alone.
- **F72 keeps the linear labels in two arrays beside the `name`
  match**: the labels are the style's (`proofs/style.rs`, one array per
  convention, which a `Labels::Table` falls back on), the names the
  rule's (`proofs/rule.rs`, read by the session's JSON and `FromStr`),
  and `names_round_trip` ties the three entry by entry; one table of
  rows would move the drawing's labels into the calculus's module. The
  ordinary rules, whose labels were a second copy of the names that no
  test compared, read both off one table now.
- **F59 is not done here**: pure moves of `derivation.rs`, `check.rs`
  and `interactive.rs` would have made every diff of the area
  unreadable to its review and check rounds. The author placed them in
  area 3.4 (2026-10-10), as pure moves each passing its gate; they are
  in that area's item of the step file, `plan/28-audit-and-refactor.md`,
  and no longer in `plan/later.md`.
- **The test oracles need `parse`** (`oracle`, `generate`, `reference`,
  the nets' tests: `cfg(all(test, feature = "parse"))`), so the test
  build without default features has no dead helpers.
- **Some ceilings of the area come from the codegen units**: between
  commits that leave a journey's code alone its count moved by up to
  ±8 % (read-spec −7.8 %, read-text −5.5 % then +2.5 %,
  search-additive-14 +2.2 % at mqxyvmkm, read-lltp +2.06 % at musowqsy,
  whose raise trxxrstp went in before it when its gate failed), as at
  pkkmlunw; the raises
  (row v) take the counts the gates measured, and T3's link-time
  optimisation, which takes the layout out, re-records every ceiling.
- **The raises go in before the commits that need them, and the gate
  logs of the commits after them are re-read, not re-run**: inserting a
  commit that changes `bench/ceilings.csv` alone leaves the code of
  every later commit as it was gated, so their clippy, tests and
  feature checks stand and their ratchet is their logged counts against
  the new ceilings; a fix that lowers a count (qnwmxkwm, musowqsy) lands
  at the head, so the earlier gates stay valid, and T3 lowers the
  ceilings with every other count.
- **The forms of `Step`, `Applicable`, `batch::Answer` and
  `ordinary::Outcome` wait for their first caller** (design 7.2, N15):
  the web client's request and response and the batch's own record; an
  item without a caller waits for one, and a form added later needs no
  new level. The readers of `Reason` and `Refutation` are built, by the
  author's answer above. **Answered by the author (2026-10-10): they wait for the web client.**
- **F31's rest stays as it is**: `Error::ReadBack`'s `calculus` and
  `Logic::calculus()` are the name a message prints (`LK`, `LJ`), and a
  program branches on the logic it asked for; `text_size`'s pair is the
  width and the height of both trees; the state equation's weights are
  the pairs design 3.12 adopted.
- **Nine of the mutation run's new survivors take no test**: four are
  equivalent (in `ordinary::same`, the count of an element by `!=` in
  lists of one length and the choice between its two right paths; the
  capacity `push` reserves; a `.spec` count `k > 0` against `>= 0`,
  where no copies of a token are nothing), three move a progress counter no
  test reads (the ordinary check's and read-back's `n + 1`, the
  checker's poll at node 4 096 or 4 097), and two give the parser's
  refusal past `Forest::MOST` another variant, which only a limit above
  the forest's own reaches.
- **`Sequent`'s `Display` stays one-sided** (the reviewer offered a
  two-sided form whenever the sides are known): the command's pinned
  output is the one-sided text, and the reading's `Display` writes an
  intuitionistic sequent two-sided, which reads back with its sides, as
  the JSON keeps them; the doc now says so. The author's answer
  (2026-10-10): it stays one-sided, and a two-sided printer may come
  later where a caller needs it.
- **`svg::sequent` keeps its own dispatch** beside LaTeX's and Typst's
  shared `math` (F36's optional rest): it lays the sequent out in two
  notations, the drawing's and the title's, into a document, where the
  two text targets write one string between `$`s.
- **F36's last point stays**: `text_size`, `write_text` and
  `write_steps` are methods of each derivation, over one generic layout
  each, not one entry over `Drawable`: a text tree is no export, and the
  methods are where a caller holding a derivation looks
  (`ordinary::Derivation::write_steps` is new, tuuywsqw).
- **F90: the literal JSON pins in `core/tests/serialize.rs` stay**: the
  forms of errors, options and settings are pinned nowhere else (the
  lock's corpus has no such line), and the older literals are the
  readable statement of each form beside the lock's bulk.
- **The command's halves go to area 3.4**: F5 (`seq print` mapping its
  flags onto the limits, and a stop on the sequent writers, which are
  bounded by the estimate before anything is laid out and linear after
  it), F16 (the text readers within `--occurrence-limit`), F27 (proof
  files and sessions through `upgrade`), H25 (a bound on `check`) and
  F65 (the `too_large` line); `Statistics::memo_entries` as a `u64`
  goes with the search's counters to area 3.2, as do F56's private
  engine names.
- **`batch::Options::plan` is crate-private** (the design has a public
  `plan(Cores)`): `run` resolves `Cores::Auto` from the problems it
  reads ahead and plans with the result, and no front end plans a batch
  it does not run; `Plan` stays public, since each work call gets one. **Answered by the author (2026-10-10): kept.**
- **`Progress::item` names a session's goal only** (the design also
  names a batch's problem): a batch's searches take no stop of the
  caller's, since `batch::prove` asks its own cancel and `run`'s work
  closure is the caller's, which knows its problem; nothing would read
  the stamp. **Answered by the author (2026-10-10): kept.**
- **H1 is fixed here** (the held-back finding of the register, area
  3.2's): orszuxqx gives minimal logic's `false` an atom's `!` under
  cbv, which the fresh-context reviewer found as the same wrong answer,
  and yxxuumxn adds the register's own witness, `false |- a -> a`, to the
  ordinary table in every translation beside the reviewer's two. H21
  (an agreement test of the translations over generated sequents) is
  not covered: the table is by hand, and the reviewer's differential
  against truth tables and G4ip ran outside the tree; it stays area
  3.2's.
- **The command's probe findings were not reached by a rewrite here**:
  H26, H28 and H30 are in `cli/src/batch.rs`, which the area did not
  touch, and go to area 3.4 whole. H31's witness, a JSON atom named so
  that the Rocq verdict comment closes early, is refused since uqsrzlvo
  (an atom name is an identifier); the comment's own escape is still the
  command's (3.4). H27 is the search area's.

#### The command's wrong answers, first (session `step-28e`)

By the author's leave to reorder (2026-10-09), before area 3.2; each
with the test that would have caught it, each a commit of its own.

| item | state | evidence |
|---|---|---|
| F165: `--isolate` with bundled short flags | done | orkrklyn: the child gets the whole command line and its entry by the hidden `--entry-file`/`--entry-format`, reading no input of the batch's; test `isolated_entries_with_bundled_flags` (fails on the old code, passes); gate passed |
| H11: a typed SEQUENT ignored in a batch | done | slnpmxrp: refused beside `--files-from` or a format of many, exit 2; test `a_typed_sequent_is_no_batch_entry`; gate passed |
| H12: an empty batch exits 0 | done | lwyutvnn: a batch that holds no entry is an error, exit 2; test `an_empty_batch_is_an_error`; README and `--help` |
| H13: an interrupted batch exits 0 | done | rtsmunkm: unknown at best after an interrupt, a line on standard error; test `an_interrupted_batch_is_unknown` (SIGINT through `kill`, Unix; exit 0 on the old code) |
| H16: `load` in `interact` changes the question (HD4) | done | tutmumqw: `load` refuses another sequent or mode, the session goes on; test `load_keeps_the_question` (exit 0 on the old code); `HELP` left as it was, so the lock's `interact-help` holds |
| H5: a `.spec` file decided in affine mode (HD2) | done | myssxkvz: `SequentInput::mode`/`io::affine_for` in `prove`, the batch and `interact`; no flag takes weakening away, so none is refused; test `a_spec_file_is_affine`; the lock's `file-spec` call reblessed in the same commit (now answered in affine mode, as the finding requires) |
| H26: a JSON Lines line without `sequent`, a record read through `Value` | done | ouxkpull: the record read member by member (`Members`, serde_json's `raw_value`), its sequent through the JSON sequent reader; test `json_lines_records_are_whole` |
| H28: the command's batch hangs after an entry panics | done | rqnnqtor (H27's library half): the work's panic resumed at its place, the batch cancelled and its queue ended; test `a_panic_ends_the_batch_at_its_place` (100 problems, two workers; the old code hangs into the test's 20 s timeout). Witness with a panic injected into a scratch build of the command (reverted): `--cores across --workers 2`, `--cores within -j 2` and `--workers 1` each exit 101 within 7 ms on 42 entries |
| H30: an entry named `.` or `/` writes outside `--output DIR` | done | nsssnqmt: a name with no component but a root or `.` is no name (`names_a_file`), the entry named by its place; test `every_entry_file_lies_inside_the_directory`; collisions (H15) stay area 3.4's |
| H31: the Rocq verdict comment closed by `*)` | done | unkroolo: `note` writes `(*` and `*)` apart and doubles every `"`; unit test `rocq_notes_stay_comments` |
| The author's additions: `Owner` and `wire::Readable` sealed | done | pysomtmo: `crate::sealed`, `#[expect(unnameable_types)]` at each supertrait; the reachability check (`-W unreachable_pub -W unnameable_types`) clean with all features and none |
| The author's answers to area 3.1's departures recorded | done | marked "Answered by the author" in area 3.1's "decided unattended" |

#### The command's wrong answers: decided unattended

- **F165: the child takes its entry by hidden flags and reads no input
  of the batch's**, so the command line passes whole, however its flags
  are spelt (bundled, `=`, abbreviated). Set aside: rebuilding the
  child's command line from the parsed `ProveArgs` (the finding's first
  option), a writer per flag that a new flag could miss silently.
- **H13: an interrupted batch writes one line on standard error**
  (`interrupted: the rest of the input was not read`) besides exiting 3
  at best: the entries never read have no line of their own.
- **H16: `HELP`'s `load` line is unchanged**: the lock pins the session's
  `help`, and the refusal says itself when it happens; README says it.
- **H5: no flag is refused**: the file's affine is added to the flags'
  mode, and no flag of the command takes weakening away; `-i` and
  `--mix` keep their meaning, coverability being the same question under
  both (a Horn program's classical and intuitionistic affine provability
  agree). The library's `mist::Problem` gains no mode field: the command
  and the harness each know the format they read.
- **H28: no `catch_unwind` per entry in the command** (the finding's
  "better" option): the library's fix ends the batch at once with the
  panic (exit 101, as with one worker), no input known today panics the
  command (the ordinary parser's was fixed in area 3.1), and a catch that
  no input reaches would have no test; `--isolate` is the way to keep a
  batch going past a crash.
- **H30: a nameless line is named by its place** (`FILE:LINE`), as an
  empty name was; collisions of names inside the directory are H15's,
  area 3.4's.
- **H31: the comment escapes its text** (`( *`, `* )`, `""`) rather than
  writing atom names in their Rocq form: every line a Rocq note carries
  goes through `note`, names or not.

#### Area 3.2, item by item (session `step-28e`)

The search's findings (`area == "search"`) and what area 3.1 moved
here, in an order where wrong answers and crashes come first, then the
front door and its types, then the measured progress commit, the race,
the inner names and refactorings, the tests and the docs.

| item | state | evidence |
|---|---|---|
| a. F93 coverability's `tried`, F128 the Horn refutation kept, F139 a refused check `Unknown`, F117 a pool's panic cancels its siblings | done but F93 (a test, item j) | vkkvvktz F128 (test `counted_refutations_are_handed_back`); onxqtvpq F139 (`Reason::Unchecked`, `WorkLimit`; test `a_refused_check_is_unknown`); mrxutqxo F117 (`RaiseOnPanic`; test `a_panicking_stop_ends_every_worker`, 20 s timeout on the old code); gates passed |
| b. H18 the pool's depth per worker, a test at a raised limit on two threads | done | slwktolx: a pool's `&` is a level of its forks (or_depth + 1), and a stolen task starts at its thread's depth (`Runtime::waiting`, `depth_here`); the witness aborted before (two threads, limit 3 000) and answers unknown at 3 000, 5 000 and 10 000 on two and four threads; test `a_raised_limit_holds_on_the_pool` (aborts on the old code in release, the flake's build). Changes the search on a pool only: for the panel. **What checked the pool on two threads and more** after H18 and F92 changed how it splits `&` premises: the parallel tests in every gate (`focus::parallel::tests` comparing two and four threads with one on the generated samples, `a_raised_limit_holds_on_the_pool`, the cancelled and given-up premises, the net engine's cubes; the workspace's tests build core with `parallel`); the witnesses of H18 and F92 at `-j 2` and `-j 4`; the panel's counterexamples at jobs 2 and 4 (95 634 runs over the generated items, and 630 runs of nested `&` towers of 1 000 to 5 000 at limits around their depth: no overflow, no contradiction); the reviewer's jobs-2 and race configurations. Nothing measured the pool's speed: the target sets run on one thread; the efficiency area takes that first (the step's plan says so) |
| c. F92 a pool's `&` premises without copying the ancestors | done | slwktolx (with b): a spawn's branch is at most `LEVELS` + 1 slices; the witness (5 000 nested `&`, two threads, 64 MiB) 24 MB against 823 MB |
| d. `Reason` with named fields and `setting()`; `NotTaken` and `Error::EngineRefused`; `Engine::parallel` exhaustive (F141), `Engine::counters`; F140 no `expect` in the dispatch | done | tvvvvluq `Reason` (`RecursionLimit { depth }` on the wire, test `reasons_name_their_setting`); ssxwovxw `EngineRefused { engine, because: NotTaken }`, the internal `Pairs` and `Linker`, F141, F140 (`NoEngine`); lxkxvszu `Engine::counters` (`Counter`: key, label, meaning; test `counters_are_fields`) |
| e. `Goal` and the front door's stages (8.1), one place a verdict is built | done | mrvtkppt `Goal` (`Copy`, `conclusion`, `new` checking the members, `is_conclusion`), `prove_goal` and `engine_for` on it; nulpvyxn `conclude` |
| f. `Statistics`: `memo_entries` a `u64` (F102), `add` public, F144 the forward level apart from `copies` | done | tpnlumuk (`u64`, `add` public and the command's and harness's hand merges gone, `add_run` crate-private, test `statistics_add_up`); F144 in syqnnmno (item g) |
| g. the progress stop at every poll site, measured, the shim removed; `Limits::work` and `Reason::WorkLimit`; lock commit (4) with `work` and `forward_copies`; R243's test | done | txuvownw (`Work`, every poll told its units, `WorkLimit`, test `the_search_counts_its_work`; the additive path polls every 1 024 pairs, which took search-additive-14 from +5.7 % to −1.1 % against its ceiling, and tells the stop the pairs after its last poll when it ends, without which a search of fewer pairs reported no work and syqnnmno's lock and README failed); syqnnmno lock commit (4) (`work`, `forward_copies`, `copies` the decider's level; both locks, README); xwqpwrwo the calling thread's count apart and the polls' fast paths inline (the journeys' counts below their ceilings: additive −1.09 %, chain-128 −0.24 %); wtqxszqu a pool worker's and the second search's last units, lost before (the net engine on two threads reported 0 units against 128; `forced_links_are_made_once` and the new sum in `default_bias_takes_turns` catch it) |
| h. `search::race` with one account (F103, F104, F168), the command and the harness its callers | done | zoywtttw: `race` (one thread, a pool of `threads − 1` once `add_pool` says so, the first to decide answering), `Account::part_of` (both searches within one bound: 2.05 GB in one call before), `threads` counting every thread (`--jobs 2` runs two); the command's `searched` and the harness's `--pool-after` call it; test `the_race_counts_every_thread` |
| i. names inside (F56, F113) and the refactorings F107 to F121, F126, F132, F133, F135 | done, some in part (decided below) | zrxrxurw F113 and F56's engine names (`Run`, `Switches`, `Plan`, `Searched`; `Pairs` and `Linker` in ssxwovxw); ysrmnpss F133 (`Reach`, `Cover`, `FORWARD_PER_BACKWARD`); xxpxyrkt F107; mvzqmplu F108 (`Rank`); ptkmmtsy F109 (`prepare`, `SetUp`, `run_kept`: seven `too_many_arguments` gone); F110 by area 3.1's `Schedule` (tuooxrol); rurkzopt F111 in part (`Reason::as_set`, the count refutations in `counts`); lrmnkmvl F112; ymosnlvq F115 (`Forest::list`); nrswyxyn F116 (`lock`, `record`); mrxutqxo F117 (item a); lrynyvqu F118; vonxrkzw F119 (`Finished`); wxzwqtkz F120 in part (the batch's `Queue`); xzkwrqsm F126 in part (`Parents`); mqnqzzyk F132 (`is_head`, `clause_head`, `is_below`); mystyvxw F135 in part (`Program::inputs`, `outputs`); F114 and F121 decided below |
| j. tests: the Horn engine's (F94 to F97, F124, F125, F127, F129 to F131, F134), F98, F105, F106, F138, F143, F145, F122, F123, H21 | done but F122, F123 (decided below) | svlnkulr, by an agent in a workspace of its own and reviewed here, the Horn engine's tests: F93 (`tries_every_transition_on_every_element`), F94 (`caps_the_places_no_clause_raises`), F95 (`the_backward_search_finds_a_long_firing_sequence`), F96, F97, F105's simplex half (`the_simplex_polls_the_stop`), F124 (`the_simplex_gives_its_memory_back`), F125 (`charges_what_it_grows`, the least bound of each search and one byte under it), F130, F131, each checked by hand against its mutants; F129 by the existing `the_certificate_names_its_clauses`; F127 and F134 in round 1's tests (lnqoknxl, wkmxpnwx); onxqtvpq and qwukxtlu F98 (`a_refused_check_is_unknown`, `a_rejected_proof_is_an_error`); ymkmoyno F105's additive and net halves (`stops_between_pairs`, the net's failed-test poll) and F106 (the stream answered question by question, the window; the panic in rqnnqtor); kumqpqyx F138; qwukxtlu and tpnlumuk F143 (the refusals, `occurrence_limit`, `Engine::parallel`, the defaults, every refutation's sentence, `statistics_add_up`); wkouswzu F145 (the two ignored timing tests dropped); qromqtpw H21 (each logic's translations against each other and a truth table on 300 generated sequents, written by an agent and reviewed here) |
| k. docs: F91, F99 to F101, F118, F137, F142, F146 | done | lrynyvqu: F91, F99, F100 (a recursion level), F101, F118 (the private docs' links), F137 (the Horn and additive engines in the front door's docs), F142 (the default bias's measurements under a time limit), F146 (the reference's refutations with exponentials) |
| l. the panel, the target sets, check rounds (at most three), the fresh-context reviewer, `nix flake check` | done: `nix flake check` passed at lvlwttum and at vuuroryz, the area's head; every code commit of the area passed the gate (two streams, the 28 rewritten by the folds run again) | **The panel** (step 26's three lenses over the area's seven changes of the search, mrxutqxo..xwqpwrwo, the argument read again on Fable 5.1): counterexamples found none (2 812 classical and 674 intuitionistic generated items, 95 634 runs over 23 configurations each, against the committed reference and step 26's second one, and the base tree); the argument and the integers each refuted claim 7 with reproduced witnesses, both fixed in ysoxrlsz with a test each: the work bound ran a whole slice late beside the default bias's second search, and a turn's last poll lost its units (no verdict either way). **The target sets** at the head (rsrlxspt): the focused engine's 265 rows the same verdicts as `head-lto.csv` and the 225 decided rows equal in `nodes`, `splits`, `memo_hits` and `memo_entries`, 116.8 s of CPU against 118.5 s; the net engine's 64 rows equal to `net-after-retype.csv` in verdict, `links` and `tests`, 213.9 s against 238.2 s (the LTO build). The families at 5 s with the harness's new columns: 123 runs, no mismatch. **Round 1** (eight lenses one effort below the audit's, over the area's 62 findings and the diff xxsymnro..kuxlqyuq, with witness runs; every H finding's witness rerun): 37 fixed, 20 partly, 5 not (F122 and F123, with an agent then; F114 and F121, the deferrals, which the lens accepts; F134, then with the Horn tests' author); 18 new, all should-fix, no must-fix. Fixed since: the docs of the Horn engine's counters, `RowEntry` (toprrtwl); every thread's work through one `Counted`, and the caller told the poll that passes the bound (orkmzlry, `the_search_counts_its_work` extended: 616 units told against 632 counted before); the front door's docs (Outcome's reasons, Options, `prove_within`, `conclude` with the check off, `engine_for`, the `copies` counter) and `Statistics: Hash` (nrsqrszl); `NotTaken`'s JSON form documented and pinned, `ALL` against every engine and bias, `counters_are_fields`' dedup (xukokyku); the lock and taken helpers in one place, the literal lists, the plans' names, a worker's poll comment (nlklyupr); the forcing side as `Branch`, every arc through its transition, `Parents::room`, the frontier's `Successor` (svwmmxvp); `Engine` in one impl block (uolssulw); the batch within one sequent at its whole bound (wnqvqlpx, F104); the focused engines' fragment check (txktlzuu, F140). Decided below: F113's `Answer`, F119's flag, F132's walk, `held_bytes`, `Decide::decide`'s stop, F122 and F123; to area 3.4: the command's `--stats` labels and F141's half. The Horn and batch tests the round asked for went to the Horn tests' author: lnqoknxl F134 (the Horn searches' counters on five runs the search alone decides), wkmxpnwx F127 (the simplex's weights and units on four nets, its refutation after the search ran out of room), rsmuytnk F124 and F125 (the frontier's retry after the simplex gives memory back, the queue's refusal), mvpktnxo F106 (dropped results end the batch's workers and raise its cancel), each test checked by hand against its mutants; writing the last it found a worker that could wait at the window for good when the queue's end was missed by a `try_lock`, fixed in mpymkkvn (a flag beside the queue). From the mutation run: uwoymotr pins that the race answers with the side that decided, which no test did. **Round 2** (the same lenses over round 1's open findings and the fixes kuxlqyuq..mpymkkvn): 28 fixed, 10 partly, 5 not, all of these the recorded deferrals and decisions (F114, F121, F122, F123 not; F111, F119, F132, F133's last name, F141's command half, `held_bytes` partly); 11 new, one must-fix (a doc: `Options` said every bound binds the proof's check, where memory and work do), all fixed or decided: the docs of the check, `Options`, `Statistics` and a counter's label, `Engine: Ord`, `Reason::name`/`NAMES` and `Verdict::name`/`NAMES` with a test against the JSON kinds (yunqlozz); `Outcome::net` boxed as the design has it (uusmpvyq); the batch's end marked once (nwzxnmms); the work count and the shared-value helpers in `search/work.rs` (rvnourus, F111's small part); decided below: the design's `Engine::modes()`, the `over` flag's test, `give_back`'s guard, F132's half, F133's last name. **The fresh-context reviewer** (Fable 5.1 at high, soundness first, at txktlzuu, by witness programs: an independent unfocused prover written from the calculus, one-sided classical and two-sided intuitionistic, linear and affine, with and without Mix, on generated provable sequents and count-keeping mutants under the default and three of 25 configurations each, through `Forest::within` and `prove_goal` or `race` inside `catch_unwind`): 90 022 classical runs, 27 125 intuitionistic, 6 733 on Horn programs in five modes, 18 900 under stops and bounds (work, the caller's stop, memory, recursion, the race), 7 112 on larger sequents compared across configurations, and the command's probes of nested `&`, tensor chains, copy and memory bounds: no disagreement with the reference or between configurations, no panic, abort or overflow, no verdict flipped by a bound, no `Stopped` where the work bound fired, the work bound at most 119 units late. No must-fix and no should-fix finding; one observation, decided below. **Round 3** (the lenses over round 2's open findings and the fixes mpymkkvn..rvnourus; the last round): 8 fixed, 8 partly and 2 not, every one of them a recorded decision or an accepted leftover (the design's `modes()`, F132's half, F133's name, F134's and F135's rest, the `over` flag's test, `give_back`'s guard, F106's miss); 7 new, all should-fix, all fixed: a panic of the problems' iterator on a worker went past its place once the queue was taken poisoned or not (nwzxnmms made it so), now caught at its place (kpvouuzp, `a_panic_of_the_problems_ends_the_batch_at_its_place`, which fails without the fix); the outcome's verdict written through `Verdict::name`, `Refutation::name` and `NAMES` as the design has them, each pinned against its JSON word, `memo_entries`' additive pairs and `Work`'s batches in the docs, and the rules' line on an engine's modes (omrpsrvr). Three rounds is the most; the fixes after it are pinned by their tests and the gate. **Mutants** (`mutants/run.sh --label area-3.2`, the six batches whose files the area changed, cores 6 to 11, from the workspace `../linlog-mutants32`; search, horn-equation, horn-cover and horn-reach at qqlpyokz, once the Horn tests of round 1 were in, horn-mod and serialize at wnlmwzqz): survivors against the baseline, search 10 against 19, horn-mod 3 against 15, horn-equation 54 against 69, horn-cover 17 against 61, horn-reach 18 against 52, serialize the 6 of area 3.1's wire reader (`DisproofForm::within`, decided there). Compared without lines and the searches' renamed types, the new ones are: the race's thread count and its choice of side (`threads - 1`, the `settled` guards), which a race this small cannot steer, one side deciding before its first poll (uwoymotr pins the choice where it can be steered); `Goal::is_conclusion` replaced by `true`, killed since by utwktmsy; two equivalent ones, the Horn table's hash made constant (the table still works, slower) and `|` for `^` on the disjoint halves of `Parents::push`; and timeouts. As `planning` asked, every mutant that timed out under load was rerun alone on quiet cores with the whole suite and a timeout of 240 s, ten times the suite's time in the `mutants` profile: of 44 selected, 28 caught (`clause_head`'s two among them) and 16 that still time out, each making a test hang (the frontier's table probing for a slot it never finds, the simplex's `gcd` and `fraction` that never end, `Cover::before`'s counts, `Options::stack_size`'s shift, the race's single search ignoring the caller's stop), a test that never ends being one that fails. **Gates**: the 28 commits rewritten by the fold of the additive path's last pairs into txuvownw run again in two streams (cores 2 to 5 and 12 to 15, workspaces `../linlog-gate` and `../linlog-gate2`); the pool worker's lost units (xwqpwrwo, fixed in wtqxszqu) failed `forced_links_are_made_once` on the six commits between, so wtqxszqu was folded into xwqpwrwo, the commits after it keeping their trees and their gates |

#### Area 3.2: decided unattended

Each choice below is the session's; where it departs from the
signed-off design (`plan/notes/api.md`, its §14 answers, the rules'
"Decisions") it is marked so, with what the design said, as `planning`
asked on 2026-10-10.

- **`Progress::work` is the poll's own units, in the engine's unit**,
  and `done` every thread's: on one thread the `work` add up to `done`
  and to `Statistics::work`, as the design's comment on `Progress` has
  it. Every thread adds its units to the shared count in batches of
  4 096 and the rest when it ends, and the calling thread is asked with
  0 units at the driver's polls, so on several threads `done` runs ahead
  of the sum of `work` by what the others did, and the bound is late by
  at most a batch and a poll per thread. The units an engine counted
  after its last poll are no work, but for the additive path's last
  pairs, which it tells the stop when it ends.
- **An engine tells the stop the units it counts at the polls it
  already had** (the focused engine's slicing work, a literal or a
  failed test, a pair, a marking, a pivot), so no poll moved and no
  counter of a decided run changed. The additive path, which polled at
  every pair, polls every 1 024 since its poll cost 5 % of its
  instructions, and tells the stop the pairs after its last poll when it
  ends; the stop's answer to that last call comes after the search and
  changes nothing.
- **`copies` is the deciding search's level, else the backward
  search's, and `forward_copies` the forward search's** (F144): the
  design's "the level of the search that decided", with the undecided
  case, which it leaves open, given the backward search's level, the
  search whose bound a `CopyBound` verdict names; before, `copies` was
  the larger of the two.
- **`--jobs N` keeps a pool of N from the start; only the default
  `auto` races** one thread against a pool once the pool's delay has
  passed: a count the user gives is a count to use.
- **`NotTaken::Shape` carries no payload**, as the design's §8.6 has
  it: the additive engine's refusal no longer counts the goal's
  formulas, which the old `NotAdditive` did, and says "two additive-only
  formulas only".
- **F114 stays an out-parameter**: the hereditary flag rides on the
  recursion's frame cycle (`decide`, `decide_with`, `last_resort`,
  `mix`), where a returned pair grows every frame of the cycle that the
  `PER_LEVEL` stack measure bounds; the parameter is documented at each.
- **F121 is not done**: one `two_premises` with three closures sits on
  the same frame cycle, so it needs the stack per level measured again
  before and after, which this session did not have the quiet cores
  for; the hold before the second premise is the invariant
  `core-focus.md` states, and the checker catches a forgotten one.
- **F120 in part**: the batch's queue is named; the forcing factor's
  `bool` stays, since the crate has no left/right type (`occurrences::
  Side` is the reading's input and output) and one enum for one flag
  read in two places was not worth it; `literal_tensor` keeps reusing
  the links' pool as its node stack rather than a ninth pool with its
  own charge; the Horn proof's `(clause, reusable)` stays, `Clause::Once`
  naming a class where the proof needs the clause the replay chose; the
  additive path's `ordered` stays one call with its flag.
- **F111 in part**: the front door's `reason` and the count refutations
  moved out of the focused engine's module; `search/mod.rs` is not split
  into `options.rs` and `outcome.rs`, and `prove_stable` keeps its six
  steps in one function: moves of a thousand lines each, which a
  reviewer reads as a whole, for no change of behaviour.
- **F126 in part**: the line of parents is one type; the two searches'
  indices of transitions by place, the span of an element and the
  growth of their heaps stay apart, since they select different arcs
  (the first input, every output) and charge different buffers.
- **F135 in part**: `Program::inputs` and `outputs` replace the slices;
  `moved`, `Tableau::new` and `caps` keep their own merges of a
  transition's inputs and outputs, `moved` being the expansion's hot
  loop, where an iterator was not measured.
- **What the panel found and this session left as it is**, each read
  in the code and none a wrong verdict: (1) the recursion limit bounds
  a worker's stack through the levels it counts, and the frames it does
  not count (a fork's and a steal's, once per fork on a branch and once
  per steal nested on a thread) rest on `Limits`' reserve of a
  mebibyte; rayon's order bounds the nesting of steals by about the
  threads, which no test witnesses. (2) The race's proof check and
  refutation run under the search's limits outside the race's whole,
  so for the moment one side concludes while the other stops, the two
  can hold up to twice `--memory-limit`. (3) The checker counts its own
  work against `Limits::work` from zero, so a search and its check may
  do close to twice the bound: each long phase has the bound, as each
  has the memory bound. (4) The net engine's own count refusal
  (`counts_admit`) is counted again at the front door, which costs a
  second pass and can turn into `Exhausted` at a stop, which is always
  true of the verdict. (5) txuvownw's message says no poll moved; the
  additive path's did, from every pair to every 1 024, so a stop at its
  first poll now sees 1 024 pairs where it saw one; no counter of a
  decided run moved.
- **No fuzz run for area 3.2**: the area changed no reader a fuzz
  target covers; what it changed in `serialize` is the written form of
  `Reason`, `Statistics` and `NotTaken` and their derived serde readers,
  pinned by round trips, and the command's JSON Lines reader (H26) is the
  command's, area 3.4's to fuzz if any.
- **Kept from round 1, each read again**: F114 and F121 as above
  (the lens accepts them as deferrals); F113's `Answer`, which the design
  names twice, crate-private for an engine's answer and public in
  `batch`, the reference prover's being test-only; F132's
  `Reader::clause`, which keeps its own walk because it gives the places
  out in the order it meets the literals, and a second walk would number
  them otherwise and change the search's order; F119's flag beside a
  turn's `Finished`, a plain `bool` that one caller reads.
- **Moved to area 3.4, the command's**: the command's `--stats` lines
  read through `Engine::counters` (round 1), and F141's half in the
  command and the harness (their engine arguments from `Engine::ALL`).
- **`Decide::decide` takes `FnMut(u64) -> bool`** (departs from the
  design): the design's engines took the caller's `FnMut(Progress) ->
  bool` inward; an engine passes its units and the front door builds the
  `Progress` with the search's count across threads, which a worker
  could not, and the `Stop` that slices the default bias's searches
  needs the units alone. The public calls take `FnMut(Progress)` as the
  design has them.
- **A search tells its stop `held_bytes` 0** (departs from the design):
  the design has `Progress::held_bytes` the bytes the call holds; a
  search's are spread over the parts and shares of its account (the
  default bias's halves count in no parent), so the poll would read a
  number that is not the search's, and summing the shares at every poll
  costs the hot path; the limits' doc says so, and a front end that
  wants the bytes reads its own allocator.
- **F122 and F123 are not taken into the line** (`plan/later.md`, the
  focused engine's follow-ups, has the agent's measurements): F122's
  generator gives the intuitionistic samples real contractions but one
  of its samples doubles the debug suite (158 s in the classical engine,
  undecided within the copy bound), and F123's transposed mutants reach
  a search but one runs for over twenty minutes under Mix; both need the
  engine's time on those sequents looked at first, which is no fix of
  this area's, and neither found a wrong verdict.
- **Kept from round 2**: the design's crate-private `Engine::modes()`,
  which every `admits` and the dispatch's rows would read (departs from
  the design): each `admits` keeps its own mode test, which names the
  mode it refuses and costs nothing, and the dispatch's `Modes` stays
  the rows' own column; one source would serve a later engine that
  takes several modes, which none does yet. The batch's `over` flag has
  no test of the miss it fixes: that needs a worker blocked in the
  problems' iterator at the drop's instant, which no test arranges
  without sleeps (the drop itself is pinned). The coverability search's
  `give_back` guard against a second call is equivalent in effect, a
  retry that fails again. F132's half (record each clause's head when it
  is read, for the reading check) is left: the check recomputes a head
  per member in intuitionistic mode only, one walk at read time. F133's
  `focus::Problem` beside `batch::Problem` is left: one is crate-private
  and the other public, in different modules.
- **The reviewer's observation is the documented choice**: when the
  default bias's forward search ends at another bound (the recursion
  limit, the memory bound) and the backward one at its copy bound, the
  answer is `CopyBound`, whose `setting()` names `search.copies`; the
  reason of two undecided searches is the backward one's, as `copies`
  is its level, and the forward search's own bound is a matter of its
  plan (`|- bot, ~b, !(?((c + ~a)))` in affine mode with Mix at a
  recursion limit of 16, unprovable by hand).
- **The panel's second reference is step 26's**, written then by a
  fresh agent from the calculus alone and kept in that session's
  scratchpad, not one written anew: this area changed how the search is
  run (levels, depths, stops, races, the work count), not the calculus,
  and the counterexample agent runs both references beside the
  committed one.
- **One panel over the area's seven changes of the search** (the
  panic's stop, the Horn refutation handed back, the unchecked proof,
  the pool's levels and depths, the work bound, the race, the work
  count on every thread) rather than one per commit: each lens reads
  each commit's diff and the head, which keeps the agents within the
  machine's cores.

#### Area 3.3, item by item (session `step-28f`)

The efficiency findings (`area == "efficiency"`, F147 to F154), the
two printing and reading costs area 3.1's review named, and F114 and
F121 from area 3.2, after the two measurements the supervisor put
first. Each fix: a profile read as text, a test that pins the
behaviour where none does, the change, the journey's count and its
pinned time after, the ceiling lowered.

| item | state | evidence |
|---|---|---|
| a. the timed validation of counts against time again (`bench/counts-against-time.py`), in a quiet window | done | tympzvtu: `bench/counts-against-time/2026-10-13*`, 01:21–01:50 on core 2, the other cores 0 to 2 % busy; a drop of 19 to 65 % in count (opt-level 1 against 3) is a drop in time on all twenty journeys; a few percent (opt-level 2 against 3) orders count and time alike on 16 of 20 (not on `search-additive-14`, `derivation-chain-64`, `render-latex`, `render-typst`, whose rounds spread by up to 9 %); `check-qbf-20-2` and `check-wide-m1-2048` run 8 % faster at opt-level 2 at about the same count, as on 2026-10-09. So a claimed saving of a few percent is shown by pinned time beside the count |
| b. the pool's speed after H18 and F92: the target set and the second baseline's parallel pass at `-j 2` and `-j 4`, the head against kktkswpq | done: no verdict differs, the sums within 1 %; Mix 1.4 to 3.6 % slower at the head on every thread count, from the sequential search (fixed, row n) | unit `step28-pool`, 01:51–03:06: `-j 2` on CPUs 2–3 (performance cores, 5.1 GHz), `-j 4` on 4–7 (efficiency cores, 4.0 GHz, one shared L2, so the two settings compare within themselves only), each harness run at both commits back to back, the order alternating (scratch script `pool.sh`; the head's binary is qqooxnzz's code, before this area's changes). Rows decided by both: `-j 2` 171, head/base 1.009 in summed median time; `-j 4` 172, 0.999; no verdict differs; nodes equal on every row the pool does not split (Mix, qbf/24 and 40, 3-Partition-MLL, the Horn rows). Overlaps: this session's compiles and callgrind runs on CPUs 12–15 overlapped the target set's first run at `-j 2` (base 01:51–01:58: two builds, the journeys' counts, the core tests; head 01:58–02:05: the tests' end, the stack probes' builds and runs) and the start of the parallel pass at `-j 2` on the head (02:05:53–02:07:15, the probes' builds). Both pairs rerun alone (unit `step28-pool-rerun`, 03:07–03:34, the order reversed): 49 decided rows, head/base 1.018 (1.012 the first time); mix/8, 9, 10 +1.4, +3.6, +2.8 % (first +4.0, +3.5, +1.8 %; `-j 4` +3.0, +3.3, +2.5 %), qbf/40 and partition-yes/24 within 0.3 %. On one thread (core 2, mix/9, three runs each, alternating) kktkswpq 8.71 s, the head 8.86 s (+1.7 %): the gap is the sequential search's; callgrind on mix/8: 28.35 G against 28.99 G instructions, all of it `Run::poll_splits`, an outlined call per step of the split search (653 M). The runs: `bench/targets/pool-{kktkswpq,head}-j{2,4}.csv`, the reruns `pool-rerun-{kktkswpq,head}-j2.csv` |
| c. F152 the Horn program read once per call | done | zrquosrw: the task keeps the program the first reader reads (`Program::of`, a `OnceLock` in `Task`), with `live` applied there; `search-spec-chain` 39.11 M → 23.27 M instructions (−40.5 %) |
| d. F154 `optimize_atoms` once and without copies; read-json's names in NFC | done (read-json: no change, decided below) | qwkozsow: `merge_atoms` then one renumbering through an index, names moved; `read-text` 17.39 M → 12.27 M (−29.5 %), `read-lltp` 32.16 M → 27.07 M (−15.8 %), `read-spec` 19.60 M → 17.07 M (−12.9 %) |
| e. F151 `forced_factor` once per split, the forced side per occurrence | done | onxpxzuv: `Counts::forced` (one byte per occurrence, in place of `literal_tensor`), the split hands its forcing factor to the chain, the choice of factor a `match`; with the others below `search-chain-128` 1 198.8 M → 973.7 M (−18.8 %) |
| f. F150 the canonical key only where a member is renamed | done | lxppxszv: `Classes::renames` (the zone's words against the renamed occurrences) |
| g. F147 a stable sequent's member list by a word loop | done | kzszyqqz: `Context::list_into`, `iter` with a cursor over the extra copies; with F150, F151 and F149 `search-qbf-20-2` 628.8 M → 508.8 M (−19.1 %), `search-chain-64-intuitionistic` 151.7 M → 130.6 M (−13.9 %), `search-partition-no-5` 8.22 M → 7.43 M (−9.7 %) |
| h. F149 the memo's chunks without the zero fill | done | zsvqrusy: chunks allocated with capacity and filled record by record; `batch-families` 80.6 M → 70.5 M (−12.5 %, with the others) |
| i. F148 a sparse memo key | not done, a decision for `planning` (below) | the measured nets now go to the Horn engine; on the widest rows the focused engines still search the key is 14.5 % (`SYJ207+1.018`) and 29 % (`SYJ212+1.010`) of rows that end at the target set's copy bound in 108 and 30 ms, and 18 % and 43 % of the same searches without a copy bound, which run into the time limit; `plan/later.md` keeps it |
| j. F153 the net engine's quadratic parts | not done (decided below) | luzlznty took the fill of the search's arrays per search away (−4.6 % of `search-wide-m2-256`'s instructions) but no time (+0.2 % pinned, spread 3.4 %); uyvwqrqp reverts it; the rest changes `tests` or is a new sequentialization |
| k. the printing journeys' one printer through a `dyn` atom writer | done: no change (decided below) | profile of `render-typst` and `render-latex`: the walk (`Notation::term`) 50 to 60 %, the atom writer (an enum since area 3.1, no `dyn`) 15 to 25 %, LaTeX's escapes 22 % |
| l. F114 and F121, with the stack per level measured before and after | done | qmwrzops F121 (`Run::holding`, `two_premises`), mzoyxxpz F114 (the flag returned beside the `Found`); stack per level, release and debug, by a scratch probe (smallest stack reaching the recursion limit at 500 and 1 000): chain of searched splits 1 184/5 521 → 1 200/5 888 bytes, nested `&` 383/1 221 → 383/1 411, Mix of pairs 1 423/6 158 → 1 409/6 576; the Mix's level exceeded half of `PER_LEVEL` before the change too, so kuwluzvm raises it to 3 KiB and 13 KiB; after round 1's fixes (kxknzumq) 1 217, 383, 1 423 bytes in release and 5 921, 1 411, 6 609 in debug, twice the deepest within 3 072 and 13 312 |
| n. found: the split search's poll outlined since area 3.2's work count | done | wrpvvvmv: the count inline, the poll `#[cold]`, as the forced chain's; mix/8 28.99 G → 27.89 G (kktkswpq 28.35 G), mix/9 on one thread, pinned on core 2, three runs each alternating: 8.71 to 8.83 s at luzlznty, 8.59 to 8.64 s at wrpvvvmv (medians 8.77 and 8.64 s, −1.6 %); `search-partition-no-5` 7.43 M → 7.12 M |
| m. the ratchet lowered, the target sets, check rounds (at most three), the fresh-context reviewer, mutants and fuzzing of what changed, `nix flake check` | done: `nix flake check` passed at twzoumll, the area's head with its records; every code commit gated | **The gates**: every code commit of the area passed, zrquosrw to luzlznty in two streams 03:39–04:05, wrpvvvmv, xpvmvxsr, kxknzumq, ouuslowo, sttxsqwq and zsktvppn 04:03–04:24, uyvwqrqp 05:00–05:04 (scratch `gates/`); ukvsppkr, which changes `bench/ceilings.csv` alone, through `nix flake check`'s ratchet and tests at the area's end (below). **The target set** at the head (zsktvppn, 04:25–04:32, quiet; `bench/targets/after-efficiency.csv`): the 265 rows the verdicts of `head-lto.csv`, every decided row equal in `nodes`, `splits`, `memo_hits` and `memo_entries`; CPU over the decided rows 86.5 s against 96.1 s (−10 %), its rows over a second the Mix's (mix/8, 9, 10: −9.8, −9.4, −9.9 %). Run at the head only, as area 3.2 did, not after each commit that touches `search/` (departs from the step's Verification): every commit claims the counters kept, the lenses read each one, and the head equals the oracle on every decided row. **The net engine's target set** at the head (05:06–05:10, `bench/targets/net-after-efficiency.csv`): the 38 rows the verdicts, `links` and `tests` of `net-after-retype.csv`. **The families** at 5 s (123 runs): no mismatch. **qcover** through the harness (177 `.spec` files, `--engines horn,auto`, 5 s, core 4, while the mutants and fuzzers ran on cores 6 to 15): no verdict against a stated one; 58 proved, 103 unprovable, 16 unknown on both rows against area 3.2's 59, 103, 15: `howait__all_workers_finished_if_wait_over__depth_1` (4.86 s in `bench/defaults/horn-qcover.csv`) timed out, and so does it at qqooxnzz under that load (4.3 to 5.3 s either binary, two runs each, core 4); `x0_AA_q5` takes 7.8 to 8.4 s either way there, and callgrind counts it 72.94 G at qqooxnzz and 73.04 G at the head (+0.13 %, the same nodes). **The mutation run** (`mutants/run.sh --label area-3.3 search horn-mod`, the two files of the area in its scope, from the main checkout at ukvsppkr, 05:11–06:03, cores 6 to 11; `mutants/area-3.3/`): `horn-mod` 3 surviving, the same three as area 3.2's (15 in the baseline), `search` 9 against area 3.2's 10 (19 in the baseline), none new (`Goal::is_conclusion` replaced by `true` now caught). Four of them timed out in the second pass; each was rerun alone (unit `step28-mutrerun`, 06:04–07:28, the workspace `../linlog-round1` at ukvsppkr, cores 6 to 11, one mutant at a time against the whole workspace's suite, timeout 900 s, the unmutated suite 21 s): each times out again, but three fail a test first, which the unmutated suite passes, so they count as caught: `DEFAULT_MEMO_LIMIT`'s `<<` for `>>` (`search::additive::tests::memoized` at 1.3 s: a memo limit of 0 finds no hit) and `clause_head`'s two guards (`search::focus::tests::the_horn_test_reads_the_goal` at once). The fourth, the race's `||` for `&&` at `search/mod.rs:436` (the second side then stops only when the caller stops and the first side has decided), fails no test and hangs `readme_examples`, so it survives, as in area 3.2. Counted so: `search` 8 surviving, `horn-mod` 1 (`|` for `^` on the disjoint halves of `Parents::push`, equivalent); `mutants/area-3.3/` keeps the run's own lists. The rerun's `--re` also took mutants it does not match, cause not looked into (the field deletions of the `Options::with_*` and of the Horn engine's `Answer`, 12 in all, all caught). **The fuzz targets** of the readers `optimize` serves (`sequent_text`, `ordinary_text`, `tptp`, `lltp`, `spec`, `json_sequent`; 05:12–06:03, cores 12 to 15, stall 600 s, cap 1 800 s; `target/fuzz/summary.tsv`, the run of 2026-10-09 kept beside it): no new find (the 3 312 files of `ordinary_text` are those of 2026-10-08, of the panic fixed then; none is newer than the run's start), coverage up on every target (2 105 to 2 136 edges on `sequent_text`, 1 785 to 1 823 on `lltp`, 1 476 to 1 506 on `spec`). **The ceilings** (ukvsppkr): every journey that went down takes the larger of the shell's count and the flake check's (`nix build .#checks.x86_64-linux.ratchet`, 05:10), `render-svg` kept (the flake's build counts it 0.9 % above its ceiling, the shell 0.45 % below). **Round 1** (eight lenses one effort below the audit's, over the area's ten findings and the diff qqooxnzz..wrpvvvmv, with witness runs; the readiness and API lenses not run, since the area changes no public item, wire form or requirement of the register; scratch `round1/`): perf: F149, F150, F151, F152, F154 fixed, F147 partly (the tally per distinct member not done, which changes nothing on qbf), F148 and F153 partly, both decisions upheld, F153's reason corrected (partner counts kept per literal would keep `tests`; they are no local update); idiom: F114 and F121 fixed. New, all should-fix, no must-fix: docs and coherence, `Counts::literal_tensor` still named in core-focus.md, and measurement history in the rules the diff changed; idiom, the copies' walk twice in `Context`, `distinct` beside `renamed`, `Forcing` a 4-tuple, `#[inline(always)]` without a recorded reason, `premises` and `parts` one-line wrappers, two docs unwrapped; soundness of refutations, no test of `renumber_atoms`'s first-naming order; tests, four gaps the mutants of the changed lines showed (`Forced::Duals` in nested tensors, the canonical key's sharing, the atom order, the graph's restart). Fixed: xpvmvxsr (the copies' walk, `distinct`), kxknzumq (`Forcing` a struct, the docs, the wrappers kept with their reason: inlined, a debug level grows from 5 921 to 6 400 bytes on the chain and from 6 609 to 7 137 on a Mix, past half of `PER_LEVEL`; the allowance an `#[expect]`), ouuslowo (the atom order), tmrwuquz (the rules), sttxsqwq (the tests; the graph test kills the restart's mutants, checked by hand). The soundness lenses found nothing: the refutations' lens argued points (a) to (h) and ran the suite and the 29 quick rows of the target set's families, every counter equal to `after-bias.csv`; the checker's lens ran a differential test of the graph's search, old against new, 2 910 searches with 37 restarts, every field equal. Not taken: the `<` to `<=` mutant of the graph's discovery test, equivalent in effect (a root found again is searched again at a later time, and every comparison `low` and the bridge test make keeps its outcome; a triangle, a square and a pendant before and after the cycle worked by hand), and the mutants of `Context::clear` and `Memo::reserve`, outside the diff's lines, which change sizes, not answers. **Round 2** (the lenses with round-1 findings, over them and the fixes wrpvvvmv..sttxsqwq: idiom, docs with coherence, tests, soundness of refutations): every round-1 finding fixed, the declined two upheld (the idiom lens: at opt-level 0 no stack colouring, so both arms' locals would sit in the frame; the tests lens: a differential run of the search with the `<=` mutant against the real one, 108 000 searches over 18 000 random linked nets, the first root found again 18 637 times, trees, reach, components and every edge's bridge flag equal); no new must-fix or should-fix, so no third round. Taste kept apart: `forced_factor`'s preference for a factor closed in place has no test that sees it (four mutants; the splits, nodes, entries and recursion are equal either way on chains of 40 000 literals). Found while counting the fixes: xpvmvxsr's shared walk was called from `canonical_from`, +1 % on `search-partition-no-5` (7.19 M against 7.12 M), attributed by building wrpvvvmv, xpvmvxsr and kxknzumq; zsktvppn inlines it, the counts those before (7 120 724, and `search-qbf-20-2` 506 226 963). **The fresh-context reviewer** (Opus 5.5 at `high`, the area being efficiency; read only, during the timed runs): the search unchanged wherever a commit claims it, the docs, comments and allowances as the conventions ask; five should-fix, all taken: no pinned time per fix (the per-fix table in "Area 3.3: the journeys before and after", which also showed luzlznty's claim unmet, reverted in uyvwqrqp), the target set at the head only (recorded as a departure above), F148 judged at the target set's copy bound only (measured without one, a decision for `planning`), the `PER_LEVEL` raise not marked as a departure and the alternatives not named (done in the decisions), `bench.md` counting four journeys and naming five, `core-search.md`'s list of a task's parts without the program's cell (yzwlvvus). **`nix flake check`** at twzoumll (13f00f04: ukvsppkr's code and ceilings with the records; 07:30–07:34, nothing else running): passed; every journey within the ratchet's 2 % of its ceiling, the two over it `render-svg` (+0.94 %, kept, above) and `check-qbf-20-2` (+0.70 %, the shell +1.3 %; its ceiling left, its pinned time +0.1 % in "Area 3.3: the journeys before and after") |

#### Area 3.3: decided unattended

Each choice below is the session's; where it departs from the
signed-off design or from a finding's proposed fix it is marked so,
with what the design or the finding said.

- **read-json keeps its readers** (area 3.1's review): its profile is
  serde_json's (the names' check 2.9 %, `merge_atoms` 2.6 %), so no
  change of the crate's code takes a share worth a commit.
- **The printers keep one walk through an enum atom writer** (area
  3.1's review, item k): the `dyn` writer the review named is an enum
  since area 3.1; the walk (`Notation::term`) is 50 to 60 % of
  `render-typst` and `render-latex`, the atom writer 15 to 25 %, LaTeX's
  escapes 22 %. `Walk::new` initializing its sixteen inline steps is
  about 4 %, not worth a change of the walk every printer shares.
- **`Tally::add` stays one call per member** (28 % of `search-qbf-20-2`
  at 113 instructions a call): adding a distinct member once with its
  count saves nothing where members repeat as rarely as there.
- **F121 is `Run::holding` and `Run::two_premises`** (reverses area
  3.2's "F121 is not done"): `two_premises` serves `premises` and
  `parts`; `with` and the chain's frame call `holding` alone, since
  `with`'s two closures would borrow its list both ways and the frame's
  first proof is already held. Measured: the stack per level of the
  three recursing shapes moved by +7 to +16 % in debug builds and by
  −14 to +16 bytes in release.
- **F114 returns the flag beside the decision** (reverses area 3.2's
  "F114 stays an out-parameter"): `decide`, `decide_with`,
  `last_resort` and `mix` return `(Found, bool)` as `prove_stable` and
  `prove_part` already did, so the flag still travels as a value no
  other rule's failure carries. Set aside: keeping the out-parameter
  (area 3.2's choice, which the finding faults), and a field of `Found`,
  which would let any rule's failure carry it.
- **`Limits::PER_LEVEL` is raised** (3 KiB release, 13 KiB debug), a
  finding of this area's (departs from the design: `plan/notes/api.md`
  gives `stack_bytes` "2 304 bytes a level optimized, measured on
  x86-64"): the Mix's levels (`prove`, `decide`, `mix`, `search_splits`,
  `parts`) are deeper than the chain of splits the bound was measured
  on, and left it at 1.6 times their depth in release where the rules
  claim twice. Set aside: shrinking the Mix cycle's frames (its locals
  boxed or moved into the engine's pools), a change of the search's
  hottest functions for a stack the default depth already fits in a
  release build (2 048 levels of 3 KiB and the reserve are 7 MiB of the
  main thread's 8); and lowering the default depth, which would end
  searches the rules record as reaching their answers.
- **The split search's poll is inlined by hand** (`#[inline(always)]`,
  its slow branch `#[cold]`), a finding of this area's from the pool
  measurement: since area 3.2's work count the compiler called it at
  every step of the split search, 653 M of mix/8's 28.99 G instructions
  (kktkswpq 28.35 G, now 27.89 G); the forced chain's poll already had
  that form. Set aside: `#[inline]`, a hint the compiler had already
  declined, and counting the steps in the split loop's own local, which
  would spread the poll's bookkeeping over its two callers.
- **F148 is not done** (departs from the finding's fix, a sparse memo
  key): the measured hot spot moved. The three nets it was measured on
  (`CloudDeployment_deploy_5_b_100_1`, `DLCround_dlcro_07_b_5_1`,
  `FlexibleBarrier_flexbar_18_b_5_1`) are now proved by the Horn
  engine in 4 to 8 ms (`head-lto.csv`); the target set's widest rows
  that the focused engines still search (`SYJ212+1.010` at 27 105
  occurrences, `SYJ207+1.018`, `SYJ201+1.019`) end at their copy bound
  in 30 to 108 ms, and no focused row ends at the memory bound, where a
  smaller record would hold more entries. On those rows the key's hash
  and compare are 14.5 % (`SYJ207+1.018`) and 29 % (`SYJ212+1.010`) of
  the instructions (callgrind, 2026-10-13); on `SYJ207+1.018` sorting
  `Θ` by class again is another 12.9 %, a cache of one `Θ` that
  alternating branches miss. Under the command's default, with no copy
  bound, both rows run into the time limit (10 s: copy levels 6 and 4),
  and there the key weighs more: hashing 36.7 % and the lookup 6.6 % of
  `SYJ212+1.010`, 13.0 % and 5.1 % of `SYJ207+1.018` (callgrind, 60 s
  each; the fresh-context reviewer's point), so a sparse key would make
  the widest two-sided searches up to about 1.7 times as fast per
  stable sequent, for no verdict the target set shows. It is a rewrite
  of `memo.rs` and the shared memo, with `Θ` interned to gain anything
  on nets whose `Θ` spans the forest: **a decision for `planning`**;
  `plan/later.md` keeps it with these numbers.
- **F153 is not done** (departs from the finding's fix): its cheapest
  part, the graph's search without the fill of `disc` over the width per
  search, was made (luzlznty: discovery times rising from search to
  search; a list of the vertices reached, tried first, cost +0.3 %) and
  saved 4.6 % of `search-wide-m2-256`'s instructions (260.4 M to
  248.5 M) but no time: pinned, 7.95 ms before, 7.96 ms after and
  8.02 ms with the search's first time in a local (nine rounds, spread
  3.4 %), a memset being cheap per instruction; uyvwqrqp reverts it, as
  a few percent claimed must show in pinned time. The rest is not
  done (departs from the finding's fix): a test period that grows
  changes `Statistics::tests`, which the lock prints with `--stats`;
  admissible-partner counts kept per literal keep it, but a link changes
  `joined` and the common-ancestor rule for literals far from it, so the
  counts are no local update (the performance lens's correction in
  round 1); and the bridges kept across
  stages with the sides assigned from the smaller one is a new
  sequentialization algorithm, all for `wide-m2` at 2 048 links in
  0.6 s; `plan/later.md`, "Net-engine pruning and routing", holds them.
- **The mutation run ran from the main checkout** (departs from the
  supervisor's rule of a jj workspace of its own at the head):
  `mutants/run.sh --label area-3.3 search horn-mod`, started at 05:11
  with the working copy on ukvsppkr (e628a366, whose code is uyvwqrqp's:
  only records lie between); the script copies the tree at each batch
  and pass, so no code changed in the checkout until it ended, and a
  mutant that timed out is rerun alone before it counts.
- **The check rounds run eight of the audit's ten lenses**: the
  readiness lens and the API lens read the register's requirements and
  the public surface, which the area does not change (no public item,
  wire form or option; `Limits::stack_bytes` answers more for the same
  depth, which the input and bounds lens judged).
- **`premises` and `parts` stay functions** (the idiom lens's fix
  declined): inlined into `search_splits`, both rules' locals sit in its
  frame in a debug build, and a level of the chain grows from 5 921 to
  6 400 bytes and of a Mix from 6 609 to 7 137, past half of
  `PER_LEVEL`'s 13 KiB; the allowance became an `#[expect]` with its
  reason and the doc says why. The margin left in debug builds is 94
  bytes a level (twice 6 609 against 13 312): a frame added to the cycle
  measures again, as the rules say.

#### Area 3.3: the journeys before and after

The counts: callgrind, the shell's release build (fat link-time
optimisation), before the area's ceilings (`bench/ceilings.csv` at
qqooxnzz), after at the area's head (`ratchet`, 05:03). The times:
`journey --repeat` pinned on core 2, the other cores 0 to 3 % busy, the
median of five rounds with the two builds' order rotating (a scratch
copy of `bench/counts-against-time.py`'s timing for two builds),
qqooxnzz against zsktvppn (04:32–04:51); uyvwqrqp, after them, changes
only `search-wide-m2-256`, whose count it takes back and whose time
moved by neither commit. A count that fell by a fifth fell in time by
half as much or more; the printers, the derivation and the checker
move within their rounds' spread, as the validation of 2026-10-13
found.

| journey | count before | count after | change | time before (ms) | time after (ms) | change |
|---|--:|--:|--:|--:|--:|--:|
| `search-qbf-20-2` | 636.04 M | 506.23 M | -20.4% | 27.353 | 24.760 | -9.5% |
| `search-chain-128` | 1202.71 M | 972.95 M | -19.1% | 37.787 | 33.843 | -10.4% |
| `search-partition-no-5` | 8.28 M | 7.12 M | -14.0% | 0.352 | 0.317 | -9.9% |
| `search-wide-m2-256` | 260.39 M | 260.44 M | +0.0% | 7.724 | 7.817 | +1.2% |
| `search-spec-chain` | 38.43 M | 23.27 M | -39.4% | 1.322 | 0.766 | -42.0% |
| `search-chain-64-intuitionistic` | 152.13 M | 123.66 M | -18.7% | 4.921 | 4.417 | -10.2% |
| `search-additive-14` | 591.40 M | 584.99 M | -1.1% | 56.400 | 56.444 | +0.1% |
| `read-text` | 17.39 M | 12.27 M | -29.5% | 0.843 | 0.688 | -18.5% |
| `read-json` | 12.61 M | 12.48 M | -1.1% | 0.362 | 0.345 | -4.6% |
| `read-lltp` | 32.17 M | 27.07 M | -15.9% | 1.418 | 1.338 | -5.6% |
| `read-tptp` | 32.26 M | 32.25 M | -0.0% | 1.289 | 1.266 | -1.8% |
| `read-spec` | 19.61 M | 17.06 M | -13.0% | 0.863 | 0.610 | -29.3% |
| `check-qbf-20-2` | 136.74 M | 138.58 M | +1.3% | 7.502 | 7.507 | +0.1% |
| `check-wide-m1-2048` | 8.80 M | 8.79 M | -0.0% | 0.502 | 0.501 | -0.1% |
| `derivation-chain-64` | 9.73 M | 9.75 M | +0.2% | 0.396 | 0.369 | -6.9% |
| `render-latex` | 5.74 M | 5.74 M | +0.1% | 0.212 | 0.205 | -3.2% |
| `render-typst` | 4.79 M | 4.79 M | -0.0% | 0.168 | 0.164 | -2.5% |
| `render-svg` | 84.85 M | 84.47 M | -0.4% | 3.252 | 3.215 | -1.1% |
| `batch-families` | 79.92 M | 69.57 M | -12.9% | 3.009 | 2.823 | -6.2% |
| `ordinary-pigeons` | 32.67 M | 32.55 M | -0.4% | 1.569 | 1.565 | -0.2% |

Each fix against its parent, the same timing on the journeys it names
(five rounds; spread is the largest of a build's rounds):

| commit | journey | parent (ms) | commit (ms) | change | spread |
|---|---|--:|--:|--:|--:|
| zrquosrw F152 | `search-spec-chain` | 1.344 | 0.780 | −42.0 % | 1.3 % |
| qwkozsow F154 | `read-text` | 0.858 | 0.696 | −18.9 % | 0.3 % |
| | `read-lltp` | 1.449 | 1.346 | −7.1 % | 1.2 % |
| | `read-spec` | 0.879 | 0.617 | −29.8 % | 2.0 % |
| zsvqrusy F149 | `batch-families` | 3.066 | 2.971 | −3.1 % | 0.6 % |
| | `search-qbf-20-2` | 27.982 | 27.645 | −1.2 % | 3.2 % |
| onxpxzuv F151 | `search-chain-128` | 38.911 | 35.602 | −8.5 % | 0.8 % |
| lxppxszv F150 | `search-qbf-20-2` | 27.381 | 25.739 | −6.0 % | 2.1 % |
| kzszyqqz F147 | `search-qbf-20-2` | 25.708 | 24.960 | −2.9 % | 1.5 % |
| | `search-partition-no-5` | 0.327 | 0.324 | −0.9 % | 1.3 % |
| qmwrzops F121 | `search-chain-128` | 34.904 | 35.267 | +1.0 % | 1.4 % |
| | `search-partition-no-5` | 0.323 | 0.321 | −0.4 % | 0.6 % |
| mzoyxxpz F114 | `search-partition-no-5` | 0.322 | 0.325 | +1.2 % | 1.0 % |
| luzlznty (reverted) | `search-wide-m2-256`, nine rounds | 7.945 | 7.959 | +0.2 % | 3.4 % |
| wrpvvvmv | mix/9, one thread (s) | 8.773 | 8.636 | −1.6 % | runs apart |
| | `search-partition-no-5` | 0.324 | 0.325 | +0.3 % | 1.9 % |
| xpvmvxsr | `search-partition-no-5` | 0.326 | 0.319 | −2.2 % | 3.5 % |
| zsktvppn | `search-partition-no-5` | 0.320 | 0.324 | +1.1 % | 1.0 % |

F121 and F114 claim no gain and cost none beyond the spread; F149's
claim is `batch-families`' (its qbf row is within the spread).


#### Area 3.4, item by item (session `step-28g`)

The command, the harness, the flake and the documents: the open
findings of `area == "command-harness-flake-docs"` (F155 to F226
without F165, H14, H15, H20), the halves earlier areas moved here (F5,
F16, F27, H25, F65; the `--stats` lines through `Engine::counters`, the
engine arguments from `Engine::ALL`, the harness's reason column through
`Reason::name`, F141's command half), `--settings FILE` (R183), F59's
pure moves, the race mutant at `search/mod.rs:436`, the ultra reviews'
findings, and the step's last evidence.

| item | state | evidence |
|---|---|---|
| a. F59: `derivation.rs`, `check.rs`, `interactive.rs` split as pure moves | done | ppsxqlln (`proofs/check/{mod,zones,error,pass}.rs`), zqmrmnwu (`proofs/derivation/{mod,view,build}.rs`), mpuzmwmy (`proofs/interactive/{mod,expand,replay,terms}.rs`): items moved whole, private ones opened to the parent (`pub(super)`) where a sibling or the tests read them, the re-exports and every public path unchanged; the rules' paths (`core.md`, `core-proofs.md`, `core-derivations.md`, `claude-infra.md`, `evidence.md`) and `mutants/run.sh`'s check batch with them; each passed its gate (scratch `gates/`, 07:48 to 08:05) |
| b. a test that kills the race's `||` for `&&`; why the reruns' `--re` took twelve mutants | done | nltxoozn: `the_caller_s_stop_ends_both_sides_of_the_race` (Mix of five pairs, the caller's stop always true: unknown, where the mutant lets the pool prove it), 30 of 30 runs pass unmutated, the mutant fails it in 0.04 s. The twelve: cargo-mutants 27.1.0 applies neither `--re` nor `exclude_re` to its struct-field deletions (`cargo mutants --list --file core/src/search/mod.rs --re zzzz` lists the eleven `delete field … in Options::with_*`, and the Horn file its `Answer` field), recorded in `evidence.md` |
| c. the command's shared search flags, `Options`, stop and verdict line (F194, F200), `--settings FILE` (R183), `--stats` and the engines from the library (area 3.2's halves) | done | lnnpmpkv (`cli/src/settings.rs`: one `linlog::Settings` per command from the command's defaults, `--settings FILE` read over them key by key, then the flags given on the command line by clap's `value_source`; `SearchArgs` flattened into `prove` and `interact`; new flags `--work-limit`, `--test-period`, `--schedule`; `seq.rs`; `Count<T>` and `Bytes` for the flags' numbers, F197; a deadline past `Instant`'s range no limit, F167; README's `--settings` example run by `readme.rs`, R183's "Met when"), ruyrrlnz (`--deterministic` takes turns too, as the design names; no lock change), zkzltzxt (`Status::from(&Verdict)`, `Stop::why`, `unread`, `Stop::BatchTimeout`, F198, and `--work-limit`'s advice), oovrotos (a session's `close` prints `verdict_line`: "provable" for "proved", the lock's `interact-session` with it), xmqwzzxr (`--stats` from `Engine::counters`, the lock's `format-stats` and README's examples with it), uysvqrto (`--engine` from `Engine::NAMES` in the command and the harness, F141's half) |
| d. the command's other findings (F5, F16, F27, H25, F65, F162 to F170, F189 to F208, H14, H15, H20) | done | svwxxqkz F16 (text and LLTP read within `--occurrence-limit`; lock `error-occurrence-limit`), lsywsspw (F161, F162, F163, F189, F190, F191, F193, F195, F196 with a test of the style help, F199 with a quote-aware `words`, F201, H20's note; `interact --file -` refused; lock `error-no-command`), xmyntvsq F203, porxkxso (F164, H14, H15), toxvsokk (F27 for proof files and sessions through `wire::upgrade`, H25: `check` under `--timeout`, `--work-limit`, `--occurrence-limit`, unknown exit 3 at a limit; F170's JSON; F208), xloykqkq (F166 `io::Stdout`, F202), pyvvnoyv F207, tzzkmuol (F205 `shown` and `Written`, F169 `Bounded`), pomuuppp F65, okyulpzs F5's command half (seq print's writers under the settings' limits), nyvzvtvp F206, kqmlktwk (records and check's JSON verdict first, the library's error as it writes it). F204 in lnnpmpkv, F200 in zkzltzxt and oovrotos, F168 fixed before the area |
| e. the command's tests (F192, F209 to F216) | done | yltzorpx (F209 `cli/tests/common/mod.rs`, F212, F213, F216: the `--jobs` note without a plural, left out where no test pins it; the CLI tests and README pass on one core, run under `taskset -c 2`), tnvqonyw (F192 `search_flags`, F210 `interrupt`, F211 `output_is_whole_or_untouched`, F214 `batch_flags`), qqwyskts (F215: the help of four commands, IALL, IMELL, ILL, intuitionistic affine, the batch's JSON and an error record, check as LaTeX, net and JSON valid, invalid and unknown, seq print as SVG; `error-memory-limit` renamed `unknown-memory-limit`). Not done: `--isolate`'s failure paths (a child that crashes or outlives its limit), which need a child that crashes on purpose |
| f. the harness (F161, F182 to F188, F219, F220, the reason column, F141's half) | done | otvurtuv: `summary::configuration` and `base`, `Ending`, `Runs::{decided, late, solved, failed}`, `COUNTERS` and a counters table, `summary --against … --counters` (the target set's check: `after-efficiency.csv` against `head-lto.csv`, 0 of 135 rows differ), `summary --strict` and the bench check over every source with it (built: `nix build .#checks.x86_64-linux.bench` passed), the tail by column name, `Reason::name`, no panicking duration, `baseline.sh --reference=` and `--reruns=` with columns by name (the same 489 and 1 040 problems as by position), unit tests of the summary, the modes and the ratchet's decision (6 tests); `COMPARISON.md` regenerated: the tables equal, the counter lines and the legend's two new endings added. Not done: a test of `compare.rs`'s printed tables, which print to standard output |
| g. the flake, CI and the hooks (F155, F172, F173, F180, F221, F222, F226, the step's flake items) | done | qvqulowr (`doc-private`: `cargo doc --document-private-items` with warnings denied, eleven broken or ambiguous links fixed), pkulyoqk (F155, F172, F173, F221: `command_line.py`, both guards on it, `format.sh` from the checkout's root, the claude-hooks check runs each guard by its path; built), omxtnrly (F180, F222: the crates inherit version and edition, crane's versions from the root; valgrind and the ratchet only where valgrind runs, `modules/ratchet.nix`; every declared system evaluates), ursvvqns (F226: the wasm target and the `wasm` check, every feature but `parallel`; built), wyryqwsm (the command's default LaTeX and Typst fragments compiled in `export`; curryst's version read from `CURRYST`, `NANOYALLA` asserted against the kernel's README; both checks built). Not done: CI builds x86_64-linux alone; the other systems are recorded as evaluated, not built |
| h. the documents (F156 to F160, F171, F174 to F179, F181, F190, F223 to F225, section 1.1's stale claims) | done | uzrwomtu (F160, F181: the oracles named in CLAUDE.md, `bench.md` and `TARGETS.md`), zwzymlvl (F171, F225: the spec's errata, the spec named in CLAUDE.md; F174, F175, F176, F177, F178, F179; F223, F224 in the register), mslrmklq F158, the bench rules in otvurtuv (F156, F157, F174's first items); xzotzuxo README's what exists. Section 1.1 of step 17's report read again: its README claims now hold (a proof is checked, the exit statuses), its CLAUDE.md, rules and help items are fixed by the commits above |
| i. the fuzz targets of the command and the read-back (F217, F218) | partly | lswsnsqs: `session_steps`, `proof_views`, `ordinary_decide` (seeded by `fuzz/seed.sh`; snzmzmvx its printf); F217 recorded out of scope in `evidence.md` (the command's readers hand the sequent to the library's, and its modules are crate-private). Not done: F218's scaling tests per reader |
| j. the ultra reviews' findings (`U…`) | done | U1 (the command fixes' slice; areas 3.2 and 3.3 gave none): an `--isolate` entry named or filed with a leading `-` got an error; vxwmzrmx lets the hidden `--entry-name` and `--entry-file` take hyphen values, `isolated_entries_named_like_flags` pins it (it fails without the fix) |
| k. stage 0's evidence again: every mutation batch, every fuzz target, the ratchet | done | **Mutation** (`mutants/area-3.4/`, the full run at rrrkklkp from `../linlog-mutants34`, the parser batches again at the head): survivors per batch against the last label that ran it: check 15 (18), horn-cover 16 (17), horn-equation 52 (54), horn-mod 3 (13), horn-reach 17 (18), lltp 1 (3), mist 15 (18), ordinary-derivation 12 (18), ordinary-parse 0 (4), search 8 (12), serialize 6 (6), parse 4 (3). Parse's four: `past_bound`'s guard and comparison, alive in `push` before the move, and two in `prefix`'s comparison of the terms owed, which is equivalent (the operand's own term reaches the bound anyway; the check refuses earlier, which is memory); the bounds' other survivors were killed by sorxxkoy, moxltnqm and slpvqpvv, tests at exactly the bound. The counts include 24 timed-out mutants; 8 of their logs show a test that fails, and every one is rerun alone (`--jobs 1`, the whole suite, scratch `timeouts.result`): all 24 hang alone too (unit `step28-timeouts`, 2026-10-14, 05:31 to 07:29, cores 12 to 15: `mist` 7, `horn-reach` 7, `horn-cover` 4, `horn-equation` 3, `horn-mod` 2, `search` 1; a counter multiplied from zero, a `+` turned into `*` in a loop's step, a comparison turned so that a loop never ends), so none is caught or missed and the counts above stand: without the 24, the survivors are mist 8, horn-cover 12, horn-equation 49, horn-mod 1, horn-reach 10 and search 7. A mutant that hangs the whole suite is one no test passes; cargo-mutants lists it apart from the missed ones. **Fuzzing** (`target/fuzz/summary.tsv`): every target ran to its cap or its stall; no crash and no new artifact but `ordinary_decide`'s 53 timeouts and 16 slow units, which found the focused engine's unpolled `&` rules (fixed in ytpvkvvz; that `Limits::work` does not count them goes to the author, below); the readers that changed after the run (tptp, sequent_text, ordinary_text, lltp) ran again at the head with no artifact. **Ratchet**: the gate's check on every commit; the flake's build put `read-tptp` 2.17 % over its ceiling, raised in ypzmzomx with the reason. **Families**: `linlog-bench run --all-families --timeout 5`, 123 runs, no mismatch, `summary --strict` passes. **Target set** at ytpvkvvz: 135 rows decided alike with the oracle `after-efficiency.csv`, 0 differ in their counters, none decided by the oracle and not by it (its times under load mean nothing; kept out of the tree as `scratch-after-and-poll`). |
| l. check rounds (at most three), the fresh-context reviewer, `nix flake check` | done: `nix flake check` passed at ypzmzomx and at nvxqnwxr, the area's head; every code commit of the area passed the gate (scratch `gates/summary`: uzrwomtu, okyulpzs and lyqqkvsm on a second run, their test's race being fixed in tyutxxqo; uknyooml and the three after it once the style-help fix was folded into it) | **Round 1** (nine lenses, one effort level below the audit's; the performance lens alone found nothing open): its fixes are rwtxwmxv, wxpuwxvv, vxwmzrmx, oxvxmwko, zututqnx to trorrowx, kknupyzk (the declines). **Round 2** (the same nine, at trorrowx; the limit stopped all nine at their start and they resumed after it): performance, coherence and readiness found their findings fixed; the API lens found that round 1's refusal of `batch.mode` and `ordinary.logic` refused the library's own `Settings` JSON (must-fix, undone in slznllpo: `batch.mode` applies unless a mode flag is given, `ordinary` under `--logic`); soundness a `--logic` derivation stopped in the read-back's checker pass, and a length cut inside a character read as no UTF-8; input runs of `(` that waited unbounded (ooxonrqk), a session's command lines without a cap, and `seq json`'s advice naming a flag it lacks; tests, docs and idiom the open halves of F188, F189, F195, F206, F210, F214, F217, F219. All fixed in tyutxxqo to slznllpo; F210's "exactly one line" is the reader thread's race, so the test bounds the rest to one line. The gate's three failures at okyulpzs, uzrwomtu and lyqqkvsm were one test racing `--batch-timeout 0ms` (tyutxxqo passes a past limit at once). **Round 3** (eight lenses, at rtsluwlo; performance had nothing open): coherence, docs, readiness and soundness found every item fixed and nothing new; idiom asked for one test helper; tests left F188's `compare.rs` (declined: it prints tables) and F214's timeout arm (a child that outlives its limit on purpose) and took a settings test of the batch's flags; API asked prove's `--settings` help and README to name `batch.mode`; input found a batch's lines and `--files-from` entries read without a cap (must-fix, against cli.md's new sentence), `seq json`'s advice naming a flag it lacked, and the depth refusal undocumented on `Limits::occurrences`. All fixed in oknowqpm; being the third round, its fixes had no round of their own, and the fresh-context reviewer reads them with the rest. **The fresh-context reviewer** (Opus 5.5, `high`, the whole diff pqrmsykr..oknowqpm with witness runs): one must-fix, `--batch-timeout` not stopping an `--isolate` child (a child under `--timeout none` ran on), and eight should-fix: Ctrl-C under `--isolate` read as an error, `--file -` and `--files-from -` read ahead under `--cores auto`, an `--output` directory refused only after the search, `summary --against` naming gains but not losses, a list's unreadable entries losing their error to the file check, `check` reading its file outside its time limit, a test that never reached the derivation it claimed to stop, and `env -S` hiding git from the guard. All fixed in trvlvsyk, pwptootq (COMPARISON.md regenerated: the reverse sections add the problems the default pass decides and a single bias does not) and pokmmrxy, each with a test. **The last reader** (Fable 5.1, `high`, README and the rustdoc's front page against the usability baseline): of the baseline's fifteen points, five resolved, seven partly, three open; its two must-fix obstacles were how to get the library (README and the front page now say) and the advice of an unknown that no limit decides (to `plan/later.md`: it needs measurements); fixed besides in koqypwoq: the syntax and the counters introduced where they are first used, an occurrence defined, `show latex` refused as a file name, `--prelude` without a standalone certificate refused. The rest, with reasons, is the new "Follow-ups: a first user's obstacles" of `plan/later.md`. |

#### Area 3.4: decided unattended

Each choice below is the session's; where it departs from the
signed-off design or from a finding's proposed fix it says so.

- **`--settings FILE` is read over the command's defaults, then the
  flags given** (R183, the design's 6.6): the file's JSON is written
  over the serialized defaults key by key and read back as `Settings`
  (a misspelt key refused by serde), and a flag applies only when the
  command line gives it, by clap's `value_source`; without a file every
  flag applies, its default being the command's. Set aside: every flag
  an `Option` without a default (the help would lose its defaults), and
  a generic `--set KEY=VALUE` (no front end asks for it).
- **The command's defaults differ from `Settings::default()` in the
  batch alone**: its workers are the machine's threads and its memory
  half the process's (`settings::defaults`), which the library cannot
  know; the rest is the library's value.
- **`--logic` stays a flag of the input, not a key**: the settings'
  `ordinary.translation` applies when `--logic` names a logic and no
  `--translation` is given; a file's `ordinary.logic` is accepted and
  `--logic`'s value replaces it (every front end's JSON holds the key, so
  it cannot be the switch); a file's
  `batch.mode` is the batch's mode unless a mode flag is given (check
  round 1 found both read and then ignored; refusing them, round 1's
  first fix, refused the library's own `Settings` JSON, which always
  holds both, and round 2 undid it).
- **New flags for keys without one**: `--work-limit`, `--test-period`,
  `--schedule` on `prove` and `interact` (the design's table), and
  `--timeout`, `--work-limit` and `--occurrence-limit` on `check`. Not
  added: `--derivation-limit` on `seq print`, whose bound a settings file
  sets (the refusal says so).
- **`--deterministic` takes turns** (`search.schedule = turns`, as the
  design names): every lock file and README example with counters stayed
  the same.
- **`check` answers unknown, exit 3, where a limit gives the check up**
  (H25, F170): it was an error, exit 2; a refusal is no verdict, and the
  command's unknown is 3 everywhere else. Its default time limit is the
  command's two seconds, counted from its start.
- **The session's verdict line is `prove`'s** (F200): "proved" became
  "provable" in a session, a lock change of its own (oovrotos).
- **`--stats` prints one line per counter `Engine::counters` lists**,
  with the library's labels, leaving out the copy bound without
  exponentials and the forward search's where it did not run: the memo
  hits get a line of their own (a lock change, xmqwzzxr).
- **`--engine` takes `Engine::NAMES`** through a possible-values parser,
  so a new engine reaches the command and the harness without code; the
  per-engine help moved into `--engine`'s long help.
- **A JSON Lines record of an error carries the library's error where
  the chain holds one, and a `message` always** (F164): the record is
  written by hand, verdict first, since `serde_json::Value` sorts keys
  and the workspace does not enable `preserve_order`. `check`'s JSON
  likewise: `{"version", "verdict", "mode", "error", "message"}`.
- **Two entries of one file are refused at the second** (H15), before
  it is searched, not before the batch: the entries stream, so the names
  are not known in advance.
- **A closed standard output is taken as a reader that has read
  enough** (F166): `io::Stdout` drops what the pipe refuses and the
  verdict's status stands; set aside: restoring `SIGPIPE`, which would
  end the command with 141 and lose the verdict.
- **An `--output` is probed before the search** (F207) by opening and
  dropping it (a partial file made and removed), rather than opened once
  for both: `Output` holds standard output's lock, which cannot cross to
  the search thread.
- **A PNG or PDF's SVG is bounded by `--memory-limit`** (F169), not by a
  constant the library exports: the text past the bound is refused as it
  is written (`Bounded`), and the drawing left out with a line.
- **`interact` splits its words as a shell does with a splitter of its
  own** (F199, the step's item): about thirty lines; set aside: the
  `shlex` crate, a dependency for that.
- **The command's library is crate-private** (F203): `main` is its one
  public item; the flake's `doc-private` check documents the rest.
- **One command-line reading for both guards** (F155, F172):
  `command_line.py`, a module the guards import (not executable, so the
  hooks check does not ask for cases of it).
- **The ratchet check is a module of its own**, `modules/ratchet.nix`,
  defined only where valgrind runs, since `lib.mkIf` does not drop a key
  of the lazy `checks` set (F222). CI keeps building x86_64-linux alone;
  the other systems are recorded as evaluated, not built (departs from
  the finding's "add the systems to CI": that is a CI change by the
  author's leave only).
- **The comparisons group by what they compare**: `--before` by file,
  family and whole configuration, `--against` by the base configuration
  alone, since its files differ in an option (a first attempt grouped
  `--against` by the whole configuration and paired nothing of the bias
  passes); the counters' rows print with `--counters` only, so
  `COMPARISON.md` keeps its tables and gains one line per comparison.
- **F217's command readers are not fuzzed** (the finding's other
  option, recorded in `evidence.md`): they hand the sequent to the
  library's readers, which are, and a target over `linlog_cli` would
  need its modules public. F218's three targets are added; its scaling
  tests per reader are not (the readers are linear by construction, and
  the ratchet's journeys count them).
- **`--isolate`'s failure paths have no test** (F214's last part): a
  child that crashes or outlives its limit needs a child that does so on
  purpose, which the command has no way to ask for.
- **`--jobs` sets two keys** (departs from the design, whose flags each
  set one key): `search.jobs`, and `clock.pool_after_ms = null` unless
  `--pool-after` is given, since `--jobs N` has always meant N threads
  from the start; `--deterministic` sets three (`jobs` 1, `schedule`
  turns, no pool).
- **An error record is `"verdict": "error"`** (departs from api.md §7.3
  and §7.4, where a record holds an outcome or an error and the verdict
  is closed): a reader of the stream then branches on one key for every
  line, and a record of an entry the batch stopped unread is `unknown`
  with a reason in the same way.
- **`seq json` and `seq fragment` take no `--settings`**: they read one
  bound, `--occurrence-limit`, and print what they read; a file would
  set nothing else they use.
- **Every input text is read within the memory limit** (check round 1,
  the input lens): a file or standard input longer than it is refused
  naming `--memory-limit` before it is held (`/dev/zero` took 3 GB
  before); settings and style files within 16 MiB, never from standard
  input. The finding proposed a cap from the occurrence limit with a
  factor for blanks; the memory limit is the bound already stated.
- **A record's `seconds` and the outcome's `version` stay text surgery**
  (`record_json`): the outcome's JSON is the library's and is written
  once; a parse into `Value` and back would sort its keys. Recorded as
  taste.
- **F169's residue**: a session's `show --png` and `seq print`'s PNG hold
  the SVG within the derivation limit, not the memory limit; a render
  may then take up to twice `--memory-limit` beside the held text.
  Recorded, not fixed: both are bounded, and the session's drawing is
  of a partial derivation the user builds by hand.
- **F182's rerun list is the first baseline's on purpose** (`bench.md`:
  a later baseline reruns the same list so the two compare), and
  **F183's resume key** stays `run`'s own (`finished`): it must match
  the rows a run appends, a file's own columns, which `configuration`
  reads too; `summary.rs`'s module doc now names both.
- **`session_steps` builds its session from text, not from a file read
  back** (F218's second part, declined): reading a session
  (`Interactive::from_parts`, through `wire::upgrade`) replays every
  closed inference through the same steps `apply` takes, so a session
  read back is one those steps built; `json_session` fuzzes the read
  itself.
- **`--settings -` and `--style-file -` are refused**: standard input
  carries the sequent or a session's commands (the input lens found the
  settings read in their place).

#### Follow-up: F122, F123 and the engines on their sequents, item by item (session `step-28i`)

Open question 10's answer: the engines on the two sequents F122 and
F123 found first, then both generators; the third ultra review's
findings; open questions 3 and 5 (the author's answers at the step's
end); open question 11, the count of one sign, which the author asked
this session to build.

| item | state | evidence |
|---|---|---|
| 1. the engines on both sequents | done | Both are unprovable. F122's sample is the mutant of a generated proof (pxsxlnrl corrects question 10): its goal needs an `a` that nothing supplies. F123's Mix mutant (`\|- ?((((!~a par c) + c) par a) par ~c) par ?((~c * ~c) * c)`) has every `~a` inside a `!` box. Its search was slow because a failure that rests on a prune of the loop check was never memoized: at copy bound 3, 5.85 million decisions of 603 sequents. qoksszzp lets such a failure wait on the shallowest ancestor it rests on (`focus/waiting.rs`, whose module doc has the argument): copy bound 3 takes 17 179 stable sequents instead of 76.5 million (21 s), copy bound 5 102 ms, and the default reaches copy bound 8 in 10 s. F122's mutant classically at copy bound 3: 9.28 million stable sequents in 1.82 s instead of 73.8 million in 8.56 s. ytvrknnp: `relieve` sets the branch's marks to the start (the argument lens's finding; `a_relieved_search_settles_what_waits` fails without it). Evidence: target sets `after-waiting` and `net-after-waiting` (pnmsymur): no verdict differs from the oracles, the 135 rows decided alike have equal counters, 11 rows at their copy bound visit fewer stable sequents, the net set's 64 rows equal; an old-against-new differential on 13 568 generated items at copy bounds 2, 3 and 4: no contradiction and the same decided rows; mix/10 timed alone on core 2, old 80.5 and 79.9 s, new 82.0 and 79.1 s; the journeys within 0.6 %; the families and qcover pass `summary --strict` |
| 2. the generators | done | wzynqlqo (F122: the intuitionistic generator contracts a used `!` hypothesis, `intuitionistic_proofs_contract` pins at least 8 of 40 samples, measured 10 to 17; `MUTANT_WORK` bounds the mutants' searches in the tests that compare verdicts, five million units, with `bounded`), ypvxqstw (F123: half the mutants swap the atoms of two literals of one sign, `mutate_two_sided` for the intuitionistic callers; `some_mutants_reach_a_search` pins at least 1 in 12, measured 11 to 20 % classically and 25 to 30 % intuitionistically). The debug suite stays at about two minutes (`engines_agree_on_horn_programs` dominates); no engine disagreed with another or with the reference prover |
| 3. the third ultra review's findings | done | U2 fixed before (vxwmzrmx, pokmmrxy); U3 and U4 vzxtsvrx; U5 oxlvywnp; U6 mlkkmqmu (the hooks' check passed); U7 xwoononu |
| 4. open questions 3 and 5 | done | yqsorumv: an `&` rule is work (`poll_with`: a step and a zone's copy), so `Limits::work` bounds a proof of `&` rules closed by `⊤`; vvwkkuzx: such a proof keeps to the memory bound and the stop (the `&` rule asks the account and relieves; `Arena::keep_all` and `Proof::build` in place, polling every 2¹⁶ nodes). 24 pairs under `--timeout 1s` end after 1.05 s (1.73 s before) at a peak of 662 MB (1.22 GB); 26 pairs end at the memory limit after 0.99 s at 531 MB (6.17 s and 4.86 GB); `--work-limit 1000000` ends 24 pairs in 56 ms. vnkywvnv raises render-svg's ceiling by 3.1 % (the heap a proof's build leaves; the writer's own counts are equal). Target sets `after-and-memory` (kqtkltmp): no verdict differs, counters equal |
| 5. open question 11: the count of one sign | done | kvpyrzuv: an atom whose literals below the exponentials all have one sign keeps its rows, and a balance is ruled out only on the side the copies cannot reach; `Refutation::OneSided`, `one_sided` in JSON (a kind of its own, since a tag keeps its meaning), the library's lock in ymmluvqt and the command's (text and JSON) in lyowmwos, README's example in qxvmrxzn, the fuzz seed in step. F122's mutant is refuted at once, classically and intuitionistically. Tests: `copies_of_one_sign_bound_the_count_one_way`; `the_count_of_one_sign_refutes_nothing_provable` (a test-only switch turns the rows off): on 1 227 generated mutants with exponentials, 1 352 refutations, 30 by the count at the root and 64 that the search without it leaves undecided, none proved without it a copy higher. The ratchet: the sign read per row entry cost qbf/20#2 3.6 % and batch-families 10.2 % instructions; with the check chosen once per row they are 0.65 % and 12.6 % below their ceilings. LLTP: of the 4 512 problems, 1 663 have such an atom and no `⊤` below an exponential (1 430 Petri nets); head and the count on them, each problem on one core for both, 5 s: at copy bound 3 one gained (SYN389+1 in 01, refuted), none lost, no contradiction, none against the status but the six known headers (head's too); without a copy bound on the 116 rows at the bound, two gained (SYJ205+1.009 and .010 in cbn). Timed alone on core 2 (median of three): SYN389+1 in 01 from the copy bound in 0.113 ms (25 stable sequents) to refuted in 0.087 ms (11); SYJ205+1.009 in cbn proved in 9.56 s (33.8 million) and now in 0.78 s (2.70 million); SYJ205+1.010 in cbn not within 30 s (106.8 million), now proved in 2.22 s (7.11 million). Target sets `after-one-sided` and `net-after-one-sided` (snrttxvw, at srtmwmwo): no verdict differs from the oracles; of the 135 rows decided alike only the chain family's four differ, one memo entry fewer and one hit more at the same stable sequents and splits (the count's own binary shows it); the net set's 64 rows equal. After qlnmtlzq the focused set again (`after-rows`, voukzpot): every decided row's counters equal `after-one-sided`'s. The families pass `summary --strict`; qcover (affine, where the count does not apply) passes it and no verdict differs from `bench/defaults/horn-qcover.csv` |
| 6. the panel and the fresh-context reviewer | done | **Argument lens** (Fable 5.1, `high`, on qoksszzp): `relieve`'s marks (ytvrknnp) and the doc's gaps, fixed. **Counterexamples lens** (Opus 5.5, `high`, on qoksszzp with ytvrknnp; the first run stopped at the usage limit): no wrong verdict in about 60 000 sequents (7 863 and 6 472 provable by construction, 6 000 shaped like F123's, 1 095 of its mutations, 21 390 enumerated Mix sequents, 30 000 random) over up to 17 configurations each, memory bounds, memo limits and pools included; both reference provers proved none of the new engine's refutations; an instrumented build's check of every reuse found no violation without a memory bound. **Integers lens** (Sonnet 5.5, `high`): no number wraps or passes a bound but the memory one: the waiting list was charged its keys alone. **The count's counterexamples lens** (Opus 5.5, `high`, on kvpyrzuv): 8 209 sequents built for the count (one sign below `!`/`?`, with `&`, `⊕`, `⊤`, `0` beside, nested exponentials, Horn programs, 209 by hand), classical, Mix and intuitionistic, under five configurations each, against the reference prover and head: no disagreement; 2 740 reach the count (994 with a `one_sided` refutation, 424 decided where head is not); its one finding, rows built where the check is off, is qlnmtlzq. **Fresh-context reviewer** (Opus 5.5, `high`, pxsxlnrl..vvwkkuzx): no soundness break; its four findings are zmqntlyl (the waiting lists charged by capacity: a Mix search under 4 MiB peaked 10 MiB above its start, now at it), ovolwupp (`holding`'s doc names the `&` rule), srtmwmwo (the settings test's missing flags) and yurnpots (`a_close_settles_by_every_rule`: six mutants of the close's and the reuse's rules, each swept over 58 000 generated cases, none wrong, each changing hundreds of counters; every one fails the test). On the count, besides its counterexamples lens: **the argument lens** (Fable 5.1, `high`): no gap that a witness shows; the invariant checked rule by rule over the proof terms and the engine's initial rule; its findings, the doc's argument left the invariant and the `?` rule, weakening and the units unstated, and the check's condition was written twice, are uyrrnvwy (with a sentence on what a goal's refutation counts), and it notes that a literal below a `!` under no `?` is used once, so the test of both signs could look at `?` alone (a follow-up, `plan/later.md`); **the integers lens** (Sonnet 5.5, `high`; the limit stopped it once): no sum, rank or charge wraps or underflows (each examined, with the reasons in its report), a `one_sided` form round-trips at `i32::MIN` and `i32::MAX`; its finding, the tallies' and splits' `touched` lists uncharged and growing with the copies of a formula now that copies have rows (142 MiB under a bound of 16 MiB on its witness), is pqnxxtsx (15.5 MiB, the same verdict), with the index's buckets counted exactly; **a second fresh-context reviewer** (Opus 5.5, `high`, kvpyrzuv..qlnmtlzq; the limit stopped it once): no soundness bug and nothing must-fix; its 13 000 random sequents biased to the count, about 1 230 `one_sided` refutations, none proved by head at copy bound 6; should-fix the command's lock without the new kind (lyowmwos) and `row_atoms`' doc (spmrupso, with its notes: the text of a `one_sided` form from JSON printed from its field, `Waiting::grown` counting the index's buckets); its note that zmqntlyl had no test is vtypozyz's `the_lists_are_charged_as_they_grow` |
| 7. mutants and `nix flake check` | done | **The batches** (`mutants/step-28i/`, from the workspace `../linlog-step28i-mutants` at yurnpots, 15:14 to 16:42): survivors against area 3.4's lists check 4 (15), search 3 (8), horn-mod 0 (3), serialize 2 (6; area 3.1's guard in `DisproofForm::within`, its line moved by the new kind). New among them: `Proof::build`'s poll every 2¹⁶ nodes (`%` for `/`), pinned in vtypozyz (the polls after the search's last work bounded from above), and two in search that hang: `few_equal_literals` always true and `DEFAULT_MEMO_LIMIT` shifted to zero, which hang the whole suite alone too (one job, the whole suite, 254 s each; unit `step28-timeouts`), so neither is caught or missed. **The focused engine's changed lines** (`--in-diff` of pxsxlnrl..qlnmtlzq over `focus/{mod,counts,waiting,arena,memo,parallel}.rs` and `refutation.rs`, from `../linlog-step28i-mutants2`): 224 mutants, 38 unviable, 23 survived both passes: the memory charge's arithmetic (`Waiting::grown`, `release`, `relieve`'s release, `Counts::new_until`'s fixed charge), `count_hit`, the `&` rule's work cadence, and three in `Waiting` (`find`'s `&&`, `pop`'s test, `clear`) that only drop reuses or are equivalent (an index chain holds one hash; the list is empty at a level's end). vtypozyz pins the charge, the hits and the cadence; rerun on those functions at uyrrnvwy, 68 of 80 caught, the rest: three of `grown`'s terms that its test's lower bound does not see, `count_hit`'s pool arm (the pinned searches run on one thread), the node count of a refusal at 2³² nodes, the `&` rule's cadence (two), and the three in `Waiting`. The `touched` fix's functions (`Tally::apply_as`, `Split::touch`) and `grown` at pqnxxtsx: 48 of 53 caught, the five left `grown`'s byte terms of the index and of the held entries. **`nix flake check`** passed at pqnxxtsx, the session's last code commit (2026-10-14, 21:07 to 21:12, every check). |

#### Follow-up: decided unattended

- **A failure that rests on the loop check is truncated, not pruned**
  (`Waiting::drop_from` at a proved sequent drops every failure decided
  below it): pruning only those that rested on the proved sequent would
  keep more, with the same counts on every measured sequent.
- **A leader moves to the least depth of the failures below it**
  (`lead_from`), the exhaust rule makes every failure below a cut
  sequent cut at its own budget, and a worker of a pool drops the
  failures that rest on the spawning engine's branch.
- **The list of waiting failures is capped at the memo's entries**, and
  a reuse counts as a memo hit.
- **The count of one sign was first sent as a decision** (recommended
  for a later session); the author asked for it in this one.
- **render-svg's ceiling was raised** (the allocator's heap state after
  the proof is built in place), not the copying `Proof::new` restored.
- **No memory check after the root's keep**: it would lose proofs where
  the memo filled the bound.
- **An `&` rule's work is `1 + forest.len() / OCCURRENCES_PER_WORK`**,
  passed at its own poll; forced chains still pass none.
- **U3 keeps `--logic` as the switch**, documented; U4 is a table test;
  U5's label lives in `Counter`.
- **`OneSided` is a new refutation kind**, not a key on `unbalanced`
  (a tag keeps its meaning on the wire).
- **The count's hot loops are monomorphized on whether any atom has
  copies of one sign** (`excluded_as`), since reading the sign per row
  entry cost up to 10 % of a journey's instructions.
- **The waiting lists are charged by their capacity after each close**
  (`Waiting::grown`), rather than a fixed size per entry.
- **The rows of one sign are built only where the interval check runs**
  (qlnmtlzq, the count's lens), rather than everywhere with the check
  reading them only there: elsewhere they reorder the split search.
- **A `one_sided` form read from JSON prints what its fields say**, not
  validated against its counts: no reader checks a refutation's truth,
  the `unbalanced` kind's included.
- **The waiting lists' tests pin counters** (`a_close_settles_by_every_rule`):
  no rule dropped gave a wrong verdict on 58 000 cases, so a verdict
  alone could not pin them.

## The step's outcome

Step 28 is done. The library's public surface was redesigned once, in
`plan/notes/api.md` (signed off by the author on 2026-10-09), and built
in four areas of fixes: the API, data model and wire forms (3.1), the
search (3.2), efficiency (3.3), and the command, the harness, the flake
and the documents (3.4). Each area passed its check rounds, a
fresh-context reviewer and `nix flake check`, and the supervisor's own
review. Every commit that touches code passed the gate (clippy, the
tests with the behaviour lock among them, both `cargo hack` runs, the
ratchet). The target sets kept their counters on every decided row
from stage 0 to the head (`bench/targets/after-efficiency.csv` against
`head-lto.csv`, `net-after-efficiency.csv` against
`net-after-retype.csv`). No ceiling of the ratchet is above its stage 0
value but `read-tptp` (+7.5 %, raised twice with the reason each time)
and the two check journeys (+1.1 % and +0.3 %, the cost of checks
added). Of the register's 254 later requirements, the table in "The
register after step 28" below gives each one's state, with the
evidence in `plan/notes/requirements.md`.

What is open is listed under "Open questions for the author": the
departures from the design, the decisions the areas left, the faults
area 3.4's review found and did not fix, and the third ultra review's
findings, which a short session after this one takes.

### The surface before and after

At stage 0 (zrvpwtoq, the ratchet's commit) the core crate had 76
source files and 42 476 lines; at the head it has 96 files and 55 143
lines (the core, command and harness crates together: 50 458 lines,
then 65 564). The growth is the new modules, the bounds and stops
threaded through every long call, the wire forms with their versions,
and the tests.

- **Before.** The root re-exported `Error` alone of the errors; the
  occurrence set `OccSet`, `Polarity`, `Position`, `Scratch`, and from
  `proofs` `Compact`, `DEFAULT_MEMORY_LIMIT`, `Labels`, `OpenGoal`,
  `Side` (the `⊕` rule's choice), `TextOptions`, `ViewError`,
  `WriteError`; the session's `Refusal`; and `prove_until`. Errors were
  35 positional variants beside `ShapeError`, `Unsupported`,
  `UnknownRule`, `NetError`, `ViewError` and the checker's `Problem`,
  whose `Memory` was a refusal beside the faults. The bounds of a
  search, a check and a view were three separate values; the stops took
  no progress; `search::Options` had setters and no wire form; the
  readers of proof files and sessions ignored the caller's limits.
- **After.** The root re-exports one error family (`Error`,
  `ErrorKind`, `Described`, `Owner`, and `ParseError`), the bounds and
  stops (`Limits`, `Progress`, `Phase`, `Refusal`), the settings
  (`Settings`, `Clock`), `Member` and `Side` (the side of `⊢`),
  `Branch` (the `⊕` choice), `Criterion` and `VertexId` of the nets,
  `CheckError`, `Named`, the session's `GoalId`, `Step`, `StepError`,
  and from `search` `Counter`, `Disproof`, `Goal`, `NotTaken`,
  `Refutation` beside the search's types (`core/src/lib.rs:175-193`).
  New modules: `limits`, `settings`, `wire` (the forms' level `LEVEL`,
  the converter `upgrade`, the bounded reader). `proofs::check`,
  `proofs::derivation` and `proofs::interactive` are directories split
  along their seams. Every options value is plain data with serde and a
  default; every long call takes `&Limits` and a stop
  `FnMut(Progress) -> bool`; every JSON form carries `version` and is
  documented in `wire`; the engines answer through one interface
  (`search::Decide`, `Answer`, the dispatch table) and `search::race`
  is the one race of a thread and a pool. The command's library
  `linlog_cli` has `main` as its only public item.

Where the code differs from the design's module tree (`api.md` 2.1):
`Refutation` and `Disproof` live in `search` (`search/refutation.rs`),
not in a top-level `refutation` module, and `engine_for` and `race` are
reached as `search::engine_for` and `search::race`, not at the root.

### What was renamed or removed

One row per old name, so that a later session can search for it. The
table starts from `api.md` 2.4 and was checked against the code at the
head: "checked" names the new item's place, or "absent" where a grep
finds the old public name nowhere. "Removed" items have no alias (no
compatibility aliases before 0.1.0).

| before | after | commit | checked |
|---|---|---|---|
| `sequents::Term::Var` | `Term::Atom` | ksnmyzmo | sequents/term.rs:55 |
| `sequents::Term::DualVar` | `Term::DualAtom` | ksnmyzmo | sequents/term.rs:57 |
| `sequents::Kind::Var` | `Kind::Atom` | ksnmyzmo | sequents/term.rs:89 |
| `sequents::Kind::DualVar` | `Kind::DualAtom` | ksnmyzmo | sequents/term.rs:91 |
| `Atom` documented as "a propositional variable" | `Atom` an atomic formula, nullary | ksnmyzmo | unconfirmed: the doc comment at sequents/term.rs:28 still reads "An atom (a propositional variable)" |
| `Sequent::optimize` sorting the roots | `Sequent::optimize` keeps the written order | rtlqploq | sequents/mod.rs:344 |
| `Sequent::optimize` fallible (`Result`) | `Sequent::optimize` infallible | ypwwvlnq | sequents/mod.rs:344 |
| (new, no old name) | `Sequent::antecedents() -> Option<u32>` | rtlqploq | sequents/mod.rs:144 |
| `Sequent::verify_integrity` | crate-private `Sequent::check` | ksnmyzmo | sequents/mod.rs:225; `verify_integrity` absent |
| `FromStr for Sequent` only (no bounded read) | `+ Sequent::parse_within(text, &Limits)` | mwvwsxyk | parse/mod.rs:528 |
| `sequents::fmt::Walk` crate-private | design: public; code: still crate-private | none | sequents/fmt.rs:53 (`pub(crate) struct Walk`) |
| `sequents::fmt::Visit` crate-private | design: public; code: still crate-private | none | sequents/fmt.rs:19 (`pub(crate) enum Visit`) |
| `Visit::Between(T)` binary | design: `Visit::Between(TermId, index)`; code: unchanged `Between(T)` | none | sequents/fmt.rs:26 |
| `Sign::Var` | `Sign::Atom` | ksnmyzmo | occurrences/mod.rs:101 |
| `Sign::DualVar` | `Sign::Dual` | ksnmyzmo | occurrences/mod.rs:103 |
| `occurrences::Position` (input or output) | `occurrences::Side` | ksnmyzmo | occurrences/reading.rs:33; `Position` absent |
| `Forest::within(&s, u64)` | `Forest::within(&s, &Limits)` | mwvwsxyk | occurrences/mod.rs:277 |
| (new, no old name) | `Forest::new(&s)`, the defaults of `within` | mwvwsxyk | occurrences/mod.rs:266 |
| `Forest::from_owned(s, u64)` | `Forest::from_owned(s, &Limits)`, still public | mwvwsxyk | occurrences/mod.rs:289 |
| `TryFrom<Sequent> for Forest` | removed | ksnmyzmo | absent (the one `TryFrom` left is `serialize/sequents.rs:152`, for the wire form) |
| `occurrences::OccSet` (public) | crate-private | ksnmyzmo, wyuzsnql | occurrences/set.rs:25; not in lib.rs or `occurrences` re-exports (occurrences/mod.rs:12 `pub(crate) use`) |
| `Forest::empty_set` | crate-private | ksnmyzmo | occurrences/set.rs:348 |
| `Forest::root_set` | crate-private | ksnmyzmo | occurrences/set.rs:355 |
| `OccId` in `Node`'s operands | `Member` | nyxmnuxv | proofs/mod.rs:169 (`principal() -> Option<Member>`); `Member` at occurrences/mod.rs:52 |
| `OccId` in `Dyadic` | `Member` | nyxmnuxv | proofs/check/error.rs:19 (`theta`, `gamma`: `Vec<Member>`) |
| `OccId` in `Inference::sequent` | `Inference::sequent() -> &[Member]` | rnnrsvnv | proofs/derivation/view.rs:149 |
| `OccId` in `Interactive::goal` | `Member` | qmpstuqr | proofs/interactive/mod.rs:511 |
| `prove_goal(forest, &[OccId], mode, &o, stop)` | `prove_goal(Goal, mode, &o, &Limits, stop)` | mrvtkppt, mwvwsxyk | search/mod.rs:343; `Goal` at search/mod.rs:244 |
| `engine_for(forest, &[OccId], …)` | `engine_for(Goal, mode, &o)` | mrvtkppt | search/mod.rs:656 (not re-exported at the root, lib.rs:188) |
| `Node::occurrences()` | `Node::members()` | nyxmnuxv | proofs/mod.rs:190; `occurrences()` absent on `Node` |
| `Scratch` at the crate root | `nets::Scratch` | ksnmyzmo | nets/mod.rs:30; absent from lib.rs |
| `Polarity` at the crate root | `occurrences::Polarity` | ksnmyzmo | occurrences/mod.rs:125; absent from lib.rs |
| `Compact` at the crate root | `proofs::Compact` | ksnmyzmo | proofs/derivation/view.rs:89, re-exported proofs/mod.rs:35; absent from lib.rs |
| `Labels` at the crate root | `proofs::style::Labels` (also `proofs::Labels`) | ksnmyzmo | proofs/style.rs:35; absent from lib.rs |
| `OpenGoal` at the crate root | `proofs::style::OpenGoal` (also `proofs::OpenGoal`) | ksnmyzmo | proofs/style.rs:295; absent from lib.rs |
| `TextOptions` at the crate root | `proofs::TextOptions` | ksnmyzmo | proofs/fmt.rs:33, re-exported proofs/mod.rs:36; absent from lib.rs |
| `DEFAULT_MEMORY_LIMIT` (root constant, three copies) | `Limits::DEFAULT_MEMORY_BYTES` | mwvwsxyk | limits.rs:89; absent |
| `Mode.intuitionistic` (public field) | private; `Mode::is_intuitionistic()` | uqwrupzo | fragment.rs:309 |
| `Mode.affine` (public field) | private; `Mode::is_affine()` | uqwrupzo | fragment.rs:314 |
| `Mode.mix` (public field) | private; `Mode::has_mix()` | uqwrupzo | fragment.rs:319 |
| `Mode::affine()` | `Mode::with_affine()` | uqwrupzo | fragment.rs:325 |
| (new, no old name) | `Mode::with_mix()` | uqwrupzo | fragment.rs:334 |
| (new, no old name) | `Mode::name()`, `Mode::NAMES`, `FromStr for Mode`, `Mode::check()` | uqwrupzo | fragment.rs:341, 298, 373, 364 |
| `Fragment::ALL` (the additive class) | `Fragment::ADDITIVE` | uqwrupzo | fragment.rs:39; `Fragment::ALL` absent |
| `Fragment::NAMED: [Self; 6]` | `Fragment::NAMED: &'static [Self]`; `FromStr for Fragment` | uqwrupzo, vrnnzurm | fragment.rs:49, 155 |
| `proofs::Side` (the `⊕` rule's choice) | `proofs::Branch` | ksnmyzmo | proofs/mod.rs:92 |
| `Proof` without a conclusion or mode | `Proof::goal()` | nolqqvoz | proofs/mod.rs:454 |
| `Proof` without a conclusion or mode | `Proof::mode()` | nolqqvoz | proofs/mod.rs:461 |
| `Proof` without a conclusion or mode | `Proof::new_of_goal` | nolqqvoz | proofs/mod.rs:421 |
| `Proof` without a conclusion or mode | `Proof::with_mode` | nolqqvoz | proofs/mod.rs:445 |
| `Proof::check_within(mode, Option<u64>)` | `Proof::check_within(mode, &Limits, stop)` | mwvwsxyk | proofs/mod.rs:564 |
| `proofs::check::check` (public free function) | crate-private | ksnmyzmo | proofs/check/mod.rs:40 (`pub(crate) use pass::{…, check, check_within, …}`) |
| `proofs::check::check_within` (public free function) | crate-private | ksnmyzmo | proofs/check/mod.rs:40 |
| `CheckError { node, rule, premises, problem }` (struct) | `CheckError::{Invalid(Box<Invalid>), Refused(Refused)}` | yzywpzko | proofs/check/error.rs:76 (`Invalid` at 88, `Refused` at 103) |
| `CheckError::is_refusal()` | removed from `CheckError`; `Error::is_refusal` and `ErrorKind::is_refusal` | yzywpzko | errors/mod.rs:635 and 427; none on `CheckError` |
| `check::Problem` | `check::Fault` | yzywpzko | proofs/check/error.rs:113; `Problem` absent |
| `Problem::Memory` | removed; a bound is `CheckError::Refused` with a `Refusal` | yzywpzko | proofs/check/error.rs:102-113 (no `Memory` in `Fault`) |
| `Proof::derivation_with(&view, stop)` | `Proof::derivation_within(&view, &Limits, stop)` | mwvwsxyk | proofs/mod.rs:621 |
| `Proof::two_sided_derivation` | removed; the sides are `view.sides` (`Sides::Two`) | mnkunorl | proofs/derivation/view.rs:71; only a test-only `pub(crate) fn two_sided` at proofs/mod.rs:637 |
| `Proof::two_sided_derivation_with` | removed (renamed `…_within` in mwvwsxyk); the sides are `view.sides` | mwvwsxyk, mnkunorl | absent |
| `Proof::derivation_size(bool)` | `Proof::derivation_size(&view, &Limits, stop)` | mnkunorl | proofs/size.rs:102 |
| `Proof::derivation_size_within(bool, Option<u64>)` | removed; `derivation_size(&view, &Limits, stop)` | mnkunorl | proofs/size.rs:102; `derivation_size_within` absent |
| `ViewOptions { limit, memory, compact }` | `ViewOptions { compact, sides }`; the bounds in `Limits` | mwvwsxyk, mnkunorl | proofs/derivation/view.rs:58 |
| `ViewOptions::DEFAULT_LIMIT` | `Limits::DEFAULT_DERIVATION_BYTES` | mwvwsxyk | limits.rs:95 |
| `ViewOptions::UNBOUNDED` | removed (`Limits::UNBOUNDED` is the unbounded value) | mwvwsxyk | limits.rs:102; `ViewOptions::UNBOUNDED` absent |
| `ViewError` | `Error::Check(CheckError)` and `Error::Refused(Refusal)` | yzywpzko | errors/mod.rs:221 and 339; `ViewError` absent |
| `WriteError` | `Error::WriteFailed` | yzywpzko | errors/mod.rs:354; `WriteError` absent |
| `RenderError` | `Error::RenderFailed` | yzywpzko | errors/mod.rs:372; `RenderError` absent |
| `svg::TooLarge` | `Error::Refused(Refusal::Output { … })` | yzywpzko | errors/mod.rs:339; `TooLarge` absent |
| `Rule` (34 variants) | `Rule` (15 one-sided rules and `Open`, 16 variants) | xnszropr | proofs/rule.rs:13 |
| `Rule::ALL: [Rule; 34]` | `Rule::ALL: [Self; 16]` and `Named::ALL: [Self; 34]` (arrays, not `&'static [Rule]`) | xnszropr | proofs/rule.rs:51 and 156 |
| `Rule::classical()` | removed; `Named { rule, side }` | xnszropr | proofs/rule.rs:144; `classical` absent |
| `Rule::intuitionistic(position)` | `Rule::on(side) -> Named` | xnszropr | proofs/rule.rs:127; `intuitionistic` absent |
| `Inference`'s public fields | `#[non_exhaustive]`, `pub(crate)` fields, accessors | rnnrsvnv | proofs/derivation/view.rs:123-132; accessors at 149 (`sequent`), 154 (`rule`), 161, 166, 174 |
| `Interactive::apply(goal, position, rule, left)` | `Interactive::apply(GoalId, &Step)` | qmpstuqr | proofs/interactive/expand.rs:89 |
| `Interactive::goal(InfId) -> &[OccId]` | `Interactive::goal(GoalId) -> Result<&[Member], StepError>` (design: `Option`) | qmpstuqr, yzywpzko | proofs/interactive/mod.rs:511 |
| `Interactive::rules -> Vec<Rule>` | `Interactive::rules(GoalId, usize) -> Result<Vec<Applicable>, StepError>` | qmpstuqr | proofs/interactive/expand.rs:38; `Applicable` at proofs/interactive/mod.rs:305 |
| `Interactive::new(&s, mode)` (only entry) | `Interactive::within(&s, mode, &Limits)`; `new` its defaults | mwvwsxyk, qmpstuqr | proofs/interactive/mod.rs:424 and 413 |
| `Interactive::close_all -> Result<Vec<…>>` | `Interactive::close_all(…) -> Vec<(GoalId, Result<Closed, Error>)>` (design: `Outcome`) | qmpstuqr | proofs/interactive/expand.rs:441; `Closed` at proofs/interactive/mod.rs:318 |
| `Interactive::inferences()` (public) | crate-private | qmpstuqr | proofs/interactive/mod.rs:493 |
| `Interactive::close_with` grafting any proof | refuses a proof of another forest, another goal, or a forbidden mode | tszyrrtw | proofs/interactive/expand.rs:412 |
| `proofs::Refusal` (a step refused) | `interactive::StepError` | yzywpzko | proofs/interactive/mod.rs:52; lib.rs:187 re-exports `StepError` |
| `ProofStructure::new(forest, mix)` | `new(forest, Criterion)` | kzwvyknq | nets/mod.rs:482 |
| `ProofStructure::from_links(forest, mix, …)` | `from_links(forest, Criterion, &[(VertexId, VertexId)])` | kzwvyknq | nets/mod.rs:522 |
| `ProofStructure::from_proof(&p, mix)` | `from_proof(&p, Criterion, &Limits, stop)` | kzwvyknq | nets/mod.rs:554 |
| `ProofStructure::mix()` | `ProofStructure::criterion()` | kzwvyknq | nets/mod.rs:655; `mix()` absent |
| `ProofStructure::is_correct()` | `is_correct(stop)` | kzwvyknq | nets/mod.rs:808 |
| `ProofStructure::sequentialize()` | `sequentialize(&Limits, stop)` | kzwvyknq | nets/sequentialize.rs:41 |
| links over `OccId` | links over `VertexId` | kzwvyknq | nets/mod.rs:96 (`VertexId`), 691 (`links() -> &[(VertexId, VertexId)]`) |
| `ProofStructure`'s `mix: bool` argument | `Criterion` | kzwvyknq | nets/mod.rs:129 |
| `NetError` exhaustive, over `OccId` | `#[non_exhaustive]`, over `VertexId`, `Refused { refusal }` | kzwvyknq, yzywpzko, kmwqxrzt | nets/mod.rs:176 (enum), 253 (`Refused`) |
| `Verdict::Unprovable(Refutation)` | `Verdict::Unprovable(Box<Disproof>)` | vxmmuxkz | search/mod.rs:1950 (enum), `Disproof` at search/refutation.rs:237 |
| `Refutation::Unbalanced { atom, name, … }` | `Refutation::Unbalanced(Unbalanced { atom, least, most })` | vxmmuxkz | search/refutation.rs:33, 50 |
| `Refutation::Equation { … }` | `Refutation::Equation(Equation { … })` | vxmmuxkz | search/refutation.rs:65 |
| `Refutation::StateEquation { weights, once }` | `Refutation::StateEquation(StateEquation { atoms, clauses, dropped })` | vxmmuxkz | search/refutation.rs:93 |
| `Reason::RecursionLimit` (unit) | `Reason::RecursionLimit { depth }` | tvvvvluq | search/mod.rs:2000 |
| `Reason::CopyBound(u32)` | `Reason::CopyBound { copies }` | tvvvvluq | search/mod.rs:2008 |
| `Reason::MemoryLimit(u64)` | `Reason::MemoryLimit { limit_bytes }` | tvvvvluq | search/mod.rs:2017 |
| (new, no old name) | `Reason::WorkLimit { limit }` | txuvownw | search/mod.rs:2032 |
| (new, no old name) | `Reason::Unchecked { limit_bytes }` | onxqtvpq | search/mod.rs:2040 |
| (new, no old name) | `Reason::setting()` | tvvvvluq | search/mod.rs:2091 |
| `Error::Unchecked` from a search | `Verdict::Unknown(Reason::Unchecked { limit_bytes })` | yzywpzko (removed), onxqtvpq (replaced) | search/mod.rs:2040; `Error::Unchecked` absent |
| `prove_until(&s, mode, &o, FnMut() -> bool)` | `prove_within(&s, mode, &o, &Limits, stop)` | mwvwsxyk | search/mod.rs:229; `prove_until` absent |
| every `impl FnMut() -> bool` stop | `impl FnMut(Progress) -> bool` | mwvwsxyk, txuvownw | limits.rs:197 (`Progress`); `FnMut() -> bool` absent from core/src |
| `search::Options` private fields and setters | `search::Options` public fields, serde, `with_*` builders | tuooxrol | search/mod.rs:1644 |
| `Options::memory_limit` | removed; `Limits::memory_bytes` | mwvwsxyk | limits.rs:51; absent |
| `Options::occurrence_limit` | removed; `Limits::occurrences` | mwvwsxyk | limits.rs:51; absent |
| `Options::recursion_limit` | removed; `Limits::recursion_depth` | mwvwsxyk | limits.rs:51; absent |
| `Options::test_period(Option<u32>)` | `Options::test_period: Cadence` | tuooxrol | search/mod.rs:1554 (`Cadence`) |
| (new, no old name) | `Options::schedule: Schedule` | tuooxrol | search/mod.rs:1597 |
| `Engine` readable only by the command and harness tables | `Engine::ALL`, `NAMES`, `name`, `FromStr` | kqvuwqmr, tuooxrol | search/mod.rs:1314, 1324, 1450 |
| `Bias` without serde | `Bias::ALL`, `NAMES`, `FromStr`, serde | kqvuwqmr, tuooxrol | search/mod.rs:1469, 1473, 1492 |
| `Engine::parallel` a negative match | `Engine::parallel` an exhaustive positive match | ssxwovxw | search/mod.rs:1431 |
| `Statistics`' hand-listed serde proxy | `derive(Serialize, Deserialize)` | vkppkovl | search/mod.rs:2132-2138 |
| `Statistics::memo_entries: usize` | `memo_entries: u64` | tpnlumuk | search/mod.rs:2138 |
| (new, no old name) | `Statistics::forward_copies`, `Statistics::work` | syqnnmno | search/mod.rs:2138 |
| `alone_first` in `cli/src/prove.rs` and `bench/src/run.rs` | `search::race` (feature `parallel`) | zoywtttw | search/mod.rs:378; `alone_first` absent from core/src, cli/src and bench/src |
| `Error::NetFragment` | `NetError::Fragment` (the structure's refusal) | ssxwovxw | nets/mod.rs:231; `NetFragment` absent |
| `Error::NetMode` | `NetError::Mode` | ssxwovxw | nets/mod.rs:237; `NetMode` absent |
| `Error::NetGoal` | `Error::EngineRefused { engine, because: NotTaken }` | ssxwovxw | errors/mod.rs:324; `NotTaken` at search/mod.rs:1188; `NetGoal` absent |
| `Error::EngineMode` | `Error::EngineRefused { … }` | ssxwovxw | errors/mod.rs:324; `EngineMode` absent |
| `Error::NotAdditive` | `Error::EngineRefused { … }` | ssxwovxw | errors/mod.rs:324; `NotAdditive` absent |
| `Error::NotHorn` | `Error::EngineRefused { … }` | ssxwovxw | errors/mod.rs:324; `NotHorn` absent |
| `Error::UnknownRule` | `Error::UnknownName { what, name, known }` | xnszropr | errors/mod.rs:102; used proofs/rule.rs:346; `UnknownRule` absent |
| the command's and the harness's engine name tables | `Engine::NAMES` | uysvqrto | cli/src/argument_parsing.rs:1306; bench/src/run.rs:178 |
| `Error` (35 positional variants, not marked) | `#[non_exhaustive]`, named fields, `kind()`, `code()`, `setting()` | yzywpzko | errors/mod.rs:48, 505, 558, 621 |
| `Error` without `describe`, `Serialize` | `Error::describe(&owner) -> Described<'_>`, `impl Serialize for Error` | uvuuwuym | errors/describe.rs:225; serialize/errors.rs:24 |
| `Error::InvalidVariableIndex` | `Error::IndexOutOfBounds { space, index, len }` | yzywpzko | errors/mod.rs:164; old name absent |
| `Error::TermIndexOutOfBounds` | `Error::IndexOutOfBounds { … }` | yzywpzko | errors/mod.rs:164; old name absent |
| `Error::SubtermIndexNotDecreasing` | `Error::NotTopological { space, index, parent }` | yzywpzko | errors/mod.rs:176; old name absent |
| `Error::NodeIndexOutOfBounds` | `Error::IndexOutOfBounds { … }` | yzywpzko | errors/mod.rs:164; old name absent |
| `Error::PremiseIndexNotDecreasing` | `Error::NotTopological { … }` | yzywpzko | errors/mod.rs:176; old name absent |
| `Error::OccurrenceIndexOutOfBounds` | `Error::IndexOutOfBounds { … }` | yzywpzko | errors/mod.rs:164; old name absent |
| `CheckError::describe` | kept, now returns the shared `Described<'a>` and takes `&impl Owner` | uvuuwuym | proofs/check/error.rs:223 |
| `NetError::describe` | kept, as above | uvuuwuym | nets/mod.rs:300 |
| `ShapeError::describe` | kept, as above | uvuuwuym | occurrences/reading.rs:111 |
| the two `Described` types, `DescribedShape` | one `Described<'a>` (an enum over the four error types) | uvuuwuym | errors/describe.rs:153; `DescribedShape` absent |
| `ParseError` (`Debug`, byte span) | `+ Clone, PartialEq, Eq, std::error::Error`, `span_utf16`, `line`, `column`, `expected` | ukxwvpnq, yzywpzko, nkwtoxsu | errors/parse.rs:13-20 (derives and fields), 107 |
| `Options::DEFAULT_OCCURRENCE_LIMIT` | `Limits::DEFAULT_OCCURRENCES` | mwvwsxyk | limits.rs:93; old name absent |
| `Forest::DEFAULT_LIMIT` | `Limits::DEFAULT_OCCURRENCES` | mwvwsxyk | limits.rs:93; old name absent |
| `Options::DEFAULT_RECURSION_LIMIT` | `Limits::DEFAULT_RECURSION_DEPTH` | mwvwsxyk | limits.rs:98; old name absent |
| `Options::stack_size()` | `Limits::stack_bytes()` | mwvwsxyk, yzywpzko | limits.rs:125; `stack_size` absent as a method |
| (new, no old name) | `Limits::recursion_depth_for_stack()` | yzywpzko | limits.rs:135 |
| `png::Options::memory` | removed; `png::from_svg(svg, fonts, &options, &Limits)` | xymzlmnl | export/png.rs:89 |
| `pdf::Options::memory` | removed; `pdf::from_svg(svg, fonts, &options, &Limits)` | xymzlmnl | export/pdf.rs:183 |
| `{latex, typst, svg}::derivation` | removed; one `write` per target | knkmnqxz | export/latex.rs:404, export/typst.rs:456, export/svg/mod.rs:870 |
| `{latex, typst, svg}::ordinary` | removed; `write(impl Into<Drawable>, …)` | knkmnqxz | same three lines; `Drawable` at export/mod.rs:134 (a public enum, design: sealed) |
| `{latex, typst, svg}::write` per type | `write(&impl Into<Drawable>, &options, out, stop)` | knkmnqxz | export/latex.rs:404, export/typst.rs:456, export/svg/mod.rs:870 |
| `{latex, typst, svg}::sequent` without a bound | `sequent(&s, mode, &options, &Limits)` | pkkmlunw | export/latex.rs:382, export/typst.rs:431, export/svg/mod.rs:787 |
| `{latex, typst, svg}::two_sided` | removed; `sequent` takes the mode | pkkmlunw | absent |
| `svg::net(&n, &style, Option<u64>)` | `svg::net(&n, &style, &Limits, stop)` | xymzlmnl | export/svg/mod.rs:910 |
| `png::from_svg(svg, fonts, &options)` | `from_svg(svg, fonts, &options, &Limits)` | xymzlmnl, yzywpzko | export/png.rs:89 |
| `pdf::from_svg(svg, fonts, &options)` | `from_svg(svg, fonts, &options, &Limits)` | xymzlmnl, yzywpzko | export/pdf.rs:183 |
| `rocq::Options.lemma: String` | `lemma: Identifier` (checked) | xnvtspot | export/rocq.rs:90, `Identifier` at 150 |
| `rocq::Options.prelude: String` | `prelude: Option<String>` | xnvtspot | export/rocq.rs:90 |
| `rocq::Unsupported` exhaustive | `#[non_exhaustive]` | lszkzunv | export/rocq.rs:226 |
| the command's `Styles` | `export::Styles` | umntwmyy | export/styles.rs:25; re-export export/mod.rs:63 |
| (new, no old name) | `settings::Settings`, `settings::Clock` | umntwmyy | settings.rs:112 and 28; lib.rs:193 |
| `batch::Options` with public fields | `#[non_exhaustive]`, serde, `Cores` | trzpqqmy, lszkzunv | search/batch.rs:74; `Cores` at 43 |
| `batch::Plan` with public fields | `#[non_exhaustive]` | trzpqqmy, lszkzunv | search/batch.rs:186 |
| `batch::Problem` with public fields | `#[non_exhaustive]`, `Problem::new` | trzpqqmy, lszkzunv | search/batch.rs:210 |
| `batch::Answer` with public fields | `#[non_exhaustive]` | trzpqqmy, lszkzunv | search/batch.rs:234 |
| `batch::Options::plan(within: bool)` (public) | crate-private `plan(&self, within, &SearchOptions, &Limits)`; `Plan::alone`; design `plan(Cores)` not built | trzpqqmy, wnqvqlpx | search/batch.rs:131 and 198 |
| (new, no old name) | `batch::Cancel`, `Results::cancel` | trzpqqmy | search/batch.rs:334 and 361 |
| (new in the design) `batch::run_local` | not built (decided: waits for its caller) | none | absent from core/src (grep) |
| `ordinary::Sequent::new -> Self` | `-> Result<Self, Error>` | zlxzmzlm | ordinary/mod.rs:651 |
| `ordinary::Formulas::add` unchecked | already `Result<NodeId, Error>` before step 28 (wkookxyw); step 28 added the occurrence bound | luzxwzwl, sllwwuqu | ordinary/mod.rs:435 |
| `Translation::target() -> &str` | `-> Target` | zlxzmzlm | ordinary/mod.rs:229; `Target` at 247 |
| `Image::read_back(&Derivation)` | `read_back(&Proof, &Limits, stop)` | zlxzmzlm | ordinary/derivation.rs:749 |
| `ordinary::Derivation::check()` | `check(&Limits, stop)` | zlxzmzlm | ordinary/derivation.rs:333 |
| (new, no old name) | `ordinary::decide`, `ordinary::Outcome` | zlxzmzlm | ordinary/decide.rs:76 and 17 |
| `families::Family::instance` (panics) | `-> Result<Instance, Error>` | rxyloqup | families.rs:85 |
| JSON key `ids` (sequent) | `roots` | mnsynnuz | serialize/sequents.rs:67; core/tests/serialize.rs:36; old key refused at tests/serialize.rs:820 |
| JSON key `var_dict` (sequent) | `atoms` | mnsynnuz | serialize/sequents.rs:69; core/tests/serialize.rs:36; refused at tests/serialize.rs:820 |
| JSON key `proof` (proof form) | `nodes` | mnsynnuz | serialize/proofs.rs:77; core/tests/serialize.rs:218 |
| JSON key `memory_limit` (reason payload) | `limit_bytes` | mnsynnuz | serialize/search.rs:70 and 82; core/tests/serialize.rs:372 area |
| JSON key `once` (`StateEquation`) | removed with the new payload (`atoms`, `clauses`, `dropped`) | vxmmuxkz | serialize/search.rs:166-172; `once` absent |
| the mode as an object (`ModeDef`) | the mode by name (`"affine"`) | mnsynnuz | serialize/search.rs:33; core/tests/serialize.rs:285-314 |
| `"reason": {"copy_bound": 3}` | tagged `{"kind": "copy_bound", "copies": 3}` | mnsynnuz, tvvvvluq | serialize/search.rs:53; core/tests/serialize.rs:372 |
| untagged refutations | tagged refutations (`{"kind": "equation", …}`) | mnsynnuz, vxmmuxkz | serialize/search.rs:135; core/tests/serialize.rs:347 |
| JSON tags `V`, `D` (stay) | unchanged | none | serialize/sequents.rs:16 and 19 |
| `linlog_cli` modules `argument_parsing`, `batch`, `interact`, `io`, `ordinary`, `prove`, `settings`, `style` (public) | private (`mod`) | xmyntvsq | cli/src/lib.rs:9-34 |
| `linlog_cli::{interrupted, clear_interrupt, catch_interrupt, exit_on_interrupt, parse_error, Status}` (public) | `pub(crate)` | vxwmzrmx | cli/src/lib.rs:46, 107, 112, 120, 134, 188 |
| `linlog_cli` public surface | `main` is its one public item (design: `run`) | xmyntvsq | cli/src/lib.rs:285 |

Paths are relative to `core/src/` unless they start with `cli/`; a commit is the jj change of step 28 that made the change, the first named where several did.

Departures from 2.4, where the code differs from the design's table:

- `Atom`'s doc comment: the design says an atomic formula, nullary until step 38. The comment at `sequents/term.rs:28` still says "a propositional variable".
- `sequents::fmt::{Walk, Visit}` stay crate-private and `Visit::Between` stays `Between(T)` (`sequents/fmt.rs:19`, `26`, `53`); the design made them public with `Between(TermId, index)`. No "decided unattended" section records this.
- `Forest::from_owned` stays public, now with `&Limits` (`occurrences/mod.rs:289`); 2.3 would have made it private. Area 3.1 decided this and the author answered "kept".
- `ViewError`, `WriteError`, `RenderError` and `svg::TooLarge` became `Error::Check(CheckError)`, `Error::Refused(Refusal)`, `Error::WriteFailed` and `Error::RenderFailed`; there is no `Error::InvalidProof` (the design's name). Area 3.1 records a refused check as `Error::Check(CheckError::Refused)`.
- `Rule::ALL` is `[Self; 16]` and `Named::ALL` is `[Self; 34]` (`proofs/rule.rs:51`, `156`), not `&'static [Rule]`.
- `Interactive::goal` answers `Result<&[Member], StepError>` (`proofs/interactive/mod.rs:511`), not `Option<&[Member]>`; `rules` answers `Result<Vec<Applicable>, StepError>`.
- `Interactive::close_all` answers `Vec<(GoalId, Result<Closed, Error>)>` with a new `Closed` struct (`proofs/interactive/mod.rs:318`), not `Result<Outcome, Error>`.
- `Refutation`, `Disproof`, `Equation`, `StateEquation` and `Unbalanced` live in the private module `search::refutation`, re-exported from `search` (`search/mod.rs:38`); only `Refutation` and `Disproof` reach the crate root (`lib.rs:188`). The design's top-level `refutation` module does not exist.
- `engine_for` is `search::engine_for` only; lib.rs's root re-export (`lib.rs:188-191`) has `prove`, `prove_goal` and `prove_within` but not `engine_for`.
- `CheckError::describe`, `NetError::describe` and `ShapeError::describe` still exist as methods; they now return the one shared `Described<'a>` and take `&impl Owner`. The design's "one writer per type" is met by sharing the type, and the methods were not removed.
- `export::Drawable` is a public `#[non_exhaustive]` enum (`export/mod.rs:134`), not a sealed trait; `write` takes `impl Into<Drawable>`.
- `batch::run_local` does not exist (area 3.1: it waits for the web client). `plan(Cores)` is not built: `Cores` is a field of `batch::Options` and `plan` is crate-private with three arguments.
- `ordinary::Formulas::add` was already checked (returned `Result<NodeId, Error>`) before step 28 began (commit wkookxyw, 2026-10-04); step 28 only added the occurrence bound. The design's "unchecked" is stale.
- `linlog_cli`'s one public item is `main`, not `run` (`cli/src/lib.rs:285`). The sentence that it carries no semver promise is not in `cli/src/lib.rs` (its docs say it is a library "so that the binary is one call and this documentation exists"); a grep for "semver" in `cli/` and `.claude/rules/cli.md` found nothing.

### The design and the first-order plan in a page

The design (`plan/notes/api.md`) rests on a few rules, each of which the
fixes implemented:

- **One options value per concern, plain data with a wire form**
  (`Settings`: `clock`, `limits`, `search`, `view`, `styles`, `batch`,
  `ordinary`), which the command, the harness and the web client set
  alike; the command's behaviour is `Settings::default()` and every flag
  is the spelling of one key (`--settings FILE` reads the whole value).
- **One bounds value and one stop** for every call that can run long:
  `Limits` (memory, work, occurrences, derivation bytes, recursion
  depth) and `FnMut(Progress) -> bool`, so that a browser tab can bound
  and cancel each call. A refusal at a bound is never a verdict: the
  checker's `CheckError::Refused` beside `Invalid`, a search's
  `Verdict::Unknown` with its `Reason`.
- **One error family** with a kind, a code, the setting that would lift
  a bound, and a written form; `#[non_exhaustive]` on every open type,
  and the closed ones (`Term`, `Kind`, `Node`, `Rule`, `Verdict`, …)
  listed with the release that adds to them.
- **The written order of a sequent is canonical** (C1): the roots stay
  as typed, and `Sequent::antecedents` records the two-sided reading.
- **An atom is an atomic formula**, nullary until quantifiers: the
  `Atom` index is the place a predicate symbol with its arguments takes.
- **Every wire form has a level** (`wire::LEVEL`, 1), the old field names
  read by `wire::upgrade` only where a converter is needed; a client
  treats `Engine` and `Reason` as open enumerations.
- **Engines are rows of a table**: `search::DISPATCH` names each engine,
  its fragments and modes, and every engine answers through `Decide`;
  a new calculus or engine is a row, never a new path through the front
  door.

**The first-order plan** (`api.md` 3.14 and 10.10). The first step of
adding quantifiers is the data model alone, with its refusal everywhere:
`Term::{Forall, Exists}` and `Kind::{Forall, Exists}` appended after the
compound variants (the literal variants stay first and contiguous, so
the literal test stays one comparison), the first-order tables of the
sequent (symbols with arities, a hash-consed term arena with de Bruijn
indices, argument lists, binder names for printing), empty for
propositional input, a quantifier bit, and in the same commit every
engine's `admits` refusing the bit and the checker refusing a forest
with binders until its frame pass exists. Ground first-order input is
then accepted by every engine as propositional input with structured
atoms. Later commits add, one layer each: a member past the forest as
an occurrence with a frame of the terms its binders are bound to
(`Instances`, hash-consed cons cells); a substitution beside the forest
in the engines (`Bindings`, a `Trail` with a mark per choice point,
fresh variables numbered per worker, the eigenvariable condition kept
by levels); witnesses and eigenvariables in proofs and their JSON;
`Limits::terms` for the unfolded size of terms. The spike measured what
the propositional case pays (`api.md` 11.5): the design's data model
(M1d) cost nothing measurable on the twenty journeys and kept the
target set's counters exact; making the focused engine generic over a
zone with a second, first-order instance (M3i) stayed within the gates
once the shared helpers were inlined. The propositional engine is then
the ground instance of one generic engine, provisional on a pinned-time
run at step 37.

### The efficiency journeys, stage 0 to the head

Each journey's ceiling in `bench/ceilings.csv` at stage 0 (zrvpwtoq)
and at the head. The head's ceilings were re-recorded under link-time
optimisation (wunyrqsz, area 3.1, the author's T3) and lowered after
area 3.3's fixes (ukvsppkr); the per-fix measurements, with pinned
times, are in "Area 3.3: the journeys before and after" above. The
raises, each a commit of its own with its reason: omsrptkl (the check
journeys, the written root order), suorpuok and klsnlozv (read-json, the
occurrence count and the atom names checked in NFC), roxtvnkn (read-tptp,
the code layout), klmlrxkr (search-additive-14, the layout a dependency
moves), ryolrmpw (render-svg and read-text, the checked style and the
escapes), mvwmtwtm (the check journeys, the entries counted), twykwnwo
(the printing journeys, the one printer), trxxrstp (read-lltp, the
layout), ypzmzomx (read-tptp, the ordinary parser's bound).

| journey | stage 0 | head | change |
|---|--:|--:|--:|
| `search-qbf-20-2` | 661 564 923 | 506 226 963 | −23.5 % |
| `search-chain-128` | 1 216 594 953 | 972 954 280 | −20.0 % |
| `search-partition-no-5` | 8 523 805 | 7 121 818 | −16.4 % |
| `search-wide-m2-256` | 274 070 486 | 260 393 363 | −5.0 % |
| `search-spec-chain` | 40 115 539 | 23 272 558 | −42.0 % |
| `search-chain-64-intuitionistic` | 154 301 282 | 123 663 470 | −19.9 % |
| `search-additive-14` | 608 840 614 | 584 985 652 | −3.9 % |
| `read-text` | 18 548 925 | 12 269 416 | −33.9 % |
| `read-json` | 12 515 726 | 12 475 495 | −0.3 % |
| `read-lltp` | 32 990 483 | 27 079 341 | −17.9 % |
| `read-tptp` | 30 658 823 | 32 958 815 | +7.5 % |
| `read-spec` | 22 458 173 | 17 066 792 | −24.0 % |
| `check-qbf-20-2` | 135 217 544 | 136 738 432 | +1.1 % |
| `check-wide-m1-2048` | 8 772 182 | 8 794 809 | +0.3 % |
| `derivation-chain-64` | 17 006 453 | 9 734 603 | −42.8 % |
| `render-latex` | 6 585 706 | 5 736 242 | −12.9 % |
| `render-typst` | 5 779 945 | 4 787 017 | −17.2 % |
| `render-svg` | 86 837 537 | 84 853 287 | −2.3 % |
| `batch-families` | 81 563 494 | 69 573 241 | −14.7 % |
| `ordinary-pigeons` | 39 298 004 | 32 574 174 | −17.1 % |
| all twenty | 3 462 244 597 | 2 933 259 768 | −15.3 % |

### The check rounds

- **Area 3.1** (session `step-28d`): three rounds of the audit's lenses
  one effort lower and a fresh-context reviewer (Fable 5.1, soundness
  first, by witness programs). Round 1 over 105 findings: 71 fixed, 28
  partly, 6 not, and 24 new (two must-fix: a batch that panicked when a
  worker could not start, a fault's formula written whole in an
  abbreviated report). Round 2: 44 fixed, 14 partly, 8 new, all fixed.
  Round 3: 12 fixed, 10 partly (recorded decisions and the command's
  halves), 5 new with one must-fix (`ALL` as an array on open
  enumerations), all fixed. The reviewer found one must-fix (minimal
  logic by call-by-value refuting valid sequents that mention `false`)
  and four should-fix, all fixed.
- **The command's wrong answers** (session `step-28e`, before 3.2):
  F165, H5, H11, H12, H13, H16, H26, H28, H30, H31, each with the test
  that would have caught it, verified by the supervisor on a release
  build.
- **Area 3.2** (session `step-28e`): the panel of step 26's three lenses
  over the seven changes of the search (95 634 runs over 23
  configurations, no counterexample; the argument and the integers
  lenses each refuted one claim, both fixed); three rounds (round 1: 37
  fixed, 20 partly, 5 not, 18 new; round 2: 28, 10, 5, 11 new with one
  must-fix, a doc; round 3: 8, 8, 2, 7 new, all fixed); the reviewer
  (Fable 5.1, an independent unfocused prover over 150 000 runs) with no
  finding.
- **Area 3.3** (session `step-28f`): two rounds of eight lenses (round 1:
  every perf finding fixed but F147 partly and the decisions F148 and
  F153; round 2: nothing new), the reviewer (Opus 5.5, five should-fix,
  all taken), the mutants and fuzzers of what changed.
- **Area 3.4** (session `step-28g`): three rounds of nine lenses (round
  2's API lens found a must-fix in round 1's own fix, undone; round 3's
  input lens a batch's lines read without a cap, fixed), the reviewer
  (Opus 5.5, one must-fix: `--batch-timeout` not stopping an `--isolate`
  child; eight should-fix, all fixed), the ultra review's U1 fixed, the
  last reader (Fable 5.1) of README and the rustdoc front page, and
  stage 0's evidence taken again in full (every mutation batch, every
  fuzz target, the ratchet, the families, the target set).

### Decisions, and the departures from the design

The decisions of each area are in its "decided unattended" section
above; the author's answers to the design's questions are `api.md` 14.
The departures from the signed-off design, in one list:

- **Area 3.1, answered by the author on 2026-10-10.** Kept:
  `Forest::from_owned` public; errors give no `source()`; an error from
  `prove_within` carries no formula text; `ProofStructure::occurrence`
  answers `Option<OccId>`; `ordinary::decide` answers a refused
  read-back as that refusal; a keyword names no atom in the text
  either; ordinary names refuse `true` and `false`; `unicode-ident` and
  `unicode-normalization` in every build; `batch::Options::plan`
  crate-private; `Progress::item` names a session's goal only. Sealed
  as the design had them: `Owner` and `wire::Readable` (pysomtmo).
  Waiting for the web client: the JSON forms of the ordinary sequent and
  derivation, the linear derivation, `Step`, `Applicable`,
  `batch::Answer` and `ordinary::Outcome`.
- **Area 3.2.** `Decide::decide` takes `FnMut(u64) -> bool` and the
  front door builds the `Progress` (the public calls take
  `FnMut(Progress)` as designed); a search tells its stop `held_bytes`
  0; there is no crate-private `Engine::modes()`, each `admits` keeping
  its own mode test.
- **Area 3.3.** `Limits::PER_LEVEL` raised to 3 KiB (release) and
  13 KiB (debug), where the design gave 2 304 bytes a level; F148 (the
  sparse memo key) not done; F153's rest not done; the target set run
  at the head only, not after each commit that touches `search/`; the
  mutation run from the main checkout.
- **Area 3.4.** `--jobs` sets two keys (`search.jobs`, and
  `clock.pool_after_ms = null` unless `--pool-after` is given); an error
  record of the batch is `"verdict": "error"` (api.md 7.3 and 7.4 had a
  record hold an outcome or an error); CI builds x86_64-linux alone,
  the other systems evaluated, not built.

### Open questions for the author

The decisions the review sections hand to the author at the step's end:

1. The departures of areas 3.2, 3.3 and 3.4 above (3.1's are answered).
2. F148, the sparse memo key: up to about 1.7 times as fast per stable
   sequent on the widest two-sided searches under the command's default,
   for a rewrite of `memo.rs` and the shared memo, and no verdict the
   target set shows (`plan/later.md` keeps the numbers).
3. `Limits::work` does not count the focused engine's `&` rules, so
   `ordinary_decide`'s 53 fuzz timeouts and 16 slow units stay; counting
   them moves the default bias's slices and needs measurements.
4. A text nested deeper than `--occurrence-limit` is refused as over
   the bound (documented on `Limits::occurrences`).
5. Found by area 3.4's review and not fixed: after the focused engine
   finds a proof of `&` rules closed by `⊤`, about a second passes
   without a poll (24 pairs end at 1.9 s under limits from 0.9 to 1.8 s;
   26 pairs at 6.4 s under 4 s), bounded by the search's own work; and
   that proof is not counted against `--memory-limit`, so 26 pairs
   peaked at 4.6 GiB under the default 1 GiB, while README says a search
   holds at most `--memory-limit` bytes. A proof of `2ⁿ` branches from
   `n` pairs is what leaving "the proof returned" outside the account
   did not foresee. Both are code changes for a session after this one.
6. Left open by area 3.4, with reasons: F188 (a test of `compare.rs`'s
   printed tables), F214's timeout arm (an `--isolate` child that
   outlives its limit on purpose), F218's scaling tests per reader; and
   the last reader's remaining obstacles, in `plan/later.md`, "a first
   user's obstacles".
7. Not done and recorded nowhere until this write-up: the design's
   public walks (`api.md` 2.4 and 3.6, R245): `sequents::fmt::{Walk,
   Visit}` stay crate-private and `Reading::walk` does not exist. Left to
   step 29 by the supervisor: its prompt now says to do it first. The
   three smaller points the checked table found are fixed (txykwztn, the
   gate passed): `Atom`'s doc comment calls it an atomic formula,
   `cli/src/lib.rs` says that `linlog_cli` is no API and makes no semver
   promise (`api.md` 2.3, R253), and `.claude/rules/cli.md` names
   `searched` and `search::race` where it named `alone_first`.
8. Found by this session's first `nix flake check` (which passed), and
   fixed: the features check warned, under every feature set with
   `interactive` and without `serialize` (ten of them), that six imports
   of `core/src/proofs/interactive/replay.rs` were unused, since its two
   items were each gated on `serialize` and the move that made the file
   (mpuzmwmy, F59) did not gate the module. rsvlqous gates the module on
   `serialize` (the gate passed; both `cargo hack` runs passed with
   `-D warnings`), and krxomqoz denies warnings in the gate's two
   `cargo hack` runs (in `target/hack`, since `RUSTFLAGS` are part of
   every artifact) and in the flake's `features` check (`env.RUSTFLAGS`),
   so that a move cannot bring one back unseen: the gate passed with
   them, and `nix build .#checks.x86_64-linux.features` built.
9. The third ultra review (`review/3.4a`, `review/3.4b`): its findings,
   if any, are fixed in a short session after this one, with the gate
   and `nix flake check`.
10. F122 and F123, both should-fix and left open by area 3.2 ("Area 3.2:
    decided unattended", `plan/later.md`, "The generators' contraction
    and mutants"). The generators' fixes exist off the line, signed,
    under the local bookmark `wip/generators`: supyqoln, F122's real
    contraction, finished, and ukrmvowr, F123's transposition, not
    finished. Each found a sequent the engines do not decide, and on the
    head's release build (2026-10-14) both are still undecided after
    60 s:
    - F122's sample is no proof the generator built but the mutant of
      one (corrected by `step-28i`: the shift of draw 32 of rule set 6 in
      `generated_ill`, seed 106, whose first hypothesis `a` became `b`).
      It is unprovable: the goal's tensor factor `a` has no `~a`
      anywhere in the sequent. Neither engine refutes it, intuitionistic
      (two-sided engine, copy bound 7 reached) or classical (focused
      engine, copy bound 4 reached);
    - F123's Mix mutant does not finish copy bound 4: 787 million stable
      sequents visited, 741 million of them from the memo, which holds
      at most 1 731 entries.

    The author's answer (2026-10-14): a session looks at the engines on
    both sequents first, then lands both generators.
    **Done in step 28i** ("Follow-up: F122, F123 and the engines on
    their sequents, item by item"): the Mix mutant's failures rested on
    the loop check and were never memoized; they now wait on their
    ancestor (qoksszzp, ytvrknnp), and the sequent's copy bound 3 takes
    17 179 stable sequents where it took 76.5 million. F122's sample is
    refuted at once by the count of one sign (kvpyrzuv, question 11).
    Both generators landed (wzynqlqo, ypvxqstw).

11. **A count of one sign for the atoms under exponentials** (found by
    `step-28i`, put to the author through `planning`, not built). The
    engine's counts give up on every atom that has a literal under `?`
    or `!`, so they never see that such an atom cannot balance. The
    check would keep those atoms' counts outside the exponentials and
    let the count grow without end only in the direction the literals
    under exponentials can push it: with no `~a` under any exponential,
    copies can only add `a`s, so the `a`s outside must not already
    outnumber the `~a`s. F122's sample is the example: its goal needs an
    `a` that nothing supplies (the only other `a` lies under `!`, and
    there is no `~a` anywhere), so it is unprovable, yet every copy
    bound leaves it undecided; the check would refute it at once. The
    cost is a change to the counts in three places (their rows, the test
    of a stable sequent and the incremental test of a split), a
    refutation with a bound on one side only (a change of its JSON
    form), and the full evidence of a search change. The
    recommendation: do it as a change of its own in a later session,
    after measuring on the LLTP library how many problems it would
    decide that are undecided today, since its reach beyond generated
    mutants is unknown and the tests already bound such mutants'
    searches (`plan/later.md`, "A count of one sign").
    **Answered by the author** (2026-10-14, through `planning`): build it
    in this session. **Built in step 28i** (kvpyrzuv; the lock's
    fixture ymmluvqt): an atom whose literals below the exponentials all
    have one sign keeps its rows, a balance is ruled out on the side the
    copies cannot reach, and the refutation is a kind of its own,
    `one_sided`. Its reach on LLTP is small: of the 1 663 problems with
    such an atom it decides 1 more at the harness's copy bound of 3
    (SYN389+1 in the 01 translation, refuted) and 2 more without a copy
    bound (SYJ205+1.009 and .010 in cbn, proved); none is lost.

### What steps 29 to 31 must know

- **Step 29 (the comparison).** A driver reads the outcome's JSON
  (`version`, `verdict`, `engine`, `statistics`, `linlog`): the verdict
  is a name from `Verdict::NAMES`, and a client treats `engine` and
  `reason` as open sets. The one-thread column is `--deterministic`
  (`search.jobs` 1, `Schedule::Turns`, no pool), the default column the
  library's `search::race`, which the command and the harness share and
  which draws both searches from one memory account, so a BenchExec
  memory limit equal to `--memory-limit` holds (below three threads
  there is no single thread: `--jobs 2` is a pool of two). "Outside the
  fragment" is `ErrorKind::Unsupported` or an `EngineRefused` error,
  never unknown. A translator reads LLTP's axioms and conjecture from
  `Sequent::antecedents`, and the roots keep their written order; the
  public walk the design gives translators (`sequents::fmt::Walk` and
  `Visit`, `Reading::walk`, R245) was not made public: `Walk` and
  `Visit` are still `pub(crate)` (`core/src/sequents/fmt.rs:19`,
  `:53`), and step 29's prompt now says to make them public first. A
  `proved` outcome is checked ground truth (`checked`); an `unprovable`
  one carries a `Disproof` that nothing checks until step 31, so a
  contradiction against it is listed as unchecked. The counters mean
  different things per engine (`Engine::counters` lists them; the Horn
  engine's `nodes` are markings reached). The harness's columns are
  read by name; `summary --against … --counters` and `--strict` are the
  checks of a run; the oracles are `bench/targets/after-efficiency.csv`
  and `net-after-efficiency.csv`.
- **Step 30 (the release).** The workspace inherits one version and
  edition from the root manifest, and the flake's crane build reads it
  there (omxtnrly): 0.1.0 is one line. The wire forms are promised by
  level (`wire::LEVEL` 1), independently of the crate's version; the
  closed enums gain variants at 0.2.0 (cut) and 0.3.0 (quantifiers),
  which the changelog's policy line names. `linlog_cli` is an internal
  library (`main` alone public); `cargo-semver-checks` would cover
  `linlog` only. The `wasm` flake check builds the library for wasm32
  with every feature but `parallel`; `doc-private` builds the private
  docs with warnings denied. The baseline's `baseline.sh` takes the
  second baseline's reference and reruns as parameters
  (`--reference=`, `--reruns=`). The pool's speed on two and four
  threads after H18 and F92 was measured by area 3.3 on the target set
  at `-j 2` and `-j 4` against kktkswpq (no verdict differed, the
  summed times within 1 %), not by a baseline: the third baseline's
  parallel pass is the first at scale. `check` now has a time limit (2 s by default)
  and answers unknown, exit 3, at a limit.
- **Step 31 (the Rocq library).** Its prompt names `core/src/proofs/check.rs`
  and `Problem::Surplus` and `Problem::Memory`; at the head the checker
  is `core/src/proofs/check/` (`mod.rs` the entry, `pass.rs` the rule
  table, `zones.rs` the zones, `error.rs` the errors), the faults are
  `Fault` (`Fault::Surplus`, `check/error.rs`) and a refusal is
  `CheckError::Refused`, no `Fault` at all: the memory refusal is no
  part of the algorithm, as the prompt says. The closer text to
  translate is still `core/src/proofs/oracle.rs`. `Node::NAMES` and
  `Node::TAGS` are pinned to the variants (`name_lists_follow_the_variants`)
  for the test against the Rocq constructors. A `Proof` carries its
  goal and mode (`Proof::goal`, `Proof::mode`), so a certificate names
  what it proves. `rocq::Options` has `lemma` checked as an identifier
  and `prelude: Option<String>`, and `Kernel` and `Unsupported` are
  non-exhaustive for the second kernel. `Refutation` (`Unbalanced`,
  `Equation`, `StateEquation`) is non-exhaustive and its payloads are
  structs, so `Classical` is an addition; `Disproof` exists and has no
  `check` yet. C2's answer puts the classical refuter in the library,
  after the search, with `refute_unknown` off by default; none of it
  exists yet (`api.md` 8.7 and 10.3). No nullary Mix (C3). `Node` and
  `Rule` are closed on purpose: cut is a new release of the calculus
  (0.2.0).

### The register after step 28

Each of the register's 254 entries (`plan/notes/requirements.md`) now
has a line "State after step 28:" beside stage 0's "State now": met,
with the file and line or the test that satisfies its "Met when"; or
partly or not met, with what is missing, its reason and the step or the
`plan/later.md` entry that takes it. One read-only agent per kind
(Sonnet 5.5 at `high`) wrote them from the code at the head; this
session spot-checked a sample of the "met" claims (the tests and lines
cited for R9, R23, R40, R44, R133, R164, R171, R183, R213, every one
there) and moved R194 from met to partly (its check reads the third
baseline, which step 30 makes).

| Kind | Requirements | Now: met | Now: partly | Now: not met | Now: unknown | After 28: met | After 28: partly | After 28: not met |
|---|---|---|---|---|---|---|---|---|
| Wire forms | 18 | 0 | 5 | 13 | 0 | 1 | 10 | 7 |
| Bounds and stops | 25 | 1 | 9 | 15 | 0 | 4 | 13 | 8 |
| Wasm portability | 9 | 3 | 4 | 2 | 0 | 2 | 6 | 1 |
| Data model | 41 | 1 | 13 | 27 | 0 | 3 | 19 | 19 |
| Engine interface | 32 | 2 | 7 | 23 | 0 | 4 | 9 | 19 |
| Proof term and checker | 14 | 0 | 3 | 11 | 0 | 0 | 5 | 9 |
| Errors | 10 | 0 | 5 | 4 | 1 | 1 | 7 | 2 |
| Options | 13 | 1 | 5 | 7 | 0 | 1 | 9 | 3 |
| Export | 21 | 4 | 3 | 14 | 0 | 4 | 4 | 13 |
| CLI | 12 | 2 | 2 | 8 | 0 | 2 | 2 | 8 |
| Harness | 34 | 6 | 12 | 15 | 1 | 5 | 14 | 15 |
| Docs | 16 | 0 | 4 | 12 | 0 | 2 | 6 | 8 |
| Other | 9 | 0 | 2 | 6 | 1 | 2 | 3 | 4 |
| Total | 254 | 20 | 74 | 157 | 3 | 31 | 107 | 116 |

Most entries not met are a later step's own work (a calculus, an engine,
the web client's bindings, the Rocq library, the release's files) for
which step 28 left the place its line cites; "partly" is most often the
room built and a test or a document still missing. Three are step 28's
own and not done: R245 (the public walks, open question 7), R87 (the
dispatch table's measurements tied to `search::DISPATCH` by a note or a
test, which `api.md` 8.4 promised), and R138's tests (a check refused
at the memory bound through `prove`, and the default bias's merged
reason), which go to the supervisor with this session's last message.

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

### Messages from `planning` (2026-10-09, evening)

- The probe's findings H22 to H31 are in the register (f2b974be): H22
  (must-fix), H23, H24, H25 and H29 are this area's; H27 the search's;
  H26, H28, H30 and H31 the command's, fixed here only where the code
  is rewritten anyway, said in the checklist. A witness run once is no
  benchmark. What was done: acknowledged; H23 and H24 in item p.
- gpg-agent's cache ended: commit unsigned with `--config
  signing.behavior=drop`, the gate's jj calls too; the supervisor
  re-signs from trzotrqx on after the area. What was done: from
  trzotrqx to kwxnmzvw unsigned.
- Signing is back from 19:49 for about two hours: test a signature
  before each commit and fall back when it fails. `bench/net-targets.sh`
  is a benchmark run: it goes into the go batch, at the retype and at
  its parent. What was done: acknowledged; the batch lists both.

### Messages from `planning` (2026-10-09, night)

- The go for the benchmarks: the machine is free from 21:50 until the
  author says otherwise; no asking run by run, one line to `planning`
  when each run starts and ends; the batch runs where each run fits
  (C1's target set on its own build, net-targets at the retype's parent
  and at the retype, T3 with its ceilings now; the head's target set,
  qcover and the families at the area's end; mutants and fuzzing in the
  check rounds); a run whose time matters stays off the gates' cores 2
  to 5, or its times are indicative. What was done: the three
  old-revision runs in their workspaces one after another
  (`../linlog-at-kwx`, `../linlog-at-kzw`, `../linlog-at-c1`), each
  start and end sent; results in items f and p; T3 after the ceiling
  raises and the gates (item g).
- Signing is back from 23:23, good until about 01:20: test before each
  commit, drop when it fails; the unsigned commits from trzotrqx on
  stay for the supervisor to re-sign after the area. What was done:
  acknowledged; the ceiling raises rebased the area's later commits,
  and jj signed every rewritten commit while the key was cached: of
  the area's commits only trzotrqx, mnkunorl and kwxnmzvw are unsigned
  now.
- Cores 0 and 1 are the desktop's: the families and qcover runs there
  were fine this once, since the machine is free and they are verdict
  runs; every later run keeps to cores 2 to 15 and queues behind the
  others rather than take 0 or 1. What was done: acknowledged.

### Messages from `planning` (2026-10-10, night)

- Signing: the cache's window ended at about 02:20; from otvprpqq on the
  area's commits are unsigned (`signing.behavior=drop`), for the
  supervisor to re-sign with trzotrqx, mnkunorl and kwxnmzvw. What was
  done: told `planning`.
- Gate queues: one launcher waited on a process id that `pgrep -f`
  matched in its own shell, so skrpywxo's gate started early (it
  passed) and the launcher after it was stopped and chained again by
  the right id; no gate was lost.
- The author's answers (2026-10-10, morning): (1) build the `Disproof`,
  `Reason` and `Refutation` readers now, as design 7.2's table says,
  each with a round trip test and an unknown kind refused as malformed,
  each its own commit through its gate, which closes F87; (2) `Sequent`'s
  `Display` stays one-sided; (3) F59's moves go into area 3.4. And:
  the cbv-minimal wrong answer is the register's held-back H1. What was
  done: the readers in kvrrwumq, ywyytxwu and myllqqsn with their tests,
  utzltllk their fuzz target; the decisions above rewritten; F59 moved
  from `plan/later.md` into area 3.4's item of the step file; H1
  recorded as fixed by orszuxqx with its witness in yxxuumxn, H21 not
  covered. Signing came back at 09:20; these commits are signed but
  kvrrwumq and ywyytxwu, made before it.
- Hold (2026-10-10, 09:26 to about 09:42): the author needed the machine
  quiet. What was done: the gate queue and the fuzzing stopped at 09:26.
  Left: the gates of ywyytxwu (stopped in its tests), myllqqsn and
  yxxuumxn (kvrrwumq had passed); the fuzz target `json_disproof` (3 of
  its 15 minutes run, coverage 2 839 after 11.7 million inputs, no
  find), to run again from the
  start. Both restart on `planning`'s go.
- Go (about 09:35): the runs restarted and ended. What was done: the
  gates of ywyytxwu, myllqqsn and yxxuumxn passed (kvrrwumq before the
  hold), and so did mzxpvzpm's, which names the nets' test's `alls`
  `additive` for the flake's `typos` check (`planning`'s find); the fuzz
  target `json_disproof` ran its 15 minutes: 61 million inputs,
  coverage 3 138, no find. `nix flake check --keep-going` on the head
  (mzxpvzpm and the records after it) passed, every check of the flake,
  which closes the area.


### Where you start, `step-28e` (2026-10-10)

- You are `step-28e`, the fifth session of step 28, Opus 5.5 at
  `xhigh`, in two parts in this order: (1) the command's wrong answers
  first, by the author's leave to reorder: F165 (`--isolate` with
  bundled short flags), H11 (a typed SEQUENT ignored in a batch), H12
  (an empty batch exits 0), H13 (an interrupted batch exits 0), H16
  (`load` in `interact` changes the question; HD4: refuse another
  sequent or mode), H5 (HD2: a `.spec` file is decided in affine mode,
  and a mode flag that contradicts it is refused), then the probe's H26,
  H28, H30 and H31, each with the test that would have caught it and
  each a commit of its own; the rest of the command's findings stay with
  area 3.4. (2) Then area 3.2, the search, as the step prompt's stage 3
  and the design say. The session ends with area 3.2's check rounds
  (three at most), the fresh-context reviewer and a passing `nix flake
  check`, all committed, the checklist current and the last message sent
  to `planning`.
- The findings: the command's above; `area == "search"` in
  `plan/reports/28-audit-findings.json`, among them H18 (the pool's
  stack under a raised recursion limit), H21 (the ordinary translations
  compared over generated sequents) and H27 (the library batch hanging
  after a worker panics); H1 is done (orszuxqx) and H21 stays this
  session's. What area 3.1 moved here: `Statistics::memo_entries` as a
  `u64` and F56's private engine names; the search's halves of H27 and
  F139; the shim `without_progress` removed in one measured commit, with
  `Limits::work` counted at the engines' polls and `Reason::WorkLimit`;
  the forced-engine refusals folded into `EngineRefused` with
  `NotTaken`; `search::race` with `Clock` (F103, F104); the stack bound
  per worker (H18); lock commit (4) of design §7.5, `work` and
  `forward_copies` in `statistics`.
- The oracles under LTO: `bench/targets/head-lto.csv` for the focused
  engine (`bench/targets.sh`, cores 2 and 3) and
  `bench/targets/net-after-retype.csv` for the net engine
  (`bench/net-targets.sh`). A commit that claims no change of the
  search keeps their counters; one that changes it is reviewed by the
  panel the prompt names.
- Benchmarks are allowed while the author leaves the machine free, one
  line to `planning` when a run starts and one when it ends; "Pause" or
  "hold" stops runs gracefully. Cores 2 to 15 only, queue rather than
  take 0 or 1. Wait on a run with a background watcher. Signing: a loop
  keeps the cache warm until about 11:20, then commit with
  `--config signing.behavior=drop` while a signature fails. The weekly
  quota is near 62 %, and the author's failsafe pauses the step at 80 %.

### Message from `planning` to `step-28e` (2026-10-10, morning)

- Two additions from the author, after the command's wrong answers: (1)
  seal `Owner` and `wire::Readable` as the signed-off design has them,
  with `#[expect(unnameable_types, reason = "…")]` at the sealing
  supertrait, a commit of its own with its gate; (2) record the author's
  answers to area 3.1's departures from the design in its "decided
  unattended", each marked answered: errors without `source()` kept; the
  two traits sealed; the Unicode crates in every build kept; keywords
  refused as atom names kept; the JSON forms of `Step`, `Applicable`,
  `batch::Answer`, `ordinary::Outcome`, the ordinary sequent and
  derivation and the linear derivation waiting for the web client; the
  smaller ones (`prove_within`'s error without formula text,
  `batch::Options::plan` private, `Progress::item` for sessions only,
  `ProofStructure::occurrence` as `Option`, `Forest::from_owned` public,
  `ordinary::decide`'s refusal) kept. And from now on a choice that
  departs from the signed-off design (`plan/notes/api.md`, its §14
  answers, the rules' "Decisions") goes to `planning` as a question
  before it is built, with the recommendation, which the session follows
  meanwhile. What was done: acknowledged; both items queued after H31.

### Messages from `planning` to `step-28e` (2026-10-10, late morning)

- A departure from the signed-off design is no longer sent as a
  question: decide it, and mark it "(departs from the design)" in
  "decided unattended" with what the design said and why. What was
  done: the section "Area 3.2: decided unattended".
- Signatures work until about 13:45; test one before each commit, and
  commit with `--config signing.behavior=drop` when it fails.
- The failsafe is lifted: finish area 3.2 with its check rounds, the
  reviewer and `nix flake check`, then end; step 28 stops after this
  area.
- Benchmarks are allowed, with one line to `planning` when a run starts
  and when it ends; "pause" or "hold" stops runs gracefully.

### Message from `planning` to `step-28e` (2026-10-10, afternoon)

- The mutation run (cores 6 to 11) and the F122/F123 test agent (cores
  6 to 9) overlap: under contention a mutant's test run can time out,
  and a timeout counts as a survivor. Move the agent's runs to cores 12
  to 15 between the gates, or read every timed-out mutant in the
  results and rerun it alone before counting it. What was done: cores
  12 to 15 carry the second gate stream, so the agent stayed; every
  timed-out mutant is read and rerun alone before it counts.

### Where you start, `step-28f` (2026-10-13)

- You are `step-28f`, the sixth session of step 28, Opus 5.5 at
  `xhigh`: area 3.3, efficiency, as the step prompt's stage 3 item 3
  says. The step resumes after its pause at area 3.2's review on the
  author's word, under a new supervisor: `planning` and this session
  run on the author's second account. The session ends with the area's
  check rounds (three at most), a fresh-context reviewer and a passing
  `nix flake check`, all committed, the checklist current and the last
  message sent to `planning`.
- Read the step prompt's "From the review" sections, area 3.2's and
  "From area 3.2, for area 3.3" last, and "Ultra reviews" (this area's
  diff is run 2's slice, of any size).
- The first measurement: the pool's speed after H18 and F92, which
  nothing measured: the focused engine's target set and the second
  baseline's parallel pass at `-j 2` and `-j 4`, at the head and at
  kktkswpq, times and nodes side by side; before the first timing, the
  timed validation of counts against time again
  (`bench/counts-against-time.py`). The quiet window is now: the author
  has left the machine to the step until they say stop. Timed runs
  pinned on core 2 or 3, nothing heavy beside them, no timer due during
  them (`nix-optimise` at 04:03).
- The findings: the eight `perf-*` (F147 to F154); from area 3.1's
  review read-json and the printing journeys' one printer through a
  `dyn` atom writer (Typst +2.9 %, LaTeX +2.5 %, SVG +3.3 %); from area
  3.2 F114 and F121, which wait for the stack per level measured on
  quiet cores, before and after. Link-time optimisation is on and the
  ceilings were recorded under it. A gain that costs more code than it
  is worth goes to `planning` as a decision.
- The oracles under LTO: `bench/targets/head-lto.csv` and
  `bench/targets/net-after-retype.csv`; a commit that claims no change
  of the search keeps their counters, one that changes it is reviewed by
  the panel.
- Benchmarks, callgrind, mutants, fuzzing and anything timed are
  allowed now, one line to `planning` when a run starts and one when it
  ends; "Pause" or "hold" stops runs gracefully. Cores 0 and 1 are the
  desktop's; builds on 2 to 5; agents' programs and long runs on 6 to
  15; every unit and scope named `step28-…`. Wait on a run with a
  background watcher. Mutation runs from a jj workspace of their own at
  the head; a mutant that timed out is rerun alone on quiet cores before
  it counts. Departures from the design are decided and marked "(departs
  from the design)" in "Area 3.3: decided unattended". Signing: test a
  signature before each commit, `--config signing.behavior=drop` while
  it fails. The account is on Max 5x: keep the checklist current, spend
  agents where they buy evidence.

### Messages from `planning` to `step-28f` (2026-10-13, night)

- 00:27: signing is not available yet (the machine rebooted); commit
  with `--config signing.behavior=drop` meanwhile. 00:31: signing works
  until about 02:25 (the passphrase entered at 00:28, the warm loop
  `step28-gpg-warm` running); test a signature before each commit, and
  after 02:25 commit unsigned. What was done: each commit tests a
  signature first.
- About 02:08: hold builds until `step28-pool` ends; until then nothing
  but the light stack probes on CPUs 12–15, no cargo, no nix. Say in the
  report which pool pairs overlapped the compile (by time); a
  difference between the head and kktkswpq within a few percent in
  those pairs is rerun alone before it counts. What was done: row b
  names the two pairs and their overlaps; both were rerun alone
  (`step28-pool-rerun`), and the Mix difference that stayed was traced
  to the split search's poll (row n).
- About 05:25: the mutation run started from the main checkout, not a
  workspace of its own; leave the code in the working copy alone until
  it ends (plan and report edits are fine, a code fix in a workspace),
  name the commit it tested and that it ran from the main checkout, and
  rerun a timed-out mutant alone before it counts. What was done: no
  code was touched until it ended; "Area 3.3: decided unattended" names
  the commit; the four that timed out were rerun one at a time on cores
  6 to 11 (row m: three caught by a failing test, one surviving).

### Where you start, `step-28g` (2026-10-13)

- You are `step-28g`, the seventh session of step 28, Opus 5.5 at
  `high`: area 3.4, the command, the harness, the flake and the
  documents, the step's last area. The session ends with the area's
  check rounds (three at most), in which the last area takes all of
  stage 0's evidence again (every mutation batch, every fuzz target, the
  ratchet); the fresh-context reviewer; the last reader of stage 4
  (Fable 5.1 at `high`, fresh context: the rustdoc's front page and
  README as a stranger reads them, against
  `plan/notes/research/usability-baseline.md`); a passing `nix flake
  check`; all committed, the checklist current, the last message sent.
- The findings: `area == "command-harness-flake-docs"` still open (the
  command's wrong answers were area 3.2's session's); from area 3.1
  F5, F16, F27, H25 and F65; from area 3.2 the `--stats` lines through
  `Engine::counters`, the engine arguments from `Engine::ALL`, the
  harness's reason column through `Reason::name` and F141's command
  half; `--settings FILE` (R183); F160 and F181 with the oracles
  `bench/targets/after-efficiency.csv` and
  `bench/targets/net-after-efficiency.csv`; F59's moves into the files
  the step prompt names; a test that kills the race's `||` for `&&` at
  `search/mod.rs:436`, and why the reruns' `--re` took twelve mutants
  it does not match; the ultra reviews' `U…` findings if they come (a
  commit of their own where the code is otherwise untouched).
- Benchmarks are allowed, one line to `planning` at a run's start and
  end; "Pause" or "hold" stops runs gracefully; a timed run alone on
  core 2 or 3, timers looked at first. Cores 0 and 1 the desktop's,
  builds on 2 to 5, agents' programs and long runs on 6 to 15, every
  scope `step28-…`, a background watcher on every run. Mutation runs
  from a jj workspace of their own at the head; a timed-out mutant
  rerun alone before it counts. Departures from the design marked
  "(departs from the design)" in "Area 3.4: decided unattended". Signing
  has lapsed: commit with `--config signing.behavior=drop` until
  `planning` says it works, testing a signature before each commit.
  Leftover workspaces `../linlog-pool-base`, `../linlog-round1` (area
  3.3's) and `../linlog-gate` (live, usable) stay; the last message
  lists the ones added.

### Messages from `planning` to `step-28g` (2026-10-13, morning)

- About 08:15: hold the full mutation run until the code it covers is
  final (this area's library halves F5, F16, F27, H25 and F65, the F59
  moves and any `U` fixes); before that, run only batches whose files
  the area will not touch, from a jj workspace of its own; the early fuzz
  run is fine for finding faults, and a reader that changes is fuzzed
  again at the end. What was done: the area's library halves landed in
  the command (no file of the mutation scope changed but by doc links);
  the full run waits for the ultra reviews' `U` findings or the check
  rounds, whichever comes first.
- 08:41: signing works until about 10:40 (the passphrase entered, the
  warm loop running); test a signature before each commit, unsigned
  after. What was done: commits from 09:00 test a signature first
  (scratch `jjc.sh`).

### Messages from `planning` to `step-28g` (2026-10-13, late morning)

- About 09:40: the ultra reviews are done; their one finding is U1
  (`plan/reports/28-audit-findings.json`): under `--isolate` an entry
  whose name or path starts with `-` got an error. What was done: the
  hidden `--entry-name` and `--entry-file` take hyphen values, and
  `isolated_entries_named_like_flags` pins it (it fails without the
  fix); in vxwmzrmx. batch.rs's mutation batch had not run, so it runs
  with the rest.
- About 09:45: Pause. What was stopped, where, and how each resumes:
  - The gate batch (`step28-gates3`, cores 2 to 5, `$S/gates.sh` in
    `../linlog-gate`): lswsnsqs to porxkxso passed; stopped in
    toxvsokk. Resume: gate toxvsokk through mslrmklq, then rwtxwmxv,
    wxpuwxvv, vxwmzrmx and whatever follows (ptpppvro and xnturszr are
    plan-only).
  - The full mutation run (`linlog-mutants`, cores 6 to 11, from
    `../linlog-mutants34` at mslrmklq): the check batch's first pass had
    ended (292 mutants, 19 missed), its survivors' pass was in flight
    and is lost; nothing was written under `mutants/area-3.4/`. Resume:
    the code it covers changed since (the parsers in wxpuwxvv), so move
    the workspace to the head (`jj edit` there onto the last commit) and
    run `mutants/run.sh --label area-3.4` from the start.
  - The fuzz run (`linlog-fuzz`, cores 12 to 15): json_sequent,
    json_session, json_structure and json_proof ended at the cap (an
    hour each, no finds); json_disproof, ordinary_text, sequent_text
    and tptp were stopped with their corpora kept; session_steps,
    proof_views, ordinary_decide and the rest had not started. Resume:
    `fuzz/run.sh` for every target but the four done; sequent_text,
    ordinary_text and tptp must run anyway, their readers having
    changed (wxpuwxvv).
  - The signing loop `step28-gpg-warm` is stopped too; start it again
    if a signature succeeds.
  - Round 1's fixes not yet made (scratch `round1/todo.md`): `prove.rs`
    split into `show.rs` and `check.rs` (pure moves); a test that every
    id `Apply::set` asks is an argument of its command; the valgrind
    predicate written once; one helper for the `STEPS_PER_CLOCK`
    closure; interact's `proof_format` error for `show` and the terminal
    refusal written once; the `linlog` path dependency in
    `[workspace.dependencies]`; `summary.rs`'s module doc; README's
    "where the library refused it" (most refusals reach a record as
    prose); the record's `seconds` and `version` by string surgery;
    `proof_views` through `wire::upgrade`; interact's `--file` doc
    (`-`); the departures to record in "Area 3.4: decided unattended"
    (`--jobs` sets two keys, `"verdict":"error"` against api.md §7.3
    and §7.4); and as declined or recorded: F169's residue (interact's
    `show --png` and `seq print`'s PNG are bounded by the derivation
    limit, not the memory limit), F182's rerun list (the first
    baseline's on purpose), F183's resume key, F214's `--isolate`
    failure arms, F217, F218's scaling tests, the input lens's taste
    items. Then check round 2, the reviewer, the last reader, the
    evidence again and `nix flake check`, as "Where you start" says.
  - Not yet run on the last three commits: the gate; `cargo test
    --workspace` passed before the help lock's rebless and the U1,
    settings and derivation tests, which passed alone with the unit
    tests and clippy.

### Message from `planning` to `step-28g` (2026-10-14, 00:48): Resume

- Go on from the pause entry; usage near its limit, so start the runs
  first. Signatures work until about 02:49. What was done, 00:50 to
  01:00: `step28-gates4` gates the 24 commits from toxvsokk to vxwmzrmx
  (log in scratch `gates/summary`, under "resume 2026-10-14");
  `linlog-mutants` runs every batch again from `../linlog-mutants34`
  at rrrkklkp (the mutation scope is core files only, which no fix
  left to make touches); `linlog-fuzz` runs the nine targets not done,
  built at oxvxmwko, where `proof_views` reads through `wire::upgrade`
  as `check` does. If paused again: the gate resumes at the first
  commit without a line in the summary, the mutation run with
  `mutants/run.sh --label area-3.4` (it skips the batches done), the
  fuzz run with `fuzz/run.sh` (it skips the targets done).


### Where you start, `step-28h` (2026-10-14)

- `step-28h` is the eighth and last session of step 28 (Opus 5.5 at
  `high`): the step's closing write-up. Every area is done and reviewed;
  the code is not this session's to change, apart from one README
  sentence. The session ends with all committed, the checklist current
  (a row "closing: the report's sections and the register" in the Stage
  3 and 4 table) and the last message sent to `planning`.
- The register (`plan/notes/requirements.md`, 254 entries): every entry
  gets a line "State after step 28:", met (a file and line, or a test,
  that satisfies its "Met when"), partly or not met (what is missing,
  its reason, and the step or the `plan/later.md` entry that takes it);
  "State now" stays as stage 0's record; the summary table gets both
  columns. One read-only agent per kind (13, Sonnet 5.5 at `high`), the
  results merged and a sample of "met" claims spot-checked.
- The report's closing sections, before "From the supervisor", as the
  Deliverables list them, written from what the report, the review
  sections, `plan/notes/api.md` and the code hold, adding nothing new:
  the surface before and after; a searchable table of what was renamed
  or removed, checked against the code; the design and the first-order
  plan in a page; the journeys' ceilings at stage 0 and now; the check
  rounds, a few lines per area; the decisions and every area's
  departures from the design in one list; the open questions; what
  steps 29 to 31 must know, from their prompts.
- One README sentence: `check`'s time limit is two seconds unless
  `--timeout` says otherwise, like `prove`'s; README's test before the
  commit.
- The mutants' timeouts: `step28-timeouts` (area 3.4's rerun of the 24
  timed-out mutants, cores 12 to 15) may still run; when it ends, its
  outcome goes into area 3.4's row k (the survivor counts it lowers, the
  mutants that hang alone). Not restarted; waited on with a background
  watcher.
- At the end, `nix flake check` once, capped and detached.
- No benchmarks and no timed runs. Builds on cores 2 to 5, agents'
  programs on 6 to 11. No departures expected: a finding that would
  change code goes to `planning` in the last message, not into the tree.
- Signing has lapsed: commits with `--config signing.behavior=drop`
  after a failed test signature; the supervisor signs later.

### Message from `planning` to `step-28h` (2026-10-14)

- Fix the `replay.rs` warning in this session, a commit each: (1) gate
  `replay.rs` on `serialize`, verified by the gate and both `cargo hack`
  runs without a warning; (2) if both runs then pass with warnings
  denied, deny them in the gate and in the flake's `features` check, and
  if anything else warns, leave this item and say what warned; (3) open
  question 7's three doc points (`Atom`'s doc comment, `cli/src/lib.rs`'s
  sentence that `linlog_cli` makes no semver promise, `cli.md`'s
  `alone_first`) in one commit with the gate. R245 stays with step 29,
  whose prompt now says to do it first. Then `nix flake check` again,
  this row and open questions 7 and 8 brought up to date, and the last
  message once the timeouts' rerun has ended. What was done: (1) rsvlqous,
  the gate passed and both runs passed with `-D warnings`; the rest in the
  commits after it and in the closing row.

### Where you start, `step-28i` (2026-10-14)

- `step-28i` is a follow-up session of step 28 (Opus 5.5 at `xhigh`):
  F122 and F123, and the two sequents they found. The author answered
  open question 10 on 2026-10-14: a session looks at the engines on both
  sequents first, then lands both generators. The session ends with
  every code commit gated, the target sets after any change of how a
  search searches, a fresh-context reviewer (Opus 5.5 at `high`) of the
  whole diff, the mutants of the files it changed run from a jj
  workspace of its own, a passing `nix flake check`, all committed, a
  row of its own in the Stage 3 and 4 table ("follow-up: F122, F123 and
  the engines on their sequents") and its last message sent to
  `planning`.
- The work so far is off the line, signed, under the local bookmark
  `wip/generators`: supyqoln (F122's real contraction, finished) and
  ukrmvowr (F123's transposition, not finished), both on the tree of
  2026-10-10. Their content goes onto the head; the bookmark is deleted
  once both have landed or been replaced, and `plan/later.md`'s entry
  says so.
- 1. The engines on the two sequents first, on the head's release build
  (both undecided after 60 s): F122's sample, said to be provable by
  construction, `b, b, (!!!c * !b), (((b -o !b) -o (b -o (b * b))) -o
  !((b -o c) * (a -o c))) |- ((((((b -o c) -o c) * b) * a) * b) * (b -o
  b))` (two-sided engine to copy bound 7, focused engine to 4 with 3.1
  billion splits); F123's Mix mutant `|- ?((((!~a par c) + c) par a) par
  ~c) par ?((~c * ~c) * c)` with `--mix` (copy bound 4 does not finish:
  787 million stable sequents visited, 741 million from the memo, which
  holds at most 1 731 entries). For each, a fix of the engine, a change
  of the copy-bound schedule or of a default, or an argument with
  measurements that it is inherent at that bound. A fix that changes how
  a search searches gets the step's evidence for search changes (both
  target sets against `after-efficiency.csv` and
  `net-after-efficiency.csv` with `summary --against … --counters`, the
  families, qcover if `search/horn/` changes, step 26's panel where a
  verdict could move); a gain that costs more than it is worth goes to
  `planning` as a decision.
- 2. Then the generators: F122's contraction landed and F123's
  transposition finished, each drawn by the callers the findings name;
  neither may double the debug test suite (a slow sample bounded
  explicitly, with the reason in a comment); the effect pinned
  (effective contractions per rule set, the share of mutants that reach
  a search); each disagreement between engines or with the reference
  prover recorded as a finding.
- 3. The third ultra review's findings (`review/3.4a`, `review/3.4b`)
  reach the session from `planning` as `U…`; each is a fix, a commit of
  its own, gated.
- Benchmarks are allowed (a line to `planning` when a run starts and
  ends); a timed run pinned on core 2 or 3 with nothing heavy beside it;
  probes of the two sequents capped in memory and time. Cores 0 and 1
  are the desktop's, builds on 2 to 5, agents' programs and long runs on
  6 to 15, every scope named `step28-…`. Wait on a run with a background
  watcher. Mutation runs from a jj workspace of their own; a mutant that
  timed out is rerun alone before it counts. Departures from the design
  are marked in a "decided unattended" list. Signing works until about
  10:45; commit with `--config signing.behavior=drop` once it fails.

### Messages from `planning` to `step-28i` (2026-10-14)

- **About 10:07, the third ultra review's findings**, U2 to U7 in
  `28-audit-findings.json` (uvlvuuxy): each a fix, a commit of its own,
  gated. Done: U2 needed none (the head probes only a regular file or a
  missing path since vxwmzrmx and pokmmrxy, with the test
  `an_output_pipe_gets_the_output`); U3 and U4 vzxtsvrx (the file's
  `ordinary.logic` is not read, said in README and `cli.md`, pinned with
  the translation the file gives; every key a flag sets pinned against
  the file on each command), U5 oxlvywnp (a counter's label for affine
  mode), U6 mlkkmqmu (the guards skip a global option's value), U7
  xwoononu (`--occurrence-limit` advised only where that limit
  refused).
- **About 10:20**: correct open question 10 and `plan/later.md`, which
  called F122's sample provable by construction (done in pxsxlnrl:
  the shift mutant of draw 32 of rule set 6 in `generated_ill`); make
  the argument of the waiting failures cover a reuse in another branch
  under the same open ancestor and the memoization at its failure, and
  run an old-against-new differential besides the panel (both done:
  waiting.rs's module doc, and 3 × 13 568 runs with no verdict changed);
  the sign-aware count only if as clean, else a decision (sent as one).
- **About 10:35, the author's answers at the step's end**: count the
  `&` rules in `Limits::work` (open question 3) with the slices measured
  again, and keep the proof of `&` rules closed by `⊤` to the memory
  bound and the stop (open question 5), both this session's; F148 and
  F188's rest are a later session's. Done: yqsorumv and vvwkkuzx.
- **About 10:50**: time mix/10 old against new with nothing else
  running, the builds before or after it, the panel and the
  differential only after the timing (done in that order).
- **12:50, Resume** after the five-hour limit (the session had stopped at
  12:06): rerun the two lenses that failed with HTTP 429, split the
  working copy's work into gated commits with the target sets beside the
  commit they measure, bring the row up to date. Done: the lenses
  restarted at 13:13 from their scratch directories, the row in ronwsunq,
  the target sets in pnmsymur, items 3 and 5 split into yqsorumv and
  vvwkkuzx with render-svg's ceiling (vnkywvnv) between.
- **About 13:20**: record the count of one sign as open question 11 and
  in `plan/later.md`, in a plan commit of its own (done: yqxqsozw).
- **Afternoon, the author's answer to open question 11** (kqxqrkut):
  build the count of one sign in this session, after the evidence of
  questions 3 and 5 and before the panel's rest, the fresh-context
  reviewer, the mutants and `nix flake check`, which all cover it; the
  evidence of any search change (both target sets with `--counters`,
  every differing row explained, the families, qcover, the panel); its
  reach on LLTP with times before and after; its JSON form by the wire
  rules, the lock's fixture in a commit of its own and the fuzz seed in
  step; question 11 and `plan/later.md` marked built; a word to
  `planning` if it costs more than a few hours or reaches nothing on
  LLTP. The register's leftovers (R87, R138, `Node::members`' `_` arm)
  are F148's session's.
- **Afternoon, the count's soundness evidence**: a counterexamples lens
  on sequents built for it (one sign below `!`/`?`, with `&`, `⊕` and
  `⊤` beside), each refutation against the reference prover and the
  head's binary without it, and how many items reach it; every LLTP
  refutation against the head and the library's status; in the tests,
  where it refutes a generated sample, a bounded search without it
  finds no proof.
- **About 14:40**: the reach noted (the six refutations against a
  "provable" status are the known headers, `plan/notes/lltp-headers.md`
  items 10 to 15); still owed: the lens on the count, the three LLTP
  rows timed alone, the mutants of `search` and `horn-mod` after the
  reviewer's four fixes.
- **18:10, Resume** after the five-hour limit (the session had stopped
  at about 15:49): compare the mutants with the baseline, rerun the
  timeouts alone, check the focused engine's `--in-diff` mutants, time
  the three LLTP rows alone, `nix flake check`, the row, the last
  message. Done in that order, the timings first while the machine was
  quiet; the three agents the limit had stopped (the count's second
  reviewer, its argument and integers lenses) resumed after them.
