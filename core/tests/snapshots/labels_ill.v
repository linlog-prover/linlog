From NanoYalla Require Import macroll.

Lemma certificate (A B : formula) : ll [awith (awith (awith (parr (dual A) (parr (tens A (dual B)) B)) (parr (parr (dual A) (dual B)) (tens B A))) (awith (parr (aplus (dual A) (dual B)) A) (parr (aplus (dual A) (dual B)) B))) (awith (awith (parr (awith (dual A) (dual B)) (aplus B A)) (parr bot one)) (awith (awith (parr top A) top) (awith (parr (wn (dual A)) (tens A A)) (awith (parr (wn (dual A)) (oc A)) (parr (wn (dual A)) one)))))].
Proof.
apply (with_r_ext []); cbn_sequent.
{
  apply (with_r_ext []); cbn_sequent.
  {
    apply (with_r_ext []); cbn_sequent.
    {
      apply (parr_r_ext []); cbn_sequent.
      apply (parr_r_ext [dual A]); cbn_sequent.
      apply (tens_r_ext [dual A]); cbn_sequent.
      {
        ax_expansion.
      }
      {
        ax_expansion.
      }
    }
    {
      apply (parr_r_ext []); cbn_sequent.
      apply (parr_r_ext []); cbn_sequent.
      apply (ex_perm_r [2; 0; 1] [dual B; tens B A; dual A]).
      apply (tens_r_ext [dual B]); cbn_sequent.
      {
        ax_expansion.
      }
      {
        ax_expansion.
      }
    }
  }
  {
    apply (with_r_ext []); cbn_sequent.
    {
      apply (parr_r_ext []); cbn_sequent.
      apply (plus_r1_ext []); cbn_sequent.
      ax_expansion.
    }
    {
      apply (parr_r_ext []); cbn_sequent.
      apply (plus_r2_ext []); cbn_sequent.
      ax_expansion.
    }
  }
}
{
  apply (with_r_ext []); cbn_sequent.
  {
    apply (with_r_ext []); cbn_sequent.
    {
      apply (parr_r_ext []); cbn_sequent.
      apply (with_r_ext []); cbn_sequent.
      {
        apply (plus_r2_ext [dual A]); cbn_sequent.
        ax_expansion.
      }
      {
        apply (plus_r1_ext [dual B]); cbn_sequent.
        ax_expansion.
      }
    }
    {
      apply (parr_r_ext []); cbn_sequent.
      apply (bot_r_ext []); cbn_sequent.
      apply one_r_ext.
    }
  }
  {
    apply (with_r_ext []); cbn_sequent.
    {
      apply (with_r_ext []); cbn_sequent.
      {
        apply (parr_r_ext []); cbn_sequent.
        apply (top_r_ext []).
      }
      {
        apply (top_r_ext []).
      }
    }
    {
      apply (with_r_ext []); cbn_sequent.
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
      {
        apply (with_r_ext []); cbn_sequent.
        {
          apply (parr_r_ext []); cbn_sequent.
          apply (oc_r_ext [dual A] (A) []); cbn_sequent.
          apply (de_r_ext []); cbn_sequent.
          ax_expansion.
        }
        {
          apply (parr_r_ext []); cbn_sequent.
          apply (wk_r_ext []); cbn_sequent.
          apply one_r_ext.
        }
      }
    }
  }
}
Qed.
