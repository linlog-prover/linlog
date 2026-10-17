# The three baselines compared

Three baselines, each taken by `bench/baseline.sh` on the same machine
under the same settings (platform profile `performance`, governor
`powersave`, energy preference `balance_performance`, turbo on):

- `bench/results/2026-09-30/`, the engines before the focused engine's
  performance pass;
- `bench/results/2026-10-02/`, after it;
- `bench/results/2026-10-17/`, the engines of the first release
  (commit `ed74387d9bf3`), with the pass under the command's default
  that is new in it (`lltp-default`).

The rustc of the second and third is the same (the flake's lock pins the
same toolchain); the dependencies moved with `Cargo.lock`. The third
started detached at 19:32 and took 2 h 58 min, where the second took
8 h 4 min; no scheduled job ran in either. The tables after "At a glance"
are printed by

```sh
linlog-bench summary --before bench/results/2026-09-30 --before bench/results/2026-10-02 \
  bench/results/2026-10-17/*.csv
linlog-bench summary --against bench/results/2026-10-17/lltp-intuitionistic.csv \
  bench/results/2026-10-17/lltp-rarer.csv bench/results/2026-10-17/lltp-forward.csv
```

the first setting every file of the three side by side and then
comparing every file with the file of the same name in the second
baseline, the second the two passes under one atom bias with the
default pass, problem by problem. A verdict found after the time limit
counts as late and is kept apart.

## At a glance

**No decided verdict differs** between the second and third baselines
on the same problem in the same configuration. **21 problems that the
second baseline decided the third does not, all of them QBF instances
that now end at the memory bound**: `qbf/44#3` on one thread (refuted in
219 s before, the bound after 29 s) and four instances of `qbf/40` and
`qbf/44` on 2, 4, 8 and 16 threads (refuted in 15 to 98 s, the bound
after 5 to 9 s). The second baseline ran without a memory bound; since
then every run has the library's default of 1 GiB, which the
configuration's label leaves out since it is the default. Every proof of
the third baseline was checked (`ok`).

Families, one thread, 300 s (stage 1 in four streams; the long runs
1 200 s):

