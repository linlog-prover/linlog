From Stdlib Require Import List String.
From Linlog Require Import IntuitionisticCheck.
Import ListNotations.
Open Scope string_scope.

Lemma certificate : ill (atom := string) {| iweakening := false; icut := true |} [ivar "A"; ilmap (ivar "A") (ivar "B")] (ivar "B").
Proof.
  apply (soundness_ill String.eqb String.eqb_spec {| iweakening := false; icut := true |}
    (ICut (ilmap (ivar "A") (ivar "B")) (ILmapR (ivar "A") (ivar "B") (ILmapL (ivar "A") (ivar "B") (IAx "A") (IAx "B"))) (ILmapL (ivar "A") (ivar "B") (IAx "A") (IAx "B")))).
  vm_compute. reflexivity.
Qed.
