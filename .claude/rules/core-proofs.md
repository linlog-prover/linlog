---
paths:
  - "core/src/proofs/mod.rs"
  - "core/src/proofs/check.rs"
  - "core/src/proofs/oracle.rs"
---

# linlog core: proof terms and the checker

Loaded, beside `core.md`, when the proof terms or the checker are read.
The derivation view, the size estimate and interactive proving, which
are observers of the checker's pass, are in `core-derivations.md`.

## Proofs are terms over occurrence ids

`Proof` (`proofs/mod.rs`) owns its `Forest` and an arena `Box<[Node]>` of
rule instances; `Node` is a 16-byte `Copy` enum (a static assertion pins the
size): the rule, the member it acts on (`Member`, `core-forest.md`), and
the premises as `NodeId`s.
The rules are the dyadic calculus's (`⊢ Θ ; Γ`): `Quest` moves a formula
into `Θ`, `Copy` uses a `Θ` formula without consuming it, `Bang` needs an
empty linear zone; `Weaken` exists for affine mode and `Mix` for Mix. The
ILL rules have no tags of their own: on the lowered sequent (D1) every one
is a classical rule (`⊸L` is `⊗`, `⊸R` and `⊗L` are `⅋`, `&L` is `⊕`, `⊕L`
is `&`, `1L` is `⊥`, `0L` is `⊤`, `!L` is dereliction), so the same terms
serve intuitionistic mode. Invariants:

- **A premise precedes its conclusion** (strictly smaller index), the root
  is the last node, and every node is reachable from the root. `Proof::new`
  verifies the bounds and the order of what the root reaches, drops the
  rest (an engine's arena holds the subproofs of failed branches) and
  renumbers; it does not check the proof. A proof has at least one node
  and at most 2³² − 1 (`Refusal::Index { what: Space::Node }`; reading a proof file
  refuses the same), which is what makes the number of nodes, and the
  checker's count of a node's readers, a `u32`. A subproof two nodes share (a
  memo hit) is stored once, so the arena is a DAG and the derivation view
  unfolds it.
- A node never records the sequent it proves; the checker derives it. So
  `Top(o)` does not say what context the `⊤` absorbs, and an engine need
  not record it.
- **A proof records what it concludes and the mode it is meant for**
  (`goal`, `None` for the sequent's roots, and `mode`, a claim the check
  tests in the mode it is given): `Proof::new` makes a proof of the
  sequent, `new_of_goal` one of a goal, `with_mode` records the mode, and
  the checker, the size pass and the derivation conclude at
  `Proof::conclusion()`. Where a proof of the sequent is needed a goal
  proof is `Error::GoalProof` (kind unsupported: no wrong proof):
  `ProofStructure::from_proof`, the Rocq writer (through
  `Derivation::is_of_goal`) and `linlog check`; `Interactive::close_with`
  refuses a proof of another goal with `Error::GoalMismatch`. Neither key
  is on the wire yet: the wire level adds them.
- Occurrence ids are those of the proof's own forest; a proof is meaningful
  only with it. Serialization stores the sequent and rebuilds the forest,
  which is deterministic (D5).
- In affine mode the spec's relaxed rules (a non-empty context under `!`
  or `1`, `Γ ⊋ {p⊥}` in the axiom) are not rules of the term: the engine
  emits `Weaken` nodes *below* the `!`, `1` or `ax` for the surplus. The
  checker's `!`, `1` and `ax` are the same in both modes. `Weaken` of a
  `?` formula is the standard `?w` and is allowed in every mode; the dyadic
  engines express the same thing as a `Quest` whose formula goes unused.

## The checker

`proofs/check.rs` is the reference for what a proof is. It shares no code
with any engine and must not: an engine's proofs are validated by something
that cannot repeat the engine's mistakes. Engines only call `Proof::check`.

- One bottom-up pass in arena order, no recursion (`Pass`). Each node
  gets a `State { theta, gamma, any, … }`: `gamma` is the linear zone as
  a multiset (`Bag`, a table of counts: copies can repeat an occurrence
  and every subformula of a copied formula), `theta` the **least**
  unrestricted zone the subproof needs (`Copy` adds the occurrence,
  `Quest` removes its subformula), and `any` says a `⊤` above absorbs any
  further linear context. The claim the tests and the review rest on:
  `State` characterises exactly the set of dyadic sequents the subterm
  proves. At the root, `theta` must be empty (every copy has its `?` step
  below it) and `gamma` must equal the roots, or be a sub-multiset of them
  under `any`.
