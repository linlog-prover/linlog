# Step 39: the focused engine's cap on the members only a `⊤` can take

## The measurement

**Outcome: the cap is not built.** No undecided row meets a stable
sequent that holds two or more copies of a literal only a `⊤` can take:
not one of the 129 undecided rows of the held-out sets (4 516 311 436
stable sequents searched), not one of the six of the seven refutations
that stay undecided (306 703 812 stable sequents), not one of the 18
undecided rows of the families (48 880 501 stable sequents). The cap
changes only such sequents, so it cannot decide any of these rows
(below, "Why this ends the step"), and by the step's item 1 nothing else
is built: no cap, no panel, no target set run.

**The probe.** A scratch copy of the tree at `xpvzuusk`, outside the jj
repository, with counters in `prove_stable` of
`core/src/search/focus/mod.rs` (the one function every stable sequent of
both focused engines passes, classical and two-sided, before the memo
and the loop check answer it), built in release mode; nothing of it is
committed. Its diff, the scripts and the raw outputs are in
`/tmp/linlog-handoff/39a-probe/` (`probe.diff`, `run.sh`,
`analyse.py`, the harness's CSV files and the probe's TSV files). For
every stable sequent it groups the literal members of `Γ`, with their
repeats, by atom and sign, and for every literal present two or more
times it tests two conditions:

- **Strict, the step's condition.** No occurrence of the dual literal
  lies at or below a member of `Γ` or a member of `Θ` (each occurrence
  of the dual in the forest's list of that literal, walked up through
  its ancestors). This is the condition under which the literal can end
  only in a `⊤` leaf in linear mode, and it is what a cap would test.
- **Loose, the copy heuristic's test as written.** The dual is not
  among the literal members of `Γ` (`mark_literals`) and lies below no
  member of `Θ` (`meets`). It is weaker than the strict condition: it
  misses a dual inside a compound positive member of `Γ`, such as the
  `~a` of a member `b ⊗ ~a`, which the focus on that member can still
  bring to the literal. It was counted only to show that the heuristic's
  test cannot serve as the cap's condition.

A sequent counts as meeting when some literal satisfies the condition.
The counters are cumulative per process and written at the end of each
search, so the last line of each child process of the harness is that
row's total. A check on `⊢ a, a, 1 ⊕ ⊤` before the run: one stable
sequent, meeting both conditions, at two copies.

