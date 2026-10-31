# The commands

The command line program is `linlog` (`cargo install --locked linlog-cli`, `nix run
github:linlog-prover/linlog -- …`, or `cargo run -p linlog-cli -- …` in a
checkout). `linlog --help` and `linlog <command> --help` document every
option.

`linlog prove` decides a sequent, written as below (`-o` is ⊸, `*` ⊗,
`par` ⅋, `~` negation; the whole syntax is in [The syntax](syntax.md),
and `linlog --help` prints it). The first line gives the verdict,
the fragment the sequent was detected to live in and the engine that
searched; a proof follows as a derivation tree of the one-sided sequent
calculus:

```console
$ linlog prove "A, A -o B |- B"
provable (MLL, classical, focus engine)
─────── ax   ─────── ax
⊢ ~A, A      ⊢ ~B, B
──────────────────── ⊗
  ⊢ ~A, A ⊗ ~B, B

$ linlog prove "A & B |- A + B"
provable (ALL, classical, additive engine)
    ─────── ax
    ⊢ ~A, A
  ─────────── ⊕₁
  ⊢ ~A, A ⊕ B
──────────────── ⊕₁
⊢ ~A ⊕ ~B, A ⊕ B

$ linlog prove "|- A par B, ~A, ~B"
unprovable (MLL, classical, focus engine): the count equation fails: a provable one-sided sequent of MLL has exactly #⊗ − #⅋ − #1 + #⊥ + 2 formulas, here 0 − 1 − 0 + 0 + 2 = 1, and this one has 3
$ linlog prove "A |- B"
unprovable (MLL, classical, focus engine): ~A occurs 1 more time than A in the one-sided sequent, so they cannot all meet in axioms
$ linlog prove "!A, A |- 1"
unprovable (MELL, classical, horn engine): ~A occurs 1 more time than A outside the exponentials of the one-sided sequent, and every literal of the atom below one, which copies add, is ~A, so they cannot all meet in axioms
```

An unprovable verdict says why where the counts of the sequent tell: an
atom whose literals cannot all meet their duals in axioms (whichever
additive alternatives a proof takes, and however often a formula under
`!` or `?` is copied where all of the atom's literals there have one
sign), or the count equation of the multiplicatives; from the inverse
method below, the sequents its saturation ended in, closed under the
rules, which are checked apart from the search as a proof is; otherwise
it says that the search was exhaustive. The
JSON output carries the same as `refutation`.

Four engines serve classical logic: for two additive-only formulas the
*additive engine* recurses on pairs of subformulas; for a Horn program
with clauses under `!`, which is a Petri net, the *Horn engine* searches
its markings (below); for everything else the *focus engine* runs a
focused sequent search over bitsets; and for MLL without units the *net
engine* searches for an axiom linking that makes the sequent's formula
trees a proof net, by default only where the focus engine stops at its
recursion limit or memory bound on a sequent whose atoms occur once with
each sign (a chain of thousands of links, some sequents of thousands of
distinct atoms), since everywhere else the focus engine decides first. `--mix`, `--affine` and `--intuitionistic`
choose the logic, `--fragment` and `--engine focus|net|two-sided|additive|horn|essential|inverse`
override what detection picks (`inverse`, the inverse method, runs alone only
when asked, and takes a turn beside the engine on some goals; below), `--timeout`, `--copies N` and
`--forward-copies N` bound the search (below), `--quiet` prints the verdict line only
and `--stats` what the search cost, in the counters of the engine that
ran (the focused engine's: a stable sequent is one where every rule left
is a choice, the memo keeps the stable sequents already decided, and a
split is one way to divide a context between the premises of a ⊗; the
net engine's: a literal chosen to link, a link tried, a correctness test
run; units of work are each engine's steps, the same on every machine
on one thread):

```console
$ linlog prove --mix --stats --deterministic --engine net "|- A par B, ~A, ~B"
provable (MLL, classical with Mix, net engine)
─────── ax   ─────── ax
⊢ A, ~A      ⊢ B, ~B
──────────────────── mix
   ⊢ A, B, ~A, ~B
   ─────────────── ⅋
   ⊢ A ⅋ B, ~A, ~B
literals chosen: 2
links tried: 2
exact tests run: 2
units of work: 2
time: 63.45µs
$ linlog prove --engine focus --stats --quiet --deterministic "|- A * B, C * (~A par ~B), ~C"
provable (MLL, classical, focus engine)
stable sequents visited: 2
from the memo: 0
memo entries at most: 2
splits examined: 3
units of work: 35
time: 56.66µs
```

