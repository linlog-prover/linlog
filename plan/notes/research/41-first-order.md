# Research note for step 38: first-order linear logic

Written 2026-10-08 from the snapshot of the repository and the sources of
section 5. `plan/notes/api.md`, which the step's prompt names, does not
exist yet (step 28 has not run); this note says what it should settle.

## 1. The problem and the state of the art

**Definitions.** First-order linear logic adds atoms `p(t₁, …, tₙ)` over
terms (variables, parameters, function applications) and the quantifiers
`∀x.A` and `∃x.A`. In the one-sided calculus `∀` introduces a fresh
*eigenvariable* `a` that may not occur in the conclusion, and `∃`
instantiates its variable with any term [LS94, Appendix B; CP05, §2].
Under focusing `∀` is negative (asynchronous, a fresh parameter) and `∃`
positive (synchronous, an arbitrary term): "positive universal and
negative existential formulas can only ever be instantiated with
parameters in a cut-free backward sequent derivation" [CP05, §4]; LJF's
positive formulas include the existential [Mil21].

**Unification and the eigenvariable condition.** Proof search replaces the
`∃` guess by a fresh *Herbrand (meta)variable* and unifies at the axioms;
the eigenvariable condition is kept by "dynamic Skolemization": a `∀`
introduces `h(T)`, a Herbrand function applied to the metavariables `T`
introduced below it, so that an occurs-check failure signals a violation
[LS94, §2–3]. *Static* Skolemization of the conclusion is unsound in
linear logic: `⊢ (∃x.p⊥ ⅋ q⊥(x)), (∀y.q(y)) ⊗ p` is not provable, yet its
Skolemized form is [LS94, §2]. Bruni, Ritter and Schürmann give the
counterexamples `∀x.A ⊗ B(x) ⊢ A ⊗ ∀u.B(u)` and `∀x.!A(x) ⊢ !∀u.A(u)`: the
order between the quantifier rules and `⊗R` or `!R` is lost [BRS24].
Dynamic Skolemization in its naive form (LLV) is complete but makes the
search try the quantifier rules in every order; Lincoln and Shankar's
LLO reduces the dependencies to the three impermutable rules `!`, `⅋`
and `&`, so that in pure MLL "all
quantifier rules can be applied immediately" [LS94, §4]. BRS24 reach the
same end for focused ILL with Skolem terms that record branch and world
dependencies, checked at unification; they report no implementation
[BRS24]. Shankar did this first for LJ [Sha92]; Miller's *raising* is the
dual of Skolemization for mixed quantifier prefixes [Mil92].

**Complexity.** MLL1 is NP-complete and MALL1 NEXPTIME-complete: a
MALL1 proof's depth is bounded by the connectives of the conclusion, each
axiom sequent is at most quadratic in it, and unification is polynomial
[LS94, Thm 3.4, 3.5]; hardness is Lincoln and Scedrov's [LSc94]. First-order
additive LL is NP-complete [HH15]. Propositional LL is undecidable
[LMSS92], hence so is every first-order fragment with exponentials; MLL
with first- *and* second-order quantifiers is undecidable [LSS95], so
second-order stays out. For MELL1 nothing was found (inference: at least
as hard as propositional MELL, which is open).

**Proof nets, tableaux, connections.** First-order MLL proof nets are
Girard's (1991) and Bellin and van de Wiele's, with "each quantifier link
uses a distinct bound variable" and the eigenvariable of a `∀` link; Moot
gives a contraction criterion for MILL1 nets [Moo20]. For first-order ALL
there are *witness nets* (witnesses recorded) and *unification nets*
(witnesses recovered by unification, canonical up to witness choice)
[HHS19]. linTAP is a tableau prover for propositional M?LL that leaves
"the additive fragment and the quantifiers out of consideration" and
handles the permutabilities by *prefix unification* [MO99]; Kreitz et
al.'s connection method covers the propositional multiplicative fragment
[KMOS97]. Neither is a first-order prover.

**Implementations.**

