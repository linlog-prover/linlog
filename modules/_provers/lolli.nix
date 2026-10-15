# linlog © Fabian Lukas Grubmüller 2026
# Licensed under the EUPL

# lolli (ibrahimcesar, MIT), a focused prover for classical MLL, MALL and
# MELL in Rust, whose command takes the sequent as an argument: `lolli
# FILE' passes it the text of FILE, under its default depth of 100.
{
  rustPlatform,
  fetchFromGitHub,
  writeShellApplication,
}:
let
  src = fetchFromGitHub {
    owner = "ibrahimcesar";
    repo = "lolli";
    rev = "cb09efc389415f447bc7c094942ff731dcb955b7";
    hash = "sha256-4BokMmA+wwq+gG4L3Cw8TMCBVyYHTvJh1A8o+ifuthA=";
  };
  lolli = rustPlatform.buildRustPackage {
    pname = "lolli";
    version = "0.2.0";
    inherit src;
    cargoHash = "sha256-+0BJdHOmRmqrfMDXgh9ClbTK/kDTiNeG8UzrOEUT1Kk=";
    cargoBuildFlags = [
      "-p"
      "lolli-cli"
    ];
    doCheck = false;
  };
in
(writeShellApplication {
  name = "lolli";
  text = ''NO_COLOR=1 exec ${lolli}/bin/lolli prove "$(cat "$1")" --depth 100'';
}).overrideAttrs
  {
    inherit (lolli) version;
    passthru.source = {
      url = "https://github.com/ibrahimcesar/lolli";
      inherit (src) rev;
      date = "2025-12-05";
      licence = "MIT";
    };
  }
