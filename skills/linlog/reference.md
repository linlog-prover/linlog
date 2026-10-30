# linlog: the reference

Everything the `linlog` command does, with the flags, the formats, the
JSON forms and the exit statuses, then the Rust library, the WebAssembly
bindings and the Rocq library. Every example below is a real call with
its real output (a time a search took varies). `linlog COMMAND --help`
documents every flag; a flag that sets a key of the settings file names
it in parentheses there.

Contents:

1. Running it
2. Writing a sequent
3. Choosing the logic
4. Deciding a sequent: `prove`
5. Why unprovable, why unknown
6. Limits and the search
7. Output formats
8. JSON forms
9. Checking a proof: `check`
10. Step by step: `interact`
11. A sequent alone: `seq`
12. Many sequents in one call
13. Ordinary logic
14. Cyclic linear logic and the Lambek calculus
15. Settings files and styles
16. The Rust library
17. The web bindings
18. Rocq certificates and the Rocq library
19. Mistakes to avoid

## 1. Running it

`cargo install --locked linlog-cli` installs the command `linlog`, and
`nix run github:linlog-prover/linlog -- ARGS` runs
it without installing. The commands are `prove`, `check`, `interact`
and `seq` (with `print`, `json` and `fragment`).

Exit statuses, the same for every command:

| status | `prove` | `check` | `interact` |
|---|---|---|---|
| 0 | provable (`valid` under `--logic`) | valid proof | the session ended with a checked proof |
| 1 | unprovable (`not valid`) | invalid proof | it ended without one |
| 2 | an error: bad input, a flag that does not apply, an engine forced on a sequent it cannot search | an error | the session could not start or read its input |
| 3 | unknown: a time limit, a bound, a limit or Ctrl-C ended the search | a limit gave the check up | |

A batch exits with its worst entry: an error (2) before unknown (3)
before unprovable (1) before proved (0). A reader that closes the pipe
early (`| head -1`) changes no status.

```console
$ linlog prove -q "A |- A"; echo "exit $?"
provable (MLL, classical, focus engine)
exit 0
$ linlog prove --engine net "A & B |- A"; echo "exit $?"
error: proof nets exist for MLL without units only, not for ALL
exit 2
$ echo '{"version":1,"sequent":{"terms":[{"V":0},{"V":1}],"roots":[0,1],"atoms":["A","B"]},"nodes":[{"ax":[0,1]}]}' | linlog check; echo "exit $?"
invalid proof of ⊢ A, B (classical): node 0 (ax on A, B): the literals are not an atom and its negation
exit 1
$ linlog prove --format json "A * B |- B * A" | linlog check --work-limit 1; echo "exit $?"
unknown: the proof of ⊢ ~A ⅋ ~B, B ⊗ A (classical) is not checked: the check was given up at node 1, without a verdict on the proof: the call did the most work allowed, 1 unit; raise it with --work-limit N
exit 3
$ echo quit | linlog interact "A |- A"; echo "exit $?"
exit 1
```

## 2. Writing a sequent

`A, B |- C, D` (or `⊢`); either side may be empty, and a formula alone
is `|- A`. Quote it for the shell.

| connective | ASCII | Unicode |
|---|---|---|
| tensor | `A * B` | `A ⊗ B` |
| par | `A par B`, `A \| B` | `A ⅋ B` |
| with | `A & B` | |
| plus | `A + B` | `A ⊕ B` |
| linear implication | `A -o B` | `A ⊸ B` |
| negation | `~A`, `A^` | |
| exponentials | `!A`, `?A` | |
| units | `1`, `bot`, `top`, `0` | `⊥`, `⊤` |
| quantifiers | `forall x. A`, `exists x. A`, `forall x y. A` | `∀x. A`, `∃x. A` |
| predicates | `p(x, f(a))` | |

Binding, tightest first: `A^`, then `~ ! ?`, then `*`, `|`, `&`, `+`,
and last `-o`, which groups to the right; a quantifier binds loosest
and reaches right as far as it can (to `)`, `,`, `|-` or the end), so
`forall x. p(x) -o q(x)` is `∀x. (p(x) ⊸ q(x))`. Atoms are identifiers,
or predicates applied to terms in brackets: inside them an identifier
bound by an enclosing quantifier is its variable, any other a constant
or, before a bracket, a function symbol; a symbol has one arity
throughout, a name is a predicate or a function symbol, not both, and no
variable is free. A sequent with a quantifier is read, printed and
written as JSON (wire level 7), `check` checks its proofs (section 9),
and `prove` decides it in the classical modes (with `--mix` and
`--affine`), by the focused engine, and with `-i` (and `--affine`) by
the two-sided engine: a witness of `∃` is left open and
filled in by unification where an axiom matches two atoms, the proof
names every witness, and MLL1 and MALL1 are decided, with `!` and `?`
within the copy bound (the memo and the loop check key a sequent by its
instances, so a `!` hypothesis that leads back to where it started is
refuted; `--memo-limit 0` switches them off; `--jobs` splits the
choices nearest the root among threads, where no rule around them may
ask for another witness); `--stats` then adds the unifications tried and
the metavariables bound (JSON `unifications` and `bindings`, written
only where not zero). `interact` proves it step by step with the
quantifier rules (section 10). The net outputs refuse it with an error
(exit status 2), `--format rocq` certifies its proofs in the Rocq
library's first-order development (section 18), while a sequent with
arguments and no quantifier is propositional and decided by every
engine.

```console
$ linlog seq print "forall x. p(x) -o q(x), p(a) |- exists y. q(y)"
⊢ ∃x. (p(x) ⊗ ~q(x)), ~p(a), ∃y. q(y)
$ linlog seq fragment "forall x. p(x) |- p(a)"
MLL1
$ linlog prove "forall x. p(x) |- p(a)"; echo "exit $?"
provable (MLL1, classical, focus engine)
  ───────────── ax
  ⊢ p(a), ~p(a)
───────────────── ∃
⊢ ∃x. ~p(x), p(a)
exit 0
$ linlog prove -q "forall x. exists y. r(x, y) |- exists y. forall x. r(x, y)"; echo "exit $?"
unprovable (MLL1, classical, focus engine): the search was exhaustive
exit 1
$ linlog prove -i "forall x. p(x) |- p(a)"
provable (IMLL1, intuitionistic, two-sided engine)
  ─────────── ax
  p(a) ⊢ p(a)
─────────────── ∀L
∀x. p(x) ⊢ p(a)
```