| prover | logic | language | licence | last change | source |
|---|---|---|---|---|---|
| LinearOne | first-order MILL ("prototype theorem prover/parser"), hybrid and Displacement grammars by translation | SWI-Prolog | LGPL-2.1 | pushed 2026-06-11, no release | [Moot-LO] |
| llprover | two-sided first-order LL, contraction capped at 3 per path, no occurs check ("known bugs"), quantified formulas not drawn as nets | Prolog | not stated | 2020-12 | [Tam20] |
| Chaudhuri–Pfenning inverse method | first-order ILL, lifted sequents with explicit unification; "the only one of its kind" | Standard ML | not found | 2005–2006 | [CP05; Cha06] |
| Lolli 0.94 | linear hereditary Harrop formulas, no higher-order unification (not in the source) | Standard ML | not stated | 1990s | [FP-sw] |
| Celf | CLF, dependent types, linear and concurrent | Standard ML | not stated | 2008 | [SS08] |

Yalla 2.0.7 lists "quantifiers" under future work [Yalla]; the Lean
FormalizedFormalLogic repository mentions no linear logic [FFL], so the
assessment's Lean LL¹ (`plan/reports/17-assessment.md`, 3.9) is not
confirmed. No first-order *linear* problem library exists: ILLTP is
propositional, from Kleene's theorems, ILTP's propositional part and
Petri nets [ODPR19]; ILTP has first-order intuitionistic problems (about
2 800 in all, licence not stated) [ILTP], and the ILLTP translations
applied to them would give first-order ILL problems (inference).

## 2. What the step needs

**Rules in the focused engine** (`core/src/search/focus/mod.rs`:
`asynchronous`, `focus`, `initial`). `∀` joins the asynchronous phase and
binds a fresh eigenvariable; `∃` is a focus step that binds a fresh
metavariable; `initial` unifies the two literals' argument lists instead
of comparing atoms. Cost: one unification with occurs check per axiom,
linear in the terms on average [MM82; ACF].

**Eigenvariable condition: levels, not Skolem terms (recommendation).**
Each variable carries the depth of the branch at its creation; binding a
metavariable to a term with a *younger* eigenvariable fails, and the
occurs check comes free in the same walk. This is LLV's bookkeeping with
an integer instead of `h(T)` (inference): sound and complete under
backtracking over the focus order [LS94, Thm 3.3], the witnesses stay
real terms (no Skolem functions in proofs, views or exports), and the
propositional path never reads the field. LLO's dependency reduction
[LS94, §4] and BRS24's constraints are the follow-up, measured on the
first-order families.

**Terms: locally nameless in a hash-consed arena (recommendation).**
Bound variables as de Bruijn indices [dB72] in the sequent's own
first-order term arena, free variables named [Cha12]: closed subformulas
stay α-canonical, so `Sequent::optimize` (`core/src/sequents/mod.rs`)
hash-conses them with no renaming; instantiation never rewrites a formula,
it binds the occurrence's binder to a variable, so the occurrence forest
survives. Maximal sharing by construction
is the standard hash-consing argument [CF06]. Terms created during search
(instances, bindings) go into a per-search arena that the trail
truncates; the sequent's arena is read-only.

**Substitution and trail.** Bindings in a `Vec<Option<TermRef>>` indexed
by metavariable, a trail of bound indices, a mark per choice point and
undo by popping to it: the WAM's trail [AK91]. A zone member is then
`(OccId, frame)`, where a frame is the list of terms the binders above the
occurrence are bound to; frames are hash-consed in the search's arena so
equal instances have equal ids (inference).

**What the prunes of step 15 assume** (`core/src/search/focus/`,
`.claude/rules/core-focus.md`). *Counts* (`counts.rs`) stay sound keyed
by predicate symbol: an instance keeps its symbol. *Interchangeable
occurrences* (`classes.rs`) rest on "no rule looks at an occurrence's
id"; with frames the class is `(term, frame, position)`, so canonical
keys are computed per zone or skipped for occurrences under a binder.
*The memo* (`memo.rs`, `Key { theta: OccSet, gamma: Context }`): a
`Complete` failure under a key with a free metavariable bound later by a
sibling is not a fact; recommendation: memoize only *ground* stable
sequents, keyed by instance, which leaves every propositional key as it
is. *`&` premises and cubes* (`parallel.rs`, `core-parallel.md`) assume
independence; premises sharing an unbound metavariable must run
sequentially with the right premise under the left's bindings and
backtracking into the left for another answer; the parallel `&` and the
cubes split only where no metavariable is shared. *The loop check*
compares keys by instance.

