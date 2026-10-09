# Scope extensions: how linlog could grow beyond its saved plans

Written 2026-10-09 from the eight field notes in `scope/`, `plan/later.md`,
`design-constraints.md` (cited D-1 to D-12) and the two judges' scores.
Candidates are cited by the field notes' ids, steps by number as in
`plan/README.md`. "(inference)" marks this note's own conclusions. The
"fable" judge and this synthesis run on the same model, so a tie is
broken here by a stated reason, never by the judge.

## 1. The ranking

Each judge gave value, feasibility and fit from 1 to 5; a candidate's
score is the sum of both judges' sums, 30 at most. An asterisk marks a
split verdict (section 2), "†" a candidate both judges place far or out
despite its score.

- **28** interop-1 (exchange-format spec), semantics-decision-1 (ideal
  certificates).
- **27** proof-theory-9 (explaining unprovability).
- **26** education-1 and engines-8 (one item: a provability light),
  education-2 (linking exercises), education-3 (hints), proof-theory-4
  (subformula linking).
- **25** semantics-decision-4 (directed reachability).
- **24** education-8 (shareable proofs), interop-7 (Python package),
  applications-1 (plans as certificates).
- **23** proof-theory-1 (proof equivalence), proof-theory-11
  (Curry–Howard export), semantics-decision-5 (Petri-net front door).
- **22** engines-4† (solver encodings), applications-4 (programs and
  processes), education-7 (accessibility), interop-5 (TPTP
  conventions), types-higher-2 (term printers).
- **21** engines-10 (STRIPS and PNML readers), engines-3 (integer and
  trap refutations), interop-10\* (notebooks), logics-1\* (SELL),
  types-higher-4\* (certificate back end), proof-theory-6 (compression).
- **20** applications-8\* (bounded exponentials), education-4 (course
  packs), education-5† (generated exercises), engines-7\* (ML
  environment), proof-theory-3\* (unification nets),
  semantics-decision-2 (Presburger invariants), semantics-decision-3
  (ideal Karp–Miller).
- **19** types-higher-3\* (sessions), engines-9† (state across calls),
  semantics-decision-9† (wider Horn), education-6\* (LK/LJ trainer),
  engines-1\* (SAT intuitionistic), interop-6† (tactics),
  semantics-decision-6\* (affine saturation).
- **18** engines-5\* (learned dispatch), proof-theory-7† (token
  machine), applications-2† (rule-system front end).
- **17** logics-6\* and types-higher-7\* (relevant mode), education-9†,
  engines-11†, interop-2†, interop-9†, logics-2†, types-higher-1†,
  types-higher-5\*.
- **16** applications-7\*, proof-theory-2†, proof-theory-8†,
  applications-3†, applications-6†, engines-2†, interop-3†, interop-8†,
  logics-7†, proof-theory-10†, semantics-decision-7†.
- **15** logics-5†, interop-4†, logics-3†, types-higher-6†.
- **14 to 11** logics-8\*, applications-5\*, semantics-decision-8\*,
  proof-theory-5†, engines-6\*, logics-4 (both: not for linlog).

Score and verdict part at engines-4: cheap printers worth writing only
when a solver route (engines-2, -3) is weighed.

## 2. Where the judges disagree, and why

Nineteen candidates split, on standard rather than fact (inference):
fable calls a candidate near when the saved design already has its
slot, opus when a user is asking and the item is a step of its own.
Fable's totals are given first.

- **The modal family** (logics-1 11/10, logics-6 10/7, types-higher-5
  9/8, types-higher-7 10/7, applications-8 10/10; fable near, opus
  far). Fable: linear and affine are the two-label case of SELL, a
  relevant flag or a per-root bound is a `Mode` builder away. Opus: five
  spellings of one design, to be settled once, and nobody has asked.
  Here opus wins on the implementation and fable on the API: payload and
  zone shape fixed now (section 5), the logics waiting for a user.
- **Assembly items** (interop-10 11/10, engines-7 10/10; fable near,
  opus far). Both agree they are cheap once step 32 and interop-7 exist
  and differ on whether an assembly deserves a plan. Near, inside those
  plans.
- **proof-theory-3** (10/10; fable far, opus near). Both say it waits
  for 38's data model and is 38's obvious net design. Near, as a
  paragraph of 38's plan.
