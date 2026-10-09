# Where the code assumes no boxes, no exponential links and no cut

Written 2026-10-08 for stage 2 of step 28, from the snapshot's code
(`core/src`, `cli/src`, `bench/src`, the tests and snapshots), read for
what steps 33 (MELL nets with boxes) and 34 (cut, and its elimination)
would have to change. It builds on `33-mell-nets.md` ("33 §n"),
`34-cut.md` ("34 §n") and `README.md` of this directory ("README §n")
and does not repeat their surveys: where a place is already an item of
the README, it says so and adds only what the code shows. Paths are
under `core/src/` unless they name another crate. "(inference)" marks a
conclusion that is not read off the code; nothing here was built or run.

Each place gives what it assumes, what the step needs there, and one of
two recommendations: **open now** (cheap now, and after 0.1.0 a
breaking change or a silent wrong answer) or **at the step** (local,
and as cheap when step 33 or 34 comes). Section 11 ranks the ten
open-now items that would cost most to leave.

## 0. Three assumptions that run through the code

1. **A net's vertex is a forest occurrence.** `ProofStructure`, its
   coloured graph, skeleton, scratch, errors, JSON, text form and
   drawing all index by `OccId`, and a structure is "every tree of the
   forest plus axiom links". An MELL net has one vertex per *instance*
   of an occurrence, plus `?` nodes of any arity, doors, box nodes and
   jumps (33 §2); a cut is a link between two roots that is no axiom
   (34 §2).
