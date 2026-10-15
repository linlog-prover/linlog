# linlog beside the other provers

What each prover of linear logic we could find can do, and what each of
those that run in batch decides on the same problems under the same
limits. Every cell of the matrix was read from the tool's own source or
page on 2026-10-15; the numbers come from one night on one machine, and
`bench/comparison.sh` reproduces them on any machine with Nix.

## The tools

| | linlog | Maude LL prover | llprover | linTAP | Click & coLLecT | Sympli | LinearOne | lolli | APLL | Proof Net Calculator |
|---|---|---|---|---|---|---|---|---|---|---|
| authors | this repository | Olarte et al. | Tamura | Mantel, Otten | Callies, Laurent | Chaudhuri | Moot | ibrahimcesar | Wu | Matsuoka |
| fragments | MLL to full LL, propositional, units | full LL and full ILL, propositional, units | full LL, first order | MLL with units and `?` | full LL, propositional, units | ILL: `! ⊗ & ⊕ ⊸ 1 0 ⊤` | MILL (`⊗ ⊸`), first order | MLL, MALL, MELL with units | full LL and full ILL, propositional, units | unit-free MLL |
| modes | classical, intuitionistic, affine, Mix | classical, intuitionistic | classical, intuitionistic | classical | classical | intuitionistic | intuitionistic | classical | classical, intuitionistic | — |
| automatic | ✓ | ✓ | ✓ | ✓ | ✓ (and interactive) | ✓ | ✓ | ✓ (and a REPL) | ✓ (backward search or inverse method) | ✗ (net checks and enumeration) |
| interactive | ✓ (`linlog interact`) | ✗ | ✗ | ✗ | ✓ (web) | ✗ | ✗ | REPL only | ✗ | GUI for drawing nets |
| decides unprovability | ✓ (an exhaustive search or a refutation) | without `!`; with `!` up to a copy bound | without exponentials; with them up to a threshold | without `?` | ✓ when the search is exhaustive | ✓ (saturation) | ✓ | ✗ (a depth limit) | ✓ unless its bound of contractions is met | — |
| proofs output | ✓ (terms, derivations) | ✓ (LaTeX tree, ILL) | ✓ (ASCII, LaTeX, Prolog term) | ✗ (yes or no) | ✓ | ✓ (proof terms) | ✓ (λ-terms, LaTeX) | ✓ (tree, λ-term) | ✓ (its proof trees, LaTeX) | nets |
| proof nets | ✓ (MLL, search and criterion) | ✗ | ✓ (an output style) | ✗ | ✗ | ✗ | ✓ (TikZ) | ✗ | ✗ | ✓ |
| checker | ✓ (independent, every proof) | ✗ | ✗ | ✗ | unverified | ✓ (its own validation) | unverified | ✗ | ✗ | ✓ (correctness criteria) |
| certificates | ✓ (Rocq, NanoYalla) | ✗ | ✗ | ✗ | ✓ (Coq, NanoYalla) | ✗ | ✗ | ✗ | ✓ (Coq, Yalla) | ✗ |
| exports | LaTeX, Typst, SVG, PNG, PDF, Rocq, JSON | LaTeX | LaTeX | — | LaTeX, PDF, PNG, ASCII, Coq | — | LaTeX, TikZ | LaTeX, DOT, SVG | LaTeX, Coq | — |
| limits | time, memory, copies, recursion, work | copy bound | threshold of contractions | — | 3 s of CPU (fixed) | — | — | depth | bound of contractions | — |
| threads | one, or a pool (`--jobs`) | one | one | one | one | one | one | one | one | unverified |
| input formats | text, JSON, LLTP, `.spec`, TPTP, batch files | Maude terms (a sed script for LLTP) | its sequents | Prolog terms | its sequents | its scripts | Prolog terms | its sequents | its sequents, LLTP | its formulas |
| licence | EUPL-1.2 | none stated | none formal ("freely distribute or modify") | GPL, no version | LGPL-2.1 | BSD-2-Clause | LGPL-2.1 | MIT | none stated | none stated |
| last activity | 2026-10 | 2020-11-03 | 2020-12-30 (SWI port) | 2012-03-30 (file) | 2025-04-07 | 2018-07-27 | 2026-06-11 | 2025-12-05 | 2019-04-29 | 2023-09-21 (page) |
| runs here in batch | ✓ | ✓ (Maude 3.5.1) | ✓ (SWI-Prolog 10) | ✓ (a driver of ours) | ✓ (a driver of ours) | ✓ (a patch for MLton) | ✓ (a driver of ours) | ✓ | ✓ (OCaml 4.14, native) | ✗ |

Sources, all read on 2026-10-15:

- **linlog**: README, "What exists and what is planned"; the crate's
  documentation, "Features", and the dispatch table on `Engine`; `linlog
  prove --help` for the limits and input formats.
- **Maude LL prover**: github.com/carlosolarte/Linear-Logic-Prover-in-Maude
  at `5e1da7f` (its README, `maude-prover/*.maude`, and
  `utils/exec-ILLTP-flat-file.sh`, the script of its LLTP runs: `solve(5,
  …)`); the LLTP library's status files are its runs of 2018.
- **llprover**: cspsat.gitlab.io/llprover, `llprover.pl` (version 2.0,
  the SWI port of 2020-12-30, lines 5, 57–63, 120–122, 381) and
  `guide.pdf` (sections 1, 3 and 4).
