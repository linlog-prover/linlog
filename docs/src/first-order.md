# First-order logic

A sequent may quantify over terms: `forall x. A` (`∀x. A`) and `exists
x. A` (`∃x. A`) bind loosest, their scope running to the right as far as
it can (to a closing bracket, a comma, the turnstile or the end), and an
atom may be a predicate applied to terms, `p(x, f(a))`, where an
identifier bound by an enclosing quantifier is its variable and any
other a constant (or a function symbol, before a bracket). A symbol has
one arity throughout the sequent, and a name is a predicate or a
function symbol, not both. Every sequent is closed: no free variable.
`seq print` writes the one-sided form, a negation swapping the
quantifiers, and `seq fragment` names the first-order fragment with a
`1` after the acronym:

<!-- readme-check: machine -->
```console
$ linlog seq print "forall x. p(x) -o q(x), p(a) |- exists y. q(y)"
⊢ ∃x. (p(x) ⊗ ~q(x)), ~p(a), ∃y. q(y)
$ linlog seq print "|- ∀x y. ~r(x, f(y)) ⅋ r(x, f(y)), (exists z. p(z))^"
⊢ ∀x. (∀y. (~r(x, f(y)) ⅋ r(x, f(y)))), ∀z. ~p(z)
$ linlog seq fragment "forall x. p(x) |- p(a)"
MLL1
$ linlog seq fragment "p(a), !p(a) -o q(f(a)) |- q(f(a))"
MELL
```

A sequent whose atoms have arguments but whose formulas have no
quantifier is propositional, with structured atoms, and every engine
decides it. A sequent with a quantifier is decided by the focused
engine in the classical modes, with Mix and in affine mode too, and by
the two-sided engine in the intuitionistic modes, affine included: it
leaves the witness of each `∃` open as a variable, fills it in where an
axiom matches two atoms (never with a term that holds the variable
itself), keeps the eigenvariable of each `∀` out of every witness
chosen before it, and asks a premise for another witness where the one
it gave fails the other premise, resuming that premise's search where
it stopped. The proof it returns names every
witness, a closed term, and passes the checker (below):

<!-- readme-check: machine -->
```console
$ linlog prove "p(a), p(a) -o q(f(a)) |- q(f(a))"
provable (MLL, classical, focus engine)
───────────── ax   ─────────────────── ax
⊢ ~p(a), p(a)      ⊢ ~q(f(a)), q(f(a))
────────────────────────────────────── ⊗
  ⊢ ~p(a), p(a) ⊗ ~q(f(a)), q(f(a))
$ linlog prove "p(a) & p(b), q(b) |- exists x. (p(x) * q(x))"
provable (MALL1, classical, focus engine)
     ───────────── ax   ───────────── ax
     ⊢ ~p(b), p(b)      ⊢ ~q(b), q(b)
     ──────────────────────────────── ⊗
       ⊢ ~p(b), ~q(b), p(b) ⊗ q(b)
    ───────────────────────────────── ∃
    ⊢ ~p(b), ~q(b), ∃x. (p(x) ⊗ q(x))
───────────────────────────────────────── ⊕₂
⊢ ~p(a) ⊕ ~p(b), ~q(b), ∃x. (p(x) ⊗ q(x))
$ linlog prove "forall x. exists y. r(x, y) |- exists y. forall x. r(x, y)"; echo "exit $?"
unprovable (MLL1, classical, focus engine): the search was exhaustive
exit 1
$ linlog prove -i "exists y. forall x. r(x, y) |- forall x. exists y. r(x, y)"
provable (IMLL1, intuitionistic, two-sided engine)
          ───────────────── ax
          r(b, a) ⊢ r(b, a)
        ───────────────────── ∃R
        r(b, a) ⊢ ∃y. r(b, y)
      ───────────────────────── ∀L
      ∀x. r(x, a) ⊢ ∃y. r(b, y)
   ─────────────────────────────── ∀R
   ∀x. r(x, a) ⊢ ∀x. (∃y. r(x, y))
───────────────────────────────────── ∃L
∃y. (∀x. r(x, y)) ⊢ ∀x. (∃y. r(x, y))
```

