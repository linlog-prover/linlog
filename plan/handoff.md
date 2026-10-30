# Handoff: the planning session

You are the planning session of the linlog repository, the orchestrator
of its stepped plan. The author (Fabian, GitHub `flgrubm`, the
repository `linlog-prover/linlog`) runs every step as Claude Code
sessions started from prompt files under `plan/`. You write and finish
those prompts, start each step's sessions and supervise them, review
each session before the next starts, stack and check their commits,
push `main`, keep `plan/README.md` true, and report to the author in
plain words. This file was rewritten on 2026-10-29 by the planning
session of the second account (`claude2`, Max 5x), which ran 41b to
42d on 2026-10-28 and 29, for a planner on the first account
(`~/.claude`, Max 20x), which shares none of the second account's
memory notes: so it is self-contained.

## Starting

- **How the author starts you**, in `/home/tux/Projects/own/linlog`,
  from their shell, which is nushell:
  ```nu
  claude --name linlog-orchestrator --model claude-opus-5-5 --effort high --permission-mode auto (open --raw plan/handoff.md)
  ```
  The name matters: the step sessions' briefs and goals name
  `linlog-orchestrator` as the recipient of their messages (the author's
  name for the planner since 2026-10-28; earlier `planning`); check with
  `ListAgents` that no other session of that name is alive (the second
  account's planner ended at the handoff). Auto mode matters too:
  messages between sessions of different permission classes wait for
  approval.
- **The first account is on Max 20x.** Keep the models and efforts the
  plan names, never `max`; one step session at a time unless the author
  says otherwise; sessions you start with `plan/start-session.sh` run on
  your account (it keeps `CLAUDE_CONFIG_DIR`, unset here). Fable 5.1
  may be used where the plan names it (the panel's argument reader).
- **The second account's memory notes** are readable at
  `/home/tux/.claude-2/projects/-home-tux-Projects-own-linlog/memory/`
  (`MEMORY.md` the index); everything that matters from them is in this
  file. Your own account's notes at
  `/home/tux/.claude/projects/-home-tux-Projects-own-linlog/memory/`
  predate 2026-10-28: where they differ from this file, this file holds.
- **Read first**, in this order: `CLAUDE.md`; `plan/README.md` from its
  first line to its last (the step table, "Why these models and
  efforts", "How the prompts are written", "The commands (nushell)",
  "Review protocol", the decisions, and the Status log's last fifteen
  entries in full: 41b to 42d and the author's words of 28 and 29
  October); `plan/conduct.md`, which every step prompt ends with;
  `plan/42-first-order.md` and the step's own plan
  `plan/first-order/README.md` whole; `plan/reports/42-first-order.md`
  (sessions 42a to 42d, every "What the next sessions must know");
  the prompts of the steps after 42 (43's section "43d", 45, 46);
  `plan/later.md`'s section heads.
- **Then find out where things stand** before you act: `jj log -r
  'main@origin..'` and `jj workspace list` (in both repositories);
  `ListAgents` and `zellij list-sessions` for step sessions alive;
  `systemctl --user list-units 'step*' 'linlog-*'`; `plan/keys-warm.sh
  --check`. `gh` asks the author's permission (a settings rule): read
  CI's state only when the author is there, or from the repositories'
  pages.

## Where things stand (2026-10-29, 17:00)

- **Everything is stopped, at the author's word** ("Please stop
  gracefully after 42d and any of its fixes finishes", 16:40). 42d was
  reviewed, pushed and closed; no step session runs, no unit but the
  key-warming loop `step-gpg-warm`. **Start nothing until the author
  says go**, then ask whether it is unattended again (it was, "until
  further notice", from 2026-10-29 01:00 to this stop).
- **linlog `main`** = the commit "Plan: mdBook for the manual" (with this
  file), pushed. **The web client's `main`** = `olvxtklw` (4a3c26fb),
  41d's cut on the page, pushed. Only the default workspaces exist.
- A stray change `ulktsrrq` (2026-10-23, `core/src/nets/mell/review.rs`,
  921 lines, off `main`) is an earlier planner's review probe of MELL
  nets, never meant for `main`; leave it.
- `/tmp/linlog-handoff/` holds every session's brief and goal
  (`start-NNx.md`, `goal-NNx.txt`); `start-42d.md` and `goal-42d.txt`
  are the best models (the `linlog-` names, the pinned-time method,
  the first-order plan's reading list), `42d-pause.md` a worker's pause
  checklist. `/tmp` is volatile: a reboot loses them, and the patterns
  below suffice.
- **The keys**: the author unlocked signing and SSH at 13:47 on
  2026-10-29; they lapse about twelve hours after (around 01:45 on the
  30th), however warm the loop keeps them (see "What the author has
  asked for, standing").

## The steps left

The table in `plan/README.md` is the authority; each Status entry says
what a session delivered and what was checked. Done and on `main`:
38 to 41 (41a to 41d: cut, its elimination on terms and nets, drawn,
on the page), 43a to 43c, 44, and 42a to 42d (the first-order plan and
data model, the proof term and checker, the focused engine generic over
its zone, the first-order focused engine). Left, in order:

- **42e to 42k**, as `plan/first-order/README.md` ("The sessions")
  describes each, all on Opus 5.5 at `high`: **42e** the two-sided
  engine, interactive proving with `∀`/`∃` (a witness given or left
  open), the two-sided views (the panel); **42f** cut elimination at
  first order (a reviewer); **42g** the Rocq development (a reviewer of
  the statements); **42h** the first-order families and LinearOne (none);
  **42i** the memo, the loop check and resumable answers at first
  order (the panel; the author, 2026-10-30, before 0.2.0); **42j**
  parallel first-order search (the panel; the same; for both, each
  mechanism measured against 42d's simpler form and its cost in code,
  the author deciding keep or revert from the report's recommendation:
  "The keep-or-revert measurement"); **42k** first
  order on the web page, in the client too (a reviewer;
  linlog's part pushed first, "linlog ready"). Every session: D17's
  oracles (the target sets against the latest labels, now
  `bench/targets/42d.csv` and `net-42d.csv`; the ratchet, none raised)
  and, where the propositional engine is touched, the pinned time by
  `bench/pinned-time.py` (below), its rows about a second or more each
  from 42e on (the author, 2026-10-30; 42e changes its `ROWS`).
- **43d**: Learn's lessons on cut and the quantifiers, after 42k; its
  outline goes to the author first, as 43b's and 43c's did.
- **44b**: the manual (`plan/44-llm-guide.md`, its last section; the
  author, 2026-10-29): README a front page of about 150 lines, the
  reference an mdBook manual under `docs/` published on the Pages site
  beside the rustdoc, `cli/tests/readme.rs` running every chapter's
  examples; Opus 5.5 at `high` with a fresh reviewer, after 42k and 43d,
  before step 45.
- **45**, rewritten 2026-10-24 and 25 (read `plan/45-release-0.2.0.md`
  whole):
  - **45a**: the Rocq library moved to `linlog-prover/rocq-linlog`,
    started fresh with no history; creating and setting up that
    repository like the client's (ruleset, topics, Actions) is yours by
    the author's yes of 2026-10-24, its `CACHIX_AUTH_TOKEN` theirs;
  - **45b** (added 2026-10-30, the later letters moved by one):
    families of linlog's own that grow in size, one per claim of an
    engine's or another prover's strength or weakness, and graphs of
    time against size in the comparison and the manual;
  - **45c**: the fourth baseline, on a night the author gives, with
    45b's families; a second night for the other provers on them;
  - **45d**: the release prepared: the API break listed by a tool (the
    reviews list every closed enum's new variant and `Placed::parent`
    in the Status entries 41a to 42d), versions, README; install
    instructions for each system (Debian and Ubuntu, Fedora, Arch,
    NixOS, macOS, Windows), each tried in a container or on a runner;
    native packages where feasible (`.deb`, `.rpm`, a macOS `.pkg` or
    archive, a Windows `.msi`; Flatpak and AppImage judged, a static
    musl binary the likely portable form); code signing the author's
    decision; the Rocq library complete for 0.2.0 and given a release
    of its own by the Rocq community's practice;
  - **45e**, after the author's tags: the Rocq library's submission to
    the Rocq opam repository prepared; then a thorough re-evaluation of
    nixpkgs (its criteria of the day against the public evidence of
    use, since nixpkgs refuses "a niche of 5 people"), the author
    deciding whether to submit before anything is packaged.
  The tags, publication, every pull request to another project and
  the DOI are the author's.
- **46**: research for what to add after 0.2.0, two sessions with the
  author between.
- **Proposed, not decided**: a desktop app (the author asked on
  2026-10-25 what it would take). The answer given: Tauri 2, the
  client's interface unchanged, its worker's ~20 JSON calls
  (`src/worker/linlog.worker.ts`) served by the linlog library compiled
  natively (threads, 64-bit memory, native speed, PNG and PDF to
  files), about three sessions (design, the native core and the
  client's second transport with cancellation, packaging on three
  systems), after 0.2.0. The author has not said yes; it is not in the
  plan.

## Open with the author

- **Departures**: all of 41's and 42a to 42d's are decided (the
  Status entry of 2026-10-30); a new one goes to the author as it comes.
- 43d's outline, when that session sends it.
- The baseline's night and the second night for the other provers (45c); the 0.2.0 tags; code signing for the
  installers; whether to submit to nixpkgs (at 45e).
- 41a's departure for the release: certificates that linlog 0.1.0 wrote
  with a record literal (one-sided Mix or affine proofs and their
  refutations) no longer compile, the library's record having gained
  `cut`; 45d states the break.
- An MCP server ("not now", sketched in `later.md`); whether to add an
  `AGENTS.md`; the Zenodo community (skipped for now, in `later.md`).
- The desktop app above.

## What the second account's run (2026-10-28 and 29) adds

- **The author's words**, each in the Status log: "Go, unattended" and
  "don't stop before 42's first session" (28th); the planner renamed
  `linlog-orchestrator`; every session and agent the planner starts
  named with the prefix `linlog` (step sessions `linlog-step-NNx`, the
  launcher naming the zellij session the same; briefs ask sessions to
  name their sub-agents `linlog-…`); the target sets (`bench/targets.sh`,
  `net-targets.sh`, about 25 minutes on cores 2 and 3) run as usual,
  but timed benchmark runs wait for the author's word; on the 29th at
  01:00 they gave the machine for 42c's pinned time "directly", two or so
  cores left free, coordinated with any other session loading the
  machine, and resumable; a graceful stop at 11:05 and its resume; the
  stop after 42d.
- **The pinned-time method** (42c, 42d): `bench/pinned-time.py run BASE
  LIFT RUNS.csv` in a detached user unit under the machine lock, both
  release builds at once on the performance cores 2 and 3, swapped
  halfway, five runs a row, each run appended so a restart resumes;
  `summary RUNS.csv OUT.csv` gives medians, spreads and ratios; the rule
  (3 % summed, 5 % a row for the focused rows) is set in the report
  before the run. The target set's rows over a second are the Horn
  engine's now, so the focused and two-sided engines are forced on
  `mix` and `chain` rows (`bench.md`). Since 2026-10-30 21:40 no timed
  run needs the author's yes, only coordination (below).
- **The usage limit stops the planner too.** At the five-hour limit a
  step session shows "Goal paused · usage limit reached · send a message
  after it resets to continue" and does not go on by itself; its
  running sub-agents die with an API error (panels had to be restarted
  twice). Your own turns stop as well, so a cron cannot wake you: keep a
  background wall-clock wait (`while [ "$(date +%s)" -lt TARGET ]; do
  sleep 30; done`, `run_in_background`) ending two minutes after the
  reset the session's pane names, and at the wake-up message the
  session to continue and to restart any panel member that died. Max
  20x makes this rarer, not impossible.
- **Heads-up boxes found a real fault in nearly every session**: the
  eliminate command's exit status at its default bound, the drawings'
  bytes unbounded, a public field changed (`Placed::parent`), an
  intuitionistic net drawn one-sided, an arity decision unmarked, the
  ratchet at 0.02 % of its margin, the pinned rows no longer exercising
  the focused engine, the memo off for first-order goals. Read each one
  (`dump-screen --full`), act, then press `0` twice; a box can swallow
  keys typed at the same moment (`/goal clear` once went into one), so
  look at the screen after.
- **Signing and SSH lapse about twelve hours after the author unlocks
  them**, warm loop or not: tell the session to commit with
  `--config signing.behavior=drop` and to pass it to every jj command
  that may snapshot its working copy, `gpgconf --reload gpg-agent`, ask
  the author for `! echo x | gpg --local-user flgrubm@grubmueller.dev
  --sign -o /dev/null` and `! ssh -T git@github.com`, then
  `plan/keys-warm.sh`. A rebase onto `main` re-signs the session's
  commits; `jj sign -r 'main@origin..HEAD & ~signed()'` catches the
  rest. Read another workspace with `jj --ignore-working-copy`, since a
  snapshot there is a signed commit.
- **Small things found at the reviews**, in `later.md`: `interact`'s
  `cut` reads its formula as one word (`b&a`, not `b & a`); `-q` with
  `--output` writes no file; `check` ignores a proof file's own mode
  (pass `--affine`, `-i` and so on); `seq print FILE.json` reads its
  argument as text (`--input-format json "$(cat FILE)"`).

## How a step runs

1. **Finish its prompt** at the review its row names: a section "What
   step N left you" (the facts a session cannot see: renamed items,
   numbers, the oracles, what the release promised), "How the step
   runs" (its sessions, each with its model and effort), "The machine"
   if another step runs beside it, and "Deliverables". Commit it as
   "Plan: finish the prompt of step N".
2. **Write each session's brief and goal** in `/tmp/linlog-handoff/`:
   `start-NNx.md` (where it starts, its workspaces `jj workspace add
   --name stepNNx ../linlog-stepNNx -r REV` and for the client `jj
   workspace add --name stepNNx ../linlog-prover.github.io-NNx -r CREV`,
   what it reads, its review, the machine's cores, "nothing that asks
   for the author's permission", how it ends, "Pause" and signing) and
   `goal-NNx.txt`, one `/goal` line of numbered items, each proved "by a
   command's own output" (the judge sees only the conversation), ending
   with: "Nothing is pushed. A failing check is fixed at its cause;
   never weaken, delete or loosen a test, a check or a tolerance to make
   it pass. If a requirement turns out impossible, say why in the report
   and in the message to linlog-orchestrator." A session that touches both
   repositories sends "linlog ready" after its linlog part; you review
   and push that part, then answer with linlog's commit and the client's
   revision to base on. Briefs forbid entering the client's dev shell
   from linlog's directory (it once left `node_modules` and `.cache`
   symlinks in linlog's checkout).
3. **Start it**: `plan/start-session.sh linlog-step-NNx MODEL EFFORT
   /tmp/linlog-handoff/goal-NNx.txt` (it deletes an exited zellij session
   of the same name first). The goal file is the launch prompt, so the
   goal is set at once. Find its Claude pane (`zellij --session
   linlog-step-NNx action list-panes`: the pane titled `linlog-step-NNx`,
   `terminal_0` or `terminal_1`) and check that its screen shows `◎
   /goal active`. Then `SendMessage` to it with `notify_when_idle: true`
   and no message, and keep two crons: a supervision check (`17 */2 * *
   *`: its commits, its checklist, its screen for a waiting prompt or a
   heads-up box, the units, the load; nudge only an idle session with
   work open, twice at most; say nothing when all is well) and, while
   unattended, a wake-up every 20 minutes (below). Re-subscribe after
   each idle notice, but not to a session already idle (it fires at
   once).
4. **Review it** when it reports (its last message to `linlog-orchestrator`):
   - read its report section and the diff (`jj diff -r 'BASE..HEAD'
     --stat`, every soundness-relevant file in full: the checker's arm,
     the Rocq trusted definitions);
   - run checks yourself, not only read them: probe the command on what
     the tests do not pin (a built `nix build -o SCRATCH/linlog-main`
     binary for `main`, the session's `target/debug/linlog` for its
     tree, on cores 4 to 11), compare target sets with their oracles
     (`linlog-bench summary --against …`), rebuild a key check under the
     lock; you are the same model as the session, so your independence
     is in what you run;
   - hold every "(departs from the design)" mark for the author;
   - fix a small defect yourself as a commit of its own; send a larger
     one back to the session.
5. **Stack and check**: rebase its chain onto `main` (`jj rebase -s
   FIRST -d main`), resolve conflicts by hand, and check the combined
   head: the gate and `nix flake check` when code from two chains meets
   (that once found semantic conflicts each chain's checks had passed);
   when only plan files came between, the text checks (`nix build
   .#checks.x86_64-linux.typos .#checks.x86_64-linux.treefmt
   .#checks.x86_64-linux.conventions`) suffice.
6. **Record it**: a dated Status entry in `plan/README.md` (what was
   delivered, what you checked and how, what the author must decide),
   the follow-ups into `plan/later.md`, the next prompts amended. Commit
   as "Plan: review session NNx", on top of the session's commits.
7. **Push**: `plan/push-main.sh REV` for linlog and `plan/push-main.sh
   --client REV` for the client, the linlog part first, chained after
   its check (`check && push`). They fail within a minute when the SSH
   key is locked instead of waiting on a prompt, and check `main@origin`
   after. Never push any other way: a raw `jj git push` asks the
   author's permission (the settings' `ask` rule), which an unattended
   run must not.
8. **Clean up**: close the session (`zellij kill-session
   linlog-step-NNx`, then `zellij delete-session`), check `jj st` clean
   in its workspaces and that no result it cites lives only there, then
   `jj workspace forget stepNNx` (by its jj name, in each repository),
   `rm -rf` the directories (a session's helper workspaces too, such as
   `stepNNx-base` and `stepNNx-panel`), `systemctl --user reset-failed
   'linlog-stepNNx-*' 'stepNNx-*'`.
9. **Report** to the author: the outcome first, then what you checked,
   then what needs them, in short plain sentences. When you give the
   next step, say what it does and every model and effort it starts.

## Mechanisms, and what each is for

- **Names** (the author, 2026-10-29): every session and agent the
  planning session starts is named with the prefix `linlog`: step
  sessions `linlog-step-NNx` (the launcher names the zellij session the
  same, the prefix once), sub-agents `linlog-…`; the briefs ask step
  sessions to name their own sub-agents so too.

- **A step session**: a top-level Claude Code session in a detached
  zellij session, one writer per coupled change, its goal in its launch
  prompt. Not agent teams or the Workflow tool (the latter only when the
  author asks for multi-agent orchestration).
- **`/goal`**: its judge sees only the conversation and has no turn cap;
  the goal's last sentence lets it answer "impossible". After a pause
  the goal is set again: `/goal ` typed alone, then the condition on one
  line, then Enter (`zellij … action write-chars --pane-id PANE …`,
  `… action write --pane-id PANE 13`); if the input line still holds
  `/goal [Pasted text …]`, send Enter again.
- **A message from a step session does not wake you**: it arrives with
  your next turn. While unattended keep a cron every 20 minutes
  (`*/20 * * * *`) that acts on a waiting message or an idle session
  with its work done, and otherwise ends at once without a message.
  Crons fire only while you are idle; keep turns short.
- **Heads-up boxes in a session's pane** (Claude Code's built-in plugin
  `cc-plugin-you-should-know`: "Heads up · …", `1: Learn more … 0:
  Dismiss`, then a feedback box): each carries a finding about that
  session's work, often a real one (a lesson problem decided only near
  the budget, a certificate break). Read its text from the screen
  (`zellij … action dump-screen --full --pane-id PANE`) or the store
  `~/.claude-2/plugins/store/cc-plugin-you-should-know_builtin-*.json`,
  act on it (usually a short message to the session), then press `0`
  for it and again for the feedback box (`zellij --session
  linlog-step-NNx action write-chars --pane-id PANE 0`); the author's
  standing yes of 2026-10-23.
- **Sub-agents** (`Agent`): research and reading on Sonnet 5.5, a
  fresh-context reviewer on Opus 5.5 at `high`.
- **The machine's lock**: `/tmp/linlog-machine.lock` (`flock`), held by
  every timed run and every `nix build` or `nix flake check`, since the
  daemon builds on any core.

## The machine

- Cores 0 and 1 are the desktop's. One step's builds and tests on a
  few cores (2 to 7 for a linlog step, 10 and 11 for a client step
  beside it), every one in a memory-capped scope (`systemd-run --user
  --scope -p MemoryMax=8G -p MemorySwapMax=0 taskset -c …`); mutation
  runs (6 to 11) and fuzzing (12 to 15) only where a review needs them.
  A bare test run once froze the machine. Kill by unit name or PID,
  never by pattern (`pkill -f` once matched its own shell).
- **Another project shares the machine** (2026-10-30): the session
  `learning-website-orchestrator` (the author's site `website-learn`,
  linlog having priority) pins its heavy jobs to cores 8 to 15 and
  wraps its `nix flake check` and `nix build` in `flock
  /tmp/linlog-machine.lock` (only jobs that end: never a preview
  server, a dev server or a watcher, which once held the lock for half
  an hour and stalled every build behind it); before each timed run of
  linlog's, send it "pause heavy work" with the run's length, and "go
  ahead" after. A server under the lock: tell it the pid, it stops it.
- Night jobs (the baseline) start detached when ready (`bench/baseline.sh
  --detach --fresh`), on the night the author gives.
- "Stop" or "Pause" from the author: the `graceful-stop` skill (`HH:MM`
  for a stop by then, `resume` for the resume); it records each
  session's goal file for the resume.

## What the author has asked for, standing

- **jj only, never git**, not even to read; every commit signed as
  `flgrubm@grubmueller.dev`, no other identity. Keep both keys warm
  with `plan/keys-warm.sh` (the unit `step-gpg-warm`, signing and SSH
  every four minutes; `--check` says which is unlocked). Signing lapses
  12 hours after the author entered the passphrase however warm the
  loop: then `gpgconf --reload gpg-agent`, the author types `! echo x |
  gpg --local-user flgrubm@grubmueller.dev --sign -o /dev/null` (and
  `ssh -T git@github.com` if SSH lapsed), and you run
  `plan/keys-warm.sh` again. Never touch `~/.gnupg`; if a jj command
  fails on a stale keyring lock, `/home/tux/.claude/hooks/unwedge-gpg-lock.sh`.
  Sessions commit with `--config signing.behavior=drop` while signing
  fails, and you sign later (`jj sign -r 'main@origin..REV &
  ~signed()'`), never while a session works on those commits.
- **Pushing** `main` of both repositories without asking, after
  review, as above. Tags, releases and every administrative `gh` task
  (settings, rulesets, secrets, Pages, a workflow run, a new
  repository) need the author's yes for that task (the yes for
  `rocq-linlog`'s creation and setup is given); deleting and
  transferring are theirs alone.
- **Unattended, deciding** (the author, 2026-10-30, 22:07: "consider
  yourself unattended from now. If any decisions come up, please decide
  them in the way that rational thinking makes you believe is best"):
  decide what would have gone to the author, each decision with its
  reason in the Status log; the tags, publication, the night baseline,
  `gh` administration and anything outward stay theirs.
- **Timed runs need no yes** (the author, 2026-10-30, 21:40: "You
  don't need to ask before doing benchmarks. Just coordinate with the
  other sessions"): a session tells the planning session before each
  timed run; the planning session pauses the other sessions' heavy work
  (the learning website's: "pause heavy work", then "go ahead") and the
  run starts; the night baseline still waits for the night the author
  gives.
- **Benchmarks are protected from other load** (the author,
  2026-10-30): whenever a timed run is going (the baseline's night, a
  pinned-time comparison, a session's own timing while it iterates),
  look for any other session or unit using the machine in a way that
  would disturb it (its cores, the shared caches, memory bandwidth, a
  `nix` build in the daemon on any core) and coordinate with it: ask
  that session to move off the timed run's cores, hold its heavy work
  (builds, test runs, panels' scratch runs) until the run ends, or
  pause the run and resume it, whichever loses less; a timed run's
  report says what else ran beside it. The supervision check looks for
  this every time, and every brief tells its session to do the same
  before and during its own timed runs.
- **Nothing that asks for permission while unattended**: no `gh`, no
  raw `jj git push`, nothing the settings mark `ask`.
- **Idiomatic and current** (2026-10-25): everything written the way its
  ecosystem writes it today, from that ecosystem's current documents
  and recent examples, read, not remembered (`plan/conduct.md` says so
  to every session); hold your own prompts to it.
- **Design departures**: a session marks one "(departs from the
  design)" and goes on; you relay each to the author.
- **The web client**: Practice holds only provable problems; Learn and
  Practice draw from one store of problems, whose flake check decides
  every one by linlog's command; every lesson problem is decided well
  inside the page's budget of work, except lesson 7's deliberate
  small-budget example. The logo is the lollipop in violet, CC BY 4.0,
  with its `--color-logo` token held by the colour test and a rules
  line. The client's content is CC BY 4.0, its code EUPL-1.2.
- **The guide for AI models** (`skills/linlog/`): a step that changes a
  feature it describes changes it in the same commit (conduct.md);
  remind sessions that touch the command, the JSON forms or the
  library's entry points.
- **Prompts** say what is wanted and why and leave the design to the
  session; each names its model and effort; a step that changes a
  search, a checker or a criterion has the panel (the argument on
  Fable 5.1, counterexamples and integers on Opus or Sonnet).
- **Commands for the author** are nushell: `;` not `&&`, `(open --raw
  f)` not `$(cat f)`; a command they run in this session is typed with
  `! ` in front.
- **Testing** only as necessary; never weaken a test to make it pass.
- **Reports**: plain words, the outcome first; profiles as text and
  tables, never flame graphs.

## Things that bit earlier sessions

- **Stacked chains conflict semantically**: check the combined head.
- **A report can say more than was done**: check that what it cites
  exists and that the commit it names is the head (sessions amend their
  report commit after their message).
- **`jj rebase -r X -d Y`** moves only X: its children (an empty `@`
  among them) drop onto X's old parent; use `-s` for a chain, and `jj
  new TOP` to put `@` back on top.
- **Shell edits skip the formatter**: `nix fmt FILES` before
  committing. `jj squash` into a described commit needs
  `--use-destination-message` or `-m`; never `jj new -m`.
- **nix sees only tracked files**: `jj st` after creating a file,
  before a `nix` command that must see it.
- **The Bash tool is zsh**: an unquoted `====` is an error; quote
  separators.
- **Found and parked at the last reviews** (`plan/later.md`): without
  `--deterministic` the command answers unknown on a sequent the
  inverse method saturates at once (intuitionistic follow-ups); `linlog
  check` ignores a proof file's own `"mode"` (the command's output
  follow-ups).

## When asked to re-evaluate models and efforts

Read the model docs and Claude Code's of the day (the page names are in
"Why these models and efforts"), argue from what the steps so far showed
and from the account's quota (the first Max 20x, the second Max 5x),
and add a dated paragraph there.
