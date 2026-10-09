# Interoperability and certificates: where linlog's scope could grow

Written 2026-10-09 from web sources and this snapshot's `plan/later.md`,
`plan/notes/research/README.md` and the step notes it summarises.
Every claim cites a source in the last section; "(inference)" marks a
conclusion of this note's own. Repository sources are cited by path.

What is already planned, so that the candidates below can say how they
relate: step 31 builds a Rocq library of linlog's own with a reflective
checker and keeps the NanoYalla export (`plan/later.md`, "Second
certificate kernels"); a Lean target is deferred until after 31, with
`leanprover/cslib` named as the library that now has classical linear
logic (`plan/later.md`, table and "Deferred beyond the table");
certified refutations are a list of escalations (Kripke and phase
models, failure certificates, reflection) of which 31 takes the cheap
ones (`plan/later.md`, "Certified refutations"); step 32 ships wasm
through wasm-bindgen in a worker, with JSON as the wire form
(`research/32-web.md`); step 22 made every output's options plain data
with serde so that "an editor plugin or a notebook reuses the same JSON"
(`plan/later.md`, "Configurable output"); step 25 reads TPTP problems,
29 compares linlog with other provers, 38 adds quantifiers
(`research/README.md`).

## 1. The field as it stands

**Universal proof checkers.** Dedukti is a type checker for the
λΠ-calculus modulo theory, in which constructive and classical predicate
logic, simple type theory, pure type systems and the calculus of
inductive constructions have been expressed, and which has checked
libraries produced by Zenon, iProver, FoCaLiZe, HOL Light and Matita
[Dedukti]. Lambdapi is the interactive successor; its export modules
write Dedukti and Coq, and HRS and XTC for the confluence and
termination tools [Lambdapi-export]. Deducteam's hol2dk translates HOL
Light proofs to Dedukti, Lambdapi and Rocq, and reports for `hol.ml`
that `dkcheck` checks the Dedukti output in 4m11s where checking the Rocq
output takes 10m23s (about 15m40s with the translation steps) [hol2dk]; an Isabelle component and a work-in-progress Lean translation
also exist [Deducteam-repos]. A search for a linear logic encoding in
Dedukti found none (absence not proved).

**Foundational proof certificates (FPC).** Miller's programme defines
the meaning of proof evidence by focused sequent calculi: a certificate
is read by "clerks and experts" that steer a focused checker, so that
producer and consumer share one formal semantics [FPC17]. The checker is
a λProlog program, chosen because binding, substitution and backtracking
search let a certificate omit details that the checker reconstructs by
bounded search [FPC13]. The published frameworks are classical and
intuitionistic first-order [FPC17]; LKU, the focused system they build
on, obtains classical, intuitionistic and MALL focused calculi as
fragments of one system [LKU]. ELPI is an embeddable λProlog
interpreter, faster than Teyjus [ELPI].

**SMT formats.** LFSC extends LF with side conditions written in a small
functional language and was applied to the solvers clsat and CVC3
[LFSC]. Alethe is veriT's format, also emitted by cvc5, with a term
language close to SMT-LIB and steps of varying granularity, reconstructed
in Isabelle and Coq [Alethe]. Carcara, written in Rust, checks Alethe
proofs and elaborates coarse steps into fine ones, and by default
refuses a rule it does not know (not in the source) [Carcara]. In Lean 4, `bv_decide` sends
a goal to CaDiCaL and checks the LRAT proof by verified algorithms in
Lean; the proof then depends on `Lean.ofReduceBool`, putting the Lean
compiler in the trusted base [bv_decide].

**Linear logic in proof assistants.** Rocq has Yalla, NanoYalla and
coq-ll (`research/31-rocq.md`). Lean 4's cslib (Apache-2.0) has
`Cslib/Logics/LinearLogic/CLL` with `Basic.lean`, `CutElimination.lean`,
`EtaExpansion.lean`, `MLL.lean` and `PhaseSemantics/` [cslib], sequents
as multisets with no exchange rule (`research/31-rocq.md`). Isabelle's
distribution has the `Sequents` object logic with an ILL theory
contributed by Kalvala, after Kalvala and de Paiva 1995 [Sequents]; I
found no AFP entry on linear logic. In Agda there is "Certified Proof
Search for Intuitionistic Linear Logic" (`research/29-comparison.md`,
reference 19) and Kokke's substructural logics development [Kokke].

