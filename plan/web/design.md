# The web front end's design

The design session of step 32 (`plan/32-design.md`) wrote this with the
author on 2026-10-18. Each decision names who made it and why, and the
sources are at the end, each with the date it was read. The research
behind it is `plan/notes/research/web-precedents.md` (the tools that
teach logic on the web, and what students stumble on) and
`plan/notes/research/web-stack.md` (the stack, the worker, URL state,
hosting, licences, accessibility), both of 2026-10-18. Step 32's
sessions treat the decisions as fixed; the open points are what the
five mockups explore, and Phase 3 closes them.

The research's two findings that shaped the most: no web tool draws or
edits proof nets interactively (the nearest is the Incredible Proof
Machine's wired blocks for natural deduction, and Click & coLLecT's
maintainer calls drawing nets from sequent proofs "not straightforward
if the output should look good", issue 174); and the Kassel Sequent
Calculus Trainer's exam cohorts, where a tool that refuses an illegal
step with a reason cut wrong-rule failures from above half to about a
quarter while the marks barely moved, because finding the proof stayed
hard once every step was legal. So the page separates "is this step
legal", which the library answers at once, from "which step helps",
which is a hint the student asks for.

## Who it is for and what it is for

- **D1. The page opens on the work, with nothing in the way** (the
  author). Someone who knows sequent calculus starts at once: the
  sequent field and the stage are the first thing on the page, with no
  landing text, no tour and no modal. The student's path (examples,
  explanations, hints) is reached from the page, never laid over it.
  The session had recommended opening on a worked example in progress;
  the author prefers a direct open, and the examples stay one click
  away. Carnap's retrospective makes trust and speed the base of
  engagement, which a direct open serves.
- **D2. The default is classical one-sided linear logic** (the author),
  linlog's own calculus. Intuitionistic mode, Mix and affine weakening
  are labelled toggles that stay in view (a labelled switch, not an
  icon, since a student landing cold has never written a negation
  normal form and must find the two-sided mode). A session is one
  sequent in one mode, so a change of mode restarts the proof, and the
  switch says so before it does. Ordinary propositional logic
  (classical, intuitionistic, minimal, through the translations) is a
  part of its own with the same stage and the same look, never mixed
  into the linear page's controls.

## The layout

- **D3. A centre stage** (the author): the derivation tree or the proof
  net being worked on fills the page, and the camera zooms and pans
  (the survey's most reported complaint about tree editors is a tree
  wider than the screen with no zoom, and linear sequents, with no
  weakening to shed formulas, grow wide early). Everything else is a
  head-up display at the edges or a drawer: the mode toggles, the view
  tabs, undo, close, hint, tour and the learning toggle in the HUD,
  easy to tell apart and never loud; import other than typing, export,
  the step list, settings and the like in a drawer.
- **D4. Derivation and net are two views of one stage** (the author).
  For a sequent the net takes (MLL, and the essential nets of IMLL) the
  two tabs switch freely; otherwise the net tab is shown greyed, with
  the reason, so that the student learns there is such a view and when
  it exists. Version 1 shows the net of a finished proof; a net editor
  (the student links literals pair by pair, the switching cycle drawn
  when one closes) has no precedent and is a later step, which the
  mockups may sketch (the session's recommendation; the author took
  it for now, unsure, so the sketches inform a later decision).
- **D5. One goal at a time** (the author): the whole tree stays on the
  stage, the active goal is brought into view and the other open goals
  are dimmed; the next and previous open goal are reached from the HUD
  and by keyboard. Clicking another open goal makes it the active one.
- **D9. The rules appear at the formula** (the session's
  recommendation, the author agreed): clicking a formula selects it
  and opens a small popover at it that lists only the rules the
  library gives for that formula (`Interactive::rules`: by connective
  and mode alone), so the eye never leaves the goal (the
  split-attention effect). A rule is never applied silently on a
  click, even when it is the only one, because the critics of Click &
  coLLecT find clicks that change the state "in cryptic ways" the
  tool's weakness; the rule's name is printed on every inference bar.
  A `⊗` or Mix asks for the split by clicking the formulas that go to
  the left premise, with `split_passes` greying a split that cannot
  pass. No rule table is on the stage; a refusal by the context (the
  axiom's dual, `1` or `!` not alone, the wrong side in intuitionistic
  mode) is tried and refused with the library's reason (D11). The
  author leaned to showing only the rules that apply from the start;
  the one counter-report (NaDeA's authors, blind clicking on small
  proofs) is one course, and the Kassel data on refusals with reasons
  are the stronger evidence.
