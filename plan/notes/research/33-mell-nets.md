# Research note for step 33: MELL proof nets with exponential boxes

Written 2026-10-08 from the snapshot of the repository and the sources
of section 5. Claims about the code name the file; claims about the
literature name a source `[Sn]`; everything else is marked
"(inference)". The web search budget ran out before Lafont's
interaction nets, Laurent's polarized nets and BOHM could be checked,
so they are not cited.

## 1. The problem and the state of the art

**Definitions.** MELL is MLL plus `!` and `?`, with the rules
dereliction `?d`, contraction `?c`, weakening `?w` and promotion `!`
(`⊢ ?Γ, A` gives `⊢ ?Γ, !A`) [S4 §2.1]. In a proof structure the
promotion is a *box*: a sub-structure with one principal door `!A` and
auxiliary doors `?B`, drawn as `?p` nodes, whose content is itself a
structure; boxes nest and never overlap [S4 §2.2; S1]. The `?`-tree of
a `?` edge is the tree of `?d`, `?w`, `?c` and `?p` nodes above it
[S4 §2.2]. The correctness criterion is Danos–Regnier's acyclicity and
connectedness of every switching graph [S2], taken at each depth with
every box contracted to one node whose conclusions are its doors, and
`?c` switched like `⅋` [S4 §2.3; `plan/later.md`]. Laurent's notes work
with Mix and the nullary Mix rule, so that only acyclicity is required
and weakening costs nothing [S4 §1.6, §2.1]. Girard's generalized
`?`-link has `n ≥ 0` premises, all occurrences of one discharged
formula `[A]`, and conclusion `?A`; `n = 0` is weakening; and every
`?`-link carries a *default jump* to another formula, which a switching
may select in place of a premise, before "the usual connected-acyclic
condition" [S3 §3, p. 22–23]. The multiplicative `⊥` is "treated like
0-ary `?`-links, i.e. they must be given a default jump" [S3 §A.2].
Laurent's `?` node with `n ≥ 0` premises of label `A` and `p` nodes
chained above it is the same idea without jumps, in the Mix setting;
its purpose is to "make more canonical the representation of
`?`-trees", and correctness and reduction carry over by translating
the `?` node to a left comb of `?c` [S4 §2.5, Lemma 2.9].

**Why weakening and units are delicate.** Without jumps, a structure
for a formula built from `1` and `⊥` alone is the formula, so a
criterion without jumps would decide constant-only MLL, which is
NP-complete [S3 §A.2; S12]; Girard notes "acyclic with `n + 1`
connected components, `n` the number of `⊥`-links" as a *necessary*
condition only [S3, footnote 5]. Jumps make correctness checkable but
not canonical: Girard declares them "not part of the proof-net, but
rather of some control structure", and nets become equivalence
classes [S3 §A.2]. That is unavoidable: proof equivalence in MLL with
units is PSPACE-complete, and the hardness "extends to equivalence of
normal forms in MELL without units, where the weakening rule for the
exponentials induces a similar rewiring problem" [S9]. Guerrini and
Masini give a formulation of "full (weakening and constants included)"
MELL nets with a confluent, strongly normalizing parsing rewriting
system whose success is a criterion equivalent to Danos–Regnier's
[S5, report abstract]; the journal text could not be fetched, so how
their formulation attaches a weakening is for the step to read.

**Algorithms.** Sequentialization of MLL nets goes through a
splitting `⊗` or a terminal `⅋`; Laurent proves it for structures with
`hyp` nodes, "at least one conclusion and no premise", which is
exactly what a contracted box is at the outer depth [S4 §1.4.3,
Theorem 1.1]. Correctness of MLL nets is decidable in linear time
[S6; S8], and correctness of MLL and MELL proof structures is
NL-complete [S7]. Cut elimination on MELL nets with boxes is confluent
and strongly normalizing [S4 Theorems 2.1, 2.2]; it is step 34's
subject.

**Box-free alternatives and their limits.** Gonthier, Abadi and Lévy
replace boxes by the graphs of optimal λ-reduction, because boxes
"impose synchronization, limit sharing, and impede a completely local
view of computation" [S18]; Guerrini, Martini and Masini give sharing
nets a correctness theory of their own [S19]. Jumps can replace boxes
by "cones" that may partially overlap [S21] or by jumping boxes [S20].
Differential interaction nets have no boxes, but promotion is then
approximated by the infinite Taylor expansion [S22]. Acclavio's
exponentially handsome nets cover MELL with units through cographs,
cut-free [S24]. All of these serve reduction or semantics; none gives
what a course draws (inference). Boxes stay, as the prompt fixes.

