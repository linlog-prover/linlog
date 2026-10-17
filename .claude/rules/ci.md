---
paths:
  - ".github/**"
---

# CI: the GitHub workflows

Loaded when a file under `.github/` is read or changed.

GitHub Actions runs `.github/workflows/ci.yml` on every push to `main`, on
every pull request and weekly: `nix flake check`, and the online
`cargo deny check advisories` in the devshell. A workflow installs nothing
but Nix, so CI checks with exactly the tools flake.lock pins.
`.github/workflows/docs.yml` publishes the flake's `doc` package, the rustdoc
of `main`, to GitHub Pages on every push to `main`. `.github/workflows/release.yml`
releases a version tag `vX.Y.Z` (below).

- **Every tool comes from the flake.** A job installs Nix and runs
  `nix flake check`, `nix build` or `nix develop --command …`. No
  `setup-rust`, `apt` or `cargo install`: they would check with other versions
  than the devshell does. A new check goes into the flake
  (`modules/checks.nix`), not into a workflow step.
- **Every action is pinned to a full commit SHA**, with the release as
  a comment: `uses: owner/repo@<sha> # vX.Y.Z`. The organization
  `linlog-prover` requires it of all actions, GitHub's own included
  (its Actions policy, since 2026-10-03): a `uses:` with a tag or a
  branch fails the run. Resolve a tag with
  `gh api repos/<owner>/<repo>/commits/<tag> --jq .sha`, or without `gh`
  by `nix flake prefetch --json github:<owner>/<repo>/<tag>` (its
  `locked.rev`; `flake metadata` fails on a repository without a
  `flake.nix`), and read that commit's `action.yml` for its inputs. Dependabot proposes the bumps, weekly,
  as one pull request. The pin of `cachix/install-nix-action` also fixes the
  Nix version CI runs.
- **Least privilege**: `permissions: contents: read` at the top, anything more
  per job; `actions/checkout` with `persist-credentials: false`.
- `install-nix-action` gets `github_access_token: ${{ github.token }}`: without
  it, fetching the flake's `github:` inputs from shared runners hits the
  anonymous rate limit.
- **No workflow runs locally.** `nix flake check` runs actionlint, with
  shellcheck on the `run:` scripts, over `.github/workflows/`: it catches
  syntax, unknown runner labels, bad expressions and shell mistakes, not
  behaviour on GitHub. Read run results with `gh run list` and
  `gh run view --log-failed`.
- **The cache: Cachix `linlog-prover`** (public,
  https://linlog-prover.cachix.org, key
  `linlog-prover.cachix.org-1:6K+DDVwwELnSYtfflwUdsHdYuolC7hprdD/r2/KOQZU=`,
  signed by Cachix). `cachix/cachix-action` follows `install-nix-action`
  in every job of the four workflows that installs Nix, with the
  repository secret `CACHIX_AUTH_TOKEN` (a write token) and `skipPush`
  unless the ref is `main` (`release.yml` only reads it): pushes to `main`, the weekly run and a dispatch on `main`
  upload, a pull request only downloads. A pull request from a fork gets
  no secret, and the action then only says that pushing is disabled. It
  pushes in daemon mode what the job realised and no upstream cache
  (cache.nixos.org) holds: the toolchain, the dependencies' builds, the
  packages, the documentation, NanoYalla, the provers of the comparison,
  and each check's empty result, so a check whose derivation did not
  change is fetched, not rerun. How narrow those derivations are is the
  flake's (`flake.md`): a change to `cli/` leaves the core crate's checks
  as they were. A download is a realised path too: `pushFilter:
  '\.tar\.xz$'` keeps the toolchain's tarballs out (183 MiB of the first
  925 MiB upload, 2026-10-17), and the toolchain's `minimal` profile
  leaves out the manual `rust-docs` (81 MiB). The rest of that upload:
  the unpacked toolchain about 350 MiB, the dependencies' two builds
  (plain and with debug assertions) 323 MiB, linlog's own 14 MiB. A
  push of code adds about the last; a lock or toolchain bump adds a new
  set of the others, and Cachix evicts the least recently used paths
  past the plan's storage.
- **Docs** deploys `result/share/doc` of `nix build .#doc` with GitHub's own
  Pages actions. The repository's Pages source must be "GitHub Actions".
- **Comparison** (`comparison.yml`, `workflow_dispatch` only, inputs
  `timeout` and `quick`) runs `bench/comparison.sh` in ten shards, one
  job each (two sequential streams on cores 0 and 1, linlog's default on
  2–3), uploads each shard's rows and records, and a last job joins
  them, writes `ranking.md` and the figures, puts the ranking in the
  run's summary and uploads it all as the artifact `comparison`; it
  commits nothing. BenchExec needs a cgroup from the user's systemd, so
  the job enables lingering for the runner's user first and exports its
  bus. It has never run: the first run is the author's.
- **Release** (`release.yml`, on a tag `vX.Y.Z` only), three jobs. `check`:
  the tag equals `[workspace.package]`'s version, CITATION.cff states it,
  and CHANGELOG.md has a dated section `## [X.Y.Z] - YYYY-MM-DD` (a
  section headed `- unreleased` fails it), then `nix flake check`.
  `publish` (skipped for `v0.1.0`, which the author published by hand,
  since crates.io's Trusted Publishing works only for a crate that
  exists): in the environment `crates-io`, which the trusted publisher
  on crates.io names and whose protection rules may ask for the author's
  approval, with `id-token: write`; it packages both crates first (the
  token from `rust-lang/crates-io-auth-action`, pinned by commit, lives
  briefly), then publishes `linlog` and `linlog-cli` in that order, each
  unless its version is on crates.io already, so that rerunning the job
  after a failure between the two finishes it. `release` (`contents:
  write`, after `publish` succeeded or was skipped, never after it
  failed): `gh release create` with the changelog's section as the notes
  and the two `.crate` files downloaded from static.crates.io, the ones
  published. The cache is only read (`skipPush`): a tag's builds are
  `main`'s. Nothing of it can run before the author's first publication
  and tag; actionlint is all that checks it.
