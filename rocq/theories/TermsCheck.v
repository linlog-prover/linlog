(* linlog © Fabian Lukas Grubmüller 2026 *)
(* Licensed under the EUPL *)

(** * Terms for the checkers

    What the first-order checkers ([FirstOrderCheck],
    [FirstOrderIntuitionisticCheck]) compute with on terms: their
    equality, decided through an equality of symbols, and the list of
    their eigenvariables; and the facts their soundness proofs use. *)

From Stdlib Require Import Bool List.
From Linlog Require Export Terms.
Import ListNotations.

Section Equality.

Context {symbol : Type} (symbol_eqb : symbol -> symbol -> bool).
Local Notation term := (term symbol).

(** Structural equality of terms, decided through [symbol_eqb]. *)
Fixpoint term_eqb (t u : term) : bool :=
  match t, u with
  | bound n, bound m => Nat.eqb n m
  | eigen a, eigen b => Nat.eqb a b
  | tapp f l, tapp g k =>
      symbol_eqb f g &&
      (fix terms_eqb (l k : list term) : bool :=
         match l, k with
         | [], [] => true
         | t :: l, u :: k => term_eqb t u && terms_eqb l k
         | _, _ => false
         end) l k
  | _, _ => false
  end.

(** Equality of lists of terms. *)
Fixpoint terms_eqb (l k : list term) : bool :=
  match l, k with
  | [], [] => true
  | t :: l, u :: k => term_eqb t u && terms_eqb l k
  | _, _ => false
  end.

(** Induction on terms, with the hypothesis on every argument of an
    application. *)
Lemma term_ind' (P : term -> Prop) :
  (forall n, P (bound n)) -> (forall a, P (eigen a)) ->
  (forall f l, List.Forall P l -> P (tapp f l)) -> forall t, P t.
Proof.
  intros Hb He Ha. fix IH 1. intros [n | a | f l]; [apply Hb | apply He | apply Ha].
  induction l as [| t l IHl]; constructor; [apply IH | exact IHl].
Qed.

(** [symbol_eqb] decides equality. *)
Hypothesis symbol_eqb_spec : forall f g, reflect (f = g) (symbol_eqb f g).

Lemma term_eqb_iff : forall t u, term_eqb t u = true <-> t = u.
Proof.
  induction t as [n | a | f l IH] using term_ind'; destruct u as [m | b | g k]; cbn;
    try (split; [discriminate | congruence]).
  - rewrite PeanoNat.Nat.eqb_eq. split; congruence.
  - rewrite PeanoNat.Nat.eqb_eq. split; congruence.
  - destruct (symbol_eqb_spec f g) as [<- | ne]; cbn;
      [| split; [discriminate | congruence]].
    revert k; induction IH as [| t l Ht _ IHl]; intros [| u k]; cbn;
      try (split; [discriminate | congruence]); [split; reflexivity |].
    rewrite andb_true_iff, Ht, IHl. split.
    + intros [-> E]. injection E as ->. reflexivity.
    + intros E. injection E as -> ->. split; reflexivity.
Qed.

Lemma term_eqb_spec : forall t u, reflect (t = u) (term_eqb t u).
Proof. intros t u. apply iff_reflect. symmetry. apply term_eqb_iff. Qed.

Lemma terms_eqb_spec : forall l k, reflect (l = k) (terms_eqb l k).
Proof.
  induction l as [| t l IH]; intros [| u k]; cbn; try (constructor; congruence).
  destruct (term_eqb_spec t u) as [<- | ne]; cbn; [| constructor; congruence].
  destruct (IH k); constructor; congruence.
Qed.

End Equality.

Section Eigenvariables.

Context {symbol : Type}.
Local Notation term := (term symbol).

(** The eigenvariables of a term, with repetitions. *)
Fixpoint teigens (t : term) : list nat :=
  match t with
  | bound _ => []
  | eigen a => [a]
  | tapp _ l => flat_map teigens l
  end.

Lemma teigens_occurs : forall a (t : term), In a (teigens t) <-> toccurs a t = true.
Proof.
  intros a t. induction t as [n | b | f l IH] using term_ind'; cbn.
  - split; [contradiction | discriminate].
  - rewrite PeanoNat.Nat.eqb_eq. split; [intros [-> | []]; reflexivity | intros ->; left; reflexivity].
  - rewrite in_flat_map, existsb_exists, Forall_forall in *.
    split; intros [t [Hin Ht]]; exists t; split; try exact Hin; apply IH; assumption.
Qed.

Lemma toccurs_tsubst : forall b k (u t : term),
  toccurs b (tsubst k u t) = true -> toccurs b t = true \/ toccurs b u = true.
Proof.
  intros b k u t. induction t as [n | a | f l IH] using term_ind'; cbn.
  - destruct (Nat.eqb n k); cbn; auto.
  - auto.
  - rewrite !existsb_exists, Forall_forall in *. intros [t' [Hin Ht]].
    apply in_map_iff in Hin as [t [<- Hin]].
    destruct (IH t Hin Ht) as [H | H]; [left; exists t; auto | right; exact H].
Qed.

End Eigenvariables.
