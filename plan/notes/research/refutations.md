# Certified refutations: research note

Written 2026-10-08 for step 31 item 8 and the escalations under
"Certified refutations" in `plan/later.md`. Repository files are cited
by path (R), everything else by the numbered sources of section 5 (S).
Claims without a source are marked "(inference)".

## 1. The problem and the state of the art

**The statement.** A certified refutation is a lemma proved once over
the library's calculus, a small certificate, and a check by computation
that the calculus does not derive the sequent; the statement is about
the inductive, never a negation over `Prop`, which Rocq cannot prove
for a classical theorem such as excluded middle [R1, R2]. What can be
certified depends on the fragment: full propositional LL is undecidable
and MALL is PSPACE-complete [S1]; full propositional affine LL is
decidable [S2]; MELL's decidability is open, a claimed proof having
been refuted [S3]. So every method below is stated for a fragment or
rests on a bound proved sufficient [R1].

**Finite phase models.** Finite phase semantics is complete for MALL
and for affine LL (LLW), not for MELL [S4, S5]; Okada and Terui extend
the finite model property to IMALL and ILLW [S6]. Lafont's construction
uses "enriched phase spaces" (a phase space with a submonoid `K`,
`!X = (X ∩ K)^⊥⊥`), which "allows to define quotients with good
properties"; the announcement states no size bound, and I could not
read the paper [S5]. I found no tool that searches for finite phase
countermodels; the provers found decide provability and emit proofs or
Yalla certificates [S7].

