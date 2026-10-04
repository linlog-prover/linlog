# Wrong status headers in LLTP's intuitionistic problems

Draft of a report to the maintainers of the LLTP library
(github.com/meta-logic/lltp), about commit
`e0394fb8e9f6ad5127c92460c3f0936ccf64b693`. Every path below is
relative to the library's `ILL/` directory unless it starts with
`ILTP+KLE/`.

## How the headers were checked

On 28 files of the intuitionistic collections, a prover's verdict
contradicts the `Status (intuit.)` header. We read each file as the
sequent `axioms ⊢ conjecture` of intuitionistic linear logic (ILL) and
decided it with linlog, a prover for linear logic
(github.com/linlog-prover/linlog, version 0.1.0, unreleased). Nine files
whose header says Non-Theorem are provable: linlog's proof checker,
which shares no code with its search, accepted each proof before the
command printed it, and the proofs are attached as JSON files that
`linlog check` checks again and as the derivations it prints. Nineteen
files whose header says Theorem are unprovable: linlog's search ends
exhaustively on each, and each has a classical countermodel that does not
depend on linlog at all. Forget linearity: erase `!` and `?`, read `⊗`
and `&` as "and", `⅋` and `⊕` as "or", `⊸` as implication, `1` and `⊤`
as true and `0` and `⊥` as false. Every rule of ILL stays sound for
classical logic under this reading, so a sequent that ILL proves is
classically valid. An assignment of the atoms under which the axioms
are true and the conjecture is false therefore shows that the sequent
has no proof in ILL. We found each assignment by truth table with a
short script that implements only this reading, and give it below. We
also compared each file with the result files the library ships, the
thirteen `ILL/*.txt` files (`PATH ; TIME ; VERDICT`) of the prover in
Maude that the README names: they cover all 28 files, agree with linlog
on the 12 they decide, and time out on the other 16.

Twenty-four of the 28 files come from one fault in the translator
`tptpparser/`, which gives a negation `~` the widest possible scope, and
the same fault makes the headers of 21 more files certainly wrong. One
file has `T` where the unit `top` belongs, and three have a header
that is already wrong in the ILTP original. The sections below give
the table, the causes, the details of each file, the further files,
and one malformed file.

## The 28 files

Each file was decided with the command

```sh
linlog prove -i --deterministic --format json --file FILE
```

with linlog's defaults otherwise: a time limit of 2 s and no copy bound
for `!` formulas. The search deepens the copy bound by itself, 0, 1, 2,
and so on, until it finds a proof or a level ends without needing
more copies. The column "bound" gives the copy bound at which it ended
(the field `statistics.copies` of the JSON output). Every call answered
in under 10 ms of wall-clock time, the start of the process included.
`--deterministic` runs one thread, so the proof is the same on every
run. `-i` selects intuitionistic linear logic, which an LLTP file does
not state.

| # | File | Header | linlog | Bound | Evidence | Maude result file |
|--:|---|---|---|--:|---|---|
| 1 | `KLE-01/KLE065+1.p` | Non-Theorem | proved | 2 | [proof](lltp-headers/KLE-01_KLE065+1.json) | true, agrees |
| 2 | `KLE-cbn/KLE065+1.p` | Non-Theorem | proved | 1 | [proof](lltp-headers/KLE-cbn_KLE065+1.json) | true, agrees |
| 3 | `KLE-cbv/KLE065+1.p` | Non-Theorem | proved | 1 | [proof](lltp-headers/KLE-cbv_KLE065+1.json) | true, agrees |
| 4 | `ILLTP-SYN-01/SYN001+1.p` | Non-Theorem | proved | 2 | [proof](lltp-headers/ILLTP-SYN-01_SYN001+1.json) | true, agrees |
| 5 | `ILLTP-SYN-cbn/SYN001+1.p` | Non-Theorem | proved | 2 | [proof](lltp-headers/ILLTP-SYN-cbn_SYN001+1.json) | true, agrees |
| 6 | `ILLTP-SYN-cbv/SYN001+1.p` | Non-Theorem | proved | 2 | [proof](lltp-headers/ILLTP-SYN-cbv_SYN001+1.json) | true, agrees |
| 7 | `ILLTP-SYJ-01/SYJ212+1.001.p` | Non-Theorem | proved | 6 | [proof](lltp-headers/ILLTP-SYJ-01_SYJ212+1.001.json) | true, agrees |
| 8 | `ILLTP-SYJ-cbn/SYJ212+1.001.p` | Non-Theorem | proved | 3 | [proof](lltp-headers/ILLTP-SYJ-cbn_SYJ212+1.001.json) | true, agrees |
| 9 | `ILLTP-SYJ-cbv/SYJ212+1.001.p` | Non-Theorem | proved | 4 | [proof](lltp-headers/ILLTP-SYJ-cbv_SYJ212+1.001.json) | true, agrees |
| 10 | `KLE-01/KLE013+1.p` | Theorem | unprovable | 1 | a, b true | false, agrees |
| 11 | `KLE-cbn/KLE013+1.p` | Theorem | unprovable | 0 | a, b true | timeout |
| 12 | `KLE-cbv/KLE013+1.p` | Theorem | unprovable | 1 | a, b true | false, agrees |
| 13 | `ILLTP-SYN-01/SYN041+1.p` | Theorem | unprovable | 0 | p, q false | timeout |
| 14 | `ILLTP-SYN-cbn/SYN041+1.p` | Theorem | unprovable | 0 | p, q false | timeout |
| 15 | `ILLTP-SYN-cbv/SYN041+1.p` | Theorem | unprovable | 0 | p, q false | timeout |
| 16 | `ILLTP-SYN-cbn/SYN915+1.p` | Theorem | unprovable | 0 | T false | false, agrees |
| 17 | `KLE-cbn/KLE017+1.p` | Theorem | unprovable | 5 | a true, b false | timeout |
| 18 | `KLE-cbv/KLE017+1.p` | Theorem | unprovable | 5 | a true, b false | timeout |
| 19 | `KLE-01/KLE069+1.p` | Theorem | unprovable | 13 | a false, b true | timeout |
| 20 | `KLE-cbn/KLE069+1.p` | Theorem | unprovable | 10 | a false, b true | timeout |
| 21 | `KLE-cbv/KLE069+1.p` | Theorem | unprovable | 5 | a false, b true | timeout |
| 22 | `KLE-cbn/KLE078+1.p` | Theorem | unprovable | 8 | a false, b true | timeout |
| 23 | `KLE-cbv/KLE078+1.p` | Theorem | unprovable | 6 | a false, b true | timeout |
| 24 | `KLE-cbn/KLE086+1.p` | Theorem | unprovable | 9 | a false, b true | timeout |
| 25 | `KLE-cbn/KLE088+1.p` | Theorem | unprovable | 9 | a false, b true | timeout |
| 26 | `ILLTP-SYJ-cbv/SYJ103+1.p` | Theorem | unprovable | 8 | a false, b true | timeout |
| 27 | `ILLTP-SYJ-cbv/SYJ105+1.003.p` | Theorem | unprovable | 6 | a, b false | timeout |
| 28 | `ILLTP-SYJ-cbv/SYJ105+1.004.p` | Theorem | unprovable | 6 | a, b false, c true | timeout |

