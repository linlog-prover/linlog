---
name: linlog
description: Decides, checks, proves step by step and exports sequents of linear logic (classical, intuitionistic, affine, with Mix, cyclic, the Lambek calculus) and of ordinary propositional logic with the `linlog` command. Use it to learn whether a sequent is provable and why not, to get a checked proof as a derivation tree, proof net, JSON, LaTeX, Typst, SVG, PNG, PDF or Rocq certificate, to check a proof file, to apply proof steps from a script, or to decide many sequents in one call.
license: EUPL-1.2
compatibility: Needs the `linlog` command, from `cargo install --locked linlog-cli` or `nix run github:linlog-prover/linlog -- ARGS`.
---

# linlog

> `linlog` decides sequents of linear logic and of ordinary propositional logic. A proof it prints has passed an independent checker; an unprovable verdict says why where it can; an unknown verdict names the limit that ended the search and the flag that raises it.

[reference.md](reference.md) beside this file has every command, flag, format and JSON form with examples; `linlog COMMAND --help` documents every flag.

**Commands.** `linlog prove SEQUENT` decides a sequent and prints a proof; `linlog check FILE` checks a proof that `prove --format json` wrote; `linlog eliminate FILE` eliminates its cuts step by step; `linlog interact SEQUENT` applies proof steps read from standard input; `linlog seq print|json|fragment SEQUENT` prints the sequent, writes its JSON or names its fragment (MLL, MALL, MELL, LL, …).

**Syntax.** Quote the sequent for the shell: `"A, A -o B |- B"` (`⊢` also works; either side may be empty). Connectives: `*` (⊗), `par` or `|` (⅋), `&`, `+` (⊕), `-o` (⊸), `~A` or `A^` (negation), `!A`, `?A`; units `1`, `bot`, `top`, `0`. Binding, tightest first: `~ ! ?`, then `*`, `|`, `&`, `+`, and `-o`, which groups to the right. Atoms are identifiers.

**The logic** is classical linear logic unless a flag says otherwise: `-i` intuitionistic (hypotheses left of `|-`, exactly one formula right of it, and no `~`, `par`, `|`, `?` or `bot`: write `A -o B`), `-a` affine (a hypothesis may go unused), `--mix`, `--cyclic` (order kept up to rotation), `--lambek` (with `A \ B` and `B / A`). `--logic classical|intuitionistic|minimal` reads ordinary logic instead, with `->`, `/\`, `\/`, `~`, `<->`, `true` and `false`, and answers `valid` or `not valid`.

**The answer.** The first line is the verdict, then the fragment, the mode and the engine in parentheses, and after a colon the reason. The exit status says the same:

- 0: provable (`valid` under `--logic`, `valid proof` for `check`);
- 1: unprovable (`not valid`, an invalid proof);
- 2: an error, such as a parse error, a flag that does not apply, or `-i` given a classical symbol;
- 3: unknown: a limit ended the search first. That is not unprovable; the line names the flag to raise.

```console
$ linlog prove "A * B |- B * A"
provable (MLL, classical, focus engine)
─────── ax   ─────── ax
⊢ ~B, B      ⊢ ~A, A
──────────────────── ⊗
  ⊢ ~A, ~B, B ⊗ A
  ──────────────── ⅋
  ⊢ ~A ⅋ ~B, B ⊗ A
