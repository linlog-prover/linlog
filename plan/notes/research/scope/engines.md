# Scope: search engines and engineering

Written 2026-10-09 from a web search and the snapshot's `plan/later.md`
and `plan/notes/research/README.md`. Sources are numbered in section 4;
"[L]" is `plan/later.md`, "[R]" the research notes' README, "[rep26]" and
"[rep25]" the reports of those steps. "(inference)" marks a conclusion of
this note's own. "Extends" means a saved plan or follow-up already names
the thing; "beside" means it touches a planned step without being in
it; "new" means nothing saved names it.

## 1. The field as it stands

**Complexity sets which solver fits.** Constant-only MLL is
NP-complete [1], and so is the smallest Horn fragment of linear logic
[2]. MALL is PSPACE-complete [3], and Das encodes QBF into focused MALL
and focused MALL search back into alternating time, which gives MALL
fragments complete for each level of the polynomial hierarchy [4]. So
MLL is a natural target for SAT and MALL for QBF. Neither search found
an implemented prover that sends MLL or MALL provability to a SAT or
QBF solver. Matsuoka argues the other way: Partition and 3D Matching
can be written as Horn MLL sequents and handed to a linear-logic prover
in place of a SAT solver [5].

**SAT for the neighbouring logics works.** intuit decides
intuitionistic propositional logic with an incremental SAT solver [6].
Fiorentini, Goré and Graham-Lengrand read its loop as a generalised
left-implication rule of LJT/G4ip, and note that it terminates without
loop detection [7]. intuitR adds restarts and writes either a
derivation or a countermodel [8]. On ILTP's 274 propositional problems
the best recorded prover decides 262 [9]. linlog's layer, going
through ILL, decides 108 at 2 s [rep25]. Fiorentini also has an ASP-based
generator of minimal intuitionistic countermodels [10].

**Petri nets are an SMT and ASP domain.** Petrinizer decides
coverability with SMT over the state equation and traps [11], and
QCover with continuous nets and Z3 [12]. SMPT is "a portfolio of
methods" for symbolic reachability with verdict certificates [13]. An
ASP encoding of Petri nets has answer sets in one-to-one correspondence
with firing sequences [14]. linlog's Horn engine already has its own
simplex for the state equation, and lists trap constraints and an exact
integer equation as follow-ups [L].

**Learned guidance** is established for classical first-order provers.
rlCoP guides a connection prover with Monte-Carlo search and models
learned from earlier attempts, which are gradient-boosted trees for the
policy and the value. Within the same inference budget it solves more
than 40 % more problems than its baseline [15]. ENIGMA learns
given-clause selection for E from earlier proofs [16]. Loos et al. use
deep networks to select clauses in E [17]. Spider-style strategy
discovery builds Vampire's schedules [18]. SATzilla picks a SAT solver
per instance from empirical hardness models [19]. In linear logic, the
only learned search found is Neural Proof Nets, which chooses MILL
axiom links with Sinkhorn networks for Dutch sentences and reaches up
to 70 % accuracy [20], and Moot's proposals that follow from it [21].
For intuitionistic logic, a seq2seq network writes λ-term proofs, and a
search guided by its near misses beats brute force [22]. LeanDojo makes
Lean an environment that programs can interact with, with a benchmark
of about 98 700 theorems [23].

**Parallel and portfolio.** Cube-and-conquer splits a problem by
lookahead into cubes and hands them to CDCL [24]. linlog removed its
portfolio after two baselines showed no gain, and its pool decides 37
more LLTP problems than one thread, at 2.9× the median time [L].