The search works on the one-sided sequent: `Γ ⊢ Δ` is `⊢ ~Γ, Δ` in
negation normal form, `A -o B` is `~A par B`. `seq print` shows it:

```console
$ linlog seq print "A * B -o C |- A -o B -o C"
⊢ (A ⊗ B) ⊗ ~C, ~A ⅋ (~B ⅋ C)
```

A file given with `--file PATH` (or `-f`, `-` for standard input) is
read by its extension unless `--input-format` says otherwise: `.p` a
problem of the LLTP library (a TPTP problem under `--logic`), `.spec` a
coverability problem in the format of the Mist tool (decided in affine
mode), `.json` a sequent in JSON (section 8), anything else the text
syntax, which may span lines. `--occurrence-limit N` refuses a sequent
with more subformula occurrences, before anything unfolds it
(`--occurrence-limit 50000000` by default, or `none`), and `--term-limit
N` one whose atoms' arguments unfold to more first-order terms
(`50000000` by default).

## 3. Choosing the logic

| flag | logic |
|---|---|
| none | classical linear logic |
| `-i`, `--intuitionistic` | intuitionistic linear logic: hypotheses left of `⊢`, exactly one formula right of it, in its own syntax (`⊗ ⊸ & ⊕ !`, `1 ⊤ 0`); `~`, `^`, `par`, `\|`, `?` and `bot` are refused with what to write instead, unless `--ill-syntax one-sided` reads the classical syntax as the one-sided sequent it stands for |
| `-a`, `--affine` | a hypothesis may go unused (weakening); with `-i` intuitionistic affine logic |
| `--mix` | the Mix rule, which proves `⊢ Γ, Δ` from `⊢ Γ` and `⊢ Δ` |
| `--cyclic` | cyclic linear logic, unit-free MLL in a fixed order (section 14) |
| `--lambek` | the Lambek calculus, `--empty-antecedents` for L* (section 14) |
| `--logic classical\|intuitionistic\|minimal` | ordinary propositional logic, through a translation into linear logic (section 13) |

The fragment is detected (MLL, MLL with units, ALL, MALL, MELL, LL, and
IMLL to ILL in intuitionistic mode) and picks the engine; `seq fragment`
names it:

```console
$ linlog seq fragment "A & B |- 1"
MALL
$ linlog seq fragment -i "A & B |- 1"
IMALL
```

```console
$ linlog prove -i "|- ~A par B"
error: cannot parse the sequent
  |- ~A par B
     ^ unexpected "~"
  intuitionistic linear logic has no negation: write a negated formula on the other side of |- (A |- B, not |- ~A, B), and ~A par B as A -o B; --ill-syntax one-sided reads the classical syntax
$ linlog prove -i "A |- B, C"
error: not an intuitionistic sequent: an intuitionistic sequent has exactly one formula right of ⊢, not 2: write the hypotheses left of ⊢
```

Classical linear logic proves more than intuitionistic linear logic once
`0` is around:

```console
$ linlog prove -q "((A * top) & (B * top)) -o 0 |- (A -o C) + (B -o C)"
provable (MALL, classical, focus engine)
$ linlog prove -i -q "((A * top) & (B * top)) -o 0 |- (A -o C) + (B -o C)"
unprovable (IMALL, intuitionistic, two-sided engine): the search was exhaustive
```

## 4. Deciding a sequent: `prove`

`linlog prove SEQUENT` prints the verdict line and, for a provable
sequent, a derivation that the proof checker has validated. The verdict
line is `provable`, `unprovable` or `unknown`, then in parentheses the
fragment, the mode and the engine, and after a colon the reason.

```console
$ linlog prove "A, A -o B |- B"
provable (MLL, classical, focus engine)
─────── ax   ─────── ax
⊢ ~A, A      ⊢ ~B, B
──────────────────── ⊗
  ⊢ ~A, A ⊗ ~B, B
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
```

