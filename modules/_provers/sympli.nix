# linlog © Fabian Lukas Grubmüller 2026
# Licensed under the EUPL

# Sympli (Chaudhuri, BSD-2-Clause), an inverse-method prover for
# propositional intuitionistic linear logic with focused rules, built with
# nixpkgs' MLton. The patch gives its pretty-printing device the functions
# that the newer SML/NJ library's PP_DEVICE signature asks for, as no-ops;
# nothing of the prover changes. `sympli FILE' runs the script in FILE.
{
  stdenv,
  fetchFromGitHub,
  mlton,
}:
let
  src = fetchFromGitHub {
    owner = "chaudhuri";
    repo = "sympli";
    rev = "50e76a6db8aca256a7de98401d7aa69c6c630c12";
    hash = "sha256-MXwfE5gPxJix+KT/RILkhe0IJ7HGSkY13kmX+4B+4UU=";
  };
in
stdenv.mkDerivation {
  pname = "sympli";
  version = "2018-07-27";
  inherit src;
  patches = [ ./sympli-pp-device.patch ];
  nativeBuildInputs = [ mlton ];
  buildPhase = ''
    runHook preBuild
    mlton -output sympli sym.mlb
    runHook postBuild
  '';
  installPhase = ''
    runHook preInstall
    install -Dm755 sympli $out/bin/sympli
    runHook postInstall
  '';
  passthru.source = {
    url = "https://github.com/chaudhuri/sympli";
    inherit (src) rev;
    date = "2018-07-27";
    licence = "BSD-2-Clause";
  };
  meta.mainProgram = "sympli";
}
