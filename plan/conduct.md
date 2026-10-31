## How to work on this step

This is one step of the plan in `plan/README.md`: a linear-logic proof-search
suite the author uses for research and teaching, built step by step by
separate sessions. Later steps build on what you leave behind, so the report
you write and the rules under `.claude/rules/` matter as much as the code.

The numbered items above are requirements, not an order of work: choose the
order and the design, and read the spec's sections named above before
deciding rather than working from memory. When you have enough information
to act, act; do not re-litigate decisions the plan records, and where you
weigh a choice, pick one and say why in the report rather than surveying the
alternatives.

Whatever you write, write it idiomatically and by current best practice
(the author, 2026-10-25): Rust as the language and its ecosystem write
it today, and anything in another ecosystem's form (a Nix module or
package, a nixpkgs or opam submission, a Rocq library, a GitHub
workflow, an npm package, a web page) as that ecosystem's own current
documentation and recently maintained examples do, read at the version
you build against rather than recalled. Say in the report what you read
where a choice follows it.

Don't add features, refactor, or introduce abstractions beyond what the step
requires. Don't design for hypothetical future requirements beyond what the
plan names. Avoid premature abstraction, and avoid half-finished
implementations too. Don't add error handling or validation for scenarios
that cannot happen; trust internal invariants and validate at the system
boundaries (parsing, deserialization, CLI input). If, while working or
testing, you find a pre-existing bug, a performance concern, or behaviour the
step doesn't mention, don't fix, optimize or extend it in this change unless
the requested behaviour cannot work without it; report it as a follow-up.
Where the step is ambiguous, implement the reading its wording, the spec and
the surrounding code most directly support, state that assumption in the
report, and don't build for the other readings as well.

When you design a library-level feature whose output people see, design
its configuration for every front end at once (plan decision D15): one
plain-data options value with defaults and serde, which the CLI maps its
flags onto, the web front end holds as JSON and any other wrapper reuses;
no choice a user might want to vary hidden in a constant. Say in the
report which options exist and how each front end would set them.

Whatever you build gets a sensible default and a way to change it (D16):
a call without flags does what a newcomer expects and what is safe,
within a bounded time and memory, and every default is a named constant
of an options value with a flag and a line of documentation. Where you
settle a type or an interface, keep the place for quantifiers and say
in the report where they go (D17); until the first release the API is
free to change for the better (D18).

Test as necessary, not as much as possible. Commit tests only where the step
names a behaviour to pin or the repository already keeps tests for this kind
of change, sized like the neighbouring test files, roughly one focused test
per stated behaviour; scratch checks and exploratory scripts stay out of the
repository. Verify your work however you like as you go (run the check
commands while you build, not only at the end), and before reporting
progress audit each claim against a tool result from this session: only
report work you can point to evidence for, say explicitly what is not yet
verified, and if a check fails say so with its output. Prefer targeted edits
to whole-file rewrites where the result is the same. Never weaken, delete or loosen a test, a check or
a tolerance to make it pass: a failing test is reported with its output,
and changed only where the behaviour it pins was meant to change, which
the commit says.

The machine is shared with its owner's other work. Every scratch program,
and every build and test (`cargo test`, `cargo clippy`, `cargo hack`, a
call of the command), yours or a sub-agent's, runs in a memory-capped
scope of its own on a few cores (`systemd-run --user --scope -p
MemoryMax=8G -p MemorySwapMax=0 taskset -c 4-9 …`, with
`CARGO_BUILD_JOBS=6` and `RUST_TEST_THREADS=4`) with its enumerations
bounded by size, because an unbounded checker once took 62 GB and the
kernel killed the whole terminal with the session in it, and a bare
`cargo test` whose new test built without a bound froze the machine
for ten minutes until its owner switched it off; say so in every
sub-agent's brief. Anything that runs for more than a few
minutes runs detached from the terminal (a systemd user unit) so that it
survives the session. Do not use every core or run `bench/baseline.sh`
unless the step says the machine is free. A benchmark or probe run by
day that the step does not name is asked for first, however small and
on whichever cores, with its purpose, its duration and the cores it
takes, and waits for the author's yes; one step ran three hours of
probes on the efficiency cores to choose a list and had to be stopped.

