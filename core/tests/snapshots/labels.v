From NanoYalla Require Import macroll.

Lemma certificate (A B : formula) : ll [awith (awith (awith (parr (parr (dual A) (dual B)) (tens A B)) one) (awith (parr bot one) top)) (awith (awith (aplus one zero) (aplus zero one)) (awith (awith (oc one) (wn one)) (awith (parr (wn A) one) (parr (wn (dual A)) (tens A A)))))].
Proof.
apply (with_r_ext []); cbn_sequent.
{
  apply (with_r_ext []); cbn_sequent.
  {
    apply (with_r_ext []); cbn_sequent.
    {
      apply (parr_r_ext []); cbn_sequent.
      apply (parr_r_ext []); cbn_sequent.
      apply (ex_perm_r [0; 2; 1] [dual A; tens A B; dual B]).
      apply (tens_r_ext [dual A]); cbn_sequent.
      {
        ax_expansion.
      }
      {
        ax_expansion.
      }
    }
    {
      apply one_r_ext.
    }
  }
  {
    apply (with_r_ext []); cbn_sequent.
    {
      apply (parr_r_ext []); cbn_sequent.
      apply (bot_r_ext []); cbn_sequent.
      apply one_r_ext.
    }
    {
      apply (top_r_ext []).
    }
  }
}
{
  apply (with_r_ext []); cbn_sequent.
  {
    apply (with_r_ext []); cbn_sequent.
    {
      apply (plus_r1_ext []); cbn_sequent.
      apply one_r_ext.
    }
    {
      apply (plus_r2_ext []); cbn_sequent.
      apply one_r_ext.
    }
  }
  {
    apply (with_r_ext []); cbn_sequent.
    {
      apply (with_r_ext []); cbn_sequent.
      {
        apply (oc_r_ext [] (one) []); cbn_sequent.
        apply one_r_ext.
      }
      {
        apply (de_r_ext []); cbn_sequent.
        apply one_r_ext.
      }
    }
    {
      apply (with_r_ext []); cbn_sequent.
      {
        apply (parr_r_ext []); cbn_sequent.
        apply (wk_r_ext []); cbn_sequent.
        apply one_r_ext.
      }
      {
        apply (parr_r_ext []); cbn_sequent.
        apply (co_r_ext []); cbn_sequent.
        apply (ex_perm_r [0; 2; 1] [wn (dual A); tens A A; wn (dual A)]).
        apply (tens_r_ext [wn (dual A)]); cbn_sequent.
        {
          apply (de_r_ext []); cbn_sequent.
          ax_expansion.
        }
        {
          apply (de_r_ext [A]); cbn_sequent.
          ax_expansion.
        }
      }
    }
  }
}
Qed.
