# Development

The flake's development shell (`nix develop`, or direnv) has the
toolchain and every tool the checks use, and `nix flake check` runs what
CI runs but the online `cargo deny check advisories`. CI uploads what
`main` builds to the public binary cache `linlog-prover` on Cachix, so a
local check fetches the results of everything a change does not touch
instead of rebuilding them. To use it, add it once, as a trusted user of
the Nix daemon (otherwise `cachix` says what to add to the system's
configuration):

```sh
nix run nixpkgs#cachix -- use linlog-prover
```

or add `https://linlog-prover.cachix.org` to `extra-substituters` and
`linlog-prover.cachix.org-1:6K+DDVwwELnSYtfflwUdsHdYuolC7hprdD/r2/KOQZU=`
to `extra-trusted-public-keys` in `nix.conf`.
