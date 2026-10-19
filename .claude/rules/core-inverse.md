---
paths:
  - "core/src/search/inverse/**"
---

# linlog core: the inverse method

Loaded, beside `core.md`, when the forward engine (`Engine::Inverse`,
`search/inverse/`) is read or changed. Its place among the engines and
the front door are in `core-search.md`.

## The engine

**Files.** `mod.rs` the saturation (`Saturation`: the sequents kept,
their store and records, the queue, the indices, the goal's holes) and
the engine's `Decide`; `rules.rs` the derived rules compiled from the
forest (`Shape`: polarity, coherence, positions, the goal's globals;
`compile`, `goal`); `eval.rs` a rule's term evaluated forward over
premises (`Calculus::derive`, `join`, `take`, `mix`), which with a node
arena writes the proof; `index.rs` the tries subsumption asks;
`tests.rs` the differential test.

- **A rule is a term with holes** (`Step`, postorder): the proof term of
  one focusing phase, a focus on a positive formula (`D1`: a positive
  occurrence whose parent is `⅋`, `&`, `!` or none, i.e. one a stable
  sequent can hold) or a copy of a `?` formula's subformula (`D2`,
  `Step::Copy` at the root), through its `⊕` choices (one rule per
  choice, `plans`), its axioms (`Step::Axiom` with a slot of partners)
  and the asynchronous phases of what it releases (`⅋`, `⊥`, `?`, `&`,
  `⊤`; `⊤` first, then the unary ones, `&` last), ending in holes. The
  goal is one more such term, the goal's own phase (`rules::goal`).
  Evaluation, search and proof writing are one function (`derive`), so
  the proof returned is the one the search judged; `build` replays each
  kept sequent's record (rule, premises, partners) once, premises first,
  with an explicit stack (no recursion over the derivation).
- **What a derived sequent stands for** is what the checker reads:
  `gamma` exactly, with any unrestricted zone holding `theta`; with
  `top` (a `⊤` above, the checker's `any`) or in affine mode any larger
  linear zone as well. `take` mirrors the checker: absent and `top`
  absorbs it (an output only when the zone has none, R2), absent in
  affine mode it is weakened in with a `Weaken` node (never an output,
  R3), else the step fails. `join` is the checker's `&`: equal exact
  zones, an absorbing zone within the exact one, both absorbing to the
  pointwise maximum, and in affine mode the exact sides weakened to the
  maximum. `Bang` needs the zone empty after its subformula and resets
  `top`. Every step's zone must be shaped under a reading (R1: one
  output, or none with `top`). A mismatch between `derive` and the
  checker shows as `Error::Rejected` (a proof the checker refuses),
  never as a wrong proof; it can hide as a wrong refutation, which only
  the differential test sees.
- **Partners.** A positive literal in focus takes a negative literal of
  its atom (`partners`): from the linear zone, or, where the partner is
  the subformula of a `?`, from the unrestricted zone (`Copy` above
  `Ax`), and from the linear zone too only where the goal holds it. So
  no `D2` rule is written for a negative-literal subformula of a `?`:
  in a backward proof such a literal reaches the linear zone only by a
  copy, whose `Θ` member stays, so a derived sequent with it in `Θ`
  instead stands for every stable sequent above the copy with the
  literal moved back (the normal form the completeness argument uses).
  Rules focusing on one formula share a literal's list (`Rules::lists`).
