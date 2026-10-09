# Semantics and decision problems: where linlog's scope could grow

Written 2026-10-09 as a scope study beside steps 29 to 38. Repository
files are cited by path (R), everything else by the numbered sources
of section 5 (S); "(inference)" marks a conclusion of this note's own.
The web-search budget ran out after two rounds; one claim rests on an
indexed abstract I could not open and says so.

## 1. The field as it stands

**Decision problems of linear logic.** Full propositional LL is
undecidable and MALL PSPACE-complete [S1]; affine LL (LLW) is decidable
[S2]; MELL is open, a claimed proof refuted [S3]. MELL provability is
equivalent to reachability in vector addition tree automata, that is
branching VASS [S4, S5]. Lazić and Schmitz show MELL Tower-hard already
in the affine case, full propositional affine LL Tower-complete, and
contractive LL Ackermann-complete [S6]; Schmitz shows implicational
relevance logic 2-EXPTIME-complete by two reductions to and from BVASS
coverability [S7], after Urquhart's non-primitive-recursive lower bound
for the implication–conjunction fragments of R, E and T [S8]. Kanovich's
Horn fragments: Horn, ⊕-Horn and &-Horn are NP-complete, (⊕,&)-Horn
PSPACE-complete, !-Horn and (!,&)-Horn equivalent to Petri-net
reachability, (!,⊕)-Horn undecidable [S9, S10]. linlog's Horn engine is
the !-Horn case [R5].

**Phase semantics and finite models.** Finite phase semantics is
complete for MALL and LLW but not for MELL, which gives a second proof
of LLW's decidability [S11]; Okada and Terui extend it to the
intuitionistic fragments [S12]; Galatos and Jipsen's residuated frames
give the finite model property, hence decidability, for involutive
FL-algebras and related varieties, the algebraic home of substructural
logics [S13]. In Rocq, Yalla 2.0.7 lists phase semantics as ongoing
work [S14], Larchey-Wendling mechanised cut elimination through a
relational phase semantics [S15], and Forster and Larchey-Wendling
proved ILL sound for trivial phase semantics [S16]. No tool searches
for phase countermodels [S17, R3] (not in the source: S17).

**VASS reachability since 2019.** The problem is not elementary (STOC
2019, JACM 2021) [S18]; Leroux and Schmitz gave an Ackermann upper
bound and primitive-recursive bounds in fixed dimension (LICS 2019)
[S19]; Czerwiński and Orlikowski, and Leroux independently, matched it:
reachability is Ackermann-complete, F_k-hard in dimension 6k (FOCS 2021)
[S20, S21]; the dimension bound is now F_d-hardness in dimension 2d+3
(FSTTCS 2023) [S22]. Low dimensions: 2-VASS is PSPACE-complete in
binary [S23], NL-complete in unary [S23a]; 3-VASS is in doubly exponential
space (ICALP 2025), between PSPACE-hard and Tower [S24]. Coverability is
EXPSPACE-complete (Lipton 1976, Rackoff 1978), and Künnemann et al.
made the bound tight, with a conditionally optimal n^(2^O(d)) algorithm
[S25]. Certificates: a target is unreachable exactly when a
semilinear (Presburger) inductive invariant contains the source and
excludes the target, which is decidable to check, so reachability is
decided by two semi-algorithms, one for runs and one for invariants
[S26, S27].

**Tools.** KReach implements Kosaraju's 1982 decision procedure in
Haskell, competitive for coverability by reduction [S28]; FastForward
does directed (heuristic) reachability [S29]; QCover prunes the
backward coverability algorithm by continuous reachability, ICover by
the state equation and sign analysis [S30, S31]; Petrinizer checks
coverability by SMT over the state equation and traps, an incomplete
technique that discharges most safe instances [S32]; SMPT is a
portfolio with polyhedral reductions, PDR and verdict certificates, an
inductive predicate entailing the property [S33]. The Model Checking
Contest 2025 qualified five tools over 142 models and 1 855 instances;
TAPAAL's team reports the reachability gold [S34, S35].

