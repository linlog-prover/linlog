(* linlog © Fabian Lukas Grubmüller 2026 *)
(* Licensed under the EUPL *)

(** * Soundness of the checker

    A term the checker accepts yields a derivation: [soundness] for any
    calculus [rules] names, and its instances [soundness_classical],
    [soundness_mix] and [soundness_affine]. The proof follows the pass: every state [derive]
    computes is satisfied in the sense of [sat], the sequents it stands
    for are derivable, and the root's state gives the sequent. *)

From Stdlib Require Import Bool List Permutation Lia.
From Linlog Require Export Check.
Import ListNotations.

Section Soundness.

Context {atom : Type} (atom_eqb : atom -> atom -> bool)
  (atom_eqb_spec : forall a b, reflect (a = b) (atom_eqb a b)).
Local Notation formula := (formula atom).
Local Notation eqb_spec := (formula_eqb_spec atom_eqb atom_eqb_spec).
Local Notation eq_dec := (formula_eq_dec atom_eqb atom_eqb_spec).
Local Notation count := (count_occ eq_dec).

(** ** The multiset functions, as permutations *)

Lemma remove_one_perm : forall (A : formula) l l',
  remove_one atom_eqb A l = Some l' -> Permutation l (A :: l').
Proof.
  induction l as [| B l IH]; cbn; intros l' H; [discriminate |].
  destruct (eqb_spec A B) as [<- | _].
  - injection H as <-. reflexivity.
  - destruct (remove_one atom_eqb A l) as [l'' |] eqn:E; cbn in H; [| discriminate].
    injection H as <-. rewrite (IH l'' eq_refl). apply perm_swap.
Qed.

Lemma remove_one_absent : forall (A : formula) l,
  remove_one atom_eqb A l = None -> ~ In A l.
Proof.
  induction l as [| B l IH]; cbn; intros H HI; [exact HI |].
  destruct (eqb_spec A B) as [_ | ne]; [discriminate |].
  destruct HI as [-> | HI]; [exact (ne eq_refl) |].
  destruct (remove_one atom_eqb A l); [discriminate | exact (IH eq_refl HI)].
Qed.

Lemma take_all_perm : forall (l1 l2 l : list formula),
  take_all atom_eqb l1 l2 = Some l -> Permutation l2 (l1 ++ l).
