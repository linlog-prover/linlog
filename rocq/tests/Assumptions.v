(* linlog © Fabian Lukas Grubmüller 2026 *)
(* Licensed under the EUPL *)

(** What the soundness theorems rest on: nothing beyond the definitions.
    Rocq answers "Closed under the global context" for each, which the
    flake's check requires. *)

From Linlog Require Import Soundness.

Print Assumptions soundness.
Print Assumptions soundness_classical.
Print Assumptions soundness_mix.
Print Assumptions soundness_affine.