**Search, decidability and complexity by fragment.** MLL is NP-complete
[S11; S10], also with constants only [S12]; MALL is PSPACE-complete and
full LL undecidable [S10]. MELL's decidability is open: Bimbó's proof
is wrong [S16]; MELL provability is equivalent to reachability for
branching VASS [S14] and Tower-hard already in the affine case [S15].
The `!`-Horn fragment is Petri-net reachability [S13], which is
Ackermann-complete [S17]. In MLL a cut-free proof is its linking, so
net search is a search over linkings (`search/net.rs`); in MELL the
structure is not fixed by the sequent, since the copies and the boxes
they cross are the proof's choices, so a net search would guess them
under a bound as the focused engine's copy bound does, and gains
nothing over the dyadic sequent (inference). No implementation of
proof search with MELL nets was found; Grail searches proof nets for
categorial grammars [S30].

**Implementations found.** Yalla, a deep embedding of linear logic in
Rocq, LGPL-3.0, no release listed [S25]; Di Guardia's `proofnet_mll`,
Coq/Rocq, LGPL-3.0, MLL nets with a Yeo-based sequentialization and cut
elimination, 264 commits, no release [S26]; Click & Collect, an
OCaml-backed sequent prover for full LL with exports, LGPL-2.1, no
nets [S27]; Matsuoka's Proof Net Calculator, Scala, MLL without units,
checks several criteria and draws, version 0.0.12 of 2023-09-21,
licence not stated [S28]; Proofnetify, LaTeX with Perl, OCaml and
MetaPost, MLL only, licence and date not stated [S29]; Grail 3.2.0 of
2015-12-01, Prolog, LGPL-2.1 [S30]; HVM, Rust, MIT, an interaction-net
runtime with "dup nodes", no release listed on the page [S31]. None
draws or checks MELL nets with boxes.

## 2. What the step needs

**Vertices are instances, not occurrences.** A `Copy` repeats an
occurrence and its whole subtree in one branch (`Node::Copy`,
`proofs/mod.rs`), so an MELL net has one vertex per *instance* of an
occurrence, allocated per net. Cost: as many vertices as the proof
has rule instances, plus the doors (inference).

**The `?` node (choice).** Options: `?d`/`?c`/`?w` as nodes [S4 §2.2],
or one generalized node [S3; S4 §2.5]. Recommendation: one node per
`?B` instance with `n ≥ 0` premises. The dyadic term has no
contraction tree: `Quest` moves `B` into `Θ` and each `Copy` is a
dereliction (`proofs/check.rs`, `Quest` and `Copy`), so the generalized
node is what the term determines, desequentialization is canonical for
`?c`/`?d`/`?p` permutations by construction, and `Weaken(?B)` and a
`Quest` never copied give the same node (inference). A switching keeps
one premise of a node with `n ≥ 2` premises, like `⅋`; `n = 1` is
unswitched.

**Doors (choice).** Laurent chains one `p` node per leaf [S4 §2.5]; the
alternative is one door per (`?`-instance, box) pair, contraction
pushed into the box. Recommendation: one door per pair, because `Θ`
carries `B` once through a promotion (`Bang` keeps `Θ`, `check.rs`),
so copies inside a box descend from one entry; a door exists only
where a copy is made inside, which makes unused `Θ` entries leave no
trace (inference). The door is a vertex of the box's content graph
(switched when it has several premises) and an edge from the box node
to the `?` instance in the outer graph.

**Weakening and `⊥` (choice).** With binary Mix only, no rule
concludes `⊢` (`nets/mod.rs`, `NetError::Empty`), so Laurent's
acyclicity-only route [S4] is not available even in Mix mode, and a
jump-free criterion is NP-hard with units [S3; S12]. Recommendation:
every zero-premise `?` node and every `⊥` carries a jump to a vertex
at its depth, the jump is an unswitched edge of the criterion [S3
§3], equality of nets ignores jumps, and `from_proof` picks the jump
canonically from the term, the principal instance of the premise rule
(inference). The alternative, nullary Mix in Mix mode, changes the
calculus, the checker and the Rocq export; it is for the author.

