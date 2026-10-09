# Exponential proof nets: MELL proof structures with boxes

A specification note for step 33, written 2026-10-08 from the snapshot
of the repository, `33-mell-nets.md`, `34-cut.md` and the sources of
section 10, to check the stage-2 API design against and to implement
from later. Claims about the literature carry a source `[Sn]`, claims
about the code name the file, everything else is marked "(inference)".
linlog's setting: one-sided sequents in negation normal form, the dyadic
proof term of `core/src/proofs/mod.rs`, binary Mix only (no rule
concludes `⊢`: `NetError::Empty`), units included, cut deferred to
step 34.

## 1. Definitions

**Formulas and rules.** MELL formulas are `X, X⊥, A ⊗ B, A ⅋ B, 1, ⊥,
!A, ?A`, with `(!A)⊥ = ?A⊥`. The rules beyond MLL are weakening `?w`,
contraction `?c`, dereliction `?d` and promotion `!` (`⊢ ?Γ, A` gives
`⊢ ?Γ, !A`) [S4 §2.1; S23 §3.1]. linlog's term has no `?c` or `?d`
nodes: `Quest` moves `B` into the unrestricted zone, each `Copy` is a
dereliction of one instance, `Weaken` on a `?` formula is `?w`
(`proofs/mod.rs:139–151`; the checker's arms `check.rs:1242–1292`).

**Proof structure.** A MELL proof structure is a finite directed graph
with the following vertices, each carrying a *formula* (its type), and a
*box nesting* (below).

| kind | premises | conclusion | note |
|---|---|---|---|
| literal `X`, `X⊥` | none | `X` | joined to its dual by an *axiom link* |
| `⊗`, `⅋` | two, ordered | `A ⊗ B`, `A ⅋ B` | as in MLL |
| `1` | none | `1` | an axiom of its own |
| `⊥` | none | `⊥` | carries a *jump* (below) |
| `!` (principal door) | one, `A`, inside the box | `!A`, outside | one per box |
| `?` (collector) | `n ≥ 0`, each `B` or a door of `?B` | `?B` | `n = 0` weakening, `n = 1` dereliction, `n ≥ 2` contraction |
| door (auxiliary door) | `n ≥ 1`, each `B` or a door of `?B`, inside the box | `?B`, leaving the box as one premise of the enclosing collector | one per (`?B` instance, box) |
| `cut` | two, `A` and `A⊥` | none | step 34's; the kind must exist in the model now |

The `?d`/`?c`/`?w`/`?p` syntax of Girard [S1] and Laurent [S4 §2.2]
and the generalized one are inter-translatable: Laurent replaces every
maximal `?`-tree by one `?` node with `n ≥ 0` premises labelled `A` and
chains of `p` nodes above it, "to make more canonical the representation
of `?`-trees", translates back with "left comb trees", and Lemma 2.9
says a structure with `?` nodes is acyclic iff its translation is
[S4 §2.5]. Girard's `?`-link has `n` unordered premises, all occurrences
of one discharged formula `[A]`, "the case `n = 0` is allowed, and
accounts for weakening", and `[A]` is "the conclusion of (unspecified)
generalized axioms (i.e. boxes)" [S3 §3 p. 22–23]. linlog's collector is
Laurent's `?` node with his `p` chains bundled: one door per (`?B`
instance, box) collects every leaf inside that box where he puts one `p`
node per leaf. The two are in bijection (inference; section 2 argues the
switching equivalence).

**Boxes and nesting.** A box `B` is its `!` vertex together with its
*content*, a set of vertices; the box's doors are its `!` vertex and its
door vertices. Boxes "never partially overlap" [S1; S3 §3]: two boxes
are disjoint or nested. Every vertex `v` has `box(v)`, the innermost box
containing it, or none; by convention the `!` vertex of `B` is *outside*
`B` (`box(!_B) = parent(B)`) and `B`'s door vertices and the premise of
`!_B` are *inside* (`box = B`). The *depth* of `v` is the number of
boxes containing it [S1]. The boxes form a tree under nesting [S19].

