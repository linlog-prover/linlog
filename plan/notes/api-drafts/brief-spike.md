# Brief: the quantifier spike of step 28's design

linlog (read `/home/tux/Projects/own/linlog/CLAUDE.md` first) is a
linear-logic proof-search suite in Rust. Step 28's design (stage 2) must say
what first-order logic (step 38) will change in the data model and the
engines, and decision D17 of `plan/README.md` binds it: **the propositional
case must not pay for quantifiers** (the target set's counters identical,
pinned CPU time within a few percent, by generics or by a duplicated fast
path). The spike builds the proposed change far enough to measure that
cost, in a jj workspace of its own, and is **thrown away** afterwards:
nothing of it is merged. What it leaves is a measurement and a short report.

## The specification

The design draft that specifies the spike is
`/tmp/claude-1000/-home-tux-Projects-own-linlog/4419c622-34f5-4db8-a2bc-0b8996d2cdd4/scratchpad/design/draft-c.md`.
Read its sections 3.2 (`Term`, `Kind`, `Atom`), 3.4 (`Member`), 3.5 (the
`QUANTIFIERS` bit), 3.6 (`Node`), 3.9 (the reserved first-order tables),
10.5 (step 38 in full: the trail, the zone, the seam) and 11 (the spike:
its milestones M1 to M3 and gates G1 to G3) in full. Build what section 11
says, with these changes:

- **M1 (required)**: as 11.3 says. The JSON forms, the text syntax and
  every output stay byte-identical (no reader produces the new variants),
  so `cargo test -p linlog` and the behaviour lock (`core/tests/lock.rs`,
  `cli/tests/lock.rs`) must pass unchanged; a test that fails is a bug of
  the spike.
- **M2 (required if M1 is done)**: as 11.3 says: `Context` and `Classes`
  lifted to `search/zone/`, the focused engine generic over `Z: Zone` with
  `Ground` the only instance the front door calls. Keep `Ground`'s methods
  `#[inline]` forwards to today's code. Same tests.
- **M3 (optional)**: only if M1 and M2 are measured and you judge it worth
  its time; otherwise say in the report that it was not run and why.
- Skip G3's interleaved timing: the machine runs other agents now. Record
  instead the target set's own `cpu_ms`/`time_ms` per row as indicative,
  with the load average (`uptime`) at the start and the end of each run.

Where draft C's text leaves a detail open, take the simplest choice that
keeps the propositional path as it is, and note it in the report.

## Where you work

- The workspace: `/home/tux/Projects/own/linlog-spike` (a jj workspace
  named `spike` of the repository, at the design session's tree). Edit only
  files in it. Never touch `/home/tux/Projects/own/linlog` (the main
  checkout).
- **jj**: only in the spike workspace, and only these two commands, each
  with the signing option (signing hangs now otherwise):
  `jj --config signing.behavior=drop commit -m "Spike M1: …"` at the end of
  a milestone (so its diff is kept), and `jj --config
  signing.behavior=drop diff --stat`. Never `git`, never `jj new -m`, no
  other jj command.
- **The base measurements are taken** (same workspace directory, the tree
  unmodified, the devshell's build): the journeys' instruction counts in
  `/tmp/claude-1000/-home-tux-Projects-own-linlog/4419c622-34f5-4db8-a2bc-0b8996d2cdd4/scratchpad/spike/base-journeys.txt`,
  the target set in `/home/tux/Projects/own/linlog-spike/bench/targets/spike-base.csv`
  (identical in its counters to the committed `after-coverability.csv`).
  The script `…/scratchpad/spike/cmp.py BASE.csv NEW.csv` compares two
  target CSVs (counter differences per decided row, CPU over rows above
  one second).

## How you build, test and measure (binding)

- Builds and tests: in a memory-capped scope on **cores 12 to 15** only,
  with the devshell of the main checkout and the workspace's own `target/`:
  `cd /home/tux/Projects/own/linlog-spike && systemd-run --user --scope -q -p MemoryMax=8G -p MemorySwapMax=0 taskset -c 12-15 env CARGO_BUILD_JOBS=4 RUST_TEST_THREADS=4 nix develop /home/tux/Projects/own/linlog -c cargo test -p linlog`
  (and `cargo test -p linlog-cli --test lock`, `cargo clippy -p linlog`).
  No `cargo test --workspace` (it runs the harness and the readme tests on
  the shared machine; the two locks are what you need), no `nix flake
  check`, no `cargo hack`.
- The journeys (G2), after a milestone builds and passes its tests, as a
  detached unit so it survives:
  `systemd-run --user --unit=step28-spike-journeys-m1 --working-directory=/home/tux/Projects/own/linlog-spike -p MemoryMax=8G -p MemorySwapMax=0 --setenv=CARGO_BUILD_JOBS=4 /run/current-system/sw/bin/bash -c 'taskset -c 12-15 nix develop /home/tux/Projects/own/linlog -c bash -c "cargo build --release --locked -p linlog-bench && ./target/release/linlog-bench ratchet --jobs 4" > /tmp/claude-1000/-home-tux-Projects-own-linlog/4419c622-34f5-4db8-a2bc-0b8996d2cdd4/scratchpad/spike/m1-journeys.txt 2>&1'`
  then wait on it with a loop in a background task
  (`until ! systemctl --user is-active --quiet step28-spike-journeys-m1; do sleep 20; done`),
  never one blocking call of more than a few minutes.
- The target set (G1), after the journeys: `bench/targets.sh` pins cores 2
  and 3 itself and starts the unit `linlog-targets`:
  `cd /home/tux/Projects/own/linlog-spike && systemd-run --user --scope -q -p MemoryMax=8G -p MemorySwapMax=0 taskset -c 12-15 env CARGO_BUILD_JOBS=4 nix develop /home/tux/Projects/own/linlog -c bench/targets.sh spike-m1`
  (about 8 minutes), wait for `linlog-targets` to end the same way, then
  compare with `cmp.py` against `spike-base.csv`. Only one `linlog-targets`
  unit can run at a time; check `systemctl --user is-active linlog-targets`
  first.
- Nothing you run is unbounded; no thread or job count exceeds four; no
  other benchmark, probe or run than these; nothing outward-facing. A run
  this brief does not name, ask for in your final message instead.

## The report

Write `/tmp/claude-1000/-home-tux-Projects-own-linlog/4419c622-34f5-4db8-a2bc-0b8996d2cdd4/scratchpad/spike/spike-report.md`
(about 5 to 12 KB): per milestone, what was built (the diff's stat, the
choices you made where the draft was open), the tests' result, G1 (rows
compared, counter differences: must be none), G2 (a table: journey, base,
milestone, change in percent; the gate's verdict), the indicative times
with the load, and what the result decides under 11.5's table. Then what
the spike taught that the design should say (a cost found, a seam that did
not fit, a variant whose arm landed on a hot path). Keep every number as
measured; say what was not run.

Your final message: in under 200 words, M1's and M2's verdicts with their
largest journey change, and the report's path.