**TPTP.** Beyond FOF, the TPTP has a typed first-order form with
arithmetic, TFF [TFF], and a higher-order form, THF [THF]. Since v9.0.0
it supports non-classical logics through a logic specification; what is
standardised so far is quantified multi-modal logic [NTF]. The SZS
ontology gives one-word statuses (Theorem, CounterSatisfiable, Timeout,
GaveUp, …) printed as `% SZS status` lines, and dataforms such as Proof
and Model [SZS]. Derivations are written in the TSTP format [TSTP], and
a TPTP format for interpretations covers Tarskian, Herbrand and Kripke
interpretations [Interp].

**Bindings and tooling.** PyO3 with maturin builds Python extension
wheels; the `abi3` feature targets CPython's stable API so that one wheel
serves several versions, and `abi3t` does so for free-threaded CPython
[PyO3], [maturin]. WASI 0.3.0 was released on 11 June 2026 with async in
the component model itself, and a 0.3 runtime need not force migration
from 0.2 [WASIp3]. Rust targets `wasm32-wasip2` (Tier 2, stable) and
`wasm32-wasip3` (Tier 3, nightly); jco (JavaScript) and componentize-py
(Python) make WASI 0.2 components; jco's 0.3 support is experimental [WASI-lang]. rust-analyzer's
`lsp-server` 0.10.0 (2026-07-16) is a synchronous LSP scaffold
[lsp-server]; tower-lsp's own fork says its development stopped
[lspower]. Tree-sitter is an incremental parser generator used by
Emacs, Neovim, Helix and Zed [tree-sitter]. JupyterLite runs Jupyter in
the browser with Pyodide, CPython compiled to WebAssembly (not in the source) [Pyodide];
anywidget pairs an ES module with a Python class to make a Jupyter
widget that runs in JupyterLab, Colab and VS Code [anywidget].

## 2. Candidate extensions

Each candidate says what it would be, who uses it, the cost, how it
relates to the saved plans, and what the API design (step 28's) should
leave room for now.

### interop-1: the proof term as a specified exchange format

*What.* A written specification, with a JSON Schema, of the forms that
are read back (`Sequent`, `Proof`, `ProofStructure`, `Interactive`, the
refutation payloads), with the preorder numbering and the rule semantics
stated so that another program can emit a term for linlog's checker or
check linlog's term without linlog. *Value.* Every other candidate
consumes it; a foreign prover (Click & coLLecT, an LLM, a student's
code) gets an independent checker by emitting it (inference). *Cost.*
Documentation and a schema test; no engine. *Relation.* Extends the
wire-form policy of 28 to 30 and the "preorder as a promise" item
(`research/README.md` §2). *Room now.* The version field and additive
keys already planned; rule names as stable strings, not serde's variant
spelling; no field whose meaning depends on the build's features.

### interop-2: Lean 4 certificates over cslib

*What.* `--format lean`: the reflective design of step 31 ported to
Lean, a checker as a Lean function proved sound against cslib's CLL
inductive, and a certificate that is the sequent, the term and
`check … = true` by `decide`. *Value.* Lean's library of computer
science lives in cslib [cslib]; a Lean user gets linear logic theorems
from an automatic prover (inference). cslib's multiset sequents drop the
exchange bookkeeping that NanoYalla's lists need (inference from
`research/31-rocq.md`). *Cost.* A second soundness proof of the same
checker, about the size of 31's first stage (inference); cslib's GitHub
page lists no releases [cslib], so the target moves.
Only classical LL exists there; ILL, affine and Mix would be
definitions of linlog's own. *Relation.* Extends 31; `plan/later.md`
defers it until after 31. *Room now.* The certificate entry should take
a target, not be Rocq's alone: `Kernel { Auto, NanoYalla, Linlog }`
generalises to an export of the term to one of several kernels, each
with its own `Unsupported`.

### interop-3: a Dedukti/Lambdapi theory of linear logic

