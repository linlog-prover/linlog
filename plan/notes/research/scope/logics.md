# Neighbouring logics: how linlog's scope could grow

Written 2026-10-09 from the web sources in section 5 and from
`plan/later.md`, `plan/notes/research/README.md`,
`design-constraints.md` and `practice.md`. Every factual claim carries a
source tag; "(inference)" marks this note's own conclusions. The steps
are cited by number as in `plan/README.md`, the design decisions of
`design-constraints.md` as D-1 to D-12.

## 1. The field in brief

Almost every neighbour of linear logic is reached by one of four moves:
change which structural rules a formula may use (subexponentials,
adjoint and graded modalities, relevant logic), weaken exchange
(Lambek, ordered logic, BV, pomset logic), add formula-level recursion
(μMALL), or change what a context is (BI's bunches).

**Structural rules as a signature.** Subexponential linear logic (SELL)
indexes `!` and `?` by labels from a preorder, each label allowing or
forbidding weakening and contraction; the idea goes back to Danos,
Joinet and Schellinx (1993) and was turned into a specification
language by Nigam and Miller (2009) [NOP17]. Olarte, Pimentel and Nigam
built focused systems over it and encoded concurrent constraint
programming [NOP17]. MLL with one unrestricted and two incomparable
linear subexponentials already encodes two-counter Minsky machines and
is undecidable [Cha14]. Adjoint logic (Pruiksma, Chargin, Pfenning,
Reed 2018) combines intuitionistic logics of different structural
properties through adjoint pairs of modalities, with a mode on every
formula; it is placed "at the confluence" of Benton's LNL and
subexponentials [PCPR18], and has a natural-deduction form and a
functional language reading [JFSCD24]. Graded modalities replace `!`
by `!_r` with a grade from a semiring; Bounded Linear Logic (Girard,
Scedrov, Scott 1992) is the ancestor and characterises polynomial time
[GSS92]; Granule is the working language (linear, indexed and graded
modal types) [OLE19]; Eades and Orchard graded adjoint logic over
pointed semirings [EO20]; Hughes and Orchard synthesise programs from
graded types with Hodas–Miller input-output contexts and an SMT solver
for the grades [HO20, HO24]. Relevant logic R is undecidable (Urquhart
1984) [Urq84]; LR, R without distribution, is decidable by Meyer's
extension of Kripke's procedure, implemented as the KRIPKE program of
Thistlewaite, McRobbie and Meyer (1988), and its implication-conjunction
fragment has no primitive recursive decision procedure, the lower bound
adapted from Lincoln, Mitchell, Scedrov and Shankar [Urq16, Urq99];
distribution is exactly what linear logic's additives lack [Urq16].

**Order.** Polakow and Pfenning's ordered linear logic conservatively
extends ILL with ordered hypotheses and carries a logic programming
language (Olli) and a framework [Pol01]; Simmons and Pfenning gave it a
weakly focused calculus [SP10]. Kanovich, Kuznetsov, Nigam and Scedrov
studied subexponentials that forbid exchange: cut elimination holds
under stated conditions and the systems are undecidable or decidable
depending on which contraction a label allows [KKNS19]; with soft
(multiplexing) subexponentials the general system is undecidable and a
fragment decidable [KKNS20], and a globally bounded multiplexing is
decidable [MWS21]. BV adds a self-dual non-commutative connective to
MLL and needs the calculus of structures, where rules apply at any
depth [Gug07]; it is NP-complete [Kah06]; MAV, BV plus the additives,
is PSPACE-complete with cut elimination [Hor15]; NEL, BV plus the
exponentials, is reported Turing-complete [GS11] (not in the source). Pomset logic (Retoré
1997) is MLL + Mix with a self-dual non-commutative connective given by
proof nets; its correctness is coNP-complete and its provability
Σ₂ᵖ-complete, and BV's theorems are a proper subset of pomset logic's
[NS20, NS23, NS22].

**Recursion.** μMALL adds least and greatest fixed points to MALL with
(co)induction rules; it has cut elimination and a complete focused
system [BM07, Bae12], and Bedwyr is a model checker built on this proof
theory [Mil19]. Finitary and circular μMALL are Σ⁰₁-complete, the
non-wellfounded system Π⁰₁-hard, and least fixed points alone are
already Σ⁰₁-complete via alternating VASS reachability [DDS22].
Validity of a circular proof under the thread criterion is PSPACE-complete
[NST19]; a local criterion is polynomial [NST18]; bouncing validity is
undecidable in general [BDKS20]; infinets are the nets of μMLL∞ [DS19].

**Bunches.** BI is the free combination of intuitionistic logic and
MILL with tree-shaped contexts [GJKR26] (not in the source). Boolean BI, the propositional
core of separation logic, is undecidable [BK10, PSP13]; BI itself was
believed decidable (Galmiche, Méry, Pym 2005; Gheorghiu, Docherty, Pym
2021) [GMP05, GDP21] until Galatos, Jipsen, Knudstorp and Ramanayake
showed it undecidable by Wang tilings and pointed out the gaps in the
decidability claims (LICS 2026) [GJKR26]. Separation-logic tools work in
the symbolic-heap fragment, decidable since Berdine, Calcagno and O'Hearn
[BCO04], compared in SL-COMP [SLC19]; the magic wand simulates `∗` and
leads to undecidability [DLM18].

**Implicit complexity and differentiation.** Light, soft and elementary
linear logic bound cut elimination (polynomial, polynomial, elementary)
and capture the matching function classes [GR09]; EAL's provability is
decidable by phase semantics [DM04]; cut elimination is EXPTIME-complete
for multiplicative SLL and 2EXPTIME-complete for multiplicative LLL
[Ter]. Differential linear logic adds codereliction, cocontraction and
coweakening, with differential interaction nets and formal sums of
proofs [ER06, Ehr18].

## 2. Candidate extensions

### logics-1. Subexponential signatures (SELL)

*What.* `!` and `?` carry a label from a user-declared preorder, each
label with its (weakening, contraction) pair; promotion checks the
preorder; the dyadic context becomes one zone per label. Today's modes
are the two-label special cases (linear: one label with W and C; affine:
every label with W) (inference). Adjoint logic's modes are the
intuitionistic reading of the same signature [PCPR18] and come with it
on the two-sided engine (inference).

*Value.* Nigam and Miller's algorithmic specifications and the CCP
encodings [NOP17]; Petri nets with locations and modal reasoning as
subexponentials [NOP17]; teaching the substructural cube in one tool.

*Cost.* The data model (a label on `Term::Bang`/`Quest`: D-4's two
`u32` payloads hold it), the parser, the focused engine's `Θ` split
into labelled zones (D-7's zone parameter), promotion's preorder check
in engine, checker, oracle and Rocq mirror, the exports' label syntax.
Undecidable in general [Cha14], so the copy bound and three-valued
answer carry over unchanged. One to two sessions after 28 (inference).

*Relation.* Extends `Mode` (D-8) and sits beside 33 (a box per label)
and 36 (an exchange-free label is logics-4). New as a logic.

*Leave room.* `Term::Bang(label, child)` where label 0 is today's `!`;
`Mode` built by builders, not fields; a zone type that is a map from
label to member set, not one `Θ`.

### logics-2. Graded and bounded exponentials

*What.* `!_r A` with `r` from a semiring; first the natural numbers and
intervals (`[0..1]` is affine, `[n]` is `n` copies), which is Bounded
Linear Logic without resource polynomials [GSS92] and what
`practice.md` row 3 already encodes by hand for Granule's sequents.

*Value.* Granule users and program synthesis from graded types, where
the grades prune the search and each SMT call is the cost [HO20, HO24];
quantitative type theories. linlog would be the propositional decider
for the grade-free shape, with the grades solved by the search's own
counting rather than SMT (inference).

*Cost.* A grade on the term (D-4's payload again), the `!` rules as
one table per semiring, the counts of the focused engine extended to
grades (the copy bound becomes the grade), the checker verifying
grade arithmetic. For a general semiring an external solver, which the
library should not link; a trait the caller implements (inference).

*Relation.* Beside 25 (another layer with translations and a read-back)
and `practice.md`'s dropped Granule rows; extends logics-1 (a grade is
a label with arithmetic).

*Leave room.* `Statistics` and `Refusal` by named fields; `Inference`'s
datum slot (D-12) holding a grade; no public field that assumes `!`
has one rule set.

### logics-3. Ordered logic and Lambek with modalities

*What.* An ordered zone (a sequence) beside the linear and unrestricted
ones [Pol01], and non-commutative or commutative subexponentials over
step 36's Lambek calculus [KKNS19], with the decidable fragments
(modality on variables only, bounded multiplexing) [KKNS19, KKNS20,
MWS21] as the default and a bound otherwise.

*Value.* Categorial grammar with controlled movement (the TLGbank rows
of `practice.md`), ordered logic programming and substructural
operational semantics [Pol01, SP10].

*Cost.* The zone type must admit an ordered member list (D-7); the
focused engine's split enumerates contiguous segments, not subsets; 36's
planar linking takes the modal rules; checker and exports follow.
Depends on 36 and logics-1.

*Relation.* Extends 36 and 38 (the Lambek embedding into first-order
MILL that `later.md` names); extends logics-1.

*Leave room.* `Mode.cyclic` as 36 plans, with a sibling `ordered`;
members in written order (D-2), since order is then meaning.

### logics-4. Deep inference: BV, MAV and pomset nets

*What.* A `Seq` connective, self-dual and non-commutative, and a proof
object that is a rewrite sequence on a structure rather than a tree,
since no shallow system is complete [Kah24]; MAV's PSPACE-completeness
and cut elimination make it the decidable target [Hor15]. The cheaper
entry is pomset-logic proof nets as a checker and drawing [NS20].

*Value.* Process-calculus verification (Horne's motivation) [Hor15];
the one logic linlog could not reach by a mode.

*Cost.* High: a new engine, a new certificate kind (the derivation is
not a `Node` tree), new exports, correctness for pomset nets is
coNP-complete [NS20], and BV and pomset logic are different logics
[NS22]. Two to three sessions, research in part (inference).

*Relation.* New; beside 35 and 36. Nothing planned reduces it.

*Leave room.* One of D-4's four reserved `Term` variants; the
certificate kinds as a second plug-in list, as `refutations.md` plans
for refuters; `svg` ids for a structure's nested context.

### logics-5. Fixed points (μMALL)

*What.* `μ` and `ν` binders over formula variables with Baelde's
finitary rules (unfolding, induction with an invariant) [Bae12], the
focused system as the engine's discipline, bounded unfolding with the
three-valued answer, since finitary μMALL is Σ⁰₁-complete [DDS22];
later circular proofs with a PSPACE validity check [NST19].

*Value.* Inductive definitions and model checking in the Bedwyr line
[Mil19]; teaching (co)induction in a linear setting.

*Cost.* Formula-level binders in the arena, walk and printers (D-10's
binder stop), a node that carries an invariant (D-5's side table), a
checker that checks the invariant's premises, the interactive session
asking the user for the invariant, a Rocq mirror with binders (31). A
circular proof is a graph, which the `Node` tree is not. After 38.

*Relation.* Extends 38 (binders, substitution) and 31; beside 34.

*Leave room.* The binder stop and locally nameless terms (D-10, D-4)
over formulas, not only first-order terms; `Node` operands as member
ids (D-1) so an unfolded instance can be a member.

### logics-6. Relevant logic LR as a mode

*What.* Contraction without weakening: the fourth corner of the
structural square, dual to affine mode. LR is decidable by Kripke's
irredundancy argument extended by Meyer [Urq16], with no primitive
recursive bound on the fragment with conjunction [Urq99]; R itself,
with distribution, is undecidable [Urq84] and out.

*Value.* Relevance logicians and teaching; the KRIPKE program is from
1988 [Urq16] and no maintained decider is known to this note
(inference).

*Cost.* A `Mode` flag; a split that sends a member to one side or both
(2ⁿ becomes 3ⁿ); termination by Kripke's lemma, a dominance check
between a sequent and its ancestors, which is the loop check on the
branch that `later.md` wants for dyadic sequents (inference);
countermodels from Urquhart's semantics for `refutations.md` [Urq16].

*Relation.* New; extends `Mode` (D-8) and the loop check of 25's
follow-ups.

*Leave room.* `Mode` with `with_contraction()` beside `with_affine()`;
the memo's key comparing by dominance, not equality only.

### logics-7. Light logics as a checker and as modes

*What.* First a judgement on proofs: stratification and depth
conditions on 33's boxed nets that say whether a proof is in ELL, LLL
or SLL and hence what its cut elimination costs [GR09, Ter]; then the
soft rule set as a mode (multiplexing in place of contraction), whose
bound is a copy bound in another dress (inference), and EAL, decidable
[DM04].

*Value.* Implicit-complexity teaching and research; type inference for
light type systems (inference).

*Cost.* Small for the judgement once 33 has boxes with depth; a rule
table per mode for the search; §-modality for LLL is a new unary
connective.

*Relation.* Extends 33 and 34 (the bounded elimination is the theorem
being illustrated); new as a logic.

*Leave room.* `Criterion` as data that can carry a stratification
check (D-6); the exponential rule set as a value, not a match on
`Mode`.

### logics-8. Adjoint logic's functional reading

*What.* On logics-1's signature, the intuitionistic multi-mode sequent
with shifts `↑`/`↓` [PCPR18], proof terms read as programs of the
adjoint functional language [JFSCD24], and a synthesis mode (a goal
type, a program back) in the style of [HO24].

*Value.* The programming-languages audience that linlog's ILL engine
serves least today (inference).

*Cost.* Shifts as two connectives, modes on the two-sided reading, a
term printer; synthesis is the two-sided engine plus a term read-back.

*Relation.* Extends logics-1 and 8/25's two-sided layer; beside 32 (a
web playground is where this would be used, inference).

*Leave room.* `Rule` growth by checklist; a `Derivation`-to-term
exporter as one more export options value (D15).

## 3. Considered and rejected

- **BI and separation logic as a decided logic.** BI is undecidable
  [GJKR26], BBI was [BK10], and the tools that users run are symbolic-heap
  entailment checkers with inductive predicates [BCO04, SLC19], a
  different engineering world (SMT, inductive predicates) with its own
  competition. A bunch-shaped context also contradicts the multiset
  zone of D-7 everywhere (inference). A bounded BI prover would be
  possible as linlog's own logic is, but no user of linlog asks
  (inference).
- **Pomset logic and BV as search engines before a checker.** Σ₂ᵖ and
  a proof object outside the tree model [NS23, Kah24]: logics-4 keeps
  only the checker and MAV.
- **NEL.** Turing-complete [GS11] (not in the source); nothing a bound would decide that
  MELL under 33 does not teach (inference).
- **Non-wellfounded μMALL.** Π⁰₁-hard, no finite proof object [DDS22];
  logics-5 keeps finitary and circular proofs.
- **Differential linear logic.** Proofs are formal sums and the
  interest is semantic [ER06, Ehr18]; no proof-search literature was
  found (inference), and a display of differential nets has no user.
- **Full R and the semilattice logic S.** Undecidable [Urq84, Knu24].
- **General semirings with an SMT solver inside the library.** Against
  the dependency rule and wasm; logics-2 stops at the counting semirings
  and offers a trait.

## 4. What the API should settle now

Everything above fits D-1 to D-12 if three readings are kept: a
`Term`'s modal variants carry a payload (label, grade) whose zero is
today's connective (D-4); the zone is a type with an ordered variant and
a per-label partition, not one bitset (D-7); and `Mode` is built, never
matched exhaustively outside the engines (D-8, D-11). Only logics-4 and
circular logics-5 need a certificate kind beside the `Node` tree
(inference).

## 5. Sources

- [Bae12] D. Baelde, Least and greatest fixed points in linear logic, ACM TOCL 13(1), 2012. <https://arxiv.org/abs/0910.3383>
- [BCO04] J. Berdine, C. Calcagno, P. O'Hearn, A decidable fragment of separation logic, FSTTCS 2004, LNCS 3328. <https://doi.org/10.1007/978-3-540-30538-5_9>
- [BDKS20] D. Baelde, A. Doumane, D. Kuperberg, A. Saurin, Bouncing threads for circular and non-wellfounded proofs, 2020. <https://arxiv.org/abs/2005.08257>
- [BK10] J. Brotherston, M. Kanovich, Undecidability of propositional separation logic and its neighbours, LICS 2010; J. ACM 2014. <https://discovery.ucl.ac.uk/id/eprint/1430439/>
- [BM07] D. Baelde, D. Miller, Least and greatest fixed points in linear logic, LPAR 2007. <https://www.lix.polytechnique.fr/~dale/papers/lpar07final.pdf>
- [Cha14] K. Chaudhuri, Undecidability of multiplicative subexponential logic, LINEARITY 2014, EPTCS 176. <https://arxiv.org/abs/1502.04769>
- [DDS22] A. Das, A. De, A. Saurin, Decision problems for linear logic with least and greatest fixed points, FSCD 2022, LIPIcs 228. <https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.FSCD.2022.20>
- [DLM18] S. Demri, É. Lozes, A. Mansutti, The effects of adding reachability predicates in quantifier-free separation logic, 2018 (cites Brochenin, Demri, Lozes 2012). <https://arxiv.org/abs/1810.05410>
- [DM04] U. Dal Lago, S. Martini, Phase semantics and decidability of elementary affine logic, TCS 318(3), 2004. <https://protocollo.di.unito.it/BIBLIO/TCS2004.html>
- [DS19] A. De, A. Saurin, Infinets: the parallel syntax for non-wellfounded proof theory, TABLEAUX 2019. <https://hal.archives-ouvertes.fr/hal-02337286>
- [Ehr18] T. Ehrhard, An introduction to differential linear logic, MSCS 28(7), 2018. <https://arxiv.org/abs/1606.01642>
- [EO20] H. Eades, D. Orchard, Grading adjoint logic, 2020. <https://arxiv.org/abs/2006.08854>
- [ER06] T. Ehrhard, L. Regnier, Differential interaction nets, TCS 364(2), 2006. <https://portal.mardi4nfdi.de/entity/Q860836>
- [GDP21] A. Gheorghiu, S. Docherty, D. Pym, Provability in BI's sequent calculus is decidable, 2021 (the arXiv abstract says the proof of the main theorem is invalid; the paper is withdrawn, and the section is not named). <https://arxiv.org/abs/2103.02343>
- [GJKR26] N. Galatos, P. Jipsen, S. B. Knudstorp, R. Ramanayake, The logic of bunched implications is undecidable, LICS 2026, LIPIcs. <https://arxiv.org/abs/2603.01595>
- [GMP05] D. Galmiche, D. Méry, D. Pym, The semantics of BI and resource tableaux, MSCS 15, 2005. <https://researchportal.bath.ac.uk/en/publications/the-semantics-of-bi-and-resource-tableaux/>
- [GR09] M. Gaboardi, S. Ronchi Della Rocca, From light logics to type assignments: a case study, Logic J. IGPL 17(5), 2009. <https://cs-people.bu.edu/gaboardi/publication/GaboardiRonchi09igpl.pdf>
- [GS11] A. Guglielmi, L. Straßburger, A system of interaction and structure IV: the exponentials and decomposition, 2009/2011. <https://arxiv.org/abs/0903.5259>
- [GSS92] J.-Y. Girard, A. Scedrov, P. Scott, Bounded linear logic, TCS 97(1), 1992. <https://www.site.uottawa.ca/~phil/papers/BLL.1992.pdf>
- [Gug07] A. Guglielmi, A system of interaction and structure, ACM TOCL 8(1), 2007. <https://arxiv.org/abs/cs/9910023>
- [HO20] J. Hughes, D. Orchard, Resourceful program synthesis from graded linear types, LOPSTR 2020, LNCS 12561. <https://www.ncbi.nlm.nih.gov/pmc/articles/PMC7880237/>
- [HO24] J. Hughes, D. Orchard, Program synthesis from graded types, ESOP 2024, LNCS 14576. <https://doi.org/10.1007/978-3-031-57262-3_4>
- [Hor15] R. Horne, The consistency and complexity of multiplicative additive system virtual, Sci. Ann. Comput. Sci. 25(2), 2015. <https://personal.cis.strath.ac.uk/ross.horne/pdf/sacs.pdf>
- [JFSCD24] J. Jang, S. Roshal, F. Pfenning, B. Pientka, Adjoint natural deduction, FSCD 2024, LIPIcs 299. <https://drops.dagstuhl.de/storage/00lipics/lipics-vol299-fscd2024/LIPIcs.FSCD.2024.15/LIPIcs.FSCD.2024.15.pdf>
- [Kah06] O. Kahramanoğulları, System BV is NP-complete, WoLLIC 2005, ENTCS 143, 2006. <https://iccl.inf.tu-dresden.de/web/WVPub22>
- [Kah24] O. Kahramanoğulları, Deep inference in proof search: the need for shallow inference, EPiC 100, 2024. <https://easychair.org/publications/open/MgHR>
- [KKNS19] M. Kanovich, S. Kuznetsov, V. Nigam, A. Scedrov, Subexponentials in non-commutative linear logic, MSCS 29, 2019. <https://arxiv.org/abs/1709.03607>
- [KKNS20] M. Kanovich, S. Kuznetsov, V. Nigam, A. Scedrov, Soft subexponentials and multiplexing, IJCAR 2020. <https://link.springer.com/10.1007/978-3-030-51074-9_29>
- [Knu24] S. B. Knudstorp, Relevant S is undecidable, LICS 2024. <https://dare.uva.nl/id/ab802007-dd7a-46a5-9f44-5b9076aab512>
- [Mil19] D. Miller, Applying a linear logic perspective to arithmetic, abstract, 2019 (cites the Bedwyr system, CADE 2007). <https://www.lix.polytechnique.fr/~dale/papers/pts2019abstract.pdf>
- [MWS21] L. McPheat, H. Wazni, M. Sadrzadeh, Vector space semantics for Lambek calculus with soft subexponentials, 2021. <https://arxiv.org/abs/2111.11331>
- [NOP17] V. Nigam, C. Olarte, E. Pimentel, On subexponentials, focusing and modalities in concurrent systems, TCS 693, 2017. <https://www.fortiss.org/ergebnisse/publikationen/details/on-subexponentials-focusing-and-modalities-in-concurrent-systems>
- [NS20] L. T. D. Nguyễn, L. Straßburger, Complexity of correctness for pomset logic proof nets, 2020 (withdrawn, subsumed by [NS23]). <https://arxiv.org/abs/1912.10606>
- [NS22] L. T. D. Nguyễn, L. Straßburger, BV and pomset logic are not the same, CSL 2022. <https://drops.dagstuhl.de/opus/volltexte/2022/15752>
- [NS23] L. T. D. Nguyễn, L. Straßburger, A system of interaction and structure III: the complexity of BV and pomset logic, LMCS 2023. <https://arxiv.org/abs/2209.07825>
- [NST18] R. Nollet, A. Saurin, C. Tasson, Local validity for circular proofs in linear logic with fixed points, CSL 2018. <https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.CSL.2018.35>
- [NST19] R. Nollet, A. Saurin, C. Tasson, PSPACE-completeness of a thread criterion for cyclic proofs in linear logic with least and greatest fixed points, TABLEAUX 2019. <https://hal-univ-tlse3.archives-ouvertes.fr/INRIA/hal-02173207>
- [OLE19] D. Orchard, V.-B. Liepelt, H. Eades, Quantitative program reasoning with graded modal types, ICFP 2019. <https://www.cs.kent.ac.uk/people/staff/dao7/publ/granule-icfp19.pdf>
- [PCPR18] K. Pruiksma, W. Chargin, F. Pfenning, J. Reed, Adjoint logic, 2018. <https://ncatlab.org/nlab/files/PCPR18-AdjointLogic.pdf>
- [Pol01] J. Polakow, Ordered linear logic and applications, PhD thesis, CMU-CS-01-152, 2001. <https://www.csd.cmu.edu/sites/default/files/phd-thesis/CMU-CS-01-152.pdf>
- [PSP13] J. Park, J. Seo, S. Park, A theorem prover for Boolean BI, POPL 2013. <https://oasis.postech.ac.kr/handle/2014.oak/15965>
- [SLC19] M. Sighireanu and others, SL-COMP: competition of solvers for separation logic, TACAS 2019. <https://hal.ccsd.cnrs.fr/LMF/hal-02388022v1>
- [SP10] R. Simmons, F. Pfenning, Weak focusing for ordered linear logic, CMU-CS-10-147. <https://kilthub.cmu.edu/articles/journal_contribution/Weak_Focusing_for_Ordered_Linear_Logic_CMU-CS-10-147_/6612839/1>
- [Ter] H. G. Mairson, K. Terui, On the computational complexity of cut-elimination in linear logic, ICTCS 2003, LNCS 2841 (listed on Terui's publications page). <https://www.kurims.kyoto-u.ac.jp/~terui/pub_bib.html>
- [Urq16] A. Urquhart, Relevance logic: problems open and closed, Australasian J. Logic 13(1), 2016. <https://ojs.victoria.ac.nz/ajl/article/view/3926>
- [Urq84] A. Urquhart, The undecidability of entailment and relevant implication, JSL 49, 1984. <https://www.cambridge.org/core/journals/journal-of-symbolic-logic/article/abs/undecidability-of-entailment-and-relevant-implication/FC9AE61D959740AB0356AEE1F5BC70E8>
- [Urq99] A. Urquhart, Complexity of decision procedures in relevance logic II, JSL 64, 1999. <https://resolve.cambridge.org/core/journals/journal-of-symbolic-logic/article/complexity-of-decision-procedures-in-relevance-logic-ii/C2C773998D8D629630ACEA0E64203A0F>

Sources checked 2026-10-09: 36 checked, 5 corrected, 0 removed, 3 claims marked.
