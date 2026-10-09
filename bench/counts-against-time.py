#!/usr/bin/env python3
# linlog © Fabian Lukas Grubmüller 2026
# Licensed under the EUPL

# Whether the journeys' instruction counts track their wall-clock time:
# three builds of the same code at opt-level 1, 2 and 3 count apart, and
# every journey is timed for all three on one pinned core.
#
#   for o in 1 2 3; do
#     CARGO_PROFILE_RELEASE_OPT_LEVEL=$o cargo build --release --locked \
#       -p linlog-bench --target-dir target/opt$o
#     target/opt$o/release/linlog-bench ratchet --jobs 4 \
#       --ceilings target/opt$o/none.csv > target/opt$o/counts.txt
#   done
#   bench/counts-against-time.py run 2 5 > target/counts-against-time.csv
#   bench/counts-against-time.py table target/counts-against-time.csv
#
# `run CORE ROUNDS` times every journey in ROUNDS rounds, each running the
# three builds once in a rotating order; a run is one process doing the
# journey until about 0.3 s of measured work, and records the median time
# of the measured part, the process's wall-clock and user+system CPU time,
# and the busy share of the other cores from /proc/stat. Run it with
# nothing heavy beside it and no timer due. `table CSV` prints, per
# journey, the counts and times of opt-level 1 and 2 relative to 3, the
# largest spread of a build's rounds, and whether the builds order alike
# by count and by time.
import collections
import csv
import os
import re
import statistics
import subprocess
import sys
import time

BUILDS = {b: f"target/{b}/release/linlog-bench" for b in ("opt1", "opt2", "opt3")}


def cpu_times():
    """Busy and total jiffies per CPU."""
    out = {}
    for line in open("/proc/stat"):
        if line.startswith("cpu") and line[3].isdigit():
            f = line.split()
            vals = list(map(int, f[1:]))
            out[int(f[0][3:])] = (sum(vals) - vals[3] - vals[4], sum(vals))
    return out


def once(core, exe, name, reps):
    """One process of the journey: the median measured time, the process's
    wall-clock and CPU seconds, the other cores' busy share."""
    before = cpu_times()
    start = time.monotonic()
    p = subprocess.Popen(
        ["taskset", "-c", core, exe, "journey", name, "--repeat", str(reps)],
        stdout=subprocess.PIPE,
        text=True,
    )
    out = p.stdout.read()
    _, status, usage = os.wait4(p.pid, 0)
    wall = time.monotonic() - start
    after = cpu_times()
    if status != 0:
        sys.exit(f"{exe} journey {name} failed")
    busy = [
        (after[c][0] - before[c][0]) / max(1, after[c][1] - before[c][1])
        for c in after
        if c != int(core)
    ]
    times = [int(x) for x in out.split()]
    return (
        statistics.median(times),
        wall,
        usage.ru_utime + usage.ru_stime,
        sum(busy) / len(busy),
    )


def run(core, rounds):
    names = subprocess.run(
        [BUILDS["opt3"], "journeys"], capture_output=True, text=True, check=True
    ).stdout
    names = [line.split(":")[0] for line in names.splitlines()]
    print(
        "journey,build,round,reps,median_ns,proc_wall_s,proc_cpu_s,others_busy",
        flush=True,
    )
    order = list(BUILDS)
    for name in names:
        probe = once(core, BUILDS["opt3"], name, 3)[0]
        reps = max(5, min(200, int(0.3e9 / max(probe, 1))))
        for r in range(rounds):
            for b in order[r % 3 :] + order[: r % 3]:
                med, wall, cpu, busy = once(core, BUILDS[b], name, reps)
                print(
                    f"{name},{b},{r},{reps},{int(med)},{wall:.3f},{cpu:.3f},{busy:.3f}",
                    flush=True,
                )


def table(path):
    counts = {}
    for b in BUILDS:
        for line in open(f"target/{b}/counts.txt"):
            m = re.match(r"\| ([a-z0-9-]+) \| (\d+) \|", line)
            if m:
                counts[(m.group(1), b)] = int(m.group(2))
    rows = collections.defaultdict(list)
    for r in csv.DictReader(open(path)):
        rows[(r["journey"], r["build"])].append(r)
    names = list(dict.fromkeys(j for j, _ in rows))
    print(
        "| journey | count opt1 | time opt1 | count opt2 | time opt2 | spread | CPU/wall | others busy | follows |"
    )
    print("|---|--:|--:|--:|--:|--:|--:|--:|---|")
    agree = 0
    for j in names:
        t = {b: [int(r["median_ns"]) for r in rows[(j, b)]] for b in BUILDS}
        med = {b: statistics.median(v) for b, v in t.items()}
        spread = max((max(v) - min(v)) / statistics.median(v) for v in t.values())
        cpu = statistics.mean(
            float(r["proc_cpu_s"]) / float(r["proc_wall_s"])
            for b in t
            for r in rows[(j, b)]
        )
        busy = statistics.mean(float(r["others_busy"]) for b in t for r in rows[(j, b)])
        c = {b: counts[(j, b)] for b in t}
        follows = sorted(t, key=lambda b: c[b]) == sorted(t, key=lambda b: med[b])
        agree += follows
        rel = lambda d, b: d[b] / d["opt3"] - 1
        print(
            f"| `{j}` | {rel(c, 'opt1'):+.1%} | {rel(med, 'opt1'):+.1%} | {rel(c, 'opt2'):+.1%} | "
            f"{rel(med, 'opt2'):+.1%} | {spread:.1%} | {cpu:.3f} | {busy:.0%} | {'yes' if follows else 'no'} |"
        )
    print(
        f"\n{agree} of {len(names)} journeys order the three builds alike by count and by time."
    )


if __name__ == "__main__":
    if len(sys.argv) == 4 and sys.argv[1] == "run":
        run(sys.argv[2], int(sys.argv[3]))
    elif len(sys.argv) == 3 and sys.argv[1] == "table":
        table(sys.argv[2])
    else:
        sys.exit("usage: bench/counts-against-time.py run CORE ROUNDS | table CSV")