- **linTAP**: leancop.de/lintap and `programs/lintap.pl` (version 1.08e,
  its header and `pp/2`); its `prove/1` raises the exponential limit for
  ever on a formula it cannot prove, so the driver of this repository
  replays its search under each limit in turn.
- **Click & coLLecT**: github.com/ComputerAidedLL/click-and-collect at
  `6b1c25f` (`LICENSE`, `ll_lexer.mll`, `ll_parser.mly`,
  `focused_proof.ml` lines 470–484, `export_as_coq.ml`, `main.ml`).
- **Sympli**: github.com/chaudhuri/sympli at `50e76a6` (`LICENSE`,
  README, `Engine2_Focusing`, `Proof.test`).
- **LinearOne**: github.com/RichardMoot/LinearOne at `d9379a5` (`LICENSE`,
  `mill1.pl` lines 672–709, `first_proof/3`); whether it checks its
  proofs is unverified.
- **lolli**: github.com/ibrahimcesar/lolli at `cb09efc` (`LICENSE`,
  README, which calls it "not suitable for batch processing", and the
  command's `prove --depth`).
- **APLL**: github.com/wujuihsuan2016/LL_prover at `c55f59d` (its
  README, which the LLTP library's README names beside the Maude prover,
  and `src/prover.ml`, `foc_ill_bwd.ml` lines 307–320: `N` only when the
  bound was never met); no licence file.
- **Proof Net Calculator**: staff.aist.go.jp/s-matsuoka/PNCalculator
  (page of 2023-09-21) and `ProofNetCalculator-Ver0.0.12.zip`
  (2021-07-04): a Scala library and GUI that checks, transforms and
  enumerates unit-free MLL nets, no prover of a given sequent, built with
  sbt from the network. It is no column of the tables.

Not provers in this sense: Celf, LolliMon and Lolli (logic programming
over fragments of ILL), Grail (categorial parsing), Yalla and the Lean
and Isabelle formalisations (no automation).

## How the comparison is made

- **The problems**: the LLTP library's intuitionistic problems (`ILL/`,
  4 495 of them, 3 137 of them Petri nets) and classical ones (`CLL/`,
  17), qcover's 176 coverability problems (intuitionistic affine) and
  the generated families of `linlog-bench families` at their default
  sizes, with their known verdicts where the source states one.
- **Every tool runs every problem of its fragment and mode**, translated
  from linlog's reading of the problem into the tool's own syntax by
  `bench/src/translate.rs`, which prints through the library's public
  walk over a formula (atoms renamed `p0`, `p1`, …, every compound
  formula in parentheses). A problem outside a tool's fragment or mode
  is counted apart and not run; no tool but linlog has an affine or a
  Mix mode, so the coverability problems and the Mix family are
  linlog's alone, and a classical tool never gets an intuitionistic
  problem. linlog reads the LLTP and `.spec` files itself. The
  translation's time is stated per tool and is not part of a run.
- **Every tool in its own documented configuration**: APLL's backward
  search under its default bound of 3 contractions, the Maude prover's
  ILL search with the copy bound 5 of its own LLTP script, llprover's
  default threshold 5, lolli's default depth 100, linTAP's search with
  its exponential limit raised without end as its `prove/1` does, Click &
  coLLecT's prover as its web server calls it with its limit of CPU time
  set to the run's, Sympli's inverse method on `(⊗Γ) ⊸ A`. A tool's
  bounded failure is `unknown`, never `unprovable`.
- **linlog in two columns**: `linlog`, the command on one thread
  (`--deterministic`), the main column, since every other tool is
  sequential; and `linlog-default`, the command as a user runs it (one
  thread, then a pool beside it), on four cores. Those four are the
  machine's low-power cluster (3.7 GHz, no share of the last-level
  cache), slower than the efficiency cores (4.0 GHz) every sequential
  tool and linlog's own one-thread column ran on, so the default column
  is measured on slower cores than the others.
- **Measured with BenchExec** (`runexec` 3.27, cgroups v2): each run is a
  process of its own under the same limits, wall time of the whole
  process (its start, its parse and, for linlog, the check of its proof
  included), CPU time and the cgroup's peak memory. linlog gets
  `--timeout` and `--memory-limit` equal to the limits BenchExec
  enforces; a run the cgroup ends for memory is that tool's, linlog's
  included.
- **The time limit, 10 s, is what one night allows** for every tool on
  every problem, and it disadvantages a tool that answers late: the
  tables list every answer that came after it, the kill a second later
  bounds what that list can show, and the Maude prover's own results of
  2018 in the LLTP library stand beside tonight's as a reference.
- **The ranking is the number of problems solved within the limit**, a
  verdict of proved or unprovable in time, as CASC and the SAT
  competition rank; times are in the figures, never averaged into
  ratios.
- **Contradictions are listed**, between two tools or against a known
  verdict, each verdict marked checked or not: linlog checks every proof
  it reports with a checker that shares no code with its search, Sympli
  validates its own; no `unprovable` is checked, linlog's included.

Reproduce, on any Linux machine with Nix and a systemd user session:

```sh
bench/comparison.sh --out comparison --cores 0,1,2,3 --default-cores 4-7 --timeout 10
```

The CI workflow `Comparison` runs the same script on GitHub's runners in
ten shards (`workflow_dispatch`) and uploads the rows, tables and figures.

## Results

The night's run has not been published yet.
