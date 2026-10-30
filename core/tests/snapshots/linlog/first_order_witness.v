From Stdlib Require Import List String.
From Linlog Require Calculus.
From Linlog Require Import FirstOrderSoundness.
Import ListNotations.
Open Scope string_scope.

Lemma certificate : ll (symbol := string) Calculus.classical [fex (covar "p" [bound 0]); var "p" [tapp "f" [tapp "a" []]]].
Proof.
  apply (soundness_classical String.eqb String.eqb_spec
    (Exists (covar "p" [bound 0]) (tapp "f" [tapp "a" []]) (Ax "p" [tapp "f" [tapp "a" []]]))).
  vm_compute. reflexivity.
Qed.