| family | second baseline | third baseline |
|---|---|---|
| 3-partition-no (bins of b) | every size in under 1 ms | the same (b = 5: 358 → 346 µs) |
| partition-yes (n items) | n = 24: 118 s, 28 > 1 200 s | n = 24: 74.4 s, 28 > 1 200 s |
| partition-no | n = 12: 3.87 s, 14 and 15 > 300 s | n = 12: 3.8 s, 14 and 15 > 300 s |
| qbf (n variables, four instances) | n = 48: three of four, 230–282 s (#3 > 300 s, #0 out of memory after 290 s) | n = 48: three of four, 162–222 s (#3 refuted in 222 s; #0 at the memory bound after 85 s); `qbf/44#3` at the memory bound |
| wide-m3 (k literals) | k = 1 024: 55 ms, 2 048 at the recursion limit | k = 2 048: 173 ms |
| wide-m4 | k = 1 024: 52 ms, 2 048 at the recursion limit | k = 2 048: 167 ms |
| mix (k pairs) | k = 10: 146 s (11 > 1 200 s) | k = 10: 62.5 s (11 > 1 200 s) |
| counter (n tokens) | n = 64: 80.7 s | n = 64: 55 µs |
| counter-over | refuted up to 16; copy bound at 32 (2.2 s) and 64 (85 s) | refuted at every size, 64 in 45 µs |
| chain (k clauses) | k = 256: 321 ms | k = 256: 275 ms |
| additive (depth d) | d = 16: 210 ms | d = 16: 182 ms |

LLTP, one thread, 5 s, problems solved within the limit (the counts of
"Decided by group" below, summed per file):

| file | first | second | third |
|---|--:|--:|--:|
| `lltp-intuitionistic` (4 495 problems, copy bound 3) | 737 | 2 047 | 3 610 |
| `lltp-classical` (4 512) | 742 | 2 006 | 3 621 |
| `lltp-rarer` (the backward search alone) | – | 969 | 3 608 |
| `lltp-forward` (the forward search alone, 30 copies) | – | 2 391 | 4 026 |
| `lltp-copies-10` (1 003 that ended at the copy bound) | 411 | 458 | 592 |
| `lltp-recursion` (995 that ended at the recursion limit) | 56 | 280 | 895 |
| `lltp-all-cores` (1 102, every core) | 29 | 623 | 993 |
| `lltp-default` (4 495, the command's default, 2 s) | – | – | 4 024 |

The Petri nets make most of it: 3 071 of 3 137 in the intuitionistic
pass (1 520 in the second baseline), the 171 of `lltp-copies-10` all
(89), 895 of `lltp-recursion`'s 995 (280). Outside the nets the
copy-bound-3 passes gain a problem in each of `SYJ-cbv`, the three KLE
translations and two in each SYN translation; the larger bounds gain
more (`lltp-copies-10`: 14, 13 and 10 in the three SYJ translations;
`lltp-forward`: 46, 44 and 24), which is where the memo's cut entries no
longer keep a level from closing. Every verdict against a header is one
of the known wrong headers or one that the fix of the memo's cut entries
newly refutes. The nine proofs
against a header that says the problem is no theorem are `SYJ212+1.001`,
`SYN001+1` and `KLE065+1`, each in three translations, known since the
first baseline. The new refutations of problems whose header says
Theorem are `SYJ103+1`, `SYJ105+1.003` and `.004` and `KLE015`, `017`,
`074`, `078`, `083`, `086` and `088` in some of their translations,
which have a classical countermodel, and `KLE075+1` in its three
translations, which has none (Sympli refutes it too, and so does the
engine with the memo switched off). No tool of the comparison proves a
problem among these refutations.

**What got slower**: on the problems that took microseconds, a fixed cost
per call. On the copy-bound-3 passes the median time of a problem both
decide is 1.5 to 2.1 times the second baseline's in the SYJ and KLE
collections, with the same stable sequents (KLE-IMP-CONJ's 162: 28 → 48
µs in the median, 14 sequents each; SYJ-cbn's 35: 67 → 146 µs), so the
set-up before the search costs 20 to 80 µs more than it did; which part
of it (the memory account's charge of the input, the readings, the
refutation's counts) was not traced. The Petri nets got faster by more:
1.72 ms → 161 µs in the median.

**The pass under the default** (`lltp-default`: no copy bound, one
thread for 0.1 s and then a pool of the other fifteen beside it, 2 s)
decides 4 024 of the 4 495 problems: 3 069 nets and 955 others, as many
others as the forward pass at 30 copies and 5 s, and 416 more than the
copy-bound-3 pass. It loses none of the problems that the old default
(copy bound 3, one thread, `lltp-intuitionistic`) decides within its
2 s, against this baseline's pass and against the second's; the three
that the old default decides between 2 and 5 s (`SYJ202+1.006` in 3.5 s,
two nets in 3.2 and 3.7 s) end at its time limit. The net that the race
lost on two cores at the time the default was chosen,
`AutoFlight_afcs_06_b_10_1`, is proved in 0.74 ms. Its latest stop:
of the 389 runs that ended at the time limit, the latest stopped at
2 138 ms, 7 after 2.1 s and none after 2.2 s, all of them SYJ problems
of the cbv and cbn translations; no run was killed or crashed. The other
82 undecided ended at the recursion limit (54) and the memory bound
(28).

The default against its two components (intuitionistic library, one
thread, 5 s):

| | default | `--bias rarer` | `--bias factors --copies 30` |
|---|--:|--:|--:|
| Petri nets decided | 3 071 | 3 071 | 3 071 |
| other collections decided | 539 | 537 | 955 |
| of those, the default does not decide | – | 0 | 416 (at the default's copy bound of 3) |
| decided by the default and not by it | – | 2 | 0 |
| default's time over it, median of both decided, nets | – | 1.01× | 1.04× |

Every core against one thread, on the 1 102 problems of
`lltp-all-cores`: 993 decided within 5 s on sixteen threads and on one,
none gained and none lost, 1.57 times slower in the median.

The passes in four streams waited for a CPU on 75 problems for over 1 %
of their time (`†` in `RESULTS.md`), where the second baseline counted
5 191, almost all of them the default's polling thread (why those
waits went was not traced). The machine was not idle throughout: a
linter, a browser and a wait loop of the session that took this
baseline (a core for ten minutes) ran beside stage 1, on the cores the
streams do not use, and a headless browser beside the default pass.

## Every baseline

### Decided by group

Problems decided (late: after the time limit) of the problems run, per group of the last.

| file | family | configuration | 2026-09-30 | 2026-10-02 | 2026-10-17 |
|---|---|---|--:|--:|--:|
| engines | 3-partition-mll-no | classical focus j1 | 4 of 4 | 4 of 4 | 4 of 4 |
| engines | 3-partition-mll-no | classical net j1 | 2 of 4 | 2 of 4 | 2 of 4 |
| engines | 3-partition-mll-yes | classical focus j1 | 4 of 4 | 4 of 4 | 4 of 4 |
| engines | 3-partition-mll-yes | classical net j1 | 3 of 4 | 3 of 4 | 4 of 4 |
| engines | additive | classical focus j1 | 3 of 4 | 3 of 4 | 4 of 4 |
| engines | cancellation | classical focus j1 | 1 of 1 | 1 of 1 | 1 of 1 |
| engines | partition-no | classical focus j1 | 2 of 2 | 4 of 4 | 4 of 4 |
| engines | partition-no | classical net j1 | 1 of 2 | 1 of 4 | 1 of 4 |
| engines | partition-table | classical focus j1 | 12 of 13 | 13 of 13 | 13 of 13 |
| engines | partition-table | classical net j1 | 9 of 13 | 9 of 13 | 9 of 13 |
| engines | partition-yes | classical focus j1 | 3 of 3 | 5 of 5 | 5 of 5 |
| engines | partition-yes | classical net j1 | 1 of 3 | 1 of 5 | 1 of 5 |
| engines | wide-m1 | classical focus j1 | 4 of 6 | 5 of 6 | 6 of 6 |
| engines | wide-m1 | classical net j1 | 6 of 6 | 6 of 6 | 6 of 6 |
| engines | wide-m2 | classical focus j1 | 4 of 6 | 5 of 6 | 6 of 6 |
| engines | wide-m2 | classical net j1 | 6 of 6 | 6 of 6 | 6 of 6 |
| engines | wide-m3 | classical focus j1 | 3 of 3 | 4 of 5 | 5 of 5 |
| engines | wide-m3 | classical net j1 | 3 of 3 | 5 of 5 | 5 of 5 |
| engines | wide-m4 | classical focus j1 | 3 of 3 | 4 of 5 | 5 of 5 |
| engines | wide-m4 | classical net j1 | 3 of 3 | 5 of 5 | 5 of 5 |
| families | 3-partition-mll-no | classical auto j1 | 4 of 4 | 4 of 4 | 4 of 4 |
| families | 3-partition-mll-yes | classical auto j1 | 4 of 4 | 4 of 4 | 4 of 4 |
| families | 3-partition-no | classical auto j1 | 1 of 2 | 2 of 2 | 2 of 2 |
| families | 3-partition-yes | classical auto j1 | 4 of 4 | 4 of 4 | 4 of 4 |
| families | additive | classical auto j1 | 4 of 4 | 4 of 4 | 4 of 4 |
| families | chain | classical auto j1 | 4 of 4 | 4 of 4 | 4 of 4 |
| families | counter | classical auto j1 | 3 of 4 | 6 of 6 | 6 of 6 |
| families | counter-over | classical auto j1 | 0 of 4 | 4 of 6 | 6 of 6 |
| families | growing | classical auto j1 | 0 of 4 | 0 of 4 | 0 of 4 |
| families | mix | mix auto j1 | 5 of 6 | 5 of 6 | 5 of 6 |
| families | partition-no | classical auto j1 | 2 of 3 | 5 of 7 | 5 of 7 |
| families | partition-yes | classical auto j1 | 3 of 4 | 8 of 9 | 8 of 9 |
| families | qbf | classical auto j1 | 15 of 20 | 34 of 36 | 34 of 36 |
| families | wide-m1 | classical auto j1 | 6 of 6 | 6 of 6 | 6 of 6 |
| families | wide-m2 | classical auto j1 | 6 of 6 | 6 of 6 | 6 of 6 |
| families | wide-m3 | classical auto j1 | 3 of 4 | 6 of 7 | 7 of 7 |
| families | wide-m4 | classical auto j1 | 4 of 5 | 7 of 8 | 8 of 8 |
| intuitionistic | 3-partition-no | intuitionistic auto j1 | 1 of 1 | 1 of 1 | 1 of 1 |
| intuitionistic | 3-partition-yes | intuitionistic auto j1 | 4 of 4 | 4 of 4 | 4 of 4 |
| intuitionistic | chain | intuitionistic auto j1 | 4 of 4 | 4 of 4 | 4 of 4 |
| intuitionistic | counter | intuitionistic auto j1 | 3 of 4 | 6 of 6 | 6 of 6 |
| intuitionistic | counter-over | intuitionistic auto j1 | 0 of 4 | 4 of 6 | 6 of 6 |
| lltp-all-cores | ILL/ILLTP-SYJ-01 | intuitionistic auto j16 | 0 of 31 | 0 of 31 | 0 of 31 |
| lltp-all-cores | ILL/ILLTP-SYJ-cbn | intuitionistic auto j16 | 0 of 31 | 0 of 31 | 0 of 31 |
| lltp-all-cores | ILL/ILLTP-SYJ-cbv | intuitionistic auto j16 | 1 of 31 | 2 of 31 | 2 of 31 |
| lltp-all-cores | ILL/petri-nets/MCC | intuitionistic auto j16 | 31 (3 late) of 1009 | 623 (2 late) of 1009 | 991 of 1009 |
| lltp-all-cores-generous | ILL/ILLTP-SYJ-01 | intuitionistic auto j16 | 0 of 5 | 0 of 5 | 0 of 5 |
| lltp-all-cores-generous | ILL/ILLTP-SYJ-cbn | intuitionistic auto j16 | 0 of 4 | 0 of 4 | 0 of 4 |
| lltp-all-cores-generous | ILL/ILLTP-SYJ-cbv | intuitionistic auto j16 | 0 of 3 | 0 of 3 | 0 of 3 |
| lltp-classical | CLL/Non-theorems | classical auto j1 | 3 of 3 | 3 of 3 | 3 of 3 |
| lltp-classical | CLL/misc | classical auto j1 | 8 of 14 | 8 of 14 | 8 of 14 |
| lltp-classical | ILL/ILLTP-LCL-01 | classical auto j1 | 0 of 2 | 0 of 2 | 0 of 2 |
| lltp-classical | ILL/ILLTP-LCL-cbn | classical auto j1 | 1 of 2 | 2 of 2 | 2 of 2 |
| lltp-classical | ILL/ILLTP-LCL-cbv | classical auto j1 | 0 of 2 | 0 of 2 | 0 of 2 |
| lltp-classical | ILL/ILLTP-SYJ-01 | classical auto j1 | 11 of 252 | 11 of 252 | 11 of 252 |
| lltp-classical | ILL/ILLTP-SYJ-cbn | classical auto j1 | 34 of 252 | 35 of 252 | 35 of 252 |
| lltp-classical | ILL/ILLTP-SYJ-cbv | classical auto j1 | 39 of 252 | 39 of 252 | 40 of 252 |
| lltp-classical | ILL/ILLTP-SYN-01 | classical auto j1 | 11 of 19 | 11 of 19 | 13 of 19 |
| lltp-classical | ILL/ILLTP-SYN-cbn | classical auto j1 | 16 of 19 | 16 of 19 | 18 of 19 |
| lltp-classical | ILL/ILLTP-SYN-cbv | classical auto j1 | 16 of 19 | 16 of 19 | 18 of 19 |
| lltp-classical | ILL/KLE-01 | classical auto j1 | 50 of 88 | 50 of 88 | 51 of 88 |
| lltp-classical | ILL/KLE-IMP-CONJ | classical auto j1 | 162 of 222 | 162 of 222 | 162 of 222 |
| lltp-classical | ILL/KLE-IMP-CONJ/ALT | classical auto j1 | 27 of 27 | 27 of 27 | 27 of 27 |
| lltp-classical | ILL/KLE-IMP-CONJ/NON-THEOREMS | classical auto j1 | 22 of 22 | 22 of 22 | 22 of 22 |
| lltp-classical | ILL/KLE-cbn | classical auto j1 | 68 of 88 | 68 of 88 | 69 of 88 |
| lltp-classical | ILL/KLE-cbv | classical auto j1 | 67 of 88 | 67 of 88 | 68 of 88 |
| lltp-classical | ILL/Non-theorems | classical auto j1 | 0 of 1 | 0 of 1 | 0 of 1 |
| lltp-classical | ILL/misc | classical auto j1 | 3 of 3 | 3 of 3 | 3 of 3 |
| lltp-classical | ILL/petri-nets/MCC | classical auto j1 | 204 of 3137 | 1468 (2 late) of 3137 | 3071 of 3137 |
| lltp-classical-generous | ILL/ILLTP-SYJ-01 | classical auto j1 | 0 of 4 | 0 of 4 | 0 of 4 |
| lltp-classical-generous | ILL/ILLTP-SYJ-cbn | classical auto j1 | 0 of 2 | 0 of 2 | 0 of 2 |
| lltp-classical-generous | ILL/ILLTP-SYJ-cbv | classical auto j1 | 0 of 3 | 0 of 3 | 0 of 3 |
| lltp-classical-generous | ILL/petri-nets/MCC | classical auto j1 | 0 of 25 | 15 of 25 | 25 of 25 |
| lltp-copies-10 | ILL/ILLTP-LCL-01 | intuitionistic auto j1 | 1 of 2 | 1 of 2 | 2 of 2 |
| lltp-copies-10 | ILL/ILLTP-LCL-cbn | intuitionistic auto j1 | 2 of 2 | 2 of 2 | 2 of 2 |
| lltp-copies-10 | ILL/ILLTP-LCL-cbv | intuitionistic auto j1 | 1 of 2 | 1 of 2 | 2 of 2 |
| lltp-copies-10 | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | 37 of 209 | 37 of 209 | 51 of 209 |
| lltp-copies-10 | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | 60 of 209 | 75 of 209 | 88 of 209 |
| lltp-copies-10 | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | 83 of 209 | 83 of 209 | 93 of 209 |
| lltp-copies-10 | ILL/ILLTP-SYN-01 | intuitionistic auto j1 | 5 of 8 | 5 of 8 | 7 of 8 |
| lltp-copies-10 | ILL/ILLTP-SYN-cbn | intuitionistic auto j1 | 5 of 8 | 5 of 8 | 8 of 8 |
| lltp-copies-10 | ILL/ILLTP-SYN-cbv | intuitionistic auto j1 | 5 of 8 | 5 of 8 | 8 of 8 |
| lltp-copies-10 | ILL/KLE-01 | intuitionistic auto j1 | 29 of 38 | 29 of 38 | 31 of 38 |
| lltp-copies-10 | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | 60 of 60 | 60 of 60 | 60 of 60 |
| lltp-copies-10 | ILL/KLE-cbn | intuitionistic auto j1 | 32 of 38 | 34 of 38 | 36 of 38 |
| lltp-copies-10 | ILL/KLE-cbv | intuitionistic auto j1 | 32 of 38 | 32 of 38 | 33 of 38 |
| lltp-copies-10 | ILL/Non-theorems | intuitionistic auto j1 | 0 of 1 | 0 of 1 | 0 of 1 |
| lltp-copies-10 | ILL/petri-nets/MCC | intuitionistic auto j1 | 59 of 171 | 89 of 171 | 171 of 171 |
| lltp-default | ILL/ILLTP-LCL-01 | intuitionistic auto j16 pool after 0.1s |  |  | 2 of 2 |
| lltp-default | ILL/ILLTP-LCL-cbn | intuitionistic auto j16 pool after 0.1s |  |  | 2 of 2 |
| lltp-default | ILL/ILLTP-LCL-cbv | intuitionistic auto j16 pool after 0.1s |  |  | 2 of 2 |
| lltp-default | ILL/ILLTP-SYJ-01 | intuitionistic auto j16 pool after 0.1s |  |  | 101 of 252 |
| lltp-default | ILL/ILLTP-SYJ-cbn | intuitionistic auto j16 pool after 0.1s |  |  | 137 of 252 |
| lltp-default | ILL/ILLTP-SYJ-cbv | intuitionistic auto j16 pool after 0.1s |  |  | 117 of 252 |
| lltp-default | ILL/ILLTP-SYN-01 | intuitionistic auto j16 pool after 0.1s |  |  | 19 of 19 |
| lltp-default | ILL/ILLTP-SYN-cbn | intuitionistic auto j16 pool after 0.1s |  |  | 19 of 19 |
| lltp-default | ILL/ILLTP-SYN-cbv | intuitionistic auto j16 pool after 0.1s |  |  | 19 of 19 |
| lltp-default | ILL/KLE-01 | intuitionistic auto j16 pool after 0.1s |  |  | 87 of 88 |
| lltp-default | ILL/KLE-IMP-CONJ | intuitionistic auto j16 pool after 0.1s |  |  | 222 of 222 |
| lltp-default | ILL/KLE-IMP-CONJ/ALT | intuitionistic auto j16 pool after 0.1s |  |  | 27 of 27 |
| lltp-default | ILL/KLE-IMP-CONJ/NON-THEOREMS | intuitionistic auto j16 pool after 0.1s |  |  | 22 of 22 |
| lltp-default | ILL/KLE-cbn | intuitionistic auto j16 pool after 0.1s |  |  | 88 of 88 |
| lltp-default | ILL/KLE-cbv | intuitionistic auto j16 pool after 0.1s |  |  | 88 of 88 |
| lltp-default | ILL/Non-theorems | intuitionistic auto j16 pool after 0.1s |  |  | 0 of 1 |
| lltp-default | ILL/misc | intuitionistic auto j16 pool after 0.1s |  |  | 3 of 3 |
| lltp-default | ILL/petri-nets/MCC | intuitionistic auto j16 pool after 0.1s |  |  | 3069 of 3137 |
| lltp-forward | ILL/ILLTP-LCL-01 | intuitionistic auto j1 bias factors |  | 1 of 2 | 2 of 2 |
| lltp-forward | ILL/ILLTP-LCL-cbn | intuitionistic auto j1 bias factors |  | 2 of 2 | 2 of 2 |
| lltp-forward | ILL/ILLTP-LCL-cbv | intuitionistic auto j1 bias factors |  | 1 of 2 | 2 of 2 |
| lltp-forward | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 bias factors |  | 53 of 252 | 99 of 252 |
| lltp-forward | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 bias factors |  | 98 of 252 | 142 of 252 |
| lltp-forward | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 bias factors |  | 94 of 252 | 118 of 252 |
| lltp-forward | ILL/ILLTP-SYN-01 | intuitionistic auto j1 bias factors |  | 16 of 19 | 19 of 19 |
| lltp-forward | ILL/ILLTP-SYN-cbn | intuitionistic auto j1 bias factors |  | 16 of 19 | 19 of 19 |
| lltp-forward | ILL/ILLTP-SYN-cbv | intuitionistic auto j1 bias factors |  | 16 of 19 | 19 of 19 |
| lltp-forward | ILL/KLE-01 | intuitionistic auto j1 bias factors |  | 80 of 88 | 86 of 88 |
| lltp-forward | ILL/KLE-IMP-CONJ | intuitionistic auto j1 bias factors |  | 222 of 222 | 222 of 222 |
| lltp-forward | ILL/KLE-IMP-CONJ/ALT | intuitionistic auto j1 bias factors |  | 27 of 27 | 27 of 27 |
| lltp-forward | ILL/KLE-IMP-CONJ/NON-THEOREMS | intuitionistic auto j1 bias factors |  | 22 of 22 | 22 of 22 |
| lltp-forward | ILL/KLE-cbn | intuitionistic auto j1 bias factors |  | 84 of 88 | 87 of 88 |
| lltp-forward | ILL/KLE-cbv | intuitionistic auto j1 bias factors |  | 82 of 88 | 86 of 88 |
| lltp-forward | ILL/Non-theorems | intuitionistic auto j1 bias factors |  | 0 of 1 | 0 of 1 |
| lltp-forward | ILL/misc | intuitionistic auto j1 bias factors |  | 3 of 3 | 3 of 3 |
| lltp-forward | ILL/petri-nets/MCC | intuitionistic auto j1 bias factors |  | 1576 (2 late) of 3137 | 3071 of 3137 |
| lltp-intuitionistic | ILL/ILLTP-LCL-01 | intuitionistic auto j1 | 0 of 2 | 0 of 2 | 0 of 2 |
| lltp-intuitionistic | ILL/ILLTP-LCL-cbn | intuitionistic auto j1 | 1 of 2 | 2 of 2 | 2 of 2 |
| lltp-intuitionistic | ILL/ILLTP-LCL-cbv | intuitionistic auto j1 | 0 of 2 | 0 of 2 | 0 of 2 |
| lltp-intuitionistic | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | 11 of 252 | 11 of 252 | 11 of 252 |
| lltp-intuitionistic | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | 34 of 252 | 35 of 252 | 35 of 252 |
| lltp-intuitionistic | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | 39 of 252 | 39 of 252 | 40 of 252 |
| lltp-intuitionistic | ILL/ILLTP-SYN-01 | intuitionistic auto j1 | 11 of 19 | 11 of 19 | 13 of 19 |
| lltp-intuitionistic | ILL/ILLTP-SYN-cbn | intuitionistic auto j1 | 16 of 19 | 16 of 19 | 18 of 19 |
| lltp-intuitionistic | ILL/ILLTP-SYN-cbv | intuitionistic auto j1 | 16 of 19 | 16 of 19 | 18 of 19 |
| lltp-intuitionistic | ILL/KLE-01 | intuitionistic auto j1 | 50 of 88 | 50 of 88 | 51 of 88 |
| lltp-intuitionistic | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | 162 of 222 | 162 of 222 | 162 of 222 |
| lltp-intuitionistic | ILL/KLE-IMP-CONJ/ALT | intuitionistic auto j1 | 27 of 27 | 27 of 27 | 27 of 27 |
| lltp-intuitionistic | ILL/KLE-IMP-CONJ/NON-THEOREMS | intuitionistic auto j1 | 22 of 22 | 22 of 22 | 22 of 22 |
| lltp-intuitionistic | ILL/KLE-cbn | intuitionistic auto j1 | 68 of 88 | 68 of 88 | 69 of 88 |
| lltp-intuitionistic | ILL/KLE-cbv | intuitionistic auto j1 | 67 of 88 | 67 of 88 | 68 of 88 |
| lltp-intuitionistic | ILL/Non-theorems | intuitionistic auto j1 | 0 of 1 | 0 of 1 | 0 of 1 |
| lltp-intuitionistic | ILL/misc | intuitionistic auto j1 | 3 of 3 | 3 of 3 | 3 of 3 |
| lltp-intuitionistic | ILL/petri-nets/MCC | intuitionistic auto j1 | 210 of 3137 | 1520 (2 late) of 3137 | 3071 of 3137 |
| lltp-intuitionistic-generous | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | 0 of 4 | 0 of 4 | 0 of 4 |
| lltp-intuitionistic-generous | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | 0 of 2 | 0 of 2 | 0 of 2 |
| lltp-intuitionistic-generous | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | 0 of 3 | 0 of 3 | 0 of 3 |
| lltp-intuitionistic-generous | ILL/petri-nets/MCC | intuitionistic auto j1 | 0 of 19 | 11 of 19 | 19 of 19 |
| lltp-rarer | ILL/ILLTP-LCL-01 | intuitionistic auto j1 bias rarer |  | 0 of 2 | 0 of 2 |
| lltp-rarer | ILL/ILLTP-LCL-cbn | intuitionistic auto j1 bias rarer |  | 1 of 2 | 1 of 2 |
| lltp-rarer | ILL/ILLTP-LCL-cbv | intuitionistic auto j1 bias rarer |  | 0 of 2 | 0 of 2 |
| lltp-rarer | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 bias rarer |  | 11 of 252 | 11 of 252 |
| lltp-rarer | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 bias rarer |  | 34 of 252 | 34 of 252 |
| lltp-rarer | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 bias rarer |  | 39 of 252 | 40 of 252 |
| lltp-rarer | ILL/ILLTP-SYN-01 | intuitionistic auto j1 bias rarer |  | 11 of 19 | 13 of 19 |
| lltp-rarer | ILL/ILLTP-SYN-cbn | intuitionistic auto j1 bias rarer |  | 16 of 19 | 18 of 19 |
| lltp-rarer | ILL/ILLTP-SYN-cbv | intuitionistic auto j1 bias rarer |  | 16 of 19 | 18 of 19 |
| lltp-rarer | ILL/KLE-01 | intuitionistic auto j1 bias rarer |  | 50 of 88 | 51 of 88 |
| lltp-rarer | ILL/KLE-IMP-CONJ | intuitionistic auto j1 bias rarer |  | 162 of 222 | 162 of 222 |
| lltp-rarer | ILL/KLE-IMP-CONJ/ALT | intuitionistic auto j1 bias rarer |  | 27 of 27 | 27 of 27 |
| lltp-rarer | ILL/KLE-IMP-CONJ/NON-THEOREMS | intuitionistic auto j1 bias rarer |  | 22 of 22 | 22 of 22 |
| lltp-rarer | ILL/KLE-cbn | intuitionistic auto j1 bias rarer |  | 68 of 88 | 69 of 88 |
| lltp-rarer | ILL/KLE-cbv | intuitionistic auto j1 bias rarer |  | 67 of 88 | 68 of 88 |
| lltp-rarer | ILL/Non-theorems | intuitionistic auto j1 bias rarer |  | 0 of 1 | 0 of 1 |
| lltp-rarer | ILL/misc | intuitionistic auto j1 bias rarer |  | 3 of 3 | 3 of 3 |
| lltp-rarer | ILL/petri-nets/MCC | intuitionistic auto j1 bias rarer |  | 442 of 3137 | 3071 of 3137 |
| lltp-recursion | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | 0 of 22 | 0 of 22 | 0 of 22 |
| lltp-recursion | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | 0 of 22 | 0 of 22 | 0 of 22 |
| lltp-recursion | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | 0 of 22 | 0 of 22 | 0 of 22 |
| lltp-recursion | ILL/petri-nets/MCC | intuitionistic auto j1 | 56 of 929 | 280 of 929 | 895 of 929 |
| lltp-recursion-generous | ILL/petri-nets/MCC | intuitionistic auto j1 | 2 (2 late) of 9 | 5 of 9 | 9 of 9 |
| long-1 | 3-partition-no | classical auto j1 | 1 of 1 | 1 of 1 | 1 of 1 |
| long-1 | counter | classical auto j1 | 0 of 1 | 2 of 2 | 2 of 2 |
| long-1 | partition-no | classical auto j1 | 1 of 1 | 1 of 2 | 1 of 2 |
| long-1 | partition-yes | classical auto j1 | 0 of 1 | 1 of 2 | 1 of 2 |
| long-2 | mix | mix auto j1 | 0 of 1 | 0 of 1 | 0 of 1 |
| long-2 | qbf | classical auto j1 | 0 of 1 | 1 of 2 | 1 of 2 |
| long-2 | wide-m3 | classical auto j1 | 1 of 1 | 1 of 1 | 1 of 1 |
| long-2 | wide-m4 | classical auto j1 | 1 of 1 | 1 of 1 | 1 of 1 |
| parallel | 3-partition-no | classical auto j1 | 1 of 2 | 2 of 2 | 2 of 2 |
| parallel | 3-partition-no | classical auto j16 | 2 of 2 | 2 of 2 | 2 of 2 |
| parallel | 3-partition-no | classical auto j2 | 1 of 2 | 2 of 2 | 2 of 2 |
| parallel | 3-partition-no | classical auto j4 | 2 of 2 | 2 of 2 | 2 of 2 |
| parallel | 3-partition-no | classical auto j8 | 2 of 2 | 2 of 2 | 2 of 2 |
| parallel | 3-partition-yes | classical auto j1 | 4 of 4 | 4 of 4 | 4 of 4 |
| parallel | 3-partition-yes | classical auto j16 | 4 of 4 | 4 of 4 | 4 of 4 |
| parallel | 3-partition-yes | classical auto j2 | 4 of 4 | 4 of 4 | 4 of 4 |
| parallel | 3-partition-yes | classical auto j4 | 4 of 4 | 4 of 4 | 4 of 4 |
| parallel | 3-partition-yes | classical auto j8 | 4 of 4 | 4 of 4 | 4 of 4 |
| parallel | counter | classical auto j1 | 3 of 4 | 6 of 6 | 6 of 6 |
| parallel | counter | classical auto j16 | 3 of 4 | 6 of 6 | 6 of 6 |
| parallel | counter | classical auto j2 | 3 of 4 | 6 of 6 | 6 of 6 |
| parallel | counter | classical auto j4 | 3 of 4 | 6 of 6 | 6 of 6 |
| parallel | counter | classical auto j8 | 3 of 4 | 6 of 6 | 6 of 6 |
| parallel | counter-over | classical auto j1 | 0 of 4 | 4 of 6 | 6 of 6 |
| parallel | counter-over | classical auto j16 | 0 of 4 | 4 of 6 | 6 of 6 |
| parallel | counter-over | classical auto j2 | 0 of 4 | 4 of 6 | 6 of 6 |
| parallel | counter-over | classical auto j4 | 0 of 4 | 4 of 6 | 6 of 6 |
| parallel | counter-over | classical auto j8 | 0 of 4 | 4 of 6 | 6 of 6 |
| parallel | mix | mix auto j1 | 2 of 4 | 2 of 4 | 3 of 4 |
| parallel | mix | mix auto j16 | 2 of 4 | 2 of 4 | 2 of 4 |
| parallel | mix | mix auto j2 | 2 of 4 | 2 of 4 | 3 of 4 |
| parallel | mix | mix auto j4 | 2 of 4 | 2 of 4 | 3 of 4 |
| parallel | mix | mix auto j8 | 2 of 4 | 2 of 4 | 3 of 4 |
| parallel | partition-no | classical auto j1 | 1 of 2 | 3 of 4 | 3 of 4 |
| parallel | partition-no | classical auto j16 | 1 of 2 | 3 of 4 | 3 of 4 |
| parallel | partition-no | classical auto j2 | 1 of 2 | 3 of 4 | 3 of 4 |
| parallel | partition-no | classical auto j4 | 1 of 2 | 3 of 4 | 3 of 4 |
| parallel | partition-no | classical auto j8 | 1 of 2 | 3 of 4 | 3 of 4 |
| parallel | partition-yes | classical auto j1 | 2 of 3 | 5 of 5 | 5 of 5 |
| parallel | partition-yes | classical auto j16 | 2 of 3 | 4 of 5 | 5 of 5 |
| parallel | partition-yes | classical auto j2 | 2 of 3 | 4 of 5 | 5 of 5 |
| parallel | partition-yes | classical auto j4 | 2 of 3 | 4 of 5 | 5 of 5 |
| parallel | partition-yes | classical auto j8 | 2 of 3 | 4 of 5 | 5 of 5 |
| parallel | qbf | classical auto j1 | 7 of 12 | 19 of 20 | 19 of 20 |
| parallel | qbf | classical auto j16 | 7 of 12 | 19 of 20 | 14 of 20 |
| parallel | qbf | classical auto j2 | 7 of 12 | 19 of 20 | 14 of 20 |
| parallel | qbf | classical auto j4 | 7 of 12 | 19 of 20 | 14 of 20 |
| parallel | qbf | classical auto j8 | 7 of 12 | 19 of 20 | 14 of 20 |
| parallel | wide-m3 | classical auto j1 | 2 of 3 | 3 of 3 | 3 of 3 |
| parallel | wide-m3 | classical auto j16 | 2 of 3 | 3 of 3 | 3 of 3 |
| parallel | wide-m3 | classical auto j2 | 2 of 3 | 3 of 3 | 3 of 3 |
| parallel | wide-m3 | classical auto j4 | 2 of 3 | 3 of 3 | 3 of 3 |
| parallel | wide-m3 | classical auto j8 | 2 of 3 | 3 of 3 | 3 of 3 |
| parallel | wide-m4 | classical auto j1 | 2 of 3 | 3 of 3 | 3 of 3 |
| parallel | wide-m4 | classical auto j16 | 2 of 3 | 3 of 3 | 3 of 3 |
| parallel | wide-m4 | classical auto j2 | 2 of 3 | 3 of 3 | 3 of 3 |
| parallel | wide-m4 | classical auto j4 | 2 of 3 | 3 of 3 | 3 of 3 |
| parallel | wide-m4 | classical auto j8 | 2 of 3 | 3 of 3 | 3 of 3 |
| parallel-net | 3-partition-mll-no | classical net j1 | 2 of 3 | 2 of 3 | 2 of 3 |
| parallel-net | 3-partition-mll-no | classical net j16 | 2 of 3 | 2 of 3 | 2 of 3 |
| parallel-net | 3-partition-mll-no | classical net j2 | 2 of 3 | 2 of 3 | 2 of 3 |
| parallel-net | 3-partition-mll-no | classical net j4 | 2 of 3 | 2 of 3 | 2 of 3 |
| parallel-net | 3-partition-mll-no | classical net j8 | 2 of 3 | 2 of 3 | 2 of 3 |
| parallel-net | partition-table | classical net j1 | 9 of 13 | 9 of 13 | 9 of 13 |
| parallel-net | partition-table | classical net j16 | 10 of 13 | 10 of 13 | 10 of 13 |
| parallel-net | partition-table | classical net j2 | 9 of 13 | 9 of 13 | 9 of 13 |
| parallel-net | partition-table | classical net j4 | 10 of 13 | 10 of 13 | 10 of 13 |
| parallel-net | partition-table | classical net j8 | 10 of 13 | 10 of 13 | 10 of 13 |
| period-1 | 3-partition-mll-no | classical net j1 period 1 | 2 of 2 | 2 of 2 | 2 of 2 |
| period-1 | partition-no | classical net j1 period 1 | 1 of 2 | 1 of 2 | 1 of 2 |
| period-1 | partition-yes | classical net j1 period 1 | 1 of 1 | 1 of 1 | 1 of 1 |
| period-1 | wide-m1 | classical net j1 period 1 | 2 of 2 | 2 of 2 | 2 of 2 |
| period-1 | wide-m2 | classical net j1 period 1 | 2 of 2 | 2 of 2 | 2 of 2 |
| period-16 | 3-partition-mll-no | classical net j1 period 16 | 1 of 2 | 1 of 2 | 1 of 2 |
| period-16 | partition-no | classical net j1 period 16 | 0 of 2 | 0 of 2 | 0 of 2 |
| period-16 | partition-yes | classical net j1 period 16 | 0 of 1 | 0 of 1 | 0 of 1 |
| period-16 | wide-m1 | classical net j1 period 16 | 2 of 2 | 2 of 2 | 2 of 2 |
| period-16 | wide-m2 | classical net j1 period 16 | 2 of 2 | 2 of 2 | 2 of 2 |
| period-2 | 3-partition-mll-no | classical net j1 period 2 | 2 of 2 | 2 of 2 | 2 of 2 |
| period-2 | partition-no | classical net j1 period 2 | 1 of 2 | 1 of 2 | 1 of 2 |
| period-2 | partition-yes | classical net j1 period 2 | 1 of 1 | 1 of 1 | 1 of 1 |
| period-2 | wide-m1 | classical net j1 period 2 | 2 of 2 | 2 of 2 | 2 of 2 |
| period-2 | wide-m2 | classical net j1 period 2 | 2 of 2 | 2 of 2 | 2 of 2 |
| period-8 | 3-partition-mll-no | classical net j1 period 8 | 1 of 2 | 1 of 2 | 1 of 2 |
| period-8 | partition-no | classical net j1 period 8 | 0 of 2 | 0 of 2 | 0 of 2 |
| period-8 | partition-yes | classical net j1 period 8 | 0 of 1 | 0 of 1 | 0 of 1 |
| period-8 | wide-m1 | classical net j1 period 8 | 2 of 2 | 2 of 2 | 2 of 2 |
| period-8 | wide-m2 | classical net j1 period 8 | 2 of 2 | 2 of 2 | 2 of 2 |

### Generated and listed problems

| file | family | configuration | problem | 2026-09-30 | 2026-10-02 | 2026-10-17 |
|---|---|---|---|--:|--:|--:|
| engines | 3-partition-mll-no | classical focus j1 | 3-partition-mll-no/4 | 15 µs ✗ | 20 µs ✗ | 39 µs ✗ |
| engines | 3-partition-mll-no | classical focus j1 | 3-partition-mll-no/5 | 18 µs ✗ | 20 µs ✗ | 41 µs ✗ |
| engines | 3-partition-mll-no | classical focus j1 | 3-partition-mll-no/6 | 17 µs ✗ | 22 µs ✗ | 41 µs ✗ |
| engines | 3-partition-mll-no | classical focus j1 | 3-partition-mll-no/8 | 16 µs ✗ | 20 µs ✗ | 46 µs ✗ |
| engines | 3-partition-mll-no | classical net j1 | 3-partition-mll-no/4 | 1.80 s ✗ | 1.86 s ✗ | 1.83 s ✗ |
| engines | 3-partition-mll-no | classical net j1 | 3-partition-mll-no/5 | 56.31 s ✗ | 57.68 s ✗ | 56.13 s ✗ |
| engines | 3-partition-mll-no | classical net j1 | 3-partition-mll-no/6 | > 60 s | > 60 s | > 60 s |
| engines | 3-partition-mll-no | classical net j1 | 3-partition-mll-no/8 | > 60 s | > 60 s | > 60 s |
| engines | 3-partition-mll-yes | classical focus j1 | 3-partition-mll-yes/12 | 37 µs ✓ | 42 µs ✓ | 57 µs ✓ |
| engines | 3-partition-mll-yes | classical focus j1 | 3-partition-mll-yes/4 | 31 µs ✓ | 40 µs ✓ | 64 µs ✓ |
| engines | 3-partition-mll-yes | classical focus j1 | 3-partition-mll-yes/6 | 31 µs ✓ | 42 µs ✓ | 55 µs ✓ |
| engines | 3-partition-mll-yes | classical focus j1 | 3-partition-mll-yes/8 | 33 µs ✓ | 45 µs ✓ | 55 µs ✓ |
| engines | 3-partition-mll-yes | classical net j1 | 3-partition-mll-yes/12 | > 60 s | > 60 s | 170 µs ✓ |
| engines | 3-partition-mll-yes | classical net j1 | 3-partition-mll-yes/4 | 802 µs ✓ | 806 µs ✓ | 62 µs ✓ |
| engines | 3-partition-mll-yes | classical net j1 | 3-partition-mll-yes/6 | 114.7 ms ✓ | 117.3 ms ✓ | 88 µs ✓ |
| engines | 3-partition-mll-yes | classical net j1 | 3-partition-mll-yes/8 | 5.55 s ✓ | 5.68 s ✓ | 107 µs ✓ |
| engines | additive | classical focus j1 | additive/12 | 354.7 ms ✓ | 320.2 ms ✓ | 38.1 ms ✓ |
| engines | additive | classical focus j1 | additive/14 | 6.19 s ✓ | 5.41 s ✓ | 391.5 ms ✓ |
| engines | additive | classical focus j1 | additive/16 | 12.21 s unknown | 9.95 s unknown | 5.79 s ✓ |
| engines | additive | classical focus j1 | additive/8 | 1.6 ms ✓ | 1.5 ms ✓ | 873 µs ✓ |
| engines | cancellation | classical focus j1 | cancellation/3-partition-4 | 50.68 s ✓ | 319 µs ✓ | 300 µs ✓ |
| engines | partition-no | classical focus j1 | partition-no/12 |  | 3.91 s ✗ | 3.81 s ✗ |
| engines | partition-no | classical focus j1 | partition-no/3 | 4.2 ms ✗ | 59 µs ✗ | 71 µs ✗ |
| engines | partition-no | classical focus j1 | partition-no/4 | 111.1 ms ✗ | 138 µs ✗ | 150 µs ✗ |
| engines | partition-no | classical focus j1 | partition-no/9 |  | 86.1 ms ✗ | 89.0 ms ✗ |
| engines | partition-no | classical net j1 | partition-no/12 |  | > 60 s | > 60 s |
| engines | partition-no | classical net j1 | partition-no/3 | 3.45 s ✗ | 3.57 s ✗ | 3.68 s ✗ |
| engines | partition-no | classical net j1 | partition-no/4 | > 60 s | > 60 s | > 60 s |
| engines | partition-no | classical net j1 | partition-no/9 |  | > 60 s | > 60 s |
| engines | partition-table | classical focus j1 | partition-table/1-1 | 38 µs ✓ | 33 µs ✓ | 51 µs ✓ |
| engines | partition-table | classical focus j1 | partition-table/1-1-1-1-1-7 | 58.37 s ✗ | 1.3 ms ✗ | 1.2 ms ✗ |
| engines | partition-table | classical focus j1 | partition-table/1-1-1-5 | 61.2 ms ✗ | 149 µs ✗ | 141 µs ✗ |
| engines | partition-table | classical focus j1 | partition-table/1-1-2-4 | 526 µs ✓ | 48 µs ✓ | 70 µs ✓ |
| engines | partition-table | classical focus j1 | partition-table/1-1-4 | 2.3 ms ✗ | 56 µs ✗ | 81 µs ✗ |
| engines | partition-table | classical focus j1 | partition-table/1-2-3-4-5-5 | 23.15 s ✓ | 80 µs ✓ | 97 µs ✓ |
| engines | partition-table | classical focus j1 | partition-table/1-2-5 | 5.8 ms ✗ | 59 µs ✗ | 83 µs ✗ |
| engines | partition-table | classical focus j1 | partition-table/1-3 | 131 µs ✗ | 29 µs ✗ | 50 µs ✗ |
| engines | partition-table | classical focus j1 | partition-table/2-1-1 | 165 µs ✓ | 45 µs ✓ | 67 µs ✓ |
| engines | partition-table | classical focus j1 | partition-table/2-2-1-1 | 2.2 ms ✓ | 62 µs ✓ | 82 µs ✓ |
| engines | partition-table | classical focus j1 | partition-table/2-2-2-2-2-2-9-1 | > 60 s | 124 µs ✓ | 129 µs ✓ |
| engines | partition-table | classical focus j1 | partition-table/2-3-2-1 | 6.1 ms ✓ | 68 µs ✓ | 75 µs ✓ |
| engines | partition-table | classical focus j1 | partition-table/3-3-3-1 | 112.2 ms ✗ | 143 µs ✗ | 137 µs ✗ |
| engines | partition-table | classical net j1 | partition-table/1-1 | 36 µs ✓ | 34 µs ✓ | 51 µs ✓ |
| engines | partition-table | classical net j1 | partition-table/1-1-1-1-1-7 | > 60 s | > 60 s | > 60 s |
| engines | partition-table | classical net j1 | partition-table/1-1-1-5 | 7.50 s ✗ | 7.59 s ✗ | 7.43 s ✗ |
| engines | partition-table | classical net j1 | partition-table/1-1-2-4 | 1.03 s ✓ | 1.10 s ✓ | 1.04 s ✓ |
| engines | partition-table | classical net j1 | partition-table/1-1-4 | 39.9 ms ✗ | 40.7 ms ✗ | 40.2 ms ✗ |
| engines | partition-table | classical net j1 | partition-table/1-2-3-4-5-5 | > 60 s | > 60 s | > 60 s |
| engines | partition-table | classical net j1 | partition-table/1-2-5 | 3.66 s ✗ | 3.75 s ✗ | 3.68 s ✗ |
| engines | partition-table | classical net j1 | partition-table/1-3 | 394 µs ✗ | 372 µs ✗ | 388 µs ✗ |
| engines | partition-table | classical net j1 | partition-table/2-1-1 | 657 µs ✓ | 644 µs ✓ | 626 µs ✓ |
| engines | partition-table | classical net j1 | partition-table/2-2-1-1 | 51.5 ms ✓ | 55.2 ms ✓ | 52.2 ms ✓ |
| engines | partition-table | classical net j1 | partition-table/2-2-2-2-2-2-9-1 | > 60 s | > 60 s | > 60 s |
| engines | partition-table | classical net j1 | partition-table/2-3-2-1 | 4.58 s ✓ | 4.67 s ✓ | 4.68 s ✓ |
| engines | partition-table | classical net j1 | partition-table/3-3-3-1 | > 60 s | > 60 s | > 60 s |
| engines | partition-yes | classical focus j1 | partition-yes/12 |  | 15.2 ms ✓ | 13.3 ms ✓ |
| engines | partition-yes | classical focus j1 | partition-yes/20 |  | 573.7 ms ✓ | 471.1 ms ✓ |
| engines | partition-yes | classical focus j1 | partition-yes/4 | 2.3 ms ✓ | 60 µs ✓ | 104 µs ✓ |
| engines | partition-yes | classical focus j1 | partition-yes/5 | 169.0 ms ✓ | 77 µs ✓ | 99 µs ✓ |
| engines | partition-yes | classical focus j1 | partition-yes/6 | 12.15 s ✓ | 130 µs ✓ | 140 µs ✓ |
| engines | partition-yes | classical net j1 | partition-yes/12 |  | > 60 s | > 60 s |
| engines | partition-yes | classical net j1 | partition-yes/20 |  | > 60 s | > 60 s |
| engines | partition-yes | classical net j1 | partition-yes/4 | 2.53 s ✓ | 2.83 s ✓ | 2.83 s ✓ |
| engines | partition-yes | classical net j1 | partition-yes/5 | > 60 s | > 60 s | > 60 s |
| engines | partition-yes | classical net j1 | partition-yes/6 | > 60 s | > 60 s | > 60 s |
| engines | wide-m1 | classical focus j1 | wide-m1/16 | 765 µs ✓ | 68 µs ✓ | 77 µs ✓ |
| engines | wide-m1 | classical focus j1 | wide-m1/2048 | 50.3 ms ? wide | 271.0 ms ? depth | 206.6 ms ✓ |
| engines | wide-m1 | classical focus j1 | wide-m1/24 | 177.5 ms ✓ | 104 µs ✓ | 101 µs ✓ |
| engines | wide-m1 | classical focus j1 | wide-m1/256 | 971 µs ? wide | 5.6 ms ✓ | 4.0 ms ✓ |
| engines | wide-m1 | classical focus j1 | wide-m1/32 | 45.53 s ✓ | 160 µs ✓ | 148 µs ✓ |
| engines | wide-m1 | classical focus j1 | wide-m1/8 | 39 µs ✓ | 43 µs ✓ | 52 µs ✓ |
| engines | wide-m1 | classical net j1 | wide-m1/16 | 111 µs ✓ | 110 µs ✓ | 120 µs ✓ |
| engines | wide-m1 | classical net j1 | wide-m1/2048 | 638.4 ms ✓ | 637.3 ms ✓ | 586.4 ms ✓ |
| engines | wide-m1 | classical net j1 | wide-m1/24 | 220 µs ✓ | 207 µs ✓ | 199 µs ✓ |
| engines | wide-m1 | classical net j1 | wide-m1/256 | 9.3 ms ✓ | 9.3 ms ✓ | 9.0 ms ✓ |
| engines | wide-m1 | classical net j1 | wide-m1/32 | 196 µs ✓ | 179 µs ✓ | 182 µs ✓ |
| engines | wide-m1 | classical net j1 | wide-m1/8 | 45 µs ✓ | 43 µs ✓ | 62 µs ✓ |
| engines | wide-m2 | classical focus j1 | wide-m2/16 | 714 µs ✓ | 66 µs ✓ | 69 µs ✓ |
| engines | wide-m2 | classical focus j1 | wide-m2/2048 | 26.9 ms ? wide | 242.7 ms ? depth | 209.9 ms ✓ |
| engines | wide-m2 | classical focus j1 | wide-m2/24 | 171.1 ms ✓ | 97 µs ✓ | 105 µs ✓ |
| engines | wide-m2 | classical focus j1 | wide-m2/256 | 566 µs ? wide | 4.3 ms ✓ | 3.2 ms ✓ |
| engines | wide-m2 | classical focus j1 | wide-m2/32 | 43.50 s ✓ | 141 µs ✓ | 128 µs ✓ |
| engines | wide-m2 | classical focus j1 | wide-m2/8 | 38 µs ✓ | 40 µs ✓ | 55 µs ✓ |
| engines | wide-m2 | classical net j1 | wide-m2/16 | 109 µs ✓ | 103 µs ✓ | 113 µs ✓ |
| engines | wide-m2 | classical net j1 | wide-m2/2048 | 629.2 ms ✓ | 635.2 ms ✓ | 610.6 ms ✓ |
| engines | wide-m2 | classical net j1 | wide-m2/24 | 217 µs ✓ | 205 µs ✓ | 205 µs ✓ |
| engines | wide-m2 | classical net j1 | wide-m2/256 | 9.3 ms ✓ | 9.4 ms ✓ | 9.0 ms ✓ |
| engines | wide-m2 | classical net j1 | wide-m2/32 | 199 µs ✓ | 179 µs ✓ | 178 µs ✓ |
| engines | wide-m2 | classical net j1 | wide-m2/8 | 44 µs ✓ | 43 µs ✓ | 55 µs ✓ |
| engines | wide-m3 | classical focus j1 | wide-m3/12 | 84 µs ✓ | 50 µs ✓ | 69 µs ✓ |
| engines | wide-m3 | classical focus j1 | wide-m3/2048 |  | 215.8 ms ? depth | 192.5 ms ✓ |
| engines | wide-m3 | classical focus j1 | wide-m3/24 | 168.6 ms ✓ | 94 µs ✓ | 96 µs ✓ |
| engines | wide-m3 | classical focus j1 | wide-m3/256 |  | 3.7 ms ✓ | 3.0 ms ✓ |
| engines | wide-m3 | classical focus j1 | wide-m3/30 | 10.85 s ✓ | 126 µs ✓ | 123 µs ✓ |
| engines | wide-m3 | classical net j1 | wide-m3/12 | 71 µs ✓ | 74 µs ✓ | 85 µs ✓ |
| engines | wide-m3 | classical net j1 | wide-m3/2048 |  | 649.4 ms ✓ | 628.1 ms ✓ |
| engines | wide-m3 | classical net j1 | wide-m3/24 | 217 µs ✓ | 213 µs ✓ | 211 µs ✓ |
| engines | wide-m3 | classical net j1 | wide-m3/256 |  | 9.7 ms ✓ | 9.4 ms ✓ |
| engines | wide-m3 | classical net j1 | wide-m3/30 | 182 µs ✓ | 166 µs ✓ | 176 µs ✓ |
| engines | wide-m4 | classical focus j1 | wide-m4/12 | 87 µs ✓ | 53 µs ✓ | 59 µs ✓ |
| engines | wide-m4 | classical focus j1 | wide-m4/2048 |  | 203.7 ms ? depth | 182.0 ms ✓ |
| engines | wide-m4 | classical focus j1 | wide-m4/24 | 171.8 ms ✓ | 92 µs ✓ | 93 µs ✓ |
| engines | wide-m4 | classical focus j1 | wide-m4/256 |  | 3.5 ms ✓ | 2.8 ms ✓ |
| engines | wide-m4 | classical focus j1 | wide-m4/28 | 2.75 s ✓ | 115 µs ✓ | 105 µs ✓ |
| engines | wide-m4 | classical net j1 | wide-m4/12 | 76 µs ✓ | 73 µs ✓ | 81 µs ✓ |
| engines | wide-m4 | classical net j1 | wide-m4/2048 |  | 673.2 ms ✓ | 651.3 ms ✓ |
| engines | wide-m4 | classical net j1 | wide-m4/24 | 223 µs ✓ | 215 µs ✓ | 218 µs ✓ |
| engines | wide-m4 | classical net j1 | wide-m4/256 |  | 10.1 ms ✓ | 9.8 ms ✓ |
| engines | wide-m4 | classical net j1 | wide-m4/28 | 290 µs ✓ | 278 µs ✓ | 258 µs ✓ |
| families | 3-partition-mll-no | classical auto j1 | 3-partition-mll-no/4 | 14 µs ✗ | 19 µs ✗ | 39 µs ✗ |
| families | 3-partition-mll-no | classical auto j1 | 3-partition-mll-no/5 | 15 µs ✗ | 18 µs ✗ | 41 µs ✗ |
| families | 3-partition-mll-no | classical auto j1 | 3-partition-mll-no/6 | 17 µs ✗ | 20 µs ✗ | 45 µs ✗ |
| families | 3-partition-mll-no | classical auto j1 | 3-partition-mll-no/8 | 16 µs ✗ | 20 µs ✗ | 40 µs ✗ |
| families | 3-partition-mll-yes | classical auto j1 | 3-partition-mll-yes/12 | 35 µs ✓ | 41 µs ✓ | 55 µs ✓ |
| families | 3-partition-mll-yes | classical auto j1 | 3-partition-mll-yes/4 | 27 µs ✓ | 36 µs ✓ | 54 µs ✓ |
| families | 3-partition-mll-yes | classical auto j1 | 3-partition-mll-yes/6 | 28 µs ✓ | 38 µs ✓ | 53 µs ✓ |
| families | 3-partition-mll-yes | classical auto j1 | 3-partition-mll-yes/8 | 31 µs ✓ | 39 µs ✓ | 56 µs ✓ |
| families | 3-partition-no | classical auto j1 | 3-partition-no/4 | 49.77 s ✗ | 299 µs ✗ | 319 µs ✗ |
| families | 3-partition-no | classical auto j1 | 3-partition-no/5 | > 300 s | 358 µs ✗ | 346 µs ✗ |
| families | 3-partition-yes | classical auto j1 | 3-partition-yes/12 | 30.1 ms ✓ | 48 µs ✓ | 64 µs ✓ |
| families | 3-partition-yes | classical auto j1 | 3-partition-yes/4 | 180 µs ✓ | 41 µs ✓ | 68 µs ✓ |
| families | 3-partition-yes | classical auto j1 | 3-partition-yes/6 | 555 µs ✓ | 43 µs ✓ | 61 µs ✓ |
| families | 3-partition-yes | classical auto j1 | 3-partition-yes/8 | 2.0 ms ✓ | 44 µs ✓ | 63 µs ✓ |
| families | additive | classical auto j1 | additive/12 | 13.6 ms ✓ | 13.8 ms ✓ | 10.8 ms ✓ |
| families | additive | classical auto j1 | additive/14 | 69.7 ms ✓ | 68.0 ms ✓ | 58.4 ms ✓ |
| families | additive | classical auto j1 | additive/16 | 527.9 ms ✓ | 209.5 ms ✓ | 181.6 ms ✓ |
| families | additive | classical auto j1 | additive/8 | 363 µs ✓ | 378 µs ✓ | 329 µs ✓ |
| families | chain | classical auto j1 | chain/128 | 269.7 ms ✓ | 40.2 ms ✓ | 33.7 ms ✓ |
| families | chain | classical auto j1 | chain/16 | 247 µs ✓ | 151 µs ✓ | 154 µs ✓ |
| families | chain | classical auto j1 | chain/256 | 2.82 s ✓ | 321.3 ms ✓ | 274.7 ms ✓ |
| families | chain | classical auto j1 | chain/64 | 22.1 ms ✓ | 5.1 ms ✓ | 4.4 ms ✓ |
| families | counter | classical auto j1 | counter/16 | > 300 s | 248 µs ✓ | 37 µs ✓ |
| families | counter | classical auto j1 | counter/2 | 11 µs ✓ | 51 µs ✓ | 27 µs ✓ |
| families | counter | classical auto j1 | counter/32 |  | 2.18 s ✓ | 43 µs ✓ |
| families | counter | classical auto j1 | counter/4 | 84 µs ✓ | 65 µs ✓ | 28 µs ✓ |
| families | counter | classical auto j1 | counter/64 |  | 80.68 s ✓ | 55 µs ✓ |
| families | counter | classical auto j1 | counter/8 | 124.0 ms ✓ | 87 µs ✓ | 34 µs ✓ |
| families | counter-over | classical auto j1 | counter-over/16 | > 300 s | 299 µs ✗ | 35 µs ✗ |
| families | counter-over | classical auto j1 | counter-over/2 | 24 µs ? bound | 55 µs ✗ | 24 µs ✗ |
| families | counter-over | classical auto j1 | counter-over/32 |  | 2.22 s ? bound | 44 µs ✗ |
| families | counter-over | classical auto j1 | counter-over/4 | 368 µs ? bound | 59 µs ✗ | 26 µs ✗ |
| families | counter-over | classical auto j1 | counter-over/64 |  | 85.03 s ? bound | 45 µs ✗ |
| families | counter-over | classical auto j1 | counter-over/8 | 492.7 ms ? bound | 110 µs ✗ | 33 µs ✗ |
| families | growing | classical auto j1 | growing/1024 | 567.8 ms ? depth | 256.8 ms ? depth | 114.7 ms ? depth |
| families | growing | classical auto j1 | growing/16 | 142 µs ? bound | 342 µs ? bound | 213 µs ? bound |
| families | growing | classical auto j1 | growing/256 | 79.3 ms ? bound | 41.8 ms ? bound | 18.4 ms ? bound |
| families | growing | classical auto j1 | growing/64 | 2.1 ms ? bound | 1.6 ms ? bound | 800 µs ? bound |
| families | mix | mix auto j1 | mix/10 | 213.99 s ✗ | 145.85 s ✗ | 62.55 s ✗ |
| families | mix | mix auto j1 | mix/11 | > 300 s | > 300 s | > 300 s |
| families | mix | mix auto j1 | mix/4 | 275 µs ✗ | 302 µs ✗ | 365 µs ✗ |
| families | mix | mix auto j1 | mix/6 | 19.9 ms ✗ | 18.4 ms ✗ | 14.0 ms ✗ |
| families | mix | mix auto j1 | mix/8 | 1.91 s ✗ | 1.55 s ✗ | 858.5 ms ✗ |
| families | mix | mix auto j1 | mix/9 | 19.82 s ✗ | 14.10 s ✗ | 7.31 s ✗ |
| families | partition-no | classical auto j1 | partition-no/12 |  | 3.87 s ✗ | 3.82 s ✗ |
| families | partition-no | classical auto j1 | partition-no/14 |  | > 300 s | > 300 s |
| families | partition-no | classical auto j1 | partition-no/15 |  | > 300 s | > 300 s |
| families | partition-no | classical auto j1 | partition-no/3 | 4.4 ms ✗ | 59 µs ✗ | 79 µs ✗ |
| families | partition-no | classical auto j1 | partition-no/4 | 113.8 ms ✗ | 141 µs ✗ | 144 µs ✗ |
| families | partition-no | classical auto j1 | partition-no/5 | > 300 s | 465 µs ✗ | 435 µs ✗ |
| families | partition-no | classical auto j1 | partition-no/9 |  | 84.9 ms ✗ | 85.5 ms ✗ |
| families | partition-yes | classical auto j1 | partition-yes/12 |  | 14.8 ms ✓ | 13.8 ms ✓ |
| families | partition-yes | classical auto j1 | partition-yes/16 |  | 186.8 ms ✓ | 147.8 ms ✓ |
| families | partition-yes | classical auto j1 | partition-yes/20 |  | 568.4 ms ✓ | 454.7 ms ✓ |
| families | partition-yes | classical auto j1 | partition-yes/24 |  | 117.66 s ✓ | 74.41 s ✓ |
| families | partition-yes | classical auto j1 | partition-yes/28 |  | > 300 s | > 300 s |
| families | partition-yes | classical auto j1 | partition-yes/4 | 2.3 ms ✓ | 55 µs ✓ | 81 µs ✓ |
| families | partition-yes | classical auto j1 | partition-yes/5 | 186.5 ms ✓ | 76 µs ✓ | 93 µs ✓ |
| families | partition-yes | classical auto j1 | partition-yes/6 | 12.74 s ✓ | 130 µs ✓ | 136 µs ✓ |
| families | partition-yes | classical auto j1 | partition-yes/7 | > 300 s | 213 µs ✓ | 211 µs ✓ |
| families | qbf | classical auto j1 | qbf/12#0 | 121.0 ms ✓ | 1.8 ms ✓ | 1.2 ms ✓ |
| families | qbf | classical auto j1 | qbf/12#1 | 28.8 ms ✗ | 502 µs ✗ | 388 µs ✗ |
| families | qbf | classical auto j1 | qbf/12#2 | 45.0 ms ✓ | 925 µs ✓ | 625 µs ✓ |
| families | qbf | classical auto j1 | qbf/12#3 | 19.2 ms ✗ | 371 µs ✗ | 297 µs ✗ |
| families | qbf | classical auto j1 | qbf/16#0 | 7.87 s ✓ | 10.1 ms ✓ | 6.0 ms ✓ |
| families | qbf | classical auto j1 | qbf/16#1 | 1.45 s ✗ | 1.5 ms ✗ | 1.1 ms ✗ |
| families | qbf | classical auto j1 | qbf/16#2 | 2.09 s ✗ | 2.4 ms ✗ | 1.7 ms ✗ |
| families | qbf | classical auto j1 | qbf/16#3 | 6.56 s ✓ | 7.7 ms ✓ | 5.4 ms ✓ |
| families | qbf | classical auto j1 | qbf/20#0 | 87.23 s ✗ | 7.3 ms ✗ | 5.3 ms ✗ |
| families | qbf | classical auto j1 | qbf/20#1 | 123.61 s ✗ | 12.0 ms ✗ | 8.2 ms ✗ |
| families | qbf | classical auto j1 | qbf/20#2 | > 300 s | 49.8 ms ✓ | 30.0 ms ✓ |
| families | qbf | classical auto j1 | qbf/20#3 | 89.32 s ✗ | 7.1 ms ✗ | 5.1 ms ✗ |
| families | qbf | classical auto j1 | qbf/24#0 | > 300 s | 163.5 ms ✗ | 101.8 ms ✗ |
| families | qbf | classical auto j1 | qbf/24#1 | > 300 s | 58.0 ms ✗ | 37.0 ms ✗ |
| families | qbf | classical auto j1 | qbf/24#2 | > 300 s | 38.7 ms ✗ | 25.7 ms ✗ |
| families | qbf | classical auto j1 | qbf/24#3 | > 300 s | 57.2 ms ✗ | 36.1 ms ✗ |
| families | qbf | classical auto j1 | qbf/32#0 |  | 875.9 ms ✗ | 552.4 ms ✗ |
| families | qbf | classical auto j1 | qbf/32#1 |  | 943.7 ms ✗ | 626.7 ms ✗ |
| families | qbf | classical auto j1 | qbf/32#2 |  | 2.04 s ✗ | 1.14 s ✗ |
| families | qbf | classical auto j1 | qbf/32#3 |  | 778.5 ms ✗ | 476.7 ms ✗ |
| families | qbf | classical auto j1 | qbf/40#0 |  | 17.53 s ✗ | 12.11 s ✗ |
| families | qbf | classical auto j1 | qbf/40#1 |  | 20.18 s ✗ | 13.65 s ✗ |
| families | qbf | classical auto j1 | qbf/40#2 |  | 23.34 s ✗ | 16.21 s ✗ |
| families | qbf | classical auto j1 | qbf/40#3 |  | 18.29 s ✗ | 12.54 s ✗ |
| families | qbf | classical auto j1 | qbf/44#0 |  | 77.60 s ✗ | 54.12 s ✗ |
| families | qbf | classical auto j1 | qbf/44#1 |  | 79.01 s ✗ | 53.43 s ✗ |
| families | qbf | classical auto j1 | qbf/44#2 |  | 115.85 s ✗ | 80.52 s ✗ |
| families | qbf | classical auto j1 | qbf/44#3 |  | 218.72 s ✗ | 28.71 s ? memory |
| families | qbf | classical auto j1 | qbf/48#0 |  | 290.27 s unknown | 84.57 s ? memory |
| families | qbf | classical auto j1 | qbf/48#1 |  | 282.04 s ✗ | 207.33 s ✗ |
| families | qbf | classical auto j1 | qbf/48#2 |  | 230.53 s ✗ | 161.73 s ✗ |
| families | qbf | classical auto j1 | qbf/48#3 |  | > 300 s | 222.46 s ✗ |
| families | qbf | classical auto j1 | qbf/8#0 | 551 µs ✗ | 123 µs ✗ | 131 µs ✗ |
| families | qbf | classical auto j1 | qbf/8#1 | 587 µs ✗ | 161 µs ✗ | 144 µs ✗ |
| families | qbf | classical auto j1 | qbf/8#2 | 830 µs ✓ | 184 µs ✓ | 156 µs ✓ |
| families | qbf | classical auto j1 | qbf/8#3 | 643 µs ✗ | 141 µs ✗ | 130 µs ✗ |
| families | wide-m1 | classical auto j1 | wide-m1/16 | 110 µs ✓ | 98 µs ✓ | 110 µs ✓ |
| families | wide-m1 | classical auto j1 | wide-m1/2048 | 610.2 ms ✓ | 581.5 ms ✓ | 523.0 ms ✓ |
| families | wide-m1 | classical auto j1 | wide-m1/24 | 198 µs ✓ | 199 µs ✓ | 192 µs ✓ |
| families | wide-m1 | classical auto j1 | wide-m1/256 | 8.8 ms ✓ | 8.2 ms ✓ | 7.7 ms ✓ |
| families | wide-m1 | classical auto j1 | wide-m1/32 | 178 µs ✓ | 166 µs ✓ | 174 µs ✓ |
| families | wide-m1 | classical auto j1 | wide-m1/8 | 40 µs ✓ | 40 µs ✓ | 54 µs ✓ |
| families | wide-m2 | classical auto j1 | wide-m2/16 | 103 µs ✓ | 97 µs ✓ | 107 µs ✓ |
| families | wide-m2 | classical auto j1 | wide-m2/2048 | 602.4 ms ✓ | 571.3 ms ✓ | 555.5 ms ✓ |
| families | wide-m2 | classical auto j1 | wide-m2/24 | 201 µs ✓ | 191 µs ✓ | 194 µs ✓ |
| families | wide-m2 | classical auto j1 | wide-m2/256 | 8.8 ms ✓ | 8.5 ms ✓ | 8.0 ms ✓ |
| families | wide-m2 | classical auto j1 | wide-m2/32 | 170 µs ✓ | 165 µs ✓ | 171 µs ✓ |
| families | wide-m2 | classical auto j1 | wide-m2/8 | 40 µs ✓ | 44 µs ✓ | 54 µs ✓ |
| families | wide-m3 | classical auto j1 | wide-m3/1024 |  | 55.2 ms ✓ | 42.8 ms ✓ |
| families | wide-m3 | classical auto j1 | wide-m3/12 | 77 µs ✓ | 44 µs ✓ | 59 µs ✓ |
| families | wide-m3 | classical auto j1 | wide-m3/2048 |  | 204.3 ms ? depth | 172.9 ms ✓ |
| families | wide-m3 | classical auto j1 | wide-m3/24 | 166.4 ms ✓ | 88 µs ✓ | 87 µs ✓ |
| families | wide-m3 | classical auto j1 | wide-m3/256 |  | 3.3 ms ✓ | 2.7 ms ✓ |
| families | wide-m3 | classical auto j1 | wide-m3/30 | 10.57 s ✓ | 112 µs ✓ | 104 µs ✓ |
| families | wide-m3 | classical auto j1 | wide-m3/36 | > 300 s | 139 µs ✓ | 128 µs ✓ |
| families | wide-m4 | classical auto j1 | wide-m4/1024 |  | 51.6 ms ✓ | 40.2 ms ✓ |
| families | wide-m4 | classical auto j1 | wide-m4/12 | 79 µs ✓ | 49 µs ✓ | 58 µs ✓ |
| families | wide-m4 | classical auto j1 | wide-m4/2048 |  | 191.8 ms ? depth | 167.4 ms ✓ |
| families | wide-m4 | classical auto j1 | wide-m4/24 | 170.2 ms ✓ | 85 µs ✓ | 92 µs ✓ |
| families | wide-m4 | classical auto j1 | wide-m4/256 |  | 3.2 ms ✓ | 2.5 ms ✓ |
| families | wide-m4 | classical auto j1 | wide-m4/28 | 2.72 s ✓ | 104 µs ✓ | 97 µs ✓ |
| families | wide-m4 | classical auto j1 | wide-m4/32 | 43.39 s ✓ | 119 µs ✓ | 108 µs ✓ |
| families | wide-m4 | classical auto j1 | wide-m4/36 | > 300 s | 137 µs ✓ | 134 µs ✓ |
| intuitionistic | 3-partition-no | intuitionistic auto j1 | 3-partition-no/4 | 25.38 s ✗ | 293 µs ✗ | 273 µs ✗ |
| intuitionistic | 3-partition-yes | intuitionistic auto j1 | 3-partition-yes/12 | 16.5 ms ✓ | 51 µs ✓ | 59 µs ✓ |
| intuitionistic | 3-partition-yes | intuitionistic auto j1 | 3-partition-yes/4 | 116 µs ✓ | 40 µs ✓ | 60 µs ✓ |
| intuitionistic | 3-partition-yes | intuitionistic auto j1 | 3-partition-yes/6 | 320 µs ✓ | 46 µs ✓ | 59 µs ✓ |
| intuitionistic | 3-partition-yes | intuitionistic auto j1 | 3-partition-yes/8 | 1.1 ms ✓ | 47 µs ✓ | 58 µs ✓ |
| intuitionistic | chain | intuitionistic auto j1 | chain/128 | 275.9 ms ✓ | 42.4 ms ✓ | 35.6 ms ✓ |
| intuitionistic | chain | intuitionistic auto j1 | chain/16 | 273 µs ✓ | 159 µs ✓ | 149 µs ✓ |
| intuitionistic | chain | intuitionistic auto j1 | chain/256 | 2.89 s ✓ | 347.4 ms ✓ | 287.5 ms ✓ |
| intuitionistic | chain | intuitionistic auto j1 | chain/64 | 22.6 ms ✓ | 5.4 ms ✓ | 4.6 ms ✓ |
| intuitionistic | counter | intuitionistic auto j1 | counter/16 | > 300 s | 251 µs ✓ | 35 µs ✓ |
| intuitionistic | counter | intuitionistic auto j1 | counter/2 | 12 µs ✓ | 50 µs ✓ | 27 µs ✓ |
| intuitionistic | counter | intuitionistic auto j1 | counter/32 |  | 19.9 ms ✓ | 42 µs ✓ |
| intuitionistic | counter | intuitionistic auto j1 | counter/4 | 48 µs ✓ | 61 µs ✓ | 28 µs ✓ |
| intuitionistic | counter | intuitionistic auto j1 | counter/64 |  | 267.7 ms ✓ | 53 µs ✓ |
| intuitionistic | counter | intuitionistic auto j1 | counter/8 | 15.4 ms ✓ | 88 µs ✓ | 32 µs ✓ |
| intuitionistic | counter-over | intuitionistic auto j1 | counter-over/16 | > 300 s | 306 µs ✗ | 35 µs ✗ |
| intuitionistic | counter-over | intuitionistic auto j1 | counter-over/2 | 21 µs ? bound | 54 µs ✗ | 22 µs ✗ |
| intuitionistic | counter-over | intuitionistic auto j1 | counter-over/32 |  | 20.3 ms ? bound | 44 µs ✗ |
| intuitionistic | counter-over | intuitionistic auto j1 | counter-over/4 | 60 µs ? bound | 66 µs ✗ | 26 µs ✗ |
| intuitionistic | counter-over | intuitionistic auto j1 | counter-over/64 |  | 268.5 ms ? bound | 54 µs ✗ |
| intuitionistic | counter-over | intuitionistic auto j1 | counter-over/8 | 18.4 ms ? bound | 107 µs ✗ | 32 µs ✗ |
| long-1 | 3-partition-no | classical auto j1 | 3-partition-no/5 | 302.42 s ✗ | 393 µs ✗ | 400 µs ✗ |
| long-1 | counter | classical auto j1 | counter/16 | > 1200 s | 280 µs ✓ | 48 µs ✓ |
| long-1 | counter | classical auto j1 | counter/64 |  | 85.14 s ✓ | 60 µs ✓ |
| long-1 | partition-no | classical auto j1 | partition-no/15 |  | > 1200 s | > 1200 s |
| long-1 | partition-no | classical auto j1 | partition-no/5 | 811.48 s ✗ | 477 µs ✗ | 441 µs ✗ |
| long-1 | partition-yes | classical auto j1 | partition-yes/28 |  | > 1200 s | > 1200 s |
| long-1 | partition-yes | classical auto j1 | partition-yes/7 | > 1200 s | 230 µs ✓ | 214 µs ✓ |
| long-2 | mix | mix auto j1 | mix/11 | > 1200 s | > 1200 s | > 1200 s |
| long-2 | qbf | classical auto j1 | qbf/24#0 | > 1200 s | 161.2 ms ✗ | 99.6 ms ✗ |
| long-2 | qbf | classical auto j1 | qbf/48#0 |  | 301.01 s unknown | 83.81 s ? memory |
| long-2 | wide-m3 | classical auto j1 | wide-m3/36 | 697.42 s ✓ | 157 µs ✓ | 154 µs ✓ |
| long-2 | wide-m4 | classical auto j1 | wide-m4/36 | 685.07 s ✓ | 144 µs ✓ | 135 µs ✓ |
| parallel | 3-partition-no | classical auto j1 | 3-partition-no/4 | 45.09 s ✗ | 395 µs ✗ | 379 µs ✗ |
| parallel | 3-partition-no | classical auto j1 | 3-partition-no/5 | > 120 s | 470 µs ✗ | 432 µs ✗ |
| parallel | 3-partition-no | classical auto j16 | 3-partition-no/4 | 5.80 s ✗ | 720 µs ✗ | 631 µs ✗ |
| parallel | 3-partition-no | classical auto j16 | 3-partition-no/5 | 37.07 s ✗ | 689 µs ✗ | 674 µs ✗ |
| parallel | 3-partition-no | classical auto j2 | 3-partition-no/4 | 23.49 s ✗ | 439 µs ✗ | 544 µs ✗ |
| parallel | 3-partition-no | classical auto j2 | 3-partition-no/5 | > 120 s | 483 µs ✗ | 567 µs ✗ |
| parallel | 3-partition-no | classical auto j4 | 3-partition-no/4 | 12.68 s ✗ | 342 µs ✗ | 428 µs ✗ |
| parallel | 3-partition-no | classical auto j4 | 3-partition-no/5 | 77.70 s ✗ | 354 µs ✗ | 461 µs ✗ |
| parallel | 3-partition-no | classical auto j8 | 3-partition-no/4 | 7.45 s ✗ | 467 µs ✗ | 550 µs ✗ |
| parallel | 3-partition-no | classical auto j8 | 3-partition-no/5 | 46.72 s ✗ | 431 µs ✗ | 562 µs ✗ |
| parallel | 3-partition-yes | classical auto j1 | 3-partition-yes/12 | 37.7 ms ✓ | 59 µs ✓ | 88 µs ✓ |
| parallel | 3-partition-yes | classical auto j1 | 3-partition-yes/4 | 217 µs ✓ | 48 µs ✓ | 103 µs ✓ |
| parallel | 3-partition-yes | classical auto j1 | 3-partition-yes/6 | 674 µs ✓ | 58 µs ✓ | 86 µs ✓ |
| parallel | 3-partition-yes | classical auto j1 | 3-partition-yes/8 | 2.4 ms ✓ | 54 µs ✓ | 86 µs ✓ |
| parallel | 3-partition-yes | classical auto j16 | 3-partition-yes/12 | 30.4 ms ✓ | 540 µs ✓ | 546 µs ✓ |
| parallel | 3-partition-yes | classical auto j16 | 3-partition-yes/4 | 706 µs ✓ | 506 µs ✓ | 591 µs ✓ |
| parallel | 3-partition-yes | classical auto j16 | 3-partition-yes/6 | 1.3 ms ✓ | 452 µs ✓ | 593 µs ✓ |
| parallel | 3-partition-yes | classical auto j16 | 3-partition-yes/8 | 2.8 ms ✓ | 601 µs ✓ | 649 µs ✓ |
| parallel | 3-partition-yes | classical auto j2 | 3-partition-yes/12 | 29.6 ms ✓ | 163 µs ✓ | 248 µs ✓ |
| parallel | 3-partition-yes | classical auto j2 | 3-partition-yes/4 | 489 µs ✓ | 209 µs ✓ | 215 µs ✓ |
| parallel | 3-partition-yes | classical auto j2 | 3-partition-yes/6 | 920 µs ✓ | 142 µs ✓ | 220 µs ✓ |
| parallel | 3-partition-yes | classical auto j2 | 3-partition-yes/8 | 2.4 ms ✓ | 164 µs ✓ | 263 µs ✓ |
| parallel | 3-partition-yes | classical auto j4 | 3-partition-yes/12 | 29.3 ms ✓ | 202 µs ✓ | 301 µs ✓ |
| parallel | 3-partition-yes | classical auto j4 | 3-partition-yes/4 | 425 µs ✓ | 198 µs ✓ | 247 µs ✓ |
| parallel | 3-partition-yes | classical auto j4 | 3-partition-yes/6 | 911 µs ✓ | 179 µs ✓ | 269 µs ✓ |
| parallel | 3-partition-yes | classical auto j4 | 3-partition-yes/8 | 2.3 ms ✓ | 183 µs ✓ | 278 µs ✓ |
| parallel | 3-partition-yes | classical auto j8 | 3-partition-yes/12 | 29.5 ms ✓ | 308 µs ✓ | 458 µs ✓ |
| parallel | 3-partition-yes | classical auto j8 | 3-partition-yes/4 | 433 µs ✓ | 328 µs ✓ | 416 µs ✓ |
| parallel | 3-partition-yes | classical auto j8 | 3-partition-yes/6 | 1.0 ms ✓ | 287 µs ✓ | 337 µs ✓ |
| parallel | 3-partition-yes | classical auto j8 | 3-partition-yes/8 | 2.5 ms ✓ | 352 µs ✓ | 423 µs ✓ |
| parallel | counter | classical auto j1 | counter/16 | > 120 s | 308 µs ✓ | 63 µs ✓ |
| parallel | counter | classical auto j1 | counter/2 | 13 µs ✓ | 65 µs ✓ | 38 µs ✓ |
| parallel | counter | classical auto j1 | counter/32 |  | 2.18 s ✓ | 72 µs ✓ |
| parallel | counter | classical auto j1 | counter/4 | 110 µs ✓ | 66 µs ✓ | 40 µs ✓ |
| parallel | counter | classical auto j1 | counter/64 |  | 78.45 s ✓ | 86 µs ✓ |
| parallel | counter | classical auto j1 | counter/8 | 129.6 ms ✓ | 98 µs ✓ | 51 µs ✓ |
| parallel | counter | classical auto j16 | counter/16 | > 120 s | 1.2 ms ✓ | 91 µs ✓ |
| parallel | counter | classical auto j16 | counter/2 | 479 µs ✓ | 465 µs ✓ | 80 µs ✓ |
| parallel | counter | classical auto j16 | counter/32 |  | 700.0 ms ✓ | 102 µs ✓ |
| parallel | counter | classical auto j16 | counter/4 | 645 µs ✓ | 559 µs ✓ | 71 µs ✓ |
| parallel | counter | classical auto j16 | counter/64 |  | 23.90 s ✓ | 116 µs ✓ |
| parallel | counter | classical auto j16 | counter/8 | 109.3 ms ✓ | 691 µs ✓ | 81 µs ✓ |
| parallel | counter | classical auto j2 | counter/16 | > 120 s | 389 µs ✓ | 83 µs ✓ |
| parallel | counter | classical auto j2 | counter/2 | 99 µs ✓ | 109 µs ✓ | 101 µs ✓ |
| parallel | counter | classical auto j2 | counter/32 |  | 2.70 s ✓ | 94 µs ✓ |
| parallel | counter | classical auto j2 | counter/4 | 225 µs ✓ | 99 µs ✓ | 73 µs ✓ |
| parallel | counter | classical auto j2 | counter/64 |  | 101.90 s ✓ | 106 µs ✓ |
| parallel | counter | classical auto j2 | counter/8 | 156.3 ms ✓ | 138 µs ✓ | 79 µs ✓ |
| parallel | counter | classical auto j4 | counter/16 | > 120 s | 648 µs ✓ | 93 µs ✓ |
| parallel | counter | classical auto j4 | counter/2 | 126 µs ✓ | 156 µs ✓ | 82 µs ✓ |
| parallel | counter | classical auto j4 | counter/32 |  | 2.02 s ✓ | 96 µs ✓ |
| parallel | counter | classical auto j4 | counter/4 | 258 µs ✓ | 161 µs ✓ | 83 µs ✓ |
| parallel | counter | classical auto j4 | counter/64 |  | 76.89 s ✓ | 102 µs ✓ |
| parallel | counter | classical auto j4 | counter/8 | 127.2 ms ✓ | 300 µs ✓ | 78 µs ✓ |
| parallel | counter | classical auto j8 | counter/16 | > 120 s | 747 µs ✓ | 93 µs ✓ |
| parallel | counter | classical auto j8 | counter/2 | 253 µs ✓ | 243 µs ✓ | 63 µs ✓ |
| parallel | counter | classical auto j8 | counter/32 |  | 629.7 ms ✓ | 100 µs ✓ |
| parallel | counter | classical auto j8 | counter/4 | 321 µs ✓ | 274 µs ✓ | 75 µs ✓ |
| parallel | counter | classical auto j8 | counter/64 |  | 22.55 s ✓ | 114 µs ✓ |
| parallel | counter | classical auto j8 | counter/8 | 105.3 ms ✓ | 419 µs ✓ | 83 µs ✓ |
| parallel | counter-over | classical auto j1 | counter-over/16 | > 120 s | 403 µs ✗ | 63 µs ✗ |
| parallel | counter-over | classical auto j1 | counter-over/2 | 27 µs ? bound | 65 µs ✗ | 49 µs ✗ |
| parallel | counter-over | classical auto j1 | counter-over/32 |  | 2.28 s ? bound | 73 µs ✗ |
| parallel | counter-over | classical auto j1 | counter-over/4 | 500 µs ? bound | 77 µs ✗ | 40 µs ✗ |
| parallel | counter-over | classical auto j1 | counter-over/64 |  | 82.99 s ? bound | 70 µs ✗ |
| parallel | counter-over | classical auto j1 | counter-over/8 | 549.9 ms ? bound | 116 µs ✗ | 55 µs ✗ |
| parallel | counter-over | classical auto j16 | counter-over/16 | > 120 s | 1.2 ms ✗ | 86 µs ✗ |
| parallel | counter-over | classical auto j16 | counter-over/2 | 446 µs ? bound | 510 µs ✗ | 68 µs ✗ |
| parallel | counter-over | classical auto j16 | counter-over/32 |  | 2.87 s ? bound | 95 µs ✗ |
| parallel | counter-over | classical auto j16 | counter-over/4 | 981 µs ? bound | 568 µs ✗ | 76 µs ✗ |
| parallel | counter-over | classical auto j16 | counter-over/64 |  | 103.02 s ? bound | 98 µs ✗ |
| parallel | counter-over | classical auto j16 | counter-over/8 | 323.2 ms ? bound | 759 µs ✗ | 73 µs ✗ |
| parallel | counter-over | classical auto j2 | counter-over/16 | > 120 s | 430 µs ✗ | 84 µs ✗ |
| parallel | counter-over | classical auto j2 | counter-over/2 | 102 µs ? bound | 93 µs ✗ | 72 µs ✗ |
| parallel | counter-over | classical auto j2 | counter-over/32 |  | 3.15 s ? bound | 96 µs ✗ |
| parallel | counter-over | classical auto j2 | counter-over/4 | 590 µs ? bound | 128 µs ✗ | 77 µs ✗ |
| parallel | counter-over | classical auto j2 | counter-over/64 |  | 112.73 s ? bound | 98 µs ✗ |
| parallel | counter-over | classical auto j2 | counter-over/8 | 502.6 ms ? bound | 165 µs ✗ | 76 µs ✗ |
| parallel | counter-over | classical auto j4 | counter-over/16 | > 120 s | 731 µs ✗ | 77 µs ✗ |
| parallel | counter-over | classical auto j4 | counter-over/2 | 153 µs ? bound | 150 µs ✗ | 72 µs ✗ |
| parallel | counter-over | classical auto j4 | counter-over/32 |  | 2.52 s ? bound | 94 µs ✗ |
| parallel | counter-over | classical auto j4 | counter-over/4 | 504 µs ? bound | 171 µs ✗ | 68 µs ✗ |
| parallel | counter-over | classical auto j4 | counter-over/64 |  | 86.58 s ? bound | 88 µs ✗ |
| parallel | counter-over | classical auto j4 | counter-over/8 | 389.9 ms ? bound | 274 µs ✗ | 73 µs ✗ |
| parallel | counter-over | classical auto j8 | counter-over/16 | > 120 s | 1.2 ms ✗ | 80 µs ✗ |
| parallel | counter-over | classical auto j8 | counter-over/2 | 216 µs ? bound | 240 µs ✗ | 78 µs ✗ |
| parallel | counter-over | classical auto j8 | counter-over/32 |  | 2.72 s ? bound | 94 µs ✗ |
| parallel | counter-over | classical auto j8 | counter-over/4 | 730 µs ? bound | 268 µs ✗ | 68 µs ✗ |
| parallel | counter-over | classical auto j8 | counter-over/64 |  | 95.97 s ? bound | 91 µs ✗ |
| parallel | counter-over | classical auto j8 | counter-over/8 | 355.2 ms ? bound | 454 µs ✗ | 87 µs ✗ |
| parallel | mix | mix auto j1 | mix/10 | > 120 s | > 120 s | 62.10 s ✗ |
| parallel | mix | mix auto j1 | mix/11 | > 120 s | > 120 s | > 120 s |
| parallel | mix | mix auto j1 | mix/8 | 1.80 s ✗ | 1.56 s ✗ | 857.8 ms ✗ |
| parallel | mix | mix auto j1 | mix/9 | 17.95 s ✗ | 13.94 s ✗ | 7.39 s ✗ |
| parallel | mix | mix auto j16 | mix/10 | > 120 s | > 120 s | > 120 s |
| parallel | mix | mix auto j16 | mix/11 | > 120 s | > 120 s | > 120 s |
| parallel | mix | mix auto j16 | mix/8 | 1.88 s ✗ | 2.43 s ✗ | 1.61 s ✗ |
| parallel | mix | mix auto j16 | mix/9 | 16.93 s ✗ | 21.18 s ✗ | 11.51 s ✗ |
| parallel | mix | mix auto j2 | mix/10 | > 120 s | > 120 s | 91.83 s ✗ |
| parallel | mix | mix auto j2 | mix/11 | > 120 s | > 120 s | > 120 s |
| parallel | mix | mix auto j2 | mix/8 | 1.86 s ✗ | 1.87 s ✗ | 965.2 ms ✗ |
| parallel | mix | mix auto j2 | mix/9 | 17.97 s ✗ | 17.40 s ✗ | 7.05 s ✗ |
| parallel | mix | mix auto j4 | mix/10 | > 120 s | > 120 s | 95.53 s ✗ |
| parallel | mix | mix auto j4 | mix/11 | > 120 s | > 120 s | > 120 s |
| parallel | mix | mix auto j4 | mix/8 | 1.64 s ✗ | 1.91 s ✗ | 1.05 s ✗ |
| parallel | mix | mix auto j4 | mix/9 | 16.72 s ✗ | 17.88 s ✗ | 7.90 s ✗ |
| parallel | mix | mix auto j8 | mix/10 | > 120 s | > 120 s | 109.66 s ✗ |
| parallel | mix | mix auto j8 | mix/11 | > 120 s | > 120 s | > 120 s |
| parallel | mix | mix auto j8 | mix/8 | 1.67 s ✗ | 2.07 s ✗ | 1.48 s ✗ |
| parallel | mix | mix auto j8 | mix/9 | 16.01 s ✗ | 18.57 s ✗ | 9.50 s ✗ |
| parallel | partition-no | classical auto j1 | partition-no/12 |  | 4.01 s ✗ | 3.59 s ✗ |
| parallel | partition-no | classical auto j1 | partition-no/14 |  | > 120 s | > 120 s |
| parallel | partition-no | classical auto j1 | partition-no/4 | 127.3 ms ✗ | 170 µs ✗ | 192 µs ✗ |
| parallel | partition-no | classical auto j1 | partition-no/5 | > 120 s | 604 µs ✗ | 568 µs ✗ |
| parallel | partition-no | classical auto j16 | partition-no/12 |  | 847.6 ms ✗ | 581.6 ms ✗ |
| parallel | partition-no | classical auto j16 | partition-no/14 |  | > 120 s | > 120 s |
| parallel | partition-no | classical auto j16 | partition-no/4 | 26.9 ms ✗ | 721 µs ✗ | 600 µs ✗ |
| parallel | partition-no | classical auto j16 | partition-no/5 | > 120 s | 840 µs ✗ | 850 µs ✗ |
| parallel | partition-no | classical auto j2 | partition-no/12 |  | 2.83 s ✗ | 2.13 s ✗ |
| parallel | partition-no | classical auto j2 | partition-no/14 |  | > 120 s | > 120 s |
| parallel | partition-no | classical auto j2 | partition-no/4 | 73.9 ms ✗ | 272 µs ✗ | 405 µs ✗ |
| parallel | partition-no | classical auto j2 | partition-no/5 | > 120 s | 681 µs ✗ | 745 µs ✗ |
| parallel | partition-no | classical auto j4 | partition-no/12 |  | 1.73 s ✗ | 1.24 s ✗ |
| parallel | partition-no | classical auto j4 | partition-no/14 |  | > 120 s | > 120 s |
| parallel | partition-no | classical auto j4 | partition-no/4 | 45.9 ms ✗ | 242 µs ✗ | 347 µs ✗ |
| parallel | partition-no | classical auto j4 | partition-no/5 | > 120 s | 518 µs ✗ | 578 µs ✗ |
| parallel | partition-no | classical auto j8 | partition-no/12 |  | 1.20 s ✗ | 806.0 ms ✗ |
| parallel | partition-no | classical auto j8 | partition-no/14 |  | > 120 s | > 120 s |
| parallel | partition-no | classical auto j8 | partition-no/4 | 29.4 ms ✗ | 392 µs ✗ | 418 µs ✗ |
| parallel | partition-no | classical auto j8 | partition-no/5 | > 120 s | 501 µs ✗ | 614 µs ✗ |
| parallel | partition-yes | classical auto j1 | partition-yes/20 |  | 535.5 ms ✓ | 396.5 ms ✓ |
| parallel | partition-yes | classical auto j1 | partition-yes/24 |  | 105.53 s ✓ | 67.45 s ✓ |
| parallel | partition-yes | classical auto j1 | partition-yes/5 | 191.6 ms ✓ | 97 µs ✓ | 117 µs ✓ |
| parallel | partition-yes | classical auto j1 | partition-yes/6 | 12.20 s ✓ | 167 µs ✓ | 187 µs ✓ |
| parallel | partition-yes | classical auto j1 | partition-yes/7 | > 120 s | 318 µs ✓ | 273 µs ✓ |
| parallel | partition-yes | classical auto j16 | partition-yes/20 |  | 651.0 ms ✓ | 445.7 ms ✓ |
| parallel | partition-yes | classical auto j16 | partition-yes/24 |  | > 120 s | 87.42 s ✓ |
| parallel | partition-yes | classical auto j16 | partition-yes/5 | 17.5 ms ✓ | 703 µs ✓ | 851 µs ✓ |
| parallel | partition-yes | classical auto j16 | partition-yes/6 | 1.33 s ✓ | 743 µs ✓ | 906 µs ✓ |
| parallel | partition-yes | classical auto j16 | partition-yes/7 | > 120 s | 849 µs ✓ | 750 µs ✓ |
| parallel | partition-yes | classical auto j2 | partition-yes/20 |  | 586.4 ms ✓ | 418.5 ms ✓ |
| parallel | partition-yes | classical auto j2 | partition-yes/24 |  | > 120 s | 70.81 s ✓ |
| parallel | partition-yes | classical auto j2 | partition-yes/5 | 192.0 ms ✓ | 207 µs ✓ | 311 µs ✓ |
| parallel | partition-yes | classical auto j2 | partition-yes/6 | 11.82 s ✓ | 280 µs ✓ | 461 µs ✓ |
| parallel | partition-yes | classical auto j2 | partition-yes/7 | > 120 s | 447 µs ✓ | 585 µs ✓ |
| parallel | partition-yes | classical auto j4 | partition-yes/20 |  | 608.0 ms ✓ | 429.9 ms ✓ |
| parallel | partition-yes | classical auto j4 | partition-yes/24 |  | > 120 s | 74.75 s ✓ |
| parallel | partition-yes | classical auto j4 | partition-yes/5 | 195.6 ms ✓ | 260 µs ✓ | 334 µs ✓ |
| parallel | partition-yes | classical auto j4 | partition-yes/6 | 8.70 s ✓ | 287 µs ✓ | 413 µs ✓ |
| parallel | partition-yes | classical auto j4 | partition-yes/7 | > 120 s | 498 µs ✓ | 621 µs ✓ |
| parallel | partition-yes | classical auto j8 | partition-yes/20 |  | 634.9 ms ✓ | 439.9 ms ✓ |
| parallel | partition-yes | classical auto j8 | partition-yes/24 |  | > 120 s | 79.07 s ✓ |
| parallel | partition-yes | classical auto j8 | partition-yes/5 | 30.5 ms ✓ | 399 µs ✓ | 461 µs ✓ |
| parallel | partition-yes | classical auto j8 | partition-yes/6 | 4.21 s ✓ | 450 µs ✓ | 546 µs ✓ |
| parallel | partition-yes | classical auto j8 | partition-yes/7 | > 120 s | 638 µs ✓ | 597 µs ✓ |
| parallel | qbf | classical auto j1 | qbf/16#0 | 8.28 s ✓ | 13.1 ms ✓ | 7.1 ms ✓ |
| parallel | qbf | classical auto j1 | qbf/16#1 | 1.34 s ✗ | 1.9 ms ✗ | 1.8 ms ✗ |
| parallel | qbf | classical auto j1 | qbf/16#2 | 1.89 s ✗ | 3.3 ms ✗ | 2.6 ms ✗ |
| parallel | qbf | classical auto j1 | qbf/16#3 | 5.99 s ✓ | 10.3 ms ✓ | 8.4 ms ✓ |
| parallel | qbf | classical auto j1 | qbf/20#0 | 78.42 s ✗ | 9.4 ms ✗ | 8.4 ms ✗ |
| parallel | qbf | classical auto j1 | qbf/20#1 | 109.45 s ✗ | 16.1 ms ✗ | 14.3 ms ✗ |
| parallel | qbf | classical auto j1 | qbf/20#2 | > 120 s | 60.2 ms ✓ | 53.3 ms ✓ |
| parallel | qbf | classical auto j1 | qbf/20#3 | 81.26 s ✗ | 9.2 ms ✗ | 8.1 ms ✗ |
| parallel | qbf | classical auto j1 | qbf/24#0 | > 120 s | 169.3 ms ✗ | 142.0 ms ✗ |
| parallel | qbf | classical auto j1 | qbf/24#1 | > 120 s | 62.6 ms ✗ | 68.0 ms ✗ |
| parallel | qbf | classical auto j1 | qbf/24#2 | > 120 s | 44.7 ms ✗ | 47.6 ms ✗ |
| parallel | qbf | classical auto j1 | qbf/24#3 | > 120 s | 62.2 ms ✗ | 57.9 ms ✗ |
| parallel | qbf | classical auto j1 | qbf/40#0 |  | 16.61 s ✗ | 13.75 s ✗ |
| parallel | qbf | classical auto j1 | qbf/40#1 |  | 18.54 s ✗ | 13.41 s ✗ |
| parallel | qbf | classical auto j1 | qbf/40#2 |  | 21.36 s ✗ | 15.46 s ✗ |
| parallel | qbf | classical auto j1 | qbf/40#3 |  | 16.91 s ✗ | 11.55 s ✗ |
| parallel | qbf | classical auto j1 | qbf/44#0 |  | 72.46 s ✗ | 50.45 s ✗ |
| parallel | qbf | classical auto j1 | qbf/44#1 |  | 72.68 s ✗ | 49.78 s ✗ |
| parallel | qbf | classical auto j1 | qbf/44#2 |  | 108.46 s ✗ | 73.42 s ✗ |
| parallel | qbf | classical auto j1 | qbf/44#3 |  | > 120 s | 26.33 s ? memory |
| parallel | qbf | classical auto j16 | qbf/16#0 | 6.33 s ✓ | 14.9 ms ✓ | 11.3 ms ✓ |
| parallel | qbf | classical auto j16 | qbf/16#1 | 709.2 ms ✗ | 2.4 ms ✗ | 2.0 ms ✗ |
| parallel | qbf | classical auto j16 | qbf/16#2 | 1.19 s ✗ | 3.4 ms ✗ | 3.5 ms ✗ |
| parallel | qbf | classical auto j16 | qbf/16#3 | 5.50 s ✓ | 13.2 ms ✓ | 9.5 ms ✓ |
| parallel | qbf | classical auto j16 | qbf/20#0 | 43.42 s ✗ | 7.8 ms ✗ | 6.7 ms ✗ |
| parallel | qbf | classical auto j16 | qbf/20#1 | 61.09 s ✗ | 12.4 ms ✗ | 13.7 ms ✗ |
| parallel | qbf | classical auto j16 | qbf/20#2 | > 120 s | 103.9 ms ✓ | 80.4 ms ✓ |
| parallel | qbf | classical auto j16 | qbf/20#3 | 41.57 s ✗ | 7.5 ms ✗ | 5.2 ms ✗ |
| parallel | qbf | classical auto j16 | qbf/24#0 | > 120 s | 214.4 ms ✗ | 166.2 ms ✗ |
| parallel | qbf | classical auto j16 | qbf/24#1 | > 120 s | 60.5 ms ✗ | 45.5 ms ✗ |
| parallel | qbf | classical auto j16 | qbf/24#2 | > 120 s | 40.2 ms ✗ | 36.5 ms ✗ |
| parallel | qbf | classical auto j16 | qbf/24#3 | > 120 s | 58.9 ms ✗ | 43.1 ms ✗ |
| parallel | qbf | classical auto j16 | qbf/40#0 |  | 14.33 s ✗ | 7.47 s ✗ |
| parallel | qbf | classical auto j16 | qbf/40#1 |  | 15.24 s ✗ | 5.53 s ? memory |
| parallel | qbf | classical auto j16 | qbf/40#2 |  | 17.72 s ✗ | 5.81 s ? memory |
| parallel | qbf | classical auto j16 | qbf/40#3 |  | 15.47 s ✗ | 7.51 s ✗ |
| parallel | qbf | classical auto j16 | qbf/44#0 |  | 54.98 s ✗ | 8.78 s ? memory |
| parallel | qbf | classical auto j16 | qbf/44#1 |  | 59.98 s ✗ | 7.42 s ? memory |
| parallel | qbf | classical auto j16 | qbf/44#2 |  | 96.80 s ✗ | 5.14 s ? memory |
| parallel | qbf | classical auto j16 | qbf/44#3 |  | > 120 s | 8.73 s ? memory |
| parallel | qbf | classical auto j2 | qbf/16#0 | 6.37 s ✓ | 14.7 ms ✓ | 10.4 ms ✓ |
| parallel | qbf | classical auto j2 | qbf/16#1 | 707.1 ms ✗ | 2.1 ms ✗ | 2.5 ms ✗ |
| parallel | qbf | classical auto j2 | qbf/16#2 | 1.21 s ✗ | 3.6 ms ✗ | 2.7 ms ✗ |
| parallel | qbf | classical auto j2 | qbf/16#3 | 5.53 s ✓ | 12.3 ms ✓ | 7.3 ms ✓ |
| parallel | qbf | classical auto j2 | qbf/20#0 | 43.03 s ✗ | 9.0 ms ✗ | 7.4 ms ✗ |
| parallel | qbf | classical auto j2 | qbf/20#1 | 61.36 s ✗ | 14.7 ms ✗ | 11.0 ms ✗ |
| parallel | qbf | classical auto j2 | qbf/20#2 | > 120 s | 103.0 ms ✓ | 64.0 ms ✓ |
| parallel | qbf | classical auto j2 | qbf/20#3 | 40.98 s ✗ | 7.7 ms ✗ | 7.8 ms ✗ |
| parallel | qbf | classical auto j2 | qbf/24#0 | > 120 s | 201.0 ms ✗ | 126.2 ms ✗ |
| parallel | qbf | classical auto j2 | qbf/24#1 | > 120 s | 58.6 ms ✗ | 37.2 ms ✗ |
| parallel | qbf | classical auto j2 | qbf/24#2 | > 120 s | 43.7 ms ✗ | 30.8 ms ✗ |
| parallel | qbf | classical auto j2 | qbf/24#3 | > 120 s | 58.2 ms ✗ | 39.2 ms ✗ |
| parallel | qbf | classical auto j2 | qbf/40#0 |  | 14.07 s ✗ | 8.61 s ✗ |
| parallel | qbf | classical auto j2 | qbf/40#1 |  | 15.30 s ✗ | 5.56 s ? memory |
| parallel | qbf | classical auto j2 | qbf/40#2 |  | 17.69 s ✗ | 5.80 s ? memory |
| parallel | qbf | classical auto j2 | qbf/40#3 |  | 15.63 s ✗ | 7.46 s ✗ |
| parallel | qbf | classical auto j2 | qbf/44#0 |  | 54.94 s ✗ | 8.79 s ? memory |
| parallel | qbf | classical auto j2 | qbf/44#1 |  | 60.47 s ✗ | 7.31 s ? memory |
| parallel | qbf | classical auto j2 | qbf/44#2 |  | 97.70 s ✗ | 5.14 s ? memory |
| parallel | qbf | classical auto j2 | qbf/44#3 |  | > 120 s | 8.56 s ? memory |
| parallel | qbf | classical auto j4 | qbf/16#0 | 6.26 s ✓ | 14.3 ms ✓ | 9.6 ms ✓ |
| parallel | qbf | classical auto j4 | qbf/16#1 | 709.6 ms ✗ | 1.7 ms ✗ | 1.6 ms ✗ |
| parallel | qbf | classical auto j4 | qbf/16#2 | 1.19 s ✗ | 3.7 ms ✗ | 2.9 ms ✗ |
| parallel | qbf | classical auto j4 | qbf/16#3 | 5.52 s ✓ | 12.3 ms ✓ | 8.5 ms ✓ |
| parallel | qbf | classical auto j4 | qbf/20#0 | 43.23 s ✗ | 8.7 ms ✗ | 7.5 ms ✗ |
| parallel | qbf | classical auto j4 | qbf/20#1 | 61.74 s ✗ | 15.0 ms ✗ | 10.0 ms ✗ |
| parallel | qbf | classical auto j4 | qbf/20#2 | > 120 s | 100.4 ms ✓ | 68.3 ms ✓ |
| parallel | qbf | classical auto j4 | qbf/20#3 | 41.17 s ✗ | 8.0 ms ✗ | 6.5 ms ✗ |
| parallel | qbf | classical auto j4 | qbf/24#0 | > 120 s | 194.5 ms ✗ | 128.5 ms ✗ |
| parallel | qbf | classical auto j4 | qbf/24#1 | > 120 s | 59.7 ms ✗ | 38.4 ms ✗ |
| parallel | qbf | classical auto j4 | qbf/24#2 | > 120 s | 37.6 ms ✗ | 29.7 ms ✗ |
| parallel | qbf | classical auto j4 | qbf/24#3 | > 120 s | 58.6 ms ✗ | 37.5 ms ✗ |
| parallel | qbf | classical auto j4 | qbf/40#0 |  | 13.95 s ✗ | 8.46 s ✗ |
| parallel | qbf | classical auto j4 | qbf/40#1 |  | 15.25 s ✗ | 5.53 s ? memory |
| parallel | qbf | classical auto j4 | qbf/40#2 |  | 17.64 s ✗ | 5.84 s ? memory |
| parallel | qbf | classical auto j4 | qbf/40#3 |  | 15.46 s ✗ | 7.57 s ✗ |
| parallel | qbf | classical auto j4 | qbf/44#0 |  | 55.96 s ✗ | 8.71 s ? memory |
| parallel | qbf | classical auto j4 | qbf/44#1 |  | 60.23 s ✗ | 7.44 s ? memory |
| parallel | qbf | classical auto j4 | qbf/44#2 |  | 97.99 s ✗ | 5.13 s ? memory |
| parallel | qbf | classical auto j4 | qbf/44#3 |  | > 120 s | 8.59 s ? memory |
| parallel | qbf | classical auto j8 | qbf/16#0 | 6.32 s ✓ | 14.0 ms ✓ | 10.3 ms ✓ |
| parallel | qbf | classical auto j8 | qbf/16#1 | 713.0 ms ✗ | 1.9 ms ✗ | 3.0 ms ✗ |
| parallel | qbf | classical auto j8 | qbf/16#2 | 1.21 s ✗ | 3.3 ms ✗ | 2.7 ms ✗ |
| parallel | qbf | classical auto j8 | qbf/16#3 | 5.76 s ✓ | 12.6 ms ✓ | 11.2 ms ✓ |
| parallel | qbf | classical auto j8 | qbf/20#0 | 43.89 s ✗ | 8.6 ms ✗ | 5.8 ms ✗ |
| parallel | qbf | classical auto j8 | qbf/20#1 | 61.97 s ✗ | 13.8 ms ✗ | 11.2 ms ✗ |
| parallel | qbf | classical auto j8 | qbf/20#2 | > 120 s | 100.1 ms ✓ | 71.9 ms ✓ |
| parallel | qbf | classical auto j8 | qbf/20#3 | 55.61 s ✗ | 7.0 ms ✗ | 8.6 ms ✗ |
| parallel | qbf | classical auto j8 | qbf/24#0 | > 120 s | 196.9 ms ✗ | 135.1 ms ✗ |
| parallel | qbf | classical auto j8 | qbf/24#1 | > 120 s | 58.5 ms ✗ | 43.4 ms ✗ |
| parallel | qbf | classical auto j8 | qbf/24#2 | > 120 s | 39.0 ms ✗ | 28.1 ms ✗ |
| parallel | qbf | classical auto j8 | qbf/24#3 | > 120 s | 58.8 ms ✗ | 38.6 ms ✗ |
| parallel | qbf | classical auto j8 | qbf/40#0 |  | 13.93 s ✗ | 7.84 s ✗ |
| parallel | qbf | classical auto j8 | qbf/40#1 |  | 15.32 s ✗ | 5.52 s ? memory |
| parallel | qbf | classical auto j8 | qbf/40#2 |  | 17.64 s ✗ | 5.82 s ? memory |
| parallel | qbf | classical auto j8 | qbf/40#3 |  | 15.53 s ✗ | 7.44 s ✗ |
| parallel | qbf | classical auto j8 | qbf/44#0 |  | 55.39 s ✗ | 8.78 s ? memory |
| parallel | qbf | classical auto j8 | qbf/44#1 |  | 59.90 s ✗ | 7.34 s ? memory |
| parallel | qbf | classical auto j8 | qbf/44#2 |  | 97.53 s ✗ | 5.14 s ? memory |
| parallel | qbf | classical auto j8 | qbf/44#3 |  | > 120 s | 8.70 s ? memory |
| parallel | wide-m3 | classical auto j1 | wide-m3/24 | 189.7 ms ✓ | 115 µs ✓ | 135 µs ✓ |
| parallel | wide-m3 | classical auto j1 | wide-m3/30 | 11.12 s ✓ | 152 µs ✓ | 182 µs ✓ |
| parallel | wide-m3 | classical auto j1 | wide-m3/36 | > 120 s | 182 µs ✓ | 192 µs ✓ |
| parallel | wide-m3 | classical auto j16 | wide-m3/24 | 131.0 ms ✓ | 666 µs ✓ | 714 µs ✓ |
| parallel | wide-m3 | classical auto j16 | wide-m3/30 | 8.24 s ✓ | 611 µs ✓ | 807 µs ✓ |
| parallel | wide-m3 | classical auto j16 | wide-m3/36 | > 120 s | 687 µs ✓ | 727 µs ✓ |
| parallel | wide-m3 | classical auto j2 | wide-m3/24 | 191.7 ms ✓ | 198 µs ✓ | 357 µs ✓ |
| parallel | wide-m3 | classical auto j2 | wide-m3/30 | 10.69 s ✓ | 269 µs ✓ | 316 µs ✓ |
| parallel | wide-m3 | classical auto j2 | wide-m3/36 | > 120 s | 274 µs ✓ | 415 µs ✓ |
| parallel | wide-m3 | classical auto j4 | wide-m3/24 | 149.5 ms ✓ | 254 µs ✓ | 315 µs ✓ |
| parallel | wide-m3 | classical auto j4 | wide-m3/30 | 8.34 s ✓ | 306 µs ✓ | 390 µs ✓ |
| parallel | wide-m3 | classical auto j4 | wide-m3/36 | > 120 s | 318 µs ✓ | 405 µs ✓ |
| parallel | wide-m3 | classical auto j8 | wide-m3/24 | 138.8 ms ✓ | 331 µs ✓ | 489 µs ✓ |
| parallel | wide-m3 | classical auto j8 | wide-m3/30 | 7.87 s ✓ | 424 µs ✓ | 503 µs ✓ |
| parallel | wide-m3 | classical auto j8 | wide-m3/36 | > 120 s | 424 µs ✓ | 453 µs ✓ |
| parallel | wide-m4 | classical auto j1 | wide-m4/28 | 2.97 s ✓ | 149 µs ✓ | 141 µs ✓ |
| parallel | wide-m4 | classical auto j1 | wide-m4/32 | 42.28 s ✓ | 179 µs ✓ | 186 µs ✓ |
| parallel | wide-m4 | classical auto j1 | wide-m4/36 | > 120 s | 175 µs ✓ | 190 µs ✓ |
| parallel | wide-m4 | classical auto j16 | wide-m4/28 | 1.98 s ✓ | 699 µs ✓ | 727 µs ✓ |
| parallel | wide-m4 | classical auto j16 | wide-m4/32 | 35.49 s ✓ | 620 µs ✓ | 807 µs ✓ |
| parallel | wide-m4 | classical auto j16 | wide-m4/36 | > 120 s | 638 µs ✓ | 722 µs ✓ |
| parallel | wide-m4 | classical auto j2 | wide-m4/28 | 2.92 s ✓ | 229 µs ✓ | 326 µs ✓ |
| parallel | wide-m4 | classical auto j2 | wide-m4/32 | 41.95 s ✓ | 266 µs ✓ | 369 µs ✓ |
| parallel | wide-m4 | classical auto j2 | wide-m4/36 | > 120 s | 270 µs ✓ | 430 µs ✓ |
| parallel | wide-m4 | classical auto j4 | wide-m4/28 | 2.19 s ✓ | 267 µs ✓ | 342 µs ✓ |
| parallel | wide-m4 | classical auto j4 | wide-m4/32 | 32.67 s ✓ | 284 µs ✓ | 368 µs ✓ |
| parallel | wide-m4 | classical auto j4 | wide-m4/36 | > 120 s | 329 µs ✓ | 435 µs ✓ |
| parallel | wide-m4 | classical auto j8 | wide-m4/28 | 2.12 s ✓ | 369 µs ✓ | 487 µs ✓ |
| parallel | wide-m4 | classical auto j8 | wide-m4/32 | 32.75 s ✓ | 393 µs ✓ | 538 µs ✓ |
| parallel | wide-m4 | classical auto j8 | wide-m4/36 | > 120 s | 412 µs ✓ | 535 µs ✓ |
| parallel-net | 3-partition-mll-no | classical net j1 | 3-partition-mll-no/4 | 1.59 s ✗ | 1.70 s ✗ | 1.63 s ✗ |
| parallel-net | 3-partition-mll-no | classical net j1 | 3-partition-mll-no/5 | 59.71 s ✗ | 51.56 s ✗ | 48.79 s ✗ |
| parallel-net | 3-partition-mll-no | classical net j1 | 3-partition-mll-no/6 | > 60 s | > 60 s | > 60 s |
| parallel-net | 3-partition-mll-no | classical net j16 | 3-partition-mll-no/4 | 142.0 ms ✗ | 139.7 ms ✗ | 138.2 ms ✗ |
| parallel-net | 3-partition-mll-no | classical net j16 | 3-partition-mll-no/5 | 4.60 s ✗ | 4.51 s ✗ | 4.42 s ✗ |
| parallel-net | 3-partition-mll-no | classical net j16 | 3-partition-mll-no/6 | > 60 s | > 60 s | > 60 s |
| parallel-net | 3-partition-mll-no | classical net j2 | 3-partition-mll-no/4 | 820.1 ms ✗ | 886.3 ms ✗ | 829.1 ms ✗ |
| parallel-net | 3-partition-mll-no | classical net j2 | 3-partition-mll-no/5 | 26.06 s ✗ | 26.63 s ✗ | 25.09 s ✗ |
| parallel-net | 3-partition-mll-no | classical net j2 | 3-partition-mll-no/6 | > 60 s | > 60 s | > 60 s |
| parallel-net | 3-partition-mll-no | classical net j4 | 3-partition-mll-no/4 | 437.4 ms ✗ | 463.9 ms ✗ | 443.0 ms ✗ |
| parallel-net | 3-partition-mll-no | classical net j4 | 3-partition-mll-no/5 | 13.98 s ✗ | 14.46 s ✗ | 13.41 s ✗ |
| parallel-net | 3-partition-mll-no | classical net j4 | 3-partition-mll-no/6 | > 60 s | > 60 s | > 60 s |
| parallel-net | 3-partition-mll-no | classical net j8 | 3-partition-mll-no/4 | 249.9 ms ✗ | 254.6 ms ✗ | 246.5 ms ✗ |
| parallel-net | 3-partition-mll-no | classical net j8 | 3-partition-mll-no/5 | 8.01 s ✗ | 7.92 s ✗ | 7.58 s ✗ |
| parallel-net | 3-partition-mll-no | classical net j8 | 3-partition-mll-no/6 | > 60 s | > 60 s | > 60 s |
| parallel-net | partition-table | classical net j1 | partition-table/1-1 | 39 µs ✓ | 40 µs ✓ | 72 µs ✓ |
| parallel-net | partition-table | classical net j1 | partition-table/1-1-1-1-1-7 | > 60 s | > 60 s | > 60 s |
| parallel-net | partition-table | classical net j1 | partition-table/1-1-1-5 | 7.40 s ✗ | 7.11 s ✗ | 7.13 s ✗ |
| parallel-net | partition-table | classical net j1 | partition-table/1-1-2-4 | 1.07 s ✓ | 1.04 s ✓ | 1.01 s ✓ |
| parallel-net | partition-table | classical net j1 | partition-table/1-1-4 | 51.5 ms ✗ | 47.5 ms ✗ | 46.5 ms ✗ |
| parallel-net | partition-table | classical net j1 | partition-table/1-2-3-4-5-5 | > 60 s | > 60 s | > 60 s |
| parallel-net | partition-table | classical net j1 | partition-table/1-2-5 | 4.09 s ✗ | 3.92 s ✗ | 3.66 s ✗ |
| parallel-net | partition-table | classical net j1 | partition-table/1-3 | 478 µs ✗ | 472 µs ✗ | 488 µs ✗ |
| parallel-net | partition-table | classical net j1 | partition-table/2-1-1 | 782 µs ✓ | 797 µs ✓ | 802 µs ✓ |
| parallel-net | partition-table | classical net j1 | partition-table/2-2-1-1 | 61.9 ms ✓ | 60.3 ms ✓ | 62.3 ms ✓ |
| parallel-net | partition-table | classical net j1 | partition-table/2-2-2-2-2-2-9-1 | > 60 s | > 60 s | > 60 s |
| parallel-net | partition-table | classical net j1 | partition-table/2-3-2-1 | 4.34 s ✓ | 4.38 s ✓ | 4.24 s ✓ |
| parallel-net | partition-table | classical net j1 | partition-table/3-3-3-1 | > 60 s | > 60 s | > 60 s |
| parallel-net | partition-table | classical net j16 | partition-table/1-1 | 482 µs ✓ | 505 µs ✓ | 508 µs ✓ |
| parallel-net | partition-table | classical net j16 | partition-table/1-1-1-1-1-7 | > 60 s | > 60 s | > 60 s |
| parallel-net | partition-table | classical net j16 | partition-table/1-1-1-5 | 611.3 ms ✗ | 598.5 ms ✗ | 649.9 ms ✗ |
| parallel-net | partition-table | classical net j16 | partition-table/1-1-2-4 | 76.5 ms ✓ | 72.5 ms ✓ | 74.5 ms ✓ |
| parallel-net | partition-table | classical net j16 | partition-table/1-1-4 | 5.2 ms ✗ | 5.3 ms ✗ | 4.7 ms ✗ |
| parallel-net | partition-table | classical net j16 | partition-table/1-2-3-4-5-5 | > 60 s | > 60 s | > 60 s |
| parallel-net | partition-table | classical net j16 | partition-table/1-2-5 | 309.5 ms ✗ | 297.9 ms ✗ | 291.2 ms ✗ |
| parallel-net | partition-table | classical net j16 | partition-table/1-3 | 1.6 ms ✗ | 1.6 ms ✗ | 965 µs ✗ |
| parallel-net | partition-table | classical net j16 | partition-table/2-1-1 | 4.4 ms ✓ | 4.4 ms ✓ | 1.4 ms ✓ |
| parallel-net | partition-table | classical net j16 | partition-table/2-2-1-1 | 7.7 ms ✓ | 7.1 ms ✓ | 6.0 ms ✓ |
| parallel-net | partition-table | classical net j16 | partition-table/2-2-2-2-2-2-9-1 | 955.4 ms ✓ | 835.8 ms ✓ | 977.2 ms ✓ |
| parallel-net | partition-table | classical net j16 | partition-table/2-3-2-1 | 337.5 ms ✓ | 316.1 ms ✓ | 336.8 ms ✓ |
| parallel-net | partition-table | classical net j16 | partition-table/3-3-3-1 | > 60 s | > 60 s | > 60 s |
| parallel-net | partition-table | classical net j2 | partition-table/1-1 | 203 µs ✓ | 245 µs ✓ | 180 µs ✓ |
| parallel-net | partition-table | classical net j2 | partition-table/1-1-1-1-1-7 | > 60 s | > 60 s | > 60 s |
| parallel-net | partition-table | classical net j2 | partition-table/1-1-1-5 | 3.91 s ✗ | 3.63 s ✗ | 3.49 s ✗ |
| parallel-net | partition-table | classical net j2 | partition-table/1-1-2-4 | 554.8 ms ✓ | 522.5 ms ✓ | 496.8 ms ✓ |
| parallel-net | partition-table | classical net j2 | partition-table/1-1-4 | 25.8 ms ✗ | 25.6 ms ✗ | 24.7 ms ✗ |
| parallel-net | partition-table | classical net j2 | partition-table/1-2-3-4-5-5 | > 60 s | > 60 s | > 60 s |
| parallel-net | partition-table | classical net j2 | partition-table/1-2-5 | 2.04 s ✗ | 1.92 s ✗ | 1.82 s ✗ |
| parallel-net | partition-table | classical net j2 | partition-table/1-3 | 557 µs ✗ | 554 µs ✗ | 584 µs ✗ |
| parallel-net | partition-table | classical net j2 | partition-table/2-1-1 | 930 µs ✓ | 931 µs ✓ | 726 µs ✓ |
| parallel-net | partition-table | classical net j2 | partition-table/2-2-1-1 | 31.6 ms ✓ | 30.7 ms ✓ | 30.6 ms ✓ |
| parallel-net | partition-table | classical net j2 | partition-table/2-2-2-2-2-2-9-1 | > 60 s | > 60 s | > 60 s |
| parallel-net | partition-table | classical net j2 | partition-table/2-3-2-1 | 2.27 s ✓ | 2.13 s ✓ | 2.18 s ✓ |
| parallel-net | partition-table | classical net j2 | partition-table/3-3-3-1 | > 60 s | > 60 s | > 60 s |
| parallel-net | partition-table | classical net j4 | partition-table/1-1 | 239 µs ✓ | 253 µs ✓ | 199 µs ✓ |
| parallel-net | partition-table | classical net j4 | partition-table/1-1-1-1-1-7 | > 60 s | > 60 s | > 60 s |
| parallel-net | partition-table | classical net j4 | partition-table/1-1-1-5 | 2.02 s ✗ | 1.92 s ✗ | 1.90 s ✗ |
| parallel-net | partition-table | classical net j4 | partition-table/1-1-2-4 | 281.9 ms ✓ | 265.7 ms ✓ | 251.8 ms ✓ |
| parallel-net | partition-table | classical net j4 | partition-table/1-1-4 | 13.5 ms ✗ | 13.3 ms ✗ | 12.9 ms ✗ |
| parallel-net | partition-table | classical net j4 | partition-table/1-2-3-4-5-5 | 3.5 ms ✓ | 3.4 ms ✓ | 2.0 ms ✓ |
| parallel-net | partition-table | classical net j4 | partition-table/1-2-5 | 1.02 s ✗ | 962.7 ms ✗ | 919.5 ms ✗ |
| parallel-net | partition-table | classical net j4 | partition-table/1-3 | 1.3 ms ✗ | 1.3 ms ✗ | 674 µs ✗ |
| parallel-net | partition-table | classical net j4 | partition-table/2-1-1 | 4.1 ms ✓ | 4.2 ms ✓ | 1.2 ms ✓ |
| parallel-net | partition-table | classical net j4 | partition-table/2-2-1-1 | 17.3 ms ✓ | 16.7 ms ✓ | 16.6 ms ✓ |
| parallel-net | partition-table | classical net j4 | partition-table/2-2-2-2-2-2-9-1 | > 60 s | > 60 s | > 60 s |
| parallel-net | partition-table | classical net j4 | partition-table/2-3-2-1 | 1.19 s ✓ | 1.06 s ✓ | 1.06 s ✓ |
| parallel-net | partition-table | classical net j4 | partition-table/3-3-3-1 | > 60 s | > 60 s | > 60 s |
| parallel-net | partition-table | classical net j8 | partition-table/1-1 | 384 µs ✓ | 312 µs ✓ | 346 µs ✓ |
| parallel-net | partition-table | classical net j8 | partition-table/1-1-1-1-1-7 | > 60 s | > 60 s | > 60 s |
| parallel-net | partition-table | classical net j8 | partition-table/1-1-1-5 | 1.11 s ✗ | 1.08 s ✗ | 1.08 s ✗ |
| parallel-net | partition-table | classical net j8 | partition-table/1-1-2-4 | 149.5 ms ✓ | 143.8 ms ✓ | 141.7 ms ✓ |
| parallel-net | partition-table | classical net j8 | partition-table/1-1-4 | 8.4 ms ✗ | 7.9 ms ✗ | 7.6 ms ✗ |
| parallel-net | partition-table | classical net j8 | partition-table/1-2-3-4-5-5 | > 60 s | > 60 s | > 60 s |
| parallel-net | partition-table | classical net j8 | partition-table/1-2-5 | 582.3 ms ✗ | 545.7 ms ✗ | 537.3 ms ✗ |
| parallel-net | partition-table | classical net j8 | partition-table/1-3 | 1.4 ms ✗ | 1.4 ms ✗ | 724 µs ✗ |
| parallel-net | partition-table | classical net j8 | partition-table/2-1-1 | 4.3 ms ✓ | 4.3 ms ✓ | 1.3 ms ✓ |
| parallel-net | partition-table | classical net j8 | partition-table/2-2-1-1 | 9.3 ms ✓ | 9.4 ms ✓ | 8.7 ms ✓ |
| parallel-net | partition-table | classical net j8 | partition-table/2-2-2-2-2-2-9-1 | 861.2 ms ✓ | 897.2 ms ✓ | 815.1 ms ✓ |
| parallel-net | partition-table | classical net j8 | partition-table/2-3-2-1 | 654.3 ms ✓ | 611.1 ms ✓ | 626.7 ms ✓ |
| parallel-net | partition-table | classical net j8 | partition-table/3-3-3-1 | > 60 s | > 60 s | > 60 s |
| period-1 | 3-partition-mll-no | classical net j1 period 1 | 3-partition-mll-no/4 | 1.80 s ✗ | 1.85 s ✗ | 1.73 s ✗ |
| period-1 | 3-partition-mll-no | classical net j1 period 1 | 3-partition-mll-no/5 | 56.27 s ✗ | 56.38 s ✗ | 55.13 s ✗ |
| period-1 | partition-no | classical net j1 period 1 | partition-no/3 | 3.49 s ✗ | 3.49 s ✗ | 3.32 s ✗ |
| period-1 | partition-no | classical net j1 period 1 | partition-no/4 | > 60 s | > 60 s | > 60 s |
| period-1 | partition-yes | classical net j1 period 1 | partition-yes/4 | 2.76 s ✓ | 2.75 s ✓ | 2.69 s ✓ |
| period-1 | wide-m1 | classical net j1 period 1 | wide-m1/2048 | 1.35 s ✓ | 1.35 s ✓ | 1.22 s ✓ |
| period-1 | wide-m1 | classical net j1 period 1 | wide-m1/256 | 19.2 ms ✓ | 19.5 ms ✓ | 17.9 ms ✓ |
| period-1 | wide-m2 | classical net j1 period 1 | wide-m2/2048 | 1.35 s ✓ | 1.38 s ✓ | 1.23 s ✓ |
| period-1 | wide-m2 | classical net j1 period 1 | wide-m2/256 | 19.3 ms ✓ | 19.6 ms ✓ | 18.5 ms ✓ |
| period-16 | 3-partition-mll-no | classical net j1 period 16 | 3-partition-mll-no/4 | 22.04 s ✗ | 20.98 s ✗ | 20.95 s ✗ |
| period-16 | 3-partition-mll-no | classical net j1 period 16 | 3-partition-mll-no/5 | > 60 s | > 60 s | > 60 s |
| period-16 | partition-no | classical net j1 period 16 | partition-no/3 | > 60 s | > 60 s | > 60 s |
| period-16 | partition-no | classical net j1 period 16 | partition-no/4 | > 60 s | > 60 s | > 60 s |
| period-16 | partition-yes | classical net j1 period 16 | partition-yes/4 | > 60 s | > 60 s | > 60 s |
| period-16 | wide-m1 | classical net j1 period 16 | wide-m1/2048 | 459.5 ms ✓ | 434.9 ms ✓ | 388.2 ms ✓ |
| period-16 | wide-m1 | classical net j1 period 16 | wide-m1/256 | 6.5 ms ✓ | 6.2 ms ✓ | 5.9 ms ✓ |
| period-16 | wide-m2 | classical net j1 period 16 | wide-m2/2048 | 442.5 ms ✓ | 429.6 ms ✓ | 405.9 ms ✓ |
| period-16 | wide-m2 | classical net j1 period 16 | wide-m2/256 | 6.6 ms ✓ | 6.6 ms ✓ | 6.2 ms ✓ |
| period-2 | 3-partition-mll-no | classical net j1 period 2 | 3-partition-mll-no/4 | 1.43 s ✗ | 1.45 s ✗ | 1.38 s ✗ |
| period-2 | 3-partition-mll-no | classical net j1 period 2 | 3-partition-mll-no/5 | 51.40 s ✗ | 51.04 s ✗ | 49.10 s ✗ |
| period-2 | partition-no | classical net j1 period 2 | partition-no/3 | 8.11 s ✗ | 7.82 s ✗ | 7.45 s ✗ |
| period-2 | partition-no | classical net j1 period 2 | partition-no/4 | > 60 s | > 60 s | > 60 s |
| period-2 | partition-yes | classical net j1 period 2 | partition-yes/4 | 6.11 s ✓ | 6.00 s ✓ | 5.72 s ✓ |
| period-2 | wide-m1 | classical net j1 period 2 | wide-m1/2048 | 856.4 ms ✓ | 860.9 ms ✓ | 818.1 ms ✓ |
| period-2 | wide-m1 | classical net j1 period 2 | wide-m1/256 | 12.2 ms ✓ | 12.4 ms ✓ | 12.2 ms ✓ |
| period-2 | wide-m2 | classical net j1 period 2 | wide-m2/2048 | 840.4 ms ✓ | 855.4 ms ✓ | 803.6 ms ✓ |
| period-2 | wide-m2 | classical net j1 period 2 | wide-m2/256 | 12.4 ms ✓ | 12.4 ms ✓ | 12.0 ms ✓ |
| period-8 | 3-partition-mll-no | classical net j1 period 8 | 3-partition-mll-no/4 | 5.01 s ✗ | 4.96 s ✗ | 4.78 s ✗ |
| period-8 | 3-partition-mll-no | classical net j1 period 8 | 3-partition-mll-no/5 | > 60 s | > 60 s | > 60 s |
| period-8 | partition-no | classical net j1 period 8 | partition-no/3 | > 60 s | > 60 s | > 60 s |
| period-8 | partition-no | classical net j1 period 8 | partition-no/4 | > 60 s | > 60 s | > 60 s |
| period-8 | partition-yes | classical net j1 period 8 | partition-yes/4 | > 60 s | > 60 s | > 60 s |
| period-8 | wide-m1 | classical net j1 period 8 | wide-m1/2048 | 520.8 ms ✓ | 494.5 ms ✓ | 448.2 ms ✓ |
| period-8 | wide-m1 | classical net j1 period 8 | wide-m1/256 | 7.3 ms ✓ | 7.2 ms ✓ | 7.0 ms ✓ |
| period-8 | wide-m2 | classical net j1 period 8 | wide-m2/2048 | 503.9 ms ✓ | 491.0 ms ✓ | 463.3 ms ✓ |
| period-8 | wide-m2 | classical net j1 period 8 | wide-m2/256 | 7.6 ms ✓ | 7.2 ms ✓ | 7.0 ms ✓ |

## Against 2026-10-02

### Verdicts that differ

None.

### Counters that differ

Of the 8501 problems both decide alike on one thread, 6003 differ in nodes, splits, memo_hits, memo_entries, links, tests.

### Decided by before, not by after

| file | family | configuration | problem | before | after | lltp-default |
|---|---|---|---|--:|--:|--:|
| families | qbf | classical auto j1 | qbf/44#3 | 218.72 s ✗ | 28.71 s ? memory |  |
| parallel | qbf | classical auto j16 | qbf/40#1 | 15.24 s ✗ | 5.53 s ? memory |  |
| parallel | qbf | classical auto j16 | qbf/40#2 | 17.72 s ✗ | 5.81 s ? memory |  |
| parallel | qbf | classical auto j16 | qbf/44#0 | 54.98 s ✗ | 8.78 s ? memory |  |
| parallel | qbf | classical auto j16 | qbf/44#1 | 59.98 s ✗ | 7.42 s ? memory |  |
| parallel | qbf | classical auto j16 | qbf/44#2 | 96.80 s ✗ | 5.14 s ? memory |  |
| parallel | qbf | classical auto j2 | qbf/40#1 | 15.30 s ✗ | 5.56 s ? memory |  |
| parallel | qbf | classical auto j2 | qbf/40#2 | 17.69 s ✗ | 5.80 s ? memory |  |
| parallel | qbf | classical auto j2 | qbf/44#0 | 54.94 s ✗ | 8.79 s ? memory |  |
| parallel | qbf | classical auto j2 | qbf/44#1 | 60.47 s ✗ | 7.31 s ? memory |  |
| parallel | qbf | classical auto j2 | qbf/44#2 | 97.70 s ✗ | 5.14 s ? memory |  |
| parallel | qbf | classical auto j4 | qbf/40#1 | 15.25 s ✗ | 5.53 s ? memory |  |
| parallel | qbf | classical auto j4 | qbf/40#2 | 17.64 s ✗ | 5.84 s ? memory |  |
| parallel | qbf | classical auto j4 | qbf/44#0 | 55.96 s ✗ | 8.71 s ? memory |  |
| parallel | qbf | classical auto j4 | qbf/44#1 | 60.23 s ✗ | 7.44 s ? memory |  |
| parallel | qbf | classical auto j4 | qbf/44#2 | 97.99 s ✗ | 5.13 s ? memory |  |
| parallel | qbf | classical auto j8 | qbf/40#1 | 15.32 s ✗ | 5.52 s ? memory |  |
| parallel | qbf | classical auto j8 | qbf/40#2 | 17.64 s ✗ | 5.82 s ? memory |  |
| parallel | qbf | classical auto j8 | qbf/44#0 | 55.39 s ✗ | 8.78 s ? memory |  |
| parallel | qbf | classical auto j8 | qbf/44#1 | 59.90 s ✗ | 7.34 s ? memory |  |
| parallel | qbf | classical auto j8 | qbf/44#2 | 97.53 s ✗ | 5.14 s ? memory |  |

### By group

Problems in both; decided (late: after the time limit); decided by one and not the other; where the undecided ended (`time` the time limit or `killed` past it, `bound` the copy bound, `depth` the recursion limit, `memory` the memory limit, `index` an index past its width, `wide` a context too wide to split, `crash` out of memory); the median ratio of after's time to before's on the problems both decide within the limit (how many); and the problems only after has.

| file | family | configuration | problems | before decided | after decided | only before | only after | before ended | after ended | after/before | new |
|---|---|---|--:|--:|--:|--:|--:|---|---|--:|--:|
| engines | 3-partition-mll-no | classical focus j1 | 4 | 4 | 4 | 0 | 0 | – | – | 2.05× (4) | 0 |
| engines | 3-partition-mll-no | classical net j1 | 4 | 2 | 2 | 0 | 0 | 2 time | 2 time | 0.98× (2) | 0 |
| engines | 3-partition-mll-yes | classical focus j1 | 4 | 4 | 4 | 0 | 0 | – | – | 1.36× (4) | 0 |
| engines | 3-partition-mll-yes | classical net j1 | 4 | 3 | 4 | 0 | 1 | 1 time | – | 7.5e-4× (3) | 0 |
| engines | additive | classical focus j1 | 4 | 3 | 4 | 0 | 1 | 1 crash | – | 0.12× (3) | 0 |
| engines | cancellation | classical focus j1 | 1 | 1 | 1 | 0 | 0 | – | – | 0.94× (1) | 0 |
| engines | partition-no | classical focus j1 | 4 | 4 | 4 | 0 | 0 | – | – | 1.09× (4) | 0 |
| engines | partition-no | classical net j1 | 4 | 1 | 1 | 0 | 0 | 3 time | 3 time | 1.03× (1) | 0 |
| engines | partition-table | classical focus j1 | 13 | 13 | 13 | 0 | 0 | – | – | 1.32× (13) | 0 |
| engines | partition-table | classical net j1 | 13 | 9 | 9 | 0 | 0 | 4 time | 4 time | 0.98× (9) | 0 |
| engines | partition-yes | classical focus j1 | 5 | 5 | 5 | 0 | 0 | – | – | 1.08× (5) | 0 |
| engines | partition-yes | classical net j1 | 5 | 1 | 1 | 0 | 0 | 4 time | 4 time | 1.00× (1) | 0 |
| engines | wide-m1 | classical focus j1 | 6 | 5 | 6 | 0 | 1 | 1 depth | – | 0.97× (5) | 0 |
| engines | wide-m1 | classical net j1 | 6 | 6 | 6 | 0 | 0 | – | – | 1.02× (6) | 0 |
| engines | wide-m2 | classical focus j1 | 6 | 5 | 6 | 0 | 1 | 1 depth | – | 1.05× (5) | 0 |
| engines | wide-m2 | classical net j1 | 6 | 6 | 6 | 0 | 0 | – | – | 1.00× (6) | 0 |
| engines | wide-m3 | classical focus j1 | 5 | 4 | 5 | 0 | 1 | 1 depth | – | 1.02× (4) | 0 |
| engines | wide-m3 | classical net j1 | 5 | 5 | 5 | 0 | 0 | – | – | 0.99× (5) | 0 |
| engines | wide-m4 | classical focus j1 | 5 | 4 | 5 | 0 | 1 | 1 depth | – | 1.01× (4) | 0 |
| engines | wide-m4 | classical net j1 | 5 | 5 | 5 | 0 | 0 | – | – | 0.97× (5) | 0 |
| families | 3-partition-mll-no | classical auto j1 | 4 | 4 | 4 | 0 | 0 | – | – | 2.25× (4) | 0 |
| families | 3-partition-mll-yes | classical auto j1 | 4 | 4 | 4 | 0 | 0 | – | – | 1.44× (4) | 0 |
| families | 3-partition-no | classical auto j1 | 2 | 2 | 2 | 0 | 0 | – | – | 1.07× (2) | 0 |
| families | 3-partition-yes | classical auto j1 | 4 | 4 | 4 | 0 | 0 | – | – | 1.43× (4) | 0 |
| families | additive | classical auto j1 | 4 | 4 | 4 | 0 | 0 | – | – | 0.87× (4) | 0 |
| families | chain | classical auto j1 | 4 | 4 | 4 | 0 | 0 | – | – | 0.86× (4) | 0 |
| families | counter | classical auto j1 | 6 | 6 | 6 | 0 | 0 | – | – | 0.39× (6) | 0 |
| families | counter-over | classical auto j1 | 6 | 4 | 6 | 0 | 2 | 2 bound | – | 0.44× (4) | 0 |
| families | growing | classical auto j1 | 4 | 0 | 0 | 0 | 0 | 3 bound, 1 depth | 3 bound, 1 depth | – | 0 |
| families | mix | mix auto j1 | 6 | 5 | 5 | 0 | 0 | 1 time | 1 time | 0.55× (5) | 0 |
| families | partition-no | classical auto j1 | 7 | 5 | 5 | 0 | 0 | 2 time | 2 time | 1.01× (5) | 0 |
| families | partition-yes | classical auto j1 | 9 | 8 | 8 | 0 | 0 | 1 time | 1 time | 0.99× (8) | 0 |
| families | qbf | classical auto j1 | 36 | 34 | 34 | 1 | 1 | 1 time, 1 crash | 2 memory | 0.69× (33) | 0 |
| families | wide-m1 | classical auto j1 | 6 | 6 | 6 | 0 | 0 | – | – | 1.05× (6) | 0 |
| families | wide-m2 | classical auto j1 | 6 | 6 | 6 | 0 | 0 | – | – | 1.04× (6) | 0 |
| families | wide-m3 | classical auto j1 | 7 | 6 | 7 | 0 | 1 | 1 depth | – | 0.93× (6) | 0 |
| families | wide-m4 | classical auto j1 | 8 | 7 | 8 | 0 | 1 | 1 depth | – | 0.93× (7) | 0 |
| intuitionistic | 3-partition-no | intuitionistic auto j1 | 1 | 1 | 1 | 0 | 0 | – | – | 0.93× (1) | 0 |
| intuitionistic | 3-partition-yes | intuitionistic auto j1 | 4 | 4 | 4 | 0 | 0 | – | – | 1.28× (4) | 0 |
| intuitionistic | chain | intuitionistic auto j1 | 4 | 4 | 4 | 0 | 0 | – | – | 0.85× (4) | 0 |
| intuitionistic | counter | intuitionistic auto j1 | 6 | 6 | 6 | 0 | 0 | – | – | 0.36× (6) | 0 |
| intuitionistic | counter-over | intuitionistic auto j1 | 6 | 4 | 6 | 0 | 2 | 2 bound | – | 0.39× (4) | 0 |
| lltp-all-cores | ILL/ILLTP-SYJ-01 | intuitionistic auto j16 | 31 | 0 | 0 | 0 | 0 | 13 bound, 12 depth, 2 killed, 4 crash | 15 bound, 12 depth, 4 memory | – | 0 |
| lltp-all-cores | ILL/ILLTP-SYJ-cbn | intuitionistic auto j16 | 31 | 0 | 0 | 0 | 0 | 16 bound, 12 depth, 2 killed, 1 crash | 16 bound, 12 depth, 3 memory | – | 0 |
| lltp-all-cores | ILL/ILLTP-SYJ-cbv | intuitionistic auto j16 | 31 | 2 | 2 | 0 | 0 | 10 time, 19 killed | 2 time, 6 bound, 21 memory | 0.94× (2) | 0 |
| lltp-all-cores | ILL/petri-nets/MCC | intuitionistic auto j16 | 1009 | 623 (2 late) | 991 | 0 | 368 | 386 time | 17 time, 1 memory | 4.1e-2× (621) | 0 |
| lltp-all-cores-generous | ILL/ILLTP-SYJ-01 | intuitionistic auto j16 | 5 | 0 | 0 | 0 | 0 | 4 time, 1 bound | 1 bound, 4 memory | – | 0 |
| lltp-all-cores-generous | ILL/ILLTP-SYJ-cbn | intuitionistic auto j16 | 4 | 0 | 0 | 0 | 0 | 2 time, 2 bound | 1 bound, 3 memory | – | 0 |
| lltp-all-cores-generous | ILL/ILLTP-SYJ-cbv | intuitionistic auto j16 | 3 | 0 | 0 | 0 | 0 | 3 time | 3 memory | – | 0 |
| lltp-classical | CLL/Non-theorems | classical auto j1 | 3 | 3 | 3 | 0 | 0 | – | – | 3.35× (3) | 0 |
| lltp-classical | CLL/misc | classical auto j1 | 14 | 8 | 8 | 0 | 0 | 6 bound | 6 bound | 3.75× (8) | 0 |
| lltp-classical | ILL/ILLTP-LCL-01 | classical auto j1 | 2 | 0 | 0 | 0 | 0 | 2 bound | 2 bound | – | 0 |
| lltp-classical | ILL/ILLTP-LCL-cbn | classical auto j1 | 2 | 2 | 2 | 0 | 0 | – | – | 2.48× (2) | 0 |
| lltp-classical | ILL/ILLTP-LCL-cbv | classical auto j1 | 2 | 0 | 0 | 0 | 0 | 2 bound | 2 bound | – | 0 |
| lltp-classical | ILL/ILLTP-SYJ-01 | classical auto j1 | 252 | 11 | 11 | 0 | 0 | 7 time, 209 bound, 22 depth, 3 killed | 2 time, 215 bound, 22 depth, 2 memory | 2.30× (11) | 0 |
| lltp-classical | ILL/ILLTP-SYJ-cbn | classical auto j1 | 252 | 35 | 35 | 0 | 0 | 8 time, 185 bound, 22 depth, 2 killed | 3 time, 190 bound, 22 depth, 2 memory | 1.96× (35) | 0 |
| lltp-classical | ILL/ILLTP-SYJ-cbv | classical auto j1 | 252 | 39 | 40 | 0 | 1 | 27 time, 173 bound, 10 depth, 3 killed | 22 time, 178 bound, 10 depth, 2 memory | 1.68× (39) | 0 |
| lltp-classical | ILL/ILLTP-SYN-01 | classical auto j1 | 19 | 11 | 13 | 0 | 2 | 8 bound | 6 bound | 2.19× (11) | 0 |
| lltp-classical | ILL/ILLTP-SYN-cbn | classical auto j1 | 19 | 16 | 18 | 0 | 2 | 3 bound | 1 bound | 2.20× (16) | 0 |
| lltp-classical | ILL/ILLTP-SYN-cbv | classical auto j1 | 19 | 16 | 18 | 0 | 2 | 3 bound | 1 bound | 1.95× (16) | 0 |
| lltp-classical | ILL/KLE-01 | classical auto j1 | 88 | 50 | 51 | 0 | 1 | 38 bound | 37 bound | 1.57× (50) | 0 |
| lltp-classical | ILL/KLE-IMP-CONJ | classical auto j1 | 222 | 162 | 162 | 0 | 0 | 60 bound | 60 bound | 1.76× (162) | 0 |
| lltp-classical | ILL/KLE-IMP-CONJ/ALT | classical auto j1 | 27 | 27 | 27 | 0 | 0 | – | – | 2.00× (27) | 0 |
| lltp-classical | ILL/KLE-IMP-CONJ/NON-THEOREMS | classical auto j1 | 22 | 22 | 22 | 0 | 0 | – | – | 8.00× (22) | 0 |
| lltp-classical | ILL/KLE-cbn | classical auto j1 | 88 | 68 | 69 | 0 | 1 | 20 bound | 19 bound | 1.81× (68) | 0 |
| lltp-classical | ILL/KLE-cbv | classical auto j1 | 88 | 67 | 68 | 0 | 1 | 21 bound | 20 bound | 1.59× (67) | 0 |
| lltp-classical | ILL/Non-theorems | classical auto j1 | 1 | 0 | 0 | 0 | 0 | 1 bound | 1 bound | – | 0 |
| lltp-classical | ILL/misc | classical auto j1 | 3 | 3 | 3 | 0 | 0 | – | – | 2.04× (3) | 0 |
| lltp-classical | ILL/petri-nets/MCC | classical auto j1 | 3137 | 1468 (2 late) | 3071 | 0 | 1603 | 1646 time, 2 bound, 7 killed, 14 crash | 58 time, 8 memory | 0.29× (1466) | 0 |
| lltp-classical-generous | ILL/ILLTP-SYJ-01 | classical auto j1 | 4 | 0 | 0 | 0 | 0 | 4 time | 1 time, 1 bound, 2 memory | – | 0 |
| lltp-classical-generous | ILL/ILLTP-SYJ-cbn | classical auto j1 | 2 | 0 | 0 | 0 | 0 | 2 time | 2 memory | – | 0 |
| lltp-classical-generous | ILL/ILLTP-SYJ-cbv | classical auto j1 | 3 | 0 | 0 | 0 | 0 | 3 time | 1 time, 2 memory | – | 0 |
| lltp-classical-generous | ILL/petri-nets/MCC | classical auto j1 | 25 | 15 | 25 | 0 | 10 | 10 time | – | 0.60× (15) | 0 |
| lltp-copies-10 | ILL/ILLTP-LCL-01 | intuitionistic auto j1 | 2 | 1 | 2 | 0 | 1 | 1 bound | – | 1.44× (1) | 0 |
| lltp-copies-10 | ILL/ILLTP-LCL-cbn | intuitionistic auto j1 | 2 | 2 | 2 | 0 | 0 | – | – | 2.05× (2) | 0 |
| lltp-copies-10 | ILL/ILLTP-LCL-cbv | intuitionistic auto j1 | 2 | 1 | 2 | 0 | 1 | 1 bound | – | 1.42× (1) | 0 |
| lltp-copies-10 | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | 209 | 37 | 51 | 0 | 14 | 107 time, 65 bound | 85 time, 73 bound | 0.51× (37) | 0 |
| lltp-copies-10 | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | 209 | 75 | 88 | 0 | 13 | 88 time, 46 bound | 78 time, 43 bound | 1.22× (75) | 0 |
| lltp-copies-10 | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | 209 | 83 | 93 | 0 | 10 | 98 time, 28 bound | 79 time, 37 bound | 0.71× (83) | 0 |
| lltp-copies-10 | ILL/ILLTP-SYN-01 | intuitionistic auto j1 | 8 | 5 | 7 | 0 | 2 | 3 bound | 1 bound | 1.04× (5) | 0 |
| lltp-copies-10 | ILL/ILLTP-SYN-cbn | intuitionistic auto j1 | 8 | 5 | 8 | 0 | 3 | 3 bound | – | 1.48× (5) | 0 |
| lltp-copies-10 | ILL/ILLTP-SYN-cbv | intuitionistic auto j1 | 8 | 5 | 8 | 0 | 3 | 3 bound | – | 1.35× (5) | 0 |
| lltp-copies-10 | ILL/KLE-01 | intuitionistic auto j1 | 38 | 29 | 31 | 0 | 2 | 9 bound | 7 bound | 0.99× (29) | 0 |
| lltp-copies-10 | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | 60 | 60 | 60 | 0 | 0 | – | – | 1.14× (60) | 0 |
| lltp-copies-10 | ILL/KLE-cbn | intuitionistic auto j1 | 38 | 34 | 36 | 0 | 2 | 4 bound | 2 bound | 1.55× (34) | 0 |
| lltp-copies-10 | ILL/KLE-cbv | intuitionistic auto j1 | 38 | 32 | 33 | 0 | 1 | 6 bound | 5 bound | 1.27× (32) | 0 |
| lltp-copies-10 | ILL/Non-theorems | intuitionistic auto j1 | 1 | 0 | 0 | 0 | 0 | 1 bound | 1 bound | – | 0 |
| lltp-copies-10 | ILL/petri-nets/MCC | intuitionistic auto j1 | 171 | 89 | 171 | 0 | 82 | 80 time, 2 bound | – | 1.6e-2× (89) | 0 |
| lltp-forward | ILL/ILLTP-LCL-01 | intuitionistic auto j1 bias factors | 2 | 1 | 2 | 0 | 1 | 1 bound | – | 1.33× (1) | 0 |
| lltp-forward | ILL/ILLTP-LCL-cbn | intuitionistic auto j1 bias factors | 2 | 2 | 2 | 0 | 0 | – | – | 2.05× (2) | 0 |
| lltp-forward | ILL/ILLTP-LCL-cbv | intuitionistic auto j1 bias factors | 2 | 1 | 2 | 0 | 1 | 1 bound | – | 1.31× (1) | 0 |
| lltp-forward | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 bias factors | 252 | 53 | 99 | 0 | 46 | 135 time, 39 bound, 22 depth, 3 killed | 124 time, 4 bound, 22 depth, 3 memory | 0.68× (53) | 0 |
| lltp-forward | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 bias factors | 252 | 98 | 142 | 0 | 44 | 100 time, 30 bound, 22 depth, 2 killed | 87 time, 22 depth, 1 memory | 1.12× (98) | 0 |
| lltp-forward | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 bias factors | 252 | 94 | 118 | 0 | 24 | 126 time, 19 bound, 10 depth, 3 killed | 118 time, 4 bound, 10 depth, 2 memory | 0.91× (94) | 0 |
| lltp-forward | ILL/ILLTP-SYN-01 | intuitionistic auto j1 bias factors | 19 | 16 | 19 | 0 | 3 | 3 bound | – | 1.80× (16) | 0 |
| lltp-forward | ILL/ILLTP-SYN-cbn | intuitionistic auto j1 bias factors | 19 | 16 | 19 | 0 | 3 | 3 bound | – | 2.27× (16) | 0 |
| lltp-forward | ILL/ILLTP-SYN-cbv | intuitionistic auto j1 bias factors | 19 | 16 | 19 | 0 | 3 | 3 bound | – | 1.83× (16) | 0 |
| lltp-forward | ILL/KLE-01 | intuitionistic auto j1 bias factors | 88 | 80 | 86 | 0 | 6 | 1 time, 7 bound | 2 bound | 1.32× (80) | 0 |
| lltp-forward | ILL/KLE-IMP-CONJ | intuitionistic auto j1 bias factors | 222 | 222 | 222 | 0 | 0 | – | – | 1.46× (222) | 0 |
| lltp-forward | ILL/KLE-IMP-CONJ/ALT | intuitionistic auto j1 bias factors | 27 | 27 | 27 | 0 | 0 | – | – | 1.68× (27) | 0 |
| lltp-forward | ILL/KLE-IMP-CONJ/NON-THEOREMS | intuitionistic auto j1 bias factors | 22 | 22 | 22 | 0 | 0 | – | – | 5.00× (22) | 0 |
| lltp-forward | ILL/KLE-cbn | intuitionistic auto j1 bias factors | 88 | 84 | 87 | 0 | 3 | 4 bound | 1 bound | 1.67× (84) | 0 |
| lltp-forward | ILL/KLE-cbv | intuitionistic auto j1 bias factors | 88 | 82 | 86 | 0 | 4 | 6 bound | 2 bound | 1.53× (82) | 0 |
| lltp-forward | ILL/Non-theorems | intuitionistic auto j1 bias factors | 1 | 0 | 0 | 0 | 0 | 1 bound | 1 bound | – | 0 |
| lltp-forward | ILL/misc | intuitionistic auto j1 bias factors | 3 | 3 | 3 | 0 | 0 | – | – | 1.65× (3) | 0 |
| lltp-forward | ILL/petri-nets/MCC | intuitionistic auto j1 bias factors | 3137 | 1576 (2 late) | 3071 | 0 | 1495 | 1443 time, 93 bound, 11 killed, 14 crash | 57 time, 9 memory | 0.26× (1574) | 0 |
| lltp-intuitionistic | ILL/ILLTP-LCL-01 | intuitionistic auto j1 | 2 | 0 | 0 | 0 | 0 | 2 bound | 2 bound | – | 0 |
| lltp-intuitionistic | ILL/ILLTP-LCL-cbn | intuitionistic auto j1 | 2 | 2 | 2 | 0 | 0 | – | – | 1.98× (2) | 0 |
| lltp-intuitionistic | ILL/ILLTP-LCL-cbv | intuitionistic auto j1 | 2 | 0 | 0 | 0 | 0 | 2 bound | 2 bound | – | 0 |
| lltp-intuitionistic | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | 252 | 11 | 11 | 0 | 0 | 7 time, 209 bound, 22 depth, 3 killed | 1 time, 215 bound, 22 depth, 3 memory | 1.94× (11) | 0 |
| lltp-intuitionistic | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | 252 | 35 | 35 | 0 | 0 | 8 time, 185 bound, 22 depth, 2 killed | 3 time, 190 bound, 22 depth, 2 memory | 2.11× (35) | 0 |
| lltp-intuitionistic | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | 252 | 39 | 40 | 0 | 1 | 27 time, 173 bound, 10 depth, 3 killed | 22 time, 178 bound, 10 depth, 2 memory | 1.72× (39) | 0 |
| lltp-intuitionistic | ILL/ILLTP-SYN-01 | intuitionistic auto j1 | 19 | 11 | 13 | 0 | 2 | 8 bound | 6 bound | 2.69× (11) | 0 |
| lltp-intuitionistic | ILL/ILLTP-SYN-cbn | intuitionistic auto j1 | 19 | 16 | 18 | 0 | 2 | 3 bound | 1 bound | 2.12× (16) | 0 |
| lltp-intuitionistic | ILL/ILLTP-SYN-cbv | intuitionistic auto j1 | 19 | 16 | 18 | 0 | 2 | 3 bound | 1 bound | 2.35× (16) | 0 |
| lltp-intuitionistic | ILL/KLE-01 | intuitionistic auto j1 | 88 | 50 | 51 | 0 | 1 | 38 bound | 37 bound | 1.61× (50) | 0 |
| lltp-intuitionistic | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | 222 | 162 | 162 | 0 | 0 | 60 bound | 60 bound | 1.81× (162) | 0 |
| lltp-intuitionistic | ILL/KLE-IMP-CONJ/ALT | intuitionistic auto j1 | 27 | 27 | 27 | 0 | 0 | – | – | 1.91× (27) | 0 |
| lltp-intuitionistic | ILL/KLE-IMP-CONJ/NON-THEOREMS | intuitionistic auto j1 | 22 | 22 | 22 | 0 | 0 | – | – | 6.00× (22) | 0 |
| lltp-intuitionistic | ILL/KLE-cbn | intuitionistic auto j1 | 88 | 68 | 69 | 0 | 1 | 20 bound | 19 bound | 2.02× (68) | 0 |
| lltp-intuitionistic | ILL/KLE-cbv | intuitionistic auto j1 | 88 | 67 | 68 | 0 | 1 | 21 bound | 20 bound | 1.68× (67) | 0 |
| lltp-intuitionistic | ILL/Non-theorems | intuitionistic auto j1 | 1 | 0 | 0 | 0 | 0 | 1 bound | 1 bound | – | 0 |
| lltp-intuitionistic | ILL/misc | intuitionistic auto j1 | 3 | 3 | 3 | 0 | 0 | – | – | 1.96× (3) | 0 |
| lltp-intuitionistic | ILL/petri-nets/MCC | intuitionistic auto j1 | 3137 | 1520 (2 late) | 3071 | 0 | 1551 | 1594 time, 2 bound, 7 killed, 14 crash | 58 time, 8 memory | 0.27× (1518) | 0 |
| lltp-intuitionistic-generous | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | 4 | 0 | 0 | 0 | 0 | 4 time | 1 bound, 3 memory | – | 0 |
| lltp-intuitionistic-generous | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | 2 | 0 | 0 | 0 | 0 | 2 time | 2 memory | – | 0 |
| lltp-intuitionistic-generous | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | 3 | 0 | 0 | 0 | 0 | 3 time | 1 time, 2 memory | – | 0 |
| lltp-intuitionistic-generous | ILL/petri-nets/MCC | intuitionistic auto j1 | 19 | 11 | 19 | 0 | 8 | 8 time | – | 0.72× (11) | 0 |
| lltp-rarer | ILL/ILLTP-LCL-01 | intuitionistic auto j1 bias rarer | 2 | 0 | 0 | 0 | 0 | 2 bound | 2 bound | – | 0 |
| lltp-rarer | ILL/ILLTP-LCL-cbn | intuitionistic auto j1 bias rarer | 2 | 1 | 1 | 0 | 0 | 1 bound | 1 bound | 1.42× (1) | 0 |
| lltp-rarer | ILL/ILLTP-LCL-cbv | intuitionistic auto j1 bias rarer | 2 | 0 | 0 | 0 | 0 | 2 bound | 2 bound | – | 0 |
| lltp-rarer | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 bias rarer | 252 | 11 | 11 | 0 | 0 | 7 time, 209 bound, 22 depth, 3 killed | 1 time, 215 bound, 22 depth, 3 memory | 2.00× (11) | 0 |
| lltp-rarer | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 bias rarer | 252 | 34 | 34 | 0 | 0 | 8 time, 186 bound, 22 depth, 2 killed | 3 time, 192 bound, 22 depth, 1 memory | 1.83× (34) | 0 |
| lltp-rarer | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 bias rarer | 252 | 39 | 40 | 0 | 1 | 27 time, 173 bound, 10 depth, 3 killed | 22 time, 178 bound, 10 depth, 2 memory | 1.64× (39) | 0 |
| lltp-rarer | ILL/ILLTP-SYN-01 | intuitionistic auto j1 bias rarer | 19 | 11 | 13 | 0 | 2 | 8 bound | 6 bound | 2.50× (11) | 0 |
| lltp-rarer | ILL/ILLTP-SYN-cbn | intuitionistic auto j1 bias rarer | 19 | 16 | 18 | 0 | 2 | 3 bound | 1 bound | 2.46× (16) | 0 |
| lltp-rarer | ILL/ILLTP-SYN-cbv | intuitionistic auto j1 bias rarer | 19 | 16 | 18 | 0 | 2 | 3 bound | 1 bound | 2.06× (16) | 0 |
| lltp-rarer | ILL/KLE-01 | intuitionistic auto j1 bias rarer | 88 | 50 | 51 | 0 | 1 | 38 bound | 37 bound | 1.51× (50) | 0 |
| lltp-rarer | ILL/KLE-IMP-CONJ | intuitionistic auto j1 bias rarer | 222 | 162 | 162 | 0 | 0 | 60 bound | 60 bound | 1.63× (162) | 0 |
| lltp-rarer | ILL/KLE-IMP-CONJ/ALT | intuitionistic auto j1 bias rarer | 27 | 27 | 27 | 0 | 0 | – | – | 1.89× (27) | 0 |
| lltp-rarer | ILL/KLE-IMP-CONJ/NON-THEOREMS | intuitionistic auto j1 bias rarer | 22 | 22 | 22 | 0 | 0 | – | – | 6.67× (22) | 0 |
| lltp-rarer | ILL/KLE-cbn | intuitionistic auto j1 bias rarer | 88 | 68 | 69 | 0 | 1 | 20 bound | 19 bound | 1.71× (68) | 0 |
| lltp-rarer | ILL/KLE-cbv | intuitionistic auto j1 bias rarer | 88 | 67 | 68 | 0 | 1 | 21 bound | 20 bound | 1.67× (67) | 0 |
| lltp-rarer | ILL/Non-theorems | intuitionistic auto j1 bias rarer | 1 | 0 | 0 | 0 | 0 | 1 bound | 1 bound | – | 0 |
| lltp-rarer | ILL/misc | intuitionistic auto j1 bias rarer | 3 | 3 | 3 | 0 | 0 | – | – | 1.91× (3) | 0 |
| lltp-rarer | ILL/petri-nets/MCC | intuitionistic auto j1 bias rarer | 3137 | 442 | 3071 | 0 | 2629 | 2347 time, 313 bound, 32 depth, 3 crash | 58 time, 8 memory | 0.75× (442) | 0 |
| lltp-recursion | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | 22 | 0 | 0 | 0 | 0 | 20 time, 2 depth | 20 time, 2 depth | – | 0 |
| lltp-recursion | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | 22 | 0 | 0 | 0 | 0 | 20 time, 2 depth | 20 time, 2 depth | – | 0 |
| lltp-recursion | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | 22 | 0 | 0 | 0 | 0 | 22 time | 22 time | – | 0 |
| lltp-recursion | ILL/petri-nets/MCC | intuitionistic auto j1 | 929 | 280 | 895 | 0 | 615 | 632 time, 17 crash | 32 time, 2 memory | 0.48× (280) | 0 |
| lltp-recursion-generous | ILL/petri-nets/MCC | intuitionistic auto j1 | 9 | 5 | 9 | 0 | 4 | 4 time | – | 1.02× (5) | 0 |
| long-1 | 3-partition-no | classical auto j1 | 1 | 1 | 1 | 0 | 0 | – | – | 1.02× (1) | 0 |
| long-1 | counter | classical auto j1 | 2 | 2 | 2 | 0 | 0 | – | – | 0.17× (2) | 0 |
| long-1 | partition-no | classical auto j1 | 2 | 1 | 1 | 0 | 0 | 1 time | 1 time | 0.92× (1) | 0 |
| long-1 | partition-yes | classical auto j1 | 2 | 1 | 1 | 0 | 0 | 1 time | 1 time | 0.93× (1) | 0 |
| long-2 | mix | mix auto j1 | 1 | 0 | 0 | 0 | 0 | 1 time | 1 time | – | 0 |
| long-2 | qbf | classical auto j1 | 2 | 1 | 1 | 0 | 0 | 1 crash | 1 memory | 0.62× (1) | 0 |
| long-2 | wide-m3 | classical auto j1 | 1 | 1 | 1 | 0 | 0 | – | – | 0.98× (1) | 0 |
| long-2 | wide-m4 | classical auto j1 | 1 | 1 | 1 | 0 | 0 | – | – | 0.94× (1) | 0 |
| parallel | 3-partition-no | classical auto j1 | 2 | 2 | 2 | 0 | 0 | – | – | 0.96× (2) | 0 |
| parallel | 3-partition-no | classical auto j16 | 2 | 2 | 2 | 0 | 0 | – | – | 0.98× (2) | 0 |
| parallel | 3-partition-no | classical auto j2 | 2 | 2 | 2 | 0 | 0 | – | – | 1.24× (2) | 0 |
| parallel | 3-partition-no | classical auto j4 | 2 | 2 | 2 | 0 | 0 | – | – | 1.30× (2) | 0 |
| parallel | 3-partition-no | classical auto j8 | 2 | 2 | 2 | 0 | 0 | – | – | 1.30× (2) | 0 |
| parallel | 3-partition-yes | classical auto j1 | 4 | 4 | 4 | 0 | 0 | – | – | 1.59× (4) | 0 |
| parallel | 3-partition-yes | classical auto j16 | 4 | 4 | 4 | 0 | 0 | – | – | 1.17× (4) | 0 |
| parallel | 3-partition-yes | classical auto j2 | 4 | 4 | 4 | 0 | 0 | – | – | 1.55× (4) | 0 |
| parallel | 3-partition-yes | classical auto j4 | 4 | 4 | 4 | 0 | 0 | – | – | 1.50× (4) | 0 |
| parallel | 3-partition-yes | classical auto j8 | 4 | 4 | 4 | 0 | 0 | – | – | 1.27× (4) | 0 |
| parallel | counter | classical auto j1 | 6 | 6 | 6 | 0 | 0 | – | – | 0.52× (6) | 0 |
| parallel | counter | classical auto j16 | 6 | 6 | 6 | 0 | 0 | – | – | 0.12× (6) | 0 |
| parallel | counter | classical auto j2 | 6 | 6 | 6 | 0 | 0 | – | – | 0.57× (6) | 0 |
| parallel | counter | classical auto j4 | 6 | 6 | 6 | 0 | 0 | – | – | 0.26× (6) | 0 |
| parallel | counter | classical auto j8 | 6 | 6 | 6 | 0 | 0 | – | – | 0.20× (6) | 0 |
| parallel | counter-over | classical auto j1 | 6 | 4 | 6 | 0 | 2 | 2 bound | – | 0.52× (4) | 0 |
| parallel | counter-over | classical auto j16 | 6 | 4 | 6 | 0 | 2 | 2 bound | – | 0.13× (4) | 0 |
| parallel | counter-over | classical auto j2 | 6 | 4 | 6 | 0 | 2 | 2 bound | – | 0.60× (4) | 0 |
| parallel | counter-over | classical auto j4 | 6 | 4 | 6 | 0 | 2 | 2 bound | – | 0.40× (4) | 0 |
| parallel | counter-over | classical auto j8 | 6 | 4 | 6 | 0 | 2 | 2 bound | – | 0.25× (4) | 0 |
| parallel | mix | mix auto j1 | 4 | 2 | 3 | 0 | 1 | 2 time | 1 time | 0.55× (2) | 0 |
| parallel | mix | mix auto j16 | 4 | 2 | 2 | 0 | 0 | 2 time | 2 time | 0.66× (2) | 0 |
| parallel | mix | mix auto j2 | 4 | 2 | 3 | 0 | 1 | 2 time | 1 time | 0.52× (2) | 0 |
| parallel | mix | mix auto j4 | 4 | 2 | 3 | 0 | 1 | 2 time | 1 time | 0.55× (2) | 0 |
| parallel | mix | mix auto j8 | 4 | 2 | 3 | 0 | 1 | 2 time | 1 time | 0.72× (2) | 0 |
| parallel | partition-no | classical auto j1 | 4 | 3 | 3 | 0 | 0 | 1 time | 1 time | 0.94× (3) | 0 |
| parallel | partition-no | classical auto j16 | 4 | 3 | 3 | 0 | 0 | 1 time | 1 time | 0.83× (3) | 0 |
| parallel | partition-no | classical auto j2 | 4 | 3 | 3 | 0 | 0 | 1 time | 1 time | 1.09× (3) | 0 |
| parallel | partition-no | classical auto j4 | 4 | 3 | 3 | 0 | 0 | 1 time | 1 time | 1.12× (3) | 0 |
| parallel | partition-no | classical auto j8 | 4 | 3 | 3 | 0 | 0 | 1 time | 1 time | 1.07× (3) | 0 |
| parallel | partition-yes | classical auto j1 | 5 | 5 | 5 | 0 | 0 | – | – | 0.86× (5) | 0 |
| parallel | partition-yes | classical auto j16 | 5 | 4 | 5 | 0 | 1 | 1 time | – | 1.21× (4) | 0 |
| parallel | partition-yes | classical auto j2 | 5 | 4 | 5 | 0 | 1 | 1 time | – | 1.50× (4) | 0 |
| parallel | partition-yes | classical auto j4 | 5 | 4 | 5 | 0 | 1 | 1 time | – | 1.28× (4) | 0 |
| parallel | partition-yes | classical auto j8 | 5 | 4 | 5 | 0 | 1 | 1 time | – | 1.16× (4) | 0 |
| parallel | qbf | classical auto j1 | 20 | 19 | 19 | 0 | 0 | 1 time | 1 memory | 0.83× (19) | 0 |
| parallel | qbf | classical auto j16 | 20 | 19 | 14 | 5 | 0 | 1 time | 6 memory | 0.77× (14) | 0 |
| parallel | qbf | classical auto j2 | 20 | 19 | 14 | 5 | 0 | 1 time | 6 memory | 0.70× (14) | 0 |
| parallel | qbf | classical auto j4 | 20 | 19 | 14 | 5 | 0 | 1 time | 6 memory | 0.68× (14) | 0 |
| parallel | qbf | classical auto j8 | 20 | 19 | 14 | 5 | 0 | 1 time | 6 memory | 0.73× (14) | 0 |
| parallel | wide-m3 | classical auto j1 | 3 | 3 | 3 | 0 | 0 | – | – | 1.17× (3) | 0 |
| parallel | wide-m3 | classical auto j16 | 3 | 3 | 3 | 0 | 0 | – | – | 1.07× (3) | 0 |
| parallel | wide-m3 | classical auto j2 | 3 | 3 | 3 | 0 | 0 | – | – | 1.51× (3) | 0 |
| parallel | wide-m3 | classical auto j4 | 3 | 3 | 3 | 0 | 0 | – | – | 1.27× (3) | 0 |
| parallel | wide-m3 | classical auto j8 | 3 | 3 | 3 | 0 | 0 | – | – | 1.19× (3) | 0 |
| parallel | wide-m4 | classical auto j1 | 3 | 3 | 3 | 0 | 0 | – | – | 1.04× (3) | 0 |
| parallel | wide-m4 | classical auto j16 | 3 | 3 | 3 | 0 | 0 | – | – | 1.13× (3) | 0 |
| parallel | wide-m4 | classical auto j2 | 3 | 3 | 3 | 0 | 0 | – | – | 1.42× (3) | 0 |
| parallel | wide-m4 | classical auto j4 | 3 | 3 | 3 | 0 | 0 | – | – | 1.30× (3) | 0 |
| parallel | wide-m4 | classical auto j8 | 3 | 3 | 3 | 0 | 0 | – | – | 1.32× (3) | 0 |
| parallel-net | 3-partition-mll-no | classical net j1 | 3 | 2 | 2 | 0 | 0 | 1 time | 1 time | 0.96× (2) | 0 |
| parallel-net | 3-partition-mll-no | classical net j16 | 3 | 2 | 2 | 0 | 0 | 1 time | 1 time | 0.99× (2) | 0 |
| parallel-net | 3-partition-mll-no | classical net j2 | 3 | 2 | 2 | 0 | 0 | 1 time | 1 time | 0.94× (2) | 0 |
| parallel-net | 3-partition-mll-no | classical net j4 | 3 | 2 | 2 | 0 | 0 | 1 time | 1 time | 0.96× (2) | 0 |
| parallel-net | 3-partition-mll-no | classical net j8 | 3 | 2 | 2 | 0 | 0 | 1 time | 1 time | 0.97× (2) | 0 |
| parallel-net | partition-table | classical net j1 | 13 | 9 | 9 | 0 | 0 | 4 time | 4 time | 1.00× (9) | 0 |
| parallel-net | partition-table | classical net j16 | 13 | 10 | 10 | 0 | 0 | 3 time | 3 time | 1.01× (10) | 0 |
| parallel-net | partition-table | classical net j2 | 13 | 9 | 9 | 0 | 0 | 4 time | 4 time | 0.96× (9) | 0 |
| parallel-net | partition-table | classical net j4 | 13 | 10 | 10 | 0 | 0 | 3 time | 3 time | 0.96× (10) | 0 |
| parallel-net | partition-table | classical net j8 | 13 | 10 | 10 | 0 | 0 | 3 time | 3 time | 0.98× (10) | 0 |
| period-1 | 3-partition-mll-no | classical net j1 period 1 | 2 | 2 | 2 | 0 | 0 | – | – | 0.98× (2) | 0 |
| period-1 | partition-no | classical net j1 period 1 | 2 | 1 | 1 | 0 | 0 | 1 time | 1 time | 0.95× (1) | 0 |
| period-1 | partition-yes | classical net j1 period 1 | 1 | 1 | 1 | 0 | 0 | – | – | 0.98× (1) | 0 |
| period-1 | wide-m1 | classical net j1 period 1 | 2 | 2 | 2 | 0 | 0 | – | – | 0.92× (2) | 0 |
| period-1 | wide-m2 | classical net j1 period 1 | 2 | 2 | 2 | 0 | 0 | – | – | 0.94× (2) | 0 |
| period-16 | 3-partition-mll-no | classical net j1 period 16 | 2 | 1 | 1 | 0 | 0 | 1 time | 1 time | 1.00× (1) | 0 |
| period-16 | partition-no | classical net j1 period 16 | 2 | 0 | 0 | 0 | 0 | 2 time | 2 time | – | 0 |
| period-16 | partition-yes | classical net j1 period 16 | 1 | 0 | 0 | 0 | 0 | 1 time | 1 time | – | 0 |
| period-16 | wide-m1 | classical net j1 period 16 | 2 | 2 | 2 | 0 | 0 | – | – | 0.94× (2) | 0 |
| period-16 | wide-m2 | classical net j1 period 16 | 2 | 2 | 2 | 0 | 0 | – | – | 0.94× (2) | 0 |
| period-2 | 3-partition-mll-no | classical net j1 period 2 | 2 | 2 | 2 | 0 | 0 | – | – | 0.96× (2) | 0 |
| period-2 | partition-no | classical net j1 period 2 | 2 | 1 | 1 | 0 | 0 | 1 time | 1 time | 0.95× (1) | 0 |
| period-2 | partition-yes | classical net j1 period 2 | 1 | 1 | 1 | 0 | 0 | – | – | 0.95× (1) | 0 |
| period-2 | wide-m1 | classical net j1 period 2 | 2 | 2 | 2 | 0 | 0 | – | – | 0.98× (2) | 0 |
| period-2 | wide-m2 | classical net j1 period 2 | 2 | 2 | 2 | 0 | 0 | – | – | 0.97× (2) | 0 |
| period-8 | 3-partition-mll-no | classical net j1 period 8 | 2 | 1 | 1 | 0 | 0 | 1 time | 1 time | 0.96× (1) | 0 |
| period-8 | partition-no | classical net j1 period 8 | 2 | 0 | 0 | 0 | 0 | 2 time | 2 time | – | 0 |
| period-8 | partition-yes | classical net j1 period 8 | 1 | 0 | 0 | 0 | 0 | 1 time | 1 time | – | 0 |
| period-8 | wide-m1 | classical net j1 period 8 | 2 | 2 | 2 | 0 | 0 | – | – | 0.96× (2) | 0 |
| period-8 | wide-m2 | classical net j1 period 8 | 2 | 2 | 2 | 0 | 0 | – | – | 0.97× (2) | 0 |

### Generated and listed problems

| file | family | configuration | problem | before | after | after/before |
|---|---|---|---|--:|--:|--:|
| engines | 3-partition-mll-no | classical focus j1 | 3-partition-mll-no/4 | 20 µs ✗ | 39 µs ✗ | 1.95× |
| engines | 3-partition-mll-no | classical focus j1 | 3-partition-mll-no/5 | 20 µs ✗ | 41 µs ✗ | 2.05× |
| engines | 3-partition-mll-no | classical focus j1 | 3-partition-mll-no/6 | 22 µs ✗ | 41 µs ✗ | 1.86× |
| engines | 3-partition-mll-no | classical focus j1 | 3-partition-mll-no/8 | 20 µs ✗ | 46 µs ✗ | 2.30× |
| engines | 3-partition-mll-no | classical net j1 | 3-partition-mll-no/4 | 1.86 s ✗ | 1.83 s ✗ | 0.98× |
| engines | 3-partition-mll-no | classical net j1 | 3-partition-mll-no/5 | 57.68 s ✗ | 56.13 s ✗ | 0.97× |
| engines | 3-partition-mll-no | classical net j1 | 3-partition-mll-no/6 | > 60 s | > 60 s |  |
| engines | 3-partition-mll-no | classical net j1 | 3-partition-mll-no/8 | > 60 s | > 60 s |  |
| engines | 3-partition-mll-yes | classical focus j1 | 3-partition-mll-yes/12 | 42 µs ✓ | 57 µs ✓ | 1.36× |
| engines | 3-partition-mll-yes | classical focus j1 | 3-partition-mll-yes/4 | 40 µs ✓ | 64 µs ✓ | 1.60× |
| engines | 3-partition-mll-yes | classical focus j1 | 3-partition-mll-yes/6 | 42 µs ✓ | 55 µs ✓ | 1.31× |
| engines | 3-partition-mll-yes | classical focus j1 | 3-partition-mll-yes/8 | 45 µs ✓ | 55 µs ✓ | 1.22× |
| engines | 3-partition-mll-yes | classical net j1 | 3-partition-mll-yes/12 | > 60 s | 170 µs ✓ |  |
| engines | 3-partition-mll-yes | classical net j1 | 3-partition-mll-yes/4 | 806 µs ✓ | 62 µs ✓ | 7.7e-2× |
| engines | 3-partition-mll-yes | classical net j1 | 3-partition-mll-yes/6 | 117.3 ms ✓ | 88 µs ✓ | 7.5e-4× |
| engines | 3-partition-mll-yes | classical net j1 | 3-partition-mll-yes/8 | 5.68 s ✓ | 107 µs ✓ | 1.9e-5× |
| engines | additive | classical focus j1 | additive/12 | 320.2 ms ✓ | 38.1 ms ✓ | 0.12× |
| engines | additive | classical focus j1 | additive/14 | 5.41 s ✓ | 391.5 ms ✓ | 7.2e-2× |
| engines | additive | classical focus j1 | additive/16 | 9.95 s unknown | 5.79 s ✓ |  |
| engines | additive | classical focus j1 | additive/8 | 1.5 ms ✓ | 873 µs ✓ | 0.58× |
| engines | cancellation | classical focus j1 | cancellation/3-partition-4 | 319 µs ✓ | 300 µs ✓ | 0.94× |
| engines | partition-no | classical focus j1 | partition-no/12 | 3.91 s ✗ | 3.81 s ✗ | 0.97× |
| engines | partition-no | classical focus j1 | partition-no/3 | 59 µs ✗ | 71 µs ✗ | 1.20× |
| engines | partition-no | classical focus j1 | partition-no/4 | 138 µs ✗ | 150 µs ✗ | 1.09× |
| engines | partition-no | classical focus j1 | partition-no/9 | 86.1 ms ✗ | 89.0 ms ✗ | 1.03× |
| engines | partition-no | classical net j1 | partition-no/12 | > 60 s | > 60 s |  |
| engines | partition-no | classical net j1 | partition-no/3 | 3.57 s ✗ | 3.68 s ✗ | 1.03× |
| engines | partition-no | classical net j1 | partition-no/4 | > 60 s | > 60 s |  |
| engines | partition-no | classical net j1 | partition-no/9 | > 60 s | > 60 s |  |
| engines | partition-table | classical focus j1 | partition-table/1-1 | 33 µs ✓ | 51 µs ✓ | 1.55× |
| engines | partition-table | classical focus j1 | partition-table/1-1-1-1-1-7 | 1.3 ms ✗ | 1.2 ms ✗ | 0.87× |
| engines | partition-table | classical focus j1 | partition-table/1-1-1-5 | 149 µs ✗ | 141 µs ✗ | 0.95× |
| engines | partition-table | classical focus j1 | partition-table/1-1-2-4 | 48 µs ✓ | 70 µs ✓ | 1.46× |
| engines | partition-table | classical focus j1 | partition-table/1-1-4 | 56 µs ✗ | 81 µs ✗ | 1.45× |
| engines | partition-table | classical focus j1 | partition-table/1-2-3-4-5-5 | 80 µs ✓ | 97 µs ✓ | 1.21× |
| engines | partition-table | classical focus j1 | partition-table/1-2-5 | 59 µs ✗ | 83 µs ✗ | 1.41× |
| engines | partition-table | classical focus j1 | partition-table/1-3 | 29 µs ✗ | 50 µs ✗ | 1.72× |
| engines | partition-table | classical focus j1 | partition-table/2-1-1 | 45 µs ✓ | 67 µs ✓ | 1.49× |
| engines | partition-table | classical focus j1 | partition-table/2-2-1-1 | 62 µs ✓ | 82 µs ✓ | 1.32× |
| engines | partition-table | classical focus j1 | partition-table/2-2-2-2-2-2-9-1 | 124 µs ✓ | 129 µs ✓ | 1.04× |
| engines | partition-table | classical focus j1 | partition-table/2-3-2-1 | 68 µs ✓ | 75 µs ✓ | 1.10× |
| engines | partition-table | classical focus j1 | partition-table/3-3-3-1 | 143 µs ✗ | 137 µs ✗ | 0.96× |
| engines | partition-table | classical net j1 | partition-table/1-1 | 34 µs ✓ | 51 µs ✓ | 1.50× |
| engines | partition-table | classical net j1 | partition-table/1-1-1-1-1-7 | > 60 s | > 60 s |  |
| engines | partition-table | classical net j1 | partition-table/1-1-1-5 | 7.59 s ✗ | 7.43 s ✗ | 0.98× |
| engines | partition-table | classical net j1 | partition-table/1-1-2-4 | 1.10 s ✓ | 1.04 s ✓ | 0.95× |
| engines | partition-table | classical net j1 | partition-table/1-1-4 | 40.7 ms ✗ | 40.2 ms ✗ | 0.99× |
| engines | partition-table | classical net j1 | partition-table/1-2-3-4-5-5 | > 60 s | > 60 s |  |
| engines | partition-table | classical net j1 | partition-table/1-2-5 | 3.75 s ✗ | 3.68 s ✗ | 0.98× |
| engines | partition-table | classical net j1 | partition-table/1-3 | 372 µs ✗ | 388 µs ✗ | 1.04× |
| engines | partition-table | classical net j1 | partition-table/2-1-1 | 644 µs ✓ | 626 µs ✓ | 0.97× |
| engines | partition-table | classical net j1 | partition-table/2-2-1-1 | 55.2 ms ✓ | 52.2 ms ✓ | 0.95× |
| engines | partition-table | classical net j1 | partition-table/2-2-2-2-2-2-9-1 | > 60 s | > 60 s |  |
| engines | partition-table | classical net j1 | partition-table/2-3-2-1 | 4.67 s ✓ | 4.68 s ✓ | 1.00× |
| engines | partition-table | classical net j1 | partition-table/3-3-3-1 | > 60 s | > 60 s |  |
| engines | partition-yes | classical focus j1 | partition-yes/12 | 15.2 ms ✓ | 13.3 ms ✓ | 0.88× |
| engines | partition-yes | classical focus j1 | partition-yes/20 | 573.7 ms ✓ | 471.1 ms ✓ | 0.82× |
| engines | partition-yes | classical focus j1 | partition-yes/4 | 60 µs ✓ | 104 µs ✓ | 1.73× |
| engines | partition-yes | classical focus j1 | partition-yes/5 | 77 µs ✓ | 99 µs ✓ | 1.29× |
| engines | partition-yes | classical focus j1 | partition-yes/6 | 130 µs ✓ | 140 µs ✓ | 1.08× |
| engines | partition-yes | classical net j1 | partition-yes/12 | > 60 s | > 60 s |  |
| engines | partition-yes | classical net j1 | partition-yes/20 | > 60 s | > 60 s |  |
| engines | partition-yes | classical net j1 | partition-yes/4 | 2.83 s ✓ | 2.83 s ✓ | 1.00× |
| engines | partition-yes | classical net j1 | partition-yes/5 | > 60 s | > 60 s |  |
| engines | partition-yes | classical net j1 | partition-yes/6 | > 60 s | > 60 s |  |
| engines | wide-m1 | classical focus j1 | wide-m1/16 | 68 µs ✓ | 77 µs ✓ | 1.13× |
| engines | wide-m1 | classical focus j1 | wide-m1/2048 | 271.0 ms ? depth | 206.6 ms ✓ |  |
| engines | wide-m1 | classical focus j1 | wide-m1/24 | 104 µs ✓ | 101 µs ✓ | 0.97× |
| engines | wide-m1 | classical focus j1 | wide-m1/256 | 5.6 ms ✓ | 4.0 ms ✓ | 0.71× |
| engines | wide-m1 | classical focus j1 | wide-m1/32 | 160 µs ✓ | 148 µs ✓ | 0.92× |
| engines | wide-m1 | classical focus j1 | wide-m1/8 | 43 µs ✓ | 52 µs ✓ | 1.21× |
| engines | wide-m1 | classical net j1 | wide-m1/16 | 110 µs ✓ | 120 µs ✓ | 1.09× |
| engines | wide-m1 | classical net j1 | wide-m1/2048 | 637.3 ms ✓ | 586.4 ms ✓ | 0.92× |
| engines | wide-m1 | classical net j1 | wide-m1/24 | 207 µs ✓ | 199 µs ✓ | 0.96× |
| engines | wide-m1 | classical net j1 | wide-m1/256 | 9.3 ms ✓ | 9.0 ms ✓ | 0.97× |
| engines | wide-m1 | classical net j1 | wide-m1/32 | 179 µs ✓ | 182 µs ✓ | 1.02× |
| engines | wide-m1 | classical net j1 | wide-m1/8 | 43 µs ✓ | 62 µs ✓ | 1.44× |
| engines | wide-m2 | classical focus j1 | wide-m2/16 | 66 µs ✓ | 69 µs ✓ | 1.05× |
| engines | wide-m2 | classical focus j1 | wide-m2/2048 | 242.7 ms ? depth | 209.9 ms ✓ |  |
| engines | wide-m2 | classical focus j1 | wide-m2/24 | 97 µs ✓ | 105 µs ✓ | 1.08× |
| engines | wide-m2 | classical focus j1 | wide-m2/256 | 4.3 ms ✓ | 3.2 ms ✓ | 0.74× |
| engines | wide-m2 | classical focus j1 | wide-m2/32 | 141 µs ✓ | 128 µs ✓ | 0.91× |
| engines | wide-m2 | classical focus j1 | wide-m2/8 | 40 µs ✓ | 55 µs ✓ | 1.38× |
| engines | wide-m2 | classical net j1 | wide-m2/16 | 103 µs ✓ | 113 µs ✓ | 1.10× |
| engines | wide-m2 | classical net j1 | wide-m2/2048 | 635.2 ms ✓ | 610.6 ms ✓ | 0.96× |
| engines | wide-m2 | classical net j1 | wide-m2/24 | 205 µs ✓ | 205 µs ✓ | 1.00× |
| engines | wide-m2 | classical net j1 | wide-m2/256 | 9.4 ms ✓ | 9.0 ms ✓ | 0.96× |
| engines | wide-m2 | classical net j1 | wide-m2/32 | 179 µs ✓ | 178 µs ✓ | 0.99× |
| engines | wide-m2 | classical net j1 | wide-m2/8 | 43 µs ✓ | 55 µs ✓ | 1.28× |
| engines | wide-m3 | classical focus j1 | wide-m3/12 | 50 µs ✓ | 69 µs ✓ | 1.38× |
| engines | wide-m3 | classical focus j1 | wide-m3/2048 | 215.8 ms ? depth | 192.5 ms ✓ |  |
| engines | wide-m3 | classical focus j1 | wide-m3/24 | 94 µs ✓ | 96 µs ✓ | 1.02× |
| engines | wide-m3 | classical focus j1 | wide-m3/256 | 3.7 ms ✓ | 3.0 ms ✓ | 0.81× |
| engines | wide-m3 | classical focus j1 | wide-m3/30 | 126 µs ✓ | 123 µs ✓ | 0.98× |
| engines | wide-m3 | classical net j1 | wide-m3/12 | 74 µs ✓ | 85 µs ✓ | 1.15× |
| engines | wide-m3 | classical net j1 | wide-m3/2048 | 649.4 ms ✓ | 628.1 ms ✓ | 0.97× |
| engines | wide-m3 | classical net j1 | wide-m3/24 | 213 µs ✓ | 211 µs ✓ | 0.99× |
| engines | wide-m3 | classical net j1 | wide-m3/256 | 9.7 ms ✓ | 9.4 ms ✓ | 0.97× |
| engines | wide-m3 | classical net j1 | wide-m3/30 | 166 µs ✓ | 176 µs ✓ | 1.06× |
| engines | wide-m4 | classical focus j1 | wide-m4/12 | 53 µs ✓ | 59 µs ✓ | 1.11× |
| engines | wide-m4 | classical focus j1 | wide-m4/2048 | 203.7 ms ? depth | 182.0 ms ✓ |  |
| engines | wide-m4 | classical focus j1 | wide-m4/24 | 92 µs ✓ | 93 µs ✓ | 1.01× |
| engines | wide-m4 | classical focus j1 | wide-m4/256 | 3.5 ms ✓ | 2.8 ms ✓ | 0.81× |
| engines | wide-m4 | classical focus j1 | wide-m4/28 | 115 µs ✓ | 105 µs ✓ | 0.91× |
| engines | wide-m4 | classical net j1 | wide-m4/12 | 73 µs ✓ | 81 µs ✓ | 1.11× |
| engines | wide-m4 | classical net j1 | wide-m4/2048 | 673.2 ms ✓ | 651.3 ms ✓ | 0.97× |
| engines | wide-m4 | classical net j1 | wide-m4/24 | 215 µs ✓ | 218 µs ✓ | 1.01× |
| engines | wide-m4 | classical net j1 | wide-m4/256 | 10.1 ms ✓ | 9.8 ms ✓ | 0.97× |
| engines | wide-m4 | classical net j1 | wide-m4/28 | 278 µs ✓ | 258 µs ✓ | 0.93× |
| families | 3-partition-mll-no | classical auto j1 | 3-partition-mll-no/4 | 19 µs ✗ | 39 µs ✗ | 2.05× |
| families | 3-partition-mll-no | classical auto j1 | 3-partition-mll-no/5 | 18 µs ✗ | 41 µs ✗ | 2.28× |
| families | 3-partition-mll-no | classical auto j1 | 3-partition-mll-no/6 | 20 µs ✗ | 45 µs ✗ | 2.25× |
| families | 3-partition-mll-no | classical auto j1 | 3-partition-mll-no/8 | 20 µs ✗ | 40 µs ✗ | 2.00× |
| families | 3-partition-mll-yes | classical auto j1 | 3-partition-mll-yes/12 | 41 µs ✓ | 55 µs ✓ | 1.34× |
| families | 3-partition-mll-yes | classical auto j1 | 3-partition-mll-yes/4 | 36 µs ✓ | 54 µs ✓ | 1.50× |
| families | 3-partition-mll-yes | classical auto j1 | 3-partition-mll-yes/6 | 38 µs ✓ | 53 µs ✓ | 1.39× |
| families | 3-partition-mll-yes | classical auto j1 | 3-partition-mll-yes/8 | 39 µs ✓ | 56 µs ✓ | 1.44× |
| families | 3-partition-no | classical auto j1 | 3-partition-no/4 | 299 µs ✗ | 319 µs ✗ | 1.07× |
| families | 3-partition-no | classical auto j1 | 3-partition-no/5 | 358 µs ✗ | 346 µs ✗ | 0.97× |
| families | 3-partition-yes | classical auto j1 | 3-partition-yes/12 | 48 µs ✓ | 64 µs ✓ | 1.33× |
| families | 3-partition-yes | classical auto j1 | 3-partition-yes/4 | 41 µs ✓ | 68 µs ✓ | 1.66× |
| families | 3-partition-yes | classical auto j1 | 3-partition-yes/6 | 43 µs ✓ | 61 µs ✓ | 1.42× |
| families | 3-partition-yes | classical auto j1 | 3-partition-yes/8 | 44 µs ✓ | 63 µs ✓ | 1.43× |
| families | additive | classical auto j1 | additive/12 | 13.8 ms ✓ | 10.8 ms ✓ | 0.79× |
| families | additive | classical auto j1 | additive/14 | 68.0 ms ✓ | 58.4 ms ✓ | 0.86× |
| families | additive | classical auto j1 | additive/16 | 209.5 ms ✓ | 181.6 ms ✓ | 0.87× |
| families | additive | classical auto j1 | additive/8 | 378 µs ✓ | 329 µs ✓ | 0.87× |
| families | chain | classical auto j1 | chain/128 | 40.2 ms ✓ | 33.7 ms ✓ | 0.84× |
| families | chain | classical auto j1 | chain/16 | 151 µs ✓ | 154 µs ✓ | 1.02× |
| families | chain | classical auto j1 | chain/256 | 321.3 ms ✓ | 274.7 ms ✓ | 0.86× |
| families | chain | classical auto j1 | chain/64 | 5.1 ms ✓ | 4.4 ms ✓ | 0.86× |
| families | counter | classical auto j1 | counter/16 | 248 µs ✓ | 37 µs ✓ | 0.15× |
| families | counter | classical auto j1 | counter/2 | 51 µs ✓ | 27 µs ✓ | 0.53× |
| families | counter | classical auto j1 | counter/32 | 2.18 s ✓ | 43 µs ✓ | 2.0e-5× |
| families | counter | classical auto j1 | counter/4 | 65 µs ✓ | 28 µs ✓ | 0.43× |
| families | counter | classical auto j1 | counter/64 | 80.68 s ✓ | 55 µs ✓ | 6.8e-7× |
| families | counter | classical auto j1 | counter/8 | 87 µs ✓ | 34 µs ✓ | 0.39× |
| families | counter-over | classical auto j1 | counter-over/16 | 299 µs ✗ | 35 µs ✗ | 0.12× |
| families | counter-over | classical auto j1 | counter-over/2 | 55 µs ✗ | 24 µs ✗ | 0.44× |
| families | counter-over | classical auto j1 | counter-over/32 | 2.22 s ? bound | 44 µs ✗ |  |
| families | counter-over | classical auto j1 | counter-over/4 | 59 µs ✗ | 26 µs ✗ | 0.44× |
| families | counter-over | classical auto j1 | counter-over/64 | 85.03 s ? bound | 45 µs ✗ |  |
| families | counter-over | classical auto j1 | counter-over/8 | 110 µs ✗ | 33 µs ✗ | 0.30× |
| families | growing | classical auto j1 | growing/1024 | 256.8 ms ? depth | 114.7 ms ? depth |  |
| families | growing | classical auto j1 | growing/16 | 342 µs ? bound | 213 µs ? bound |  |
| families | growing | classical auto j1 | growing/256 | 41.8 ms ? bound | 18.4 ms ? bound |  |
| families | growing | classical auto j1 | growing/64 | 1.6 ms ? bound | 800 µs ? bound |  |
| families | mix | mix auto j1 | mix/10 | 145.85 s ✗ | 62.55 s ✗ | 0.43× |
| families | mix | mix auto j1 | mix/11 | > 300 s | > 300 s |  |
| families | mix | mix auto j1 | mix/4 | 302 µs ✗ | 365 µs ✗ | 1.21× |
| families | mix | mix auto j1 | mix/6 | 18.4 ms ✗ | 14.0 ms ✗ | 0.76× |
| families | mix | mix auto j1 | mix/8 | 1.55 s ✗ | 858.5 ms ✗ | 0.55× |
| families | mix | mix auto j1 | mix/9 | 14.10 s ✗ | 7.31 s ✗ | 0.52× |
| families | partition-no | classical auto j1 | partition-no/12 | 3.87 s ✗ | 3.82 s ✗ | 0.99× |
| families | partition-no | classical auto j1 | partition-no/14 | > 300 s | > 300 s |  |
| families | partition-no | classical auto j1 | partition-no/15 | > 300 s | > 300 s |  |
| families | partition-no | classical auto j1 | partition-no/3 | 59 µs ✗ | 79 µs ✗ | 1.34× |
| families | partition-no | classical auto j1 | partition-no/4 | 141 µs ✗ | 144 µs ✗ | 1.02× |
| families | partition-no | classical auto j1 | partition-no/5 | 465 µs ✗ | 435 µs ✗ | 0.94× |
| families | partition-no | classical auto j1 | partition-no/9 | 84.9 ms ✗ | 85.5 ms ✗ | 1.01× |
| families | partition-yes | classical auto j1 | partition-yes/12 | 14.8 ms ✓ | 13.8 ms ✓ | 0.93× |
| families | partition-yes | classical auto j1 | partition-yes/16 | 186.8 ms ✓ | 147.8 ms ✓ | 0.79× |
| families | partition-yes | classical auto j1 | partition-yes/20 | 568.4 ms ✓ | 454.7 ms ✓ | 0.80× |
| families | partition-yes | classical auto j1 | partition-yes/24 | 117.66 s ✓ | 74.41 s ✓ | 0.63× |
| families | partition-yes | classical auto j1 | partition-yes/28 | > 300 s | > 300 s |  |
| families | partition-yes | classical auto j1 | partition-yes/4 | 55 µs ✓ | 81 µs ✓ | 1.47× |
| families | partition-yes | classical auto j1 | partition-yes/5 | 76 µs ✓ | 93 µs ✓ | 1.22× |
| families | partition-yes | classical auto j1 | partition-yes/6 | 130 µs ✓ | 136 µs ✓ | 1.05× |
| families | partition-yes | classical auto j1 | partition-yes/7 | 213 µs ✓ | 211 µs ✓ | 0.99× |
| families | qbf | classical auto j1 | qbf/12#0 | 1.8 ms ✓ | 1.2 ms ✓ | 0.67× |
| families | qbf | classical auto j1 | qbf/12#1 | 502 µs ✗ | 388 µs ✗ | 0.77× |
| families | qbf | classical auto j1 | qbf/12#2 | 925 µs ✓ | 625 µs ✓ | 0.68× |
| families | qbf | classical auto j1 | qbf/12#3 | 371 µs ✗ | 297 µs ✗ | 0.80× |
| families | qbf | classical auto j1 | qbf/16#0 | 10.1 ms ✓ | 6.0 ms ✓ | 0.60× |
| families | qbf | classical auto j1 | qbf/16#1 | 1.5 ms ✗ | 1.1 ms ✗ | 0.75× |
| families | qbf | classical auto j1 | qbf/16#2 | 2.4 ms ✗ | 1.7 ms ✗ | 0.70× |
| families | qbf | classical auto j1 | qbf/16#3 | 7.7 ms ✓ | 5.4 ms ✓ | 0.69× |
| families | qbf | classical auto j1 | qbf/20#0 | 7.3 ms ✗ | 5.3 ms ✗ | 0.72× |
| families | qbf | classical auto j1 | qbf/20#1 | 12.0 ms ✗ | 8.2 ms ✗ | 0.68× |
| families | qbf | classical auto j1 | qbf/20#2 | 49.8 ms ✓ | 30.0 ms ✓ | 0.60× |
| families | qbf | classical auto j1 | qbf/20#3 | 7.1 ms ✗ | 5.1 ms ✗ | 0.72× |
| families | qbf | classical auto j1 | qbf/24#0 | 163.5 ms ✗ | 101.8 ms ✗ | 0.62× |
| families | qbf | classical auto j1 | qbf/24#1 | 58.0 ms ✗ | 37.0 ms ✗ | 0.64× |
| families | qbf | classical auto j1 | qbf/24#2 | 38.7 ms ✗ | 25.7 ms ✗ | 0.67× |
| families | qbf | classical auto j1 | qbf/24#3 | 57.2 ms ✗ | 36.1 ms ✗ | 0.63× |
| families | qbf | classical auto j1 | qbf/32#0 | 875.9 ms ✗ | 552.4 ms ✗ | 0.63× |
| families | qbf | classical auto j1 | qbf/32#1 | 943.7 ms ✗ | 626.7 ms ✗ | 0.66× |
| families | qbf | classical auto j1 | qbf/32#2 | 2.04 s ✗ | 1.14 s ✗ | 0.56× |
| families | qbf | classical auto j1 | qbf/32#3 | 778.5 ms ✗ | 476.7 ms ✗ | 0.61× |
| families | qbf | classical auto j1 | qbf/40#0 | 17.53 s ✗ | 12.11 s ✗ | 0.69× |
| families | qbf | classical auto j1 | qbf/40#1 | 20.18 s ✗ | 13.65 s ✗ | 0.68× |
| families | qbf | classical auto j1 | qbf/40#2 | 23.34 s ✗ | 16.21 s ✗ | 0.69× |
| families | qbf | classical auto j1 | qbf/40#3 | 18.29 s ✗ | 12.54 s ✗ | 0.69× |
| families | qbf | classical auto j1 | qbf/44#0 | 77.60 s ✗ | 54.12 s ✗ | 0.70× |
| families | qbf | classical auto j1 | qbf/44#1 | 79.01 s ✗ | 53.43 s ✗ | 0.68× |
| families | qbf | classical auto j1 | qbf/44#2 | 115.85 s ✗ | 80.52 s ✗ | 0.69× |
| families | qbf | classical auto j1 | qbf/44#3 | 218.72 s ✗ | 28.71 s ? memory |  |
| families | qbf | classical auto j1 | qbf/48#0 | 290.27 s unknown | 84.57 s ? memory |  |
| families | qbf | classical auto j1 | qbf/48#1 | 282.04 s ✗ | 207.33 s ✗ | 0.74× |
| families | qbf | classical auto j1 | qbf/48#2 | 230.53 s ✗ | 161.73 s ✗ | 0.70× |
| families | qbf | classical auto j1 | qbf/48#3 | > 300 s | 222.46 s ✗ |  |
| families | qbf | classical auto j1 | qbf/8#0 | 123 µs ✗ | 131 µs ✗ | 1.07× |
| families | qbf | classical auto j1 | qbf/8#1 | 161 µs ✗ | 144 µs ✗ | 0.89× |
| families | qbf | classical auto j1 | qbf/8#2 | 184 µs ✓ | 156 µs ✓ | 0.85× |
| families | qbf | classical auto j1 | qbf/8#3 | 141 µs ✗ | 130 µs ✗ | 0.92× |
| families | wide-m1 | classical auto j1 | wide-m1/16 | 98 µs ✓ | 110 µs ✓ | 1.12× |
| families | wide-m1 | classical auto j1 | wide-m1/2048 | 581.5 ms ✓ | 523.0 ms ✓ | 0.90× |
| families | wide-m1 | classical auto j1 | wide-m1/24 | 199 µs ✓ | 192 µs ✓ | 0.96× |
| families | wide-m1 | classical auto j1 | wide-m1/256 | 8.2 ms ✓ | 7.7 ms ✓ | 0.94× |
| families | wide-m1 | classical auto j1 | wide-m1/32 | 166 µs ✓ | 174 µs ✓ | 1.05× |
| families | wide-m1 | classical auto j1 | wide-m1/8 | 40 µs ✓ | 54 µs ✓ | 1.35× |
| families | wide-m2 | classical auto j1 | wide-m2/16 | 97 µs ✓ | 107 µs ✓ | 1.10× |
| families | wide-m2 | classical auto j1 | wide-m2/2048 | 571.3 ms ✓ | 555.5 ms ✓ | 0.97× |
| families | wide-m2 | classical auto j1 | wide-m2/24 | 191 µs ✓ | 194 µs ✓ | 1.02× |
| families | wide-m2 | classical auto j1 | wide-m2/256 | 8.5 ms ✓ | 8.0 ms ✓ | 0.93× |
| families | wide-m2 | classical auto j1 | wide-m2/32 | 165 µs ✓ | 171 µs ✓ | 1.04× |
| families | wide-m2 | classical auto j1 | wide-m2/8 | 44 µs ✓ | 54 µs ✓ | 1.23× |
| families | wide-m3 | classical auto j1 | wide-m3/1024 | 55.2 ms ✓ | 42.8 ms ✓ | 0.78× |
| families | wide-m3 | classical auto j1 | wide-m3/12 | 44 µs ✓ | 59 µs ✓ | 1.34× |
| families | wide-m3 | classical auto j1 | wide-m3/2048 | 204.3 ms ? depth | 172.9 ms ✓ |  |
| families | wide-m3 | classical auto j1 | wide-m3/24 | 88 µs ✓ | 87 µs ✓ | 0.99× |
| families | wide-m3 | classical auto j1 | wide-m3/256 | 3.3 ms ✓ | 2.7 ms ✓ | 0.82× |
| families | wide-m3 | classical auto j1 | wide-m3/30 | 112 µs ✓ | 104 µs ✓ | 0.93× |
| families | wide-m3 | classical auto j1 | wide-m3/36 | 139 µs ✓ | 128 µs ✓ | 0.92× |
| families | wide-m4 | classical auto j1 | wide-m4/1024 | 51.6 ms ✓ | 40.2 ms ✓ | 0.78× |
| families | wide-m4 | classical auto j1 | wide-m4/12 | 49 µs ✓ | 58 µs ✓ | 1.18× |
| families | wide-m4 | classical auto j1 | wide-m4/2048 | 191.8 ms ? depth | 167.4 ms ✓ |  |
| families | wide-m4 | classical auto j1 | wide-m4/24 | 85 µs ✓ | 92 µs ✓ | 1.08× |
| families | wide-m4 | classical auto j1 | wide-m4/256 | 3.2 ms ✓ | 2.5 ms ✓ | 0.78× |
| families | wide-m4 | classical auto j1 | wide-m4/28 | 104 µs ✓ | 97 µs ✓ | 0.93× |
| families | wide-m4 | classical auto j1 | wide-m4/32 | 119 µs ✓ | 108 µs ✓ | 0.91× |
| families | wide-m4 | classical auto j1 | wide-m4/36 | 137 µs ✓ | 134 µs ✓ | 0.98× |
| intuitionistic | 3-partition-no | intuitionistic auto j1 | 3-partition-no/4 | 293 µs ✗ | 273 µs ✗ | 0.93× |
| intuitionistic | 3-partition-yes | intuitionistic auto j1 | 3-partition-yes/12 | 51 µs ✓ | 59 µs ✓ | 1.16× |
| intuitionistic | 3-partition-yes | intuitionistic auto j1 | 3-partition-yes/4 | 40 µs ✓ | 60 µs ✓ | 1.50× |
| intuitionistic | 3-partition-yes | intuitionistic auto j1 | 3-partition-yes/6 | 46 µs ✓ | 59 µs ✓ | 1.28× |
| intuitionistic | 3-partition-yes | intuitionistic auto j1 | 3-partition-yes/8 | 47 µs ✓ | 58 µs ✓ | 1.23× |
| intuitionistic | chain | intuitionistic auto j1 | chain/128 | 42.4 ms ✓ | 35.6 ms ✓ | 0.84× |
| intuitionistic | chain | intuitionistic auto j1 | chain/16 | 159 µs ✓ | 149 µs ✓ | 0.94× |
| intuitionistic | chain | intuitionistic auto j1 | chain/256 | 347.4 ms ✓ | 287.5 ms ✓ | 0.83× |
| intuitionistic | chain | intuitionistic auto j1 | chain/64 | 5.4 ms ✓ | 4.6 ms ✓ | 0.85× |
| intuitionistic | counter | intuitionistic auto j1 | counter/16 | 251 µs ✓ | 35 µs ✓ | 0.14× |
| intuitionistic | counter | intuitionistic auto j1 | counter/2 | 50 µs ✓ | 27 µs ✓ | 0.54× |
| intuitionistic | counter | intuitionistic auto j1 | counter/32 | 19.9 ms ✓ | 42 µs ✓ | 2.1e-3× |
| intuitionistic | counter | intuitionistic auto j1 | counter/4 | 61 µs ✓ | 28 µs ✓ | 0.46× |
| intuitionistic | counter | intuitionistic auto j1 | counter/64 | 267.7 ms ✓ | 53 µs ✓ | 2.0e-4× |
| intuitionistic | counter | intuitionistic auto j1 | counter/8 | 88 µs ✓ | 32 µs ✓ | 0.36× |
| intuitionistic | counter-over | intuitionistic auto j1 | counter-over/16 | 306 µs ✗ | 35 µs ✗ | 0.11× |
| intuitionistic | counter-over | intuitionistic auto j1 | counter-over/2 | 54 µs ✗ | 22 µs ✗ | 0.41× |
| intuitionistic | counter-over | intuitionistic auto j1 | counter-over/32 | 20.3 ms ? bound | 44 µs ✗ |  |
| intuitionistic | counter-over | intuitionistic auto j1 | counter-over/4 | 66 µs ✗ | 26 µs ✗ | 0.39× |
| intuitionistic | counter-over | intuitionistic auto j1 | counter-over/64 | 268.5 ms ? bound | 54 µs ✗ |  |
| intuitionistic | counter-over | intuitionistic auto j1 | counter-over/8 | 107 µs ✗ | 32 µs ✗ | 0.30× |
| long-1 | 3-partition-no | classical auto j1 | 3-partition-no/5 | 393 µs ✗ | 400 µs ✗ | 1.02× |
| long-1 | counter | classical auto j1 | counter/16 | 280 µs ✓ | 48 µs ✓ | 0.17× |
| long-1 | counter | classical auto j1 | counter/64 | 85.14 s ✓ | 60 µs ✓ | 7.0e-7× |
| long-1 | partition-no | classical auto j1 | partition-no/15 | > 1200 s | > 1200 s |  |
| long-1 | partition-no | classical auto j1 | partition-no/5 | 477 µs ✗ | 441 µs ✗ | 0.92× |
| long-1 | partition-yes | classical auto j1 | partition-yes/28 | > 1200 s | > 1200 s |  |
| long-1 | partition-yes | classical auto j1 | partition-yes/7 | 230 µs ✓ | 214 µs ✓ | 0.93× |
| long-2 | mix | mix auto j1 | mix/11 | > 1200 s | > 1200 s |  |
| long-2 | qbf | classical auto j1 | qbf/24#0 | 161.2 ms ✗ | 99.6 ms ✗ | 0.62× |
| long-2 | qbf | classical auto j1 | qbf/48#0 | 301.01 s unknown | 83.81 s ? memory |  |
| long-2 | wide-m3 | classical auto j1 | wide-m3/36 | 157 µs ✓ | 154 µs ✓ | 0.98× |
| long-2 | wide-m4 | classical auto j1 | wide-m4/36 | 144 µs ✓ | 135 µs ✓ | 0.94× |
| parallel | 3-partition-no | classical auto j1 | 3-partition-no/4 | 395 µs ✗ | 379 µs ✗ | 0.96× |
| parallel | 3-partition-no | classical auto j1 | 3-partition-no/5 | 470 µs ✗ | 432 µs ✗ | 0.92× |
| parallel | 3-partition-no | classical auto j16 | 3-partition-no/4 | 720 µs ✗ | 631 µs ✗ | 0.88× |
| parallel | 3-partition-no | classical auto j16 | 3-partition-no/5 | 689 µs ✗ | 674 µs ✗ | 0.98× |
| parallel | 3-partition-no | classical auto j2 | 3-partition-no/4 | 439 µs ✗ | 544 µs ✗ | 1.24× |
| parallel | 3-partition-no | classical auto j2 | 3-partition-no/5 | 483 µs ✗ | 567 µs ✗ | 1.17× |
| parallel | 3-partition-no | classical auto j4 | 3-partition-no/4 | 342 µs ✗ | 428 µs ✗ | 1.25× |
| parallel | 3-partition-no | classical auto j4 | 3-partition-no/5 | 354 µs ✗ | 461 µs ✗ | 1.30× |
| parallel | 3-partition-no | classical auto j8 | 3-partition-no/4 | 467 µs ✗ | 550 µs ✗ | 1.18× |
| parallel | 3-partition-no | classical auto j8 | 3-partition-no/5 | 431 µs ✗ | 562 µs ✗ | 1.30× |
| parallel | 3-partition-yes | classical auto j1 | 3-partition-yes/12 | 59 µs ✓ | 88 µs ✓ | 1.49× |
| parallel | 3-partition-yes | classical auto j1 | 3-partition-yes/4 | 48 µs ✓ | 103 µs ✓ | 2.15× |
| parallel | 3-partition-yes | classical auto j1 | 3-partition-yes/6 | 58 µs ✓ | 86 µs ✓ | 1.48× |
| parallel | 3-partition-yes | classical auto j1 | 3-partition-yes/8 | 54 µs ✓ | 86 µs ✓ | 1.59× |
| parallel | 3-partition-yes | classical auto j16 | 3-partition-yes/12 | 540 µs ✓ | 546 µs ✓ | 1.01× |
| parallel | 3-partition-yes | classical auto j16 | 3-partition-yes/4 | 506 µs ✓ | 591 µs ✓ | 1.17× |
| parallel | 3-partition-yes | classical auto j16 | 3-partition-yes/6 | 452 µs ✓ | 593 µs ✓ | 1.31× |
| parallel | 3-partition-yes | classical auto j16 | 3-partition-yes/8 | 601 µs ✓ | 649 µs ✓ | 1.08× |
| parallel | 3-partition-yes | classical auto j2 | 3-partition-yes/12 | 163 µs ✓ | 248 µs ✓ | 1.52× |
| parallel | 3-partition-yes | classical auto j2 | 3-partition-yes/4 | 209 µs ✓ | 215 µs ✓ | 1.03× |
| parallel | 3-partition-yes | classical auto j2 | 3-partition-yes/6 | 142 µs ✓ | 220 µs ✓ | 1.55× |
| parallel | 3-partition-yes | classical auto j2 | 3-partition-yes/8 | 164 µs ✓ | 263 µs ✓ | 1.60× |
| parallel | 3-partition-yes | classical auto j4 | 3-partition-yes/12 | 202 µs ✓ | 301 µs ✓ | 1.49× |
| parallel | 3-partition-yes | classical auto j4 | 3-partition-yes/4 | 198 µs ✓ | 247 µs ✓ | 1.25× |
| parallel | 3-partition-yes | classical auto j4 | 3-partition-yes/6 | 179 µs ✓ | 269 µs ✓ | 1.50× |
| parallel | 3-partition-yes | classical auto j4 | 3-partition-yes/8 | 183 µs ✓ | 278 µs ✓ | 1.52× |
| parallel | 3-partition-yes | classical auto j8 | 3-partition-yes/12 | 308 µs ✓ | 458 µs ✓ | 1.49× |
| parallel | 3-partition-yes | classical auto j8 | 3-partition-yes/4 | 328 µs ✓ | 416 µs ✓ | 1.27× |
| parallel | 3-partition-yes | classical auto j8 | 3-partition-yes/6 | 287 µs ✓ | 337 µs ✓ | 1.17× |
| parallel | 3-partition-yes | classical auto j8 | 3-partition-yes/8 | 352 µs ✓ | 423 µs ✓ | 1.20× |
| parallel | counter | classical auto j1 | counter/16 | 308 µs ✓ | 63 µs ✓ | 0.20× |
| parallel | counter | classical auto j1 | counter/2 | 65 µs ✓ | 38 µs ✓ | 0.58× |
| parallel | counter | classical auto j1 | counter/32 | 2.18 s ✓ | 72 µs ✓ | 3.3e-5× |
| parallel | counter | classical auto j1 | counter/4 | 66 µs ✓ | 40 µs ✓ | 0.61× |
| parallel | counter | classical auto j1 | counter/64 | 78.45 s ✓ | 86 µs ✓ | 1.1e-6× |
| parallel | counter | classical auto j1 | counter/8 | 98 µs ✓ | 51 µs ✓ | 0.52× |
| parallel | counter | classical auto j16 | counter/16 | 1.2 ms ✓ | 91 µs ✓ | 7.5e-2× |
| parallel | counter | classical auto j16 | counter/2 | 465 µs ✓ | 80 µs ✓ | 0.17× |
| parallel | counter | classical auto j16 | counter/32 | 700.0 ms ✓ | 102 µs ✓ | 1.5e-4× |
| parallel | counter | classical auto j16 | counter/4 | 559 µs ✓ | 71 µs ✓ | 0.13× |
| parallel | counter | classical auto j16 | counter/64 | 23.90 s ✓ | 116 µs ✓ | 4.9e-6× |
| parallel | counter | classical auto j16 | counter/8 | 691 µs ✓ | 81 µs ✓ | 0.12× |
| parallel | counter | classical auto j2 | counter/16 | 389 µs ✓ | 83 µs ✓ | 0.21× |
| parallel | counter | classical auto j2 | counter/2 | 109 µs ✓ | 101 µs ✓ | 0.93× |
| parallel | counter | classical auto j2 | counter/32 | 2.70 s ✓ | 94 µs ✓ | 3.5e-5× |
| parallel | counter | classical auto j2 | counter/4 | 99 µs ✓ | 73 µs ✓ | 0.74× |
| parallel | counter | classical auto j2 | counter/64 | 101.90 s ✓ | 106 µs ✓ | 1.0e-6× |
| parallel | counter | classical auto j2 | counter/8 | 138 µs ✓ | 79 µs ✓ | 0.57× |
| parallel | counter | classical auto j4 | counter/16 | 648 µs ✓ | 93 µs ✓ | 0.14× |
| parallel | counter | classical auto j4 | counter/2 | 156 µs ✓ | 82 µs ✓ | 0.53× |
| parallel | counter | classical auto j4 | counter/32 | 2.02 s ✓ | 96 µs ✓ | 4.7e-5× |
| parallel | counter | classical auto j4 | counter/4 | 161 µs ✓ | 83 µs ✓ | 0.52× |
| parallel | counter | classical auto j4 | counter/64 | 76.89 s ✓ | 102 µs ✓ | 1.3e-6× |
| parallel | counter | classical auto j4 | counter/8 | 300 µs ✓ | 78 µs ✓ | 0.26× |
| parallel | counter | classical auto j8 | counter/16 | 747 µs ✓ | 93 µs ✓ | 0.12× |
| parallel | counter | classical auto j8 | counter/2 | 243 µs ✓ | 63 µs ✓ | 0.26× |
| parallel | counter | classical auto j8 | counter/32 | 629.7 ms ✓ | 100 µs ✓ | 1.6e-4× |
| parallel | counter | classical auto j8 | counter/4 | 274 µs ✓ | 75 µs ✓ | 0.27× |
| parallel | counter | classical auto j8 | counter/64 | 22.55 s ✓ | 114 µs ✓ | 5.1e-6× |
| parallel | counter | classical auto j8 | counter/8 | 419 µs ✓ | 83 µs ✓ | 0.20× |
| parallel | counter-over | classical auto j1 | counter-over/16 | 403 µs ✗ | 63 µs ✗ | 0.16× |
| parallel | counter-over | classical auto j1 | counter-over/2 | 65 µs ✗ | 49 µs ✗ | 0.75× |
| parallel | counter-over | classical auto j1 | counter-over/32 | 2.28 s ? bound | 73 µs ✗ |  |
| parallel | counter-over | classical auto j1 | counter-over/4 | 77 µs ✗ | 40 µs ✗ | 0.52× |
| parallel | counter-over | classical auto j1 | counter-over/64 | 82.99 s ? bound | 70 µs ✗ |  |
| parallel | counter-over | classical auto j1 | counter-over/8 | 116 µs ✗ | 55 µs ✗ | 0.47× |
| parallel | counter-over | classical auto j16 | counter-over/16 | 1.2 ms ✗ | 86 µs ✗ | 7.0e-2× |
| parallel | counter-over | classical auto j16 | counter-over/2 | 510 µs ✗ | 68 µs ✗ | 0.13× |
| parallel | counter-over | classical auto j16 | counter-over/32 | 2.87 s ? bound | 95 µs ✗ |  |
| parallel | counter-over | classical auto j16 | counter-over/4 | 568 µs ✗ | 76 µs ✗ | 0.13× |
| parallel | counter-over | classical auto j16 | counter-over/64 | 103.02 s ? bound | 98 µs ✗ |  |
| parallel | counter-over | classical auto j16 | counter-over/8 | 759 µs ✗ | 73 µs ✗ | 9.6e-2× |
| parallel | counter-over | classical auto j2 | counter-over/16 | 430 µs ✗ | 84 µs ✗ | 0.20× |
| parallel | counter-over | classical auto j2 | counter-over/2 | 93 µs ✗ | 72 µs ✗ | 0.77× |
| parallel | counter-over | classical auto j2 | counter-over/32 | 3.15 s ? bound | 96 µs ✗ |  |
| parallel | counter-over | classical auto j2 | counter-over/4 | 128 µs ✗ | 77 µs ✗ | 0.60× |
| parallel | counter-over | classical auto j2 | counter-over/64 | 112.73 s ? bound | 98 µs ✗ |  |
| parallel | counter-over | classical auto j2 | counter-over/8 | 165 µs ✗ | 76 µs ✗ | 0.46× |
| parallel | counter-over | classical auto j4 | counter-over/16 | 731 µs ✗ | 77 µs ✗ | 0.11× |
| parallel | counter-over | classical auto j4 | counter-over/2 | 150 µs ✗ | 72 µs ✗ | 0.48× |
| parallel | counter-over | classical auto j4 | counter-over/32 | 2.52 s ? bound | 94 µs ✗ |  |
| parallel | counter-over | classical auto j4 | counter-over/4 | 171 µs ✗ | 68 µs ✗ | 0.40× |
| parallel | counter-over | classical auto j4 | counter-over/64 | 86.58 s ? bound | 88 µs ✗ |  |
| parallel | counter-over | classical auto j4 | counter-over/8 | 274 µs ✗ | 73 µs ✗ | 0.27× |
| parallel | counter-over | classical auto j8 | counter-over/16 | 1.2 ms ✗ | 80 µs ✗ | 6.7e-2× |
| parallel | counter-over | classical auto j8 | counter-over/2 | 240 µs ✗ | 78 µs ✗ | 0.33× |
| parallel | counter-over | classical auto j8 | counter-over/32 | 2.72 s ? bound | 94 µs ✗ |  |
| parallel | counter-over | classical auto j8 | counter-over/4 | 268 µs ✗ | 68 µs ✗ | 0.25× |
| parallel | counter-over | classical auto j8 | counter-over/64 | 95.97 s ? bound | 91 µs ✗ |  |
| parallel | counter-over | classical auto j8 | counter-over/8 | 454 µs ✗ | 87 µs ✗ | 0.19× |
| parallel | mix | mix auto j1 | mix/10 | > 120 s | 62.10 s ✗ |  |
| parallel | mix | mix auto j1 | mix/11 | > 120 s | > 120 s |  |
| parallel | mix | mix auto j1 | mix/8 | 1.56 s ✗ | 857.8 ms ✗ | 0.55× |
| parallel | mix | mix auto j1 | mix/9 | 13.94 s ✗ | 7.39 s ✗ | 0.53× |
| parallel | mix | mix auto j16 | mix/10 | > 120 s | > 120 s |  |
| parallel | mix | mix auto j16 | mix/11 | > 120 s | > 120 s |  |
| parallel | mix | mix auto j16 | mix/8 | 2.43 s ✗ | 1.61 s ✗ | 0.66× |
| parallel | mix | mix auto j16 | mix/9 | 21.18 s ✗ | 11.51 s ✗ | 0.54× |
| parallel | mix | mix auto j2 | mix/10 | > 120 s | 91.83 s ✗ |  |
| parallel | mix | mix auto j2 | mix/11 | > 120 s | > 120 s |  |
| parallel | mix | mix auto j2 | mix/8 | 1.87 s ✗ | 965.2 ms ✗ | 0.52× |
| parallel | mix | mix auto j2 | mix/9 | 17.40 s ✗ | 7.05 s ✗ | 0.40× |
| parallel | mix | mix auto j4 | mix/10 | > 120 s | 95.53 s ✗ |  |
| parallel | mix | mix auto j4 | mix/11 | > 120 s | > 120 s |  |
| parallel | mix | mix auto j4 | mix/8 | 1.91 s ✗ | 1.05 s ✗ | 0.55× |
| parallel | mix | mix auto j4 | mix/9 | 17.88 s ✗ | 7.90 s ✗ | 0.44× |
| parallel | mix | mix auto j8 | mix/10 | > 120 s | 109.66 s ✗ |  |
| parallel | mix | mix auto j8 | mix/11 | > 120 s | > 120 s |  |
| parallel | mix | mix auto j8 | mix/8 | 2.07 s ✗ | 1.48 s ✗ | 0.72× |
| parallel | mix | mix auto j8 | mix/9 | 18.57 s ✗ | 9.50 s ✗ | 0.51× |
| parallel | partition-no | classical auto j1 | partition-no/12 | 4.01 s ✗ | 3.59 s ✗ | 0.90× |
| parallel | partition-no | classical auto j1 | partition-no/14 | > 120 s | > 120 s |  |
| parallel | partition-no | classical auto j1 | partition-no/4 | 170 µs ✗ | 192 µs ✗ | 1.13× |
| parallel | partition-no | classical auto j1 | partition-no/5 | 604 µs ✗ | 568 µs ✗ | 0.94× |
| parallel | partition-no | classical auto j16 | partition-no/12 | 847.6 ms ✗ | 581.6 ms ✗ | 0.69× |
| parallel | partition-no | classical auto j16 | partition-no/14 | > 120 s | > 120 s |  |
| parallel | partition-no | classical auto j16 | partition-no/4 | 721 µs ✗ | 600 µs ✗ | 0.83× |
| parallel | partition-no | classical auto j16 | partition-no/5 | 840 µs ✗ | 850 µs ✗ | 1.01× |
| parallel | partition-no | classical auto j2 | partition-no/12 | 2.83 s ✗ | 2.13 s ✗ | 0.75× |
| parallel | partition-no | classical auto j2 | partition-no/14 | > 120 s | > 120 s |  |
| parallel | partition-no | classical auto j2 | partition-no/4 | 272 µs ✗ | 405 µs ✗ | 1.49× |
| parallel | partition-no | classical auto j2 | partition-no/5 | 681 µs ✗ | 745 µs ✗ | 1.09× |
| parallel | partition-no | classical auto j4 | partition-no/12 | 1.73 s ✗ | 1.24 s ✗ | 0.72× |
| parallel | partition-no | classical auto j4 | partition-no/14 | > 120 s | > 120 s |  |
| parallel | partition-no | classical auto j4 | partition-no/4 | 242 µs ✗ | 347 µs ✗ | 1.43× |
| parallel | partition-no | classical auto j4 | partition-no/5 | 518 µs ✗ | 578 µs ✗ | 1.12× |
| parallel | partition-no | classical auto j8 | partition-no/12 | 1.20 s ✗ | 806.0 ms ✗ | 0.67× |
| parallel | partition-no | classical auto j8 | partition-no/14 | > 120 s | > 120 s |  |
| parallel | partition-no | classical auto j8 | partition-no/4 | 392 µs ✗ | 418 µs ✗ | 1.07× |
| parallel | partition-no | classical auto j8 | partition-no/5 | 501 µs ✗ | 614 µs ✗ | 1.23× |
| parallel | partition-yes | classical auto j1 | partition-yes/20 | 535.5 ms ✓ | 396.5 ms ✓ | 0.74× |
| parallel | partition-yes | classical auto j1 | partition-yes/24 | 105.53 s ✓ | 67.45 s ✓ | 0.64× |
| parallel | partition-yes | classical auto j1 | partition-yes/5 | 97 µs ✓ | 117 µs ✓ | 1.21× |
| parallel | partition-yes | classical auto j1 | partition-yes/6 | 167 µs ✓ | 187 µs ✓ | 1.12× |
| parallel | partition-yes | classical auto j1 | partition-yes/7 | 318 µs ✓ | 273 µs ✓ | 0.86× |
| parallel | partition-yes | classical auto j16 | partition-yes/20 | 651.0 ms ✓ | 445.7 ms ✓ | 0.68× |
| parallel | partition-yes | classical auto j16 | partition-yes/24 | > 120 s | 87.42 s ✓ |  |
| parallel | partition-yes | classical auto j16 | partition-yes/5 | 703 µs ✓ | 851 µs ✓ | 1.21× |
| parallel | partition-yes | classical auto j16 | partition-yes/6 | 743 µs ✓ | 906 µs ✓ | 1.22× |
| parallel | partition-yes | classical auto j16 | partition-yes/7 | 849 µs ✓ | 750 µs ✓ | 0.88× |
| parallel | partition-yes | classical auto j2 | partition-yes/20 | 586.4 ms ✓ | 418.5 ms ✓ | 0.71× |
| parallel | partition-yes | classical auto j2 | partition-yes/24 | > 120 s | 70.81 s ✓ |  |
| parallel | partition-yes | classical auto j2 | partition-yes/5 | 207 µs ✓ | 311 µs ✓ | 1.50× |
| parallel | partition-yes | classical auto j2 | partition-yes/6 | 280 µs ✓ | 461 µs ✓ | 1.65× |
| parallel | partition-yes | classical auto j2 | partition-yes/7 | 447 µs ✓ | 585 µs ✓ | 1.31× |
| parallel | partition-yes | classical auto j4 | partition-yes/20 | 608.0 ms ✓ | 429.9 ms ✓ | 0.71× |
| parallel | partition-yes | classical auto j4 | partition-yes/24 | > 120 s | 74.75 s ✓ |  |
| parallel | partition-yes | classical auto j4 | partition-yes/5 | 260 µs ✓ | 334 µs ✓ | 1.28× |
| parallel | partition-yes | classical auto j4 | partition-yes/6 | 287 µs ✓ | 413 µs ✓ | 1.44× |
| parallel | partition-yes | classical auto j4 | partition-yes/7 | 498 µs ✓ | 621 µs ✓ | 1.25× |
| parallel | partition-yes | classical auto j8 | partition-yes/20 | 634.9 ms ✓ | 439.9 ms ✓ | 0.69× |
| parallel | partition-yes | classical auto j8 | partition-yes/24 | > 120 s | 79.07 s ✓ |  |
| parallel | partition-yes | classical auto j8 | partition-yes/5 | 399 µs ✓ | 461 µs ✓ | 1.16× |
| parallel | partition-yes | classical auto j8 | partition-yes/6 | 450 µs ✓ | 546 µs ✓ | 1.21× |
| parallel | partition-yes | classical auto j8 | partition-yes/7 | 638 µs ✓ | 597 µs ✓ | 0.94× |
| parallel | qbf | classical auto j1 | qbf/16#0 | 13.1 ms ✓ | 7.1 ms ✓ | 0.54× |
| parallel | qbf | classical auto j1 | qbf/16#1 | 1.9 ms ✗ | 1.8 ms ✗ | 0.97× |
| parallel | qbf | classical auto j1 | qbf/16#2 | 3.3 ms ✗ | 2.6 ms ✗ | 0.78× |
| parallel | qbf | classical auto j1 | qbf/16#3 | 10.3 ms ✓ | 8.4 ms ✓ | 0.81× |
| parallel | qbf | classical auto j1 | qbf/20#0 | 9.4 ms ✗ | 8.4 ms ✗ | 0.90× |
| parallel | qbf | classical auto j1 | qbf/20#1 | 16.1 ms ✗ | 14.3 ms ✗ | 0.89× |
| parallel | qbf | classical auto j1 | qbf/20#2 | 60.2 ms ✓ | 53.3 ms ✓ | 0.89× |
| parallel | qbf | classical auto j1 | qbf/20#3 | 9.2 ms ✗ | 8.1 ms ✗ | 0.88× |
| parallel | qbf | classical auto j1 | qbf/24#0 | 169.3 ms ✗ | 142.0 ms ✗ | 0.84× |
| parallel | qbf | classical auto j1 | qbf/24#1 | 62.6 ms ✗ | 68.0 ms ✗ | 1.09× |
| parallel | qbf | classical auto j1 | qbf/24#2 | 44.7 ms ✗ | 47.6 ms ✗ | 1.06× |
| parallel | qbf | classical auto j1 | qbf/24#3 | 62.2 ms ✗ | 57.9 ms ✗ | 0.93× |
| parallel | qbf | classical auto j1 | qbf/40#0 | 16.61 s ✗ | 13.75 s ✗ | 0.83× |
| parallel | qbf | classical auto j1 | qbf/40#1 | 18.54 s ✗ | 13.41 s ✗ | 0.72× |
| parallel | qbf | classical auto j1 | qbf/40#2 | 21.36 s ✗ | 15.46 s ✗ | 0.72× |
| parallel | qbf | classical auto j1 | qbf/40#3 | 16.91 s ✗ | 11.55 s ✗ | 0.68× |
| parallel | qbf | classical auto j1 | qbf/44#0 | 72.46 s ✗ | 50.45 s ✗ | 0.70× |
| parallel | qbf | classical auto j1 | qbf/44#1 | 72.68 s ✗ | 49.78 s ✗ | 0.68× |
| parallel | qbf | classical auto j1 | qbf/44#2 | 108.46 s ✗ | 73.42 s ✗ | 0.68× |
| parallel | qbf | classical auto j1 | qbf/44#3 | > 120 s | 26.33 s ? memory |  |
| parallel | qbf | classical auto j16 | qbf/16#0 | 14.9 ms ✓ | 11.3 ms ✓ | 0.76× |
| parallel | qbf | classical auto j16 | qbf/16#1 | 2.4 ms ✗ | 2.0 ms ✗ | 0.80× |
| parallel | qbf | classical auto j16 | qbf/16#2 | 3.4 ms ✗ | 3.5 ms ✗ | 1.04× |
| parallel | qbf | classical auto j16 | qbf/16#3 | 13.2 ms ✓ | 9.5 ms ✓ | 0.72× |
| parallel | qbf | classical auto j16 | qbf/20#0 | 7.8 ms ✗ | 6.7 ms ✗ | 0.86× |
| parallel | qbf | classical auto j16 | qbf/20#1 | 12.4 ms ✗ | 13.7 ms ✗ | 1.10× |
| parallel | qbf | classical auto j16 | qbf/20#2 | 103.9 ms ✓ | 80.4 ms ✓ | 0.77× |
| parallel | qbf | classical auto j16 | qbf/20#3 | 7.5 ms ✗ | 5.2 ms ✗ | 0.69× |
| parallel | qbf | classical auto j16 | qbf/24#0 | 214.4 ms ✗ | 166.2 ms ✗ | 0.78× |
| parallel | qbf | classical auto j16 | qbf/24#1 | 60.5 ms ✗ | 45.5 ms ✗ | 0.75× |
| parallel | qbf | classical auto j16 | qbf/24#2 | 40.2 ms ✗ | 36.5 ms ✗ | 0.91× |
| parallel | qbf | classical auto j16 | qbf/24#3 | 58.9 ms ✗ | 43.1 ms ✗ | 0.73× |
| parallel | qbf | classical auto j16 | qbf/40#0 | 14.33 s ✗ | 7.47 s ✗ | 0.52× |
| parallel | qbf | classical auto j16 | qbf/40#1 | 15.24 s ✗ | 5.53 s ? memory |  |
| parallel | qbf | classical auto j16 | qbf/40#2 | 17.72 s ✗ | 5.81 s ? memory |  |
| parallel | qbf | classical auto j16 | qbf/40#3 | 15.47 s ✗ | 7.51 s ✗ | 0.49× |
| parallel | qbf | classical auto j16 | qbf/44#0 | 54.98 s ✗ | 8.78 s ? memory |  |
| parallel | qbf | classical auto j16 | qbf/44#1 | 59.98 s ✗ | 7.42 s ? memory |  |
| parallel | qbf | classical auto j16 | qbf/44#2 | 96.80 s ✗ | 5.14 s ? memory |  |
| parallel | qbf | classical auto j16 | qbf/44#3 | > 120 s | 8.73 s ? memory |  |
| parallel | qbf | classical auto j2 | qbf/16#0 | 14.7 ms ✓ | 10.4 ms ✓ | 0.71× |
| parallel | qbf | classical auto j2 | qbf/16#1 | 2.1 ms ✗ | 2.5 ms ✗ | 1.20× |
| parallel | qbf | classical auto j2 | qbf/16#2 | 3.6 ms ✗ | 2.7 ms ✗ | 0.74× |
| parallel | qbf | classical auto j2 | qbf/16#3 | 12.3 ms ✓ | 7.3 ms ✓ | 0.60× |
| parallel | qbf | classical auto j2 | qbf/20#0 | 9.0 ms ✗ | 7.4 ms ✗ | 0.83× |
| parallel | qbf | classical auto j2 | qbf/20#1 | 14.7 ms ✗ | 11.0 ms ✗ | 0.75× |
| parallel | qbf | classical auto j2 | qbf/20#2 | 103.0 ms ✓ | 64.0 ms ✓ | 0.62× |
| parallel | qbf | classical auto j2 | qbf/20#3 | 7.7 ms ✗ | 7.8 ms ✗ | 1.01× |
| parallel | qbf | classical auto j2 | qbf/24#0 | 201.0 ms ✗ | 126.2 ms ✗ | 0.63× |
| parallel | qbf | classical auto j2 | qbf/24#1 | 58.6 ms ✗ | 37.2 ms ✗ | 0.63× |
| parallel | qbf | classical auto j2 | qbf/24#2 | 43.7 ms ✗ | 30.8 ms ✗ | 0.70× |
| parallel | qbf | classical auto j2 | qbf/24#3 | 58.2 ms ✗ | 39.2 ms ✗ | 0.67× |
| parallel | qbf | classical auto j2 | qbf/40#0 | 14.07 s ✗ | 8.61 s ✗ | 0.61× |
| parallel | qbf | classical auto j2 | qbf/40#1 | 15.30 s ✗ | 5.56 s ? memory |  |
| parallel | qbf | classical auto j2 | qbf/40#2 | 17.69 s ✗ | 5.80 s ? memory |  |
| parallel | qbf | classical auto j2 | qbf/40#3 | 15.63 s ✗ | 7.46 s ✗ | 0.48× |
| parallel | qbf | classical auto j2 | qbf/44#0 | 54.94 s ✗ | 8.79 s ? memory |  |
| parallel | qbf | classical auto j2 | qbf/44#1 | 60.47 s ✗ | 7.31 s ? memory |  |
| parallel | qbf | classical auto j2 | qbf/44#2 | 97.70 s ✗ | 5.14 s ? memory |  |
| parallel | qbf | classical auto j2 | qbf/44#3 | > 120 s | 8.56 s ? memory |  |
| parallel | qbf | classical auto j4 | qbf/16#0 | 14.3 ms ✓ | 9.6 ms ✓ | 0.67× |
| parallel | qbf | classical auto j4 | qbf/16#1 | 1.7 ms ✗ | 1.6 ms ✗ | 0.95× |
| parallel | qbf | classical auto j4 | qbf/16#2 | 3.7 ms ✗ | 2.9 ms ✗ | 0.80× |
| parallel | qbf | classical auto j4 | qbf/16#3 | 12.3 ms ✓ | 8.5 ms ✓ | 0.69× |
| parallel | qbf | classical auto j4 | qbf/20#0 | 8.7 ms ✗ | 7.5 ms ✗ | 0.86× |
| parallel | qbf | classical auto j4 | qbf/20#1 | 15.0 ms ✗ | 10.0 ms ✗ | 0.67× |
| parallel | qbf | classical auto j4 | qbf/20#2 | 100.4 ms ✓ | 68.3 ms ✓ | 0.68× |
| parallel | qbf | classical auto j4 | qbf/20#3 | 8.0 ms ✗ | 6.5 ms ✗ | 0.82× |
| parallel | qbf | classical auto j4 | qbf/24#0 | 194.5 ms ✗ | 128.5 ms ✗ | 0.66× |
| parallel | qbf | classical auto j4 | qbf/24#1 | 59.7 ms ✗ | 38.4 ms ✗ | 0.64× |
| parallel | qbf | classical auto j4 | qbf/24#2 | 37.6 ms ✗ | 29.7 ms ✗ | 0.79× |
| parallel | qbf | classical auto j4 | qbf/24#3 | 58.6 ms ✗ | 37.5 ms ✗ | 0.64× |
| parallel | qbf | classical auto j4 | qbf/40#0 | 13.95 s ✗ | 8.46 s ✗ | 0.61× |
| parallel | qbf | classical auto j4 | qbf/40#1 | 15.25 s ✗ | 5.53 s ? memory |  |
| parallel | qbf | classical auto j4 | qbf/40#2 | 17.64 s ✗ | 5.84 s ? memory |  |
| parallel | qbf | classical auto j4 | qbf/40#3 | 15.46 s ✗ | 7.57 s ✗ | 0.49× |
| parallel | qbf | classical auto j4 | qbf/44#0 | 55.96 s ✗ | 8.71 s ? memory |  |
| parallel | qbf | classical auto j4 | qbf/44#1 | 60.23 s ✗ | 7.44 s ? memory |  |
| parallel | qbf | classical auto j4 | qbf/44#2 | 97.99 s ✗ | 5.13 s ? memory |  |
| parallel | qbf | classical auto j4 | qbf/44#3 | > 120 s | 8.59 s ? memory |  |
| parallel | qbf | classical auto j8 | qbf/16#0 | 14.0 ms ✓ | 10.3 ms ✓ | 0.73× |
| parallel | qbf | classical auto j8 | qbf/16#1 | 1.9 ms ✗ | 3.0 ms ✗ | 1.59× |
| parallel | qbf | classical auto j8 | qbf/16#2 | 3.3 ms ✗ | 2.7 ms ✗ | 0.82× |
| parallel | qbf | classical auto j8 | qbf/16#3 | 12.6 ms ✓ | 11.2 ms ✓ | 0.88× |
| parallel | qbf | classical auto j8 | qbf/20#0 | 8.6 ms ✗ | 5.8 ms ✗ | 0.68× |
| parallel | qbf | classical auto j8 | qbf/20#1 | 13.8 ms ✗ | 11.2 ms ✗ | 0.81× |
| parallel | qbf | classical auto j8 | qbf/20#2 | 100.1 ms ✓ | 71.9 ms ✓ | 0.72× |
| parallel | qbf | classical auto j8 | qbf/20#3 | 7.0 ms ✗ | 8.6 ms ✗ | 1.23× |
| parallel | qbf | classical auto j8 | qbf/24#0 | 196.9 ms ✗ | 135.1 ms ✗ | 0.69× |
| parallel | qbf | classical auto j8 | qbf/24#1 | 58.5 ms ✗ | 43.4 ms ✗ | 0.74× |
| parallel | qbf | classical auto j8 | qbf/24#2 | 39.0 ms ✗ | 28.1 ms ✗ | 0.72× |
| parallel | qbf | classical auto j8 | qbf/24#3 | 58.8 ms ✗ | 38.6 ms ✗ | 0.66× |
| parallel | qbf | classical auto j8 | qbf/40#0 | 13.93 s ✗ | 7.84 s ✗ | 0.56× |
| parallel | qbf | classical auto j8 | qbf/40#1 | 15.32 s ✗ | 5.52 s ? memory |  |
| parallel | qbf | classical auto j8 | qbf/40#2 | 17.64 s ✗ | 5.82 s ? memory |  |
| parallel | qbf | classical auto j8 | qbf/40#3 | 15.53 s ✗ | 7.44 s ✗ | 0.48× |
| parallel | qbf | classical auto j8 | qbf/44#0 | 55.39 s ✗ | 8.78 s ? memory |  |
| parallel | qbf | classical auto j8 | qbf/44#1 | 59.90 s ✗ | 7.34 s ? memory |  |
| parallel | qbf | classical auto j8 | qbf/44#2 | 97.53 s ✗ | 5.14 s ? memory |  |
| parallel | qbf | classical auto j8 | qbf/44#3 | > 120 s | 8.70 s ? memory |  |
| parallel | wide-m3 | classical auto j1 | wide-m3/24 | 115 µs ✓ | 135 µs ✓ | 1.17× |
| parallel | wide-m3 | classical auto j1 | wide-m3/30 | 152 µs ✓ | 182 µs ✓ | 1.20× |
| parallel | wide-m3 | classical auto j1 | wide-m3/36 | 182 µs ✓ | 192 µs ✓ | 1.05× |
| parallel | wide-m3 | classical auto j16 | wide-m3/24 | 666 µs ✓ | 714 µs ✓ | 1.07× |
| parallel | wide-m3 | classical auto j16 | wide-m3/30 | 611 µs ✓ | 807 µs ✓ | 1.32× |
| parallel | wide-m3 | classical auto j16 | wide-m3/36 | 687 µs ✓ | 727 µs ✓ | 1.06× |
| parallel | wide-m3 | classical auto j2 | wide-m3/24 | 198 µs ✓ | 357 µs ✓ | 1.80× |
| parallel | wide-m3 | classical auto j2 | wide-m3/30 | 269 µs ✓ | 316 µs ✓ | 1.17× |
| parallel | wide-m3 | classical auto j2 | wide-m3/36 | 274 µs ✓ | 415 µs ✓ | 1.51× |
| parallel | wide-m3 | classical auto j4 | wide-m3/24 | 254 µs ✓ | 315 µs ✓ | 1.24× |
| parallel | wide-m3 | classical auto j4 | wide-m3/30 | 306 µs ✓ | 390 µs ✓ | 1.27× |
| parallel | wide-m3 | classical auto j4 | wide-m3/36 | 318 µs ✓ | 405 µs ✓ | 1.27× |
| parallel | wide-m3 | classical auto j8 | wide-m3/24 | 331 µs ✓ | 489 µs ✓ | 1.48× |
| parallel | wide-m3 | classical auto j8 | wide-m3/30 | 424 µs ✓ | 503 µs ✓ | 1.19× |
| parallel | wide-m3 | classical auto j8 | wide-m3/36 | 424 µs ✓ | 453 µs ✓ | 1.07× |
| parallel | wide-m4 | classical auto j1 | wide-m4/28 | 149 µs ✓ | 141 µs ✓ | 0.95× |
| parallel | wide-m4 | classical auto j1 | wide-m4/32 | 179 µs ✓ | 186 µs ✓ | 1.04× |
| parallel | wide-m4 | classical auto j1 | wide-m4/36 | 175 µs ✓ | 190 µs ✓ | 1.09× |
| parallel | wide-m4 | classical auto j16 | wide-m4/28 | 699 µs ✓ | 727 µs ✓ | 1.04× |
| parallel | wide-m4 | classical auto j16 | wide-m4/32 | 620 µs ✓ | 807 µs ✓ | 1.30× |
| parallel | wide-m4 | classical auto j16 | wide-m4/36 | 638 µs ✓ | 722 µs ✓ | 1.13× |
| parallel | wide-m4 | classical auto j2 | wide-m4/28 | 229 µs ✓ | 326 µs ✓ | 1.42× |
| parallel | wide-m4 | classical auto j2 | wide-m4/32 | 266 µs ✓ | 369 µs ✓ | 1.39× |
| parallel | wide-m4 | classical auto j2 | wide-m4/36 | 270 µs ✓ | 430 µs ✓ | 1.59× |
| parallel | wide-m4 | classical auto j4 | wide-m4/28 | 267 µs ✓ | 342 µs ✓ | 1.28× |
| parallel | wide-m4 | classical auto j4 | wide-m4/32 | 284 µs ✓ | 368 µs ✓ | 1.30× |
| parallel | wide-m4 | classical auto j4 | wide-m4/36 | 329 µs ✓ | 435 µs ✓ | 1.32× |
| parallel | wide-m4 | classical auto j8 | wide-m4/28 | 369 µs ✓ | 487 µs ✓ | 1.32× |
| parallel | wide-m4 | classical auto j8 | wide-m4/32 | 393 µs ✓ | 538 µs ✓ | 1.37× |
| parallel | wide-m4 | classical auto j8 | wide-m4/36 | 412 µs ✓ | 535 µs ✓ | 1.30× |
| parallel-net | 3-partition-mll-no | classical net j1 | 3-partition-mll-no/4 | 1.70 s ✗ | 1.63 s ✗ | 0.96× |
| parallel-net | 3-partition-mll-no | classical net j1 | 3-partition-mll-no/5 | 51.56 s ✗ | 48.79 s ✗ | 0.95× |
| parallel-net | 3-partition-mll-no | classical net j1 | 3-partition-mll-no/6 | > 60 s | > 60 s |  |
| parallel-net | 3-partition-mll-no | classical net j16 | 3-partition-mll-no/4 | 139.7 ms ✗ | 138.2 ms ✗ | 0.99× |
| parallel-net | 3-partition-mll-no | classical net j16 | 3-partition-mll-no/5 | 4.51 s ✗ | 4.42 s ✗ | 0.98× |
| parallel-net | 3-partition-mll-no | classical net j16 | 3-partition-mll-no/6 | > 60 s | > 60 s |  |
| parallel-net | 3-partition-mll-no | classical net j2 | 3-partition-mll-no/4 | 886.3 ms ✗ | 829.1 ms ✗ | 0.94× |
| parallel-net | 3-partition-mll-no | classical net j2 | 3-partition-mll-no/5 | 26.63 s ✗ | 25.09 s ✗ | 0.94× |
| parallel-net | 3-partition-mll-no | classical net j2 | 3-partition-mll-no/6 | > 60 s | > 60 s |  |
| parallel-net | 3-partition-mll-no | classical net j4 | 3-partition-mll-no/4 | 463.9 ms ✗ | 443.0 ms ✗ | 0.96× |
| parallel-net | 3-partition-mll-no | classical net j4 | 3-partition-mll-no/5 | 14.46 s ✗ | 13.41 s ✗ | 0.93× |
| parallel-net | 3-partition-mll-no | classical net j4 | 3-partition-mll-no/6 | > 60 s | > 60 s |  |
| parallel-net | 3-partition-mll-no | classical net j8 | 3-partition-mll-no/4 | 254.6 ms ✗ | 246.5 ms ✗ | 0.97× |
| parallel-net | 3-partition-mll-no | classical net j8 | 3-partition-mll-no/5 | 7.92 s ✗ | 7.58 s ✗ | 0.96× |
| parallel-net | 3-partition-mll-no | classical net j8 | 3-partition-mll-no/6 | > 60 s | > 60 s |  |
| parallel-net | partition-table | classical net j1 | partition-table/1-1 | 40 µs ✓ | 72 µs ✓ | 1.80× |
| parallel-net | partition-table | classical net j1 | partition-table/1-1-1-1-1-7 | > 60 s | > 60 s |  |
| parallel-net | partition-table | classical net j1 | partition-table/1-1-1-5 | 7.11 s ✗ | 7.13 s ✗ | 1.00× |
| parallel-net | partition-table | classical net j1 | partition-table/1-1-2-4 | 1.04 s ✓ | 1.01 s ✓ | 0.97× |
| parallel-net | partition-table | classical net j1 | partition-table/1-1-4 | 47.5 ms ✗ | 46.5 ms ✗ | 0.98× |
| parallel-net | partition-table | classical net j1 | partition-table/1-2-3-4-5-5 | > 60 s | > 60 s |  |
| parallel-net | partition-table | classical net j1 | partition-table/1-2-5 | 3.92 s ✗ | 3.66 s ✗ | 0.93× |
| parallel-net | partition-table | classical net j1 | partition-table/1-3 | 472 µs ✗ | 488 µs ✗ | 1.03× |
| parallel-net | partition-table | classical net j1 | partition-table/2-1-1 | 797 µs ✓ | 802 µs ✓ | 1.01× |
| parallel-net | partition-table | classical net j1 | partition-table/2-2-1-1 | 60.3 ms ✓ | 62.3 ms ✓ | 1.03× |
| parallel-net | partition-table | classical net j1 | partition-table/2-2-2-2-2-2-9-1 | > 60 s | > 60 s |  |
| parallel-net | partition-table | classical net j1 | partition-table/2-3-2-1 | 4.38 s ✓ | 4.24 s ✓ | 0.97× |
| parallel-net | partition-table | classical net j1 | partition-table/3-3-3-1 | > 60 s | > 60 s |  |
| parallel-net | partition-table | classical net j16 | partition-table/1-1 | 505 µs ✓ | 508 µs ✓ | 1.01× |
| parallel-net | partition-table | classical net j16 | partition-table/1-1-1-1-1-7 | > 60 s | > 60 s |  |
| parallel-net | partition-table | classical net j16 | partition-table/1-1-1-5 | 598.5 ms ✗ | 649.9 ms ✗ | 1.09× |
| parallel-net | partition-table | classical net j16 | partition-table/1-1-2-4 | 72.5 ms ✓ | 74.5 ms ✓ | 1.03× |
| parallel-net | partition-table | classical net j16 | partition-table/1-1-4 | 5.3 ms ✗ | 4.7 ms ✗ | 0.89× |
| parallel-net | partition-table | classical net j16 | partition-table/1-2-3-4-5-5 | > 60 s | > 60 s |  |
| parallel-net | partition-table | classical net j16 | partition-table/1-2-5 | 297.9 ms ✗ | 291.2 ms ✗ | 0.98× |
| parallel-net | partition-table | classical net j16 | partition-table/1-3 | 1.6 ms ✗ | 965 µs ✗ | 0.61× |
| parallel-net | partition-table | classical net j16 | partition-table/2-1-1 | 4.4 ms ✓ | 1.4 ms ✓ | 0.32× |
| parallel-net | partition-table | classical net j16 | partition-table/2-2-1-1 | 7.1 ms ✓ | 6.0 ms ✓ | 0.85× |
| parallel-net | partition-table | classical net j16 | partition-table/2-2-2-2-2-2-9-1 | 835.8 ms ✓ | 977.2 ms ✓ | 1.17× |
| parallel-net | partition-table | classical net j16 | partition-table/2-3-2-1 | 316.1 ms ✓ | 336.8 ms ✓ | 1.07× |
| parallel-net | partition-table | classical net j16 | partition-table/3-3-3-1 | > 60 s | > 60 s |  |
| parallel-net | partition-table | classical net j2 | partition-table/1-1 | 245 µs ✓ | 180 µs ✓ | 0.73× |
| parallel-net | partition-table | classical net j2 | partition-table/1-1-1-1-1-7 | > 60 s | > 60 s |  |
| parallel-net | partition-table | classical net j2 | partition-table/1-1-1-5 | 3.63 s ✗ | 3.49 s ✗ | 0.96× |
| parallel-net | partition-table | classical net j2 | partition-table/1-1-2-4 | 522.5 ms ✓ | 496.8 ms ✓ | 0.95× |
| parallel-net | partition-table | classical net j2 | partition-table/1-1-4 | 25.6 ms ✗ | 24.7 ms ✗ | 0.97× |
| parallel-net | partition-table | classical net j2 | partition-table/1-2-3-4-5-5 | > 60 s | > 60 s |  |
| parallel-net | partition-table | classical net j2 | partition-table/1-2-5 | 1.92 s ✗ | 1.82 s ✗ | 0.95× |
| parallel-net | partition-table | classical net j2 | partition-table/1-3 | 554 µs ✗ | 584 µs ✗ | 1.05× |
| parallel-net | partition-table | classical net j2 | partition-table/2-1-1 | 931 µs ✓ | 726 µs ✓ | 0.78× |
| parallel-net | partition-table | classical net j2 | partition-table/2-2-1-1 | 30.7 ms ✓ | 30.6 ms ✓ | 1.00× |
| parallel-net | partition-table | classical net j2 | partition-table/2-2-2-2-2-2-9-1 | > 60 s | > 60 s |  |
| parallel-net | partition-table | classical net j2 | partition-table/2-3-2-1 | 2.13 s ✓ | 2.18 s ✓ | 1.02× |
| parallel-net | partition-table | classical net j2 | partition-table/3-3-3-1 | > 60 s | > 60 s |  |
| parallel-net | partition-table | classical net j4 | partition-table/1-1 | 253 µs ✓ | 199 µs ✓ | 0.79× |
| parallel-net | partition-table | classical net j4 | partition-table/1-1-1-1-1-7 | > 60 s | > 60 s |  |
| parallel-net | partition-table | classical net j4 | partition-table/1-1-1-5 | 1.92 s ✗ | 1.90 s ✗ | 0.99× |
| parallel-net | partition-table | classical net j4 | partition-table/1-1-2-4 | 265.7 ms ✓ | 251.8 ms ✓ | 0.95× |
| parallel-net | partition-table | classical net j4 | partition-table/1-1-4 | 13.3 ms ✗ | 12.9 ms ✗ | 0.97× |
| parallel-net | partition-table | classical net j4 | partition-table/1-2-3-4-5-5 | 3.4 ms ✓ | 2.0 ms ✓ | 0.59× |
| parallel-net | partition-table | classical net j4 | partition-table/1-2-5 | 962.7 ms ✗ | 919.5 ms ✗ | 0.96× |
| parallel-net | partition-table | classical net j4 | partition-table/1-3 | 1.3 ms ✗ | 674 µs ✗ | 0.52× |
| parallel-net | partition-table | classical net j4 | partition-table/2-1-1 | 4.2 ms ✓ | 1.2 ms ✓ | 0.29× |
| parallel-net | partition-table | classical net j4 | partition-table/2-2-1-1 | 16.7 ms ✓ | 16.6 ms ✓ | 1.00× |
| parallel-net | partition-table | classical net j4 | partition-table/2-2-2-2-2-2-9-1 | > 60 s | > 60 s |  |
| parallel-net | partition-table | classical net j4 | partition-table/2-3-2-1 | 1.06 s ✓ | 1.06 s ✓ | 1.01× |
| parallel-net | partition-table | classical net j4 | partition-table/3-3-3-1 | > 60 s | > 60 s |  |
| parallel-net | partition-table | classical net j8 | partition-table/1-1 | 312 µs ✓ | 346 µs ✓ | 1.11× |
| parallel-net | partition-table | classical net j8 | partition-table/1-1-1-1-1-7 | > 60 s | > 60 s |  |
| parallel-net | partition-table | classical net j8 | partition-table/1-1-1-5 | 1.08 s ✗ | 1.08 s ✗ | 1.00× |
| parallel-net | partition-table | classical net j8 | partition-table/1-1-2-4 | 143.8 ms ✓ | 141.7 ms ✓ | 0.99× |
| parallel-net | partition-table | classical net j8 | partition-table/1-1-4 | 7.9 ms ✗ | 7.6 ms ✗ | 0.96× |
| parallel-net | partition-table | classical net j8 | partition-table/1-2-3-4-5-5 | > 60 s | > 60 s |  |
| parallel-net | partition-table | classical net j8 | partition-table/1-2-5 | 545.7 ms ✗ | 537.3 ms ✗ | 0.98× |
| parallel-net | partition-table | classical net j8 | partition-table/1-3 | 1.4 ms ✗ | 724 µs ✗ | 0.53× |
| parallel-net | partition-table | classical net j8 | partition-table/2-1-1 | 4.3 ms ✓ | 1.3 ms ✓ | 0.29× |
| parallel-net | partition-table | classical net j8 | partition-table/2-2-1-1 | 9.4 ms ✓ | 8.7 ms ✓ | 0.93× |
| parallel-net | partition-table | classical net j8 | partition-table/2-2-2-2-2-2-9-1 | 897.2 ms ✓ | 815.1 ms ✓ | 0.91× |
| parallel-net | partition-table | classical net j8 | partition-table/2-3-2-1 | 611.1 ms ✓ | 626.7 ms ✓ | 1.03× |
| parallel-net | partition-table | classical net j8 | partition-table/3-3-3-1 | > 60 s | > 60 s |  |
| period-1 | 3-partition-mll-no | classical net j1 period 1 | 3-partition-mll-no/4 | 1.85 s ✗ | 1.73 s ✗ | 0.93× |
| period-1 | 3-partition-mll-no | classical net j1 period 1 | 3-partition-mll-no/5 | 56.38 s ✗ | 55.13 s ✗ | 0.98× |
| period-1 | partition-no | classical net j1 period 1 | partition-no/3 | 3.49 s ✗ | 3.32 s ✗ | 0.95× |
| period-1 | partition-no | classical net j1 period 1 | partition-no/4 | > 60 s | > 60 s |  |
| period-1 | partition-yes | classical net j1 period 1 | partition-yes/4 | 2.75 s ✓ | 2.69 s ✓ | 0.98× |
| period-1 | wide-m1 | classical net j1 period 1 | wide-m1/2048 | 1.35 s ✓ | 1.22 s ✓ | 0.90× |
| period-1 | wide-m1 | classical net j1 period 1 | wide-m1/256 | 19.5 ms ✓ | 17.9 ms ✓ | 0.92× |
| period-1 | wide-m2 | classical net j1 period 1 | wide-m2/2048 | 1.38 s ✓ | 1.23 s ✓ | 0.89× |
| period-1 | wide-m2 | classical net j1 period 1 | wide-m2/256 | 19.6 ms ✓ | 18.5 ms ✓ | 0.94× |
| period-16 | 3-partition-mll-no | classical net j1 period 16 | 3-partition-mll-no/4 | 20.98 s ✗ | 20.95 s ✗ | 1.00× |
| period-16 | 3-partition-mll-no | classical net j1 period 16 | 3-partition-mll-no/5 | > 60 s | > 60 s |  |
| period-16 | partition-no | classical net j1 period 16 | partition-no/3 | > 60 s | > 60 s |  |
| period-16 | partition-no | classical net j1 period 16 | partition-no/4 | > 60 s | > 60 s |  |
| period-16 | partition-yes | classical net j1 period 16 | partition-yes/4 | > 60 s | > 60 s |  |
| period-16 | wide-m1 | classical net j1 period 16 | wide-m1/2048 | 434.9 ms ✓ | 388.2 ms ✓ | 0.89× |
| period-16 | wide-m1 | classical net j1 period 16 | wide-m1/256 | 6.2 ms ✓ | 5.9 ms ✓ | 0.94× |
| period-16 | wide-m2 | classical net j1 period 16 | wide-m2/2048 | 429.6 ms ✓ | 405.9 ms ✓ | 0.94× |
| period-16 | wide-m2 | classical net j1 period 16 | wide-m2/256 | 6.6 ms ✓ | 6.2 ms ✓ | 0.94× |
| period-2 | 3-partition-mll-no | classical net j1 period 2 | 3-partition-mll-no/4 | 1.45 s ✗ | 1.38 s ✗ | 0.95× |
| period-2 | 3-partition-mll-no | classical net j1 period 2 | 3-partition-mll-no/5 | 51.04 s ✗ | 49.10 s ✗ | 0.96× |
| period-2 | partition-no | classical net j1 period 2 | partition-no/3 | 7.82 s ✗ | 7.45 s ✗ | 0.95× |
| period-2 | partition-no | classical net j1 period 2 | partition-no/4 | > 60 s | > 60 s |  |
| period-2 | partition-yes | classical net j1 period 2 | partition-yes/4 | 6.00 s ✓ | 5.72 s ✓ | 0.95× |
| period-2 | wide-m1 | classical net j1 period 2 | wide-m1/2048 | 860.9 ms ✓ | 818.1 ms ✓ | 0.95× |
| period-2 | wide-m1 | classical net j1 period 2 | wide-m1/256 | 12.4 ms ✓ | 12.2 ms ✓ | 0.98× |
| period-2 | wide-m2 | classical net j1 period 2 | wide-m2/2048 | 855.4 ms ✓ | 803.6 ms ✓ | 0.94× |
| period-2 | wide-m2 | classical net j1 period 2 | wide-m2/256 | 12.4 ms ✓ | 12.0 ms ✓ | 0.97× |
| period-8 | 3-partition-mll-no | classical net j1 period 8 | 3-partition-mll-no/4 | 4.96 s ✗ | 4.78 s ✗ | 0.96× |
| period-8 | 3-partition-mll-no | classical net j1 period 8 | 3-partition-mll-no/5 | > 60 s | > 60 s |  |
| period-8 | partition-no | classical net j1 period 8 | partition-no/3 | > 60 s | > 60 s |  |
| period-8 | partition-no | classical net j1 period 8 | partition-no/4 | > 60 s | > 60 s |  |
| period-8 | partition-yes | classical net j1 period 8 | partition-yes/4 | > 60 s | > 60 s |  |
| period-8 | wide-m1 | classical net j1 period 8 | wide-m1/2048 | 494.5 ms ✓ | 448.2 ms ✓ | 0.91× |
| period-8 | wide-m1 | classical net j1 period 8 | wide-m1/256 | 7.2 ms ✓ | 7.0 ms ✓ | 0.96× |
| period-8 | wide-m2 | classical net j1 period 8 | wide-m2/2048 | 491.0 ms ✓ | 463.3 ms ✓ | 0.94× |
| period-8 | wide-m2 | classical net j1 period 8 | wide-m2/256 | 7.2 ms ✓ | 7.0 ms ✓ | 0.97× |


## The passes under one bias against the default

### lltp-rarer against lltp-intuitionistic

#### Verdicts that differ

None.

#### Counters that differ

Of the 3608 problems both decide alike on one thread, 79 differ in nodes, splits, memo_hits, memo_entries, links, tests.

#### Decided by lltp-rarer, not by lltp-intuitionistic

None.

#### Decided by lltp-intuitionistic, not by lltp-rarer

| file | family | configuration | problem | lltp-rarer | lltp-intuitionistic |
|---|---|---|---|--:|--:|
|  | ILL/ILLTP-LCL-cbn | intuitionistic auto j1 | LCL181+1.p | 46 µs ? bound | 123 µs ✗ |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ212+1.002.p | 50 µs ? bound | 145 µs ✗ |

#### By group

Problems in both; decided (late: after the time limit); decided by one and not the other; where the undecided ended (`time` the time limit or `killed` past it, `bound` the copy bound, `depth` the recursion limit, `memory` the memory limit, `index` an index past its width, `wide` a context too wide to split, `crash` out of memory); the median ratio of lltp-intuitionistic's time to lltp-rarer's on the problems both decide within the limit (how many); and the problems only lltp-intuitionistic has.

| file | family | configuration | problems | lltp-rarer decided | lltp-intuitionistic decided | only lltp-rarer | only lltp-intuitionistic | lltp-rarer ended | lltp-intuitionistic ended | lltp-intuitionistic/lltp-rarer | new |
|---|---|---|--:|--:|--:|--:|--:|---|---|--:|--:|
|  | ILL/ILLTP-LCL-01 | intuitionistic auto j1 | 2 | 0 | 0 | 0 | 0 | 2 bound | 2 bound | – | 0 |
|  | ILL/ILLTP-LCL-cbn | intuitionistic auto j1 | 2 | 1 | 2 | 0 | 1 | 1 bound | – | 1.14× (1) | 0 |
|  | ILL/ILLTP-LCL-cbv | intuitionistic auto j1 | 2 | 0 | 0 | 0 | 0 | 2 bound | 2 bound | – | 0 |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | 252 | 11 | 11 | 0 | 0 | 1 time, 215 bound, 22 depth, 3 memory | 1 time, 215 bound, 22 depth, 3 memory | 1.04× (11) | 0 |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | 252 | 34 | 35 | 0 | 1 | 3 time, 192 bound, 22 depth, 1 memory | 3 time, 190 bound, 22 depth, 2 memory | 2.74× (34) | 0 |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | 252 | 40 | 40 | 0 | 0 | 22 time, 178 bound, 10 depth, 2 memory | 22 time, 178 bound, 10 depth, 2 memory | 1.08× (40) | 0 |
|  | ILL/ILLTP-SYN-01 | intuitionistic auto j1 | 19 | 13 | 13 | 0 | 0 | 6 bound | 6 bound | 1.10× (13) | 0 |
|  | ILL/ILLTP-SYN-cbn | intuitionistic auto j1 | 19 | 18 | 18 | 0 | 0 | 1 bound | 1 bound | 1.21× (18) | 0 |
|  | ILL/ILLTP-SYN-cbv | intuitionistic auto j1 | 19 | 18 | 18 | 0 | 0 | 1 bound | 1 bound | 1.04× (18) | 0 |
|  | ILL/KLE-01 | intuitionistic auto j1 | 88 | 51 | 51 | 0 | 0 | 37 bound | 37 bound | 1.08× (51) | 0 |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | 222 | 162 | 162 | 0 | 0 | 60 bound | 60 bound | 1.05× (162) | 0 |
|  | ILL/KLE-IMP-CONJ/ALT | intuitionistic auto j1 | 27 | 27 | 27 | 0 | 0 | – | – | 1.03× (27) | 0 |
|  | ILL/KLE-IMP-CONJ/NON-THEOREMS | intuitionistic auto j1 | 22 | 22 | 22 | 0 | 0 | – | – | 0.90× (22) | 0 |
|  | ILL/KLE-cbn | intuitionistic auto j1 | 88 | 69 | 69 | 0 | 0 | 19 bound | 19 bound | 1.23× (69) | 0 |
|  | ILL/KLE-cbv | intuitionistic auto j1 | 88 | 68 | 68 | 0 | 0 | 20 bound | 20 bound | 1.05× (68) | 0 |
|  | ILL/Non-theorems | intuitionistic auto j1 | 1 | 0 | 0 | 0 | 0 | 1 bound | 1 bound | – | 0 |
|  | ILL/misc | intuitionistic auto j1 | 3 | 3 | 3 | 0 | 0 | – | – | 1.09× (3) | 0 |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | 3137 | 3071 | 3071 | 0 | 0 | 58 time, 8 memory | 58 time, 8 memory | 1.01× (3071) | 0 |

### lltp-forward against lltp-intuitionistic

#### Verdicts that differ

None.

#### Counters that differ

Of the 3610 problems both decide alike on one thread, 0 differ in nodes, splits, memo_hits, memo_entries, links, tests.

#### Decided by lltp-forward, not by lltp-intuitionistic

| file | family | configuration | problem | lltp-forward | lltp-intuitionistic |
|---|---|---|---|--:|--:|
|  | ILL/ILLTP-LCL-01 | intuitionistic auto j1 | LCL181+1.p | 106 µs ✗ | 83 µs ? bound |
|  | ILL/ILLTP-LCL-01 | intuitionistic auto j1 | LCL230+1.p | 68 µs ✗ | 66 µs ? bound |
|  | ILL/ILLTP-LCL-cbv | intuitionistic auto j1 | LCL181+1.p | 67 µs ✗ | 59 µs ? bound |
|  | ILL/ILLTP-LCL-cbv | intuitionistic auto j1 | LCL230+1.p | 73 µs ✗ | 75 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ103+1.p | 225 µs ✗ MISMATCH | 73 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ105+1.003.p | 186 µs ✗ MISMATCH | 68 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ105+1.004.p | 757 µs ✗ MISMATCH | 65 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ106+1.p | 167 µs ✓ | 86 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ107+1.002.p | 599 µs ✓ | 92 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ107+1.003.p | 6.2 ms ✓ | 127 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ107+1.004.p | 66.6 ms ✓ | 155 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ201+1.001.p | 1.0 ms ✓ | 112 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ201+1.002.p | 72.2 ms ✓ | 225 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ202+1.002.p | 536 µs ✓ | 279 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ202+1.003.p | 124.8 ms ✓ | 1.6 ms ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ203+1.002.p | 102 µs ✓ | 96 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ203+1.003.p | 201 µs ✓ | 117 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ203+1.004.p | 511 µs ✓ | 175 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ203+1.005.p | 1.7 ms ✓ | 205 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ203+1.006.p | 6.0 ms ✓ | 262 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ203+1.007.p | 22.6 ms ✓ | 360 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ203+1.008.p | 87.0 ms ✓ | 416 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ203+1.009.p | 347.3 ms ✓ | 537 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ203+1.010.p | 1.44 s ✓ | 685 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ204+1.002.p | 69 µs ✓ | 57 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ204+1.003.p | 89 µs ✓ | 56 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ204+1.004.p | 271 µs ✓ | 59 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ204+1.005.p | 399 µs ✓ | 69 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ204+1.006.p | 501 µs ✓ | 77 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ204+1.007.p | 781 µs ✓ | 77 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ204+1.008.p | 11.9 ms ✓ | 78 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ204+1.009.p | 15.2 ms ✓ | 80 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ204+1.010.p | 18.6 ms ✓ | 83 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ204+1.011.p | 22.2 ms ✓ | 89 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ204+1.012.p | 26.9 ms ✓ | 97 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ204+1.013.p | 38.0 ms ✓ | 118 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ204+1.014.p | 44.8 ms ✓ | 114 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ204+1.015.p | 52.6 ms ✓ | 119 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ205+1.001.p | 201 µs ✓ | 54 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ205+1.002.p | 732 µs ✓ | 56 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ205+1.003.p | 2.9 ms ✓ | 60 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ205+1.004.p | 13.8 ms ✓ | 65 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ205+1.005.p | 46.6 ms ✓ | 73 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ205+1.006.p | 138.7 ms ✓ | 57 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ205+1.007.p | 419.8 ms ✓ | 60 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ205+1.008.p | 1.66 s ✓ | 62 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ205+1.009.p | 3.46 s ✓ | 67 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ206+1.003.p | 555 µs ✓ | 131 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ206+1.004.p | 11.3 ms ✓ | 201 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ206+1.005.p | 363.7 ms ✓ | 307 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ207+1.001.p | 365 µs ✗ | 95 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ207+1.002.p | 69.6 ms ✗ | 175 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ208+1.001.p | 229 µs ✗ | 79 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ208+1.002.p | 3.7 ms ✗ | 309 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ208+1.003.p | 812.8 ms ✗ | 1.7 ms ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ209+1.002.p | 82 µs ✗ | 69 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ209+1.003.p | 192 µs ✗ | 96 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ209+1.004.p | 744 µs ✗ | 123 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ209+1.005.p | 2.7 ms ✗ | 163 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ209+1.006.p | 11.9 ms ✗ | 212 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ209+1.007.p | 51.7 ms ✗ | 269 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ209+1.008.p | 209.0 ms ✗ | 349 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ209+1.009.p | 872.0 ms ✗ | 443 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ209+1.010.p | 3.96 s ✗ | 546 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ210+1.001.p | 120 µs ✗ | 60 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ210+1.002.p | 646 µs ✗ | 63 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ210+1.003.p | 5.3 ms ✗ | 63 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ210+1.004.p | 34.9 ms ✗ | 72 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ211+1.001.p | 65 µs ✗ | 49 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ211+1.002.p | 73 µs ✗ | 64 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ211+1.003.p | 90 µs ✗ | 59 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ211+1.004.p | 112 µs ✗ | 67 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ211+1.005.p | 162 µs ✗ | 68 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ211+1.006.p | 224 µs ✗ | 74 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ211+1.007.p | 301 µs ✗ | 85 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ211+1.008.p | 418 µs ✗ | 85 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ211+1.009.p | 567 µs ✗ | 88 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ211+1.010.p | 766 µs ✗ | 103 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ211+1.011.p | 994 µs ✗ | 106 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ211+1.012.p | 1.3 ms ✗ | 115 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ211+1.013.p | 1.7 ms ✗ | 124 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ211+1.014.p | 2.3 ms ✗ | 134 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ211+1.015.p | 2.8 ms ✗ | 138 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ211+1.016.p | 3.7 ms ✗ | 149 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ211+1.017.p | 4.5 ms ✗ | 163 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ211+1.018.p | 5.5 ms ✗ | 176 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ211+1.019.p | 6.6 ms ✗ | 185 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ211+1.020.p | 8.0 ms ✗ | 196 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ212+1.001.p | 117 µs ✓ MISMATCH | 52 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ212+1.002.p | 144 µs ✗ | 90 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ212+1.003.p | 9.0 ms ✗ | 119 µs ? bound |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | SYJ212+1.004.p | 521.7 ms ✗ | 209 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ103+1.p | 113 µs ✗ MISMATCH | 63 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ105+1.003.p | 141 µs ✗ MISMATCH | 49 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ105+1.004.p | 518 µs ✗ MISMATCH | 50 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ106+1.p | 61 µs ✓ | 159 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ107+1.002.p | 281 µs ✓ | 76 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ107+1.003.p | 2.2 ms ✓ | 103 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ107+1.004.p | 20.8 ms ✓ | 129 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ201+1.001.p | 145 µs ✓ | 62 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ201+1.002.p | 1.2 ms ✓ | 107 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ201+1.003.p | 5.2 ms ✓ | 188 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ201+1.004.p | 162.4 ms ✓ | 328 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ201+1.005.p | 617.9 ms ✓ | 495 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ201+1.006.p | 3.07 s ✓ | 755 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ202+1.002.p | 398 µs ✓ | 224 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ202+1.003.p | 83.7 ms ✓ | 1.3 ms ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ203+1.002.p | 105 µs ✓ | 69 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ203+1.003.p | 159 µs ✓ | 99 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ203+1.004.p | 376 µs ✓ | 119 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ203+1.005.p | 1.1 ms ✓ | 148 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ203+1.006.p | 4.7 ms ✓ | 191 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ203+1.007.p | 15.0 ms ✓ | 243 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ203+1.008.p | 56.2 ms ✓ | 304 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ203+1.009.p | 237.9 ms ✓ | 391 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ203+1.010.p | 904.9 ms ✓ | 483 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ204+1.003.p | 52 µs ✓ | 134 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ204+1.004.p | 67 µs ✓ | 152 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ204+1.005.p | 57 µs ✓ | 98 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ204+1.006.p | 70 µs ✓ | 150 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ204+1.007.p | 79 µs ✓ | 172 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ204+1.008.p | 91 µs ✓ | 164 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ204+1.009.p | 111 µs ✓ | 186 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ204+1.010.p | 125 µs ✓ | 199 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ204+1.011.p | 153 µs ✓ | 139 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ204+1.012.p | 174 µs ✓ | 190 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ204+1.013.p | 199 µs ✓ | 208 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ204+1.014.p | 223 µs ✓ | 205 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ204+1.015.p | 255 µs ✓ | 211 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ204+1.016.p | 286 µs ✓ | 227 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ204+1.017.p | 470 µs ✓ | 232 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ204+1.018.p | 383 µs ✓ | 268 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ204+1.019.p | 452 µs ✓ | 263 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ204+1.020.p | 493 µs ✓ | 235 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ205+1.001.p | 64 µs ✓ | 178 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ205+1.002.p | 118 µs ✓ | 201 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ205+1.003.p | 252 µs ✓ | 230 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ205+1.004.p | 616 µs ✓ | 265 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ205+1.005.p | 1.8 ms ✓ | 305 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ205+1.006.p | 5.1 ms ✓ | 374 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ205+1.007.p | 15.5 ms ✓ | 445 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ205+1.008.p | 42.9 ms ✓ | 552 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ205+1.009.p | 115.7 ms ✓ | 605 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ205+1.010.p | 318.1 ms ✓ | 858 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ205+1.011.p | 967.5 ms ✓ | 1.0 ms ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ205+1.012.p | 3.55 s ✓ | 1.2 ms ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ206+1.004.p | 229 µs ✓ | 286 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ206+1.005.p | 1.6 ms ✓ | 455 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ206+1.006.p | 13.9 ms ✓ | 717 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ206+1.007.p | 208.3 ms ✓ | 1.2 ms ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ207+1.001.p | 117 µs ✗ | 88 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ207+1.002.p | 4.9 ms ✗ | 121 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ207+1.003.p | 426.8 ms ✗ | 202 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ208+1.001.p | 118 µs ✗ | 74 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ208+1.002.p | 1.4 ms ✗ | 230 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ208+1.003.p | 328.7 ms ✗ | 1.2 ms ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ209+1.002.p | 65 µs ✗ | 185 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ209+1.003.p | 140 µs ✗ | 95 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ209+1.004.p | 441 µs ✗ | 108 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ209+1.005.p | 1.8 ms ✗ | 136 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ209+1.006.p | 7.1 ms ✗ | 194 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ209+1.007.p | 30.8 ms ✗ | 216 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ209+1.008.p | 131.1 ms ✗ | 288 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ209+1.009.p | 524.1 ms ✗ | 428 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ209+1.010.p | 2.13 s ✗ | 442 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ210+1.001.p | 58 µs ✗ | 175 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ210+1.002.p | 52 µs ✗ | 165 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ210+1.003.p | 66 µs ✗ | 157 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ210+1.004.p | 74 µs ✗ | 142 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ210+1.005.p | 94 µs ✗ | 185 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ210+1.006.p | 111 µs ✗ | 179 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ210+1.007.p | 142 µs ✗ | 185 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ210+1.008.p | 166 µs ✗ | 226 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ210+1.009.p | 205 µs ✗ | 218 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ210+1.010.p | 245 µs ✗ | 311 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ210+1.011.p | 303 µs ✗ | 240 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ210+1.012.p | 359 µs ✗ | 263 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ210+1.013.p | 430 µs ✗ | 263 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ210+1.014.p | 508 µs ✗ | 328 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ210+1.015.p | 592 µs ✗ | 302 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ210+1.016.p | 682 µs ✗ | 319 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ210+1.017.p | 812 µs ✗ | 322 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ210+1.018.p | 929 µs ✗ | 377 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ210+1.019.p | 1.1 ms ✗ | 368 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ210+1.020.p | 1.2 ms ✗ | 381 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ212+1.003.p | 70 µs ✗ | 222 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ212+1.004.p | 124 µs ✗ | 303 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ212+1.005.p | 213 µs ✗ | 477 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ212+1.006.p | 406 µs ✗ | 683 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ212+1.007.p | 778 µs ✗ | 961 µs ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ212+1.008.p | 1.6 ms ✗ | 1.7 ms ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ212+1.009.p | 3.6 ms ✗ | 3.1 ms ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ212+1.010.p | 8.6 ms ✗ | 6.8 ms ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ212+1.011.p | 23.1 ms ✗ | 15.8 ms ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ212+1.012.p | 66.3 ms ✗ | 41.3 ms ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ212+1.013.p | 191.6 ms ✗ | 106.8 ms ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ212+1.014.p | 513.6 ms ✗ | 310.8 ms ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ212+1.015.p | 1.15 s ✗ | 875.8 ms ? bound |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | SYJ212+1.016.p | 2.43 s ✗ | 1.99 s ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ103+1.p | 79 µs ✗ MISMATCH | 64 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ105+1.003.p | 60 µs ✗ MISMATCH | 60 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ105+1.004.p | 57 µs ✗ MISMATCH | 61 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ106+1.p | 124 µs ✓ | 85 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ107+1.004.p | 1.1 ms ✓ | 174 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ201+1.001.p | 145 µs ✓ | 115 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ201+1.002.p | 1.1 ms ✓ | 216 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ201+1.003.p | 4.0 ms ✓ | 521 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ201+1.004.p | 40.2 ms ✓ | 731 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ201+1.005.p | 122.9 ms ✓ | 1.1 ms ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ201+1.006.p | 302.1 ms ✓ | 1.9 ms ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ201+1.007.p | 668.0 ms ✓ | 2.7 ms ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ203+1.002.p | 80 µs ✓ | 74 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ203+1.003.p | 158 µs ✓ | 98 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ203+1.004.p | 506 µs ✓ | 121 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ203+1.005.p | 2.1 ms ✓ | 168 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ203+1.006.p | 9.0 ms ✓ | 218 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ203+1.007.p | 39.5 ms ✓ | 297 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ203+1.008.p | 170.4 ms ✓ | 370 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ203+1.009.p | 709.9 ms ✓ | 504 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ203+1.010.p | 2.98 s ✓ | 612 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ204+1.002.p | 99 µs ✓ | 52 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ204+1.003.p | 113 µs ✓ | 58 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ204+1.004.p | 354 µs ✓ | 61 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ204+1.005.p | 454 µs ✓ | 63 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ204+1.006.p | 584 µs ✓ | 68 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ204+1.007.p | 906 µs ✓ | 72 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ204+1.008.p | 14.0 ms ✓ | 75 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ204+1.009.p | 17.5 ms ✓ | 85 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ204+1.010.p | 25.1 ms ✓ | 94 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ204+1.011.p | 39.7 ms ✓ | 96 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ204+1.012.p | 29.6 ms ✓ | 108 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ204+1.013.p | 48.2 ms ✓ | 113 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ204+1.014.p | 49.5 ms ✓ | 121 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ204+1.015.p | 57.2 ms ✓ | 124 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ205+1.002.p | 192 µs ✓ | 99 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ205+1.003.p | 328 µs ✓ | 127 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ205+1.004.p | 1.3 ms ✓ | 160 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ205+1.005.p | 2.7 ms ✓ | 211 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ205+1.006.p | 4.9 ms ✓ | 280 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ205+1.007.p | 8.5 ms ✓ | 363 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ205+1.008.p | 43.6 ms ✓ | 482 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ205+1.009.p | 74.7 ms ✓ | 574 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ205+1.010.p | 120.4 ms ✓ | 754 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ205+1.011.p | 188.6 ms ✓ | 938 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ205+1.012.p | 293.9 ms ✓ | 1.1 ms ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ205+1.013.p | 467.8 ms ✓ | 1.4 ms ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ205+1.014.p | 688.3 ms ✓ | 1.7 ms ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ205+1.015.p | 1.31 s ✓ | 2.0 ms ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ206+1.004.p | 959 µs ✓ | 269 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ206+1.005.p | 9.2 ms ✓ | 575 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ206+1.006.p | 116.7 ms ✓ | 885 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ206+1.007.p | 1.93 s ✓ | 1.4 ms ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ207+1.001.p | 179 µs ✗ | 109 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ207+1.002.p | 23.1 ms ✗ | 171 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ207+1.003.p | 3.04 s ✗ | 307 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ208+1.001.p | 99 µs ✗ | 63 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ208+1.002.p | 71 µs ✗ | 65 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ208+1.003.p | 137 µs ✗ | 124 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ208+1.004.p | 412 µs ✗ | 392 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ208+1.005.p | 2.5 ms ✗ | 2.6 ms ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ208+1.006.p | 29.1 ms ✗ | 29.2 ms ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ208+1.007.p | 613.8 ms ✗ | 636.7 ms ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ209+1.002.p | 70 µs ✗ | 58 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ209+1.003.p | 320 µs ✗ | 75 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ209+1.004.p | 3.2 ms ✗ | 99 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ209+1.005.p | 35.9 ms ✗ | 134 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ209+1.006.p | 345.3 ms ✗ | 176 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ210+1.001.p | 78 µs ✗ | 52 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ210+1.002.p | 225 µs ✗ | 56 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ210+1.003.p | 1.3 ms ✗ | 58 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ210+1.004.p | 7.9 ms ✗ | 63 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ210+1.005.p | 45.9 ms ✗ | 61 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ212+1.001.p | 59 µs ✓ MISMATCH | 45 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ212+1.002.p | 121 µs ✗ | 76 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ212+1.003.p | 2.4 ms ✗ | 132 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ212+1.004.p | 60.2 ms ✗ | 266 µs ? bound |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | SYJ212+1.005.p | 1.16 s ✗ | 473 µs ? bound |
|  | ILL/ILLTP-SYN-01 | intuitionistic auto j1 | SYN044+1.p | 114 µs ✓ | 79 µs ? bound |
|  | ILL/ILLTP-SYN-01 | intuitionistic auto j1 | SYN045+1.p | 182 µs ✓ | 107 µs ? bound |
|  | ILL/ILLTP-SYN-01 | intuitionistic auto j1 | SYN388+1.p | 51 µs ✗ | 58 µs ? bound |
|  | ILL/ILLTP-SYN-01 | intuitionistic auto j1 | SYN391+1.p | 286 µs ✓ | 117 µs ? bound |
|  | ILL/ILLTP-SYN-01 | intuitionistic auto j1 | SYN392+1.p | 86 µs ✗ | 76 µs ? bound |
|  | ILL/ILLTP-SYN-01 | intuitionistic auto j1 | SYN393+1.p | 2.7 ms ✗ | 150 µs ? bound |
|  | ILL/ILLTP-SYN-cbn | intuitionistic auto j1 | SYN393+1.p | 82 µs ✗ | 171 µs ? bound |
|  | ILL/ILLTP-SYN-cbv | intuitionistic auto j1 | SYN393+1.p | 167 µs ✗ | 126 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE004+1.p | 62 µs ✓ | 60 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE017+1.p | 69 µs ✗ MISMATCH | 61 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE018+1.p | 130 µs ✓ | 105 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE026+1.p | 271 µs ✓ | 117 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE027+1.p | 267 µs ✓ | 92 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE028+1.p | 134 µs ✓ | 79 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE029+1.p | 229 µs ✓ | 91 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE030+1.p | 1.6 ms ✓ | 86 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE031+1.p | 8.4 ms ✓ | 86 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE032+1.p | 93 µs ✓ | 85 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE033+1.p | 108 µs ✓ | 90 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE034+1.p | 119 µs ✓ | 91 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE035+1.p | 121 µs ✓ | 101 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE036+1.p | 172 µs ✓ | 88 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE037+1.p | 162 µs ✓ | 97 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE038+1.p | 681 µs ✓ | 77 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE043+1.p | 133 µs ✓ | 90 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE044+1.p | 183 µs ✓ | 103 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE058+1.p | 138 µs ✓ | 51 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE061+1.p | 48 µs ✓ | 45 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE062+1.p | 82 µs ✓ | 49 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE066+1.p | 63 µs ✓ | 52 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE067+1.p | 102 µs ✓ | 227 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE068+1.p | 63 µs ✗ | 60 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE069+1.p | 265 µs ✗ MISMATCH | 62 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE073+1.p | 71 µs ✓ | 67 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE074+1.p | 86.5 ms ✗ MISMATCH | 163 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE076+1.p | 134 µs ✓ | 66 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE078+1.p | 350 µs ✗ MISMATCH | 55 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE080+1.p | 79 µs ✓ | 64 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE081+1.p | 76 µs ✓ | 63 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE082+1.p | 114 µs ✓ | 62 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE084+1.p | 5.4 ms ✓ | 131 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE086+1.p | 1.3 ms ✗ MISMATCH | 52 µs ? bound |
|  | ILL/KLE-01 | intuitionistic auto j1 | KLE088+1.p | 1.2 ms ✗ MISMATCH | 53 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_14_01.p | 79 µs ✓ | 76 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_14_CBV.p | 69 µs ✓ | 64 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_22_01.p | 263 µs ✓ | 111 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_23_01.p | 278 µs ✓ | 94 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_23_CBN.p | 76 µs ✓ | 168 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_23_CBV.p | 196 µs ✓ | 75 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_24_01.p | 145 µs ✓ | 78 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_24_CBN.p | 59 µs ✓ | 151 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_24_CBV.p | 94 µs ✓ | 64 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_25_01.p | 227 µs ✓ | 89 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_25_CBN.p | 102 µs ✓ | 157 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_25_CBV.p | 255 µs ✓ | 71 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_26_01.p | 221 µs ✓ | 71 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_26_CBN.p | 73 µs ✓ | 66 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_26_CBV.p | 90 µs ✓ | 84 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_27_01.p | 516 µs ✓ | 88 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_27_CBN.p | 99 µs ✓ | 161 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_27_CBV.p | 164 µs ✓ | 69 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_28_01.p | 111 µs ✓ | 81 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_29_01.p | 110 µs ✓ | 84 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_30_01.p | 116 µs ✓ | 95 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_31_01.p | 129 µs ✓ | 103 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_32_01.p | 110 µs ✓ | 83 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_44_01.p | 50 µs ✓ | 47 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_45_01.p | 87 µs ✓ | 56 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_45_CBN.p | 42 µs ✓ | 150 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_45_CBV.p | 49 µs ✓ | 49 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_46_01.p | 65 µs ✓ | 54 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_46_CBN.p | 42 µs ✓ | 38 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_46_CBV.p | 48 µs ✓ | 47 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_47_01.p | 103 µs ✓ | 77 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_48_01.p | 65 µs ✓ | 61 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_49_01.p | 71 µs ✓ | 59 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_4_01.p | 60 µs ✓ | 61 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_50_01.p | 117 µs ✓ | 94 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_51_01.p | 152 µs ✓ | 111 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_51_CBN.p | 72 µs ✓ | 179 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_51_CBV.p | 100 µs ✓ | 78 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_52_01.p | 162 µs ✓ | 93 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_52_CBN.p | 62 µs ✓ | 185 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_52_CBV.p | 96 µs ✓ | 88 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_53_01.p | 106 µs ✓ | 69 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_54_01.p | 83 µs ✓ | 64 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_55_01.p | 81 µs ✓ | 67 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_56_01.p | 144 µs ✓ | 71 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_56_CBV.p | 67 µs ✓ | 58 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_57_01.p | 158 µs ✓ | 73 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_57_CBN.p | 51 µs ✓ | 158 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_57_CBV.p | 71 µs ✓ | 60 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_58_01.p | 149 µs ✓ | 66 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_58_CBN.p | 59 µs ✓ | 144 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_58_CBV.p | 74 µs ✓ | 67 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_59_01.p | 185 µs ✓ | 82 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_59_CBN.p | 52 µs ✓ | 161 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_59_CBV.p | 81 µs ✓ | 58 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_60_01.p | 160 µs ✓ | 102 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_60_CBV.p | 79 µs ✓ | 80 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_61_01.p | 158 µs ✓ | 80 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_61_CBN.p | 65 µs ✓ | 55 µs ? bound |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | KLE_61_CBV.p | 96 µs ✓ | 66 µs ? bound |
|  | ILL/KLE-cbn | intuitionistic auto j1 | KLE017+1.p | 49 µs ✗ MISMATCH | 51 µs ? bound |
|  | ILL/KLE-cbn | intuitionistic auto j1 | KLE018+1.p | 61 µs ✓ | 53 µs ? bound |
|  | ILL/KLE-cbn | intuitionistic auto j1 | KLE027+1.p | 72 µs ✓ | 172 µs ? bound |
|  | ILL/KLE-cbn | intuitionistic auto j1 | KLE028+1.p | 56 µs ✓ | 147 µs ? bound |
|  | ILL/KLE-cbn | intuitionistic auto j1 | KLE029+1.p | 102 µs ✓ | 161 µs ? bound |
|  | ILL/KLE-cbn | intuitionistic auto j1 | KLE030+1.p | 195 µs ✓ | 51 µs ? bound |
|  | ILL/KLE-cbn | intuitionistic auto j1 | KLE031+1.p | 217 µs ✓ | 149 µs ? bound |
|  | ILL/KLE-cbn | intuitionistic auto j1 | KLE038+1.p | 53 µs ✓ | 158 µs ? bound |
|  | ILL/KLE-cbn | intuitionistic auto j1 | KLE058+1.p | 48 µs ✓ | 144 µs ? bound |
|  | ILL/KLE-cbn | intuitionistic auto j1 | KLE062+1.p | 39 µs ✓ | 105 µs ? bound |
|  | ILL/KLE-cbn | intuitionistic auto j1 | KLE068+1.p | 57 µs ✗ | 50 µs ? bound |
|  | ILL/KLE-cbn | intuitionistic auto j1 | KLE069+1.p | 90 µs ✗ MISMATCH | 48 µs ? bound |
|  | ILL/KLE-cbn | intuitionistic auto j1 | KLE074+1.p | 4.2 ms ✗ MISMATCH | 95 µs ? bound |
|  | ILL/KLE-cbn | intuitionistic auto j1 | KLE075+1.p | 271 µs ✗ MISMATCH | 201 µs ? bound |
|  | ILL/KLE-cbn | intuitionistic auto j1 | KLE078+1.p | 66 µs ✗ MISMATCH | 139 µs ? bound |
|  | ILL/KLE-cbn | intuitionistic auto j1 | KLE084+1.p | 551 µs ✓ | 183 µs ? bound |
|  | ILL/KLE-cbn | intuitionistic auto j1 | KLE086+1.p | 154 µs ✗ MISMATCH | 161 µs ? bound |
|  | ILL/KLE-cbn | intuitionistic auto j1 | KLE088+1.p | 164 µs ✗ MISMATCH | 59 µs ? bound |
|  | ILL/KLE-cbv | intuitionistic auto j1 | KLE017+1.p | 62 µs ✗ MISMATCH | 56 µs ? bound |
|  | ILL/KLE-cbv | intuitionistic auto j1 | KLE018+1.p | 88 µs ✓ | 79 µs ? bound |
|  | ILL/KLE-cbv | intuitionistic auto j1 | KLE027+1.p | 177 µs ✓ | 76 µs ? bound |
|  | ILL/KLE-cbv | intuitionistic auto j1 | KLE028+1.p | 92 µs ✓ | 62 µs ? bound |
|  | ILL/KLE-cbv | intuitionistic auto j1 | KLE029+1.p | 169 µs ✓ | 77 µs ? bound |
|  | ILL/KLE-cbv | intuitionistic auto j1 | KLE030+1.p | 236 µs ✓ | 75 µs ? bound |
|  | ILL/KLE-cbv | intuitionistic auto j1 | KLE031+1.p | 1.1 ms ✓ | 71 µs ? bound |
|  | ILL/KLE-cbv | intuitionistic auto j1 | KLE038+1.p | 122 µs ✓ | 60 µs ? bound |
|  | ILL/KLE-cbv | intuitionistic auto j1 | KLE058+1.p | 67 µs ✓ | 45 µs ? bound |
|  | ILL/KLE-cbv | intuitionistic auto j1 | KLE062+1.p | 54 µs ✓ | 46 µs ? bound |
|  | ILL/KLE-cbv | intuitionistic auto j1 | KLE069+1.p | 63 µs ✗ MISMATCH | 53 µs ? bound |
|  | ILL/KLE-cbv | intuitionistic auto j1 | KLE074+1.p | 3.6 ms ✗ MISMATCH | 158 µs ? bound |
|  | ILL/KLE-cbv | intuitionistic auto j1 | KLE076+1.p | 80 µs ✓ | 57 µs ? bound |
|  | ILL/KLE-cbv | intuitionistic auto j1 | KLE078+1.p | 64 µs ✗ MISMATCH | 54 µs ? bound |
|  | ILL/KLE-cbv | intuitionistic auto j1 | KLE082+1.p | 62 µs ✓ | 58 µs ? bound |
|  | ILL/KLE-cbv | intuitionistic auto j1 | KLE084+1.p | 1.9 ms ✓ | 114 µs ? bound |
|  | ILL/KLE-cbv | intuitionistic auto j1 | KLE086+1.p | 320 µs ✗ MISMATCH | 55 µs ? bound |
|  | ILL/KLE-cbv | intuitionistic auto j1 | KLE088+1.p | 275 µs ✗ MISMATCH | 61 µs ? bound |

#### Decided by lltp-intuitionistic, not by lltp-forward

None.

#### By group

Problems in both; decided (late: after the time limit); decided by one and not the other; where the undecided ended (`time` the time limit or `killed` past it, `bound` the copy bound, `depth` the recursion limit, `memory` the memory limit, `index` an index past its width, `wide` a context too wide to split, `crash` out of memory); the median ratio of lltp-intuitionistic's time to lltp-forward's on the problems both decide within the limit (how many); and the problems only lltp-intuitionistic has.

| file | family | configuration | problems | lltp-forward decided | lltp-intuitionistic decided | only lltp-forward | only lltp-intuitionistic | lltp-forward ended | lltp-intuitionistic ended | lltp-intuitionistic/lltp-forward | new |
|---|---|---|--:|--:|--:|--:|--:|---|---|--:|--:|
|  | ILL/ILLTP-LCL-01 | intuitionistic auto j1 | 2 | 2 | 0 | 2 | 0 | – | 2 bound | – | 0 |
|  | ILL/ILLTP-LCL-cbn | intuitionistic auto j1 | 2 | 2 | 2 | 0 | 0 | – | – | 2.73× (2) | 0 |
|  | ILL/ILLTP-LCL-cbv | intuitionistic auto j1 | 2 | 2 | 0 | 2 | 0 | – | 2 bound | – | 0 |
|  | ILL/ILLTP-SYJ-01 | intuitionistic auto j1 | 252 | 99 | 11 | 88 | 0 | 124 time, 4 bound, 22 depth, 3 memory | 1 time, 215 bound, 22 depth, 3 memory | 1.02× (11) | 0 |
|  | ILL/ILLTP-SYJ-cbn | intuitionistic auto j1 | 252 | 142 | 35 | 107 | 0 | 87 time, 22 depth, 1 memory | 3 time, 190 bound, 22 depth, 2 memory | 2.75× (35) | 0 |
|  | ILL/ILLTP-SYJ-cbv | intuitionistic auto j1 | 252 | 118 | 40 | 78 | 0 | 118 time, 4 bound, 10 depth, 2 memory | 22 time, 178 bound, 10 depth, 2 memory | 1.08× (40) | 0 |
|  | ILL/ILLTP-SYN-01 | intuitionistic auto j1 | 19 | 19 | 13 | 6 | 0 | – | 6 bound | 1.19× (13) | 0 |
|  | ILL/ILLTP-SYN-cbn | intuitionistic auto j1 | 19 | 19 | 18 | 1 | 0 | – | 1 bound | 1.33× (18) | 0 |
|  | ILL/ILLTP-SYN-cbv | intuitionistic auto j1 | 19 | 19 | 18 | 1 | 0 | – | 1 bound | 1.14× (18) | 0 |
|  | ILL/KLE-01 | intuitionistic auto j1 | 88 | 86 | 51 | 35 | 0 | 2 bound | 37 bound | 1.10× (51) | 0 |
|  | ILL/KLE-IMP-CONJ | intuitionistic auto j1 | 222 | 222 | 162 | 60 | 0 | – | 60 bound | 1.12× (162) | 0 |
|  | ILL/KLE-IMP-CONJ/ALT | intuitionistic auto j1 | 27 | 27 | 27 | 0 | 0 | – | – | 1.18× (27) | 0 |
|  | ILL/KLE-IMP-CONJ/NON-THEOREMS | intuitionistic auto j1 | 22 | 22 | 22 | 0 | 0 | – | – | 1.04× (22) | 0 |
|  | ILL/KLE-cbn | intuitionistic auto j1 | 88 | 87 | 69 | 18 | 0 | 1 bound | 19 bound | 1.33× (69) | 0 |
|  | ILL/KLE-cbv | intuitionistic auto j1 | 88 | 86 | 68 | 18 | 0 | 2 bound | 20 bound | 1.08× (68) | 0 |
|  | ILL/Non-theorems | intuitionistic auto j1 | 1 | 0 | 0 | 0 | 0 | 1 bound | 1 bound | – | 0 |
|  | ILL/misc | intuitionistic auto j1 | 3 | 3 | 3 | 0 | 0 | – | – | 1.12× (3) | 0 |
|  | ILL/petri-nets/MCC | intuitionistic auto j1 | 3137 | 3071 | 3071 | 0 | 0 | 57 time, 9 memory | 58 time, 8 memory | 1.04× (3071) | 0 |

