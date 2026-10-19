# Step 37: the focused inverse method

## Session 37a: the engine and its measurement

**Outcome: the engine is built, reviewed and measured, and it wins
rows; 37b is needed.** `Engine::Inverse` (`--engine inverse`) saturates
forward in every commutative mode, decides without exponentials, proves
with them without a copy bound and refutes where its saturation ends
(`Exhausted`, without a certificate). At 60 s on one thread it decides
all ten of Sympli's class C proofs (in at most 0.3 ms; the default
decides one of them, in 14.5 s) and all six refutations (in at most
0.64 s; the default two, in 21.5 and 50.7 s), 113 of the 384
intuitionistic rows no tool decided at 10 s (44 proofs, 69
refutations; the default at 60 s 26 of them), and on the held-out sets
38 rows the default leaves undecided against 16 it loses; on the 713
held-out rows both decide it takes 0.93 times the default's time
(geometric mean; 65 s against 187 s in all). On the generated families
it loses: 45 of 123 decided against the default's 111, none won (the
Partition, QBF and wide sequents, where the focused engine's counts and
splits are far ahead), but it refutes Mix's family in 0.05 ms where the
default takes up to 60 s. No verdict of either engine contradicts the
other's anywhere. The panel's four findings were fixed and re-checked;
the dispatch is unchanged and both target sets are unchanged on every
row that ends within its limit.

### What was built

`Engine::Inverse` (`--engine inverse`, JSON `"inverse"`), in
`core/src/search/inverse/` (the engine's module: `mod.rs` the
saturation, `rules.rs` the derived rules, `eval.rs` a rule evaluated
forward, `index.rs` the tries of subsumption, `tests.rs` its tests),
behind the engine interface (`search::Decide`), in every commutative
mode: classical linear, affine, with Mix, intuitionistic and
intuitionistic affine. It refuses the ordered modes (`NotTaken::Mode`)
and nothing else; the dispatch gives it no goal.

- **Rules are terms with holes.** Before the search the forest is
  compiled into one rule per focusing phase (`rules::compile`): a focus
  on each positive formula a stable sequent can hold (`D1`: its parent is
  `⅋`, `&`, `!` or none) and a copy of each subformula of a `?` (`D2`),
  under each choice of its `⊕` sides (`plans`), written as the proof term
  of the phase in postorder (`Step`), the axioms of its positive
  literals as slots of partners (negative literals of the atom, taken
  from the linear zone or, for the subformula of a `?`, from the
  unrestricted one under a `Copy`), the asynchronous phases of what it
  releases (`⊤` first, then `⅋`, `⊥` and `?`, `&` last) ending in holes,
  each hole knowing the members it takes and its release (`Hole::group`).
  The goal is one more such term, its own asynchronous phase
  (`rules::goal`), whose holes the search fills. The research note's
  recommendation, "rules as data, not closures, so that the proof term is
  emitted while matching", is this.
- **Evaluation mirrors the checker** (`eval::Calculus::derive`): a step
  derives what `core/src/proofs/check/` derives of that rule, the
  checker's `any` flag as `top` (absorption by a `⊤`, R2 under a
  reading), affine weakening as explicit `Weaken` nodes (never an output,
  R3), the checker's `&` join, promotion's empty zone, R1 at every step.
  The same function evaluates during the search and writes the proof
  afterwards from each kept sequent's record (rule, premises, partners),
  so the proof returned is the one judged, and the front door checks it.
- **The saturation** (`Saturation`): the rules without holes give the
  first sequents; the smallest sequent waiting (members of both zones,
  then age) is taken up, put into every hole it can fill beside those
  taken up before it (each combination applied once, when its last
  premise is taken up), with Mix joined to each one taken up where the
  mode has it and is not affine; every derived sequent is kept unless a
  kept one subsumes it or it cannot stand in a proof of the goal (below);
  the goal is proved once each of its holes has a kept sequent that fills
  it, and `Unprovable` when the queue empties.
- **Subsumption and the index.** A derived sequent `Γ ; Θ` stands for
  exactly `Γ` with any unrestricted zone holding `Θ`, and with `top` or
  in affine mode for any larger `Γ` too. Two tries (`index::Trie`) over
  the sorted members of `Γ` (repeats spelled out), a separator and the
  members of `Θ`, one of the exact sequents and one of the absorbing or
  weakening ones, answer "same `Γ`, `Θ` within" and "both within"
  walking only the paths whose members the query holds. Sequents killed
  later (a kept one subsumes them) are skipped when popped and in new
  combinations. The postings of the sequents taken up by member find the
  premises for a hole.
- **Sequents as sorted lists of occurrences**, not the focused engine's
  bitsets of the forest's width: the intuitionistic library's forests
  reach millions of occurrences, its stable sequents a handful of
  members, and the tries want sorted keys.
- **No copy bound.** With exponentials the search is a semi-decision: it
  proves every provable goal given time (the queue is fair) and answers
  `Unprovable` only where its saturation ends, which it did on the six
  refutations and many more rows (the measurement).

**Options and front ends (D15, D16).** The engine reads
`Options::bias` (the focused engines' rule; on the class C and refutation
rows `rarer` and `factors` gave identical counts, so `Auto` is read as
they read it) and `Options::check`; the copy bounds, the memo limit,
`jobs` and the recursion limit have nothing to bound and are documented
as not read. A front end selects it as every engine: `Options::engine`
(`--engine inverse` in the command and `--engines inverse` in the
harness, `"engine": "inverse"` in the options' JSON, which the web page
holds). No option was added: the one choice a user might vary would be a
bound on the linear multiplicities (the thesis's future work), which no
measured row asked for.

**Where quantifiers go (D17).** A forward sequent's members would carry
terms: the place is the zone's members (`Zone::gamma`, the tries' keys),
with unification at an axiom's partner and at a hole's members, and
subsumption as matching instead of inclusion; a rule's term keeps its
shape (the steps name occurrences, the terms come with the premises).
`core-inverse.md` says so for the next step.

### Every filter, and why it removes no sequent a proof needs

The engine's `Unprovable` rests on each of these; nothing checks a
saturation (below, "Refutations"). The common argument: take a backward
focused proof of the goal (focusing is complete for every bias and in
every mode, `core-focus.md`; under a reading the one-sided proof whose
every sequent has one output, which the two-sided engine searches). By
induction on it, every stable sequent `T` of the proof, in its normal
form (a negative-literal subformula of a `?` in `Γ`, which only a copy
put there, moved back to `Θ`, where the copy left it), is subsumed by a
kept sequent. The step: the phase above `T` is an instance of one of the
rules (its focus, its `⊕` choices, its partners, its releases), and
evaluation is monotone in subsumption: with premises that subsume the
backward premises every step succeeds and derives a zone that subsumes
the backward one.

- `take` of a member the backward premise holds: an exact premise holds
  it; an absorbing one may lack it, and if it is an output the premise,
  a subset of the backward one less that output, has none (R2 holds); a
  weakening one (affine) may lack it, and it is no output under a
  reading, since a kept affine sequent there has exactly one output,
  which the backward premise's one output is.
