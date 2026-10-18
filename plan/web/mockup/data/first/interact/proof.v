Lemma certificate (A B : formula) : ll [dual A; tens A (dual B); B].
Proof.
apply (tens_r_ext [dual A]); cbn_sequent.
{
  ax_expansion.
}
{
  ax_expansion.
}
Qed.
