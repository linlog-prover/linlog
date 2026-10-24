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
  its `version:`), `README.md` (the library's front page, linlog's logo by
  URL from the site, as the root README has it; no logo file is in the
  repository), `theories/` (the library, logical path `Linlog`) and
  `tests/` (certificates the check compiles against the installed library,
  not part of it) and `bridges/` (the equivalences with NanoYalla and
  `nanoill.v`, not part of the package either, compiled by the check
  `rocq-bridges`). The build's artifacts (`.vo`, `.vos`, `.vok`, `.glob`,
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
  (`rules` with `mix`, `weakening` and `cut`, `classical` with none of
  them, `ll R` with exchange as one `Permutation` rule, every other rule
  on the first formula, and `cut_r` under `cut R`, `⊢ Γ, Δ` from
  `⊢ A, Γ` and `⊢ A⊥, Δ`); `Proofs` (`branch`, `proof`: one constructor per `Node` variant of
  `core/src/proofs/mod.rs`, in the variants' order, naming formulas rather
  than occurrences); `Check` (`remove_one`, `remove_all`, `take_all`,
  `diff` on lists read as multisets; `state`, `take`, `join_with`,
  `derive`, `check`); `Soundness` (`sat`, `derive_sound`, `soundness`,
  `soundness_classical`, `soundness_mix`, `soundness_affine`, and `perm`,
  the tactic that decides a permutation of formula lists by `count_occ`
  and `lia`). Beside them, importing nothing of theirs: `Intuitionistic`
  (`iformula` with the constructors of Yalla's `nanoill.v`, `ivar`,
  `ione`, `itens`, `ilmap`, `itop`, `iwith`, `izero`, `iplus`, `ioc`;
  `irules` with `iweakening` and `icut`; `ill R Γ C`, the rules of
  `nanoill.v` with exchange as one `Permutation` rule, `weakening_ilr`
  under `iweakening` and `cut_ir` under `icut`, `Γ, Δ ⊢ C` from `Γ ⊢ A`
  and `A, Δ ⊢ C`, which `nanoill.v` lacks) and `IntuitionisticCheck`
  (`iformula_eqb`, the two-sided term `iproof`, one constructor per
  two-sided rule, `pick`, `same`, `unoc`, `cut_by`, `icut_conclusion`,
  `iderive`, `icheck`, `soundness_ill`).
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
  resetting the flag, `Weaken` of a `?` formula in every calculus, `Cut A
  l r` taking `A` from the left premise and `dual A` from the right one
  and joining them as `Tensor` does, refused without `cut R` as `Mix` is
  without `mix R`. The
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
  conclusion by its type. `bridges/NanoIllBridge.v` proves that `ill
  intuitionistic` derives exactly what `nanoill.v` derives; the review
  proved, in a probe kept outside the repository, that every derivation
  of `ill` has a term `icheck` accepts.
- **Cut is an optional rule in both calculi** (`cut`, `icut`), off in
  every named calculus (`classical`, `intuitionistic`, `affine`), so the
  cut-free statements and the bridges keep their meaning; a certificate
  of a proof with a cut states a record literal with `cut := true`,
  proved by the general `soundness`, or `icut := true` with
  `soundness_ill`. `ICut A l r` names the cut formula as the premise
  that proves it has it, and either premise may be that one: the
  exporter writes the premises in the derivation's order, where the
  occurrence of the cut formula's tree is a hypothesis when the cut is
  on an input subformula. `icut_conclusion` tries the first premise as
  the one that proves `A`, then the second; where both apply, both
  goals are `A` and the two conclusions are the same multiset.
- **Quantifiers in the two-sided calculus** plug in as cut did: a
  constructor of `ill`, one of `iproof`, an arm of `iderive` and a case
  of `iderive_sound`.
