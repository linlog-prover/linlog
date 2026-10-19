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