With exponentials (MELL and full LL) the focus engine searches dyadic
sequents under a bound on how often a `?` formula is copied on one branch,
which it deepens from zero: by default it goes on to the next bound until
it decides or the time limit passes, two seconds unless `--timeout` says
otherwise (`--timeout none` lifts it), and `--copies N` caps the bound at
N. The derivation shows the standard rules: dereliction, contraction,
weakening and promotion.

```console
$ linlog prove "!A |- !(A * A)"
provable (MELL, classical, focus engine)
─────── ax    ─────── ax
⊢ ~A, A       ⊢ ~A, A
──────── ?d   ──────── ?d
⊢ ?~A, A      ⊢ ?~A, A
────────────────────── ⊗
  ⊢ ?~A, ?~A, A ⊗ A
  ───────────────── ?c
    ⊢ ?~A, A ⊗ A
   ─────────────── !
   ⊢ ?~A, !(A ⊗ A)
```

A *Horn program* is atoms, implications between tensors of atoms used
once, such implications under `!`, used any number of times, and one goal
that is a tensor of atoms: `!(a * b -o c * d)` is a transition of a Petri
net, the atoms on the left of `⊢` its marking, and the goal the marking to
reach. With a clause under `!` it goes to the Horn engine, which searches
the markings instead of sequents, each reached once and the one nearest
the goal first, and reads the proof off the firing sequence it finds:
each firing a copy of its clause, whose body takes its atoms by axioms.
It needs no copy bound, and when every reachable marking has been seen
without the goal, or the counts of an atom's literals rule the goal out,
it answers "unprovable". So it does when the net's state equation has no
solution, which a simplex beside the search finds out: then weights for
the atoms, checked exactly, show that no firing raises the weighted
count of the tokens while the goal needs it raised, however many
markings the net has. A net whose markings grow without end and whose
equation has a solution is searched until the time or the memory limit.
Two more refutations reach nets whose markings grow without end: a
clause with an atom no reachable marking holds can never fire and is
left out, and once the search has run a while, the same search runs
backward from the goal beside it, where such a net often has few
markings. In affine mode, where the tokens and clauses left over may be
weakened, the question is whether some reachable marking covers the
goal, and the engine decides it backward from the goal: the least
markings from which a firing covers it are kept until one lies below the
start, or none is new, which always comes. `--stats` counts the
markings:

```console
$ linlog prove "!A, !(A -o B), !(B -o C) |- C"
provable (MELL, classical, horn engine)
             ─────── ax   ─────── ax
             ⊢ ~B, B      ⊢ ~C, C
             ──────────────────── ⊗
               ⊢ ~B, B ⊗ ~C, C
─────── ax    ────────────────── ?d
⊢ ~A, A       ⊢ ~B, ?(B ⊗ ~C), C
──────────────────────────────── ⊗
   ⊢ ~A, A ⊗ ~B, ?(B ⊗ ~C), C
  ───────────────────────────── ?d
  ⊢ ~A, ?(A ⊗ ~B), ?(B ⊗ ~C), C
  ────────────────────────────── ?d
  ⊢ ?~A, ?(A ⊗ ~B), ?(B ⊗ ~C), C
$ linlog prove -q --deterministic --stats "!(a * a -o b), !(b * b -o c), !(c * c -o d), a, a, a, a, a, a, a, a |- d"
provable (MELL, classical, horn engine)
markings reached: 7
markings met again: 0
markings kept: 7
units of work: 6
time: 45.06µs
$ linlog prove -q --deterministic --stats "!(a * a -o b), !(b * b -o c), !(c * c -o d), a, a, a, a, a, a, a, a |- d * a"
unprovable (MELL, classical, horn engine): the search was exhaustive
markings reached: 12
markings met again: 2
markings kept: 10
units of work: 13
time: 31.81µs
$ linlog prove -q "!(a -o a * a), !(b * b -o c), a, b |- c"
unprovable (MELL, classical, horn engine): the state equation of the Petri net has no solution, so no firing of its clauses yields the goal's atoms: weighting each a by −2, b by 1 and c by 2, no clause that can fire raises the weighted count of the atoms, and the goal asks it raised
$ linlog prove -q --affine --stats "!(a * a -o b * b * b), a, a, a |- b * b * b"
provable (MELL, classical affine, horn engine)
markings computed backward: 1
markings covered already: 0
markings kept: 1
units of work: 2
time: 268.07µs
$ linlog prove -q --affine "!(a * a -o b * b * b), a, a, a |- b * b * b * b"
unprovable (MELL, classical affine, horn engine): the search was exhaustive
```