## The causes

**The translator reads `~` with the widest scope.** The grammar
`tptpparser/tptpparser.mly` declares no precedence for any connective,
and ocamlyacc resolves the resulting shift/reduce conflicts by shifting.
So `NOT formula` extends as far to the right as it can, and
`~a => b` becomes `~(a => b)` where TPTP means `(~a) => b`. The binary
connectives are not affected: TPTP requires parentheses around mixed
`&` and `|` and around a chain of `=>` or `<=>`, and none of the ILTP
and KLE originals omits them. Twenty-four of the 28 files come from
this:

| Problem | The ILTP original | What the translator read |
|---|---|---|
| SYN001+1 | `~ ~ p <=> p` | `~ ~ (p <=> p)` |
| SYJ212+1.001 | `~(~(a1)) <=> a1` | `~((~(a1)) <=> a1)` |
| KLE013+1 | `a ⊢ ~a => b` | `a ⊢ ~(a => b)` |
| SYN041+1 | `~ ( p => q ) => ( q => p )` | `~((p => q) => (q => p))` |
| KLE017+1 | `~a => b ⊢ ~b => ~~a` | `~(a => b) ⊢ ~(b => ~~a)` |
| KLE069+1 | `(a \| b) => ~(~a & ~b)` | `(a \| b) => ~~(a & ~b)` |
| KLE078+1 | `(a => b) => ~~(~a \| b)` | `(a => b) => ~~~(a \| b)` |
| KLE086+1 | `~(a \| b) <=> ~(~a => b)` | `~((a \| b) <=> ~~(a => b))` |
| KLE088+1 | `~(a \| b) <=> (~a & ~b)` | `~((a \| b) <=> ~(a & ~b))` |
| SYJ103+1 | `~(a) \| ~(b) ⊢ ~(b) \| ~(a)` | `~(a \| ~(b)) ⊢ ~(b \| ~(a))` |
| SYJ105+1.003 | `~~((a & b) \| (~a \| ~b))` | `~~((a & b) \| ~(a \| ~b))` |
| SYJ105+1.004 | `~~((a & (b & c)) \| (~a \| (~b \| ~c)))` | `~~((a & (b & c)) \| ~(a \| ~(b \| ~c)))` |

The first two turn intuitionistic non-theorems into theorems; the
others turn theorems into formulas that are not even classically valid.
We confirmed this reading on the files themselves. We checked the 687
translated files of the `KLE-01`, `KLE-cbn`, `KLE-cbv` and `ILLTP-*`
collections whose problems have at most 14 atoms. In every one but
`ILLTP-SYN-cbn/SYN915+1.p` (below), the classical reading of the file
has the same truth table as the ILTP original read with `~` widest. For
72 of them, it differs from the truth table of the original read as
TPTP means it. Declaring the usual precedences, for instance

```
%right IMP BIMP
%left OR
%left AND
%nonassoc NOT
```

before the `%%`, would fix the parser; the translations would then
need to be generated again.

**The call-by-name translation writes `top` as `T`.**
`tptpparser/translation.ml` prints the unit `TOP` as `T` and `BOT` as
`B`, while the library's syntax (README) spells them `top` and `bot` and
reads uppercase names as atoms. Girard's call-by-name translation maps
`$true` to `TOP`, so `ILLTP-SYN-cbn/SYN915+1.p` has the conjecture `T`,
an atom, where the ILTP original has `$true`. The 01 and call-by-value
translations map `$true` to `1` and are right. No other file of the
`KLE-01`, `KLE-cbn`, `KLE-cbv` and `ILLTP-*` collections has `T` or `B`
outside its comments.

**KLE065's header is wrong at the source.** `ILTP+KLE/KLE/KLE065+1.p`
states `(a & (b | ~b)) => a` with the status Non-Theorem, but the
formula is an intuitionistic theorem (it follows from `a & X => a`).
Its three translations are faithful and provable, so the header of the
original and of all three translations should say Theorem, unless a
different formula was intended.

## The files one by one

### KLE065+1 (header Non-Theorem; provable)

The conjecture is the translation of `(a & (b | ~b)) => a`. Each proof
uses only the left `a` of the conjunction.

- `KLE-01/KLE065+1.p`: proved at copy bound 2;
  [proof](lltp-headers/KLE-01_KLE065+1.json). The Maude result file has
  `KLE-01/KLE065+1.p ; 17 ; true`.
- `KLE-cbn/KLE065+1.p`: proved at copy bound 1;
  [proof](lltp-headers/KLE-cbn_KLE065+1.json). The Maude result file
  has `true` (22 ms).
- `KLE-cbv/KLE065+1.p`: proved at copy bound 1;
  [proof](lltp-headers/KLE-cbv_KLE065+1.json). The Maude result file
  has `true` (150 ms).

