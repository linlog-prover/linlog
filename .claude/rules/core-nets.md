---
paths:
  - "core/src/nets/**"
  - "core/src/search/net.rs"
  - "core/src/search/essential.rs"
  - "core/src/search/planar.rs"
---

# linlog core: proof nets, essential nets and the net engines

Loaded, beside `core.md`, when a file of the proof nets or the net
engine is read or changed. The net engine's cubes on a pool are in
`core-parallel.md`, which loads for `search/net.rs` too.

## Proof nets

`nets/mod.rs` is the proof-net model for unit-free MLL, with or without
Mix; `ProofStructure::new` refuses any other fragment
(`NetError::Fragment`) and a forest past `MOST` (`u32::MAX / 3 − 1`
vertices: the graph's offsets are `u32` and a vertex has three slots at
most), and there is no net for affine mode (`Criterion::of` answers
`NetError::Mode`; the CLI refuses it before searching); in
intuitionistic mode the net is the one of the one-sided sequent. A
structure is the forest plus `partner` (one `u32` per vertex, `NONE` for
unlinked literals and connectives), the stack of links in the order
they were made, the coloured graph (`graph.rs`) and the `⅋`-free
skeleton (`skeleton.rs`); the `Criterion` (`mix`, the one value later
calculi extend, its fields flattened into the JSON) says whether Mix is
allowed, which is the difference between the two criteria. What the
code relies on:

- **Vertices, not occurrences, in the public calls** (`VertexId`): in
  MLL vertex `i` is occurrence `i`, with no table, so the engine's hot
  path converts for free (`VertexId::of`, `occ`, crate-private);
  `vertex(o)` and `occurrence(v)` are the public, checked forms. A
  structure with boxes will number its own vertices. **An id the
  structure does not hold is answered, never a panic**: `partner` is
  `None`, `same_component` false, `occurrence` `None`, and `link`
  refuses it (`NetError::NoVertex`); `is_acyclic` replaces a scratch
  sized for another structure with one of its own (one comparison).
  The probe found the panics (H23, H24).
- **`from_proof` matches every node** (`ax` a link, `⊗`, `⅋` and Mix
  nothing, every other rule `NetError::Rule` by its name, `wk` on an
  MLL sequent included), so a node kind added later is a compile error
  there; it refuses a goal proof (`Error::GoalProof`), charges its
  estimate (`STRUCTURE_BYTES` and `SCRATCH_BYTES` per vertex, the
  arena's terms) to `limits.memory_bytes` before it builds anything,
  and asks the stop per node and per round of the criterion.
- **The criterion's stop is a `dyn` inside the graph** (`Graph::acyclic`,
  `cycle`, the sequentialization's stages): generic, `acyclic` was
  instantiated in its caller's codegen unit, where `search` and
  `deletable`, which run per vertex and round, were not inlined, and the
  net engine's journey counted 4.8 % more (`search-wide-m2-256`, 287.2
  million against 274.2). A closure asked once a round costs nothing
  measurable as a `dyn`; keep functions on the hot path of the exact
  test non-generic.
- **`is_correct(stop)` takes no limits**: its memory is linear in the
  structure (the scratch), and `stop` is asked per round of the deletion
  procedure, which a cycle's witness runs once per edge. `sequentialize`
  takes `&Limits` (`SEQUENTIALIZE_BYTES` per vertex) and asks the stop
  per stage too; the proof it returns records the criterion's mode.
  `NetError::Refused { Memory | Stopped, phase: Net }` is no verdict.
  The codes: `invalid_net` for the malformed and the invalid, `no_nets`
  for `Fragment`, `Mode` and `Rule` (kind unsupported), the refusal's
  own otherwise. The net engine sequentializes the net it found under
  the search's limits and stop: a refusal there is `Unknown`
  (`MemoryLimit`, `Stopped`) with no net.
- **Links are a stack.** `link(x, y)` pushes and `unlink()` pops the last
  link: the skeleton's union-find has an undo log (union by rank, no path
  compression, one entry per union, so undo is O(1) and find is
  logarithmic), and a backtracking search takes links back in reverse
  order anyway. `link` validates in every build that its arguments are
  two unlinked dual literals of the forest (`check_link`, safe for any
  id) and returns a `NetError`, leaving the structure as it is; `from_links`,
  deserialization and `from_proof` go through it. The net engine links
  through the crate-private `link_unchecked`, which only debug-asserts:
  its candidates are unlinked dual literals by construction, and its loop
  pays for no check. A link that names no vertex of the structure is
  `NetError::NoVertex`, which `describe` prints without a formula.
