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

### Changed

- With no kernel named, `--format rocq` writes a proof of Mix, affine or
  intuitionistic logic for linlog's library, as the mode chooses; a
  classical proof is written for NanoYalla exactly as before, and
  `--kernel nanoyalla` keeps the NanoYalla certificate of an
  intuitionistic proof and the refusal of Mix and affine weakening.
- Unit-free MLL and IMLL go to the focused engines by default, which
  measured faster than the net engine on every sequent both decide
  (`wide` sequents 1.07 to 3.6 times, the Horn encodings of Partition
  and 3-Partition up to 30 000 times); the net engine takes a sequent
  over where they stop at their recursion limit or memory bound (a chain
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
