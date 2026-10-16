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
of `main`, to GitHub Pages on every push to `main`.

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
  in every job of the three workflows, with the repository secret
  `CACHIX_AUTH_TOKEN` (a write token) and `skipPush` unless the ref is
  `main`: pushes to `main`, the weekly run and a dispatch on `main`
  upload, a pull request only downloads. A pull request from a fork gets
  no secret, and the action then only says that pushing is disabled. It
  pushes in daemon mode, what the job built and nothing it fetched: the
  dependencies' builds, the packages, the documentation, NanoYalla, the
  provers of the comparison, and each check's empty result, so a check
  whose derivation did not change is fetched, not rerun. How narrow those
  derivations are is the flake's (`flake.md`): a change to `cli/` leaves
  the core crate's checks as they were. A check's output holds no build
  tree, so the cache's size is that of the dependencies' builds (about
  185 MiB each) and the packages.
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
