# Step 44: a guide to linlog for AI models

## Outcome

linlog has a guide for AI models, written as an Agent Skill:
`skills/linlog/SKILL.md` is the compact summary (front matter for the
skill's loader, then a body in the shape of an `llms.txt` file) and
`skills/linlog/reference.md` beside it the full reference. The rustdoc
site serves the summary's body as `/linlog/llms.txt` and the reference
as `/linlog/llms-full.txt` and `/linlog/reference.md`; README's
documentation list and the crate's front page link to them, and the web
page's root serves an `llms.txt` of its own that describes the page and
points to them. `cli/tests/readme.rs` runs every example of the guide,
as it runs README's, and fails on a command, flag, session command or
settings key the guide names that does not exist; it also holds every
default and number the guide states to the command's help, the guide's
Rust example to a doctest of the library, and the summary to its
budget. A small and a medium model, given only the guide and the
command, did the brief's seven tasks; the before and after are below.
The research found that an MCP server is not what lets a model use
linlog most easily now; `plan/later.md` answers the question for the
author with a sketch for when it is revisited. `plan/conduct.md` and
CLAUDE.md now say that a step changing a feature the guide describes
changes the guide in the same commit.

## The research

A research sub-agent (Sonnet 5.5 at `medium`) read the sources below on
2026-10-24 with web search and fetch, in 33 tool calls, and marked what
a page states ([S]) apart from what it inferred ([I]); this section keeps
the distinction, and says where it could not read a source. Nothing
here is from memory.

**llms.txt.**

- [S] The proposal (<https://llmstxt.org/>, v2, last modified
  2026-08-10): an H1 with the project's name, the only required part;
  then a blockquote summary; then detail sections of Markdown "of any
  type except headings"; then H2 sections of links, `- [name](url):
  notes`, one named "Optional" for what an agent may skip. Served at
  `/llms.txt` of a site "or at any subpath", the most specific file
  covering the URLs under it. Each page should have a Markdown twin at
  the same URL with `.md` added.
- [S] `llms-full.txt` is not in the proposal's text the agent read;
  Mintlify's blog says Anthropic asked Mintlify for both files
  (<https://www.mintlify.com/blog/the-value-of-llms-txt-hype-or-real>).
  [I] It is a convention of documentation vendors: the whole
  documentation inlined in one file.
- [S] Adopters, fetched live: Anthropic
  (`platform.claude.com/docs/llms.txt`, about 700 links, about 20 000
  tokens, pointing to an `llms-full.txt`), Stripe
  (`docs.stripe.com/llms.txt`, about 96 KB, pages linked as `.md`, and a
  Skills page), Cloudflare (an index of per-product `llms.txt` files),
  agentskills.io, modelcontextprotocol.io and code.claude.com.
- [S] Whether crawlers read it: an Ahrefs study of 137 000 domains (May
  2026) found 97 % of `llms.txt` files never requested
  (<https://ahrefs.com/de/blog/llmstxt-study/>), naming GPTBot first and
  Claude Code second among the fetchers that did; Google's John Mueller
  said Google does not use it (a secondary write-up,
  <https://www.searchenginejournal.com/googles-mueller-says-llms-txt-cant-help-llms-differentiate-sites/566027/>).
  [I] It helps an agent already pointed at a site, not discovery.

**Agent Skills.**

- [S] The specification (<https://agentskills.io/specification>): a
  directory with `SKILL.md`, whose YAML front matter has `name` (1 to 64
  characters, lowercase letters, digits and hyphens, the directory's
  name) and `description` (1 to 1 024 characters: what it does and when
  to use it), optionally `license`, `compatibility` (at most 500
  characters), `metadata` and `allowed-tools`; optional `scripts/`,
  `references/` and `assets/`. Progressive disclosure: about 100 tokens
  of metadata are loaded for every skill at start, the body only when
  the skill is used ("< 5000 tokens recommended", "under 500 lines"),
  references one level deep, on demand.
- [S] Anthropic's best practices
  (<https://platform.claude.com/docs/en/agents-and-tools/agent-skills/best-practices>):
  the same 500 lines; "only add context Claude doesn't already have";
  the description in the third person; a table of contents for a
  reference over 100 lines; one default rather than many options;
  input and output examples; evaluations, tested with Haiku, Sonnet and
  Opus.
- [S] Claude Code (<https://code.claude.com/docs/en/skills>) installs
  skills under `~/.claude/skills/NAME/` or a project's
  `.claude/skills/NAME/`, cuts the listed description at 1 536
  characters, and refuses unknown front matter outside Claude Code.
  [S] Anthropic made Agent Skills an open standard on 2025-12-18
  (<https://www.anthropic.com/engineering/equipping-agents-for-the-real-world-with-agent-skills>);
  [I] which other vendors load the format was not verified from a
  primary source.
- [I] A skill fits a command a third party installs: Stripe ships one
  for its command.

**AGENTS.md.**

- [S] <https://agents.md/>: "a README for agents" working on a
  repository (build and test commands, style, review rules), plain
  Markdown, nested files allowed, now stewarded under the Linux
  Foundation, read by Codex, Jules, Gemini CLI, Cursor, Copilot's agent,
  Aider, Zed, Devin, Amp, Windsurf and others.
- [S] Claude Code reads `AGENTS.md` when there is no `CLAUDE.md`, and
  only `CLAUDE.md` when there are both, which may import the other with
  `@AGENTS.md` (<https://code.claude.com/docs/en/memory>); its target for
  `CLAUDE.md` is under 200 lines.
- [I] It is for agents that work on linlog, not for agents that use it.

**The vendors' guidance on describing tools.**

- [S] Anthropic, "Writing tools for agents"
  (<https://www.anthropic.com/engineering/writing-tools-for-agents>):
  describe a tool as to a new hire, with its formats and terms
  explicit; few tools for the workflows that matter; outputs within a
  budget (Claude Code caps a tool's response at 25 000 tokens), with a
  concise and a detailed form and, where output is cut, how to narrow
  it; errors that say what to do, with a correct input; evaluate on
  realistic tasks and read the transcripts.
- [S] OpenAI's function calling guide
  (<https://developers.openai.com/api/docs/guides/function-calling>):
  clear names and parameters, what the output means, examples and edge
  cases, enums for closed sets, the "intern test" (could a person use it
  from what the model sees?), fewer than 20 functions at once.
- [S] Google's Gemini guide
  (<https://ai.google.dev/gemini-api/docs/function-calling>): 10 to 20
  tools at most, specific descriptions, strong types and enums; the
  agent's summary of it was thin.

**MCP.**

- [S] The specification's current revision is 2026-07-28
  (<https://modelcontextprotocol.io/specification/latest/server/tools>):
  stdio is newline-delimited messages over a subprocess's standard
  streams; a tool has `name`, `title`, `description`, an `inputSchema`
  and an optional `outputSchema`, its result `content` and optional
  `structuredContent`; `notifications/cancelled` and
  `notifications/progress` cancel and report. [I] The annotations'
  defaults (`readOnlyHint` and the others) come from earlier revisions;
  the 2026 text was not read.
- [S] Anthropic's post on code execution with MCP
  (<https://www.anthropic.com/engineering/code-execution-with-mcp>):
  most clients load every tool definition up front; exposing tools as
  files an agent discovers cut one example from 150 000 to 2 000 tokens.
  Claude Code now defers MCP tools by tool search
  (<https://code.claude.com/docs/en/mcp>).
- [S] Practitioners, vendor-written: the same task cost 1 365 tokens by
  a command line and 44 026 through MCP
  (<https://scalekit.com/blog/mcp-vs-cli-use>); with lazy loading the
  gap has largely closed (<https://www.checklyhq.com/blog/mcp-vs-cli-token-efficiency/>);
  MCP for remote services and many users, a command and a skill for a
  local tool (<https://stacklok.com/blog/mcp-vs-cli-tools-why-security-changes-the-answer/>).

**Budgets and Rust.**

- [S] No source sets a budget for `llms.txt` or `AGENTS.md`. A skill's
  body: under 500 lines and 5 000 tokens; its description 1 024
  characters; `CLAUDE.md` under 200 lines.
- [S] A crate's package holds the files under its directory
  (<https://doc.rust-lang.org/cargo/reference/manifest.html>);
  `#[doc = include_str!(…)]` puts a Markdown file into rustdoc
  (<https://doc.rust-lang.org/rustdoc/write-documentation/the-doc-attribute.html>).
  [S] The agent found no convention of docs.rs or rustdoc for
  `llms.txt` and no well-known crate that ships one; generators exist
  (`rustdoc-llms`, `crates_llms_txt`, `aidoc-core`).

## The forms, and why

- **An Agent Skill as the guide's home** (`skills/linlog/`): the
  research's first recommendation, since models use linlog as a tool.
  `SKILL.md` is the summary, `reference.md` the reference one level
  below it with a table of contents, as the specification and
  Anthropic's practices ask. The directory is named `skills/` at the
  repository's root, where tools that install skills from a repository
  look, and is not part of this repository's own Claude Code setup
  (`.claude/skills/`), which CLAUDE.md says.
- **The summary is also an `llms.txt`**: its body after the front
  matter is in the proposal's shape (an H1, a blockquote, detail
  paragraphs with bold leads instead of headings, then `## Docs` and
  `## Optional` link lists), so that the same text is the skill and the
  site's `llms.txt`; the doc derivation strips the front matter. One
  text, not two copies that could drift.
- **The budget**: the research names a body of about 1 000 to 2 000
  tokens and under 100 lines for the summary, its own choice within the
  sources' ceilings (a skill's body under 500 lines and 5 000 tokens,
  CLAUDE.md under 200 lines, a skill file of about 800 tokens reported
  as effective). The test holds the body to fewer than 100 lines and
  8 000 bytes; at about four bytes a token of English (this session's
  estimate, no tokenizer was run) that is about 2 000 tokens. The body
  is 93 lines and about 7 500 bytes.
