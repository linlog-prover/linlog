From Stdlib Require Import List String.
From Linlog Require Calculus.
From Linlog Require Import FirstOrderSoundness.
Import ListNotations.
Open Scope string_scope.

Lemma certificate : ll (symbol := string) {| Calculus.mix := false; Calculus.weakening := false; Calculus.cut := true |} [fex (parr (covar "p" [bound 0]) (covar "q" [bound 0])); fall (tens (var "p" [bound 0]) (var "q" [bound 0]))].
Proof.
  apply (soundness String.eqb String.eqb_spec {| Calculus.mix := false; Calculus.weakening := false; Calculus.cut := true |}
    (Forall (tens (var "p" [bound 0]) (var "q" [bound 0])) 0 (Cut (tens (var "p" [eigen 0]) (var "q" [eigen 0])) (Exists (parr (covar "p" [bound 0]) (covar "q" [bound 0])) (eigen 0) (Par (covar "p" [eigen 0]) (covar "q" [eigen 0]) (Tensor (var "p" [eigen 0]) (var "q" [eigen 0]) (Ax "p" [eigen 0]) (Ax "q" [eigen 0])))) (Par (covar "p" [eigen 0]) (covar "q" [eigen 0]) (Tensor (var "p" [eigen 0]) (var "q" [eigen 0]) (Ax "p" [eigen 0]) (Ax "q" [eigen 0])))))).
  vm_compute. reflexivity.
Qed.
