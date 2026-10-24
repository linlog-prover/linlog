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