- **Where it is served**: in the repository (the skill, linked from
  README), on the rustdoc site that `docs.yml` publishes
  (`/linlog/llms.txt`, `/linlog/llms-full.txt`, `/linlog/reference.md`,
  so that the summary's relative link to `reference.md` resolves there
  as in the skill), and at the page's root (the client's `llms.txt`).
  `llms-full.txt` follows the vendors' convention, which agents look for.
- **Not in the crates' packages** (departs from the research's
  recommendation, not from the design): the plan asks for the guide "with
  the crate where that is how a package carries it", and the research
  found no such convention for Rust; a package carries only files under
  its own directory, and the guide describes the command and the library
  together. Docs.rs is reached through the crate's front page, which now
  links to the site's `llms.txt`.
- **No AGENTS.md**: the research says it is for agents working on a
  repository, which CLAUDE.md already is for Claude, and the step is
  about agents that use linlog. A thin `AGENTS.md` that `CLAUDE.md`
  imports, or the reverse, would serve other vendors' coding agents;
  that is the author's to decide, and left open here.
- **No MCP server**: see below and `plan/later.md`.

## Accurate by construction

Every statement of the guide was written from what defines it: the
`--help` of every command (the doc comments of
`cli/src/argument_parsing.rs`, read from the built command), the
session's `help`, README's usage section, and the rustdoc of `Sequent`,
`Proof`, `Outcome`, `Reason`, `Refutation`, `Error` and the crate's
front page for the JSON forms and the library. Every example's output
was taken from the command, not written by hand. The tests in
`cli/tests/readme.rs`:

- `guide_examples` runs every `console` block of `SKILL.md` and
  `reference.md` as `readme_examples` runs README's (the harness is now
  one function over a document; `printf` joined `cat` and `echo` as a
  shell command, since a script piped into `interact` is how an agent
  steps through a proof);
- `guide_names_what_exists` fails on a `linlog` subcommand that does
  not exist, a flag after `linlog COMMAND` that the command's `--help`
  lacks, any other flag no help names, a session command `help` does
  not list, and a settings key no help names; made-up names (`--copy`,
  `seq show`, `redo`, `--hurry`, `search.speed`) added to the reference
  each failed it, and the reference restored passed;
- `guide_s_rust_is_a_doctest` holds the reference's Rust example to the
  crate-level doctest of `core/src/lib.rs` it copies, which the
  library's tests compile and run;
- `the_summary_keeps_its_budget` holds the front matter to the Agent
  Skills limits and the body to its budget;
- `guide_states_the_defaults`, which the supervisor asked for after the
  trials: every default and number the guide states is held to the
  command's `--help` (each flag's own section lists its default) or to
  the library's constant: the reference's table of limits row by row,
  every span `--flag VALUE` in a sentence that says "default", the
  batch's order of exit statuses against `prove --help`'s sentence, and
  the library's copy bound against `Options::default().copies`. The exit
  statuses are shown by examples that print them (`; echo "exit $?"`),
  which `guide_examples` runs: 0, 1, 2 and 3 for `prove`, 1 and 3 for
  `check`, 1 for a session that ends without a proof. The numbers that
  fit none of these were rewritten into one or left out: the minimum
  Rust version, the exit status 130 (which no `--help` states), the
  crate's version, NanoYalla's version (`prove --help` names it), and
  Typst's height for curryst. Wrong values put into the guide (a
  recursion limit of 4096, a pool after 200ms, a library copy bound of
  4) each failed the test, and the guide restored passed.

