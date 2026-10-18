# linlog © Fabian Lukas Grubmüller 2026
# Licensed under the EUPL

# The Rocq library of linlog's own under rocq/ (logical path `Linlog'):
# the package `rocq-linlog', built as nixpkgs builds a Rocq library
# (mkRocqDerivation over the library's own Makefile, the reference
# manual's wrapper of `rocq makefile' on _RocqProject), and the check
# `rocq-linlog', which compiles every certificate under rocq/tests against
# the installed library and requires `Print Assumptions' of the soundness
# theorems to answer `Closed under the global context'. Rocq prints
# nothing for a file it accepts, so any other output fails the check, as
# does an `Admitted', `admit', `Axiom', `Parameter' or `Conjecture' in a
# source. Rocq and its standard library are nixpkgs', the ones the
# NanoYalla check uses (rocq.nix).
{ ... }:
{
  perSystem =
    { config, pkgs, ... }:
    let
      inherit (pkgs) lib;
      rocq = pkgs.rocq-core;
      stdlib = pkgs.rocqPackages.stdlib;
      userContrib = "lib/coq/${rocq.rocq-version}/user-contrib";

      # The library's version is the opam file's.
      version = builtins.head (
        builtins.match ''.*version: "([0-9.]+)".*'' (builtins.readFile ../rocq/rocq-linlog.opam)
      );

      library = lib.fileset.toSource {
        root = ../rocq;
        fileset = lib.fileset.unions [
          ../rocq/_RocqProject
          ../rocq/Makefile
          ../rocq/theories
        ];
      };

      tests = lib.fileset.toSource {
        root = ../rocq/tests;
        fileset = lib.fileset.fileFilter (file: file.hasExt "v") ../rocq/tests;
      };
    in
    {
      packages.rocq-linlog = pkgs.rocqPackages.mkRocqDerivation {
        pname = "linlog";
        opam-name = "rocq-linlog";
        inherit version;
        # A release whose source is this tree: `hash' must be present, and
        # the empty string stands for the fake hash nothing fetches.
        release.${version} = {
          src = library;
          hash = "";
        };
        propagatedBuildInputs = [ stdlib ];
        meta = {
          description = "The Rocq library of linlog: linear logic, proof terms, their checker and its soundness";
          license = lib.licenses.eupl12;
        };
      };

      checks.rocq-linlog =
        pkgs.runCommand "check-rocq-linlog"
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
                ${library}/theories ${tests}; then
              echo "an assumption or an admitted proof in the library"
              exit 1
            fi
            cp ${tests}/*.v .
            for file in *.v; do
              [ "$file" = Assumptions.v ] && continue
              rocq compile "$file" 2>&1 | tee log
              if [ -s log ]; then exit 1; fi
            done
            rocq compile Assumptions.v >assumptions 2>&1 || { cat assumptions; exit 1; }
            cat assumptions
            # One line per theorem the file asks about, and nothing else.
            theorems=$(grep -c '^Print Assumptions' Assumptions.v)
            for _ in $(seq "$theorems"); do echo 'Closed under the global context'; done |
              diff - assumptions
            touch $out
          '';
    };
}