A classical derivation is one-sided; `--classical-sides two` draws it
two-sided as written, the rules named by side (`⊗L`, `⅋R`, …) with a
step `~L` or `~R` where a negation was written, a first-order one with
`∀L`, `∀R`, `∃L` and `∃R` by side and its instances written out.
Intuitionistic derivations are always two-sided.

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
```

`-q` (`--quiet`) prints the verdict line alone; `--stats` adds the
search's counters, a line each, and the time; `--no-check` reports a
proof without the checker (by default a proof the checker rejects is an
error, exit status 2). On a terminal the text tree is printed only if it
fits (`--tree always` prints it regardless, `--tree never` leaves it out);
into a file or a pipe it is always written.

The engines (`--engine NAME`; `--engine auto` by default, the one the
fragment and the mode call for):

| engine | searches |
|---|---|
| `focus` | focused sequent search, every classical fragment, with or without units, Mix, exponentials and weakening |
| `two-sided` | the same for every intuitionistic fragment |
| `net` | proof-net search, MLL without units, with or without Mix, and IMLL without units by its embedding |
| `additive` | two additive-only formulas, in every mode |
| `horn` | the markings of a Petri net, for Horn programs (atoms, implications between tensors of atoms, such implications under `!`, one goal that is a tensor of atoms): reachability in linear mode, coverability in affine mode |
| `essential` | essential nets, IMLL without units, only when named |
| `inverse` | the focused inverse method, forward from the axioms, in every mode but the ordered ones, only when named (and in turns beside the default on intuitionistic goals with an exponential and under Mix) |

`--fragment NAME` (`mll`, `mll-units`, `all`, `mall`, `mell`, `ll`)
searches in a fragment instead of the detected one; a sequent outside it
is an error.

## 5. Why unprovable, why unknown

An unprovable verdict gives its reason; the JSON `refutation` has the
same, tagged by `kind`:

| `kind` | the verdict line says |
|---|---|
| `unbalanced` | an atom's literals cannot all meet their duals in axioms |
| `one_sided` | the same, counted outside the exponentials |
| `unbalanced_symbol`, `one_sided_symbol` | the same for a predicate with a bound variable in one of its atoms, every atom of it counted together (`"symbol"` its index) |
| `equation` | the count equation of the multiplicatives fails |
| `state_equation` | a Horn program's state equation has no solution: weights for the atoms that no clause raises and the goal needs raised |
| `classical` | the sequent is false read classically, with the atoms' values: no provable sequent is |
| `saturated` | the inverse method's saturated set, closed under the rules, lacks the goal |
| `exhausted` | the search was exhaustive |

```console
$ linlog prove "|- A par B, ~A, ~B"
unprovable (MLL, classical, focus engine): the count equation fails: a provable one-sided sequent of MLL has exactly #⊗ − #⅋ − #1 + #⊥ + 2 formulas, here 0 − 1 − 0 + 0 + 2 = 1, and this one has 3
$ linlog prove -q "!(A -o A * A), !(A -o B), !(B * B -o C), A |- C * ?D"
unprovable (MELL, classical, focus engine): the classical reading is false where A, B and C are true and every other atom false (! and ? erased, ⊗ and & as and, ⅋ and ⊕ as or, 1 and ⊤ true, ⊥ and 0 false), and every provable sequent reads as a tautology
$ linlog prove -q "!(a -o a * a), !(b * b -o c), a, b |- c"
unprovable (MELL, classical, horn engine): the state equation of the Petri net has no solution, so no firing of its clauses yields the goal's atoms: weighting each a by −2, b by 1 and c by 2, no clause that can fire raises the weighted count of the atoms, and the goal asks it raised
```

The classical refutation runs after a search that did not prove the
sequent, within `--classical-work N` formulas evaluated
(`--classical-work 67108864` by default, `0` switches it off): it turns an unknown search into
unprovable and never changes a verdict the search gave.

Unknown means a limit ended the search before it decided; it is not
unprovable. MELL has no known decision procedure and full linear logic
is undecidable, so with `!` or `?` the search deepens a bound on how
often a `?` formula is copied on one branch. The line says which limit,
after how long, at which copy bound, and which flag to raise; the JSON
`reason` is tagged by `kind` (`stopped`, `copy_bound`, `memory_limit`,
`work_limit`, `recursion_limit`, `index_limit`, `unchecked`, and of a
first-order search `witness_depth` and `instances`):

```console
$ linlog prove -q --copies 1 "!(A & B) |- A * B"
unknown (LL, classical, focus engine): the copy bound of 1 was reached after 3.56ms; raise it with --copies N, or lift it with --copies none to deepen it while the time limit lasts
$ linlog prove --memory-limit 100 "|- (a & b) + (a & c), ~a par (~b & ~c)"
unknown (MALL, classical, focus engine): the memory limit of 100 B was reached after 32.28µs; raise it with --memory-limit SIZE
```

A time limit's line says at which copy bound the search was:

<!-- readme-check: machine -->
```console
$ linlog prove -q --timeout 1s --classical-work 0 "!(A -o A * A), !(A -o B), !(B * B -o C), A |- C * ?D"; echo "exit $?"
unknown (MELL, classical, focus engine): the time limit of 1s was reached at a copy bound of 46; --timeout DURATION gives the search longer
exit 3
```

## 6. Limits and the search

| flag | default | what it bounds |
|---|---|---|
| `--timeout DURATION` | `2s` | the whole command, reading included (`500ms`, `10s`, `2m`, `1h`, `none`) |
| `--copies N` | `none` | how often a `?` formula may be copied on one branch; by default the bound deepens until the search decides or the time is up |
| `--forward-copies N` | `30` | the forward search's copy bound on a Horn program, under `--copies` |
| `--memory-limit SIZE` | `1GiB` | the search's memory (`512MiB`, `4GiB`, `none`) |
| `--work-limit N` | `none` | units of work: the same verdict on every machine |
| `--recursion-limit N` | `2048` | the nesting of rules on one branch |
| `--memo-limit N` | `1048576` | the decided sequents remembered at once |
| `--classical-work N` | `67108864` | the classical refutation after the search |
| `--witness-depth N` | `64` | the deepest term a first-order search fills a witness in with |
| `--instances N` | `65536` | the instances of quantified formulas a first-order search holds at once |
| `--derivation-limit SIZE` | `64MiB` | the derivation built from a proof; above it the verdict stands without it |
| `--occurrence-limit N` | `50000000` | the sequent's subformula occurrences |
| `--term-limit N` | `50000000` | the first-order terms the sequent's atoms unfold to, and a proof's instance table |

Threads: one thread searches first, and after `--pool-after 100ms` by
default the machine's other threads join it; `-j N` (`--jobs`) uses N threads from
the start. `--deterministic` runs one thread throughout, so that the
proof and the statistics are a function of the input; a script that
must get the same answer everywhere names `--copies` or `--work-limit`,
or lifts the time limit with `--timeout none`.

`--bias auto|rarer|factors` chooses how the focused engines pick each
atom's positive literal, which changes the search and never what is
provable.

```console
$ linlog prove -q --deterministic --stats "!(a * a -o b), !(b * b -o c), !(c * c -o d), a, a, a, a, a, a, a, a |- d"
provable (MELL, classical, horn engine)
markings reached: 7
markings met again: 0
markings kept: 7
units of work: 6
time: 45.06µs
```

## 7. Output formats

`--format text|json|latex|typst|svg|png|pdf|rocq`, or `--output FILE`
(`-o`) whose extension names it (`.txt`, `.json`, `.tex`, `.typ`,
`.svg`, `.png`, `.pdf`, `.v`). A file is made only with a derivation or
a net in it, or in JSON always; for an unprovable sequent the verdict
goes to standard error and no file is made. PNG and PDF are never
written to a terminal.

- `latex`: the verdict as a comment, then a proof tree for the ebproof
  package with the connectives of cmll and amssymb.
- `typst`: a proof tree for the curryst package, or for a high tree in
  linlog's own layout (`--style layout=linlog` asks for it always).
- `svg`, `png`, `pdf`: the derivation drawn, set in Euler Math unless
  `--font stix-two|garamond|asana|new-computer-modern` names another.
- `rocq`: a certificate (section 18).
- `--standalone` writes a document that compiles on its own (latex,
  typst, rocq); `--no-verdict` leaves the verdict line out.
- `--compact always|auto|never` draws a run of one structural rule as
  one inference with a starred label (`?w*`).

```console
$ linlog prove -i --format latex "A, A -o B |- B"
% provable (IMLL, intuitionistic, two-sided engine)
\begin{prooftree}
\infer0[$\mathrm{ax}$]{A &\vdash A}
\infer0[$\mathrm{ax}$]{B &\vdash B}
\infer2[$\multimap\mathrm{L}$]{A, A \multimap B &\vdash B}
\end{prooftree}
$ linlog prove --output proof.svg "A, A -o B |- B"
$ linlog prove -i --output proof.pdf "1, A & B, B -o C |- C"
provable (IMALL, intuitionistic, two-sided engine)
```

`--net` writes the proof net instead of the derivation, for MLL without
units (with or without Mix) and with `!`-boxes for MELL without units,
classical or intuitionistic: in text the sequent, the axiom links as
pairs of literals with their positions, and the criterion's verdict; in
`json` the net's document; drawn in `svg`, `png` or `pdf`.

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

## 8. JSON forms

Every document carries `version`, its wire level. Atoms are named in
`atoms` and referred to by index; `{"V": i}` is atom `i`, `{"D": i}` its
dual; every other term is an object of one key naming earlier terms by
index (`{"⊗": [0, 1]}`, `⅋`, `&`, `⊕`, `{"!": 2}`, `{"?": 2}`), a unit
its symbol (`"1"`, `"⊥"`, `"⊤"`, `"0"`). `roots` are the formulas in
order, and `antecedents` how many of them stood left of `⊢`.

```console
$ linlog seq json "A |- A"
{"version":1,"terms":[{"D":0},{"V":0}],"roots":[0,1],"atoms":["A"],"antecedents":1}
$ linlog seq json -i "A * B -o C |- A -o B -o C"
{"version":5,"terms":[{"V":0},{"V":1},{"⊗":[0,1]},{"V":2},{"⊸":[2,3]},{"⊸":[1,3]},{"⊸":[0,5]}],"atoms":["A","B","C"],"hypotheses":[4],"goal":6}
```

`prove --format json` writes the outcome: `verdict` (`proved`,
`unprovable`, `unknown`), `checked` for a proof, `refutation` for an
unprovable sequent and `reason` for an unknown one (section 5),
`fragment`, `mode`, `engine`, `statistics` (the counters by name), and
for a proof its `sequent` and `nodes`, so that `check` reads the outcome
as a proof. A node is one rule instance, premises before conclusions and
the root last: `{"ax": [x, y]}` on two literals, `{"⊗": [o, l, r]}` on
an occurrence with two premises' indices, `{"⅋": [o, p]}` with one. An
occurrence is a subformula's number in the sequent's numbering. A proof
with a cut (from `interact`'s `cut`) is at level 6: its sequent's
`cuts` name the cut formulas, numbered after the sequent's own, and a
cut is `{"cut": [a, l, r]}`.

```console
$ linlog prove --format json "A |- A"
{"version":1,"linlog":"…","verdict":"proved","checked":true,"fragment":"MLL","mode":"classical","engine":"focus","statistics":{"nodes":1,"memo_hits":0,"memo_entries":1,"splits":0,"links":0,"tests":0,"copies":0,"forward_copies":0,"work":16},"sequent":{"terms":[{"D":0},{"V":0}],"roots":[0,1],"atoms":["A"],"antecedents":1},"nodes":[{"ax":[0,1]}]}
$ linlog prove --format json "A |- B"
{"version":1,"linlog":"…","verdict":"unprovable","refutation":{"kind":"unbalanced","atom":0,"least":-1,"most":-1},"fragment":"MLL","mode":"classical","engine":"focus","statistics":{"nodes":1,"memo_hits":0,"memo_entries":1,"splits":0,"links":0,"tests":0,"copies":0,"forward_copies":0,"work":16},"sequent":{"terms":[{"D":0},{"V":1}],"roots":[0,1],"atoms":["A","B"],"antecedents":1}}
```

An error of the library, in a batch record, `check`'s JSON or the web
bindings, is `{"code", "kind", "message", "setting", "details"}`:
`code` is the stable reason to branch on, `setting` the settings key
that lifts the bound that refused the call, where one did.

## 9. Checking a proof: `check`

`linlog check FILE` (or standard input) reads a proof as `prove --format
json` writes it and checks it independently of the search. The mode
comes from the flags, never from the file: pass the flags the proof was
found with. Without `-i`, a proof that `prove -i` found is checked as a
proof of its classical one-sided sequent, which it also is. `--format json` writes `{"version", "verdict", "mode"}`,
the verdict `valid`, `invalid` or `unknown`, with the library's `error`
for the last two. Its time limit is `--timeout 2s` by default, and `--work-limit`
and `--memory-limit` bound it too; a check a limit gives up is unknown,
exit status 3. It takes `prove`'s output formats and `--net`.

```console
$ linlog prove --format json "A |- A" | linlog check --quiet
valid proof of ⊢ ~A, A (classical)
$ linlog prove --format json "A |- A" | linlog check --format json
{"version":1,"verdict":"valid","mode":"classical"}
```

A proof of a first-order sequent (wire level 7) adds the steps
`{"∀": [member, eigenvariable, premise]}` and `{"∃": [member, term,
premise]}` and the key `instances`: `members`, each `[occurrence,
frame]` for a member past the sequent's occurrences (an occurrence
under quantifiers with the terms they were opened with), `frames`, each
`[parent, term]` with the innermost quantifier's term first and `null`
past the outermost, and `terms`, the proof's own terms after the
sequent's `fo`, an eigenvariable as `{"E": k}`. `check` holds such a
proof to the eigenvariable condition: an eigenvariable a `∀` step
introduces is introduced by that step alone and named by witnesses above
it alone (one no step introduces is a constant outside the signature);
the error names the one that is not (`eigenvariable a occurs outside the ∀ step that
introduces it`).

```console
$ echo '{"version":7,"sequent":{"terms":[{"D":0},{"∃":0},{"V":1}],"roots":[1,2],"atoms":[{"P":[0,[0]]},{"P":[0,[2]]}],"symbols":[["p",1],["a",0],["f",1]],"fo":[{"B":0},{"F":[1,[]]},{"F":[2,[1]]}],"binders":["x"],"antecedents":1},"nodes":[{"ax":[3,2]},{"∃":[0,2,0]}],"instances":{"members":[[1,0]],"frames":[[null,2]],"terms":[]}}' | linlog check --quiet
valid proof of ⊢ ∃x. ~p(x), p(f(a)) (classical)
```

## 10. Step by step: `interact`

`linlog interact SEQUENT` (or `--file`; standard input carries the
commands) starts a session with the sequent as goal 0 and reads one
command per line. Formulas are named by their position in a goal, from
0, as `goals` lists them. A step closes its goal and opens its premises
as new goals with new numbers, each listed (`opened goal 1: ⊢ 0: ~A, 1:
A`) with its own positions; a goal it closes at once is `closed`. In
`apply G P RULE [P…]` every position is one of goal G before the step:
for `⊗` the further positions are G's other formulas that go to the
left premise, the rest going to the right; `ax` takes either of its two
literals. A command's error is printed as `error: …` and the session
goes on, so a script that misnumbers a goal goes on failing: read the
numbers from the output before writing the next step.

| session command | what it does |
|---|---|
| `goals` | the open goals, with the positions of their formulas |
| `rules G P` | the rules that act on formula P of goal G |
| `apply G P RULE [P…]` | applies a rule; for `⊗` or Mix the further positions go to the left premise |
| `apply G P RULE [T]` | `∃R` or `∀L` (`exists` one-sided) with the witness T, or open without one |
| `cut G C [P…]` | cuts goal G on the formula C (quoted if it has blanks): the positions P go to the first premise, which proves C, the others to the second, which uses it |
| `undo` | retracts the last step |
| `invertible on\|off` | applies every invertible rule by itself, or stops doing so |
| `close [G]` | lets the search close goal G, or every open goal |
| `show [--FORMAT] [FILE]` | the derivation so far, open goals included: text, or `--latex`, `--typst`, `--svg`, `--png`, `--pdf` (these two need a FILE) |
| `proof [--FORMAT] [FILE]` | checks the finished proof and prints it or writes it: text, `--json`, `--latex`, `--typst`, `--svg`, `--png`, `--pdf` or `--rocq`; a FILE without a format takes its extension's, else JSON |
| `save FILE` | writes the session as JSON, which `load` and `--state` read and `check` does not |
| `load FILE` | resumes a session of the same sequent and mode |
| `help` | the list |
| `quit` | ends the session |

A RULE is its usual name or its ASCII spelling: `ax`, `1`, `bot`, `top`,
`*`, `par`, `&`, `+1`, `+2`, `?d`, `?c`, `?w`, `!`, `mix`, `wk`, and in
intuitionistic mode `-oL`, `-oR`, `*L`, `*R`, `&L1`, `&L2`, `&R`, `+L`,
`+R1`, `+R2`, `1L`, `1R`, `0L`, `topR`, `!L`, `!R`, `!c`, `!w`, and for
the quantifiers `forall`, `exists` (one-sided) and `forallR`, `existsL`,
`existsR`, `forallL`; `rules` prints the names it applies. `--state
FILE` resumes a saved session (the mode is the file's), `--invertible`
applies the invertible rules from the start, and the search flags of
`prove` apply to `close`.

With quantifiers, `∀R` and `∃L` (`forall` one-sided) open the formula
with a fresh eigenvariable, written `a`, `b`, … (a letter no symbol
takes). `∃R` and `∀L` (`exists` one-sided) take the witness after the
rule's name, a term of the sequent's function symbols and of the
eigenvariables of the `∀R`/`∃L` steps below the goal; without one the
witness stays open, written `$1`, `$2`, …, and the axiom fills it by
unification, or `close` by the search. An open witness never takes an
eigenvariable made after it (`error: the eigenvariable b cannot stand
there: …`, the eigenvariable condition); `undo` of the step that filled
it opens it again; `save` keeps it (the session's `instances`, wire
level 7). Such a session takes no `cut`.

```console
$ printf 'apply 0 0 forallL\napply 1 1 ax\nproof\n' | linlog interact -i "forall x. p(x) |- p(f(a))"
opened goal 1: 1: p($1) ⊢ 0: p(f(a))
closed; no goal is open: `proof` checks the proof
valid proof (intuitionistic)
───────────────── ax
p(f(a)) ⊢ p(f(a))
────────────────── ∀L
∀x. p(x) ⊢ p(f(a))
```

`cut G C [P…]` is the one step not on a formula of the goal: C is
written as the mode writes formulas (intuitionistic in `-i` mode, as the
sequent is), and of a goal `Γ, Δ ⊢ D` it leaves `Γ ⊢ C` (the formulas
at P) and `Δ, C ⊢ D`; `undo` takes it back with its formula. The search
never makes a cut, `close` proves each premise without one, and the
finished proof keeps the cut (its JSON is at wire level 6, which `check`
reads).

```console
$ printf 'goals\nrules 0 2\napply 0 2 * 0\nclose\nproof\n' | linlog interact "A, B |- A * B"
goal 0: ⊢ 0: ~A, 1: ~B, 2: A ⊗ B
⊗ (with a split)
opened goal 1: ⊢ 0: ~A, 1: A
opened goal 2: ⊢ 0: ~B, 1: B
goal 1: provable (MLL, classical, focus engine)
goal 2: provable (MLL, classical, focus engine)
no goal is open: `proof` checks the proof
valid proof (classical)
─────── ax   ─────── ax
⊢ ~A, A      ⊢ ~B, B
──────────────────── ⊗
  ⊢ ~A, ~B, A ⊗ B
