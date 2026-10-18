# Step 31: a Rocq library of linlog's own

## The checklist

| item | state | evidence |
|---|---|---|
| Session 31a: the definitions, the checker and its soundness (session `step-31a`, 2026-10-18) | done (2026-10-18) | The library under `rocq/` (eae340fd): `Formulas`, `Calculus` (the one-sided calculus as a plain inductive, Mix and weakening as parameters), `Proofs` (the term, one constructor per `Node` variant), `Check` (the checker as a function), `Soundness` (`soundness` for every `rules` value and `soundness_classical`), no `Admitted`, no axiom, `Print Assumptions` of both theorems "Closed under the global context". The flake (889c1cc5): the package `rocq-linlog` built with `mkRocqDerivation` and the check `rocq-linlog`, which compiles the fourteen hand-written lemmas (nine accepted proofs, five rejections) and seven translated proofs of `rocq/tests/` (the largest 192 nodes, all in under a second) and requires the two assumption lines; both built under the machine lock. Documentation (da2a7e46): `.claude/rules/rocq.md`, CLAUDE.md, README, the changelog. One reviewer (Opus 5.5, `high`) read the inductive against the textbook calculus and NanoYalla's kernel: no must-fix, two should-fix (the `let*` notation made local; `ll` in `Prop` shapes 31c's bridge as `inhabited`), three notes, all answered and the wording fixed in 31f48eff; its probe proved `ll classical` and NanoYalla's `ll` derive the same sequents. Gate: passed at every code commit in the gate workspace `../linlog-step31a-gate` (eae340fd 07:41 to 07:45, 889c1cc5 to 07:47, 31f48eff to 07:53, 2c7e88f6 to 07:54, each `gate: passed`, exit 0), and once before on the working copy they were split from. `nix flake check` at the last commit: passed ("all checks passed!", exit 0) under the lock at 2c7e88f6, the last code commit, and again at the report commit. Every commit unsigned (`signing.behavior=drop`; the passphrase was not cached). |

## Session 31a: the definitions, the checker and its soundness

**Outcome.** The Rocq library exists under `rocq/`, logical path
`Linlog`, on Rocq 9.1.1 and its standard library 9.0.0 (what the flake's
nixpkgs ships) and nothing else. It holds the one-sided cut-free sequent
calculus of classical linear logic as a plain inductive `ll`, with
binary Mix and general weakening as the two Boolean fields of a `rules`
record and `classical` the value with neither; linlog's proof term as the
datatype `proof`, one constructor per variant of the Rust `Node`; the
checker as the functions `derive` and `check`; and the theorem
`soundness`, which says that a term `check` accepts for a sequent, in
any `rules`, proves that sequent in `ll`, and its instance
`soundness_classical`. Both are closed under the global context, and no
file under `rocq/` has an `Admitted`, an `admit` or an axiom; the flake's
check greps for them and prints the assumptions. A certificate is a
lemma stating `ll classical Γ`, proved by `apply (soundness_classical
String.eqb String.eqb_spec term); vm_compute; reflexivity`: fourteen
lemmas written by hand (nine accepted proofs covering every rule but
Mix, five pinning what the checker rejects) and seven certificates
translated from the command's own proofs by a scratch program, the
largest 192 nodes, compile in under a second together, and the
assumptions file prints the two theorems' assumptions. The
flake builds the library as nixpkgs builds a Rocq library (the package
`rocq-linlog`) and checks it (the check `rocq-linlog`). Nothing in the
exporter changed: the NanoYalla export and its snapshots are as they
were, and the kernel behind `rocq::Options` is session 31b's.

### The library

- `rocq/_RocqProject` (`-R theories Linlog`, the five files listed),
  `rocq/Makefile` (the reference manual's wrapper: it generates
  `RocqMakefile` from `_RocqProject` with `rocq makefile` and hands it
  every target), `rocq/rocq-linlog.opam` (30c's draft with its "draft"
  comment removed; nothing else needed changing: its `build` and
  `install` run `make -C rocq`, which the wrapper serves, and the flake
  reads its `version:`), `rocq/theories/` (the library) and `rocq/tests/`
  (the certificates the check compiles, not installed).
