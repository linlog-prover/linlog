From Stdlib Require Import List String.
From Linlog Require Calculus.
From Linlog Require Import FirstOrderSoundness.
Import ListNotations.
Open Scope string_scope.

Lemma certificate : ll (symbol := string) Calculus.classical [fex (covar "p" [bound 0]); fex (var "p" [bound 0])].
Proof.
  apply (soundness_classical String.eqb String.eqb_spec
    (Exists (covar "p" [bound 0]) (eigen 2) (Exists (var "p" [bound 0]) (eigen 2) (Ax "p" [eigen 2])))).
  vm_compute. reflexivity.
Qed.
