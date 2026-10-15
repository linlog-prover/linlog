# Step 29 report: linlog beside the other provers

Session `step-29` (Opus 5.5 at `high`), started 2026-10-15 14:03,
unattended; the supervisor is `planning`.

## The checklist

- [x] The public walk (R245): `sequents::{Walk, Visit}` public,
  `Sequent::walk`, `Reading::walk`, `Reading::connective` and
  `IllConnective`; the depth test walks both at 100 000 levels; gated
  (behaviour lock included) and committed alone (sxpyzwqr).
- [x] The tools researched and built by the flake at pinned versions
  (`modules/comparison.nix`, `modules/_provers/`): the Maude prover,
  Click & coLLecT, llprover, linTAP, LinearOne, Sympli, lolli, APLL;
  the Proof Net Calculator is a row of the matrix only (stzntvzo).
- [x] A translator per tool through the public walk
  (`bench/src/translate.rs`) and a driver per tool
  (`bench/src/tools.rs`), each checked on 18 sequents of known verdict
  in both modes: every tool's every answer read right.
- [x] The harness: `run --tools` under BenchExec's `runexec`, the
  columns `tool`, `tool_version`, `translate_ms`, `peak_bytes`, `raw`,
  `--shard K/N`, the run's record `NAME.env.json`, `ranking`
  (rpzlotll); gated.
- [x] The figures (`bench/plots.py`), the script (`bench/comparison.sh`)
  and the CI job (`.github/workflows/comparison.yml`) (rnznzvkz).
- [x] An end-to-end run of the script (`--quick`, 63 problems, 11 tools).
- [x] The reviewer's findings on the translators and readers: five, all
  fixed (below), the gate passed, folded into rpzlotll and stzntvzo.
- [x] `nix flake check` on the code commits: passed at zvwnosrz (the
  first run found the in-process rows one tail short and two `typos`
  hits, both fixed before).
- [x] `bench/TOOLS.md` (the matrix, the method) and the README's short
  form; the drafts of the messages to the authors
  (`plan/notes/comparison-messages.md`).
- [ ] The night's run: the estimate sent to `planning`, the run, its
  rows, tables and figures committed, the results in `bench/TOOLS.md`
  and README.
- [x] The register's entries of step 29 updated ("State after step
  29"); R18, R186 and R187 to be read again after the night.
- [ ] This report complete; the last message to `planning`.

## The outcome

(Written as the step goes; the results come from the night of
2026-10-15.)

- **The public walk** (R245): `sequents::Walk` and `Visit` are public,
  `Visit::{Enter(t), Between(t, index), Exit(t)}` with the index of the
  operand that follows (1 today) so that n-ary connectives and a binder
  fit later; `Sequent::walk(root)` and `Reading::walk(o)` return it;
  `Reading::connective(o)` returns an `IllConnective` (non-exhaustive:
  `Atom`, `One`, `Zero`, `Top`, `Bang`, `Tensor`, `With`, `Plus`,
  `Lollipop`), the one table of what an occurrence is under its
  position, which the crate's own intuitionistic printer now follows.
  The crate's printers keep their own stops with the bracket flag
  (`Stops`, crate-private). The behaviour lock and every test passed
  unchanged.
- **Eight other provers run in batch**, each built by the flake from its
  own source at a pinned revision (`nix build .#prover-NAME`, all of
  them in `.#comparison`): the Maude prover of Olarte et al. (both its
  ILL and its classical system), Click & coLLecT's automatic prover,
  llprover (both systems), linTAP, LinearOne, Sympli, lolli and APLL
  (both systems; Jui-Hsuan Wu's OCaml prover, which the LLTP library's
  README names and the comparison note had missed). The
  Proof Net Calculator is a row of the matrix only: it checks and
  enumerates nets and proves no given sequent, and its sbt build fetches
  from the network.
- **The harness runs them** (`linlog-bench run --tools`), each run a
  process measured by BenchExec's `runexec` under one time and memory
  limit, with a row per run in the CSV the summaries read (five columns
  appended); `ranking` prints the comparison, `bench/plots.py` the
  figures, `bench/comparison.sh` the whole of it on any machine, and
  `.github/workflows/comparison.yml` the same in ten shards on GitHub.
- **Every driver reads every known verdict right**: 18 sequents of known
  verdict, classical and intuitionistic, through every tool whose
  fragment has them, all answered as expected (a bounded failure read as
  unknown). The reviewer (one, fresh context) found no translation or
  reader that publishes a wrong verdict; its findings and what became of
  them are below.

## Decisions (decided unattended)

