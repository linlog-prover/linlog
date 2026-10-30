From Stdlib Require Import List String.
From Linlog Require Intuitionistic.
From Linlog Require Import FirstOrderIntuitionisticCheck.
Import ListNotations.
Open Scope string_scope.

Lemma certificate : ill (symbol := string) {| Intuitionistic.iweakening := false; Intuitionistic.icut := true |} [iall (ivar "p" [bound 0])] (ivar "p" [tapp "a" []]).
Proof.
  apply (soundness_ill String.eqb String.eqb_spec {| Intuitionistic.iweakening := false; Intuitionistic.icut := true |}
    (ICut (iall (ivar "p" [bound 0])) (IAllR (ivar "p" [bound 0]) 0 (IAllL (ivar "p" [bound 0]) (eigen 0) (IAx "p" [eigen 0]))) (IAllL (ivar "p" [bound 0]) (tapp "a" []) (IAx "p" [tapp "a" []])))).
  vm_compute. reflexivity.
Qed.
