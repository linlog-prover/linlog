#!/usr/bin/env bash
# linlog © Fabian Lukas Grubmüller 2026
# Licensed under the EUPL

# start-session.sh NAME MODEL EFFORT PROMPT_FILE...
# start-session.sh --resume SESSION_ID NAME MODEL EFFORT
#
# Starts an interactive Claude Code session named NAME in the repository,
# in auto mode, with the files concatenated as its prompt, or resumes the
# session SESSION_ID (after a reboot, say), inside the detached zellij
# session `linlog-NAME` (`zellij attach linlog-NAME` to watch it or type
# into it), and prints the session's id once it has registered. The
# planning session runs this to start a step's sessions itself.
set -euo pipefail

resume=()
if [[ ${1-} == --resume ]]; then
  resume=(--resume "$2")
  shift 2
fi
if ((${#resume[@]} ? $# != 3 : $# < 4)); then
  echo "usage: $0 NAME MODEL EFFORT PROMPT_FILE..." >&2
  echo "       $0 --resume SESSION_ID NAME MODEL EFFORT" >&2
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

prompt=()
if ((! ${#resume[@]})); then
  text=""
  for file in "$@"; do
    text+="$(cat "$file")"$'\n'
  done
  prompt=("$text")
fi

cd "$(dirname "$0")/.."
zellij attach --create-background "linlog-$name" >/dev/null
zellij --session "linlog-$name" run --name "$name" --cwd "$PWD" -- \
  claude "${resume[@]}" --model "$model" --effort "$effort" \
  --permission-mode auto --name "$name" "${prompt[@]}"

# The session registers itself under ~/.claude/sessions/PID.json.
for _ in $(seq 60); do
  sleep 1
  id=$(
    python3 - "$name" <<'EOF'
import glob, json, sys
for path in glob.glob(f"{__import__('os').path.expanduser('~')}/.claude/sessions/*.json"):
    try:
        record = json.load(open(path))
    except (OSError, ValueError):
        continue
    if record.get("name") == sys.argv[1]:
        print(record["sessionId"])
        break
EOF
  )
  if [[ -n $id ]]; then
    echo "session $name: $id"
    exit 0
  fi
done
echo "session $name: not registered after a minute" >&2
exit 1
