# Changelog

Every release of the crates `linlog` (the library) and `linlog-cli` (the
`linlog` command), newest first. The form is that of
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the
versions follow [Semantic Versioning](https://semver.org/) as Cargo reads
it for versions below 1.0: a change that breaks a caller raises the
second number (0.1.x to 0.2.0), anything else the third.

## [Unreleased]

### Added

- The Rocq library `rocq-linlog` under `rocq/` (logical path `Linlog`,
  Rocq 9 with its standard library and nothing else): formulas in
  negation normal form over an abstract atom type, the one-sided cut-free
  calculus of classical linear logic as a plain inductive with binary Mix
  and general weakening as parameters, linlog's proof term as a datatype,
  a checker as a function and the theorem that a term the checker accepts
  proves its sequent, closed under the global context. The flake builds
  it as a package and compiles the certificates under `rocq/tests/`
  against it.
- In the Rocq library, the theorems for Mix and for affine logic
  (`soundness_mix`, `soundness_affine`), and the two-sided calculus of
  intuitionistic linear logic and its affine variant (`ill`, over
  `iformula` named as in Yalla's `nanoill.v`), with two-sided proof terms,
  their checker and the theorem `soundness_ill`, each closed under the
  global context.
- A second Rocq kernel in the exporter, linlog's own library:
  `rocq::Kernel` (`NanoYalla`, `Linlog`), `rocq::Options::kernel` (JSON
  `styles.rocq.kernel`, `"nanoyalla"` or `"linlog"`) and the command's
  `--kernel`, with the version `rocq::LINLOG` and the preludes
  `LINLOG_PRELUDE` and `LINLOG_TWO_SIDED_PRELUDE`. A certificate for it
  states the sequent in the mode's calculus, two-sided for an
  intuitionistic derivation, and proves it from the proof term; Mix and
  affine weakening have one. The flake's `rocq-linlog` check compiles
  the exporter's certificates for it.
- In the Rocq library, the theorems `mix_to_ll` and `mix_to_classical`
  (`Mix.v`): a derivation with binary Mix gives one of `?(⊥⊗⊥), Γ`
  without Mix and without cut. Beside the library, under `rocq/bridges/`,
  the equivalences of `ll classical` with NanoYalla's `nanoll.ll` and of
  `ill intuitionistic` with Yalla's `nanoill.ill`, which the flake's new
  `rocq-bridges` check compiles against the two kernels' files as its
  inputs pin them (Yalla is a new input), each closed under the global
  context.
- In the Rocq library, the refutations (`Refutations.v`): every
  derivable sequent of `ll R`, for every `R`, reads as a tautology
  classically (`ll_tautology`); without weakening an atom's excess over
  the additive alternatives contains zero (`balance`) and a
  multiplicative sequent with units has the formulas the count equation
  asks for, at least that many with Mix (`count_equation`); and the
  two-sided calculus embeds in the one-sided one (`ill_ll`), so that
  each gives `~ ll R Γ` or `~ ill R Γ C` from an assignment, an atom or
  nothing, tested by computation (`classical_refutation`,
  `unbalanced_refutation`, `equation_refutation` and their two-sided
  forms), all closed under the global context.
- A classical refuter after the search: where a search did not prove its
  goal, an assignment of truth values to the atoms under which the goal
  read classically is false (`!` and `?` erased, `⊗` and `&` as "and",
  `⅋` and `⊕` as "or", `1` and `⊤` true, `⊥` and `0` false), which no
  provable sequent has in any mode, makes an unknown verdict unprovable
  and gives an exhausted search a certificate: the refutation
  `Refutation::Classical` (`search::Classical`, its `assignment` a value
  per atom), bounded by `search::Options::classical_work` (JSON
  `classical_work`, default `Options::DEFAULT_CLASSICAL_WORK`, 2²⁶ terms
  evaluated, zero for none) and the memory bound, and the command's
  `--classical-work N`, which the harness takes too. No verdict a search
  gave changes, nor its counters.
- `Disproof::check`, which verifies a classical assignment, an
  unbalanced atom and the count equation against the sequent, the goal
  and the mode, sharing no code with the search, and its error
  `Error::Refutation` with `search::RefutationFault` (codes
  `invalid_refutation`, kind invalid, and `uncertified`, kind
  unsupported, for the refutations that rest on the search). The
  harness writes its result in the `checked` column of an unprovable
  row.

- Certificates of non-derivability in the exporter, for linlog's
  library: `rocq::write_disproof` writes, for a disproof whose refutation
  `Disproof::check` accepts, the lemma `~ ll R Γ` (or `~ ill R Γ C` in
  intuitionistic mode) proved by the library's refutation theorem by
  computation, with the prelude `rocq::LINLOG_REFUTATION_PRELUDE`, and
  `rocq::write_not_valid` the lemma `~ (forall a b : Prop, F)` for an
  ordinary sequent whose image an assignment refutes, by instantiating
  its atoms; `rocq::Unsupported` gains `Disproof` (NanoYalla named) and
  `NotValid`. The command's `--format rocq` writes them for an
  unprovable or not valid verdict, and says on standard error where
  there is none.

### Wire forms

- Level 2: the refutation kind `classical`, `{"kind": "classical",
  "assignment": [true, false, …]}`. A disproof or an unprovable outcome
  that holds one is written at level 2; every other document stays at
  level 1, byte for byte.
- The essential-net engine for IMLL without units, `Engine::Essential`
  (`--engine essential`, JSON `"essential"`): the net engine's linking
  search over the essential net the intuitionistic reading directs,
  keeping directed acyclicity by a transitive closure of bits with an
  undo log, counted under the memory bound, and testing the dominator
  condition; it runs only when forced. `ProofStructure::is_essential`
  decides that criterion independently of any search, with the
  witnesses `NetError::DirectedCycle` and `NetError::NoDominator` (and
  `NetError::NotIntuitionistic` for a sequent without a reading), and
  `ProofStructure::reading` gives a structure's reading.
- The drawing of an essential net in the SVG export: `svg::Style`'s
  `essential` (`--style svg.essential=true`, off by default) draws the
  net of an intuitionistic sequent with every edge an arrow from the
  hypotheses toward the goal, the input side in the colour `input`, the
  nodes as `⊸` and `⊗`, each implication joined to its antecedent by a
  dotted line in the colour `dominator`, and the essential criterion's
  witness highlighted.

### Changed

- The command refutes by the classical reading by default: a sequent
  whose search ended unknown (at the time limit, a copy bound or another
  limit) and whose classical reading an assignment falsifies is
  unprovable (exit status 1); `--classical-work 0` gives the search's
  word alone. The refutation runs after the search, past the time limit,
  within its bound of work, and Ctrl-C stops it.

- With no kernel named, `--format rocq` writes a proof of Mix, affine or
  intuitionistic logic for linlog's library, as the mode chooses; a
  classical proof is written for NanoYalla exactly as before, and
  `--kernel nanoyalla` keeps the NanoYalla certificate of an
  intuitionistic proof and the refusal of Mix and affine weakening.
- Unit-free MLL and IMLL go to the focused engines by default, which
  measured faster than the net engine on every sequent both decide
  (`wide` sequents 1.07 to 3.6 times, the Horn encodings of Partition
  and 3-Partition up to 32 000 times); the net engine takes a sequent
  whose atoms occur once with each sign over where they stop at their
  recursion limit or memory bound (a chain
  of thousands of implications, `wide` with 4 096 literals). Before, MLL
  with no literal more than twice went to the net engine.
- The net engine breaks the symmetry of equal literals and equal
  subformulas that are conclusions or leaves of one tree of `⊗` alone
  or `⅋` alone, and checks each atom's balance over its skeleton's
  components: Lincoln's MLL 3-Partition with bins of five is refuted in
  227 nodes instead of 16 million, Matsuoka's Partition with three
  items in 391 instead of 269 950.

## [0.1.0] - 2026-10-18

The first release.

### What is promised

- **The Rust API of `linlog`** follows semantic versioning from this
  release on. Every public enum and every struct with public fields
  that may grow is `#[non_exhaustive]`, so a new variant or field is no
  break. The enums of the calculus are closed on purpose, so that a
  `match` downstream stops compiling when the calculus grows instead of
  falling into a wildcard arm: `Term`, `Kind`, `Node`, `Rule`,
  `search::Verdict`, `Branch`, `Sign`, `occurrences::Polarity`, `Side`,
  `ordinary::{Node, Rule, Side}` and `sequents::Visit`. Cut and, later,
  the quantifiers will add variants to them, each in a release that
  raises the second number.
- **The cargo features** of `linlog` are `parse`, `serialize`,
  `interactive`, `latex`, `typst`, `svg` and `rocq` (on by default) and
  `parallel`, `png` and `pdf` (off). None is removed within 0.1.
- **The JSON forms** of the data (sequents, proofs, disproofs, proof
  structures, sessions, outcomes) are promised by their wire level, not
  by the crate's version: every such document carries `"version": n`, a
  reader reads every level up to its own and refuses a later one by
  name, and a new level is a line of this file under "Wire forms",
  never by itself a break of the crates. This release writes and reads
  level 1. The options and settings values carry no version and refuse
  a key they do not know, so a later release reads this one's settings,
  and a settings file that uses a later key is refused by name here.
- **The minimum supported Rust version** is 1.92 (`rust-version`), for
  both crates with every feature; raising it is a new 0.y version and a
  line of this file.
- **The command**: within 0.1 its subcommands, flags and exit statuses
  are only added to, and its JSON output (`--format json`) follows the
  wire levels, so a 0.1.x release may write a new level (a new kind of
  refutation, say); its text for people (verdict lines, explanations,
  help) may change in any release. The library
  `linlog_cli` inside the package `linlog-cli` is the binary's and no
  API.
- **Not promised**: the text of `Display` and of error messages (their
  `code` is), the defaults' numbers (`Limits::DEFAULT_*`, the clock's
  defaults), the engines' counters, which engine decides a sequent, and
  the time it takes.

### Wire forms

- Level 1: the sequent, the proof, the disproof, the proof structure,
  the interactive session and the outcome (written only), each
  described on its type and in the module `wire`. The repository keeps
  documents of these forms as this release writes them, which every
  later release must read.

### Added

- Sequents of classical and intuitionistic linear logic, with or without
  units, exponentials, Mix and weakening (affine mode), in a text syntax
  with ASCII and Unicode spellings and a compact JSON form; detection of
  the fragment, from MLL to LL and IMLL to ILL.
- Proof search for every fragment, returning a proof the independent
  checker has validated, "unprovable" after an exhaustive search (with
  the reason where the sequent's counts tell), or "unknown" with the
  bound or limit that ended it: a focused sequent engine, a proof-net
  engine for MLL, a recursion for additive sequents and an engine for
  Horn programs under `!` (Petri nets), chosen by fragment and mode; a
  thread pool behind the feature `parallel`; stop conditions, memory
  and occurrence limits honoured everywhere a search spends time or
  memory.
- Proofs as terms over subformula occurrences, their checker, and the
  derivation view of the one-sided and two-sided sequent calculus.
- Proof nets for MLL with or without Mix: the Danos–Regnier criterion,
  sequentialization and desequentialization.
- Interactive proving: rules applied to the goals of a derivation in
  progress, undo, and the search closing a goal.
- Exports of sequents, derivations and proof nets to LaTeX, Typst and
  SVG, rendered as PNG and PDF behind the features `png` and `pdf`, and
  of finished proofs to Rocq scripts for the NanoYalla kernel.
- Ordinary classical, intuitionistic and minimal propositional logic,
  decided through its translations into linear logic, with the proof
  read back as a checked derivation of LK or LJ, and a reader for TPTP
  problems.
- Readers for the LLTP library's problems and for coverability problems
  in the `.spec` format, and generated problem families with known
  verdicts.
- The command `linlog`: `prove` (one sequent, or many from files,
  directories and standard input), `check`, `interact` and
  `seq print|json|fragment`, with a time limit of two seconds by
  default, Ctrl-C, JSON output and every export. Its exit status is 0
  for proved or valid, 1 for unprovable or invalid, 2 for an error, 3
  for unknown and 130 at a second Ctrl-C.

[Unreleased]: https://github.com/linlog-prover/linlog/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/linlog-prover/linlog/releases/tag/v0.1.0
