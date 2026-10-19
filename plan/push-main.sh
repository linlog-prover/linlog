#!/usr/bin/env bash
# linlog © Fabian Lukas Grubmüller 2026
# Licensed under the EUPL

# push-main.sh REV
#
# Moves `main` to REV and pushes it, failing within a minute instead of
# waiting on a passphrase prompt when the SSH key is locked, and checks
# that `main@origin` is REV afterwards (a push once failed silently).
# Exit status 0 pushed, 1 the key is locked or the push failed.
set -euo pipefail

if (($# != 1)); then
  echo "usage: $0 REV" >&2
  exit 2
fi
cd "$(dirname "$0")/.."
want=$(jj log --no-graph -r "$1" -T 'commit_id')
# GitHub's `ssh -T` exits 1 even when it authenticates, so the output decides.
if [[ $(timeout 20 ssh -o BatchMode=yes -T git@github.com 2>&1) != *successfully* ]]; then
  echo "push-main: the SSH key is locked; nothing pushed" >&2
  exit 1
fi
jj bookmark set main -r "$want"
GIT_SSH_COMMAND='ssh -o BatchMode=yes' timeout 60 jj git push --bookmark main
got=$(jj log --no-graph -r main@origin -T 'commit_id')
if [[ $got != "$want" ]]; then
  echo "push-main: main@origin is ${got:0:12}, not ${want:0:12}" >&2
  exit 1
fi
echo "push-main: main@origin is ${want:0:12}"
