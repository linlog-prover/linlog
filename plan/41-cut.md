# Step 41: cut, and cut elimination

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing. The step takes four
sessions; this prompt is finished at the reviews of steps 39 and 40. It
says what is fixed.

Read `plan/reports/17-assessment.md` (5.5), `plan/README.md` (D5, D6,
D6a, D13, D17), `plan/notes/api.md`, and the reports of steps 2, 5, 9,
31 and 33. The Rocq library (step 31) gains the cut rule as a case of
its datatype, its checker and its soundness proof.

## Goal

What a course on linear logic shows and the suite cannot yet: a proof
with cuts, and its cut-free form reached step by step. A cut rule in
interactive proofs (the user names the cut formula), proof terms with
cuts and the checker's rule for them, cut elimination on terms one step
at a time and to the end, and on MLL nets (and on the nets of step 39),
where it is the reason nets exist, each step drawn.

## Fixed now

The forest holds the sequent's subformulas only, so a cut formula and
its dual need roots of their own beside the sequent's: the first session
is the data model, the checker and the interactive rule, with the
propositional search untouched (every proof it returns stays cut-free)
and `plan/notes/api.md` followed for where quantifiers will go (D17).
The second is elimination on terms, the third on nets with the drawing,
the fourth cut on the web page (below).
Termination and the preservation of the conclusion are tested on
generated proofs with cuts, every result through the checker.
Opus 5.5 at `high`; every change of a search or a criterion is reviewed by
the panel step 26's prompt describes, its argument read on Fable 5.1 at
`high`.

## Built on step 40 (2026-10-23)

Step 40 makes the two-sided sequent the public face of intuitionistic
linear logic: an ILL sequent lowered to the one-sided forest with every
occurrence's side recorded, every output raised through the reading.
A cut keeps that exact: the cut formula and its dual, which get roots of
their own beside the sequent's, get their sides too in intuitionistic
mode (ILL's cut from `Γ ⊢ A` and `Δ, A ⊢ C` to `Γ, Δ ⊢ C`: the cut
formula the goal of the left premise and a hypothesis of the right), so
every inference of a proof with cuts prints as the ILL sequent it is,
and step 40's round trip extends to proofs with cuts and to every step
of their elimination.

## The fourth session: cut on the page (the author, 2026-10-23)

A session of its own after the third, in the bindings and the web client
(`/home/tux/Projects/own/linlog-prover.github.io`), whose CLAUDE.md
applies there: the cut rule in the page's interactive proving (the
student names the cut formula, its two premises open as goals), and
elimination step by step on the derivation and on the net, each step
drawn, undoable and named, within step 38's tokens, grammar and
44-pixel targets; the bindings' calls for both, tested; the client's
lock moved after linlog's push. No lesson or practice here: those are
step 43's fourth session (43d).

## What steps 39 and 40 left you (finished 2026-10-24)

- **The two surfaces** (step 40): an intuitionistic sequent is an
  `IllSequent` (strict parser, its own JSON form at wire level 5, sides
  recorded at lowering and pinned equal to `Reading::new`), and a
  classical one a `ClassicalSequent` with each occurrence's side and
  implication bit recorded by the parser's own pass, drawn two-sided by
  `TwoSided` behind `ViewOptions::classical`. Step 40's report says
  where a cut's formulas get their record: typed as written (ILL text in
  intuitionistic mode, classical text in classical mode), its two
  occurrences recorded as roots on each side where the cut is made,
  never read off the one-sided `C` and `C⊥`; the two-sided view names
  the cut `cut`, and step 40's round trip and recorded-sides tests
  extend to them.
- **Nets with boxes** (step 39): `MellStructure`, classical with or
  without Mix and intuitionistic (the criterion without Mix, exact by
  the one-output invariant of unit-free proofs, which a cut keeps:
  a cut adds one input and one output), the weakenings hung on axiom
  links, sequentialization and desequentialization, the drawings with
  boxes in both looks (`svg::NetLook`) and the essential drawing; a cut
  link is a new kind of link, and its elimination on nets with boxes
  includes the exponential steps (a box entering a box, duplication by
  contraction, erasure by weakening), each drawn.
- **The interactive layer** (step 38h): `Interactive::invert` and
  `invert_all` apply the invertible rules; a cut is no invertible rule,
  and the automatic steps must leave a goal's cut alone.
- **The oracles**: the focused target set's latest label is
  `bench/targets/ill-surface.csv`, the net set's `net-ill-surface.csv`
  (step 40b's labels, if it recorded new ones, are in its report); the
  propositional search stays cut-free and every decided row's counters
  equal.
- **The Rocq library** still lives under `rocq/`; it moves to a
  repository of its own only at step 45, so this step extends it here.

## How the step runs

Four sessions on Opus 5.5 at `high`, each with its goal in its launch
prompt and reviewed by the planning session before the next; the panel
of step 26's prompt (the argument on Fable 5.1 at `high`, the
counterexamples on Opus 5.5 at `high`, the integers on Sonnet 5.5 at
`high`) for the checker's cut rule (41a), for elimination on terms
(41b) and on nets (41c), each on a frozen tree; a fresh-context reviewer
on Opus 5.5 at `high` for the page (41d). Each session's linlog part is
pushed by the planning session before any client part.

## The machine

Steps 43b, 43c (the lessons, in the client alone) and 44 (the guide for
AI models, documentation in linlog) may run beside this step. This
step's timed runs on cores 2 and 3 and its builds, the gate and the
panel's runs on cores 4 to 7; step 44's on 8 and 9; step 43's on 10 and
11; `nix flake check`, `nix build` and every timed run under `flock
/tmp/linlog-machine.lock`.

## Known already (2026-10-19), for the review that finishes this prompt

- **The checker has ordered modes** (step 35): cyclic MLL and the
  Lambek calculus check the arcs at every `⊗` and, in L, a non-empty
  antecedent; a cut in those modes must keep the order (the cut formula
  between the two arcs) or be refused there, said so.
- **Refutations** carry four checked kinds (classical, unbalanced, the
  count equation, and since step 37 the saturated set, wire level 3);
  cut changes no refutation, since every one is about cut-free
  derivability and cut is admissible.
- **The nets** of step 39 (MELL with boxes) and the drawing of step 36
  are what cut elimination on nets draws, step by step.

Deliverable: thematic jj commits; `plan/reports/41-cut.md`.
