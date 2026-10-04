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
  `is_program`, the shape test alone; `reach.rs` the
  search over markings; `proof.rs` the proof read off a firing
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
- **Why a proof is a firing sequence, and a refutation sound.** Call a
  subformula an output when it is the goal or lies in a body (a body
  literal, a `1`, a tensor of them), an input otherwise (markings, heads,
  clauses, the tensors on the path to a head). In every rule of a
  proof, `q − 1 = (Mix rules above) + (outputs weakened above)`, `q` the
  outputs of the sequent: an axiom pairs one body literal with one head
  literal (`q = 1`), `1` is an output, a tensor on a clause's path to its
  head gives its body factors' premises one output each and takes it,
  `⅋`, `⊥`, copies and weakening of inputs keep `q`. The root has
  `q = 1`, so no proof of such a goal uses Mix or weakens an output, and
  every sequent of it has exactly one output: it is an ILL proof, and
  ILL proofs of Horn sequents are firing sequences (Kanovich, "The
  complexity of Horn fragments of linear logic", 1995). Hence
  `Unprovable` from an exhausted set of reachable markings is sound in
  classical and intuitionistic mode, with or without Mix. The argument
  is in prose here and in the step's report; no test can exhaust it.
- **Intuitionistic mode** additionally requires the reading to put
  exactly the outputs above on the right of `⊢` (`Program::read` checks
  every occurrence's `Position`): the proof built has one output on
  every sequent, so the one-succedent check accepts it only if the
  reading agrees. A goal whose reading differs is no program for the
  engine, never a wrong proof.
- **Affine mode is refused** (`Error::EngineMode`): there a proof may
  weaken leftover tokens and unused clauses, which makes the question
  coverability, a different search.

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
- **Limits, each a refusal tested in `refuses_at_its_limits`**: the
  markings kept (`MOST_MARKINGS`, their indices are `u32`), a count past
  `u32::MAX`, the proof's nodes (`proof::MOST_NODES`), and the memory
  bound, which counts every buffer that grows with the markings: the
  bytes, the ends, the hashes, the parents, the table and the frontier
  (`room`, checked before each growth). The buffers sized by the program
  (the dense counts of the marking at hand, the index of transitions,
  the enabled list) are not counted, like the forest.
- **Polled** once per successor taken from the frontier: an expansion
  costs the transitions indexed by the marked places, which on the
  largest LLTP net (33 676 transitions) is well under a millisecond.

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
