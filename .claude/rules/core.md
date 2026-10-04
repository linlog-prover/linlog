---
paths:
  - "core/**"
---

# linlog core: the crate, and where its rules are

Loaded when any file under `core/` is read. This file holds what is
true of the whole crate; the invariants of each module are in a file of
their own, which loads beside this one when a file of that module is
read, so a session reads the rules of the code it touches and no others.
A note for the rules goes into the file of its module, one point per
bullet; one that two modules need lives in one file and the other names
it; only what holds for the whole crate goes here.

| file | loads for | what it holds |
|---|---|---|
| `core-sequents.md` | `sequents/`, `parse/`, `fragment.rs`, `serialize/`, the parse and serialize tests | the arena and its order, negation normal form, terms, kinds, fragments and modes, the parser, the JSON forms of every value |
| `core-forest.md` | `occurrences/` | the occurrence forest (numbering, the bound on its size, the atom bias, literal lists), `OccSet`, the intuitionistic reading |
| `core-proofs.md` | `proofs/mod.rs`, `check.rs`, `oracle.rs` | proof terms and their invariants, the checker: its one pass, its memory bound, its integers, the one-succedent condition |
| `core-derivations.md` | `proofs/derivation.rs`, `size.rs`, `fmt.rs`, `multiset.rs`, `interactive.rs` | the derivation view, the size estimate, the bounds of `ViewOptions`, the compact view, the text tree, interactive proving |
| `core-search.md` | `search/mod.rs`, `memory.rs`, `additive.rs` | the front door (`prove_goal`, the check of every proof, `Outcome`, `Options`, refutations, the dispatch, where every engine polls its stop), the memory bound, the additive path |
| `core-focus.md` | `search/focus/`, `search/generate.rs` | the focused engine, one- and two-sided: dyadic sequents, the copy bound and the memo, the two searches of the default bias, the arena, counts, interchangeable occurrences, the split search, recursion, allocation |
| `core-nets.md` | `nets/`, `search/net.rs` | proof structures, the correctness criterion, sequentialization, the net engine |
| `core-parallel.md` | `search/parallel.rs`, `search/focus/parallel.rs`, `search/net.rs` | the pool, stops, cube-and-conquer, the shared memo and arena, the net engine's cubes, what a pool promises |
| `core-export.md` | `export/`, `proofs/style.rs`, the export test and snapshots | the options values, the one signature, rule labels, notations, the packages' limits, Typst's own layout, fonts, SVG, PNG, PDF, Rocq |
| `core-inputs.md` | `lltp.rs`, `families.rs` | the LLTP reader and the generated families |

## Layout

`sequents` (arena, printing), `parse`, `serialize`, `fragment`, `occurrences`
(forest, sets, and the intuitionistic `reading`), `proofs` (terms in
`mod.rs`, `check`, `derivation`, the renderer `fmt`, the crate-private
`multiset`, and `interactive` behind the feature of that name), `search`
(the front door in `mod.rs`, the focused engine in
`focus/` with `counts` and `memo`, the net engine in `net`, the additive
path in `additive`, the test-only `generate` with its classical and
intuitionistic proof generators), `nets` (structures and the criterion's front door
in `mod.rs`, the graph and the Yeo test in `graph`, the union-find in
`skeleton`, `sequentialize`), and `export` (the shared `notation`, and
`latex`, `typst`, `svg` and `rocq` behind the features of those names). `lib.rs`
re-exports the public types, so users write
`linlog::Sequent`, `linlog::Proof`, `linlog::prove`, and so on. `hash` is
crate-private.

## Crate-wide rules

**Nothing recurses over a formula.** A sequent read from JSON can be nested
as deep as it is long, and a recursion per level ends the process where
an error is owed. So every walk is a pass over the arena or over the
forest in index order (`fragment`, `optimize`, `Forest`, `Reading::new`,
the engines' `Classes` and `Counts`), or, where a formula is written, a
loop over `sequents::fmt::Walk`: the stops of a formula in the order it is
written (`Enter` a subformula, `Between` the two subformulas of a binary
one, `Exit` a compound one), from a stack of its own whose first sixteen
steps are inline, so that an ordinary formula costs no allocation. The
`Display` of `Sequent` and `Formula`, `Reading`'s printing, the exports'
`Notation::term` and `Notation::ill` and the Rocq printer are each one
`match` over those stops, and a new printer is another: never a function
that calls itself. `core/tests/depth.rs` runs every walk the public API
offers on formulas nested 100 000 deep, on a thread with a stack of
256 KiB.

Doc examples that parse are fenced with `cfg_attr(feature = "parse", doc =
"```")` and an `ignore` fence otherwise, so `cargo test --no-default-features`
passes; copy that pattern for a new example.
