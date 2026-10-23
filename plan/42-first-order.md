# Step 42: first-order linear logic

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing. The step is several
sessions with a plan of its own, which its first session writes; this
prompt was finished on 2026-10-19, at the review of step 37. It says
what is fixed.

Read `plan/later.md` ("First-order linear logic"),
`proof-search-specifications.md` ("First-order fragments"),
`plan/notes/api.md` (where terms and binders go, decided in step 28),
`plan/reports/17-assessment.md` (3.9, the author's answer 4), the
reports of steps 26, 28 and 31, and `plan/README.md` (D1, D5, D6, D17).

## Goal

Quantifiers and predicates over terms through every layer: the data
model, the parser, the printer and the JSON; unification with a trail in
the engines; witnesses in proofs and the checker's eigenvariable
condition; the views, the interactive rules (a witness given or left
open), the outputs; certificates in the Rocq library's first-order
development.

## Fixed now (the author, 2026-10-03; D17)

The propositional case must not become meaningfully slower: the target
set's counters stay identical and its pinned CPU time within a few
percent at every stage, by generics or by a duplicated fast path if that
is what it takes. The data model comes first and by itself. First-order
MLL stays in NP and first-order MALL is NEXPTIME-complete (Lincoln and
Shankar, LICS 1994; Lincoln and Scedrov, TCS 1994); with exponentials
the copy bound and its three-valued answer carry over. No problem
library exists: the families gain first-order generators, and Moot's
LinearOne is the prover to compare with. The first session, which
writes the step's plan and settles how quantifiers enter the data
model, the proof term and the checker (decisions every later session
builds on), on Fable 5.1 at `high`; the rest on Opus 5.5 at `high`
(the re-evaluation of 2026-10-18 in `plan/README.md`); every change of a search, a criterion or the checker is
reviewed by the panel step 26's prompt describes, its argument read on
Fable 5.1 at `high`.

## What steps 31 to 37 left you

- **The engines** (`search::DISPATCH`, documented on `Engine`): the
  focused engine one- and two-sided, `additive`, `horn`, `net` (a
  hand-over at the focused engines' limits), `essential` (forced
  only), the planar search of the ordered modes, and since step 37 the
  focused inverse method, which takes turns beside the default on
  intuitionistic goals with an exponential and on Mix. The inverse
  method is the forward method that extends to first-order logic by
  unification on its initial sequents (Chaudhuri's work on the focused
  inverse method treats the first-order case; read the source before
  relying on it), and step 37's report says where the
  terms go in its database's keys; a backward engine needs unification
  with a trail. Say which engines gain quantifiers in this step and
  which refuse them.
- **Refutations**: the classical refuter, the unbalanced atom, the
  count equation and the saturated set (step 37, `Refutation::Saturated`,
  checked in `search/saturated.rs`) are propositional; with quantifiers
  a saturation is a semi-decision, and the classical reading of a
  first-order sequent is no longer finite, so each refuter refuses
  quantified goals or carries over with its argument.
- **The ordered modes** (step 35) refuse every engine but the planar
  search; the Lambek calculus's embedding into first-order MILL (Moot
  and Piazza, 2001) is the other route step 35 left for after this
  step: weigh it against the planar search once the data model exists.
- **The outputs**: the derivation view, the exports, the SVG drawings
  (nets with every formula written, step 36), the web bindings
  (`web/`, `linlog-web`) and the client in its own repository all read
  the forest; a term inside an atom changes what they print and what
  the page's buttons stand over. The wire forms are at level 3; a
  quantified sequent is a new level.
- **The oracles** for D17: `bench/targets/after-cyclic.csv` and
  `bench/targets/net-after-cyclic.csv`, with the rows
  `bench/TARGETS.md` records; `mix/11` and the rows the inverse
  method's turns answer (`mix/8` to `mix/11`) are compared at equal
  work, since a row that ends at its time limit has no repeatable
  counters.
- **The Rocq library** (`rocq/`, step 31) proves the propositional
  calculi, the checker's soundness and the refutations' invariants; its
  first-order development is this step's, its certificates checked by
  the flake as today's are.

## Built on steps 40 and 41 (2026-10-23)

Step 40's ILL formula type keeps the place for `∀` and `∃` (D17): the
quantifiers join it and its strict parser, and a formula's side is still
a property of its occurrence, so the side table carries over. What
changes is the text: an instance in a derivation (an eigenvariable, a
term substituted at `∃R` or `∀L`) is raised with the substitution in
force at that inference, so the two-sided sequent printed there is the
instance, and step 40's round trip extends to first-order sequents and
their proofs. Step 41's cut, if it reaches first order, cuts on an
instance with the same rule for its sides.

## How the step runs

Several sessions, a plan of their own (`plan/first-order/README.md`).
The first, on Fable 5.1 at `high`, writes the plan and settles the data
model, the proof term and the checker's eigenvariable condition; the
rest on Opus 5.5 at `high`; the panel of step 26's prompt for every
change of a search, a criterion or the checker, its argument on Fable
5.1 at `high`. The last session brings first-order logic to the web
page (the author, 2026-10-23): input with quantifiers and terms, the
quantifier rules in interactive proving (the student gives the witness
at `∃R` and `∀L`, the page checks the eigenvariable condition through
the library), the drawing, the bindings' calls, tested, and the
client's lock moved after linlog's push; no lesson or practice there,
which step 43's fourth session (43d) writes.

Deliverable: `plan/first-order/README.md` (the step's own plan), thematic
jj commits, `plan/reports/42-first-order.md`.
