# Research note for step 35: engines for MLL and IMLL

Written 2026-10-08 from the repository as it stands and from the sources
of section 5. A claim without a bracket is read off the repository file
named beside it; "(inference)" marks my own conclusion.

## 1. The problem and the state of the art

**Definitions.** A proof structure of unit-free MLL is the formula trees
of a one-sided sequent plus axiom links pairing dual literal occurrences;
a switching chooses one premise edge of every `⅋`, and the structure is a
proof net (Danos–Regnier criterion) when every switching is acyclic and,
without Mix, connected [Danos–Regnier 1989]. A cut-free MLL proof is
determined by its linking, so provability is "some complete linking is a
net", and the choice points of a net-based search are the pairings only
(`core/src/search/net.rs`, module docs; [spec] § MLL).

**Complexity.** MLL provability is NP-complete: membership because a
cut-free proof has one rule per connective occurrence [LMSS 1992],
hardness already for the Horn fragment over atoms [Kanovich 1992], and it
survives two literals or constants only, by 3-Partition
[Lincoln–Winkler 1994]. Correctness of a complete structure is much
cheaper: linear time by Danos's contractibility reformulated as a
union-find [Guerrini 2011], linear time by a dominator tree after a
translation of the structure into an essential net [Murawski–Ong 2000;
Murawski–Ong 2006], NL-complete [de Naurois–Mogbil 2011], and with Mix
equivalent to the uniqueness of a perfect matching, which gives a
linear-time criterion and quasi-linear sequentialization [Nguyễn 2020].
The repository's test is the deletion procedure of Yeo's theorem on
edge-coloured graphs [Yeo 1997], exact on partial structures, costing one
search per round (`.claude/rules/core-nets.md`); the same theorem is now
the basis of a sequentialization proof [Di Guardia et al. 2025]. Other
criteria that a search could use incrementally: the dependency graph of
[Bagnol–Doumane–Saurin 2015], which unifies contractibility and the
de Naurois–Mogbil criterion, and a three-rule tree rewriting
[Matsuoka 2019].

**Essential nets.** Lamarche's essential nets are the proof nets of
IMLL: a polarized, directed structure over the two-sided sequent, with
a theorem that a sequent is provable in IMLL iff its essential net is
correct [Lamarche 1994/2008, as stated in Moot 2004]. In de Groote's
orientation, which Moot adopts, correctness is: (1) the directed graph is
acyclic, (2) every path from the negative premise of a positive
implication link passes through that link's conclusion, (3) every path
from the inputs reaches the output [Moot 2004, Definition 6;
de Groote 1999]. Murawski and Ong decide (2) in linear time with a
dominator tree and read a sequentialization off it, and reduce classical
correctness to the essential-net problem in linear time
[Murawski–Ong 2000]. Lamarche's polarization is what the repository's
`Reading` computes (`core/src/occurrences/reading.rs`; `plan/README.md`
D1).

**What makes instances easy.** With every atom once per sign the linking
is forced and the correctness check decides in linear time ([spec] § MLL;
`distinct_atoms_need_no_backtracking` in `net.rs`). Hardness lives in
repeated literals: the number of complete linkings is the product over
atoms of the factorial of the multiplicity ([spec], table), and the Horn
encodings of Partition and 3-Partition, whose equal literals under one
`⊗` or `⅋` are interchangeable, are where the net engine takes seconds
and the focused engine microseconds (`core-nets.md`, "Where it loses";
`bench/COMPARISON.md`). [Matsuoka 2017] gives direct Horn-MLL encodings
of 3D Matching and Partition and argues they make better solvers than
chained reductions; his Proof Net Calculator prunes the backtracking by
"dependency relations of ID-links" [PNC].

**Incremental criteria in search.** Moot keeps the transitive closure of
the partial essential net, precomputed by Floyd–Warshall in O(v³), after
which a link that would close a directed cycle is rejected in constant
time, and ranks links by weights [Moot 2004]. The spec's version keeps
the closure as a bit matrix, updated in O(n · W) word operations per
accepted link with an undo log, and recomputes dominators only at the end
([spec] § IMLL-Net). The repository does the classical analogue: two O(1)
rejections (tensor LCA, `⅋`-free skeleton) and the exact Yeo test every
link up to 200 occurrences, then every fourth (`core-nets.md`).

