# Step 38: the web client's polish

You are working on linlog's web client, in the client's repository
(`/home/tux/Projects/own/linlog-prover.github.io`, served at
https://linlog-prover.github.io/), whose CLAUDE.md and `.claude/rules/`
apply throughout; in this repository, CLAUDE.md applies to anything you
read or change. The step takes one session; this prompt was written on
2026-10-19 from the author's requests of that day, after the site went
live. It says what is fixed.

Read `plan/later.md` ("The web client's polish", the list this prompt
turns into requirements, and "Follow-ups: the web client"),
`plan/web/design.md` (D1 to D60, fixed), the mockup
(`plan/web/mockup/` and its README), `plan/web/README.md`, and the
reports `plan/reports/34-web.md` (the client's three sessions) and
`plan/reports/36-net-drawings.md` (the nets with their formulas, and
step 36b's open points for the page).

## Goal

The page looks finished and modern, is consistent everywhere, is easy
to use on a phone as on a desktop, shows which engine answered and
where it comes from, and offers many more examples to practise on,
without crowding what an expert sees (D1, D6, D24).

## What is fixed

Use both plugins vetted for the design session throughout (the author,
2026-10-19): the `frontend-design:frontend-design` skill for the look,
the consistency and the controls, and the `education-agent-skills`
skills for the practice set, the progression, where the engines'
references sit in Learn, and ease of use for a student (at least
`practice-problem-sequence-designer`, `learning-progression-builder`,
`interleaving-unit-planner`, `cognitive-load-analyser` and
`udl-options-designer`, whichever fit). Each requirement below is the
author's:

1. **Touch targets**: every control measured at phone and desktop
   widths by Playwright, 44 by 44 pixels on a phone, never below WCAG
   2.2's 24 anywhere, with a test that fails below the floor; nothing
   needs precise tapping or a drag.
2. **Spacing**: one spacing scale in the tokens, used everywhere, no
   one-off value; wrong or ugly spacing found by a screenshot pass at
   both widths in both themes and fixed.
3. **Visual consistency** (D48): one visual grammar; the same component
   looks and behaves alike everywhere (sizes, states for hover, focus,
   pressed and disabled, radii, type scale, spacing), with a check that
   fails on a raw value outside the tokens.
4. **Visual polish**: a more modern look and feel within the chosen
   "Instrument" direction and the design's decisions; no control moves
   from where the design put it unless a requirement here asks.
5. **Ease of use**: the common tasks (prove a sequent, undo, close a
   goal, ask for a hint, export, open a practice example, switch the
   calculus) in as few steps as possible, with affordances clear before
   a press, walked through at both widths, and what is awkward fixed.
6. **The two-sided toggle only in classical mode**: shown for classical
   logic, hidden (not greyed) in intuitionistic mode, whose derivations
   are two-sided already.
7. **Which engine ran, and its reference**: the engine of every search
   the page ran (a goal's close, all goals', a hint, the verdict) shown
   beside the work it did, and each engine's source in the literature
   one gesture away (a hover, and Learn's reference): the focused
   search, proof-net search and its criterion, essential nets, the
   planar nets of the ordered modes, the Horn engine, the inverse
   method, the additive path and the classical refuter. Every reference
   is checked against its source by reading it, never written from
   memory, with a link where a stable one exists.
8. **Choosing the engine by hand**, only if it adds no clutter (the
   author is unsure): one choice in Settings, "Engine", default
   "Automatic", listing the engines that take the current proof's
   sequent and the others greyed with their one-line reason, a forced
   engine's refusal said at the goal, nothing on the stage or in the
   bands. If it still crowds the page, leave it out and say why.
9. **Practice**: many more examples, along D60's progression, each with
   a difficulty, provable and unprovable mixed, interleaved within a
   lesson's practice; every sequent decided by linlog's command before
   it is used, each provable one with a saved session to watch; the
   written content under CC BY 4.0 (D23).
10. **Other nitpicks of the same kind**: a sweep of every control,
    label and text for what does not apply where it is shown, says the
    same thing two ways, or surprises. Fix what is clearly wrong;
    collect the judgement calls for the author. Known already: the
    practice cards write negation as `~A` while the drawings write
    `A⊥`; the essential net's green input side beside D26's "green only
    for a checked proof"; Enter in the sequent field dropped silently
    while another action runs.
11. **linlog's latest**: the client's linlog input moved to `main` as
    it stands when the session starts (the inverse method's turns, the
    saturation refutations at wire level 3, which the page shows as it
    shows the other checked refutations).

## How the step runs

One session on Opus 5.5 at `high`, its goal in its launch prompt. A
fresh-context reviewer on Opus 5.5 at `high` judges the before-and-
after screenshots (desktop and phone, light and dark) against the
requirements above and reports only what breaks one. The author judges
the look: the session ends with the screenshots' paths in its message,
and every judgement call of item 10 in one list. The client's tests
(Vitest and Playwright) grow by one test per requirement that a test
can see.

## The machine

No other step runs beside this one unless the planning session says so:
builds, npm and Playwright in capped scopes on the cores the brief
names; `nix flake check` and `nix build` under
`/tmp/linlog-machine.lock`; Chromium from the devshell with `env -u
LD_LIBRARY_PATH`.

Deliverable: thematic jj commits in the client's repository; in this
repository `plan/reports/38-web-polish.md`.

## Session 38e: the author's decisions of 2026-10-23

One more session of this step, on Opus 5.5 at `high`, in the client's
repository alone (no change of linlog), its goal in its launch prompt;
it may run beside step 39's first session, which touches no file of the
client. The author decided each item below from the judgement calls of
sessions 38 to 38c; the reports of those sessions say what each one is.

1. **A privacy page** under the Swiss revFADP, worded so that it also
   meets the GDPR's information duty: the person responsible, Fabian
   Lukas Grubmüller, reached at privacy@grubmueller.dev; no Impressum.
   It says what `plan/later.md` lists ("Follow-ups: the web client",
   the privacy notice): no cookies, no tracking, nothing loaded from third parties
   (the CSP's `default-src 'none'`); GitHub's logs of visitors' IP
   addresses for hosting and security, in the US, and the safeguard
   (the Swiss-U.S. Data Privacy Framework: check GitHub's own entry on
   the framework's list and say what it shows); what the browser keeps
   (`linlog-settings`, the theme, autosaved tabs, the service worker's
   cache and the optional fonts) and how to clear it; the visitors'
   rights. Every legal statement read in its source (Fedlex, the FDPIC,
   GitHub's privacy statement), never from memory; linked from About and
   the footer; About's one line on GitHub's logs points to it.
2. **The calculus's switches on the tab's chip alone**: the desktop
   stage's cluster goes, so the chip is the one place at every width.
3. **A forced engine lasts for the visit**: the page starts on
   "Automatic"; every other setting is still kept.
4. **Colours a colour-blind visitor can tell apart** (the author's
   rule): every colour that carries a meaning (the verdicts, the nets'
   colours, the essential net's sides, the split's state, the focus)
   stays distinct under simulated deuteranopia, protanopia and
   tritanopia (a published simulation, such as Machado, Oliveira and
   Fernandes 2009, read in its source), checked by a test over the
   tokens; the nets keep the Okabe–Ito palette they use; and no meaning
   rests on colour alone (a word, a sign or a shape beside it).
5. **The essential net's input side** leaves green, which D26 keeps for
   a checked proof, for a colour of that palette.
6. **The status popover** writes the mode in the switches' plain words
   ("Intuitionistic, affine"), not linlog's `intuitionistic-affine`.
7. **The dock's counter** reads "Goal 2 · 1 open".
8. **The additive path's reference** claims nothing a source does not
   state: Whitman's free lattices are named as related, or not at all.
9. **"Fill from …"** names the source tab by its sequent, shortened, not
   by its place in the row, which changes as tabs close.
10. **A stale comment**: `src/islands/BottomBand.tsx`'s header still says
    that More's sheet holds the calculus's switches, which left it at
    38c; it says what the sheet holds.

What the author accepted as built is listed in `plan/later.md`, "Kept as
built: the web client's choices, for the author to revisit"; keep that
list true where this session changes one of them (the chip becomes the
one place for the mode).

Everything else the judgement calls of sessions 38 to 38c kept stays as
built. Before-and-after screenshots of each change at a phone's and a
desktop's width in both themes; a test per item a test can see; the
client's `nix flake check` at its last commit; the report as a section
of `plan/reports/38b-zoom-fonts-dismiss.md`.

The machine, beside step 39's first session: this session's builds,
npm and Playwright on cores 8 to 11 in capped scopes, step 39's on 2 to
7; `nix flake check` and `nix build` under `flock
/tmp/linlog-machine.lock`.

## Sessions 38f and 38g: a logo for linlog (the author, 2026-10-23)

The author asked for a modern logo or word mark for linlog, shown
wherever linlog appears. Two sessions, after 38e (both touch the
client):

**38f, the design** (Fable 5.1 at `high`, the design session's model in
the plan's table, as step 34's mockups were; the
`frontend-design:frontend-design` skill). Four or five candidates, each a
mark and a word mark, in the spirit of the page's "Instrument"
direction and today's icon (`public/icon.svg` in the client: an
inference, two premises over a bar, the turnstile below), drawn from
what linear logic writes (`⊸`, `⊗`, `⅋`, the turnstile, a proof net's
links) and never from another project's mark. Each candidate comes as a
family, one spirit in two shapes (the author, 2026-10-23): the full word
mark for the wide places (the header, the README's head, the rustdoc,
link previews), and a compact square form for the square ones (the
favicon, the app icons, the Apple touch icon, GitHub's avatar), which
may differ from the word mark (the mark alone, a monogram, a simpler
drawing) as long as the two are plainly one logo. Each is SVG,
its text outlined so that no font file goes with it, legible as a
16-pixel favicon and as a 512-pixel icon, in a light and a dark version
and in one colour, its colours told apart by colour-blind viewers (the
author's rule, session 38e). They are shown to the author side by side,
at every size they will be used, in one page (published privately as an
artifact where the tool is there, or written to a file whose path the
session gives), and the author chooses; nothing is placed before that.
The session proposes the logo's licence (the code is EUPL-1.2, the
written content CC BY 4.0) for the author to decide.

**38g, the logo everywhere** (Opus 5.5 at `high`), from the chosen
SVG, every derived size made by a script of the client's flake from
it, never by hand:
- **the client**: the favicon (SVG, with the PNG fallbacks browsers
  ask for), the web app manifest's icons (maskable included), the Apple
  touch icon, the header where a plain "linlog" stands now, the 404 page
  and About, and an Open Graph image for link previews, within the CSP
  and the licences' check;
- **linlog**: the README's head (an image by absolute URL from the
  site, which crates.io renders too, since it has no logo of its own),
  the rustdoc's logo and favicon (`#![doc(html_logo_url = …,
  html_favicon_url = …)]` on the core crate, by the site's URL, so the
  Pages documentation and docs.rs show it), and the Rocq library's
  README;
- **what only the author can do**, prepared as files with a checklist in
  the report: GitHub's organisation avatar and each repository's social
  preview (1 280 by 640) are set in GitHub's web interface alone, and a
  Zenodo community with the logo, if the author wants one (the DOI
  record archives each release, the logo with it). No repository
  setting is changed by the session.

Tests where a test can see it (the header and the icons served, the
manifest's sizes, the CSP); screenshots before and after at both widths
in both themes; the reports as sections of
`plan/reports/38b-zoom-fonts-dismiss.md`.

## Session 38h: invertible rules applied automatically (the author, 2026-10-23)

The author asked for a toggle, off by default with an explanatory
tooltip, that applies the invertible rules by itself, as focusing's
asynchronous phase does and as Click & coLLecT's "auto-reverse" option
does (its page, read 2026-10-23: "applies systematically reversible
rules to generated sequents", the introductions of `⅋`, `&`, `⊥`, `⊤`,
`!` and `1`, and also the axiom on atoms and `⊗` with an empty context).
One session, Opus 5.5 at `high`, in linlog and then the client, after
step 39b's push and before step 40 (both touch the interactive layer),
its goal in its launch prompt:

1. **In the library** (`Interactive`): one call that applies to a goal,
   or to every open goal, each rule that is invertible in the goal's
   mode until none applies, every application an ordinary step of the
   state, so that the completed proof is checked as any other (D13).
   Which rules are invertible in each mode (classical, intuitionistic
   two-sided, affine, with Mix, the ordered modes) is read in the
   sources (Andreoli 1992 for the classical case; the intuitionistic
   and affine ones from their own sources), never from memory, and
   written in the rules file; where a rule is invertible only under a
   condition (`!` when every other formula is a `?`, `1` alone, the
   axiom, `⊗` with an empty context), the condition is checked. A test
   that it never turns a provable goal into an unprovable one (the
   search decides the goal before and the goals after, on generated
   sequents of every mode), and that it ends.
2. **One options value** (D15, D16): off by default, with which rules
   it applies if there is more than one sensible set; `linlog interact`
   gains a flag and a command for it; the bindings take it as a
   setting; the JSON state is unchanged or raised as `wire.rs` says.
3. **On the page**: a toggle where the page keeps its proving
   behaviours (the session's choice with `frontend-design`, within
   step 38's grammar), off by default, kept with the other settings,
   its tooltip saying in two sentences what an invertible rule is and
   why applying it never loses a proof; undo takes back the automatic
   steps in a way that does not surprise (the session decides, and says
   why); the hint ladder's first rung (D12) takes the toggle into
   account; a test per behaviour; before-and-after screenshots at both
   widths in both themes.

A fresh-context reviewer on Opus 5.5 at `high` reads the invertible
sets against their sources and the tests, reporting only what breaks a
stated requirement. linlog's part first, as 38b's was: the supervisor
reviews and pushes it, then the client's lock moves to it. The report
as a section of `plan/reports/38b-zoom-fonts-dismiss.md`.

