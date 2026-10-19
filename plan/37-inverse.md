# Step 37: the focused inverse method

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing. The step takes two
sessions; this prompt is finished at the review of step 36, from the
third baseline's rows. It says what is fixed.

Read `plan/later.md` ("The focused inverse method"),
`proof-search-specifications.md` ("Alternative: focused inverse method"),
`plan/reports/17-assessment.md` (3.6), `26-focused-engine.md`,
`27-horn.md`, `30-baseline-release.md`, and `plan/README.md` (D8, D19).

## Goal

The spec's second engine for MALL and its semi-decision alternative with
exponentials: forward saturation from initial sequents in the subformula
closure, with subsumption and an index, as an engine behind the
interface of step 26, and a row of the dispatch wherever it is the
fastest (D19).

## Fixed now

The honest starting point (step 17): after steps 15, 26 and 27 two cases
are left for it, sequents with many hypotheses and a small goal that the
backward search still loses on, and whatever of Mix the prune of step 26
does not take. Its first session ends with a measurement on the third
baseline's undecided rows and the families; if no row is won, the engine
stays an option (`--engine inverse`) and the report says so.
Opus 5.5 at `high`; every change of a search or a criterion is reviewed by
the panel step 26's prompt describes, its argument read on Fable 5.1 at
`high`.

**Aimed by the comparison's gap study** (R4 of
`plan/notes/research/comparison-gaps.md`, adopted by the author on
2026-10-18). The premise above that the library ends at the copy bound
is out of date: outside the nets it ends at the time limit (the note's
2.4). Step 30b closed the levels of its class A (R1), which decided
class B's rows too (the comparison of 2026-10-18). Step 39 measured
R3's cap and built nothing (`plan/reports/39-top-cap.md`): not one of
the held-out sets' 129 undecided rows, of the six undecided rows among
the seven refutations Sympli has and linlog lacks, or of the families'
18 undecided rows meets a stable sequent the cap would change (4.5
billion stable sequents searched), so the undecided rows are not a
`⊤`'s copies but long proofs and refutations past the time limit. This
engine's case is class C's long proofs (in that comparison Sympli's ten
proofs that linlog lacks, `ILLTP-SYJ-cbn/SYJ203+1.011` and
`SYJ205+1.012` to `.020`) and a second refuter by saturation for what
R1 leaves of A and B, the six refutations among them. Its first
measurement is on the
held-out sets and the 384 intuitionistic rows no tool decides, at the
longer limit step 29 proposed, never on the 122 rows Sympli wins alone.

## What steps 31 to 36 and 39 left you

- **The engines and the dispatch** (`search::Engine`, `search::DISPATCH`,
  documented on `Engine` with the measurement behind each row): `focus`
  and `two-sided` for every classical and intuitionistic fragment,
  `additive`, `horn` (Horn programs, reachability and coverability),
  `net` (taking a goal over only at the focused engines' recursion
  limit or memory bound, step 35a), `essential` (forced only), and the
  planar search of the ordered modes (cyclic MLL and the Lambek
  calculus, step 36), which every other engine refuses
  (`NotTaken::Mode`). The inverse method refuses the ordered modes too.
- **What already answers an undecided search**: the classical refuter
  runs after every search that does not prove its goal (step 31d,
  `Options::classical_work`), so the rows still undecided are those
  whose classical reading holds. A refutation by saturation is a new
  kind of `Refutation`; `Disproof::check` verifies the classical,
  unbalanced and count-equation ones today, so a saturation refutation
  either gets a check of its own (the saturated set closed under the
  rules, checked without the engine's code) or is reported as exhausted
  without one, said so in the report.
- **What the comparison left** (the author's run 39110897772, linlog
  0.1.0, 10 s, one thread): linlog decides 4 307 of 4 811 problems;
  Sympli decides 17 that linlog does not, ten proofs of class C
  (`ILLTP-SYJ-cbn/SYJ203+1.011`, `SYJ205+1.012` to `.020`) and seven
  refutations, of which step 39's run refuted `ILLTP-SYJ-01/SYJ210+1.007`
  at 5.4 s, so six remain (`SYJ208+1.004` in `-01` and `-cbn`,
  `SYJ209+1.011` and `.012` in `-cbn`, `.008` and `.009` in `-cbv`).
  The longer limit is 60 s (`plan/reports/29-comparison.md`, "A second
  night at a longer limit").
- **The oracles**: `bench/targets/after-cyclic.csv` and
  `bench/targets/net-after-cyclic.csv`, equal to the earlier oracles on
  every row `bench/TARGETS.md` does not record; `mix/11` ends at the
  time limit, so its counters differ between runs of one binary and are
  compared at equal work (36b's report). The engine leaves every
  existing engine's counters as they are; a dispatch row it takes
  changes only that row's verdicts and counters, each said.
- **Every engine polls by work and charges its tables** (steps 19, 20,
  30b): the stop counts units (the web page has no clock and stops a
  search by its work budget, step 32), and the memory account refuses
  rather than crashes; the inverse method's database and index are
  charged to it from the first entry.
- **Quantifiers come next** (step 38, D17): the forward method extends
  to first-order logic by unification on the initial sequents; keep the
  place for terms in the database's keys and say where it goes.

## How the step runs

Two sessions on Opus 5.5 at `high` (plan/README.md, "Why these models
and efforts", 2026-10-18), each with its goal in its launch prompt, and
the panel of step 26's prompt for every change of a search or a
criterion: the argument on Fable 5.1 at `high`, the counterexamples and
the integers and limits on Opus 5.5 and Sonnet 5.5 at `high`, each on
a frozen tree.

- **37a**: the engine for MALL, and its semi-decision with
  exponentials, behind the engine interface as `--engine inverse`, its
  proofs reconstructed as terms the checker accepts, a differential test
  against the focused engine and the reference prover, and the first
  measurement: the held-out sets and the 384 intuitionistic rows no tool
  decides at 60 s, Sympli's ten proofs of class C and the six
  refutations, the families, on cores 2 and 3.
- **37b**, only if 37a's measurement wins a row: the dispatch rows
  where it is fastest (D19), measured as step 35a measured its routing,
  and the saturation refuter's certificate if it refutes what the
  classical refuter does not. If no row is won, the step ends after 37a
  with the engine as an option.

## The machine

Step 40 (the net drawings, the export only) may run beside this one on
cores 8 to 15. This step's timed runs take cores 2 and 3 and its
builds 4 to 7; every timed run and every `nix` build holds
`/tmp/linlog-machine.lock`.

Deliverable: thematic jj commits; `plan/reports/37-inverse.md`.