The last goal's state equation has a solution, one and a third firings
of the clause, so only the search refutes it; the `a` left over in the
proof above is weakened.

Provability in MELL has no known decision procedure, and full linear logic
is undecidable, so the verdict is three-valued: "unprovable" is reported
only when a bound was searched exhaustively without ever hitting it, or
when the sequent is false read classically (below), and "unknown" when the
time limit, the bound of `--copies` or another limit ended the search
first. The line says which, after how long, at which copy bound, and which
flag changes it; `--stats` adds the copy bound reached:

<!-- readme-check: machine -->
```console
$ linlog prove -q "!(A & B) |- A * B"
provable (LL, classical, focus engine)
$ linlog prove -q --copies 1 "!(A & B) |- A * B"
unknown (LL, classical, focus engine): the copy bound of 1 was reached after 54.08µs; raise it with --copies N, or lift it with --copies none to deepen it while the time limit lasts
$ linlog prove -q "A |- !A"
unprovable (MELL, classical, focus engine): the search was exhaustive
$ linlog prove -q --classical-work 0 "!(A -o A * A), !(A -o B), !(B * B -o C), A |- C * ?D"
unknown (MELL, classical, focus engine): the time limit of 2s was reached at a copy bound of 97; --timeout DURATION gives the search longer
```

After a search that did not prove the sequent, the command reads it
classically, with `!` and `?` erased, `*` and `&` as "and", `|` and `+`
as "or", `-o` as implication, `1` and `top` true and `bot` and `0` false,
and looks for truth values of its atoms that make it false. No provable
sequent has them, in any mode, so an unknown search becomes unprovable
with the values as its reason, and an exhausted one gets them as a
certificate (the library's `Disproof::check` and the
[Rocq export](rocq.md) verify it). The search
for the values runs after the proof search, past the time limit, within
`--classical-work N` formulas evaluated (2²⁶ by default; `0` switches
it off), and never changes a verdict the proof search
gave. The last sequent above is unprovable so, and `A |- !A`, true read
classically, keeps the search's word:

```console
$ linlog prove -q "!(A -o A * A), !(A -o B), !(B * B -o C), A |- C * ?D"
unprovable (MELL, classical, focus engine): the classical reading is false where A, B and C are true and every other atom false (! and ? erased, ⊗ and & as and, ⅋ and ⊕ as or, 1 and ⊤ true, ⊥ and 0 false), and every provable sequent reads as a tautology
```

The deepening is why a call without flags answers within its time limit
whatever it is given, and why an answer near the limit depends on the
machine: a script that must get the same answer everywhere names its
bound (`--copies`) or lifts the limit (`--timeout none`), and
`--deterministic` (below) makes the statistics a function of the input:

```console
$ linlog prove -q --deterministic --stats --engine focus --bias rarer --copies 3 "!(A -o B), !(B -o C), !(C -o D), !(D -o E), A |- E"
unknown (MELL, classical, focus engine): the copy bound of 3 was reached after 36.65µs; raise it with --copies N, or lift it with --copies none to deepen it while the time limit lasts
stable sequents visited: 10
from the memo: 0
memo entries at most: 4
splits examined: 24
copy bound reached: 3
units of work: 250
time: 36.65µs
$ linlog prove -q --deterministic --stats --engine focus --bias rarer --timeout none "!(A -o B), !(B -o C), !(C -o D), !(D -o E), A |- E"
provable (MELL, classical, focus engine)
stable sequents visited: 15
from the memo: 0
memo entries at most: 5
splits examined: 28
copy bound reached: 4
units of work: 380
time: 60.01µs
```

While a search runs longer than half a second, a line on standard error
says so when that is a terminal, and goes again when the answer comes.

`--affine` allows weakening: a hypothesis may go unused, which the
derivation shows as `wk` below the leaf that leaves it over. With
exponentials the affine search deepens its copy bound like the linear
one:

