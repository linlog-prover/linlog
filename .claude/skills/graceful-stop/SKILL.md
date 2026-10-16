---
name: graceful-stop
description: >-
  Pause all running work so that it resumes later with as little loss as
  possible: this session's own runs, every session it supervises, their
  sub-agents, workflows and systemd units. Use when the author says
  "stop" (or "pause", or that they need the machine now), or when a
  supervising session sends "Pause". With the argument `resume`, picks
  the paused work up again, which is what the author's "go" or a
  supervisor's "Resume" asks for.
argument-hint: "[resume]"
---

# Graceful stop

The machine is shared with the author's other work, and they may need it
at any moment. "Stop" means: free the machine now, keep everything that
was done, and leave each piece of work where it can be picked up.
Nothing is abandoned and nothing is deleted.

`$ARGUMENTS` is empty for a stop and `resume` for a resume.

## Rules for every stop

- **A short task in progress may finish** if that saves work: a commit,
  a batch with a minute left, a test run nearly done. Anything longer
  stops at its nearest resumable point, and its cut part reruns later.
- **Record before you stop**: what each run has done, what it has not,
  and the exact command that resumes it. A stop that leaves no record
  loses the work as surely as a kill.
- **Stop by unit name or PID**, never `pkill -f` or `killall`: a
  pattern matches the asking shell too.
- **Keep the gpg warm loop** (a unit named `*-gpg-warm`): it takes no
  CPU, and a resume needs signing.
- **Delete nothing**: scratch directories, workspaces, partial outputs
  and logs stay.

## As a supervisor (you started other sessions)

1. **Take stock**: the sessions you run (`ListAgents`, `zellij
   list-sessions`), your sub-agents and workflows, your background
   tasks and monitors, your crons, and the units:
   `systemctl --user list-units 'step*' 'linlog-*' --no-pager`.
2. **Clear each worker's goal first.** A session started with a
   `/goal` has a fresh model check the goal after every turn and start
   the next turn while it is unmet, which undoes a pause; a goal cannot
   be paused, only cleared and set again (code.claude.com/docs/en/goal,
   checked 2026-10-16). Keys typed into a busy session are dropped, and
   an idle one is restarted by the goal within moments, so: interrupt
   it with Escape (`zellij --session linlog-NAME action write 27`; it
   cuts only the model's current step, its units run on), type
   `zellij --session linlog-NAME action write-chars "/goal clear"`,
   then `… action write 13` for Enter, and look at its screen (`zellij
   --session linlog-NAME action dump-screen --pane-id PANE`): it must
   say `Goal cleared`; if not, repeat once. Note which file held the
   goal's text (the one given to `plan/start-session.sh`) for the
   resume.
3. **Pause each worker**: `SendMessage` "Pause" with `notify_when_idle`.
   A worker whose brief does not mention this skill gets "Pause: run
   the graceful-stop skill as a worker".
4. **Pause your own work**: tell your sub-agents to stop at a resumable
   point, then `TaskStop` them; stop the background tasks that run
   work, and your units by name. Delete your crons (`CronDelete`): a
   supervision check would nudge a paused worker back to work. Note
   each cron's schedule and prompt so the resume recreates it.
5. **Wait for the idle notices**, about fifteen minutes at most (at
   once if the author says "stop now"), then check the units and the
   load (`uptime`). Stop a worker's leftover unit by its name and tell
   the worker which, for its resume.
6. **Tell the author**, shortly: the machine is free (the load, no
   unit left but the warm loop), what was paused and where, what each
   piece loses (the batch it was in), and that "go" resumes it.

## As a worker (you got "Pause", or the author said "stop" to you)

1. **Finish or cut** the current task, by the rule above.
2. **Stop your sub-agents and workflows** at a resumable point (tell
   them, then `TaskStop`), and **your units** by name. A run that skips
   its finished parts on a rerun (a mutation batch whose list exists, a
   baseline that resumes) needs only a note of which part was cut.
3. **Commit what is finished** (`jj st` first). If a test signature
   fails, commit with `--config signing.behavior=drop`; the supervisor
   signs later. Unfinished edits stay in `@`: say in the checklist what
   they are.
4. **Write the checklist**: each run's state, the command that resumes
   it, what comes next.
5. **Report and end the turn**: one line to the supervisor with
   `SendMessage` (the machine is free, the checklist is written), or to
   the author directly if there is no supervisor. No background task
   waits on anything while paused.
6. **Stay paused**: a turn started by a goal's check before the
   supervisor clears the goal does no work and ends at once. If you
   have a goal and no supervisor, ask the author to type `/goal clear`.

## Resume (`resume`, "go", "Resume")

- **A supervisor**: test signing with a signature (`echo x | gpg
  --batch --pinentry-mode error --local-user flgrubm@grubmueller.dev
  --sign -o /dev/null`); if it fails, ask the author for the passphrase
  once and restart the warm loop. Restart your own units, recreate your
  crons, and send each paused worker "Resume" with `notify_when_idle`.
  For a worker whose goal you cleared, set it again while it is idle:
  `zellij --session linlog-NAME action write-chars "$(cat GOAL_FILE)"`,
  then `… action write 13`; its screen shows `◎ /goal active`. The
  goal's turn count and timer start afresh; the work is kept.
- **A worker**: read your checklist, rerun the commands it records, and
  carry on from there.
