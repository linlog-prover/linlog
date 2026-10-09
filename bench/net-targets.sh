#!/usr/bin/env bash
# linlog © Fabian Lukas Grubmüller 2026
# Licensed under the EUPL

# Runs the net engine's target set: the problems of the baseline's
# `engines' and `period-N' passes (bench/baseline.sh) that the net engine
# decided within their minute in bench/results/2026-10-02/, under the same
# test periods, on one thread, into bench/targets/net-LABEL.csv, so that
#
#   bench/net-targets.sh before    # at the commit to compare with
#   bench/net-targets.sh after     # at the commit to judge
#
# give two files whose columns `verdict', `links' and `tests' are equal on
# every decided row exactly when the net engine searches alike: on one
# thread they are a function of the input. A change that must not change
# the search (a retype of the proof structure, say) shows here; the
# focused engine's target set (bench/targets.sh) runs no net search.
#
# One stream on one performance core, no run over 300 s, about five
# minutes. It runs as the user unit `linlog-net-targets', with 8 GiB and
# no swap and 6 GiB of address space per process; `journalctl --user -fu
# linlog-net-targets' follows it, and the unit is gone when the file
# bench/targets/net-LABEL.csv exists. A run of a label that has rows
# already (in net-LABEL.part.csv) resumes it.
set -euo pipefail
self=$(realpath "$0")
cd "$(dirname "$self")/.."
label=${1:?usage: bench/net-targets.sh LABEL}
out=bench/targets
core=2

if [ "${2:-}" != --inside ]; then
  # A copy of the binary, so that builds in the checkout meanwhile do not
  # touch the one the unit runs.
  cargo build --release --locked --package linlog-bench
  install -D "${CARGO_TARGET_DIR:-target}/release/linlog-bench" "target/targets/net-$label/linlog-bench"
  mkdir -p "$out"
  systemctl --user reset-failed linlog-net-targets.service 2>/dev/null || true
  systemd-run --user --unit=linlog-net-targets --same-dir --collect \
    -p MemoryMax=8G -p MemorySwapMax=0 -p OOMPolicy=continue -p LimitCORE=0 \
    --setenv=PATH="$PATH" "$self" "$label" --inside
  echo "into $out/net-$label.csv; follow it with: journalctl --user -fu linlog-net-targets"
  exit
fi

bench=target/targets/net-$label/linlog-bench
part=$out/net-$label.part.csv

# The Partition encodings of the slow tests that the net engine decided:
# `--only' matches by substring, and 1-1 is a part of 1-1-1-1-1-7, which
# it did not.
table=$(mktemp)
trap 'rm -f "$table"' EXIT
grep -E '^partition-table/(1-1|1-3|2-1-1|1-1-4|2-2-1-1|1-2-5|1-1-2-4|1-1-1-5|2-3-2-1);' \
  bench/problems/slow-tests.txt >"$table"

# run ARGS...: one `run' of the net engine on one thread into the file.
run() {
  prlimit --as=$((6 << 30)) taskset -c "$core" "$bench" run "$@" --engines net --jobs 1 \
    --timeout 300 --output "$part" --append --resume 2>>"$out/net-$label.log"
}

run --family partition-yes=4 --family partition-no=3 --family 3-partition-mll-yes=4,6,8 \
  --family 3-partition-mll-no=4,5 --family wide-m1=8,16,24,32,256,2048 \
  --family wide-m2=8,16,24,32,256,2048 --family wide-m3=12,24,30,256,2048 \
  --family wide-m4=12,24,28,256,2048 --problems "$table"
for period in 1 2; do
  run --family wide-m1=256,2048 --family wide-m2=256,2048 --family 3-partition-mll-no=4,5 \
    --family partition-yes=4 --family partition-no=3 --test-period $period
done
for period in 8 16; do
  run --family wide-m1=256,2048 --family wide-m2=256,2048 --family 3-partition-mll-no=4 \
    --test-period $period
done
mv "$part" "$out/net-$label.csv"
