---
paths:
  - "core/src/occurrences/**"
---

# linlog core: the occurrence forest and its intuitionistic reading

Loaded, beside `core.md`, when a file under `core/src/occurrences/` is
read.

## The occurrence forest

`Forest` (`occurrences/mod.rs`) is what every engine, checker and net works
on; a sequent inside a search is an `OccSet` of occurrence ids, never a list
of terms. Invariants the code relies on:

- **Numbering is DFS preorder**: roots in `Sequent::roots` order, and under
  a binary connective the left subterm before the right one. Hence a root
  precedes its subtree, `left(o) == o + 1`, `right(o)` follows the left
  subtree, and `subtree(o)` is the id range `o .. o + size(o)`; `is_below`
  is two comparisons. Renderers (proof nets, SVG) depend on this child order.
  The numbering is a pure function of the sequent, so ids are stable across
  runs and serializations of the same sequent; never memoize across forests.
- Two occurrences of one arena term (a shared subterm, or repeated roots)
  are distinct ids with the same `TermId`.
- `parent` is stored raw as `u32` with `u32::MAX` for a root; the accessor
  returns `Option<OccId>`. That is why no forest has `u32::MAX` or more
  occurrences (`Forest::MOST`, crate-private, is the most).
- **A forest is refused before it is built** when the sequent unfolds to
  more occurrences than a limit (`Refusal::Occurrences { occurrences,
  limit }`): `Forest::new` within
  `Forest::DEFAULT_LIMIT` (50 million: the largest problem of the LLTP
  library has 27.8 million, and a forest takes about 25 bytes per
  occurrence), `Forest::within(&sequent, limit)` within another, which
  `Forest::MOST` caps. `Sequent::occurrences()` is the count: one pass
  over the arena (`Sequent::sizes`, the occurrences below every term),
  saturating, since an arena that shares its subterms unfolds
  exponentially: a JSON sequent of a few hundred bytes whose subterm is
  shared 25 times over has 67 million occurrences, a forest of 1.9 GiB.
  The check comes before the sequent is cloned or any per-occurrence
  array is reserved. Every path that makes a forest of a sequent it was
  handed (the search's front door, the proofs, nets and interactive
  states read from JSON) goes through one of the two, so the default
  limit holds wherever no caller names another.
- **The forest has no atom bias**: which literal of an atom is positive
  is the focused engine's choice (`search::Bias`, `focus/bias.rs`,
  `core-focus.md`), and `Kind::polarity` is the fixed polarity of a
  connective only. A literal's polarity is never read off the forest.
- Literal lists are one `Box<[OccId]>` in CSR layout, grouped by atom, then
  sign (`Atom` first), ascending ids within a group; `literals(atom, sign)`
  slices it. `all_literals()` is the whole thing.
- The forest owns a clone of its `Sequent` so that `formula(o)` can print.
  Everything else per occurrence is a `Box<[u32]>` or narrower; keep it that
  way (no per-occurrence heap objects, no strings).
- `lca` is a parent walk from the first argument and is `None` across roots;
  the net search's cycle rejection is only valid within one root.

`OccSet` (`occurrences/set.rs`, crate-private with `Forest::empty_set` and
`root_set`: no public call takes a set) is `Box<[u64]>` with the forest's width fixed
at creation (`Forest::empty_set`, `root_set`, `OccSet::empty(len)`). Its
binary operations are defined on sets of different widths, in every build,
as on the sets of ids they are: the words a narrower set lacks count as
empty (`is_subset` is false when the receiver has a member beyond the
other's width), a set that changes keeps its width, and the one result a
width cannot hold, a union with a member beyond it, panics rather than
lose the member; `&a | &b` is as wide as the wider. Between sets of one
forest they cost what they did but a comparison of two lengths. Combining
sets of different forests is still a bug nothing catches: the ids mean
different occurrences. Equality and `Hash` are over the words, the latter
through the crate's `hash::HashMap` (foldhash with a fixed seed: reproducible
runs, no OS randomness, works on wasm).

## The intuitionistic reading