A cut in such a session takes a formula of the sequent's symbols,
quantifiers among them, and of the eigenvariables of the `∀R` and `∃L`
steps below the goal, as a witness does: a cut on `p(a)` is the lemma
about the `a` at hand, held as the instance of `∀a. p(a)`'s body.
`cut` refuses a name the sequent lacks or has with another arity, and
an eigenvariable of another branch, by name:

```console
$ printf 'apply 0 2 forallR\ncut 1 "r(a)" 1\ncut 1 "p(a)" 1\nclose\nproof\n' | linlog interact -i "forall x. (p(x) -o q(x)), forall x. p(x) |- forall y. q(y)"
opened goal 1: 0: ∀x. (p(x) ⊸ q(x)), 1: ∀x. p(x) ⊢ 2: q(a)
error: the cut formula names r, which the sequent has not with this arity and role: a cut formula in a session with quantifiers holds the sequent's symbols and the goal's eigenvariables only
opened goal 2: 0: ∀x. p(x) ⊢ 1: p(a)
opened goal 3: 0: ∀x. (p(x) ⊸ q(x)), 2: p(a) ⊢ 1: q(a)
goal 2: provable (IMLL1, intuitionistic, two-sided engine)
goal 3: provable (IMLL1, intuitionistic, two-sided engine)
no goal is open: `proof` checks the proof
valid proof (intuitionistic)
                      ─────────── ax   ─────────── ax
                      p(a) ⊢ p(a)      q(a) ⊢ q(a)
  ─────────── ax      ──────────────────────────── ⊸L
  p(a) ⊢ p(a)           p(a), p(a) ⊸ q(a) ⊢ q(a)
─────────────── ∀L   ────────────────────────────── ∀L
∀x. p(x) ⊢ p(a)      ∀x. (p(x) ⊸ q(x)), p(a) ⊢ q(a)
─────────────────────────────────────────────────── cut
        ∀x. (p(x) ⊸ q(x)), ∀x. p(x) ⊢ q(a)
      ────────────────────────────────────── ∀R
      ∀x. (p(x) ⊸ q(x)), ∀x. p(x) ⊢ ∀y. q(y)
```

Without exponentials the search decides every sequent, as MLL1 and
MALL1 are decidable. The counts carry over: a predicate one of whose
atoms has a bound variable is counted over all its atoms, since an
instance may be any of them. With exponentials the copy bound and the
three verdicts work as they do without quantifiers, and so does
`--copies none`, which deepens while the time limit lasts: first-order
linear logic with exponentials is undecidable, and a sequent the search
cannot refute ends at the time limit. Two bounds of their own cut a
branch where the search would go further, and a search that cut one is
unknown rather than unprovable: `--witness-depth N` (64), the deepest
term a witness is filled in with, and `--instances N` (65536), the most
instances of quantified formulas the search holds at once. `--stats`
adds two counters on a first-order sequent, the unifications of two
literals tried and the bindings of a witness made (in JSON `unifications`
and `bindings`, written only where they are not zero). The memo and
the loop check work on a first-order sequent by its instances: a
failure is remembered for every renaming of the witnesses it leaves
open, and a sequent met again on its own branch as the same instance is
a loop, so a hypothesis under `!` that leads back to where it started is
refuted rather than cut at the copy bound (`--memo-limit 0` switches
both off). On several threads (`--jobs`, and the pool that joins one
thread by default) a first-order sequent is searched as a propositional
one is, the choices nearest the root split among the threads, where no
rule around a choice may ask its premise for another witness:

<!-- readme-check: machine -->
```console
$ linlog prove -q -j 4 "p(a1), p(a2), p(a3), p(a4), q(b) |- (exists x. (p(x) * q(x))) * (exists x. p(x)) * (exists x. p(x)) * (exists x. p(x))"; echo "exit $?"
unprovable (MLL1, classical, focus engine): the search was exhaustive
exit 1
```

The proof
nets refuse a quantifier yet, saying so with the exit status 2, the
error's, and a first-order proof's Rocq certificate is written for
linlog's library ([Rocq certificates](rocq.md)). The verdicts and the bounds:

