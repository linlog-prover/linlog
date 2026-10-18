(* linlog © Fabian Lukas Grubmüller 2026 *)
(* Licensed under the EUPL *)

(** * Formulas of classical linear logic in negation normal form

    Formulas are built over a type of atoms that stays abstract: the
    calculus ([Calculus]) needs nothing of it, and the checker ([Check])
    needs only a Boolean equality that decides it. Negation is the
    function [dual], not a connective, so every formula is in negation
    normal form by construction. The constructors are named as in
    Yalla and NanoYalla. *)

From Stdlib Require Import Bool List.
Import ListNotations.

Section Formulas.

Context {atom : Type}.

(** A formula: a literal, a unit, a binary connective or an exponential. *)
Inductive formula : Type :=
| var : atom -> formula                   (** an atom [a] *)
| covar : atom -> formula                 (** its negation [a⊥] *)
| one : formula                           (** [1] *)
| bot : formula                           (** [⊥] *)
| top : formula                           (** [⊤] *)
| zero : formula                          (** [0] *)
| tens : formula -> formula -> formula    (** [A ⊗ B] *)
| parr : formula -> formula -> formula    (** [A ⅋ B] *)
| awith : formula -> formula -> formula   (** [A & B] *)
| aplus : formula -> formula -> formula   (** [A ⊕ B] *)
| oc : formula -> formula                 (** [!A] *)
| wn : formula -> formula.                (** [?A] *)

(** Linear negation: an involution that swaps each connective with its
    dual and [var] with [covar]. *)
Fixpoint dual (A : formula) : formula :=
  match A with
  | var a => covar a
  | covar a => var a
  | one => bot
  | bot => one
  | top => zero
  | zero => top
  | tens A B => parr (dual A) (dual B)
  | parr A B => tens (dual A) (dual B)
  | awith A B => aplus (dual A) (dual B)
  | aplus A B => awith (dual A) (dual B)
  | oc A => wn (dual A)
  | wn A => oc (dual A)
  end.

Lemma dual_dual : forall A, dual (dual A) = A.
Proof. induction A; cbn; congruence. Qed.

(** Whether a formula is a [?] formula, which weakening may always
    introduce. *)
Definition is_wn (A : formula) : bool :=
  match A with
  | wn _ => true
  | _ => false
  end.

Section Equality.

(** A Boolean equality on atoms, which the checker compares literals
    with. *)
Variable atom_eqb : atom -> atom -> bool.

(** Structural equality of formulas, decided through [atom_eqb]. *)
Fixpoint formula_eqb (A B : formula) : bool :=
  match A, B with
  | var a, var b => atom_eqb a b
  | covar a, covar b => atom_eqb a b
  | one, one => true
  | bot, bot => true
  | top, top => true
  | zero, zero => true
  | tens A1 A2, tens B1 B2 => formula_eqb A1 B1 && formula_eqb A2 B2
  | parr A1 A2, parr B1 B2 => formula_eqb A1 B1 && formula_eqb A2 B2
  | awith A1 A2, awith B1 B2 => formula_eqb A1 B1 && formula_eqb A2 B2
  | aplus A1 A2, aplus B1 B2 => formula_eqb A1 B1 && formula_eqb A2 B2
  | oc A, oc B => formula_eqb A B
  | wn A, wn B => formula_eqb A B
  | _, _ => false
  end.

(** [atom_eqb] decides equality: what the soundness theorem assumes of
    it, and what the certificates prove of the equality they use. *)
Hypothesis atom_eqb_spec : forall a b, reflect (a = b) (atom_eqb a b).

Lemma formula_eqb_spec : forall A B, reflect (A = B) (formula_eqb A B).
Proof.
  induction A; destruct B; cbn; try (constructor; congruence).
  all: repeat match goal with
       | |- reflect _ (atom_eqb ?a ?b) => destruct (atom_eqb_spec a b)
       | |- reflect _ (formula_eqb ?A ?B && _) =>
           match goal with IH : forall B, reflect (A = B) _ |- _ => destruct (IH B) end
       | |- reflect _ (formula_eqb ?A ?B) =>
           match goal with IH : forall B, reflect (A = B) _ |- _ => destruct (IH B) end
       | |- reflect _ (true && _) => cbn
       | |- reflect _ (false && _) => cbn
       end.
  all: constructor; congruence.
Qed.

(** Equality of formulas as a decision, for the proofs; the checker
    computes with [formula_eqb]. *)
Definition formula_eq_dec (A B : formula) : {A = B} + {A <> B} :=
  reflect_dec _ _ (formula_eqb_spec A B).

End Equality.

End Formulas.

Arguments formula atom : clear implicits.
