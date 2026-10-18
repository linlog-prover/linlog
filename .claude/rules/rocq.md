---
paths:
  - "rocq/**"
---

# The Rocq library

Loaded when a file under `rocq/` is read or changed. `modules/rocq-linlog.nix`
builds and checks it (`flake.md`). The NanoYalla export
(`core/src/export/rocq.rs`, `core-export.md`) writes for another kernel
and is unchanged by this library.

- **Layout.** `_RocqProject` (`-R theories Linlog`, every file listed),
  `Makefile` (the reference manual's wrapper: it generates `RocqMakefile`
  from `_RocqProject` with `rocq makefile` and hands it every target, so
  `make` builds and `make install` installs under `COQLIBINSTALL`),
  `rocq-linlog.opam` (the archive's name for the library; the flake reads
  its `version:`), `theories/` (the library, logical path `Linlog`) and
  `tests/` (certificates the check compiles against the installed library,
  not part of it). The build's artifacts (`.vo`, `.vos`, `.vok`, `.glob`,
  `.*.aux`, `RocqMakefile`, `RocqMakefile.conf`, `.lia.cache`) are
  ignored; a `make` in `rocq/` from the devshell needs Rocq, which the
  devshell does not carry (`nix shell --inputs-from . nixpkgs#rocq-core
  nixpkgs#rocqPackages.stdlib`, then `ROCQPATH` to the standard library's
  `user-contrib`).
- **The trusted base is `Formulas.v` and `Calculus.v`**, nothing else: a
  certificate's lemma states `ll classical Γ` over them, and the checker
  (`Check.v`) and its proof (`Soundness.v`) appear in the proof only. A
  change of a rule changes what every certificate means; a fresh reader
  compared the inductive with a textbook presentation of one-sided
  classical linear logic and with NanoYalla's `nanoll.v` before it was
  committed, and a change gets the same reading.
- **The modules**, each exporting the one before: `Formulas` (negation
  normal form over an abstract atom type, the constructors named as in
  Yalla and NanoYalla, `dual`, `formula_eqb` and `formula_eqb_spec` under a
  reflecting atom equality, `formula_eq_dec` for the proofs); `Calculus`
  (`rules` with `mix` and `weakening`, `classical`, `ll R` with exchange as
  one `Permutation` rule and every other rule on the first formula, no
  cut); `Proofs` (`branch`, `proof`: one constructor per `Node` variant of
  `core/src/proofs/mod.rs`, in the variants' order, naming formulas rather
  than occurrences); `Check` (`remove_one`, `remove_all`, `take_all`,
  `diff` on lists read as multisets; `state`, `take`, `join_with`,
  `derive`, `check`); `Soundness` (`sat`, `derive_sound`, `soundness`,
  `soundness_classical`, and `perm`, the tactic that decides a permutation
  of formula lists by `count_occ` and `lia`).
- **Atoms are a parameter** (`Context {atom : Type}`): the calculus
  assumes nothing of them, the checker takes `atom_eqb : atom -> atom ->
  bool`, and the theorem assumes `forall a b, reflect (a = b) (atom_eqb a
  b)`. The certificates use `string` with `String.eqb` and
  `String.eqb_spec`; a sequent without an atom names the type (`ll (atom
  := string) classical [one; bot]`). The implicit `atom` is cleared on
  the types `formula`, `proof` and `state`, so they are written applied.
- **The checker runs the Rust checker's algorithm over formulas, not
  occurrences.** Two occurrences of one formula are interchangeable in a
  multiset, so `Ax a` stands for `[covar a; var a]`, `Copy A` needs no `?`
  above it (the root requires an empty unrestricted zone), and `Quest A`
  removes every `A` from the unrestricted zone (the proof contracts the
  copies, or weakens when there is none). Everything else follows
  `core/src/proofs/oracle.rs` rule by rule: `take` with the absorbing
  flag, the four cases of `&` in `join_with` (the union of two absorbing
  zones is `gl ++ diff gr gl`), promotion needing an empty linear zone and
  resetting the flag, `Weaken` of a `?` formula in every calculus. The
  Rust `Fault::Surplus` has no counterpart: zones are lists, and a zone
  too large fails at the root. No intuitionistic reading yet: the
  one-succedent condition and the two-sided statement come with the
  second kernel.
- **`sat θ Γ any`** is the invariant of the proof: `forall Δ, (any = false
  -> Δ = []) -> ll R (Γ ++ map wn θ ++ Δ)`, with `Γ` first so that a
  premise is `A :: rest` after `cbn` and a rule applies without exchange;
  `take_sat` carries a prefix for a second `take`. `soundness` holds for
  every `rules` value, so Mix and affine soundness are instances.
- **Every proof ends in `Qed`**; no `Admitted`, `admit`, `Axiom`,
  `Parameter` or `Conjecture` anywhere under `rocq/`. The check greps for
  them and requires `Print Assumptions` of both theorems
  (`tests/Assumptions.v`) to answer `Closed under the global context`, the
  only output it accepts.
- **Certificates** (`tests/Certificates.v` by hand, `tests/FromLinlog.v`
  from the command's proofs): `apply (soundness_classical String.eqb
  String.eqb_spec term). vm_compute. reflexivity.` Rocq prints nothing for
  one it accepts, and lemmas `check … = false` pin what is rejected. The
  term of a proof with shared subproofs is the unfolded tree.
  `vm_compute` leaves a cast the kernel checks with its virtual machine,
  so the VM stands beside the kernel in what a certificate trusts;
  `reflexivity` alone avoids that at a cost in time. A certificate of an
  intuitionistic proof states classical derivability of its one-sided
  translation until the two-sided statement exists, and says so.
- **`ll` is in `Prop`** ("a derivation exists"), as a textbook reads it,
  and `Permutation` is `Prop` too, while NanoYalla's and Yalla's kernels
  are in `Type`. A bridge therefore states `ll classical l <-> inhabited
  (NanoYalla.ll (map tr l))`, or proves a `Type`-valued soundness from
  `check … = true` directly (the term is data and every case analysis is
  on a Boolean or an option), never a `match` on `ll` into `Type`, which
  Rocq refuses. The reviewer of 31a proved both directions of the
  `inhabited` form in a probe, with atoms as `nat`.
- **Names.** `Soundness` exports `Check` and `Calculus`, so a certificate
  that imports it sees `take`, `diff`, `remove_one`, `derive`, `check`
  and the rules' constructors unqualified; a development that mixes
  libraries can `Require` without `Import` and qualify. The `let*`
  notation is local to `Check.v`.
- **Conventions.** The 9.1 reference manual (the Rocq nixpkgs ships) names
  the project file `_CoqProject` and the generated makefile `CoqMakefile`;
  the 9.3 manual names `_RocqProject` and `RocqMakefile`, which this
  library uses, both being passed by `-f` and `-o`. `From Stdlib Require`
  throughout. The licence header is two `(* … *)` lines; the conventions
  check covers `.v` files; `implicits` (of `Arguments … : clear
  implicits`) is a word of `typos.toml`.
- **Where things plug in.** A new rule (cut) is a constructor of `ll`, a
  constructor of `proof`, an arm of `derive` and a case of `derive_sound`,
  nothing redone. A new calculus is a `rules` value and an instance of
  `soundness`. A first-order extension is a second development beside
  this one, as the plan wants: its own formula type (atoms become
  predicates applied to terms, with quantifiers), its own `ll` with the
  quantifier rules and their side conditions, and a checker with two more
  cases, reusing the multiset functions and the shape of `sat`; nothing
  here changes.