<!-- readme-check: machine -->
```console
$ linlog prove "forall x. p(x), p(a) |- p(b)"; echo "exit $?"
unprovable (MLL1, classical, focus engine): ~p(…) occurs 1 more time than p(…) in the one-sided sequent, every atom of the predicate counted, since one has a bound variable, so they cannot all meet in axioms
exit 1
$ linlog prove --copies 2 "p(z), !forall x. (p(x) -o p(s(x))) |- p(s(s(z)))" | head -1
provable (MELL1, classical, focus engine)
$ linlog prove -i --copies 6 "!(forall x. (p(x) -o p(x))), !(q -o r), !(r -o q), p(e) |- r"; echo "exit $?"
unprovable (IMELL1, intuitionistic, two-sided engine): the search was exhaustive
exit 1
$ linlog prove --witness-depth 2 "|- exists x. p(x), ~p(f(f(f(a))))" | sed 's/after .*;/after …;/'; echo "exit $?"
unknown (MLL1, classical, focus engine): the witness depth of 2 was reached after …; raise it with --witness-depth N
exit 3
$ linlog prove -q --deterministic --stats "p(a), p(b) & q(c) |- exists x. (p(x) * p(b))"
provable (MALL1, classical, focus engine)
stable sequents visited: 2
from the memo: 0
memo entries at most: 0
splits examined: 6
unifications tried: 5
metavariables bound: 3
units of work: 36
time: 391.45µs
```

In `interact` the quantifier rules are `∀R` and `∃L` (`∀` one-sided),
which open the formula with a fresh eigenvariable, written `a`, `b`, …
(a letter no symbol of the sequent takes), and `∃R` and `∀L` (`∃`
one-sided), which take their witness after the rule's name: a term of
the sequent's function symbols and of the eigenvariables of the `∀R`
and `∃L` steps below the goal, or nothing, which leaves the witness
open as `$1`, `$2`, … (a `$` that no formula holds) for a later axiom to
fill by unification, or `close` by the search. An open witness never
takes an eigenvariable made after it, which would stand in the
conclusion of its own step: the eigenvariable condition, which `apply`
reports by the eigenvariable's name. `undo` of the step that filled a
witness leaves it open again, and `save` keeps the witnesses and what
fills them (the session's JSON at wire level 7, its `instances`, with
each step's mark, so that `undo` in a session resumed from its file
drops what the step made, as in the session saved):

```console
$ linlog interact -i "exists y. forall x. r(x, y) |- forall x. exists y. r(x, y)"
> apply 0 0 ∃L
opened goal 1: 1: ∀x. r(x, a) ⊢ 0: ∀x. (∃y. r(x, y))
> apply 1 0 ∀R
opened goal 2: 0: ∀x. r(x, a) ⊢ 1: ∃y. r(b, y)
> rules 2 0
∀L (with a witness or none)
> apply 2 0 ∀L
opened goal 3: 1: r($1, a) ⊢ 0: ∃y. r(b, y)
> apply 3 0 ∃R a
opened goal 4: 0: r($1, a) ⊢ 1: r(b, a)
> apply 4 0 ax
closed; no goal is open: `proof` checks the proof
> undo
reopened goal 4: 0: r($1, a) ⊢ 1: r(b, a)
> close
goal 4: provable (IMLL1, intuitionistic, two-sided engine)
no goal is open: `proof` checks the proof
> proof
valid proof (intuitionistic)
          ───────────────── ax
          r(b, a) ⊢ r(b, a)
        ───────────────────── ∃R
        r(b, a) ⊢ ∃y. r(b, y)
      ───────────────────────── ∀L
      ∀x. r(x, a) ⊢ ∃y. r(b, y)
   ─────────────────────────────── ∀R
   ∀x. r(x, a) ⊢ ∀x. (∃y. r(x, y))
───────────────────────────────────── ∃L
∃y. (∀x. r(x, y)) ⊢ ∀x. (∃y. r(x, y))
```

