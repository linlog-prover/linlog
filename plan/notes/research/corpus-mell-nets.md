# Corpus: MELL proof nets

A test set for later steps of linlog, in the notation of `mell-nets-spec.md` (this directory). The question for each item: is the proof structure a proof net (a correct proof of its sequent)? Each answer was given by one reader and checked blind by a second of another model; where they disagreed, a third decided by argument. No item was disputed, so no item carries a dispute note. Totals: 50 items, 21 proof-net, 29 not-a-proof-net, 0 dropped.

## correct/dereliction

### mell-nets-01
Item:
```
Sequent ⊢ ?X⊥, X. Criterion without Mix.
⊢ q, x
q: ?X⊥ = ?(y)
y: X⊥
x: X
x — y
```
Answer: proof-net
Justification: Depth 0: vertices q, y, x; edges q–y (collector with n = 1, not switched) and y–x. 3 vertices, 2 edges, connected: a tree. No boxes. Proof: ax ⊢ X⊥, X; ?d ⊢ ?X⊥, X.

## correct/box-under-par

### mell-nets-02
Item:
```
Sequent ⊢ ?X⊥ ⅋ !X. Criterion without Mix.
⊢ p
p: ?X⊥ ⅋ !X = (q, b)
q: ?X⊥ = ?(d)
b: !X = box B (x)
x: X in B
y: X⊥ in B
d: ?X⊥ = door B (y)
x — y
```
Answer: proof-net
Justification: Depth 0: vertices p, q, b; the door edge q–b and one of p–q, p–b. Either switching gives 2 edges on 3 vertices, connected: a tree. G(B): x, y, d with x–y, d–y: a tree, conclusions x (premise of !) and d. Proof: ax ⊢ X⊥, X; ?d ⊢ ?X⊥, X; ! ⊢ ?X⊥, !X; ⅋ ⊢ ?X⊥ ⅋ !X.

## correct/box-two-doors

### mell-nets-03
Item:
```
Sequent ⊢ ?X⊥, ?Y⊥, !(X ⊗ Y). Criterion without Mix.
⊢ q1, q2, b
q1: ?X⊥ = ?(d1)
q2: ?Y⊥ = ?(d2)
b: !(X ⊗ Y) = box B (t)
t: X ⊗ Y = (x, y) in B
x: X in B
y: Y in B
a: X⊥ in B
c: Y⊥ in B
d1: ?X⊥ = door B (a)
d2: ?Y⊥ = door B (c)
x — a
y — c
```
Answer: proof-net
Justification: Depth 0: q1, q2, b with door edges q1–b, q2–b: a tree. G(B): t, x, y, a, c, d1, d2 (7) with t–x, t–y, x–a, y–c, d1–a, d2–c (6, nothing switched), connected (d1-a-x-t-y-c-d2): a tree. Proof: ax ⊢ X⊥, X and ⊢ Y⊥, Y; ⊗ ⊢ X⊥, Y⊥, X ⊗ Y; ?d twice; ! ⊢ ?X⊥, ?Y⊥, !(X ⊗ Y).

## correct/contraction-three

### mell-nets-04
Item:
```
Sequent ⊢ ?X⊥, X ⊗ (X ⊗ X). Criterion without Mix.
⊢ q, t1
q: ?X⊥ = ?(y1, y2, y3)
t1: X ⊗ (X ⊗ X) = (x1, t2)
t2: X ⊗ X = (x2, x3)
x1: X
x2: X
x3: X
y1: X⊥
y2: X⊥
y3: X⊥
x1 — y1
x2 — y2
x3 — y3
```
Answer: proof-net
Justification: Depth 0 has 9 vertices. A switching keeps the four tensor edges, the three links and one edge q–yi: 8 edges. Without q, the tensor tree with its three pendant links is connected (y1-x1-t1-t2-x2-y2, t2-x3-y3), and q hangs on the one yi kept, so every switching is a tree. Proof: three axioms, ⊗ twice ⊢ X⊥, X⊥, X⊥, X ⊗ (X ⊗ X), ?d three times, ?c twice.

## correct/contraction-door-and-dereliction

### mell-nets-05
Item:
```
Sequent ⊢ ?X⊥, !X ⊗ X. Criterion without Mix.
⊢ q, t
q: ?X⊥ = ?(d, y2)
t: !X ⊗ X = (b, x2)
b: !X = box B (x)
x: X in B
y: X⊥ in B
d: ?X⊥ = door B (y)
x — y
x2: X
y2: X⊥
x2 — y2
```
Answer: proof-net
Justification: Depth 0: q, t, b, y2, x2 (5). Fixed edges t–b, t–x2, x2–y2. q has 2 premises and is switched: keeping the door edge q–b gives the path q-b-t-x2-y2, and keeping q–y2 gives b-t-x2-y2-q. Both are trees. G(B): x–y, d–y, a tree. Proof: ⊢ ?X⊥, !X (ax, ?d, !) and ⊢ ?X⊥, X (ax, ?d); ⊗ ⊢ ?X⊥, ?X⊥, !X ⊗ X; ?c.

## correct/nested-door-chain

