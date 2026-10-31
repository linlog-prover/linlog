#!/usr/bin/env bash
# linlog © Fabian Lukas Grubmüller 2026
# Licensed under the EUPL

# Draws every figure of the manual from the benchmarks' CSV rows with
# bench/plots.py, in both schemes: OUTDIR/light/ and OUTDIR/dark/, each
# results chapter's figures in a directory of its own. The flake's
# `manual' package runs it into the book's sources; for a local preview,
# `docs/figures.sh docs/src/figures' (Python with matplotlib on the path).
#
#     docs/figures.sh OUTDIR
set -euo pipefail

out=${1:?usage: docs/figures.sh OUTDIR}
root=$(cd "$(dirname "$0")/.." && pwd)
plots=$root/bench/plots.py
comparison=$root/bench/comparison
results=$root/bench/results

for theme in light dark; do
  python3 -I "$plots" families --theme "$theme" \
    "$results/2026-10-17/families.csv" "$results/2026-10-17/engines.csv" \
    "$out/$theme/baseline-2026-10-17"
  python3 -I "$plots" comparison --theme "$theme" \
    "$comparison/2026-10-15/runs.csv" "$out/$theme/provers-2026-10-15"
  python3 -I "$plots" comparison --theme "$theme" \
    "$comparison/first-order/times-1.csv" "$comparison/first-order/times-2.csv" \
    "$out/$theme/first-order-2026-10-30"
  python3 -I "$plots" growth --theme "$theme" \
    "$comparison/first-order/times-1.csv" "$comparison/first-order/times-2.csv" \
    "$out/$theme/first-order-2026-10-30"
done
