From Stdlib Require Import List String.
From Linlog Require Import Refutations.
Import ListNotations.
Open Scope string_scope.

Lemma certificate : ~ ll (atom := string) classical [covar "A"; tens (var "A") (var "A")].
Proof.
  apply (unbalanced_refutation String.eqb "A" _).
  vm_compute. reflexivity.
Qed.
