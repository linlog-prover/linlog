From Stdlib Require Import List String.
From Linlog Require Import Soundness.
Import ListNotations.
Open Scope string_scope.

Lemma certificate : ll (atom := string) classical [parr (covar "A") (covar "B"); tens (var "B") (var "A")].
Proof.
  apply (soundness_classical String.eqb String.eqb_spec
    (Par (covar "A") (covar "B") (Tensor (var "B") (var "A") (Ax "B") (Ax "A")))).
  vm_compute. reflexivity.
Qed.
