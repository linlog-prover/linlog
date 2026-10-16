# linlog © Fabian Lukas Grubmüller 2026
# Licensed under the EUPL

# `nix flake check', besides treefmt-nix's own `treefmt' check. cargo-deny's
# `advisories' needs the network, so it is not a check here: run
# `cargo deny check' in the shell (CI runs it on every push and weekly).
{
  perSystem =
    {
      config,
      craneLib,
      lib,
      pkgs,
      workspace,
      ...
    }:
    let
      # Each crate's checks, from the parts of the workspace it builds from
      # (`workspace.nix'): a change to one crate reruns its own checks and
      # those of the crates that depend on it, and the cache serves the rest.
      # The core crate is checked with every feature, as the command, which
      # enables them all, builds it (`workspace.selections').
      crates = {
        core = {
          parts = [
            "core"
            "core-tests"
          ];
          selection = workspace.selections.core;
        };
        cli = {
          parts = [
            "core"
            "cli"
            "cli-tests"
          ];
          selection = workspace.selections.cli;
        };
        bench = {
          parts = [
            "core"
            "bench"
            "bench-tests"
          ];
          selection = workspace.selections.bench;
        };
      };

      # The checks of every crate, `NAME-CRATE' for each of the checks given.
      perCrate =
        checks:
        lib.concatMapAttrs (
          crate: c:
          lib.mapAttrs' (
            name: check:
            lib.nameValuePair "${name}-${crate}" (
              check (
                workspace.argsOf c.parts
                // {
                  inherit (workspace) cargoArtifacts;
                  cargoExtraArgs = "--locked ${c.selection}";
                  # crane appends its own suffix (`-test', `-clippy').
                  pname = "linlog-${crate}";
                  # A check's result is that it passed; the cache holds no
                  # build tree of it.
                  doInstallCargoArtifacts = false;
                }
              )
            )
          ) checks
        ) crates;

      nixSources = lib.fileset.toSource {
        root = ../.;
        fileset = lib.fileset.fileFilter (file: file.hasExt "nix") ../.;
      };

      workflowSources = lib.fileset.toSource {
        root = ../.;
        fileset = ../.github/workflows;
      };
    in
    {
      checks = {
        build = config.packages.linlog-cli;

      }
      // perCrate {
        clippy =
          args: craneLib.cargoClippy (args // { cargoClippyExtraArgs = "--all-targets -- --deny warnings"; });

        # `cargo test', not nextest: nextest skips doc tests.
        test = craneLib.cargoTest;

        # The tests once more with the assertions a development build has:
        # crane tests in the release profile, where `debug_assert!' is
        # nothing, and the engines state their invariants in that macro (a
        # proof checked where it is made, a zone that must be empty). The
        # doc tests ran above.
        test-debug-assertions =
          args:
          craneLib.cargoTest (
            args
            // workspace.debugAssertions
            // {
              pname = "${args.pname}-debug-assertions";
              cargoArtifacts = workspace.cargoArtifactsDebugAssertions;
              cargoTestExtraArgs = "--all-targets";
            }
          );
      }
      // {
        doc = config.packages.doc;

        # The docs of every item, the private ones too, which a maintainer
        # reads and the published site leaves out: their links must resolve
        # as the public ones do.
        doc-private = craneLib.cargoDoc (
          workspace.argsOf [
            "core"
            "cli"
          ]
          // {
            inherit (workspace) cargoArtifacts;
            pnameSuffix = "-doc-private";
            cargoDocExtraArgs = "--no-deps --document-private-items --package linlog --package linlog-cli";
            env.RUSTDOCFLAGS = "--deny warnings";
          }
        );

        deny = craneLib.cargoDeny {
          src = lib.fileset.toSource {
            root = ../.;
            fileset = lib.fileset.unions [
              workspace.fileset
              ../deny.toml
            ];
          };
        };

        # The library as a browser will run it: built for wasm32 with every
        # feature but `parallel', whose thread pool a browser does not give.
        # The library reads no clock and starts threads only behind that
        # feature; this check keeps it so.
        wasm = craneLib.mkCargoDerivation (
          workspace.argsOf [ "core" ]
          // {
            pnameSuffix = "-wasm";
            cargoArtifacts = null;
            buildPhaseCargoCommand = ''
              cargo check --locked --target wasm32-unknown-unknown --package linlog \
                --no-default-features \
                --features parse,serialize,interactive,latex,typst,svg,rocq,png,pdf
            '';
            doInstallCargoArtifacts = false;
          }
        );

        # The core crate's features one at a time and in pairs, with all of
        # them and with none; the CLI enables all of them and would never
        # notice one that breaks alone. The full powerset doubles with every
        # feature, and a breakage needs more than two features rarely.
        # Warnings are denied, so an item or import that one feature set
        # leaves unused fails the check.
        features = craneLib.mkCargoDerivation (
          workspace.argsOf [ "core" ]
          // {
            inherit (workspace) cargoArtifacts;
            pnameSuffix = "-features";
            env.RUSTFLAGS = "-D warnings";
            nativeBuildInputs = [ pkgs.cargo-hack ];
            buildPhaseCargoCommand = ''
              cargo hack check --each-feature --locked --package linlog
              cargo hack check --feature-powerset --depth 2 --locked --package linlog
            '';
            doInstallCargoArtifacts = false;
          }
        );

        deadnix = pkgs.runCommand "check-deadnix" { nativeBuildInputs = [ pkgs.deadnix ]; } ''
          cd ${nixSources}
          deadnix --fail .
          touch $out
        '';

        # The GitHub workflows, which no local run exercises otherwise;
        # nixpkgs' actionlint runs shellcheck on their `run' scripts.
        actionlint = pkgs.runCommand "check-actionlint" { nativeBuildInputs = [ pkgs.actionlint ]; } ''
          cd ${workflowSources}
          actionlint .github/workflows/*
          touch $out
        '';
      };
    };
}
