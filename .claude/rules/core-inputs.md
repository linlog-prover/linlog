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
  conjecture` by assembling text for the crate's own parser: the
  library's connectives and precedences (`*` over `|` over `&` over `+`
  over `-o`, prefix `!`/`?`, postfix `^`) are this crate's, checked on
  every mixed-operator formula of the library. Lines from `%` on are
  comments; the status (`Status::Theorem`, or `NonTheorem` for
  `Non-Theorem` and `CounterSatisfiable`) is the first `Status (intuit.)`
  or `Status (linear)` comment's, else the first plain one's, because the
  translated ILLTP problems carry the classical source's `Status` first
  (39 files, the excluded middle among them, would read as theorems);
  roles other than `axiom`, `hypothesis` and `conjecture`, an
  annotation after the formula, an empty formula (it read as an empty
  succedent and was decided) and a second conjecture
  (`Error::SeveralConjectures`, naming it; joined right of `⊢` the two
  read as their par) are refused, in `lltp::clauses`, which
  `ordinary::read_tptp` shares. A `-` between two name
  characters is part of the name unless it starts `-o` (and `-o` between
  two name characters, `a-ob`, is refused: a name to the library's
  convention and `a ⊸ b` to this crate's, H6; no file of the LLTP or
  ILTP libraries has one) and becomes
  `lltp::HYPHEN` (`‿`), a `.` there becomes `lltp::DOT` (`·`), since the
  Petri nets name places `P-start_1_1` and `merge.s00001061.input` and
  this crate's identifiers hold neither. The mode
  is not in the file: the caller decides (the harness by the `ILL`
  directory). A header's status is the library's claim, not a fact: the
  statuses of translated problems are those of the intuitionistic source.
- **`mist::read`** (feature `parse`) turns a coverability problem in
  Mist's `.spec` format into a Horn program for affine mode, built as an
  arena (`Sequent::from_parts`, then `optimize`, so it is shared as the
  parser leaves a sequent; the one-sided form the parser would give
  `rules, params, tokens |- goal`, in that order, with every root but the
  goal an antecedent), every token of a counter
  one shared literal: written out as text, a 6 KB file of one long name
  and a count of 49 million asked for 49 GB before any limit (the second
  panel's finding). A rule's guards and updates are merged as sorted
  lists, since one rule may update tens of thousands of counters: a rule `guards -> updates`
  becomes `!(in -o out)`, `in` taking from each counter the most of its
  guard and its decrement (a decrement without a guard still needs the
  tokens: counters do not go below zero) and `out` giving that less the
  decrement plus the increment; an initial `x = k` is `k` tokens, `x >=
  k` is `k` tokens and `!x` (any number more); the target's lines are a
  disjunction of conjunctions, so one line is the goal itself and several
  are clauses `!(line -o goal)` to a fresh atom (`goal`, else `goal_1`,
  …), which in affine mode is covered exactly when a line is. Every form
  of the qcover suite's 176 files is read (guards `x >= k`, updates `x' =
  x ± k`, init `=` and `>=`, `#` comments, a comma at the end of a line
  or at the start of the next joining the two, as the bug-tracking files
  write their initial markings, an `invariants` section, which is
  skipped); rules and targets are kept sparse, since a file declares up
  to 66 950 counters and 213 625 rules; a count of `k` is `k` occurrences
  of an atom, so the tokens are summed before the arena is built and a
  problem with more than the limit is `Refusal::Occurrences`
  (`read(text, &limits)` within `limits.occurrences`, the command
  `--occurrence-limit`): a 40-byte file asked for 20 million tokens,
  617 MiB, before the limit counted (the panel's finding); an update from another counter, a counter updated twice,
  a counter given twice in `init` (`x >= 1, x = 3` was read as `x >= 3`,
  neither constraint), and the names `top` and `bot` (units in this
  syntax) are refused. The
  expected result is the first line's `#expected result: safe|unsafe`,
  which only 12 of the suite's files state (`Safety::Unsafe` is
  provable). The mode is not in the file: a coverability question is
  affine, the harness runs it intuitionistic affine, and the command
  makes the flags' mode affine for every `.spec` input.
- **`families`** (feature `parse`): `FAMILIES` lists the benchmark
  families, each a name, a summary, default sizes, instances per size,
  the sizes it takes (`least`, `powers_of_two`) and a generator `(size,
  index) → Instance` (sequent, mode, `provable`, `copies`);
  `Family::instance` refuses a size the generator has no instance of
  with `Error::FamilySize` before it runs (a generator asserts its
  precondition, which a caller's size reached as a panic, F6), and
  generates without an occurrence bound, a large size being the
  caller's to ask for. A family's `provable` comes from the problem it encodes
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

## Decisions

The author's answers for the release (`plan/notes/api.md` §14), which
the fixes implement and later rounds judge against. Where a bullet above
still describes code that a decision changes, the decision holds, and
the commit that lands it rewrites that bullet.

- **A file with several conjectures is refused**, naming the second (the
  LLTP reader, as `ordinary::read_tptp`). No file of the libraries has
  several, and conjoining them would carry TPTP's meaning into linear
  logic by a guess.
- **A `.spec` file is affine, from the file**: its question is
  coverability, and a mode flag that contradicts it is refused.
