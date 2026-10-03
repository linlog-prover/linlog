# Other provers, and how to compare with them

Researched on 2026-10-03 by a sub-agent of the planning session, from the
tools' pages and repositories and the papers named; what it could not
verify is said. Step 29 checks every line again before anything is
published.

## The tools

| tool | language, licence, last activity | fragments | what it does and outputs | runs in batch today |
|---|---|---|---|---|
| Maude LL prover, Olarte et al. (github.com/carlosolarte/Linear-Logic-Prover-in-Maude), the one whose 2018 runs the LLTP library's status files record | Maude 2.7.1; no licence shown; code 2018, README 2020 | focused ILL (ILLF), focused CLL (LLF), LJ | automatic; proofs, LaTeX | a `maude` batch query; nixpkgs has Maude 3.5.1, whether the 2.7.1 code runs on it is unknown |
| llprover, Tamura (cspsat.gitlab.io/llprover) | Prolog (SICStus; SWI port of 2020); licence not stated | first-order two-sided LL, three contractions per path by default | automatic; cut-free proof, LaTeX, MLL proof nets | SWI-Prolog (nixpkgs) |
| linTAP, Mantel and Otten (leancop.de/lintap) | Prolog, 4 KB; licence not stated; 1999, page 2008 | propositional classical MLL with `?` | automatic; yes or no | SWI-Prolog |
| Click & coLLecT, Callies and Laurent (click-and-collect.linear-logic.org, github.com/ComputerAidedLL/click-and-collect) | OCaml and JavaScript; LGPL-2.1; code 2022 | propositional LL with units and exponentials, cut; Mix, ILL and nets not verified | interactive, and an automatic prover that answers proved, exhausted or timed out (3 s); LaTeX, PDF, PNG, ASCII, Rocq through NanoYalla | a web server with an HTTP API; no package found |
| Sympli, Chaudhuri (github.com/chaudhuri/sympli), after Chaudhuri and Pfenning's focused inverse method (CADE 2005) | SML (MLton); BSD-2; archived 2021 | propositional ILL with `! ⊗ & ⊕ ⊸` | automatic; proof terms | MLton (nixpkgs); whether it is the paper's prover is unverified |
| LinearOne, Moot (github.com/RichardMoot/LinearOne) | SWI-Prolog; LGPL-2.1; active 2026 | first-order MILL | automatic; proof nets, natural deduction, LaTeX | `swipl` |
| Proof Net Calculator, Matsuoka (staff.aist.go.jp/s-matsuoka/PNCalculator) | Scala; licence not stated; 2023 | unit-free MLL nets | net construction, correctness, GoI | JVM; unverified |
| lolli, ibrahimcesar (github.com/ibrahimcesar/lolli) | Rust; MIT; 2025 | MLL, MALL, MELL, focused | automatic, REPL; ASCII, LaTeX, Graphviz | `cargo`; its README says up to about 20 connectives |

Not provers in this sense: Celf, LolliMon and Lolli (logic programming
over fragments of ILL), Grail (categorial parsing), Yalla and the Lean
and Isabelle formalisations (no automation). No other tool's LLTP
results were found; a "timeout" in the library's status files is the
Maude prover's result of 2018, not a status. The ILLTP paper (EPTCS 292,
arXiv 1904.06850) reports that prover's numbers on a 2 GHz virtual CPU
at five minutes.

## How an honest comparison is made

- **Rank by problems solved within the limit**, as CASC
  (tptp.org/CASC) and the SAT competition do; show cactus plots and
  pairwise scatter plots; count wrong answers and show them (Beyer,
  Löwe and Wendler, "Reliable benchmarking", STTT 21, 2019).
- **Measure with BenchExec** (github.com/sosy-lab/benchexec): CPU time,
  wall time and memory per run in cgroups, child processes included,
  used by SV-COMP; `time` and `ulimit` are unreliable.
- **Give every tool its best documented configuration**, and send it to
  the tool's authors before publishing (SV-COMP's results are approved
  by the tools' authors; Raasveldt et al., "Fair Benchmarking
  Considered Difficult", DBTest 2018).
- **Publish what reproduces it**: versions and commits, command lines,
  hardware, limits, the raw rows; state absolute numbers and never
  average ratios (Hoefler and Belli, SC 2015).
- **Count apart** the problems outside a tool's fragment, and say whose
  answers are checked (linlog checks every proof) and how long the
  translation into each tool's syntax takes.

## Where to run it

- GitHub's hosted runners for public repositories: 4 vCPU, 16 GB, 6 h a
  job, 20 jobs at once; runs land on different CPUs (AMD and Intel), with
  a few percent of noise. Counts of problems solved survive that; close
  timings do not. A run is split by problems, each job running every
  tool on its problems, so that each comparison is on one machine.
- Self-hosted runners should almost never serve a public repository
  (GitHub's security guidance).
- A job publishes through an artifact; a bot's commit or pull request
  would not be signed by the author's key, which the repository's
  policy asks of every commit. A repository has one Pages site, which
  the rustdoc holds.
