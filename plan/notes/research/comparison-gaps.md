# Where the other provers beat linlog, and what to do about it

Written 2026-10-17 from step 29's run (`bench/comparison/2026-10-15/`),
the engines' code at the working copy of that day, the earlier baselines
and reports, and the sources of section 7. Citations are in brackets:
[n] for the numbered sources, `file:line` for code. "(inference)" marks a
conclusion of this note's own. No probe was run: the units
`step30-mut-before` and `step30-mut-after` and a `nix flake check` were
active the whole session, so every probe this note needs is written down
in section 6 instead.

The short answer. Every loss is one prover (Sympli) on one kind of input
(the intuitionistic LLTP collections, which are translations of ILTP's
propositional formulas), and three mechanisms explain them. The largest
(94 of 122) does not look like a limit of backward search at all: the
spaces are tiny, the backward search deepens its copy bound to millions,
every level is cut, and engines of 2026-10-02 to 10-04 refuted at least
four of these rows at levels 3 to 13. That is a defect of decisiveness
to diagnose and fix, cheap and general. The second (19) is a linear
zone that grows forever with members that only a `⊤` can absorb. The
third (9 proofs) is what forward saturation does better than backward
deepening, which step 37 is planned for.

## 1. The losses in numbers

The method: one row per problem and tool in `runs.csv` (62 543 rows). A
row is *solved* as `linlog-bench ranking` counts it (`bench/src/ranking.rs:24-120`):
`proved` or `unprovable` within 10 s, and not wrong (a refutation of a
problem some tool proved with a checked proof, or a verdict against a
generated problem's construction). A *loss* is a problem some other
tool solves and neither of linlog's two columns (`linlog`, which is
`linlog prove --deterministic`, one thread [`bench/src/tools.rs:138-193`];
`linlog-default`, the race of one thread and a pool on four cores) does.
Scripts: `scratchpad/gaps/gaps.py`, `classes.py`, `buckets.py`,
`classical.py` (not in the repository).

### 1.1 Both directions, by group

| group | problems | linlog alone | both | others alone | neither |
|---|--:|--:|--:|--:|--:|
| LLTP intuitionistic, not nets | 1 358 | 78 | 774 | **122** | 384 |
| LLTP Petri nets | 3 137 | 2 206 | 866 | **31** | 34 |
| LLTP classical | 17 | 2 | 15 | 0 | 0 |
| qcover | 176 | 162 | 0 | 0 | 14 |
| generated families | 123 | 64 | 37 | 0 | 22 |
| all | 4 811 | 2 512 | 1 692 | 153 | 454 |

"linlog alone" counts either column. The 153 losses are two sets:

- **122 to Sympli**, all intuitionistic (`ILL`), all of them `timeout`
  in both of linlog's columns, all on the two-sided engine. Step 29's
  report counts 132 against the one-thread column; the default solves 10
  of those, and 122 are lost by both.
- **31 to APLL (ILL)**: Petri nets that APLL refutes against their
  `Theorem` header and that linlog's Horn engine leaves at the time
  limit. APLL refutes 1 296 nets that linlog proves with checked proofs
  [`bench/TOOLS.md`, "Wrong answers"], 126 of them in the same 13 model
  families as these 31 (all 60 of `PolyORBLF`'s wrong refutations, for
  one). They are not counted as losses below (section 5).

### 1.2 The 122 Sympli wins

| | count |
|---|--:|
| Sympli refutes, header Non-Theorem | 93 |
| Sympli refutes, header Theorem (header wrong or Sympli wrong) | 18 |
| Sympli refutes, no status in the header (`SYJ209+1.011`, `.012` in cbn) | 2 |
| Sympli proves (all `ILLTP-SYJ-cbn`: `SYJ205+1.013` to `.020`, `SYJ203+1.011`) | 9 |

By collection: `ILLTP-SYJ-cbn` 45, `ILLTP-SYJ-01` 42, `ILLTP-SYJ-cbv`
15, `KLE` (three translations) 14, `ILLTP-SYN` 4, `ILLTP-LCL` 2. The
rows are small: 15 to 387 occurrences, median 81, 75 of them under 100.
Sympli is fast on them: median 5.4 ms (most of it its start), 98 within
50 ms, 110 within 1 s. They come from only 22 formula stems (SYJ210 32
rows, SYJ209 24, SYJ211 20, SYJ208 9, SYJ205 8, the rest 1 to 4 each),
because the SYJ problems are parameterised families in three
translations each: a count of rows overstates how many distinct
weaknesses there are.

### 1.3 The three classes

