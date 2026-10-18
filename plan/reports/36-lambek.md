# Step 36: cyclic MLL and the Lambek calculus

## The checklist

| item | state | evidence |
|---|---|---|
| Session 36a: the order kept, and cyclic MLL (session `step-36a`, 2026-10-19) | done (2026-10-19) | The cyclic mode (`Mode::CYCLIC`, `Order`), the ordered parse, the checker's arcs, the planar criterion (`ProofStructure::is_planar`), the planar search as the mode's default (`search/planar.rs`), the session's arc splits, the exports and `--cyclic`; the commutative modes' sequents, JSON forms and snapshots unchanged (behaviour lock passing, its help texts reblessed for the flag). `agrees_with_every_planar_linking` (the engine against every planar linking, every proof checked) and `planarity` (every linking against a pairwise test) pass. 35a's prunes: the skeleton balance kept, the three symmetry breaks off (argument and counterexamples above). Panel in two rounds (Fable 5.1 argument, Opus 5.5 counterexamples, Sonnet 5.5 integers, each at `high` on a frozen tree): four defects found and fixed, nothing of the arguments refuted. Target sets: `net-after-prunes` every counter on 64 rows, `after-efficiency` every counter but the five recorded rows. Measurement on `bench/problems/cyclic.txt` (cores 2 and 3). Every code commit gated; `nix flake check` at the last commit (exit status in the message to `planning`). |
| Session 36b: the Lambek calculus | to do | |

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
