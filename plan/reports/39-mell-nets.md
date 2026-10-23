# Step 39: MELL proof nets with exponential boxes

## Session 39a: the structure, the criterion and the conversions

**Outcome: MELL proof nets with boxes exist in the library, reviewed;
the drawing, the JSON form and the command are 39b's.** `MellStructure`
(in `core/src/nets/mell/`) is a proof structure of unit-free MELL in
classical mode, with or without Mix: its vertices are instances of the
forest's occurrences, a `!` vertex opens a box, a `?` vertex has any
number of copies of its subformula (the generalised `?` node), and a
`?` vertex without copies, a weakening, hangs on an axiom link. Its
criterion `is_correct` is Danos–Regnier at every box depth with the
boxes inside contracted and the `?` vertices outside standing as doors,
independent of any engine, with a switching cycle or a disconnection and
the box it lies in as the witness of failure. `sequentialize` turns a
net into a dyadic proof term that the checker accepts in the structure's
mode, `from_proof` turns a checked term into its net, and the round trip
is the identity up to the order of a `?`'s copies and the links the
weakenings hang on. The MLL `ProofStructure`, the net engine and every
counter of the net target set are unchanged.

The fresh-context reviewer and the three panelists found nothing that
breaks soundness or completeness. About 1.6 million structures were
compared with a switching enumeration written from the definition, and
about 49 000 with a second, brute-force sequentialization reference;
every verdict agreed, and every witness was a true one. Every
structure accepted sequentialized into a term the checker accepts, and
every checked term desequentialized into an accepted net. Two findings
held, and both are answered. First, the round trip net → term → net is
not the identity when an occurrence has two vertices, because a term
names occurrences: the claim, the docs and the round-trip test now say
so. Second, a disconnection's witness and one step of a stage were
quadratic and asked no stop: they are now linear. The memory estimates
now cover what the construction holds while it builds. Every code commit
was gated, and `nix flake check` passed at the last commit.

**Canonicity, plainly.** The net of a proof is canonical only up to
which axiom link each weakening hangs on: `from_proof` takes the first
link that the subproof of the weakening's rule makes. A loose weakening
cannot be checked, because a criterion that counts the net's parts
accepts `⊢ ?c ⊗ (a ⅋ (b ⅋ ~b)), ~a`, which has no proof. So two terms
that differ only in the order of their rules can give different nets in
two cases. In the first, moving a `?` rule moves the first link above
it. In the second, a sequent holds two instances of one occurrence (two
copies of one `?` formula), and the term does not say which instance a
rule acts on, nor which of two `?` vertices of one occurrence a copy
behind a box door belongs to. With no occurrence of two vertices, the
net is canonical up to the weakenings' links.

### What was built

- **`MellStructure`** (`nets/mell/mod.rs`), with its data form
  **`Parts`** (the non-root vertices as `Placed`, each its occurrence,
  its parent and the `!` vertex of its innermost box; the links; the
  weakenings, each a `?` vertex and the literal it hangs on), validated
  by `from_parts` within `limits.memory_bytes`, and returned by `parts`.
  The roots are vertices `0..r` in the forest's order. Accessors for a
  front end: `vertices`, `occurrence`, `parent`, `premises`, `inside`,
  `boxes`, `auxiliary_doors`, `partner`, `links`, `hangs_on`.
  `MellStructure::exist(fragment, mode)` says where such nets exist.
- **The criterion** (`is_correct`, `nets/mell/graph.rs`): one graph of
  all box depths side by side, each depth with its boxes contracted to
  their `!` vertex and a door vertex for every `?` vertex outside with a
  wire into it; `?` vertices and doors switched like a `⅋` of any arity
  over their wires; Yeo's deletion test as in MLL; the count of kept
  edges per depth for connectedness without Mix.
- **Sequentialization** (`nets/mell/sequentialize.rs`) through boxes:
  the splitting tensor lemma at every depth, `?` rules with their copy
  rules, promotions when a box's `!` vertex is all that is left.
- **Desequentialization** (`nets/mell/desequentialize.rs`): the checked
  term unfolded as a tree, read from the leaves; refused before anything
  is made when it unfolds past the vertices a structure holds or the
  memory bound.
- **Errors**: `NetError::Placement` (with the fault, a `Placement`),
  `Unattached`, `InBox` (a failure inside a box, wrapping the depth's
  `SwitchingCycle` or `Disconnected`) and `NotMell`, in JSON the kinds
  `placement`, `unattached`, `in_box` and `not_mell`; `describe` prints a
  structure's vertices as formulas through the sealed `Owner::vertices`.
