# Step 28, stage 2: the design

One session (`step-28c`, Opus 5.5 at `xhigh`) on 2026-10-09, unattended
and supervised by the planning session. It wrote `plan/notes/api.md`,
the public surface after step 28, its data model and its wire forms, and
how each later step enters it, from three drafts and two independent
judgements, walked it through against every later step and measured the
quantifier spike. The step's checklist is in
`plan/reports/28-audit-and-refactor.md`; the drafts, the briefs, the
judgements, the walk-through's reports and the spike's report are in
`plan/notes/api-drafts/`.

## Outcome

- **`plan/notes/api.md` is written, judged, synthesised, walked through
  and measured** (206 KB). It fixes the public surface after step 28
  (marked **[28]**), its data model and its wire forms, and the place each
  later step fills (marked **[N]**), so that none of 29 to 38 needs a
  breaking change after 0.1.0; section 10.10 is the first-order plan,
  whose first step is the data model alone with its refusal everywhere.
- **What it decides, in brief**: a sequent keeps its written order and
  sides (C1, H9, H10); an atom is an interned atomic formula, so ground
  first-order input is propositional by construction (decision 1);
  every member of a sequent is one `u32`, the occurrence below the
  forest's length and an instance past it; `Term`, `Kind`, `Node`,
  `Rule` stay closed with planned bumps; a proof records its goal and
  mode, an unprovable verdict carries a `Disproof`; the checker is
  specified rule by rule for the Rocq library; one error family with
  seven kinds and a refusal variant in every type; one `Limits` value
  and a stop that is told the work done, on every long call; options as
  plain data with `Settings` and `Clock`; one global wire level, the
  lowest a reader needs; the engine interface and the dispatch as data
  as step 26 shaped them; the focused engine generic over its zone at
  step 38 (D-7).
- **The walk-through** found 2 blocking items, 70 friction items and 59
  notes across the ten later steps; every one is answered in api.md
  section 12, most by a change of the design (the two blocking ones:
  `NetError`'s variants with named fields so step 33 can name a box,
  and one distinct key per atom so that `p(a)` and `p(b)` never share a
  name in an export or a translator).
- **The spike** (api.md 11.5) measured the drafts' data model as costing
  6.6 % on the focused engine's search journeys (two `Term` variants
  appended among the literals turn the hot literal test into a bit test),
  this design's data model (M1d) as free (G1 exact on 225 decided rows,
  search journeys −0.94 % in sum, worst +0.03 %), the generic zone with
  one instance as free (M2d) and with a second, first-order instance as
  within the gates once the shared helpers are inlined (M3i: worst search
  journey +0.94 %, sum −0.53 %; `mix/8` under callgrind −0.35 %). A rule
  follows for step 28: `Term`'s and `Kind`'s literal variants stay first.
- **For the author**: api.md section 14, the open questions (C1 to C3,
  T1 to T7, HD1 to HD5) on their provisional answers with what each other
  answer changes, and twenty decisions this design adds; those that
  matter most are 1 (the interned atom, which departs from the research's
  recommendation), 2 (the written sides refuse intuitionistic text written
  one-sided with several roots), 3 (the stop's form), 18 (the closed core
  enums) and 19 (classical cyclic MLL read with a reversing dual).

## What the fix sessions take

| area | from api.md |
|---|---|
| 3.1 the library's API, data model and wire forms | sections 2 to 7 and 9 (the renames, `Member`, `Mode`, the written order and sides, `Proof { goal, mode }`, `Disproof`, the checker's `CheckError`, `Rule`/`Named`, `Interactive`'s signatures, `ProofStructure`'s `VertexId` and `Criterion`, the errors, `Limits` and the stop at every long call outside the search, the options and `Settings`, the wire level and the three lock commits of 7.5, the exports' one `write`), the net-engine counter list before the retype (3.11), the literal-variant rule (3.2) |
| 3.2 the search | section 8 (the front door's stages, `Goal`, `Engine::{ALL, name, counters}`, `NotTaken`, `Statistics`), 5.2 to 5.4 (the progress stop at every poll site in a measured commit of its own, `Schedule`, `search::race` with one account, H18's per-thread depth, fallible reservation), `NET_MULTIPLICITY` private |
| 3.3 efficiency | nothing beyond the gates; the spike's measurements are its starting evidence |
| 3.4 the command, the harness, the flake and the documents | 6.6 (flags onto `Settings`' keys, `--settings`, `--work-limit`, `--schedule`, `--test-period`), the exit statuses by kind (4.1), the harness's one table of mode words and `Engine::counters()`, README's changed examples (the intuitionistic refusals of 3.6, the JSON forms of 7.5) |

## How the stage ran

| part | agents | model, effort | time |
|---|---|---|---|
| three drafts (A: the web client and the wire forms; B: the proof term, the checker and Rocq; C: engines, calculi and quantifiers) | 3 | Opus 5.5, `xhigh` | 06:55 to 07:37 |
| two judges, independent, scoring each draft's thirteen sections against the register and the rubric, with two added questions | 2 | Fable 5.1 `high`, Opus 5.5 `high` | 07:37 to 08:00 |
| the synthesis | the session | Opus 5.5, `xhigh` | 08:00 to 08:14 |
| the walk-through, one agent per step 29 to 38 | 10 | Sonnet 5.5, `high` | 08:16 to 08:26 |
| answering the walk-through in the design | the session | Opus 5.5, `xhigh` | 08:26 to 08:55 |
| the spike, in the jj workspace `spike` | 1 | Opus 5.5, `xhigh` | 07:31 to 09:00, paused; finished by the session 10:20 to 10:55 |

- **The judges agreed on the synthesis** though not on the ranking (Opus:
  A 56, C 52, B 49 of 65; Fable: A 56, B 56, C 54, ranked B, A, C): A's
  frame and tables, B's trusted core (members in every node, a proof
  that records its goal and mode, the checker specified rule by rule,
  `CheckError` split into fault and refusal, `VertexId` and `Criterion`,
  `Disproof`, `Rule` and `Named`, the Rocq design), C's engine interface,
  later steps and spike. Both found that only C's rule for the written
  sides refuses both H9 and H10, and both chose the interned atomic
  formula over the drafts' `Pred`/`DualPred` (the design's decision 1).
