# linlog © Fabian Lukas Grubmüller 2026
# Licensed under the EUPL

# The `claude-hooks' check: nothing else runs the PreToolUse guards in
# .claude/hooks/, so a regression there would only show as a guard that
# silently stops guarding or starts blocking ordinary commands. Pins, for each
# guard, the commands it blocks and allows, that malformed payloads are
# allowed (a guard fails open), and that settings.json registers it.
{ lib, ... }:
{
  perSystem =
    { pkgs, ... }:
    let
      hooks = ../.claude/hooks;
      settings = lib.importJSON ../.claude/settings.json;

      cases = {
        "block-git.py" = {
          blocked = [
            "git status"
            "git"
            "git --version"
            "git log --oneline -5"
            ''git commit -m "x"''
            "git push origin main"
            "git -C /elsewhere status"
            "git -c user.name=x commit -m y"
            "/usr/bin/git diff"
            ''git "status"''
            "GIT_PAGER=cat git log"
            "echo hi; git stash"
            "true && git add ."
            "(git commit -m x)"
            "echo $(git rev-parse HEAD)"
            "env git status"
            "env - git status"
            "env -S 'git push'"
            "env --split-string='git status'"
            "env -S'git log' -1"
            "timeout 10 git fetch"
            "command git add ."
            "run0 git reset --hard"
            "ls | xargs git add"
            ''bash -c "git push"''
            ''bash -o errexit -c "git status"''
            ''sh -ec "cd core && git diff"''
            "nix shell nixpkgs#git -c git status"
            "nix run nixpkgs#git -- status"
            "nix run nixpkgs#gitFull -- log"
            "jj util exec -- git status"
            "systemd-run --user --scope -p MemoryMax=8G -p MemorySwapMax=0 taskset -c 2-5 git log"
            "systemd-run --user --unit=x -E A=b git status"
            "flock /tmp/lock git fetch"
            "ionice -c 3 git gc"
            "chrt -f 10 git status"
            "nice -n 5 git status"
            "eval git status"
            ''eval "git push"''
            "systemd-inhibit --what=sleep git status"
            "nix develop -c git status"
            "nix --option sandbox false develop -c git status"
            "nix --extra-experimental-features flakes shell nixpkgs#git -c git log"
            "jj -R ../other util exec -- git status"
            "jj --config ui.color=never --repository . util exec git log"
            "nix flake update --commit-lock-file"
            "nix flake lock --commit-lock-file"
          ];
          allowed = [
            "jj st"
            "jj git push"
            "jj git fetch"
            "jj git remote list"
            ''jj commit -m "Fix git-related typo"''
            "jj util exec -- cargo test"
            "jj -R ../other util exec -- cargo test"
            "nix --option sandbox false develop -c cargo test"
            "systemd-run --user --scope -p MemoryMax=8G taskset -c 2-5 cargo test"
            "eval jj st"
            "nix flake update"
            "nix run nixpkgs#gitui"
            "command -v git"
            "cargo test --workspace"
            ''grep -rn "git commit" CLAUDE.md''
            ''grep -rn "git commit\|git push" .claude''
            ''echo "never run git push here"''
            ''jj describe -m "no git; ever"''
            ''grep -n "Bash(git *)" .claude/settings.json''
            "digit status"
          ];
        };

        "block-jj-new-message.py" = {
          blocked = [
            ''jj new -m "msg"''
            "jj new -m msg"
            ''jj new --message "msg"''
            "jj new -mmsg"
            ''jj new @- -m "msg"''
            ''echo hi && jj new -m "msg"''
            "jj --ignore-working-copy new -m x"
            "jj -R ../other new --message=x"
            "taskset -c 2 jj new -m x"
            ''bash -c "jj new -m x"''
            "nix develop -c jj new -m x"
          ];
          allowed = [
            ''jj commit -m "msg"''
            ''jj describe -m "msg"''
            "jj new"
            "jj new main"
            ''jj new && jj describe -m "msg"''
            ''grep "jj new -m" CLAUDE.md''
            ''jj describe -m "not jj new -m"''
            "jj --config signing.behavior=drop commit -m x"
          ];
        };
      };

      malformed = [
        "not json"
        "{}"
        ''{"tool_input":{}}''
        ''{"tool_input":{"command":null}}''
        ''{"tool_input":{"command":["git","status"]}}''
      ];

      payload = command: builtins.toJSON { tool_input = { inherit command; }; };

      # One `<expected exit>\t<hook>\t<payload>' line per case; toJSON escapes
      # tabs and newlines, so every payload stays on its line.
      rows = lib.concatLists (
        lib.mapAttrsToList (
          hook:
          { blocked, allowed }:
          map (c: "2\t${hook}\t${payload c}") blocked
          ++ map (c: "0\t${hook}\t${payload c}") allowed
          ++ map (p: "0\t${hook}\t${p}") malformed
        ) cases
      );

      registered = lib.concatMap (
        entry: lib.optionals (entry.matcher == "Bash") (map (hook: baseNameOf hook.command) entry.hooks)
      ) (settings.hooks.PreToolUse or [ ]);
    in
    {
      checks.claude-hooks =
        pkgs.runCommand "check-claude-hooks"
          {
            nativeBuildInputs = [ pkgs.python3 ];
            cases = pkgs.writeText "claude-hook-cases" (lib.concatLines rows);
            unregistered = lib.subtractLists registered (lib.attrNames cases);
          }
          ''
            fail=0
            # settings.json runs each guard by its path, so it must be
            # executable and say its interpreter (the cases below run it
            # with python3, which tests neither); and every executable
            # script there is a guard with cases (command_line.py is a module
            # the guards import, not executable).
            for hook in ${lib.concatStringsSep " " (lib.attrNames cases)}; do
              if [ ! -x ${hooks}/"$hook" ] || [ "$(head -n 1 ${hooks}/"$hook")" != "#!/usr/bin/env python3" ]; then
                echo "$hook is not executable with #!/usr/bin/env python3" >&2
                fail=1
              fi
            done
            for script in ${hooks}/*.py; do
              name=''${script##*/}
              if [ -x "$script" ] && ! grep -qxF "$name" <<< "${lib.concatLines (lib.attrNames cases)}"; then
                echo "$name is an executable guard without cases here" >&2
                fail=1
              fi
            done
            for hook in $unregistered; do
              echo "$hook is not registered under PreToolUse/Bash in .claude/settings.json" >&2
              fail=1
            done
            while IFS=$'\t' read -r want hook json; do
              got=0
              printf '%s' "$json" | python3 ${hooks}/"$hook" 2>/dev/null || got=$?
              if [ "$got" != "$want" ]; then
                echo "$hook: want exit $want, got $got: $json" >&2
                fail=1
              fi
            done < "$cases"
            [ "$fail" = 0 ]
            touch $out
          '';
    };
}
