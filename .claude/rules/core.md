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
| `core-search.md` | `search/mod.rs`, `memory.rs`, `additive.rs`, `reference.rs` | the front door (`prove_goal`, the one engine interface `Decide`, the check of every proof, `Outcome`, `Options`, refutations, the dispatch, where every engine polls its stop), the memory bound, the additive path, the test-only reference prover |
| `core-focus.md` | `search/focus/`, `search/generate.rs` | the focused engine, one- and two-sided: dyadic sequents, the copy bound and the memo, the two searches of the default bias, the arena, counts, interchangeable occurrences, the split search, recursion, allocation |
| `core-nets.md` | `nets/`, `search/net.rs` | proof structures, the correctness criterion, sequentialization, the net engine |
| `core-parallel.md` | `search/parallel.rs`, `search/focus/parallel.rs`, `search/net.rs` | the pool, stops, cube-and-conquer, the shared memo and arena, the net engine's cubes, what a pool promises |
| `core-export.md` | `export/`, `proofs/style.rs`, the export test and snapshots | the options values, the one signature, rule labels, notations, the packages' limits, Typst's own layout, fonts, SVG, PNG, PDF, Rocq |
| `core-batch.md` | `search/batch.rs` | many sequents in one call: the order of results, the workers and the stream, how the batch's memory bound is shared |
| `core-inputs.md` | `lltp.rs`, `families.rs` | the LLTP reader and the generated families |
| `core-ordinary.md` | `ordinary/` | ordinary logic: its syntax and TPTP reader, the translations as one pattern table, the read-back to LK and LJ, its checker, the certificate over `Prop` |

## Layout

`sequents` (the arena, its terms, and the formula walk `fmt`), `parse`,
`serialize`, `fragment`, `errors` (the one `Error` and the parse error),
`occurrences` (forest, sets, and the intuitionistic `reading`), `proofs`
(terms in `mod.rs`, `check`, the checker's first implementation `oracle`
for tests only, `derivation`, `size`, the text tree `fmt`, the rule
labels `style`, the crate-private `multiset`, and `interactive` behind
the feature of that name), `search` (the front door in `mod.rs`, the
memory account in `memory`, the focused engine in `focus/` with
`arena`, `classes`, `context`, `counts`, `memo`, `schedule`, `scratch`,
`split`, `tests` and `parallel`, the net engine in
`net`, the runtime of the pool in `parallel` behind the feature of that
name, the additive path in `additive`, the test-only `generate` with its
classical and intuitionistic proof generators and the test-only
`reference` prover), `nets` (structures and
the criterion's front door in `mod.rs`, the graph and the Yeo test in
`graph`, the union-find in `skeleton`, `sequentialize`), `export` (the
shared `notation`, and `latex`, `typst`, `svg` with `font`, `tree` and
`net`, `png`, `pdf` and `rocq` behind the features of those names),
`ordinary` (the arena and syntax in `mod.rs`, `parse` behind the
feature of that name, `translate`, `derivation` with the read-back and
its checker, `rocq` behind that feature), and `lltp` and `families`
behind `parse`. `lib.rs` re-exports the
public types, so users write `linlog::Sequent`, `linlog::Proof`,
`linlog::prove`, and so on. `hash` is crate-private.

## The public API at a glance

Each entry point is described in the file of its module:
- sequents: `"…".parse::<Sequent>()`, `Display`, serde, `fragment()`,
  `occurrences()`; `Fragment`, `Mode` (`core-sequents.md`);
- `Forest::new(&sequent)` and `Forest::within(&sequent, limit)`
  (`Error::TooManyOccurrences` past the limit), `Reading::new(&forest)`
  or a `ShapeError` (`core-forest.md`);
- `Proof::new(forest, nodes, root)`, `check(mode)`,
  `check_within(mode, memory)` and `CheckError::is_refusal`
  (`core-proofs.md`);
- `derivation()`, `two_sided_derivation()`, `derivation_size(two_sided)`,
  `derivation_with(&view, stop)` under `ViewOptions`, `write_text`;
  `Interactive` (`core-derivations.md`);
- `prove`, `prove_until`, `prove_goal` with `Options`, returning an
  `Outcome` with a `Verdict` (`Proved`, `Unprovable` with a `Refutation`,
  `Unknown` with a `Reason`) and `Statistics`; `Options::engine` forces
  an `Engine`, whose variants describe the engines of the crate-private
  `search::focus`, `search::net` and `search::additive`,
  `Options::pool` names a `search::Pool` kept across searches
  (`core-search.md`, `core-focus.md`, `core-nets.md`, with `parallel`
  `core-parallel.md`);
- `ProofStructure`: `from_links`, `link`/`unlink`, `is_correct()`,
  `sequentialize()`, `from_proof(&proof, mix)` (`core-nets.md`);
- `export::latex`, `typst`, `svg`, `png`, `pdf`, `rocq`, each with one
  options value and `write(&derivation, &options, out, stop)`
  (`core-export.md`);
- `search::batch::run` and `prove`, many sequents under one options
  value (`core-batch.md`);
- `lltp::read` and `families` (`core-inputs.md`);
- `ordinary`: `"…".parse::<ordinary::Sequent>()`, `read_tptp`,
  `translate(&sequent, logic, translation)` to an `Image`,
  `Image::linear_derivation` and `read_back` to a `Derivation` of LK or
  LJ, `Derivation::check`, `export::rocq::ordinary` (`core-ordinary.md`).

## Crate-wide rules

**The rustdoc is the library's manual**, published from `main`. What a
user of the crate needs and the code keeps private is said on a public
item: the text syntax on `Sequent` and the ordinary one on
`ordinary::Sequent` (the parsers are private), every JSON
form on its type (`Sequent`, `Proof`, `Outcome`, `ProofStructure`,
`Interactive`; `serialize` is private), each engine on its `Engine`
variant (the engine modules are private), the features in the crate
docs, and "Needs the cargo feature" on every feature-gated public module
and item (`doc(cfg)` is unstable on the pinned toolchain). A change to
any of these changes those docs in the same commit. No public module is
left without public items, and `RUSTFLAGS="-W unreachable_pub -W
unnameable_types" cargo check -p linlog --all-features` stays clean: a
`pub` item is reachable from the root, and a type in a public signature
can be named.

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
