# Research notes for steps 29 to 38: what they ask of the library

Written 2026-10-08 from the eleven notes in this directory and
`.claude/rules/core.md`. Everything below is taken from the notes;
"(inference)" marks a conclusion of this summary's own. Steps are cited
by number, the refutations note as "ref".

## 1. The notes

| step | file | one line |
|---|---|---|
| 29 | `29-comparison.md` | linlog beside eight provers under BenchExec: process timing, three-valued verdicts, translators from the parsed `Sequent` |
| 30 | `30-release.md` | 0.1.0 on crates.io: semver in 0.y, `#[non_exhaustive]` before the tag, MSRV, docs.rs, changelog, Trusted Publishing from 0.1.1 |
| 31 | `31-rocq.md` | a Rocq kernel (`ll`, `ill` in `Prop`) and a reflective checker mirroring `check.rs`; certificates for every mode |
| 32 | `32-web.md` | wasm in a worker with page-held JSON state; a progress-carrying `Stop`, tab presets, 32-bit tests |
| 33 | `33-mell-nets.md` | MELL nets with boxes: vertices as instances, one generalized `?` node, one door per `?`-instance and box, jumps |
| 34 | `34-cut.md` | `Cut(OccId, NodeId, NodeId)`, dual pairs by offset in an extended forest, bounded elimination on terms and MLL nets |
| 35 | `35-mll-engines.md` | leaf symmetry breaking over pure trees, a balance prune, essential nets as a criterion behind the one net engine |
| 36 | `36-lambek.md` | cyclic MLL and Lambek by a planar linking search: unsorted roots, a mode-aware dual, `cyclic` in `Mode`, rule R4 |
| 37 | `37-inverse.md` | the focused inverse method as `--engine inverse`: the weak flag is the checker's `any`; `Context`, `Classes` lifted |
| 38 | `38-first-order.md` | quantifiers: levels instead of Skolem terms, locally nameless hash-consed terms, members as `(OccId, frame)` |
| ref | `refutations.md` | certified refutations in five groups; `Refutation::check` beside the engines; refuters as a second plug-in kind |

## 2. Requirements shared by several steps

Grouped by the item of the API. Each names the steps, then what a later
session looks for to see it met.

**Data model**

- *A public, non-recursive formula walk with room for binders*
  (29, 32, 36, 38, ref): per-tool printers, the goal view, the two
  divisions, binder stops, the printer to the Rocq formulas. Met when
  `sequents::fmt::Walk` is public with `Enter`/`Between`/`Exit` and a
  binder stop, and `core/tests/depth.rs` runs every printer built on it
  at 100 000 levels (`core.md`).
- *Root order is the written order; extra trees follow the roots*
  (29, 34, 36). `optimize` sorts roots today: 36 needs the written order
  canonical, 29 loses the clause roles, 34 appends `A`, `A⊥` per cut
  after the roots and needs `Reading::new` to ignore them. Met when
  `Sequent`'s docs name the canonical form, the snapshots are
  regenerated in that commit, and `Forest` has `cut_pairs()`.
- *The preorder numbering is a stated contract* (31, 33, 34, 36): the
  Rocq checker recomputes the forest, `dual(x) = root(A⊥) + (x −
  root(A))` and "ascending ids are the cyclic order" rest on it, 33's
  vertex table keeps ids of the net's own. Met when `Forest`'s doc calls
  the numbering a promise a foreign checker may recompute, with a test.
- *A bound on every entry that builds a forest or a table*
  (32, 34, 35, 37). `Interactive::new` and every `Deserialize` use
  `Forest::DEFAULT_LIMIT` with no way to pass another; the closure
  matrix and the inverse method's database must charge `memory::Account`
  and answer `Reason::MemoryLimit`. Met when `Interactive::within` and a
  `from_json_within` exist and each engine has a test at its limit.
- *Zones that carry more than an occurrence id* (33, 37, 38). 37 lifts
  `Context` and `Classes` to `search/`; 38 needs members `(OccId,
  frame)` and an engine generic over its zone type with no branch on
  the ground path (D17); 33's instances are `(OccId, origin Copy)`. Met
  when the lift is a counter-neutral commit (`bench/targets.sh` columns
  equal) and `plan/notes/api.md`, which 38 requires, names the zone and
  frame types.
- *`Mode` and `Fragment` can grow* (29, 30, 36, 38): `cyclic` with
  `serde(default)`, a `quantifiers` flag, finer admission flags. `Mode`
  is three public fields, unmarked, so any field breaks the lib.rs
  example. Met when `Mode` is `#[non_exhaustive]`, built by its
  constants and `with_*` builders.

**Proof term**

- *`Node` stays 16 bytes and its variant policy is written down*
  (30, 31, 33, 34, 38). 34 adds `Cut`; 38 adds `Forall(OccId, Eigen,
  NodeId)` and `Exists(OccId, Witness, NodeId)` with a witness arena the
  `Proof` owns; the Rocq `node` mirrors it. 30 recommends exhaustive,
  each variant a 0.y bump. Met when the size assertion holds, the
  changelog's policy line says exhaustive, and the Rocq datatype has one
  constructor per variant.
