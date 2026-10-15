% linlog © Fabian Lukas Grubmüller 2026
% Licensed under the EUPL

% Driver for LinearOne: reads one term `Antecedent => Goal.` (a list of
% formulas and a formula, in LinearOne's own syntax) from the file given
% after `--`, runs first_proof/3 and prints the verdict. Writes no file and
% starts no LaTeX.
:- initialization(lo_main, main).

:- op(700, xfx, =>).

lo_main :-
    current_prolog_flag(argv, Argv), last(Argv, File), File \== '--',
    source_file(user:lo_main, Me),
    file_directory_name(Me, Dir),
    directory_file_path(Dir, 'mill1.pl', Mill),
    consult(Mill),
    open(File, read, S),
    read_term(S, Ants => Goal, []),
    close(S),
    % initialisation/0 would open graph.tex and latex_proofs.tex, so set
    % the counters that prove1/3 and portray_graph/1 read by hand
    % (first_proof/3 itself sends the graph output to a null stream)
    retractall(user:'$AXIOMS'(_)), assert(user:'$AXIOMS'(0)),
    retractall(user:'$PROOFS'(_, _)), assert(user:'$PROOFS'(0, [])),
    retractall(portray_graph_tikz:'$GRAPH_NO'(_)), assert(portray_graph_tikz:'$GRAPH_NO'(0)),
    statistics(cputime, T0),
    (   catch(user:first_proof(Ants, Goal, Sem), E, (print_message(error, E), halt(2)))
    ->  statistics(cputime, T1), T is T1 - T0,
        format("PROVABLE (~3f s)~nsemantics: ", [T]), print(Sem), nl, halt(0)
    ;   statistics(cputime, T1), T is T1 - T0,
        format("NOT PROVABLE (~3f s)~n", [T]), halt(1)
    ).
lo_main :- format(user_error, "usage: linearone FILE~n", []), halt(2).
