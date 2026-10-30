(* linlog © Fabian Lukas Grubmüller 2026 *)
(* Licensed under the EUPL *)

(** * First-order two-sided proof terms, their checker and its soundness

    The two-sided terms of [IntuitionisticCheck] over first-order
    intuitionistic formulas, the axiom naming its predicate and arguments,
    with the four quantifier rules: [IAllR] and [IExL] with the
    eigenvariable they introduce, [IExR] and [IAllL] with their witness.
    Every formula a node names is an instance, the terms of the
    quantifiers below substituted for their variables. [icheck] runs in
    two passes, as linlog's checker does: [iintroduced] collects the
    eigenvariables the [∀R] and [∃L] steps introduce and fails where one
    is introduced again above its step; [iderive] computes every node's
    sequent bottom-up and carries a set of eigenvariables to the root,
    which [∃R] and [∀L] add their witness's to, a cut its formula's, and
    [∀R] and [∃L] remove their own from. At the root no eigenvariable a
    step introduces may be left in the set or occur in the sequent.
    [soundness_ill] proves that a term it accepts yields a derivation in
    [ill], the condition on eigenvariables included. *)

From Stdlib Require Import Bool List Permutation Lia.
From Linlog Require Intuitionistic.
From Linlog Require Export TermsCheck FirstOrderIntuitionistic.
Import ListNotations.

Section Formulas.

Context {symbol : Type}.
Local Notation iformula := (iformula symbol).

Section Equality.

Variable symbol_eqb : symbol -> symbol -> bool.

(** Structural equality of intuitionistic formulas, decided through
    [symbol_eqb]. *)
Fixpoint iformula_eqb (A B : iformula) : bool :=
  match A, B with
  | ivar p l, ivar q k => symbol_eqb p q && terms_eqb symbol_eqb l k
  | ione, ione => true
  | itop, itop => true
  | izero, izero => true
  | itens A1 A2, itens B1 B2 => iformula_eqb A1 B1 && iformula_eqb A2 B2
  | ilmap A1 A2, ilmap B1 B2 => iformula_eqb A1 B1 && iformula_eqb A2 B2
  | iwith A1 A2, iwith B1 B2 => iformula_eqb A1 B1 && iformula_eqb A2 B2
  | iplus A1 A2, iplus B1 B2 => iformula_eqb A1 B1 && iformula_eqb A2 B2
  | ioc A, ioc B => iformula_eqb A B
  | iall A, iall B => iformula_eqb A B
  | iex A, iex B => iformula_eqb A B
  | _, _ => false
  end.

(** [symbol_eqb] decides equality, as the soundness theorem assumes. *)
Hypothesis symbol_eqb_spec : forall f g, reflect (f = g) (symbol_eqb f g).

Lemma iformula_eqb_spec : forall A B, reflect (A = B) (iformula_eqb A B).
Proof.
  induction A; destruct B; cbn; try (constructor; congruence).
  all: repeat match goal with
       | |- reflect _ (symbol_eqb ?a ?b && _) => destruct (symbol_eqb_spec a b)
       | |- reflect _ (terms_eqb _ ?l ?k) =>
           destruct (terms_eqb_spec symbol_eqb symbol_eqb_spec l k)
       | |- reflect _ (iformula_eqb ?A ?B && _) =>
           match goal with IH : forall B, reflect (A = B) _ |- _ => destruct (IH B) end
       | |- reflect _ (iformula_eqb ?A ?B) =>
           match goal with IH : forall B, reflect (A = B) _ |- _ => destruct (IH B) end
       | |- reflect _ (true && _) => cbn
       | |- reflect _ (false && _) => cbn
       end.
  all: constructor; congruence.
Qed.

(** Equality of intuitionistic formulas as a decision, for the proofs;
    the checker computes with [iformula_eqb]. *)
Definition iformula_eq_dec (A B : iformula) : {A = B} + {A <> B} :=
  reflect_dec _ _ (iformula_eqb_spec A B).

End Equality.

(** The eigenvariables of a formula, with repetitions. *)
Fixpoint ieigens (A : iformula) : list nat :=
  match A with
  | ivar _ l => flat_map teigens l
  | ione | itop | izero => []
  | itens A B | ilmap A B | iwith A B | iplus A B => ieigens A ++ ieigens B
  | ioc A | iall A | iex A => ieigens A
  end.

End Formulas.

(** Sequencing of partial computations: [k] runs on the value of [e], and
    [None] is failure. Local to this file. *)
Local Notation "'let*' x ':=' e 'in' k" :=
  (match e with Some x => k | None => None end)
  (at level 200, x pattern, e at level 100, k at level 200, right associativity).

Section Check.

Context {symbol : Type} (symbol_eqb : symbol -> symbol -> bool).
Local Notation term := (term symbol).
Local Notation iformula := (iformula symbol).
Local Notation eqb := (iformula_eqb symbol_eqb).

(** A two-sided proof term. [Γ ⊢ C] below is the conclusion of the node,
    the hypotheses read as a multiset. *)
