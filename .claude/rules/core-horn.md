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
  search over markings; `equation.rs` the state equation and the exact
  check of its refutation; `proof.rs` the proof read off a firing
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
- **Affine mode is refused** (`Error::EngineMode`): there a proof may
  weaken leftover tokens and unused clauses, which makes the question
  coverability, a different search.
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
- **Polled** once per successor taken from the frontier: an expansion
  costs the transitions indexed by the marked places, which on the
  largest LLTP net (33 676 transitions) is well under a millisecond.
- **It runs on the calling thread** whatever `Options::jobs` says
  (`Engine::parallel` is false), so the command adds no pool beside it
  (`search::engine_for`, asked when the pool would start): a pool would
  run the same search again, at twice the memory bound.

## The state equation (`equation.rs`)

- **What refutes.** A firing sequence from `M₀` to `M` fires each
  transition `t` some `xₜ ≥ 0` times, so `M − M₀ = Σ xₜ·Cₜ`, `Cₜ` its
  effect (the tickets of clauses used once are places, so their count
  is in it). Integer weights `y` per place with `y·Cₜ ≤ 0` for every
  transition and `y·(M − M₀) > 0` make that impossible: `y·(M − M₀) =
  Σ xₜ·(y·Cₜ) ≤ 0`. That is Farkas' lemma for the rational relaxation;
  `certify` checks exactly those inequalities, in `i128` with checked
  operations (a weight below 2⁶³ times an arc's weight below 2³², fewer
  than 2³² terms: no sum reaches 2¹²⁷), over every transition of the
  `Program`, including the ones the tableau left out. Only a vector it
  accepts becomes `Refutation::StateEquation`; nothing else of the
  module bears on soundness.
- **Who proposes the weights.** A dense tableau of the simplex's first
  phase (`Tableau`): a row per place that a transition changes or the
  target names, signed so the right-hand side is not negative; a column
  per transition with an effect, an artificial column per row. Bland's
  rule (first improving column, first basic column among tied rows) at
  a tolerance of 10⁻⁹; at the optimum, an objective above zero means no
  solution, and the duals read off the artificial columns' reduced costs
  (`yᵢ = σᵢ(1 − dᵢ)`) are the weights. Floating point, so they are read
  as fractions of denominator at most 2²⁰ (continued fractions), brought
  to integers by the common denominator (at most 2⁴⁰) and divided by
  their gcd; a vector that rounding spoiled fails the check and costs a
  refutation, never makes a wrong one. No crate: an LP solver from
  crates.io would neither poll the caller's stop nor charge the memory
  account, and `microlp`, the maintained pure-Rust one, reads a clock.
- **Interleaved with the search by work** (`Equation::wants`, `run`):
  the search counts its work (transitions examined and markings written
  in `reach.rs`), and the simplex may touch `ENTRIES_PER_UNIT` (16)
  tableau entries per unit, its setup counting the whole tableau; so a
  net the search decides at once never builds a tableau, and one it
  cannot decide gets the simplex at no more than about the time the
  search had. The library's nets are all reachable, so there the
  equation can only cost. The tableau is charged to the search's
  account; one that does not fit is never built. Bland's rule ends in
  exact arithmetic; in floating point the simplex gives up after `64 ×
  (rows + columns)` pivots. The stop is polled at every pivot.
- **After the search runs out of room** (`MemoryLimit`, `IndexLimit`) its
  memory is given back and the simplex runs to its end with the rest of
  the time: an unbounded net is what fills the memory, and what the
  equation is for.

## The proof (`proof.rs`)

- Built after the search, premises first: the goal's tensors with an
  axiom per body literal, then each firing from the last to the first
  (the clause's tensors over axioms for its body and the proof so far
  above its head, wrapped in the head's `⅋` and `⊥`; a `Copy` for a
  clause under `?`), then the markings' `⅋` and `⊥`, then a `Quest` per
  member under `?`. Subtrees are walked in decreasing id order, which is
  post-order with the right factor first, so a tensor pops its left
  proof, then its right one: no recursion over a formula.
- **Which token a body literal takes** comes from a forward replay of
  the firings on stacks of head-literal occurrences per place (`replay`),
  recorded in the order the backward build consumes them.
- The proof's nodes and pairs are charged to the account while it is
  built; `nodes > most` is `IndexLimit`. The checker at the end of
  `prove_goal` judges it like every engine's.