$ linlog prove -q "A |- B"; echo "exit $?"
unprovable (MLL, classical, focus engine): ~A occurs 1 more time than A in the one-sided sequent, so they cannot all meet in axioms
exit 1
$ linlog prove -q --copies 1 "!(A & B) |- A * B"; echo "exit $?"
unknown (LL, classical, focus engine): the copy bound of 1 was reached after 3.56ms; raise it with --copies N, or lift it with --copies none to deepen it while the time limit lasts
exit 3
$ linlog prove --engine net "A & B |- A"; echo "exit $?"
error: proof nets exist for MLL without units only, not for ALL
exit 2
$ linlog prove --logic intuitionistic -q "~~a -> a"
not valid (intuitionistic logic by the cbn translation into ILL, two-sided engine): the image is unprovable: the search was exhaustive
```

The derivation is one-sided (`Γ ⊢ Δ` read as `⊢ ~Γ, Δ`); `-i` and `--logic` draw two-sided ones, and `--classical-sides two` a classical one as written.

**Limits.** By default a search stops after `--timeout 2s` (`none` for no limit, counted from the command's start) and within `--memory-limit 1GiB`; with `!` or `?` it deepens how often a `?` formula may be copied until it decides, and `--copies N` caps that. `--work-limit N` gives the same answer on every machine; `--deterministic` makes the proof and `--stats` reproducible.

**Output.** `-q` prints the verdict line alone. `--format json` writes one JSON object: `verdict` (`proved`, `unprovable` or `unknown`), `refutation` or `reason`, `fragment`, `mode`, `engine`, `statistics` and the proof, which `check` reads back. `check` takes the logic from its flags, never from the file: check a proof of `prove -i` with `check -i` (the file's `"mode"` says which logic it was found in), or it is checked as the classical one-sided sequent. `--format latex|typst|svg|png|pdf|rocq` or `--output FILE` with that extension exports the derivation; `--net` gives the proof net instead (MLL and MELL without units); `--stats` adds the search's counters.

```console
$ linlog prove --format json "A |- A" > proof.json
$ linlog check -q proof.json
valid proof of ⊢ ~A, A (classical)
$ linlog prove --format latex "A, A -o B |- B"
% provable (MLL, classical, focus engine)
\begin{prooftree}
\infer0[$\mathrm{ax}$]{\vdash A^\bot, A}
\infer0[$\mathrm{ax}$]{\vdash B^\bot, B}
\infer2[$\otimes$]{\vdash A^\bot, A \otimes B^\bot, B}
\end{prooftree}
```

**Many sequents.** `--input-format lines` reads `NAME: SEQUENT` or `SEQUENT` per line (from `--file` or standard input; blank lines and `#` comments skipped) and answers `NAME: VERDICT` per line in input order; the exit status is the worst: an error (2) before unknown (3) before unprovable (1) before proved (0). `--format json` makes each answer a JSON line.

**Step by step.** `interact` reads one command per line: `goals`, `rules G P`, `apply G P RULE [P…]` (goal G, the formula at position P; for `*` the positions of G's other formulas that go to the left premise follow, and no other rule takes more positions), `cut G C [P…]` (a cut on the formula C, the positions P going to the premise that proves it), `undo`, `close [G]` (the search closes a goal), `show`, `proof [FILE]` (checks the finished proof; a FILE gets the proof's JSON, which `check` reads), `save FILE` and `load FILE` (the session's own JSON, for `load` only), `quit`.

The session starts with goal 0; a step closes its goal and opens new ones with new numbers (`opened goal 1: ⊢ 0: ~A, 1: A`), each goal's formulas numbered from 0, so read the numbers from the output (or `goals`) before the next step. A RULE is `ax` (on either literal), `*`, `par`, `&`, `+1`, `+2`, `1`, `bot`, `top`, `?d`, `?c`, `?w`, `!`, `mix` or `wk`, and with `-i` the two-sided names (`-oL`, `-oR`, `*L`, `*R`, …, in reference.md). It exits 0 when the session ends with a checked proof, 1 otherwise.

```console
$ printf 'apply 0 2 * 0\nclose\nproof\n' | linlog interact "A, B |- A * B"
opened goal 1: ⊢ 0: ~A, 1: A
opened goal 2: ⊢ 0: ~B, 1: B
goal 1: provable (MLL, classical, focus engine)
goal 2: provable (MLL, classical, focus engine)
no goal is open: `proof` checks the proof
valid proof (classical)
─────── ax   ─────── ax
⊢ ~A, A      ⊢ ~B, B
──────────────────── ⊗
  ⊢ ~A, ~B, A ⊗ B
```

**The library** is the Rust crate `linlog`: `"…".parse::<linlog::Sequent>()`, `linlog::prove(&sequent, Mode::CLASSICAL, &Options::default())` and the `Outcome` it returns; its default bounds the copies at 3, where the command deepens under its time limit.

## Docs

- [Reference](reference.md): every command, flag, format and JSON form, with examples
- [README](https://github.com/linlog-prover/linlog#usage): the usage section, with every command's output
- [Library API](https://linlog-prover.github.io/linlog/linlog/): the Rust crate's documentation, with the text syntax and every JSON form

## Optional

- [Web page](https://linlog-prover.github.io/): the prover in a browser, step by step, for people
- [Rocq library](https://github.com/linlog-prover/linlog/tree/main/rocq): the calculi and checkers the `--kernel linlog` certificates are proved with
