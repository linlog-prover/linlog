(* linlog © Fabian Lukas Grubmüller 2026 *)
(* Licensed under the EUPL *)

(** * Two-sided proof terms, their checker and its soundness

    A two-sided term is the certificate linlog writes for an
    intuitionistic proof: one node per inference of its two-sided
    derivation, named by the rule and the side of [⊢] its principal
    formula stands on, naming the immediate subformulas of that formula
    and the subterms of its premises. A [⊤] on the right and a [0] on the
    left name the context they absorb, so every node's sequent is
    determined by its subterms. [icheck] computes that sequent bottom-up
    and compares it with the one stated; [soundness_ill] proves that a
    term it accepts yields a derivation in [ill].

    The one-sided terms of [Proofs] do not determine this statement: a
    one-sided formula does not say which side of [⊢] it stands on when it
    is built from [⊤] and [0] alone ([top] is both [⊤] on the right and
    [0] on the left), and a derivation's single conclusion is a property
    of those sides. Here every formula of every sequent is an
    intuitionistic formula on a known side, and a sequent has its one
    conclusion by its type. *)

From Stdlib Require Import Bool List Permutation Lia.
From Linlog Require Export Intuitionistic.
Import ListNotations.

Section Formulas.

Context {atom : Type}.
Local Notation iformula := (iformula atom).

Section Equality.

(** A Boolean equality on atoms, which the checker compares atoms
    with. *)
Variable atom_eqb : atom -> atom -> bool.

(** Structural equality of intuitionistic formulas, decided through
    [atom_eqb]. *)
Fixpoint iformula_eqb (A B : iformula) : bool :=
  match A, B with
  | ivar a, ivar b => atom_eqb a b
  | ione, ione => true
  | itop, itop => true
  | izero, izero => true
  | itens A1 A2, itens B1 B2 => iformula_eqb A1 B1 && iformula_eqb A2 B2
  | ilmap A1 A2, ilmap B1 B2 => iformula_eqb A1 B1 && iformula_eqb A2 B2
  | iwith A1 A2, iwith B1 B2 => iformula_eqb A1 B1 && iformula_eqb A2 B2
  | iplus A1 A2, iplus B1 B2 => iformula_eqb A1 B1 && iformula_eqb A2 B2
  | ioc A, ioc B => iformula_eqb A B
  | _, _ => false
  end.

(** [atom_eqb] decides equality, as the soundness theorem assumes. *)
Hypothesis atom_eqb_spec : forall a b, reflect (a = b) (atom_eqb a b).

Lemma iformula_eqb_spec : forall A B, reflect (A = B) (iformula_eqb A B).
Proof.
  induction A; destruct B; cbn; try (constructor; congruence).
  all: repeat match goal with
       | |- reflect _ (atom_eqb ?a ?b) => destruct (atom_eqb_spec a b)
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

End Formulas.

Section Check.

Context {atom : Type} (atom_eqb : atom -> atom -> bool).
Local Notation iformula := (iformula atom).
Local Notation eqb := (iformula_eqb atom_eqb).

(** A two-sided proof term. [Γ ⊢ C] below is the conclusion of the node,
    the hypotheses read as a multiset. *)
Inductive iproof : Type :=
| IAx (a : atom)
    (** the axiom [a ⊢ a] *)
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
| IWeaken (A : iformula) (premise : iproof).
    (** weakening of any hypothesis, [Γ, A ⊢ C] from [Γ ⊢ C], with
        [iweakening] *)

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

(** ** The pass *)

(** The conclusion [Γ ⊢ C] of a term, or [None] at the first node that
    misapplies its rule or uses one [R] forbids. *)
