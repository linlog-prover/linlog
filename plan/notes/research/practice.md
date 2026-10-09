# Problems from practice: the sources, ranked

Written 2026-10-08 from four researchers' findings (planning, concurrency,
programming, other), `plan/later.md` ("Problems from practice"),
`plan/29-comparison.md`, `plan/30-baseline-release.md`,
`.claude/rules/bench.md` and `.claude/rules/core-horn.md`. Where I doubted
a licence, a count or an encoding I checked the source; "verified" marks
that, "(inference)" a conclusion of my own, "unverified" a researcher's
figure I could not check. Every source below is one a researcher found.

## 1. What the evidence lacks

The benchmark's problems from practice are the LLTP library's 3 137 net
problems (theorems by construction) and qcover's 176 coverability
problems, of which only 12 files state an `#expected result`. Blondin,
Haase and Offtermatt (TACAS 2021, §5, verified in the PDF) call the same
176 "61 positive and 115 negative coverability instances", and their
figshare dataset's `coverability/` folder holds the 61 positive ones. So
the harness can get a verdict for every qcover row by name matching,
without a new reader (inference from the two counts; checking that the 61
names are among the 176 settles it). That is the cheapest gain here.

Missing, in order of weight: real non-theorems with known answers;
linear-mode reachability questions on real nets; and small ILL sequents
with additives and `!` from programming, the only practice problems the
other provers of step 29 can run (inference: a net of 2 000 transitions
is outside every tool in `29-comparison.md`'s table except QCover, so
nets compare the Horn engine with Petri tools, not linlog with LL
provers).

## 2. The ranking

| # | source | size | licence | format | encoding, fragment | answers | effort |
|---|---|---|---|---|---|---|---|
| 1 | MCC 2025 via pnmcc-models-2025 | 1 601 P/T instances, 142 models (unverified counts); oracle 2.0 MB | models: "expected to become part of the public domain" (2026 call, verified); results page "CC-BY-NC-SA" for its documents (verified); scripts GPL-3.0 | PNML, formula XML/.txt, oracle lines `FORMULA <id> TRUE/FALSE/?` (verified) | Kanovich 1995 `!`-Horn; EF of an upward-closed set as coverability, AG of a downward-closed set as its negation; affine | yes, consensus per formula, `?` for none | medium: PNML and formula readers, filter, oracle |
| 2 | figshare "Benchmark Instances for Reachability and Coverability" (sypet, random walks) | 30 + 127 instances, 86.3 MB zip, MD5 verified | MIT (verified, API record) | Mist `.spec` with `x = k` targets; LoLA | exact reachability, `!`-Horn, linear mode | yes: all provable | small: `x = k` in `mist.rs`, a mode per source |
| 3 | hand file of programming sequents: Lolli's 26 formulas, Granule's top-level and `hilbert/` goals, SILI's 20, GHC LinearTypes signatures | about 80 to 100 sequents | Granule BSD-3; GHC BSD-style; Lolli and SILI none | text, one per line | direct: ILL with `⊸ ⊗ & ⊕ ! 1 ⊤` | Lolli: 20 to 24 unprovable, 17 has eight proofs, 25 two (verified); others provable; GHC by hand (Linear1.hs's `a ⊸ (a,a)` verified "Must fail") | low, by hand into `bench/problems/` |
| 4 | Eriksson, Unsolvable PDDL Benchmarks | 704 tasks, 20 domains (count unverified), 9.8 MB | CC-BY-4.0 (verified) | PDDL, STRIPS, some `:action-costs`, `:equality` | ground to SAS+, one atom per variable value, `!(pre ⊸ post)`, goal covered; affine `!`-Horn | yes: all unprovable | medium to high: Fast Downward's translator in the flake, a `.sas` reader |
| 5 | Mugdan, Christen, Eriksson ICAPS 2023 | optimal STRIPS subset of downward-benchmarks plus its unsolvability compilation, 45.9 MB (verified) | none stated (verified: "No further description") | PDDL; costs via Downward Lab properties, unit-cost tasks only (verified) | as 4, plus a budget atom: `!(c ⊗ pre ⊸ post)`, `c^B` on the left | yes: `B = C*` provable, `C* − 1` not | medium once 4 exists |
| 6 | downward-benchmarks / potassco / classical-domains | 4 153 / 332 variants / 147 domains | none stated | PDDL | as 4 | mostly provable; 13 unsolvable | low once 4 exists; theorems only |
| 7 | Celf `petri.clf`, Chrpa's blocks world, NDSS 2007 credentials | a handful each | GPL-3.0 / paper / none | CLF, paper | `!`-Horn by hand | yes | low; illustrative |
| 8 | XPP resource-constrained planning | 1 700 PDDL files | MIT | PDDL, fuel levels | as 4 | no | medium |
| 9 | TLGbank Light | 1 599 sentences | none stated | Prolog terms | Lambek; IMLL image | all provable | medium; step 36 |
| 10 | IBM BPM'09 workflow nets; PDC 2021-25 | 1 386 nets; 96 a year | none / CC0 | PNML | soundness as reachability (linear) and coverability (affine) questions | partly / none | blocked: links dead, 4TU in maintenance (2026-10-08) |
| 11 | BioModels | about 800 curated | CC0 | SBML | reactions as `!`-Horn | no questions | medium-high |
| 12 | BPMAI | 29 810 models | CC-BY-3.0 | Signavio JSON | BPMN to net, then as 10 | no | large |
| 13 | PSPLIB | 2 040 single-mode | none stated | `.sm` | time-indexed `!`-Horn, the researcher's own, unproven | yes | high |
| 14 | BPPLIB | thousands | CC BY-NC-ND 4.0 (verified) | text | bins as unit atoms, MALL | yes | intractable in unary |

