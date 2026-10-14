# linlog © Fabian Lukas Grubmüller 2026
# Licensed under the EUPL
"""The command lines of a Bash call, as the PreToolUse guards read them: the
commands it runs, past separators, quotes, assignments, wrappers (`env`,
`timeout`, `systemd-run`, `taskset`, ...), shells (`bash -c`), `eval`, `nix
shell ... -c`, `nix develop -c` and `jj util exec`. Not a hook itself: the
guards import it.
"""

import os
import re
import shlex

# Programs that run their arguments: (options taking a value, positionals
# before the program).
WRAPPERS = {
    "builtin": (set(), 0),
    "chrt": (
        {"-T", "--sched-runtime", "-P", "--sched-period", "-D", "--sched-deadline"},
        1,
    ),
    "command": (set(), 0),
    "doas": ({"-u", "-C"}, 0),
    "env": ({"-u", "--unset", "-C", "--chdir", "-S", "--split-string"}, 0),
    "exec": ({"-a"}, 0),
    "flock": ({"-w", "--timeout", "-E", "--conflict-exit-code"}, 1),
    "ionice": ({"-c", "--class", "-n", "--classdata", "-p", "--pid"}, 0),
    "nice": ({"-n", "--adjustment"}, 0),
    "nohup": (set(), 0),
    "run0": (
        {"-u", "--user", "-g", "--group", "-D", "--chdir", "--setenv", "--unit"},
        0,
    ),
    "setsid": (set(), 0),
    "stdbuf": ({"-i", "-o", "-e"}, 0),
    "systemd-inhibit": ({"--what", "--who", "--why", "--mode"}, 0),
    "systemd-run": (
        {
            "-p",
            "--property",
            "-u",
            "--unit",
            "-E",
            "--setenv",
            "--slice",
            "-M",
            "--machine",
            "--uid",
            "--gid",
            "--working-directory",
            "--description",
            "--nice",
        },
        0,
    ),
    "sudo": (
        {"-u", "--user", "-g", "--group", "-D", "--chdir", "-C", "-p", "--prompt"},
        0,
    ),
    "taskset": (set(), 1),
    "time": ({"-f", "--format", "-o", "--output"}, 0),
    "timeout": ({"-s", "--signal", "-k", "--kill-after"}, 1),
    "xargs": ({"-a", "-d", "-E", "-I", "-L", "-n", "-P", "-s"}, 0),
}
SHELLS = {"bash", "dash", "ksh", "sh", "zsh"}

# jj's global options that take a value, which may come before the
# subcommand (`jj -R PATH util exec -- ...`).
JJ_GLOBAL_VALUES = {
    "-R",
    "--repository",
    "--at-op",
    "--at-operation",
    "--color",
    "--config",
    "--config-toml",
    "--config-file",
}
ASSIGNMENT = re.compile(r"[A-Za-z_][A-Za-z0-9_]*=\S*\Z")

# `(` separates only where it opens a subshell or `$(`, not inside a word such
# as the permission pattern `Bash(git *)`.
SEPARATOR = re.compile(r"[;&|\n)]|(?<!\w)\(")


def mask_quoted(line):
    """Replace quoted text with `x`, keeping every offset, so that a separator
    inside an argument (`grep "a|git push"`) does not split the line.
    An unterminated quote leaves the rest of the line visible."""
    out, i, n = list(line), 0, len(line)
    while i < n:
        c = line[i]
        if c == "\\":
            i += 2
        elif c in "\"'":
            j = i + 1
            while j < n and line[j] != c:
                j += 2 if c == '"' and line[j] == "\\" else 1
            if j >= n:
                break
            out[i + 1 : j] = "x" * (j - i - 1)
            i = j + 1
        else:
            i += 1
    return "".join(out)