**Incremental and interactive.** Incremental SAT reuses one solver
across a series of related instances [25]. Isabelle/PIDE and Coq's
asynchronous processing check a document as separate prioritised
tasks, not as a read-eval-print loop [26, 27]. In Click & coLLecT, an
exhaustive failure of its auto-prover turns a turnstile red ("known
not to be provable"), and a time-out turns it orange [28].

**Competitions.** CASC's divisions are all classical [29]. The TPTP
World has supported non-classical logics since v9.0.0, so far modal
ones declared through a logic specification [30, 31]. ILTP keeps a
results page [9]. The Model Checking Contest 2025 had 142 models, 1 855
instances and 24 115 queries in PNML, run on up to 4 cores and 16 GB
[32]. The 2016 Unsolvability IPC asked planners for solvable,
unsolvable or unknown, within 8 GB and 30 min, and penalised false
"unsolvable" answers heavily [33]. No linear-logic competition exists
(inference, from [9, 29, 30] and the comparison note's finding that no
other tool has results on LLTP).

## 2. Candidate extensions

### engines-1: A SAT-based intuitionistic engine for ordinary logic

*What.* An intuit/intuitR-style decision procedure [6, 8] behind the
ordinary-logic layer. It answers with an LJ-style derivation or a
finite Kripke countermodel, beside the ILL translations.
*Value.* The layer refutes no non-theorem that runs into the copy
bound. 166 ILTP problems are undecided, 35 of them deepening without
end [L, rep25]. The recorded provers decide up to 262 [9]. A countermodel
is the certificate that later.md's "Kripke countermodels" escalation
lacks a source for [L].
*Cost.* A CDCL solver with assumptions. varisat is pure Rust and
MIT/Apache but has had no release since 2020 [34]. The `cadical` crate
compiles C++ [34], which works against wasm (inference). RustSAT is a
framework over several backends [35]. One session for the algorithm,
one for the countermodel's check (inference).
*Relation.* Extends step 25 and the certified refutations; sits beside
step 31's Rocq refutations.
*Room now.* Engines registered per logic, not only per linear
fragment. `Refutation::KripkeModel` as a named variant [R]. The
ordinary layer's verdict able to carry a proof that is not a linear
`Proof`.

### engines-2: Lazy SAT over axiom linkings for MLL

*What.* A SAT solver picks a linking (one partner per literal
occurrence), and the net engine's criterion checks it. A failing
switching becomes a blocking clause, as in lazy SMT.
*Value.* MLL is NP-complete [1], and the hard MLL inputs are Horn
encodings with repeated literals [5]. There, step 26 measured the
focused engines 10 to 10⁵ times faster than the net engine [rep26]. A
solver that learns from conflicts might beat both. I found no
published encoding, so this is research (inference).
*Cost.* A SAT dependency (see engines-1), the encoding,
conflict-clause extraction from the criterion, and a measurement on the
partition families.
*Relation.* Beside step 35 (net pruning, essential nets). New as an
engine.
*Room now.* "Registering an engine is a list" and `Feature` as data
[R]. `Stop` polled from a solver callback. The `Criterion` value that
four steps extend [R], usable as a black-box checker.

### engines-3: Integer and trap refutations for the Horn engine through a solver

*What.* The state equation over the integers, with Esparza–Melzer
traps added on demand (Petrinizer's loop [11]), and bounded
reachability by unrolling for proofs, solved by SMT or ILP.
*Value.* This settles the follow-ups where "the `a` stay odd" and the
equation is rational-feasible, and the weights above 2²⁰ [L]. It also
covers the qcover elements that QCover prunes by LP [12].
*Cost.* An exact solver. Z3 in wasm needs threads [R, 32-web note], so
the choice is a pure-Rust rational simplex with branch-and-bound or an
optional native feature (inference). A trap is a small certificate
checked by computation (inference).
*Relation.* Extends step 27's follow-ups and the "Petri-net
unreachability" item under certified refutations.
*Room now.* `Refutation` growing by named fields (`StateEquation`
weights, a `Trap`) [R], `Equation.needed: i128` and the rule for
numbers above 2⁵³ [R].

### engines-4: Encodings as exports (DIMACS, SMT-LIB, ASP, QBF)

*What.* `linlog seq dimacs|smtlib|lp` writes a bounded problem for an
external solver: an MLL linking (engines-2), a Horn unrolling
(engines-3), an ASP program for nets [14]. QBF for MALL would need a
direction Das does not give [4] (inference).
*Value.* It shows whether a solver route pays before any engine is
built, and gives researchers inputs for their own solvers. Each
encoding is a written-down reduction a teacher can show (inference).
*Cost.* Low per encoding, and no dependency: these are printers over
the forest.
*Relation.* Beside the exports (step 22) and step 29's translators from
the parsed `Sequent`.
*Room now.* The public non-recursive formula walk [R]. Translators as
one family next to the per-tool printers of step 29.

### engines-5: Learned dispatch and engine schedules

*What.* Per-instance selection of the engine, bias, copies and threads
from cheap features of the sequent (SATzilla [19]; Vampire's CASC mode
reads syntactic counters [18] (not in the source)). It also covers time-sliced schedules
across engines, such as the focused search handing over to the net
engine at the recursion limit, which step 26 names as a follow-up
[rep26].
*Value.* The default is one hand-measured table of four rows [rep26].
Every new engine (Horn, inverse, nets, SAT) multiplies the choices.
The portfolio failed for the same engine run twice [L]; engines with
different strengths are the case where schedules pay in [18, 19]
(inference that this transfers).
*Cost.* Feature extraction, training on the baselines' CSVs, and the
learned rules emitted as `DISPATCH` rows with their measurement, so
the dispatch stays deterministic and documented. Data exists from
steps 14 to 30.
*Relation.* Extends step 26 item 4 and step 21's defaults. It needs
37's inverse engine to have rows to choose among.
*Room now.* `Engine` as data with its fragments and modes [R]. A
public `Features` value. The verdict saying which row and which slice
decided. A schedule expressible in `search::Options` with serde.

### engines-6: Learned choice ordering inside the focused engine

*What.* A small model (trees, as in rlCoP [15]) ranks the focus
candidates and context splits at each synchronous choice, trained on
the engine's own traces of LLTP and the families.
*Value.* Uncertain. An exhaustive failure explores every choice
whatever their order, so a ranking can only speed up proofs found
early (inference). The same kind of ranker could also order the Horn
engine's greedy search, which meets plateaus on the BART and DES nets
[L]. Research.
*Cost.* A trace format, offline training outside the workspace, and a
model compiled into code. The counters of `bench/targets.sh` stay a
function of the input under a fixed model (inference).
*Relation.* New. It generalises the atom bias, which is already the
focused engine's own dispatch [rep26].
*Room now.* An ordering hook in options, as the bias is, not
hard-wired. A stable trace or wire form of decision points.

### engines-7: A proof-search environment and dataset for machine learning

*What.* `interact` over a JSON line protocol (state in, legal actions
out, apply), plus released datasets of checked proofs and certified
refutations from the baselines. This is LeanDojo's role for Lean [23].
*Value.* ML for linear logic has worked on MILL in grammar [20, 21].
A decidable domain with checked negatives, where an unprovable subgoal
is a label, is something Lean cannot give (inference).
*Cost.* Low: the interactive state and its JSON exist [L]. A filtered
legal-action list means trying each rule on a clone [L].
*Relation.* Beside step 32 (the same calls the web client makes). New
as a product.
*Room now.* Versioned `Interactive` JSON [R], stable goal ids, and
`rules` able to return only applicable rules.

### engines-8: Live provability feedback while proving by hand

*What.* After every step, each open goal is searched in the
background on a small budget and marked proved, unprovable (with the
refuter's reason) or unknown. This is Click & coLLecT's turnstile
colours [28], but automatic, on PIDE's asynchronous model [26].
*Value.* A wrong split of a `⊗` context shows at once and says why,
which matters most in teaching (inference).
*Cost.* A per-goal budget, a cache keyed by canonical sub-sequent, a
worker in the web client. A suspendable search would avoid restarts
[L, R].
*Relation.* Extends steps 9 and 32 and the refuters (ref): later.md
assigns "a budget per goal" to 22 and 32 [L].
*Room now.* `Stop::poll(progress)` [R]. Goals with stable identities
across `apply`/`undo`. A cache owned by a session value.

### engines-9: Search state kept across calls

*What.* A `Runtime` or session that keeps the memo and refutation
cache between goals sharing a context, in the spirit of incremental
SAT [25]: in `interact`, in batch runs, and for Horn programs queried
with many goals.
*Value.* LLTP's 3 137 net problems come from 76 nets [L]. Problems
sharing a net could share reachable-marking sets when their initial
markings agree (inference, unchecked). An interactive session re-proves
the same subgoals.
*Cost.* Memo entries must not depend on the goal (inference). Memory
accounting across calls is needed.
*Relation.* Extends the deferred "a `Runtime` kept across calls"
(parallel follow-ups) and step 24's batch [L].
*Room now.* A runtime value beside D9's plain-data options, as
later.md already notes it would need [L]. `memory::Account` charged
per session [R].

### engines-10: Competition inputs for the Horn engine (STRIPS planning, PNML nets)

*What.* Two readers into Horn programs. Grounded STRIPS (for example
Fast Downward's translator output) uses the encodings of Masseron et
al. and of Kanovich and Vauzeilles [36, 37]. PNML P/T nets with the
MCC's reachability queries, as far as they are coverability or a
target marking [32].
*Value.* Real non-theorems, which later.md says the benchmarks lack.
Planning domains are a deferred source there [L]. The Unsolvability
IPC's three-valued scoring matches linlog's verdicts [33]. The MCC
supplies public instances and a scoreboard [32]. The Horn engine would
not compete with dedicated planners or MCC tools (inference).
*Cost.* A grounded-input reader (no grounder of linlog's own), a PNML
reader (an XML dependency), and a mapping of query kinds; boolean
combinations of bounds are out of reach (inference).
*Relation.* Extends "Problems from practice" [L] and step 27. It sits
beside step 29's comparison.
*Room now.* `Program` buildable from sources other than a parsed
sequent, and three-valued verdicts with reasons in the batch output.

### engines-11: Linear logic in the TPTP World

*What.* Propose linear logic as a TPTP non-classical logic, through a
logic specification as modal logics have [30, 31], and convert LLTP to
it. linlog becomes a SystemOnTPTP-callable prover.
*Value.* Every LL prover reads its own syntax; LLTP is "TPTP-like" [R,
29-comparison note]. A shared format is what a future competition
division would need (inference).
*Cost.* Mostly coordination with the TPTP maintainers. A reader and
printer in linlog.
*Relation.* Beside steps 29 and 30, and the LLTP header report [L].
New as an outreach item.
*Room now.* The parser with room for a second concrete syntax, and
`Mode` and `Fragment` growable [R].

## 3. Considered and rejected

- **A QBF engine for MALL.** No implemented MALL-to-QBF prover was
  found, and Das's encodings serve complexity bounds [4]. The focused
  engine already decides the generated QBF family, where a second
  thread gains 1.2× [L]. engines-4 can test the idea cheaply.
- **ASP (clingo) as an engine.** Its Petri-net encoding enumerates
  firing sequences to a horizon [14], which the Horn engine already
  searches explicitly. It would add a large dependency for a bounded
  answer (inference). It is kept only as an export (engines-4).
- **Classical ordinary logic through SAT.** Trivial for a SAT solver,
  and later.md explicitly does not offer linlog as one [L].
- **More parallelism (cube-and-conquer over the focused engine,
  threads in the browser).** The pool's measured gains are small,
  cancellation-bound and memory-costly [L]. wasm threads need nightly
  and cross-origin isolation [R, 32-web note].
- **An LLM inside the library.** The core reads no clock and must stay
  deterministic and offline (CLAUDE.md). An LLM is better served as a
  client of engines-7's protocol.
- **Entering CASC.** It has no non-classical division [29], and its
  propositional reach would be via the trivial classical embedding.

## 4. Sources

1. P. Lincoln, T. Winkler, "Constant-only multiplicative linear logic is NP-complete", Theoretical Computer Science 135(1) (1994) 155–169. https://ftp.math.utah.edu/pub/tex/bib/idx/tcs1990/135/1/155_169.html
2. M. Kanovich, "Horn programming in linear logic is NP-complete", LICS 1992. https://www.lfcs.inf.ed.ac.uk/events/lics/1992/Kanovich-Hornprogramminginli.html
3. P. Lincoln, J. Mitchell, A. Scedrov, N. Shankar, "Decision problems for propositional linear logic", APAL 56 (1992) 239–311. https://doi.org/10.1016/0168-0072(92)90075-B
4. A. Das, "From QBFs to MALL and back via focussing", J. Automated Reasoning (2020). https://arxiv.org/abs/1906.03611
5. S. Matsuoka, direct encodings of 3D Matching and Partition into Horn MLL, HCVS 2017. https://software.imdea.org/Conferences/hcvs17/papers/paper_1.pdf
6. K. Claessen, D. Rosén, "SAT modulo intuitionistic implications", LPAR 2015, LNCS 9450, 622–637. https://research.chalmers.se/en/publication/?id=230822
7. C. Fiorentini, R. Goré, S. Graham-Lengrand, "A proof-theoretic perspective on SMT-solving for intuitionistic propositional logic", TABLEAUX 2019. https://www.csl.sri.com/users/sgl/Work/Reports/2019-07-G4-SMT-Tableaux19.pdf
8. C. Fiorentini, "Efficient SAT-based Proof Search in Intuitionistic Propositional Logic", CADE-28 2021, 217–233; intuitR, BSD-3-Clause. https://github.com/cfiorentini/intuitR
9. J. Otten, T. Raths, ILTP results page (2017). https://www.iltp.de/results.html
10. C. Fiorentini, software page (minimal countermodels by ASP, IJCAI 2019). https://fiorentini.di.unimi.it/sw.html
11. J. Esparza, R. Ledesma-Garza, R. Majumdar, P. Meyer, F. Niksic, "An SMT-based approach to coverability analysis", CAV 2014. https://archive.model.in.tum.de/um/courses/petri/SS2017/material/cav2014-paper.pdf
12. M. Blondin, A. Finkel, C. Haase, S. Haddad, "Approaching the Coverability Problem Continuously", TACAS 2016. https://arxiv.org/abs/1510.05724
13. N. Amat, S. Dal Zilio, "SMPT: A Testbed for Reachability Methods in Generalized Petri Nets", FM 2023. https://arxiv.org/abs/2302.14741
14. S. Anwar, C. Baral, K. Inoue, "Encoding Petri Nets in Answer Set Programming for Simulation Based Reasoning", arXiv 2013. https://arxiv.org/abs/1306.3542
15. C. Kaliszyk, J. Urban, H. Michalewski, M. Olšák, "Reinforcement Learning of Theorem Proving", NeurIPS 2018. https://arxiv.org/abs/1805.07563
16. J. Jakubův, J. Urban, "ENIGMA: Efficient Learning-based Inference Guiding Machine", arXiv 2017. https://arxiv.org/abs/1701.06532
17. S. Loos, G. Irving, C. Szegedy, C. Kaliszyk, "Deep Network Guided Proof Search", LPAR-21 2017. https://research.google/pubs/pub45813/
18. F. Bártek, K. Chvalovský, M. Suda, "Regularization in Spider-Style Strategy Discovery and Schedule Construction", IJCAR 2024. https://arxiv.org/abs/2403.12869
19. L. Xu, F. Hutter, H. Hoos, K. Leyton-Brown, "SATzilla: Portfolio-based Algorithm Selection for SAT", JAIR 32 (2008) 565–606. https://arxiv.org/abs/1111.2249
20. K. Kogkalidis, M. Moortgat, R. Moot, "Neural Proof Nets", CoNLL 2020. https://arxiv.org/abs/2009.12702
21. R. Moot, "Perspectives on neural proof nets", arXiv 2022. https://arxiv.org/abs/2211.04141
22. T. Sekiyama, A. Imanishi, K. Suenaga, "Towards Proof Synthesis Guided by Neural Machine Translation for Intuitionistic Propositional Logic", arXiv 2017. https://arxiv.org/abs/1706.06462
23. K. Yang et al., "LeanDojo: Theorem Proving with Retrieval-Augmented Language Models", NeurIPS 2023 Datasets and Benchmarks. https://arxiv.org/abs/2306.15626
24. M. Heule, O. Kullmann, S. Wieringa, A. Biere, "Cube and Conquer: Guiding CDCL SAT Solvers by Lookaheads", HVC 2011, LNCS 7261, 50–65. https://research.jku.at/en/publications/cube-and-conquer-guiding-cdcl-sat-solvers-by-lookaheads/
25. N. Eén, N. Sörensson, "Temporal Induction by Incremental SAT Solving", ENTCS 89(4) (2003) 543–560. https://research.chalmers.se/en/publication/74330
26. M. Wenzel, "Asynchronous User Interaction and Tool Integration in Isabelle/PIDE", ITP 2014. https://www21.in.tum.de/~wenzelm/papers/itp-pide.pdf
27. B. Barras, C. Tankink, E. Tassi, "Asynchronous processing of Coq documents: from the kernel up to the user interface", ITP 2015. https://arxiv.org/abs/1506.05605
28. Click & coLLecT site (read 2026-10-09). https://click-and-collect.linear-logic.org/
29. CASC-29 design (divisions). https://tptp.org/CASC/29/Design.html
30. A. Steen, G. Sutcliffe, "TPTP World Infrastructure for Non-classical Logics", arXiv 2025; TPTP non-classical logics page. https://arxiv.org/abs/2508.09318 and https://tptp.org/UserDocs/TPTPLanguage/NonClassicalLogics.html
31. M. Taprogge et al., "The NTF format: Extension of the logic specification", TPTP-TP 2024 slides. https://tptp.org/TPTPTParty/2024/SpeakersSlides/MelanieTaprogge.pdf
32. Model Checking Contest 2025, results and call. https://mcc.lip6.fr/2025/
33. Unsolvability International Planning Competition 2016. https://unsolve-ipc.eng.unimelb.edu.au
34. varisat (MIT/Apache-2.0, 0.2.2 of 2020) https://github.com/jix/varisat; cadical crate (CaDiCaL, MIT, C++ compiled in) https://lib.rs/crates/cadical
35. C. Jabs, "RustSAT: A Library For SAT Solving in Rust", SAT 2025, LIPIcs 341. https://drops.dagstuhl.de/storage/00lipics/lipics-vol341-sat2025/html/LIPIcs.SAT.2025.15/LIPIcs.SAT.2025.15.html
36. M. Masseron, C. Tollu, J. Vauzeilles, "Generating plans in linear logic I: Actions as proofs", TCS 113(2) (1993) 349–370. https://hal.archives-ouvertes.fr/hal-00003530
37. M. Kanovich, J. Vauzeilles, "The classical AI planning problems in the mirror of Horn linear logic", MSCS 11(6) (2001) 689–716. https://dblp.org/pid/34/462

Repository: [L] `plan/later.md`; [R] `plan/notes/research/README.md`
and `32-web.md`, `29-comparison.md` in that directory; [rep25]
`plan/reports/25-ordinary-logic.md` via later.md; [rep26]
`plan/reports/26-focused-engine.md`, "Item 4".

Sources checked 2026-10-09: 37 checked, 1 corrected, 0 removed, 1 claims marked.
