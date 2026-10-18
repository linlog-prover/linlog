# Web and desktop precedents for a click-to-prove logic page

Survey for the design of linlog's web front end. All sources were read on
2026-10-18. Marks show how well a claim was checked:

- **[full]** the paper was read as text by me (PDF converted with
  `pdftotext`).
- **[page]** a live page fetched through a summarising tool; wording is
  second-hand, and "not stated" means the page does not say.
- **[snippet]** a search-engine summary only; treat as a lead.

Unreachable: the HAL copy of the Click & coLLecT paper (bot wall), the
Pandora papers (404), the Jape paper's part II, the Lean Game Server page
(empty fetch).

## Summary: what the field agrees on

Tools that were evaluated with students agree on three things. First,
immediate local feedback beats delayed grading: the student should learn at
the step whether it was legal; the one before/after comparison (Kassel, 97
students with the tool, uncontrolled) points that way. Second, the student
has two different difficulties and the tool should separate them: applying
a rule correctly (which formula, which side, how the context splits) comes
first and a tool can remove it by construction; finding a proof is the
harder part, and aids for it (hints, red/green provability lights) trade
against students button-mashing. Third, trust and low-friction input decide
whether students stay: formula entry, lost work on reload, confusing error
text and inconsistent UI state are reported more often than logic
problems. There is no agreement, and no study, on whether to hide, grey
out or refuse inapplicable rules. Evidence is mostly self-reported
questionnaires (SUS 67 to 80) from one course each. There is almost no
linear-logic-specific evidence beyond Click & coLLecT's tracker and course
pages, and I found no interactive proof-net editor for the web.

## Tools

### Click & coLLecT (linear logic, sequent calculus)

- **What.** Interactive prover for classical linear logic sequents, by
  Etienne Callies and Olivier Laurent (ENS Lyon); TLLA 2021 paper "Click
  and coLLecT: An Interactive Linear Logic Prover" (HAL lirmm-03271501,
  not readable). https://click-and-collect.linear-logic.org/ ; code (OCaml,
  LGPL-2.1) github.com/ComputerAidedLL/click-and-collect. [page]
- **Clicks.** Formula and rule in one gesture: clicking the turnstile or an
  atom closes by axiom; clicking a connective applies its rule "if it is
  unique". Choices are resolved by the click target: for a plus, the
  sub-formula to keep; for a why-not, one click weakening, double click
  contraction, click on the sub-formula dereliction. A tensor's context
  split comes from formula *order*, so the student first drags formulas
  into place (exchange by drag, not a rule). Cut is optional: click
  between formulas, type the cut formula. [page]
- **Display.** Upward tree with rule names. "Auto-reverse" applies
  invertible rules for the student. An auto-prover (double-click a
  turnstile) paints it red (unprovable), orange (time-out) or leaves it
  black (no claim). [page]
- **Inapplicable rule.** Not described; I did not use the tool. Test it
  live before copying anything.
- **Failure.** Provability by turnstile colour. Parse errors not stated.
  Recursive notations disable full axiom expansion and cut elimination.
- **Undo/share.** Global undo/redo, click a rule name to undo it. "Share"
  makes a URL holding the proof (README: LZMA). Export: LaTeX, PDF, PNG,
  ASCII/UTF-8, and Coq, also for open proofs. [page]
- **Keyboard/phone.** Not stated; no touch issue among 50 issues seen.
  Drag-for-exchange makes touch a real question. Unverified.
