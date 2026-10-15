# linlog © Fabian Lukas Grubmüller 2026
# Licensed under the EUPL

# The linear-logic prover in Maude of Olarte et al., which ran the LLTP
# library's problems in 2018. Written for Maude 2.7.1; the files load and
# run unchanged on nixpkgs' Maude 3. No licence is stated, so it is fetched,
# never kept here. `maude-ll ill|cll FILE' runs Maude on the focused ILL
# system (`ILL-Prover-flat', yes or no) or the classical one with the
# commands of FILE, which the harness's translator writes.
{
  writeShellApplication,
  fetchFromGitHub,
  maude,
}:
let
  src = fetchFromGitHub {
    owner = "carlosolarte";
    repo = "Linear-Logic-Prover-in-Maude";
    rev = "5e1da7fc7e910019accadd2e95278ccb1b0876cd";
    hash = "sha256-3nDimwISf6SETgTWq5drBmKmHrLLeG40/B1elPis2Hk=";
  };
in
(writeShellApplication {
  name = "maude-ll";
  runtimeInputs = [ maude ];
  text = ''
    case $1 in
      ill) system=ILL-Prover-flat.maude ;;
      cll) system=cll-system.maude ;;
      *) echo "usage: maude-ll ill|cll FILE" >&2; exit 2 ;;
    esac
    exec maude -no-banner -no-advise -no-wrap -batch "${src}/maude-prover/$system" <"$2"
  '';
}).overrideAttrs
  {
    version = "2020-11-03";
    passthru.source = {
      url = "https://github.com/carlosolarte/Linear-Logic-Prover-in-Maude";
      inherit (src) rev;
      date = "2020-11-03";
      licence = "none stated";
      runs_on = "Maude ${maude.version}";
    };
  }