- **Tests** (`nets/mell/tests.rs`): `classic_nets` (fixed nets, their
  terms and witnesses, the weakening that a count of components would
  take for a net), `round_trips` (random proofs of the focused engine,
  with and without Mix), `agrees_with_the_multiplicative_criterion` (the
  verdict of `ProofStructure` on MLL), `agrees_with_enumeration` (a
  brute-force switching enumeration from the definition, on nets and
  their mutants), `an_unfolding_past_the_vertices_is_refused` (a term of
  100 nodes and 3·2⁴⁸ vertices).

### The `?` node, and why

**One generalised `?` vertex per `?` instance, with any number of copies
of its subformula, unordered, each at its own depth or in a box inside
its own; a `?` vertex without copies is a weakening and hangs on an
axiom link of its box or of a box inside it.**

- *Canonical desequentialization.* The generalised node absorbs what a
  term fixes and a net should not: the order and grouping of
  contractions, dereliction as one copy, and whether a contraction
  happens inside a box or outside it (Accattoli, arXiv:1303.7326, §7,
  the "nouvelle syntaxe" of n-ary `?`-links). It is also what linlog's
  dyadic terms say: a `?` rule moves the subformula into `Θ`, a copy uses
  it, nothing records an order. With dereliction, contraction and
  weakening as separate nodes, two terms that differ by the place of a
  contraction would give two nets.
- *Checkable weakening.* A weakening with nothing to hang on cannot be
  checked by Danos–Regnier, which needs connectedness, nor by counting
  components: ACC#w (acyclic, `#w + 1` components) is necessary and not
  sufficient (Di Donna and Tortora de Falco, FSCD 2025, Remark 21,
  Regnier's counterexample with a weakening under a `⊗`). Derived here:
  `⊢ ?c ⊗ (a ⅋ (b ⅋ ~b)), ~a` has two components under every switching
  and one weakening, and no proof (the only rule at the bottom splits the
  `⊗`, and `⊢ ?c` and `⊢ ?c, ~a` are unprovable). Guerrini and Masini
  (*Parsing MELL proof nets*, TCS 254, 2001; read in their IRCS report
  96-37 of the same title and abstract) glue every weakening formula to
  an axiom, so that nets are connected and Danos–Regnier applies "without
  the need to refer to connected components" (their Remark 2). That is
  linlog's dyadic calculus exactly: an axiom allows any `Θ`, so a `?` rule
  whose subformula goes unused is weakened at the axioms above it, and a
  promotion takes all of `Θ` (their `!?` rule).
- *What stays non-canonical.* The link a weakening hangs on: there is no
  canonical position for it in general (Di Donna and Tortora de Falco,
  §4, on jumps). `from_proof` takes the first link that the `?` rule's
  subproof makes. And where one branch of a term holds two `?` vertices
  of one occurrence, the term (whose `Θ` is a set of occurrences) does
  not say which of them a copy belongs to; `from_proof` gives it to the
  innermost, as the checker reads `Θ`, and `sequentialize` puts each `?`
  rule right below its own copy rules so that reading the term back gives
  every copy at the `?`'s depth to its own `?`.

### Where the criterion plugs in

It stands apart from `Correctness`: that trait is the linking search's
interface (constant-time rejections of a candidate link, `link` and
`unlink` on a stack, an exact test at a cadence, a balance prune), and
nothing searches MELL nets: the search stays with the focused engine,
and boxes and copies are not chosen by linking. So the criterion is
`MellStructure::is_correct`, built once over a fixed structure, and the
net engine and `ProofStructure` are untouched.

### The criterion and its sources

A structure is a proof net when every literal is linked, every weakening
hangs, and at every depth (outside every box, inside each box) no
switching has a cycle and, without Mix, every switching is connected.
The sources say this in parts: Straßburger (*Proof nets and the identity
of proofs*, INRIA RR 6013, §3.1) gives Danos–Regnier with `?c` switched
as `⅋` and "each box has to be treated separately", valid "only if no
instance of `?w` is present"; Di Donna and Tortora de Falco (§5) extend
a criterion to MELL "by requiring that it is satisfied by the ground
structure and, inductively, by the content of every box"; Guerrini and
Masini (Definition 4) quantify over every set of boxes collapsed to a
single link. The per-depth form with doors is the box-by-box reading;
that it decides linlog's calculus is derived, not quoted: every checked
term's net passes (by induction on the term, each rule keeping every
depth correct, the weakening's link in its `?` rule's subproof), and
every net that passes sequentializes into a checked term (below).

