#!/usr/bin/env bash
# linlog © Fabian Lukas Grubmüller 2026
# Licensed under the EUPL

# The comparison of linlog with the other provers (bench/TOOLS.md): every
# problem of the LLTP library (intuitionistic and classical), qcover's
# coverability problems and the generated families, run by every tool whose
# fragment covers it, each run measured by BenchExec's runexec under one
# time and memory limit. The same script runs on the author's machine and
# in the CI job; nothing in it is specific to a machine.
#
#   bench/comparison.sh [--out DIR] [--shard K/N] [--cores LIST]
#                       [--default-cores LIST] [--timeout S] [--memory BYTES]
#                       [--quick]
#
# --shard K/N    this job's share of the problems (every N-th, from the K-th)
# --cores LIST   one stream of sequential runs per core, comma-separated
#                (default: the first core this process may use); each stream
#                takes every problem of its share with every sequential tool
# --default-cores LIST  the cores of the stream of linlog's default, which
#                runs a pool beside its first thread: CPU lists separated by
#                `/', one stream each (default: none, the column left out)
# --timeout S    the time limit of every run (default 10)
# --memory B     the memory limit of every run in bytes (default 4 GiB)
# --quick        three problems of each source only: a check that every
#                tool runs
#
# It builds the flake's `comparison', `lltp' and `qcover' packages, runs the
# streams side by side, joins their rows into DIR/runs.csv, and writes
# DIR/ranking.md (`linlog-bench ranking') and DIR/figures/ (bench/plots.py).
# A stream's file is resumed when the script runs again on the same DIR.
set -euo pipefail

out=bench/comparison
shard=1/1
cores=
default_cores=
timeout=10
memory=$((4 << 30))
quick=
while [ $# -gt 0 ]; do
  case $1 in
  --out)
    out=$2
    shift
    ;;
  --shard)
    shard=$2
    shift
    ;;
  --cores)
    cores=$2
    shift
    ;;
  --default-cores)
    default_cores=$2
    shift
    ;;
  --timeout)
    timeout=$2
    shift
    ;;
  --memory)
    memory=$2
    shift
    ;;
  --quick) quick=1 ;;
  *)
    echo "unknown option $1" >&2
    exit 2
    ;;
  esac
  shift
done

root=$(cd "$(dirname "$0")/.." && pwd)
mkdir -p "$out"
out=$(cd "$out" && pwd)
build() { nix build --no-link --print-out-paths "$root#$1"; }
tools=$(build comparison)
lltp=$(build lltp)
qcover=$(build qcover)
export PATH="$tools/bin:$PATH"
export LINLOG_COMPARISON_TOOLS="$tools/share/linlog-comparison/tools.json"
# The commit the tools were built from, where the caller knows it.
export LINLOG_COMMIT=${LINLOG_COMMIT:-${GITHUB_SHA:-}}

sequential=linlog,maude-ill,maude-cll,click-and-collect,llprover-cll,llprover-ill,lintap,linearone,sympli,lolli,apll-ll,apll-ill
# Each stream runs two passes: the small sets first, then the
# intuitionistic library, whose Petri nets come last, so that a night that
# ends early leaves Petri nets for the next.
small=(--lltp "$lltp/CLL" --spec "$qcover/stable/benchmarks" --all-families)
large=(--lltp "$lltp/ILL")
if [ -n "$quick" ]; then
  small=(--lltp "$lltp/CLL/misc" --spec "$qcover/stable/benchmarks/mist" --family mix=4 --family chain=16)
  large=(--lltp "$lltp/ILL/ILLTP-SYJ-01/SYJ101+1.p" --lltp "$lltp/ILL/ILLTP-SYN-01")
fi
# The job's shard of N, which its streams split once more.
k=${shard%/*} n=${shard#*/}
[ -z "$cores" ] && cores=$(taskset -cp $$ | sed -E 's/.*: ([0-9]+).*/\1/')
IFS=, read -ra streams <<<"$cores"
IFS=/ read -ra defaults <<<"$default_cores"

# One stream: the i-th of `count' parts of the job's share with some tools,
# pinned to cores. Shard i·n + k of count·n is every (count·n)-th problem
# from that place: together the parts are the share, whatever the count.
stream() {
  local name=$1 cpus=$2 tools=$3 index=$4 count=$5 pass
  for pass in small large; do
    local file="$out/$name-$pass.csv" resume=() sources
    [ -s "$file" ] && resume=(--append --resume)
    if [ "$pass" = small ]; then sources=("${small[@]}"); else sources=("${large[@]}"); fi
    taskset -c "$cpus" linlog-bench run --tools "$tools" "${sources[@]}" \
      --shard "$((index * n + k))/$((count * n))" \
      --timeout "$timeout" --memory-limit "$memory" \
      --output "$file" "${resume[@]}" --work "$out/work-$name" 2>>"$out/$name.log"
  done
}

pids=()
for i in "${!streams[@]}"; do
  stream "seq-$k-$((i + 1))" "${streams[$i]}" "$sequential" "$i" "${#streams[@]}" &
  pids+=($!)
done
for i in "${!defaults[@]}"; do
  stream "default-$k-$((i + 1))" "${defaults[$i]}" linlog-default "$i" "${#defaults[@]}" &
  pids+=($!)
done
status=0
for pid in "${pids[@]}"; do
  wait "$pid" || status=1
done
{
  head -n 1 "$out/seq-$k-1-small.csv"
  for f in "$out"/seq-*.csv "$out"/default-*.csv; do
    [ -f "$f" ] && tail -n +2 "$f"
  done
} >"$out/runs.csv"
linlog-bench ranking "$out/runs.csv" >"$out/ranking.md"
python3 -I "$root/bench/plots.py" comparison "$out/runs.csv" "$out/figures"
exit "$status"