**Other engines.** The net engine (`search/net.rs`) gains unification on
links and the first-order criterion [Moo20] later; the additive path
(`search/additive.rs`) and the Horn engine (`search/horn/`) refuse
quantifiers (first-order ALL is NP-complete [HH15], so no linear path
exists). The dispatch needs no new row: rows name a largest fragment
(`DISPATCH`, `search/mod.rs`), and a `quantifiers` flag in `Fragment`
keeps them from taking a first-order goal, which falls to `focus` or
`two_sided`.

**Problems.** Generators (inference): chains `∀x.(p(x) ⊸ p(s x))` under `!`
with `p(0) ⊢ p(sⁿ 0)`; Lambek-grammar parsing through the MILL1 embedding
[MP01; Moo13], also the comparison with LinearOne; alternating `∀`/`∃`
towers after the QBF encoding of [LSc94]. Verdicts by construction, as
`core/src/families.rs` does.

## 3. What linlog's library must offer

**Data model.** `Term` (`core/src/sequents/term.rs`) has `Var(Atom)`,
`DualVar(Atom)`, units, binary connectives and `Bang`/`Quest`; `Kind` has
arity 0, 1 or 2; `Term::dual` and `Kind::dual` pair the connectives.
Missing: `Pred(Atom, Args)` and `DualPred` (argument list by index into a
CSR table, so `Term` keeps its size and `Var`/`DualVar` remain for nullary
predicates), `Forall(TermId)` and `Exists(TermId)` (unary; `dual` swaps
them, the parser's `finish` passes the negation flag through), a
first-order term arena (`FoTerm::Bound(u32)`, `Free(Symbol)`,
`App(Symbol, Args)`), a symbol table and bound-variable names. `Sequent` (`sequents/mod.rs`: `terms`, `roots`, `atoms`) gains
those tables; `optimize` hash-conses and sorts them like `terms`, `add`
offsets them, `verify_integrity` checks them, `sizes`/`occurrences` are
unchanged. `fragment()` (`fragment.rs`) gains a sixth flag and names
(`MLL1`, …); `Mode` is unchanged. The parser (`parse/mod.rs`, one explicit
stack `pending`) gets a binder entry in the operand state, keyword or
`∀`/`∃` spellings, and resolves names against the open binders on that
stack, so nothing recurses; the formula walk (`sequents/fmt.rs`, `Walk`,
`Visit`: `Enter`, `Between`, `Exit`) gets binder stops and a term walk.
The forest (`occurrences/mod.rs`: `term`, `kind`, `parent`, `root`,
`size`, `depth`, CSR `literals` by atom and sign) is untouched by binders
as unary occurrences; a binder-depth array is added only when the sequent
has quantifiers. The reading (`occurrences/reading.rs`) gives `∀` and `∃`
both positions. **In the way:** `Context` (`focus/context.rs`: bitset plus
`extra: Vec<(OccId, u32)>` copy counts) and `OccSet` zones cannot carry an
instance per copy, which `?` copies need (`plan/later.md`); the engine
must become generic over its zone type, or a first-order engine shares
the phases with the propositional one (D17's "duplicated fast path").

**Proof term and checker.** `Node` (`proofs/mod.rs`) is pinned at 16 bytes
(`const _: () = assert!(size_of::<Node>() == 16)`), rule, occurrence and
two `NodeId`s; it needs `Forall(OccId, Eigen, NodeId)` and `Exists(OccId,
Witness, NodeId)` with `u32` ids into a witness arena the `Proof` owns
(`Proof::new(forest, nodes, root)` gains it; `Answer::of_arena` in
`search/mod.rs` passes it). The checker (`proofs/check.rs`: `Pass`,
`State { theta, gamma, any }`, `Bag`, `examine`, `Observer`, `Problem`)
needs members as `(OccId, frame)`; `Ax` compares the two instances
syntactically (eigenvariables as constants); `&` needs equal zones *by
instance*. The eigenvariable condition cannot be checked on the derived
conclusion, because a `⊤` above may absorb members the bottom-up pass
never sees (`core-proofs.md`, "the `any` flag"). Recommendation
(inference, for the panel): check instead that every eigenvariable is
introduced by exactly one `∀` node, occurs in no root, and occurs only in
witnesses of `∃` nodes inside that node's subtree. Sound: a free `a` in
the `∀` conclusion can only come from a root or from a witness below.
Complete up to renaming: in a valid proof an instance with `a` from an
`∃` outside the subtree never flows into it (it would reach the `∀`
conclusion), so `a` can be renamed there. Implemented as a set of
"unbound eigenvariables" that flows up with the state: `∃` adds, `∀`
removes its own, the root must be empty. Memo hits share subproofs
(`core-proofs.md`), so a shared `∀` subproof introduces `a` on two paths;
the check must count by node, not by path. `Rule` (`proofs/derivation.rs`)
gains `∀`, `∃` and the two-sided `∀L ∀R ∃L ∃R`; `Inference` sequents need
frames to print instances; `Interactive` (`proofs/interactive.rs`:
`rules`, `apply(goal, position, rule, left)`, `Terms`) needs `apply` to
take a witness or leave a metavariable, so the state must hold a
substitution that `close`'s graft extends.

**Engine interface and dispatch.** `Task { forest, goal, fragment, mode,
reading, roots }` (`search/mod.rs`) describes a goal by occurrence ids;
an interactive open goal under binders needs frames beside the ids
(`Option`, `None` propositional). `Decide::admits` refuses quantifiers in
every engine but the focused ones until each is extended. `Statistics`,
`Reason`, `Engine`, `Outcome` are `#[non_exhaustive]`, so `unifications`
and `bindings` counters and a term-depth reason are additive. `Options`
has private fields and setters; D16 asks for a witness-depth bound, since
with exponentials the proof depth is unbounded [LS94, §4.1].

**JSON.** `serialize/sequents.rs` writes `{terms, ids, var_dict}` with
tags `V D 1 ⊥ ⊤ 0 ⊗ ⅋ & ⊕ ! ?`; add tags `P`/`N` (predicate, dual), `∀`,
`∃`, and keys `fo_terms`, `symbols`, `variables` with `#[serde(default)]`
and skipped when empty, so every propositional file reads and writes as
before (D18). `serialize/proofs.rs`
(`Step` tags `ax ⊗ ⅋ 1 ⊥ & ⊕₁ ⊕₂ ⊤ ! ? copy wk mix`) adds `∀ [o, a, p]`
and `∃ [o, w, p]` plus a `witnesses` key. `serialize/interactive.rs`
writes inference sequents as id lists, which no longer name instances:
members become `[id, frame]` under quantifiers. `serialize/search.rs`
gets the new fragment names.

**Exports and options.** `export/notation.rs` prints over `Walk`, so the
binder stops reach LaTeX, Typst and SVG in one place; `export/rocq.rs`
answers `Unsupported`, since NanoYalla and Yalla are propositional
[Yalla], until step 31's library has quantifiers. D15 options: variable notation, whether
witnesses show in a derivation (`ViewOptions`), the quantifier spelling.

## 4. Risks, open questions, and what the prompt should add

**Risks.** D17: `Term` and `Node` must not grow and the hot loop must not
gain a branch for the ground case; the spike (`plan/28-audit-and-refactor.md`,
stage 2, item 4) measures this before the design is fixed. Soundness: the
memo with metavariables, `&` premises sharing variables, and the
checker's eigenvariable rule each need the panel. No benchmark exists;
LinearOne decides MILL1 only and llprover caps contraction at three
[Moot-LO; Tam20], so the comparison is on the generated families.

**Open questions.** (1) The text syntax: keywords `forall x.`/`exists x.`
with `∀`/`∃`, scope to the right as far as possible, and whether capitals
are variables as in llprover [Tam20]. (2) Untyped terms, or sorts (the
Lambek embedding needs none [MP01]). (3) Metavariables still free in a
found proof: a fresh constant, as Lincoln and Shankar instantiate [LS94],
or left as named variables in the term. (4) Whether the net engine gets
unification in this step. (5) How an interactive witness is parsed (new
constants allowed?). (6) Whether shared subproofs with eigenvariables are
allowed in the proof DAG. (7) Whether the author's use is MILL1 grammar
parsing, first-order programs with `!`, or both; it decides the first
families.

**The prompt should add:** the syntax decision or who makes it; that
`plan/notes/api.md` must exist and name the zone type and frame design
before the step; a second first-order reference prover written from the
calculus for the panel (`core-search.md`, "The reference prover"); the
ground-only memo rule and the sequential `&` under shared metavariables
as fixed points; the eigenvariable rule above as a claim to prove or
refute; the family list with its construction arguments; the comparison
protocol with LinearOne and llprover; and that the JSON additions are
pinned by a test reading a pre-step file unchanged.

## 5. Sources

- [LS94] P. Lincoln, N. Shankar, *Proof search in first-order linear logic and other cut-free sequent calculi*, LICS 1994, pp. 282–291. <https://lics.siglog.org/1994/LincolnShankar-Proofsearchinfirsto.html>, PDF <https://csl.sri.com/papers/lics94/lics94.pdf>
- [LSc94] P. Lincoln, A. Scedrov, *First-order linear logic without modalities is NEXPTIME-hard*, TCS 135(1), 1994, pp. 139–153. <https://www.csl.sri.com/~lincoln/papers/mall1-hard.pdf>
- [LSS95] P. Lincoln, N. Shankar, A. Scedrov, *Decision problems for second-order linear logic*, LICS 1995, pp. 476–485. <https://lics.siglog.org/1995/LincolnShankarScedr-DecisionProblemsFor.html>
- [LMSS92] P. Lincoln, J. Mitchell, A. Scedrov, N. Shankar, *Decision problems for propositional linear logic*, APAL 56, 1992. <https://curien.galene.org/ECI2023/Lincoln%2B-decision-LL.pdf>
- [HH15] W. Heijltjes, D. Hughes, *Complexity bounds for sum-product logic via additive proof nets and Petri nets*, LICS 2015. <https://lics.siglog.org/2015/HeijltjesHughes-ComplexityBoundsfor.html>
- [HHS19] W. Heijltjes, D. Hughes, L. Straßburger, *Proof nets for first-order additive linear logic*, FSCD 2019, LIPIcs 131. <https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.FSCD.2019.22>
- [CP05] K. Chaudhuri, F. Pfenning, *A focusing inverse method theorem prover for first-order linear logic*, CADE 2005. <https://www.cs.cmu.edu/~fp/papers/cade05.pdf>
- [Cha06] K. Chaudhuri, *The focused inverse method for linear logic*, PhD thesis, CMU-CS-06-162, 2006. <https://www.csd.cmu.edu/sites/default/files/phd-thesis/CMU-CS-06-162.pdf>
- [BRS24] A. Bruni, E. Ritter, C. Schürmann, *Skolemisation for intuitionistic linear logic*, IJCAR 2024, LNCS 14740, pp. 61–77. <https://arxiv.org/abs/2405.01375>
- [Sha92] N. Shankar, *Proof search in the intuitionistic sequent calculus*, CADE-11, LNAI 607, 1992. <https://www.csl.sri.com/papers/cade92-ns/>
- [Mil92] D. Miller, *Unification under a mixed prefix*, J. Symbolic Computation 14, 1992. <https://www.lix.polytechnique.fr/~dale/papers/jsc92.pdf>
- [Mil21] D. Miller, *A survey of the proof-theoretic foundations of logic programming*, arXiv 2109.01483, 2021. <https://arxiv.org/pdf/2109.01483>
- [MO99] H. Mantel, J. Otten, *linTAP: a tableau prover for linear logic*, TABLEAUX 1999. <https://www.jens-otten.de/papers/lintap_tab99.pdf>
- [KMOS97] C. Kreitz, H. Mantel, J. Otten, S. Schmitt, *Connection-based proof construction in linear logic*, CADE 1997. <https://www.jens-otten.de/papers/linlogic_cade97.pdf>
- [MP01] R. Moot, M. Piazza, *Linguistic applications of first order intuitionistic linear logic*, JoLLI 10(2), 2001, pp. 211–232. <https://dc2.philarchive.org/rec/MOOLAO>
- [Moo13] R. Moot, *Extended Lambek calculi and first-order linear logic*, 2013. <https://arxiv.org/abs/1305.6238>
- [Moo20] R. Moot, *Partial orders, residuation, and first-order linear logic*, arXiv 2008.06351, 2020. <https://arxiv.org/abs/2008.06351>
- [Moot-LO] R. Moot, *LinearOne*, GitHub repository. <https://github.com/RichardMoot/LinearOne>
- [Tam20] N. Tamura, *A linear logic prover (llprover)*, Prolog, last changed December 2020. <https://cspsat.gitlab.io/llprover/>
- [FP-sw] F. Pfenning, *15-816 Linear Logic: software*. <https://www.cs.cmu.edu/~fp/courses/linear/software.html>
- [SS08] A. Schack-Nielsen, C. Schürmann, *Celf: a logical framework for deductive and concurrent systems*, IJCAR 2008, LNCS 5195. <https://www.itu.dk/~carsten/papers/ijcar08.pdf>
- [ODPR19] C. Olarte, V. de Paiva, E. Pimentel, G. Reis, *The ILLTP library for intuitionistic linear logic*, EPTCS 292, 2019. <https://arxiv.org/abs/1904.06850>; repository <https://github.com/meta-logic/lltp>
- [ILTP] T. Raths, J. Otten, C. Kreitz, *The ILTP problem library for intuitionistic logic*, JAR 2007 (DOI 10.1007/s10817-006-9060-z); site <https://www.iltp.de/>
- [Yalla] O. Laurent, *YALLA: an LL library for Rocq*, version 2.0.7, 2025-03-26. <https://perso.ens-lyon.fr/olivier.laurent/yalla/>
- [FFL] FormalizedFormalLogic, *Foundation* (Lean, Apache-2.0). <https://github.com/FormalizedFormalLogic/Foundation>
- [Cha12] A. Charguéraud, *The locally nameless representation*, JAR 49, 2012, pp. 363–408. <https://chargueraud.org/softs/ln>
- [dB72] N. G. de Bruijn, *Lambda calculus notation with nameless dummies*, Indag. Math. 34, 1972, pp. 381–392. DOI 10.1016/1385-7258(72)90034-0
- [CF06] S. Conchon, J.-C. Filliâtre, *Type-safe modular hash-consing*, ACM SIGPLAN Workshop on ML, 2006. <https://researchportal.ip-paris.fr/fr/publications/type-safe-modular-hash-consing/>
- [MM82] A. Martelli, U. Montanari, *An efficient unification algorithm*, ACM TOPLAS 4(2), 1982, pp. 258–282. <https://lara.epfl.ch/w/_media/sav08/unification-p258-martelli.pdf>
- [ACF] L. Albert, R. Casas, F. Fages, *Average-case analysis of unification algorithms*. <https://researchportal.ip-paris.fr/en/publications/average-case-analysis-of-unification-algorithms-2/>
- [AK91] H. Aït-Kaci, *Warren's Abstract Machine: a tutorial reconstruction*, MIT Press, 1991. <https://is.muni.cz/publication/138334?lang=en>
- Girard (1991) and Bellin and van de Wiele (1995) are known here only through [Moo20]; no copy was found.

Sources checked 2026-10-08: 31 checked, 1 corrected, 0 removed, 1 claims marked.
