(* linlog © Fabian Lukas Grubmüller 2026 *)
(* Licensed under the EUPL *)

(** * First-order proof terms

    The certificate linlog writes for a first-order proof: the terms of
    [Proofs] over first-order formulas, the axiom naming its predicate and
    arguments, and two more nodes, [Forall] with the eigenvariable it
    introduces and [Exists] with its witness. Every formula a node names
    is an instance, the terms of the quantifiers below substituted for
    their variables, so the checker ([FirstOrderCheck]) compares formulas
    alone. The constructors mirror linlog's [Node] variants, in their
    names and their order. *)

From Linlog Require Export FirstOrderFormulas.

Section Proofs.

Context {symbol : Type}.
Local Notation formula := (formula symbol).
Local Notation term := (term symbol).

(** Which disjunct a [⊕] rule keeps. *)
Inductive branch : Type :=
| Left
| Right.

(** A proof term. Below, [⊢ Θ ; Γ] is a dyadic sequent with the
    unrestricted zone [Θ] and the linear zone [Γ]; the checker keeps the
    least [Θ] a subproof needs. *)
Inductive proof : Type :=
| Ax (p : symbol) (l : list term)
    (** the axiom [⊢ Θ ; p(l)⊥, p(l)] *)
| Tensor (A B : formula) (left right : proof)
    (** [⊢ Θ ; Γ, Δ, A ⊗ B] from [⊢ Θ ; Γ, A] and [⊢ Θ ; Δ, B] *)
| Par (A B : formula) (premise : proof)
    (** [⊢ Θ ; Γ, A ⅋ B] from [⊢ Θ ; Γ, A, B] *)
| One
    (** [⊢ Θ ; 1] *)
| Bot (premise : proof)
    (** [⊢ Θ ; Γ, ⊥] from [⊢ Θ ; Γ] *)
| With (A B : formula) (left right : proof)
    (** [⊢ Θ ; Γ, A & B] from [⊢ Θ ; Γ, A] and [⊢ Θ ; Γ, B] *)
| Plus (A B : formula) (side : branch) (premise : proof)
    (** [⊢ Θ ; Γ, A ⊕ B] from [⊢ Θ ; Γ, A] or from [⊢ Θ ; Γ, B] *)
| Top
    (** [⊢ Θ ; Γ, ⊤] for any [Γ] *)
| Bang (A : formula) (premise : proof)
    (** promotion, [⊢ Θ ; !A] from [⊢ Θ ; A] *)
| Quest (A : formula) (premise : proof)
    (** [⊢ Θ ; Γ, ?A] from [⊢ Θ, A ; Γ]: [A] moves into the unrestricted
        zone *)
| Copy (A : formula) (premise : proof)
    (** [⊢ Θ, A ; Γ] from [⊢ Θ, A ; Γ, A]: a use of [A] that leaves it
        in the unrestricted zone *)
| Weaken (A : formula) (premise : proof)
    (** [⊢ Θ ; Γ, A] from [⊢ Θ ; Γ], for a [?] formula [A] in every
        calculus and for any [A] with [weakening] *)
| Mix (left right : proof)
    (** [⊢ Θ ; Γ, Δ] from [⊢ Θ ; Γ] and [⊢ Θ ; Δ], with [mix] *)
| Cut (A : formula) (left right : proof)
    (** [⊢ Θ ; Γ, Δ] from [⊢ Θ ; Γ, A] and [⊢ Θ ; Δ, A⊥], with [cut] *)
| Forall (A : formula) (a : nat) (premise : proof)
    (** [⊢ Θ ; Γ, ∀x. A] from [⊢ Θ ; Γ, A[a/x]], the eigenvariable [a]
        occurring in no formula of the conclusion *)
| Exists (A : formula) (t : term) (premise : proof).
    (** [⊢ Θ ; Γ, ∃x. A] from [⊢ Θ ; Γ, A[t/x]], for a closed term [t] *)

End Proofs.

Arguments proof symbol : clear implicits.
