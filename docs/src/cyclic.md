# Cyclic linear logic

`--cyclic` decides cyclic linear logic, multiplicative and without
units: a one-sided sequent is a sequence of formulas read up to
rotation, with no other exchange, so `⊗` joins the formulas on either
side of it and `⅋` two neighbours. The text is read in that order: the
dual of a product reverses it, `~(A * B)` being `~B par ~A`, the formulas
left of `|-` come in the reverse order, each negated, and `A -o B` is
`~A par B`, so that `A, A -o B |- B` holds and `A -o B, A |- B` does not.
`seq print --cyclic` and `seq json --cyclic` show the one-sided sequent
it reads:

```console
$ linlog seq print --cyclic "a * b |- a * b"
⊢ ~b ⅋ ~a, a ⊗ b
$ linlog prove --cyclic "a * b |- a * b"
provable (MLL, cyclic, net engine)
─────── ax   ─────── ax
⊢ ~a, a      ⊢ ~b, b
──────────────────── ⊗
  ⊢ ~b, ~a, a ⊗ b
  ──────────────── ⅋
  ⊢ ~b ⅋ ~a, a ⊗ b
$ linlog prove --cyclic "a * b |- b * a"
unprovable (MLL, cyclic, net engine): the search was exhaustive
```

Every sequent of a derivation is in its cyclic order, read from its
first formula. The net engine decides the mode: it searches for an axiom
linking that is a proof net and planar, no two links crossing when the
literals stand in their order around a circle, and `--net` draws it so:

```console
$ linlog prove --cyclic --net "a, a -o b |- b"
provable (MLL, cyclic, net engine)
⊢ ~b ⊗ a, ~a, b
~b[1] — b[4]
a[2] — ~a[3]
planar proof net
```

Units, additives, exponentials, Mix, weakening and `--intuitionistic`
are refused in the mode, the other engines too, and Rocq certificates,
whose calculi are commutative:

```console
$ linlog prove --cyclic "|- a * b, 1"
error: cyclic mode decides unit-free MLL, and the sequent lies in MLL with units
$ linlog prove --cyclic --format rocq "a |- a"
error: no certificate: the Rocq calculi are commutative, so a derivation or a disproof of the cyclic mode has no certificate
```
