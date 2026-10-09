# linlog © Fabian Lukas Grubmüller 2026
# Licensed under the EUPL

# The repository's conventions as checks, so that they never regress:
# `conventions' (every source file starts with the licence header, and no
# comment or README line names the plan that organised the work),
# `typos' (spelling, configured by typos.toml) and `shear' (no
# dependency a crate declares and never uses).
{ lib, ... }:
{
  perSystem =
    {
      craneLib,
      pkgs,
      workspace,
      ...
    }:
    let
      # The tracked tree but the plan's own files and the data, which
      # quote other projects and name the plan by design.
      tree = lib.fileset.toSource {
        root = ../.;
        fileset = lib.fileset.difference ../. (
          lib.fileset.unions [
            ../plan
            ../bench/results
            ../bench/problems
          ]
        );
      };
    in
    {
      checks = {
        conventions = pkgs.runCommand "check-conventions" { nativeBuildInputs = [ pkgs.ripgrep ]; } ''
          cd ${tree}
          status=0
          sources=$(rg --files --hidden --glob '!.git' \
            --glob '*.rs' --glob '*.nix' --glob '*.toml' --glob '*.sh' \
            --glob '*.py' --glob '*.yml' --glob '*.yaml')
          for file in $sources; do
            # A script's header follows its shebang line.
            if ! head -n 3 "$file" | rg --quiet --fixed-strings 'linlog © Fabian Lukas Grubmüller 2026' \
              || ! head -n 4 "$file" | rg --quiet --fixed-strings 'Licensed under the EUPL'; then
              echo "$file: no licence header"
              status=1
            fi
          done
          # A comment says what the code does or why, in terms of the
          # repository: never a step, a decision number or a file of the
          # plan. `.claude/' is the setup of the sessions themselves.
          plan='plan/|\b[Ss]teps? [0-9]+\b|\bD[0-9]{1,2}a?\b|conduct\.md'
          if rg --line-number --glob '!.claude/**' \
            --glob '*.rs' --glob '*.nix' --glob '*.toml' --glob '*.sh' \
            --glob '*.py' --glob '*.yml' --glob '*.yaml' \
            "(//|#).*($plan)" .; then
            echo "a comment names the plan"
            status=1
          fi
          if rg --line-number 'plan/|\b[Ss]teps? [0-9]+\b' README.md; then
            echo "README names the plan"
            status=1
          fi
          [ "$status" = 0 ] && touch $out
          exit "$status"
        '';

        typos = pkgs.runCommand "check-typos" { nativeBuildInputs = [ pkgs.typos ]; } ''
          cd ${tree}
          typos --config typos.toml .
          touch $out
        '';

        shear = craneLib.mkCargoDerivation (
          workspace.commonArgs
          // {
            inherit (workspace) cargoArtifacts;
            pnameSuffix = "-shear";
            nativeBuildInputs = [ pkgs.cargo-shear ];
            buildPhaseCargoCommand = "cargo shear";
            doInstallCargoArtifacts = false;
          }
        );
      };
    };
}