<details><summary>The three derivations, as <code>linlog check -i --tree always</code> prints them</summary>

```
valid proof of ⊢ !(!(!a & !(!b ⊕ !(!b ⊸ !0))) ⊸ a) (intuitionistic)
             ───── ax
             a ⊢ a
             ────── !L
             !a ⊢ a
   ─────────────────────────── &L₁
   !a & !(!b ⊕ !(!b ⊸ !0)) ⊢ a
  ────────────────────────────── !L
  !(!a & !(!b ⊕ !(!b ⊸ !0))) ⊢ a
 ──────────────────────────────── ⊸R
 ⊢ !(!a & !(!b ⊕ !(!b ⊸ !0))) ⊸ a
─────────────────────────────────── !R
⊢ !(!(!a & !(!b ⊕ !(!b ⊸ !0))) ⊸ a)
```

```
valid proof of ⊢ !(a & (!b ⊕ !(!b ⊸ 0))) ⊸ a (intuitionistic)
           ───── ax
           a ⊢ a
  ──────────────────────── &L₁
  a & (!b ⊕ !(!b ⊸ 0)) ⊢ a
 ─────────────────────────── !L
 !(a & (!b ⊕ !(!b ⊸ 0))) ⊢ a
───────────────────────────── ⊸R
⊢ !(a & (!b ⊕ !(!b ⊸ 0))) ⊸ a
```

```
valid proof of ⊢ !((!a ⊗ (!b ⊕ !(!b ⊸ 0))) ⊸ !a) (intuitionistic)
  ───── ax            ───── ax
  a ⊢ a               a ⊢ a
  ────── !L           ────── !L
  !a ⊢ a              !a ⊢ a
  ─────── !R          ─────── !R
  !a ⊢ !a             !a ⊢ !a
─────────── !w   ────────────────── !w
!a, !b ⊢ !a      !a, !(!b ⊸ 0) ⊢ !a
─────────────────────────────────── ⊕L
      !a, !b ⊕ !(!b ⊸ 0) ⊢ !a
     ────────────────────────── ⊗L
     !a ⊗ (!b ⊕ !(!b ⊸ 0)) ⊢ !a
   ────────────────────────────── ⊸R
   ⊢ (!a ⊗ (!b ⊕ !(!b ⊸ 0))) ⊸ !a
  ───────────────────────────────── !R
  ⊢ !((!a ⊗ (!b ⊕ !(!b ⊸ 0))) ⊸ !a)
```

</details>

### SYN001+1 (header Non-Theorem; provable)

Pelletier's problem 2, `~ ~ p <=> p`, is not intuitionistically valid,
but the translated conjecture is `~~((p => p) & (p => p))`, which is.

- `ILLTP-SYN-01/SYN001+1.p`: proved at copy bound 2;
  [proof](lltp-headers/ILLTP-SYN-01_SYN001+1.json). The Maude result
  file has `true` (173 ms).
- `ILLTP-SYN-cbn/SYN001+1.p`: proved at copy bound 2;
  [proof](lltp-headers/ILLTP-SYN-cbn_SYN001+1.json). The Maude result
  file has `true` (33 ms).
- `ILLTP-SYN-cbv/SYN001+1.p`: proved at copy bound 2;
  [proof](lltp-headers/ILLTP-SYN-cbv_SYN001+1.json). The Maude result
  file has `true` (156 ms).

<details><summary>The three derivations</summary>

```
valid proof of ⊢ !(!(!!(!(!p ⊸ p) & !(!p ⊸ p)) ⊸ !0) ⊸ 0) (intuitionistic)
  ───── ax         ───── ax
  p ⊢ p            p ⊢ p
  ────── !L        ────── !L
  !p ⊢ p           !p ⊢ p
 ──────── ⊸R      ──────── ⊸R
 ⊢ !p ⊸ p         ⊢ !p ⊸ p
─────────── !R   ─────────── !R
⊢ !(!p ⊸ p)      ⊢ !(!p ⊸ p)
──────────────────────────── &R
  ⊢ !(!p ⊸ p) & !(!p ⊸ p)
 ────────────────────────── !R    ───── 0L
 ⊢ !(!(!p ⊸ p) & !(!p ⊸ p))       0 ⊢ 0
 ─────────────────────────── !R   ────── !L
 ⊢ !!(!(!p ⊸ p) & !(!p ⊸ p))      !0 ⊢ 0
 ─────────────────────────────────────── ⊸L
   !!(!(!p ⊸ p) & !(!p ⊸ p)) ⊸ !0 ⊢ 0
  ───────────────────────────────────── !L
  !(!!(!(!p ⊸ p) & !(!p ⊸ p)) ⊸ !0) ⊢ 0
 ─────────────────────────────────────── ⊸R
 ⊢ !(!!(!(!p ⊸ p) & !(!p ⊸ p)) ⊸ !0) ⊸ 0
────────────────────────────────────────── !R
⊢ !(!(!!(!(!p ⊸ p) & !(!p ⊸ p)) ⊸ !0) ⊸ 0)
```

```
valid proof of ⊢ !(!((!p ⊸ p) & (!p ⊸ p)) ⊸ 0) ⊸ 0 (intuitionistic)
  ───── ax      ───── ax
  p ⊢ p         p ⊢ p
  ────── !L     ────── !L
  !p ⊢ p        !p ⊢ p
 ──────── ⊸R   ──────── ⊸R
 ⊢ !p ⊸ p      ⊢ !p ⊸ p
 ────────────────────── &R
 ⊢ (!p ⊸ p) & (!p ⊸ p)
──────────────────────── !R   ───── 0L
⊢ !((!p ⊸ p) & (!p ⊸ p))      0 ⊢ 0
─────────────────────────────────── ⊸L
  !((!p ⊸ p) & (!p ⊸ p)) ⊸ 0 ⊢ 0
 ───────────────────────────────── !L
 !(!((!p ⊸ p) & (!p ⊸ p)) ⊸ 0) ⊢ 0
─────────────────────────────────── ⊸R
⊢ !(!((!p ⊸ p) & (!p ⊸ p)) ⊸ 0) ⊸ 0
```

