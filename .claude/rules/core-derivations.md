---
paths:
  - "core/src/proofs/derivation/**"
  - "core/src/proofs/size.rs"
  - "core/src/proofs/fmt.rs"
  - "core/src/proofs/multiset.rs"
  - "core/src/proofs/interactive/**"
---

# linlog core: the derivation view and interactive proving

Loaded, beside `core.md`, when the derivation view, its size estimate,
the text tree or interactive proving is read or changed. What a derivation knows of
a proof comes from the checker's pass (`core-proofs.md`, "The checker").

## The derivation view

`proofs/derivation/` (`view.rs` the options, `Inference` and
`Derivation`, `build.rs` the translation and what it holds, `mod.rs`
the tests) unfolds a checked term into the tree of standard
one-sided inferences (`Inference`, non-exhaustive with `pub(crate)`
fields and the accessors `sequent()`, `rule()`, `principal()`,
`premises()`, `times()`; premises before conclusions, root last, the rule
a `Named` with the usual spellings). The sequent is members (`Member`)
in ascending order with repeats, which the writers read through
`Member::occ` where they print (`fmt::write_sequent`,
`Notation::sequent`), and the builder makes from its multisets
(`Multiset::into_members`); `principal` is a position in it, `None` for
`ax` (its sequent is the two literals) and Mix. `Derivation::occurrence`
and `formula` read a member, as `Proof`'s and `Interactive`'s do;
`formula` writes it under the reading of a two-sided derivation (a
session in an intuitionistic mode, a proof that records one, the last
recomputing the reading per call), else one-sided, so that no
intuitionistic formula leaves the library one-sided.
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

