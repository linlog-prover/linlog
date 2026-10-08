# Research for step 30: the first release

Written on 2026-10-08 for step 30 (`plan/30-baseline-release.md`, item 2)
and for step 28's API design, from the snapshot of the repository and the
sources in section 5. `[Sn]` cites a source; "(inference)" marks my own
reasoning. Tool versions are the newest on crates.io on the day read.

## 1. The problem and the state of the art

**Definitions.**

- *Breaking change in 0.y.z.* Cargo treats a change of `y` as major and a
  change of `z` as minor: "only changes in the left-most non-zero component
  are considered incompatible" [S1]. So after 0.1.0, any break is 0.2.0,
  and 0.1.z may only add.
- *What breaks* [S1]: adding a variant to an enum without
  `#[non_exhaustive]`; adding any field to a struct whose fields are all
  public; adding `#[non_exhaustive]` later to an enum or an all-public
  struct; removing a cargo feature. Adding a feature is minor. The guide
  advises marking types `#[non_exhaustive]` "when first introducing them".
- *MSRV.* `package.rust-version` declares the oldest supported toolchain.
  Cargo errors early on an older one, and `cargo add` and the resolver
  respect it [S2]. Resolver `"3"`, the default of edition 2024, needs
  Rust 1.84+. It prefers dependency versions compatible with
  `rust-version` (`incompatible-rust-versions = "fallback"`) [S3]. Raising
  the MSRV is "possibly-breaking", treated as minor [S1]. The Cargo book
  asks a crate to choose a policy and publish it (examples: latest stable,
  "N-2", every version of the current year) [S2].
- *Permanence.* A published version "can never be overwritten, and the
  code cannot be deleted". `cargo yank` blocks new dependents and deletes
  nothing. The `.crate` limit is 10 MB. `cargo publish` packages the
  crate, unpacks it and checks that it compiles [S4].
- *Workspace publishing.* `cargo publish --workspace` became stable in
  Rust 1.90 (2025-09-18). It publishes in dependency order and checks the
  to-be-published crates together, "including during dry runs". Publishes
  "are still not atomic" [S9].
- *Trusted Publishing.* On crates.io since July 2025, for GitHub Actions
  only. A workflow swaps its OIDC identity for a short-lived token. "You'll
  need to publish your first release manually" [S13]. The job needs
  `id-token: write` [S15]. `rust-lang/crates-io-auth-action@v1` outputs the
  token and revokes it when the job ends [S14].
- *Name policy.* Since 2023 the crates.io terms are stricter on name
  squatting [S16]. No placeholder crates, as `plan/notes/distribution.md`
  already concludes [S43].