**Kripke and resource semantics.** For LL without exponentials, Allwein
and Dunn give Kripke-style models with three-valued valuations, built on
Urquhart's lattice representations [S8]; Urquhart's semilattice
semantics is the origin of the "pieces of information" reading [S9];
Pym, O'Hearn and Yang's resource semantics of BI is on preordered
commutative monoids, Petri nets among its models [S10]. None comes with
a countermodel search I could find (inference from the searches run).
For LJ the situation is mature: G4ip (Dyckhoff's LJT) terminates
without a loop check by a multiset ordering, and cut and contraction
are admissible in it [S11, S12]; Dyckhoff and Pinto implemented a
loop-free countermodel construction [S13]; the SAT-based provers intuit
and intuitR return a Kripke countermodel on failure, intuitR's being
"in general small" [S14, S15]. In Rocq, Férée and van Gool formalised
Pitts' propositional quantifiers by induction over LJT and extracted an
OCaml program [S16, S17]; a Saarland development proves Kripke and
Heyting semantics sound for the implication–falsum fragment [S18];
`itauto` is a reflexive intuitionistic SAT solver in Rocq [S19].

**Failure certificates with a verified checker.** SAT solvers certify
UNSAT in LRAT, which has certified checkers in Coq and ACL2 [S20];
cake_lpr is verified down to machine code [S21]. Heath and Miller ground
model checking, non-reachability included, in focused proofs with fixed
points, synthetic rules replacing infinitely many premises [S22]. The
completeness of focusing for LL is mechanised in Coq twice: `coq-ll`
(propositional and first-order LL, cut elimination, completeness of the
focused system; tested with Rocq 9.2; GPL-3.0 in the sidebar, LGPL in
the README) [S23, S24], and Felty and coauthors for the dyadic system
[S25].

**Farkas and integer certificates for Petri nets.** The state equation
`M − M₀ = C·x`, `x ≥ 0` is necessary for reachability; its rational
infeasibility has a Farkas vector as witness, which the Horn engine
computes and checks exactly [R3]. Esparza and Melzer strengthen it with
traps [S26]; Esparza et al. solve equation plus traps by SMT and
extract inductive invariants [S27]; Blondin et al. prune backward
coverability by continuous reachability (qcover, Apache-2.0, Python
2.7) [S28, S29]. Where the rational relaxation is feasible but no
integer solution exists, modulo-`k` invariants prove exactly integer
infeasibility, and a finite set of place and modulo invariants suffices
[S30]; parity is `k = 2`. For general integer infeasibility,
cutting-plane proofs are complete for rational polyhedra (Chvátal,
Schrijver) and fit in polynomial space [S31]; VIPR is a certificate
format for MIP infeasibility with checkers and an SMT-verified schema
[S32, S33].

**Checking in Rocq.** Micromega's `lia` combines Positivstellensatz
refutations, cutting planes and case splits, "a complete proof
principle for integer linear arithmetic", the oracle's certificate
normalised by `ring`; `lra` is the rational case [S34]. Large
certificates go through a checker written as a Rocq function, as the
LRAT checkers do [S20].

**Libraries linlog touches.** Yalla 2.0.7 (27 Mar 2025, LGPL-3.0, Rocq
9.0 to 9.2) [S35, S36]; NanoYalla ships inside Click & coLLecT
(LGPL-2.1, OCaml backend) [S37].

## 2. What the step needs

**Group A, the cheap invariants (step 31 item 8).** Each is a lemma by
induction on the derivation inductive plus a function the kernel
computes [R2].

- *Unbalanced*: `interval : formula → atom → Z × Z` (literals `±1`,
  `⊗`/`⅋` sum, `&`/`⊕` hull, `⊤` absorbs) and the lemma that a
  derivable sequent's summed interval contains zero, for
  exponential-free formulas without weakening, the conditions
  `Rules::intervals` applies [R4]. The Rocq function may take the
  intersection for `&`: sound too, and it refutes whatever the hull
  refutes (inference). Certificate: the atom. Cost: one pass.
- *Equation*: `formulas = tensors − pars − ones + bottoms + 2` (`≥`
  with Mix), sound only without additives, additive units,
  exponentials and weakening [R4]. Certificate: nothing beyond the
  sequent and the mode.
- *The classical reading*: erase `!`/`?`, read `⊗`,`&` as and,
  `⅋`,`⊕` as or, `1`,`⊤` true, `0`,`⊥` false; every rule, Mix and
  weakening included, is classically sound, so a falsifying assignment
  refutes in every mode [R2, R5]. Certificate: `atom → bool`; the
  checker evaluates. The search is a truth table today [R5]; recommend
  a small DPLL over the erased formula with an atom count and a work
  bound as options (D16), since a truth table is `2ⁿ`.
- *Classical "not valid" over `Prop`*: the same assignment,
  instantiating atoms with `True`/`False` to prove `~ (forall …, F)`,
  beside `ordinary::rocq`'s positive certificate [R2]. No library.

**Group B, finite phase models (MALL, affine, their intuitionistic
versions).** Certificate: a finite commutative monoid as an `n × n`
table with unit, the fact `⊥` as a bitvector, each atom's fact; the
checker computes every formula's fact bottom-up with closures `X^⊥⊥`
(`O(n²)` per node) and checks the unit outside the sequent's `⅋`.
Soundness is the short proof [R1]; trivial phase semantics of ILL is
already sound in Coq [S38]. The open part is the search: no tool
[S7], no size bound read [S5], and the quotient of the syntactic model
may be large (inference). Recommend: a prototype that enumerates small
monoids under a size bound on the refuted MALL targets and reports how
often one is found; no step before that number exists.

**Group C, Kripke countermodels for LJ (the ordinary layer).**
Certificate: worlds, a preorder, a monotone valuation, a root; the
checker evaluates the forcing clauses and checks every hypothesis
forced and the conclusion not (`O(worlds² × formula)`). Soundness is a
short induction over LJ (inference; [S18] has it for a fragment).
linlog's route through ILL produces no model, so a G4ip search with
countermodel extraction [S11, S13] or an intuit-style SAT loop [S14,
S15] is needed; recommend G4ip, the loop check's alternative that
would settle the 35 ILTP non-theorems of step 25's report [R6], with
textbook termination [S11]. Minimal logic makes `false` an atom [R7],
so a model treating it as one is the certificate (inference).
Countermodels for ILL itself: no search exists (section 1); defer.

**Group D, failure certificates from the focused engine.** Certificate:
the DAG of stable sequents visited, memo sharing kept, each with its
synchronous candidates and per candidate the failing premise and its
reason (a child node, a count prune, a bound) [R1]. The checker
re-enumerates the candidates and checks each failure. Two costs: the
splits of a `⊗` or Mix are `2ⁿ` unless the certificate carries the
prune that cut them, so group A's prunes become part of the checked
calculus; and soundness needs the completeness of the focused calculus
for the fragment, mechanised for other presentations [S23, S25] but not
for linlog's system with its copy bound. Recommend: MALL and affine
first, a Rust payload and a Rust checker independent of the engine
before any Rocq, and certificate sizes measured on `bench/targets`;
with exponentials only where a bound is proved sufficient [R1].

**Group E, Petri nets.** The Farkas vector exists [R3]. The lemma is
`core-horn.md`'s prose argument as a Rocq theorem: a proof of a Horn
program is a firing sequence, by induction on the dyadic proof [R3].
Three facts about the emitted certificate must change first: `certify`
sees only the transitions `live` kept, so the certificate rests on the
dead-transition argument [R3]; `Refutation::StateEquation` folds the
class places' weights (clauses used once) into `once: bool` [R8]; the
place indices are not in it. Recommend: every place's weight, the class
structure as occurrence ids, and either weights valid over all
transitions or the closed set of reachable places (the checker verifies
closure and that each dropped transition has an input outside it). For
the integer gap: a modulo-`k` invariant `y, k` with `y·Cₜ ≡ 0 (mod k)`
for all `t` and `y·(M − M₀) ≢ 0` is a second certificate kind, checked
by computation and complete with Farkas for integer infeasibility
[S30]; recommend it before cutting planes, whose derivations are
longer and need a VIPR-like format [S32]. A small concrete system can be
left to `lia` [S34]; hundreds of variables are a risk (inference).

## 3. What linlog's library must offer

**Data model.** `Sequent` (`core/src/sequents/mod.rs`): a one-sided NNF
arena, atoms by index, intuitionistic sequents by `Reading` on the same
model [R9]. A Rocq certificate needs the sequent printed as the
library's own formula type; `export/rocq.rs` prints NanoYalla's through
`identifiers` and the `Walk` printer, `notation::ill` prints two-sided
text [R10]. Missing: a printer to the new library's formulas, two-sided
for the ILL statement, and a place for terms (D17) in every certificate
datatype. The written succedent among `⊤`/`0` roots is by id, which the
two-sided statement inherits [R11].

**The refutation type.** `search::Refutation` (`core/src/search/mod.rs`,
`#[non_exhaustive]`): `Exhausted`, `Unbalanced {atom, name, least,
most}`, `Equation {formulas, tensors, pars, ones, bottoms, mix}`,
`StateEquation {weights: Vec<(String, i64)>, once}`; `Display` is what
the CLI prints, the JSON a write-only proxy `WhyNot` in
`serialize/search.rs` [R8, R12]. Missing: the class weights and live
set (section 2, E); a `Classical { assignment }` variant computed in
the library behind an option rather than in the exporter as step 31
words it, so CLI, web and JSON share it (D15); later `Failure`,
`PhaseModel`, `KripkeModel` as named payload structs with serde, which
the Rocq datatypes mirror; and reading: `Outcome` serialises only
[R12], so no refutation file exists for `linlog check` to re-check.

**A checker for refutations.** Only `horn::equation::certify`
(`pub(super)`) checks a refutation; `focus::refutation` recomputes the
engine's `Counts` [R3, R13]. The proof checker's rule, no code shared
with an engine [R14], should hold here: `Refutation::check(&forest,
goal, mode, fragment)` in a module of its own, the interval and count
functions written again, the Petri-net translation a pure function the
engine and the checker both call, the Rocq checker its mirror.

**Engine interface and dispatch.** `Decide::decide` returns `Answer`
with `refutation: Option<Refutation>`; `prove_goal` turns `None` into
`focus::refutation`'s, the one place an answer becomes a `Verdict`
[R13]. A countermodel search decides nothing by itself, so it is no
engine; yet a found countermodel is a verdict even when the search was
`Unknown(CopyBound)`, the case of the 35 ILTP problems [R6]. The front
door needs a second plug-in kind: refuters run after an `Unprovable`
(to say why) and, by option, beside or after an `Unknown` (to decide),
under the same stop and account, each a variant with a row like
`DISPATCH` [R13].

**Proof term and checker.** `Proof`/`Node` (`proofs/mod.rs`) and
`check.rs` are what the Rocq library verifies [R14, R2]; the refutation
lemmas are inductions on the derivation inductive, the Horn lemma on
the dyadic calculus with `Quest`/`Copy` as the term has them [R3, R14].
Nothing to add to the term.

**Exports and options.** `export::rocq::{Options {form, lemma,
prelude}, write, derivation, ordinary}` take a `Derivation` and refuse
Mix, affine weakening, open goals and compact runs [R10]. Missing:
`rocq::refutation(&sequent, mode, &refutation, &options, out, stop)`,
a kernel choice in `Options` (step 31 item 6), `Unsupported::
NoCertificate` for `Exhausted`, and `ordinary::rocq`'s negative
counterpart taking the assignment. The CLI makes a document only "with
something in it beyond the verdict" [R15]; a certificate counts, so
`Show::holds` changes and README with it.

**Options.** `search::Options` has private fields and setters [R13];
add the refuters and their bounds (atoms and work for the assignment
search, worlds, monoid size), every one a named default (D16).
`batch.rs` reduces a refutation to `Status::No` and should keep the
payload.

## 4. Risks, open questions, and what the prompt should add

- With exponentials nothing is complete [S1, S2, S3]; the prompt must
  state per certificate kind its fragment and mode, and that
  `Exhausted` stays uncertified.
- Phase models: no size bound read, no search exists [S5, S7]. Open:
  how large the models of the refuted MALL targets are; measure before
  any Rocq work.
- Kripke for ILL: the completeness results are for phase models [S6];
  open whether an ILL countermodel search is worth building.
- Group D's soundness needs focusing completeness for linlog's dyadic
  system with its prunes and copy bound; the mechanised proofs are for
  other presentations [S23, S25]. Open: prove it, or certify through a
  translation into `coq-ll`'s system.
- Group E: the dead-transition set and the class weights are not in the
  emitted certificate [R3, R8]; fixing it changes the JSON form of
  `state_equation`, free before 0.1.0 and a version after (D18).
  `certify`'s `i128` bounds become `Z` lemmas or nothing [R2].
- Rocq checking time on large certificates is unmeasured (inference).
- Kanovich's Horn paper is cited as 1995 in `core-horn.md` [R3]; the
  journal volume is 1994 [S39].
- The prompt should add: the Rust-side checker as each kind's first
  deliverable, with a panel on the lemma's statement; the payload
  structs and their JSON pinned in `core/tests/serialize.rs`; the counts
  of what each kind certifies on the target sets (the nineteen LLTP
  files by the classical reading [R5], qcover's refutations by Farkas,
  the ILTP 35 by a G4ip step); the order A, E, C, then D and B as
  research; and the rule that a refuter never changes a verdict the
  search gave, only `Unknown` into `Unprovable` with a checked
  certificate.

## 5. Sources

Repository (paths in the snapshot):

- R1. `plan/later.md`, "Certified refutations".
- R2. `plan/31-rocq-library.md`, items 3, 7, 8.
- R3. `.claude/rules/core-horn.md`; `core/src/search/horn/equation.rs`
  (`certify`), `horn/mod.rs` (`Program`, `live`, `refutation`).
- R4. `.claude/rules/core-focus.md`, "Counts", "Affine mode".
- R5. `plan/notes/lltp-headers.md`.
- R6. `plan/reports/25-ordinary-logic.md`, "Termination on dyadic
  sequents".
- R7. `.claude/rules/core-ordinary.md`.
- R8. `core/src/search/mod.rs`: `Refutation`, `Verdict`, `Outcome`.
- R9. `.claude/rules/core-sequents.md`.
- R10. `.claude/rules/core-export.md`, "Rocq"; `core/src/export/rocq.rs`.
- R11. `plan/later.md`, "Follow-ups: intuitionistic mode".
- R12. `core/src/serialize/search.rs`; `core-sequents.md`,
  "Serialization".
- R13. `.claude/rules/core-search.md`; `core/src/search/focus/mod.rs`
  (`refutation`).
- R14. `.claude/rules/core-proofs.md`.
- R15. `.claude/rules/cli.md`.

Literature and software:

- S1. Lincoln, Mitchell, Scedrov, Shankar, "Decision problems for
  propositional linear logic", APAL 56, 1992.
  https://www.csl.sri.com/papers/lmss90/lmss90.pdf
- S2. Kopylov, "Decidability of linear affine logic", Information and
  Computation 164(1), 2001 (LICS 1995).
  https://www.cs.cornell.edu/people/kopylov/papers/llw/index.htm
- S3. Straßburger, "On the decision problem for MELL", TCS 768, 2019.
  https://www.lix.polytechnique.fr/Labo/Lutz.Strassburger/papers/OnDeciMELL.pdf
- S4. Lafont, "The finite model property for various fragments of
  linear logic", JSL 62(4), 1997.
  https://www.cambridge.org/core/journals/journal-of-symbolic-logic/article/abs/finite-model-property-for-various-fragments-of-linear-logic/F75D11ED6907E37EA0424226108DDA66
- S5. Lafont, announcement of S4, Types Forum, 3 Nov 1995.
  https://www.engineering.upenn.edu/~sweirich/types/archive/1995/msg00160.html
- S6. Okada, Terui, "The finite model property for various fragments
  of intuitionistic linear logic", JSL 64(2), 1999.
  https://resolve.cambridge.org/core/journals/journal-of-symbolic-logic/article/abs/finite-model-property-for-various-fragments-of-intuitionistic-linear-logic/6D066A0EFD6AB6E0A9EC501D170E9BDD
- S7. Web search of 2026-10-08 for phase-model countermodel tools; the
  nearest found is the OCaml prover APLL, which emits Yalla
  certificates. https://github.com/wujuihsuan2016/LL_prover
- S8. Allwein, Dunn, "Kripke models for linear logic", JSL 58(2), 1993.
  https://philpapers.org/rec/ALLKMF
- S9. Urquhart, "Semantics for relevant logics", JSL 37, 1972.
  https://www.cambridge.org/core/journals/journal-of-symbolic-logic/article/abs/semantics-for-relevant-logics/B657FA09114501CF5765EDF2D9409F8E
- S10. Pym, O'Hearn, Yang, "Possible worlds and resources: the
  semantics of BI", TCS 315(1), 2004.
  https://researchportal.bath.ac.uk/en/publications/possible-worlds-and-resources-the-semantics-of-bi/
- S11. Dyckhoff, "Contraction-free sequent calculi for intuitionistic
  logic", JSL 57(3), 1992, and its 2018 correction (JSL 83(4)).
  https://research-repository.st-andrews.ac.uk/bitstream/handle/10023/16793/Dyckhoff_2018_contraction_free_JSL_1680.pdf
- S12. Dyckhoff, Negri, "Admissibility of structural rules for
  contraction-free systems of intuitionistic logic", JSL 65, 2000.
  https://cambridge.org/core/journals/journal-of-symbolic-logic/article/abs/admissibility-of-structural-rules-for-contractionfree-systems-of-intuitionistic-logic/BDC66A51FC0EC10130EB269CEF0A7BAF
- S13. Dyckhoff, Pinto, "Implementation of a loop-free method for
  construction of counter-models for intuitionistic propositional
  logic", St Andrews research portal, 1996.
  https://research-portal.st-andrews.ac.uk/en/publications/implementation-of-a-loop-free-method-for-construction-of-counter-/
- S14. Claessen, Rosén, "SAT modulo intuitionistic implications", LPAR
  2015. https://research.chalmers.se/en/publication/?id=230822
- S15. Fiorentini, "Efficient SAT-based proof search in intuitionistic
  propositional logic", CADE 28, 2021,
  https://air.unimi.it/handle/2434/855762 ; intuitR (Haskell,
  BSD-3-Clause, no release shown) https://github.com/cfiorentini/intuitR
