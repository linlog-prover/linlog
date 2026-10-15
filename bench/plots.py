# linlog © Fabian Lukas Grubmüller 2026
# Licensed under the EUPL

"""Draws the comparison's figures from the CSV rows of `linlog-bench run
--tools` alone, reading the columns by their names: a cactus plot per mode
(the problems each tool solves within the limit against the time each
took, sorted) and a scatter plot of linlog on one thread against each other
tool on the problems both cover, both times in log scale, a run not solved
drawn at the limit.

    python3 bench/plots.py RUNS.csv... OUTDIR
"""

import csv
import sys
from collections import defaultdict
from pathlib import Path

import matplotlib

matplotlib.use("svg")
import matplotlib.pyplot as plt  # noqa: E402

# The categorical slots in their fixed order, linlog's two columns first.
SLOTS = [
    "#2a78d6",
    "#eb6834",
    "#1baf7a",
    "#eda100",
    "#e87ba4",
    "#008300",
    "#6250d6",
    "#e34948",
]
MARKERS = ["o", "s", "^", "D", "v", "P", "X", "*"]
SURFACE = "#fcfcfb"
INK = "#0b0b0b"
MUTED = "#52514e"
REFERENCE = "linlog"


def solved(row):
    """Whether a row decided its problem within the time limit and not
    wrongly, as `linlog-bench ranking` counts it."""
    if row["verdict"] not in ("proved", "unprovable") or row.get("judged") == "wrong":
        return False
    return float(row["time_ms"] or "inf") <= float(row["timeout_s"]) * 1000


def judge(rows):
    """Marks as wrong, as `linlog-bench ranking` does, a refutation of a
    problem some tool proved with a checked proof, and a verdict on a
    generated problem against its verdict by construction."""

    def key(r):
        return (r["source"], r["family"], r["problem"], r["mode"])

    checked = {
        key(r) for r in rows if r["verdict"] == "proved" and r["checked"] == "ok"
    }
    for r in rows:
        family = r["source"] == "family"
        verdict, expected = r["verdict"], r["expected"]
        wrong = (verdict == "unprovable" and key(r) in checked) or (
            family
            and r["checked"] != "ok"
            and (verdict, expected)
            in (("unprovable", "provable"), ("proved", "unprovable"))
        )
        r["judged"] = "wrong" if wrong else ""


def style(ax, title, xlabel, ylabel):
    """A recessive frame: muted axes, a light grid, the title in ink."""
    ax.set_facecolor(SURFACE)
    ax.set_title(title, color=INK, loc="left", fontsize=11)
    ax.set_xlabel(xlabel, color=MUTED)
    ax.set_ylabel(ylabel, color=MUTED)
    ax.tick_params(colors=MUTED, labelsize=8)
    for side in ("top", "right"):
        ax.spines[side].set_visible(False)
    for side in ("left", "bottom"):
        ax.spines[side].set_color("#c3c2b7")
    ax.grid(True, color="#e6e5e0", linewidth=0.6)


def cactus(rows, mode, tools, out):
    """The problems of a mode each tool solves, by the time it took."""
    fig, ax = plt.subplots(figsize=(8.5, 4.5), facecolor=SURFACE)
    style(
        ax,
        f"Problems solved within the limit, {mode}",
        "problems solved",
        "time per problem (s, log)",
    )
    for slot, tool in enumerate(tools[: len(SLOTS)]):
        times = sorted(
            float(r["time_ms"]) / 1000
            for r in rows
            if r["tool"] == tool and r["mode"].startswith(mode) and solved(r)
        )
        if not times:
            continue
        ax.plot(
            range(1, len(times) + 1),
            times,
            color=SLOTS[slot],
            linewidth=2,
            marker=MARKERS[slot],
            markevery=max(1, len(times) // 8),
            markersize=5,
            label=f"{tool} ({len(times)})",
        )
    ax.set_yscale("log")
    ax.legend(
        frameon=False,
        fontsize=8,
        labelcolor=INK,
        loc="upper left",
        bbox_to_anchor=(1.01, 1),
    )
    fig.tight_layout()
    fig.savefig(out)
    plt.close(fig)


def scatter(by_problem, tool, out):
    """linlog's time against the tool's on every problem both cover."""
    points = []
    for runs in by_problem.values():
        a, b = runs.get(REFERENCE), runs.get(tool)
        if not a or not b or "refused" in (a["verdict"], b["verdict"]):
            continue
        limit = float(a["timeout_s"])

        def seconds(row):
            return float(row["time_ms"]) / 1000 if solved(row) else limit

        points.append((seconds(b), seconds(a), limit))
    if not points:
        return
    limit = max(p[2] for p in points)
    low = max(1e-3, min(min(p[0], p[1]) for p in points) / 2)
    fig, ax = plt.subplots(figsize=(4.8, 4.8), facecolor=SURFACE)
    style(
        ax,
        f"{REFERENCE} against {tool}, {len(points)} problems",
        f"{tool} (s, log; not solved at the limit)",
        f"{REFERENCE} on one thread (s, log)",
    )
    ax.plot([low, limit], [low, limit], color="#c3c2b7", linewidth=1)
    # A run not solved sits on the limit's line.
    ax.axhline(limit, color="#c3c2b7", linewidth=1, linestyle="--")
    ax.axvline(limit, color="#c3c2b7", linewidth=1, linestyle="--")
    ax.scatter(
        [p[0] for p in points],
        [p[1] for p in points],
        s=14,
        color=SLOTS[0],
        edgecolors=SURFACE,
        linewidths=0.5,
        alpha=0.8,
    )
    ax.set_xscale("log")
    ax.set_yscale("log")
    ax.set_xlim(low, limit * 1.5)
    ax.set_ylim(low, limit * 1.5)
    ax.set_aspect("equal")
    fig.tight_layout()
    fig.savefig(out)
    plt.close(fig)


def main(argv):
    """Reads the files, draws every figure into the directory."""
    if len(argv) < 3:
        sys.exit(__doc__)
    *files, outdir = argv[1:]
    out = Path(outdir)
    out.mkdir(parents=True, exist_ok=True)
    rows = []
    for name in files:
        with open(name, newline="") as f:
            rows.extend(r for r in csv.DictReader(f) if r.get("tool"))
    judge(rows)
    tools = []
    for r in rows:
        if r["tool"] not in tools:
            tools.append(r["tool"])
    # linlog's columns first, so that they keep the first slots.
    order = list(tools)
    tools.sort(key=lambda t: (not t.startswith(REFERENCE), order.index(t)))
    by_problem = defaultdict(dict)
    for r in rows:
        key = (r["source"], r["family"], r["problem"], r["mode"])
        by_problem[key].setdefault(r["tool"], r)
    for mode in ("intuitionistic", "classical"):
        covering = [
            t
            for t in tools
            if any(
                r["tool"] == t
                and r["mode"].startswith(mode)
                and r["verdict"] != "refused"
                for r in rows
            )
        ]
        if covering:
            cactus(rows, mode, covering, out / f"cactus-{mode}.svg")
    for tool in tools:
        if tool != REFERENCE:
            scatter(by_problem, tool, out / f"scatter-{tool}.svg")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
