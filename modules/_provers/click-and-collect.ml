(* linlog © Fabian Lukas Grubmüller 2026
   Licensed under the EUPL *)

(* Click & coLLecT's automatic prover on one sequent in its ASCII syntax,
   read from the file given, called as its web server's handler for
   `/auto_prove_sequent' calls it: the weakening proof where it applies, the
   focused search otherwise. `--seconds S' sets the search's limit of CPU
   time, which the build patches from its fixed 3 s into the variable
   CLICK_AND_COLLECT_SECONDS. Prints PROVED (exit 0), NOT PROVABLE (1),
   TIMEOUT (3) or ERROR (2). *)
let () =
  let args = Array.to_list Sys.argv in
  let rec options = function
    | "--seconds" :: s :: rest ->
        Unix.putenv "CLICK_AND_COLLECT_SECONDS" s;
        options rest
    | [ file ] -> file
    | _ ->
        prerr_endline "usage: click-and-collect FILE [--seconds S]";
        exit 2
  in
  let file =
    match args with
    | _ :: file :: rest -> options (rest @ [ file ])
    | _ -> options []
  in
  let ic = open_in file in
  let text = really_input_string ic (in_channel_length ic) in
  close_in ic;
  (* The lexer stops at the first newline. *)
  let text = String.map (fun c -> if c = '\n' || c = '\r' then ' ' else c) text ^ "\n" in
  match Parse_sequent.ll_parse text with
  | exception e ->
      print_endline ("ERROR: " ^ Printexc.to_string e);
      exit 2
  | raw -> (
      let sequent = Raw_sequent.to_sequent raw in
      let result =
        try (Some (Proof.auto_weak sequent), true)
        with Proof.AutoWeakNotApplicable ->
          Focused_proof.prove_sequent { Sequent_with_notations.sequent; notations = [] }
      in
      match result with
      | Some _, _ ->
          print_endline "PROVED";
          exit 0
      | None, false ->
          print_endline "NOT PROVABLE";
          exit 1
      | None, true ->
          print_endline "TIMEOUT";
          exit 3)