Proof.
  induction l1 as [| A l1 IH]; cbn; intros l2 l H.
  - injection H as <-. reflexivity.
  - destruct (remove_one atom_eqb A l2) as [l2' |] eqn:E; [| discriminate].
    rewrite (remove_one_perm _ _ _ E). apply perm_skip, IH, H.
Qed.

Lemma remove_all_perm : forall (A : formula) l,
  Permutation l (repeat A (count l A) ++ remove_all atom_eqb A l).
Proof.
  unfold remove_all. induction l as [| B l IH]; cbn; [reflexivity |].
  destruct (eq_dec B A) as [e | ne]; destruct (eqb_spec A B) as [e' | ne'];
    try congruence; cbn.
  - subst. apply perm_skip, IH.
  - apply (Permutation_trans (perm_skip B IH)), Permutation_middle.
Qed.

Lemma diff_count : forall (l1 l2 : list formula) x,
  count (diff atom_eqb l2 l1) x = count l2 x - count l1 x.
Proof.
  induction l1 as [| A l1 IH]; cbn; intros l2 x; [lia |].
  destruct (remove_one atom_eqb A l2) as [l2' |] eqn:E; rewrite IH.
  - apply remove_one_perm in E.
    rewrite (Permutation_count_occ eq_dec) in E. rewrite (E x). cbn.
    destruct (eq_dec A x); lia.
  - apply remove_one_absent, (count_occ_not_In eq_dec) in E.
    destruct (eq_dec A x); subst; lia.
Qed.

(** Permutations of lists of formulas, decided by counting every
    formula's occurrences on both sides. *)
Ltac perm :=
  apply (Permutation_count_occ eq_dec); intros ?x;
  repeat rewrite map_app; repeat rewrite app_nil_r; cbn [map];
  repeat first [rewrite count_occ_app | progress cbn [count_occ]];
  repeat rewrite diff_count;
  repeat match goal with
         | |- context [eq_dec ?a ?b] => destruct (eq_dec a b); try subst
         end;
  lia.

(** ** Derived rules of the calculus *)

Variable R : rules.

(** Weakening of a list of [?] formulas. *)
Lemma wk_list : forall (Θ l : list formula), ll R l -> ll R (map wn Θ ++ l).
Proof. induction Θ; cbn; intros; [assumption | apply wk_r; auto]. Qed.

Lemma wn_contract_cons : forall (A : formula) n (l : list formula),
  ll R (wn A :: map wn (repeat A n) ++ l) -> ll R (wn A :: l).
Proof. induction n; cbn; intros l H; [exact H | apply IHn, co_r, H]. Qed.

(** Any number of copies of [?A], none included, contract into one. *)
Lemma wn_contract : forall (A : formula) n (l : list formula),
  ll R (map wn (repeat A n) ++ l) -> ll R (wn A :: l).
Proof.
  intros A [| n] l H; cbn in H; [apply wk_r, H | apply (wn_contract_cons A n), H].
Qed.

(** ** What a state stands for *)

(** The sequents a state stands for are derivable: [⊢ gamma, ?theta, Δ]
    for every [Δ] when the state absorbs, and for [Δ] empty otherwise. *)
Definition sat (theta gamma : list formula) (any : bool) : Prop :=
  forall Δ, (any = false -> Δ = []) -> ll R (gamma ++ map wn theta ++ Δ).

Lemma sat_perm : forall theta gamma gamma' any,
  Permutation gamma gamma' -> sat theta gamma any -> sat theta gamma' any.
Proof.
  intros * HP H Δ HΔ. eapply ex_r; [apply H, HΔ | apply Permutation_app_tail, HP].
Qed.

Lemma take_spec : forall A gamma any gamma',
  take atom_eqb A gamma any = Some gamma' ->
  Permutation gamma (A :: gamma') \/ (any = true /\ gamma' = gamma).
Proof.
  unfold take; intros * H.
  destruct (remove_one atom_eqb A gamma) eqn:E.
  - injection H as <-. left. apply remove_one_perm, E.
  - destruct any; [| discriminate]. injection H as <-. auto.
Qed.

(** Consuming [A] from a satisfied zone leaves a satisfied zone with [A]
    in front, wherever [pre] the rule has put before it. *)
Lemma take_sat : forall pre A theta gamma any gamma',
  take atom_eqb A gamma any = Some gamma' ->
  sat theta (pre ++ gamma) any -> sat theta (pre ++ A :: gamma') any.
Proof.
  intros * T H. destruct (take_spec _ _ _ _ T) as [HP | [-> ->]].
  - apply (sat_perm _ (pre ++ gamma)); [apply Permutation_app_head, HP | exact H].
  - intros Δ _. eapply ex_r; [apply (H (A :: Δ)); easy | perm].
Qed.

(** ** The pass is sound *)

Theorem derive_sound : forall p s,
  derive atom_eqb R p = Some s -> sat (theta s) (gamma s) (any s).
Proof.
  induction p as [a | A B l IHl r IHr | A B q IHq | | q IHq | A B l IHl r IHr
    | A B side q IHq | | A q IHq | A q IHq | A q IHq | A q IHq | l IHl r IHr];
    cbn; intros s H.
  - (* Ax *)
    injection H as <-; cbn. intros Δ HΔ. rewrite (HΔ eq_refl). cbn. apply ax_r.
  - (* Tensor *)
    destruct (derive atom_eqb R l) as [[θl γl al] |] eqn:El; [| discriminate].
    destruct (derive atom_eqb R r) as [[θr γr ar] |] eqn:Er; [| discriminate].
    cbn in H.
    destruct (take atom_eqb A γl al) as [gl |] eqn:Tl; [| discriminate].
    destruct (take atom_eqb B γr ar) as [gr |] eqn:Tr; [| discriminate].
    injection H as <-; cbn.
    specialize (IHl _ eq_refl); specialize (IHr _ eq_refl); cbn in IHl, IHr.
    pose proof (take_sat [] _ _ _ _ _ Tl IHl) as Hl; cbn in Hl.
    pose proof (take_sat [] _ _ _ _ _ Tr IHr) as Hr; cbn in Hr.
    intros Δ HΔ. destruct al.
    + eapply ex_r; [apply tens_r; [apply (Hl Δ); easy | apply (Hr []); easy] | perm].
    + destruct ar.
      * eapply ex_r; [apply tens_r; [apply (Hl []); easy | apply (Hr Δ); easy] | perm].
      * rewrite (HΔ eq_refl).
        eapply ex_r; [apply tens_r; [apply (Hl []); easy | apply (Hr []); easy] | perm].
  - (* Par *)
    destruct (derive atom_eqb R q) as [[θ γ a] |] eqn:E; [| discriminate]. cbn in H.
    destruct (take atom_eqb A γ a) as [g1 |] eqn:T1; [| discriminate].
    destruct (take atom_eqb B g1 a) as [g2 |] eqn:T2; [| discriminate].
    injection H as <-; cbn.
    specialize (IHq _ eq_refl); cbn in IHq.
    pose proof (take_sat [] _ _ _ _ _ T1 IHq) as H1; cbn in H1.
    pose proof (take_sat [A] _ _ _ _ _ T2 H1) as H2; cbn in H2.
    intros Δ HΔ. apply parr_r, H2, HΔ.
  - (* One *)
    injection H as <-; cbn. intros Δ HΔ. rewrite (HΔ eq_refl). cbn. apply one_r.
  - (* Bot *)
    destruct (derive atom_eqb R q) as [[θ γ a] |] eqn:E; [| discriminate].
    injection H as <-; cbn.
    specialize (IHq _ eq_refl); cbn in IHq.
    intros Δ HΔ. apply bot_r, IHq, HΔ.
  - (* With *)
    destruct (derive atom_eqb R l) as [[θl γl al] |] eqn:El; [| discriminate].
    destruct (derive atom_eqb R r) as [[θr γr ar] |] eqn:Er; [| discriminate].
    cbn in H.
    destruct (take atom_eqb A γl al) as [gl |] eqn:Tl; [| discriminate].
    destruct (take atom_eqb B γr ar) as [gr |] eqn:Tr; [| discriminate].
    specialize (IHl _ eq_refl); specialize (IHr _ eq_refl); cbn in IHl, IHr.
    pose proof (take_sat [] _ _ _ _ _ Tl IHl) as Hl; cbn in Hl.
    pose proof (take_sat [] _ _ _ _ _ Tr IHr) as Hr; cbn in Hr.
    unfold join_with in H; cbn in H.
    destruct al, ar; cbn in H.
    + (* both premises absorb: the union of their zones *)
      injection H as <-; cbn. intros Δ _. apply with_r.
      * eapply ex_r; [apply (Hl (diff atom_eqb gr gl ++ map wn θr ++ Δ)); easy | perm].
      * eapply ex_r; [apply (Hr (diff atom_eqb gl gr ++ map wn θl ++ Δ)); easy | perm].
    + (* the left premise absorbs: the right one's zone *)
      destruct (take_all atom_eqb gl gr) as [rest |] eqn:T; [| discriminate].
      injection H as <-; cbn. apply take_all_perm in T.
      intros Δ HΔ. rewrite (HΔ eq_refl). apply with_r.
      * eapply ex_r; [apply (Hl (rest ++ map wn θr)); easy | rewrite T; perm].
      * eapply ex_r; [apply (wk_list θl), (Hr []); easy | perm].
    + (* the right premise absorbs: the left one's zone *)
      destruct (take_all atom_eqb gr gl) as [rest |] eqn:T; [| discriminate].
      injection H as <-; cbn. apply take_all_perm in T.
      intros Δ HΔ. rewrite (HΔ eq_refl). apply with_r.
      * eapply ex_r; [apply (wk_list θr), (Hl []); easy | perm].
      * eapply ex_r; [apply (Hr (rest ++ map wn θl)); easy | rewrite T; perm].
    + (* both exact: the same zone *)
      destruct (take_all atom_eqb gl gr) as [[| r' rest] |] eqn:T;
        [| discriminate | discriminate].
      injection H as <-; cbn. apply take_all_perm in T. rewrite app_nil_r in T.
      intros Δ HΔ. rewrite (HΔ eq_refl). apply with_r.
      * eapply ex_r; [apply (wk_list θr), (Hl []); easy | perm].
      * eapply ex_r; [apply (wk_list θl), (Hr []); easy | rewrite T; perm].
  - (* Plus *)
    destruct (derive atom_eqb R q) as [[θ γ a] |] eqn:E; [| discriminate]. cbn in H.
    specialize (IHq _ eq_refl); cbn in IHq.
    destruct side; cbn in H.
    + destruct (take atom_eqb A γ a) as [g |] eqn:T; [| discriminate].
      injection H as <-; cbn.
      pose proof (take_sat [] _ _ _ _ _ T IHq) as H1; cbn in H1.
      intros Δ HΔ. apply plus_r1, H1, HΔ.
    + destruct (take atom_eqb B γ a) as [g |] eqn:T; [| discriminate].
      injection H as <-; cbn.
      pose proof (take_sat [] _ _ _ _ _ T IHq) as H1; cbn in H1.
      intros Δ HΔ. apply plus_r2, H1, HΔ.
  - (* Top *)
    injection H as <-; cbn. intros Δ _. apply top_r.
  - (* Bang *)
    destruct (derive atom_eqb R q) as [[θ γ a] |] eqn:E; [| discriminate]. cbn in H.
    destruct (take atom_eqb A γ a) as [[| g1 g'] |] eqn:T; [| discriminate | discriminate].
    cbn in H. injection H as <-; cbn.
    specialize (IHq _ eq_refl); cbn in IHq.
    pose proof (take_sat [] _ _ _ _ _ T IHq) as H1; cbn in H1.
    intros Δ HΔ. rewrite (HΔ eq_refl).
    eapply ex_r; [apply (oc_r R A θ); eapply ex_r; [apply (H1 []); easy | perm] | perm].
  - (* Quest *)
    destruct (derive atom_eqb R q) as [[θ γ a] |] eqn:E; [| discriminate].
    injection H as <-; cbn.
    specialize (IHq _ eq_refl); cbn in IHq.
    intros Δ HΔ. apply (wn_contract A (count θ A)).
    eapply ex_r; [apply IHq, HΔ |].
    transitivity (γ ++ map wn (repeat A (count θ A) ++ remove_all atom_eqb A θ) ++ Δ).
    + apply Permutation_app_head, Permutation_app_tail, Permutation_map, remove_all_perm.
    + perm.
  - (* Copy *)
    destruct (derive atom_eqb R q) as [[θ γ a] |] eqn:E; [| discriminate]. cbn in H.
    destruct (take atom_eqb A γ a) as [g |] eqn:T; [| discriminate].
    injection H as <-; cbn.
    specialize (IHq _ eq_refl); cbn in IHq.
    pose proof (take_sat [] _ _ _ _ _ T IHq) as H1; cbn in H1.
    intros Δ HΔ. eapply ex_r; [apply de_r, (H1 Δ), HΔ | perm].
  - (* Weaken *)
    destruct (weakening R || is_wn A) eqn:W; [| discriminate].
    destruct (derive atom_eqb R q) as [[θ γ a] |] eqn:E; [| discriminate].
    injection H as <-; cbn.
    specialize (IHq _ eq_refl); cbn in IHq.
    intros Δ HΔ. apply orb_true_iff in W as [W | W].
    + apply (weakening_r R W), IHq, HΔ.
    + destruct A; cbn in W; try discriminate. apply wk_r, IHq, HΔ.
  - (* Mix *)
    destruct (mix R) eqn:M; [| discriminate].
    destruct (derive atom_eqb R l) as [[θl γl al] |] eqn:El; [| discriminate].
    destruct (derive atom_eqb R r) as [[θr γr ar] |] eqn:Er; [| discriminate].
    injection H as <-; cbn.
    specialize (IHl _ eq_refl); specialize (IHr _ eq_refl); cbn in IHl, IHr.
    intros Δ HΔ. destruct al.
    + eapply ex_r; [apply (mix_r R M); [apply (IHl Δ); easy | apply (IHr []); easy] | perm].
    + destruct ar.
      * eapply ex_r; [apply (mix_r R M); [apply (IHl []); easy | apply (IHr Δ); easy] | perm].
      * rewrite (HΔ eq_refl).
        eapply ex_r; [apply (mix_r R M); [apply (IHl []); easy | apply (IHr []); easy] | perm].
Qed.

(** ** The theorem *)

(** A term the checker accepts for a sequent proves it in the calculus. *)
Theorem soundness : forall p sequent,
  check atom_eqb R p sequent = true -> ll R sequent.
Proof.
  unfold check; intros p Γ H.
  destruct (derive atom_eqb R p) as [[θ γ a] |] eqn:E; [| discriminate].
  destruct θ; cbn in H; [| discriminate].
  pose proof (derive_sound _ _ E) as S; cbn in S.
  destruct (take_all atom_eqb γ Γ) as [[| r rest] |] eqn:T; [| | discriminate];
    apply take_all_perm in T.
  - eapply ex_r; [apply (S []); easy | rewrite T; perm].
  - subst a. eapply ex_r; [apply (S (r :: rest)); easy | rewrite T; perm].
Qed.

End Soundness.

(** The theorem for classical linear logic, the calculus without Mix and
    without weakening. *)
Theorem soundness_classical {atom : Type} (atom_eqb : atom -> atom -> bool)
  (atom_eqb_spec : forall a b, reflect (a = b) (atom_eqb a b)) :
  forall (p : proof atom) (sequent : list (formula atom)),
  check atom_eqb classical p sequent = true -> ll classical sequent.
Proof. exact (soundness atom_eqb atom_eqb_spec classical). Qed.

(** The theorem for linear logic with Mix: binary Mix, no weakening. *)
Theorem soundness_mix {atom : Type} (atom_eqb : atom -> atom -> bool)
  (atom_eqb_spec : forall a b, reflect (a = b) (atom_eqb a b)) :
  forall (p : proof atom) (sequent : list (formula atom)),
  check atom_eqb {| mix := true; weakening := false |} p sequent = true ->
  ll {| mix := true; weakening := false |} sequent.
Proof. exact (soundness atom_eqb atom_eqb_spec _). Qed.

(** The theorem for affine linear logic: weakening of any formula, no
    Mix. *)
Theorem soundness_affine {atom : Type} (atom_eqb : atom -> atom -> bool)
  (atom_eqb_spec : forall a b, reflect (a = b) (atom_eqb a b)) :
  forall (p : proof atom) (sequent : list (formula atom)),
  check atom_eqb {| mix := false; weakening := true |} p sequent = true ->
  ll {| mix := false; weakening := true |} sequent.
Proof. exact (soundness atom_eqb atom_eqb_spec _). Qed.
