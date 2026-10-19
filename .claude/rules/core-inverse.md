---
paths:
  - "core/src/search/inverse/**"
  - "core/src/search/saturated.rs"
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

Every `Unprovable` of this engine rests on each of these, and its
certificate (the set it saturated with, "The certificate" below) is
checked against the same argument by code of its own. The argument: a
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
- **Coherence** (`rules::coherent`, `coherent_list`, `Saturation::coherent`): outside
  the subformulas of `?` (`copied`), a sequent of a proof holds each
  occurrence once, none below another, and two members only below a
  `⅋` (a `⊗`, `&` or `⊕` above both sends them to different premises
  or keeps one side); `theta`'s members join in, since a `?`'s formula
  stood beside the others' ancestors when it was decomposed. Pairwise
  is consecutive in id order: the common ancestors of a preorder-sorted
  list are those of its neighbours. A subset of a coherent sequent is
  coherent, so a sequent that subsumes one of a proof passes. The
  argument needs the goal's own members coherent (their common ancestor
  is decomposed below them in no proof): the roots are, and so is every
  goal of a derivation of them, but `Goal::new` takes any multiset, so
  `admits` refuses a goal off the roots that is not (`NotTaken::Shape`;
  the panel's argument found `[a, ~a]` of `⊢ a ⊗ ~a` refuted before,
  proved by the focused engine). Two members joined by a `?` formula
  (the formula twice, or beside its own subformula: an interactive goal
  after `?c` and `?d`, which the first refusal turned away, the panel's
  second finding) are let through: the goal's phase decomposes the `?`
  first, so no stable sequent holds the pair, and below its subformula
  all is `copied`.
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
- **Work**: a step of a rule written, a choice of `⊕` sides looked at
  (one a `0` ends writes no step, and there can be `2ⁿ`), a literal
  scanned for an axiom's partners (a pass over the atom's literals with
  an `lca` walk each, seconds on `SYJ206+1.019`), a member copied into
  the tables by `record` (the goal's phase of `SYJ202+1.007` copies 52
  million, charged and polled as it goes), all polled every
  `STEPS_PER_POLL` (the panel's limits reviewer found each of these
  unpolled at first: a time limit of 2 s ending after 3.2 s, of 4 s
  after 10.9 s, a memory bound passed by a third). In the saturation
  the units are what takes the time: a member of a linear zone copied
  into an evaluation, compared with a hole's or indexed when a sequent
  is taken up, a member of a derived sequent tested for coherence, a
  child looked up in a trie (`Trie::lookups`, counted by every query),
  and one for each application and each premise choice pruned. Polls
  stay where they were (every sequent taken up, every
  `APPLICATIONS_PER_POLL` applications or choices, which the tests'
  poll budgets count), plus one wherever `UNITS_PER_POLL` units passed
  since the last, which only large zones reach. Counting one unit per
  application instead made the units per millisecond range from 1 (the
  `wide` sequents of 2 048 literals, a 1.4 s late stop) to 146 000; by
  members and lookups they range from about 5 000 (the Partition
  families) to 143 000 (`SYJ207` in its cbn translation), 18 000 to
  89 000 on the library's intuitionistic problems, which is what lets a
  budget of units bound the engine's time. Linear members only: an unrestricted zone is
  copied whole, which costs little per member, and counting it made
  `SYJ207`'s units a hundred times cheaper than the rest; a choice
  pruned is one unit, not its depth, for the same reason. The proof's
  writing polls every `BUILT_PER_POLL` kept sequents with no units.
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
- **A new filter or rule** in the engine needs its twin in the check
  (`saturated.rs`), or the engine's sets fail it (`NotClosed`, an error
  of the search): a filter that drops sequents the check keeps leaves
  conclusions the set does not subsume.

## The certificate

- **What it is** (`Refutation::Saturated`, `search::Saturated`,
  `search::Stable`): the polarity (`positive`, the bias's signs) and
  every kept sequent not killed at its pop (`Saturation::saturated`),
  zones as kept (globals left out), `absorbs` the `top` flag. Closed:
  every application to sequents taken up was made, and what it derived
  was kept or subsumed, a kill leaving a stronger alive one. `conclude`
  prefers the counts' and the classical refutations (computed first, as
  before), and checks the set like a proof (`Options::check`), under the
  stop with `Phase::Check` (a stop gives `Unknown(Stopped)`, a failed
  check `Error::Refutation`, an error: the engine and the check
  disagree).
- **The check** (`search/saturated.rs`, `check`) shares no code with
  `inverse/`: it re-reads the rules off the forest (`Calculus::rules`:
  `D1` foci, `D2` copies but negative literals, every `⊕` choice not
  ending in a `0`), writes each as postorder steps (`write`, no
  recursion over the formula: forests can be deep), evaluates them with
  its own `take`, `join` and shape test (`derive`), and applies every
  rule to every choice of premises among the set (candidates: exact
  sequents holding the hole's leaves, every weak one) and of partners,
  plus Mix on every pair where the mode has it and is not affine; each
  conclusion, globals left out, must be incoherent or subsumed. Then
  some hole of the goal's phase must be filled by no sequent. Before
  that it refuses an ordered mode, intuitionistic Mix, an incoherent
  goal (`NotApplicable`), a polarity of another length, members past
  the forest or zones out of order (`Mismatch`), and a sequent not
  shaped as the mode's (`Unshaped`), which the subsumption's soundness
  needs (an affine sequent with no output would otherwise stand for one
  with an output weakened in).
- **What the check prunes, and why that is sound**: only choices whose
  evaluation must fail: two exact premises of one release with
  different contexts (its `&` joins compare them after taking the
  shared leaves, so they fail). No pre-filter of partners by coherence
  with the focus: the conclusion holds the focus (or, for a copy of a
  global, nothing it is incoherent with: a global is coherent with every
  occurrence of the goal), so coherence drops the same conclusions.
- **Its argument** is the engine's ("What removes a sequent" above):
  focusing completeness under the set's polarity, the normal form of
  the `?`-literal partners, the monotonicity of each step, coherence of
  every proof's sequents given a coherent goal, the globals. A fault
  shared by the design of both (a rule missing from the calculus as
  written in both) is what no check catches; the differential test
  against the reference prover is what guards the calculus itself.
- **Measured**: the 213 refutations of the measurement's rows the
  engine refutes within 60 s, every one accepted (the harness's own
  `Disproof::check`); on the 58 that nothing else refutes the check
  costs 17 to 90 % of the search (`SYJ210+1.020` in cbv 209 to 330 ms,
  `SYJ208+1.005` in cbn 9.1 to 10.7 s); on `SYJ207+1.002` in cbv
  (1 704 sequents) 37 s beside 46 s of search, so a refutation near the
  time limit can become `Unknown`. Its units count each step by the zone
  it yields, each candidate and member compared and each subsumption
  test (polls every `UNITS_PER_POLL`): before they did, a stop came
  11 s late. The tests: `the_engines_sets_pass` (every commutative
  mode), `a_wrong_set_is_refused` (one wrong set per condition), and the
  differential test and the reference comparison, where 69 sets pass
  through the check.
