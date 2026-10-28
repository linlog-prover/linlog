---
paths:
  - "core/src/occurrences/**"
---

# linlog core: the occurrence forest and its intuitionistic reading

Loaded, beside `core.md`, when a file under `core/src/occurrences/` is
read or changed.

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
- **Cut formulas are trees past the conclusion**: a sequent's
  `cuts` (pairs of terms `A`, `A⊥` of one shape) are numbered after the
  roots' trees, `A` then `A⊥` per pair, so `dual(x) = x + size(A)` on the
  `A` tree and the inverse on the `A⊥` tree (`Forest::dual`, a binary
  search of the `A` roots, `cuts`, by `root(x)`). `roots()` stays the
  conclusion; `ids()`, `len()`, the literal lists and `measure` cover the
  cut trees too. Code that means "the conclusion's occurrences" asks
  `is_conclusion`; a pass over every id sees the cut trees.
- **`Sequent::push_cuts` reuses the arena's equal terms** for the duals
  (a hash of the terms, the first index winning) and dualises each term
  once for all the cut formulas it is given (a JSON document's together:
  one at a time cost the arena per cut, quadratic in the document), so a sequent written
  to JSON with its duals in the arena reads back to the same arena
  (`cuts` on the wire names `A` only); `add_cut` appends the formula's
  own terms unshared, as `add` does. `pop_cut` (a session's `undo`)
  truncates the arena and the atom table to what the roots and the other
  cut formulas reach, which is exactly what the last cut appended, but
  an atom no term names (only a hand-written JSON sequent has one) goes
  too.
- `parent` is stored raw as `u32` with `u32::MAX` for a root; the accessor
  returns `Option<OccId>`. That is why no forest has `u32::MAX` or more
  occurrences (`Forest::MOST`, crate-private, is the most).
- **A forest is refused before it is built** when the sequent unfolds to
  more occurrences than a limit (`Refusal::Occurrences { occurrences,
  limit }`): `Forest::new` within
  `Limits::DEFAULT_OCCURRENCES` (50 million: the largest problem of the LLTP
  library has 27.8 million, and a forest takes about 25 bytes per
  occurrence), `Forest::within(&sequent, &limits)` within
  `limits.occurrences`, which
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
- **Axiom partners are one predicate**, `Forest::dual_literals(x, y)` (an
  atom and its negation): the checker, its first implementation, the
  interactive axiom, the additive path and `ProofStructure`'s links ask
  it, so that atoms with arguments change one function (F58); the
  focused engine's lists by `2 · atom + sign` and the Horn engine's sign
  of the bodies are candidates and shapes, not this test. A new site
  that pairs literals asks it.
- The forest owns a clone of its `Sequent` so that `formula(o)` can print.
  Everything else per occurrence is a `Box<[u32]>` or narrower; keep it that
  way (no per-occurrence heap objects, no strings).
- **`Member` names a member of a sequent of an owner** (a proof, a
  derivation, a session, a goal): a `u32` that below the forest's length
  is the occurrence with that id, the same number in every owner, so the
  wire forms do not change. `Node`'s operands, `Dyadic` and `Fault` hold
  members; `Forest`, `Reading` and the engines speak of `OccId`s and
  convert where they build a node (`Member::from`) or read one
  (`Member::occ`, crate-private, the identity while no owner keeps an
  instance table; `Member::occurrence(&forest)` is the public, checked
  form). Code that reads a node's operand converts it once per match
  arm, never at every use.
- `lca` is a parent walk from the first argument and is `None` across roots;
  the net search's cycle rejection is only valid within one root.

`OccSet` (`occurrences/set.rs`, crate-private with `Forest::empty_set`:
no public call takes a set) is `Box<[u64]>` with the forest's width fixed
at creation (`Forest::empty_set`, `OccSet::empty(len)`). Its
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
- **The ordered reading of the Lambek calculus** (`Reading::of_mode(forest,
  mode)`, `is_ordered`; `new` is the commutative one, `of_mode` in a
  commutative mode is `new`): read on a sequent of the reversing parse,
  an implication's antecedent is the factor in the other position,
  either one (`A⊥ ⅋ B` is `A \ B`, `B ⅋ A⊥` is `B / A`; in input position
  `B⊥ ⊗ A` the hypothesis `A \ B` and `A ⊗ B⊥` the hypothesis `B / A`);
  in input position the operands of `⊗` and `⅋` are written right one
  first (`operands`), since the dual turned them round; `connective` is
  `IllConnective::Under` or `Over` by which written operand is the
  antecedent; `hypotheses` and the crate-private `written` (the order a
  two-sided sequent prints its members in, which `Notation::sequent` and
  the SVG's positions follow) go round from the goal backwards. In
  unit-free MLL every occurrence has one position, so taking either
  factor guesses nothing there; with units it could (H9's `(A -o bot) -o
  bot |- A`), which no Lambek path decides: the front door, the session
  and the checker refuse units in an ordered mode. Without empty
  antecedents a root sequent with no hypothesis is
  `ShapeError::EmptyAntecedent`. `ProofStructure::reading` gives a
  cyclic structure the ordered reading with empty antecedents allowed.
  A session keeps its positions and rebuilds the reading with
  `of_parts(…, ordered)`.
- **`Fragment::name_in(mode)`** is the mode-aware name (`IMLL`, `IMLL with
  units`, `IALL`, `IMALL`, `IMELL`, `ILL`; the classical `Display` is
  unchanged), and the JSON of an `Outcome` uses it; a `Fragment` reads
  back from either spelling.

## The intuitionistic sequent as written

`occurrences/ill.rs`: `IllSequent` is a sequent of ILL as its author
wrote it (hypotheses in the order written, exactly one goal), held as
the lowered forest, the side of every occurrence and the goal.

- **Three ways in**: `IllSequent::parse_within` (strict text), the JSON
  form (`serialize/ill.rs`, wire level 5) and `read_one_sided`, the
  explicit reading of a one-sided sequent, which is `Reading::new` and
  refuses what it refuses. The first two lower through `lowered`.
- **The sides are recorded, not inferred**: `lowered` gives the roots
  their written sides and passes each down, flipping at the left factor
  of a `⅋` on the goal's side or a `⊗` on the hypotheses' side. That is
  exact because typed input writes no `⅋`, so those are exactly the
  written `⊸` (whose lowering puts the antecedent left), and a `⊤` or
  `0` takes its root's side, never a guess. A `debug_assert` and the
  test `the_recorded_sides_are_the_readings` (generated sequents and the
  units on either side) pin that the recorded sides are
  `Reading::new`'s, on which the checker's one-succedent condition
  rests; the engines, the checker and the views keep calling
  `Reading::new` on the lowered forest (the front door takes a
  `Sequent`), and the test is what makes them one reading.
- **The strict parser leaves the parser alone**: it runs the
  commutative parser and `parse::classical`, a scan of the text for a
  classical token (`~ ^ ⅋ ? ⊥`, `|` not before `-`, the identifiers
  `par` and `bot`, every identifier skipped whole), which no state of
  the parser is needed for, since none of those tokens is ILL syntax
  anywhere; a parse error before the first classical token wins. The
  hint is `ParseError::hint`. A guard in the parser's arms would have
  cost the `read-text` journey on every text.
- **The scan must cut tokens where the parser does**: `|-` and `-o` are
  one token each and an identifier starts only where the parser starts
  one, else `A -obot` (the parser's `-o` then `bot`) passes as the atom
  `obot`, and the recorded sides then describe another sequent than the
  forest (a reviewer's finding; `debug_assert` in `lowered` catches it in
  a debug build). It scans the whole text and reports only a token before
  the parser's error, so that a cut text does not split a `|-`. A new
  token of the parser is a new case of the scan.
- **Equality is the written sequent's**: occurrence by occurrence
  (kind, atom name, side), so a parsed sequent and its JSON, whose
  arenas share differently, are equal.
- Commutative only: the Lambek calculus keeps `parse_ordered` and
  `Reading::of_mode`.
- `IllSyntax` (`strict`, `one-sided`) in `InputOptions`
  (`Settings::input`) is how a front end reads an intuitionistic
  sequent's text; the command's `--ill-syntax` and the bindings' settings
  set it.
- **Raising**: every printer of an intuitionistic sequent reads a
  `Reading` (`Reading::formula`, `Display`, `Derivation::two_sided`). An
  error report (`Described`) reads the reading of its owner's mode
  (the sealed `Owner::mode`: a proof's recorded mode, a two-sided
  derivation, a session) or of `Described::in_mode`, and `Names`
  (`errors/describe.rs`) carries it to every writer that names an
  occurrence. The property tests
  `no_intuitionistic_output_writes_a_classical_symbol` (library and
  command) and the bindings' one pin that nothing one-sided leaks; a new
  output joins them.
- **A cut formula's sides are recorded where the cut is made**
  (`Interactive::cut`): the `CutFormula` read from the typed text keeps
  the sides `lowered` gave its formula as a goal, the `A` tree takes them
  and the `A⊥` tree their flips (the flips sit at the same places, a
  `⅋` on the output side and a `⊗` on the input side being duals);
  `Reading::new` reads the pair the same way (`A` output, `A⊥` input,
  `ShapeError::CutFormula` otherwise), which a `debug_assert` in `cut`
  and `the_recorded_sides_are_the_readings` pin. A classical session's
  record (`Written`) takes the typed formula's record on the `A` tree and
  every side flipped on the `A⊥` tree (the same text written left of
  `⊢`); `written::context` puts an `A` root right of `⊢` and an `A⊥` root
  left of it, and `written::one_sided` records the pair so.
- **Where later steps plug in**: an instance's
  substitution enters the raising at `Notation::write_atom`, the one
  place an atom is written (a predicate's arguments through
  `FirstOrder::write`, a bound variable by the name of its binder in the
  walk's `Scope`, renamed with `_` after it where a symbol or an
  enclosing binder has the name). The quantifiers are
  `IllConnective::Forall` and `Exists` (an output `∀` or an input `∃`,
  and the reverse), their body on their own side as `!`'s is, and the
  JSON form's tags `∀` and `∃`, the names in `binders`
  (`core-sequents.md`, level 7).

## The classical sequent as written

`occurrences/written.rs`: `ClassicalSequent` is the lowered forest and,
per occurrence, a `Written` (one byte: its side, and whether it is an
implication), which is what the lowering merges and position alone
cannot tell in classical syntax.

- **The side is the parity the parser's negation pass leaves**: after
  `dualise`, `Parser::negated[t]` holds whether term `t` was dualised
  (every `~`, `^`, `⊸` antecedent and left-of-`⊢` above it counted), and
  that is `Input`; an implication is a term made by `Binary::Lollipop`
  (`Parser::implications`, filled only while recording). Before
  `optimize` every term has one parent, so the terms in preorder from
  the roots in text order are the forest's occurrences in its numbering;
  `Parser::written` walks them so. Nothing is inferred from the
  one-sided form.
- **A negation is where a side differs from its context**: `context(o)`
  is a root's side of `⊢` (by `antecedents`), the other side for an
  implication's left factor, the parent's side otherwise; `negated(o)`
  compares. Double negations record nothing, so printing gives the text
  back up to them and to brackets. An atom is always `Output`, a negated
  atom `Input` (`fault` refuses another record: the session reader
  calls it).
- **The printer is `Notation::written`** (one walk over `Stops`): the
  connective is the one-sided kind on the output side and its dual on
  the input side, `⊸` for an implication, a negation as the target's
  `dual_prefix`/`dual` around the subformula, in brackets for a binary
  one and, where the negation is postfix, for `!`/`?`. `Notation::classical`
  writes `left ⊢ right` with each formula's negation shown or not, for
  the sequent's `Display` and for `TwoSided`'s inferences.
- **`read_one_sided`** is the record of the one-sided syntax: every
  formula on its root's side (`antecedents`), a literal's side its sign's;
  a JSON, LLTP or `.spec` sequent is drawn two-sided through it.
- The round trip (`a_written_sequent_prints_back_to_itself`) writes
  generated sequents at random through `generate::written`, which also
  says the record the parser must make, and checks the record, the
  lowering and print-parse equality.

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