$ printf 'apply 0 1 -oL 0\napply 1 0 ax\napply 2 0 ax\nproof --json p.json\n' | linlog interact -i "A, A -o B |- B"
opened goal 1: 0: A ⊢ 1: A
opened goal 2: 0: B ⊢ 1: B
closed
closed; no goal is open: `proof` checks the proof
valid proof written to p.json
$ linlog check -i -q p.json
valid proof of A, A ⊸ B ⊢ B (intuitionistic)
$ printf 'cut 0 B 0 1\nclose\n' | linlog interact -i "A, A -o B, B -o C |- C"
opened goal 1: 0: A, 1: A ⊸ B ⊢ 2: B
opened goal 2: 0: B ⊸ C, 2: B ⊢ 1: C
goal 1: provable (IMLL, intuitionistic, two-sided engine)
goal 2: provable (IMLL, intuitionistic, two-sided engine)
no goal is open: `proof` checks the proof
```

`linlog eliminate FILE` eliminates the cuts of a proof file step by
step: it checks the proof in the flags' mode (`check`'s flags and
formats), reduces the cut with no cut above it, prints each step's case
(`the axiom case`, `the key case of ⊗ and ⅋`, `of 1 and ⊥`, `of & and
⊕`, `of ∀ and ∃`, `a commutative case`, `weakening`, `dereliction`,
`contraction`, `a box entering a box`) and cut formula, checks every
proof reached, and draws the last. A first-order proof is eliminated
alike, its cuts on quantified formulas or on instances: the key case
of ∀ and ∃ puts the ∃ step's witness for the ∀ step's eigenvariable
throughout the ∀ premise (copied where another cut still reads it, the
copy's own ∀ steps given fresh eigenvariables, the copy counted against
`--node-limit` before it is made), and each step's line writes the
instance it cut. `--steps N` (`cut.steps`) bounds the steps, 10 000 by
default, and `--node-limit N` (`cut.nodes`) the nodes of a proof a step
makes, 100 000; a bound prints the proof reached with its cuts. One
step is `--steps 1`. Exit 0 when no cut is
left or the steps asked for with `--steps` were taken, 1 for an invalid
proof, 3 when the default bound on steps, another bound or the time
limit stopped it with cuts left. `--format json` writes `{"version",
"proof", "steps": [{"cut", "formula", "case", "above"}], "end"}`, `end`
`cut_free` or `refused` with `reason`, at its proof's wire level (6 with
cuts, 7 at first order).

`--net` eliminates on the proof's net instead (for the sequents `--net`
takes): the cuts are cut links, each step reduces the first cut link
(`the axiom case`, `the key case of ⊗ and ⅋`, and on boxes `weakening`,
`dereliction`, `contraction`, `a box entering a box`; no commutative
case), every net reached passes the criterion, and the net reached is
printed as text, a cut link as `cut A[x] — A⊥[y]`; `--node-limit`
counts a net's vertices. `--format svg|png|pdf --output FILE` writes
`FILE-k` (the net step k was taken on, its cut link and what it acts
on highlighted) and `FILE` (the net reached), each within
`--derivation-limit`, the steps' files 256 MiB together at most
(`--drawings-limit SIZE`, `cut.drawn_bytes`; past it the steps go on
undrawn and standard error says from which); `--format json` writes
`{"version", "net", "steps": [{"cut": [x, y], "case", "above"}],
"end"}`.

```console
$ printf 'cut 0 !a 0\nclose\nproof c.json\n' | linlog interact -i "!a |- a * a" > /dev/null
$ linlog eliminate -q -i --steps 2 c.json
1. contraction on the cut of !a
2. dereliction on the cut of !a
proof of !a ⊢ a ⊗ a (intuitionistic) with 2 cuts left after 2 steps: the elimination took the most steps allowed, 2; --steps N allows more
$ linlog eliminate -q -i c.json
1. contraction on the cut of !a
2. dereliction on the cut of !a
3. a commutative case on the cut of a
4. the axiom case on the cut of a
5. a commutative case on the cut of a
6. the axiom case on the cut of a
cut-free proof of !a ⊢ a ⊗ a (intuitionistic) after 6 steps
$ linlog eliminate -q -i --net c.json
1. contraction on the cut of !a
2. dereliction on the cut of !a
3. the axiom case on the cut of a
4. dereliction on the cut of !a
5. the axiom case on the cut of a
cut-free proof net of !a ⊢ a ⊗ a (intuitionistic) after 5 steps
$ echo '{"version":7,"sequent":{"terms":[{"D":0},{"∃":0},{"V":1},{"V":0},{"∀":3}],"roots":[1,2],"atoms":[{"P":[0,[0]]},{"P":[0,[1]]}],"symbols":[["p",1],["a",0]],"fo":[{"B":0},{"F":[1,[]]}],"binders":["x","x"],"antecedents":1,"cuts":[4]},"nodes":[{"ax":[7,8]},{"∃":[0,2,0]},{"∀":[3,0,1]},{"ax":[9,2]},{"∃":[5,1,3]},{"cut":[3,2,4]}],"mode":"classical","instances":{"members":[[1,0],[4,0],[6,1]],"frames":[[null,2],[null,1]],"terms":[{"E":0}]}}' > cut.json
$ linlog eliminate -q -i cut.json
1. the key case of ∀ and ∃ on the cut of ∀x. p(x)
2. the axiom case on the cut of p(a)
cut-free proof of ∀x. p(x) ⊢ p(a) (intuitionistic) after 2 steps
```

## 11. A sequent alone: `seq`

- `seq print` prints the sequent one-sided in negation normal form, or
  with `-i` two-sided as intuitionistic linear logic reads it, under
  `--logic` the image of the translation; `--format text|latex|typst|svg|png|pdf`.
- `seq json` writes its JSON (section 8), with `-i` the intuitionistic
  sequent's own form; `--optimize` merges equal subformulas of a JSON
  input.
- `seq fragment` names its fragment.

```console
$ linlog seq print -i "A * B -o C |- A -o B -o C"
(A ⊗ B) ⊸ C ⊢ A ⊸ (B ⊸ C)
$ linlog seq print --format typst "A * B -o C |- A -o B -o C"
$⊢ (A ⊗ B) ⊗ C^⊥, A^⊥ ⅋ (B^⊥ ⅋ C)$
```

## 12. Many sequents in one call

`prove` decides a batch when given `--file` more than once, a directory
(walked in sorted order for `.p` and `.json` files), `--files-from LIST`
(one path per line, `--null` for NUL-separated, `-` for standard input),
or an input format of many sequents:

- `--input-format lines`: `NAME: SEQUENT` or `SEQUENT` per line; blank
  lines and everything from `#` on are skipped;
