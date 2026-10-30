(* linlog © Fabian Lukas Grubmüller 2026 *)
(* Licensed under the EUPL *)

(** * The checker of first-order proof terms

    [check symbol_eqb R p Γ] computes whether the proof term [p] proves
    the sequent [⊢ Γ] in the first-order calculus [R], in two passes, as
    linlog's checker does. The first, [introduced], collects the
    eigenvariables the [∀] steps introduce and fails where a [∀] step's
    eigenvariable is introduced again above it. The second, [derive], is
    the propositional checker's pass ([Check]) over first-order formulas,
    a [∀] step consuming its body's instance at its eigenvariable and an
    [∃] step its body's instance at its witness, and it carries a set of
    eigenvariables to the root: an [∃] step adds those of its witness, a
    cut those of its formula, and a [∀] step removes its own. At the root
    no eigenvariable a [∀] step introduces may be left in the set or occur
    in the sequent; one that no step introduces is a constant like any
    other. [FirstOrderSoundness] proves that a term the checker accepts
    yields a derivation, the condition on eigenvariables included. *)

From Stdlib Require Import Bool List.
From Linlog Require Export TermsCheck FirstOrderCalculus FirstOrderProofs.
Import ListNotations.

(** Sequencing of partial computations: [k] runs on the value of [e], and
    [None] is failure. Local to this file. *)
Local Notation "'let*' x ':=' e 'in' k" :=
  (match e with Some x => k | None => None end)
  (at level 200, x pattern, e at level 100, k at level 200, right associativity).

Section Equality.

Context {symbol : Type} (symbol_eqb : symbol -> symbol -> bool).
Local Notation term := (term symbol).
Local Notation formula := (formula symbol).

(** Structural equality of formulas. *)
Fixpoint formula_eqb (A B : formula) : bool :=
  match A, B with
  | var p l, var q k => symbol_eqb p q && terms_eqb symbol_eqb l k
  | covar p l, covar q k => symbol_eqb p q && terms_eqb symbol_eqb l k
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
  | fall A, fall B => formula_eqb A B
  | fex A, fex B => formula_eqb A B
  | _, _ => false
  end.

(** [symbol_eqb] decides equality: what the soundness theorem assumes of
    it, and what the certificates prove of the equality they use. *)
Hypothesis symbol_eqb_spec : forall f g, reflect (f = g) (symbol_eqb f g).

Lemma formula_eqb_spec : forall A B, reflect (A = B) (formula_eqb A B).
Proof.
  induction A; destruct B; cbn; try (constructor; congruence).
  all: repeat match goal with
       | |- reflect _ (symbol_eqb ?a ?b && _) => destruct (symbol_eqb_spec a b)
       | |- reflect _ (terms_eqb _ ?l ?k) => destruct (terms_eqb_spec symbol_eqb symbol_eqb_spec l k)
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

Section Eigenvariables.

Context {symbol : Type}.
Local Notation term := (term symbol).
Local Notation formula := (formula symbol).

(** The eigenvariables of a formula, with repetitions. *)
Fixpoint feigens (A : formula) : list nat :=
  match A with
  | var _ l | covar _ l => flat_map teigens l
  | one | bot | top | zero => []
  | tens A B | parr A B | awith A B | aplus A B => feigens A ++ feigens B
  | oc A | wn A | fall A | fex A => feigens A
  end.

(** Whether a formula is a [?] formula, which weakening may always
    introduce. *)
Definition is_wn (A : formula) : bool :=
  match A with
  | wn _ => true
  | _ => false
  end.

End Eigenvariables.

Section Check.

Context {symbol : Type} (symbol_eqb : symbol -> symbol -> bool).
Local Notation term := (term symbol).
Local Notation formula := (formula symbol).
Local Notation proof := (proof symbol).
Local Notation eqb := (formula_eqb symbol_eqb).

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
    [any] set [⊢ theta ; gamma, Δ] for every [Δ] in which no eigenvariable
    its [∀] steps introduce occurs, because a [⊤] leaf above absorbs
    whatever context it is given; and [eigens], the eigenvariables its
    [∃] steps' witnesses and its cut formulas bring in and no [∀] step
    below them removes. *)
