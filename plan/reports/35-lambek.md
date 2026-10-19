# Step 36: cyclic MLL and the Lambek calculus

## The checklist

| item | state | evidence |
|---|---|---|
| Session 36a: the order kept, and cyclic MLL (session `step-36a`, 2026-10-19) | done (2026-10-19) | The cyclic mode (`Mode::CYCLIC`, `Order`), the ordered parse, the checker's arcs, the planar criterion (`ProofStructure::is_planar`), the planar search as the mode's default (`search/planar.rs`), the session's arc splits, the exports and `--cyclic`; the commutative modes' sequents, JSON forms and snapshots unchanged (behaviour lock passing, its help texts reblessed for the flag). `agrees_with_every_planar_linking` (the engine against every planar linking, every proof checked) and `planarity` (every linking against a pairwise test) pass. 35a's prunes: the skeleton balance kept, the three symmetry breaks off (argument and counterexamples above). Panel in two rounds (Fable 5.1 argument, Opus 5.5 counterexamples, Sonnet 5.5 integers, each at `high` on a frozen tree): four defects found and fixed, nothing of the arguments refuted. Target sets: `net-after-prunes` every counter on 64 rows, `after-efficiency` every counter but the five recorded rows. Measurement on `bench/problems/cyclic.txt` (cores 2 and 3). Every code commit gated; `nix flake check` at the last commit (exit status in the message to `planning`). |
| Session 36b: the Lambek calculus (session `step-36b`, 2026-10-19) | done (2026-10-19) | The Lambek calculus `L` (`Mode::LAMBEK`, `--lambek`) and `L*` (`with_empty_antecedents`, `--empty-antecedents`): the divisions `A \ B` and `B / A` in the ordered parse; the ordered reading (`Reading::of_mode`, `IllConnective::Under`/`Over`); the checker's empty-antecedent rule beside the arcs (`Fault::EmptyAntecedent`, the oracle's own `antecedent`; `check/mod.rs`'s `lambek` refuses a proof whose `⊗R` premise has none); the two-sided view with the divisions' rules (`Named::division`), the session's rules (`StepError::EmptyAntecedent`), the exports (`lambek.tex`, `.typ`, `.svg`, `lambek-net.svg`), the command and README; the search: `Planar<C>` generic, `Planar<Essential>` as the mode's default with `keeps_antecedents` on complete linkings; `lambek_agrees_with_every_planar_essential_linking` (enumeration of planar essential linkings, brute-force sequent calculus, every proof checked) passes. Commutative and cyclic modes unchanged (behaviour lock passing, only the help reblessed). Panel in two rounds (Fable 5.1 argument, Opus 5.5 counterexamples, Sonnet 5.5 integers, each at `high` on a frozen tree; round 2 on the fixes after round 1's tree): nothing of the argument refuted, no unsound or incomplete verdict; an error index fixed, a goal of a sequent outside `L` answered; a handover defect found before it, fixed. Target sets: `net-after-cyclic` every counter on 64 rows, `after-cyclic` every counter on the 144 rows decided within the limit, and `mix/11`, time-limited there, every counter equal to the base at equal work. Measurement on `bench/problems/lambek.txt` (cores 2 and 3). Every code commit gated; `nix flake check` at the last commit (exit status in the message to `planning`). |

## Session 36a: the order kept, and cyclic MLL

**Outcome.** linlog has a cyclic mode: `Mode::CYCLIC` (`--cyclic`, the
word `cyclic`), cyclic linear logic multiplicative and without units,
where a one-sided sequent is a sequence of formulas read up to rotation
and nothing else is exchanged. The order is kept everywhere a sequent
goes: the ordered parse reads text with the dual that reverses products
and the antecedents reversed (`Sequent::parse_ordered`); the checker
accepts a cyclic proof exactly when every `⊗` joins the two arcs on
either side of it (`Fault::Order`); proof structures have the planar
criterion (`ProofStructure::is_planar`, decided by a stack scan
independent of any search, and `Criterion::CYCLIC`); the net engine,
the mode's default and its one engine, searches planar linkings under a
third criterion that wraps the classical one with two rejections (the
regions between the links made, and a balanced span); a session splits
a `⊗` only into arcs (`Step::at`, `StepError::Exchange`) and the search
closes any goal; derivations, LaTeX, Typst and SVG draw it with no
exchange, a planar net with no crossing link; Rocq certificates are
refused. Keeping the order changed nothing of the commutative modes:
the arena's canonical form was the written order already, their
sequents, JSON forms and snapshots are as they were (the behaviour lock
passes, its only change the new flag in the help), both target sets keep
every counter, and the ratchet's journeys are within 0.02 % of their
counts before the step but four: `search-wide-m2-256` (+0.24 %) and the
three that check proofs, which pay the checker's test of the mode at
each node (`check-wide-m1-2048` +0.72 %, `check-qbf-20-2` +0.44 %,
`ordinary-pigeons` +0.21 %). Of 35a's prunes the planar search keeps the skeleton balance
and switches the three symmetry breaks off, the equal conclusions'
among them, which the differential test caught left on. The panel
reviewed every change of a search or a criterion in two rounds; it
found four defects, all fixed, and refuted nothing of the arguments.
Measured on a new problem file, the planar search costs what the
classical net search costs where the linking is forced (chains of 4 096
links in 0.46 s), refutes the exchanged forms at once, and decides
random derivations in under half a millisecond where the classical net
search runs past a minute on four of forty. The Lambek calculus is the
second session's; where it goes is under "What is left open".

### What keeping the order took

**No change of the arena's canonical form.** Step 28 had made the
written order canonical already (`optimize` merges and drops but never
sorts the roots), and the occurrence forest numbers a sequent in
preorder over the roots in written order, a formula's left operand
before its right one. That numbering is the cyclic order of the
literals, and of the formulas of every subproof, read from its first
occurrence: so the order is kept by everything that already reads the
forest, and nothing in the arena, the forest, the JSON form of a
sequent or the stored proofs changed. The commutative modes' sequents,
JSON forms and snapshots are as they were: the behaviour lock
(`cli/tests/lock`, `core/tests/lock/json.txt`) passes, and its only
change is the command's help (the new flag, and the problems format's
list of mode words, which now names every mode), reblessed in the
commit that adds the flag.

**What did change** is how text becomes a cyclic sequent and what the
order costs where code is shared:

