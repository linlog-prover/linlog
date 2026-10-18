(* linlog © Fabian Lukas Grubmüller 2026 *)
(* Licensed under the EUPL *)

(** * Proof terms

    A proof term is the certificate linlog writes: one node per rule
    instance of the dyadic sequent calculus every engine searches in,
    naming the formulas the rule acts on and the subproofs of its
    premises, and never the sequent it proves. The checker ([Check])
    derives the sequents bottom-up, so a term is small and checking it
    is a computation. The constructors mirror linlog's [Node] variants,
    in their names and their order. *)

From Linlog Require Export Formulas.

Section Proofs.

Context {atom : Type}.
Local Notation formula := (formula atom).

(** Which disjunct a [⊕] rule keeps. *)
Inductive branch : Type :=
| Left
| Right.

(** A proof term. Below, [⊢ Θ ; Γ] is a dyadic sequent with the
    unrestricted zone [Θ] and the linear zone [Γ]; the checker keeps the
    least [Θ] a subproof needs. *)
Inductive proof : Type :=
| Ax (a : atom)
    (** the axiom [⊢ Θ ; a⊥, a] *)
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
| Mix (left right : proof).
    (** [⊢ Θ ; Γ, Δ] from [⊢ Θ ; Γ] and [⊢ Θ ; Δ], with [mix] *)

End Proofs.

Arguments proof atom : clear implicits.