**Edges.** Tree edges (premise to conclusion of `⊗`, `⅋`, `!`), axiom
links between two dual literals *of the same box*, collector premise
edges (from a `B` vertex at the collector's depth, or from a door of a
box nested directly in the collector's box), jumps, and cut links. A
collector edge from a door `d` of box `B'` to a collector `c` is, in the
graph of `c`'s depth, an edge between `!_{B'}` and `c`: the box is one
vertex there and its doors are its ports (section 2).

**Instances.** The vertices are *instances*, not occurrences: a `Copy`
repeats an occurrence and its whole subtree in one branch
(`proofs/mod.rs:144–147`), so a net has one vertex per rule instance
plus the doors. In MLL every occurrence has exactly one instance, which
is why `ProofStructure` can index by `OccId` today (`nets/mod.rs:182`).

**Jumps.** Every `⊥` vertex and every collector with `n = 0` carries a
jump to a vertex at its depth. Girard: "⊥-links are treated like 0-ary
?-links, i.e. they must be given a default jump", a switching "will
select a jump for each ?-link L: this jump may be the default one or
any premise" [S3 §3, §A.2]. For `n ≥ 1` a default jump equal to a
premise makes the extra choice vacuous, so linlog keeps jumps for
`n = 0` and `⊥` only (inference). Girard's own verdict: "the only
solution consists in declaring that the jumps are not part of the
proof-net, but rather of some control structure", so nets are
"equivalence classes" [S3 §A.2]. A jump's target is a vertex of the same
depth other than the jumping vertex and its parent, and two jumps never
join the same pair: each excluded case is a pair of parallel edges,
which some switching keeps as a cycle, so no verdict changes and the
depth graphs stay simple, as `graph.rs`'s search assumes (inference).

**Units.** `1` is a vertex with no premise and needs nothing further.
`⊥` is the zero-premise case of weakening and needs the jump. Without
jumps a structure over `1` and `⊥` alone is the formula itself, and
correctness would decide constant-only MLL, NP-complete [S3 §A.2; S14];
Girard's footnote 5 offers only "the necessary condition acyclic graph
with n + 1 connected components, where n is the number of ⊥-links"
[S3 fn. 5]. The alternatives that make units canonical (a second formula
tree of thinning links [S24], Hughes's nets with units [S4 §4.4]) are
equivalence classes of graphs too [S23 §3.5]; none is cheaper for a
checker, so the step keeps jumps.

## 2. Correctness

**Switchings and the criterion.** For a box `B` (or the outside, depth
0) the *depth graph* `G(B)` has as vertices every `v` with `box(v) = B`,
where a nested box `B'` appears only as its `!` vertex `!_{B'}` with the
door edges of `B'` attached to it; the doors of `B` itself are vertices
of `G(B)`. Its edges are the tree edges, links, collector edges, jumps
and (34) cut links among those vertices. The *conclusions* of `G(B)`
are the premise of `!_B` and the doors of `B`; those of `G(outside)` are
the roots. A *switching* keeps, for every `⅋` and every collector or
door with `n ≥ 2` premises, exactly one premise edge, and every other
edge. The structure is *correct* iff for every depth graph every
switching is acyclic and, without Mix, connected; with Mix, acyclic and
non-empty. Sources: each box "treated separately" and `?c` behaving as
`⅋` [S23 §3.1]; boxes as generalized axioms [S3 §3], Laurent's `hyp`
nodes with "at least one conclusion (and no premise)" [S4 §1.4.3]; the
criterion itself [S2]; with binary Mix acyclic plus "at least one
connected component", with `void` acyclicity alone [S4 Prop. 1.5, 1.6].
A box content is a proof of `⊢ ?Γ, A` (`check.rs:1242`), so it is
connected without Mix and non-empty with it (inference).

**Weakening and connectedness (the jump question).** Without jumps a
weakened `?B` or a `⊥` is an isolated vertex of every switching, so it
breaks connectedness and nothing distinguishes `⊢ ?X, Y, Y⊥` (provable)
from `⊢ ?X, ?Z` (unprovable, since no rule concludes `⊢`): with binary
Mix both are acyclic and non-empty. Laurent avoids this by taking the
nullary Mix `void`, under which "the weakening rule is equivalent to the
more traditional one" and only acyclicity remains [S4 §1.6, §2.1]; that
changes linlog's calculus, checker and Rocq export and is the author's
to decide (`33-mell-nets.md` §2). With jumps as unswitched edges, `⊢ ?X,
?Z` forces the two weakenings to jump to each other, a parallel pair,
hence incorrect; `⊢ ?X, ?Z, 1` jumps both to `1` and is correct
(inference). Soundness of the jump criterion for terms is by
construction (`from_proof` chooses the jump, section 7); completeness is
Girard's "sequentialisation is an easy exercise" / "immediate" for his
calculus [S3 §3, §A.2], not a theorem stated for linlog's calculus with
binary Mix, which is why `33-mell-nets.md` asks for a brute-force
comparison against the sequent calculus on every small structure before
the panel. Canonicity holds only modulo jumps [S3 §A.2], and equivalence
of normal forms in MELL without units is already PSPACE-hard because
"the weakening rule for the exponentials induces a similar rewiring
problem" [S13].

**Why collectors are switched.** `⊢ ?X⊥, X ⊗ X` with two copies has the
plain cycle `?–y₁–x₁–⊗–x₂–y₂–?`; every switching keeps one premise of
the collector and breaks it (section 8, example 3). A door with `n ≥ 2`
premises is switched for the same reason. Laurent's form is equivalent:
his `n` leaf chains of one `?` node through one box are `n` parallel
switched edges to the box node outside and `n` pendant `p` nodes
inside; one door edge and one switched door vertex keep the same
switching graphs up to pendant vertices, which lie on no cycle
(inference).

**Algorithms and costs.**

| test | MLL | with boxes |
|---|---|---|
| switching enumeration | exponential; the oracle of `nets::graph`'s tests | per depth graph |
| Danos's contractibility [S6], made linear through union-find [S7; S8] | linear | Guerrini–Masini's parsing for "full (weakening and constants included)" MELL, confluent and strongly normalizing, equivalent to Danos–Regnier [S5]; cost not in the abstract |
| Murawski–Ong via IMLL [S9] | linear | — |
| de Naurois–Mogbil dependency criterion [S10; S11] | NL-complete | the CSL 2007 title covers exponential structures [S10] |
| Yeo deletion, linlog's test (`graph.rs`; [S12]) | linear per round, rounds = nesting of cycles through both premises of a `⅋` | the same on the disjoint union of the depth graphs (inference) |

