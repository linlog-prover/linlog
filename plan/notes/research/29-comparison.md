# Research for step 29: linlog beside the other provers

Written on 2026-10-08 for `plan/29-comparison.md`. It checks again and extends
`plan/notes/comparison.md` (2026-10-03). Sources are numbered in section 5.
"(inference)" marks a conclusion of my own that no source states.
"Unverified" means I looked for the fact and did not find it.

## 1. The problem and the state of the art

**The question each tool answers.** Given a propositional sequent of
classical or intuitionistic linear logic (CLL, ILL), is it provable? The
usual fragments are MLL, MALL, MELL and LL, with affine and Mix variants.
Full propositional LL is undecidable, and MALL is PSPACE-complete [26].
For MELL, Lazić and Schmitz prove non-elementary lower bounds [28]; whether
MELL is decidable at all is open. Intuitionistic propositional logic is
PSPACE-complete [29]. So a prover answers with a proof, a refutation, or
"unknown" at a limit, and a comparison has to count all three. Focusing [27]
underlies most automatic LL provers: the focused backward search (Maude,
APLL, linlog) and the focused inverse method [10] (Sympli, APLL `-inv`).

**The benchmark sets.** LLTP [2] (GPL-3.0; the flake pins commit `e0394fb`
of 2025-01-27, which is still the latest) has CLL and ILL problems in a
TPTP-like `fof` syntax. Its ILL part comes from Kleene's theorems, from
ILTP's propositional problems under three translations, and from Petri
nets of the Model Checking Contest [1, 2]. The ILLTP paper ran a single
prover, the authors' own "very basic", "prototypical" focused prover in
Maude, on a QEMU virtual CPU at 2 GHz with a 5-minute limit [1]. It lists
seven other provers but says "there has been no discussion about the
efficiency or adequacy of these provers", and it ran none of them [1]. I
found no published results of any other tool on LLTP. ILTP [20] records
propositional results for seven intuitionistic provers. The qcover suite
of 176 coverability problems, which step 27 added, has results for four
tools [23].

**linlog's own numbers to date** (repository reports [R]). On the 1 342
problems that the Maude result files cover, the Maude prover decides 659
at 300 s. linlog's default decides 822 at 5 s on two cores. The two
deciding sets share 657 problems, with no verdict between them that differs
(`plan/reports/21-defaults.md`). On ILTP's 274 problems at 2 s,
linlog decides 108 intuitionistically, with its three translations taken
together, and 156 classically (`25-ordinary-logic.md`). On qcover at 5 s
the Horn engine decides 162 (`27-horn.md`).

**Existing implementations**