**SAT and CSP.** I found no published SAT, ILP or ASP encoding of MLL
provability or of linking search; the spec found none either and notes
that only the reverse direction exists ([spec] § MLL; [Matsuoka 2017]).
The closest constraint method is Harland and Pym's: Boolean expressions
on side formulas turn the context splits of multiplicative rules into
Boolean equations solved lazily or eagerly [Harland–Pym 2003].
(inference) Acyclicity under every switching is not a local constraint,
so I expect no gain from a CNF encoding over the dedicated test.

**Implementations.**

| tool | what | language | licence | last activity | source |
|---|---|---|---|---|---|
| Grail 3 | proof-net prover for multimodal type-logical grammars | Prolog (SWI) | LGPL-2.1 | release 3.2.0 on 2015-12-01; last push 2021-02-11 | [Grail] |
| LinearOne | prototype prover for first-order multiplicative intuitionistic LL, proof nets with unification, LaTeX output | Prolog (SWI) | LGPL-2.1 | last push 2026-06-11 | [LinearOne] |
| Proof Net Calculator | MLL proof-net search, normalization, correctness by DR, de Naurois–Mogbil and the author's linear criterion, NP-problem encodings | Scala | none stated | version 0.0.12, page updated 2023-09-21 | [PNC] |
| Click & coLLecT | interactive sequent-calculus tool with an automatic prover; the spec read its source as naive `⊗`-splitting over a bitmask with a 3 s timeout | OCaml | LGPL-2.1 | last push 2025-04-07 | [Click & coLLecT; spec] |
| LL_prover (APLL) | focused backward search and inverse method for LL and ILL, LLTP input, Yalla certificates | OCaml | none stated | last push 2019-04-29 | [LL_prover] |
| llprover | web sequent prover for two-sided LL, announced 1997 | CGI | not stated | status unknown | [Tamura 1997] |

No controlled comparison of these exists ([spec] § MLL), so linlog's own
numbers are the reference: the focused engine is within a factor of
three of the net engine on `wide` with 8 to 1 024 literals, faster from
256 on, microseconds against seconds on every Horn encoding, and behind
only at the recursion limit (`bench/COMPARISON.md`). In intuitionistic
mode the two-sided engine is 7 to 18 times faster than the embedding on
wide and curried sequents of 1 024 and 4 096 atoms, and the net engine
alone decides the implication chain of 1 024 links
(`core/src/search/mod.rs`, the table on `Engine`).

## 2. What the step needs

**Leaf symmetry breaking.** The leaves of a pure `⊗` tree of equal
literals share every switching's component, and a pure `⅋` tree opens to
interchangeable conclusions, so swapping two such leaves is an
automorphism of the structure (`core-nets.md`). The existing break for
equal literal conclusions chains the copies in id order (`copy_before`,
`copy_after` in `net.rs`) and admits a link only if the partners ascend
with the chain, in O(1) per candidate. Recommendation: generalise the
chain's key from "root literal of the same atom and sign" to "literal of
the same atom and sign under the same maximal pure tree", computed in one
pass over the forest in index order (no recursion, `core.md`), so the hot
loop is unchanged; a tree mixing `⊗` and `⅋` is not pure. The argument
must be redone for leaves inside one tree, since the spec forbade it
([spec], "Symmetry breaking") and step 6's reviewer found the spec's
first-literal key unsound (`plan/reports/06-net-search.md`): the panel
reads it, and step 6's brute-force harness (every per-atom bijection of
random sequents, about 54 000 cases) is rerun with trees of repeated
leaves added.

