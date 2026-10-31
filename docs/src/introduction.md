# linlog

A linear logic suite for all your needs: a command line program and a Rust
library that parse, print and decide sequents of classical and
intuitionistic linear logic and their fragments, keep the proofs in a
checkable form and show them as derivation trees or proof nets, prove
step by step, export to LaTeX, Typst, SVG and Rocq, and decide ordinary
propositional logic through its embeddings into linear logic.

This manual describes the `main` branch of
[the repository](https://github.com/linlog-prover/linlog); a released
version's manual is the `docs/` directory of its tag. linlog's
[web page](https://linlog-prover.github.io/) proves step by step in the
browser on the same library.

## A first proof

```console
$ linlog prove "A, A -o B |- B"
provable (MLL, classical, focus engine)
─────── ax   ─────── ax
⊢ ~A, A      ⊢ ~B, B
──────────────────── ⊗
  ⊢ ~A, A ⊗ ~B, B
```

The verdict, the fragment the sequent lives in and the engine that
searched, then the proof as a derivation tree of the one-sided sequent
calculus, which the library's independent checker has validated.
[The commands](commands.md) has every command; `linlog --help` and
`linlog <command> --help` document every option.

## Documentation, results and how linlog was written

- The library's API: [docs.rs/linlog](https://docs.rs/linlog) for the
  releases, [linlog-prover.github.io/linlog](https://linlog-prover.github.io/linlog/)
  for `main`, rebuilt on every push.
- For AI models: a guide to the command and the library, the Agent Skill
  [`skills/linlog/`](https://github.com/linlog-prover/linlog/tree/main/skills/linlog)
  (a summary, `SKILL.md`, and the full reference beside it, whose every
  example a test runs), served as
  [llms.txt](https://linlog-prover.github.io/linlog/llms.txt) and
  [llms-full.txt](https://linlog-prover.github.io/linlog/llms-full.txt);
  copy the directory into an agent's skills (`~/.claude/skills/` for
  Claude Code) to give it the guide.
- The changes of every release:
  [CHANGELOG.md](https://github.com/linlog-prover/linlog/blob/main/CHANGELOG.md).
- The benchmarks: the latest baseline of the engines on the generated
  families and the LLTP library,
  [bench/RESULTS.md](https://github.com/linlog-prover/linlog/blob/main/bench/RESULTS.md),
  the baselines compared,
  [bench/COMPARISON.md](https://github.com/linlog-prover/linlog/blob/main/bench/COMPARISON.md),
  and the comparison with the other provers of linear logic,
  [bench/TOOLS.md](https://github.com/linlog-prover/linlog/blob/main/bench/TOOLS.md)
  ([Beside the other provers](benchmarks.md#beside-the-other-provers)).
- To cite linlog, GitHub's "Cite this repository" reads
  [CITATION.cff](https://github.com/linlog-prover/linlog/blob/main/CITATION.cff).
- linlog's code, tests and documentation were written with substantial
  AI assistance, by Anthropic's Claude models in Claude Code, in steps
  the author planned, directed and accepted. Every proof the library
  returns has passed its independent checker, and a Rocq certificate
  can be checked by Rocq.