**A session starts from its sequent's conclusion** (`Interactive::within`
pops a sequent's cut formulas, `Sequent::pop_cut`): a session's cut
formulas must be those of its cuts in the order made, which its reader
checks, so a session started on a sequent with cut formulas could not be
read back. A front end replays a session's cut by typing its formula
again (`Interactive::formula_as_written` for a classical record, the
reading's text intuitionistically), which adds the same terms.

**The node of an inference** (`Inference::node`): `Build` sets it on
every inference a node unfolds into (`Task::Infer` carries it, `leaf`
sets it), and leaves `None` on what the view adds below a node (the
weakenings of `pad`, the contractions of `Task::Contract`); a run of a
compact view takes its lowest application's, as its principal. A
session's derivation has none: `of_goal` clears the grafts' (they name
nodes of a proof the session does not keep) and
`Interactive::derivation_within` writes `None`. The web bindings
highlight a cut by it, so a change that unfolds a cut into another
inference must keep the cut's node on it
(`every_inference_names_its_node`). `Inference`'s equality leaves the
node out (a hand-written `PartialEq`), as 0.1.0's derived one had no
node to compare: the same step from terms numbered apart, or from a
session, is one inference.

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
  (`Multiset::sum`, `difference`), and a chain of weakenings of one
  named rule, `?w` or `wk` (through the `?` steps whose formula is used
  above, which are no inference), is walked in one go (`weaken_run`): a
  sequent per node of a chain of 65 000 `?` steps over a sequent of
  65 000 formulas is 17 GB of copying, and a chain of 40 000 `wk` took
  3 GB (F20, `a_chain_of_weakenings_is_one_step`).
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
open goal and no run (`times` is 1), and writes `Γ ⊢ Δ` with both sides
in their stored order through `TEXT.ordinary`, into the formatter
itself.

- **The cyclic mode needs nothing of the view**: a derivation is built
  in the most liberal one-sided mode whatever the proof records, and
  ascending order is a cyclic sequent's order read from its first
  occurrence, so every sequent of a cyclic proof is a rotation of the
  order its rule reads and no exchange appears (documented on
  `Inference::sequent`). Rocq refuses the mode (`core-export.md`).
- **The Lambek calculus's two-sided view** (`Derivation::two_sided_mode`):
  a proof meant for an ordered mode unfolds under `L*`'s rules and the
  ordered reading, and every printer writes a sequent's hypotheses in the
  reading's order (`Reading::written`: round from the goal backwards),
  so no exchange appears. The rules of an implication are named by its
  division (`Named::division`, a public field with the closed enum
  `Division`; `Named::applied` is the one place a rule gets its
  two-sided name, used by the builder and the session): `\L`, `\R`,
  `/L`, `/R`. `Named::ALL` is a public array of 34 and stays so (its
  length is in its type); the divisions follow it in the label tables
  (`Named::DIVISIONS`, `Named::at`, the tables of `LABELLED` entries),
  and `\` and `/` are label symbols that LaTeX and Typst set as their
  own. A term's premises keep their order, the left operand's premise
  first, as every view draws `⊗` and as `terms.rs` maps premises to
  children by position: so `\L` draws its consequent's premise first and
  `/L` its argument's (the stored operands of `B⊥ ⊗ A` and `A ⊗ B⊥`).

- **A cut is a split** (`Join::Cut` in `Build::split`, as `⊗` and Mix):
  the first premise ensures the cut formula's occurrence, the second its
  dual, the cut's conclusion is both premises without them, and an
  absorbed goal goes to the premise without one, the second where the
  cut formula is the first's goal. The size estimate (`size.rs`) does the
  same, the goal's premise read off `reading.position(a)`, so the two
  stay equal (`is_the_size_of_the_derivation_built` runs proofs with
  cuts). A cut inference records its formula (`Inference::cut`, the
  member of `A` in the first premise), which `TwoSided`, the Rocq writer
  and the session's term read; a session's JSON carries it as `"cut"`.

## The two-sided view of a classical derivation

`proofs/derivation/classical.rs`: `TwoSided::new(&derivation, writing,
&limits)` reads a one-sided `Derivation` through the record of how its
sequent was written (`ClassicalSequent::writing`, a session's
`writing`; core-forest.md) and holds only, per inference of its own, the
one-sided inference whose sequent it shows (`shows`), the positions of
that sequent shown before their negation step, its `ClassicalRule`, its
premises and `times`; no sequent is copied.

- **A formula is shown on the side its connective was written on**; one
  written with a negation is shown with it, on its context's side, only
  between the rule that exposes it and the negation step right above
  that rule's premise (or, for a formula of the sequent, below the last
  rule, the root last). So every rule acts on formulas as written, and
  the negation steps of one premise stack in position order. The children
  a rule exposes in premise `j` are `exposed`'s: both factors of `⅋`, one
  each of `⊗` and `&`, the operand of `⊕ᵢ`, `!`, `?d`, and the body of
  `∀` and `∃`; structural rules expose none. A derivation of a goal (`is_of_goal`) gets no steps
  below its root.
- **The names** (`ClassicalRule::applied`): the one-sided rule and the
  principal's `Written`: `⊗` on the input side `⅋L` (`⊸L` for an
  implication), `⅋` `⊗L` on the input side and `⅋R` (`⊸R`) on the
  output, `1`/`⊥` on the input side `⊥L`/`1L`, `!` promotion `!R` or
  `?L`, `?d` `?R` or `!L`, `?c`/`?w` `?c`/`?w` on the right and `!c`/`!w`
  on the left, `⊤` `⊤R` or `0L`, `∀` `∀R` or `∃L` on the input side, `∃`
  `∃R` or `∀L`; ax, mix and wk keep theirs. A negation
  step is `~L`/`~R` by the side its formula stands on in its conclusion.
  The view has its own label tables (`UPRIGHT`, `SUBSCRIPT`, indexed by
  the declaration order, `ALL` crate-private; the quantifier rules
  `ForallLeft`, `ForallRight`, `ExistsLeft`, `ExistsRight` sit between
  `QuestWeakening` and `Mix`); a `Labels::Table` keyed by `Named` leaves
  them upright, as for LK. `rules_are_labelled_in_declaration_order`
  holds `ALL` to the declaration order and pins the quantifier rules'
  names and labels; `every_label` pins them in every target
  (`first_order_classical`).
- **A compact run is split by side** (`by_side`): the one-sided builder
  merges `?w` (or `?c`) applications whatever side their formulas were
  written on, so a run that acts on both sides is shown as two
  inferences, the input side's (`!w`, `!c`) below; the upper one shows a
  sequent with some positions left out (`Shown::hidden`: the lower half's
  weakened formulas, or the copies the upper half has not made yet). A
  reviewer's finding: one name for the whole run mixed `?w` with `!w`.
- **Read one-sided every inference is the term's**: a rule's sequents
  are its one-sided inference's, a negation step's premise and
  conclusion one one-sided sequent. `every_inference_parses_back_and_reads_as_its_term`
  checks, over proofs of generated sequents written at random, drawn
  with and without compaction, that each
  inference prints as a sequent that parses back to the same formulas,
  sides and record, that a rule's principal stands unnegated on the side
  its name says (implication names exactly for implications), that a
  negation step moves exactly one formula across, that a structural run
  adds or removes only formulas on its name's side, as many as it
  counts (some runs are split in the sample), and that the root is the
  sequent as written.
- **Bounded before it is made**: `new` walks once counting
  (`Counted`: inferences, and bytes by `Size::bytes`'s estimate with each
  inference's one-sided sequent's weights, by member through
  `Weights::of_members`, so that an instance counts at most) and refuses past
  `limits.derivation_bytes` (`Refusal::Output`) or past
  `Derivation::MOST` inferences (`Refusal::Index`, reachable only
  unbounded: the view has at most three times the derivation's
  inferences and one per root more), then walks again making them. A
  record of another sequent is a panic (`written::fault`), as an id of
  another derivation is.
- Commutative classical only: the command and the bindings draw it in a
  classical mode of the commutative order (the cyclic mode stays
  one-sided), and `ViewOptions::classical` (`ClassicalSides`, `one` by
  default) is the front ends' switch; the library's own derivations are
  one-sided whatever it says.

- **A cut's formula is exposed as a root is** (`walk`): its premises get
  the cut formula's occurrence (the first) and its dual (the second),
  each with a negation step above where the typed formula has a negation
  before it (`written::negated` against `context`, which puts the `A`
  root right of `⊢` and the `A⊥` root left); the cut is
  `ClassicalRule::Cut`, `cut`, with no side.
  `every_inference_parses_back_and_reads_as_its_term` runs proofs with a
  cut on a formula typed as written.

## Interactive proving

`proofs/interactive/` (feature `interactive`; `mod.rs` the types, the
queries and the tests, `expand.rs` the steps, `replay.rs` a session
rebuilt from its parts, behind `serialize`, whose reader is its one
caller, `terms.rs` the proof term) is the state a client
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
the command's race does), `fill(goal, &proof, view, limits, stop)` (a
proof of an equal sequent from elsewhere, another session's, carried over
by `Proof::onto` and grafted by `close_with`), `close_all` (a result per
goal, F68),
`derivation()` and `derivation_within(view, limits, stop)` (refused past
`limits.derivation_bytes` by the size estimate's measure) with open
goals as `Rule::Open` leaves, `derivation_ids()` (a drawn inference's
goal in the state), `occurrence(m)` and `formula(m)`, `proof(limits,
stop)`, and `invert(goal, &limits)`, `invert_all(&limits)` and
`is_invertible(sequent, &step)` (`invert.rs`, below). A Mix that leaves its right premise empty is
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
  (`focus::counts::split_passes`, which builds the engine's `Switches` and tallies
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
- **The cyclic mode** (`expand`): only `ax`, `⊗` and `⅋` apply
  (`StepError::Mode` for the others) and a session of a sequent beyond
  unit-free MLL is refused at the start (`Error::OrderedFragment`). A
  `⊗`'s split must be the two arcs: going round from the `⊗`, the
  formulas going right first, then those going left, else
  `StepError::Exchange` naming the first left one before a right one.
  `Step::at(position)` (`Split::At`) gives the cut point directly, the
  formulas from `position` round to the `⊗` going left, the step's own
  position sending none; `left_positions` turns it into positions, so
  `expand`, `split_passes` and the replay of a stored session read one
  list. `⅋` needs no check (its operands are neighbours in id order).
  `close` works on any goal through the net engine's goals off the roots
  (`core-nets.md`).
- **The Lambek calculus** (`expand`): the cyclic mode's rules and splits
  under the ordered reading, a rule given by its two-sided name only when
  it is the division's (`⊸L` is refused on a `/`), and without empty
  antecedents a premise without an input is `StepError::EmptyAntecedent`
  (beside the one-output check). `close` goes through the essential
  engine's goals off the roots.
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

- **The cut is a method, not a `Step`** (`Interactive::cut`, `cut.rs`):
  the cut formula's pair is appended to the session's sequent, so the
  forest is rebuilt within the caller's `limits.occurrences` and every
  existing occurrence keeps its id, the new trees following; the reading's
  positions and the classical record are extended as typed
  (`core-forest.md`). `expand` refuses a `Step` naming `cut` (`Rule`), and
  `rules` never offers it. `undo` of a cut pops the last pair
  (`Sequent::pop_cut`), which is the cut's since the steps after it were
  undone first; `from_parts` refuses a session whose cut formulas are not
  those of its cuts in the order made (the history, then the inferences
  of earlier steps), so that order holds after a read too. A cut's
  replay (`replay_cut`) recovers the split from the first premise as a
  `⊗`'s is. The invertible steps leave a cut alone: it is no step of
  theirs.
- **The search never sees a cut formula** (`search::on_own_forest`):
  `close` and the invertible steps call `prove_goal` on the session's
  forest, which, when it has cut formulas, decides the goal on a
  cut-free forest of the goal's formulas over the same arena (the
  hypotheses first with `antecedents`, so the reading's choices are
  those of the session) and carries the proof back by the shift.

## First-order sessions (`quantified.rs`, `witness.rs`, `handover.rs`)

- **A session of a sequent with a binder has `fo: Some(Quantified)`**,
  its own instance table: members past the forest as `(occurrence,
  frame)`, frames as cons cells, closed terms, eigenvariables and
  metavariables (`FoTerm::Meta`), each hash-consed (`members`, `frames`,
  `terms`, the sequent's closed terms registered first), so that a step
  done again names the same members and the replay's lookups find the
  recorded ones. A member below the forest is a bare occurrence under no
  binder; one under a binder only ever stands as its instance (the
  reader refuses it bare). The propositional session has `fo: None` and
  runs the code it ran before; `expand` checks occurrences for both,
  and `framed_premises`/`mirror` make the member premises of a
  first-order one.
- **`undo` goes back to the step's mark**: every step that makes
  entries (`framed_premises`, `carry_back`) takes a `Mark` first (the
  table's, metavariables', eigenvariables', constants' and trail's
  lengths) and records it under the step's number (`history.len()`
  before the push); `undo` takes back the trail past it and truncates
  everything made since, the hash-consing maps with it; a step that
  fails goes back to its mark at once. So a session's table holds the
  live derivation's entries only, and 20 000 apply/undo cycles cost what
  one does (the panel's integers reviewer measured them quadratic and
  the table unbounded when undo left entries behind; `close`'s constants
  outlived its undo). The session's JSON carries the marks (`"marks":
  [[step, members, frames, terms, eigenvariables, constants]]`): a
  front end that saves and reads the session between every two calls
  undoes as a live session does, with the same letters and table
  (`a_session_read_back_undoes_as_it_was_written`); without them every
  apply/undo cycle grew the table and drifted the next eigenvariable's
  letter. A mark's argument lists, metavariables and trail are derived
  on reading (each `App` term made one list, each metavariable its term,
  the trail by step number); `read_marks` refuses a mark that would drop
  what an entry, a binding, a level or an inference before its step
  names, after `replay_binders`, so the older refusals keep their
  reasons.
- **Every term of a session unfolds to at most `limits.terms`, and its
  table holds at most that many entries** (members, frames and terms, as
  the reader counts them; the session's start's limit,
  `Quantified::limit`, `Limits::DEFAULT_TERMS` for `Interactive::new`,
  the command's `--term-limit` through `Interactive::within`; one
  measure for both, else a session's own file was refused under the
  limit it was made in): `show` counts the shown terms' unfolded sizes and
  a step past it goes back to its mark (`StepError::Witness`, "would
  unfold to more than …"), `close` refuses its graft
  (`Refusal::Terms`), the reader refuses the document. A term of shared
  halves is a few nodes and 2⁴⁰ terms written out; before the bound a
  1 KB session file printed for minutes and `close` ran out of memory
  building its atom's text.
- **Shared terms are walked once**: `unify_pairs` and `same_atom` skip a
  pair met before (unification of two DAGs was exponential), and
  `same_atom` reads the shown table, where bindings are followed
  already, so no chain of bindings is walked per comparison.
- **Levels are the eigenvariable condition, as in the engine**: a
  metavariable's level is the eigenvariables made before it; a binding
  refuses an eigenvariable at or above it that is no constant and
  lowers the term's metavariables to it (trailed). A given witness is
  held to the goal's scope instead (`scope`: the eigenvariables of the
  `∀` steps on the path from the root, and the constants), which the
  checker's condition demands of a witness: an eigenvariable of a
  sibling branch is refused though it would be sound as a constant.
- **`shown` is the table with bound metavariables replaced**, members
  and frames keeping their numbers, made again after every change
  (`Interactive::show`); the views, the size estimate and the proof
  read it, never the raw table. `proof` replaces each open metavariable
  `k` by the eigenvariable `eigens + k`, which no `∀` step introduces:
  the checker's constant outside the signature.
- **An open witness is written `$1`, `$2`, …** (`instances::meta_name`):
  no reader takes a `$`, where `?1` read as the formula `?(1)`; LaTeX
  escapes it, Typst writes it as a string.
- **The axiom unifies** the two literals' terms (`Quantified::literal`
  closes an atom's arguments under its frame into closed terms, made
  where new), and the automatic axiom of the invertible steps closes
  only equal instances (`same_atom`, which builds and binds nothing):
  binding a witness is a choice. `∀` is invertible, `∃` is not.
- **`close` writes the goal as a sequent of its own** (`handover.rs`):
  one formula, the goal's formulas joined by `⅋` with the goal last
  (the reading takes a right-nested `⅋`'s left operand as the input),
  under a prefix that binds the open witnesses by `∃` and the
  eigenvariables made after the first of them by `∀`, in the order
  made; older eigenvariables and the constants are new constant
  symbols. The search's proof opens the prefix and the spine first;
  `carry_back` drops those steps, maps every other member to the
  session's (an occurrence by its offset in its formula, the frame
  extended by the proof's inner terms), maps the prefix's
  eigenvariables to the session's and every other one to a fresh one
  (a constant where no `∀` step introduces it), grafts the derivation
  and binds the prefix's witnesses. The outcome is the search's on the
  written sequent. The command's `close` calls `Interactive::close` for
  such a session (no race), on one thread.
- **Every walk over a session's terms has a stack of its own** (`close`,
  `show`, `bind`, `same_atom`, the copy into the written sequent): a
  witness is as deep as its nodes or its bindings go
  (`a_session_holds_deep_witnesses_and_bounds_its_table`, 100 000 deep).
  The reader bounds the table by `limits.terms` (members, frames and
  terms together), as the proof reader does.
- **The reader refuses what no step makes**: a binding whose term's top
  is a bound metavariable (unification follows bindings first; a chain
  of such made reading quadratic), a binding made by a step that closes
  no goal (only an axiom, `1`, `⊤` or a graft binds; `undo` of the named
  step would otherwise leave a state no session reaches), an open
  witness whose level admits an eigenvariable a `∀` step made at its
  step or later (`later`, the suffix minimum of the steps' eigenvariable
  numbers). A given witness's scope is read by intervals of a walk of
  the tree (`replay_binders`), not by walking up per step, which made
  100 000 `∃` steps take 38 s.
- **`close` takes a goal without a binder or an open witness** (the
  written sequent then has none, and its proof no table: an empty one
  stands in) and the empty goal of a classical mode (the written
  sequent has no formula); both broke at the panel's first round
  (`GoalMismatch`, a panic).
- **The JSON's `instances`** are read by `Quantified::from_parts` (every
  entry before its users, frames as long as their occurrence's binders,
  no entry twice, function symbols of the right arity, levels at most
  the eigenvariables made) and its bindings replayed through `bind` in
  the order made; then every inference by `replay_framed` (an axiom's
  literals equal instances, a binder's body under the principal's frame
  pushed, the premises the `mirror` of the rule) and `replay_binders`
  (each eigenvariable introduced once and no constant, each open
  witness opened once, a given witness free of open witnesses and in
  its scope). `a_session_proves_with_the_quantifier_rules` round-trips
  a session before and after its axiom bound a witness.
- **`close` grafts `Derivation::of_goal` of the written sequent's root**,
  two-sided in an intuitionistic mode as the propositional `close`
  grafts, not the one-sided `Derivation::new`: the one-sided unfolding
  placed structural steps where the reader's one-succedent check
  refuses them, so a session `close` had grafted did not read back.
- **A cut is refused in a first-order session** (`Error::Quantifiers`,
  "a cut in interactive proving"): it rebuilds the forest, which would
  renumber the members past it.

## The invertible rules applied by themselves

`proofs/interactive/invert.rs`: `Interactive::invert(goal, &limits)` and
`invert_all(&limits)` apply every invertible rule until none applies,
`is_invertible(sequent, &step)` judges one step, and
`interactive::Options { invertible }` (off by default,
`Settings::interactive`) says whether a front end calls them after each
step. What the code relies on:

- **The set, the same in every mode, each rule then validated by
  `expand`**: `⅋`, `⊥`, `&`, `⊤` unconditionally; `!` when every other
  formula is a `?` (one count of the non-`?` formulas per goal, so a goal
  of many `!` costs linear time, not quadratic); `1`, `⊗` and the axiom
  only when their formulas are the whole goal. In intuitionistic mode
  these are, by the reading, `⊸R`, `⊗L`, `&R`, `⊕L`, `1L`, `⊤R`, `0L`,
  `!R` (every hypothesis a `!`), `1R`, `⊗R` with no hypotheses and the
  axiom; in the Lambek calculus `\R`, `/R`, `⊗L`, `⊗R` with nothing to
  split and the axiom; in the cyclic mode `⅋`, `⊗` alone and the axiom
  (the ordered modes are unit-free MLL, so the rest never occur). A
  candidate that `expand` refuses (a premise without its one succedent
  or, in `L`, without an antecedent) is skipped, so the reading's
  conditions need no copy here. No `?` rule is in the set: `?d` and `?w`
  are not invertible, `?c` is but never ends. Mix and `wk` are not
  invertible. Click & coLLecT's "auto-reverse" applies the same set
  (its `proof.ml`, `apply_reversible_rule`: `⊤`, `⊥`, `⅋`, `&`,
  promotion when its context is all `?`, `1`, `⊗` on a one-formula
  sequent, the axiom), read in its repository in October 2026.
- **Why each is invertible, and where that was read.** Andreoli's
  classification (1992; the paper itself could not be fetched): the
  asynchronous connectives `⊥ ⅋ ? ⊤ & ∀`, whose right-introduction
  rule is always invertible, as Liang and Miller state it ("Focusing and
  polarization in linear, intuitionistic, and classical logics", TCS 410,
  2009, §2) and Miller and Saurin ("From proofs to focused proofs", CSL
  2007, §2). For ILL, Chaudhuri's thesis ("The focused inverse method for
  linear logic", CMU-CS-06-162, 2006, §6.1): the right-asynchronous `∀ &
  ⊤ ⊸` and left-asynchronous `∃ ⊗ 1 ⊕ 0 !`, and "Search using the
  backward calculus can always apply invertible rules eagerly in any
  order". The method behind every case is Curien's ("Introduction to
  linear logic and ludics", arXiv cs/0501035, the reversibility of `⅋`
  and `&`): the premise follows from the conclusion by a cut with a
  sequent made of axioms. **Derived here, not read**: the affine modes,
  Mix, the Lambek calculus and the cyclic mode (no source with these
  statements was obtainable) by the same cut, which each of these
  calculi admits (`⅋` with `⊢ B⊥ ⊗ A⊥, A, B` in the cyclic order, `\R`
  with `A, A \ B ⊢ B`); `!` with every other formula a `?` by a cut
  with `⊢ ?A⊥, A` (a dereliction on an axiom; `!A ⊢ A` two-sided),
  Chaudhuri's `!R` being a dyadic calculus's rule, which is no
  statement on this one; `⊗`, `1` and the axiom on the whole goal by
  the last rule of a cut-free proof: a one-formula sequent `⊢ A ⊗ B`
  ends in `⊗` with empty contexts, since no structural rule acts on a
  formula that is no `?` but `wk`, from the empty sequent, which nothing
  concludes, and a binary Mix needs two nonempty premises; `1` and the
  axiom close the goal, which loses nothing.
- **It ends**: every premise of these rules is lighter than its
  conclusion (fewer connectives, units and literals), which
  `inverting_keeps_every_proof_and_ends` checks on every step it takes.
  `&` copies the context, so a goal of `n` formulas `a & b` makes `2ⁿ`
  goals: the bytes of the inferences a run adds (`size_of::<Inference>()`
  and a member's per formula, per premise) are counted against
  `limits.memory_bytes`, and at the bound the run stops with
  `Refusal::Memory { phase: View, … }` in `Inverted::finished`, the steps
  taken standing (`inverting_stops_at_the_memory_bound`). The web
  bindings bound a run by the smaller of the memory and the derivation
  bounds.
- **Order**: a rule that closes the goal (`⊤`, the axiom, `1`) before
  the others, then the first position's; depth first, so `Inverted::open`
  is the goals left open from left to right (not the arena's order).
  Which order is taken changes the tree, never provability.
- **Each application is an ordinary step** (`record`, which `apply`
  ends with): the history gains one entry per step and `undo` retracts
  one at a time. Taking back a user's step with the invertible steps
  after it is the front end's: the command groups them by its step
  counts (`cli.md`), the page by the view's `invertible` list.
- **What the tests cannot catch**: the generated sequents (`generate`'s
  classical, ILL and Lambek generators, the last read cyclic too) never
  produce `⊗` alone in intuitionistic or ordered mode; the fixed cases of
  `the_invertible_rules_under_their_conditions` cover `⊗R` and the
  cyclic `⊗`. Provability is judged by the search with the default copy
  bound, and every goal left was proved, but a set that loses proofs
  only beyond three copies of a `!` would pass.
  The affine modes are fed only sequents provable without weakening (the
  classical generator's), so a rule that lost only proofs that need a
  weakening would pass there; a review that made `⊗` with a context,
  `⊕` and dereliction invertible saw the test fail in every mode where
  `expand` lets them through.

## Instances in the views (step 42b)

- `Build`, `Record` and the size's `Measure` are generic over
  `check::Instancing` like the pass (`Ground` for a proof without
  binders, `Framed` with one): the propositional instance is what it
  was. A derivation's sequents hold members; `Derivation` keeps the
  proof's `Framing` (`occurrences::instances`), and `Notation::sequent`
  takes it: a sequent of a proof with instances goes to `framed`, out of
  line, so the propositional writer kept its shape (the `render-*` and
  `derivation-*` journeys rose 5 % to 28 % before).
- An instance is printed by `Notation::instance`: a `Scope` made with
  the frame's terms written in the target's markup and the
  eigenvariables' names open, so a binder of the formula with such a
  name is renamed. It writes through `Apart`, a writer type of its own,
  so that `term_in`'s walk stays instantiated once for a propositional
  print with its iterator inlined.
- The size counts the members of a sequent with binders as the
  printer writes them (`Weights`, one vector by member): an occurrence
  per occurrence, with the names `printed_names` computes by
  `Scope::enter`'s renaming inside its root (`x` for a binder without a
  name), since an open atom's key writes `#i` and one atom term sits
  under binders of different names, and a nested binder in brackets; an
  instance as its occurrence with each loose variable's name replaced
  by its frame's term (`loose_variables`, level 0 the nearest binder
  above), plus a bound per written name where an eigenvariable's name
  can rename a binder inside it (an instance's scope holds them).
  `exact` is false for a proof with instances. Where an inner binder was
  renamed in its root only because an enclosing binder of its name was
  open (shadowing), the instance writes it shorter: its weight is then a
  bound, and the width may pass the widest sequent, the one case where
  `Size::width` is not a lower bound. Any change of the printers'
  naming or bracketing changes these weights with it
  (`binder_names_are_counted_as_written`, the size test's samples).
- An eigenvariable is written `a`, `b`, … `z`, `a_1`, … in the order of
  its number, skipping every symbol's name (`instances::eigen_names`),
  so a derivation's sequent reads as a sequent with those constants.
- The classical two-sided view draws a first-order derivation: a member
  is read through `Derivation::occurrence` wherever the view asks its
  record (`writing`, `context`, `negated`), and `Notation::classical`
  takes members and the framing, writing an instance through
  `written_instance` (the frame's scope of `instance` around
  `written_in`).
- **An exposed child is found by member, not occurrence**
  (`TwoSided::exposed_at`): with instances, a premise may hold several
  members of one occurrence (a `!∀` derelicted twice holds the body under
  two witnesses). Without a framing every member is its occurrence and a
  binary search finds it; with one, the child is the member of that
  occurrence under the frame of the member it comes from (the principal,
  or the cut formula), a binder's body under a frame whose parent is that
  frame, and of several such the one the premise holds more often than
  the conclusion. `a_first_order_view_reads_as_its_term` runs the checks
  of `reads_as_its_term` (now shared with the generated sequents' test,
  plus the printed formula equal to `Derivation::formula`'s) over
  first-order proofs, one with two negated bodies of one binder.

