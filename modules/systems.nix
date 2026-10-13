# linlog © Fabian Lukas Grubmüller 2026
# Licensed under the EUPL

# Not `lib.systems.flakeExposed': that adds platforms nobody builds linlog on.
# x86_64-darwin is gone from nixpkgs. CI builds and checks x86_64-linux
# alone; the others evaluate (aarch64-darwin without valgrind, so without
# the ratchet) and are not built anywhere.
{
  systems = [
    "x86_64-linux"
    "aarch64-linux"
    "aarch64-darwin"
  ];
}
