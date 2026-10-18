# The web front end's visual reference: "Instrument"

This is the mockup the author chose on 2026-10-18 (`plan/web/design.md`,
"The choice"), copied from the design session's working directory
without its fonts: `fonts/` is not committed, because no third-party
file is. To view the page as designed, put the fonts back under
`fonts/`, each beside its licence file, as the section "Fonts" at the
end says; without them the page falls back to the system's fonts and
the drawings keep their layout through `textLength`. Everything else
(the data, the scripts, the screenshots under `shots/`) is as the
session left it. (third pass, to D44–D56)

**A precision instrument for proofs: one row of open proofs, a dotted stage for the work, a band of named actions with pictograms.**

## 1. The direction

The page is one proof on a stage, inside a single mental model (D48).
The top band is one row of about 44 px (D53): the open proofs as tabs.
Each tab is a proof: its verdict as a coloured dot, with the details on
hover or click; its sequent in Euler Math, which turns into the sequent
field when clicked; its calculus as a small mark (`LL`, `ILL`,
`LL+mix`, `LK`) whose popover holds the switches. The stage is the
work: a deep blue-ink field with a faint dotted grid, the derivation
glowing on it as the one bold element, and everything that acts on one
goal docked at that goal. The bottom band is what I can do with the
proof, each action a pictogram with a short label (D54). The page is
concise (D44) and plain (D55): sentence case, Figtree for every word,
Euler Math for the logic, Space Mono only inside the sequent field, keys
in the tooltips and the Keys pane unless Settings prints them. Learn
(D47) is the second bold element: an orchid frame around the window
with Tutorial, Practice and Reference; its lessons go worked example,
your turn, practice (D56).

## 2. The design plan

### Tokens

Unchanged colours from the second pass. Contrast ratios (WCAG):

| role | dark | light | on field (dark / light) | on panel (dark / light) |
|---|---|---|---|---|
| `--field` stage, field | `#0C1222` | `#E9EDF4` | | |
| `--panel` bands, clusters | `#151C30` | `#FBFCFE` | | |
| `--ink` text, the drawing | `#E3E8F4` | `#121A2E` | 15.2 / 14.8 | 13.8 / 16.9 |
| `--dim` secondary text | `#9BA5C2` | `#46506C` | 7.6 / 6.8 | 6.9 / 7.8 |
| `--faint` the search's labels (D10) | `#7D88A8` | `#5A6482` | 5.3 / 5.0 | 4.8 / 5.7 |
| `--accent` selection, active goal | `#9AA8FF` | `#3446C8` | 8.4 / 6.3 | 7.6 / 7.2 |
| `--ok` checked only | `#4CCB8A` | `#127A47` | 9.1 / 4.6 | 8.2 / 5.2 |
| `--warn` unknown, searching | `#F0B43C` | `#8F5600` | 10.0 / 5.1 | 9.1 / 5.9 |
| `--bad` refusal, unprovable | `#FF7A73` | `#BF2A2A` | 7.4 / 5.0 | 6.7 / 5.7 |
| `--learn` the Learn frame | `#D98BE6` | `#8E3A9E` | 7.8 / 5.5 | 7.1 / 6.3 |
| `--learn-wash` the frame's panel | `#2A1E3A` | `#F3E6F5` | ink on it 12.7 / 14.4 | |
| `--line` control borders (1.4.11) | `#5E6A90` | `#7D86A0` | 3.5 / 3.1 | 3.2 / 3.5 |
| `--edge` hairlines | `#2A3352` | `#B9C1D4` | decorative | |
| `--dot` the stage's dotted grid | `#2E3858` | `#A9B2C8` | decorative | |
| `--net-par` the net's `⅋` edges | `#6FB3EE` | `#0072B2` | | |

Type (D55): **Figtree** for all interface text, sentence case, 600
weight for labels, no capitals, no tracking; **Euler Math** for every
formula and rule name (the drawings, the tabs, the popover, the dock,
the frame); **Space Mono** only inside the sequent field, where ASCII
input is typed. Scale 11, 12.5, 14, 15 and 18 px; the drawing at 1000
SVG units = 30 px at zoom 100 %. Spacing 4 px base; radius 6 on
clusters and tabs, 3 on keys and buttons. Motion: 120 ms popovers,
220 ms a label settling, 380 ms the camera, one 240 ms refusal shake;
all 0 under `prefers-reduced-motion`.

Decoration: two bold elements only, the glowing tree on the dotted
field and the Learn frame (a mode). Everything else is the quiet panel
style.

### One grammar

- One cluster style (`.cluster`, `.panel`) for the dock, the camera
  control, every menu and popover, the tour caption.
- One label style: Figtree 12.5 px, 600, sentence case (`.kw-label`);
  a fact-bearing keyword (`.kw`) is the same face, outlined and tinted
  by its colour role.
