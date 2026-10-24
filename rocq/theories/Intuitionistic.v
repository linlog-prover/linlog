(* linlog © Fabian Lukas Grubmüller 2026 *)
(* Licensed under the EUPL *)

(** * The two-sided sequent calculus of intuitionistic linear logic

    The trusted definition of the intuitionistic statements: [ill R Γ C]
    says that the sequent [Γ ⊢ C], with the hypotheses [Γ] and exactly
    one conclusion [C], has a derivation in intuitionistic linear logic,
    or in its affine variant when [R] adds weakening, a cut-free one
    when [icut R = false]. The formulas and the rules are those of
    Yalla's standalone kernel [nanoill.v] and of the textbook calculus
    over [1], [⊗], [⊸], [⊤], [&], [0], [⊕] and [!], without [⊥]: every
    sequent has exactly one conclusion, so no rule weakens on the right,
    and the axiom is on atoms (the identity on any formula is
    derivable); [nanoill.v] has no cut, which [R] adds here. The
    hypotheses are a list read as a multiset, so the exchange rule
    [ex_ir] admits any permutation, and every left rule acts on the first
    hypothesis, the cut on the first hypothesis of its second premise. Nothing is derived here, and nothing is assumed. The derivations live
    in [Prop]: [ill R Γ C] says that one exists. *)

From Stdlib Require Import List Permutation.
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


End Formulas.

Arguments iformula atom : clear implicits.

(** The optional rules of a two-sided calculus: intuitionistic linear
    logic has none, affine logic adds the weakening of any hypothesis
    ([weakening_ilr] below), never of the conclusion, while the weakening
    of a [!] hypothesis, [wk_ilr], belongs to both; a calculus with cut
    adds [cut_ir]. *)
Record irules : Type := IRules {
  iweakening : bool;  (** weakening of any hypothesis, not only of [!] ones *)
  icut : bool;        (** cut: [Γ, Δ ⊢ C] from [Γ ⊢ A] and [Δ, A ⊢ C] *)
}.

(** Intuitionistic linear logic: no weakening but of [!] hypotheses, no
    cut. *)
Definition intuitionistic : irules := {| iweakening := false; icut := false |}.

(** Affine intuitionistic logic: weakening of any hypothesis, no cut. *)
Definition affine : irules := {| iweakening := true; icut := false |}.

Section Calculus.

Context {atom : Type}.
Local Notation iformula := (iformula atom).

(** The derivable sequents [Γ ⊢ C], with [R] the optional rules in
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
    ill R l C -> ill R (A :: l) C
| cut_ir : icut R = true -> forall A l1 l2 C,
    ill R l1 A -> ill R (A :: l2) C -> ill R (l1 ++ l2) C.

End Calculus.