**The runs.** One detached user unit, `step39a-probe`, under
`/tmp/linlog-machine.lock`, 2026-10-18 23:57:53 to 2026-10-19 00:10:09,
`linlog-bench run` on one thread per row: the held-out sets (`KLE-01`,
`KLE-cbn`, `KLE-cbv`, `KLE-IMP-CONJ`, `ILLTP-SYN` and `ILLTP-LCL` in the
translations `01`, `cbn` and `cbv`, `ILLTP-SYJ-cbv`) in two shards on
cores 2 and 3 with `--copies none --timeout 10`; then the seven
refutations on core 2 with the same flags; and the families on core 3
with `--all-families --timeout 5` (each problem's own copy bound).

| set | rows | undecided | stable sequents of the undecided | meeting on the undecided, strict | rows meeting, strict | rows meeting, loose |
|---|---|---|---|---|---|---|
| held-out sets | 850 | 129 | 4 516 311 436 | 0 | 12, all decided | 12, all decided |
| the seven refutations | 7 | 6 | 306 703 812 | 0 | 0 | 0 |
| families | 123 | 18 | 48 880 501 | 0 | 0 | 26 (Partition encodings) |

- **The held-out sets decide what they decided after session 31d**: 568
  proved, 153 unprovable, 129 unknown (101 at the time limit, 18 at the
  memory bound, 10 at the recursion limit), every row in the two-sided
  engine. The sum of the harness's `nodes` over the 129 undecided rows
  equals the probe's own count of their stable sequents
  (4 516 311 436), so the probe saw every stable sequent.
- **The twelve held-out rows that meet such a sequent are decided, and
  small**, each at two copies of the literal: `KLE-01/KLE018+1` (1 of
  355 stable sequents), `KLE-01/KLE069+1` (110 of 1 483),
  `KLE-01/KLE078+1` (199 of 2 394), `KLE-cbn/KLE018+1` (1 of 77),
  `KLE-cbn/KLE043+1` (2 of 82), `KLE-cbn/KLE067+1` (1 of 36),
  `KLE-cbn/KLE069+1` (10 of 266), `KLE-cbn/KLE071+1` (1 of 19),
  `KLE-cbv/KLE018+1` (1 of 205), `KLE-IMP-CONJ/KLE_26_CBN` (2 of 107),
  `KLE-IMP-CONJ/KLE_47_CBN` (1 of 36) and `ILLTP-SYJ-cbv/SYJ103+1` (5
  of 148). The loose test adds one sequent of `KLE-cbn/KLE043+1`.
- **The seven refutations**: `ILLTP-SYJ-01/SYJ210+1.007` was refuted in
  this run, in 5.39 s on one thread and 54 943 940 stable sequents
  (the comparison's run of 2026-10-18 had it at its limit of 10 s); the
  other six (`SYJ208+1.004` in `-01` and `-cbn`, `SYJ209+1.011` and
  `.012` in `-cbn`, `.008` and `.009` in `-cbv`) ended at the time
  limit. None of the seven meets such a sequent.
- **The families**: 61 proved, 44 unprovable, 18 unknown, none against
  its expected verdict. No row meets the strict condition. The loose
  test fires on 26 rows of the Partition and 3-Partition encodings
  (on nearly every stable sequent of `partition-yes` and
  `partition-no`), where the dual of a repeated literal lies inside a
  compound member of `Γ`: there a cap on the heuristic's test would
  drop copies a proof needs.

**Why this ends the step.** The cap changes the context and the key of
a stable sequent only where a literal satisfying the strict condition
is present twice or more; everywhere else the capped sequent is the
uncapped one. On one thread the search is deterministic, so a capped
search runs step for step as the uncapped one until it first meets
such a sequent, and a row whose uncapped search meets none within its
limits gets the same verdict and the same counters with the cap. That
holds for every undecided row above. One limit of the argument, stated
rather than measured: the probe's own pass over the members of each
stable sequent costs time, so under the clock the probe reached
somewhat fewer stable sequents per row than an unprobed search would in
10 s, and a sequent lying just beyond where the probe's clock stopped
is not seen. Nothing in the counts suggests one: no undecided row met a
single such sequent in more than four billion.

**What this says about the gap study's hypotheses.** Class B's own rows
(`SYJ210+1` in `-cbn`) are decided without the cap, as the step's
prompt records, and the probe finds no growing zone of such literals on
them either (`SYJ210+1.002` in `-cbn`: refuted in 74 stable sequents,
none meeting). The study's "Beyond", that call-by-name images of
negations `!A ⊸ 0` would meet such sequents whenever an atom's only
positive occurrence has been used, is not borne out where it was to be
measured: of the `cbn` sets only decided rows meet one, at two copies.
A reading of the two-sided engine, an inference not probed further: the
`0` of `!A ⊸ 0` is a `⊤` in the lowered sequent, which the asynchronous
phase closes as soon as the split of `⊸L` releases it, so the literals
sent to its side never stand beside it in a stable sequent; copies of a
literal pile up only where the `⊤` stays below a positive formula,
which is the narrow case the twelve decided rows show.

## The checklist

| item | state | evidence |
|---|---|---|
| 1. The measurement | done (2026-10-19) | The probe above, uncommitted, on the held-out sets, the seven refutations and the families: 0 meeting sequents on every undecided row (129, 6 and 18 rows); 12 decided held-out rows meet one; the raw outputs in `/tmp/linlog-handoff/39a-probe/`. |
| 2. The cap in linear mode, its argument and reconstruction | not built | Item 1 found no undecided row it could decide; the step's prompt ends the step there. |
| 3. The panel on a frozen tree | not run | Nothing was built for it to review. |
| 4. The target set, the held-out sets and the seven before and after | not run | No code changed: the target set and the counters are those of `xpvzuusk`; the held-out sets and the seven were run once, by the probe, above. |
| 5. Gated code commits and `nix flake check` | no code commit; flake check at the report's commit | The session's one commit is this report. `nix flake check` under the machine lock at that commit: its exit status is in the message to `planning`. |
| `.claude/rules/core-focus.md`, README and the changelog | unchanged | The cap did not land, so nothing they describe changed. |

## What is left open

- **R3 stays unbuilt.** Should a later change of the search (step 37's
  inverse method, or a change of the bias) bring undecided rows to
  such sequents, the probe's diff in `/tmp/linlog-handoff/39a-probe/`
  measures it again in about twelve minutes on two cores.
- **The copy heuristic's test is not a cap's test.** `meets` and
  `mark_literals` look at `Γ`'s literal members only, not at duals
  inside compound members, which is right for ranking copies, all they
  do. The gap study named them as the cap's test; a later cap must use
  the strict condition above, or it drops copies the Partition
  encodings' proofs need.