- `join` (`&`): the backward premises' zones are equal; exact sides that
  subsume them are equal to them, an absorbing one lies within, and both
  absorbing join to a zone within; in affine mode the exact sides are
  weakened to the maximum, within the backward zone.
- `Bang`: the backward premise is exactly its subformula, so the zone
  left is empty. R1: a subsuming zone has the backward zone's one output,
  or fewer with `top`.

The filters:

1. **Forward subsumption** (`subsumed`): an exact `T` subsumes an exact
   `S` with the same `Γ` and `Θ_T ⊆ Θ_S`; an absorbing or weakening `T`
   any `S` with both zones within. Each relation is transitive and
   implies "every sequent `S` stands for, `T` does", so the induction's
   kept sequent may be the subsuming one. An exact sequent never
   subsumes a weak one.
2. **Killing at the pop** (`dead`): a sequent another alive kept one
   subsumes is not taken up and is left out of new combinations. Each
   kill is by a strictly stronger sequent (no two kept sequents are
   equal: the second would have been subsumed at its keep), so a chain of
   kills ends in an alive sequent, which is taken up.
3. **Coherence** (`Shape::coherent`): outside the subformulas of `?`
   nothing is repeated, so a sequent of a proof holds each occurrence
   once, none below another, and two members only below a `⅋` (a `⊗`,
   `&` or `⊕` above both sends them to different premises or keeps one
   side); `Θ`'s members join in, since a `?` stood beside the other
   members' ancestors where it was decomposed. Testing the neighbours in
   id order is testing all pairs (the common ancestors of a list sorted
   in preorder are those of its neighbours). A subset of a coherent
   sequent is coherent, so a subsuming sequent passes.
4. **The shape under a reading** (R1 at every step, partners of the
   opposite position): every sequent of an intuitionistic proof has one
   output, or none under a `⊤` that supplies it.
5. **Partners pre-filtered** by coherence with the formula the focus
   began on: both stand in the conclusion.
6. **No `D2` rule for a negative-literal subformula of a `?`**: the
   partner from `Θ` plays its part (the normal form above); from `Γ` too
   only where the goal holds the literal.
7. **Globals left out of the kept `Θ`**: the subformulas of the `?` the
   goal reaches through `⅋` alone are in `Θ` on every branch of every
   proof, their `?` steps below every `&` of the goal's phase (which
   decomposes the unary connectives first); a `?` under an `&` of the
   goal is no global.
8. **Mix left out in affine mode**, as the focused engine leaves it out:
   with weakening Mix proves nothing new.
9. **Pruned premise choices** (`fits`): two exact premises of one
   release with different contexts, or, under a reading, releases whose
   contexts bring two outputs; `derive` would fail on either (the `&`
   steps of one release join its contexts and nothing else changes
   them, and the conclusion holds every release's context with at least
   the most outputs of each).
10. **Fairness**: finitely many sequents of each size, the smallest taken
    up first, so every kept sequent is taken up; a combination is applied
    when its last premise is.

Without exponentials coherence forbids repeats, so the sequents are
finitely many and the saturation ends: the engine decides MALL, MLL and
their units, in every mode.

**Each filter's fault is caught.** In a scratch copy of the tree, one
deliberate fault per filter, each over-removing (the script and logs in
`/tmp/claude-1000/-home-tux-Projects-own-linlog/9d9a4ad6-682b-41be-a324-d4066260a2be/scratchpad/faults.py`
and `faults-out/`), and `cargo test -p linlog --lib -- inverse reference`
on it. Every one makes a committed test fail ("diff" is the engine's own
differential test, the others the reference comparisons and
`decides_small_sequents`):

| fault | caught by |
|---|---|
| exact sequents subsume as weak ones (a smaller `Γ`) | diff, classical, Horn, intuitionistic |
| every `Θ` left out of the keys, as if global | diff, classical, intuitionistic |
| a sequent killed at its pop by itself | diff, small, additive, classical, Horn, contractions, intuitionistic |
| coherence takes a `⊗` for the connective two members share | diff, small, classical, intuitionistic |
| coherence forbids repeats inside the subformulas of `?` | Horn, intuitionistic |
| under a reading an absorbing sequent with no output dropped | diff, additive, intuitionistic |
| no axiom takes its partner from `Θ` | diff, small, classical, contractions, intuitionistic |
| a partner beside the focused formula only from another root | diff, small, classical, intuitionistic |
| Mix left out in every mode | diff, classical |
| premise choices that bring one output pruned | diff, additive, Horn, intuitionistic |
| exact contexts compared across releases | diff, classical, Horn, contractions, intuitionistic |
| a sequent never fills two holes of one rule | small, after the case `⊢ ?a, ?~a, ⊥ ⊗ ⊥` was added (no test caught it before) |
| no focus on a positive formula under `!` | diff, small, classical, contractions, intuitionistic |
| a goal's hole filled only by an exact sequent | diff, additive, classical, Horn, intuitionistic |
| an absorbing premise of `&` must equal the exact one | diff, classical, intuitionistic |
| a premise never absorbs an output | diff, additive, intuitionistic |

### Refutations

