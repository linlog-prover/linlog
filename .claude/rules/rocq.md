---
paths:
  - "rocq/**"
---

# The Rocq library

Loaded when a file under `rocq/` is read or changed. `modules/rocq-linlog.nix`
builds and checks it (`flake.md`). The exporter writes certificates for
it as its second kernel (`core/src/export/rocq/linlog.rs`,
`core-export.md`), beside the NanoYalla export, which it leaves as it
was.

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
- **The trusted base is `Formulas.v` and `Calculus.v`** for the one-sided
  statements and **`Intuitionistic.v`** for the two-sided ones, nothing
  else: a certificate's lemma states `ll R Γ` or `ill R Γ C` over them,
  and the checkers (`Check.v`, `IntuitionisticCheck.v`) and their proofs
  appear in the proof only. `Intuitionistic.v` holds `iformula`, `irules`
  (`intuitionistic`, `affine`) and `ill` alone: the two-sided formulas'
  equality is in `IntuitionisticCheck.v`, which the review of the
  two-sided calculus asked for so that the trusted file derives nothing. A
  change of a rule changes what every certificate means; a fresh reader
  compared the inductive with a textbook presentation of one-sided
  classical linear logic and with NanoYalla's `nanoll.v` before it was
  committed, and a change gets the same reading.
- **The modules**, the one-sided ones each exporting the one before: `Formulas` (negation
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
  `soundness_classical`, `soundness_mix`, `soundness_affine`, and `perm`,
  the tactic that decides a permutation of formula lists by `count_occ`
  and `lia`). Beside them, importing nothing of theirs: `Intuitionistic`
  (`iformula` with the constructors of Yalla's `nanoill.v`, `ivar`,
  `ione`, `itens`, `ilmap`, `itop`, `iwith`, `izero`, `iplus`, `ioc`;
  `irules` with `iweakening`; `ill R Γ C`, the rules of `nanoill.v` with
  exchange as one `Permutation` rule and `weakening_ilr` under
  `iweakening`) and `IntuitionisticCheck` (`iformula_eqb`, the two-sided
  term `iproof`, one constructor per two-sided rule, `pick`, `same`,
  `unoc`, `iderive`, `icheck`, `soundness_ill`).
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
  too large fails at the root.
- **The two-sided statement has a term of its own**, not the one-sided
  term under a one-succedent condition: a one-sided formula built from
  `⊤` and `0` alone does not say which side of `⊢` it stands on (`top` is
  `⊤` on the right and `0` on the left, so `⊢ ⊤, ⊤` is `0 ⊢ ⊤` or `⊤ ⊢ 0`
  read with one goal), the one-sided `Top` names no context, and a
  formula-level checker cannot count goals without the sides; the Rust
  checker has them from the occurrences, which the Rocq term does not
  name. So `iproof` names the rule with its side, the principal formula's
  immediate subformulas as intuitionistic formulas, and the context of
  `ITopR` and of `IZeroL` (and the goal of `IZeroL`); `iderive` computes
  every node's sequent exactly, bottom-up, and a sequent has its one
  conclusion by its type. The review proved, in probes kept outside the
  repository, that `ill intuitionistic` derives exactly what `nanoill.v`
  derives and that every derivation of `ill` has a term `icheck`
  accepts, both closed under the global context.
- **Cut and quantifiers in the two-sided calculus** plug in as in the
  one-sided one: a constructor of `ill`, one of `iproof`, an arm of
  `iderive` and a case of `iderive_sound`.
- **`sat θ Γ any`** is the invariant of the proof: `forall Δ, (any = false
  -> Δ = []) -> ll R (Γ ++ map wn θ ++ Δ)`, with `Γ` first so that a
  premise is `A :: rest` after `cbn` and a rule applies without exchange;
  `take_sat` carries a prefix for a second `take`. `soundness` holds for
  every `rules` value, so Mix and affine soundness are instances.
- **Every proof ends in `Qed`**; no `Admitted`, `admit`, `Axiom`,
  `Parameter` or `Conjecture` anywhere under `rocq/`. The check greps for
  them and requires `Print Assumptions` of both theorems
  (`tests/Assumptions.v`) to answer `Closed under the global context`, the
  only output it accepts, once per `Print Assumptions` line (the check
  counts them).
- **Certificates** (`tests/Certificates.v`, `tests/Modes.v` and
  `tests/TwoSided.v` by hand, `tests/FromLinlog.v` from the command's
  proofs; the exporter's own in `core/tests/snapshots/linlog/`): `apply
  (soundness_classical String.eqb String.eqb_spec term). vm_compute.
  reflexivity.`, `soundness_mix`/`soundness_affine` with the calculus
  as a record literal in the lemma, `soundness_ill … intuitionistic
  term` for the two-sided. Rocq prints nothing for one it accepts, and
  lemmas `check … = false` and `icheck … = false` pin what is rejected. The
  term of a proof with shared subproofs is the unfolded tree.
  `vm_compute` leaves a cast the kernel checks with its virtual machine,
  so the VM stands beside the kernel in what a certificate trusts;
  `reflexivity` alone avoids that at a cost in time. A certificate of an
  intuitionistic proof states classical derivability of its one-sided
  translation; the exporter writes the two-sided statement for a
  two-sided derivation instead.
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