- **Globals.** The subformulas of the `?` the goal reaches through `⅋`
  alone (`Shape::globals`) are in the unrestricted zone of every
  sequent of every proof (their `?` steps are below every `&` in the
  goal's phase, which decomposes the unary connectives first), so they
  are left out of the kept sequents' `theta`; the proof still copies
  them, and the goal's `Quest` steps empty the root's zone. A `?` under
  an `&` of the goal is no global.

## What removes a sequent, and why it removes none a proof needs

Every `Unprovable` of this engine rests on each of these (`Exhausted`,
no certificate: the saturation is not checked). The argument: a
backward focused proof of the goal (complete for every bias and mode,
`core-focus.md`) has each stable sequent `T` derived forward, up to the
normal form above, as some kept `S` that subsumes it, by induction on
the proof, because evaluation is monotone in subsumption (a premise
that subsumes the backward premise makes every step succeed and derive
a zone that subsumes the backward one; `take`, `join`, `Bang`, R1 and
R2 each checked case by case in `plan/reports/37-inverse.md`).

- **Subsumption** (`subsumed`, `index::Trie`): an exact `T` subsumes an
  exact `S` with the same `gamma` and `theta_T ⊆ theta_S`; an absorbing
  or weakening `T` subsumes any `S` with `gamma_T ⊆ gamma_S` and
  `theta_T ⊆ theta_S`. Never an exact sequent a weak one: it stands for
  less. Transitive, so a sequent subsumed later (killed at its pop,
  `dead`) has an alive one that subsumes it, which is taken up in turn.
  Under a reading a kept exact or affine sequent has exactly one output,
  so a weakening `T ⊆ S` never needs an output weakened in.
- **Coherence** (`Shape::coherent`, `Saturation::coherent`): outside
  the subformulas of `?` (`copied`), a sequent of a proof holds each
  occurrence once, none below another, and two members only below a
  `⅋` (a `⊗`, `&` or `⊕` above both sends them to different premises
  or keeps one side); `theta`'s members join in, since a `?`'s formula
  stood beside the others' ancestors when it was decomposed. Pairwise
  is consecutive in id order: the common ancestors of a preorder-sorted
  list are those of its neighbours. A subset of a coherent sequent is
  coherent, so a sequent that subsumes one of a proof passes.
- **Shape under a reading**: R1 at every step (above); a partner of the
  other position than its literal (an `Ax` has one output).
- **Partners pre-filtered** by coherence with the formula the focus
  began on (both stand in the conclusion).
- **Mix is left out in affine mode** (as the focused engine: weakening
  proves what Mix would).
- **Pruned premise choices** (`fits`): two exact premises of one
  release (`Hole::group`) with different contexts, or, under a reading,
  releases that bring more than one output: `derive` would fail on
  them, so no sequent is lost.
- **Fairness**: the queue takes the smallest sequent (`gamma` plus
  `theta`) first, then the oldest; finitely many sequents have each
  size, so every one kept is taken up. A combination of premises is
  applied when its last premise is taken up, at its first place among
  the holes (the holes before it take the sequents taken up before it,
  those after it it too), so each is applied once.

A fault in any of these that over-removes a sequent makes
`agrees_with_the_focused_engine_and_the_reference` fail (each tried in
a scratch copy; the report lists them).

## Limits and work

- **Memory**: the rules (charged as written, `Writer::settle` after
  every rule and every `STEPS_PER_POLL` steps within one, the phases on
  the stack and the holes counted too: the goal's phase of `SYJ202`
  multiplies `&` branches inside one rule), the kept sequents, store,
  records, queue, tries, postings and lists, and the proof's nodes are
  charged to the search's account; over it the answer is
  `MemoryLimit`, since the kept sequents cannot be given back without
  losing what was derived. Not charged: a rule application's transient
  premise lists and zones.
- **Indices are `u32`** (kept ids, ranks, zone lengths, the rule
  tables' ranges, trie nodes); each is refused with `IndexLimit` where
  it would pass (`keep`, `Writer::record`, `Trie::insert`), which only a
  search without a memory bound reaches. Output counts are `u64`.
- **Work**: a step of a rule written (polled every `STEPS_PER_POLL`), a
  sequent taken up (polled each time), a rule or Mix application and a
  premise choice pruned (polled every `APPLICATIONS_PER_POLL`); the
  proof's writing polls every `BUILT_PER_POLL` kept sequents with no
  units.
- **Options**: `bias` (the focused engines' rule; `Auto` as they read
  it; on the class C and refutation rows the two rules gave the same
  counts) and `check`. No copy bound: with exponentials the search runs
  until the stop, the work bound or the memory bound.

## Where the next steps plug in

- **Quantifiers**: a forward sequent's members would carry terms; the
  place is the zone's members (`Zone::gamma`, the trie's keys), with
  unification at the axioms' partners and at a hole's members, and
  subsumption as matching. The rules' terms keep their shape.
- **A dispatch row**: the engine has none; a row needs the measurement
  of the step report, and its feature in `Feature`.
- **A certificate for its refutations**: the saturated set closed under
  the rules, checked without this code, would replace `Exhausted`.