Each class is read off linlog's own report in the row's `raw` column,
"the time limit of 10s was reached at a copy bound of N", the level the
backward search had begun (`Statistics::copies` [core-focus.md, "The
copy budget"]).

| class | rows | verdict | linlog's copy level at 10 s | Sympli | stems |
|---|--:|---|---|---|--:|
| A: every level cut | 94 | refutations | 9 to 11 256 956, median 231 847 | median 5.9 ms, max 8.2 s | 20 |
| B: a growing linear zone | 19 | refutations | 512 on every row | 4 ms | 1 (SYJ210 in cbn, sizes 2 to 20) |
| C: levels too expensive | 9 | proofs | 6 to 11 | 13 ms to 5.2 s | 2 (SYJ205, SYJ203 in cbn) |

Of class A, 78 rows reached a level of 1 000 or more; the other 16 are
the larger sizes of the same families (SYJ208 `.003`/`.004`, SYJ209
`.007` to `.012`, SYJ210 `.007` in `01`), where each level costs more.

**Earlier engines decided some of these rows.**

- `KLE-cbv/KLE017+1` and `KLE-01/KLE069+1`: refuted on 2026-10-04 by
  `linlog prove -i --deterministic` at copy levels 5 and 13, in under
  10 ms each [`plan/notes/lltp-headers.md`, rows 18, 19]; in step 29's
  run (60226b2c) undecided at levels 3 542 531 and 491 245.
  (`KLE-cbn/KLE017+1`, refuted at level 5 then, now only by the default.)
- `ILLTP-LCL-cbv/LCL181+1`: "one thread refutes it at once (0.12 ms,
  levels 3 and 5)" at step 26 [`plan/reports/26-focused-engine.md:329-332`];
  now undecided at 3 225 212 levels.
- `ILLTP-SYJ-01/SYJ212+1.002` (and the two above in `cbv`): refuted at
  `--copies 10` in 0.05 to 0.9 ms in the baselines of 2026-09-30 and
  10-02 [`bench/results/2026-10-02/lltp-copies-10.csv`].

Since a search without a copy bound runs the levels of the bounded one
first [core-focus.md, "Without a copy bound"], a level that ended uncut
then and does not now is a change in the engine (inference). At least
four of the 94 rows are such regressions; how many more is what a
bisection would show (section 6).

**A classical countermodel exists for 26 of the 113 refutations**
(`classical.py`: `!` erased, `⊸` as implication, `⊗` and `&` as "and",
`⊕` as "or", `0` false, truth tables up to 18 atoms; 69 are
classically valid, 18 have more atoms). They include 15 of the 18
wrong-header rows (as [`bench/TOOLS.md`, "The library's headers"]
found) and 11 genuine Non-Theorems (SYJ211 `.001` to `.007` in `01`,
SYN392 in `01` and `cbv`, SYN040 and LCL181 in `cbv`). The three
`KLE075+1` rows have none (section 5).

### 1.4 Not losses to another tool, but owed

- The default column ends 3 runs at the 4 GiB cgroup limit
  (`SYJ206+1.020`, `SYJ212+1.020` in two translations): a charge missing
  from the memory bound, step 30b's fix [`plan/later.md`, "The memory
  bound is passed on wide intuitionistic searches by default"].
- 384 intuitionistic and 34 net problems nobody solves in 10 s; step
  29's report proposes a second night at 60 s for them.

## 2. Why, class by class

### 2.1 How the two-sided engine refutes at all

The two-sided engine is the focused engine on the lowered sequent
[core-focus.md, "Two-sided is one constraint"]. With exponentials it
deepens a per-branch copy budget from 0 (`core/src/search/focus/mod.rs:686-718`);
a copy of a `Θ` member costs one unit (`mod.rs:1617-1633`), and a stable
sequent at budget 0 that still has something to copy marks its level cut
(`mod.rs:1518-1523`). `Unprovable` needs one level that ended with no
cut (`mod.rs:710`); otherwise the next level runs, without end under the
command's default (no copy bound) until the time limit. Two things
replace cuts with facts:

- the **loop check**: a stable sequent equal to an ancestor on the
  branch is pruned (`mod.rs:1079`, `repeated` at `mod.rs:1317`), and
  the failure carries a dependency that is discharged at the ancestor
  [core-focus.md, "The loop check"]; equality is of the keys, `Θ` by
  its id and `Γ` as a multiset;
- the **memo**: `Proved` and `Complete` are facts at any budget;
  `Exhausted(r)`, "cut by the budget with `r` copies left, applies only
  when at most `r` are left now", and **entries survive across levels**,
  "that is where the re-exploration of deepening is recovered"
  [core-focus.md, "Memo contract with the bound"; `memo.rs:466-475`]. A
  hit on an `Exhausted` entry is answered as a cut without a search,
  after the loop check and the waiting failures (`mod.rs:1101`).

So a refutation needs the whole reachable space of stable sequents to
end in facts and repeats within one level. That fails in two ways:
something keeps the level from closing although the space is finite
(class A), or the space is infinite (class B).

### 2.2 Class A: every level cut, on a small space

**Evidence.**

- The levels are cheap and cut: at 10 s the copy level reached is in
  the millions on 15 to 60 occurrences (`KLE-cbv/KLE015+1` 11 256 956;
  `SYJ211+1.001` in `01` 6 158 999). A chain of copies as deep as the
  budget would hit the recursion limit (2 048, `core/src/limits.rs:98`,
  three or four levels of recursion per copy) at about level 512, as
  class B does; so in class A the cut at each level lies at a depth
  that does not grow with the level (inference).
- The spaces are tiny: `ILLTP-SYJ-cbn/SYJ208+1.001` (21 occurrences,
  `!(!(!o11 ⊸ 0) ⊸ 0), !(!(!o21 ⊸ 0) ⊸ 0) ⊢ !o11 ⊕ !o21`, a
  Non-Theorem) at `--copies 10`: 375 stable sequents over 11 levels, 15
  memo entries, 228 hits, `copy_bound`
  [`bench/results/2026-09-30/lltp-copies-10.csv`]. By hand its stable
  sequents are `Θ₀` with each goal, then `Θ₀ ∪ {o11⊥}` and `Θ₀ ∪ {o11⊥,
  o21⊥}` with goal `0` and at most one released copy of each literal
  (`one_of_each_copy`, `mod.rs:1699`): about 15, every branch ending in
  a repeat within four copies (inference). The memo holds that space,
  and every level still ends cut.
- The verdict depends on the order of the search: `LCL181+1` in `01`
  stays undecided on one thread for 3.2 million levels, while pools of
  two refute it at levels 28 to 544 and pools of four at 10 to 67;
  `SYN393+1` in `cbn` is refuted by a pool of four in milliseconds and
  never by one thread [`plan/reports/26-focused-engine.md:1062-1069`].
  That report names the cause it suspected: "where the loop check meets
  the memo's cut entries".
- Engines of 2026-10-02 to 10-04 refuted four of these rows at levels 3
  to 13 (section 1.3); the space did not change, the engine did.

**The mechanism, as far as the code shows it** (inference; probe A1 of
section 6 decides it). An `Exhausted` entry records that a sequent
failed within a budget *in the branch it was searched in*, where some of
its descendants were cut and others pruned as repeats of that branch's
ancestors. Read in another branch, or at the next level, it answers
"cut" without a search (`mod.rs:1101`), although a search there would
have pruned the same descendants as repeats of the new branch's
ancestors, or would no longer run out of budget. A sequent reached at
level `k + 1` by a path with one copy more than at level `k` meets its
level-`k` entry at a budget it covers (`memo.rs:468-471`), so the cut
of level `k` becomes a cut of level `k + 1`, and so on: once a cut is in
the memo it can be carried from level to level for ever, at a constant
cost per level. Nothing in the memo contract is wrong for verdicts (a
cut never refutes anything), but deepening is then not complete even
on a finite space. This is the graph-history interaction of game-tree
search: a result that depends on the path that led to a position,
cached under the position alone [10]. Which branch meets which entry
first is the order of the search, which is why pools of different sizes
decide at different levels, and why changes to the order (the canonical
keys, the waiting failures of step 28i, which carry `exhausted` flags
to the failures below a cut sequent [`core/src/search/focus/waiting.rs:24-36`],
the one-sided rework) can lose refutations without any target row
moving: the target set runs at a copy bound of 3 [`bench/TARGETS.md`],
and none of these sequents is refuted at 3 (inference).

**What Sympli does instead** [1, 3, 4]. A forward sequent keeps the
unrestricted zone as a set (`factor` resets every count to 1) and the
linear zone as a multiset, with a weak flag; a weak sequent subsumes
every one with a larger linear zone, and a strong one only an equal zone
[1, `core/sequent.sml`, `subsumes''`]. Saturation stops when no new
sequent survives subsumption; "ABORT: Saturated" is its refutation, and
there is no budget, so no level can be cut (driver:
`bench/src/tools.rs:343-364`). On these translations nearly every
hypothesis is under `!`, so the derived sequents are few (inference).

### 2.3 Class B: a linear zone that grows

`ILLTP-SYJ-cbn/SYJ210+1.002` (25 occurrences): `!(!(!p2 ⊸ 0) ⊸ 0),
!(!p1 ⊸ (!p1 ⊸ p0)), !(!p2 ⊸ (!p2 ⊸ p1)) ⊢ p0`, Schwichtenberg's
formula, a Non-Theorem. Read by hand (inference): the goal `p0` uses the
second hypothesis, `p1` the third, `p2` only the first, whose `0` on the
left absorbs everything and leaves `Θ ∪ {p2⊥} ; ⊢ 0`. There, a copy of
the second hypothesis sends the goal `0` to the consequent's side with a
fresh `p0` hypothesis: `Θ' ; p0 ⊢ 0`, then `p0, p0 ⊢ 0`, and so on. The
`p0`s can never meet an axiom (the only `p0` on the right was the root's
goal), only the `⊤` (the left `0`) can take them, and every such
sequent is provable exactly when the one without them is. But they are
different multisets, so the loop check never sees a repeat, and the
skip of a formula with an unconsumed copy (`mod.rs:1699`) looks at the
copied formula, not at what its consequent left in `Γ`. The branch is
as deep as the budget, and at level 512 the recursion limit ends the
backward search: every one of the 19 rows stops at 512, with a peak
of about 5 MB, whatever the size. That is the same number as the `growing`
sequent `!(A ⊸ A ⊗ A), A ⊢ ?B`, which "ends at the recursion limit at a
copy bound of 512" [`plan/reports/21-defaults.md:503-506`] (and the
engine's test `classic_exponentials` on `!(a ⊗ b) ⊢ !a ⊗ !b`: "every
copy grows the context"). The forward search then runs to the limit.

Sympli saturates this in 4 ms: the left `0` makes its sequents *weak*,
and a weak sequent subsumes all of `p0ⁿ ⊢ 0` at once [1, 3, Def. 3.7 as
cited in `37-inverse.md`].

### 2.4 Class C: proofs whose levels cost too much

`SYJ205+1.n` in `cbn` (Korn and Kreitz's provable formulas: the whole
hypothesis is one `!` of a conjunction of `n` implications) and
`SYJ203+1.011`. linlog proves `SYJ205` up to `.011`, its time growing
about fourfold per size (19.7 ms at `.006`, 4.2 s at `.011`), and stops
at copy level 6 to 9 from `.012` on; Sympli takes 4 to 24 ms for every
size [`runs.csv`]. A proof needs a copy per use of a conjunct, the
backward search at budget `b` tries the interleavings of every copyable
formula at every stable sequent, so each level costs about the number
of candidates to the power `b` (inference). Forward, each conjunct is a
derived rule, and subsumption keeps one sequent per goal and context
set [3]. `SYJ203` (`.011`) is near the limit for both: Sympli 5.2 s,
linlog 1.1 s at `.010`. This is the case the inverse method is for:
"many hypotheses and a small goal" [`plan/37-inverse.md`]. Note that
later.md's premise for step 37, "the intuitionistic library outside the
nets ends at the copy bound, not at the time limit … a question of
`--copies`" [`plan/later.md:243-245`], no longer holds since the
default deepens without a bound: these rows end at the time limit.

### 2.5 Why only Sympli

The other ILL provers lose on the same rows for the same reason as
linlog or worse: the Maude prover ends at its copy bound of 5
(`copy_bound` on `SYJ208+1.001`, `SYJ210+1.00n` in 130 to 210 ms),
llprover at its threshold, APLL at its bound of three contractions
[`bench/TOOLS.md`, "The tools"]. All of them are backward searches with
a contraction bound; Sympli is the only forward one.

## 3. What the literature offers

**What any tool can do.** MALL is PSPACE-complete, and full propositional
linear logic is undecidable, its intuitionistic fragment with `!`
included [5]; MELL's decidability is open; affine linear logic is
decidable [6] and Tower-complete, already for its multiplicative
exponential fragment [7]. Intuitionistic propositional logic is
PSPACE-complete [8]. So no procedure refutes every unprovable ILL
sequent; the losses here are on images of propositional intuitionistic
formulas (LLTP's ILLTP collections are ILTP's problems under three
translations [9]), which are decidable through the original logic, and
Sympli's saturation happens to terminate on them.

**Loop checking in backward search.** For intuitionistic logic the
context only grows, so a loop is a repeated goal with the same context,
and Heuerding, Seyfried and Zimmermann's history mechanism stores goals
only and empties the history whenever the context grows [11, §2,
describing 12]. Howe's two variants detect loops earlier or with less
storage, and both are complete [11]. The linear zone of ILL does not
only grow, so equality on the branch is linlog's check, and it is
complete only where the space is finite. Contraction-free calculi go
further: Dyckhoff's G4ip (LJT) terminates without any loop check,
decides intuitionistic propositional logic, and Hudelmaier's variant
runs in `O(n log n)` space [13, 14]; Pinto and Dyckhoff read
countermodels off its failed searches [15]. No linear analogue for ILL
with `!` exists that we found (inference: none can be terminating in
general [5]).

**Caching under a loop check.** A result that depends on the path is
unsafe to cache under the position alone: the graph-history interaction
problem of game-tree search, with a general solution by Kishimoto and
Müller that records the path dependence with the cached value [10].
linlog's waiting failures are one such record for the loop check's
prunes [`waiting.rs:1-70`]; `Exhausted` entries carry no such record of
where their cuts came from (inference).

**Forward search and disproving.** The focused inverse method keeps only
neutral sequents, uses weak sequents for `⊤` and `0`, and saturates
[2, 3]; `37-inverse.md` §1 has the details and the cross-prover numbers
(Imogen 261 of ILTP's 274 propositional problems, better on
non-theorems). Forward search can fail to terminate on unprovable
queries; Brock-Nannestad and Chaudhuri bound it by finite approximations
of the types, with forward subsumption and globalisation, and refine
them iteratively to disprove [16]. Fiorentini and Ferrari's forward
refutation calculus for intuitionistic propositional logic yields
countermodels of minimal height [17]. SAT-based intuitionistic provers
(intuit) were the strongest on about 1 200 standard benchmarks, ILTP
included, as of 2015 [18, as reported in 19].

**Countermodels as refutations.** Classical countermodels refute any
linear sequent whose classical reading is invalid (every rule is
classically sound under the reading), and the certificate is an
assignment [`plan/later.md`, "Certified refutations"]. Finite phase
models are complete for MALL and the affine fragments [20, 21]; Kripke
models for intuitionistic logic need a terminating calculus to find
them [15, 17].

**Not what these losses need.** Lazy resource distribution [22, 23] and
multi-focusing [24] cut the cost of `⊗` splits and permutations; these
rows are copy-bound, not split-bound (inference). Focusing [25] linlog
has.

## 4. Recommendations, ranked by gain per effort

Each says what it is, which losses it addresses, why it is general,
what it costs in linlog's design, and how to measure it without fitting
LLTP. The held-out discipline for all of them: decide the method on one
part, judge it on another, and report stems as well as rows (section
1.2), saying which collections translate one another (ILLTP and ILTP),
so that one family's twenty sizes do not read as twenty weaknesses.

- **Tuning set**: `ILLTP-SYJ-01` (42 rows lost), and the problems of
  step 25's report that deepen past 30 [`plan/reports/25-ordinary-logic.md:300-318`].
- **Held-out sets**: `KLE` (Kleene's theorems, another source than ILTP),
  `ILLTP-SYN` and `-LCL`, the `cbv` translation, and two sets with no
  LLTP in them: random intuitionistic formulas judged by an independent
  G4ip decision procedure [13] written for the test (a page of code,
  inference), translated by `ordinary::translate` under each
  translation (provability is preserved, so the oracle's verdict is the
  image's); and the generators of `search/generate.rs`, whose
  intuitionistic sequents with effective contractions (F122's fix
  [`plan/later.md`, "The generators' contraction and mutants"]) need
  copies. ILTP itself (`nix build .#iltp`) is *not* held out: its
  problems are the ILLTP collections' sources [9; step 25's report: 199
  of 274 images equal LLTP's files].
- **Regression oracles**: both target sets against their
  `after-efficiency` oracles (CLAUDE.md's table), the behaviour lock,
  the families (no `MISMATCH`), qcover, and the reference prover's
  comparisons [core-search.md, "The reference prover"].

### R1. Diagnose and fix the cut entries that keep levels from closing (class A)

*What.* First the probes A1 to A3 (section 6): if the memo off
(`--memo-limit 0`) refutes `SYJ208+1.001` in `cbn` and `LCL181+1` in
`cbv`, the mechanism of 2.2 is confirmed. Then bisect the four known
regressions between the engine of 2026-10-04 (b77af819, step 25's
report) and 60226b2c (probe A4). Then one of two fixes, each a change of
the memo contract with a panel's argument:

1. *No `Exhausted` entry across levels*: a level reads only `Proved`
   and `Complete` from earlier levels. On a finite space with the exact
   loop check, a level whose budget exceeds the copies of the longest
   loop-free branch has no genuine cut, so its first cut entry cannot
   arise, and the level ends uncut (inference, to be argued by the
   panel). The price is the re-exploration that deepening shares
   today, a constant factor on searches that deepen far.
2. *A level without the old cuts when the search stagnates*: keep
   today's reuse, and when a level ends cut without having recorded a
   new key, run the next level with the `Exhausted` entries of earlier
   levels ignored. Rows that decide at copy 3 keep their counters
   (inference: they do not stagnate), so the target set and the
   behaviour lock stay as they are, and the trigger costs one counter.

*Losses addressed.* Up to 94 rows in 20 stems, all refutations; the 78
with levels of 1 000 or more first, which by their cheap levels are the
likely ones. Also step 25's 35 ILTP problems that "deepened past a copy
bound of 30 … exactly what a loop check … would turn into refutations"
[`plan/reports/25-ordinary-logic.md:300-310`], which are the same
formulas through `ordinary`.

*Beyond the benchmark.* Any sequent with `!` whose reachable space is
finite but whose branches reach the same sequent with different copy
counts: every intuitionistic formula through `ordinary`, every
`!`-hypothesis used more than once. Deepening that is complete on finite
spaces is a property a user can rely on; today it depends on the search
order [`plan/reports/26-focused-engine.md:1062-1069`].

*Cost and risk.* `memo.rs` and `prove_listed` (`mod.rs:1012-1215`), the
waiting failures' rules (`waiting.rs:24-56`), core-focus.md's "Memo
contract with the bound". Soundness is not at stake: a cut never
refutes, and `Unprovable` still needs an uncut level. Counters of
undecided-at-3 rows move; the target set and the behaviour lock show
whether decided rows moved (fix 2 should move none).

*Measure.* Tune on `ILLTP-SYJ-01`; judge on KLE, SYN, LCL, `cbv`, the
random G4ip-judged formulas and the generators; report rows and stems;
add the regressions as tests under a work bound (deterministic:
`--deterministic`, `Limits::work`). And add to the target set a handful
of small rows run *without* a copy bound under a work bound, since its
copy bound of 3 is why none of this was seen (section 6, A5).

### R2. A classical countermodel as a refuter

*What.* After a search ends `Unknown`, test the classical reading of
the sequent with a small SAT or truth-table pass (bounded by work), and
answer `Unprovable` with the assignment as the refutation. The checker
is a one-line evaluation, the Rocq certificate is planned for step 31
[`plan/later.md`, "Certified refutations"; `refutations.md`].

*Losses addressed.* 26 of the 113 refutations (section 1.3), 15 of them
the wrong headers, 11 Non-Theorems; all 26 are in class A, so R1 may
decide them too, without a certificate.

*Beyond.* Any linear sequent whose classical reading fails, the common
case of a wrong conjecture; it is how the header report found its
countermodels [`plan/notes/lltp-headers.md`].

*Cost and risk.* Small. It runs against a decision of the author's:
"Refuters run after the search, never during it … after an `Unknown`
only with `refute_unknown`, which is off by default" [core-search.md,
"Decisions"]. Turning it on for this refuter alone, or making the
command's default turn it on, is the author's call (section 6, open
question 1). No verdict a search gave changes.

*Measure.* No row it refutes may have a proof by any tool; a test of the
reading against checked proofs of generated sequents pins the reading.

### R3. Cap the members only a `⊤` can take (class B)

*What.* In a stable sequent, a literal of `Γ` whose dual occurs nowhere
in `Γ` nor below a member of `Θ` can end only in a `⊤` leaf (linear
mode: it cannot be weakened, and promotion needs it gone). Keep at most
one copy of each such literal in the key and the context. Two copies
end in `⊤` leaves; dropping one gives a proof of the smaller sequent,
and a proof of the smaller one takes a second copy into the same leaf,
across `&` in both branches (inference, a panel's argument). The test is
the copy heuristic's own (`mark_literals` and `meets`, `mod.rs:1827-1863`).
It is a narrow, backward form of Chaudhuri's weak sequents [3].

*Losses addressed.* The 19 rows of class B (one stem): with the cap
their space is finite, so with R1 they refute (inference).

*Beyond.* Every intuitionistic negation under the call-by-name
translation is `!A ⊸ 0`, a `⊤` on the left in every use, so images of
formulas with negation meet this whenever an atom's only positive
occurrence has been used (inference). Measured on the held-out sets, not
on SYJ210.

*Cost and risk.* Medium: the context, the memo key, and the proof, which
must put each dropped copy back into a `⊤` leaf for the checker. After
R1, so that its own effect is measured. *Measure.* The held-out sets,
the reference prover's comparisons, the target set's counters.

### R4. The focused inverse method (step 37), aimed where it is needed

*What.* Step 37 as planned [`plan/37-inverse.md`; `37-inverse.md`], but
measured on this note's classes: class C's proofs, and, as a second
refuter by saturation, whatever R1 and R3 leave of A and B.

*Losses addressed.* 9 proofs in 2 stems directly; up to all 122 as a
second engine (Sympli is one).

*Beyond.* "Many hypotheses, small goal" is the inverse method's case
[2, 3]; Imogen's ILTP results [`37-inverse.md` §1].

*Cost and risk.* Two sessions and a panel, the largest by far; the risks
are `37-inverse.md` §4's (completeness of the derived rules, memory,
non-saturation with exponentials). Do R1 first: if R1 decides class A,
the engine's case rests on C and on the "neither" rows, and its dispatch
row must be measured against a backward search that refutes.

*Measure.* The held-out sets and the 384 intuitionistic "neither" rows
at the longer limit of step 29's proposal, never only the 122.

### Ranking

| | addresses | rows (stems) | effort | general beyond LLTP |
|---|---|--:|---|---|
| R1 | class A, regressions | ≤ 94 (20) | small to medium, a panel | yes: deepening complete on finite spaces |
| R2 | classically invalid | 26 (overlaps R1) | small, an author's decision | yes: any wrong conjecture |
| R3 | class B | 19 (1) | medium, a panel | probable: cbn negation |
| R4 | class C, and a second refuter | 9 (2), more as a refuter | large, two sessions | yes: many hypotheses |

## 5. What not to chase

- **APLL's 31 nets.** APLL refutes 1 296 nets linlog proves with checked
  proofs, through its own LLTP reader too [`bench/TOOLS.md`]; these 31
  lie in the same model families. Nothing proves them; nothing should
  be built to refute them.
- **The 15 wrong headers** as a goal. They are 15 rows where the right
  answer is a refutation with a classical countermodel; R2 takes them in
  passing, and the header report owes them to the library's maintainers
  [`plan/notes/lltp-headers.md`].
- **`KLE075+1` in its three translations** until it is understood:
  Sympli refutes it, the header says Theorem, and it has no classical
  countermodel ([`bench/TOOLS.md`]; confirmed by `classical.py`). Either
  the header is wrong for an intuitionistic reason or Sympli is wrong;
  Sympli's refutations are unchecked. Decide it by hand or with an
  independent intuitionistic prover before counting it.
- **The larger sizes of the exponential families** (SYJ208 from `.004`,
  SYJ209 from `.009`, SYJ210 in `01` from `.007`): Sympli's own times
  grow by about four to seven per size there (SYJ209 in `01`: 185 ms,
  1.1 s, 8.2 s for `.007` to `.009`), so a win is a size or two, which
  is the benchmark's shape, not a technique.
- **Anything keyed to the translations' shapes** (recognising Girard's
  `!A ⊸ B`, or a decision for images only): the general techniques of
  section 4 cover the same rows, and a shape test overfits to how LLTP
  was generated.
- **The turn shares** (`FIRST_TURN`, `BACKWARD_SHARE`,
  `core/src/search/focus/schedule.rs:113-120`) and the default's pool:
  moving them to win these rows is tuning to them; the one-thread column
  loses the same rows.

## 6. Open questions for the author, and the probes owed

**Open questions.**

1. May a refuter answer an `Unknown` by default (R2)? The decision of
   [core-search.md, "Decisions"] keeps `refute_unknown` off so that
   every baseline keeps its verdicts; a refuter that only turns
   `Unknown` into `Unprovable` changes no decided verdict.
2. Is a change of the memo contract (R1) acceptable if it moves counters
   of rows undecided at copy 3 but of no decided row (fix 2), or of
   decided rows too (fix 1)?
3. Does step 37's prompt keep its starting point? Its premise that the
   library ends at the copy bound rather than the time limit is out of
   date (section 2.4), and its first measurement should follow R1.
4. Should the target set gain rows run without a copy bound under a work
   bound (A5), so that a loss of decisiveness at deeper levels shows?
5. `KLE075+1`: worth an hour by hand, or a message to Sympli's author
   (the drafts are in `plan/notes/comparison-messages.md`)?

**Probes**, each on one problem, in a capped scope on cores 12 to 15
(`systemd-run --user --scope -p MemoryMax=4G -p MemorySwapMax=0 taskset
-c 12 timeout 30 …`), with the release build of the working copy, once
no mutation unit or flake check runs. `L=bench/lltp/ILL`.

- A1, the memo's part: `linlog prove -i --deterministic --stats
  --timeout 10s --file $L/ILLTP-SYJ-cbn/SYJ208+1.001.p`, then the same
  with `--memo-limit 0`. Expected if 2.2 is right: the first undecided at
  a level in the millions, the second refuted at a level below 10.
- A2, the same pair on `$L/ILLTP-LCL-cbv/LCL181+1.p` and
  `$L/KLE-cbv/KLE017+1.p`, and with `--bias rarer` and `--bias factors`
  each, to see which search fails to close.
- A3, class B's chain: `$L/ILLTP-SYJ-cbn/SYJ210+1.002.p` with
  `--recursion-limit 8192 --stats`: the level reached should rise from
  512 to about 2 048, proportional to the limit, and stay undecided.
- A4, the bisection: build at b77af819 (2026-10-04) and at 60226b2c in a
  jj workspace each, run `KLE-cbv/KLE017+1` and `KLE-01/KLE069+1` with
  `--deterministic --timeout 2s`; then halve the range over the focused
  engine's commits of 10-04 to 10-15 (`jj log -r 'files("core/src/search/focus/")'`).
- A5, the oracle gap: the 122 rows at `--deterministic --work-limit N`
  with N about a second's work, as a set to compare before and after R1.

**Where this note changes the earlier notes.** `37-inverse.md` §2 and
`plan/later.md` ("The focused inverse method") take the intuitionistic
library's undecided rows to end at the copy bound; under the default
without a bound they end at the time limit, and most of what Sympli wins
is not a matter of direction but of levels that never close (R1).
`refutations.md` and `later.md` ("Certified refutations") are confirmed:
the classical countermodel is the cheap refuter, and here it would
settle 26 rows. Step 25's estimate of 35 ILTP problems for a loop check
is confirmed in kind, with the correction that linlog now has a loop
check and the rows are still undecided, which points at the memo's cut
entries rather than at the check itself.

## 7. Sources

Repository sources are cited inline by path. External sources, read
2026-10-17 unless marked "cited", which means taken from the note or
file named and not reread.

1. K. Chaudhuri, Sympli, https://github.com/chaudhuri/sympli at `50e76a6`
   (the flake's revision): README, `core/sequent.sml` (`factor`, `subsumes''`).
2. K. Chaudhuri and F. Pfenning, "Focusing the inverse method for linear
   logic", CSL 2005, LNCS 3634, pp. 200–215 (cited, `37-inverse.md`).
3. K. Chaudhuri, *The Focused Inverse Method for Linear Logic*, PhD thesis,
   CMU-CS-06-162, 2006, Defs. 3.5–3.7, 4.11 (cited, `37-inverse.md`).
4. K. Chaudhuri and F. Pfenning, "A focusing inverse method theorem prover
   for first-order linear logic", CADE-20, LNCS 3632, 2005 (cited, same).
5. P. Lincoln, J. Mitchell, A. Scedrov and N. Shankar, "Decision problems
   for propositional linear logic", APAL 56, 1992, pp. 239–311 (cited,
   `plan/later.md`, "Certified refutations").
6. A. Kopylov, "Decidability of linear affine logic", LICS 1995 (cited, same).
7. R. Lazić and S. Schmitz, "Non-elementary complexities for branching VASS,
   MELL, and extensions", ACM TOCL 16(3), 2015; arXiv:1401.6785.
8. R. Statman, "Intuitionistic propositional logic is polynomial-space
   complete", TCS 9, 1979, pp. 67–72 (not reread).
9. C. Olarte, V. de Paiva, E. Pimentel and G. Reis, "The ILLTP library for
   intuitionistic linear logic", EPTCS 292, 2019, pp. 118–132; arXiv:1904.06850.
10. A. Kishimoto and M. Müller, "A general solution to the graph history
    interaction problem", AAAI 2004, pp. 644–649;
    https://webdocs.cs.ualberta.ca/~mmueller/ghi.html.
11. J. M. Howe, "Two loop detection mechanisms: a comparison", TABLEAUX 1997;
    https://openaccess.city.ac.uk/id/eprint/1707/ (§1, §2.1).
12. A. Heuerding, M. Seyfried and H. Zimmermann, "Efficient loop-check for
    backward proof search in some non-classical propositional logics",
    TABLEAUX 1996, LNCS 1071, pp. 210–225 (not read; through [11]).
13. R. Dyckhoff, "Contraction-free sequent calculi for intuitionistic logic",
    JSL 57(3), 1992, pp. 795–807 (not reread).
14. J. Hudelmaier, "An O(n log n)-space decision procedure for intuitionistic
    propositional logic", J. Logic Comput. 3(1), 1993, pp. 63–75 (not reread).
15. L. Pinto and R. Dyckhoff, "Loop-free construction of counter-models for
    intuitionistic propositional logic", *Symposia Gaussiana*, 1995 (not reread).
16. T. Brock-Nannestad and K. Chaudhuri, "Disproving using the inverse method
    by iterative refinement of finite approximations", TABLEAUX 2015, LNCS
    9323, pp. 153–168; https://www.lix.polytechnique.fr/~kaustuv/papers/tableaux15saturate.pdf
    (abstract only).
17. C. Fiorentini and M. Ferrari, "Duality between unprovability and
    provability in forward proof-search for intuitionistic propositional
    logic", arXiv:1804.06689 (abstract only).
18. K. Claessen and D. Rosén, "SAT modulo intuitionistic implications",
    LPAR-20, LNCS 9450, 2015, pp. 622–637 (not read).
19. C. Fiorentini, R. Goré and S. Graham-Lengrand, "A proof-theoretic
    perspective on SMT-solving for intuitionistic propositional logic",
    TABLEAUX 2019 (its claim about intuit through a search summary; not read).
20. Y. Lafont, "The finite model property for various fragments of linear
    logic", JSL 62(4), 1997 (cited, `plan/later.md`).
21. M. Okada and K. Terui, "The finite model property for various fragments
    of intuitionistic linear logic", JSL 64(2), 1999 (cited, same).
22. J. Hodas and D. Miller, "Logic programming in a fragment of
    intuitionistic linear logic", Inf. Comput. 110(2), 1994 (not reread).
23. J. Harland and D. Pym, "Resource-distribution via Boolean constraints",
    ACM TOCL 4(1), 2003 (not reread).
24. K. Chaudhuri, D. Miller and A. Saurin, "Canonical sequent proofs via
    multi-focusing", IFIP TCS 2008 (not reread).
25. J.-M. Andreoli, "Logic programming with focusing proofs in linear logic",
    J. Logic Comput. 2(3), 1992, pp. 297–347 (not reread).