<!-- readme-check: machine -->
```console
$ linlog prove -a "A, B |- A"
provable (MLL, classical affine, focus engine)
  ─────── ax
  ⊢ ~A, A
─────────── wk
⊢ ~A, ~B, A
$ linlog prove -q -a --classical-work 0 "!(A -o A * A), A |- ?B"
unknown (MELL, classical affine, focus engine): the time limit of 2s was reached at a copy bound of 18; --timeout DURATION gives the search longer
```

The focused engines treat one literal of every atom as positive, which
decides where a proof keeps its focus and never what is provable; `--bias`
names the rule. `factors` makes the literal positive that is more often a
direct factor of a `⊗` (such a `⊗` needs no search for its split), `rarer`
the one with fewer occurrences. Without exponentials the default, `auto`,
is `factors`, and under `--affine` it is `rarer`. With exponentials
neither wins: on Horn clauses under `!` (which the focused engines meet
inside larger sequents, or forced by `--engine`),
`factors` chains forward from the facts and is often faster by orders of
magnitude, but a forward chain takes one copy per step on a single
branch, where `rarer` chains backward from the goal within a few. So on a
sequent with exponentials `auto` runs both searches and answers with the
first that decides: alternating in slices of work on one core, so that
the run stays a function of the input, and side by side on several. By
default both deepen while the time limit lasts. Under `--copies N` the
backward search keeps to the bound; so does the forward one, except on a
Horn program (clauses such as `!(a * b -o c * d)`, a marking and a goal
of atoms, which is what a Petri net is), where it runs within
`--forward-copies` (30 by default), a bound in steps of the chain:

```console
$ linlog prove -q --deterministic --stats --engine focus "!(a * a -o b), !(b * b -o c), !(c * c -o d), a, a, a, a, a, a, a, a |- d"
provable (MELL, classical, focus engine)
stable sequents visited: 47
from the memo: 5
memo entries at most: 9
splits examined: 151
copy bound reached: 7
forward search's copy bound reached: 7
units of work: 1297
time: 136.45µs
$ linlog prove -q --deterministic --stats --engine focus --bias rarer "!(a * a -o b), !(b * b -o c), !(c * c -o d), a, a, a, a, a, a, a, a |- d"
provable (MELL, classical, focus engine)
stable sequents visited: 230
from the memo: 178
memo entries at most: 34
splits examined: 1036
copy bound reached: 3
units of work: 4256
time: 181.10µs
$ linlog prove -q --deterministic --stats --engine focus --bias factors --copies 3 "!(a * a -o b), !(b * b -o c), !(c * c -o d), a, a, a, a, a, a, a, a |- d"
unknown (MELL, classical, focus engine): the copy bound of 3 was reached after 55.51µs; raise it with --copies N, or lift it with --copies none to deepen it while the time limit lasts
stable sequents visited: 11
from the memo: 0
memo entries at most: 5
splits examined: 31
copy bound reached: 3
units of work: 322
time: 55.51µs
$ linlog prove -q --engine focus --copies 1 "!A, !(A -o B), !(B -o C) |- C"
provable (MELL, classical, focus engine)
$ linlog prove -q --engine focus --copies 1 --forward-copies 1 "!A, !(A -o B), !(B -o C) |- C"
unknown (MELL, classical, focus engine): the copy bound of 1 was reached after 149.56µs; raise it with --copies N, or lift it with --copies none to deepen it while the time limit lasts
```

`--timeout` counts from the start of the command, so it covers reading
and parsing the sequent as well as the search, and the command answers
within a fraction of a second of it, on one thread and on several; the
classical refutation after the search runs past it within its own bound
of work, which the example switches off:

<!-- readme-check: machine -->
```console
$ linlog prove -q --engine focus --timeout 1s --classical-work 0 "!(A -o A * A), !(A -o B), !(B * B -o C), A |- C * ?D"
unknown (MELL, classical, focus engine): the time limit of 1s was reached at a copy bound of 82; --timeout DURATION gives the search longer
```

<!-- readme-check: skip, the file is the LLTP library's -->
```console
$ linlog prove -q -i --timeout 1s --file SYJ212+1.020.p
unknown: the time limit of 1s was reached while the sequent was read
```

The second sequent is the largest problem of the LLTP library, a file of
86 MB that takes twelve seconds to parse; both commands
end with exit status 3 a second after they started.

