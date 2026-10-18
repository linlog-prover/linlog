---
paths:
  - "flake.nix"
  - "flake.lock"
  - "modules/**"
---

# The flake's modules

Loaded when `flake.nix`, its lock or a file under `modules/` is read or changed.
CLAUDE.md says how the flake is put together (dendritic flake-parts,
import-tree, one aspect per file); this is what each module holds.

Modules share values through `_module.args`:
`rustToolchain` and `craneLib` (`toolchain.nix`), `workspace` (the crane
arguments, `workspace.nix`). `checks.nix`, `devshell.nix`, `treefmt.nix`
and `systems.nix` are what their names say.

- **Each check reads only its crate's parts.** `workspace.nix` cuts the
  workspace into parts (`core`, `core-tests`, `cli`, `cli-tests`,
  `bench`, `bench-tests`: a crate's `src/` with its manifest, and its
  `tests/` with what they read: the core crate's export tests the font
  in `cli/fonts`, the command's README's examples), and `argsOf PARTS`
  gives the crane arguments of a source of those parts. Every source
  keeps the four manifests and the lock, since cargo loads every member,
  and `stubMembers` (the `postPatch`) gives a member left out an empty
  file for each target its manifest names (`cli/src/lib.rs`,
  `cli/src/main.rs`, `bench/src/main.rs`): a new explicit target path in
  a manifest goes into that list. A file no part names is in no Rust
  check: `plan/`, the benchmark data, the scripts, `fuzz/`.
- `checks.nix`'s `clippy-CRATE`, `test-CRATE` and
  `test-debug-assertions-CRATE` (CRATE `core`, `cli`, `bench`) are one
  check per crate from `perCrate`: the core crate's from `core` and
  `core-tests` with `--all-features` (the set the command enables, so
  that `parallel`, `png` and `pdf` are tested; `cargo test --workspace`
  unifies to the same set), the command's from `core`,
  `cli`, `cli-tests`, the harness's from `core`, `bench`,
  `bench-tests`. So a change to `cli/` reruns the command's checks only
  and one to `core/src/` all three. `test-CRATE` is crane's, in the
  release profile; `test-debug-assertions-CRATE` the same tests with
  debug assertions and overflow checks on, from their own dependencies'
  build (`cargoArtifactsDebugAssertions`), since the flags are in every
  artifact. Checks install no build tree (`doInstallCargoArtifacts =
  false`): the cache holds only that they passed. The packages
  (`linlog-cli` from `core` and `cli`, `linlog-bench` from `core` and
  `bench`) leave out the tests, so a test's edit reruns neither `export`,
  `rocq`, `bench` nor `ratchet`; `doc`, `doc-private` read `core` and
  `cli`, `wasm` and `features` `core` alone, `deny` and `shear` every
  part.
