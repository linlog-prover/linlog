(* linlog © Fabian Lukas Grubmüller 2026 *)
(* Licensed under the EUPL *)

(** Certificates of first-order proofs, two-sided, in the form the
    exporter writes: the sequent over the first-order intuitionistic
    calculus, proved by [soundness_ill] from a term the checker accepts
    by computation. The last lemmas pin that an eigenvariable escaping
    its step through a witness below it, and a wrong witness, are
    refused. *)

From Stdlib Require Import List String.
From Linlog Require Intuitionistic.
From Linlog Require Import FirstOrderIntuitionisticCheck.
Import ListNotations.
Open Scope string_scope.

(** [∃y. ∀x. r(x, y) ⊢ ∀x. ∃y. r(x, y)]: [∃L] and [∀R] introduce the
    eigenvariables below the witnesses of [∀L] and [∃R] that name them. *)
Lemma exists_forall : ill (symbol := string) Intuitionistic.intuitionistic
  [iex (iall (ivar "r" [bound 0; bound 1]))] (iall (iex (ivar "r" [bound 1; bound 0]))).
Proof.
  apply (soundness_ill String.eqb String.eqb_spec Intuitionistic.intuitionistic
    (IExL (iall (ivar "r" [bound 0; bound 1])) 0
      (IAllR (iex (ivar "r" [bound 1; bound 0])) 1
        (IAllL (ivar "r" [bound 0; eigen 0]) (eigen 1)
          (IExR (ivar "r" [eigen 1; bound 0]) (eigen 0)
            (IAx "r" [eigen 1; eigen 0])))))).
  vm_compute. reflexivity.
Qed.

(** [∀x. p(x), q ⊢ ∃y. p(y)], affine: the witness a constant no step
    introduces. *)
Lemma affine_nonempty : ill (symbol := string) Intuitionistic.affine
  [iall (ivar "p" [bound 0]); ivar "q" []] (iex (ivar "p" [bound 0])).
Proof.
  apply (soundness_ill String.eqb String.eqb_spec Intuitionistic.affine
    (IWeaken (ivar "q" [])
      (IAllL (ivar "p" [bound 0]) (eigen 0)
        (IExR (ivar "p" [bound 0]) (eigen 0) (IAx "p" [eigen 0]))))).
  vm_compute. reflexivity.
Qed.

(** [∀x. p(x) ⊢ ∃y. p(y)] by a cut on [∀z. p(z)]. *)
Lemma cut_of_forall : ill (symbol := string)
  {| Intuitionistic.iweakening := false; Intuitionistic.icut := true |}
  [iall (ivar "p" [bound 0])] (iex (ivar "p" [bound 0])).
Proof.
  apply (soundness_ill String.eqb String.eqb_spec
    {| Intuitionistic.iweakening := false; Intuitionistic.icut := true |}
    (ICut (iall (ivar "p" [bound 0]))
      (IAllR (ivar "p" [bound 0]) 0
        (IAllL (ivar "p" [bound 0]) (eigen 0) (IAx "p" [eigen 0])))
      (IAllL (ivar "p" [bound 0]) (eigen 1)
        (IExR (ivar "p" [bound 0]) (eigen 1) (IAx "p" [eigen 1]))))).
  vm_compute. reflexivity.
Qed.

(** An eigenvariable escaping its step: the unprovable [∀x. ∃y. r(x, y)
    ⊢ ∃y. ∀x. r(x, y)] with the witnesses chosen below the steps that
    introduce them. *)
Lemma escaping :
  icheck String.eqb Intuitionistic.intuitionistic
    (IExR (iall (ivar "r" [bound 0; bound 1])) (eigen 0)
      (IAllL (iex (ivar "r" [bound 1; bound 0])) (eigen 1)
        (IAllR (ivar "r" [bound 0; eigen 0]) 1
          (IExL (ivar "r" [eigen 1; bound 0]) 0
            (IAx "r" [eigen 1; eigen 0])))))
    [iall (iex (ivar "r" [bound 1; bound 0]))] (iex (iall (ivar "r" [bound 0; bound 1])))
  = false.
Proof. vm_compute. reflexivity. Qed.

(** A wrong witness: [∀x. p(x) ⊢ p(c)] is not proved at [d]. *)
Lemma wrong_witness :
  icheck String.eqb Intuitionistic.intuitionistic
    (IAllL (ivar "p" [bound 0]) (tapp "d" []) (IAx "p" [tapp "c" []]))
    [iall (ivar "p" [bound 0])] (ivar "p" [tapp "c" []]) = false.
Proof. vm_compute. reflexivity. Qed.