- S16. Férée, van Gool, "Formalizing and computing propositional
  quantifiers", CPP 2023, DOI 10.1145/3573105.3575668.
  https://dblp1.uni-trier.de/rec/conf/cpp/FereeG23.html
- S17. Férée, van Gool, talk abstract, IRIF types seminar, 2022.
  https://www.irif.fr/seminaires/types/types2022
- S18. Saarland University, "Heyting algebras and Kripke models", Coq
  development. https://ps.uni-saarland.de/HeytingKripke/description.html
- S19. Besson, "Itauto: an extensible intuitionistic SAT solver", ITP
  2021, https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.ITP.2021.9 ;
  package `coq-itauto` 9.1.0, 22 Jul 2026, https://rocq-prover.org/p/coq-itauto
- S20. Cruz-Filipe, Heule, Hunt, Kaufmann, Schneider-Kamp, "Efficient
  certified RAT verification", CADE 2017.
  https://arxiv.org/pdf/1612.02353
- S21. Tan, Heule, Myreen, "cake_lpr: verified propagation redundancy
  checking in CakeML", TACAS 2021.
  https://www.springerprofessional.de/en/cake-lpr-verified-propagation-redundancy-checking-in-cakeml/18996008
- S22. Heath, Miller, "A proof theory for model checking", JAR 63(4),
  2019. https://www.lix.polytechnique.fr/~dale/papers/ptmc-final.pdf
