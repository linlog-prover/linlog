(* linlog © Fabian Lukas Grubmüller 2026 *)
(* Licensed under the EUPL *)

(** * The bridge to Yalla's standalone kernel of intuitionistic linear logic

    [ill intuitionistic] derives exactly the sequents that Yalla's
    [nanoill.ill] derives, over [nanoill.v]'s own definitions, imported
    unchanged. Its atoms are [nat], so the statements are about
    [iformula nat]; the library's calculus is the same at every atom
    type. The two formula types have the same constructors and the
    translations below map each to its namesake, so they are inverse
    bijections. [nanoill.v]'s derivations are data (an inductive declared
    in [Type], which Rocq lowers to [Set]) and [ill] is in [Prop], so the
    statements say that a derivation of [nanoill.ill] exists, with
    [inhabited]: they equate derivability, not derivations. Exchange is one [Permutation]
    rule here and a transposition of neighbours there, which is the only
    difference between the two calculi. *)

From Stdlib Require Import List Permutation.
From Linlog Require Import Intuitionistic.
From Yalla.MicroYalla Require nanoill.
Import ListNotations.

(** A formula of linlog's library as [nanoill.v]'s. *)
Fixpoint to_nanoill (A : iformula nat) : nanoill.iformula :=
  match A with
  | ivar a => nanoill.ivar a
  | ione => nanoill.ione
  | itens A B => nanoill.itens (to_nanoill A) (to_nanoill B)
  | ilmap A B => nanoill.ilmap (to_nanoill A) (to_nanoill B)
  | itop => nanoill.itop
  | iwith A B => nanoill.iwith (to_nanoill A) (to_nanoill B)
  | izero => nanoill.izero
  | iplus A B => nanoill.iplus (to_nanoill A) (to_nanoill B)
  | ioc A => nanoill.ioc (to_nanoill A)
  end.

(** A formula of [nanoill.v] as linlog's library's. *)
Fixpoint of_nanoill (A : nanoill.iformula) : iformula nat :=
  match A with
  | nanoill.ivar a => ivar a
  | nanoill.ione => ione
  | nanoill.itens A B => itens (of_nanoill A) (of_nanoill B)
  | nanoill.ilmap A B => ilmap (of_nanoill A) (of_nanoill B)
  | nanoill.itop => itop
  | nanoill.iwith A B => iwith (of_nanoill A) (of_nanoill B)
  | nanoill.izero => izero
  | nanoill.iplus A B => iplus (of_nanoill A) (of_nanoill B)
  | nanoill.ioc A => ioc (of_nanoill A)
  end.

Lemma of_to_nanoill : forall A, of_nanoill (to_nanoill A) = A.
Proof. induction A; cbn; rewrite ?IHA, ?IHA1, ?IHA2; reflexivity. Qed.

Lemma to_of_nanoill : forall A, to_nanoill (of_nanoill A) = A.
Proof. induction A; cbn; rewrite ?IHA, ?IHA1, ?IHA2; reflexivity. Qed.

Lemma map_of_to_nanoill : forall l, map of_nanoill (map to_nanoill l) = l.
Proof. induction l; cbn; rewrite ?of_to_nanoill; congruence. Qed.

Lemma map_to_of_nanoill : forall l, map to_nanoill (map of_nanoill l) = l.
Proof. induction l; cbn; rewrite ?to_of_nanoill; congruence. Qed.

(** The [!] context of a promotion, translated. *)
Lemma to_nanoill_oc : forall l,
  map to_nanoill (map ioc l) = nanoill.map nanoill.ioc (map to_nanoill l).
Proof. induction l; cbn; congruence. Qed.

Lemma of_nanoill_oc : forall l,
  map of_nanoill (nanoill.map nanoill.ioc l) = map ioc (map of_nanoill l).
Proof. induction l; cbn; congruence. Qed.