- One hover rule: any element with `data-tip` or `data-key` shows the
  same tip on hover and on keyboard focus, the key as a small key cap
  after the words ("Undo on this branch U"). Keywords that hide a
  sentence are buttons, and their accessible name carries the sentence.
- No middle dots: "Goal 2 (1 of 2 open)", "ax refused: no dual
  partner", "Unprovable, checked: atom balance"; the licences in About
  are a list.

### The pictograms (D54)

Twenty-one symbols in one inline SVG sprite, drawn for this mockup in
Lucide's manner (24 px grid, 1.5 px stroke, round caps and joins, no
fill; nothing downloaded, the real build uses Lucide under ISC): undo,
redo, steps, close all, hint, export, import, settings, learn, tour,
keys, about, zoom in, zoom out, fit, sun, moon, close, add, and the
subject's own two: **tree** (two premise bars over a conclusion bar
with its label tick) and **net** (two axiom arcs over four literals).
Rules:

- Icon alone only for undo, redo, zoom in/out/fit, the theme toggle,
  close and add, each with an `aria-label` and a tooltip on hover and
  on focus.
- Every other bottom-band action is icon plus a short label at
  desktop width; with Learn on every label shows, undo and redo
  included; in the phone sheet every action is labelled.
- What acts on a goal uses the logic's own signs: rule names in Euler
  Math in the popover and the dock, never a generic icon.
- Settings → Keys → "Show keys beside actions" prints the key caps
  next to the labels again (scenes `keys-on`, `keys-off`).

### Layout

Desktop:

```
┌────────────────────────────────────────────────────────────────────────────┐
│ linlog [● 1, A&B, B⊸C⊢C ILL ×][● !A⊢A⊗!A LL ×][● A⊗B⊢B⊗A LL ×][+]    ☾ │ 44 px
├────────────────────────────────────────────────────────────────────────────┤
│ · · · · · · · · · · · · · · · · · · · · · · · · · · · · · · · · · · · · · │
│ · · · · [ax refused: no dual partner] [↶ Undo]       (dock at the goal)   │
│ · · · · [‹ Goal 2 (1 of 2 open) › | ✓✓ Close goal]                        │
│ · · · ·     ⋮          ⋮            ┌ Rules  B ⊗ A ┐ popover beside goal  │
│ · · · · ⊢ B⊥, B    ⊢ A⊥, A          │ ⊗      split │                      │
│ · · · · ───────────────── ⊗         └──────────────┘                      │
│ · · · ·  ⊢ A⊥, B⊥, B ⊗ A                    [Tree|Net ⊖ 140% ⊕ ⛶ ▭]     │
├────────────────────────────────────────────────────────────────────────────┤
│  ↶ ↷  ☰ Steps  ✓✓ Close all  ◌ Hint │ ⇪ Export ⇩ Import ⚙ Settings │ 🎓 Learn ◎ Tour ⌨ Keys ⓘ About │
└────────────────────────────────────────────────────────────────────────────┘
```

