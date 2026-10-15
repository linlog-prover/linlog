# linlog © Fabian Lukas Grubmüller 2026
# Licensed under the EUPL

# The provers written in Prolog, run by nixpkgs' SWI-Prolog, each as a
# command on one problem file: llprover (Tamura's sequent-calculus prover
# for first-order linear logic, its SWI port of 2020), linTAP (Mantel and
# Otten's tableau prover for classical MLL with `?'), and LinearOne (Moot's
# prover for first-order multiplicative intuitionistic linear logic). The
# drivers `lintap.pl' and `linearone.pl' call each prover's own search on
# one formula or sequent and print its verdict. Each command takes the
# problem's file and the limit of SWI-Prolog's stacks in bytes, which the
# harness sets to the run's memory limit (SWI-Prolog's own default is
# 1 GiB).
{
  fetchurl,
  fetchFromGitHub,
  swi-prolog,
  writeShellApplication,
  runCommand,
}:
let
  # A command `name' running `body' with the sources in `share'.
  tool =
    {
      name,
      version,
      share,
      body,
      source,
    }:
    (writeShellApplication {
      inherit name;
      runtimeInputs = [ swi-prolog ];
      text = body share;
    }).overrideAttrs
      {
        inherit version;
        passthru.source = source;
      };
  llprover-src = fetchurl {
    url = "https://cspsat.gitlab.io/llprover/llprover.pl";
    hash = "sha256-Qnx4NcjZlBCVC4xq9ab6YODn4CCWR/O3Z2Hoxx8Fk4E=";
  };
  lintap-src = fetchurl {
    url = "https://www.leancop.de/lintap/programs/lintap.pl";
    hash = "sha256-AJLShm025z877wSo9setDgnM0Ofh3TabVz4iFyvQJtE=";
  };
  linearone-src = fetchFromGitHub {
    owner = "RichardMoot";
    repo = "LinearOne";
    rev = "d9379a53dbad076d11febc6d3d413c726c7a73c6";
    hash = "sha256-26nx0/CucNcJpM4X6nKD2tWDGAZy0zrKEgqZpyffKTE=";
  };
in
{
  # The file holds llprover's commands: the system, then the sequent.
  llprover = tool {
    name = "llprover";
    version = "2.0";
    share = llprover-src;
    body = src: ''exec swipl --stack-limit="$2" -q -f ${src} -g ll -t halt <"$1"'';
    source = {
      url = "https://cspsat.gitlab.io/llprover/llprover.pl";
      date = "2020-12-30";
      licence = "none formal (\"You can freely distribute or modify this program\")";
    };
  };
  lintap = tool {
    name = "lintap";
    version = "1.08e";
    share = runCommand "lintap-share" { } ''
      mkdir $out
      cp ${lintap-src} $out/lintap.pl
      cp ${./lintap.pl} $out/driver.pl
    '';
    body = share: ''exec swipl --stack-limit="$2" -q ${share}/driver.pl -- "$1"'';
    source = {
      url = "https://www.leancop.de/lintap/programs/lintap.pl";
      date = "2012-03-30";
      licence = "GPL (no version stated)";
    };
  };
  linearone = tool {
    name = "linearone";
    version = "2026-06-11";
    share = runCommand "linearone-share" { } ''
      mkdir $out
      cp ${linearone-src}/*.pl $out/
      cp ${./linearone.pl} $out/driver.pl
    '';
    body = share: ''exec swipl --stack-limit="$2" -q ${share}/driver.pl -- "$1"'';
    source = {
      url = "https://github.com/RichardMoot/LinearOne";
      inherit (linearone-src) rev;
      date = "2026-06-11";
      licence = "LGPL-2.1";
    };
  };
}