### Sequentialization

A depth's graph is a multiplicative structure in which a `?` vertex or a
door is a `⅋` of any arity over its wires, a box with its `!` vertex a
hypothesis whose conclusions are the `!` formula and the wires entering
it, and a link with the weakenings on it an axiom with more conclusions.
Laurent's splitting theorem (*An introduction to proof nets*, 2013,
Theorem 1.1: a connected multiplicative net with hypotheses
sequentializes, a terminal hypothesis being the whole net, otherwise a
terminal `⅋` or a splitting terminal `⊗` existing) is stated for binary
`⅋` and hypotheses; its extension to `⅋` of any arity and to hypotheses
with several conclusions is derived (a `⅋` of arity `k` is a tree of
binary ones up to switching). Per stage: open every `⅋`, every `?`
vertex (its `?` rule, then a copy rule per copy at its depth) and, at a
box's start, every door (copy rules only: the door's `?` rule lies below
the promotion); then Mix by parts, two literals an axiom, a lone `!`
vertex its promotion, else the splitting `⊗` with the smallest id. A
lone `!` vertex has no live wire, since the sources of its wires were
opened in the same branch, so their `?` rules lie below the promotion
and `Θ` holds their subformulas there.

### Departures from the design

- Desequentialization is canonical only up to the links the weakenings
  hang on and, where an occurrence has two vertices, up to the instance
  each rule takes and the `?` vertex a copy behind a door belongs to. So
  two terms that differ only in the order of their rules can give
  different nets, and some nets are the reading of no term at all
  (departs from the design: the plan asks the choice of `?` nodes to
  make desequentialization canonical. The sources say no placement of
  weakening achieves that in general, and a term's naming of occurrences
  rather than instances is the calculus's. The generalised node makes
  the order and the place of contractions canonical).
- `MellStructure` is a type of its own beside `ProofStructure`, not an
  extension of it (departs from the design as `plan/later.md` words it,
  "extend `nets` with `!`-boxes"): a vertex of an MLL structure is its
  occurrence, which the net engine's hot path relies on, and a MELL
  structure needs a vertex per instance.
- `from_parts` and `from_proof` take `&Limits` and charge the graph to
  `limits.memory_bytes`, which `ProofStructure::from_links` does not:
  the doors are wires times depth at worst.

### Options, front ends and quantifiers (D15, D16, D17)

This session adds no output: no options value. The defaults are those of
`Limits`. Quantifiers would enter as two vertex kinds of `Placed`: `∃`
a unary vertex with an unswitched edge, `∀` a unary vertex whose
eigenvariable's occurrences need a box-like region or jumps to it, which
the depth graph's builder takes as switched edges; the propositional
structure pays nothing for them until then.

### What 39b must know

