(* valid (classical logic by the affine translation into affine MALL, focus engine) *)
Lemma certificate : forall a b : Prop, (((a -> b) -> a) -> a).
Proof.
  exact (fun (a b : Prop) => NNPP (((a -> b) -> a) -> a) (fun k'1 : ~ (((a -> b) -> a) -> a) => (k'1 (fun h'2 : ((a -> b) -> a) => NNPP a (fun k'3 : ~ a => (let h'4 : a := h'2 (NNPP (a -> b) (fun k'5 : ~ (a -> b) => (k'5 (fun h'6 : a => NNPP b (fun k'7 : ~ b => (k'3 h'6)))))) in (k'3 h'4))))))).
Qed.

