From Stdlib Require Import List String.
From Linlog Require Import Soundness.
Import ListNotations.
Open Scope string_scope.

Lemma certificate : ll (atom := string) {| mix := false; weakening := false; cut := true |} [covar "A"; tens (var "A") (covar "B"); var "B"].
Proof.
  apply (soundness String.eqb String.eqb_spec {| mix := false; weakening := false; cut := true |}
    (Cut (parr (covar "A") (var "B")) (Par (covar "A") (var "B") (Tensor (var "A") (covar "B") (Ax "A") (Ax "B"))) (Tensor (var "A") (covar "B") (Ax "A") (Ax "B")))).
  vm_compute. reflexivity.
Qed.
