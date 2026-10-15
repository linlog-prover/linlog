# linlog © Fabian Lukas Grubmüller 2026
# Licensed under the EUPL

# APLL (Jui-Hsuan Wu, 2019, no licence stated), a focused prover for LL and
# ILL in OCaml, by backward search or by the inverse method, which reads
# the LLTP syntax and writes Coq certificates for Yalla. Built native with
# the OCaml 4.14 of nixpkgs (it was written for 4.06); `apll' is its
# `prover' program.
{
  stdenv,
  fetchFromGitHub,
  ocaml-ng,
}:
let
  ocamlPackages = ocaml-ng.ocamlPackages_4_14;
  src = fetchFromGitHub {
    owner = "wujuihsuan2016";
    repo = "LL_prover";
    rev = "c55f59d27824d3685f41ed554c2e7d4f8e3e6e47";
    hash = "sha256-5ClvLGtNS4uBilF2RiKMWGIeb7Xu7rcWZY8KR9LlVvA=";
  };
in
stdenv.mkDerivation {
  pname = "apll";
  version = "2019-04-29";
  inherit src;
  nativeBuildInputs = with ocamlPackages; [
    ocaml
    ocamlbuild
    findlib
    menhir
  ];
  buildPhase = ''
    runHook preBuild
    cd src
    ocamlbuild -lib unix -use-menhir -use-ocamlfind prover.native
    runHook postBuild
  '';
  installPhase = ''
    runHook preInstall
    install -Dm755 prover.native $out/bin/apll
    runHook postInstall
  '';
  passthru.source = {
    url = "https://github.com/wujuihsuan2016/LL_prover";
    inherit (src) rev;
    date = "2019-04-29";
    licence = "none stated";
  };
  meta.mainProgram = "apll";
}