The JSON form of a first-order sequent is written at wire level 7: its
`symbols` (each a name with its arity), its first-order terms `fo`
(`{"B": n}` a bound variable, the innermost quantifier's `0`, `{"F":
[symbol, [terms]]}` an application), each atom as `{"P": [symbol,
[terms]]}`, a quantifier as `{"∀": body}` or `{"∃": body}`, and the
quantifiers' variable names in `binders`, one per quantifier in the
order of the terms:

<!-- readme-check: machine -->
```console
$ linlog seq json "forall x. p(x) |- p(a)"
{"version":7,"terms":[{"D":0},{"∃":0},{"V":1}],"roots":[1,2],"atoms":[{"P":[0,[0]]},{"P":[0,[1]]}],"symbols":[["p",1],["a",0]],"fo":[{"B":0},{"F":[1,[]]}],"binders":["x"],"antecedents":1}
```

A proof of a first-order sequent names, for every member under
quantifiers, the terms its quantifiers were opened with: a `∀` step its
eigenvariable, a fresh constant the proof numbers, as `{"∀": [member,
eigenvariable, premise]}`, and a `∃` step its witness, a closed term, as
`{"∃": [member, term, premise]}`. The proof's `instances` list each
member past the sequent's occurrences as `[occurrence, frame]`, each
frame as `[parent, term]` (the innermost quantifier's term first, `null`
past the outermost), and the proof's own terms after the sequent's `fo`
(an eigenvariable `{"E": k}`). `check` checks such a proof, with the
eigenvariable condition: an eigenvariable a `∀` step introduces is
introduced by that step alone and named by witnesses above it alone,
while one no step introduces is a constant outside the signature, a
witness like any other. The derivation writes every
instance with its terms, the eigenvariables named after letters no
symbol takes:

<!-- readme-check: machine -->
```console
$ echo '{"version":7,"sequent":{"terms":[{"D":0},{"∃":0},{"V":1}],"roots":[1,2],"atoms":[{"P":[0,[0]]},{"P":[0,[2]]}],"symbols":[["p",1],["a",0],["f",1]],"fo":[{"B":0},{"F":[1,[]]},{"F":[2,[1]]}],"binders":["x"],"antecedents":1},"nodes":[{"ax":[3,2]},{"∃":[0,2,0]}],"instances":{"members":[[1,0]],"frames":[[null,2]],"terms":[]}}' | linlog check
valid proof of ⊢ ∃x. ~p(x), p(f(a)) (classical)
─────────────────── ax
⊢ p(f(a)), ~p(f(a))
──────────────────── ∃
⊢ ∃x. ~p(x), p(f(a))
```

`eliminate` eliminates the cuts of a first-order proof as it does a
propositional one, in every mode that has the cut. A cut may be on a
formula with quantifiers or on an instance of one, and the key case of
`∀` and `∃` cuts the instance of the `∀` formula's body at the `∃`
step's witness: the witness takes the eigenvariable's place throughout
the `∀` step's premise (its instances and the witnesses above), in a
copy where another cut still reads that premise, a copied `∀` step
taking an eigenvariable of its own. Every step's line writes the
instance it cut with its terms. A copy is counted against
`--node-limit` before it is made. Here `cut.json` proves `∀x. p(x) ⊢
p(a)` through the cut formula `∀x. p(x)`, by a `∀` step against a `∃`
step with the witness `a`:

```console
$ echo '{"version":7,"sequent":{"terms":[{"D":0},{"∃":0},{"V":1},{"V":0},{"∀":3}],"roots":[1,2],"atoms":[{"P":[0,[0]]},{"P":[0,[1]]}],"symbols":[["p",1],["a",0]],"fo":[{"B":0},{"F":[1,[]]}],"binders":["x","x"],"antecedents":1,"cuts":[4]},"nodes":[{"ax":[7,8]},{"∃":[0,2,0]},{"∀":[3,0,1]},{"ax":[9,2]},{"∃":[5,1,3]},{"cut":[3,2,4]}],"mode":"classical","instances":{"members":[[1,0],[4,0],[6,1]],"frames":[[null,2],[null,1]],"terms":[{"E":0}]}}' > cut.json
$ linlog eliminate cut.json
1. the key case of ∀ and ∃ on the cut of ∀x. p(x)
2. the axiom case on the cut of p(a)
cut-free proof of ⊢ ∃x. ~p(x), p(a) (classical) after 2 steps
  ───────────── ax
  ⊢ p(a), ~p(a)
───────────────── ∃
⊢ ∃x. ~p(x), p(a)
$ linlog eliminate -i --steps=1 cut.json
1. the key case of ∀ and ∃ on the cut of ∀x. p(x)
proof of ∀x. p(x) ⊢ p(a) (intuitionistic) with 1 cut left after 1 step: the elimination took the most steps allowed, 1; --steps N allows more
  ─────────── ax
  p(a) ⊢ p(a)
─────────────── ∀L   ─────────── ax
∀x. p(x) ⊢ p(a)      p(a) ⊢ p(a)
──────────────────────────────── cut
        ∀x. p(x) ⊢ p(a)
```
