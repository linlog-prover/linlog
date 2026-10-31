# Benchmarks and the comparison

The results, each run in a chapter of its own with its figures:
[the baseline of 2026-10-17](results/baseline-2026-10-17.md), linlog's
engines on the generated families and the LLTP library;
[beside the other provers, 2026-10-15](results/provers-2026-10-15.md),
the comparison with the other provers of linear logic; and
[first order against LinearOne, 2026-10-30](results/first-order-2026-10-30.md).
Every figure is drawn by `bench/plots.py` from the rows of the run's CSV
files when the manual is built (`docs/figures.sh`), so none is drawn by
hand and a figure always shows the rows it names. This chapter says how
the runs are made.

`linlog-bench` (`cargo run --release -p linlog-bench -- …` in a checkout,
or `nix build .#linlog-bench`) times the engines on four kinds of
problems: generated families with known verdicts (`linlog-bench families`
lists them: 3-Partition as a Horn program and as an MLL sequent,
Matsuoka's Partition, random QBF, wide sequents, contexts under Mix,
Petri-net counters, the Lambek calculus in first-order MILL, a chain of
instances of a quantified clause under `!`, a witness among many and
more, each at any size), the problems of the
[LLTP library](https://github.com/meta-logic/lltp) (`nix build .#lltp -o
bench/lltp` fetches it at a pinned commit; its problems under `ILL/` run
intuitionistically), coverability problems in the `.spec` format with
`--spec` (the 176 of the [qcover](https://github.com/blondimi/qcover)
suite: `nix build .#qcover -o bench/qcover`; they run intuitionistic
affine), and problem files of lines `name; mode; expected; copies;
sequent` such as `bench/problems/slow-tests.txt`. `run` runs every
problem in every mode, engine and thread count asked for, each run in a
child process of its own with a time limit, and writes one CSV row per
run with the verdict, the time and the engine's counters; `summary`
prints Markdown tables of CSV files: the problems solved within the time
limit per family and configuration, and a time per problem and
configuration.

```console
$ linlog-bench run --family partition-no=3,4 --engines focus,net --jobs 1,4 --timeout 10 --output runs.csv
[1/8] partition-no/3 classical focus j1: unprovable  0.089 ms, about 0 min left
[2/8] partition-no/3 classical focus j4: unprovable  0.146 ms, about 0 min left
[3/8] partition-no/3 classical net j1: unprovable  3355.671 ms, about 0 min left
[4/8] partition-no/3 classical net j4: unprovable  1020.791 ms, about 0 min left
[5/8] partition-no/4 classical focus j1: unprovable  0.146 ms, about 0 min left
[6/8] partition-no/4 classical focus j4: unprovable  0.303 ms, about 0 min left
[7/8] partition-no/4 classical net j1: unknown timeout 10000.591 ms, about 0 min left
[8/8] partition-no/4 classical net j4: unknown timeout 10000.139 ms, about 0 min left

$ linlog-bench summary runs.csv
…
## partition-no

| problem | runs: classical focus j1 | runs: classical focus j4 | runs: classical net j1 | runs: classical net j4 |
|---|--:|--:|--:|--:|
| partition-no/3 | 89 µs ✗ | 146 µs ✗ | 3.36 s ✗ | 1.02 s ✗ |
| partition-no/4 | 146 µs ✗ | 303 µs ✗ | > 10 s | > 10 s |
```

`summary --before DIR` compares every file with the file of the same
name in `DIR`, an earlier baseline, problem by problem instead: the
verdicts that differ, the problems decided before and not after, one row
per family or LLTP collection and configuration (problems decided before
and after, late verdicts kept apart, where the undecided ended, the
median ratio of the times), and every generated problem side by side;
given again for each earlier baseline, oldest first, it sets them all
side by side before it compares with the last. `summary --against FILE` compares every other file with `FILE` in the
same way whatever the files' names, as the passes under one bias with the
default:

```console
$ linlog-bench summary --before bench/results/2026-09-30 bench/results/2026-10-02/long-1.csv
…
| file | family | configuration | problem | before | after | after/before |
|---|---|---|---|--:|--:|--:|
| long-1 | 3-partition-no | classical auto j1 | 3-partition-no/5 | 302.42 s ✗ | 393 µs ✗ | 1.3e-6× |
| long-1 | counter | classical auto j1 | counter/16 | > 1200 s | 280 µs ✓ |  |
| long-1 | counter | classical auto j1 | counter/64 |  | 85.14 s ✓ |  |
| long-1 | partition-no | classical auto j1 | partition-no/15 |  | > 1200 s |  |
| long-1 | partition-no | classical auto j1 | partition-no/5 | 811.48 s ✗ | 477 µs ✗ | 5.9e-7× |
| long-1 | partition-yes | classical auto j1 | partition-yes/28 |  | > 1200 s |  |
| long-1 | partition-yes | classical auto j1 | partition-yes/7 | > 1200 s | 230 µs ✓ |  |
```

`run --bias rarer|factors` runs the focused engines under that bias and
`run --forward-copies N` the default's forward search within that bound,
and the rows say which. `run --copies none` deepens the copy bound until
the time limit, and `run --pool-after SECONDS` searches on one thread for
that long before a pool of the other `--jobs` threads joins it, as the command does by
default; the rows have the copy bound reached (`copies_reached`). `bench/targets.sh LABEL` runs the target set of the
focused engine's performance work, the instances the first baseline
showed it losing on (the hard families at the sizes that took minutes or
did not finish, and a fixed sample of 113 LLTP problems), on two pinned
cores in a memory-capped user unit, into `bench/targets/LABEL.csv`;
`bench/TARGETS.md` sets the engine before that work beside the engine
after it. On one thread the engine's counters are a function of the
input, so two such files tell whether a change altered the search at all.

`bench/baseline.sh --detach --fresh` takes the whole baseline, every
family, engine and thread count and the whole LLTP library, at once as
a systemd user unit on an otherwise idle machine, in about three hours;
`--arm` instead starts it unattended in the night, at 20:00 (or at once
if that has passed) once the machine is idle, and stops it at 07:00
whatever its state (`--slot=HH:MM-HH:MM` for other times; the script run
again without `--fresh` finishes a stopped baseline).
Every baseline keeps a directory of its own named by the day it started,
`bench/results/DAY/`: the CSV files of its runs, `starts.txt` with the
commit measured, and its tables in `RESULTS.md`, which `bench/RESULTS.md`
copies for the latest baseline; `journalctl --user -fu linlog-baseline`
follows it. A last stage runs again, with more time before the kill and
more memory, the runs that `bench/reruns.txt` lists: those of an earlier
baseline that were killed or crashed and that measurement showed to
finish with more room. The first baseline, of the night of 2026-09-30,
took 9 h 21 min on a 16-core Intel Core Ultra X9 388H, and a supplement
on the next night added that last stage to it in 1 h 28 min; the second,
of 2026-10-02, after the performance work on the focused engine, took
8 h 4 min; the third, of 2026-10-17, of the engines of the first
release, 2 h 58 min, with the intuitionistic library under the command's
default as a pass of its own. `bench/RESULTS.md` has the third's tables
and `bench/COMPARISON.md` sets the three side by side: no verdict differs
between them, and the intuitionistic LLTP pass decides 3 610 problems
within 5 s where the first decided 737 and the second 2 047. Under the
command's default (no copy bound, every core, 2 s) it decides 4 024 of
the 4 495.

`run --tools` runs the same problems through other provers of linear
logic beside linlog's command, each built by the flake at a pinned
version (`nix build .#comparison` has them all, with BenchExec): every
problem is translated into each tool's syntax, every run is a process
measured by BenchExec's `runexec` under the same time and memory limit
(`--timeout`, `--memory-limit`, 4 GiB by default), and a problem outside a
tool's fragment or mode is a row of its own and not run. `ranking`
prints the comparison: the problems each tool solves within the limit,
per group of problems, linlog on one thread against each tool, and every
problem on which tools contradict each other or the known verdict.
`--shard K/N` runs every N-th problem from the K-th, so that jobs on
several machines split a set.

