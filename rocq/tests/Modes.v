(* linlog © Fabian Lukas Grubmüller 2026 *)
(* Licensed under the EUPL *)

(** Certificates with Mix and with affine weakening, in the form the
    exporter writes: the sequent over the calculus with the rule, proved
    by that calculus's soundness theorem from a term the checker accepts.
    The last lemmas pin that each rule is refused in the calculus without
    it. *)

From Stdlib Require Import List String.
From Linlog Require Import Soundness.
Import ListNotations.
Open Scope string_scope.

(** [⊢ A⊥, A, B⊥, B] with Mix: the two axioms side by side. *)
Lemma mix_of_axioms : ll {| mix := true; weakening := false |}
  [covar "A"; var "A"; covar "B"; var "B"].
Proof.
  apply (soundness_mix String.eqb String.eqb_spec (Mix (Ax "A") (Ax "B"))).
  vm_compute. reflexivity.
Qed.

(** [⊢ ⊥ ⅋ ⊥, 1, 1] with Mix, which classical linear logic does not
    derive. *)
Lemma mix_of_units : ll (atom := string) {| mix := true; weakening := false |}
  [parr bot bot; one; one].
Proof.
  apply (soundness_mix String.eqb String.eqb_spec
    (Par bot bot (Bot (Bot (Mix One One))))).
  vm_compute. reflexivity.
Qed.

(** [⊢ A⊥, A, B] in affine logic: [B] weakened. *)
Lemma weakening_of_atom : ll {| mix := false; weakening := true |}
  [var "B"; covar "A"; var "A"].
Proof.
  apply (soundness_affine String.eqb String.eqb_spec (Weaken (var "B") (Ax "A"))).
  vm_compute. reflexivity.
Qed.

(** [⊢ A ⊗ 1, A⊥, C] in affine logic: [C] weakened below a [⊗]. *)
Lemma weakening_under_tensor : ll {| mix := false; weakening := true |}
  [tens (var "A") one; covar "A"; var "C"].
Proof.
  apply (soundness_affine String.eqb String.eqb_spec
    (Weaken (var "C") (Tensor (var "A") one (Ax "A") One))).
  vm_compute. reflexivity.
Qed.

Lemma rejects_mix_in_affine :
  check String.eqb {| mix := false; weakening := true |} (Mix One One) [one; one] = false.
Proof. reflexivity. Qed.

Lemma rejects_weakening_with_mix :
  check String.eqb {| mix := true; weakening := false |}
    (Weaken (var "B") (Ax "A")) [var "B"; covar "A"; var "A"] = false.
Proof. reflexivity. Qed.
