# Interactive proving

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

With `--invertible`, or after the command `invertible on`, the session
applies by itself every rule that is invertible in its mode, whose
premises are provable whenever its conclusion is, so that applying it
never loses a proof: `⅋`, `⊥`, `&` and `⊤` (under their two-sided names
in intuitionistic mode, `⊸R`, `⊗L`, `&R`, `⊕L`, `1L`, `⊤R`, `0L`), `!`
when every other formula is a `?`, and `1`, `⊗` and the axiom when their
formulas are the whole goal; at once to every open goal, then to the
goals each step opens. Each is a step of the session, and `undo`
retracts a step together with the invertible steps that followed it:

```console
$ linlog interact --invertible "A -o B, B -o C |- A -o C"
applied 1 invertible step
> goals
goal 1: ⊢ 0: A ⊗ ~B, 1: B ⊗ ~C, 2: ~A, 3: C
> apply 1 0 ⊗ 2
then 1 invertible step
opened goal 3: ⊢ 0: ~B, 1: B ⊗ ~C, 2: C
> apply 3 1 ⊗ 0
then 2 invertible steps
closed; no goal is open: `proof` checks the proof
> undo
retracted 3 steps
reopened goal 3: ⊢ 0: ~B, 1: B ⊗ ~C, 2: C
> invertible off
invertible rules off
> apply 3 1 ⊗ 0
opened goal 4: ⊢ 0: ~B, 1: B
opened goal 5: ⊢ 0: ~C, 1: C
> invertible on
invertible rules on; then 2 invertible steps
no goal is open: `proof` checks the proof
> proof
valid proof (classical)
             ─────── ax   ─────── ax
             ⊢ ~B, B      ⊢ ~C, C
─────── ax   ──────────────────── ⊗
⊢ A, ~A        ⊢ ~B, B ⊗ ~C, C
────────────────────────────── ⊗
   ⊢ A ⊗ ~B, B ⊗ ~C, ~A, C
   ──────────────────────── ⅋
   ⊢ A ⊗ ~B, B ⊗ ~C, ~A ⅋ C
```

The exit status is 0 when the session ends with a finished proof that
checks, 1 otherwise; a command's error is printed as `error: …` and the
session goes on, and 2 means the session could not start or read its
input. The same operations are the library's `Interactive`
type, whose JSON form is what a web client will hold between requests.
