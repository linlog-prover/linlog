# linlog © Fabian Lukas Grubmüller 2026
# Licensed under the EUPL

"""Pinned CPU time of two builds of the harness, side by side.

    python3 bench/pinned-time.py run BASE LIFT RUNS.csv
    python3 bench/pinned-time.py summary RUNS.csv OUT.csv [NOTE...]

`run` takes every row of ROWS five times with both binaries at once,
BASE on core 2 and LIFT on core 3 for the first half of the runs and
the cores swapped for the second, so that a core, a minute or the
machine's load weighs on both alike. Each run is a process of its own:
a target-set row through `linlog-bench run`, whose `cpu_ms` it takes,
or a journey through `linlog-bench journey NAME --repeat N`, whose
measured parts' times it sums (one thread on an idle core: its CPU
time). A focused row runs about a second or more: `cpu_ms` counts in
ticks of 10 ms, so a row of a tenth of a second moves 9 % by one tick,
and a journey of milliseconds is repeated until it sums to a second. Every run's row is appended to RUNS.csv as it ends, so a run
started again skips what is there: it runs as a user unit that
`systemctl --user stop` stops and a second start resumes.

`summary` writes per row the median of each build's runs, their spread
(the largest minus the smallest over the median), the ratio of the
medians LIFT over BASE and the range of the ratios of the runs taken
together, and the same summed over the rows of each group: the sums of
the medians and the range of each round's sums. Its first lines are the
notes given and the machine's load, as `#` lines.
"""

import csv
import os
import statistics
import subprocess
import sys
import tempfile

RUNS = 5
CORES = (2, 3)
LLTP = "bench/lltp/ILL"

# (name, group, arguments): a journey's name with how many times it is
# repeated, or a row's arguments to `linlog-bench run`. The groups:
# `focused` the focused engine's searches, one- and two-sided (the
# families' rows forced onto it where the dispatch sends them elsewhere),
# each about a second or more, `other` the other engines' journeys, the
# end-to-end ones and the target set's rows over a second, which the Horn
# engine decides. The family `growing` has no focused row: its size is a
# copy bound, and its search takes about 0.14 s at every size from 2 048
# to 16 384.
ROWS = [
    ("search-qbf-20-2", "focused", 36),
    ("search-chain-128", "focused", 30),
    ("search-partition-no-5", "focused", 2400),
    ("search-chain-64-intuitionistic", "focused", 200),
    ("chain/512", "focused", ["--family", "chain=512", "--timeout", "300"]),
    (
        "qbf/34#1",
        "focused",
        ["--family", "qbf=34", "--only", "qbf/34#1", "--timeout", "300"],
    ),
    (
        "mix/9",
        "focused",
        ["--family", "mix=9", "--engines", "focus", "--timeout", "300"],
    ),
    (
        "chain/384-intuitionistic",
        "focused",
        [
            "--family",
            "chain=384",
            "--modes",
            "intuitionistic",
            "--engines",
            "two-sided",
            "--timeout",
            "300",
        ],
    ),
    (
        "chain/512-intuitionistic",
        "focused",
        [
            "--family",
            "chain=512",
            "--modes",
            "intuitionistic",
            "--engines",
            "two-sided",
            "--timeout",
            "300",
        ],
    ),
    ("search-wide-m2-256", "other", 1),
    ("search-spec-chain", "other", 1),
    ("search-additive-14", "other", 1),
    ("batch-families", "other", 1),
    ("ordinary-pigeons", "other", 1),
    (
        "Peterson-4_50_1",
        "other",
        [
            "--lltp",
            LLTP,
            "--copies",
            "3",
            "--timeout",
            "5",
            "--only",
            "ILL/petri-nets/MCC/Peterson-4_50_1.p",
        ],
    ),
    (
        "JoinFreeModules_joinFree-10_50_1",
        "other",
        [
            "--lltp",
            LLTP,
            "--copies",
            "3",
            "--timeout",
            "5",
            "--only",
            "ILL/petri-nets/MCC/JoinFreeModules_joinFree-10_50_1.p",
        ],
    ),
]

FIELDS = [
    "round",
    "row",
    "group",
    "build",
    "core",
    "cpu_ms",
    "verdict",
    "nodes",
    "load",
]


def start(binary, core, args, scratch):
    """Starts one run of a row on a core; returns the process and where its CSV goes."""
    name, _, run_args = args
    pinned = ["taskset", "-c", str(core), binary]
    if isinstance(run_args, int):
        return subprocess.Popen(
            [*pinned, "journey", name, "--repeat", str(run_args)],
            stdout=subprocess.PIPE,
            text=True,
        ), None
    out = os.path.join(scratch, f"{core}.csv")
    if os.path.exists(out):
        os.remove(out)
    command = [*pinned, "run", *run_args, "--jobs", "1", "--output", out]
    return subprocess.Popen(command, stdout=subprocess.DEVNULL, text=True), out


