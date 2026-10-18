# linlog © Fabian Lukas Grubmüller 2026
# Licensed under the EUPL

# The web bindings (`web/', the crate linlog-web) as a browser runs them:
# the package a page loads, built for wasm32 with the wasm-bindgen CLI and
# wasm-opt, which the client's flake takes through its pinned input; and
# the checks that run them there. The CLI must be the version of the
# wasm-bindgen crate that Cargo.lock pins, or the module it writes does
# not load: it is picked by that version from nixpkgs' versioned packages,
# and `web-bindgen' compares the two. A bump of the crate the nixpkgs in
# flake.lock has no CLI for fails here at evaluation.
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
      lock = builtins.fromTOML (builtins.readFile ../Cargo.lock);
      bindgenVersion = (lib.findFirst (p: p.name == "wasm-bindgen") null lock.package).version;
      cliName = "wasm-bindgen-cli_${lib.replaceStrings [ "." ] [ "_" ] bindgenVersion}";
      wasm-bindgen-cli =
        pkgs.${cliName}
          or (throw "Cargo.lock pins wasm-bindgen ${bindgenVersion}, and nixpkgs has no ${cliName}");

      target = "wasm32-unknown-unknown";

      # The library's tests that hold its integers' arguments (the checker,
      # the size estimate, the memo and the memory account) on a 32-bit
      # target: libtest runs on WASI under wasmtime, which the web target
      # has no runner for, and the two share wasm32's data layout.
      core32 = "proofs::check proofs::size search::focus::memo search::memory";

      args = workspace.argsOf [
        "core"
        "core-tests"
        "web"
        "web-tests"
      ];

      # The dependencies for wasm32, of the package and of its tests.
      wasmArtifacts = craneLib.buildDepsOnly (
        args
        // {
          pname = "linlog-web-wasm";
          CARGO_BUILD_TARGET = target;
          buildPhaseCargoCommand = ''
            cargoWithProfile build --locked --package linlog-web
            cargo test --locked --package linlog-web --no-run
          '';
          checkPhaseCargoCommand = "";
          doCheck = false;
        }
      );

      # The bindings' tests under Node, as wasm-bindgen-test-runner runs
      # them with Node's own stack, which is a browser's size.
      wasmTests = craneLib.mkCargoDerivation (
        args
        // {
          pnameSuffix = "-web-wasm-tests";
          cargoArtifacts = wasmArtifacts;
          nativeBuildInputs = [
            wasm-bindgen-cli
            pkgs.nodejs-slim
          ];
          CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER = "wasm-bindgen-test-runner";
          WASM_BINDGEN_TEST_TIMEOUT = "300";
          buildPhaseCargoCommand = "cargo test --locked --target ${target} --package linlog-web";
          doInstallCargoArtifacts = false;
        }
      );

      core32Tests = craneLib.mkCargoDerivation (
        workspace.argsOf [
          "core"
          "core-tests"
        ]
        // {
          pnameSuffix = "-core-wasm32";
          cargoArtifacts = null;
          nativeBuildInputs = [ pkgs.wasmtime ];
          CARGO_TARGET_WASM32_WASIP1_RUNNER = "wasmtime run --";
          buildPhaseCargoCommand = ''
            export HOME=$TMPDIR
            cargo test --locked --target wasm32-wasip1 --package linlog --lib -- ${core32}
          '';
          doInstallCargoArtifacts = false;
        }
      );
    in
    {
      packages.linlog-web = craneLib.mkCargoDerivation (
        args
        // {
          pname = "linlog-web";
          inherit (craneLib.crateNameFromCargoToml { cargoToml = ../Cargo.toml; }) version;
          cargoArtifacts = wasmArtifacts;
          nativeBuildInputs = [
            wasm-bindgen-cli
            pkgs.binaryen
            pkgs.cargo-about
          ];
          buildPhaseCargoCommand = ''
            cargo build --locked --release --target ${target} --package linlog-web
            cargo about generate --frozen --manifest-path web/Cargo.toml \
              --config web/about.toml web/about.hbs --output-file THIRD-PARTY-NOTICES.txt
          '';
          installPhaseCommand = ''
            wasm-bindgen --target web --out-dir $out \
              target/${target}/release/linlog_web.wasm
            wasm-opt -O3 $out/linlog_web_bg.wasm -o $out/linlog_web_bg.wasm
            install -Dm644 LICENSE $out/LICENSE
            install -Dm644 THIRD-PARTY-NOTICES.txt $out/THIRD-PARTY-NOTICES.txt
          '';
          doInstallCargoArtifacts = false;
        }
      );

      checks = {
        web-package = config.packages.linlog-web;
        web-wasm-tests = wasmTests;
        web-core-wasm32 = core32Tests;
        # The CLI the package is built with speaks the crate's version.
        web-bindgen = pkgs.runCommand "check-web-bindgen" { nativeBuildInputs = [ wasm-bindgen-cli ]; } ''
          test "$(wasm-bindgen --version)" = "wasm-bindgen ${bindgenVersion}"
          touch $out
        '';
      };

      devshells.default.devshell.packages = [
        wasm-bindgen-cli
        pkgs.binaryen
        pkgs.nodejs-slim
        pkgs.wasmtime
      ];
    };
}
