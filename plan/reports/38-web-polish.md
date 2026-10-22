# Step 38: the web client's polish

## The checklist

| item | state | evidence |
|---|---|---|
| 11. linlog's latest: the input at `main`, the saturation refutation shown as the other checked ones | done | the client's `yulwlqwo` (`flake.lock`: `linlog` 8bdc8605 → 94ff155a, `trzqnvtx`); "Unprovable, checked: saturation" in `src/lib/words.ts`; Vitest `a verdict … names a saturation refutation` (the real module: kind `saturated`, version 3) and Playwright `a saturation refutation is shown checked, by the inverse method` |
| 1. Touch targets, measured, with a test that fails below the floor | done | `tests/targets.spec.ts`: every visible control in 23 states (24 on a phone, with More), 44×44 at a phone's width (Pixel 7), 24×24 at a desktop's (1280×800); it failed first (on a phone the segmented buttons 38 px high, the file input 18 px, two licence links 42 px wide; on a desktop the file input), each fixed at its cause, then passed |
| 2. One spacing scale in the tokens, used everywhere; spacing fixed from a screenshot pass | done | `src/styles/tokens.css` (2, 4, 6, 8, 12, 16, 24, 32, 48, 96 px); `src/styles/tokens.test.ts`; the passes below |
| 3. One visual grammar, with a check that fails on a raw value | done | one control rule with its states in `app.css`; `tokens.test.ts` fails on any raw length, colour, duration, weight, opacity or layer in `app.css`, on an icon's `size` or a length in a string in the island, and on a second breakpoint (shown failing on the island's icon sizes and the shake's pixels before they went) |
| 4. A more modern look within "Instrument" | done | translucent clusters with a top sheen, the radius hierarchy, segmented controls and switches redrawn, outlined formulas of the active goal; the screenshots |
| 5. Ease of use, walked at both widths | done | below, "Ease of use" |
| 6. The two-sided toggle in classical mode alone | done | `Calculus.tsx`; Playwright `the calculus's switches restart the proof after a warning, and two-sided shows in classical mode alone` |
| 7. The engine of every search beside its work, its source one gesture away, each reference checked | done, with the gaps said below | `src/lib/engines.ts`, `texts/reference.md` ("The engines"); Playwright `a search names its engine beside its work, with its source on hover and in Learn`; the references' check below |
| 8. The engine choice in Settings | built | Settings → Engine; Playwright `Settings forces an engine: …` |
| 9. Practice along D60, graded, interleaved, decided by the command, sessions saved | done | 52 sequents in 8 lessons; `practice-decided.txt` (52 `linlog prove` outputs, 29 `linlog interact` sessions with `proof` exit 0); Vitest `examples.test.ts` (each verdict and session, and the interleaving); Playwright `practice is grouped by lesson, …` |
| 10. The sweep | done | fixes and judgement calls below |
| Before-and-after screenshots, the reviewer's findings answered | done | `/home/tux/Projects/own/linlog-step38-screenshots/` (`before/`, `after/`, 76 each); "The review" |
| The client's `nix flake check` at its last commit | passed | exit 0 at `zpyotloq` (57167ab6), under the machine lock, "all checks passed!" |
| Any change of linlog gated | no change of linlog's code; this report and `plan/later.md` only | the commit of this report; `nix flake check` at it, under the machine lock; its exit status is in the message to `linlog-ef` |

## Outcome

The page has one set of tokens and one grammar for every control, and a
check that fails on any value outside the tokens. Every control is at
least 44 by 44 pixels on a phone and 24 by 24 on a desktop, measured in
every state by a Playwright test. On a phone the bottom band keeps its
common actions and More holds the rest with the calculus's switches. The
two-sided switch shows in classical mode alone. Every search names its
engine beside its work, with its source on hover and in Learn's reference,
whose references were checked against the works. Settings can force an
engine, offering those that take the sequent. Practice has 52 sequents
along the lessons, provable and unprovable mixed, each decided by linlog's
command, each provable one with a session that Watch plays step by step.
The page reads linlog's `main`, and the inverse method's saturation
refutation shows as the other checked refutations do.

