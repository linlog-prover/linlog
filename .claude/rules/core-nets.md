---
paths:
  - "core/src/nets/**"
  - "core/src/search/net.rs"
---

# linlog core: proof nets and the net engine

Loaded, beside `core.md`, when a file of the proof nets or the net
engine is read. The net engine's cubes on a pool are in
`core-parallel.md`, which loads for `search/net.rs` too.

## Proof nets

`nets/mod.rs` is the proof-net model for unit-free MLL, with or without
Mix; `ProofStructure::new` refuses any other fragment
(`Error::NetFragment`), and there is no net for affine mode (the CLI
refuses it before searching); in intuitionistic mode the net is the one of
the one-sided sequent. A structure is the forest
plus `partner` (one `u32` per occurrence, `NONE` for unlinked literals and
connectives), the stack of links in the order they were made, the coloured
graph (`graph.rs`) and the `⅋`-free skeleton (`skeleton.rs`); `mix` says
whether Mix is allowed, which is the difference between the two criteria.
What the code relies on:

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
  pays for no check. A link that names no occurrence of the forest is
  `NetError::NoOccurrence`, which `describe` prints without a formula.
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
  one `Scratch`. The structure offers `partner`, `unlinked`,
  `link_unchecked`, `unlink`, `same_component` (the skeleton's rejection)
  and `is_acyclic`, and `Forest::lca` is the other O(1) rejection.
- The text form (`Display`: the sequent, `~A[0] — A[2]` per link sorted
  by first id, then `proof net`, `proof net with Mix` or `not a proof net:
  ` with the reason in formulas) and the JSON form are pinned in tests.

## The net engine

`search/net.rs` is the spec's MLL-Net: axiom-linking search over a
`ProofStructure`, for unit-free MLL with or without Mix. A cut-free proof
of MLL is its linking, so the only choices are which dual literals to pair.
What the code relies on:

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
  skeleton already joins them (`same_component`: a cycle with no `⅋`
  premise edge, kept by every switching). `Forest::lca` is `None` across
  roots, and the rule is only valid within one root.
- **Symmetry breaking for equal literal conclusions only.** Conclusions
  that are the same literal (same atom and sign, both roots) are chained
  in id order (`copy_before`, `copy_after`), and a link is admissible only
  if the partners of the chain ascend with the conclusions. Sound because
  swapping two such conclusions is an automorphism of the structure, so it
  maps nets to nets, and the lexicographically least member of an orbit
  (partners listed by literal id) has ascending partners: if consecutive
  copies `x_i < x_{i+1}` had partners `p > q`, the swap differs at
  `{x_i, x_{i+1}, p, q}` and is smaller at `min(x_i, q)`. The argument
  covers every group at once, however the partners lie. It does **not**
  extend to equal compound conclusions with a first-literal key: two
  copies of `F` and two of `G` whose first literals link into each other's
  copies at non-first positions have an orbit in which no member sorts
  both groups; a compound extension needs keys under roots that no
  symmetry moves, and is a follow-up. The spec forbids symmetry breaking
  inside formulas, which is the loss on Horn encodings (below).
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
  and is what `next_partner` will find); `Engine::choices` counts the
  frames that are not, which is what a pool's cubes are cut by (below).
  Neither changes the search on one thread.
- **Where it loses.** Horn encodings (Matsuoka's Partition and Lincoln's
  two-literal 3-Partition, `families::partition` and
  `families::three_partition_mll`) have few atoms
  with many occurrences, and the equal literals inside `b ⊗ b ⊗ b` and
  `~b ⅋ ~b` are interchangeable, so a wrong early choice costs a whole
  symmetric subtree before a cycle appears; the focused engine refutes the
  same sequents through the counts of each `⊗` split in milliseconds
  where the net engine needs seconds or does not finish. Distinct atoms
  and wide contexts are where the net engine wins. Symmetry breaking for
  the leaves of a pure `⊗` or `⅋` tree of equal literals (sound: the
  leaves of a `⊗` tree share every switching's component, and a `⅋` tree
  opens to interchangeable conclusions) is a follow-up (`plan/later.md`,
  "Net-engine pruning and routing"); until then the dispatch routes MLL with a literal of
  multiplicity above 2 to the focused engine (`Feature::FewEqualLiterals`, a row of `search::DISPATCH`).
- **Every proof passes the checker** (in `prove_goal`, in every build;
  `debug_assert!` in `search` besides; every test), and every net is the
  net of its proof (`from_proof` in the tests' `run`). The differential test against the focused engine
  (`agrees_with_the_focused_engine`) covers generated provable sequents,
  their mutants, doubled sequents (equal conclusions) and random
  balanced sequents from `generate::balanced`, which pass the counts and
  are mostly unprovable; extend it rather than pinning verdicts by hand.
