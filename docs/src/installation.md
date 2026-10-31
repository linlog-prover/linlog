# Installation

The command `linlog` is the package `linlog-cli`, on
[crates.io](https://crates.io/crates/linlog-cli), and the flake of this
repository:

```sh
cargo install --locked linlog-cli                 # a Rust toolchain builds it
```

Building needs Rust 1.92 or later; a release raises that only with its
second number (0.1 to 0.2).

### With Nix

The flake's default package is the command (flakes enabled; on NixOS
and with the Determinate installer they are):

```sh
nix run github:linlog-prover/linlog -- --help            # run it once, without installing
nix profile add github:linlog-prover/linlog              # install it into your profile
nix profile add github:linlog-prover/linlog/v0.1.0       # a release, by its tag
nix profile upgrade --all                                # later: the newest of what you installed
nix build github:linlog-prover/linlog                    # or just build it: result/bin/linlog
```

On Nix before 2.27, `nix profile add` is `nix profile install`. The
package carries the EUPL and the embedded fonts' licences under
`share/doc/linlog-cli/`.

In a flake of your own, a NixOS or Home Manager configuration say, take
it as an input:

```nix
{
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  inputs.linlog.url = "github:linlog-prover/linlog/v0.1.0";

  outputs = { nixpkgs, linlog, ... }: {
    nixosConfigurations.HOST = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        ({ pkgs, ... }: {
          environment.systemPackages = [
            linlog.packages.${pkgs.stdenv.hostPlatform.system}.default
          ];
        })
      ];
    };
  };
}
```

(`home.packages` takes it the same way.) Nix builds it from source on
first use, a few minutes; the binary cache under [Development](development.md) holds the
builds of `main` for x86_64-linux and saves that, once it is trusted.

The library is the crate [`linlog`](https://crates.io/crates/linlog):

```toml
[dependencies]
linlog = "0.1"
```

with the default features `parse`, `serialize`, `interactive`, `latex`,
`typst`, `svg` and `rocq`, and `parallel` (a thread pool, not for wasm),
`png` and `pdf` to add. The documentation's examples read and write JSON
with `serde_json`, the caller's own dependency.

The Rocq certificates the command writes (`--format rocq`) are checked
by the NanoYalla kernel, whose installation [Rocq certificates](rocq.md)
describes with them.
