---
paths:
  - "flake.nix"
  - "flake.lock"
  - "modules/**"
---

# The flake's modules

Loaded when `flake.nix`, its lock or a file under `modules/` is read.
CLAUDE.md says how the flake is put together (dendritic flake-parts,
import-tree, one aspect per file); this is what each module holds.

Modules share values through `_module.args`:
`rustToolchain` and `craneLib` (`toolchain.nix`), `workspace` (the crane
arguments, `workspace.nix`, whose source is what crane's
`commonCargoSources` keeps plus `core/tests/snapshots`, `cli/fonts` and
`README.md`, which a CLI test runs the examples of). `checks.nix`,
`devshell.nix`, `treefmt.nix` and `systems.nix` are what their names
say; `checks.nix`'s `test` is crane's, in the release profile, and
`test-debug-assertions` the same tests with debug assertions and
overflow checks on. `claude-hooks.nix` is the check that pins the
`.claude/hooks` guards (`claude-infra.md`). `export.nix` is the `export`
check, which compiles the snapshots (the fragments inside a document of
its own) and a CLI proof with pdfLaTeX and with Typst and the curryst of
nixpkgs (the version `export::typst::CURRYST` names), with no font but
Typst's own, renders the SVG snapshots and CLI drawings with resvg with
only the Euler Math font of nixpkgs' TeX Live, and checks the CLI's PNG
(pngcheck) and PDF (poppler: the font embedded, the text extractable;
veraPDF: every profile conforms), offline; `rocq.nix` is the `rocq`
check, which builds NanoYalla from the non-flake input `nanoyalla`
(Click & coLLecT pinned to a commit; `export::rocq::NANOYALLA` names
the version) with nixpkgs' Rocq and standard library and compiles the
`.v` snapshots and two CLI certificates against it, requiring Rocq to
print nothing; `bench.nix` is the `linlog-bench` package, the `bench`
check (the harness on the smallest instance of every family and on the
problem file, failing on a verdict against a known one, a `MISMATCH`)
and the `lltp` package, the LLTP library fetched at a pinned commit with
its Petri-net archives unpacked and its one malformed file repaired,
which no check uses.
