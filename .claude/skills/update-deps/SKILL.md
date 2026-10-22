---
name: update-deps
description: >-
  Update every dependency of linlog and of its web client: Cargo.lock
  (`cargo update`), the semver-incompatible crate releases and the exact pins
  (a `Cargo.toml` change, with the code it forces), flake.lock (`nix flake
  update`, which also moves the Rust toolchain), the fuzz workspace's lock and
  nightly, the GitHub Actions' SHA pins, and in the client
  (`linlog-prover.github.io`) npm, its flake inputs and its actions. Verifies
  each with `nix flake check` before committing, commits each as a change of
  its own ("Cargo update", "flake.lock: Update" with nix's summary), and
  reports what it held back and why. Use when asked to update or bump
  dependencies, crates, the lock files, nixpkgs, the flake inputs, the actions
  or the Rust toolchain.
argument-hint: "[cargo|flake|actions|client]"
allowed-tools: Bash(cargo update *) Bash(nix flake update *) Bash(nix develop -c rustc --version) Bash(jj new) Bash(jj restore *) Bash(jj commit *)
---

# Updating every dependency

Scope: `$ARGUMENTS` names one part, or empty for all of them in the order
below. "Update the dependencies" means all of them, in both repositories,
not only what `cargo update` takes.

A version moving is not the interesting part. What matters is whether the
workspace still builds, tests, lints and counts clean with it, and what moved.

## Invariants

- **Nothing unverified enters history.** Update, verify, then commit. Never
  `nix flake update --commit-lock-file`: it commits through git, before
  anything is checked, and a hook blocks it.
- **One change per kind of bump**: a lock change holds its lock file and
  nothing else (`jj commit <lock file> -m …`); a crate's major bump, an exact
  pin, the fuzz nightly, the actions and a fix a bump forces (a new lint, a
  ceiling) are changes of their own on top. A fix that changes behaviour or
  a check's tolerance waits for the author's word.
- **jj only.** Commits are signed by jj itself.
- **Pushing is not part of this** unless the author says so; the client's
  lock may only name a linlog commit on GitHub (below), so its update waits
  for that push.

`$S` below is the session's scratchpad directory: outside the tree, and it
survives a context compaction. Builds run in capped scopes on the cores the
session has; every `nix flake check` and `nix build` under
`flock /tmp/linlog-machine.lock`, detached when it runs long.

## 0. Baseline

```sh
jj st
jj op log --limit 1 --no-graph -T 'id.short(16)'
nix flake check
```

`@` must be empty and undescribed. If it holds work, ask before going on, and
if the user agrees, `jj new` so the bumps start from a clean change. Keep the
operation id: `jj op restore <id>` puts the lock files *and* history back as
they were, whatever happens below.

Note every check that already fails. A failure present before a bump is not
the bump's, and one absent before it is.

## 1. Cargo.lock

```sh
cargo update --dry-run --verbose    # "Unchanged x (available: y)": a major release or an exact pin
cargo update
nix flake check
cargo deny check advisories         # online; the flake check cannot fetch the database
jj commit Cargo.lock -m "Cargo update"
```

If anything newly fails, back out with `jj restore Cargo.lock` and report the
failure and the crate that caused it.

## 2. flake.lock and the toolchain

The toolchain is `stable` of the locked rust-overlay (`rust-toolchain.toml`,
`modules/toolchain.nix`), so it moves with flake.lock; `nanoyalla` and `yalla`
are pinned to commits in `flake.nix` (compare them with their upstream heads).

```sh
nix develop -c rustc --version      # before
nix flake update 2> "$S/flake-update.txt"; cat "$S/flake-update.txt"
nix develop -c rustc --version      # after
nix flake check
jj commit flake.lock -m "$(printf 'flake.lock: Update\n\nFlake lock file updates:\n\n'; awk '/^•/{p=1} p' "$S/flake-update.txt")"
```

What a new toolchain breaks, and how each was met (2026-10-22, 1.98 → 1.99):

- **New lints**: fix them at their sites as a change on top (clippy's
  suggestion where it is one, `#[expect]` with a reason where it is not). A
  rustdoc warning without a span: `cargo doc --message-format=json` names
  the item whose docs it is in (the first line of its outer docs).
- **The ratchet** (`bench/ceilings.csv`): codegen moves the journeys' counts
  both ways. Find what moved before touching a ceiling: build `linlog-bench`
  at the commit before the bump in a workspace of its own (`jj workspace add
  ../linlog-deps-old -r REV`) and after it, `ratchet --jobs 1 --keep DIR` in
  each, then compare callgrind's self counts per function (`callgrind_annotate`
  on both files), and time the journeys that moved, pinned to one core,
  alternating the two binaries (`journey NAME --repeat N`). A clean change of
  linlog's code that wins the counts back is a commit of its own, verified as
  its area demands (a search's layout: the target sets equal to the latest
  label, `bench/TARGETS.md`); a ceiling raised for what no clean change wins
  back is a commit of its own, on the author's word, giving the functions and
  the times.
