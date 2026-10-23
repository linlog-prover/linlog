# <picture><source media="(prefers-color-scheme: dark)" srcset="https://linlog-prover.github.io/logo/wordmark-dark.svg"><img alt="linlog" src="https://linlog-prover.github.io/logo/wordmark-light.svg" height="80"></picture> in Rocq

`rocq-linlog`, logical path `Linlog`: the Rocq library of
[linlog](https://github.com/linlog-prover/linlog), a prover of classical
and intuitionistic linear logic, on Rocq 9 and its standard library
alone. It holds the one-sided sequent calculus of linear logic, with Mix
and weakening as parameters, and the two-sided calculus of
intuitionistic linear logic and its affine variant, as inductives;
linlog's proof terms as datatypes, checkers as functions, and for every
mode the theorem that a term the checker accepts proves its sequent. A
certificate that linlog writes (`linlog prove --format rocq --kernel
linlog`) is the sequent, the term and one computation. The invariants
every derivation keeps (the classical reading, an atom's balance, the
count equation) give certificates that a sequent is not derivable.

- `theories/`: the library. `Formulas.v` and `Calculus.v` (the one-sided
  calculus) and `Intuitionistic.v` (the two-sided one) are what a
  certificate's statement rests on; the checkers and their proofs
  appear in the proofs alone.
- `tests/`: certificates compiled against the installed library.
- `bridges/`: the calculi proved equivalent to NanoYalla's `nanoll.v`
  and Yalla's `nanoill.v`, compiled against the files the repository's
  flake pins, never copied.

Build it with `make` in this directory (Rocq on the path), or with
`nix build .#rocq-linlog` from the repository's root, and compile a
certificate with the theories on the load path:

```console
$ linlog prove --mix --format rocq --standalone --output mix.v "A, B |- A, B"
$ rocq compile -R path/to/rocq/theories Linlog mix.v
```

The library is licensed under the EUPL-1.2, as linlog is. The logo above
is linlog's, under CC BY 4.0, served from linlog's site.