- **D10. The search's inferences are marked apart from the student's**
  (the author): when the search closes a goal, its inferences are
  grafted into the tree with a lighter rule label, so a student sees
  what they proved and what the machine did.

## The interaction

- **D11. Two kinds of feedback, kept apart** (the session, from Shute,
  Narciss and the Kassel data; the author agreed). A refusal is
  elaborated: the library's reason as one plain sentence at the goal
  ("ax needs A and its dual alone in the goal; B is still there"),
  persisting until the next action, never modal, never a code and
  never praise. A legal step that leads nowhere gets no comment unless
  the student asks for a hint. The verdicts stay three-valued with an
  honest "unknown": Carnap's author says a time-out shown as "no" costs
  the tool's credibility.
- **D12. Hints on request, from the search** (the session; the author
  agreed). Never on a timer: the default visitor is an expert, and the
  expertise-reversal effect says proactive help costs them. The source
  is the library itself, with no server: the search on a copy of the
  active goal yields a proof whose first inference is the next step.
  Four rungs, each one thing: which goal and which kind of connective
  to try first (the invertible rules first, a lesson of focusing);
  which formula; which rule and, for a split, which formulas go left;
  bottom-out, where the page applies the step and asks the student to
  say why it was the one. The ladder is the same with the learning
  toggle on; what the toggle adds is the examples and explanations,
  not earlier hints.
- **D13. Undo is linear per branch** (the author): undo retracts the
  last step on the active goal's branch; undoing past a split retracts
  the split and every step in its other premises. Redo is kept until a
  new step. The step list in the drawer shows every step, and clicking
  one goes back to it. The library's `undo` retracts the last step of
  the session, so the client keeps the steps as a tree and replays the
  rest (`replay.rs` is the model) when a branch is retracted; the
  replay is also what a link needs (D20).
- **D14. Closing a goal** is a button at the active goal and one for
  all goals, each with the page's budget (work, not time: the library
  has no clock under wasm); the result is grafted (D10) or reported
  (D15).
- **D15. The failures** (the session's recommendation; the author set
  the tone: plain sentences, no error codes, the command's facts in
  the page's own words). Input that does not parse: the message under
  the sequent field with a caret at the place, the stage untouched.
  Unprovable: the verdict where the goal was, the reason in one
  sentence (an atom's balance, the count equation, the classical
  countermodel), "checked" beside a refutation that `Disproof::check`
  verified, and an expander for the countermodel or the balance. A
  search that gave up: amber, the budget it used, and a button that
  doubles it; the engine named. The verdicts are the library's
  `Refutation` and `Outcome` values, which carry the facts; the words
  are the page's.
- **D16. The exports, all of them** (the author): SVG, PNG, PDF, LaTeX,
  Typst, the session as JSON, the Rocq certificate (and the
  certificate of non-derivability for a refuted goal), and the link,
  in the drawer. PNG and PDF take the font bytes the page already
  loads. A partial proof exports too, with its open goals as the
  command draws them.
- **D17. Keyboard** (the author agreed): every formula is a button
  reached by Tab, Enter opens its rules, a digit picks one, single
  letters for undo, close and hint, arrows between goals, and a key
  list behind `?`. Everything the mouse does the keyboard does.
- **D18. Screens** (the author): large screens are the main use, and
  the page works well at every size; on a phone the HUD becomes a
  bottom sheet and the stage keeps the zoom and pan. Targets are 24
  pixels at least (WCAG 2.2's 2.5.8) and nothing needs a drag (2.5.7).
- **D19. No console** (the author): the session had proposed the
  command language of `linlog interact` in the drawer as a second
  means of action; the author finds it unnecessary. The derivation as
  text stays available as an export, and the keyboard path (D17) is
  the second means of action.

## The learning design

- **D6. Learning is a top-level toggle** (the author). The page has one
  switch that turns the student's additions on: with it off the page is
  the expert's editor of D1; with it on, further elements appear (the
  examples to load, the explanations, the prompts) without changing the
  stage or the controls the expert uses. One switch answers the
  expertise-reversal problem: what helps a novice (worked examples,
  prompts, visible guidance) costs an expert attention. The content
  behind the toggle is the written part of the site, under CC BY 4.0.
- **D7. Loading a sequent to prove is one click** (the author): the
  examples are a list from which a sequent goes onto the stage at
  once, replacing the current work after a warning if that work is
  unsaved. The list ships in version 1, since a replayable example is
  only a saved session.
- **D8. A tour of the page, behind a button** (the author): a guided
  walk through the interface, started by the visitor and never on its
  own, so that D1 holds. The session's recommendation: a live tour,
  each of its seven or eight stops performing a real step on the
  README's first example, so the visitor ends it with a checked proof
  on the stage. The tour explains the interface, not the logic; it
  works by keyboard, under reduced motion and with a screen reader.
- **D21. The learning content's shape** (the session, from Renkl,
  Atkinson, McLaren; the author agreed to the scope). Every proof is a
  replayable session and the search can finish any prefix, so worked
  examples and their fading come free: an example is a finished proof
  played step by step with a one-line reason at each inference and a
  "predict the next rule" prompt before each reveal; a faded one is
  the same proof stopped some steps before the end (backward fading);
  then a bare sequent. Erroneous examples cannot be wrong derivations
  here, so they are dead ends: a legal split that sends both duals to
  one premise, a sequent that looks provable until the refutation shows
  an atom's balance, a net with a switching cycle. Version 1 ships the
  examples list with replay; the reasons, the prompts, the fading and
  the dead ends are a later session's content, and the mockups show
  them so that the layout has a place for them.
