# Cut and its elimination

`cut G C P…` cuts goal G on a formula C, written as the mode writes
formulas (intuitionistic in intuitionistic mode, as the sequent is) and
quoted when it has blanks: the formulas at the positions P go to the
first premise, which proves C, and the others to the second, which takes
C as a hypothesis, so that a goal `Γ, Δ ⊢ D` leaves `Γ ⊢ C` and
`Δ, C ⊢ D`; `undo` takes the cut back with its formula. The search never
makes a cut, and `close` proves each premise without one:

```console
$ linlog interact -i "A, A -o B, B -o C |- C"
> cut 0 B 0 1
opened goal 1: 0: A, 1: A ⊸ B ⊢ 2: B
opened goal 2: 0: B ⊸ C, 2: B ⊢ 1: C
> close
goal 1: provable (IMLL, intuitionistic, two-sided engine)
goal 2: provable (IMLL, intuitionistic, two-sided engine)
no goal is open: `proof` checks the proof
> proof
valid proof (intuitionistic)
───── ax   ───── ax   ───── ax   ───── ax
A ⊢ A      B ⊢ B      B ⊢ B      C ⊢ C
──────────────── ⊸L   ──────────────── ⊸L
  A, A ⊸ B ⊢ B          B ⊸ C, B ⊢ C
  ────────────────────────────────── cut
         A, A ⊸ B, B ⊸ C ⊢ C
```

`linlog eliminate` reads a proof file, as `proof FILE` or `prove
--format json` writes it, checks it in the mode the flags give, and
eliminates its cuts one step at a time: each step reduces the cut that
comes first in the proof, the one with no cut above it, and is printed
with its case (the axiom case, the key case of a connective and its
dual, a commutative case, and for the exponentials weakening,
dereliction, contraction and a box entering a box) and the cut
formula, every proof reached is checked, and the proof the elimination
ends with is drawn as `check` draws it. `--steps N` takes N steps at
most (10 000 by default) and `--node-limit N` refuses a step that makes
a proof of more than N nodes (100 000 by default): the exponentials can
make a proof grow beyond any practical size, and a bound reached prints
the proof reached, cuts and all. With `--format json` the output is the
record `{"version", "proof", "steps", "end"}`: the proof reached, each
step's `cut` (a node of the proof it was taken on), `formula` and
`case`, and `end`, `cut_free` or `refused` with its `reason`. The exit
status is 0 when no cut is left or the steps asked for with `--steps`
were taken, 1 for a proof that is not valid, 3 when the default bound
on steps, another bound or the time limit stopped it with cuts left,
and 2 for an error.

With `--net` the cuts are eliminated on the proof's net instead, for the
sequents `--net` takes (MLL without units, and nets with boxes of MELL
without units, classical with or without `--mix` or intuitionistic): the
proof's cuts are the net's cut links, each step reduces the first cut
link (nets have no commutative cases: the axiom case, the key case of ⊗
and ⅋, and on boxes weakening, which erases a box, dereliction, which
opens one, contraction, which duplicates one, and a box entering a box),
every net reached passes the correctness criterion, and the net reached
is printed as text, every cut link as a line `cut A[x] — A⊥[y]`; the
bound on nodes counts a net's vertices. `--format svg`, `png` or `pdf`
with `--output FILE` draws every net into a file of its own: `FILE-k`
(`net-1.svg` for `net.svg`) the net step k was taken on, with its cut
link, a bar under its two formulas, and what the step acts on (the links
at its ends, its `⊗` and `⅋`, the box of its `!`, the box it enters)
highlighted, and `FILE` the net reached; an SVG file's first line is a
comment naming its step. Each file is written as soon as it is drawn,
within `--derivation-limit`, and the steps' files take 256 MiB together
at most (`--drawings-limit SIZE`, the settings key `cut.drawn_bytes`):
past it the steps go on undrawn, a line on standard error says from
which step, and `FILE` still holds the net reached. `--format json` writes the record `{"version",
"net", "steps", "end"}` with the net reached, each step's `cut` the two
vertices of its cut link.

```console
$ linlog interact -i "!a |- a * a"
> cut 0 !a 0
opened goal 1: 0: !a ⊢ 1: !a
opened goal 2: 1: !a ⊢ 0: a ⊗ a
> close
goal 1: provable (IMELL, intuitionistic, two-sided engine)
goal 2: provable (IMELL, intuitionistic, horn engine)
no goal is open: `proof` checks the proof
> proof contraction.json
valid proof written to contraction.json
$ linlog eliminate -q -i contraction.json
1. contraction on the cut of !a
2. dereliction on the cut of !a
3. a commutative case on the cut of a
4. the axiom case on the cut of a
5. a commutative case on the cut of a
6. the axiom case on the cut of a
cut-free proof of !a ⊢ a ⊗ a (intuitionistic) after 6 steps
$ linlog eliminate -i --steps=2 contraction.json
1. contraction on the cut of !a
2. dereliction on the cut of !a
proof of !a ⊢ a ⊗ a (intuitionistic) with 2 cuts left after 2 steps: the elimination took the most steps allowed, 2; --steps N allows more
            ───── ax    ───── ax   ───── ax
            a ⊢ a       a ⊢ a      a ⊢ a
───── ax    ────── !L   ──────────────── ⊗R
a ⊢ a       !a ⊢ a        a, a ⊢ a ⊗ a
────── !L   ────────────────────────── cut
!a ⊢ a            !a, a ⊢ a ⊗ a
─────────────────────────────── cut
        !a, !a ⊢ a ⊗ a
        ────────────── !c
          !a ⊢ a ⊗ a
$ linlog eliminate -i --net --steps=1 contraction.json
1. contraction on the cut of !a
proof net of !a ⊢ a ⊗ a (intuitionistic) with 2 cut links left after 1 step: the elimination took the most steps allowed, 1; --steps N allows more
!a ⊢ a ⊗ a
!a[0]: a[2] in !a[10], a[3] in !a[6]
!a[8]: a[9]
!a[12]: a[13]
cut !a[6] — !a[8]
cut !a[10] — !a[12]
a[2] — a[11]
a[3] — a[7]
a[4] — a[13]
a[5] — a[9]
proof net
$ linlog eliminate -i --net --format svg --output net.svg contraction.json
1. contraction on the cut of !a
2. dereliction on the cut of !a
3. the axiom case on the cut of a
4. dereliction on the cut of !a
5. the axiom case on the cut of a
cut-free proof net of !a ⊢ a ⊗ a (intuitionistic) after 5 steps
```

The library's `proofs::cut::step` reduces a cut the caller names, or the
strategy's, and says which cut and which case; `proofs::cut::eliminate`
runs to the end within `cut::Options`.

The library's `nets::cut::step` and `nets::cut::eliminate` do the same
on a `ProofStructure` or a `MellStructure`, and `svg::net_step` and
`svg::mell_net_step` draw a step.
