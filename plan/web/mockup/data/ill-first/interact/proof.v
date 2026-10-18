Lemma certificate : ill (atom := string) intuitionistic [ivar "A"; ilmap (ivar "A") (ivar "B")] (ivar "B").
Proof.
  apply (soundness_ill String.eqb String.eqb_spec intuitionistic
    (ILmapL (ivar "A") (ivar "B") (IAx "A") (IAx "B"))).
  vm_compute. reflexivity.
Qed.