### mell-nets-06
Item:
```
Sequent ⊢ ?X⊥, !!X. Criterion without Mix.
⊢ q, b1
q: ?X⊥ = ?(d1)
b1: !!X = box B1 (b2)
b2: !X = box B2 (x) in B1
x: X in B2
y: X⊥ in B2
d2: ?X⊥ = door B2 (y)
d1: ?X⊥ = door B1 (d2)
x — y
```
Answer: proof-net
Justification: Depth 0: q–b1, a tree. G(B1): b2 and d1, joined by the edge that d2 (a door of B2, nested directly in B1) contributes: a tree with conclusions b2 (premise of !_B1) and d1. G(B2): x–y, d2–y, a tree. Every door premise sits one level down. Proof: ax; ?d ⊢ ?X⊥, X; ! ⊢ ?X⊥, !X; ! ⊢ ?X⊥, !!X.

## correct/nested-contraction-in-door

### mell-nets-07
Item:
```
Sequent ⊢ ?X⊥, !(!X ⊗ X). Criterion without Mix.
⊢ q, b1
q: ?X⊥ = ?(d1)
b1: !(!X ⊗ X) = box B1 (t)
t: !X ⊗ X = (b2, x1) in B1
b2: !X = box B2 (x2) in B1
x2: X in B2
y2: X⊥ in B2
d2: ?X⊥ = door B2 (y2)
x2 — y2
x1: X in B1
y1: X⊥ in B1
x1 — y1
d1: ?X⊥ = door B1 (d2, y1)
```
Answer: proof-net
Justification: Depth 0: q–b1, a tree. G(B1): t, b2, x1, y1, d1 (5). Fixed edges t–b2, t–x1, x1–y1. Door d1 has 2 premises and is switched: keeping d1–b2 (via d2) gives d1-b2-t-x1-y1, and keeping d1–y1 gives b2-t-x1-y1-d1. Both are trees. G(B2): x2–y2, d2–y2, a tree. Proof: ⊢ ?X⊥, !X (ax, ?d, !); ax ⊢ X⊥, X; ⊗ ⊢ ?X⊥, X⊥, !X ⊗ X; ?d ⊢ ?X⊥, ?X⊥, !X ⊗ X; ?c; ! ⊢ ?X⊥, !(!X ⊗ X).

## correct/box-empty-context

### mell-nets-08
Item:
```
Sequent ⊢ !1. Criterion without Mix.
⊢ b
b: !1 = box B (o)
o: 1 in B
```
Answer: proof-net
Justification: Depth 0: b alone, a tree. G(B): o alone, a tree with conclusion o (premise of !). The box has no doors. Proof: 1 ⊢ 1; ! with empty context ⊢ !1.

## correct/weakening-jump-to-box

### mell-nets-09
Item:
```
Sequent ⊢ ?Y, !1. Criterion without Mix.
⊢ w, b
w: ?Y = ?()
b: !1 = box B (o)
o: 1 in B
w -> b
```
Answer: proof-net
Justification: Depth 0: w, b with the jump w–b, a tree. The jump targets b, a vertex at w's depth that is not w's parent (w has none). G(B): o alone. Proof: 1; ! ⊢ !1; ?w ⊢ ?Y, !1.

## correct/weakening-inside-box

