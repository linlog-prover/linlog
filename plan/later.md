# Later work: the sketches, what is deferred, and the follow-ups

Step 17 (`reports/17-assessment.md`) assessed the candidates below, and
the author decided on 2026-10-03; the steps from 18 are in `README.md`'s
table. The sketches stay because the prompts point at them: a sketch is
the detail of its step, corrected where the table below says so. The
follow-up lists at the end are assigned to steps, each list under its
heading.

## Where each candidate went (2026-10-03)

| candidate | step | what step 17 changed in it |
|---|---|---|
| Code audit and refactoring | 23 (what every session reads), 26 (the focused engine), 28 (the audit, the API and data model, the command, the harness, the documents; once, before the release, D23) | the audit is section 1 of step 17's report; the API may change freely (D18) and keeps the place for quantifiers (D17) |
| Configurable output, no font | 22 | the curryst limit is reported upstream and closed as not planned (curryst issue 19), so the Typst tree of linlog's own is built, not weighed |
| Net-engine pruning and routing | 35 | no verdict to gain (the focused engine is within a factor of three on the net engine's case); kept by D19, each row to be earned by a measurement; the cubes' defect on a pool is step 19's |
| MELL proof nets with boxes | 33 | – |
| Essential nets for IMLL | 35 | as an engine only where it beats the embedding; as a drawing regardless. Moot's paper is of 2004 (arXiv 2008) |
| The focused inverse method | 37 | two cases left (many hypotheses, Mix); stays an option if it wins no row |
| The !-Horn fragment through Petri-net reachability | 27 | reshaped: an engine of linlog's own with coverability for affine mode. KReach has not changed since 2020; the maintained tools (verifypn, Mist) are GPL-3.0 and could only be called, never linked |
| Cyclic MLL and the Lambek calculus | 36 | – |
| First-order linear logic | 38 | wanted (D17). First-order MALL is NEXPTIME-complete (Lincoln and Shankar 1994 for membership) |
| A Rocq library of linlog's own | 31 (after the release, D23) | named `linlog`, under `rocq/` (D20). The Mix reduction is machine-checked in Yalla without cut (`mix2_to_ll`); Yalla's general Mix is on its untagged master only; a Lean target exists now (`leanprover/cslib` has classical linear logic with units), deferred until after step 31 |
| Certified refutations (2026-10-04) | 31 for the cheap ones (the search's invariants, the classical reading, classical "not valid" over `Prop`); the rest are goals for later | a negative answer certified by a lemma proved once and a small certificate; see the section of that name |
| MALL proof nets | dropped | non-canonical or exponentially large: a display feature without a use |
| A batch mode for the CLI | 24 | with LLTP input and the draft of the header report |
| Ordinary logic through its embeddings | 25 (done) | the layer; termination on the image of a translation is deferred and assessed in step 25's report (the literature terminates on the intuitionistic side: Dyckhoff's LJT, loop-checked LJ): a loop check would refute 35 of ILTP's 274 problems, a step of its own if the author wants it; the follow-ups under "Follow-ups: ordinary logic" |
| The web front end | 32 (after the release, D23) | the bindings (`linlog-web`) in the workspace, the client in a repository of its own under the organization (D22) |
| Problems from practice | 27 | the coverability suite of `blondimi/qcover` (176 instances, real non-theorems). Deferred: Model Checking Contest nets beyond the 76 LLTP used, and planning domains, for which no collection in linear logic exists. Dropped: Granule's synthesis benchmarks (graded signatures with data types, few of them propositional ILL) and llprover's examples (one file of 70 lines without a licence) |

New from step 17, sketched in its report's section 5: a call that keeps
its limits (steps 18, 19 and 20), the defaults a user meets and why a
sequent is unprovable (21), cut and cut elimination (34), a release
(30), the wrong LLTP headers (drafted in 24, the author reminded in 30);
new on 2026-10-03, a comparison with the other provers (29, D23).

Deferred beyond the table, with the reason: a search that can be
suspended (an explicit stack in the focused engine; decided by what
step 34 measures under wasm); a Lean certificate target (after 31); a
per-worker proof arena, the duplicated exploration of `&` premises, a
thread sanitizer (no measurement asks); sharing more among
interchangeable sequents, the order of a split search's members, the
intersection for `&` (no target asks); the net engine on a sub-forest
(no client asks). Dropped by step 27, whose Horn engine takes the nets
by default: the restart of a copy-bound level from the frontier, the
tuning of the unit of work (the default's contract at the limit), and
the throughput of short clause bodies (`TCPcondis`), each of which only
served the focused engine on nets. Dropped before: the member list, `OccSet` and link-time optimisation among the
constant factors; Matsuoka's 3D-Matching family.

## Code audit and refactoring

Requested by the author on 2026-09-30. Sixteen steps by separate sessions
built the code, each reviewed for its own correctness and none for the
whole: read the workspace as one maintainer would and put it in order,
changing no behaviour. The audit first, as a ranked list of findings, then
the refactorings the list justifies, each a commit of its own. Step 15
leaves it a second list to start from: the hot spots its profile showed
and it did not take (`plan/reports/15-performance.md`), which are
changes of representation with a measured share of the time behind them.

What to look for: modules grown past what a reader holds (the focused
engine's `mod.rs` is over 2 000 lines, the interactive state 1 500, the
checker and the derivation view over 1 100 each); things done twice (lock
helpers, the split and Mix enumerations, the derivation functions of the
CLI, the symbol and label tables of the exports); the public surface
(what is `pub` and need not be, names and argument orders that differ
between neighbours, `#[non_exhaustive]` and error types applied unevenly,
options types against D15); the crate-wide `#![allow(dead_code)]` and
`#![allow(unused_variables)]` in `core/src/lib.rs`, kept "while things
are scaffolded", and what they hide once removed; invariants that live
only in `.claude/rules/*.md` and could be a type, a debug assertion or a
test; tests in excess of what they pin or missing for a stated behaviour,
and the suite's running time; feature gates and what each combination
really compiles; dependencies against their use; documentation that has
drifted from the code (doc comments, the rules files, CLAUDE.md, README).

What must hold: every test and every flake check passes after every
commit; the JSON formats and the pinned snapshots do not change; on one
thread the engines' counters on the benchmark target set are identical
before and after, which is the regression oracle a refactoring of a
search engine needs, since the search is deterministic; the two baselines
stay comparable. A change of behaviour that the audit finds necessary is
reported, not slipped in. Where it belongs in the order, and what it
should settle before new engines are written on top, is for step 17 to
say.

## Configurable output, and no font in the LaTeX and Typst output

The author's requests of 2026-09-29, which D15 records as a decision for
every later step; this item applies it to what steps 10 and 11 built.

First the fix: the standalone LaTeX and Typst documents set a font
(`eulervm` in `export::latex`'s preamble, `#show math.equation: set
text(font: "Euler Math")` in `export::typst`'s page setup, both from step
11's item 1a). Remove both: a user pastes linlog's output into a document
and wants it to look like the rest of that document, so the program never
chooses a font there, standalone or not. Re-bless the snapshots, drop
`eulervm` from `modules/export.nix`'s TeX Live and keep Typst's
`--font-path` only for what still needs it (nothing, once no document
names a font; Typst's bundled fonts then serve the check). Euler stays
where linlog draws (SVG, the web).

Then the options. Each export gets one plain-data options value with
`Default`, `Clone`, `PartialEq` and serde behind `serialize` (the SVG
`Style` is the model; the LaTeX and Typst emitters take a `Form` only
today, which becomes a field). What a user might vary:
- the shape of an open goal: vertical dots over the sequent (today),
  the bare sequent, a marked leaf (`?`, a name), a dotted or dashed line
  where the target can draw one;
- the rule-label convention: upright `L`/`R` with subscripts (today),
  `\multimap_L`-style, no labels, or a user table;
- turnstile alignment in two-sided LaTeX trees on or off;
- whether the CLI's verdict and statistics comments are emitted in a
  source-file format, and whether the LaTeX standalone class is
  `standalone` or `article` with a preamble the user supplies;
- the curryst import (version) and the ebproof options;
- for SVG: the font (family name and its advance table, Euler Math the
  default, a monospace preset with one advance for a viewer that has no
  math font), the sizes, gaps and colours `Style` already has, a dark
  preset, and per-formula ids on or off (the export follow-ups);
- for the text renderer: the bar character and the gap between premises;
- for the certificates: `rocq::Options` exists (`lemma`, `prelude`); the
  CLI gets `--lemma` and `--prelude` through the same `--style` surface,
  and `interact` a way to certify a finished session (`show rocq` cannot,
  since `show` draws the user's partial derivation; the certificate is
  `Interactive::proof()` then `proof.derivation()`, so a `proof rocq`
  spelling or a `certify` command);
- for the interactive session: the message language of `Refusal` if the
  web front end localises.
Presets are named values of the options type (`Style::dark()`), not code
paths. The CLI maps `--style KEY=VALUE` flags or a `--style-file` (JSON,
the options' serde form) onto the options; the web front end holds the
JSON in its settings and sends it back with each request; an editor
plugin or a notebook reuses the same JSON. The step that does this
records in `.claude/rules/core-export.md` that a new export option is a field,
never a constant, and its report says how each front end sets each
option. Opus 5.5, xhigh (it wrote these exports in steps 10 and 11).

## Net-engine pruning and routing for repeated literals

After the performance pass on the focused engine (step 15), the net
engine's turn: leaf symmetry breaking for pure `⊗` and `⅋` trees of equal literals, a per-atom
balance over the `⊗`-skeleton components of a partial structure (the net
engine's analogue of the focused engine's split counts), and the sound
variant of symmetry breaking for equal compound conclusions (keys under
roots no symmetry moves; the spec's first-literal key is unsound across
groups, as step 6's report shows). Step 14's numbers
(`plan/reports/14-benchmarks.md`, "Focus against net") give the targets:
the Partition table (4.6 s and 3.7 s against 6 ms on the focused engine)
and the MLL 3-Partition at bins of five (56 s against 18 µs; six bins
stay over 60 s at every thread count, though the cubes scale 13× at
sixteen threads). They also
show that literal multiplicity is the wrong routing feature: the net
engine wins by four orders of magnitude on literals repeated three or
four times across conclusions (`wide-m3`, `wide-m4`) and loses as badly
on equal literals inside one pure `⊗` or `⅋` tree, and the reviewer's
counterexample at multiplicity four kept `NET_MULTIPLICITY` at two. So
the dispatch should route on "no two equal literals under one pure tree",
or the leaf symmetry break should remove that weakness and the threshold
rise; decide by the harness. Fable 5.1, xhigh.

Step 15 changed the premise (its report, "What is left for the net
engine and the inverse method", and the planning session's check of
2026-10-02): the focused engine now refutes the Horn encodings in
microseconds and proves `wide-m3` at 30 and `wide-m4` at 28 in 0.15 ms,
as fast as the net engine, so these are no longer cases where the suite
is slow, only where that engine is. What the net engine alone still does
is width at the default recursion limit (`wide-m1` at 2 048: a free
split costs the focused engine a level per link) and, with it, the net
itself as a result. So the candidate is now worth what nets are worth as
a search vehicle in their own right (teaching, the canonical proof
object, the cubes' near-linear speedup), not what it gains the suite in
verdicts; the second baseline's `engines` runs say whether `net` should
stay a default route at all, and a loop for chains of free splits in the
focused engine would take the last case.

The second baseline's `engines` runs (`plan/reports/16-baseline.md`,
`bench/COMPARISON.md`): the focused engine is within a factor of three of
the net engine on the wide sequents from 8 to 1 024 literals, faster from
256 on (`wide-m3` at 256: 3.7 ms against 9.7 ms), microseconds where the
net engine takes seconds or times out on every Horn encoding, and loses
only at 2 048 literals, where it meets the recursion limit (the net
engine 0.64 s). So `net` as a default route earns its place only on that
width; the routing question is whether a loop for chains of free splits
takes it from the focused engine, after which the net engine is the net
as a result, not a route to verdicts.

Step 28's audit (F153) measured the engine quadratic on `wide-m2`
(instructions ×3.96 per doubling, 4.38 G at 1 024 pairs): the exact test
every `period`-th link searches the whole structure (38 %), `choose`
recounts every unlinked literal's partners per decision (22 %), and
sequentializing searches a sub-net per stage (40 %). Area 3.3
(2026-10-13) tried the cheapest part, the search without the fill of
its arrays per search, and reverted it: 4.6 % of `search-wide-m2-256`'s
instructions, no measured time. Left, each to be earned by the harness: a
test period that grows while tests pass, which changes
`Statistics::tests` and so the behaviour lock's `--stats` lines (a
commit of its own); partner counts kept per literal, which keep `tests`
but must follow `joined` and the common-ancestor rule, which one link
changes for literals far from it; the bridges a stage
found kept for its sub-nets (a bridge stays one in a subgraph) with the
split's sides assigned by searching from both premises in turn and
stopping at the smaller, which keeps the splitting `⊗` the smallest id
and so the printed derivation.

## MELL proof nets with exponential boxes

Extend `nets` (step 5) with `!`-boxes and the `?` nodes (dereliction,
contraction, weakening as net nodes; or the "generalized ?" node with
auxiliary doors), correctness as the Danos–Regnier criterion applied at
each box depth with boxes contracted to single nodes (Guerrini–Masini 2001
for the parsing view), sequentialization through boxes, and the SVG drawing
with boxes as rectangles. Search stays with the focused engine; the value
is the representation (display, conversion, correctness). Fable 5.1, xhigh.

## Essential nets for IMLL

The spec's IMLL-Net: Lamarche's polarized structures using the D1
polarization from step 8, correctness by directed acyclicity plus the
dominator condition (Murawski–Ong), incremental search with a transitive
closure bit-matrix and an undo log (Moot 2008). Compare against the
embedding route of step 8 on the benchmarks of step 14 before making it the
default for IMLL: step 8 showed the verdict needs no essential-net
condition (every sequentialization of a classical net of an IMLL sequent
is intuitionistic), so this is a performance alternative, and its report
sketches the engine as the dominator condition added to the net engine's
`complete` branch. Fable 5.1, xhigh.

## The focused inverse method

The spec's second engine for MALL and the semi-decision alternative for
MELL/ILL when Θ is large: forward saturation from initial sequents in the
subformula closure with subsumption indexing (feature vectors as in Schulz
2013). A new `search::inverse` engine with its own dispatch row driven by a
heuristic (many hypotheses, small goal) or `--engine inverse`. Fable 5.1,
xhigh.

What is left for it after the second baseline: Mix (`mix` at ten pairs
146 s, at eleven over 1 200 s, the same `3^n` memo lookups on every
thread count), and the Petri nets beyond the forward search (1 594 of
3 137 at the 5 s limit under the default).
The intuitionistic library outside the nets ends at the copy bound, not
at the time limit (727 problems), which is a question of `--copies`, not
of the direction of search.

## The !-Horn fragment through Petri-net reachability

Detect the fragment, build the net, and either call an external reachability
tool (KReach) through the CLI or implement coverability for the affine case.
Only worth it with the ILLTP Petri-net problems from step 14 as the
benchmark: of the library's 3 137 Petri nets the first baseline decides
210 in 5 s; 982 time out, 171 stop at the copy bound and 1 774 at the
recursion limit, the 63-member split limit or the harness's kill (a
search that misses its stop), all of which step 15 addresses, so assess
after step 16 what is left for a reachability route. Opus 5.5, xhigh.
After step 15 the nets no longer stop at a limit but run into the time
limit (69 of its 113 sampled problems), and forward chaining by
`--bias factors` with a raised copy bound proves 48 of 109 where the
default proves 11: the focused engine under that bias is most of what a
reachability route would add for the provable nets, so what is left for
this candidate is refutation (a net whose goal is unreachable ends at
the copy bound, not at `Unprovable`) and the nets beyond the bound.
The second baseline measured it on the whole library: the default decides
1 520 of the 3 137 nets within 5 s (210 in the first baseline), the
backward search alone 442, the forward search alone at 30 copies 1 576;
1 594 end at the time limit, 2 at the copy bound, 7 are killed (the
GPPP-1000 nets, whose forward search misses its stop) and 14 are proved
by the search in under 2 s and counted as crashes, since the harness's
check of their proofs runs out of 12 GiB (the review of step 16; the
focused engine's follow-ups have both). Every LLTP net is a theorem, so
the library cannot show refutation; the nets beyond 5 s are what a
reachability route would be measured on.

Step 27's first session built the engine (`core/src/search/horn/`,
`Engine::Horn`): reachability by an explicit-state search of the
markings, the proof read off the firing sequence, a refutation when the
reachable markings are exhausted, and the default for Horn programs with
a clause under `!` in linear mode (`plan/reports/27-horn.md` has the
measurement). Its second session added coverability in affine mode (the
backward algorithm, with a trie of its elements), the net's state
equation in both modes (a revised simplex proposes Farkas weights, an
exact check in integers confirms them), dropped dead transitions and a
backward reachability search beside the forward one, made the Horn row
take affine mode too, and fetched the qcover suite with a reader for
its `.spec` format. Its follow-ups are under "Follow-ups: the Horn
engine".

## Cyclic MLL and the Lambek calculus

A non-commutative mode: planar axiom linkings in the net engine (links may
not cross in the cyclic order of literals), no exchange in the derivation
view and exports, and the Lambek restrictions (no empty antecedent, the two
divisions). Fable 5.1, xhigh.

## First-order linear logic

Quantifiers and predicates with terms (the author's question,
2026-10-03: is it intractable?). It is not, in the sense that the
quantifiers cost less than the connectives linlog decides already
(`proof-search-specifications.md`, "First-order fragments", has the
sources): first-order MLL stays in NP, since without contraction every
quantifier is instantiated once and the instance is found by
unification at the axioms; first-order MALL is decidable and
NEXPTIME-hard where the propositional logic is PSPACE-complete; and with
exponentials the logic is undecidable, as propositional LL is, so the
copy bound and its three-valued answer carry over unchanged. The
specification said not to build this before the propositional engines
pass their tests, which they now do.

What it means for search is known technique: `∀` is invertible and takes
a fresh eigenvariable, `∃` is a focused rule that takes a metavariable,
an axiom unifies its two atoms, and the eigenvariable condition is kept
by Skolem terms or by levels on the variables. Moot's LinearOne
(first-order MILL by proof nets with unification) and Chaudhuri and
Pfenning's focused inverse method are the working designs, and
llprover reads first-order input.

What it costs here is that it goes through every layer, which is why it
is a candidate and not a follow-up:

- **The data model.** Atoms become predicates applied to terms, formulas
  get binders, and the sequent's arena, its negation normal form, the
  parser, the printer and the JSON change with them. Without
  exponentials the occurrence forest survives, since an occurrence is
  instantiated once on a branch and a substitution beside the forest
  says with what; under `!` and `?` every copy needs its own instance,
  which the copies of the dyadic context have to carry.
- **The engines.** Bindings are made and undone with the search (a
  trail). What step 15 built has to be gone through piece by piece:
  counting atoms prunes by predicate symbol only, classes of
  interchangeable occurrences and failures keyed up to renaming have to
  treat open terms correctly, and a goal that shares a metavariable
  with its sibling is no longer independent of it, which the `&`
  premises and the cubes of the parallel runtime assume. The net engine
  gains unification on its links and loses nothing else.
- **Proofs.** Proof terms carry witnesses, the checker checks them and
  the eigenvariable conditions, the derivation views, the interactive
  rules (where the user gives a witness or leaves it open) and the four
  exports follow. NanoYalla is propositional, so certificates wait for
  the Rocq library of linlog's own, which would have to be planned with
  quantifiers from the start if this candidate is wanted at all.
- **Problems.** LLTP is propositional and no first-order library for
  linear logic is known; the problems come from use: linear logic
  programs, planning and Petri nets with parameters, and categorial
  grammars, where the Lambek calculus embeds in first-order MILL (which
  bears on "Cyclic MLL and the Lambek calculus": the embedding may be
  the cheaper way to that candidate).

Several steps, the data model first and by itself, since everything
else stands on it and it must not slow the propositional case, which
both baselines pin. The question for the author before any of it is
whether research or teaching needs quantifiers; if they do, the audit
should know before it settles the types.

## Second certificate kernels: a Rocq library of linlog's own, NanoYalla kept for compatibility

Left open by step 12 (`plan/reports/12-certificates.md`, "Open questions").
The shape is the author's, of 2026-10-02: keep the NanoYalla export as it
is, for compatibility, with its refusals, and write everything new as a
library of linlog's own, idiomatic and by current practice, rather than
growing a development of 2021. (The planning session had first
recommended a fork of NanoYalla that adds kernels beside the unmodified
one; it stays below as the smaller alternative.) Today a certificate
exists for classical proofs only:
Mix and affine weakening are refused, and an intuitionistic proof is
certified as the classical proof of its one-sided sequent, not as
`Γ ⊢ A`.

The facts, read from the sources on 2026-10-02 (they correct step 12's
report in one point). NanoYalla is Click & coLLecT's `nanoyalla/`
(LGPL-2.1, unchanged since 2021): `nanoll.v`, 48 lines, the trusted
definition of cut-free one-sided LL, and `macroll.v`, the proved
positional layer (`_ext` rules, `ex_perm_r`). Yalla (olaure01/yalla,
LGPL-3.0, active, on opam as `rocq-yalla`, not in nixpkgs, needs OLlibs)
is the library of meta-theory behind it; its `microyalla/nanoll.v` is the
same definition up to binder syntax, and its `microyalla/nanoill.v`, there
since 2019, is a two-sided ILL kernel with exactly linlog's connectives
and rules, exchange by adjacent transposition, and no imports: it needs
neither full Yalla nor OLlibs, which step 12's report assumed. What it
lacks is a positional layer. Full Yalla has a general Mix rule
(`pmix`) and proves Mix-provability of `Γ` equivalent, with cut, to
provability of `?(⊥⊗⊥), Γ` (`ll_fragments.v`); no nano kernel has Mix.
Nothing in Yalla has general weakening.

The candidate has two parts.

**The NanoYalla export stays as it is** (`--format rocq`): classical
proofs against the unmodified kernel that a Click & coLLecT user has
installed, with today's trusted base and today's three refusals. It is
the compatibility target and is not grown. The one thing worth adding
there, if wanted, is an unfinished proof with its open goals as
hypotheses of the lemma, which is how Click & coLLecT's own exporter
writes one (`Goal H1 -> H2 -> conclusion`).

**A Rocq library of linlog's own**, written from nothing, in which every
mode has a statement and a certificate:
- formulas in negation normal form over a type of atoms with decidable
  equality, and the calculi as plain inductives that a reader checks
  against a textbook: one-sided LL with Mix and general weakening as
  parameters, and two-sided ILL with its affine variant over the
  connectives of the reading (the two-sided statement then carries
  step 8's `⊤`/`0` ambiguity, see the intuitionistic follow-ups below);
- the certificate as data rather than a tactic script (the planning
  session's proposal for what "idiomatic" should mean for proofs a
  program emits, which the author accepted on 2026-10-02): linlog's proof
  term (D6) as a Rocq datatype, a checker written as a function, and one
  theorem that a term the checker accepts yields a derivation. A
  certificate is then the sequent, the term and `check … = true` by
  computation: no exchange bookkeeping, no dependence on how tactics
  unify lists across Rocq versions, size and checking time linear in the
  proof, and the algorithm of the Rust checker verified once. The
  plainer alternative keeps tactic scripts with positional lemmas, as
  the NanoYalla export has, against the new inductives; it is less work
  and scales worse;
- bridge modules, optional and not needed to check a certificate, that
  import the pinned NanoYalla and Yalla's standalone `nanoill.v` and
  prove that our classical calculus and theirs derive the same sequents,
  and likewise for ILL, so that the new kernel is ours and provably the
  standard one; for Mix the theorem that a derivation of `Γ` with Mix
  gives one of `?(⊥⊗⊥), Γ` without (sketched on paper without cut: `⊥`
  on both premises, `⊗`, dereliction, two contractions; not
  machine-checked); for affine no such reduction is known, and the
  weakening rule is the definition;
- the standard library only, so that nixpkgs' Rocq builds it, and the
  conventions of the day, which the step reads from the Rocq reference
  manual and packaging documentation rather than from memory (`From
  Stdlib`, a `_RocqProject` or a dune theory, explicit locality
  attributes, `rocqdoc` comments, an opam file);
- in the exporter a second kernel behind `rocq::Options` (D15), chosen
  by the mode where the user did not choose, and in the flake a check
  that builds the library and compiles the certificates of both kernels;
- in this repository under linlog's licence, since nothing in it derives
  from LGPL text: the bridges only import the pinned kernels when they
  are checked.

What must hold: NanoYalla certificates exactly as today; no `Admitted`
and no axiom anywhere; the lemma a user reads states the sequent over
the plain inductive, with the checker only in its proof.

Size and risk: the checker's soundness over the dyadic exponentials (the
least unrestricted zone, the absorbing `⊤`), the one-succedent condition
and Mix is real proof engineering. Three to four sessions, in stages that
each leave something usable: the definitions and the checker with its
soundness for classical LL; Mix, affine and the two-sided statement; the
bridges. Fable 5.1, `xhigh` for the checker and its proof, `high` for
the rest. With tactic scripts instead of the checker, about two.

The smaller alternative, should step 17 find this too much: a fork of
NanoYalla that adds and does not edit (upstream's `nanoll.v`,
`macroll.v` and `nanoill.v` verbatim and pinned; beside them a
positional layer for `nanoill`, a Mix and an affine kernel with one more
rule each, in a namespace of our own over upstream's `formula` and `ll`,
so that Click & coLLecT's exports compile next to it unchanged). About
two sessions; it derives from LGPL files and would live in a repository
of its own, pinned as the kernel is today.

For the author to decide: the library's name, and whether it is
published on its own (opam) or only built by the flake.

Not part of it, and why: anchoring the Mix calculus to full Yalla's Mix
fragment (it needs Yalla and OLlibs built from source at a pinned Rocq
minor version, about a session, for a link the theorem above gives more
cheaply); affine logic in Yalla proper (a new parameter of its central
inductive and the meta-theory over it, the maintainer's project); a
direct certificate for nets (a Rocq development of unit-free MLL nets
with sequentialization exists, RemiDiG/proofnet_mll, and nets are
certified through sequentialization already); a Lean 4 target once
FormalizedFormalLogic/LinearLogic has units and a release.

Small things in the NanoYalla export, unchanged by this: `ex_perm_r`
makes Rocq compute `permL_of_perm`, whose cost grows with the sequent's
width (a chain of `ex_t_r` swaps if a wide sequent turns out slow), and
identifier escaping writes non-ASCII as code points where Rocq would
accept many Unicode letters.

## Certified refutations

The author asked on 2026-10-04 whether an unprovable sequent can be
certified in Rocq other than by listing every derivation. It can, in
three standard ways, and what is possible depends on the fragment: MALL
is decidable and PSPACE-complete, affine linear logic with exponentials
is decidable (Kopylov 1995), MELL's decidability is open and full
linear logic is undecidable (Lincoln, Mitchell, Scedrov and Shankar
1992), so no method certifies every refutation once exponentials
appear. Every such statement is about a calculus as an inductive, never
a negation over `Prop`: Rocq cannot prove `~ (forall P : Prop, P \/ ~ P)`,
since excluded middle may be added to it consistently, but it can prove
that LJ does not derive it.

Step 31 takes the cheap ones (item 8 of its prompt): the invariants the
search already states (`Unbalanced`, `Equation`), the classical reading
with a falsifying assignment, and classical "not valid" of ordinary
logic over `Prop`. The escalations, each a potential goal of its own:

- **Kripke countermodels** for intuitionistic and minimal "not valid":
  a deep embedding of LJ, the soundness of Kripke semantics proved once,
  and a finite Kripke model as the certificate. linlog's route through
  ILL produces none, so it needs a countermodel search of its own, most
  naturally from a terminating calculus for intuitionistic logic
  (Dyckhoff's G4ip, formalised in Coq by Férée and van Gool, CPP 2023),
  which is also the loop check's alternative of step 25's report.
- **Finite phase models** for MALL and the affine fragments: phase
  semantics is complete for linear logic, and these fragments have the
  finite model property (Lafont 1997; Okada and Terui 1999 for the
  intuitionistic ones), so a finite model refutes every unprovable
  sequent there. Its soundness is a short proof; finding a model is a
  search of its own, and a model can be large. Research.
- **Failure certificates with a verified checker**, the pattern by
  which SAT solvers certify "unsatisfiable" (DRAT and LRAT, checked by
  verified checkers such as cake_lpr): the focused engine records its
  exhaustive failure as a DAG of its synchronous choices, the memo's
  sharing kept, so that its size is the search's work and not the
  number of derivations; a checker proved sound once accepts it. That
  proof needs the completeness of the focused calculus for the
  fragment, the largest proof in this list. MALL and affine mode first.
- **Proof by reflection**: the decision procedure written in Rocq,
  proved sound and complete, and run by the kernel. No certificate at
  all, the largest development, and the kernel runs the whole search:
  for small MALL instances, and G4ip for intuitionistic logic.
- **With exponentials** a refutation is certified only where it rests
  on a bound proved sufficient for the sequent, which is the loop check
  or the bound for dyadic sequents of step 25's report.
- **Petri-net unreachability by the state equation** (step 27's second
  session, if it takes the review's advice): a Farkas vector that
  certifies the state equation infeasible is a refutation certificate
  checked by computation, given a lemma once that a Horn program's
  proofs are firing sequences.

## MALL proof nets

Only if a use case appears: Hughes–van Glabbeek nets or conflict nets are
non-canonical or exponentially large, so they are a display feature, not a
search vehicle. Assess first.

## A batch mode for the CLI

*Done in step 24* (`plan/reports/24-batch.md`); what it left open is
listed there. One candidate from its review: the second pass inside the
command (a flag that runs what the pass across the cores left unknown
again within, as README now shows by hand). Across, the batch decided
57 fewer LLTP problems than single calls on four cores; within, all 57
were decided in 64 s. Worth building when a user asks for it in one
call; it keeps every answer a function of its input and its pass.

The CLI decides one sequent per call (the author's question,
2026-10-03): `prove` takes it as an argument, from `--file` or from
standard input, and `check`, `interact` and `seq` likewise take one. The
only thing that runs many is `linlog-bench run`, which is not published,
measures rather than answers, and pays a child process per run. Someone
with a file of sequents, a directory of problems or a program that asks
many questions writes a shell loop and pays the process start, the
thread sized from `--recursion-limit` and, with `--jobs` above one, a
pool per sequent, which on the small sequents that are the common case
is most of the time.

What is wanted is `prove` over many sequents in one call, in the form
practical use takes:

- **Input.** A file or standard input with one sequent per line (blank
  lines and comments skipped, an optional name per line), several
  `--file` arguments, a directory; and the formats that exist already:
  the harness's problem files (`name; mode; expected; copies; sequent`)
  and LLTP problems, which `lltp::read` reads in the core crate but the
  CLI cannot take today. Whether a line may carry its own mode and copy
  bound, as the problem files do, or the flags hold for the whole batch.
- **Output.** One result per sequent, in input order, as it is decided:
  a line of text (name, verdict, reason, time) or a JSON Lines record
  with what `--format json` and `--stats` carry now, the proof included
  on request; for the drawing formats a directory with one file per
  sequent. A malformed line is that line's error, not the batch's end.
  The exit status needs a rule for many verdicts (the worst, by the
  order error, unknown, unprovable, proved, is the obvious one). By D15
  the options are one value that the web front end and other wrappers
  use too, so the batch is a library notion (an iterator of problems to
  an iterator of results) with the CLI as its first caller.
- **Limits.** `--timeout` per sequent, and one for the whole batch.
- **Cores.** With many sequents the cores belong across them (one
  sequent per worker on the sequential engines, deterministic and
  without a pool's set-up), and within one only when the batch is short
  or a sequent is hard; which of the two a default takes is the same
  question as the CLI's default `--jobs` ("Follow-ups: parallel search")
  and should be decided with it.
- **Isolation.** In one process a sequent that exhausts memory or the
  stack takes the batch with it, which is why the harness starts a child
  per run. The search thread per sequent stays; whether a memory bound
  per sequent is needed (the memo and the arena are the growing parts,
  and the additive memo has a cap already) or `--isolate` falls back to
  children is the design's to say.
- **A stream.** Reading standard input line by line and flushing each
  answer makes the same command a server for an editor plug-in or a
  script that asks, waits and asks again, without a start per question;
  it costs nothing if the batch is built as a stream from the start.

One session. It touches `cli/` and a small entry point in the core
crate, no engine. It is checked by the batch's results being those of
the single calls on the problem file and on a sample of the LLTP
library, in both orders of cores, and by a timing that shows what the
loop paid.

## Ordinary logic through its embeddings: classical, intuitionistic and minimal sequents

Deciding sequents of ordinary propositional logic by translating them
into linear logic and running the engines that exist (the author's
question, 2026-10-03, "if something like that is even possible"). It is
possible, for all three, and part of it is measured already. Only
propositional logic: linlog has no quantifiers.

- **Classical logic needs no exponentials.** The one-sided calculus of
  classical propositional logic with invertible rules (an axiom with a
  context, `∨` keeping both disjuncts in one premise, `∧` copying the
  context into two) is complete without contraction, and it is affine
  MALL read with `∨` as `⅋`, `∧` as `&`, true as `⊤` and false as `⊥`.
  So a classical sequent in negation normal form is valid exactly when
  its image is provable in affine mode, which the focused engine decides
  today with no copy bound and no choice to backtrack over (by hand:
  `--affine '|- ((a^ | b) & a^) | a'`, Peirce's law, is proved, and
  `'|- (a & b^) | (a^ & b)'` refuted). The cost is one branch per way
  through the conjunctions, as a tableau's: this is not a SAT solver and
  should not be offered as one. The translations with exponentials
  (Girard's, and LKT and LKQ of Danos, Joinet and Schellinx) are for
  showing, not for deciding.
- **Intuitionistic logic goes through `!`.** Girard's translation
  (`A → B` as `!A ⊸ B`, `∧` as `&`, `A ∨ B` as `!A ⊕ !B`, false as `0`,
  hypotheses under `!`) preserves and reflects provability into ILL,
  which is intuitionistic mode with the two-sided engine; the
  call-by-value variant and Liang and Miller's 0/1 translation do the
  same with the exponentials elsewhere, and which one is taken decides
  what search the focused engine performs, much as the atom bias does.
  The LLTP library's `ILL/ILLTP-*` and `ILL/KLE-*` collections are
  exactly these three translations applied to the ILTP library's
  propositional problems and to Kleene's theorems, and
  `bench/lltp/ILTP+KLE` has the originals, so both baselines measure
  this already.
- **Minimal logic** is the same with false translated as an atom rather
  than as `0`.

What the baselines show is the obstacle: the copy bound. In the second
baseline, at the harness's bound of three copies, `ILLTP-SYJ-cbn` has 12
problems proved and 22 refuted of about 250, and 185 answered
`copy_bound`; `KLE-cbn` has 65 of 85 theorems proved and 19 at the bound
(`bench/results/2026-10-02/lltp-intuitionistic.csv`; `lltp-copies-10.csv`
says what a larger bound buys). An unknown is no answer for a logic that
is decidable. On the image of a translation the search can be made to
terminate: every hypothesis is under `!`, so the unbounded context is a
set of subformulas of the input, the linear part is small, and a branch
that meets a sequent it has met is cut. The engine has no such check; it
deepens to `Options::copies` and answers `Reason::CopyBound`. Either a
loop check on the branch for dyadic sequents (worth having for MELL and
LL in general, where it turns some unknowns into refutations, and to be
weighed against what it costs the memo, whose entries become dependent
on the branch) or a bound computed from the input that is proved enough
for the image. This is the engine work of the candidate and its risk;
the rest is a layer on top.

The layer:

1. A type and a syntax for ordinary formulas and sequents (`->`, `/\`,
   `\/`, `~`, `<->`, true, false), apart from the linear ones so that
   nobody writes `&` and gets the wrong connective, and names that do
   not collide: `--intuitionistic` means ILL today.
2. The translations as public functions, each by its name, with the
   image printable (`seq` could show it): for teaching the embeddings
   are worth seeing by themselves.
3. Deciding by logic: classical through affine MALL, intuitionistic and
   minimal through ILL with the termination above, the translation
   chosen by what the baselines and a run of the ILTP problems show.
4. The proof read back: a derivation of the image mapped to LK or LJ
   with their rule names (the exponential rules become contraction and
   weakening or disappear), drawn by the exporters that take a
   derivation now. The linear proof is checked by the checker that
   exists; the read-back needs a check of its own or is trusted, which
   the design says. A Rocq certificate is cheap here and needs no
   library: the proposition as a lemma over `Prop` with its proof (the
   classical ones on the standard library's excluded middle).
5. The ILTP propositional problems with their statuses as the benchmark,
   read in their own syntax and translated here. LLTP's translated files
   are not a substitute: some lost a negation on the way (the original
   of `KLE078+1` is `(a => b) => ~~(~a | b)` and its `KLE-cbn` file has
   `!a + !b` in the disjunction; likewise `KLE086+1` there and
   `KLE069+1` in `KLE-01`), so that their headers say "Theorem" of
   sequents that linlog refutes, rightly, once the copy bound lets it.

Not competitive with the provers made for these logics (SAT solvers,
the intuitionistic provers ILTP records) and not meant to be: it is for
someone who has linlog open and a formula of ordinary logic in hand, and
for showing how the logics sit inside linear logic. Two sessions, the
termination on dyadic sequents first, since it stands without the rest
and the layer is of little use without it; it depends on what step 17
decides about the default copy bound.

*Done as step 25 (2026-10-04), the layer alone.* The default deepening
(step 21) made the layer useful without the termination: at 2 s, cbn
decides 96 of the 274 ILTP problems, cbv 88, 01 68, 108 together, and
classical logic 156. What termination would add is measured in step
25's report: of the 166 problems no intuitionistic translation decides,
35 deepen past a copy bound of 30 with every level cheap and none
proved, all Non-Theorems, which a loop check on the branch would refute
(SYJ207 to SYJ211, SYN392, SYN393); the other 131, the 48 undecided
Theorems among them, stay below 30 because each level is too expensive,
which no loop check changes. So the engine work is a step of its own
if refuting intuitionistic non-theorems matters for teaching, and not
worth one for the theorems.

## The web front end

`linlog-web`: the `core` crate compiled to wasm without the `parallel`
feature (and without whichever optional features of D14 the client does not
ship), the interactive state of step 9 as the client's state with its JSON
as the wire form, the SVG of partial derivations from step 11 as the
picture (Euler Math served through `@font-face`, the same advance table
as the SVG), and the JSON formats for import and export. Its own plan, which
starts from `plan/reports/09-interactive.md`, "For steps 10 to 12 and the
web front end": the calls the client makes (`Interactive::new` or the JSON,
`goals`/`goal`/`reading` to draw the goals with positions as click
targets, `rules` for the menu, `split_passes` to grey out a split,
`apply`, `undo`, `close` with a node-counting stop closure since wasm has
no clock in core, `derivation` for the picture, `proof` at the end), and
the `linlog interact` command of step 9 as the reference behaviour; and
`plan/reports/11-svg.md`, "What the web front end will call": pure,
clock-free `svg::derivation`, `svg::net`, `svg::sequent` and
`svg::two_sided` with a `Style` (a dark theme sets its colours), the ids
`i<n>` (conclusion of inference `n`), `o<n>` (literal or node of
occurrence `n`) and `l<m>-<n>` (a link) as click targets, and Euler Math
(OFL) served as a web font under the family name `Euler Math`, the layout
holding without it through `textLength`.

## Follow-ups: the web bindings

From step 32a (2026-10-19), `plan/reports/34-web.md`, "Left open":

- **A turn's set-up kept across restarts** (unassigned): under wasm the
  default bias's two searches take turns, and each turn restarts its
  search; on `PermAdmissibility_unf-8x8-4stageSEN-50_10_1` that costs
  19.4 times the better search's time for 2.95 times its work. A turn
  could keep the classes and counts its search set up. It matters only
  without threads.
- **Clippy for wasm32** (unassigned): four sites of the core crate that
  are right on both widths fire there (`absurd_extreme_comparisons` at
  `core/src/ordinary/mod.rs:452`, three `unnecessary_box_returns` in
  `core/src/proofs/check/pass.rs`); no check runs clippy for that
  target.
- **The library's whole unit suite on 32 bits** (unassigned):
  `web-core-wasm32` runs the checker's, the size estimate's, the memo's
  and the memory account's tests on `wasm32-wasip1`; the rest (many use
  threads, which WASI lacks) would show what else assumes 64 bits.
- **The first steps of a goal's proof as a library call, and `Step` and
  `Applicable` as wire forms** (step 34's later sessions, if the page
  wants them in the library; additive at wire level 1): the bindings
  define their own JSON for both and read the hint off the goal's proof.
- **Ordinary logic's JSON forms** (`ordinary::Outcome`, the ordinary
  sequent and its derivation): with ordinary logic on the page, after
  version 1 (unassigned).

## Follow-ups: the net engine after the cyclic mode

From step 36a (2026-10-19), `plan/reports/35-lambek.md`, "What is left
open":

- **A panic past the structure's vertex bound** (unassigned; older than
  step 35): `Switching::new` expects `ProofStructure::new` to succeed,
  but nothing between the front door and it checks `nets::MOST`
  (1 431 655 763 vertices), so with `Limits::occurrences` lifted a
  sequent of shared subterms that unfolds past it would panic instead
  of being refused, in every mode. The default limit shields every call
  that keeps it; a check of the forest's length in `Nets::admits`
  refuses it.
- **A conditional symmetry break for the planar search** (unassigned):
  35a's three breaks are off in the cyclic mode, since exchanging equal
  literals can break planarity; a break that keeps the cyclic order
  waits for a row that needs it.
- **The inherited quadratic `choose`** (35b's open item) holds for the
  planar search too; it never dominated a measured row.

## The web client's polish: the next step after the break

The author, 2026-10-19, after step 34 went live: one session in the
client's repository, with both plugins vetted for the design session
(`frontend-design` and `education-agent-skills`), run as the next step
when work resumes; its prompt is written with the renumbering of the
steps (it becomes step 38). What it fixes:

- **Touch targets**: some buttons are too small or hard to press,
  especially on a phone; every control measured at phone and desktop
  widths, 44 by 44 pixels on a phone, never below WCAG 2.2's 24, with a
  test that fails below the floor.
- **Spacing**: wrong or ugly in places; one spacing scale in the tokens,
  used everywhere.
- **Practice**: many more examples, along the learning progression
  (D60), graded by difficulty, provable and unprovable mixed, every one
  decided by the command first.
- **Visual polish**: a more modern look and feel, within the chosen
  "Instrument" direction and the design's decisions.
- **Visual consistency**: one visual grammar (D48): the same component
  looks and behaves alike everywhere (sizes, states, radii, type,
  spacing), checked against one-off values.
- **Ease of use**: the common tasks in as few steps as possible, with
  clear affordances, walked through at both widths.
- **The two-sided toggle only in classical mode** (the author,
  2026-10-19): intuitionistic derivations are two-sided already, so the
  toggle (D58) is shown for classical logic alone and hidden, not
  greyed, in intuitionistic mode.
- **Which engine ran, and its reference** (the author, 2026-10-19): the
  page shows the engine of every search it ran (a goal's close, all
  goals', a hint, the verdict), beside the work it did, and each engine
  names its source in the literature, one gesture away (a hover or
  Learn's reference): the focused search (Andreoli's focusing), proof-
  net search (Girard; Danos and Regnier's criterion), essential nets
  (Lamarche; Murawski and Ong), the planar nets of the ordered modes
  (Lambek; Retoré), the Horn engine (Petri-net reachability and
  coverability), the inverse method (Chaudhuri and Pfenning's focused
  inverse method), the additive path, and the classical refuter; every
  reference checked against its source, not written from memory, with
  a link where one is stable.
- **Choosing the engine by hand, without crowding the page** (the
  author, 2026-10-19, unsure it is worth the clutter; the planning
  session's proposal): one choice in Settings, "Engine", default
  "Automatic", listing only the engines that take the current proof's
  sequent and the others greyed with their one-line reason (as the net
  tab is, D4); a forced engine's refusal said at the goal; nothing on
  the stage or in the bands. If it still adds clutter, the session says
  so and leaves it out, for the author.
- **Both plugins throughout** (the author, 2026-10-19):
  `frontend-design` for the look, the consistency and the controls, and
  `education-agent-skills` for the practice set, the progression, the
  engine references' place in Learn, and ease of use for a student.
- **Other nitpicks of the same kind** (the author, 2026-10-19: look
  for them): a sweep of every control, label and text for what does not
  apply where it is shown, says the same thing two ways, or surprises.
  The session fixes what is clearly wrong and collects the judgement
  calls for the author. Known already: the practice cards write
  negation as `~A` while the drawings write `A⊥`; the essential net's
  green input side beside D26's "green only for a checked proof" (step
  40b's open point); Enter in the sequent field dropped silently while
  another action runs (step 32c's follow-up).

*Done in step 38 (2026-10-22, `plan/reports/38-web-polish.md`)*, with
the engine choice in Settings built; the essential net's green and the
other judgement calls of the sweep are in the report, for the author.

## Kept as built: the web client's choices, for the author to revisit

*Recorded 2026-10-23 at the author's word ("persist those points, so that
I can revisit them"), after the author accepted them as built; each
checked that day against the client at `main` 0397f950 (`lwrmzpqv`).
Each says what the page does, where, why it was kept, and which session
chose it (its report has the reasoning). To change one, name it to the
planning session; a change is a session of the client.*

- **Practice hides its answer.** A card's verdict, explanation and Watch
  open only from "Answer" (`src/islands/Learn.tsx`, the `<summary>`
  "Answer"): with provable and unprovable problems mixed, showing them
  would give the answer away. Session 38.
- **Practice mixes near misses.** Each harder provable problem has an
  unprovable near miss beside it, so a lesson stays mixed. Session 38b.
- **A proof opened from a link draws its grafts without the engine's
  label**: links carry the session, not the engines (`src/lib/link.ts`;
  the engines are kept per tab, `Proof.engines`, `src/islands/state.tsx`).
  Session 38.
- **A tab whose sequent did not parse stays open** beside the next
  proof, so the typo can be mended. Session 38.
- **A goal wider than a phone is panned, not shrunk**, at the size that
  keeps its formulas 44-pixel targets; the goal navigator brings it into
  view. Session 38 (finding 4) and 38b.
- **After a step the stage zooms back up to that size, unless the
  visitor zoomed by hand** (`src/islands/Stage.tsx`, `byHand` and
  `floor()`). Session 38b.
- **Light dismiss** (`src/islands/App.tsx`, the press handler): a press
  outside a menu, a pane, More's sheet, the bands' popovers, the rule
  popover, the split chooser or the mode warning closes it; Learn and
  the tour are not closed by it (each is where the visitor works); a
  press on the stage that closes something does nothing else, while a
  press on a band's button closes and acts; on a phone, a press on the
  bands closes a sheet that covers the stage. Session 38b.
- **The fonts**: the export's font is chosen in the export menu, the
  stage's in Settings ("Font of the formulas"), and the export follows
  the stage's font until the export menu picks another, which is then
  kept (`exportFont`, `src/islands/state.tsx`); the four optional fonts
  are fetched only when chosen, and the service worker caches each on
  its first use (`src/offline/sw.js`, `LATER`); the tab's sequent
  follows the stage's font (`.m` reads `--font-math`); every font is laid
  out with Euler Math's vertical metrics, net edges starting at each
  font's own depth (linlog's `core/src/export/svg/font.rs`). Session
  38b; the fonts other than Euler Math depart from D25, D16 and D53 at
  the author's request.
- **With Learn open, the bottom band packs tighter**, keeping one row at
  1 280 pixels; a narrower desktop window still wraps it
  (`src/styles/app.css`, `.app[data-learn]`). Session 38.
- **Engine names** are lower case inside a sentence ("closed by the
  focused search"), in sentence case standing alone ("Focused search").
  Session 38.
- **The tab's chip**: the calculus mark stands 12 pixels of padding from
  the sequent (with the sequent's own spacing, at least 16 pixels on the
  screen, `tests/chip.spec.ts`), its calculus and fragment 8 apart
  (`.pt-mark`). Session 38b. Since 38c the chip opens the tab's mode
  settings (D58, at the author's request); session 38e makes it the one
  place for them.
- **A new empty tab starts in the calculus last chosen in the visit**
  (classical at a visit's start, `lastMode`, `src/islands/state.tsx`).
  Session 38c.
- **Only a key moves the focus into the chip's popover**; a press leaves
  it where it was, so no tip appears, and Escape gives it back to the
  chip (`src/islands/TopBand.tsx`). Session 38c.
- **The phone's bottom band** shows seven icons (undo, redo, close all,
  hint, export, Learn, More), More's sheet holding steps, import,
  settings, the tour, keys and About; the mockup had a collapsed sheet
  of three actions. Session 38, departing from the design.
- **Safari's trackpad pinch** (its gesture events) is handled and covered
  by no test (`tests/zoom.spec.ts` runs Chromium); one check by hand on a
  Mac or an iPhone would settle it. Session 38b.

## Follow-ups: the web client

*Assigned to session 38e (the author decided on 2026-10-23: Fabian Lukas Grubmüller, privacy@grubmueller.dev, no Impressum).*
The web client's privacy notice under the Swiss revFADP (and the GDPR,
should the site count as offering a service in the EU): the name and
e-mail address to publish as the person responsible (an alias such as
`privacy@` on the author's domain), and whether to add a postal
address or an Impressum (Swiss UWG Art. 3(1)(s) likely does not reach a
free non-commercial site). The page would say: no cookies, no tracking,
nothing loaded from third parties (the policy's `default-src 'none'`,
checked on the live site); GitHub logs visitors' IP addresses for
hosting and security, in the US, under the Swiss-U.S. Data Privacy
Framework (GitHub's own entry on its list still to check); what the
browser keeps (`linlog-settings`, the theme, autosaved tabs, the
service worker's cache, the optional fonts) and how to clear it; the
visitors' rights. About has one line on GitHub's logs today.

*The math text's weight (done 2026-10-23, client `lwrmzpqv`).* The active tab's
sequent is shown in a synthesised bold: `.tab[aria-selected="true"]` sets
weight 600, and the math fonts have one weight each, so the browser
fakes one (likewise wherever math sits in a semibold label, as the
practice cards' headings may); `font-synthesis: none` and the normal
weight on math text fix it. In dark mode `.drawing svg` draws a glow
(`drop-shadow(0 0 6px)`, light blue at 35 %) around every glyph, which
reads as extra weight; keeping it off the formulas, or fainter, keeps
them crisp. The stage itself is Euler Math's own weight, Medium, drawn
large (checked 2026-10-23: no inherited weight, no fallback font).

*Left by step 38b (the compact edges assigned to step 39's second session; Safari's pinch for the author to check by hand).* Safari's trackpad pinch (gesture
events) is handled and tested by no test; a compact net's edges begin
a tenth of an em below each literal, inside every font's descenders,
Euler Math's included (older than step 38b).

From the review of step 32b (2026-10-19):

- **A link whose sequent does not parse opens an empty stage** (step
  32c): no message, where D15 asks for the parse error under the field
  with its place, as when the sequent is typed.
- **The parser reads `a⊥`** (unassigned, the library): the drawings and
  the derivations print a negated atom as `a⊥`, the parser refuses it
  (`unexpected "⊥"`), so a student who types what the page shows gets
  an error.
- **A link longer than the page writes** (step 32c, or unassigned): the
  page unpacks whatever `d` holds; a crafted link can expand to a large
  document in the visitor's own tab before the library's limits read
  it. A cap on the unpacked bytes refuses it.
- **The tab shows the sequent as typed** (`a^`, `par`), where D53 asks
  for it in Euler Math (step 32c).

*Done in session 32c (2026-10-19, `plan/reports/34-web.md`):* a link's
parse error is shown under the field, a link's session unpacks to at most
512 KiB, and the tab shows the sequent in signs and Euler Math. The
parser's `a⊥` stays unassigned.

Left by session 32c (2026-10-19):

- **Learn's content** (step 43, assigned 2026-10-23): the lessons of D60 in the
  order of D56, the rules' schemas in the reference, and the hover
  explanations inside the frame (D47); every sequent proved or refuted
  by the command first, as the examples' were.
- **Enter in the sequent field while another action runs does nothing**
  (the client, unassigned): one action that changes a proof runs at a
  time (32b's answer to its review), and a sequent submitted meanwhile,
  while the module still loads on a slow machine for one, is dropped
  without a word; it could wait for the running action instead. *Done in
  step 38: it waits.*
- **`linlog interact` prints a stray comma** (the command, unassigned):
  a two-sided goal whose first hypothesis is not at position 0 prints as
  `goal 2:, 1: A ⊢ 0: A` (`A -o B, B -o C |- A -o C` in `-i` after
  `apply 0 2 ⊸R`, `apply 1 0 ⊸L 2`).

Left by step 38 (2026-10-22, `plan/reports/38-web-polish.md`):

- **The inverse method's turns rarely come in a tab** (the bindings or
  the dispatch, unassigned): the tab's recursion limit (384) ends the
  two-sided engine on an intuitionistic goal with `!` long before the
  head start of 2²⁴ units (`!(a -o a * a * a), a |- a * (a + 0)`:
  unknown at the recursion limit after 404 262 units, where the command
  refutes it by saturation at 16.8 million; of the ten goals with a
  copying clause that the session tried at the tab's settings, six ended
  so and four were refuted by a turn), and on the page those stay unknown. A turn
  before a recursion limit stops the engine, or a head start measured
  against the tab's limits, would bring them to the page.
- **A bindings call for the engines that take a sequent** (the
  bindings, unassigned): the page asks `prove` forced to each engine
  with a budget of one unit, which refuses before searching where the
  engine does not take the sequent; a call over `search::engine_for`
  would ask once per engine without starting a search. Written and
  tested in step 38 and set aside, since the client's lock names only
  linlog's `main`.
- **The parser reads no `a⊥`** stays (above).
- **The judgement calls of step 38's sweep** (the author): in the
  report, "Judgement calls".

## Follow-ups: the focused engine

*Assigned (2026-10-03).* A reference prover kept in the repository,
test-only, for the differential runs that reviewers so far wrote and
threw away (the unfocused two-sided prover of step 8, for one): step 26, before
it changes the engine; the engines of steps 27, 32 and 37 join its test.
The checker has its own since step 18 (`proofs/oracle.rs`). The check and the derivation of a large net:
step 18. The forward search's missed stop and the portfolio's removal:
19. The search's own memory: 20. The copy bound, the five sampled
problems and the default's contract at the limit: 21. The Horn test on
the goal, Mix's `3^n`, a level of recursion per link of a free chain and
the first three constant factors: 26. The unit of work, the forward
bound on Horn programs only, the restart from the frontier and the free
splits without rows: superseded by step 27 if its engine takes the nets,
else 26. A search that can be suspended: by step 34's measurement. The
pool's split of threads between the two searches: measured in 30. A
sound affine prune: step 27 answers it for Horn programs. The rest stays
as written.

*Found by the comparison's gap study (2026-10-17,
`plan/notes/research/comparison-gaps.md`, unassigned).* Of the 122
intuitionistic problems Sympli decides and linlog does not, 94 are
refutations whose copy levels never close on spaces of about fifteen
stable sequents, and at least four of them were refuted by the engine
of 2026-10-04: a likely regression, suspected in the memo's cut entries
carried from level to level. The note ranks four fixes (R1 the cut
entries, R2 a classical countermodel, R3 a cap on members only `⊤`
takes, R4 the inverse method), each with a held-out measurement, and
lists the probes owed (A1 to A5). The author decides where they go.
*R1 and A5 done in session 30b (2026-10-17,
`plan/reports/30-baseline-release.md`):* the cut entries are read only
within their generation, A1's four rows are refuted at levels 5 to 12,
and five rows without a copy bound are in the target set. R2, R3 and R4
were adopted by the author on 2026-10-18: R2 in session 31d (on by
default), R3 as step 33, R4 in step 37; KLE075+1, which Sympli refutes against its header and which
has no classical countermodel, is now refuted by linlog in all three
translations too.

*Left by step 30's baseline (2026-10-17, `plan/reports/30-baseline-release.md`,
session 30c; unassigned, for the next performance work).* A fixed cost
per call: on the problems that take microseconds the median time is
1.5 to 2.1 times the second baseline's with the same counters
(KLE-IMP-CONJ 28 → 48 µs, SYJ-cbn 67 → 146 µs), 20 to 80 µs of set-up
more per call, not traced; the memory account that now charges the
input and the race's set-up are where to look first. And 21 QBF runs
that the second baseline refuted end at the default memory bound of
1 GiB, which the second baseline did not have; above it they are
decided as before.

*Left by step 26 (2026-10-04, `plan/reports/26-focused-engine.md`).*
The reference prover, the Horn test on the goal, the Mix prune, the
chain loop and the profile's hot spots are done. Left, each measured
there:

- **A sparse memo key**: both zones are still hashed, compared and
  stored at the forest's width per stable sequent, now the largest cost
  on the widest nets (hashing and comparison 31 % and 27 % of
  `CloudDeployment_deploy_5_b_100_1`); the zone's range made the rest
  proportional to the members. *Step 28's area 3.3 (F148, 2026-10-13)
  left it:* those nets are now proved by the Horn engine in 4 to 8 ms;
  on the widest rows of the target set that the focused engines still
  search the key is 14.5 % (`SYJ207+1.018`, 5 770 occurrences, at its
  copy bound in 108 ms) and 29 % (`SYJ212+1.010`, 27 105 occurrences,
  30 ms) of the instructions, and no focused row ends at the memory
  bound. Without a copy bound, as the command searches by default, both
  run into the time limit and the key is 18 % (`SYJ207+1.018`) and 43 %
  (`SYJ212+1.010`): up to about 1.7 times the stable sequents per
  second. Γ by its members would halve that; `Θ` spans the forest on a
  net of transitions, so the rest needs `Θ` interned (an id carried
  with it, shared by a pool's workers), a rewrite of `memo.rs`, the
  shared memo and the signatures that pass `Θ`. *Done in step 28's
  follow-up `step-28j` (2026-10-14):* `Θ` interned, `Γ` by its members,
  records of varying length; the target set's counters equal, both rows
  1.45 and 2.19 times the stable sequents in 10 s on one core, and
  18 % and 36 % fewer instructions per stable sequent over the same work
  (step 28's report has the table). What the key left: on a forest of a
  word or two a member list is larger than the bitsets were (the memo's
  entries larger, `mix/10` none the slower), and the unit of work still
  counts the forest's width per stable sequent, which moves the default
  bias's slices if it goes.
- **`Θ`'s members by class** (`one_of_each_copy`) are kept for one `Θ`
  only and sorted again when the next stable sequent's differs: 12.9 %
  of `SYJ207+1.018` (2026-10-13), whose branches alternate between
  `Θ`s. A small cache of several, or the sort kept with each `Θ` where
  it grows. *Since the sparse key (2026-10-14)* the cache is keyed by
  `Θ`'s id, so a cache of several costs a compare of a word each, and
  the sort is the largest cost left on that row: `one_of_each_copy` and
  its sort 34 % of the instructions (callgrind, 100 million units of
  work without a copy bound), beside the copies' `meets` walk.
- **`a_raised_limit_holds_on_the_pool` overflows the test thread's
  stack in a debug build run with one test thread**
  (`RUST_TEST_THREADS=1`, or `cargo test -p linlog --features parallel`
  alone), at the head before step 28's sparse key as after it, and
  passes with `RUST_MIN_STACK=67108864`; the search runs on the pool's
  threads, so the depth is in the set-up on the calling thread. Found
  by `step-28j`'s panel; why the gate's `cargo test --workspace` passes
  it was not looked into.
- **The canonical key** is built for every stable sequent of a forest
  with two equal formulas (38 % of `HypercubeGrid_hc3k4p4b12_5_1`).
  *Done in step 28's area 3.3 (F150):* only where a member of the zone
  is renamed.
- **A focused search that hands over to the net engine at the recursion
  limit**, the feature the dispatch's net rows stand in for (the net
  engine decides chains of thousands of links that the focused engines
  recurse through).
- **A choice's error still cancels its siblings on a pool**, the mirror
  of the `&` case step 26 fixed: a pool may answer the recursion limit
  where one thread refutes (`|- !?~c, (?(a * c) par ?!~c)` with Mix at a
  limit of 9 on four threads). The late cancellation at a `&` stays
  unreproduced.
- **The Mix parts' cost** (3 to 10 % more stable sequents on generated
  Mix sequents), and `mix(11)`'s splits, still `3ⁿ` under `⊕ 0`, which no
  count cuts.
- **Mix in affine mode** is left out of the search (admissible with
  weakening); a panel measured 3 decisions lost at a copy bound against
  1 638 gained, kept on the author's decision.


Left open by step 15 (`plan/reports/15-performance.md`, which has the
numbers behind each). Step 15 took the canonical choice among identical
members, the forced rule for a tensor of positive literals and the hash
per branch-stack entry, and measured and dropped the restart of a level
from the frontier.

- **The default bias with exponentials**, built by the second session of
  step 15 (`Bias::Auto` runs the backward and the forward search and
  answers with the first that decides; `.claude/rules/core-focus.md` has the
  scheme). What it leaves open:
  - *The unit of work is rough.* The two searches share one core by
    work the engine counts (a split step, a stable sequent weighted by
    its size), and the time a unit takes varies by two orders of
    magnitude between problems, so one search can get several times the
    other's time. The costs not counted are known (the lookups of duals
    in a forced chain, the sort of the copies); a unit calibrated on
    more nets, or the removal of those costs (`meets` compares every
    literal of every copy with every member of `Γ`, which is most of a
    stable sequent's cost on a net of thousands of transitions), would
    bring the measured cost nearer the scheme's 1.5 and 3 times.
  - *The forward search's own bound applies only to a Horn program*
    (clauses, a marking, a goal). A sequent with one formula of another
    shape gets the forward search within `--copies` only, and the
    price of the bound where it applies is a slower "unknown" on
    programs whose markings grow (45 of a review's 2 000 random ones
    over a second). A
    count per copy (a clause's copy as a step, any other as a copy)
    would lift that, at the price of a budget with two parts in the
    memo's `Exhausted` entries.
  - *The deepening restarts every level from the root*, which on a
    forward chain of `n` steps costs `n²/2`; with a bound of 30 that
    is nothing, with nets that need 100 steps it is what the restart
    from the frontier (below) would save.
  - *Five sampled LLTP problems that `--copies 10` decides under either
    bias stay at the default bound of 3* (translations of intuitionistic
    problems, not Horn): whether the default of `--copies` should rise
    is a question for the second baseline's `lltp-copies-10` pass.
  - *`Options::portfolio` has no use left that a measurement supports*:
    the two searches side by side are the portfolio that pays. Remove
    it if the second baseline shows no gain once more.
  - *On a pool the threads are split evenly* between the two searches;
    not measured (it needs the machine), and the split is a candidate
    for a measurement in the second baseline's all-core stage.
  - *The two searches alternate on two threads, and only where threads
    exist.* The engine recurses on its thread's stack, so a search
    cannot be suspended and resumed in place; to alternate without
    starting again, the backward search gets a thread of its own and a
    baton lets one of the two run at a time. What that costs and where
    it leaks (the author asked, 2026-10-02): a thread and its stack per
    call even under `--jobs 1` and `--deterministic` (37 µs against
    7 µs on a small sequent, which a caller that closes many small
    goals pays every time); a library user who builds with the
    `parallel` feature gets a thread started inside `prove` without
    asking for one; and a build without the feature, wasm above all,
    runs the other scheme, turns that start again on growing budgets,
    which costs up to five times the better search where the threaded
    one costs one and a half to three, and gives other counters for
    the same input, so a stop that counts work (the web front end's)
    behaves differently from the command. The fix is a search that can
    be suspended: the engine's recursion turned into an explicit stack
    (the net engine has one; for the focused engine it is a rewrite of
    its control flow, with the counters as the oracle), after which the
    two alternate on one thread everywhere, the baton and the restarts
    go, and the same input gives the same counters in every build. It
    belongs with the code audit's refactoring or with the web front
    end, whichever comes first. Short of that: an option to choose the
    scheme, so that a caller can refuse the thread.
  - *The Horn test reads the forest's roots, not the goal* (`chains`,
    found by the planning session's review): for a goal off the roots,
    as `prove_goal` and the interactive `close` hand one over, the
    forward bound follows the shape of the whole sequent. It decides a
    bound and no verdict, and the closes tried by hand behaved; the
    test belongs on the goal's members.
  - *The rows the backward search decides slowly pay most*: the
    `NeighborGrid_z_2d_3n_1m_t_1_2_*` nets take 2.2 s under `--bias
    rarer` and 3.9 s under the default (1.4 s in the first baseline),
    1.8 times, where the scheme's own figure is 1.5; under a 5 s limit
    on a loaded machine one of them was lost. They are what a better
    unit of work is measured on.
- **Mix costs `3^n` memo lookups** for `n` members that no prune
  separates (the `mix` family: 14.3 million stable sequents at eight
  pairs, eleven pairs not within 300 s), since every part enumerates its
  own partitions. A fact "no subset of this part is provable", which
  holds for a part when it fails without Mix and holds for each of its
  subsets with one member less, would make that `n·2^n`.
- **Free splits where the counts have no rows**: every atom under a `!`
  or `?` has no row, so on a Petri net with a clause body that is not a
  tensor of positive literals (the rarer-literal bias makes some body
  atoms negative) the split search cuts nothing, and such nets still
  spend their time limit at one stable sequent (69 of the 113 sampled
  LLTP problems time out). The forward bias removes those splits; a
  count for exponential atoms that is sound under copies does not exist.
- **Constant factors the profile showed and step 15 did not take**, in
  the order of their share: hashing and comparing memo keys (29 % of the
  samples on a Petri net with 12 926 occurrences, where both zones are
  hashed in full for every stable sequent though `Θ` rarely changes;
  25 % on Mix), the allocations of a memo insert (23 % on that net: two
  boxes and a vector per key; keys in an arena), the canonical key built
  for every stable sequent (6 to 11 %; a bitset of the occurrences that
  have an earlier equal would skip it where no member is renamed), the
  member list and tally built per stable sequent (35 % on `growing`),
  `OccSet` as a `Box<[u64]>` at every size (no target over a second has
  a forest of at most 64 occurrences, so the profile does not point at
  it), link-time optimisation (1 to 6 % for twice the build time).
- **Sharing more among interchangeable sequents.** Only complete
  failures are shared. Failures cut by the copy budget, shared the same
  way, halved the stable sequents of `chain` and saved a fifth on the
  counter, and kept sequents that the search used to refute at the copy
  bound for good (a relative answered by the sequent's own entry of the
  level before); a deepening that can tell "cut" from "cut because a
  relative was cut" would get the saving back. Keying `Θ` up to
  interchangeable members would merge more as well; it is sound by the
  lemma in `.claude/rules/core-focus.md` and costs a pass over `Θ`.
- **A free split still costs a level of recursion per link** of a chain
  of `⊗`; only forced chains and `?` rules run in a loop. Two sampled
  ILLTP-SYJ problems still end at the limit of 2 048.
- **The order in which a split search tries the members** changes which
  proof is found first, and on some generated sequents with many `⊤` the
  new order visits more stable sequents than the enumeration it
  replaced did (the report has the cases); an order informed by which
  side needs a member is untried.
- Whether a sound and useful affine prune exists (Kopylov's decidability
  argument does not give one directly; the spec's was unsound) is a
  research question to keep open; until then affine mode stays bounded.
- The interval of `&` could be the intersection instead of the hull.
- *The Horn forward bound under affine with Mix* (from step 26): the
  forward search's own copy bound stays off whenever `mode.mix` is set
  (`schedule::plan`), for a reason (a chain's stable sequents tried in
  every partition) that no longer holds where affine mode leaves Mix
  out of the search. Following it would move answers between decided
  and unknown, so it needs its own measurement.
- *A free split under weakening is monotone* (from step 26): in affine
  mode a split `(L, R)` of `Γ` for `A ⊗ B` works only if `⊢ Θ ; Γ, A`
  and `⊢ Θ ; Γ, B` are provable (weaken the rest in), so a complete
  failure of either refutes every split at once, where the search now
  enumerates all `2ⁿ` (no count prunes under weakening; the test sequent
  of `stops_inside_a_split_search` has `2⁴²`). Cuts and the order of the
  search need the same care as the Mix prune's.

From the second baseline (`plan/reports/16-baseline.md`, as its review
on 2026-10-03 corrected it: the Status log of `plan/README.md` has what
was run), none of them a wrong verdict. The first two are what a user
meets first on a large problem, and come before any new engine:

- **The proof check and the derivation of a large net run out of
  memory, not the search.** Of the 76 runs that ran out of their 12 GiB,
  58 are 14 Petri nets of tens of thousands of transitions with a
  one-step firing sequence (BART-040 to -060, TokenRing-40 and -50,
  Philosophers-10000_1_1, AirplaneLD-pt-4000, GPPP-1000-1000_1_1 and
  others; 14 per default pass and in the forward pass, 3 under `--bias
  rarer`, 13 in `lltp-recursion`). The search proves every one of them
  in 49 ms to 2.0 s within 232 MB (`linlog prove -i --jobs 1 --quiet
  --stats` on the sequent, each of the 14 tried; four of them also in
  classical mode and under the forward search alone). What fills the
  memory, at a gigabyte or more per second, is `proofs::check::derive`,
  which keeps what every node derives (`Derived`: a set of the forest's
  width and a list of occurrences), on a sequent of 65 000 `!` clauses a
  list of that length for each of as many nodes. It is reached from two
  places: the harness's check of the proof (the baseline's `crash`
  rows, which therefore say `unknown` where the search answered
  `proved`), and `Derivation::build`, so the command's default text
  output and every drawn format: `linlog prove -i --file` on
  TokenRing-40 without `--quiet` takes 6 GiB in 2.3 s, with no cap of
  its own and after the search, where `--timeout` no longer applies. On
  a machine without a limit that is the whole memory in half a minute.
  JSON output is unaffected (5.5 MB in 0.3 s). The fix is the one "the
  benchmarks" below names for the additive identity (what a node
  derives shared along a branch, or kept only while a premise still
  needs it); and the command wants a bound on the derivation it builds
  whatever the checker does, since the tree itself is that large
  ("Follow-ups: the command's output" below).
- **The search's own memory** was step 20's
  (`plan/reports/20-memory-and-boundaries.md`): a bound in bytes
  (`Options::memory_limit`), a memo of records in chunks, the kept
  arena collected when the memo is emptied, the counts' tallies as wide
  as the atoms that have rows. What it left:
  - *The zones of a memo key are bitsets of the forest's width*, nearly
    all of an entry on a forest of thousands of occurrences (528 of 536
    bytes on `SYJ202+1.008` in cbv); a sparse form would multiply what a
    bound holds. Step 26, with the memo's other constant factors.
  - *The pool's arena is not collected* (its workers hold ids nobody
    can rename), so a pool reaches the bound sooner than one thread on
    a search that proves much.
  - *A memo starved by a small bound makes a search slow, not
    "unknown"*: whether "unknown" should come earlier is step 21's
    question of what an "unknown" tells the user.
  - *The reason of the default bias's two searches is the backward
    one's*, so a forward search that ended at its memory bound is not
    what the user reads.
  - `Forest::build` allocates a vector per occurrence (half a million
    allocations on a net of that size), and `Forest::lca` is a parent
    walk, so the net engine's "constant-time" rejection costs the depth
    of the formula.
  - A proof file and a session's state are read under the default
    occurrence limit, whatever `--occurrence-limit` says: serde's
    `Deserialize` takes no options. Step 28.
  - *The checker on a hostile file* (the review of step 20 assigned
    these): its pass polls no stop and can be made quadratic in time
    within flat memory (step 28, with the checker's interface); an error
    report with formulas is not bounded (step 22); the crate's hasher
    has a fixed seed, which the checker's tables now face untrusted
    input with (deferred: a seed per process would cost the
    reproducible runs, a second hasher for the checker would not); the
    32-bit case of every integer argument (step 34, under wasm32).
- **The forward search misses its stop on the GPPP-1000 nets, by
  minutes.** On one thread they are killed past 10.5 s under a 5 s
  limit, or proved 0.7 to 1.0 s late, in the default and the `--bias
  factors` passes and never under `--bias rarer`; the kill hides how
  late the stop is: `linlog prove -i --jobs 1 --timeout 5s` on
  `GPPP_G-PPP-1000-10_10_1` (24 clauses, a marking of thousands of equal
  tokens) answers "provable" after 140 s (568 stable sequents, 4.75
  million split steps; on an efficiency core), with a limit of 60 s the
  same, and on `GPPP_G-PPP-1000-1000_5_1` after 11.2 s. On a pool the
  limit holds (5.4 s on four cores), so the command's default is not
  affected, and `--jobs 1`, `--deterministic` and a library caller on
  one thread are. Somewhere between two stable sequents the forward
  search does thousands of split steps without a poll. Other stops come
  up to 3.1 s late on one thread (`PaceMaker_20_1` under `--bias
  rarer`) and 1.6 s on a pool. Since step 27 the nets go to the Horn
  engine, so this is met only with `--engine focus` or `two-sided`, or
  on a Horn-shaped goal the Horn engine does not take (affine mode).
- **The copy bound**: of the 832 problems outside the nets that the first
  baseline ended at the copy bound of 3, a bound of 10 decides 369; the
  forward search alone at 30 copies decides 288 problems outside the nets
  that the default leaves at 3, all in under 5 s. Whether `--copies`
  should rise is step 17's question.

- **The generators' contraction and mutants** (step 28's F122 and F123,
  begun in area 3.2 and left there, 2026-10-10). F122: the intuitionistic
  generator's contraction was a no-op in 98.5 % of its draws; a fix that
  derelicts a hypothesis and contracts it beside an identity on it
  (change `supyqoln`, workspace `step28-gen-tests`) gives 65 to 131
  effective contractions per exponential rule set on the tests' seeds,
  where there were 0 to 3, but the mutant of one of the new samples
  takes the classical engine 158 s unoptimized (`b, b, (!!!c * !b),
  (((b -o !b) -o (b -o (b * b))) -o !((b -o c) * (a -o c))) |- ((((((b -o
  c) -o c) * b) * a) * b) * (b -o b))`, the first hypothesis `a` shifted
  to `b`, so unprovable: the goal's `a` has no `~a` anywhere; undecided
  within the copy bound), which doubles the debug test suite. F123: every generated negative of the rule sets
  without additives and exponentials is refuted by the interval check
  before any search; a second mutation that transposes two literals of
  one sign (change `ukrmvowr`, unfinished) sends 10 to 25 % of them to a
  search, and found a sequent the focused engine searches for over
  twenty minutes unoptimized without deciding: `|- (?((((!~a par c) + c)
  par a) par ~c) par ?((~c * ~c) * c))`, classical with Mix, the default
  options. That sequent is the first thing to look at: a Mix search
  that the copy bound of 3 does not end in reasonable time. No engine
  disagreed with another or with the reference prover in either run.
  **Landed in step 28i** (2026-10-14): both sequents are unprovable; the
  Mix one was slow because its failures rested on the loop check and
  were never memoized, which qoksszzp fixed (the failures wait on their
  ancestor); F122's contraction is wzynqlqo and F123's swap ypvxqstw,
  each with its effect pinned (`intuitionistic_proofs_contract`,
  `some_mutants_reach_a_search`) and the slow mutants under the tests'
  work bound. The bookmark `wip/generators` is deleted. Since kvpyrzuv
  the count of one sign refutes F122's mutant at once (below).

- **A count of one sign for the atoms under exponentials**: built in
  step 28i (kvpyrzuv, 2026-10-14, the author's answer to open question
  11 of step 28's report). An atom whose literals below the
  exponentials all have one sign keeps its count rows, and a balance is
  ruled out only on the side the copies cannot reach
  (`Refutation::OneSided`); F122's sample is refuted at once. Left: the
  tests' work bound (`MUTANT_WORK`) stays, since other mutants still run
  long, and the count's reach on LLTP is small (three problems more).
  A tightening the panel's argument lens noted: a literal below a `!`
  that lies below no `?` is used once in linear mode (a `!A` in the
  linear zone is decomposed by promotion alone), so the test of both
  signs could look at the literals below a `?` only; the counts treat
  `!` as `?`, as they did before. Sound either way; to measure first.

## Follow-ups: the Horn engine

Left by step 27's first session (`plan/reports/27-horn.md`):

- **The nets it leaves undecided** within 5 s (66 of the library's 3 137
  after step 27's second session, 60 at the time limit and 6 at the
  memory bound): goals far from the initial marking in large state
  spaces (the BART, CloudDeployment, DES and AutoFlight models), where
  the distance in tokens leads the greedy search onto plateaus. Candidates, each to be
  measured: the state equation (`M₀ + C·x = M` over the integers, or its
  rational relaxation) as a prune and a heuristic, partial-order
  reduction (stubborn sets), and structural reductions of the net.
- **What no refutation reaches yet** (step 27's second session added the
  state equation, dead transitions and the backward search): a net whose
  markings grow without end both ways while its state equation has a
  rational solution and its goal is unreachable for a reason of
  integers or order (`!(a -o a * a * a), !(a * a -o b), !(d -o d * d),
  !(d * d -o d), a, d |- b * d`: the `a` stay odd). Candidates: the
  state equation over the integers, trap constraints (Esparza and
  Melzer), and the equation refined by the backward search's markings.
  The step's review met one on its random programs near the Horn shape,
  which the focused engine refutes and the row now leaves unknown (1 of
  3 751, linear mode): `!((c * d) -o (d * 1)), !(a -o 1), (a -o (b *
  d)), !((d * d) -o 1), !a |- b`, where `d` stays odd.
- **The state equation misses certificates whose weights span more than
  2²⁰** in the ratio of the largest (a chain of forty doublings, exact
  weights `2ⁱ`): the floating-point simplex's vertex reads as zero below
  10⁻⁹ of the largest weight. An exact rational simplex on small nets,
  or reading the weights off the basis exactly, would find them.
- **The simplex's basis is dense**: a net of tens of thousands of places
  (qcover's largest has 66 950) never gets the equation, its basis
  passing the memory bound. A sparse factorization (an LU of the basis,
  updated per pivot) would reach them.
- **Memory at tight bounds**: the trie of the backward coverability
  search takes 30 to 75 % more memory than a scan would, its hash table
  is charged at about 14 % under what hashbrown allocates, and at a bound
  of a few kilobytes the checker gives up checking a proof the engine
  found (an error, exit 2), as before step 27.
- **The shares are set by a sample**: the simplex's fifth of the time in
  linear mode and half in affine mode, the backward search's quarter and
  its start after 2¹⁴ markings were chosen on a sample of 50 library
  nets and the qcover runs; a baseline should look at them again.
- **Coverability that the backward search leaves**: the counter is
  exponential backward (128 tokens: 6 s in affine mode, 2.5 ms forward in
  linear mode); a forward search for a covering marking beside the
  backward one would prove such goals. qcover's own algorithm prunes
  every backward element whose rational state equation from the initial
  marking has no solution (an LP per element), which is what its paper
  measures on the problems this engine leaves at 5 s.
- **The frontier keeps duplicates**: successors are written out only
  when taken, so a marking reached by many paths is on the frontier many
  times (sixteen bytes each); on nets of high branching that is what
  meets the memory bound (`DNAwalker_dnawalk-18_20_1` after 1.3 s).
- **Constant factors**: `Program::read` runs three times per call (the
  dispatch's feature, `admits`, `decide`; the dead transitions are
  dropped in `decide` alone), and a goal refuted by the
  counts or by exhaustion has its counts built twice (the engine's test
  before the search, `prove_goal`'s reason after).
- **The counts before the search are quadratic** where atoms outside
  every `!` sit in nested tensors (a long clause used once, a goal of
  thousands of distinct atoms: 20 000 meet the 1 GiB bound, which the
  search alone decided in 20 ms). Only atoms with rows can refute, so a
  test that builds rows only for atoms the goal could leave unbalanced,
  or skips the counts when every atom is in a clause under `!`, would
  keep the cheap case cheap.
- **A shortest firing sequence** (breadth-first) would give the smallest
  proof, for teaching; the order is the engine's scheduling today, not an
  option.
- **Horn programs without `!`** stay with the focused engine, whose
  counts decide the Partition encodings 10 to 100 times faster
  (`partition-yes` 12: 15 ms against 2.0 s; 16 and 20 not within 10 s).
- **The classical library's nets** (`CLL`) go to the same engine and
  were not measured on their own.

## Follow-ups: intuitionistic mode

*Assigned (2026-10-03).* The written succedent: step 31, whose
two-sided statement carries it. The rest stands with no need shown.


Left open by step 8, none of them a correctness issue. The written
succedent: for formulas built from `⊤` and `0` alone the reading's goal is
the last root by id, not the written one (`0, ⊤ ⊢ ⊤` prints as `0, 0 ⊢ 0`;
provability never differs, 607 464 cases brute-forced), and recovering it
means the arena keeps the parser's root order or the count of right-hand
roots, a change to `Sequent`'s canonical form and JSON; do it only if a
user of the two-sided print or the certificates asks. The additive path on
more than two roots, and on a `!` of an additive formula, is decided by
the focused engine today. The canonical choice among identical hypotheses
(step 15) applies two-sided as well.

- **The memory bound is passed on wide intuitionistic searches by default**
  (found at the review of step 29, 2026-10-16; *done in session 30b,
  2026-10-17*: the search's account held, and what passed the cgroup
  was the input, which the bound now counts, with each search's reading
  and the verdict's check after a race; `plan/reports/30-baseline-release.md`). `linlog prove -i --file
  bench/lltp/ILL/ILLTP-SYJ-01/SYJ206+1.020.p --memory-limit 4GiB --timeout
  10s` on four cores peaked at 4 267 MiB of resident memory and ended at
  the time limit, not at the memory bound (the two-sided engine at copy
  bound 4, 55 589 stable sequents, 4 224 memo entries at most, 11.4
  billion units of work); on one thread it peaked at 2.1 GiB. In step
  29's run the cgroup ended this file and SYJ212+1.020 (two translations)
  at 4 GiB. Something the default's race allocates is not charged to the
  bound: find it (the second search's own structures, the pool's
  workers, or a proof of `&` rules outside the focused engine's charge),
  charge it, and pin it with a test under a bound.

## Follow-ups: ordinary logic

Left by step 25 (`plan/reports/25-ordinary-logic.md`); 1 and 4 of its
list, the JSON forms and the certificate's import as an option, and a
stop for `Image::read_back` and `Derivation::check`, went to step 28.

- `interact --logic`: a session over the image whose display is read
  back, the goals shown as ordinary sequents.
- A loop check for dyadic sequents (the section above): 35 of the ILTP
  problems, all refutations.
- Classical logic through affine MALL costs one branch per way through
  the conjunctions: 116 of the 274 are unknown at 2 s, all large SYJ2xx
  instances. Not a SAT solver, and not to be offered as one.
- A derivation read back from a proof that shares subproofs unfolds
  exponentially: seven classical proofs (8 to 32 million inferences)
  could not be checked within 6 GiB. A read-back on the proof term (a
  DAG), with a checker on terms, would avoid it.

## The web front end: a two-sided presentation of classical derivations

*Assigned to step 40's second session (2026-10-23).*

*From the design session (2026-10-18, `plan/web/design.md`, D58).* The
author wants a toggle between the one-sided and a two-sided
presentation of a classical linear-logic derivation on the page. The
library draws classical derivations one-sided only (`svg::two_sided`
serves intuitionistic mode); a two-sided reading of a one-sided
derivation (the antecedents moved left of `⊢` by their polarity, the
rules renamed) is a presentation pass over `Derivation`, a feature of
its own, after step 34.

## Follow-ups: interactive proving

*Assigned (2026-10-03).* The empty goal a one-sided Mix opens, and
`close_all` dropping its outcomes on an error: step 28. The reading
recomputed per operation, a filtered `rules` list, a budget per goal for
`close_all`, a map from a drawing's inferences to the session's goals:
steps 22 and 34. The net engine on a sub-forest: deferred.


Left open by step 9, none a correctness issue. The net engine works on a
proof structure over the whole forest, so an MLL goal off the roots goes
to the focused engine and `Engine::Net` forced on it is `Error::NetGoal`;
a structure over a sub-forest (the goal's subtrees as conclusions) would
let the net engine close such goals, worth it only if the front end's
profiles show `close` on wide MLL goals. The intuitionistic reading is
recomputed per operation (O(n) in the forest), since `Reading` borrows the
forest the state owns; positions stored in the state fix that if a client
with thousands of occurrences asks. `rules` lists by connective and mode
only, and `apply` says what the context lacks; a fully filtered list means
trying each rule on a clone. A Mix that sends every formula to one side
opens an empty goal that nothing closes (nullary Mix is not a rule);
refusing that split is a one-line policy decision. `close_all` runs the
goals in order under one stop closure; a per-goal budget is the client's
wrapping. Reading a state back requires the two-sided rule names that the
library writes, while `apply` also accepts the classical name in
intuitionistic mode; harmless for the library's own JSON.

## Follow-ups: the exports

*Assigned (2026-10-03).* All of it is step 22's. Reporting the curryst
limit upstream is done by others (issue 19, closed as not planned); the
layout of linlog's own follows.


Left open by step 10 (`plan/reports/10-latex-typst.md`, "Open questions").
Typst refuses a curryst 0.6.0 tree more than about eleven inferences
high ("maximum show rule depth exceeded", curryst nesting several layout
elements per level), so the Typst export is for small proofs: report it
upstream, and if it stays, write the tree with linlog's own layout (the
subtree widths of step 11's SVG, emitted as a Typst `grid`/`stack` with
explicit widths) instead of curryst, which also removes the package
import. Greek atom names under pdfLaTeX (`α` to `\alpha` in one table)
if users name atoms that way. `interact`'s `proof` could take a format
with a spelling that does not collide with its file argument (`proof
--latex`). The rule-label convention (upright `L`/`R`, subscript `1`/`2`,
`?d`) is one table per target for a user who wants another.

From step 11 (`plan/reports/11-svg.md`, "Open questions"), for the web
front end above all: a `<g>` per formula of a goal's sequent with the
position in its id, so that a click on a formula maps to `(InfId,
position)` for `Interactive::apply` (today `i<n>` names a whole
conclusion); edges and links meeting a negated literal at its atom rather
than at the middle of `A⊥`; a nesting-safe cap on the height of wide
axiom links (the height grows with the width, and a plain cap makes an
inner arc poke through its outer one); colouring a disconnection
(`NetError::Disconnected`) as the switching cycle is coloured; Greek
letters as mathematical italic code points to match Typst; and a smaller
row height for trees without a raised `⊥` (`Style::line_height` is the
knob).

## Follow-ups: the command's output

*Found by step 30's release review (2026-10-17, finding 15; unassigned).*
The batch's and `check`'s records write `wire::LEVEL` as a constant,
which is right at level 1 and wrong once a level 2 exists: they should
write the level of what they contain.

*Assigned (2026-10-03).* Step 18, as the author decided: on a terminal a
tree is shown only if it fits; into a file or a pipe it is written
unless its estimated size passes a safety bound (64 MiB by default, an
option, D16). The compact view is step 22's.

*Done by step 18* (`--tree`, `--derivation-limit`, `ViewOptions`,
`Proof::derivation_size`; `plan/reports/18-bounded-proofs.md`). What it
left for step 22: the exports return a `String` and cannot be stopped
inside; the SVG layout takes 50 bytes of memory per character of
sequent, six times the other formats; the command assembles its output
as one string, twice the text tree's size at the peak; `SCREENS` and the
text tree's `GAP` are constants; `check` prints the whole sequent in its
verdict line. For step 20: a malformed proof term (`Mix(p, p)` repeated)
doubles a zone of the checker per node; the review of step 18 found that
this made the zones' counters wrap and a non-proof pass, and added the
test (a zone longer than the goal plus twice the nodes still to come is
refused, `Problem::Surplus`). Left for step 20: the clone per reader of
a shared node, the other integers of the pass, the verdict lost when
`--derivation-limit none` meets a derivation larger than memory.
For the web front end: the derivation builder recurses to the
derivation's height, which `Size::height` tells beforehand.


**A proof tree that does not fit is not printed** (the author,
2026-10-03, after the review of step 16 showed the command's default
output taking gigabytes on a large net): where the tree does not
reasonably fit the terminal, `prove` prints the verdict and leaves the
tree out, with a manual override. What to do when the output goes to a
file the author left open; the rest of this entry is the planning
session's recommendation, for step 17 and the author to settle.

Two bounds that are different in kind, and one rule for each:

- *Fit*, on a terminal only. When standard output is a terminal and the
  format is the text tree, the tree is shown if its widest line fits
  the terminal's columns (a wrapped tree is unreadable at any height)
  and its height is within some screens. Otherwise the verdict line is
  followed by one line that says how large the derivation is and names
  the ways to get it: the override, `--output FILE`, `--format json`.
  The switch is of the kind `--color` is: `auto` (the default), `always`
  and `never`, `--quiet` staying what it is.
- *Safety*, everywhere. Into a file, a pipe or a drawing format there
  is nothing to fit, the user has asked for the derivation by naming
  where it goes, and a script must get the same output whatever the
  terminal: so it is written, unless its size, estimated before
  anything is built (inferences times the width of their sequents),
  passes a bound of the order of what an editor or a typesetter can
  still open. Past that bound the verdict is written without the
  derivation, a line on standard error says why and names `--format
  json` and the option that lifts the bound, and the exit status stays
  the verdict's, which is what scripts read. `always` on a terminal is
  under the same bound.

The estimate and the bound belong to the library, in the options value
of the derivation's output (D15), so that the web front end and a batch
mode decide as the command does; whether a tree fits a terminal is the
command's own question. The estimate must not build what it measures:
a pass that counts, not `Derivation::build`. A compact view of a large
derivation (a run of one structural rule drawn as one inference, the
65 641 `?` steps of a net as one line) would move both bounds far out
and is the better answer for teaching; it changes the type the exports
read, so it goes with the code audit.

## Follow-ups: parallel search

*Assigned (2026-10-03).* The false assertion of `agree`, the missed
stop on the SYJ problems (which step 17 reproduced at two and four
threads on a 19 KB file), the net engine's cubes on sequents with forced
links, a bound on `--jobs` and the portfolio's removal: step 19. The
pool's memory on the largest files: 20. The default thread count: one
thread first, then the pool (the author's answer; step 21). A `Runtime`
kept across calls: 24. "The parallel tests take about a minute" no
longer holds (5 s for the core crate's 122 tests). The rest is deferred.

*After step 19 (2026-10-03).* Done there: the stop at a `&` on the pool
(a stopped premise started both premises of every `&` below it, which
was the SYJ miss), the cubes of the net engine split in place with
forced links followed, the threads bounded by the machine's parallelism,
the portfolio removed, `agree` asserting the contract. Left, and
assigned: a cancellation at a `&` that comes late and an error of one
premise that cancels the other, with a differential run at small
recursion limits: 26. Left, and deferred: where the list of cubes stays
short (one surviving branch per choice) the root engine does the whole
net search with a seed per cube while the workers wait; the pool's
speculative work on towers of `&` is bounded by cancellation alone, and
nested scopes still stack on a waiting thread.


Found by the review of step 15's second session, and older than it: the
doc comment and the second assertion of `focus::parallel::tests::agree`
claim that a pool proves exactly where one thread proves. That is false
(`b, ((a * 1) -o !a), !(b -o 1), !(1 -o ((1 * 1) -o a)), b |- (!!a * b)`
with a copy bound of 2 is at its bound on one thread and proved on
four, every time); the test passes on its samples only. The contract in
`.claude/rules/core-parallel.md` is the true one (decisiveness within the bound
may differ either way); the helper should assert that and no more.

Left open by step 13 (`plan/reports/13-parallel.md`, "Open questions").
A per-worker proof arena with a relocation pass at the merge
(`Proof::new` renumbers already, so `(worker, index)` ids are a bounded
change) if a profile ever shows the shared arena's lock, which the
report's table does not. A `Runtime` kept across calls for a caller that
closes many small goals (a web server, an `interact` session), which
needs a runtime value in the public API next to D9's plain-data options.
The duplicated exploration of and-parallel `&` premises and of copies as
alternatives on memo-bound families, which is where the parallel focused
engine gains little (the baselines of steps 14 and 16 measure it). What
the first baseline adds, for step 17 to weigh with the second: on the
LLTP problems one thread does not decide at once, sixteen threads are
2.2× slower in the median and decide 32 against 28 (11 gained, 7 lost),
so the CLI's default of every core is in question for small problems (a
smaller default, or a sequential first attempt of a few milliseconds);
the portfolio gains nothing on the families nor on LLTP and is a
candidate for removal; QBF gains 1.8× from the second thread and nothing
from more; and a pool on the largest SYJ problems (8 to 15 million
occurrences) needs more than 12 GiB where one thread stays within it,
presumably the workers' per-forest state (not looked into). A
thread-sanitizer run needs
nightly and a rebuilt standard library; the code has no `unsafe` and
every shared value is behind a lock or an atomic, so it stays a wish.
The second baseline: on the 1 102 LLTP problems sixteen threads decide
623 within 5 s against 586 on one thread under the default (37 gained,
none lost) and are 2.9× slower in the median on those both decide; the
portfolio decides 627 (10 gained, 6 lost) at the same time (1.01×), so
it gains nothing again and can go. On the families the speedups are
mostly gone with the refutations in microseconds: Partition with 12
items refuted 4.7× faster on sixteen threads, the counter (two searches
side by side, a pool each) 3.5× on eight, QBF 1.2× from the second
thread and nothing from more, Mix slower with every thread; and
`partition-yes` at 24 items is proved in 106 s on one thread and times
out at 120 s on every pool. Sixteen SYJ202 and SYJ208 problems in their
cbv translation (2 000 to 12 000 occurrences) are killed past 10.5 s on
sixteen threads in both baselines, where one thread stops at 5.1 to
5.4 s: a stop a pool misses.
What the ratio leaves out (the review of step 16, from the same rows):
in time, the 586 problems both decide take 1.1 ms in the median on one
thread and 4.6 ms on sixteen, the difference is 0.7 ms in the median
and 9 ms at the ninetieth percentile, 15 problems are more than 100 ms
slower on sixteen threads and 106 more than 100 ms faster, besides the
37 gained. So the command's default of every core costs a user
milliseconds nobody sees on small problems and gains on the large
ones, and a pool keeps the time limit where one thread's forward
search misses it by minutes (the focused engine's follow-ups). The
case against the default is the laptop's other work and the memory of
a pool on the largest files, not the time.
The parallel tests take about a minute in debug builds; trim the samples
if the suite's time matters more than the coverage.

## Follow-ups: the benchmarks

*Assigned (2026-10-03).* `derive`'s memory, the verdict written before
the check, the kill counted from the end of the load: step 18 (done:
the checker keeps no sequent a later node does not read, the child
prints its row before it checks, `--load-limit`; whether
`bench/reruns.txt` is still needed for the large SYJ files was not
measured). `summary`
counting a late verdict as solved, a table of counters, the script's
values hard-coded for the second baseline, `bench/reruns.txt`: 28. LLTP
input for the command and the draft of the header report: 24 (the author
sends it; step 30 reminds). The net engine's test period: 35. Problems
from practice: the table at the top.


Left open by step 14 (`plan/reports/14-benchmarks.md`), beyond the two
baselines, which are steps 14 and 16. Twenty-eight LLTP files have
headers that contradict them (23 headers: KLE065, SYJ212+1.001, SYN001,
KLE013, SYN041, SYN915, and, found with a copy bound of 10 and confirmed
by classical countermodels, KLE017, KLE069, KLE078, KLE088, SYJ103,
SYJ105+1.003 and +1.004, in the translations the report lists; and three
files more that the second baseline's passes with 10 and 30 copies
refute, found by the review of step 16 and confirmed the same way:
KLE069 in `KLE-01`, and KLE078 and KLE086 in `KLE-cbn`, each a
translation that lost a negation of the ILTP original under
`bench/lltp/ILTP+KLE`) and
SYJ206+1.018 in its 01 translation has a tab for a closing parenthesis
(repaired in the flake's fetch): worth reporting upstream with linlog's
checked proofs and the countermodels attached. The CLI does not read
LLTP files, a one-flag addition over `linlog::lltp::read`. The net
engine's exact test could run less often on large structures (a period
of 16 was 1.39× faster at 14 000 occurrences). Matsuoka's 3D-Matching
encoding is not among the families. `summary` counts a verdict found after the time limit as
solved, which a long grace makes possible (step 16 keeps those apart in
its comparison). The largest SYJ files (up to 103 MB) load in up to 16 s
with 2 GB, past the default kill, so only the reruns of
`bench/reruns.txt` reach their search; a kill that counts from the end
of the load would make the list unnecessary for them. The proof
checker's `derive` keeps a bitset of the forest's width for every node
of the proof, 7.6 GB for the additive identity of depth 16 (262 141
nodes of 32 KB), which is what the first baseline took for the additive
memo; a `Θ` shared along a branch, or a set of ids, would fix it, and
the caps of `bench/baseline.sh` are sized for it until then. Since the
performance pass it is also what ends 14 large nets per pass of the
second baseline and the command's own output on them (the focused
engine's follow-ups above), so it is no longer the harness's matter
alone. The harness should also write the search's verdict before it
checks, so that a check that dies leaves the row its verdict and says
`checked` failed. `summary`
prints times, not the counters that step 15's comparisons rest on; a
table of `nodes` and `splits` per configuration would serve the code
audit's oracle.

**Problems from practice** (the author's question during the second
session of step 15, 2026-10-02: the point of making the tool efficient
is that it can be used). The benchmark's one real-world set is the LLTP
library's 3 137 reachability problems over 76 Petri nets of the Model
Checking Contest, and they are theorems by construction: each goal is
the marking a replayed firing sequence of 1 to 150 steps ends in. What
is missing, as candidates whose sources, formats and licences are not
yet checked:

- real non-theorems on nets: coverability suites from software
  verification (the nets of concurrent C and Erlang programs that Mist,
  BFC and Petrinizer are measured on) have unreachable targets, and
  coverability is affine mode, which no set from practice exercises;
- planning domains (blocks world, logistics) as `!` Horn clauses, the
  nets' shape with goals that fail;
- program synthesis from linear types (the Granule synthesis
  benchmarks, for one): small ILL sequents with additives where the
  proof is a program;
- llprover's example collection: classical LL with additives and
  exponentials together, which the generated families barely cover.

Each enters as a problem file under `bench/problems/` or as a source in
`bench/src/problems.rs`. The nets among them are also what the forward
search's own bound and the unit of work of the default bias should be
tuned on, beyond the 109 sampled LLTP problems they were set on.


*Instruction counts as the ratchet's measure (unassigned, 2026-10-22,
the author's question at the update to rustc 1.99.1).* What the sources
say: rustc's perf suite gates on `instructions:u` because wall time
varies too much to see a change under 1 % (its author measured
instruction counts within ±0.2 % on a no-op change, wall time to ±9.7 %;
<https://internals.rust-lang.org/t/what-is-perf-rust-lang-org-measuring-and-why-is-instructions-u-the-default/9815>);
SQLite measures with cachegrind for the same reason (repeatable to seven
digits, wall time to one; <https://www.sqlite.org/cpu.html>); rustls
gates on callgrind counts and keeps a wall-time mode beside them,
because counts "do not account for important factors such as cache
misses and branch mispredictions"
(<https://github.com/rustls/rustls/blob/main/ci-bench/README.md>).
Callgrind counts instruction reads alone unless told to simulate the
caches (`--cache-sim=yes`) or branches (`--branch-sim=yes`;
<https://valgrind.org/docs/manual/cl-manual.html>). And wall time is no
safe arbiter either: a program's time moved by a third with the size of
the environment, once by 300 % (Mytkowicz et al., "Producing Wrong Data
Without Doing Anything Obviously Wrong!", ASPLOS 2009). What linlog's
own runs showed: the readers rose 4 to 6 % in count and 7 to 8.5 % in
time under 1.99.1, read-spec rose 6 % in count and ran 4 % faster,
`render-svg`'s ceiling was raised three times in a week for the
allocator's paths in another binary layout, and the counts' layout of
2026-10-22 (an atom's sums together) cut 10 to 17 % of the count where
the gain in cache lines is not counted at all. The search's memo tables
and bitsets are where cache behaviour matters and counts see least.
Taken on the author's word (2026-10-22: the cache model's misses beside the counts, `.claude/rules/bench.md`); the proposal was: keep the counts as the per-commit tripwire, cheap and
deterministic on a shared laptop and in CI; add callgrind's cache
simulation (L1 and last-level misses, deterministic too) as columns
beside each journey's count; read a toolchain's or allocator's move as
a question for pinned time, never as a regression by itself; and leave
the baselines' wall time, on an idle machine, the arbiter of speed.

## Follow-ups: certified refutations

*From step 31d (`plan/reports/31-rocq-library.md`, "Session 31d:
certified refutations"; unassigned).*

- **The comparison counts linlog's unprovable as unchecked.** Its
  driver gives a checked mark to a proof only; a disproof whose
  refutation `Disproof::check` accepts is as checked. The next
  comparison's driver reads the outcome's refutation and checks it.
- **The one-sided and state-equation refutations** have neither a
  check nor a Rocq lemma, so they stay uncertified; each needs its
  invariant stated, checked without the search, and proved.
- **`linlog check` reads proofs only.** A disproof's JSON could be
  read and checked the same way, with the exit statuses of a proof.

## Follow-ups: the release's licences

*Found by the licence audit before 0.1.0 (2026-10-18, unassigned; the
first change after the release).* The `linlog-cli` binary that the
public cache holds links some 140 crates under MIT, Apache-2.0, BSD,
Zlib and Unicode-3.0 statically, and several ask for their copyright
notice and licence text in every copy, binaries included; the package
installs the EUPL's text and the font's OFL only. A notices file
generated from the lock (cargo-about, through the new-tool skill),
installed beside them under `share/doc/linlog-cli/` and checked by the
flake so that it follows every `cargo update`, closes it. The crates on
crates.io and the release's `.crate` files are sources and need none.

## Follow-ups: a first user's obstacles (step 28's last reader)

The last reader of step 28 (2026-10-14, a fresh context reading README
and the rustdoc's front page as a newcomer, against
`plan/notes/research/usability-baseline.md`) met these, which the step
left for later; the fixes it made are in the step's report.

- **An unknown that more resources will not decide.** On `!(A + B) |-
  !A + !B` the verdict line sends the reader from `--timeout` to
  `--recursion-limit` as if a larger limit would decide it. Linear
  logic with exponentials is undecidable, so no line can say when it
  will not; what could be said is how far the deepening went without a
  proof and that a copy bound (`--copies N`) ends the search with a
  definite answer within that bound. The wording needs measurements on
  the LLTP library of which unknowns a larger limit ever decided.
- **Rocq.** A refused certificate (`prove --mix --format rocq`) prints
  the refusal alone and exits 2, so the verdict is lost; printing the
  verdict as a comment first and exiting with the verdict's status would
  change what cli.md pins (a refused certificate leaves standard output
  empty). A batch's certificate files take the entry's name (`modus
  ponens.v`, not a module name) and every lemma is `certificate`; README
  shows how to compile a certificate but not how to apply its lemma to a
  two-sided goal in another file. All belong with the Rocq library of
  linlog's own (above).
- **Nets on the command line.** `check` reads proofs only: a
  `ProofStructure` in JSON, as `wire` documents it, has no command that
  reads or writes it (`prove --net --format json` is refused). A `check
  --net FILE` printing the criterion's verdict, and `--net` in JSON, go
  with the MALL proof nets above.
- **The style table.** `--style labels.table.RULE=LABEL` on a table not
  yet set fails with "labels is not an object", and no help lists the
  rule keys; a dotted path that makes the table, and the keys on
  `labels`' line, are the fix.
- **`check --file`.** Every other command reads its input with `--file`;
  `check` takes the proof as its argument (or standard input).
- **The library's front page.** The examples of a stop all read
  `Instant`, which a browser lacks, though the page says the crate reads
  no clock; one example whose stop counts `Progress::work` against a
  budget, and one that builds the command's behaviour from
  `Settings::default()` (the clock's limit as a stop), would let a web
  developer and a front end copy rather than derive. The Euler Math
  font's source and licence are named in the command's tree
  (`cli/fonts/`), not where a reader of README looks.