| tool | language, licence | last activity | fragment, method | input; batch | published results |
|---|---|---|---|---|---|
| Maude LL prover [3] | Maude (tested on 2.7.1); no licence file | 2020-11-03 | focused ILL (ILLF), CLL (LLF), LJ | `search` query in Maude; `utils/` holds scripts for LLTP files | ILLTP Table 1; LLTP status files [1, 2] |
| APLL (repo `LL_prover`) [6, 7] | OCaml ≥ 4.06 with Coq 8.8; no licence file | 2019-04-29 | LL and ILL (`-ill`); backward search, or the inverse method (`-inv`); `-bound` | `-lltp` reads LLTP files directly; `-c` writes Coq certificates for Yalla | a handful of tests in [7]; none on LLTP |
| llprover [4] | Prolog (SICStus, SWI port of 2020-12-19); "You can freely distribute or modify this program" | v2.0, 2020 | first-order two-sided LL; systems `ill`, `ilz`, `cll`; iterative deepening on cut and contraction up to a threshold (5 in the source; the page says three) | `batch` predicate reads a sequent from stdin; `*` tensor, `+` par, `/\` with, `\/` plus | none found |
| linTAP [5] | Prolog (SWI, ECLiPSe, SICStus); no licence stated | page 2008, paper 1999 | MLL with `?` and `!` ("M?LL"), prefixed tableaux | `prove(F).`; `@` par | some in the 1999 paper; none on LLTP |
| Sympli [9] | Standard ML (MLton ≥ 20130715); BSD-2-Clause | archived 2021-09-09, last commit 2018-07-27 | propositional linear inverse method (`*`, `-o`, `&`, `+`, `!`) | `%prove` / `%refute` directives | none found; whether it is the prover of [10] is unverified |
| Click & coLLecT [8] | OCaml 4.12, web front end; LGPL-2.1 | 2025-04-07 | LL with units, exponentials and cut; interactive, plus an auto-prover that answers proved, not provable, or time-out | HTTP server (`auto_prove_sequent.ml`); exports Coq through NanoYalla, and LaTeX | none found |
| lolli (Rust) [11] | Rust; MIT | v0.2.0, 2025-12-05 | MLL, MALL, MELL, focused; "~10-20 connectives" | CLI `prove … --depth` | none ("benchmarks" planned) |
| LinearOne [12] | SWI-Prolog; LGPL-2.1 | 2026-06-11 | first-order MILL, as a grammar parser | Prolog session | none |
| linear-logic [13] | Haskell; GPL-3.0 | 2014-09-03 | not stated | not stated | none |
| intuitR [22] | Haskell; BSD-3-Clause (intuitRGC: GPL-3.0) | 2022-07-22 | intuitionistic propositional, SAT-based (a restart added to intuit [21]) | own syntax and TPTP | its paper: beats intuit and others on a standard suite [22] |
| ILTP's provers [20] | various (PITP C, STRIP, ft, LJT) | page 2017 | intuitionistic propositional | TPTP | PITPINV 262, PITP 238, STRIP 205, ft-C 199, ft-Prolog 188, LJT 175 of 274 |
| QCover [23] | Python 2.7, NumPy, SciPy, Z3; Apache-2.0 | 2021-03-03 | coverability: continuous Petri nets with SMT | Mist `.spec` | on the 176 problems at 2 000 s: QCover 142, bfc 122, Petrinizer 95, mist 74 [23] |

**Not batch provers** (rows of the matrix at most): Lolli [14] and Celf
[15], which are logic programming languages; Yalla [16] (LGPL-3.0, a deep
embedding in Rocq with no search) and coq-ll [17] (GPL-3.0, focused LL
mechanized in Coq); Isabelle's `Sequents` ILL [18], made for interactive
search; a certified ILL solver in Agda [19], whose page names no authors;
and Chaudhuri's first-order focused inverse-method prover [10], for which I
found no code. Alcove, which [1] lists, did not resolve
(`cic.puj.edu.co`, 2026-10-08). Matsuoka's Proof Net Calculator is in the
note of 2026-10-03, not re-checked. For mist, bfc and Petrinizer [24, 25] I
found papers and a tool-database entry (mist2: "free of charge", last
updated 2010) but no repository I could verify.

## 2. What the step needs

**The pipeline.** It runs: the set, then per tool a fragment filter, a
translation, the run under one measuring tool, the verdict read from the
output, the CSV, then tables, plots and contradictions. Translation is linear
in the unfolded formula. The run costs P problems × K tools × T seconds in
the worst case, and a weak tool's row is nearly all worst case.

**Budget arithmetic** (inference). The night is 11 h on the four
performance cores, about 158 000 core-seconds. With seven columns (five
external tools, linlog on one thread, linlog's default):
- the 1 342 problems with Maude results, at T = 10 s, cost at most
  94 000 s;
- the whole LLTP library (9 007 problems) at 10 s would cost 630 000 s,
  four nights.

The fragment filters shrink this: linTAP takes no additives, and Sympli
takes ILL only.

The choices, each with a recommendation:

1. **Which tools become columns.** Maude prover, APLL, llprover, linTAP,
   Sympli and lolli (Rust) for LL. Click & coLLecT's auto-prover if a
   driver can call it without the web server. intuitR for ILTP, and QCover
   for qcover. The rest are rows. *Why:* these are the ones that run
   unattended on a sequent and have code I could find.
2. **Where each tool gets its input.** Use a tool's own LLTP reader where
   it has one (APLL `-lltp`, the Maude scripts in `utils/`), since that is
   its best documented configuration. Otherwise print linlog's parsed
   `Sequent`, or for ILL its `Reading`, in the tool's syntax. *Why:* one
   parser and one set of names, with a translator per tool and no second
   LLTP parser. Validate every translator on the families, whose verdicts
   are known by construction, and on the 1 342 problems Maude recorded. Time
   the translation and report it apart.
3. **Measurement.** Use BenchExec [30] (Apache-2.0; cgroups; CPU time, wall
   time, memory) for every
   tool, linlog included, with the process measured whole: start-up and
   parse included. Give each run one pinned performance core, set a CPU-time
   limit, and use the same memory limit for all. *Why:* the harness's
   `time_ms` is the search alone (`.claude/rules/bench.md`), and that is not
   comparable with another tool's process time. For linlog's default (a
   pool), declare the cores and rank by wall time in its own column.
4. **What counts as a verdict.** Count three outcomes per tool: proved,
   refuted, unknown. A bounded search that finds no proof (llprover's
   threshold, APLL's `-bound`, lolli's `--depth`) is unknown, unless the
   tool's documentation says the bound is complete for the fragment
   (inference from [4, 6, 11]). Rank by solved within the limit, list wrong
   answers apart [30], and draw cactus plots and pairwise scatter plots
   from the CSV.
5. **Ground truth.** LLTP's headers are not the truth: 28 files contradict
   their problems (`.claude/rules/bench.md`). Decide a contradiction by a
   checked proof, whether linlog's or one checked through another tool's
   certificate (APLL's Yalla output, Click & coLLecT's NanoYalla), or by a
   countermodel, and list the rest open.
6. **Fairness of configuration.** Give each tool its documented best
   settings. Send them to the tools' authors before publishing [31], and
   give absolute numbers, never mean ratios [32].
7. **CI.** GitHub's standard Linux runner for public repositories has 4
   CPUs and 16 GB [33]. The job limit and the concurrency were not stated
   on the page I read. Split the work by problems, so that every tool runs
   on the same machine within a job; counts survive the noise, close
   times do not (inference).

## 3. What linlog's library must offer for it

**Data model.** `Sequent` (`core/src/sequents/mod.rs`) is an arena of
`Term` (`sequents/term.rs`: 12 variants, NNF, no `⊸`, no quantifiers),
`roots` and `atoms`. Its fields are `pub(crate)`, `from_parts` is
`pub(crate)`, and the public surface is read-only accessors.
- *In the way:* the non-recursive formula walk `sequents::fmt::Walk`
  (with `Visit` and `Step`) is `pub(crate)`. A per-tool printer in
  `bench/` can only recurse over `Sequent::term`, which the crate's rule
  forbids for deep inputs, or rebuild a stack of its own. *Needed:* a
  public walk, for example `Formula::stops()` yielding
  `Enter(Kind)/Between/Exit`, with room for a binder stop (D17).
- *In the way:* `lltp::Problem` (`core/src/lltp.rs`) keeps `sequent` and
  `status` only. `optimize` sorts the roots, so which root was an axiom
  and which the conjecture is lost. For ILL the public `Reading`
  (`occurrences/reading.rs`: `goal`, `hypotheses`, `implication`,
  `position`) recovers the two sides, but `IllFormula` prints only
  linlog's syntax. *Needed:* a public ILL walk over a `Reading`, with
  stops for `⊸ ⊗ & ⊕ ! 1 0 ⊤` and atoms, so that a two-sided tool gets
  `Γ ⊢ C` as written. Optionally also the clause roles on `lltp::Problem`.
- Atom names contain `lltp::HYPHEN` (`‿`) and `lltp::DOT` (`·`). Prolog
  and Maude want ASCII, so each printer renames atoms through
  `Sequent::atom_names()`. No library change is needed.
- `Fragment` (`fragment.rs`) has five flags. A tool's admission needs finer
  ones: `⊤` apart from `0`, `!` apart from `?`, `1` apart from `⊥`
  (inference from [5, 9]). A pass over `terms()` with the public `Kind`
  does this. A `Sequent::kinds()` set would make it one call.

**Proof term and checker.** `proofs::Node` (16 bytes, 13 rules),
`Proof::check(mode)` and `check_within(mode, memory)` (`proofs/check.rs`);
every proof that `prove_goal` returns is checked (`Options::check`). In
the comparison linlog runs with its check on, since "every proof checked"
is a claim of the matrix. *In the way:* in the second baseline the
child's check of 14 large nets filled 12 GiB (`.claude/rules/bench.md`, "A
`crash` row"). Under BenchExec that is a linlog failure. The step must
confirm that `check_within` holds those nets inside the comparison's
memory limit. Certificates: `export::rocq` writes NanoYalla, but
`Unsupported` refuses Mix, affine weakening and compact derivations, so
the matrix says "certificate for linear, Mix-free proofs". An ILL proof
certified this way is certified as a one-sided CLL proof only
(inference).

**Engine interface and dispatch.** The crate-private trait `Decide`
(`search/mod.rs`) and `DISPATCH` (5 rows: additive, net, Horn, two-sided,
focus). Public are `Engine` (whose `Display` is its name), `engine_for` and
`Outcome::engine`. That is enough to report which engine decided each
row. *Missing, minor:* `Engine` does not give its fragments and modes as
data, so the matrix's "fragments" row has to be written by hand.

**JSON wire forms.** `Outcome` serializes `verdict`, `reason`,
`refutation` (`Unbalanced`, `Equation`, `StateEquation`, or `Exhausted`),
`fragment`, `mode`, `engine`, `statistics`, and the proof flattened
(`serialize/search.rs`). *Missing:* `search::Options` has no serde (it
derives only `Clone, Debug, PartialEq, Eq`), and no output records the
crate version or the options a verdict ran under. The page has to show
"the configuration each tool ran in" (step goal). *Needed:* serde on
`Options`, as D15 asks of every options value, and a version field or a
CLI flag that prints both.

**Options.** For the comparison the CLI takes the limits from BenchExec:
`--jobs 1`, `--timeout` equal to the limit, `--memory-limit` set just
under BenchExec's, `--copies none`. *In the way:* the defaults (2 s,
1 GiB, the recursion limit) would end runs before the external limit, so
the step names every flag. `Reason::RecursionLimit` and `CopyBound` count
as unknown.

**Ordinary layer and coverability.** `ordinary::read_tptp`, `translate`
and `Translation` already decide ILTP. For intuitR the input is the TPTP
file itself, and the same holds for QCover and `mist::read` on `.spec`.
No library change is needed.

## 4. Risks, open questions, and what the prompt should add

**Risks**
- *Old toolchains.* APLL needs OCaml 4.06 and Coq 8.8. The Maude prover was
  tested on Maude 2.7.1, while nixpkgs has 3.5.1 [34]. QCover needs Python
  2.7. Sympli needs MLton. Each may take hours to build, or may not build
  (inference).
- *Licences.* The Maude prover, APLL, linTAP and Lolli state none. Fetching
  them by the flake, never vendored, follows the step, but whether CI may
  fetch and run unlicensed code is the author's call (inference).
- *Start-up.* Prolog, Maude and the JVM pay a load time on every problem.
  Report it, and do not rank on problems under a second (inference).
- *Large inputs.* LLTP files of up to 103 MB may break other tools'
  readers; count those apart as "input not read".
- *BenchExec on NixOS* needs cgroup v2 delegation set up by the
  administrator [30]. If that fails, the step's fallback (the harness,
  with its guarantees argued) applies.

**Open questions**
1. Does a CLL-only tool's verdict on an ILL problem answer the ILL question?
   It depends on conservativity per fragment; [1] cites Schellinx (1991).
   Check this before such cells are filled.
2. The step's set omits ILTP, but the matrix covers the ordinary layer.
   Is intuitR a column, or only a row?
3. Do QCover, and mist/bfc/Petrinizer if they can be found, join the qcover
   table?
4. The time limit: 10 s on the 1 342 recorded problems plus the families
   and qcover for the night, with the whole library at a short limit in CI
   only?
5. Is Click & coLLecT's auto-prover callable without its server, and what
   is its time-out? The code read in `auto_prove_sequent.ml` sets none.
6. llprover's default threshold: 5 in the source, three on its page.

**What the prompt should add.** The tool list above, with the commit or
release of each. A per-tool verdict semantics, with bounded search counted
as unknown. The budget arithmetic and the chosen limit. Process-level
timing for every tool. A check of translator correctness on the families.
The treatment of the 28 wrong headers. The checker's memory on the 14 large
nets. The library changes of section 3 (public walks, serde on `Options`, a
version field) as the refactor's input.

## 5. Sources

1. C. Olarte, V. de Paiva, E. Pimentel, G. Reis, "The ILLTP Library for Intuitionistic Linear Logic", Linearity-TLLA 2018, EPTCS 292 (2019) 118–132. https://arxiv.org/abs/1904.06850
2. LLTP, meta-logic, GitHub, GPL-3.0, commit e0394fb (2025-01-27). https://github.com/meta-logic/lltp
3. C. Olarte, Linear-Logic-Prover-in-Maude, GitHub (last commit 2020-11-03). https://github.com/carlosolarte/Linear-Logic-Prover-in-Maude
4. N. Tamura, llprover, CSPSAT project, page and source v2.0 (2020). https://cspsat.gitlab.io/llprover/ and https://cspsat.gitlab.io/llprover/llprover.pl
5. H. Mantel, J. Otten, "linTAP: A Tableau Prover for Linear Logic", TABLEAUX 1999, LNAI 1617, 217–231; page of 2008. http://www.leancop.de/lintap/
6. Wu Jui-Hsuan, APLL (LL_prover), GitHub (last commit 2019-04-29). https://github.com/wujuihsuan2016/LL_prover
7. Wu Jui-Hsuan, "Recherche de preuve automatique en logique linéaire", ENS L3 internship report, 2018. https://www.lix.polytechnique.fr/Labo/Jui-Hsuan.WU/assets/pdf/l3report.pdf
8. E. Callies, O. Laurent, "Click and coLLecT: An Interactive Linear Logic Prover", TLLA 2021, https://hal-lirmm.ccsd.cnrs.fr/lirmm-03271501; code (LGPL-2.1, last commit 2025-04-07) https://github.com/ComputerAidedLL/click-and-collect; site https://click-and-collect.linear-logic.org/
9. K. Chaudhuri, Sympli, GitHub, BSD-2-Clause, archived 2021. https://github.com/chaudhuri/sympli
10. K. Chaudhuri, F. Pfenning, "Focusing the inverse method for linear logic", CSL 2005, LNCS 3634, 200–215; "A focusing inverse method theorem prover for first-order linear logic", CADE-20 2005, LNCS 3632, 69–83; K. Chaudhuri, PhD thesis CMU-CS-06-162, 2006. https://www.csd.cmu.edu/sites/default/files/phd-thesis/CMU-CS-06-162.pdf
11. GitHub user ibrahimcesar, lolli, GitHub, MIT, v0.2.0 (2025-12-05). https://github.com/ibrahimcesar/lolli
12. R. Moot, LinearOne, GitHub, LGPL-2.1 (last commit 2026-06-11). https://github.com/RichardMoot/LinearOne
13. GitHub user andykitchen, linear-logic ("LL prover explorer" in [1]), GitHub, GPL-3.0 (last commit 2014-09-03). https://github.com/andykitchen/linear-logic
14. J. Hodas, D. Miller, Lolli. http://www.lix.polytechnique.fr/~dale/lolli/
15. A. Schack-Nielsen, C. Schürmann, "Celf – A Logical Framework for Deductive and Concurrent Systems", IJCAR 2008. https://pure.itu.dk/en/publications/celf-a-logical-framework-for-deductive-and-concurrent-systems-sys/
16. O. Laurent, Yalla, GitHub, LGPL-3.0. https://github.com/olaure01/yalla
17. B. Xavier, C. Olarte, G. Reis, V. Nigam, "Mechanizing Focused Linear Logic in Coq", ENTCS 338 (2018) 219–236; coq-ll, GPL-3.0 (last commit 2026-04-09). https://github.com/meta-logic/coq-ll
18. S. Kalvala, V. de Paiva, "Mechanizing Linear Logic in Isabelle", 1995. https://www.cl.cam.ac.uk/~lcp/papers/Workshop/papers/kalvala-linear.pdf; Isabelle `Sequents` README. https://isabelle.in.tum.de/website-Isabelle2011/dist/library/Sequents/README.html
19. "Certified Proof Search for Intuitionistic Linear Logic" (Agda; authors not stated on the page). http://gallais.github.io/proof-search-ILLWiL/
20. T. Raths, J. Otten, C. Kreitz, "The ILTP Problem Library for Intuitionistic Logic (Release v1.1)", J. Automated Reasoning 38 (2007) 261–271, doi:10.1007/s10817-006-9060-z; results page (2017). https://www.iltp.de/results.html
21. K. Claessen, D. Rosén, "SAT modulo intuitionistic implications", LPAR 2015, LNCS 9450, 622–637. https://research.chalmers.se/en/publication/?id=230822
22. C. Fiorentini, "Efficient SAT-based Proof Search in Intuitionistic Propositional Logic", CADE-28 2021, 217–233; intuitR (BSD-3-Clause, last commit 2022-07-22) https://github.com/cfiorentini/intuitR; intuitRGC (GPL-3.0) https://github.com/cfiorentini/intuitRGC; software list https://fiorentini.di.unimi.it/papers.html
23. M. Blondin, A. Finkel, C. Haase, S. Haddad, "Approaching the Coverability Problem Continuously", TACAS 2016, https://arxiv.org/abs/1510.05724; QCover, Apache-2.0 (last commit 2021-03-03) https://github.com/blondimi/qcover
24. J. Esparza, R. Ledesma-Garza, R. Majumdar, P. Meyer, F. Niksic, "An SMT-based approach to coverability analysis", CAV 2014, 603–619. https://archive.model.in.tum.de/um/courses/petri/SS2017/material/cav2014-paper.pdf
25. mist2, Petri Nets Tool Database, University of Hamburg. https://www.informatik.uni-hamburg.de/TGI/PetriNets/tools/db/mist2.html
26. P. Lincoln, J. Mitchell, A. Scedrov, N. Shankar, "Decision problems for propositional linear logic", Annals of Pure and Applied Logic 56 (1992) 239–311. https://doi.org/10.1016/0168-0072(92)90075-B
27. J.-M. Andreoli, "Logic Programming with Focusing Proofs in Linear Logic", J. Logic and Computation 2 (1992) 297–347. https://doi.org/10.1093/logcom/2.3.297
28. R. Lazić, S. Schmitz, "Nonelementary Complexities for Branching VASS, MELL, and Extensions", ACM TOCL 16 (2015). https://doi.org/10.1145/2733375
29. R. Statman, "Intuitionistic propositional logic is polynomial-space complete", TCS 9 (1979) 67–72. https://doi.org/10.1016/0304-3975(79)90006-9
30. D. Beyer, S. Löwe, P. Wendler, "Reliable benchmarking: requirements and solutions", STTT 21(1) (2019) 1–29, doi:10.1007/s10009-017-0469-y; BenchExec, Apache-2.0. https://github.com/sosy-lab/benchexec
31. M. Raasveldt, P. Holanda, T. Gubner, H. Mühleisen, "Fair Benchmarking Considered Difficult", DBTest 2018. https://doi.org/10.1145/3209950.3209955
32. T. Hoefler, R. Belli, "Scientific benchmarking of parallel computing systems", SC 2015. https://doi.org/10.1145/2807591.2807644
33. GitHub Docs, "GitHub-hosted runners" (read 2026-10-08). https://docs.github.com/en/actions/reference/runners/github-hosted-runners
34. nixpkgs, `pkgs/by-name/ma/maude/package.nix` (master, read 2026-10-08: 3.5.1, GPL-2.0+). https://github.com/NixOS/nixpkgs/blob/master/pkgs/by-name/ma/maude/package.nix

[R] linlog repository at this snapshot: `plan/29-comparison.md`, `plan/notes/comparison.md`, `plan/reports/17-assessment.md` §2.2, `21-defaults.md`, `25-ordinary-logic.md`, `27-horn.md`, `bench/COMPARISON.md`, `.claude/rules/core-search.md`, `core-sequents.md`, `core-inputs.md`, `bench.md`, and the source files named in section 3.

Sources checked 2026-10-08: 34 checked, 0 corrected, 0 removed, 0 claims marked.