- `Formulas.v`: `formula` over `Context {atom : Type}`, the constructors
  `var`, `covar`, `one`, `bot`, `top`, `zero`, `tens`, `parr`, `awith`,
  `aplus`, `oc`, `wn`, named as in Yalla and NanoYalla so that 31c's
  bridge reads line by line; `dual` with `dual_dual`; `is_wn`;
  `formula_eqb` over a Boolean atom equality with `formula_eqb_spec` under
  the hypothesis that the atom equality reflects, and `formula_eq_dec`
  for the proofs.
- `Calculus.v`: `rules` (`mix`, `weakening`), `classical`, and `ll R :
  list formula -> Prop` with sixteen constructors: `ax_r` on an atom
  (`[covar a; var a]`, NanoYalla's orientation), `ex_r` with the standard
  library's `Permutation`, `one_r`, `bot_r`, `top_r`, `tens_r` (context
  split), `parr_r`, `with_r` (shared context), `plus_r1`, `plus_r2`,
  `oc_r` (a `?` context, written `map wn l`), `de_r`, `wk_r`, `co_r`,
  `mix_r` under `mix R = true` and `weakening_r` under `weakening R =
  true`. No rule for `zero`, no cut. Every rule acts on the first formula,
  and exchange does the rest.
- `Proofs.v`: `branch` (`Left`, `Right`) and `proof` with `Ax (a : atom)`,
  `Tensor A B l r`, `Par A B p`, `One`, `Bot p`, `With A B l r`,
  `Plus A B side p`, `Top`, `Bang A p`, `Quest A p`, `Copy A p`,
  `Weaken A p`, `Mix l r`, in the order of `Node::NAMES`.
- `Check.v`: lists read as multisets (`remove_one`, `remove_all`,
  `take_all`, `diff`), the `state` record (`theta`, `gamma`, `any`),
  `take`, `join_with` (the four cases of `&`), `derive` and `check`, with
  a `let*` notation for the option monad so that a rule reads as a
  sequence of steps.
- `Soundness.v`: the multiset functions as permutations, `wk_list` and
  `wn_contract` as derived rules, the invariant `sat θ Γ any := forall Δ,
  (any = false -> Δ = []) -> ll R (Γ ++ map wn θ ++ Δ)`, `take_spec` and
  `take_sat`, `derive_sound` by induction on the term (thirteen cases,
  the `&` case in four), `soundness` and `soundness_classical`. One
  tactic, `perm`, decides every permutation of formula lists by
  `Permutation_count_occ`, `count_occ_app` and `lia`, which is what keeps
  the proof short (300 lines for everything).

### Decisions

- **Formulas, not occurrences, in the term.** The Rust node names an
  occurrence of the forest; in a multiset of formulas two occurrences of
  one formula are interchangeable, so the Rocq node names the formulas
  the rule acts on and no forest, numbering or reading is needed. Three
  consequences, each sound: `Ax` takes one atom (the Rust node's two
  literals in either order give the same multiset), `Copy A` needs no `?`
  above it (what makes it sound is the root's empty unrestricted zone,
  which only a `Quest` of `?A` below can give), and `Quest A` removes
  every `A` from the unrestricted zone, the proof contracting the copies
  or weakening `?A` when there is none. The exporter (31b) writes the
  formula of each occurrence; a term whose subformulas repeat can bind
  them with `let` or `Definition`s if its size matters.
- **Exchange as one `Permutation` rule.** NanoYalla's kernel exchanges
  adjacent formulas; a textbook reads sequents as multisets, and the
  standard library's `Permutation` is the smallest trusted statement of
  that. The checker never needs exchange: it compares multisets.
