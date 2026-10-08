# linlog © Fabian Lukas Grubmüller 2026
# Licensed under the EUPL

# The fuzz targets under fuzz/, a workspace of their own: cargo-fuzz needs
# a nightly compiler (libFuzzer's instrumentation and AddressSanitizer are
# unstable flags), which only the `fuzz' devshell carries, pinned by date
# against the locked rust-overlay; the default shell and every check stay
# on the stable toolchain. `nix develop .#fuzz -c fuzz/run.sh' runs them.
{ inputs, ... }:
{
  perSystem =
    { pkgs, ... }:
    let
      nightly = (inputs.rust-overlay.lib.mkRustBin { } pkgs).nightly."2026-09-25".minimal;
    in
    {
      devshells.fuzz = {
        devshell.packages = [
          nightly
          # libfuzzer-sys compiles libFuzzer, which is C++.
          pkgs.stdenv.cc
          pkgs.cargo-fuzz
        ];
      };
    };
}
