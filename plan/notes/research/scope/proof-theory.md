# Scope note: proof-theoretic tooling beyond the saved plans

Written 2026-10-09 from the snapshot of the repository (`plan/later.md`,
`plan/notes/research/README.md` and the notes it summarises) and the
sources of section 4, cited `[Sn]`; repository files are cited by path.
A claim without a bracket is marked "(inference)". The question is how
linlog's scope could grow beside or beyond steps 29 to 38; each
candidate says whether it extends a saved plan, sits beside one, or is
new.

## 1. The field as it stands

**Proof equivalence and canonical forms.** Cut-free sequent proofs that
differ only in the order of commuting rules are taken as one proof; the
question is which object represents the class. Focusing restricts the
order [S1], and Chaudhuri, Miller and Saurin show that maximally
multi-focused proofs are canonical representatives of permutation
classes in MALL, in bijection with proof nets on unit-free MLL [S2]. For
unit-free MLL the net is the canonical form, so equivalence is
"translate both and compare" [S3]. With the multiplicative units
equivalence is PSPACE-complete, which rules out any low-complexity net,
and the hardness extends to normal forms of MELL without units [S3]; for
unit-free MALL it is Logspace-complete via binary decision diagrams
[S4]. Hughes and van Glabbeek's MALL nets are sets of linkings, one per
`&`-resolution (one branch deleted above each `&`), canonical but
exponential [S5, S6]; monomial nets are small but not canonical;
conflict nets (2016) are linear in the sequent proof and *locally*
canonical, invariant under every rule permutation but the non-local one
between `⊗` and `&` [S7]. For first-order MLL, Hughes's unification nets
drop the existential witnesses and are canonical where Girard's nets
are not: `∀x Px ⊢ ∃x Px` has infinitely many Girard nets and one
unification net [S8]; first-order additive logic followed [S9].
Straßburger's lecture notes introduce the whole question [S10]. Hughes's
combinatorial proofs give classical propositional logic a canonical
graph-homomorphism proof object with polynomial-time correctness [S11];
the intuitionistic version is an IMLL net plus a skew fibration,
size-aware and locally canonical, and reads as a dag-like Hyland–Ong
strategy [S12]. Semantically, the relational model is injective for
MELL nets: equality in the model is cut-elimination equivalence [S13].

**Deep inference.** The calculus of structures lets rules act at any
depth inside a formula; Guglielmi's system BV extends MLL by a self-dual
non-commutative operator with a modular cut elimination [S14].
Straßburger's thesis builds linear logic and its fragments in it, with
a system in which every rule is local [S15]; NEL adds the exponentials
to BV [S16]. Tiu proved that BV needs deep inference: bounding the depth
of rules loses completeness [S17]; BV provability is NP-complete [S18].
Chaudhuri, Guenot and Straßburger transplant focusing to the calculus of
structures and relate it to focused sequent proofs both ways [S19]. Deep
inference has Frege strength and admits analytic proofs exponentially
shorter than analytic Gentzen proofs [S20]; atomic flows trace atoms
through a derivation and control normalisation [S21]. For search, a 2024
LPAR paper reports that BV needs shallow and deep rules together, that
prioritising deeper instances shrinks the contexts, and ships a
large MLL benchmark with and without Mix [S22]; the ESSLLI 2019
notes are the current introduction [S23]. Deep inference also produced
an interaction style: Chaudhuri's subformula linking (the user links two
subformulas and the system rewrites the goal; Profound is a linking
prover for first-order classical linear logic) [S24], and Actema's
drag-and-drop tactic, built on the same tools and now a Coq front end
[S25].

**Proof compression and minimal proofs.** Boolos's argument that proofs
with cut can be far shorter is the classic [S26]; the complexity of cut
elimination in LL (PTIME-complete for MLL, coNP-complete for MALL,
non-elementary with exponentials) is its linear form [S27]. Hetzl,
Leitsch and Weller invert cut elimination, computing a cut formula from
a compressed term set, with exponential compression possible [S28]; the
Skeptik line compresses resolution DAGs by rewriting [S29]. In
propositional deep inference, normalisation is quasipolynomial via
atomic flows [S21].

