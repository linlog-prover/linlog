---
paths:
  - "core/src/lltp.rs"
  - "core/src/families.rs"
---

# linlog core: the benchmark inputs

Loaded, beside `core.md`, when the LLTP reader or the generated
families are read; the harness that runs them is `bench.md`'s.

## Benchmark inputs: LLTP and the families

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
