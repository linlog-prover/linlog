#!/usr/bin/env bash
# linlog © Fabian Lukas Grubmüller 2026
# Licensed under the EUPL

# Mutation testing of the code soundness rests on: the checker, the
# readers of untrusted input, the search's front door, the Horn engine's
# refutations and the ordinary layer's checker. A mutant that no test
# notices is a behaviour no test pins.
#
#   mutants/run.sh                     # every batch not done yet, detached
#   mutants/run.sh check mist          # only these batches
#   mutants/run.sh --label round-1     # a later run, beside the baseline
#   mutants/run.sh --compare round-1   # its survivors against the baseline
#
# Each batch is one group of files, tested in two passes: first against
# the tests that exercise it (a nextest filterset), then the mutants that
# survive that once more against the whole workspace suite (cargo-mutants'
# --iterate skips what the first pass caught). What survives both is the
# batch's list, copied to mutants/LABEL/BATCH.txt (LABEL `baseline' by
# default), which a later run under another label compares with. A batch
# whose list exists is done and skipped, so a run that was stopped loses
# at most the batch in flight; cargo-mutants' output stays under
# target/mutation/LABEL/BATCH/ (target/mutants/ is where the cargo
# profile of that name builds).
#
# It runs as the user unit `linlog-mutants' on cores 6 to 11, three
# mutants at a time, with 24 GiB and no swap for all of it:
# `journalctl --user -fu linlog-mutants' follows it, `systemctl --user
# stop linlog-mutants' stops it.
set -euo pipefail
self=$(realpath "$0")
cd "$(dirname "$self")/.."
cores=6-11
jobs=3

# name | files (space-separated globs) | the tests that exercise them
batches=(
  "check|core/src/proofs/check.rs core/src/proofs/mod.rs|test(/^proofs::/) | binary_id(linlog::serialize) | binary_id(linlog-cli::cli)"
  "parse|core/src/parse/mod.rs|binary_id(linlog::parse) | binary_id(linlog::depth) | test(/^(parse|sequents|fragment)::/)"
  "serialize|core/src/serialize/*.rs|binary_id(linlog::serialize) | binary_id(linlog::depth)"
  "lltp|core/src/lltp.rs|test(/^lltp::/) | package(linlog-bench)"
  "mist|core/src/mist.rs|test(/^(mist|search::horn)::/) | package(linlog-bench)"
  "ordinary-parse|core/src/ordinary/parse.rs|test(/^ordinary::/) | binary_id(linlog::depth) | binary_id(linlog-cli::cli)"
  "ordinary-derivation|core/src/ordinary/derivation.rs|test(/^ordinary::/) | binary_id(linlog::depth) | binary_id(linlog-cli::cli) | binary_id(linlog-cli::readme)"
  "search|core/src/search/mod.rs|test(/^search::/) | binary_id(linlog::serialize) | binary_id(linlog-cli::cli)"
  "horn-mod|core/src/search/horn/mod.rs|test(/^(search::(horn|reference)|mist)::/)"
  "horn-equation|core/src/search/horn/equation.rs|test(/^(search::(horn|reference)|mist)::/)"
  "horn-cover|core/src/search/horn/cover.rs|test(/^(search::(horn|reference)|mist)::/)"
  "horn-reach|core/src/search/horn/reach.rs|test(/^(search::(horn|reference)|mist)::/)"
)

if [ "${1:-}" = --compare ]; then
  label=${2:?usage: mutants/run.sh --compare LABEL}
  for list in mutants/"$label"/*.txt; do
    name=$(basename "$list" .txt)
    before=mutants/baseline/$name.txt
    echo "$name: $(wc -l <"$before") surviving before, $(wc -l <"$list") now"
    comm -13 "$before" "$list" | sed 's/^/  new: /'
  done
  exit
fi

if [ "${1:-}" != --inside ]; then
  systemctl --user reset-failed linlog-mutants.service 2>/dev/null || true
  systemd-run --user --unit=linlog-mutants --same-dir --collect \
    -p MemoryMax=24G -p MemorySwapMax=0 -p OOMPolicy=continue -p LimitCORE=0 \
    --setenv=PATH="$PATH" "$self" --inside "$@"
  echo "follow it with: journalctl --user -fu linlog-mutants"
  exit
fi
shift
label=baseline
if [ "${1:-}" = --label ]; then
  label=${2:?--label needs a name}
  shift 2
fi

wanted=("$@")
mkdir -p "mutants/$label" "target/mutation/$label"
for batch in "${batches[@]}"; do
  IFS='|' read -r name files filter <<<"$batch"
  if [ ${#wanted[@]} -gt 0 ] && [[ " ${wanted[*]} " != *" $name "* ]]; then
    continue
  fi
  list=mutants/$label/$name.txt
  if [ -e "$list" ]; then
    echo "$name: done, $(wc -l <"$list") surviving"
    continue
  fi
  args=(--package linlog --test-workspace true --no-shuffle --colors never
    --jobs "$jobs" --jobserver-tasks 6 --output "target/mutation/$label/$name")
  for f in $files; do args+=(--file "$f"); done
  start=$SECONDS
  # cargo-mutants exits 2 when a mutant survives, 3 on a timeout: both
  # are results, not failures of the run.
  echo "$name: against its tests"
  NEXTEST_TEST_THREADS=2 taskset -c "$cores" \
    cargo mutants "${args[@]}" -- -E "$filter" || [ $? -le 3 ]
  echo "$name: the survivors against the whole suite"
  NEXTEST_TEST_THREADS=2 taskset -c "$cores" \
    cargo mutants "${args[@]}" --iterate || [ $? -le 3 ]
  out=target/mutation/$label/$name/mutants.out
  cat "$out/missed.txt" "$out/timeout.txt" 2>/dev/null | sort >"$list.tmp"
  mv "$list.tmp" "$list"
  echo "$name: $(wc -l <"$list") surviving, $((SECONDS - start)) s"
done