**Geometry of interaction and the execution of nets.** Girard's
execution formula encodes cut elimination as an operator invariant
[S30]; Danos and Regnier read it on paths of the net [S31]; Mackie's
machine evaluates by following paths without rewriting [S32]; Laurent's
token machine covers full linear logic, additives included [S33];
Muroya and Ghica's dynamic GoI machine mixes token passing with graph
rewriting and is sound and complete for call-by-need and call-by-value
[S34]. Lafont's interaction nets make cut an active pair and reduction
local; HVM is a current runtime on interaction combinators [S35].

**Games and coherence spaces.** Coherence spaces are Girard's original
semantics [S36]. Blass's Lorenzen-style games validate weakening and are
complete for the additive fragment only [S37]; Abramsky and Jagadeesan
prove full completeness for MLL with Mix, every winning strategy being a
unique cut-free net [S38]; Abramsky and Melliès's concurrent games reach
MALL [S39]. Fermüller translates sequent calculi for intuitionistic
linear logic into Lorenzen-style dialogue games read as information
extraction [S40]; Alama and Uckelman's web tool plays Lorenzen games and
computes winning strategies for several rule sets, none linear [S41].
Ludics rebuilds the rules from interaction [S42]; Terui gives it a term
syntax and an abstract machine [S43].

**Explaining unprovability.** `plan/notes/research/refutations.md`
covers certified refutations (counts, assignments, phase and Kripke
models, failure DAGs, Farkas vectors), not the human reading.
Neighbouring fields have it: SAT solvers return minimal unsatisfiable
subsets [S44]; bi-abduction in separation logic infers the missing part
of a state (the anti-frame) and the unused part (the frame) together,
the basis of Infer [S45]; teaching tools show the stuck leaf of a failed
attempt [S46].

## 2. Candidate extensions

