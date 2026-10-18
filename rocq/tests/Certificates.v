(* linlog © Fabian Lukas Grubmüller 2026 *)
(* Licensed under the EUPL *)

(** Certificates of classical proofs, in the form the exporter writes:
    the sequent over the calculus, proved by the soundness theorem from a
    term the checker accepts by computation. Atoms are strings. Every
    rule of the calculus but Mix is exercised; the last lemmas pin what
    the checker rejects. [vm_compute] leaves a cast that the kernel
    checks with its virtual machine, which thereby stands beside the
    kernel in what a certificate trusts; [reflexivity] alone avoids it at
    a cost in time. *)

From Stdlib Require Import List String.
From Linlog Require Import Soundness.
Import ListNotations.
Open Scope string_scope.

(** [⊢ A ⅋ A⊥] *)
Lemma par_of_axiom : ll classical [parr (var "A") (covar "A")].
Proof.
  apply (soundness_classical String.eqb String.eqb_spec
    (Par (var "A") (covar "A") (Ax "A"))).
  vm_compute. reflexivity.
Qed.

(** [⊢ 1, ⊥] *)
Lemma units : ll (atom := string) classical [one; bot].
Proof.
  apply (soundness_classical String.eqb String.eqb_spec (Bot One)).
  vm_compute. reflexivity.
Qed.

(** [⊢ A & B, A⊥ ⊕ B⊥]: the two premises of [&] prove the same context. *)
Lemma with_plus : ll classical [awith (var "A") (var "B"); aplus (covar "A") (covar "B")].
Proof.
  apply (soundness_classical String.eqb String.eqb_spec
    (With (var "A") (var "B")
      (Plus (covar "A") (covar "B") Left (Ax "A"))
      (Plus (covar "A") (covar "B") Right (Ax "B")))).
  vm_compute. reflexivity.
Qed.

(** [⊢ A & ⊤, A⊥]: the [⊤] premise adapts to the other one's context. *)
Lemma with_top : ll classical [awith (var "A") top; covar "A"].
Proof.
  apply (soundness_classical String.eqb String.eqb_spec
    (With (var "A") top (Ax "A") Top)).
  vm_compute. reflexivity.
Qed.

(** [⊢ ⊤ ⊗ A, A⊥, B]: the [⊤] side of the split absorbs [B]. *)
Lemma top_tensor : ll classical [tens top (var "A"); covar "A"; var "B"].
Proof.
  apply (soundness_classical String.eqb String.eqb_spec
    (Tensor top (var "A") Top (Ax "A"))).
  vm_compute. reflexivity.
Qed.

(** [⊢ ⊤ & ⊤, A]: both premises absorb, so the conclusion does. *)
Lemma top_with_top : ll classical [awith top top; var "A"].
Proof.
  apply (soundness_classical String.eqb String.eqb_spec (With top top Top Top)).
  vm_compute. reflexivity.
Qed.

(** [⊢ ?A⊥, A ⊗ A]: two copies of one formula, then its [?] step. *)
Lemma contraction : ll classical [wn (covar "A"); tens (var "A") (var "A")].
Proof.
  apply (soundness_classical String.eqb String.eqb_spec
    (Quest (covar "A")
      (Tensor (var "A") (var "A")
        (Copy (covar "A") (Ax "A"))
        (Copy (covar "A") (Ax "A"))))).
  vm_compute. reflexivity.
Qed.

(** [⊢ ?A⊥, !A]: promotion over the unrestricted zone. *)
Lemma promotion : ll classical [wn (covar "A"); oc (var "A")].
Proof.
  apply (soundness_classical String.eqb String.eqb_spec
    (Quest (covar "A") (Bang (var "A") (Copy (covar "A") (Ax "A"))))).
  vm_compute. reflexivity.
Qed.

(** [⊢ ?B, 1]: weakening of a [?] formula, a rule of every calculus. *)
Lemma weakening_of_quest : ll classical [wn (var "B"); one].
Proof.
  apply (soundness_classical String.eqb String.eqb_spec (Weaken (wn (var "B")) One)).
  vm_compute. reflexivity.
Qed.

(** What the checker rejects: a [⅋] on two subformulas it does not find,
    a [⊗] that leaves the axiom's other literal over, Mix and weakening
    of an ordinary formula outside their calculi. *)
Lemma rejects_missing_subformula :
  check String.eqb classical (Par (var "A") (var "A") (Ax "A")) [parr (var "A") (var "A")] = false.
Proof. vm_compute. reflexivity. Qed.

Lemma rejects_surplus :
  check String.eqb classical (Tensor (var "A") (covar "A") (Ax "A") (Ax "A"))
    [tens (var "A") (covar "A")] = false.
Proof. vm_compute. reflexivity. Qed.

Lemma rejects_mix : check String.eqb classical (Mix One One) [one; one] = false.
Proof. vm_compute. reflexivity. Qed.

Lemma rejects_weakening : check String.eqb classical (Weaken (var "A") One) [var "A"; one] = false.
Proof. vm_compute. reflexivity. Qed.

Lemma rejects_copy_without_quest :
  check String.eqb classical (Copy (covar "A") (Ax "A")) [var "A"] = false.
Proof. vm_compute. reflexivity. Qed.