Fixpoint iderive (R : irules) (p : iproof) : option (list iformula * iformula) :=
  match p with
  | IAx a => Some ([ivar a], ivar a)
  | IOneR => Some ([], ione)
  | IOneL q =>
      match iderive R q with
      | Some (l, C) => Some (ione :: l, C)
      | None => None
      end
  | ITensR A B q r =>
      match iderive R q, iderive R r with
      | Some (l1, A'), Some (l2, B') =>
          if eqb A A' && eqb B B' then Some (l1 ++ l2, itens A B) else None
      | _, _ => None
      end
  | ITensL A B q =>
      match iderive R q with
      | Some (l, C) =>
          match pick A l with
          | Some l' =>
              match pick B l' with
              | Some l'' => Some (itens A B :: l'', C)
              | None => None
              end
          | None => None
          end
      | None => None
      end
  | ILmapR A B q =>
      match iderive R q with
      | Some (l, B') =>
          if eqb B B' then
            match pick A l with
            | Some l' => Some (l', ilmap A B)
            | None => None
            end
          else None
      | None => None
      end
  | ILmapL A B q r =>
      match iderive R q, iderive R r with
      | Some (l1, A'), Some (l2, C) =>
          if eqb A A' then
            match pick B l2 with
            | Some l2' => Some (ilmap A B :: l1 ++ l2', C)
            | None => None
            end
          else None
      | _, _ => None
      end
  | ITopR l => Some (l, itop)
  | IWithR A B q r =>
      match iderive R q, iderive R r with
      | Some (l1, A'), Some (l2, B') =>
          if eqb A A' && eqb B B' && same l1 l2 then Some (l1, iwith A B) else None
      | _, _ => None
      end
  | IWithL1 A B q =>
      match iderive R q with
      | Some (l, C) =>
          match pick A l with
          | Some l' => Some (iwith A B :: l', C)
          | None => None
          end
      | None => None
      end
  | IWithL2 A B q =>
      match iderive R q with
      | Some (l, C) =>
          match pick B l with
          | Some l' => Some (iwith A B :: l', C)
          | None => None
          end
      | None => None
      end
  | IZeroL l C => Some (izero :: l, C)
  | IPlusR1 A B q =>
      match iderive R q with
      | Some (l, A') => if eqb A A' then Some (l, iplus A B) else None
      | None => None
      end
  | IPlusR2 A B q =>
      match iderive R q with
      | Some (l, B') => if eqb B B' then Some (l, iplus A B) else None
      | None => None
      end
  | IPlusL A B q r =>
      match iderive R q, iderive R r with
      | Some (l1, C1), Some (l2, C2) =>
          match pick A l1, pick B l2 with
          | Some l1', Some l2' =>
              if eqb C1 C2 && same l1' l2' then Some (iplus A B :: l1', C1) else None
          | _, _ => None
          end
      | _, _ => None
      end
  | IOcR A q =>
      match iderive R q with
      | Some (l, A') =>
          match unoc l with
          | Some _ => if eqb A A' then Some (l, ioc A) else None
          | None => None
          end
      | None => None
      end
  | IDe A q =>
      match iderive R q with
      | Some (l, C) =>
          match pick A l with
          | Some l' => Some (ioc A :: l', C)
          | None => None
          end
      | None => None
      end
  | IWk A q =>
      match iderive R q with
      | Some (l, C) => Some (ioc A :: l, C)
      | None => None
      end
  | ICo A q =>
      match iderive R q with
      | Some (l, C) =>
          match pick (ioc A) l with
          | Some l' =>
              match pick (ioc A) l' with
              | Some l'' => Some (ioc A :: l'', C)
              | None => None
              end
          | None => None
          end
      | None => None
      end
  | IWeaken A q =>
      if iweakening R then
        match iderive R q with
        | Some (l, C) => Some (A :: l, C)
        | None => None
        end
      else None
  end.

(** Whether [p] proves [hypotheses ⊢ goal] in [R]: its conclusion has
    that goal and those hypotheses, as a multiset. *)
Definition icheck (R : irules) (p : iproof) (hypotheses : list iformula) (goal : iformula)
    : bool :=
  match iderive R p with
  | Some (l, C) => eqb C goal && same l hypotheses
  | None => false
  end.

End Check.

Arguments iproof atom : clear implicits.

Section Soundness.

Context {atom : Type} (atom_eqb : atom -> atom -> bool)
  (atom_eqb_spec : forall a b, reflect (a = b) (atom_eqb a b)).
Local Notation iformula := (iformula atom).
Local Notation eqb_spec := (iformula_eqb_spec atom_eqb atom_eqb_spec).

Lemma pick_perm : forall (A : iformula) l l',
  pick atom_eqb A l = Some l' -> Permutation l (A :: l').
Proof.
  induction l as [| B l IH]; cbn; intros l' H; [discriminate |].
  destruct (eqb_spec A B) as [<- | _].
  - injection H as <-. reflexivity.
  - destruct (pick atom_eqb A l) as [l'' |] eqn:E; cbn in H; [| discriminate].
    injection H as <-. rewrite (IH l'' eq_refl). apply perm_swap.
Qed.

Lemma same_perm : forall (l1 l2 : list iformula),
  same atom_eqb l1 l2 = true -> Permutation l1 l2.
Proof.
  induction l1 as [| A l1 IH]; cbn; intros l2 H.
  - destruct l2; [reflexivity | discriminate].
  - destruct (pick atom_eqb A l2) as [l2' |] eqn:E; [| discriminate].
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

Local Notation eq_dec := (iformula_eq_dec atom_eqb atom_eqb_spec).

(** Rewrites with the Boolean equalities of formulas a node compared. *)
Local Ltac equalities :=
  repeat match goal with
  | H : (_ && _) = true |- _ => apply andb_prop in H; destruct H
  | H : iformula_eqb atom_eqb ?A ?B = true |- _ =>
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

Variable R : irules.

(** Proves a premise from the derivation of a subterm, up to a
    permutation of its hypotheses. *)
Local Ltac premise :=
  match goal with
  | H : ill R ?l ?C |- ill R _ ?C => apply (ex_ir R l _ C H); iperm
  end.

(** Every conclusion the pass computes is derivable. *)
Lemma iderive_sound : forall p l C,
  iderive atom_eqb R p = Some (l, C) -> ill R l C.
Proof.
  induction p; cbn; intros l C H; steps; equalities;
    repeat match goal with
    | H : Some _ = Some _ |- _ => injection H as <- <-
    | IH : forall l C, Some (?l', ?C') = Some (l, C) -> _ |- _ =>
        specialize (IH l' C' eq_refl)
    | H : pick _ _ _ = Some _ |- _ => apply pick_perm in H
    | H : same _ _ _ = true |- _ => apply same_perm in H
    | H : unoc ?l = Some _ |- _ => apply unoc_map in H; subst l
    end.
  - apply ax_ir.
  - apply one_irr.
  - apply one_ilr; premise.
  - apply tens_irr; premise.
  - apply tens_ilr; premise.
  - apply lmap_irr; premise.
  - apply lmap_ilr; premise.
  - apply top_irr.
  - apply with_irr; premise.
  - apply with_ilr1; premise.
  - apply with_ilr2; premise.
  - apply zero_ilr.
  - apply plus_irr1; premise.
  - apply plus_irr2; premise.
  - apply plus_ilr; premise.
  - apply oc_irr; premise.
  - apply de_ilr; premise.
  - apply wk_ilr; premise.
  - apply co_ilr; premise.
  - apply weakening_ilr; [assumption | premise].
Qed.

End Soundness.

(** A term the checker accepts for a sequent proves it in the
    two-sided calculus [R]: intuitionistic linear logic or its affine
    variant. *)
Theorem soundness_ill {atom : Type} (atom_eqb : atom -> atom -> bool)
  (atom_eqb_spec : forall a b, reflect (a = b) (atom_eqb a b)) (R : irules) :
  forall (p : iproof atom) (hypotheses : list (iformula atom)) (goal : iformula atom),
  icheck atom_eqb R p hypotheses goal = true -> ill R hypotheses goal.
Proof.
  unfold icheck; intros p Γ C H.
  destruct (iderive atom_eqb R p) as [[l C'] |] eqn:E; [| discriminate].
  apply andb_prop in H as [HC HΓ].
  destruct (iformula_eqb_spec atom_eqb atom_eqb_spec C' C); [subst | discriminate].
  eapply ex_ir; [apply (iderive_sound atom_eqb atom_eqb_spec R _ _ _ E) |].
  apply (same_perm atom_eqb atom_eqb_spec). assumption.
Qed.