Record state : Type := State {
  theta : list formula;  (** the unrestricted zone the subterm needs *)
  gamma : list formula;  (** the linear zone *)
  any : bool;            (** whether a [⊤] above absorbs more *)
  eigens : list nat;     (** the eigenvariables carried to the root *)
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
  let eigens := eigens sl ++ eigens sr in
  match any sl, any sr with
  | false, false =>
      match take_all (gamma sl) (gamma sr) with
      | Some [] => Some (State theta (awith A B :: gamma sl) false eigens)
      | _ => None
      end
  | true, false =>
      let* _ := take_all (gamma sl) (gamma sr) in
      Some (State theta (awith A B :: gamma sr) false eigens)
  | false, true =>
      let* _ := take_all (gamma sr) (gamma sl) in
      Some (State theta (awith A B :: gamma sl) false eigens)
  | true, true =>
      Some (State theta (awith A B :: gamma sl ++ diff (gamma sr) (gamma sl)) true eigens)
  end.

(** ** The first pass *)

(** The eigenvariables the [∀] steps of [p] introduce, or [None] when a
    [∀] step's eigenvariable is introduced again above it. *)
Fixpoint introduced (p : proof) : option (list nat) :=
  match p with
  | Ax _ _ | One | Top => Some []
  | Par _ _ q | Bot q | Plus _ _ _ q | Bang _ q | Quest _ q | Copy _ q | Weaken _ q
  | Exists _ _ q => introduced q
  | Tensor _ _ l r | With _ _ l r | Mix l r | Cut _ l r =>
      let* il := introduced l in
      let* ir := introduced r in
      Some (il ++ ir)
  | Forall _ a q =>
      let* i := introduced q in
      if existsb (Nat.eqb a) i then None else Some (a :: i)
  end.

(** ** The second pass *)

(** Derives what a term proves, or fails at the first node that misapplies
    its rule or uses one that [R] forbids. *)
Fixpoint derive (R : Calculus.rules) (p : proof) : option state :=
  match p with
  | Ax a l =>
      Some (State [] [covar a l; var a l] false [])
  | Tensor A B l r =>
      let* sl := derive R l in
      let* sr := derive R r in
      let* gl := take A (gamma sl) (any sl) in
      let* gr := take B (gamma sr) (any sr) in
      Some (State (theta sl ++ theta sr) (tens A B :: gl ++ gr) (any sl || any sr)
        (eigens sl ++ eigens sr))
  | Par A B q =>
      let* s := derive R q in
      let* g := take A (gamma s) (any s) in
      let* g := take B g (any s) in
      Some (State (theta s) (parr A B :: g) (any s) (eigens s))
  | One =>
      Some (State [] [one] false [])
  | Bot q =>
      let* s := derive R q in
      Some (State (theta s) (bot :: gamma s) (any s) (eigens s))
  | With A B l r =>
      let* sl := derive R l in
      let* sr := derive R r in
      let* gl := take A (gamma sl) (any sl) in
      let* gr := take B (gamma sr) (any sr) in
      join_with A B (State (theta sl) gl (any sl) (eigens sl))
        (State (theta sr) gr (any sr) (eigens sr))
  | Plus A B side q =>
      let* s := derive R q in
      let* g := take (match side with Left => A | Right => B end) (gamma s) (any s) in
      Some (State (theta s) (aplus A B :: g) (any s) (eigens s))
  | Top =>
      Some (State [] [top] true [])
  | Bang A q =>
      let* s := derive R q in
      let* g := take A (gamma s) (any s) in
      match g with
      | [] => Some (State (theta s) [oc A] false (eigens s))
      | _ :: _ => None
      end
  | Quest A q =>
      let* s := derive R q in
      Some (State (remove_all A (theta s)) (wn A :: gamma s) (any s) (eigens s))
  | Copy A q =>
      let* s := derive R q in
      let* g := take A (gamma s) (any s) in
      Some (State (A :: theta s) g (any s) (eigens s))
  | Weaken A q =>
      if Calculus.weakening R || is_wn A then
        let* s := derive R q in
        Some (State (theta s) (A :: gamma s) (any s) (eigens s))
      else None
  | Mix l r =>
      if Calculus.mix R then
        let* sl := derive R l in
        let* sr := derive R r in
        Some (State (theta sl ++ theta sr) (gamma sl ++ gamma sr) (any sl || any sr)
          (eigens sl ++ eigens sr))
      else None
  | Cut A l r =>
      if Calculus.cut R then
        let* sl := derive R l in
        let* sr := derive R r in
        let* gl := take A (gamma sl) (any sl) in
        let* gr := take (dual A) (gamma sr) (any sr) in
        Some (State (theta sl ++ theta sr) (gl ++ gr) (any sl || any sr)
          (feigens A ++ eigens sl ++ eigens sr))
      else None
  | Forall A a q =>
      let* s := derive R q in
      let* g := take (subst 0 (eigen a) A) (gamma s) (any s) in
      Some (State (theta s) (fall A :: g) (any s)
        (filter (fun b => negb (Nat.eqb a b)) (eigens s)))
  | Exists A t q =>
      if tclosed t then
        let* s := derive R q in
        let* g := take (subst 0 t A) (gamma s) (any s) in
        Some (State (theta s) (fex A :: g) (any s) (teigens t ++ eigens s))
      else None
  end.

(** Whether [p] proves [⊢ sequent] in [R]: no eigenvariable its [∀]
    steps introduce is left in the set its root carries or occurs in the
    sequent, and its root needs no unrestricted zone and derives the
    sequent's formulas exactly, or a part of them when a [⊤] absorbs the
    rest. *)
Definition check (R : Calculus.rules) (p : proof) (sequent : list formula) : bool :=
  match introduced p, derive R p with
  | Some i, Some (State [] gamma any e) =>
      forallb (fun a => negb (existsb (Nat.eqb a) e)
        && forallb (fun A => negb (occurs a A)) sequent) i &&
      match take_all gamma sequent with
      | Some [] => true
      | Some (_ :: _) => any
      | None => false
      end
  | _, _ => false
  end.

End Check.

Arguments state symbol : clear implicits.