- **D22. Dual coding** (the session): the tree is the picture and the
  rule names and reasons are the words, so a reason sits at its
  inference and never in a side panel; the net and the tree are two
  representations of one proof, which the tabs let the student
  compare; colour carries meaning only (the active goal, the principal
  formula, the switching cycle), never decoration.
- **D23. The written content lives in the client's repository** (the
  author), as Markdown under CC BY 4.0: the explanations, the examples
  and their sessions, the tour's text.

## The visual design

- **D24. The look** (the author): visually appealing and modern,
  intuitive, with visual feedback on every action, accessible; it may
  be a little flashy, but the interface stays unobtrusive and never in
  the way of the work. Information or settings that matter less live
  behind a tooltip or inside an expandable dialog (progressive
  disclosure). The mockups range from the textbook page (paper-like
  surfaces, generous margins, a quiet text face) to the instrument
  (dark, dense, a HUD that looks like one), and the author chooses.
- **D25. Type** (the project's standing decision): Euler Math wherever
  linlog draws (the SVG of sequents, derivations and nets), served as
  a web font under the family name `Euler Math` with its `OFL.txt`
  beside it (the CTAN package has none, so the client writes one with
  the font's three copyright lines; the font has no Reserved Font
  Name, so serving it unmodified under its own name is fine), and
  nowhere else; the text face is the mockup's choice among OFL or
  otherwise free faces, with no third-party file committed to this
  repository.
- **D26. Colour** (the session's recommendation, taken): one accent for
  selection and the active goal; red kept for refusals and unprovable,
  amber for unknown and a search that gave up, green only for a
  checked proof, the three-valued scheme of Click & coLLecT and
  Kassel; light and dark follow `prefers-color-scheme`, with a manual
  override in the drawer; the drawing's `Style` takes the theme's
  colours.

## The architecture

**D37. The client follows the author's two sites** (the author,
2026-10-18): `~/Projects/own/website` (grubmueller.dev, Hakyll, on its
way to the same stack) and `~/Projects/own/website-learn`
(learn.grubmueller.dev, the learning sites), whose stack
`~/Projects/own/website/docs/stack-spec.md` decides and explains, with
sources. The client takes every part that applies, so that components,
design tokens and Claude guidance can be shared, and the step 32
session that sets the repository up reads that specification and
`website-learn`'s `CLAUDE.md`, `.claude/rules/` (`web.md`,
`islands.md`, `design.md`, `nix.md`, `ci.md`) and `modules/` before it
writes a line, copying what applies rather than reinventing it. The
stack survey (`web-stack.md`) agrees with the specification on every
point it covers. In particular:

