(* linlog © Fabian Lukas Grubmüller 2026 *)
(* Licensed under the EUPL *)

(** * Refutations: invariants that every derivation keeps

    Three properties hold of every derivable sequent, so a sequent that
    lacks one is not derivable, and a few numbers or an assignment that
    a function computes with show it:

    - the classical reading: erasing [!] and [?] and reading [⊗] and [&]
      as "and", [⅋] and [⊕] as "or", [1] and [⊤] as true and [⊥] and [0]
      as false, every derivable sequent is a tautology, in every
      calculus [ll R], Mix and weakening included ([ll_tautology]);
    - the balance of an atom: without weakening, whichever additive
      alternatives a derivation takes, the literals of an atom meet in
      axioms, so the interval of its excess of positive over negative
      literals contains zero, where no [⊤] absorbs and no [!] or [?]
      copies it ([balance]);
    - the count equation: without weakening, a derivable sequent of
      multiplicative formulas with units has [#⊗ − #⅋ − #1 + #⊥ + 2]
      formulas, at least that many with Mix ([count_equation]).

    Each gives a theorem [~ ll R Γ] from a Boolean test of [Γ] that
    computation decides, and the two-sided calculus inherits all three
    through its one-sided translation ([ill_ll]): [~ ill R Γ C]. The
    statements mention [ll] and [ill] alone, and the tests appear in
    their hypotheses only, which [vm_compute] discharges. *)

From Stdlib Require Import Bool List Permutation ZArith Lia.
From Linlog Require Export Calculus.
From Linlog Require Export Intuitionistic.
Import ListNotations.

Section Refutations.

Context {atom : Type}.
Local Notation formula := (formula atom).
Local Notation iformula := (iformula atom).

(** ** The classical reading *)

(** A formula's classical reading under an assignment of the atoms. *)
Fixpoint eval (v : atom -> bool) (A : formula) : bool :=
  match A with
  | var a => v a
  | covar a => negb (v a)
  | one | top => true
  | bot | zero => false
  | tens A B | awith A B => eval v A && eval v B
  | parr A B | aplus A B => eval v A || eval v B
  | oc A | wn A => eval v A
  end.

(** A sequent reads as the disjunction of its formulas, which a
    permutation keeps. *)
Lemma existsb_perm : forall (f : formula -> bool) l1 l2,
  Permutation l1 l2 -> existsb f l1 = existsb f l2.
Proof.
  intros f l1 l2 H. induction H; cbn.
  - reflexivity.
  - now rewrite IHPermutation.
  - destruct (f x), (f y); reflexivity.
  - congruence.
Qed.

(** Every derivable sequent, in every calculus, reads as a tautology. *)
Theorem ll_tautology : forall R l, ll R l -> forall v, existsb (eval v) l = true.
Proof.
  intros R l H v. induction H; cbn in *; rewrite ?existsb_app in *.
  - destruct (v a); reflexivity.
  - now rewrite <- (existsb_perm _ _ _ H0).
  - reflexivity.
  - exact IHll.
  - reflexivity.
  - destruct (eval v A), (eval v B), (existsb (eval v) l1), (existsb (eval v) l2);
      cbn in *; congruence.
  - destruct (eval v A), (eval v B); cbn in *; congruence.
  - destruct (eval v A), (eval v B); cbn in *; congruence.
  - destruct (eval v A), (eval v B); cbn in *; congruence.
  - destruct (eval v A), (eval v B); cbn in *; congruence.
  - exact IHll.
  - exact IHll.
  - rewrite IHll. apply orb_true_r.
  - destruct (eval v A); cbn in *; congruence.
  - now rewrite IHll1.
  - rewrite IHll. apply orb_true_r.
Qed.

(** Whether an assignment falsifies a sequent's reading. *)
Definition falsifies (v : atom -> bool) (l : list formula) : bool :=
  negb (existsb (eval v) l).

(** An assignment that falsifies a sequent's reading refutes it in every
    calculus. *)
Theorem classical_refutation : forall R v l, falsifies v l = true -> ~ ll R l.
Proof.
  intros R v l F H. unfold falsifies in F.
  rewrite (ll_tautology R l H v) in F. discriminate.
Qed.

(** ** The balance of an atom *)

Section Balance.

(** The equality of atoms the literals are compared with, and the atom
    counted. The invariant holds whatever the function is, since both
    literals of an axiom compare alike; a decision gives it its
    meaning. *)
Variable atom_eqb : atom -> atom -> bool.
Variable a : atom.

(** Intervals of the excess, [None] where it is unbounded. *)
Local Notation span := (option (Z * Z)).

(** The interval of a sum. *)
Definition add (x y : span) : span :=
  match x, y with
  | Some (l1, h1), Some (l2, h2) => Some ((l1 + l2)%Z, (h1 + h2)%Z)
  | _, _ => None
  end.

(** The hull of two intervals, the excess of either alternative. *)
Definition hull (x y : span) : span :=
  match x, y with
  | Some (l1, h1), Some (l2, h2) => Some (Z.min l1 l2, Z.max h1 h2)
  | _, _ => None
  end.

(** Below [!] or [?]: nothing where the atom is absent, else unbounded,
    since copies add any number. *)
Definition bang (x : span) : span :=
  match x with
  | Some (Z0, Z0) => Some (0%Z, 0%Z)
  | _ => None
  end.

(** The interval of the atom's excess of positive over negative literals
    in a formula, over its additive alternatives: [None] where a [⊤]
    absorbs or the atom lies below an exponential. *)
Fixpoint excess (A : formula) : span :=
  match A with
  | var b => Some (if atom_eqb a b then (1%Z, 1%Z) else (0%Z, 0%Z))
  | covar b => Some (if atom_eqb a b then ((-1)%Z, (-1)%Z) else (0%Z, 0%Z))
  | one | bot | zero => Some (0%Z, 0%Z)
  | top => None
  | tens A B | parr A B => add (excess A) (excess B)
  | awith A B | aplus A B => hull (excess A) (excess B)
  | oc A | wn A => bang (excess A)
  end.

(** The interval of a sequent: the sum over its formulas. *)
Fixpoint excesses (l : list formula) : span :=
  match l with
  | [] => Some (0%Z, 0%Z)
  | A :: l => add (excess A) (excesses l)
  end.

(** That an interval contains zero, or is unbounded. *)
Definition holds (x : span) : Prop :=
  match x with
  | Some (lo, hi) => (lo <= 0 <= hi)%Z
  | None => True
  end.

Lemma add_none_r : forall x, add x None = None.
Proof. intros [[]|]; reflexivity. Qed.

Lemma add_zero_l : forall x, add (Some (0%Z, 0%Z)) x = x.
Proof. intros [[]|]; reflexivity. Qed.

Lemma add_comm : forall x y, add x y = add y x.
Proof. intros [[]|] [[]|]; cbn; try reflexivity. f_equal. f_equal; lia. Qed.

Lemma add_assoc : forall x y z, add x (add y z) = add (add x y) z.
Proof. intros [[]|] [[]|] [[]|]; cbn; try reflexivity. f_equal. f_equal; lia. Qed.

Lemma excesses_app : forall l1 l2,
  excesses (l1 ++ l2) = add (excesses l1) (excesses l2).
Proof.
  induction l1 as [|A l1 IH]; intro l2; cbn [app excesses].
  - now rewrite add_zero_l.
  - now rewrite IH, add_assoc.
Qed.

Lemma excesses_perm : forall l1 l2, Permutation l1 l2 -> excesses l1 = excesses l2.
Proof.
  intros l1 l2 H. induction H; cbn [excesses].
  - reflexivity.
  - now rewrite IHPermutation.
  - now rewrite !add_assoc, (add_comm (excess y)).
  - congruence.
Qed.

Lemma bang_cases : forall x,
  bang x = None \/ (bang x = Some (0%Z, 0%Z) /\ x = Some (0%Z, 0%Z)).
Proof. intros [[[] []]|]; cbn; first [left; reflexivity | right; split; reflexivity]. Qed.

Lemma excesses_wn : forall l,
  excesses (map wn l) = None \/ excesses (map wn l) = Some (0%Z, 0%Z).
Proof.
  induction l as [|A l IH]; cbn [map excesses excess]; [now right |].
  destruct (bang_cases (excess A)) as [-> | [-> _]]; [now left |].
  rewrite add_zero_l. exact IH.
Qed.

(** Without weakening, every derivable sequent's interval contains zero
    for every atom. *)
Theorem balance : forall R l, ll R l -> weakening R = false -> holds (excesses l).
Proof.
  intros R l H W. induction H; cbn [excesses excess] in *.
  - destruct (atom_eqb a a0); cbn; lia.
  - now rewrite <- (excesses_perm _ _ H0).
  - cbn; lia.
  - now rewrite add_zero_l.
  - cbn; exact I.
  - rewrite excesses_app in *.
    destruct (excess A) as [[]|], (excess B) as [[]|], (excesses l1) as [[]|],
      (excesses l2) as [[]|]; cbn in *; auto; lia.
  - rewrite <- add_assoc. exact IHll.
  - destruct (excess A) as [[]|], (excess B) as [[]|], (excesses l) as [[]|];
      cbn in *; auto; lia.
  - destruct (excess A) as [[]|], (excess B) as [[]|], (excesses l) as [[]|];
      cbn in *; auto; lia.
  - destruct (excess A) as [[]|], (excess B) as [[]|], (excesses l) as [[]|];
      cbn in *; auto; lia.
  - destruct (bang_cases (excess A)) as [-> | [-> _]]; [cbn; exact I |].
    rewrite add_zero_l.
    destruct (excesses_wn l) as [-> | ->]; cbn; [exact I | lia].
  - destruct (bang_cases (excess A)) as [-> | [-> E]]; [cbn; exact I |].
    rewrite E in IHll. exact IHll.
  - destruct (bang_cases (excess A)) as [-> | [-> _]]; [cbn; exact I |].
    now rewrite add_zero_l.
  - destruct (bang_cases (excess A)) as [B | [B _]]; rewrite B in *; [cbn; exact I |].
    now rewrite !add_zero_l in *.
  - rewrite excesses_app.
    destruct (excesses l1) as [[]|], (excesses l2) as [[]|]; cbn in *; auto; lia.
  - congruence.
Qed.

(** Whether an interval excludes zero. *)
Definition excluded (x : span) : bool :=
  match x with
  | Some (lo, hi) => (0 <? lo)%Z || (hi <? 0)%Z
  | None => false
  end.

(** Whether the balance of the atom refutes a sequent in a calculus: one
    without weakening, and an interval that excludes zero. *)
Definition unbalanced (R : rules) (l : list formula) : bool :=
  negb (weakening R) && excluded (excesses l).

(** An unbalanced atom refutes a sequent. *)
Theorem unbalanced_refutation : forall R l, unbalanced R l = true -> ~ ll R l.
Proof.
  intros R l U H. unfold unbalanced in U.
  apply andb_prop in U as [W E]. apply negb_true_iff in W.
  pose proof (balance R l H W) as B.
  destruct (excesses l) as [[lo hi]|]; cbn in *; [| discriminate].
  apply orb_true_iff in E as [E | E]; apply Z.ltb_lt in E; lia.
Qed.

End Balance.

(** ** The count equation *)

(** Whether a formula is multiplicative, with units. *)
Fixpoint mll (A : formula) : bool :=
  match A with
  | var _ | covar _ | one | bot => true
  | tens A B | parr A B => mll A && mll B
  | top | zero | awith _ _ | aplus _ _ | oc _ | wn _ => false
  end.

(** [#⊗ − #⅋ − #1 + #⊥] of a formula. *)
Fixpoint weight (A : formula) : Z :=
  match A with
  | one => (-1)%Z
  | bot => 1%Z
  | tens A B => (1 + weight A + weight B)%Z
  | parr A B => (-1 + weight A + weight B)%Z
  | _ => 0%Z
  end.

(** The weight of a sequent: the sum over its formulas. *)
Fixpoint weights (l : list formula) : Z :=
  match l with
  | [] => 0%Z
  | A :: l => (weight A + weights l)%Z
  end.

Lemma weights_app : forall l1 l2, weights (l1 ++ l2) = (weights l1 + weights l2)%Z.
Proof. induction l1 as [|A l1 IH]; intro l2; cbn; [| rewrite IH]; lia. Qed.

Lemma weights_perm : forall l1 l2, Permutation l1 l2 -> weights l1 = weights l2.
Proof. intros l1 l2 H. induction H; cbn; lia. Qed.

Lemma forallb_perm : forall (f : formula -> bool) l1 l2,
  Permutation l1 l2 -> forallb f l1 = forallb f l2.
Proof.
  intros f l1 l2 H. induction H; cbn.
  - reflexivity.
  - now rewrite IHPermutation.
  - destruct (f x), (f y); reflexivity.
  - congruence.
Qed.

(** What the count equation says of a sequent in a calculus. *)
Definition counted (R : rules) (l : list formula) : Prop :=
  if mix R then (weights l + 2 <= Z.of_nat (length l))%Z
  else Z.of_nat (length l) = (weights l + 2)%Z.

(** Without weakening, every derivable sequent of multiplicative formulas
    with units has as many formulas as the equation says, at least as
    many with Mix. *)
Theorem count_equation : forall R l,
  ll R l -> weakening R = false -> forallb mll l = true -> counted R l.
Proof.
  intros R l H W. unfold counted.
  induction H; cbn [weights weight length forallb mll andb] in *; intro M.
  - destruct (mix R); lia.
  - rewrite <- (Permutation_length H0), <- (weights_perm _ _ H0).
    apply IHll. now rewrite (forallb_perm _ _ _ H0).
  - destruct (mix R); lia.
  - specialize (IHll M). destruct (mix R); lia.
  - discriminate.
  - rewrite forallb_app in M.
    apply andb_prop in M as [M M2]. apply andb_prop in M as [MA MB].
    apply andb_prop in M2 as [M1 M2].
    specialize (IHll1 (andb_true_intro (conj MA M1))).
    specialize (IHll2 (andb_true_intro (conj MB M2))).
    rewrite length_app, weights_app. destruct (mix R); lia.
  - apply andb_prop in M as [M M1]. apply andb_prop in M as [MA MB].
    specialize (IHll (andb_true_intro (conj MA (andb_true_intro (conj MB M1))))).
    destruct (mix R); lia.
  - discriminate.
  - discriminate.
  - discriminate.
  - discriminate.
  - discriminate.
  - discriminate.
  - discriminate.
  - rewrite forallb_app in M. apply andb_prop in M as [M1 M2].
    specialize (IHll1 M1). specialize (IHll2 M2).
    match goal with E : mix R = true |- _ => rewrite E in * end.
    rewrite length_app, weights_app. lia.
  - congruence.
Qed.

(** Whether the count equation refutes a sequent in a calculus: one
    without weakening, multiplicative formulas with units, and fewer
    formulas than the equation asks for with Mix, another number
    without. *)
Definition fails_equation (R : rules) (l : list formula) : bool :=
  negb (weakening R) && forallb mll l
  && (if mix R then (Z.of_nat (length l) <? weights l + 2)%Z
      else negb (Z.of_nat (length l) =? weights l + 2)%Z).

(** A failed count equation refutes a sequent. *)
Theorem equation_refutation : forall R l, fails_equation R l = true -> ~ ll R l.
Proof.
  intros R l F H. unfold fails_equation in F.
  apply andb_prop in F as [F E]. apply andb_prop in F as [W M].
  apply negb_true_iff in W.
  pose proof (count_equation R l H W M) as C. unfold counted in C.
  destruct (mix R).
  - apply Z.ltb_lt in E. lia.
  - apply negb_true_iff, Z.eqb_neq in E. lia.
Qed.

(** ** The two-sided calculus in the one-sided one *)

(** An intuitionistic formula as a one-sided one: [A ⊸ B] is [A⊥ ⅋ B]. *)
Fixpoint itr (A : iformula) : formula :=
  match A with
  | ivar a => var a
  | ione => one
  | itens A B => tens (itr A) (itr B)
  | ilmap A B => parr (dual (itr A)) (itr B)
  | itop => top
  | iwith A B => awith (itr A) (itr B)
  | izero => zero
  | iplus A B => aplus (itr A) (itr B)
  | ioc A => oc (itr A)
  end.

(** A two-sided sequent [Γ ⊢ C] as the one-sided [⊢ C, Γ⊥]. *)
Definition one_sided (G : list iformula) (C : iformula) : list formula :=
  itr C :: map (fun A => dual (itr A)) G.

(** The one-sided calculus of a two-sided one: no Mix, and its
    weakening. *)
Definition lrules (R : irules) : rules :=
  {| mix := false; weakening := iweakening R |}.

(** The first two formulas exchanged. *)
Local Lemma swap : forall R (A B : formula) l, ll R (A :: B :: l) -> ll R (B :: A :: l).
Proof. intros. eapply ex_r; [eassumption | apply perm_swap]. Qed.

(** Every derivation of the two-sided calculus gives one of its
    one-sided translation, without Mix and with the same weakening. *)
Theorem ill_ll : forall R G C, ill R G C -> ll (lrules R) (one_sided G C).
Proof.
  intros R G C H. unfold one_sided. induction H; cbn in *.
  - apply swap, ax_r.
  - eapply ex_r; [exact IHill |]. apply perm_skip, Permutation_map, H0.
  - apply one_r.
  - apply swap, bot_r, IHill.
  - rewrite map_app. apply tens_r; assumption.
  - apply swap, parr_r.
    eapply ex_r; [exact IHill |].
    apply perm_trans with (dual (itr A) :: itr C :: dual (itr B) :: map (fun A => dual (itr A)) l);
      [apply perm_swap | apply perm_skip, perm_swap].
  - apply parr_r. apply swap. exact IHill.
  - rewrite dual_dual, map_app. apply swap.
    eapply ex_r.
    + apply (tens_r _ (itr A) (dual (itr B)) (map (fun A => dual (itr A)) l1)
               (itr C :: map (fun A => dual (itr A)) l2)); [exact IHill1 | apply swap, IHill2].
    + apply perm_skip. symmetry. apply Permutation_middle.
  - apply top_r.
  - apply with_r; assumption.
  - apply swap, plus_r1, swap, IHill.
  - apply swap, plus_r2, swap, IHill.
  - apply swap, top_r.
  - apply plus_r1. exact IHill.
  - apply plus_r2. exact IHill.
  - apply swap, with_r; apply swap; assumption.
  - rewrite map_map in *. cbn in *.
    replace (map (fun x => wn (dual (itr x))) l) with (map wn (map (fun x => dual (itr x)) l))
      in * by now rewrite map_map.
    apply oc_r, IHill.
  - apply swap, de_r, swap, IHill.
  - apply swap, wk_r, IHill.
  - apply swap, co_r.
    eapply ex_r; [exact IHill |].
    apply perm_trans with (wn (dual (itr A)) :: itr C :: wn (dual (itr A)) :: map (fun A => dual (itr A)) l);
      [apply perm_swap | apply perm_skip, perm_swap].
  - apply swap, weakening_r; [exact H | exact IHill].
Qed.

(** An intuitionistic formula's classical reading: [⊸] as implication. *)
Fixpoint ieval (v : atom -> bool) (A : iformula) : bool :=
  match A with
  | ivar a => v a
  | ione | itop => true
  | izero => false
  | itens A B | iwith A B => ieval v A && ieval v B
  | iplus A B => ieval v A || ieval v B
  | ilmap A B => implb (ieval v A) (ieval v B)
  | ioc A => ieval v A
  end.

Lemma eval_dual : forall v A, eval v (dual A) = negb (eval v A).
Proof.
  intros v A. induction A; cbn; rewrite ?IHA, ?IHA1, ?IHA2;
    try destruct (eval v A1), (eval v A2); try destruct (v a); reflexivity.
Qed.

Lemma eval_itr : forall v A, eval v (itr A) = ieval v A.
Proof.
  intros v A. induction A; cbn; rewrite ?eval_dual, ?IHA, ?IHA1, ?IHA2; try reflexivity.
  destruct (ieval v A1), (ieval v A2); reflexivity.
Qed.

(** True hypotheses read false on the one-sided side. *)
Lemma duals_false : forall v G,
  forallb (ieval v) G = true -> existsb (eval v) (map (fun A => dual (itr A)) G) = false.
Proof.
  intros v G T. induction G as [|A G IH]; cbn in *; [reflexivity |].
  apply andb_prop in T as [TA TG].
  rewrite eval_dual, eval_itr, TA. exact (IH TG).
Qed.

(** Every derivable two-sided sequent, in either calculus, reads as a
    tautology: where every hypothesis is true, so is the conclusion. *)
Theorem ill_tautology : forall R G C, ill R G C ->
  forall v, forallb (ieval v) G = true -> ieval v C = true.
Proof.
  intros R G C H v T.
  pose proof (ll_tautology _ _ (ill_ll R G C H) v) as E.
  unfold one_sided in E. cbn in E. rewrite eval_itr, (duals_false v G T) in E.
  now rewrite orb_false_r in E.
Qed.

(** An assignment that falsifies the translation refutes a two-sided
    sequent. *)
Theorem iclassical_refutation : forall R v G C,
  falsifies v (one_sided G C) = true -> ~ ill R G C.
Proof.
  intros R v G C F H. exact (classical_refutation _ v _ F (ill_ll R G C H)).
Qed.

(** An unbalanced atom of the translation refutes a two-sided sequent. *)
Theorem iunbalanced_refutation : forall atom_eqb a R G C,
  unbalanced atom_eqb a (lrules R) (one_sided G C) = true -> ~ ill R G C.
Proof.
  intros atom_eqb a R G C U H. exact (unbalanced_refutation atom_eqb a _ _ U (ill_ll R G C H)).
Qed.

(** A failed count equation of the translation refutes a two-sided
    sequent. *)
Theorem iequation_refutation : forall R G C,
  fails_equation (lrules R) (one_sided G C) = true -> ~ ill R G C.
Proof.
  intros R G C F H. exact (equation_refutation _ _ F (ill_ll R G C H)).
Qed.

End Refutations.