```
valid proof of ⊢ !(!((!(!p ⊸ !p) ⊗ !(!p ⊸ !p)) ⊸ 0) ⊸ 0) (intuitionistic)
  ───── ax          ───── ax
  p ⊢ p             p ⊢ p
  ────── !L         ────── !L
  !p ⊢ p            !p ⊢ p
  ─────── !R        ─────── !R
  !p ⊢ !p           !p ⊢ !p
 ───────── ⊸R      ───────── ⊸R
 ⊢ !p ⊸ !p         ⊢ !p ⊸ !p
──────────── !R   ──────────── !R
⊢ !(!p ⊸ !p)      ⊢ !(!p ⊸ !p)
────────────────────────────── ⊗R   ───── 0L
  ⊢ !(!p ⊸ !p) ⊗ !(!p ⊸ !p)         0 ⊢ 0
  ─────────────────────────────────────── ⊸L
     (!(!p ⊸ !p) ⊗ !(!p ⊸ !p)) ⊸ 0 ⊢ 0
    ──────────────────────────────────── !L
    !((!(!p ⊸ !p) ⊗ !(!p ⊸ !p)) ⊸ 0) ⊢ 0
   ────────────────────────────────────── ⊸R
   ⊢ !((!(!p ⊸ !p) ⊗ !(!p ⊸ !p)) ⊸ 0) ⊸ 0
  ───────────────────────────────────────── !R
  ⊢ !(!((!(!p ⊸ !p) ⊗ !(!p ⊸ !p)) ⊸ 0) ⊸ 0)
```

</details>

### SYJ212+1.001 (header Non-Theorem; provable)

The original `~(~(a1)) <=> a1` is not intuitionistically valid, but the
translated conjecture is `~(~a1 <=> a1)`, which is.

- `ILLTP-SYJ-01/SYJ212+1.001.p`: proved at copy bound 6;
  [proof](lltp-headers/ILLTP-SYJ-01_SYJ212+1.001.json). The Maude
  result file has `true` (255 ms).
- `ILLTP-SYJ-cbn/SYJ212+1.001.p`: proved at copy bound 3;
  [proof](lltp-headers/ILLTP-SYJ-cbn_SYJ212+1.001.json). The Maude
  result file has `true` (163 ms).
- `ILLTP-SYJ-cbv/SYJ212+1.001.p`: proved at copy bound 4;
  [proof](lltp-headers/ILLTP-SYJ-cbv_SYJ212+1.001.json). The Maude
  result file has `true` (248 ms).

<details><summary>The three derivations (the second is 194 characters wide)</summary>

```
valid proof of ⊢ !(!(!(!!(!a1 ⊸ 0) ⊸ !a1) & !(!a1 ⊸ !(!a1 ⊸ !0))) ⊸ 0) (intuitionistic)
                        ─────── ax
                        a1 ⊢ a1
                        ──────── !L    ───── 0L
                        !a1 ⊢ a1       0 ⊢ 0
         ─────── ax     ───────── !R   ────── !L
         a1 ⊢ a1        !a1 ⊢ !a1      !0 ⊢ 0
         ──────── !L    ───────────────────── ⊸L                  ─────── ax
         !a1 ⊢ a1         !a1, !a1 ⊸ !0 ⊢ 0                       a1 ⊢ a1
         ───────── !R    ──────────────────── !L                  ──────── !L    ───── 0L
         !a1 ⊢ !a1       !a1, !(!a1 ⊸ !0) ⊢ 0                     !a1 ⊢ a1       0 ⊢ 0
         ──────────────────────────────────── ⊸L   ─────── ax     ───────── !R   ────── !L
           !a1, !a1, !a1 ⊸ !(!a1 ⊸ !0) ⊢ 0         a1 ⊢ a1        !a1 ⊢ !a1      !0 ⊢ 0
           ─────────────────────────────── !c      ──────── !L    ───────────────────── ⊸L
             !a1, !a1 ⊸ !(!a1 ⊸ !0) ⊢ 0            !a1 ⊢ a1         !a1, !a1 ⊸ !0 ⊢ 0
            ───────────────────────────── !L       ───────── !R    ──────────────────── !L
            !a1, !(!a1 ⊸ !(!a1 ⊸ !0)) ⊢ 0          !a1 ⊢ !a1       !a1, !(!a1 ⊸ !0) ⊢ 0
            ────────────────────────────── ⊸R      ──────────────────────────────────── ⊸L
            !(!a1 ⊸ !(!a1 ⊸ !0)) ⊢ !a1 ⊸ 0           !a1, !a1, !a1 ⊸ !(!a1 ⊸ !0) ⊢ 0
           ───────────────────────────────── !R      ─────────────────────────────── !c
           !(!a1 ⊸ !(!a1 ⊸ !0)) ⊢ !(!a1 ⊸ 0)           !a1, !a1 ⊸ !(!a1 ⊸ !0) ⊢ 0
           ────────────────────────────────── !R      ───────────────────────────── !L
           !(!a1 ⊸ !(!a1 ⊸ !0)) ⊢ !!(!a1 ⊸ 0)         !a1, !(!a1 ⊸ !(!a1 ⊸ !0)) ⊢ 0
           ──────────────────────────────────────────────────────────────────────── ⊸L
              !!(!a1 ⊸ 0) ⊸ !a1, !(!a1 ⊸ !(!a1 ⊸ !0)), !(!a1 ⊸ !(!a1 ⊸ !0)) ⊢ 0
              ───────────────────────────────────────────────────────────────── !c
                         !!(!a1 ⊸ 0) ⊸ !a1, !(!a1 ⊸ !(!a1 ⊸ !0)) ⊢ 0
                        ────────────────────────────────────────────── !L
                        !(!!(!a1 ⊸ 0) ⊸ !a1), !(!a1 ⊸ !(!a1 ⊸ !0)) ⊢ 0
             ───────────────────────────────────────────────────────────────────── &L₂
             !(!!(!a1 ⊸ 0) ⊸ !a1) & !(!a1 ⊸ !(!a1 ⊸ !0)), !(!!(!a1 ⊸ 0) ⊸ !a1) ⊢ 0
            ──────────────────────────────────────────────────────────────────────── !L
            !(!(!!(!a1 ⊸ 0) ⊸ !a1) & !(!a1 ⊸ !(!a1 ⊸ !0))), !(!!(!a1 ⊸ 0) ⊸ !a1) ⊢ 0
 ─────────────────────────────────────────────────────────────────────────────────────────────── &L₁
 !(!(!!(!a1 ⊸ 0) ⊸ !a1) & !(!a1 ⊸ !(!a1 ⊸ !0))), !(!!(!a1 ⊸ 0) ⊸ !a1) & !(!a1 ⊸ !(!a1 ⊸ !0)) ⊢ 0
────────────────────────────────────────────────────────────────────────────────────────────────── !L
!(!(!!(!a1 ⊸ 0) ⊸ !a1) & !(!a1 ⊸ !(!a1 ⊸ !0))), !(!(!!(!a1 ⊸ 0) ⊸ !a1) & !(!a1 ⊸ !(!a1 ⊸ !0))) ⊢ 0
────────────────────────────────────────────────────────────────────────────────────────────────── !c
                        !(!(!!(!a1 ⊸ 0) ⊸ !a1) & !(!a1 ⊸ !(!a1 ⊸ !0))) ⊢ 0
                       ──────────────────────────────────────────────────── ⊸R
                       ⊢ !(!(!!(!a1 ⊸ 0) ⊸ !a1) & !(!a1 ⊸ !(!a1 ⊸ !0))) ⊸ 0
                      ─────────────────────────────────────────────────────── !R
                      ⊢ !(!(!(!!(!a1 ⊸ 0) ⊸ !a1) & !(!a1 ⊸ !(!a1 ⊸ !0))) ⊸ 0)
```