An `Unprovable` from the saturation carries no certificate: the front
door gives it the counts' refutation where they rule the goal out, the
classical refuter's where its reading is false, and otherwise
`Refutation::Exhausted`, which `Disproof::check` answers as
`uncertified`. A check of its own (the saturated set closed under the
rules, checked without the engine's code) is 37b's if the engine wins
rows, which it does; until then every saturation refutation is
"exhausted without a check". The six refutations of the comparison are
all of this kind (classically valid, balanced), and so are the 58 rows
of the measurement it refutes and the default leaves undecided (54 of
them among the 384): the default runs the counts and the classical
refuter after its search too, and neither refuted them.

### Tests

- `inverse::tests::agrees_with_the_focused_engine_and_the_reference`:
  generated sequents and their mutants, every rule set of
  `generate::Rules::ALL` (linear and affine, with Mix where the rules
  use it) and of `generate::IllRules::ALL` (intuitionistic, linear and
  affine), 628 cases: the inverse method within 1 024 polls against the
  focused or two-sided engine within the generator's copies and against
  the reference prover, `Unprovable` judged as strictly as `Proved`
  (either against the other is a failure); every sequent without
  exponentials must be decided; 621 of 628 were.
- `decides_small_sequents`, `refuses_an_incoherent_goal` (the panel's
  findings 1 and 2), `choices_a_zero_ends_are_polled` (the limits
  review's B), and `index::tests` (the trie's queries and its refusal at
  the last `u32`).
- The reference comparison (`search/reference.rs`) runs the engine as one
  more configuration, within `INVERSE_POLLS` (2⁸) polls; the comparison
  took 33 s before and 43 s after, its own test about 45 s.

### The panel

Three agents, each in a fresh context on a frozen jj workspace at the
commit under review (`../linlog-step37a-panel` at `mkyktvpn`,
`../linlog-step37a-panel2` at `nlmpprlt`, `../linlog-step37a-panel3` at
`omksyqok`), briefed with the machine's rules and told to report only
what breaks soundness, completeness or a stated requirement; their
scratch directories hold every witness.

**The argument (Fable 5.1, high)**, the invariants derived from the code
before the rules file was read, then each named filter checked at every
site. Every item holds (the rule set's focusing completeness, the normal
form of the `Θ` partners, evaluation monotone in subsumption, both
subsumptions, the kill at the pop, coherence and its neighbour lemma,
the shape filter, the partner pre-filter, the globals, Mix in affine
mode, `fits`, the goal's holes, fairness and enumeration, termination
without exponentials), with two findings:

1. *A goal off the roots whose members are not coherent is refuted*
   (`[a, ~a]` of `⊢ a ⊗ ~a` and of `⊢ a & ~a`, `[0, 0, 3, 4, 5, 6]` of
   `⊢ a ⊗ b, ~a, ~b, ~a, ~b` with Mix; the focused engine proves them and
   the checker accepts its proofs): coherence presumes the goal's own
   members coherent, as the roots are, and `Goal::new` takes any
   multiset. Answered at `nlmpprlt`: `admits` refuses such a goal
   (`NotTaken::Shape`); pinned by `refuses_an_incoherent_goal`.
2. *The first answer refused the interactive prover's goals after `?c`
   and `?d`* (`[?~a, ?~a, a ⊗ a]`, `[?~a, ~a, a ⊗ a]`), which the engine
   had decided: a pair joined by a `?` formula never reaches a stable
   sequent. Answered at `omksyqok`: such a pair is coherent; the reviewer
   checked that the clause never fires in the kept-sequent filter (a
   `?` formula is never a linear member), that the neighbour lemma still
   holds, and that every goal `admits` accepts meets the argument's
   precondition, and reran the witness: proved and grafted. Pinned in
   the same test.

It also checked at `nlmpprlt` that the limits fixes leave the rule set
and the evaluation exactly as they were (44 sequents in every mode with
equal verdicts, proofs and counters before and after).

**The counterexamples (Opus 5.5, high)**: a second reference written for
the review (Rust, no shared code, its own parser, unfocused dyadic,
contractions bounded and deepened, affine weakening, Mix, the one-output
restriction from the two-sided text), checked by hand on 37 sequents;
then about 18 500 sequent and mode cases, each under the three biases,
against the focused engines, the committed reference and its own:
generated sequents and mutants of every rule set in every mode up to
budget 24, random and balanced sequents, 79 aimed at the weak spots
(globals, `?` under `&`, nested `?!`, `⊤` under a reading, affine `!`,
Mix with exponentials), a debug build with the `debug_assert!`s on.
**No finding**: no `Unprovable` against a proof, no proof against a
refutation, no rejected proof, no bias against another; of 1 836
exponential sequents the engine refuted, its reference with six more
copies and two million visits proves none. At `nlmpprlt`, 47 260
verdicts equal to the ones before the fix. The poll gap measured: at
most 34.5 ms. Reported, not wrong: some Mix sequents and large
intuitionistic ones without exponentials stay undecided at 20 s (the
work goes into choices of premises), which the measurement shows too.

**The integers and limits (Sonnet 5.5, high)**: every integer listed;
none wraps unrefused (ids, ranks, zone lengths, the rule tables and the
trie refused at `u32`, the output counts `u64`); the ordered modes
refused; the memory bound held in the saturation (16 MiB and 64 MiB end
in `MemoryLimit` at peaks of 23 and 68 MiB). Three findings, all in the
rules' compilation:

- A: *the scan of an atom's literals for an axiom's partners was
  neither work nor polled*: a time limit of 2 s on `SYJ206+1.017` ended
  after 3.2 s, of 4 s on `SYJ206+1.019` after 10.9 s (at the memory
  bound, no unit counted).
- B: *choices of `⊕` sides that a `0` ends were never polled*:
  `⊢ 0 ⊗ ((a₀ ⊕ b₀) ⊗ …)` with 24 factors took 18 s under a 1 s limit.
- C: *`Writer::record` copied a rule's holes uncharged and unpolled*: the
  goal's phase of `SYJ202+1.007` (52 million members) peaked at 1 612 MiB
  under a bound of 1 200 MiB.

Answered at `nlmpprlt` (each choice looked at, each literal scanned and
each member copied is work, polled; the copy charged as the holes leave
the writer) and re-checked by the reviewer there: A and B closed (the
first poll on `SYJ212+1.020` after 3.9 s instead of 23.9 s, the rest
being the reading of the sequent; `zero24` to `zero40` end at 1.005 s),
C's memory closed (1 116 MiB under 1 200 MiB). Pinned by
`choices_a_zero_ends_are_polled`. **A residue, not fixed**: on a goal
phase of millions of holes the stop is seen on time (999.8 ms under
1 s), but freeing the millions of small lists afterwards and the front
door's next allocation take some 40 % more (1.38 s for 1 s on
`SYJ202+1.007` under a 4 GiB bound): teardown proportional to the
memory built, bounded by the memory bound; holding the holes in flat
arrays would remove it (left open, below).

### The measurement

