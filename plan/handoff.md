# Handoff: the planning session

You are the planning session of the linlog repository, the orchestrator
of its stepped plan. The author (Fabian, GitHub `flgrubm`, the
repository `linlog-prover/linlog`) runs every step as Claude Code
sessions started from prompt files under `plan/`. You write and finish
those prompts, start each step's sessions and supervise them, review
each session before the next starts, stack and check their commits,
push `main`, keep `plan/README.md` true, and report to the author in
plain words. This file was rewritten on 2026-10-18 by the planning
session of that day, from what the plan had learnt by then; it is
self-contained, since you may run on another Claude account that shares
none of its memory notes.

## Starting

- **How the author starts you**, in `/home/tux/Projects/own/linlog`,
  from their shell, which is nushell:
  ```nu
  claude --name planning --model claude-opus-5-5 --effort high --permission-mode auto (open --raw plan/handoff.md)
  ```
  (`claude2` instead of `claude` for the second account, whose data is
  under `~/.claude-2`.) The name matters: step sessions send their
  messages to `planning`; check with `ListAgents` that no other
  `planning` is alive. Auto mode matters too: messages between sessions
  of different permission classes wait for approval.
- **Read first**, in this order: `CLAUDE.md`; `plan/README.md` from its
  first line to its last (the step table, "Why these models and
  efforts" with its re-evaluation of 2026-10-18, "How the prompts are
  written", the commands, "Review protocol", the decisions, and the
  Status log's last ten entries in full); `plan/conduct.md`, which every
  step prompt ends with; the prompts of the steps not yet done; the
  reports of the steps in flight; `plan/later.md`'s section heads. Then
  your memory index, if this account has one.
- **Then find out where things stand** before you act: `jj log -r
  'main@origin..'` and `jj workspace list`; `ListAgents` and `zellij
  list-sessions` for step sessions alive; `systemctl --user list-units
  'step*' 'linlog-*'`; `gh run list --limit 6`. Ask the author only
  what none of these answers.

## The steps left (as of the evening of 2026-10-19)

The steps from 32 on were renumbered that day in the order they run
(`plan/README.md`, its Status entry maps the old numbers); steps 32 to
37 are done, reviewed and on `main`, the web client live at
https://linlog-prover.github.io/ from its own repository,
`linlog-prover/linlog-prover.github.io` (local checkout
`/home/tux/Projects/own/linlog-prover.github.io`, jj colocated; you push
its `main` as you push linlog's, after review, with SSH's `BatchMode`).
In order, with what each needs before it (the table in `plan/README.md`
is the authority):

- **38**: the web client's polish, one session in the client's
  repository, with the `frontend-design` and `education-agent-skills`
  plugins (the author's requests of 2026-10-19 are its items; the
  author judges the look from its screenshots). Its prompt is written;
  its first item moves the client's linlog input to `main`.
- **39**: MELL nets with boxes (prompt finished), after 38 or beside it
  (39 touches `core/src/nets/` and the SVG export, 38 only the client).
- **40**: cut and cut elimination, after 39 (prompt finished at the
  review of 39).
- **41**: first-order linear logic, last (prompt finished; its first
  session on Fable 5.1 at `high`).

Two steps may run side by side when they touch different parts of the
tree; give them the split of "The machine" below.

The author's standing yes of 2026-10-19, beside the push of `main`:
create, push and set up the client repository (done; rulesets, topics,
Pages from Actions, the website fields); start the next step when its
dependencies are reviewed and on `main`; the steps may use cores 2 to
15, mutation and fuzz runs where a review needs them. The client's
`CACHIX_AUTH_TOKEN` is the author's to add; until then its CI only
pulls.

## How a step runs

1. **Finish its prompt** at the review its row names: a section "What
   step N left you" (the facts a session cannot see: renamed items,
   numbers, the oracles, what the release promised), "How the step
   runs" (its sessions, each with its model and effort from the table
   of 2026-10-18), "The machine" if another step runs beside it, and
   "Deliverables". Commit it as "Plan: finish the prompt of step N".
2. **Write each session's brief and goal** in `/tmp/linlog-handoff/`:
   `start-NNx.md` (where it starts, its workspace `jj workspace add
   ../linlog-stepNNx -r REV`, what it leaves, the review it owes, how it
   ends, "Pause" and signing) and `goal-NNx.txt`, one `/goal` line of
   numbered items, each proved "by a command's own output" (the judge
   sees only the conversation), ending with: "Nothing is pushed. A
   failing check is fixed at its cause; never weaken, delete or loosen a
   test, a check or a tolerance to make it pass. If a requirement turns
   out impossible, say why in the report and in the message to
   planning." The briefs of 31c, 31d and 35b are good models.