(** [nanoill.v]'s exchange of neighbours gives any permutation. *)
Lemma nanoill_ex : forall l1 l2, Permutation l1 l2 ->
  forall pre C, nanoill.ill (pre ++ l1) C -> inhabited (nanoill.ill (pre ++ l2) C).
Proof.
  intros l1 l2 P; induction P; intros pre C H.
  - exact (inhabits H).
  - assert (E : forall m, pre ++ x :: m = (pre ++ [x]) ++ m)
      by (intro; rewrite <- app_assoc; reflexivity).
    rewrite E in H |- *. exact (IHP _ _ H).
  - exact (inhabits (nanoill.ex_t_ir pre l y x C H)).
  - destruct (IHP1 pre C H) as [H']. exact (IHP2 pre C H').
Qed.

(** The derivations of [ill intuitionistic] give derivations of
    [nanoill.ill]. *)
Lemma to_nanoill_ill : forall G C,
  ill intuitionistic G C -> inhabited (nanoill.ill (map to_nanoill G) (to_nanoill C)).
Proof.
  intros G C H; induction H; cbn in *;
    repeat match goal with Hi : inhabited _ |- _ => destruct Hi end;
    rewrite ?map_app in *; try discriminate.
  - exact (inhabits (nanoill.ax_ir a)).
  - eapply (nanoill_ex _ _ (Permutation_map to_nanoill H0) []); eassumption.
  - exact (inhabits nanoill.one_irr).
  - apply inhabits; eapply nanoill.one_ilr; eassumption.
  - apply inhabits; eapply nanoill.tens_irr; eassumption.
  - apply inhabits; eapply nanoill.tens_ilr; eassumption.
  - apply inhabits; eapply nanoill.lmap_irr; eassumption.
  - apply inhabits; eapply nanoill.lmap_ilr; eassumption.
  - exact (inhabits (nanoill.top_irr _)).
  - apply inhabits; eapply nanoill.with_irr; eassumption.
  - apply inhabits; eapply nanoill.with_ilr1; eassumption.
  - apply inhabits; eapply nanoill.with_ilr2; eassumption.
  - exact (inhabits (nanoill.zero_ilr _ _)).
  - apply inhabits; eapply nanoill.plus_irr1; eassumption.
  - apply inhabits; eapply nanoill.plus_irr2; eassumption.
  - apply inhabits; eapply nanoill.plus_ilr; eassumption.
  - rewrite to_nanoill_oc in *. apply inhabits; eapply nanoill.oc_irr; eassumption.
  - apply inhabits; eapply nanoill.de_ilr; eassumption.
  - apply inhabits; eapply nanoill.wk_ilr; eassumption.
  - apply inhabits; eapply nanoill.co_ilr; eassumption.
Qed.

(** The derivations of [nanoill.ill] are derivations of [ill R], in
    either calculus. *)
Lemma of_nanoill_ill : forall R l D,
  nanoill.ill l D -> ill R (map of_nanoill l) (of_nanoill D).
Proof.
  intros R l D H; induction H; cbn in *; rewrite ?map_app in *.
  - apply ax_ir.
  - eapply ex_ir; [exact IHill |]. cbn. apply Permutation_app_head, perm_swap.
  - apply one_irr.
  - apply one_ilr; assumption.
  - apply tens_irr; assumption.
  - apply tens_ilr; assumption.
  - apply lmap_irr; assumption.
  - apply lmap_ilr; assumption.
  - apply top_irr.
  - apply with_irr; assumption.
  - apply with_ilr1; assumption.
  - apply with_ilr2; assumption.
  - apply zero_ilr.
  - apply plus_irr1; assumption.
  - apply plus_irr2; assumption.
  - apply plus_ilr; assumption.
  - rewrite of_nanoill_oc in *. apply oc_irr; assumption.
  - apply de_ilr; assumption.
  - apply wk_ilr; assumption.
  - apply co_ilr; assumption.
Qed.

(** [ill intuitionistic] derives a sequent exactly when [nanoill.ill]
    derives its translation. *)
Theorem ill_iff_nanoill : forall (G : list (iformula nat)) C,
  ill intuitionistic G C <-> inhabited (nanoill.ill (map to_nanoill G) (to_nanoill C)).
Proof.
  split; [apply to_nanoill_ill |]. intros [H].
  pose proof (of_nanoill_ill intuitionistic _ _ H) as H'.
  rewrite map_of_to_nanoill, of_to_nanoill in H'. exact H'.
Qed.

(** The same over [nanoill.v]'s sequents: [nanoill.ill] derives a
    sequent exactly when [ill intuitionistic] derives the translation. *)
Theorem nanoill_iff_ill : forall (l : list nanoill.iformula) D,
  inhabited (nanoill.ill l D) <-> ill intuitionistic (map of_nanoill l) (of_nanoill D).
Proof.
  intros l D. rewrite ill_iff_nanoill, map_to_of_nanoill, to_of_nanoill. reflexivity.
Qed.
