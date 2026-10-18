From Stdlib Require Import List String.
From Linlog Require Import Soundness.
Import ListNotations.
Open Scope string_scope.

Lemma certificate : ll (atom := string) classical [awith (awith (awith (parr (parr (covar "A") (covar "B")) (tens (var "A") (var "B"))) one) (awith (parr bot one) top)) (awith (awith (aplus one zero) (aplus zero one)) (awith (awith (oc one) (wn one)) (awith (parr (wn (var "A")) one) (parr (wn (covar "A")) (tens (var "A") (var "A"))))))].
Proof.
  apply (soundness_classical String.eqb String.eqb_spec
    (With (awith (awith (parr (parr (covar "A") (covar "B")) (tens (var "A") (var "B"))) one) (awith (parr bot one) top)) (awith (awith (aplus one zero) (aplus zero one)) (awith (awith (oc one) (wn one)) (awith (parr (wn (var "A")) one) (parr (wn (covar "A")) (tens (var "A") (var "A")))))) (With (awith (parr (parr (covar "A") (covar "B")) (tens (var "A") (var "B"))) one) (awith (parr bot one) top) (With (parr (parr (covar "A") (covar "B")) (tens (var "A") (var "B"))) one (Par (parr (covar "A") (covar "B")) (tens (var "A") (var "B")) (Par (covar "A") (covar "B") (Tensor (var "A") (var "B") (Ax "A") (Ax "B")))) One) (With (parr bot one) top (Par bot one (Bot One)) Top)) (With (awith (aplus one zero) (aplus zero one)) (awith (awith (oc one) (wn one)) (awith (parr (wn (var "A")) one) (parr (wn (covar "A")) (tens (var "A") (var "A"))))) (With (aplus one zero) (aplus zero one) (Plus one zero Left One) (Plus zero one Right One)) (With (awith (oc one) (wn one)) (awith (parr (wn (var "A")) one) (parr (wn (covar "A")) (tens (var "A") (var "A")))) (With (oc one) (wn one) (Bang one One) (Quest one (Copy one One))) (With (parr (wn (var "A")) one) (parr (wn (covar "A")) (tens (var "A") (var "A"))) (Par (wn (var "A")) one (Quest (var "A") One)) (Par (wn (covar "A")) (tens (var "A") (var "A")) (Quest (covar "A") (Tensor (var "A") (var "A") (Copy (covar "A") (Ax "A")) (Copy (covar "A") (Ax "A")))))))))).
  vm_compute. reflexivity.
Qed.
