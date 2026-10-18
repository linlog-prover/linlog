# Step 31: a Rocq library of linlog's own

## The checklist

| item | state | evidence |
|---|---|---|
| Session 31a: the definitions, the checker and its soundness (session `step-31a`, 2026-10-18) | done (2026-10-18) | The library under `rocq/` (eae340fd): `Formulas`, `Calculus` (the one-sided calculus as a plain inductive, Mix and weakening as parameters), `Proofs` (the term, one constructor per `Node` variant), `Check` (the checker as a function), `Soundness` (`soundness` for every `rules` value and `soundness_classical`), no `Admitted`, no axiom, `Print Assumptions` of both theorems "Closed under the global context". The flake (889c1cc5): the package `rocq-linlog` built with `mkRocqDerivation` and the check `rocq-linlog`, which compiles the fourteen hand-written lemmas (nine accepted proofs, five rejections) and seven translated proofs of `rocq/tests/` (the largest 192 nodes, all in under a second) and requires the two assumption lines; both built under the machine lock. Documentation (da2a7e46): `.claude/rules/rocq.md`, CLAUDE.md, README, the changelog. One reviewer (Opus 5.5, `high`) read the inductive against the textbook calculus and NanoYalla's kernel: no must-fix, two should-fix (the `let*` notation made local; `ll` in `Prop` shapes 31c's bridge as `inhabited`), three notes, all answered and the wording fixed in 31f48eff; its probe proved `ll classical` and NanoYalla's `ll` derive the same sequents. Gate: passed at every code commit in the gate workspace `../linlog-step31a-gate` (eae340fd 07:41 to 07:45, 889c1cc5 to 07:47, 31f48eff to 07:53, 2c7e88f6 to 07:54, each `gate: passed`, exit 0), and once before on the working copy they were split from. `nix flake check` at the last commit: passed ("all checks passed!", exit 0) under the lock at 2c7e88f6, the last code commit, and again at the report commit. Every commit unsigned (`signing.behavior=drop`; the passphrase was not cached). |
| Session 31b: Mix, affine, the two-sided statement and the second kernel (session `step-31b`, 2026-10-18) | done (2026-10-18) | The library (a06b52e0, 18b99bb4, 61d949e1): `soundness_mix` and `soundness_affine` with certificates in `rocq/tests/Modes.v`; the two-sided calculus `ill` (`Intuitionistic.v`, `nanoill.v`'s rules) with its own term, checker and `soundness_ill` (`IntuitionisticCheck.v`), certificates in `rocq/tests/TwoSided.v`; five theorems "Closed under the global context", no `Admitted`, `admit` or axiom. The exporter (c7088b5c): `rocq::Kernel` behind `rocq::Options::kernel` and `--kernel`, chosen by the mode where none is named, the NanoYalla snapshots' diff empty, nine library snapshots in `core/tests/snapshots/linlog/` and six CLI certificates compiled by the `rocq-linlog` check, README and the changelog. One reviewer (Opus 5.5, `high`) of the two-sided inductive and the theorem's statement: no must-fix, one should-fix (the equality out of the trusted file) and four notes, all answered; its probes proved `ill intuitionistic` equivalent to `nanoill.v`'s `ill` and the checker complete. Gate: passed at all four code commits (`gate: passed`, ratchet's largest rise +0.87 % on `batch-families`) in a capped unit on cores 8 to 11, 09:05 to 09:17. `nix flake check` under the lock: passed ("all checks passed!", exit 0) at the documentation commit after a first run's `export` failure was fixed at its cause; the run at the report commit is in the message to `planning`. Every commit signed. |
| Session 31c: the bridges (session `step-31c`, 2026-10-18) | done (2026-10-18) | The library (27bcc6b2): `Mix.v` with `mix_to_ll` (a derivation in any `ll R` gives one of `?(⊥⊗⊥), Γ` without Mix, weakening kept, no cut) and `mix_to_classical`. The bridges (0bca3835): `rocq/bridges/NanoYallaBridge.v` (`ll_iff_nanoll`, `nanoll_iff_ll`) and `NanoIllBridge.v` (`ill_iff_nanoill`, `nanoill_iff_ill`), importing NanoYalla's `nanoll.v` and Yalla's `microyalla/nanoill.v` unchanged from the flake inputs `nanoyalla` and the new `yalla` (pinned to commit a1384376), never copied; the check `rocq-bridges` compiles them; all six theorems "Closed under the global context", no `Admitted`, `admit` or axiom under `rocq/`. Documentation (e91c361d): `rocq.md`, `flake.md`, CLAUDE.md, README, the changelog. One reviewer (Opus 5.5, `high`) of the statements: no must-fix, no should-fix, six notes, all answered (the headers fixed in 01632049). Gate: passed at all three code commits (10:43 to 10:51, exit 0) in `../linlog-step31c-gate` on cores 8 to 11. `nix flake check` under the lock at the last commit: in the message to `planning`. Every commit signed. |
| Session 31d: certified refutations (session `step-31d`, 2026-10-18) | done (2026-10-18) | The classical refuter in the library, on by default (uzsqzswr): after every search that does not prove its goal, an assignment falsifying the classical reading makes an unknown verdict unprovable and an exhausted one certified (`Refutation::Classical`), bounded by `Options::classical_work` (`DEFAULT_CLASSICAL_WORK` 2²⁶, `--classical-work`); `Disproof::check` for the classical, unbalanced and equation refutations with `RefutationFault`; wire level 2 for the classical kind alone. The command's default (xpywqoqt, the behaviour lock's and README's verdict changes in a commit of their own). The Rocq library (vyqupqzq): `Refutations.v`, `ll_tautology`, `balance`, `count_equation`, `ill_ll`, `ill_tautology` and six refutation theorems, eighteen theorems of `Assumptions.v` "Closed under the global context", no `Admitted`, `admit` or axiom. The exporter (rrlvvovq): `rocq::write_disproof` and `rocq::write_not_valid`, `--format rocq` on unprovable and not valid verdicts, ten snapshots and six command-written certificates compiled. One reviewer (Opus 5.5, `high`): no must-fix, one should-fix (fixed in qlsnqsyy), four notes answered. Target sets (wkoltnqm): no verdict or counter of a decided row changed, five undecided rows refuted and checked; held-out sets at 10 s: 721 of 850 decided before and after, no verdict changed, certified refutations 15 to 90 of 153; no problem with a checked proof refuted (3 854 probed). Gate passed at every code commit (11:32 to 12:33). The flake's `rocq-linlog`, `rocq` and `rocq-bridges` checks built under the lock (12:23 to 12:24, eighteen assumption lines). `nix flake check` under the lock: passed at wokltmsn ("all checks passed!", exit 0, 12:30 to 12:34) after a first run's `doc` failure was fixed at its cause; the run at the report commit is in the message to `planning`. |

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

## Session 31b: Mix, affine, the two-sided statement and the second kernel

**Outcome.** Every mode now has a statement and a certificate. In the
library, `soundness_mix` and `soundness_affine` are the instances of
`soundness` for `{| mix := true; weakening := false |}` and `{| mix :=
false; weakening := true |}`, each with certificates in `rocq/tests/Modes.v`
(the module of hand-written Mix and affine certificates). The two-sided
calculus of intuitionistic linear logic and its affine variant is the
plain inductive `ill R Γ C` of `rocq/theories/Intuitionistic.v` (the new
trusted file), over `iformula` with the constructors of Yalla's
`nanoill.v`. The theorem `soundness_ill` proves `ill R Γ C` from `icheck
atom_eqb R p Γ C = true`, for a two-sided term `p` of
`rocq/theories/IntuitionisticCheck.v` (the term, its checker and the
proof). Every theorem's `Print Assumptions` answers `Closed under the
global context` (five of them, `rocq/tests/Assumptions.v`), and no file
under `rocq/` has an `Admitted`, an `admit` or an axiom. The exporter
has a second kernel, linlog's library, behind `rocq::Options::kernel`
(`rocq::Kernel`, the JSON key `styles.rocq.kernel`, the command's
`--kernel`). Where none is named, the mode chooses: a classical proof is
written for NanoYalla byte for byte as before, and a proof of Mix,
affine or intuitionistic logic for the library, one-sided in the mode's
calculus or, for an intuitionistic derivation, as the two-sided
statement `ill intuitionistic Γ C` or `ill affine Γ C`. The NanoYalla
snapshots are unchanged; the library's are new under
`core/tests/snapshots/linlog/`, and the flake's `rocq-linlog` check
compiles them with six certificates the command writes. README's usage
shows `--kernel` with real output, and the changelog's `[Unreleased]`
names the additions and the changed default.

### Decisions

- **The two-sided statement has a term and a checker of its own
  (departs from the design).** The brief asked for a theorem from the
  one-sided `check` under the one-succedent condition over the checker's
  states. That condition counts goals, and a goal is a property of a
  formula's side of `⊢`. A one-sided formula built from `⊤` and `0` alone
  does not determine that side: `top` is `⊤` on the right and `0` on the
  left, so `⊢ ⊤, ⊤` reads as `0 ⊢ ⊤` or as `⊤ ⊢ 0`. The Rocq term names
  formulas, not occurrences (31a's decision), and the one-sided `Top`
  names no context, so a bottom-up pass over formula states cannot tell
  which formula is the goal where one stems from a `⊤` leaf. The Rust
  checker can, because its occurrences carry the reading's positions.
  Three ways out were weighed:
  - carrying a pending sign through the pass, which grows with every
    `⊤` and makes the proof combinatorial;
  - a top-down pass with backtracking at each ambiguous choice;
  - the one taken: a two-sided term, `iproof`, whose nodes name the rule
    with its side, the principal formula's immediate subformulas as
    intuitionistic formulas, and the context of `ITopR` and `IZeroL` (and
    the goal of `IZeroL`).

  `iderive` then computes every node's sequent exactly, and a sequent has
  its one conclusion by its type. That is the one-succedent condition
  made structural: R1 (one goal, or none under `⊤`) is the type, R2 is
  that a premise's goal slot is either filled or absorbed, and R3 is that
  `IWeaken` weakens a hypothesis. The soundness proof is 60 lines,
  because each node is one rule up to a permutation. The reviewer judged
  the argument right and the design the more auditable one. Its probe
  proved that every derivation of `ill` has a term the checker accepts,
  so nothing is lost. The cost is a certificate somewhat larger than the
  one-sided term (`⊤R` and `0L` write their contexts, contraction is
  explicit).
- **The two-sided calculus follows `nanoill.v`** rule for rule, with
  exchange as one `Permutation` rule as in the one-sided calculus, in
  `Prop`, and `weakening_ilr` (any hypothesis, never the conclusion)
  under `iweakening R = true` for affine logic. `irules` is a record of
  its own with `intuitionistic` and `affine` as named values, so that a
  statement reads `ill intuitionistic [...] C`. The one-sided `rules`
  record would have carried a meaningless `mix` and the name `classical`.
- **The Mix and affine statements name the calculus as a record
  literal**, `ll {| mix := true; weakening := false |} Γ`, rather than
  new named values, so the trusted `Calculus.v` is unchanged. A proof
  with both rules uses `soundness` with `{| mix := true; weakening := true
  |}`.
- **Which kernel the mode chooses, and where the brief's reading did not
  fit the code.** The brief read item 6 as "Mix, affine and
  intuitionistic proofs, which NanoYalla refuses, go to linlog's
  kernel". The code shows that NanoYalla does not refuse intuitionistic
  proofs: it certifies them as the one-sided classical proof behind them
  (the snapshots `ill.v` and `labels_ill.v`, and the `rocq` check's
  `cli_ill.v`). The mode still decides, as item 6 says, so an
  intuitionistic proof with no kernel named now gets the two-sided
  statement, the stronger one. That changes the command's default output
  for `-i --format rocq`, which the changelog lists under "Changed". The
  NanoYalla export is otherwise exactly as it was:
  - `--kernel nanoyalla` gives the old certificate;
  - the NanoYalla snapshots, the `rocq` check's `cli_ill.v` and the CLI
    tests that pin NanoYalla's refusals and scripts name that kernel;
  - a classical proof is written for it by default, byte for byte (the
    snapshots' diff is empty).

  A recorded mode that is classical but whose derivation uses Mix or
  affine weakening (a proof file checked in another mode than it claims)
  goes to the library, and a proof that records no mode goes there when
  its derivation is two-sided or uses those rules.
- **The exporter writes from the derivation, not the proof term.**
  `rocq::write` takes a `Derivation`, which keeps the API, the stop
  callback and the compact-run refusal as they are. A standard
  derivation has explicit contractions and promotions over `?`
  contexts, while the library's checker is dyadic, so the one-sided
  writer moves each `?` formula into the unrestricted zone with a
  `Quest` where it first appears (the root's members, and a premise's
  members that are children of the rule's principal formula). Then a
  dereliction is `Copy`, a contraction or `?` weakening writes nothing,
  and a promotion finds its context in the zone. The term is not the
  Rust proof's own, which the checker does not need: 31a's review showed
  that the Rocq checker accepts every term that keeps its zones
  consistent. Every snapshot and CLI certificate compiled, among them
  `labels.v`, which uses every one-sided rule. The derivation now records
  the proof's mode (`Derivation::mode`, crate-private), which is what
  "chosen by the mode" reads.
- **The statement's calculus is what the mode allows and what the
  derivation uses**, so a certificate never claims a calculus the proof
  exceeds.
- **Atoms are Rocq strings** with `"` doubled and `(atom := string)`
  always written, so no atom is renamed and a sequent without atoms needs
  nothing special. The preludes are `rocq::LINLOG_PRELUDE` (one-sided)
  and `rocq::LINLOG_TWO_SIDED_PRELUDE`; `rocq::LINLOG` is the library's
  version, asserted equal to the opam file's at evaluation.
- **Layout.** The exporter moved to `core/src/export/rocq/mod.rs` with the
  new `linlog.rs` beside it, since the workspace's lints require
  `mod.rs` files. The library's snapshots are a directory of their own so
  that the `rocq` check's `*.v` glob keeps to NanoYalla's. The
  conventions check's exclusion became `core/tests/snapshots/**/*.v`,
  because a ripgrep `*` does not cross `/`. The `rocq-linlog` check
  requires one `Closed under the global context` per `Print Assumptions`
  line of `Assumptions.v` instead of a fixed two.

### Which goal a certificate states

The two-sided lemma states the reading's goal, the root the reading
calls the goal: the one that can only be a goal, else the last root by
id (`plan/later.md`, "Follow-ups: intuitionistic mode"). Its hypotheses
are the other roots, each written by its position, so a `top` in input
position is `izero` and in output position `itop`. For `0, top |- top`,
`top, 0 |- top`, `0 |- top` and `0, top |- 0` the stated sequents were
the written ones (`[izero; itop] itop`, `[itop; izero] itop`, `[izero]
itop`, `[izero; itop] izero`), and the flake's check compiles the first.
Whatever goal the reading picks, the lemma states a sequent that `ill`
derives. Where the reading's goal differs from the written one, the
lemma says which sequent was proved: every reading of a provable
sequent is provable, as step 8's brute force found.

### The options, per front end (D15, D16)

`rocq::Options` gains `kernel: Option<Kernel>`, default `None` (the
mode's choice), `with_kernel`, and serde as `"nanoyalla"` or `"linlog"`.
The command sets it with `--kernel nanoyalla|linlog` or `--style
rocq.kernel=linlog`. The web front end and any wrapper hold it as the
JSON key `styles.rocq.kernel`. `prelude: None` picks each kernel's own
prelude, and `lemma` is unchanged. A certificate of ordinary logic needs
no kernel and ignores the field.

### Quantifiers (D17)

The two-sided development adds no binder either. A first-order ILL is a
second development beside it, as item 5 has it for the one-sided one:
its own `iformula` with quantifiers over terms, `ill` with the two rules
on each side and their eigenvariable conditions, and `iproof` with the
witness and the eigenvariable named. Because `iproof` writes the
contexts of `⊤R` and `0L`, the eigenvariable check of `∀R` and `∃L`
sees every formula of the sequent, which avoids the difficulty that the
absorbing flag poses for the one-sided checker (31a's item 5).

### What is verified

- The library and every certificate under `rocq/tests/` (`Certificates`,
  `FromLinlog`, `Modes`, `TwoSided`) compile with the pinned Rocq 9.1.1
  and standard library 9.0.0 on cores 8 to 11 in capped scopes.
  `Assumptions.v` prints `Closed under the global context` five times
  (`soundness`, `soundness_classical`, `soundness_mix`,
  `soundness_affine`, `soundness_ill`).
- The nine snapshots of `core/tests/snapshots/linlog/` and the six
  certificates the `rocq-linlog` check writes with the command, compiled
  by hand against the library, were all accepted silently.
- `cargo clippy --workspace --all-targets -- --deny warnings`: clean.
  `cargo test --workspace`: every test passed after the behaviour lock's
  four help outputs were blessed. The diff of the NanoYalla snapshots
  under `core/tests/snapshots/*.v` is empty.
- `gate` at every code commit, in the workspace `../linlog-step31b-gate`,
  as the transient unit `linlog-31b-gates` (`MemoryMax=8G`,
  `AllowedCPUs=8-11`, `CARGO_BUILD_JOBS=4`, `RUST_TEST_THREADS=4`). Each
  log ends `gate: passed`: clippy, every test with the behaviour lock,
  both `cargo hack` runs, and the ratchet, whose largest rise is +0.87 %
  on `batch-families`, as at 31a, within every ceiling. The runs were at
  a06b52e0, 18b99bb4 and 61d949e1 (09:05 to 09:17), and at the kernel
  commit as f284e3c1 (09:05) and again at its final id c7088b5c (09:22
  to 09:25), after the export check's fix below was folded into it.
  - A first run, at the kernel commit before its test fix, failed in
    `compact_runs`. That test pinned the NanoYalla script of an
    intuitionistic proof, which by default now goes to the library, and
    the behaviour lock's help outputs needed blessing. I had committed
    without running the CLI's tests, and the gate caught it. The test
    now pins the same script with `--kernel nanoyalla` and the library's
    three `IWk` by default. It and the lock's four help outputs are in
    the kernel commit, which says why.
- `nix flake check` under `flock /tmp/linlog-machine.lock`:
  - A first run, at the documentation commit, failed in `export`: that
    check copies `core/tests/snapshots/*` and refused the new `linlog/`
    directory. Its fileset now leaves the directory out, which belongs
    to `rocq-linlog` (folded into the kernel commit, which says so).
  - The rerun passed ("all checks passed!", exit 0, finished by 09:25) with
    `check-rocq`, `check-rocq-linlog` (its log the five assumption
    lines), `check-export`, `check-conventions` and `check-typos`
    rebuilt.
  - The run at the session's last commit, this report's, is in the
    final message to `planning`.

### The reviewer's findings

One reviewer, Opus 5.5 at `high`, in a fresh context, read
`Intuitionistic.v`, `IntuitionisticCheck.v` and `TwoSided.v` against
Girard's and Troelstra's calculi, Yalla's `nanoill.v` and linlog's
reading. It compiled two probes in a capped scope, both closed under the
global context:
- `equiv_nanoill`: `ill intuitionistic G C <-> inhabited (nanoill.ill
  (map tr G) (tr C))`, with atoms as `nat`, so the calculus derives
  exactly what `nanoill.v` derives;
- `icheck_complete`: every derivation of `ill R` has a term `icheck`
  accepts, in both calculi.

The probes are kept outside the repository, in
`/tmp/linlog-handoff/31b-review/` (`Equiv.v`, `Complete.v`), for 31c,
whose bridge to `nanoill.v` the first one already is. Its findings, each
answered:

1. *Should-fix.* The trusted file defined and proved the formula
   equality (`iformula_eqb`, its spec, `iformula_eq_dec`), which no
   statement uses, against its own sentence "nothing is derived here".
   Answer: moved to `IntuitionisticCheck.v` (61d949e1). `Intuitionistic.v`
   now holds `iformula`, `irules` and `ill` alone.
2. *Note.* The calculus has no `⊥` and no empty succedent, unlike
   Troelstra's ILL, and leaves unsaid that the axiom is on atoms and
   that there is no cut. Answer: the file's header says all three now
   (the same commit).
3. *Note.* The affine variant weakens hypotheses only, which is standard
   for one conclusion and is linlog's R3, worth a sentence. Answer: the
   `irules` doc says "never of the conclusion" (the same commit).
4. *Note.* "The one-sided terms cannot carry this statement" said more
   than the argument shows. With sides annotated and a translation proof
   they could. Answer: the header now says they "do not determine" it.
5. *Note.* The calculus is a parameter of the theorem, so a reader must
   check that a linear claim says `ill intuitionistic`, not `ill
   affine`. Answer: the exporter writes the calculus from the mode and
   the rules used, and the rules file says so. Nothing else to do.

Verdict quoted: "`ill intuitionistic` is the cut-free single-conclusion
sequent calculus of propositional ILL with 1, ⊗, ⊸, ⊤, &, 0, ⊕ and !,
without ⊥ … No rule is missing and none is too strong." On the
statement: "The conclusion of `soundness_ill` mentions only `ill`,
`iformula` and `irules` … `vm_compute; reflexivity` establishes exactly
`ill R Γ C`, with Γ read as a multiset and C exact." On the departure:
"The argument is right … The separate two-sided term avoids that and is
the more auditable choice. Nothing is lost."

### What 31c inherits

- The bridge to `nanoill.v` is half done: the reviewer's `Equiv.v`
  (`/tmp/linlog-handoff/31b-review/`) proves `ill intuitionistic G C <->
  inhabited (nanoill.ill (map tr G) (tr C))` with a pasted copy of the
  kernel and atoms as `nat`; 31c's module imports the pinned file
  instead. The one-sided bridge to NanoYalla's `ll` is 31a's reviewer's
  probe, in the same form.
- Mix's reduction to `?(⊥⊗⊥), Γ` is untouched; `soundness_mix` gives the
  Mix derivation it starts from.
- The exporter's library certificates are in `core/tests/snapshots/linlog/`
  and the `rocq-linlog` check compiles them; a bridge module that the
  check should compile goes into `rocq/` with the pinned kernels as
  inputs, which the check does not have yet.

### Left open

- The size of a library certificate on a large real proof is not
  measured. Formulas repeat at every node, and the two-sided `⊤R`/`0L`
  write their contexts; binding shared subformulas with `let` is the
  remedy if it matters.
- `vm_compute` puts Rocq's virtual machine beside the kernel in what a
  certificate trusts (31a's note); `reflexivity` alone avoids it at a
  cost in time, and the exporter has no option for it.
- The prompt's "What step 30 left you" names a `Kernel` type that did not
  exist; it is new here.

### Workspaces

`../linlog-step31b` (this session's, every commit above), made with `jj
workspace add ../linlog-step31b -r znksttkk`, after a first attempt
failed on a signing timeout and left a half-made directory and its git
worktree entry, both removed by hand. `../linlog-step31b-gate` holds the
gates, moved from commit to commit with `jj new`. The main checkout was
not touched.

## Session 31c: the bridges

**Outcome.** The Rocq library is tied to the two kernels it stands
beside, and Mix is reduced to a hypothesis. In the library,
`rocq/theories/Mix.v` (in the package, importing `Calculus` alone)
proves `mix_to_ll`: for every `rules` value `R`, a derivation of `⊢ Γ`
in `ll R` gives one of `⊢ ?(⊥⊗⊥), Γ` in `ll {| mix := false; weakening
:= weakening R |}`, without cut since the calculus has none; its
instance `mix_to_classical` goes from `ll {| mix := true; weakening :=
false |} Γ` to `ll classical (wn (tens bot bot) :: Γ)`. Beside the
library, in `rocq/bridges/` (not part of the package), two modules
import the upstream kernels unchanged and prove the equivalences:
`NanoYallaBridge.v` proves `ll_iff_nanoll : ll classical l <->
inhabited (nanoll.ll (map to_nanoyalla l))` over NanoYalla's `nanoll.v`,
and `NanoIllBridge.v` proves `ill_iff_nanoill : ill intuitionistic G C
<-> inhabited (nanoill.ill (map to_nanoill G) (to_nanoill C))` over
Yalla's `microyalla/nanoill.v`, each with its converse form over the
upstream sequents (`nanoll_iff_ll`, `nanoill_iff_ill`). The two files
are fetched by the flake, NanoYalla's from the existing input
`nanoyalla` and `nanoill.v` from the new input `yalla` (Yalla pinned to
commit `a1384376`, 2026-04-13), and no copy of either is in the
repository. The new check `rocq-bridges` (`modules/rocq-bridges.nix`)
compiles both upstream files unchanged, then the bridges against them
and the installed library, and requires `Print Assumptions` of the four
bridge theorems to answer `Closed under the global context`; the
library's own `tests/Assumptions.v` now asks about seven theorems,
`mix_to_ll` and `mix_to_classical` among them, all closed. No file under
`rocq/` has an `Admitted`, an `admit` or an axiom.

### Decisions

- **Mix's reduction is in the library, the bridges beside it.**
  `Mix.v` needs nothing but `Calculus.v`, so it is installed with the
  library and checked by `rocq-linlog`. The bridges need the LGPL files,
  which the brief keeps out of the repository and out of anything a
  certificate needs, so they live in `rocq/bridges/`, which neither the
  package nor `rocq-linlog` reads, and have a check of their own. A
  separate check, rather than more lines in `rocq-linlog`, keeps the
  library's check independent of the two inputs (one aspect per file).
- **The statement of `mix_to_ll` is general in `R`, with weakening
  kept.** One induction covers both Mix calculi, the plain one and Mix
  with affine weakening, at no extra cost, and the instance the brief
  names is the corollary. The statements write `wn (tens bot bot)` and
  the record literal, not definitions of `Mix.v`, so a reader checks
  them against `Calculus.v` alone. The proof follows the argument of
  Yalla's `mix_to_ll` for the binary case, read from the pinned source
  and written afresh for this calculus: `?(⊥⊗⊥)` travels up the
  derivation, an axiom, `1` or `⊤` weakens it in, a two-premise rule
  contracts the two copies, a promotion keeps it in its `?` context, and
  a Mix of `⊢ Γ` and `⊢ Δ` becomes the dereliction of the `⊗` of `⊢ ⊥,
  ?(⊥⊗⊥), Γ` and `⊢ ⊥, ?(⊥⊗⊥), Δ`, contracted twice. Only this
  direction is stated, as the brief asks; the converse (from `⊢
  ?(⊥⊗⊥), Γ` without Mix to `⊢ Γ` with it) is proved in Yalla with cut
  (`ll_to_mix2_cut`), and a cut-free proof here would need `⊥`'s
  inversion and a generalisation over the copies of `?(⊥⊗⊥)`; it is not
  needed by any certificate and is left open.
- **The bridges state the `inhabited` form and both directions of the
  translation.** The upstream inductives are in `Type` and `ll` and `ill`
  in `Prop` (31a's decision), so each equivalence says that an upstream
  derivation exists. Each bridge proves its translations inverse
  (`of_to_…`, `to_of_…`) and states the equivalence twice, over the
  library's sequents and over the upstream ones, so a reader who starts
  from either calculus finds the statement in its own terms. Atoms are
  `nat`, the upstream `Atom` and `IAtom`, which instantiates the library's
  abstract atom type.
- **The upstream files are imported under their own projects' logical
  paths**: `NanoYalla` (its `_CoqProject` maps `.` to it, so the module
  is `NanoYalla.nanoll`) and `Yalla.MicroYalla` (Yalla's `_RocqProject`
  maps `microyalla` to it). `Require` without `Import`, every upstream
  name qualified, since both define `map` and their own formulas. The
  bridges themselves are compiled as `LinlogBridges`.
- **Yalla is pinned to a commit, not a tag.** Its last tag on GitHub,
  `v2.0.7`, predates `nanoill.v` (its `microyalla/` holds `ll.v` and
  `ill.v` only), and the 2.1.0 its release notes describe is not tagged.
  The input fetches the whole repository (1.1 MB), from which the check
  takes the one file; `ll_fragments.v` was read from the same store path.
  `nanoill.v` at that commit is byte-identical to the copy 31b's
  reviewer pasted (`diff` empty), so the reviewer's proof carried over.

### What is verified

- `rocq compile` of the library with `Mix.v` and of every bridge with
  the pinned Rocq 9.1.1 and standard library 9.0.0, on cores 8 to 11 in
  capped scopes, against scratch copies of `nanoll.v` and `nanoill.v`
  taken from the inputs' store paths (outside the repository).
  `tests/Assumptions.v` prints `Closed under the global context` seven
  times, `bridges/Assumptions.v` four times.
- `grep -rnE 'Admitted|\badmit\b|Axiom' rocq/` finds nothing, and no
  file named `nanoll*` or `nanoill*` is under `rocq/`.
- `nix build .#checks.x86_64-linux.rocq-bridges
  .#checks.x86_64-linux.rocq-linlog` under `flock
  /tmp/linlog-machine.lock`: both built, their logs the four and the
  seven assumption lines, the package rebuilt with `Mix.vo` installed.
- `gate` at every code commit, in the workspace `../linlog-step31c-gate`
  moved to each with `jj new`, as the transient units `linlog-31c-gates`
  and `linlog-31c-gates2` (`MemoryMax=8G`, `AllowedCPUs=8-11`,
  `CARGO_BUILD_JOBS=4`, `RUST_TEST_THREADS=4`; each commit's devshell
  built under the lock first): 27bcc6b2 10:43:08 to 10:47:49, 0bca3835 to
  10:49:51, 01632049 10:49:58 to 10:51:44, each `gate: passed` with exit
  0. No Rust file changed; the ratchet's largest rise was +1.22 % on
  `read-text` and `read-lltp`, within every ceiling. The documentation
  commit e91c361d and the report's touch no code.
- `nix flake check` under the lock at the session's last commit, this
  report's: in the final message to `planning`.

### The reviewer's findings

One reviewer, Opus 5.5 at `high`, in a fresh context, read the
statements of `NanoYallaBridge.v`, `NanoIllBridge.v` and `Mix.v` against
the pinned `nanoll.v`, `nanoill.v` and Yalla's `ll_fragments.v`, and the
check `rocq-bridges`. It confirmed the inputs: `nix hash path` of both
store paths equals the `narHash` in `flake.lock`, the compiled copies are
byte-identical to the pinned files, no `nanoll*` or `nanoill*` file is
under `rocq/`, and the logical paths the check binds are the ones the
bridges import, with nothing else in `ROCQPATH` that could shadow them.
Its probe (outside the repository) checked by `reflexivity` that every
constructor maps to its namesake with its arguments in order, derived
`ll classical [top; var 3]` through `ll_iff_nanoll` from `nanoll.top_r`,
used the affine instance of `mix_to_ll` and derived `⊢ ?(⊥⊗⊥), 1, 1`
classically through `mix_to_classical` from a Mix of `⊢ 1` and `⊢ 1`;
`Print Assumptions` of all six theorems printed `Closed under the global
context`. It answered its four questions yes: the translations are the
namesake bijections (A); each equivalence says that the same sequents
are derivable, over the upstream inductives as they stand (B);
`mix_to_ll` is the standard cut-free reduction, made stronger by keeping
weakening, and its converse is neither claimed nor implied (C); no
statement uses a definition a reader cannot check, the four translation
functions excepted, which are a dozen lines each (D). No must-fix and no
should-fix; six notes, each answered:

1. *N1, the atom type.* The bridges hold at `nat` only; the result
   transfers to any atom type by an injective renaming, which is not
   proved. Answer: both headers now say that the statements are about
   `formula nat` and `iformula nat`, the upstream atom types
   (01632049). A bridge at another atom type has no upstream kernel to
   meet, so the renaming lemma is not added.
2. *N2, derivability rather than derivations.* Intended, and the headers
   now say so in those words (the same commit).
3. *N3, the equivalences do not pin the translations*: swapping `var`
   and `covar` everywhere, or the arguments of `aplus`, preserves
   derivability, so a mis-mapped translation would still prove; the
   reviewer suggested `Example`s. Answer: the translations are read, as
   the reviewer read them, and its probe checked them; an `Example` per
   constructor would restate the twelve arms of the definition beside it,
   so none is added. The report and `rocq.md` say that the translations
   are part of what a reader checks.
4. *N4, the converse of `mix_to_ll`*, which needs cut admissibility for
   LL with Mix or an inversion argument. Answer: not claimed; recorded
   under "Left open".
5. *N5, `Type` or `Set`.* The upstream inductives are declared in
   `Type` and Rocq lowers them to `Set`. Answer: the headers say so now
   (01632049).
6. *N6, the check's `rocq compile … | tee log`* loses the exit status;
   harmless, since a failure prints and `[ -s log ]` catches any output.
   Answer: the pattern is `rocq-linlog`'s and `rocq`'s; nothing to do.

Verdict quoted: "The four bridge iffs state, over the pinned upstream
inductives compiled unchanged, that linlog's `ll classical` and `ill
intuitionistic` derive exactly the sequents NanoYalla's `nanoll.ll` and
Yalla's `nanoill.ill` derive (as derivability at atom type `nat`),
through translations that are the obvious namesake bijections.
`mix_to_ll` is the standard cut-free reduction of binary Mix to
`?(⊥⊗⊥)`, made stronger, not weaker, by keeping weakening, and no
must-fix or should-fix issue was found."

### What 31d inherits

- The pattern of an invariant lemma over the calculus: `mix_to_ll` is an
  induction over `ll R` for every `R` at once, with one case per
  constructor and the permutation steps written with the standard
  library's `perm_swap`, `perm_skip` and `Permutation_middle` (atoms are
  abstract here, so `Soundness.v`'s `perm` tactic, which needs a decided
  atom equality and is local to its section, is not available). Item 8's
  refutation lemmas (the balance of an atom's literals, the count
  equation, the classical reading) are inductions of the same shape,
  stated as `~ ll R Γ` over the inductive.
- Refutations carry over to the kernels: `nanoll_iff_ll` turns a proof
  of `~ ll classical (map of_nanoyalla l)` into one that NanoYalla's
  kernel derives no `l`, and `nanoill_iff_ill` likewise for `nanoill.v`,
  if a certificate for those kernels is wanted.
- With Mix, `mix_to_classical` is the contrapositive route: a classical
  refutation of `⊢ ?(⊥⊗⊥), Γ` refutes `⊢ Γ` with Mix. The classical
  reading needs no such detour, since every rule of every `rules` value
  keeps tautologies; `?(⊥⊗⊥)` reads as false, so the route adds nothing
  there.
- The check `rocq-bridges` compiles every `.v` under `rocq/bridges/`, so
  a module added there is checked without a change to the flake.

### Left open

- The converse of `mix_to_ll`, from `⊢ ?(⊥⊗⊥), Γ` without Mix to `⊢ Γ`
  with it, cut-free: an inversion of `⊥` and an induction over the copies
  of `?(⊥⊗⊥)`; not needed by any certificate.
- Bridges at atom types other than `nat`, by an injective renaming of
  atoms; no upstream kernel uses another.
- Bridges for the Mix and affine calculi and for the two-sided affine
  one: neither upstream file states those calculi (Yalla proper does,
  over OLlibs, which the brief keeps out).

### Workspaces

`../linlog-step31c` (this session's, every commit above), made with `jj
workspace add ../linlog-step31c -r tumwxxml`, and
`../linlog-step31c-gate`, the gates', moved from code commit to code
commit with `jj new`. The main checkout was not touched.

## Session 31d: certified refutations

**Outcome.** An unprovable verdict can now be certified, and a search
that did not decide is answered where the sequent is false read
classically.

In the library, after every search that does not prove its goal, a
classical refuter (`core/src/search/classical.rs`) looks for an
assignment of the atoms under which the goal's classical reading is
false. The reading erases `!` and `?`, reads `⊗` and `&` as "and", `⅋`
and `⊕` as "or", `1` and `⊤` as true and `⊥` and `0` as false. Where it
finds one:

- an `Unknown` search becomes `Unprovable` with the new refutation
  `Refutation::Classical { assignment }`;
- an exhausted search whose counts give no refutation gets the
  assignment as its certificate.

This is on by default, bounded by work: `search::Options::classical_work`
(`DEFAULT_CLASSICAL_WORK`, 2²⁶ terms evaluated, zero for none), the
command's and the harness's `--classical-work N`. No decided verdict and
no counter of a search changes.

`Disproof::check` verifies the classical, `Unbalanced` and `Equation`
refutations against the sequent, the goal and the mode without the
search's code, and rejects one that does not hold
(`Error::Refutation` with `search::RefutationFault`).

The Rocq library proves the three invariants once in
`rocq/theories/Refutations.v`:

- every derivable sequent of every `ll R` is a tautology classically;
- without weakening an atom's excess interval contains zero;
- without weakening a multiplicative sequent with units meets the count
  equation (`≥` with Mix);
- the two-sided calculus embeds in the one-sided one (`ill_ll`).

Each gives `~ ll R Γ` or `~ ill R Γ C` from a Boolean test that
`vm_compute` decides. The exporter writes such a certificate for every
refutation `check` accepts (`rocq::write_disproof`), and an ordinary
sequent that is not valid gets `~ (forall a b : Prop, F)` by
instantiating its atoms (`rocq::write_not_valid`). The command writes
both for `--format rocq`.

All eighteen theorems of `rocq/tests/Assumptions.v` are closed under the
global context, and nothing under `rocq/` is admitted or assumed.

### Decisions

- **The refuter runs in `conclude`, after the search, on every
  `Unknown` the engine gives and on every `Exhausted`** (C2's place, the
  author's decision of 2026-10-18 for its default). It is a plain
  function, not the `Refute` trait of `plan/notes/api.md` 8.7. That kind
  waits for a second refuter, so that its interface is drawn from two
  (departs from the design). `refute_unknown` does not exist, since the
  only switch the author kept is the bound, and `classical_work = 0`
  turns the refuter off. An `Unknown` that the check made (a proof whose
  check a bound refused) is not refuted: that sequent has a proof.
- **The algorithm**: a backtracking over the goal's atoms in index
  order, false before true. At every step it evaluates in three values
  the terms of the arena the goal reaches, in arena order (operands
  first, so no recursion), and backs off where the reading is already
  true. Its unit of work is one term evaluated, its buffers (a byte per
  term twice, four per reached term) are charged to the account, and its
  polls are `Phase::Refute` with its units. The goal is the disjunction
  of its members' readings, so a goal off the roots is refuted the same
  way. The atoms outside the goal are false in the assignment.
- **Where the argument holds**: every rule of `ll R` keeps tautologies
  for every `R`. Mix joins two tautologies, both weakenings add a
  disjunct, contraction repeats one, and promotion and dereliction read
  as the formula. An intuitionistic derivation is a classical one of its
  one-sided form, whose reading is "the hypotheses imply the
  conclusion". So the refutation holds in every mode, Mix and affine
  included, and for goals off the roots. Cut would keep it too (modus
  ponens). The Rocq library proves it (`ll_tautology`, `ill_tautology`),
  and `classical::tests::checked_proofs_read_as_tautologies` pins it in
  Rust: every generated sequent of the 16 rule sets and 12
  intuitionistic ones that the search proves with a checked proof, in
  its mode and the affine one, reads as a tautology (at least 400 proved
  cases asserted), and every mutant it refutes passes `Disproof::check`.
- **The command lets the refutation run past the time limit.**
  `halting` (`cli/src/prove.rs`) answers a `Phase::Refute` poll with
  Ctrl-C alone, and the harness's child does the same. A search that
  deepens until the clock is otherwise never refuted, since the caller's
  stop would end the refuter at its first poll. The overrun is bounded
  by the work bound: on the held-out sets' 129 undecided rows the
  refutation took at most 136 ms after the limit (median 0.3 ms). A row
  so refuted can be `late` in the harness's ranking, as `mix/11` of the
  target set is, by 6 ms. The alternative, a reserve cut from the
  search's limit, would have moved decided rows near the limit.
- **`Disproof::check`** recomputes each refutation from a `Forest` of
  the sequent (default limits), bottom-up over occurrences. Its
  conditions are those under which the search states the refutation
  (`Switches`, `intervals_run`):
  - `Unbalanced` needs no weakening, and the atom's interval, with `⊤`
    and an exponential over the atom unbounded, must equal `[least,
    most]` and exclude zero;
  - `Equation` needs no weakening, the multiplicative fragment with
    units in the goal, `mix` equal to the mode's, and every count equal;
  - `Exhausted`, `OneSided` and `StateEquation` answer `Uncertified`
    (code `uncertified`, kind unsupported); a refutation that does not
    hold answers `invalid_refutation`.

  The error is a new `Error::Refutation(RefutationFault)`, an addition
  to non-exhaustive types, as are `Refutation::Classical`,
  `search::Classical`, `Options::classical_work` and the two
  `rocq::Unsupported` variants.
- **Wire level 2.** A 0.1.0 reader refuses a refutation kind it does not
  know, so the kind `classical` is a new tag and raises the level. The
  writer computes the level from the content: a disproof or an
  unprovable outcome with a classical refutation is level 2, and every
  other document stays level 1 byte for byte. The command's batch
  records take their body's level, and `check`'s JSON stays at level 1.
  `core/tests/wire/level-2.txt` holds the two new forms, read by
  `the_second_level_reads`.
- **The Rocq lemmas are inductions over `ll R`, and the two-sided
  statements go through the embedding.** `balance` holds whatever the
  atom equality is, since both literals of an axiom compare alike.
  `excess` returns `None` under `⊤` and under an exponential that
  contains the atom, which is the Rust check's condition, so a
  certificate exists exactly for a refutation that `check` accepts.
  `ill_ll` (`one_sided G C = itr C :: map (dual ∘ itr) G`) gives the
  two-sided theorems and `ill_tautology` without a second induction per
  invariant.
- **The certificate is the test, not the search's numbers.** The proof
  is `apply (THEOREM …). vm_compute. reflexivity.`, with the atom for
  `Unbalanced`, nothing for `Equation`, and for `Classical` the true
  atoms as `fun a => existsb (String.eqb a) [...]`. The Rust numbers are
  not restated: the kernel recomputes them. The statement's calculus is
  the mode's rules, since a disproof records no rules used.
- **The ordinary "not valid" certificate** checks the image's
  assignment against the ordinary sequent itself before writing, by
  name, with an atom the image lacks false. It then writes `intro H.
  specialize (H True False …). tauto.`, where `tauto` decides a closed
  formula over `True` and `False`. It is the same in LK and LJ, since a
  classical countermodel refutes both.
- **The command's default came in a commit of its own**
  (`xpywqoqt`): the library's commit (`uzsqzswr`) kept the command's
  settings off, so that the behaviour lock's and README's changes of
  verdict sit in the commit that says why. The lock's help texts and
  the new calls of `--classical-work` are in the first, as a new flag's
  calls are.

### The options, per front end (D15, D16)

- `search::Options::classical_work: u64`, default
  `Options::DEFAULT_CLASSICAL_WORK` (2²⁶), JSON `classical_work`, the
  command's and the harness's `--classical-work N`, the settings key
  `search.classical_work`. `Settings::default()` takes the library's.
  The web front end holds it in the search options' JSON, and a wrapper
  sets it with `with_classical_work`.
- `rocq::Options` is unchanged: `write_disproof` and `write_not_valid`
  read `form`, `lemma` and `prelude` (default
  `rocq::LINLOG_REFUTATION_PRELUDE` for the first, none for the second)
  and refuse `kernel: Some(NanoYalla)` for a disproof.

### Quantifiers (D17)

The classical refutation of a quantified goal needs a model, not an
assignment: `Refutation::Classical` stays the propositional case, and a
first-order countermodel would be a new variant. The Rocq invariants
carry over: `ll_tautology` becomes soundness for a first-order
structure, `balance` and `count_equation` keep their statements over
formulas with quantifiers counted as their bodies, with the eigenvariable
rules as two more cases. The refuter's place in `conclude` and
`Disproof::check` take a new arm each.

### What the library's shape makes easier (later.md, "Certified refutations")

- **Petri-net unreachability by the state equation**: `StateEquation`'s
  weights are a few numbers, and with a lemma that a Horn program's
  derivations are firing sequences (an induction over `ll` like
  `balance`) its certificate would be a test like `unbalanced`.
- **The `OneSided` count**: `bang` keeping a one-sided interval (`[0,
  ∞)` where every literal below is positive) gives its lemma with the
  same induction; the Rust check would add the matching case.
- **Finite phase models and Kripke countermodels** need a semantics
  proved sound once over `ll` or `ill`, which `ll_tautology` is the
  two-valued instance of.
- **Failure certificates** and proof by reflection are unchanged in
  difficulty: they need the focused calculus's completeness.

### What is verified

- Rocq: `rocq/theories/Refutations.v` and `rocq/tests/Refutations.v`
  (nine certificates and seven rejections) compile with Rocq 9.1.1 and
  the standard library 9.0.0 on cores 8 to 11 in capped scopes;
  `tests/Assumptions.v` prints `Closed under the global context`
  eighteen times. The exporter's ten snapshots (`refuted_*.v`,
  `not_valid_*.v`) and the six certificates the flake's checks now write
  with the command were compiled by hand and accepted silently. `grep`
  finds no `Admitted`, `admit` or `Axiom` under `rocq/`. The flake's
  checks `rocq-linlog`, `rocq` and `rocq-bridges` built under the lock
  at the exporter's tree (12:23 to 12:24, exit 0, `rocq-linlog`'s log
  the eighteen assumption lines).
- `nix flake check` under the lock:
  - a first run at the report's first commit (12:28) failed in `doc`:
    two intra-doc links of `write_disproof` did not resolve. They were
    fixed at their cause (`wokltmsn`, gated 12:30 to 12:33), and the
    docs public and private were built locally with warnings denied;
  - the rerun at `wokltmsn` passed ("all checks passed!", exit 0, 12:30
    to 12:34, `--keep-going`; `rocq-linlog` printed its eighteen
    assumption lines);
  - the run at this report's last commit is in the message to
    `planning`.
- Rust: `cargo clippy --workspace --all-targets -- --deny warnings`
  clean and `cargo test --workspace` passed at every commit. The new
  tests:
  - `a_wrong_refutation_is_rejected`: each fault, a satisfying and a
    short assignment, numbers not the goal's or that allow a proof,
    affine mode, `⊤`, an exponential, additives, Mix;
  - `checked_proofs_read_as_tautologies`;
  - `finds_an_assignment_within_its_bound`;
  - `disproof_certificates`: the snapshots, the uncertified and the
    NanoYalla refusals, and the reviewer's goal without a conclusion;
  - `the_second_level_reads`.

  The behaviour lock gained eight calls.
- The gate passed at every code commit:
  - `uzsqzswr`, 11:32 to 11:36, and `xpywqoqt`, 11:36 to 11:39, on the
    working copy at each state, in capped units on cores 8 to 11;
  - `vyqupqzq`, 11:40 to 11:44, `rrlvvovq`, 11:48 to 11:50,
    `qlsnqsyy`, 12:01 to 12:04, and `vsuttpkt` (the documentation fix),
    12:25 to 12:28, and `wokltmsn` (two documentation links), 12:30 to
    12:33, in `../linlog-step31d-gate`.

  Each ended `gate: passed`, exit 0. The ratchet's largest rise was
  +1.82 % on `batch-families`, within its ceiling: the batch's
  exhausted families now run the refuter.
- The families' verdicts (`linlog-bench run --all-families --timeout 5`,
  cores 8 to 11, 11:55 to 11:57): 123 runs and `summary --strict` passes,
  no mismatch; 10 unprovable rows carry a checked refutation.
- No problem with a checked proof is refuted. Over every problem of the
  comparison's `runs.csv` that some tool proved with a checked proof and
  that is a file (3 795 LLTP, 59 `.spec`), the command ran with
  `--work-limit 1`, so that each answer is the refuter's. All 3 854
  ended `unknown` (the work limit), none `unprovable`; the 62 family
  rows are covered by the families' run above. This probe ran on cores 8
  to 11 in four workers and was not timed.
- The target sets, under the machine lock on cores 2 and 3, 11:52:43 to
  11:59:54 (`bench/targets/after-classical.csv`, `net-after-classical.csv`):
  - against `after-routing` (`summary --against … --counters`): no
    verdict differs, the 140 rows both decide have equal counters, and
    five undecided rows are refuted, each with its assignment checked
    (`growing/16` to `/1024`, at the copy bound or the recursion limit
    before; `mix/11`, at the time limit, refuted 6 ms after it);
  - against the oracle `after-efficiency`: the same, with the five
    counter differences `bench/TARGETS.md` records for `after-routing`;
  - the net set equals `net-after-prunes` on all 64 rows.
- The held-out sets of the gap study at 10 s on one thread: the ILL
  collections KLE-01, KLE-cbn, KLE-cbv, KLE-IMP-CONJ, ILLTP-SYN and
  -LCL in all three translations, and ILLTP-SYJ-cbv (850 problems).
  The runs were `--copies none`, two shards on cores 2 and 3 under the
  lock, before with `--classical-work 0` 11:59:54 to 12:10:11 and after
  with the default 12:10:11 to 12:20:24.
  - Both passes decide the same 721 rows (568 proved, 153 unprovable,
    none late); no verdict changed and no row was newly decided.
  - The rows the gap study found classically refutable there (SYN392 and
    SYN040 in every translation, LCL181 in cbv) were decided by the
    search in both passes: the engines decide them since the
    comparison of 2026-10-15.
  - What the refuter adds there is certificates: 90 of the 153
    unprovable rows carry a checked refutation after, 15 before (the
    counts' alone). None is against a checked proof (the probe above).
  - The comparison's 26 classically refutable rows outside these sets
    (`ILLTP-SYJ-01`, the tuning set) were not run.

### The reviewer's findings

One reviewer, Opus 5.5 at `high`, in a fresh context, read the refuter,
`conclude`, `Disproof::check`, the search's conditions, `Refutations.v`
against the trusted calculi, and the two exporters. It compiled the
library and its tests, printed the assumptions of the six refutation
theorems (all closed), and wrote and compiled 39 certificates with the
command: one-sided in all four calculi, two-sided in both, every
refutation kind, an `Unknown` turned classical, and seven "not valid".
Each statement matched its input. No must-fix. Its findings, each
answered:

1. *Should-fix*: `write_disproof` took the first output member as the
   conclusion of an intuitionistic goal and panicked with none. A
   disproof read from JSON can carry such a goal, though the search
   never gives one. Answer: the writer now answers `Error::GoalOutputs`
   unless the goal has exactly one conclusion, pinned with the
   reviewer's example in `disproof_certificates` (`qlsnqsyy`).
2. *Note*: `check` does not validate the mode against the reading. The
   reviewer judged this sound, since the one-sided claim is the stronger
   one. Answer: nothing to do.
3. *Note*: the overflow comments ignored repeated goal members. Answer:
   reworded (`qlsnqsyy`); leaving an `i64` needs 2³¹ members.
4. *Note*: in minimal logic an image assignment that makes the atom
   `false` true gives no "not valid" certificate. Answer: correct
   refusal (`not_valid` re-evaluates `False`), a coverage gap only,
   recorded here.
5. *Note*: the refutation overruns the time limit by its bound. Answer:
   documented (README, `--classical-work`'s help, `cli.md`), and
   measured above.

Verdicts quoted:

- (A) "Sound in every mode and for goals off the roots … `conclude`
  touches only Unknown and Exhausted results, never Proved or another
  refutation";
- (B) "`check_classical`, `check_unbalanced` and `check_equation`
  compute exactly the Rocq tests `falsifies`, `unbalanced` and
  `fails_equation`";
- (C) "Each conclusion is `~ ll R l` or `~ ill R G C` over the trusted
  inductives … The only way to get a wrong goal is should-fix 1";
- (D) "a correct, slightly stronger statement of non-validity … The
  proof uses no axiom".

### Left open

- The comparison's driver (`bench/src/tools.rs`) still counts linlog's
  `unprovable` as unchecked: its rows come from the command's JSON. The
  harness's own runs now write `checked = ok` for a certified
  refutation, and a pass of the driver over the JSON disproof with
  `Disproof::check` would let the comparison mark them too.
- `OneSided` and `StateEquation` have no check and no lemma (above).
- `linlog check` reads proofs only; a disproof file is checked through
  the library.
- The tuning set's 26 classically refutable rows were not measured in
  this session.

### Workspaces

- `../linlog-step31d`: this session's, every commit above, made with
  `jj workspace add ../linlog-step31d -r znnkylzu`.
- `../linlog-step31d-gate`: the gates', moved from commit to commit with
  `jj new`.

The main checkout was not touched. Commits from the documentation fix
on are unsigned (`signing.behavior=drop`), since the passphrase's cache
ran out at 12:21; the six before are signed.

## Step 31: closing summary

The step built linlog's own Rocq library and tied the command's
certificates to it.

- **31a**: the one-sided calculus, the proof term, the checker and its
  soundness.
- **31b**: Mix, affine and the two-sided calculus, and the exporter's
  second kernel.
- **31c**: the bridges to NanoYalla's `nanoll.v` and Yalla's
  `nanoill.v`, and Mix's reduction.
- **31d**: certified refutations, the classical refuter by default, and
  `Disproof::check`.

Every mode has a positive certificate over a plain inductive, and an
unprovable verdict with an unbalanced atom, a failed count equation or a
classical countermodel has a negative one. Every theorem is closed under
the global context, and the NanoYalla export is as it was.

The plan's items 1 to 8 are met. What is left is in the four sections
"Left open":

- the converse of `mix_to_ll`;
- the bridges at other atom types;
- the certificates' size on large proofs;
- `OneSided`, the state equation and the comparison's checked
  refutations.