By default the search runs on one thread first, and if that has not
decided within a tenth of a second (`--pool-after`), a pool of the
machine's other cores searches beside it, the first to decide answering
(the additive and the Horn engine run on one thread and get none): a
small sequent is decided at once and always the same way, a hard one gets
the machine, and what one thread decides within the limit stays decided.
`--jobs N` (`-j`) runs N threads from the start (with `--pool-after`, after
one thread), and `--deterministic` runs the sequential engines throughout,
whose proof and statistics are a function of the input, where a parallel
run may find a different proof of the same sequent, never a different
verdict. The
focus engine splits the choices nearest the root among the threads and
shares its memo (on a first-order sequent it splits only the choices
that no rule around them may ask for another witness); the net engine splits its search into cubes at its
first choices, and where every link is forced it makes each link once,
as one thread does; the additive engine is sequential in every case. A
search never uses more threads than the machine runs at once, and a
larger `--jobs` is taken as that many, with a note:

<!-- readme-check: machine -->
```console
$ linlog prove -q -j 4 --copies 3 --classical-work 0 "!(A -o A * A), !(B * B -o C), A, B |- A * A * A * ?D"
unknown (MELL, classical, focus engine): the copy bound of 3 was reached after 1.79ms; raise it with --copies N, or lift it with --copies none to deepen it while the time limit lasts
$ linlog prove -q --deterministic --copies 3 --classical-work 0 "!(A -o A * A), !(B * B -o C), A, B |- A * A * A * ?D"
unknown (MELL, classical, focus engine): the copy bound of 3 was reached after 2.55ms; raise it with --copies N, or lift it with --copies none to deepen it while the time limit lasts
$ linlog prove -q -j 10000 "A |- A"
note: --jobs 10000 is more than this machine runs at once; a search uses 16
provable (MLL, classical, focus engine)
```

`--intuitionistic` (`-i`) reads the sequent as intuitionistic linear logic,
as it is written: the hypotheses left of `⊢`, exactly one formula right of
it, in the syntax of ILL (`⊗ ⊸ & ⊕ !`, `1 ⊤ 0`). A symbol of the
classical syntax (`~`, `^`, `par`, `|`, `?`, `bot`) is refused with what
to write instead; `--ill-syntax one-sided` (the settings key
`input.intuitionistic`) reads the classical syntax as the one-sided
sequent it stands for, `~(A * ~B), A |- B` as `A ⊸ B, A ⊢ B`, where the
one-sided formulas decide the sides. The verdict line names the
intuitionistic fragment, and every output writes the sequent as it was
written: the derivation is two-sided with the rules of ILL, and the
refutations, the nets and the errors speak of its formulas. A
sequent of two additive-only
formulas to the *additive engine* (a recursion on pairs of subformulas,
linear in the product of their sizes, in every mode), a Horn program
with a clause under `!` to the Horn engine, as classically, and
everything else to the *two-sided engine*, the focused search keeping one goal on every
branch, with the same copy bound and affine mode as the classical one;
IMLL without units, each atom once with each sign, goes to the net engine
where the two-sided engine stops at its recursion limit or memory bound
(the classical net of the one-sided sequent is always an intuitionistic
proof). `--engine essential` searches IMLL without units for an
*essential net* instead: the reading directs every edge from the
hypotheses toward the goal, and a linking is a proof net when the
directed graph has no cycle and every implication `A ⊸ B` of the goal's
side dominates its antecedent, every path from `A` to the goal passing
through it; the engine keeps which occurrence reaches which, a matrix
of bits that `--memory-limit` bounds. It is slower than the two-sided
engine on all but the smallest sequents it was measured on, so it runs
only when asked:

