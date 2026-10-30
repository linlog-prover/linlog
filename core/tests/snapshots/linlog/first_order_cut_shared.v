From Stdlib Require Import List String.
From Linlog Require Calculus.
From Linlog Require Import FirstOrderSoundness.
Import ListNotations.
Open Scope string_scope.

Lemma certificate : ll (symbol := string) {| Calculus.mix := false; Calculus.weakening := false; Calculus.cut := true |} [fex (fex (covar "p" [bound 1; bound 0])); awith (fall (var "p" [tapp "a" []; bound 0])) (fall (var "p" [tapp "a" []; bound 0]))].
Proof.
  apply (soundness String.eqb String.eqb_spec {| Calculus.mix := false; Calculus.weakening := false; Calculus.cut := true |}
    (Cut (fall (fall (var "p" [bound 1; bound 0]))) (Forall (fall (var "p" [bound 1; bound 0])) 0 (Forall (var "p" [eigen 0; bound 0]) 1 (Exists (fex (covar "p" [bound 1; bound 0])) (eigen 0) (Exists (covar "p" [eigen 0; bound 0]) (eigen 1) (Ax "p" [eigen 0; eigen 1]))))) (With (fall (var "p" [tapp "a" []; bound 0])) (fall (var "p" [tapp "a" []; bound 0])) (Forall (var "p" [tapp "a" []; bound 0]) 2 (Exists (fex (covar "p" [bound 1; bound 0])) (tapp "a" []) (Exists (covar "p" [tapp "a" []; bound 0]) (eigen 2) (Ax "p" [tapp "a" []; eigen 2])))) (Forall (var "p" [tapp "a" []; bound 0]) 3 (Exists (fex (covar "p" [bound 1; bound 0])) (tapp "a" []) (Exists (covar "p" [tapp "a" []; bound 0]) (eigen 3) (Ax "p" [tapp "a" []; eigen 3]))))))).
  vm_compute. reflexivity.
Qed.