- *docs.rs.* It builds with nightly rustdoc, with no network, about
  6.4 GB of RAM and 15 minutes per build. It passes `--cfg docsrs` to the
  final rustdoc only [S18]. `[package.metadata.docs.rs]` takes `features`,
  `all-features`, `no-default-features`, `default-target`, `targets`,
  `additional-targets`, `rustc-args`, `rustdoc-args` and `cargo-args`.
  Without `targets` it builds five default targets [S17]. In September 2025
  `doc_auto_cfg` was folded into `doc_cfg` (rust-lang/rust#138907). This
  broke crates that turned the old feature on under `cfg(docsrs)`. regex
  and jiff moved to a crate-specific cfg [S19].
- *Changelog.* Keep a Changelog 1.1.0 asks for an entry per version,
  newest first, with an ISO date and an "Unreleased" section on top. Its
  groups are Added, Changed, Deprecated, Removed, Fixed and Security. It
  says not to paste commit logs [S26].
- *Licence.* `EUPL-1.2` is the SPDX identifier, and crates.io reads
  `license` as an SPDX 2.3 expression (`AND`, `OR`, `WITH`) [S5, S36].
  The Commission recommends "licensed under the EUPL", which "targets
  the current EUPL version". The alternative is "Licensed under the
  EUPL-1.2-or-later" [S38]. The font the command embeds is under
  `OFL-1.1` [S37] (`cli/fonts/OFL.txt`).
- *Provenance.* GitHub artifact attestations (`actions/attest@v4`, with
  `id-token: write`, `attestations: write` and `contents: read`) sign
  where and how a release file was built. You check them with
  `gh attestation verify` [S32]. Alone they give SLSA v1.0 Build Level 2,
  or Level 3 from a reusable workflow. Public repositories log to
  Sigstore's public transparency log [S33].
- *Citation.* A `CITATION.cff` on the default branch is linked from the
  repository page [S35] (not in the source). Zenodo, switched on for a public repository,
  archives each GitHub release with a new DOI. For an organisation, the
  owner may have to approve the Zenodo app [S34].

**Existing implementations** (all are Rust crates on crates.io):

| tool | what it does | licence | newest release | source |
|---|---|---|---|---|
| cargo-semver-checks | lints the rustdoc JSON of two versions for semver breaks. The baseline is crates.io by default, or a git revision. Default feature set excludes `unstable*` and `_*` features. Misses some type and generics changes | Apache-2.0 OR MIT | 0.51.0, 2026-10-03 (needs Rust 1.93) | [S20] |
| release-plz | release pull request, version bump from conventional commits, git-cliff changelog, semver-checks, tags, GitHub releases, publication; supports Trusted Publishing | MIT OR Apache-2.0 | 0.3.170, 2026-10-07 | [S21] |
| cargo-release | local bump, tag, publish | MIT OR Apache-2.0 | 1.1.6, 2026-09-16 | [S23] |
| dist (cargo-dist) | builds release binaries and installers (shell, PowerShell, npm, Homebrew, MSI) from a generated `release.yml`; `github-attestations = true` | MIT OR Apache-2.0 | 0.32.0, 2026-05-22 | [S24] |
| cargo-binstall | installs binaries from GitHub releases by URL templates, otherwise compiles | GPL-3.0-only | 1.25.2, 2026-10-06 | [S25] |
| git-cliff | changelog from commit history | MIT OR Apache-2.0 | 2.14.2, 2026-09-18 | [S27] |
| cargo-deny | licences, bans, sources, advisories | MIT OR Apache-2.0 | 0.20.2, 2026-07-09 | [S28] |
| cargo-vet | records audits of dependencies, imports others' audits; exemptions shrink over time | Apache-2.0/MIT | 0.10.2, 2026-01-13 | [S29] |
| cargo-auditable | puts the dependency list (under 4 kB) into the binary; read by cargo audit, trivy, grype, syft; NixOS builds with it | MIT OR Apache-2.0 | 0.7.7, 2026-10-02 | [S30] |
| cargo-cyclonedx | CycloneDX SBOM of a crate | Apache-2.0 | 0.5.9, 2026-03-19 | [S31] |
| cargo-msrv | finds the MSRV | Apache-2.0 OR MIT | 0.19.3 stable; 0.20.0-beta.2, 2026-10-07 | [S41] |
| cargo-hack | `--rust-version` and `--version-range` run a command on the MSRV or a range of toolchains | Apache-2.0 OR MIT | 0.6.45, 2026-05-30 | [S40] |

Cargo's own SBOM output (`-Z sbom`, `<artifact>.cargo-sbom.json`) is still
unstable [S7]. The newest stable Rust is 1.99.0 (2026-10-01) [S12].

## 2. What the step needs

The step is mostly process. Its "algorithms" are checks, each with a cost.

| check | cost | when |
|---|---|---|
| `cargo publish --workspace --dry-run` | one release build of `linlog` and `linlog-cli` from their packages | every release; cheap enough for CI (inference) |
| `cargo package --list` per crate | seconds | before the first publication |
| cargo-semver-checks against crates.io | two rustdoc JSON builds | from 0.1.1 on; there is no baseline before 0.1.0 [S20] |
| MSRV build (`cargo hack check --rust-version`) | one more toolchain in the flake closure | every push, once `rust-version` is set [S40] |
| docs.rs | its own build, ≤ 15 min, 6.4 GB [S18] | on publication |

The choices, each with a recommendation:

1. **Release tool.** Recommend none for 0.1.0. Write the changelog by
   hand and run the dry run in CI.
   - release-plz bumps versions from conventional commits [S21]. Without
     them, a bump is a patch [S22]. linlog's subjects have no type prefix
     (CLAUDE.md), so release-plz would only ever bump the patch
     (inference).
   - cargo-release commits and tags locally through git, and the repo
     runs no git (inference from its role, not verified in its source).
   - Revisit release-plz once releases are frequent. It also runs
     semver-checks and Trusted Publishing [S21].
2. **Publication path.** Recommend the order in [S13]:
   - publish 0.1.0 by hand with a scoped token (`cargo publish
     --workspace`), then revoke the token;
   - register `linlog-prover/linlog` and the release workflow as a
     trusted publisher of both crates;
   - add a `release.yml`: on a tag `v*`, check that the tag equals the
     manifests' version, run `nix flake check`, publish through
     `crates-io-auth-action`, and create the GitHub release from the
     changelog section.
   - Publishing is not atomic [S9], so order the steps `linlog` before
     `linlog-cli` and let them re-run.
3. **Changelog.** Recommend `CHANGELOG.md` at the root, Keep a Changelog
   form [S26]. 0.1.0 says "first release" and lists what exists (README's
   "Built" list), not the history. git-cliff adds nothing while commits
   are not typed (inference).
4. **MSRV.** Recommend setting `rust-version` in `[workspace.package]`
   [S6] and checking it in a flake check.
   - The floor is 1.85 for edition 2024 [S11] and 1.88 for the let chains
     the code uses (`core/src/search/mod.rs`, `prove_goal`) [S10]. The
     dependencies may need more.
   - Measure the real minimum with cargo-msrv or `cargo hack check
     --version-range` [S40, S41]. Do not guess.
   - Policy: "latest stable minus N, raised only in a 0.y bump". That is
     stricter than Cargo's minor (inference), and the README states it.
5. **docs.rs.** Recommend `[package.metadata.docs.rs] all-features = true`
   and `targets = ["x86_64-unknown-linux-gnu"]` for `linlog`.
   - `all-features` documents `parallel`, `png` and `pdf`, which are off
     by default. One target is enough because nothing in the API depends
     on the target (inference) [S17].
   - Keep the hand-written "Needs the cargo feature" lines (core.md's
     rule) and do not turn on `doc_cfg` under `cfg(docsrs)`, given the
     2025 breakage [S19]. If badges are wanted later, use a crate-specific
     cfg passed through `rustdoc-args`, as regex does [S19].
   - For `linlog-cli`, decide whether `linlog_cli` is public API (see 4).
6. **Binaries.** Recommend no prebuilt binaries for 0.1.0. Install with
   `cargo install linlog-cli` and `nix run github:linlog-prover/linlog`.
   - `nix run` runs `packages.default` by `meta.mainProgram` [S42], which
     `modules/workspace.nix` already sets.
   - dist adds a matrix of targets and installers [S24]. That is worth it
     only if users without Rust or Nix ask (D22, [S43]).
   - If binaries come, build them with cargo-auditable [S30], attest them
     [S32, S33], and add `[package.metadata.binstall]` only if the file
     names differ from binstall's defaults [S25].
7. **Supply chain.**
   - Keep cargo-deny as it is. It already allows EUPL-1.2 and runs
     advisories weekly (`.github/workflows/ci.yml`).
   - cargo-vet needs an audit trail per dependency [S29]. That is
     disproportionate for one author with about ten direct dependencies
     (inference). Defer it.
   - An SBOM file is optional. Cargo's own is unstable [S7], and
     cargo-cyclonedx exists [S31].
   - crates.io itself publishes no attestation I could find. The
     provenance of a crate is the Trusted Publishing link to the workflow
     (inference from [S13]).
8. **Licence metadata.**
   - `linlog`: `license = "EUPL-1.2"`.
   - `linlog-cli`: embeds Euler Math (`cli/src/prove.rs`, `FONT`), so its
     expression should be `EUPL-1.2 AND OFL-1.1` [S5, S37] (inference).
     `cli/fonts/OFL.txt` lies inside the package.
   - Put a `LICENSE` inside `core/` and `cli/` (or `license-file`) so each
     `.crate` carries the text. distribution.md lists it as missing [S43].
   - Whether the headers' unversioned "Licensed under the EUPL" and the
     manifests' `EUPL-1.2` say the same is the author's call [S38] (see 4).
9. **Metadata.** Inherit from `[workspace.package]` [S6]:
   - `repository = "https://github.com/linlog-prover/linlog"`;
   - `homepage` (the Pages address or the repository);
   - `rust-version`, `license`, `authors`;
   - at most five keywords, ASCII, ≤ 20 characters each [S5];
   - categories from the crates.io slugs [S5], for example `science`,
     `mathematics` and `command-line-utilities` (the last for the CLI).
     All three exist (crates.io category API, read 2026-10-08).
   - Leave `documentation` unset so crates.io links docs.rs [S5].
   - `linlog-cli` needs a description of its own [S43].

## 3. What linlog's library must offer for it

A release turns the API and the wire forms into promises [S1], D18. This
section lists, by area, what the current code promises by accident and
what it cannot promise yet.

**Packaging, beside the code.**
- `cli/tests/readme.rs` uses `include_str!("../../README.md")`, and
  `core/tests/export.rs` reads `../cli/fonts/Euler-Math.otf`. Both paths
  leave their package, so `cargo test` on a downloaded `.crate` fails.
  `cargo publish` compiles the package but does not run its tests [S4]
  (inference). Either exclude those tests from the package (`exclude`
  [S4]) or give each crate its own copy.
- A `readme` outside the package root is a case Cargo's packaging handles
  [S8]. Check with `cargo package --list` that `README.md` lands in both
  crates.
- `bench` has `publish = false` and is left out of the dry run.

**Data model** (`core/src/sequents/term.rs`: `Term`, `Kind`;
`sequents/mod.rs`: `Sequent`; `occurrences/mod.rs`: `OccId`, `Forest`,
`Sign`, `Polarity`; `fragment.rs`: `Fragment`, `Mode`).
- `Term` and `Kind` are exhaustive enums. First-order logic (D17, step 38)
  adds quantifiers to both, which is a 0.2.0 [S1].
  - Recommendation: keep them exhaustive and accept 0.2.0 when
    quantifiers come. A printer or checker downstream should fail to
    compile on a new connective rather than meet `_ => unreachable!()`
    (inference).
  - This should be a stated decision, not an accident.
- `Mode` has three public fields and no `#[non_exhaustive]`, and the crate
  docs build it with a struct literal (`core/src/lib.rs`). Any further
  flag (an ordered or cyclic mode for step 36) breaks it [S1].
  - Recommendation: `#[non_exhaustive]` with the constants and `with_*`
    builders it already has (`Mode::CLASSICAL`, `with_mix`). Change the
    lib.rs example.
- `Fragment(u8)` is opaque. Good.

**Proof term and checker** (`proofs/mod.rs`: `Node`, `NodeId`, `Side`,
`Proof`; `proofs/check.rs`: `CheckError`, `Problem`;
`proofs/derivation.rs`: `Rule`).
- `Node` is exhaustive with 13 variants. Cut (step 34) and the quantifier
  rules (step 38) add variants. The same argument as for `Kind` applies:
  keep it exhaustive, plan 0.2.0 and 0.3.0, and say so in the changelog's
  policy line.
- `Rule` (derivation) is exhaustive. It follows `Node`.
- `check::Problem` is `#[non_exhaustive]`. Good.

**Engine interface and dispatch** (`search/mod.rs`).
- `Decide` is `pub(crate)` and `DISPATCH` is private: free to change.
- `Engine`, `Bias`, `Reason`, `Refutation`, `Outcome` and `Statistics`
  are already `#[non_exhaustive]`.
- `Verdict` is exhaustive, which fits D9's three values. Keep it.
- `Error` (`errors/mod.rs`) has about 28 variants and is *not*
  `#[non_exhaustive]`. Every new refusal (such as `Error::NotHorn` at
  step 27) would be a break. Mark it before 0.1.0, since adding the mark
  later is itself major [S1]. The same holds for the enums `NetError`
  (`nets/mod.rs`) and `ShapeError` (`occurrences/reading.rs`), which are
  not marked. `ViewError`, `WriteError`, `RenderError` and `Refusal` are
  marked already. `ParseError` (`errors/parse.rs`) and `CheckError`
  (`proofs/check.rs`) are structs: check whether either has a public
  field.
- `engine_for` and `Engine::parallel` are public queries (step 27). They
  are a promise from 0.1.0.

**Options.**
- `search::Options` has private fields, a builder and `DEFAULT_*`
  constants. It can grow in a patch release. It has no serde form, which
  step 28 plans.
- The export options (`export/latex.rs`, `typst.rs`, `rocq.rs`, `png.rs`,
  `pdf.rs`, `svg`: each `pub struct Options`) have all-public fields and
  no `#[non_exhaustive]`. D15 says options grow, so each new option would
  be a 0.y bump [S1].
  - Recommendation: `#[non_exhaustive]` on each. Callers then write
    `let mut o = Options::default(); o.align = false;`.
  - Their serde form is already `default, deny_unknown_fields`. So a new
    field is backward-compatible for readers of old JSON but not the
    other way round (inference).

**JSON wire forms.**
- The forms are documented on `Sequent`, `Proof`, `Outcome` (written,
  never read), `ProofStructure` and `Interactive`. None carries a version
  (`core/src/serialize/`: no version field).
- D18 keeps them out of "the API", but the web client (step 32) and saved
  files depend on them.
- Recommendation, for step 28:
  - one top-level `"version"` on every form a program reads back
    (`Proof`, `ProofStructure`, `Interactive`, `Sequent` inside them);
  - readers accept a missing version as 1 (inference);
  - the changelog lists form changes under their own heading.
- `Outcome`, written only, needs a version as well if the CLI's
  `--format json` is to be machine-readable across releases.

**Public dependencies** [S39] (C-STABLE).
- serde appears in public impls behind `serialize`, at 1.x. Good.
- rayon stays behind `search::Pool` (`search/parallel.rs`, an `Arc`
  wrapper). Good.
- The export functions take `&str`, `&[&[u8]]` and `Vec<u8>`, not
  resvg or krilla types. Good. So resvg and krilla, which are pre-1.0,
  can move without a break of linlog (inference).

**Features.** Removing any of the ten features is major [S1]. Fix the set
before 0.1.0. `parse` gating `lltp`, `mist`, `families` and
`ordinary::read_tptp` is the one place where a split might still be
wanted (inference).

**`linlog_cli`.** The CLI crate's library is public on crates.io and so
under semver too (inference from [S1]). Either document it as internal
(`#[doc(hidden)]`, or a crate doc that says it carries no promise) or
review it like `linlog`.

**`#![allow(dead_code)]`** in `core/src/lib.rs` is due to go at step 28.
It does not block publishing.

## 4. Risks, open questions, and what the prompt should add

**Risks.**
- A wrong publish cannot be undone, only yanked [S4]. A partial workspace
  publish is possible [S9]. Mitigation: the dry run in CI, `cargo package
  --list`, and the author's manual first publish.
- docs.rs may fail where the flake's rustdoc passes: nightly vs stable,
  no network [S18]. The `export` feature set pulls resvg and krilla. A
  local `cargo docs-rs`-style build before publishing is advised [S18].
- Raising the MSRV unnoticed through a `Cargo update`. A flake check on
  the `rust-version` toolchain stops it (inference).
- The release workflow is the first with `id-token: write` and publishing
  rights. It should take the pinned-SHA style of `ci.yml` and run only on
  tags (inference).

**Inconsistencies found in the snapshot.**
- `plan/notes/distribution.md` says "Step 31 reads this before it
  prepares the release". The release is step 30 since the renumbering
  (D23).
- The step's item 2 asks for "the Rocq library's opam file", but the
  library is written at step 31 (D20, D23). D22 publishes `rocq-linlog`
  "once the library has a release". At step 30 only a draft is possible.
- README's "Planned" list puts the web front end and the Rocq library
  before "A first release". D23's order is release, Rocq, web.
- `core` and `cli` share one `description`, "A linear logic suite for
  all your needs."

**Open questions for the author.**
1. Do the semver promises cover the JSON forms and the CLI's output, or
   the Rust API only? (D18 excludes them before 0.1.0.)
2. Are `Kind`, `Term`, `Node` and `Rule` exhaustive (a 0.y bump for cut
   and quantifiers) or `#[non_exhaustive]`?
3. Is `linlog_cli` public API?
4. Which MSRV policy, and is it in the README?
5. `EUPL-1.2` or `EUPL-1.2-or-later` (SPDX `EUPL-1.2+`)? The headers say
   "Licensed under the EUPL" [S38].
6. Prebuilt binaries at 0.1.0, or `cargo install` and Nix only?
7. Trusted Publishing from 0.1.1 on, with a GitHub environment that needs
   the author's approval?
8. Is the opam file of item 2 a draft or dropped from step 30?

**The prompt should add:**
- a checklist that cites [S1]: `#[non_exhaustive]` on `Error`, `Mode`
  and every export `Options`, decided before the tag;
- `cargo publish --workspace --dry-run` and `cargo package --list` in the
  report;
- the tests that read outside their package fixed or excluded;
- `rust-version` measured and checked by the flake;
- `[package.metadata.docs.rs]`;
- `linlog-cli`'s licence expression with OFL-1.1;
- the manifests' metadata inherited from `[workspace.package]`;
- `CHANGELOG.md` and `CITATION.cff` (`cff-version: 1.2.0` [S35]);
- a `release.yml` that publishes only from 0.1.1 on;
- the README's install section with `cargo install linlog-cli` and
  `nix run`;
- the reminder that the Zenodo app needs the organisation owner's
  approval [S34];
- cargo-semver-checks as a CI job from the first release on, with
  `--all-features`, since `parallel` is off by default (inference from
  [S20]'s feature defaults).

## 5. Sources

- [S1] The Cargo Book, "SemVer Compatibility". Rust project, read
  2026-10-08. https://doc.rust-lang.org/cargo/reference/semver.html
- [S2] The Cargo Book, "Rust Version". Rust project, read 2026-10-08.
  https://doc.rust-lang.org/cargo/reference/rust-version.html
- [S3] The Cargo Book, "Dependency Resolution". Rust project, read
  2026-10-08. https://doc.rust-lang.org/cargo/reference/resolver.html
- [S4] The Cargo Book, "Publishing on crates.io". Rust project, read
  2026-10-08. https://doc.rust-lang.org/cargo/reference/publishing.html
- [S5] The Cargo Book, "The Manifest Format". Rust project, read
  2026-10-08. https://doc.rust-lang.org/cargo/reference/manifest.html
- [S6] The Cargo Book, "Workspaces". Rust project, read 2026-10-08.
  https://doc.rust-lang.org/cargo/reference/workspaces.html
- [S7] The Cargo Book, "Unstable Features" (`-Z sbom`). Rust project,
  read 2026-10-08. https://doc.rust-lang.org/cargo/reference/unstable.html
- [S8] Cargo internals documentation,
  `cargo::ops::cargo_package::vcs::dirty_files_outside_pkg_root`. Rust
  project.
  https://doc.rust-lang.org/stable/nightly-rustc/cargo/ops/cargo_package/vcs/fn.dirty_files_outside_pkg_root.html
- [S9] The Rust Release Team, "Announcing Rust 1.90.0". Rust Blog,
  2025-09-18. https://blog.rust-lang.org/2025/09/18/Rust-1.90.0/
- [S10] The Rust Release Team, "Announcing Rust 1.88.0". Rust Blog,
  2025-06-26. https://blog.rust-lang.org/2025/06/26/Rust-1.88.0/
- [S11] The Rust Edition Guide, "Rust 2024". Rust project.
  https://doc.rust-lang.org/edition-guide/rust-2024/index.html
- [S12] Rust Blog, release announcements index (1.99.0 on 2026-10-01).
  https://blog.rust-lang.org/releases/
- [S13] The crates.io team, "crates.io: development update". Rust Blog,
  2025-07-11.
  https://blog.rust-lang.org/2025/07/11/crates-io-development-update-2025-07/
- [S14] rust-lang, `crates-io-auth-action`, README. GitHub.
  https://github.com/rust-lang/crates-io-auth-action
- [S15] Rust Forge, "Trusted publishing" (Rust infrastructure docs).
  https://forge.rust-lang.org/infra/docs/trusted-publishing.html (read
  through a search summary only)
- [S16] RFC 3463, "crates.io policy update". Rust RFCs, accepted 2023.
  https://rust-lang.github.io/rfcs/3463-crates-io-policy-update.html
- [S17] docs.rs, "Metadata for custom builds". Read 2026-10-08.
  https://docs.rs/about/metadata
- [S18] docs.rs, "Builds". Read 2026-10-08 (nightly 1.101.0 of
  2026-10-07). https://docs.rs/about/builds
- [S19] GuillaumeGomez (GitHub), rust-lang/rust PR #138907 (RFC 3631, `doc_cfg`),
  merged 2025, https://github.com/rust-lang/rust/pull/138907. Rust Users
  Forum, "Doc_auto_cfg is gone: what am I supposed to do?",
  https://users.rust-lang.org/t/doc-auto-cfg-is-gone-what-am-i-supposed-to-do/135070
  (both read through a search summary).
- [S20] obi1kenobi et al., cargo-semver-checks, README, and the crates.io
  record of 0.51.0. https://github.com/obi1kenobi/cargo-semver-checks ,
  https://crates.io/api/v1/crates/cargo-semver-checks
- [S21] release-plz, documentation and GitHub quickstart, and the
  crates.io record of 0.3.170. https://release-plz.dev/docs ,
  https://release-plz.dev/docs/github/quickstart ,
  https://github.com/release-plz/release-plz
- [S22] `next_version` crate documentation (version rules used by
  release-plz's core). docs.rs. https://docs.rs/next_version
- [S23] crate-ci, cargo-release, crates.io record of 1.1.6.
  https://github.com/crate-ci/cargo-release
- [S24] axo, dist (cargo-dist) book and "GitHub Attestations" page;
  crates.io record of 0.32.0. https://axodotdev.github.io/cargo-dist/book/
  , https://axodotdev.github.io/cargo-dist/book/supplychain-security/attestations/github.html
- [S25] cargo-bins, cargo-binstall, SUPPORT.md, and the crates.io record
  of 1.25.2. https://github.com/cargo-bins/cargo-binstall/blob/main/SUPPORT.md
- [S26] O. Lacan, "Keep a Changelog" 1.1.0.
  https://keepachangelog.com/en/1.1.0/
- [S27] orhun (GitHub), git-cliff, crates.io record of 2.14.2.
  https://github.com/orhun/git-cliff
- [S28] Embark Studios, cargo-deny, crates.io record of 0.20.2.
  https://github.com/EmbarkStudios/cargo-deny
- [S29] Mozilla, cargo-vet book; crates.io record of 0.10.2.
  https://mozilla.github.io/cargo-vet/
- [S30] rust-secure-code, cargo-auditable, README; crates.io record of
  0.7.7. https://github.com/rust-secure-code/cargo-auditable
- [S31] CycloneDX, cargo-cyclonedx, crates.io record of 0.5.9.
  https://github.com/CycloneDX/cyclonedx-rust-cargo
- [S32] GitHub Docs, "Using artifact attestations to establish provenance
  for builds".
  https://docs.github.com/en/actions/how-tos/secure-your-work/use-artifact-attestations/use-artifact-attestations
- [S33] GitHub Docs, "Artifact attestations" (concepts).
  https://docs.github.com/en/actions/concepts/security/artifact-attestations
- [S34] GitHub Docs, "Referencing and citing content".
  https://docs.github.com/en/repositories/archiving-a-github-repository/referencing-and-citing-content
- [S35] Citation File Format. https://citation-file-format.github.io/
- [S36] SPDX License List, "European Union Public License 1.2".
  https://spdx.org/licenses/EUPL-1.2.html
- [S37] SPDX License List, "SIL Open Font License 1.1".
  https://spdx.org/licenses/OFL-1.1.html
- [S38] European Commission, Interoperable Europe, "EUPL text (EUPL
  1.2)".
  https://interoperable-europe.ec.europa.eu/collection/eupl/eupl-text-eupl-12
- [S39] Rust Library Team, Rust API Guidelines, checklist.
  https://rust-lang.github.io/api-guidelines/checklist.html
- [S40] taiki-e (GitHub), cargo-hack, README.
  https://github.com/taiki-e/cargo-hack
- [S41] foresterre (GitHub), cargo-msrv, crates.io record.
  https://github.com/foresterre/cargo-msrv
- [S42] Nix Reference Manual 2.28, "nix run".
  https://nix.dev/manual/nix/2.28/command-ref/new-cli/nix3-run
- [S43] linlog, `plan/notes/distribution.md`, facts checked 2026-10-03
  (repository-internal).

Sources checked 2026-10-08: 42 checked, 2 corrected, 0 removed, 1 claims marked.