- *`Proof` carries what rebuilds its forest* (31, 34, 38): the cut
  formulas, the witness arena, and the `Mode` the exporter needs, which
  `Derivation` does not supply. Met when `Proof::new` takes them and
  `Answer::of_arena` passes them.
- *`Rule` grows by one checklist* (34 `Cut`; 36 `\L \R /L /R`; 38 `∀ ∃`
  and the two-sided four): `Rule::ALL`, `name`/`from_str`, the label
  tables, the size estimate, the Rocq printer. Met when
  `core-export.md`'s list is the one place and a round-trip test covers
  `Rule::ALL`.

**Checker**

- *Checker, oracle and Rocq mirror change together* (31, 34, 36, 37, 38).
  The `Cut` arm, the per-`⊗` cyclic test and R4, the framed members and
  eigenvariable set go into `Pass::rule` and `oracle.rs` alike; 31
  translates `oracle.rs` rule by rule and must say which refusals
  (`Surplus`, `Memory`) have no Rocq counterpart. Met when
  `core-proofs.md` states each rule's state semantics and the flake's
  Rocq check compiles every certificate of both kernels with `Print
  Assumptions`.
- *`check_within` holds the large inputs* (29, 31, 32): the 14 nets that
  filled 12 GiB, `vm_compute` on 46 768 nodes, `MOST` at 32 bits. Met
  when those nets check inside the comparison's memory limit and the
  Rocq report gives the time on the largest snapshot.
- *A refutation checker beside the engines* (29, 31, ref):
  `Refutation::check(&forest, goal, mode, fragment)` in a module of its
  own, sharing no engine code. Met when a Rust checker exists per
  certificate kind before any Rocq, payloads pinned in
  `core/tests/serialize.rs`.

**Engine interface**

- *One stop and one account for every long run* (31, 32, 34, 37, ref):
  `Stop::poll(progress)` with the work since the last poll, kept for
  `FnMut() -> bool` by a blanket impl; polled by the exporter, the
  inverse method, elimination and the refuters alike. Met when the trait
  exists, every poll site uses it, and a search at `recursion_limit`
  passes under wasm32.
- *Registering an engine is a list* (29, 35, 36, 37): the `Engine`
  variant with `Display`, `implementation`, `parallel`, a `Feature` and
  a `DISPATCH` row, `cli::EngineArg`, the JSON `engine` string, README's
  blocks with `cli/tests/readme.rs`, `reference.rs::configurations`, the
  harness's `--engines`; 29 wants `Engine` to give its fragments and
  modes as data. Met when `core-search.md` holds the list and a test
  round-trips every variant through its name.
- *Refusals are variants of non-exhaustive enums* (30, 33, 35, 36, 38):
  `Error`, `NetError` and `ShapeError` gain variants but are unmarked,
  and marking later is itself major. Met when all three carry
  `#[non_exhaustive]` before the tag.
- *`from_proof` and the net's flag* (33, 34, 35, 36). `from_proof` reads
  `Ax` only; 34 needs `Cut`, 35 warns it would silently drop links. All
  four extend `ProofStructure::new(mix)` (vertex table, cut slot,
  `Criterion`, `cyclic`). Met when `from_proof` errors on a node it does
  not handle and the flag is one value all four extend.

**Wire forms**

- *serde on `search::Options`* (29, 30, 32). Met when it round-trips in
  `core/tests/serialize.rs` with `default, deny_unknown_fields`, `jobs`
  and `pool` absent without `parallel`.
