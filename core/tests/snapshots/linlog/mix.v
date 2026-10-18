From Stdlib Require Import List String.
From Linlog Require Import Soundness.
Import ListNotations.
Open Scope string_scope.

Lemma certificate : ll (atom := string) {| mix := true; weakening := false |} [covar "A"; covar "B"; var "A"; var "B"].
Proof.
  apply (soundness_mix String.eqb String.eqb_spec
    (Mix (Ax "A") (Ax "B"))).
  vm_compute. reflexivity.
Qed.
