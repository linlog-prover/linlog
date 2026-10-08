# Research note for step 31: a Rocq library of linlog's own

Written 2026-10-08 from the repository snapshot and the sources of
section 5. "(inference)" marks a conclusion of this note's own.

## 1. The problem and the state of the art

**What the step certifies.** linlog's proof is a term over the
occurrence forest of its sequent: an arena of fourteen kinds of `Node`,
the rules of Andreoli's dyadic calculus `⊢ Θ ; Γ` [Andreoli] plus
`Weaken` and `Mix`, premises before conclusions, no sequent recorded at
any node (`core/src/proofs/mod.rs`). The checker derives per node a
`State`: the linear zone `Γ` as a multiset of occurrence ids, the
*least* unrestricted zone `Θ` the subproof needs, and the flag `any`
that a `⊤` above absorbs any further context; the root must need an
empty `Θ` and derive the roots, or a sub-multiset of them under `any`
(`.claude/rules/core-proofs.md`, `core/src/proofs/oracle.rs`).
Intuitionistic mode adds the one-succedent condition in three clauses
over the sequent's `Reading`. Today's certificate is a tactic script for
NanoYalla, classical only, refusing Mix, affine weakening and open goals
(`core/src/export/rocq.rs`); the step wants a statement and a
certificate for every mode, proved by computation from the term
(`plan/31-rocq-library.md`).

**Formalizations of linear logic in Rocq.**

- *Yalla* (Olivier Laurent): "yet another deep embedding of linear
  logic in Rocq", LGPL-3.0, latest tag v2.0.7 of 2025-03-26, on opam as
  `rocq-yalla` 2.0.7 with `rocq-core >= 9.0 & < 9.3~`, logpath `Yalla`;
  proofs in `Type` since v2.0 (2019); master is an unreleased 2.1.0 for
  Rocq 9.2 with generalised mix rules, needing OLlibs 2.1.1 (LGPL-3.0,
  untagged) [Yalla], [Yalla-notes], [Yalla-tags], [rocq-yalla], [OLlibs].
  Its `microyalla/nanoill.v` is a standalone two-sided ILL kernel with no
  imports, `ill : list iformula -> iformula -> Type`, exchange by
  adjacent transposition, promotion requiring every hypothesis under `!`
  [nanoill]. `yalla/ll_fragments.v` proves `mix2_to_ll : ll
  (pmixupd_point_pfrag P 2 b2) l -> ll P (wn (tens bot bot) :: l)` by
  `mix_to_ll`, with structural rules and no cut [ll_fragments]. nixpkgs
  packages neither Yalla nor OLlibs [nixpkgs-rocq-packages].
- *NanoYalla*, the `nanoyalla/` directory of Click & coLLecT (LGPL-2.1,
  version 1.1.3): `nanoll.v` is about 47 lines, `ll : list formula ->
  Type` over `Atom := nat` with fourteen constructors (the axiom on
  `covar X :: var X :: nil`, an adjacent swap `ex_t_r`, the logical and
  exponential rules), no Mix, no cut; `macroll.v` the derived positional
  rules; three files the cut variants
  [C&C], [nanoyalla-dir], [nanoll]. Click & coLLecT's own exporter
  writes `From NanoYalla Require Import macrollcut`, a `Section`,
  `Variable … : formula`, and every proof as `Goal H1 -> … ->
  conclusion` with `intros`, finished or not [export_as_coq].
- *coq-ll* (Xavier, Olarte): propositional and first-order LL, several
  calculi proved equivalent, cut elimination, completeness of focusing;
  tested with Rocq 9.2; the sidebar says GPL-3.0 while the README says
  LGPL [coq-ll]. The papers: Xavier, Olarte, Reis, Nigam, ENTCS 338
  (2018) [Xavier2018]; Felty, Olarte, Xavier, MSCS (2021), with
  cut-elimination of the focused system and object logics encoded as LL
  theories [Felty2021].
- Older: Power and Webster, "Working with linear logic in Coq", TPHOLs
  1999 (a shallow consequence relation with tactics) [Power1999].