Nothing of linlog's code changed: the client's lock names `trzqnvtx`, as
the brief asks, and stays there. A bindings call for item 8 was written and
tested in `../linlog-step38`, then set aside (below).

The client's commits, on its `main` (`pnrwuykw`):

| change | what |
|---|---|
| `yulwlqwo` | linlog's input at `main` 94ff155a |
| `vxwvlwrk` | the practice set: 52 sequents, their sessions, the lesson in the schema |
| `kpmvzpuk` | the tokens and the grammar, the targets, More, the two-sided switch, the engines and their references, Settings' engine, practice by lesson, the field's Enter, the saturation's words, and the tests |
| `kruyqvqm` | what the screenshot pass found (below) |
| `xntxytuy` | no tip after a touch; README and the rules |
| `upzpwtwt` | the review's answers (below), and Biome's findings the flake's lint met (native radios, no export from a test file) |
| `zpyotloq` | the split chooser names its rule in its title |

## Decisions

- **The engine choice is built** (item 8): one radio group in Settings,
  "Engine", Automatic first with the engine linlog picks for the current
  proof's sequent ("here the focused search"), then every engine, those
  that do not take the sequent greyed with a short reason and linlog's
  sentence on hover; a forced engine's refusal is said at the goal ("The
  proof-net search does not take this goal"). Nothing on the stage or in
  the bands. It adds one fieldset to a pane the visitor opens, so it
  crowds nothing the expert sees.
- **Which engines take a sequent is asked through `prove`**, not a new
  call: a search forced to an engine with a budget of one unit refuses
  before it starts where the engine does not take the sequent (the same
  `prepare` that `search::engine_for` runs). The bindings call
  `engines(session, settings)` over `engine_for` was written and passed
  its test, but the client's lock may name only linlog's `main`, and the
  client's `nix flake check` must pass at its last commit with that lock,
  so a client that called it could not. Eight calls of `prove` at one unit
  each cost microseconds on the page's sequents.
- **An engine is named where its work shows**: the chip beside a goal's
  note (a refutation, an unknown) and beside the hint line, a label in the
  drawing beside the rule of a search's graft, the step list ("The focused
  search closed goal 2"), the live region, and the status's facts. A
  classical refutation is named the classical refuter's, which an outcome
  gives by its refutation's kind. The chip's hover gives the source in a
  line; a press opens Learn's reference at the engine, marked. A graft's
  engine is kept with the proof in the browser (`Proof.engines`); a link
  carries no engines, so a proof opened from a link draws its grafts
  without the label.
- **Practice's answer is kept until asked**: with provable and
  unprovable mixed, a card that says "No proof" before the student tries
  gives the answer away. Each card has Prove, and "Answer" opens the
  verdict, the explanation and Watch. The lessons are collapsible, the
  first open; there are 8 (lesson 9, ordinary logic, is not on the page).
- **The sessions follow linlog's hints**: a session closed by the search
  replays as one graft, which Watch would play in one step. So the steps
  are those linlog's own hints give (the module in Node,
  `steps.mjs`), and `linlog interact` applies them, saves the session and
  checks the proof (exit 0 for each of 29), as the client's rule says.
