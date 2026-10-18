# linlog © Fabian Lukas Grubmüller 2026
# Licensed under the EUPL

# The check `rocq-bridges': the modules under rocq/bridges, which prove that
# linlog's Rocq library derives what NanoYalla's `nanoll.v' and Yalla's
# `microyalla/nanoill.v' derive, compiled against the installed library and
# against those two files as the flake inputs pin them. The upstream files
# are LGPL and stay outside this repository: the check copies them from the
# inputs and compiles them unchanged, judged by their exit status. Each
# bridge must compile in silence, and `Assumptions.v' must print `Closed
# under the global context' once per `Print Assumptions' line and nothing
# else. No certificate needs a bridge, so the package `rocq-linlog' leaves
# them out.
{ inputs, ... }:
{
  perSystem =
    { config, pkgs, ... }:
    let
      inherit (pkgs) lib;
      rocq = pkgs.rocq-core;
      stdlib = pkgs.rocqPackages.stdlib;
      userContrib = "lib/coq/${rocq.rocq-version}/user-contrib";

      bridges = lib.fileset.toSource {
        root = ../rocq/bridges;
        fileset = lib.fileset.fileFilter (file: file.hasExt "v") ../rocq/bridges;
      };
    in
    {
      checks.rocq-bridges =
        pkgs.runCommand "check-rocq-bridges"
          {
            nativeBuildInputs = [
              rocq
              stdlib
              config.packages.rocq-linlog
            ];
            ROCQPATH = "${stdlib}/${userContrib}:${config.packages.rocq-linlog}/${userContrib}";
          }
          ''
            export HOME=$TMPDIR
            if grep -rnE '^[[:space:]]*(Axiom|Axioms|Parameter|Parameters|Conjecture|Admitted)\b|\badmit\.' \
                ${bridges}; then
              echo "an assumption or an admitted proof in a bridge"
              exit 1
            fi
            mkdir nanoyalla microyalla bridges
            cp ${inputs.nanoyalla}/nanoyalla/nanoll.v nanoyalla/
            cp ${inputs.yalla}/microyalla/nanoill.v microyalla/
            cp ${bridges}/*.v bridges/
            chmod -R u+w .
            rocq compile -R nanoyalla NanoYalla nanoyalla/nanoll.v >kernel.log 2>&1 ||
              { cat kernel.log; exit 1; }
            rocq compile -R microyalla Yalla.MicroYalla microyalla/nanoill.v >kernel.log 2>&1 ||
              { cat kernel.log; exit 1; }
            flags="-R nanoyalla NanoYalla -R microyalla Yalla.MicroYalla -R bridges LinlogBridges"
            for file in bridges/*.v; do
              [ "$file" = bridges/Assumptions.v ] && continue
              rocq compile $flags "$file" 2>&1 | tee log
              if [ -s log ]; then exit 1; fi
            done
            rocq compile $flags bridges/Assumptions.v >assumptions 2>&1 || { cat assumptions; exit 1; }
            cat assumptions
            # One line per theorem the file asks about, and nothing else.
            theorems=$(grep -c '^Print Assumptions' bridges/Assumptions.v)
            for _ in $(seq "$theorems"); do echo 'Closed under the global context'; done |
              diff - assumptions
            touch $out
          '';
    };
}