**A per-atom balance over skeleton components.** The skeleton is a
union-find over `⊗` premise edges and links (`core/src/nets/skeleton.rs`),
and two literals of one component may never link (`same_component`).
(inference) Hence for every component `C` and atom `a`, the unlinked `a`
in `C` must be matched by unlinked `~a` outside `C`: `unlinked(a, C) ≤
remaining[a] − unlinked(~a, C)`. A failure is a dead end before the exact
test sees a cycle. Data: a count table per component root and atom,
summed on `union` and restored from the undo log, O(A) per link for `A`
atoms, which is small exactly on the Horn encodings where the prune is
wanted; `remaining[atom]` already exists in `Engine`. Recommendation:
implement behind a crate-private switch, measure on `partition` and
`three_partition_mll` against the leaf break alone, keep it only if it
removes nodes the break does not; the panel checks the inequality, which
is Hall's condition on one side and nothing more.

**Equal compound conclusions.** The hash-consed arena makes equal
subformulas equal `TermId`s (`Forest::term`), so copies are found for
free; the sound break needs keys under roots no symmetry moves
(`plan/later.md`). Recommendation: defer unless the baseline's inputs show
repeated compound conclusions; the doubled sequents of the differential
sample are a test, not a use case (inference).

**The routing feature.** Today `Feature::FewEqualLiterals` is
`NET_MULTIPLICITY = 2` (`search/mod.rs`); step 14 showed multiplicity is
the wrong feature, since the net engine wins by orders of magnitude on
literals repeated across conclusions and loses on equal literals inside
one pure tree (`plan/later.md`). Recommendation: add
`Feature::NoEqualLeaves` ("no two equal literals under one pure tree", one
forest pass, the same pass that builds the chains) and let the harness
decide among the three candidates (multiplicity, no-equal-leaves, both);
the row that wins is written with its measurement on `Engine`, as D19 and
step 26 require, and a feature that wins nowhere is deleted, not kept as
an option. The follow-up of step 26, a focused search that hands over to
the net engine at the recursion limit, is the alternative to any row and
should be named in the report as the measured counterfactual.

**Essential nets.** Two designs. (a) The spec's: a directed structure,
a closure bit matrix (`n` rows of `n/64` words), O(n · n/64) per link,
dominators at the end ([spec] § IMLL-Net). (b) Report 8's: the classical
engine with the dominator condition added to the `complete` branch
(`plan/reports/08-intuitionistic.md`, "For step 15"). Since every
classical net of an IMLL sequent sequentializes into an intuitionistic
proof (report 8, "IMLL by embedding"), (b) proves nothing new and can
only cost; the value of (a) is pruning: a directed cycle in a partial
linking is an O(1) rejection once the closure is kept, where the Yeo test
costs a search per round. Recommendation: build (a) as a *criterion*
behind the existing `Engine` loop, not as a second engine (D7): the
loop, the stack, the chains and the cube splitting stay, and the
criterion is a trait with the Yeo and the directed implementation. The
honest baseline is the two-sided engine, the faster of the two on IMLL
today; the row is earned only against it. Memory: `n²` bits is 2 MiB at
4 096 occurrences, 1.2 GB at 100 000 and hopeless at the default
occurrence limit of 50 million, so the closure must be charged to the search's `Account` and
refused with `Reason::MemoryLimit` beyond the bound (D16).
Sequentialization reuses `nets/sequentialize.rs` on the same structure;
the dominator read-off of [Murawski–Ong 2000] is not needed.

**The drawing.** An essential net is the classical structure with
direction and polarity: input and output literals, arrows on the edges,
the dominator tree as a second layer. Recommendation: extend
`export::svg::net` with the `Reading` rather than a second layout, and
expose the choices as `Style` fields (arrows on or off, polarity colours,
dominator tree on or off), as D15 asks; the report of step 17 already
files it as a variant of step 33's drawing.

## 3. What the library must offer

**Data model.** `Forest` (`core/src/occurrences/mod.rs`) has `parent`,
`kind`, `lca`, `root`, `subtree`, `term`, `literals(atom, sign)`,
`all_literals`, `roots`; everything the pure-tree pass needs. `Reading`
(`occurrences/reading.rs`) has `position`, `goal`, `hypotheses`,
`implication(o) -> (antecedent, consequent)`: exactly the orientation an
essential net needs, built only in intuitionistic mode
(`search::read`). Nothing is missing here. In the way: a `Reading`
borrows the forest, and `ProofStructure` owns a clone of it
(`nets/mod.rs`), so a directed structure either clones the reading's
position table (a byte per occurrence, cheap) or the structure learns to
borrow, which step 6 already wanted ("Setup cost per problem").

