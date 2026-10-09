# Walk-through of step 34 (cut, cut elimination) against `plan/notes/api.md`

Read: api.md in full, `plan/34-cut.md`, `research/34-cut.md`, `research/impact-boxes.md`,
the register entries naming 34, and `core/src` where the first change lands
(`occurrences/mod.rs`, `occurrences/reading.rs`, `proofs/{mod,interactive}.rs`,
`search/{mod,net}.rs`, `search/focus/{bias,counts}.rs`). Nothing was built or run.

## 1. The first change, sketched

Session 1 is two commits: (A) the data model, the checker and the wire; (B) the interactive rule. The
search stays cut-free. Wire level: "the level current when it lands" (3 if 33 takes 2).

### Commit A: `Cut` in the term, the forest, the checker, the wire (0.2.0)

```rust
// occurrences (3.3): the contract grows, the numbering of every cut-free forest does not
impl Forest {
    pub fn with_cuts(s: &Sequent, cuts: &Sequent, limits: &Limits) -> Result<Self, Error>;
        // cuts: one root per cut formula A (a Sequent, the wire's own form); the forest appends,
        // per cut, the tree of A and then the tree of A⊥ (the NNF dual, same shape), after the
        // conclusion's trees and the earlier pairs
    pub fn cut_pairs(&self) -> &[CutPair];          // CutPair { a: OccId, dual: OccId, size: u32 }
    pub fn dual(&self, x: OccId) -> Option<OccId>;  // root(A⊥) + (x - root(A)), both directions
    pub fn is_conclusion(&self, x: OccId) -> bool;  // x lies below one of roots()
}
// proofs (3.7-3.9)
pub enum Node { /* … */ Cut(Member, NodeId, NodeId) }   // 16 bytes; a, left (Γ, a), right (Δ, dual(a))
pub enum Rule { /* … */ Cut }                           // Named { rule: Cut, side: None }; principal() None
pub enum Fault { /* … */ NotACut(Member) }
pub enum Phase { /* … */ Eliminate }                    // used in commit C, reserved here
// checker row: Cut(a, p, q): S, T | a in a cut tree, dual(a) | take(S, a) ⊎ take(T, dual(a)); Surplus argument as ⊗
```

Reading (3.6): `A` root output, `A⊥` root input, goal chosen among `roots()` only; the positions of a cut
tree follow by the existing propagation. `Goal::is_whole_forest()` keeps the net engine off a forest with
cut trees (10.6, R98).

```json
{"version": 3, "sequent": {"terms": […], "roots": […], "atoms": […], "antecedents": 2},
 "cuts": {"terms": [{"V": 0}, {"D": 0}], "roots": [0], "atoms": ["C"]},
 "nodes": [{"ax": [5, 2]}, {"ax": [6, 3]}, {"cut": [5, 0, 1]}], "mode": "classical"}
```
`cuts` is a `Sequent` form whose roots are the cut formulas (merged by atom name); A⊥ is derived, not
written; the key is skipped when empty, so a cut-free proof stays level 1 byte for byte. The tag `cut`
joins `Node::TAGS` and the Rocq `node` (R118).

### Commit B: `Interactive::cut`

```rust
impl Interactive {
    pub fn cut(&mut self, goal: GoalId, formula: /* ? */, split: Split) -> Result<[GoalId; 2], StepError>;
}
```
It builds a larger forest (`with_cuts`) with the same prefix, extends the positions, opens `Γ, A` and
`Δ, A⊥`. Session JSON: `cuts` beside `sequent`, and inferences `{"sequent": […], "rule": "cut",
"premises": [i, j]}` (no `principal`).

### Commits C and D (sessions 2, 3), only as far as they touch the design

```rust
pub mod cut { /* 10.6: step, eliminate, Options { strategy }, Reduction, Elimination, End */ }
// nets: cut links beside axiom links, an erased set, NetError::CutNotDual, svg::net per step
```

## 2. Workarounds

