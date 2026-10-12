# Conducting step 28

You take over conducting step 28 of linlog from the planning session of
the author's first Claude account. That session ends before you start.
You run on the author's second account (`claude2`, whose Claude Code
data is under `~/.claude-2`), so you share none of its conversation, its
memory notes or its scratch directory. Everything you need is in this
repository; where something lives outside it, its path is given below.
The author reads what you write in the terminal, so keep reports short
and in plain words.

## Starting

- **How the author starts you**: in `/home/tux/Projects/own/linlog`,
  from the author's shell, which is nushell:
  ```nu
  claude2 --name planning --model claude-opus-5-5 --effort high --permission-mode auto (open --raw plan/28-conductor.md)
  ```
  - **The name** matters: the step's sessions send their messages to
    `planning`, and two sessions of one name confuse the messaging.
  - **Auto mode** matters too: a message between sessions of different
    permission classes waits for approval.
  - Check with `ListAgents` that no other `planning` exists.
- **Read first**, in this order, before you act:
  1. `CLAUDE.md`.
  2. `plan/handoff.md`: "Step 28: supervising", "What the author has
     asked for, standing" and "Things that bit the earlier session".
  3. `plan/28-audit-and-refactor.md`, the step's prompt, which every
     step session also reads: "Unattended, supervised", every "From the
     review …" section, "From area 3.2, for area 3.3", "Ultra reviews",
     stages 3 and 4, "Deliverables".
  4. `plan/reports/28-audit-and-refactor.md`: the checklist's table
     under "Stage 3 and 4", and each area's "decided unattended".
  5. The last Status entries of `plan/README.md`.
- **Background, as notes rather than instructions**:
  - `plan/notes/step28-supervision-log.md`, the first conductor's log;
  - its memory notes at
    `/home/tux/.claude/projects/-home-tux-Projects-own-linlog/memory/`
    (`MEMORY.md` is the index). Your own account's memory is separate:
    save into it what you will rely on, starting with the standing
    instructions below.
- **Ask the author at your first report**:
  - what this account's plan and usage limits are;
  - whether they want step 28 stopped at some level of the weekly quota
    (the first account had a stop at 80 %, later lifted);
  - when the machine is theirs and when it is free.

## Where step 28 stands (2026-10-12)

- **Done**: stage 0 (requirements, baselines, gates), stage 1 (the
  audit), stage 2 (the design, `plan/notes/api.md`, signed off by the
  author with every decision answered on 2026-10-09), area 3.1 (the
  library's API, data model and wire forms) and area 3.2 (the search,
  preceded by the command's ten wrong answers). Each was reviewed;
  everything through area 3.2's review is signed and pushed (`main` at
  3488a0dd), and CI passed.
- **After `main`**, signed and not pushed:
  - two commits of the author's on the rules files;
  - the ultra-review plan;
  - the commit that adds this file.
- **Paused** by the author since 2026-10-10. Resume only on their word.
- **What remains**:
  - **area 3.3**, efficiency: session `step-28f`, Opus 5.5 at `xhigh`;
  - **area 3.4**, the command, the harness, the flake, the documents and
    the F59 split: session `step-28g`, Opus 5.5 at `high`, with the last
    reader of the rustdoc's front page and README (Fable 5.1 at `high`)
    inside it;
  - **the ultra reviews** (below);
  - **the step's end** (below).
- **The findings**: `plan/reports/28-audit-findings.json`, 1 MB, so
  select from it with `python3 -I` and never read it whole.
  - `F1` to `F226` are the audit's.
  - `H1` to `H21` are the first conductor's held-back audit, and `H22`
    to `H31` its probe.
  - Each has an `area` (`library`, `search`, `efficiency`,
    `command-harness-flake-docs`), a severity, evidence and a fix, and
    the H findings have witness commands. What is fixed is in the
    checklist.

## Starting an area's session

```sh
plan/start-session.sh NAME claude-opus-5-5 EFFORT \
  plan/28-audit-and-refactor.md plan/conduct.md NOTE
```

- The launcher keeps your account (`CLAUDE_CONFIG_DIR`), so the session
  runs on this account too, and it prints the session's id. NOTE is a
  file in your scratch directory titled "Where you start".
- Then:
  - subscribe with `SendMessage` and `notify_when_idle`;
  - arm a watch with `CronCreate` every three hours (the text below);
  - log what you started.