A tab in detail: `[dot][sequent, Euler][mark][×]`. Hover or click the
dot: the verdict details (fragment, calculus, engine, refutation and
its reason, the countermodel or the balance, certificate, "Try 200 000
units"). Click the sequent of the active tab: it becomes the field, in
place; Enter loads, Esc cancels. Click the mark: the calculus popover
(Linear or Ordinary, Classical or Intuitionistic, Mix, Affine, and the
restart warning). `+` opens a new tab already in edit mode. With no
proof open the page is that new tab: the field focused with the
placeholder `A * B |- B * A` and one line under it, "Type a sequent,
or open one from Practice", where Practice opens Learn at Practice.
A parse error underlines the place inside the field and puts its
keywords in that same line.

Phone (390 wide): the same one-row band; the tabs scroll sideways and
the active one is kept in view by scrolling the row only. The bottom
band is a sheet: Undo, Close all and Hint collapsed, everything
labelled when expanded. With Learn on the frame takes the lower half.

The goal list stays gone: choosing a goal is the dock's navigator, the
`[ ]` keys and a click on any dimmed goal.

## 3. Learning principles, and what on the page does each

- **Expertise reversal and cognitive load.** The default page is the
  expert's: one row, keywords, no prompts. Learn frames the window and
  adds everything for the student; off, none of it is on the page.
- **Split attention.** The popover opens beside the goal, the dock sits
  over it, reasons sit beside their inference.
- **Feedback.** A refusal is a red keyword at the goal; its elaborated
  reason is one hover or Tab away. A split shows "Passes" or "Cannot
  close" before it is applied. Verdicts are three-valued on the tab's
  dot, with the evidence in its details.
- **Hint ladder.** H climbs four rungs in the dock ("Hint 1 of 4"):
  goal and connective, formula, rule and split, then the bottom-out,
  which applies the step and asks why.
- **Worked examples and fading (D56).** Every lesson shows its three
  stages as a stepper: 1 Worked example (the transcript played step by
  step, "Predict the next rule on goal 1 (step 2 of 4)", then the
  reason pinned beside the new inference), 2 Your turn (the same proof
  stopped before its last steps, with a checklist that ticks as the
  student finishes it), 3 Practice (a sequent from the Practice
  rubric). Lessons: 1 Sequents and the axiom (`first`, then
  `ill-first`), 2 Invertible first (`mll`, then `first`), 3
  Exponentials (`exp`; the data has no second exponential proof, so the
  practice is the same sequent bare), 4 Two sides (`ill`, then
  `ill-first`).
- **Erroneous example (D56).** Lesson 5, "When there is no proof": the
  legal split that leads nowhere is on the stage, "Which step went
  wrong?" offers the two steps, with feedback on each; then "The
  refutation" closes goal 2, whose verdict appears at the goal, and
  the same reasoning refutes `A ⊗ B ⊢ A` outright.
- **Dual coding.** The Reference draws each rule as a schema with its
  one-sided and two-sided names; inside the frame a formula's hover
  gives its connective's reading and a rule in the popover its schema.
- **UDL.** Mouse, touch, the full keyboard; labels, icons and their
  names; light, dark, higher contrast; keys printable on demand.

## 4. The failures

| failure | where | what it says | points at | next |
|---|---|---|---|---|
| does not parse | under the field in the tab | "Unexpected * at column 4", "Expected a formula"; sentence on hover | a red mark under the place, inside the field, and the place selected | fix the input; the stage is untouched |
| not intuitionistic (`A \|- B, C`) | under the field | "2 formulas right of ⊢"; sentence on hover | the part after ⊢, marked in the field | move the hypotheses left |
| rule refused | dock at the goal; the formula shakes | "ax refused: no dual partner"; reason on hover | the formula | another rule; H |
| split refused (ILL) | split popover | "Not allowed"; reason on hover | the premises | send the succedent right |
| split that cannot close | split popover | "Cannot close", premises dashed | the premises | change it, or apply and learn |
| subgoal unprovable | dock at the goal | "Unprovable, checked: atom balance"; Undo | the goal | undo the split |
| sequent unprovable (balance, countermodel) | the tab's dot, red; its details | "Unprovable", "✓ Checked", refutation, reason, table, Certificate | the tab | Export → Rocq |
| unprovable, exhausted | the dot, red | "Not checked": no check and no certificate exist | the tab | none to certify |
| gave up | the dot, amber | budget, used, copy bound, engine; "Try 200 000 units" (⇧C) | the tab | retry; a pulsing amber dot and "Stop the search" while running |
| link not restored | keyword on the stage | "Proof not restored", why on hover | the stage | start again or import |
| import refused | Import panel, drop target | "Level 9 unknown: not imported" | the panel | export from this version |

## 5. The scenes

`?scene=NAME` with `&theme=light|dark` (and `&keys=1` to print keys),
or the dashed `scenes` control at the top right (key `` ` ``).

- Opening: `open` (the empty stage: field, placeholder, one line).
- 1: `mll-0`, `mll-pop`, `mll-1`, `mll-split-wrong`, `mll-split`,
  `mll-2`, `mll-done`, `mll-net`. 2: `dead-split`, `dead-refused`,
  `refusal-tip`, `dead-close`, `dead-undo`. 3: `close-before`,
  `close-done`. 4: `exp-0`, `exp-pop`, `exp-3`, `exp-done`, `ill-0`,
  `ill-pop`, `ill-done`. 5: `unprov-balance`, `unprov-classical`,
  `unprov-exhaustive`, `status-pop`. 6: `unknown`, `unknown-retry`.
  7: `parse-star`, `parse-missing`, `parse-ill`.
- 8 Learn: `tut-1`, `tut-2` (lesson 2, worked, predicting),
  `tut-2-worked` (a step played, its reason pinned), `tut-2-faded`
  (your turn), `tut-2-progress` (one of two steps done),
  `tut-2-practice`, `tut-3`, `tut-5` (find the wrong step),
  `tut-5-found` (answered), `tut-5-refute` (the refutation at the
  goal), `worked-predict`, `worked` (lesson 3), `practice`,
  `reference`, `learn-rule`, `learn-connective`, `learn`.
- 9: `hint-1` to `hint-4`. 10: `tour-1`, `tour-3`, `tour-4`, `tour-6`.
- 11: `exports`, `exports-link`, `exports-cert`, `steps`, `import`,
  `import-paste`, `import-level`, `import-drop`, `import-dropped`,
  `link-broken`, `settings`, `about`.
- 12 Calculus: `calc` (the mark's popover), `mode-warn`, `mode-ill`,
  `exp-net`.
- 13 Keys and phone: `keys`, `keys-on`, `keys-off`, `focus`,
  `focus-undo` (an icon-only button's tooltip on focus), `sheet`.
- 14: `wide`, `wide-net`, `mll-2`. 15: the sun/moon in the band,
  system in Settings. 16: `ordinary`.
- Several proofs: `proofs` (three tabs), `proofs-switch`, `new-tab`,
  `edit-tab` (the field in place of the tab's text).

## 6. Cost in the decided stack

Components (one Solid island): `ProofRow` with `ProofTab` (dot and
`VerdictDetails`, inline `SequentField`, `CalculusMark` and its
popover), `Stage` (SVG host, formula overlay, camera, minimap, drop
target), `GoalDock`, `RulePopover`, `SplitChooser`, `BottomBand` with
`ExportMenu`, `ImportPanel`, `SettingsPanel`, `StepsPanel`,
`AboutPane`, `KeyList`, `Tour`, `LearnFrame` (`Tutorial` with the
lesson stepper and the worked-example engine, `Practice`, `Reference`),
`Tip`, an `Icon` wrapper over `lucide-solid` plus the two drawn
symbols, and the worker client; the service worker and manifest
outside the island.

Cheap from the library: the SVG with ids and `desc`, rules per formula
with their two-sided names, `split_passes`, refusal reasons, verdicts
with their facts, the session and sequent JSON with levels, every
export. Built by the client: per-tab sessions and step trees, the
camera, dock and popover placement, the tip system with keys, the
in-field error mark, the URL codec, hint texts, the lesson engine
(worked, faded and practice stages that watch the session), the drop
handling, the service worker's scope.

Rough size: tokens, type and grammar 2 days; one-row proof row with
inline edit, calculus popover and verdict details 3; stage and camera
4; dock, popover, split 3; step tree and undo 3; per-tab storage 1.5;
Export, Import, Settings, About 2.5; icons and tooltips 1; hints 2;
tutorial engine and five staged lessons 5; practice 1; reference 2;
tour 1.5; offline and manifest 1.5; phone and accessibility 2.5;
Playwright 4. About 39 days for one developer, the bindings excluded.

## 7. Self-critique

Fixed after this round's screenshots: on the phone, keeping the active
tab in view scrolled the whole page sideways (now only the row
scrolls, and the window's grid column is bounded); the camera control
showed on an empty stage (now hidden until a proof is open); the goal
navigator read "Goal 2 of 2 open" for goal 2 at position 1 (now "Goal
2 (1 of 2 open)"); sentences starting with "A " set that word as a
formula (reworded).

Still weak: Export and Settings open over the tree; a schema tooltip in
the Learn frame can cover the popover's own heading; on the phone with
Learn on the stage is small; lesson 3's practice is its own sequent
bare, because the data holds one exponential proof.

Departures for the author's review: a **Steps** action in the bottom
band (D13 needs the step list a home; D48 does not list it), marked
"(departs from the design)"; a subgoal's verdict shows at its goal and
not on the tab's dot, which speaks for the whole sequent (departs from
the design if "one place" is read literally).

Not possible statically: the real search (the doubled budget has no
answer), the service worker, storage across reloads, steps off the
transcripts' paths, PNG and PDF except for `A ⊗ B ⊢ B ⊗ A`. Import
matches a session against the `state-N.json` files and a sequent by its
terms; the dead end's split state and the unknown goal are drawn by the
mockup where the data has no SVG.

## Files and licences

`index.html` (with the icon sprite), `style.css`, `app.js`, `data.js`
(generated by `tools/bundle.py` from `data/`), `manifest.webmanifest`,
`icon.svg`, `third-party-notices.txt`. Fonts in `fonts/`, all OFL 1.1
with their licence files: Euler Math, Figtree, Space Mono. The
pictograms are drawn for this mockup; no code or icon file is taken
from elsewhere. Screenshots in `shots/`.

## Fonts

Not committed; the page's `style.css` expects them under `fonts/`:

| file | font | licence | where it came from |
|---|---|---|---|
| `Euler-Math.otf`, `Euler-Math-OFL.txt` | Euler Math 0.75 | SIL OFL 1.1, no Reserved Font Name | `cli/fonts/` of this repository (CTAN's `euler-math` package) |
| `figtree-latin-wght-normal.woff2`, `figtree-latin-ext-wght-normal.woff2`, `Figtree-LICENSE.txt` | Figtree (variable) | SIL OFL 1.1 | the npm package `@fontsource-variable/figtree` 5.3.0 |
| `space-mono-latin-400-normal.woff2`, `space-mono-latin-700-normal.woff2`, `SpaceMono-LICENSE.txt` | Space Mono | SIL OFL 1.1 | the npm package `@fontsource/space-mono` 5.3.0 |