- **types-higher-4** (11/10; fable near, opus far). Opus objects to
  freezing a builder at 0.1.0; D-5 fixes `Proof::new`'s shape anyway.
  Near, merged into interop-1.
- **types-higher-3** (11/8; fable near, opus far). A layer the size of
  step 25 waiting on 34. Far; its near part is the CP export.
- **education-6** (9/10; fable far, opus near). `interact --logic` is
  already a step-25 follow-up in `later.md`; a full LK/LJ rule set is a
  second interactive calculus. Far beyond that follow-up.
- **engines-1** (9/10; fable far, opus near). Both give value 4 to the
  ILTP gap (108 of 274 against 262); fable reads `later.md`'s "not a
  SAT solver" and prefers G4ip. Near as the need, far as the method.
- **semantics-decision-6** (8/11; fable far, opus near). Both want it
  measured on 37's engine first. Far, recorded in 37's prompt.
- **engines-5** (10/8; fable near, opus far). Far until 35 and 37 give
  it rows to choose among.
- **Far against not for linlog** (applications-5 8/5, applications-7
  9/7, logics-8 8/6, semantics-decision-8 7/5, engines-6 6/5). Opus
  draws the line where a maintained tool or another product owns the
  problem; fable keeps a note for "after X". Opus's line is taken:
  nothing in steps 29 to 38 reduces these costs, so a note has nothing
  to wait for (inference).

## 3. The groups

### Near

Thirty candidates in six clusters: what each takes, which saved step it
follows or extends; the API room is gathered in section 5.

**A. Saying and certifying "no"**: the author's stated goal and the
refutations note's cheapest certificates.

- *proof-theory-9*: a failing `&`-slice by counts, missing and surplus
  resources, the deepest failed branch drawn with its stuck leaf.
  Extends 21's `Refutation` and 32's SVG. The slice is small, the
  missing-resource search needs a bound, the kept attempt costs memory.
  One session.
- *semantics-decision-1*: the backward coverability basis as a
  `Refutation` payload with a quadratic Rust checker, a Rocq lemma
  later. Extends 27's follow-ups, feeds 31. Small; certifies every
  refuted qcover instance.
- *semantics-decision-2 with engines-3* (one item): integer state
  equation, traps and modulo-k congruences as inductive invariants
  checked per transition, in exact pure-Rust arithmetic. Extends 27's
  follow-ups (the odd `a`, weights past 2²⁰). A session.
- *semantics-decision-4*: A*-style ordering and flat-cycle acceleration
  for the 66 undecided LLTP nets, each heuristic a row earned on
  `bench/targets.sh` (D19); the checker untouched. Extends 27.
- *semantics-decision-3*: ω-markings with acceleration, coverability
  forward and boundedness as a new question. Extends 27's follow-up on
  the 128-token counter.

**B. The teaching client over step 32**: in 32's client plan or the step
after it; no new engine.

- *education-1 = engines-8*: a bounded search per new goal after every
  `apply`, verdict cached; `later.md` assigns the per-goal budget to 22
  and 32 already.
- *education-3*: layered hints from a found proof, invertible rules
  applied automatically; the one care is mapping a `⊗` split onto the
  goal's occurrences.
- *proof-theory-4*: `Interactive::link(goal, a, b)`, a bounded focused
  search on two paths, a menu where splits make it non-unique.
- *education-2*: a partial linking checked by the existing criterion,
  the witness mapped to SVG ids; MLL now, boxes after 33. Fills
  usability story 9.
- *education-8*: the JSON state compressed into the URL, an embed;
  client work with a size check.
- *education-7*: an outline printer and spoken forms in the library,
  `graphics-document` and keyboard focus in the client.
- *education-4*: a versioned exercise format (mode, fragment, allowed
  rules, expected verdict), graded by the checker, typeset by the
  exports; a rule restriction in `Interactive` is new. Beside 24, 32.
- *interop-10, engines-7* (assembly): an anywidget over interop-7 and
  32's client; a JSON line protocol over `interact` with the filtered
  legal-actions list, a step-9 follow-up already.

**C. Proof identity and size.**

- *proof-theory-1*: `Proof::canonical(mode)` and `linlog equiv`: MLL by
  the net, MALL by multi-focusing or `&`-resolution linkings, units
  refused. Beside 34 and 35. One session; also a regression oracle
  between engines (D19).
