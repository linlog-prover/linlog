(* linlog © Fabian Lukas Grubmüller 2026 *)
(* Licensed under the EUPL *)

(** Certificates of first-order proofs, one-sided, in the form the
    exporter writes: the sequent over a first-order calculus, proved by
    its soundness theorem from a term the checker accepts by
    computation, with strings as symbols. The last lemmas pin what the
    checker refuses: a wrong witness, an eigenvariable that escapes its
    [∀] step through a witness below it, one introduced again above its
    step, one that the sequent names, and a witness with a bound
    variable. *)

From Stdlib Require Import List String.
From Linlog Require Calculus.
From Linlog Require Import FirstOrderSoundness.
Import ListNotations.
Open Scope string_scope.

(** [∀x. p(x) ⊢ ∃y. p(y)]: domains are not empty, so a constant no [∀]
    step introduces, the eigenvariable [0], is a witness. *)
Lemma nonempty : ll (symbol := string) Calculus.classical
  [fex (covar "p" [bound 0]); fex (var "p" [bound 0])].
Proof.
  apply (soundness_classical String.eqb String.eqb_spec
    (Exists (covar "p" [bound 0]) (eigen 0)
      (Exists (var "p" [bound 0]) (eigen 0) (Ax "p" [eigen 0])))).
  vm_compute. reflexivity.
Qed.

(** [∀x. ∀y. r(x, y) ⊢ ∀y. ∀x. r(x, y)]: two eigenvariables, each
    introduced below the witnesses that name it. *)
Lemma swap : ll (symbol := string) Calculus.classical
  [fex (fex (covar "r" [bound 1; bound 0])); fall (fall (var "r" [bound 0; bound 1]))].
Proof.
  apply (soundness_classical String.eqb String.eqb_spec
    (Forall (fall (var "r" [bound 0; bound 1])) 0
      (Forall (var "r" [bound 0; eigen 0]) 1
        (Exists (fex (covar "r" [bound 1; bound 0])) (eigen 1)
          (Exists (covar "r" [eigen 1; bound 0]) (eigen 0)
            (Ax "r" [eigen 1; eigen 0])))))).
  vm_compute. reflexivity.
Qed.

(** [⊢ ∀x. ⊤, p(c)]: a [⊤] above a [∀] step absorbs the rest of the
    sequent, in which the eigenvariable does not occur. *)
Lemma absorbed : ll (symbol := string) Calculus.classical
  [fall top; var "p" [tapp "c" []]].
Proof.
  apply (soundness_classical String.eqb String.eqb_spec (Forall top 0 Top)).
  vm_compute. reflexivity.
Qed.

(** [⊢ ∃x. p(x)⊥, ∃y. p(y), 1] with Mix. *)
Lemma with_mix : ll (symbol := string)
  {| Calculus.mix := true; Calculus.weakening := false; Calculus.cut := false |}
  [fex (covar "p" [bound 0]); fex (var "p" [bound 0]); one].
Proof.
  apply (soundness_mix String.eqb String.eqb_spec
    (Mix
      (Exists (covar "p" [bound 0]) (eigen 0)
        (Exists (var "p" [bound 0]) (eigen 0) (Ax "p" [eigen 0])))
      One)).
  vm_compute. reflexivity.
Qed.

(** [⊢ ∃x. p(x)⊥, ∀y. p(y), q(a)⊥], affine: [∀x. p(x), q(a) ⊢ ∀y. p(y)]. *)
Lemma affine_weakening : ll (symbol := string)
  {| Calculus.mix := false; Calculus.weakening := true; Calculus.cut := false |}
  [fex (covar "p" [bound 0]); fall (var "p" [bound 0]); covar "q" [tapp "a" []]].
Proof.
  apply (soundness_affine String.eqb String.eqb_spec
    (Weaken (covar "q" [tapp "a" []])
      (Forall (var "p" [bound 0]) 0
        (Exists (covar "p" [bound 0]) (eigen 0) (Ax "p" [eigen 0]))))).
  vm_compute. reflexivity.