2. **The forest is exactly the conclusion.** `Forest::roots()` is the
   conclusion (the checker's goal, the derivation's root, the net's
   conclusions, the reading's goal candidates), and `ids()`, `len()`,
   `all_literals()` and `sequent()` mean at once "everything under the
   roots", "everything the ids index" and "the sequent proved". Step 34
   appends `A`, `A⊥` per cut after the roots (34 §2), which pulls these
   three meanings apart.
3. **A node is one of fourteen cut-free rules, and "the rest" has a
   default.** The node matches in `proofs/` are exhaustive, so `Cut`
   will be a compile error there; but a dozen places beside them route
   every other rule or kind to a default through `_ =>` or `matches!`:
   an axiom slot, a `⊗` symbol, a Mix node, a panic.

## 1. Data model: types, layout, sizes, indices

- **`occurrences/mod.rs:16-19` `OccId(u32)`**, the only index the net
  API speaks. Assumes vertex = occurrence. 33 needs vertex ids of the
  net's own, not `(OccId, NodeId)` pairs, because 34 duplicates boxes
  (33 §3). **Open now** (item 2).
- **`occurrences/mod.rs:155-180, 202-239, 387-479` `Forest`**: fields
  per occurrence, built only from a `Sequent` (`within`, `from_owned`,
  `TryFrom`, `build` walking `sequent.roots()`). Assumes the trees are
  the sequent's roots and nothing else. 34 needs extra trees after the
  roots, `cut_pairs()`, `dual(o)` by offset, `roots()` still the
  conclusion (34 §2-3). **Open now** for the contract (item 1); the
  constructor at the step.
- **`occurrences/mod.rs:242-244` `Forest::sequent()`, `proofs/mod.rs:377`
  `Proof::sequent()`, `nets/mod.rs:298` `ProofStructure::sequent()`.**
  One value serves as the arena (atom count and terms:
  `search/focus/bias.rs:17`, `search/focus/counts.rs:202`,
  `search/focus/classes.rs:35`, `search/horn/mod.rs:198, 292`,
  `search/net.rs:164`, `search/mod.rs:635`, `export/notation.rs:108,
  166`, `export/rocq.rs:304, 429, 560`, `bench/src/run.rs:826`) and as
  the sequent proved (`serialize/proofs.rs:123`, `serialize/nets.rs:26`,
  `nets/mod.rs:203, 435`, `export/svg/net.rs:360`,
  `search/focus/schedule.rs:191`, `cli/src/prove.rs:1297-1320`). A cut
  formula may name atoms the sequent lacks (inference), so the arena
  must hold terms and atoms the conclusion does not reach. This already
  fits the types: `Sequent::fragment` counts only terms reachable from
  the roots (`fragment.rs:197-213`), JSON reading keeps unreachable terms
  and merges atoms by name without sorting roots
  (`serialize/sequents.rs:124-132`), and `Sequent::add` appends roots and
  keeps the first sequent's atom indices (`sequents/mod.rs:361-381`).
  **Open now** (item 1): say which of the two `sequent()` is.
- **`occurrences/mod.rs:191-195, 228-239` `DEFAULT_LIMIT`, `MOST`**
  count the roots' occurrences. 34: the sum with the cut trees. 33: an
  unfolded term has more instances than occurrences (a `Copy` repeats a
  subtree), so a net needs a vertex limit of its own, and more than
  2³² − 2 instances must be refused, not wrapped. At the step for the
  forest sum; the net's bound is item 4.
- **`nets/mod.rs:182-197` `ProofStructure { forest, mix, partner:
  Box<[u32]> per occurrence, links: Vec<(OccId, OccId)>, graph,
  skeleton }`.** Assumes occurrences as vertices, axiom links only, a
  boolean criterion. 33: vertex table, `?` nodes, doors, box nodes,
  jumps; 34: cut links, an `erased` set (34 §2). **Open now** for the
  one-type-or-two decision (item 2); the fields at the step.
- **`nets/graph.rs:32-44, 133-156` the CSR graph**: per vertex the
  parent slot first (none for a root, L140-142), the children from
  `forest.children(o)`, an axiom slot last; `switched_tree_edges` counts
  two per `⊗`, one per `⅋`. Assumes binary connectives and slots fixed
  by the forest. 33: a `?` vertex with n ≥ 0 premise slots (instances,
  not the one child occurrence), door, box and jump edges, a tree edge
  that stops at a box; 34: a cut slot on a cut root and one switched
  edge per cut (a cut counts as a tensor, 34 §2). At the step
  (internal).
- **`nets/graph.rs:52-74` `Scratch`** holds the vertex sets `deleted`
  and `stuck` as `OccSet`s of the forest's width. At the step; with
  item 2 they become sets over vertices. Its fields are private, so the
  public type need not change.
- **`nets/skeleton.rs:14-95` union-find with an undo log**, no
  deletion; `nets/mod.rs:365-372` `unlink()` pops the last link only.
  Assumes links come and go in stack order (the net engine's
  backtracking). 34's elimination removes arbitrary links and erases
  vertices ("a step is a diff", 34 §2). At the step: elimination
  rebuilds per step or keeps its own representation; promise no O(1)
  arbitrary removal on `ProofStructure` (inference).
- **`proofs/mod.rs:117-158` `Node`**, fourteen variants, 16 bytes
  asserted. 34's `Cut(OccId, NodeId, NodeId)` fits (34 §2); 33 needs no
  node (boxes are `Bang`, instances `(OccId, origin Copy)`, 33 §3).
  README §2 ("`Node` stays 16 bytes") and §3.1 cover the policy. At the
  step.
- **`proofs/mod.rs:256-260` "a subproof that two nodes share is stored
  once".** Desequentializing an MELL term must unfold sharing per path,
  since each path through a shared subproof is its own set of instances,
  and the unfolding can be exponential in the arena (inference; the
  checker's note on a Mix of a subproof with itself,
  `proofs/check.rs:1069-1075`, is the doubling case). **Open now** (item 4: `from_proof` takes a
  bound).
- **`lib.rs:138` `pub use nets::{…, Scratch}`**: `linlog::Scratch` is
  the net graph's scratch. A second net type or the elimination's
  working memory would make the root name ambiguous. Open now, cheap:
  keep it as `nets::Scratch` only (D18 allows the rename before 0.1.0).
- **`fragment.rs:31-42`**: `Fragment::MELL` includes `1` and `⊥`; there
  is no constant for unit-free MELL. 33 decides whether nets take units
  (jumps, 33 §2). At the step (constants are additive).

## 2. Matches and visitors over formula and link kinds

Formula kinds do not grow at 33 or 34 (`!` and `?` exist; a cut adds no
kind), so the exhaustive matches over `Kind` need nothing:
`Reading::new` (`occurrences/reading.rs:206-213`), `Interactive::rules`
(`proofs/interactive.rs:501-513`), `Kind::dual`, `arity`, `fragment`,
`polarity`. The matches over `Node` and `Rule` that are exhaustive make
`Cut` a compile error, which is wanted: `Node::principal`,
`occurrences`, `premises`, `name`, `map_premises`
(`proofs/mod.rs:163-230, 463-478`), `Pass::rule` (`proofs/check.rs:1156-1305`),
the oracle (`proofs/oracle.rs:262-391`), `Build::unfold`
(`proofs/derivation.rs:1211-1289`), the size estimate
(`proofs/size.rs:343-490, 498-512`), the JSON tags
(`serialize/proofs.rs:13-115`), the Rocq rule match
(`export/rocq.rs:386-469`), the label tables (`proofs/style.rs:47-57`,
whose length is `Rule::ALL.len()`). These are not exhaustive, and a new
variant or kind falls through them silently:

- **`nets/graph.rs:144-148`** `match kind { Tensor => 2, Par => 1, _ =>
  axiom slot }`: `!`, `?`, `1` and `⊥` are laid out as literals. 33.
  Open now (item 7): make the match full.
- **`nets/graph.rs:273-287` `deletable`**: only `Kind::Par` is switched.
  33: a `?` vertex with two premises or more is switched like `⅋`
  (33 §2); a box node is not. At the step.
- **`nets/graph.rs:377-404` `parts`** and **`export/svg/net.rs:62-86`
  `components`**: "the switching that keeps the left premise of every
  `⅋`". 33: also the first premise of every `?` node. At the step.
- **`nets/sequentialize.rs:73-83, 137-214`** `Step { Prove, Mix,
  Tensor, Pars }`; `stage` opens `⅋` only (L148), makes an axiom of two
  literals (L181-185) and otherwise `expect`s a splitting `⊗`
  (L189-195). 33: `⊥`/weakening removed first, `?` opened, a box proved
  as `Bang` then its content (33 §2); 34: a splitting cut. At the step.
- **`nets/mod.rs:249-261` `from_proof`**: `filter_map(Ax => Some, _ =>
  None)`. **Open now** (item 5).
- **`search/net.rs:140-148` `counts_admit`** `match kind { Tensor, Par,
  _ => {} }` over every id. The search stays cut-free, but see §5.
- **`export/svg/net.rs:207-212`** gives a layer and an x only to
  occurrences with two children: a `!` or `?` would sit at layer 0,
  x 0. **`:289-292`** `match kind { Par => "⅋", _ => "⊗" }`: a `!` or
  `?` would be drawn as `⊗`. **`:243-259`** `match verdict { cycle,
  parts, _ => {} }`: a new `NetError` draws no highlight. At the step,
  but the symbol match is item 7.
- **`proofs/derivation.rs:878-899` `Record::new`** keeps the standard
  sequents a split needs for `matches!(Tensor | Mix)` only; a cut splits
  its context like `⊗`, and a forgotten arm panics at `standard(id)`
  (`self.record.standard[&id]`, L1008-1010). `split` (L1296-1372) takes
  `tensor: Option<(OccId, OccId, OccId)>`, two shapes; a cut is a third
  (removes `A` on the left, `A⊥` on the right, adds nothing). Item 7 for
  the `matches!`; `split` at the step.
- **`proofs/interactive.rs`**: `replay` `has_principal = !matches!(Ax |
  Mix)` (L353) and the split `matches!(Tensor | Mix)` (L368); `apply`
  `principal = (rule != Ax && rule != Mix)` (L564); `expand` `acts`
  ends in `_ => unreachable!` (L619), `splits = matches!(Tensor | Mix)`
  (L643), the premises match ends in `_ => unreachable!` (L727);
  `Terms::term` leaf `_ => Node::Top` (L1054), `introduced` `_ =>
  unreachable!` (L1072), the two-premise build `_ => Node::Mix(l, r)`
  (L1095) and `_ => unreachable!` (L1098). Each compiles with `Rule::Cut`
  added and then panics, refuses as inconsistent, or builds a Mix node.
  **Open now** (item 7).
- **`cli/src/interact.rs:188-192`** marks `⊗` and Mix "(with a split)"
  only. Item 7.
- **`export/rocq.rs:552-557`** refuses Open, Mix and affine weakening
  and lets `_ => {}` through; the rule match below is exhaustive
  (L386-469), so a forgotten refusal is a compile error there. At the
  step.
- **`proofs/derivation.rs:230-279` `Rule::classical`, `intuitionistic`**
  map any other rule to itself, which is right for `Cut`;
  **`:308-347` `from_str`** ends in `_ => Err`, which README §2 ("`Rule`
  grows by one checklist") already covers with a round trip of
  `Rule::ALL`.
- **Tests**: `nets/graph.rs:458-468` brute-force switchings `_ =>
  unreachable!()` on a parent's kind; `proofs/check.rs:1360-1411` the
  mutants never make a `Cut`. At the step.

## 3. Invariants: numbering, sets, counts, orderings, hashing

- **Preorder numbering and the dual by offset**
  (`occurrences/mod.rs:112-126, 328-356`). 34 relies on `A⊥`'s tree
  following `A`'s with the same shape, so that `dual(x) = root(A⊥) +
  (x − root(A))` (34 §2); `Term::dual` keeps a node's children in order
  (`sequents/term.rs:176-192`), so dualizing node by node gives the
  shape (inference, as 34 §2). `Sequent::optimize` sorts roots
  (`sequents/mod.rs:204-205, 314-320`) and must never run on the
  extended arena. README §2 covers the numbering as a promise and the
  root order. Open now as part of item 1.
- **"The roots' subtrees partition the ids"**: the forest test
  `check_invariants` (`occurrences/mod.rs:502-517`) and the per
  occurrence `root` (L166-167, 293-300): a cut tree's occurrences have a
  `root(o)` that is not in `roots()`. `lca` is `None` across trees
  (L361-370), which is what the net engine wants. Item 1 names a tree
  apart from a conclusion.
- **Literal tables** (`occurrences/mod.rs:173-179, 372-384`): grouped by
  atom and sign, ascending ids, so cut-tree literals follow the
  conclusion's in each group. `all_literals()` stands for "the literals
  to link" in `is_complete` (`nets/mod.rs:322-324`, `2 · links ==
  literals`), `unlinked` (L328-334), the net engine's `remaining`
  (`search/net.rs:229-231`), the dispatch's `few_equal_literals`
  (`search/mod.rs:633-645`) and the harness's `multiplicity`
  (`bench/src/run.rs:824-836`). 33: completeness is every literal
  *instance* linked; a literal under a `?` never copied has none. 34: a
  net with cuts links the cut trees' literals too. At the step.
- **The connectedness count** (`nets/mod.rs:419`,
  `nets/graph.rs:197-201`): switched edges + 1 = `forest.len()`. 33: per
  box content, over vertices; 34: one edge per cut, erased vertices out.
  At the step.
- **The net engine's equation** (`search/net.rs:136-160`): `c = t − p +
  2`, `t` and `p` over every id, `c` = `roots().len()`; right only when
  the forest is the conclusion. Item 10.
- **Occurrence sets** (`occurrences/set.rs:8-27, 343-351`): one width
  per forest, `root_set()` the conclusion. Extended forests widen the
  focused engine's contexts and memo keys without changing them
  (search stays cut-free). Nothing to do.
- **Orderings and determinism**: links in the order made, which is the
  JSON's order (`nets/mod.rs:189-192`, `serialize/nets.rs:28-33`); the
  text form sorted by first id (`nets/mod.rs:434-445`); parts by
  smallest vertex and the cycle from its first vertex
  (`nets/graph.rs:345-404`); the splitting `⊗` with the smallest id
  (`nets/sequentialize.rs:187-195`); "two proofs that differ by rule
  permutations give the same net" (`nets/mod.rs:240-248`), tested by
  comparing sorted links (L538-541, 657-666). For instances these hold
  only if vertex ids are a canonical function of the term up to rule
  permutation (for instance by occurrence, then by the instance's path
  of copies), not of arena order, and equality ignores jumps (33 §4).
  At the step, but the requirement belongs to the vertex-id decision
  (item 2).
- **Hashing**: `Node` derives `Hash` and `Eq` (`proofs/mod.rs:116`) for
  the arenas' sharing; a `Cut` hashes like any node. `ProofStructure`
  has no `PartialEq`; keep it so, since net equality is modulo jumps
  (33 §4).
- **Three scopes that coincide today**: the goal (`goal_fragment`,
  `search/mod.rs:613-621`, right for cut roots), the sequent
  (`nets/mod.rs:203-206`, `search/focus/schedule.rs:188-191`,
  `cli/src/prove.rs:949-959`) and the whole forest
  (`search/focus/bias.rs:26-28`, `search/net.rs:140-160, 226-252`,
  `search/mod.rs:633-645`). With cut trees, `ProofStructure::new` would
  admit an MLL sequent whose cut formula is additive, since it reads the
  conclusion's fragment; at the step, read the forest's kinds there. The
  schedule and the bias are speed heuristics only (inference).

## 4. The proof term and the checker

- **`proofs/mod.rs:4-16`**: "the rules of the dyadic sequent calculus
  every engine searches in". 34 adds a rule no engine searches in. Docs
  at the step.
- **`Node::principal`, `occurrences`** (`proofs/mod.rs:161-190`): "the
  occurrence the rule acts on: `None` for Mix". For `Cut(o, l, r)` it is
  `o`, the `A` root, which is in neither the conclusion nor
  `Inference.principal` (`proofs/derivation.rs:549-552`, `None` for a
  cut, 34 §3); `occurrences()` feeds `Proof::new`'s bound check
  (`proofs/mod.rs:336-340`) and the error display (`proofs/check.rs:554`)
  and cannot list `dual(o)` without the forest. At the step; one doc
  line.
- **`Proof::new(forest, nodes, root)`** (`proofs/mod.rs:322-369`). If the
  forest carries its cut pairs (item 1), `Proof::new` keeps its
  signature and a cut on an occurrence that is no cut root is a checker
  `Problem` (`proofs/check.rs:463`, already non-exhaustive). README §2
  asks that `Proof::new` take the cut formulas; one record in the
  forest gives the same with one source of truth (inference).
- **The checker's goal** (`proofs/check.rs:696-707`) is
  `proof.forest().roots()`; it must stay the conclusion (item 1).
- **The reading** (`proofs/check.rs:711-728` → `Reading::new`,
  `occurrences/reading.rs:219-276`): the goal is chosen among
  `roots()`, and the positions start from `Input` for every id and flow
  down from the roots. A cut root gets `Input`; 34 needs `A` in output
  and `A⊥` in input (34 §2). At the step, once item 1 says what `roots()`
  is; otherwise every intuitionistic cut proof fails with
  `Succedents` (34 §4).
- **`Pass::rule`** (`proofs/check.rs:1156-1305`): exhaustive, so the
  `Cut` arm is asked for; `take`, `join` and the `Surplus` bound
  (L1035-1060, 1144-1153, 1068-1088) hold for it (34 §2). `Facts`
  (L830-849) documents `shared`, `needs`, `left_goal` "for `⊗`, `&` and
  Mix": the derivation's contractions below a split read `shared`, so a
  cut sets them as `⊗` does. At the step.
- **The oracle** (`proofs/oracle.rs:57, 84, 99, 262-391`) mirrors every
  arm and the reading over `roots()`. README §2 ("Checker, oracle and
  Rocq mirror change together"). At the step.
- **33 needs nothing in the checker** (33 §3).
- **The derivation view**: §2 for `Record::new` and `split`; the size
  estimate is exhaustive. At the step.
- **Interactive proving** (`proofs/interactive.rs`): the forest is fixed
  at `new` (L198-232), and `apply(goal, position, rule, left)` (L538-583)
  addresses formulas of it, so a `cut(goal, formula, left)` must replace
  the forest by a larger one with the same prefix (34 §3), an additive
  method. `from_parts` requires inference 0 to conclude `roots()`
  (L258-262), which holds if `roots()` stays the conclusion. `proof()`
  puts the `?` nodes of `?` conclusions below the root (L962-966); a `?`
  cut root's node goes below the cut through `Terms`'s `Quests` step
  (L1040-1048) with `[[Some(A), None], [Some(A⊥), None]]`. Forest growth
  is append-only, so an `undo` of a cut leaves its trees (item 10). At
  the step, except the silent defaults (item 7).

## 5. The engines and the dispatch

- **Search stays cut-free, and MELL search stays with the focused
  engine** (plan 33, 34). No engine changes for either step.
- **`prove_goal` and `is_roots`** (`search/mod.rs:229-299, 375,
  390-401`): a goal equal to the roots sets `task.roots`, which makes the
  net engine eligible (`Feature::FewEqualLiterals`, L601-602;
  `Nets::admits`, `search/net.rs:50-60`) and the proof checked
  (L274-276). With cut trees in the forest, `Interactive::close` on a
  goal equal to the conclusion (`proofs/interactive.rs:816-829`) reaches
  the net engine on the whole forest. Example (inference from the code
  paths): `⊢ ~A, A` with cut trees `b ⊗ c`, `~b ⅋ ~c`; the goal's
  fragment is empty, so the dispatch picks the net engine; the equation
  holds (one `⊗`, one `⅋`, two conclusions), the atoms balance, the
  linking of all six literals is acyclic (the only cycle passes the `⅋`
  through both premises), and `answer` `expect`s that it sequentializes
  (`search/net.rs:121-124`), while `is_correct` counts 6 switched edges
  for 8 vertices where a tree has 7: `Disconnected`, and the `expect`
  panics. **Open now** (item 10).
- **`Answer.net`, `Outcome::net: Option<ProofStructure>`**
  (`search/mod.rs:428-439, 1205-1220`): only the net engine makes a net,
  and it makes MLL nets; MELL nets come from `from_proof`. Either answer
  of item 2 keeps the field. `Outcome` is non-exhaustive.
- **The net engine's hot path** (`link_unchecked`, `unlink`,
  `same_component`, `is_acyclic`, `partner`, `unlinked`; core-nets.md):
  a generalized structure must keep them at today's cost (33 §3), and
  nothing measures the net engine counter-exactly. Item 9.
- **`Engine::Net` docs** (`search/mod.rs:679-693`) and
  **`Error::NetFragment`, `NetMode`** (`errors/mod.rs:117-123`, "proof
  nets exist for MLL without units only", pinned in `README.md:102`).
  At the step.
- **Elimination is not an engine** (34 §3): no `Engine` variant, no
  `DISPATCH` row, nothing in `bench`'s `EngineChoice`.

## 6. JSON wire forms and their stability

- **Proof** (`serialize/proofs.rs:60-66`): `{"sequent", "proof"}`; the
  `Step` tags (L13-58) "are part of the interchange format"; reading
  rebuilds `Forest::try_from(p.sequent)` at the default limit
  (L132-142). 34: `"cuts"` and `{"cut": [o, l, r]}`, the forest rebuilt
  with the cut pairs in recorded order (34 §4, D5). The outcome
  flattens the proof's keys into its own (`serialize/search.rs:218-240`),
  so `cuts` must be skipped when empty for `prove --format json` to stay
  byte-identical (`README.md:504` pins one). Item 3.
- **Net** (`serialize/nets.rs:13-20, 37-52`): `{"sequent", "mix",
  "links"}`, links as occurrence pairs in the order made, read through
  `from_links`. 33: the vertex table, boxes, jumps, links over vertices,
  the MLL form unchanged (33 §3, D18); 34: cuts, and erased vertices
  during elimination. Items 2-4.
- **Interactive** (`serialize/interactive.rs:30-43, 72-73`):
  `{"sequent", "mode", "inferences", "history"}`. 34: the cut formulas in
  the order the steps added them, so `from_parts` rebuilds the same
  forest (34 §3). Item 3.
- **No proxy denies unknown keys** (only the options values carry
  `deny_unknown_fields`). A 0.1.0 reader drops a later `cuts` or
  `vertices` key without a word and then fails on an id beyond its
  forest (`NoOccurrence`, `OccurrenceIndexOutOfBounds`) or on an unknown
  tag: an error that does not name the cause. It drops a `version` key
  added later just the same. Item 3.
- **The cut formulas' form** (inference): a second value in `Sequent`'s
  form whose roots are the cut formulas and whose atoms merge with the
  sequent's by name, as `Sequent::add` does (`sequents/mod.rs:361-381`),
  keeps the `sequent` key byte-identical to today's for the same
  conclusion.
- **Pinned in tests**: `core/tests/serialize.rs:104` (proof), `:269`
  (outcome), `:322` (net), `:361` (interactive).

## 7. The exports

- **The net drawing** (`export/svg/net.rs`): `x` and `layer` per
  occurrence (L190-191); literals along the top in occurrence order
  (L194-206); layers for binary connectives only (L207-212); conclusions
  on the bottom layer (L213-219); links as arcs above the literals
  (L223-232, 333-344); the switching by `⅋` (L62-86); ids `o<n>` and
  `l<m>-<n>` (L276, 283, 339, documented in `export/svg/mod.rs:617-619`
  as "occurrence `n`"); the title the conclusion (L360). The estimate
  counts occurrences, roots and links (L138-174). 33: boxes as
  rectangles placed as a unit, doors, dashed jumps, a layout over
  instances (33 §2); 34: a cut as an arc below the conclusions, erased
  vertices skipped, one drawing per step (34 §3). The ids and the style
  are **open now** (item 8); the drawing at the step.
- **`svg::Style`** (`export/svg/mod.rs:58-130`): public fields, not
  non-exhaustive, serde `default` and `deny_unknown_fields`. 33 needs
  box padding, stroke, corner and door radius, box and jump colours,
  jumps on or off (33 §3); 34 a cut colour. README §2 asks for
  `#[non_exhaustive]`. Item 8.
- **`svg::net(&ProofStructure, &Style, Option<u64>)`**
  (`export/svg/mod.rs:626-634`): follows item 2.
- **PNG and PDF costs** (`export/mod.rs:140-160`) were measured on nets
  of links; rectangles and dashed jumps with heads cost otherwise.
  Re-measure at step 33.
- **Derivation exports** (LaTeX, Typst, SVG tree, text) are generic in
  the number of premises; a cut needs its label in both tables and its
  name (README §2 checklist). At the step.
- **Rocq** (`export/rocq.rs`): `Unsupported` (L107-130) is not
  non-exhaustive; the module doc (L20-28) lists what is refused;
  NanoYalla has no cut (34 §3). README §2 has the `Kernel` option.
  `#[non_exhaustive]` is item 6; the policy at the step.
- **Sequents printed two-sided** (`export/latex.rs:336-347`,
  `export/typst.rs:280-291`, `export/svg/mod.rs:535-556`) take
  `forest.roots()`: the conclusion, right under item 1.

## 8. The command

- **`--net`** (`cli/src/argument_parsing.rs:777-783`, help "for MLL
  without units"), formats text, svg, png and pdf only
  (`cli/src/prove.rs:217-224`). `nets_exist` (`prove.rs:949-959`)
  repeats the library's admission (MLL, not affine) and serves `prove`
  (L1003-1004), `check` (L1319-1320) and `batch`
  (`cli/src/batch.rs:518-519`); `net_into` (`prove.rs:908-944`) calls
  `from_proof(proof, mode.mix)` (L922); `net_too_large` (L889-898) counts
  "occurrences and links". 33: admit MELL, still refuse affine mode and
  additives (33 §3), count vertices and boxes; 34: a proof with cuts as
  a net. At the step; a library predicate the CLI asks would make it
  one place (additive, any time).
- **`interact`** (`argument_parsing.rs:56-71`, `cli/src/interact.rs:154-300`):
  `apply G P RULE [P…]` parses a `Rule`; a cut needs `cut G FORMULA
  [P…]` with the formula read against the sequent's atoms (34 §3). At
  the step.
- **`check`** (`prove.rs:1290-1337`) prints the conclusion through a
  forest of `proof.sequent()`, right under item 1; an elimination
  command is new (34 §3). At the step.
- **README's console blocks**, run by `cli/tests/readme.rs`: `--net`
  (`README.md:86-102`), the drawn net (650-730), "the certificate uses
  no cut" (772), the outcome JSON (504). At the step.

## 9. The harness and the families

- `bench/src/run.rs:158-185` `EngineChoice` mirrors `Engine`;
  elimination is no engine, so nothing changes.
- The CSV columns are the harness's interface (`run.rs:32-36`,
  bench.md); `links` and `tests` are the net engine's counters,
  `multiplicity` repeats the dispatch's threshold over the whole forest
  (`run.rs:824-836`). Families are sequents, so no extended forest
  reaches them.
- `families.rs` makes sequents only; 34's tests compose two searched
  proofs (34 §2), which needs a generator in `search/generate.rs`
  (test-only; `Rules`, L45-53, has units, additives, Mix and
  exponentials, no cuts). At the step.
- The target set measures the focused engine only (`bench/targets.sh:5-11`,
  bench.md). Item 9.
- `run.rs:636-641` classifies `NetFragment` and `NetMode` as `refused`;
  new net errors join the list at the step.
- Stress inputs for 33 (inference): the Horn engine's proofs of the
  Petri-net suites, thousands of copies each, for desequentialization
  and its bound, beside README §2's large nets for `check_within`.

## 10. The tests and the docs

Tests that pin today's assumptions, all at the step unless an item says
otherwise:

- `occurrences/mod.rs:502-587` `check_invariants`: the roots' subtrees
  partition the ids, root `i` is the sequent's root `i`.
- `nets/mod.rs:507` `desequentialize`, `:580` the round trip with
  `exponentials: false`, `:620` the text form, `:673` `validation`,
  which asserts `|- !A` refused (L706-714).
- `nets/graph.rs:426-504` the brute-force switchings and `check_cycle`
  (adjacency by parent or partner, L494-495), which 33's prompt asks to
  extend to instances.
- `nets/sequentialize.rs:244-278` a high derivation on a 128 KiB stack;
  33 needs the same for nested boxes, and `core/tests/depth.rs` (towers
  of `!` and `?` 100 000 deep, no net today) a net walk.
- `core/tests/export.rs:378, 410, 509`: `net.svg`, `cycle.svg`,
  `disconnected.svg` and the element counts `[8, 8, 2]`.
- `core/tests/serialize.rs` (§6); `proofs/check.rs:1421` the differential
  test and its mutants; `proofs/derivation.rs:1547` the MELL derivation.

Docs that state the assumptions: `nets/mod.rs:4-18, 143-180` (unit-free
MLL, the JSON), `export/svg/mod.rs:617-619` (ids by occurrence),
`proofs/mod.rs:4-16, 293-307`, `occurrences/mod.rs:112-126` (roots in
the sequent's order), `search/mod.rs:195-212` (the net engine "works on
the roots only"), `Engine::Net`, `README.md` (`--net`, Rocq), and the
rules files `core-nets.md` (MLL throughout), `core-forest.md`,
`core-proofs.md`, `core-export.md`, `cli.md`.

The word "cut" already names other things: the focused engine's `Cuts`
(`search/focus/mod.rs:405-450`, branches cut by the copy budget) and the
reference prover's (`search/reference.rs:136-178`), the checker's
`Budget::cut` (`proofs/check.rs:409-429`, a report cut short), the CLI's
`Shown::Cut` (`cli/src/prove.rs:430-432`, output cut short), and the
switching's left-out premise (`nets/graph.rs:389`,
`export/svg/net.rs:73`): about 190 matching lines, of which only a few
"cut-free" mean the rule. Open now, cheap and internal: rename them in
stage 3's refactor (for instance `Cuts` to `Bounds`, `Shown::Cut` to
`Shown::Truncated`), so that step 34's search for "cut" finds the rule.

## 11. The ten open-now items, ranked

1. **The forest's contract: the conclusion, the trees, the arena.**
   `Forest::roots()` is the conclusion in written order; ids may
   continue past it with extra trees (cut pairs, `A` then `A⊥`, the same
   shape, in recorded order); `sequent()` is the conclusion, whose arena
   may hold terms and atoms its roots do not reach; the forest records
   its extra trees (an empty `cut_pairs()` today), so that `Proof::new`,
   `ProofStructure::new` and `Interactive` keep their signatures and the
   JSON carries the cut formulas once (`occurrences/mod.rs:155-266`,
   `proofs/mod.rs:377`, `nets/mod.rs:298`). *Why first:* some sixty call
   sites read `roots()`, `ids()` or `sequent()` in one of three meanings
   (§1, §3); after 0.1.0 the Rocq checker (31) and the web client (32)
   are built on whichever meaning the docs give, and changing what a
   public accessor denotes is a break no compiler reports.
2. **The net's vertex: one type or two, and the id its API speaks.**
   Either `ProofStructure` takes a vertex id (MLL: the occurrence itself,
   no table, and an order that is canonical up to rule permutation) in
   `link`, `partner`, `links`, `from_links`, `same_component`,
   `unlinked` and the error payloads, or it is named and documented as
   the MLL structure and MELL nets get a second type, with
   `Outcome::net` and `svg::net` written so that one fits
   (`nets/mod.rs:46-77, 182-379`, `serialize/nets.rs:13-20`,
   `export/svg/net.rs:276-339`). *Why:* a dozen public signatures, the
   errors, the JSON's `links`, the text form and the drawing's ids all
   say "occurrence"; generalizing after the release breaks each, and 34
   (boxes duplicated) and 38 (quantifier nodes) build on the same table
   (33 §3).
3. **A version that 0.1.0 reads and refuses, and the keys reserved**:
   `cuts` on the proof, the net and the interactive state, the MELL keys
   on the net, `cuts` skipped when empty (`serialize/proofs.rs:60-66`,
   `serialize/nets.rs:13-20`, `serialize/interactive.rs:30-43`). *Why:*
   no proxy denies unknown keys, so a 0.1.0 reader drops `cuts`,
   `vertices` and a later `version` alike and fails further on with an
   id out of range; only a version the first release already checks
   makes old readers refuse new files by name. README §2 has the rule;
   this is its key list for 33 and 34.
4. **One options value for nets, with a bound, in place of `mix:
   bool`** in `new`, `from_links`, `from_proof` and the JSON's `"mix"`
   (`nets/mod.rs:202, 232, 249`, `nets/sequentialize.rs:45-51`). *Why:*
   33 (jumps, perhaps nullary Mix), 34 (cuts), 35 (criterion) and 36
   (cyclic) each extend the flag (README §3.4), four signature breaks
   after the release; and `from_proof` on an MELL term unfolds shared
   subproofs per path (`proofs/mod.rs:256-260`), which a shared or
   hostile term makes exponential, so its bound and stop (D16) belong in
   the first signature.
5. **`from_proof` handles every node kind and refuses the rest**
   (`nets/mod.rs:249-261`). *Why:* it keeps `Ax` nodes and drops all
   others; for a proof with a `Cut` over an extended forest it would
   return, with Mix, a "proof net with Mix" whose conclusions include the
   cut formulas, and without Mix a `Disconnected` error for a valid proof
   (inference). A public function that answers wrongly costs more than an
   error variant added now (README §2's criterion for `from_proof`).
6. **`#[non_exhaustive]` on `NetError` and `rocq::Unsupported`, with
   witnesses typed by vertex** (`nets/mod.rs:45-77`,
   `export/rocq.rs:107-130`). *Why:* 33 adds a link across depths, a jump
   into another box, a door without a copy, a disconnected box (33 §3),
   34 a cut on no dual pair and `Unsupported::Cut`; each is a major
   version on an exhaustive enum, and `SwitchingCycle(Vec<OccId>)` and
   `Disconnected(Vec<Vec<OccId>>)` change payload with item 2. README §2
   lists `NetError`; `Unsupported` is not on its list.
7. **Full matches in place of the silent defaults over rules and
   kinds**: `const fn` helpers such as `Rule::splits`,
   `Rule::has_principal` and `Node::splits_context` with exhaustive
   matches, used by `proofs/interactive.rs:353, 368, 564, 619, 643, 727,
   1054, 1072, 1095, 1098`, `proofs/derivation.rs:883-890`,
   `cli/src/interact.rs:190`, and full matches at `nets/graph.rs:144-148`
   and `export/svg/net.rs:289-292`. *Why:* `Node` and `Rule` stay
   exhaustive (README §3.1) so that a new variant is a compile error, but
   these places compile with `Cut` added and then panic (`standard(id)`,
   `unreachable!`), refuse a correct state as inconsistent, or build a
   Mix node; counter-neutral now, a debugging session at step 34.
8. **The drawing's ids and `Style`**: `o<n>` defined as a vertex id
   (the occurrence in MLL), prefixes reserved for boxes, doors, jumps and
   cut links, `Style` non-exhaustive (`export/svg/mod.rs:58-130,
   617-619`, `export/svg/net.rs:276, 283, 339`). *Why:* step 32's web
   client binds its clicks to these ids before step 33 changes what a
   vertex is; a renamed id breaks every client and snapshot, and box and
   cut fields cannot be added to an exhaustive public struct (README §2,
   §4.6).
9. **A counter-exact target list for the net engine**, beside
   `bench/targets.sh`, taken with stage 0's baselines
   (`bench/targets.sh:5-11` runs the focused engine only). *Why:* if
   item 2 changes `ProofStructure` at 28, nothing gates the net engine's
   hot path (`link_unchecked`, `same_component`, `is_acyclic`), and 33
   must show that its generalization costs nothing (33 §3); the `links`
   and `tests` counters are a function of the input on one thread, so a
   baseline from before the change makes that a comparison instead of a
   rebuild of an old revision on an idle machine.
10. **`is_roots` means "the goal is every tree of the forest"**
    (`search/mod.rs:375, 390-401`, with the doc of `prove_goal`,
    L195-212, and `Error::NetGoal`). *Why:* after a `cut` in an
    interactive state, and after its `undo` since the trees stay,
    `close` on a goal equal to the conclusion sends the whole extended
    forest to the net engine, which counts and links the cut trees and
    `expect`s a net (`search/net.rs:121-124`): the example of §5 panics.
    Free today, since no forest has extra trees, and one comparison of
    lengths.