- **The phone's More**: 44-pixel targets leave room for seven in the
  band at 412 pixels; Undo, Redo, Close all, Hint, Export, Learn and More
  stay, and More's sheet holds the calculus's switches (D58: "the cluster
  folds into the phone's sheet") and Steps, Import, Settings, Tour, Keys
  and About.
- **A formula's button** covers half the gap to its neighbours and is at
  least a control high on the screen; the camera's fit never zooms out
  past the size at which the active goal's formulas are a control wide,
  and then shows the active goal instead of the whole tree. Zooming out by
  hand can make them smaller: the visitor's own choice, which the
  measurement does not cover.
- **The breakpoint** (640 px) cannot be a CSS variable inside a media
  query; the check holds every media query and the island's `phone()` to
  one width instead.

## The references (item 7)

A research sub-agent (Sonnet 5.5, `medium`) fetched and read each source;
where a work is paywalled it read a copy or a secondary text and said so.
What rests on what:

| engine | reference | read |
|---|---|---|
| focused search, two-sided | Andreoli 1992, JLC 2(3) 297–347, doi:10.1093/logcom/2.3.297 | the paper (CMU copy): "a subclass of proofs, called the 'focusing' proofs, which is both complete … and tractable" |
| proof-net search | Girard 1987, TCS 50(1) 1–102, doi:10.1016/0304-3975(87)90045-4 | the paper |
| its criterion | Danos and Regnier 1989, AML 28(3) 181–203, doi:10.1007/BF01622878 | bibliographic record (Crossref); the criterion as a later paper states it, citing them (Nishimuta and Okada, arXiv 1804.02540) |
| essential nets | Lamarche 1994, preliminary report, Imperial College | no copy found; cited with its theorem by Moot (arXiv 0805.2303) |
| | Murawski and Ong 2000, LICS 181–191, doi:10.1109/LICS.2000.855768 | the LICS abstract (dominator trees, linear time); the TOCL 2006 version was not reachable, so the page cites the LICS paper |
| planar nets | Lambek 1958, AMM 65(3) 154–170, doi:10.1080/00029890.1958.11989160 | bibliographic records only (two bibliographies read) |
| | Lamarche and Retoré 1996, Proofs and Linguistic Categories, CLUEB, 241–262 | the paper (the author's PDF, linked); the volume's data from two bibliographies |
| | Yetter 1990, JSL 55(1) 41–64, doi:10.2307/2274953 | record and an extract (cyclic linear logic) |
| Horn engine | Kanovich 1995, APAL 75(1–2) 107–135, doi:10.1016/0168-0072(94)00060-G | the abstract as the Petri-net bibliography gives it (Horn programs, Petri nets); the full text not reachable |
| | Abdulla, Čerāns, Jonsson, Tsay 1996, LICS 313–321 | the paper's extended abstract: "The reachability problem is solved by a backwards reachability analysis", upward-closed sets by their minimal states |
| inverse method | Chaudhuri and Pfenning 2005, CSL, LNCS 3634, 200–215, doi:10.1007/11538363_15 | the paper (CMU copy) |
| additive path | Whitman 1941, Ann. Math. 42(1) 325–330, doi:10.2307/1969001 | page range and Whitman's condition from a paper that states them (arXiv 2310.03366); that the additive path is the free-lattice order is this session's reading, which no linear-logic source found states |
| classical refuter | Girard 1987 | the paper: no sentence states that provable sequents are classically valid; its §5.1 makes the erasure argument for the intuitionistic case. The page says what the refuter does and cites Girard for linear logic itself |

The sub-agent disclosed that, in its second round, it sent the author's
e-mail address to the Unpaywall API, which asks for one as a parameter;
nothing else was sent.

## Ease of use (item 5)

Walked at both widths, in the screenshots' scenes and the tests:

- Prove a sequent: type, Enter (one step); Enter while a search runs now
  waits for it instead of being dropped.
- Undo: U, or the band's first button; a goal's refusal has its own Undo,
  which no longer shows beside a verdict, where there is nothing to undo.
- Close a goal: the dock's Close goal, or C; every goal: Close all.
- Hint: H, or the band; the hint line names its engine.
- Export: the band (desktop and phone), one menu.
- Open a practice example: Learn, the lesson, Prove (two steps for the
  first lesson, which is open, three for the others).
- Switch the calculus: one press on the stage at a desktop's width (with
  the restart warning after a step); two on a phone (More, the switch).
- Affordances: the active goal's formulas are outlined before a hover,
  since a touch screen has none; a practice card's actions are outlined
  buttons; the tooltips follow a mouse or a keyboard and never a touch,
  where they covered the goal.

## The sweep (item 10)

Fixed, being clearly wrong:

- The practice cards wrote `~A`; they write `A⊥`, as the drawings do.
- Enter in the field while another action ran was dropped; it waits.
- Undo beside a search's verdict at the root, where nothing can be undone.
- The calculus mark and the fragment said "ILL ILL" in intuitionistic
  mode; the fragment shows only where it differs from the mark.
- The calculus popover said the switches are on the stage, which on a
  phone they are not; it says where they are on a phone.
- The dock's note line covered the goal it was about (the dock was placed
  before the note was drawn); it is placed again when the note, the hint
  or the search's state changes, and below the goal where the calculus's
  cluster leaves no room above.
