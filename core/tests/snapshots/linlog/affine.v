From Stdlib Require Import List String.
From Linlog Require Import Soundness.
Import ListNotations.
Open Scope string_scope.

Lemma certificate : ll (atom := string) {| mix := false; weakening := true; cut := false |} [covar "A"; covar "B"; var "A"].
Proof.
  apply (soundness_affine String.eqb String.eqb_spec
    (Weaken (covar "B") (Ax "A"))).
  vm_compute. reflexivity.
Qed.