- The drawing reads `boxes()` (each `!` vertex), `inside(v)` (the
  innermost box of a vertex), `auxiliary_doors(d)` (the `?` vertices
  with a door on box `d`'s border), `premises`, `partner` and
  `hangs_on`; a box's rectangle encloses every vertex whose chain of
  `inside` boxes passes through it, with its `!` door and the `?` doors
  on its border. A weakening's link is drawn or not as 39b decides.
- Vertex ids are not occurrence ids in a MELL structure (copies share an
  occurrence), so the drawing's `o<n>` ids, which the web page's buttons
  use, need a per-vertex form for MELL nets.
- The JSON form: `Parts` is the natural content (vertices after the
  roots, links, weakenings); a structure document with boxes is a new
  form, whose level `wire.rs` decides. The error kinds are in JSON
  already.
- `nets::exist` and the command's and the bindings' gates are MLL only;
  `MellStructure::exist` is the MELL one, classical with or without Mix.
- `from_proof` checks the term first (one checker pass), then unfolds a
  shared term as a tree.

### The review

One round each, on the frozen tree `yrywwwwo` (b4361c5a). Each reviewer
worked in a jj workspace of its own on cores 4 to 7, in a capped scope,
with bounded runs, and was told to report only what breaks soundness,
completeness or a stated requirement. The claims briefed were these:
(1) an accepted structure sequentializes into a checked term; (2) a
checked term's net is accepted; (3) the criterion is the switching
condition at every depth; (4) the witnesses are true; (5) no panic, wrap
or unbounded allocation, with `from_parts` as a system boundary; (6) the
round trip is the identity up to the order of copies and the
weakenings' links.

- **The fresh-context reviewer (Opus 5.5 at `high`)** wrote its own
  enumeration of switchings from claim 3 before it read the committed
  one. It ran about 1.6 million structures: 24 hand-made, about 960 000
  from 13 700 proofs of the focused engine and their mutants, 458 000
  random structures over sequents dense in `!` and `?`, and 946 000
  edited `Parts`. It compared the error kinds and the witnesses as well
  as the verdicts. Where the enumeration could decide (222 structures had
  more than 4 096 switchings at some depth), no verdict disagreed and no
  witness was false, and none of the about 640 000 accepted structures
  failed to sequentialize or to check. When it seeded a deliberate fault
  in its own model (door wires left unswitched), 62 disagreements
  appeared, so the agreement is not vacuous. *No finding.*
- **The argument (Fable 5.1 at `high`)** first stated the invariants
  from the code (the depth graphs, the deletion rule, the stage
  invariant that a sub-net is what its conclusions reach, why a lone `!`
  vertex has no live wire, the checker's `Θ`), then checked the argument
  in `core-nets.md` against them. It ran its own brute-force criterion,
  its own isomorphism test and a witness checker on about 120 000
  structures. Claims 1 to 5 held.
  - *F1, confirmed: claim 6 fails.* Copies behind a box's doors all go to
    the innermost `?` of their occurrence (`⊢ ?(~e ⅋ ?a), (!(~a ⊗ ~a) ⊗
    e) ⊗ e`), and crossed links of two copies of `?(a ⅋ ~a)` come back
    parallel. **Answer: holds; the claim was wrong, not the code.** A
    term names occurrences, so it cannot fix those. The struct's and
    `from_proof`'s docs and `core-nets.md` now say so, and `round_trip`
    asserts the isomorphism only where no occurrence has two vertices.
    It still asserts in every case that the net read back is correct
    (`supzqvrl`).
  - *F2, no witness: the written argument for the splitting `⊗` with Mix
    was missing.* **Answer: holds as a gap in the text, not in the
    code.** It is now derived from Laurent's Proposition 1.6 (a
    structure is a net of MLL with Mix iff it is acyclic). Its
    sequentialization's last rule is no Mix, since the two nets would
    lie apart in the plain graph, and no `⅋`, since none is terminal. So
    it is a `⊗` whose two sides meet only at it. This is the code path
    that `ProofStructure::sequentialize` with Mix already takes
    (`core-nets.md`).
- **The counterexamples (Opus 5.5 at `high`)** wrote a second reference
  sharing no code with the module: an exhaustive, memoised and bounded
  search for a bottom-up sequent-calculus reading of a structure. It
  compared that reference with `is_correct` on about 49 000 decided
  structures, drawn from 2 819 proofs of the engine at budgets 10 to 39,
  5 000 random checked terms of its own (shared sub-proofs, `Weaken`,
  several `?` rules of one occurrence in a branch, Mix in the middle),
  their mutants, and random structures, plus 20 000 adversarial `Parts`
  and tight memory bounds. There was no disagreement and no panic, and
  claims 1, 2 and 5 held.
  - *Finding, confirmed: claim 6 fails.* The witnesses: W1, `⊢ ??~a,
    !(a ⊗ a)`, each `?~a` owning one copy inside the box, a net that no
    term reads back to; W2, crossed links over two copies of
    `?(a ⅋ b)`, which give the same term as the straight ones. Nine of
    the 2 819 unmutated nets of engine proofs fail it. All 311 failures
    have an occurrence with two vertices. **Answer: as for F1**, with W1
    recorded in `core-nets.md`.
- **The integers and limits (Sonnet 5.5 at `high`)** went through every
  counter and bound, and drove those it could reach to their limit. A
  shared term of 3·2ᵏ vertices is built at k ≤ 20, refused for memory at
  25 and 29 and refused by index from 30 up to 1 000, the count
  saturating. `MOST` was lowered for an off-by-one test (accepted at the
  bound, refused one past it). 4 million doors are refused within the
  memory bound. Faults at `u32::MAX` were all typed errors, and about
  1.3 million fuzzed cases raised no panic, wrap or hang.
  - *F1, confirmed: the `Disconnected` witness was quadratic in its
    parts and asked no stop* (40 000 parts, 3.4 s). **Answer: holds,
    fixed.** The parts are now indexed by their component's root: 40 000
    parts take 51 ms, 160 000 take 204 ms.
  - *F2, confirmed: a stage was quadratic in its opened conclusions*
    (`gamma.remove`, and Mix's part search): 320 000 weakened roots took
    3.7 s and asked the stop 3 times. **Answer: holds, fixed.** Opened
    conclusions are now marked and dropped once, and Mix's parts are
    sorted by label: 320 000 roots take 0.39 s.
  - *Observations: the memory estimates were 1.5 to 1.85 times below
    the measured peaks.* **Answer: fixed.** `BUILD_BYTES` (the
    construction's transient arrays) is now charged by `from_parts` and
    `from_proof`, and `UNFOLD_BYTES` recounted. The criterion's linear
    scratch stays uncharged, as in MLL. Its conjecture about `wasm32`
    without any memory bound is not pursued: the default bound keeps it
    out of reach.

The scratch modules of the four reviews are kept outside the repository,
in the session's scratch directory (`review-modules/`).

### The net target set

`bench/net-targets.sh after-mell` ran under the machine lock, its runs
pinned to core 2, at `yrywwwwo`. Then `linlog-bench summary --against
bench/targets/net-after-cyclic.csv bench/targets/net-after-mell.csv
--counters` gave: verdicts that differ, none; "Of the 64 problems both
decide alike on one thread, 0 differ in nodes, splits, memo_hits,
memo_entries, links, tests"; none decided by one and not the other. The
file is committed (`lslwwrvs`). The panel's fixes touch only
`nets/mell/`, which the net engine does not use.

### Public items added

These are additions only. `MellStructure`, `Parts` and `Placed` are new
types, and so is `Placement` (a `#[non_exhaustive]` enum). `NetError`
(`#[non_exhaustive]`) gains the variants `Placement`, `Unattached`,
`InBox` and `NotMell`, so no closed enum got a new variant and the break
that step 45 lists does not grow. The sealed `Owner` supertrait gains a
method with a default. No wire form changed level, since there is no
JSON form of the structure yet.

### Commits

- `yrywwwwo` (b4361c5a) Add MELL proof nets with boxes: gated twice.
  The first gate ran on the tree before `nix fmt`; the second, on the
  commit itself, passed (exit 0).
- `lslwwrvs` (75ea9e91) Record the net target set after the MELL nets.
  A results file, no code.
- `supzqvrl` (6aed2402) Answer the MELL panel's findings: gated, passed
  (exit 0).
- The last commit holds this report, the README's and `core-nets.md`'s
  wording on canonicity, and W1. It is prose only; `nix flake check`
  ran at it (below).

### The checklist

| item | state | evidence |
|---|---|---|
| MELL structures with `!`-boxes and `?` nodes over the forest; a criterion independent of any engine, with a witness; tests | done | `nets/mell/mod.rs`, `graph.rs`; `classic_nets`, `agrees_with_enumeration`, `agrees_with_the_multiplicative_criterion`, `an_unfolding_past_the_vertices_is_refused` |
| Sequentialization through boxes, each term checked in the mode; desequentialization; a round-trip test | done | `sequentialize.rs`, `desequentialize.rs`; `round_trips` and `round_trip` (`Proof::check` in the structure's mode, every read-back net correct, isomorphic where no occurrence repeats) |
| The reviewer's comparison with switching enumeration; the panel on a frozen tree, the argument on Fable 5.1 at `high`; every finding answered | done | "The review" |
| The net target set against `net-after-cyclic` | unchanged | "The net target set" |
| Every code commit gated; `nix flake check` at the last commit | done | "Commits"; the flake check's exit status is in the message to `planning` |
| `core-nets.md` amended; README and the changelog true | done | the section "Proof nets with boxes: MELL"; README's "What exists"; the changelog's `[Unreleased]` |
| The `?` node chosen and argued | done | "The `?` node, and why" |

### What is left open

- The drawing, the JSON form, the command and the bindings are 39b's
  ("What 39b must know").
- Intuitionistic MELL nets: left out (`NotMell`). The essential-net
  reading inside every box would be needed.
- A term → net reading that keeps instances. It would need terms that
  name instances, a change of the calculus's representation. It would
  make the round trip exact and reach the nets that no term reads back
  to now.
