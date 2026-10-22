# Step 38b: native zoom, the exports' fonts, light dismiss

## The checklist

| item | state | evidence |
|---|---|---|
| 1. Native zoom: a pinch and ctrl with the wheel zoom the drawing on the stage; nothing blocks the browser's zoom elsewhere | done | the client's `wvzrmkqt`; `tests/zoom.spec.ts`, two tests (desktop: ctrl with the wheel about the pointer and the check of every element outside the stage; touch screen: a two-finger pinch about the fingers, a one-finger pan, and the browser's own pinch on the top band zooming the page), passing |
| 2. A font for the rendered exports: an option of linlog (serde, a CLI flag, the bindings' settings), Euler Math the default and unchanged, STIX Two Math and two more, each table read from its file, each licence served; New Computer Modern Math too (the supervisor's addition) | done | linlog's `tvrwrqsp`, `zupynzoz`, `qzxqzvqk` (the descenders), `ormuvkok` and `spxtytpy` (New Computer Modern and its lock); `every_table_is_its_font_s` (skrifa reads each file), `fonts` and `every_style_option` in `cli/tests/cli.rs`, `a_font_is_chosen_by_its_name` and `an_export_is_set_in_the_font_the_settings_name` in the bindings; the snapshots and the behaviour lock unchanged but for `--help`; the `export` check, once per font, passed (exit 0) |
| 2'. The font chosen in the client's export, with a test; one PNG and one PDF per font | done | the client's `wrlrwoum` and `okxuulsz`; Playwright `the export menu chooses the font of the SVG, PNG and PDF, …`; `/home/tux/Projects/own/linlog-step38b-screenshots/exports/{euler,stix-two,garamond,asana,new-computer-modern}.{png,pdf}`, each PDF embedding its font alone (`pdffonts`) |
| 3. Light dismiss at both widths | done | the client's `xqyzqkpw`; `tests/dismiss.spec.ts`, a desktop test and a phone test, passing |
| 4. More complex provable practice problems (the supervisor's addition) | done | the client's `wpspsroq`: 21 problems, 14 provable at difficulty 3, two per lesson from the second, each with a near miss that is not provable; `practice-decided.txt` (21 `linlog prove` outputs, 14 `linlog interact` sessions with `proof` exit 0); Vitest `give every lesson from the second at least two provable problems at the hardest difficulty` and the per-example verdict and session tests |
| 5. The tab chip's spacing (the supervisor's addition) | done | the client's `vlktpknq`; the screenshots `01-chip` and `01-chip-closeup` at both widths in both themes |
| 7. Every copyright and copyleft obligation respected (the supervisor's addition; there is no item 6, New Computer Modern going under item 2 as asked) | done | "The licences", the table below; fixed: STIX Two's OFL text verbatim (`vurxrnok`), New Computer Modern's clause 6 notice, the client's check that every served font is linlog's file unmodified (`okxuulsz`), the creator named in the CC BY notice (`pusswryq`) |
| 8. The stage's font, a setting among the same five, Euler Math the default; the exports' font following it (the supervisor's addition) | done | the client's commit of item 8; Playwright `Settings sets the formulas' font: …`; `/home/tux/Projects/own/linlog-step38b-screenshots/stage-fonts/{desktop,phone}-light-stage-{euler,stix-two,garamond,asana,new-computer-modern}.png`; the targets test and the token check pass with it |
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
  `zupynzoz` (the lock blessed), `qzxqzvqk` (the descenders), this
  report's `lysrossv`, `ormuvkok` (New Computer Modern), `spxtytpy` (its
  lock) and `vurxrnok` (STIX Two's licence verbatim), on `zxxqmxmx`.
- The client: `/home/tux/Projects/own/linlog-prover.github.io`, its
  commits above `yxtvvlsm` (`main`), local, nothing pushed.
- Screenshots, exports and scripts:
  `/home/tux/Projects/own/linlog-step38b-screenshots/`.
