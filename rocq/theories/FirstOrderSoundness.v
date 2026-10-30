(* linlog © Fabian Lukas Grubmüller 2026 *)
(* Licensed under the EUPL *)

(** * Soundness of the first-order checker

    A term the checker accepts yields a derivation: [soundness] for any
    calculus [Calculus.rules] names, cut included, and its instances
    [soundness_classical], [soundness_mix] and [soundness_affine], the
    calculi without cut. The proof follows the second pass, as the
    propositional one does ([Soundness]), with one more invariant handed
    from the root to the leaves: [good i s] says that no eigenvariable
    [i] holds stands in the set a subterm carries or in its conclusion.
    The root's check gives it for the root and the [∀] steps' eigenvariables;
    every rule passes it to its premises, since a formula of a premise is a
    formula of the conclusion, a subformula of one, an instance at a
    witness whose eigenvariables the set holds, or the instance at its
    own eigenvariable, which the first pass keeps out of the premise's
    [i]. At a [∀] step it is the side condition of the rule, for the
    absorbed context of a [⊤] above too, which every context handed up
    avoids. *)

From Stdlib Require Import Bool List Permutation Lia.
From Linlog Require Calculus.
From Linlog Require Export FirstOrderCheck.
Import ListNotations.

Section Soundness.

Context {symbol : Type} (symbol_eqb : symbol -> symbol -> bool)
  (symbol_eqb_spec : forall f g, reflect (f = g) (symbol_eqb f g)).
Local Notation term := (term symbol).
Local Notation formula := (formula symbol).
Local Notation state := (state symbol).
Local Notation eqb_spec := (formula_eqb_spec symbol_eqb symbol_eqb_spec).
Local Notation eq_dec := (formula_eq_dec symbol_eqb symbol_eqb_spec).
Local Notation count := (count_occ eq_dec).

(** ** The multiset functions, as permutations *)

Lemma remove_one_perm : forall (A : formula) l l',
  remove_one symbol_eqb A l = Some l' -> Permutation l (A :: l').
Proof.
  induction l as [| B l IH]; cbn; intros l' H; [discriminate |].
  destruct (eqb_spec A B) as [<- | _].
  - injection H as <-. reflexivity.
  - destruct (remove_one symbol_eqb A l) as [l'' |] eqn:E; cbn in H; [| discriminate].
    injection H as <-. rewrite (IH l'' eq_refl). apply perm_swap.
Qed.

Lemma remove_one_absent : forall (A : formula) l,
  remove_one symbol_eqb A l = None -> ~ In A l.
Proof.
  induction l as [| B l IH]; cbn; intros H HI; [exact HI |].
  destruct (eqb_spec A B) as [_ | ne]; [discriminate |].
  destruct HI as [-> | HI]; [exact (ne eq_refl) |].
  destruct (remove_one symbol_eqb A l); [discriminate | exact (IH eq_refl HI)].
Qed.

Lemma take_all_perm : forall (l1 l2 l : list formula),
  take_all symbol_eqb l1 l2 = Some l -> Permutation l2 (l1 ++ l).
