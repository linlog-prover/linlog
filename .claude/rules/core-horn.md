---
paths:
  - "core/src/search/horn/**"
---

# linlog core: the Horn engine

Loaded, beside `core.md`, when a file of `search/horn/` is read. The
front door, the dispatch and the memory account it plugs into are in
`core-search.md`.

## What it decides, and why that is sound

- **Files.** `mod.rs` holds `Horn` (the `Decide` implementation,
  `Engine::Horn`), `Program` (a goal read as a Petri net) and
  `is_net`, the dispatch's feature (a program with a clause under `!`:
  the goal's own `?` members, since the fragment the options assert may
  have exponentials the goal lacks); `reach.rs` the
  search over markings; `cover.rs` the backward search for affine mode;
  `equation.rs` the state equation and the exact check of its
  refutation; `proof.rs` the proof read off a firing
  sequence; `tests.rs` the engine's own tests (the reference comparison
  is `engines_agree_on_horn_programs` in `search/reference.rs`).
- **The shape** (`Program::read`), one-sided, with the bodies' atoms of
  one sign (`a`, or else all `~a`): every member under `?` is a clause
  with exactly one head; every other member is a marking (a `⅋`/`⊥` tree
  of head literals), a clause with exactly one head (used once), or the
  goal (a clause without a head), of which there is exactly one. A
  clause is a tensor of body literals and `1` with at most one factor a
  head, a `⅋`/`⊥` tree of head literals. Two-sided that is Kanovich's
  !-Horn sequent `W, Γ, !Δ ⊢ Z`: atoms, implications between tensors of
  atoms used once and under `!`, and one tensor of atoms to reach. It is
  stricter than `focus::schedule::chains`, the forward bound's test,
  which also takes a goal under `?` (`growing`), no goal or several:
  there the correspondence below fails (`⊢ ~a, a, ~b, b` is reachable as
  a net and unprovable without Mix).
