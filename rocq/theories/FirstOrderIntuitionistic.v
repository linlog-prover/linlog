(* linlog © Fabian Lukas Grubmüller 2026 *)
(* Licensed under the EUPL *)

(** * The two-sided sequent calculus of first-order intuitionistic linear logic

    The trusted definition of the first-order intuitionistic statements:
    [ill R Γ C] says that the sequent [Γ ⊢ C] has a derivation in
    first-order intuitionistic linear logic, or in its affine variant
    when [R] adds weakening, a cut-free one when [Intuitionistic.icut R
    = false] ([Intuitionistic.irules], the optional rules of the
    propositional calculus). The formulas and the rules are those of
    [Intuitionistic] with predicates applied to terms ([Terms]) and the
    quantifiers [∀] and [∃], each with its two rules: [∀R] and [∃L] with
    an eigenvariable that occurs nowhere in their conclusion, [∃R] and
    [∀L] with any closed term as the witness (an eigenvariable among
    them, a constant outside the signature). A quantifier binds the bound
    variable [0] of its body. Every left rule acts on the first
    hypothesis, the cut on the first hypothesis of its second premise,
    and the exchange rule [ex_ir] admits any permutation. Nothing is
    derived here, and nothing is assumed. The derivations live in
    [Prop]: [ill R Γ C] says that one exists. *)

From Stdlib Require Import List Permutation.
From Linlog Require Intuitionistic.
From Linlog Require Export Terms.
Import ListNotations.

Section Formulas.

Context {symbol : Type}.
Local Notation term := (term symbol).

(** A first-order intuitionistic formula. *)
Inductive iformula : Type :=
| ivar : symbol -> list term -> iformula     (** a predicate [p(t₁, …, tₙ)] *)
| ione : iformula                            (** [1] *)
| itens : iformula -> iformula -> iformula   (** [A ⊗ B] *)
| ilmap : iformula -> iformula -> iformula   (** [A ⊸ B] *)
| itop : iformula                            (** [⊤] *)
| iwith : iformula -> iformula -> iformula   (** [A & B] *)
| izero : iformula                           (** [0] *)
| iplus : iformula -> iformula -> iformula   (** [A ⊕ B] *)
| ioc : iformula -> iformula                 (** [!A] *)
| iall : iformula -> iformula                (** [∀x. A], [x] the bound variable [0] of [A] *)
| iex : iformula -> iformula.                (** [∃x. A] *)

(** Substitutes the closed term [u] for the bound variable [k] of [A],
    one more under every quantifier: [isubst 0 u A] is the instance at
    [u] of the body [A] of [∀x. A] or [∃x. A]. *)
Fixpoint isubst (k : nat) (u : term) (A : iformula) : iformula :=
  match A with
  | ivar p l => ivar p (map (tsubst k u) l)
  | ione => ione
  | itens A B => itens (isubst k u A) (isubst k u B)
  | ilmap A B => ilmap (isubst k u A) (isubst k u B)
  | itop => itop
  | iwith A B => iwith (isubst k u A) (isubst k u B)
  | izero => izero
  | iplus A B => iplus (isubst k u A) (isubst k u B)
  | ioc A => ioc (isubst k u A)
  | iall A => iall (isubst (S k) u A)
  | iex A => iex (isubst (S k) u A)
  end.

(** Whether the eigenvariable [a] occurs in [A]. *)
Fixpoint ioccurs (a : nat) (A : iformula) : bool :=
  match A with
  | ivar _ l => existsb (toccurs a) l
  | ione | itop | izero => false
  | itens A B | ilmap A B | iwith A B | iplus A B => ioccurs a A || ioccurs a B
  | ioc A | iall A | iex A => ioccurs a A
  end.

(** The eigenvariable [a] occurs in no formula of [l]. *)
Definition ifresh (a : nat) (l : list iformula) : Prop :=
  forall A, In A l -> ioccurs a A = false.

End Formulas.

Arguments iformula symbol : clear implicits.

Section Calculus.

Context {symbol : Type}.
Local Notation iformula := (iformula symbol).

(** The derivable sequents [Γ ⊢ C], with [R] the optional rules in
    force. *)
Inductive ill (R : Intuitionistic.irules) : list iformula -> iformula -> Prop :=
| ax_ir : forall p l,
    ill R [ivar p l] (ivar p l)
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
| weakening_ilr : Intuitionistic.iweakening R = true -> forall A l C,
    ill R l C -> ill R (A :: l) C
| cut_ir : Intuitionistic.icut R = true -> forall A l1 l2 C,
    ill R l1 A -> ill R (A :: l2) C -> ill R (l1 ++ l2) C
| all_irr : forall A a l,
    ifresh a (iall A :: l) -> ill R l (isubst 0 (eigen a) A) -> ill R l (iall A)
| all_ilr : forall A t l C,
    tclosed t = true -> ill R (isubst 0 t A :: l) C -> ill R (iall A :: l) C
| ex_irr : forall A t l,
    tclosed t = true -> ill R l (isubst 0 t A) -> ill R l (iex A)
| ex_ilr : forall A a l C,
    ifresh a (iex A :: C :: l) -> ill R (isubst 0 (eigen a) A :: l) C -> ill R (iex A :: l) C.

End Calculus.
