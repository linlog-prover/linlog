# linlog © Fabian Lukas Grubmüller 2026
# Licensed under the EUPL

# Click & coLLecT's automatic prover (Callies and Laurent, LGPL-2.1), whose
# only program is a web server: built from its OCaml sources with the
# driver `click-and-collect.ml' in place of the server, which calls the
# prover as the server's handler does. Its search's limit of CPU time, a
# fixed 3 s, is read from the environment, so that it gets the limit every
# tool gets.
{
  stdenv,
  fetchFromGitHub,
  ocamlPackages,
}:
let
  src = fetchFromGitHub {
    owner = "ComputerAidedLL";
    repo = "click-and-collect";
    rev = "6b1c25ff30a1c1dbfccdc0c1cebc272a67ab0137";
    hash = "sha256-r8VqJAoVCmRkHopZHC8NmANna9LFqZ4d/N3gQL+fTj4=";
  };
in
stdenv.mkDerivation {
  pname = "click-and-collect";
  version = "2025-04-07";
  inherit src;
  nativeBuildInputs = with ocamlPackages; [
    ocaml
    dune_3
    findlib
  ];
  buildInputs = [ ocamlPackages.yojson ];
  postPatch = ''
    cp ${./click-and-collect.ml} batch.ml
    substituteInPlace focused_proof.ml --replace-fail \
      'let max_execution_time_in_seconds = 3. in' \
      'let max_execution_time_in_seconds = (try float_of_string (Sys.getenv "CLICK_AND_COLLECT_SECONDS") with Not_found -> 3.) in'
    cat > dune <<'DUNE'
    (executable
     (name batch)
     (modules :standard \ main)
     (libraries yojson str unix))
    DUNE
  '';
  buildPhase = ''
    runHook preBuild
    dune build --root . ./batch.exe
    runHook postBuild
  '';
  installPhase = ''
    runHook preInstall
    install -Dm755 _build/default/batch.exe $out/bin/click-and-collect
    runHook postInstall
  '';
  passthru.source = {
    url = "https://github.com/ComputerAidedLL/click-and-collect";
    inherit (src) rev;
    date = "2025-04-07";
    licence = "LGPL-2.1";
  };
  meta.mainProgram = "click-and-collect";
}
