# Step 34: cut, and cut elimination

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing. The step takes three
sessions; this prompt is finished at the review of step 33. It says what
is fixed.

Read `plan/reports/17-assessment.md` (5.5), `plan/README.md` (D5, D6,
D6a, D13, D17), `plan/notes/api.md`, and the reports of steps 2, 5, 9,
31 and 33. The Rocq library (step 31) gains the cut rule as a case of
its datatype, its checker and its soundness proof.

## Goal

What a course on linear logic shows and the suite cannot yet: a proof
with cuts, and its cut-free form reached step by step. A cut rule in
interactive proofs (the user names the cut formula), proof terms with
cuts and the checker's rule for them, cut elimination on terms one step
at a time and to the end, and on MLL nets (and on the nets of step 33),
where it is the reason nets exist, each step drawn.

## Fixed now

The forest holds the sequent's subformulas only, so a cut formula and
its dual need roots of their own beside the sequent's: the first session
is the data model, the checker and the interactive rule, with the
propositional search untouched (every proof it returns stays cut-free)
and `plan/notes/api.md` followed for where quantifiers will go (D17).
The second is elimination on terms, the third on nets with the drawing.
Termination and the preservation of the conclusion are tested on
generated proofs with cuts, every result through the checker.
Opus 5.5 at `high`; every change of a search or a criterion is reviewed by
the panel step 26's prompt describes, its argument read on Fable 5.1 at
`high`.

## Known already (2026-10-19), for the review that finishes this prompt

- **The checker has ordered modes** (step 36): cyclic MLL and the
  Lambek calculus check the arcs at every `⊗` and, in L, a non-empty
  antecedent; a cut in those modes must keep the order (the cut formula
  between the two arcs) or be refused there, said so.
- **Refutations** carry four checked kinds (classical, unbalanced, the
  count equation, and since step 37 the saturated set, wire level 3);
  cut changes no refutation, since every one is about cut-free
  derivability and cut is admissible.
- **The nets** of step 33 (MELL with boxes) and the drawing of step 40
  are what cut elimination on nets draws, step by step.

Deliverable: thematic jj commits; `plan/reports/34-cut.md`.
