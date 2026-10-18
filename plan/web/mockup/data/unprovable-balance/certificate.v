(* unprovable (MLL, classical, focus engine): ~B occurs 1 more time than B in the one-sided sequent, so they cannot all meet in axioms *)
Lemma certificate : ~ ll (atom := string) classical [parr (covar "A") (covar "B"); var "A"].
Proof.
  apply (unbalanced_refutation String.eqb "B" _).
  vm_compute. reflexivity.
Qed.
