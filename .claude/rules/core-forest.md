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
  more occurrences than a limit (`Error::TooManyOccurrences { occurrences,
  limit }`): `Forest::new` and `TryFrom<Sequent>` within
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
- **Atom bias** (`Forest::bias`, computed once in `Forest::new`, a
  function of the sequent alone, so a run stays deterministic; the
  focused engine is its only reader, through `Forest::bias_under`).
  Focusing is
  complete for every assignment of polarities to atoms, so the bias is
  chosen for speed and can never change what is provable. Without an
  exponential in the sequent: per atom, the literal that is more often a
  direct factor of a `⊗` is positive, each occurrence weighted by ½ per
  `&` or `⊕` above it (a proof takes one side of a choice, so the two
  heads `~d` of a clause `(… ⊗ ~d) ⊕ (… ⊗ ~d)` count as one); a `⊗`
  with a positive literal factor has its split forced. On a tie, and
  whenever the sequent has a `!` or `?`, the old rule: the literal with
  fewer occurrences is positive, a tie makes `Var` positive, so an atom
  with one sign only has all its literals negative. The engine takes
  the old rule in affine mode as well (`focus::plan`): nothing forces a
  split there, so the factors have nothing to say, and a review measured
  up to 700 times the stable sequents on generated affine sequents with
  the factor rule. With exponentials the bias decides the shape of the
  focused proofs, hence the copies a branch needs, and neither rule
  wins: the counter family chains forward under the factor rule and
  needs `n − 1` copies on its one branch where the rarer-literal rule
  needs `log₂ n`, and at a bound of `n − 1` the factor rule decides the
  counter with 16 tokens in 19 stable sequents instead of 473 232. So
  `Forest::bias` is the rarer-literal rule there, and the engine's
  default runs a search under each ("The default bias with exponentials
  is two searches", below). `Options::bias` names the rules:
  `Bias::Rarer` and `Bias::Factors` for any sequent
  (`Forest::bias_under`; the engine reads its polarities off
  `Counts::positive`, never off `Forest::polarity`, so that the option
  reaches every place a literal's polarity matters, `literal_tensor`
  included). Rules tried on the
  exponential-free targets and not taken: `Var` always (as good on the
  families written two-sided, where it is forward chaining, but it
  depends on how the atoms happen to be written and loses the gains on
  Partition and the wide sequents), `DualVar` always (30 to 300 times
  more stable sequents on the Horn families), the factor count without
  the ½ (the heads of the 3-Partition clauses outvote the goal: 317 138
  stable sequents against 923 at bins of four). Measured, old rule and
  new (stable sequents, splits): unsolvable 3-Partition with bins of
  five 3 373 and 41 160 against 971 and 36 072; QBF 20 #2 105 667 and
  1 037 858 against 60 883 and 97 394; Partition with seven items 179
  and 6 008 against 178 and 2 078; `wide-m3` at 30 91 and 768 against 31
  and 688.
- Literal lists are one `Box<[OccId]>` in CSR layout, grouped by atom, then
  sign (`Var` first), ascending ids within a group; `literals(atom, sign)`
  slices it. `all_literals()` is the whole thing.
- The forest owns a clone of its `Sequent` so that `formula(o)` can print.
  Everything else per occurrence is a `Box<[u32]>` or narrower; keep it that
  way (no per-occurrence heap objects, no strings).
- `lca` is a parent walk from the first argument and is `None` across roots;
  the net search's cycle rejection is only valid within one root.

`OccSet` (`occurrences/set.rs`) is `Box<[u64]>` with the forest's width fixed
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
`Position`, `Input` (a hypothesis, or the antecedent of a goal) or
`Output` (the goal, or the antecedent of a hypothesis), and names the
`goal` root, or fails with a `ShapeError` (`describe(&forest)` for
formulas). This is Lamarche's polarization, and what the two-sided engine,
the checker, the two-sided derivation and the future essential nets read.

- **The grammar.** In output position `⊗ ⊕ & ! 1 ⊤ 0`, atoms `a`, and
  `A ⊸ B` stored as `A⊥ ⅋ B`; in input position the duals: `⅋ & ⊕ ? ⊥ 0
  ⊤`, `~a`, and `A ⊗ B⊥` for a hypothesis `A ⊸ B`. The position flips at
  the antecedent of an implication (an output `⅋` or an input `⊗`) and
  nowhere else. `Reading::implication(o)` returns (antecedent, consequent)
  for exactly those occurrences; `formula(o)` prints an occurrence as the
  intuitionistic formula its position makes it (`⊥` as `1`, an input `⊤`
  as `0`, an input `⊗` as `⊸`); `Display` prints `Γ ⊢ A`.
- **The choices, made deterministically.** A bottom-up pass computes which
  positions each occurrence can take (`⊤` and `0` both, `Var` output
  only, and so on); the first occurrence with neither, in descending id
  order, is `ShapeError::Formula` (a minimal offending subformula). The
  goal is the root that can only be output (two such roots:
  `SeveralGoals`; none that can be output: `NoGoal`), else the *last*
  root, by id, that can be output. Inside an implication the left factor
  is the antecedent when that reading works and the right one otherwise,
  so `b ⅋ ~a` reads as `a ⊸ b` too (the symmetric reading).
- **Ambiguity is real and cannot be resolved from the arena.** Only
  formulas built from `⊤` and `0` alone can stand on either side, and for
  those the written succedent is lost: `Sequent::optimize` sorts the roots
  by term and hash-conses, so a `⊤`-built succedent equal to a hypothesis
  subterm gets an early id and another root becomes the goal (`0, ⊤ ⊢ ⊤`
  prints as `0, 0 ⊢ 0`). A review brute-forced 607 464 such sequents and
  found no pair of readings that differ in provability, so the verdict is
  unaffected; only the two-sided print and derivation show the other
  reading. Engine, checker and view all call `Reading::new` on the same
  forest, which is what keeps them consistent; never hand one of them a
  reading of a different forest.
- **`Fragment::name_in(mode)`** is the mode-aware name (`IMLL`, `IMLL with
  units`, `IALL`, `IMALL`, `IMELL`, `ILL`; the classical `Display` is
  unchanged), and the JSON of an `Outcome` uses it; a `Fragment` reads
  back from either spelling.
