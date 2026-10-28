# Step 41: cut, and cut elimination

## Session 41a: cut in the data model, the checker and interactive proving

### Outcome

**Proofs with cuts exist in the library, the command and the bindings,
checked in every commutative mode and certified in linlog's Rocq
library; the search stays cut-free and no counter of either target set
moved.** A sequent holds its cut formulas beside its conclusion
(`Sequent::cuts`), which its forest numbers as trees of their own after
the conclusion's, the formula `A` and then its dual `A⊥`. `Node::Cut(a,
l, r)` is the cut of the dyadic calculus, from `⊢ Θ ; Γ, A` and `⊢ Θ ;
Δ, A⊥` to `⊢ Θ ; Γ, Δ`, and the checker's rule for it holds in the
classical, affine, Mix and intuitionistic modes; the ordered modes
refuse it. In interactive proving the user names the cut formula, typed
as the mode writes formulas (`CutFormula`, `Interactive::cut`), and the
two premises open as goals; the two occurrences the cut adds are
recorded on their sides as typed, in both surfaces, so every view,
one-sided or two-sided, prints the cut formula as written. The command's
`interact` has `cut G C [P…]`, README's usage section shows it with real
output, and the bindings have `cut`. The derivation views, the text
tree, LaTeX, Typst, SVG and the JSON forms carry the cut (wire level 6),
and the Rocq library gained the cut as an optional rule of both its
calculi, with its checker arm and soundness case, so that a proof with a
cut has a certificate.

The panel's three members (the argument on Fable 5.1 at `high`, the
counterexamples on Opus 5.5 at `high`, the integers and limits on Sonnet
5.5 at `high`) reviewed the checker's cut rule on the frozen tree of the
first commit; their findings and the answers are below. A fresh reader
(Opus 5.5 at `high`) compared the Rocq trusted base's two new rules with
the textbook calculi and with NanoYalla's and Yalla's kernels and found
nothing.

### What was built

- **The data model.** `Sequent` has a private `cuts: Vec<[TermId; 2]>`,
  each pair a cut formula `A` and its dual, of one shape;
  `Sequent::add_cut(formula)` joins a one-root sequent's arena (atoms by
  name) and builds the dual's terms (`push_cuts`, each term once for
  all the cut formulas given, reusing equal terms of the arena, so that
  a written sequent reads back to the same arena). `optimize`, `add`, `check`,
  `occurrences` and `fragment` cover the cut formulas; `Display` writes
  the conclusion only. The forest numbers the pairs after the roots
  (`Forest::cut_pairs`, `has_cuts`, `is_conclusion`, `dual`,
  `is_cut_formula`); `roots()` stays the conclusion, as the forest's
  contract in `plan/notes/api.md` (3.3) fixed. The intuitionistic
  reading reads `A` in output position and `A⊥` in input position
  (`ShapeError::CutFormula` otherwise).
- **The checker** (`proofs/check/pass.rs`, and its first implementation
  `oracle.rs` from the pairs by an independent route): `Cut(a, l, r)`
  takes `a` from the first premise and `dual(a)` from the second and
  joins them as `⊗` does; `Fault::NotACut` for a member that is no
  occurrence of a cut formula's `A` tree; `Fault::Forbidden` in an
  ordered mode, with every rule but `ax`, `⊗` and `⅋` (the reversing
  dual of an ordered formula is no tree of `A`'s shape, so there is no
  order-keeping cut to check; refused, as `plan/41-cut.md` allows).
  Intuitionistically nothing is added: a premise holding `a` in output
  position has it as its one goal, so a cut keeps one goal whichever
  premise proves the cut formula, and a `⊤` premise that has its goal
  cannot absorb the cut formula as a second one (R2). The test `cut`
  pins the deliberately wrong cuts: on the dual, on a formula of the
  conclusion, premises the other way round, a cut formula the premises
  lack, a premise that would be `⊢ ⊤, A` with two goals in
  intuitionistic mode, a cut formula that is no intuitionistic formula,
  and every ordered mode.
- **Proofs with cuts for the tests**: `with_cuts`
  (`proofs/interactive/cut.rs`, test-only) cuts the session of every
  generated sequent's proof on one of its formulas and then one of the
  two goals again, and closes both by the search; `oracle::proofs` holds
  them, so `agrees_with_the_first_implementation` (verdict and error, on
  the proofs and their mutants, which now mutate cuts too), the size
  estimate's equality with the derivation built, and the compact view
  all run proofs with two cuts each (over 50 classical and over 50
  intuitionistic ones, which the test asserts).
- **The views.** The derivation builder splits a cut as it splits a `⊗`
  (`Join::Cut`); the size estimate follows it; `Inference::cut` gives a
  cut inference's formula. The two-sided view of a classical derivation
  (`TwoSided`) exposes the cut formula in the first premise and its dual
  in the second as it exposes a root, with a negation step above where
  the typed formula has a negation, and names the rule `cut`
  (`ClassicalRule::Cut`). LaTeX, Typst and SVG label it `cut`
  (`every_label` pins a classical and an intuitionistic proof with a cut,
  `cut.*` and `cut_ill.*`).
- **The recorded sides** (step 40's notes). `CutFormula::parse_within`
  reads the typed text as the goal `⊢ C` of the mode's syntax (strict
  ILL, or one-sided ILL as `input.intuitionistic` says, or classical as
  written), a parse error pointing into the text typed; the session
  gives the `A` tree the sides (or the classical record) of that goal
  and the `A⊥` tree their flips, which is what the same text written
  left of `⊢` records, never reading them off the one-sided `C` and
  `C⊥`. `the_recorded_sides_are_the_readings` now also cuts every
  generated sequent on its goal and on two formulas of units and checks
  the session's sides against `Reading::new` of the extended forest.
- **The round trip**, extended to proofs with cuts:
  `every_inference_parses_back_and_reads_as_its_term` (classical,
  two-sided) and `every_inference_prints_as_an_intuitionistic_sequent`
  run proofs with a cut on a formula typed as written, over 50 cut
  inferences each.
- **Interactive proving.** `Interactive::cut(goal, &formula, left,
  &limits)`: the session's sequent gains the cut formula, its forest is
  rebuilt within `limits.occurrences` (every existing id kept), and the
  premises `Γ, A` and `Δ, A⊥` open as goals; intuitionistically each
  must have one goal (`StepError::Succedents`), and an ordered mode
  refuses the cut (`StepError::Mode`). `undo` takes the cut back with
  its formula (`Sequent::pop_cut`); the session's JSON names a cut's
  formula, and reading a session back checks that its cut formulas are
  those of its cuts in the order made. The invertible steps leave a cut
  alone; `rules` never offers one, and a `Step` naming `cut` is refused.
- **The search never sees a cut formula.** `prove_goal` and `race`
  decide a goal of a forest with cut formulas on a cut-free forest of
  the goal's formulas over the same arena and carry the proof back by
  the shift, checking it again (`search::on_own_forest`); a cut-free
  forest takes the path it took before.
  `a_forest_with_cuts_is_searched_without_them` runs the default, the
  focused (or two-sided) and the net engine on a sequent with a cut
  formula and on goals holding cut occurrences: every proof is cut-free
  and checks.
- **The JSON forms**, at wire level 6 as `wire.rs` now says: a sequent's
  `"cuts": [a, …]` (the term of each `A`), the proof's tag `{"cut": [a,
  l, r]}`, a session's `"cut": a`; every document whose sequent holds a
  cut formula is written at level 6, every other one byte for byte as
  before. `wire/level-6.txt` pins a proof and a session.
- **The front ends.** `linlog interact`'s `cut G C [P…]` (README, the
  guide for AI models in `skills/linlog/`, `--help`, `help`), a test of
  it in `cli/tests/cli.rs` with a proof file that `check` validates; the
  bindings' `cut(session, goal, formula, left, settings)`, tested, and
  every session of the bindings' property test of intuitionistic text
  cut on its goal before it is closed.
- **Elsewhere.** A net is not built of a proof with a cut
  (`NetError::Rule`, until the third session's cut links); the read-back
  to LK and LJ refuses one (a cut-free search never hands it one);
  NanoYalla's certificate refuses one (`Unsupported::Cut`), and the
  kernel by default is then linlog's library.
- **The Rocq library** (`rocq/`, by a sub-agent on Opus 5.5 at `high`,
  reviewed): `rules` gains `cut` and `ll` the rule `cut_r` (NanoYalla's
  `axiomcut.v` statement under the guard `cut R = true`); `irules` gains
  `icut` and `ill` the rule `cut_ir` (`Γ ⊢ A` and `A, Δ ⊢ C` to `Γ, Δ ⊢
  C`); `classical`, `intuitionistic` and `affine` keep their cut-free
  meaning (`cut := false`), so the bridges' statements are unchanged.
  `proof` has `Cut A l r` (last, as `Node`), `iproof` `ICut A l r` with
  a symmetric checker arm (either premise may prove `A`), each refused
  without cut; `derive_sound` and `iderive_sound` have their cases. The
  refutation theorems that hold only cut-free (an atom's balance, the
  count equation) gained `cut R = false`, which the certificates' tests
  compute, so a refutation certificate stated with cut fails closed.
  `rocq/tests/Cuts.v` holds certificates with cuts (on a `⅋`, on a `?`
  formula, with Mix, two-sided with the proving premise first and
  second) and the rejections. The writer states the calculus with
  `cut := true` where the derivation uses a cut and moves a `?` cut
  formula into the unrestricted zone on its premise.

### Decisions

- **The cut formulas belong to the sequent** (`plan/notes/api.md` 3.1),
  stored with their duals' terms, so that the forest, the proof, the
  session and the wire carry them through the `Sequent` they hold.
- **Ordered modes refuse the cut**, as `plan/41-cut.md` allows; the
  report of the ordered calculi (step 35) has no order-keeping cut whose
  checker this step was asked for.
- **The search decides a goal of a cut forest on a cut-free forest of
  its own** rather than teaching every engine, the dispatch's features
  and the counts to skip the cut trees: it keeps every engine's input as
  it was, so the counters are unchanged by construction, and it is the
  pattern the net engine already follows for an ordered goal. It costs a
  copy of the arena per search of such a goal.
- **A cut on any occurrence of an `A` tree**, not only its root, is a
  term the checker accepts (`plan/notes/api.md` 3.8: elimination moves
  cuts to subformulas); the interactive rule makes root cuts only.
- **The cut formula's record is the typed formula's, flipped for the
  dual**: the text is read once as the goal `⊢ C`, and the `A⊥` tree
  takes the opposite side at every occurrence, which is what the parser
  records of the same text written left of `⊢` (a `⅋` on the output side
  and a `⊗` on the input side are where the side flips, and they are
  dual, so the flips stand at the same places).
- **`push_cuts` reuses equal terms**: without it the arena gained a
  second copy of every dual at each JSON round trip. It reads the arena
  with `get` and `insert`, not `entry`, since a second use of
  `HashMap<Term, TermId>::entry` made the compiler outline the one in
  `optimize` and cost the parser's journey 4 % (below); and it dualises
  every term once for all the cut formulas it is given (the panel's
  finding 1).

### Departures from the design

- **The wire form of the cut formulas** (departs from the design):
  `plan/notes/api.md` (section 7, the proof form) put `cuts` on the
  proof form as a value in the sequent's form; they are a key of the
  sequent's own form, `"cuts": [a, …]`, the terms of the cut formulas in
  the sequent's arena, so every form that holds a sequent carries them
  and none needs a key of its own.
- **`Interactive::cut(goal, &CutFormula, left, &limits)`** (departs from
  the design): the design's `cut(goal, formula: &Sequent, split)` took a
  one-root sequent; step 40's notes ask for the formula as typed, with
  its record, so it takes a `CutFormula` (`From` a `Sequent` still works,
  read as the one-sided syntax writes it), the positions going left as
  `Step::left` takes them, and the limits that bound the grown forest.
- **No `Goal::is_whole_forest` and no goal-naming of trees in `Task`**
  (departs from the design, 10.6, R98): the search re-roots a goal of a
  cut forest instead (Decisions).
- **A certificate of 0.1.0 that states its calculus by a record literal
  no longer compiles** (departs from the design, for the author): a
  one-sided certificate of a proof with Mix or affine weakening, and the
  refutation certificates of such a calculus, wrote `{| mix := true;
  weakening := false |}`, and the record now has a third field, `cut`,
  which a Rocq record literal must give (derived from Rocq's record
  syntax, not compiled here). Every certificate that names its calculus
  (`classical`, and two-sided `intuitionistic` or `affine`), which is
  every classical one-sided and every intuitionistic certificate, keeps
  its form and its meaning: `rocq/tests/Certificates.v`, `TwoSided.v`
  and `FromLinlog.v` and the exporter's snapshots `ll.v`, `mll.v`,
  `mall.v`, `mell.v`, `ill.v`, `labels.v`, `labels_ill.v` and the
  classical and intuitionistic refutations are 0.1.0's form, unchanged by
  this session, and compile against the new datatype and checker (the
  check `rocq-linlog` in `nix flake check`); `rocq/tests/Modes.v` and the
  snapshots `mix.v`, `affine.v`, `refuted_classical_affine.v` and
  `refuted_equation_mix.v` gained `cut := false`, and nothing else. No
  constructor was renamed or reordered (`Cut` and `ICut` come last), and
  the checkers and the soundness theorems keep their names and
  arguments.
  Keeping the two-field literal would take the cut out of the record (a
  calculus of its own beside `ll`, or a parameter of `ll`, which would
  change every certificate's statement), so the session kept the field
  and the break for the 0.2.0 release (step 45) to state.
- **No new certificate `FORMAT`**: the design's `params.cut` is the
  record field `cut` (and `icut`); the code has no format marker to
  raise, and the library's version stays 0.1.0 for step 45 to bump.

### Options, front ends and quantifiers (D15, D16, D17)

- No output gained an option: the cut is drawn and written where its
  inference is, under the existing `ViewOptions` and styles. The input
  of a cut formula follows `InputOptions::intuitionistic` (the command's
  `--ill-syntax`, the bindings' `input.intuitionistic`), the default
  strict.
- Quantifiers: a cut formula is a term of the sequent's arena, so a
  first-order one is built of the binders and instance tables of
  `plan/notes/api.md` 3.14 as the conclusion's are; `push_cuts` dualises
  a binder with the other connectives (`Term::dual`, `∀` to `∃`), the
  recorded sides pass into a binder's body as `lowered` and `Written`
  pass them, and the checker's arm takes members, so an instance of a
  cut formula is a member like any other.

### The closed enums' new variants, for step 45

`Node::Cut` and `Rule::Cut` (so `Rule::ALL` has 17 entries and
`Named::ALL` 35, both arrays). Marked enums gained `Fault::NotACut`,
`ShapeError::CutFormula`, `Error::CutFormula` (code `cut_formula`),
`export::rocq::Unsupported::Cut` and `ClassicalRule::Cut`. New public
items: `Sequent::cuts`, `Sequent::add_cut`, `Forest::cut_pairs`,
`has_cuts`, `is_conclusion`, `dual`, `is_cut_formula`, `Inference::cut`,
`CutFormula` (`parse_within`, `sequent`, `From` an `IllSequent`, a
`ClassicalSequent` or a `Sequent`; a new public type, unmarked: it has no
public field), `Interactive::cut`, the bindings' `cut`. The changelog
lists them under `[Unreleased]`.

### The panel

Three members, each in a fresh context with the change, the claim and
the code but not this session's reasoning, reviewed the checker's cut
rule on the frozen tree of the first commit (`kzpnymnw`, in the jj
workspace `../linlog-step41a-panel`), briefed with conduct.md's rules on
cores (4 to 7), memory (8 GiB per scope) and bounded runs, and told to
report only what breaks soundness, completeness or a stated requirement;
one round each. The claim they judged: `Proof::check(mode)` accepts a
term with cuts only if it is a derivation of its conclusion in the
mode's calculus with cut, refuses a cut on any member that is no
occurrence of a cut formula, a cut whose premises lack `A` or `A⊥`, a
cut in an ordered mode and an intuitionistic cut whose premise would
have two goals, no integer of the pass wraps, and the oracle reports the
same verdict and error.

- **The argument** (Fable 5.1 at `high`): **no finding.** It derived the
  invariant from the code before reading any argument (a `State` stands
  for every `⊢ Θ′ ; Γ` with `Θ′ ⊇ theta`, and with `any` for every
  extension of `Γ`; the cut is `⊗` without its `put`), checked the
  forest's numbering, `Forest::dual`, `push_cut`, the JSON reader, the
  absorption, the unrestricted zones, affine weakening and Mix site by
  site, showed by induction on the reading's table that `x` and
  `dual(x)` always take opposite positions, so the intuitionistic cut is
  sound and complete whichever premise proves the cut formula, and
  compared the oracle's route. Seventeen hand-written witnesses in six
  modes gave the verdicts it predicted.
- **The counterexamples** (Opus 5.5 at `high`): **no finding**, over
  about a million terms. It wrote a judge of its own (a formula-level
  checker numbering the occurrences itself and dualising with its own
  function), used the engines as a second judge (every accepted term
  with a cut decided cut-free by `prove_goal`, never unprovable), built
  5 605 correct proofs with cuts (2 062 intuitionistic, 957 with two or
  more cuts, 897 on a strict subformula of `A`; all accepted in their
  mode, all `Forbidden` in the ordered ones), and checked random terms
  with a quarter of their nodes cuts, twelve mutants of every proof, and
  hand-written level-6 JSON files, against the oracle in six modes: no
  disagreement. Its one apparent incompleteness, a cut naming the first
  pair's `A` with a second premise holding the second pair's `A⊥` when
  the same formula is cut twice (`Missing`), is not one: a term names
  occurrences, not formulas, as for every rule, and the same term naming
  its own pair's dual is accepted. **Answer**: holds as designed;
  nothing changed.
- **The integers and limits** (Sonnet 5.5 at `high`): **one finding,
  about time.** No integer wraps or saturates into a wrong verdict, and
  nothing allocates past a bound: the occurrence totals with cut trees
  saturate and are refused (`Refusal::Index`), out-of-range cut indices
  are refused before any work, a chain of `Cut(a, p, p)` doubling the
  zone is refused by `Fault::Surplus` within microseconds, and the size
  estimate, the derivation builder, `Interactive::cut` and the search's
  re-rooting hold their arithmetic. The finding: `push_cut` built a map
  of the whole arena per cut formula, so a JSON sequent of `n` terms and
  `c` cut formulas cost `n · c` before the occurrence limit was asked (a
  chain of 20 000 terms with 20 000 cuts, 389 KB, read in 9.7 s; a
  document of 4 MB would take minutes). **Answer**: holds, fixed at the
  cause in the first commit: `Sequent::push_cuts` takes all of a
  document's cut formulas at once and dualises each term once, with a
  stack instead of a pass over the arena below each cut, so the read is
  linear (`many_cut_formulas_read_in_linear_time` reads that document;
  the test file runs in 0.04 s). The fix's first form built the map for
  a sequent without cut formulas too, which the ratchet caught
  (`read-json` +12.9 %); it now returns at once without them.

The Rocq trusted base had its own fresh reader (Opus 5.5 at `high`),
as `.claude/rules/rocq.md` asks of a change of a rule: `cut_r` is
NanoYalla's `axiomcut.v` cut word for word under the guard `cut R =
true`, `cut_ir` the textbook intuitionistic cut, every existing rule is
unchanged, `classical`, `intuitionistic` and `affine` stay cut-free, the
refutation theorems that gained `cut R = false` still say what their
certificates claim (a certificate stated with cut fails closed), and
both checkers refuse a cut in a calculus without one. No finding.

### Gating

Every code commit passed the gate (`systemd-run … taskset -c 4-7 nix
develop -c gate`), each at its final tree after the rebase onto `main`
and the panel's fix: "Add the cut to sequents, proof terms, the checker,
the views and the Rocq library" in the workspace `../linlog-step41a-panel`,
"Bless the behaviour lock for interact's cut command" at its tree (which
holds "Add the cut command to linlog interact", whose help the lock
pins and which is blessed in that commit of its own: only
`interact-help` and `help-interact` moved), and "Add the cut to the web
bindings" at the top. Two gates on the way failed on the ratchet and
were fixed at the cause. First, the parser's journey `read-text` counted
12 857 516 instructions, 4.79 % over its ceiling of 12 269 416:
callgrind of the journey at the base and at the change put the whole
difference in `Sequent::optimize`, whose `HashMap<Term, TermId>::entry`
was inlined before and was an outlined call (954 527 instructions) once
the cut's dual used the same method; with the dual reading the map by
`get` and `insert` the journey counts 12 330 934 (the base 12 348 729).
Second, the panel's fix first built the map for a sequent without cut
formulas, and `read-json` counted 14 447 633 (+12.87 %); returning at
once without them, 12 906 513 (+0.83 %, as before). The ratchet's one
note, never a failure: the cache misses of `check-qbf-20-2` moved by more
than 5 %. Both `cargo hack` runs passed inside the gate and by hand
(the first gate had failed there on an import that only `parse` uses,
now behind that feature).

### The target sets

From the release build of the stack before the rebase (its search code
is the final one's: the rebase brought only documentation and the fix
only how a cut formula's dual is built), the focused set (`bench/targets.sh cut`, cores 2 and
3) and the net set (`bench/net-targets.sh cut`) ran in one detached unit
(`step41a-targets`) under the machine lock. `linlog-bench summary
--against bench/targets/classical-surface.csv bench/targets/cut.csv
--counters`: no verdict differs, and of the 145 problems both decide
alike on one thread, 0 differ in `nodes`, `splits`, `memo_hits`,
`memo_entries`, `links` or `tests`. Against
`bench/targets/net-classical-surface.csv` the net set: no verdict
differs, 0 of 64.

### What the next sessions must know

- **Elimination (41b) moves cuts to subformulas of the `A` tree**: the
  checker already accepts `Cut(x, l, r)` for any `x` of an `A` tree, its
  dual `Forest::dual(x)`; every intermediate proof lives over the same
  forest, so a reduction step adds no cut formula. A cut on a subformula
  in input position has its premises' roles swapped intuitionistically,
  which the checker, the derivation view and the Rocq `ICut` arm all
  accept; the size estimate and the view put the absorbed goal by
  `reading.position(a)`.
- **A proof with cuts cannot be carried by `Proof::onto`** (its cut
  members lie outside every conclusion), so `close_with` and `fill` take
  cut-free proofs; a graft of a proof with cuts would first append its
  cut formulas to the session.
- **Nets (41c)**: `ProofStructure::from_proof` and
  `MellStructure::from_proof` refuse a cut node today (`NetError::Rule`);
  a cut link joins `a` and `dual(a)`, both vertices of the forest's cut
  trees, which `ids()` already numbers.
- **The page (41d)**: the bindings' `cut` takes the formula's text and
  the positions going to the first premise; its errors are the parser's
  (with a span in the text), `cut_formula`, and the step's.

### Open

- The library's `Error::Parse` message says "cannot parse the sequent"
  for a cut formula too; the command prefixes "cannot read the cut
  formula", the bindings pass the error as it is.
- `Sequent::pop_cut` drops an atom no term names, which only a
  hand-written JSON sequent has, when a cut is undone.

### Checklist

- [x] Proof terms with cuts: the cut formula and its dual as trees of
  their own beside the sequent's (`Sequent::cuts`, the forest's cut
  pairs), the place for quantifiers kept (the arena's terms, 3.14's
  tables); `Node::Cut`.
- [x] The checker's cut rule in every mode: the commutative ones check it,
  the ordered ones refuse it (`Fault::Forbidden`), said so; the oracle's
  arm; tests that wrong cuts are refused (`cut`: a mismatched cut
  formula, the dual or a conclusion's member, swapped premises, a
  premise with two goals in intuitionistic mode, a non-intuitionistic
  cut formula, every ordered mode), and the differential test over
  proofs with two cuts.
- [x] The cut's two occurrences recorded on their sides in both surfaces
  (`CutFormula`, `Interactive::cut`); the two-sided view names it `cut`;
  the recorded-sides test and both round trips extended to proofs with
  cuts.
- [x] The interactive cut rule in the library, `linlog interact` (`cut G C
  [P…]`) and the bindings (`cut`), the formula typed as written in the
  mode, with tests; README's usage section shows it with real output
  that `cli/tests/readme.rs` runs; the guide for AI models
  (`skills/linlog/`) says it.
- [x] The derivation views, the text tree, LaTeX, Typst, SVG and the JSON
  forms carry cuts; the wire level 6 as `wire.rs` now says, pinned by
  `wire/level-6.txt`.
- [x] The Rocq library's datatype, checker and soundness proof with the
  cut rule (both calculi), certificates with cuts compiled
  (`rocq/tests/Cuts.v`, the exporter's `cut.v` and `cut_ill.v`), the
  theorems' assumptions closed; the trusted base read by a fresh reader.
- [x] The panel's review of the checker's cut rule on a frozen tree (the
  argument on Fable 5.1 at `high`, the counterexamples on Opus 5.5, the
  integers on Sonnet 5.5), each finding answered (one held and is fixed).
- [x] Both target sets against `classical-surface` and
  `net-classical-surface`: no verdict differs, every decided row's
  counters equal (145 and 64); the search returns only cut-free proofs
  (`a_forest_with_cuts_is_searched_without_them`).
- [x] Every code commit gated; the behaviour lock changed only in a
  commit of its own.
- [x] `nix flake check` at the session's last commit, under the machine
  lock (exit 0).
- [x] `.claude/rules/` amended (`core-forest`, `core-proofs`,
  `core-derivations`, `core-search`, `core-sequents`, `core-nets`,
  `core-export`, `rocq`, `cli`, `web`), README and the changelog true.
- [x] Every new variant of a closed enum listed for step 45 (`Node::Cut`,
  `Rule::Cut`).

## Session 41b: cut elimination on proof terms

### Outcome

**Cut elimination works on proof terms in every mode that has the cut,
one step at a time and to the end, each step named by its case and
checked; the library and the command have it, and the search and its
counters are unchanged.** `proofs::cut::step(&proof, mode, at, &options,
&limits, stop)` reduces the cut at `at`, or the one the strategy picks,
and returns a `Reduction`: the proof made, which the checker accepted in
`mode` for the same conclusion over the same forest, and the `Step`,
which names the cut (`cut`, its node in the proof the step was taken on,
and `formula`, the occurrence it cut), the `Case` by its usual name, and
`above`, the node the step acted at besides the cut (the rule a
commutative case moved the cut above, the copy a dereliction or a
contraction opened, the box a box entered), which is what the page (41d)
needs to highlight a step and undo it by keeping the proof before it.
`proofs::cut::eliminate` takes steps by `Strategy::Topmost` (the cut
first in the arena, so that no cut stands above it) until no cut is
left, or until a bound of `cut::Options` (`steps`, 10 000 by default;
`nodes`, 100 000 by default), of `Limits` or the stop ends it, and then
returns the last proof it checked, with its cuts, and the refusal
(`End::Refused`). `linlog eliminate PROOF` prints each step's case and
cut formula and the proof reached in every output format, or the
record in JSON at wire level 6; README's usage section shows it with
real output that `cli/tests/readme.rs` runs, and the guide for AI models
says it.

The panel's three members (the argument on Fable 5.1 at `high`, the
counterexamples on Opus 5.5 at `high`, the integers and limits on Sonnet
5.5 at `high`) reviewed the elimination on the frozen tree of
"Bless the behaviour lock for the eliminate command" (`b05f1292`), each
in a jj workspace of its own; two findings held (the argument and the
counterexamples found the same defect of the axiom case, the integers a
step that its bounds did not reach inside), and both are fixed at the
cause with a test that reaches them; their answers are below.

### What was built

- **The module `proofs::cut`** (`core/src/proofs/cut/mod.rs`): `Options`
  (`strategy`, `steps`, `nodes`, serde, `Settings::cut`), `Strategy`
  (`Topmost`), `Case`, `Step`, `Reduction`, `End`, `Elimination`, `step`
  and `eliminate`; `Proof::is_cut_free`; `Refusal::Elimination { what:
  Bound, limit }` (code `elimination_limit`, the code `plan/notes/api.md`
  reserved; settings keys `cut.steps` and `cut.nodes`), `Phase::Eliminate`,
  `Error::NoCut` (code `no_cut`, kind unsupported) and `Error::Reduction`
  (code `reduction_rejected`, kind defect).
- **A step rewrites the arena copy-on-write.** The proof's nodes stay as
  they are; a step appends the nodes it makes, puts its replacement in
  the cut's place wherever the proof reads the cut (every node below it
  made anew), and `Proof::new` drops what the root no longer reaches. A
  subproof a step duplicates (the other premise of a commutation over
  `&`, the box of a contraction) is shared by the arena, not copied.
- **What a step reads off the checker.** One pass of the checker with an
  observer (`Record`): whether each node absorbs through a `⊤` above, and
  the derived linear zone of every premise of `⊗`, Mix and cut. That pass
  is also the check of the proof the previous step made, so `eliminate`
  checks every proof it reaches without a pass of its own; `step` checks
  its result with a second pass.
- **The cases** (`Rewrite::reduce`), in this order: an axiom on the cut
  formula in either premise (the other premise takes the axiom's other
  literal in place of the occurrence, traced up through the rules to the
  axiom or affine weakening that introduces it, or the `⊤` that absorbs
  it: `rename`); in affine mode a weakened cut formula (the other premise
  erased and its formulas weakened in); a commutation into the first
  premise when its last rule does not introduce the cut formula, then
  into the second; and the key cases: `⊗`/`⅋` (two cuts on the
  subformulas), `1`/`⊥`, `&`/`⊕` (one cut on the subformula the `⊕`
  chose), `!` against `?w` (weakening), and `!` against a `?` step. The
  exponentials are the dyadic calculus's: the box `!A` goes up the `?`
  step's premise with that step, and what it meets names the case: a
  subproof that does not use `A⊥` (weakening, the box erased), a copy of
  `A⊥` that is its last use (dereliction: the box's content cut against
  the copy's premise), a copy whose premise uses `A⊥` again
  (contraction: the content cut and the box kept for the uses above), a
  `!` (a box entering a box), a rule of two premises that both use it
  (contraction: the box cut into both), any other rule (commutative).
- **Intuitionistic mode**: the classical reductions of an ILL proof are
  ILL proofs, and the one place the code had to choose was where a `⊤`
  absorbs the cut formula: an absorbed formula right of `⊢` goes to an
  absorbing premise with no goal yet (`absorber`); the first version took
  the first absorbing premise, and the tests' intuitionistic affine
  proofs made a `⊤` premise with two goals, which the checker refused
  (found before the first commit, fixed at the cause). Every proof made
  prints as the ILL derivation it is (README's example shows one with two
  cuts).
- **The one context-sensitive case**: an affine erasure whose erased
  premise absorbs through a `⊤` while the kept premise does not. The
  context the erased premise stands for is then what each place the
  proof reads the cut at gives it, which the checker's bottom-up zones do
  not say; the step specializes every path from the root to the cut by
  the exact conclusion it gives there (`specialize`, `above`, the
  top-down instantiation the derivation view makes) and weakens each
  place's own formulas. `an_erasure_weakens_what_each_place_gives` pins a
  cut that both premises of a `&` read, absorbing `c` in one and `d` in
  the other.
- **The command** `linlog eliminate PROOF` (`cli/src/eliminate.rs`):
  `check`'s flags, bounds, mode and output formats, and `--steps N`
  (`cut.steps`) and `--node-limit N` (`cut.nodes`); a line per step
  (`1. contraction on the cut of !a`) and the end (`cut-free proof
  of !a ⊢ a ⊗ a (intuitionistic) after 6 steps`, or `proof of … with 2
  cuts left after 2 steps: the elimination took the most steps allowed,
  2; --steps N allows more`) as one note that every format comments line
  by line, then the derivation of the proof reached; exit 0 at no cut
  left or at the steps asked for with `--steps` (or a settings file), 3
  at the default bound on steps with cuts left, at any other bound and
  at the time limit, 1 for an invalid proof, 2 for an error. `--format json` writes the
  library's record.
- **The record's JSON form** (written only, `Serialize for
  Elimination`): `{"version": 6, "proof": {…}, "steps": [{"cut": 9,
  "formula": 5, "case": "contraction", "above": 7}], "end": "cut_free"}`,
  or `"end": "refused"` with `"reason"`, the refusal in an error's form.
  Its level is its proof's, 6 for a sequent with cut formulas, as
  `wire.rs` now says (level 6 adds the cut and the record of its
  elimination); no level was raised.
- **Tests** (`core/src/proofs/cut/tests.rs`): every step of every proof
  with cuts the oracle's set holds (41a's `with_cuts` over the generated
  sequents of every commutative mode) and of 26 proofs made for the
  exponential cases (contraction, dereliction, a box entering a box,
  weakening, classical and intuitionistic, with and without affine
  weakening and Mix), taken one at a time: the same sequent with the same
  cut formulas, the same conclusion, the checker's acceptance in the
  mode, every cut on a cut formula's tree, and the end cut-free within
  5 000 steps, as `eliminate` reaches in the same steps; 1 423 proofs
  (206 classical, 205 with Mix, 206 affine, 199 affine with Mix, 304
  intuitionistic, 303 intuitionistic affine), 1 397 with nested cuts and
  242 with a cut on an exponential formula, 18 248 steps, every case met
  (commutative 9 469, axiom 3 283, `⊗`/`⅋` 1 794, `&`/`⊕` 1 315, `1`/`⊥`
  659, weakening 744, dereliction 572, a box entering a box 337,
  contraction 75). `any_cut_may_be_reduced` reduces every cut of every
  proof first, not only the strategy's; `a_bound_stops_with_a_proof`
  reaches the steps bound, the nodes bound (one below the largest proof
  the elimination of a contraction reaches) and the stop at every poll
  from the first to the fortieth, each ending with a proof the checker
  accepts that still has its cuts.

### Does every intermediate proof live over the same forest, with no new cut formula?

**Yes, for every case of the term language as it is, and the tests
assert it at every one of the 18 248 steps** (`made.sequent() ==
proof.sequent()`, the cut formulas included, and every cut node on an
occurrence where `Forest::is_cut_formula` holds). Why, case by case:

- **Key cases** cut the children of the occurrence cut, which lie in the
  same `A` tree, their duals in the same `A⊥` tree (`Forest::dual` is
  positional, so `dual(left(a)) = left(dual(a))`).
- **Commutative cases** keep the occurrence they were given and move the
  cut node above a rule; the cut formula's occurrence does not change.
- **Copies**: a dyadic copy repeats an occurrence of the unrestricted
  zone in the linear zone, which is a multiset, so a contraction needs
  no second occurrence: the box's content is cut against the copy's
  premise on the child of the box's formula, and the box itself stays
  in the arena, shared by every cut that uses it. The copy bound is an
  option of the search, never part of a term, so nothing of it is
  represented.
- **Boxes' contexts** live in the unrestricted zone `Θ`, which every
  place shares and which the checker computes as the least zone a
  subproof needs; a box that enters another box or is duplicated takes
  its context with it without any formula being added, and the `?` steps
  below the original cut still discharge it.
- **The axiom case** renames one occurrence of a literal to another
  literal of the same atom and sign, both of the forest.

What is not literally kept: node ids. A step makes a new arena, so a
`Step`'s `cut` and `above` name nodes of the proof the step was taken on,
not of the one it made; the page keeps the proof before each step to
draw and undo it.

### Decisions

- **The exponentials step by step in the dyadic calculus.** The term
  language has no `?d`, `?c`, `?w` and no unrestricted cut, so the step
  that meets a `?` step moves the box up its premise with it, and the
  standard names are given to what it meets (above). This keeps the
  steps as fine as the standard calculus's and the names a student
  expects, without a new node (an unrestricted cut would be a new
  variant of `Node`, a checker arm and a Rocq rule).
- **A shared cut node is reduced for every place that reads it, in one
  step**, as the node is one term; the derivation view unfolds it into
  several cut inferences, which the step reduces together.
- **The order of the cases**: axioms first (the cheapest, and what a
  student expects at a leaf), then affine erasure (weakening erases
  whatever the other premise is), then commutation into the first
  premise, then the second, then the key case; deterministic, so that
  `eliminate` and a client stepping by the strategy agree step for step
  (the test asserts it).
- **Checked at every step, not trusted**: `eliminate`'s check of each
  proof is the pass the next step needs anyway; `step` pays a second pass
  for its result. A refusal of the pass that checks the latest proof
  returns the proof before it with one step fewer, so the proof returned
  has always passed the checker.
- **The bounds** are `cut::Options::steps` and `nodes` (D16: named
  defaults, flags `--steps` and `--node-limit`, a line of documentation
  each); `Limits` bound each pass of the checker (memory and work) and a
  proof made past `memory_bytes` at 16 bytes a node is refused. The
  front ends: the command maps its flags; the bindings (41d) hold
  `Settings::cut` as JSON (`{"cut": {"steps": 10000, "nodes": 100000,
  "strategy": "topmost"}}`); a library caller passes `cut::Options`.
- **Exit statuses of `eliminate`**: 0 at no cut left and at a steps
  bound the user set (what was asked: `--steps 1` is one step), 3 at the
  default steps bound with cuts left, at another bound or at the time
  limit, as an unknown is elsewhere; the proof reached is printed in
  every one of them. The first version answered 0 at the default bound
  too, which read as done with cuts left; the supervisor's review found
  it, and `the_default_bound_on_steps_is_unknown` and the command test
  in `cli/tests/cli.rs` pin the statuses.
- **Quantifiers (D17)**: a step reads nodes and occurrences only; a
  first-order cut's key case (`∀`/`∃`) will substitute the witness term
  into the subproof, which is a renaming of the arena's terms as
  `rename` renames an occurrence, so it joins `Rewrite::reduce` as one
  more pair of the key match, with an eigenvariable check in the
  checker that it does not need here.

### Departures from the design

- **`Strategy::Topmost`, not `LowestFirst`** (departs from the design,
  `plan/notes/api.md` 10.6): the same rule (the lowest arena index is a
  cut with none above it), named for what a reader of a derivation sees.
- **`step` and `eliminate` take the mode as their second argument and
  return `Result`** (the design's own table left this to the step); the
  `Reduction` holds a `Step` (`cut`, `formula`, `case`, `above`) rather
  than `cut` and `case` beside the proof, so that `Elimination::steps`
  holds the same values; `End::CutFree` rather than `Normal`.
- **The record** `{"version", "proof", "steps", "end"}` (improved, as the
  brief allowed): each step has `formula` and, where there is one,
  `above`; `end` is `cut_free` or `refused` with a `reason` in an error's
  form.
- **No `Proof::cut` composing two goal proofs**: the tests compose proofs
  through the interactive cut and the search, as 41a's `with_cuts` does,
  and nothing else asked for it.
- **The command takes no `--cut NODE`**: choosing the cut is the page's
  (the library's `step` with `at`); the command's one step is `--steps
  1`.

### The panel

Each member worked in a fresh context with the code, the claim and the
rules file, not this session's reasoning, briefed with conduct.md's
rules on cores (4 to 7), memory (8 GiB per scope) and bounded runs, and
told to report only what breaks soundness, termination, the conclusion
or a stated requirement; one round each. The claim they judged: `step`
on a proof the checker accepts in a mode with the cut, at any cut node
or the strategy's, returns a proof over the same sequent with the same
conclusion that the checker accepts, never `Error::Reduction`;
`eliminate` by `Topmost` ends cut-free, and within the bounds ends with
a refusal and a checked proof with its cuts; the cases are named as
usual; no integer wraps and every bound holds.

- **The argument** (Fable 5.1 at `high`): **one finding, which held.** It
  derived from the code that a `State` stands for a set of dyadic
  sequents in which every rule of the checker is monotone, so a step may
  put in a cut's place anything that proves a superset, and checked every
  case against `pass.rs`: the axiom case, the key cases (the two trees of
  one shape, the sides of `a` and `dual(a)` opposite at every depth in
  intuitionistic mode, so each new cut keeps one goal), the commutations
  (the absorbing premise forced by R1 and R2), the exponentials (`uses`
  is the checker's least `Θ`, `wrap` proves exactly its premise's zone),
  the specialized erasure (an exact node's instantiated zone equals its
  derived one), and termination of `Topmost` by Gentzen's double
  induction region by region (the arena keeps its order and a step
  appends, so the lowest cut always has cut-free premises). The finding:
  `rename` remade a node on the traced path with `map_premises`, which
  remaps every premise slot naming a remade node, so a `⊗`, Mix or cut
  reading one shared subproof in both slots had the literal renamed in
  both where the cut consumed one; witnesses: a proof with `Mix(0, 0)`
  in classical mode with Mix and an affine `⊗` of one absorbing subproof
  twice, both of which the checker accepts and on which `step` returned
  `Error::Reduction`. **Answer**: holds; reproduced (both witnesses
  failed on the frozen tree), fixed at the cause in "Fix the panel's
  findings in cut elimination": the trace records, at a rule of two
  premises, which premise its path entered, and only that slot is made
  anew (`&` keeps both, since both branches hold the occurrence).
  `the_axiom_case_renames_one_premise_of_a_shared_pair` holds the two
  witnesses and the counterexamples' first.
- **The counterexamples** (Opus 5.5 at `high`): **one finding, the same
  defect, which held.** About 330 000 random interactive proofs with cuts
  (cuts on members, on wrapped members and on random formulas up to depth
  4, goals closed by the search), each in every one of the six modes
  whose check it passes, about 28 000 of them also as hash-consed DAGs;
  every cut reduced by `step` with `Some(id)`, `Topmost` to the end with a
  cap of 3 000 steps (never reached: the longest elimination took 603
  steps, the largest proof 1 277 nodes), `eliminate` compared with
  stepping by the strategy, `eliminate` with one step and one node too
  few, and random orders of cuts up to 400 steps; every step checked for
  the checker, the sequent, the conclusion and the step's `cut` and
  `formula`; every case met (in its last two runs alone about 2 000
  contractions, 13 500 derelictions and 5 600 boxes entering boxes). The
  sixteen failures were all the axiom case and all `rename`'s defect: a
  node that a split on the path chose and another split on the path left
  unchosen was remade under both. Witnesses: a 16-node proof of `a, !a ⊢
  a ⊗ a` with the cut formula `!(a ⊕ a)` (a node read by both `⊗`s under a
  `&`), on which `linlog eliminate` exited 2 in all six modes; a proof
  with `Mix(3, 3)`; and a chain of 72 steps at chosen cuts from a tree,
  whose sharing the elimination's own commutations over `&` made.
  **Answer**: holds; reproduced, and the same fix answers it: after it
  the 16-node proof and the `Mix` proof eliminate cut-free in every mode
  their check passes (`linlog eliminate` on the files), the two shorter
  intermediates step at the nodes that failed in all six modes, and the
  72-step chain completes (a scratch test over the reviewer's files, not
  kept); the 16-node proof is in the regression test, in all six modes.
- **The integers and limits** (Sonnet 5.5 at `high`): **one finding,
  which held, and a minor one.** Every bound swept over every proof of
  the test set in a debug build (30 163 outcomes: the proof returned
  always checked, kept its conclusion and had its cuts at a refusal), and
  through the command: the steps bound (a chain of 20 000 axiom cuts
  stopped at 10 000, exit 0), the node limit, the work and memory limits
  (refused at the first check without a proof, exit 3; later with the
  proof before, which checks), the time limit, an invalid proof (exit
  1), a family of nested box contractions (cut-free in 6 157 steps,
  17 MB). The finding: `specialize` bounded its places by `cut.nodes`
  but not the weakenings it makes for each (places × context), held its
  conclusions without charging them to `memory_bytes` and never asked the
  stop, so a proof file of 15.6 KB (a cut that 15 nested `&`s read under
  2¹⁵ contexts, 5 000 formulas of context) took 5 GB and 7.4 s inside one
  step against a limit of 1 GiB and a time limit of 2 s, and 28 KB was
  killed by its scope at 8 GB; past 2³² nodes `Rewrite::push` would have
  aliased ids and panicked. **Answer**: holds; reproduced and fixed at the
  cause in the same commit: `specialize` counts every node it will make
  (its places and their weakenings) against `cut.nodes` before making
  any, charges each conclusion it keeps (twice, the place's and the
  index's) and each node to `memory_bytes`, and asks the stop every
  4 096 places; `Rewrite::push` marks an arena past what a `NodeId` names
  and the step is refused as `Refusal::Index`. The reviewer's file now
  ends at the memory limit with a peak of 1.06 GB in 1.8 s (exit 3);
  `a_specialized_erasure_is_bounded_inside_the_step` reaches the stop,
  the node bound and the memory bound inside the step on the same shape
  (13 nested `&`s). The minor point: `Limits::work` bounds each check of
  an elimination, not the call, and `limits.rs`'s table had no row for
  the elimination; **answer**: the table now has its row, saying that
  `memory_bytes` and `work` bound each check and where the stop is
  asked; the bound on the call's work is `cut.steps` times a check's, as
  the elimination's documentation says.

One more thing happened during the panel: a member wrote its scratch
files into this session's scratchpad and replaced the helper script this
session ran its builds through; one release build of the command
(`cargo build --release -p linlog-cli`) ran through the replaced script
in the main checkout's directory and wrote its binary to the main
checkout's `target/`, no source touched. Every other run of this session
was in `../linlog-step41b` (the gates' logs compile
`linlog-step41b/core`); the helper was restored under another name.

### Gating

Every code commit passed the gate (`systemd-run --user --scope -p
MemoryMax=8G -p MemorySwapMax=0 taskset -c 4-7 nix develop -c gate`, in
`../linlog-step41b`). "Add cut elimination on proof terms and the
eliminate command" passed it at the tree of "Bless the behaviour lock
for the eliminate command", which holds it and the lock its help moved
(only `error-no-command`, the list of commands, which is blessed in that
commit of its own): clippy, the tests, both `cargo hack` runs, and the
ratchet, every journey within its ceiling (the largest change
`check-qbf-20-2`, +1.17 %; its one note, never a failure, the cache
misses of `check-qbf-20-2`). "Fix the panel's findings in cut
elimination" passed it at its own tree, with the ratchet's counts as
before (the same note on `check-qbf-20-2`'s cache misses). "Number the steps
eliminate prints" passed it at its own tree too: the first `nix flake
check` failed its `conventions` check, which refuses `step N` and `steps
N` in README and in comments (the words that would name the plan), on
the command's lines `step 1: …` and the example `--steps 2`; the lines
are now numbered (`1. contraction on the cut of !a`) and the flag
written `--steps=2`, the check unchanged. The second `nix flake check` failed its
`doc` check on a redundant explicit link target in the elimination's
docs (`rustdoc::redundant_explicit_links`, denied); "Drop a redundant
link target in the elimination's docs" removed it and passed the gate,
as did "Answer unknown when eliminate stops at its default bound on
steps" (the supervisor's finding, under Decisions), each at its own
tree.

### The target sets

From a release build of "Bless the behaviour lock for the eliminate
command", the focused set and the net set ran one after the other in one
detached unit (`step41b-targets`) under the machine lock, the scripts'
own streams on cores 2 and 3. `linlog-bench summary --against
bench/targets/cut.csv bench/targets/elimination.csv --counters`: no
verdict differs, and of the 145 problems both decide alike on one
thread, 0 differ in `nodes`, `splits`, `memo_hits`, `memo_entries`,
`links` or `tests`. Against `bench/targets/net-cut.csv`,
`net-elimination.csv`: no verdict differs, 0 of 64. The panel's fixes
touch `proofs/cut/` and a table of documentation alone, which no search
reads, so the sets were not run again. The search returns only cut-free
proofs: it never reads `proofs::cut`, and 41a's
`a_forest_with_cuts_is_searched_without_them` passes in every gate.

### The closed enums' new variants, for step 45

None: this session added no variant to `Node`, `Rule`, `Term`, `Kind` or
their kin. Marked enums gained `Refusal::Elimination` (code
`elimination_limit`), `Phase::Eliminate`, `Error::NoCut` (`no_cut`) and
`Error::Reduction` (`reduction_rejected`). New public items:
`proofs::cut::{step, eliminate, Options, Strategy, Case, Step,
Reduction, End, Elimination}` (all marked), `limits::Bound` (marked),
`Settings::cut` and `Settings::with_cut`, `Proof::is_cut_free`. The
changelog lists them under `[Unreleased]`.

### What the next sessions must know

- **Nets (41c)**: the term elimination's cases are the reference for the
  net steps' names (`Case` in snake case on the wire: `axiom`,
  `tensor_par`, `one_bot`, `with_plus`, `weakening`, `dereliction`,
  `contraction`, `box_into_box`, `commutative`); nets have no commutative
  cases, and their exponential steps (a box entering a box, duplication,
  erasure) correspond to `box_into_box`, `contraction` and `weakening`
  here. `cut::Options` holds `steps` and `nodes`; a net elimination can
  take the same options value (its `nodes` then counting links or
  vertices, said so) rather than a second one.
- **The page (41d)**: the bindings need two calls, `step(proof, mode,
  at?, settings)` returning the proof made and the `Step` (`cut` and
  `above` are nodes of the proof given, which the page has drawn; to
  highlight a cut inference the page maps the node to its inferences in
  the derivation it drew, one or more if the node is shared), and
  `eliminate(proof, mode, settings)` returning the record. Undo is
  keeping the proof before the step. `Error::NoCut` answers a node that
  is no cut. The derivation of a proof with cuts already draws them
  (41a), two-sided in intuitionistic mode.
- **A step's result is a new arena**: node ids do not carry over from a
  proof to the next; the cut formulas' occurrences (`Step::formula`) do,
  since the forest is the same.
- **Topmost and termination**: `Strategy::Topmost` terminates (the
  panel's argument below); a caller who picks a cut that has cuts above
  it may move a cut above another (a commutation over `cut`), which the
  strategy never does: a client that lets a student pick freely can
  cycle, and should offer the strategy's cut when the student asks for
  "the next step".

### Open

- `linlog interact` prints a goal whose first formula is a cut formula's
  hypothesis as `opened goal 2:, 1: !a ⊢ 0: a ⊗ a` (a stray comma, from
  41a's goal line), which README's new example shows as it is; a
  follow-up for the goal line, not changed here.
- The CLI has no flag to name the cut a step reduces (the library's `at`);
  the page has it.

### Checklist

- [x] One step in the library (`proofs::cut::step`) that reduces the cut
  the caller names or the strategy's, returning the proof and the `Step`
  (the cut, its formula, its case, the node acted at), and elimination to
  the end (`eliminate`) by `Strategy::Topmost`, documented, in the
  classical, affine, Mix and intuitionistic modes (and intuitionistic
  affine); the ordered modes have no cut.
- [x] Bounded by default: `cut::Options::steps` (10 000) and `nodes`
  (100 000), named constants with flags `--steps` and `--node-limit`
  and keys `cut.steps` and `cut.nodes`; a bound ends with
  `Refusal::Elimination` and the last proof checked, with its cuts
  (`a_bound_stops_with_a_proof`, `a_specialized_erasure_is_bounded_inside_the_step`).
- [x] Tests on generated proofs with cuts in every commutative mode
  (1 423 proofs, 1 397 with nested cuts, 242 with a cut on an
  exponential formula, 18 248 steps): every step keeps the sequent and
  the conclusion and passes the checker, every elimination ends
  cut-free; every case met; any cut may be reduced first.
- [x] The question of the forest answered with evidence: every
  intermediate proof lives over the same forest with no new cut formula,
  asserted at every step; copies, boxes and commutative cases explained.
- [x] `linlog eliminate`, one step (`--steps 1`) or to the end, text, the
  derivation views and JSON; the record at wire level 6 as `wire.rs`
  says; README's usage section with real output that
  `cli/tests/readme.rs` runs; the guide for AI models updated and its
  tests passing; the changelog under `[Unreleased]`.
- [x] The panel on a frozen tree (Fable 5.1, Opus 5.5, Sonnet 5.5, each
  at `high`): two findings held, both fixed at the cause with tests,
  every finding answered above.
- [x] Both target sets against `cut` and `net-cut`: no verdict differs,
  every decided row's counters equal (145 and 64); the search cut-free.
- [x] Every code commit gated; the behaviour lock changed only in a
  commit of its own.
- [x] `nix flake check` at the session's last commit, under the machine
  lock (exit 0; the first two runs failed on `conventions` and on `doc`,
  both fixed at the cause, above).
- [x] `.claude/rules/` amended (`core-proofs`, `core`, `cli`), README,
  the guide and the changelog true.
- [x] No new variant of a closed enum (`Node`, `Rule`, `Term`, `Kind`);
  the marked enums' new variants listed for step 45.
