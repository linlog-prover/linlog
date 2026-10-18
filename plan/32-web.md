# Step 32: the web front end

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing. The step takes three
sessions and has a plan of its own, which its first session writes; this
prompt is finished at the review of step 31. Read before you
start:

- `plan/later.md`: "The web front end", "Follow-ups: interactive
  proving".
- `plan/reports/09-interactive.md` ("For steps 10 to 12 and the web
  front end"), `11-svg.md` ("What the web front end will call"),
  `17-assessment.md` (3.14, D9, the author's answers), `18`, `22`, `28`.
- `plan/README.md`: D11, D12, D13, D15, D16, D18.
- `plan/notes/distribution.md`.
- `plan/web/design.md` and the mockup it names (`plan/web/mockup/`),
  written by the design session (`plan/32-design.md`) with the author:
  its decisions are fixed for this step, as the items below are.

## Goal

A student opens a page, types a sequent and proves it by clicking, or
lets the search close a goal, and sees the derivation and, for MLL, the
net drawn as the command draws them. Nothing is installed and nothing is
sent to a server.

## What is fixed now

1. **Two repositories** (D22; the author, 2026-10-03). In this
   workspace the crate `linlog-web`, the bindings: the library compiled
   to `wasm32-unknown-unknown` without the `parallel` feature, behind a
   small API of JSON in and JSON or SVG out (the interactive state, the
   options values of steps 22 and 28, the outcome of a search), with its
   tests run under wasm and its build a flake check from the first
   session, so that a change of the library that breaks them fails here.
   In a repository of its own under the organization `linlog-prover`
   the client: the page, its interface code, its assets and its
   deployment, with a flake that takes this repository as a pinned
   input, its own CI, licence headers and CLAUDE.md. The session sets
   that repository up locally, with jj as this one is; creating it on
   GitHub and pushing are the author's.
   **The repository's architecture is the session's** (the author,
   2026-10-18: "I don't mind, just choose whatever is idiomatic and
   follows current best practices"): its name, its directory, its
   layout, its build and its CI follow what is idiomatic today for the
   stack, checked against the stack's current documentation, not from
   memory; the report says what was chosen and why. A default if
   nothing argues against it: named `linlog-prover.github.io`, served
   at the organization's root address with the rustdoc staying at
   `/linlog`, in `/home/tux/Projects/own/linlog-prover.github.io`.
2. **What the target lacks** (checked 2026-10-03): `std::thread::spawn`
   and `Instant::now()` panic there; the stack is 1 MiB unless a link
   argument raises it; threads need cross-origin isolation, which GitHub
   Pages cannot set. So: one thread; a stop that counts the engine's
   work, with a budget that is an option; the stack raised or the
   recursion limit lowered, with a test at the limit under wasm; step
   18's bound on every derivation (`ViewOptions`), with a smaller
   default than the command's, and `Size::height` asked before a
   derivation is built, since the builder recurses to that depth unless
   step 20 gave it a stack of its own (it did: the builder, the parser,
   the printers and sequentialization no longer recurse on the input).
   The target is the first 32-bit one: the arguments that no integer of
   the checker, of the size estimate and of the memo wraps were made for
   64 bits, with a sentence each for 32, and nothing was ever compiled
   there. The bindings' tests run the checker's and the memo's tests at
   their limits under wasm32, and the memory and occurrence bounds get
   defaults that fit a tab (the library's gibibyte is close to what a
   tab has in all).
3. **The two searches of the default bias** take turns from their start
   without threads, at up to five times the better search. The first
   session measures it under wasm on the target set's small rows and
   says whether a search that can be suspended (an explicit stack in the
   focused engine) is needed; if so it is a step of its own, after this one.
4. **The first version is `linlog interact` with a mouse** and no more:
   a sequent, the modes, the goals with clickable formulas, the rules
   that apply, undo, close, the drawing, the finished proof's exports.
   Its plan says what is left out.
5. **Hosting** from the client's repository, built by its flake; this
   repository's Pages keep the rustdoc.
6. **Licences** (the author, 2026-10-18): the client's code under
   EUPL-1.2, as linlog's, with the same headers; its written content
   (explanations, tutorials, worked examples, exercises) under CC BY 4.0,
   said so in its README and on the pages that carry it. The bundle sent
   to a browser contains Astro's, Solid's and their dependencies' code,
   whose licences ask for their notices: the build collects them into a
   third-party licences file served with the site (a licence plugin of
   Vite or Rollup), and checks it; a font served for the drawings keeps
   its `OFL.txt` beside it. The stack the author expects: Astro,
   TypeScript and Solid. Once the client is served, the repository's
   website field (docs.rs/linlog since 2026-10-18) points to it, on the
   author's word.

## What waits

The choice of client technology (the first session compares a plain page
over wasm-bindgen with one Rust framework, by size, maintenance and what
the author would read); the names of steps 22 and 28. One caveat on the
two repositories: a client written in a Rust framework is a crate that
depends on `linlog`, which is the case for a workspace. If the
comparison ends there and the split then costs more than it gives,
depart from D22 by the author's word of 2026-10-18 above, marked
"(departs from the design)" with the reasons in the report.

## What steps 30, 31 and 35 left you

- **0.1.0 is released** (2026-10-18): `linlog` and `linlog-cli` on
  crates.io, Zenodo's DOI 10.5281/zenodo.23448588 in README and
  `CITATION.cff`. The bindings depend on the workspace's `linlog` by
  path, never on the published crate, and stay unpublished until the
  author says otherwise. The API may still change for the better until
  the next minor version (the changelog's policy line), and every
  change of it is a line of `[Unreleased]`.
- **The wire forms are promised by level**: `wire::LEVEL` is 2 since
  step 31, used only for a disproof or an outcome that holds a
  classical refutation; every other document is written at level 1,
  byte for byte. The client reads the level and refuses one it does
  not know.
- **What a search answers now**: after every search that does not
  prove its goal, the classical refuter looks for an assignment that
  falsifies the goal's classical reading, bounded by work
  (`search::Options::classical_work`, JSON `classical_work`, default
  2²⁶ terms, 0 for none) and by the memory account, never by a clock.
  An Unknown it refutes becomes Unprovable (`Refutation::Classical`).
  `Disproof::check` verifies the classical, unbalanced and
  count-equation refutations without the search's code; a page can
  show "checked" beside them as it does beside a checked proof. The
  command lets the refutation run past `--timeout` within its work
  bound; the page's stop counts work, so the bound is the page's
  budget, and a tab's default may be smaller.
- **Certificates**: `rocq::Options::kernel` (JSON
  `styles.rocq.kernel`, `"nanoyalla"` or `"linlog"`), and
  `rocq::write_disproof` and `rocq::write_not_valid` write Rocq
  certificates of non-derivability. The page offers them for download
  beside the proof's exports; it checks nothing in Rocq itself.
- **The limits**: `limits::DEFAULT_MEMORY_BYTES` is one gibibyte and
  `DEFAULT_OCCURRENCES` fifty million, both too much for a tab (item 2).
  The memory account (step 30) charges the searches' tables, the
  refuter's buffers and the net engine's, so a smaller bound is a
  refusal, never a crash.
- **The engines** (step 35): detection picks the engine (`focus`,
  `two-sided` for intuitionistic sequents, `additive`, `horn`, and
  `net` where the focused or two-sided engine stops at its recursion
  limit or memory bound on a sequent whose atoms occur once with each
  sign), the verdict line names the one that ran, and README says
  which goes where; under wasm's smaller stack and memory that hand-off
  to `net` happens sooner, which the first session measures. Step 35's
  second session adds essential nets for IMLL, as an engine and as a
  drawing (its report says what the SVG export takes). The page draws
  what the command draws.
- **What is checked for wasm today**: the flake check `wasm` runs
  `cargo check` of the library for `wasm32-unknown-unknown` with every
  feature but `parallel`, and nothing more: no test has run there, and
  nothing has run on a 32-bit target (item 2). The bindings' crate adds
  the tests.
- **The public cache** `linlog-prover` takes what the jobs on `main`
  build; nothing third-party may land in it. The client's CI pulls from
  it and pushes only its own outputs, and the client's flake builds no
  other prover and no problem library.

## How the step runs

Three sessions, each on Opus 5.5 at `high` (plan/README.md, "Why these
models and efforts", 2026-10-18), each with its goal in its launch
prompt: the first writes `plan/web/README.md`, measures item 3 under
wasm and builds the bindings with their wasm tests and flake check, in
this repository alone; the second sets up the client's repository and
its first version (item 4); the third finishes it, its hosting and its
licence check (items 5 and 6). The second session creates the
client's repository locally where item 1 says; when its first commits
are reviewed, the planning session creates it on GitHub with `gh repo
create` (the author, 2026-10-18), after one yes for that call as for
every administrative task, and pushes it. Sub-agents as
`plan/conduct.md` says: research on Sonnet 5.5, a fresh-context
reviewer on Opus 5.5 for the bindings' JSON boundary and for every
integer argument made for 32 bits.

## The machine

If another step runs beside this one, this step builds on cores 8 to
11 and the other keeps 2 to 7. A change of the core crate that the
target needs (a stack, a limit, an integer's width) is a commit of its
own that says why, so that the planning session can stack it beside a
step that changes the search; the bindings and the client's code touch
nothing else of the workspace.

## Deliverables

`plan/web/README.md` (the step's own plan, by its first session) and
`plan/reports/32-web.md`, both in this repository; thematic jj commits
here for the bindings and in the client's repository for the client.