- **`sat θ Γ any`** is the invariant of the proof: `forall Δ, (any = false
  -> Δ = []) -> ll R (Γ ++ map wn θ ++ Δ)`, with `Γ` first so that a
  premise is `A :: rest` after `cbn` and a rule applies without exchange;
  `take_sat` carries a prefix for a second `take`. `soundness` holds for
  every `rules` value, so Mix and affine soundness are instances, and a
  calculus with cut takes `soundness` itself.
- **Every proof ends in `Qed`**; no `Admitted`, `admit`, `Axiom`,
  `Parameter` or `Conjecture` anywhere under `rocq/`. The check greps for
  them and requires `Print Assumptions` of every theorem
  (`tests/Assumptions.v`, and `bridges/Assumptions.v` in `rocq-bridges`)
  to answer `Closed under the global context`, the
  only output it accepts, once per `Print Assumptions` line (the check
  counts them).
- **Certificates** (`tests/Certificates.v`, `tests/Modes.v`,
  `tests/TwoSided.v` and `tests/Cuts.v` by hand, `tests/FromLinlog.v` from the command's
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
- **`Refutations.v`** (in the package, importing `Calculus` and
  `Intuitionistic`): the three invariants every derivation keeps and the
  theorems `~ ll R Γ` they give, each from a Boolean test that
  `vm_compute` decides. `eval v` is the classical reading (`!`/`?`
  erased) and `ll_tautology` holds for every `R`, cut included, so
  `falsifies v Γ = true` refutes in every calculus
  (`classical_refutation`); `excess atom_eqb a` is the hull interval of
  the atom's excess, `None` under `⊤` and under an exponential that
  contains the atom (`bang` keeps `(0, 0)` alone), `balance` needs
  `weakening R = false` and `cut R = false` and holds with Mix, and
  `unbalanced` tests all three (`unbalanced_refutation`); `weights` sums
  `#⊗ − #⅋ − #1 + #⊥` and `count_equation` needs `weakening R = false`,
  `cut R = false` and `forallb mll Γ`, with `≥` under Mix
  (`fails_equation`, `equation_refutation`). The invariant of `balance` holds whatever
  `atom_eqb` is, since both literals of an axiom compare alike; the
  certificates pass `String.eqb`. These are the conditions under which
  the Rust search states its refutations and `Disproof::check` accepts
  them, so a certificate exists exactly for a refutation of those three
  kinds that check accepts; a saturated set, which the check accepts
  too, has no theorem here (`Unsupported::Saturated`). `ill_ll` embeds the two-sided calculus in `ll (lrules R)`, which keeps
  `iweakening` and `icut`
  (`one_sided G C` is `itr C :: map (dual ∘ itr) G`, `⊸` as `A⊥ ⅋ B`),
  which gives `ill_tautology` and the two-sided forms
  `iclassical_refutation`, `iunbalanced_refutation`,
  `iequation_refutation`. `excesses` and `weights` are fixpoints, not
  `fold_right`s, so that `cbn` refolds them; the proofs unfold only
  `excesses`, `excess`, `weights` and the like (`cbn [...]`), since a
  full `cbn` turns `add (Some (0, 0)) x` and `1 + x` into matches no
  lemma rewrites. Certificates of non-derivability are in
  `tests/Refutations.v` and, from the exporter, in
  `core/tests/snapshots/linlog/`; a OneSided count (copies of one sign)
  or a state equation has no lemma yet: `bang` would keep a one-sided
  interval for the first, and the second needs Horn programs' proofs as
  firing sequences.
- **Balance and the count equation are cut-free.** A cut formula is no
  subformula of the conclusion, so neither invariant of the premises
  gives the conclusion's (an exponential cut formula has no interval, a
  non-multiplicative one fails `mll`); with cut the theorems need cut
  elimination, which the library does not prove. The tests compute
  `negb (cut R)`, so a refutation's statement is unchanged and holds of
  the calculi without cut, the ones the search refutes in.
