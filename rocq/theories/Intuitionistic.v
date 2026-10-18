(* linlog © Fabian Lukas Grubmüller 2026 *)
(* Licensed under the EUPL *)

(** * The two-sided sequent calculus of intuitionistic linear logic

    The trusted definition of the intuitionistic statements: [ill R Γ C]
    says that the sequent [Γ ⊢ C], with the hypotheses [Γ] and exactly
    one conclusion [C], has a cut-free derivation in intuitionistic
    linear logic, or in its affine variant when [R] adds weakening. The
    formulas and the rules are those of Yalla's standalone kernel
    [nanoill.v] and of the textbook calculus: the hypotheses are a list
    read as a multiset, so the exchange rule [ex_ir] admits any
    permutation, and every left rule acts on the first hypothesis.
    Nothing is derived here, and nothing is assumed. The derivations live
    in [Prop]: [ill R Γ C] says that one exists. *)

From Stdlib Require Import Bool List Permutation.
Import ListNotations.

Section Formulas.

Context {atom : Type}.

(** An intuitionistic formula. *)
Inductive iformula : Type :=
| ivar : atom -> iformula                    (** an atom [a] *)
| ione : iformula                            (** [1] *)
| itens : iformula -> iformula -> iformula   (** [A ⊗ B] *)
| ilmap : iformula -> iformula -> iformula   (** [A ⊸ B] *)
| itop : iformula                            (** [⊤] *)
| iwith : iformula -> iformula -> iformula   (** [A & B] *)
| izero : iformula                           (** [0] *)
| iplus : iformula -> iformula -> iformula   (** [A ⊕ B] *)
| ioc : iformula -> iformula.                (** [!A] *)

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

Arguments iformula atom : clear implicits.

(** The optional rule of a two-sided calculus: intuitionistic linear
    logic has none, affine logic adds the weakening of any hypothesis
    ([weakening_ilr] below), while the weakening of a [!] hypothesis,
    [wk_ilr], belongs to both. *)
Record irules : Type := IRules {
  iweakening : bool;  (** weakening of any hypothesis, not only of [!] ones *)
}.

(** Intuitionistic linear logic: no weakening but of [!] hypotheses. *)
Definition intuitionistic : irules := {| iweakening := false |}.

(** Affine intuitionistic logic: weakening of any hypothesis. *)
Definition affine : irules := {| iweakening := true |}.

Section Calculus.

Context {atom : Type}.
Local Notation iformula := (iformula atom).

(** The derivable sequents [Γ ⊢ C], with [R] the optional rule in
    force. *)
Inductive ill (R : irules) : list iformula -> iformula -> Prop :=
| ax_ir : forall a,
    ill R [ivar a] (ivar a)
| ex_ir : forall l1 l2 C,
    ill R l1 C -> Permutation l1 l2 -> ill R l2 C
| one_irr :
    ill R [] ione
| one_ilr : forall l C,
    ill R l C -> ill R (ione :: l) C
| tens_irr : forall A B l1 l2,
    ill R l1 A -> ill R l2 B -> ill R (l1 ++ l2) (itens A B)
| tens_ilr : forall A B l C,
    ill R (A :: B :: l) C -> ill R (itens A B :: l) C
| lmap_irr : forall A B l,
    ill R (A :: l) B -> ill R l (ilmap A B)
| lmap_ilr : forall A B l1 l2 C,
    ill R l1 A -> ill R (B :: l2) C -> ill R (ilmap A B :: l1 ++ l2) C
| top_irr : forall l,
    ill R l itop
| with_irr : forall A B l,
    ill R l A -> ill R l B -> ill R l (iwith A B)
| with_ilr1 : forall A B l C,
    ill R (A :: l) C -> ill R (iwith A B :: l) C
| with_ilr2 : forall A B l C,
    ill R (B :: l) C -> ill R (iwith A B :: l) C
| zero_ilr : forall l C,
    ill R (izero :: l) C
| plus_irr1 : forall A B l,
    ill R l A -> ill R l (iplus A B)
| plus_irr2 : forall A B l,
    ill R l B -> ill R l (iplus A B)
| plus_ilr : forall A B l C,
    ill R (A :: l) C -> ill R (B :: l) C -> ill R (iplus A B :: l) C
| oc_irr : forall A l,
    ill R (map ioc l) A -> ill R (map ioc l) (ioc A)
| de_ilr : forall A l C,
    ill R (A :: l) C -> ill R (ioc A :: l) C
| wk_ilr : forall A l C,
    ill R l C -> ill R (ioc A :: l) C
| co_ilr : forall A l C,
    ill R (ioc A :: ioc A :: l) C -> ill R (ioc A :: l) C
| weakening_ilr : iweakening R = true -> forall A l C,
    ill R l C -> ill R (A :: l) C.

End Calculus.