- **The checker's state is a list for Θ too.** The Rust Θ is a set; a
  list with duplicates is simpler to compute with and the proof's
  `wn_contract` absorbs the difference. `Γ` is a list compared by
  `take_all`, whose lemma `take_all_perm` (`take_all l1 l2 = Some r ->
  Permutation l2 (l1 ++ r)`) carries the `&` cases and the root; the
  union of two absorbing zones is `gl ++ diff gr gl`, and `diff_count`
  (`count (diff l2 l1) x = count l2 x - count l1 x`) makes its symmetry
  one `lia` call.
- **One theorem for every `rules` value.** The parameters were to be in
  the calculus from the start; proving soundness for each mode separately
  would have cost more than the two trivial cases (`Weaken` under
  `weakening`, `Mix` under `mix`) they add. So `soundness` is general and
  `soundness_classical` is its instance; 31b's Mix and affine theorems
  are instances too, and its work is the two-sided statement, the kernel
  in the exporter and the check over both kernels.
- **Atoms are abstract, certificates use strings.** The calculus assumes
  nothing of atoms (NanoYalla's `Atom` is `nat`, Yalla's a parameter, so
  the bridges instantiate freely); the checker takes `atom_eqb` and the
  theorem its reflection proof. A certificate reads best with the atoms'
  names, so it uses `string`, `String.eqb` and `String.eqb_spec`. A
  sequent without an atom names the type: `ll (atom := string) classical
  [one; bot]`. `formula`, `proof` and `state` have their `atom` cleared of
  implicitness so they are written applied, as `list (formula string)`.
- **`_RocqProject` and `RocqMakefile`.** The 9.1 reference manual (the
  Rocq nixpkgs ships) names `_CoqProject` and `CoqMakefile`; the 9.3
  manual names `_RocqProject` and `RocqMakefile`, and nothing in 9.1 reads
  either name by itself (both are passed with `-f` and `-o`). The plan's
  names are the forward ones, so the library uses them; the wrapper is
  the 9.3 manual's text, whose `$(COQBIN)` is also what the 9.1 generated
  makefile uses.