- **Why a proof is a firing sequence, and a refutation sound.** Every
  literal of the bodies' sign is an output, every literal of the other
  sign an input, and `⅋` and `⊥` occur in inputs only; call a subformula
  an output when it is the goal or lies in a body (a body literal, a
  `1`, a tensor holding no head), an input otherwise (markings, heads,
  the `?` members, the clauses and the tensors on a clause's path to its
  head), and let `q` be the outputs of a sequent. Then `q − 1 = (Mix
  rules above) + (outputs weakened above)`, rule by rule over the
  checker's calculus: an axiom pairs a body literal with a head literal
  (`q = 1`); `1` is an output (`q = 1`); a tensor on a path to a head
  has `q = q₁ − 1 + q₂` (its body factor is its premise's one output
  less), a tensor in a body or the goal `q = q₁ + q₂ − 1`; `⅋`, `⊥`,
  `Quest`, `Copy` and the weakening of an input keep `q`; the checker's
  Mix is binary (no Mix of nothing), `q = q₁ + q₂`. The root has
  `q = 1`, so no proof of such a goal uses Mix or weakens an output, and
  every sequent of it has exactly one output. From such a proof a firing
  sequence follows by induction on the dyadic proof, with a partial
  clause `B₁ ⊸ … ⊸ H` in the linear zone used once and a clause in `Θ`
  any number of times: axiom and `1` fire nothing; `⊥` and `⅋` keep the
  marking; a tensor of body factors runs its premises' sequences one
  after the other; a tensor on a path to a head (a `⊸L`) runs the body
  premise's sequence, then the other's with the body's tokens carried;
  `Copy` and `Quest` are the clause's transition under `!`. That covers
  everything `Program::read` accepts, which is wider than Kanovich's
  Horn implications `X ⊸ Y` ("The complexity of Horn fragments of
  linear logic", 1995): curried clauses with the head anywhere in the
  tensor tree, `1` in bodies, clauses without a body, `⊥` in heads,
  lone head literals as markings, the goal `1`, and bodies written `~a`
  (by the symmetry that swaps every atom with its negation). Hence
  `Unprovable` from an exhausted set of reachable markings is sound in
  classical and intuitionistic mode, with or without Mix. The argument
  is prose here and in step 27's report; the panel that reviewed it
  checked the count rule by rule and wrote the induction independently;
  no test can exhaust it.
- **What the counts rule out is refuted before the search**
  (`Horn::decide` calls `focus::refutation`, the test the front door
  applies to every engine's `Unprovable`): a goal with an atom that
  nothing balances, `|- b, ?~c` or `c, !c |- a`, is an infinite net the
  search would run to the memory bound, where the focused engines
  refute it at once from the same counts. Atoms under a `?` have no rows
  in the counts, so on a net proper (every atom in a clause under `!`)
  the test says nothing and costs the counts' set-up, a few passes; an
  atom outside every `!` in nested tensors (a long clause used once, a
  goal of thousands of distinct atoms) has rows quadratic in the nesting,
  the cost the focused engines pay on the same sequent (a goal of 20 000
  distinct atoms: 0.95 s to the 1 GiB bound, which `--engine horn` met
  in 20 ms before). The counts are charged to a fork of the search's
  account (`Account::fork`), which goes with them: charged to the
  search's own, they stayed counted and cost the search decisions near
  its bound (the second panel's finding). When they refute,
  `prove_goal` computes them again to say why, so a stop that fires
  between the two on a forest of 65 536 occurrences or more makes the
  reason "exhausted" for a goal no search touched: the verdict is right.
  Found when the Horn row made such sequents the Horn engine's by
  default: three tests of the focused engines answered "the memory
  limit" instead of "unprovable".
- **Intuitionistic mode** additionally requires the reading to put
  exactly the outputs above on the right of `⊢` (`Program::read` checks
  every occurrence's `Position`): the proof built has one output on
  every sequent, so the one-succedent check accepts it only if the
  reading agrees. A goal whose reading differs is no program for the
  engine, never a wrong proof.
- **In affine mode the question is coverability** (`cover.rs`): a proof
  may weaken the tokens and clauses a firing sequence leaves, so the
  target need only be covered and a clause used once is used at most
  once (its ticket may stay). The count above holds with weakening too
  (a weakened input keeps `q`, a weakened output would lower it), so no
  proof weakens an output and every sequent still has one output; the
  induction then gives a firing sequence of the lossy net, where a
  weakening drops tokens (a token, a clause used once, or the rest of a
  clause whose body part was already paid, which dropped its tokens).
  Dropping tokens never helps to cover (more tokens enable at least the
  same firings), so a lossy sequence that covers the target gives one
  without drops that covers it, and backward coverability decides the
  goal in affine mode, classical or intuitionistic, with or without Mix.
- **The state equation refutes beside the search** (`equation.rs`,
  below): the second way to `Unprovable`, the one that reaches nets whose
  markings grow without end (`!d, !((c * d * d) -o 1) |- c * c`, where
  nothing makes `c`).

## The net

- **Places** are the atoms the goal names (`Program::place_of`, given
  out in the order met) and one per class of clauses used once, whose
  tokens are the class's unused clauses: a clause used once is a
  transition that also takes a token of its class, and the target holds
  none, so a firing sequence that reaches it uses every such clause once.
  Clauses used once with equal arcs are one class, so interchangeable
  clauses do not multiply the markings.
- **Transitions under `?`** with equal arcs are one transition, and one
  whose inputs equal its outputs is dropped; every member under `?` is
  still moved into `Θ` by the proof, used or not.
- **Transitions that can never fire are dropped** (`live`): the places
  that some marking reachable from the initial one marks lie within the
  least set that holds the initial marking's places (the tokens given
  and the class places) and the outputs of every transition whose inputs
  it holds all of, by induction on the firing sequence; a transition with
  an input outside it never fires. Every search, the state equation and
  its `certify` then see only the transitions that may fire, so the
  equation's refutation rests on this too (`!(a -o a * a), !(a * c -o
  b * c), a |- b`: without the clause that needs a `c`, nothing makes
  `b`). One pass, a count of missing inputs per transition.
- **Every count of the program fits a `u32`**: a weight counts literals
  of one clause, a marking's count literals of the goal, a place an atom
  or a class, and a forest has fewer than 2³² occurrences. Counts that
  firings grow are checked (`successor`), and passing `u32::MAX` is
  `Reason::IndexLimit`.

## The search (`reach.rs`)

- **Markings are kept sparse**: the marked places in order, each as its
  gap to the previous one and its count less one, in LEB128, so a
  marking costs bytes per token and not per place (Philosophers-10000 of
  the LLTP nets has some 50 000 places: dense `u32` vectors met the 1 GiB
  bound after 4 096 markings). The encoding is a function of the
  marking, so the table compares bytes.
- **Clauses used once cost their tickets in every marking**: a class
  of equal clauses is one place, but distinct clauses used once are a
  place each, and every marking holds a token per unused clause, so a
  chain of `n` distinct clauses used once costs `n²` bytes over its `n`
  markings (50 000 links meet the 1 GiB bound after 9 000 markings;
  under `!` the same chain is proved in 0.4 s). Such programs have no
  `!` and stay with the focused engines by default.
- **The frontier holds successors unwritten**: a key (the successor's
  distance to the target above, its parent's index complemented below)
  and the transition. A successor is written out only when it is taken,
  and dropped then if it is kept already (`memo_hits`); only markings
  taken and new are kept, and each is expanded at once. So the kept set
  is the expanded set, and the frontier is sixteen bytes a successor.
- **Order: nearest to the target first**, the distance being the tokens
  by which a marking and the target differ (computed incrementally per
  transition, `moved`), ties to the successors of the latest marking
  expanded. Greedy best-first, as the heuristic search of explicit-state
  net checkers does; it finds a firing sequence, not a shortest one. The
  order does not matter for a refutation, which needs every reachable
  marking expanded.
- **The goal is tested when a successor is generated** (distance zero),
  so the target is never kept.
- **Arithmetic on `usize` is checked** where a buffer's size is
  computed (`room`, `grow_table`, the frontier's growth, the proof's
  bytes): on a target of 32 bits a product could pass `usize::MAX`, and
  a size the bound cannot hold is a refusal there.
- **Limits, each a refusal tested in `refuses_at_its_limits`**: the
  markings kept (`MOST_MARKINGS`, their indices are `u32`), a count past
  `u32::MAX`, the proof's nodes (`proof::MOST_NODES`), and the memory
  bound, which counts every buffer that grows with the markings: the
  bytes, the ends, the hashes, the parents, the table and the frontier
  (`room`, checked before each growth). The buffers sized by the program
  (the dense counts of the marking at hand, the index of transitions,
  the enabled list) are not counted, like the forest.
- **An expansion costs the marked places and the transitions indexed by
  them**, never every place: the distance of a marking is the target's
  total, kept once (`target_total`), corrected over its marked places. A
  first version summed the target at every expansion, which made a
  chain of 160 000 places take 6.4 s instead of 0.17 s (the panel's
  finding).
- **A backward search beside the forward one** (`both`): once the
  forward search has kept `BACKWARD_AFTER` (2¹⁴) markings without
  deciding (a count of work started it after a few expansions of the
  library's largest nets, a quarter slower in the median), the same search runs on the reversed program
  (`Program::reversed`: every transition's inputs and outputs swapped,
  the initial and the target marking swapped), a unit of work for every
  `BACKWARD_SHARE` (4) of the forward's. Its finding its target, the
  forward initial marking, is a firing sequence read backward (the same
  transition indices); its exhausting its markings refutes, since every
  marking a firing sequence from the initial marking to the target
  passes is reached from the target backward. The places that no
  transition raises are capped at their initial count (`Program::caps`;
  a marking above a cap is dropped, counted with `memo_hits`): every
  marking such a sequence passes has at most that many, and without the
  caps the tickets of clauses used once grow without end backward. Nets
  decided within the first 2¹⁴ markings are searched as before, but for
  the order among successors at the same distance from the same marking,
  which the transitions' indices break and the dropped dead transitions
  shift. The reversed program and the backward search are made only when
  it starts.
  Measured on random programs near the Horn shape (step 27's report):
  with the state equation and the dead transitions, the backward search
  is what refutes the rest of what the focused engine refutes
  (`!(1 -o 1), !a, (a * a -o a) |- 1`: the goal has no predecessor).
  Out of room, the forward search takes back the simplex's memory, then
  the backward search's (`give_back`, which ends it, and gives nothing
  before the search started); the backward search out of room ends
  itself.
- **Polled** once per successor taken from the frontier: an expansion
  costs the transitions indexed by the marked places, which on the
  largest LLTP net (33 676 transitions) is well under a millisecond.
- **It runs on the calling thread** whatever `Options::jobs` says
  (`Engine::parallel` is false), so the command adds no pool beside it
  (`search::engine_for`, asked when the pool would start): a pool would
  run the same search again, at twice the memory bound.

## The backward search (`cover.rs`)

- **The algorithm** (Abdulla, Čerāns, Jonsson and Tsay, 1996): the
  markings from which the target can be covered are upward-closed and
  kept as their minimal elements. From an element `m` and a transition
  `t` with an output on a place `m` marks, `max(m − out(t), 0) + in(t)` is
  the least marking from which `t` leads to one at least `m`; it is kept
  unless an element is at most it (`memo_hits`), and a kept one at most
  the initial marking ends the search with the firing sequence `t`, then
  the transitions up the line of parents to the target. A transition
  without an output on `m`'s places gives a marking at least `m`, which
  `m` already covers, so only the transitions indexed by `m`'s places
  are tried (`by_output`).
- **Why it ends, and why the end refutes.** No element is kept that is
  at least an earlier one (that one, which stays in the index, would
  cover it), and Dickson's lemma says every infinite sequence of
  count vectors has an element at least an earlier one: the kept
  sequence is finite. An empty queue means every element's predecessors
  were computed and covered, so the elements' upward closure is the
  whole set of markings from which the target is coverable; every kept
  element was compared with the initial marking when computed, so the
  initial marking is outside, and the target is not coverable. Dickson
  bounds nothing: the memory bound and the stop do.
- **No element is removed.** An element that a later, smaller one covers
  stays in the index, harmlessly (whatever it covers, the smaller one
  covers), and is skipped when it comes out of the queue
  (`dominated(…, e)`), since what it would compute is covered by what
  the smaller one computes (`max(m − out, 0) + in` is monotone in `m`).
- **The index is a trie of the elements' places** (`edges`, `listed`,
  `after`): a path follows places in increasing order, and an element is
  listed at the node its places lead to. An element at most a marking
  has its places among the marking's, so `dominated` walks from the root
  along the marking's places only (an explicit stack of a node and where
  in the marking its children start) and compares the counts of the
  elements listed there of no more tokens (`sums`). At each node it
  steps through the smaller side: the node's children (`first_child`,
  `sibling`, `place_of_node`, counted in `children`), each kept if the
  marking marks its place after the node's (`position`, an index plus one
  per place, written with the probe), or the marking's places after the
  node's, each looked up in `edges`. Looking up every remaining place at
  every node cost `k²` lookups on markings of `k` places, and a goal of
  4 000 places that the scan by first place decided in 0.25 s ran past
  10 s (the second panel's finding; now 71 ms). It answers exactly
  as a scan of every element would: on 4 542 random affine programs and
  qcover's Mist and medical nets the counters equal those of the scan by
  first place that came before it, which took twice as long on
  `extendedread-write` and spent most of its time in buckets of tens of
  thousands of elements that share a control place. The nodes are `u32`
  (a refusal at `u32::MAX`); the table of edges is charged at its
  capacity, a slot and a control byte each.
- **Order: fewest tokens first**, ties by age; it changes no verdict.
- **Counts** grow backward by a transition's inputs, checked
  (`before`): passing `u32::MAX` is `IndexLimit`, tested with inputs of
  almost 2³²; the elements kept are at most `MOST_MARKINGS` (their
  indices are `u32`, and `tried` stores an index plus one).
- **Polled** at every element taken from the queue and every marking
  computed. The state equation runs beside it as beside the forward
  search, its work counted in transitions tried and elements compared.

## The state equation (`equation.rs`)

- **What refutes.** A firing sequence from `M₀` to `M` fires each
  transition `t` some `xₜ ≥ 0` times, so `M − M₀ = Σ xₜ·Cₜ`, `Cₜ` its
  effect (the tickets of clauses used once are places, so their count
  is in it). Integer weights `y` per place with `y·Cₜ ≤ 0` for every
  transition and `y·(M − M₀) > 0` make that impossible: `y·(M − M₀) =
  Σ xₜ·(y·Cₜ) ≤ 0`. In affine mode the marking reached `M'` need only be
  at least the target, and weights not below zero give `y·(M' − M₀) ≥
  y·(M − M₀) > 0` against the same `≤ 0`: there `certify` also asks
  every weight to be at least zero, and the tableau has a surplus column
  per row (`C·x ≥ M − M₀`), whose reduced cost at the optimum is that
  sign. That is Farkas' lemma for the rational relaxation;
  `certify` checks exactly those inequalities, in `i128` with checked
  operations (a weight below 2⁶³ times an arc's weight below 2³², fewer
  than 2³² terms: no sum reaches 2¹²⁷), over every transition of the
  `Program`, including the ones the tableau left out. Only a vector it
  accepts becomes `Refutation::StateEquation`; nothing else of the
  module bears on soundness.
- **Who proposes the weights.** The first phase of a revised simplex
  (`Tableau`): a row per place that a transition changes or the target
  names, signed so the right-hand side is not negative; a column per
  distinct effect of a transition, kept sparse, in affine mode a surplus
  column per row, and an artificial column per row; the basis as its
  dense inverse. A pivot costs `2m² + nnz` (`m` rows, `nnz` the columns'
  entries), not `m × columns`: qcover's bug-tracking nets have 754
  places and 26 672 distinct effects, where a dense tableau of 21 million
  entries took 20 ms a pivot and never finished in 5 s. The steepest
  column enters (Dantzig), and after `STALLED` (64) pivots without
  progress the first improving one (Bland), with the first basic column
  among tied rows leaving; tolerance 10⁻⁹. At the optimum an objective
  above zero means no solution, and the prices `c_B·B⁻¹`, the row's sign
  applied, are the weights. Floating point, so they are read
  as fractions of denominator at most 2²⁰ (continued fractions), brought
  to integers by the common denominator (at most 2⁴⁰) and divided by
  their gcd; a vector that rounding spoiled fails the check and costs a
  refutation, never makes a wrong one. Weights below 10⁻⁹ of the largest
  read as zero, so certificates whose weights span more (`!(a1 * a1 -o
  a2), !(a2 -o a1 * a1), …` to `a40`, exact weights `2ⁱ`) are missed; a
  weight that is no finite number is no certificate (the continued
  fraction of NaN would not end). No crate: an LP solver from
  crates.io would neither poll the caller's stop nor charge the memory
  account, and `microlp`, the maintained pure-Rust one, reads a clock.
- **Interleaved with the search by work** (`Equation::wants`, `run`):
  the search counts its work (transitions examined and markings written
  in `reach.rs`; transitions tried, elements compared and the trie's
  lookups in `cover.rs`), and the simplex may touch
  `LINEAR_ENTRIES_PER_UNIT` (4, about a fifth of the time, beside a
  search that on the nets of practice mostly proves) or
  `AFFINE_ENTRIES_PER_UNIT` (16, about half, since coverability problems
  from verification are mostly safe and the equation refutes most of
  them) entries per unit (`Equation::budget`). Its set-up waits until
  that budget reaches three times the places squared plus the arcs (a
  basis of up to a row per place laid out, a first pivot touching it
  twice, every arc read once, each transition's effect merged from its
  sorted arcs in one pass), and a pivot is made only within the budget:
  so a net the search decides at once never builds one, and one it
  cannot decide gets the simplex at no more than its share. With the
  share at 16 for both, and the set-up waiting for the places squared
  alone, the library's nets took 1.6 times as long in the median of a
  sample of 50 (2.2 ms against 3.7 ms); now the median is the first
  session's. The first panel of the second session found the set-up
  built at the first poll whatever the budget, with a quadratic merge:
  a clause of 100 000 outputs took 2.6 s where the search alone took
  22 ms, a chain of 5 000 clauses 279 ms against 3.5 ms now. The library's nets are all reachable, so there the
  equation can only cost. The tableau is charged to the search's
  account (the inverse, the vectors of a row each, the columns' entries);
  one that does not fit is never built (qcover's largest net, 66 950
  places, would need 36 GB). Bland's rule ends in exact arithmetic; in
  floating point the simplex gives up after `64 × (columns + 2m)`
  pivots. The stop is polled at every pivot.
- **The search never loses a decision to the simplex's memory.** The
  tableau is charged to the search's own account, so when the search has
  no room for a marking, an element or the frontier, the simplex gives
  its tableau back (`Equation::release`, state `Yielded`), and in linear
  mode the backward search its markings, both at once, and the search
  tries once more; the simplex then waits for the search's end. A
  simplex whose basis does not fit beside the search waits the same way
  (`Yielded`, not `Done`), so that `finish` gives it the memory the search
  gave back. The second panel found both: the release stopped at the
  simplex while the backward search kept its markings, and a basis that
  did not fit was never tried again. Before
  a proof is built the tableau goes too. The first panel of the second
  session found goals the search proved before ending at the memory
  limit, the tableau of a chain of 8 190 clauses taking the whole GiB.
- **After the search runs out of room** (`MemoryLimit`, `IndexLimit`) its
  memory is given back and the simplex runs to its end with the rest of
  the time (`Equation::finish`, starting afresh if it had yielded): an
  unbounded net is what fills the memory, and what the equation is for.
  The reason then reported is the search's, after the time the simplex
  took too.

## The proof (`proof.rs`)

- Built after the search, premises first: the goal's tensors with an
  axiom per body literal, in affine mode a `Weaken` for each token left
  beside the goal's (those the replay leaves) and each clause used once
  that no firing used, then each firing from the last to the first
  (the clause's tensors over axioms for its body and the proof so far
  above its head, wrapped in the head's `⅋` and `⊥`; a `Copy` for a
  clause under `?`), then the markings' `⅋` and `⊥`, then a `Quest` per
  member under `?`. Subtrees are walked in decreasing id order, which is
  post-order with the right factor first, so a tensor pops its left
  proof, then its right one: no recursion over a formula.
- **Which token a body literal takes** comes from a forward replay of
  the firings on stacks of head-literal occurrences per place (`replay`),
  recorded in the order the backward build consumes them.
- The proof's nodes, its pairs, the tokens of the replay and the clauses
  fired with where each one's pairs start are charged to the account
  before it is built (the tokens and the clauses were missed until a
  panel found them); the builder reads each firing's pairs where the
  replay wrote them, the goal's last, with no second copy (a second
  panel found the copy costing proofs at tight bounds: a 10-bit counter
  needed 303 KB, now under 262 KB, where the first session's charge,
  which left the tokens out, said 197 KB). `nodes > most` is
  `IndexLimit`. The checker at the end of
  `prove_goal` judges it like every engine's.
