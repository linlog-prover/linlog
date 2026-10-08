% A Horn program in the LLTP syntax: two tokens of b from one of a.
% Status   : Theorem
fof(double, axiom, a -o (b * b)).
fof(start, axiom, a).
fof(goal, conjecture, b * b).