```
valid proof of ⊢ !((!(!a1 ⊸ 0) ⊸ a1) & (!a1 ⊸ (!a1 ⊸ 0))) ⊸ 0 (intuitionistic)
                                                                                                                               ─────── ax
                                                                                                                               a1 ⊢ a1
                                  ─────── ax                                                                    ─────── ax     ──────── !L
                                  a1 ⊢ a1                                                                       a1 ⊢ a1        !a1 ⊢ a1
                   ─────── ax     ──────── !L                                                                   ──────── !L    ───────── !R   ───── 0L
                   a1 ⊢ a1        !a1 ⊢ a1                                                                      !a1 ⊢ a1       !a1 ⊢ !a1      0 ⊢ 0
                   ──────── !L    ───────── !R   ───── 0L                                                       ───────── !R   ──────────────────── ⊸L
                   !a1 ⊢ a1       !a1 ⊢ !a1      0 ⊢ 0                                                          !a1 ⊢ !a1        !a1, !a1 ⊸ 0 ⊢ 0
                   ───────── !R   ──────────────────── ⊸L                                                       ───────────────────────────────── ⊸L
                   !a1 ⊢ !a1        !a1, !a1 ⊸ 0 ⊢ 0                                                              !a1, !a1, !a1 ⊸ (!a1 ⊸ 0) ⊢ 0
                   ───────────────────────────────── ⊸L                                                           ───────────────────────────── !c
                     !a1, !a1, !a1 ⊸ (!a1 ⊸ 0) ⊢ 0                                                                  !a1, !a1 ⊸ (!a1 ⊸ 0) ⊢ 0
                     ───────────────────────────── !c                                                    ────────────────────────────────────────────── &L₂
                       !a1, !a1 ⊸ (!a1 ⊸ 0) ⊢ 0                                                          (!(!a1 ⊸ 0) ⊸ a1) & (!a1 ⊸ (!a1 ⊸ 0)), !a1 ⊢ 0
            ────────────────────────────────────────────── &L₂                                          ───────────────────────────────────────────────── !L
            (!(!a1 ⊸ 0) ⊸ a1) & (!a1 ⊸ (!a1 ⊸ 0)), !a1 ⊢ 0                                              !((!(!a1 ⊸ 0) ⊸ a1) & (!a1 ⊸ (!a1 ⊸ 0))), !a1 ⊢ 0
           ───────────────────────────────────────────────── !L                                         ────────────────────────────────────────────────── ⊸R
           !((!(!a1 ⊸ 0) ⊸ a1) & (!a1 ⊸ (!a1 ⊸ 0))), !a1 ⊢ 0                                            !((!(!a1 ⊸ 0) ⊸ a1) & (!a1 ⊸ (!a1 ⊸ 0))) ⊢ !a1 ⊸ 0
           ────────────────────────────────────────────────── ⊸R                                       ───────────────────────────────────────────────────── !R   ─────── ax
           !((!(!a1 ⊸ 0) ⊸ a1) & (!a1 ⊸ (!a1 ⊸ 0))) ⊢ !a1 ⊸ 0                                          !((!(!a1 ⊸ 0) ⊸ a1) & (!a1 ⊸ (!a1 ⊸ 0))) ⊢ !(!a1 ⊸ 0)      a1 ⊢ a1
          ───────────────────────────────────────────────────── !R   ─────── ax                        ────────────────────────────────────────────────────────────────── ⊸L
          !((!(!a1 ⊸ 0) ⊸ a1) & (!a1 ⊸ (!a1 ⊸ 0))) ⊢ !(!a1 ⊸ 0)      a1 ⊢ a1                             !((!(!a1 ⊸ 0) ⊸ a1) & (!a1 ⊸ (!a1 ⊸ 0))), !(!a1 ⊸ 0) ⊸ a1 ⊢ a1
          ────────────────────────────────────────────────────────────────── ⊸L               ──────────────────────────────────────────────────────────────────────────────────── &L₁
            !((!(!a1 ⊸ 0) ⊸ a1) & (!a1 ⊸ (!a1 ⊸ 0))), !(!a1 ⊸ 0) ⊸ a1 ⊢ a1                    !((!(!a1 ⊸ 0) ⊸ a1) & (!a1 ⊸ (!a1 ⊸ 0))), (!(!a1 ⊸ 0) ⊸ a1) & (!a1 ⊸ (!a1 ⊸ 0)) ⊢ a1
 ──────────────────────────────────────────────────────────────────────────────────── &L₁    ─────────────────────────────────────────────────────────────────────────────────────── !L
 !((!(!a1 ⊸ 0) ⊸ a1) & (!a1 ⊸ (!a1 ⊸ 0))), (!(!a1 ⊸ 0) ⊸ a1) & (!a1 ⊸ (!a1 ⊸ 0)) ⊢ a1        !((!(!a1 ⊸ 0) ⊸ a1) & (!a1 ⊸ (!a1 ⊸ 0))), !((!(!a1 ⊸ 0) ⊸ a1) & (!a1 ⊸ (!a1 ⊸ 0))) ⊢ a1
─────────────────────────────────────────────────────────────────────────────────────── !L   ─────────────────────────────────────────────────────────────────────────────────────── !c
!((!(!a1 ⊸ 0) ⊸ a1) & (!a1 ⊸ (!a1 ⊸ 0))), !((!(!a1 ⊸ 0) ⊸ a1) & (!a1 ⊸ (!a1 ⊸ 0))) ⊢ a1                           !((!(!a1 ⊸ 0) ⊸ a1) & (!a1 ⊸ (!a1 ⊸ 0))) ⊢ a1
─────────────────────────────────────────────────────────────────────────────────────── !c                        ────────────────────────────────────────────── !R                       ───── 0L
                     !((!(!a1 ⊸ 0) ⊸ a1) & (!a1 ⊸ (!a1 ⊸ 0))) ⊢ a1                                                !((!(!a1 ⊸ 0) ⊸ a1) & (!a1 ⊸ (!a1 ⊸ 0))) ⊢ !a1                          0 ⊢ 0
                     ────────────────────────────────────────────── !R                                            ───────────────────────────────────────────────────────────────────────────── ⊸L
                     !((!(!a1 ⊸ 0) ⊸ a1) & (!a1 ⊸ (!a1 ⊸ 0))) ⊢ !a1                                                           !((!(!a1 ⊸ 0) ⊸ a1) & (!a1 ⊸ (!a1 ⊸ 0))), !a1 ⊸ 0 ⊢ 0
                     ────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────── ⊸L
                                                !((!(!a1 ⊸ 0) ⊸ a1) & (!a1 ⊸ (!a1 ⊸ 0))), !((!(!a1 ⊸ 0) ⊸ a1) & (!a1 ⊸ (!a1 ⊸ 0))), !a1 ⊸ (!a1 ⊸ 0) ⊢ 0
                                                ─────────────────────────────────────────────────────────────────────────────────────────────────────── !c
                                                                     !((!(!a1 ⊸ 0) ⊸ a1) & (!a1 ⊸ (!a1 ⊸ 0))), !a1 ⊸ (!a1 ⊸ 0) ⊢ 0
                                                          ─────────────────────────────────────────────────────────────────────────────────── &L₂
                                                          !((!(!a1 ⊸ 0) ⊸ a1) & (!a1 ⊸ (!a1 ⊸ 0))), (!(!a1 ⊸ 0) ⊸ a1) & (!a1 ⊸ (!a1 ⊸ 0)) ⊢ 0
                                                         ────────────────────────────────────────────────────────────────────────────────────── !L
                                                         !((!(!a1 ⊸ 0) ⊸ a1) & (!a1 ⊸ (!a1 ⊸ 0))), !((!(!a1 ⊸ 0) ⊸ a1) & (!a1 ⊸ (!a1 ⊸ 0))) ⊢ 0
                                                         ────────────────────────────────────────────────────────────────────────────────────── !c
                                                                              !((!(!a1 ⊸ 0) ⊸ a1) & (!a1 ⊸ (!a1 ⊸ 0))) ⊢ 0
                                                                             ────────────────────────────────────────────── ⊸R
                                                                             ⊢ !((!(!a1 ⊸ 0) ⊸ a1) & (!a1 ⊸ (!a1 ⊸ 0))) ⊸ 0
```

