# Changelog

Every release of the crates `linlog` (the library) and `linlog-cli` (the
`linlog` command), newest first. The form is that of
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the
versions follow [Semantic Versioning](https://semver.org/) as Cargo reads
it for versions below 1.0: a change that breaks a caller raises the
second number (0.1.x to 0.2.0), anything else the third.

## [Unreleased]

## [0.1.0] - unreleased

The first release.

### What is promised

- **The Rust API of `linlog`** follows semantic versioning from this
  release on. Every public enum and every struct with public fields
  that may grow is `#[non_exhaustive]`, so a new variant or field is no
  break. The enums of the calculus are closed on purpose, so that a
  `match` downstream stops compiling when the calculus grows instead of
  falling into a wildcard arm: `Term`, `Kind`, `Node`, `Rule`,
  `Verdict`, `Branch`, `Sign`, `Polarity`, `Side` and
  `ordinary::{Node, Rule, Side}`. Cut and, later, the quantifiers will
  add variants to them, each in a release that raises the second
  number.
- **The cargo features** of `linlog` are `parse`, `serialize`,
  `interactive`, `latex`, `typst`, `svg` and `rocq` (on by default) and
  `parallel`, `png` and `pdf` (off). None is removed within 0.1.
- **The JSON forms** are promised by their wire level, not by the
  crate's version: every top-level document carries `"version": n`, a
  reader reads every level up to its own and refuses a later one by
  name, and a new level is a line of this file under "Wire forms",
  never by itself a break of the crates. This release writes and reads
  level 1.
- **The minimum supported Rust version** is 1.92 (`rust-version`), for
  both crates with every feature; raising it is a new 0.y version and a
  line of this file.
- **The command** keeps its subcommands, flags, exit statuses and JSON
  output (`--format json`) within 0.1; its text for people (verdict
  lines, explanations, help) may change in any release. The library
  `linlog_cli` inside the package `linlog-cli` is the binary's and no
  API.
- **Not promised**: the text of `Display` and of error messages (their
  `code` is), the defaults' numbers (`Limits::DEFAULT_*`, the clock's
  defaults), the engines' counters, which engine decides a sequent, and
  the time it takes.

### Wire forms

- Level 1: the sequent, the proof, the disproof, the proof structure,
  the interactive session and the outcome (written only), each
  described on its type and in the module `wire`; the options and
  settings values, which refuse a key they do not know. The repository
  keeps a document of every form as this release writes it, which every
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
