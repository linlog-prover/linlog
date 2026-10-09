---
paths:
  - "core/src/proofs/derivation.rs"
  - "core/src/proofs/size.rs"
  - "core/src/proofs/fmt.rs"
  - "core/src/proofs/multiset.rs"
  - "core/src/proofs/interactive.rs"
---

# linlog core: the derivation view and interactive proving

Loaded, beside `core.md`, when the derivation view, its size estimate,
the text tree or interactive proving is read. What a derivation knows of
a proof comes from the checker's pass (`core-proofs.md`, "The checker").

## The derivation view

`proofs/derivation.rs` unfolds a checked term into the tree of standard
one-sided inferences (`Inference`, non-exhaustive with `pub(crate)`
fields and the accessors `sequent()`, `rule()`, `principal()`,
`premises()`, `times()`; premises before conclusions, root last, the rule
a `Named` with the usual spellings). The sequent is members (`Member`)
in ascending order with repeats, which the writers read through
`Member::occ` where they print (`fmt::write_sequent`,
`Notation::sequent`), and the builder makes from its multisets
(`Multiset::into_members`); `principal` is a position in it, `None` for
`ax` (its sequent is the two literals) and Mix. `Derivation::occurrence`
and `formula` read a member, as `Proof`'s and `Interactive`'s do.
`ProofStructure::from_proof` reads the axiom links off the `Ax` nodes.

`Derivation::two_sided` (crate-private, like `Derivation::new`: the
public door is `Proof::derivation_within`, which picks one by the
view's `sides`; `Sides::Two`, or `Sides::Auto` on a proof found in
intuitionistic mode) is the same tree
read two-sided: it checks the proof in intuitionistic affine mode (so
`wk` shows where used), keeps the `Reading` (`Derivation::reading`), names
each rule by the position of its principal formula
(`Rule::on(side)`, a `Named` rule: `⊗` on a hypothesis is `⊸L`, `⅋` on one `⊗L`,
`⊕₁` on one `&L₁`, `&` on one `⊕L`, `⊥` is `1L`, an input `⊤` is `0L`, a
dereliction `!L`, a promotion `!R`, `?c`/`?w` are `!c`/`!w`; the axiom and
`wk` keep their names), and the renderer prints `Γ ⊢ A` with the
hypotheses in id order. The one place the reading changes the tree: at a
`⊗` where both premises absorb, the goal among the absorbed formulas goes
to the premise that has none (the classical rule gives everything to the
left one), which is what makes the instantiation an ILL derivation. The
`Inference` sequents are the same ids as one-sided; only the rule names
and the rendering differ.

The dyadic-to-standard translation is *not* Andreoli's (which contracts all
of `Θ` at every `⊗` and weakens all of it at every leaf): the standard
sequent of a subproof is `⊢ ?Θ, Γ` for the least `Θ` the checker derived,
so structural rules appear only where needed:
- `Copy` is `?d`, plus `?c` below it when the formula is used again above.
- `Quest` is no inference when its formula is used above (the sequents
  coincide) and `?w` otherwise.
- `⊗` and Mix contract, below the rule, the `?` formulas both premises use.
- `&` weakens, above a premise, the `?` formulas only the other premise
  uses, unless a `⊤` in that premise absorbs them.
- Whatever a `⊤` absorbs flows down to it through every rule; a `⊗` split
  gives the absorbed part to the absorbing premise.
The builder walks the tree on a stack of its own (`Task`: a subproof to
unfold, an inference to conclude from the subtrees finished last, the
weakenings above a `&` premise, the contractions below a `⊗`), so a
derivation of any height is built on a call stack of any size
(`any_height_on_a_small_stack`: 120 000 inferences on 256 KiB); a DAG
with heavy sharing unfolds to a tree exponentially larger than the
arena. The order of the inferences is the recursion's: the left
premise's subtree, the right one's, the rule, then its structural rules.
`Derivation::new` checks the term first (any mode)
and fails as the checker would. What the builder knows of the term comes
from the checker's pass through an observer (`Record`): a flag per node
for `absorbs` and for `used` (a `?` step whose formula a copy above uses,
a copy whose occurrence is copied again above), the shared unrestricted
occurrences of a `⊗` or Mix, and the standard sequent `?Θ, Γ` only where
the builder splits or pads a context, which is at every `⊗` and Mix and
at the premises of `⊗`, Mix and `&`. Those sequents appear in the
derivation anyway, so the record is never larger than what is built; a
table of every node's sequent would be, by any factor, on a chain of `?`
steps (which are no inferences).