The extension of `graph.rs`'s colouring: the premise edges of a `⅋`
share one colour, the premise edges of a collector or door with `n ≥ 2`
share one colour of their own, every other edge (tree edge of a `⊗` or
`!`, link, jump, door edge seen from the box vertex, cut) has its own.
A cycle survives some switching iff it is properly coloured, so Yeo's
deletion condition stays "every non-premise edge is a bridge", and a
door inside its box, having premise edges only, is deletable at once,
like a root `⅋` (inference from the argument in `core-nets.md`). Because
every vertex lies in exactly one depth graph and each box adds one
vertex to its parent's graph, the sum of the depth graphs' sizes is the
structure's size plus the number of boxes, so one deletion run over the
disjoint union costs what MLL costs (inference). Connectedness is a
count per depth graph: a switching keeps `2t + p + c₊ + k + j` edges
(`t` tensors, `p` pars, `c₊` collectors and doors with `n ≥ 1`, `k`
links, `j` jumps), a tree iff that is `V − 1` with each nested box
counted once (inference, as `core-nets.md`'s equation). Witnesses are
per depth graph: the cycle with its vertices, or the parts, each error
naming the box.

## 3. Sequentialization with boxes

The algorithm extends `sequentialize.rs`'s stages to a depth graph with
box vertices, collectors, weakenings and jumps; it follows Laurent's
proof with `hyp` nodes, where a terminal `hyp` node alone is
sequentialized by its own proof and otherwise a descent path reaches a
terminal `⅋` or a splitting `⊗` [S4 Thm. 1.1], with the adaptations
marked (inference). `Γ` is the list of conclusions of the sub-net, each
a vertex; `θ` says whether a `Quest` is pending for a collector.

```
sequentialize(G, Γ):                      -- G a depth graph, Γ ⊆ its vertices
  steps := [Prove(Γ)]; proved := []
  while steps not empty:
    pop step; match step:
      Prove(Γ):
        -- 1. removable conclusions: a ⊥ or a zero-premise collector is a
        --    pendant vertex (its jump) of every switching, so the rest is a net
        for w in Γ with kind ⊥ or collector n = 0:
          Γ -= w; delete w and its jump; push Emit(Bot(w) | Weaken(w))
        -- 2. open every ⅋, every collector and every door conclusion
        for v in Γ with kind ⅋: Γ := Γ − v + {left(v), right(v)}; delete v; push Emit(Par(v))
        for c in Γ with kind collector, n ≥ 1:
          Γ -= c; delete c
          for each dereliction premise u of c: Γ += u; push Emit(Copy(u))
          -- a door premise is a port of a nested box: it disappears with c
          if c is a ? vertex: push Emit(Quest(c))     -- a door of G's own box emits nothing
        -- 3. parts (Mix) over the plain graph: tree edges, links, collector
        --    edges, jumps, with every box one vertex
        parts := search(G, Γ)
        if parts > 1: assert mix; push Prove(part_i) for each, Mix between them
        else if Γ = {x, y} two linked literals: push Emit(Ax(x, y))
        else if Γ = {u} with kind 1: push Emit(One(u))
        else if Γ = ports of one box B (its !A and its remaining doors):
          push Emit(Bang(!_B)); push Prove_in(G(B), {premise(!_B)} ∪ doors(B))
        else:
          t := the ⊗ in Γ with the smallest id whose premise edge is a bridge
               of the plain graph              -- exists: splitting lemma (inference)
          delete t; split Γ − t by reachability from left(t)
          push Emit(Tensor(t)); push Prove(right side + right(t)); push Prove(left side + left(t))
      Emit(node): take the proved premises it needs from `proved`, push the node
```

`Prove_in(G(B), …)` is the same procedure on the box's depth graph, on
the same explicit stack, so a derivation of any height costs the
caller's stack nothing (`a_high_derivation_needs_no_stack`). The dyadic
reading of step 2: opening a `?` vertex emits `Quest` below and one
`Copy` per dereliction premise directly above it, since each premise is
then a conclusion of the remaining sub-net and `Copy` only adds `B` to
the linear zone bottom-up (`check.rs:1266`); a door emits nothing
because `Θ` holds `B` across the promotion (`check.rs:1242`). Jumps stay
in the plain graph while their vertex is a premise: a `⊥` under a `⊗`
with a jump into `Δ` makes that `⊗` splitting exactly when the proof
from `⊢ Δ` (then `⊥`) and `⊢ A, Γ` exists (inference). The splitting
lemma with box vertices is Laurent's with `hyp` nodes [S4 Thm. 1.1];
with jumps and collectors it is the claim the panel must examine, and
the oracle of section 2 covers it. Cost: one search per stage over the
depth graph, at most the square of the structure's size overall
(inference; `core-nets.md`); Guerrini's linear sequentialization [S8]
would apply per depth graph if ever needed (inference).

## 4. Representations in implementations

Four shapes are in use.

**A tree of boxes with explicit membership.** Each vertex carries its
innermost box, each box its parent, principal door and doors. Guerrieri,
Manara, Pellissier, Tortora de Falco and Vaux Auclair make this a graph
morphism from the structure's underlying graph to the tree of boxes,
replacing "auxiliary door nodes, a map from boxed nodes to their main
door, and ad hoc nesting conditions" [S19]; Laurent's "numbered proof
nets" attach a label to each box [S4 §2.4]. The depth graphs are derived
by grouping vertices on their box.

**Depth labels.** Each vertex carries only its depth, as in the geometry
of interaction and in sharing graphs, where "every operator is decorated
with an integer tag giving its level" and brackets and croissants manage
the levels [S20; S18]. A depth alone cannot tell two sibling boxes
apart, so a checker that must build box contents needs the box id anyway
(inference).

**Flat structures with jumps or implicit boxes.** Polarized nets allow
structural rules on any negative formula [S16]; Accattoli shows that a
polarized MELL "allows to internalize boxes, so that they are induced by
the proof itself, instead of being explicitly added to the graph" [S17];
jumping boxes [S21] and Di Giamberardino's cones, "a looser structure
defined by jumps" that may overlap [S22], replace the box by sequential
constraints; sharing graphs remove boxes for optimal reduction [S18;
S20]. All need polarity or an equivalence class to recover what the box
states explicitly.

**Wire forms.** Di Guardia and Laurent formalize MLL nets in Rocq as a
total directed multigraph: vertices labelled by rule (`ax`, `cut`, `⊗`,
`⅋`, `c` for conclusions), edges by formula and a Boolean for the left
premise, conclusion vertices ordered to recover the sequent; they judge
"extensions to exponential or additive proof-nets seem doable" [S15].
HVM2 writes an interaction net as trees whose nodes are the agents:
"aux-to-main wires are implicit through the tree-like structure of the
syntax, while aux-to-aux wires are explicit through variable nodes,
which are always paired" [S25]. linlog's current form is the forest plus
links by occurrence id (`serialize/nets.rs`).

Comparison for linlog's four uses:

| use | tree of boxes | depth labels | flat with jumps | wire form |
|---|---|---|---|---|
| checker | depth graphs by one grouping pass; per-box witnesses | needs box ids anyway | needs polarity | the input to build from |
| search | MLL only, by occurrence (`search/net.rs`); section 6 | — | — | — |
| JSON | a `box` per vertex keeps the MLL keys; nested arrays would not | lossy | — | links over vertex ids, jumps, boxes, version |
| drawing | a box is its content's layout in a rectangle, one unit outside | — | — | ids per vertex, box, jump |

Recommendation: the tree of boxes with explicit membership, stored as a
`box` field per vertex plus a box table; MLL keeps vertex = occurrence
(inference; `33-mell-nets.md` §3).

## 5. Box-free and other alternatives, and their limits

Polarized nets [S16; S17], jumping boxes [S21], cones [S22], sharing
nets [S18; S20] and differential nets, where promotion exists only
through the Taylor expansion into promotion-free nets [S26], each buy
locality of reduction at the price of a polarity restriction, an
equivalence class, or the loss of promotion as a single node. Acclavio's
exponentially handsome nets cover MELL with units through cographs,
cut-free [S27]. Intuitionistic combinatorial proofs split a proof into
"a linear part, which resembles an IMLL proof net, and a
contraction-weakening part", with translations polynomial in size
[S28]. Multi-focusing takes the opposite route and keeps the sequent
calculus: maximally multi-focused proofs are canonical and correspond
one-to-one to unit-free MLL nets, while for "linear logic with units and
exponentials" the authors note that "the nature of proof nets is less
well developed or satisfying" [S29]. None of these draws what a course
draws, a box around a sub-proof, which `plan/33-mell-nets.md` fixes as
the representation.

## 6. Proof search with MELL nets, and the focused engine

In MLL a cut-free proof is its linking, so the net engine searches
linkings over a fixed structure (`search/net.rs`; `core-nets.md`). In
MELL the structure is not fixed by the sequent: how many instances each
`?B` gets and which boxes each instance crosses are the proof's choices,
so a net search would enumerate instance counts under a bound and
linkings over the resulting structures, which is the focused engine's
copy bound in another dress (inference; `33-mell-nets.md` §1). The
literature on constructing nets by search is focused search read as
graph construction: Andreoli's "bipolar decomposition of formulas"
maps into "a sequent system using only atoms" [S30], his proof-net
construction is "the computational paradigm based on proof-construction
in terms of proof-nets" [S31], extended to concurrent construction with
Mazaré [S32], and Maieli's bipolar nets to MALL [S33]; Chaudhuri, Miller
and Saurin's correspondence shows that a maximally multi-focused proof
*is* a construction order of the net [S29]. Grail and Moot's thesis use
nets for categorial grammars, MILL and its first-order extension, not
MELL [S34; S35]. Conclusion: the focused engine's proof desequentializes
to the net (section 7), and search stays with the focused engine as the
prompt fixes; the only use of the criterion in search would be the
acyclicity test on a partial structure as a pruning of the dyadic
search's axiom choices, a follow-up for step 35 if a measurement asks
(inference).

## 7. Consequences for linlog's data model

**What exists.** `ProofStructure { forest, mix, partner, links, graph,
skeleton }` indexed by occurrence; `new` refuses anything outside
unit-free MLL; `link`/`unlink` as a stack; `from_proof` reads `Ax` nodes
and ignores every other node; `is_correct` is Yeo deletion then the edge
count; `sequentialize` emits `Ax`, `Tensor`, `Par`, `Mix`; `NetError`
has eight variants, exhaustive; `Display` prints `~A[0] — A[2]` by
occurrence; JSON is `{"sequent", "mix", "links"}` without a version;
`svg::net` lays out literals in occurrence order with ids `o<n>` and
`l<m>-<n>` and `Style` has no box field (`export/svg/mod.rs:61`);
`Outcome` carries `net: Option<ProofStructure>`; the CLI's `nets_exist`
admits MLL only.

**Desequentialization** (`from_proof`), one walk of the term with a
stack, carrying the current box and a map from occurrence to its active
instance: `Tensor`/`Par`/`Bang` create their children's vertices;
`Bang(o, p)` opens a box with principal `vertex(o)` and walks `p`
inside; `Quest(o, p)` creates the collector of `o`; `Copy(a, p)` creates
a vertex for `a` at the current box and attaches it to the collector of
`parent(a)`'s instance through one door per box between, created on
first use; `Weaken(o)` on a `?` formula is a collector with `n = 0`,
`Bot(o)` a `⊥`, both jumping to the principal vertex of the premise's
rule (through `Mix` to its left premise); `Ax` a link, refused across
boxes; `With`, `Plus`, `Top`, affine `Weaken` and any later node are
refused by an exhaustive match. Cost:
linear in the term plus one door per crossed box per copy (inference).
A `Quest` never copied and a `Weaken` give the same vertex, so the net
is canonical for the term modulo rule permutations, `?c` associativity
and commutativity, contractions moved across a box border, and jumps
(inference; the first two are Laurent's purpose in [S4 §2.5]); it is
coarser than Girard's nets, which tell two doors from one (inference).

**What the model must offer.**

- A vertex table `Vertex { occ: OccId, parent: Option<VertexId>, kind,
  box: Option<BoxId> }`, with `VertexId` the net's own (`u32`), so that
  step 34's box duplication and step 38's quantifier nodes fit; for MLL
  the identity `vertex = occurrence` keeps the net engine's hot path
  untouched (`33-mell-nets.md` §3; D17). Vertex ids are a *numbering*
  the structure assigns, not a canonical form (open question 1).
- Edge kinds as data: `Link::Axiom(v, w)`, `Link::Cut(v, w)` (34),
  `Jump(v, w)`, collector premises with variable arity, door edges. The
  CSR `Graph` is built once per MELL structure; incremental
  `link`/`unlink` stays an MLL operation.
- A box table `Box { principal: VertexId, parent: Option<BoxId>, doors:
  Vec<VertexId> }` and the depth graphs as ranges over vertices grouped
  by box.
- `NetError` variants: `LinkAcrossBoxes`, `JumpAcrossBoxes`,
  `JumpToNeighbour` (the parallel-pair rule), `MissingJump`,
  `DoorWithoutPremise`, and `SwitchingCycle`/`Disconnected` carrying the
  box they are in; `Empty` only for depth 0.
- `ProofStructure::new(forest, mix)` becomes `new(forest, &Criterion)`
  or similar, the one value that `mix`, 35's `Criterion`, 36's `cyclic`
  and this step extend (README §3 conflict 4).
- A bound: building the table charges `memory::Account`, with a
  `within` constructor (README §2, "a bound on every entry").
- JSON: the MLL form unchanged when the structure has no exponential;
  otherwise `"version"`, `"vertices": [[occ, parent, box], …]`,
  `"boxes": [[principal, parent], …]`, `"links"` over vertex ids,
  `"jumps": [[from, to], …]`, every new key under `serde(default)`
  (D18; README §2 "wire forms").
- Text form: `Display` keeps `A[n]` by occurrence where the instance is
  unique and writes `A[n#k]` for the `k`-th instance otherwise, so
  README's MLL blocks stay byte-identical (inference).
- Drawing: a box content is laid out by the present algorithm inside a
  rectangle, placed in the enclosing layout as one unit whose ports,
  `!A` and the doors, sit on its bottom edge; jumps are dashed arrows,
  hidden by default; `Style` gains `box_padding`, `box_stroke`,
  `box_radius`, `door_radius`, `box` and `jump` colours and `jumps:
  bool` (D12, D15); `estimate` counts instances and boxes; ids `o<n>` by
  vertex (the occurrence id in MLL), `b<n>` per box, `j<n>` per jump,
  `l<m>-<n>` by vertex ids (`plan/reports/11-svg.md`; the web client's
  click targets, `32-web.md`).
- `Outcome::net` and the CLI's `net_into` take the one type;
  `nets_exist` admits MELL and still refuses affine mode and additives.

**What the API design must leave open now.** The vertex id as a type
distinct from `OccId` in every public signature of `nets` (`partner`,
`links`, `unlinked`, the JSON, the SVG ids); the edge-kind enum and
`NetError` marked `#[non_exhaustive]`; `Style` and the net's options
value `#[non_exhaustive]`; `"version"` on the net's JSON; `from_proof`
exhaustive over `Node`; no new `Node` variant is needed for MELL nets,
since `Bang`, `Quest`, `Copy`, `Weaken`, `Bot`, `One`, `Mix` suffice
(`proofs/mod.rs`), but `Proof` must expose a walk with premises in
order, which it does.

## 8. A notation for small MELL structures

The test set writes a structure as lines. Names are identifiers of the
writer's choice; the first line lists the conclusions in the written
order of the sequent, so occurrence ids follow from the formula trees.

```
<structure> ::= "⊢" <name> ("," <name>)* <line>*
<line>      ::= <name> ":" <formula> ["=" <build>] ["in" <box>]
              | <name> "—" <name>                 -- axiom link
              | <name> "->" <name>                -- jump
<build>     ::= "(" <name> "," <name> ")"        -- ⊗ or ⅋ premises, left first
              | "box" <box> "(" <name> ")"        -- ! with its box and premise
              | "?(" [<name> ("," <name>)*] ")"   -- collector premises, n ≥ 0
              | "door" <box> "(" <name> ("," <name>)* ")"
```

A literal, `1` or `⊥` has no `<build>`. A vertex without `in` is at
depth 0; a door is inside the box it names. A parser derives each
vertex's occurrence from its position under a conclusion and checks the
formula against the forest; two vertices at the same position are two
instances.

**Example 1, correct: `⊢ ?X⊥, !X`** (a dereliction inside a box;
the term `Quest(Bang(Copy(Ax)))`).

```
⊢ q, b
q: ?X⊥ = ?(d)
b: !X = box B (x)
x: X in B
y: X⊥ in B
d: ?X⊥ = door B (y)
x — y
```

Depth 0: vertices `q`, `b`; one door edge `q–b`: a tree. Box `B`:
vertices `x`, `y`, `d`; edges `x–y`, `d–y`; conclusions `x` and `d`: a
tree. Correct. Sequentialization: depth 0 opens `q` (`Quest`), the rest
is the box alone (`Bang`), inside it the door opens (`Copy(y)`) and
`{x, y}` is an axiom.

**Example 2, incorrect: `⊢ !X ⊗ ?X⊥`** (a cycle through a door).

```
⊢ t
t: !X ⊗ ?X⊥ = (b, q)
b: !X = box B (x)
q: ?X⊥ = ?(d)
x: X in B
y: X⊥ in B
d: ?X⊥ = door B (y)
x — y
```

Depth 0: `t–b`, `t–q`, `q–b`, no switch (`q` has one premise):
`SwitchingCycle` in the outside, through `t`, `b`, `q`. The sequent is
unprovable: the `⊗` would need `⊢ !X` and `⊢ ?X⊥` apart.

**Example 3, correct: `⊢ ?X⊥, X ⊗ X`** (contraction as a two-premise
collector).

```
⊢ q, t
q: ?X⊥ = ?(y1, y2)
t: X ⊗ X = (x1, x2)
x1: X
x2: X
y1: X⊥
y2: X⊥
x1 — y1
x2 — y2
```

The plain graph has the cycle `q–y1–x1–t–x2–y2–q`. Each switching keeps
one of `q–y1`, `q–y2`: five edges on six vertices, connected: a tree.
Correct. With `q` unswitched the structure would be rejected, which is
why `n ≥ 2` collectors switch.

**Example 4, correct with a jump, and its incorrect variants:
`⊢ ?Y, X⊥, X`.**

```
⊢ w, a, b
w: ?Y = ?()
a: X⊥
b: X
a — b
w -> b
```

Edges `a–b`, `w–b`: a tree, correct without Mix. Without the jump line
the structure fails validation (`MissingJump`); `⊢ ?Y, ?Z` alone can
only jump `w1 -> w2` and `w2 -> w1`, a parallel pair, refused
(`JumpToNeighbour`), and the sequent is unprovable with binary Mix.
Further cases for the set: example 1 with `y` outside `B`
(`LinkAcrossBoxes`); `⊢ ?X⊥, !!X` with the door chain `q = ?(d1)`,
`d1 = door B1 (d2)`, `d2 = door B2 (y)`, correct at all three depths;
`⊢ 1, 1` correct with Mix only; `⊢ ⊥ ⊗ ⊥, 1, 1` correct with jumps
`⊥₁ -> 1₁`, `⊥₂ -> 1₂` and incorrect when both jump to `1₁`.

## 9. Open questions

1. **Equality of nets.** Two desequentializations of equivalent proofs
   agree only up to a renaming of instances, so equality is isomorphism
   over the forest (occurrence, kind, premise order, box, links; jumps
   ignored), not `links == links` as in MLL. A canonical numbering, or
   an isomorphism test exploiting that every vertex is typed by its
   occurrence, is needed for the round-trip tests; small cases can
   brute-force it (inference).
2. **Jumps versus `void`.** The nullary Mix would make weakening free
   and the criterion acyclicity alone [S4], at the cost of the calculus,
   the checker and the Rocq kernels; the author decides
   (`33-mell-nets.md` §4).
3. **Exactness of the jump criterion** for linlog's calculus with binary
   Mix: argued here (inference), stated by Girard for his calculus
   [S3], to be established by the oracle test before the panel.
4. **Guerrini–Masini's formulation** of weakening and constants [S5]
   should be read before the jump choice is final; the journal text was
   not reachable from here.
5. **Collector premise order.** Unordered in the theory; the table, the
   drawing and the text form need a deterministic order, by vertex id.
6. **Where cut edges (34) and quantifier nodes (38) go**: cut as a link
   kind at one depth, duplicating boxes through the box table; `∀`/`∃`
   as unary vertices of the same table (D17); for the step's report.

## 10. Sources

- [S1] J.-Y. Girard, "Linear logic", Theoretical Computer Science 50
  (1987) 1–101. https://doi.org/10.1016/0304-3975(87)90045-4
- [S2] V. Danos, L. Regnier, "The structure of multiplicatives", Archive
  for Mathematical Logic 28 (1989) 181–203.
  https://doi.org/10.1007/BF01622878
- [S3] J.-Y. Girard, "Proof-nets: the parallel syntax for proof-theory",
  in Logic and Algebra, Lecture Notes in Pure and Applied Mathematics
  180, Marcel Dekker, 1996, 97–124; author's PDF (pages cited as
  printed there). https://girard.perso.math.cnrs.fr/Proofnets.pdf
- [S4] O. Laurent, "An Introduction to Proof Nets", lecture notes,
  18 October 2013. https://perso.ens-lyon.fr/olivier.laurent/pn.pdf
- [S5] S. Guerrini, A. Masini, "Parsing MELL proof nets", Theoretical
  Computer Science 254 (2001) 317–335,
  https://doi.org/10.1016/S0304-3975(99)00299-6; report IRCS-96-37,
  University of Pennsylvania, October 1996 (abstract read),
  https://repository.upenn.edu/ircs_reports/115
- [S6] V. Danos, La Logique Linéaire appliquée à l'étude de divers
  processus de normalisation (principalement du λ-calcul), thèse de
  doctorat, Université Paris VII, 1990 (cited through [S4], [S7],
  [S11]).
- [S7] S. Guerrini, "Correctness of multiplicative proof nets is
  linear", LICS 1999, 454–463.
  https://lics.siglog.org/1999/Guerrini-CorrectnessofMultip.html
- [S8] S. Guerrini, "A linear algorithm for MLL proof net correctness
  and sequentialization", Theoretical Computer Science 412(20) (2011)
  1958–1978. https://doi.org/10.1016/j.tcs.2010.12.021
- [S9] A. S. Murawski, C.-H. L. Ong, "Fast verification of MLL proof
  nets via IMLL", ACM Transactions on Computational Logic 7 (2006)
  473–498. https://doi.org/10.1145/1149114.1149116
- [S10] P. Jacobé de Naurois, V. Mogbil, "Correctness of multiplicative
  (and exponential) proof structures is NL-complete", CSL 2007, LNCS
  4646, 435–450; journal version "Correctness of linear logic proof
  structures is NL-complete", Theoretical Computer Science 412(20)
  (2011) 1941–1957. https://doi.org/10.1016/j.tcs.2010.12.020
- [S11] M. Bagnol, A. Doumane, A. Saurin, "On the dependencies of
  logical rules", FoSSaCS 2015, LNCS 9034; author's PDF.
  https://www.normalesup.org/~bagnol/articles/conferences/dependency.pdf
- [S12] R. Di Guardia, O. Laurent, L. Tortora de Falco, L. Vaux Auclair,
  "Yeo's theorem for locally colored graphs: the path to
  sequentialization in linear logic", FSCD 2025, LIPIcs 337, 16:1–16:18.
  https://doi.org/10.4230/LIPIcs.FSCD.2025.16 ; A. Yeo, "A note on
  alternating cycles in edge-coloured graphs", Journal of Combinatorial
  Theory B 69 (1997) 222–225.
- [S13] W. Heijltjes, R. Houston, "Proof equivalence in MLL is
  PSPACE-complete", Logical Methods in Computer Science 12(1:2) (2016).
  https://doi.org/10.2168/LMCS-12(1:2)2016
- [S14] P. Lincoln, T. Winkler, "Constant-only multiplicative linear
  logic is NP-complete", Theoretical Computer Science 135 (1994)
  155–169.
- [S15] R. Di Guardia, O. Laurent, "A Formalization of Multiplicative
  Proof-Nets in Rocq", TLLA 2025.
  https://perso.ens-lyon.fr/olivier.laurent/rocqpn.pdf ; code
  https://github.com/RemiDiG/proofnet_mll
- [S16] O. Laurent, "Polarized proof-nets and λμ-calculus", Theoretical
  Computer Science 290(1) (2003) 161–188.
  https://hal.archives-ouvertes.fr/hal-00009114
- [S17] B. Accattoli, "Compressing polarized boxes", LICS 2013, 428–437,
  https://lics.siglog.org/archive/2013/Accattoli-CompressingPolarize.html ;
  talk abstract "Toward a new theory of exponential proof-nets",
  Chocola seminar, 14 November 2013,
  https://chocola.ens-lyon.fr/events/seminaire-2013-11-14/talks/accattoli
- [S18] G. Gonthier, M. Abadi, J.-J. Lévy, "Linear logic without
  boxes", LICS 1992, 223–234. https://doi.org/10.1109/LICS.1992.185535
  ; S. Guerrini, S. Martini, A. Masini, "Coherence for sharing
  proof-nets", Theoretical Computer Science 294 (2003) 379–409.
  https://doi.org/10.1016/S0304-3975(01)00162-1
- [S19] G. Guerrieri, G. Manara, L. Pellissier, L. Tortora de Falco,
  L. Vaux Auclair, "MELL proof-nets in the category of graphs", TLLA
  2021. https://hal-lirmm.ccsd.cnrs.fr/lirmm-03271478
- [S20] A. Asperti, S. Guerrini, The Optimal Implementation of
  Functional Programming Languages, Cambridge Tracts in Theoretical
  Computer Science 45, Cambridge University Press, 1998.
- [S21] B. Accattoli, S. Guerrini, "Jumping boxes", CSL 2009, LNCS
  5771, 55–70. https://doi.org/10.1007/978-3-642-04027-6_7
- [S22] P. Di Giamberardino, "Jump from parallel to sequential proofs:
  exponentials", listed as in press by the Cagliari repository.
  https://iris.unica.it/handle/11584/74873
- [S23] L. Straßburger, "Proof nets and the identity of proofs", lecture
  notes, ESSLLI 2006, arXiv cs/0610123. https://arxiv.org/abs/cs/0610123
- [S24] L. Straßburger, F. Lamarche, "On proof nets for multiplicative
  linear logic with units", CSL 2004, LNCS 3210, 145–159; journal
  version F. Lamarche, L. Straßburger, "From proof nets to the free
  *-autonomous category", Logical Methods in Computer Science 2(4:3)
  (2006). https://doi.org/10.2168/LMCS-2(4:3)2006
- [S25] V. Taelin, "HVM2: a parallel evaluator for interaction
  combinators", HigherOrderCO, paper in the repository.
  https://github.com/HigherOrderCO/HVM/blob/main/paper/HVM2.pdf
- [S26] T. Ehrhard, L. Regnier, "Differential interaction nets",
  Theoretical Computer Science 364 (2006) 166–195.
  https://doi.org/10.1016/j.tcs.2006.08.003
- [S27] M. Acclavio, "Exponentially handsome proof nets and their
  normalization", EPTCS 353 (2021) 1–25.
  https://arxiv.org/abs/2112.14962
- [S28] W. Heijltjes, D. J. D. Hughes, L. Straßburger, "Intuitionistic
  proofs without syntax", LICS 2019.
  https://doi.org/10.1109/LICS.2019.8785827
- [S29] K. Chaudhuri, D. Miller, A. Saurin, "Canonical sequent proofs
  via multi-focusing", IFIP TCS 2008, 383–396.
  https://doi.org/10.1007/978-0-387-09680-3_26 ; author's PDF
  https://www.lix.polytechnique.fr/~dale/papers/tcs08trackb.pdf
- [S30] J.-M. Andreoli, "Focussing and proof construction", Annals of
  Pure and Applied Logic 107(1) (2001) 131–163.
  https://europe.naverlabs.com/research/publications/focussing-and-proof-construction/
- [S31] J.-M. Andreoli, "Focussing proof-net construction as a
  middleware paradigm", CADE-18, LNCS 2392, 2002, 501–516.
  https://doi.org/10.1007/3-540-45620-1_39
- [S32] J.-M. Andreoli, L. Mazaré, "Concurrent construction of
  proof-nets", CSL 2003.
  https://europe.naverlabs.com/research/publications/concurrent-construction-of-proof-nets/
- [S33] R. Maieli, "Bipolar proof nets for MALL", arXiv 1210.5946,
  2012. https://arxiv.org/abs/1210.5946
- [S34] R. Moot, Proof Nets for Linguistic Analysis, PhD thesis,
  Utrecht University, 2002.
  https://www.labri.fr/perso/moot/moot02proofnets.pdf
- [S35] R. Moot, "The Grail theorem prover: type theory for syntax and
  semantics", 2016. https://arxiv.org/abs/1602.00812 ; Grail 3,
  https://github.com/RichardMoot/Grail

Read from the repository snapshot: `plan/33-mell-nets.md`,
`plan/later.md` ("MELL proof nets with exponential boxes", "MALL proof
nets"), `plan/README.md` (D5, D6a, D12, D15–D18), `.claude/rules/core-nets.md`,
`core/src/nets/{mod,graph,sequentialize,skeleton}.rs`,
`core/src/serialize/nets.rs`, `core/src/export/svg/{mod,net}.rs`,
`core/src/proofs/{mod,check}.rs`, `plan/reports/05-proof-nets.md`,
`plan/reports/11-svg.md`, and the research notes `33-mell-nets.md`,
`34-cut.md`, `32-web.md`, `35-mll-engines.md`, `README.md`.