- **An exact pin** that waits for nixpkgs: `wasm-bindgen = "=…"` must equal a
  `wasm-bindgen-cli_X_Y_Z` the locked nixpkgs has (`modules/web.nix` fails at
  evaluation otherwise); after the flake update, list them and move the pin
  and `wasm-bindgen-test`'s with it.

## 3. Major releases

Each `Unchanged x (available: y)` of step 1 that is not an exact pin: read the
release's changes (fetch it into the registry with a scratch crate's `cargo
fetch`, then diff the sources), make the `Cargo.toml` change and what the code
needs, as a commit per crate. Hold one back, and say why, when another
dependency still needs the old one across an API (`resvg` waits for
`krilla-svg` to move to the same `usvg`), or when it moves a search: a crate
under the engines (a hasher, a container) is taken only if the target sets
(`bench/targets.sh`, `bench/net-targets.sh`) keep every decided row's verdict
and counters and pinned time is not worse.

## 4. The fuzz workspace

`fuzz/` has its own lock and a nightly pinned by date in `modules/fuzz.nix`:
move the date to a recent nightly the locked rust-overlay has, then

```sh
nix develop .#fuzz -c cargo update --manifest-path fuzz/Cargo.toml
nix develop .#fuzz -c cargo check --manifest-path fuzz/Cargo.toml --bins
```

## 5. GitHub Actions

Every `uses:` is pinned to a commit SHA with the release as a comment
(`.claude/rules/ci.md`). Take each action's latest release that is at least
fourteen days old (the client's age gate, applied to both repositories),
resolve its tag with `nix flake prefetch --json github:OWNER/REPO/TAG`
(`locked.rev`), and diff its `action.yml` against the pinned one for inputs
that changed.

## 6. The web client

`/home/tux/Projects/own/linlog-prover.github.io`, in a jj workspace of its own
when a session works there. Its CLAUDE.md and `.claude/rules/flake.md` hold
its procedure: every ecosystem at its latest release at least fourteen days
old (`.npmrc`'s `min-release-age=14`), each lock a commit of its own,
verified by its `nix flake check`. Its nixpkgs follows linlog's, and its lock
names only a linlog commit on GitHub, so its flake update comes after
linlog's update is pushed and moves Node.js and Playwright with it.
`@playwright/test` must equal nixpkgs' `playwright-driver`: a version
younger than the gate passes it only with the author's word, noted in its
rules. Its other deliberate holds (TypeScript 6, solid-js 1.x) stay.

## 7. Report

- per change: committed or backed out, and the change id
- what moved, as `name old -> new`, crates linlog depends on directly first
- what was held back and why: major releases, actions younger than the gate,
  the client's holds
- the rustc version before → after, and every count or time a bump moved
- every failure a bump introduced, and what fixing it took or would take
