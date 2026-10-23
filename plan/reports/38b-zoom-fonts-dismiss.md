# Step 38b: native zoom, the exports' fonts, light dismiss

## The checklist

| item | state | evidence |
|---|---|---|
| 1. Native zoom: a pinch and ctrl with the wheel zoom the drawing on the stage; nothing blocks the browser's zoom elsewhere | done | the client's `wvzrmkqt`; `tests/zoom.spec.ts`, two tests (desktop: ctrl with the wheel about the pointer and the check of every element outside the stage; touch screen: a two-finger pinch about the fingers, a one-finger pan, and the browser's own pinch on the top band zooming the page), passing |
| 2. A font for the rendered exports: an option of linlog (serde, a CLI flag, the bindings' settings), Euler Math the default and unchanged, STIX Two Math and two more, each table read from its file, each licence served; New Computer Modern Math too (the supervisor's addition) | done | linlog's `tvrwrqsp`, `zupynzoz`, `qzxqzvqk` (the descenders), `ormuvkok` and `spxtytpy` (New Computer Modern and its lock); `every_table_is_its_font_s` (skrifa reads each file), `fonts` and `every_style_option` in `cli/tests/cli.rs`, `a_font_is_chosen_by_its_name` and `an_export_is_set_in_the_font_the_settings_name` in the bindings; the snapshots and the behaviour lock unchanged but for `--help`; the `export` check, once per font, passed (exit 0) |
| 2'. The font chosen in the client's export, with a test; one PNG and one PDF per font | done | the client's `wrlrwoum` and `okxuulsz`; Playwright `the export menu chooses the font of the SVG, PNG and PDF, …`; `/home/tux/Projects/own/linlog-step38b-screenshots/exports/{euler,stix-two,garamond,asana,new-computer-modern}.{png,pdf}`, each PDF embedding its font alone (`pdffonts`) |
| 3. Light dismiss at both widths | done | the client's `xqyzqkpw`; `tests/dismiss.spec.ts`, a desktop test and a phone test, passing |
| 4. More complex provable practice problems (the supervisor's addition) | done | the client's `wpspsroq`: 21 problems, 14 provable at difficulty 3, two per lesson from the second, each with a near miss that is not provable; `practice-decided.txt` (21 `linlog prove` outputs, 14 `linlog interact` sessions with `proof` exit 0); Vitest `give every lesson from the second at least two provable problems at the hardest difficulty` and the per-example verdict and session tests |
| 5. The tab chip's spacing (the supervisor's addition) | done | the client's `vlktpknq`; `tests/chip.spec.ts` (`xmoymlwt`): the gaps measured on the text (16 and 8 pixels at least), and the phone's tab and `+` inside the row, at both widths; it fails on `main`'s site (a gap of 10) and passes here; the screenshots `01-chip` and `01-chip-closeup` at both widths in both themes |
| 6. (none) | — | the supervisor's numbering has no item 6: New Computer Modern Math went under item 2, as asked |
| 7. Every copyright and copyleft obligation respected (the supervisor's addition; there is no item 6, New Computer Modern going under item 2 as asked) | done | "The licences", the table below; fixed: STIX Two's OFL text verbatim (`vurxrnok`), New Computer Modern's clause 6 notice, the client's check that every served font is linlog's file unmodified (`okxuulsz`), the creator named in the CC BY notice (`pusswryq`) |
| 8. The stage's font, a setting among the same five, Euler Math the default; the exports' font following it (the supervisor's addition) | done | the client's commit of item 8; Playwright `Settings sets the formulas' font: …`; `/home/tux/Projects/own/linlog-step38b-screenshots/stage-fonts/{desktop,phone}-light-stage-{euler,stix-two,garamond,asana,new-computer-modern}.png`; the targets test and the token check pass with it |
| 9. Moving work between tabs (the supervisor's addition): (a) a goal opened in a new tab, which remembers where it came from; (b) a goal filled from another tab's finished proof, linlog's new call carrying the proof over | done | linlog's `qsoqwtqp` (`Proof::onto`, `Error::SequentMismatch`, `Interactive::fill`, the bindings' `fill`; `fill_carries_a_proof_of_an_equal_sequent` and `a_goal_is_filled_from_another_session`, a refusal for each way it fails); the client's `zlwkpvtr`; Playwright `a goal opens in a new tab …` and `a goal is filled from another tab's finished proof …`; screenshots `11-goal-dock` to `14-filled`, before and after |
| Before-and-after screenshots; the reviewer's findings answered | see below | `/home/tux/Projects/own/linlog-step38b-screenshots/before/` and `after/` (11 scenes at two widths in two themes, the chip close-ups); "The review" |
| linlog's part gated, its `nix flake check` passed, "linlog ready" sent | done | "linlog ready" sent after gate and `nix flake check` exit 0 at `tzpvrrut`; the supervisor rebased the chain onto `zvktswqy`, added `wvxvwrwo` and `qmopnnvu`, and pushed `main` at `qmopnnvu` (bf80f7e4), gate and check passing there |
| The client's lock on the commit the supervisor answered with; its `nix flake check` at its last commit | done | `zvywrmku`, the lock's `linlog` at `bf80f7e48b1192f78b5fc7d58c14a2aef363f553`; the client's `nix flake check` passed (exit 0, "all checks passed!") at its last commit `xmoymlwt` (e507ebe7): Vitest 114 (`examples.test.ts` 76 among them), Playwright 61, the tests of items 1, 3, 5, 8, 9 (a) and 9 (b) and the fonts' among them |

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

linlog's drawings can be set in five math fonts: Euler Math, still the
default and byte for byte unchanged, STIX Two Math, Garamond-Math, Asana
Math and New Computer Modern Math. The option is `styles.svg.font`, which reads a font's name; the
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
  1.1, and, by the author's later request, New Computer Modern Math 7.1.1
  (below). Every candidate among TeX Live's OpenType math fonts was checked
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
  Euler's upright one. **New Computer Modern Math** came afterwards by the
  author's request, in its Book weight, the `newcomputermodern` package's
  default (its Regular weight shares the family name
  `NewComputerModernMath`, so a renderer gets the Book file alone). Its
  package ships no licence file: its README says "GustFLv1 or later" and
  the font's copyright notice points to
  http://tug.org/fonts/licenses/GUST-FONT-LICENSE.txt, which was read
  there (version 1.0 of 22 June 2009; TeX Live's `lm-math` carries the
  2006 preliminary text). The GUST Font License is the LPPL 1.3c or
  later with a request to rename derived fonts that is "not legally
  required"; the LPPL's clause 2 counts distributing part of the Work as
  modification, so linlog's `NewCMMath-Book-LICENSE.txt` meets clause 6:
  the file is unchanged (a, b: what changed is that the package's other
  files are left out), no support is implied (c), and where the complete
  Work is (d: CTAN), followed by the GUST text and the LPPL 1.3c (LaTeX's
  own `lppl.txt`), verbatim, since the LPPL forbids changing its text. No
  rule of either repository stands against it: `deny.toml`'s licence list
  governs crates, and its exception for `linlog-cli` now names
  `LPPL-1.3c` beside `OFL-1.1`; the client's rule against committed
  third-party files holds, the font coming from linlog's input. The
  command's package licence becomes `EUPL-1.2 AND OFL-1.1 AND LPPL-1.3c`,
  a field only crates.io validates (a valid SPDX expression, checked by
  hand). Each licence was read in its source: STIX Two's
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

- **The stage's font** (item 8, the author's request after the first
  seven): Settings has "Font of the formulas", five native radios in the
  grammar of the engine choice, saved in `linlog-settings` as
  `stageFont`. The stage's drawing asks linlog for it
  (`styles.svg.font`), so it is laid out in its advances; the page sets
  `--font-math` to it (`:root[data-math-font]`, a token per font), so
  every formula of the page follows: the formulas' buttons, the dock,
  the popovers, Learn's and the **tab's sequent too** (one font for the
  logic everywhere, as Euler Math was). The font's file is fetched and
  added to the document when it is chosen and not before; the service
  worker no longer caches the four other math fonts at install but when
  first fetched (`LATER`), so a visitor who never chooses one never
  downloads it, and one chosen once works offline. **The exports'
  font** is `font: MathFont | null`, null following the stage's; a pick
  in the export menu is kept (it is a setting like the others), and
  picking the stage's font there makes it follow the stage again. The
  sizes, all fit for a phone: Euler Math 438 KB and Asana Math 436 KB,
  Garamond-Math 824 KB and STIX Two Math 839 KB, New Computer Modern
  Math 1 316 KB, about 2.2, 4.1, 4.2 and 6.6 seconds at a slow 4G's 1.6
  Mbit/s, once; until it arrives the drawing is shown in a fallback
  serif at the right widths (`textLength`), so nothing waits on it.
  (departs from the design: D53, at the author's request)

- **Moving work between tabs** (item 9): **(a)** the dock's arrow ("Open
  this goal in a new tab", icon alone with its tip, as a browser's
  open-in-new-tab is conventional) opens the goal's formulas as the
  field reads them, in the same calculus, in a new tab whose `origin`
  is the tab and the goal's path, kept in the browser. **(b)** linlog's
  `Proof::onto(forest, goal)` carries a proof to a goal of another
  forest: every conclusion takes the first free goal member that is the
  same formula (preorder of kinds, atoms by name), and every occurrence
  below it the one at the same offset of that member's subtree; it
  proves nothing itself. `Interactive::fill` grafts the carried proof
  through `close_with`, so linlog's checker decides it in the goal's
  mode, as every graft. The refusals: `Error::SequentMismatch` (code
  `sequent_mismatch`, the formulas only the proof concludes and only the
  goal holds), `open_goals` for an unfinished source (the bindings'
  `fill` takes the source as a session and asks for its checked proof),
  `invalid_proof` for a proof of another mode's rules, and
  `IndexOutOfBounds` for a goal outside the forest; each is tested. The
  page offers "Fill from tab N" under the dock where another tab's
  proof is finished and its sequent has the goal's formulas in any
  order; a fill is one step, named "Proof from tab N" in the drawing and
  "The proof from tab N closed goal …" in the step list, undone by one
  Undo, and replayed from its own inferences, never by the search. A
  proof valid only in another calculus (a Mix proof offered to a
  classical goal) is refused at the goal: the note "Not filled: not a
  proof in this calculus", with the checker's reason as the page shows
  it ("invalid proof: node 2 (mix from 0, 1) with premises ⊢ A, ~A and ⊢
  B, ~B: the mode forbids the rule") and the line that its own tab's
  calculus allows a rule this one does not; the goal stays open
  (`after/desktop-light-15-fill-refused.png`). Done after (a), (b) brings the
  proof back to the goal it came from. Neither half is unsound: the
  carried proof passes the checker or is refused.

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
8. Tabs are named by their place in the row ("tab 2"), as the row shows
   them; a name kept with a fill ("Proof from tab 2") is the place the
   tab had then.
9. The offer to fill lists every finished tab of the goal's formulas, the
   one the goal was opened into first among them only by its place; the
   `origin` is kept but not shown.
10. The zoom floor: a step now zooms in to it (the targets test found a
    formula 43 pixels wide on a phone in Garamond-Math), unless the
    visitor has zoomed by hand since the last fit, which step 38 left as
    the visitor's choice.

## Departures from the design

- (departs from the design) **D25 and D16**: D25 has Euler Math wherever
  linlog draws, and D16 has PNG and PDF take the font bytes the page
  already loads. The exports may now be set in three other fonts, by the
  author's request, and their bytes are fetched when an export first
  needs them; the page's own drawing stays in Euler Math (D53).

## The licences (item 7)

What each licence asks of a redistributor, read in the text that ships
with the font (`cli/fonts/OFL.txt` for Euler Math, written from the
font's three copyright lines and its README's "licensed under the SIL
Open Font License, Version 1.1"; STIX Two's `OFL.txt`; Garamond-Math's
README and name table; Asana Math's README; New Computer Modern's
README, name table and the GUST and LPPL texts), and where each holds.
Every copy of a font is byte for byte the TeX Live file (`cmp` of each
`cli/fonts/*.otf` with nixpkgs' TeX Live, all five equal).

| licence | obligation | where it is met | evidence |
|---|---|---|---|
| OFL 1.1 (Euler Math, STIX Two Math, Garamond-Math, Asana Math) | condition 2: each copy contains the copyright notice and the licence | linlog's `cli/fonts/` (each `.otf` beside its `*-OFL.txt`: copyright lines and the OFL); the `linlog-cli` crate (`cargo package --list`: `fonts/*.otf` and every `fonts/*.txt`); the command's flake package and so the Cachix cache (`share/doc/linlog-cli/*.txt`, every `cli/fonts/*.txt`); the site (`/fonts/<Family>-OFL.txt` beside each font, byte for byte linlog's, the `licences` check) | `cargo package --list -p linlog-cli`; `modules/workspace.nix` `postInstall`; `licences.py` exit 0 on the build |
| OFL 1.1 | condition 1: not sold by itself | nothing is sold; the fonts are bundled with software (the command, the site), which condition 2 allows | the licence's text |
| OFL 1.1 | condition 3: no Modified Version uses a Reserved Font Name ("TM Math" for STIX Two's MicroPress portions; "Asana Math", "pxfonts", "gsmn1000" for Asana Math; none for Euler Math and Garamond-Math), "the primary font name as presented to the users" | every copy is unmodified: the committed files are TeX Live's, the crate and the binary carry them as they are (`include_bytes!`), and the site serves linlog's files byte for byte, which the client's `licences` check now compares (`okxuulsz`); the site's file names carry a hash (`Asana-Math.<hash>.otf`), which changes no byte of the font and no name it presents | `cmp`; `licences.py` |
| OFL 1.1 | condition 4: no copyright holder's name promotes a Modified Version | no modified version exists; the fonts are named as what they are (export menu, About, README) | |
| OFL 1.1 | condition 5: distributed under the OFL only | each copy travels with the OFL alone; the crate's licence field is `EUPL-1.2 AND OFL-1.1 AND LPPL-1.3c`, one licence per part | `cli/Cargo.toml` |
| OFL 1.1 | embedding: a PDF embeds a subset (krilla), a PNG rasterises the glyphs | condition 5: "The requirement for fonts to remain under this license does not apply to any document created using the Font Software"; the preamble: the fonts "can be bundled, embedded"; OFL-FAQ 1.12 ("either in full or a subset") and 1.13 (a graphic made with the font is not subject to the OFL); linlog distributes no such document (its PDFs are made in the `export` check's sandbox, and by users) | STIX Two's `OFL-FAQ.txt` |
| GUST Font License (LPPL 1.3c), New Computer Modern Math | LPPL clause 2 (a complete unmodified copy) does not apply to one file; clause 6 for a part: (a, b) changes identified, (c) no support implied, (d) where the complete Work is | `NewCMMath-Book-LICENSE.txt` says each, before the GUST text and the LPPL, beside the font in `cli/fonts/`, in the crate, in the package's docs, and on the site as `/fonts/NewCMMath-LICENSE.txt` | the file; `licences.py` |
| GUST Font License | the request (not legally required) to rename derived fonts | the font is unchanged, so it keeps its name; the notice says so | |
| LPPL 1.3c | its text not modified ("modification of it is not allowed") | the LPPL and GUST texts are verbatim (whitespace too; checked by comparing with the sources) | the file's tail equals LaTeX's `lppl.txt` |
| LPPL 1.3c | embedding in a PDF, rasterising in a PNG | making a drawing is running the Work, which clause 1 leaves free ("the act of running the Work is not restricted"); a PNG holds no font program. A PDF embeds the subset of the font its text uses, which is part of the Work, and the LPPL, unlike the OFL, makes no exception for documents (clauses 2 and 9). linlog and the site distribute no such PDF (the export check's PDFs stay in its sandbox; a visitor's export is the visitor's), so linlog's own obligations hold; README no longer says the documents are outside every font's licence, but outside the OFL's, with New Computer Modern's PDFs named (the reviewer's finding) | the LPPL's text; README |
| LPPL 1.3c | sale | no clause forbids it, and nothing is sold | |
| Every font | the licence listed where the site lists the fonts | About lists all five with their licences and links (ten licences, `learn.spec.ts` fetches each); README's licence sections (linlog and the client); the export menu names the fonts only, as a choice, not a copy | `BottomBand.tsx` `LICENCES` |
| The flake's packages and Cachix | the same as the files they hold | `linlog-cli` installs every `cli/fonts/*.txt`; `linlog-web` holds no font (`LICENSE`, `THIRD-PARTY-NOTICES.txt`, the module); the client's site package is the served files above; Cachix holds the store paths CI builds on `main`. Two copies were found by the reviewer and fixed: the export check's directory for New Computer Modern (one file of the package) now holds its licence too, and the `test-*` checks, whose outputs kept the target directory with the command's test binaries (every font inside, no licence), keep none now (`doInstallCargoArtifacts = false`, as every other check here) | `modules/export.nix`; `modules/checks.nix`; crane's `mkCargoDerivation` (`doInstallCargoArtifacts = args.doInstallCargoArtifacts or true`) |
| The GitHub release | the same as the crate | `release.yml` attaches the `.crate` files, which carry `fonts/*.txt` beside the fonts | `.github/workflows/release.yml` |
| The core crate `linlog` | — | contains no font (`cargo package --list -p linlog`: no `.otf`); its font test reads `../cli/fonts` in the repository | |
| CC BY 4.0, the written content (D23) | attribution: the creator, the licence, a link | every page with written content says CC BY 4.0 with the link (the `licences` check); the notice now names the creator (`pusswryq`), as section 3(a)(1)(A) asks a re-user to credit; the new practice problems are written content, under the same notice | `licences.py`; `index.astro` |
| Quotations | short, with the source named | the reference's one quotation (the Handbook of linear logic, section 1.1, one sentence) names the work, its section and a link | `texts/reference.md` |
| Third-party files | none committed where a rule forbids | the client commits none (its rule): every font comes from linlog's input or npm; linlog's `cli/fonts/` is where the command's fonts live, as Euler Math's always has, which no rule of linlog forbids | `jj file list` of the client: no font file |

The `linlog-cli` crate grows by the fonts: they compress to about 2.7 MB
(`gzip -9` of `cli/fonts/`), under crates.io's 10 MB limit.

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

The supervisor then asked the same reviewer, in the same round, to judge
New Computer Modern Math and the licence table; it reported four more:

5. The export check's store path for New Computer Modern held the font
   alone, one file of an LPPL work without its notice, and CI pushes
   check dependencies to the public cache. **Holds**: the path now holds
   `NewCMMath-Book-LICENSE.txt` beside the font (`syrwpuww`).
6. The `test-*` checks keep crane's target directory, so their outputs
   hold the command's test binaries, every font inside, without the
   licences, and CI pushes them. **Holds** (older than this step, Euler
   Math's case too): `doInstallCargoArtifacts = false` for both, as
   every other check of the flake has (`syrwpuww`).
7. The table's LPPL row cited clauses 1 and 11 for embedding, which
   cover running the font but not handing on a PDF that embeds part of
   it; README said documents made with the fonts are outside their
   licence. **Holds**: the row and README now say that the OFL frees
   documents and the LPPL makes no exception, and that linlog
   distributes no such PDF.
8. The table missed the GitHub release's `.crate` files. **Holds**, and
   the obligation is met there (they carry `fonts/*.txt`): a row added.

It found New Computer Modern holding on every point (advances, glyphs,
depth 250, bounds, the export check's file, the notice against LPPL
clause 6, the texts verbatim) and the rest of the table holding.

## Verification

- linlog, last: `gate` and `nix flake check` passed (exit 0) at
  `qsoqwtqp` (207468e1 before the rebase, `fill`), and before it at
  `wttllvpx` and `umzuqsrr`; `nix flake check` passed (exit 0) at
  `tzpvrrut`, the report's commit, after which "linlog ready" went to
  `linlog-ef`. The supervisor rebased the chain onto `zvktswqy` and added
  `wvxvwrwo` (`MathFont::ALL` a slice, `SequentMismatch`
  non-exhaustive, `MathFont` in `core.md`'s list) and `qmopnnvu`
  (`Proof::onto` matches through buckets keyed by the written formula
  before `alike` decides, instead of comparing every pair with no poll),
  gate and check passing at `qmopnnvu` (bf80f7e4), `main`. This report's
  last commit sits on it, its check's exit status in the last message.
- linlog, first: `gate` passed (exit 0) at `zupynzoz` (a first run failed on
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
  path:…`): Vitest 114 passed (the token check among them); `lint`
  (Biome and `astro check`) clean; Playwright 59 passed, among them
  `tests/zoom.spec.ts` (2), `tests/dismiss.spec.ts` (2), the fonts'
  tests (the export menu, Settings' font, the targets in every font at
  both widths) and item 9's two; the `licences` check on the build exit
  0. One test changed for behaviour changed on purpose: About lists 10
  licences, not 6 (`learn.spec.ts`).
- The client's `nix flake check` at its last commit `xmoymlwt`
  (e507ebe7), the lock on linlog's `main` bf80f7e4: exit 0, Vitest 114,
  Playwright 61.
- The client's `nix flake check` against this workspace (a preview, no
  lock committed) failed once in the phone's dismiss test, which the dev
  shell passed: a sheet slides in, and measured while it moved the press
  landed on its title whenever the run was fast (CI's mode, no trace).
  Fixed in the test's aim, not its assertions (`uqwwwvnq`): it measures
  once no animation runs, and a sheet starting at or above the stage's
  top covers it there; three CI-mode runs then passed, and the preview
  check passed (exit 0).
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

- linlog: `../linlog-step38b`, rebased by the supervisor onto
  `zvktswqy` and pushed as `main` (`qmopnnvu`, bf80f7e4) with its two
  commits; this report's last commit on top. The chain: `tvrwrqsp` (the fonts),
  `zupynzoz` (the lock blessed), `qzxqzvqk` (the descenders), `lysrossv`
  (this report), `ormuvkok` (New Computer Modern), `spxtytpy` (its
  lock), `vurxrnok` (STIX Two's licence verbatim), `umzuqsrr` (the
  report), `syrwpuww` (no font without its licence in the cache's
  paths), `wttllvpx` (the report), `qsoqwtqp` (`fill`), and the report's
  last commit.
- The client's commits above `yxtvvlsm`: `wvzrmkqt` (zoom), `xqyzqkpw`
  (light dismiss), `vlktpknq` (the chip), `wpspsroq` (practice),
  `wrlrwoum` (the export's font), `yumxuxuo` (README and rules),
  `okxuulsz` (New Computer Modern, the fonts checked unmodified),
  `pusswryq` (the CC BY notice's creator), `qvzwknlo` (the stage's font),
  `rmvmsmxo` (the zoom floor after a step), `zlwkpvtr` (moving work
  between tabs), `uqwwwvnq` (the dismiss test's aim), `zvywrmku` (the
  lock on linlog's `main`, bf80f7e4) and `xmoymlwt` (item 5's test).
- The client: `/home/tux/Projects/own/linlog-prover.github.io`, its
  commits above `yxtvvlsm` (`main`), local, nothing pushed.
- Screenshots, exports and scripts:
  `/home/tux/Projects/own/linlog-step38b-screenshots/`: `before/` and
  `after/` (scenes 01 to 14 at a desktop's and a phone's width in both
  themes, the same names), `exports/` (a PNG and a PDF per font),
  `stage-fonts/` (the stage in each font, both widths, light),
  `practice-decided.txt` (the practice problems decided), and the
  scripts `shots.mjs`, `exports.mjs`, `stagefonts.mjs`, `gen.py`,
  `harder.py`, `steps.mjs`.

## Session 38c: the mode per tab, and the chip

The author's three requests of 2026-10-23 (08:10), in the client on top
of its `main` (e507ebe7); this section in the workspace
`../linlog-step38c` on linlog's `main` (`lqqomvlo`).

### The checklist

| request | state | evidence |
|---|---|---|
| 1. A goal opened in a new tab keeps the original tab's whole mode | done | the client's `szzqpnyt`; `tests/modes.spec.ts`, a goal of an intuitionistic, an intuitionistic affine, a Mix and an affine tab: before the fix the two intuitionistic ones failed (the new tab never started, no formula on its stage) and the Mix and affine ones passed; after it all four pass |
| 2. Every tab keeps its own mode, across a reload | done | the client's `nvoxzkyx`; `tests/modes.spec.ts`, `each tab keeps its own calculus, across a reload` (three tabs in three calculi, one changed, the others unchanged, the same after a reload and the restore bar) |
| 3. A tab's calculus chip opens that tab's mode settings, at both widths, by keyboard too; the switches leave the phone's More sheet | done | the client's `nvoxzkyx`; `tests/modes.spec.ts`, `the tab's calculus mark opens its switches, by a press and by a key` at a desktop's and a phone's width (the popover drawn on top where it is, Mix set, Escape, a key opens it with the focus inside, Escape returns the focus to the mark, a new tab's calculus set before it starts, and on a phone More holds no switches); step 38's touch-target walk (both widths, the popover with its switches among its states) and the token check pass |
| Before-and-after screenshots of the chip and its popover | done | `/home/tux/Projects/own/linlog-step38c-screenshots/before/` and `after/`: `01-chip` (and `-closeup`), `02-chip-popover`, `03-more-sheet`, at a desktop's and a phone's width in both themes |
| The client's `nix flake check` at its last commit | passed | exit 0, "all checks passed!", at `nvoxzkyx` (0a16a1e6): Vitest 114, Playwright 68, `tests/modes.spec.ts`'s seven among them |

### Outcome

A goal of an intuitionistic tab now opens in a new tab in the same
calculus, as Mix and affine tabs' goals already did. Each tab keeps its
own calculus, across a reload. A tab's calculus mark opens its switches
at every width and by keyboard; on a phone they are no longer under
More.

### Decisions

- **Where the mode was lost** (request 1): `sequentText` wrote a
  hypothesis's formula as linlog gives it, one-sided (`A⊥` for the
  hypothesis `A`, `A ⊗ B⊥` for `A ⊸ B`), left of `⊢`, so the new tab's
  text was another sequent with no intuitionistic reading; `start`
  refused it, and the tab stayed unstarted (in the visit's last mode).
  The mode itself was passed on correctly (`open(text, p.mode)`), which
  is why Mix and affine tabs' goals opened right. A hypothesis is now
  written as the dual of its one-sided formula: a negated atom without
  its negation, any other formula negated whole (`~(A ⊗ ~B)`), which
  linlog reads as `A ⊸ B`.
- **The whole mode**: linlog's mode is the calculus (classical or
  intuitionistic) with Mix and affine weakening, one `Mode` per tab
  (`Proof.mode`), which the new tab takes whole. The two-sided
  presentation is not part of it: the switch is greyed (linlog does not
  draw classical derivations two-sided yet) and holds no state, so there
  is nothing to carry over.
- **A new empty tab** (request 2) starts in `lastMode`, the calculus last
  chosen or started in this visit, classical at a visit's start: a
  student working in intuitionistic logic wants the next proof in it
  too, and a visit starts from the design's default (D2). It is not kept
  across visits, since each tab's own mode is.
- **The chip's popover** (request 3) holds the "Linear" and "Ordinary"
  choice as before, the calculus's switches (the same component as the
  stage's cluster, with its restart warning) and the fragment. Opened
  by a key (Enter or Space on the mark), the switches take the focus,
  and Escape gives it back to the mark; opened by a press, the focus
  stays where it was, so no tip pops up. A tab not yet started can have
  its calculus set by its mark before it starts.
- **The desktop's stage keeps its cluster** (judged): a change of
  calculus stays one press where the work is (step 38's walk), and the
  cluster shows the calculus at a glance on the stage, as D58 has it.
  While the mark's popover is open the cluster hides, so the switches and
  the restart warning (one `ui.pendingMode`) never show twice.
- **The band's popovers were clipped**: the row of tabs scrolls
  (`nav.proofs`, `overflow: auto`, 34 pixels high), and the absolute
  Status and Logic popovers under a tab were cut off by it, invisible,
  in `main` (the before screenshots show no popover). Its controls still
  passed the targets walk, which measures what is laid out. Both
  popovers are now `position: fixed` under the active tab, placed from
  its box and kept inside the window; the test asks what is drawn at the
  popover's middle.
- Two tests reached the switches through More on a phone (the targets
  walk and the dismiss test); they reach them through the mark now, the
  behaviour having changed on purpose.

### Departures from the design

- (departs from the design: D58, at the author's request) The calculus's
  switches are opened by each tab's calculus mark at every width, and on
  a phone they are no longer folded into More's sheet; on a desktop the
  stage's cluster stays as D58 places it.

### Judgement calls

1. The desktop's stage cluster stays (above), hidden while the mark's
   popover is open.
2. A hypothesis of a goal opened in a new tab is written as a negated
   formula where it is compound (`(A ⊗ B⊥)⊥` in the tab, which linlog
   reads as `A ⊸ B`): the page has no reading of a goal's formulas but
   the one-sided text, and linlog's bindings give none. A reading of a
   goal's formulas from linlog would let the tab write `A ⊸ B`, a
   follow-up for the bindings.
3. Status's popover was placed fixed with the mark's, since one fault
   clipped both.
4. A new empty tab starts in the visit's last calculus (above).

### Verification

- `tests/modes.spec.ts` (7 tests) failing where it failed before the
  fix and passing after, as above; the whole Playwright suite 68 passed
  (four workers, and again in CI's mode), Vitest 114 (the token check
  among them), `lint` clean. One run with the default number of workers
  failed two older tests under load (a link's restore, the tab's signs);
  both passed alone and in the two full runs after.
- The client's `nix flake check` at its last commit `nvoxzkyx`
  (0a16a1e6): exit 0, "all checks passed!", Vitest 114, Playwright 68.

### Repositories and workspaces

- The client: `/home/tux/Projects/own/linlog-prover.github.io`,
  `szzqpnyt` and `nvoxzkyx` on `main` (e507ebe7), local.
- linlog: `../linlog-step38c` on `main` (`lqqomvlo`), this section's
  commit. (Making the workspace, a gpg prompt timed out half-way and left
  a git worktree registered for a directory without files; that entry
  under `.git/worktrees/` was removed by hand, as `git worktree prune`
  would, and the workspace made again.)
- Screenshots and the script: `/home/tux/Projects/own/linlog-step38c-screenshots/`.

## Session 38d: goals in their reading

The author's go of 2026-10-23 (08:45) for the follow-up session 38c left:
a goal's formulas written in their intuitionistic reading wherever the
page writes them as text. linlog in the workspace `../linlog-step38d`
(made on `main`, `mvmwrrpr`); the client on its `main` (0a16a1e6).

### The checklist

| item | state | evidence |
|---|---|---|
| A public, additive way to write one occurrence as its reading writes it | exists already | `Reading::formula(o) -> IllFormula` (public, `occurrences/reading.rs`), whose `Display` is the crate's two-sided printer for one occurrence (`Notation::ill`, through `TEXT`), iterative over the formula (`Stops`); no new core API, as the supervisor's correction says. A core test pins it on occurrences below the roots: `one_occurrence_is_written_in_its_position` (a hypothesis `A ⊸ B` as `A ⊸ B`, the same under a hypothesis's `!` and the goal's `!`, no `~` or `⊥` written) |
| The bindings' view gives each goal formula its reading | done | linlog's `wstpsxur` (0a8db839): `Formula.reading`, in intuitionistic mode alone (absent in classical mode); `a_goal_s_formulas_carry_their_reading` (a hypothesis `A ⊸ B` reads `A ⊸ B` where its `text` is `A ⊗ ~B`, a goal under `!` opened by a promotion reads `A ⊸ B`, a classical session has no `reading`); the gate and `nix flake check` exit 0 at it |
| The client uses it | done | the client's `rzkqnwtk`: a goal opened in a new tab, the split chooser's Left and Right lists, the rules popover's heading and the formula buttons' names (`shown`), the dual kept as the fallback for a formula without a reading; `tests/modes.spec.ts`, `a goal's hypothesis A ⊸ B opens in a new tab as A ⊸ B, and the split chooser lists the formulas as they read` |
| The client's lock on the supervisor's commit, its `nix flake check` | done | the client's `vzypyoxl` (ffc93e59): `flake.lock` names linlog `0a8db83907c5d1a4b621c4ce496fd6d0e0aebfb6`, the commit the supervisor answered with (`wstpsxur`, pushed as it was); the client's `nix flake check` exit 0, "all checks passed!", at it: Vitest 114, Playwright 69 |

### Decisions

- **No new core call.** `Reading::formula` already writes one occurrence
  as the reading reads it in its position (an input `⊗` as `⊸`, `⊥` as
  `1`, an input `⊤` as `0`, a literal as its atom), public and documented,
  and `Interactive::reading()` gives a session's reading. The bindings
  call them; the core gained one test.
- **The field is `reading`**, beside `text` and `side`, present in
  intuitionistic mode and absent in classical mode, where the one-sided
  `text` is how the formula is written. `text` stays one-sided: it is
  what a step's position and the parser read.
- **No wire level** (`wire.rs`): the view is the answer of a call, never
  a stored document, and a key an older reader may ignore raises no level
  even in a document.
- **The client shows a formula through one helper**, `shown`: the
  reading where linlog gives one, else the one-sided text in signs. The
  formula buttons' accessible names change accordingly (`A ⊸ B, formula 2
  of goal 1`), which the new-tab tests of session 38c name now; a step
  still names a formula by its position.
- **The dual stays as the fallback** of `sequentText` for a formula
  without a reading, which an intuitionistic session never has now.

### Verification

- linlog: the gate and `nix flake check` exit 0 at `wstpsxur` (0a8db839),
  then "linlog ready"; the supervisor pushed it as `main`, its own check
  passing.
- The client, against the workspace first (Vitest 114, Playwright 69),
  then at its last commit `vzypyoxl` with the lock on `0a8db839`: `nix
  flake check` exit 0, "all checks passed!", Vitest 114, Playwright 69,
  `a goal's hypothesis A ⊸ B opens in a new tab as A ⊸ B, and the split
  chooser lists the formulas as they read` among them.

### Repositories and workspaces

- linlog: `../linlog-step38d`, `wstpsxur` (pushed as `main` by the
  supervisor) and this section's commit on it.
- The client: `rzkqnwtk` (the readings on the page) and `vzypyoxl` (the
  lock) on `main` (0a16a1e6), local.

## Session 38e: the author's decisions

The author's ten decisions of 2026-10-23 (`plan/38-web-polish.md`,
"Session 38e"), in the client on top of its `main` (`lwrmzpqv`,
0397f950); this section in the workspace `../linlog-step38e` on
`vrvtpwsl`. Nothing of linlog's code changed.

### The checklist

| item | state | evidence |
|---|---|---|
| 1. A privacy page (Fabian Lukas Grubmüller, privacy@grubmueller.dev, no Impressum), each legal statement read in its source and cited, linked from About and the footer, About's line on GitHub's logs pointing to it | done | the client's `vspwvwtv` and, after the review, `osyprzqo`: `src/pages/privacy.astro` at `/privacy/`, ten numbered sources, every statement citing one; Playwright `the privacy page is served, names who is responsible, cites a source for each statement, and About and the band link it` (and the 404 page's link) and, on a phone, `More's sheet links the privacy page at a phone's width`; both fail on the site before (no page) |
| 2. The calculus's switches on the tab's chip alone at every width, the desktop stage's cluster gone | done | `uzmnzktm`; Playwright `the calculus's switches are on the tab's mark alone, never on the stage` at both widths: the desktop one fails on the site before, the phone one passed there already (38c had taken them off the phone's stage) |
| 3. A forced engine lasts for the visit | done | `vytkrqmn`; Playwright `a forced engine holds for the visit: the next visit starts on Automatic, every other setting kept` (fails before: the engine was stored) |
| 4. Every colour of meaning distinct under simulated deuteranopia, protanopia and tritanopia, checked over the tokens; the nets on Okabe–Ito; no meaning by colour alone | done | `twzyqtqs` and, after the review, `yuluurur`; Vitest `src/styles/colours.test.ts` (each scheme, every pair of each group, 8 under each deficiency and 15 in full colour; the nets' hues within 3° of Okabe–Ito's; the check's own check), which on the old tokens fails 13 pairs by day and 9 by night (with the first groups) and the input side's hue; Playwright `a proof's state is told by its mark's shape as well as its colour` (fails before: a dot) |
| 5. The essential net's input side in a colour of that palette, not green | done | `yuluurur`: Okabe–Ito's blue (#045383 by day, #80c6ff by night), after the review found the first choice, yellow, too near the amber of "unknown"; Playwright `the essential net draws its input side in the Okabe–Ito palette's blue, never green, in both themes` (fails before) |
| 6. The status popover writes the mode in the switches' words | done | `rzquvkyx`; Vitest `a mode's words`, Playwright `the status popover writes the calculus in the switches' words` (fails before) |
| 7. The dock's counter reads "Goal 2 · 1 open" | done | `sumwmuvr`; Playwright `the split chooser sends formulas left …` expects "Goal 3 · 2 open" (fails before) |
| 8. The additive path's reference claims nothing a source does not state | done | `mlyxkmnm`; Vitest `src/content/reference.test.ts` (the rules cited, Whitman after "Related:" alone, nothing of Whitman in the hover's source) |
| 9. "Fill from …" names the source tab by its shortened sequent | done | `vzuxxyzm`; Vitest `a tab's sequent, shortened to name the tab`; Playwright `a goal is filled from another tab's finished proof as one step named with the tab's sequent, …` and `a fill names its tab by the tab's sequent, which stays as tabs before it close` (both fail before) |
| 10. BottomBand.tsx's header comment true | done | `xxsxmuoq` (the header, More's tip and its sheet's doc comment); a comment, which no test sees |
| Before-and-after screenshots at a phone's and a desktop's width in both themes | done | `/home/tux/Projects/own/linlog-step38e-screenshots/before/` and `after/`, 88 each, the same names (below) |
| The reviewer's one round, each finding answered | see "The review" | |
| The client's `nix flake check` at its last commit; README and rules true; `plan/later.md`'s "Kept as built" true | passed | exit 0 at `osyprzqo` (8fb9b3fe), "Verification"; README's "What the page does", `.claude/rules/web.md`, `CLAUDE.md`'s layout; `plan/later.md` in this workspace |

### Outcome

The page has a privacy notice at `/privacy/`, reached from About, from the
end of the bottom band (More's sheet on a phone) and from the 404 page. It
says who is responsible and how to write to them, that the site sets no
cookies, tracks nothing and loads nothing from another server, what
GitHub logs as the host and under which safeguard it goes to the United
States, what the browser keeps and how to clear it, and the visitor's
rights under the Swiss Act and, where it applies, the GDPR; each
statement cites one of ten sources read in themselves, and the page says
it is information, not legal advice.

The calculus's switches are on each tab's calculus mark alone, at every
width. A forced engine holds until the page is loaded again. The status
writes "Intuitionistic, affine", the dock "Goal 2 · 1 open", and a fill
names its tab by its sequent ("Fill from ⊢ A⊥, B⊥, B ⊗ A"). The additive
path is cited by the rules it applies, Whitman's lattices named as
related.

Every colour that carries a meaning stays apart from the colours seen
with it under simulated protanopia, deuteranopia and tritanopia, held by
a test over the tokens; the nets keep Okabe and Ito's hues, the essential
net's input side yellow instead of green; a tab's state is a shape as
well as a colour.

The client's commits, on its `main` (`lwrmzpqv`), local:

| change | what |
|---|---|
| `uzmnzktm` (108227e1) | item 2, the switches on the mark alone |
| `rzquvkyx` (01aceb16) | item 6, the mode in the switches' words |
| `sumwmuvr` (2d1ec1d7) | item 7, the dock's counter |
| `vzuxxyzm` (0eb40fe2) | item 9, a fill named by its tab's sequent |
| `vytkrqmn` (d8f86861) | item 3, a forced engine for the visit |
| `mlyxkmnm` (01767c71) | item 8, the additive path's reference |
| `xxsxmuoq` (940e6be2) | item 10, the bottom band's header |
| `twzyqtqs` (7dd8c8fc) | items 4 and 5, the colours |
| `vspwvwtv` (96c533e7) | item 1, the privacy page |
| `xkurktxq` (41d218f8) | the bottom band in one row with Learn on, the privacy link included (found by the screenshots) |
| `yuluurur` (e54cefbb) | the review's finding 1: the nets' colours against the bands' |
| `osyprzqo` (8fb9b3fe) | the review's findings 2 and 3 on the privacy page |

Each of the first nine commits passed lint and Vitest on its own; the
chain passed Playwright (78) before it was split into them, whose ninth
is byte for byte the tree that was tested. The last three passed lint,
Vitest (121) and Playwright (79) together.

### Decisions

- **The colour check** (item 4). The simulation is Machado, Oliveira and
  Fernandes 2009, its severity-1.0 matrices read in the authors' own
  table (their page, Table 1), applied to linear RGB: the paper builds
  them from the primaries' spectra (its Eq. 8), which add in linear
  light, so sRGB is linearised first (CSS Color 4's `lin_sRGB`); this is
  the session's reading, the page itself not saying which RGB. Distance
  is Euclidean in OKLab times 100 (Ottosson's post, its matrices read
  there): at least 8 under each deficiency, 15 in full colour, the
  thresholds of the `dataviz` skill's validator, which uses the same
  simulation (the page's own choice of threshold, not a published one).
  The pairs checked are those seen together with a meaning: a proof's
  states (`ok`, `warn`, `bad`, `accent` for open, `line` for not started),
  the focus (`accent` among `ink`, inside Learn's frame), each drawing's
  colours with one another (a net: ink, ⅋, a witness; an essential net:
  ink, the input side, the dominator lines, a witness), and each drawing's
  colours with the bands' state and selection colours around it (the
  review's finding 1; the first groups compared a net's colours only with
  each other).
- **Tritanopia**: Machado et al.'s paper says its model "is not intended
  to handle tritanopia"; the authors' table gives a tritanomaly matrix
  all the same, and it is the published simulation at hand (Brettel,
  Viénot and Mollon 1997 is behind a paywall, with no open copy found).
  The test uses it and says so (departs from nothing, but it is weaker
  ground than the other two).
- **The new colours.** Each meaning keeps its hue family; values were
  searched (a scratch program over OKLCH, lightness and a few degrees of
  hue, the nets' hues fixed to the palette's) to pass the check with
  text at 4.5:1 on the field and the panels, and a keyword at 4.5:1 on
  its wash over a cluster (the old green was 4.43 there). By day: ok
  #0d430d, warn #715301, bad #b23554, accent #6b46cf, Learn #730e73; by
  night: ok #06b68c, warn #f5cb4d, bad #ec6e52, accent #958ad5, Learn
  #f197ed. The net colours are Okabe–Ito's hues at 4.5:1 on the field,
  the same member per role in both themes: ⅋ and the input side blue
  (#045383, #80c6ff), the dominator lines reddish purple (#6d2351,
  #ef99c8), a witness vermilion (#933f02, #d05c03); the palette's values
  were read in Okabe and Ito's own figure. The accent moved from indigo
  to violet, to stand apart from the nets' blue.
- **The input side's colour** (item 5) is the palette's blue, the colour
  of ⅋'s edges, which are never in an essential net: the session first
  chose yellow (no meaning on the page), but by day at 4.5:1 a yellow is
  a dark olive that deuteranopia makes the amber of "unknown" (the
  review's finding 1). Searched with each of the palette's members for
  the input side against the bands' colours, by day at 4.5:1: yellow,
  orange and vermilion reach at most 7.2, sky blue and blue 8.2; blue
  was taken, as one colour for the two (the test holds both to the
  palette's blue).
- **No meaning by colour alone.** A tab's state, a coloured dot before,
  is now a shape in its colour (a check for proved, a cross for refused
  or unprovable, a question mark for unknown, a dotted circle for open
  goals, a dashed one for not started; Lucide's icons). A hint's rungs
  are a full bar when climbed and a hairline ahead. The rest had a sign
  already: the keywords are words, the split's left formulas carry an
  "L" and a failing split dashed premises, the active goal is outlined,
  Learn's frame says "Learn", ⅋ edges are dashed, dominator lines
  dotted, the input side's edges point up. A witness's colour has no
  sign of its own, and is never drawn on the page: it marks a structure
  that is no proof net, and the page draws nets of finished proofs only.
- **The privacy page's content.** Swiss law: FADP Art. 19(2) and (4)
  (identity and contact, purpose, recipients, the state abroad and the
  safeguard), Art. 16(1) and the DPO's Annex 1 for the United States,
  Art. 25, 32 and 49 for the rights, the TCA's Art. 45c for what the
  browser keeps (the visitor told what and why, and that they may
  refuse); the GDPR's Art. 13 where it applies, with Art. 6(1)(f) as the
  basis (the page's reading), Art. 45(1) and Decision 2023/1795 for the
  transfer, Art. 15 to 18, 20, 21 and 77. GitHub's Pages documentation
  is quoted for the logging ("When a GitHub Pages site is visited, the
  visitor's IP address is logged and stored for security purposes, …"),
  its privacy statement (effective 27 April 2026) for the places,
  retention criteria and its own certifications. GitHub's entry on the
  Data Privacy Framework list, read on 23 October 2026 through the API
  the list's page calls (`participant/6174`): "GitHub", San Francisco,
  the EU-US framework, its UK extension and the Swiss-US framework all
  active, non-HR data, each to 3 August 2027 (the record also lists
  lapses from 2021 to 2023, before the present frameworks, which the
  page does not repeat). Each statement is read in its source by a
  research sub-agent (Sonnet 5.5, `medium`) and the load-bearing quotes
  checked again by the session in the files it fetched; Fedlex's own
  pages being a script, its filestore's HTML of the same texts was read,
  and EUR-Lex through the Publications Office's CELEX addresses. The
  page claims no ePrivacy exemption and names no EU representative (GDPR
  Art. 27); it gives no postal address, as the author decided.
- **The privacy page's look** (the `frontend-design` skill): a page of
  text in the 404 page's frame, left-aligned, 640 pixels at most, the
  title at a new step of the type scale (`--text-title`, 32 pixels,
  which the 404 page's title takes too), and one figure, "who has which
  of your data", read like an instrument's readout (three rows: this
  site, GitHub, your browser), the citations quiet and numbered, the
  sources a numbered list each citation links to.
- **A visit** is a load of the page, as for `lastMode` (session 38c): a
  reload starts on Automatic. Settings says so under the engines.
- **Item 2** removed the stage's cluster and its offset (`hudBottom`):
  the drawing's fit and the dock's place keep 96 pixels at the stage's
  top for the dock, as before when no cluster was drawn.

### Departures from the design

- (departs from the design: D58, at the author's request) The calculus's
  switches are no longer a floating cluster on the stage at any width;
  the tab's calculus mark opens them.
- (departs from the design: D55, at the author's request) The dock's
  counter joins two facts with a middle dot, "Goal 2 · 1 open", which
  D55 lists among the tells to avoid.
- (departs from the design: D53, by the author's rule on colour) A tab's
  verdict is a coloured shape, no longer a coloured dot.
- (departs from the design: D26, in its values) The verdict colours keep
  D26's roles and hue families, re-chosen in lightness and a few degrees
  of hue so that they pass the colour check; the green of a checked
  proof is a bluish green by night.

### Judgement calls

1. A fill names its tab by its sequent everywhere the tab was named by
   its place: the button, its tip, the refusal, the live region and the
   graft's label in the drawing and the step list ("Proof from ⊢ A⊥, B⊥,
   B ⊗ A"); cut at 24 characters with an ellipsis (`SHORT`). A graft
   saved before this change keeps its old key, `tab:2`, and is labelled
   "proof from 2".
2. The calculus mark's tip and accessible name use the switches' words
   too ("Calculus: Intuitionistic, affine; fragment IMLL"), as the status
   does; the order is the switches' (Mix before affine).
3. The privacy link stands at the wide bottom band's end with its label
   and a shield (Lucide's `Shield`), in More's sheet on a phone, and on
   the 404 page. With Learn on it wrapped the band at 1 280 pixels, which
   "Kept as built" says keeps one row (seen in the screenshots); the
   actions pack two pixels tighter with Learn on, and a test holds the
   row.
4. The colour thresholds, the groups and the tritanopia matrix, above.
5. The input side blue, shared with ⅋, above.
6. The witness's colour, never drawn on the page, has no sign of its
   own.
7. The privacy page's legal basis (a legitimate interest) and its reading
   that GitHub decides what it does with its logs are the page's reading
   of its sources, which its closing note says.

### What is left open

- The status popover's keyword reads "1 open goals" for one goal (older
  than this session, seen in the screenshots): a follow-up for the
  client.
- A graft filled before this change is labelled by its old key
  ("proof from 2").

### The review

One fresh-context reviewer (Opus 5.5, `high`) judged the privacy page
against its sources and the screenshots against the ten items, one
round, reporting only what breaks an item. Its findings and the answers:

1. **Item 4 and 5: the essential net's yellow input side was not held
   apart from the bands' colours shown with it**: by day 0.4 from the
   amber of "unknown" under deuteranopia and 7.7 from the green of a
   checked proof under protanopia, the test comparing a net's colours
   only with each other. Holds. The test's groups now put each drawing's
   colours with the bands' state and selection colours; no set with a
   yellow input side passes by day at 4.5:1, so the input side is the
   palette's blue and the bands' colours were chosen again (`yuluurur`,
   above). The new groups fail on the tokens before it.
2. **Item 1: the page named one transfer safeguard where GitHub's
   statement relies on two**: GitHub's statement names other countries
   besides the United States and the European Commission's standard
   contractual clauses (Decision 2021/914) for them (FADP Art. 19(4),
   GDPR Art. 13(1)(f)). Holds. The page now says so in GitHub's words
   and that GitHub names those countries no further (`osyprzqo`).
3. **Item 1: "running to 3 August 2027"** read the list's "Next
   Certification Due Date" (its page's own label for the record's
   `UsageEndDate`) as an end. Holds, checked in the list's script; the
   page says "the next certification's due date" (`osyprzqo`).
4. **The rules**: `.claude/rules/web.md` still named the stage's cluster
   as inside the restart warning's area. Holds; it names the switches
   (`yuluurur`).

Everything else the reviewer checked held: the FADP's, the DPO's, the
TCA's, the GDPR's and Decision 2023/1795's statements against their
texts, the Pages quotation, the retention paraphrase, what the browser
keeps against the code, the matrices, constants and palette against
their sources, and items 2, 3, 6 to 10.

### Verification

- Vitest 121 and Playwright 79 at the last commit (four workers), lint
  clean; each new Playwright test failing on the site built before the
  session (`SITE_DIR` at `site-before`) but the phone-width switches
  test, which passed there already; the colour test failing on the old
  tokens; the one-row band test failing on the build of `vspwvwtv`.
- The client's `nix flake check` at its last commit `osyprzqo`
  (8fb9b3fe), under the machine lock: exit 0, "all checks passed!",
  Vitest 121, typecheck 0 errors, Playwright 79 passed.

### Repositories and workspaces

- The client: `/home/tux/Projects/own/linlog-prover.github.io`, the twelve
  commits above on `main` (`lwrmzpqv`), local, not pushed.
- linlog: `../linlog-step38e` on `vrvtpwsl`, this section's and
  `plan/later.md`'s commit.
- Screenshots: `/home/tux/Projects/own/linlog-step38e-screenshots/before/`
  and `after/` (88 each: 13 scenes at a desktop's and a phone's width in
  both themes, the colour scenes 04 and 05 also under Chromium's
  simulated deuteranopia, protanopia and tritanopia), the script
  `shots.mjs`, and the two built sites `site-before/` and `site-after/`.
- The sources the privacy page was checked against, and the colour
  search's scratch programs, are in the session's scratch directory,
  not kept.


## Session 38f: the logo's candidates

The logo's design session (`plan/38-web-polish.md`, "Sessions 38f and
38g", the 38f paragraph), on Fable 5.1 at `high`, 2026-10-23. Nothing
was placed in the client or in linlog: the candidates and the page are
in `/home/tux/Projects/own/linlog-logo/`, outside both repositories, and
this section, in the workspace `../linlog-step38f` on `luwsvqkq`, is the
session's only commit. The author chooses; session 38g places the
chosen one.

### The checklist

| item | state | evidence |
|---|---|---|
| Four or five candidates, each a word mark and a compact square form, SVG with the text outlined, in a light, a dark and a one-colour version | done | five: `turnstile/`, `lollipop/`, `link/`, `par/`, `inference/`, each `wordmark-{light,dark,mono}.svg` and `square-{light,dark,mono}.svg` (30 files, listed by `tools/make.py`); `grep -c "<text" */*.svg` is 0 for every file |
| Legible at 16 and at 512 pixels; each candidate rasterised at 16, 32, 180 and 512 and the rasters looked at | done, with two weaknesses | `tools/raster.sh` (resvg 0.48.1): `<candidate>/png/square-{light,dark}-{16,32,180,512}.png` and a contact sheet `sheet.png` per candidate, each looked at; the par's ⅋ is a smudge at 16 pixels and the inference's figure is dense there (below, "What the rasters show") |
| Every candidate's colours distinct under simulated deuteranopia, protanopia and tritanopia by the client's method | done | `node tools/colours.mjs turnstile lollipop link par inference`: 20 groups (each candidate, each theme, the word mark on the field and the square form), every pair at least 8 apart under each deficiency and 15 in full colour, exit 0; the script copies the matrices, the linearisation, OKLab and the thresholds of the client's `src/styles/colours.test.ts` unchanged |
| One page showing every candidate at every size and place it will be used, in both themes, its screenshots looked at, published as a private artifact | done | `/home/tux/Projects/own/linlog-logo/index.html` (`tools/page.py`); the artifact https://claude.ai/artifact/UEq8nbTaXg9fYcLPEVCrwH (private, the page with its 70 files); `screenshots/page-{light,dark}.png` and one per candidate and theme (`tools/shots.mjs` in the client's devshell's Chromium), five looked at |
| The outlined fonts' licences read in their own files and named | done | Figtree: `node_modules/@fontsource-variable/figtree/LICENSE` in the client, the SIL Open Font License 1.1, no Reserved Font Name; Euler Math: `.cache/linlog-fonts/OFL.txt` in the client (the file the client writes, D25), the SIL Open Font License 1.1, no Reserved Font Name; its condition 5 read: "The requirement for fonts to remain under this license does not apply to any document created using the Font Software" |
| Where each idea comes from, never another project's mark | done | below, "The candidates": each from a sign of the calculus (⊢, ⊸, an axiom link, ⅋, the inference bar) or from today's icon |
| The recommendation and the proposed licence | done | below |
| The report committed in `../linlog-step38f`, `jj st` clean, nothing pushed | done | this section; the commit below |

### Outcome

Five candidates, each one logo in two shapes. The word is "linlog" in
Figtree Bold, the typeface and weight of the page's own top band (its
`.wordmark`: bold, 18 pixels, letter-spacing −0.5 pixels, which the
outline keeps as −28 units per em), shaped by HarfBuzz with the font's
kerning and outlined by fontTools, so no font file goes with any file.
Each mark is drawn in the word's own measures (a stem of 132 units on an
ascender of 700, the thin line of a ring, an arc or a bar at 70), which
is what makes the two shapes one logo. Every colour is one of the page's
tokens (`tokens.css`): ink, and one of the accent, the nets' blue and the
green of a checked proof, which is why the page's own colour rule holds
for every pair a candidate shows. The square form carries the page's
field as its background (`rx` 96 of 512, as today's icon) in its light
and dark versions; the one-colour version of both shapes is
`currentColor`, with no background.

The page shows, for each candidate in both themes: the square form at
16, 32, 180 and 512 pixels (the rasters themselves); as a browser tab's
favicon beside the title, under iOS's icon mask, inside a maskable icon's
circle, and as GitHub's avatar at 40 and 96 pixels; the word mark in the
page's 48-pixel top band at the band's own type size beside a sample
tab; at the README's head on GitHub's white and its dark canvas; in a
1 280 by 640 social preview at half size; and the one-colour versions in
ink and in white.

### The candidates

Each line: the idea and where it comes from, the colours, what the
rasters and the page show.

1. **Turnstile**: "⊢ linlog", the word as the conclusion of a one-sided
   sequent, since every sequent linlog decides begins with ⊢; today's
   icon's turnstile raised to the mark, drawn in the word's stem weight,
   not Euler's thin glyph, which 16 pixels would lose. The turnstile in
   the accent, the word in ink. Clear at every size; the plainest of the
   five. Its weakness: ⊢ is every logic's sign, not linear logic's.
2. **Lollipop**: the i of linlog as ⊸ stood on end, its dot the ring of
   the lollipop (the i is Figtree's dotless i with a ring of radius 130
   and line 70 joined to the stem); the square form is ⊸ laid down. The
   ring in the accent, the letters in ink. The one candidate whose mark
   is inside the word rather than beside it; the square form reads as ⊸
   at 16 pixels. Its weakness: in the band at 18 pixels the ring is
   about 5 pixels across, a visible open dot but not much more.
3. **Link**: an axiom link, the arc a proof net draws above a formula
   from an atom to its dual, here from the first l of linlog to the
   second; the square form two stems under their arc. The arc in the
   nets' blue, the page's colour for a ⅋ edge, the letters in ink. The
   arc is the one curve among the marks. Its weakness: the square form
   reads as headphones or a padlock to anyone who does not know nets;
   widening the stems and lowering the arc did not remove that.
4. **Par**: Euler Math's ⅋, the connective nobody but linear logic
   writes, in the glyph the page's every formula is set in, outlined and
   emboldened by a stroke of 22 units on 1 000 (the hairlines would break
   at small sizes); the square form the glyph alone. ⅋ in the nets'
   blue, the word in ink. The most beautiful at 180 and 512 and in the
   README's head, and the most linlog of the five; at 32 pixels a blue
   scribble, at 16 a smudge: it does not work as a favicon.
5. **Inference**: today's icon's figure, two premises over a bar, with
   the word as the conclusion and the green lamp of a checked proof at
   the bar's end; the square form today's icon redrawn in the word's
   stem weight (premises, bar, turnstile below, lamp). The rule in ink,
   the lamp in the green of a checked proof. The continuity candidate. Its
   weaknesses: the stacked word mark is small in the 48-pixel band (the
   word's x-height about 11 pixels), and the square form at 16 pixels is
   three bars and a dot, a generic list pictogram.

### The recommendation

The lollipop. It is the only candidate whose mark is the word itself, one
letter changed, so the word mark and the square form are plainly one
logo without a mark standing beside the word; ⊸ is the sign people
associate with linear logic first, and the square form reads as ⊸ at
16 pixels, which the par's ⅋ does not. The turnstile is the runner-up,
the clearest at every size, if the author prefers a mark that stands
apart from the word. Both are the author's call; the page shows all five
alike.

### The licence proposed

CC BY 4.0 for the logo's files, as for the client's written content, with
a one-line note beside it (in the client's About and licences, where the
fonts are named) that the name "linlog" and the mark identify this
project and may not be used to suggest its endorsement of something else;
CC BY-ND 4.0 is the stricter alternative, which keeps the mark from being
altered but is not a free-culture licence and sits oddly beside the EUPL
and CC BY. On the fonts' side nothing constrains the choice: both are
under the OFL 1.1, whose condition 5 exempts "any document created using
the Font Software", which an outlined logo is, and no font file travels
with the SVGs; naming the fonts in the logo's README is a courtesy, not a
requirement. The author decides.

### Decisions

- **The fonts**: Figtree for the word because the band sets "linlog" in
  it today, so the word mark in the band is the same word, outlined; the
  weight the band's (bold, 700, instanced from the variable font). Euler
  Math only for the par's ⅋, the glyph the page draws. Space Mono (the
  field's face) was not used: the logo is not typed input.
- **Euler Math in the par candidate (departs from the design)**: D25 has
  Euler Math wherever linlog draws and nowhere else; the par's mark is
  that font's ⅋ outlined into a logo, no font served. Marked for the
  author's review; the other four candidates use no Euler Math.
- **The colours**: taken from the tokens so that the client's rule holds
  by construction, which the check confirms. Two candidates pair ink
  with the nets' blue rather than the accent (D26 has one accent for
  selection; a logo is not a selection, but the author may prefer the
  accent throughout: `THEMES` and each candidate's roles in
  `tools/make.py` change it).
- **The marks in the word's stem weight**, not the symbols' own glyphs
  (except ⅋): a turnstile or a lollipop at Euler's hairline weight is
  gone at 16 pixels, and a mark in the word's weight is the word's.
- **The square form's background** is the field, light or dark, with the
  figure in the centre 60 per cent (inside a maskable icon's safe zone).
  The light square on GitHub's white is a faint grey tile; the dark
  square is the one that reads on either, as today's icon does.
- **The page's own chrome** uses the system font and no web font: the
  logo files carry their own outlines, and nothing is fetched from any
  service when the page is viewed.
- **GitHub's canvas colours** on the page, white and #0d1117, are from
  memory of GitHub's default themes, not read in a source (an
  assumption; the README's head is shown on them only to show the two
  backgrounds).
- **The tools**: resvg, fontTools, uharfbuzz and brotli from the nixpkgs
  linlog's `flake.lock` pins (`tools/build-tools.sh`); Chromium through
  Playwright in the client's devshell, entered from the client's
  directory with `env -u LD_LIBRARY_PATH`; every run in a capped scope
  on cores 8 to 11, nix under `/tmp/linlog-machine.lock`.

### What the rasters show

Looked at on each candidate's `png/sheet.png` (16 and 32 magnified
eightfold, pixel for pixel, 180 and 512 as they are, the word marks at
1 200 wide in all three versions): the turnstile, the lollipop and the
link's square forms hold at 16; the par's ⅋ and the inference's figure
do not (above). The page's screenshots were looked at for the turnstile,
the lollipop, the link, the par and the inference and the whole page;
the first run showed the band row broken by a CSS rule that let the word
mark take the pane's width, fixed in `tools/page.py` and reshot.

### Departures from the design

- The par candidate outlines Euler Math into the logo (D25 has the font
  nowhere but the drawings) (departs from the design).

### What is left open

- The author's choice among the five, and the licence.
- If the lollipop is chosen: whether the band keeps the ring at 18
  pixels or takes a slightly larger word mark; 38g sees it in place.
- If the link is chosen: the square form's headphone reading needs
  another drawing (dots for the atoms, or the letters themselves).
- A candidate's colours other than the tokens' would need the check run
  on them (`tools/colours.mjs` reads each `colours.json`).

### Repositories and workspaces

- The candidates: `/home/tux/Projects/own/linlog-logo/` (one directory
  per candidate with its six SVGs, `colours.json` and `png/`; `tools/`
  the scripts; `index.html`; `screenshots/`; `README.md`), outside both
  repositories, uncommitted.
- The artifact: https://claude.ai/artifact/UEq8nbTaXg9fYcLPEVCrwH,
  private.
- linlog: `../linlog-step38f` on `luwsvqkq`, this section's commit;
  nothing pushed. The client is untouched.

### The lollipop's colours (the author's follow-up, 2026-10-23)

The author chose the lollipop and asked for its main colour among a few.
Seven variants of the lollipop alone, the letters in ink and the ring in
the colour, each in a light, a dark and a one-colour version, under
`/home/tux/Projects/own/linlog-logo/lollipop-colours/<variant>/`
(`tools/variants.py`, `raster.sh`, `page.py` beside them); the page
`lollipop-colours/index.html`, published as a second private artifact,
https://claude.ai/artifact/EY4cDDSrRxTbJwm6bCwCkz, the first page left as
it is. It shows the variants side by side in both themes as the 16- and
32-pixel favicon in a tab, in the 48-pixel top band at its type size, at
the README's head on GitHub's light and dark canvas, and in the 1 280 by
640 social preview, with the contrast table and the accent question
beside them; `lollipop-colours/screenshots/` is the page in Chromium,
looked at, and `sheet.png` the favicons magnified, looked at.

Every variant passes the client's colour-vision check (`tools/colours.mjs`
on each `colours.json`: the ring, the letters and the field, both themes,
14 groups, exit 0). The ring's WCAG 2 contrast:

| variant | light | dark | on white | on the light field | on GitHub's dark canvas | on the dark field |
|---|---|---|---|---|---|---|
| violet (`--color-accent`) | #6b46cf | #958ad5 | 6.2:1 | 5.3:1 | 6.2:1 | 6.1:1 |
| blue (`--color-net-par`) | #045383 | #80c6ff | 8.2:1 | 7.0:1 | 10.3:1 | 10.2:1 |
| vermilion (`--color-net-highlight`) | #933f02 | #d05c03 | 7.1:1 | 6.1:1 | 4.7:1 | 4.6:1 |
| orange (Okabe–Ito #e69f00, light darkened) | #9a5b00 | #e69f00 | 5.4:1 | 4.6:1 | 8.4:1 | 8.3:1 |
| bluish green (Okabe–Ito #009e73, both moved) | #00704f | #21c08f | 6.1:1 | 5.2:1 | 8.1:1 | 8.0:1 |
| reddish purple (`--color-net-dominator`) | #6d2351 | #ef99c8 | 10.5:1 | 8.9:1 | 9.1:1 | 8.9:1 |
| ink (`--color-ink`) | #121a2e | #e3e8f4 | 17.3:1 | 14.7:1 | 15.4:1 | 15.2:1 |

Would the site's accent have to change to match? One line each:

- **Violet**: no, it is the accent itself.
- **Blue**: no change needed, but the ring would wear the nets' colour
  (a ⅋ edge, an essential net's input side), not the selection's.
- **Vermilion**: no change needed, but vermilion is a witness's colour in
  the nets, a meaning the logo would borrow.
- **Orange**: no change needed; orange carries no meaning on the page
  today, and its light value is a brown to hold 4.5:1.
- **Bluish green**: no change needed, but green is kept for a checked
  proof alone (D26), which a green ring blurs.
- **Reddish purple**: no change needed, but it is the dominator lines'
  colour in an essential net and near the accent's hue, so the band
  would show two purples.
- **Ink alone**: no second colour at all; the ring is told by its shape.

The recommendation: violet, the page's accent, so that the ring in the
band is the same colour as the active goal under it and nothing on the
page has to move; blue is the one other that carries no clash in the
band, at the price of wearing the nets' meaning. The ink-only variant is
the safe fallback for places that print in one colour, and is what the
mono files are anyway.

The commit of this addition is the workspace's second; nothing placed,
nothing pushed.

## Session 38g: the logo placed

The logo's placing session (`plan/38-web-polish.md`, "Sessions 38f and
38g", the 38g paragraph; the brief of 2026-10-23), on Opus 5.5 at
`high`, 2026-10-23. The author's choice, the lollipop in violet under
CC BY 4.0, is in the web client and in linlog. The client's commits are
reviewed and pushed by the supervisor, who added one at the author's
request (the band's mark at 30 pixels). linlog's two commits are in
the workspace `../linlog-step38g`, and nothing of linlog is pushed.

### The checklist

| item | state | evidence |
|---|---|---|
| The chosen SVGs in the client as the one source | done | `src/logo/` holds `wordmark-{light,dark,mono}.svg` and `square-{light,dark,mono}.svg`. Each is the file under `linlog-logo/lollipop-colours/violet/` with only its title and label changed from "linlog (violet)" to "linlog" (a `diff` per file showed nothing else). `src/logo/LICENSE.txt` is the notice. |
| Every raster made from them by a script of the client's flake | done | `modules/logo.nix` (`packages.logo`, the option `logoFiles`) runs `modules/_logo/derive.py` with nixpkgs' resvg 0.48.1. It writes `site/`: `favicon-32.png`, `apple-touch-icon.png` (180), `icon-192.png`, `icon-512.png`, `icon-maskable-512.png`, `og.png` (1200 by 630). It writes `github/`: `avatar.png` (512) and `social-preview.png` (1280 by 640). The dev shell and every npm task link it at `.cache/linlog-logo`, and the build copies `site/` and `src/logo/` to `/logo/`. |
| Favicon (SVG and PNG fallback), the manifest's icons (maskable too), Apple touch icon, Open Graph image | done | `Base.astro` links `/logo/favicon-32.png` (32 by 32), `/logo/square-dark.svg` and `/logo/apple-touch-icon.png`. Every page carries Open Graph tags with `og:image` at `https://linlog-prover.github.io/logo/og.png`. The manifest names the three icons. Playwright test `the logo's icons and link preview are served at the sizes the page and the manifest name` fetches every one, checks its status and media type, and reads each PNG's width and height from its header. |
| The header's word mark, About, the 404 page | done | The new component `Logo.tsx` (the word mark inline) replaces the plain "linlog" in `TopBand.tsx` and opens About. The 404 page links the mark to `/`. Playwright test `the header and the 404 page show linlog's word mark, its ring in the logo's colour` checks both schemes: the mark is visible, it is as high as the band's token (24 pixels in this session's commit, 30 since the supervisor's `rnzzvnuk`), and the ring's computed stroke is the token's value. The About test finds the mark. |
| Guard 1: the logo's colour a token of its own | done | `--color-logo: light-dark(#6b46cf, #958ad5)` in `tokens.css`, beside `--color-accent` but never `var(--color-accent)`. `app.css` paints the ring with it. |
| Guard 2: the token in the colour-vision test's band group | done | `colours.test.ts`: the group `logo` is the logo with the bands' five colours. The logo is also paired with ink, Learn's frame and the nets' three colours. Only the pair logo and accent is exempt (`SAME`), and only while the two are equal. A self-check moves line, learn, net-dominator and accent next to the violet and expects each to fail. A live run with `--color-line` set to `#7050c8` failed with `logo: logo and line in full colour` and the three deficiencies; the token was then restored. |
| Guard 3: the rule line | done | The client's `.claude/rules/web.md`: "The logo's colour is fixed …", with the exemption and the self-check. The section "The logo" covers Figtree's OFL condition 5. |
| CC BY 4.0 in the licence check, the licences list, the README; About's line | done | `modules/_checks/licences.py`: every image the site serves is under `logo/`, beside `logo/LICENSE.txt`, which names CC BY 4.0 with its link, and every logo SVG has its text outlined. The check printed "the 12 images are the logo's under CC BY 4.0". About lists "linlog's logo: CC BY 4.0" (`/logo/LICENSE.txt`) and says "The name linlog and its mark identify this project, and may not be used to suggest its endorsement of anything else." The client's README and CLAUDE.md say the same. |
| The client's `nix flake check` at its last commit | done | At `tvkpptwy` (2bd9b5f0), under the machine lock, exit status 0: e2e 81 passed, unit 122 passed, licences, csp and the other checks. The first run failed Biome's `useAnchorContent` on the 404 page's link, which was fixed in the commit (the drawing is a child `Fragment`) before the second run. Again at the last commit, `uwxqplov` (405bc502, the rules' line on the mark's height), exit status 0. |
| "client ready" to planning before linlog's part ended | done | SendMessage to `planning` with the commits and the URLs; the supervisor pushed the client (its `main` is `tvkpptwy`). |
| linlog: README's head, the rustdoc's logo and favicon, the Rocq library's README; no logo file | done | README's first line is a `<picture>` with the dark and light word marks by absolute URL from the site. The Licence section says where the logo lives and under which licence. `core/src/lib.rs` has `#![doc(html_logo_url = …/logo/square-dark.svg, html_favicon_url = …/logo/favicon-32.png)]`. `rocq/README.md` is new, with the word mark. No SVG or PNG was added (`jj diff --stat` of the code commit). |
| Every linlog code commit gated; `nix flake check` at the last commit | done | The one code commit, `yxtzrqkk`, passed `gate` (exit status 0) before it was committed; `nix flake check` ran at this section's commit (its exit status in the message to planning). |
| The author's files under `linlog-logo/for-github/` | done | `organisation-avatar.png` (512), `social-preview-linlog.png` and `social-preview-linlog-prover.github.io.png` (1280 by 640), `zenodo-community.png` (512) and `CHECKLIST.md`, copied from the client's `nix build .#logo` at its committed head. |
| Screenshots before and after | done | `/home/tux/Projects/own/linlog-step38g-screenshots/{before,after}/`: `header-`, `about-`, `404-`, `tab-` (the favicon in a mock tab strip, with the PNG fallback beside it), `preview-` (a mock link preview from the page's Open Graph tags) and `page-`, each at `phone` (412 wide) and `desktop` (1280 wide) in `light` and `dark`. That is 24 files on each side, the same names. Taken by a scratch Playwright script in the client's devshell's Chromium; `after/` from the site at `uwxqplov`, with the 30-pixel mark. |

### Outcome

On the site, the mark replaces the plain word in the top band at 30
pixels high. The letters are in ink and the ring is in the logo's own
violet, drawn inline, so a theme change repaints it without a redraw.
About and the 404 page show it at 32 pixels. The browser tab shows the
square form (`⊸` lying down) on the dark field, as an SVG with a 32-pixel
PNG fallback. The manifest, the Apple touch icon and the link preview
take the flake's rasters. The logo's colour cannot follow a later change
of the accent: it is a token of its own, and the colour-vision test fails
any colour of meaning that comes near it. In linlog, GitHub's and
crates.io's README, the rustdoc's sidebar and favicon (on the Pages
documentation and on docs.rs) and the Rocq library's new README fetch
the files from the site. No logo file is in linlog.

### Decisions

- **Where the rasters are made**: a derivation of the client's flake
  (`modules/logo.nix`) running a Python script with nixpkgs' resvg, as the
  brief asks. Before, the build drew the old icon with linlog's own
  WebAssembly renderer in Node; that path is gone, along with
  `public/icon.svg` and the `public/` directory. The build copies the
  derivation's files the way it takes linlog's fonts, through `.cache/`.
- **The sources under `src/logo/`, served at `/logo/`**: they are
  imported raw by the components (Vite does not import from `public/`),
  and the build copies them. `/logo/` holds every file of the logo, so
  one notice beside them covers them and the licence check has one place
  to look.
- **The dark square everywhere a square is small** (the favicon, the
  Apple touch icon, the manifest's icons, the rustdoc's logo, GitHub's
  avatar): it reads on light and dark backgrounds alike, as the old icon
  did, and a favicon cannot follow the page's theme without a second set
  of links whose `media` browsers honour unevenly.
- **The maskable icon** scales the figure to 0.8 about the centre. The
  figure spans 77 per cent of the square, and its ring reached 208 of the
  205 pixels a launcher's safe circle allows.
- **The band's mark larger than the band's words**: the first
  screenshot at 18 pixels showed the ring as a dot, as session 38f
  expected, so this session set 24 (`--size-wordmark`). The author then
  asked for 30, which the supervisor committed (`rnzzvnuk`, the header
  test's height following); this session's last client commit puts 30
  in the rules. On a phone the mark stays hidden, as the plain word was.
- **The link preview and the social previews** are the dark word mark,
  640 pixels wide, centred on the dark field, with no text beside it:
  text would need another outlining tool in the client's flake. Both
  repositories get the same image.
- **The README's head is the heading itself** (`# <picture>…`) with the
  alt text "linlog", so the page keeps one first-level heading. The
  `<source>` for the dark scheme serves GitHub's dark theme. crates.io's
  sanitiser may drop `<picture>`, which leaves the light `<img>`.
- **The rustdoc's favicon is the PNG** and its logo the SVG square: PNG
  is the format every browser takes as a favicon.
- **The colour guard's shape** (with the supervisor's correction): one
  group `logo` (the logo and the bands' colours, every pair of which
  passed already). The logo is also paired with ink, Learn's frame and
  the nets' colours, one pair each, because a single group would also
  compare those colours with one another, which no rule asks for: Learn's
  purple and the dominator lines fail each other and are never shown as
  rivals. The exemption holds only while logo and accent are equal, so an
  accent moved near the violet but not onto it fails.

### Judgement calls

- The SVGs' title and `aria-label` say "linlog", not "linlog (violet)":
  the one change made to the chosen files, metadata only.
- The About test now finds two links labelled "CC BY 4.0" and eleven
  local licence links. It picks the written content's link by its list
  item. This changed because the behaviour changed (a licence entry was
  added), which the commit says. The header's mark is found by its
  element, since the top band is inside `main` and has no banner role.
- No `favicon.ico`: the pages link their icons, and resvg writes no ICO.
- The Rocq library's README is new (there was none). It says what the
  library holds, its layout, how to build it, and its licence, from the
  rules file and the root README.
- linlog's README's Licence section gains a paragraph on the logo,
  so that the CC BY 4.0 notice travels with the README that shows it.

### Departures from the design

- None from D1 to D60 knowingly. The header's mark is drawn larger than
  the band's word size (30 pixels where the band sets its words at 18),
  which session 38f left to this session and the author settled.

### The author's checklist

The files are under `/home/tux/Projects/own/linlog-logo/for-github/`,
whose `CHECKLIST.md` repeats this list:

1. The organisation's avatar: `organisation-avatar.png`, in the
   organisation's settings, "Profile picture".
2. linlog's social preview: `social-preview-linlog.png`, in linlog's
   settings, "Social preview".
3. The site's social preview: `social-preview-linlog-prover.github.io.png`,
   in the site repository's settings, "Social preview".
4. Optionally a Zenodo community with `zenodo-community.png`.
5. Once the site is deployed, check that
   https://linlog-prover.github.io/logo/wordmark-light.svg answers, so
   that README shows the logo on GitHub and crates.io.

### What is left open

- crates.io's handling of `<picture>` is not verified here; at worst it
  shows the light word mark.
- The README, the rustdoc and the Rocq README show nothing until the site
  serves `/logo/` (the supervisor is deploying it).

### Repositories and workspaces

- The client, `linlog-prover.github.io`, on top of `zxqpkpnt`:
  `onplpnxq` (dc16f68d) "Draw linlog's logo from its SVGs in the flake";
  `tvkpptwy` (2bd9b5f0) "Place linlog's logo on the site, its colour
  fixed", both pushed by the supervisor; the supervisor's `rnzzvnuk`
  (6188d4ae) "Draw the header's word mark at 30 pixels, its ring legible
  in the band", pushed; `uwxqplov` (405bc502) "Say in the rules that the
  band's word mark is 30 pixels high", not pushed.
- linlog: `../linlog-step38g` on `ulqqvuxr`: `yxtzrqkk` (d1e2f6fd) "Show
  linlog's logo in README, the rustdoc and the Rocq library's README",
  gated (`gate: passed`, exit status 0, on cores 8 to 11); then this
  section's commit. Nothing pushed.

## Session 38h: invertible rules applied automatically

The session of `plan/38-web-polish.md`'s section "Session 38h:
invertible rules applied automatically" (the brief of 2026-10-23), on
Opus 5.5 at `high`, 2026-10-23, in linlog's workspace `../linlog-step38h`
and the client's `../linlog-prover.github.io-38h`. The author asked for
the automatic application of the invertible rules, off by default, with
a toggle and an explanatory tooltip, as Click & coLLecT's "auto-reverse"
does. linlog's part was gated, checked, reviewed and pushed by the
supervisor (`main` at `fe008c50`); the client's input moved to it.

### The checklist

| item | state | evidence |
|---|---|---|
| One library call applying every rule invertible in the goal's mode, to a goal or every open goal, until none applies, each an ordinary step | done | `Interactive::invert(goal, &limits)` and `invert_all(&limits)` (`core/src/proofs/interactive/invert.rs`), each step through `record`, which `apply` ends with too; `Inverted` says the steps, the goals left open and the refusal of the memory bound if one stopped it. `is_invertible(sequent, &step)` judges one step. |
| The invertible set of each mode in the rules file, from sources read | done | `.claude/rules/core-derivations.md`, "The invertible rules applied by themselves": the set, its names per mode, and per rule where it was read or how it was derived (below). |
| The conditional rules' conditions checked | done | `!` only when it is the one formula that is no `?`; `1`, `⊗` and the axiom only when their formulas are the whole goal (`invertible_rule`), and every candidate validated by `expand`. `the_invertible_rules_under_their_conditions` pins `⊢ 1, a`, `⊢ !a, b`, `⊢ a ⊗ b, c`, `⊢ a, ~a, b` (affine) and `⊢ a, ?b` as taking no step. |
| Never a provable goal into an unprovable one, on generated sequents of every mode | done | `inverting_keeps_every_proof_and_ends`: classical, affine, Mix, both (the 16 rule sets of `generate::provable`), intuitionistic and its affine variant (the 12 of `generate::ill`), `L`, `L*` and the cyclic mode (`generate::lambek`); 568 sequents the search proves, and the search proves every goal the steps leave; every rule of the set fires in the classical, Mix, affine and both intuitionistic modes, and `⅋` and the axiom in the ordered ones (asserted). |
| It ends | done | The same test checks that every premise of every step is lighter than its conclusion and that a second run takes no step; `inverting_stops_at_the_memory_bound` reaches the bound (`⊢ a & a` six times, 4 KiB) and gets `Refusal::Memory`, its steps undone one by one, and the unbounded run takes its 63 steps. |
| A completed proof made with it passes the checker | done | The generated test closes the rest by the search and calls `Interactive::proof` (the checker) on each; the fixed cases check the proofs the steps finish alone. |
| One options value, off by default; `interact`'s flag and command; the bindings' setting | done | `proofs::interactive::Options { invertible }` (`DEFAULT_INVERTIBLE` false) is `Settings::interactive` (`{"interactive": {"invertible": true}}`). `linlog interact --invertible`, the command `invertible on|off`; the bindings read `interactive.invertible` in `start` and `apply` and add `invert_all`. `each_flag_overrides_its_key_in_a_file` pins the flag's key; `the_invertible_rules_apply_by_the_setting` the bindings. |
| README's usage with real output, run by `cli/tests/readme.rs`; the changelog | done | README's `interact` section has the transcript of `linlog interact --invertible "A -o B, B -o C \|- A -o C"`, which `readme_examples` runs; the status section names the feature; CHANGELOG's `[Unreleased]` entry. |
| Every linlog code commit gated; `nix flake check` at the last commit; "linlog ready" to planning | done | `gate: passed` (exit status 0, cores 4 to 11) on the tree of the lock's commit; `nix flake check` there: "all checks passed!", exit status 0. SendMessage "linlog ready" with the commits; the supervisor pushed them. |
| The page: the toggle, off by default, kept with the other settings, its tooltip | done | Settings' switch "Apply invertible rules by themselves" after Budget, stored with the settings; `tests/invertible.spec.ts`: off by default, kept over a reload, turning it on applies the rules at once; its tip. |
| Undo of the automatic steps | done | `retract` with the view's `invertible` takes back the nearest step that is not invertible with all since (`steps.test.ts`), and the Playwright test undoes `⊕₁` with the `⅋` and the axiom after it, and says so where every step on the branch was automatic. |
| The hint ladder taking it into account | done | Rung 1 reads linlog's `Hint.invertible` and, with the switch on, says the invertible rules are done (`words.test.ts`, through the real module: `!L` is not invertible, `\R` is; the Playwright test of the rung). |
| Screenshots before and after | done | `/home/tux/Projects/own/linlog-step38h-screenshots/{before,after}/`: `{desktop,phone}-{light,dark}-{01-settings,02-tooltip,03-proof}.png`, twelve each side, the same names; `shots.mjs` beside them made them from the sites built at `zszotswt` and at the client's commit. |
| The fresh-context review, each finding answered | done | One round, below. |
| The client's `nix flake check` at its last commit; `jj st` clean in both | done | At `ttyrqwmr` (45d839d4), under the machine lock: "all checks passed!", exit status 0 (e2e 92 passed, Vitest 485). linlog's at this section's commit, in the message to planning. |

### Outcome

A session can now apply every invertible rule by itself. The library's
`Interactive::invert` and `invert_all` take, until none applies, the
steps of the rules whose premises are provable whenever their
conclusion is, each an ordinary step that `undo` retracts and the
checker sees in the finished proof. The command takes `--invertible`
and `invertible on|off`, and its `undo` takes back a step with the
invertible steps that followed it. On the page, Settings' new switch,
off by default, applies them when a proof starts, after each step and,
when it is turned on, at once to every open goal; undo takes back the
visitor's last choice together with them; the hint ladder's first rung
knows whether the next rule is invertible from linlog itself and, with
the switch on, says the invertible rules are done.

### The invertible set and its sources

The same rules in every mode, each step then validated by `expand`
(which refuses what a mode or the intuitionistic reading does not
allow): `⅋`, `⊥`, `&`, `⊤`; `!` when every other formula is a `?`;
`1`, `⊗` and the axiom when their formulas are the whole goal. Two-sided
in intuitionistic mode these are `⊸R`, `⊗L`, `&R`, `⊕L`, `1L`, `⊤R`,
`0L`, `!R` (every hypothesis a `!`), `1R`, `⊗R` with no hypothesis and
the axiom; in the Lambek calculus `\R`, `/R`, `⊗L`, `⊗R` with nothing to
split and the axiom; in the cyclic mode `⅋`, `⊗` alone and the axiom.
No `?` rule (dereliction and weakening are not invertible, contraction
never ends), no Mix and no `wk`. It is Click & coLLecT's set, read in its
`proof.ml` (`apply_reversible_rule`).

- **Read**: Andreoli's asynchronous connectives (`⊥ ⅋ ? ⊤ &`), whose
  right rules are always invertible, in Liang and Miller (TCS 410, 2009,
  §2) and Miller and Saurin (CSL 2007, §2), since Andreoli 1992 itself
  could not be fetched (OUP and ScienceDirect refused, no open copy);
  ILL's right-asynchronous `& ⊤ ⊸` and left-asynchronous `⊗ 1 ⊕ 0` in
  Chaudhuri's thesis (CMU-CS-06-162, §6.1); the method, the premise
  derived from the conclusion by a cut with axioms, in Curien's notes
  (arXiv cs/0501035, the reversibility of `⅋` and `&`).
- **Derived from those, and said so in the rules file**: the affine
  modes, Mix, the Lambek calculus and the cyclic mode by the same cut,
  which each admits (no source stating them was obtainable); `!` with a
  `?`-only context by a cut with `⊢ ?A⊥, A`; `⊗`, `1` and the axiom on
  the whole goal by the last rule of a cut-free proof, since a
  one-formula sequent `⊢ A ⊗ B` ends in `⊗` with empty contexts when the
  empty sequent is underivable and Mix needs two nonempty premises.

### Decisions

- **One set, no choice of sets in the options**: the rules that close a
  goal (the axiom, `1`, `⊤`) and the conditional ones are forced moves,
  and focusing's asynchronous set alone would leave the axioms to click.
  The options value holds the one switch, and the set is in
  `is_invertible`'s documentation.
- **The bound is the memory one**: `&` copies its context, so a goal of
  `n` formulas `a & b` makes `2ⁿ` goals; the bytes of the inferences a
  run adds count against `limits.memory_bytes`, and the bindings use the
  smaller of it and `limits.derivation_bytes`, so that no run makes a
  session the page cannot draw.
- **Order**: a rule that closes the goal first, then the first position's
  rule, depth first; which order is taken changes the tree, never
  provability.
- **Undo, how and why**: in the library each step is ordinary and `undo`
  retracts one. The command groups a user's step with the invertible
  steps after it by its step counts, and its `undo` takes the group back
  whole. The page derives everything from the session document, which
  does not say who took a step, so with the switch on its undo per branch
  goes past every invertible step to the nearest that is not, and takes
  that back with all since: the visitor's last choice. Taking back only
  an automatic step would be undone by the next step (the page would
  apply it again), which is the surprise to avoid. An invertible step the
  visitor took by hand before turning the switch on is passed over too;
  where every step on the branch is invertible the note says so.
- **Replays run with the setting off**: the steps are in the session as
  they were taken, and a replay that applied the rules after each would
  find the recorded ones' goals closed.
- **Where the toggle sits**: Settings, after Budget, as the proving
  behaviours' place (with `frontend-design`): the page's `.sw` switch,
  its tip in two sentences, and a line under it, since tips never follow
  a touch.
- **The hint's invertibility from linlog**: the bindings' hint says
  whether its last step is invertible (`is_invertible`), and the page's
  `ruleKind` reads that instead of a list of names.

### Judgement calls

- **A fix of an existing bug** (the supervisor's addition): the page's
  first rung called a dereliction (`!L`) "an invertible rule" and missed
  the Lambek calculus's `\R` and `/R`, from a list of names. It now
  takes linlog's judgement, the single source; `words.test.ts` checks
  `!L` and `\R` through the real module.
- `invert` and `invert_all` are two calls, as `close` and `close_all`
  are, both over one private run.
- Turning the switch on applies the rules to every open goal of every
  proof on the page, not only the visible one, so that no tab waits for a
  step to catch up.
- The command prints `applied N invertible steps` when a session starts
  with them, and `then N invertible steps` before the goals a step left.
- The behaviour lock's change (only `interact`'s help) is a commit of
  its own after the code's, as earlier blesses were, so the code commit
  alone fails the lock test; the gate passed on the tree of the second.
- The view's new key `invertible` and the answers' `inverted` are keys of
  a call's answer, no stored document: no wire level.

### Departures from the design

- None from D1 to D60 knowingly. D13's undo per branch is kept, with
  the switch on taking back the branch's last step that is not
  invertible; D12's first rung keeps its words for "an invertible rule"
  and adds the switch's sentence.

### The review

One fresh-context reviewer on Opus 5.5 at `high` read the set against
the sources and the derivations, and the tests, reporting only what
breaks a requirement. It found nothing that breaks one: it checked the
reading's positions (`1` and `!` only in output position, so only `1R`
and `!R` fire; an input `⊗` alone is refused by the succedent check),
the `⊗`-alone argument in the affine and Mix modes, and the bound. It
made `⊗` with a context, `⊕` and dereliction invertible in a copy, and
the generated test failed in every mode where `expand` lets them
through. Its one remark, a limit that breaks nothing: the affine modes
are fed only sequents provable without weakening; it is recorded in the
rules file.

### What is left open

- The page's Lambek and cyclic modes do not exist yet; the switch will
  serve them as it is, linlog's set covering both.
- A run stopped by the memory bound says so in the command; the page
  shows the steps taken and no message of its own (the answer's
  `inverted_refused`).

### Repositories and workspaces

- linlog, `../linlog-step38h` on `twllnuzt`: `qlukuulo` "Apply the
  invertible rules by themselves in an interactive session" and
  `vmklwpyw` "Bless the behaviour lock for --invertible", pushed by the
  supervisor (`main` at `fe008c50`); then this section's commit.
- The client, `../linlog-prover.github.io-38h` on `zszotswt`:
  `tztzrqyv` "flake.lock: Update" (linlog's input to `fe008c50`) and
  `ttyrqwmr` "Apply the invertible rules by themselves, a switch in
  Settings". Nothing pushed.
