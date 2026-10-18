(* linlog © Fabian Lukas Grubmüller 2026 *)
(* Licensed under the EUPL *)

(** Certificates of intuitionistic proofs, in the form the exporter
    writes: the two-sided sequent over [ill], proved by [soundness_ill]
    from a two-sided term the checker accepts by computation. Every rule
    of the calculus is exercised; the last lemmas pin what the checker
    rejects. *)

From Stdlib Require Import List String.
From Linlog Require Import IntuitionisticCheck.
Import ListNotations.
Open Scope string_scope.

(** [A, A ⊸ B ⊢ B] *)
Lemma modus_ponens : ill intuitionistic [ivar "A"; ilmap (ivar "A") (ivar "B")] (ivar "B").
Proof.
  apply (soundness_ill String.eqb String.eqb_spec intuitionistic
    (ILmapL (ivar "A") (ivar "B") (IAx "A") (IAx "B"))).
  vm_compute. reflexivity.
Qed.

(** [A ⊗ B ⊢ B ⊗ A] *)
Lemma tensor_swap : ill intuitionistic [itens (ivar "A") (ivar "B")] (itens (ivar "B") (ivar "A")).
Proof.
  apply (soundness_ill String.eqb String.eqb_spec intuitionistic
    (ITensL (ivar "A") (ivar "B") (ITensR (ivar "B") (ivar "A") (IAx "B") (IAx "A")))).
  vm_compute. reflexivity.
Qed.

(** [⊢ 1 ⊗ (1 ⊸ (A ⊸ A))] *)
Lemma units_and_implication :
  ill intuitionistic [] (itens ione (ilmap ione (ilmap (ivar "A") (ivar "A")))).
Proof.
  apply (soundness_ill String.eqb String.eqb_spec intuitionistic
    (ITensR ione (ilmap ione (ilmap (ivar "A") (ivar "A"))) IOneR
      (ILmapR ione (ilmap (ivar "A") (ivar "A"))
        (IOneL (ILmapR (ivar "A") (ivar "A") (IAx "A")))))).
  vm_compute. reflexivity.
Qed.

(** [A & B ⊢ B & A]: both premises of [&R] have the same hypotheses. *)
Lemma with_swap : ill intuitionistic [iwith (ivar "A") (ivar "B")] (iwith (ivar "B") (ivar "A")).
Proof.
  apply (soundness_ill String.eqb String.eqb_spec intuitionistic
    (IWithR (ivar "B") (ivar "A")
      (IWithL2 (ivar "A") (ivar "B") (IAx "B"))
      (IWithL1 (ivar "A") (ivar "B") (IAx "A")))).
  vm_compute. reflexivity.
Qed.

(** [A ⊕ B ⊢ B ⊕ A]: both premises of [⊕L] have the same goal. *)
Lemma plus_swap : ill intuitionistic [iplus (ivar "A") (ivar "B")] (iplus (ivar "B") (ivar "A")).
Proof.
  apply (soundness_ill String.eqb String.eqb_spec intuitionistic
    (IPlusL (ivar "A") (ivar "B")
      (IPlusR2 (ivar "B") (ivar "A") (IAx "A"))
      (IPlusR1 (ivar "B") (ivar "A") (IAx "B")))).
  vm_compute. reflexivity.
Qed.

(** [A, B ⊢ ⊤] and [A, 0 ⊢ B]: the two rules that name their context. *)
Lemma top_right : ill intuitionistic [ivar "A"; ivar "B"] itop.
Proof.
  apply (soundness_ill String.eqb String.eqb_spec intuitionistic
    (ITopR [ivar "B"; ivar "A"])).
  vm_compute. reflexivity.
Qed.

Lemma zero_left : ill intuitionistic [ivar "A"; izero] (ivar "B").
Proof.
  apply (soundness_ill String.eqb String.eqb_spec intuitionistic
    (IZeroL [ivar "A"] (ivar "B"))).
  vm_compute. reflexivity.
Qed.

(** [0 ⊢ ⊤], which both rules prove: one-sided, both formulas are [⊤]. *)
Lemma zero_top_by_zero : ill (atom := string) intuitionistic [izero] itop.
Proof.
  apply (soundness_ill String.eqb String.eqb_spec intuitionistic (IZeroL [] itop)).
  vm_compute. reflexivity.
Qed.

Lemma zero_top_by_top : ill (atom := string) intuitionistic [izero] itop.
Proof.
  apply (soundness_ill String.eqb String.eqb_spec intuitionistic (ITopR [izero])).
  vm_compute. reflexivity.
Qed.

(** [!A ⊢ !(A ⊗ A)]: contraction, dereliction and promotion. *)
Lemma exponentials : ill intuitionistic [ioc (ivar "A")] (ioc (itens (ivar "A") (ivar "A"))).
Proof.
  apply (soundness_ill String.eqb String.eqb_spec intuitionistic
    (IOcR (itens (ivar "A") (ivar "A"))
      (ICo (ivar "A")
        (ITensR (ivar "A") (ivar "A")
          (IDe (ivar "A") (IAx "A")) (IDe (ivar "A") (IAx "A")))))).
  vm_compute. reflexivity.
Qed.

(** [!A, B ⊢ B]: a [!] hypothesis weakened. *)
Lemma weakening_of_oc : ill intuitionistic [ioc (ivar "A"); ivar "B"] (ivar "B").
Proof.
  apply (soundness_ill String.eqb String.eqb_spec intuitionistic (IWk (ivar "A") (IAx "B"))).
  vm_compute. reflexivity.
Qed.

(** [A, B ⊢ A] in affine logic: any hypothesis weakened. *)
Lemma affine_weakening : ill affine [ivar "A"; ivar "B"] (ivar "A").
Proof.
  apply (soundness_ill String.eqb String.eqb_spec affine (IWeaken (ivar "B") (IAx "A"))).
  vm_compute. reflexivity.
Qed.

Lemma rejects_weakening :
  icheck String.eqb intuitionistic (IWeaken (ivar "B") (IAx "A"))
    [ivar "A"; ivar "B"] (ivar "A") = false.
Proof. reflexivity. Qed.

Lemma rejects_promotion_of_linear_hypothesis :
  icheck String.eqb intuitionistic (IOcR (ivar "A") (IAx "A")) [ivar "A"] (ioc (ivar "A")) = false.
Proof. reflexivity. Qed.

Lemma rejects_another_goal :
  icheck String.eqb intuitionistic (IAx "A") [ivar "A"] (ivar "B") = false.
Proof. reflexivity. Qed.

Lemma rejects_missing_hypothesis :
  icheck String.eqb intuitionistic (IAx "A") [] (ivar "A") = false.
Proof. reflexivity. Qed.

Lemma rejects_with_of_other_contexts :
  icheck String.eqb intuitionistic
    (IWithR (ivar "A") (ivar "B") (IAx "A") (IAx "B"))
    [ivar "A"] (iwith (ivar "A") (ivar "B")) = false.
Proof. reflexivity. Qed.