- The phone's rule popover was placed before its rules arrived and then
  covered the stage's controls; it is placed again when they arrive.
- Tooltips after a tap covered the goal on a phone.
- The reference broke `A ⊗ B` over two lines.
- On a phone the editing field pushed the tab's close and `+` off the row.

Judgement calls, for the author:

1. The essential net's input side stays linlog's green (repainted at
   4.5:1), beside D26's "green only for a checked proof": it shows only on
   a finished, checked proof, but green there means a side, not a check.
   A neutral hue (the accent's) would keep D26 to the letter.
2. Practice keeps the verdict behind "Answer"; Watch, behind it too,
   would otherwise give the verdict away.
3. On a phone the calculus's switches are in More (D58's fold): a change
   of calculus is two presses there, one on a desktop.
4. The status popover writes the mode as linlog names it
   (`intuitionistic-affine`) where the tab's mark writes `ILL+aff`.
5. With Learn on, every label shown (D54), the bottom band packs its
   actions tighter to keep one row at 1280 pixels; a narrower desktop
   window still wraps it.
6. A proof opened from a link draws its grafts without their engine's
   label, since the link carries no engines.
7. A tab whose sequent did not parse stays open beside the next proof.
8. The additive path's reference (Whitman's free lattices) is this
   session's reading of what the engine decides.
9. The engines' names in sentences are lower case ("closed by the
   focused search"), standing alone in sentence case ("Focused search").
10. The dock's "Goal 2 (1 of 1 open)": the goal's number is the step
    list's and the formulas' names', so a goal keeps its number as goals
    close; the reviewer read the two numbers as contradicting.
11. A goal wider than a phone at the targets' size is panned rather than
    shrunk below 44-pixel targets.

## Departures from the design