```
valid proof of ⊢ !((!(!(!a1 ⊸ 0) ⊸ !a1) ⊗ !(!a1 ⊸ !(!a1 ⊸ 0))) ⊸ 0) (intuitionistic)
               ─────── ax
               a1 ⊢ a1
               ──────── !L
               !a1 ⊢ a1
─────── ax     ───────── !R   ───── 0L                  ─────── ax
a1 ⊢ a1        !a1 ⊢ !a1      0 ⊢ 0                     a1 ⊢ a1
──────── !L    ──────────────────── ⊸L                  ──────── !L
!a1 ⊢ a1         !a1, !a1 ⊸ 0 ⊢ 0                       !a1 ⊢ a1
───────── !R    ─────────────────── !L   ─────── ax     ───────── !R   ───── 0L
!a1 ⊢ !a1       !a1, !(!a1 ⊸ 0) ⊢ 0      a1 ⊢ a1        !a1 ⊢ !a1      0 ⊢ 0
─────────────────────────────────── ⊸L   ──────── !L    ──────────────────── ⊸L
  !a1, !a1, !a1 ⊸ !(!a1 ⊸ 0) ⊢ 0         !a1 ⊢ a1         !a1, !a1 ⊸ 0 ⊢ 0
  ────────────────────────────── !c      ───────── !R    ─────────────────── !L
    !a1, !a1 ⊸ !(!a1 ⊸ 0) ⊢ 0            !a1 ⊢ !a1       !a1, !(!a1 ⊸ 0) ⊢ 0
   ──────────────────────────── !L       ─────────────────────────────────── ⊸L
   !a1, !(!a1 ⊸ !(!a1 ⊸ 0)) ⊢ 0            !a1, !a1, !a1 ⊸ !(!a1 ⊸ 0) ⊢ 0
   ───────────────────────────── ⊸R        ────────────────────────────── !c
   !(!a1 ⊸ !(!a1 ⊸ 0)) ⊢ !a1 ⊸ 0             !a1, !a1 ⊸ !(!a1 ⊸ 0) ⊢ 0
  ──────────────────────────────── !R       ──────────────────────────── !L
  !(!a1 ⊸ !(!a1 ⊸ 0)) ⊢ !(!a1 ⊸ 0)          !a1, !(!a1 ⊸ !(!a1 ⊸ 0)) ⊢ 0
  ────────────────────────────────────────────────────────────────────── ⊸L
      !(!a1 ⊸ 0) ⊸ !a1, !(!a1 ⊸ !(!a1 ⊸ 0)), !(!a1 ⊸ !(!a1 ⊸ 0)) ⊢ 0
      ────────────────────────────────────────────────────────────── !c
                !(!a1 ⊸ 0) ⊸ !a1, !(!a1 ⊸ !(!a1 ⊸ 0)) ⊢ 0
               ──────────────────────────────────────────── !L
               !(!(!a1 ⊸ 0) ⊸ !a1), !(!a1 ⊸ !(!a1 ⊸ 0)) ⊢ 0
               ───────────────────────────────────────────── ⊗L
               !(!(!a1 ⊸ 0) ⊸ !a1) ⊗ !(!a1 ⊸ !(!a1 ⊸ 0)) ⊢ 0
             ───────────────────────────────────────────────── ⊸R
             ⊢ (!(!(!a1 ⊸ 0) ⊸ !a1) ⊗ !(!a1 ⊸ !(!a1 ⊸ 0))) ⊸ 0
            ──────────────────────────────────────────────────── !R
            ⊢ !((!(!(!a1 ⊸ 0) ⊸ !a1) ⊗ !(!a1 ⊸ !(!a1 ⊸ 0))) ⊸ 0)
```