- **What a start note says**:
  - the area, and that the session ends with the area's check rounds,
    a fresh-context reviewer and a passing `nix flake check`, all
    committed, with the checklist current and its last message sent to
    `planning`;
  - read the step prompt's "From the review" sections first, and record
    the note under "From the supervisor" in the step's report;
  - **its findings**: the area's key in the findings file, the items
    earlier areas moved to it (their "decided unattended" and the
    review sections name them), and what the reviews said for it;
  - **the oracles**: the focused engine's target set is
    `bench/targets/head-lto.csv` (`bench/targets.sh`), the net engine's
    `bench/targets/net-after-retype.csv` (`bench/net-targets.sh`);
  - **benchmarks** (target sets, the harness, callgrind outside the
    gate, mutants, fuzzing, anything timed) run only while the author
    leaves the machine free. The session sends `planning` one line when
    a run starts and one when it ends, and on "Pause" or "hold" it stops
    runs gracefully, as the prompt says;
  - **cores**: 0 and 1 are the desktop's (queue a run rather than take
    them), builds go on 2 to 5, agents' programs and long runs on 6 to
    15, everything in memory-capped scopes;
  - **waiting**: a turn that ends while a run goes needs a background
    task waiting on it (`until ! systemctl --user is-active --quiet
    UNIT; do sleep 30; done`), or nothing wakes the session. Sessions
    forgot this three times;
  - **signing**: test a signature before each commit, and commit with
    `--config signing.behavior=drop` while it fails;
  - **departures**: a choice that departs from the signed-off design
    (`plan/notes/api.md`, its §14 answers, the rules' "Decisions") is
    made with the session's recommendation and marked "(departs from the
    design)" in "decided unattended", with what the design said and why.
    The session asks no question about it;
  - **mutation runs** go from a jj workspace of their own at the head,
    never the main checkout, and a mutant that timed out is rerun alone
    before it counts;
  - **usage**: the weekly level, and any stop the author set.
- **Area 3.3, efficiency** starts with:
  - **its first measurement**: the pool's speed after H18 and F92 on a
    quiet machine. That is the focused engine's target set and the
    second baseline's parallel pass at `-j 2` and `-j 4`, at the head and
    at kktkswpq, with times and nodes side by side; the author names the
    quiet window;
  - **the timed validation** of counts against time
    (`bench/counts-against-time.py`), which stage 0 left provisional,
    run again before its first timing;
  - **its findings**: `area == "efficiency"`, F147 to F154;
  - **from area 3.1's review**: read-json, and the printing journeys'
    one printer through a `dyn` atom writer (Typst +2.9 %, LaTeX +2.5 %,
    SVG +3.3 %);
  - **from area 3.2**: F114 and F121, which wait for the stack measured
    on quiet cores.

  Link-time optimisation is on (T3), and the ceilings were recorded
  under it.