- *proof-theory-6*: remove unused `Weaken`/`Copy`, hash-cons identical
  subproofs, check afterwards. Attacks `vm_compute` on 46 768 nodes and
  `later.md`'s exponential read-back. Small; shortest proofs and cut
  introduction stay far.
- *proof-theory-3*: in 38's plan, links carry a unifier and the
  first-order `ProofStructure` is linking plus mgu.

**D. Proofs as programs** (*proof-theory-11, types-higher-2,
applications-4*: one step). A read-back pass over the two-sided term and
printers: a neutral linear λ-calculus, then GHC `LinearTypes`, Granule,
and CP/DILL processes for classical proofs. Extends 22's
one-options-value discipline; a GHC round trip tests the checker. Half
a session to one.

**E. Front doors, formats and bindings.**

- *interop-1 with types-higher-4*: a spec and JSON Schema of every form
  read back, rule semantics and preorder numbering stated. Documentation
  and a schema test, with or right after 30.
- *semantics-decision-5 with engines-10's PNML half*: a PNML reader and
  the MCC coverability and target-marking queries; `practice.md` ranks
  MCC first. Medium (an XML dependency, the query fragment); the net
  drawing last, since D12 has no layout for it. Beside 27 and 29.
- *applications-1 with engines-10's STRIPS half*: the firing sequence as
  a plan with its partial order, state-equation weights as a no-plan
  certificate, a `.sas` reader (the GPL translator outside the flake's
  closure). The planning step `practice.md` defers past the release.
- *interop-5 (a)*: SZS status lines, small; (b) a TFF dialect belongs
  to 38; (c) is engines-11, far.
- *interop-7*: PyO3 bindings with maturin wheels in the workspace
  (D22), the web bindings' JSON options, the GIL released during search.
  Beside 32.

**F. Intuitionistic non-theorems in the ordinary layer** (*engines-1*
as the need). 166 ILTP problems are undecided and the Kripke
countermodel has no source. The method is open: the loop check or G4ip
that `later.md` and the refutations note name needs no dependency; a
SAT route needs a pure-Rust solver for wasm. Measure both on the 35
problems that deepen without end (inference). Extends 25.

### Far

Each with its gate.

- The modal family (*logics-1, -2, -6, types-higher-5, -7,
  applications-8*): one design for labels, grades and structural flags
  when a SELL or Granule user asks; the API room is taken now.
- *logics-3* after 36 and that design; *logics-5* after 38 and 31, a
  plan of its own; *logics-7* a follow-up of 33/34.
- *proof-theory-2* if proof-theory-1 needs MALL nets; *-7* after 33 and
  34; *-8* after the trainer features; *-10* with the proof-identity
  thread.
- *semantics-decision-6* measured on 37's engine; *-7* after a prototype
  count on the refuted MALL targets; *-9* after counting the (!,&)-shaped
  programs.
- *types-higher-1, applications-3* after proof-theory-1 and 34 settle
  identity; *types-higher-3* after 34; *types-higher-6* after 38.
- *applications-2, education-9* once the Horn net view has a step API;
  *applications-6* after the PNML reader; *education-5* after
  education-4 has users and the refuters land; *education-6* beyond
  `interact --logic`.
- *interop-2* when cslib has a release and 31's design is known to port;
  *-3, -4, -6, -8, -9* on demand; *engines-2, -4* when a solver route is
  weighed; *engines-5* after 35 and 37; *engines-9* when a profile shows
  re-proving dominates; *engines-11* as outreach beside the LLTP header
  report.

### Not for linlog

- *logics-4, proof-theory-5 (b)*: deep inference needs a proof object
  outside the `Node` tree, a new engine and certificate kind.
- *applications-5, logics-8*: a logic programming or functional language
  is another product; linlog can be its back end through the exports
  and the Python package.
- *semantics-decision-8*: protocol analysis is Tamarin's, Hyper-Ackermann
  in the worst case.
- *engines-6*: a training pipeline that speeds up only proofs found
  early.
- *applications-7*: a worked example in the documentation.

## 4. What the near list says about the plans (inference)

The near items cluster around what exists: the Horn engine's net view
(A), the interactive state (B), the proof term (C, D) and the wire forms
(E); none needs a new logic. Two saved plans grow: 32 carries cluster B
as its client's feature list, 38 carries proof-theory-3. Two steps
emerge: "explaining and certifying unprovability" (A, after 31) and
"proofs as programs and the exchange-format spec" (D with interop-1,
beside 30). The front door and planning readers are the step
`practice.md` defers past the release.