- S23. Xavier, Olarte, Reis, Nigam, "Mechanizing focused linear logic
  in Coq", ENTCS 2018.
  https://repositorio.ufrn.br/items/ff81affa-bc24-484e-9aae-007f537c19f7
- S24. `coq-ll` repository (GPL-3.0 sidebar, LGPL README, tested with
  Rocq 9.2, no release). https://github.com/meta-logic/coq-ll
- S25. Felty and coauthors, cut elimination and completeness of
  focusing for the dyadic system, MSCS 2021.
  https://www.site.uottawa.ca/~afelty/dist/mscs21.pdf
- S26. Esparza, Melzer, "Verification of safety properties using
  integer programming: beyond the state equation", FMSD 16, 2000.
  https://archive.model.in.tum.de/um/bibdb/info/esparza.EM00.shtml.html
- S27. Esparza, Ledesma-Garza, Majumdar, Meyer, Niksic, "An SMT-based
  approach to coverability analysis", CAV 2014.
  https://archive.model.in.tum.de/um/courses/petri/SS2017/material/cav2014-paper.pdf
- S28. Blondin, Finkel, Haase, Haddad, "Approaching the coverability
  problem continuously", TACAS 2016. https://arxiv.org/abs/1510.05724
- S29. qcover repository (Apache-2.0, Python 2.7, no release).
  https://github.com/blondimi/qcover