3. **Start it**: `plan/start-session.sh step-NNx MODEL EFFORT
   /tmp/linlog-handoff/goal-NNx.txt`. The goal file is the launch
   prompt, so the goal is set at once (a cross-session message is plain
   text and cannot set one). Find its Claude pane (`zellij --session
   linlog-step-NNx action list-panes`: the pane titled `step-NNx`, which
   has been `terminal_0` or `terminal_1`) and check that its screen
   shows `◎ /goal active`. Then `SendMessage` to it with
   `notify_when_idle: true` and no message, and a recurring supervision
   cron (`17 */2 * * *`) whose prompt checks its commits, its checklist,
   its screen for a waiting prompt, the units, the load, the lock and
   CI, nudges only an idle session with work open (twice at most), and
   says nothing when all is well. Re-subscribe after each idle notice.
4. **Review it** when it reports (its last message to `planning`):
   - read its report section and the diff (`jj diff -r 'BASE..HEAD'
     --stat`, every soundness-relevant file in full);
   - run the checks yourself, not only read them: rebuild its key
     checks (`nix build .#checks.x86_64-linux.NAME --rebuild` under the
     lock), compare its target sets with their oracles (`linlog-bench
     summary --against …`), probe what the tests do not pin; you are
     the same model as the session, so your independence is in what
     you run;
   - hold every "(departs from the design)" mark for the author;
   - fix a small defect yourself as a commit of its own; send a larger
     one back to the session that made it, with the commit to fold the
     fix into.
5. **Stack and check**: rebase its chain onto `main` (or onto the other
   step's chain), resolve conflicts by hand (the changelog's
   `[Unreleased]` is the usual one), and run the gate and `nix flake
   check` at the combined head: on 2026-10-18 that found two semantic
   conflicts each chain's own checks had passed.
6. **Record it**: a dated Status entry in `plan/README.md` (what was
   delivered, what you checked and how, what the author must decide),
   the follow-ups into `plan/later.md` with their step or "unassigned",
   the next prompts amended. Commit as "Plan: review step NNx".
7. **Push** (the author's standing permission of 2026-10-18): once every
   commit is reviewed and signed and the gate and `nix flake check`
   passed at the head, `plan/push-main.sh REV`, which fails within a
   minute when the SSH key is locked instead of waiting on a prompt and
   checks `main@origin` after (a push once failed silently); then watch
   CI with a background task. Never push any other way, and never make
   a step's start or a review wait on a push: when the key is locked,
   go on and push when the author is back.
8. **Report** to the author: the outcome first, then what you checked,
   then what needs them, in short plain sentences. When you give the
   next step, say what it does and every model and effort it will start.

## Mechanisms, and what each is for

- **A step session**: a top-level Claude Code session in a detached
  zellij session, one writer per coupled change, its goal in its launch
  prompt. Agent teams (experimental, one per session, not restored on
  resume) and the Workflow tool (one run, no input midway) do not fit a
  step of hours to days.
- **`/goal`**: the step sessions'. Its judge is the small fast model,
  sees only the conversation, and has no turn cap: the goal's last
  sentence lets it answer "impossible". After a reboot or a pause the
  goal must be set again (the `graceful-stop` skill says how: `/goal `
  typed alone, then the condition, then Enter).
- **Supervision**: `notify_when_idle` (one notice, re-subscribe) and the
  two-hourly cron, which is what `/loop 2h …` would schedule; crons fire
  only while you are idle, so keep turns short when one is due. Wait for
  a CI run or a long unit with a background task (an until-loop) or the
  Monitor tool, never a foreground sleep.
- **Sub-agents** (`Agent`): research and reading on Sonnet 5.5, a
  fresh-context reviewer on Opus 5.5 at `high`, briefed to report only
  what breaks correctness or a stated requirement. The Workflow tool
  only when the author asks for multi-agent orchestration.
- **The machine's lock**: `/tmp/linlog-machine.lock` (`flock`), held by
  every timed run and every command the Nix daemon builds, since the
  daemon builds on any core.

## The machine

- Cores 0 and 1 are the desktop's. With two steps side by side: one
  step's timed runs on cores 2 and 3 (`bench/targets.sh` pins 2 and 3,
  `bench/net-targets.sh` 2), its builds on 4 to 7; the other's builds on
  8 to 11; mutation runs (cores 6 to 11) and fuzzing (12 to 15) only on
  your word. Everything in a memory-capped scope or unit
  (`systemd-run --user --scope -p MemoryMax=8G -p MemorySwapMax=0
  taskset -c …`). Kill by unit name or PID, never by pattern.