Inductive iproof : Type :=
| IAx (p : symbol) (l : list term)
    (** the axiom [p(l) ⊢ p(l)] *)
| IOneR
    (** [⊢ 1] *)
| IOneL (premise : iproof)
    (** [Γ, 1 ⊢ C] from [Γ ⊢ C] *)
| ITensR (A B : iformula) (left right : iproof)
    (** [Γ, Δ ⊢ A ⊗ B] from [Γ ⊢ A] and [Δ ⊢ B] *)
| ITensL (A B : iformula) (premise : iproof)
    (** [Γ, A ⊗ B ⊢ C] from [Γ, A, B ⊢ C] *)
| ILmapR (A B : iformula) (premise : iproof)
    (** [Γ ⊢ A ⊸ B] from [Γ, A ⊢ B] *)
| ILmapL (A B : iformula) (left right : iproof)
    (** [Γ, Δ, A ⊸ B ⊢ C] from [Γ ⊢ A] and [Δ, B ⊢ C] *)
| ITopR (context : list iformula)
    (** [Γ ⊢ ⊤], with [Γ] the context *)
| IWithR (A B : iformula) (left right : iproof)
    (** [Γ ⊢ A & B] from [Γ ⊢ A] and [Γ ⊢ B] *)
| IWithL1 (A B : iformula) (premise : iproof)
    (** [Γ, A & B ⊢ C] from [Γ, A ⊢ C] *)
| IWithL2 (A B : iformula) (premise : iproof)
    (** [Γ, A & B ⊢ C] from [Γ, B ⊢ C] *)
| IZeroL (context : list iformula) (goal : iformula)
    (** [Γ, 0 ⊢ C], with [Γ] the context and [C] the goal *)
| IPlusR1 (A B : iformula) (premise : iproof)
    (** [Γ ⊢ A ⊕ B] from [Γ ⊢ A] *)
| IPlusR2 (A B : iformula) (premise : iproof)
    (** [Γ ⊢ A ⊕ B] from [Γ ⊢ B] *)
| IPlusL (A B : iformula) (left right : iproof)
    (** [Γ, A ⊕ B ⊢ C] from [Γ, A ⊢ C] and [Γ, B ⊢ C] *)
| IOcR (A : iformula) (premise : iproof)
    (** promotion, [!Γ ⊢ !A] from [!Γ ⊢ A] *)
| IDe (A : iformula) (premise : iproof)
    (** dereliction, [Γ, !A ⊢ C] from [Γ, A ⊢ C] *)
| IWk (A : iformula) (premise : iproof)
    (** weakening of a [!] hypothesis, [Γ, !A ⊢ C] from [Γ ⊢ C] *)
| ICo (A : iformula) (premise : iproof)
    (** contraction, [Γ, !A ⊢ C] from [Γ, !A, !A ⊢ C] *)
| IWeaken (A : iformula) (premise : iproof)
    (** weakening of any hypothesis, [Γ, A ⊢ C] from [Γ ⊢ C], with
        [iweakening] *)
| ICut (A : iformula) (left right : iproof)
    (** [Γ, Δ ⊢ C] from [Γ ⊢ A] and [Δ, A ⊢ C], with [icut]: either
        premise may be the one that proves [A] *)
| IAllR (A : iformula) (a : nat) (premise : iproof)
    (** [Γ ⊢ ∀x. A] from [Γ ⊢ A[a/x]], the eigenvariable [a] occurring
        in no formula of the conclusion *)
| IExL (A : iformula) (a : nat) (premise : iproof)
    (** [Γ, ∃x. A ⊢ C] from [Γ, A[a/x] ⊢ C], the eigenvariable [a]
        occurring in no formula of the conclusion *)
| IExR (A : iformula) (t : term) (premise : iproof)
    (** [Γ ⊢ ∃x. A] from [Γ ⊢ A[t/x]], for a closed term [t] *)
| IAllL (A : iformula) (t : term) (premise : iproof).
    (** [Γ, ∀x. A ⊢ C] from [Γ, A[t/x] ⊢ C], for a closed term [t] *)

(** ** Multisets, as lists compared through [iformula_eqb] *)

(** Removes one occurrence of [A] from [l], or fails when there is none. *)
Fixpoint pick (A : iformula) (l : list iformula) : option (list iformula) :=
  match l with
  | [] => None
  | B :: l' => if eqb A B then Some l' else option_map (cons B) (pick A l')
  end.

(** Whether [l1] and [l2] are the same multiset. *)
Fixpoint same (l1 l2 : list iformula) : bool :=
  match l1 with
  | [] => match l2 with [] => true | _ :: _ => false end
  | A :: l1' => match pick A l2 with Some l2' => same l1' l2' | None => false end
  end.

(** The formulas under the [!] of a list of [!] formulas, or [None] when
    one of them is not a [!] formula. *)
Fixpoint unoc (l : list iformula) : option (list iformula) :=
  match l with
  | [] => Some []
  | ioc A :: l' => option_map (cons A) (unoc l')
  | _ :: _ => None
  end.

