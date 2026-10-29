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

## Session 44b: the manual (the author, 2026-10-29)

The author asked whether to split README into GitHub's wiki; the
planning session advised against the wiki and for a manual, and the
author agreed. README has grown to about 2 500 lines, 1 350 of them
usage: too long for a front page (crates.io shows it as the crate's
page) and too short a place for a reference. The wiki was ruled out
because it is a repository of its own outside the flake, CI and review,
so `cli/tests/readme.rs` could not run its examples and they would
drift; it is one set of pages for every release; and GitHub's documents
say search engines do not index wikis, pointing to Pages instead.

- **README becomes a front page** of about 150 lines: what linlog is,
  install, a first proof, where to go next (the manual, the web page,
  the guide for AI models, the API documentation), citing, licence.
- **A manual in mdBook** (the author's choice, 2026-10-29, over
  Astro's Starlight, Material for MkDocs and its successor Zensical,
  Docusaurus, Sphinx and rustdoc chapters: the form the Rust ecosystem
  uses, one binary from nixpkgs and no new toolchain in linlog's flake,
  plain Markdown beside the code that the README test reads), adopted
  through the `new-tool` skill at its current version with its
  documents read, its theme in the page's colours (the violet accent,
  the logo in the header, both themes) and links both ways between the
  manual and the page, under `docs/`, a chapter per area: the commands and
  their output formats, interactive proving, cut and its elimination,
  proof nets, first-order logic, ordinary logic, the batch, the
  benchmarks and the comparison, what exists and what is planned, the
  architecture, development. README's sections move there whole, not
  rewritten; links inside the repository and to the page are kept and
  checked.
- **Every example still run**: `cli/tests/readme.rs` reads every
  chapter's console blocks as it reads README's and the guide's today,
  and fails on a name that does not exist, so the guarantee that every
  shown output is the command's own holds for the manual.
- **Published beside the API documentation**: the Docs workflow and the
  flake's `doc` package build the manual into the same GitHub Pages
  site as the rustdoc of `main`, its address in README, the crate's
  `documentation` key if fitting, the guide for AI models and the
  client's About; how a released version's manual is kept apart from
  `main`'s follows the practice the session reads (a version selector,
  a directory per release, or the release's own tag), said in the
  report.
- **The rules follow**: `plan/conduct.md`'s paragraph on README (a step
  extends the usage section) and CLAUDE.md's verification table name
  the manual where a step adds a command, flag or format; `cli.md`,
  `ci.md` and `flake.md` say where the manual lives and how it is
  built and checked.

One session on Opus 5.5 at `high`, documentation and its tests only,
with a fresh-context reviewer on Opus 5.5 at `high` (every section of
the old README present in the manual or the front page, every example
run, every link resolving). It runs after 42i and 43d, which still add
to README, and before step 45, whose 45c then prepares the front page
and the manual for 0.2.0.