`linlog-bench run` at 60 s, one thread per run (`--jobs 1`), no copy
bound (`--copies none`, the command's default; the families with each
problem's own), the library's memory bound (1 GiB), two shards per
engine and part; every part under `/tmp/linlog-machine.lock`. The
default (`--engines auto`) on the held-out sets on cores 2 and 3
(12:03 to 12:55), its other parts on cores 4-5 and 6-7 after the pause
(13:27 to 15:14); the inverse method (`--engines inverse`, the binary at
`nlmpprlt`) on cores 2 and 3 (13:27 to 15:43). The last engine commit,
`omksyqok`, changes only which goals off the roots `admits` takes; on the
roots its clause never fires (the panel's argument), so the measurement
stands for it. The scripts, the CSV
files and the analysis are in `/tmp/linlog-handoff/37a-measure/`
(`measure.sh`, `group.sh`, `analyse.py`, `out/`, `analysis-*.txt`). The
rows: the held-out sets of step 39 (850: `KLE-01`, `-cbn`, `-cbv`,
`KLE-IMP-CONJ`, `ILLTP-SYN` and `-LCL` in the three translations,
`ILLTP-SYJ-cbv`); the 384 intuitionistic rows no tool decided at 10 s in
the comparison of 2026-10-15 (computed from its `runs.csv`: no tool's
verdict but a refutation of a proved problem; 135 of them in the
held-out sets, the other 249 in `ILLTP-SYJ-01`, `-cbn` and one in
`Non-theorems`); Sympli's ten class C proofs and the six refutations
(14 of them outside the other sets); the 123 rows of the families.

| set | rows | default decides | inverse decides | inverse only | default only | contradictions |
|---|--:|--:|--:|--:|--:|--:|
| held-out sets | 850 | 729 | 751 | 38 | 16 | 0 |
| the other rows (384's rest, class C, the six) | 263 | 12 | 80 | 72 | 4 | 0 |
| of these, the 384 rows no tool decided at 10 s | 384 | 26 | 113 | 96 | 9 | 0 |
| families | 123 | 111 | 45 | 0 | 66 | 0 |

(The harness's row for a problem listed by file has no family, so the
translations of one problem share its name: the analysis keys them by
name and size and names the translation from the comparison's rows. The
harness's `--resume` keys them by name too, so after the pause the
default's second part skipped `SYJ201+1.007` to `.010` of `-cbn`, whose
`-01` rows it had; they were run alone afterwards, on cores 2 and 3
under the lock, into `out/auto-extra-missing.csv`. A follow-up for the
harness: key a listed file's row by its path.)

- **Class C** (`ILLTP-SYJ-cbn/SYJ203+1.011`, `SYJ205+1.012` to `.020`):
  the inverse method proves all ten in 0.14 to 0.29 ms; the default at
  60 s proves `SYJ205+1.012` in 14.5 s and no other.
- **The six refutations** (`SYJ208+1.004` in `-01` and `-cbn`,
  `SYJ209+1.011` and `.012` in `-cbn`, `.008` and `.009` in `-cbv`): the
  inverse method refutes all six, in 639 ms, 547 ms, 1.9, 3.8, 3.7 and
  5.8 ms; the default refutes `-cbn/SYJ209+1.011` in 50.7 s and
  `-cbv/SYJ209+1.008` in 21.5 s and leaves the others at the limit.
- **Rows the engine decides that no linlog engine decided** (neither the
  default here at 60 s nor linlog's two columns of the comparison at 10
  s): 38 of the held-out sets (`SYJ203+1.011` to `.020`, `SYJ204+1.020`,
  `SYJ205+1.017` to `.020` proved; `SYJ209+1.009` to `.020`,
  `SYJ210+1.010` to `.020` refuted, all in `ILLTP-SYJ-cbv`, in 3.8 to
  341 ms) and 72 of the other rows (`SYJ203`, `SYJ204`, `SYJ205` proved,
  `SYJ205+1.010` to `.012` of `-01` in 2.7 to 10.1 s, the others in
  under 11 ms; `SYJ208+1.004`, `.005`, `SYJ209`, `SYJ210` refuted, in
  `-01` and `-cbn`; `SYJ201+1.008` of `-cbn` proved in 33.3 s); 96 of
  the 384 in all.
- **Rows lost** (the default decides, the inverse method not): 16 of the
  held-out sets (`ILLTP-SYJ-cbv`: `SYJ201+1.002` to `.008` and
  `SYJ206+1.004` to `.008` proved by the default in 0.9 ms to 42 s,
  `SYJ207+1.003`, `SYJ208+1.007`, `SYJ212+1.004` and `.005` refuted),
  4 of the other rows (`SYJ207+1.003`, `.004` and `SYJ212+1.018` in
  `-cbn`, `SYJ212+1.004` in `-01`), and 66 of the families (Partition,
  3-Partition, QBF, the counters, `wide-m2` to `-m4`). The pigeonhole
  formulas (`SYJ201`, `SYJ206`) keep millions of sequents that differ
  only in their unrestricted zones; the families' sequents without
  exponentials lose in the choices of premises the counts would rule
  out.
- **Time on the rows both decide**: held-out sets 713 rows, the inverse
  method faster on 386, median 0.056 ms against 0.063 ms, geometric
  mean of the ratio 0.93, 65.4 s against 187.4 s in all (both on cores
  2 and 3); the other rows 8 (the inverse method 40.4 s, the default
  123.7 s, `SYJ201+1.007` of `-cbn` among them: 13.7 s against 9.2 s); the families 45 (geometric mean 2.40, the inverse method
  slower; 117 s against 132 s in all, the `mix` rows 0.05 ms against up
  to 60 s); the default's families ran on the slower cores 4 to 7.
- **Against the library's headers**: both engines disagree with the same
  44 rows (the wrong headers `bench/TOOLS.md` names, the proofs checked),
  the inverse method with none more.

### The target sets

`bench/targets.sh 37a` (cores 2 and 3, the focused engine's 280 rows) and
`bench/net-targets.sh 37a` (core 10, a copy of the script with that core,
the net engine's rows), the binaries at `mkyktvpn`; `linlog-bench summary
--against … --counters`:

- against `bench/targets/after-cyclic.csv`: no verdict differs, none
  decided by one only; of the 145 rows decided alike on one thread, one
  differs in its counters, `mix/11`, which ends at its 300 s limit in
  both (50 532 616 stable sequents against 48 748 043): a row that does
  not end within its limit, whose counters differ between runs of one
  binary (36b's report).
- against `bench/targets/net-after-cyclic.csv`: no verdict differs; of
  the 64 rows decided alike, none differs in its counters.

So every row that ends within its limit is unchanged, as a session that
does not touch the dispatch must leave it.

### Departures from the design

- (departs from the design) The index is a pair of tries over the sorted
  zones (a set-trie), not feature-vector indexing (Schulz 2013) that
  `plan/later.md` and the spec name: subsumption here is multiset and set
  inclusion of two short sorted lists, which a trie answers by walking
  only the paths the query holds; a profile of `SYJ201+1.003` found 92 %
  of the time in the first version's list scans, and the trie took the
  saturation from 39 thousand to 2.5 million sequents kept in 5 s.
- (departs from the design) No partially applied rules and percolation
  (Chaudhuri's lazy OTTER loop, the research note's item 4): a sequent
  taken up is combined with the lists of candidates for the other holes,
  pruned as they are chosen (`fits`); each combination is applied once.
- (departs from the design) No irredundancy conditions for weak premises
  (Chaudhuri 2006, §3.4): a weak premise may fill any hole, and the
  redundant conclusions are subsumed.
- (departs from the design) `Context` and `Classes` stay in `focus/`: the
  engine keeps sorted lists, not the focused engine's bitsets, and does
  not key sequents up to interchangeable occurrences.
- (departs from the design) A rule application's transient lists of
  candidates and their zones are not charged to the memory account; the
  rules, the kept sequents, their store, records, queue, tries, postings
  and the proof's nodes are.

### The checklist

| item | state | evidence |
|---|---|---|
| The engine for MALL and its semi-decision with exponentials behind the engine interface, saturation from initial sequents in the subformula closure, subsumption and an index, polling by work, its tables charged, the ordered modes refused, every proof checked | done | `mkyktvpn`, `nlmpprlt`, `omksyqok`; `core/src/search/inverse/`; README's example; the limits review |
| The differential test against the focused engine and the reference prover | done | `agrees_with_the_focused_engine_and_the_reference` (628 cases, 0 contradictions, 621 decided), the reference comparison, 16 faults caught |
| Refutations | exhausted without a check, said so | "Refutations" above |
| The panel on frozen trees, every finding answered | done | "The panel": the argument (Fable 5.1, high) on `mkyktvpn`, `nlmpprlt` and `omksyqok`, the counterexamples and the limits on `mkyktvpn` and `nlmpprlt`; findings 1, 2, A, B, C fixed and re-checked; the teardown residue left open |
| The measurement | done (2026-10-19) | "The measurement" |
| The target sets | done, unchanged within their limits | "The target sets"; `bench/targets/37a.csv`, `net-37a.csv` |
| Every code commit gated | done | `gate: passed` before each of `mkyktvpn`, `nlmpprlt` and `omksyqok` (`wqvkmztv` is the ceiling alone) |
| `nix flake check` at the last commit | see the message to `planning` | run under the machine lock after this report's commit |
| `.claude/rules/` (`core-inverse.md`, `core.md`, `core-search.md`, CLAUDE.md's table), README, the changelog | done | `mkyktvpn` and later |

### What is left open

- **37b is needed**: the engine wins rows the default does not decide,
  so the dispatch rows where it is fastest (D19), measured as 35a
  measured its routing: the held-out wins are all in `ILLTP-SYJ-cbv`'s
  and the 384's `SYJ203`, `SYJ204`, `SYJ205`, `SYJ208`, `SYJ209` and
  `SYJ210` (intuitionistic, a `!` hypothesis whose conjuncts each use
  the goal), and it loses on the families and the pigeonhole formulas,
  so a row needs a feature that tells them apart, or a portfolio; and
  the saturation refuter's certificate, since it refutes what the
  classical refuter and the counts do not (58 rows here).
- **Undecided without exponentials**: some Mix sequents and large
  intuitionistic ones stay undecided for seconds where the focused
  engines decide in milliseconds (the counterexamples' report, the
  families), the work in the choices of premises: the count invariants
  (`Counts`) applied to a choice before it is evaluated, and Chaudhuri's
  irredundancy conditions, are the candidates.
- **The teardown residue** of the limits review: after a stop seen on
  time, freeing a goal phase of millions of holes costs some 40 % more
  (bounded by the memory bound); flat arrays for the holes would remove
  it.
- **The harness's `--resume` and a file listed alone**: such a row has
  no family, so two translations of one problem share a key and a
  resumed run skips the second (four rows here, run again by hand).
- **The workspaces**: `../linlog-step37a` (this session's commits),
  `../linlog-step37a-base` (at `trppoyys`, for the ratchet's
  comparison), `../linlog-step37a-panel`, `-panel2` and `-panel3` (the
  panel's frozen trees at `mkyktvpn`, `nlmpprlt` and `omksyqok`); none
  holds uncommitted work.
- **A test of the refusals at `u32`**: `keep`'s and `record`'s refusals
  are argued, not exercised (the trie's is pinned); a test would need
  limits injected below `u32::MAX`.

## Session 37b: the dispatch and the certificate

**Outcome: the inverse method takes turns beside the default engine on
two rows of the dispatch, and its refutations carry a checked
certificate.** On an intuitionistic goal with an exponential and on a
goal under Mix without weakening, the engine the dispatch picks searches
alone for 2²⁴ of its units, then the inverse method takes a turn, from
its start, each time the engine's own work has doubled, with a
thirty-second of that work as its budget: the turns together do at most
a sixteenth of the engine's work. Measured as 37a measured, at 60 s on
one thread, before (`tqxslulk`) and after (`ykntmzxo`) on the same cores:
the default decides **102 rows more and loses none** (39 of the held-out
sets, 63 of the other rows, none of the families), all ten of Sympli's
class C proofs (0.17 to 0.29 s) and all six refutations (0.15 s to
22.8 s), no verdict against another and none against a checked proof of
any tool in the comparison's runs; the median of after over before is
1.00 at every speed of the default, the worst 1.24 from 10 ms to 1 s,
1.11 from 1 to 10 s and 1.42 over 10 s. The saturation's refutation is
`Refutation::Saturated`, the set the engine saturated with, checked by
`search/saturated.rs`, which shares no code with the engine, refused for
each condition it states when a set breaks it (committed tests); the 58
refutations nothing else gives are all answered `Unprovable` with it,
each checked (the inverse method forced), 56 of them by the default's
turns within 60 s. Two panel rounds on frozen trees (the argument on
Fable 5.1 at `high`) found no break of soundness; their findings of
completeness and of stated requirements were fixed, each answered below.

### The dispatch

**What separates the wins from the losses: no static feature.** 37a's
wins are intuitionistic sequents with `!`-hypotheses (`SYJ203`, `204`,
`205`, `208` to `210`), its losses the pigeonhole formulas and others
(`SYJ201`, `206`, `207`, `212`, `SYJ208+1.007` in cbv) and every family
but `mix`. On the held-out sets (850 rows, 37a's runs), among the
problems with an exponential, the 38 the inverse method alone decided
against the 16 the default alone decided:

| feature | 38 won | 16 lost |
|---|---|---|
| occurrences | 101 to 483 | 159 to 5 601 |
| multiplicity | 2 to 21 | 7 to 256 |
| `!` | 41 to 220 | 65 to 2 546 |
| `⊸` | 12 to 88 | 15 to 1 018 |
| `⊗` | 0 to 43 | 29 to 509 |
| `⊕` | 0 to 20 | 0 to 243 |
| axioms | 0 to 21 | 0 to 17 |
| fragment | ILL, IMELL | ILL, IMELL |

Every range overlaps; the smallest instances of the losing problems
(`SYJ201+1.002`, `SYJ206+1.004`, `SYJ212+1.004`) sit among the wins, and
`SYJ208` wins at sizes 4 and 5 and loses at 7. Of the 640 rows both
decide, the default is faster on 290. So a row of its own, by a feature,
would lose rows the default decides (as 35a found for the net engine).

**The turns** (`Row::probe`, `answer_goal`, `Turns::take`). Two rows of
`DISPATCH` before the general ones: `LL`, intuitionistic, the feature
`Exponentials` (an exponential in the goal), the two-sided engine; `LL`,
`Modes::Mix` (classical with Mix, not affine), the focused engine; both
with turns. Elsewhere nothing changes: the measurement found no win of
the inverse method on intuitionistic goals without exponentials or on
classical ones without Mix. A turn comes only to a search on one thread
with no engine forced (the race's single side takes them, not its pool),
at a poll of the engine's own work on the calling thread (the default
bias's second search on its own thread polls the caller with no units
while it runs, so no turn runs beside it), and its decision answers: a
proof checked as any, a refutation by the counts or the classical reading
where they give one, else the saturated set checked. Where no turn
decides, the engine goes on where it stopped, its verdict and counters
as alone (`the_inverse_method_takes_its_turn`, and the target sets).

**How the design came to be.** The first design (`mtwmloox`) gave one
turn of 2²⁶ of the inverse method's units after a head start of 2²⁶ of
the engine's: rows decided within the head start pay nothing, the rest
up to the turn. The supervisor (`planning`) asked for the cost at every
speed, not only on fast rows, the after over before ratio bucketed by
the default's time, median and worst within a small factor, for instance
by a share of the work the default has done or by turns that grow as the
default bias's do; a row decided in half a second could have taken
several times longer. So the turns grow (`ssmwxkoq`): each time the
engine's own work doubles, a turn of a share of it, from the inverse
method's start (it cannot be suspended mid-search: the engine's recursion
and the saturation's loop are both on the calling thread). With a
sixteenth each (an eighth in all) the measurement lost `SYJ201+1.008` in
cbv, which the two-sided engine proves in 54.2 s here: the turns took
11 % of its time and it ended at the limit. With a thirty-second each
(`unywztnt`, a sixteenth in all) it is proved in 59.4 s, kept, by a
margin of 0.6 s. Any share loses a row decided close enough to the time
limit; the report's measurement is the claim. The head start, 2²⁴ units,
is some hundredths of a second, and the 737 rows the default decides in
under 10 ms keep a median of 1.00.

**The poll.** With the turn's comparison in every poll, a focused search
did 0.25 % more instructions (`search-qbf-20-2`, about seven a poll),
which took the ratchet's `batch-families` past its ceiling (base 1.86 %
over it already, then 2.06 %). `answer_goal` now asks `probes` once and
gives the engine the plain `poll` where the row gives no turn, the poll
with the comparison where it does; `Turns::take` is `#[cold]`.
`batch-families` stands at 1.90 %, the focused journeys at the base.

### The measurement

`linlog-bench run` at 60 s, one thread per run, no copy bound (the
families with their own), the library's memory bound, four shards each
pinned to two cores (8-9, 10-11, 12-13, 14-15), every part under
`/tmp/linlog-machine.lock`, the same rows as 37a's (the held-out sets,
850; the other rows, 263: the 384's rest, class C, the six; the
families, 123). Before: the default at `tqxslulk`, 16:15 to 17:42. After:
the default at `ykntmzxo`, 18:54 to 19:52, then the inverse method forced
on the 58 refutations. Scripts, CSV files and the analysis in
`/tmp/linlog-handoff/37b-measure/` (`measure.sh`, `run-all.sh`,
`after.sh`, `analyse.py`, `analysis.txt`, `out/`; `out/share16/` holds the
partial run with a sixteenth that lost the row above).

| set | rows | decided before | after | won | lost | contradictions | checks failed |
|---|--:|--:|--:|--:|--:|--:|--:|
| held-out sets | 850 | 727 | 766 | 39 | 0 | 0 | 0 |
| the other rows | 263 | 12 | 75 | 63 | 0 | 0 | 0 |
| families | 123 | 111 | 111 | 0 | 0 | 0 | 0 |

After over before on every row both decide, by the default's time
before:

| default's time | rows | median | 90th percentile | worst |
|---|--:|--:|--:|---|
| under 10 ms | 737 | 1.000 | 1.143 | 3.35, `KLE-IMP-CONJ/KLE_13_CBN` 0.1 to 0.4 ms (within the head start: no turn; timer noise at 0.1 ms) |
| 10 ms to 1 s | 76 | 1.003 | 1.071 | 1.24, `ILLTP-SYJ-cbv/SYJ208+1.007` 995 to 1 235 ms |
| 1 to 10 s | 18 | 0.988 | 1.086 | 1.11, `ILLTP-SYJ-cbv/SYJ206+1.007` 2.34 to 2.60 s |
| over 10 s | 19 | 0.013 | 1.097 | 1.42, `ILLTP-SYJ-cbn/SYJ212+1.018` 36.0 to 51.0 s |

The worst over 10 s is a row where the inverse method's units are slow
against the two-sided engine's (its saturation there ends at the memory
bound only late): a sixteenth of the units took 42 % of the time. The
rows within the head start keep their counters on every row (705 + 2 +
102 rows, the same engine, `nodes`, `splits`, `memo_hits`, `memo_entries`
equal), and so do the rows past it that the engine still decides.

- **Class C** (`ILLTP-SYJ-cbn/SYJ203+1.011`, `SYJ205+1.012` to `.020`):
  all ten proved by the turns in 0.17 to 0.29 s, each checked (before:
  `SYJ205+1.012` in 16.8 s, the others at the limit).
- **The six refutations**: `SYJ208+1.004` in `-01` and `-cbn` in 22.5 and
  22.8 s, `SYJ209+1.011` and `.012` in cbn in 0.16 and 0.24 s,
  `SYJ209+1.008` and `.009` in cbv in 0.15 and 0.34 s, each with its
  saturated set checked (before: `cbn/SYJ209+1.011` in 52.2 s, `cbv/.008`
  in 29.1 s, the others at the limit).
- **The rows won**: `SYJ203`, `SYJ204`, `SYJ205` proved, `SYJ208+1.004`,
  `SYJ209`, `SYJ210` refuted, in the three translations (`analysis.txt`
  lists each), and `KLE075` of `KLE-01` faster; `mix/8` to `/11` of the
  families answered by the turns in 0.43 to 0.47 s (before 1.2 s, 9.6 s,
  and for `mix/10` and `/11` the classical refuter after the limit).
- **The 384 rows no tool decided at 10 s**: 37a's list, kept in its
  session's scratch directory (`neither.txt`, copied to
  `/tmp/linlog-handoff/37b-measure/the-384.txt`), 135 of them among the
  held-out sets and 249 among the other rows, all measured before and
  after: the default decides 24 before and 112 after, 88 won, none lost,
  no contradiction (after: 92 by the turns, 56 refutations and 36 proofs,
  and 20 by the two-sided engine; `out/the-384.txt`).
- **The binary measured is the last commit's**: the commits after
  `ykntmzxo` change comments, documentation and `plan/` only, and the
  release `linlog-bench` built at the last code commit is byte for byte
  the one the after run used (`cmp`).
- **Against the comparison's runs** (`bench/comparison/2026-10-15/runs.csv`,
  every tool): 3 916 problems have a checked proof by some tool; the
  after run answers `unprovable` on none of them.

### The certificate

**What it is.** `Refutation::Saturated(Saturated { positive, sequents })`
with `Stable { linear, unrestricted, absorbs }`: the polarity the rules
were compiled under and every kept sequent not killed when the saturation
ended (`Saturation::saturated`), closed under the rules (every
application to sequents taken up was made, what it derived was kept or
subsumed, a kill leaves a stronger alive one). JSON `{"kind":
"saturated", …}`, wire level 3 (`wire::level_of`; a level-3 fixture under
`core/tests/wire/`, `the_third_level_reads`). `conclude` prefers the
counts' and the classical refutations, which are smaller and rest on
simpler invariants, and checks the set before it answers, as a proof,
under the stop (`Phase::Check`) and within `Limits::work`; a failed check
is `Error::Refutation`, a defect. The Rocq export refuses it
(`Unsupported::Saturated`): linlog's library has no theorem for it.

**The check** (`search/saturated.rs`, no code shared with `inverse/`):
the rules read off the forest again (`D1` foci, `D2` copies but negative
literals, every `⊕` choice not ending in a `0`, partners within the goal's
subtrees in the normal form of the `?`-literals), written as postorder
steps without recursion over the formula, evaluated with its own `take`,
`join` and shape test, applied to every choice of premises among the set
and of partners, Mix on every pair where the mode has it; each
conclusion, the globals left out, must be incoherent or stood for by a
sequent of the set; then a stable sequent of the goal's phase must be one
none stands for. Its argument is the engine's (focusing complete under
any polarity; the normal form; each step monotone; coherence of a proof's
sequents given a coherent goal; the globals), in the module's
documentation and in `core-inverse.md`, "The certificate".

**Each condition refused** (`saturated::tests::a_wrong_set_is_refused`,
one wrong set per condition): a polarity of another length, a member
past the forest, an unsorted zone (`Mismatch`); a sequent with two
outputs under the reading (`Unshaped`); the empty set, whose axioms are
not stood for, and the set with one needed sequent dropped (`NotClosed`);
a set closed under the rules but not under Mix (`NotClosed` under Mix);
one absorbing sequent that stands for every hole (`Filled`); the cyclic
mode (`NotApplicable`). `the_engines_sets_pass` (every commutative
mode), `a_goal_off_the_roots_is_refuted` (the panel's witness), and the
differential test and the reference comparison, through whose 69 sets
the check runs on every inverse refutation.

**The 58.** The inverse method forced on the 58 rows that it alone
refutes (37a's list: `SYJ208+1.004`, `.005`, `SYJ209`, `SYJ210` in the
three translations): all 58 `unprovable`, each set checked `ok` by the
harness's own `Disproof::check`; through the command (`linlog prove -i
--engine inverse --format json`, which checks the set before it
answers), all 58 `unprovable` with a refutation of kind `saturated`, at
wire level 3. With the default, 56 within 60 s (the two
`SYJ208+1.005`, whose saturation needs some 700 million units, would need
a minute and more of the engine first). On every refuted row of 37a's
measurement (213 rows, the inverse method forced, before the last fixes)
every set passed.

**Its cost.** The check applies every rule to every choice of the set's
sequents, as long as the saturation took at most: on `SYJ210+1.020` in
cbv 209 ms of search and 121 ms of check, on `SYJ208+1.005` in cbn 9.1 s
and 1.6 s. Its memory (the set's index, candidate lists, contexts) is not
charged to an account: of the set's order, which the search held within
its bound. `Disproof::check` of a set read from elsewhere has no stop and
can take time exponential in the goal's `&` under `⅋` (the integers
review's 2 491-byte document past 100 s): documented, not bounded.

### The panel

Two rounds, each of agents in fresh contexts on frozen jj workspaces,
briefed with conduct.md's rules on cores and memory (cores 2 to 7; the
timed runs held 8 to 15) and told to report only what breaks soundness,
completeness where claimed, or a stated requirement. The briefs are in the
session's scratch directory (`panel-common.md`, `panel-argument.md`,
`panel-counterexamples.md`, `panel-integers.md`, `panel2-common.md`), the
witnesses in each reviewer's directory beside them.

**Round one** on `../linlog-step37b-panel` at `mtwmloox` (the single
turn):

- *The argument* (Fable 5.1, `high`): the invariants derived from the code
  first, then checked. No set accepted for a provable goal (32 provable
  goals, 80 000 random sets each, every mode). Findings: **F1** the check
  refused the engine's valid set for a goal off the roots whose atom has
  a literal outside the goal's subtrees (`⊢ (?a ⅋ ?~a ⅋ (b ⊗ ~b)) ⊗ (a ⊗
  ~a)`, goal the left factor: `Error::Refutation(NotClosed)`); answered
  in `zplymslu`: partners within the goal's subtrees (sound: a proof of a
  goal holds nothing outside them), pinned by
  `a_goal_off_the_roots_is_refuted`, which fails without the fix. **F2**
  `conclude`'s documentation claimed the check's memory charged; answered
  in `zplymslu` (documented as uncharged).
- *The counterexamples* (Opus 5.5, `high`): its own reference (Python,
  unfocused, every mode, checked by hand on 33 sequents) beside the
  committed one; 1 444 engine sets checked, all accepted, none against a
  reference; 276 145 mutated sets, 70 050 accepted, every accepted goal
  judged unprovable by both references; on 1 942 small provable goals
  every set of up to two members of size two under every polarity, 71.3
  million sets, none accepted; the turn on 600 aimed sequents, 189 LLTP
  problems and 626 generated ones, no contradiction. Finding: a turn of
  2²⁶ units took 24 s on `!(b ⊸ ⊤), !(((c ⊕ b) & !b) ⊕ (a ⊗ (a ⊕ c))),
  !(((c ⊕ a) & !a) ⊕ (b ⊗ (b ⊕ c))) ⊢ b ⊗ a`: `candidates` counted the
  sequents it returned, not the posting list it scanned. Answered in
  `ykntmzxo`: the scan counted, 2²⁶ units now 0.19 s there.
- *The integers and limits* (Sonnet 5.5, `high`): nothing wraps; 11 000
  mutated sets in a debug build, none accepted for a provable goal; 247
  engine sets of the library accepted. Findings: **(c)** `derive` counted
  its premises, not its steps (stops 130 ms late on `wide` with shared
  literals); **(d)** `fill` counted nothing (1.66 s late on `(a & b) ⅋ …`
  of 18 pairs); **(e)** the set's check ignored `Limits::work`; **(f)** a
  certificate read from a file can make `Disproof::check` run
  exponentially long. (c) to (e) answered in `zplymslu`, (f) documented.

**Round two** on `../linlog-step37b-panel2` at `zplymslu` (the growing
turns and the answers):

- *The argument* (Fable 5.1, `high`): derived the bound (turns `< 2W/s +
  n·g` for the engine's own work `W`, share `s`, `n` turns, `g` one poll's
  overshoot), confirmed verdicts and counters as alone where no turn
  decides (pigeonhole images, identical `nodes`, `memo_hits`,
  `memo_entries`), the stop, the work bound and the memory bound inside a
  turn, no turn on two threads, and 23 400 goals with 12 165 saturated
  sets, 6 290 off the roots, all accepted, no verdict against the focused
  engines. No finding; two notes: in a race the single side's `done`
  counts the pool's work, so its turns can take up to `2(p + 1)/s` of that
  thread's time while the pool searches on (recorded in `core-search.md`);
  a work limit counts the turns' units.
- *The integers and limits* (Sonnet 5.5, `high`): (c), (d), (e) closed
  (1.2 ms, 3.9 ms late; `WorkLimit` within one poll); the next worst poll
  gaps under 15 ms. Two findings older than or beside the change, left
  open: **A** the teardown after a stop on the rule compiler's millions of
  small lists (37a's residue; up to 0.9 s on `SYJ208+1.010` in cbv at a
  2.4 s limit); **B** the resident memory after turns passes the bound
  (660 to 740 MB under 400 MiB, against 517 MB without turns: the turns'
  pages stay with the allocator; the account is right).

A third round was not run: the changes after round two are the share (a
constant, whose bound round two derived for any share) and the scan's
units (`ykntmzxo`, checked against the counterexample's witness).

### Commits

| commit | what |
|---|---|
| `ozwvpkoy` | the inverse method's work counted by the members it handles and the trie's lookups, so that a budget of its units bounds its time (1 to 146 000 units a millisecond before, 5 000 to 143 000 after) |
| `ysqzqtvv` | `Refutation::Saturated` and its check apart from the engine (`search/saturated.rs`), wire level 3, the Rocq export's refusal |
| `mtwmloox` | two rows of `DISPATCH` with a turn of the inverse method (the first design: one turn) |
| `ssmwxkoq` | the turns grow with the engine's work; the poll split by whether the row gives turns (gated in `../linlog-step37b-gate`) |
| `zplymslu` | the first panel's findings answered |
| `unywztnt` | the turns halved, a thirty-second each |
| `ykntmzxo` | the scan of the posting lists counted (the counterexample review) |
| `tozoztmp` | the turns' share of a race and the resident memory recorded; a test's comment reworded |
| `yzyklvot` | this section, `bench/targets/37b.csv`, `net-37b.csv`, `bench/TARGETS.md` |
| the last commit | the turns' measurement in `Engine`'s table, beside the inverse method's alone |

### Options, front ends and quantifiers (D15, D16, D17)

No option was added: the head start and the share are dispatch
thresholds, private constants beside their measurement (the author's
decision for the release: "a front end must not move a row the library's
measurement placed"); `Options::engine` forces an engine and so removes
the turns (`--engine two-sided`, `"engine": "two-sided"` in the options'
JSON, which the web page holds). `Options::check` off skips the set's
check as it skips a proof's. Quantifiers: the certificate's sequents
would carry terms where the engine's do (`Stable`'s zones), and the check
would match instead of compare; `core-inverse.md` says where.

### The target sets

`bench/targets.sh 37b` and `bench/net-targets.sh 37b` (copies pinned to
cores 8 and 9 and to core 10, under the machine lock, 19:57:13 to
19:57:30: the focused set's rows take 20 s in all, against the oracle's
404 s, which `mix/10` and `/11` spent at their limits; the four rows
more in `37b.csv` are those two run three times, as every row under
10 s is), the binaries at `ykntmzxo`; `linlog-bench summary --against …
--counters`:

- against `bench/targets/after-cyclic.csv`: no verdict differs, none
  decided by one only; of the 145 rows decided alike on one thread, four
  differ in their counters, `mix/8` to `mix/11`, the rows the dispatch
  moves: the inverse method's first turn answers them (unprovable, as
  before) in under half a second, with its counters (all zero: no rule
  survives, every `⊕` there has a `0` side) in place of the focused
  engine's. Every other row, the intuitionistic ones included, has the
  oracle's counters: decided within the head start, or by the engine
  after turns that did not decide.
- against `bench/targets/net-after-cyclic.csv`: no verdict differs; of
  the 64 rows decided alike, none differs in its counters.

Both files are committed (`bench/targets/37b.csv`, `net-37b.csv`), with
an entry in `bench/TARGETS.md`; the oracles stay.

### Departures from the design

- (departs from the design) The dispatch row is a **turn beside the
  default engine, not a row of the inverse method**: no static feature
  separates its wins from its losses. The turns grow with the engine's
  work, each from the inverse method's start (it cannot be suspended), so
  a turn redoes the previous one's work: at most twice the last one's.
- (departs from the design) **Rows decided near the time limit can be
  lost to the turns**: the turns take up to a sixteenth of the engine's
  units, which in time was up to 42 % on one row (`SYJ212+1.018` in cbn,
  where the inverse method's units are slow), and `SYJ201+1.008` in cbv,
  proved in 54.2 s, is proved in 59.4 s. None was lost in this
  measurement; one was with a share of a sixteenth.
- (departs from the design) **The check's memory is not charged** to an
  account, and **`Disproof::check` has no stop**: a set from elsewhere
  can make it run exponentially long (documented).
- (departs from the design) **Two of the 58 are not answered by the
  default within 60 s** (`SYJ208+1.005` in `-01` and `-cbn`): their
  saturation needs some 700 million of the inverse method's units, and
  the turns get a thirty-second of the engine's work. Forced, the inverse
  method answers them in about 12 s, its set checked.

### The checklist

| item | state | evidence |
|---|---|---|
| The dispatch: the inverse method where a measurement shows it winning, a row of `DISPATCH` with its measurement, no row the default decides lost, a few percent at most where the default is fast | done | `mtwmloox`, `ssmwxkoq`, `unywztnt`, `ykntmzxo`; `Engine`'s table and paragraph; "The measurement": 102 won, 0 lost, medians 1.00 at every speed |
| The saturation's certificate, checked without the engine's code, a wrong certificate refused for each condition, the 58 answered `Unprovable` with it | done | `ysqzqtvv`, `zplymslu`; `saturated::tests`; the 58 forced: 58 `unprovable`, checked `ok` |
| The panel on frozen trees (the argument on Fable 5.1, `high`), each finding answered | done | "The panel": two rounds, F1, F2, (c) to (f), the scan, A and B |
| The measurement before and after (held-out sets, the 384, class C, the six, the families), no verdict against a checked proof of any tool | done | "The measurement"; `analysis.txt`; the 384: 24 decided before, 112 after, none lost |
| The target sets against `after-cyclic.csv` and `net-after-cyclic.csv` | done: every row the dispatch does not move unchanged | "The target sets" |
| Every code commit gated | done | `gate: passed` before each of `ozwvpkoy`, `ysqzqtvv`, `mtwmloox`, `ssmwxkoq` (in `../linlog-step37b-gate`), `zplymslu`, `unywztnt`, `ykntmzxo`, `tozoztmp` and the last commit |
| `nix flake check` at the last commit | passed | exit 0 at `mqoxzmnk` and at `kuylznmt` (20:06 and 20:11, "all checks passed!"), run again under the machine lock after this report's commit; that exit status is in the message to `planning` |
| `.claude/rules/` (`core-inverse.md`, `core-search.md`, `core.md`, `core-sequents.md`, `core-export.md`, `bench.md`, `rocq.md`), README, the changelog | done | the commits above |

### What is left open

- **The two `SYJ208+1.005`**: a turn's budget is a share of the engine's
  work; a goal the inverse method needs much longer for stays the
  engine's.
- **The teardown after a stop** (37a's residue, the integers review's A)
  and **the resident memory after turns** (B): flat arrays for the rule
  compiler's holes would cut both.
- **The inverse method's units against time**: within a factor of 30 now
  (5 000 to 143 000 a millisecond before the last fixes), not 1; a turn's
  time can be several times a sixteenth of the engine's on rows like
  `SYJ212+1.018` in cbn.
- **A Rocq theorem for a saturated set**: linlog's library would need the
  focused calculus and its completeness.
- **The workspaces**: `../linlog-step37b` (this session's commits),
  `../linlog-step37b-gate` (the gate of `ssmwxkoq`, an empty change on
  it), `../linlog-step37b-panel` and `-panel2` (the panels' frozen trees
  at `mtwmloox` and `zplymslu`); none holds uncommitted work.