```console
$ linlog prove -i --engine essential --net "A -o B, B -o C |- A -o C"
provable (IMLL, intuitionistic, essential engine)
A ⊸ B, B ⊸ C ⊢ A ⊸ C
A[1] — A[7]
B[2] — B[4]
C[5] — C[8]
proof net
$ linlog prove -i "A -o B -o C, A * B |- C"
provable (IMLL, intuitionistic, two-sided engine)
           ───── ax   ───── ax
           B ⊢ B      C ⊢ C
───── ax   ──────────────── ⊸L
A ⊢ A        B ⊸ C, B ⊢ C
───────────────────────── ⊸L
  A ⊸ (B ⊸ C), A, B ⊢ C
  ────────────────────── ⊗L
  A ⊸ (B ⊸ C), A ⊗ B ⊢ C
$ linlog prove -i "!A, !(A -o B) |- !B & A"
provable (ILL, intuitionistic, two-sided engine)
───── ax
A ⊢ A
────── !L   ───── ax
!A ⊢ A      B ⊢ B
───────────────── ⊸L         ───── ax
  !A, A ⊸ B ⊢ B              A ⊢ A
 ──────────────── !L         ────── !L
 !A, !(A ⊸ B) ⊢ B            !A ⊢ A
 ───────────────── !R   ──────────────── !w
 !A, !(A ⊸ B) ⊢ !B      !A, !(A ⊸ B) ⊢ A
 ─────────────────────────────────────── &R
          !A, !(A ⊸ B) ⊢ !B & A
$ linlog prove -i -q --stats "(A & B) + (A & C) |- A & (B + C)"
provable (IALL, intuitionistic, additive engine)
pairs of subformulas visited: 20
from the memo: 0
memo entries: 20
units of work: 20
time: 89.90µs
$ linlog prove -i "|- ~A par B"
error: cannot parse the sequent
  |- ~A par B
     ^ unexpected "~"
  intuitionistic linear logic has no negation: write a negated formula on the other side of |- (A |- B, not |- ~A, B), and ~A par B as A -o B; --ill-syntax one-sided reads the classical syntax
$ linlog prove -i -q --ill-syntax one-sided "|- ~A par B"
unprovable (IMLL, intuitionistic, two-sided engine): A occurs negatively 1 more time than positively (positive right of ⊢, negative left of it, an implication's antecedent on the other side), so they cannot all meet in axioms
$ linlog prove -i --ill-syntax one-sided "|- A par B"
error: not an intuitionistic sequent: the subformula A ⅋ B is neither an intuitionistic formula nor the negation of one (⅋ only as A ⊸ B, that is ~A ⅋ B, and ? only under a negation)
$ linlog prove -i "A |- B, C"
error: not an intuitionistic sequent: an intuitionistic sequent has exactly one formula right of ⊢, not 2: write the hypotheses left of ⊢
```

Classical linear logic proves more than intuitionistic linear logic once
`0` is around, and the two-sided search knows the difference:

```console
$ linlog prove -q "((A * top) & (B * top)) -o 0 |- (A -o C) + (B -o C)"
provable (MALL, classical, focus engine)
$ linlog prove -i -q "((A * top) & (B * top)) -o 0 |- (A -o C) + (B -o C)"
unprovable (IMALL, intuitionistic, two-sided engine): the search was exhaustive
```