Dropped without a row: SAP-SAM (no derivative works), Ceptre (licence
not open source, no goals), the BFC archives (in qcover already), Hadara
(synthetic, all positive), LLF, LolliMon, Forum, Rast, SILL, linear-base
and Granule's ESOP 2024 set (recursive or dependent types, no
propositional image), Proof-Carrying Plans (ten tiny tasks the IPC sets
hold), unsolve-ipc-2016 (Eriksson's set is its filtered form), LinGraph
(no files), and the papers of Chrpa et al. and Kanovich and Vauzeilles,
which are citations.

## 3. The encodings, checked

**Planning (rows 4 to 6).** Chrpa, Surynek and Vyskočil 2009, §3.1,
read from the PDF: an action with positive precondition `p(a)`, delete
list `e⁻(a)` and add list `e⁺(a)` becomes `(p₁ ⊗ … ⊗ pₗ ⊗ r₁ ⊗ … ⊗ rₘ) ⊸
(p₁ ⊗ … ⊗ pₗ ⊗ s₁ ⊗ … ⊗ sₙ)` with `pᵢ ∈ p(a) ∖ e⁻(a)`, `rⱼ ∈ e⁻(a)`,
`sₖ ∈ e⁺(a)`, each under `!`, the initial facts on the left and the goal
`g₁ ⊗ … ⊗ g_q ⊗ ⊤`; "the plan exists if and only if the above expression
is provable". §3.4 gives the two conditions under which "the basic
encoding is accurate": `e⁻(a) ⊆ p(a)` for every action, and `p(a) ⊆ s`
implies `s ∩ e⁺(a) = ∅` for every state. The researcher's summary is
right. The SAS+ route is sound where those conditions fail (inference,
agreeing with the researcher): one token among a variable's values is a
place invariant, an operator consumes the old value and produces the new,
a prevail condition consumes and returns it, an effect without a
precondition becomes one clause per old value; no token is duplicated,
every reachable marking is a state, the net is 1-safe. Two cautions.
`⊤` is Chrpa's device for "the state can certainly contain more
predicates", and `Program::read` takes a goal that is a tensor of atoms
and `1` (`core-horn.md`), so these problems run intuitionistic affine
without `⊤`, as qcover does, which `core-horn.md`'s coverability argument
covers. The budget atom of row 5 is sound in the same mode: each clause
consumes one `c` (or `cost(a)` copies), unused budget is weakened, so a
proof exists exactly when a plan of cost at most `B` does (inference).
Only tasks whose `C*` the properties file records give a pair.

**Model Checking Contest (row 1).** A `ReachabilityFireability` atom
`is-fireable(t)` says the preset of `t` is covered, so an EF formula over
a positive Boolean combination of them is coverability of a finite
disjunction of markings, which `mist.rs` already builds from multi-line
targets, one clause per disjunct into one goal atom. `AG ψ` with `ψ`
downward-closed is `¬EF ¬ψ`, so the sequent is provable exactly when the
oracle says FALSE, as both researchers say. One caution they did not
state: a `ReachabilityCardinality` bound on a sum, `tokens-count(p₁) +
tokens-count(p₂) ≥ k`, is upward-closed but has `k + 1` minimal
markings, so the disjunction grows combinatorially with the summed
places; admit single-place bounds first and sums below a cap
(inference). The 25 % and 18 % shares come from one 45-instance sample,
unverified. The oracle is a consensus "pondered by confidence rate", not
a proof: by step 29's method (choice 5) a row where linlog's checked
proof contradicts it is a finding to report, not a `MISMATCH` to fix.
Terms, verified: the results page's CC-BY-NC-SA covers "these documents"
of that page; the call for models says models "are expected to become
part of the public domain" and "If your model is proprietary, do not
submit it". So fetch at a pinned year, never vendor, and name the terms
in `bench/RESULTS.md`.

**Exact reachability (row 2).** Verified against the paper: the sypet
suite is "30 positive (standard) reachability instances arising from
queries encountered in type-directed program synthesis", 65 to 1 199
places and 537 to 8 340 transitions; the random walks are 127 instances
on the largest quarter of the coverability nets, 33 in all, with every
instance FastForward or LoLA solved with a witness of at most 20
removed, and an upward-closed initial marking replaced by 1 to 20 % of
the walk's length in tokens. Within 60 s
FastForward (GBFS) decided 27 of 30 and 123 of 127, LoLA 8 of 30 and 2 of
127: published numbers of two Petri tools on the same files for step 29.
The sequent is `M₀, !(rules) ⊢ ⊗ M` in linear mode, Kanovich 1995, which
`core-horn.md`'s firing-sequence argument covers. Caution (inference): a
target that mixes `=` and `≥`, or names only some places, is neither
exact nor a cover, so the reader should refuse it unless Mist's meaning
of an unnamed place is checked and found to be zero. SyPet's clone
transitions `A ⊸ A ⊗ A` make the state space infinite, so only the
positive answer comes by search, which suits a set that is all theorems.

**Programming (row 3).** Lolli's `examples.pro` verified: 26 formulas,
`=>` for `!A ⊸ B`, and the comment that 20 to 24 are not provable, 17
has eight proofs and 25 two. GHC's `Linear1.hs` verified: `incorrectDup
:: a ⊸ (a,a)` and `incorrectDrop :: a ⊸ ()`, each "Must fail", the
sequents `a ⊢ a ⊗ a` and `a ⊢ 1`, unprovable in linear mode, the second
provable in affine mode. The Granule encoding (`A [n]` as `n` copies, `[0..1]` as
`A & 1`, `[0..∞]` as `!A`) is the researcher's; `A & 1` is the standard
affine reading, but each grade should be checked against Granule's
semiring before the file is written (inference). Vendor only Granule's
and GHC's sequents, with their notices; fetch Lolli's and SILI's at a
pinned URL or ask the authors, since their files state no licence.

**BPPLIB (row 14).** CC BY-NC-ND 4.0 (verified), so translated files
cannot be committed; the unit-atom encoding needs affine mode because
bins need not be full (inference) and is intractable in unary at
capacities up to 100 000. Drop.

## 4. Adopt, defer, drop

**For step 29 (the comparison).** Adopt rows 2 and 3 and the qcover
verdicts of section 1. Both take hours, not days; row 3 is the one
practice set every column can run and holds real non-theorems (Lolli's
five, GHC's); row 2 has published numbers of two Petri tools on the same
files and is the only large set of hard linear-mode questions on nets of
real programs. They add at most 157 large rows (Horn engine only) and
about 100 small ones (every tool) to the night's arithmetic.

**For step 30 (the release baseline).** Adopt row 1, the MCC
coverability subset, if its readers are built before the baseline night,
as a pass of its own beside `horn-qcover.csv` with a short time limit, so
the third baseline stays comparable with the second under the second's
flags (`30-baseline-release.md`, item 1). It is the largest gain in
evidence: thousands of real questions with real FALSE answers, what
`plan/later.md` deferred as "nets beyond the 76 LLTP used". If the
readers are not ready, the release ships without it; the baseline's
eleven and a half hours must not grow for it.

**Defer, with the reason.** Rows 4 and 5 (planning): 704 non-theorems
from practice and matched pairs from one task are a real gain, but the
route runs a GPL translator from the flake, grounding blows up on
`diagnosis` and `3unsat`, many rows will time out, and the comparison
does not need them; after the release, as one step, Eriksson first. Row
6 adds theorems only once row 4 exists. Row 7 can join the row 3 file at
any time. Row 8 has no verdicts; row 9 waits for step 36's Lambek
engine; row 10 is blocked by dead links and a maintenance window, and
the author may write to Lohmann, Wolf or Fahland for the BPM'09 nets;
rows 11 and 12 need an oracle tool or a BPMN translation, a project of
their own.

**Drop.** Rows 13 and 14 and everything under the table: a restrictive
licence, no answers, no propositional image, or a duplicate.

## 5. What the library and the harness need

**Nothing new in the core's engines.** The Horn engine decides
linear-mode reachability by exhausting markings and the state equation,
and affine-mode coverability by backward search (`core-horn.md`), which
is what rows 1, 2, 4 and 5 ask. `mist::read` needs `x = k` in a target
and a way to say the problem is exact; `read_within`'s occurrence limit
already refuses problems too large to build, which the harness should
count as skipped rows, not errors.

**Readers.** For row 1: a P/T PNML reader (places, transitions, weighted
arcs, initial marking; coloured nets refused), a reader of the formula
XML for the two reachability examinations, a monotonicity filter with a
DNF cap, the oracle's line format and the inversion of AG verdicts. A
PNML reader serves a web front end too, so a `pnml` feature of the core
beside `parse`, with `quick-xml` scoped to it, is the D15-consistent
place (inference); the rest belongs in `bench/src/`. For row 2: the
`.spec` change above. For rows 4 and 5: a flake package building Fast
Downward at a pinned commit and running its translator at build time
(called, never linked, as the LLTP library is fetched), a `.sas` reader
in `bench/src/`, the one-clause-per-old-value expansion, and for row 5
the budget atom from the properties file.

**The harness.** `problems.rs` fixes the mode per source (`.spec` is
intuitionistic affine); the new sources need a mode per `Reference`,
since figshare's `.spec` files are linear-mode questions, and an expected
verdict from beside the problem (figshare: provable by folder; MCC: the
oracle; planning: unprovable by set). `modules/bench.nix` gains pinned
fetches, by MD5 for the figshare zip and by instance URL for the MCC, so
a subset can be pinned rather than the 1 GB archive. The CSV's columns
stay, since they are the interface (`bench.md`); `summary`'s `MISMATCH`
is a finding rather than a bug where the oracle is a consensus. Row 3 is
a problem file in the existing `name; mode; expected; copies; sequent`
format, each line's source and licence in a comment. In step 29's
report, rows 1 and 2 enter linlog's and QCover's columns only, and row 3
is what the translators to the LL provers' syntaxes exercise.
