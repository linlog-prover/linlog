(* linlog © Fabian Lukas Grubmüller 2026 *)
(* Licensed under the EUPL *)

(** Certificates of proofs with cuts, in the form the exporter writes:
    the sequent over a calculus with cut, given as a record literal,
    proved by [soundness] or [soundness_ill] from a term the checker
    accepts by computation. The last lemmas pin that a cut whose
    premises do not hold the cut formula and its dual is refused, and
    that a cut is refused in a calculus without it. *)

From Stdlib Require Import List String.
From Linlog Require Import Soundness IntuitionisticCheck.
Import ListNotations.
Open Scope string_scope.

(** [⊢ A⊥, A ⊗ B⊥, B]: a cut on [A⊥ ⅋ B] against its dual. *)
Lemma cut_of_par : ll {| mix := false; weakening := false; cut := true |}
  [covar "A"; tens (var "A") (covar "B"); var "B"].
Proof.
  apply (soundness String.eqb String.eqb_spec {| mix := false; weakening := false; cut := true |}
    (Cut (parr (covar "A") (var "B"))
      (Par (covar "A") (var "B") (Tensor (var "A") (covar "B") (Ax "A") (Ax "B")))
      (Tensor (var "A") (covar "B") (Ax "A") (Ax "B")))).
  vm_compute. reflexivity.
Qed.

(** [⊢ !A, ?A⊥]: a cut on [?A⊥], which each premise moves into its
    unrestricted zone and back, against [!A]. *)
Lemma cut_of_quest : ll {| mix := false; weakening := false; cut := true |}
  [oc (var "A"); wn (covar "A")].
Proof.
  apply (soundness String.eqb String.eqb_spec {| mix := false; weakening := false; cut := true |}
    (Cut (wn (covar "A"))
      (Quest (covar "A") (Bang (var "A") (Copy (covar "A") (Ax "A"))))
      (Quest (covar "A") (Bang (var "A") (Copy (covar "A") (Ax "A")))))).
  vm_compute. reflexivity.
Qed.

(** [⊢ A⊥, 1, A] with Mix and cut: a cut on [A] above a Mix. *)
Lemma cut_with_mix : ll {| mix := true; weakening := false; cut := true |}
  [covar "A"; one; var "A"].
Proof.
  apply (soundness String.eqb String.eqb_spec {| mix := true; weakening := false; cut := true |}
    (Cut (var "A") (Mix (Ax "A") One) (Ax "A"))).
  vm_compute. reflexivity.
Qed.

(** [A, A ⊸ B ⊢ B]: a cut on [A ⊸ B], whose first premise proves it. *)
Lemma icut_proved_first : ill {| iweakening := false; icut := true |}
  [ivar "A"; ilmap (ivar "A") (ivar "B")] (ivar "B").
Proof.
  apply (soundness_ill String.eqb String.eqb_spec {| iweakening := false; icut := true |}
    (ICut (ilmap (ivar "A") (ivar "B"))
      (ILmapR (ivar "A") (ivar "B") (ILmapL (ivar "A") (ivar "B") (IAx "A") (IAx "B")))
      (ILmapL (ivar "A") (ivar "B") (IAx "A") (IAx "B")))).
  vm_compute. reflexivity.
Qed.

(** The same cut with its premises the other way round: the first has
    [A ⊸ B] as a hypothesis, the second proves it. *)
Lemma icut_proved_second : ill {| iweakening := false; icut := true |}
  [ivar "A"; ilmap (ivar "A") (ivar "B")] (ivar "B").
Proof.
  apply (soundness_ill String.eqb String.eqb_spec {| iweakening := false; icut := true |}
    (ICut (ilmap (ivar "A") (ivar "B"))
      (ILmapL (ivar "A") (ivar "B") (IAx "A") (IAx "B"))
      (ILmapR (ivar "A") (ivar "B") (ILmapL (ivar "A") (ivar "B") (IAx "A") (IAx "B"))))).
  vm_compute. reflexivity.
Qed.

Lemma rejects_cut_on_another_formula :
  check String.eqb {| mix := false; weakening := false; cut := true |}
    (Cut (var "A") (Ax "A") (Ax "B")) [covar "A"; var "B"] = false.
Proof. reflexivity. Qed.

Lemma rejects_cut_in_classical :
  check String.eqb classical (Cut (var "A") (Ax "A") (Ax "A")) [covar "A"; var "A"] = false.
Proof. reflexivity. Qed.

Lemma rejects_icut_on_another_formula :
  icheck String.eqb {| iweakening := false; icut := true |}
    (ICut (ivar "B") (IAx "A") (IAx "A")) [ivar "A"] (ivar "A") = false.
Proof. reflexivity. Qed.

Lemma rejects_icut_in_intuitionistic :
  icheck String.eqb intuitionistic
    (ICut (ilmap (ivar "A") (ivar "B"))
      (ILmapR (ivar "A") (ivar "B") (ILmapL (ivar "A") (ivar "B") (IAx "A") (IAx "B")))
      (ILmapL (ivar "A") (ivar "B") (IAx "A") (IAx "B")))
    [ivar "A"; ilmap (ivar "A") (ivar "B")] (ivar "B") = false.
Proof. reflexivity. Qed.
