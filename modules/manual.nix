# linlog © Fabian Lukas Grubmüller 2026
# Licensed under the EUPL

# The manual (docs/): mdBook builds it, with every figure drawn by
# bench/plots.py from the benchmarks' CSV rows (docs/figures.sh), into the
# package `manual', which the `doc' package serves under `manual/' beside
# the API documentation, the Pages site. The `manual-links' check resolves
# every link of the manual and of README offline: one into the repository
# (GitHub's `blob', `tree' and `edit' pages, raw files) against the tree,
# one into the site against the `doc' package, fragments included; links
# elsewhere, the web page's among them, are not fetched.
{ lib, ... }:
{
  perSystem =
    { config, pkgs, ... }:
    let
      # The figures' Python, the comparison's (`comparison.nix').
      python = pkgs.python3.withPackages (ps: [ ps.matplotlib ]);
    in
    {
      packages.manual = pkgs.stdenvNoCC.mkDerivation {
        name = "linlog-manual";
        src = lib.fileset.toSource {
          root = ../.;
          fileset = lib.fileset.unions [
            ../docs
            ../bench/plots.py
            ../bench/results
            ../bench/comparison
          ];
        };
        nativeBuildInputs = [
          pkgs.mdbook
          python
        ];
        buildPhase = ''
          runHook preBuild
          patchShebangs docs/figures.sh
          export MPLCONFIGDIR=$TMPDIR/matplotlib
          docs/figures.sh docs/src/figures
          mdbook build docs --dest-dir "$out"
          runHook postBuild
        '';
        dontInstall = true;
      };

      checks = {
        manual = config.packages.manual;
        manual-links =
          pkgs.runCommand "check-manual-links"
            {
              # lychee builds its HTTP client, which no link reaches
              # offline, with the system's certificates.
              nativeBuildInputs = [
                pkgs.lychee
                pkgs.cacert
              ];
            }
            ''
              site=${config.packages.doc}/share/doc
              tree=${../.}
              # The site is served at /linlog/, which mdBook's 404 page
              # names absolutely.
              mkdir root
              ln -s "$site" root/linlog
              lychee --offline --no-progress --include-fragments \
                --root-dir "$PWD/root" \
                --remap "^https://github\.com/linlog-prover/linlog/(?:blob|tree|edit)/main/(.*)$ file://$tree/\$1" \
                --remap "^https://raw\.githubusercontent\.com/linlog-prover/linlog/main/(.*)$ file://$tree/\$1" \
                --remap "^https://linlog-prover\.github\.io/linlog/(.*)$ file://$site/\$1" \
                "$site/manual/**/*.html" "$tree/README.md"
              touch $out
            '';
      };

      devshells.default = {
        devshell.packages = [
          pkgs.mdbook
          python
        ];
        commands = [
          {
            name = "manual";
            help = "Draw the manual's figures and serve it at localhost:3000 (arguments go to `mdbook serve`)";
            command = ''
              docs/figures.sh docs/src/figures
              mdbook serve docs "$@"
            '';
          }
        ];
      };
    };
}