Proof.
  induction l1 as [| A l1 IH]; cbn; intros l2 l H.
  - injection H as <-. reflexivity.
  - destruct (remove_one symbol_eqb A l2) as [l2' |] eqn:E; [| discriminate].
    rewrite (remove_one_perm _ _ _ E). apply perm_skip, IH, H.
Qed.

Lemma remove_all_perm : forall (A : formula) l,
  Permutation l (repeat A (count l A) ++ remove_all symbol_eqb A l).
Proof.
  unfold remove_all. induction l as [| B l IH]; cbn; [reflexivity |].
  destruct (eq_dec B A) as [e | ne]; destruct (eqb_spec A B) as [e' | ne'];
    try congruence; cbn.
  - subst. apply perm_skip, IH.
  - apply (Permutation_trans (perm_skip B IH)), Permutation_middle.
Qed.

Lemma diff_count : forall (l1 l2 : list formula) x,
  count (diff symbol_eqb l2 l1) x = count l2 x - count l1 x.
Proof.
  induction l1 as [| A l1 IH]; cbn; intros l2 x; [lia |].
  destruct (remove_one symbol_eqb A l2) as [l2' |] eqn:E; rewrite IH.
  - apply remove_one_perm in E.
    rewrite (Permutation_count_occ eq_dec) in E. rewrite (E x). cbn.
    destruct (eq_dec A x); lia.
  - apply remove_one_absent, (count_occ_not_In eq_dec) in E.
    destruct (eq_dec A x); subst; lia.
Qed.

(** A formula of [l2] is in [l1] or in what is left of [l2] without it. *)
Lemma diff_in : forall (l1 l2 : list formula) x,
  In x l2 -> In x l1 \/ In x (diff symbol_eqb l2 l1).
Proof.
  intros l1 l2 x H.
  destruct (in_dec eq_dec x l1) as [I | N]; [left; exact I | right].
  apply (count_occ_In eq_dec). rewrite diff_count.
  apply (count_occ_In eq_dec) in H. apply (count_occ_not_In eq_dec) in N. lia.
Qed.

(** What is left of [l2] is in [l2]. *)
Lemma diff_incl : forall (l1 l2 : list formula) x,
  In x (diff symbol_eqb l2 l1) -> In x l2.
Proof.
  intros l1 l2 x H. apply (count_occ_In eq_dec) in H. rewrite diff_count in H.
  apply (count_occ_In eq_dec). lia.
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

(** ** Eigenvariables *)

Lemma feigens_occurs : forall a (A : formula), In a (feigens A) <-> occurs a A = true.
Proof.
  intros a A. induction A; cbn;
    try (rewrite in_app_iff, orb_true_iff; tauto); try tauto; try (split; [contradiction | discriminate]).
  all: rewrite in_flat_map, existsb_exists;
    split; intros [t [Hin Ht]]; exists t; split; try exact Hin; apply teigens_occurs; assumption.
Qed.

Lemma occurs_subst : forall b k u (A : formula),
  occurs b (subst k u A) = true -> occurs b A = true \/ toccurs b u = true.
Proof.
  intros b k u A. revert k. induction A; cbn; intros k H; try discriminate;
    try (apply orb_true_iff in H as [H | H];
         [destruct (IHA1 _ H) | destruct (IHA2 _ H)]; rewrite ?orb_true_iff; tauto);
    try exact (IHA _ H); try exact (IHA _ H).
  all: rewrite existsb_exists in *; destruct H as [t' [Hin Ht]];
    apply in_map_iff in Hin as [t [<- Hin]];
    destruct (toccurs_tsubst _ _ _ _ Ht); [left; exists t; auto | right; assumption].
Qed.

Lemma occurs_dual : forall b (A : formula), occurs b (dual A) = occurs b A.
Proof. induction A; cbn; congruence. Qed.

(** ** Freshness *)

Lemma fresh_nil : forall b, fresh (symbol := symbol) b [].
Proof. intros b A []. Qed.

Lemma fresh_cons_iff : forall b (A : formula) l,
  fresh b (A :: l) <-> occurs b A = false /\ fresh b l.
Proof.
  unfold fresh; split.
  - intros H. split; [apply H; left; reflexivity | intros X HX; apply H; right; exact HX].
  - intros [HA Hl] X [<- | HX]; [exact HA | apply Hl, HX].
Qed.

Lemma fresh_app_iff : forall b (l1 l2 : list formula),
  fresh b (l1 ++ l2) <-> fresh b l1 /\ fresh b l2.
Proof.
  unfold fresh; split.
  - intros H. split; intros X HX; apply H, in_or_app; auto.
  - intros [H1 H2] X HX. apply in_app_or in HX as [HX | HX]; auto.
Qed.

Lemma fresh_wn_iff : forall b (l : list formula), fresh b (map wn l) <-> fresh b l.
Proof.
  unfold fresh; split.
  - intros H X HX. apply (H (wn X)), in_map, HX.
  - intros H X HX. apply in_map_iff in HX as [Y [<- HY]]. cbn. apply H, HY.
Qed.

Lemma fresh_incl : forall b (l l' : list formula),
  (forall X, In X l -> In X l') -> fresh b l' -> fresh b l.
Proof. unfold fresh; auto. Qed.

Lemma fresh_perm : forall b (l l' : list formula),
  Permutation l' l -> fresh b l' -> fresh b l.
Proof.
  intros * P H. apply (fresh_incl _ _ l'); [| exact H].
  intros X HX. apply (Permutation_in _ (Permutation_sym P) HX).
Qed.

Lemma take_in : forall (A : formula) gamma any g X,
  take symbol_eqb A gamma any = Some g -> In X gamma -> X = A \/ In X g.
Proof.
  unfold take; intros * H HX.
  destruct (remove_one symbol_eqb A gamma) eqn:E.
  - injection H as <-. apply remove_one_perm in E.
    apply (Permutation_in _ E) in HX as [-> | HX]; auto.
  - destruct any; [| discriminate]. injection H as <-. auto.
Qed.

(** What a premise's zone held is fresh when what the rule left of it
    and the formula it took are. *)
Lemma fresh_take : forall b (A : formula) gamma any g,
  take symbol_eqb A gamma any = Some g -> occurs b A = false -> fresh b g -> fresh b gamma.
Proof.
  intros * T HA Hg X HX. destruct (take_in _ _ _ _ _ T HX) as [-> | HX']; auto.
Qed.

(** Breaks the freshness and the absences in the context into those of
    single formulas and lists. *)
Ltac facts :=
  repeat match goal with
  | H : fresh _ (_ ++ _) |- _ => apply (proj1 (fresh_app_iff _ _ _)) in H as [? ?]
  | H : fresh _ (_ :: _) |- _ => apply (proj1 (fresh_cons_iff _ _ _)) in H as [? ?]
  | H : fresh _ (map wn _) |- _ => apply (proj1 (fresh_wn_iff _ _)) in H
  | H : occurs _ _ = false |- _ => progress cbn [occurs] in H
  | H : (_ || _) = false |- _ => apply orb_false_iff in H as [? ?]
  | H : ~ In _ (_ ++ _) |- _ => rewrite in_app_iff in H; apply Decidable.not_or in H as [? ?]
  end.

(** Splits a freshness goal into those of single formulas and lists. *)
Ltac fresh_split :=
  repeat first
    [ apply (proj2 (fresh_app_iff _ _ _)); split
    | apply (proj2 (fresh_cons_iff _ _ _)); split
    | apply (proj2 (fresh_wn_iff _ _))
    | progress rewrite map_app ].

(** ** Derived rules of the calculus *)

Variable R : Calculus.rules.

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
    for every [Δ] in which no eigenvariable of [i] occurs when the state
    absorbs, and for [Δ] empty otherwise. *)
Definition sat (i : list nat) (theta gamma : list formula) (any : bool) : Prop :=
  forall Δ, (any = false -> Δ = []) -> (forall b, In b i -> fresh b Δ) ->
  ll R (gamma ++ map wn theta ++ Δ).

(** No eigenvariable of [i] stands in the set [s] carries or in the
    sequent [⊢ ?theta, gamma] it proves: what the rules below a subterm
    guarantee for the eigenvariables it introduces. *)
Definition good (i : list nat) (s : state) : Prop :=
  forall b, In b i -> ~ In b (eigens s) /\ fresh b (gamma s ++ map wn (theta s)).

Lemma good_of : forall i i' s s',
  (forall b, In b i' -> In b i) -> good i s ->
  (forall b, In b i' -> ~ In b (eigens s) -> fresh b (gamma s ++ map wn (theta s)) ->
     ~ In b (eigens s') /\ fresh b (gamma s' ++ map wn (theta s'))) ->
  good i' s'.
Proof. intros * I G H b Hb. destruct (G b (I b Hb)). auto. Qed.

Lemma sat_perm : forall i theta gamma gamma' any,
  Permutation gamma gamma' -> sat i theta gamma any -> sat i theta gamma' any.
Proof.
  intros * HP H Δ HΔ HD. eapply ex_r; [apply (H Δ HΔ HD) | apply Permutation_app_tail, HP].
Qed.

Lemma take_spec : forall A gamma any gamma',
  take symbol_eqb A gamma any = Some gamma' ->
  Permutation gamma (A :: gamma') \/ (any = true /\ gamma' = gamma).
Proof.
  unfold take; intros * H.
  destruct (remove_one symbol_eqb A gamma) eqn:E.
  - injection H as <-. left. apply remove_one_perm, E.
  - destruct any; [| discriminate]. injection H as <-. auto.
Qed.

(** Consuming [A] from a satisfied zone leaves a satisfied zone with [A]
    in front, wherever [pre] the rule has put before it; where a [⊤]
    absorbs [A], [A] must be fresh as every absorbed formula is. *)
Lemma take_sat : forall i pre A theta gamma any gamma',
  take symbol_eqb A gamma any = Some gamma' ->
  (forall b, In b i -> occurs b A = false) ->
  sat i theta (pre ++ gamma) any -> sat i theta (pre ++ A :: gamma') any.
Proof.
  intros * T HA H. destruct (take_spec _ _ _ _ T) as [HP | [-> ->]].
  - apply (sat_perm _ _ (pre ++ gamma)); [apply Permutation_app_head, HP | exact H].
  - intros Δ _ HD. eapply ex_r; [apply (H (A :: Δ)); [easy |] | perm].
    intros b Hb. apply fresh_cons_iff. auto.
Qed.

(** Hands a context to a premise's [sat]: fresh for the premise's
    eigenvariables, which are among the conclusion's. *)
Ltac delta :=
  intros ?b ?Hb;
  first
    [ apply fresh_nil
    | match goal with HD : forall b, In b _ -> fresh b ?Δ |- fresh _ ?Δ =>
        apply HD; rewrite ?in_app_iff; cbn; auto
      end ].

(** ** The pass is sound *)

Theorem derive_sound : forall p i s,
  introduced p = Some i -> derive symbol_eqb R p = Some s -> good i s ->
  sat i (theta s) (gamma s) (any s).
Proof.
  induction p as [a l | A B l IHl r IHr | A B q IHq | | q IHq | A B l IHl r IHr
    | A B side q IHq | | A q IHq | A q IHq | A q IHq | A q IHq | l IHl r IHr
    | A l IHl r IHr | A a q IHq | A t q IHq];
    cbn; intros i s I H G.
  - (* Ax *)
    injection H as <-; cbn. intros Δ HΔ _. rewrite (HΔ eq_refl). cbn. apply ax_r.
  - (* Tensor *)
    destruct (introduced l) as [il |] eqn:Il; [| discriminate].
    destruct (introduced r) as [ir |] eqn:Ir; [| discriminate].
    injection I as <-.
    destruct (derive symbol_eqb R l) as [[θl γl al El] |] eqn:Dl; [| discriminate].
    destruct (derive symbol_eqb R r) as [[θr γr ar Er] |] eqn:Dr; [| discriminate].
    cbn in H.
    destruct (take symbol_eqb A γl al) as [gl |] eqn:Tl; [| discriminate].
    destruct (take symbol_eqb B γr ar) as [gr |] eqn:Tr; [| discriminate].
    injection H as <-; cbn.
    assert (Gl : good il (State θl γl al El)).
    { apply (good_of _ _ _ _ (fun b H => in_or_app _ _ _ (or_introl H)) G).
      cbn; intros b _ HE HF; rewrite map_app in HF; facts.
      split; [assumption | fresh_split; [eapply fresh_take |]; eassumption]. }
    assert (Gr : good ir (State θr γr ar Er)).
    { apply (good_of _ _ _ _ (fun b H => in_or_app _ _ _ (or_intror H)) G).
      cbn; intros b _ HE HF; rewrite map_app in HF; facts.
      split; [assumption | fresh_split; [eapply fresh_take |]; eassumption]. }
    cbn in G.
    specialize (IHl _ _ eq_refl eq_refl Gl); specialize (IHr _ _ eq_refl eq_refl Gr);
      cbn in IHl, IHr.
    assert (FA : forall b, In b il -> occurs b A = false).
    { intros b Hb. destruct (G b (in_or_app _ _ _ (or_introl Hb))) as [_ HF]. cbn in HF. facts.
      assumption. }
    assert (FB : forall b, In b ir -> occurs b B = false).
    { intros b Hb. destruct (G b (in_or_app _ _ _ (or_intror Hb))) as [_ HF]. cbn in HF. facts.
      assumption. }
    pose proof (take_sat _ [] _ _ _ _ _ Tl FA IHl) as Hl; cbn in Hl.
    pose proof (take_sat _ [] _ _ _ _ _ Tr FB IHr) as Hr; cbn in Hr.
    intros Δ HΔ HD. destruct al.
    + eapply ex_r; [apply tens_r; [apply (Hl Δ) | apply (Hr [])]; first [easy | delta] | perm].
    + destruct ar.
      * eapply ex_r; [apply tens_r; [apply (Hl []) | apply (Hr Δ)]; first [easy | delta] | perm].
      * rewrite (HΔ eq_refl).
        eapply ex_r; [apply tens_r; [apply (Hl []) | apply (Hr [])]; first [easy | delta] | perm].
  - (* Par *)
    destruct (derive symbol_eqb R q) as [[θ γ a E] |] eqn:D; [| discriminate]. cbn in H.
    destruct (take symbol_eqb A γ a) as [g1 |] eqn:T1; [| discriminate].
    destruct (take symbol_eqb B g1 a) as [g2 |] eqn:T2; [| discriminate].
    injection H as <-; cbn.
    assert (Gq : good i (State θ γ a E)).
    { apply (good_of _ _ _ _ (fun b H => H) G).
      cbn; intros b _ HE HF; facts.
      split; [assumption | fresh_split; [| assumption]].
      eapply fresh_take; [eassumption | assumption |].
      eapply fresh_take; eassumption. }
    specialize (IHq _ _ I eq_refl Gq); cbn in IHq.
    assert (FA : forall b, In b i -> occurs b A = false /\ occurs b B = false).
    { intros b Hb. destruct (G b Hb) as [_ HF]. cbn in HF. facts. auto. }
    pose proof (take_sat _ [] _ _ _ _ _ T1 (fun b H => proj1 (FA b H)) IHq) as H1; cbn in H1.
    pose proof (take_sat _ [A] _ _ _ _ _ T2 (fun b H => proj2 (FA b H)) H1) as H2; cbn in H2.
    intros Δ HΔ HD. apply parr_r, H2; assumption.
  - (* One *)
    injection H as <-; cbn. intros Δ HΔ _. rewrite (HΔ eq_refl). cbn. apply one_r.
  - (* Bot *)
    destruct (derive symbol_eqb R q) as [[θ γ a E] |] eqn:D; [| discriminate].
    injection H as <-; cbn.
    assert (Gq : good i (State θ γ a E)).
    { apply (good_of _ _ _ _ (fun b H => H) G).
      cbn; intros b _ HE HF; facts. split; [assumption | fresh_split; assumption]. }
    specialize (IHq _ _ I eq_refl Gq); cbn in IHq.
    intros Δ HΔ HD. apply bot_r, IHq; assumption.
  - (* With *)
    destruct (introduced l) as [il |] eqn:Il; [| discriminate].
    destruct (introduced r) as [ir |] eqn:Ir; [| discriminate].
    injection I as <-.
    destruct (derive symbol_eqb R l) as [[θl γl al El] |] eqn:Dl; [| discriminate].
    destruct (derive symbol_eqb R r) as [[θr γr ar Er] |] eqn:Dr; [| discriminate].
    cbn in H.
    destruct (take symbol_eqb A γl al) as [gl |] eqn:Tl; [| discriminate].
    destruct (take symbol_eqb B γr ar) as [gr |] eqn:Tr; [| discriminate].
    unfold join_with in H; cbn in H.
    (* Each premise's zone is fresh where the conclusion's is: the rule
       keeps one zone, and the other is included in it. *)
    assert (Z : forall b, fresh b (gamma s) -> occurs b A = false /\ occurs b B = false /\
      fresh b gl /\ fresh b gr).
    { intros b HF.
      destruct al, ar; cbn in H.
      - injection H as <-; cbn in HF; facts. repeat split; try assumption.
        apply (fresh_incl _ _ (gl ++ diff symbol_eqb gr gl)); [| fresh_split; assumption].
        intros X HX. apply in_or_app, diff_in, HX.
      - destruct (take_all symbol_eqb gl gr) as [rest |] eqn:T; [| discriminate].
        injection H as <-; cbn in HF; facts. apply take_all_perm in T.
        repeat split; try assumption.
        apply (fresh_incl _ _ gr); [| assumption].
        intros X HX. apply (Permutation_in _ (Permutation_sym T)), in_or_app; auto.
      - destruct (take_all symbol_eqb gr gl) as [rest |] eqn:T; [| discriminate].
        injection H as <-; cbn in HF; facts. apply take_all_perm in T.
        repeat split; try assumption.
        apply (fresh_incl _ _ gl); [| assumption].
        intros X HX. apply (Permutation_in _ (Permutation_sym T)), in_or_app; auto.
      - destruct (take_all symbol_eqb gl gr) as [[| r' rest] |] eqn:T;
          [| discriminate | discriminate].
        injection H as <-; cbn in HF; facts. apply take_all_perm in T.
        repeat split; try assumption.
        apply (fresh_perm _ _ _ (Permutation_sym T)). rewrite app_nil_r. assumption. }
    assert (TH : theta s = θl ++ θr /\ eigens s = El ++ Er).
    { destruct al, ar; cbn in H; repeat (destruct (take_all _ _ _) as [[|] |]; try discriminate);
        injection H as <-; auto. }
    destruct TH as [TH TE].
    assert (Gl : good il (State θl γl al El)).
    { apply (good_of _ _ _ _ (fun b H => in_or_app _ _ _ (or_introl H)) G).
      cbn; intros b _ HE HF; rewrite TE, TH, map_app in *; facts.
      destruct (Z b ltac:(assumption)) as [ZA [ZB [Zl Zr]]].
      split; [assumption | fresh_split; [eapply fresh_take |]; eassumption]. }
    assert (Gr : good ir (State θr γr ar Er)).
    { apply (good_of _ _ _ _ (fun b H => in_or_app _ _ _ (or_intror H)) G).
      cbn; intros b _ HE HF; rewrite TE, TH, map_app in *; facts.
      destruct (Z b ltac:(assumption)) as [ZA [ZB [Zl Zr]]].
      split; [assumption | fresh_split; [eapply fresh_take |]; eassumption]. }
    specialize (IHl _ _ eq_refl eq_refl Gl); specialize (IHr _ _ eq_refl eq_refl Gr);
      cbn in IHl, IHr.
    assert (F : forall b, In b (il ++ ir) -> occurs b A = false /\ occurs b B = false /\
      fresh b gl /\ fresh b gr /\ fresh b θl /\ fresh b θr).
    { intros b Hb. destruct (G b Hb) as [_ HF]. rewrite TH, map_app in HF. facts.
      destruct (Z b ltac:(assumption)) as [ZA [ZB [Zl Zr]]]. auto 6. }
    pose proof (take_sat _ [] _ _ _ _ _ Tl
      (fun b H => proj1 (F b (in_or_app _ _ _ (or_introl H)))) IHl) as Hl; cbn in Hl.
    pose proof (take_sat _ [] _ _ _ _ _ Tr
      (fun b H => proj1 (proj2 (F b (in_or_app _ _ _ (or_intror H))))) IHr) as Hr; cbn in Hr.
    destruct al, ar; cbn in H.
    + (* both premises absorb: the union of their zones *)
      injection H as <-; cbn. intros Δ _ HD. apply with_r.
      * eapply ex_r; [apply (Hl (diff symbol_eqb gr gl ++ map wn θr ++ Δ)); [easy |] | perm].
        intros b Hb. pose proof (F b (in_or_app _ _ _ (or_introl Hb))) as [_ [_ [_ [Fr [_ Ft]]]]].
        fresh_split; [| assumption | apply HD, in_or_app; auto].
        apply (fresh_incl _ _ gr); [apply diff_incl | assumption].
      * eapply ex_r; [apply (Hr (diff symbol_eqb gl gr ++ map wn θl ++ Δ)); [easy |] | perm].
        intros b Hb. pose proof (F b (in_or_app _ _ _ (or_intror Hb))) as [_ [_ [Fl [_ [Ft _]]]]].
        fresh_split; [| assumption | apply HD, in_or_app; auto].
        apply (fresh_incl _ _ gl); [apply diff_incl | assumption].
    + (* the left premise absorbs: the right one's zone *)
      destruct (take_all symbol_eqb gl gr) as [rest |] eqn:T; [| discriminate].
      injection H as <-; cbn. apply take_all_perm in T.
      intros Δ HΔ HD. rewrite (HΔ eq_refl). apply with_r.
      * eapply ex_r; [apply (Hl (rest ++ map wn θr)); [easy |] | rewrite T; perm].
        intros b Hb. pose proof (F b (in_or_app _ _ _ (or_introl Hb))) as [_ [_ [_ [Fr [_ Ft]]]]].
        fresh_split; [| assumption].
        apply (fresh_incl _ _ gr); [| assumption].
        intros X HX. apply (Permutation_in _ (Permutation_sym T)), in_or_app; auto.
      * eapply ex_r; [apply (wk_list θl), (Hr []); [easy | delta] | perm].
    + (* the right premise absorbs: the left one's zone *)
      destruct (take_all symbol_eqb gr gl) as [rest |] eqn:T; [| discriminate].
      injection H as <-; cbn. apply take_all_perm in T.
      intros Δ HΔ HD. rewrite (HΔ eq_refl). apply with_r.
      * eapply ex_r; [apply (wk_list θr), (Hl []); [easy | delta] | perm].
      * eapply ex_r; [apply (Hr (rest ++ map wn θl)); [easy |] | rewrite T; perm].
        intros b Hb. pose proof (F b (in_or_app _ _ _ (or_intror Hb))) as [_ [_ [Fl [_ [Ft _]]]]].
        fresh_split; [| assumption].
        apply (fresh_incl _ _ gl); [| assumption].
        intros X HX. apply (Permutation_in _ (Permutation_sym T)), in_or_app; auto.
    + (* both exact: the same zone *)
      destruct (take_all symbol_eqb gl gr) as [[| r' rest] |] eqn:T;
        [| discriminate | discriminate].
      injection H as <-; cbn. apply take_all_perm in T. rewrite app_nil_r in T.
      intros Δ HΔ HD. rewrite (HΔ eq_refl). apply with_r.
      * eapply ex_r; [apply (wk_list θr), (Hl []); [easy | delta] | perm].
      * eapply ex_r; [apply (wk_list θl), (Hr []); [easy | delta] | rewrite T; perm].
  - (* Plus *)
    destruct (derive symbol_eqb R q) as [[θ γ a E] |] eqn:D; [| discriminate]. cbn in H.
    destruct (take symbol_eqb (match side with Left => A | Right => B end) γ a)
      as [g |] eqn:T; [| discriminate].
    injection H as <-; cbn.
    assert (FC : forall b, In b i -> occurs b (match side with Left => A | Right => B end) = false).
    { intros b Hb. destruct (G b Hb) as [_ HF]. cbn in HF. facts. destruct side; assumption. }
    assert (Gq : good i (State θ γ a E)).
    { apply (good_of _ _ _ _ (fun b H => H) G).
      cbn; intros b Hb HE HF; facts.
      split; [assumption | fresh_split; [eapply fresh_take; [eassumption | apply FC, Hb |] |]];
        assumption. }
    specialize (IHq _ _ I eq_refl Gq); cbn in IHq.
    pose proof (take_sat _ [] _ _ _ _ _ T FC IHq) as H1; cbn in H1.
    intros Δ HΔ HD. destruct side; [apply plus_r1 | apply plus_r2]; apply H1; assumption.
  - (* Top *)
    injection H as <-; cbn. intros Δ _ _. apply top_r.
  - (* Bang *)
    destruct (derive symbol_eqb R q) as [[θ γ a E] |] eqn:D; [| discriminate]. cbn in H.
    destruct (take symbol_eqb A γ a) as [[| g1 g'] |] eqn:T; [| discriminate | discriminate].
    cbn in H. injection H as <-; cbn.
    assert (Gq : good i (State θ γ a E)).
    { apply (good_of _ _ _ _ (fun b H => H) G).
      cbn; intros b _ HE HF; facts.
      split; [assumption | fresh_split; [eapply fresh_take; [eassumption | assumption | apply fresh_nil] |]];
        assumption. }
    specialize (IHq _ _ I eq_refl Gq); cbn in IHq.
    assert (FA : forall b, In b i -> occurs b A = false).
    { intros b Hb. destruct (G b Hb) as [_ HF]. cbn in HF. facts. assumption. }
    pose proof (take_sat _ [] _ _ _ _ _ T FA IHq) as H1; cbn in H1.
    intros Δ HΔ HD. rewrite (HΔ eq_refl).
    eapply ex_r; [apply (oc_r R A θ); eapply ex_r; [apply (H1 []); [easy | delta] | perm] | perm].
  - (* Quest *)
    destruct (derive symbol_eqb R q) as [[θ γ a E] |] eqn:D; [| discriminate].
    injection H as <-; cbn.
    assert (Gq : good i (State θ γ a E)).
    { apply (good_of _ _ _ _ (fun b H => H) G).
      cbn; intros b _ HE HF; facts.
      split; [assumption | fresh_split; [assumption |]].
      intros X HX. destruct (eq_dec X A) as [-> | ne]; [assumption |].
      match goal with HT : fresh _ (remove_all _ _ _) |- _ => apply HT end.
      unfold remove_all. apply filter_In. split; [exact HX |].
      destruct (eqb_spec A X); [congruence | reflexivity]. }
    specialize (IHq _ _ I eq_refl Gq); cbn in IHq.
    intros Δ HΔ HD. apply (wn_contract A (count θ A)).
    eapply ex_r; [apply (IHq Δ HΔ HD) |].
    transitivity (γ ++ map wn (repeat A (count θ A) ++ remove_all symbol_eqb A θ) ++ Δ).
    + apply Permutation_app_head, Permutation_app_tail, Permutation_map, remove_all_perm.
    + perm.
  - (* Copy *)
    destruct (derive symbol_eqb R q) as [[θ γ a E] |] eqn:D; [| discriminate]. cbn in H.
    destruct (take symbol_eqb A γ a) as [g |] eqn:T; [| discriminate].
    injection H as <-; cbn.
    assert (Gq : good i (State θ γ a E)).
    { apply (good_of _ _ _ _ (fun b H => H) G).
      cbn; intros b _ HE HF; facts.
      split; [assumption | fresh_split; [eapply fresh_take |]; eassumption]. }
    specialize (IHq _ _ I eq_refl Gq); cbn in IHq.
    assert (FA : forall b, In b i -> occurs b A = false).
    { intros b Hb. destruct (G b Hb) as [_ HF]. cbn in HF. facts. assumption. }
    pose proof (take_sat _ [] _ _ _ _ _ T FA IHq) as H1; cbn in H1.
    intros Δ HΔ HD. eapply ex_r; [apply de_r, (H1 Δ); assumption | perm].
  - (* Weaken *)
    destruct (Calculus.weakening R || is_wn A) eqn:W; [| discriminate].
    destruct (derive symbol_eqb R q) as [[θ γ a E] |] eqn:D; [| discriminate].
    injection H as <-; cbn.
    assert (Gq : good i (State θ γ a E)).
    { apply (good_of _ _ _ _ (fun b H => H) G).
      cbn; intros b _ HE HF; facts. split; [assumption | fresh_split; assumption]. }
    specialize (IHq _ _ I eq_refl Gq); cbn in IHq.
    intros Δ HΔ HD. apply orb_true_iff in W as [W | W].
    + apply (weakening_r R W), IHq; assumption.
    + destruct A; cbn in W; try discriminate. apply wk_r, IHq; assumption.
  - (* Mix *)
    destruct (Calculus.mix R) eqn:M; [| discriminate].
    destruct (introduced l) as [il |] eqn:Il; [| discriminate].
    destruct (introduced r) as [ir |] eqn:Ir; [| discriminate].
    injection I as <-.
    destruct (derive symbol_eqb R l) as [[θl γl al El] |] eqn:Dl; [| discriminate].
    destruct (derive symbol_eqb R r) as [[θr γr ar Er] |] eqn:Dr; [| discriminate].
    injection H as <-; cbn.
    assert (Gl : good il (State θl γl al El)).
    { apply (good_of _ _ _ _ (fun b H => in_or_app _ _ _ (or_introl H)) G).
      cbn; intros b _ HE HF; rewrite map_app in HF; facts.
      split; [assumption | fresh_split; assumption]. }
    assert (Gr : good ir (State θr γr ar Er)).
    { apply (good_of _ _ _ _ (fun b H => in_or_app _ _ _ (or_intror H)) G).
      cbn; intros b _ HE HF; rewrite map_app in HF; facts.
      split; [assumption | fresh_split; assumption]. }
    specialize (IHl _ _ eq_refl eq_refl Gl); specialize (IHr _ _ eq_refl eq_refl Gr);
      cbn in IHl, IHr.
    intros Δ HΔ HD. destruct al.
    + eapply ex_r; [apply (mix_r R M); [apply (IHl Δ) | apply (IHr [])]; first [easy | delta] | perm].
    + destruct ar.
      * eapply ex_r; [apply (mix_r R M); [apply (IHl []) | apply (IHr Δ)]; first [easy | delta] | perm].
      * rewrite (HΔ eq_refl).
        eapply ex_r; [apply (mix_r R M); [apply (IHl []) | apply (IHr [])]; first [easy | delta] | perm].
  - (* Cut *)
    destruct (Calculus.cut R) eqn:C; [| discriminate].
    destruct (introduced l) as [il |] eqn:Il; [| discriminate].
    destruct (introduced r) as [ir |] eqn:Ir; [| discriminate].
    injection I as <-.
    destruct (derive symbol_eqb R l) as [[θl γl al El] |] eqn:Dl; [| discriminate].
    destruct (derive symbol_eqb R r) as [[θr γr ar Er] |] eqn:Dr; [| discriminate].
    cbn in H.
    destruct (take symbol_eqb A γl al) as [gl |] eqn:Tl; [| discriminate].
    destruct (take symbol_eqb (dual A) γr ar) as [gr |] eqn:Tr; [| discriminate].
    injection H as <-; cbn.
    (* The cut formula's eigenvariables are in the set. *)
    assert (FA : forall b, In b (il ++ ir) -> occurs b A = false).
    { intros b Hb. destruct (G b Hb) as [HE _]. cbn in HE. facts.
      destruct (occurs b A) eqn:O; [| reflexivity].
      exfalso. apply (proj2 (feigens_occurs b A)) in O. contradiction. }
    assert (Gl : good il (State θl γl al El)).
    { apply (good_of _ _ _ _ (fun b H => in_or_app _ _ _ (or_introl H)) G).
      cbn; intros b Hb HE HF; rewrite map_app in HF; facts.
      split; [assumption | fresh_split; [eapply fresh_take; [eassumption | | eassumption] |]];
        [apply FA, in_or_app; auto | assumption]. }
    assert (Gr : good ir (State θr γr ar Er)).
    { apply (good_of _ _ _ _ (fun b H => in_or_app _ _ _ (or_intror H)) G).
      cbn; intros b Hb HE HF; rewrite map_app in HF; facts.
      split; [assumption | fresh_split; [eapply fresh_take; [eassumption | | eassumption] |]];
        [rewrite occurs_dual; apply FA, in_or_app; auto | assumption]. }
    specialize (IHl _ _ eq_refl eq_refl Gl); specialize (IHr _ _ eq_refl eq_refl Gr);
      cbn in IHl, IHr.
    pose proof (take_sat _ [] _ _ _ _ _ Tl
      (fun b H => FA b (in_or_app _ _ _ (or_introl H))) IHl) as Hl; cbn in Hl.
    pose proof (take_sat _ [] _ _ _ _ _ Tr
      (fun b H => eq_trans (occurs_dual b A) (FA b (in_or_app _ _ _ (or_intror H)))) IHr) as Hr;
      cbn in Hr.
    intros Δ HΔ HD. destruct al.
    + eapply ex_r; [apply (cut_r R C A); [apply (Hl Δ) | apply (Hr [])]; first [easy | delta] | perm].
    + destruct ar.
      * eapply ex_r; [apply (cut_r R C A); [apply (Hl []) | apply (Hr Δ)]; first [easy | delta] | perm].
      * rewrite (HΔ eq_refl).
        eapply ex_r; [apply (cut_r R C A); [apply (Hl []) | apply (Hr [])]; first [easy | delta] | perm].
  - (* Forall *)
    destruct (introduced q) as [iq |] eqn:Iq; [| discriminate].
    destruct (existsb (Nat.eqb a) iq) eqn:X; [discriminate |].
    injection I as <-.
    assert (Na : forall b, In b iq -> b <> a).
    { intros b Hb ->.
      assert (existsb (Nat.eqb a) iq = true) as Y; [| congruence].
      apply existsb_exists. exists a. split; [exact Hb | apply PeanoNat.Nat.eqb_refl]. }
    clear X.
    destruct (derive symbol_eqb R q) as [[θ γ ab E] |] eqn:D; [| discriminate]. cbn in H.
    destruct (take symbol_eqb (subst 0 (eigen a) A) γ ab) as [g |] eqn:T; [| discriminate].
    injection H as <-; cbn [theta gamma any eigens].
    (* The body's instance at [a] is fresh for every other eigenvariable. *)
    assert (FA : forall b, In b iq -> occurs b (subst 0 (eigen a) A) = false).
    { intros b Hb. destruct (G b (or_intror Hb)) as [_ HF]. cbn in HF. facts.
      destruct (occurs b (subst 0 (eigen a) A)) eqn:O; [| reflexivity].
      destruct (occurs_subst _ _ _ _ O) as [O' | O']; [congruence |].
      cbn in O'. apply PeanoNat.Nat.eqb_eq in O'. exfalso. exact (Na b Hb O'). }
    assert (Gq : good iq (State θ γ ab E)).
    { apply (good_of (a :: iq) iq _ _ (fun b H => or_intror H) G).
      cbn; intros b Hb HE HF; facts. split.
      - intros HI. apply HE, filter_In. split; [exact HI |].
        destruct (PeanoNat.Nat.eqb_spec a b) as [-> | _]; [exfalso; exact (Na b Hb eq_refl) |].
        reflexivity.
      - fresh_split; [eapply fresh_take; [eassumption | apply FA, Hb | assumption] | assumption]. }
    specialize (IHq _ _ eq_refl eq_refl Gq); cbn in IHq.
    pose proof (take_sat _ [] _ _ _ _ _ T FA IHq) as H1; cbn in H1.
    intros Δ HΔ HD. change (ll R (fall A :: (g ++ map wn θ ++ Δ))). apply (fall_r R A a).
    + (* the side condition: [a] is fresh in the conclusion, the context
         the rules below hand up included *)
      destruct (G a (or_introl eq_refl)) as [_ HF]. cbn [gamma theta] in HF.
      rewrite <- app_comm_cons in HF.
      apply (proj1 (fresh_cons_iff _ _ _)) in HF as [HA HF].
      apply (proj2 (fresh_cons_iff _ _ _)). split; [exact HA |].
      rewrite app_assoc. apply (proj2 (fresh_app_iff _ _ _)).
      split; [exact HF | apply HD; left; reflexivity].
    + apply H1; [assumption |]. intros b Hb. apply HD. right. exact Hb.
  - (* Exists *)
    destruct (tclosed t) eqn:Ct; [| discriminate].
    destruct (derive symbol_eqb R q) as [[θ γ ab E] |] eqn:D; [| discriminate]. cbn in H.
    destruct (take symbol_eqb (subst 0 t A) γ ab) as [g |] eqn:T; [| discriminate].
    injection H as <-; cbn.
    (* The witness's eigenvariables are in the set. *)
    assert (FA : forall b, In b i -> occurs b (subst 0 t A) = false).
    { intros b Hb. destruct (G b Hb) as [HE HF]. cbn in HE, HF. facts.
      destruct (occurs b (subst 0 t A)) eqn:O; [| reflexivity].
      destruct (occurs_subst _ _ _ _ O) as [O' | O']; [congruence |].
      exfalso. apply (proj2 (teigens_occurs b t)) in O'. contradiction. }
    assert (Gq : good i (State θ γ ab E)).
    { apply (good_of _ _ _ _ (fun b H => H) G).
      cbn; intros b Hb HE HF; facts.
      split; [assumption | fresh_split; [eapply fresh_take; [eassumption | apply FA, Hb | assumption] | assumption]]. }
    specialize (IHq _ _ I eq_refl Gq); cbn in IHq.
    pose proof (take_sat _ [] _ _ _ _ _ T FA IHq) as H1; cbn in H1.
    intros Δ HΔ HD. apply (fex_r R A t); [exact Ct |]. apply H1; assumption.
Qed.

(** ** The theorem *)

(** A term the checker accepts for a sequent proves it in the calculus. *)
Theorem soundness : forall p sequent,
  check symbol_eqb R p sequent = true -> ll R sequent.
Proof.
  unfold check; intros p Γ H.
  destruct (introduced p) as [i |] eqn:I; [| discriminate].
  destruct (derive symbol_eqb R p) as [[θ γ a E] |] eqn:D; [| discriminate].
  destruct θ; cbn in H; [| discriminate].
  apply andb_prop in H as [HI H].
  rewrite forallb_forall in HI.
  (* Every eigenvariable a [∀] step introduces is absent from the set and
     fresh in the sequent. *)
  assert (HΓ : forall b, In b i -> ~ In b E /\ fresh b Γ).
  { intros b Hb. specialize (HI b Hb). apply andb_prop in HI as [HE HS].
    split.
    - intros HE'. apply negb_true_iff in HE.
      assert (existsb (Nat.eqb b) E = true) as Y; [| congruence].
      apply existsb_exists. exists b. split; [exact HE' | apply PeanoNat.Nat.eqb_refl].
    - intros X HX. rewrite forallb_forall in HS. specialize (HS X HX).
      apply negb_true_iff in HS. exact HS. }
  destruct (take_all symbol_eqb γ Γ) as [[| r rest] |] eqn:T; [| | discriminate];
    apply take_all_perm in T.
  - assert (G : good i (State [] γ a E)).
    { intros b Hb. destruct (HΓ b Hb) as [HE HF]. split; [exact HE |].
      cbn. rewrite app_nil_r. apply (fresh_incl _ _ Γ); [| exact HF].
      intros X HX. apply (Permutation_in _ (Permutation_sym T)), in_or_app; auto. }
    pose proof (derive_sound _ _ _ I D G) as S; cbn in S.
    eapply ex_r; [apply (S []); [easy | intros ? ?; apply fresh_nil] | rewrite T; perm].
  - subst a.
    assert (G : good i (State [] γ true E)).
    { intros b Hb. destruct (HΓ b Hb) as [HE HF]. split; [exact HE |].
      cbn. rewrite app_nil_r. apply (fresh_incl _ _ Γ); [| exact HF].
      intros X HX. apply (Permutation_in _ (Permutation_sym T)), in_or_app; auto. }
    pose proof (derive_sound _ _ _ I D G) as S; cbn in S.
    eapply ex_r; [apply (S (r :: rest)); [easy |] | rewrite T; perm].
    intros b Hb. destruct (HΓ b Hb) as [_ HF]. apply (fresh_incl _ _ Γ); [| exact HF].
    intros X HX. apply (Permutation_in _ (Permutation_sym T)), in_or_app; auto.
Qed.

End Soundness.

(** The theorem for first-order classical linear logic, the calculus
    without Mix, weakening or cut. *)
Theorem soundness_classical {symbol : Type} (symbol_eqb : symbol -> symbol -> bool)
  (symbol_eqb_spec : forall f g, reflect (f = g) (symbol_eqb f g)) :
  forall (p : proof symbol) (sequent : list (formula symbol)),
  check symbol_eqb Calculus.classical p sequent = true -> ll Calculus.classical sequent.
Proof. exact (soundness symbol_eqb symbol_eqb_spec Calculus.classical). Qed.

(** The theorem for first-order linear logic with Mix: binary Mix, no
    weakening, no cut. *)
Theorem soundness_mix {symbol : Type} (symbol_eqb : symbol -> symbol -> bool)
  (symbol_eqb_spec : forall f g, reflect (f = g) (symbol_eqb f g)) :
  forall (p : proof symbol) (sequent : list (formula symbol)),
  check symbol_eqb {| Calculus.mix := true; Calculus.weakening := false; Calculus.cut := false |}
    p sequent = true ->
  ll {| Calculus.mix := true; Calculus.weakening := false; Calculus.cut := false |} sequent.
Proof. exact (soundness symbol_eqb symbol_eqb_spec _). Qed.

(** The theorem for first-order affine linear logic: weakening of any
    formula, no Mix, no cut. *)
Theorem soundness_affine {symbol : Type} (symbol_eqb : symbol -> symbol -> bool)
  (symbol_eqb_spec : forall f g, reflect (f = g) (symbol_eqb f g)) :
  forall (p : proof symbol) (sequent : list (formula symbol)),
  check symbol_eqb {| Calculus.mix := false; Calculus.weakening := true; Calculus.cut := false |}
    p sequent = true ->
  ll {| Calculus.mix := false; Calculus.weakening := true; Calculus.cut := false |} sequent.
Proof. exact (soundness symbol_eqb symbol_eqb_spec _). Qed.
