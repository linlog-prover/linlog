From Stdlib Require Import List String.
From Linlog Require Calculus.
From Linlog Require Import FirstOrderSoundness.
Import ListNotations.
Open Scope string_scope.

Lemma certificate : ll (symbol := string) {| Calculus.mix := false; Calculus.weakening := false; Calculus.cut := true |} [fex (covar "p" [bound 0]); covar "q" []; tens (var "p" [tapp "f" [tapp "g" [tapp "a" []]]]) (var "q" [])].
Proof.
  apply (soundness String.eqb String.eqb_spec {| Calculus.mix := false; Calculus.weakening := false; Calculus.cut := true |}
    (Cut (fall (tens (var "p" [tapp "f" [tapp "g" [bound 0]]]) (var "q" []))) (Forall (tens (var "p" [tapp "f" [tapp "g" [bound 0]]]) (var "q" [])) 0 (Tensor (var "p" [tapp "f" [tapp "g" [eigen 0]]]) (var "q" []) (Exists (covar "p" [bound 0]) (tapp "f" [tapp "g" [eigen 0]]) (Ax "p" [tapp "f" [tapp "g" [eigen 0]]])) (Ax "q" []))) (Exists (parr (covar "p" [tapp "f" [tapp "g" [bound 0]]]) (covar "q" [])) (tapp "a" []) (Par (covar "p" [tapp "f" [tapp "g" [tapp "a" []]]]) (covar "q" []) (Tensor (var "p" [tapp "f" [tapp "g" [tapp "a" []]]]) (var "q" []) (Ax "p" [tapp "f" [tapp "g" [tapp "a" []]]]) (Ax "q" [])))))).
  vm_compute. reflexivity.
Qed.
