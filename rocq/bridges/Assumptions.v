(* linlog © Fabian Lukas Grubmüller 2026 *)
(* Licensed under the EUPL *)

(** What the bridges rest on: nothing beyond the definitions of the two
    sides. Rocq answers "Closed under the global context" for each,
    which the flake's check requires. *)

From LinlogBridges Require Import NanoYallaBridge NanoIllBridge.

Print Assumptions ll_iff_nanoll.
Print Assumptions nanoll_iff_ll.
Print Assumptions ill_iff_nanoill.
Print Assumptions nanoill_iff_ill.
