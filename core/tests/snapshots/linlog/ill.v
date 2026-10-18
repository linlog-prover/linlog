From Stdlib Require Import List String.
From Linlog Require Import IntuitionisticCheck.
Import ListNotations.
Open Scope string_scope.

Lemma certificate : ill (atom := string) intuitionistic [ione; iwith (ivar "A") (ivar "B"); ilmap (ivar "B") (ivar "C")] (ivar "C").
Proof.
  apply (soundness_ill String.eqb String.eqb_spec intuitionistic
    (IOneL (IWithL2 (ivar "A") (ivar "B") (ILmapL (ivar "B") (ivar "C") (IAx "B") (IAx "C"))))).
  vm_compute. reflexivity.
Qed.
