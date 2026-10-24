# linlog © Fabian Lukas Grubmüller 2026
# Licensed under the EUPL

# The cargo workspace through crane. The dependencies are built once
# (`buildDepsOnly', from Cargo.lock) and the CLI and every check start from
# them; `workspace' passes the shared arguments on to checks.nix. A C library
# a crate links goes into `buildInputs' here.
{
  perSystem =
    {
      config,
      craneLib,
      lib,
      pkgs,
      ...
    }:
    let
      inherit (lib) fileset;

      # What each crate is built and tested from. A check's source is the
      # parts it needs, so that a change elsewhere leaves its derivation,
      # and its cached result, as they were: the core crate's tests do not
      # see the command's code, and no Rust check sees the prose (README
      # and the guide for AI models aside, whose examples the command's
      # tests run), the scripts or the
      # benchmark data. The core crate's export tests draw with the font the
      # command embeds, and its tests of the fonts' tables read the fonts'
      # files.
      parts = {
        core = fileset.unions [
          ../core/Cargo.toml
          (craneLib.fileset.commonCargoSources ../core/src)
        ];
        core-tests = fileset.unions [
          ../core/tests
          ../cli/fonts
        ];
        cli = fileset.unions [
          ../cli/Cargo.toml
          (craneLib.fileset.commonCargoSources ../cli/src)
          ../cli/fonts
          ../LICENSE
        ];
        cli-tests = fileset.unions [
          ../cli/tests
          ../README.md
          ../skills
        ];
        # The guide for AI models, an Agent Skill, which the rustdoc site
        # serves.
        guide = ../skills;
        bench = fileset.unions [
          ../bench/Cargo.toml
          (craneLib.fileset.commonCargoSources ../bench/src)
        ];
        bench-tests = ../bench/tests;
        web = fileset.unions [
          ../web/Cargo.toml
          ../web/build.rs
          ../web/about.toml
          ../web/about.hbs
          (craneLib.fileset.commonCargoSources ../web/src)
          ../LICENSE
        ];
        # The bindings' tests render with the font the command embeds.
        web-tests = fileset.unions [
          ../web/tests
          ../cli/fonts
        ];
      };

      # cargo loads every member of the workspace, so every source keeps
      # every member's manifest, and `stubMembers' gives a member left out
      # an empty file for each target its manifest names; nothing builds
      # them.
      filesetOf =
        names:
        fileset.unions (
          [
            ../Cargo.toml
            ../Cargo.lock
            ../core/Cargo.toml
            ../cli/Cargo.toml
            ../bench/Cargo.toml
            ../web/Cargo.toml
          ]
          ++ map (name: parts.${name}) names
        );
      srcOf =
        names:
        fileset.toSource {
          root = ../.;
          fileset = filesetOf names;
        };
      everything = builtins.attrNames parts;

      stubMembers = ''
        for target in cli/src/lib.rs cli/src/main.rs bench/src/main.rs web/src/lib.rs; do
          if [ ! -e "$target" ]; then
            mkdir -p "$(dirname "$target")"
            touch "$target"
          fi
        done
      '';

      # The crane arguments of a build from the named parts.
      argsOf =
        names:
        commonArgs
        // {
          src = srcOf names;
          postPatch = stubMembers;
        };

      # Every crate with its tests: the dependencies' build, which reads the
      # manifests alone, and the checks that read the whole workspace.
      commonArgs = {
        src = srcOf everything;
        strictDeps = true;

        buildInputs = lib.optionals pkgs.stdenv.hostPlatform.isDarwin [ pkgs.libiconv ];
      };

      # How cargo selects each crate for its checks. cargo resolves the
      # dependencies' features for the packages it selects, so a crate
      # alone builds some of them otherwise than the whole workspace does:
      # the core crate without the command's `serde_json` features, the
      # harness without `png` and `pdf`.
      selections = {
        core = "--package linlog --all-features";
        cli = "--package linlog-cli";
        bench = "--package linlog-bench";
        web = "--package linlog-web";
      };

      # The dependencies of every crate as its checks and packages select
      # it, in one build tree, so that none of them builds a dependency
      # again.
      depsOnly =
        args:
        craneLib.buildDepsOnly (
          args
          // {
            buildPhaseCargoCommand = lib.concatMapStrings (selection: ''
              cargoWithProfile check --locked --all-targets ${selection}
              cargoWithProfile build --locked ${selection}
            '') (builtins.attrValues selections);
            checkPhaseCargoCommand = lib.concatMapStrings (selection: ''
              cargoWithProfile test --locked --no-run ${selection}
            '') (builtins.attrValues selections);
          }
        );

      cargoArtifacts = depsOnly commonArgs;

      # The dependencies once more with the debug assertions and overflow
      # checks the `test-debug-assertions-*' checks turn on: the flags are
      # part of every artifact, so each of those checks would rebuild them.
      debugAssertions = {
        CARGO_PROFILE_RELEASE_DEBUG_ASSERTIONS = "true";
        CARGO_PROFILE_RELEASE_OVERFLOW_CHECKS = "true";
      };
      cargoArtifactsDebugAssertions = depsOnly (
        commonArgs
        // debugAssertions
        // {
          pname = "linlog-workspace-debug-assertions";
        }
      );
    in
    {
      _module.args.workspace = {
        # Every crate with its tests, for the checks that read all of it.
        fileset = filesetOf everything;
        inherit
          argsOf
          selections
          commonArgs
          cargoArtifacts
          cargoArtifactsDebugAssertions
          debugAssertions
          ;
      };

      packages = {
        linlog-cli = craneLib.buildPackage (
          argsOf [
            "core"
            "cli"
          ]
          // {
            inherit cargoArtifacts;
            inherit (craneLib.crateNameFromCargoToml { cargoToml = ../cli/Cargo.toml; }) pname;
            # The crates inherit the workspace's version.
            inherit (craneLib.crateNameFromCargoToml { cargoToml = ../Cargo.toml; }) version;
            cargoExtraArgs = "--locked --package linlog-cli";
            # The `test-cli' check runs them.
            doCheck = false;
            # A copy of the binary carries the licences of what it contains:
            # the EUPL's text, and each embedded font's OFL with its notice.
            postInstall = ''
              install -Dm644 LICENSE $out/share/doc/linlog-cli/LICENSE
              install -Dm644 -t $out/share/doc/linlog-cli cli/fonts/*.txt
            '';
            # The binary is `linlog', not the package name `nix run' assumes.
            meta.mainProgram = "linlog";
          }
        );

        # The rustdoc site the Docs workflow publishes, which the `doc' check
        # also builds: both crates, `linlog' and `linlog_cli', in one tree.
        # rustdoc writes no top-level index, so one redirects to the core
        # crate. Beside it the guide for AI models: its summary as
        # `llms.txt' (the Agent Skill's body, without its front matter) and
        # its reference as `llms-full.txt' and as the `reference.md' the
        # summary links to.
        doc = craneLib.cargoDoc (
          argsOf [
            "core"
            "cli"
            "guide"
          ]
          // {
            inherit cargoArtifacts;
            cargoDocExtraArgs = "--no-deps --package linlog --package linlog-cli";
            env.RUSTDOCFLAGS = "--deny warnings";
            postInstall = ''
              cat > $out/share/doc/index.html <<'EOF'
              <!DOCTYPE html>
              <meta charset="utf-8">
              <title>linlog</title>
              <meta http-equiv="refresh" content="0; url=linlog/">
              <a href="linlog/">linlog</a>
              EOF
              awk 'n >= 2 && (body || NF) { body = 1; print } /^---$/ { n++ }' \
                skills/linlog/SKILL.md > $out/share/doc/llms.txt
              install -Dm644 skills/linlog/reference.md $out/share/doc/reference.md
              install -Dm644 skills/linlog/reference.md $out/share/doc/llms-full.txt
            '';
          }
        );

        default = config.packages.linlog-cli;
      };
    };
}
