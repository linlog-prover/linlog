From Stdlib Require Import List String.
From Linlog Require Import Soundness.
Import ListNotations.
Open Scope string_scope.

Lemma certificate : ll (atom := string) classical [wn (aplus (covar "A") (covar "B")); tens (oc (var "A")) (oc (var "B"))].
Proof.
  apply (soundness_classical String.eqb String.eqb_spec
    (Quest (aplus (covar "A") (covar "B")) (Tensor (oc (var "A")) (oc (var "B")) (Bang (var "A") (Copy (aplus (covar "A") (covar "B")) (Plus (covar "A") (covar "B") Left (Ax "A")))) (Bang (var "B") (Copy (aplus (covar "A") (covar "B")) (Plus (covar "A") (covar "B") Right (Ax "B"))))))).
  vm_compute. reflexivity.
Qed.