- `jsonl`: a sequent as `seq json` writes it, or `{"name", "mode",
  "sequent"}` with the sequent as JSON or text, per line;
- `problems`: `NAME; MODE; EXPECTED; COPIES; SEQUENT` per line.

Every sequent gets one line, `NAME: VERDICT`, in input order and as soon
as it is decided, named by its path, its line's name, or `FILE:LINE`.
With `--format json` each is a JSON line, `version` and `name` first and
then the outcome's keys, or `"verdict": "error"` with a `message`. A
drawn format writes a file per proved sequent into `--output DIR`. The
exit status is the worst. On standard input the batch is a stream,
answered line by line.

```console
$ printf 'identity: A |- A\nmp: A, A -o B |- B\nA |- B\n' > sequents.txt
$ linlog prove --input-format lines --file sequents.txt
identity: provable (MLL, classical, focus engine)
mp: provable (MLL, classical, focus engine)
sequents.txt:3: unprovable (MLL, classical, focus engine): ~A occurs 1 more time than A in the one-sided sequent, so they cannot all meet in axioms
$ echo '{"name": "pair", "mode": "intuitionistic", "sequent": "A, B |- A * B"}' | linlog prove --input-format jsonl --format json
{"version":1,"name":"pair","linlog":"…","verdict":"proved","checked":true,"fragment":"IMLL","mode":"intuitionistic","engine":"two-sided","statistics":{"nodes":1,"memo_hits":0,"memo_entries":1,"splits":1,"links":0,"tests":0,"copies":0,"forward_copies":0,"work":16},"sequent":{"terms":[{"D":0},{"D":1},{"V":0},{"V":1},{"⊗":[2,3]}],"roots":[0,1,4],"atoms":["A","B"],"antecedents":2},"nodes":[{"ax":[3,0]},{"ax":[4,1]},{"⊗":[2,0,1]}]}
```

