From Stdlib Require Import List String.
From Linlog Require Import Soundness.
Import ListNotations.
Open Scope string_scope.

Lemma certificate : ll (atom := string) classical [wn (covar "A"); wn (covar "B"); oc (tens (var "A") (var "A"))].
Proof.
  apply (soundness_classical String.eqb String.eqb_spec
    (Quest (covar "A") (Quest (covar "B") (Bang (tens (var "A") (var "A")) (Tensor (var "A") (var "A") (Copy (covar "A") (Ax "A")) (Copy (covar "A") (Ax "A"))))))).
  vm_compute. reflexivity.
Qed.
