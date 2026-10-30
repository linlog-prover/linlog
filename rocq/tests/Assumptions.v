(* linlog © Fabian Lukas Grubmüller 2026 *)
(* Licensed under the EUPL *)

(** What the soundness theorems, the first-order ones among them, the
    reduction of Mix and the refutations rest on: nothing beyond the
    definitions. Rocq answers "Closed under the global context" for
    each, which the flake's check requires. The first-order theorems
    share their names with the propositional ones and are named by their
    files. *)

From Linlog Require Import Soundness IntuitionisticCheck Mix Refutations.
From Linlog Require FirstOrderSoundness FirstOrderIntuitionisticCheck.

Print Assumptions soundness.
Print Assumptions soundness_classical.
Print Assumptions soundness_mix.
Print Assumptions soundness_affine.
Print Assumptions soundness_ill.
Print Assumptions mix_to_ll.
Print Assumptions mix_to_classical.
Print Assumptions ll_tautology.
Print Assumptions classical_refutation.
Print Assumptions balance.
Print Assumptions unbalanced_refutation.
Print Assumptions count_equation.
Print Assumptions equation_refutation.
Print Assumptions ill_ll.
Print Assumptions ill_tautology.
Print Assumptions iclassical_refutation.
Print Assumptions iunbalanced_refutation.
Print Assumptions iequation_refutation.
Print Assumptions FirstOrderSoundness.soundness.
Print Assumptions FirstOrderSoundness.soundness_classical.
Print Assumptions FirstOrderSoundness.soundness_mix.
Print Assumptions FirstOrderSoundness.soundness_affine.
Print Assumptions FirstOrderIntuitionisticCheck.soundness_ill.