## The trials

Each trial was a fresh agent (`general-purpose`, at `medium`) given a
prompt naming only a directory with a copy of the guide, the brief's
inputs and a wrapper of the release command that held every call to
cores 8 and 9, 2 GiB and 60 seconds; it was told to read nothing else,
use no web tool, run every other command under `timeout 60` and never
more than two threads. No personal data was in any prompt. The seven
tasks (the brief's): prove `A -o B, B -o C |- A -o C` and read the
verdict; check a given proof file (an intuitionistic proof, so that the
mode matters); export the derivation of `!A |- !A * !A` as LaTeX and
SVG; prove `A * B |- B * A` step by step through a pipe, every rule
chosen, and save the proof; decide a given batch of five named
sequents (provable, unprovable for want of weakening, contraction under
`!`, a classical refutation, a parse error) and explain the exit
status; explain why `A * B |- A par B` is unprovable and give the
refutation's `kind`; decide Peirce's law in intuitionistic and in
classical logic. Every answer was judged by the command's own output:
the expected verdicts from the same calls, the saved proofs by `linlog
check` (`valid proof of ⊢ ~A ⅋ ~B, B ⊗ A (classical)`, exit 0, for all
four), the exported files by their content.

| run | guide | tasks right | commands that failed | what it found missing |
|---|---|---|---|---|
| Haiku 5.5, before | as first committed (`nwnzuysk`) | 7 of 7 | 2 sessions of task 4 (positions of a goal it misread, a goal number the session had retired) | that a step retires its goal and opens new numbers; which goal a split's positions name; which literal `ax` takes; that `check` takes its mode from its flags |
| Haiku 5.5, after | with those four fixed (`ztmlkxyz` but its last three sentences, which this run asked for) | 7 of 7 | 1 command (a stray position after `par`) | that no rule but `*` takes more positions; the session's exit 1; the batch's line format; the file's mode |
| Sonnet 5.5, before | with Haiku's fixes (`ztmlkxyz`) | 7 of 7 | 1 call (it fed `check` the file `save` wrote) | that `save` writes the session and `proof FILE` the proof; the rule names `apply` takes |
| Sonnet 5.5, after | with those fixed (this step's last guide but its clause on the file's `mode`, which this run asked for) | 7 of 7 | none but its probes of `goals` and `rules` | how to tell a proof file's logic (the file's `mode`), which the guide now says |

What changed in the guide between them: the summary says that the
session starts with goal 0 and a step opens new numbers, read from the
output; that every position of `apply` is one of the goal before the
step, and only `*` takes more; that `ax` takes either literal; the rule
names; that `proof FILE` writes what `check` reads and `save` the
session; the session's exit status 1; the batch's lines (`NAME:`
optional, `#` comments); and that `check` takes its logic from its flags,
the file's `mode` telling which. The reference says the same in its
section on the session. Every model got every verdict right from the
first guide on; what the guide taught worse was the session's
numbering, which cost the small model two failed runs and the medium one
a wrong file, and none after the fixes.

Each fix went through the names test first: the first fix put a rule's
name and an output line among the spans of the summary's paragraph on
the session, which the test read as session commands that do not
exist, so they moved to a paragraph of their own; a later sentence named
`check` there, so the test now lets a span name a command of `linlog`
(a name that exists), and still fails on any other.

## The page's root

In the client (`linlog-prover.github.io`, after linlog's push), one
commit, `rsxtpzrz`, written on the client's `main` `munpltsm` and
rebased without a conflict onto `qmlqvzzn`, where the supervisor moved
`main` with step 43b's lessons: `src/pages/llms.txt.ts`
serves `/llms.txt` from the written text `src/content/llms.txt`, in the
proposal's shape (an H1, a blockquote that says the page is a prover in
the browser for people with no API to call, a paragraph that sends a
model to the command and the library, then `## Docs` with linlog's
`/linlog/llms.txt`, `/linlog/llms-full.txt` and the library's API, and
`## Optional` with the repository and the privacy page), under CC BY
4.0 as the page's written content. About's text links it. It points to
the guide rather than copying it, so that the guide stays one text that
linlog's tests keep true. The client's input of linlog stays where it
was, since the file only links. A browser test beside About's
(`tests/learn.spec.ts`) holds About's link, the file's media type, the
proposal's shape and the two guide links; the client's CLAUDE.md and
`.claude/rules/web.md` name the file. The six tests of the spec passed,
and the client's `nix flake check` passed at the commit on both bases
(`all checks passed!`, exit 0).

