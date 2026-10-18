% linlog © Fabian Lukas Grubmüller 2026
% Licensed under the EUPL

% linTAP on one formula, a Prolog term ended by a full stop in the file
% named after `--': lintap.pl's own search (the first clause of its pp/2)
% with the expansion limit of the exponentials raised from 0 one at a time,
% as its pp/2 does. Without `?' the search is exhaustive at the limit 0 and
% a failure is a refutation; with `?' it goes on until the caller stops it,
% as lintap.pl's prove/1 does, whose second clause of pp/2 would raise the
% limit for ever on a formula it cannot prove without saying so.
:- initialization(main, main).

main([File]) :-
    source_file(user:main(_), Me),
    file_directory_name(Me, Dir),
    directory_file_path(Dir, 'lintap.pl', Lintap),
    consult(Lintap),
    open(File, read, S),
    read_term(S, F, []),
    close(S),
    (   has_exp(F) -> Max = inf ; Max = 0 ),
    (   between(0, Max, I), try(F, I)
    ->  format("PROVABLE (expansion limit ~w)~n", [I]), halt(0)
    ;   format("NOT PROVABLE~n"), halt(1)
    ).
main(_) :- format(user_error, "usage: lintap FILE~n", []), halt(2).

has_exp(F) :- sub_term(S, F), nonvar(S), ( S = ?(_) ; S = !(_) ), !.

% The body of the first clause of pp/2 in lintap.pl 1.08e (© 1998 Jens
% Otten, under the GNU GPL), which this driver loads rather than
% redistributes: the call of its search and the conditions it puts on
% the result.
try(F, I) :-
    prove([(F,0),[0],l], [], [], 0, I, PU, At, Bt, [], C1),
    length(Bt, Nbt), length(C1, Nc1), Nc1 =:= (2*Nbt)+2,
    length(At, Nat), Nc1 =:= Nat+Nbt+1,
    t_string_unify(PU).