`--engine inverse` searches forward instead, by the *focused inverse
method*: it derives stable sequents from the axioms upward, each
focusing phase of the sequent's formulas one rule, keeps a derived
sequent unless one kept already stands for every sequent it stands for,
and proves the goal once every stable sequent the goal decomposes into
is derived. Without exponentials the sequents it can derive are finitely
many and it decides; with them it needs no copy bound, proving what is
provable given time and refuting where nothing new comes, which suits a
small goal under many hypotheses that `!` makes reusable. Where it
refutes, the refutation is the set of sequents it saturated with, which
a check of its own (the rules read off the sequent again and applied to
every choice of the set's sequents) verifies before the verdict. Its
counters are the sequents taken up, subsumed and kept, and it runs in
every commutative mode. Without `--engine`, on an intuitionistic goal
with an exponential and on a goal under Mix (not affine), the engine
that detection picks searches alone for some hundredths of a second,
then the inverse method takes a turn on the same thread each time the
engine's work has doubled, starting afresh, of at most a thirty-second of
that work, and the engine goes on where it stopped: the turns cost at
most a sixteenth of the engine's work, a goal decided within the head
start sees none, and one the inverse method decides in a turn is
answered by it, which the verdict line names (`inverse engine`).
On the intuitionistic problems of the LLTP library that is where it
decides in milliseconds what the backward search leaves undecided after
a minute. Alone it runs only when asked:

```console
$ linlog prove -i --engine inverse --stats "!(a -o b), !(b -o c), a |- c"
provable (IMELL, intuitionistic, inverse engine)
           ───── ax   ───── ax
           b ⊢ b      c ⊢ c
           ──────────────── ⊸L
             b, b ⊸ c ⊢ c
───── ax    ─────────────── !L
a ⊢ a       b, !(b ⊸ c) ⊢ c
─────────────────────────── ⊸L
  a ⊸ b, !(b ⊸ c), a ⊢ c
 ───────────────────────── !L
 !(a ⊸ b), !(b ⊸ c), a ⊢ c
sequents taken up: 2
sequents subsumed: 0
sequents kept: 3
units of work: 50
time: 102.44µs
$ linlog prove -i --engine inverse -q "!(a -o b), a |- b * b"
unprovable (IMELL, intuitionistic, inverse engine): the search saturated: 2 sequents closed under the rules, and the goal needs one they do not give
```

The exit status tells scripts the verdict: 0 provable, 1 unprovable, 3
unknown (the time limit, a bound, a limit or Ctrl-C stopped the search), 2 an
error, such as a sequent outside the asserted fragment or an engine forced
on a sequent it cannot search. A reader that closes the pipe early, as
`linlog prove … | head -1` does, changes none of it, and a second Ctrl-C
ends the command at once with 130, as the shell's own interrupt does.

Every proof the search finds passes the proof checker before anything is
reported, whatever the output (`--no-check` reports it unchecked). A proof
is small where its derivation can be huge: the derivation writes the whole
sequent at every inference and repeats every subproof the proof shares.
So the command shows a derivation only where that is sensible, and
otherwise keeps the verdict, with its exit status, and says in one line
what it left out and how to get it. On a terminal the text format prints
the proof tree if no line is wider than the terminal and the tree is at
most three screens long (`--tree always` prints it regardless, `--tree
never` leaves it out; into a file or a pipe it is always written). On a
terminal 30 columns wide:

<!-- readme-check: terminal -->
```console
$ linlog prove "A, A -o B, B -o C, C -o D |- D"
provable (MLL, classical, focus engine)
the proof tree is not shown: 7 inferences, at least 31 columns by 8 lines, for a terminal of 30 columns and at most 30 lines (3 screens); print it with --tree always, write it with --output FILE, or get the proof with --format json
```

And in every format a derivation whose estimated size passes
`--derivation-limit` (64 MiB by default; `none` lifts it) is not built at
all. The line then goes to standard error, unless the output is a
terminal:

```console
$ linlog prove --derivation-limit 100 "A, A -o B |- B" > verdict.txt
the derivation is not written: its 3 inferences with 29 characters of sequents are estimated at 616 B, over the limit of 100 B; --format json writes the proof itself, --derivation-limit SIZE raises the limit and --derivation-limit none lifts it
$ cat verdict.txt
provable (MLL, classical, focus engine)
```

A run of one structural rule, such as the weakenings of every `?`
formula a proof does not use, can be drawn as one inference whose label
has a star: `--compact always` does so in every format but Rocq, and the
default, `--compact auto`, does so where the whole derivation would pass
`--derivation-limit` or not fit the terminal; `--compact never` draws
every rule.

```console
$ linlog prove -i --compact always '!A, !B, !C |- 1'
provable (IMELL, intuitionistic, horn engine)
     ─── 1R
     ⊢ 1
────────────── !w*
!A, !B, !C ⊢ 1
```

A classical derivation is drawn one-sided by default, as the calculus
is. `--classical-sides two` draws it two-sided as the sequent was written,
in every format but Rocq: every sequent has its formulas on the sides of
`⊢` they were written on, the rules are named by side (`⊗L`, `⅋R`, `⊸L`,
…), and where a negation was written a step `~L` or `~R` moves the
formula across `⊢`, which the one-sided proof does without. The text
keeps how each formula was written; a sequent given as JSON, LLTP or
`.spec` is written one-sided, its antecedents on the left.

```console
$ linlog prove --classical-sides two "A par B |- ~(~A * ~B)"
provable (MLL, classical, focus engine)
───── ax   ───── ax
A ⊢ A      B ⊢ B
──────────────── ⅋L
  A ⅋ B ⊢ A, B
  ───────────── ~L
  A ⅋ B, ~A ⊢ B
 ─────────────── ~L
 A ⅋ B, ~A, ~B ⊢
 ──────────────── ⊗L
 A ⅋ B, ~A ⊗ ~B ⊢
────────────────── ~R
A ⅋ B ⊢ ~(~A ⊗ ~B)
$ linlog prove --classical-sides two --format latex "A -o B, A |- B"
% provable (MLL, classical, focus engine)
\begin{prooftree}
\infer0[$\mathrm{ax}$]{A &\vdash A}
\infer0[$\mathrm{ax}$]{B &\vdash B}
\infer2[$\multimap\mathrm{L}$]{A \multimap B, A &\vdash B}
\end{prooftree}
```

A first-order derivation is drawn so too, its quantifier steps named by
side (`∀L`, `∀R`, `∃L`, `∃R`) and every instance written out:

```console
$ linlog prove --classical-sides two "|- ~(forall x. p(x)), p(a)"
provable (MLL1, classical, focus engine)
    ─────────── ax
    p(a) ⊢ p(a)
  ─────────────── ∀L
  ∀x. p(x) ⊢ p(a)
─────────────────── ~R
⊢ ~(∀x. p(x)), p(a)
```

`--timeout` and Ctrl-C hold while a derivation is built and written as
they do during the search, and an `--output` file holds a whole output or
is left as it was.

A search also keeps within a bound on its memory, `--memory-limit` (one
gibibyte by default; a size such as `512MiB` or `4GiB`, or `none`). What
the search remembers is emptied first when it no longer fits, which costs
time and never an answer; when that is not enough, the verdict is
"unknown" with the limit as its reason, exit status 3:

```console
$ linlog prove --memory-limit 100 "|- (a & b) + (a & c), ~a par (~b & ~c)"
unknown (MALL, classical, focus engine): the memory limit of 100 B was reached after 32.28µs; raise it with --memory-limit SIZE
$ linlog prove --memory-limit 100 --format json "|- (a & b) + (a & c), ~a par (~b & ~c)"
{"version":1,"linlog":"…","verdict":"unknown","reason":{"kind":"memory_limit","limit_bytes":100},"fragment":"MALL","mode":"classical","engine":"focus","statistics":{"nodes":0,"memo_hits":0,"memo_entries":0,"splits":0,"links":0,"tests":0,"copies":0,"forward_copies":0,"work":0}}
```

The bound counts the sequent as the search holds it (about 25 bytes for
each subformula occurrence) and what grows with the search (what it
remembers, the proofs it keeps, what each level of its recursion takes).
The sequent has a limit of its own too, `--occurrence-limit` (fifty million
subformula occurrences by default, or `none`), on every command that
reads a sequent: a sequent in JSON can share subformulas, so a file of
438 bytes that doubles one atom 25 times stands for 67 million
occurrences, and is refused before anything unfolds it:

```console
$ linlog seq fragment --file shared.json
error: the sequent unfolds to 67108863 subformula occurrences, more than the limit of 50000000; raise it with --occurrence-limit
```

A first-order sequent's terms have a limit of their own, `--term-limit`
(fifty million by default, or `none`): the terms its atoms' arguments
unfold to, which shared terms in JSON can make many from a short file,
and the entries of a proof's instance table that `check` reads:

<!-- readme-check: machine -->
```console
$ linlog seq fragment --term-limit 2 "|- p(f(f(a)))"
error: the first-order terms unfold to at least 3 terms, more than the limit of 2; raise it with --term-limit
```

No input ends the command other than by a verdict or an error: a formula
nested a hundred thousand deep is parsed, printed and searched without
recursion on it (the search itself gives up at `--recursion-limit`), and
a parse error far into a long line is shown with the part of the line
around it.

`--format json` writes the outcome as one JSON object, which is also a proof
file that `linlog check` verifies independently of the search (pass the same
logic flags):

```console
$ linlog prove --format json "A |- A"
{"version":1,"linlog":"…","verdict":"proved","checked":true,"fragment":"MLL","mode":"classical","engine":"focus","statistics":{"nodes":1,"memo_hits":0,"memo_entries":1,"splits":0,"links":0,"tests":0,"copies":0,"forward_copies":0,"work":16},"sequent":{"terms":[{"D":0},{"V":0}],"roots":[0,1],"atoms":["A"],"antecedents":1},"nodes":[{"ax":[0,1]}]}
$ linlog prove --format json "A |- A" | linlog check --quiet
valid proof of ⊢ ~A, A (classical)
$ linlog prove --format json "A |- A" | linlog check --format json
{"version":1,"verdict":"valid","mode":"classical"}
```

`check`'s time limit is that of `prove`: two seconds unless `--timeout`
says otherwise (`--timeout none` lifts it), counted from the start of the
command. It exits with 0 for a valid proof, 1 for an invalid one and 3
when it gave up at `--timeout`, `--work-limit` or `--memory-limit` without a
verdict (a proof file can make its check quadratic); in JSON an invalid
or unknown verdict carries the library's error object.
