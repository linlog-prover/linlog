# First order against LinearOne, 2026-10-30

linlog's first-order families against LinearOne, Moot's prover of
first-order MILL, the only other prover of first-order linear logic
that the comparison runs: `lambek-fo` (the Lambek calculus in Moot and
Piazza's translation into MILL1, at 16, 64, 256 and 512 formulas, five
shapes) and `witness-fo` (a witness among 4, 8, 10 and 12, four shapes,
of which LinearOne's fragment holds the MILL1 shapes #2 and #3).
[bench/TOOLS.md](https://github.com/linlog-prover/linlog/blob/main/bench/TOOLS.md#the-first-order-families)
says how the problems are translated and discusses the verdicts.

The times are `bench/comparison/first-order/times-1.csv` and
`times-2.csv`: on 2026-10-30, `linlog-bench run --tools
linlog,linearone --family lambek-fo --family witness-fo --timeout 10
--memory-limit 4294967296` in two shards on cores 2 and 3 of an
otherwise quiet Intel Core Ultra X9 388H, measured by runexec 3.35, one
run per problem, linlog on one thread. The run did not record the
commit of linlog's build (its `env.json` leaves `linlog_commit` empty);
the rows were committed in `af34ca95711d`.

| tool | in its fragment | proved | unprovable | undecided | wrong |
|---|---|---|---|---|---|
| linlog, one thread | 36 | 18 | 15 | 3 | 0 |
| LinearOne | 28 | 16 | 12 | 0 | 0 |

On the 25 problems both decide the verdicts are equal, and every
verdict of either equals the family's by construction. linlog takes
4.0 ms in the median and LinearOne 51 ms, linlog the faster on 24;
LinearOne's floor of about 46 ms is SWI-Prolog's start. LinearOne alone
decides three: `lambek-fo/512` #0 and #2, where linlog's two-sided
search meets its recursion limit of 2 048, and `witness-fo/12#3`, which
linlog leaves at the time limit.

<img class="figure-light" src="../figures/light/first-order-2026-10-30/growth.svg" alt="Time against size of each first-order family's shape, linlog and LinearOne, 2026-10-30">
<img class="figure-dark" src="../figures/dark/first-order-2026-10-30/growth.svg" alt="Time against size of each first-order family's shape, linlog and LinearOne, 2026-10-30">

**Time against size.** The wall time of each shape of the two families
against its size, both logarithmic, a line per tool; a size not
decided within 10 s is a hollow marker on the limit's line, and a shape
outside LinearOne's fragment (`witness-fo` #0 and #1) has linlog's line
alone. linlog is ten to twenty times faster at the smallest sizes; on
`lambek-fo` it grows more slowly than LinearOne on #0 and #3 and faster
on #1, #2 and #4, the two meeting at the largest sizes of #1 and #2;
linlog loses on `witness-fo` #3, whose time rises steeply from size 8,
and at size 512 of `lambek-fo` #0 and #2.

<img class="figure-light" src="../figures/light/first-order-2026-10-30/scatter-linearone.svg" alt="Scatter plot of linlog against LinearOne on the first-order families, 2026-10-30">
<img class="figure-dark" src="../figures/dark/first-order-2026-10-30/scatter-linearone.svg" alt="Scatter plot of linlog against LinearOne on the first-order families, 2026-10-30">

**linlog against LinearOne.** A point per problem both cover, LinearOne's
time across and linlog's up, a problem not solved on the dashed line of
the 10 s limit: all but four lie below the diagonal, and those four are
the three linlog leaves undecided and `witness-fo/10#3`, 2.5 s against
LinearOne's 46 ms.

<img class="figure-light" src="../figures/light/first-order-2026-10-30/cactus-intuitionistic.svg" alt="Cactus plot of the first-order families, linlog and LinearOne, 2026-10-30">
<img class="figure-dark" src="../figures/dark/first-order-2026-10-30/cactus-intuitionistic.svg" alt="Cactus plot of the first-order families, linlog and LinearOne, 2026-10-30">

**Problems solved within the limit.** Each tool's problems solved within
10 s, sorted by time: linlog solves 33 of its 36, LinearOne 28 of its
28.