**Proof structures.** `ProofStructure` (`nets/mod.rs`): `partner`, the
link stack, `Graph` (CSR, `graph.rs`) with `Scratch`, `Skeleton`
(union-find with undo, `find` is `pub(super)`), `is_acyclic`,
`is_correct`, `same_component`, `link_unchecked`. Missing for step 35:
the skeleton exposes no component payload, so the balance table lives in
the engine keyed by `Skeleton::find`, which must become `pub(crate)` or
get a `component(o)` accessor; a directed closure and a dominator pass
have no place yet (a `nets/essential.rs`); `NetError` has no variant for a
directed cycle or a failed dominator condition, and `NetError`,
`ProofStructure`, `Scratch` and `svg::Style` are not `#[non_exhaustive]`,
so after 0.1.0 a new variant or public field is a version bump (D18): the
refactor should mark them, or step 35 lands before the release freezes
them. `ProofStructure::new` takes `mix: bool`; an essential net is a third
flavour, and a `Criterion` enum (`Classical { mix }`, `Essential`) in
place of the flag is the cleaner surface (inference). The text form and
the JSON `{"sequent", "mix", "links"}` are pinned in tests
(`serialize/nets.rs`); the reading is derivable from the sequent, so the
JSON needs no new key for an essential net, only a `criterion` field if
the flag becomes an enum.

**Proof term and checker.** `Node::Ax` and the rest (`proofs/mod.rs`),
`Proof::check`, `from_proof` reading links off `Ax` nodes: unchanged.
Risk from step 34: a `Cut` node would make `from_proof`'s filter silently
drop links; it must refuse or handle cuts (inference).

**Engine interface and dispatch.** `Decide { admits, decide }`, `Task {
forest, goal, fragment, mode, reading, roots }`, `Answer { result,
statistics, net: Option<ProofStructure>, refutation }`, `Engine`
(`#[non_exhaustive]`, with `implementation()`, `Display`, `parallel()`),
`DISPATCH: [Row; 5]` with `Modes` and `Feature`, `few_equal_literals`
(`search/mod.rs`). Needed: a `Feature` variant for the pure-tree
property; an `Engine::Essential` variant (then `cli::EngineArg`, the
JSON `engine` string, `Engine`'s table, README's console blocks and
`cli/tests/readme.rs` change together), which I recommend over choosing
the criterion inside `Nets::decide` from `task.reading`, since the
measurement needs to force each. `Outcome.net` carries a
`ProofStructure`; the CLI's `--net` drawing of an essential net needs
the reading beside it, which the outcome has only through `mode`.
`net::parallel` cubes reuse `explore(Some(1))`, so every prune applies to
the splitting and the workers alike, and a closure matrix is per worker
([spec]).

**Options and statistics.** `Options` has `test_period` (not in the CLI,
against D16), `jobs`, `pool`, `check`, `occurrence_limit`,
`memory_limit`; the net engine ignores its `Account` (`Engine::Net`
docs), which the closure matrix must end. New prunes change no verdict
and need no public option, only a crate-private switch for the
differential tests. `Statistics` is flat in JSON (`serialize/search.rs`);
a `pruned` counter is a JSON change, so add one only if the harness needs
it (inference).

**Exports.** Nets export to SVG, PNG and PDF only (`export/svg/net.rs`,
`svg::net(&ProofStructure, &Style, limit)`); LaTeX and Typst have no net
form. `Style` has `link_height`, `link_cap`, colours and fonts; the
essential drawing adds fields (D15) and a `Reading` parameter.

## 4. Risks, open questions, what the prompt should add

