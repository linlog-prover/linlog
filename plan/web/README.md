# The web front end's plan

Written by step 32's first session (2026-10-19) from `plan/34-web.md`
and the design (`plan/web/design.md`, D1 to D60, fixed). It says which
session builds what, what version 1 is and leaves out, the bindings'
API as `linlog-web` offers it, and what the library lacks for the page.
The decisions are the design's; where this plan settles something the
design left open, it says so.

## The sessions

Three sessions, each on Opus 5.5 at `high` (`plan/34-web.md`, "How the
step runs").

**32a, the plan and the bindings** (this repository alone): this
plan; the crate `linlog-web` (`web/`), its tests under wasm32 and its
flake checks; item 3's measurement. It builds, of the design:

- D28's half in this repository: the module built with the
  `wasm-bindgen` CLI at the version `Cargo.lock` pins (the flake check
  `web-bindgen` compares the two), `--target web`, `wasm-opt` after it,
  the stack raised with `-zstack-size`, the memory bound's default set
  by a tab's 256 MiB; the package `linlog-web` that the client's flake
  takes through its pinned input.
- D32's half in this repository: the notices of the Rust crates inside
  the module, collected by `cargo-about` into the package
  (`THIRD-PARTY-NOTICES.txt`).
- The values the page words itself: D11 and D15 (every refusal and
  verdict as the library's JSON), D12 (the hint's search on a copy of
  the goal), D14 (a goal's close and all goals' within one work
  budget), D16 (every export), D20 and D51 (a session and a sequent read
  back at their level), D59 (the fragment), D26 (the drawing's `Style`
  takes the theme's colours), D4 (the reason a sequent has no net, for
  the greyed tab).

**32b, the client's repository and version 1's core**: sets up
`/home/tux/Projects/own/linlog-prover.github.io` with jj (the default
of `plan/34-web.md` item 1, unless that session finds a reason against
it; creating it on GitHub and pushing are the planning session's). It
reads `~/Projects/own/website/docs/stack-spec.md` and `website-learn`'s
`CLAUDE.md`, `.claude/rules/` and `modules/` first (D37) and builds:

- the stack and its tooling: D27 (Astro 7, TypeScript 6, Solid 1.9, the
  app one island), D38 (Tailwind v4, the mockup's tokens in CSS), D39
  (Biome, Vitest with fast-check, Playwright), D40 (npm under Nix), D41
  (the dendritic flake, taking this repository's flake as a pinned
  input), D43's procedure for updates;
- D28's other half: the module in a module Worker, the request-id
  protocol, cancellation by `terminate()` and a fresh worker;
- the page of `linlog interact` with a mouse: D1, D2 (the default
  calculus, the labelled switches, the restart warning), D3, D5, D9,
  D10, D11, D13 (undo per branch, by the client's own step tree and
  replay), D14, D15, D16, D17, D18, D19, D22, D24 to D26, D33, D34,
  D35, D44 to D48 (the one-row top band, the dock at the goal, the
  bottom band by intent, the light and dark toggle, the dotted grid),
  D53 to D55, D58 (the cluster of switches; the two-sided presentation
  of classical derivations greyed with its reason), D59;
- what holds the work: D20 (the link), D29 (autosave), D50 (several
  proofs at once), D51 (import).

**32c, the finish**: hosting and the licence check (`plan/34-web.md`
items 5 and 6) and the rest of version 1:

- D31 (GitHub Pages at `linlog-prover.github.io`, the rustdoc staying
  at `/linlog`, a `404.html`, the CI of the sites), D42 (the security
  baseline as far as Pages allows, the CSP as a `<meta>` tag with
  `'wasm-unsafe-eval'`), D36 (no tracking, the line on GitHub's logs);
- D32's other half (Vite's `build.license` for the npm bundle, served
  with the package's `THIRD-PARTY-NOTICES.txt` and the font's
  `OFL.txt`), item 6's CC BY 4.0 for the written content, said in the
  client's README and on the pages that carry it;
- D49 (offline and installable: the service worker, scoped away from
  `/linlog/`, and the manifest), D52 (About), D7 (the examples list),
  D8 (the tour), D6 and D47 (the Learn toggle and its frame, with the
  place for the tutorial, practice and reference), D21 and D23 (the
  content's shape and where it lives);
- the repository's website field pointing to the client, on the
  author's word.

Learn's lessons (D56, D60) are written by 32c or a content session after
it, every sequent in them proved or refuted by the command first.

## Version 1

`linlog interact` with a mouse (`plan/34-web.md` item 4), widened by the
design: a sequent in the field, the calculus's switches, the goals with
clickable formulas, the rules a formula takes, the split, undo per
branch and redo, a goal's close and all goals', the hint ladder, the
derivation drawn and, for a finished MLL or IMLL proof, its net, every
export, the link, autosave, several proofs, import, About, the examples,
the tour and Learn's frame.

Left out of version 1, each for a later step:

- the net editor (D4): linking literals pair by pair; the library's
  `ProofStructure::link` and `unlink` exist, the bindings do not offer
  them yet;
- the two-sided presentation of classical derivations (D58): a library
  feature (`plan/later.md`), its toggle greyed;
- ordinary propositional logic (D2, D57): the tab's choice between
  linear and ordinary is greyed; the bindings would need the JSON forms
  the library lacks (below);
- Learn's lessons (D56, D60) and the content of D21 beyond the examples
  list (the reasons, the prompts, the fading, the dead ends);
- a progress display during a long search: a call answers when it
  ends; the page shows that it is searching and can stop it.

## The bindings' API

`linlog-web` (`web/`, `publish = false`) is the library compiled to
`wasm32-unknown-unknown` without `parallel`, with every other feature.
Every call takes JSON text and returns JSON text, an SVG document or
bytes, and keeps nothing between calls: the page holds the session
document and the settings, so a worker stopped mid-call loses nothing.
A failed call throws a string, the error's JSON as the library writes
it (`{"code", "kind", "message", "setting", "details"}`, with formulas
written as text where a session has them); the page branches on `code`,
`kind` and `details.kind` and words the message itself.

`settings` is always the JSON of `linlog::Settings` read over the tab's
defaults: an empty text is the defaults, and a key the page gives
replaces the default there, an object key by key; a value that is not
an object where the defaults have one is refused. The calls (exported
under these names; `goal` and `position` are numbers, integers from 0
to 2³² − 1, anything else refused):

| call | answers |
|---|---|
| `about()` | `{"linlog", "level", "stack_bytes"}`: the library's version, the highest wire level it reads |
| `defaults()` | the tab's settings, for the Settings pane |
| `read(input, settings)` | a typed sequent, a sequent's JSON (`seq json`) or a session (`save`): `{"kind": "sequent" or "session", "sequent", "text", "fragment": {"classical", "intuitionistic"}, "mode", "session"}`; a parse error carries the caret's place, a level above the module's is `unsupported_version` |
| `start(sequent, mode, settings)` | a new session: `{"session", "view"}`; `mode` one of `classical`, `affine`, `mix`, `affine-mix`, `intuitionistic`, `intuitionistic-affine` |
| `view(session, settings)` | `{"mode", "fragment", "complete", "steps", "goals": [{"goal", "formulas": [{"text", "occurrence", "side", "reading"}]}], "inferences", "net"}`: `reading`, in intuitionistic mode alone, the formula as the reading writes it in its position (a hypothesis `A ⊸ B`, whose `text` is `A ⊗ ~B`); `inferences[n]` is the goal the drawing's `i<n>` is, `net` the refusal that says why the sequent has no net, or `null` |
| `rules(session, goal, position, settings)` | `[{"rule", "split"}]`: the rules the formula takes by its connective and the mode |
| `split_passes(session, goal, step, settings)` | `false` when the split cannot close |
| `apply(session, goal, step, settings)` | `{"session", "view", "opened"}`, or the refusal with its reason; a step is `{"position", "rule", "left"}`, `left` the positions going to the left premise of a `⊗` or Mix |
| `undo(session, settings)` | `{"session", "view", "reopened"}` |
| `close(session, goal, settings)` | `{"session", "view", "outcome", "refused", "refutation_checked"}`: the search within `limits.work`, its proof grafted; `refused` the graft's refusal |
| `fill(session, goal, source, settings)` | `{"session", "view", "filled"}`: the checked proof of `source`, another finished session of an equal sequent, carried over to the goal (its formulas matched up to order) and grafted, checked in this session's mode; `sequent_mismatch` with `details.proof` and `details.goal` (the formulas only on each side), `open_goals` for an unfinished source, `invalid_proof` for a rule this mode does not allow |
| `close_all(session, settings)` | `{"session", "view", "results": [{"goal", "outcome", "refused", "error", "work"}]}`: every goal's search within one budget of `limits.work` for all of them, a goal it runs out on `work_limit` |
| `hint(session, goal, settings)` | `{"outcome", "steps"}`: the search on the goal, the session unchanged, and the first steps of its proof (the structural rules at its root and the first rule that is not one), each for `apply` on the goal the one before opened; following the hints proves the goal |
| `proof(session, settings)` | the checked proof of a finished session |
| `prove(sequent, mode, settings)` | `{"outcome", "refutation_checked"}`: the verdict for the status light, `refutation_checked` what `Disproof::check` said |
| `draw(session, settings)` | the session's derivation as SVG, open goals as leaves, `i<n>` and `i<n>-<p>` on every conclusion and formula |
| `draw_net(session, settings)` | the net of a finished session's proof as SVG (`styles.svg.essential` for an essential net) |
| `export_as(session, format, settings)` (`export` is reserved in JavaScript) | `text`, `latex`, `typst`, `svg` (the session's tree, open goals included) or `rocq` (the certificate of the finished proof) |
| `render(svg, format, font, settings)` | `png` or `pdf` bytes of a drawing, rendered with the font's bytes the page passes; a PDF needs `styles.pdf.date` from the page's clock |
| `certificate(document, settings)` | the Rocq certificate of non-derivability of a disproof, or of an unprovable outcome |

The session's JSON itself is the session export (D16) and what the link
carries (D20).

**The tab's defaults** (`web/src/tab.rs`, `tab::defaults()`), the
command's but for:

| key | tab | command | why |
|---|---|---|---|
| `limits.memory_bytes` | 96 MiB | 1 GiB | a tab on iOS gets about 256 MiB of wasm memory; the rest holds what the bound does not count |
| `limits.occurrences` | 1 000 000 | 50 000 000 | a forest of 25 MB, copied by a session, a proof and a net |
| `limits.derivation_bytes` | 4 MiB | 64 MiB | a drawing peaks at several times its estimate |
| `limits.work` | 2²⁶ | none | the page's budget, about a second (68 million units a second under wasm): no clock in the module |
| `limits.recursion_depth` | 384 | 2 048 | the host's stack, not the module's, runs out (below) |
| `search.copies` | none | none | the work bound ends the deepening |
| `search.jobs` | 1 | auto | one thread |
| `search.classical_work` | 2²² | 2²⁶ | within a tab's budget |
| `clock` | no limits | 2 s | the page applies a time by stopping its worker |
| `styles.svg.ids`, `.description` | on | off | the page's buttons over the formulas, the screen reader's description |

## What the library lacks for the page

Each item of `plan/34-web.md`'s "What the bindings must offer the page",
checked against the library on 2026-10-19:

| the page needs | offered by | missing, and where it goes |
|---|---|---|
| the rules a formula takes, `split_passes` | `rules`, `split_passes` (`Interactive::rules`, `split_passes`) | nothing |
| each refusal's reason as a value | `apply`'s thrown error: `StepError`'s JSON in `details`, by `kind` with its positions | nothing |
| a search on a copy of the goal for the hint ladder | `hint` | the library has no "first steps of a proof of a goal"; the bindings read them off the derivation of the goal's proof (`calls::first_steps`); a library call would serve other front ends, a later addition to `Interactive` |
| a goal's close and all goals' within a work budget | `close`, `close_all` (`limits.work`, one budget for all) | nothing |
| the verdicts as `Outcome` and `Refutation` with `Disproof::check` | `prove`, `close` (`refutation_checked`) | nothing |
| every export: SVG, PNG and PDF from the page's font, LaTeX, Typst, Rocq, the certificates of non-derivability, the session as JSON | `draw`, `draw_net`, `export`, `render`, `certificate`, the session document | the certificate of non-validity of ordinary logic (`rocq::write_not_valid`) waits for ordinary logic on the page |
| the session document read back at its level, a sequent as `seq json` writes it | `read`, every call's `session` | nothing |
| the detected fragment | `read`, `view` | nothing |
| the drawing's `Style` with the theme's colours | `styles.svg` in the settings (`text`, `line`, `par`, `link`, `highlight`, `background`) | nothing |
| a step and a rule offered, as JSON | the bindings' own forms, `{"position", "rule", "left"}` and `{"rule", "split"}` | `Step` and `Applicable` have no wire form in the library; when one comes it is at level 1, and the bindings take it |
| ordinary logic (D2, D57, after version 1) | none | the JSON forms of `ordinary::Outcome`, the ordinary sequent and its derivation, at level 1 |
| the two-sided presentation of a classical derivation (D58) | none | a presentation pass over `Derivation` (`plan/later.md`), after step 32 |
| the net editor (D4, after version 1) | none | the bindings' calls over `ProofStructure::link` and `unlink`; nothing in the library |

## Item 3: the two searches of the default bias under wasm

See `plan/reports/34-web.md`, "Session 32a", for the measurement and
its answer.
