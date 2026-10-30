# linlog © Fabian Lukas Grubmüller 2026
# Licensed under the EUPL

# The Rocq library of linlog's own under rocq/ (logical path `Linlog'):
# the package `rocq-linlog', built as nixpkgs builds a Rocq library
# (mkRocqDerivation over the library's own Makefile, the reference
# manual's wrapper of `rocq makefile' on _RocqProject), and the check
# `rocq-linlog', which compiles every certificate under rocq/tests, every
# one the core tests pin for this kernel in core/tests/snapshots/linlog
# and some the CLI writes against the installed library, first-order ones
# among them, requires a certificate with a planted fault to be refused,
# and requires
# `Print Assumptions' of the soundness theorems to answer `Closed under
# the global context'. Rocq prints
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

      snapshots = lib.fileset.toSource {
        root = ../core/tests/snapshots/linlog;
        fileset = lib.fileset.fileFilter (file: file.hasExt "v") ../core/tests/snapshots/linlog;
      };

      # The version `linlog::export::rocq::LINLOG' names must be the
      # library's.
      named = builtins.head (
        builtins.match ''.*pub const LINLOG: &str = "([0-9.]+)";.*'' (
          builtins.readFile ../core/src/export/rocq/mod.rs
        )
      );
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
        assert lib.assertMsg (named == version) "export::rocq::LINLOG is ${named}, the library ${version}";
        pkgs.runCommand "check-rocq-linlog"
          {
            nativeBuildInputs = [
              rocq
              stdlib
              config.packages.rocq-linlog
              config.packages.linlog-cli
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
            # The exporter's certificates, in a directory of their own so
            # that their names meet no test's.
            mkdir exporter
            cp ${snapshots}/*.v exporter/
            cd exporter
            linlog prove --mix --format rocq --standalone --output cli_mix.v 'A, B |- A, B * (C -o C)'
            linlog prove --affine --format rocq --standalone --output cli_affine.v '!A, A -o B, C |- B'
            linlog prove -i --format rocq --standalone --output cli_ill.v '!A, A -o B |- B * !A'
            linlog prove -i --affine --format rocq --standalone --output cli_ill_affine.v \
              'A, B, A -o C |- C * 1'
            # A goal built from top and 0 alone: the reading's goal.
            linlog prove -i --format rocq --standalone --output cli_ill_units.v '0, top |- top'
            linlog prove --kernel linlog --format rocq --standalone --output cli_classical.v \
              '!(A & B) |- !A * !B'
            # Certificates of non-derivability, one-sided and two-sided; the
            # command exits with 1 for an unprovable sequent.
            linlog prove --format rocq --standalone --output cli_refuted.v 'A, A -o B |- B * B' || [ $? -eq 1 ]
            linlog prove --mix --format rocq --standalone --output cli_refuted_mix.v '|- (A * ~A) * 1' || [ $? -eq 1 ]
            linlog prove -i --format rocq --standalone --output cli_refuted_ill.v 'A + B |- A & B' || [ $? -eq 1 ]
            linlog prove --copies 1 --format rocq --standalone --output cli_refuted_unknown.v \
              '!(A -o A * A), A |- ?B' || [ $? -eq 1 ]
            # First-order proofs: classical with a witness no ∀ step
            # introduces, two-sided, affine, and a session's open witness
            # closed into such a constant.
            linlog prove --kernel linlog --format rocq --standalone --output cli_fo.v \
              'forall x. p(x) |- exists y. p(y)'
            linlog prove -i --format rocq --standalone --output cli_fo_ill.v \
              'exists y. forall x. r(x, y) |- forall x. exists y. r(x, y)'
            linlog prove --affine --format rocq --standalone --output cli_fo_affine.v \
              'forall x. p(x), q |- exists y. p(y)'
            linlog prove -i --affine --format rocq --standalone --output cli_fo_ill_affine.v \
              'forall x. p(x), q |- exists y. p(y)'
            printf '%s\n' 'apply 0 0 ∀L' 'apply 1 0 ∃R' 'close 2' 'proof --rocq cli_fo_session.v' |
              linlog interact -i --style rocq.form=standalone 'forall x. p(x) |- exists y. p(y)' >/dev/null
            for file in *.v; do
              rocq compile "$file" 2>&1 | tee log
              if [ -s log ]; then exit 1; fi
            done
            # A planted fault: the first certificate with a wrong witness is
            # refused.
            mkdir ../fault
            sed 's/(Exists (var "p" \[bound 0\]) (eigen 0)/(Exists (var "p" [bound 0]) (eigen 1)/' \
              cli_fo.v >../fault/fault.v
            if cmp -s cli_fo.v ../fault/fault.v; then
              echo "no fault was planted"
              exit 1
            fi
            if rocq compile ../fault/fault.v >/dev/null 2>&1; then
              echo "a certificate with a wrong witness was accepted"
              exit 1
            fi
            cd ..
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