- *The ordered parse* (`Sequent::parse_ordered(text, order, limits)`, the
  sibling of `parse_within` that `plan/notes/api.md` 3.1 and 10.8 had
  reserved): in the cyclic order the dual of a product reverses its
  operands, `(A ⊗ B)⊥ = B⊥ ⅋ A⊥`, and `A₁, …, Aₙ ⊢ Δ` is read
  `⊢ Aₙ⊥, …, A₁⊥, Δ`; `A ⊸ B` stays `A⊥ ⅋ B`, the left division, so
  `a, a -o b |- b` holds and `a -o b, a |- b` does not. Without the
  reversing dual `|- ~(a * b), a, b` would be unprovable in the cyclic
  mode, a wrong answer (api.md's walk-through 36). `FromStr` stays the
  commutative reader. *Cost*: none in the end, but the layout matters.
  With the parser's loop out of line and the parser passed to it, the
  commutative reader's `read-text` journey took 1.5 % more instructions;
  with a copy of the loop per reader (inlined, or the parser generic
  over the order) 2 to 7 % once one binary links both readers (the
  harness, and the command), since a callee both copies call
  (`Parser::atom`, `HashMap::insert`) lost the inlining LLVM gives a
  single caller. Both readers now call one function that makes the
  parser as its own local and runs the loop (`parse::read`), the order
  read once at the end (`dualise::<ORDERED>`): `read-text` counts
  12 418 964 instructions against 12 418 960 before the step.
- *The mode* (`Order`, `Mode::CYCLIC`): a private field with a getter,
  as api.md 3.5 planned, so every earlier mode is `Order::Commutative`
  and keeps its value, name and wire form; `Modes::take` destructures
  it, so the dispatch had to place it (a row of its own; every other
  row takes commutative modes only).
- *The net engine's classical loop*: the planar criterion calls the
  classical criterion's `admits`, which made LLVM outline `Forest::lca`
  from the classical loop, +2.4 % of `search-wide-m2-256`'s
  instructions with every counter equal; `#[inline]` on `lca` takes it
  to +0.24 %. The checker pays a test of the mode per node
  (`check-wide-m1-2048` +0.72 %, `check-qbf-20-2` +0.44 %,
  `ordinary-pigeons`, which checks its proofs, +0.21 %). Every other
  journey is within 0.02 % of its count before the step (both counted
  under callgrind in this session, the base at `yttsoxkx`; every count
  within its ceiling's tolerance).
- *The wire*: a document that names the cyclic mode, or a structure of
  the cyclic order (`"order": "cyclic"`), is written at level 2, which
  0.1.0 does not read; level 2 was not released yet (it holds the
  classical refutation), so it takes the mode too, and every commutative
  document stays at level 1 byte for byte.

### Decisions

- **The planar engine is the net engine** (`Engine::Net` under the cyclic
  mode), not an `Engine` of its own: it is the same algorithm (D7), the
  mode has no other engine, and a verdict line naming `net` says which
  search ran. The essential engine has a name of its own because it
  competes with the net engine in one mode; nothing competes here.
- **Planarity is a third criterion that wraps the classical one**
  (`Planar` around `Switching`), not a second type parameter of `Linker`
  beside the criterion with an empty commutative instance, as api.md
  10.8 had it (departs from the design): the loop was made generic over
  its criterion in 35b, a criterion is where rejections live, and the
  commutative search is untouched by construction (its counters equal;
  the one instruction cost, `lca`'s inlining, is taken back). The
  Lambek calculus wraps the essential criterion the same way.
- **The arcs are checked by a scan, not a sort** (api.md 10.8 asked "one
  sort per `⊗`"; departs from the design in form only): the distances
  from the `⊗` going round give the condition in one pass over the two
  contexts, and the oracle sorts, so the two are independent. The axiom
  needs no adjacency check: its sequent is its two literals.
- **Unit-free MLL only.** Cyclic linear logic has units, and `⊥` and `1`
  would need no order check; the step fixes unit-free MLL, the net
  engine has no units, and the checker forbids them in the mode so that
  the mode is one calculus everywhere.
- **The derivation view is unchanged**: ascending occurrence order is a
  cyclic sequent's order read from its first formula, so every sequent
  of a cyclic derivation is a rotation of the order its rule reads and
  no exchange appears; the view does not rotate each sequent to the
  rule's literal shape (`⊢ Γ, A` above a `⊗`), which a reader of cyclic
  sequents does not need. (Assumption: "no exchange in the derivation
  view" means no step that is not a rotation.)
- **A goal off the roots** in the cyclic mode is searched as a sequent
  of its own and carried back, so that `prove_goal` and a session's
  `close` work on any goal; in the commutative modes the net engine
  still refuses goals (`NotTaken::Goal`), as before.
- **Rocq refuses disproofs of the mode too**: a commutative refutation
  certificate would be true (a cyclic proof is a commutative one) but
  would state another calculus's sequent.
- **Level 2 takes the cyclic mode**: level 2 was not released, a mode is
  an enumeration a reader refuses when it does not know it (`wire.rs`),
  so a document of the mode or order is written at level 2; every
  commutative document is unchanged.
- **`Split::At` names the first formula going left** going round from
  the `⊗`, the step's own position meaning none; it is accepted in the
  commutative modes too, as a split like any other, since nothing is
  gained by refusing it there.

### The criterion and its argument

**The checker** (`arcs` in `core/src/proofs/check/pass.rs`, an
independent sort in `oracle.rs`). In the cyclic mode a proof term passes
when the commutative rules accept it, every node is `Ax`, `Par` or
`Tensor` (`Fault::Forbidden` otherwise), and at every `Tensor(o, l, r)`
the contexts left after taking `o`'s operands are the arcs on either
side of `o`: going round from `o` in occurrence order, every member of
the right premise's context before every member of the left one's
(`Fault::Order`). *Why that is cyclic MLL*: in a term the commutative
rules accept, every zone holds each literal under it once (the
footprint argument of the review: `Ax` adds two distinct literals,
`Par` and `Tensor` add their premises' footprints, and the root's is the
goal's), so every zone is an antichain of the forest, and its ascending
order is the cyclic order the derivation induces on it. `Par`'s operands
are then neighbours, left before right, and need no check; a two-formula
axiom has one cyclic order; and since no context member lies in `o`'s
subtree, the operands take `o`'s place, so `⊢ Γ, A` and `⊢ B, Δ` give
`⊢ Γ, A ⊗ B, Δ` exactly when, going round from `o`, `Δ` precedes `Γ`.
Conversely every cyclic derivation of the root in occurrence order has
its sequents in induced order, so it is such a term. A member in both
contexts (passed by `<=`, as the oracle orders ties) belongs to no
accepted term. A goal off the roots is read in ascending order, which is
its induced order.

**The criterion** (`ProofStructure::is_planar`, `Criterion::CYCLIC`). A
structure of unit-free MLL is a proof net of cyclic MLL exactly when it
is a Danos–Regnier net without Mix and planar, no two links crossing in
the cyclic order of the literals (their id order; crossing is the same
in the linear and the cyclic reading, whatever the rotation). *Only if*:
`Par` keeps the order of the literals, and `Tensor` places its premises'
links in the two arcs `Γ, A` and `B, Δ`, disjoint and contiguous, so a
crossing would lie within one premise. *If*, and the claim that every
sequentialization of the existing sequentializer is a cyclic proof (the
argument reviewer's version, which holds at every stage, the splitting
`⊗` a root of the sub-net or not): removing a splitting `⊗` `t` leaves
two connected parts, the left one holding `t`'s left operand and the
right one its right operand; each conclusion's literals form a block; if
a block of the left part stood, going round from `t`, before a block of
the right part, the chain of links in the right part from `t`'s right
operand to that block would contain a link around the left block, and
the chain in the left part from that block back to `t`'s left operand
would leave it by a link that crosses it. So the blocks read right then
left, which is `arcs`. `is_planar` is a bracket matching: with the links
laminar each closing literal closes the link on top of the stack, and
when it does not, the partner is below the top and the top's partner not
yet reached, which is the witness `NetError::Crossing` (partial
structures included: unlinked literals are skipped). `is_correct` asks
it after the commutative conditions, so a commutative structure's
verdict and witness are as before.

### The search

`core/src/search/planar.rs`: the net engine's loop (`Linker`, generic
over its criterion since step 35b) runs in the cyclic mode under
`Planar`, a third criterion beside `Switching` and `Essential`, which
wraps the classical one (its structure under `Criterion::CYCLIC`, so
that the net found is a planar one and sequentializes in the cyclic
mode) and adds two rejections to `admits`, each sound for every partial
linking:

- *Regions*: a literal's region is the innermost made link around it in
  the linear order (the place of the link's left end), or none. A
  candidate link crosses no made link exactly when its two ends lie in
  one region; `link` moves the places of its span that were in its
  region into its own, `unlink` (last in, first out, as the loop calls
  it) moves them back. So the linking stays planar by construction (a
  `debug_assert` in `test` checks it on complete ones), at a cost
  linear in the span per link.
- *Balanced spans*: a link leaves the literals strictly between its ends
  to be linked among themselves, so every atom must have as many `a` as
  `~a` there. That depends on the two ends alone and is compared through
  prefix sums of a hash of the signed atoms (wrapping on purpose: equal
  counts give equal sums in any build, and a collision only lets a link
  through, which the regions and the exact test then judge).

`test` is the classical exact test and `balanced` the classical skeleton
balance (a planar net is a net); the count equation and each atom's
balance (`counts_admit`) refuse before any link, as before. The
dispatch's first row gives unit-free MLL in an ordered mode to the net
engine (`Modes::Ordered`), its default there from the start; every
other engine is refused in the mode (`NotTaken::Mode`), and a fragment
beyond unit-free MLL is `Error::OrderedFragment`. It runs on one thread
(the cubes are the classical criterion's), and a race runs one search.
A goal off the roots, in the cyclic mode, is a sequent of its own: the
goal's formulas (distinct, none inside another) as the roots of the
same arena, whose forest numbers each formula as the goal's forest
does, shifted; the proof found is carried back by the shift, so a
session can close any goal.

The test (`agrees_with_every_planar_linking`) runs the engine through
`prove` on 900 rounds of random derivations of the cyclic calculus over
one to three atoms (built rule by rule, turned at random), their
mutants and random balanced sequents, against an enumeration of every
planar linking (pairwise crossing test, the classical criterion on each
complete one), with every proof through the cyclic checker and every
net planar; it requires 600 decided, 200 provable, and 50 provable only
commutatively. It found one defect at once: the equal conclusions'
symmetry break, which predates 35a and was not behind 35a's switches,
was still on; it is now a switch of its own (`Prunes::conclusions`).

### 35a's prunes under the order

Each of 35a's prunes, for the planar criterion:

- **Equal literals that are conclusions** (the oldest break) and **equal
  literals under one tree of `⊗` alone or `⅋` alone** (35a's first):
  *switched off*. The orbit argument needs every exchange to map nets to
  nets; an exchange of two equal leaves moves both links, and the
  nested linking `x₁ < x₂ < p₂ < p₁` exchanges into a crossing one. The
  canonical (ascending) representative can be the one that is not
  planar: `|- a * a, ~a, ~a` is provable only by the descending linking
  (1–4, 2–3), which the break rejects; the differential test caught the
  conclusions' break on `|- ~b, ((b * (c * a)) * ((a * ~c) * b)), (c par
  (~c * ~b)), c, ~a, ~a, ~c`.
- **Equal compound subformulas** (35a's second): *switched off*, by the
  same argument for whole subtrees: `|- (a * b) * (a * b), ~b, ~a, ~b,
  ~a` is provable only by the fully nested linking. A conditional break
  (reject the descending linking only when its exchange crosses no made
  link) would be sound; it is not built, since no measured row needs it
  (below).
- **The balance over the skeleton's components** (35a's third): *kept*,
  with its argument unchanged: it rejects a partial linking only when no
  Danos–Regnier completion exists (a literal never links into its own
  component, so each component's `a` need distinct `~a` outside it),
  and a planar net is a Danos–Regnier net. The same holds for the count
  equation, the per-atom balance and the two classical constant-time
  rejections.

The planar search adds its own balance, per span, which the order makes
exact where the skeleton balance only bounds.

### Options (D15, D16, D17)

- **A mode, not an option**: the cyclic mode is a value of `Mode`
  (`Mode::CYCLIC`, D2), its name `cyclic` on the wire (a client holds
  it as JSON, as any mode), `--cyclic` on the command, `"mode":
  "cyclic"` in a batch entry and a harness problem file. Its default
  engine is the net engine, chosen by the dispatch (D19); nothing else
  takes it.
- **The parse takes the order** (`Sequent::parse_ordered(text, order,
  limits)`): a front end that holds a mode passes `mode.order()`; the
  web client reads text through it (or sends JSON, which is lowered
  already).
- **The criterion's order** is a public field of `Criterion` (`order`,
  `Criterion::CYCLIC`), flattened on the wire as `"order": "cyclic"`.
- **The split at a cut point** is `Step::at(position)` (`Split::At`),
  which a client offers for an ordered goal; `Step::left` still works
  where its positions make arcs.
- **No new knob**: the planar search is exact and has no threshold;
  its tables are not counted under `Limits::memory_bytes`, as the net
  engine's structure is not (documented on `Engine::Net`), and it reads
  the net engine's options (`test_period`, `check`), `jobs` being
  ignored in the mode.
- **Quantifiers (D17)**: `Order` is non-exhaustive and independent of
  the fragment, so a first-order cyclic sequent is the same mode; the
  ordered parse's reversal applies to binders as new tokens (a binder's
  dual keeps its single operand); the planar criterion is over literal
  occurrences and ignores what an atom carries, so first-order linking
  adds unification to the candidate test in `Planar::admits` beside the
  regions, and the prefix hash then weighs predicate symbols.

### The reviews

Step 26's panel, twice, each member a fresh context on a frozen tree (a
jj workspace at the commit under review), briefed with conduct.md's
rules on cores and memory (cores 4 to 7, capped scopes, bounded
enumerations, no run the brief does not name) and told to report only
what breaks soundness, completeness or a stated requirement. The briefs
are `panel-common.md` and `panel-round2.md` in the session's scratchpad;
their substance is this section.

**Round 1** (tree `yronpqyn`, git `12cc1fc3`: the mode and checker, the
ordered parse, the planar criterion, the planar search).

- *Argument* (Fable 5.1 at `high`): no step fails. It redid the checker's
  argument (the footprint argument that every zone is an antichain in
  induced order, `Par` and the axiom needing no check, `Tensor` exactly
  the cyclic rule, completeness by induction from the root), the ordered
  parse as the standard one-sided reading (`A ⊸ B` the left division),
  both directions of the planar criterion with the sequentializer at a
  non-root stage (the chain-of-links argument above), the stack scan,
  the region invariant under last-in-first-out unlinking, the span
  hashes, the soundness of the classical rejections and the balance for
  planar completions, that `Prunes::PLANAR` makes no chain at all, and
  that the commutative search and checker are unchanged. On the prunes:
  none of the three symmetry breaks could be kept as stated (its two
  counterexamples are in "35a's prunes" above); the balance is rightly
  kept. Its edge notes were the command's commutative parse of cyclic
  input and the cyclic criterion with Mix, both answered below.
- *Counterexamples* (Opus 5.5 at `high`): the engine, the checker,
  planarity, the criterion and the parse held on every input: about
  56 000 sequents against its own prover in the cyclic sequent calculus
  (also at test periods 1 to 6, two jobs, the check off), 3.2 million
  complete linkings (`is_planar` against a pairwise test, the cyclic
  criterion against the classical one plus non-crossing, every planar
  net's sequentialization accepted by the checker and by its own),
  about 470 000 terms through `check(CYCLIC)` and its own checker, 50
  sequents of 200 to 753 occurrences, 2 716 two-sided texts through the
  ordered parse; and 11 028 commutative runs byte-identical to the base
  commit (verdict, engine, every counter, the proof). Two findings, on
  paths that read the cyclic mode without its rules at that tree: (1)
  the command parsed a batch entry of mode `cyclic` commutatively, so it
  "proved" `a * b |- b * a` there; (2) a session in the cyclic mode took
  any split and every rule, its `proof()` then refused by the checker,
  so no unchecked proof escaped but `terms.rs`'s "never" was false.
  *Answer*: both hold, and both were already fixed in the commits after
  the frozen tree (the command's ordered reading, `Add --cyclic to the
  command`; the session's arcs and rules, `Prove step by step in the
  cyclic order`); round 2 confirmed the fixes. It also noted the
  dispatch table's placeholder for the measurement, filled at the end.
- *Integers and limits* (Sonnet 5.5 at `high`): no integer, index,
  length or bound that breaks soundness, and no wrap in any build (the
  arcs' distances, the places and regions as `u32` with `NONE` never a
  place, the prefix sums a ring homomorphism, the slices' bounds,
  `is_planar`'s unreachable branches, the antecedents' slice, the stop,
  work and memory bounds on the planar path, the front door's refusals,
  the wire levels). Four findings. (1) `Criterion::CYCLIC.with_mix()` was
  accepted and sequentialized into Mix nodes recorded as cyclic:
  *fixed*, `ProofStructure::new` refuses it (`NetError::Mode`). (2) The
  session's unchecked steps: *fixed*, as above. (3) A race and
  `Engine::parallel` treated the net engine as parallel in the cyclic
  mode (a pool searching the same single-thread search again; verdicts
  right): *fixed*, a race runs one search in an ordered mode, and the
  documentation says so. (4) A batch's error record that names the
  cyclic mode is written at level 1: *not changed*, since `wire.rs`
  names an error's detail among what does not raise the level and the
  record is written only.

**Round 2** (tree `lztpnvst`, git `8f132bbb`: the fixes, the net
engine's goals off the roots, the session's cyclic rules, the command's
and the harness's ordered reading, the Rocq refusal).

- *Argument* (Fable 5.1 at `high`): no step fails. The shift is the
  forest's own numbering of each member's subtree, strictly monotone, so
  it keeps the cyclic order the checker reads and the inverse shift
  gives completeness; `apart` admits exactly the distinct, non-nested
  goals; the session's split check accepts exactly the suffixes of the
  round order, which is `arcs`, and every cyclic derivation is built
  with `Step::at`; the Mix refusal reaches every constructor and no
  commutative criterion. Run against its own brute-force cyclic
  calculus: 24 590 goals off the roots, 1 908 proofs, every split of
  every `⊗` of 312 goals, an exhaustive step search proving exactly the
  provable ones.
- *Counterexamples* (Opus 5.5 at `high`): nothing breaks. About 91 000
  goal sets (52 000 repeated or nested, all refused; 39 000 antichains,
  each verdict its own decider's), 74 700 frontier goals (11 400
  proved, every proof through both checkers); about 13 300 random
  sessions and 175 000 steps (accepted exactly when the left context is
  the arc before the `⊗`), 44 000 closes, 6 400 finished proofs, 38 000
  JSON replays driven in lockstep, 30 500 derivations built step by
  step; the Mix refusal in the API and in JSON; and the command against
  the library on 1 400 texts (batches, single calls, `seq print`, mixed
  modes): no mismatch.
- *Integers and limits* (Sonnet 5.5 at `high`): nothing breaks (the
  shift cannot wrap or leave the forest, `partition_point` finds the
  member, `Proof::new` cannot fail, the goal path's stop, work and memory
  answers equal the roots', a refusal of the goal's own forest is an
  error, the cut point's indices, the parser's slice), and 332 864
  goals compared with their printed sub-sequents. One older defect,
  outside the step: a panic past the structure's vertex bound with the
  occurrence limit lifted (in "What is left open").

### The target sets

Both at the session's code, before the documentation (`bench/net-targets.sh
step36a` on core 2 and `bench/targets.sh step36a` on cores 2 and 3, one
unit under the machine lock, 01:11:40 to 01:26:02); kept as
`bench/targets/net-after-cyclic.csv` and `after-cyclic.csv`, recorded in
`bench/TARGETS.md`.

- **Net set**: `linlog-bench summary --against
  bench/targets/net-after-prunes.csv bench/targets/net-after-cyclic.csv
  --counters`: no verdict differs, and "of the 64 problems both decide
  alike on one thread, 0 differ in nodes, splits, memo_hits,
  memo_entries, links, tests".
- **Focused set**: `--against bench/targets/after-efficiency.csv`: no
  verdict differs; of the 135 problems both decide alike, 5 differ in
  counters, `partition-table/1-1` and the chain family's four (16, 64,
  128, 256: a memo entry and a hit), the five `TARGETS.md` records as
  differing since `after-routing`; and five rows decided that
  `after-efficiency` did not (`growing/16`, `/64`, `/256`, `/1024` and
  `mix/11`), the classical refuter's of `after-classical`. So every
  counter of the commutative modes is kept.

### The measurement

By 35a's method: the harness at the session's code (release), one
thread, two shards on cores 2 and 3 under the machine lock, 60 s per
run, runs under 2 s three times (median), 2026-10-19 01:26:02 to
01:28:20; the problem file `bench/problems/cyclic.txt` (new, made by
the script below), each problem in the cyclic mode (the dispatch's
engine and the net engine forced, the same run) and in the classical
mode on the same one-sided sequent (the focused engine by the dispatch,
and the classical net search forced). The planar search has no rival in
its mode, so the comparison is with the commutative engines on the same
sequents, which answer another question (provability with exchange).

| row | planar (cyclic) | classical net search | focused engine (classical) |
|---|---|---|---|
| chain `a₀, a₀ ⊸ a₁, … ⊢ aₙ`, 1 024 / 4 096 links | 29 ms / 0.46 s | 28 ms / 0.46 s | 53 ms / 0.59 s |
| the chain with its first two formulas exchanged | refuted, 8 ms / 82 ms | proved, 28 ms / 0.45 s | proved, 53 ms / 0.59 s |
| wide: a product of 1 024 / 4 096 atoms entails itself | 50 ms / 0.89 s | 48 ms / 0.89 s | 5 ms / 90 ms |
| wide-reversed: entails its reverse | refuted, 15 ms / 0.16 s | proved, 48 ms / 0.85 s | proved, 5 ms / 86 ms |
| products of one atom, and of `a`, `b` in turn, 4 to 32 factors | 34 to 187 µs | 35 to 302 µs | 31 to 41 µs |
| random derivations of the cyclic calculus, two atoms, 20 | all proved, 23 µs to 0.4 ms (total 1.3 ms) | 18, two past 60 s | all, total 1.3 ms |
| the same with two neighbours exchanged, 20 | 12 proved, 8 refuted, total 0.7 ms | 18 proved, two past 60 s | 20 proved, total 1.2 ms |

*Reading*: where the linking is forced (chains, wide products, one
atom), the planar search costs what the classical net search costs on
the same sequent, its counters equal (`nodes` and `links` alike): the
regions and the span hashes add nothing measurable to a search that
makes one link per literal, so keeping the order costs the net engine
nothing there. The exchanged and reversed forms, provable only with
exchange, it refutes before any link or after a few (the span balance
leaves some literal without a partner), five to ten times faster than
it proves the unexchanged ones. On random derivations, where the
classical net search must choose, planarity cuts nearly every choice:
the planar search decides all forty in under half a millisecond, where
the classical search on the same sequents runs past the minute on four.
The focused engine is ten times faster than either net search on the
wide products, as 35a measured for the commutative modes; it does not
take the cyclic mode.

The script that wrote the problem file (kept here, since the file is
committed and a later session may extend it):

```python
# Writes bench/problems/cyclic.txt: scalable sequents of cyclic MLL.
import sys
out = []
def add(name, expected, seq):
    out.append(f"{name}; cyclic; {expected}; -; {seq}")
for n in [16, 64, 256, 1024, 4096]:
    ants = ["a0"] + [f"a{i} -o a{i+1}" for i in range(n)]
    add(f"cyclic-chain/{n}", "provable", ", ".join(ants) + f" |- a{n}")
    swapped = [ants[1], ants[0]] + ants[2:]
    add(f"cyclic-chain-exchanged/{n}", "unprovable", ", ".join(swapped) + f" |- a{n}")
    prod = " * ".join(f"a{i}" for i in range(n))
    add(f"cyclic-wide/{n}", "provable", f"{prod} |- {prod}")
    rev = " * ".join(f"a{i}" for i in reversed(range(n)))
    add(f"cyclic-wide-reversed/{n}", "unprovable", f"{prod} |- {rev}")
for n in [4, 8, 12, 16, 24, 32]:
    prod = " * ".join(["a"] * n)
    add(f"cyclic-equal/{n}", "provable", f"{prod} |- {prod}")
    alt = " * ".join(["a", "b"] * (n // 2))
    add(f"cyclic-alternating/{n}", "provable", f"{alt} |- {alt}")
import random
def derivation(rng, atoms, budget):
    """A random derivation of cyclic MLL: its formulas in their cyclic order."""
    if budget <= 1 or rng.random() < 0.2:
        a = rng.choice(atoms)
        out = [a, "~" + a]
    elif rng.random() < 0.5:
        left = derivation(rng, atoms, budget // 2)
        right = derivation(rng, atoms, budget - budget // 2)
        a, b = left.pop(), right.pop(0)
        out = left + [f"({a} * {b})"] + right
    else:
        out = derivation(rng, atoms, budget - 1)
        if len(out) > 2:
            b, a = out.pop(), out.pop()
            out.append(f"({a} par {b})")
    k = rng.randrange(len(out))
    return out[k:] + out[:k]
rng = random.Random(36)
for budget in [40, 80, 160, 320]:
    for k in range(5):
        formulas = derivation(rng, ["a", "b"], budget)
        add(f"cyclic-random/{budget}-{k}", "provable", "|- " + ", ".join(formulas))
        i = rng.randrange(len(formulas) - 1)
        formulas[i], formulas[i + 1] = formulas[i + 1], formulas[i]
        add(f"cyclic-random-exchanged/{budget}-{k}", "-", "|- " + ", ".join(formulas))
head = """# Sequents of cyclic MLL, read in the cyclic order (the dual of a
# product reversed, the formulas left of |- in the reverse order), one
# per line as in slow-tests.txt: name; mode; expected; copies; sequent.
# chain: a0, a0 -o a1, ..., |- an, provable; chain-exchanged: its first
# two formulas exchanged, provable only commutatively; wide: a product
# entails itself; wide-reversed: a product entails its reverse, provable
# only commutatively; equal and alternating: products of one atom, and of
# a and b in turn, entailing themselves, where many linkings are planar;
# random: random derivations of the cyclic calculus over two atoms, with
# up to the budget of rules, and random-exchanged: each with two
# neighbouring formulas exchanged.
"""
open(sys.argv[1], "w").write(head + "\n".join(out) + "\n")
```

### Commits

In the workspace `../linlog-step36a` on `yttsoxkx` (the plan's head with
step 39 reviewed), each code commit gated in `../linlog-step36a-gate` at
that commit (clippy, the tests with the behaviour lock, both `cargo
hack` runs, the ratchet), every one signed but this report's commit,
made with signing dropped after the passphrase's cache expired (to be
signed by the supervisor):

| change | subject | gate |
|---|---|---|
| `lpuvnrym` | Add the cyclic mode and its checker | passed |
| `ltrqqqxt` | Add the ordered parse: the cyclic order's reversing dual | passed (`read-text` 12 418 963) |
| `lkqvzrrq` | Decide planarity: the criterion of cyclic MLL | passed |
| `yronpqyn` | Search planar linkings: the net engine in the cyclic mode | passed |
| `pyqurrqz` | Prove step by step in the cyclic order | passed |
| `nrwzyywm` | Export the cyclic mode, and refuse its Rocq certificates | passed |
| `txusttsk` | Add --cyclic to the command (the help's lock reblessed) | passed |
| `lmnvvktz` | Read cyclic problems in their order; add the cyclic problem file | passed |
| `lztpnvst` | Document the planar engine's measurement; keep the target sets | passed |
| (this one) | Report session 36a; the rules, the changelog | documentation |

`nix flake check` runs at the last commit, under the machine lock; its
exit status is in the last message to `planning`. Beside the gates: the
families' verdicts (`linlog-bench run --all-families --timeout 5`, as
the table asks for a change of `bench/`): no mismatch in 17 rows.

### What is left open, and where the Lambek calculus goes

- **The Lambek calculus** (session 36b) goes where the code left room:
  a mode that is intuitionistic and ordered (`Mode::check` refuses it
  today, `Error::OrderedMode`; its words `lambek` and `lambek-star` and
  the empty-antecedent switch are 36b's, `name` names the combination
  `cyclic-intuitionistic` until then); the divisions `A \ B` (`A⊥ ⅋
  B`) and `B / A` (`B ⅋ A⊥`) as new tokens of the ordered parse, which
  already reverses the antecedents; `Reading::of_mode` taking an
  implication's antecedent from whichever factor is in input position;
  the checker's R4 (no empty antecedent) beside `arcs`; and the search:
  `Planar` wraps `Switching` today, and the Lambek criterion is the
  essential net's with planarity, so `Planar` becomes generic over the
  criterion it wraps (`Planar<Essential>`), its rejections unchanged.
  The interactive split at a cut point and the exports need nothing new
  but the two-sided view's reading.
- **A conditional symmetry break** for the planar search (above), if a
  row ever needs it.
- **The batch's error records** name the cyclic mode at level 1 (an
  error's detail does not raise the level, `wire.rs`); a reviewer
  flagged it, and it stays as the rule says.
- **A panic past the structure's vertex bound** (older than this step,
  found by the second round's integers reviewer): `Switching::new`
  expects `ProofStructure::new` to succeed, but nothing between the
  front door and it checks `nets::MOST` (1 431 655 763 vertices), so
  with `Limits::occurrences` lifted a sequent of shared subterms that
  unfolds past it and passes the counts would panic instead of being
  refused, in the commutative modes as in the cyclic one. The default
  limit of fifty million occurrences shields every call that keeps it.
  A check of the forest's length in `Nets::admits` would refuse it; not
  done here, as conduct.md asks of a pre-existing defect.
- **The inherited quadratic `choose`** (35b's open item) holds for the
  planar search too; on the measured rows it never dominated.
- **Certificates**: the Rocq library has no cyclic calculus; the
  exporter refuses the mode (`Unsupported::Ordered`).

## Session 36b: the Lambek calculus

**Outcome.** linlog decides the Lambek calculus, read two-sided with its
two divisions: `Mode::LAMBEK` (`--lambek`, the word `lambek`), the
calculus `L` without empty antecedents, and
`Mode::LAMBEK.with_empty_antecedents()` (`--lambek --empty-antecedents`,
`lambek-star`), the calculus `L*`. A sequent `A₁, …, Aₙ ⊢ B` is read by
the ordered parse of 36a, which is Abrusci's embedding into cyclic MLL
(`⊢ Aₙ⊥, …, A₁⊥, B`, the dual reversing products), and which now reads
`A \ B` (as `A -o B`) and `B / A` (as `B par ~A`). The reading of the
mode (`Reading::of_mode`) takes an implication's antecedent from
whichever factor stands in the other position and prints the sequent
back as written; the checker is 36a's cyclic check under that reading
with one output per sequent and, in `L`, an input per sequent
(`Fault::EmptyAntecedent`, which the oracle checks its own way); the
derivation view, the session's rules and the exports are two-sided with
the divisions' rules `\L`, `\R`, `/L`, `/R`; the search is the essential
engine under `Planar`, now generic over the criterion it wraps (Lamarche
and Retoré's planar essential nets), with a test of complete linkings
that keeps every antecedent nonempty in `L`. It is the mode's default
and its one engine. The commutative and cyclic modes are unchanged:
their sequents, JSON forms and snapshots are as they were (the behaviour
lock passes, its only change the help of the new flags), and
both target sets keep every counter of the commutative modes (the net set on all 64 rows, the focused set on all 145 decided rows but `mix/11`, whose counters at the time limit are no function of the input). The differential test against an enumeration of planar
essential linkings and a brute-force sequent calculus decides 1 566
sequents in both calculi, 959 of them provable, 71 only with empty
antecedents. The panel (Fable 5.1, Opus 5.5 and Sonnet 5.5, each at `high` on a
frozen tree) refuted nothing of the argument and found no unsound or
incomplete answer; it found an error index naming the wrong formula
(fixed) and a goal off the roots that `prove_goal` refuses in `L`
(kept, below); a gap I found before it (the net engine taking over a
Lambek goal after a memory bound) is fixed. Measured, the search decides
random derivations of the calculus in under 5 ms where the classical net
search on the same sequents runs past a minute on 8 of 20, and proves
chains of 4 096 links in 0.4 to 2.8 s.

### Decisions

- **`L` is the default, `L*` an option (D16)**: the mode's field
  `empty_antecedents`, false in `Mode::LAMBEK`; `with_empty_antecedents`
  sets it only in an ordered intuitionistic mode and returns every other
  mode as it is, so no two values of `Mode` name one calculus, and
  `allows_empty_antecedents()` is the one question code asks. The words
  are `lambek` and `lambek-star` (api.md 3.5), the command's flags
  `--lambek` and `--empty-antecedents` (requiring `--lambek`). `name`
  stays total over the refused combinations (`lambek-affine`, …).
  `Mode::check` now refuses an ordered mode with weakening or Mix only
  (the intuitionistic reading is the Lambek calculus); 36a's
  changelog entry and `Error::OrderedMode`'s message say so.
- **The reversing parse serves the Lambek calculus** (api.md decision
  17, left to this step's panel between the reversing parse and an
  order derived from the commutative lowering through the reading): one
  order, the forest's, for both ordered modes, so the planar criterion,
  the arcs and the session's cut points of 36a apply unchanged, and the
  two-sided reading is a view of it. The divisions are tokens of the
  ordered parse only; the commutative reader refuses `\` and `/` as
  before.
- **The reading of the mode** (`Reading::of_mode`, api.md 3.8's
  `of_mode`; `Reading::new` stays the commutative reading): an
  implication's antecedent is the factor in the other position, either
  one; in input position the operands of `⊗` and `⅋` are written right
  one first; the hypotheses go round from the goal backwards. A sequent
  whose sides are unknown (a JSON sequent without them, a goal searched
  as a sequent of its own) is read from its shapes, which in unit-free
  MLL fix every position, rather than refused as api.md 3.8 had it
  (departs from the design): the engine's goals off the roots are such
  sequents.
- **The divisions' rule names** are a field of `Named`
  (`division: Option<Division>`, the closed enum `Division { Under, Over
  }`), not entries of `Named::ALL`, a public array of 34 whose length is
  part of its type at 0.1.0 (departs from the design in form: api.md
  said "`Named` gains the orientation"). The label tables have the four
  after `ALL` (`Named::DIVISIONS`, `Named::at`), and `\` and `/` are
  label symbols, so a user's label with either character now sets it as
  a symbol (in the changelog).
- **The derivation keeps a term's premise order** (the left operand's
  premise first), as every view draws `⊗` and as the session maps a
  premise to a child by position; so `\L` draws its consequent's premise
  first and `/L` its argument's. A view that drew both argument first
  would need the session's mapping changed for a matter of layout.
- **The search is the essential engine's, not a third engine**
  (`Engine::Essential` in the Lambek modes, chosen by the dispatch's
  first row): it is the essential criterion with planarity, the engine
  of the same algorithm (D7), and its name says which search ran. The
  net engine refuses the Lambek modes (`Nets::admits`), which also keeps
  the handover after a memory bound from running a search without the
  antecedent condition (below, "A defect found before the panel").
- **The antecedent condition is the search's, on complete linkings**
  (`Essential::keeps_antecedents`), not a field of the public
  `Criterion` with a `NetError::EmptyAntecedent` (api.md 3.8; departs
  from the design): `Criterion::of(Mode::LAMBEK)` is `CYCLIC`, the
  planar criterion of `L*`, documented as such; every path that answers
  for `L` goes through the search or the checker, and the command and
  the bindings draw only nets of proofs the checker passed in the mode
  (planning asked that this be so and pinned by one command test:
  `check_keeps_the_lambek_calculus_s_antecedents`). A criterion of `L`
  on structures can be added when a client judges a net it did not
  find.
- **Rocq certificates are refused** in the Lambek modes, as in the cyclic
  one (`Unsupported::Ordered`): the Rocq library has no non-commutative
  calculus.

### The criterion and its argument

**The calculus.** By Abrusci's embedding a sequent `A₁, …, Aₙ ⊢ B` of
`L*` is derivable exactly when `⊢ Aₙ⊥, …, A₁⊥, B` is derivable in cyclic
MLL, and the cyclic derivations of such a sequent in which every sequent
has one output are the `L*` derivations; those in which every sequent
also has an input are the `L` derivations (the restriction of `L` is a
nonempty antecedent at `⊗R`'s premises, at the argument of `\L` and
`/L`, and at the conclusion of `\R` and `/R`, all of which are sequents
of the derivation).

**The checker** (`one_succedent` in `pass.rs`, `antecedent` in
`oracle.rs`). The cyclic check of 36a (the arcs) under the ordered
reading, the intuitionistic rule (one output per sequent), and in `L`
an input per sequent: `d.gamma.len <= d.outputs` is
`Fault::EmptyAntecedent`, exact there because `within` has held the
zone's count to its bound before; the oracle looks for a member in
input position instead of counting. A root sequent without a hypothesis
has no reading in `L` (`ShapeError::EmptyAntecedent`).

**The net condition** (`keeps_antecedents`). In an essential net the
term at an output occurrence is what reaches it; its free hypotheses are
the sources that reach it (the sequent's hypotheses and the antecedents
of its implications, the occurrences no edge enters) but those bound
inside it, whose binding implication reaches it too. The condition: the
body (consequent) of every implication but the goal is reached by a
source other than its own antecedent whose binder does not reach the
body. The goal's body holds the sequent's hypotheses, of which the
reading demands one in `L`. *Claim*: for a planar essential net, some
sequentialization is an `L` proof iff every one is iff the condition
holds. *If the condition fails at `l`*: in any sequentialization, the
conclusion of `l`'s `⅋` rule is `l` with a context Γ; each member of Γ
reaches `l`, so reaches its body, by a path that starts with input
occurrences down from a source; that source is not `l`'s antecedent
(whose subformulas are in the premise, not the conclusion) and is free:
its binder `m`, if it has one, concludes a rule below `l`'s, whose
sub-net holds `l` and has `m` as its one output, so the body reaches
`m` (every vertex of a one-output sub-net reaches its output), and `m`
reaching the body would close a cycle; so Γ is empty. *If a
sequentialization has a sequent without an input*: it is a sub-net
whose one conclusion is an output; no edge enters such a sub-net from
outside (an edge in would come from an input connective outside to an
input conclusion of it, of which it has none), so every source in it is
an antecedent bound inside it; an implication of it that reaches no other
implication of it has every other source in its body bound inside
(the binder lies between the source and the body, or after the body,
which would make it reach another implication), so it breaks the
condition. The argument reviewer redid both directions (the first
precisely as written here after its two notes, below) and confirmed them
by enumeration.

**The search.** `Planar<C>` keeps 36a's two rejections (regions and
balanced spans) around any criterion; `link` asks the wrapped criterion
first so that a refusal there (the essential closure's memory) leaves
the regions as they were. Under it the essential criterion of the
ordered reading: its rejections (a directed cycle, an antecedent's
literal reaching the goal around its implication) and its dominator
test at the cadence are sound for every completion, so for every planar
one, and exact on complete linkings; Lamarche's equality of the
essential and the Danos–Regnier verdicts holds under the ordered reading
(the differential test asserts it on every complete planar linking).
35a's symmetry breaks are off (`Prunes::PLANAR`), by 36a's argument;
the essential criterion keeps no skeleton, so the balance is moot.
`keeps_antecedents` runs on complete linkings only, where every
occurrence reaches the goal and the closure is the reachability; it
prunes nothing on partial ones, and costs implications × sources bit
tests per complete linking. Goals off the roots are searched as a
sequent of their own, as the net engine does in the cyclic mode
(`net::goal`, now generic over its search); a goal without a hypothesis
is unprovable in `L`.

**The test** (`lambek_agrees_with_every_planar_essential_linking` in
`search/planar.rs`): 800 rounds plus four fixed cases, each sequent in
`L` and in `L*`, from random derivations of either calculus
(`generate::lambek`, built rule by rule), mutants (two hypotheses
exchanged, a division flipped), and random sequents; the engine's
verdict against an enumeration of every planar linking (a pairwise
crossing test) that is essential and whose sequentialization the checker
accepts in the mode, and against a cut-free brute-force sequent calculus
of `L` and `L*`; every proof through the checker, every net essential
and planar. 1 566 decided, 959 proved, 71 provable only with empty
antecedents. The first version of the test passed with the binder
condition dropped from `keeps_antecedents`, since random derivations
rarely nest a division whose body's other hypothesis is bound outside
or inside it; the four fixed cases (`e ⊢ ((c \ (c ⊗ a)) / a) ⊗ e`,
`e ⊢ e ⊗ (c \ (a \ (a ⊗ c)))`, `e ⊢ e ⊗ (a \ a)`, and the `L` sequent
`e ⊢ (e ⊗ (c \ (c ⊗ a))) / a`) make that mutant fail it. The checker's
differential test (`agrees_with_the_first_implementation`) runs both
checkers on the proofs this search finds of generated `L` and `L*`
sequents and their mutants, in both modes.

### A defect found before the panel

The front door hands a goal to the net engine when the dispatched
engine ends at its memory bound and the linking is forced. In the Lambek
modes the net engine admitted the goal and searched planar Danos–Regnier
nets without the antecedent condition, so after the essential closure
met the memory bound a strict-`L` search could return an `L*`-only proof,
which the front door's check then refuses. Fixed: the net engine
refuses the Lambek modes; `lambek_has_no_handover` failed before the
fix (the engine reported was `net`) and passes after it.

### The review

Step 26's panel, one round, each member a fresh context on a frozen tree
(the jj workspace `../linlog-step36b-panel` at `knrnokul`, git
`8303fa14`, the head of the code before the fixes below; the base in
`../linlog-step36b-base` at `xmqspvzu`), briefed with conduct.md's rules
on cores and memory (cores 4 to 7, scopes capped at 6 GiB, four jobs at
most, bounded enumerations, nothing over ten minutes, no run the brief
does not name) and told to report only what breaks soundness,
completeness or a stated claim. The common brief (in the session's
scratchpad, `panel-common.md`) stated six claims: the reading of the
calculus, the checker, planar essential nets as `L*`'s nets, the
equivalence of the antecedent condition with every and some
sequentialization keeping antecedents, the soundness of the search's
rejections, and the commutative and cyclic modes unchanged.

- *Argument* (Fable 5.1 at `high`): nothing breaks; every claim holds.
  It proved claim 4 both ways by hand and by enumeration: 400 000
  random Lambek sequents, the 42 377 balanced ones with every planar
  linking enumerated, 27 683 planar essential nets, each against its own
  computation of the condition, the checker after sequentialization in
  `L` and `L*`, and its own enumeration of every cut-free `L*`
  derivation with its linking (51 308): the linkings of `L*` derivations
  are exactly the planar essential nets, and for each net the condition,
  some derivation keeping antecedents and every one doing so coincide
  (3 310 linkings had several derivations); the engine's verdicts agreed
  on every sequent, and 202 623 goals off the roots agreed with its own
  decider. It read `keeps_antecedents` as the condition verbatim, the
  rejections of `Planar<Essential>` as sound for every partial linking,
  and compared 22 670 commutative and cyclic runs with the base byte for
  byte. Two notes on the text of the proof, not on the claim: "no edge
  enters a sub-net from outside" holds for a sub-net whose one
  conclusion is an output, not for every sub-net, and the converse
  needed the acyclicity step (the body reaches the binder). *Answer*:
  both taken into the proof above.
- *Counterexamples* (Opus 5.5 at `high`): nothing breaks soundness or the
  commutative and cyclic modes. With its own cut-free decider and
  checker of `L` and `L*`: 594 000 random sequents in both calculi and
  every sequent of up to six connectives over one or two atoms (about
  55 million, in both), no verdict differing; 1.4 million proofs through
  its checker and `check`; the checker against its own on 1.5 million
  terms, 4.9 million mutants and a million random terms; 11.9 million
  goals off the roots; 33 600 sessions with 13.9 million steps (accepted
  exactly as its model accepts them, 490 000 refused in `L` as empty
  antecedents), 77 000 closes and 28 000 finished proofs; every engine
  but the essential one refused in the Lambek modes; and 2.1 million
  commutative and cyclic runs identical to the base in verdict, engine,
  every counter and the proof. One finding: `prove_goal` in `L` refuses
  a goal off the roots of a sequent that has no hypothesis itself
  (`a ⊢ a` within `⊢ a \ a`) as `NotIntuitionistic`, where the goal is
  provable. *Answer*: kept. `⊢ a \ a` is no sequent of `L`; its reading
  in the mode is refused, and so is every question about it (a session,
  a check of a proof of one of its goals, which reads the forest the
  same way), as the documentation of `prove_goal` says by naming the
  errors of `prove`; `L*` answers it. An observation: mixed divisions
  associate to the right (`a / b \ c` is `a / (b \ c)`), which no
  documentation said. *Answer*: the syntax documentation on `Sequent`
  says so now.
- *Integers and limits* (Sonnet 5.5 at `high`): nothing breaks
  soundness or a limit. It checked the index arithmetic of `hypotheses`
  and `written` on every rotation and every subset of roots (13 798
  subsets), the label tables, the exactness of `gamma.len` where the
  checker's rule reads it, the charge of the new tables (peak heap
  against the bound measured with a counting allocator: no overshoot
  beyond the commutative essential engine's), 2 724 stopped and about
  8 000 memory-bound runs on the Lambek path (every one `Unknown` or the
  unbounded verdict), the cost of the unpolled `keeps_antecedents` (at
  most microseconds on 12 800 hypotheses or 10 000 nested divisions),
  the front door's refusals and the wire level of every document of the
  new modes (level 2, which is not released). One finding:
  `ShapeError::Hypothesis` under the ordered reading named the last
  written formula that cannot be a hypothesis, not the first, as the
  commutative reading does (`~a, b, ~c |- d`: index 2, not 0).
  *Answer*: fixed (`rposition`, and the test
  `the_first_written_hypothesis_is_named`). It also noted that the
  planar regions and the loop's tables are not charged, as for the
  essential engine before: not changed, as conduct.md asks of a
  pre-existing gap.

**Round 2** (tree `../linlog-step36b-panel2` at git `2eb9cffd`, the
session's final code before `planning` stacked it on `main`; brief
`panel-round2.md` in the session's scratchpad): the changes after round
1's tree, which are the error index, the net engine's refusal of the
Lambek modes, the `let … else` in `is_essential` (a lint), and
documentation.

- *Argument* (Fable 5.1 at `high`): nothing breaks. The refusal's
  predicate names exactly the two Lambek modes among those `Mode::check`
  admits; every path that answers for them (the dispatch, `prepare`, the
  handover, `race`, goals off the roots, a session's `close`, the batch,
  the command's and the bindings' net drawings) reaches no search or net
  judgement without the antecedent condition; the commutative handover
  is intact (a 40-link chain at 2 000 bytes still goes to the net engine
  in classical and intuitionistic mode); `is_essential` visits the same
  implications in the same order with the same polls; the index is the
  first written offender, as the commutative reading's.
- *Counterexamples* (Opus 5.5 at `high`): nothing breaks. 3.06 million
  runs of forced-linking `L` and `L*` sequents under memory bounds from
  16 bytes to 1 MiB, on the roots and on goals off them, each `Unknown`
  at the bound or the right verdict, the engine always the essential one,
  3.4 million proofs through its checker and `check`; the same runs on
  round 1's code reproduce the defect (2 013 handed to the net engine);
  every other engine refused on 40 000 forced runs; 25 880 commutative
  and cyclic runs byte-identical to the base, 690 handovers among them;
  1 440 `Hypothesis` errors each naming the first refused formula;
  422 780 structures judged by `is_essential` as at round 1's tree.
- *Integers and limits* (Sonnet 5.5 at `high`): nothing breaks. The
  index's arithmetic on every hypothesis list of up to five formulas
  (2 730 readings), 2 340 runs over memory bounds, recursion depths and
  work limits in the Lambek modes, each `Ok` with the essential engine
  and its verdict or the bound's `Unknown`, and the polls of
  `is_essential`'s loop.

Both rounds noted that a sequent read by the commutative parse and then
proved in a Lambek mode counts `Hypothesis`'s index in the mirrored
order; not changed: the ordered modes read text through the ordered
parse everywhere (the command, the harness), and a library caller who
mixes the two parses asks the reading of another sequent.

### The target sets

Both at the code of the session, in the workspace `../linlog-step36b-measure`
at `knrnokul` (whose later fixes touch no commutative search: the net
engine's refusal of the Lambek modes, a lint, an error index),
`bench/net-targets.sh step36b` on core 2 and `bench/targets.sh step36b`
on cores 2 and 3, one unit under the machine lock (09:07 to 09:22). Not
kept as files: they equal the oracles, as `bench/TARGETS.md` now records.

- **Net set**: `linlog-bench summary --against
  bench/targets/net-after-cyclic.csv bench/targets/net-step36b.csv
  --counters`: no verdict differs, and "of the 64 problems both decide
  alike on one thread, 0 differ in nodes, splits, memo_hits,
  memo_entries, links, tests".
- **Focused set**: `--against bench/targets/after-cyclic.csv`: no verdict
  differs, nothing decided by one alone; "of the 145 problems both decide
  alike on one thread, 1 differ": `mix/11` (nodes 49 947 692 against
  48 748 043, splits and memo hits likewise). It ends at the 300 s time
  limit and is refuted by the classical refutation after it (late), so
  its counters are how far the search got in 300 s on the cores of the
  day: `after-classical` has 41 450 894 nodes on the same row. The 144
  rows decided within the limit keep every counter. At equal work the
  row keeps them too: the family's instance under one work limit
  (`Limits::with_work`), built at the base `xmqspvzu` and at the
  session's code, gives at 2·10⁷ units 172 326 nodes, 148 127 memo hits,
  24 187 entries and 17 152 523 splits in both, and at 5·10⁹ units
  19 354 846 nodes, 17 397 466 memo hits, 1 048 576 entries and
  4 680 221 742 splits in both, every counter equal (two scratch
  programs, `mix11/` in the session's scratchpad), each then refuted by
  the classical refutation. **So "every counter kept" cannot hold for
  this row, at any code**: its counters at the time limit differ between
  two runs of one binary. Run twice at the session's code with the
  target set's settings (the harness's `run --family mix=11 --jobs 1
  --timeout 300`, cores 2 and 3 at once, under the machine lock) it gave
  49 971 253 and 48 978 201 nodes, 44 945 783 and 44 047 410 memo hits,
  12 796 647 197 and 12 484 886 143 splits. The requirement holds for
  every row whose search ends within its limit, and for this one at
  equal work.
- **The ratchet** (the gate's journeys under callgrind; the base counted
  at `xmqspvzu` in this session): the divisions in the parser cost the
  commutative reader `read-text` 12 418 964 → 12 468 102 instructions
  (+0.40 %) and `read-lltp` 27 200 282 → 27 262 280 (+0.23 %), both
  within their ceilings' tolerance; the counts of the last commit are in
  the commits' table below.

### The measurement

By 36a's method: the harness at the session's code (release), one
thread, two shards on cores 2 and 3 under the machine lock (the unit
above, 09:22 to 09:31), 60 s per run, runs under 2 s three times
(median), on the new problem file `bench/problems/lambek.txt`, each
problem in its own mode (`lambek`, or `lambek-star` where named; the
dispatch gives the essential engine, the net engine refused there) and in
the classical mode on the same one-sided sequent (the focused engine by
the dispatch, and the classical net search forced). The Lambek search has
no rival in its mode; the comparison answers another question
(provability with exchange). Every verdict is the one expected, every
proof checked.

| row | Lambek (essential engine) | focused engine (classical) | classical net search |
|---|---|---|---|
| chain `a₀, a₀ \ a₁, … ⊢ aₙ`, 1 024 / 4 096 links | 59 ms / 2.8 s | 51 ms / 0.57 s | 27 ms / 0.43 s |
| the chain written with `/`, 1 024 / 4 096 | 23 ms / 0.39 s | 37 ms / 0.49 s | 27 ms / 0.44 s |
| the chain with its first two hypotheses exchanged | refuted, 8 ms / 88 ms | proved, 52 ms / 0.57 s | proved, 26 ms / 0.43 s |
| product `a₀, …, aₙ ⊢ a₀ ⊗ … ⊗ aₙ`, 1 024 / 4 096 | 32 ms / 0.66 s | 3 ms / 40 ms | 38 ms / 0.63 s |
| its goal reversed | refuted, 12 ms / 0.16 s | proved, 3 ms / 40 ms | proved, 36 ms / 0.63 s |
| adjectives `np / n, n / n, …, n, np \ s ⊢ s`, 64 / 256 | 1.0 ms / 52 ms | 0.18 ms / 1.1 ms | 1.1 ms / 52 ms |
| `x ⊢ x ⊗ ((a₀ \ a₀) ⊗ …)`, 16 to 256 factors | `L`: refuted, 14 to 81 ms; `L*`: proved, 0.05 to 3 ms | proved, 0.06 to 1.2 ms | proved, 0.07 to 3.4 ms |
| random derivations of `L` over two atoms, 20 | all proved, at most 4.7 ms (9.8 ms in all) | all proved, up to 3.5 s (3.5 s in all) | 12 proved, 8 past 60 s |
| the same with two neighbouring hypotheses exchanged, 20 | 5 proved, 15 refuted, at most 0.4 ms (1.4 ms in all) | 20 proved, up to 3.5 s | 12 proved, 8 past 60 s |

*Reading*: where the linking is forced, the search costs what the net
searches cost but on the chain of `\`, 2.8 s against 0.4 s with `/`
(not profiled: a link ORs a row of the closure into every row that
reaches its input literal, so the order in which the chain's links are
made decides how many rows each widens, which is the likely cause). The exchanged and reversed forms, provable only with exchange, are
refuted at once (the planar span balance). On random derivations the
planarity and the directed rejections cut nearly every choice: the
search decides all forty in under 5 ms each, where the classical net
search on the same sequents runs past the minute on 16 of 40 and the
focused engine takes up to 3.5 s. Without empty antecedents a sequent
whose only nets need one costs a search through every planar essential
net, each tested at the end (14 to 81 ms here), where `L*` proves it at
once; a prune of partial linkings by the antecedent condition is not
built (no row needs it).

The script that wrote the problem file:

```python
# Writes bench/problems/lambek.txt: scalable sequents of the Lambek calculus.
import random, sys
out = []
def add(name, mode, expected, seq):
    out.append(f"{name}; {mode}; {expected}; -; {seq}")
for n in [16, 64, 256, 1024, 4096]:
    under = ["a0"] + [f"a{i} \\ a{i+1}" for i in range(n)]
    add(f"lambek-chain/{n}", "lambek", "provable", ", ".join(under) + f" |- a{n}")
    swapped = [under[1], under[0]] + under[2:]
    add(f"lambek-chain-exchanged/{n}", "lambek", "unprovable", ", ".join(swapped) + f" |- a{n}")
    over = [f"a{i+1} / a{i}" for i in reversed(range(n))] + ["a0"]
    add(f"lambek-chain-over/{n}", "lambek", "provable", ", ".join(over) + f" |- a{n}")
    prod = " * ".join(f"a{i}" for i in range(n))
    add(f"lambek-product/{n}", "lambek", "provable", ", ".join(f"a{i}" for i in range(n)) + f" |- {prod}")
    rev = " * ".join(f"a{i}" for i in reversed(range(n)))
    add(f"lambek-product-reversed/{n}", "lambek", "unprovable", ", ".join(f"a{i}" for i in range(n)) + f" |- {rev}")
for k in [4, 16, 64, 256]:
    # A noun phrase with k adjectives: np / n, n / n (k times), n, np \ s |- s.
    add(f"lambek-adjectives/{k}", "lambek", "provable", "np / n, " + "n / n, " * k + "n, np \\ s |- s")
    units = " * ".join(f"(a{i} \\ a{i})" for i in range(k))
    add(f"lambek-empty/{k}", "lambek", "unprovable", f"x |- x * ({units})")
    add(f"lambek-empty/{k}", "lambek-star", "provable", f"x |- x * ({units})")
def derivation(rng, atoms, budget):
    """A random derivation of L: its hypotheses in order and its goal."""
    if budget <= 1 or rng.random() < 0.05:
        a = rng.choice(atoms)
        return [a], a
    r = rng.randrange(5)
    if r == 0:
        g, a = derivation(rng, atoms, budget // 2)
        d, c = derivation(rng, atoms, budget - budget // 2)
        return g + d, f"({a} * {c})"
    if r == 1:
        g, c = derivation(rng, atoms, budget - 1)
        if len(g) >= 2:
            i = rng.randrange(len(g) - 1)
            g[i:i + 2] = [f"({g[i]} * {g[i + 1]})"]
        return g, c
    if r == 2:
        g, c = derivation(rng, atoms, budget - 1)
        if len(g) < 2:
            return g, c
        if rng.random() < 0.5:
            return g[1:], f"({g[0]} \\ {c})"
        return g[:-1], f"({c} / {g[-1]})"
    g, a = derivation(rng, atoms, budget // 2)
    d, c = derivation(rng, atoms, budget - budget // 2)
    k = rng.randrange(len(d))
    b = d[k]
    middle = g + [f"({a} \\ {b})"] if rng.random() < 0.5 else [f"({b} / {a})"] + g
    return d[:k] + middle + d[k + 1:], c
rng = random.Random(36)
for budget in [40, 80, 160, 320]:
    for k in range(5):
        g, c = derivation(rng, ["a", "b"], budget)
        add(f"lambek-random/{budget}-{k}", "lambek", "provable", ", ".join(g) + f" |- {c}")
        if len(g) >= 2:
            i = rng.randrange(len(g) - 1)
            g[i], g[i + 1] = g[i + 1], g[i]
        add(f"lambek-random-exchanged/{budget}-{k}", "lambek", "-", ", ".join(g) + f" |- {c}")
head = """# Sequents of the Lambek calculus, read in its order (as cyclic.txt's), one
# per line as in slow-tests.txt: name; mode; expected; copies; sequent.
# chain: a0, a0 \\ a1, ... |- an, and chain-over the same with /, provable;
# chain-exchanged: its first two formulas exchanged, provable only
# commutatively; product: a0, ..., an |- a0 * ... * an, and
# product-reversed its goal reversed, provable only commutatively;
# adjectives: np / n, n / n (k times), n, np \\ s |- s; empty: x |- x * a
# product of k formulas a \\ a, provable only with empty antecedents
# (lambek-star); random: random derivations of the calculus over two
# atoms with up to the budget of rules, and random-exchanged: each with
# two neighbouring hypotheses exchanged.
"""
open(sys.argv[1], "w").write(head + "\n".join(out) + "\n")
```

### Commits

In the workspace `../linlog-step36b` on `xmqspvzu` (the head of `main`
with 36a reviewed), every code commit gated at that commit in the
workspace `../linlog-step36b-gate` (clippy, the tests with the
behaviour lock, both `cargo hack` runs, the ratchet), after the fixes
of the review were folded into the commits they belong to:

| change | subject | gate |
|---|---|---|
| `mtvxowll` | Read the Lambek calculus's divisions in the ordered parse | passed (`read-text` 12 468 102) |
| `qpuznluv` | Add the Lambek calculus: its mode, reading, checker and rule names | passed |
| `pysqxtkl` | Search planar essential nets: the Lambek calculus's engine | passed |
| `lowoqurz` | Prove step by step in the Lambek calculus | passed |
| `orpxumkp` | Export the Lambek calculus, and refuse its Rocq certificates | passed |
| `knrnokul` | Add --lambek to the command (the help's lock reblessed) | passed |
| `wwowwnyz` | Add the Lambek calculus's problem file | passed |
| `lqnvqkkr` | Document the Lambek search's measurement; record the target sets | passed |
| (this one) | Report session 36b; the rules, the changelog | documentation |

The journeys at the last code commit against the base (`xmqspvzu`,
counted in this session): the parser's divisions `read-text` +0.40 %
and `read-lltp` +0.23 %; the checker's test of the mode per node and the
reading moved every journey that checks proofs, in the commit that adds
them, by +0.4 to +0.65 % (`search-chain-128` 958 770 441 → 964 956 347,
`search-chain-64-intuitionistic` +0.63 %, `check-qbf-20-2` +0.56 %,
`check-wide-m1-2048` +0.46 %, `search-spec-chain` +0.26 %,
`derivation-chain-64` +0.17 %, `search-partition-no-5` +0.16 %,
`ordinary-pigeons` +0.11 %); `render-svg` −0.40 %; every other journey
within 0.07 %; every count within its ceiling's tolerance. Beside the
gates: the families' verdicts (`linlog-bench run --all-families
--timeout 5`, as the table asks for a change of `bench/`): no mismatch.
`nix flake check` runs at the last commit, under the machine lock; its
exit status is in the last message to `planning`.

### What is left open

- **A criterion of `L` on structures**: `ProofStructure` judges planar
  essential nets of `L*`; a client that judges a net it did not find in
  `L` (none does today) needs `keeps_antecedents` as a public criterion.
- **`keeps_antecedents` is unpolled**: implications × sources bit tests
  per complete linking, a few milliseconds on the largest measured rows.
- **The premise order of `\L`** in the drawings (above, decisions).
- **The embedding into first-order MILL** (Moot and Piazza) is weighed
  against this search once step 38 exists, as the step prompt says.
- **Certificates**: the Rocq library has no non-commutative calculus.