1. **[friction] The cut formulas have no owner but `Forest`: `Forest::sequent()`, `Disproof`, `Goal`
   and the wire all lose them.** Design: 3.3 (`Forest::with_cuts` in 10.6, `sequent()` never
   defined), 3.12 (`Disproof { sequent, goal, … }`), 3.7 (`Proof::sequent()` "unchanged"), 7.3
   (Proof `cuts`). Today a forest owns a copy of its sequent and prints occurrences through it
   (`Forest::formula`), so the copy must hold the terms and atoms of A and A⊥: `forest.sequent()`
   becomes the conclusion plus an arena tail its roots do not reach. Then (a) `Proof::sequent()` and the
   `sequent` key either write that extended arena (the "byte-identical" claim of impact-boxes §6 is
   false, and the reader meets A twice, in `sequent` and in `cuts`) or must prune; (b) `prove_goal` on a
   session goal `Γ, A` builds `Disproof::new(forest.sequent().clone(), …)` whose `goal` names
   cut-tree members that `Forest::new(&disproof.sequent())` (what 31's `Disproof::check` must do)
   cannot number, since `Disproof` has no `cuts`; the check fails or numbers wrongly; (c)
   `close_with` compares forests, so a proof found before a later cut is a `ForeignProof` though ids
   are stable. Why: ~60 sites read `sequent()` as the arena and as the conclusion at once (impact-boxes
   item 1, "say which"); a certificate (31) built on the wrong meaning is a silent failure. Smallest
   change: reserve in `Sequent` itself (next to the [38] tables of 3.1) a private `cuts: Vec<TermId>`,
   empty for propositional input, part of `Eq`/`Hash`, written as the `cuts` key at a level, numbered
   by `Forest::new(&s)` after the roots. Then `Proof`, `Disproof`, `Goal`, `Interactive` and the wire
   carry them with no parallel parameter, `with_cuts` becomes crate-private, and 3.3 states that
   `forest.sequent()` is that value and `roots()` the conclusion. The alternative is a private `cuts`
   slot on `Disproof` and the same in every form that pairs a sequent with members.

2. **[friction] 3.8's `Cut` row and `Fault::NotACut` say "the `A` root of a cut pair"; elimination
   needs any occurrence of the `A` half.** Design: 3.8 table row `Cut(a, p, q)`, 3.2/10.6, R57. The
   `⊗/⅋` and `!/?` cases replace a cut on `o` by cuts on `o+1` and `o+1+size(left)` (and the
   dereliction case on the box's body), whose duals are `dual(o)+1…` by the same offset. Every
   intermediate proof must check, so `Cut` must name a non-root occurrence of a pair's `A` tree
   (never of the `A⊥` tree: the premise holding `a` is always the first). Read literally, the checker
   rejects every intermediate term of the first `⊗/⅋` step. Why it matters beyond 34: 3.8's table is
   the text step 31's Rocq `check` is written against (R117), and 31 comes first; a mirror built for
   roots is rewritten. Smallest change: the row requires "`a` lies in the `A` tree of a cut pair
   (`Forest::dual(a)` is defined)", `NotACut(a)` otherwise; `Forest::dual` is documented total on
   the pair's `A` tree and defined on the `A⊥` tree as the inverse.

3. **[friction] `cut::step` and `cut::eliminate` take no mode, and `eliminate` cannot say "invalid
   input".** Design: 10.6. `Proof::mode()` is "a claim, not a check" and `None` for a file without the
   key (3.7); every other call takes the mode (`check(mode)`, exporters). The cases differ by mode:
   affine `Weaken` erases a premise, Mix and intuitionistic forbid each other, the intuitionistic
   reading decides which side holds the output. `eliminate(...) -> Elimination` has no `Err`, yet the
   input may fail `check`, or sit in an ordered mode where cut is refused (R57, R134), and `End`
   (`Normal | Refused(Refusal)`) is for bounds only (P5: a bound is never "invalid"). Also undefined:
   the type `Case` (R15 wants a `Display`/`FromStr` pair and `#[non_exhaustive]`), the code
   `elimination_limit` of 4.4 (the bound is `Refusal::Work`, code `work_limit`), what `proof` means in
   the record `{"proof", "steps", "end"}` (input or result; NodeIds of step i are in step i's
   renumbering, so replay is the only reading). Smallest change: add `mode: Mode` to both, make
   `eliminate` return `Result<Elimination, Error>`, drop `elimination_limit`, say the record's `proof`
   is the input and a reader replays `step(…, Some(cut))` checking each `case`.

4. **[friction] The engines have no "the goal's occurrences" scope; 10.6 and R98 say "the conclusion's".**
   Design: 10.6 ("the dispatch's features and counts read the conclusion's occurrences"), 8.2 (`Task`),
   3.4. A cut premise `Γ, A` is not the conclusion: part of it lies in a cut tree. In the present code
   `few_equal_literals`, `bias::signs`, `counts` (`exponential`, `absorbs_from_copies`), `net::counts_admit`
   and `net::atoms` iterate `forest.ids()` or `forest.literals()`. For a cut-free forest nothing
   changes; with cut trees the cut's literals and `!`/`?` change the dispatch and the bias of a
   conclusion goal (R98's test, same engine and statistics on a conclusion goal with and without
   cuts, fails) and `counts_admit` plus `is_correct` on the whole forest hits the panic of impact-boxes
   item 10. `is_whole_forest` alone (as 10.6 has it) fixes the net engine only. Why: heuristics
   change silently; no verdict is wrong (the per-forest flags only weaken the counts), so it is a cost,
   not a bug, but R98 cannot be met as written. Smallest change: `Task` (8.2) and `Goal` (8.1) carry
   `trees()`, the ranges of the trees the goal's members lie in; for `Goal::conclusion` of a cut-free
   forest it is `0..len` and the passes keep their fast path (D17); replace "conclusion's" by
   "goal's" in 10.6.

5. **[friction] `Interactive::cut` has no formula type, no bound, no refusal that can carry one, and no
   rule for atomic validation.** Design: 3.10 (`cut(goal, formula, split) -> Result<[GoalId; 2],
   StepError>`), 3.1 (builder), 5.6, R58, R95. (a) R95 wants text under `parse` and a term otherwise;
   `Sequent::builder()` builds a new sequent and `Sequent::add` appends roots, so nothing grafts a
   formula and its NNF dual into an existing arena (R58, listed in 13 as met by 3.3 and 10.6, which
   say nothing about it). Smallest: the formula is a one-root `Sequent` (the type `cuts` already is on the
   wire), parsed by `Sequent::parse_within("|- F")`; a crate-private `Sequent::graft(&mut self, &Sequent)
   -> (TermId, TermId)` merges atoms by name and builds the dual iteratively. (b) `cut` rebuilds the
   forest (O(forest) per call, unbounded growth by repeated calls) but takes no `&Limits`, the 5.6 table
   lists it with `apply` as "none", and `StepError` has no `Refused(Refusal)` (P5 asks it of every
   refusable type). Smallest: the session keeps the `occurrences` bound given to `within`, and
   `cut` returns `Result<_, Error>` or `StepError` gains `Refused`. (c) In intuitionistic mode `Reading::new`
   runs over every id and fails with `ShapeError::Formula(o)` on a cut tree with no reading, which would
   poison every later `read`: `cut` must validate before it changes the state (3.6 says nothing), and
   `undo` must say whether the pair stays (impact-boxes: append-only) or is dropped (R95: "forest included");
   the `cuts` key then lists dead pairs or not. (d) Nothing lists goal-level rules, so 32's `GoalView` has no
   way to offer a cut (`rules(goal, position)` is per position, `Needs` has no `Formula`). Smallest: a
   `Needs::Formula`-style entry or `Interactive::can_cut(goal) -> Result<(), StepError>`.

6. **[note] 32's `Request`, `Response`, `GoalView` are the wire of `cut` and must be `#[non_exhaustive]`
   when 32 writes them.** Design: 3.10 ([32]), 7.3. 2.5's list is 28's; `{"request": "cut", "goal",
   "formula", "left"}` is a new variant at 34. Add `Request`, `Response`, `GoalView` and `FormulaView`
   to the marked list in 2.5 when they land.

7. **[friction] Net cut links are neither `links` nor determined by the forest; key names collide.**
   Design: 3.11 (`from_links(…, &[(VertexId, VertexId)])`, "[34] cut links, the erased set"), 7.3
   (net form, "[34] cut links and the erased set"), 7.1 ("one concept one key"). An atomic cut pair
   links two literals that are also dual: `(5, 6)` is then an axiom link in `links` and a cut link, and a
   literal can carry both. After the first `⊗/⅋` step the cut links join non-root occurrences, so the
   forest's pairs do not give them. A constructor reading `links` alone cannot tell, and `new(forest,
   criterion)` must create the initial cut links for a forest with pairs (R80: "only behind an explicit
   constructor"). The proof form's `cuts` means cut formulas; the net form needs a different key for links.
   Smallest: 3.11 states the cut links are a second list stored in the structure (`from_links` takes
   axiom links only, a sibling takes `cut_links` and `erased`), and 7.3 names the net keys
   `cut_links` and `erased`, with `cuts` (formulas) as in the proof. Also: the redex highlight (R167) is a
   per-call datum, not a `Style` value; 9.1's `svg::net(&n, &style, &limits, stop)` lacks it (an
   additive `svg::net_step`, or a `highlight` argument chosen before 0.1.0 since `net` is public from 28).

8. **[note] Small mismatches a session meets in the first commit.** (a) `Node::principal()` is
   documented as the acted-on occurrence; for `Cut` it must be `None` to agree with `Rule::has_principal()`
   and `Inference::principal()` (3.9: "None for ax, Mix, Cut") while `members()` yields `a`; say so in 3.7.
   (b) `ordinary::Image::read_back(&Proof, …)` has an exhaustive `Node` match (P3) that 10.6 does not list: a
   cut needs an answer (refuse as `Unsupported`, or read it as the cut rule of LK/LJ). (c) 4.4 reserves
   `cut_formula` and `not_dual_pair` without variants or payloads; `Fault::NotACut` and `Missing` cover the
   checker, `Error::Parse` the text, so name which reserved codes remain. (d) 7.3's inference form for a cut
   has no `principal`; say that reading finds the pair from the premises' sequents, as for Mix.

9. **[note] Cut is a rule-set choice the checker has no parameter for.** Design: 3.5 (`Mode`), 3.8, R119
   ("whether cut is allowed is a rule-set parameter"). `Proof::check(mode)` accepts `Cut` in every
   commutative mode and refuses it in an ordered one; there is no way to ask "check this as cut-free",
   which the Rocq writer (`params.cut`) and the termination test (R100: the end has no `Cut`) want. Not
   needed to ship: `Proof::is_cut_free()` (a scan, additive) serves. Adding a `Mode` field later is
   additive too (private fields).

10. **[note] Composing two proofs on a cut formula (R202) is not placed.** Design: none. Tests and users
    that `prove_goal` both premises (`Goal::new(&forest, members)` over the extended forest) must join the
    node arenas by hand into `Proof::new`. A public `Proof::cut(&left, &right, a) -> Result<Proof, Error>`
    over two goal proofs of one forest (F89's goal makes it checkable) is additive and is also what
    `Interactive::proof()` does through `Terms`.

11. **[note] Quantifier room for cut (the report's "Where quantifiers go", R230).** 3.14 and 10.10 suffice
    for a closed cut formula: the `cuts` form is a `Sequent` form and inherits 38's keys, `Cut(Member)`
    with `Member` an instance `(x+1, F·w)` covers the `∀/∃` principal case (the new cut is on the body
    under the witness). Two gaps. A cut formula that mentions an eigenvariable (a cut inside a `∀` premise)
    cannot be written: `cuts` formulas are ground or bound (3.14), and instantiation never rewrites a
    formula; it would be a cut on a quantified formula, or a proof-local term extension on the cut pair
    (a 38 decision to record). And `dual` of a member needs the owner's table (`(dual(o), F)` must exist in
    `Instances`): add `dual(m) -> Member` to the owner methods of 3.4 beside `occurrence` and `formula`.

12. **[note] `elimination` re-checks per step.** `step(&Proof, …) -> Reduction { proof }` takes and returns
    owned proofs (P1), each cloning the extended forest (25 bytes an occurrence) and each needing the
    per-node zones from the crate-private checker pass (R100: a `⊤` or an absorbed context cannot be read
    from a node). Fine for a front end that steps; `eliminate` must run on one private arena and one
    forest, or it is quadratic in steps times nodes, and `work` must count it so. State in 10.6 that
    `eliminate` shares its forest and that the pass is crate-private to `proofs::cut`.

## 3. Register entries naming 34

| entry | state | where |
|---|---|---|
| R6 compatibility policy | met | 7.1 (level by content, `cuts` raises it, cut-free stays level 1) |
| R13 cut formulas in Proof JSON | met, with item 1 | 7.3 Proof (`cuts`, tag `cut`) |
| R14 in the session JSON | placed, `undo` rule open (item 5c) | 3.10, 7.3 Interactive |
| R15 elimination record | partly: record yes, `Case` and `proof` undefined, no session value (items 3, 12) | 7.3 last paragraph, 10.6 |
| R31 memory bound with cuts | met | 3.8 (integers, "`Cut` too") |
| R32 bounded elimination | met by `Limits::work`/`memory_bytes`, `End::Refused`, `Phase::Eliminate`; mode missing (item 3) | 10.6, 5.2, 5.6 |
| R243 stop with work | met | 5.2, 5.6 |
| R42, R43 no clock, no recursion | met | P6, P9 |
| R50 non_exhaustive | met; add `Case`, `Request`, `Response` (items 3, 6) | P3, 2.5, 10.6 |
| R57 cut roots in the forest, refused when ordered | met in the forest; the ordered refusal's error is unnamed (R134) | 3.3, 10.6 |
| R58 dual of a term in the arena | not placed (item 5a) | 3.1 builder is for new sequents |
| R59 builder | placed (3.1); not the graft | 3.1 |
| R60 `Rule::Cut` in the view | met | 3.9 |
| R76, R77 boxes, vertices | met | 3.11, 10.5 (VertexId past the forest) |
| R80 cut link, explicit constructor | partly (item 7) | 3.11, 7.3 |
| R95 interactive cut | partly: formula type, bound, refusal (item 5) | 3.10, 10.6 |
| R98 engines see the same goal | partly: net engine yes, dispatch and bias scope no (item 4) | 10.6, 8.1 |
| R99 reading and R1 to R3 with a pair | met | 3.6, 3.8 |
| R100 elimination on terms | partly (items 3, 12) | 10.6, 3.8 |
| R101 elimination on nets | partly: step function named, signature, `cut_links`, highlight absent (item 7) | 10.6, 3.11 |
| R117, R118, R120 checker rule, tags, oracle | met, with item 2 for the rule's wording | 3.7, 3.8, 9.2 |
| R119 `Cut` fits 16 bytes | met; "rule-set parameter" not placed (item 9) | 3.7 |
| R129, R131 error forms, cut errors | partly: codes reserved, variants and payloads not (item 8c) | 4.3, 4.4 |
| R139, R160 Rocq refusal and certificate | met | 9.2 (`Unsupported::Rule`, `params.cut`, `FORMAT`) |
| R167 drawing of cut links and steps | partly (item 7) | 9.3 (`c` ids), 10.6 |
| R179 command | flags map onto `limits.*`; `cut` command and elimination subcommand are the command area's, exit statuses by `ErrorKind` | 6.6 |
| R202 building proofs with cuts | not placed (item 10) | none |
| R206 counter-exact net target set | partly: 3.11 states a gate on "the target set's net rows", which `bench/targets.sh` (focused engine only) lacks until R206 creates them | 3.11 |
| R228 rustdoc and rules files | commit duty, nothing for the design | |
| R230 "Where quantifiers go" | met by 3.14, 10.10 plus item 11 | |
| R248 matches name variants | met | P3 (the lint covers `proofs/`) |
| R254 `from_proof` refuses | met | 3.11 |

## 4. What fits well

- Offset ids (P2) and the dual by offset: `Cut(Member, NodeId, NodeId)` stays 16 bytes, no table, and
  every elimination case stays local; the forest numbering contract (3.3) is what makes the Rocq
  mirror possible.
- Closed `Node`/`Rule` with a planned 0.2.0 bump, the wire level computed from content, and `cuts`
  skipped when empty: a cut-free value keeps its bytes (D17, P8), and an old reader refuses a cut proof
  by name.
- `Limits`, `Progress`, `Phase` and the stop give elimination its bound and its stop with no new
  machinery; `Named { rule, side }` takes `Cut` as one row; `Goal` and `Interactive::cut` as a method
  beside `apply` keep `Step` a position-only command.
