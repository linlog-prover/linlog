#!/usr/bin/env bash
# linlog © Fabian Lukas Grubmüller 2026
# Licensed under the EUPL

# start-session.sh NAME MODEL EFFORT PROMPT_FILE...
# start-session.sh --resume SESSION_ID NAME MODEL EFFORT
#
# Starts an interactive Claude Code session named NAME in the repository,
# in auto mode, with the files concatenated as its prompt, or resumes the
# session SESSION_ID (after a reboot, say), inside the detached zellij
# session `linlog-NAME`, the prefix not doubled (`zellij attach
# linlog-step-42c` to watch it or type into it), and prints the session's id once it has registered. The
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
# is what the author would start from a terminal. The account
# (CLAUDE_CONFIG_DIR, ~/.claude when unset) is kept, so that the new
# session runs on the same account as the one that starts it.
account=()
if [[ -n ${CLAUDE_CONFIG_DIR-} ]]; then
  account=("CLAUDE_CONFIG_DIR=$CLAUDE_CONFIG_DIR")
fi
config_dir=${CLAUDE_CONFIG_DIR:-$HOME/.claude}
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
# The zellij session is named after the session, with linlog's prefix
# once: `linlog-step-42c` stays as it is, `step-42c` gains it.
zsession=linlog-${name#linlog-}
# An exited session of the same name would be resurrected with its old
# panes, one of them an earlier session's command waiting for Enter.
# delete-session refuses a running one.
zellij delete-session "$zsession" >/dev/null 2>&1 || true
zellij attach --create-background "$zsession" >/dev/null
zellij --session "$zsession" run --name "$name" --cwd "$PWD" -- \
  env "${account[@]}" \
  claude "${resume[@]}" --model "$model" --effort "$effort" \
  --permission-mode auto --name "$name" "${prompt[@]}"

# The session registers itself under the account's sessions/PID.json.
for _ in $(seq 60); do
  sleep 1
  id=$(
    python3 - "$name" "$config_dir" <<'EOF'
import glob, json, sys
for path in glob.glob(f"{sys.argv[2]}/sessions/*.json"):
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
