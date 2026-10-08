# Research note for step 34: cut, and cut elimination

Written 2026-10-08. `[S…]` are external sources, `[R…]` files of this
repository (section 5); "(inference)" marks my own reasoning.

## 1. The problem and the state of the art

**Definitions.** The cut rule of one-sided linear logic concludes `⊢ Γ, Δ`
from `⊢ Γ, A` and `⊢ Δ, A⊥`; Girard's 1987 paper proves that every proof
reduces to one without cut and introduces proof nets as the syntax in
which that reduction is local [S1]. In linlog's dyadic calculus the rule
reads `⊢ Θ ; Γ, Δ` from `⊢ Θ ; Γ, A` and `⊢ Θ ; Δ, A⊥`, `Θ` shared as at
a `⊗` (inference from the checker's `Tensor` arm [R3]). A cut-free proof
has the subformula property, which every engine relies on: the forest
holds the sequent's subformulas only, and every searched proof is
cut-free [R1, R2].

**Sequent calculus.** The classical proof is an induction on the size of
the cut formula and the heights of the two derivations: a *principal*
case where both premises introduce the cut formula replaces the cut by
cuts on subformulas, a *commutative* case moves the cut above a rule
that does not touch it; with exponentials a cut against a contraction
duplicates the promotion's subproof and one against a weakening erases
it [S1, S24]. Pfenning's proof is three nested structural inductions,
whose computational content is a nondeterministic cut-elimination
algorithm [S13] (not in the source). Girard's view is that in sequent calculus "75% of a
cut-elimination proof is devoted to endless commutations of rules"
[S3], which is why the step wants the nets too [R1].

**MLL nets.** A cut in a proof structure is a link between two dual
conclusions; it behaves as a `⊗` for the Danos–Regnier criterion [S2],
and Hughes and van Glabbeek count "a cut as a tensor" in their balance
condition [S4]. Two reductions exist: axiom against cut (reconnect the
axiom's other literal to the cut's other premise) and `⊗` against `⅋`
(two cuts on the subformulas) [S1, S21]. Each step removes links, so the
procedure is strongly normalising and confluent (inference; the same
argument is Theorem 5.5 of [S4]). Girard's lazy procedure is linear in
the number of links [S3]. Di Guardia and Laurent formalised MLL nets in
Rocq with `cut` vertices, proved sequentialization, and defined cut
elimination with a proof that it preserves correctness and terminates
[S10].

**MALL.** Hughes and van Glabbeek's nets are sets of linkings; a cut is a
"cut pair" `A ∗ A⊥` in the sequent, and elimination has three cases: a
literal cut composes links, a `⊗/⅋` cut splits into two pairs, a `&/⊕`
cut splits and deletes the inconsistent linkings; it is confluent and
strongly normalising [S4]. Girard's additive nets have a lazy procedure
linear in the size of the net [S3]; Laurent and Maieli treat monomial
MALL nets [S22]. MALL nets are out of linlog's scope (D6a) [R6], so the
step eliminates MALL cuts on terms only.

**MELL.** With boxes the steps are: `!` against dereliction opens the
box, against contraction duplicates it, against weakening erases it,
and a box against another box's auxiliary door enters it [S1].
Guerrini and Masini's parsing presentation, which step 33 is to use, is
a confluent, strongly normalising rewriting system equivalent to the
Danos–Regnier criterion [S6, R9]. Strong normalisation for full
second-order linear logic, MELL included, was proved by Pagani and
Tortora de Falco, who note that Girard's 1987 argument relied on an
unproved standardisation theorem [S5]. Accattoli gives the first
polynomial cost model for cut elimination with unconstrained
exponentials, for IMELL [S20].

**Complexity.** The cut-elimination problem (do two proofs reduce to the
same normal form) is PTIME-complete for MLL and coNP-complete for MALL
[S7]. With exponentials the normal form can be non-elementarily larger
than the proof: the simply typed λ-calculus, whose normalisation is not
elementary recursive [S14], embeds in MELL (inference). For comparison,
MLL is in NP, MALL is PSPACE-complete and full LL is undecidable [S15].
So elimination on terms needs a bound in the sense of D16 and a
three-valued answer, as the search has [R7] (inference).

**Interaction nets and the geometry of interaction.** Lafont's interaction
nets generalise proof structures; a cut is an *active pair* and
reduction is local rewriting of such pairs [S8, S9]; the interaction
combinators are a universal system of three symbols and six rules
[S19]. The geometry of interaction reads cut elimination as the
execution formula over paths of the net, and Mackie's machine follows
those paths without rewriting [S11, S12]. Neither draws the
intermediate proof a course shows, so both are out of scope (inference).

**Why a prover wants cut.** Boolos's "Don't eliminate cut" is the classic
argument that proofs with cut can be far shorter [S23]; the complexity
results above are its linear-logic form [S7, S14]. Click & coLLecT put
cut into an interactive prover so a learner cuts on a formula of their
own and watches the reduction [S16]. For linlog: lemmas in interactive
proofs, composition of two searched proofs, and the drawing of the
reduction, which the suite cannot show today [R1].

**Implementations.**

| tool | what it does with cut | URL | licence | last release | language |
|---|---|---|---|---|---|
| Click & coLLecT (Callies, Laurent) | cut rule as an option; eliminate one cut (✄) or all; commute a cut left or right; undo/redo; export to Coq and LaTeX [S16] | https://click-and-collect.linear-logic.org/, source https://github.com/etiennecallies/click-and-collect | LGPL-2.1 | no tag or release found | OCaml backend, HTML/JS front end |
| Yalla (Laurent) | deep embedding of LL in Rocq; cut admissibility proved for LL and full ILL; a per-formula cut parameter on its working branch [S17] | https://github.com/olaure01/yalla | LGPL-3.0 | 2.0.7, 2025-03-26 | Rocq |
| proofnet_mll (Di Guardia, Laurent) | MLL proof nets in Rocq with cut vertices; cut elimination defined, preserves nets, terminates [S10] | https://github.com/RemiDiG/proofnet_mll | LGPL-3.0 | no release found | Rocq (Graph Theory, MathComp) |
| NanoYalla | `nanoll.v` has no cut; `axiomcut.v` postulates it, `yallacut.v` proves it through Yalla [R10] | Click & coLLecT's `nanoyalla/` | LGPL-2.1 | 1.1.3 [R10] | Rocq |
| cslib (Lean 4) | classical LL with a `cut` rule and `CutFreeProof`; cut admissibility and elimination are TODO comments [S18] | https://github.com/leanprover/cslib | Apache-2.0 | no release | Lean 4 |
| HVM2 | interaction-combinator runtime, no logic [S19] | https://github.com/HigherOrderCO/HVM2 | Apache-2.0 | crate `hvm` 2.0.22 | Rust, C, CUDA |

I found no tool that draws cut elimination on proof nets step by step;
Click & coLLecT does it on sequent derivations [S16].

## 2. What the step needs

**The cut node.** `Node` is a 16-byte `Copy` enum under a static
assertion [R2]; `Cut(OccId, OccId, NodeId, NodeId)` is 17 bytes with
its tag. Recommendation: `Cut(OccId, NodeId, NodeId)`, naming the
occurrence of `A` whose proof is the left premise, the dual found
through the forest (next item). The assertion stays and D17's
propositional cost holds [R6].

**Dual pairs by offset.** In negation normal form `A` and `A⊥` have the
same tree shape, since duality flips every kind and keeps the children
(`Kind::dual`, [R4]), and the forest numbers a subtree contiguously in
DFS preorder [R5]. So if the two trees of a cut pair are appended one
after the other, `dual(x) = root(A⊥) + (x − root(A))` for every `x`
under `A`, in O(1) with no table (inference). That makes every
reduction local: the `⊗/⅋` step turns a cut on `o` into cuts on
`left(o)` and `right(o)`, whose duals need no lookup. Keep it as an
invariant of the extended forest and test it.

**Extending the forest.** `Forest` is built from a `Sequent` only and
`roots` are the sequent's [R5]. `Sequent::add` appends roots, but
`Sequent::optimize` sorts roots and hash-conses, which would move ids
against D5 [R4, R6]. Recommendation: `Forest::with_cuts(&sequent,
&[Formula])` lays the sequent's forest out as today, appends `A` and
`A⊥` per cut formula as extra trees, keeps `roots()` as the sequent's
and exposes `cut_pairs()`; a proof, a net and an interactive state
record their cut formulas beside the sequent and rebuild the same
forest. Cost: about 25 bytes per occurrence [R5]; `Forest::DEFAULT_LIMIT`
applies to the sum.

**The checker's arm.** `Pass::rule` for `Cut(o, l, r)`: `take` `o` from
the left state and `dual(o)` from the right, `join` as `Tensor` does,
`put` nothing [R3]. The `Problem::Surplus` argument ("a rule takes two
members of a premise's zone at most") still holds (inference). In
intuitionistic mode the cut formula is the goal of one premise and a
hypothesis of the other, so the `Reading` gives the `A` root output
position and `A⊥` input, and R1–R3 apply unchanged (inference from
[R3, R11]); the user chooses the side by giving `A`. `oracle.rs` gets
the same arm, since the differential test requires the same error [R3].

**Elimination on terms.** The arena is a DAG with shared subproofs [R2],
and a rewriting step changes one occurrence of a subproof: either the
subproof is unshared first (copy on write) or the step applies to every
sharer. Recommendation: an owned `Vec<Node>` arena where the redex's
path to the root is copied and the rest stays shared, new nodes
appended, and `Proof::new` renumbering and dropping what the root no
longer reaches [R2]. Cost per step: the principal cases are O(1) nodes
plus, for the axiom case, a renaming pass over the subproof whose
occurrences move from the cut tree to the sequent's (by offset); a
commutative step copies the cut node only; a `&` commutation and a
contraction case duplicate a subproof by reference, and later steps
inside one copy unshare it. The run is bounded by an options value
(`steps`, `memory`, the strategy), and a run at its bound answers a
third value, not a proof (D16) [R6]. Each intermediate term is a `Proof`
over the same extended forest and passes `Proof::check`; the last has
no `Cut` node. Strategy: the lowest cut first, principal cases before
commutations (the measure of [S1, S24]); and a `step` on an explicit
cut node, as Click & coLLecT lets the user pick the cut and the
direction of a commutation [S16].

**Elimination on MLL nets.** `ProofStructure` has `partner` for literals,
a stack of links, the CSR `Graph` with a parent slot, children and an
axiom slot per literal, and the `⅋`-free `Skeleton` [R8]. A cut is a new
kind of link between the two roots of a cut pair, which the criterion
treats as a `⊗` [S2, S4]: the graph needs a cut slot on a root vertex,
the skeleton unites the two roots, and the connectedness count
`2t + p + k = V − 1` counts a cut as a tensor [R8]. Steps: axiom against
cut relinks two literals (two `partner` writes); `⊗/⅋` deletes two
connective vertices and makes two cuts (O(1) with the offset map).
Deleted occurrences leave without renumbering: an `erased` bitset over
the forest that `is_correct` and the drawing skip, so a step is a diff.
Correctness is preserved by theorem [S4, S10]; a `debug_assert!` per
step and a full check at the end suffice [R8]. The final structure,
every cut tree erased, sequentializes to a cut-free `Proof` [R8].

**MELL nets.** The contraction step duplicates a box, so the forest must
grow during elimination; whether step 33's representation allows that
is unknown [R9]. Recommendation: term-level elimination complete for
MELL, net-level complete for MLL, the exponential steps on nets the
third session's stretch goal.

**What the output is and how it is checked.** A run returns the sequence
of proofs (or nets) over one extended forest, plus a record per step
(the cut node, the case, the nodes erased or made), so a front end
draws step `i` and the diff to `i + 1`. Checks: every proof passes
`Proof::check(mode)`, whose root condition (the linear zone equals the
sequent's roots) is the preservation of the conclusion; the last has no
`Cut`; for nets `is_correct` per step and `sequentialize` at the end;
termination by a counter against the bound. Tests on generated proofs
with cuts [R1]: compose two searched proofs (`prove_goal` on `⊢ Γ, A`
and `⊢ Δ, A⊥` over the extended forest, joined by a `Cut`) and permute
rules below cuts for the commutative cases.

## 3. What linlog's library must offer

**Formulas and sequents** (`core/src/sequents/`). `Sequent` is an arena
in NNF with `roots`; `Kind::dual` gives the dual kind [R4]. Missing: a
public way to build the dual of a cut formula (the two-sided lowering
negates, but not as a public item) and a `Formula`-typed argument for
the cut, so that a later binder is a term and not a string (D17) [R6].

**The forest** (`core/src/occurrences/mod.rs`). `Forest::within` from a
`Sequent` only; `roots`, `root(o)`, `size`, `literals` in CSR [R5].
Missing: the constructor with cut pairs, `cut_pairs()`, `dual(o)` by
offset, and a range telling the sequent's roots from cut roots. In the
way: `Reading::new(&forest)` chooses the goal among *all* roots [R11],
so it must take only the sequent's roots and assign the cut pair's
positions as section 2 says.

**The proof term** (`core/src/proofs/mod.rs`). `Node` with fourteen
variants and its `match`es (`principal`, `occurrences`, `premises`,
`name`, `Display`, `map_premises`); `Proof { forest, nodes }` [R2].
Missing: `Cut(OccId, NodeId, NodeId)` in every match, and the proof's
forest being the extended one.

**The checker** (`core/src/proofs/check.rs`). `Pass::rule` with one arm
per node, `take`/`put`/`join`, `Facts`, `Problem`, the root condition
`gamma == roots` [R3]. Missing: the `Cut` arm (and in `oracle.rs`) and
a `Problem` for a cut on no dual pair (`NotDual` may serve). `State`
and the memory account need nothing new.

**Derivations and the views** (`derivation.rs`, `size.rs`, `fmt.rs`,
`style.rs`). `Rule` has 34 variants, `Rule::ALL`, the `UPRIGHT` and
`SUBSCRIPT` tables, the `name`/`from_str` round trip;
`Inference.principal` is `None` for `ax` and Mix [R12, R13]. Missing:
`Rule::Cut` with `principal: None` (the formula is not in the
conclusion), its label entries, the size estimate's arm, and the Rocq
exporter's policy: NanoYalla's `nanoll.v` has no cut [R10], so
`rocq::write` refuses a cut with `Unsupported` unless `rocq::Options`
selects a kernel with cut (`macrollcut`, or the step-31 library, whose
datatype gains the case [R14]).

**Interactive proving** (`proofs/interactive.rs`). `Interactive::new(&sequent, mode)`
owns the forest; `rules` and `apply` address a formula by position;
`Terms` translates the derivation to a term; `from_parts` replays every
inference [R13]. Missing: `cut(goal, formula, left)` that extends the
state's forest (so the forest must be replaceable), a `Refusal` for a
cut formula the fragment or mode forbids, the `Cut` node in `Terms`,
and the cut formulas in the state's JSON so `from_parts` rebuilds the
same forest. `close(goal)` calls `prove_goal`, which accepts any
multiset of a forest [R7], so a goal holding a cut root is searched
without change (inference), the net engine excepted [R7].

**The engine interface and the dispatch** (`search/mod.rs`). `Decide`,
`Task`, `prove_goal`, `DISPATCH`, `Outcome` [R7]. Nothing changes: the
search stays cut-free [R1], and `goal_fragment` over the goal's
subtrees covers cut roots (inference). Cut elimination is not an
engine but a module beside `proofs` (`proofs/cut.rs`, `nets/cut.rs`)
with its own options value, as the view has `ViewOptions` [R12].

**Proof nets** (`nets/`). `ProofStructure::new` refuses non-MLL; `link`,
`unlink`, `from_links`, `from_proof` reads `Ax` only; `is_correct`,
`sequentialize`; JSON `{sequent, mix, links}` [R8]. Missing: cut links
between root pairs, the graph's cut slot and count, `from_proof`
reading `Cut` nodes, `sequentialize` treating a cut as a splitting
tensor, the `erased` set, `NetError` variants, `cuts` in the JSON, and
`svg::net` drawing a cut as an arc below the conclusions, since links
are arcs above the literals [R15].

**JSON wire forms** (`serialize/proofs.rs`, `nets.rs`, `interactive.rs`).
A proof is `{"sequent", "proof": [{"ax": …}, …]}` with one tag per rule
[R16]. Missing: `"cuts": [formula, …]` beside the sequent (absent in
old files, which read unchanged) and `{"cut": [o, l, r]}`; the same
`cuts` on the net and the interactive state; a form for an elimination
trace.

**Options** (D15, D16). `ViewOptions` and the export options are the
pattern: plain data, `Default`, serde with `deny_unknown_fields` [R12,
R15]. Missing: `cut::Options { steps, memory, strategy }` with measured
defaults, a `cut` command in `interact` (an arm in `Session::command`
[R17]) and a command that eliminates the cuts of a proof file step by
step or to the end.

## 4. Risks, open questions, and what the prompt should add

Risks:
- The `Node` size assertion and D17's cost: the 13-byte `Cut` avoids a
  layout change; two occurrences in the node would not [R2, R6].
- Id stability (D5): every path that rebuilds a forest from JSON must
  append the cut pairs in the recorded order [R5, R16].
- The `Reading` with cut roots: a wrong goal choice makes every
  intuitionistic cut proof `Problem::Succedents` [R11].
- Non-elementary blow-up with exponentials [S14]: a default bound and
  a refusal that is no verdict (D16); sharing must not make a step
  quadratic.
- Nets: the criterion with cuts is standard [S2, S4], but the
  coloured-graph test was argued for tree edges and axiom links [R8]
  and must be re-argued with cut edges (own colour, like a `⊗` edge).
- MELL on nets needs a growing forest; step 33's shape is unknown [R9].
- Affine mode (a cut against `Weaken` erases a premise) and Mix (a cut
  commutes with Mix) need arms and tests [R3].
- Rocq: the step-31 library gains the case [R14]; the NanoYalla path
  refuses cuts or imports `macrollcut` and its axiom [R10].

Open questions:
1. One extended forest per proof, or a forest that grows on each `cut`
   of an interactive session?
2. Every intermediate proof, or a trace of diffs with proofs rebuilt on
   demand, and under which bound?
3. Which strategy is the default, and does the user pick the cut and
   the direction of a commutation as in Click & coLLecT [S16]?
4. Rocq export of a proof with cuts: refuse, or a kernel option?
5. Do MELL cuts on nets belong to this step or to a follow-up?
6. Does the net's `erased` set live in `ProofStructure` or in the
   elimination's wrapper?

What the prompt should add: the node layout and the dual-by-offset
invariant; the forest constructor and the `cuts` field of every JSON
form; that the search stays cut-free and the net engine never sees cut
roots; the bounded options value with a measured default; the test
design (composed searched proofs, every step checked, termination
counted); the Rocq policy; the scope of the third session on MELL nets.

## 5. Sources

External:
- [S1] J.-Y. Girard, "Linear logic", *Theoretical Computer Science* 50(1), 1987, 1–101. https://doi.org/10.1016/0304-3975(87)90045-4
- [S2] V. Danos, L. Regnier, "The structure of multiplicatives", *Archive for Mathematical Logic* 28, 1989, 181–203. https://doi.org/10.1007/BF01622878
- [S3] J.-Y. Girard, "Proof-nets: the parallel syntax for proof-theory" (1996; cited as [Gir96] in [S10]). https://girard.perso.math.cnrs.fr/Proofnets.pdf
- [S4] D. J. D. Hughes, R. J. van Glabbeek, "Proof nets for unit-free multiplicative-additive linear logic", *ACM Transactions on Computational Logic* 6(4), 2005, 784–842. https://doi.org/10.1145/1094622.1094629 (PDF: https://cgi.cse.unsw.edu.au/~rvg/mallTOCL/p1-d_hughes.pdf)
- [S5] M. Pagani, L. Tortora de Falco, "Strong normalization property for second order linear logic", *Theoretical Computer Science* 411(2), 2010, 410–444. https://doi.org/10.1016/j.tcs.2009.07.053
- [S6] S. Guerrini, A. Masini, "Parsing MELL proof nets", *Theoretical Computer Science* 254(1–2), 2001, 317–335. https://doi.org/10.1016/s0304-3975(99)00299-6
- [S7] H. G. Mairson, K. Terui, "On the computational complexity of cut-elimination in linear logic", ICTCS 2003, LNCS 2841, 23–36. Abstract at https://www.kurims.kyoto-u.ac.jp/~terui/pub_bib.html
- [S8] Y. Lafont, "Interaction nets", POPL '90, 95–108. https://doi.org/10.1145/96709.96718
- [S9] Y. Lafont, "From proof nets to interaction nets", in *Advances in Linear Logic*, LMS Lecture Notes 222, 1995, 225–248. https://www.cambridge.org/core/books/advances-in-linear-logic/from-proof-nets-to-interaction-nets/8C2C748AFE97D1E5B7310AB855CCB51B
- [S10] R. Di Guardia, O. Laurent, "A Formalization of Multiplicative Proof-Nets in Rocq", TLLA 2025. https://perso.ens-lyon.fr/olivier.laurent/rocqpn.pdf ; code https://github.com/RemiDiG/proofnet_mll
- [S11] V. Danos, L. Regnier, "Proof-nets and the Hilbert space", in *Advances in Linear Logic*, 1995, 307–328. https://www.cambridge.org/core/books/abs/advances-in-linear-logic/proofnets-and-the-hilbert-space/4C074845DBBF5BA14529DE8FFFD8DDF3
- [S12] I. Mackie, "The geometry of interaction machine", POPL '95 (programme listing). https://compilers.iecc.com/comparch/article/94-09-185 ; overview: https://en.wikipedia.org/wiki/Geometry_of_interaction
- [S13] F. Pfenning, "Structural cut elimination in linear logic", CMU-CS-94-222, 1994; "Structural cut elimination", LICS 1995, 156–166. https://lics.siglog.org/1995/Pfenning-StructuralCutElimin.html
- [S14] R. Statman, "The typed λ-calculus is not elementary recursive", *Theoretical Computer Science* 9(1), 1979, 73–81. https://doi.org/10.1016/0304-3975(79)90007-0
- [S15] P. Lincoln, J. Mitchell, A. Scedrov, N. Shankar, "Decision problems for propositional linear logic", *Annals of Pure and Applied Logic* 56, 1992, 239–311. https://www.csl.sri.com/papers/lmss90/lmss90.pdf
- [S16] E. Callies, O. Laurent, Click & coLLecT: site https://click-and-collect.linear-logic.org/ ; source https://github.com/etiennecallies/click-and-collect ; cut-elimination exercises https://github.com/etiennecallies/click-and-collect/wiki/Cut-Elimination ; paper (2021) https://hal-lirmm.ccsd.cnrs.fr/lirmm-03271501
- [S17] O. Laurent, Yalla: https://perso.ens-lyon.fr/olivier.laurent/yalla/ ; https://github.com/olaure01/yalla ; https://rocq-prover.org/p/rocq-yalla
- [S18] leanprover/cslib, `Cslib/Logics/LinearLogic/CLL/CutElimination.lean`. https://github.com/leanprover/cslib ; https://raw.githubusercontent.com/leanprover/cslib/main/Cslib/Logics/LinearLogic/CLL/CutElimination.lean
- [S19] HigherOrderCO, HVM and HVM2: https://github.com/HigherOrderCO/HVM ; https://github.com/HigherOrderCO/HVM2 ; https://docs.rs/crate/hvm/latest ; Y. Lafont, "Interaction combinators", *Information and Computation* 137(1), 1997, 69–101. https://doi.org/10.1006/inco.1997.2643
- [S20] B. Accattoli, "Exponentials as Substitutions and the Cost of Cut Elimination in Linear Logic", *Logical Methods in Computer Science* 19(4), 2023. https://arxiv.org/abs/2205.15203
- [S21] P.-L. Curien, "Introduction to linear logic and ludics, part II", 2005. https://arxiv.org/abs/cs/0501039
- [S22] O. Laurent, R. Maieli, "Cut elimination for monomial MALL proof nets", LICS 2008. https://lics.siglog.org/2008/LaurentMaieli-CutEliminationforMo.html
- [S23] G. Boolos, "Don't eliminate cut", *Journal of Philosophical Logic* 13, 1984, 373–378. https://doi.org/10.1007/BF00247711
- [S24] A. S. Troelstra, *Lectures on Linear Logic*, CSLI Lecture Notes 29, 1992 (1990 report: https://eprints.illc.uva.nl/593).

This repository (the snapshot of 2026-10-08):
- [R1] `plan/34-cut.md`; `plan/reports/17-assessment.md` §5.5.
- [R2] `core/src/proofs/mod.rs` (`Node`, the size assertion, `Proof`, `Proof::new`).
- [R3] `core/src/proofs/check.rs` (`State`, `Pass::rule`, `take`, `join`, `Problem`); `.claude/rules/core-proofs.md`.
- [R4] `core/src/sequents/mod.rs` (`Sequent`, `optimize`, `add`); `core/src/sequents/term.rs` (`Kind::dual`).
- [R5] `core/src/occurrences/mod.rs` (`Forest`, `within`, `roots`); `.claude/rules/core-forest.md`.
- [R6] `plan/README.md`, decisions D5, D6, D6a, D13, D15, D16, D17, D18, D23.
- [R7] `core/src/search/mod.rs` (`Engine`, `Options`, `Outcome`, `prove_goal`); `.claude/rules/core-search.md`.
- [R8] `core/src/nets/mod.rs` (`ProofStructure`, `from_proof`); `.claude/rules/core-nets.md`; `plan/reports/05-proof-nets.md`.
- [R9] `plan/33-mell-nets.md`; `plan/later.md`, "MELL proof nets with exponential boxes".
- [R10] `plan/reports/12-certificates.md` (NanoYalla's files and the `macroll`/`macrollcut` decision).
- [R11] `core/src/occurrences/reading.rs`; `.claude/rules/core-forest.md`, "The intuitionistic reading".
- [R12] `core/src/proofs/derivation.rs` (`Rule`, `Inference`); `.claude/rules/core-derivations.md`.
- [R13] `core/src/proofs/interactive.rs`; `core/src/serialize/interactive.rs`.
- [R14] `plan/31-rocq-library.md`, "What comes later".
- [R15] `core/src/export/rocq.rs`; `core/src/export/svg/net.rs`; `.claude/rules/core-export.md`.
- [R16] `core/src/serialize/proofs.rs`; `core/src/serialize/nets.rs`.
- [R17] `.claude/rules/cli.md`, "Extension points".

Sources checked 2026-10-08: 24 checked, 1 corrected, 0 removed, 1 claims marked.
