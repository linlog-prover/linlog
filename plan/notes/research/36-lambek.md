# Research note for step 36: cyclic MLL and the Lambek calculus

Written 2026-10-08 against the snapshot of that day, for the prompt of
step 36 and the API design of step 28. Code claims name the file and
item; literature claims cite section 5; "(inference)" marks a
conclusion of this note.

## 1. The problem and the state of the art

**Definitions.** The Lambek calculus L (Lambek 1958) has types built
from primitive types with a product `•` and two divisions `\` and `/`;
a sequent `Π → A` has a non-empty sequence `Π` on the left and one type
on the right, and there is no exchange: the rules `(→ /)`, `(→ \)`,
`(/ →)`, `(\ →)` insert and remove types at the ends or in the middle
of the sequence (Savateev 2012, section 1). L* is the same calculus with
empty antecedents allowed; the product-free fragment `L(\, /)` drops the
product, and the unidirectional fragments keep one division (Savateev
2012). Lambek's restriction is a linguistic choice: with empty
antecedents `1 → n/n` is derivable, so "very book" would be a noun
phrase (Kanovich, Kuznetsov, Morrill and Scedrov 2017). Cyclic linear
logic (Yetter 1990) is one-sided classical linear logic with exchange
restricted to cyclic permutations; Abrusci's non-commutative classical
linear logic (Abrusci 1991) drops exchange altogether and has two
negations; both are conservative extensions of the Lambek calculus
(Abrusci 2002). The spec's section "MLL
variants" states the consequence the step relies on: derivability of the
multiplicative fragments of cyclic and non-commutative LL coincides with
L* (proof-search-specifications.md). In the one-sided form the two
divisions are the two orders of one `⅋` with a negated factor, `A\B` as
`A⊥ ⅋ B` and `B/A` as `B ⅋ A⊥`, and a two-sided `A₁, …, Aₙ ⊢ C` is the
cycle `⊢ Aₙ⊥, …, A₁⊥, C`, the negation of a product reversing the order
of its factors (inference from the cyclic systems of Yetter 1990 and
Abrusci 1991; the prompt should have the panel confirm the exact
lowering against Abrusci 2002).

**Complexity.** Derivability in L and L* is NP-complete (Pentus 2006).
The product-free fragments `L(\, /)` and `L*(\, /)` are NP-complete too,
by a reduction from SAT (Savateev 2012, first published at LFCS 2009).
The unidirectional fragments have a polynomial membership algorithm for
a fixed grammar (Savateev 2010); Savateev's 2011 Izvestiya paper proves
NP-completeness of derivability for the unidirectional fragment when the
types are part of the input, so the two results concern different
problems (search summary of Savateev 2010 and MathNet im4118). With
additives the calculus is PSPACE-complete (Kanovich and Kanazawa, cited
in Kanovich, Kuznetsov and Scedrov 2019), and one implication with `∧`
or with `∨` already is (Kanovich, Kuznetsov and Scedrov 2019). Bounded
order is the polynomial escape: Pentus (2010) decides L* sequents of
bounded order in polynomial time by tabulating proof nets; Fowler (2008,
2010) gives O(n⁵) and then O(n³) derivability and O(n⁴) parsing for the
product-free calculus of bounded order, and Kanovich, Kuznetsov, Morrill
and Scedrov (2017) extend Pentus's method to brackets, in time
poly(N, 2^R, N^B) for size N, order R and bracket depth B, so polynomial
only when R and B are fixed. Lambek grammars generate exactly the
context-free languages (Pentus 1993, 1997).

**Proof nets.** Roorda's thesis (1991) and article (1992) give proof
nets for the Lambek calculus; Lamarche and Retoré (1996) survey them;
Moot and Retoré (2012, chapter 6) is the current textbook account,
building on Moot's thesis (2002). Correctness for cyclic and
non-commutative MLL nets is the Danos–Regnier condition plus planarity:
Nagayama and Okada (2001) give a criterion on marked Danos–Regnier
graphs that brings the check from quadratic to linear time; Melliès
(2004) gives a topological criterion, extends the planarity criterion
of cyclic MLL to multiplicative non-commutative logic and shows it
equivalent to Abrusci and Ruet's long-trip criterion. Kanovich, Kuznetsov, Morrill and Scedrov (2017)
state the net definitions the step can copy: a proof structure is a
pairing of the literal occurrences; it is planar when the pairing can be
drawn in a half-plane without crossings with the literals on the border
in their order; and it is a net when every region has exactly one `⅋`
on its border and the graph linking each `⊗` to its region's `⅋`,
together with the dominance order, is acyclic (Pentus's conditions, in
the framework of Fadda and Morrill). Moot and Puite (2002) give nets for
the multimodal Lambek calculus; Morrill and Fadda (2008) for the
discontinuous one. Neither source read gives the net-side form of
Lambek's restriction (L against L*) explicitly; the step must take it
from Roorda (1992) or Moot and Retoré (2012) (see section 4).

**Sequent search.** The cut-free sequent calculus has spurious
ambiguity; Hepple (1990) gives a normal form with one proof per
reading. Moot (2008) picks an optimal axiom link in O(n⁴) during net
search.

**Implementations.**

| tool | what | URL | licence | last activity | language |
|---|---|---|---|---|---|
| Grail 3 | theorem prover for multimodal type-logical grammars, proof-net based (Moot) | github.com/RichardMoot/Grail | LGPL-2.1 | release 3.2.0 of 2015-12-01; last push 2021-02-11 | SWI-Prolog |
| Grail Light | chart parser for multimodal type-logical grammars | github.com/RichardMoot/GrailLight | LGPL-2.1 | last push 2026-09-17 | SWI-Prolog |
| LinearOne | prover for first-order MILL, with hybrid and Displacement grammars by translation; `lambek_grammar.pl` in the tree | github.com/RichardMoot/LinearOne | LGPL-2.1 | last push 2026-06-11 | SWI-Prolog |
| CatLog3 | parser/theorem-prover for type-logical grammar (Morrill 2019) | cs.upc.edu/~morrill (manual and examples; no download link or licence found) | not stated | manual dated 2018 | Prolog |
| coq-lambek | three formalisations of L in Coq with certified translations between them | github.com/coq-contribs/lambek | LGPL-2.1 | version 8.10.0 of 2019-12-07 for Coq 8.10; last push 2024-07-24 | Coq |
| Yalla | deep embedding of linear logic in Rocq, with a parameter for cyclic permutations only, and "Lambek calculus included when permutation is equality" in `ill_def.v` | github.com/olaure01/yalla | LGPL-3.0 | active (no release listed) | Rocq |
| unicode_fol_kit | a Python package with a memoised exhaustive Lambek prover for L (non-empty antecedents) | unicode-fol-kit.readthedocs.io | not found | not found | Python |

The `lambekseq` prover the spec names could not be found; that claim
is unverified. Zhao and Penn (2025) release a bounded-order product-free
parser as open source and say the earlier approaches had no open-source
implementations; no repository link was found. Only LinearOne and Grail
Light moved in 2026.

## 2. What the step needs

**The planar linking search.** The spec's design is MLL-Net with a
planarity constraint: order the literals as they appear in the cyclic
sequent and admit a link `x–y` only if no existing link `u–v` crosses
it, `x < u < y < v` or `u < x < v < y` in the cyclic order
(proof-search-specifications.md). In linlog the forest's preorder gives
that order for free: ascending `OccId` is the left-to-right order of
the leaves with the roots in the sequent's order (`core-forest.md`,
"Numbering is DFS preorder"). Costs: a crossing test against the stack
of links is O(k) per candidate with k links made, O(log n) with an
interval structure; the exact Yeo test after every link
(`Options::test_period`) dominates either way (inference from
`core-nets.md`). Planarity cuts a literal's candidates to the dual
literals of its region (the spec: it "removes almost all linkings");
the worst case stays exponential, as Pentus (2006) requires. Recommendation: the crossing
test as a third constant-time rejection beside the LCA and the skeleton
(`core-nets.md`, "Two constant-time rejections"), an O(k) scan over the
link stack first, measured before anything cleverer, because the exact
test costs O(n) per link anyway.

**The count equation stays.** A cyclic proof is an MLL proof, so
`c = t − p + 2` and the per-atom balance (`counts_admit`,
`core/src/search/net.rs`) remain necessary (inference). No stronger
count prune for the cyclic case was found in the literature.

**The contiguity of the `⊗` split.** In the cyclic calculus the rule
`⊢ Γ, A ⊗ B, Δ` from `⊢ Γ, A` and `⊢ B, Δ` splits the context into two
arcs around the tensor, so there are at most n + 1 splits of a context
of n formulas instead of 2ⁿ (inference from the rules in Savateev 2012,
section 1, read one-sided). A sequent-style cyclic engine would be far
cheaper than the focused engine's submask enumeration, but it is a
second engine for one fragment and this step builds the planar net
search; the sequent form matters for the checker (section 3).

**Bounded-order algorithms** (Pentus 2010; Fowler 2010; Kanovich et al.
2017) are polynomial only with the order fixed, and stayed theoretical
until Zhao and Penn (2025). Recommendation: not in this step; record
them in `plan/later.md` as the route for grammar workloads.

**The other route**, the embedding of L into first-order MILL (Moot and
Piazza 2001; Moot 2013), needs quantifiers and is step 38's business,
as the prompt fixes. Note the title: the paper is "Linguistic
applications of first order *intuitionistic* linear logic", not
"multiplicative" as `plan/36-lambek.md` and `17-assessment.md` write.

**L against L*.** The net engine on the lowered sequent decides L*
(the spec; Abrusci 2002). Lambek's restriction is an extra condition.
Recommendation: implement it as a rule of the checker and of the
sequentialization, "every sequent of the derivation has a hypothesis"
(an input occurrence under the `Reading`), which is sound by
construction, and have the panel settle whether a planar net of an
L*-provable sequent is L-provable exactly when some sequentialization
meets it, or whether a net-level condition from Roorda (1992) or Moot
and Retoré (2012) is needed to keep the search complete (section 4).

**Units.** L with the unit is outside unit-free MLL, so outside the net
engine; Kuznetsov (2017) eliminates the unit by translating into the
calculus with empty antecedents, but states it for the bracketed
calculus. Recommendation: refuse units in cyclic mode in this step with
a message, and note the translation as a follow-up to verify for the
plain calculus.

**Certificates.** NanoYalla has only the full exchange `ex_perm_r`
(`core/src/export/rocq.rs`), so no cyclic certificate exists on it;
Yalla's cyclic-permutation parameter and coq-lambek are targets for the
Rocq library of step 31, not for this step.

## 3. What linlog's library must offer

**Formulas and sequents.** `Sequent` (`core/src/sequents/mod.rs`) keeps
`roots` "in the order the sequent lists them", but `optimize_roots`
sorts them and `optimize` runs after every parse (`core-sequents.md`).
The fix the prompt fixes is to stop sorting: the written order is the
canonical form in every mode. Cost: the occurrence ids of every stored
proof and snapshot whose roots were not already sorted shift (the
`core/tests/snapshots`, `core/tests/serialize.rs`, `cli/tests/readme.rs`
outputs), which the report must list; hash-consing (`optimize_terms`)
is unaffected, since the forest unfolds shared terms. `Term::dual` (`core/src/sequents/term.rs`) maps
`Tensor(k, l)` to `Par(k, l)`: it keeps the factor order, which the
commutative logic allows and the cyclic one does not (inference from
Yetter 1990). It has no mode, and the parser applies it in
`Parser::finish` (`core/src/parse/mod.rs`). So the lowering must become
mode-aware or order-neutral: either a `Term::dual_reversed` used by a
cyclic lowering, or the parser builds the cyclic one-sided sequent
directly (the left side reversed and negated, factors swapped under a
negation). A JSON sequent is already one-sided, so its author did the
lowering; the `Sequent` docs must say which. The parser has one
implication, `Binary::Lollipop`, pushed as `⅋` with its antecedent
flagged negative; Lambek needs `\` and `/` as two tokens (`A\B` as
`A⊥ ⅋ B`, `B/A` as `B ⅋ A⊥`), and `⊸` may stay as a spelling of one of
them in commutative modes. `Display` of `Sequent` and `Formula`
(`core/src/sequents/fmt.rs`) must print the two divisions two-sided, and
`Notation::term` and `Notation::ill` (`core/src/export/notation.rs`)
need the symbols for LaTeX, Typst and SVG, as `core-export.md` asks for
any new symbol.

**Fragments and modes.** `Fragment` (`core/src/fragment.rs`) is the
connective classes and is right as it is: order is not a connective.
`Mode` is three bools with a `Display` and the JSON
`{"intuitionistic", "affine", "mix"}` (`ModeDef`,
`core/src/serialize/search.rs`). Recommendation: a fourth field,
`cyclic: bool` (the exchange restricted to rotations), with
`serde(default)` so old files read, and `Mode::LAMBEK` as
`intuitionistic` plus `cyclic` plus Lambek's restriction; whether the
restriction is a fifth field (`empty_antecedents`, default false in
Lambek mode, so L is the default and L* the option, D16) or implied by
the pair is for the API design; a field is the D16 reading.
`Fragment::name_in(mode)` gains the names "cyclic MLL" and "Lambek",
and the `Outcome` JSON uses them. The CLI's `ModeArgs`
(`cli/src/argument_parsing.rs`) gains the flags.

**Occurrences and the reading.** `Forest` (`core/src/occurrences/mod.rs`)
needs nothing new: the preorder is the cyclic order, `literals(atom,
sign)` lists in ascending id, and `lca` is the within-root rejection.
`Reading` (`core/src/occurrences/reading.rs`) reads `b ⅋ ~a` as `a ⊸ b`
too ("the symmetric reading"); in cyclic mode the two readings are the
two divisions and must stay apart, so `Reading::implication` must
return the antecedent by position, not by which reading works
(inference). The reading's goal is the last root that can be output,
which matches a lowering that puts `C` last. The one-succedent
condition R1 to R3 of the checker (`core-proofs.md`) is reused; Lambek
adds R4, at least one input occurrence per derived sequent.

**Terms, checker and derivations.** `Node` (`core/src/proofs/mod.rs`)
needs no new variant: `Tensor(o, l, r)` names the premises of the left
and right factor, and the cyclic rule is the same node with a condition
on the contexts. The checker's `State.gamma` is a multiset `Bag`
(`core/src/proofs/check.rs`), and the cyclic order need not be stored:
for cut-free MLL no occurrence repeats, so the ascending ids of any
derived sequent are a valid linearisation of its cyclic order
(inference: a sub-multiset of a cyclically ordered sequence inherits the
order, and subformulas take their parent's place). What the cyclic check
adds is one test per `⊗` node: in the cyclic order of the two premises'
contexts and the tensor, read from the tensor, the right premise's
context comes first, then the left premise's; one sort per `⊗`, O(w log
w) in the width. At the root the sequent must be the roots up to
rotation (`is_roots` in `core/src/search/mod.rs` accepts any order
today). `Weaken`, `Mix` and `Copy` are outside unit-free MLL anyway;
Mix in cyclic mode needs a placement and should be refused. The oracle
`core/src/proofs/oracle.rs` gets the same rule, as `core-proofs.md`
requires. The derivation view's `Inference.sequent` (ascending ids,
`core/src/proofs/derivation.rs`) stays correct by the same argument;
the two-sided renderer must rotate so the goal is last and print the
hypotheses reversed, and `Rule` needs `\L`, `\R`, `/L`, `/R` beside
`ImpLeft`/`ImpRight`: new entries in `Rule::ALL`, `name`, `from_str`,
the `UPRIGHT` and `SUBSCRIPT` tables (`core/src/proofs/style.rs`) and
the Rocq printer's arms, as `core-export.md` lists for a new rule.
`Interactive` (`core/src/proofs/interactive.rs`) addresses formulas by
position in the ascending sequent, which is the cyclic position; its
`⊗` split takes "the positions going left" and must take a cut position
in cyclic mode, which `expand` validates.

**Nets and the net engine.** `ProofStructure` (`core/src/nets/mod.rs`)
holds `mix` as a flag that changes the criterion; `cyclic` is a second
such flag, and `is_correct` adds planarity (a crossing scan over
`links`) before the Yeo test; the JSON `{"sequent", "mix", "links"}`
gains `"cyclic"`. The engine (`core/src/search/net.rs`) adds the
crossing test to the admissibility of a candidate in `choose` and
`next_partner`, under the `Frame` stack and the undo order the structure
requires. Two things break: the symmetry breaking for equal literal
conclusions (`copy_before`, `copy_after`) rests on a swap being an
automorphism, which an ordered structure has not, so it must be off in
cyclic mode; and `Feature::FewEqualLiterals` is moot when the net
engine is the only cyclic engine.
`sequentialize.rs` picks the splitting `⊗` with the smallest id and
sends the conclusions reached from its left premise left; in a planar
net the two parts are the two arcs (inference, for the panel), and the
Lambek restriction is checked per stage. The SVG drawing
(`core/src/export/svg/net.rs`) already puts literals in occurrence
order and draws links as arcs, so a planar net draws without crossings,
except a link that wraps around the cycle, which the layout must allow.

**Engine interface and dispatch.** `Task` carries `goal`, `fragment`,
`mode`, `reading`, `roots` (`core/src/search/mod.rs`); the mode carries
the new flag, so `Decide::admits` of Focus, TwoSided, Additive and Horn
refuses a cyclic goal with a new `Error` variant, and `DISPATCH` gains a
row for unit-free MLL in cyclic modes to `Engine::Net`, with `Modes`
extended by a cyclic case. `prove_goal` takes `goal: &[OccId]` "in any
order"; in cyclic mode the slice is the cyclic order and `is_roots`
compares up to rotation. `Refutation` and `Statistics` need nothing;
`Options` needs nothing new (`test_period` applies).

## 4. Risks, open questions, what the prompt should add

- **The net-side Lambek restriction.** Not pinned by the sources read.
  The prompt should name Roorda (1992) and Moot and Retoré (2012,
  chapter 6) as the texts to settle it from, and require the panel's
  argument that the checker's rule R4 plus the engine's search is
  complete for L, or a net-level criterion otherwise.
- **The lowering.** The one-sided form of `A₁, …, Aₙ ⊢ C` and of the
  negated product is an inference here; the prompt should have it
  checked against Abrusci (2002) and tested against a brute-force cyclic
  sequent prover, as step 6's differential test did.
- **The ascending-ids argument** (section 3) carries the checker, the
  view and the interactive state; it should be a stated lemma in the
  report, with a test over random cyclic proofs.
- **Symmetry breaking** is unsound under order, and
  `agrees_with_the_focused_engine` has no cyclic counterpart: the prompt
  should ask for a cyclic reference prover in `search/reference.rs`.
- **The id shift** from unsorted roots touches every snapshot; one
  commit should change the canonical form and regenerate them.
- **Units, Mix and affine** in cyclic mode: refuse, say why, list in
  `plan/later.md`.
- **The title of Moot and Piazza (2001)** should be corrected in
  `plan/36-lambek.md` and `17-assessment.md`.
- **Comparison targets.** No Lambek benchmark set was found; Grail's
  example grammars and coq-lambek's lexicons are the candidates. The
  prompt should ask for a generated family of cyclic sequents (random
  planar linkings read back, as `generate.rs` does for MLL).

## 5. Sources

- Lambek, J. 1958. The mathematics of sentence structure. American
  Mathematical Monthly 65, 154–170. Reprint record:
  https://www.jbe-platform.com/content/books/9789027278685-llsee.25.12lam
- Yetter, D. N. 1990. Quantales and (noncommutative) linear logic.
  Journal of Symbolic Logic 55, 41–64. https://doi.org/10.2307/2274953
- Abrusci, V. M. 1991. Phase semantics and sequent calculus for pure
  noncommutative classical linear propositional logic. Journal of
  Symbolic Logic 56, 1403–1451. https://doi.org/10.2307/2275485
- Abrusci, V. M. 2002. Classical conservative extensions of Lambek
  calculus. Studia Logica 71, 277–314.
  https://doi.org/10.1023/a:1020560613199
- Roorda, D. 1991. Resource logics: proof-theoretical investigations.
  PhD thesis, Universiteit van Amsterdam. Announcement:
  https://seas.upenn.edu/~sweirich/types/archive/1991/msg00099.html
- Roorda, D. 1992. Proof nets for Lambek calculus. Journal of Logic and
  Computation 2(2), 211–231. https://doi.org/10.1093/logcom/2.2.211
- Lamarche, F. and Retoré, C. 1996. Proof nets for the Lambek
  calculus: an overview. Proceedings of the Roma workshop "Proofs and
  Linguistic Categories".
  https://www.lirmm.fr/~retore/ARTICLES/LaReRoma96.pdf
- Nagayama, M. and Okada, M. 2001. A new correctness criterion for the
  proof nets of non-commutative multiplicative linear logics. Journal
  of Symbolic Logic 66, 1524–1542. https://doi.org/10.2307/2694960
- Melliès, P.-A. 2004. A topological correctness criterion for
  multiplicative non-commutative logic. In Linear Logic in Computer
  Science, Cambridge University Press, 283–322.
  https://doi.org/10.1017/cbo9780511550850.009 (preprint
  https://hal.archives-ouvertes.fr/hal-00154204)
- Moot, R. 2002. Proof nets for linguistic analysis. PhD thesis,
  Utrecht University. https://www.labri.fr/perso/moot/moot02proofnets.pdf
- Moot, R. and Puite, Q. 2002. Proof nets for the multimodal Lambek
  calculus. Studia Logica 71, 415–442.
  https://doi.org/10.1023/a:1020525032763
- Morrill, G. and Fadda, M. 2008. Proof nets for basic discontinuous
  Lambek calculus. Journal of Logic and Computation 18(2), 239–256.
  https://doi.org/10.1093/logcom/exm089
- Fadda, M. Geometry of grammar: exercises in Lambek style. PhD thesis,
  UPC. https://www.cs.upc.edu/~morrill/papers/MarioThesis/all.pdf
- Moot, R. and Retoré, C. 2012. The Logic of Categorial Grammars.
  LNCS 6850, Springer. https://doi.org/10.1007/978-3-642-31555-8
- Moot, R. 2008. Graph algorithms for improving type-logical proof
  search. arXiv:0805.2303 (Categorial grammars workshop, Montpellier
  2004). https://arxiv.org/abs/0805.2303
- Pentus, M. 1993. Lambek grammars are context free. LICS 1993,
  429–433. https://lics.siglog.org/archive/1993/Pentus-Lambekgrammarsareco.html
- Pentus, M. 1997. Product-free Lambek calculus and context-free
  grammars. Journal of Symbolic Logic.
  https://www.cambridge.org/core/journals/journal-of-symbolic-logic/article/abs/productfree-lambek-calculus-and-contextfree-grammars/6E670F54D03706004C26ED975464A56A
- Pentus, M. 2006. Lambek calculus is NP-complete. Theoretical Computer
  Science 357, 186–201. https://doi.org/10.1016/j.tcs.2006.03.018
- Pentus, M. 2010. A polynomial-time algorithm for Lambek grammars of
  bounded order. Linguistic Analysis 36, 441–472.
  https://www.linguisticanalysis.com/?p=1219
- Savateev, Y. 2010. Unidirectional Lambek grammars in polynomial time.
  Theory of Computing Systems 46, 662–672.
  https://doi.org/10.1007/s00224-009-9208-4
- Savateev, Y. 2012. Product-free Lambek calculus is NP-complete.
  Annals of Pure and Applied Logic 163(7), 775–788.
  https://doi.org/10.1016/j.apal.2011.09.017 (LFCS 2009 version
  https://doi.org/10.1007/978-3-540-92687-0_26; preprint
  https://home.inf.unibe.ch/ltg/publications/2012/sav12.pdf); Savateev
  2011. An application of proof-nets to the study of fragments of the
  Lambek calculus. Izvestiya: Mathematics 75(3), 631–663,
  https://mathnet.ru/eng/im4118
- Fowler, T. A. D. 2008. Efficient parsing with the product-free Lambek
  calculus. COLING 2008, 217–224. https://doi.org/10.3115/1599081.1599109
- Fowler, T. A. D. 2010. A polynomial time algorithm for parsing with
  the bounded order Lambek calculus. The Mathematics of Language, LNCS,
  36–43. https://doi.org/10.1007/978-3-642-14322-9_4
- Kanovich, M., Kuznetsov, S., Morrill, G. and Scedrov, A. 2017. A
  polynomial-time algorithm for the Lambek calculus with brackets of
  bounded order. FSCD 2017, LIPIcs 84, article 22.
  https://arxiv.org/abs/1705.00694
- Kanovich, M., Kuznetsov, S. and Scedrov, A. 2019. The complexity of
  multiplicative-additive Lambek calculus: 25 years later. WoLLIC 2019,
  LNCS 11541, 356–372. https://doi.org/10.1007/978-3-662-59533-6_22
- Kuznetsov, S. 2017. Eliminating the unit constant in the Lambek
  calculus with brackets. arXiv:1711.06361. https://arxiv.org/abs/1711.06361
- Zhao, J. and Penn, G. 2025. An efficient parser for bounded-order
  product-free Lambek categorial grammar via term graph. IWPT 2025,
  1–10. https://aclanthology.org/2025.iwpt-1.1
- Hepple, M. 1990. Normal form theorem proving for the Lambek calculus.
  COLING 1990. https://aclanthology.org/C90-2030/
- Moot, R. and Piazza, M. 2001. Linguistic applications of first order
  intuitionistic linear logic. Journal of Logic, Language and
  Information 10(2), 211–232. https://philarchive.org/rec/MOOLAO
- Moot, R. 2013. Extended Lambek calculi and first-order linear logic.
  arXiv:1305.6238. https://arxiv.org/abs/1305.6238
- Morrill, G. 2019. Parsing/theorem-proving for logical grammar
  CatLog3. Journal of Logic, Language and Information 28(2), 183–216;
  CatLog page https://www.cs.upc.edu/~morrill/
- Grail: https://github.com/RichardMoot/Grail and
  https://www.labri.fr/perso/moot/grail3.html; Grail Light:
  https://github.com/RichardMoot/GrailLight; LinearOne:
  https://github.com/RichardMoot/LinearOne (metadata from the GitHub
  API, read 2026-10-08)
- coq-lambek: https://rocq-prover.org/p/coq-lambek and
  https://github.com/coq-contribs/lambek
- Yalla: https://github.com/olaure01/yalla, README of `yalla/`
- unicode_fol_kit:
  https://unicode-fol-kit.readthedocs.io/en/latest/_autosummary/unicode_fol_kit.lambek_prove.html
- proof-search-specifications.md, section "MLL variants: units, Mix,
  cyclic MLL and Lambek" (this repository)
- The repository snapshot of 2026-10-08: `core/src/sequents/mod.rs`,
  `term.rs`, `fmt.rs`; `core/src/parse/mod.rs`; `core/src/fragment.rs`;
  `core/src/occurrences/mod.rs`, `reading.rs`; `core/src/proofs/mod.rs`,
  `check.rs`, `derivation.rs`, `interactive.rs`, `style.rs`;
  `core/src/nets/mod.rs`, `sequentialize.rs`; `core/src/search/mod.rs`,
  `net.rs`; `core/src/serialize/search.rs`, `nets.rs`;
  `core/src/export/notation.rs`, `rocq.rs`, `svg/net.rs`;
  `cli/src/argument_parsing.rs`; `.claude/rules/core-*.md`;
  `plan/36-lambek.md`, `plan/later.md`, `plan/reports/17-assessment.md`

Sources checked 2026-10-08: 36 checked, 2 corrected, 0 removed, 0 claims marked.