- Nets: `RemiDiG/proofnet_mll`, LGPL-3.0, MLL proof nets with
  sequentialization (through Yeo's theorem) and cut elimination, tested
  with Coq 8.20.0 on MathComp 2.2.0, OLlibs 2.0.7 and Graph Theory 0.9.5
  [proofnet_mll].
- Lean: `leanprover/cslib` (Apache-2.0) has `Cslib/Logics/LinearLogic/CLL`:
  formulas with all four units and both exponentials, sequents as
  `Multiset` with no exchange rule, files `CutElimination.lean`,
  `MLL.lean` and `PhaseSemantics/`; `Basic.lean`'s docstring still lists
  cut elimination as a TODO [cslib], [cslib-CLL], [cslib-Basic].

**How certificates are checked.** Wildmoser and Nipkow name the two
embeddings: shallow, where formulas are predicates of the host logic,
and deep, where they are a datatype [Wildmoser2004]. A certificate "as
data" is proof by reflection: a decision procedure written in Gallina
and proved sound, so that the kernel checks a goal by running it, with
a proof term of constant size beyond the input (Boutin 1997 [Boutin],
Grégoire and Mahboubi's `ring` 2005 [Gregoire2005], Chlipala's chapter
[CPDT]). Rocq runs such a check with `vm_compute` ("dramatically more
efficient than the algorithm used for the cbv tactic", one explicit cast
in the proof term) or `native_compute` (two to five times faster, more
overhead) [vm_compute]. `Print Assumptions` "displays all the
assumptions (axioms, parameters and variables)" a theorem depends on and
prints "Closed under the global context" when there are none
[PrintAssumptions]. The other route, a verified checker extracted and run outside
the prover (extraction to OCaml, Haskell or Scheme [Extraction];
cake_lpr in CakeML [cake_lpr]; Lammich's LRAT checker verified to LLVM
[Lammich2024]), buys speed but puts the extractor in the trusted base
and yields no Rocq lemma per sequent; foundational proof certificates
are the same idea in λProlog [FPC].

**Decidability, for the refutations.** Full propositional LL is
undecidable, MALL is PSPACE-complete and MLL is in NP [LMSS1992];
propositional affine LL is decidable [Kopylov1995].

**Rocq 9's ecosystem.** Releases: 9.0.0 on 2025-03-12, 9.1.0
2025-09-15, 9.1.1 2026-02-09, 9.2.0 2026-03-27, and on GitHub V9.3.0 on
19 September, marked latest (the year is not shown; it follows 9.2.0,
so 2026 (inference)) [rocq-releases], [rocq-github-releases]. The
standard library is the package `rocq-stdlib`, logpath `Stdlib`,
required as `From Stdlib Require …`; its 9.2.0 release depends on
`rocq-core >= 9.1 & < 9.4~` [stdlib-readme], [rocq-stdlib]. The 9.3.0
manual spells the project file `_RocqProject` and says "some IDEs still
look for the old name `_CoqProject`"; the 9.2.0 manual knows only
`_CoqProject`; both generate the makefile with `rocq makefile -f … -o
CoqMakefile` (9.2.0) or `-o RocqMakefile` (9.3.0) and call dune "experimental" [refman-9.3-utilities],
[refman-9.2-utilities]. The opam archive wants the name to "start with
`rocq-`", a file at `released/packages/rocq-foo/rocq-foo.1.0.0/opam`,
a `sha512=` checksum of a release asset, bounds on both sides of every
dependency, `tags` with `logpath:`, `opam lint --check-upstream`, no
`Admitted` and every axiom documented [opam-packaging]. nixpkgs builds a
library with `rocqPackages.mkRocqDerivation` (`pname`, `owner`,
`release` with hashes, `defaultVersion` by `lib.switch` on the Rocq
version, `propagatedBuildInputs`, `make` then install under
`lib/coq/<version>/user-contrib`); `rocqPackages` defaults to 9.1 and
`rocqPackages_9_3` exists [nixpkgs-rocq-manual], [nixpkgs-rocq-build],
[nixpkgs-rocq-packages]. coq-community's templates generate opam, README
and CI files from a `meta.yml`, still speaking of Coq [templates].

## 2. What the step needs

**The kernel inductives.** `formula atom` over a parameter `atom` with
decidable equality, in negation normal form with the twelve
constructors linlog's `Term` has (`core/src/sequents/term.rs`); the
`dual` as a function. `ll (p : params) : list formula -> Prop` with
`params` holding `mix` and `affine`, exchange as one rule by `Permutation`
(Stdlib: `perm_nil`, `perm_skip`, `perm_swap`, `perm_trans`, in `Prop`,
with `Permutation_count_occ` for lists over a decidable type
[Permutation]); `ill : list iformula -> iformula -> Prop` with its own
`affine` flag over `ivar ione itop izero itens ilmap iwith iplus ioc`,
Yalla's `nanoill.v` shape [nanoill]. Recommendation: `Prop`, not `Type`
as Yalla and NanoYalla have it [Yalla-notes], [nanoll], because a
certificate states provability and never computes with the derivation,
and `Prop` keeps the bridges' direction `nano -> ours` trivial and the
other direction a plain induction (inference). Recommendation: one
exchange rule by `Permutation` rather than adjacent swaps, since the
soundness proof moves whole zones around and `Permutation_middle`,
`Permutation_app_comm` and `Permutation_app_inv` are there [Permutation];
the bridge to NanoYalla's `ex_t_r` is one lemma either way (inference).

**The checker.** A `Fixpoint` over the node list that keeps the states
of nodes a later node reads. Costs, from the Rust pass: a rule reads at
most two premise states and consumes at most two members
(`core-proofs.md`). Data structures and their costs in Rocq:

- Node table: a `PositiveMap` (binary trie keyed by `positive`, no path
  compression [FMapPositive]) from node index to state, logarithmic in
  the index; a `list state` with `nth_error` would be linear per lookup
  and quadratic overall (inference). Recommendation: `PositiveMap`, and
  remove a state once its reader count is used up only if the proof
  gets long enough to matter; first version keeps all.
- `Γ`: a sorted association list `(occ, count)`, canonical, so `&`'s
  "equal zones" is syntactic `list` equality and the sub-multiset test,
  the sum and the pointwise maximum are linear merges (inference).
  Stdlib's `Multiset` is a `Bag` of a function to `nat` whose `meq` is a
  `Prop` and not decidable [Multiset], so it serves the statement of
  lemmas, not the checker.
- `Θ`: a sorted `list nat` of occurrence ids; union and removal linear.
- The forest: computed in Rocq from the formula list by linlog's
  preorder walk (roots in order, left before right, subtree `o .. o +
  size(o)`; `core/src/occurrences/mod.rs`), as records (kind, parent,
  children, atom, sign) in a `PositiveMap`. Computing it in Rocq removes
  a side condition the certificate would otherwise carry (inference).
- Mode checks as in `Pass::rule`: `Weaken` allowed on a `?` formula in
  every mode and on anything in affine mode, never on an output; `Mix`
  needs `mix` and not `intuitionistic` (`core/src/proofs/check.rs`).
- `Surplus` is not needed over `nat`: the counters never wrap, and a
  node whose zone cannot be consumed fails at the root anyway (the
  prompt's item 7; inference for the second half, to be stated as the
  lemma "a Rust `Surplus` rejection is a Rocq `false` too", or left out
  with the report saying why).

**The soundness theorem and its invariant.** The Rust rules file
claims the state characterises exactly the dyadic sequents the subterm
proves (`core-proofs.md`); Rocq needs soundness alone: if node `i` gets
the state `(Θ, Γ, any)`, then for every `Θ' ⊇ Θ` and every `Δ` (empty
unless `any`), `ll (map wn (formulas Θ') ++ formulas Γ ++ Δ)`. The
calculus supplies weakening and contraction of `?` formulas, and `any`
encodes that the absorbed `Δ` passes through every rule (inference).
Recommendation: strong induction on the node index over the
`PositiveMap` built so far, the checker's loop stated as a fold with an
invariant, as Chlipala's reflective tautology checker does [CPDT].

**The two-sided statement.** Intuitionistic proofs are classical nodes
plus the clauses R1–R3 over positions (`core-proofs.md`); the reader
wants `ill (hypotheses) goal` over `iformula`. Recommendation: the
checker run with the `Reading`, its soundness proof building the
two-sided derivation directly with a "this subproof's output is `o`, or
absorbed" component; the alternative, a general theorem that a one-sided
proof with one output per sequent is an `ill` derivation, is the
polarization theorem and larger (inference). The statement names the
goal the `Reading` chose, with the `⊤`/`0` ambiguity (`plan/later.md`,
"Follow-ups: intuitionistic mode").

**Atoms in the statement.** `Nat.eqb a b` computes only on numerals, so
the term is checked over atoms `0, 1, …`. Click & coLLecT's export is
schematic over `Variable A : formula` [export_as_coq], and linlog's binds
`(A B : formula)` (`core/src/export/rocq.rs`). Recommendation: prove
once that `ll` is closed under substituting formulas for atoms (its
axiom case is the usual `ax` expansion) and state every lemma
schematically, proved by that lemma on the computed instance, so the
statement keeps the NanoYalla export's strength (inference).

**Certified refutations (item 8).** `Unbalanced` and `Equation` are
invariants of every derivation in the modes the search relies on them
(`core/src/search/mod.rs`, `Refutation`); the classical reading's
falsifying assignment needs a lemma "every `ll` sequent is a classical
tautology under the erasure". Each is a `Theorem … : ~ ll l`-shaped
statement over the inductive, proved from the lemma and a `vm_compute`
of a few numbers (inference). Where Mix is on, `Equation` is an
inequality (`Refutation::Equation.mix`).

**The build.** `rocq/` with `_RocqProject` (`-R theories Linlog`), a
`Makefile` from `rocq makefile`, `rocq-linlog.opam` by the archive's
rules [opam-packaging], a `mkRocqDerivation` with
`propagatedBuildInputs = [ stdlib ]` [nixpkgs-rocq-manual], and a check
that compiles every certificate of both kernels and prints the main
theorem's assumptions. Recommendation: build on the nixpkgs default
`rocqPackages` (9.1) and also on `rocqPackages_9_3` in the same check,
so that the opam bounds are tested [nixpkgs-rocq-packages]; `rocq
makefile -f` takes any file name [refman-9.3-utilities], so the name
`_RocqProject` works on 9.1 too (inference).

## 3. What linlog's library must offer

**Data model.** `Sequent` is an arena of `Term`s in NNF with roots in
order and an atom dictionary; `Term`'s twelve variants map one to one
onto the kernel's constructors, and `export/rocq.rs::term` already
writes them over a `Walk` (`core/src/sequents/mod.rs`, `term.rs`,
`core/src/export/rocq.rs`). `Forest` numbers occurrences in preorder,
deterministically (`core/src/occurrences/mod.rs`); `Reading` gives
positions, goal and hypotheses (`core/src/occurrences/reading.rs`). In
the way: nothing. Missing: a documented promise that the numbering is a
contract a foreign checker may recompute (today the doc comment of
`Forest`), since the Rocq forest and the term's ids must agree.

**Proof term.** `Node` (16 bytes, `OccId`s and `NodeId`s) and `Proof`
(forest plus `Box<[Node]>`, premises before conclusions, root last,
`Proof::new` checks bounds and order; `core/src/proofs/mod.rs`). This is
exactly the datatype the Rocq `node` mirrors. Missing: an `Iterator`
over nodes that yields what the exporter writes (`Node::occurrences`,
`Node::premises` exist and suffice); nothing in the way. The cut rule
(step 34) will add a variant and the Rocq datatype a constructor
(`plan/34-cut.md`); D17's terms and witnesses would turn an occurrence
into "occurrence under a substitution" in `Bag`'s key
(`plan/reports/18-bounded-proofs.md`), which the Rocq side should keep
room for by making the node's occurrence argument a parameter type.

**Checker.** `check.rs` is the one pass with tables and the memory
bound; `oracle.rs` (test-only, `Derived { theta: OccSet, gamma:
Multiset, any }`, `Step::rule`) is the text to translate, rule by rule
(`core/src/proofs/oracle.rs`). Problems: `Forbidden`, `Shape`,
`Succedents(n)`, `Kind`, `NotDual`, `Missing`, `NotEmpty`, `Differ`,
`NotUnderQuest`, `Surplus`, `Conclusion`, `Memory` (`check.rs`). Missing
for the step: nothing in code; the report must say which Rust refusals
(`Surplus`, `Memory`) have no Rocq counterpart and why the verdicts
still agree on every term both accept or reject (inference).

**Engine interface and dispatch.** `Decide` (`admits`, `decide`),
`DISPATCH`, `engine_for` (`core/src/search/mod.rs`) are untouched by the
step: a certificate is made from a checked `Proof` after the search,
whichever engine produced it. In the way: nothing.

**The exporter's API.** Today `rocq::write(&Derivation, &Options, out,
stop)` reads the derivation view, which holds a `Forest` and
`Inference`s but not the term or the mode (`core/src/proofs/derivation.rs`,
`core/src/export/rocq.rs`). A term-as-data certificate needs the
`Proof` and the `Mode` (the lemma states `ll` with Mix, the affine
calculus, or `ill`). Missing: a second entry, `rocq::write_proof(&Proof,
Mode, &Options, out, stop)` with the same `WriteError`, the stop asked
per node, and a `String` twin; a `Proof` reference inside `Derivation`
would not supply the mode (inference). The verdict comment and
`Compact::Never` in `cli/src/prove.rs` stay; `interact`'s `proof
--rocq` has a finished term (`cli/src/interact.rs`) and takes the new
entry.

**Options (D15).** `rocq::Options { form, lemma, prelude }` with
`serde(default, deny_unknown_fields)`; the default prelude is `From
NanoYalla Require Import macroll.` (`core/src/export/rocq.rs`). Missing:
a `kernel: Kernel` field with `Auto` (NanoYalla for a classical linear
proof, linlog's kernel elsewhere), `NanoYalla` and `Linlog`; a prelude
per kernel (an `Option<String>` whose `None` is the kernel's default);
the snapshots stay byte for byte under `Auto`, since all are classical
linear (`core/tests/snapshots/*.v`). `Unsupported::{Mix,
AffineWeakening}` become kernel-specific: refused for NanoYalla, accepted
by linlog's; `Open` and `Compact` stay. The CLI maps `--lemma`,
`--prelude`, `--style rocq.KEY=VALUE` and `--standalone` onto the value
(`cli/src/style.rs`, `cli/src/argument_parsing.rs`); a `--kernel` flag or
`rocq.kernel=…` is one line there. `RESERVED` must grow by the new
kernel's names (`rocq.rs`).

**JSON wire forms.** `Proof` is `{"sequent": …, "proof": [{"ax":[x,y]},
{"⊗":[o,l,r]}, …]}` with tags that are part of the interchange format
(`core/src/serialize/proofs.rs`), and `Sequent` is `{"terms", "ids",
"var_dict"}` (`core/src/serialize/sequents.rs`). The `.v` certificate
carries the same data as a Rocq list literal; nothing new is needed on
the wire. A web client (step 32) will send `rocq::Options` as JSON and
receive the `.v`, as it does today (`plan/reports/12-certificates.md`).

**The flake.** `modules/rocq.nix` builds NanoYalla from the non-flake
input and compiles the snapshots and four CLI certificates with
`rocq-core` and `rocqPackages.stdlib`, requiring Rocq to print nothing
(`modules/rocq.nix`, `.claude/rules/flake.md`). Missing: a package
`rocq-linlog` by `mkRocqDerivation` from the in-tree `rocq/` source, the
check extended to compile that package, every certificate of both
kernels, and `Print Assumptions`; the ordinary-logic certificates
(`core/src/ordinary/rocq.rs`) need no library and stay as they are.

## 4. Risks, open questions, what the prompt should add

Risks:
- The soundness proof over the least `Θ` and the absorbing `⊤` is the
  real work; the `&` case with one absorbing premise (`is_subset`,
  result exact) and the promotion that resets `any` are the delicate
  cases (`check.rs`, `oracle.rs`). The two-sided soundness is larger
  still (section 2).
- Checking time: `vm_compute` runs the whole pass; a proof of tens of
  thousands of nodes (the 46 768-node file of `core-proofs.md`) may be
  slow in the bytecode VM; `native_compute` is the fallback [vm_compute].
- Rocq version drift: nixpkgs defaults to 9.1, the archive's newest is
  9.3.0, `rocq-stdlib` 9.2.0 needs `rocq-core >= 9.1`; a `From Stdlib`
  development with bounds `>= 9.1 & < 9.4~` covers all of them
  [nixpkgs-rocq-packages], [rocq-github-releases], [rocq-stdlib]
  (inference on the bounds).
- Licence: nothing of Yalla or NanoYalla is copied; the bridges only
  `Require` the pinned kernels when checked (`plan/later.md`), `nanoill.v`
  being a file without its own header in an LGPL-3.0 repository
  [nanoill], [Yalla]; the report should say so.

Open questions:
1. `Prop` or `Type` for `ll` and `ill` (section 2 recommends `Prop`;
   with NanoYalla and Yalla in `Type`, only `theirs -> ours` is then
   provable).
2. Does the Rocq checker recompute the forest from the formula list, or
   does the certificate carry the forest and the checker verify it?
   (Recommended: recompute.)
3. Schematic lemmas through a substitution lemma, or concrete atoms as
   numerals with `Definition A := 0`?
4. One checker with a `Reading` parameter or two checkers for the
   one-sided and the two-sided statements?
5. Whether `Surplus` is stated as a lemma or dropped with a sentence.
6. Where the certified refutations' exporter searches for a falsifying
   assignment and within what bound (item 8 of the prompt).
7. Whether `rocq-linlog` gets an opam release at the end of the step or
   only the file (D22 says after a release of its own).

What the prompt should add:
- The exact signature of the new export entry (`&Proof`, `Mode`) and the
  `Kernel` field with its `Auto` rule, so that the snapshots stay.
- The list of Rocq versions the check runs (`rocqPackages` default and
  `rocqPackages_9_3`), and the `rocq-core`/`rocq-stdlib` bounds.
- That `_RocqProject` is the file's name even on a 9.1 toolchain, with
  `rocq makefile -f _RocqProject`.
- A measurement of checking time on the largest snapshot and one family
  instance, reported in the step's report.
- That `coq-ll`'s `FOLL/` and Yalla's first-order formulas are the
  references to read before fixing where quantifiers go (D17).

## 5. Sources

In the repository (the snapshot of 2026-10-08): `plan/31-rocq-library.md`;
`plan/later.md` ("Second certificate kernels", "Certified refutations",
"Follow-ups: intuitionistic mode"); `plan/README.md` (D15–D23);
`plan/notes/distribution.md`; `plan/reports/12-certificates.md`,
`17-assessment.md` (3.10), `18-bounded-proofs.md`; `plan/34-cut.md`;
`.claude/rules/core.md`, `core-proofs.md`, `core-export.md`, `flake.md`;
`core/src/lib.rs`, `core/src/proofs/mod.rs`, `check.rs`, `oracle.rs`,
`derivation.rs`; `core/src/occurrences/mod.rs`, `reading.rs`;
`core/src/sequents/mod.rs`, `term.rs`; `core/src/serialize/proofs.rs`,
`sequents.rs`; `core/src/export/rocq.rs`, `mod.rs`;
`core/src/ordinary/rocq.rs`; `core/src/search/mod.rs`;
`core/tests/snapshots/*.v`; `cli/src/style.rs`, `argument_parsing.rs`,
`prove.rs`, `interact.rs`; `modules/rocq.nix`, `flake.nix`.

Outside:

- [Andreoli] J.-M. Andreoli, "Logic programming with focusing proofs in
  linear logic", Journal of Logic and Computation 2(3), 1992, 297–347.
  https://doi.org/10.1093/logcom/2.3.297
- [Yalla] O. Laurent, Yalla repository (README, licence).
  https://github.com/olaure01/yalla
- [Yalla-notes] Yalla, RELEASE_NOTES.md.
  https://raw.githubusercontent.com/olaure01/yalla/master/RELEASE_NOTES.md
- [Yalla-tags] Yalla, tags. https://github.com/olaure01/yalla/tags
- [nanoill] Yalla, `microyalla/nanoill.v`.
  https://raw.githubusercontent.com/olaure01/yalla/master/microyalla/nanoill.v
- [ll_fragments] Yalla, `yalla/ll_fragments.v`.
  https://raw.githubusercontent.com/olaure01/yalla/master/yalla/ll_fragments.v
- [rocq-yalla] Rocq package index, `rocq-yalla`.
  https://rocq-prover.org/p/rocq-yalla
- [OLlibs] O. Laurent, OLlibs repository. https://github.com/olaure01/ollibs
- [C&C] Click & coLLecT repository (licence, README).
  https://github.com/ComputerAidedLL/click-and-collect
- [nanoyalla-dir] Click & coLLecT, `nanoyalla/`.
  https://github.com/ComputerAidedLL/click-and-collect/tree/master/nanoyalla
- [nanoll] Click & coLLecT, `nanoyalla/nanoll.v`.
  https://raw.githubusercontent.com/ComputerAidedLL/click-and-collect/master/nanoyalla/nanoll.v
- [export_as_coq] Click & coLLecT, `export_as_coq.ml`.
  https://raw.githubusercontent.com/ComputerAidedLL/click-and-collect/master/export_as_coq.ml
- [coq-ll] B. Xavier, C. Olarte, coq-ll repository.
  https://github.com/meta-logic/coq-ll
- [Xavier2018] B. Xavier, C. Olarte, G. Reis, V. Nigam, "Mechanizing
  focused linear logic in Coq", ENTCS 338, 2018, 219–236 (record).
  https://repositorio.ufrn.br/items/ff81affa-bc24-484e-9aae-007f537c19f7/full
- [Felty2021] A. Felty, C. Olarte, B. Xavier, "A focused linear logical
  framework and its application to metatheory of object logics",
  Mathematical Structures in Computer Science, 2021.
  https://www.site.uottawa.ca/~afelty/dist/mscs21.pdf
- [Power1999] J. F. Power, C. Webster, "Working with linear logic in
  Coq", TPHOLs 1999 (work-in-progress version).
  https://mural.maynoothuniversity.ie/id/eprint/6461/
- [proofnet_mll] R. Di Guardia, proofnet_mll repository.
  https://github.com/RemiDiG/proofnet_mll
- [cslib] leanprover/cslib repository. https://github.com/leanprover/cslib
- [cslib-CLL] cslib, `Cslib/Logics/LinearLogic/CLL`.
  https://github.com/leanprover/cslib/tree/main/Cslib/Logics/LinearLogic/CLL
- [cslib-Basic] cslib, `CLL/Basic.lean`.
  https://raw.githubusercontent.com/leanprover/cslib/main/Cslib/Logics/LinearLogic/CLL/Basic.lean
- [Wildmoser2004] M. Wildmoser, T. Nipkow, "Certifying machine code
  safety: shallow versus deep embedding", TPHOLs 2004, LNCS 3223,
  305–320. https://www21.in.tum.de/~nipkow/pubs/tphols04.pdf
- [Boutin] S. Boutin, "Using reflection to build efficient and certified
  decision procedures", LNCS 1281, 1997, from p. 515.
  https://doi.org/10.1007/BFb0014565
- [Gregoire2005] B. Grégoire, A. Mahboubi, "Proving equalities in a
  commutative ring done right in Coq", TPHOLs 2005, LNCS 3603, 98–113.
  https://doi.org/10.1007/11541868_7
- [CPDT] A. Chlipala, Certified Programming with Dependent Types,
  chapter "Proof by Reflection".
  http://adam.chlipala.net/cpdt/html/Reflection.html
- [cake_lpr] Y. K. Tan, M. J. H. Heule, M. O. Myreen, "cake_lpr:
  Verified propagation redundancy checking in CakeML", TACAS 2021, LNCS
  12652, 223–241. https://link.springer.com/chapter/10.1007/978-3-030-72013-1_12
- [Lammich2024] P. Lammich, "Fast and verified UNSAT certificate
  checking", IJCAR 2024.
  https://research.utwente.nl/en/publications/fast-andverified-unsat-certificate-checking/
- [FPC] Z. Chihani, D. Miller, F. Renaud, "Checking foundational proof
  certificates for first-order logic", PxTP 2013.
  https://www.lix.polytechnique.fr/~dale/papers/checking-fpc.pdf
- [LMSS1992] P. Lincoln, J. Mitchell, A. Scedrov, N. Shankar, "Decision
  problems for propositional linear logic", Annals of Pure and Applied
  Logic 56, 1992, 239–311. https://www.csl.sri.com/papers/lmss90/
- [Kopylov1995] A. P. Kopylov, "Decidability of linear affine logic",
  LICS 1995, 496–504.
  https://lics.siglog.org/archive/1995/Kopylov-DecidabilityofLinea.html
- [rocq-releases] Rocq Prover, releases page.
  https://rocq-prover.org/releases
- [rocq-github-releases] rocq-prover/rocq, GitHub releases.
  https://github.com/rocq-prover/rocq/releases
- [stdlib-readme] rocq-prover/stdlib, README.
  https://raw.githubusercontent.com/rocq-prover/stdlib/master/README.md
- [rocq-stdlib] Rocq package index, `rocq-stdlib`.
  https://rocq-prover.org/p/rocq-stdlib/latest
- [refman-9.3-utilities] Rocq 9.3.0 reference manual, "Building Rocq
  projects". https://rocq-prover.org/doc/V9.3.0/refman/practical-tools/utilities.html
- [refman-9.2-utilities] Rocq 9.2.0 reference manual, the same page.
  https://rocq-prover.org/doc/V9.2.0/refman/practical-tools/utilities.html
- [opam-packaging] Rocq Prover, "Opam packaging".
  https://rocq-prover.org/docs/opam-packaging
- [PrintAssumptions] Rocq reference manual, vernacular commands (`Print
  Assumptions`). https://rocq-prover.org/refman/proof-engine/vernacular-commands.html
- [vm_compute] Rocq reference manual, "Reasoning with equalities"
  (`vm_compute`, `native_compute`).
  https://rocq-prover.org/refman/proofs/writing-proofs/equality.html
- [Extraction] Rocq reference manual, "Program extraction".
  https://rocq-prover.org/refman/addendum/extraction.html
- [Permutation] Rocq standard library, `Stdlib.Sorting.Permutation`.
  https://rocq-prover.org/stdlib/Stdlib.Sorting.Permutation.html
- [Multiset] Rocq standard library, `Stdlib.Sets.Multiset`.
  https://rocq-prover.org/stdlib/Stdlib.Sets.Multiset.html
- [FMapPositive] Rocq standard library, `Stdlib.FSets.FMapPositive`.
  https://rocq-prover.org/stdlib/Stdlib.FSets.FMapPositive.html
- [nixpkgs-rocq-manual] nixpkgs manual, Rocq section.
  https://raw.githubusercontent.com/NixOS/nixpkgs/master/doc/languages-frameworks/rocq.section.md
- [nixpkgs-rocq-build] nixpkgs, `pkgs/build-support/rocq/default.nix`.
  https://raw.githubusercontent.com/NixOS/nixpkgs/master/pkgs/build-support/rocq/default.nix
- [nixpkgs-rocq-packages] nixpkgs, `pkgs/top-level/rocq-packages.nix`.
  https://raw.githubusercontent.com/NixOS/nixpkgs/master/pkgs/top-level/rocq-packages.nix
- [templates] coq-community/templates repository.
  https://github.com/coq-community/templates

Sources checked 2026-10-08: 47 checked, 1 corrected, 0 removed, 0 claims marked.
