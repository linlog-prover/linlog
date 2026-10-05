---
paths:
  - "core/src/lltp.rs"
  - "core/src/mist.rs"
  - "core/src/families.rs"
---

# linlog core: the benchmark inputs

Loaded, beside `core.md`, when the LLTP reader, the `.spec` reader or
the generated families are read; the harness that runs them is
`bench.md`'s.

## Benchmark inputs: LLTP, `.spec` and the families

- **`lltp::read`** (feature `parse`) turns an LLTP file into `axioms ⊢
  conjectures` by assembling text for the crate's own parser: the
  library's connectives and precedences (`*` over `|` over `&` over `+`
  over `-o`, prefix `!`/`?`, postfix `^`) are this crate's, checked on
  every mixed-operator formula of the library. Lines from `%` on are
  comments; the status (`Status::Theorem`, or `NonTheorem` for
  `Non-Theorem` and `CounterSatisfiable`) is the first `Status (intuit.)`
  or `Status (linear)` comment's, else the first plain one's, because the
  translated ILLTP problems carry the classical source's `Status` first
  (39 files, the excluded middle among them, would read as theorems);
  roles other than `axiom`, `hypothesis` and `conjecture`, and an
  annotation after the formula, are refused. A `-` between two name
  characters is part of the name unless it starts `-o` and becomes
  `lltp::HYPHEN` (`‿`), a `.` there becomes `lltp::DOT` (`·`), since the
  Petri nets name places `P-start_1_1` and `merge.s00001061.input` and
  this crate's identifiers hold neither. The mode
  is not in the file: the caller decides (the harness by the `ILL`
  directory). A header's status is the library's claim, not a fact: the
  statuses of translated problems are those of the intuitionistic source.
- **`mist::read`** (feature `parse`) turns a coverability problem in
  Mist's `.spec` format into a Horn program for affine mode, by
  assembling text for the crate's parser: a rule `guards -> updates`
  becomes `!(in -o out)`, `in` taking from each counter the most of its
  guard and its decrement (a decrement without a guard still needs the
  tokens: counters do not go below zero) and `out` giving that less the
  decrement plus the increment; an initial `x = k` is `k` tokens, `x >=
  k` is `k` tokens and `!x` (any number more); the target's lines are a
  disjunction of conjunctions, so one line is the goal itself and several
  are clauses `!(line -o goal)` to a fresh atom (`goal`, else `goal_1`,
  …), which in affine mode is covered exactly when a line is. Every form
  of the qcover suite's 176 files is read (guards `x >= k`, updates `x' =
  x ± k`, init `=` and `>=`, `#` comments, an `invariants` section, which
  is skipped); an update from another counter, a counter updated twice,
  and the names `top` and `bot` (units in this syntax) are refused. The
  expected result is the first line's `#expected result: safe|unsafe`,
  which only 12 of the suite's files state (`Safety::Unsafe` is
  provable). The mode is not in the file: a coverability question is
  affine, and the harness runs it intuitionistic affine.
- **`families`** (feature `parse`): `FAMILIES` lists the benchmark
  families, each a name, a summary, default sizes, instances per size and
  a generator `(size, index) → Instance` (sequent, mode, `provable`,
  `copies`). A family's `provable` comes from the problem it encodes
  (subset sums, QBF evaluation, 3-Partition by construction) or from a
  construction argument written at the generator, never from an engine,
  so that an engine disagreeing is a finding. Random families seed
  SplitMix64 from the size and index. The encodings are public
  (`three_partition`, `three_partition_mll`, `partition`, `qbf`,
  `counter`, `wide`, `mix`), and the engines' tests use them instead of
  private copies. The QBF encoding sequences quantifiers with key atoms:
  `∃x` is `((~tx ⅋ ~kx) ⊕ (~fx ⅋ ~kx)) ⅋ (kx ⊗ S)`, so the rest `S` can
  only be focused once the choice released `~kx`; `∀x` is `(~tx & ~fx) ⅋
  S`; a clause is the `⊕` of `(tx ⊗ ⊤)`/`(fx ⊗ ⊤)`, and the matrix their
  `&`. `mix` wraps each tensor pair in `⊕ 0` because in MLL the count
  equation refutes the bare pairs at once.