*What.* A `.dk`/`.lp` theory declaring linlog's formulas and the
one-sided and two-sided calculi as constants (a deep encoding), and an
export of a proof as a λΠ-term over it, checked by Dedukti's checker.
*Value.* One small, maintained checker that already serves other
provers' libraries [Dedukti], and a route to further systems through
Lambdapi's exports [Lambdapi-export], without writing a kernel per
assistant (inference). A linear theory there would be new, since none
was found. *Cost.* The theory is short; the risk is the exchange rule,
which a deep encoding must make explicit or absorb by rewriting
(inference). An encoding faithful to the textbook needs a small review
by someone from Deducteam (inference). *Relation.* Beside 31: the same
proof term, a different checker. *Room now.* The exporter over the term
should be a trait or table of targets (interop-2's point), and the
derivation view should be reachable without the text layout, since this
target writes terms, not trees.

### interop-4: focused certificates with holes

*What.* A certificate of the focused engine's own shape: the decisions
(which formula is focused, how a `⊗` splits its context, which `⊕`
branch, how many copies) with the invertible phases left out, checked
by a small focused checker that redoes the forced parts, in the style of
FPC's clerks and experts [FPC17]; a reference checker in λProlog run by
ELPI [ELPI] beside the Rust one. *Value.* Certificates proportional to
the choices rather than the derivation; the focused calculus is the
standard semantics for this evidence [FPC17], [LKU]. The same record of
choices, with the memo's sharing, is the failure certificate of
`plan/later.md`'s refutation list (inference). *Cost.* The checker is
small; its soundness relative to the unfocused calculus rests on the
completeness of focusing for the fragment, which `plan/later.md` calls
the largest proof in that list. Research. *Relation.* Beside 31 (proof
terms) and extends "Certified refutations". *Room now.* The engines'
statistics and stop interface should allow a recording sink (a trait
parameter that is a no-op by default), so that recording choices does
not later need a second engine; the proof term should keep room for a
`Hole` or elided-subproof node in a non-default form, without changing
`Node`'s 16 bytes.

### interop-5: TPTP conventions in and out

*What.* Three parts. (a) `--szs`: SZS status lines on every verdict
(Theorem, CounterSatisfiable, Timeout, GaveUp) [SZS], and
the proof between `SZS output start/end` markers. (b) Reading TFF and
THF's first-order part where 38's quantifiers need typed terms [TFF],
[THF]. (c) A proposal to the TPTP of a logic specification for linear
logic in the non-classical syntax, which today covers modal logics only
[NTF]; LLTP's TPTP-like files (`research/29-comparison.md`) are the
existing practice it would standardise. Countermodels (the Kripke models
of the refutation list) written in the TPTP interpretation format,
which names Kripke interpretations [Interp]. *Value.* Tools that drive
provers by SZS status (the TPTP's own infrastructure) can call linlog
unchanged (inference); 29's comparison reads three-valued verdicts.
*Cost.* (a) is small; (b) is a parser dialect; (c) is community work
and slow. *Relation.* Extends 25 (`read_tptp`), 29 and 38; (c) is new.
*Room now.* The verdict's reasons as codes (planned) map onto SZS words
one to one only if "unknown" keeps its cause (time, memory, copy bound)
as data; TPTP options stay on `read_tptp` (`research/design-constraints.md`).

### interop-6: tactics that call linlog from a proof assistant

*What.* A Rocq tactic and a Lean tactic that take a goal stated over the
library's inductive, call the `linlog` binary (or the library through
FFI), and close the goal with the returned certificate, as `bv_decide`
does with an external SAT solver and a verified checker [bv_decide].
*Value.* Users of 31's library prove linear sequents inside their own
developments without writing derivations (inference). *Cost.* Small on
top of interop-2 and 31: the reification of the goal and a process
call; the trusted base is the checker's proof, and in Lean
`ofReduceBool` if the check is compiled [bv_decide]. *Relation.*
Extends 31 and interop-2. *Room now.* The CLI's JSON input and output
already suffice; keep `--format json` stable across patch releases and
the reading of a sequent from JSON lossless.

### interop-7: a Python package

*What.* `linlog` on PyPI, built with PyO3 and maturin as an `abi3`
wheel [PyO3], [maturin]: parse, prove with a time limit, check, the
interactive session, the exports, with options passed as the same JSON
the CLI's `--style-file` reads. *Value.* Python is where teaching
notebooks, data pipelines and LLM tooling call provers (inference);
today they must spawn the binary. *Cost.* A crate of bindings in the
workspace, wheel builds per platform in CI, and the GIL released during
a search so that Ctrl-C and threads behave; the `parallel` feature can
stay on natively (inference). *Relation.* New; beside 32 (both are
bindings over the same calls). *Room now.* Every entry point callable
with plain data and a `Stop`; no global state; errors as codes; the
`Stop` trait object-safe so that a Python callable can implement it; a
`Send` session type so that a binding can move it between threads.

### interop-8: a WebAssembly component with a WIT interface

*What.* `linlog` as a component: a WIT world with functions for
proving and checking and a resource for the interactive session,
compiled for `wasm32-wasip2` and usable from any host with a component
runtime [WASI-lang]. *Value.* One sandboxed artefact for JavaScript
through jco, Python through componentize-py hosts and Rust hosts,
without per-language native builds (inference). *Cost.* Moderate: WIT
definitions mirroring the JSON forms, and a second wasm build beside
32's wasm-bindgen one. The tooling moves fast: broad 0.3 support
across languages "is still landing" [WASI-lang], [WASIp3]. *Relation.*
Beside 32, which targets the browser through wasm-bindgen
(`research/32-web.md`). *Room now.* The library already reads no clock;
keep the 32-bit tests 32 plans, coarse calls that return whole results
(no callback in the search's hot path), and the session's JSON as its
complete state so that a resource can be rebuilt from it.

### interop-9: a language server and a tree-sitter grammar

*What.* A tree-sitter grammar for linlog's sequent syntax, the batch
`lines` format and interactive scripts, for highlighting in Neovim,
Helix, Zed and Emacs [tree-sitter]; and `linlog lsp` built on
`lsp-server` [lsp-server]: parse errors as diagnostics, the fragment
and the intuitionistic reading on hover, "prove", "open in interact"
and "export" as code actions, verdicts as inlay hints within a short
limit. *Value.* People who keep files of sequents (teaching, the
benchmark families, LLTP-style problem sets) edit them with feedback
(inference). *Cost.* A grammar (small), a server command in the CLI
crate (moderate), a client extension per editor (small each). *Relation.*
New; it reuses 24's batch formats and 22's options. *Room now.* Parse
errors with spans in UTF-16 or documented bytes (planned,
`research/README.md` §2); a parser that recovers and returns several
errors; spans for every subformula, kept beside the forest so that an
occurrence maps back to text.

### interop-10: notebooks

*What.* A Jupyter widget built with anywidget [anywidget]: a sequent in,
the derivation's SVG out, and the interactive session clickable through
the SVG ids that 32 defines; the Python side from interop-7, the
JavaScript side reusing 32's client code. In JupyterLite [Pyodide] the
widget could call 32's wasm module from JavaScript with no Python build
at all (inference). *Value.* Course material where students prove step
by step inside the notes they read (inference). *Cost.* Small once 32
and interop-7 exist. *Relation.* Extends 32 and 22 (`plan/later.md`
names notebooks as a consumer of the options' JSON). *Room now.* 32's
click-target ids (`i<n>`, `o<n>`, `l<m>-<n>`, per-formula ids) as a
documented contract, and the session's operations as data (an `apply`
request in JSON) so that two front ends drive it the same way.

## 3. Considered and rejected

- **Alethe as an output.** Its term language is SMT-LIB's and its rules
  are SMT reasoning [Alethe]; linear rules would be unknown to Carcara,
  which refuses unknown rules by default (not in the source) [Carcara]. Kept: the idea of
  coarse steps elaborated by the checker, inside interop-4.
- **LFSC.** Built for SMT proof systems and the solvers clsat and CVC3
  [LFSC]; Dedukti fills the "one framework, many logics" role with a
  record of foreign libraries checked [Dedukti] (inference).
- **Isabelle export.** The only linear logic found is the `Sequents`
  ILL object logic of 1995 [Sequents], not Isabelle/HOL, and no AFP
  entry; a certificate there would have few readers (inference). Through
  interop-3, Deducteam's Isabelle component is the other direction
  [Deducteam-repos].
- **Agda export.** No maintained linear logic library was found, only
  research developments [Kokke], (`research/29-comparison.md`, 19).
- **OpenTheory.** A framework for sharing proofs among HOL provers
  [FPC13]; linear logic has no place in it (inference).
- **A Jupyter kernel for linlog's syntax.** A widget over the Python
  package (interop-10) gives the same with Python's ecosystem around it
  (inference).
- **A Python wheel for Pyodide.** Not verified that PyO3 and maturin
  build for Pyodide's target; the JavaScript route of interop-10 avoids
  the question (inference).
- **tower-lsp.** Its fork reports that development stopped [lspower];
  `lsp-server` is maintained in rust-analyzer's repository [lsp-server].

## 4. What the API design should decide now

In order of how many candidates lean on it (inference, from section 2):

1. The read-back forms as a specification with a schema and stable rule
   names (interop-1 to 6, 10).
2. The certificate export as a list of targets over the proof term,
   each with its own refusals (interop-2, 3, 6).
3. Entry points that take plain data and an object-safe `Stop`, with a
   `Send` session and no global state (interop-7, 8, 10).
4. Parser spans per subformula, recovery, and UTF-16 or byte positions
   documented (interop-9).
5. A recording hook in the engines, a no-op by default (interop-4).
6. Unknown verdicts with their cause as data (interop-5).

## 5. Sources

- [Dedukti] A. Assaf, G. Burel, R. Cauderlier, D. Delahaye, G. Dowek,
  C. Dubois, F. Gilbert, P. Halmagrand, O. Hermant, R. Saillard,
  "Dedukti: a Logical Framework based on the λΠ-Calculus Modulo
  Theory", manuscript 2016, arXiv 2023. https://arxiv.org/abs/2311.07185
- [Lambdapi-export] Deducteam, lambdapi, API documentation of
  `lambdapi.export` (2.5.1, 3.0.0), ocaml.org.
  https://ocaml.org/p/lambdapi/3.0.0/lambdapi.export/Export/index.html
- [hol2dk] Deducteam, hol2dk README, GitHub.
  https://github.com/Deducteam/hol2dk/blob/main/README.md
- [Deducteam-repos] Repository listing of the Deducteam organisation
  (isabelle_dedukti, lean2dk, Logipedia), code.gouv.fr.
  https://ecosystem.code.gouv.fr/hosts/GitHub/owners/Deducteam
- [FPC13] Z. Chihani, D. Miller, F. Renaud, "Checking foundational
  proof certificates for first-order logic (extended abstract)", PxTP
  2013. https://www.lix.polytechnique.fr/~dale/papers/checking-fpc.pdf
- [FPC17] Z. Chihani, D. Miller, F. Renaud, "A Semantic Framework for
  Proof Evidence", J. Automated Reasoning 59(3), 2017, 287–330.
  https://hal.archives-ouvertes.fr/hal-01390912
- [LKU] C. Liang, D. Miller, "A focused approach to combining logics",
  Annals of Pure and Applied Logic 162(9), 2011, 679–697.
  https://www.lix.polytechnique.fr/~dale/papers/lku.pdf
- [ELPI] C. Dunchev, C. Sacerdoti Coen, E. Tassi et al., "ELPI: Fast,
  Embeddable, λProlog Interpreter", LPAR-20, 2015 (the authors named
  are those the University of Bologna's record confirms).
  https://doi.org/10.1007/978-3-662-48899-7_32 ; https://cris.unibo.it/handle/11585/552439
- [LFSC] A. Stump, D. Oe, A. Reynolds, L. Hadarean, C. Tinelli, "SMT
  proof checking using a logical framework", Formal Methods in System
  Design 42(1), 2013, 91–118.
  https://homepage.cs.uiowa.edu/~tinelli/papers/StuEtAl-FMSD-13.pdf
- [Alethe] H.-J. Schurr, M. Fleury, H. Barbosa, P. Fontaine, "Alethe:
  Towards a Generic SMT Proof Format (extended abstract)", PxTP 2021,
  EPTCS 336, 49–54. https://arxiv.org/abs/2107.02354
- [Carcara] B. Andreotti, H. Lachnitt, H. Barbosa, "Carcara: An
  Efficient Proof Checker and Elaborator for SMT Proofs in the Alethe
  Format", TACAS 2023; code. https://link.springer.com/chapter/10.1007/978-3-031-30823-9_19 ;
  https://github.com/ufmg-smite/alethe-proof-checker
- [bv_decide] Lean FRO, Lean 4.12.0 release notes, Lean Language
  Reference, 2024. https://lean-lang.org/doc/reference/latest/releases/v4.12.0/
- [cslib] leanprover/cslib, GitHub, Apache-2.0, directory
  `Cslib/Logics/LinearLogic/CLL` (read 2026-10-09).
  https://github.com/leanprover/cslib/tree/main/Cslib/Logics/LinearLogic/CLL
- [Sequents] Isabelle, `Sequents` README (Kalvala; Kalvala and de Paiva,
  "Linear Logic in Isabelle", 1995).
  https://isabelle.in.tum.de/website-Isabelle2011/dist/library/Sequents/README.html
- [Kokke] W. Kokke, SubstructuralLogicsInAgda, GitHub.
  https://github.com/wenkokke/SubstructuralLogicsInAgda
- [TFF] G. Sutcliffe, S. Schulz, K. Claessen, P. Baumgartner, "The TPTP
  Typed First-order Form with Arithmetic", LPAR-18, 2012, LNCS 7180,
  406–419. https://doi.org/10.1007/978-3-642-28717-6_32
- [THF] G. Sutcliffe, C. Benzmüller, "Automated Reasoning in
  Higher-Order Logic using the TPTP THF Infrastructure", J. Formalized
  Reasoning 3(1), 2010, 1–27. https://jfr.unibo.it/article/view/1710
- [NTF] A. Steen, D. Fuenmayor, T. Gleißner, G. Sutcliffe, C.
  Benzmüller, "Automated Reasoning in Non-classical Logics in the TPTP
  World", PAAR 2022, arXiv:2202.09836; scope as of 2025 from A. Steen et
  al., arXiv:2508.09318. https://arxiv.org/abs/2202.09836 ;
  https://arxiv.org/html/2508.09318v1
- [Interp] G. Sutcliffe, A. Steen, P. Fontaine, L. Kondylidou, "The
  TPTP Format for Interpretations", arXiv:2406.06108, 2024 (v2 2026).
  https://arxiv.org/abs/2406.06108
- [SZS] G. Sutcliffe, "The SZS Ontologies for Automated Reasoning
  Software", KEAPPA 2008; SZS ontology page.
  https://www.cs.miami.edu/home/geoff/Papers/Conference/2008_Sut08_KEAPPA-38-49.pdf ;
  https://tptp.org/UserDocs/SZSOntology/
- [TSTP] G. Sutcliffe, J. Zimmer, S. Schulz, "Communication Formalisms
  for Automated Theorem Proving Tools", 2003; TSTP library page.
  https://www.cs.miami.edu/home/geoff/Papers/Conference/2003_SZS03_AAR-52-57.pdf ;
  https://tptp.org/TSTP/
- [PyO3] PyO3 user guide, "Building and distribution", 0.29.2, docs.rs.
  https://docs.rs/crate/pyo3/0.29.2/source/guide/src/building-and-distribution.md
- [maturin] maturin 1.15.0, docs.rs. https://docs.rs/crate/maturin
- [WASIp3] WASI, "WASI 0.3" release page (read 2026-10-09).
  https://wasi.dev/releases/wasi-p3
- [WASI-lang] WASI, "Languages" (read 2026-10-09).
  https://wasi.dev/languages
- [lsp-server] lsp-server 0.10.0, docs.rs (2026-07-16).
  https://docs.rs/crate/lsp-server/latest
- [lspower] lspower 1.5.0 documentation, docs.rs.
  https://docs.rs/crate/lspower/latest
- [tree-sitter] "Tree-sitter (parser generator)", Wikipedia.
  https://en.wikipedia.org/wiki/Tree-sitter_(parser_generator)
- [Pyodide] Pyodide documentation, "Related projects" (JupyterLite).
  https://pyodide.org/en/314.0.6/_sources/project/related-projects.md
- [anywidget] anywidget, PyPI. https://pypi.org/project/anywidget/
- Repository (this snapshot): `plan/later.md`;
  `plan/notes/research/README.md`, `29-comparison.md`, `31-rocq.md`,
  `32-web.md`, `design-constraints.md`.

Sources checked 2026-10-09: 31 checked, 2 corrected, 0 removed, 3 claims marked.
