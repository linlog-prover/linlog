# Step 38b: native zoom, the exports' fonts, light dismiss

## The checklist

| item | state | evidence |
|---|---|---|
| 1. Native zoom: a pinch and ctrl with the wheel zoom the drawing on the stage; nothing blocks the browser's zoom elsewhere | done | the client's `wvzrmkqt`; `tests/zoom.spec.ts`, two tests (desktop: ctrl with the wheel about the pointer and the check of every element outside the stage; touch screen: a two-finger pinch about the fingers, a one-finger pan, and the browser's own pinch on the top band zooming the page), passing |
| 2. A font for the rendered exports: an option of linlog (serde, a CLI flag, the bindings' settings), Euler Math the default and unchanged, STIX Two Math and two more, each table read from its file, each licence served | done | linlog's `tvrwrqsp` and `zupynzoz`; `every_table_is_its_font_s` (skrifa reads each file), `fonts` and `every_style_option` in `cli/tests/cli.rs`, `a_font_is_chosen_by_its_name` and `an_export_is_set_in_the_font_the_settings_name` in the bindings; the snapshots and the behaviour lock unchanged but for `--help`; the `export` check, once per font, passed (exit 0) |
| 2'. The font chosen in the client's export, with a test; one PNG and one PDF per font | done | the client's `wrlrwoum`; Playwright `the export menu chooses the font of the SVG, PNG and PDF, …`; `/home/tux/Projects/own/linlog-step38b-screenshots/exports/{euler,stix-two,garamond,asana}.{png,pdf}`, each PDF embedding its font alone (`pdffonts`) |
| 3. Light dismiss at both widths | done | the client's `xqyzqkpw`; `tests/dismiss.spec.ts`, a desktop test and a phone test, passing |
| 4. More complex provable practice problems (the supervisor's addition) | done | the client's `wpspsroq`: 21 problems, 14 provable at difficulty 3, two per lesson from the second, each with a near miss that is not provable; `practice-decided.txt` (21 `linlog prove` outputs, 14 `linlog interact` sessions with `proof` exit 0); Vitest `give every lesson from the second at least two provable problems at the hardest difficulty` and the per-example verdict and session tests |
| 5. The tab chip's spacing (the supervisor's addition) | done | the client's `vlktpknq`; the screenshots `01-chip` and `01-chip-closeup` at both widths in both themes |
| Before-and-after screenshots; the reviewer's findings answered | see below | `/home/tux/Projects/own/linlog-step38b-screenshots/before/` and `after/` (11 scenes at two widths in two themes, the chip close-ups); "The review" |
| linlog's part gated, its `nix flake check` passed, "linlog ready" sent | see "Verification" | |
| The client's lock on the commit the supervisor answered with; its `nix flake check` at its last commit | see "Verification" | |

## Outcome

On the stage a pinch of two fingers zooms the drawing about the fingers'
midpoint and one finger still pans it; ctrl with the wheel, which is also
how Chromium and Firefox send a trackpad's pinch, zooms about the pointer,
and Safari's trackpad pinch (its gesture events) does the same. Outside the
stage nothing keeps the browser from zooming the page: no `touch-action`,
no listener that cancels ctrl with the wheel or ctrl with plus or minus,
and a viewport that allows zoom (WCAG 2.2, 1.4.4). Ctrl with plus and minus
is always the browser's page zoom; plus and minus alone zoom the drawing,
as before.

linlog's drawings can be set in four math fonts: Euler Math, still the
default and byte for byte unchanged, STIX Two Math, Garamond-Math and Asana
Math. The option is `styles.svg.font`, which reads a font's name; the
command takes `--font`, and the bindings take the name in their settings.
Each font's advance table was read from its file and is checked against
it by a test, every character a drawing sets has a glyph in every font,
and the export check renders the command's drawings in each font with that
font alone. The page's export menu chooses the font of the SVG, PNG and
PDF; the page itself draws in Euler Math.

A press outside an open menu, pane, sheet or popover closes it, as Escape
does. A press on the stage that closes something does nothing else; a
press on another of the bands' buttons closes and acts.

Practice has 21 more problems, 14 of them provable at the hardest
difficulty, two per lesson from the second, each beside a near miss. The
tab's calculus mark stands apart from the sequent.

## Decisions

- **The fonts** (requirement 2): STIX Two Math 2.12, Garamond-Math
  2022-01-03 and Asana Math 0.962, all under the SIL Open Font License
  1.1. Every candidate among TeX Live's OpenType math fonts was checked
  against the 208 characters the drawings set (fontTools, `getBestCmap`):
  Latin Modern Math and the five TeX Gyre math fonts lack `⅋`, Libertinus
  Math `⅋` and `⊸`, Fira Math `⅋ ⊢ ⊤ ⊥ ⊸ ⋮`, so none of them can set a
  drawing of linear logic. Of those that cover every character, New
  Computer Modern Math (the Computer Modern look, Typst's default) is under
  the GUST Font License, which is the LPPL 1.3c with a request; its clause
  2, read in the LPPL's text, counts distributing part of the Work (one
  file of the package) as modification, which the OFL does not, so it was
  left out; XITS Math is STIX's predecessor in the same design. STIX Two
  (Times), Garamond-Math (Garamond) and Asana Math (Palatino) are the most
  used OFL math fonts that remain, and three different faces beside
  Euler's upright one. Each licence was read in its source: STIX Two's
  `OFL.txt` (copyright "2001-2021 The STIX Fonts Project Authors", Reserved
  Font Name "TM Math" for MicroPress's portions), Garamond-Math's CTAN
  README ("licensed under the SIL Open Font License, Version 1.1") and
  name table (copyright "2018-2022 Yuansheng Zhao and Xiangdong Zeng", no
  Reserved Font Name), Asana Math's README (copyright and the Reserved
  Font Names "Asana Math", "pxfonts", "gsmn1000"). Serving the unmodified
  files under their own names is allowed, and the OFL-FAQ (1.12) allows
  embedding a subset in a document, which is what a PDF does.
- **Where the files come from and how they are pinned**: CTAN's packages as
  nixpkgs' TeX Live has them at linlog's locked nixpkgs
  (`texlivePackages.stix2-otf`, `garamond-math`, `asana-math`), copied
  unmodified into `cli/fonts/` beside Euler Math (whose committed file is
  byte for byte TeX Live's too), each with a licence file: STIX Two's own
  `OFL.txt`, and for the others the OFL's text with the copyright lines the
  font's source states, each under a line saying which font and version it
  covers. The sha256 sums: Euler-Math.otf `d6faf2b5…`,
  STIXTwoMath-Regular.otf `95bc2729…`, Garamond-Math.otf `c6161377…`,
  Asana-Math.otf `9fa3bd72…`. The export check renders with nixpkgs' copies;
  `every_table_is_its_font_s` reads the committed ones.
- **How the licences are served**: the command's package installs every
  `cli/fonts/*.txt` under `share/doc/linlog-cli/`; README's licence section
  names the four fonts and their files; the client takes every file of
  `cli/fonts/` through its `linlog` input, serves each font under
  `/fonts/` beside `<Family>-OFL.txt`, and its `licences` check compares
  each served licence with linlog's file byte for byte; About lists them.
- **The option, for every front end** (D15): `svg::Style::font`, a
  `svg::Font` (family and advances, as before), which now also reads in
  JSON as the name of a `svg::MathFont` (`"euler"`, `"stix-two"`,
  `"garamond"`, `"asana"`); `MathFont::font` is the preset, and
  `svg::Advances` gains the variants `StixTwo`, `Garamond` and `Asana`.
  The command sets it with `--font NAME` (`FontArg`), or `--style
  svg.font=NAME`, or a style file's `"svg": {"font": "NAME"}`; the
  bindings take `{"styles": {"svg": {"font": "NAME"}}}` in any call's
  settings, and `render` takes the bytes of that font's file; a front end
  that keeps settings as JSON writes the same. The default is Euler Math,
  so no output changes (the snapshots and the behaviour lock pass; the
  lock's `--help` files were blessed for the new flag, a commit of its
  own). No quantifier bears on a font (D17).
- **A name rather than a pair of fields**: a font is laid out by its
  advances and rendered by its family name, and setting one without the
  other would give a drawing whose widths are another font's; the name
  sets both. `Font` always writes its fields, so a front end that reads
  the settings back sees them.
- **The bindings' settings take a string where the tab has an object**:
  the overlay refused anything but an object over the tab's font object,
  which would have refused the name; it refuses an array there still (the
  reason it refused, since serde reads an array as a struct), and serde
  refuses a string that names nothing.
- **The command renders with Euler Math and the font the style names**,
  not all four: a render loads what it loaded before for the default
  style, and the memory estimate's base (32 MiB) covers at most two font
  files of 0.4 and 0.8 MB.
- **The vertical metrics stay Euler Math's** (`HEIGHT`, `DEPTH`, `AXIS`,
  `SCRIPT`, `RAISE`, `LOWER`), as `font.rs` said: measured in each file,
  the raised `⊥` reaches 875, 752 and 805 thousandths under `HEIGHT` 890,
  every comma lies within `DEPTH` 210, the math axis is 258, 250 and 271
  against 250 (a fifth of a pixel at 16 px), and the deepest descender of
  the table (235, 290, 286; Euler's own `q` 243) stays inside `DEPTH`
  plus the drawing's margin. Making them a font's would have changed
  Euler Math's output.
- **A missing glyph** (only a character outside the table can be one,
  such as an atom named in another script): the SVG gives it the
  fallback advance of 650 and `textLength` keeps the layout whatever
  font the viewer sets it in; in PNG and PDF usvg sets it in another
  loaded font that has it, with a warning, or draws the font's `.notdef`.
  No character of the table is missing in any of the four fonts (the
  test).
- **The table's layout**: the characters and the four advance tables are
  kept apart (`CHARACTERS`, `ADVANCES`), because rows of a character and
  four advances made the binary search walk wider entries and the
  `render-svg` journey counted 2.1 % more, past the ratchet; with the
  split it counts 0.44 % more than its ceiling.
- **Zoom** (requirement 1): the stage keeps `touch-action: none`, so the
  browser makes no gesture there and the stage reads its pointers: a
  second finger makes a pinch wherever it lands, a formula's button too,
  since on a phone the buttons cover much of the drawing. The page's zoom
  is the browser's everywhere else, and ctrl with plus or minus is the
  browser's everywhere. The drawing is zoomed about the gesture's centre.
- **Light dismiss** (requirement 3): one capture handler in `App.tsx`.
  What counts as inside: the open menu or pane and its own button (every
  toggle carries `aria-expanded`; the calculus mark now does), the
  restart warning with the calculus's cluster, and for the rule popover
  and the split chooser every formula's button, by which a rule or a side
  is chosen. A press on the stage that closes something is swallowed (no
  pan begins, no formula is picked, no stage button acts): the visitor
  aimed at the open thing, and a formula picked by the same press would
  surprise. A press on a band's button closes the open menu and acts (a
  second menu opens, Undo undoes), as menus of desktop programs do.
- **The font is chosen in the export menu**, four radio items under the
  formats, kept in the settings: it applies to three of the menu's
  formats, so it sits where they are; the menu stays one gesture away.
- **Practice** (requirement 4): two provable problems at difficulty 3 per
  lesson from 2 to 8 (MLL associativity of four, implications side by
  side, a proof by cases, two implications under `&`, `0` absorbing, the
  three units together, Seely's isomorphism both ways, exchange of
  arguments, `!` carrying a chain, two Petri nets, weak distributivity,
  two implications in parallel), each beside a near miss that is not
  provable (crossed wires, `&` not distributing, `⊤` not giving `1`, `!A &
  !B` not promotable, a higher-order argument, a one-to-one machine, the
  converse of weak distributivity). The near misses keep the lessons'
  rule of never three alike in a row, which two more provable problems at
  a lesson's end would break. Lesson 1 (the axiom and negation) has no
  harder provable problem that is not a later lesson's.

## Judgement calls

1. The near misses (seven unprovable problems) were added beside the
   fourteen provable ones the supervisor asked for, to keep each lesson
   mixed; the explanations are this session's, written from linlog's
   reasons.
2. Learn's frame and the tour are not light-dismissed: Learn is a mode the
   visitor works beside (practice's Prove opens a proof on the stage with
   Learn open), and the tour's caption is a guided sequence that Escape
   ends.
3. On a phone, Settings, About and the long menus cover the stage; outside
   is then the strip above them or the bands, which a tap there closes.
4. The exports' three other fonts are cached by the service worker with
   every other file (about 2 MB on the first visit), so exports work
   offline as the rest of the page does.
5. The new examples' `order` is 100 + 10 · lesson + k, after each lesson's
   earlier ones, so that no existing example was renumbered.
6. The chip: 12 pixels (the scale's 3) before the mark and 8 (2) between
   its calculus and its fragment, up from 4 and 4; the phone's tab still
   fits its row (the screenshots).
7. Safari's gesture events are handled but untested: the tests run
   Chromium only.

## Departures from the design

- (departs from the design) **D25 and D16**: D25 has Euler Math wherever
  linlog draws, and D16 has PNG and PDF take the font bytes the page
  already loads. The exports may now be set in three other fonts, by the
  author's request, and their bytes are fetched when an export first
  needs them; the page's own drawing stays in Euler Math (D53).

## The review

One fresh-context reviewer (Opus 5.5, `high`), briefed with the
requirements F1 to F5 (the fonts' option, tables, licences, the page's
drawing) and S1, S2 (the screenshots and the exports), conduct.md's rules
on cores and memory (cores 12 to 15, no builds), and told to report only
what breaks one, read the diff, the font files (fontTools), the
licences, the client and the screenshots; one round. It reported four
findings and found F1, F3, F4, F5 and S2 holding, and F2 and S1
otherwise:

1. Phone scene 04 still showed the Settings sheet open: the screenshot
   script tapped at 30 % of the stage's height, inside the sheet, which
   on a phone covers the stage but for a strip. **Holds** (of the
   script, not the page: `tests/dismiss.spec.ts` taps the strip and
   passed). `shots.mjs` now taps above whatever is open, or the top
   band's empty part where a sheet covers the stage; the pressed-outside
   scenes (04, 06, 08, 09) were taken again in both sets, and phone 04
   shows the sheet closed.
2. The desktop pinch pair shows no difference. **Does not hold**: on a
   desktop the pinch is ctrl with the wheel (a trackpad's pinch), which
   zoomed the drawing before this step too; the change is the phone's,
   whose pair shows it (before unchanged, after zoomed in).
3. In the phone's More sheet a tip covers the About row. **Does not
   hold** for this step: the same in `before/`, the tip of the screenshot
   script's emulated mouse (step 38's finding 13; a touch raises no tip).
4. A net's premise edge under a written formula started at `DEPTH` (210
   thousandths below the baseline), inside the descenders of
   Garamond-Math (290) and Asana Math (286), against Euler Math's 243.
   **Holds**: linlog's `qzxqzvqk` starts the edge at the font's own depth
   (`Advances::depth`, the deepest descender of the table in each file),
   Euler Math keeping 210 so that its drawings stay byte for byte; the
   test `net_edges_clear_the_font_s_descenders` pins all four. The
   compact drawing's edges start 100 below a literal in every font, as
   they always did (a follow-up, below).

## Verification

- linlog: `gate` passed (exit 0) at `zupynzoz` (a first run failed on
  the behaviour lock's `--help` files, blessed in a commit of its own,
  and a second on the ratchet's `render-svg`, +2.11 %, fixed at its
  cause by the table's layout, +0.44 % after); `nix flake check` passed
  (exit 0) at `zupynzoz` (43a2c90b) under the machine lock; the
  descender fix (`qzxqzvqk`) and this report gated and checked again,
  their exit statuses in the message to `linlog-ef`.
- The `export` check alone, before the gate: exit 0, its script running
  the per-font loop (`nix derivation show` of the check).
- `pdffonts` of the client's exports: each PDF embeds its font alone
  (`KJPMGG+Euler-Math`, `YFNLGA+STIXTwoMath-Regular`,
  `ARBTJL+Garamond-Math`, `FWCMYA+Asana-Math`).
- The client, built against this workspace (`--override-input linlog
  path:…`): Vitest 114 passed; `lint` (Biome and `astro check`) clean;
  Playwright 54 passed, among them `tests/zoom.spec.ts` (2),
  `tests/dismiss.spec.ts` (2) and the export font test. Two tests changed
  for behaviour changed on purpose: About lists 9 licences, not 6
  (`learn.spec.ts`).
- Chromium zooms the page from two dispatched touch points spread from
  20 to 100 pixels either side, not from 20 to 60, and CDP's
  `Input.synthesizePinchGesture` zoomed nothing here; the test uses the
  former (probed on the built site and on a bare page).
- Signing tested before every commit; every commit signed.

## What is left open

- Safari's trackpad pinch (gesture events) is written but not run in a
  test.
- A compact net's premise edges start 100 thousandths below a literal's
  baseline (`FOOT`), inside every font's descenders, Euler Math's too;
  a follow-up for the net drawing, not of this step.

## Repositories and workspaces

- linlog: `../linlog-step38b`, the commits `tvrwrqsp` (the fonts),
  `zupynzoz` (the lock blessed) and `qzxqzvqk` (the descenders), on
  `zxxqmxmx`, and this report's.
- The client: `/home/tux/Projects/own/linlog-prover.github.io`, its
  commits above `yxtvvlsm` (`main`), local, nothing pushed.
- Screenshots, exports and scripts:
  `/home/tux/Projects/own/linlog-step38b-screenshots/`.