**The size of a derivation is computed from the term, without building
it** (`proofs/size.rs`, `Proof::derivation_size(&view, &limits, stop)`,
a `Size` of the derivation the view's sides choose): one pass of the checker with an observer that keeps a few numbers
per node, all saturating, since a term with shared subproofs unfolds
exponentially. Every sum, product and difference there saturates;
nothing is ever taken away from a count of inferences, of characters or
of a height on its way to the root, and a node's characters are never
fewer than its sequent's weight, so a number that reached `u64::MAX`
anywhere shows in the `Size` returned, whose `bytes()` is then over
every bound. What `Size` promises: `inferences` and `height` (the
inferences on the longest branch; the text tree has two lines for each)
are exact; `characters` is the sum over the inferences of their sequents
written one-sided, each formula with two characters for its separator,
exact when `exact` is set and an upper bound otherwise; `width` is the
widest sequent the pass saw, a *lower* bound of the text tree's width
(the tree's own width needs the layout: `Derivation::text_size`). The
one place the sum is a bound: a `&` with several `?` formulas that only
one premise uses, whose weakenings above the other premise are each
counted with the sequent of the last (their order is not kept). How it
is exact elsewhere, which is what a change to the builder must keep in
step: per node the inferences and characters of its subtree under the
sequent it derives itself, and `reached`/`reached_goal`, the number of
its inferences whose sequent holds a formula that a `⊤` in the subtree
absorbs (a hypothesis, or two-sided the goal, which the builder sends to
the premise without one at a `⊗` whose premises both absorb: the right
one, since the reading takes `⅋` only as `~A ⅋ B`, so a `⊗`'s left
factor is in output position and its premise has a goal). An
absorbed formula adds its characters times that number: at the root for
what the conclusion holds beyond the root's sequent, at every rule for a
subformula its premise lacked (`Facts::absent`), at a `&` for what the
conclusion's context holds beyond an absorbing premise's. A `⊗` or Mix
with `k` shared `?` formulas is `k + 1` inferences, the rule's sequent
holding each formula twice and each contraction below taking one away in
ascending order. `is_the_size_of_the_derivation_built` compares all of
it with the derivation actually built, on the samples of the checker's
differential test, one-sided and two-sided. `Size::bytes()` turns the
count into the estimate that a bound is compared with
(`BYTES_PER_CHARACTER` 8, `BYTES_PER_INFERENCE` 128): measured on a
derivation of 5 119 inferences and 17.0 million characters (`wide-m1` at
1 024), the text tree is 76 MB written and 160 MB at its peak, LaTeX 49
and 106, Typst 59 and 126, the Rocq script 62 and 132, the SVG 212 MB
written and 855 MB at its peak, against an estimate of 130 MiB: the
order of magnitude for every format but the SVG's memory.

**Nothing builds a derivation it was not allowed to** (`ViewOptions`,
the one options value of every path that builds one, plain data with
serde, a field absent from the JSON taking its default: `compact`,
above; and the `Limits` beside it: `derivation_bytes`, the most bytes
of `Size::bytes()` a derivation may be estimated at,
`DEFAULT_DERIVATION_BYTES` 64 MiB, `None` for no bound; and
`memory_bytes`, the most bytes the making of one may hold, `None` for
no bound, which bounds every pass of the checker on the way and the
derivation by the same estimate). `unfold`
is the one place derivations are made, for `Derivation::new`,
`two_sided` and `of_goal` alike: the size first, always (a pass of the
checker); `Error::Refused(Refusal::Output { estimate_bytes, limit_bytes,
least_bytes })` past `derivation_bytes` (`least_bytes` the compact view's lower
bound, `size::Firm`, where the view may compact), else
`Refusal::Memory { phase: View, needed_bytes: Some(estimate) }` past
`memory_bytes`, else `Refusal::Index { what: Space::Inference }` for more
inferences than an `InfId` counts
(`Derivation::MOST`, which only a call with both bounds lifted can
reach), each with nothing built; then the pass that records what the
builder reads, then the builder, which polls the caller's `stop` once
per node and answers `Refusal::Stopped { phase: View }`. A pass that the
checker gives up for its memory is the checker's own
`Error::Check(CheckError::Refused(_))`, never
`Invalid`: `From<CheckError>` sees to it. The record's sequents count
against its pass (`Record::held`), though each is in the derivation
anyway, so that the pass and its record together stay within the bound.
Every one of them is a refusal (`Error::is_refusal`), never a fault of
the proof. So the text tree, the four exports
(which take a `Derivation`), the graft of `Interactive::close` and a
front end's check output are all under the bound by construction, and a
new path that needs a derivation gets it from there or not at all.
`Proof::derivation()` is the default view (`Sides::Auto`: two-sided
exactly for a proof whose recorded mode is intuitionistic) and the
default options with no stop; `…_within(&view, &limits, stop)` take all three. A proof whose
derivation is refused for its size has passed the checker (the size's
pass is one). `Interactive::close(goal, options, view, limits, stop)` leaves a
goal open whose graft is refused (`Error::Refused`), though the search
proved it; what it then tells the user is the front end's to say. The
builder needs no stack to speak of, so a front end on a small one (the
web) builds what the bounds allow; `Size::height` is what it asks to
know whether a tree fits a view.

**The compact view** (`ViewOptions::compact`, set with `with_compact`, `Compact::Auto` by
default, `Always`, `Never`; `Inference::times`, how many applications of
its rule an inference stands for). What the code relies on:
- **A run is merged where it is made**: in `Build::infer`, a structural
  rule (`Rule::is_structural`: `?w`, `?c`, `wk`, `!w`, `!c`, compared
  after the intuitionistic renaming) whose one premise has the same rule
  replaces that premise in place and adds to its `times`. The premise of
  a rule with one premise is the subtree finished last, so it is the last
  inference (debug-asserted), and the order "premises before
  conclusions, root last" holds. The principal is the lowest
  application's. The weakenings above a `&` premise and the contractions
  below a `⊗` become one inference without a sequent per step
  (`Multiset::sum`, `difference`), and a chain of `?` steps that weaken
  (through the `?` steps whose formula is used above, which are no
  inference) is walked in one go (`weaken_run`): a sequent per node of a
  chain of 65 000 `?` steps over a sequent of 65 000 formulas is 17 GB
  of copying.
- **`Auto` compacts only where the whole derivation is over a bound, and
  only under a bound in bytes.** The size pass also returns what a
  compact view holds at least (`size::Firm`: the inferences that are no
  weakening or contraction and their bytes, which compaction never
  changes); an attempt is refused at once when that is over the bound or
  over `Derivation::MOST`. Otherwise the builder counts what it holds by
  `Size::bytes`'s estimate (`Held`) and gives up at the bound with the
  whole derivation's error, so a failed attempt costs at most the bound.
  Without any bound in bytes `Auto` does not try at all: an attempt on
  `tower(70)` (2⁷⁰ inferences, both bounds lifted, `TooMany` expected)
  had nothing to end it, and the test run took the machine's memory
  until it froze. `Always` without a bound is the caller's explicit
  request. `Never` is the old behaviour exactly.
- **Never compact**: a graft (`of_goal` forces `Never`: the session reads
  it rule by rule, and `proof()` would translate a run wrongly), the
  interactive state's own derivation (its inferences are made one rule
  at a time), and a Rocq certificate (`Unsupported::Compact`: a run names
  one formula; the command builds Rocq's derivation with `Never`).
- The label of a run is the rule's label and `*` (`Drawn::label`, `RUN`),
  `write_steps` says `by ?w 3 times`, the text tree's `bars` have a
  second half for runs. On a terminal the command tries the compact tree
  when the whole one does not fit (`--compact auto`), and lays out
  whichever it builds before it decides.

`Rule::Open` is the rule of an open goal in the derivation of a proof in
progress (below) and appears nowhere else. **A rule is one-sided, and a
derivation names it with a side** (`proofs/rule.rs`): `Rule` has the
sixteen one-sided rules, `Named { rule, side }` the rule as an inference
names it (`side` `None` one-sided and for the rules whose name has none;
`Named::new` normalizes the side of `1`, `⊥` and the exponential rules,
which have one two-sided name each), so the one-sided rule behind a
two-sided name is the `rule` field and nothing matches on eighteen
two-sided variants. `Named::from_str` reads a rule from its name or an
ASCII spelling (`Error::UnknownName` otherwise), and `Named::ALL` is
every named rule in the order the label tables have.
`Derivation::from_parts` wraps inferences that already have the
derivation's shape, and `Derivation::of_goal` unfolds a proof whose root
concludes a goal rather than the roots (as `prove_goal` returns it) into
inferences, for grafting.

`proofs/fmt.rs` draws the tree: premises side by side, bottom-aligned, three
columns apart; a bar of `─` spanning their conclusions or the conclusion,
whichever is wider, with the rule name after it; the conclusion centred
under the bar; an open goal is its sequent alone, with no bar, which is
how a leaf without a rule is told from a closed one. Widths are character
counts (every symbol used is one column in a monospace font), lines are
trimmed on the right, and there is no trailing newline. The renderings are
pinned in tests, so a layout change is a test change. The layout is two
passes over the inferences in their own order, which needs no walk since
premises precede conclusions (`layout`: box sizes and the premises'
offsets ascending, then absolute columns and depths descending), and
`Display` writes the pieces (a conclusion, a bar with its rule name) row
by row in column order, each sequent written straight into the formatter
and only counted in the first pass. So a tree costs time linear in its
text and memory linear in its inferences; the first renderer built every
subtree as a block of padded lines and copied it at every level (24 s and
333 MiB for a tree of 76 MB). A bar that would start left of its box
moves the premises right (`shift`), exactly as the block renderer padded
them. `Derivation::text_size` is the width and height from the first pass
alone, which is how a front end learns whether a tree fits before it
draws it. The layout, `text_size`, `write_text` and `write_steps` are
free functions of `fmt.rs` generic over `style::Drawn`
(`core-export.md`), and the methods of `proofs::Derivation` and of
`ordinary::Derivation` (`write_text`, `text_size`, `Display`) call
them, so the two trees are one layout. A derivation of LK or LJ has no
open goal and no run (`times` is 1), writes `Γ ⊢ Δ` with both sides in
their stored order, and builds each sequent in a `String` before it is
counted or written (`write_sides`), once per inference and pass.

## Interactive proving

`proofs/interactive.rs` (feature `interactive`) is the state a client
holds for step-by-step proving: the forest, the mode, the inferences of a
derivation of the standard calculus with open goals as leaves, and the
steps taken. It reuses `Inference` (its arena private, numbered by
`InfId` inside) and shares no second representation with anything. Its
API names goals by `GoalId`, the session's own numbering, apart from a
derivation's `InfId`: `new(&sequent, mode)` and `within(&sequent, mode,
&limits)`, `goals()` and `goal(id)` (members), `rules(goal, position)`
(each an `Applicable`: the `Named` rule and what a step of it `Needs`,
nothing or a split; or a `StepError`), `apply(goal, &step)` with a `Step`
(position, rule, `Split`), `split_passes(goal, &step)`, `undo()`,
`close(goal, options, view, limits, stop)` (the search, then the graft,
returning `Closed`: the outcome and whether the graft was refused, the
goal then still open and the proof kept), `close_with(goal, &proof, view,
limits, stop)` (the graft of a proof the caller's own search found, as
the command's race does), `close_all` (a result per goal, F68),
`derivation()` and `derivation_within(view, limits, stop)` (refused past
`limits.derivation_bytes` by the size estimate's measure) with open
goals as `Rule::Open` leaves, `derivation_ids()` (a drawn inference's
goal in the state), `occurrence(m)` and `formula(m)`, and `proof(limits,
stop)`. A Mix that leaves its right premise empty is
`StepError::EmptyPremise`: nothing concludes the empty sequent. What the
code relies on:

- **The arena is top-down.** Inference 0 concludes the sequent; a step
  closes one open goal in place (its rule, principal and premises are
  filled in) and appends the goals it opens at the end, so a premise has a
  larger index than its conclusion, the reverse of `Derivation`'s order;
  `derivation()` renumbers into postorder, so its ids are not the state's.
  Since steps only append, the inferences a step added are a suffix of the
  arena as long as no later step exists, which is why `undo` is "truncate
  to the smallest index in the closed goal's subtree, reopen the goal" and
  why the history is just the list of goals closed, in order. A search
  graft is one step (its whole subtree is the suffix).
- **Positions, not ids, address formulas**: `position` indexes the goal's
  sequent (ascending ids with repeats), as Click & coLLecT's
  `formulaPosition` does; the split of a `⊗` or Mix is the positions of
  the context formulas going left. Equal ids at different positions are
  interchangeable (the sequent is a multiset), so which copy the client
  picks does not matter.
- **Validation at application time is complete for the checker**, so that
  a derivation built through `apply` always translates into a term the
  checker accepts: `expand` checks the connective, the mode (`wk` only
  affine, Mix only with Mix), the context (`ax` exactly the dual, `1`
  alone, `!` with a `?`-only context, since the term's `Bang` needs the
  linear zone empty), the split, and in intuitionistic mode R1 (every
  premise has exactly one occurrence in output position) and R3 (never
  weaken the output). R2 of the checker (an absent output child absorbed
  by a `⊤` only if the premise has no output already) is implied: the
  linear zone the term derives for an inference is a sub-multiset of the
  inference's non-`?` formulas, so where the child is absent the zone's
  outputs are among the sequent's other formulas, of which R1 leaves none
  (`Θ` members are inputs, so a `Copy` never triggers R2 either). A
  fresh-context review confirmed this on about 2 500 random derivations
  in every mode, each translated and checked. In intuitionistic mode a rule is accepted under its
  classical or its two-sided name and recorded under the two-sided one,
  so a two-sided state's inferences look like `Derivation::two_sided`'s.
- **The standard-to-dyadic translation** (`Terms`, in `proof()`) is the
  view's table read upwards: every rule is its node; `?d` is `Copy(A)`;
  `?c` and `?w` are nothing; and every `?` formula gets its `Quest` node
  *where it enters the derivation*: below the rule that introduces it as
  a subformula (`Terms::premise`) or below the root for a `?` root. So a
  `?` formula is in `Θ` from its entry upwards, a contraction's second
  instance is the same `Θ` member, and a weakened instance is simply not
  copied (the unused `Quest` is what the view shows as `?w`). This keeps
  the dyadic linear zone free of `?` formulas above their entry, which is
  what `Bang` needs, and makes every `Copy` sit above its `Quest`, which
  is what the checker's empty root `Θ` needs. The term's own derivation
  view may place structural rules elsewhere than the user did (it
  contracts below a `⊗`, the user contracted above the root, say); the
  state's derivation is the user's tree, the term's is the checker's.
  `proof()` runs the checker on the term, always: the layer is not trusted
  more than an engine. The translation walks the derivation with a stack
  of its own (`Terms::term`: the steps still to take, the terms of the
  subtrees done), in the order a recursion would, so the nodes are those
  it always pushed; a state read from a file is as high as its formulas
  are deep, and a recursion over it overflowed the stack.
- **Search from a goal**: `close` calls `prove_goal` on the goal's
  sequent, and grafts `Derivation::of_goal` of the proof found, within
  the `ViewOptions` it is given and until its stop fires; the goal's
  fragment is its own (`search::goal_fragment`), so the prunes are those
  of the goal, and the net engine never runs off the roots. The outcome
  returned is the search's, its proof the proof of the goal alone.
  `split_passes` lends the client the focused engine's count prunes
  (`focus::split_passes`, which builds the engine's `Rules` and tallies
  for the two sides) as a "this split cannot close" test; a split that
  passes may still fail.
- **Reading a state back** (`from_parts`, used by deserialization) checks
  the shape (root 0 concludes the sequent, every other inference is the
  premise of exactly one earlier one, sequents ascending and within the
  forest, a principal position exactly for the rules that have one),
  replays every closed inference (`replay` recovers the split of a `⊗`
  from the left premise less the subformula, and of a Mix from the left
  premise alone, whose first formula stands for the position; then
  `expand`'s premises must equal the recorded ones) and checks the history
  from the last step back: its entries are distinct closed inferences
  below the arena's length as the step left it (an entry inside a later
  step's subtree passed the suffix test vacuously, and `undo` then indexed
  past the arena), and the inferences a step added, which are those of
  its subtree that exist
  at that point (the later ones belong to later steps), are the suffix of
  the arena then. So a loaded state is as trustworthy as one built through
  the API; the checker at the end is the final word anyway. A history that
  does not cover every closed inference is allowed (those steps are just
  not undoable). A review fed hundreds of API-built states through JSON
  and found the first version of these checks rejecting chains of steps,
  every Mix and every graft (`graft` now appends a found derivation in
  reverse so that premises keep larger indices); keep the round trip of
  such states in `core/tests/serialize.rs`.
- **The reading is kept** (`Interactive::reading`, O(1)): the state
  stores the positions and the goal `Reading::new` computed when it was
  made (`Reading::into_parts`), and lends a reading over them
  (`Reading::of_parts`, whose positions are a `Cow`), since `Reading`
  borrows the forest the state owns. Recomputing it per call was O(n)
  per `rules` or `apply`: 2 000 queries on a sequent of 300 003
  occurrences took 2.1 s.
- **Reading a session back is linear** (`from_parts`): the split of a `⊗`
  or Mix is recovered by two pointers over the sorted conclusion and the
  sorted premise context, and the history is walked from the last step
  back without entering an inference a later step added (a premise has a
  larger index than its conclusion), so every inference is walked by one
  step; `added` inferences below `len` none of which is below `len −
  added` are exactly the suffix the step must have added.
