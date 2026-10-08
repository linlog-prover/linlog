# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

<!--
Maintainer note (stripped before it reaches Claude's context). This file is
loaded into every session: keep it well under 200 lines, to commands, rules,
conventions and pointers. Guidance for one area of the tree goes in
`.claude/rules/<area>.md` behind a `paths:` glob; procedures go in
`.claude/skills/<name>/`. `.claude/rules/claude-infra.md` explains the split.
-->

## Project

linlog is a linear logic suite: it decides, checks, proves step by step
and exports sequents of classical and intuitionistic linear logic, and
decides ordinary propositional logic through its embeddings (README's
"What exists and what is planned"). It is written in Rust with
WebAssembly in mind for a planned `linlog-web` crate: the library reads
no clock and uses threads only behind a feature. Performance is a stated
goal: the data structures are compact and cache-friendly.

Workspace crates:
- `core/` is package **`linlog`**: all logic. It has seven optional
  default features, `parse` (the text parser, with unicode-ident for the
  identifiers, and the LLTP reader and the generated families),
  `serialize` (serde), `interactive` (step-by-step proving), and
  `latex`, `typst`, `svg` and `rocq` (the exports), and three off by
  default, `parallel` (rayon: the search on a thread pool, off for
  wasm), `png` (resvg) and `pdf` (krilla), which render the SVG with
  fonts the caller gives; the CLI enables the last eight.
- `cli/` is package **`linlog-cli`**, library **`linlog_cli`** and binary
  **`linlog`** (one call into the library; `doc = false` because it shares
  the core crate's name): `prove` (one sequent, or a batch of many),
  `check`, `interact` and `seq print|json|fragment`. README's usage section shows every command,
  format and flag with its output, and `cli/tests/readme.rs` runs those
  examples; `linlog <command> --help` is the doc comments of
  `cli/src/argument_parsing.rs`. Exit status 0 proved/valid, 1
  unprovable/invalid, 2 error, 3 unknown.
- `bench/` is package **`linlog-bench`** (not published, `doc = false`):
  the benchmark harness, the baselines under `bench/results/DAY/`
  (`bench/RESULTS.md`, `bench/COMPARISON.md`) and the focused engine's
  target set (`bench/targets.sh`, `bench/TARGETS.md`).

`plan/README.md` is the proof-search plan the code follows, `plan/reports/`
what each step of it did.

## Where the rules are

A file under `.claude/rules/` loads when a file under its `paths` is
read, and holds that area's invariants, the reasons behind them, what a
check cannot catch, and where a new engine, format, family or option
plugs in. Read it before changing the area, and record there, one point
per bullet, what a later session must know and cannot see in the code.

| rules file | loads for | holds |
|---|---|---|
| `core.md` | `core/**` | the crate's layout and API in brief, its crate-wide rules, and the table of the thirteen module files `core-*.md` (sequents and parsing, the forest, proofs and the checker, derivations and interactive proving, the search's front door, the focused engine, the Horn engine, the parallel runtime, proof nets and the net engine, the exports, the batch, the benchmark inputs, ordinary logic), each loaded for its own module |
| `cli.md` | `cli/**` | the command's layout, exit statuses, defaults, time limit, outputs, README's examples as a test, extension points |
| `bench.md` | `bench/**` | the harness, the CSV columns as its interface, the baselines, the target set, extension points |
| `flake.md` | `flake.nix`, `flake.lock`, `modules/**` | what each flake module holds and checks |
| `evidence.md` | `mutants/**`, `fuzz/**`, `.cargo/mutants.toml` | the mutation run's scope and passes, the fuzz targets, their seeds and runs |
| `ci.md` | `.github/**` | how the workflows are written and pinned |
| `claude-infra.md` | `.claude/**`, `CLAUDE.md` | this Claude Code setup |

## Commands

The dev environment is the flake's devshell (direnv, or `nix develop`): the
toolchain `rust-toolchain.toml` names (edition 2024; rustfmt, clippy and
rust-analyzer included), cargo-hack, cargo-deny, cargo-mutants,
cargo-nextest, valgrind, bacon and treefmt; the `fuzz` devshell has a nightly
compiler and cargo-fuzz for `fuzz/` alone. `menu`
lists its commands: `check`, `gate`, `tests`, `ratchet`, `launch` (the CLI),
`live` (bacon), `dev` (zellij) and `up`.

```sh
cargo build
cargo test --workspace
cargo test -p linlog <test_name>                           # single test in core
cargo clippy --workspace --all-targets -- --deny warnings
cargo hack check --each-feature -p linlog                  # each feature alone, all, none
cargo hack check --feature-powerset --depth 2 -p linlog    # and every pair
cargo deny check                                           # licenses, bans, sources + advisories (online)
cargo run -p linlog-cli -- <args>
cargo run --release -p linlog-bench -- run --family partition-no=3,4 --engines focus,net
nix build .#lltp -o bench/lltp   # the LLTP library (1.1 GB, GPL-3.0, fetched at a pinned commit)
nix build .#iltp -o bench/iltp   # the ILTP library's 274 propositional problems (no licence stated, fetched at v1.1.2)
nix build .#qcover -o bench/qcover   # the qcover coverability suite's 176 .spec problems (no licence of their own, fetched at a pinned commit); linlog-bench run --spec
bench/targets.sh LABEL            # the focused engine's target set into bench/targets/LABEL.csv (two cores, about 20 min, detached)
bench/baseline.sh --arm --fresh   # the whole baseline, unattended from 20:00 to 07:00 (about 11.5 h, so it may need a second night): bench/results/DAY/, bench/RESULTS.md

nix flake check   # build, clippy, test, test-debug-assertions (the tests with debug_assert! on, which the release profile drops), doc, deny, features (cargo-hack), export (the LaTeX and Typst output compiles, the SVG renders), rocq (NanoYalla checks the certificates), bench (the harness on the smallest problems), ratchet (the journeys' instruction counts against their ceilings), deadnix, actionlint, treefmt, claude-hooks
nix build .#checks.x86_64-linux.rocq   # the certificates alone: Rocq is a 1.2 GB closure from the binary cache
gate              # what every commit that touches code passes: clippy, the tests (the behaviour lock among them), both cargo hack runs, the ratchet (about 4 min on four cores)
ratchet           # the journeys' instruction counts under callgrind against bench/ceilings.csv; ratchet --lower after a count went down
mutants/run.sh    # mutation testing of the checker, the readers, the front door, the Horn refutations (cores 6-11, detached): mutants/baseline/
nix develop .#fuzz -c fuzz/run.sh   # every fuzz target until its coverage stops growing (nightly, cores 12-15, detached), after fuzz/seed.sh
nix fmt           # nixfmt, rustfmt, taplo, shfmt, shellcheck (a hook runs it on each file changed with Write or Edit; a file changed from the shell needs it run by hand before the commit, or the treefmt check fails)
nix build         # linlog-cli, whose binary is result/bin/linlog
nix build .#doc   # the rustdoc site, as the Docs workflow publishes it
```

Verify as much as the change needs:

| the change | the proof |
|---|---|
| any commit that touches code | `gate` passes (capped: `systemd-run --user --scope -p MemoryMax=8G -p MemorySwapMax=0 taskset -c 2-5 nix develop -c gate`); a change of the behaviour lock's files or of a ceiling is a commit of its own that says why |
| any `.rs` edit | `cargo clippy …` and `cargo test --workspace` |
| touches `#[cfg(feature = …)]` or `[features]` | add `cargo hack check --each-feature -p linlog` and `cargo hack check --feature-powerset --depth 2 -p linlog` (both cover `parallel`, which is off by default: `--each-feature`'s `--all-features` run is the one that differs from the defaults) |
| touches `bench/` or `core/src/families.rs` | add `cargo run --release -p linlog-bench -- run --all-families --timeout 5` for the verdicts (a `MISMATCH` in `summary` is a bug); timings only from `bench/baseline.sh` on an idle machine |
| changes how the focused engine searches, or must not (a refactoring) | add `bench/targets.sh LABEL` and compare the columns `verdict`, `nodes`, `splits`, `memo_hits` and `memo_entries` of `bench/targets/LABEL.csv` with those of `bench/targets/after-bias.csv` (the script names no `--bias`, so both take the default): on one thread they are a function of the input, so a decided row has them equal exactly when the search is (`linlog-bench summary` of the two files sets the times side by side) |
| changes what the command prints, or a console block of README | `cargo test --workspace` runs README's examples (`cli/tests/readme.rs`); a change of output changes README in the same commit |
| touches `search/parallel.rs`, `focus/parallel.rs` or `net::parallel` | `cargo test --workspace` covers them (the CLI depends on `parallel`, and cargo unifies features across a workspace run); `cargo test -p linlog` alone needs `--features parallel` |
| adds or changes a dependency | add `cargo deny check`. New deps must use a license `deny.toml` allows: EUPL-1.2, MIT, Apache-2.0 (± LLVM-exception), Unicode-3.0, Zlib, BSD-2-Clause or BSD-3-Clause |
| `flake.nix`, `modules/`, `.github/`, the toolchain, a lock bump, or before a push | `nix flake check`, which runs all of the above |

## The flake

Dendritic flake-parts: `flake.nix` only declares inputs, and import-tree
loads every `.nix` file under `modules/` (a path segment starting with `_`
skipped) as a flake-parts module, one aspect per file contributing to
every output it needs; no file is ever added to an imports list.

## CI

`ci.yml` runs `nix flake check` and the online `cargo deny check
advisories` on every push to `main`, every pull request and weekly, with
nothing installed but Nix; `docs.yml` publishes the rustdoc of `main` to
GitHub Pages.

## Version control: jj only

A jj repository, colocated with git only so that nix and the GitHub remote
`origin` (`github.com/linlog-prover/linlog`) keep working. **Never run git, not
even to read.** Every operation goes through `jj`, including lock updates:
`nix … --commit-lock-file` commits through git. The `Bash(git *)` deny rule and
`.claude/hooks/block-git.py` enforce this.

- Reading: `jj st`, `jj log`, `jj diff`, `jj show`, `jj file annotate`, `jj root`.
- **Commit thematically, without being asked**: one logical unit per change,
  committed as soon as it is done with `jj commit -m`, which describes `@` and
  opens a fresh change on top. `jj describe -m` only names `@`; `jj squash`
  folds `@` into its parent (the amend) and, when the parent is described,
  needs `--use-destination-message` or `-m`: without one it opens an editor,
  which hangs a session. Never `jj new -m`: it describes a new,
  empty change and leaves the work behind (a hook blocks it).
- Edits land in `@` retroactively. If `@` already holds unrelated work, run
  `jj new` before editing. Another session or the author may commit in the
  same working copy meanwhile, so run `jj st` right before every commit
  rather than assuming `@` and `@-` from the session's start.
- nix sees only files in the git tree, which any jj command updates: run
  `jj st` after creating a file, before a `nix` command that must see it.
- Subjects are short, imperative and capitalised, with no type prefix
  ("Fix bug in reachability analysis for exponentials"). Add a body only when
  the why is not obvious. jj signs every commit, and a snapshot of a
  changed working copy is one: when the signing key's passphrase is no
  longer cached and nobody answers the prompt, every jj command that
  writes fails with a signing timeout, `jj st` included. The edits stay
  on disk, `jj --ignore-working-copy …` still reads, and `nix flake
  check` runs on the tree as it is; do not retry in a loop, finish what
  needs no commit and ask the author to enter the passphrase. A long
  session commits as it goes rather than at the end for that reason.
- Lock bumps are changes of their own, "Cargo update" and "flake.lock: Update".
  `/update-deps` does both, verifying before it commits; the shell's `up` is
  the unverified shortcut.
- Work lands on `main` directly or through a GitHub pull request. Pushing is
  outward-facing, so only when asked: `jj bookmark set main -r @-`, then
  `jj git push --bookmark main`.
- The GitHub CLI (`gh`) is logged in with the author's own rights, over the
  organization `linlog-prover` and everything else the account reaches.
  Ask before every administrative task through it (a setting of the
  organization or a repository, Pages, Actions, branch rules, secrets,
  releases, a new repository, a workflow run): say what the call does and
  wait for a yes, which covers that task only. Deleting and transferring
  are the author's own acts. A permission rule asks before any `gh`
  command.

## Conventions

- Every source file starts with the license header, in the file's comment
  syntax: `//` in Rust, `#` in Nix, TOML, shell, Python and YAML.
  ```
  // linlog © Fabian Lukas Grubmüller 2026
  // Licensed under the EUPL
  ```
  Prose and configuration carry none: Markdown (CLAUDE.md, `.claude/`,
  `plan/`, README), JSON, `.gitignore`, `LICENSE` and the lock files. The
  project is EUPL-1.2.
- Comments are short and targeted: they say why, or what the code cannot.
  Types and names carry the rest.
- Comments are self-contained: they make sense from inside this repository,
  with no references to other repositories, machines or conversations, and
  none to the Claude Code sessions, prompts, plan steps or decision numbers
  (`plan/`) that produced the code. A comment says what the code does or
  why; how the work was organised belongs in `plan/` and the jj history.
- Every Rust item gets a concise doc comment that a human understands at
  first read: a function says what it does and returns, not how; a type,
  field, variant or module says what it is. The workspace lints in
  `Cargo.toml` enforce it (rustc's `missing_docs` for public items, clippy's
  `missing_docs_in_private_items` for the rest). clap shows the docs on
  `cli/`'s argument types as `--help` text.
- Dependencies are welcome where they earn their place: prefer well-made
  library code over an ad-hoc implementation, and among candidates the more
  popular, better maintained and faster one. Each serves a particular reason
  and is scoped to it: only the crate that uses it, behind the feature that
  needs it (or in `[dev-dependencies]`), with only the crate features used.
  Never add one for its own sake.
- `core/src/lib.rs` allows `dead_code` and `unused_variables` crate-wide:
  some items are live under one feature only or used by tests only, and
  a few are dead; the audit before the release sorts them out.
- `scratchpad*.md` are the author's gitignored notes. `scratchpad1.md` is about
  250 KB, so don't read it in full.

## Claude Code setup

`.claude/` is checked in: the hooks (the git and `jj new -m` guards, the
formatter, a SessionStart note on the jj working copy), permission rules,
the `crate-source-explorer` agent (use it before guessing at a
dependency's API), the `update-deps` and `new-tool` skills (the latter
whenever a crate or tool is added or adopted) and the rules above.
Claude Code's built-in git instructions and status snapshot are off.
When the repo changes shape, amend `.claude/` and this file in the same
change.

## Compact instructions

When the context is compacted, keep the prompt file the session works
from (`plan/NN-*.md`) and where in it the work stands, the report and
checklist it keeps, every instruction from the author or the
supervising session that is not yet in a file, and what was verified
and by what (a killed or partial run is not a result). After a
compaction, read the prompt's section for the current work and the
checklist again before going on.