**The criterion.** Reuse `graph.rs`'s Yeo deletion [`core-nets.md`]
on one graph that is the disjoint union of the depth graphs: tree
edges except the one from `!A` to its content, links, `?` premise
edges, door edges, jump edges, and a box node joined to its `!A`
instance and its doors. The colouring extends: the premise edges of a
`⅋` or of a `?` node with several premises share one colour, every
other edge its own, so the deletion condition stays "every non-premise
edge is a bridge" (inference from the proof in `core-nets.md`).
Connectedness is the edge count per box content, as now. Cost: linear
per round, as today [`plan/reports/05-proof-nets.md`].

**Sequentialization.** Extend `sequentialize.rs`'s stages: a weakening
or `⊥` conclusion is removed first and its rule emitted below
(`Weaken`, `Bot`); `⅋` and `?` conclusions are opened (`Par`, `Quest`),
a copy premise becoming a conclusion whose `Copy` is emitted when its
sub-net is proved; a box alone is `Bang` then its content with the
doors already opened; a splitting `⊗` is a bridge of the plain graph
with box nodes and jump edges as vertices and edges (inference; the
hyp-node form of the theorem is [S4 §1.4.3]). The step's prompt asks
for the criterion to be compared with switching enumeration; add the
sequent calculus itself as the oracle for weakening and `⊥` on small
structures, since the jump criterion's exactness for them is the
claim most likely to be wrong (inference).

**Drawing.** A box is the rectangle around its content's layout,
placed as one unit in the outer layout; doors are marks on the
bottom edge; jumps are dashed arrows, hidden by default (inference,
D12).

## 3. What linlog's library must offer

**Data model.** `Forest` (`occurrences/mod.rs`) has one `OccId` per
subformula occurrence, `kind`, `parent`, `left`/`right`, `depth`,
`literals(atom, sign)`. Missing: a vertex table over instances. The
step needs a `Vertex { occ: OccId, parent: Option<VertexId>, box:
Option<BoxId> }` table in the net, with doors and box nodes as
further vertices, and the MLL case the identity `vertex = occurrence`.
`OccSet` is per occurrence and does not fit instances. Step 34 will
duplicate boxes, so vertex ids must be the net's own, not
`(OccId, NodeId)` pairs (those serve only inside `from_proof`); step
38's quantifier nodes fit the same table (D17).

**Proof term and checker.** `Node` has `Bang`, `Quest`, `Copy`,
`Weaken`, `Bot`, `One`, `Mix` (`proofs/mod.rs:117`); the checker
enforces an empty linear zone under `Bang` and `Copy` only of a
formula under `?` (`check.rs:1242–1292`). Nothing is missing for
desequentialization: instances are `(OccId, origin Copy node)`, boxes
are `Bang` nodes, the content of a box is the premise sub-derivation
(one pass with parent pointers), links are `Ax` nodes. In the way:
`from_proof` (`nets/mod.rs`) reads only `Ax`, and `sequentialize`
emits only `Ax`, `Tensor`, `Par`, `Mix`.

**Nets.** `ProofStructure` is forest, `mix`, `partner: Box<[u32]>` by
occurrence, the link stack, `Graph` (CSR over occurrences, premise
edges fixed at construction, one axiom slot per literal) and
`Skeleton`. `ProofStructure::new` refuses every fragment outside
unit-free MLL (`Error::NetFragment`). Missing: variable-arity
premise slots (a `?` node), door, box and jump edges, a vertex table,
the per-content connectedness count, a tree edge that stops at a box
boundary. Either `ProofStructure` is generalized with MLL as the
special case, or a second type carries MELL and the MLL one stays as
the net engine's hot structure; the net engine relies on
`link_unchecked`, `same_component` and `is_acyclic` on partial
structures (`core-nets.md`), so the generalization must cost it
nothing, which `bench` must show. `NetError` needs variants for a
link across depths, a jump into another box, a door without a copy,
and disconnection per box. `Display` prints `~A[0] — A[2]` by
occurrence id; instances need a notation.

**Engine interface and dispatch.** `Decide` and `Answer` (`search/mod.rs`)
carry `net: Option<ProofStructure>`, set by the net engine only;
`Outcome::net` likewise. The CLI builds the net of any other engine's
proof with `from_proof` (`cli/src/prove.rs`, `net_into`). Search stays
with `DISPATCH`; nothing changes there, but the type of `Outcome::net`
follows the decision above.

**JSON.** `serialize/nets.rs` writes `{"sequent", "mix", "links"}` with
occurrence ids. MELL needs the vertex table (`[occ, parent, box]` per
vertex), `jumps`, and links over vertex ids; keep the MLL form
unchanged when there is no exponential, as D18 asks for JSON that
changes only where a step says so. The web client needs stable ids
for click targets: `o<n>` by vertex, `l<m>-<n>` as now
(`plan/reports/11-svg.md`).

