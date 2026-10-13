# linlog © Fabian Lukas Grubmüller 2026
# Licensed under the EUPL

# The `ratchet' check: every journey's instruction count under callgrind
# against its ceiling in bench/ceilings.csv. Counts do not depend on the
# build machine's load, so this check means something where the timings of
# `bench' do not; `valgrind.out', since the package's first output is its
# manual. Only where valgrind runs: not on aarch64-darwin, where nixpkgs
# marks it broken. `valgrindRuns' says where, for the devshell too.
{
  perSystem =
    {
      config,
      lib,
      pkgs,
      ...
    }:
    let
      valgrindRuns =
        lib.meta.availableOn pkgs.stdenv.hostPlatform pkgs.valgrind && !pkgs.valgrind.meta.broken;
    in
    {
      _module.args = { inherit valgrindRuns; };

      checks = lib.optionalAttrs valgrindRuns {
        ratchet =
          pkgs.runCommand "check-ratchet"
            {
              nativeBuildInputs = [
                config.packages.linlog-bench
                pkgs.valgrind.out
              ];
            }
            ''
              linlog-bench ratchet --check --jobs "$NIX_BUILD_CORES" --ceilings ${../bench/ceilings.csv}
              touch $out
            '';
      };
    };
}
