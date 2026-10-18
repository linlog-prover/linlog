(* linlog © Fabian Lukas Grubmüller 2026 *)
(* Licensed under the EUPL *)

(** * The checker

    [check atom_eqb R p Γ] computes whether the proof term [p] proves the
    sequent [⊢ Γ] in the calculus [R]. It derives, bottom-up, the dyadic
    sequent [⊢ Θ ; Γ] every subterm proves: [Θ] the least unrestricted
    zone the subterm needs (a [Copy] adds its formula, a [Quest] removes
    it), [Γ] the linear zone as a multiset, and a flag saying that a [⊤]
    above absorbs any further linear context. At the root [Θ] must be
    empty and [Γ] the sequent, or part of it under the flag. [Soundness]
    proves that a term the checker accepts yields a derivation. *)

From Stdlib Require Import Bool List.
From Linlog Require Export Calculus Proofs.
Import ListNotations.

(** Sequencing of partial computations: [k] runs on the value of [e], and
    [None] is failure. Local to this file, so that it clashes with no
    other library's. *)
Local Notation "'let*' x ':=' e 'in' k" :=
  (match e with Some x => k | None => None end)
  (at level 200, x pattern, e at level 100, k at level 200, right associativity).

Section Check.

Context {atom : Type} (atom_eqb : atom -> atom -> bool).
Local Notation formula := (formula atom).
Local Notation proof := (proof atom).
Local Notation eqb := (formula_eqb atom_eqb).

(** ** Multisets, as lists compared through [formula_eqb] *)

(** Removes one occurrence of [A] from [l], or fails when there is none. *)
Fixpoint remove_one (A : formula) (l : list formula) : option (list formula) :=
  match l with
  | [] => None
  | B :: l' => if eqb A B then Some l' else option_map (cons B) (remove_one A l')
  end.

(** Removes every occurrence of [A] from [l]. *)
Definition remove_all (A : formula) (l : list formula) : list formula :=
  filter (fun B => negb (eqb A B)) l.

(** Removes each formula of [l1] from [l2]: what is left of [l2] when [l1]
    is included in it as a multiset, [None] otherwise. *)
Fixpoint take_all (l1 l2 : list formula) : option (list formula) :=
  match l1 with
  | [] => Some l2
  | A :: l1' =>
      let* l2' := remove_one A l2 in
      take_all l1' l2'
  end.

(** [l2] less [l1]: each formula of [l1] removed from [l2] where it has
    one. [l1 ++ diff l2 l1] is the multiset union of the two. *)
Fixpoint diff (l2 l1 : list formula) : list formula :=
  match l1 with
  | [] => l2
  | A :: l1' =>
      diff (match remove_one A l2 with Some l2' => l2' | None => l2 end) l1'
  end.

(** ** States *)

(** What a subterm proves: the dyadic sequent [⊢ theta ; gamma], or with
    [any] set [⊢ theta ; gamma, Δ] for every [Δ], because a [⊤] leaf
    above absorbs whatever context it is given. [theta] is the least
    unrestricted zone the subterm needs; since weakening and contraction
    on it are implicit, any larger zone does as well. *)
Record state : Type := State {
  theta : list formula;  (** the unrestricted zone the subterm needs *)
  gamma : list formula;  (** the linear zone *)
  any : bool;            (** whether a [⊤] above absorbs more *)
}.

(** Consumes one copy of [A] from a premise's linear zone [gamma]: a [⊤]
    above ([any]) stands in for a missing one. *)
Definition take (A : formula) (gamma : list formula) (any : bool)
    : option (list formula) :=
  match remove_one A gamma with
  | Some gamma' => Some gamma'
  | None => if any then Some gamma else None
  end.

(** The conclusion of a [&] rule from its premises, after each consumed
    its own subformula: both premises must prove the same context. An
    exact premise fixes it, an absorbing one adapts to the other, and two
    absorbing premises absorb together over the union of their zones. *)
Definition join_with (A B : formula) (sl sr : state) : option state :=
  let theta := theta sl ++ theta sr in
  match any sl, any sr with
  | false, false =>
      match take_all (gamma sl) (gamma sr) with
      | Some [] => Some (State theta (awith A B :: gamma sl) false)
      | _ => None
      end
  | true, false =>
      let* _ := take_all (gamma sl) (gamma sr) in
      Some (State theta (awith A B :: gamma sr) false)
  | false, true =>
      let* _ := take_all (gamma sr) (gamma sl) in
      Some (State theta (awith A B :: gamma sl) false)
  | true, true =>
      Some (State theta (awith A B :: gamma sl ++ diff (gamma sr) (gamma sl)) true)
  end.

(** ** The pass *)

(** Derives what a term proves, or fails at the first node that misapplies
    its rule or uses one that [R] forbids. *)
Fixpoint derive (R : rules) (p : proof) : option state :=
  match p with
  | Ax a =>
      Some (State [] [covar a; var a] false)
  | Tensor A B l r =>
      let* sl := derive R l in
      let* sr := derive R r in
      let* gl := take A (gamma sl) (any sl) in
      let* gr := take B (gamma sr) (any sr) in
      Some (State (theta sl ++ theta sr) (tens A B :: gl ++ gr) (any sl || any sr))
  | Par A B q =>
      let* s := derive R q in
      let* g := take A (gamma s) (any s) in
      let* g := take B g (any s) in
      Some (State (theta s) (parr A B :: g) (any s))
  | One =>
      Some (State [] [one] false)
  | Bot q =>
      let* s := derive R q in
      Some (State (theta s) (bot :: gamma s) (any s))
  | With A B l r =>
      let* sl := derive R l in
      let* sr := derive R r in
      let* gl := take A (gamma sl) (any sl) in
      let* gr := take B (gamma sr) (any sr) in
      join_with A B (State (theta sl) gl (any sl)) (State (theta sr) gr (any sr))
  | Plus A B side q =>
      let* s := derive R q in
      let* g := take (match side with Left => A | Right => B end) (gamma s) (any s) in
      Some (State (theta s) (aplus A B :: g) (any s))
  | Top =>
      Some (State [] [top] true)
  | Bang A q =>
      let* s := derive R q in
      let* g := take A (gamma s) (any s) in
      match g with
      | [] => Some (State (theta s) [oc A] false)
      | _ :: _ => None
      end
  | Quest A q =>
      let* s := derive R q in
      Some (State (remove_all A (theta s)) (wn A :: gamma s) (any s))
  | Copy A q =>
      let* s := derive R q in
      let* g := take A (gamma s) (any s) in
      Some (State (A :: theta s) g (any s))
  | Weaken A q =>
      if weakening R || is_wn A then
        let* s := derive R q in
        Some (State (theta s) (A :: gamma s) (any s))
      else None
  | Mix l r =>
      if mix R then
        let* sl := derive R l in
        let* sr := derive R r in
        Some (State (theta sl ++ theta sr) (gamma sl ++ gamma sr) (any sl || any sr))
      else None
  end.

(** Whether [p] proves [⊢ sequent] in [R]: its root needs no unrestricted
    zone and derives the sequent's formulas exactly, or a part of them
    when a [⊤] absorbs the rest. *)
Definition check (R : rules) (p : proof) (sequent : list formula) : bool :=
  match derive R p with
  | Some (State [] gamma any) =>
      match take_all gamma sequent with
      | Some [] => true
      | Some (_ :: _) => any
      | None => false
      end
  | _ => false
  end.

End Check.

Arguments state atom : clear implicits.
