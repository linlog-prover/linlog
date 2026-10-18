# Step 37: the focused inverse method

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing. The step takes two
sessions; this prompt is finished at the review of step 36, from the
third baseline's rows. It says what is fixed.

Read `plan/later.md` ("The focused inverse method"),
`proof-search-specifications.md` ("Alternative: focused inverse method"),
`plan/reports/17-assessment.md` (3.6), `26-focused-engine.md`,
`27-horn.md`, `30-baseline-release.md`, and `plan/README.md` (D8, D19).

## Goal

The spec's second engine for MALL and its semi-decision alternative with
exponentials: forward saturation from initial sequents in the subformula
closure, with subsumption and an index, as an engine behind the
interface of step 26, and a row of the dispatch wherever it is the
fastest (D19).

## Fixed now

The honest starting point (step 17): after steps 15, 26 and 27 two cases
are left for it, sequents with many hypotheses and a small goal that the
backward search still loses on, and whatever of Mix the prune of step 26
does not take. Its first session ends with a measurement on the third
baseline's undecided rows and the families; if no row is won, the engine
stays an option (`--engine inverse`) and the report says so.
Opus 5.5 at `high`; every change of a search or a criterion is reviewed by
the panel step 26's prompt describes, its argument read on Fable 5.1 at
`high`.

**Aimed by the comparison's gap study** (R4 of
`plan/notes/research/comparison-gaps.md`, adopted by the author on
2026-10-18). The premise above that the library ends at the copy bound
is out of date: outside the nets it ends at the time limit (the note's
2.4). Step 30b closed the levels of its class A (R1), which decided
class B's rows too (the comparison of 2026-10-18). Step 39 measured
R3's cap and built nothing (`plan/reports/39-top-cap.md`): not one of
the held-out sets' 129 undecided rows, of the six undecided rows among
the seven refutations Sympli has and linlog lacks, or of the families'
18 undecided rows meets a stable sequent the cap would change (4.5
billion stable sequents searched), so the undecided rows are not a
`⊤`'s copies but long proofs and refutations past the time limit. This
engine's case is class C's long proofs (in that comparison Sympli's ten
proofs that linlog lacks, `ILLTP-SYJ-cbn/SYJ203+1.011` and
`SYJ205+1.012` to `.020`) and a second refuter by saturation for what
R1 leaves of A and B, the six refutations among them. Its first
measurement is on the
held-out sets and the 384 intuitionistic rows no tool decides, at the
longer limit step 29 proposed, never on the 122 rows Sympli wins alone.

Deliverable: thematic jj commits; `plan/reports/37-inverse.md`.