- **Soundness of the leaf break inside `⅋` trees.** The `⊗` case rests on
  a shared switching component, the `⅋` case on interchangeable
  conclusions after opening, which the spec never allowed. The prompt
  should require step 6's brute-force test extended with pure trees of
  repeated leaves, in both modes, and the panel's reading of the orbit
  argument.
- **The balance prune may buy nothing** once the leaf break exists; the
  prompt should say it is kept only on a measured difference in `nodes`
  on the Horn families.
- **Essential nets have a higher bar than the prompt states**: the
  two-sided engine, not the embedding, is the engine to beat on IMLL
  (7 to 18 times faster on wide sequents). The prompt should name it and
  the chain family (where the net engine wins) as the two sides of the
  measurement, and say that a losing engine stays as `--engine essential`
  with its drawing, no row.
- **Memory of the closure matrix** at large `n`; the prompt should ask
  for the `Account` charge and a test at the occurrence limit.
- **Interaction with step 34** (cuts in terms and nets) and step 33
  (boxes): `from_proof` and the SVG layout are shared; the prompt should
  say which lands first or how the files are split.
- **API stability**: `NetError`, `Style`, `ProofStructure::new(mix)` are
  not extensible after 0.1.0; the step or the refactor must decide.
- **Open**: whether `Feature::NoEqualLeaves` or a hand-over at the
  recursion limit is the right routing; whether the dominator condition
  prunes partial linkings at all (Moot uses only acyclicity incrementally
  [Moot 2004]) or is an end check; whether the drawing wants de Groote's
  or Lamarche's arrow direction (Moot follows de Groote).
- I could not open the HAL record of Lamarche's report (bot protection);
  its content is cited through Moot and the 1994 announcement.

## 5. Sources

- [Danos–Regnier 1989] V. Danos, L. Regnier, "The structure of
  multiplicatives", Archive for Mathematical Logic 28(3), 181–203, 1989.
  https://link.springer.com/article/10.1007/bf01622878
- [LMSS 1992] P. Lincoln, J. Mitchell, A. Scedrov, N. Shankar, "Decision
  problems for propositional linear logic", Annals of Pure and Applied
  Logic 56, 239–311, 1992. https://www.csl.sri.com/papers/lmss90/lmss90.pdf
- [Kanovich 1992] M. Kanovich, "Horn programming in linear logic is
  NP-complete", LICS 1992, 200–210.
  https://lics.siglog.org/1992/Kanovich-Hornprogramminginli.html
- [Lincoln–Winkler 1994] P. Lincoln, T. Winkler, "Constant-only
  multiplicative linear logic is NP-complete", Theoretical Computer
  Science 135(1), 155–169, 1994.
  https://ftp.math.utah.edu/pub/tex/bib/idx/tcs1990/135/1/155_169.html
- [Lamarche 1994/2008] F. Lamarche, "Proof nets for intuitionistic linear
  logic I: essential nets", report, Imperial College 1994; HAL deposit
  inria-00347336 (2008), not opened. 1994 announcement:
  https://seas.upenn.edu/~sweirich/types/archive/1994/msg00053.html ;
  HAL: https://inria.hal.science/inria-00347336
- [de Groote 1999] P. de Groote, "An algebraic correctness criterion for
  intuitionistic multiplicative proof-nets", Theoretical Computer Science
  224, 115–134, 1999. https://members.loria.fr/PdeGroote/publi.html
- [Murawski–Ong 2000] A. Murawski, C.-H. L. Ong, "Dominator trees and fast
  verification of proof nets", LICS 2000, 181–191.
  https://lics.siglog.org/2000/MurawskiOng-DominatorTreesandFa.html
