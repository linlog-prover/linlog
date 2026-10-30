From Stdlib Require Import List String.
From Linlog Require Calculus.
From Linlog Require Import FirstOrderSoundness.
Import ListNotations.
Open Scope string_scope.

Lemma certificate : ll (symbol := string) {| Calculus.mix := false; Calculus.weakening := false; Calculus.cut := true |} [fex (covar "p" [bound 0]); var "p" [tapp "a" []]].
Proof.
  apply (soundness String.eqb String.eqb_spec {| Calculus.mix := false; Calculus.weakening := false; Calculus.cut := true |}
    (Cut (fall (var "p" [bound 0])) (Forall (var "p" [bound 0]) 0 (Exists (covar "p" [bound 0]) (eigen 0) (Ax "p" [eigen 0]))) (Exists (covar "p" [bound 0]) (tapp "a" []) (Ax "p" [tapp "a" []])))).
  vm_compute. reflexivity.
Qed.