**Well-structured systems and counter systems.** WSTS go back to
Finkel 1987; the backward algorithm of Abdulla, Čerāns, Jonsson and
Tsay decides coverability for every WSTS; Finkel and Schnoebelen
systematised them [S36, S37]. Downward-closed sets are finite unions of
ideals, and the ideal Karp–Miller algorithm computes coverability sets
of "very-WSTS", Petri nets among them [S38]. Schmitz's fast-growing
hierarchy F_α is the yardstick for all of this [S39]. For counter
systems, accelerated symbolic model checking (FAST, LASH, TReX)
terminates exactly on flat systems, and Leroux and Sutre showed many
semilinear classes flat, 2-VASS included [S40, S23].

**Petri nets with data.** Unordered-data nets (tokens carry values
compared for equality) are WSTS: coverability and boundedness are
decidable with Hyper-Ackermann coverability trees [S41];
bi-reachability is decidable (CONCUR 2024) while reachability stays
open [S42]. ν-Petri nets add fresh-name creation: coverability is
double-Ackermann-complete [S43], reachability undecidable (TCS 2011,
abstract as indexed) [S44]. The linear-logic reading exists: MSR,
multiset rewriting with ∃ for fresh nonces, is a fragment of linear
logic designed as input to linear-logic tools [S45, S46].

## 2. Candidate extensions

Each candidate: what it is in linlog, who would use it and why, what
it costs, how it relates to the saved plans, what step 28's API design
should leave room for.

### semantics-decision-1. Ideal certificates for non-coverability