- **`Mix.v`** (in the package, importing `Calculus` alone): `mix_to_ll`,
  for every `rules` value, gives `⊢ ?(⊥⊗⊥), Γ` in the calculus with
  `mix := false` and the same `weakening` and `cut` from `⊢ Γ` in the
  calculus (a cut contracts its premises' copies as `⊗` does),
  and `mix_to_classical` is its instance from Mix to `classical`. The
  statements write `wn (tens bot bot)` and the record literal rather
  than definitions of this file, so a reader checks them against
  `Calculus.v` alone. The proof carries `?(⊥⊗⊥)` up the derivation,
  turns a Mix into the `⊗` of the two premises under `⊥` and a
  dereliction, and contracts the copies of two-premise rules. Only this
  direction is proved: the converse needs cut in Yalla
  (`ll_to_mix2_cut`); with `cut := true` it is a cut against
  `⊢ !(1 ⅋ 1)`, which Mix derives, and is proved here neither with cut
  nor cut-free.
- **The bridges** (`bridges/NanoYallaBridge.v`, `bridges/NanoIllBridge.v`,
  `bridges/Assumptions.v`) import the upstream files by their logical
  paths, `From NanoYalla Require nanoll.` and `From Yalla.MicroYalla
  Require nanoill.` (the paths of their own projects), and qualify every
  name, since both define `map` and formulas. Each states, with atoms as
  `nat` (the upstream `Atom`/`IAtom`), `ll classical l <-> inhabited
  (nanoll.ll (map to_nanoyalla l))` and `ill intuitionistic G C <->
  inhabited (nanoill.ill (map to_nanoill G) (to_nanoill C))`, and the
  same over the upstream sequents with `of_…`; the translations map each
  constructor to its namesake and are proved inverse. Neither upstream
  kernel has cut, so the bridges state the named calculi, which have
  none; relating a calculus with cut would need cut elimination. The
  `cut_r` and `cut_ir` cases of `to_nanoll` and `to_nanoill_ill` are
  discharged as `mix_r`'s are, `cut classical` computing to `false`. They are part of
  what a reader checks, by reading: an equivalence would still hold for
  a translation that swaps `var` and `covar` everywhere, or the
  arguments of `aplus`, so it does not pin the translation down. The upstream files
  are LGPL: never copy them into the repository; to compile a bridge by
  hand, copy `nanoll.v` and `nanoill.v` from the inputs' store paths into
  a scratch directory and pass `-R nanoyalla NanoYalla -R microyalla
  Yalla.MicroYalla -R bridges LinlogBridges` as the check does. A bridge
  to another calculus (the Mix or affine ones) has no upstream kernel to
  state it against in either file.
- **`ll` is in `Prop`** ("a derivation exists"), as a textbook reads it,
  and `Permutation` is `Prop` too, while NanoYalla's and Yalla's kernels
  are in `Type`. A bridge therefore states `ll classical l <-> inhabited
  (NanoYalla.ll (map tr l))`, or proves a `Type`-valued soundness from
  `check … = true` directly (the term is data and every case analysis is
  on a Boolean or an option), never a `match` on `ll` into `Type`, which
  Rocq refuses. The bridges state the `inhabited` form.
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
- **Where things plug in.** A new rule is a constructor of `ll`, a
  constructor of `proof`, an arm of `derive` and a case of `derive_sound`,
  nothing redone, as cut was; an optional one is also a field of `rules`
  (every record literal under `rocq/` and the exporter's gains it) and a
  case of every induction on `ll` (`Mix.v`, `Refutations.v`, the
  bridges). A new calculus is a `rules` value and an instance of
  `soundness`. A first-order extension is a second development beside
  this one, as the plan wants: its own formula type (atoms become
  predicates applied to terms, with quantifiers), its own `ll` with the
  quantifier rules and their side conditions, and a checker with two more
  cases, reusing the multiset functions and the shape of `sat`; nothing
  here changes.
