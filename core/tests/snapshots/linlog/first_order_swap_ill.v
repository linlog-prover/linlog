From Stdlib Require Import List String.
From Linlog Require Intuitionistic.
From Linlog Require Import FirstOrderIntuitionisticCheck.
Import ListNotations.
Open Scope string_scope.

Lemma certificate : ill (symbol := string) Intuitionistic.intuitionistic [iex (iall (ivar "p" [bound 1; bound 0]))] (iall (iex (ivar "p" [bound 0; bound 1]))).
Proof.
  apply (soundness_ill String.eqb String.eqb_spec Intuitionistic.intuitionistic
    (IExL (iall (ivar "p" [bound 1; bound 0])) 0 (IAllR (iex (ivar "p" [bound 0; bound 1])) 1 (IAllL (ivar "p" [eigen 0; bound 0]) (eigen 1) (IExR (ivar "p" [bound 0; eigen 1]) (eigen 0) (IAx "p" [eigen 0; eigen 1])))))).
  vm_compute. reflexivity.
Qed.
