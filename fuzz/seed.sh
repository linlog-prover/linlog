#!/usr/bin/env bash
# linlog © Fabian Lukas Grubmüller 2026
# Licensed under the EUPL

# Seeds the fuzz targets' corpora under fuzz/corpus/TARGET with inputs a
# reader accepts, so that the fuzzers start from the inside of each
# format: the sequents of bench/problems, their JSON forms, proofs and
# sessions the command writes, and the problem files of the LLTP, ILTP and
# qcover libraries where they are built (nix build .#lltp -o bench/lltp,
# .#iltp, .#qcover). Run from the repository with the command built:
#
#   cargo build --package linlog-cli && fuzz/seed.sh
set -euo pipefail
cd "$(dirname "$(realpath "$0")")/.."
linlog=${LINLOG:-target/debug/linlog}
corpus=fuzz/corpus
for t in sequent_text ordinary_text tptp lltp spec json_sequent json_proof json_structure json_session json_disproof; do
  mkdir -p "$corpus/$t"
done
seed() { # TARGET NAME: standard input into the corpus
  cat >"$corpus/$1/$2"
}

i=0
grep -v '^#' bench/problems/* | cut -d';' -f5 | sed 's/^ //' | while read -r s; do
  [ -n "$s" ] || continue
  i=$((i + 1))
  printf '%s' "$s" | seed sequent_text "problem-$i"
  "$linlog" seq json "$s" | seed json_sequent "problem-$i"
  # The first byte of a proof seed is the mode: 0 is classical.
  if out=$(timeout 5 "$linlog" prove --format json --timeout 2 "$s"); then
    {
      printf '\0'
      printf '%s' "$out"
    } | seed json_proof "problem-$i"
  elif [ $? -eq 1 ]; then
    # An unprovable outcome reads as its disproof.
    printf '%s' "$out" | seed json_disproof "problem-$i"
  fi
done
for s in "A |- A" "A * B |- B * A" "!A |- A * !A" "A & B |- A + B" "|- 1" "0 |-" \
  "A, A -o B |- B" "?A, !~A |-" "|- T, 0"; do
  i=$((i + 1))
  printf '%s' "$s" | seed sequent_text "small-$i"
  "$linlog" seq json "$s" | seed json_sequent "small-$i"
  if out=$("$linlog" prove --format json "$s"); then
    {
      printf '\0'
      printf '%s' "$out"
    } | seed json_proof "small-$i"
  fi
  if out=$("$linlog" prove -i --format json "$s"); then
    {
      printf '\1'
      printf '%s' "$out"
    } | seed json_proof "small-i-$i"
  fi
  dir=$(mktemp -d)
  printf 'save %s/open.json\nclose\nsave %s/closed.json\n' "$dir" "$dir" |
    "$linlog" interact "$s" >/dev/null 2>&1 || true
  for f in open closed; do
    [ -e "$dir/$f.json" ] && seed json_session "small-$i-$f" <"$dir/$f.json"
  done
  rm -rf "$dir"
done
s=$("$linlog" seq json "A, A -o B |- B")
printf '{"sequent":%s,"mix":false,"links":[[0,2],[3,4]]}' "$s" | seed json_structure net-1
printf '{"sequent":%s,"mix":true,"links":[[0,2]]}' "$s" | seed json_structure net-2
s=$("$linlog" seq json "A * B |- B * A")
printf '{"sequent":%s,"mix":false,"links":[]}' "$s" | seed json_structure net-3

for s in 'a \/ ~a' 'a -> b, a |- b' '~~a |- a' '(a /\ b) -> (b /\ a)' \
  'a <-> a' '|- ((a -> b) -> a) -> a' 'a, b |- a /\ b' 'T' 'F |-'; do
  i=$((i + 1))
  printf '%s' "$s" | seed ordinary_text "small-$i"
done

# Problem files up to 16 KiB, the longest input the runs take.
files() { # TARGET DIR PATTERN COUNT
  [ -d "$2" ] || return 0
  # head closes the pipe early, which ends sort with SIGPIPE.
  local -
  set +o pipefail
  find -L "$2" -name "$3" -size -16k | sort | head -n "$4" |
    while read -r f; do
      seed "$1" "$(echo "$f" | tr / _)" <"$f"
    done
}
files lltp bench/lltp '*.p' 400
files tptp bench/iltp '*.p' 300
files spec bench/qcover '*.spec' 200
for t in "$corpus"/*; do echo "$(basename "$t"): $(find "$t" -type f | wc -l)"; done