</details>

### The nineteen refuted files (header Theorem)

For each of these files the command prints the verdict line

```
unprovable (ILL, intuitionistic, two-sided engine): the search was exhaustive
```

(`"verdict":"unprovable","refutation":"exhausted"` in JSON) and exits
with status 1, except `ILLTP-SYN-cbn/SYN915+1.p`, given below. Each
entry gives the classical reading of the file (axioms ⊢ conjecture, with
`¬X` for `X → false`) and an assignment under which the axioms are true
and the conjecture is false.

- **KLE013+1**, all three translations, `KLE-01/KLE013+1.p` (copy
  bound 1), `KLE-cbn/KLE013+1.p` (bound 0) and `KLE-cbv/KLE013+1.p`
  (bound 1). Classical reading: `a ⊢ ¬(a → b)`. Countermodel: a and b
  true, the only one. The conjecture is `0` behind implications, and no
  axiom can produce `0`. The Maude result files have `false` for the 01
  (68 ms) and call-by-value (60 ms) translations, which agrees, and
  `timeout` for call-by-name.
- **SYN041+1**, all three translations, `ILLTP-SYN-01/SYN041+1.p`,
  `ILLTP-SYN-cbn/SYN041+1.p` and `ILLTP-SYN-cbv/SYN041+1.p` (bound 0
  each). Classical reading: `⊢ ¬((p → q) → (q → p))`. Countermodel: p
  and q false (any assignment but p false and q true). The Maude result
  files have `timeout` for all three.
- **SYN915+1**, `ILLTP-SYN-cbn/SYN915+1.p` only. The conjecture is the
  atom `T`. The command reads the file as a sequent of the
  multiplicative fragment and answers with the net engine:
  `unprovable (IMLL, intuitionistic, net engine): T occurs 1 more time
  than ~T in the one-sided sequent, so they cannot all meet in axioms`
  (`"refutation":{"unbalanced":{"atom":"T","least":1,"most":1}}` in
  JSON). Countermodel: T false. The Maude result file has `false`
  (18 ms), which agrees. The other two translations have the conjecture
  `1` and are right.
- **KLE017+1**, `KLE-cbn/KLE017+1.p` and `KLE-cbv/KLE017+1.p` (copy
  bound 5 each). Classical reading: `¬(a → b) ⊢ ¬(b → ¬¬a)`.
  Countermodel: a true and b false, the only one. The Maude result files
  have `timeout`.
- **KLE069+1**, all three translations, `KLE-01/KLE069+1.p` (copy
  bound 13), `KLE-cbn/KLE069+1.p` (bound 10) and `KLE-cbv/KLE069+1.p`
  (bound 5). Classical reading: `⊢ (a ∨ b) → ¬¬(a ∧ ¬b)`.
  Countermodel: a false and b true (or both true). The Maude result files
  have `timeout`.
- **KLE078+1**, `KLE-cbn/KLE078+1.p` (copy bound 8) and
  `KLE-cbv/KLE078+1.p` (bound 6). Classical reading:
  `⊢ (a → b) → ¬¬¬(a ∨ b)`. Countermodel: a false and b true (or both
  true). The Maude result files have `timeout`.
- **KLE086+1**, `KLE-cbn/KLE086+1.p` (copy bound 9). Classical reading:
  `⊢ ¬(((a ∨ b) → ¬¬(a → b)) ∧ (¬¬(a → b) → (a ∨ b)))`. Countermodel:
  a false and b true (or both true). The Maude result file has
  `timeout`.
- **KLE088+1**, `KLE-cbn/KLE088+1.p` (copy bound 9). Classical reading:
  `⊢ ¬(((a ∨ b) → ¬(a ∧ ¬b)) ∧ (¬(a ∧ ¬b) → (a ∨ b)))`. Countermodel:
  a false and b true (or both true). The Maude result file has
  `timeout`.
