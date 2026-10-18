"""Item 3's numbers from turns.mjs's output: the default bias in turns
against the better explicit search, on the rows one of them decides."""

import json
import statistics
import sys

rows = [json.loads(line) for line in open(sys.argv[1]) if line.strip()]
decided = lambda r: r["verdict"] in ("proved", "unprovable")
small, lost, neither, wrong = [], [], [], []
for row in rows:
    explicit = [row[k] for k in ("rarer", "factors") if decided(row[k])]
    auto = row["auto"]
    for k in ("rarer", "factors", "auto"):
        r = row[k]
        if (
            decided(r)
            and row["oracle"] in ("proved", "unprovable")
            and r["verdict"] != row["oracle"]
        ):
            wrong.append((row["name"], k, r["verdict"], row["oracle"]))
    if not explicit:
        neither.append((row["name"], decided(auto)))
        continue
    better = min(explicit, key=lambda r: r["ms"])
    if not decided(auto):
        lost.append(row["name"])
        continue
    small.append(
        (
            row["name"],
            row["mode"],
            auto["ms"] / better["ms"],
            auto["work"] / max(better["work"], 1),
            better["ms"],
            auto["ms"],
            "rarer" if better is row["rarer"] else "factors",
        )
    )
print(
    f"rows {len(rows)}; one explicit search decides {len(small) + len(lost)}; neither {len(neither)} "
    f"(of which auto decides {sum(1 for _, a in neither if a)}); auto lost {len(lost)}: {lost}"
)
print(f"verdicts against the oracle: {wrong or 'none differ'}")
if small:
    times = [s[2] for s in small]
    works = [s[3] for s in small]
    print(
        f"time ratio auto/better: median {statistics.median(times):.2f}, max {max(times):.2f}, "
        f"sum {sum(s[5] for s in small):.1f} ms against {sum(s[4] for s in small):.1f} ms "
        f"({sum(s[5] for s in small) / sum(s[4] for s in small):.2f})"
    )
    print(
        f"work ratio auto/better: median {statistics.median(works):.2f}, max {max(works):.2f}"
    )
    over = [s for s in small if s[2] > 5]
    print(f"rows over five times: {len(over)}")
    for s in sorted(small, key=lambda s: -s[2])[:12]:
        print(
            f"  {s[0]} ({s[1]}): {s[2]:.2f}x time, {s[3]:.2f}x work, better {s[6]} {s[4]:.2f} ms, auto {s[5]:.2f} ms"
        )
    rate = [
        row["rarer"]["work"] / row["rarer"]["ms"] * 1000
        for row in rows
        if row["rarer"]["ms"] > 50
    ]
    if rate:
        print(
            f"units per second of the backward search (runs over 50 ms): median {statistics.median(rate):.3g}"
        )
