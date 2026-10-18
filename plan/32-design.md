# Step 32's design session: the web front end's design and five mockups

You are the design session of the linlog repository's web front end, the
client step 32 builds (`plan/32-web.md`). The author (Fabian, GitHub
`flgrubm`) talks with you first, then you have five mockups made, each
independently, and the author picks one or none. You do not build the
client: step 32's sessions do, from what you leave. CLAUDE.md applies
throughout (jj only, never git; thematic commits; no pushing; the
author's shell is nushell, so commands for them use `;` and `(open
--raw f)`).

## Read first

- `plan/32-web.md`, whole: the goal, what is fixed (two repositories,
  what WebAssembly lacks, the first version as `linlog interact` with a
  mouse, the hosting, the licences), and the author's word of
  2026-10-18 that the client repository's architecture is to be
  whatever is idiomatic and follows current best practice.
- `plan/later.md`, "The web front end" and "Follow-ups: interactive
  proving"; `plan/reports/09-interactive.md` ("For steps 10 to 12 and
  the web front end"), `11-svg.md` ("What the web front end will
  call"), `22-configurable-output.md`.
- README's "A first proof" and its `interact` section: what the command
  already does, which the page does with a mouse.
- `plan/notes/distribution.md` (one repository or several).
- The project's design notes: the SVG export draws with the Euler Math
  font (`cli/fonts/Euler-Math.otf`, OFL-1.1, its `OFL.txt` beside it);
  the page uses it wherever linlog draws, and only there.

## Phase 1: the conversation

The author wants a conversation, not a questionnaire. Ask a few
questions at a time, propose an answer to each (your recommendation
first, with its reason), and go on from what they say. Cover:

- **Who it is for and what it is for**: students meeting sequent
  calculus and proof nets, teachers showing a proof, researchers
  checking one; which comes first, and what the first visit must
  achieve.
- **The learning design**, with the education plugin's skills
  (`education-agent-skills:*`), whichever fit; at least the cognitive
  load (`cognitive-load-analyser`), feedback in a digital tool
  (`ai-feedback-design-principles`), hints that do not give the answer
  away (`adaptive-hint-sequence-designer`), worked examples and their
  fading (`digital-worked-example-sequence`,
  `worked-example-fading-designer`), wrong steps as a way to learn
  (`erroneous-example-designer`), pictures beside words
  (`dual-coding-designer`) and access for every learner
  (`udl-options-designer`). Say which apply to a proof assistant whose
  rules are exact and whose checker is the authority, and which do not.
- **The visual design**, with the `frontend-design:frontend-design`
  skill: the subject's own vernacular (sequents, derivation trees, proof
  nets, the textbook page), the typography beside Euler Math, colour in
  light and dark, how a goal, a clickable formula, an applicable rule,
  an error and a finished proof look.
- **The interaction**: how one clicks a formula and a rule, how the
  derivation grows (tree, list of goals, both), undo and history, the
  search closing a goal, the net view for MLL, the exports, the keyboard
  path for everything the mouse does, phone and tablet.
- **The architecture**: the client's stack (the author expects Astro,
  TypeScript and Solid; weigh it honestly against today's alternatives),
  the WebAssembly bindings' place (in a Web Worker, so the page never
  blocks), state and URLs (a proof shareable as a link, nothing sent to
  a server), offline use, the repository's layout, build, CI, hosting on
  GitHub Pages, the licence notices of the bundle.
- **Quality floors**: WCAG 2.2 at AA, reduced motion, the time a first
  page takes on a slow phone, no tracking.

**Look things up**: current best practice for each of these, from
official documentation and reputable sources (the frameworks' own docs,
W3C, MDN, web.dev, research on learning), checked online today, never
from memory alone; and the precedents in logic teaching (Click & coLLecT,
the Incredible Proof Machine, Carnap, Logitext, and others you find):
what each does well and what students stumble on. Cite each source with
its date in the design document.

Record each decision as it is made in `plan/web/design.md` (create the
directory): the decision, its reason, its source; and the open points
the mockups are to explore. Commit it with jj as you go ("Plan: the web
front end's design, …"). Ask the author before Phase 2 whether the
document is right.

## Phase 2: five mockups, each made independently

Prepare, before any mockup starts:

- **The real content**: build the command (`nix build`, cached; capped
  as below) and write, with it, the SVG drawings, the derivations' text
  and the JSON of the interactive states for three or four example
  sequents the conversation chose (an MLL one with its net, one with an
  exponential, an intuitionistic one, an unprovable one with its
  refutation), into `/home/tux/Projects/own/linlog-design/data/`, with
  a short `README.md` saying what each file is. Every mockup uses this
  real output, not invented formulas.
- **Five briefs**, each the design document's decisions plus one
  direction of its own, chosen so the five differ along the axes the
  conversation left open (for example the layout of the derivation, the
  density, the visual tone, how much the page guides), never in what was
  decided.

Then start five sub-agents (`Agent`) at once, each with a fresh context,
on Opus 5.5 at `high`, each with its brief and told:

- to load and follow the `frontend-design:frontend-design` skill (its
  two passes: a token system first, reviewed against the brief, then
  the code) and the education skills its brief names;
- to write a self-contained static prototype in
  `/home/tux/Projects/own/linlog-design/mockup-N/`: `index.html` with
  its CSS and JavaScript, no build step and no network at view time
  (fonts copied in with their licences; only fonts under the OFL or
  another free licence), that walks through the example proofs by
  clicking, with scripted states, and shows the drawing, the hints, an
  error, a finished proof and its export;
- to write `mockup-N/README.md`: its design plan (palette, type,
  layout, principles), which learning principles it applies and how,
  and what it would cost to build in the decided stack;
- not to read any other `mockup-*` directory, and to work only in its
  own;
- the machine's rules: every command in a memory-capped scope
  (`systemd-run --user --scope -p MemoryMax=4G -p MemorySwapMax=0
  taskset -c 12-15 …`), nothing that runs unbounded, no server left
  running (a local static server only on `127.0.0.1` and stopped
  after), no `npm install` or other download; screenshots of its own
  pages for self-critique if a browser is available (a headless
  Chromium from nixpkgs, capped as above), at phone and desktop width,
  in light and dark.

When all five are done, write `/home/tux/Projects/own/linlog-design/index.html`,
a page that shows the five side by side with each one's screenshot and
its one-paragraph rationale, and links to each.

## Phase 3: the choice

Show the author the five (`xdg-open` the index, or the command for them
to type). Help them choose, or combine one's layout with another's
details; record the choice and what is taken from the others in
`plan/web/design.md`, with the chosen mockup's path as the visual
reference, and copy the chosen mockup to `plan/web/mockup/` so that it
survives, without its fonts (no third-party file is committed; its
README names each font and where it came from), with the licence header
in each file's comment syntax, and with the flake's `conventions`,
`typos` and `treefmt` checks passing on it. Commit, then tell the author
the design is ready for step 32, whose orchestrator is started with the
command in `plan/handoff.md`. Push nothing; the planning session pushes.