- **SYJ103+1**, `ILLTP-SYJ-cbv/SYJ103+1.p` (copy bound 8). Classical
  reading: `¬(a ∨ ¬b) ⊢ ¬(b ∨ ¬a)`. Countermodel: a false and b true,
  the only one. The Maude result file has `timeout`.
- **SYJ105+1.003**, `ILLTP-SYJ-cbv/SYJ105+1.003.p` (copy bound 6).
  Classical reading: `⊢ ¬¬((a ∧ b) ∨ ¬(a ∨ ¬b))`. Countermodel: a and b
  false (or a true and b false). The Maude result file has `timeout`.
- **SYJ105+1.004**, `ILLTP-SYJ-cbv/SYJ105+1.004.p` (copy bound 6).
  Classical reading: `⊢ ¬¬((a ∧ b ∧ c) ∨ ¬(a ∨ ¬(b ∨ ¬c)))`.
  Countermodel: a and b false and c true (three more exist). The Maude
  result file has `timeout`.

## Twenty-one further files with a wrong header

The same translator fault makes the header of 21 more files certainly
wrong: each says Theorem, and each has a classical countermodel. linlog
does not decide them: after its 2 s the command reports `unknown`
(exit status 3), having deepened the copy bound to between 18 and about
1.9 million without reaching a level that needs no more copies. The
Maude result files have `timeout` for all 21. The countermodels are:

| File | Countermodel |
|---|---|
| `KLE-01/KLE015+1.p`, `KLE-cbn/KLE015+1.p`, `KLE-cbv/KLE015+1.p` | a, b false |
| `KLE-01/KLE074+1.p`, `KLE-cbn/KLE074+1.p`, `KLE-cbv/KLE074+1.p` | a, b false |
| `KLE-01/KLE083+1.p`, `KLE-cbn/KLE083+1.p`, `KLE-cbv/KLE083+1.p` | a false, b true |
| `KLE-01/KLE017+1.p` | a true, b false |
| `KLE-01/KLE078+1.p` | a false, b true |
| `KLE-01/KLE086+1.p`, `KLE-cbv/KLE086+1.p` | a false, b true |
| `KLE-01/KLE088+1.p`, `KLE-cbv/KLE088+1.p` | a false, b true |
| `ILLTP-SYJ-01/SYJ103+1.p`, `ILLTP-SYJ-cbn/SYJ103+1.p` | a false, b true |
| `ILLTP-SYJ-01/SYJ105+1.003.p`, `ILLTP-SYJ-cbn/SYJ105+1.003.p` | a, b false |
| `ILLTP-SYJ-01/SYJ105+1.004.p`, `ILLTP-SYJ-cbn/SYJ105+1.004.p` | a, b false, c true |

For example, `KLE015+1` is `a => b ⊢ ~b => ~a` in ILTP, and the
translator read its conjecture as `~(b => ~a)`, which is false when a
and b are.

Beyond these, 101 of the 362 problems under `ILTP+KLE/` are read
differently by the translator than TPTP means them, in all three
translations each (303 files):

- KLE013, 015, 017, 018, 028, 030, 031, 038, 058, 059, 060, 064, 069,
  070, 071, 072, 074, 075, 076, 077, 078, 079, 082, 083, 084, 085, 086,
  087 and 088;
- LCL181+1; SYJ103+1, SYJ105+1.003 and .004, SYJ106+1, SYJ209+1.001 to
  .020, SYJ211+1.001 to .020 and SYJ212+1.001 to .020;
- SYN001+1, SYN040+1, SYN041+1, SYN046+1, SYN047+1, SYN391+1 and
  SYN392+1.

Of these, 13 problems headed Theorem (the 39 files above) are refuted
by classical countermodels, and two headed Non-Theorem (SYN001+1 and
SYJ212+1.001, six files) are proved. In the other 86, each file states
a formula other than the one its header describes, so the header is
right only by chance. Where the header says Non-Theorem and the
translation is classically invalid (KLE085, LCL181, SYJ211+1.001 to
.006, SYN040, SYN046, SYN047 and SYN392), it stays right. Where the
translation is classically valid, which holds for the 21 other problems
headed Theorem and for SYJ209+1.001 to .010 and SYJ212+1.002 to .016
headed Non-Theorem, only an intuitionistic prover can tell. The rest
are headed Unsolved or have more atoms than a truth table can take
here. Generating the translations again after fixing the grammar
settles all of them at once.

linlog's own translations of the ILTP originals (step 25, `linlog
--logic`) confirm the list for the 72 ILTP problems among the 101: in each
of the three translations the image differs from LLTP's file exactly on
those 72, on SYN915+1 under cbn (the atom `T`), and on SYN977+1, where
only the grouping of `a | b | c` differs, without changing the formula's
meaning. On the misread problems that both sides decide at 2 s, linlog's
verdicts on its own images agree with the ILTP statuses where LLTP's
files contradict them (`plan/reports/25-ordinary-logic.md`).

## The malformed file

`ILLTP-SYJ-01/SYJ206+1.018.p` has a tab character where a closing
parenthesis belongs: at byte offset 8 773 422 of the file (counting from
0), on line 22 at column 8 772 494, in `!((!(a1<TAB> -o !(a2)))))))`. It
is the only tab in the library, and the file does not parse: linlog
stops with "clause `con1` is not closed" (exit status 2). Replacing the
tab by `)` gives a well-formed problem whose size continues its family:
each size from 15 to 19 has twice the occurrences of the previous one
plus 34. The file's SHA-256 at the commit above is
`a00d9154595c7b78f466e92d4731a571ebd90d027116d296e93c5085fdc53785`.

## Reproducing

A proof file is checked again, and its derivation printed, with

```sh
linlog check -i --tree always lltp-headers/KLE-cbn_KLE065+1.json
```

which prints `valid proof of ⊢ … (intuitionistic)` and exits with status
0. The JSON holds the sequent, in linlog's one-sided form, and the
proof's rule applications; the check reads both and does not use the
search. A countermodel needs no tool: evaluate the classical reading
of the file under the assignment.
