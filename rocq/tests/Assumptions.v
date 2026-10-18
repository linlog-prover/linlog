(* linlog © Fabian Lukas Grubmüller 2026 *)
(* Licensed under the EUPL *)

(** What the soundness theorems and the reduction of Mix rest on:
    nothing beyond the definitions. Rocq answers "Closed under the
    global context" for each, which the flake's check requires. *)

From Linlog Require Import Soundness IntuitionisticCheck Mix.

Print Assumptions soundness.
Print Assumptions soundness_classical.
Print Assumptions soundness_mix.
Print Assumptions soundness_affine.
Print Assumptions soundness_ill.
Print Assumptions mix_to_ll.
Print Assumptions mix_to_classical.