*What.* In affine mode the Horn engine's backward search ends, when
the goal is not coverable, with the finite basis B of the upward-closed
set Pre*(↑goal) [R5]. B is itself a certificate: the goal is above some
element, every one-step predecessor of an element is above an element,
the initial marking is above none; a checker needs |B|²·|T| comparisons
of vectors and nothing from the engine [S36, S37] (the reading of the
basis as a certificate and its cost: inference). *Value.* Every refuted
qcover instance, 176 problems from software verification [R7, R8], gets
a checked "unprovable", later a Rocq lemma: group E of the refutations
note with a certificate the search already holds [R3]. *Cost.* A
`Refutation::Ideal` payload, a Rust checker, a Rocq lemma that a
covering firing sequence yields a proof (the converse of
`core-horn.md`'s induction) [R5]; small. *Relation.* Extends step 27's
follow-ups and refutations group E; feeds step 31. *API.* `Refutation`
payloads as versioned named structs; the Petri translation a pure,
public function with place = atom name and transition = clause
`OccId`, which the note already asks for [R3].

### semantics-decision-2. Presburger inductive invariants for non-reachability

*What.* For linear mode the goal unreachable "for a reason of integers
or order" (`a` stays odd) has no certificate today [R1]. Leroux's
characterisation makes a semilinear inductive invariant the universal
certificate [S26, S27]. A tractable subclass first: conjunctions of
linear inequalities and congruences, which subsume the Farkas vector,
traps and modulo-k invariants [S32, R3]; inductiveness is one integer
feasibility check per transition, which `lia` does in Rocq [S47].
*Value.* Closes the Horn engine's refutation gap on the random programs
and the odd-counter nets [R1]; a certificate format no Petri tool emits
in this form (inference). *Cost.* A search for invariants (templates
over small coefficients, or the backward markings as seeds [R1]);
medium; the general semilinear case is research. *Relation.* Extends refutations group E;
beside step 31. *API.* `Refutation` growth as above; numbers beyond
2⁵³ already a rule [R4].

### semantics-decision-3. Ideal Karp–Miller: coverability sets and boundedness

*What.* A forward search over ω-markings with acceleration [S38] in
the Horn engine, beside the backward one: it decides coverability from
the other side (the 128-token counter the backward search takes 6 s
on [R1]) and answers questions provability does not: is a place
bounded, is the net bounded, how many copies of a `!`-clause can ever
be used. *Value.* Teaching and modelling: "this resource is unbounded"
with the covering set as the witness; the LLTP nets beyond 5 s [R7].
*Cost.* The ω-marking type, the acceleration, a bound through
`memory::Account`; a new verdict kind. Medium. *Relation.* Extends
step 27's follow-up on coverability the backward search leaves; the
questions are new. *API.* A question kind beside provability in the
front door and the JSON (`Verdict` is about provability [R3]); the
`Engine` row saying which questions it answers [R4].

### semantics-decision-4. Directed and accelerated reachability

*What.* For the 66 LLTP nets whose goals lie on plateaus [R1]: an
A*-style search ordered by a relaxation of the net, as FastForward does
[S29], and cycle acceleration where the control graph is flat [S40].
An accelerated run is a parametric firing sequence; its instance is an
ordinary proof, so the checker needs nothing new (inference). *Value.*
The nets of practice that stay undecided; the state-equation heuristic
is already listed as a candidate [R1]. *Cost.* Low to medium per
heuristic; each row earned by `bench/targets.sh` [R1]. *Relation.*
Extends step 27's follow-ups; beside step 29, which measures against
the tools. *API.* `Stop::poll(progress)` and the account, already
decided [R4].

### semantics-decision-5. A Petri-net front door

*What.* A PNML reader and MCC-style reachability and coverability
queries beside the `.spec` reader: `linlog` answers a net's question
as a sequent with a proof or a certificate, and prints a !-Horn
sequent as a net. *Value.* The Petri-net community measures tools at
the MCC [S34]; linlog would be the entrant whose every answer is a
checkable linear-logic certificate (inference), and step 29's
comparison gains the MCC models beyond the 76 the LLTP used [R7]. *Cost.* Readers, the query language's fragment, the drawing;
medium. *Relation.* Beside steps 27 and 29; new. *API.* `Program`
constructible from net data without a `Sequent`, and printable to one,
both public (inference from `core-horn.md`'s private `Program::read`
[R5]).

### semantics-decision-6. Affine decision by saturation with subsumption

*What.* The provable sequents of affine LL are closed under weakening;
over the finite set of subformulas, Dickson's lemma makes their minimal
elements a finite antichain, which is Kopylov's and Lafont's
decidability [S2, S11]. The inverse method of step 37 run in affine
mode with weakening-subsumption keeps an antichain and terminates
(inference: an infinite sequence of newly kept sequents would be a bad
sequence over multisets). *Value.* Affine mode decided, not bounded:
the open question "whether a sound affine prune exists" [R1] answered
by a different engine; classical logic through affine MALL [R1]
inherits it. *Cost.* Full propositional affine LL is Tower-complete
[S6], so no bound; the saturated antichain is the refutation witness,
large. *Relation.* Extends step 37; beside the focused engine's affine
mode. *API.* `Engine` as data with modes; the database charged to the
account [R4].

### semantics-decision-7. Finite phase countermodels

*What.* A search for a finite phase model refuting a MALL or affine
sequent, as refutations group B proposes [R3]; what this note adds is
the method: enumerate small commutative monoids with a closure
operator, SAT-encoded, in the way Mace4 finds finite models of
equations [S48], with Galatos–Jipsen's frames as the theory that
extends it to other fragments [S13]. *Value.* A semantic "why not"
for MALL, where the focused engine's `Exhausted` is uncertified [R3];
the Rocq side can rest on Yalla's phase semantics when it lands
[S14]. *Cost.* Research: no size bound is known [S11, R3]; measure on
the refuted MALL targets first. *Relation.* Extends refutations group
B. *API.* `Refutation::PhaseModel` as a versioned payload [R3].

### semantics-decision-8. Petri nets with data: first-order Horn with fresh names

*What.* Once step 38 gives terms, the Horn reading extends to clauses
with variables and ∃ in heads: unordered-data nets and ν-nets [S41,
S43], the MSR fragment of protocol analysis [S45]. Coverability is
decidable by the WSTS route with name abstraction [S41, S43];
reachability is open for unordered data and undecidable with fresh
names [S42, S44], so the engine decides coverability and
semi-decides reachability. *Value.* Protocol and parameterised-system
analysis in linear logic, the use MSR was built for [S45, S46]. *Cost.*
High: data markings, the ordering on them, Hyper-Ackermann worst cases
[S41]; a second WSTS instance, which is when an ideal framework earns
its place (inference). *Relation.* Extends steps 27 and 38; new.
*API.* D-1's `Member` with frames and D-4's term arena [R6]; `Program`
must not assume propositional places.

### semantics-decision-9. Horn shapes beyond !-Horn

*What.* ⊕ in bodies is a transition with alternative inputs, & in
heads a branching that must succeed both ways; Kanovich places
(!,&)-Horn with Petri-net reachability and (!,⊕)-Horn beyond
decidability [S9, S10]. Widening `Program::read` to (!,&)-Horn keeps
the engine's decidability. *Value.* More LLTP and random programs take
the Horn row instead of the focused engine [R1]. *Cost.* The proof
read-off and the soundness induction of `core-horn.md` redone for the
new shapes [R5]; medium, and the measured gain unknown. *Relation.*
Extends step 27. *API.* The shape test as data on the `Engine` row
[R4].

## 3. Considered and rejected

- **MELL through BVASS reachability.** Equivalent to MELL provability
  and open [S4, S5], Tower-hard even affine [S6]: nothing decidable to
  implement beyond the bounded focused search.
- **A contractive (relevance) mode.** Decidable but Ackermann-complete
  [S6], 2-EXPTIME for the implicational fragment [S7]; not linear
  logic in the project's sense and no plan asks for it.
- **Bunched implications.** A different logic with a resource
  semantics of its own [S49]; no plan, no user named.
- **Kripke countermodels for ILL.** Completeness results are for phase
  models and no search exists; the refutations note defers it [R3].
- **Fixed-dimension specialisations** (2-VASS, 3-VASS) [S23, S24]: the
  nets of practice have tens of thousands of places [R1].
- **Calling external net tools.** Decided against at step 17: GPL-3.0
  tools can be called, never linked, and KReach is unchanged since 2020
  [R9].
- **Ordered-data and timed nets.** Harder than ν-nets [S43] and
  without a linear-logic reading in the plans.

## 4. What the API should settle now

(inference) Across the candidates: a question kind beside provability
(1, 3, 5); `Refutation` as versioned payload structs, the Petri
translation public with its numbering stated (1, 2, 7); `Program`
built from and printed to net data, over `Member`s and terms (5, 8,
9); `Engine` as data naming its modes, shapes and questions (3, 6, 9);
`Stop` with progress and the account on every long run (2, 4, 6). All
but the first are on the README's list [R4]; the question kind is new.

## 5. Sources

Repository:

- R1. `plan/later.md`, "Follow-ups: the focused engine", "Follow-ups: the Horn engine".
- R2. `plan/later.md`, "Certified refutations".
- R3. `plan/notes/research/refutations.md`.
- R4. `plan/notes/research/README.md`, sections 2 and 4.
- R5. `.claude/rules/core-horn.md`.
- R6. `plan/notes/research/design-constraints.md`, D-1 to D-4.
- R7. `plan/later.md`, "The !-Horn fragment through Petri-net reachability".
- R8. `plan/later.md`, "Follow-ups: the benchmarks".
- R9. `plan/later.md`, "Where each candidate went (2026-10-03)".

Literature and software:

- S1. Lincoln, Mitchell, Scedrov, Shankar, "Decision problems for propositional linear logic", APAL 56, 1992. https://www.csl.sri.com/papers/lmss90/lmss90.pdf
- S2. Kopylov, "Decidability of linear affine logic", Inf. Comput. 164, 2001. https://www.cs.cornell.edu/people/kopylov/papers/llw/index.htm
- S3. Straßburger, "On the decision problem for MELL", TCS 768, 2019. https://www.lix.polytechnique.fr/Labo/Lutz.Strassburger/papers/OnDeciMELL.pdf
- S4. de Groote, Guillaume, Salvati, "Vector addition tree automata", LICS 2004. https://lics.siglog.org/archive/2004/deGrooteGuillaumeSa-VectorAdditionTreeA.html
- S5. Verma, Goubault-Larrecq, "Karp-Miller trees for a branching extension of VASS", DMTCS 7, 2005. https://dmtcs.episciences.org/350
- S6. Lazić, Schmitz, "Non-elementary complexities for branching VASS, MELL, and extensions", LICS 2014; ACM TOCL 16(3), 2015. https://arxiv.org/abs/1401.6785
- S7. Schmitz, "Implicational relevance logic is 2-EXPTIME-complete", JSL 81(2), 2016. https://arxiv.org/abs/1402.0705
- S8. Urquhart, "The complexity of decision procedures in relevance logic II", JSL 64, 1999. https://resolve.cambridge.org/core/journals/journal-of-symbolic-logic/article/complexity-of-decision-procedures-in-relevance-logic-ii/C2C773998D8D629630ACEA0E64203A0F
- S9. Kanovich, "Generalized Horn fragments of LL", Types Forum, 25 Nov 1991 (preliminary announcement). https://www.engineering.upenn.edu/~sweirich/types/archive/1991/msg00101.html
- S10. Kanovich, "The complexity of Horn fragments of linear logic", APAL 69(2–3), 1994. https://api.openalex.org/works/doi:10.1016%2F0168-0072%2894%2990085-X
- S11. Lafont, "The finite model property for various fragments of linear logic", JSL 62(4), 1997. https://www.cambridge.org/core/journals/journal-of-symbolic-logic/article/abs/finite-model-property-for-various-fragments-of-linear-logic/F75D11ED6907E37EA0424226108DDA66
- S12. Okada, Terui, "The finite model property for various fragments of intuitionistic linear logic", JSL 64(2), 1999. https://api.philpapers.org/rec/OKATFM
- S13. Galatos, Jipsen, "Residuated frames with applications to decidability", Trans. AMS 365, 2013. https://www.ams.org/tran/2013-365-03/S0002-9947-2012-05573-5/
- S14. Laurent, "YALLA: an LL library for Rocq", release 2.0.7 (2025-03-26), ongoing work. https://perso.ens-lyon.fr/olivier.laurent/yalla/
- S15. Larchey-Wendling, "Mechanizing cut-elimination in Coq via relational phase semantics", talk, Dec 2018. https://members.loria.fr/DLarchey/files/papers/cut_elim_2018.pdf
- S16. Forster, Larchey-Wendling, "Certified undecidability of intuitionistic linear logic via binary stack machines and Minsky machines", CPP 2019. https://www.ps.uni-saarland.de/Publications/details/ForsterLarchey-Wendling:2018:Undecidability-ILL.html
- S17. APLL, the nearest prover found, emits Yalla certificates. https://github.com/wujuihsuan2016/LL_prover
- S18. Czerwiński, Lasota, Lazić, Leroux, Mazowiecki, "The reachability problem for Petri nets is not elementary", STOC 2019; JACM 68(1), 2021. https://arxiv.org/abs/1809.07115
- S19. Leroux, Schmitz, "Reachability in vector addition systems is primitive-recursive in fixed dimension", LICS 2019. https://arxiv.org/abs/1903.08575
- S20. Czerwiński, Orlikowski, "Reachability in vector addition systems is Ackermann-complete", FOCS 2021. https://arxiv.org/abs/2104.13866
- S21. Leroux, "The reachability problem for Petri nets is not primitive recursive", FOCS 2021. https://arxiv.org/abs/2104.12695
- S22. Czerwiński, Jecker, Lasota, Leroux, Orlikowski, "New lower bounds for reachability in vector addition systems", FSTTCS 2023. https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.FSTTCS.2023.35
- S23. Blondin, Finkel, Göller, Haase, McKenzie, "Reachability in two-dimensional vector addition systems with states is PSPACE-complete", LICS 2015. https://arxiv.org/abs/1412.4259
- S23a. Englert, Lazić, Totzke, "Reachability in two-dimensional unary vector addition systems with states is NL-complete", LICS 2016 (not fetched; cited from memory).
- S24. Czerwiński, Jecker, Lasota, Orlikowski, "Reachability in 3-VASS is elementary", ICALP 2025. https://arxiv.org/abs/2502.13916
- S25. Künnemann, Mazowiecki, Schütze, Sinclair-Banks, Węgrzycki, "Coverability in VASS revisited", ICALP 2023. https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.ICALP.2023.131
- S26. Leroux, "The general vector addition system reachability problem by Presburger inductive invariants", LMCS 6(3), 2010. https://lmcs.episciences.org/1024
- S27. Leroux, "Vector addition system reachability problem: a short self-contained proof", POPL 2011. https://hal.archives-ouvertes.fr/hal-00599756
- S28. Dixon, Lazić, "KReach: a tool for reachability in Petri nets", TACAS 2020. https://link.springer.com/chapter/10.1007/978-3-030-45190-5_22
- S29. Blondin, Haase, Offtermatt, "Directed reachability for infinite-state systems", TACAS 2021. https://arxiv.org/abs/2010.07912
- S30. Blondin, Finkel, Haase, Haddad, "Approaching the coverability problem continuously", TACAS 2016; QCover (Apache-2.0). https://arxiv.org/abs/1510.05724 ; https://github.com/blondimi/qcover
- S31. Geffroy, Leroux, Sutre, "Occam's razor applied to the Petri net coverability problem", RP 2016; TCS 750, 2018. https://arxiv.org/abs/1607.05956
- S32. Esparza, Ledesma-Garza, Majumdar, Meyer, Niksic, "An SMT-based approach to coverability analysis", CAV 2014. https://archive.model.in.tum.de/um/courses/petri/SS2017/material/cav2014-paper.pdf
- S33. Amat, Dal Zilio, "SMPT: a testbed for reachability methods in generalized Petri nets", FM 2023, with slides. https://arxiv.org/abs/2302.14741 ; https://homepages.laas.fr/namat/slides/FM_2023.pdf
- S34. Model Checking Contest 2025, results. https://mcc.lip6.fr/2025/results.php
- S35. TAPAAL team, "MCC'25: three gold medals". https://www.tapaal.net/news/mcc25-three-gold-medals/
- S36. Abdulla, Čerāns, Jonsson, Tsay, "Algorithmic analysis of programs with well quasi-ordered domains", Inf. Comput. 160, 2000 (LICS 1996), as cited at https://en.wikipedia.org/wiki/Well-structured_transition_system
- S37. Finkel, Schnoebelen, "Well-structured transition systems everywhere!", TCS 256, 2001. https://people.irisa.fr/Nicolas.Markey/biblio/tcs256(1-2)-FS
- S38. Blondin, Finkel, Goubault-Larrecq, "Forward analysis for WSTS, part III: Karp-Miller trees", FSTTCS 2017; LMCS 16(2), 2020. https://lmcs.episciences.org/4971
- S39. Schmitz, "Complexity hierarchies beyond elementary", ACM TOCT 8(1), 2016. https://arxiv.org/abs/1312.5686
- S40. Leroux, Sutre, "Flat counter automata almost everywhere!", ATVA 2005. https://hal.archives-ouvertes.fr/hal-00346310
- S41. Hofman, Lasota, Lazić, Leroux, Schmitz, Totzke, "Coverability trees for Petri nets with unordered data", FoSSaCS 2016. https://wrap.warwick.ac.uk/75834/
- S42. Kamiński, Lasota, "Bi-reachability in Petri nets with data", CONCUR 2024. https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.CONCUR.2024.31
- S43. Lazić, Schmitz, "The complexity of coverability in ν-Petri nets", LICS 2016. https://wrap.warwick.ac.uk/79162/
- S44. Rosa-Velardo, de Frutos-Escrig, "Decidability and complexity of Petri nets with unordered data", TCS 412, 2011 (abstract as indexed; the repository was unreachable). https://eprints.ucm.es/20624/
- S45. Cervesato, Durgin, Lincoln, Mitchell, Scedrov, "A meta-notation for protocol analysis", CSFW 1999. https://www.cs.cmu.edu/~iliano/papers/csfw99.pdf
- S46. Cervesato, Scedrov, "Relating state-based and process-based concurrency through linear logic", Inf. Comput. 207, 2009. https://kilthub.cmu.edu/articles/journal_contribution/Relating_State-Based_and_Process-Based_Concurrency_through_Linear_Logic/6608957
- S47. Rocq reference manual, "Micromega" (`lia`). https://rocq-prover.org/refman/addendum/micromega.html
- S48. McCune, Prover9 and Mace4 (Mace4 searches for finite models). https://www.cs.unm.edu/~mccune/prover9/
- S49. Pym, O'Hearn, Yang, "Possible worlds and resources: the semantics of BI", TCS 315, 2004. https://researchportal.bath.ac.uk/en/publications/possible-worlds-and-resources-the-semantics-of-bi/

Sources checked 2026-10-09: 38 checked, 3 corrected, 0 removed, 1 claims marked.