- *A version on every form read back, and additive keys only*
  (29, 30, 32, 33, 34, 36, 38, ref): `"version"` on `Proof`,
  `ProofStructure`, `Interactive`, `Sequent`, missing read as 1; the
  crate version and options beside a verdict; every new key under
  `serde(default)`. Met when a test reads a pre-step file unchanged per
  form (38's rule) and the changelog has a heading for form changes.
- *Codes, not English, for `Error` and `Refusal`* (32, 34). Met when
  both serialize with a stable code and `ParseError::span` is in UTF-16
  units or documented as bytes.
- *Statistics and refutations grow by named fields* (32, 35, 37, 38,
  ref): new counters, `Saturated`, `Classical { assignment }`,
  `Failure`, `PhaseModel`, `KripkeModel`, the class weights in
  `StateEquation`. Met when each has its proxy line and serialize test;
  (inference) new fields rather than new meanings, since
  `bench/targets.sh` and 29's tables read `nodes` and `memo_hits` by
  name.
- *Numbers above 2⁵³* (32, ref): `Size`, `Equation.needed: i128`,
  `memory_limit`, the counters. Met when one documented rule has a test.

**Options**

- *`#[non_exhaustive]` on every options value and `svg::Style`*
  (30, 33, 35, 36, 38): box, arrow, polarity, notation and witness
  fields are coming. Met when every `pub struct Options` and `Style` is
  marked and callers write `Options::default()` plus assignments.
- *`rocq::Options.kernel`* (31, 34, 36, 38, ref): `Kernel { Auto,
  NanoYalla, Linlog }` with snapshots byte-identical under `Auto`; cut
  refused on NanoYalla unless `macrollcut` or the new kernel; no cyclic
  certificate on NanoYalla; `Unsupported` for quantifiers;
  `rocq::refutation(…)`. Met when `rocq::write_proof(&Proof, Mode,
  &Options, out, stop)` exists and `Unsupported` is kernel-specific.
- *Named defaults with a flag and a measurement* (29, 32, 35, 37, 38,
  ref): the BenchExec flags, `Options::browser()` and
  `ViewOptions::browser()`, `test_period` missing from the CLI, the new
  bounds. Met when every `DEFAULT_*` has a CLI flag (D16) and each
  preset cites its measurement.

## 3. Conflicts and their resolution

1. **Exhaustive enums against steps that add variants.** 30 keeps
   `Term`, `Kind`, `Node`, `Rule` exhaustive with a 0.y bump when they
   grow; 34, 36 and 38 grow them after the release. 35 wants its
   `NetError`, `Style` and `ProofStructure::new` changes to land "before
   the release freezes them", but 30 precedes 35 (D23). Resolution: mark
   the extensible types at 28, keep the four enums exhaustive, and write
   the version plan (0.2.0 at 34, 0.3.0 at 38) into the changelog.
2. **Root order.** 36's unsorted roots shift every stored id; 34's D5
   stability and the release make a later shift costly. Resolution
   (inference): stop sorting at 28, before 0.1.0, in the commit that
   regenerates the snapshots.
3. **The dual.** 34's offset map needs `A` and `A⊥` to have the same
   shape with children in place; 36's cyclic dual reverses a product's
   factors. Resolution (inference): state the invariant for the
   commutative dual and give the cyclic lowering a mirrored map or a
   table, once 36's panel confirms the lowering.
4. **The net structure's flag.** Four steps extend
   `ProofStructure::new(mix)`, and 33 asks one type or two. Resolution:
   one `Criterion`-like value decided at 28; MLL stays the engine's hot
   structure, and `bench` shows the generalization costs nothing (33).
5. **Symmetry breaking and routing.** 35 generalizes the leaf chains and
   proposes `Feature::NoEqualLeaves`; 36 finds the break unsound under
   order; 37 wants a feature of its own. Resolution: the break off when
   ordered; `Feature` as data on the row, kept only where the harness
   shows a win.
6. **Zone types.** 37's lift must keep the focused counters identical;
   38's generic zone must add no branch to the ground path. Resolution:
   lift first as 37's own commit, then 28's spike (stage 2, item 4)
   measures the generic form before 38 fixes it.
7. **The stop.** 32's progress stop touches every poll site, and 37 adds
   more. Resolution: change `Stop` at 28, as 32's question 4 suggests.
8. **Who decides a verdict.** ref's refuters turn `Unknown` into
   `Unprovable`; 37 and 29 count bounded failure as unknown. Consistent
   under ref's rule: a refuter never changes a verdict the search gave.
9. **`Outcome` write-only.** 30 and 32 accept it; ref needs a refutation
   file for `linlog check`, 29 certificates as ground truth. Resolution:
   `Refutation` payloads readable with a version; `Outcome` stays
   write-only and says so.
10. **`Prop` against `Type`.** 31 recommends `Prop`; Yalla and NanoYalla
    are in `Type`, so only `theirs -> ours` is then provable. The author
    decides (31, question 1).

## 4. What the API design should decide first

In order of how many steps wait on it:

1. The extensibility policy: which enums stay exhaustive, which types
   take `#[non_exhaustive]`, and the version plan per step (30, 33 to 38).
2. What a zone member is (`OccId`, `(OccId, frame)`, a net instance),
   where `Context` and `Classes` live, and the node's occurrence
   argument as a type the Rocq side can parameterize (31, 33, 34, 37, 38).
3. The forest's contract: preorder as a promise, written root order
   canonical, extra trees after the roots, `Reading` over the sequent's
   roots, a bound on every entry (29, 31 to 38).
4. `Stop` with progress and `Account` charging as the one interface for
   searches, checks, exports, elimination and refuters (31, 32, 34, 35,
   37, ref).
5. The wire-form policy: version field, additive keys, pre-step-file
   tests, codes for `Error` and `Refusal`, the 2⁵³ rule, serde on
   `search::Options`, which forms are read back (29, 30, 32 to 38, ref).
6. The export entry over the term with `Kernel`, and the SVG id scheme a
   client clicks (`o<n>`, `l<m>-<n>`, state ids beside `i<n>-<p>`)
   (31 to 36, 38, ref).
7. Options discipline: one value per feature, named defaults with flags
   and measurements, presets for the browser and BenchExec (29, 32, 34,
   35, 37, 38, ref).
8. The engine registration list, `Engine` as data, and the second
   plug-in kind for refuters (29, 35, 36, 37, ref).