1. **BenchExec's `runexec`, called by the harness per run**, not the
   `benchexec` front end with a tool-info module per tool: the drivers,
   the CSV and the summaries stay the harness's (R185), and every run
   is still a cgroup measured by BenchExec. `--no-container` (the tools
   read and write files of the harness's directory), and the streams
   pinned by `taskset`, since the user manager here does not delegate
   the cpuset controller and `runexec --cores` refuses to run without
   it.
2. **linlog is measured as its command**, `linlog prove` on the LLTP or
   `.spec` file itself (or the sequent's JSON for a generated problem),
   over the whole process like every other tool (R184), the check of its
   proof included, rather than the harness's in-process timing of the
   search alone.
3. **The limits**: 10 s and 4 GiB for every run, the limit of wall time
   enforced at 11 s so that linlog's own stop answers first; a verdict
   after 10 s counts as late, not solved. The night's budget decided the
   time: about 4 900 problems, up to nine tools each.
4. **Each tool in its own documented configuration**, and every bounded
   failure read as `unknown`: the Maude prover's copy bound 5 from its
   own LLTP script, llprover's threshold 5, lolli's depth 100.
5. **Drivers of our own where a tool has no batch mode**: Click &
   coLLecT's prover is called as its web server's handler calls it, by
   twenty lines of OCaml, and its fixed 3 s of CPU time is read from the
   environment (a one-line patch) so that it gets the run's limit
   rather than through its HTTP API, which would add a server's start
   to every run; linTAP's
   search is replayed under each exponential limit in turn, because its
   `prove/1` never ends on a formula it cannot prove; LinearOne's
   `first_proof/3` with its counters set, without its LaTeX; Sympli
   needed a patch of no-op functions to build with today's MLton; APLL
   is built native with OCaml 4.14 rather than the bytecode its README
   shows, the faster form of the same program.
6. **A sequent linTAP would misread is refused, not run**: linTAP reads
   a `!` in positive position as an atom, and a one-sided sequent has
   every `!` there. lolli takes its sequent as one argument, so a
   sequent longer than 128 KiB is refused.
7. **Translation from linlog's reading**: an intuitionistic problem is
   handed to an ILL tool as linlog reads it (hypotheses left of `⊢`, one
   goal), atoms renamed `p0`, `p1`, …, every compound formula in
   parentheses; Sympli, which takes no hypotheses, gets `(⊗Γ) ⊸ A`.
8. **linlog's default column on four cores** (12–15, the low-power
   cluster), the sequential tools on one core each (6–11): the default
   races a thread and a pool and needs cores to mean anything. The
   default is the second column, and the page says which cores it had.
9. **A run the cgroup ends for memory is that tool's**, `unknown` with
   the reason `out_of_memory`, linlog's included, as the prompt says for
   step 28's open question 5.
10. **The results are committed** under `bench/comparison/2026-10-15/`:
    `runs.csv`, the streams' records `*.env.json`, `ranking.md` and
    `figures/`; the streams' own files and logs are ignored.

## The reviewer's findings

APLL was added after the review; its translator is the same printer
with its spelling, and its driver was checked by hand on twelve
sequents of known verdict in both systems.

One reviewer (Opus, fresh context) read the translators and the driver
table for soundness. It found no translation or reader that publishes a
wrong `proved` or `unprovable`, and checked the units' spellings, the
signs, the posing of `(⊗Γ) ⊸ A` and of `F1 ⅋ (F2 ⅋ …)`, each reader's
meaning of a failure and the coverage of modes. Its findings:

1. linTAP's translator subtracted one from the number of formulas, which
   panics on the empty sequent and would end the whole run: fixed (the
   empty `⅋` is `0.`, linTAP's `⊥`; llprover's empty side is `[]`), with
   a test.
2. SWI-Prolog caps its stacks at 1 GiB, so the three Prolog provers ran
   under a smaller memory limit than the others, and a stack overflow
   read as an error: fixed, each command takes the run's memory limit as
   `--stack-limit`, and an overflow reads as `out_of_memory`.
3. Wildcard arms over `IllConnective` and atoms defaulting to `p0` would
   print a later connective wrongly: fixed, every arm named and the rest
   an error.
4. Any other end of a run than memory was called a timeout: fixed, the
   reason is runexec's own word.
5. The module's sentence on precedence overstated: corrected (the
   translation relies on `!` and `?` binding tighter than every binary
   connective, which every tool's grammar has).

## For the author

- **The messages to the tools' authors** are drafts in
  `plan/notes/comparison-messages.md`; nothing was sent. The comparison
  should not be published before the answers.
- **The CI job has never run**: its first run (`gh workflow run
  Comparison`, or the Actions tab) is yours. BenchExec needs a cgroup
  from the runner user's systemd, which the job sets up by enabling
  lingering; whether GitHub's runners allow that is untested.
- **Nothing is pushed**, and this step's commits are unsigned
  (`signing.behavior=drop`).