- **Does well.** One-click rules, tutorial plus example list, step-by-step
  cut elimination (the winter-school sheet says "try to predict the result
  before asking the machine to reduce the cut"), Coq export.
- **Reported to confuse (tracker only, thin).** Exchange: issue 165 (open
  since July 2021) says displayed permutations, explicit-exchange export and
  proof-transformation mode disagree; 169 asks that the proof update when
  hypotheses are exchanged; 143 (open) mode confusion; 168 return to the
  start of a cut-elimination sequence; 173 parser precedence of
  multiplicative versus additive. The help warns "some proposed sequents
  are not provable!". Issue 174 (open, June 2023) asks for proof nets,
  interactive building and cut elimination; Laurent replies that drawing a
  net from a sequent proof "is not straightforward if the output should
  look good". Wiki roadmap: export to proof nets, two-sided and
  intuitionistic variants, mix. [page, issues]
- Inspired by Logitext; the Proof Tree Builder paper says it has "almost
  the same interface". [full]

### The Incredible Proof Machine (natural deduction as wired blocks)

- **What.** Joachim Breitner, ITP 2016 "Visual theorem proving with the
  Incredible Proof Machine"; http://incredible.pm/ ; github.com/nomeata/
  incredible; Haskell via GHCJS, runs offline. [full]
- **Does.** Drag blocks (assumptions, rules, conclusions), wire ports;
  mouse or touch; unification labels the wires. The graph is the proof;
  nothing is "clicked" as a formula or rule. Port graph, not a tree. [full]
- **Inapplicable rule.** Allowed and flagged: a wrong connection is
  coloured red with a warning symbol; a red port marks an incomplete proof;
  the conclusion turns green when all checks. Partial and wrong proofs may
  stay on the canvas deliberately. Checking is continuous and "gets a
  little sluggish with larger proofs". [full]
- **Undo/share/phone.** Undo/redo, zoom, SVG export, state in Web Storage;
  no share link in the paper; a smartboard with finger drag was used. [full,
  blog]
- **Does well.** No syntax to learn; task list (attempted yellow, done
  green); sessions that unlock a few rules at a time; tasks and logics by
  editing files. 13 pupils aged 13 to 20, 14 hours, UEQ top quartile on
  attractiveness, dependability, novelty; some found it "rather high" in
  difficulty. Best engagement evidence in the survey, but one workshop.
  [full]
- **Confusion (author's blog, 2016).** Pupils linked output to output;
  scope of local constants "not easily understood"; the strongest pupils
  finished predicate tasks but he doubts they understood quantifiers ("a
  mechanical intuition"); a reload bug lost proofs. [page]
- No controlled learning study found.

### Logitext (first-order sequent calculus)

- **What.** Edward Z. Yang. http://logitext.mit.edu/main and tutorial
  http://logitext.mit.edu/logitext.fcgi/tutorial (the https host has a
  certificate for another name; plain http works). "Powered by Coq,
  Haskell and Ur/Web". [full: tutorial text]
- **Clicks.** A formula in the sequent; the rule follows from connective and
  side. Clicking an atom on both sides closes the goal; clicking the
  context variable Gamma "will fail". Quantifier rules ask for a term; the
  invertible ones pick a fresh variable. A contraction button duplicates.
  Upward tree, bars over closed goals, reset by clicking the turnstile.
- **Inapplicable rule.** The tutorial says a click completes the proof or
  "gives an error"; the form of the error is not described.
- **Undo, share, keyboard, phone.** Not described.
- **Does well.** The tutorial: tiny live sequents, underlined terms with
  hover definitions, rules introduced one at a time.
- **Confusion.** The tutorial's own conclusion names the complaint "I
  clicked around and managed to prove this, but I'm not really sure what
  happened!". The Proof Tree Builder paper: clicks "change the proof state
  in cryptic ways" and it is "too easy once the user learns how", by
  clicking "random formulas until the proof is finished" (an opinion in a
  related-work paragraph, not a measurement). The tutorial warns that
  quantifier-rule order can leave you stuck. [full]

### Sequent Calculus Trainer (Kassel)

- **What.** Arno Ehle, Norbert Hundeshagen, Martin Lange; Java/JavaFX
  desktop, BSD-3; https://www.uni-kassel.de/eecs/tifm/software/
  sequent-calculus-trainer ; arXiv 1507.03666 (2015), 1803.01467 (2018).
  Not web. [full]
- **Clicks.** Pick a rule button, then click the operator in the formula;
  the part in that operator's scope is highlighted (teaches precedence).
  Quantifier and equality rules prompt for terms. Rule buttons carry the
  formal rule on hover. [full]
- **Inapplicable rule.** Applied and refused with an explanatory message
  and a hint; a subformula click gets a warning that rules act on
  top-level formulas. [full; the second from the Proof Tree Builder paper]
- **Failure/strategy.** Version 2: a traffic light per sequent (green
  valid, red invalid, yellow unknown, Z3 behind it); a rule that turns
  green to red says the last step lost the proof. The authors fear a
  counter-model is "a step too big" and plan a Socratic dialogue. Not
  tested in a major course. [full]
- **Undo.** Applying another rule to a sequent retracts everything above.
- **Evidence.** Exams over three years (70, 51, 46 students): wrong rule
  application on a purely syntactic proof by more than 50% (80% counting
  non-attempts) without the tool, 25% and 46% with it; average points
  1.2 to 1.5 of 4, because strategy, not syntax, was the bottleneck.
  Three cohorts, different sequents, no control; the authors say so. [full]
- **Errors the authors classify.** Wrong connective's rule, rule applied to
  a subformula, wrong or non-fresh instantiation, adding Gamma/Delta to a
  sequent; the commonest in year three (7 of 19) was misreading conjunction
  precedence in the succedent. [full]

### Proof Tree Builder (Princeton)

- **What.** Joomy Korkut, ThEdu'22 (EPTCS 375); static JavaScript,
  https://proof-tree-builder.github.io (paper read, site not fetched);
  LK and Hoare logic. [full]
- **Clicks.** A hole button over each open leaf opens a rule popup; a
  prompt asks which formula only when ambiguous. The goal is typed with a
  live parse display (precedence, binder scope) and red parse errors.
- **Display.** Upward tree on a zoomable canvas; detach a subtree, drag it
  onto another hole, fold; per-rule undo; LaTeX export.
- **Inapplicable rule.** Learning mode offers all rules and warns if one
  cannot apply. Automation mode hides rules whose connectives do not
  match, adds AUTO (stops when a choice among formulas exists, on purpose)
  and a Z3 button (discharges or shows a countermodel).
- **Evidence.** One graduate course: all used it, no incorrect proofs, no
  usage questions on the forum, midterm scores unchanged; the professor
  first worried students could "button-mash". "No further user study."

### NaDeA, SeCaV and MiniCalc (DTU)

- **NaDeA** (https://nadea.compute.dtu.dk/ ; Villadsen, From, Schlichtkrull;
  arXiv 1904.00618): click-only natural deduction, verified in Isabelle.
  It **shows only the rules that can apply** and never accepts an
  ill-formed formula. Questionnaire (12 sheets, informal): disadvantages
  listed were tedious clicking and progress "by clicking blindly"; authors
  conclude this "is not a problem for larger proofs". Biggest classroom
  gripe: clicks needed to enter formulas. Undo reaching all earlier states
  was praised; one student wanted to edit a formula without undoing
  everything after it. They say one assistant per 40 students is "more or
  less mandatory". [full]
- **SeCaV, MiniCalc** (arXiv 2505.05988; 100+ students 2021 to 2024):
  text-based; the student writes the result of each rule; detailed
  warnings. MiniCalc is a **one-sided** calculus with an explicit `ext`
  rule (permutation, weakening, contraction) whose omission produces a
  warning; not for absolute beginners. The nearest one-sided precedent.
  [full]

### Carnap (carnap.io)

- **What.** Graham Leach-Krouse; free, open-source Haskell framework,
  https://carnap.io and https://carnap.philosophy.ubc.ca/ ; 50+ formal
  systems, graded exercises; "over 45,000 students", "over four million
  problems" (arXiv 2607.07722, July 2026). [full]
- **Does.** Mostly typed line-by-line proofs (Fitch, Lemmon, Kalish-Montague)
  with live per-line verdicts; the checker runs in the browser and the
  server stores the verdict without rechecking. Zach (2020) says it also has
  Gentzen-style tree editors [snippet; not seen].
- **Phone.** No mobile-friendly design (stated in arXiv 2002.12553). [full]
- **The lesson to keep.** "Engagement requires trust. Students need to trust
  the system, and feel that any obstruction they encounter is intrinsic to
  the logical problem they're facing, not a limitation of the tool. A
  superficial bug, a confusing error message, a slightly inconsistent user
  interface state all reduce trust ... when trust is broken, a student's
  natural response is to try to replace the mental model of logic ... with a
  new model of how the software actually works, typically discovered
  through increasingly desperate rounds of trial and error." [full] One
  designer's report, from the largest user base here.
- **Costs listed.** Haskell front end, GHCJS rot, higher-order unification
  per rule (complexity begat bugs, search begat bottlenecks), server load at
  in-class exams; client-side checking scaled well.
- **No published learning analytics from Carnap found**; the retrospective
  has none.

### Jape (Bornat and Sufrin)

- **What.** Generic proof editor, Oxford and Queen Mary, since 1996, Java
  desktop; Bornat's *Proof and Disproof in Formal Logic* uses it; a 2026
  BCS seminar page says it is still distributed and used in teaching [snippet].
  Paper: "User Interfaces for Generic Proof Assistants, Part I" (UITP 1996,
  rev. 1998). [full]
- **Clicks.** Select a hypothesis and/or conclusion, then choose a rule from
  a menu or double-click the formula (meaning declared by the logic's
  designer). Text-selecting a term first supplies a parameter; otherwise
  Jape makes a named proof unknown so the choice can wait. Boxed Fitch or
  tree display. [full]
- **Ambiguity/failure.** With no hypothesis selected and several fitting,
  a choice box appears; the authors call the "sudden switch in the style of
  interaction" "disconcerting". A failing gesture reports its tactic's
  message. Principle: no "magic" transformations. [full]
- **Evidence.** A 1999 PPIG study with CS undergraduates exists (not read).
  The Kassel authors call Jape "lacking a feedback system".

### Other tools, one paragraph each

- **Pandora** (Imperial; box proofs; Tcl/Tk editor, Prolog checker;
  https://www.doc.ic.ac.uk/pandora) [page]: Broda et al. say every rule
  application gets "either success or a helpful error message" and usage
  logs were kept [snippet]; no analysis found. Desktop.
- **Logic Daemon** (Allen and Menzel, https://logic.tamu.edu/): web form,
  typed natural deduction lines checked per line, hints. [snippet] Thin.
- **Holbert** (O'Connor and Amjad, Edinburgh, arXiv 2210.11411): browser
  textbook with user-defined rules and graphical proofs; work in progress,
  no evaluation. [full]
- **OnlineProver** (arXiv 2505.05987): typed Gentzen derivations with
  feedback inside the tree. 55 students, SUS 67.3 (average 68); top problem
  was understanding deduction rules (20 of 48 answers), then symbol input
  (5) and understanding error messages (4); "check" pressed after almost
  every step; students judged pen and paper more effective. [full]
- **LogicProof** (arXiv 2607.04321, https://6ximik9.github.io/naturalDeduction/):
  natural deduction and sequent calculus, classical or constructive; select
  a node, pick from a rule list (all or only applicable); Monaco input with
  parse underlining; tour, undo, auto-focus on next open premise. 35
  students, SUS 79.5, success 91% and 83%; issues: UI and navigation 25%,
  how to begin or apply rules 20%, symbols 15%; 50% trial and error;
  complaint "loss of progress upon page refresh without warning". Short
  sessions, self-reported. [full]
- **Hazel Deriver** (arXiv 2506.10781): live derivation editor with typed
  holes, wrong subterm highlighted, rule documentation in a sidebar; small
  study. [full, abstract and intro]
- **Edukera** (Coq in the page, point-and-click, "syntactically correct by
  construction, as using Scratch", about 1000 students per Narboux's
  slides) and **deaduction** (Lean, local). [full: slides] Claims that
  Edukera is unmaintained since 2020 and that students finished without
  understanding came from a search summary: unverified.
- **Actema / drag-and-drop tactic** (Donato, Strub, Werner, CPP 2022):
  drag one item onto another to trigger a step; no student study. [full,
  skimmed]
- **AXolotl** (arXiv 2002.12553): Android, touch-only, swipe to undo,
  zoomable proof viewer. States that web logic tools "did not have a mobile
  device friendly web-design" and that Peanoware (drag-and-drop natural
  deduction) impedes progress: no tutorial, "at no point is one really told
  what they are doing". The only phone-oriented evidence; no learning
  evaluation yet. [full]
- **LogicLearner** (arXiv 2503.19280): two-level hint ladder (rule, then
  expression); "hints that backtrack to a start state make for a poor user
  experience". [full]
- **Lean Game Server / Natural Number Game** (https://adam.math.hhu.de/):
  tactics unlocked per level; no evaluation of the game found. Carnap's
  author cites a ceiling of about 70 simultaneous users because Lean runs
  server-side. Thoma and Iannone (IJRUME 2022): voluntary Lean workshops,
  36 proofs, qualitative, students who engaged wrote more accurately and
  split goals better, framed as a hypothesis. [snippet]
- **"Proofs with Prolog / Proof Tree".** I could not identify a tool by that
  name. Nearest: a web app in arXiv 1301.5077 (Utrecht) where pupils drag
  Prolog clauses onto a query to build a proof tree, open goals yellow,
  closed nodes green. [snippet only; PDF would not convert]

### Proof nets

- **No interactive web proof-net editor found.** Only: the AIST Proof Net
  Calculator (Scala library, MLL, desktop GUI, EPS output, several
  correctness criteria; https://staff.aist.go.jp/s-matsuoka/PNCalculator/)
  [snippet]; xpnet, a 1992 interface [snippet]; ProofNet-IR, a Lean 4
  checker [snippet]; and Click & coLLecT issue 174, a request for exactly
  this. Laurent's "MLL-ego: Linear Logic as a Lego Game" (https://perso.
  ens-lyon.fr/olivier.laurent/mllego_en.pdf) is a paper game on nets as
  blocks; not read. The search tool is limited; a niche project could exist.
- The Incredible Proof Machine is the closest graph precedent: wires
  between ports, red/green checking, tolerance of incomplete graphs, touch.
  It is natural deduction, not nets.

## What students stumble on

Mostly classical or first-order evidence; none is about linear logic.

1. **Applying a rule is the first barrier, and a tool can remove it.**
   Kassel: more than 50% failed a purely syntactic proof through wrong rule
   application before the trainer, 25% and 46% in the two cohorts with it
   (arXiv 1507.03666). Types: rule for the wrong connective, rule on a
   subformula, bad instantiation, context variables written into a sequent,
   misread precedence. Parse display (Proof Tree Builder) and scope
   highlighting (Kassel) target the last.
2. **Finding the proof is separate and larger.** Kassel points 1.2 to 1.5 of
   4. OnlineProver: "understanding deduction rules" was the top complaint
   and outranked tool complaints. LogicProof: 20% could not begin a proof or
   apply rules, especially in natural deduction.
3. **Trial and error dominates, and authors disagree on its worth.** 50% in
   LogicProof, "check after almost every step" in OnlineProver logs,
   "button-mash" at Princeton, "clicking blindly" in NaDeA. Kassel uses the
   compiler analogy as a reason for feedback; Logitext's critics as a danger.
   NaDeA and Princeton report no loss of comprehension, with no control.
4. **Input friction.** Symbols and clicks to enter formulas: NaDeA's biggest
   gripe, OnlineProver (shortcuts the most requested improvement, 11), LogicProof
   (15% and 25%). Precedence: the Kassel exam error; C&C issue 173.
5. **Lost work, unclear state.** LogicProof reload without warning; IPM
   reload bug; Carnap's "inconsistent user interface state".
6. **Undo that misleads.** LogicProof "step back"; C&C issues 145 and 150
   (closed); NaDeA's praised full reachability but no mid-proof edit.
7. **Scope and structure.** IPM port types and local variables; Kassel fresh
   constants; Logitext quantifier order. Linear analogues are my inference,
   not evidence: the context split for tensor and the choice among
   exponential rules, which C&C encodes in click position and formula order,
   and exchange is where its tracker shows the most pain.
8. **Reading versus writing proofs; teacher support.** The 2025 survey
   (arXiv 2505.13472) quotes a Coq course report ([BK18] there) that
   transfer of proof-assistant skill to textbook proofs is "extremely
   underestimated", and says evaluation of didactic impact "is still at its
   debut". OnlineProver students rated paper more effective. NaDeA's
   authors: one assistant per 40 students.

Gaps: no peer-reviewed learning analytics from Carnap or the Incredible
Proof Machine; log analyses exist for OnlineProver and, older, Vaculik et
al. (2013, 2014, sequence mining of undergraduate logic proofs; search
summary only, not read).

## What a linear-logic proof page should take, and avoid

Each bullet is one claim with its source. "Take/avoid" is my design reading
of the evidence, not a finding.

Take:

- Check each step in the browser with no round trip: Carnap scaled to 45,000
  students checking client-side and its author puts fast approve/reject at
  the core of engagement (arXiv 2607.07722). A WebAssembly core fits.
- Make applicability visible and local: IPM marks the offending wire and
  leaves it (ITP 2016); Kassel highlights the clicked operator's scope and
  explains refusals (arXiv 1507.03666).
- Separate legality from strategy: Kassel's data support the first, and its
  authors say the second needs more than a verdict (arXiv 1803.01467).
- Use a three-valued answer with an honest "unknown": C&C (red, orange,
  black) and Kassel (red, yellow, green); linlog's exit statuses and
  `close` verdicts already have it; Carnap's trust argument says a
  time-out shown as "no" costs credibility.
- Offer hints as a ladder, never as a jump back to the start (arXiv
  2503.19280).
- Show a parse preview with as-you-type errors for the sequent box (Proof
  Tree Builder; LogicProof), and accept ASCII (`-o`, `|-`, `&`) with symbol
  buttons, since symbol entry is the most reported input pain (OnlineProver,
  LogicProof, NaDeA). linlog's parser already takes ASCII.
- Give a visible undo reaching every earlier state, and test it: NaDeA's
  students liked full reachability; C&C and LogicProof shipped undo bugs.
  linlog's `undo` and JSON session make a total history cheap.
- Put the whole state in a share URL and autosave locally with a warning
  before discarding: C&C shares by URL; LogicProof users lost work; IPM
  uses Web Storage.
- For rules a formula does not determine, let the click position or a
  visible chooser decide: C&C picks plus by sub-formula and weakening by
  click count. linlog's `apply` takes positions for tensor and Mix; the page
  needs a visible way to choose the split, not formula order alone.
- Keep exchange out of the student's way: C&C's drag-to-reorder and its
  explicit-exchange export produce its longest-open issues (165, 169);
  MiniCalc has an explicit permutation step. linlog contexts are multisets
  by position; say so on the page.
- Model the tutorial on Logitext: tiny live examples, hover definitions, a
  reset click, plus Laurent's "predict the result before pressing".
- Close the loop with export: SVG/LaTeX of a partial proof and a
  certificate (C&C exports Coq for open proofs; NaDeA verifies in
  Isabelle). linlog's `proof --rocq` is the analogue.
- Work offline from static files: IPM survived bad workshop Wi-Fi;
  Princeton's tool and LogicProof are static pages.
- Provide a tap-only path (no drag, no hover): no web tool in the survey has
  one, and the AXolotl authors found web tools not mobile-friendly. Evidence
  of demand is thin; the claim is that the gap exists.

Avoid:

- Click-the-formula with no visible rule or split: its critics say clicks
  change state in "cryptic ways" (arXiv 2303.05865) and its author concedes
  "not sure what happened". Print the rule name and context split on every
  step.
- Hiding every inapplicable rule without a reason: NaDeA's authors report
  blind clicking on small proofs, Princeton feared button-mashing; counter
  evidence is one course each. Untested middle path: list rules that act on
  the selected formula, grey the rest with a one-line reason, refuse on
  click with a message (Kassel style). No source tests this.
- Silent no-op clicks: C&C's page does not describe a message, and Carnap
  says an unexplained obstruction makes students suspect the tool.
- A tree wider than the screen with no zoom, pan or fold: LogicProof
  reports trees not fitting and dialogs covering context (5% of problem
  reports; branching succeeded for 83%); Proof Tree Builder added zoom,
  detach and fold. Linear sequents have no weakening to shed formulas, so
  width will bite earlier than in classical tools.
- Modal dialogs over the goal (LogicProof) and sudden changes of interaction
  style (Jape's choice box, "disconcerting").
- A bare "unprovable" verdict: Kassel's authors think a counter-model is "a
  step too big" for struggling students. linlog's refutation reasons (an
  atom's balance, the count equation) are a smaller step; no source tests
  whether students follow them.
- Treating proof nets as one more picture: no precedent exists to learn
  from, C&C's maintainers call drawing nets from sequent proofs hard to make
  look good (issue 174), and IPM shows only that port graphs can be edited
  by touch with red/green feedback. Treat net editing as a research bet,
  with the sequent page as the baseline.
- Per-calculus code that makes a new rule set expensive: Carnap's author
  blames higher-order unification per rule and a monolithic stack for bugs
  and contributor loss. Keep rules in the Rust core and the page thin.
- Claiming learning gains. The base is two exam comparisons (Kassel),
  several SUS questionnaires, one workshop, a few qualitative studies;
  none measures transfer, and OnlineProver's students preferred paper. Log
  steps and refusals if evidence is wanted later; no click tool publishes
  such logs.

## Open questions

- How Click & coLLecT reports an inapplicable click, and whether it works on
  a phone or by keyboard: needs hands-on testing.
- What Logitext prints on a failed click.
- The Click & coLLecT paper's user-feedback section (HAL is behind a bot
  wall; try the authors' pages).
- Pandora's evaluation, Jape's PPIG 1999 study, Vaculik et al.'s results.
- Any published student errors in a linear-logic course; the 2022 winter
  school page and the Lego-game note are the only teaching material found.