- **D27. Astro 7, TypeScript 6, Solid 1.9** (stack-spec §3.1, §3.4,
  §3.7): the app as one Solid island (`client:only`), the written
  content (D23) in content collections as MDX on the unified pipeline
  with remark-math and rehype-katex (`strict`, errors fail the build,
  §3.2) where it sets mathematics; no framework for small things (a
  toggle, the URL sync: plain TypeScript); the Web Animations API for
  motion; Solid 2.0 only when the sites move, by the specification's
  procedure ("Moving to Solid 2.0"); the sites' Solid idioms rule
  (`islands.md`) copied. `tsconfig` extends `astro/tsconfigs/strictest`
  with `verbatimModuleSyntax` and `erasableSyntaxOnly`.
- **D38. Tailwind v4 through the Vite plugin, tokens in CSS** (§3.6):
  Tailwind's own palette, fonts and scale switched off (`--color-*:
  initial`), every colour role a `light-dark()` pair under `@theme
  static` so that one definition serves both schemes and `<html
  data-theme>` pins one, nothing anywhere naming a raw colour or size
  (`website-learn/src/styles/tokens.css` and `docs/design-system.md`
  are the model); the mockup's token system becomes this file.
- **D39. Biome 2, Vitest with fast-check, Playwright** (§3.7, §6.6):
  Biome formats and lints TypeScript, JSON and CSS from one
  `biome.json`, `.astro` files as the sites do; `astro check` for
  types; Vitest for the client's logic (the URL codec, the step tree,
  the worker protocol); Playwright against the built site in nixpkgs'
  Chromium, one test per interaction of the island and one per page
  for the rules every page meets, each test at the cheapest level that
  sees what it guards; under CI 60 s and one retry.
- **D40. npm under Nix** (§3.8): `importNpmLock` from
  `package-lock.json` (no hash to keep in step), foreign platforms'
  packages filtered before fetching, `.npmrc` with
  `package-lock-only`, `save-exact`, `min-release-age=14`,
  `ignore-scripts`, `allow-git=none`, `engine-strict`; the devshell
  links `node_modules` from the store (`modules/node-modules.nix` and
  `devshell.nix` of `website-learn`, taken over). Not pnpm, which the
  survey had suggested: the specification weighed it and chose npm's
  settings.
- **D41. The flake** in the dendritic pattern as this repository's and
  the sites': flake-parts, import-tree, devshell, treefmt-nix, nixpkgs
  from the channel tarball; one check per concern, each reading only
  its own files through `lib.fileset`; a `ci-cache` derivation that
  roots everything CI must find in the store.
- **D28. The bindings in a module Worker** (§3.5 and the survey): the
  crate `linlog-web` of this workspace compiled to
  `wasm32-unknown-unknown` with the `wasm-bindgen` CLI at the version
  `Cargo.lock` pins (a flake check compares the two), never wasm-pack,
  `wasm-opt` after it, built by this repository's flake as a package
  the client's flake takes through its pinned input, so the client
  carries no Rust toolchain of its own; loaded as `--target web`
  through Vite's `?url` import, the worker created as `new Worker(new
  URL(…, import.meta.url), {type: "module"})`, a hand-rolled request-id
  protocol, cancellation by `terminate()` and a fresh worker. The
  stack is raised with `-zstack-size` from the 1 MiB default; a tab on
  iOS gets about 256 MiB of wasm memory, which sets the memory bound's
  default. `'wasm-unsafe-eval'` in `script-src`.
- **D42. The security baseline of the sites** (§4) as far as the host
  allows: no `style` attribute in the built site (the sites'
  styles-to-classes plugin, `inline-styles.ts`, taken over, with a
  check), all CSS from files (`inlineStylesheets: 'never'`), inline
  scripts and styles allowed by generated hashes, `'wasm-unsafe-eval'`
  only. See D31 on where the policy can live.
- **D20. The link** (the author: the Unicode sequent, readable): the
  URL fragment, never the query, so nothing reaches a server, holding a
  version mark, the sequent in the parser's own Unicode syntax, the
  mode and the steps as a short list (goal, position, rule, the left
  positions of a split), replayed by the library on load, so a link
  stays valid across versions of the library. Plain text while it is
  short, deflate-raw through `CompressionStream` with base64url when
  the fragment would pass about 2000 characters; written with a
  debounced `replaceState`.
- **D29. Autosave** of the current session in the browser's storage,
  with a restore bar on the next visit; the precedents' most reported
  loss is work gone on a reload.
- **D30. No offline support in version 1**, as the sites (installable,
  no service worker); later `vite-plugin-pwa` with a navigation
  denylist for `/linlog/`, since every Pages site of the organization
  shares one origin and a root-scoped service worker would also
  control the rustdoc.
- **D31. Hosting, build, CI**: `plan/32-web.md` fixes hosting from the
  client's repository and names GitHub Pages at
  `linlog-prover.github.io`, with `/linlog` staying the rustdoc, a
  `404.html`, no `.nojekyll` needed, `upload-pages-artifact` and
  `deploy-pages` (both v5). The sites are on Cloudflare Workers with
  Wrangler, which sets headers: a CSP as an HTTP header, HSTS, COOP,
  CORP, the `Permissions-Policy`, and Brotli. GitHub Pages sets no
  header, so the CSP there is a `<meta>` tag, which cannot carry
  `frame-ancestors`, and the other headers are lost; it serves `.wasm`
  gzip only with `max-age=600`. **GitHub Pages it stays** (the author,
  2026-10-18): the organization's, free, with no account behind the
  site, and nothing on the page handles data that would want those
  headers. CI as the sites': every push
  to `main` checked and deployed, a pull request only checked, actions
  pinned by SHA, `cachix/install-nix-action`, `cache-nix-action`
  restoring the newest store and saving under `ci-cache`'s hash; with
  Pages the public Cachix cache `linlog-prover` is available too, and
  the client pushes only its own outputs to it.
- **D43. Updates** (the author, 2026-10-18): whenever the author asks
  for an update, every tool and dependency moves to its latest release
  that is at least fourteen days old (the `.npmrc` cooldown applied to
  every ecosystem the client uses: npm, the flake inputs, the actions,
  the toolchain), and the update is verified by the flake's checks
  before it is committed, as this repository's `/update-deps` does.
  Solid 2.0 is the one exception: it waits for the specification's
  two conditions (a stable release and Astro's integration accepting
  it), and then gets a session of its own.
- **D32. Licence notices**: Vite's own `build.license` collects the
  npm bundle's; the Rust crates inside the wasm module need
  `cargo-about` or the like, run in this repository's flake beside the
  bindings' package and served with them; the OFL as D25 says.

## The quality floors

- **D33. WCAG 2.2 at AA**: the criteria that bite are keyboard (2.1.1),
  focus visible (2.4.7) and not obscured (2.4.11), dragging movements
  (2.5.7), target size (2.5.8), reflow (1.4.10), non-text contrast
  (1.4.11) and status messages (4.1.3, every verdict and refusal in a
  live region). An interactive SVG must not sit inside `role="img"`,
  which makes its descendants presentational: the clickable formulas
  are HTML buttons over or beside the drawing, the drawing stays the
  picture with a title and description.
- **D34. Motion and scheme**: `prefers-reduced-motion` turns every
  transition off (the camera's moves become cuts), `prefers-color-
  scheme` and `prefers-contrast` are honoured.
- **D35. The first load**: the critical path under web.dev's budget of
  about 170 KB compressed on slow 3G, so the wasm module loads after
  the first paint with the sequent field usable before it; Interaction
  to Next Paint under 200 ms, which the worker guarantees for the
  main thread.
- **D36. No tracking**, and a footer line that GitHub logs visitors'
  addresses, since GitHub's privacy statement says so for Pages.

## Open points for the mockups

What the five mockups vary, each one direction; nothing decided above
moves.

1. The tone between the textbook page and the instrument (D24), and
   the text face beside Euler Math (D25).
2. The HUD's form: bars at the edges, floating clusters, a single
   toolbar; and the drawer's: a side panel, a bottom sheet, a modal
   sheet.
3. The density: how much is on the stage at once, how much the HUD
   shows before a hover.
4. The goal focus (D5): how the dimming and the camera's move look,
   and whether a goal list sits beside the tree.
5. The rule popover (D9): its shape, how the split selection reads.
6. The failures (D15): the exact place and wording of each, and how
   the "checked" mark and the expander look.
7. How much the learning toggle (D6) adds and where: the examples
   list, the explanation of the current rule, the prompts.
8. The hint ladder (D12): where the button sits and how a rung appears
   and goes away.
9. The tour (D8): its form (spotlight, side notes, inline).
10. The net view (D4): the finished proof's net, the tab's greyed
    state, and a sketch of the editor.
11. The phone layout (D18).

## Sources

Read on 2026-10-18 unless another date is given; the research notes
hold the full list with what was read in the source itself.

- Carnap's retrospective, arXiv 2607.07722 (July 2026): "engagement
  requires trust"; client-side checking at the scale of 45 000
  students.
- The Kassel Sequent Calculus Trainer, arXiv 1507.03666 and arXiv
  1803.01467: refusals with explanations; the exam cohorts; "finding
  the proof" as the remaining difficulty.
- Click & coLLecT, https://click-and-collect.linear-logic.org/ and its
  issue tracker (issue 174 on nets; 165, 169, 143 on exchange).
- The critique of click-to-apply interfaces, arXiv 2303.05865; the hint
  ladder, arXiv 2503.19280; NaDeA, https://nadea.compute.dtu.dk/;
  MiniCalc, arXiv 2505.05988; the Incredible Proof Machine,
  http://incredible.pm/ and its ITP 2016 paper; Logitext,
  http://logitext.mit.edu/main.
- Sweller (1988, 1994), Kalyuga et al. (2003) on the expertise
  reversal effect; Shute (2008), Narciss (2008), Hattie and Timperley
  (2007) on feedback; VanLehn (2011), Aleven and Koedinger (2002),
  Razzaq and Heffernan (2010) on hints; Renkl (2014), Atkinson et al.
  (2000) on worked examples and fading; McLaren et al. (2012, 2015) on
  erroneous examples; Mayer (2009) on multimedia; CAST (2018) on UDL:
  as the education skills cite them.
- Astro 7, https://astro.build/blog/astro-7/ and
  https://docs.astro.build/ (the Solid integration, content
  collections, client directives, imports); Solid,
  https://github.com/solidjs/solid/releases; Vite 8,
  https://vite.dev/blog/announcing-vite8 and
  https://vite.dev/config/build-options (`build.license`),
  https://vite.dev/config/worker-options.
- wasm-bindgen, https://wasm-bindgen.github.io/wasm-bindgen/ (web
  workers, deployment, size); the rustwasm sunset,
  https://blog.rust-lang.org/inside-rust/2025/07/21/sunsetting-the-rustwasm-github-org/;
  the wasm32 target, https://doc.rust-lang.org/rustc/platform-support/wasm32-unknown-unknown.html;
  the 1 MiB stack in rustc's `wasm.rs`; WebKit bug 269777 on wasm
  memory on iOS; https://v8.dev/blog/4gb-wasm-memory.
- MDN: module workers, `Worker.terminate`, `CompressionStream`,
  `History.replaceState`, `crossOriginIsolated`, the `img` role;
  https://caniuse.com/mdn-api_worker_worker_ecmascript_modules; the
  compression streams baseline at web-platform-dx.
- GitHub Pages: about, publishing source, limits, custom 404,
  https://github.com/orgs/community/discussions/13309 (no custom
  headers), the general privacy statement; `actions/upload-pages-
  artifact` and `deploy-pages`.
- The OFL text and FAQ, https://openfontlicense.org/; the Euler Math
  package, https://tug.ctan.org/fonts/euler-math/README.md; fonttools'
  subsetter.
- WCAG 2.2, https://www.w3.org/WAI/WCAG22/Understanding/ and what is
  new in 2.2; the SVG accessibility API mappings; TPGi on ARIA in SVG.
- web.dev: performance budgets, Core Web Vitals, INP, web font
  loading, service worker lifecycle.
- Precedents for wasm in a worker: Pyodide's web worker and keyboard
  interrupt pages; Excalidraw; eframe's template.
