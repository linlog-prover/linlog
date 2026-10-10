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
| b. H18 the pool's depth per worker, a test at a raised limit on two threads | done | slwktolx: a pool's `&` is a level of its forks (or_depth + 1), and a stolen task starts at its thread's depth (`Runtime::waiting`, `depth_here`); the witness aborted before (two threads, limit 3 000) and answers unknown at 3 000, 5 000 and 10 000 on two and four threads; test `a_raised_limit_holds_on_the_pool` (aborts on the old code in release, the flake's build). Changes the search on a pool only: for the panel |
| c. F92 a pool's `&` premises without copying the ancestors | done | slwktolx (with b): a spawn's branch is at most `LEVELS` + 1 slices; the witness (5 000 nested `&`, two threads, 64 MiB) 24 MB against 823 MB |
| d. `Reason` with named fields and `setting()`; `NotTaken` and `Error::EngineRefused`; `Engine::parallel` exhaustive (F141), `Engine::counters`; F140 no `expect` in the dispatch | done | tvvvvluq `Reason` (`RecursionLimit { depth }` on the wire, test `reasons_name_their_setting`); ssxwovxw `EngineRefused { engine, because: NotTaken }`, the internal `Pairs` and `Linker`, F141, F140 (`NoEngine`); lxkxvszu `Engine::counters` (`Counter`: key, label, meaning; test `counters_are_fields`) |
| e. `Goal` and the front door's stages (8.1), one place a verdict is built | done | mrvtkppt `Goal` (`Copy`, `conclusion`, `new` checking the members, `is_conclusion`), `prove_goal` and `engine_for` on it; nulpvyxn `conclude` |
| f. `Statistics`: `memo_entries` a `u64` (F102), `add` public, F144 the forward level apart from `copies` | done | tpnlumuk (`u64`, `add` public and the command's and harness's hand merges gone, `add_run` crate-private, test `statistics_add_up`); F144 in syqnnmno (item g) |
| g. the progress stop at every poll site, measured, the shim removed; `Limits::work` and `Reason::WorkLimit`; lock commit (4) with `work` and `forward_copies`; R243's test | done | txuvownw (`Work`, every poll told its units, `WorkLimit`, test `the_search_counts_its_work`; the additive path polls every 1 024 pairs, which took search-additive-14 from +5.7 % to −1.1 % against its ceiling, and tells the stop the pairs after its last poll when it ends, without which a search of fewer pairs reported no work and syqnnmno's lock and README failed); syqnnmno lock commit (4) (`work`, `forward_copies`, `copies` the decider's level; both locks, README); xwqpwrwo the calling thread's count apart and the polls' fast paths inline (the journeys' counts below their ceilings: additive −1.09 %, chain-128 −0.24 %); wtqxszqu a pool worker's and the second search's last units, lost before (the net engine on two threads reported 0 units against 128; `forced_links_are_made_once` and the new sum in `default_bias_takes_turns` catch it) |
| h. `search::race` with one account (F103, F104, F168), the command and the harness its callers | done | zoywtttw: `race` (one thread, a pool of `threads − 1` once `add_pool` says so, the first to decide answering), `Account::part_of` (both searches within one bound: 2.05 GB in one call before), `threads` counting every thread (`--jobs 2` runs two); the command's `searched` and the harness's `--pool-after` call it; test `the_race_counts_every_thread` |
| i. names inside (F56, F113) and the refactorings F107 to F121, F126, F132, F133, F135 | done, some in part (decided below) | zrxrxurw F113 and F56's engine names (`Run`, `Switches`, `Plan`, `Searched`; `Pairs` and `Linker` in ssxwovxw); ysrmnpss F133 (`Reach`, `Cover`, `FORWARD_PER_BACKWARD`); xxpxyrkt F107; mvzqmplu F108 (`Rank`); ptkmmtsy F109 (`prepare`, `SetUp`, `run_kept`: seven `too_many_arguments` gone); F110 by area 3.1's `Schedule` (tuooxrol); rurkzopt F111 in part (`Reason::as_set`, the count refutations in `counts`); lrmnkmvl F112; ymosnlvq F115 (`Forest::list`); nrswyxyn F116 (`lock`, `record`); mrxutqxo F117 (item a); lrynyvqu F118; vonxrkzw F119 (`Finished`); wxzwqtkz F120 in part (the batch's `Queue`); xzkwrqsm F126 in part (`Parents`); mqnqzzyk F132 (`is_head`, `clause_head`, `is_below`); mystyvxw F135 in part (`Program::inputs`, `outputs`); F114 and F121 decided below |
| j. tests: the Horn engine's (F94 to F97, F124, F125, F127, F129 to F131, F134), F98, F105, F106, F138, F143, F145, F122, F123, H21 | running: the Horn engine's and F122, F123 with agents in workspaces of their own | onxqtvpq and qwukxtlu F98 (`a_refused_check_is_unknown`, `a_rejected_proof_is_an_error`); ymkmoyno F105's additive and net halves (`stops_between_pairs`, the net's failed-test poll) and F106 (the stream answered question by question, the window; the panic in rqnnqtor); kumqpqyx F138; qwukxtlu and tpnlumuk F143 (the refusals, `occurrence_limit`, `Engine::parallel`, the defaults, every refutation's sentence, `statistics_add_up`); wkouswzu F145 (the two ignored timing tests dropped); qromqtpw H21 (each logic's translations against each other and a truth table on 300 generated sequents, written by an agent and reviewed here) |
| k. docs: F91, F99 to F101, F118, F137, F142, F146 | done | lrynyvqu: F91, F99, F100 (a recursion level), F101, F118 (the private docs' links), F137 (the Horn and additive engines in the front door's docs), F142 (the default bias's measurements under a time limit), F146 (the reference's refutations with exponentials) |
| l. the panel, the target sets, check rounds (at most three), the fresh-context reviewer, `nix flake check` | running | **The panel** (step 26's three lenses over the area's seven changes of the search, mrxutqxo..xwqpwrwo, the argument read again on Fable 5.1): counterexamples found none (2 812 classical and 674 intuitionistic generated items, 95 634 runs over 23 configurations each, against the committed reference and step 26's second one, and the base tree); the argument and the integers each refuted claim 7 with reproduced witnesses, both fixed in ysoxrlsz with a test each: the work bound ran a whole slice late beside the default bias's second search, and a turn's last poll lost its units (no verdict either way). **The target sets** at the head (rsrlxspt): the focused engine's 265 rows the same verdicts as `head-lto.csv` and the 225 decided rows equal in `nodes`, `splits`, `memo_hits` and `memo_entries`, 116.8 s of CPU against 118.5 s; the net engine's 64 rows equal to `net-after-retype.csv` in verdict, `links` and `tests`, 213.9 s against 238.2 s (the LTO build). The families at 5 s with the harness's new columns: 123 runs, no mismatch |

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
