#!/usr/bin/env python3
# linlog © Fabian Lukas Grubmüller 2026
# Licensed under the EUPL
"""PreToolUse(Bash) guard: this repository is managed with jj only.

Blocks git, read-only invocations included, behind the wrappers it knows,
and nix's `--commit-lock-file`, which commits through git. The `Bash(git *)`
deny rule already covers the plain form; this also sees git behind a wrapper
(`env`, `timeout`, `systemd-run`, `taskset`, ...), a shell (`bash -c`),
`eval`, `nix shell ... -c`, `nix run nixpkgs#git`, `jj util exec` and an
absolute path, as `command_line.py` reads them.

Exit 2 blocks and shows stderr to Claude. Anything else allows, including
every error: a guard must never wedge the session. The `claude-hooks` flake
check (modules/claude-hooks.nix) pins the cases.
"""

import json
import os
import re
import sys

# No bytecode beside the module: it would land in the checkout.
sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.realpath(__file__)))
from command_line import commands  # noqa: E402

GIT_PACKAGE = re.compile(r"#git(Full|Minimal)?\Z")

GIT_MESSAGE = """\
Blocked: this repository is managed with jj only; git is not used, not even to read.

  git status / log / diff / show  ->  jj st / jj log / jj diff / jj show
  git blame / ls-files            ->  jj file annotate / jj file list
  git rev-parse --show-toplevel   ->  jj root
  git add                         ->  nothing: every jj command snapshots the tree
  git commit -m                   ->  jj commit -m
  git commit --amend              ->  jj squash
  git stash / switch / checkout   ->  jj new / jj new <rev> / jj edit <rev>
  git reset / revert              ->  jj undo, jj op restore <id>, jj revert
  git branch                      ->  jj bookmark
  git fetch / pull / push         ->  jj git fetch / jj git push (ask before pushing)
"""

LOCK_MESSAGE = """\
Blocked: --commit-lock-file makes nix commit through git.

Update the lock, verify, then commit with jj: /update-deps does exactly that.
"""


def verdict(line):
    """0 to allow the command line, 2 for git, 3 for `--commit-lock-file`."""
    for program, rest in commands(line):
        if program == "git":
            return 2
        if program == "nix":
            if "--commit-lock-file" in rest:
                return 3
            if rest[:1] == ["run"] and any(GIT_PACKAGE.search(w) for w in rest):
                return 2
    return 0


def main():
    try:
        command = json.load(sys.stdin)["tool_input"]["command"]
        code = verdict(command) if isinstance(command, str) else 0
    except Exception:
        return 0
    if code:
        print(GIT_MESSAGE if code == 2 else LOCK_MESSAGE, file=sys.stderr, end="")
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main())
