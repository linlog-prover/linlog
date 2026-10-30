From Stdlib Require Import List String.
From Linlog Require Intuitionistic.
From Linlog Require Import FirstOrderIntuitionisticCheck.
Import ListNotations.
Open Scope string_scope.

Lemma certificate : ill (symbol := string) Intuitionistic.intuitionistic [iall (ivar "p" [bound 0])] (ivar "p" [tapp "f" [tapp "a" []]]).
Proof.
  apply (soundness_ill String.eqb String.eqb_spec Intuitionistic.intuitionistic
    (IAllL (ivar "p" [bound 0]) (tapp "f" [tapp "a" []]) (IAx "p" [tapp "f" [tapp "a" []]]))).
  vm_compute. reflexivity.
Qed.
