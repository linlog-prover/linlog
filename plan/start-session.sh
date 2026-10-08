#!/usr/bin/env bash
# linlog © Fabian Lukas Grubmüller 2026
# Licensed under the EUPL

# start-session.sh NAME MODEL EFFORT PROMPT_FILE...
#
# Starts an interactive Claude Code session named NAME in the repository,
# in auto mode, with the files concatenated as its prompt, inside the
# detached zellij session `linlog-NAME` (`zellij attach linlog-NAME` to
# watch it or type into it). The planning session runs this to start a
# step's sessions itself.
set -euo pipefail

if (($# < 4)); then
  echo "usage: $0 NAME MODEL EFFORT PROMPT_FILE..." >&2
  exit 2
fi
name=$1 model=$2 effort=$3
shift 3

# A session started from another one inherits markers (CLAUDECODE,
# CLAUDE_CODE_CHILD_SESSION, its messaging socket, …) that make the new
# one a child: no transcript, no entry in the session registry, so no
# --resume and no messages by name. Clear them, so that the new session
# is what the author would start from a terminal.
while IFS= read -r var; do
  case $var in CLAUDE* | AI_AGENT) unset "$var" ;; esac
done < <(compgen -e)

prompt=""
for file in "$@"; do
  prompt+="$(cat "$file")"$'\n'
done

cd "$(dirname "$0")/.."
zellij attach --create-background "linlog-$name" >/dev/null
zellij --session "linlog-$name" run --name "$name" --cwd "$PWD" -- \
  claude --model "$model" --effort "$effort" --permission-mode auto \
  --name "$name" "$prompt"