- S30. Desel, Neuendorf, Radola, "Proving nonreachability by
  modulo-invariants", TCS 153(1–2), 1996.
  https://ftp.math.utah.edu/pub/tex/bib/idx/tcs1995/153/1/49-64.html
- S31. Cook, "Cutting-plane proofs in polynomial space" (citing Chvátal
  1973, Discrete Mathematics 4, and Schrijver 1980, Annals of Discrete
  Mathematics 9). https://math.uwaterloo.ca/~bico/papers/cpspace.pdf
- S32. Cheung, Gleixner, Steffy, "Verifying integer programming
  results", IPCO 2017; VIPR specification and checkers (C++).
  https://github.com/scipopt/vipr
- S33. Wood et al., "Satisfiability modulo theories for verifying MILP
  certificates", arXiv 2312.10420. https://arxiv.org/abs/2312.10420
- S34. Rocq reference manual, "Micromega".
  https://rocq-prover.org/refman/addendum/micromega.html
- S35. Rocq package `rocq-yalla` 2.0.7, 27 Mar 2025.
  https://rocq-prover.org/p/rocq-yalla
- S36. Laurent, Yalla repository (LGPL-3.0; `microyalla` kernel).
  https://github.com/olaure01/yalla
- S37. Click & coLLecT repository (LGPL-2.1; `nanoyalla/`).
  https://github.com/etiennecallies/click-and-collect
- S38. Forster, Larchey-Wendling, "Certified undecidability of
  intuitionistic linear logic via binary stack machines and Minsky
  machines", CPP 2019 (soundness of ILL for trivial phase semantics).
  https://www.ps.uni-saarland.de/Publications/details/ForsterLarchey-Wendling:2018:Undecidability-ILL.html
- S39. Kanovich, "The complexity of Horn fragments of linear logic",
  APAL 69(2–3), 1994.
  https://api.openalex.org/works/doi:10.1016%2F0168-0072%2894%2990085-X

Sources checked 2026-10-08: 39 checked, 1 corrected, 0 removed, 0 claims marked.
