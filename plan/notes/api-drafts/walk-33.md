# Walk-through of step 33 (MELL proof nets with boxes) against `api.md`

Read: `api.md` in full, `plan/33-mell-nets.md`, `research/33-mell-nets.md`,
`mell-nets-spec.md` (section 7 and the notation of section 8),
`corpus-mell-nets.md`, `impact-boxes.md`, the register entries naming 33
(28 of them), and `core/src/nets/{mod,graph,sequentialize}.rs`,
`serialize/nets.rs`, `export/svg/{mod,net}.rs`, `search/net.rs` (use of the
net API). Section numbers are `api.md`'s.

## 1. The first change, sketched

The prompt's first session is "the structure, the criterion and both
conversions". Two commits come first, and neither changes a propositional
byte.

**Commit 1, "Give nets a vertex table, boxes and jumps"** (counter-neutral;
MLL keeps `VertexId == OccId`, no table).

```rust
// nets/mod.rs; BoxId re-exported from lib.rs beside VertexId (2.2 lists neither)
#[repr(transparent)] pub struct BoxId(u32);          // its own id space (P2); Space::Box is new
// private, behind ProofStructure.exp: Option<Box<Exponential>>  (None for plain MLL: P10)
struct Exponential { rows: Vec<Row>, boxes: Vec<BoxRow>, jumps: Vec<(VertexId, VertexId)>, by_occ: Csr }
struct Row    { occ: OccId, parent: Option<VertexId>, within: Option<BoxId> }   // a door: occ = the ? occurrence, same as its collector
struct BoxRow { principal: VertexId, parent: Option<BoxId> }

impl ProofStructure {                    // the accessors 3.11/10.5 name, with their types
    pub fn boxes(&self) -> impl ExactSizeIterator<Item = BoxId> + '_;
    pub fn box_of(&self, v: VertexId) -> Option<BoxId>;        // innermost; `!` itself is outside its box
    pub fn depth(&self, v: VertexId) -> u32;
    pub fn jumps(&self) -> &[(VertexId, VertexId)];
    // NOT in the design, needed (items 1, 2): 
    pub fn instances(&self, o: OccId) -> impl Iterator<Item = VertexId> + '_;
    pub fn builder(forest: Forest, criterion: Criterion) -> Result<NetBuilder, Error>;
}
impl NetBuilder {                        // validated, every call a NetError and no change on refusal
    pub fn instance(&mut self, occ: OccId, parent: Option<VertexId>, within: Option<BoxId>) -> Result<VertexId, NetError>;
    pub fn open_box(&mut self, principal: VertexId, parent: Option<BoxId>) -> Result<BoxId, NetError>;
    pub fn door(&mut self, of: VertexId, within: BoxId) -> Result<VertexId, NetError>;   // of: collector or outer door
    pub fn link(&mut self, x: VertexId, y: VertexId) -> Result<(), NetError>;
    pub fn jump(&mut self, from: VertexId, to: VertexId) -> Result<(), NetError>;
    pub fn finish(self, limits: &Limits) -> Result<ProofStructure, Error>;    // builds the CSR once
}
// NetError, #[non_exhaustive] (3.11), new variants, all named-field:
//   Fragment { fragment }, Mode { mode },                       // new/Criterion::of refusals (item 4)
//   LinkAcrossBoxes { link }, JumpAcrossBoxes { jump }, JumpToNeighbour { jump },
//   MissingJump(VertexId), DoorWithoutPremise(VertexId), DoorMismatch { door, premise },
//   CollectorAcrossBoxes { collector, premise }, NotInBox(VertexId)
// Space gains Box; Phase::Net, Refusal::Index { what: Space::Vertex } as designed.
```

`from_proof(proof, Criterion, &Limits, stop)` is commit 3; its walk keeps
an explicit stack carrying (box, per-occurrence pool of unused instances),
one visit per path through shared nodes, `Refusal::Memory` on the table.

**Wire** (level 2: `vertices` changes what the ids in `links` mean, 7.1).
Corpus item 5, `⊢ ?X⊥, !X ⊗ X` (forest ids: 0 `?X⊥`, 1 `X⊥`, 2 `⊗`, 3 `!X`,
4 `X`, 5 `X`; vertices q0 t1 bx2 d3 x4 y5 y2=6 x2=7):