- **Built with `mkRocqDerivation`**, the function behind every Rocq
  library of nixpkgs, from a release entry whose `src` is the tree's
  `_RocqProject`, `Makefile` and `theories/` (documented overrides of the
  fetcher's result; `hash = ""` stands for the fake hash nothing fetches),
  so the package is `rocq9.1-linlog-0.1.0` and installs under
  `lib/coq/9.1/user-contrib/Linlog/`. The check is a `runCommand` with
  `ROCQPATH` naming the standard library's and the package's
  `user-contrib`, as the NanoYalla check does; the tests are not part of
  the package, so a certificate's edit rebuilds the check alone.
- **What the check accepts**: silence from every certificate (Rocq prints
  nothing for a file it accepts, and the existing `rocq` check holds the
  same), and from `Assumptions.v` exactly the two lines `Closed under the
  global context`, compared with `diff`; and no `Admitted`, `admit`,
  `Axiom`, `Parameter` or `Conjecture` in the sources, a grep that
  documents the rule, since `Print Assumptions` is the proof of it.
- **Conventions applied**: `From Stdlib Require` throughout, explicit
  `Local Notation`s instead of implicit section variables where a type
  is meant, rocqdoc comments on every definition, the licence header as
  two `(* … *)` lines, and the `conventions` check extended to `.v` files
  (header and no plan names in comments); `implicits` (Rocq's `Arguments
  … : clear implicits`) is a word of `typos.toml`.

### Item 5: how a first-order extension sits beside this one

The propositional development is closed over its formula type and
nothing in it anticipates binders, as D17 asks. A first-order library is
a second development beside it: its own formula type, in which an atom
becomes a predicate symbol applied to terms and two constructors add the
quantifiers, its own `dual` and equality (now over terms and binders),
its own `ll` with the two quantifier rules and the eigenvariable side
condition of `∀`, its own `proof` with two more constructors (the witness
of `∃` and the eigenvariable of `∀`), and a checker with two more cases,
reusing the multiset functions (they are generic in the formula type
once given an equality), the shape of `sat` and the `perm` tactic. The
checker's `take` and `join_with` need no change in kind; what a
first-order checker adds is a freshness check on the eigenvariable
against the whole sequent, which the state has (`gamma`, `theta` and
the absorbed `Δ`, which the eigenvariable condition must quantify over
and which the absorbing flag makes delicate: a `⊤` above a `∀` would have
to absorb only formulas without the eigenvariable, so that development
will carry the absorbed context explicitly or forbid `⊤` to absorb
across a `∀`). Nothing here changes for it.

### Item 7: `Fault::Surplus` over `nat`

Nothing. The Rust refusal exists because its zones are counters of a
fixed width, and a term may double a zone at every node; over lists the
zone is whatever it is, no arithmetic can wrap, and a zone the rest of
the proof cannot consume fails where the Rust checker would otherwise
fail too, at the root's comparison (`take_all gamma sequent`). A lemma
stating the bound (node `i` of `n` derives at most `|Γ| + 2(n − 1 − i)`
formulas when the term is accepted) is true and was not needed for
soundness, so it is not in the library. `CheckError::Refused` has no
counterpart either, as the prompt says.

### What is verified

- `rocq compile` of every module and every test file with the pinned
  Rocq (9.1.1, `/nix/store/8gzfbjj2z8di0i7fl2pifn7m9xhj1spr-rocq-9.1.1`)
  and standard library (9.0.0) on cores 8 to 11 in capped scopes;
  `tests/Assumptions.v` prints `Closed under the global context` twice;
  `tests/FromLinlog.v` (seven certificates, 38, 10, 10, 10, 25, 192 and
  30 nodes) compiles in 0.65 s wall.
- `nix build .#rocq-linlog` under `flock /tmp/linlog-machine.lock`:
  `/nix/store/hmf4azgfzjma8ypf7jqg4fngk32gysk6-rocq9.1-linlog-0.1.0`, the
  five `.vo` files installed under `lib/coq/9.1/user-contrib/Linlog/`.
- `nix build .#checks.x86_64-linux.rocq-linlog` under the lock: built,
  exit 0, its log the two assumption lines.
- `typos` (nixpkgs' pinned one) on `rocq/` and the new module: clean
  after `implicits` was added; `nix fmt` changed nothing.
- `gate` at every code commit, in the gate workspace
  `../linlog-step31a-gate` moved to each with `jj new`, from the transient
  unit `linlog-31a-gates` (`MemoryMax=8G`, `AllowedCPUs=8-11`,
  `CARGO_BUILD_JOBS=4`, `RUST_TEST_THREADS=4`; each commit's devshell built
  under the lock first, the gate itself outside it): eae340fd 07:41:01 to
  07:45:29, 889c1cc5 to 07:47:14, 31f48eff to 07:53:07, 2c7e88f6 to
  07:54:52, each ending in `gate: passed` with exit 0 (clippy, every test,
  both `cargo hack` runs and the ratchet; no Rust file changed in this
  session, so the later runs reuse the first build). Before that, one gate
  on the working copy the first three commits were split from (unit
  `linlog-31a-gate`, 07:27:52 to before 07:33:42, `gate: passed`, the
  ratchet's largest rise +0.87 % on `batch-families`, within every
  ceiling). The logs are the session's scratchpad's `gates.log` and
  `gate.log`.
- `nix flake check` under `flock /tmp/linlog-machine.lock`: a first run at
  31f48eff failed in `conventions` alone, because the `.v` glob it had
  gained reached the exporter's NanoYalla snapshots under
  `core/tests/snapshots/`, generated files without a header; the glob now
  excludes them (2c7e88f6), and the rerun at that commit passed ("all
  checks passed!", exit 0, 07:37), `rocq-linlog` rebuilt with the review
  commit's sources; the run at the session's last commit, the report's,
  is below the checklist's evidence.
- A scratch translator (`translate.py`, in the session's scratchpad, not
  committed) wrote `tests/FromLinlog.v` from `linlog prove --format json`:
  occurrences numbered in DFS preorder as `core-forest.md` says, nodes
  mapped one to one, shared subproofs unfolded. It is the sketch of
  31b's writer.

### The reviewer's findings

One reviewer, Opus 5.5 at `high`, in a fresh context, read `Formulas.v`,
`Calculus.v`, `Proofs.v`, `Check.v` and the statements of `Soundness.v`
against Girard's and Troelstra's one-sided cut-free CLL, NanoYalla's
`nanoll.v` and the Rust checker (`oracle.rs`, `core-proofs.md`). Beyond
reading, it compiled two probes of its own in the capped scope: one
pastes NanoYalla's kernel into a module and proves, with atoms as `nat`,
`N.ll l -> ll classical (map tr l)` and `ll classical l -> inhabited
(N.ll (map rt l))`, both closed under the global context, so `ll
classical` derives exactly what NanoYalla's kernel derives; the other
checks that `mix classical = true` and `weakening classical = true` are
uninhabited, that a bogus atom equality cannot be passed to
`soundness_classical`, and that the checker rejects a `Copy` with no
`Quest` of its formula below it in two shapes and accepts a `Quest` over
two equal `?A⊥` occurrences for the derivable `⊢ ?A⊥, ?A⊥, A`. Its
findings, each answered:

1. *Question A, no defect.* Every rule of `ll` is the textbook one; the
   axiom on atoms loses nothing (eta-expansion); `Permutation` exchange
   is complete (the probe derives it from NanoYalla's transpositions and
   back); Mix is binary only, which matches the Rust `Mix`, and worth a
   word in the `rules` doc; cut's absence is the stronger statement and
   the "cut-free" wording of `Calculus.v` must change when cut arrives.
   Answer: the word on binary Mix is in the `rules` doc (the review
   commit); the wording is step 34's to change, noted there.
2. *Question B, no defect.* The twelve constructors are those of Yalla
   and NanoYalla, `dual` swaps the right pairs and is proved an
   involution, `formula_eqb_spec` reflects equality; `ll` never uses
   `dual`, so `dual` is outside what a certificate's statement trusts.
   Answer: agreed, nothing to do.
3. *Question C, no defect: the statement is right.* The conclusion `ll R
   sequent` does not mention `atom_eqb`, so the reflection hypothesis can
   only be met by a real decision procedure (the probe shows `fun _ _ =>
   true` with `String.eqb_spec` does not typecheck); `ll` in `Prop` is
   "a derivation exists"; `Permutation` is the multiset reading;
   `classical` is a transparent record; the sequent is matched exactly
   unless `⊤` absorbs, and the proof then uses the surplus as the
   absorbed context. Three notes: `vm_compute` leaves a cast the kernel
   checks with its virtual machine, so the VM joins the trusted base;
   a sequent without atoms needs `ll (atom := string)`, which the
   exporter must emit; a certificate of an intuitionistic proof states
   CLL derivability of the one-sided translation, which is weaker than
   the ILL claim. Answer: the VM note is in `Certificates.v`'s header
   and `rocq.md`; the other two are in `rocq.md` for 31b, whose exporter
   writes the form and whose two-sided statement removes the weakness.
4. *Question D, sound, and no acceptance of an underivable sequent.* The
   four departures from the Rust checker are covered by the closed
   theorem; in the other direction, every term the Rust checker accepts
   passes the Rocq checker after translation, by the invariant that the
   Rocq `gamma` equals the Rust zone's formulas when exact and is a
   sub-multiset under `⊤`, the Rocq `theta` is contained in the Rust one
   as a set of formulas, and every later test (`Bang`, `join_with`, the
   root) is monotone in that direction. The Rocq checker accepts strictly
   more terms (a `Par` that consumes an equal formula from another
   occurrence), each proving a derivable sequent. Answer: recorded here
   and in `rocq.md`; it is the fact 31b's exporter relies on.
5. *Should-fix, design, before the bridge:* `ll` is in `Prop` while
   NanoYalla's is in `Type`, and a `match` on `ll` into `Type` is refused,
   so a bridge `ll classical l -> nanoll.ll (tr l)` cannot be written as
   such; state it as `inhabited (…)`, or prove a `Type`-valued soundness
   from `check … = true`, or move `ll` to `Type` with a `Type`-valued
   permutation. Answer: `ll` stays in `Prop`, the textbook's reading and
   the only one the standard library's `Permutation` allows (it has no
   `Permutation_Type`); the bridge of 31c states `ll classical l <->
   inhabited (NanoYalla.ll (map tr l))`, which the reviewer's probe
   already proves in both directions, or a `Type`-valued soundness from
   `check` if a function is wanted. `rocq.md` says so.
6. *Should-fix, hygiene:* the `let*` notation was exported through
   `Require Export` to every certificate, with generic names like `take`,
   `diff` and `remove_one`, and could clash with another library's.
   Answer: the notation is `Local` now (the review commit); the names
   stay, and `rocq.md` says a development that mixes libraries can
   `Require` without `Import`.
7. *Note:* "each acting on the first formula of its conclusion" was not
   true of `ex_r` or `mix_r`. Answer: fixed in the review commit.
8. *Notes:* the axiom's `[covar a; var a]` is Yalla's and a textbook's
   `⊢ A⊥, A`; `ex_r`'s direction carries no meaning; `weakening`,
   `weakening_r` and `wk_r` are easy to confuse and the doc should say
   which is which; the dyadic readings in `Proofs.v` are correct. Answer:
   the sentence on `wk_r` and `weakening_r` is in the `rules` doc (the
   review commit); nothing else to do.

Verdict quoted: "`ll classical` is standard cut-free one-sided
propositional CLL with units and exponentials … `soundness_classical` is
the right statement for a certificate … Nothing is must-fix."


### What 31b inherits

- `soundness` already covers Mix and affine (`{| mix := true; weakening
  := false |}`, `{| mix := false; weakening := true |}`): their statements
  are instances, with a certificate each to add to `rocq/tests/`. The
  two-sided intuitionistic statement is the new work: an inductive of
  two-sided ILL (Yalla's `nanoill.v` is the reference), the one-succedent
  condition over the checker's states (the Rust `Reading`, `Succedents`
  and the three clauses of `core-proofs.md`), and a theorem from `check`
  under that condition to the two-sided inductive.
- The exporter's kernel: write the sequent as `list (formula string)`,
  the term from the proof (the translator above is the map), `Lemma
  NAME : ll classical [...]` and the three-tactic proof; `Mix` and
  `Weaken` nodes, refused for NanoYalla, are plain constructors here. The
  check `rocq-linlog` then compiles the exporter's snapshots as `rocq`
  compiles NanoYalla's.
- The rules file `.claude/rules/rocq.md` and this report say where a
  rule, a calculus and quantifiers plug in.

### Left open

- The term carries formulas, so a proof over a large formula repeats its
  subformulas at every node; the exporter can bind shared subformulas
  with `let` if a certificate's size matters. Not measured on a large
  real proof yet.
- The bound lemma of item 7, if anyone wants it stated.
- The package's `version` is read from the opam file and the opam file's
  `url` section is still the archive's to add at a release, as 30c said.

### Workspaces

`../linlog-step31a` (this session's, every commit above), made with `jj
workspace add ../linlog-step31a -r sonqsxku` after a first attempt hung
on the signing prompt and left a stale git worktree entry, removed by
hand; and `../linlog-step31a-gate`, the gate's, moved from code commit to
code commit with `jj new`. The main checkout was not touched.
