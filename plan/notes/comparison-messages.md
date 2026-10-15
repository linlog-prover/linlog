# Messages to the other provers' authors (drafts, not sent)

Drafted by the session of step 29 on 2026-10-15, for the author to send
before the comparison (`bench/TOOLS.md`) is published. Each names the
configuration the tool ran in and its column of the matrix, and asks for
corrections and for the configuration the authors would choose. Attach
`bench/TOOLS.md` and the tool's rows of the night's `runs.csv` (column
`tool`); the matrix's own row for the tool is quoted in each.

## Common text

> Dear …,
>
> I am the author of linlog, a prover for classical and intuitionistic
> linear logic (github.com/linlog-prover/linlog, EUPL-1.2). Before I
> publish a comparison of linlog with the other linear-logic provers I
> could find, I would like you to check what it says about yours.
>
> Your prover was built from its own source at the revision below, by a
> Nix derivation that fetches it (nothing of it is redistributed), and
> run on the LLTP library's problems, qcover's coverability problems
> and some generated families, each problem translated into your input
> syntax, under BenchExec with the same time and memory limit for every
> tool (10 s and 4 GiB on one core). The attached page has the feature
> matrix and the results; your tool's rows are attached too.
>
> Would you tell me whether the configuration below is the one you
> would choose, whether the matrix states your tool's features correctly,
> and whether any of its answers looks wrong to you? I would change
> either before publishing, and say in the page that you checked it.
>
> With thanks, …

## Per tool

- **Maude LL prover** (Carlos Olarte; github.com/carlosolarte/Linear-Logic-Prover-in-Maude
  at `5e1da7fc`, on Maude 3.5.1 unchanged). Intuitionistic problems:
  `red solve(5, ([emp] Γ |~ A)) .` on `ILL-Prover-flat.maude`, the copy
  bound of `utils/exec-ILLTP-flat-file.sh`; a `false` on a sequent with
  `!` is counted unknown (copy bound), not unprovable. Classical ones:
  `search [1] prove(|~ [emp] Γ) =>* proved .` on `cll-system.maude`.
  Questions: is there a configuration without a copy bound that you
  would rather have measured; SYJ103+1 (a theorem) answered `false`
  under every bound from 1 to 40 on Maude 3.5.1 in our checks, which
  may be a difference from Maude 2.7.1. Its licence: none is stated in
  the repository; may we name one?
- **llprover** (Naoyuki Tamura; `llprover.pl` 2.0, the SWI port of
  2020-12-30, on SWI-Prolog 10.0.2): `cll(full).` and `[] --> Γ.` for a
  classical problem, `ill(full).` and `Γ --> A.` for an intuitionistic
  one, the default threshold 5; a failure on a sequent with `!` or `?`
  is counted unknown. Question: is a larger threshold, or deepening it,
  the better configuration for the LLTP problems?
- **linTAP** (Heiko Mantel, Jens Otten; `lintap.pl` 1.08e on SWI-Prolog
  10.0.2): the sequent `⊢ F1, …, Fn` as the formula `F1 @ (F2 @ …)`; a
  driver replays the first clause of `pp/2` under the exponential limit
  0, 1, 2, … until the time limit (`prove/1` would do the same without
  saying it failed), and a failure without `?` is a refutation. A
  sequent with a `!` that the one-sided form puts in positive position
  is not given to it, since it would read it as an atom. Question: is
  that reading right, and the licence (GPL, which version)?
- **Click & coLLecT** (Callies and Laurent, as the comparison note names
  them; github.com/ComputerAidedLL/click-and-collect
  at `6b1c25ff`): the automatic prover called as the server's
  `/auto_prove_sequent` handler calls it (`Proof.auto_weak`, then
  `Focused_proof.prove_sequent`), by a small OCaml driver in place of
  the web server, on `|- Γ`; its limit of CPU time, fixed at 3 s, read
  from the environment and set to the run's. Finding: it answers `NOT
  PROVABLE` (its exhaustive answer) within milliseconds on 12 provable
  MLL sequents, among them the generated `wide-m1` at 256 literals,
  which linlog proves with a checked proof from the
  very file Click & coLLecT read. Question: is that the use you would
  choose, and is a proof it reports checked anywhere?
- **Sympli** (Kaustuv Chaudhuri; github.com/chaudhuri/sympli at
  `50e76a6d`, MLton 20241230 with a patch adding no-op functions to its
  pretty-printing device for the newer SML/NJ library): `Γ ⊢ A` posed as
  `%prove (⊗Γ) -o A.` with `%log validation : highlevel.`; `certificate
  OK` is a checked proof and `ABORT: Saturated` a refutation. Question:
  is it the prover of your CADE 2005 paper with Pfenning, and is there a
  better configuration (`%refute`, an engine)? Its results stand out:
  132 intuitionistic LLTP problems linlog does not solve, and 15
  refutations of files the library marks Theorem that have classical
  countermodels.
- **LinearOne** (Richard Moot; github.com/RichardMoot/LinearOne at
  `d9379a53`, on SWI-Prolog 10.0.2): `first_proof/3` on `[Γ] => A` for
  the multiplicative intuitionistic problems (its fragment), by a driver
  that sets its counters and writes no LaTeX. Question: does it check
  the proofs it finds, and is `first_proof/3` the right entry?
- **lolli** (ibrahimcesar; github.com/ibrahimcesar/lolli at `cb09efc3`,
  v0.2.0): `lolli prove "⊢ Γ" --depth 100` on the classical problems;
  `NOT PROVABLE (within depth limit …)` is counted unknown. A sequent
  longer than one command-line argument (128 KiB) is not given to it.
  Question: is there a setting that decides unprovability, and the
  depth you would choose?
- **APLL** (Jui-Hsuan Wu; github.com/wujuihsuan2016/LL_prover at
  `c55f59d2`, built native with OCaml 4.14): `prover -t FILE` on `|- Γ`,
  and with `-ill` on `Γ |- A`, the backward search under its default
  bound of 3; `-` is counted unknown. Finding: on 1 296 Petri nets of
  the library that linlog proves with checked proofs, the ILL search
  answers `N`, also through its own `-lltp` reader (for example
  `ILL/petri-nets/MCC/SmallOperatingSystem-MT0032DC0008_50_1.p`, `N` in
  0.09 s), and the LL search refutes 12 generated MLL sequents provable
  by construction (`wide-m1` at 256 literals). Questions: would you choose the
  inverse method (`-inv`) or another bound for the LLTP problems, and
  under which licence may the source be used?
- **Proof Net Calculator** (Satoshi Matsuoka): listed in the matrix as a
  net checker and enumerator, not run. Question: is that a fair
  description, and is there a licence for the source?