- [Murawski–Ong 2006] A. Murawski, C.-H. L. Ong, "Fast verification of
  MLL proof nets via IMLL", ACM Transactions on Computational Logic 7(3),
  473–498, 2006 (bibliographic record only; no URL, the record
  formerly cited here is Guerrini's paper).
- [Moot 2004] R. Moot, "Graph algorithms for improving type-logical proof
  search", Categorial Grammars: an efficient tool for natural language
  processing, Montpellier, June 2004; arXiv 0805.2303 (2008).
  https://arxiv.org/abs/0805.2303
- [Guerrini 2011] S. Guerrini, "A linear algorithm for MLL proof net
  correctness and sequentialization", Theoretical Computer Science
  412(20), 1958–1978, 2011 (extends LICS 1999).
  https://portal.mardi4nfdi.de/entity/Q534705
- [de Naurois–Mogbil 2011] P. Jacobé de Naurois, V. Mogbil, "Correctness
  of linear logic proof structures is NL-complete", Theoretical Computer
  Science 412(20), 1941–1957, 2011. https://dblp.uni-trier.de/pid/10/2060.html
- [Bagnol–Doumane–Saurin 2015] M. Bagnol, A. Doumane, A. Saurin, "On the
  dependencies of logical rules", FoSSaCS 2015.
  https://www.normalesup.org/~bagnol/articles/conferences/dependency.pdf
- [Yeo 1997] A. Yeo, "A note on alternating cycles in edge-coloured
  graphs", Journal of Combinatorial Theory B 69, 222–225, 1997; the
  theorem as surveyed by Bang-Jensen and Gutin:
  https://www.cs.rhul.ac.uk/~gutin/paperstsp/alt.pdf
- [Nguyễn 2020] L. T. D. Nguyễn, "Unique perfect matchings, forbidden
  transitions and proof nets for linear logic with Mix", Logical Methods
  in Computer Science 16(1), 27:1–27:31, 2020.
  https://lmcs.episciences.org/6172
- [Di Guardia et al. 2025] R. Di Guardia, O. Laurent, L. Tortora de Falco,
  L. Vaux Auclair, "Yeo's theorem for locally colored graphs: the path to
  sequentialization in linear logic", FSCD 2025, LIPIcs 337, 16:1–16:18.
  https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.FSCD.2025.16
- [Matsuoka 2017] S. Matsuoka, "Direct encodings of NP-complete problems
  into Horn sequents of multiplicative linear logic", HCVS 2017.
  https://software.imdea.org/Conferences/hcvs17/papers/paper_1.pdf
- [Matsuoka 2019] S. Matsuoka, "A new linear time correctness condition
  for multiplicative linear logic", arXiv 1902.09693, 2019 (v4 2020).
  https://arxiv.org/abs/1902.09693
- [Harland–Pym 2003] J. Harland, D. Pym, "Resource-distribution via
  Boolean constraints", ACM Transactions on Computational Logic 4(1),
  56–90, 2003. https://arxiv.org/abs/cs/0012018
- [Grail] R. Moot, Grail, GitHub, LGPL-2.1, Prolog; release 3.2.0
  2015-12-01, last push 2021-02-11. https://github.com/RichardMoot/Grail
- [LinearOne] R. Moot, LinearOne, GitHub, LGPL-2.1, Prolog; last push
  2026-06-11. https://github.com/RichardMoot/LinearOne
- [PNC] S. Matsuoka, Proof Net Calculator, version 0.0.12, Scala, no
  licence stated, page updated 2023-09-21.
  https://staff.aist.go.jp/s-matsuoka/PNCalculator/index.html
- [Click & coLLecT] E. Callies, O. Laurent, "Click and coLLecT", TLLA
  2021, https://hal-lirmm.ccsd.cnrs.fr/lirmm-03271501 ; source, OCaml,
  LGPL-2.1, last push 2025-04-07:
  https://github.com/etiennecallies/click-and-collect
- [LL_prover] J.-H. Wu, LL_prover (APLL), GitHub, OCaml, no licence
  stated, last push 2019-04-29. https://github.com/wujuihsuan2016/LL_prover
- [Tamura 1997] N. Tamura, announcement of the linear logic sequent
  prover llprover, Types list, 1997.
  https://seas.upenn.edu/~sweirich/types/archive/1997-98/msg00036.html
- [spec] `proof-search-specifications.md` in this repository, sections
  "MLL" and "Intuitionistic fragments".
- Repository files are cited by path in the text.

Sources checked 2026-10-08: 24 checked, 2 corrected, 0 removed, 0 claims marked.