- **The pass holds only the sequents a later node still reads.**
  `readers` counts, per node, the premise references to it; a node's
  `State` is built from its premises' own, the last reader *taking* a
  premise's state and changing it in place, an earlier reader a clone,
  and a state with no reader left is gone. Both zones are hash tables of
  their members (the crate's fixed-seed hasher), so a state costs what it
  holds and nothing for the forest's width, an insertion or a removal
  costs the same whatever the zone's size, and the two-premise rules pour
  the smaller table into the larger. The number of members in output
  position is kept in the state, never recounted (a recount per node made
  the intuitionistic check quadratic in a zone of thousands). What this
  buys, and what it does not: on a proof without shared subproofs the
  live states are those of disjoint subtrees, each no larger than twice
  its subtree, so the memory is linear in the proof whatever the order of
  the arena; a node read by several others is cloned for all but the
  last, so a proof whose shared nodes have large zones takes
  nodes × zone (the weakenings of one large sequent under a tower of `&`
  is the shape), which no engine's proofs were seen to do and which the
  pass refuses past its bound (next item). The first
  implementation kept every node's sequent with a bitset as wide as the
  forest (6 GiB on a net of 65 000 clauses whose proof the search finds
  in 43 ms; the new pass checks it in a few milliseconds within the
  search's own memory). It is kept as `proofs/oracle.rs`, compiled for
  tests only, and `agrees_with_the_first_implementation` requires the
  same verdict *and the same error* from both on the engines' proofs of
  the generated sequents and the families, in every mode, and on mutants
  of them: a change to a rule is made in both, or the test says where
  they part.
- **The pass counts what it holds and refuses to pass its bound**
  (`Proof::check_within(mode, &limits, stop)` within
  `limits.memory_bytes`, `check(mode)` being that within the default,
  1 GiB; `None` for no bound; `limits.work` bounds the units of work,
  and `stop` is asked every 4 096 nodes or `POLL_WORK` (65 536) units,
  whichever comes first). **A unit of work is a node, or an entry of a
  premise's sequent handed to its rule** (`State::entries`, counted in
  `Pass::premise`): a rule copies, pours or compares what it is handed,
  so a node read by many others with a large sequent costs each of them
  that sequent, and the memory bound does not see it when every copy is
  consumed at once. Counted by nodes, a file of 8.6 MB (a `⊤` under
  128 000 `⊥` steps, then 128 000 `&` nodes each reading that node) took
  83 s between polls seconds apart, within 49 MB (H25,
  `counts_the_work_of_shared_premises`); such a check is now stoppable
  and bounded. It stays quadratic: the sequents are what the proof
  means, and comparing them per `&` is the work. The refusal is
  `CheckError::Refused(Refused { node, refusal: Refusal::Memory { phase:
  Check, .. } })` at the node the pass had come to, a variant of its own
  beside every fault of a proof (`CheckError::Invalid`): **a refusal is
  no verdict**, and a front end must never print it as "invalid"; its
  `ErrorKind` is `Limit`, its code `memory_limit`. Inside the pass a
  fault and the bound are `Halt`, which `examine` splits. What is counted
  (`Pass::held`): twelve bytes per node for the pass's two tables and
  every state in `live` or in the hands of the current rule at
  `State::bytes`, which is the value plus its two tables; and, added at
  every comparison with the bound, what the observer says it holds by
  then (`Observer::bytes`: the size estimate's tables, the sequents the
  derivation's record has kept). A table is counted by the most members
  it ever held (`Zone::most`, `Bag::most`, through `table_bytes`), not
  by its members now and not by `capacity()`: the standard library's
  table never gives slots back, a clone has the slots of its original
  whatever it holds, and `capacity()` goes down with every tombstone, so
  a zone that was large and then consumed would be held for nothing and
  counted as nothing. Counted that way a table that only grew is exact
  to the byte, and one that members came and went from is at most four
  times what is counted (a doubling forced by tombstones at half load;
  twice in a random churn). A copy is charged before it is made, the
  premises a rule was handed (`Pass::passed`) are taken off when its
  own state is charged, and the count is compared with the bound at
  every charge; the tables themselves are asked for before they are
  allocated (`check::afford`, which the size estimate calls before it
  makes its own, 72 bytes a node). Not counted: what one rule needs while it joins two
  states, the one list of shared occurrences, the sequents of an error
  report, each a small multiple of one counted state, and what is
  proportional to the forest (the reading, the weights' scratch
  tables). Measured on a proof file of 1.1 MB (46 768 nodes: 14 000 `⊥`
  formulas introduced once, then read by the 16 384 leaves of a balanced
  `&` tree, all before the first `&`): without a bound the pass holds
  2.31 GiB by its count and the process's peak grows by 2.31 GiB (ratio
  0.999) in 1.45 s; within 64 MiB it is refused after 28 ms with 62 MiB
  held, within the default after 0.40 s with 1 022 MiB.
  `holds_no_more_than_its_bound` pins the file's term.
- **An error costs a second pass.** The states a failing node read are
  gone or changed by the time it fails, so `examine` runs the pass again
  up to that node and reports its premises' sequents from there; they
  are kept then, since the node itself has yet to read them. `Fault`
  is found by the first pass; the second does exactly what the first
  did up to the node, so it holds no more and cannot fail before it. A
  refusal has no second pass, which would take the memory that was
  refused: its `premises` are empty.
- **`examine(proof, goal, mode, reading, memory, observer)` is the one
  pass behind everything**: `check` is it on the roots with no observer;
  the derivation view and the size estimate are observers (`Observer`:
  every node's `State` in arena order and the `Facts` of how its rule
  applied: `used`, `shared`, `absent`, `needs`), so what
  they know of a proof is what the checker derived, never a second
  reading of the rules, and every one of them is under the bound. An
  observer may give occurrences a `weight` (a `u32`), which a state adds
  up over its zones as they change (`State::weight`, `goal_weight`); the
  checker's own observer is `()` and the sums are zeros. A goal other
  than the roots is checked the same way (`Derivation::of_goal`).
- **No integer of the pass wraps, in any build**, and each says why at
  its declaration. The arguments, all from four facts: a proof has
  fewer than 2³² nodes, a node has two premises at most, a forest has
  fewer than 2³² − 1 occurrences, and a state that a rule or an
  observer reads has passed `within`, so its linear zone has at most
  `Bag::MOST` = 2³² − 2 members. `readers` (`u32`): one node has at most
  as many readers as the proof has nodes, because every later node but
  the root must itself be somebody's premise. `Bag::counts` and
  `Bag::len` saturate, and a saturated one is over `MOST`
  (`Fault::Surplus`). `State::outputs`, `linear` and `goal` saturate
  while a rule builds a state, which only a zone over `MOST` can make
  them do, and are exact when read; every subtraction (`take`) is on a
  premise's exact sums and comes before any addition of its rule.
  `State::unrestricted` is at every moment the sum over a set of
  occurrences, below 2⁶⁴ with `u32` weights. `State::weight()`, the two
  zones together, saturates, and the size estimate carries that to its
  result (below). `Pass::held` never exceeds what the pass has
  allocated, and saturates all the same. The second pass no longer adds
  a reader to pin the premises (the one `+= 1` that could have passed
  `u32::MAX`).
- **A zone the rest of the proof cannot consume is refused where it
  arises** (`Pass::within`, `Fault::Surplus`): a rule takes two members
  of a premise's zone at most and passes the others on, and the root's
  zone lies within the goal, so node `i` of `n` may derive at most
  `|goal| + 2·(n − 1 − i)` members, and never more than `Bag::MOST`.
  This is a soundness rule, not a convenience: a term may double a zone
  at every node (`Mix(p, p)`, or a `⊗` on a `⊤` premise taken twice), a
  proof file can hold such a term though no engine builds one, and the
  zones are counters of a fixed width. Before the rule the counters
  wrapped in a release build: a file of 131 nodes that mixes 2⁶⁴ copies
  of `⊢ 1`, promotes `⊥` over them (the zone's length read zero) and
  mixes one more in was a "valid proof" of `⊢ !⊥, 1`, which has none
  (`refuses_a_zone_too_large_to_conclude`). Now the counters saturate
  and a saturated one is always over the bound, so every count a rule
  reads is the true one; the first implementation has the same rule, so
  the two still agree on the error. The rule also bounds what a
  malformed term costs: every zone, and so every error report, is within
  the goal plus twice the nodes. In ids, that is: `describe` writes every
  member as its formula, so a report with formulas is members times
  formula text, and a file that shares subformulas deeply makes it large
  (65 MB from a file of 47 KB was measured): `Described::abbreviated`
  cuts every formula and every list of formulas after a number of
  characters, which the command takes from `--abbreviate`. The weights an observer adds up are
  not part of the argument: they are sums of at most that many formula
  sizes.
- The `any` flag is what makes `⊤` checkable without a recorded context:
  consuming a subformula from an absorbing premise succeeds when it is
  absent; `⊗` and Mix sum the zones and or the flags; `&` needs equal zones,
  or the absorbing side's zone included in the exact side's (result exact),
  or the pointwise maximum when both absorb; promotion needs the zone empty
  after its subformula and resets the flag, because `⊢ !A, Δ` holds only for
  `?` contexts, which `Θ` already covers. The derivation view instantiates
  the absorbed context top-down.
- **Intuitionistic mode is the classical check plus the one-succedent
  condition** against the sequent's `Reading` (`Fault::Shape` when there
  is none). Every ILL rule is a classical node, so what is checked is only
  that every sequent of the proof has one goal: (R1) every derived `gamma`
  holds at most one occurrence in output position, and exactly one unless
  `any` (a `⊤` above supplies the goal); (R2) in `take`, an absent child in
  output position may be absorbed by `any` only if the premise's zone has
  no output already (else the premise's sequent would have two goals);
  (R3) `Weaken` never weakens an output; Mix is `Forbidden`. Any failure
  is `Fault::Succedents { count }`. These are sound and complete for "some
  top-down instantiation of the absorbed contexts is an ILL derivation":
  `Ax` has one output by construction, `Bang` resets `any`, `Θ` holds only
  input occurrences (subformulas of `?`), and at a `⊗` the fixed output
  counts of the two premises sum to two, so the side with none must
  absorb, which R1 guarantees. The derivation view builds that
  instantiation. A review compared the checker with an independent
  two-sided prover on 8 000 classical proofs of ILL-shaped sequents.
- The rule interpretation was reviewed against an independent top-down
  reference checker with random proofs, mutants and random terms (step 2's
  report); the intricate cases that review named are pinned in
  `check.rs`'s `accepts_every_rule`, so keep them when the checker changes.
- `CheckError` reports ids, not formulas: `node`, its `rule`, the derived
  `premises` as `Dyadic` sequents and a `Fault` (`CheckError::Invalid`'s
  `Invalid`; a refusal has no premises). `Display` prints ids too;
  `describe(&owner)` (the proof or its forest) prints the same message
  with formulas (nodes keep their ids) as the crate's one `Described`,
  which is what the CLI shows. Both go through one writer
  (`Invalid::write`), so a new `Fault` gets one arm (and one in its
  written form, `serialize/errors.rs`), and the test-only
  first implementation (`oracle.rs`) reports the same faults.

## Decisions

The author's answers for the release (`plan/notes/api.md` §14), which
the fixes implement and later rounds judge against. Where a bullet above
still describes code that a decision changes, the decision holds, and
the commit that lands it rewrites that bullet.

- **A proof records its conclusion and its claimed mode** (`goal`,
  `mode`), and the checker checks a term against its own conclusion. A
  goal proof is then never taken for a proof of the sequent. `linlog
  check`, `from_proof` and the Rocq writer require a proof of the sequent
  (`GoalProof`, kind unsupported, otherwise).
- **`Rule` is a one-sided rule, and `Named` adds the side of `⊢`**: a
  two-sided rule name is a `(rule, side)` pair, not a variant. Consumers
  lose their unreachable arms, and a new connective adds one rule, not
  three.
- **No nullary Mix**: the calculus, the checker and the Rocq library stay
  as they are, and a net places weakening and `⊥` by a jump chosen from
  the term. A rule the Rocq library freezes is cheaper to add later as a
  node than to carry now.