**Options.** `svg::Style` (`export/svg/mod.rs:61`) has colours, gaps,
`node_radius`, `link_height`, `link_cap`, and no box fields. D15 asks
for box padding, stroke, corner radius, door radius, box and jump
colours, and whether jumps are drawn, all in `Style`. `svg::net` takes
a `limit` and estimates from the structure; the estimate must count
instances and boxes. The CLI's `nets_exist` (`prove.rs:949`) must admit
MELL and still refuse affine mode and additives.

## 4. Risks, open questions, what the prompt should add

- The jump criterion for weakening and `⊥` with binary Mix is the
  soundness risk; Girard states only a necessary condition without
  jumps [S3], and no source here proves the exact statement linlog
  needs. The prompt should require a brute-force comparison against
  the sequent calculus on every structure of a few nodes, with and
  without Mix, before the panel.
- Canonicity must be claimed only modulo jumps [S3; S9]. The report
  should say so, and the equality tests should ignore them.
- Open: one type or two for MLL and MELL nets; the net engine's
  timings decide (the target set does not cover it).
- Open: nullary Mix in Mix mode, which would simplify weakening [S4]
  but changes the calculus; the author decides.
- Open: whether to read Guerrini–Masini's exact formulation of
  weakening [S5] before choosing jumps; the step should obtain the
  paper.
- Open: how an instance is named in the text form and in the web
  client.
- The prompt should fix: vertices as instances, the generalized `?`
  node, one door per `?`-instance and box, jumps stored but ignored by
  equality, `Style` fields for boxes, the oracle tests, and a report
  section on where cut edges (step 34) and quantifier nodes (step 38)
  go in the vertex table.

## 5. Sources

- [S1] J.-Y. Girard, "Linear logic", Theoretical Computer Science 50
  (1987) 1–101. https://doi.org/10.1016/0304-3975(87)90045-4
- [S2] V. Danos, L. Regnier, "The structure of multiplicatives",
  Archive for Mathematical Logic 28 (1989) 181–203.
  https://doi.org/10.1007/BF01622878
- [S3] J.-Y. Girard, "Proof-nets: the parallel syntax for
  proof-theory" (1996), author's PDF.
  https://girard.perso.math.cnrs.fr/Proofnets.pdf
- [S4] O. Laurent, "An Introduction to Proof Nets", lecture notes,
  18 October 2013. https://perso.ens-lyon.fr/olivier.laurent/pn.pdf
- [S5] S. Guerrini, A. Masini, "Parsing MELL proof nets", Theoretical
  Computer Science 254 (2001) 317–335,
  https://doi.org/10.1016/S0304-3975(99)00299-6; report IRCS-96-37
  (1996), https://repository.upenn.edu/ircs_reports/115
- [S6] S. Guerrini, "A linear algorithm for MLL proof net correctness
  and sequentialization", Theoretical Computer Science 412 (2011),
  from p. 1958. https://portal.mardi4nfdi.de/entity/Q534705
- [S7] P. Jacobé de Naurois, V. Mogbil, "Correctness of linear logic
  proof structures is NL-complete", Theoretical Computer Science 412
  (2011) 1941–1957. https://doi.org/10.1016/j.tcs.2010.12.020
- [S8] A. S. Murawski, C.-H. L. Ong, "Fast verification of MLL proof
  nets via IMLL", ACM Transactions on Computational Logic 7 (2006)
  473–498. https://doi.org/10.1145/1149114.1149116
- [S9] W. Heijltjes, R. Houston, "Proof equivalence in MLL is
  PSPACE-complete", Logical Methods in Computer Science 12(1:2)
  (2016); LICS 2014. https://doi.org/10.2168/LMCS-12(1:2)2016
- [S10] P. Lincoln, J. Mitchell, A. Scedrov, N. Shankar, "Decision
  problems for propositional linear logic", Annals of Pure and
  Applied Logic 56 (1992) 239–311.
  https://doi.org/10.1016/0168-0072(92)90075-B
- [S11] M. I. Kanovich, "The complexity of Horn fragments of linear
  logic", Annals of Pure and Applied Logic 69 (1994) 195–241.
  https://doi.org/10.1016/0168-0072(94)90085-X