- **The coloured graph** (`graph.rs`): vertices are the occurrences, edges
  the premise edges of every `⊗` and `⅋` plus the links, in CSR layout
  with the parent edge in a vertex's first slot, then its children, and
  for a literal its axiom slot last (`NONE` while unlinked), so `link` and
  `unlink` write two slots. The colouring is the spec's: the two premise
  edges of a `⅋` share a colour of their own, every other edge has its own
  colour, so a cycle survives some switching iff it is properly coloured.
  `⊗` premise edges must never share a colour (that would forbid the cycle
  `⊗ – A – ~A – ⊗` in `⊢ A ⊗ ~A` and accept a wrong structure), and `⅋`
  premise edges must always share one (else `⊢ A ⅋ ~A` would be rejected).
  The colours are not stored: with this colouring, Yeo's deletion
  condition ("no component of `G − z` meets `z` in two colours") is
  exactly "every present edge of `z` is a bridge" for a vertex that is not
  a `⅋`, whose incident colours are all distinct, and "the parent edge is
  absent or a bridge" for a `⅋`, whose premise edges share a colour.
  Bridges come from one iterative Tarjan search per round (`search`,
  which skips the search-tree parent vertex, valid because the graph is
  simple: a literal's tree edge and its link never join the same pair).
  Each search fills `disc` over the width: discovery times that rise
  from search to search instead save the fill's instructions but no
  measured time, as a memset is cheap per instruction.
- **The Yeo test is exact on partial structures.** `acyclic` deletes, in
  rounds, every vertex deletable by the round's bridges and stops when a
  round deletes nothing: everything gone means no switching cycle, and
  otherwise the vertices left carry one. Deleting several vertices found
  deletable in one round is sound because deletability only grows as
  vertices go (components of `G − z` can only split). A deletable vertex
  never lies on a properly coloured cycle, and Yeo's theorem gives a
  deletable vertex whenever there is none, which is the exactness. An
  unlinked literal is a leaf and lies on no cycle, so
  `is_acyclic(&mut Scratch)` answers the same question about a partial
  structure; it allocates nothing, the `Scratch` (from
  `ProofStructure::scratch`) holds the bitsets and the search arrays.
  The cost is one search per round, and the number of rounds is the
  nesting depth of cycles that pass through both premises of a `⅋` (each
  round peels one layer of them); a forest of unlinked trees goes in one.
- **Connectedness is a count, checked after acyclicity only.** When no
  switching has a cycle, every switching is a forest with
  `2t + p + k` edges (`t` tensors, `p` pars, `k` links), so it is a tree
  iff that is `V − 1`. Before acyclicity the equation says nothing. With
  Mix the equation is dropped: correctness is acyclicity alone. The empty
  structure is not a net in either case (`NetError::Empty`), since no rule
  concludes `⊢`. `is_correct` requires completeness first
  (`NetError::Unlinked`).
- **Witnesses are for humans and tests, not the hot loop.** `is_correct`
  allocates its own scratch. A `SwitchingCycle` is isolated from the
  vertices the procedure got stuck on by removing each edge among them in
  turn and keeping it out when a cycle survives; an edge found necessary
  stays necessary as edges go, so one pass leaves exactly one cycle, at the
  cost of one deletion procedure per edge. `Disconnected` lists the parts
  of the switching that keeps every left premise, each by its vertices
  without a parent edge there (roots and right premises of `⅋`s); a part
  may hold no root at all. `nets::graph`'s tests check every witness
  against a brute-force enumeration of switchings and assert the two
  criteria agree; keep that test when the criterion changes.
- **Sequentialization** (`sequentialize.rs`) is the splitting-tensor
  lemma on the *plain* graph of a sub-net (tree edges and links, no
  switching): a `⊗` conclusion is splitting iff its premise edge is a
  bridge there. The spec's "delete it and count components under one
  switching" is wrong: under any switching, every `⊗` conclusion of a
  correct net leaves exactly two components (the switching is a tree and
  loses two edges), so the count cannot tell. Per stage: open every `⅋`
  conclusion (its rule goes below the rest), one search from the
  conclusions gives the parts and the bridges, parts are joined with Mix
  (only with Mix; a sub-net of a connected net is connected), two
  literals are an axiom (`Ax(min, max)`), else the splitting `⊗` with the
  smallest id is applied and the conclusions reached from its left premise
  go left. Handled conclusions are deleted in the scratch, so a sub-net is
  what its conclusions reach. O(n²) in the net's size. It does not
  recurse: a stage leaves what proves its sub-net as steps on a stack of
  the sequentialization's own (`Step`: a sub-net to prove, a Mix, a `⊗`,
  the `⅋` rules of the stage), taken in the order a recursion would make
  its calls, so the nodes come in that order too and a derivation of any
  height costs the caller's stack nothing
  (`a_high_derivation_needs_no_stack`). The proof is checked in a
  `debug_assert!` and the round trip derivation → net → derivation is a
  test.