- (departs from the design) **The calculus's switches are not on a
  phone's stage** but in More's sheet, as D58 words it ("folds into the
  phone's sheet"); the mockup's phone showed them on the stage.
- (departs from the design) **The bottom band on a phone** shows icons
  and More instead of the mockup's collapsed sheet of three actions, to
  keep seven 44-pixel targets in one row.

## The review

A fresh-context reviewer (Opus 5.5, `high`), briefed with the items, the
colours' meanings and conduct.md's rules on cores and memory (cores 12 to
15, no builds), read all 76 after screenshots and the befores it needed,
and reported 18 findings. Each answered; the screenshots were taken again
after the changes (the after set is the final one; the reviewer saw the
set at `xntxytuy`):

1. Settings' segmented tracks wrapped one segment, the others outside.
   **Holds**: a fieldset's legend is no grid item, so the control sat in
   the label column. Every Settings label now stands above its control.
2. The phone's empty tab cut its placeholder, and the field's focus ring
   met the mark. **Holds**: the field takes what the row's other five
   controls leave; the ring is drawn inside.
3. On a phone the finished proof and its graft's engine label ran off the
   right edge. **Holds**: a proof just finished is fitted whole, and the
   fit counts the engines' labels.
4. A long goal overflows both edges on a phone, and the `+` is cut.
   **Partly**: the goal is shown at the size at which its formulas are
   44-pixel targets, and a goal wider than the phone is panned (the goal
   navigator brings it into view); the tab's sequent is narrower now
   (30 % of the width), which keeps the `+` in view.
5. On a phone the hint line covered the goal. **Holds**: the dock is
   placed again whenever it grows (a resize observer).
6. In an intuitionistic goal the `⊢` straddled two formulas' outlines.
   **Holds**: a button reaches half the gap to a neighbour only across a
   comma's gap, not across the turnstile.
7. The split's "Passes" was green, which D26 keeps for a checked proof.
   **Holds** (and older than this step): it is the accent now.
8. The split chooser's `⊗` read as a close button, and the chooser covers
   the conclusion. **Partly**: the `⊗` is the rule's sign, which the
   title now names ("Split for ⊗"); the chooser sits under the goal's
   row by the design, so that every formula of the goal stays clickable.
9. The clusters and the "Proved" keyword showed the field through them.
   **Holds**: the clusters are nearly opaque, and a keyword on the stage
   is laid on the panel's colour.
10. About's spacing was uneven and its headings indented. **Holds**: the
    headings align with the text and the licences' list has no gap of its
    own; each licence link is still a 44-pixel target on a phone.
11. With Learn on, the desktop's band wrapped to two rows. **Holds**: the
    band packs tighter with Learn on and keeps one row at 1280 pixels.
12. The selected Reference tab had a box. **Does not hold**: it is the
    hover state, the mouse still over the tab in the screenshot.
13. Key caps and keyboard settings on a phone. **Partly**: the tips in
    the phone shots come from the screenshot script's emulated mouse; a
    touch raises no tip (`xntxytuy`); "Show keys beside actions" is no
    longer offered at a phone's width.
14. "Not derivable" greyed on a finished proof. **Holds**: it is not
    offered there.
15. "Goal 2 (1 of 1 open)" looks contradictory. **Judgement call** (the
    list's item 10): the goal's number is the one the step list and the
    formulas' names use.
16. The popover's "split" was faint. **Holds**: secondary text colour and
    the small size.
17. Settings mixed a label beside its field with labels above. **Holds**
    (with 1).
18. A wide gap before "Practice" and the line's full stop gone. **Holds**:
    the line is plain text again, with its full stop.

Held by the reviewer's reading: the touch targets on a phone, the
two-sided switch, the engine beside the note, the hint and the graft
(the phone's graft aside, finding 3), practice's difficulties, mixing and
`A⊥`, the saturation's wording, and the controls' places.

## Verification

- Vitest: 92 tests passed at `zpyotloq` (the token check, the saturation's verdict
  with the real module, the 52 examples and their sessions, the
  interleaving).
- Playwright: 49 tests passed against the built site at `zpyotloq` (5 new in
  `tests/engines.spec.ts`, 2 in `tests/targets.spec.ts`; four changed
  for behaviour this step changes on purpose: the two-sided switch, the
  step list's engine, and practice by lesson in `learn.spec.ts` and
  `phone.spec.ts`).
- `astro check` 0 errors, `biome lint --error-on-warnings` exit 0, `nix
  fmt` applied. A first `nix flake check` (at `xntxytuy`) failed in its
  `lint` check (three Biome findings: `role="radio"` for which native
  radios are meant, and an export from a test file); fixed at their
  cause in `upzpwtwt`, where the check then passed (exit 0).
- The client's `nix flake check`: exit 0 at `zpyotloq` (57167ab6), its
  last commit.
- The practice set: `/home/tux/Projects/own/linlog-step38-screenshots/practice-decided.txt`
  holds each `linlog prove --timeout 10s` output (linlog at `trzqnvtx`,
  `nix build` of this workspace) and each session's `interact` exit;
  `gen.py`, `practice.py`, `steps.mjs` and `shots.mjs` beside it are the
  scripts.
- Signing tested before every commit; every commit signed.

## What is left open

- The inverse method's turns rarely reach a tab, and the bindings call
  for the engines: `plan/later.md`, "Left by step 38".
- Learn's lessons (D56, D60), the reference's rule schemas and the hover
  explanations (D47) stay a content session's.
- The client's lock moves only with linlog's `main`; this step needs no
  move beyond `trzqnvtx`.

## Repositories and workspaces

- The client: `/home/tux/Projects/own/linlog-prover.github.io`, its
  commits above `pnrwuykw` (`main`), local, nothing pushed.
- linlog: `../linlog-step38`, at `trzqnvtx` plus this report's commit
  (with `plan/later.md`); no code changed.
- Screenshots and evidence: `/home/tux/Projects/own/linlog-step38-screenshots/`.