## The MCP question

The research says an MCP server is not what lets a model use linlog
most easily now: linlog is a local command without state, whose text,
JSON and exit statuses an agent with a shell reads directly, and a
skill costs about a hundred tokens until it is used, where most MCP
clients pay for every tool's schema on every turn (Anthropic's post on
code execution with MCP; vendor-written measurements). A server would
add typed schemas, a read-only hint, cancellation and progress, and use
from clients without a shell. So nothing was built, and
`plan/later.md` ("For the author: an MCP server for linlog") gives the
author the answer, the reasons, when to revisit it, and a sketch for
then: a subcommand `linlog mcp` over stdio with a handful of tools
named after the commands, the session's steps without state as the web
bindings have them, the command's defaults as every call's limits,
`notifications/cancelled` as the Ctrl-C flag and progress from the work
count.

## Kept true from now on

`plan/conduct.md` (after the paragraph on README) and CLAUDE.md (the
crate list and a row of the verification table) say that a step that
changes a command, a flag, a format, a JSON form, an exit status, a
verdict's wording or a library entry point the guide describes changes
the guide in the same commit, as it does README; the tests above hold
it there, and `.claude/rules/cli.md` describes them for the session
that next changes them. Step 41, which runs beside this step and adds
cut, brings cut into the guide under this rule.

## Departures and assumptions

- **The guide is not in the crates' packages** (departs from the
  research's recommendation, not from the design): the plan asks for the
  guide with the crate only "where that is how a package carries it", and
  the research found no convention for Rust crates; the rustdoc site
  serves it and the crate's front page links there.
- **No `AGENTS.md`**: the research places it with agents working on a
  repository, which is not this step's audience; whether to add a thin
  one beside CLAUDE.md for other vendors' coding agents is left to the
  author.
- **The page's `llms.txt` points to the guide rather than holding it**:
  the plan says it describes the page and points to the rest; the guide
  itself stays one text, on the rustdoc site of the same origin.
- **The budget is a byte count**: no tokenizer was run (none was at hand
  without sending the text to a service); 8 000 bytes at about four bytes
  a token of English is this session's estimate of the research's
  2 000 tokens, and Unicode symbols, three bytes each, make it an
  overestimate.
- **`printf` joined the shell commands the README test runs**, so that a
  session's script on a pipe, how an agent steps through a proof, is an
  example the test runs.

## What is left open

- A value of a flag (`--engine NAME`, `--format NAME`) in the guide's
  prose is not checked by name; only the examples that run it check it.
- The web bindings are described by their calls' names only, and the
  Rocq compile commands are not run by the guide's test (the flake's
  `rocq` and `rocq-linlog` checks compile the certificates the command
  writes).
- Agents read the guide when they are pointed at it; the research found
  little evidence that crawlers fetch `llms.txt` on their own.

## Checklist

- [x] Research by a sub-agent (Sonnet 5.5, `medium`), findings with
  sources above, and the forms written following them.
- [x] The summary (`skills/linlog/SKILL.md`, 93 lines, about 7 500 bytes
  of body) within the budget the research names, and the full reference
  (`skills/linlog/reference.md`) beside it, every statement from the
  help, README or the rustdoc.
- [x] Served in the repository and on the rustdoc site (`nix build
  .#doc`: `llms.txt`, `llms-full.txt`, `reference.md` beside the crates'
  docs); linked from README and the crate's front page.
- [x] At the page's root (`/llms.txt`, the client's `rsxtpzrz`) and
  linked from About, after linlog's push.
- [x] `guide_examples` runs every example of the guide;
  `guide_names_what_exists` fails on a command, flag, session command or
  settings key that does not exist (shown with made-up names);
  `guide_s_rust_is_a_doctest`, `the_summary_keeps_its_budget`, and
  `guide_states_the_defaults` for every default and number it states.
- [x] Trials: Haiku 5.5, then Sonnet 5.5, at `medium`, given only the
  guide and the command, judged by the command's output, the guide fixed
  where they failed, before and after above.