def env_split(tokens, i):
    """The command `env -S STRING` (`--split-string`, either spelling)
    runs when `tokens[i]` is that option: STRING's words, then the rest;
    None for any other option."""
    token = tokens[i]
    if token in ("-S", "--split-string") and i + 1 < len(tokens):
        value, rest = tokens[i + 1], tokens[i + 2 :]
    elif token.startswith("--split-string="):
        value, rest = token.split("=", 1)[1], tokens[i + 1 :]
    elif token.startswith("-S") and len(token) > 2:
        value, rest = token[2:], tokens[i + 1 :]
    else:
        return None
    try:
        return shlex.split(value) + rest
    except ValueError:
        return value.split() + rest


def resolve(tokens):
    """The program a command runs and its arguments, past assignments and
    wrappers; `(None, [])` if it runs nothing (`command -v git`)."""
    i = 0
    while i < len(tokens):
        token = tokens[i]
        if ASSIGNMENT.match(token):
            i += 1
            continue
        program = os.path.basename(token)
        if program in WRAPPERS:
            if program == "command" and tokens[i + 1 : i + 2] in (["-v"], ["-V"]):
                return None, []
            takes_value, positionals = WRAPPERS[program]
            i += 1
            while i < len(tokens) and tokens[i].startswith("-"):
                if tokens[i] == "--":
                    i += 1
                    break
                # `env -S STRING` runs STRING split into words, followed by
                # the rest: a command line of its own.
                split = env_split(tokens, i) if program == "env" else None
                if split is not None:
                    return resolve(split)
                i += 2 if tokens[i] in takes_value else 1
            i += positionals
            continue
        rest = tokens[i + 1 :]
        if program == "nix":
            # nix takes every setting as `--NAME VALUE` before its
            # subcommand (`--option NAME VALUE` too), which no table lists:
            # `shell` or `develop` anywhere before `-c` is taken for it.
            for flag in ("-c", "--command"):
                if flag in rest:
                    at = rest.index(flag)
                    if "shell" in rest[:at] or "develop" in rest[:at]:
                        return resolve(rest[at + 1 :])
        if program == "jj":
            command = jj_subcommand(rest)
            if command[:2] == ["util", "exec"]:
                command = command[2:]
                return resolve(command[1:] if command[:1] == ["--"] else command)
        return program, rest
    return None, []


def jj_subcommand(rest):
    """jj's arguments from its subcommand on, past its global options and
    their values."""
    i = 0
    while i < len(rest) and rest[i].startswith("-"):
        i += 2 if rest[i] in JJ_GLOBAL_VALUES else 1
    return rest[i:]


def shell_script(tokens):
    """The SCRIPT of `bash -c SCRIPT`, under any wrapper, or None."""
    program, rest = resolve(tokens)
    if program not in SHELLS:
        return None
    has_c, j = False, 0
    while j < len(rest):
        token = rest[j]
        if token in ("--", "-"):
            j += 1
            break
        if token.startswith("--"):
            j += 2 if token in ("--rcfile", "--init-file") else 1
        elif token[0] in "-+" and len(token) > 1:
            has_c = has_c or (token[0] == "-" and "c" in token[1:])
            j += 2 if token[-1] in "oO" else 1
        else:
            break
    return rest[j] if has_c and j < len(rest) else None


def commands(line, depth=0):
    """Every command the line runs, as (program, arguments), the commands of
    a shell's script and of `eval` among them, four levels deep at most."""
    start = 0
    for separator in [*SEPARATOR.finditer(mask_quoted(line)), None]:
        end = separator.start() if separator else len(line)
        segment, start = line[start:end], separator.end() if separator else end
        try:
            tokens = shlex.split(segment)
        except ValueError:
            tokens = segment.split()
        script = shell_script(tokens)
        if script is not None and depth < 4:
            yield from commands(script, depth + 1)
        program, rest = resolve(tokens)
        if program == "eval" and depth < 4:
            yield from commands(" ".join(rest), depth + 1)
        elif program is not None:
            yield program, rest
