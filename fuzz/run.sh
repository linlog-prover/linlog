#!/usr/bin/env bash
# linlog © Fabian Lukas Grubmüller 2026
# Licensed under the EUPL

# Runs every fuzz target until its coverage stops growing: a target stops
# once libFuzzer has found no new edge for STALL seconds, or at CAP
# seconds. Run it in the `fuzz' devshell, after fuzz/seed.sh:
#
#   nix develop .#fuzz -c fuzz/run.sh          # every target not done yet
#   nix develop .#fuzz -c fuzz/run.sh spec     # only these targets
#
# It builds the targets with debug assertions and overflow checks, then
# runs as the user unit `step28-fuzz' on cores 12 to 15, one target per
# core, each in a scope of its own with 4 GiB and no swap (libFuzzer's own
# limit is the same). libFuzzer runs in fork mode and goes on past a
# crash, a timeout (20 s for one input) or an out-of-memory: each is kept
# under fuzz/artifacts/TARGET/, the corpus under fuzz/corpus/TARGET/ and
# the log under target/fuzz/TARGET.log. A target whose line is in
# target/fuzz/summary.tsv is done and skipped, so a stopped run loses at
# most the targets in flight, and their corpora stay.
set -euo pipefail
self=$(realpath "$0")
cd "$(dirname "$self")/.."
cores=(12 13 14 15)
stall=${STALL:-900}
cap=${CAP:-5400}
targets=(json_proof json_session json_structure json_sequent sequent_text
  ordinary_text tptp lltp spec)
bin=fuzz/target/x86_64-unknown-linux-gnu/release
out=target/fuzz

if [ "${1:-}" != --inside ]; then
  (cd fuzz && systemd-run --user --scope -q -p MemoryMax=8G -p MemorySwapMax=0 \
    taskset -c 2-5 cargo fuzz build --debug-assertions)
  mkdir -p "$out"
  systemctl --user reset-failed step28-fuzz.service 2>/dev/null || true
  systemd-run --user --unit=step28-fuzz --same-dir --collect \
    -p OOMPolicy=continue -p LimitCORE=0 \
    --setenv=PATH="$PATH" --setenv=STALL="$stall" --setenv=CAP="$cap" \
    "$self" --inside "$@"
  echo "follow it with: journalctl --user -fu step28-fuzz"
  exit
fi
shift
[ $# -gt 0 ] && targets=("$@")

# one TARGET CORE: fuzz one target until it stalls or reaches the cap, and
# append its line to the summary.
one() {
  local t=$1 core=$2 log=$out/$1.log
  local max_len=4096
  case $t in tptp | lltp | spec | json_*) max_len=16384 ;; esac
  mkdir -p "fuzz/corpus/$t" "fuzz/artifacts/$t"
  systemd-run --user --scope -q --unit="step28-fuzz-$t" -p MemoryMax=4G -p MemorySwapMax=0 \
    taskset -c "$core" "$bin/$t" -fork=1 -ignore_crashes=1 \
    -ignore_timeouts=1 -ignore_ooms=1 -rss_limit_mb=4096 -timeout=20 \
    -max_len="$max_len" -artifact_prefix="fuzz/artifacts/$t/" \
    "fuzz/corpus/$t" >"$log" 2>&1 &
  local pid=$! start=$SECONDS last=0 grown=$SECONDS cov why=exited
  while kill -0 "$pid" 2>/dev/null; do
    sleep 30
    cov=$(grep -oE 'cov: [0-9]+' "$log" | tail -n 1 | cut -d' ' -f2)
    cov=${cov:-0}
    if [ "$cov" -gt "$last" ]; then
      last=$cov
      grown=$SECONDS
    fi
    if [ $((SECONDS - grown)) -ge "$stall" ]; then why=stalled; fi
    if [ $((SECONDS - start)) -ge "$cap" ]; then why=cap; fi
    if [ "$why" != exited ]; then
      systemctl --user stop "step28-fuzz-$t.scope"
      break
    fi
  done
  wait "$pid" || true
  local finds
  finds=$(find "fuzz/artifacts/$t" -type f | wc -l)
  printf '%s\t%s s\t%s\tcov %s\tcorpus %s\tfinds %s\n' "$t" $((SECONDS - start)) \
    "$why" "$last" "$(find "fuzz/corpus/$t" -type f | wc -l)" "$finds" |
    tee -a "$out/summary.tsv"
}

# worker CORE TARGET...: the targets of one core, one after another.
worker() {
  local core=$1 t
  shift
  for t in "$@"; do
    if grep -q "^$t	" "$out/summary.tsv" 2>/dev/null; then
      echo "$t: done"
      continue
    fi
    one "$t" "$core"
  done
}

todo=()
for t in "${targets[@]}"; do
  grep -q "^$t	" "$out/summary.tsv" 2>/dev/null || todo+=("$t")
done
for i in "${!cores[@]}"; do
  mine=()
  for j in "${!todo[@]}"; do
    [ $((j % ${#cores[@]})) -eq "$i" ] && mine+=("${todo[$j]}")
  done
  [ ${#mine[@]} -gt 0 ] && worker "${cores[$i]}" "${mine[@]}" &
done
wait