- [x] The MCP question answered for the author in `plan/later.md`
  (research: no server now; a sketch for later; nothing built).
- [x] The rule in `plan/conduct.md` and CLAUDE.md.
- [x] Every linlog commit gated (`gate: passed` at the guide's first
  version with the rule and the MCP answer in the tree, at each trial's
  fixes, and at the test of the defaults, whose tree the report's
  commits do not change for any check), and `nix flake check` passed at
  the last commit (exit 0).
- [x] The client: `llms.txt`, About's link, a browser test, its `nix
  flake check` passed at its last commit (exit 0).
- [x] `jj st` clean in both workspaces; nothing pushed by this session.

## Session 44b: the manual

### Outcome

README is a front page of 149 lines (what linlog is, install, a first
proof and a refutation, where to go next: the manual, the web page, the
guide for AI models, the API documentation and the changelog; citing;
licence), and the reference is an mdBook manual under `docs/`, built by
the flake into the Pages site beside the rustdoc at
`linlog-prover.github.io/linlog/manual/`. README's sections moved into
the chapters whole: every line of the old README is in the front page
or a chapter, apart from headings re-levelled or renamed as chapter
titles and cross-references made links (the reviewer checked it line
by line). The chapters: the introduction (README's opening, first proof
and documentation list), Installation; under "Using linlog" The
commands (README's usage of `prove` and `check`) with Output formats,
Rocq certificates, The syntax, Cyclic linear logic and The Lambek
calculus beneath it, Interactive proving, Cut and its elimination,
Proof nets, First-order logic, Ordinary logic, Many sequents in one
call; under "Results" Benchmarks and the comparison (README's harness
section and "Beside the other provers") with three results chapters,
The baseline of 2026-10-17, Beside the other provers 2026-10-15 and
First order against LinearOne 2026-10-30; under "The project" What
exists and what is planned, Architecture and Development. The theme is
the page's: its tokens on mdBook's light and navy themes (the other
three hidden from the menu, as the page has two schemes), the violet
accent, the word mark of the page in the menu bar and the square mark
as the tab's icon, each in both schemes.

`cli/tests/readme.rs` runs every `console` block of README and of every
chapter `SUMMARY.md` lists (174 command lines, 165 of them run; the
rest are the harness, pdfLaTeX and Rocq, as before, and one block
marked `skip`), fails on a Markdown file under `docs/src` that
`SUMMARY.md` does not list, and holds README and the chapters to the
names the guide is held to; `bench/tests/manual.rs` checks the
harness's names in the benchmark chapters. Made-up names planted in
chapters (`linlog prove --copy`, `linlog seq show`, the key
`search.speed`, `linlog-bench run --familly`, `linlog-bench rerun`)
each failed a test, and the chapters restored passed.

Every figure is drawn by `bench/plots.py` from the CSV rows when the
manual is built (`docs/figures.sh`, run by the flake's `manual`
package), in the light and the dark scheme, and none is committed. The
check `manual-links` resolves every link of the manual and README into
the manual, the repository and the site offline (721 links, 0 errors);
a planted broken fragment, missing repository file, missing page of the
site and missing anchor each failed it.

### Decisions, and what was read

- **mdBook 0.5.4**, the version the locked nixpkgs pins, adopted through
  the `new-tool` skill: its guide at that version (the renderer options,
  the theme, the general configuration, continuous integration) and its
  changelog from 0.5.0 (unknown configuration fields are errors,
  `smart-punctuation` on by default, hashed asset names, `--dest-dir`
  relative to the working directory). `smart-punctuation` is off, since
  the text is README's, written for GitHub's rendering;
  `create-missing = false`, so a listed chapter that is missing fails
  the build; `site-url = "/linlog/manual/"` for the 404 page.
- **The link checker is lychee** (nixpkgs' 0.24.2), not
  `mdbook-linkcheck2`, which mdBook's guide recommends since 0.5.4:
  lychee checks the built HTML, fragments included, and its `--remap`
  turns the repository's GitHub URLs into files of the tree and the
  site's URLs into files of the `doc` package, so links that leave the
  book are checked too, offline. It needs `cacert` in the sandbox even
  offline.
- **The figures are drawn at build time, not committed**, so that no
  figure can drift from its rows or be drawn by hand; `svg.hashsalt` and
  no date make the SVGs a function of the rows. `bench/plots.py` gained
  subcommands (`comparison`, as before; `families`; `growth`) and
  `--theme`; the palette is the `dataviz` skill's categorical slots,
  validated by its script against the manual's two surfaces (light: every
  check passes, three slots below 3:1 contrast, relieved by a marker per
  series and the tables beside each figure; dark: every check passes).
  The figures are static images, so the skill's hover layer does not
  apply; each figure's table view is the chapter's table.
- **The figures**: the baseline's `families.svg` (time against size of
  every family under the engine the command chooses, one panel per
  family, a line per engine) and `engines.svg` (the focused against the
  net engine on the families `engines.csv` forces both on); the
  comparison's two cactus plots and twelve scatter plots; the first
  order's `growth.svg` (time against size of each shape, a line per
  tool), its scatter plot and its cactus plot. Each caption says what
  is measured, on which machine, at which commit and date (the first
  order's commit is not recorded by its run, and the chapter says so,
  naming the commit that holds the rows), and what it shows; every
  "what it shows" was computed from the rows, and the reviewer computed
  them again.
- **A line per engine** (departs from the design): `families.csv` holds
  one engine per family, the one the command chooses, so its figure
  colours each family's line by that engine, and a second figure from
  the same baseline's `engines.csv` draws the focused and the net engine
  side by side where the baseline forces both.
- **A released version's manual is the `docs/` of its tag**, as the
  release's API is docs.rs: the practice read was mdBook's own guide
  (no version selector; it describes deploying at every push or at a
  tag) and the Docs workflow's `deploy-pages`, which replaces the site
  whole at every deployment, so a directory per release would mean
  building every tag's manual at every push. The introduction and README
  say so. Nothing in GitHub's Pages settings changes: the site's source
  is already "GitHub Actions", and the manual appears under `manual/` at
  the next deployment of `main`.
- **The logo stays out of the repository**: the menu bar's word mark
  and the tab's icon are the page's files by URL, as README's licence
  section says of the logo (CC BY 4.0, served by the page). The icon is
  set by a few lines of script, since mdBook writes its own favicon link
  unless the theme carries the file (departs from the design only in
  the means).
- **Links both ways with the page**: the introduction, README and the
  guide link the page; the page's About links the manual (the client's
  part).
- **The names of the harness's chapters** are the harness's: the CLI's
  test checks only the calls of `linlog` in `benchmarks.md` and
  `results/` (`HARNESS_CHAPTERS`), and `bench/tests/manual.rs` checks
  the rest against `linlog-bench`'s help and the scripts' text, since
  the command's tests cannot run the harness.
- **A settings key in README or the manual passes when the settings
  file can hold it** (`settings_know`), where the guide's must be in a
  flag's help: README already named `ordinary.logic`, which the file
  holds and no flag sets, and the help that would name it is pinned by
  the behaviour lock.
- **README's test asserts more than 150 commands**, README's and the
  manual's together, where it asserted more than 40 of README's alone
  (the commit says so).
- **The crate's `documentation` key** is set on `linlog-cli` to the
  manual (Cargo's reference: a URL of the package's documentation,
  docs.rs when unset); the library keeps docs.rs. No `homepage`, which
  Cargo's reference reserves for a site other than the repository and
  the documentation.
- **The `doc` package is the rustdoc and the manual put together** after
  both are built, so a change of the manual does not rebuild the
  rustdoc. The `conventions` check now also fails on a line of the
  manual that names the plan.
- **The plan names `bench/COMPARISON.md` for the comparison with the
  other provers**; that comparison is `bench/TOOLS.md` (COMPARISON.md
  compares the baselines), which the provers chapter follows.

### The reviewer

A fresh-context reviewer on Opus 5.5 checked the six requirements
(every old section present, the front page, every example run, every
link resolving, every figure from the rows with true captions, room for
later runs) and ran both tests. Its findings and the answers:

- README's licence section named "Beside the other provers", no longer
  in README, without a link: holds; it links the manual's section now.
- The baseline's figure caption said the partitions rise ever more
  steeply, but the four 3-Partition families are flat: holds; the
  caption names `partition-yes` and `partition-no` and says the
  3-Partition families stay flat.
- The engines figure's caption said the net engine leaves
  `3-partition-mll-no` undecided past its smallest size, but it decides
  sizes 4 and 5: holds; it says past size 5.
- Headings renamed as chapter titles (`Usage` as "The commands",
  `First-order sequents` as "First-order logic", `Benchmarks` as
  "Benchmarks and the comparison"): by design, a chapter per area as
  the plan lists them; the text under them is unchanged.
- Links to the page outside `/linlog/` (the logo, the web page) are not
  checked by `manual-links`: true, offline they cannot be; the check's
  comment and the rules say so. The reviewer could not run lychee; this
  session did, with the planted breaks above.
- The comparison's committed light SVGs in
  `bench/comparison/2026-10-15/figures/`, which `bench/TOOLS.md` embeds:
  they are the comparison script's own output, not the manual's, and
  stay.
- A new run adds an item to the benchmark chapter's opening sentence,
  and "Beside the other provers" (README's text) states the 2026-10-15
  numbers as current: the first is the one line a new run adds besides
  its chapter; the second is for the session that runs the next
  comparison (below).

### What the next sessions must know

- **45b's growing families join** as a chapter
  `docs/src/results/<name>-<date>.md` listed in `SUMMARY.md` under
  "Benchmarks and the comparison", its figures a few lines of
  `docs/figures.sh` (`plots.py growth` for `run --tools` rows, a line
  per tool per shape; `plots.py families` for a baseline's
  `families.csv` and `engines.csv`), and an item in the benchmark
  chapter's opening sentence; the existing chapters stay as they are.
- **45c's fourth baseline** is a chapter `results/baseline-<date>.md`
  drawn the same way; `bench/RESULTS.md` and `bench/COMPARISON.md` stay
  the full tables.
- **45d prepares the front page and the manual for 0.2.0**: README's
  Install (`linlog = "0.1"`) and the Installation chapter's tag
  (`v0.1.0`) and Rust version; the introduction and README say a
  release's manual is its tag's `docs/`; "Beside the other provers" in
  `benchmarks.md` is README's old summary of the 2026-10-15 run and
  becomes stale with a new comparison.
- **A step that adds a command, flag or format** adds it to its chapter
  (`plan/conduct.md`, CLAUDE.md's verification table); a new area gets a
  chapter in `SUMMARY.md`. A chapter cannot drop out of the test: the
  test fails on a Markdown file `SUMMARY.md` does not list.
- **Locally**: `manual` in the devshell draws the figures into
  `docs/src/figures/` (ignored) and serves the book; `nix build
  .#manual` builds it as CI does.