The modes a record or a problem names: `classical`, `mix`, `affine`,
`affine-mix`, `intuitionistic`, `intuitionistic-affine`, `cyclic`,
`lambek`, `lambek-star`. `--timeout` is each sequent's limit and
`--batch-timeout` the whole batch's; `--cores across|within`,
`--workers N`, `--batch-memory SIZE` and `--isolate` (a child process
per sequent) choose how it runs. By default the cores go across the
sequents, one thread each. What that leaves unknown is worth a second
pass with `--cores within`.

## 13. Ordinary logic

`--logic classical`, `intuitionistic` or `minimal` reads a formula or a
sequent of propositional logic: `->`/`→`, `/\`/`∧`, `\/`/`∨`, `~`/`¬`,
`<->`/`↔`, `true`/`⊤`, `false`/`⊥` (`~` binds tightest, then `/\`, `\/`,
`->`, and last `<->`; `->` and `<->` group to the right). The verdict is
`valid`, `not valid` or `unknown`, and the proof is read back as a
derivation of LK or LJ, checked before it is shown. Classical logic goes
into affine MALL (no copy bound); intuitionistic and minimal logic into
ILL by `--translation cbn` (the default), `cbv` or `01`, under the time
limit, so they may be answered unknown. `--linear` shows the linear
proof of the image, and `seq print --logic` the image. A `.p` file is a
TPTP problem.

```console
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
$ linlog prove --logic classical -q '((a -> b) -> a) -> a'
valid (classical logic by the affine translation into affine MALL, focus engine)
$ linlog seq print --logic intuitionistic --translation cbv 'a, a -> b |- b'
!a, !(!a ⊸ !b) ⊢ !b
```

`-i`, `-a` and `--mix` are refused beside `--logic`: the translation
decides the mode.

## 14. Cyclic linear logic and the Lambek calculus

`--cyclic` decides unit-free MLL whose formulas keep their order up to
rotation: `~(A * B)` is `~B par ~A`, the formulas left of `|-` come in
the reverse order, each negated, so `A, A -o B |- B` holds and `A -o B,
A |- B` does not. `--lambek` decides the Lambek calculus: intuitionistic,
in the order written, with `A \ B` and `B / A` beside `*`; every sequent
of a proof has a hypothesis unless `--empty-antecedents` is given. Units,
additives, exponentials, Mix, weakening and Rocq certificates are refused
in these modes.

```console
$ linlog prove --cyclic "a * b |- b * a"
unprovable (MLL, cyclic, net engine): the search was exhaustive
$ linlog prove --lambek -q "c / b, a, a \ b |- c"
provable (IMLL, Lambek, essential engine)
```

## 15. Settings files and styles

`--settings FILE` reads every setting the flags set from one JSON object
with any of `clock`, `limits`, `search`, `view`, `styles`, `batch`,
`ordinary`, `input` and `cut`; each flag's help names its key in parentheses
(`--copies` is `search.copies`, `--timeout` `clock.time_limit_ms`,
`--memory-limit` `limits.memory_bytes`). A key left out keeps the
default, a misspelt one is an error, and a flag beside the file
overrides it.

```console
$ echo '{"search": {"copies": 0, "engine": "focus"}}' > settings.json
$ linlog prove --settings settings.json --copies 3 "|- ?(~a * ~a), a * a * a * a"
unprovable (MELL, classical, focus engine): the search was exhaustive
```

`--style KEY=VALUE` (repeatable) sets one option of an output format,
prefixed `text.`, `latex.`, `typst.`, `svg.`, `png.`, `pdf.` or `rocq.`
for another format than `--format`'s; `--style-file FILE` reads them all
as JSON, an object per format. `prove --help` lists every field.

```console
$ linlog prove --style gap=6 --style bar="=" "A * B |- A * B"
provable (MLL, classical, focus engine)
======= ax      ======= ax
⊢ ~A, A         ⊢ ~B, B
======================= ⊗
    ⊢ ~A, ~B, A ⊗ B
    ================ ⅋
    ⊢ ~A ⅋ ~B, A ⊗ B
