# linlog

[![DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.23448588.svg)](https://doi.org/10.5281/zenodo.23448588)

A linear logic suite for all your needs: a command line program and a Rust
library that parse, print and decide sequents of classical and
intuitionistic linear logic and their fragments, keep the proofs in a
checkable form and show them as derivation trees or proof nets, prove
step by step, export to LaTeX, Typst, SVG and Rocq, and decide ordinary
propositional logic through its embeddings into linear logic.

## Install

The command `linlog` is the package `linlog-cli`, on
[crates.io](https://crates.io/crates/linlog-cli), and the flake of this
repository:

```sh
cargo install --locked linlog-cli                 # a Rust toolchain builds it
nix run github:linlog-prover/linlog -- --help     # or run it with Nix
nix build github:linlog-prover/linlog             # result/bin/linlog
```

Building needs Rust 1.92 or later; a release raises that only with its
second number (0.1 to 0.2).

The library is the crate [`linlog`](https://crates.io/crates/linlog):

```toml
[dependencies]
linlog = "0.1"
```

with the default features `parse`, `serialize`, `interactive`, `latex`,
`typst`, `svg` and `rocq`, and `parallel` (a thread pool, not for wasm),
`png` and `pdf` to add. The documentation's examples read and write JSON
with `serde_json`, the caller's own dependency.

The Rocq certificates the command writes (`--format rocq`) are checked
by the NanoYalla kernel, whose installation "Usage" describes with them.

## A first proof

```console
$ linlog prove "A, A -o B |- B"
provable (MLL, classical, focus engine)
─────── ax   ─────── ax
⊢ ~A, A      ⊢ ~B, B
──────────────────── ⊗
  ⊢ ~A, A ⊗ ~B, B
```

The verdict, the fragment the sequent lives in and the engine that
searched, then the proof as a derivation tree of the one-sided sequent
calculus, which the library's independent checker has validated.
"Usage" below has every command; `linlog --help` and
`linlog <command> --help` document every option.

## Documentation, results and how linlog was written

- The library's API: [docs.rs/linlog](https://docs.rs/linlog) for the
  releases, [linlog-prover.github.io/linlog](https://linlog-prover.github.io/linlog/)
  for `main`, rebuilt on every push.
- The changes of every release:
  [CHANGELOG.md](https://github.com/linlog-prover/linlog/blob/main/CHANGELOG.md).
- The benchmarks: the latest baseline of the engines on the generated
  families and the LLTP library,
  [bench/RESULTS.md](https://github.com/linlog-prover/linlog/blob/main/bench/RESULTS.md),
  the baselines compared,
  [bench/COMPARISON.md](https://github.com/linlog-prover/linlog/blob/main/bench/COMPARISON.md),
  and the comparison with the other provers of linear logic,
  [bench/TOOLS.md](https://github.com/linlog-prover/linlog/blob/main/bench/TOOLS.md)
  ("Beside the other provers" below).
- To cite linlog, GitHub's "Cite this repository" reads
  [CITATION.cff](https://github.com/linlog-prover/linlog/blob/main/CITATION.cff).
- linlog's code, tests and documentation were written with substantial
  AI assistance, by Anthropic's Claude models in Claude Code, in steps
  the author planned, directed and accepted. Every proof the library
  returns has passed its independent checker, and a Rocq certificate
  can be checked by Rocq.

## Usage

The command line program is `linlog` (`cargo install --locked linlog-cli`, `nix run
github:linlog-prover/linlog -- …`, or `cargo run -p linlog-cli -- …` in a
checkout). `linlog --help` and `linlog <command> --help` document every
option.

`linlog prove` decides a sequent, written as below (`-o` is ⊸, `*` ⊗,
`par` ⅋, `~` negation; the whole syntax is under "The syntax" further
down, and `linlog --help` prints it). The first line gives the verdict,
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
sign), or the count equation of the multiplicatives; otherwise it says
that the search was exhaustive. The
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
choose the logic, `--fragment` and `--engine focus|net|two-sided|additive|horn`
override what detection picks, `--timeout`, `--copies N` and
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

`--net` shows the proof as a proof net instead of a derivation: the
sequent, the axiom links as pairs of literals with their positions in the
sequent's subformula numbering, and the verdict of the correctness
criterion. With the net engine this is the net the search found; with the
focus engine it is read off the proof. Proof nets exist for MLL without
units, with or without Mix:

```console
$ linlog prove --net "|- A * B, C * (~A par ~B), ~C"
provable (MLL, classical, focus engine)
⊢ A ⊗ B, C ⊗ (~A ⅋ ~B), ~C
A[1] — ~A[6]
B[2] — ~B[7]
C[4] — ~C[8]
proof net
$ linlog prove --engine net "A & B |- A"
error: proof nets exist for MLL without units only, not for ALL
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
only when a bound was searched exhaustively without ever hitting it, and
"unknown" when the time limit, the bound of `--copies` or another limit
ended the search first. The line says which, after how long, at which copy
bound, and which flag changes it; `--stats` adds the copy bound reached:

<!-- readme-check: machine -->
```console
$ linlog prove -q "!(A & B) |- A * B"
provable (LL, classical, focus engine)
$ linlog prove -q --copies 1 "!(A & B) |- A * B"
unknown (LL, classical, focus engine): the copy bound of 1 was reached after 54.08µs; raise it with --copies N, or lift it with --copies none to deepen it while the time limit lasts
$ linlog prove -q "A |- !A"
unprovable (MELL, classical, focus engine): the search was exhaustive
$ linlog prove -q "!(A -o A * A), !(A -o B), !(B * B -o C), A |- C * ?D"
unknown (MELL, classical, focus engine): the time limit of 2s was reached at a copy bound of 97; --timeout DURATION gives the search longer
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
$ linlog prove -q -a "!(A -o A * A), A |- ?B"
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
within a fraction of a second of it, on one thread and on several:

<!-- readme-check: machine -->
```console
$ linlog prove -q --engine focus --timeout 1s "!(A -o A * A), !(A -o B), !(B * B -o C), A |- C * ?D"
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
shares its memo; the net engine splits its search into cubes at its
first choices, and where every link is forced it makes each link once,
as one thread does; the additive engine is sequential in every case. A
search never uses more threads than the machine runs at once, and a
larger `--jobs` is taken as that many, with a note:

<!-- readme-check: machine -->
```console
$ linlog prove -q -j 4 --copies 3 "!(A -o A * A), !(B * B -o C), A, B |- A * A * A * ?D"
unknown (MELL, classical, focus engine): the copy bound of 3 was reached after 1.79ms; raise it with --copies N, or lift it with --copies none to deepen it while the time limit lasts
$ linlog prove -q --deterministic --copies 3 "!(A -o A * A), !(B * B -o C), A, B |- A * A * A * ?D"
unknown (MELL, classical, focus engine): the copy bound of 3 was reached after 2.55ms; raise it with --copies N, or lift it with --copies none to deepen it while the time limit lasts
$ linlog prove -q -j 10000 "A |- A"
note: --jobs 10000 is more than this machine runs at once; a search uses 16
provable (MLL, classical, focus engine)
```

`--intuitionistic` (`-i`) reads the sequent as intuitionistic linear logic,
as it is written: the hypotheses left of `⊢`, exactly one formula right of
it, and pars only as implications, which the one-sided form keeps as
`~A ⅋ B` with the antecedent on the left. The verdict line names the intuitionistic
fragment, and the derivation is two-sided with the rules of ILL. A
sequent of two additive-only
formulas to the *additive engine* (a recursion on pairs of subformulas,
linear in the product of their sizes, in every mode), a Horn program
with a clause under `!` to the Horn engine, as classically, and
everything else to the *two-sided engine*, the focused search keeping one goal on every
branch, with the same copy bound and affine mode as the classical one;
IMLL without units, each atom once with each sign, goes to the net engine
where the two-sided engine stops at its recursion limit or memory bound
(the classical net of the one-sided sequent is always an intuitionistic
proof):

```console
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
$ linlog prove -i "|- A par B"
error: not an intuitionistic sequent: the subformula A ⅋ B is neither an intuitionistic formula nor the negation of one (⅋ only as A ⊸ B, that is ~A ⅋ B, and ? only under a negation)
$ linlog prove -i "|- ~A, B"
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

`linlog interact` proves a sequent step by step, reading commands from
standard input: the open goals are listed with the position of every
formula, `rules` names the rules that act on a formula, `apply` applies one
(a `⊗` or Mix takes the positions of the formulas that go to its left
premise), `undo` retracts the last step, `close` lets the search close one
goal or all of them, `show` draws the derivation so far with the open
goals as bare sequents (`show --latex`, `show --typst` and `show --svg`
as proof trees), `save` and `load` keep a session as JSON (`load` takes
a session of the same sequent and mode, `--state FILE` any), and `proof`
checks the finished proof independently and prints it or writes it for
`check` (`proof --rocq` certifies it, `proof --pdf FILE` draws it, and
`show proof.pdf` writes the derivation so far, open goals included;
every format of `prove` is a word with dashes there, and a file without
one takes the format its extension names). In intuitionistic mode the goals are two-sided and the
rules carry the names of ILL:

```console
$ linlog interact -i "A, A -o B |- B"
> goals
goal 0: 0: A, 1: A ⊸ B ⊢ 2: B
> rules 0 1
⊸L (with a split)
> apply 0 1 -oL 2
error: a premise would have 2 formulas on the right of ⊢ instead of one
> apply 0 1 -oL 0
opened goal 1: 0: A ⊢ 1: A
opened goal 2: 0: B ⊢ 1: B
> apply 1 0 ax
closed
> show
───── ax
A ⊢ A      B ⊢ B
──────────────── ⊸L
  A, A ⊸ B ⊢ B
> close
goal 2: provable (IMLL, intuitionistic, two-sided engine)
no goal is open: `proof` checks the proof
> proof
valid proof (intuitionistic)
───── ax   ───── ax
A ⊢ A      B ⊢ B
──────────────── ⊸L
  A, A ⊸ B ⊢ B
```

The exit status is 0 when the session ends with a finished proof that
checks, 1 otherwise; a command's error is printed as `error: …` and the
session goes on, and 2 means the session could not start or read its
input. The same operations are the library's `Interactive`
type, whose JSON form is what a web client will hold between requests.

`linlog seq` prints a sequent one-sided in negation normal form, or
two-sided as intuitionistic linear logic reads it, converts it to JSON
(which a `.json` file or `--input-format json` reads back), or names its fragment:

```console
$ linlog seq print "A * B -o C |- A -o B -o C"
⊢ (A ⊗ B) ⊗ ~C, ~A ⅋ (~B ⅋ C)
$ linlog seq print -i "A * B -o C |- A -o B -o C"
(A ⊗ B) ⊸ C ⊢ A ⊸ (B ⊸ C)
$ linlog seq fragment -i "A & B |- 1"
IMALL
$ linlog seq json "A |- A"
{"version":1,"terms":[{"D":0},{"V":0}],"roots":[0,1],"atoms":["A"],"antecedents":1}
$ linlog seq fragment "A & B |- 1"
MALL
```

`--format latex` and `--format typst` write the derivation of `prove` and
`check` as a proof tree to paste into a paper or slides: LaTeX for the
[ebproof](https://ctan.org/pkg/ebproof) package with the connectives of
[cmll](https://ctan.org/pkg/cmll) and amssymb (turnstiles aligned in
two-sided trees), Typst for the
[curryst](https://typst.app/universe/package/curryst) package. The verdict
line becomes a comment. `seq print` takes the same formats for a sequent:

```console
$ linlog prove -i --format latex "A, A -o B |- B"
% provable (IMLL, intuitionistic, two-sided engine)
\begin{prooftree}
\infer0[$\mathrm{ax}$]{A &\vdash A}
\infer0[$\mathrm{ax}$]{B &\vdash B}
\infer2[$\multimap\mathrm{L}$]{A, A \multimap B &\vdash B}
\end{prooftree}
$ linlog prove --format typst "A & B |- A + B"
// provable (ALL, classical, additive engine)
#prooftree(
  rule(
    name: $⊕_1$,
    rule(
      name: $⊕_1$,
      rule(name: $"ax"$, $⊢ A^⊥, A$),
      $⊢ A^⊥, A ⊕ B$,
    ),
    $⊢ A^⊥ ⊕ B^⊥, A ⊕ B$,
  ),
)
$ linlog seq print -i --format latex "A * B -o C |- A -o B -o C"
$(A \otimes B) \multimap C \vdash A \multimap (B \multimap C)$
$ linlog seq print --format typst "A * B -o C |- A -o B -o C"
$⊢ (A ⊗ B) ⊗ C^⊥, A^⊥ ⅋ (B^⊥ ⅋ C)$
```

`--standalone` writes a document that compiles on its own instead, cropped
to the tree. Neither form chooses a font: the output takes the fonts of
the document it goes into. `pdflatex` needs the ebproof, cmll, amsfonts
and standalone packages, and `typst compile` fetches curryst 0.6.0 on
first use. Typst refuses a curryst tree with binary rules more than nine
inferences high, so a higher tree is written in a layout of linlog's own
instead: a `#context` block that lists the inferences and the Typst code
that measures and places them, which needs no package and sets a tree of
any height. `--style layout=linlog` asks for it at any height,
`layout=curryst` for curryst always, and `premise_gap`, `label_gap`,
`band` and `stroke` set its spacing, each a number and one of the units
`pt`, `mm`, `cm`, `in` and `em`:

```console
$ linlog prove --format typst --style layout=linlog "A & B |- A + B" | head -12
// provable (ALL, classical, additive engine)
#context {
  let open = "dots"
  let gap = (1.5em).to-absolute()
  let name-gap = (0.2em).to-absolute()
  let band = (0.8em).to-absolute()
  let stroke = 0.05em
  let dots = $dots.v$
  let nodes = (
    (1, $⊕_1$, $⊢ A^⊥ ⊕ B^⊥, A ⊕ B$),
    (1, $⊕_1$, $⊢ A^⊥, A ⊕ B$),
    (0, $"ax"$, $⊢ A^⊥, A$),
```

```console
$ linlog prove --format latex --standalone --output proof.tex "!A |- A * !A"
$ pdflatex proof.tex
```

Names of more than one letter are set in italics (`\mathit{foo}`,
`italic("foo")`), with the characters LaTeX or Typst treat specially
escaped; LaTeX writes Greek letters as their commands (`\alpha`) and the
`‿` and `·` of the names read from the LLTP library as `\smallsmile` and
`\cdotp`, so that pdfLaTeX compiles them. In `interact`, `show --latex` and `show --typst` draw the
derivation so far, an open goal as its sequent under vertical dots.

`--format svg` draws the derivation as an SVG image instead, and with
`--net` the proof net of the proof, for the sequents `--net` takes: the conclusions at the bottom, every `⊗` and `⅋` a circle (the
premise edges of a `⅋` dashed and blue, since a switching keeps one of
them), and every axiom link an arc over the literals it joins. The
verdict becomes an XML comment. `seq print --format svg` and `show --svg`
in `interact` draw a sequent and the derivation so far. The drawings ask for
the Euler Math font without embedding it; every text is stretched to the
width Euler Math gives it, so a viewer without the font keeps the layout.
The text stays selectable, and `--standalone` is refused, since an SVG is
always a document.

```console
$ linlog prove -i --format svg --output proof.svg "1, A & B, B -o C |- C"
$ linlog prove --net --output net.svg "A * B |- B * A"
$ linlog seq print --format svg "A |- A"
<svg xmlns="http://www.w3.org/2000/svg" role="img" width="70.272" height="27.2" viewBox="0 0 4392 1700" font-family="'Euler Math', 'Neo Euler', serif" font-size="1000" fill="black">
<title>⊢ A⊥, A</title>
<g>
<text x="300" y="1190" textLength="1801" lengthAdjust="spacing">⊢ 𝐴</text>
<text x="2101" y="790" textLength="611" lengthAdjust="spacing" font-size="700">⊥</text>
<text x="2712" y="1190" textLength="1380" lengthAdjust="spacing">, 𝐴</text>
</g>
</svg>
```

`--format png` and `--format pdf` render the same drawings as a PNG image
(at twice the drawing's size, with its title and description) or a PDF
page (its text selectable), in the Euler Math font the command carries;
with `--output` the extension names the format, which `--format`
overrides, and the verdict goes to standard error. Neither is written to
a terminal. A file is made only with a derivation or a net in it (or in
JSON, always): for an unprovable sequent the verdict goes to standard
error and no file is made, and an SVG on standard output is a drawing
or nothing. The PDF is an archival PDF/A-4 document (PDF 2.0);
`--style pdf.compatible=true` makes it PDF/A-2u (PDF 1.7) for tools and
archives that take nothing newer, and `--style pdf.accessible=true` an
accessible PDF/UA-1 document (PDF/A-2a), whose drawing a screen reader
reads as a numbered list of the proof's inferences. The document's date
is `SOURCE_DATE_EPOCH` when it is set, so a build can reproduce it byte
for byte, and the current time otherwise. Every drawing carries that
reading too, as the SVG's description.

```console
$ linlog prove -i --output proof.pdf "1, A & B, B -o C |- C"
provable (IMALL, intuitionistic, two-sided engine)
```

Everything an output shows can be changed. `--style KEY=VALUE` sets one
option of the format's options (`labels` upright, subscript or off, the
shape of an open goal, LaTeX's turnstile alignment, ebproof options and
preamble, Typst's import and page, the SVG's font, sizes and colours, a
PNG's scale, the text tree's bar and gap; `--help` lists them), with a
prefix such as `svg.` for another format; `--style-file` reads them all
as JSON, one object per format, which is the form a front end keeps;
`--lemma` and `--prelude` name the Rocq lemma and its file's first lines;
`--no-verdict` writes the derivation alone.

```console
$ linlog prove -i --format latex --style labels=subscript --style align=false "A, A -o B |- B"
% provable (IMLL, intuitionistic, two-sided engine)
\begin{prooftree}
\infer0[$\mathrm{ax}$]{A \vdash A}
\infer0[$\mathrm{ax}$]{B \vdash B}
\infer2[$\multimap_{\mathrm{L}}$]{A, A \multimap B \vdash B}
\end{prooftree}
$ linlog prove --style gap=6 --style bar="=" "A * B |- A * B"
provable (MLL, classical, focus engine)
======= ax      ======= ax
⊢ ~A, A         ⊢ ~B, B
======================= ⊗
    ⊢ ~A, ~B, A ⊗ B
    ================ ⅋
    ⊢ ~A ⅋ ~B, A ⊗ B
$ echo '{"svg": {"background": "white"}, "png": {"scale": 3}}' > style.json
$ linlog prove --net --style-file style.json --output net.png "A * B |- B * A"
provable (MLL, classical, focus engine)
```

The first two draw these:

![The derivation of 1, A & B, B ⊸ C ⊢ C](https://raw.githubusercontent.com/linlog-prover/linlog/main/core/tests/snapshots/ill.svg)
![The proof net of ⊢ A⊥ ⅋ B⊥, B ⊗ A](https://raw.githubusercontent.com/linlog-prover/linlog/main/core/tests/snapshots/net.svg)

`--settings FILE` reads every setting the flags set from one JSON file,
in the form a front end keeps: an object with any of `clock`, `limits`,
`search`, `view`, `styles`, `batch` and `ordinary`, each holding the
fields the flags name in their `--help` (`--copies` sets
`search.copies`, `--timeout` `clock.time_limit_ms`, `--memory-limit`
`limits.memory_bytes`). A field the file leaves out keeps the command's
default, and a flag given beside the file overrides it. `batch.mode` is
the mode of a batch's entries that name none, as the mode flags set it;
`ordinary` applies under `--logic`, which names the logic: the file's
`ordinary.logic` is not read (the whole settings a front end keeps always
hold one, which would make every input ordinary logic), and its
`ordinary.translation` is the translation unless `--translation` names
one.

```console
$ echo '{"search": {"copies": 0, "engine": "focus"}}' > settings.json
$ linlog prove --settings settings.json "|- ?(~a * ~a), a * a * a * a"
unknown (MELL, classical, focus engine): the copy bound of 0 was reached after 300.39µs; raise it with --copies N, or lift it with --copies none to deepen it while the time limit lasts
$ linlog prove --settings settings.json --copies 3 "|- ?(~a * ~a), a * a * a * a"
unprovable (MELL, classical, focus engine): the search was exhaustive
```

`--format rocq` writes the derivation as a Rocq certificate for one of
two kernels, which `--kernel` names: `nanoyalla`, the default for
classical linear logic, or `linlog`, linlog's own Rocq library, the
default for Mix, affine and intuitionistic logic. The verdict becomes a
comment, and `--standalone` adds the import lines.

For `nanoyalla` it is a proof script that the Rocq kernel
[NanoYalla](https://github.com/ComputerAidedLL/click-and-collect/tree/master/nanoyalla)
checks, the kernel of Click & coLLecT: a lemma stating the one-sided
sequent over the atoms as `formula` variables, proved by one derived rule
of the kernel per inference, with an exchange before a `⊗` where the
kernel's list sequents need one, and closed by `Qed`, so Rocq accepts the
file only if the kernel accepts the proof. A proof with Mix or with the
weakening of affine mode has no certificate there, since the kernel has
no such rule; an intuitionistic proof is certified as the classical proof
it is.

```console
$ linlog prove --format rocq "A * B |- B * A"
(* provable (MLL, classical, focus engine) *)
Lemma certificate (A B : formula) : ll [parr (dual A) (dual B); tens B A].
Proof.
apply (parr_r_ext []); cbn_sequent.
apply (ex_perm_r [2; 0; 1] [dual B; tens B A; dual A]).
apply (tens_r_ext [dual B]); cbn_sequent.
{
  ax_expansion.
}
{
  ax_expansion.
}
Qed.
```

To check a certificate, install NanoYalla 1.1.3 (the `nanoyalla`
directory of the Click & coLLecT repository) with Rocq 9 and its standard
library: `./configure && make && make install` there, or
`rocq compile -R . NanoYalla nanoll.v` and the same for `macroll.v`, then
compile the certificate with the kernel on the load path:

```console
$ linlog prove --format rocq --standalone --output proof.v "!A, !B |- !(A * A)"
$ rocq compile -R path/to/nanoyalla NanoYalla proof.v
```

Rocq prints nothing for a certificate it accepts. The kernel needs no
Yalla installation, and the certificate uses no cut and no axiom.

For `linlog` it is a lemma stating the sequent over the library's own
calculus, with the atoms as strings: one-sided in the calculus of the
mode (`ll classical`, or with Mix or weakening), and two-sided for an
intuitionistic proof (`ill intuitionistic` or `ill affine`, hypotheses
and one goal). It is proved by the library's soundness theorem from the
derivation written as a proof term, which the library's checker accepts
by computation, so Rocq accepts the file only if the checker accepts the
term; every rule has its term, Mix and weakening included:

```console
$ linlog prove -i --format rocq "A, A -o B |- B"
(* provable (IMLL, intuitionistic, two-sided engine) *)
Lemma certificate : ill (atom := string) intuitionistic [ivar "A"; ilmap (ivar "A") (ivar "B")] (ivar "B").
Proof.
  apply (soundness_ill String.eqb String.eqb_spec intuitionistic
    (ILmapL (ivar "A") (ivar "B") (IAx "A") (IAx "B"))).
  vm_compute. reflexivity.
Qed.
$ linlog prove --format rocq --kernel linlog "A * B |- B * A"
(* provable (MLL, classical, focus engine) *)
Lemma certificate : ll (atom := string) classical [parr (covar "A") (covar "B"); tens (var "B") (var "A")].
Proof.
  apply (soundness_classical String.eqb String.eqb_spec
    (Par (covar "A") (covar "B") (Tensor (var "B") (var "A") (Ax "B") (Ax "A")))).
  vm_compute. reflexivity.
Qed.
```

The library is `rocq/` of this repository, on Rocq 9 and its standard
library alone (`nix build .#rocq-linlog` builds it, or `make` in `rocq/`
with Rocq on the path); compile a certificate with its theories on the
load path:

```console
$ linlog prove --mix --format rocq --standalone --output mix.v "A, B |- A, B"
$ rocq compile -R path/to/rocq/theories Linlog mix.v
```

The syntax: `*`/`⊗` tensor, `|`/`par`/`⅋` par, `&` with, `+`/`⊕` plus,
`-o`/`⊸` linear implication, `~A` or `A^` negation, `!` and `?`, and the
units `1`, `bot`/`⊥`, `top`/`⊤`, `0`; `|-` or `⊢` separates the sides.

### Ordinary logic

`--logic classical`, `intuitionistic` or `minimal` reads a formula or a
sequent of ordinary propositional logic instead, written with `->`/`→`,
`/\`/`∧`, `\/`/`∨`, `~`/`¬`, `<->`/`↔`, `true`/`⊤` and `false`/`⊥`
(`~` binds tightest, then `/\`, `\/`, `->`, and last `<->`; `->` and
`<->` group to the right). It is decided by its translation into linear
logic, and the proof is read back as a derivation of LK or LJ, which is
checked by the rules of the logic before it is shown:

```console
$ linlog prove --logic classical '((a -> b) -> a) -> a'
valid (classical logic by the affine translation into affine MALL, focus engine)
  ───── ax
  a ⊢ a
 ──────── WR
 a ⊢ b, a
────────── →R   ───── ax
⊢ a → b, a      a ⊢ a
───────────────────── →L
   (a → b) → a ⊢ a
 ─────────────────── →R
 ⊢ ((a → b) → a) → a
$ linlog prove --logic intuitionistic 'a \/ b -> b \/ a'
valid (intuitionistic logic by the cbn translation into ILL, two-sided engine)
  ───── ax        ───── ax
  a ⊢ a           b ⊢ b
───────── ∨R₂   ───────── ∨R₁
a ⊢ b ∨ a       b ⊢ b ∨ a
───────────────────────── ∨L
      a ∨ b ⊢ b ∨ a
   ─────────────────── →R
   ⊢ (a ∨ b) → (b ∨ a)
$ linlog prove --logic intuitionistic '~~a -> a'
not valid (intuitionistic logic by the cbn translation into ILL, two-sided engine): the image is unprovable: the search was exhaustive
```

Classical logic goes into affine MALL without exponentials (negation
normal form, `∧` as `&`, `∨` as `⅋`), where the search needs no copy
bound. Intuitionistic logic goes into ILL by `--translation cbn`
(Girard's call-by-name translation, the default), `cbv` (his call-by-value
one) or `01` (Liang and Miller's 0/1 translation); minimal logic is the
same with false an atom. There the search deepens its copy bound under
the time limit, so an intuitionistic formula may be answered unknown
(exit status 3). `seq print` shows the image, and `--linear` the linear
proof of it:

```console
$ linlog seq print --logic intuitionistic --translation cbv 'a, a -> b |- b'
!a, !(!a ⊸ !b) ⊢ !b
$ linlog seq print --logic minimal '~a |- a -> b'
!(!a ⊸ false) ⊢ !a ⊸ b
$ linlog prove --logic intuitionistic --linear 'a, a -> b |- b'
valid (intuitionistic logic by the cbn translation into ILL, two-sided engine)
───── ax
a ⊢ a
────── !L
!a ⊢ a
─────── !R   ───── ax
!a ⊢ !a      b ⊢ b
────────────────── ⊸L
  !a, !a ⊸ b ⊢ b
 ───────────────── !L
 !a, !(!a ⊸ b) ⊢ b
```

A `.p` file under `--logic` is a problem in TPTP's syntax, as the ILTP
library writes them (`nix build .#iltp` fetches its 274 propositional
problems), and the batch takes a directory of them. `--format latex`,
`typst`, `svg`, `png` and `pdf` draw the derivation of LK or LJ, and
`--format rocq` writes a certificate over `Prop` that needs no library
(a classical one the standard library's excluded middle):

```console
$ echo 'fof(pel1, conjecture, (p => q) <=> (~ q => ~ p)).' > pel1.p
$ linlog prove --logic intuitionistic --quiet --file pel1.p
not valid (intuitionistic logic by the cbn translation into ILL, two-sided engine): the image is unprovable: the search was exhaustive
$ linlog prove --logic classical --quiet --file pel1.p
valid (classical logic by the affine translation into affine MALL, focus engine)
$ linlog prove --logic classical --format rocq 'a \/ ~a'
(* valid (classical logic by the affine translation into affine MALL, focus engine) *)
Lemma certificate : forall a : Prop, (a \/ ~ a).
Proof.
  exact (fun (a : Prop) => NNPP (a \/ ~ a) (fun k'1 : ~ (a \/ ~ a) => (let k'2 : ~ a := fun x' => k'1 (or_introl x') in let k'3 : ~ (~ a) := fun x' => k'1 (or_intror x') in (k'3 (fun h'4 : a => (k'2 h'4)))))).
Qed.
```

### Many sequents in one call

`prove` decides many sequents in one call, a batch, when it is given
`--file` more than once, a directory, a list of paths (`--files-from LIST`,
one per line, or separated by NUL with `--null` as `find -print0` writes
them, `-` for standard input), or an input format of many sequents:
`--input-format lines` (`NAME: SEQUENT` or `SEQUENT` per line, blank lines
and everything from `#` on skipped), `jsonl` (a sequent as `seq json`
writes it, or a record with a name, a mode and the sequent, per line) or
`problems` (the benchmark harness's problem files). Every sequent gets one
line, in the order of the input and as soon as it is decided, named by
its path as given, its line's name, or its file and line number:

```console
$ cat > sequents.txt
> # a comment, then four sequents
> identity: A |- A
> modus ponens: A, A -o B |- B
> A |- B
> broken: A |- (
$ linlog prove --input-format lines --file sequents.txt
identity: provable (MLL, classical, focus engine)
modus ponens: provable (MLL, classical, focus engine)
sequents.txt:4: unprovable (MLL, classical, focus engine): ~A occurs 1 more time than A in the one-sided sequent, so they cannot all meet in axioms
broken: error: cannot parse the sequent
   A |- (
         ^ unexpected end of input, expected a formula
$ linlog prove --input-format lines --file sequents.txt --format svg --output drawings
identity: provable (MLL, classical, focus engine)
modus ponens: provable (MLL, classical, focus engine)
sequents.txt:4: unprovable (MLL, classical, focus engine): ~A occurs 1 more time than A in the one-sided sequent, so they cannot all meet in axioms
broken: error: cannot parse the sequent
   A |- (
         ^ unexpected end of input, expected a formula
```

The exit status is the worst verdict: an error (2) before unknown (3)
before unprovable (1) before proved (0), so this one is 2; a batch that
holds no sequent is an error, and one interrupted with Ctrl-C, which
ends its input, is unknown at best. The second
command writes each proved sequent's derivation into the directory
`--output` names, in the format asked for, named after the sequent with
the format's extension (`drawings/identity.svg`; `ILL/01/X.p` becomes
`DIR/ILL/01/X.p.svg`, so the LLTP library's translations of one problem
do not overwrite each other). A file's kind
is its extension's unless `--input-format` says otherwise: `.p` is a
problem of the LLTP library, `.spec` a coverability problem in the format
of the Mist tool, `.json` a sequent in JSON, anything else the text
syntax, for a single `prove` as well. An LLTP problem does not say
whether it is intuitionistic (the library keeps those under `ILL/`), so
the flags say it:

```console
$ cat > problem.p
> % Status (intuit.) : Theorem
> fof(rule, axiom, !(a -o b)).
> fof(fact, axiom, a).
> fof(goal, conjecture, b * 1).
$ linlog prove -i -q --file problem.p
provable (IMELL, intuitionistic, horn engine)
```

A `.spec` problem is a Petri net or a vector addition system as the
coverability suites of software verification write it: counters, rules
that test and change them, an initial marking (`x >= k` for at least `k`
tokens, any number more), and the markings to cover, a line each. It is
read as a Horn program, the rules under `!` and the initial tokens left
of `⊢`, and covering is a question of affine mode, in which a `.spec`
file is decided whatever the flags say (`-i` and `--mix` keep their
meaning). Two processes in the critical section at once, for any number of
processes and one lock:

```console
$ cat > mutex.spec
> vars idle critical lock
> rules
>   idle >= 1, lock >= 1 -> idle' = idle - 1, lock' = lock - 1, critical' = critical + 1;
>   critical >= 1 -> critical' = critical - 1, idle' = idle + 1, lock' = lock + 1;
> init idle >= 1, lock = 1
> target critical >= 2
$ linlog prove -i --affine -q --file mutex.spec
unprovable (IMELL, intuitionistic affine, horn engine): the state equation of the Petri net has no solution, so no firing of its clauses yields the goal's atoms: weighting each lock by 1 and critical by 1, no clause that can fire raises the weighted count of the atoms, and the goal asks it raised
```

A directory is walked in sorted order, links followed, for the files of
the input format's extension (`.p` and `.json` by default); paths are
taken literally, relative to the current directory. On standard input
the batch is a stream: each line is answered as soon as it is decided,
so a program can ask, wait for the answer and ask again. With `--format
json` every answer is a JSON Lines record, the wire level and the name
first and then what `--format json` writes for one sequent (with
`--stats` also `seconds`, the time it took, which a single sequent's JSON
leaves out to stay reproducible); an entry
that could not be read or searched is `"verdict": "error"` with a
`message`, and the library's `error` object (its `code`, `kind` and
`details`) where the error is the library's, such as a parse error or a
bound that refused the input, and one the batch stopped before
it was decided `"verdict": "unknown"` with a `message`. The mode is the
flags' (else a settings file's `batch.mode`), a problem file's column or
a record's:

```console
$ echo '{"name": "pair", "mode": "intuitionistic", "sequent": "A, B |- A * B"}' | linlog prove --input-format jsonl --format json
{"version":1,"name":"pair","linlog":"…","verdict":"proved","checked":true,"fragment":"IMLL","mode":"intuitionistic","engine":"two-sided","statistics":{"nodes":1,"memo_hits":0,"memo_entries":1,"splits":1,"links":0,"tests":0,"copies":0,"forward_copies":0,"work":16},"sequent":{"terms":[{"D":0},{"D":1},{"V":0},{"V":1},{"⊗":[2,3]}],"roots":[0,1,4],"atoms":["A","B"],"antecedents":2},"nodes":[{"ax":[3,0]},{"ax":[4,1]},{"⊗":[2,0,1]}]}
```

By default the cores go across the sequents, one sequent per worker on
the sequential engines (as with `--deterministic`, so a sequent that
only a pool decides within its time limit stays unknown), and within one
sequent, as for a single call, when the batch has fewer sequents than
workers or is a stream from standard input; `--cores across|within`
chooses, and `--workers N` caps the sequents decided at once.
Across the cores a batch answers the most sequents in a given time, and
each answer is a function of its input; what it leaves unknown is
worth a second pass on the whole machine, which costs little because
it is a few sequents:

```sh
linlog prove -i --file problems/ > results.txt
grep ': unknown' results.txt | cut -d: -f1 > unknown.txt
linlog prove -i --files-from unknown.txt --cores within > second.txt
```

`--timeout` is each sequent's limit, reading included, and
`--batch-timeout` the whole batch's. Each search holds at most
`--memory-limit`, and as many run at once as `--batch-memory` holds (by
default half the memory the process may use, the machine's or its
control group's), so that a batch never holds more
than that for its searches; `--isolate` decides each sequent in a child
process of its own, so that one that exhausts the memory or the stack
ends only itself.

### Benchmarks

`linlog-bench` (`cargo run --release -p linlog-bench -- …` in a checkout,
or `nix build .#linlog-bench`) times the engines on four kinds of
problems: generated families with known verdicts (`linlog-bench families`
lists them: 3-Partition as a Horn program and as an MLL sequent,
Matsuoka's Partition, random QBF, wide sequents, contexts under Mix,
Petri-net counters and more, each at any size), the problems of the
[LLTP library](https://github.com/meta-logic/lltp) (`nix build .#lltp -o
bench/lltp` fetches it at a pinned commit; its problems under `ILL/` run
intuitionistically), coverability problems in the `.spec` format with
`--spec` (the 176 of the [qcover](https://github.com/blondimi/qcover)
suite: `nix build .#qcover -o bench/qcover`; they run intuitionistic
affine), and problem files of lines `name; mode; expected; copies;
sequent` such as `bench/problems/slow-tests.txt`. `run` runs every
problem in every mode, engine and thread count asked for, each run in a
child process of its own with a time limit, and writes one CSV row per
run with the verdict, the time and the engine's counters; `summary`
prints Markdown tables of CSV files: the problems solved within the time
limit per family and configuration, and a time per problem and
configuration.

```console
$ linlog-bench run --family partition-no=3,4 --engines focus,net --jobs 1,4 --timeout 10 --output runs.csv
[1/8] partition-no/3 classical focus j1: unprovable  0.089 ms, about 0 min left
[2/8] partition-no/3 classical focus j4: unprovable  0.146 ms, about 0 min left
[3/8] partition-no/3 classical net j1: unprovable  3355.671 ms, about 0 min left
[4/8] partition-no/3 classical net j4: unprovable  1020.791 ms, about 0 min left
[5/8] partition-no/4 classical focus j1: unprovable  0.146 ms, about 0 min left
[6/8] partition-no/4 classical focus j4: unprovable  0.303 ms, about 0 min left
[7/8] partition-no/4 classical net j1: unknown timeout 10000.591 ms, about 0 min left
[8/8] partition-no/4 classical net j4: unknown timeout 10000.139 ms, about 0 min left

$ linlog-bench summary runs.csv
…
## partition-no

| problem | runs: classical focus j1 | runs: classical focus j4 | runs: classical net j1 | runs: classical net j4 |
|---|--:|--:|--:|--:|
| partition-no/3 | 89 µs ✗ | 146 µs ✗ | 3.36 s ✗ | 1.02 s ✗ |
| partition-no/4 | 146 µs ✗ | 303 µs ✗ | > 10 s | > 10 s |
```

`summary --before DIR` compares every file with the file of the same
name in `DIR`, an earlier baseline, problem by problem instead: the
verdicts that differ, the problems decided before and not after, one row
per family or LLTP collection and configuration (problems decided before
and after, late verdicts kept apart, where the undecided ended, the
median ratio of the times), and every generated problem side by side;
given again for each earlier baseline, oldest first, it sets them all
side by side before it compares with the last. `summary --against FILE` compares every other file with `FILE` in the
same way whatever the files' names, as the passes under one bias with the
default:

```console
$ linlog-bench summary --before bench/results/2026-09-30 bench/results/2026-10-02/long-1.csv
…
| file | family | configuration | problem | before | after | after/before |
|---|---|---|---|--:|--:|--:|
| long-1 | 3-partition-no | classical auto j1 | 3-partition-no/5 | 302.42 s ✗ | 393 µs ✗ | 1.3e-6× |
| long-1 | counter | classical auto j1 | counter/16 | > 1200 s | 280 µs ✓ |  |
| long-1 | counter | classical auto j1 | counter/64 |  | 85.14 s ✓ |  |
| long-1 | partition-no | classical auto j1 | partition-no/15 |  | > 1200 s |  |
| long-1 | partition-no | classical auto j1 | partition-no/5 | 811.48 s ✗ | 477 µs ✗ | 5.9e-7× |
| long-1 | partition-yes | classical auto j1 | partition-yes/28 |  | > 1200 s |  |
| long-1 | partition-yes | classical auto j1 | partition-yes/7 | > 1200 s | 230 µs ✓ |  |
```

`run --bias rarer|factors` runs the focused engines under that bias and
`run --forward-copies N` the default's forward search within that bound,
and the rows say which. `run --copies none` deepens the copy bound until
the time limit, and `run --pool-after SECONDS` searches on one thread for
that long before a pool of the other `--jobs` threads joins it, as the command does by
default; the rows have the copy bound reached (`copies_reached`). `bench/targets.sh LABEL` runs the target set of the
focused engine's performance work, the instances the first baseline
showed it losing on (the hard families at the sizes that took minutes or
did not finish, and a fixed sample of 113 LLTP problems), on two pinned
cores in a memory-capped user unit, into `bench/targets/LABEL.csv`;
`bench/TARGETS.md` sets the engine before that work beside the engine
after it. On one thread the engine's counters are a function of the
input, so two such files tell whether a change altered the search at all.

`bench/baseline.sh --detach --fresh` takes the whole baseline, every
family, engine and thread count and the whole LLTP library, at once as
a systemd user unit on an otherwise idle machine, in about three hours;
`--arm` instead starts it unattended in the night, at 20:00 (or at once
if that has passed) once the machine is idle, and stops it at 07:00
whatever its state (`--slot=HH:MM-HH:MM` for other times; the script run
again without `--fresh` finishes a stopped baseline).
Every baseline keeps a directory of its own named by the day it started,
`bench/results/DAY/`: the CSV files of its runs, `starts.txt` with the
commit measured, and its tables in `RESULTS.md`, which `bench/RESULTS.md`
copies for the latest baseline; `journalctl --user -fu linlog-baseline`
follows it. A last stage runs again, with more time before the kill and
more memory, the runs that `bench/reruns.txt` lists: those of an earlier
baseline that were killed or crashed and that measurement showed to
finish with more room. The first baseline, of the night of 2026-09-30,
took 9 h 21 min on a 16-core Intel Core Ultra X9 388H, and a supplement
on the next night added that last stage to it in 1 h 28 min; the second,
of 2026-10-02, after the performance work on the focused engine, took
8 h 4 min; the third, of 2026-10-17, of the engines of the first
release, 2 h 58 min, with the intuitionistic library under the command's
default as a pass of its own. `bench/RESULTS.md` has the third's tables
and `bench/COMPARISON.md` sets the three side by side: no verdict differs
between them, and the intuitionistic LLTP pass decides 3 610 problems
within 5 s where the first decided 737 and the second 2 047. Under the
command's default (no copy bound, every core, 2 s) it decides 4 024 of
the 4 495.

`run --tools` runs the same problems through other provers of linear
logic beside linlog's command, each built by the flake at a pinned
version (`nix build .#comparison` has them all, with BenchExec): every
problem is translated into each tool's syntax, every run is a process
measured by BenchExec's `runexec` under the same time and memory limit
(`--timeout`, `--memory-limit`, 4 GiB by default), and a problem outside a
tool's fragment or mode is a row of its own and not run. `ranking`
prints the comparison: the problems each tool solves within the limit,
per group of problems, linlog on one thread against each tool, and every
problem on which tools contradict each other or the known verdict.
`--shard K/N` runs every N-th problem from the K-th, so that jobs on
several machines split a set.

```console
$ linlog-bench run --tools linlog,maude-ill,sympli --lltp lltp/ILL/ILLTP-SYJ-01 --only SYJ101+1,SYJ105+1.003 --timeout 10 --output tools.csv
[1/6] ILLTP-SYJ-01/SYJ101+1.p linlog: proved  3.4 ms, about 0 min left
[2/6] ILLTP-SYJ-01/SYJ101+1.p maude-ill: proved  89.8 ms, about 0 min left
[3/6] ILLTP-SYJ-01/SYJ101+1.p sympli: proved  11.7 ms, about 0 min left
[4/6] ILLTP-SYJ-01/SYJ105+1.003.p linlog: unprovable  4.0 ms, about 0 min left
[5/6] ILLTP-SYJ-01/SYJ105+1.003.p maude-ill: unknown copy_bound 131.5 ms, about 0 min left
[6/6] ILLTP-SYJ-01/SYJ105+1.003.p sympli: unprovable  3.0 ms, about 0 min left

$ linlog-bench ranking tools.csv
…
## Solved within the limit

| tool | solved | proved | unprovable | wrong | late | time | memory | gave up | error | outside | against the header |
|---|---|---|---|---|---|---|---|---|---|---|---|
| linlog | 2 | 1 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| sympli | 2 | 1 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| maude-ill | 1 | 1 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
…
```

(linlog's and Sympli's refutations of SYJ105+1.003 count as solved and
against the file's header, which says Theorem; the formula has a
classical countermodel, `a` and `b` false, so the header is what is
wrong.)
`bench/comparison.sh` runs the whole comparison on any machine with Nix,
and `bench/TOOLS.md` has the tools, how they were run and the results.

## Beside the other provers

The provers of linear logic we found, as their own sources state them
(read on 2026-10-15); `bench/TOOLS.md` has every feature with its source
and the results of running all of them on the same problems.

| prover | fragments | modes | refutes | checked proofs | licence |
|---|---|---|---|---|---|
| linlog | MLL to LL, units | classical, intuitionistic, affine, Mix | ✓ | ✓ (and Rocq certificates) | EUPL-1.2 |
| Maude LL prover | LL, ILL | classical, intuitionistic | without `!` | ✗ | none stated |
| llprover | first-order LL | classical, intuitionistic | without `!`, `?` | ✗ | none formal |
| linTAP | MLL with `?` | classical | without `?` | ✗ | GPL |
| Click & coLLecT | LL | classical | ✓ | Coq certificates | LGPL-2.1 |
| Sympli | ILL without `⅋ ?` | intuitionistic | ✓ | its own | BSD-2-Clause |
| LinearOne | first-order MILL | intuitionistic | ✓ | unverified | LGPL-2.1 |
| lolli | MLL, MALL, MELL | classical | ✗ (a depth limit) | ✗ | MIT |
| APLL | LL, ILL | classical, intuitionistic | unless its bound is met | Coq certificates | none stated |
| Proof Net Calculator | unit-free MLL nets | — | no prover | net criteria | none stated |

On the same 4 811 problems (the LLTP library, qcover's coverability
problems and the generated families), each run under 10 s and 4 GiB on
one core and measured by BenchExec, linlog on one thread solves 4 181,
Sympli 1 684, the Maude prover 768 with its intuitionistic system and
31 with its classical one, APLL 612 and 31, llprover 479 and 11, Click &
coLLecT 31, LinearOne 69, linTAP 15 and lolli 14, each of the problems
in its own fragment and modes. Most of linlog's lead is the library's
3 137 Petri nets, which it decides with an engine of its own (3 072,
against Sympli's 811). On the other 1 358 intuitionistic problems Sympli
solves the most, 873 against linlog's 829 (852 by its default), and 132
that linlog does not, mostly refutations of the SYJ collections. No tool proves a
sequent linlog refutes; APLL and Click & coLLecT refute provable
sequents, 1 308 and 12 of them.

## What exists and what is planned

Built:

- A comparison with the other provers of linear logic: each built from
  its own source at a pinned version, each problem translated into its
  syntax, every run measured by BenchExec under one time and memory
  limit, the problems solved, the contradictions and the figures drawn
  from the rows (`bench/TOOLS.md`, `bench/comparison.sh`, and a CI
  workflow anyone can start).
- Parsing and printing of sequents in the syntax above, with the ASCII and
  Unicode spellings of every connective, and a compact JSON form.
- Detection of the fragment a sequent lives in (MLL, MLL with units, ALL,
  MALL, MELL, LL, and their intuitionistic counterparts IMLL to ILL) and
  the modes classical, affine, intuitionistic and Mix as user choices.
- Intuitionistic linear logic on the same one-sided representation: a
  sequent is read two-sided as it was written (the hypotheses left of
  `⊢`, one goal right of it, `⊸` recovered from `~A ⅋ B`, each formula
  checked to stand where its polarization allows), printed as `Γ ⊢ A`, and
  proved by the two-sided focused search, by the embedding of IMLL into
  MLL proof nets, by the additive fast path, or by the Horn engine.
- Proofs as compact terms over subformula occurrences, an independent
  checker that decides whether a term proves its sequent (in
  intuitionistic mode also that every sequent of the proof has one goal)
  in one pass and in memory proportional to the proof,
  and a derivation view that unfolds a term into the tree of the standard
  sequent calculus, one-sided or two-sided with the rules of ILL, printed
  as text. The size of a derivation is computed from the term without
  building it, and every path that builds one (the text tree, the
  exports, the search inside an interactive proof) keeps within a bound
  on that size, which is an option.
- Automatic proof search for every fragment, MLL to full LL and IMLL to
  ILL, with or without Mix, returning a checked proof (the checker runs
  on every proof, in every build), "unprovable" after
  an exhaustive search, or "unknown" with the reason: a focused sequent
  engine over dyadic sequents of occurrence bitsets with a memo, counts
  that prune sequents and direct the search for the split of a `⊗`
  (contexts of any width), one representative for formulas that occur
  several times, an atom bias chosen from the sequent or by `--bias`
  (with exponentials a forward and a backward search together), a
  per-branch bound on the copies of `?` formulas that deepens
  iteratively, without end or up to a bound, and a loop check, one-sided
  or two-sided (under Mix, a sequent none of whose parts is provable is
  refuted through its parts with one member less, not its partitions;
  a tensor whose splits are searched costs one level of recursion
  however long); for
  MLL without units, where the focused engines stop at a limit or when
  asked, a proof-net engine that searches the axiom linkings with count
  checks, constant-time cycle rejections, the exact acyclicity test, a
  symmetry break for equal literals and subformulas that are conclusions
  or leaves of one tree of `⊗` alone or `⅋` alone, and a per-atom balance
  over the skeleton's components, then sequentializes the net it finds;
  for two additive-only formulas a
  recursion on subformula pairs; and for Horn programs with clauses under
  `!` (Petri nets) a decision of the net's markings: in linear mode a
  search of the reachable markings, kept sparse and taken nearest the
  goal first, with a search backward from the goal beside it, whose
  firing sequence is the proof and whose exhaustion refutes; in affine
  mode the backward coverability algorithm, which always ends; and in
  both the net's state equation, whose refutation is a weighting of the
  atoms that no clause raises and the goal needs raised, found by a
  simplex and checked exactly. Which engine decides a goal is a table
  of rows, by fragment, mode and one more feature, each the engine
  measured fastest there or the one that decides there at all; the
  library's documentation of `Engine` has it with the measurements.
- Affine mode, where weakening is allowed, in every fragment.
- Proof nets for MLL, with or without Mix, as a representation of their
  own: proof structures over the subformula occurrences, an independent
  correctness criterion (Danos–Regnier, decided by Yeo's deletion test on
  the coloured structure graph, with a switching cycle or the
  disconnection named when it fails), sequentialization into a checked
  proof and desequentialization of a proof into its net, a text form and
  a JSON form.
- Interactive proving: a proof in progress as a derivation with open
  goals, rules applied to a formula of a goal and validated (the
  connective, the mode, the context a promotion or an axiom needs, one
  goal per premise in intuitionistic mode), undo, the search closing any
  goal, translation of the finished derivation into a term the checker
  validates, and a JSON form of the session.
- Export of sequents and derivations, finished or in progress, to LaTeX
  (ebproof proof trees) and Typst (curryst proof trees, and trees of any
  height in a layout of linlog's own), as fragments or standalone
  documents that choose no font, and drawings of sequents,
  derivations and proof nets as SVG, laid out with the character widths
  of the Euler Math font (or another font's), a switching cycle or the
  parts of a disconnected structure highlighted, with a reading for
  screen readers, and
  rendered as PNG and as archival (PDF/A-4, PDF/A-2u) or accessible
  (PDF/UA-1) PDF.
- Output configured through the library: one options value per format,
  with serde, for rule labels (one table per convention, or the user's),
  the shape of an open goal, alignment, preambles, the SVG's font, sizes
  and colours, and the certificate's names; the command sets them with
  `--style` and `--style-file`, and writes a derivation as it is made.
- Proof certificates: a finished proof as a Rocq script for the NanoYalla
  kernel, a lemma proved rule by rule and closed by `Qed`, for every
  classical fragment and for intuitionistic proofs as the classical
  proofs they are, with a flake check that runs Rocq on them.
- A Rocq library of linlog's own (`rocq/`, the package `rocq-linlog`,
  on Rocq 9 and its standard library alone): the one-sided calculus of
  classical linear logic as a plain inductive, with Mix and general
  weakening as parameters, and the two-sided calculus of intuitionistic
  linear logic and its affine variant; linlog's proof terms as
  datatypes, one-sided and two-sided, checkers as functions, and for
  every mode the theorem that a term the checker accepts proves its
  sequent, closed under the global context; a certificate is the
  sequent, the term and one computation. `--format rocq` writes them as
  its second kernel (`--kernel linlog`, the default for Mix, affine and
  intuitionistic proofs). The flake builds the library and compiles the
  certificates under `rocq/tests/` and those the command writes against
  it.
- Parallel search, behind the library's `parallel` feature and on by
  default in the command: the focus engine runs the choices nearest the
  root on a thread pool, cube-and-conquer style, with the `&` premises in
  parallel and one memo shared by every thread; the net engine splits its
  search into cubes at its first choices for the pool; a stop condition
  reaches every thread; a pool has at most as many threads as the machine
  runs at once; the sequential engines stay one flag away.
- Time limits that hold: a stop condition is asked wherever a search can
  spend time (stable sequents, split searches, chains of forced splits,
  the set-up on a large sequent, every thread of a pool), and the
  command's `--timeout` counts from its start, reading and parsing
  included, and is kept to within a fraction of a second.
- Limits on memory and on the input: a search holds at most
  `--memory-limit` bytes (its memo is emptied first, then the answer is
  "unknown" with the reason), a sequent unfolds to at most
  `--occurrence-limit` occurrences, and no walk over a formula, a net
  or a derivation recurses on the input's depth.
- Defaults a newcomer can use, each of them an option: the copy bound
  deepens while a time limit of two seconds lasts, one thread searches
  for a tenth of a second before the other cores join it, an "unknown"
  says which bound or limit ended the search, after how long and at which
  copy bound, with the flag that changes it, and an "unprovable" says why
  where the counts of the sequent tell (an atom whose literals cannot
  pair up, the count equation that fails).
- The `linlog` command: `prove`, `check`, `interact` and `seq`, with time
  limits, Ctrl-C, statistics, JSON output, proof nets, LaTeX, Typst,
  SVG and Rocq output, and `--jobs`, `--pool-after` and `--deterministic`
  for the search; every setting a flag sets read from one file with
  `--settings` as well, the form a front end keeps; `check` under a time
  limit and limits of work and memory, unknown where one ends it;
  a proof tree is printed on a terminal where it fits, a run of one
  structural rule is drawn as one inference where the whole tree would
  not fit or pass `--derivation-limit`, and a derivation past the limit
  either way is left out with a line that says so.
- Many sequents in one call: files, directories, lists of paths and
  streams on standard input, in the text syntax, as JSON Lines, in the
  harness's problem files and in the LLTP library's format, one answer
  per sequent in input order, with a time limit per sequent and for the
  batch, a memory bound for the batch, and a child process per sequent
  on request.
- Ordinary classical, intuitionistic and minimal propositional logic
  decided through their embeddings into linear logic: a syntax of its
  own and a reader for TPTP problems, the translations by name (classical
  logic into affine MALL, Girard's call-by-name and call-by-value
  translations and the 0/1 translation into ILL, minimal logic with false
  an atom) with the image printable, the linear proof read back as a
  derivation of LK or LJ that a checker of its own validates and every
  output draws, and a certificate over `Prop` for Rocq.
- Benchmarks: readers for the problems of the LLTP library and for
  coverability problems in the `.spec` format of the Mist tool, generated
  families with known verdicts (the hard families of the literature and
  the cases where one engine is known to be slow), and `linlog-bench`,
  which runs them with a time limit per run, writes CSV and summarises it,
  with a script that takes a whole baseline on an idle machine and one
  that runs the focused engine's target set by day, and a comparison of
  two baselines problem by problem.
- Performance work on the focused engine, measured by three baselines,
  one before it, one after it and one at the first release.
- Releases: the crates `linlog` and `linlog-cli` on crates.io, with a
  changelog that states what each version promises, and a workflow that
  publishes a version tag.

Planned, in roughly this order:

- The Rocq library's bridges to NanoYalla and Yalla, the reduction of
  Mix to `?(⊥⊗⊥)`, and certified refutations.
- A web front end for proving step by step in the browser.
- Later: proof nets with exponential boxes, cut elimination on proofs
  and on nets, further engines for intuitionistic MLL (essential
  nets), the Lambek calculus, the inverse method,
  and first-order linear logic.

linlog is very much inspired by [Click & coLLecT](https://www.click-and-collect.linear-logic.org).
It aims to support more use cases on a modern, more efficient base.
Feature requests and contributions are welcome. linlog is licensed
under the EUPL; see [Licence](#licence).

## Architecture

Three crates, with a fourth to come: the library `linlog` in `core/` holds
all the logic; the command line program `linlog` (package `linlog-cli`) in
`cli/` is a thin front end; the benchmark harness `linlog-bench` in
`bench/` runs the library's engines on problem sets; a web front end will
compile the library to
WebAssembly, so the library uses no clock, and threads only behind its
`parallel` feature, which the web front end leaves off (the flake builds
it for `wasm32-unknown-unknown` with every other feature).

The library keeps a sequent as a compact arena of subformulas in negation
normal form, one-sided (`Γ ⊢ Δ` becomes `⊢ Γ^⊥, Δ`); an intuitionistic
sequent is the same arena read two-sided, through the polarization of its
subformulas, with no second representation. On top of that it detects the
fragment a sequent lives in and builds the occurrence forest, the
numbering of subformula occurrences that every proof-search engine, proof
checker and proof net works on. A proof is a compact term over those
occurrences, one node per rule instance; an independent checker decides
whether it proves its sequent, and a derivation view unfolds it into the
tree of explicit sequents of the standard sequent calculus, one-sided or
two-sided. A proof net is the same forest with axiom links, checked by its
own criterion and convertible to and from a proof term. Proof search
decides a sequent, or any goal within one, with the engine its fragment
and mode call for, sequent search, net search, the additive recursion or
the search of a Petri net's markings,
and returns a checked proof, that there is none, or why it could not tell.
Interactive proving holds a derivation with open goals over the same
forest, with the same inferences as the derivation view, and turns it back
into a term for the checker once it is finished. The exports write
sequents and derivations, finished or not, as LaTeX and Typst source, one
inference at a time, draw them and proof nets as SVG from layouts of
their own: a tree by subtree widths, a net by its formula trees under the
axiom links, and write a finished derivation as a Rocq proof script for
the NanoYalla kernel, tracking the order of each goal's formulas so that
one exchange per `⊗` suffices.

## Development

The flake's development shell (`nix develop`, or direnv) has the
toolchain and every tool the checks use, and `nix flake check` runs what
CI runs but the online `cargo deny check advisories`. CI uploads what
`main` builds to the public binary cache `linlog-prover` on Cachix, so a
local check fetches the results of everything a change does not touch
instead of rebuilding them. To use it, add it once, as a trusted user of
the Nix daemon (otherwise `cachix` says what to add to the system's
configuration):

```sh
nix run nixpkgs#cachix -- use linlog-prover
```

or add `https://linlog-prover.cachix.org` to `extra-substituters` and
`linlog-prover.cachix.org-1:6K+DDVwwELnSYtfflwUdsHdYuolC7hprdD/r2/KOQZU=`
to `extra-trusted-public-keys` in `nix.conf`.

## Citing

Each release is archived on Zenodo. To cite linlog, use the DOI
[10.5281/zenodo.23448588](https://doi.org/10.5281/zenodo.23448588),
which always resolves to the latest version, or the DOI of the version
you used, which its Zenodo record gives. [`CITATION.cff`](CITATION.cff)
holds the entry, and GitHub's "Cite this repository" writes it as BibTeX
or APA.

## Licence

linlog is © Fabian Lukas Grubmüller and licensed under the European
Union Public Licence, version 1.2 (EUPL-1.2), whose text is in
[`LICENSE`](LICENSE).

One part of the repository is someone else's work under its own licence:
the Euler Math font in [`cli/fonts/`](cli/fonts), which the `linlog`
command embeds to draw proofs, is © the American Mathematical Society,
Khaled Hosny and Daniel Flipo, under the SIL Open Font License 1.1
([`cli/fonts/OFL.txt`](cli/fonts/OFL.txt)). The package `linlog-cli`
is therefore `EUPL-1.2 AND OFL-1.1`; the library `linlog` contains no
font. Drawings and documents made with the font are not covered by its
licence.

The problem libraries and the other provers that the benchmarks and the
comparison use are not part of this repository: the flake fetches each
at a pinned version when it is needed, under its own licence. The LLTP
library is GPL-3.0, the ILTP library states no licence, qcover's
problems are Apache-2.0, and "Beside the other provers" gives the
provers' licences.