Qed.

(** [∀x. p(x) ⊢ ∃y. p(y)] by a cut on [∀z. p(z)]: the left premise
    proves [∀x. p(x) ⊢ ∀z. p(z)], the right one [∀z. p(z) ⊢ ∃y. p(y)]. *)
Lemma cut_of_forall : ll (symbol := string)
  {| Calculus.mix := false; Calculus.weakening := false; Calculus.cut := true |}
  [fex (covar "p" [bound 0]); fex (var "p" [bound 0])].
Proof.
  apply (soundness String.eqb String.eqb_spec
    {| Calculus.mix := false; Calculus.weakening := false; Calculus.cut := true |}
    (Cut (fall (var "p" [bound 0]))
      (Forall (var "p" [bound 0]) 0
        (Exists (covar "p" [bound 0]) (eigen 0) (Ax "p" [eigen 0])))
      (Exists (covar "p" [bound 0]) (eigen 1)
        (Exists (var "p" [bound 0]) (eigen 1) (Ax "p" [eigen 1]))))).
  vm_compute. reflexivity.
Qed.

(** A wrong witness: [∀x. p(x) ⊢ p(c)] is not proved at [d]. *)
Lemma wrong_witness :
  check String.eqb Calculus.classical
    (Exists (covar "p" [bound 0]) (tapp "d" []) (Ax "p" [tapp "c" []]))
    [fex (covar "p" [bound 0]); var "p" [tapp "c" []]] = false.
Proof. vm_compute. reflexivity. Qed.

(** An eigenvariable escaping its step: the unprovable [∀x. ∃y. r(x, y)
    ⊢ ∃y. ∀x. r(x, y)] with the witnesses chosen below the [∀] steps
    that introduce them, each [∀] step's eigenvariable standing in its
    conclusion. Every rule is applied as written, and the set the root
    carries holds both eigenvariables. *)
Lemma escaping :
  check String.eqb Calculus.classical
    (Exists (fall (covar "r" [bound 1; bound 0])) (eigen 1)
      (Exists (fall (var "r" [bound 0; bound 1])) (eigen 0)
        (Forall (covar "r" [eigen 1; bound 0]) 0
          (Forall (var "r" [bound 0; eigen 0]) 1
            (Ax "r" [eigen 1; eigen 0])))))
    [fex (fall (covar "r" [bound 1; bound 0])); fex (fall (var "r" [bound 0; bound 1]))]
  = false.
Proof. vm_compute. reflexivity. Qed.

(** An eigenvariable introduced again above its step: the unprovable
    [∃x. p(x) ⊢ ∀y. p(y)] with both bodies opened at [0]. *)
Lemma reintroduced :
  check String.eqb Calculus.classical
    (Forall (covar "p" [bound 0]) 0 (Forall (var "p" [bound 0]) 0 (Ax "p" [eigen 0])))
    [fall (covar "p" [bound 0]); fall (var "p" [bound 0])] = false.
Proof. vm_compute. reflexivity. Qed.

(** An eigenvariable the sequent names: [p(a) ⊢ ∀y. p(y)] with [a] the
    eigenvariable the [∀] step introduces. *)
Lemma in_the_sequent :
  check String.eqb Calculus.classical
    (Forall (var "p" [bound 0]) 0 (Ax "p" [eigen 0]))
    [fall (var "p" [bound 0]); covar "p" [eigen 0]] = false.
Proof. vm_compute. reflexivity. Qed.

(** A witness with a bound variable, which a quantifier above could
    capture. *)
Lemma open_witness :
  check String.eqb Calculus.classical
    (Exists (covar "p" [bound 0]) (bound 0)
      (Exists (var "p" [bound 0]) (bound 0) (Ax "p" [bound 0])))
    [fex (covar "p" [bound 0]); fex (var "p" [bound 0])] = false.
Proof. vm_compute. reflexivity. Qed.
