# linlog © Fabian Lukas Grubmüller 2026
# Licensed under the EUPL

# `msrv': the workspace, every target and every feature, checked with the
# oldest toolchain it claims to support, the `rust-version' of
# Cargo.toml's [workspace.package], against the locked rust-overlay. A
# dependency bumped by `cargo update', or a language feature newer than
# that version, fails here rather than at a user's. The dependencies are
# checked too, with that toolchain, so nothing is shared with the other
# checks' builds.
{ inputs, ... }:
{
  perSystem =
    { pkgs, workspace, ... }:
    let
      version = (builtins.fromTOML (builtins.readFile ../Cargo.toml)).workspace.package.rust-version;
      oldest = p: (inputs.rust-overlay.lib.mkRustBin { } p).stable.${version + ".0"}.minimal;
      craneLib = (inputs.crane.mkLib pkgs).overrideToolchain oldest;
    in
    {
      checks.msrv = craneLib.mkCargoDerivation (
        workspace.commonArgs
        // {
          pname = "linlog-msrv";
          inherit version;
          cargoArtifacts = null;
          buildPhaseCargoCommand = "cargo check --locked --workspace --all-targets --all-features";
          doInstallCargoArtifacts = false;
        }
      );
    };
}
