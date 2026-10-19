# Step 35: cyclic MLL and the Lambek calculus

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing. The step takes two
sessions; this prompt is finished at the review of step 35. It says what
is fixed.

Read `plan/later.md` ("Cyclic MLL and the Lambek calculus"),
`proof-search-specifications.md` ("MLL variants: units, Mix, cyclic MLL
and Lambek"), `plan/reports/17-assessment.md` (3.8), `32-mll-engines.md`,
and `plan/README.md` (D2, D8, D19).

## Goal

A non-commutative mode: cyclic MLL by planar axiom linkings in the net
engine, no exchange in the derivation view, the interactive rules and
the outputs, and the Lambek calculus with its two divisions and without
empty antecedents, read two-sided. The mode is a value of `Mode` (D2),
and the dispatch gives it its engine (D19).

## Fixed now

The order of a sequent's formulas, which `Sequent::optimize` sorts away
today, has to be kept for this mode, a change of the arena's canonical
form that `plan/notes/api.md` should have left room for; say in the
report what it cost. The embedding of the Lambek calculus into
first-order MILL (Moot and Piazza, 2001) is the other route and is
weighed against the planar search once step 38 exists; this step builds
the planar one. Opus 5.5 at `high`; every change of a search or a criterion is reviewed by
the panel step 26's prompt describes, its argument read on Fable 5.1 at
`high`.

## What step 35 left you

- **The net engine's loop is generic over its criterion**
  (`plan/reports/32-mll-engines.md`, "The search"): `Linker<'a, C:
  Correctness>` keeps the choice, the frames, the cadence, the stop,
  the counters and the cubes, and a criterion answers `mate`, `admits`
  (its constant-time rejections), `link` (fallible, for the memory
  bound), `unlink`, `test` and `balanced`. `Switching` is the classical
  criterion, `Essential` the polarized one for IMLL (directed
  acyclicity by a closure of bits with an undo log, the dominator
  condition). Planarity is a third criterion in that place, or a
  rejection inside `admits`; the Lambek calculus is the intuitionistic
  one, so its criterion is the essential net's with planarity
  (Lamarche and Retoré's planar essential nets).
  `ProofStructure::is_essential` decides essential nets independently
  of any search, as a planar test must too.
- **The prunes do not carry over unexamined**: step 35a's three prunes
  (`Symmetry`, `admissible`, `ordered`, `balanced` in
  `core/src/search/net.rs`) rest on exchanging equal literals, which
  keeps a classical or an essential verdict but can break planarity,
  since it moves an axiom link to another place in the cyclic order.
  Each prune is kept for the planar criterion only with its argument
  redone there, or switched off for it; the counting and balance
  arguments survive (they ignore the order), the exchanges need
  thought.
- **The routing** (35a, departs from the design: no static feature):
  the focused engines were faster on every MLL and IMLL row both
  decide, so the dispatch has no net row; the net engine takes over
  only at their recursion limit or memory bound, and the essential
  engine runs only when forced (`--engine essential`). A
  non-commutative mode has no focused engine, so the planar engine is
  its default from the start, and its measurement is against itself
  and the commutative engines on the same sequents, not against a
  rival.
- **The drawing**: `svg::Style::essential` (`--style
  svg.essential=true`) draws the essential net's direction, polarities
  and witness; the planar drawing keeps the conclusions in their
  cyclic order with no crossing link, as the net SVG draws its links
  today (step 11's layout).
- **Left open by 35b that touches this step**: the shared loop's
  `choose` counts every unlinked literal's partners before a poll,
  quadratic in one atom's literals (a 1 s stop overshot by 23 s on
  65 000 copies of `a` and `~a`); a planar engine inherits it.
- **Certificates**: the Rocq library (step 31) has no non-commutative
  calculus; certificates for this mode are out of scope here, and the
  exporter refuses the mode rather than write a commutative proof.

## How the step runs

Two sessions on Opus 5.5 at `high` (plan/README.md, "Why these models
and efforts", 2026-10-18), each with its goal in its launch prompt: the
first keeps the order (the arena's canonical form, the mode, the
derivation view without exchange, the interactive rules, the parser
and the outputs) and builds cyclic MLL's planar criterion and engine;
the second the Lambek calculus, two-sided, with its two divisions and
without empty antecedents. Each change of a search or a criterion has
step 26's panel: the argument read on Fable 5.1 at `high`, the
counterexamples and the integers and limits on Opus 5.5 and Sonnet 5.5
at `high`, each on a frozen tree. The net target set
(`bench/net-targets.sh`, against `bench/targets/net-after-prunes.csv`)
must keep every counter of the commutative modes.

Deliverable: thematic jj commits; `plan/reports/35-lambek.md`.