## 5. What the coming API design should leave room for now

Merged from the near candidates and the far ones' items that cost
nothing now. "(D-n)" marks what `design-constraints.md` already decides,
"(new)" what it does not.

1. `Refutation` payloads as versioned named structs under a
   `#[non_exhaustive]` enum: `Ideal`, `Invariant`, `Slice`, `Missing`,
   `Attempt`, `KripkeModel`, `PhaseModel` (D-3, D-8; the list is new);
   the Petri translation a pure public function, place = atom name,
   transition = clause `OccId` (new).
2. `Stop::poll(progress)` and `memory::Account` on every long entry, so
   several small searches share one budget (D-9).
3. A question kind beside provability (coverability set, boundedness)
   in the front door and the JSON; `Engine` as data naming fragments,
   modes, shapes and questions (new; `Engine` as data is in the README).
4. The Horn `Program`, marking and firing sequence as public
   serialisable types with `enabled` and `fire`, built from net data
   without a `Sequent` and printable to one; the sequence kept on the
   answer (new).
5. Optional labels on roots, hypotheses and clauses, carried through
   `Forest`, `Proof`, the derivation view, the exports and the JSON
   under `serde(default)` (new).
6. Counts as numbers: multiplicities on literals, copy counts on `Copy`
   nodes and the dyadic zone, under the 2⁵³ rule (D-3 has the rule).
7. The export entry over `&Proof` as a list of targets, each with its
   own `Unsupported`: Rocq kernels, λ-terms, CP, later Lean, Dedukti, a
   calculus-of-structures view; `Proof` carrying its `Mode` (D-5;
   targets beyond `rocq::Options.kernel` new).
8. `Interactive`: goal status as data, goal ids stable across `apply`
   and `undo`, an `apply` payload of several rule instances under one
   undo, `close` as a peek that grafts nothing, a hint value (goal,
   position, `Rule`, split), a rule subset the session filters by,
   invertibility per rule and mode, every operation as a JSON request
   (D-5 has `apply` as a value; the rest new).
9. SVG ids as a contract: `i<n>`, `o<n>`, `l<m>-<n>`, a `<g>` per
   subformula occurrence, prefixes reserved for boxes, doors, cuts,
   slices, edges, states and firings (D-6 has vertex ids and box
   prefixes; occurrences, edges, states new).
10. The wire forms as a specification with a schema: `version`,
    additive keys, rule names as stable strings, no field whose meaning
    depends on build features, explicit sharing in the `Proof` JSON,
    `from_json_within` with a bound (D-3, D-9; schema and sharing new).
11. Entry points taking plain data and an object-safe `Stop`, a `Send`
    session, no global state, errors as codes, unknown verdicts keeping
    their cause (D-3, D-9; object safety and `Send` new).
12. `Mode` `#[non_exhaustive]` with constants and `with_*` builders, so
    `with_contraction`, `cyclic` and `ordered` are additions (D-8).
13. `Term`'s modal variants with a payload whose zero is today's
    connective; the zone a type with a per-label partition and an
    ordered variant, not one bitset (D-4, D-7; label map and ordered
    variant new).
14. The `Exists` witness slot admitting an implicit witness, a
    metavariable the unifier resolves, beside an explicit term (extends
    D-5; new).
15. The net flag as a `Criterion` value open to `Mall { kind }` and a
    stratification check; `from_proof` strict on unhandled nodes (D-6;
    the MALL variant new).
16. A public `Walk` with binder stops able to drive a spoken and a term
    printer (D-10).
17. A documented "equal up to permutation" on `Proof`, not `PartialEq`,
    the DAG term's node order stable so a canonical order is a
    renumbering (new).
18. `search::Options` `#[non_exhaustive]` with room for `max_proofs`, a
    schedule and an ordering hook; a recording sink in the engines, a
    no-op by default (D-8 marks it; fields and sink new).
19. Parse errors with documented spans, a recovering parser, spans per
    subformula beside the forest (the span unit is in the README; the
    rest new).
20. `families::Instance` public, proof statistics derived from the term
    (new, for education-5).

Items 1, 2, 7 to 10 carry most of the near list; 12 to 14 cost nothing
now and keep the far logics possible (inference).
