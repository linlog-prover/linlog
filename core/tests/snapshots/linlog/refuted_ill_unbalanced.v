From Stdlib Require Import List String.
From Linlog Require Import Refutations.
Import ListNotations.
Open Scope string_scope.

Lemma certificate : ~ ill (atom := string) intuitionistic [ivar "A"] (ivar "B").
Proof.
  apply (iunbalanced_refutation String.eqb "A" _).
  vm_compute. reflexivity.
Qed.
