# Step 44: a guide to linlog for AI models

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing. The step takes one
session; this prompt was written on 2026-10-23 at the author's word. It
says what is fixed.

## Goal

A compact summary of everything linlog does (the command first, then
the library, the web page and its bindings, and the Rocq library where
an agent would use it), written for language models of any vendor by
the practices current on the day the step runs, so that a model given
it uses linlog's features correctly with little else: which command or
call answers which question, the input syntax, the JSON forms, the exit
statuses, the limits and their flags, what a verdict and a refutation
mean, and short examples with their real output.

## Fixed now (the author, 2026-10-23)

- **Research first.** A research sub-agent (Sonnet 5.5 at `medium`)
  reads, in their sources and never from memory, what the current
  practice is for documentation that AI models read: the `llms.txt`
  proposal and its `llms-full.txt` (llmstxt.org) and who follows it,
  Agent Skills (a `SKILL.md` an agent loads to use a tool) and their
  open specification, `AGENTS.md` (for agents working on a repository,
  which linlog's `CLAUDE.md` already is for Claude), the model vendors'
  guidance on describing tools and command-line programs to models,
  and whether an MCP server is what lets a model use a program's
  features most easily. The report has the findings with links, and the
  forms the session writes follow them.
- **Accurate by construction.** Everything the guide states comes from
  what defines it (the command's `--help`, which is the doc comments of
  `cli/src/argument_parsing.rs`; the rustdoc; the JSON forms' docs;
  README's usage), and every example in it is run by a test, as
  `cli/tests/readme.rs` runs README's, so that a changed flag or output
  fails the gate rather than leaving the guide wrong; a test also fails
  on a command or flag the guide names that does not exist.
- **Compact**: plain Markdown, the short summary within a budget the
  research names (a few thousand tokens), the full reference beside it;
  no marketing, no history, nothing a model cannot act on.
- **Where it lives**: in the repository (served with the crate where
  that is how a package carries it), on the rustdoc site that `docs.yml`
  publishes at `/linlog/`, and at the web page's root
  (`linlog-prover.github.io/llms.txt`, describing the page and pointing
  to the rest), the last a short change in the client's repository after
  linlog's push; README and the page's About link to it.
- **Tried on a model that has nothing else**: a fresh-context agent on a
  small model (Haiku 5.5, then Sonnet 5.5, at `medium`), given only the
  guide and the command, performs a set of tasks (prove a sequent and
  read the verdict, check a proof file, export LaTeX and SVG, step
  through a proof non-interactively, decide a batch, read a refutation,
  decide an ordinary-logic sequent), each judged by the command's own
  output; what it gets wrong is fixed in the guide, and the report
  gives the before and after.
- **An MCP server is the author's to decide**: if the research says it is
  what lets models use linlog most easily, the session sketches it (a
  `linlog` subcommand serving the command's actions as tools, its
  limits and its stop) in `plan/later.md` with the reasons, for the
  author; it is not built here.
- **Kept true from now on**: the step adds to `plan/conduct.md` and to
  CLAUDE.md that a step changing a feature the guide describes changes
  the guide in the same commit, as it does README, and the tests above
  hold it to that.

## How the step runs

One session on Opus 5.5 at `high`, its goal in its launch prompt, after
step 40 (the two-sided surface reshapes what the guide describes); it
touches only documentation and its tests, so it may run beside step
41's sessions, whose new features it then leaves to them to add under
the rule above. Its client part comes after linlog's push.

Deliverable: thematic jj commits in both repositories;
`plan/reports/44-llm-guide.md`.