- **Area 3.4, the command, harness, flake and documents** takes:
  - **its findings**: `area == "command-harness-flake-docs"` that are
    still open; the command's wrong answers are already fixed;
  - **halves moved to it**: from area 3.1, F5, F16, F27, H25 and F65;
    from area 3.2, `--stats` through `Engine::counters`, the engine
    arguments from `Engine::ALL` and the harness's reason column through
    `Reason::name`;
  - **`--settings FILE`** (R183);
  - **F160 and F181**: CLAUDE.md and `bench.md` still name
    `after-bias.csv` as the oracle, and it is `head-lto.csv` now;
  - **F59**, splitting `derivation.rs`, `check.rs` and `interactive.rs`
    as pure moves, each passing its gate (the author's placement);
  - **the last reader** of stage 4.

## Supervising a running session

- **The watch**: `CronCreate` with this prompt, every three hours at an
  odd minute (it lapses after seven days):

  > Step 28 watch (planning session, plan/handoff.md "Step 28:
  > supervising"). Quietly check the running step-28 session, if any is
  > running and not paused: ListAgents status; new commits (`jj log -r
  > 'main@origin..' --no-graph`), the checklist in
  > plan/reports/28-audit-and-refactor.md; its screen via `zellij
  > --session linlog-<name> action dump-screen --pane-id terminal_0` for
  > a waiting permission prompt or an error; `systemctl --user list-units
  > 'step28-*'` and the load. If it is idle with its part open and not
  > paused, nudge it naming the open items (twice at most, then tell the
  > author with a push notification). If it waits on a prompt,
  > push-notify the author. Re-subscribe notify_when_idle if the last
  > subscription is older than 11 hours. Write a report to the author
  > only if something needs them, or if no report was written for about
  > a day; otherwise say nothing beyond one line.

- **An idle session**: read its screen. A "· N" after "auto mode on"
  means N background tasks, so it is waiting properly. Without one,
  while its runs go, nudge it to arm the waiting recipe.
- **"Pause", "stop" or "hold"** from the author:
  1. send the session "Pause" with `notify_when_idle`;
  2. stop your own heavy runs;
  3. check `systemctl --user list-units 'step28-*' 'linlog-*'` and the
     load, and stop leftover units by name;
  4. tell the author the machine is free.

  On "go", send "Resume". For a short hold, the session stops its runs
  and waits for your go.
- **Signing**: the gpg cache lasts two hours from the author's entry,
  whatever a loop does. The author refreshes it with:
  ```
  ! gpg-connect-agent 'CLEAR_PASSPHRASE --mode=normal 2DD4A80714617BA2CF92FF8C2A544F9421F92E27' /bye; echo x | gpg --local-user flgrubm@grubmueller.dev --sign -o /dev/null
  ```
  After a refresh:
  - keep the idle cache warm with a loop, `systemd-run --user
    --unit=step28-gpg-warm --collect bash -c 'while echo x | gpg --batch
    --pinentry-mode error --local-user flgrubm@grubmueller.dev --sign -o
    /dev/null; do sleep 240; done'`;
  - tell the session until when it can sign.

  Test signing by an actual signature, never by the agent's flag. On a
  gpg lock, run `/home/tux/.claude/hooks/unwedge-gpg-lock.sh`. Never
  touch anything under `~/.gnupg`.
- **The usage figure**: `.claude/settings.local.json` (gitignored)
  draws the status line with a script that writes
  `$XDG_RUNTIME_DIR/claude-rate-limits.json`. The script sits in the
  first conductor's scratch directory, which `/tmp`'s cleanup removes
  about ten days after its last change: copy it into your own
  scratch directory, from
  `/tmp/claude-1000/-home-tux-Projects-own-linlog/e2e2fd87-6303-4e01-bfe1-5c29850a26e0/scratchpad/statusline.py`,
  and point the setting there. Every session in this repository writes
  that one file, of either account, so it shows the account of
  whichever session drew last.

## Reviewing an area

1. **Check for yourself**, since your model is the sessions' and
   independence has to come from methods:
   - build the head's release binary (capped, cores 6 to 11, a
     `CARGO_TARGET_DIR` of your own) and rerun the witnesses of the
     findings the area fixed;
   - run `nix flake check --keep-going` yourself;
   - read the area's checklist rows and its "decided unattended", and
     collect every "(departs from the design)" for the step's end;
   - note each ceiling raise and its reason.

   The first conductor's own `nix flake check` once found a spelling
   failure that the area's gates had missed.
2. **Record** a review commit, "Plan: review step 28, area 3.N":
   - "From the review of area 3.N" in the step prompt, saying what you
     verified and with what, and what the next areas take;
   - a Status entry in `plan/README.md`;
   - the area marked done in the checklist's table.
3. **Sign** with the author there and no session running: `jj sign -r
   'main@origin..@- & ~signed()'`.
4. **Push** only on the author's yes: `jj bookmark set main -r <the
   review commit>`, then `jj git push --bookmark main`. Then watch CI
   with `gh run list` and `gh run view`.
5. **End the session**: `zellij kill-session linlog-NAME && zellij
   delete-session linlog-NAME`.
6. **Recap the next session** with its orchestration, every model and
   effort, before you start it.

## The ultra reviews

The plan is in the step prompt's "Ultra reviews" and in the handoff:
three free runs of `/code-review ultra`, which only the author starts,
each limited to about 8 000 changed lines. They cover:
1. the command's wrong answers and area 3.2 (kktkswpq to 3488a0dd,
   about 8 150 lines, so it may need splitting);
2. area 3.3;
3. area 3.4.

You measure each slice, make a bookmark `review/NAME` at its end, and
give the author the base commit. The author runs it in a separate git
worktree. You record its findings as `U…` in the findings file for the
area that fixes them, then delete the bookmark. Run 1 can go whenever
the author wishes.

## The end of step 28

- **The departures from the design**, from every area, go to the author
  in one list for their review:
  - area 3.1's were answered on 2026-10-10;
  - area 3.2 has three, listed in its review section;
  - add those of areas 3.3 and 3.4.
- **The step's deliverables**, as its prompt lists them, checked
  present and true.
- **`plan/29-comparison.md`**, finished after the last area's review, as
  the handoff says.
- **Leftover workspace directories** (`../linlog-gate2`,
  `../linlog-horn-tests`, `../linlog-horn-tests2`, `../linlog-gen-tests`,
  `../linlog-mutants32`, `../linlog-ordinary-test`, `../linlog-prof`,
  and any later ones) go to the author to delete. You never delete
  them. `../linlog-gate` is a live jj workspace.
- **A push notification** when the step is done.

## Standing instructions of the author

These are on top of CLAUDE.md and the handoff:

- **Reports**: plain words, short. The outcome first, then what you need
  from the author, the evidence, and what is unverified.
- **Workflows**: the Workflow tool only with the author's explicit yes,
  for that run. Never `max` effort. Fable only where the hardest
  reasoning decides.
- **Questions**: the author answers the step's decisions themselves, but
  during the unattended run, departures are noted rather than asked.
- **Pushing and signing**: push only after a review, after signing,
  and on the author's yes. Never sign or rewrite commits while a session
  runs.
- **`gh`**: every administrative task through it (a setting, Pages,
  Actions, branch rules, secrets, releases, a workflow run, a pull
  request) waits for the author's yes, which covers that task only.
  Deleting and transferring are the author's acts.
- **Processes**: kill by PID or unit name only, never with `pkill -f`
  or `killall`. Builds, tests and probes run in memory-capped scopes,
  and long runs run detached. Profiles are read as text.
- **Third-party components**: scan a skill, plugin, agent or hook
  before its first use.
- **The author's email address** is for identifying the author only;
  never send it to any service.
- **The author's shell is nushell.** A command you give the author to
  type in a terminal must work there: `;` between commands (no `&&`),
  `(open --raw FILE)` for a file's text (no `$(cat FILE)`), and no other
  bash-only syntax. A command the author runs through Claude Code's `!`
  prefix runs in Claude Code's own shell, but keep those to the syntax
  both shells share as well.
