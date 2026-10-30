From Stdlib Require Import List String.
From Linlog Require Calculus.
From Linlog Require Import FirstOrderSoundness.
Import ListNotations.
Open Scope string_scope.

Lemma certificate : ll (symbol := string) {| Calculus.mix := false; Calculus.weakening := false; Calculus.cut := true |} [fex (covar "p" [bound 0]); var "p" [tapp "a" []]].
Proof.
  apply (soundness String.eqb String.eqb_spec {| Calculus.mix := false; Calculus.weakening := false; Calculus.cut := true |}
    (Cut (var "p" [tapp "a" []]) (Exists (covar "p" [bound 0]) (tapp "a" []) (Ax "p" [tapp "a" []])) (Ax "p" [tapp "a" []]))).
  vm_compute. reflexivity.
Qed.