```console
$ linlog-bench run --tools linlog,maude-ill,sympli --lltp lltp/ILL/ILLTP-SYJ-01 --only SYJ101+1,SYJ105+1.003 --timeout 10 --output tools.csv
[1/6] ILLTP-SYJ-01/SYJ101+1.p linlog: proved  3.4 ms, about 0 min left
[2/6] ILLTP-SYJ-01/SYJ101+1.p maude-ill: proved  89.8 ms, about 0 min left
[3/6] ILLTP-SYJ-01/SYJ101+1.p sympli: proved  11.7 ms, about 0 min left
[4/6] ILLTP-SYJ-01/SYJ105+1.003.p linlog: unprovable  4.0 ms, about 0 min left
[5/6] ILLTP-SYJ-01/SYJ105+1.003.p maude-ill: unknown copy_bound 131.5 ms, about 0 min left
[6/6] ILLTP-SYJ-01/SYJ105+1.003.p sympli: unprovable  3.0 ms, about 0 min left

$ linlog-bench ranking tools.csv
…
## Solved within the limit

| tool | solved | proved | unprovable | wrong | late | time | memory | gave up | error | outside | against the header |
|---|---|---|---|---|---|---|---|---|---|---|---|
| linlog | 2 | 1 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| sympli | 2 | 1 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| maude-ill | 1 | 1 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 |
…
```

(linlog's and Sympli's refutations of SYJ105+1.003 count as solved and
against the file's header, which says Theorem; the formula has a
classical countermodel, `a` and `b` false, so the header is what is
wrong.)
`bench/comparison.sh` runs the whole comparison on any machine with Nix,
and `bench/TOOLS.md` has the tools, how they were run and the results.

## Beside the other provers

The provers of linear logic we found, as their own sources state them
(read on 2026-10-15); `bench/TOOLS.md` has every feature with its source
and the results of running all of them on the same problems.

| prover | fragments | modes | refutes | checked proofs | licence |
|---|---|---|---|---|---|
| linlog | MLL to LL, units | classical, intuitionistic, affine, Mix, cyclic and Lambek (MLL) | ✓ | ✓ (and Rocq certificates) | EUPL-1.2 |
| Maude LL prover | LL, ILL | classical, intuitionistic | without `!` | ✗ | none stated |
| llprover | first-order LL | classical, intuitionistic | without `!`, `?` | ✗ | none formal |
| linTAP | MLL with `?` | classical | without `?` | ✗ | GPL |
| Click & coLLecT | LL | classical | ✓ | Coq certificates | LGPL-2.1 |
| Sympli | ILL without `⅋ ?` | intuitionistic | ✓ | its own | BSD-2-Clause |
| LinearOne | first-order MILL | intuitionistic | ✓ | unverified | LGPL-2.1 |
| lolli | MLL, MALL, MELL | classical | ✗ (a depth limit) | ✗ | MIT |
| APLL | LL, ILL | classical, intuitionistic | unless its bound is met | Coq certificates | none stated |
| Proof Net Calculator | unit-free MLL nets | — | no prover | net criteria | none stated |

On the same 4 811 problems (the LLTP library, qcover's coverability
problems and the generated families), each run under 10 s and 4 GiB on
one core and measured by BenchExec, linlog on one thread solves 4 181,
Sympli 1 684, the Maude prover 768 with its intuitionistic system and
31 with its classical one, APLL 612 and 31, llprover 479 and 11, Click &
coLLecT 31, LinearOne 69, linTAP 15 and lolli 14, each of the problems
in its own fragment and modes. Most of linlog's lead is the library's
3 137 Petri nets, which it decides with an engine of its own (3 072,
against Sympli's 811). On the other 1 358 intuitionistic problems Sympli
solves the most, 873 against linlog's 829 (852 by its default), and 132
that linlog does not, mostly refutations of the SYJ collections. No tool proves a
sequent linlog refutes; APLL and Click & coLLecT refute provable
sequents, 1 308 and 12 of them. On the first-order families in
LinearOne's fragment (28 problems) the two agree wherever both decide:
LinearOne decides all 28 and linlog 25, the other three ending at its
recursion and time limits (`bench/TOOLS.md`, "The first-order
families").