- [S12] P. Lincoln, T. Winkler, "Constant-only multiplicative linear
  logic is NP-complete", Theoretical Computer Science 135 (1994)
  155–169.
  https://ftp.math.utah.edu/pub/tex/bib/idx/tcs1990/135/1/155_169.html
- [S13] M. I. Kanovich, "Petri nets, Horn programs, linear logic and
  vector games", Annals of Pure and Applied Logic 75 (1995) 107–135.
  https://www2.informatik.uni-hamburg.de/TGI/pnbib/k/kanovich_m_i1.html
- [S14] P. de Groote, B. Guillaume, S. Salvati, "Vector addition tree
  automata", LICS 2004, 64–73.
  https://lics.siglog.org/archive/2004/deGrooteGuillaumeSa-VectorAdditionTreeA.html
- [S15] R. Lazić, S. Schmitz, "Nonelementary complexities for
  branching VASS, MELL, and extensions", ACM Transactions on
  Computational Logic 16(3) (2015), article 20,
  https://doi.org/10.1145/2733375; https://arxiv.org/abs/1401.6785
- [S16] L. Straßburger, "On the decision problem for MELL",
  Theoretical Computer Science 768 (2019) 91–98.
  https://doi.org/10.1016/j.tcs.2019.02.022
- [S17] W. Czerwiński, Ł. Orlikowski, "Reachability in vector addition
  systems is Ackermann-complete", FOCS 2021, 1229–1240.
  https://arxiv.org/abs/2104.13866
- [S18] G. Gonthier, M. Abadi, J.-J. Lévy, "Linear logic without
  boxes", LICS 1992, 223–234.
  https://doi.org/10.1109/LICS.1992.185535
- [S19] S. Guerrini, S. Martini, A. Masini, "Coherence for sharing
  proof-nets", Theoretical Computer Science 294 (2003) 379–409.
  https://doi.org/10.1016/S0304-3975(01)00162-1
- [S20] B. Accattoli, S. Guerrini, "Jumping boxes", CSL 2009, LNCS
  5771, 55–70. https://doi.org/10.1007/978-3-642-04027-6_7
- [S21] P. Di Giamberardino, "Jump from parallel to sequential proofs:
  exponentials", Mathematical Structures in Computer Science, listed
  as in press by the Cagliari repository.
  https://iris.unica.it/handle/11584/74873
- [S22] T. Ehrhard, L. Regnier, "Differential interaction nets",
  Theoretical Computer Science 364 (2006) 166–195.
  https://doi.org/10.1016/j.tcs.2006.08.003
- [S23] L. Tortora de Falco, "Obsessional experiments for linear logic
  proof-nets", Mathematical Structures in Computer Science 13 (2003)
  799–855. https://doi.org/10.1017/S0960129503003967
- [S24] M. Acclavio, "Exponentially handsome proof nets and their
  normalization", EPTCS 353 (2021) 1–25.
  https://arxiv.org/abs/2112.14962
- [S25] O. Laurent, Yalla: Yet Another deep embedding of Linear Logic
  in Rocq, LGPL-3.0. https://github.com/olaure01/yalla
- [S26] R. Di Guardia, proofnet_mll, Coq/Rocq, LGPL-3.0.
  https://github.com/RemiDiG/proofnet_mll
- [S27] E. Callies, O. Laurent, Click & Collect, interactive linear
  logic prover, OCaml backend, LGPL-2.1.
  https://click-and-collect.linear-logic.org/ ;
  https://github.com/etiennecallies/click-and-collect ; TLLA 2021
  paper https://hal-lirmm.ccsd.cnrs.fr/lirmm-03271501
- [S28] S. Matsuoka, Proof Net Calculator, Scala, version 0.0.12
  (2023-09-21). https://staff.aist.go.jp/s-matsuoka/PNCalculator/index.html
- [S29] R. Duncan, Proofnetify: Draw Proof-nets in LaTeX.
  http://www.cs.ox.ac.uk/ross.duncan/proofnets/
- [S30] R. Moot, Grail 3.2.0 (2015-12-01), Prolog, LGPL-2.1,
  https://github.com/RichardMoot/Grail ; R. Moot, "The Grail theorem
  prover: Type theory for syntax and semantics" (2016),
  https://arxiv.org/abs/1602.00812
- [S31] HigherOrderCO, HVM, Rust, MIT. https://github.com/HigherOrderCO/HVM
Sources checked 2026-10-08: 31 checked, 1 corrected, 0 removed, 0 claims marked.