- The dependencies' build (`buildDepsOnly`) reads the manifests and the
  lock alone (crane's dummy source), so it is the same derivation for
  every change but a manifest's or the lock's.

`doc` is the published site and `doc-private` the
docs of every item with `--document-private-items`, both with
rustdoc's warnings denied, so a link in a private item's doc resolves too
(a split of a module once left four such links pointing nowhere). `claude-hooks.nix` is the check that pins the
`.claude/hooks` guards (`claude-infra.md`). `export.nix` is the `export`
check, which compiles the snapshots (the fragments inside a document of
its own; `core/tests/snapshots/linlog/`, the Rocq library's
certificates, is left out of its fileset, since it copies the files at
the top and `rocq-linlog` compiles those), the command's default fragments (LaTeX and Typst, wrapped the
same way) and a CLI proof with pdfLaTeX and with Typst and the curryst of
nixpkgs (the version `export::typst::CURRYST` names, read from the source
at evaluation, so the package and the import line it writes cannot
drift), with no font but
Typst's own, renders the SVG snapshots and CLI drawings with resvg with
only the Euler Math font of nixpkgs' TeX Live, and checks the CLI's PNG
(pngcheck) and PDF (poppler: the font embedded, the text extractable;
veraPDF: every profile conforms), offline; `rocq.nix` is the `rocq`
check, which builds NanoYalla from the non-flake input `nanoyalla`
(Click & coLLecT pinned to a commit; `export::rocq::NANOYALLA` names
the version, which an assertion compares with the kernel's README at
evaluation) with nixpkgs' Rocq and standard library and compiles the
`.v` snapshots and six CLI certificates against it (four of them over
`Prop`, of ordinary logic, classical and intuitionistic, two of `true`,
which bind nothing; the intuitionistic linear one names `--kernel
nanoyalla`, since the mode alone would choose linlog's library),
requiring Rocq to print nothing; `rocq-linlog.nix` is the package
`rocq-linlog`, the library under `rocq/` built with
`rocqPackages.mkRocqDerivation` from a release entry whose `src` is the
tree's `_RocqProject`, `Makefile` and `theories/` (`hash = ""` stands for
the fake hash nothing fetches), the version read from
`rocq/rocq-linlog.opam` and the standard library propagated, and the
check `rocq-linlog`, which greps the sources for `Admitted`, `admit`,
`Axiom`, `Parameter` and `Conjecture`, compiles every `.v` under
`rocq/tests/`, then in a directory of their own the exporter's
snapshots for this kernel (`core/tests/snapshots/linlog/`) and six
certificates the CLI writes for it (Mix, affine, intuitionistic,
intuitionistic affine, a `⊤`/`0` goal, classical with `--kernel
linlog`), against the installed library (`ROCQPATH` names both
`user-contrib` directories), requires silence from each certificate and
requires `Assumptions.v` to print `Closed under the global context` once
per `Print Assumptions` line it has and nothing else (`rocq.md`); an
assertion at evaluation compares `export::rocq::LINLOG` with the opam
file's version; `rocq-bridges.nix` is the check `rocq-bridges`, which
copies `nanoll.v` from the input `nanoyalla` and `microyalla/nanoill.v`
from the input `yalla` (Yalla pinned to a commit, since its last
release tag predates that file; LGPL, so fetched and never copied into
the tree), compiles them unchanged under their own logical paths
(`NanoYalla`, `Yalla.MicroYalla`, judged by exit status), then the
modules of `rocq/bridges/` against them and the installed `rocq-linlog`
(silence required) and `bridges/Assumptions.v` as `rocq-linlog` does
its own; the package leaves the bridges out, so no certificate and no
build of the library needs either input; `bench.nix` is the `linlog-bench` package, the `bench`
check (the harness on the smallest instance of every family, the
problem file, and an LLTP and a `.spec` file of the command's lock
inputs, failing through `summary --strict` on a verdict against a known
one, an `error` row, a crash, a kill or no rows)
and the `lltp` package, the LLTP library fetched at a pinned commit with
its Petri-net archives unpacked and its one malformed file repaired,
which no check uses, and the `iltp` package, the ILTP library's
propositional part (v1.1.2, 274 TPTP problems, the input of the
ordinary-logic run) as its site serves it, fetched because it states no
licence; `nix build .#iltp -o bench/iltp` puts it where `.gitignore`
expects it. The `qcover` package is `github:blondimi/qcover` at its last
commit (2021-03-03), 442 MB, whose 176 coverability problems under
`stable/benchmarks/` (Mist 27, Soter 50, Wahl–Kroening 46, medical 12,
bug tracking 41) the harness reads with `--spec`; the repository is
Apache-2.0, the problem files state nothing of their own, so they are
fetched, not committed (`nix build .#qcover -o bench/qcover`).
`bench.nix` also has the `ratchet` check: `linlog-bench ratchet --check`
under nixpkgs' valgrind (`valgrind.out`, since the package's first output
is its manual) against `bench/ceilings.csv` (`bench.md`).
`conventions.nix` holds three checks over the tracked tree but `plan/`
and the benchmark data: `conventions` (the licence header on every
`.rs`, `.nix`, `.toml`, `.sh`, `.py` and `.yml` file, within the first
four lines so that a shebang may come first; no comment and no README
line naming a step, a decision number or a file of the plan; `.claude/`
is exempt), `typos` (configured by `typos.toml`, whose words of the
domain are TPTP's `fof`, Rocq's `falso` and the like) and `shear`
(cargo-shear in a crane derivation, which reads the vendored
dependencies offline). The spec is outside the plan grep because it
names Andreoli's rules D1 and D2.
`systems.nix` declares x86_64-linux, aarch64-linux and aarch64-darwin;
CI builds and checks x86_64-linux alone, the other two evaluate and are
built nowhere. valgrind is broken in nixpkgs on aarch64-darwin, so the
devshell takes it and `ratchet.nix` defines the `ratchet` check only
where it runs (`lib.meta.availableOn` and not `meta.broken`, written once
as the module argument `valgrindRuns`, which the devshell reads; the
devshell once failed to evaluate there). The crates inherit `version` and
`edition` from `[workspace.package]`, so crane's package versions come
from the root `Cargo.toml` (`crateNameFromCargoToml` on a member's falls
back to 0.0.1).
`checks.nix`'s `wasm` builds the core crate for
`wasm32-unknown-unknown` (the target `rust-toolchain.toml` names) with
every feature but `parallel`, a `cargo check` without the host's
artifacts: the rule that the library stays fit for a browser was held by
convention alone before it.
`checks.nix`'s `package` packages both crates as `cargo publish`
uploads them and builds each from its package: `cargo package
--registry crates-io --package linlog` (which verifies the library's
package by building it), then the command's package without
verification and without a lock, both unpacked outside the source tree
(whose manifest would claim them as members) and the command built in
release from its package with the library's patched in (`[patch.crates-io]`)
and the workspace's `Cargo.lock`. Offline, the vendored dependencies
replace crates.io, so cargo can neither see the unpublished library as
the command's dependency (its local overlay fails) nor package without
`--registry crates-io`. A package that needs a file from outside its
directory, or a test that does not travel left out of `exclude`, fails
there; what only crates.io validates at the upload (keywords, at most
five of at most 20 ASCII characters; categories from its list, an
unknown one only warned about; the licence's SPDX expression) no local
command checks, so a change of those fields is checked by hand. Its source is the whole
workspace with the licence files (`LICENSE` and the links
`core/LICENSE`, `cli/LICENSE`). The `linlog-cli` package installs
`LICENSE` and the font's `OFL.txt` under `share/doc/linlog-cli/`, since
a copy of the binary (the public cache holds one) must carry the EUPL's
text and the embedded font's licence; the `cli` part carries `LICENSE`
for that. The upload
needs the network and the author's token; `cargo publish --dry-run
--locked --workspace` in the shell packages and verifies both against
the registry, the command through cargo's local overlay of the
unpublished library, and must run on a clean working copy (`jj new` on
the release commit): cargo's dirty check reads git, to which a non-empty
`@` is uncommitted, and `--allow-dirty` would record that in the
package's `.cargo_vcs_info.json`.
`msrv.nix` is the `msrv` check: `cargo check --workspace --all-targets
--all-features` with the stable toolchain of `rust-version` in
`[workspace.package]` (read at evaluation, `1.92` written as rust-overlay's
`1.92.0`), its dependencies built with it too, nothing shared with the
other checks. A `cargo update` that takes a dependency past that
version, or code that needs a newer compiler, fails it; raising
`rust-version` is a new 0.y version and a line of the changelog.
`checks.nix`'s `features` (cargo-hack, each feature and every pair) and
the gate's two `cargo hack` runs deny warnings (`RUSTFLAGS="-D
warnings"`; the gate's in `target/hack`, since the flags are part of
every artifact and would rebuild the tests'): an import that a move left
unused without `serialize` once warned in ten feature sets while both
passed.
`devshell.nix` carries cargo-mutants and cargo-nextest for
`mutants/run.sh` and valgrind for the ratchet, and its `ratchet` command; `fuzz.nix` is the `fuzz` devshell, a nightly compiler
pinned by date against the locked rust-overlay and cargo-fuzz, for the
workspace under `fuzz/` alone (`evidence.md`).
`comparison.nix` builds the other provers of the comparison from their
own sources at pinned revisions (never vendored: several state no licence,
or GPL), each a `prover-NAME` package whose derivation is in
`modules/_provers/` (a directory import-tree skips): the Maude prover on
nixpkgs' Maude 3 unchanged, Click & coLLecT's prover with a driver of our
own in place of its web server and its 3 s limit read from the
environment, llprover, linTAP and LinearOne on SWI-Prolog with drivers of
our own for the last two (linTAP's `prove/1` never ends on a formula it
cannot prove), Sympli on MLton with a patch of no-op functions for the
newer SML/NJ `PP_DEVICE`, lolli with its `cargoHash`, APLL native with
OCaml 4.14 (it was written for 4.06). Each carries
`passthru.source` (url, revision, date, licence), which `comparison`'s
`share/linlog-comparison/tools.json` records per `--tools` name.
`comparison` joins linlog's command and harness, BenchExec, the provers
and Python with matplotlib; no check builds it.
