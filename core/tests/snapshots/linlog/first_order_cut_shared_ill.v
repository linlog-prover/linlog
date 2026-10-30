From Stdlib Require Import List String.
From Linlog Require Intuitionistic.
From Linlog Require Import FirstOrderIntuitionisticCheck.
Import ListNotations.
Open Scope string_scope.

Lemma certificate : ill (symbol := string) {| Intuitionistic.iweakening := false; Intuitionistic.icut := true |} [iall (iall (ivar "p" [bound 1; bound 0]))] (iwith (iall (ivar "p" [tapp "a" []; bound 0])) (iall (ivar "p" [tapp "a" []; bound 0]))).
Proof.
  apply (soundness_ill String.eqb String.eqb_spec {| Intuitionistic.iweakening := false; Intuitionistic.icut := true |}
    (ICut (iall (iall (ivar "p" [bound 1; bound 0]))) (IAllR (iall (ivar "p" [bound 1; bound 0])) 0 (IAllR (ivar "p" [eigen 0; bound 0]) 1 (IAllL (iall (ivar "p" [bound 1; bound 0])) (eigen 0) (IAllL (ivar "p" [eigen 0; bound 0]) (eigen 1) (IAx "p" [eigen 0; eigen 1]))))) (IWithR (iall (ivar "p" [tapp "a" []; bound 0])) (iall (ivar "p" [tapp "a" []; bound 0])) (IAllR (ivar "p" [tapp "a" []; bound 0]) 2 (IAllL (iall (ivar "p" [bound 1; bound 0])) (tapp "a" []) (IAllL (ivar "p" [tapp "a" []; bound 0]) (eigen 2) (IAx "p" [tapp "a" []; eigen 2])))) (IAllR (ivar "p" [tapp "a" []; bound 0]) 3 (IAllL (iall (ivar "p" [bound 1; bound 0])) (tapp "a" []) (IAllL (ivar "p" [tapp "a" []; bound 0]) (eigen 3) (IAx "p" [tapp "a" []; eigen 3]))))))).
  vm_compute. reflexivity.
Qed.