`occurrences/reading.rs` reads a one-sided sequent as a two-sided
intuitionistic one: `Reading::new(&forest)` gives every occurrence a
`Side`, `Input` (a hypothesis, or the antecedent of a goal) or
`Output` (the goal, or the antecedent of a hypothesis), and names the
`goal` root, or fails with a `ShapeError` (`describe(&forest)` for
formulas). This is Lamarche's polarization, and what the two-sided engine,
the checker, the two-sided derivation and the future essential nets read.

- **The grammar.** In output position `⊗ ⊕ & ! 1 ⊤ 0`, atoms `a`, and
  `A ⊸ B` stored as `A⊥ ⅋ B`; in input position the duals: `⅋ & ⊕ ? ⊥ 0
  ⊤`, `~a`, and `A ⊗ B⊥` for a hypothesis `A ⊸ B`. The position flips at
  the antecedent of an implication (an output `⅋` or an input `⊗`) and
  nowhere else. `Reading::implication(o)` returns (antecedent, consequent)
  for exactly those occurrences, the antecedent always the left factor,
  where the lowering of `A ⊸ B` puts it; `formula(o)` prints an occurrence as the
  intuitionistic formula its position makes it (`⊥` as `1`, an input `⊤`
  as `0`, an input `⊗` as `⊸`); `Display` prints `Γ ⊢ A`.
- **The reading is what was written, and guesses nothing.** A bottom-up
  pass computes which positions each occurrence can take (`⊤` and `0`
  both, an atom output only, an implication only with its left factor as
  the antecedent, and so on); the first occurrence with neither, in
  descending id order, is `ShapeError::Formula` (a minimal offending
  subformula), so `b ⅋ ~a` has no reading in output position. With the
  sides known (`Sequent::antecedents`) the roots after the first
  `antecedents` are the succedents: anything but one is
  `ShapeError::Succedents` (whose message says to write the hypotheses
  left of `⊢`), a succedent that cannot be output `NoGoal`, an
  antecedent that cannot be input `Hypothesis { index }`. With the sides
  unknown the goal is the one root that cannot be input (two:
  `SeveralGoals`), else the one root that can be output (none: `NoGoal`;
  two, which only `⊤`- and `0`-built formulas allow, as in `⊢ 0, ⊤`:
  `Undetermined`, which asks for the sides). `⊢ ⊤, a` has one reading,
  `0 ⊢ a`. The guessed goal (the last root that could be one) and the
  symmetric implication (the right factor as the antecedent when the left
  did not fit) answered `|- top, a` and `(A -o bot) -o bot |- A` as
  provable (the held-back audit's H10 and H9); `the_written_sides_decide`
  pins both refusals, with the sides and without. Engine, checker and
  view all call `Reading::new` on the same forest, which is what keeps
  them consistent; never hand one of them a reading of a different
  forest.
- **`Fragment::name_in(mode)`** is the mode-aware name (`IMLL`, `IMLL with
  units`, `IALL`, `IMALL`, `IMELL`, `ILL`; the classical `Display` is
  unchanged), and the JSON of an `Outcome` uses it; a `Fragment` reads
  back from either spelling.

## Decisions

The author's answers for the release (`plan/notes/api.md` §14), which
the fixes implement and later rounds judge against. Where a bullet above
still describes code that a decision changes, the decision holds, and
the commit that lands it rewrites that bullet.

- **With the sides known, the reading is what was written**: the roots
  after the first `antecedents` are the succedents, of which
  intuitionistic mode takes exactly one; every antecedent reads as input;
  an implication's antecedent is its left factor, where the lowering of
  `A ⊸ B` puts it. The guessed goal and the symmetric reading answered
  `|- top, a` and `(A -o bot) -o bot |- A` as provable.
- **With the sides unknown, the reading answers only where it is the one
  reading**: the goal is the one root that can be it, and an
  implication's antecedent is the left factor. Where two roots can each
  be the goal, it refuses (`ShapeError::Undetermined`) and asks for the
  sides. A guess there kept the same wrong answers reachable through JSON
  and the library.