### mell-nets-10
Item:
```
Sequent ⊢ ?X⊥, !(?Y ⅋ X). Criterion without Mix.
⊢ q, b
q: ?X⊥ = ?(d)
b: !(?Y ⅋ X) = box B (p)
p: ?Y ⅋ X = (w, x) in B
w: ?Y = ?() in B
x: X in B
y: X⊥ in B
d: ?X⊥ = door B (y)
x — y
w -> x
```
Answer: proof-net
Justification: Depth 0: q–b, a tree. G(B): p, w, x, y, d. Fixed edges w–x (the jump, same depth, not w's parent p), x–y and d–y, plus one of p–w, p–x. Switching p–w gives the path p-w-x-y-d; switching p–x gives the star at x plus y-d. Both are trees. Proof: ax ⊢ X⊥, X; ?w ⊢ X⊥, ?Y, X; ⅋; ?d; ! ⊢ ?X⊥, !(?Y ⅋ X).

## correct/bot-inside-box

### mell-nets-11
Item:
```
Sequent ⊢ ?X⊥, !(X ⅋ ⊥). Criterion without Mix.
⊢ q, b
q: ?X⊥ = ?(d)
b: !(X ⅋ ⊥) = box B (p)
p: X ⅋ ⊥ = (x, z) in B
x: X in B
z: ⊥ in B
y: X⊥ in B
d: ?X⊥ = door B (y)
x — y
z -> x
```
Answer: proof-net
Justification: Depth 0: q–b. G(B): p, x, z, y, d. Fixed edges z–x (the jump, inside B, not to z's parent p), x–y, d–y, plus p–x or p–z. Switching p–x: edges p–x, z–x, x–y, y–d, a tree. Switching p–z: the path p-z-x-y-d, a tree. Proof: ax; ⊥ ⊢ X⊥, X, ⊥; ⅋ ⊢ X⊥, X ⅋ ⊥; ?d; !.

## correct/weakening-under-par

### mell-nets-12
Item:
```
Sequent ⊢ ?Y ⅋ X⊥, X. Criterion without Mix.
⊢ p, b
p: ?Y ⅋ X⊥ = (w, a)
w: ?Y = ?()
a: X⊥
b: X
a — b
w -> b
```
Answer: proof-net
Justification: Depth 0: p, w, a, b. Fixed edges a–b and the jump w–b, plus one of p–w, p–a. Switching p–w gives the path p-w-b-a; switching p–a gives p-a-b-w. Both are trees. Proof: ax ⊢ X⊥, X; ?w ⊢ ?Y, X⊥, X; ⅋ ⊢ ?Y ⅋ X⊥, X.

## correct/units-with-jumps

### mell-nets-13
Item:
```
Sequent ⊢ ⊥ ⊗ ⊥, 1, 1. Criterion without Mix.
⊢ t, o1, o2
t: ⊥ ⊗ ⊥ = (z1, z2)
z1: ⊥
z2: ⊥
o1: 1
o2: 1
z1 -> o1
z2 -> o2
```
Answer: proof-net
Justification: Depth 0: t, z1, z2, o1, o2 (5), with edges t–z1, t–z2, z1–o1, z2–o2 (4) forming the path o1-z1-t-z2-o2: a tree. Nothing is switched. Proof: 1 ⊢ 1; ⊥ ⊢ ⊥, 1, twice; ⊗ ⊢ ⊥ ⊗ ⊥, 1, 1.

## correct/dereliction-of-tensor

### mell-nets-14
Item:
```
Sequent ⊢ ?(X⊥ ⊗ Y⊥), X ⅋ Y. Criterion without Mix.
⊢ q, p
q: ?(X⊥ ⊗ Y⊥) = ?(s)
s: X⊥ ⊗ Y⊥ = (a, c)
a: X⊥
c: Y⊥
p: X ⅋ Y = (x, y)
x: X
y: Y
a — x
c — y
```
Answer: proof-net
Justification: Depth 0 has 7 vertices. Fixed edges q–s, s–a, s–c, a–x, c–y, plus p–x or p–y: 6 edges. Switching p–x gives q-s, s-a-x-p, s-c-y, a tree. p–y is symmetric. Proof: ax twice; ⊗ ⊢ X⊥ ⊗ Y⊥, X, Y; ⅋; ?d.

## correct/two-boxes

### mell-nets-15
Item:
```
Sequent ⊢ ?X⊥ ⅋ ?X, !X ⊗ !X⊥. Criterion without Mix.
⊢ p, t
p: ?X⊥ ⅋ ?X = (q1, q2)
q1: ?X⊥ = ?(d1)
q2: ?X = ?(d2)
t: !X ⊗ !X⊥ = (b1, b2)
b1: !X = box B1 (x1)
x1: X in B1
y1: X⊥ in B1
d1: ?X⊥ = door B1 (y1)
x1 — y1
b2: !X⊥ = box B2 (y2)
y2: X⊥ in B2
x2: X in B2
d2: ?X = door B2 (x2)
y2 — x2
```
Answer: proof-net
Justification: Depth 0: p, q1, q2, t, b1, b2 (6). Fixed edges q1–b1, q2–b2 (door edges), t–b1, t–b2, plus p–q1 or p–q2. Switching p–q1 gives the path p-q1-b1-t-b2-q2; p–q2 gives p-q2-b2-t-b1-q1. Both are trees. Each box: link plus door edge, a tree. Proof: ⊢ ?X⊥, !X and ⊢ ?X, !X⊥ (ax, ?d, !); ⊗ ⊢ ?X⊥, ?X, !X ⊗ !X⊥; ⅋.

## correct/door-of-compound

### mell-nets-16
Item:
```
Sequent ⊢ ?(X⊥ ⊗ X⊥), !(X ⅋ X). Criterion without Mix.
⊢ q, b
q: ?(X⊥ ⊗ X⊥) = ?(d)
b: !(X ⅋ X) = box B (p)
p: X ⅋ X = (x1, x2) in B
x1: X in B
x2: X in B
s: X⊥ ⊗ X⊥ = (y1, y2) in B
y1: X⊥ in B
y2: X⊥ in B
d: ?(X⊥ ⊗ X⊥) = door B (s)
x1 — y1
x2 — y2
```
Answer: proof-net
Justification: Depth 0: q–b. G(B): p, x1, x2, s, y1, y2, d (7). Fixed edges s–y1, s–y2, x1–y1, x2–y2, d–s, plus p–x1 or p–x2: 6 edges. Switching p–x1 gives the path p-x1-y1-s-y2-x2 with d on s, a tree; p–x2 is symmetric. Proof: ax twice; ⊗ ⊢ X⊥ ⊗ X⊥, X, X; ⅋; ?d; !.

## correct/mix-only

### mell-nets-17
Item:
```
Sequent ⊢ ?X⊥, X ⅋ X. Criterion WITH binary Mix (acyclic and non-empty per depth graph).
⊢ q, p
q: ?X⊥ = ?(y1, y2)
p: X ⅋ X = (x1, x2)
x1: X
x2: X
y1: X⊥
y2: X⊥
x1 — y1
x2 — y2
```
Answer: proof-net
Justification: Depth 0 has 6 vertices, and every switching keeps 4 edges: x1–y1, x2–y2, one p–xi and one q–yj. For i = j = 1 the edges are p-x1-y1-q plus x2-y2; for i = 1, j = 2 they are p-x1-y1 plus q-y2-x2. The other cases are symmetric. No switching has a cycle, and the graph is non-empty. Proof with Mix: ax, ax, Mix ⊢ X⊥, X⊥, X, X; ?d twice; ?c ⊢ ?X⊥, X, X; ⅋.

## cut/correct-box-against-dereliction

### mell-nets-18
Item:
```
Sequent ⊢ ?X⊥, X. Criterion without Mix. A cut is written c: cut = (u, v): a vertex with the two dual premises u, v at one depth, no conclusion, not listed after ⊢.
⊢ q, x2
q: ?X⊥ = ?(d)
b: !X = box B (x)
x: X in B
y: X⊥ in B
d: ?X⊥ = door B (y)
x — y
r: ?X⊥ = ?(y2)
y2: X⊥
x2: X
y2 — x2
c: cut = (b, r)
```
Answer: proof-net
Justification: Depth 0: q, b, r, y2, x2, c (6). Edges q–b (door), c–b, c–r, r–y2, y2–x2 (5, none switched) form the path q-b-c-r-y2-x2: a tree. G(B): x–y, d–y, a tree. The cut joins !X and ?X⊥, which are dual and both at depth 0. Proof: ⊢ ?X⊥, !X (ax, ?d, !) and ⊢ ?X⊥, X (ax, ?d); cut on !X / ?X⊥.

## cut/correct-duplication-redex

### mell-nets-19
Item:
```
Sequent ⊢ ?X⊥, X ⊗ X. Criterion without Mix. Cut notation c: cut = (u, v), as in mell-nets-18.
⊢ q, t
q: ?X⊥ = ?(d)
b: !X = box B (x)
x: X in B
y: X⊥ in B
d: ?X⊥ = door B (y)
x — y
r: ?X⊥ = ?(y1, y2)
y1: X⊥
y2: X⊥
t: X ⊗ X = (x1, x2)
x1: X
x2: X
x1 — y1
x2 — y2
c: cut = (b, r)
```
Answer: proof-net
Justification: Depth 0 has 9 vertices. Fixed edges q–b, c–b, c–r, y1–x1, y2–x2, t–x1, t–x2, plus r–y1 or r–y2: 8 edges. Switching r–y1 gives the path q-b-c-r-y1-x1-t-x2-y2; r–y2 is symmetric. Both are trees. G(B) is a tree. Proof: ⊢ ?X⊥, !X (ax, ?d, !) and ⊢ ?X⊥, X ⊗ X (ax, ax, ⊗, ?d, ?d, ?c); cut on !X / ?X⊥.

## cut/correct-erasure-redex

### mell-nets-20
Item:
```
Sequent ⊢ 1. Criterion without Mix. Cut notation c: cut = (u, v), as in mell-nets-18.
⊢ o2
b: !1 = box B (o)
o: 1 in B
w: ?⊥ = ?()
o2: 1
c: cut = (b, w)
w -> o2
```
Answer: proof-net
Justification: Depth 0: b, w, o2, c, with edges c–b, c–w and the jump w–o2 (to a vertex at w's depth that is not w's parent c). These form the path b-c-w-o2, a tree. G(B): o alone. !1 and ?⊥ are dual. Proof: ⊢ !1 (1, !); ⊢ ?⊥, 1 (1, ?w); cut on !1 / ?⊥ ⊢ 1.

## cut/correct-cut-inside-box

### mell-nets-21
Item:
```
Sequent ⊢ ?X⊥, !X. Criterion without Mix. Cut notation c: cut = (u, v), as in mell-nets-18.
⊢ q, b
q: ?X⊥ = ?(d)
b: !X = box B (x)
x: X in B
y1: X⊥ in B
x1: X in B
y: X⊥ in B
c: cut = (y1, x1) in B
d: ?X⊥ = door B (y)
x — y1
x1 — y
```
Answer: proof-net
Justification: Depth 0: q–b, a tree. G(B): x, y1, x1, y, c, d (6), with edges x–y1, y1–c, c–x1, x1–y, y–d (5) forming the path x-y1-c-x1-y-d: a tree. Its conclusions are x (premise of !) and the door d, and the cut lies wholly inside B. Proof: ax ⊢ X, X⊥ twice; cut on X⊥ / X ⊢ X, X⊥; ?d; ! ⊢ ?X⊥, !X.

## switching-cycle/depth-0-dereliction

### mell-nets-22
Item:
```
Sequent ⊢ ?X⊥ ⊗ X. Criterion without Mix.
⊢ t
t: ?X⊥ ⊗ X = (q, x)
q: ?X⊥ = ?(y)
y: X⊥
x: X
x — y
```
Answer: not-a-proof-net
Justification: Depth 0: edges t–q, t–x, q–y and y–x. Nothing is switched: q has n = 1 and there is no ⅋. So every switching contains the cycle t-q-y-x-t (SwitchingCycle at depth 0). The sequent is unprovable anyway: the only rule is ⊗, which needs ⊢ X apart.

## switching-cycle/depth-0-two-boxes

### mell-nets-23
Item:
```
Sequent ⊢ ?X⊥ ⊗ ?X, !X ⊗ !X⊥. Criterion without Mix.
⊢ t1, t2
t1: ?X⊥ ⊗ ?X = (q1, q2)
q1: ?X⊥ = ?(d1)
q2: ?X = ?(d2)
t2: !X ⊗ !X⊥ = (b1, b2)
b1: !X = box B1 (x1)
x1: X in B1
y1: X⊥ in B1
d1: ?X⊥ = door B1 (y1)
x1 — y1
b2: !X⊥ = box B2 (y2)
y2: X⊥ in B2
x2: X in B2
d2: ?X = door B2 (x2)
y2 — x2
```
Answer: not-a-proof-net
Justification: Depth 0 has edges t1–q1, t1–q2, q1–b1 and q2–b2 (door edges), t2–b1 and t2–b2. None is switched: no ⅋, and every collector has n = 1. The cycle t1-q1-b1-t2-b2-q2-t1 survives every switching. (The boxes themselves are trees.) The sequent is also unprovable: either ⊗ leaves one lone formula among ?X⊥, ?X, !X, !X⊥, and none is provable alone.

## switching-cycle/depth-0-dereliction-of-tensor

### mell-nets-24
Item:
```
Sequent ⊢ ?(X⊥ ⊗ Y⊥), X ⊗ Y. Criterion without Mix.
⊢ q, t
q: ?(X⊥ ⊗ Y⊥) = ?(s)
s: X⊥ ⊗ Y⊥ = (a, c)
a: X⊥
c: Y⊥
t: X ⊗ Y = (x, y)
x: X
y: Y
a — x
c — y
```
Answer: not-a-proof-net
Justification: Depth 0 has edges q–s, s–a, s–c, t–x, t–y, a–x and c–y, none switched. They contain the cycle s-a-x-t-y-c-s. The sequent is unprovable as well: the one X needs exactly one dereliction, and the resulting MLL sequent ⊢ X⊥ ⊗ Y⊥, X ⊗ Y has no proof.

## switching-cycle/in-box

### mell-nets-25
Item:
```
Sequent ⊢ !(X ⊗ X⊥). Criterion without Mix.
⊢ b
b: !(X ⊗ X⊥) = box B (t)
t: X ⊗ X⊥ = (x, y) in B
x: X in B
y: X⊥ in B
x — y
```
Answer: not-a-proof-net
Justification: Depth 0 (b alone) is fine. G(B) has edges t–x, t–y and x–y, none switched: the cycle t-x-y-t (SwitchingCycle in box B). The sequent is unprovable: promotion needs ⊢ X ⊗ X⊥, whose ⊗ splits into ⊢ X and ⊢ X⊥.

## switching-cycle/in-box-through-collector

### mell-nets-26
Item:
```
Sequent ⊢ !(?X⊥ ⊗ X). Criterion without Mix.
⊢ b
b: !(?X⊥ ⊗ X) = box B (t)
t: ?X⊥ ⊗ X = (q, x) in B
q: ?X⊥ = ?(y) in B
y: X⊥ in B
x: X in B
x — y
```
Answer: not-a-proof-net
Justification: G(B) has edges t–q, t–x, q–y (collector with n = 1, not switched) and y–x: the cycle t-q-y-x-t, in box B. The sequent is unprovable: it needs ⊢ ?X⊥ ⊗ X, which splits into ⊢ X alone.

## switching-cycle/in-box-through-door

### mell-nets-27
Item:
```
Sequent ⊢ ?(X⊥ ⊗ X⊥), !(X ⊗ X). Criterion without Mix.
⊢ q, b
q: ?(X⊥ ⊗ X⊥) = ?(d)
b: !(X ⊗ X) = box B (t)
t: X ⊗ X = (x1, x2) in B
x1: X in B
x2: X in B
s: X⊥ ⊗ X⊥ = (y1, y2) in B
y1: X⊥ in B
y2: X⊥ in B
d: ?(X⊥ ⊗ X⊥) = door B (s)
x1 — y1
x2 — y2
```
Answer: not-a-proof-net
Justification: Depth 0 (q–b) is a tree. G(B) has edges t–x1, t–x2, s–y1, s–y2, x1–y1, x2–y2 and d–s. Nothing is switched: d has n = 1. The cycle t-x1-y1-s-y2-x2-t is in box B. The sequent is unprovable: two X need one dereliction, and the MLL sequent ⊢ X⊥ ⊗ X⊥, X ⊗ X has no proof.

## switching-cycle/in-nested-box

### mell-nets-28
Item:
```
Sequent ⊢ !!(X ⊗ X⊥). Criterion without Mix.
⊢ b1
b1: !!(X ⊗ X⊥) = box B1 (b2)
b2: !(X ⊗ X⊥) = box B2 (t) in B1
t: X ⊗ X⊥ = (x, y) in B2
x: X in B2
y: X⊥ in B2
x — y
```
Answer: not-a-proof-net
Justification: Depth 0 (b1) and G(B1) (b2 alone) are trees. G(B2) has edges t–x, t–y and x–y: a cycle at depth 2. The sequent is unprovable: two promotions reduce it to ⊢ X ⊗ X⊥.

## disconnected/depth-0

### mell-nets-29
Item:
```
Sequent ⊢ ?X⊥, !X, Y⊥, Y. Criterion without Mix.
⊢ q, b, a, c
q: ?X⊥ = ?(d)
b: !X = box B (x)
x: X in B
y: X⊥ in B
d: ?X⊥ = door B (y)
x — y
a: Y⊥
c: Y
a — c
```
Answer: not-a-proof-net
Justification: Depth 0: vertices q, b, a, c with edges q–b and a–c only: two components (Disconnected), though acyclic. It would be correct with Mix. Without Mix the sequent is unprovable: promotion of !X needs an all-? context, and Y⊥, Y can only leave through an axiom that would have to be the whole sequent.

## disconnected/contraction-without-mix

### mell-nets-30
Item:
```
Sequent ⊢ ?X⊥, X ⅋ X. Criterion without Mix.
⊢ q, p
q: ?X⊥ = ?(y1, y2)
p: X ⅋ X = (x1, x2)
x1: X
x2: X
y1: X⊥
y2: X⊥
x1 — y1
x2 — y2
```
Answer: not-a-proof-net
Justification: There are 6 vertices, and every switching keeps 4 edges: 2 links, one p–xi and one q–yj. Fewer than 5 edges, so every switching is disconnected. (With Mix it is correct; see mell-nets-17.) Without Mix the sequent is unprovable: after ⅋, the X, X need two derelicted X⊥ and then a split, which only Mix provides.

## box-content-not-a-net/disconnected

### mell-nets-31
Item:
```
Sequent ⊢ ?X⊥, ?Y⊥, !(X ⅋ Y). Criterion without Mix.
⊢ q1, q2, b
q1: ?X⊥ = ?(d1)
q2: ?Y⊥ = ?(d2)
b: !(X ⅋ Y) = box B (p)
p: X ⅋ Y = (x, y) in B
x: X in B
y: Y in B
a: X⊥ in B
c: Y⊥ in B
d1: ?X⊥ = door B (a)
d2: ?Y⊥ = door B (c)
x — a
y — c
```
Answer: not-a-proof-net
Justification: Depth 0 (q1–b, q2–b) is a tree. G(B) has 7 vertices (p, x, y, a, c, d1, d2), and a switching keeps 5 edges: one of p–x, p–y, plus x–a, y–c, d1–a, d2–c. So it is disconnected: the box content is not a net. The sequent is unprovable without Mix: promotion leaves ⊢ ?X⊥…, ?Y⊥…, X, Y, which needs exactly one X⊥ and one Y⊥ and then a split of ⊢ X⊥, Y⊥, X, Y.

## wrong-door/formula-mismatch

### mell-nets-32
Item:
```
Sequent ⊢ ?Y⊥, !X. Criterion without Mix.
⊢ q, b
q: ?Y⊥ = ?(d)
b: !X = box B (x)
x: X in B
y: X⊥ in B
d: ?Y⊥ = door B (y)
x — y
```
Answer: not-a-proof-net
Justification: A door of type ?B takes premises of type B (or doors of ?B). d is ?Y⊥ but its premise y is X⊥, so this is not a proof structure. The sequent is unprovable as well: promotion leaves ⊢ ?Y⊥…, X, and neither weakening nor dereliction of ?Y⊥ yields a provable sequent.

## wrong-door/skips-a-level

### mell-nets-33
Item:
```
Sequent ⊢ ?X⊥, !!X. Criterion without Mix.
⊢ q, b1
q: ?X⊥ = ?(d2)
b1: !!X = box B1 (b2)
b2: !X = box B2 (x) in B1
x: X in B2
y: X⊥ in B2
d2: ?X⊥ = door B2 (y)
x — y
```
Answer: not-a-proof-net
Justification: A collector's premise is a vertex at its depth or a door of a box nested directly in its box. q is at depth 0, but d2 is a door of B2, which is nested in B1, not in the outside. B1 lacks the door d1, so B1's content is ⊢ ?X⊥, !X with the ?X⊥ leaving B1 through no door: not a proof structure. The sequent is provable; mell-nets-06 is its correct net.

## wrong-door/door-of-sibling-box

### mell-nets-34
Item:
```
Sequent ⊢ ?X⊥, !X, !1. Criterion without Mix.
⊢ q, b1, b2
q: ?X⊥ = ?(d)
b1: !X = box B1 (x)
x: X in B1
y: X⊥ in B1
x — y
b2: !1 = box B2 (o)
o: 1 in B2
d: ?X⊥ = door B2 (y)
```
Answer: not-a-proof-net
Justification: d is a door of B2 and lies inside B2, but its premise y lies inside the disjoint box B1. A door's premises must be inside its own box. B1's content keeps y as a conclusion that is neither the premise of !_B1 nor a door. This is not a proof structure. The sequent is unprovable without Mix as well: neither promotion has an all-? context, and the ?-rules on ?X⊥ do not help.

## wrong-door/collected-inside-its-box

### mell-nets-35
Item:
```
Sequent ⊢ !(?X⊥ ⅋ X). Criterion without Mix.
⊢ b
b: !(?X⊥ ⅋ X) = box B (p)
p: ?X⊥ ⅋ X = (q, x) in B
q: ?X⊥ = ?(d) in B
d: ?X⊥ = door B (y)
y: X⊥ in B
x: X in B
x — y
```
Answer: not-a-proof-net
Justification: A door of B leaves B as a premise of a collector in B's parent. Here d is collected by q, which is inside B itself. Equivalently, q at depth B may take only vertices of B or doors of boxes nested in B, and d is neither. This is not a proof structure. The sequent is provable (ax, ?d, ⅋, ! with empty context); its net has q = ?(y) and no door.

## wrong-door/door-without-premise

### mell-nets-36
Item:
```
Sequent ⊢ ?Y, !1. Criterion without Mix.
⊢ q, b
q: ?Y = ?(d)
b: !1 = box B (o)
o: 1 in B
d: ?Y = door B ()
```
Answer: not-a-proof-net
Justification: A door has n ≥ 1 premises (DoorWithoutPremise; the notation's door production also requires one name). A weakening is a collector with n = 0 at its own depth, so it never becomes a door. This is not a proof structure. The sequent is provable; mell-nets-09 is its correct net.

## contraction-across-box/no-door

### mell-nets-37
Item:
```
Sequent ⊢ ?X⊥, !X ⊗ X. Criterion without Mix.
⊢ q, t
q: ?X⊥ = ?(y, y2)
t: !X ⊗ X = (b, x2)
b: !X = box B (x)
x: X in B
y: X⊥ in B
x — y
x2: X
y2: X⊥
x2 — y2
```
Answer: not-a-proof-net
Justification: The collector q at depth 0 contracts y, which lies inside B (depth 1), with no door in between. Collector premises must be at the collector's depth or be doors of directly nested boxes. Box B's content then has the conclusions X and X⊥, a promotion with a non-? context. This is not a proof structure. The sequent is provable; mell-nets-05 is its correct net, with door d.

## contraction-across-box/dereliction-no-door

### mell-nets-38
Item:
```
Sequent ⊢ ?X⊥, !X. Criterion without Mix.
⊢ q, b
q: ?X⊥ = ?(y)
b: !X = box B (x)
x: X in B
y: X⊥ in B
x — y
```
Answer: not-a-proof-net
Justification: The collector edge q–y crosses the border of B without a door: q is at depth 0 and y at depth 1. This is not a proof structure, and B's content ⊢ X⊥, X is not an all-? promotion context. The sequent is provable; the correct net routes y through d = door B (y) and q = ?(d).

## promotion-context-not-all-?/conclusion-inside-box

### mell-nets-39
Item:
```
Sequent ⊢ ?X⊥, !(X ⊗ Y), Y⊥. Criterion without Mix.
⊢ q, b, c
q: ?X⊥ = ?(d)
b: !(X ⊗ Y) = box B (t)
t: X ⊗ Y = (x, y) in B
x: X in B
y: Y in B
a: X⊥ in B
c: Y⊥ in B
d: ?X⊥ = door B (a)
x — a
y — c
```
Answer: not-a-proof-net
Justification: c lies in B but is listed as a conclusion of the sequent. The conclusions of G(B) may only be the premise of !_B and B's doors, and the roots must be at depth 0. B's content is ⊢ ?X⊥, Y⊥, X ⊗ Y, a promotion with the non-? context Y⊥. This is not a proof structure. The sequent is unprovable without Mix: Y⊥ never leaves the context of the promotion.

## promotion-context-not-all-?/link-across-boxes

### mell-nets-40
Item:
```
Sequent ⊢ X⊥, !X. Criterion without Mix.
⊢ y, b
y: X⊥
b: !X = box B (x)
x: X in B
x — y
```
Answer: not-a-proof-net
Justification: The axiom link x–y joins x in B to y at depth 0 (LinkAcrossBoxes; links join literals of the same box). Read as a proof, it is a promotion with the context X⊥, which is not a ?. The sequent is unprovable: ! needs an all-? context, and Mix would leave ⊢ X⊥ or ⊢ !X alone.

## promotion-context-not-all-?/non-?-door

### mell-nets-41
Item:
```
Sequent ⊢ X⊥, !X. Criterion without Mix.
⊢ d, b
b: !X = box B (x)
x: X in B
y: X⊥ in B
d: X⊥ = door B (y)
x — y
```
Answer: not-a-proof-net
Justification: A door has type ?B. Here d has type X⊥, so this is not a proof structure: it encodes a promotion whose context X⊥ is not a ? formula. The sequent ⊢ X⊥, !X is unprovable (see mell-nets-40).

## cut/switching-cycle

### mell-nets-42
Item:
```
Sequent ⊢ X. Criterion without Mix. Cut notation c: cut = (u, v), as in mell-nets-18.
⊢ x2
b: !X = box B (x)
x: X in B
y: X⊥ in B
d: ?X⊥ = door B (y)
x — y
r: ?X⊥ = ?(d, y2)
y2: X⊥
x2: X
y2 — x2
c: cut = (b, r)
```
Answer: not-a-proof-net
Justification: Depth 0: b, r, y2, x2, c. Fixed edges c–b, c–r and y2–x2. r has 2 premises and is switched. The switching that keeps the door edge r–b has the cycle b-c-r-b, with y2-x2 a separate component, so the criterion fails. (Keeping r–y2 gives a tree, but every switching must pass.) ⊢ X is unprovable.

## cut/across-boxes

### mell-nets-43
Item:
```
Sequent ⊢ !X, X⊥. Criterion without Mix. Cut notation c: cut = (u, v), as in mell-nets-18.
⊢ b, y2
b: !X = box B (x)
x: X in B
y1: X⊥ in B
x — y1
x2: X
y2: X⊥
x2 — y2
c: cut = (y1, x2)
```
Answer: not-a-proof-net
Justification: The cut c at depth 0 takes y1 from inside B. Cuts, like axiom links, join vertices of one depth. B's content leaves y1 neither as a door nor as the premise of !. This is not a proof structure. ⊢ !X, X⊥ is unprovable (see mell-nets-40).

## cut/jump-cycle

### mell-nets-44
Item:
```
Sequent ⊢ 1. Criterion without Mix. Cut notation c: cut = (u, v), as in mell-nets-18.
⊢ o2
b: !1 = box B (o)
o: 1 in B
w: ?⊥ = ?()
o2: 1
c: cut = (b, w)
w -> b
```
Answer: not-a-proof-net
Justification: Depth 0 has edges c–b, c–w and the jump w–b: the cycle b-c-w-b. o2 is isolated as well (Disconnected). mell-nets-20 is the correct variant, with w -> o2.

## jump/to-parent

### mell-nets-45
Item:
```
Sequent ⊢ ?Y ⅋ X⊥, X. Criterion without Mix.
⊢ p, b
p: ?Y ⅋ X⊥ = (w, a)
w: ?Y = ?()
a: X⊥
b: X
a — b
w -> p
```
Answer: not-a-proof-net
Justification: The jump goes to w's own parent p (JumpToNeighbour). In the switching that keeps p–w, the tree edge p–w and the jump w–p form a 2-cycle. In that same switching, {a, b} is not connected to {p, w}. mell-nets-12 is the correct variant, with w -> b.

## jump/parallel-pair

### mell-nets-46
Item:
```
Sequent ⊢ ?Y, ⊥. Criterion without Mix.
⊢ w, z
w: ?Y = ?()
z: ⊥
w -> z
z -> w
```
Answer: not-a-proof-net
Justification: The two jumps join the same pair, giving the parallel edges w–z twice: a cycle in every switching (JumpToNeighbour). Each of w and z has only the other as a possible target, so no choice of jumps succeeds. The sequent is unprovable with binary Mix: ⊥ and ?w lead to ⊢ (empty), ?d leads to ⊢ Y, ⊥, and Mix leaves ⊢ ?Y or ⊢ ⊥ alone.

## jump/missing

### mell-nets-47
Item:
```
Sequent ⊢ ⊥, X⊥, X. Criterion without Mix.
⊢ z, a, b
z: ⊥
a: X⊥
b: X
a — b
```
Answer: not-a-proof-net
Justification: The ⊥ vertex z carries no jump (MissingJump). Read with no jump, z is isolated in every switching, so depth 0 is disconnected. The sequent is provable (ax, ⊥), and adding z -> a or z -> b makes the structure correct.

## jump/units-cycle

### mell-nets-48
Item:
```
Sequent ⊢ ⊥ ⊗ ⊥, 1, 1. Criterion without Mix.
⊢ t, o1, o2
t: ⊥ ⊗ ⊥ = (z1, z2)
z1: ⊥
z2: ⊥
o1: 1
o2: 1
z1 -> o1
z2 -> o1
```
Answer: not-a-proof-net
Justification: Depth 0 has edges t–z1, t–z2, z1–o1 and z2–o1, none switched: the cycle t-z1-o1-z2-t. o2 is isolated as well. The sequent is provable, and mell-nets-13 (z2 -> o2) is its correct net.

## jump/no-target-in-box

### mell-nets-49
Item:
```
Sequent ⊢ !⊥. Criterion without Mix.
⊢ b
b: !⊥ = box B (z)
z: ⊥ in B
```
Answer: not-a-proof-net
Justification: z needs a jump to another vertex of G(B), but it is the only vertex there, and b lies outside B. So the jump is missing (MissingJump). The sequent is unprovable: ⊢ !⊥ needs ⊢ ⊥, which needs ⊢ (empty), and no rule concludes that.

## jump/across-boxes

### mell-nets-50
Item:
```
Sequent ⊢ ?X⊥, !(X ⅋ ⊥). Criterion without Mix.
⊢ q, b
q: ?X⊥ = ?(d)
b: !(X ⅋ ⊥) = box B (p)
p: X ⅋ ⊥ = (x, z) in B
x: X in B
z: ⊥ in B
y: X⊥ in B
d: ?X⊥ = door B (y)
x — y
z -> q
```
Answer: not-a-proof-net
Justification: z lies in B, but its jump target q is at depth 0. A jump's target must be at the jumping vertex's depth (JumpAcrossBoxes), so this is not a proof structure. Without a valid jump, z is pendant only through p in the switching that keeps p–x, so it is disconnected there. mell-nets-11 (z -> x) is the correct net of this provable sequent.

## Dropped items

None.