**proof-theory-1. Proof equivalence and canonical proofs.** A function
`Proof::canonical(mode)` and a command `linlog equiv P Q`: for unit-free
MLL by the net (`nets::from_proof`, today's canonical object); for MALL
by the maximally multi-focused normal form of the term [S2] or by the
linking set of `&`-resolutions [S5, S6], cheap by Bagnol's result [S4];
with units refused or bounded, the problem being PSPACE-complete [S3].
*Value:* grading and self-checking in courses, research on proof
identity, and a regression oracle for the engines (two engines' proofs
equal up to permutation). *Cost:* a normaliser on terms (independent
rule instances permuted to a fixed order, the checker verifying the
result), a `Linkings` type beside `ProofStructure`; theory settled; one
session for MLL and MALL.
*Relation:* new; beside 35 (nets) and 34 (cut, the other half of
identity). *API room:* `Proof` equality is structural today; reserve a
documented "equal up to permutation" rather than overloading
`PartialEq`; keep the term a DAG with stable node order so a canonical
order is a renumbering (`Proof::new` renumbers already,
`core-proofs.md`).

**proof-theory-2. MALL nets as conflict nets or linking sets, for
display and equivalence.** `plan/later.md` dropped MALL nets as
"non-canonical or exponentially large"; conflict nets are neither
(linear size, locally canonical) [S7], and the linking-set view is what
candidates 1 and 9 need. *What:* `nets::mall` with vertices for `&` and
`⊕`, a set of linkings or a conflict relation, the criterion of [S7],
SVG with slices toggled. Search stays with the focused engine, as 33
decided for MELL. *Value:* teaching MALL, the one core fragment with no
drawing of its own, and the equivalence command. *Cost:* a second net
structure and its correctness check (the paper read in full),
sequentialization optional; a session; the risk is the `⊗`/`&`
non-locality the paper isolates. *Relation:* reverses a drop in
`later.md`; extends 35's one `Criterion`-like flag on
`ProofStructure::new` (README conflict 4). *API room:* make that flag an
enum open to `Mall { kind }`; `from_proof` strict on unhandled nodes
(already required) keeps MLL hot; SVG ids `o<n>`, `l<m>-<n>` need a
slice index.

**proof-theory-3. Unification nets for first-order MLL.** Extends 38.
Step 38's note chooses levels over Skolem terms and recovers witnesses
by unification; unification nets are the canonical object for exactly
that choice: no witness stored, cut elimination local and linear,
cut-free nets linear in the sequent, correctness quadratic [S8]. *What:*
the net engine's links carry a unifier; a first-order `ProofStructure`
is the linking plus the most general unifier; the drawing shows no
witness. *Value:* the first-order net engine 38 wants ("gains
unification on its links and loses nothing else") with a canonical
result and a known correctness algorithm. *Cost:* a term arena and
unifier in `nets`, the criterion's dependence on the unifier; half a
session once 38's data model exists. *API room (now):* the witness slot
of the planned `Exists(OccId, Witness, NodeId)` must allow "implicit" (a
metavariable the unifier resolves) beside an explicit term, or the net
cannot be read back without inventing witnesses (inference).

**proof-theory-4. Subformula linking in the interactive session.**
Extends 32 and 9. `Interactive::apply` takes a goal and a position: one
principal formula at the root. Chaudhuri's linking lets the user connect
two subformulas anywhere and derives the rule sequence that brings them
together [S24]; Actema does it by drag-and-drop [S25]; Click & coLLecT
already proves LL by clicking [S46]. *What:* `Interactive::link(goal,
occ_a, occ_b)` computing the focused rule chain on the paths to the two
occurrences and applying it, or refusing with the first blocked rule.
*Value:* the web front end's main gesture; fewer clicks on wide
sequents; a view of "why these two must meet". *Cost:* the chain is a
bounded focused search on two paths (inference); linear splits make it
non-unique (where does the rest of a `⊗` go?), so the result is a menu
or a choice rule; small. *API room:* positions as `OccId` of subformula
occurrences in the SVG (`later.md` asks for a `<g>` per formula; make it
per occurrence), and an `apply` payload holding several rule instances
under one undo.

**proof-theory-5. A calculus-of-structures view, and BV.** (a) A
translation of the proof term into a calculus-of-structures derivation
(Straßburger's thesis has the systems and translations for LL and its
fragments [S15]) as an export beside 22's: a chain of formulas rewritten
inside, side by side with the tree, which many readers find easier
(inference). (b) BV as a mode: a self-dual non-commutative `◁` beside
`⊗` and `⅋` [S14], an engine that needs deep rules [S17] and the
nondeterminism reduction of [S18, S22]. *Value:* (a) teaching and the
deep-inference community; (b) sequential composition of resources, and
BV's MLL benchmark [S22] is a corpus 29 and 35 could run today. *Cost:*
(a) a printer and the CoS rule tables, one session; (b) a new data model
(derivations of formulas, not of occurrence sequents: D5's forest does
not survive rewriting inside formulas), a new engine, NP-complete search
without counts; two or more sessions, research-grade. *Relation:* (a)
new; (b) new, beside 36 (Lambek is ordered; BV's operator is self-dual
and sits between `⊗` and `⅋`, a different logic [S14]). *API room:*
nothing in forest or term should assume a derivation's lines are
occurrence sets of one sequent; keep the export entry over `&Proof` with
a target enum, as `rocq::write_proof` is planned.

**proof-theory-6. Proof compression and minimal proofs.** Beside 34 and
22. The proof is already a DAG (`later.md`: a read-back that unfolds
shared subproofs reached 32 million inferences), and 22's compact view
draws runs of structural rules as one line. Missing is a compression
with a meaning: (i) redundancy removal on the term (a `Weaken` of a
formula later used, an unused `Copy`, identical subproofs merged by
hash-consing), checked afterwards; (ii) shortest proof as a search
option (the Horn engine's breadth-first firing sequence is a follow-up
already; for the focused engine an iterative deepening on proof size);
(iii) cut introduction as compression once 34 exists, after [S28]:
factor a repeated subderivation as a lemma. *Value:* smaller
certificates (31's Rocq time grows with the proof), readable derivations
on the 65 641-step nets, "the smallest proof" for teaching. *Cost:* (i)
small; (ii) measured against the baselines, likely no default; (iii) a
session, research for the exponentials. *API room:* `Statistics` gains
proof-size fields by name; the JSON of `Proof` represents sharing
explicitly and is versioned, so a compressed proof round-trips.

**proof-theory-7. A token machine on nets.** Beside 34, on top of 33.
Once a net has cuts, GoI gives a second way to show what a cut does: a
token travels the paths of the net and the execution formula is the set
of paths that survive [S30, S31]; Mackie's and Laurent's machines run it
[S32, S33], Laurent's with additives. *What:* `nets::token`, a machine
over the coloured graph (`graph.rs` holds the edges in CSR form) with a
step iterator, so the CLI prints the path and the SVG animates it under
a `Stop`. *Value:* teaching GoI and the proofs-as-programs reading with
a picture; the field has machines but, as far as the search of
2026-10-09 found, no visualiser [S34]. *Cost:* small for MLL with cut
(finite paths); boxes and exponential tokens need 33; additives need
Laurent's slices [S33]; a session after 33 and 34. *API room:* stable
ids for edges, not only vertices and links, in SVG and JSON; the graph's
edge order a documented contract, as the forest's preorder is becoming.

**proof-theory-8. A dialogue mode: play against the prover.** Extends 32
and 9; new. A proof is a winning strategy for the Proponent [S37, S38];
Fermüller shows how an ILL sequent calculus becomes a Lorenzen game
[S40]. *What:* `Interactive` driven as a game: the user plays Opponent
(chooses the `&` branch, attacks a `⅋`) and the engine plays Proponent
from a proof it holds; or the user plays Proponent on an unprovable
sequent and the engine, as Opponent, wins every play. In a finite
determined game an Opponent strategy is a refutation experienced move by
move (inference; for MLL with Mix every P-strategy is a net [S38], so
the absence of one is unprovability there). *Value:* teaching, and an
explanation of unprovability without a countermodel. *Cost:* game rules
per connective are the focused rules read as moves (inference from
[S40]); exponentials are delicate, since Blass's games validate
weakening [S37]; MALL first under [S39]; a session plus a panel on the
game's soundness. *API room:* `Interactive` exposing the term as a
strategy (move = rule instance at a goal); `Refusal` as codes so a game
client words its own messages (32 asks this too).

**proof-theory-9. Explanations of unprovability for a reader.** Extends
step 21's `Refutation` and the refutations note, whose certificates
serve checkers. Three human views: (i) the *failing slice*: a
`&`-resolution whose multiplicative residue fails by the counts,
"whichever branch you choose here, this `⊗` cannot be balanced", the
human face of the linking-set view [S5, S6] (inference); (ii) *missing
and surplus resources*: the smallest `Δ` making `Γ, Δ` provable and what
is left over, bi-abduction's anti-frame and frame [S45] in a resource
logic (inference); in affine mode a minimal unprovable subset of the
hypotheses is sound, weakening keeping unprovability downward, and MUS
algorithms apply [S44] (inference); in linear mode only the count-based
form is; (iii) the *best attempt*: the deepest failed branch of the
focused search as a partial derivation with the stuck leaf marked, as
teaching tools do [S46], drawn by the exports that take open goals.
*Value:* the author's goal of 2026-10-03 (why a sequent is unprovable)
for a user who is not a checker. *Cost:* (i) small over the counts; (ii)
a bounded search of its own (D16); (iii) the engine keeps its deepest
attempt, which costs memory on wide searches; one session. *API room:*
new `Refutation` variants as named serde payload structs (`Slice`,
`Missing`, `Attempt`), the enum staying `#[non_exhaustive]`; an
`Attempt` reuses the interactive `Derivation` with open goals.

**proof-theory-10. Combinatorial proofs for the ordinary layer.** Beside
25; new. Step 25 reads LK and LJ derivations back from linear proofs.
Combinatorial proofs are the canonical, polynomially checkable object
for classical propositional logic [S11]; the intuitionistic ones split
into an IMLL net and a skew fibration, with polynomial full completeness
and local canonicity [S12]. *What:* `ordinary::combinatorial`: from the
linear proof of the image, the graph `C → G(φ)` (classical) or net plus
fibration (intuitionistic), drawn over the formula, with a correctness
check independent of the read-back. *Value:* a second, canonical
certificate for the layer, and a drawing of the linear core of a
classical proof (the translation's `!` become the fibration). *Cost:*
two translations and two correctness conditions, one session; theory
settled. *API room:* the layer's `Image` keeps its map from image
occurrences to source subformulas public; the SVG needs graphs over a
formula, not only trees and nets.

**proof-theory-11. Proofs as programs: a Curry–Howard export.** Beside
22 and 32. An ILL proof is a linear λ-term, a CLL proof a session-typed
process: Caires and Pfenning for dual intuitionistic linear logic [S47],
Wadler's CP for classical [S48]. *What:* an export target printing the
term as a typed term or process, with cut (34) as communication.
*Value:* courses teaching session types from linear logic, and synthesis
from linear types (the Granule benchmarks `later.md` dropped are of this
kind). *Cost:* a printer with a target enum; exponentials and Mix need
the papers' treatment (Mix is not in CP as published [S48]); half a
session. *API room:* the same export entry over `&Proof` as 31 plans;
`Proof` carries its `Mode`, as the README asks.

## 3. Considered and rejected

- **Coherence-space or relational semantics made visible** (a proof's
  interpretation with atoms as small sets, drawn as a web). Injectivity
  [S13] means the picture decides equivalence for MELL, but candidate 1
  does that syntactically and faster; the interpretation is exponential
  in the proof and no user asks for it (inference).
- **Ludics** [S42, S43]: no finite object a prover emits and no tooling
  to meet; research, not scope.
- **Atomic flows and quasipolynomial normalisation** [S21]: classical
  propositional deep inference, not linear logic; nothing for 25's layer
  either, whose proofs come back through LK and LJ.
- **An interaction-net or combinator evaluator** (HVM-style) [S35]: a
  runtime, not a prover feature; candidate 7 keeps the GoI reading where
  it shows something about a proof.
- **Hyland–Ong full completeness without Mix** as a basis for the
  dialogue mode: date and venue could not be verified on 2026-10-09;
  [S39] covers MALL and suffices.
- **Deep inference as the main engine**: the LPAR 2024 results are for
  BV and MLL formula classes [S22]; the focused engine's counts and memo
  have no deep counterpart, and 35 measures MLL engines already.
- **Skeptik-style resolution compression** [S29]: resolution DAGs, not
  sequent terms; what transfers is in candidate 6 (i).

## 4. Sources

- S1. J.-M. Andreoli, "Logic programming with focusing proofs in linear logic", *Journal of Logic and Computation* 2(3), 1992. https://doi.org/10.1093/logcom/2.3.297
- S2. K. Chaudhuri, D. Miller, A. Saurin, "Canonical sequent proofs via multi-focusing", IFIP TCS 2008. https://www.lix.polytechnique.fr/~dale/papers/tcs08trackb.pdf
- S3. W. Heijltjes, R. Houston, "No proof nets for MLL with units: proof equivalence in MLL is PSPACE-complete", CSL-LICS 2014; LMCS 12(1), 2016. https://lmcs.episciences.org/1625
- S4. M. Bagnol, "MALL proof equivalence is Logspace-complete, via binary decision diagrams", TLCA 2015 (corrected in LMCS 2017). https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.TLCA.2015.60
- S5. D. J. D. Hughes, R. J. van Glabbeek, "Proof nets for unit-free multiplicative-additive linear logic", *ACM TOCL* 6(4), 2005. https://doi.org/10.1145/1094622.1094629
- S6. R. J. van Glabbeek, D. J. D. Hughes, "MALL proof nets identify proofs modulo rule commutation", 2016. https://arxiv.org/abs/1609.04693
- S7. D. J. D. Hughes, W. Heijltjes, "Conflict nets: efficient locally canonical MALL proof nets", LICS 2016. https://doi.org/10.1145/2933575.2934559 (PDF: https://people.bath.ac.uk/wbh22/pdf/2016-hughes-heijltjes.pdf)
- S8. D. J. D. Hughes, "Unification nets: canonical proof net quantifiers", LICS 2018. https://arxiv.org/abs/1802.03224
- S9. W. Heijltjes, D. J. D. Hughes, L. Straßburger, "Proof nets for first-order additive linear logic", FSCD 2019. https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.FSCD.2019.22
- S10. L. Straßburger, "Proof nets and the identity of proofs", ESSLLI 2006 lecture notes, INRIA RR-6013. https://arxiv.org/abs/cs/0610123
- S11. D. J. D. Hughes, "Proofs without syntax", *Annals of Mathematics* 164(3), 2006. https://annals.math.princeton.edu/2006/164-3/p09
- S12. W. Heijltjes, D. J. D. Hughes, L. Straßburger, "Intuitionistic proofs without syntax", LICS 2019. https://people.bath.ac.uk/wbh22/pdf/2019-heijltjes-hughes-strassburger-LICS.pdf
- S13. D. de Carvalho, "The relational model is injective for multiplicative exponential linear logic", CSL 2016. https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.CSL.2016.41
- S14. A. Guglielmi, "A system of interaction and structure", *ACM TOCL* 8(1), 2007. https://arxiv.org/abs/cs/9910023
- S15. L. Straßburger, *Linear logic and noncommutativity in the calculus of structures*, PhD thesis, TU Dresden, 2003. https://www.lix.polytechnique.fr/~lutz/papers/dissvonlutz.pdf
- S16. A. Guglielmi, L. Straßburger, "Non-commutativity and MELL in the calculus of structures", CSL 2001. https://www.lix.polytechnique.fr/~lutz/papers/NoncMELLCoS.pdf
- S17. A. Tiu, "A system of interaction and structure II: the need for deep inference", LMCS 2(2), 2006. https://arxiv.org/abs/cs/0512036
- S18. O. Kahramanoğulları, "System BV is NP-complete", WoLLIC 2005, ENTCS 143. https://iccl.inf.tu-dresden.de/web/WVPub22
- S19. K. Chaudhuri, N. Guenot, L. Straßburger, "The focused calculus of structures", CSL 2011. https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.CSL.2011.159
- S20. P. Bruscoli, A. Guglielmi, "On the proof complexity of deep inference", *ACM TOCL* 10(2), 2009. https://arxiv.org/abs/0709.1201
- S21. A. Guglielmi, T. Gundersen, "Normalisation control in deep inference via atomic flows", LMCS 4(1), 2008, https://lmcs.episciences.org/1081 ; quasipolynomial normalisation: Bruscoli, Guglielmi, Gundersen, Parigot, https://arxiv.org/abs/0903.5392
- S22. O. Kahramanoğulları, "Deep inference in proof search: the need for shallow inference", LPAR 2024, EPiC 100. https://easychair.org/publications/paper/MgHR
- S23. A. Aler Tubella, L. Straßburger, *Introduction to deep inference*, ESSLLI 2019 lecture notes. https://www.lix.polytechnique.fr/~lutz/papers/ESSLLI19notes.pdf
- S24. K. Chaudhuri, "Subformula linking as an interaction method", ITP 2013, https://researchportal.ip-paris.fr/en/publications/subformula-linking-as-an-interaction-method/ ; Profound, https://github.com/chaudhuri/profound
- S25. P. Donato, P.-Y. Strub, B. Werner, "A drag-and-drop proof tactic", CPP 2022. https://arxiv.org/abs/2210.11820
- S26. G. Boolos, "Don't eliminate cut", *Journal of Philosophical Logic* 13, 1984. https://doi.org/10.1007/BF00247711
- S27. H. G. Mairson, K. Terui, "On the computational complexity of cut-elimination in linear logic", ICTCS 2003. https://www.kurims.kyoto-u.ac.jp/~terui/pub_bib.html
- S28. S. Hetzl, A. Leitsch, D. Weller, "Towards algorithmic cut-introduction", LPAR 2012. https://dmg.tuwien.ac.at/hetzl/research/towards.pdf
- S29. J. Boudou, B. Woltzenlogel Paleo, "Compression of propositional resolution proofs by lowering subproofs", TABLEAUX 2013 (Skeptik). https://www.springerprofessional.de/en/compression-of-propositional-resolution-proofs-by-lowering-subpr/4164474
- S30. J.-Y. Girard, "Geometry of interaction I: interpretation of system F", Logic Colloquium '88, Studies in Logic 127, 1989. Overview: https://en.wikipedia.org/wiki/Geometry_of_interaction
- S31. V. Danos, L. Regnier, "Proof-nets and the Hilbert space", *Advances in Linear Logic*, 1995. https://www.cambridge.org/core/books/abs/advances-in-linear-logic/proofnets-and-the-hilbert-space/4C074845DBBF5BA14529DE8FFFD8DDF3
- S32. I. Mackie, "The geometry of interaction machine", POPL 1995; thesis *The geometry of implementation*, Imperial College. https://spiral.imperial.ac.uk/handle/10044/1/46072
- S33. O. Laurent, "A token machine for full geometry of interaction", TLCA 2001, LNCS 2044. https://perso.ens-lyon.fr/olivier.laurent/goi.bib
- S34. K. Muroya, D. R. Ghica, "The dynamic geometry of interaction machine: a token-guided graph rewriter", LMCS 15(4), 2019. https://lmcs.episciences.org/5882
- S35. Y. Lafont, "Interaction nets", POPL 1990, https://doi.org/10.1145/96709.96718 ; HigherOrderCO, HVM, https://github.com/HigherOrderCO/HVM
- S36. J.-Y. Girard, "Linear logic", *TCS* 50(1), 1987. https://doi.org/10.1016/0304-3975(87)90045-4
- S37. A. Blass, "A game semantics for linear logic", *APAL* 56, 1992. https://deepblue.lib.umich.edu/handle/2027.42/30097
- S38. S. Abramsky, R. Jagadeesan, "Games and full completeness for multiplicative linear logic", *JSL* 59(2), 1994. https://arxiv.org/abs/1311.6057
- S39. S. Abramsky, P.-A. Melliès, "Concurrent games and full completeness", LICS 1999. https://www.irif.fr/~mellies/papers/AbramskyMellies99.pdf
- S40. C. G. Fermüller, "Connecting sequent calculi with Lorenzen-style dialogue games", in *Paul Lorenzen: Mathematician and Logician*, 2021. https://doi.org/10.1007/978-3-030-65824-3_8
- S41. J. Alama, S. L. Uckelman, "Playing Lorenzen dialogue games on the web", LPAR-17 short papers, 2013. https://easychair.org/publications/paper/8r
- S42. J.-Y. Girard, "Locus solum: from the rules of logic to the logic of rules", *MSCS* 11(3), 2001. https://resolve.cambridge.org/core/services/aop-cambridge-core/content/view/F93205D6292F744EB1C1FEABC4E2DFF2/S0960129501003371a.pdf/preface-to-locus-solum.pdf
- S43. K. Terui, "Computational ludics", *TCS* 412(20), 2011. https://doi.org/10.1016/j.tcs.2010.12.026
- S44. M. H. Liffiton, K. A. Sakallah, "Algorithms for computing minimal unsatisfiable subsets of constraints", *JAR* 40(1), 2008. https://sun.iwu.edu/~mliffito/publications/jar_liffiton_CAMUS.pdf
- S45. C. Calcagno, D. Distefano, P. W. O'Hearn, H. Yang, "Compositional shape analysis by means of bi-abduction", POPL 2009; *JACM* 58(6), 2011. https://www.cs.ox.ac.uk/publications/publication4972-abstract.html
- S46. Teaching tools: J. Breitner, "Visual theorem proving with the Incredible Proof Machine", ITP 2016, https://www.joachim-breitner.de/publications/Incredible_ITP2016_preprint.pdf ; Logitext, http://logitext.mit.edu ; A. Ehle, N. Hundeshagen, M. Lange, "The Sequent Calculus Trainer with automated reasoning", 2018, https://arxiv.org/abs/1803.01467 ; E. Callies, O. Laurent, Click & coLLecT, https://click-and-collect.linear-logic.org/
- S47. L. Caires, F. Pfenning, "Session types as intuitionistic linear propositions", CONCUR 2010, LNCS 6269. https://www.springerprofessional.de/session-types-as-intuitionistic-linear-propositions/3352308
- S48. P. Wadler, "Propositions as sessions", ICFP 2012; *JFP* 24(2–3), 2014. https://www.cambridge.org/core/journals/journal-of-functional-programming/article/propositions-as-sessions/0985539E5D607AC00FB00FF900BA1C86

Sources checked 2026-10-09 by web search. One candidate source (a CSL
2020 paper on "exponentially handsome" nets) was dropped because its
authors and venue could not be confirmed; Hyland and Ong's MLL result
is mentioned without a citation for the same reason.

Sources checked 2026-10-09: 48 checked, 1 corrected, 0 removed, 0 claims marked.
