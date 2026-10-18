(* linlog © Fabian Lukas Grubmüller 2026 *)
(* Licensed under the EUPL *)

(** * The bridge to NanoYalla

    [ll classical] derives exactly the sequents that NanoYalla's kernel
    [nanoll.ll] derives, over NanoYalla's own definitions, imported
    unchanged (its atoms are [nat]). The two formula types have the same
    constructors and the translations below map each to its namesake, so
    they are inverse bijections. NanoYalla's derivations are in [Type]
    and [ll] is in [Prop], so the statements say that a NanoYalla
    derivation exists, with [inhabited]. Exchange is one [Permutation]
    rule here and a transposition of neighbours there, which is the only
    difference between the two calculi. *)

From Stdlib Require Import List Permutation.
From Linlog Require Import Calculus.
From NanoYalla Require nanoll.
Import ListNotations.

(** A formula of linlog's library as NanoYalla's. *)
Fixpoint to_nanoyalla (A : formula nat) : nanoll.formula :=
  match A with
  | var a => nanoll.var a
  | covar a => nanoll.covar a
  | one => nanoll.one
  | bot => nanoll.bot
  | top => nanoll.top
  | zero => nanoll.zero
  | tens A B => nanoll.tens (to_nanoyalla A) (to_nanoyalla B)
  | parr A B => nanoll.parr (to_nanoyalla A) (to_nanoyalla B)
  | awith A B => nanoll.awith (to_nanoyalla A) (to_nanoyalla B)
  | aplus A B => nanoll.aplus (to_nanoyalla A) (to_nanoyalla B)
  | oc A => nanoll.oc (to_nanoyalla A)
  | wn A => nanoll.wn (to_nanoyalla A)
  end.

(** A formula of NanoYalla as linlog's library's. *)
Fixpoint of_nanoyalla (A : nanoll.formula) : formula nat :=
  match A with
  | nanoll.var a => var a
  | nanoll.covar a => covar a
  | nanoll.one => one
  | nanoll.bot => bot
  | nanoll.top => top
  | nanoll.zero => zero
  | nanoll.tens A B => tens (of_nanoyalla A) (of_nanoyalla B)
  | nanoll.parr A B => parr (of_nanoyalla A) (of_nanoyalla B)
  | nanoll.awith A B => awith (of_nanoyalla A) (of_nanoyalla B)
  | nanoll.aplus A B => aplus (of_nanoyalla A) (of_nanoyalla B)
  | nanoll.oc A => oc (of_nanoyalla A)
  | nanoll.wn A => wn (of_nanoyalla A)
  end.

Lemma of_to_nanoyalla : forall A, of_nanoyalla (to_nanoyalla A) = A.
Proof. induction A; cbn; rewrite ?IHA, ?IHA1, ?IHA2; reflexivity. Qed.

Lemma to_of_nanoyalla : forall A, to_nanoyalla (of_nanoyalla A) = A.
Proof. induction A; cbn; rewrite ?IHA, ?IHA1, ?IHA2; reflexivity. Qed.

(** NanoYalla's [map] is the standard library's. *)
Lemma nanoll_map : forall (X Y : Type) (f : X -> Y) l, nanoll.map f l = map f l.
Proof. induction l; cbn; congruence. Qed.

(** NanoYalla's derivations are derivations of [ll classical]. *)
Lemma of_nanoll : forall l, nanoll.ll l -> ll classical (map of_nanoyalla l).
Proof.
  intros l H; induction H; cbn in *; rewrite ?map_app in *; cbn in *.
  - apply ax_r.
  - eapply ex_r; [exact IHll |]. apply Permutation_app_head, perm_swap.
  - apply one_r.
  - apply bot_r; assumption.
  - apply tens_r; assumption.
  - apply parr_r; assumption.
  - apply top_r.
  - apply plus_r1; assumption.
  - apply plus_r2; assumption.
  - apply with_r; assumption.
  - rewrite nanoll_map, map_map in *. cbn in *.
    replace (map (fun x => wn (of_nanoyalla x)) l) with (map wn (map of_nanoyalla l)) in *
      by (rewrite map_map; reflexivity).
    apply oc_r; assumption.
  - apply de_r; assumption.
  - apply wk_r; assumption.
  - apply co_r; assumption.
Qed.

(** NanoYalla's exchange of neighbours gives any permutation. *)
Lemma nanoll_ex : forall l1 l2, Permutation l1 l2 ->
  forall pre, nanoll.ll (pre ++ l1) -> inhabited (nanoll.ll (pre ++ l2)).
Proof.
  intros l1 l2 P; induction P; intros pre H.
  - exact (inhabits H).
  - specialize (IHP (pre ++ [x])). rewrite <- !app_assoc in IHP. exact (IHP H).
  - exact (inhabits (nanoll.ex_t_r pre l y x H)).
  - destruct (IHP1 pre H) as [H']. exact (IHP2 pre H').
Qed.

(** The derivations of [ll classical] give NanoYalla derivations. *)
Lemma to_nanoll : forall l, ll classical l -> inhabited (nanoll.ll (map to_nanoyalla l)).
Proof.
  intros l H; induction H; cbn in *;
    repeat match goal with Hi : inhabited _ |- _ => destruct Hi end;
    rewrite ?map_app in *; try discriminate.
  - exact (inhabits (nanoll.ax_r a)).
  - eapply (nanoll_ex _ _ (Permutation_map to_nanoyalla H0) []); eassumption.
  - exact (inhabits nanoll.one_r).
  - apply inhabits; eapply nanoll.bot_r; eassumption.
  - exact (inhabits (nanoll.top_r _)).
  - apply inhabits; eapply nanoll.tens_r; eassumption.
  - apply inhabits; eapply nanoll.parr_r; eassumption.
  - apply inhabits; eapply nanoll.with_r; eassumption.
  - apply inhabits; eapply nanoll.plus_r1; eassumption.
  - apply inhabits; eapply nanoll.plus_r2; eassumption.
  - rewrite map_map in *. cbn in *.
    replace (map (fun x => nanoll.wn (to_nanoyalla x)) l)
      with (nanoll.map nanoll.wn (map to_nanoyalla l)) in *
      by (rewrite nanoll_map, map_map; reflexivity).
    apply inhabits; eapply nanoll.oc_r; eassumption.
  - apply inhabits; eapply nanoll.de_r; eassumption.
  - apply inhabits; eapply nanoll.wk_r; eassumption.
  - apply inhabits; eapply nanoll.co_r; eassumption.
Qed.

(** [ll classical] derives a sequent exactly when NanoYalla's kernel
    derives its translation. *)
Theorem ll_iff_nanoll : forall l : list (formula nat),
  ll classical l <-> inhabited (nanoll.ll (map to_nanoyalla l)).
Proof.
  split; [apply to_nanoll |]. intros [H].
  pose proof (of_nanoll _ H) as H'. rewrite map_map in H'.
  erewrite map_ext in H' by apply of_to_nanoyalla. rewrite map_id in H'. exact H'.
Qed.

(** The same over NanoYalla's sequents: its kernel derives a sequent
    exactly when [ll classical] derives the translation. *)
Theorem nanoll_iff_ll : forall l : list nanoll.formula,
  inhabited (nanoll.ll l) <-> ll classical (map of_nanoyalla l).
Proof.
  intro l. rewrite ll_iff_nanoll, map_map.
  erewrite map_ext by apply to_of_nanoyalla. rewrite map_id. reflexivity.
Qed.