### What is left open

- Links to the web page outside `/linlog/` are not checked, since the
  check is offline and the page is another repository's build.
- The figures are static SVGs: no hover, no table inside the figure.

### Checklist

- [x] README a front page of about 150 lines (149).
- [x] The manual under `docs/` (mdBook 0.5.4 through `new-tool`), a
  chapter per area, README's sections moved whole, the page's violet
  and logo in both schemes.
- [x] `cli/tests/readme.rs` runs every chapter's examples and checks
  every name; `bench/tests/manual.rs` the harness's; planted names fail.
- [x] The benchmark chapter: the third baseline in brief, the comparison
  with its cactus and scatter plots, linlog against LinearOne at first
  order, time against size for the families; every figure by
  `bench/plots.py` from the rows in both schemes, with what, where,
  when and what it shows.
- [x] Built into the Pages site beside the rustdoc (`doc`), checks
  `manual` and `manual-links`, every link resolving.
- [x] The rules (`cli.md`, `ci.md`, `flake.md`, `bench.md`),
  `plan/conduct.md` and CLAUDE.md name the manual.
- [x] The fresh-context reviewer's round, each finding answered above.
- [x] The gate passed (`gate: passed`) at the manual's commits.
- [x] `nix flake check` passed at the manual's commits (`c92d43d8`, exit
  0; a first run failed on the spell check, `typos`, for mdBook's own
  variable `--sidebar-non-existant`, now ignored by a pattern, and a
  made-up flag of a test, now a word) and again at the report's commit.
- [x] The client's About links the manual: `xpvoxxmk` (`0e1636ab`) on
  the client's `main` `ooyvskyv`, after linlog's push; About's sentence
  names the manual at `/linlog/manual/`, the About test pins the link,
  and `.claude/rules/web.md` says `/linlog/` holds the manual too (the
  service worker already leaves `/linlog/` to the network). The
  client's input of linlog stays where it was, since the page only
  links. Its `nix flake check` passed (Vitest 557, Playwright 124,
  `all checks passed!`, exit 0).
- [x] Nothing pushed by this session.