(** The conclusion [Γ1, Γ2 ⊢ C2] of a cut on [A] whose first premise
    [Γ1 ⊢ C1] proves [A] and whose second [Γ2 ⊢ C2] has it as a
    hypothesis, or [None] when they do not. *)
Definition cut_by (A : iformula) (l1 : list iformula) (C1 : iformula)
    (l2 : list iformula) (C2 : iformula) : option (list iformula * iformula) :=
  if eqb A C1 then
    match pick A l2 with
    | Some l2' => Some (l1 ++ l2', C2)
    | None => None
    end
  else None.

(** The conclusion of a cut on [A] from its premises [Γ1 ⊢ C1] and
    [Γ2 ⊢ C2], whichever of them proves [A]. *)
Definition icut_conclusion (A : iformula) (l1 : list iformula) (C1 : iformula)
    (l2 : list iformula) (C2 : iformula) : option (list iformula * iformula) :=
  match cut_by A l1 C1 l2 C2 with
  | Some s => Some s
  | None => cut_by A l2 C2 l1 C1
  end.

(** Removes [a] from a set of eigenvariables. *)
Definition without (a : nat) (e : list nat) : list nat :=
  filter (fun b => negb (Nat.eqb a b)) e.

(** ** The first pass *)

(** The eigenvariables the [∀R] and [∃L] steps of [p] introduce, or
    [None] when one is introduced again above its step. *)
Fixpoint iintroduced (p : iproof) : option (list nat) :=
  match p with
  | IAx _ _ | IOneR | ITopR _ | IZeroL _ _ => Some []
  | IOneL q | ITensL _ _ q | ILmapR _ _ q | IWithL1 _ _ q | IWithL2 _ _ q
  | IPlusR1 _ _ q | IPlusR2 _ _ q | IOcR _ q | IDe _ q | IWk _ q | ICo _ q
  | IWeaken _ q | IExR _ _ q | IAllL _ _ q => iintroduced q
  | ITensR _ _ q r | ILmapL _ _ q r | IWithR _ _ q r | IPlusL _ _ q r | ICut _ q r =>
      let* i := iintroduced q in
      let* j := iintroduced r in
      Some (i ++ j)
  | IAllR _ a q | IExL _ a q =>
      let* i := iintroduced q in
      if existsb (Nat.eqb a) i then None else Some (a :: i)
  end.

(** ** The second pass *)

(** The conclusion [Γ ⊢ C] of a term and the eigenvariables it carries
    to the root, or [None] at the first node that misapplies its rule or
    uses one [R] forbids. *)
