# Benchmark results

The baseline of 2026-10-17 on an Intel(R) Core(TM) Ultra X9 388H
(16 cores, 62 GB), release build, from
`bench/baseline.sh`, which writes this file, the CSV files beside it
(`bench/results/2026-10-17/`, one row per run) and a copy of this file as
`bench/RESULTS.md`, the latest baseline's. It started (once more for
every resumption), with the commit it measured, the LLTP problems of its
later stages chosen from `bench/results/2026-09-30/` and its reruns from
`bench/reruns.txt`:

- 2026-10-17 19:32, commit ed74387d9bf3 (Plan: review step 30b), load average 0.70

Its last part took 2 h 58 min, and the scheduled jobs that ran
meanwhile were: none. The package throttled
130119 times for 1494 s in that part
(platform profile performance, governor powersave, energy preference balance_performance, turbo on).
The journal of `linlog-baseline` says every ten minutes what else used
a CPU. The sequential runs went in four
streams at once, each pinned to a performance core of its own; the
parallel runs had the machine to themselves. A time is the median of up to
three runs; `✓` is proved, `✗` refuted, `?` unknown (`bound`: the copy
bound, `wide`: a context too wide to split, `depth`: the recursion
limit), `>` a time limit reached, `MISMATCH` a verdict against the
problem's known one (for LLTP, the one its header claims). `linlog-bench
summary bench/results/2026-10-17/*.csv` prints the tables again.

## Solved within the time limit

Sequential problems that waited for a CPU for over 1 % of their time (another process slowed them down; `†` in the tables below): 75.

| family | configuration | engines | problems | solved | proved | refuted | late | timeout | bound | other | mismatch | median solved | total solved |
|---|---|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| partition-yes | engines: classical focus j1 | focus | 5 | 5 | 5 | 0 | 0 | 0 | 0 | 0 | 0 | 140 µs | 484.7 ms |
| partition-yes | engines: classical net j1 | net | 5 | 1 | 1 | 0 | 0 | 4 | 0 | 0 | 0 | 2.83 s | 2.83 s |
| partition-no | engines: classical focus j1 | focus | 4 | 4 | 0 | 4 | 0 | 0 | 0 | 0 | 0 | 89.0 ms | 3.89 s |
| partition-no | engines: classical net j1 | net | 4 | 1 | 0 | 1 | 0 | 3 | 0 | 0 | 0 | 3.68 s | 3.68 s |
| 3-partition-mll-yes | engines: classical focus j1 | focus | 4 | 4 | 4 | 0 | 0 | 0 | 0 | 0 | 0 | 57 µs | 231 µs |
| 3-partition-mll-yes | engines: classical net j1 | net | 4 | 4 | 4 | 0 | 0 | 0 | 0 | 0 | 0 | 107 µs | 427 µs |
| 3-partition-mll-no | engines: classical focus j1 | focus | 4 | 4 | 0 | 4 | 0 | 0 | 0 | 0 | 0 | 41 µs | 167 µs |
| 3-partition-mll-no | engines: classical net j1 | net | 4 | 2 | 0 | 2 | 0 | 2 | 0 | 0 | 0 | 56.13 s | 57.96 s |
| wide-m1 | engines: classical focus j1 | focus | 6 | 6 | 6 | 0 | 0 | 0 | 0 | 0 | 0 | 148 µs | 211.0 ms |
| wide-m1 | engines: classical net j1 | net | 6 | 6 | 6 | 0 | 0 | 0 | 0 | 0 | 0 | 199 µs | 596.0 ms |
| wide-m2 | engines: classical focus j1 | focus | 6 | 6 | 6 | 0 | 0 | 0 | 0 | 0 | 0 | 128 µs | 213.5 ms |
| wide-m2 | engines: classical net j1 | net | 6 | 6 | 6 | 0 | 0 | 0 | 0 | 0 | 0 | 205 µs | 620.1 ms |
| wide-m3 | engines: classical focus j1 | focus | 5 | 5 | 5 | 0 | 0 | 0 | 0 | 0 | 0 | 123 µs | 195.8 ms |
| wide-m3 | engines: classical net j1 | net | 5 | 5 | 5 | 0 | 0 | 0 | 0 | 0 | 0 | 211 µs | 638.0 ms |
| wide-m4 | engines: classical focus j1 | focus | 5 | 5 | 5 | 0 | 0 | 0 | 0 | 0 | 0 | 105 µs | 185.0 ms |
| wide-m4 | engines: classical net j1 | net | 5 | 5 | 5 | 0 | 0 | 0 | 0 | 0 | 0 | 258 µs | 661.6 ms |
| additive | engines: classical focus j1 | focus | 4 | 4 | 4 | 0 | 0 | 0 | 0 | 0 | 0 | 391.5 ms | 6.22 s |
| partition-table | engines: classical focus j1 | focus | 13 | 13 | 7 | 6 | 0 | 0 | 0 | 0 | 0 | 82 µs | 2.2 ms |
| partition-table | engines: classical net j1 | net | 13 | 9 | 5 | 4 | 0 | 4 | 0 | 0 | 0 | 52.2 ms | 16.92 s |
| cancellation | engines: classical focus j1 | focus | 1 | 1 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 300 µs | 300 µs |
| 3-partition-yes | families: classical auto j1 | focus | 4 | 4 | 4 | 0 | 0 | 0 | 0 | 0 | 0 | 64 µs | 256 µs |
| 3-partition-no | families: classical auto j1 | focus | 2 | 2 | 0 | 2 | 0 | 0 | 0 | 0 | 0 | 346 µs | 665 µs |
| 3-partition-mll-yes | families: classical auto j1 | focus | 4 | 4 | 4 | 0 | 0 | 0 | 0 | 0 | 0 | 55 µs | 218 µs |
| 3-partition-mll-no | families: classical auto j1 | focus | 4 | 4 | 0 | 4 | 0 | 0 | 0 | 0 | 0 | 41 µs | 165 µs |
| partition-yes | families: classical auto j1 | focus | 9 | 8 | 8 | 0 | 0 | 1 | 0 | 0 | 0 | 13.8 ms | 75.02 s |
| partition-no | families: classical auto j1 | focus | 7 | 5 | 0 | 5 | 0 | 2 | 0 | 0 | 0 | 435 µs | 3.91 s |
| qbf | families: classical auto j1 | focus | 36 | 34 | 6 | 28 | 0 | 0 | 0 | 2 | 0 | 36.1 ms | 837.15 s |
| wide-m1 | families: classical auto j1 | net | 6 | 6 | 6 | 0 | 0 | 0 | 0 | 0 | 0 | 192 µs | 531.3 ms |
| wide-m2 | families: classical auto j1 | net | 6 | 6 | 6 | 0 | 0 | 0 | 0 | 0 | 0 | 194 µs | 564.0 ms |
| wide-m3 | families: classical auto j1 | focus | 7 | 7 | 7 | 0 | 0 | 0 | 0 | 0 | 0 | 128 µs | 218.9 ms |
| wide-m4 | families: classical auto j1 | focus | 8 | 8 | 8 | 0 | 0 | 0 | 0 | 0 | 0 | 134 µs | 210.6 ms |
| mix | families: mix auto j1 | focus | 6 | 5 | 0 | 5 | 0 | 1 | 0 | 0 | 0 | 858.5 ms | 70.73 s |
| counter | families: classical auto j1 | horn | 6 | 6 | 6 | 0 | 0 | 0 | 0 | 0 | 0 | 37 µs | 224 µs |
| counter-over | families: classical auto j1 | horn | 6 | 6 | 0 | 6 | 0 | 0 | 0 | 0 | 0 | 35 µs | 207 µs |
| growing | families: classical auto j1 | focus | 4 | 0 | 0 | 0 | 0 | 0 | 3 | 1 | 0 | – | – |
| chain | families: classical auto j1 | focus | 4 | 4 | 4 | 0 | 0 | 0 | 0 | 0 | 0 | 33.7 ms | 313.0 ms |
| additive | families: classical auto j1 | additive | 4 | 4 | 4 | 0 | 0 | 0 | 0 | 0 | 0 | 58.4 ms | 251.1 ms |
| counter | intuitionistic: intuitionistic auto j1 | horn | 6 | 6 | 6 | 0 | 0 | 0 | 0 | 0 | 0 | 35 µs | 217 µs |
| counter-over | intuitionistic: intuitionistic auto j1 | horn | 6 | 6 | 0 | 6 | 0 | 0 | 0 | 0 | 0 | 35 µs | 213 µs |
| chain | intuitionistic: intuitionistic auto j1 | two-sided | 4 | 4 | 4 | 0 | 0 | 0 | 0 | 0 | 0 | 35.6 ms | 327.9 ms |
| 3-partition-yes | intuitionistic: intuitionistic auto j1 | two-sided | 4 | 4 | 4 | 0 | 0 | 0 | 0 | 0 | 0 | 59 µs | 236 µs |
| 3-partition-no | intuitionistic: intuitionistic auto j1 | two-sided | 1 | 1 | 0 | 1 | 0 | 0 | 0 | 0 | 0 | 273 µs | 273 µs |
| ILL/ILLTP-SYJ-01 | lltp-all-cores: intuitionistic auto j16 | two-sided | 31 | 0 | 0 | 0 | 0 | 0 | 15 | 16 | 0 | – | – |
| ILL/ILLTP-SYJ-cbn | lltp-all-cores: intuitionistic auto j16 | two-sided | 31 | 0 | 0 | 0 | 0 | 0 | 16 | 15 | 0 | – | – |
| ILL/ILLTP-SYJ-cbv | lltp-all-cores: intuitionistic auto j16 | two-sided | 31 | 2 | 2 | 0 | 0 | 2 | 6 | 21 | 0 | 2.46 s | 2.55 s |
| ILL/petri-nets/MCC | lltp-all-cores: intuitionistic auto j16 | horn | 1009 | 991 | 991 | 0 | 0 | 17 | 0 | 1 | 0 | 245 µs | 11.88 s |
| ILL/ILLTP-SYJ-01 | lltp-all-cores-generous: intuitionistic auto j16 | two-sided | 5 | 0 | 0 | 0 | 0 | 0 | 1 | 4 | 0 | – | – |
| ILL/ILLTP-SYJ-cbn | lltp-all-cores-generous: intuitionistic auto j16 | two-sided | 4 | 0 | 0 | 0 | 0 | 0 | 1 | 3 | 0 | – | – |
| ILL/ILLTP-SYJ-cbv | lltp-all-cores-generous: intuitionistic auto j16 | two-sided | 3 | 0 | 0 | 0 | 0 | 0 | 0 | 3 | 0 | – | – |
| ILL/petri-nets/MCC | lltp-classical: classical auto j1 | horn | 3137 | 3071 | 3071 | 0 | 0 | 58 | 0 | 8 | 0 | 386 µs | 39.29 s |
| ILL/misc | lltp-classical: classical auto j1 | focus | 3 | 3 | 3 | 0 | 0 | 0 | 0 | 0 | 0 | 47 µs | 237 µs |
| ILL/Non-theorems | lltp-classical: classical auto j1 | focus | 1 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | – | – |
| ILL/KLE-cbv | lltp-classical: classical auto j1 | focus | 88 | 68 | 64 | 4 | 0 | 0 | 20 | 0 | 0 | 45 µs | 3.1 ms |
| ILL/KLE-cbn | lltp-classical: classical auto j1 | focus | 88 | 69 | 66 | 3 | 0 | 0 | 19 | 0 | 0 | 66 µs | 5.1 ms |
| ILL/KLE-IMP-CONJ/NON-THEOREMS | lltp-classical: classical auto j1 | focus, net | 22 | 22 | 0 | 22 | 0 | 0 | 0 | 0 | 0 | 24 µs | 565 µs |
| ILL/KLE-IMP-CONJ | lltp-classical: classical auto j1 | focus, net | 222 | 162 | 162 | 0 | 0 | 0 | 60 | 0 | 0 | 44 µs | 8.9 ms |
| ILL/KLE-IMP-CONJ/ALT | lltp-classical: classical auto j1 | focus, net | 27 | 27 | 27 | 0 | 0 | 0 | 0 | 0 | 0 | 38 µs | 1.4 ms |
| ILL/KLE-01 | lltp-classical: classical auto j1 | focus | 88 | 51 | 48 | 3 | 0 | 0 | 37 | 0 | 0 | 52 µs | 2.6 ms |
| ILL/ILLTP-SYN-cbv | lltp-classical: classical auto j1 | focus | 19 | 18 | 7 | 11 | 0 | 0 | 1 | 0 | 0 | 39 µs | 729 µs |
| ILL/ILLTP-SYN-cbn | lltp-classical: classical auto j1 | focus, net | 19 | 18 | 6 | 12 | 0 | 0 | 1 | 0 | 0 | 69 µs | 1.2 ms |
| ILL/ILLTP-SYN-01 | lltp-classical: classical auto j1 | focus | 19 | 13 | 4 | 9 | 0 | 0 | 6 | 0 | 0 | 35 µs | 471 µs |
| ILL/ILLTP-SYJ-cbv | lltp-classical: classical auto j1 | focus | 252 | 40 | 19 | 21 | 0 | 22 | 178 | 12 | 0 | 58 µs | 3.65 s |
| ILL/ILLTP-SYJ-cbn | lltp-classical: classical auto j1 | focus, horn | 252 | 35 | 13 | 22 | 0 | 3 | 190 | 24 | 0 | 123 µs | 4.0 ms |
| ILL/ILLTP-SYJ-01 | lltp-classical: classical auto j1 | focus, horn | 252 | 11 | 10 | 1 | 0 | 2 | 215 | 24 | 0 | 57 µs | 692 µs |
| ILL/ILLTP-LCL-cbv | lltp-classical: classical auto j1 | focus | 2 | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | – | – |
| ILL/ILLTP-LCL-cbn | lltp-classical: classical auto j1 | focus | 2 | 2 | 0 | 2 | 0 | 0 | 0 | 0 | 0 | 149 µs | 225 µs |
| ILL/ILLTP-LCL-01 | lltp-classical: classical auto j1 | focus | 2 | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | – | – |
| CLL/misc | lltp-classical: classical auto j1 | focus, net | 14 | 8 | 8 | 0 | 0 | 0 | 6 | 0 | 0 | 60 µs | 898 µs |
| CLL/Non-theorems | lltp-classical: classical auto j1 | focus | 3 | 3 | 0 | 3 | 0 | 0 | 0 | 0 | 0 | 57 µs | 219 µs |
| ILL/ILLTP-SYJ-01 | lltp-classical-generous: classical auto j1 | focus | 4 | 0 | 0 | 0 | 0 | 1 | 1 | 2 | 0 | – | – |
| ILL/ILLTP-SYJ-cbn | lltp-classical-generous: classical auto j1 | focus | 2 | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 | – | – |
| ILL/ILLTP-SYJ-cbv | lltp-classical-generous: classical auto j1 | focus | 3 | 0 | 0 | 0 | 0 | 1 | 0 | 2 | 0 | – | – |
| ILL/petri-nets/MCC | lltp-classical-generous: classical auto j1 | horn | 25 | 25 | 25 | 0 | 0 | 0 | 0 | 0 | 0 | 237 µs | 36.8 ms |
| ILL/ILLTP-LCL-01 | lltp-copies-10: intuitionistic auto j1 | two-sided | 2 | 2 | 0 | 2 | 0 | 0 | 0 | 0 | 0 | 92 µs | 161 µs |
| ILL/ILLTP-LCL-cbn | lltp-copies-10: intuitionistic auto j1 | two-sided | 2 | 2 | 0 | 2 | 0 | 0 | 0 | 0 | 0 | 125 µs | 183 µs |
| ILL/ILLTP-LCL-cbv | lltp-copies-10: intuitionistic auto j1 | two-sided | 2 | 2 | 0 | 2 | 0 | 0 | 0 | 0 | 0 | 81 µs | 159 µs |
| ILL/ILLTP-SYJ-01 | lltp-copies-10: intuitionistic auto j1 | two-sided | 209 | 51 | 35 | 16 | 0 | 85 | 73 | 0 | 3 | 503 µs | 926.8 ms |
| ILL/ILLTP-SYJ-cbn | lltp-copies-10: intuitionistic auto j1 | two-sided | 209 | 88 | 40 | 48 | 0 | 78 | 43 | 0 | 3 | 170 µs | 8.20 s |
| ILL/ILLTP-SYJ-cbv | lltp-copies-10: intuitionistic auto j1 | two-sided | 209 | 93 | 58 | 35 | 0 | 79 | 37 | 0 | 4 | 345 µs | 10.46 s |
| ILL/ILLTP-SYN-01 | lltp-copies-10: intuitionistic auto j1 | two-sided | 8 | 7 | 3 | 4 | 0 | 0 | 1 | 0 | 0 | 95 µs | 846 µs |
| ILL/ILLTP-SYN-cbn | lltp-copies-10: intuitionistic auto j1 | two-sided | 8 | 8 | 3 | 5 | 0 | 0 | 0 | 0 | 0 | 114 µs | 829 µs |
| ILL/ILLTP-SYN-cbv | lltp-copies-10: intuitionistic auto j1 | two-sided | 8 | 8 | 3 | 5 | 0 | 0 | 0 | 0 | 0 | 50 µs | 536 µs |
| ILL/KLE-01 | lltp-copies-10: intuitionistic auto j1 | two-sided | 38 | 31 | 28 | 3 | 0 | 0 | 7 | 0 | 2 | 126 µs | 19.3 ms |
| ILL/KLE-IMP-CONJ | lltp-copies-10: intuitionistic auto j1 | two-sided | 60 | 60 | 60 | 0 | 0 | 0 | 0 | 0 | 0 | 118 µs | 8.3 ms |
| ILL/KLE-cbn | lltp-copies-10: intuitionistic auto j1 | two-sided | 38 | 36 | 28 | 8 | 0 | 0 | 2 | 0 | 7 | 119 µs | 4.9 ms |
| ILL/KLE-cbv | lltp-copies-10: intuitionistic auto j1 | two-sided | 38 | 33 | 28 | 5 | 0 | 0 | 5 | 0 | 4 | 60 µs | 5.1 ms |
| ILL/Non-theorems | lltp-copies-10: intuitionistic auto j1 | two-sided | 1 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | – | – |
| ILL/petri-nets/MCC | lltp-copies-10: intuitionistic auto j1 | horn | 171 | 171 | 171 | 0 | 0 | 0 | 0 | 0 | 0 | 183 µs | 1.95 s |
| ILL/ILLTP-LCL-01 | lltp-default: intuitionistic auto j16 pool after 0.1s | two-sided | 2 | 2 | 0 | 2 | 0 | 0 | 0 | 0 | 0 | 124 µs | 224 µs |
| ILL/ILLTP-LCL-cbn | lltp-default: intuitionistic auto j16 pool after 0.1s | two-sided | 2 | 2 | 0 | 2 | 0 | 0 | 0 | 0 | 0 | 119 µs | 208 µs |
| ILL/ILLTP-LCL-cbv | lltp-default: intuitionistic auto j16 pool after 0.1s | two-sided | 2 | 2 | 0 | 2 | 0 | 0 | 0 | 0 | 0 | 113 µs | 219 µs |
| ILL/ILLTP-SYJ-01 | lltp-default: intuitionistic auto j16 pool after 0.1s | horn, two-sided | 252 | 101 | 55 | 46 | 0 | 123 | 0 | 28 | 4 | 2.1 ms | 11.69 s |
| ILL/ILLTP-SYJ-cbn | lltp-default: intuitionistic auto j16 pool after 0.1s | horn, two-sided | 252 | 137 | 66 | 71 | 0 | 88 | 0 | 27 | 4 | 368 µs | 9.31 s |
| ILL/ILLTP-SYJ-cbv | lltp-default: intuitionistic auto j16 pool after 0.1s | two-sided | 252 | 117 | 68 | 49 | 0 | 110 | 0 | 25 | 4 | 523 µs | 11.93 s |
| ILL/ILLTP-SYN-01 | lltp-default: intuitionistic auto j16 pool after 0.1s | two-sided | 19 | 19 | 7 | 12 | 0 | 0 | 0 | 0 | 2 | 80 µs | 6.0 ms |
| ILL/ILLTP-SYN-cbn | lltp-default: intuitionistic auto j16 pool after 0.1s | net, two-sided | 19 | 19 | 6 | 13 | 0 | 0 | 0 | 0 | 3 | 87 µs | 1.6 ms |
| ILL/ILLTP-SYN-cbv | lltp-default: intuitionistic auto j16 pool after 0.1s | two-sided | 19 | 19 | 7 | 12 | 0 | 0 | 0 | 0 | 2 | 73 µs | 1.5 ms |
| ILL/KLE-01 | lltp-default: intuitionistic auto j16 pool after 0.1s | two-sided | 88 | 87 | 76 | 11 | 0 | 1 | 0 | 0 | 10 | 94 µs | 725.8 ms |
| ILL/KLE-IMP-CONJ/ALT | lltp-default: intuitionistic auto j16 pool after 0.1s | net, two-sided | 27 | 27 | 27 | 0 | 0 | 0 | 0 | 0 | 0 | 67 µs | 2.0 ms |
| ILL/KLE-IMP-CONJ | lltp-default: intuitionistic auto j16 pool after 0.1s | net, two-sided | 222 | 222 | 222 | 0 | 0 | 0 | 0 | 0 | 0 | 84 µs | 23.1 ms |
| ILL/KLE-IMP-CONJ/NON-THEOREMS | lltp-default: intuitionistic auto j16 pool after 0.1s | net, two-sided | 22 | 22 | 0 | 22 | 0 | 0 | 0 | 0 | 0 | 40 µs | 1.0 ms |
| ILL/KLE-cbn | lltp-default: intuitionistic auto j16 pool after 0.1s | two-sided | 88 | 88 | 76 | 12 | 0 | 0 | 0 | 0 | 11 | 102 µs | 145.4 ms |
| ILL/KLE-cbv | lltp-default: intuitionistic auto j16 pool after 0.1s | two-sided | 88 | 88 | 76 | 12 | 0 | 0 | 0 | 0 | 11 | 73 µs | 209.2 ms |
| ILL/Non-theorems | lltp-default: intuitionistic auto j16 pool after 0.1s | two-sided | 1 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | – | – |
| ILL/misc | lltp-default: intuitionistic auto j16 pool after 0.1s | two-sided | 3 | 3 | 3 | 0 | 0 | 0 | 0 | 0 | 0 | 64 µs | 262 µs |
| ILL/petri-nets/MCC | lltp-default: intuitionistic auto j16 pool after 0.1s | horn | 3137 | 3069 | 3069 | 0 | 0 | 66 | 0 | 2 | 0 | 802 µs | 35.85 s |
| ILL/ILLTP-LCL-01 | lltp-forward: intuitionistic auto j1 bias factors | two-sided | 2 | 2 | 0 | 2 | 0 | 0 | 0 | 0 | 0 | 106 µs | 174 µs |
| ILL/ILLTP-LCL-cbn | lltp-forward: intuitionistic auto j1 bias factors | two-sided | 2 | 2 | 0 | 2 | 0 | 0 | 0 | 0 | 0 | 58 µs | 103 µs |
| ILL/ILLTP-LCL-cbv | lltp-forward: intuitionistic auto j1 bias factors | two-sided | 2 | 2 | 0 | 2 | 0 | 0 | 0 | 0 | 0 | 73 µs | 140 µs |
| ILL/ILLTP-SYJ-01 | lltp-forward: intuitionistic auto j1 bias factors | horn, two-sided | 252 | 99 | 54 | 45 | 0 | 124 | 4 | 25 | 4 | 1.0 ms | 15.14 s |
| ILL/ILLTP-SYJ-cbn | lltp-forward: intuitionistic auto j1 bias factors | horn, two-sided | 252 | 142 | 68 | 74 | 0 | 87 | 0 | 23 | 4 | 229 µs | 18.42 s |
| ILL/ILLTP-SYJ-cbv | lltp-forward: intuitionistic auto j1 bias factors | two-sided | 252 | 118 | 70 | 48 | 0 | 118 | 4 | 12 | 4 | 412 µs | 19.35 s |
| ILL/ILLTP-SYN-01 | lltp-forward: intuitionistic auto j1 bias factors | two-sided | 19 | 19 | 7 | 12 | 0 | 0 | 0 | 0 | 2 | 47 µs | 3.9 ms |
| ILL/ILLTP-SYN-cbn | lltp-forward: intuitionistic auto j1 bias factors | net, two-sided | 19 | 19 | 6 | 13 | 0 | 0 | 0 | 0 | 3 | 34 µs | 760 µs |
| ILL/ILLTP-SYN-cbv | lltp-forward: intuitionistic auto j1 bias factors | two-sided | 19 | 19 | 7 | 12 | 0 | 0 | 0 | 0 | 2 | 37 µs | 884 µs |
| ILL/KLE-01 | lltp-forward: intuitionistic auto j1 bias factors | two-sided | 88 | 86 | 76 | 10 | 0 | 0 | 2 | 0 | 9 | 59 µs | 111.3 ms |
| ILL/KLE-IMP-CONJ/ALT | lltp-forward: intuitionistic auto j1 bias factors | net, two-sided | 27 | 27 | 27 | 0 | 0 | 0 | 0 | 0 | 0 | 33 µs | 905 µs |
| ILL/KLE-IMP-CONJ | lltp-forward: intuitionistic auto j1 bias factors | net, two-sided | 222 | 222 | 222 | 0 | 0 | 0 | 0 | 0 | 0 | 44 µs | 13.6 ms |
| ILL/KLE-IMP-CONJ/NON-THEOREMS | lltp-forward: intuitionistic auto j1 bias factors | net, two-sided | 22 | 22 | 0 | 22 | 0 | 0 | 0 | 0 | 0 | 23 µs | 563 µs |
| ILL/KLE-cbn | lltp-forward: intuitionistic auto j1 bias factors | two-sided | 88 | 87 | 76 | 11 | 0 | 0 | 1 | 0 | 10 | 40 µs | 9.2 ms |
| ILL/KLE-cbv | lltp-forward: intuitionistic auto j1 bias factors | two-sided | 88 | 86 | 76 | 10 | 0 | 0 | 2 | 0 | 9 | 47 µs | 11.6 ms |
| ILL/Non-theorems | lltp-forward: intuitionistic auto j1 bias factors | two-sided | 1 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | – | – |
| ILL/misc | lltp-forward: intuitionistic auto j1 bias factors | two-sided | 3 | 3 | 3 | 0 | 0 | 0 | 0 | 0 | 0 | 38 µs | 122 µs |
| ILL/petri-nets/MCC | lltp-forward: intuitionistic auto j1 bias factors | horn | 3137 | 3071 | 3071 | 0 | 0 | 57 | 0 | 9 | 0 | 405 µs | 37.88 s |
| ILL/ILLTP-LCL-01 | lltp-intuitionistic: intuitionistic auto j1 | two-sided | 2 | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | – | – |
| ILL/ILLTP-LCL-cbn | lltp-intuitionistic: intuitionistic auto j1 | two-sided | 2 | 2 | 0 | 2 | 0 | 0 | 0 | 0 | 0 | 123 µs | 181 µs |
| ILL/ILLTP-LCL-cbv | lltp-intuitionistic: intuitionistic auto j1 | two-sided | 2 | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | – | – |
| ILL/ILLTP-SYJ-01 | lltp-intuitionistic: intuitionistic auto j1 | horn, two-sided | 252 | 11 | 10 | 1 | 0 | 1 | 215 | 25 | 0 | 51 µs | 564 µs |
| ILL/ILLTP-SYJ-cbn | lltp-intuitionistic: intuitionistic auto j1 | horn, two-sided | 252 | 35 | 13 | 22 | 0 | 3 | 190 | 24 | 1 | 146 µs | 4.5 ms |
| ILL/ILLTP-SYJ-cbv | lltp-intuitionistic: intuitionistic auto j1 | two-sided | 252 | 40 | 19 | 21 | 0 | 22 | 178 | 12 | 0 | 62 µs | 3.62 s |
| ILL/ILLTP-SYN-01 | lltp-intuitionistic: intuitionistic auto j1 | two-sided | 19 | 13 | 4 | 9 | 0 | 0 | 6 | 0 | 2 | 45 µs | 583 µs |
| ILL/ILLTP-SYN-cbn | lltp-intuitionistic: intuitionistic auto j1 | net, two-sided | 19 | 18 | 6 | 12 | 0 | 0 | 1 | 0 | 3 | 74 µs | 1.3 ms |
| ILL/ILLTP-SYN-cbv | lltp-intuitionistic: intuitionistic auto j1 | two-sided | 19 | 18 | 7 | 11 | 0 | 0 | 1 | 0 | 2 | 43 µs | 801 µs |
| ILL/KLE-01 | lltp-intuitionistic: intuitionistic auto j1 | two-sided | 88 | 51 | 48 | 3 | 0 | 0 | 37 | 0 | 3 | 53 µs | 2.7 ms |
| ILL/KLE-IMP-CONJ/ALT | lltp-intuitionistic: intuitionistic auto j1 | net, two-sided | 27 | 27 | 27 | 0 | 0 | 0 | 0 | 0 | 0 | 41 µs | 1.4 ms |
| ILL/KLE-IMP-CONJ | lltp-intuitionistic: intuitionistic auto j1 | net, two-sided | 222 | 162 | 162 | 0 | 0 | 0 | 60 | 0 | 0 | 48 µs | 9.5 ms |
| ILL/KLE-IMP-CONJ/NON-THEOREMS | lltp-intuitionistic: intuitionistic auto j1 | net, two-sided | 22 | 22 | 0 | 22 | 0 | 0 | 0 | 0 | 0 | 21 µs | 575 µs |
| ILL/KLE-cbn | lltp-intuitionistic: intuitionistic auto j1 | two-sided | 88 | 69 | 66 | 3 | 0 | 0 | 19 | 0 | 3 | 76 µs | 5.6 ms |
| ILL/KLE-cbv | lltp-intuitionistic: intuitionistic auto j1 | two-sided | 88 | 68 | 64 | 4 | 0 | 0 | 20 | 0 | 3 | 46 µs | 3.2 ms |
| ILL/Non-theorems | lltp-intuitionistic: intuitionistic auto j1 | two-sided | 1 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | – | – |
| ILL/misc | lltp-intuitionistic: intuitionistic auto j1 | two-sided | 3 | 3 | 3 | 0 | 0 | 0 | 0 | 0 | 0 | 42 µs | 219 µs |
| ILL/petri-nets/MCC | lltp-intuitionistic: intuitionistic auto j1 | horn | 3137 | 3071 | 3071 | 0 | 0 | 58 | 0 | 8 | 0 | 423 µs | 40.15 s |
| ILL/ILLTP-SYJ-01 | lltp-intuitionistic-generous: intuitionistic auto j1 | two-sided | 4 | 0 | 0 | 0 | 0 | 0 | 1 | 3 | 0 | – | – |
| ILL/ILLTP-SYJ-cbn | lltp-intuitionistic-generous: intuitionistic auto j1 | two-sided | 2 | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 | – | – |
| ILL/ILLTP-SYJ-cbv | lltp-intuitionistic-generous: intuitionistic auto j1 | two-sided | 3 | 0 | 0 | 0 | 0 | 1 | 0 | 2 | 0 | – | – |
| ILL/petri-nets/MCC | lltp-intuitionistic-generous: intuitionistic auto j1 | horn | 19 | 19 | 19 | 0 | 0 | 0 | 0 | 0 | 0 | 291 µs | 36.7 ms |
| ILL/ILLTP-LCL-01 | lltp-rarer: intuitionistic auto j1 bias rarer | two-sided | 2 | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | – | – |
| ILL/ILLTP-LCL-cbn | lltp-rarer: intuitionistic auto j1 bias rarer | two-sided | 2 | 1 | 0 | 1 | 0 | 0 | 1 | 0 | 0 | 51 µs | 51 µs |
| ILL/ILLTP-LCL-cbv | lltp-rarer: intuitionistic auto j1 bias rarer | two-sided | 2 | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | – | – |
| ILL/ILLTP-SYJ-01 | lltp-rarer: intuitionistic auto j1 bias rarer | horn, two-sided | 252 | 11 | 10 | 1 | 0 | 1 | 215 | 25 | 0 | 43 µs | 549 µs |
| ILL/ILLTP-SYJ-cbn | lltp-rarer: intuitionistic auto j1 bias rarer | horn, two-sided | 252 | 34 | 13 | 21 | 0 | 3 | 192 | 23 | 1 | 54 µs | 1.8 ms |
| ILL/ILLTP-SYJ-cbv | lltp-rarer: intuitionistic auto j1 bias rarer | two-sided | 252 | 40 | 19 | 21 | 0 | 22 | 178 | 12 | 0 | 57 µs | 3.58 s |
| ILL/ILLTP-SYN-01 | lltp-rarer: intuitionistic auto j1 bias rarer | two-sided | 19 | 13 | 4 | 9 | 0 | 0 | 6 | 0 | 2 | 43 µs | 544 µs |
| ILL/ILLTP-SYN-cbn | lltp-rarer: intuitionistic auto j1 bias rarer | net, two-sided | 19 | 18 | 6 | 12 | 0 | 0 | 1 | 0 | 3 | 35 µs | 819 µs |
| ILL/ILLTP-SYN-cbv | lltp-rarer: intuitionistic auto j1 bias rarer | two-sided | 19 | 18 | 7 | 11 | 0 | 0 | 1 | 0 | 2 | 43 µs | 817 µs |
| ILL/KLE-01 | lltp-rarer: intuitionistic auto j1 bias rarer | two-sided | 88 | 51 | 48 | 3 | 0 | 0 | 37 | 0 | 3 | 48 µs | 2.7 ms |
| ILL/KLE-IMP-CONJ/ALT | lltp-rarer: intuitionistic auto j1 bias rarer | net, two-sided | 27 | 27 | 27 | 0 | 0 | 0 | 0 | 0 | 0 | 38 µs | 1.0 ms |
| ILL/KLE-IMP-CONJ | lltp-rarer: intuitionistic auto j1 bias rarer | net, two-sided | 222 | 162 | 162 | 0 | 0 | 0 | 60 | 0 | 0 | 43 µs | 7.4 ms |
| ILL/KLE-IMP-CONJ/NON-THEOREMS | lltp-rarer: intuitionistic auto j1 bias rarer | net, two-sided | 22 | 22 | 0 | 22 | 0 | 0 | 0 | 0 | 0 | 24 µs | 693 µs |
| ILL/KLE-cbn | lltp-rarer: intuitionistic auto j1 bias rarer | two-sided | 88 | 69 | 66 | 3 | 0 | 0 | 19 | 0 | 3 | 39 µs | 2.9 ms |
| ILL/KLE-cbv | lltp-rarer: intuitionistic auto j1 bias rarer | two-sided | 88 | 68 | 64 | 4 | 0 | 0 | 20 | 0 | 3 | 45 µs | 3.2 ms |
| ILL/Non-theorems | lltp-rarer: intuitionistic auto j1 bias rarer | two-sided | 1 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | – | – |
| ILL/misc | lltp-rarer: intuitionistic auto j1 bias rarer | two-sided | 3 | 3 | 3 | 0 | 0 | 0 | 0 | 0 | 0 | 42 µs | 138 µs |
| ILL/petri-nets/MCC | lltp-rarer: intuitionistic auto j1 bias rarer | horn | 3137 | 3071 | 3071 | 0 | 0 | 58 | 0 | 8 | 0 | 421 µs | 39.66 s |
| ILL/ILLTP-SYJ-01 | lltp-recursion: intuitionistic auto j1 | two-sided | 22 | 0 | 0 | 0 | 0 | 20 | 0 | 2 | 0 | – | – |
| ILL/ILLTP-SYJ-cbn | lltp-recursion: intuitionistic auto j1 | two-sided | 22 | 0 | 0 | 0 | 0 | 20 | 0 | 2 | 0 | – | – |
| ILL/ILLTP-SYJ-cbv | lltp-recursion: intuitionistic auto j1 | two-sided | 22 | 0 | 0 | 0 | 0 | 22 | 0 | 0 | 0 | – | – |
| ILL/petri-nets/MCC | lltp-recursion: intuitionistic auto j1 | horn | 929 | 895 | 895 | 0 | 0 | 32 | 0 | 2 | 0 | 3.6 ms | 18.05 s |
| ILL/petri-nets/MCC | lltp-recursion-generous: intuitionistic auto j1 | horn | 9 | 9 | 9 | 0 | 0 | 0 | 0 | 0 | 0 | 2.4 ms | 28.3 ms |
| 3-partition-no | long-1: classical auto j1 | focus | 1 | 1 | 0 | 1 | 0 | 0 | 0 | 0 | 0 | 400 µs | 400 µs |
| partition-no | long-1: classical auto j1 | focus | 2 | 1 | 0 | 1 | 0 | 1 | 0 | 0 | 0 | 441 µs | 441 µs |
| partition-yes | long-1: classical auto j1 | focus | 2 | 1 | 1 | 0 | 0 | 1 | 0 | 0 | 0 | 214 µs | 214 µs |
| counter | long-1: classical auto j1 | horn | 2 | 2 | 2 | 0 | 0 | 0 | 0 | 0 | 0 | 60 µs | 108 µs |
| mix | long-2: mix auto j1 | focus | 1 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | – | – |
| wide-m3 | long-2: classical auto j1 | focus | 1 | 1 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 154 µs | 154 µs |
| wide-m4 | long-2: classical auto j1 | focus | 1 | 1 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 135 µs | 135 µs |
| qbf | long-2: classical auto j1 | focus | 2 | 1 | 0 | 1 | 0 | 0 | 0 | 1 | 0 | 99.6 ms | 99.6 ms |
| 3-partition-yes | parallel: classical auto j1 | focus | 4 | 4 | 4 | 0 | 0 | 0 | 0 | 0 | 0 | 88 µs | 363 µs |
| 3-partition-yes | parallel: classical auto j2 | focus | 4 | 4 | 4 | 0 | 0 | 0 | 0 | 0 | 0 | 248 µs | 946 µs |
| 3-partition-yes | parallel: classical auto j4 | focus | 4 | 4 | 4 | 0 | 0 | 0 | 0 | 0 | 0 | 278 µs | 1.1 ms |
| 3-partition-yes | parallel: classical auto j8 | focus | 4 | 4 | 4 | 0 | 0 | 0 | 0 | 0 | 0 | 423 µs | 1.6 ms |
| 3-partition-yes | parallel: classical auto j16 | focus | 4 | 4 | 4 | 0 | 0 | 0 | 0 | 0 | 0 | 593 µs | 2.4 ms |
| 3-partition-no | parallel: classical auto j1 | focus | 2 | 2 | 0 | 2 | 0 | 0 | 0 | 0 | 0 | 432 µs | 811 µs |
| 3-partition-no | parallel: classical auto j2 | focus | 2 | 2 | 0 | 2 | 0 | 0 | 0 | 0 | 0 | 567 µs | 1.1 ms |
| 3-partition-no | parallel: classical auto j4 | focus | 2 | 2 | 0 | 2 | 0 | 0 | 0 | 0 | 0 | 461 µs | 889 µs |
| 3-partition-no | parallel: classical auto j8 | focus | 2 | 2 | 0 | 2 | 0 | 0 | 0 | 0 | 0 | 562 µs | 1.1 ms |
| 3-partition-no | parallel: classical auto j16 | focus | 2 | 2 | 0 | 2 | 0 | 0 | 0 | 0 | 0 | 674 µs | 1.3 ms |
| partition-yes | parallel: classical auto j1 | focus | 5 | 5 | 5 | 0 | 0 | 0 | 0 | 0 | 0 | 273 µs | 67.85 s |
| partition-yes | parallel: classical auto j2 | focus | 5 | 5 | 5 | 0 | 0 | 0 | 0 | 0 | 0 | 585 µs | 71.23 s |
| partition-yes | parallel: classical auto j4 | focus | 5 | 5 | 5 | 0 | 0 | 0 | 0 | 0 | 0 | 621 µs | 75.18 s |
| partition-yes | parallel: classical auto j8 | focus | 5 | 5 | 5 | 0 | 0 | 0 | 0 | 0 | 0 | 597 µs | 79.52 s |
| partition-yes | parallel: classical auto j16 | focus | 5 | 5 | 5 | 0 | 0 | 0 | 0 | 0 | 0 | 906 µs | 87.86 s |
| partition-no | parallel: classical auto j1 | focus | 4 | 3 | 0 | 3 | 0 | 1 | 0 | 0 | 0 | 568 µs | 3.60 s |
| partition-no | parallel: classical auto j2 | focus | 4 | 3 | 0 | 3 | 0 | 1 | 0 | 0 | 0 | 745 µs | 2.13 s |
| partition-no | parallel: classical auto j4 | focus | 4 | 3 | 0 | 3 | 0 | 1 | 0 | 0 | 0 | 578 µs | 1.24 s |
| partition-no | parallel: classical auto j8 | focus | 4 | 3 | 0 | 3 | 0 | 1 | 0 | 0 | 0 | 614 µs | 807.0 ms |
| partition-no | parallel: classical auto j16 | focus | 4 | 3 | 0 | 3 | 0 | 1 | 0 | 0 | 0 | 850 µs | 583.1 ms |
| qbf | parallel: classical auto j1 | focus | 20 | 19 | 3 | 16 | 0 | 0 | 0 | 1 | 0 | 57.9 ms | 228.24 s |
| qbf | parallel: classical auto j2 | focus | 20 | 14 | 3 | 11 | 0 | 0 | 0 | 6 | 0 | 30.8 ms | 16.41 s |
| qbf | parallel: classical auto j4 | focus | 20 | 14 | 3 | 11 | 0 | 0 | 0 | 6 | 0 | 29.7 ms | 16.38 s |
| qbf | parallel: classical auto j8 | focus | 20 | 14 | 3 | 11 | 0 | 0 | 0 | 6 | 0 | 28.1 ms | 15.65 s |
| qbf | parallel: classical auto j16 | focus | 20 | 14 | 3 | 11 | 0 | 0 | 0 | 6 | 0 | 36.5 ms | 15.41 s |
| mix | parallel: mix auto j1 | focus | 4 | 3 | 0 | 3 | 0 | 1 | 0 | 0 | 0 | 7.39 s | 70.35 s |
| mix | parallel: mix auto j2 | focus | 4 | 3 | 0 | 3 | 0 | 1 | 0 | 0 | 0 | 7.05 s | 99.84 s |
| mix | parallel: mix auto j4 | focus | 4 | 3 | 0 | 3 | 0 | 1 | 0 | 0 | 0 | 7.90 s | 104.49 s |
| mix | parallel: mix auto j8 | focus | 4 | 3 | 0 | 3 | 0 | 1 | 0 | 0 | 0 | 9.50 s | 120.65 s |
| mix | parallel: mix auto j16 | focus | 4 | 2 | 0 | 2 | 0 | 2 | 0 | 0 | 0 | 11.51 s | 13.13 s |
| counter | parallel: classical auto j1 | horn | 6 | 6 | 6 | 0 | 0 | 0 | 0 | 0 | 0 | 63 µs | 350 µs |
| counter | parallel: classical auto j2 | horn | 6 | 6 | 6 | 0 | 0 | 0 | 0 | 0 | 0 | 94 µs | 536 µs |
| counter | parallel: classical auto j4 | horn | 6 | 6 | 6 | 0 | 0 | 0 | 0 | 0 | 0 | 93 µs | 534 µs |
| counter | parallel: classical auto j8 | horn | 6 | 6 | 6 | 0 | 0 | 0 | 0 | 0 | 0 | 93 µs | 528 µs |
| counter | parallel: classical auto j16 | horn | 6 | 6 | 6 | 0 | 0 | 0 | 0 | 0 | 0 | 91 µs | 541 µs |
| counter-over | parallel: classical auto j1 | horn | 6 | 6 | 0 | 6 | 0 | 0 | 0 | 0 | 0 | 63 µs | 350 µs |
| counter-over | parallel: classical auto j2 | horn | 6 | 6 | 0 | 6 | 0 | 0 | 0 | 0 | 0 | 84 µs | 503 µs |
| counter-over | parallel: classical auto j4 | horn | 6 | 6 | 0 | 6 | 0 | 0 | 0 | 0 | 0 | 77 µs | 472 µs |
| counter-over | parallel: classical auto j8 | horn | 6 | 6 | 0 | 6 | 0 | 0 | 0 | 0 | 0 | 87 µs | 498 µs |
| counter-over | parallel: classical auto j16 | horn | 6 | 6 | 0 | 6 | 0 | 0 | 0 | 0 | 0 | 86 µs | 496 µs |
| wide-m3 | parallel: classical auto j1 | focus | 3 | 3 | 3 | 0 | 0 | 0 | 0 | 0 | 0 | 182 µs | 509 µs |
| wide-m3 | parallel: classical auto j2 | focus | 3 | 3 | 3 | 0 | 0 | 0 | 0 | 0 | 0 | 357 µs | 1.1 ms |
| wide-m3 | parallel: classical auto j4 | focus | 3 | 3 | 3 | 0 | 0 | 0 | 0 | 0 | 0 | 390 µs | 1.1 ms |
| wide-m3 | parallel: classical auto j8 | focus | 3 | 3 | 3 | 0 | 0 | 0 | 0 | 0 | 0 | 489 µs | 1.4 ms |
| wide-m3 | parallel: classical auto j16 | focus | 3 | 3 | 3 | 0 | 0 | 0 | 0 | 0 | 0 | 727 µs | 2.2 ms |
| wide-m4 | parallel: classical auto j1 | focus | 3 | 3 | 3 | 0 | 0 | 0 | 0 | 0 | 0 | 186 µs | 517 µs |
| wide-m4 | parallel: classical auto j2 | focus | 3 | 3 | 3 | 0 | 0 | 0 | 0 | 0 | 0 | 369 µs | 1.1 ms |
| wide-m4 | parallel: classical auto j4 | focus | 3 | 3 | 3 | 0 | 0 | 0 | 0 | 0 | 0 | 368 µs | 1.1 ms |
| wide-m4 | parallel: classical auto j8 | focus | 3 | 3 | 3 | 0 | 0 | 0 | 0 | 0 | 0 | 535 µs | 1.6 ms |
| wide-m4 | parallel: classical auto j16 | focus | 3 | 3 | 3 | 0 | 0 | 0 | 0 | 0 | 0 | 727 µs | 2.3 ms |
| 3-partition-mll-no | parallel-net: classical net j1 | net | 3 | 2 | 0 | 2 | 0 | 1 | 0 | 0 | 0 | 48.79 s | 50.42 s |
| 3-partition-mll-no | parallel-net: classical net j2 | net | 3 | 2 | 0 | 2 | 0 | 1 | 0 | 0 | 0 | 25.09 s | 25.92 s |
| 3-partition-mll-no | parallel-net: classical net j4 | net | 3 | 2 | 0 | 2 | 0 | 1 | 0 | 0 | 0 | 13.41 s | 13.86 s |
| 3-partition-mll-no | parallel-net: classical net j8 | net | 3 | 2 | 0 | 2 | 0 | 1 | 0 | 0 | 0 | 7.58 s | 7.83 s |
| 3-partition-mll-no | parallel-net: classical net j16 | net | 3 | 2 | 0 | 2 | 0 | 1 | 0 | 0 | 0 | 4.42 s | 4.56 s |
| partition-table | parallel-net: classical net j1 | net | 13 | 9 | 5 | 4 | 0 | 4 | 0 | 0 | 0 | 62.3 ms | 16.15 s |
| partition-table | parallel-net: classical net j2 | net | 13 | 9 | 5 | 4 | 0 | 4 | 0 | 0 | 0 | 30.6 ms | 8.04 s |
| partition-table | parallel-net: classical net j4 | net | 13 | 10 | 6 | 4 | 0 | 3 | 0 | 0 | 0 | 16.6 ms | 4.17 s |
| partition-table | parallel-net: classical net j8 | net | 13 | 10 | 6 | 4 | 0 | 3 | 0 | 0 | 0 | 141.7 ms | 3.22 s |
| partition-table | parallel-net: classical net j16 | net | 13 | 10 | 6 | 4 | 0 | 3 | 0 | 0 | 0 | 74.5 ms | 2.34 s |
| wide-m1 | period-16: classical net j1 period 16 | net | 2 | 2 | 2 | 0 | 0 | 0 | 0 | 0 | 0 | 388.2 ms | 394.1 ms |
| wide-m2 | period-16: classical net j1 period 16 | net | 2 | 2 | 2 | 0 | 0 | 0 | 0 | 0 | 0 | 405.9 ms | 412.1 ms |
| 3-partition-mll-no | period-16: classical net j1 period 16 | net | 2 | 1 | 0 | 1 | 0 | 1 | 0 | 0 | 0 | 20.95 s | 20.95 s |
| partition-yes | period-16: classical net j1 period 16 | net | 1 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | – | – |
| partition-no | period-16: classical net j1 period 16 | net | 2 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | – | – |
| wide-m1 | period-1: classical net j1 period 1 | net | 2 | 2 | 2 | 0 | 0 | 0 | 0 | 0 | 0 | 1.22 s | 1.24 s |
| wide-m2 | period-1: classical net j1 period 1 | net | 2 | 2 | 2 | 0 | 0 | 0 | 0 | 0 | 0 | 1.23 s | 1.25 s |
| 3-partition-mll-no | period-1: classical net j1 period 1 | net | 2 | 2 | 0 | 2 | 0 | 0 | 0 | 0 | 0 | 55.13 s | 56.86 s |
| partition-yes | period-1: classical net j1 period 1 | net | 1 | 1 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 2.69 s | 2.69 s |
| partition-no | period-1: classical net j1 period 1 | net | 2 | 1 | 0 | 1 | 0 | 1 | 0 | 0 | 0 | 3.32 s | 3.32 s |
| wide-m1 | period-2: classical net j1 period 2 | net | 2 | 2 | 2 | 0 | 0 | 0 | 0 | 0 | 0 | 818.1 ms | 830.3 ms |
| wide-m2 | period-2: classical net j1 period 2 | net | 2 | 2 | 2 | 0 | 0 | 0 | 0 | 0 | 0 | 803.6 ms | 815.6 ms |
| 3-partition-mll-no | period-2: classical net j1 period 2 | net | 2 | 2 | 0 | 2 | 0 | 0 | 0 | 0 | 0 | 49.10 s | 50.48 s |
| partition-yes | period-2: classical net j1 period 2 | net | 1 | 1 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 5.72 s | 5.72 s |
| partition-no | period-2: classical net j1 period 2 | net | 2 | 1 | 0 | 1 | 0 | 1 | 0 | 0 | 0 | 7.45 s | 7.45 s |
| wide-m1 | period-8: classical net j1 period 8 | net | 2 | 2 | 2 | 0 | 0 | 0 | 0 | 0 | 0 | 448.2 ms | 455.2 ms |
| wide-m2 | period-8: classical net j1 period 8 | net | 2 | 2 | 2 | 0 | 0 | 0 | 0 | 0 | 0 | 463.3 ms | 470.3 ms |
| 3-partition-mll-no | period-8: classical net j1 period 8 | net | 2 | 1 | 0 | 1 | 0 | 1 | 0 | 0 | 0 | 4.78 s | 4.78 s |
| partition-yes | period-8: classical net j1 period 8 | net | 1 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | – | – |
| partition-no | period-8: classical net j1 period 8 | net | 2 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | – | – |

## Counters of the decided problems

| family | configuration | decided | nodes | splits | memo_hits | memo_entries | links | tests |
|---|---|--:|--:|--:|--:|--:|--:|--:|
| partition-yes | engines: classical focus j1 | 5 | 330930 | 2909630 | 277975 | 52955 | 0 | 0 |
| partition-yes | engines: classical net j1 | 1 | 235737 | 0 | 0 | 0 | 1071471 | 1071471 |
| partition-no | engines: classical focus j1 | 4 | 4369900 | 28361297 | 3818668 | 551232 | 0 | 0 |
| partition-no | engines: classical net j1 | 1 | 269950 | 0 | 0 | 0 | 1313537 | 1313537 |
| 3-partition-mll-yes | engines: classical focus j1 | 4 | 36 | 176 | 0 | 36 | 0 | 0 |
| 3-partition-mll-yes | engines: classical net j1 | 4 | 84 | 0 | 0 | 0 | 84 | 84 |
| 3-partition-mll-no | engines: classical focus j1 | 4 | 4 | 96 | 0 | 4 | 0 | 0 |
| 3-partition-mll-no | engines: classical net j1 | 2 | 16931690 | 0 | 0 | 0 | 40487784 | 40487784 |
| wide-m1 | engines: classical focus j1 | 6 | 2390 | 4268900 | 0 | 2390 | 0 | 0 |
| wide-m1 | engines: classical net j1 | 6 | 4768 | 0 | 0 | 0 | 4768 | 1264 |
| wide-m2 | engines: classical focus j1 | 6 | 2390 | 3203460 | 0 | 2390 | 0 | 0 |
| wide-m2 | engines: classical net j1 | 6 | 4768 | 0 | 0 | 0 | 4768 | 1264 |
| wide-m3 | engines: classical focus j1 | 5 | 2375 | 2848074 | 0 | 2375 | 0 | 0 |
| wide-m3 | engines: classical net j1 | 5 | 4740 | 0 | 0 | 0 | 4740 | 1239 |
| wide-m4 | engines: classical focus j1 | 5 | 2373 | 2670434 | 0 | 2373 | 0 | 0 |
| wide-m4 | engines: classical net j1 | 5 | 4736 | 0 | 0 | 0 | 4736 | 1280 |
| additive | engines: classical focus j1 | 4 | 3036271 | 0 | 869899 | 1487175 | 0 | 0 |
| partition-table | engines: classical focus j1 | 13 | 3629 | 13830 | 2550 | 1079 | 0 | 0 |
| partition-table | engines: classical net j1 | 9 | 1321234 | 0 | 0 | 0 | 6276936 | 6276936 |
| cancellation | engines: classical focus j1 | 1 | 925 | 2794 | 713 | 212 | 0 | 0 |
| 3-partition-yes | families: classical auto j1 | 4 | 28 | 140 | 0 | 28 | 0 | 0 |
| 3-partition-no | families: classical auto j1 | 2 | 1894 | 6088 | 1446 | 448 | 0 | 0 |
| 3-partition-mll-yes | families: classical auto j1 | 4 | 36 | 176 | 0 | 36 | 0 | 0 |
| 3-partition-mll-no | families: classical auto j1 | 4 | 4 | 96 | 0 | 4 | 0 | 0 |
| partition-yes | families: classical auto j1 | 8 | 38671826 | 388163209 | 33889921 | 1120561 | 0 | 0 |
| partition-no | families: classical auto j1 | 5 | 4370711 | 28365458 | 3819236 | 551475 | 0 | 0 |
| qbf | families: classical auto j1 | 34 | 825287750 | 5425130223 | 90299364 | 13899059 | 0 | 0 |
| wide-m1 | families: classical auto j1 | 6 | 4768 | 0 | 0 | 0 | 4768 | 1264 |
| wide-m2 | families: classical auto j1 | 6 | 4768 | 0 | 0 | 0 | 4768 | 1264 |
| wide-m3 | families: classical auto j1 | 7 | 3437 | 3551165 | 0 | 3437 | 0 | 0 |
| wide-m4 | families: classical auto j1 | 8 | 3468 | 3330514 | 0 | 3468 | 0 | 0 |
| mix | families: mix auto j1 | 5 | 13394875 | 3139824676 | 12014272 | 1380603 | 0 | 0 |
| counter | families: classical auto j1 | 6 | 120 | 0 | 0 | 120 | 0 | 0 |
| counter-over | families: classical auto j1 | 6 | 171 | 0 | 23 | 148 | 0 | 0 |
| growing | families: classical auto j1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| chain | families: classical auto j1 | 4 | 86060 | 9484732 | 42220 | 928 | 0 | 0 |
| additive | families: classical auto j1 | 4 | 14404893 | 0 | 3788841 | 2442270 | 0 | 0 |
| counter | intuitionistic: intuitionistic auto j1 | 6 | 120 | 0 | 0 | 120 | 0 | 0 |
| counter-over | intuitionistic: intuitionistic auto j1 | 6 | 171 | 0 | 23 | 148 | 0 | 0 |
| chain | intuitionistic: intuitionistic auto j1 | 4 | 86060 | 9484732 | 42220 | 928 | 0 | 0 |
| 3-partition-yes | intuitionistic: intuitionistic auto j1 | 4 | 28 | 140 | 0 | 28 | 0 | 0 |
| 3-partition-no | intuitionistic: intuitionistic auto j1 | 1 | 923 | 2794 | 713 | 210 | 0 | 0 |
| ILL/ILLTP-SYJ-01 | lltp-all-cores: intuitionistic auto j16 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| ILL/ILLTP-SYJ-cbn | lltp-all-cores: intuitionistic auto j16 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| ILL/ILLTP-SYJ-cbv | lltp-all-cores: intuitionistic auto j16 | 2 | 13451242 | 11663067 | 7995792 | 1281547 | 0 | 0 |
| ILL/petri-nets/MCC | lltp-all-cores: intuitionistic auto j16 | 991 | 23708490 | 0 | 16669051 | 7039439 | 0 | 0 |
| ILL/ILLTP-SYJ-01 | lltp-all-cores-generous: intuitionistic auto j16 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| ILL/ILLTP-SYJ-cbn | lltp-all-cores-generous: intuitionistic auto j16 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| ILL/ILLTP-SYJ-cbv | lltp-all-cores-generous: intuitionistic auto j16 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| ILL/petri-nets/MCC | lltp-classical: classical auto j1 | 3071 | 36664493 | 0 | 25351462 | 11313031 | 0 | 0 |
| ILL/misc | lltp-classical: classical auto j1 | 3 | 46 | 74 | 16 | 22 | 0 | 0 |
| ILL/Non-theorems | lltp-classical: classical auto j1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| ILL/KLE-cbv | lltp-classical: classical auto j1 | 68 | 1970 | 945 | 438 | 900 | 0 | 0 |
| ILL/KLE-cbn | lltp-classical: classical auto j1 | 69 | 1761 | 554 | 471 | 784 | 0 | 0 |
| ILL/KLE-IMP-CONJ/NON-THEOREMS | lltp-classical: classical auto j1 | 22 | 24 | 32 | 0 | 23 | 0 | 0 |
| ILL/KLE-IMP-CONJ | lltp-classical: classical auto j1 | 162 | 3767 | 1699 | 876 | 1593 | 116 | 116 |
| ILL/KLE-IMP-CONJ/ALT | lltp-classical: classical auto j1 | 27 | 129 | 164 | 0 | 84 | 20 | 20 |
| ILL/KLE-01 | lltp-classical: classical auto j1 | 51 | 2172 | 452 | 502 | 944 | 0 | 0 |
| ILL/ILLTP-SYN-cbv | lltp-classical: classical auto j1 | 18 | 380 | 251 | 92 | 170 | 0 | 0 |
| ILL/ILLTP-SYN-cbn | lltp-classical: classical auto j1 | 18 | 435 | 104 | 121 | 208 | 0 | 0 |
| ILL/ILLTP-SYN-01 | lltp-classical: classical auto j1 | 13 | 172 | 33 | 24 | 77 | 0 | 0 |
| ILL/ILLTP-SYJ-cbv | lltp-classical: classical auto j1 | 40 | 13475845 | 11681873 | 8006922 | 1294983 | 0 | 0 |
| ILL/ILLTP-SYJ-cbn | lltp-classical: classical auto j1 | 35 | 1178 | 926 | 578 | 411 | 0 | 0 |
| ILL/ILLTP-SYJ-01 | lltp-classical: classical auto j1 | 11 | 408 | 64 | 98 | 168 | 0 | 0 |
| ILL/ILLTP-LCL-cbv | lltp-classical: classical auto j1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| ILL/ILLTP-LCL-cbn | lltp-classical: classical auto j1 | 2 | 72 | 33 | 24 | 21 | 0 | 0 |
| ILL/ILLTP-LCL-01 | lltp-classical: classical auto j1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| CLL/misc | lltp-classical: classical auto j1 | 8 | 59 | 66 | 7 | 40 | 4 | 4 |
| CLL/Non-theorems | lltp-classical: classical auto j1 | 3 | 104 | 223 | 45 | 59 | 0 | 0 |
| ILL/ILLTP-SYJ-01 | lltp-classical-generous: classical auto j1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| ILL/ILLTP-SYJ-cbn | lltp-classical-generous: classical auto j1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| ILL/ILLTP-SYJ-cbv | lltp-classical-generous: classical auto j1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| ILL/petri-nets/MCC | lltp-classical-generous: classical auto j1 | 25 | 76194 | 0 | 52744 | 23450 | 0 | 0 |
| ILL/ILLTP-LCL-01 | lltp-copies-10: intuitionistic auto j1 | 2 | 271 | 133 | 94 | 42 | 0 | 0 |
| ILL/ILLTP-LCL-cbn | lltp-copies-10: intuitionistic auto j1 | 2 | 72 | 33 | 24 | 21 | 0 | 0 |
| ILL/ILLTP-LCL-cbv | lltp-copies-10: intuitionistic auto j1 | 2 | 239 | 125 | 103 | 38 | 0 | 0 |
| ILL/ILLTP-SYJ-01 | lltp-copies-10: intuitionistic auto j1 | 51 | 6645791 | 2960126 | 4865327 | 821743 | 0 | 0 |
| ILL/ILLTP-SYJ-cbn | lltp-copies-10: intuitionistic auto j1 | 88 | 27029832 | 6788461 | 17925057 | 3107564 | 0 | 0 |
| ILL/ILLTP-SYJ-cbv | lltp-copies-10: intuitionistic auto j1 | 93 | 49461027 | 45346262 | 33361616 | 8726460 | 0 | 0 |
| ILL/ILLTP-SYN-01 | lltp-copies-10: intuitionistic auto j1 | 7 | 3069 | 529 | 1692 | 685 | 0 | 0 |
| ILL/ILLTP-SYN-cbn | lltp-copies-10: intuitionistic auto j1 | 8 | 556 | 246 | 188 | 200 | 0 | 0 |
| ILL/ILLTP-SYN-cbv | lltp-copies-10: intuitionistic auto j1 | 8 | 849 | 779 | 344 | 211 | 0 | 0 |
| ILL/KLE-01 | lltp-copies-10: intuitionistic auto j1 | 31 | 118265 | 49497 | 77940 | 23269 | 0 | 0 |
| ILL/KLE-IMP-CONJ | lltp-copies-10: intuitionistic auto j1 | 60 | 22321 | 11382 | 11709 | 4906 | 0 | 0 |
| ILL/KLE-cbn | lltp-copies-10: intuitionistic auto j1 | 36 | 8348 | 6013 | 4535 | 2029 | 0 | 0 |
| ILL/KLE-cbv | lltp-copies-10: intuitionistic auto j1 | 33 | 18388 | 16704 | 10241 | 4916 | 0 | 0 |
| ILL/Non-theorems | lltp-copies-10: intuitionistic auto j1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| ILL/petri-nets/MCC | lltp-copies-10: intuitionistic auto j1 | 171 | 5129049 | 0 | 3887187 | 1241862 | 0 | 0 |
| ILL/ILLTP-LCL-01 | lltp-default: intuitionistic auto j16 pool after 0.1s | 2 | 271 | 133 | 94 | 42 | 0 | 0 |
| ILL/ILLTP-LCL-cbn | lltp-default: intuitionistic auto j16 pool after 0.1s | 2 | 72 | 33 | 24 | 21 | 0 | 0 |
| ILL/ILLTP-LCL-cbv | lltp-default: intuitionistic auto j16 pool after 0.1s | 2 | 239 | 125 | 103 | 38 | 0 | 0 |
| ILL/ILLTP-SYJ-01 | lltp-default: intuitionistic auto j16 pool after 0.1s | 101 | 219409287 | 109014871 | 183660116 | 6263154 | 0 | 0 |
| ILL/ILLTP-SYJ-cbn | lltp-default: intuitionistic auto j16 pool after 0.1s | 137 | 89887636 | 41863454 | 68944269 | 6722471 | 0 | 0 |
| ILL/ILLTP-SYJ-cbv | lltp-default: intuitionistic auto j16 pool after 0.1s | 117 | 145654956 | 130145183 | 118239269 | 7747118 | 0 | 0 |
| ILL/ILLTP-SYN-01 | lltp-default: intuitionistic auto j16 pool after 0.1s | 19 | 27766 | 9512 | 16890 | 1189 | 0 | 0 |
| ILL/ILLTP-SYN-cbn | lltp-default: intuitionistic auto j16 pool after 0.1s | 19 | 594 | 248 | 189 | 227 | 0 | 0 |
| ILL/ILLTP-SYN-cbv | lltp-default: intuitionistic auto j16 pool after 0.1s | 19 | 934 | 807 | 352 | 257 | 0 | 0 |
| ILL/KLE-01 | lltp-default: intuitionistic auto j16 pool after 0.1s | 87 | 12773396 | 6176805 | 10640192 | 51827 | 0 | 0 |
| ILL/KLE-IMP-CONJ/ALT | lltp-default: intuitionistic auto j16 pool after 0.1s | 27 | 123 | 123 | 0 | 79 | 20 | 20 |
| ILL/KLE-IMP-CONJ | lltp-default: intuitionistic auto j16 pool after 0.1s | 222 | 26067 | 13013 | 12564 | 6499 | 116 | 116 |
| ILL/KLE-IMP-CONJ/NON-THEOREMS | lltp-default: intuitionistic auto j16 pool after 0.1s | 22 | 24 | 32 | 0 | 23 | 0 | 0 |
| ILL/KLE-cbn | lltp-default: intuitionistic auto j16 pool after 0.1s | 88 | 1132942 | 746202 | 958531 | 9304 | 0 | 0 |
| ILL/KLE-cbv | lltp-default: intuitionistic auto j16 pool after 0.1s | 88 | 1415848 | 1449245 | 1197850 | 10429 | 0 | 0 |
| ILL/Non-theorems | lltp-default: intuitionistic auto j16 pool after 0.1s | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| ILL/misc | lltp-default: intuitionistic auto j16 pool after 0.1s | 3 | 46 | 74 | 16 | 22 | 0 | 0 |
| ILL/petri-nets/MCC | lltp-default: intuitionistic auto j16 pool after 0.1s | 3069 | 33140643 | 0 | 23242718 | 9897925 | 0 | 0 |
| ILL/ILLTP-LCL-01 | lltp-forward: intuitionistic auto j1 bias factors | 2 | 271 | 133 | 94 | 42 | 0 | 0 |
| ILL/ILLTP-LCL-cbn | lltp-forward: intuitionistic auto j1 bias factors | 2 | 72 | 33 | 24 | 21 | 0 | 0 |
| ILL/ILLTP-LCL-cbv | lltp-forward: intuitionistic auto j1 bias factors | 2 | 239 | 125 | 103 | 38 | 0 | 0 |
| ILL/ILLTP-SYJ-01 | lltp-forward: intuitionistic auto j1 bias factors | 99 | 107477968 | 25879358 | 83660754 | 5827110 | 0 | 0 |
| ILL/ILLTP-SYJ-cbn | lltp-forward: intuitionistic auto j1 bias factors | 142 | 89735874 | 36682155 | 67645795 | 6015274 | 0 | 0 |
| ILL/ILLTP-SYJ-cbv | lltp-forward: intuitionistic auto j1 bias factors | 118 | 118239512 | 152653783 | 96578385 | 9939741 | 0 | 0 |
| ILL/ILLTP-SYN-01 | lltp-forward: intuitionistic auto j1 bias factors | 19 | 27766 | 9512 | 16890 | 1189 | 0 | 0 |
| ILL/ILLTP-SYN-cbn | lltp-forward: intuitionistic auto j1 bias factors | 19 | 594 | 248 | 189 | 227 | 0 | 0 |
| ILL/ILLTP-SYN-cbv | lltp-forward: intuitionistic auto j1 bias factors | 19 | 934 | 807 | 352 | 257 | 0 | 0 |
| ILL/KLE-01 | lltp-forward: intuitionistic auto j1 bias factors | 86 | 939844 | 420835 | 718184 | 44789 | 0 | 0 |
| ILL/KLE-IMP-CONJ/ALT | lltp-forward: intuitionistic auto j1 bias factors | 27 | 123 | 123 | 0 | 79 | 20 | 20 |
| ILL/KLE-IMP-CONJ | lltp-forward: intuitionistic auto j1 bias factors | 222 | 26067 | 13013 | 12564 | 6499 | 116 | 116 |
| ILL/KLE-IMP-CONJ/NON-THEOREMS | lltp-forward: intuitionistic auto j1 bias factors | 22 | 24 | 32 | 0 | 23 | 0 | 0 |
| ILL/KLE-cbn | lltp-forward: intuitionistic auto j1 bias factors | 87 | 51515 | 39946 | 39335 | 2962 | 0 | 0 |
| ILL/KLE-cbv | lltp-forward: intuitionistic auto j1 bias factors | 86 | 47283 | 37860 | 30392 | 7983 | 0 | 0 |
| ILL/Non-theorems | lltp-forward: intuitionistic auto j1 bias factors | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| ILL/misc | lltp-forward: intuitionistic auto j1 bias factors | 3 | 46 | 74 | 16 | 22 | 0 | 0 |
| ILL/petri-nets/MCC | lltp-forward: intuitionistic auto j1 bias factors | 3071 | 36664493 | 0 | 25351462 | 11313031 | 0 | 0 |
| ILL/ILLTP-LCL-01 | lltp-intuitionistic: intuitionistic auto j1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| ILL/ILLTP-LCL-cbn | lltp-intuitionistic: intuitionistic auto j1 | 2 | 72 | 33 | 24 | 21 | 0 | 0 |
| ILL/ILLTP-LCL-cbv | lltp-intuitionistic: intuitionistic auto j1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| ILL/ILLTP-SYJ-01 | lltp-intuitionistic: intuitionistic auto j1 | 11 | 408 | 64 | 98 | 168 | 0 | 0 |
| ILL/ILLTP-SYJ-cbn | lltp-intuitionistic: intuitionistic auto j1 | 35 | 1178 | 926 | 578 | 411 | 0 | 0 |
| ILL/ILLTP-SYJ-cbv | lltp-intuitionistic: intuitionistic auto j1 | 40 | 13475781 | 11681639 | 8006872 | 1294983 | 0 | 0 |
| ILL/ILLTP-SYN-01 | lltp-intuitionistic: intuitionistic auto j1 | 13 | 172 | 33 | 24 | 77 | 0 | 0 |
| ILL/ILLTP-SYN-cbn | lltp-intuitionistic: intuitionistic auto j1 | 18 | 435 | 104 | 121 | 208 | 0 | 0 |
| ILL/ILLTP-SYN-cbv | lltp-intuitionistic: intuitionistic auto j1 | 18 | 376 | 192 | 88 | 170 | 0 | 0 |
| ILL/KLE-01 | lltp-intuitionistic: intuitionistic auto j1 | 51 | 2172 | 452 | 502 | 944 | 0 | 0 |
| ILL/KLE-IMP-CONJ/ALT | lltp-intuitionistic: intuitionistic auto j1 | 27 | 123 | 123 | 0 | 79 | 20 | 20 |
| ILL/KLE-IMP-CONJ | lltp-intuitionistic: intuitionistic auto j1 | 162 | 3746 | 1631 | 855 | 1593 | 116 | 116 |
| ILL/KLE-IMP-CONJ/NON-THEOREMS | lltp-intuitionistic: intuitionistic auto j1 | 22 | 24 | 32 | 0 | 23 | 0 | 0 |
| ILL/KLE-cbn | lltp-intuitionistic: intuitionistic auto j1 | 69 | 1761 | 554 | 471 | 784 | 0 | 0 |
| ILL/KLE-cbv | lltp-intuitionistic: intuitionistic auto j1 | 68 | 1962 | 852 | 430 | 900 | 0 | 0 |
| ILL/Non-theorems | lltp-intuitionistic: intuitionistic auto j1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| ILL/misc | lltp-intuitionistic: intuitionistic auto j1 | 3 | 46 | 74 | 16 | 22 | 0 | 0 |
| ILL/petri-nets/MCC | lltp-intuitionistic: intuitionistic auto j1 | 3071 | 36664493 | 0 | 25351462 | 11313031 | 0 | 0 |
| ILL/ILLTP-SYJ-01 | lltp-intuitionistic-generous: intuitionistic auto j1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| ILL/ILLTP-SYJ-cbn | lltp-intuitionistic-generous: intuitionistic auto j1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| ILL/ILLTP-SYJ-cbv | lltp-intuitionistic-generous: intuitionistic auto j1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| ILL/petri-nets/MCC | lltp-intuitionistic-generous: intuitionistic auto j1 | 19 | 76039 | 0 | 52719 | 23320 | 0 | 0 |
| ILL/ILLTP-LCL-01 | lltp-rarer: intuitionistic auto j1 bias rarer | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| ILL/ILLTP-LCL-cbn | lltp-rarer: intuitionistic auto j1 bias rarer | 1 | 52 | 17 | 16 | 16 | 0 | 0 |
| ILL/ILLTP-LCL-cbv | lltp-rarer: intuitionistic auto j1 bias rarer | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| ILL/ILLTP-SYJ-01 | lltp-rarer: intuitionistic auto j1 bias rarer | 11 | 408 | 64 | 98 | 168 | 0 | 0 |
| ILL/ILLTP-SYJ-cbn | lltp-rarer: intuitionistic auto j1 bias rarer | 34 | 1259 | 947 | 606 | 468 | 0 | 0 |
| ILL/ILLTP-SYJ-cbv | lltp-rarer: intuitionistic auto j1 bias rarer | 40 | 13475781 | 11681639 | 8006872 | 1294983 | 0 | 0 |
| ILL/ILLTP-SYN-01 | lltp-rarer: intuitionistic auto j1 bias rarer | 13 | 172 | 33 | 24 | 77 | 0 | 0 |
| ILL/ILLTP-SYN-cbn | lltp-rarer: intuitionistic auto j1 bias rarer | 18 | 438 | 104 | 121 | 211 | 0 | 0 |
| ILL/ILLTP-SYN-cbv | lltp-rarer: intuitionistic auto j1 bias rarer | 18 | 376 | 192 | 88 | 170 | 0 | 0 |
| ILL/KLE-01 | lltp-rarer: intuitionistic auto j1 bias rarer | 51 | 2172 | 452 | 502 | 944 | 0 | 0 |
| ILL/KLE-IMP-CONJ/ALT | lltp-rarer: intuitionistic auto j1 bias rarer | 27 | 143 | 169 | 0 | 99 | 20 | 20 |
| ILL/KLE-IMP-CONJ | lltp-rarer: intuitionistic auto j1 bias rarer | 162 | 3993 | 1769 | 929 | 1760 | 116 | 116 |
| ILL/KLE-IMP-CONJ/NON-THEOREMS | lltp-rarer: intuitionistic auto j1 bias rarer | 22 | 28 | 51 | 0 | 27 | 0 | 0 |
| ILL/KLE-cbn | lltp-rarer: intuitionistic auto j1 bias rarer | 69 | 2026 | 625 | 563 | 945 | 0 | 0 |
| ILL/KLE-cbv | lltp-rarer: intuitionistic auto j1 bias rarer | 68 | 1962 | 852 | 430 | 900 | 0 | 0 |
| ILL/Non-theorems | lltp-rarer: intuitionistic auto j1 bias rarer | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| ILL/misc | lltp-rarer: intuitionistic auto j1 bias rarer | 3 | 79 | 81 | 29 | 45 | 0 | 0 |
| ILL/petri-nets/MCC | lltp-rarer: intuitionistic auto j1 bias rarer | 3071 | 36664493 | 0 | 25351462 | 11313031 | 0 | 0 |
| ILL/ILLTP-SYJ-01 | lltp-recursion: intuitionistic auto j1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| ILL/ILLTP-SYJ-cbn | lltp-recursion: intuitionistic auto j1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| ILL/ILLTP-SYJ-cbv | lltp-recursion: intuitionistic auto j1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| ILL/petri-nets/MCC | lltp-recursion: intuitionistic auto j1 | 895 | 1959911 | 0 | 1041264 | 918647 | 0 | 0 |
| ILL/petri-nets/MCC | lltp-recursion-generous: intuitionistic auto j1 | 9 | 3856 | 0 | 1169 | 2687 | 0 | 0 |
| 3-partition-no | long-1: classical auto j1 | 1 | 971 | 3294 | 733 | 238 | 0 | 0 |
| partition-no | long-1: classical auto j1 | 1 | 811 | 4161 | 568 | 243 | 0 | 0 |
| partition-yes | long-1: classical auto j1 | 1 | 178 | 1097 | 97 | 81 | 0 | 0 |
| counter | long-1: classical auto j1 | 2 | 78 | 0 | 0 | 78 | 0 | 0 |
| mix | long-2: mix auto j1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| wide-m3 | long-2: classical auto j1 | 1 | 37 | 970 | 0 | 37 | 0 | 0 |
| wide-m4 | long-2: classical auto j1 | 1 | 37 | 916 | 0 | 37 | 0 | 0 |
| qbf | long-2: classical auto j1 | 1 | 172882 | 511289 | 10646 | 162236 | 0 | 0 |
| 3-partition-yes | parallel: classical auto j1 | 4 | 28 | 140 | 0 | 28 | 0 | 0 |
| 3-partition-yes | parallel: classical auto j2 | 4 | 52 | 296 | 0 | 52 | 0 | 0 |
| 3-partition-yes | parallel: classical auto j4 | 4 | 100 | 546 | 4 | 72 | 0 | 0 |
| 3-partition-yes | parallel: classical auto j8 | 4 | 294 | 1717 | 37 | 177 | 0 | 0 |
| 3-partition-yes | parallel: classical auto j16 | 4 | 320 | 1664 | 48 | 192 | 0 | 0 |
| 3-partition-no | parallel: classical auto j1 | 2 | 1894 | 6088 | 1446 | 448 | 0 | 0 |
| 3-partition-no | parallel: classical auto j2 | 2 | 1927 | 6171 | 1474 | 448 | 0 | 0 |
| 3-partition-no | parallel: classical auto j4 | 2 | 2224 | 7255 | 1679 | 448 | 0 | 0 |
| 3-partition-no | parallel: classical auto j8 | 2 | 2603 | 8367 | 1994 | 448 | 0 | 0 |
| 3-partition-no | parallel: classical auto j16 | 2 | 2641 | 8485 | 2022 | 448 | 0 | 0 |
| partition-yes | parallel: classical auto j1 | 5 | 38546799 | 387076933 | 33786817 | 1098638 | 0 | 0 |
| partition-yes | parallel: classical auto j2 | 5 | 76957011 | 772025135 | 67460947 | 1098642 | 0 | 0 |
| partition-yes | parallel: classical auto j4 | 5 | 152815292 | 1526445396 | 134013029 | 1098706 | 0 | 0 |
| partition-yes | parallel: classical auto j8 | 5 | 280303570 | 2647054534 | 247069373 | 1098972 | 0 | 0 |
| partition-yes | parallel: classical auto j16 | 5 | 455683733 | 4092574756 | 403402299 | 1100811 | 0 | 0 |
| partition-no | parallel: classical auto j1 | 3 | 4252557 | 27627908 | 3720792 | 531765 | 0 | 0 |
| partition-no | parallel: classical auto j2 | 3 | 4635001 | 30226902 | 4047577 | 531765 | 0 | 0 |
| partition-no | parallel: classical auto j4 | 3 | 5142713 | 33348615 | 4487049 | 531765 | 0 | 0 |
| partition-no | parallel: classical auto j8 | 3 | 5686681 | 35961241 | 4981511 | 531765 | 0 | 0 |
| partition-no | parallel: classical auto j16 | 3 | 6057801 | 37878288 | 5309177 | 531765 | 0 | 0 |
| qbf | parallel: classical auto j1 | 19 | 258912034 | 1596527780 | 28181763 | 7813436 | 0 | 0 |
| qbf | parallel: classical auto j2 | 14 | 27118473 | 148800688 | 2702176 | 2634645 | 0 | 0 |
| qbf | parallel: classical auto j4 | 14 | 27117596 | 148799368 | 2702177 | 2633768 | 0 | 0 |
| qbf | parallel: classical auto j8 | 14 | 27114430 | 148794538 | 2702176 | 2630602 | 0 | 0 |
| qbf | parallel: classical auto j16 | 14 | 27112578 | 148791713 | 2702175 | 2628755 | 0 | 0 |
| mix | parallel: mix auto j1 | 3 | 13369293 | 3139384956 | 11993040 | 1376253 | 0 | 0 |
| mix | parallel: mix auto j2 | 3 | 20305205 | 4660712016 | 18225564 | 1372605 | 0 | 0 |
| mix | parallel: mix auto j4 | 3 | 20305205 | 4660712016 | 18225564 | 1372605 | 0 | 0 |
| mix | parallel: mix auto j8 | 3 | 20305205 | 4660712016 | 18225564 | 1372605 | 0 | 0 |
| mix | parallel: mix auto j16 | 2 | 2883552 | 345553416 | 2555874 | 327678 | 0 | 0 |
| counter | parallel: classical auto j1 | 6 | 120 | 0 | 0 | 120 | 0 | 0 |
| counter | parallel: classical auto j2 | 6 | 120 | 0 | 0 | 120 | 0 | 0 |
| counter | parallel: classical auto j4 | 6 | 120 | 0 | 0 | 120 | 0 | 0 |
| counter | parallel: classical auto j8 | 6 | 120 | 0 | 0 | 120 | 0 | 0 |
| counter | parallel: classical auto j16 | 6 | 120 | 0 | 0 | 120 | 0 | 0 |
| counter-over | parallel: classical auto j1 | 6 | 171 | 0 | 23 | 148 | 0 | 0 |
| counter-over | parallel: classical auto j2 | 6 | 171 | 0 | 23 | 148 | 0 | 0 |
| counter-over | parallel: classical auto j4 | 6 | 171 | 0 | 23 | 148 | 0 | 0 |
| counter-over | parallel: classical auto j8 | 6 | 171 | 0 | 23 | 148 | 0 | 0 |
| counter-over | parallel: classical auto j16 | 6 | 171 | 0 | 23 | 148 | 0 | 0 |
| wide-m3 | parallel: classical auto j1 | 3 | 93 | 2112 | 0 | 93 | 0 | 0 |
| wide-m3 | parallel: classical auto j2 | 3 | 93 | 2109 | 0 | 93 | 0 | 0 |
| wide-m3 | parallel: classical auto j4 | 3 | 93 | 2109 | 0 | 93 | 0 | 0 |
| wide-m3 | parallel: classical auto j8 | 3 | 93 | 2115 | 0 | 93 | 0 | 0 |
| wide-m3 | parallel: classical auto j16 | 3 | 93 | 2124 | 0 | 93 | 0 | 0 |
| wide-m4 | parallel: classical auto j1 | 3 | 99 | 2222 | 0 | 99 | 0 | 0 |
| wide-m4 | parallel: classical auto j2 | 3 | 99 | 2219 | 0 | 99 | 0 | 0 |
| wide-m4 | parallel: classical auto j4 | 3 | 99 | 2219 | 0 | 99 | 0 | 0 |
| wide-m4 | parallel: classical auto j8 | 3 | 99 | 2219 | 0 | 99 | 0 | 0 |
| wide-m4 | parallel: classical auto j16 | 3 | 99 | 2228 | 0 | 99 | 0 | 0 |
| 3-partition-mll-no | parallel-net: classical net j1 | 2 | 16931690 | 0 | 0 | 0 | 40487784 | 40487784 |
| 3-partition-mll-no | parallel-net: classical net j2 | 2 | 16931690 | 0 | 0 | 0 | 40488636 | 40487784 |
| 3-partition-mll-no | parallel-net: classical net j4 | 2 | 16931690 | 0 | 0 | 0 | 40488636 | 40487784 |
| 3-partition-mll-no | parallel-net: classical net j8 | 2 | 16931690 | 0 | 0 | 0 | 40491516 | 40487784 |
| 3-partition-mll-no | parallel-net: classical net j16 | 2 | 16931690 | 0 | 0 | 0 | 40491516 | 40487784 |
| partition-table | parallel-net: classical net j1 | 9 | 1321234 | 0 | 0 | 0 | 6276936 | 6276936 |
| partition-table | parallel-net: classical net j2 | 9 | 1314124 | 0 | 0 | 0 | 6243133 | 6238227 |
| partition-table | parallel-net: classical net j4 | 10 | 1296756 | 0 | 0 | 0 | 6153389 | 6144358 |
| partition-table | parallel-net: classical net j8 | 10 | 1368092 | 0 | 0 | 0 | 7102255 | 7022571 |
| partition-table | parallel-net: classical net j16 | 10 | 1434302 | 0 | 0 | 0 | 8244010 | 8066119 |
| wide-m1 | period-16: classical net j1 period 16 | 2 | 4608 | 0 | 0 | 0 | 4608 | 288 |
| wide-m2 | period-16: classical net j1 period 16 | 2 | 4608 | 0 | 0 | 0 | 4608 | 288 |
| 3-partition-mll-no | period-16: classical net j1 period 16 | 1 | 49883557 | 0 | 0 | 0 | 78913956 | 29030400 |
| partition-yes | period-16: classical net j1 period 16 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| partition-no | period-16: classical net j1 period 16 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| wide-m1 | period-1: classical net j1 period 1 | 2 | 4608 | 0 | 0 | 0 | 4608 | 4608 |
| wide-m2 | period-1: classical net j1 period 1 | 2 | 4608 | 0 | 0 | 0 | 4608 | 4608 |
| 3-partition-mll-no | period-1: classical net j1 period 1 | 2 | 16931690 | 0 | 0 | 0 | 40487784 | 40487784 |
| partition-yes | period-1: classical net j1 period 1 | 1 | 235737 | 0 | 0 | 0 | 1071471 | 1071471 |
| partition-no | period-1: classical net j1 period 1 | 1 | 269950 | 0 | 0 | 0 | 1313537 | 1313537 |
| wide-m1 | period-2: classical net j1 period 2 | 2 | 4608 | 0 | 0 | 0 | 4608 | 2304 |
| wide-m2 | period-2: classical net j1 period 2 | 2 | 4608 | 0 | 0 | 0 | 4608 | 2304 |
| 3-partition-mll-no | period-2: classical net j1 period 2 | 2 | 28930058 | 0 | 0 | 0 | 61561032 | 40785324 |
| partition-yes | period-2: classical net j1 period 2 | 1 | 506877 | 0 | 0 | 0 | 2524723 | 2138489 |
| partition-no | period-2: classical net j1 period 2 | 1 | 633574 | 0 | 0 | 0 | 3148361 | 2648382 |
| wide-m1 | period-8: classical net j1 period 8 | 2 | 4608 | 0 | 0 | 0 | 4608 | 576 |
| wide-m2 | period-8: classical net j1 period 8 | 2 | 4608 | 0 | 0 | 0 | 4608 | 576 |
| 3-partition-mll-no | period-8: classical net j1 period 8 | 1 | 11586037 | 0 | 0 | 0 | 18356196 | 6779520 |
| partition-yes | period-8: classical net j1 period 8 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| partition-no | period-8: classical net j1 period 8 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |

## partition-yes

| problem | engines: classical focus j1 | engines: classical net j1 | families: classical auto j1 | long-1: classical auto j1 | parallel: classical auto j1 | parallel: classical auto j2 | parallel: classical auto j4 | parallel: classical auto j8 | parallel: classical auto j16 | period-16: classical net j1 period 16 | period-1: classical net j1 period 1 | period-2: classical net j1 period 2 | period-8: classical net j1 period 8 |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| partition-yes/4 | 104 µs ✓ | 2.83 s ✓ | 81 µs ✓ |  |  |  |  |  |  | > 60 s | 2.69 s ✓ | 5.72 s ✓ | > 60 s |
| partition-yes/5 | 99 µs ✓ | > 60 s | 93 µs ✓ |  | 117 µs ✓ | 311 µs ✓ | 334 µs ✓ | 461 µs ✓ | 851 µs ✓ |  |  |  |  |
| partition-yes/6 | 140 µs ✓ | > 60 s | 136 µs ✓ |  | 187 µs ✓ | 461 µs ✓ | 413 µs ✓ | 546 µs ✓ | 906 µs ✓ |  |  |  |  |
| partition-yes/12 | 13.3 ms ✓ | > 60 s | 13.8 ms ✓ |  |  |  |  |  |  |  |  |  |  |
| partition-yes/20 | 471.1 ms ✓ | > 60 s | 454.7 ms ✓ |  | 396.5 ms ✓ | 418.5 ms ✓ | 429.9 ms ✓ | 439.9 ms ✓ | 445.7 ms ✓ |  |  |  |  |
| partition-yes/7 |  |  | 211 µs ✓ | 214 µs ✓ | 273 µs ✓ | 585 µs ✓ | 621 µs ✓ | 597 µs ✓ | 750 µs ✓ |  |  |  |  |
| partition-yes/16 |  |  | 147.8 ms ✓ |  |  |  |  |  |  |  |  |  |  |
| partition-yes/24 |  |  | 74.41 s ✓ |  | 67.45 s ✓ | 70.81 s ✓ | 74.75 s ✓ | 79.07 s ✓ | 87.42 s ✓ |  |  |  |  |
| partition-yes/28 |  |  | > 300 s | > 1200 s |  |  |  |  |  |  |  |  |  |

## partition-no

| problem | engines: classical focus j1 | engines: classical net j1 | families: classical auto j1 | long-1: classical auto j1 | parallel: classical auto j1 | parallel: classical auto j2 | parallel: classical auto j4 | parallel: classical auto j8 | parallel: classical auto j16 | period-16: classical net j1 period 16 | period-1: classical net j1 period 1 | period-2: classical net j1 period 2 | period-8: classical net j1 period 8 |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| partition-no/3 | 71 µs ✗ | 3.68 s ✗ | 79 µs ✗ |  |  |  |  |  |  | > 60 s | 3.32 s ✗ | 7.45 s ✗ | > 60 s |
| partition-no/4 | 150 µs ✗ | > 60 s | 144 µs ✗ |  | 192 µs ✗ | 405 µs ✗ | 347 µs ✗ | 418 µs ✗ | 600 µs ✗ | > 60 s | > 60 s | > 60 s | > 60 s |
| partition-no/9 | 89.0 ms ✗ | > 60 s | 85.5 ms ✗ |  |  |  |  |  |  |  |  |  |  |
| partition-no/12 | 3.81 s ✗ | > 60 s | 3.82 s ✗ |  | 3.59 s ✗ | 2.13 s ✗ | 1.24 s ✗ | 806.0 ms ✗ | 581.6 ms ✗ |  |  |  |  |
| partition-no/5 |  |  | 435 µs ✗ | 441 µs ✗ | 568 µs ✗ | 745 µs ✗ | 578 µs ✗ | 614 µs ✗ | 850 µs ✗ |  |  |  |  |
| partition-no/14 |  |  | > 300 s |  | > 120 s | > 120 s | > 120 s | > 120 s | > 120 s |  |  |  |  |
| partition-no/15 |  |  | > 300 s | > 1200 s |  |  |  |  |  |  |  |  |  |

## 3-partition-mll-yes

| problem | engines: classical focus j1 | engines: classical net j1 | families: classical auto j1 |
|---|--:|--:|--:|
| 3-partition-mll-yes/4 | 64 µs ✓ | 62 µs ✓ | 54 µs ✓ |
| 3-partition-mll-yes/6 | 55 µs ✓ | 88 µs ✓ | 53 µs ✓ |
| 3-partition-mll-yes/8 | 55 µs ✓ | 107 µs ✓ | 56 µs ✓ |
| 3-partition-mll-yes/12 | 57 µs ✓ | 170 µs ✓ | 55 µs ✓ |

## 3-partition-mll-no

| problem | engines: classical focus j1 | engines: classical net j1 | families: classical auto j1 | parallel-net: classical net j1 | parallel-net: classical net j2 | parallel-net: classical net j4 | parallel-net: classical net j8 | parallel-net: classical net j16 | period-16: classical net j1 period 16 | period-1: classical net j1 period 1 | period-2: classical net j1 period 2 | period-8: classical net j1 period 8 |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| 3-partition-mll-no/4 | 39 µs ✗ | 1.83 s ✗ | 39 µs ✗ | 1.63 s ✗ | 829.1 ms ✗ | 443.0 ms ✗ | 246.5 ms ✗ | 138.2 ms ✗ | 20.95 s ✗ | 1.73 s ✗ | 1.38 s ✗ | 4.78 s ✗ |
| 3-partition-mll-no/5 | 41 µs ✗ | 56.13 s ✗ | 41 µs ✗ | 48.79 s ✗ | 25.09 s ✗ | 13.41 s ✗ | 7.58 s ✗ | 4.42 s ✗ | > 60 s | 55.13 s ✗ | 49.10 s ✗ | > 60 s |
| 3-partition-mll-no/6 | 41 µs ✗ | > 60 s | 45 µs ✗ | > 60 s | > 60 s | > 60 s | > 60 s | > 60 s |  |  |  |  |
| 3-partition-mll-no/8 | 46 µs ✗ | > 60 s | 40 µs ✗ |  |  |  |  |  |  |  |  |  |

## wide-m1

| problem | engines: classical focus j1 | engines: classical net j1 | families: classical auto j1 | period-16: classical net j1 period 16 | period-1: classical net j1 period 1 | period-2: classical net j1 period 2 | period-8: classical net j1 period 8 |
|---|--:|--:|--:|--:|--:|--:|--:|
| wide-m1/8 | 52 µs ✓ | 62 µs ✓ | 54 µs ✓ |  |  |  |  |
| wide-m1/16 | 77 µs ✓ | 120 µs ✓ | 110 µs ✓ |  |  |  |  |
| wide-m1/24 | 101 µs ✓ | 199 µs ✓ | 192 µs ✓ |  |  |  |  |
| wide-m1/32 | 148 µs ✓ | 182 µs ✓ | 174 µs ✓ |  |  |  |  |
| wide-m1/256 | 4.0 ms ✓ | 9.0 ms ✓ | 7.7 ms ✓ | 5.9 ms ✓ | 17.9 ms ✓ | 12.2 ms ✓ | 7.0 ms ✓ |
| wide-m1/2048 | 206.6 ms ✓ | 586.4 ms ✓ | 523.0 ms ✓ | 388.2 ms ✓ | 1.22 s ✓ | 818.1 ms ✓ | 448.2 ms ✓ |

## wide-m2

| problem | engines: classical focus j1 | engines: classical net j1 | families: classical auto j1 | period-16: classical net j1 period 16 | period-1: classical net j1 period 1 | period-2: classical net j1 period 2 | period-8: classical net j1 period 8 |
|---|--:|--:|--:|--:|--:|--:|--:|
| wide-m2/8 | 55 µs ✓ | 55 µs ✓ | 54 µs ✓ |  |  |  |  |
| wide-m2/16 | 69 µs ✓ | 113 µs ✓ | 107 µs ✓ |  |  |  |  |
| wide-m2/24 | 105 µs ✓ | 205 µs ✓ | 194 µs ✓ |  |  |  |  |
| wide-m2/32 | 128 µs ✓ | 178 µs ✓ | 171 µs ✓ |  |  |  |  |
| wide-m2/256 | 3.2 ms ✓ | 9.0 ms ✓ | 8.0 ms ✓ | 6.2 ms ✓ | 18.5 ms ✓ | 12.0 ms ✓ | 7.0 ms ✓ |
| wide-m2/2048 | 209.9 ms ✓ | 610.6 ms ✓ | 555.5 ms ✓ | 405.9 ms ✓ | 1.23 s ✓ | 803.6 ms ✓ | 463.3 ms ✓ |

## wide-m3

| problem | engines: classical focus j1 | engines: classical net j1 | families: classical auto j1 | long-2: classical auto j1 | parallel: classical auto j1 | parallel: classical auto j2 | parallel: classical auto j4 | parallel: classical auto j8 | parallel: classical auto j16 |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| wide-m3/12 | 69 µs ✓ | 85 µs ✓ | 59 µs ✓ |  |  |  |  |  |  |
| wide-m3/24 | 96 µs ✓ | 211 µs ✓ | 87 µs ✓ |  | 135 µs ✓ | 357 µs ✓ | 315 µs ✓ | 489 µs ✓ | 714 µs ✓ |
| wide-m3/30 | 123 µs ✓ | 176 µs ✓ | 104 µs ✓ |  | 182 µs ✓ | 316 µs ✓ | 390 µs ✓ | 503 µs ✓ | 807 µs ✓ |
| wide-m3/256 | 3.0 ms ✓ | 9.4 ms ✓ | 2.7 ms ✓ |  |  |  |  |  |  |
| wide-m3/2048 | 192.5 ms ✓ | 628.1 ms ✓ | 172.9 ms ✓ |  |  |  |  |  |  |
| wide-m3/36 |  |  | 128 µs ✓ | 154 µs ✓ | 192 µs ✓ | 415 µs ✓ | 405 µs ✓ | 453 µs ✓ | 727 µs ✓ |
| wide-m3/1024 |  |  | 42.8 ms ✓ |  |  |  |  |  |  |

## wide-m4

| problem | engines: classical focus j1 | engines: classical net j1 | families: classical auto j1 | long-2: classical auto j1 | parallel: classical auto j1 | parallel: classical auto j2 | parallel: classical auto j4 | parallel: classical auto j8 | parallel: classical auto j16 |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| wide-m4/12 | 59 µs ✓ | 81 µs ✓ | 58 µs ✓ |  |  |  |  |  |  |
| wide-m4/24 | 93 µs ✓ | 218 µs ✓ | 92 µs ✓ |  |  |  |  |  |  |
| wide-m4/28 | 105 µs ✓ | 258 µs ✓ | 97 µs ✓ |  | 141 µs ✓ | 326 µs ✓ | 342 µs ✓ | 487 µs ✓ | 727 µs ✓ |
| wide-m4/256 | 2.8 ms ✓ | 9.8 ms ✓ | 2.5 ms ✓ |  |  |  |  |  |  |
| wide-m4/2048 | 182.0 ms ✓ | 651.3 ms ✓ | 167.4 ms ✓ |  |  |  |  |  |  |
| wide-m4/32 |  |  | 108 µs ✓ |  | 186 µs ✓ | 369 µs ✓ | 368 µs ✓ | 538 µs ✓ | 807 µs ✓ |
| wide-m4/36 |  |  | 134 µs ✓ | 135 µs ✓ | 190 µs ✓ | 430 µs ✓ | 435 µs ✓ | 535 µs ✓ | 722 µs ✓ |
| wide-m4/1024 |  |  | 40.2 ms ✓ |  |  |  |  |  |  |

## additive

| problem | engines: classical focus j1 | families: classical auto j1 |
|---|--:|--:|
| additive/8 | 873 µs ✓ | 329 µs ✓ |
| additive/12 | 38.1 ms ✓ | 10.8 ms ✓ |
| additive/14 | 391.5 ms ✓ | 58.4 ms ✓ |
| additive/16 | 5.79 s ✓ | 181.6 ms ✓ |

## partition-table

| problem | engines: classical focus j1 | engines: classical net j1 | parallel-net: classical net j1 | parallel-net: classical net j2 | parallel-net: classical net j4 | parallel-net: classical net j8 | parallel-net: classical net j16 |
|---|--:|--:|--:|--:|--:|--:|--:|
| partition-table/1-1 | 51 µs ✓ | 51 µs ✓ | 72 µs ✓ | 180 µs ✓ | 199 µs ✓ | 346 µs ✓ | 508 µs ✓ |
| partition-table/1-3 | 50 µs ✗ | 388 µs ✗ | 488 µs ✗ | 584 µs ✗ | 674 µs ✗ | 724 µs ✗ | 965 µs ✗ |
| partition-table/2-1-1 | 67 µs ✓ | 626 µs ✓ | 802 µs ✓ | 726 µs ✓ | 1.2 ms ✓ | 1.3 ms ✓ | 1.4 ms ✓ |
| partition-table/1-1-4 | 81 µs ✗ | 40.2 ms ✗ | 46.5 ms ✗ | 24.7 ms ✗ | 12.9 ms ✗ | 7.6 ms ✗ | 4.7 ms ✗ |
| partition-table/2-2-1-1 | 82 µs ✓ | 52.2 ms ✓ † | 62.3 ms ✓ | 30.6 ms ✓ | 16.6 ms ✓ | 8.7 ms ✓ | 6.0 ms ✓ |
| partition-table/1-2-5 | 83 µs ✗ | 3.68 s ✗ | 3.66 s ✗ | 1.82 s ✗ | 919.5 ms ✗ | 537.3 ms ✗ | 291.2 ms ✗ |
| partition-table/1-1-2-4 | 70 µs ✓ | 1.04 s ✓ | 1.01 s ✓ | 496.8 ms ✓ | 251.8 ms ✓ | 141.7 ms ✓ | 74.5 ms ✓ |
| partition-table/1-1-1-5 | 141 µs ✗ | 7.43 s ✗ | 7.13 s ✗ | 3.49 s ✗ | 1.90 s ✗ | 1.08 s ✗ | 649.9 ms ✗ |
| partition-table/2-3-2-1 | 75 µs ✓ | 4.68 s ✓ | 4.24 s ✓ | 2.18 s ✓ | 1.06 s ✓ | 626.7 ms ✓ | 336.8 ms ✓ |
| partition-table/3-3-3-1 | 137 µs ✗ | > 60 s | > 60 s | > 60 s | > 60 s | > 60 s | > 60 s |
| partition-table/1-2-3-4-5-5 | 97 µs ✓ | > 60 s | > 60 s | > 60 s | 2.0 ms ✓ | > 60 s | > 60 s |
| partition-table/1-1-1-1-1-7 | 1.2 ms ✗ | > 60 s | > 60 s | > 60 s | > 60 s | > 60 s | > 60 s |
| partition-table/2-2-2-2-2-2-9-1 | 129 µs ✓ | > 60 s | > 60 s | > 60 s | > 60 s | 815.1 ms ✓ | 977.2 ms ✓ |

## cancellation

| problem | engines: classical focus j1 |
|---|--:|
| cancellation/3-partition-4 | 300 µs ✓ |

## 3-partition-yes

| problem | families: classical auto j1 | intuitionistic: intuitionistic auto j1 | parallel: classical auto j1 | parallel: classical auto j2 | parallel: classical auto j4 | parallel: classical auto j8 | parallel: classical auto j16 |
|---|--:|--:|--:|--:|--:|--:|--:|
| 3-partition-yes/4 | 68 µs ✓ | 60 µs ✓ | 103 µs ✓ | 215 µs ✓ | 247 µs ✓ | 416 µs ✓ | 591 µs ✓ |
| 3-partition-yes/6 | 61 µs ✓ | 59 µs ✓ | 86 µs ✓ | 220 µs ✓ | 269 µs ✓ | 337 µs ✓ | 593 µs ✓ |
| 3-partition-yes/8 | 63 µs ✓ | 58 µs ✓ | 86 µs ✓ | 263 µs ✓ | 278 µs ✓ | 423 µs ✓ | 649 µs ✓ |
| 3-partition-yes/12 | 64 µs ✓ | 59 µs ✓ | 88 µs ✓ | 248 µs ✓ | 301 µs ✓ | 458 µs ✓ | 546 µs ✓ |

## 3-partition-no

| problem | families: classical auto j1 | intuitionistic: intuitionistic auto j1 | long-1: classical auto j1 | parallel: classical auto j1 | parallel: classical auto j2 | parallel: classical auto j4 | parallel: classical auto j8 | parallel: classical auto j16 |
|---|--:|--:|--:|--:|--:|--:|--:|--:|
| 3-partition-no/4 | 319 µs ✗ | 273 µs ✗ |  | 379 µs ✗ | 544 µs ✗ | 428 µs ✗ | 550 µs ✗ | 631 µs ✗ |
| 3-partition-no/5 | 346 µs ✗ |  | 400 µs ✗ | 432 µs ✗ | 567 µs ✗ | 461 µs ✗ | 562 µs ✗ | 674 µs ✗ |

## qbf

| problem | families: classical auto j1 | long-2: classical auto j1 | parallel: classical auto j1 | parallel: classical auto j2 | parallel: classical auto j4 | parallel: classical auto j8 | parallel: classical auto j16 |
|---|--:|--:|--:|--:|--:|--:|--:|
| qbf/8#0 | 131 µs ✗ |  |  |  |  |  |  |
| qbf/8#1 | 144 µs ✗ |  |  |  |  |  |  |
| qbf/8#2 | 156 µs ✓ |  |  |  |  |  |  |
| qbf/8#3 | 130 µs ✗ |  |  |  |  |  |  |
| qbf/12#0 | 1.2 ms ✓ |  |  |  |  |  |  |
| qbf/12#1 | 388 µs ✗ |  |  |  |  |  |  |
| qbf/12#2 | 625 µs ✓ |  |  |  |  |  |  |
| qbf/12#3 | 297 µs ✗ |  |  |  |  |  |  |
| qbf/16#0 | 6.0 ms ✓ |  | 7.1 ms ✓ | 10.4 ms ✓ | 9.6 ms ✓ | 10.3 ms ✓ | 11.3 ms ✓ |
| qbf/16#1 | 1.1 ms ✗ |  | 1.8 ms ✗ | 2.5 ms ✗ | 1.6 ms ✗ | 3.0 ms ✗ | 2.0 ms ✗ |
| qbf/16#2 | 1.7 ms ✗ |  | 2.6 ms ✗ | 2.7 ms ✗ | 2.9 ms ✗ | 2.7 ms ✗ | 3.5 ms ✗ |
| qbf/16#3 | 5.4 ms ✓ |  | 8.4 ms ✓ | 7.3 ms ✓ | 8.5 ms ✓ | 11.2 ms ✓ | 9.5 ms ✓ |
| qbf/20#0 | 5.3 ms ✗ |  | 8.4 ms ✗ | 7.4 ms ✗ | 7.5 ms ✗ | 5.8 ms ✗ | 6.7 ms ✗ |
| qbf/20#1 | 8.2 ms ✗ |  | 14.3 ms ✗ | 11.0 ms ✗ | 10.0 ms ✗ | 11.2 ms ✗ | 13.7 ms ✗ |
| qbf/20#2 | 30.0 ms ✓ |  | 53.3 ms ✓ | 64.0 ms ✓ | 68.3 ms ✓ | 71.9 ms ✓ | 80.4 ms ✓ |
| qbf/20#3 | 5.1 ms ✗ |  | 8.1 ms ✗ | 7.8 ms ✗ | 6.5 ms ✗ | 8.6 ms ✗ | 5.2 ms ✗ |
| qbf/24#0 | 101.8 ms ✗ | 99.6 ms ✗ | 142.0 ms ✗ † | 126.2 ms ✗ | 128.5 ms ✗ | 135.1 ms ✗ | 166.2 ms ✗ |
| qbf/24#1 | 37.0 ms ✗ |  | 68.0 ms ✗ | 37.2 ms ✗ | 38.4 ms ✗ | 43.4 ms ✗ | 45.5 ms ✗ |
| qbf/24#2 | 25.7 ms ✗ |  | 47.6 ms ✗ | 30.8 ms ✗ | 29.7 ms ✗ | 28.1 ms ✗ | 36.5 ms ✗ |
| qbf/24#3 | 36.1 ms ✗ |  | 57.9 ms ✗ | 39.2 ms ✗ | 37.5 ms ✗ | 38.6 ms ✗ | 43.1 ms ✗ |
| qbf/32#0 | 552.4 ms ✗ |  |  |  |  |  |  |
| qbf/32#1 | 626.7 ms ✗ |  |  |  |  |  |  |
| qbf/32#2 | 1.14 s ✗ |  |  |  |  |  |  |
| qbf/32#3 | 476.7 ms ✗ |  |  |  |  |  |  |
| qbf/40#0 | 12.11 s ✗ |  | 13.75 s ✗ | 8.61 s ✗ | 8.46 s ✗ | 7.84 s ✗ | 7.47 s ✗ |
| qbf/40#1 | 13.65 s ✗ |  | 13.41 s ✗ | 5.56 s ? memory | 5.53 s ? memory | 5.52 s ? memory | 5.53 s ? memory |
| qbf/40#2 | 16.21 s ✗ |  | 15.46 s ✗ | 5.80 s ? memory | 5.84 s ? memory | 5.82 s ? memory | 5.81 s ? memory |
| qbf/40#3 | 12.54 s ✗ |  | 11.55 s ✗ | 7.46 s ✗ | 7.57 s ✗ | 7.44 s ✗ | 7.51 s ✗ |
| qbf/44#0 | 54.12 s ✗ |  | 50.45 s ✗ | 8.79 s ? memory | 8.71 s ? memory | 8.78 s ? memory | 8.78 s ? memory |
| qbf/44#1 | 53.43 s ✗ |  | 49.78 s ✗ | 7.31 s ? memory | 7.44 s ? memory | 7.34 s ? memory | 7.42 s ? memory |
| qbf/44#2 | 80.52 s ✗ |  | 73.42 s ✗ | 5.14 s ? memory | 5.13 s ? memory | 5.14 s ? memory | 5.14 s ? memory |
| qbf/44#3 | 28.71 s ? memory |  | 26.33 s ? memory | 8.56 s ? memory | 8.59 s ? memory | 8.70 s ? memory | 8.73 s ? memory |
| qbf/48#0 | 84.57 s ? memory | 83.81 s ? memory |  |  |  |  |  |
| qbf/48#1 | 207.33 s ✗ |  |  |  |  |  |  |
| qbf/48#2 | 161.73 s ✗ |  |  |  |  |  |  |
| qbf/48#3 | 222.46 s ✗ |  |  |  |  |  |  |

## mix

| problem | families: mix auto j1 | long-2: mix auto j1 | parallel: mix auto j1 | parallel: mix auto j2 | parallel: mix auto j4 | parallel: mix auto j8 | parallel: mix auto j16 |
|---|--:|--:|--:|--:|--:|--:|--:|
| mix/4 | 365 µs ✗ |  |  |  |  |  |  |
| mix/6 | 14.0 ms ✗ |  |  |  |  |  |  |
| mix/8 | 858.5 ms ✗ |  | 857.8 ms ✗ | 965.2 ms ✗ | 1.05 s ✗ | 1.48 s ✗ | 1.61 s ✗ |
| mix/9 | 7.31 s ✗ |  | 7.39 s ✗ | 7.05 s ✗ | 7.90 s ✗ | 9.50 s ✗ | 11.51 s ✗ |
| mix/10 | 62.55 s ✗ |  | 62.10 s ✗ | 91.83 s ✗ | 95.53 s ✗ | 109.66 s ✗ | > 120 s |
| mix/11 | > 300 s | > 1200 s | > 120 s | > 120 s | > 120 s | > 120 s | > 120 s |

## counter

| problem | families: classical auto j1 | intuitionistic: intuitionistic auto j1 | long-1: classical auto j1 | parallel: classical auto j1 | parallel: classical auto j2 | parallel: classical auto j4 | parallel: classical auto j8 | parallel: classical auto j16 |
|---|--:|--:|--:|--:|--:|--:|--:|--:|
| counter/2 | 27 µs ✓ | 27 µs ✓ |  | 38 µs ✓ | 101 µs ✓ | 82 µs ✓ | 63 µs ✓ | 80 µs ✓ |
| counter/4 | 28 µs ✓ | 28 µs ✓ |  | 40 µs ✓ | 73 µs ✓ | 83 µs ✓ | 75 µs ✓ | 71 µs ✓ |
| counter/8 | 34 µs ✓ | 32 µs ✓ |  | 51 µs ✓ | 79 µs ✓ | 78 µs ✓ | 83 µs ✓ | 81 µs ✓ |
| counter/16 | 37 µs ✓ | 35 µs ✓ | 48 µs ✓ | 63 µs ✓ | 83 µs ✓ | 93 µs ✓ | 93 µs ✓ | 91 µs ✓ |
| counter/32 | 43 µs ✓ | 42 µs ✓ |  | 72 µs ✓ | 94 µs ✓ | 96 µs ✓ | 100 µs ✓ | 102 µs ✓ |
| counter/64 | 55 µs ✓ | 53 µs ✓ | 60 µs ✓ | 86 µs ✓ | 106 µs ✓ | 102 µs ✓ | 114 µs ✓ | 116 µs ✓ |

## counter-over

| problem | families: classical auto j1 | intuitionistic: intuitionistic auto j1 | parallel: classical auto j1 | parallel: classical auto j2 | parallel: classical auto j4 | parallel: classical auto j8 | parallel: classical auto j16 |
|---|--:|--:|--:|--:|--:|--:|--:|
| counter-over/2 | 24 µs ✗ | 22 µs ✗ | 49 µs ✗ | 72 µs ✗ | 72 µs ✗ | 78 µs ✗ | 68 µs ✗ |
| counter-over/4 | 26 µs ✗ | 26 µs ✗ | 40 µs ✗ | 77 µs ✗ | 68 µs ✗ | 68 µs ✗ | 76 µs ✗ |
| counter-over/8 | 33 µs ✗ | 32 µs ✗ | 55 µs ✗ | 76 µs ✗ | 73 µs ✗ | 87 µs ✗ | 73 µs ✗ |
| counter-over/16 | 35 µs ✗ | 35 µs ✗ | 63 µs ✗ | 84 µs ✗ | 77 µs ✗ | 80 µs ✗ | 86 µs ✗ |
| counter-over/32 | 44 µs ✗ | 44 µs ✗ | 73 µs ✗ | 96 µs ✗ | 94 µs ✗ | 94 µs ✗ | 95 µs ✗ |
| counter-over/64 | 45 µs ✗ | 54 µs ✗ | 70 µs ✗ | 98 µs ✗ | 88 µs ✗ | 91 µs ✗ | 98 µs ✗ |

## growing

| problem | families: classical auto j1 |
|---|--:|
| growing/16 | 213 µs ? bound |
| growing/64 | 800 µs ? bound |
| growing/256 | 18.4 ms ? bound |
| growing/1024 | 114.7 ms ? depth |

## chain

| problem | families: classical auto j1 | intuitionistic: intuitionistic auto j1 |
|---|--:|--:|
| chain/16 | 154 µs ✓ | 149 µs ✓ |
| chain/64 | 4.4 ms ✓ | 4.6 ms ✓ |
| chain/128 | 33.7 ms ✓ | 35.6 ms ✓ |
| chain/256 | 274.7 ms ✓ | 287.5 ms ✓ |
