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
stays an option (`--engine inverse`) and the report says so. Fable 5.1
at `xhigh`.

Deliverable: thematic jj commits; `plan/reports/37-inverse.md`.