Fixpoint iderive (R : Intuitionistic.irules) (p : iproof)
    : option (list iformula * iformula * list nat) :=
  match p with
  | IAx a l => Some ([ivar a l], ivar a l, [])
  | IOneR => Some ([], ione, [])
  | IOneL q =>
      let* (l, C, e) := iderive R q in
      Some (ione :: l, C, e)
  | ITensR A B q r =>
      let* (l1, A', e1) := iderive R q in
      let* (l2, B', e2) := iderive R r in
      if eqb A A' && eqb B B' then Some (l1 ++ l2, itens A B, e1 ++ e2) else None
  | ITensL A B q =>
      let* (l, C, e) := iderive R q in
      let* l' := pick A l in
      let* l'' := pick B l' in
      Some (itens A B :: l'', C, e)
  | ILmapR A B q =>
      let* (l, B', e) := iderive R q in
      if eqb B B' then
        let* l' := pick A l in
        Some (l', ilmap A B, e)
      else None
  | ILmapL A B q r =>
      let* (l1, A', e1) := iderive R q in
      let* (l2, C, e2) := iderive R r in
      if eqb A A' then
        let* l2' := pick B l2 in
        Some (ilmap A B :: l1 ++ l2', C, e1 ++ e2)
      else None
  | ITopR l => Some (l, itop, [])
  | IWithR A B q r =>
      let* (l1, A', e1) := iderive R q in
      let* (l2, B', e2) := iderive R r in
      if eqb A A' && eqb B B' && same l1 l2 then Some (l1, iwith A B, e1 ++ e2) else None
  | IWithL1 A B q =>
      let* (l, C, e) := iderive R q in
      let* l' := pick A l in
      Some (iwith A B :: l', C, e)
  | IWithL2 A B q =>
      let* (l, C, e) := iderive R q in
      let* l' := pick B l in
      Some (iwith A B :: l', C, e)
  | IZeroL l C => Some (izero :: l, C, [])
  | IPlusR1 A B q =>
      let* (l, A', e) := iderive R q in
      if eqb A A' then Some (l, iplus A B, e) else None
  | IPlusR2 A B q =>
      let* (l, B', e) := iderive R q in
      if eqb B B' then Some (l, iplus A B, e) else None
  | IPlusL A B q r =>
      let* (l1, C1, e1) := iderive R q in
      let* (l2, C2, e2) := iderive R r in
      let* l1' := pick A l1 in
      let* l2' := pick B l2 in
      if eqb C1 C2 && same l1' l2' then Some (iplus A B :: l1', C1, e1 ++ e2) else None
  | IOcR A q =>
      let* (l, A', e) := iderive R q in
      let* _ := unoc l in
      if eqb A A' then Some (l, ioc A, e) else None
  | IDe A q =>
      let* (l, C, e) := iderive R q in
      let* l' := pick A l in
      Some (ioc A :: l', C, e)
  | IWk A q =>
      let* (l, C, e) := iderive R q in
      Some (ioc A :: l, C, e)
  | ICo A q =>
      let* (l, C, e) := iderive R q in
      let* l' := pick (ioc A) l in
      let* l'' := pick (ioc A) l' in
      Some (ioc A :: l'', C, e)
  | IWeaken A q =>
      if Intuitionistic.iweakening R then
        let* (l, C, e) := iderive R q in
        Some (A :: l, C, e)
      else None
  | ICut A q r =>
      if Intuitionistic.icut R then
        let* (l1, C1, e1) := iderive R q in
        let* (l2, C2, e2) := iderive R r in
        let* (l, C) := icut_conclusion A l1 C1 l2 C2 in
        Some (l, C, ieigens A ++ e1 ++ e2)
      else None
  | IAllR A a q =>
      let* (l, B, e) := iderive R q in
      if eqb (isubst 0 (eigen a) A) B then Some (l, iall A, without a e) else None
  | IExL A a q =>
      let* (l, C, e) := iderive R q in
      let* l' := pick (isubst 0 (eigen a) A) l in
      Some (iex A :: l', C, without a e)
  | IExR A t q =>
      if tclosed t then
        let* (l, B, e) := iderive R q in
        if eqb (isubst 0 t A) B then Some (l, iex A, teigens t ++ e) else None
      else None
  | IAllL A t q =>
      if tclosed t then
        let* (l, C, e) := iderive R q in
        let* l' := pick (isubst 0 t A) l in
        Some (iall A :: l', C, teigens t ++ e)
      else None
  end.

(** Whether [p] proves [hypotheses ⊢ goal] in [R]: no eigenvariable its
    [∀R] and [∃L] steps introduce is left in the set its root carries or
    occurs in the sequent, and its conclusion has that goal and those
    hypotheses, as a multiset. *)
Definition icheck (R : Intuitionistic.irules) (p : iproof) (hypotheses : list iformula)
    (goal : iformula) : bool :=
  match iintroduced p, iderive R p with
  | Some i, Some (l, C, e) =>
      forallb (fun a => negb (existsb (Nat.eqb a) e)
        && forallb (fun A => negb (ioccurs a A)) (goal :: hypotheses)) i &&
      eqb C goal && same l hypotheses
  | _, _ => false
  end.

End Check.

Arguments iproof symbol : clear implicits.

Section Soundness.

Context {symbol : Type} (symbol_eqb : symbol -> symbol -> bool)
  (symbol_eqb_spec : forall f g, reflect (f = g) (symbol_eqb f g)).
Local Notation term := (term symbol).
Local Notation iformula := (iformula symbol).
Local Notation eqb_spec := (iformula_eqb_spec symbol_eqb symbol_eqb_spec).
Local Notation eq_dec := (iformula_eq_dec symbol_eqb symbol_eqb_spec).

Lemma pick_perm : forall (A : iformula) l l',
  pick symbol_eqb A l = Some l' -> Permutation l (A :: l').
Proof.
  induction l as [| B l IH]; cbn; intros l' H; [discriminate |].
  destruct (eqb_spec A B) as [<- | _].
  - injection H as <-. reflexivity.
  - destruct (pick symbol_eqb A l) as [l'' |] eqn:E; cbn in H; [| discriminate].
    injection H as <-. rewrite (IH l'' eq_refl). apply perm_swap.
Qed.

Lemma same_perm : forall (l1 l2 : list iformula),
  same symbol_eqb l1 l2 = true -> Permutation l1 l2.
Proof.
  induction l1 as [| A l1 IH]; cbn; intros l2 H.
  - destruct l2; [reflexivity | discriminate].
  - destruct (pick symbol_eqb A l2) as [l2' |] eqn:E; [| discriminate].
    rewrite (pick_perm _ _ _ E). apply perm_skip, IH, H.
Qed.

Lemma unoc_map : forall (l l' : list iformula),
  unoc l = Some l' -> l = map ioc l'.
Proof.
  induction l as [| A l IH]; cbn; intros l' H.
  - injection H as <-. reflexivity.
  - destruct A; try discriminate.
    destruct (unoc l) as [l'' |]; cbn in H; [| discriminate].
    injection H as <-. cbn. f_equal. apply IH. reflexivity.
Qed.

Lemma ieigens_occurs : forall a (A : iformula), In a (ieigens A) <-> ioccurs a A = true.
Proof.
  intros a A. induction A; cbn;
    try (rewrite in_app_iff, orb_true_iff; tauto); try tauto; try (split; [contradiction | discriminate]).
  rewrite in_flat_map, existsb_exists.
  split; intros [t [Hin Ht]]; exists t; split; try exact Hin; apply teigens_occurs; assumption.
Qed.

Lemma ioccurs_subst : forall b k u (A : iformula),
  ioccurs b (isubst k u A) = true -> ioccurs b A = true \/ toccurs b u = true.
Proof.
  intros b k u A. revert k. induction A; cbn; intros k H; try discriminate;
    try (apply orb_true_iff in H as [H | H];
         [destruct (IHA1 _ H) | destruct (IHA2 _ H)]; rewrite ?orb_true_iff; tauto);
    try exact (IHA _ H).
  rewrite existsb_exists in *; destruct H as [t' [Hin Ht]];
    apply in_map_iff in Hin as [t [<- Hin]];
    destruct (toccurs_tsubst _ _ _ _ Ht); [left; exists t; auto | right; assumption].
Qed.

(** An instance is fresh where its body and its term are. *)
Lemma ioccurs_isubst_false : forall b k u (A : iformula),
  ioccurs b A = false -> toccurs b u = false -> ioccurs b (isubst k u A) = false.
Proof.
  intros * HA Hu. destruct (ioccurs b (isubst k u A)) eqn:O; [| reflexivity].
  destruct (ioccurs_subst _ _ _ _ O); congruence.
Qed.

(** A term is fresh where none of its eigenvariables is [b]. *)
Lemma toccurs_false : forall b (t : term), ~ In b (teigens t) -> toccurs b t = false.
Proof.
  intros b t H. destruct (toccurs b t) eqn:O; [| reflexivity].
  exfalso. apply H, teigens_occurs, O.
Qed.

Lemma ifresh_cons_iff : forall b (A : iformula) l,
  ifresh b (A :: l) <-> ioccurs b A = false /\ ifresh b l.
Proof.
  unfold ifresh; split.
  - intros H. split; [apply H; left; reflexivity | intros X HX; apply H; right; exact HX].
  - intros [HA Hl] X [<- | HX]; [exact HA | apply Hl, HX].
Qed.

Lemma ifresh_app_iff : forall b (l1 l2 : list iformula),
  ifresh b (l1 ++ l2) <-> ifresh b l1 /\ ifresh b l2.
Proof.
  unfold ifresh; split.
  - intros H. split; intros X HX; apply H, in_or_app; auto.
  - intros [H1 H2] X HX. apply in_app_or in HX as [HX | HX]; auto.
Qed.

Lemma ifresh_perm : forall b (l l' : list iformula),
  Permutation l' l -> ifresh b l' -> ifresh b l.
Proof. intros * P H X HX. apply H, (Permutation_in _ (Permutation_sym P) HX). Qed.

Lemma ifresh_nil : forall b, ifresh (symbol := symbol) b [].
Proof. intros b A []. Qed.

Lemma ifresh_map_ioc : forall b (l : list iformula), ifresh b (map ioc l) -> ifresh b l.
Proof. intros b l H X HX. apply (H (ioc X)), in_map, HX. Qed.

Lemma not_in_without : forall a b e, b <> a -> ~ In b (without a e) -> ~ In b e.
Proof.
  intros a b e N H HI. apply H. unfold without. apply filter_In. split; [exact HI |].
  destruct (PeanoNat.Nat.eqb_spec a b); [congruence | reflexivity].
Qed.

(** The first pass refused an eigenvariable introduced again above its
    step. *)
Lemma not_reintroduced : forall a i, existsb (Nat.eqb a) i = false -> forall b, In b i -> b <> a.
Proof.
  intros a i H b Hb ->.
  assert (existsb (Nat.eqb a) i = true) as Y; [| congruence].
  apply existsb_exists. exists a. split; [exact Hb | apply PeanoNat.Nat.eqb_refl].
Qed.

(** No eigenvariable of [i] stands in the set [e] or in the sequent
    [l ⊢ C]: what the rules below a subterm guarantee for the
    eigenvariables it introduces. *)
Definition igood (i : list nat) (l : list iformula) (C : iformula) (e : list nat) : Prop :=
  forall b, In b i -> ~ In b e /\ ifresh b (C :: l).

Variable R : Intuitionistic.irules.

(** A cut's conclusion, from premises whose sequents are derivable, is
    derivable. *)
Lemma cut_by_sound : forall A l1 C1 l2 C2 l C,
  Intuitionistic.icut R = true -> ill R l1 C1 -> ill R l2 C2 ->
  cut_by symbol_eqb A l1 C1 l2 C2 = Some (l, C) -> ill R l C.
Proof.
  unfold cut_by; intros * K H1 H2 E.
  destruct (eqb_spec A C1) as [-> | _]; [| discriminate].
  destruct (pick symbol_eqb C1 l2) as [l2' |] eqn:P; [| discriminate].
  injection E as <- <-. apply pick_perm in P.
  apply (cut_ir R K C1 l1 l2' C2); [exact H1 | exact (ex_ir R _ _ _ H2 P)].
Qed.

Lemma icut_conclusion_sound : forall A l1 C1 l2 C2 l C,
  Intuitionistic.icut R = true -> ill R l1 C1 -> ill R l2 C2 ->
  icut_conclusion symbol_eqb A l1 C1 l2 C2 = Some (l, C) -> ill R l C.
Proof.
  unfold icut_conclusion; intros * K H1 H2 E.
  destruct (cut_by symbol_eqb A l1 C1 l2 C2) as [[l' C'] |] eqn:B.
  - injection E as <- <-. exact (cut_by_sound _ _ _ _ _ _ _ K H1 H2 B).
  - exact (cut_by_sound _ _ _ _ _ _ _ K H2 H1 E).
Qed.

(** The formulas of a cut's premises are those of its conclusion and the
    cut formula. *)
Lemma cut_by_in : forall A l1 C1 l2 C2 l C X,
  cut_by symbol_eqb A l1 C1 l2 C2 = Some (l, C) ->
  In X (C1 :: l1) \/ In X (C2 :: l2) -> X = A \/ In X (C :: l).
Proof.
  unfold cut_by; intros * E HX.
  destruct (eqb_spec A C1) as [-> | _]; [| discriminate].
  destruct (pick symbol_eqb C1 l2) as [l2' |] eqn:P; [| discriminate].
  injection E as <- <-. apply pick_perm in P.
  destruct HX as [[-> | HX] | [-> | HX]]; [left; reflexivity | right; right; apply in_or_app; auto
    | right; left; reflexivity |].
  apply (Permutation_in _ P) in HX as [-> | HX]; [left; reflexivity |].
  right; right; apply in_or_app; auto.
Qed.

Lemma icut_conclusion_in : forall A l1 C1 l2 C2 l C X,
  icut_conclusion symbol_eqb A l1 C1 l2 C2 = Some (l, C) ->
  In X (C1 :: l1) \/ In X (C2 :: l2) -> X = A \/ In X (C :: l).
Proof.
  unfold icut_conclusion; intros * E HX.
  destruct (cut_by symbol_eqb A l1 C1 l2 C2) as [[l' C'] |] eqn:B.
  - injection E as <- <-. exact (cut_by_in _ _ _ _ _ _ _ _ B HX).
  - apply (cut_by_in _ _ _ _ _ _ _ _ E). tauto.
Qed.

(** Rewrites with the Boolean equalities of formulas a node compared. *)
Local Ltac equalities :=
  repeat match goal with
  | H : (_ && _) = true |- _ => apply andb_prop in H; destruct H
  | H : iformula_eqb symbol_eqb ?A ?B = true |- _ =>
      destruct (eqb_spec A B); [subst | discriminate]
  end.

(** Unfolds a node's [match]es on its subterms' conclusions and on the
    multiset operations, keeping what each one computed. *)
Local Ltac steps :=
  repeat match goal with
  | H : match ?e with _ => _ end = Some _ |- _ =>
      let E := fresh "E" in destruct e eqn:E; try discriminate
  | H : (if ?b then _ else _) = Some _ |- _ =>
      let E := fresh "E" in destruct b eqn:E; try discriminate
  end.

(** Decides a permutation of lists of formulas from the permutations in
    the context, by counting every formula's occurrences. *)
Local Ltac iperm :=
  apply (Permutation_count_occ eq_dec); intros ?x;
  repeat match goal with
  | H : Permutation _ _ |- _ =>
      rewrite (Permutation_count_occ eq_dec) in H; specialize (H x)
  end;
  repeat first [rewrite count_occ_app in * | progress cbn [count_occ] in *];
  repeat match goal with
  | |- context [eq_dec ?a ?b] => destruct (eq_dec a b)
  | H : context [eq_dec ?a ?b] |- _ => destruct (eq_dec a b)
  end;
  lia.

(** Proves a premise from the derivation of a subterm, up to a
    permutation of its hypotheses. *)
Local Ltac premise :=
  match goal with
  | H : ill R ?l ?C |- ill R _ ?C => apply (ex_ir R l _ C H); iperm
  end.

(** Breaks the freshness and the absences in the context into those of
    single formulas and lists. *)
Local Ltac facts :=
  repeat match goal with
  | H : ifresh _ (_ ++ _) |- _ => apply (proj1 (ifresh_app_iff _ _ _)) in H as [? ?]
  | H : ifresh _ (_ :: _) |- _ => apply (proj1 (ifresh_cons_iff _ _ _)) in H as [? ?]
  | H : ioccurs _ _ = false |- _ => progress cbn [ioccurs] in H
  | H : (_ || _) = false |- _ => apply orb_false_iff in H as [? ?]
  | H : ~ In _ (_ ++ _) |- _ => rewrite in_app_iff in H; apply Decidable.not_or in H as [? ?]
  end.

(** Proves [ioccurs b X = false] for a formula [X] the context says is
    fresh, or an instance of one at an eigenvariable other than [b] or at
    a witness whose eigenvariables [b] is not among. *)
Local Ltac instance :=
  first
    [ assumption
    | apply ioccurs_isubst_false; [assumption |];
      first
        [ apply toccurs_false; assumption
        | cbn; apply PeanoNat.Nat.eqb_neq;
          match goal with
          | N : existsb (Nat.eqb ?a) ?i = false, Hb : In ?b ?i |- ?b <> ?a =>
              exact (not_reintroduced _ _ N _ Hb)
          end ] ].

(** Proves [ioccurs b X = false] from [HX : In X l]: [l] is a list the
    context says is fresh, or a list built from such lists and formulas
    the context says are fresh, up to at most [n] permutations in the
    context. *)
Local Ltac ifresh_in n :=
  match goal with
  | HX : In ?X ?l, H : ifresh _ ?l |- _ => exact (H X HX)
  | HX : In ?X ?l, H : ifresh _ (map ioc ?l) |- _ => exact (ifresh_map_ioc _ _ H X HX)
  | HX : In ?X (_ :: _) |- _ => destruct HX as [<- | HX]; [instance | ifresh_in n]
  | HX : In ?X (_ ++ _) |- _ => apply in_app_or in HX as [HX | HX]; ifresh_in n
  | _ =>
      match n with
      | S ?m =>
          match goal with
          | HX : In ?X ?l, P : Permutation ?l _ |- _ =>
              apply (Permutation_in _ P) in HX; ifresh_in m
          | HX : In ?X ?l, P : Permutation _ ?l |- _ =>
              apply (Permutation_in _ (Permutation_sym P)) in HX; ifresh_in m
          end
      end
  end.

(** Proves that a premise's sequent is fresh, formula by formula, from
    the freshness of the conclusion's formulas: every formula of the
    premise is one of the conclusion's, found through the permutations
    in the context, or a subformula of one. *)
Local Ltac ifresh_goal :=
  let X := fresh "X" in
  let HX := fresh "HX" in
  intros X HX; ifresh_in 3.

(** Proves that a premise's sequent is good from the conclusion's
    [G], where its eigenvariables are among the conclusion's and each of
    its formulas is one of the conclusion's or a subformula of one. *)
Local Ltac good_tac G :=
  intros ?b ?Hb;
  let HE := fresh "HE" in
  let HF := fresh "HF" in
  destruct (G b ltac:(rewrite ?in_app_iff; cbn; auto)) as [HE HF]; facts;
  split;
  [ first
      [ assumption
      | match goal with
        | N : existsb (Nat.eqb ?a) ?i = false, Hb : In ?b ?i, HE : ~ In ?b (without ?a _) |- _ =>
            exact (not_in_without _ _ _ (not_reintroduced _ _ N _ Hb) HE)
        end ]
  | ifresh_goal ].

(** Every conclusion the pass computes is derivable, where no
    eigenvariable a step of the term introduces stands in it or in the
    set it carries. *)
Lemma iderive_sound : forall p i l C e,
  iintroduced p = Some i -> iderive symbol_eqb R p = Some (l, C, e) -> igood i l C e ->
  ill R l C.
Proof.
  induction p; intros i lc Cc ec I H G.
  21: {
    (* ICut: the cut formula's eigenvariables are in the set, and every
       other formula of a premise is one of the conclusion's. *)
    cbn -[icut_conclusion] in I, H.
    destruct (iintroduced p1) as [il |] eqn:Il; [| discriminate].
    destruct (iintroduced p2) as [ir |] eqn:Ir; [| discriminate].
    injection I as <-.
    destruct (Intuitionistic.icut R) eqn:K; [| discriminate].
    destruct (iderive symbol_eqb R p1) as [[[l1 C1] e1] |] eqn:D1; [| discriminate].
    destruct (iderive symbol_eqb R p2) as [[[l2 C2] e2] |] eqn:D2; [| discriminate].
    destruct (icut_conclusion symbol_eqb A l1 C1 l2 C2) as [[l C] |] eqn:Ec; [| discriminate].
    injection H as <- <- <-.
    assert (FX : forall b X, In b (il ++ ir) -> In X (C1 :: l1) \/ In X (C2 :: l2) ->
      ioccurs b X = false).
    { intros b X Hb HX. destruct (G b Hb) as [HE HF].
      destruct (icut_conclusion_in _ _ _ _ _ _ _ _ Ec HX) as [-> | HX'].
      - destruct (ioccurs b A) eqn:O; [| reflexivity].
        exfalso. apply HE, in_or_app. left. apply ieigens_occurs, O.
      - apply HF, HX'. }
    assert (G1 : igood il l1 C1 e1).
    { intros b Hb. destruct (G b (in_or_app _ _ _ (or_introl Hb))) as [HE _]. split.
      - intros HI. apply HE. rewrite !in_app_iff. auto.
      - intros X HX. apply (FX b X (in_or_app _ _ _ (or_introl Hb))). auto. }
    assert (G2 : igood ir l2 C2 e2).
    { intros b Hb. destruct (G b (in_or_app _ _ _ (or_intror Hb))) as [HE _]. split.
      - intros HI. apply HE. rewrite !in_app_iff. auto.
      - intros X HX. apply (FX b X (in_or_app _ _ _ (or_intror Hb))). auto. }
    exact (icut_conclusion_sound _ _ _ _ _ _ _ K (IHp1 _ _ _ _ eq_refl eq_refl G1)
      (IHp2 _ _ _ _ eq_refl eq_refl G2) Ec). }
  all: cbn in I, H; steps; equalities;
    repeat match goal with
    | H : Some (_, _, _) = Some (_, _, _) |- _ => injection H as <- <- <-
    | H : Some _ = Some ?x |- _ => is_var x; injection H as <-
    | H : pick _ _ _ = Some _ |- _ => apply pick_perm in H
    | H : same _ _ _ = true |- _ => apply same_perm in H
    | H : unoc ?l = Some _ |- _ => apply unoc_map in H; subst l
    end.
  all: try (apply ax_ir || apply one_irr || apply top_irr || apply zero_ilr).
  (* Every subterm's sequent is good: the eigenvariables of a premise are
     among the conclusion's, but the one its step introduces, and its
     formulas among the conclusion's, their subformulas, or instances
     whose terms the set holds. *)
  all: repeat match goal with
       | IH : forall i l C e, _ |- _ =>
           let H' := fresh "IH" in
           pose proof IH as H';
           specialize (H' _ _ _ _ ltac:(first [reflexivity | eassumption])
             ltac:(first [reflexivity | eassumption]));
           clear IH;
           match type of H' with
           | igood ?i ?l ?C ?e -> _ =>
               let G' := fresh "G" in
               assert (G' : igood i l C e) by good_tac G; specialize (H' G')
           end
       end.
  (* The rule, its side condition on the eigenvariable from the root's
     guarantee for the conclusion. *)
  all: first
    [ match goal with
      | G : igood (?a :: _) ?l (iall ?A) _ |- ill R ?l (iall ?A) =>
          apply (all_irr R A a); [exact (proj2 (G a (or_introl eq_refl))) | premise]
      | G : igood (?a :: _) (iex ?A :: ?l) ?C _ |- ill R (iex ?A :: ?l) ?C =>
          apply (ex_ilr R A a); [| premise];
          apply (ifresh_perm _ _ (C :: iex A :: l)); [apply perm_swap |];
          exact (proj2 (G a (or_introl eq_refl)))
      end
    | solve
        [ apply one_ilr; premise
        | apply tens_irr; premise
        | apply tens_ilr; premise
        | apply lmap_irr; premise
        | apply lmap_ilr; premise
        | apply with_irr; premise
        | apply with_ilr1; premise
        | apply with_ilr2; premise
        | apply plus_irr1; premise
        | apply plus_irr2; premise
        | apply plus_ilr; premise
        | apply oc_irr; premise
        | apply de_ilr; premise
        | apply wk_ilr; premise
        | apply co_ilr; premise
        | apply weakening_ilr; [assumption | premise]
        | eapply ex_irr; [eassumption | premise]
        | eapply all_ilr; [eassumption | premise] ] ].
Qed.

(** A term the checker accepts for a sequent proves it in the
    two-sided calculus [R]: first-order intuitionistic linear logic or
    its affine variant, with or without cut. *)
Theorem soundness_ill : forall p hypotheses goal,
  icheck symbol_eqb R p hypotheses goal = true -> ill R hypotheses goal.
Proof.
  unfold icheck; intros p Γ C H.
  destruct (iintroduced p) as [i |] eqn:I; [| discriminate].
  destruct (iderive symbol_eqb R p) as [[[l C'] e] |] eqn:D; [| discriminate].
  apply andb_prop in H as [H HΓ]. apply andb_prop in H as [HI HC].
  destruct (eqb_spec C' C); [subst | discriminate].
  apply same_perm in HΓ.
  rewrite forallb_forall in HI.
  assert (G : igood i l C e).
  { intros b Hb. specialize (HI b Hb). apply andb_prop in HI as [HE HS]. split.
    - intros HE'. apply negb_true_iff in HE.
      assert (existsb (Nat.eqb b) e = true) as Y; [| congruence].
      apply existsb_exists. exists b. split; [exact HE' | apply PeanoNat.Nat.eqb_refl].
    - apply (ifresh_perm _ _ (C :: Γ)); [apply perm_skip, Permutation_sym, HΓ |].
      intros X HX. rewrite forallb_forall in HS. apply negb_true_iff, HS, HX. }
  apply (ex_ir R l Γ C); [exact (iderive_sound _ _ _ _ _ I D G) | exact HΓ].
Qed.

End Soundness.
