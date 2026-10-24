(* linlog © Fabian Lukas Grubmüller 2026 *)
(* Licensed under the EUPL *)

(** * The one-sided sequent calculus of classical linear logic

    The trusted definition: [ll R Γ] says that the sequent [⊢ Γ] has a
    derivation in the calculus whose optional rules [R] names, a
    cut-free one when [cut R = false]. A sequent is a list of formulas
    read as a multiset, so the exchange rule [ex_r] admits any
    permutation. The rules are the textbook ones, every rule but
    exchange, Mix and cut acting on the first formula of its conclusion
    or of its premises; nothing is derived here, and nothing is assumed.
    The derivations live in [Prop]: [ll R Γ] says that one exists. *)

From Stdlib Require Import List Permutation.
From Linlog Require Export Formulas.
Import ListNotations.

(** The optional rules of a calculus: classical linear logic has none,
    Mix adds the first, affine logic the second, and a calculus with cut
    the third. Mix is the binary rule alone, never the nullary one ([⊢]
    with nothing); [weakening] is the rule [weakening_r] below, while
    the weakening of [?] formulas, [wk_r], belongs to every calculus;
    [cut] is the rule [cut_r]. *)
Record rules : Type := Rules {
  mix : bool;        (** binary Mix: [⊢ Γ, Δ] from [⊢ Γ] and [⊢ Δ] *)
  weakening : bool;  (** weakening of any formula, not only of [?] ones *)
  cut : bool;        (** cut: [⊢ Γ, Δ] from [⊢ A, Γ] and [⊢ A⊥, Δ] *)
}.

(** Classical linear logic: no Mix, no weakening, no cut. *)
Definition classical : rules := {| mix := false; weakening := false; cut := false |}.

Section Calculus.

Context {atom : Type}.
Local Notation formula := (formula atom).

(** The derivable sequents, with [R] the optional rules in force. *)
Inductive ll (R : rules) : list formula -> Prop :=
| ax_r : forall a,
    ll R [covar a; var a]
| ex_r : forall l1 l2,
    ll R l1 -> Permutation l1 l2 -> ll R l2
| one_r :
    ll R [one]
| bot_r : forall l,
    ll R l -> ll R (bot :: l)
| top_r : forall l,
    ll R (top :: l)
| tens_r : forall A B l1 l2,
    ll R (A :: l1) -> ll R (B :: l2) -> ll R (tens A B :: l1 ++ l2)
| parr_r : forall A B l,
    ll R (A :: B :: l) -> ll R (parr A B :: l)
| with_r : forall A B l,
    ll R (A :: l) -> ll R (B :: l) -> ll R (awith A B :: l)
| plus_r1 : forall A B l,
    ll R (A :: l) -> ll R (aplus A B :: l)
| plus_r2 : forall A B l,
    ll R (B :: l) -> ll R (aplus A B :: l)
| oc_r : forall A l,
    ll R (A :: map wn l) -> ll R (oc A :: map wn l)
| de_r : forall A l,
    ll R (A :: l) -> ll R (wn A :: l)
| wk_r : forall A l,
    ll R l -> ll R (wn A :: l)
| co_r : forall A l,
    ll R (wn A :: wn A :: l) -> ll R (wn A :: l)
| mix_r : mix R = true -> forall l1 l2,
    ll R l1 -> ll R l2 -> ll R (l1 ++ l2)
| weakening_r : weakening R = true -> forall A l,
    ll R l -> ll R (A :: l)
| cut_r : cut R = true -> forall A l1 l2,
    ll R (A :: l1) -> ll R (dual A :: l2) -> ll R (l1 ++ l2).

End Calculus.
