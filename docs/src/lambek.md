# The Lambek calculus

`--lambek` decides the Lambek calculus: intuitionistic sequents
`A₁, …, Aₙ ⊢ B` whose hypotheses keep the order they are written in,
built from atoms with `*` and the two divisions, `A \ B` (from `A` on
its left it gives `B`, as `A -o B` does) and `B / A` (from `A` on its
right), and every sequent of a proof with a hypothesis.
`--empty-antecedents` allows sequents without one (the calculus L*). The
text is read as `--cyclic` reads it, of which the calculus is the
intuitionistic part; `seq print --lambek` shows the sequent as the
calculus reads it, and derivations are two-sided with the divisions'
rules:

```console
$ linlog seq print --lambek "b / a, a |- b"
b / a, a ⊢ b
$ linlog prove --lambek "c / b, a, a \ b |- c"
provable (IMLL, Lambek, essential engine)
───── ax   ───── ax
b ⊢ b      c ⊢ c
──────────────── /L   ───── ax
  c / b, b ⊢ c        a ⊢ a
  ───────────────────────── \L
     c / b, a, a \ b ⊢ c
$ linlog prove --lambek "a, b |- b * a"
unprovable (IMLL, Lambek, essential engine): the search was exhaustive
$ linlog prove --lambek "e |- e * (a \ a)"
unprovable (IMLL, Lambek, essential engine): the search was exhaustive
$ linlog prove --lambek --empty-antecedents "e |- e * (a \ a)"
provable (IMLL, Lambek with empty antecedents, essential engine)
            ───── ax
            a ⊢ a
───── ax   ─────── \R
e ⊢ e      ⊢ a \ a
────────────────── ⊗R
 e ⊢ e ⊗ (a \ a)
$ linlog prove --lambek "|- a \ a"
error: not an intuitionistic sequent: the sequent has no formula left of ⊢, which the Lambek calculus without empty antecedents needs
```

The essential engine decides the calculus: it searches for an axiom
linking that is a planar essential net, its links crossing none in the
cyclic order and its implications dominating their antecedents, and
without empty antecedents the body of every division with a hypothesis
of its own. Batches and problem files name the modes `lambek` and
`lambek-star`.
