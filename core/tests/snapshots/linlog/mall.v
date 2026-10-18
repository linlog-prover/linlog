From Stdlib Require Import List String.
From Linlog Require Import Soundness.
Import ListNotations.
Open Scope string_scope.

Lemma certificate : ll (atom := string) classical [awith (covar "A") (covar "B"); bot; aplus (var "B") (var "A")].
Proof.
  apply (soundness_classical String.eqb String.eqb_spec
    (With (covar "A") (covar "B") (Bot (Plus (var "B") (var "A") Right (Ax "A"))) (Bot (Plus (var "B") (var "A") Left (Ax "B"))))).
  vm_compute. reflexivity.
Qed.
