# <picture><source media="(prefers-color-scheme: dark)" srcset="https://linlog-prover.github.io/logo/wordmark-dark.svg"><img alt="linlog" src="https://linlog-prover.github.io/logo/wordmark-light.svg" height="80"></picture>

[![DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.23448588.svg)](https://doi.org/10.5281/zenodo.23448588)

A linear logic suite for all your needs: a command line program and a Rust
library that parse, print and decide sequents of classical and
intuitionistic linear logic and their fragments, keep the proofs in a
checkable form and show them as derivation trees or proof nets, prove
step by step, export to LaTeX, Typst, SVG and Rocq, and decide ordinary
propositional logic through its embeddings into linear logic.

linlog's manual, [linlog-prover.github.io/linlog/manual](https://linlog-prover.github.io/linlog/manual/),
describes every command, format and engine with real output, and the
benchmarks and the comparison with the other provers with their figures;
the [web page](https://linlog-prover.github.io/) proves step by step in
the browser on the same library.

## Install

The command `linlog` is the package `linlog-cli`, on
[crates.io](https://crates.io/crates/linlog-cli), and the flake of this
repository:

```sh
cargo install --locked linlog-cli                        # a Rust toolchain builds it
nix run github:linlog-prover/linlog -- --help            # run it once, without installing
nix profile add github:linlog-prover/linlog              # install it into your profile
```

Building needs Rust 1.92 or later. The library is the crate
[`linlog`](https://crates.io/crates/linlog):

```toml
[dependencies]
linlog = "0.1"
```

The manual's
[Installation](https://linlog-prover.github.io/linlog/manual/installation.html)
has the rest: a release by its tag, the flake as an input of a NixOS or
Home Manager configuration, the binary cache, the library's features and
the Rocq kernel that checks the certificates.

## A first proof

```console
$ linlog prove "A, A -o B |- B"
provable (MLL, classical, focus engine)
─────── ax   ─────── ax
⊢ ~A, A      ⊢ ~B, B
──────────────────── ⊗
  ⊢ ~A, A ⊗ ~B, B
$ linlog prove "A |- B"
unprovable (MLL, classical, focus engine): ~A occurs 1 more time than A in the one-sided sequent, so they cannot all meet in axioms
```

The verdict, the fragment the sequent lives in and the engine that
searched, then the proof as a derivation tree of the one-sided sequent
calculus, which the library's independent checker has validated, or
why there is none. `linlog --help` and `linlog <command> --help`
document every option.

## Where to go next

- **The manual**,
  [linlog-prover.github.io/linlog/manual](https://linlog-prover.github.io/linlog/manual/),
  for `main`: [the commands](https://linlog-prover.github.io/linlog/manual/commands.html)
  and their output formats, interactive proving, cut and its
  elimination, proof nets, first-order and ordinary logic, many
  sequents in one call,
  [the benchmarks and the comparison](https://linlog-prover.github.io/linlog/manual/benchmarks.html)
  with the other provers, what exists and what is planned, the
  architecture and development. A release's manual is the `docs/`
  directory of its tag.
- **The web page**, [linlog-prover.github.io](https://linlog-prover.github.io/):
  proving step by step in the browser, with lessons.
- **For AI models**: a guide to the command and the library, the Agent
  Skill [`skills/linlog/`](https://github.com/linlog-prover/linlog/tree/main/skills/linlog)
  (a summary, `SKILL.md`, and the full reference beside it, whose every
  example a test runs), served as
  [llms.txt](https://linlog-prover.github.io/linlog/llms.txt) and
  [llms-full.txt](https://linlog-prover.github.io/linlog/llms-full.txt);
  copy the directory into an agent's skills (`~/.claude/skills/` for
  Claude Code) to give it the guide.
- **The library's API**: [docs.rs/linlog](https://docs.rs/linlog) for the
  releases, [linlog-prover.github.io/linlog](https://linlog-prover.github.io/linlog/)
  for `main`, rebuilt on every push.
- **The changes of every release**:
  [CHANGELOG.md](https://github.com/linlog-prover/linlog/blob/main/CHANGELOG.md).

linlog's code, tests and documentation were written with substantial AI
assistance, by Anthropic's Claude models in Claude Code, in steps the
author planned, directed and accepted. Every proof the library returns
has passed its independent checker, and a Rocq certificate can be
checked by Rocq. linlog is very much inspired by
[Click & coLLecT](https://www.click-and-collect.linear-logic.org);
feature requests and contributions are welcome.

## Citing

Each release is archived on Zenodo. To cite linlog, use the DOI
[10.5281/zenodo.23448588](https://doi.org/10.5281/zenodo.23448588),
which always resolves to the latest version, or the DOI of the version
you used, which its Zenodo record gives. [`CITATION.cff`](CITATION.cff)
holds the entry, and GitHub's "Cite this repository" writes it as BibTeX
or APA.

## Licence

linlog is © Fabian Lukas Grubmüller and licensed under the European
Union Public Licence, version 1.2 (EUPL-1.2), whose text is in
[`LICENSE`](LICENSE).

One part of the repository is someone else's work under its own licence:
the five math fonts in [`cli/fonts/`](cli/fonts), which the `linlog`
command embeds to draw proofs, four under the SIL Open Font License 1.1:
Euler Math, © the American Mathematical Society, Khaled Hosny and
Daniel Flipo ([`cli/fonts/OFL.txt`](cli/fonts/OFL.txt)); STIX Two Math,
© the STIX Fonts Project Authors
([`cli/fonts/STIXTwoMath-Regular-OFL.txt`](cli/fonts/STIXTwoMath-Regular-OFL.txt));
Garamond-Math, © Yuansheng Zhao and Xiangdong Zeng
([`cli/fonts/Garamond-Math-OFL.txt`](cli/fonts/Garamond-Math-OFL.txt));
and Asana Math, © Apostolos Syropoulos, Young Ryu and Claudio Beccari
([`cli/fonts/Asana-Math-OFL.txt`](cli/fonts/Asana-Math-OFL.txt)); and
New Computer Modern Math (its Book weight), © Antonis Tsolomitis, under
the GUST Font License, which is the LaTeX Project Public License 1.3c
([`cli/fonts/NewCMMath-Book-LICENSE.txt`](cli/fonts/NewCMMath-Book-LICENSE.txt),
which also says where the complete package is). The package
`linlog-cli` is therefore `EUPL-1.2 AND OFL-1.1 AND LPPL-1.3c`; the
library `linlog` contains no font. Drawings and documents made with the OFL
fonts are not covered by their licence (its condition 5); a PNG holds
no font, while a PDF embeds the part of the font its text uses, and for
New Computer Modern Math the LPPL makes no exception for documents.

linlog's logo, at the head of this file, is not in the repository: it is
served by [linlog's site](https://linlog-prover.github.io/logo/LICENSE.txt),
© Fabian Lukas Grubmüller, under
[CC BY 4.0](https://creativecommons.org/licenses/by/4.0/). The name
linlog and its mark identify this project, and may not be used to
suggest its endorsement of anything else.

The problem libraries and the other provers that the benchmarks and the
comparison use are not part of this repository: the flake fetches each
at a pinned version when it is needed, under its own licence. The LLTP
library is GPL-3.0, the ILTP library states no licence, qcover's
problems are Apache-2.0, and "Beside the other provers" gives the
provers' licences.
