#!/usr/bin/env python3
# linlog © Fabian Lukas Grubmüller 2026
# Licensed under the EUPL
"""PreToolUse(Bash) guard against `jj new -m`.

`jj new` starts an empty change and `-m` describes that empty change, so the
work meant to be committed stays behind, undescribed, in the previous one.
`jj commit -m` is what was meant.

Exit 2 blocks and shows stderr to Claude. Anything else allows, including
every error. The `claude-hooks` flake check (modules/claude-hooks.nix) pins
the cases.
"""

import json
import os
import sys

# No bytecode beside the module: it would land in the checkout.
sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.realpath(__file__)))
from command_line import commands, jj_subcommand  # noqa: E402

MESSAGE = """\
Blocked: `jj new -m` describes a new, empty change; the work stays behind in the previous one.

  commit the work in @ and start a fresh change:  jj commit -m "msg"
  only describe @:                                jj describe -m "msg"
  a described empty change, on purpose:           jj new && jj describe -m "msg"
"""


def is_jj_new_with_message(command):
    """Whether some command the line runs is `jj new` carrying
    -m/--message, behind any wrapper, shell or global option.

    Each command is read on its own, so a `-m` of another command (`jj new
    && jj describe -m x`) does not count, and quoted text that merely
    mentions it (`grep "jj new -m"`) is no command."""
    for program, rest in commands(command):
        if program != "jj":
            continue
        words = jj_subcommand(rest)
        if words[:1] != ["new"]:
            continue
        if any(
            word in ("-m", "--message")
            or word.startswith("--message=")
            or (word.startswith("-m") and not word.startswith("--"))
            for word in words[1:]
        ):
            return True
    return False


def main():
    try:
        command = json.load(sys.stdin)["tool_input"]["command"]
        blocked = isinstance(command, str) and is_jj_new_with_message(command)
    except Exception:
        return 0
    if blocked:
        print(MESSAGE, file=sys.stderr, end="")
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main())