def finish(process, out):
    """Waits for a run; returns its CPU time in milliseconds, its verdict and its nodes."""
    stdout, _ = process.communicate()
    if process.returncode != 0:
        raise SystemExit(f"a run failed with status {process.returncode}")
    if out is None:
        return sum(float(ns) for ns in stdout.split()) / 1e6, "", ""
    with open(out, newline="") as file:
        (row,) = list(csv.DictReader(file))
    return float(row["cpu_ms"]), row["verdict"], row["nodes"]


def run(base, lift, path):
    """Takes every run not yet in the file at `path`."""
    done = set()
    if os.path.exists(path):
        with open(path, newline="") as file:
            done = {
                (int(r["round"]), r["row"])
                for r in csv.DictReader(file)
                if r["build"] == "lift"
            }
    jobs = [(i, row) for i in range(RUNS) for row in ROWS]
    new = not os.path.exists(path)
    with open(path, "a", newline="") as file, tempfile.TemporaryDirectory() as scratch:
        writer = csv.DictWriter(file, FIELDS)
        if new:
            writer.writeheader()
        for at, (i, row) in enumerate(jobs):
            if (i, row[0]) in done:
                continue
            cores = CORES if at < len(jobs) // 2 else CORES[::-1]
            load = os.getloadavg()[0]
            os.makedirs(os.path.join(scratch, "base"), exist_ok=True)
            os.makedirs(os.path.join(scratch, "lift"), exist_ok=True)
            started = [
                (build, core, start(binary, core, row, os.path.join(scratch, build)))
                for build, binary, core in (
                    ("base", base, cores[0]),
                    ("lift", lift, cores[1]),
                )
            ]
            for build, core, (process, out) in started:
                cpu, verdict, nodes = finish(process, out)
                writer.writerow(
                    {
                        "round": i,
                        "row": row[0],
                        "group": row[1],
                        "build": build,
                        "core": core,
                        "cpu_ms": f"{cpu:.3f}",
                        "verdict": verdict,
                        "nodes": nodes,
                        "load": f"{load:.2f}",
                    }
                )
            file.flush()


def spread(values):
    """The largest minus the smallest value over the median."""
    return (max(values) - min(values)) / statistics.median(values)


def summary(path, out, notes):
    """Writes the notes, the medians, spreads and ratios of the runs at `path` to `out`."""
    with open(path, newline="") as file:
        runs = list(csv.DictReader(file))
    loads = [float(r["load"]) for r in runs]
    by = {}
    for r in runs:
        by.setdefault((r["row"], r["build"]), {})[int(r["round"])] = float(r["cpu_ms"])
    rows = []
    for name, group, _ in ROWS:
        base, lift = by[(name, "base")], by[(name, "lift")]
        rounds = sorted(set(base) & set(lift))
        b, l = [base[i] for i in rounds], [lift[i] for i in rounds]
        ratios = [y / x for x, y in zip(b, l)]
        rows.append((name, group, len(rounds), b, l, ratios))
    with open(out, "w", newline="") as file:
        for note in notes:
            file.write(f"# {note}\n")
        file.write(
            f"# load, the 1-minute average at each run's start: {min(loads):.2f} to {max(loads):.2f}, "
            f"median {statistics.median(loads):.2f}\n"
        )
        writer = csv.writer(file)
        writer.writerow(
            [
                "row",
                "group",
                "runs",
                "base_median_ms",
                "base_spread",
                "lift_median_ms",
                "lift_spread",
                "ratio",
                "ratio_min",
                "ratio_max",
            ]
        )
        for name, group, n, b, l, ratios in rows:
            writer.writerow(
                [
                    name,
                    group,
                    n,
                    f"{statistics.median(b):.3f}",
                    f"{spread(b):.4f}",
                    f"{statistics.median(l):.3f}",
                    f"{spread(l):.4f}",
                    f"{statistics.median(l) / statistics.median(b):.4f}",
                    f"{min(ratios):.4f}",
                    f"{max(ratios):.4f}",
                ]
            )
        # The sums over a group: of the medians, and of each round's runs
        # for the range of the ratios.
        for group in ("focused", "other", "all"):
            members = [r for r in rows if group in ("all", r[1])]
            b = sum(statistics.median(r[3]) for r in members)
            l = sum(statistics.median(r[4]) for r in members)
            n = min(r[2] for r in members)
            ratios = [
                sum(r[4][i] for r in members) / sum(r[3][i] for r in members)
                for i in range(n)
            ]
            writer.writerow(
                [
                    f"sum:{group}",
                    group,
                    n,
                    f"{b:.3f}",
                    "",
                    f"{l:.3f}",
                    "",
                    f"{l / b:.4f}",
                    f"{min(ratios):.4f}",
                    f"{max(ratios):.4f}",
                ]
            )


if __name__ == "__main__":
    match sys.argv[1:]:
        case ["run", base, lift, path]:
            run(base, lift, path)
        case ["summary", path, out, *notes]:
            summary(path, out, notes)
        case _:
            raise SystemExit(__doc__)
