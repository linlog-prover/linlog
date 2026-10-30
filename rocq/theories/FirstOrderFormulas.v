(* linlog © Fabian Lukas Grubmüller 2026 *)
(* Licensed under the EUPL *)

(** * Formulas of first-order classical linear logic in negation normal form

    The formulas of [Formulas], with predicates applied to terms
    ([Terms]) in place of atoms and the two quantifiers [∀] and [∃],
    over a type of symbols that stays abstract (predicate and function
    symbols share it, as linlog's sequents do). A quantifier binds the
    bound variable [0] of its body, and the variables of the quantifiers
    around it from [1] on. Negation is the function [dual], so every
    formula is in negation normal form by construction. This development
    stands beside the propositional one and shares no name with it: a
    file that needs both requires them without importing one. *)

From Stdlib Require Import List.
From Linlog Require Export Terms.
Import ListNotations.

Section Formulas.

Context {symbol : Type}.
Local Notation term := (term symbol).

(** A formula: a literal, a unit, a binary connective, an exponential or
    a quantifier. *)
Inductive formula : Type :=
| var : symbol -> list term -> formula    (** a predicate [p(t₁, …, tₙ)] *)
| covar : symbol -> list term -> formula  (** its negation [p(t₁, …, tₙ)⊥] *)
| one : formula                           (** [1] *)
| bot : formula                           (** [⊥] *)
| top : formula                           (** [⊤] *)
| zero : formula                          (** [0] *)
| tens : formula -> formula -> formula    (** [A ⊗ B] *)
| parr : formula -> formula -> formula    (** [A ⅋ B] *)
| awith : formula -> formula -> formula   (** [A & B] *)
| aplus : formula -> formula -> formula   (** [A ⊕ B] *)
| oc : formula -> formula                 (** [!A] *)
| wn : formula -> formula                 (** [?A] *)
| fall : formula -> formula               (** [∀x. A], [x] the bound variable [0] of [A] *)
| fex : formula -> formula.               (** [∃x. A] *)

(** Linear negation: an involution that swaps each connective with its
    dual, [var] with [covar] and [∀] with [∃]. *)
Fixpoint dual (A : formula) : formula :=
  match A with
  | var p l => covar p l
  | covar p l => var p l
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
  | fall A => fex (dual A)
  | fex A => fall (dual A)
  end.

(** Substitutes the closed term [u] for the bound variable [k] of [A],
    which is [k] at the top and one more under every quantifier: [subst 0
    u A] is the instance at [u] of the body [A] of [∀x. A] or [∃x. A]. *)
Fixpoint subst (k : nat) (u : term) (A : formula) : formula :=
  match A with
  | var p l => var p (map (tsubst k u) l)
  | covar p l => covar p (map (tsubst k u) l)
  | one => one
  | bot => bot
  | top => top
  | zero => zero
  | tens A B => tens (subst k u A) (subst k u B)
  | parr A B => parr (subst k u A) (subst k u B)
  | awith A B => awith (subst k u A) (subst k u B)
  | aplus A B => aplus (subst k u A) (subst k u B)
  | oc A => oc (subst k u A)
  | wn A => wn (subst k u A)
  | fall A => fall (subst (S k) u A)
  | fex A => fex (subst (S k) u A)
  end.

(** Whether the eigenvariable [a] occurs in [A]. *)
Fixpoint occurs (a : nat) (A : formula) : bool :=
  match A with
  | var _ l | covar _ l => existsb (toccurs a) l
  | one | bot | top | zero => false
  | tens A B | parr A B | awith A B | aplus A B => occurs a A || occurs a B
  | oc A | wn A | fall A | fex A => occurs a A
  end.

End Formulas.

Arguments formula symbol : clear implicits.
