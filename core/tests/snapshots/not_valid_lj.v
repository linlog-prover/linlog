Lemma certificate : ~ (forall a b : Prop, (a \/ (b -> a))).
Proof.
  intro H. specialize (H False True). tauto.
Qed.