Delegate independent work to sub-agents and keep working while they run:
the `crate-source-explorer` agent for any question about a dependency's API
at the pinned version, and a fresh-context reviewer for a soundness-critical
piece (a checker, a criterion, a prune) before you call it done. Intervene
if a sub-agent goes off track. A sub-agent's brief carries every rule of
the paragraph above, not the memory cap alone: which cores it may use and
for how long, that no thread or job count it hands to any program
exceeds them, that nothing it runs is unbounded, and that a run its
brief does not name is asked for through you. One sub-agent, told only
about memory, probed the command with `--jobs 100000` and loaded every
core for five minutes.

Pick each sub-agent's model for its job: Sonnet 5.5 at `medium` for
research and reading, Haiku 5.5 or no agent at all for a mechanical
extraction, Opus 5.5 at `high` for a reviewer, as `plan/README.md` ("Why
these models and efforts") lists them. Brief a reviewer with the
criteria it judges against and tell it to report only what breaks
correctness or a stated requirement: a reviewer asked for gaps reports
some even when the work is sound. Answer each finding in the report, and
change the work only for those that hold. When the checks your goal
names pass, stop: no further rounds of review, probing or polishing that
the step does not ask for.

Where soundness rests on a number (a counter, a length, an index, a
bound), say at its declaration why it cannot reach its limit, or make
reaching it a refusal, and test the refusal with an input that gets
there: no arithmetic of a checker or a criterion may wrap in any build.
A rewritten checker once kept its zones as counters, passed a
differential test and a reviewer's 27 000 random terms, and accepted a
proof file of 131 nodes for an unprovable sequent, because 64 doublings
wrapped a counter that no random term had come near.

Record what a future session must know and cannot see in the code in
the rules file of the module it is about (`.claude/rules/core-*.md` for
the core crate, whose index `core.md` lists them; `cli.md`, `bench.md`,
`flake.md`, `ci.md`): invariants, why a choice was made, what a check
cannot catch, one point per bullet; and in your step report. Update an
existing note rather than adding a duplicate; delete what turns out to be
wrong. Code comments and doc comments never mention this session, the
prompt, the plan, its steps or its decision numbers: they say what the
code does or why, in terms a reader of the repository alone understands.
The plan and the reports are where the organisation of the work lives.

`README.md` is the public face of the repository, a front page of about
150 lines, and the manual under `docs/src/` (an mdBook, a chapter per
area, published beside the API documentation) is its reference; both
must describe what exists after your step: extend the manual's chapter
with the commands, flags and formats you added (real invocations with
their output; a new area gets a chapter of its own in `SUMMARY.md`), and
keep its "What exists and what is planned" chapter (`status.md`) true,
moving what you built from planned to built; README changes only where
the front page's few lines stop being true. A new benchmark run's
results go in a chapter of their own under `docs/src/results/`, its
figures drawn by `bench/plots.py` from the rows and listed in
`docs/figures.sh`. Do not mention the plan, its steps or these sessions
in README or the manual (the `conventions` check fails on it).

The guide for AI models, `skills/linlog/` (its summary `SKILL.md`, an
Agent Skill served as `llms.txt`, and `reference.md`), describes the same
features for models: a step that changes a command, a flag, a format, a
JSON form, an exit status, a verdict's wording or a library entry point
the guide describes changes the guide in the same commit, as it does
README, and adds a feature it builds there in a sentence and, where
there is output, an example. `cli/tests/readme.rs` runs every example of
README, the manual and the guide and fails on a command, flag, session command or settings key
it names that does not exist; the summary keeps within its budget
(fewer than 100 lines, 8 000 bytes), so a new feature goes into
`reference.md` and into the summary only where a model needs it to
start.

Everything you produce in one reply, including any reasoning or drafting
before the reply, counts toward a single limit of about 128 000 tokens. If
that limit is reached before the reply is finished, the whole turn is lost
and the work has to start over. Composing an entire module in full as
reasoning and then again as a reply would double the length of the turn
without improving the result, so do not do that: spend the reasoning on
understanding the requirements, checking the code your design depends on,
and settling the structure and the hard decisions, then write the code
into files and let the compiler and the tests take it from there.

In the report and in your final message, lead with the outcome, then the
decisions, then what is left open. Complete sentences, no working
shorthand, and every file, type or flag you name gets its own clause saying
what it is.
