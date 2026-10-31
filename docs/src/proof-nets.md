# Proof nets

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

A proof with exponentials, of MELL without units, classical with or
without Mix or intuitionistic, gets its net with `!`-boxes. A vertex stands for one
instance of a subformula, since a `?` formula may be used several times:
each `?` vertex is listed with its copies, each in the `!`-box it lies in
where that is another than the `?`'s own, or with `no copy` and the
literal of the axiom link its weakening hangs on. `--format json` writes
the net's document instead of the outcome, which the library reads back
(`MellStructure`, the form the wire level 4 added):

```console
$ linlog prove --net "!a, !b |- !a"
provable (MELL, classical, focus engine)
⊢ ?~a, ?~b, !a
?~a[0]: ~a[3] in !a[2]
?~b[1]: no copy, hangs on a[4]
~a[3] — a[4]
proof net
$ linlog prove --net --format json "!a, !b |- !a"
{"version":4,"sequent":{"terms":[{"D":0},{"?":0},{"D":1},{"?":2},{"V":0},{"!":4}],"roots":[1,3,5],"atoms":["a","b"],"antecedents":2},"mix":false,"vertices":[[1,0,2],[5,2,2]],"links":[[4,3]],"weakenings":[[1,4]]}
```

An intuitionistic proof's net is the one of its one-sided sequent,
written as the sequent was: a hypothesis `!A`, whose copies the boxes
and the derivation use, stands for the conclusion `?~A`, and an axiom
links an atom left of `⊢` to the same atom right of it. Affine logic has
no proof nets:

```console
$ linlog prove --net -i "!a, !(a -o b) |- !b"
provable (IMELL, intuitionistic, two-sided engine)
!a, !(a ⊸ b) ⊢ !b
!a[0]: a[3] in !b[2]
!(a ⊸ b)[1]: a ⊸ b[4] in !b[2]
a[3] — a[5]
b[6] — b[7]
proof net
$ linlog prove --net -i --affine "!a |- !a"
error: proof nets exist in linear mode only, with or without --mix, not in intuitionistic affine mode
```
