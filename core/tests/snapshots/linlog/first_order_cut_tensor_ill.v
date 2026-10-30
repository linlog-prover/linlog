From Stdlib Require Import List String.
From Linlog Require Intuitionistic.
From Linlog Require Import FirstOrderIntuitionisticCheck.
Import ListNotations.
Open Scope string_scope.

Lemma certificate : ill (symbol := string) {| Intuitionistic.iweakening := false; Intuitionistic.icut := true |} [iall (itens (ivar "p" [bound 0]) (ivar "q" [bound 0]))] (iall (itens (ivar "p" [bound 0]) (ivar "q" [bound 0]))).
Proof.
  apply (soundness_ill String.eqb String.eqb_spec {| Intuitionistic.iweakening := false; Intuitionistic.icut := true |}
    (IAllR (itens (ivar "p" [bound 0]) (ivar "q" [bound 0])) 0 (ICut (itens (ivar "p" [eigen 0]) (ivar "q" [eigen 0])) (IAllL (itens (ivar "p" [bound 0]) (ivar "q" [bound 0])) (eigen 0) (ITensL (ivar "p" [eigen 0]) (ivar "q" [eigen 0]) (ITensR (ivar "p" [eigen 0]) (ivar "q" [eigen 0]) (IAx "p" [eigen 0]) (IAx "q" [eigen 0])))) (ITensL (ivar "p" [eigen 0]) (ivar "q" [eigen 0]) (ITensR (ivar "p" [eigen 0]) (ivar "q" [eigen 0]) (IAx "p" [eigen 0]) (IAx "q" [eigen 0])))))).
  vm_compute. reflexivity.
Qed.