```json
{"version": 2, "sequent": {…}, "mix": false,
 "vertices": [[0,null,null],[2,null,null],[3,1,null],[0,0,0],[4,2,0],[1,3,0],[1,0,null],[5,1,null]],
 "boxes": [[2,null]], "links": [[4,5],[7,6]], "jumps": []}
```

A row is `[occurrence, parent, box]`; a door is the row whose parent has the
same occurrence. Corpus item 10, `⊢ ?X⊥, !(?Y ⅋ X)`: occurrence 5 (`Y`) has
no row (the weakened `?Y` is a collector with no premise), so
`instances(OccId(5))` is empty, `jumps: [[3,4]]`. Corpus item 40,
`⊢ X⊥, !X` with the link `x — y` across the border: `finish` returns
`Net(LinkAcrossBoxes { link: (v2, v0) })`, kind `malformed`.

**Commit 2, "Check nets at each box depth"**: `Graph` built from the table
(variable-arity `?` and door slots, no tree edge from `!` to its content, a
box one vertex in its parent's graph); `deletable` treats a `?`/door with
two or more premises like `⅋`; `is_correct(stop)` returns the witness with
the box; `acyclic` stays allocation-free through `Scratch`; the
enumeration test in `graph.rs` extended to boxes and checked against the
corpus (50 items: 43 are 33's, items 18 to 21 and 42 to 44 are step 34's).

## 2. Workarounds

**1. No way to construct, or mutate, a MELL structure. `friction`.**
Need: a net of the corpus's kind (hand-built, incorrect ones included) for
the oracle tests (R201), for the JSON reader (R8 "validates boxes as it
validates links") and for step 34's later growth. Design: 3.11 gives only
`new(forest, Criterion)` and `from_links(forest, Criterion, &[(VertexId,
VertexId)])`; a caller cannot name a `VertexId` past the forest before the
vertex exists, so `from_links` cannot build a MELL net, and `link/unlink`
are a stack over a CSR fixed at construction (graph.rs: "premise edges fixed
at construction"). 10.5 lists accessors only; R76's "validated way to add a
box" is unplaced, and so is what `new` does on a MELL sequent (R76/R89 want
it to succeed with an empty box list, yet every `!` occurrence is a box).
Smallest change: add to 3.11 a staged builder as sketched (`builder` ->
`NetBuilder` -> `finish(&Limits)`), say that the vertex table and the boxes
are immutable after `finish` while `link/unlink` stay the stack, and say
that `new` on a MELL sequent returns the copy-free structure (one box per
`!`, every `?` a collector with no premise) or refuses it with
`NetError::Fragment`. Additive, so not blocking, but the reader, the tests
and 34 all go through it.

**2. "Past the forest" contradicts what a MELL table must be. `friction`.**
Need: an occurrence under a never-copied `?` has no vertex (corpus 10: `Y`
under a weakened `?Y`); an occurrence under a copied `?` has several
(corpus 5: `X⊥` twice). Design: P2, 2.4's "a net's vertices past the forest
are instances of `?` subtrees" and 10.5's "vertex table past the forest"
promise the prefix `VertexId(i) == OccId(i)` also in MELL; 3.11's own
comment ("in MELL an instance, a collector, a door") and `vertex(o) -> Option
(None when o has several instances)` do not. Without the 34 "erased set"
(placed at 34, 3.11) a hole cannot stay in the prefix. Why it matters: the
wire's `vertices` is all rows or only the extra ones, and `vertex()`'s
documented `None` is wrong for zero instances; fixed at 0.1.0 in prose it
becomes a silent contract. Smallest change: state "a structure with a table
numbers its own vertices; the identity holds exactly when there is no
table", document `vertex(o)` as `Some` only for exactly one instance, and
add `instances(o)`.

**3. `NetError`'s tuple variants cannot carry the box. `blocking` for the
variants as written.** Need (10.5, R132, spec section 2): "witnesses naming
the box" for a cycle and for disconnection, and `Empty` only at depth 0.
Design: 3.11 gives `SwitchingCycle(Vec<VertexId>)`, `Disconnected(Vec<Vec<
VertexId>>)`, `NotDual(VertexId, VertexId)`; P3's rule marks "struct-like
variants" `#[non_exhaustive]`, and these are tuple variants, so adding a
second field breaks every 0.1.0 `match`. The alternative (parallel
`BoxSwitchingCycle`, `BoxDisconnected`) leaves two variants for one fact and
a 0.1.0 client that handles `SwitchingCycle` shows a depth-1 cycle with no
box. Smallest change: in 3.11 write every data-carrying `NetError` variant
with named fields (`SwitchingCycle { cycle }`, `Disconnected { parts }`,
`NotDual { x, y }`), `#[non_exhaustive]`; 33 then adds `within:
Option<BoxId>`. Also 4.3's "details: the variant's named fields" needs them.

**4. `NetError` has no variant for the refusals `new` and `Criterion::of`
return; `has_nets()` serves two masters. `friction`.** Need: `new` on an
additive sequent, `Criterion::of(affine)`, R133 ("refuse with an error whose
message names the fragment"), R145. Design: 3.11's comment says
`NetError::Mode`, but the enum lists neither `Mode` nor `Fragment`; 4.4 folds
`NetFragment`/`NetMode` into `EngineRefused` (engines only) and says "a
structure's own refusal is NetError". Separately 3.5 makes `Fragment::
has_nets()` "the one predicate for `ProofStructure`, the net engine and the
command", but at 33 the structure takes MELL and units while the net engine
stays at `Fragment::MLL` (8.6, R89, R174): one predicate cannot answer both.
Smallest change: add `NetError::{Fragment { fragment }, Mode { mode }}`
(kind `unsupported`) and word 3.5 as "`has_nets()`: where a structure
exists; each engine's `admits` keeps its own largest fragment".

**5. `Error::describe(&Forest)` cannot print an instance vertex.
`friction` (a signature fixed at 0.1.0, `blocking` for the 38 walk).** Need:
formulas for the vertices of a box witness (door, second instance), R132
("printing ids and, via describe(&forest), formulas"). Design: 4.1 fixes
`describe<'a>(&'a self, forest: &'a Forest)`; a vertex past the forest
means nothing to a `Forest`, and the vertex table lives in
`ProofStructure`. 3.4 solves the same for `Member` by owner methods
(`Proof::formula(m)`) but not for the errors. Smallest change: make the
parameter a sealed trait `Owner` (`Forest`, `Proof`, `ProofStructure`,
`Interactive`) now; the call `err.describe(&forest)` still compiles.

**6. `Member` and "a copy's instance in a net at 33" do not match what
`from_proof` has. `note`.** Need: instances for desequentialization. The
proof's operands at 33 are occurrence-valued members (no table until 38's
`with_instances`, 3.7), and shared nodes are identical for every copy
(`Ax(x, y)` is one node for three copies), so the walk assigns instances by
a pool per occurrence under the path, and the result is canonical only up to
isomorphism. Design: 3.4's text says a member past `forest.len()` is "a
copy's instance in a net at 33"; nets speak `VertexId`, never `Member`.
Also 3.11 gives no equality (no `PartialEq`, correct) and no test for "same
net up to isomorphism", which R78 and R122 need as "net(proof) equal to
net(sequentialize(net(proof)))". Smallest change: delete the 33 clause from
3.4; add a public or test-only `ProofStructure::isomorphic(&self, &other)
-> bool` (jumps ignored) to 3.11.

**7. The gate on the net engine has no instrument. `friction`.** Need:
retype `partner`/`link_unchecked`/`unlinked` to `VertexId` and branch on the
table in `is_complete`/`is_acyclic` without moving `links`/`tests`. Design:
3.11 says "the net engine's rows of the target set keep `links` and `tests`,
taken before the retype", but `bench/targets.sh` runs the focused engine
only and none of the 20 ratchet journeys is a net-engine search (R206 is
`Not met`, and appears nowhere in `api.md`). Smallest change: add R206 to
the 28 fix list as a commit before the retype, and list it in section 13.

**8. Per-variant `kind()` and the code for box errors are undecided.
`note`.** Need: `linlog check --net f.json` with `LinkAcrossBoxes` is a
malformed file (exit 2) in the corpus's own terms ("not a proof structure"),
`SwitchingCycle` an invalid net (exit 1). Design: 4.4 maps all of
`Net(Box<NetError>)` to `invalid_net`/invalid and also "reserves codes for
the box kinds of NetError (33)", so a box error is either a new code or a
`details.kind` of `invalid_net`, and a structure error is either kind.
Smallest change: one sentence in 4.4 stating `NetError::kind()` per variant
(construction errors malformed, criterion errors invalid) and that box
variants ride `invalid_net` with `details.kind`.

**9. Levels, goal proofs, ids. `note` each.**
(a) 7.1 computes the level from content; a net of a MELL or unit sequent
with an identity table and no jumps would read as level 1 and fail in a
0.1.0 reader with a fragment error, not `unsupported_version`. Add "a
structure whose fragment 0.1.0 refused" to what raises the level.
(b) `from_proof` of a goal proof (3.7) is unstated; it must refuse
(`Error::GoalProof`) since the net would have unlinked literals.
(c) 9.3 reserves `b`, `d`, `j` but door vertices are vertices (`o<n>`):
state that `d<n>` is the door's vertex id, `b<n>` the `BoxId`, `j<n>` the
jump's index, and that a door has no `o` id.
(d) R165's permuted layout is the drawer's internal order; no hook is
needed, but 9.1 should say `svg::net` may draw conclusions in an order
other than `Forest::roots()` (C1 keeps that order canonical).
(e) New public types `BoxId` and the box view (if any) need a P3/2.5 entry
and a re-export; `Space::Box` is additive (Space is non-exhaustive).
(f) `Display for ProofStructure` and the SVG `<desc>` run `is_correct` with
no stop or limit (5.6 gives `is_correct` `memory_bytes`, 3.11 gives it only
`stop`); `is_acyclic(&mut Scratch)`, `scratch()` and `is_complete` are not in
3.11, and `is_acyclic` has no poll per round, so R25's "box-aware calls are
boundable" is only met for `is_correct` and the conversions.

## 3. Register entries naming step 33

| entry | verdict |
|---|---|
| R6 | met, 7.1 (see 9a) |
| R8 | placed, 7.3 and 10.5 (additive `vertices`, `boxes`, `jumps` at a level); the reader's validation needs the builder (item 1) |
| R25 | met, 3.11 (`from_proof`, `is_correct`, `sequentialize` with `Limits` and stop); `is_acyclic` unbounded (9f) |
| R26 | met, 9.1 `svg::net(…, &Limits, stop)`; the estimate counting boxes is the step's |
| R43 | placed, P9; depth test for nested boxes is the step's |
| R50 | partly, 2.5 lists `NetError`, `Criterion`, `Style`; new 33 types unplaced (9e); item 3 |
| R76 | partly: accessors placed (10.5), the validated way to add a box and `new` on MELL are not (item 1) |
| R77 | partly: P2 and 3.11 place the table, but contradict each other on the prefix (item 2); growth after construction is 34's erased set |
| R78 | placed, 10.5 (generalized `?`, jumps from the term, C3 answer A); the isomorphism test is not (item 6) |
| R79 | met, 3.11 `Criterion` |
| R89 | met, 10.5 and 8.6; consistent only if item 4 is applied |
| R102 | met, 10.5 |
| R117 | met: no `Node` added, 3.7 |
| R122 | placed, 10.5 and 3.11 |
| R129 | partly: tuple variants have no named details (item 3), kind and code per variant open (item 8) |
| R132 | partly: variants reserved in 3.11 and 4.4, box in witnesses not carriable (item 3), `describe` (item 5) |
| R133 | partly: `has_nets()` in 3.5, no error variant, conflated with the engine (item 4) |
| R144 | met, 9.3 |
| R145 | met in 3.11 (`Criterion::of`), error type missing (item 4) |
| R165 | placed loosely, 10.5 (9d) |
| R168 | met, 9.3 ids; `<desc>` text form unchanged by the design (9c, 9f) |
| R174 | command area; needs only 3.5 and `from_proof`, both placed |
| R201 | not placed: oracle test; needs the builder (item 1) |
| R206 | not placed (item 7) |
| R216 | met by 10.10 (l) and 11 for the focused engine only |
| R227 | not placed: documentation sweep, the fix area's |
| R244 | met, 3.4 (clause to delete, item 6) |
| R254 | met, 3.11 |

## 4. What fits well

- `VertexId` distinct from `OccId` in every `nets` signature from 28, with
  `Criterion` replacing `mix: bool` and its `mix` key flattened: step 33
  changes no signature of `link`, `partner`, `is_correct` or the JSON's
  MLL bytes, and no `Node` is added (so 31's mirror and the oracle stay).
- `partner` as an array per vertex means the net engine's hot path needs no
  branch; the table is an `Option<Box<_>>` beside it (P10).
- Levels by content (7.1), `Refused(Refusal)` in `NetError`, `from_proof`
  exhaustive and bounded (3.11), the reserved `b`/`d`/`j` ids and a
  non-exhaustive `Style` (9.3) cover the conversions, the limits and the
  drawing without a rewrite.