- **Where the judges split, the synthesis chose** (api.md §14.2): the
  stop as a closure over `Progress` (Opus, on a probe showing the trait's
  wrapper does not infer; Fable preferred the trait); one global wire
  level (Opus; Fable found it equally defensible to a version per form);
  `u64::MAX` for a saturated count (Fable, whose probe refuted draft A's
  reason for 2⁵³), with every integer read back below 2⁵³ (A's concern for
  JavaScript); `Drawable` for the exports (Opus; Fable called it taste).
- **The two witnesses** of H9 and H10's mirror were run on the release
  binary of the tree of 2026-10-09 (cores 2 to 5, capped): `linlog prove
  -i --deterministic '|- top, a'` and `'(A -o bot) -o bot |- A'` both
  answer `provable`, exit 0 (the first as `0 ⊢ a`, the second as `1 ⊸ (A ⊗
  1) ⊢ A`); api.md §3.6 names them as the tests of the change.

## Decided unattended

- **The drafts' angles** are the prompt's three; each draft designed the
  whole surface, most thoroughly on its angle, under one brief and one
  outline of thirteen sections, so the judges could compare section by
  section. Set aside: drafts of only their angle, which would have left
  the synthesis without competing answers on the shared parts.
- **The judges answered two questions of the supervisor's** beside their
  scoring: the interned atomic formula, which no draft proposed but
  `impact-quantifiers.md` §1 names as the other way out, and whether each
  draft's rule refuses H9's and H10's witnesses. Both answers changed the
  design.
- **The spike started before the synthesis**, from draft C's
  specification, to save its build and measurement time; it measures the
  drafts' four reserved `Term` variants, a superset of the synthesis's
  two, so its bound holds for the design either way.
- **The synthesis follows the judges' common base** and decides each
  point they split on with a reason (api.md §14.2); every decision the
  author could take otherwise is in api.md §14 with what the other answer
  changes.
- **The drafts and the judgements are committed** under
  `plan/notes/api-drafts/` (the spelling check skips `plan/`), so the
  supervisor's review can check the synthesis against them.

## Deviations

- Draft B's agent compiled its two scratch probes on its cores (9 to 11)
  with `taskset` and time limits but outside a memory-capped scope, which
  its brief required. The probes were two small `rustc` builds; nothing
  came of it, and it is recorded here.
- The three drafts ran over the brief's 90 KB (93 to 98 KB), and the
  synthesis is 171 KB: it carries B's checker table, A's wire forms and
  C's later steps whole, which a fix session needs as written.
- The spike's agent was stopped at the pause while it switched to M3i;
  the session finished M3i (committed its prepared tree, built and tested
  it) and ran the last measurements after the supervisor's go. M3i's
  target set ran beside another workflow's builds (load 24.7): its CPU
  times are load, as a count of `mix/8` under callgrind confirmed; G3
  (pinned time) was never run, so no claim of the design rests on time.

## What is not verified

- The design's signatures were not compiled, except the data model's
  parts the spike built (`Member` in `Node`, the empty tables, the binder
  variants, the generic zone): the fix sessions find what a type checker
  says of the rest.
- The planar order derived from D1's lowering (10.8, the Lambek
  calculus) is a lemma nobody tested; step 36's panel decides.
- The synthesis was reviewed by no fresh context in this stage: the
  supervisor has it reviewed before the author's sign-off.

## What the stage cost

Tokens as each agent's completion reported them (input, cache and
output together):

| model | agents | tokens |
|---|--:|--:|
| Opus 5.5 (three drafts at `xhigh`, one judge at `high`, the spike's agent at `xhigh` before the pause) | 5 | 3 282 225 |
| Fable 5.1 (one judge at `high`) | 1 | 508 076 |
| Sonnet 5.5 (ten walk-throughs at `high`) | 10 | 2 546 622 |
| all | 16 | 6 336 923 |

The spike's agent's work after its resumption (M1d, M2d, M1b, M3i's
tree) is not in the table: it was stopped at the pause, which reports no
count.

