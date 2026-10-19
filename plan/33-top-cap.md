# Step 33: the focused engine's cap on the members only a `⊤` can take

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing. The step takes one
session; this prompt was written at the review of step 35 (2026-10-18).
It says what is fixed, and it begins with a measurement that can end it.

Read `plan/notes/research/comparison-gaps.md` (2.3 "Class B", R3, the
ranking, 6's probes), `plan/reports/30-baseline-release.md` (session
30b: R1, the cut entries read within their generation),
`plan/reports/31-rocq-library.md` ("Session 31d: certified
refutations": the held-out run and its command), `plan/reports/26-focused-engine.md`
(the copy heuristic, the memo key), `.claude/rules/core-focus.md`, and
`plan/README.md` (D8, D19).

## Goal

R3 of the gap study, adopted by the author on 2026-10-18: in linear
mode, a literal of a stable sequent's linear zone whose dual occurs
nowhere in that zone nor below a member of the unrestricted zone can
end only in a `⊤` leaf; keep at most one copy of each such literal in
the context and the memo key, so that a branch that only adds such
copies stops growing. The proof puts every dropped copy back into the
`⊤` leaf that takes the kept one, and the checker accepts it unchanged.

## What has changed since the gap study

- **Class B's own rows are decided without it.** The study's 19 rows
  of class B (`ILLTP-SYJ-cbn/SYJ210+1.*`, Schwichtenberg's formula, one
  stem) ended at the recursion limit at level 512. In the full
  comparison of 2026-10-18 (the author's `Comparison` run 39110897772,
  linlog 0.1.0, 10 s, one thread) all twenty `SYJ210+1` files of
  `ILLTP-SYJ-cbn` are refuted by linlog, as by Sympli: R1 (session
  30b) closed their levels. The cap's case now rests on the study's
  "Beyond" alone (every intuitionistic negation under the call-by-name
  translation is `!A ⊸ 0`, a `⊤` on the left in every use), which no
  one has measured.
- **What other tools still decide and linlog does not** (the same run):
  Sympli 17 rows, ten of them proofs (`ILLTP-SYJ-cbn/SYJ203+1.011` and
  `SYJ205+1.012` to `.020`, class C, step 37's case) and seven
  refutations (`SYJ208+1.004` in `-01` and `-cbn`, `SYJ210+1.007` in
  `-01`, `SYJ209+1.011` and `.012` in `-cbn`, `.008` and `.009` in
  `-cbv`), each where linlog reached the time limit; and apll-ill 31
  Petri nets of `petri-nets/MCC`, all "unprovable", unconfirmed (no
  other tool decides them, and apll-ill gave 1 306 refutations of
  problems another tool proved with a checked proof). linlog decides
  4 307 of the 4 811 problems, against 4 181 on 2026-10-15.
- **The classical refuter runs after every search** (step 31d): an
  undecided row whose classical reading is falsifiable is refuted
  already; the cap's gains are the rows it is not.

## What is fixed

1. **First, the measurement that can end the step.** Without changing
   the search, count on the undecided rows of the held-out sets
   (`KLE-01`, `KLE-cbn`, `KLE-cbv`, `KLE-IMP-CONJ`, `ILLTP-SYN` and
   `-LCL` in all three translations, `ILLTP-SYJ-cbv`: 850 problems, 129
   undecided at 10 s on one thread after step 31d), on the seven
   refutations above and on the families, how many stable sequents hold
   two or more copies of a literal that only a `⊤` can take, by the
   test the copy heuristic already makes (`mark_literals` and `meets`
   in `core/src/search/focus/`). A counter behind `#[cfg(test)]` or a
   scratch build is enough; nothing of it is committed. If no undecided
   row meets such a sequent, the cap cannot decide one: the report says
   so with the counts, nothing else is built, and the session ends. The
   probe runs on cores 2 and 3 under the machine's lock, about twenty
   minutes, and needs no further asking.
2. **Otherwise, the cap**, in the focused engine (classical and
   two-sided, linear mode only: weakening and Mix change which literals
   only a `⊤` can take, so affine and Mix modes keep today's search
   unless the argument covers them), with its argument (why a proof of
   the capped sequent gives one of the uncapped, across `&` in both
   branches, and why a cut memo entry of the capped key is sound for
   every uncapped sequent it stands for) and the proof's reconstruction.
   The panel of step 26's prompt reviews it on a frozen tree.
3. **Measured as the study says**: the target set
   (`bench/targets.sh LABEL`, against `bench/targets/after-efficiency.csv`;
   on rows the cap does not touch, every counter equal, and the five
   rows `bench/TARGETS.md` records for `after-routing` and the five
   that `after-classical` decides as there), the held-out sets before
   and after, the seven refutations, and the reference prover's
   differential tests. A row lost or a counter changed on an untouched
   row is a defect, not a cost. If the cap decides no new row, it is
   not merged: the report keeps the measurement and the argument.

## How the step runs

One session on Opus 5.5 at `high` (plan/README.md, "Why these models
and efforts", 2026-10-18), its goal in its launch prompt. The panel:
the argument on Fable 5.1 at `high`, the counterexamples on Opus 5.5 at
`high`, the integers and limits on Sonnet 5.5 at `high`, each a fresh
context on the frozen tree, briefed to report only what breaks
soundness, completeness or a stated requirement. When the measurement
of item 1 ends the step, there is no panel.

## The machine

If step 32 runs beside this one, this step's timed runs take cores 2
and 3 and its builds 4 to 7, step 32's builds 8 to 11; every timed run
and every `nix` build holds `/tmp/linlog-machine.lock`.

Deliverable: thematic jj commits (none if item 1 ends the step);
`plan/reports/33-top-cap.md`.