```

## 16. The Rust library

The crate `linlog` on crates.io holds all of it; its default
features are `parse`, `serialize`, `interactive`, `latex`, `typst`, `svg`
and `rocq`, and `parallel`, `png` and `pdf` are off. Its documentation
is at <https://linlog-prover.github.io/linlog/linlog/> for `main` and on
docs.rs for the releases: the text syntax on `Sequent`, every JSON form
on its type (`Sequent`, `Proof`, `Outcome`, `ProofStructure`,
`Interactive`), each engine on `Engine`.

```rust
use linlog::{Fragment, Mode, Options, Sequent, Verdict, prove};

// Two-sided input becomes one-sided: ⊢ ~A, A ⊗ ~B, B.
let sequent: Sequent = "A, A -o B |- B".parse()?;
assert_eq!(sequent.fragment(), Fragment::MLL);

let outcome = prove(&sequent, Mode::CLASSICAL, &Options::default())?;
let Verdict::Proved(proof) = &outcome.verdict else {
    panic!("the sequent is provable");
};
assert_eq!(
    proof.derivation()?.to_string(),
    "─────── ax   ─────── ax\n\
     ⊢ ~A, A      ⊢ ~B, B\n\
     ──────────────────── ⊗\n\
    \x20 ⊢ ~A, A ⊗ ~B, B"
);