- **Nets and terms.** `from_proof` reads the links off the `Ax` nodes and
  returns the net only if it is correct; every proof the checker accepts
  gives a correct net, two proofs that differ by rule permutations give the
  same one, and `sequentialize` gives one proof per net, so the net is the
  canonical form of an MLL proof. The net of a term is meaningful only over
  the term's forest, like the term itself.
- What the net engine keeps outside the structure: the per-atom counts,
  the copies of literal conclusions, the explicit stack, statistics, and
  one `Scratch`. The structure offers `mate` and `joined` (`partner` and
  `same_component`, the skeleton's rejection, without the check of the
  id against the structure, since the search asks both for every
  candidate link and its ids are the forest's by construction),
  `unlinked`, `link_unchecked` and `unlink`
  and `is_acyclic`, and `Forest::lca` is the other O(1) rejection.
- The text form (`Display`: the sequent, `~A[0] — A[2]` per link sorted
  by first id, then `proof net`, `proof net with Mix` or `not a proof net:
  ` with the reason in formulas) and the JSON form are pinned in tests.

## Planar nets: cyclic MLL

- **The criterion** (`Criterion::order`, `Criterion::CYCLIC`): a
  structure is a proof net of cyclic MLL exactly when it is a
  Danos–Regnier net without Mix and planar, no two links crossing in the
  cyclic order of the literals, which is their id order
  (`ProofStructure::is_planar`, a stack scan, `NetError::Crossing` with
  the two links). `is_correct` asks it last, after the commutative
  conditions, so a commutative structure's verdict and witness are as
  before. Both directions and the sequentializer were reviewed: every
  sequentialization of a planar net is a cyclic proof, since the two
  parts a splitting `⊗` leaves are joined by chains of links that would
  cross if a conclusion of one stood between two of the other; the
  `debug_assert` in `sequentialize` checks it in `Mode::CYCLIC`.
- **No order with Mix**: `ProofStructure::new` refuses a cyclic criterion
  with Mix (`NetError::Mode`), since `Criterion` is public with public
  fields; such a structure would sequentialize into Mix nodes recorded as
  cyclic, which the checker refuses. `Criterion::of` refuses the ordered
  modes `Mode::check` refuses.
- **The planar search** (`search/planar.rs`, `Planar`): the classical
  criterion (`Switching`, whose structure carries the cyclic criterion so
  that the net found is one) wrapped with two rejections in `admits`:
  the two ends must lie in one region (the innermost made link around a
  place in the linear order, kept per literal place, rewritten inside a
  link's span at `link` and restored at `unlink`, which the loop calls
  last in first out), which is exactly "crosses no made link"; and the
  literals strictly between the ends must be balanced in every atom,
  compared by prefix sums of a hash (wrapping on purpose: equal counts
  give equal sums, a collision only lets a link through). `test` and
  `balanced` are the classical ones: a planar net is a net. Planarity of
  the complete linking holds by construction (a `debug_assert` in
  `test`).
- **35a's prunes under the order**: the three symmetry breaks are off
  (`Prunes::PLANAR`): exchanging equal literals or subformulas moves a
  link and can make two cross, so the canonical linking of an orbit may
  be the non-planar one (`|- a * a, ~a, ~a` is provable only by the
  descending linking 1–4, 2–3; `|- (a * b) * (a * b), ~b, ~a, ~b, ~a`
  only by the nested one). A conditional break (reject the descending
  linking only when its exchange crosses nothing) would be sound and is
  not built. The skeleton balance and the count equation are kept: each
  rejects only partial linkings without a Danos–Regnier completion.
- **One thread, and goals**: `Nets::decide` runs the planar search on
  the calling thread whatever `jobs` says (the cubes are
  `Linker<Switching>`'s), and `race` runs one search in an ordered mode.
  A goal off the roots is admitted in an ordered mode when its formulas
  are distinct and none lies in another's subtree (`apart`); `goal`
  searches the sequent of those formulas as roots (the same arena), whose
  forest numbers each formula's occurrences as the goal's forest does,
  shifted, and carries the proof back by the shift; the answer has no
  net. That is how a session closes a cyclic goal.
- **What it costs the commutative search**: nothing in its counters (the
  net target set equal to `net-after-prunes` on every row); the planar
  criterion's call of `Switching::admits` made LLVM outline
  `Forest::lca` from the classical loop, +2.4 % of
  `search-wide-m2-256`'s instructions, which `#[inline]` on `lca` takes
  back to +0.24 %. The tests: `planarity` in `nets/mod.rs` (every
  linking of small sequents against a pairwise crossing test, every
  sequentialization checked in the cyclic mode) and
  `agrees_with_every_planar_linking` in `search/planar.rs` (the engine
  against an enumeration of every planar linking on random derivations
  of the cyclic calculus, their mutants and balanced sequents; it caught
  the conclusions' break, first left on).

## Essential nets

`nets/essential.rs` reads the structure of an intuitionistic sequent as
Lamarche's polarized structure and decides it by a directed criterion,
`ProofStructure::is_essential`; `ProofStructure::reading` gives the
reading (recomputed per call, linear). What the code relies on:

- **The edges** (`fixed_successors` plus the links): an output
  occurrence that is no root to its parent, an input connective to each
  input child, a link from its input literal (`~a`, always input) to its
  output literal (`a`, always output). The antecedent of an output `⅋`
  (`A ⊸ B` stored as `A⊥ ⅋ B`, left child input) has no edge in. Only an
  input `⅋` (a hypothesis `A ⊗ B`) has two edges out; in a complete
  linking every occurrence but the goal has one, so in an acyclic one
  every occurrence reaches the goal, which is why no connectedness or
  count condition is needed. The reading fixes which child is which
  (`Reading::new`: a `⊗` in input position has its output child left).
- **The criterion**: no directed cycle (an iterative depth-first search,
  the stack's places in `state`), and every output `⅋` but the goal
  dominates its antecedent: a breadth-first search from the antecedent
  that never enters the implication must not reach the goal (one round
  stamp per implication). Witnesses: `NetError::DirectedCycle` (each
  vertex followed by its successor), `NetError::NoDominator` (the
  implication and a path from its antecedent to the goal);
  `NetError::NotIntuitionistic` for a sequent without a reading. Linear
  memory, time linear per implication, `stop` asked per implication.
- **It is the Danos–Regnier verdict without Mix** on every complete
  linking of an intuitionistic sequent (Lamarche's theorem, with the
  fact that every sequentialization of a classical net of an IMLL
  sequent is intuitionistic): `agrees_with_the_switchings` checks it on
  every linking of generated IMLL sequents and their mutants, the
  dominator condition against an enumeration of every simple path, and
  every witness. Keep that test when either criterion changes. The
  essential engine and the drawing rest on this equality; the structure
  keeps the classical `Criterion` and sequentializes classically.

## The net engine

`search/net.rs` is the spec's MLL-Net: axiom-linking search over a
`ProofStructure`, for unit-free MLL with or without Mix. A cut-free proof
of MLL is its linking, so the only choices are which dual literals to pair.
What the code relies on:

- **The loop is generic over a criterion** (`Linker<'a, C: Correctness>`):
  the choice, the frames, the cadence, the stop, the counters and the
  cubes are the loop's; the criterion holds the links and answers `mate`,
  `admits` (its constant-time rejections), `link` (fallible: a bound on
  what it holds stops the search), `unlink`, `test` (at the cadence and
  on every complete linking) and `balanced`. `Switching` is the
  Danos–Regnier criterion: the `ProofStructure`, its `Scratch`, the
  lowest-common-ancestor and skeleton rejections in that order, the
  exact test, the balance; `Essential` (`search/essential.rs`, below) the
  directed one. Monomorphized, the generic loop left the classical
  search's instructions as they were (`search-wide-m2-256` at the
  ratchet, every counter of the net target set). The cubes
  (`net::parallel`) are written against `Linker<Switching>`: the
  essential engine runs on one thread.

- **Preprocessing** (`counts_admit`): `c = t − p + 2` (`≥` with Mix) and
  as many `a` as `~a` per atom, else `Unprovable` at once. Both are
  necessary (induction on cut-free proofs; Mix only raises `c`). The
  equation is also sufficient for connectedness once a complete linking
  is acyclic: every switching keeps `2t + p + k` edges, the forest has
  `c = 2k − t − p` formulas (every connective is binary), so
  `c = t − p + 2` is `k = t + 1` is "`V − 1` edges", a tree. That is why
  `run` calls a complete linking that passed the exact test a proof net
  without another connectedness test, and why `search` then `expect`s
  `sequentialize` (which re-runs `is_correct`) to succeed. Do not relax
  the preprocessing or move it after the search. The empty sequent fails
  the equation in both modes.
- **Two constant-time rejections** of a candidate link, sound in every
  partial linking because a switching cycle survives every extension:
  the lowest common ancestor of two literals of one conclusion is a `⊗`
  (the tree path plus the link is a cycle kept by the switchings that
  keep the path's `⅋` premises: *some* switching, which is all the
  criterion needs; it is not kept by every switching when a `⅋` lies on
  the path, so no stronger rule follows from it), and the `⅋`-free
  skeleton already joins them (`joined`: a cycle with no `⅋`
  premise edge, kept by every switching). `Forest::lca` is `None` across
  roots, and the rule is only valid within one root.
- **Symmetry breaking** (`Symmetry`, built once per run): two kinds of
  exchange map nets to nets and non-nets to non-nets. *Literal classes*:
  equal literals (atom and sign) that are conclusions, or leaves of one
  maximal tree of `⊗` alone or of `⅋` alone (the tree named by its top,
  the "anchor"); exchanging two of them keeps every switching's
  cyclomatic number and components (a `⊗` tree contracts to one vertex
  the leaves hang from; every switching of a `⅋` tree joins its root to
  one leaf and leaves the others as paths, so the switchings join the
  root to each leaf in turn). *Compound classes*: equal compound
  subformulas (`shape`, a structural id computed bottom-up, not the
  arena's `TermId`) that are conclusions or leaves of one such tree, by
  the same argument for whole subtrees. A link is admissible only if the
  partners ascend by id along each literal class (chained in id order),
  and, for a compound class whose copies have a *key literal*, the keys
  of the key literals' partners do not descend along the copies. Why one
  linking of every orbit survives: sort the copies of each compound
  class by key, outer classes first, then take the lexicographically
  least linking that exchanges within literal classes reach (consecutive
  `x < x'` with partners `p > q`: the exchange is smaller at
  `min(x, q)`). That needs (1) the key literal in no literal class of two
  or more and inside no copy of a smaller broken class, so neither step
  moves it, and (2) a partner's key to be the same for every partner the
  exchanges move it to: the first literal of its literal class, or
  `NONE` (above every key) inside a copy of a broken class. It also
  rests on: an exchange of partners is the original linking on the tree
  reshaped; sorting one class moves only literals inside its copies (all
  keyed `NONE`) and carries inner classes along, so it leaves every
  other class's order and outer classes go first; copies have one shape,
  so the key literal's offset in the first copy holds in every copy; and
  partial linkings are pruned without loss because every link of the
  canonical net passes the orders, the balance and the partner count at
  every stage. A vacuous chain is dropped because two `NONE` keys never
  reject. The step-6
  counterexample to the spec's first-literal key (two groups whose first
  literals link into each other's copies) is exactly a key that moves;
  (2) makes it `NONE`. A class without a key literal is not broken (its
  copies stay unmoved, so their literals keep finite keys: the bins of
  `three_partition_mll` are such a class, the items keyed by the bin's
  `~k` class), and a class whose key literals can only take `NONE`-keyed
  partners gets no chain (vacuous; `wide-m2`'s pairs, whose check cost
  5 % of `search-wide-m2-256`). The brute-force test
  (`agrees_with_every_linking`) failed when a literal class spanned
  trees, when keys were taken inside broken copies, or with the balance
  bound one too low; a key literal allowed inside a literal class passed
  it and 40 000 more rounds, so that restriction stands on the argument
  alone. The spec forbade breaking symmetry inside formulas; the
  argument above is why it is sound here.
- **The balance prune** (`balanced`, in `choose`, per atom with three
  pairs or more left, at the nodes where the exact test runs: its
  cadence, so on a large structure every fourth link): no `⅋`-free
  skeleton component may hold more
  unlinked literals of an atom than the atom's unlinked pairs. A literal
  never links into its own component, so each `a` of a component needs
  its own `~a` outside it. With two pairs or fewer, a component that
  breaks it leaves a literal with no admissible partner, which the
  partner count finds, so the check skips those atoms (it cost 30 % of
  `search-wide-m2-256` when it ran on every atom). It never fired on
  the families (Partition, 3-Partition, `wide`: equal node counts with
  and without it), where the count equation and the `⅋` boundaries keep
  a component to one sign of an atom, and it cost `wide-m3` and
  `wide-m4` time at every node, hence the cadence; it is the net engine's analogue of the
  focused engine's split counts only in its form, and a stronger one
  needs sets joined in some switching, which no cheap structure keeps.
- **`Prunes`** switches the conclusions' break, the leaf break, the
  compound break and the balance prune off one at a time for the tests
  only (`with_prunes`); a commutative search applies all four
  (`Prunes::ALL`), the planar search the balance alone
  (`Prunes::PLANAR`, below). The conclusions' break was always on before
  the field existed: `Symmetry::new`'s `chained` reads it for a literal
  whose anchor is the root, the leaf break for the others.
- **Choice order** (`choose`): the unlinked literal with the fewest
  admissible partners, ties to the first in atom order, `a` before `~a`,
  then id; every unlinked literal is inspected, so a literal without an
  admissible partner is a dead end found now (forward checking). The
  counting of one literal stops at the best count so far, which never
  hides a zero. Partners are then enumerated in id order over
  `forest.literals(atom, !sign)`. All of this makes the run deterministic.
- **The exact test** (`is_acyclic`) runs after every link on a structure
  of at most 200 occurrences and after every fourth link above that
  (`Options::test_period` overrides; zero counts as one), and always on a
  complete linking. Between tests a doomed branch is followed for at most
  `period − 1` links. `is_correct` (witnesses, allocation) is never called
  in the loop; `sequentialize` calls it once at the end.
- **The stack** (`Frame { literal, next, forced }`): every frame but the top has
  its current link made, the top is looking for one; `next_partner`
  moves `next` past the partner it returns, so a linking is tried at most
  once; a dead end, a failed test or an exhausted frame takes the last
  link back, always in stack order, which the structure's undo log
  requires. `remaining[atom]` (unlinked pairs per atom) follows every
  link and unlink. The stop condition is polled once per node, in
  `decide`, and once per exact test that fails: a run of failed
  candidates and exhausted frames chooses no literal, and each failure
  costs a test over the structure. A frame is `forced` when its literal
  had exactly one admissible partner when it was chosen (`choose`
  counts the best literal's partners to the end, so the count is exact
  and is what `next_partner` will find); `Linker::choices` counts the
  frames that are not, which is what a pool's cubes are cut by (below).
  Neither changes the search on one thread.
- **Where it loses.** Horn encodings (Matsuoka's Partition and Lincoln's
  two-literal 3-Partition, `families::partition` and
  `families::three_partition_mll`): the symmetry breaks took the
  interchangeable leaves of `b ⊗ b ⊗ b` and `~b ⅋ ~b` and the equal items
  out of the search (`partition-no/3` 269 950 nodes to 391,
  `3-partition-mll-no/6` from over 60 s to 352 nodes), but what is left
  is the subset choice itself, which a cycle shows only after the links,
  where the focused engine's counts of each `⊗` split refute at once.
  The dispatch decides where the engine is the default
  (`core-search.md`, the dispatch).
- **The essential-net engine** (`search/essential.rs`,
  `Engine::Essential`, only when forced: intuitionistic mode, unit-free
  IMLL, the roots) is the loop under `Essential`, the reflexive
  transitive closure of the directed graph as rows of bits
  (`n·⌈n/64⌉` words), charged with the linear tables to the search's
  account before it is allocated, in checked arithmetic, and allocated
  fallibly (`try_reserve_exact`): without a bound the charge always
  fits, and a 33.5-million-occurrence sequent once asked for 140 TB and
  aborted; now the allocator's refusal is `MemoryLimit { limit_bytes:
  u64::MAX }`, which reads as such and names no flag. The set-up polls
  the stop every `SETUP_ROWS` rows. A link `i → o` ORs row `o` into every row
  that reaches `i`, logging each changed word (row, word, old value)
  after a mark per link; `unlink` restores them in reverse, which the
  loop's stack order requires. The log grows by chunks charged before
  they are reserved; a refusal in the middle of a link restores the
  words it changed and stops the search.
  - *`admits`*: reject when `o` reaches `i` (a directed cycle, kept by
    every completion; so the graph stays acyclic and the closure exact),
    and when `i` lies in the antecedent of an implication `l ≠ goal` by
    input premises alone (`binder`) while `o` reaches the goal and not
    `l` (a path from the antecedent around `l` in every completion).
  - *`test`* (the dominator condition, at the cadence): for each
    implication `l ≠ goal` with antecedent `s`, fail when `s` reaches no
    `l` but `escapes` (reaches the goal or something `l` reaches), or
    when an input `⅋` `p` with `s ⇝ p ⇝ l` has a premise that reaches no
    `l` but escapes. Sound on partial linkings: a vertex `l` reaches (or
    the goal) never reaches `l` in an acyclic completion, so the path
    goes around `l` to the goal; exact on complete linkings, where every
    vertex reaches the goal, by the lemma that every path from `s` to the
    goal passes `l` iff `s` reaches `l` and every successor of a vertex
    between them reaches `l` (only input `⅋` has two successors).
  - The symmetry breaks are the net engine's: they preserve the
    Danos–Regnier verdict on complete linkings, which is the essential
    one, and every rejection is sound, so the canonical net of an orbit
    survives. They are no automorphisms of the directed graph (in
    `a ⊸ a ⊸ b` the two `~a` sit at different implications), and need
    not be. The balance prune is not used (no skeleton).
  - Measured (one thread): slower than the two-sided engine on every
    intuitionistic row but the 64-link chain, and slower than the net
    engine at the two-sided engine's recursion limit, so no dispatch row
    and no handover (`plan/reports/35-mll-engines.md`, session 35b). It
    beats the net engine on Partition by orders of magnitude: the
    directed rejections cut where the switchings see a cycle only later.
  - What it takes from the front door and the count check: the reading
    (`task.reading`, which `read` computes before any engine runs in
    intuitionistic mode, so the `expect` holds); the balance of every
    atom, on which `is_complete` (twice the links is the literals)
    relies. The count equation `c = t − p + 2` never rejects there:
    counting each occurrence through its parent gives
    `c = t − p + 2g` for `g` output roots, and an intuitionistic
    sequent has one. `oriented` makes the `~a` of a link its input end,
    which `admits`' binder (defined on input occurrences) reads.
  - The shared `choose` counts the admissible partners of every
    unlinked literal before a node's poll: quadratic in one atom's
    literals, so a sequent of 65 000 `a` and `~a` overshot a 1 s stop by
    5 s here and by 23 s in the net engine (a follow-up of the loop's).
  - A linking search is exponential in a literal's copies: on a
    generated sequent with each of three atoms some fifteen times both
    net engines ran 20 s without an answer where the two-sided engine
    proves at once. The tests bound the copies (three for this engine,
    two for the net engine).
- **Every proof passes the checker** (in `prove_goal`, in every build;
  `debug_assert!` in `search` besides; every test), and every net is the
  net of its proof (`from_proof` in the tests' `run`). The differential test against the focused engine
  (`agrees_with_the_focused_engine`) covers generated provable sequents,
  their mutants, doubled sequents (equal conclusions) and random
  balanced sequents from `generate::balanced`, which pass the counts and
  are mostly unprovable; extend it rather than pinning verdicts by hand.
