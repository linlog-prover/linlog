# linlog © Fabian Lukas Grubmüller 2026
# Licensed under the EUPL

# The other provers of the comparison (`bench/TOOLS.md'), each built at a
# pinned version from its own source, since several state no licence or a
# licence other than linlog's; their derivations live in `_provers/'.
# `comparison' is everything `bench/comparison.sh' runs: linlog's command
# and harness, BenchExec, the provers, Python with matplotlib for the
# figures, and `share/linlog-comparison/tools.json', the record of every
# tool's version and source that the harness copies into a run's record
# (each `--tools' name to its prover's).
{
  perSystem =
    {
      config,
      lib,
      pkgs,
      ...
    }:
    let
      provers = {
        maude-ll = pkgs.callPackage ./_provers/maude-ll.nix { };
        click-and-collect = pkgs.callPackage ./_provers/click-and-collect.nix { };
        sympli = pkgs.callPackage ./_provers/sympli.nix { };
        lolli = pkgs.callPackage ./_provers/lolli.nix { };
        apll = pkgs.callPackage ./_provers/apll.nix { };
      }
      // pkgs.callPackages ./_provers/prolog.nix { };
      linlog = config.packages.linlog-cli;
      # Each name of `run --tools' to the program that runs it.
      tools = {
        linlog = linlog;
        linlog-default = linlog;
        maude-ill = provers.maude-ll;
        maude-cll = provers.maude-ll;
        click-and-collect = provers.click-and-collect;
        llprover-cll = provers.llprover;
        llprover-ill = provers.llprover;
        lintap = provers.lintap;
        linearone = provers.linearone;
        sympli = provers.sympli;
        lolli = provers.lolli;
        apll-ll = provers.apll;
        apll-ill = provers.apll;
      };
      record = pkgs.writeTextDir "share/linlog-comparison/tools.json" (
        builtins.toJSON (
          lib.mapAttrs (
            _: p:
            {
              inherit (p) version;
              package = p.pname or p.name;
            }
            // (p.passthru.source or { })
          ) tools
        )
      );
    in
    {
      packages = lib.mapAttrs' (name: lib.nameValuePair "prover-${name}") provers // {
        comparison = pkgs.symlinkJoin {
          name = "linlog-comparison";
          paths = [
            linlog
            config.packages.linlog-bench
            pkgs.benchexec
            (pkgs.python3.withPackages (ps: [ ps.matplotlib ]))
            record
          ]
          ++ lib.attrValues provers;
        };
      };
    };
}