- The baseline and other night jobs: started detached as soon as they
  are ready (`bench/baseline.sh --detach --fresh`), no timer; the author
  says when the machine is not free. System timers inside a run
  (`nix-gc.timer`, `nix-optimise.timer`) need the author's `run0
  systemctl stop …` and start again after.
- "Stop" or "Pause" from the author: the `graceful-stop` skill
  (`HH:MM` for a stop by then, `resume` for the resume).

## What the author has asked for, standing

- **jj only, never git**, every commit signed as
  `flgrubm@grubmueller.dev`. Keep both keys warm with
  `plan/keys-warm.sh` (the unit `step-gpg-warm` uses the signing key
  and the SSH key every four minutes; `--check` says which is
  unlocked); the caches' maximum ends it, and then the author enters
  both passphrases (`! echo x | gpg --local-user
  flgrubm@grubmueller.dev --sign -o /dev/null; ssh -T
  git@github.com`) and you run the script again. Never touch
  `~/.gnupg`. Never sign or rewrite
  commits while a session works on them; when you must, tell it to run
  `jj workspace update-stale`.
- **Pushing** `main` without asking, as in step 7 above. Tags, releases,
  and every administrative `gh` task (settings, rulesets, environments,
  secrets, Pages, a workflow run) need the author's yes for that task;
  deleting and transferring are theirs alone.
- **Unattended**: when the author says so, decide and go on; ask only
  what is theirs to decide, collect those questions, and put them to the
  author in one message.
- **Design departures**: a session marks one "(departs from the
  design)" and goes on; you relay each to the author.
- **Prompts** say what is wanted and why and leave the design to the
  session; each names its model and effort; a step that changes a search
  or a criterion has step 26's panel.
- **Commands for the author** are nushell: `;` not `&&`, `(open --raw f)`
  not `$(cat f)`.
- **Testing** only as necessary; never weaken a test to make it pass.
- **Licences**: the repository is EUPL-1.2, the font under `cli/fonts/`
  OFL-1.1; no third-party file is committed or uploaded to the public
  cache (a check must not build the other provers or the problem
  libraries); the web front end's code EUPL-1.2, its content CC BY 4.0.
- **Quota**: the author runs on a Max 20x plan with no extra usage:
  Fable 5.1 where hardest reasoning decides in a small context, Opus
  5.5 at `high` otherwise, never `max`.

## Things that bit earlier sessions

- **Stacked chains conflict semantically**: an example names the engine
  that another chain's routing changed; a test's "this takes long"
  breaks when a prune makes it fast. Check the combined head.
- **A pasted goal can lose its Enter**: if the input line still holds
  `/goal [Pasted text …]`, send Enter again.
- **Signing lapses** after two hours however warm the loop; jj commands
  that write then fail. Sessions commit with `--config
  signing.behavior=drop` and you sign later (`jj sign -r
  'main@origin..REV & ~signed()'`).
- **The SSH key's cache lapses** separately from the signing key's,
  30 minutes after its last use (gpg-agent's `default-cache-ttl-ssh`),
  however warm the signing key is kept: on 2026-10-19 a loop that only
  signed let it lapse, the push failed, and a retry without SSH's
  `BatchMode` waited six hours on a passphrase prompt with the planning
  session stalled behind it. `plan/keys-warm.sh` uses both keys, and
  `plan/push-main.sh` never waits on a prompt.
- **`nix flake check` is unpinned**: never beside a timed run; the lock
  serialises them.
- **A report can say more than was done**: check that what it cites
  exists, and that a commit it names is the one at the head (sessions
  amend their report commit after their message).
- **Shell edits skip the formatter**: `nix fmt` on them before
  committing. `jj squash` into a described commit needs
  `--use-destination-message` or `-m`.
- **A volatile `/tmp`**: briefs and scratch data under `/tmp` vanish
  with a reboot; what a later session needs goes into the repository.

## When asked to re-evaluate models and efforts

Read the model docs and Claude Code's of the day (the page names are in
"Why these models and efforts"), argue from what the steps so far showed,
and add a dated paragraph there.