let json = serde_json::to_string(&proof)?;
let back: linlog::Proof = serde_json::from_str(&json)?;
back.check(Mode::CLASSICAL)?;
```

`Options::default()` bounds the copies of a `?` formula at 3, since a
library call without a stop must end; the command deepens without a
bound under its time limit. `prove_within(&sequent, mode, &options,
&limits, stop)` takes `Limits` and a stop closure (the crate reads no
clock), and `Settings::default()` is the command's behaviour as data.
Every error is `linlog::Error`, whose `code()` is the stable reason.
`proofs::cut::step(&proof, mode, at, &options, &limits, stop)` reduces
one cut (the node `at`, or the strategy's with `None`) and returns the
checked proof and the `Step` (`cut`, `formula`, `case`);
`proofs::cut::eliminate(&proof, mode, &options, &limits, stop, each)`
runs to the end within `cut::Options` (`each` sees every step and the
proof it was taken on) and returns the proof reached, the steps and the
`End`; at first order a `Step`'s `formula` is a member of the proof the
step was taken on, whose `Proof::formula` writes it as the instance it
is. On nets,
`nets::cut::step(&net, at, &options, &limits, stop)` (a `ProofStructure`
or a `MellStructure`, `at` a vertex of a cut link) and
`nets::cut::eliminate(&net, &options, &limits, stop, each)` (`each` sees
every step and the net it was taken on); `svg::net_step` and
`svg::mell_net_step` draw a step, the cut link `c<x>-<y>`.

## 17. The web bindings

The page at <https://linlog-prover.github.io/> is the prover in a
browser, for people: a sequent proved step by step with the mouse. Its
engine is the unpublished crate `linlog-web` (`web/` of the repository,
`nix build github:linlog-prover/linlog#linlog-web` builds the module and
its JavaScript): the library compiled to WebAssembly behind calls of
JSON text in and JSON, SVG or bytes out, which keep nothing between
calls. The calls are `about`, `defaults`, `read`, `read_as`, `start`,
`view`, `rules`, `split_passes`, `apply`, `cut`, `invert_all`, `undo`, `close`,
`fill`, `close_all`, `hint`, `proof`, `prove`, `draw`, `draw_net`,
`cuts`, `cut_step`, `eliminate`, `net_cuts`, `net_cut_step`,
`net_eliminate` (cut elimination step by step and to the end, on terms
and on nets), `export`, `render` and `certificate`; each takes the page's settings,
the JSON of the library's `Settings` laid over a browser tab's defaults,
and a failed call throws the error's JSON (section 8). An agent with a
shell uses the command instead.

## 18. Rocq certificates and the Rocq library

A Rocq certificate (`--format rocq`) is written for the kernel
`--kernel` names. The kernel `nanoyalla` is the default for classical
linear logic: NanoYalla, the kernel of Click & coLLecT at the version
`prove --help` names, a proof script checked rule by rule, which
refuses Mix and affine weakening. The kernel `linlog` is the default for
Mix, affine and intuitionistic logic and for a sequent with a
quantifier: the Rocq library in `rocq/` of the repository, a proof term
its checker accepts. A first-order proof is stated in the library's
first-order development (`ll (symbol := string) Calculus.classical`,
`ill (symbol := string) Intuitionistic.intuitionistic`, …): predicates
applied to terms, `bound 0` the innermost quantifier's variable,
`eigen 0` an eigenvariable, a `∀` step naming its body and its
eigenvariable, a `∃` step its witness; a refutation of a sequent with a
quantifier has no certificate. An unprovable sequent whose refutation is `unbalanced`,
`equation` or `classical` gets a certificate that the calculus does not
derive it; one that rests on the search has none. `--standalone` adds
the imports; `--lemma NAME` names the lemma.

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
$ linlog prove --format rocq "forall x. p(x) |- exists y. p(y)"
(* provable (MLL1, classical, focus engine) *)
Lemma certificate : ll (symbol := string) Calculus.classical [fex (covar "p" [bound 0]); fex (var "p" [bound 0])].
Proof.
  apply (soundness_classical String.eqb String.eqb_spec
    (Exists (covar "p" [bound 0]) (eigen 0) (Exists (var "p" [bound 0]) (eigen 0) (Ax "p" [eigen 0])))).
  vm_compute. reflexivity.
Qed.
$ linlog prove --format rocq "A & B |- A * B"
no certificate: a refutation of kind `exhausted` rests on the search and has no check
(* unprovable (MALL, classical, focus engine): the search was exhaustive *)
```

Compile a certificate with the kernel on the load path: `rocq compile
-R path/to/nanoyalla NanoYalla proof.v`, or for `--kernel linlog`
`rocq compile -R path/to/rocq/theories Linlog proof.v`; Rocq prints
nothing for a certificate it accepts.

## 19. Mistakes to avoid

- Unknown (exit status 3) is not unprovable: raise the limit the line
  names, or name a bound (`--copies N`) to get a definite answer within
  it.
- With `-i`, write `A -o B` and put hypotheses left of `|-`: `~`, `par`,
  `?` and `bot` are refused, and so are two formulas right of `|-`.
- `check` takes the mode from its flags: check a proof of `prove -i`
  with `check -i`.
- A derivation is one-sided by default: `⊢ ~A, A` is the axiom
  `A ⊢ A`.
- Quote the sequent; `*`, `|`, `&`, `!` and `?` mean something to the
  shell.
