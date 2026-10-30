From Stdlib Require Import List String.
From Linlog Require Intuitionistic.
From Linlog Require Import FirstOrderIntuitionisticCheck.
Import ListNotations.
Open Scope string_scope.

Lemma certificate : ill (symbol := string) {| Intuitionistic.iweakening := false; Intuitionistic.icut := true |} [iall (ivar "p" [bound 0]); ivar "q" []] (itens (ivar "p" [tapp "f" [tapp "g" [tapp "a" []]]]) (ivar "q" [])).
Proof.
  apply (soundness_ill String.eqb String.eqb_spec {| Intuitionistic.iweakening := false; Intuitionistic.icut := true |}
    (ICut (iall (itens (ivar "p" [tapp "f" [tapp "g" [bound 0]]]) (ivar "q" []))) (IAllR (itens (ivar "p" [tapp "f" [tapp "g" [bound 0]]]) (ivar "q" [])) 0 (ITensR (ivar "p" [tapp "f" [tapp "g" [eigen 0]]]) (ivar "q" []) (IAllL (ivar "p" [bound 0]) (tapp "f" [tapp "g" [eigen 0]]) (IAx "p" [tapp "f" [tapp "g" [eigen 0]]])) (IAx "q" []))) (IAllL (itens (ivar "p" [tapp "f" [tapp "g" [bound 0]]]) (ivar "q" [])) (tapp "a" []) (ITensL (ivar "p" [tapp "f" [tapp "g" [tapp "a" []]]]) (ivar "q" []) (ITensR (ivar "p" [tapp "f" [tapp "g" [tapp "a" []]]]) (ivar "q" []) (IAx "p" [tapp "f" [tapp "g" [tapp "a" []]]]) (IAx "q" [])))))).
  vm_compute. reflexivity.
Qed.
