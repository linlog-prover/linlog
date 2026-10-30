# Tools compared

36 problems, 2 tools, limits 10 s and 4294967296 bytes.

## Solved within the limit

| tool | solved | proved | unprovable | wrong | late | time | memory | gave up | error | outside | against the header |
|---|---|---|---|---|---|---|---|---|---|---|---|
| linlog | 33 | 18 | 15 | 0 | 0 | 1 | 0 | 2 | 0 | 0 | 0 |
| linearone | 28 | 16 | 12 | 0 | 0 | 0 | 0 | 0 | 0 | 8 | 0 |

`wrong`: a refutation of a problem some tool proved with a checked proof, or a verdict against a generated problem's by construction, never counted as solved; `late`: decided after the limit; `gave up`: the tool's own bound or no answer; `outside`: not run, outside the tool's fragment or mode; `against the header`: a verdict, counted as solved, that contradicts an LLTP file's header, which a few of the library's files have wrong.

## By group: solved / in the fragment

| group | problems | linlog | linearone |
|---|---|---|---|
| generated families | 36 | 33 / 36 | 28 / 28 |

## linlog against each tool, on the problems both cover

| tool | both cover | both solve | linlog alone | the tool alone | neither |
|---|---|---|---|---|---|
| linearone | 28 | 25 | 0 | 3 | 0 |

Solved by linearone and not by linlog (3):

- lambek-fo/lambek-fo/512#0 (intuitionistic)
- lambek-fo/lambek-fo/512#2 (intuitionistic)
- witness-fo/witness-fo/12#3 (intuitionistic)

## Translation and memory

| tool | translated | translation, median | translation, most | peak memory of solved runs, median | most |
|---|---|---|---|---|---|
| linlog | 36 | 0 µs | 100 µs | 1.2 MiB | 24.2 MiB |
| linearone | 28 | 100 µs | 300 µs | 10.1 MiB | 156.1 MiB |

linlog reads LLTP and `.spec` files itself and the rest as JSON, which the harness writes; the translation time is the harness's, outside the measured run.

## Answered after the limit

None.

## Contradictions

A problem on which one tool proves what another refutes, or a tool decides against the known verdict. A proof is checked where the row says so (linlog's command checks every proof it reports, Sympli validates its own); no tool's `unprovable` is checked, linlog's included.

None.
