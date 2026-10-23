# Step 43: Practice and Learn, from one store of problems

You are working on linlog's web client, in the client's repository
(`/home/tux/Projects/own/linlog-prover.github.io`), whose CLAUDE.md and
`.claude/rules/` apply throughout; in this repository CLAUDE.md applies
to anything you read or change. The step is content, written into the
place step 34 made for it and step 38 filled with practice; it takes
four sessions. This prompt was written on 2026-10-23 at the author's
word. It says what is fixed.

Read `plan/web/design.md` (D1 to D60, fixed: D21, D23, D47, D56 and D60
above all), the mockup (`plan/web/mockup/` and its README),
`plan/web/README.md`, the reports `plan/reports/34-web.md`,
`38-web-polish.md` (its "Practice" and the references of item 7) and
`38b-zoom-fonts-dismiss.md` (the harder practice, the stage's fonts),
and `plan/later.md`'s "Learn's content".

## Goal

Practice and Learn are two places with two purposes, drawing from one
store of problems (the author, 2026-10-23). **Practice** is where a
visitor who knows the rules proves sequents: a large set, from easy to
very hard, sorted by sensible criteria, with a button for a random one,
and every sequent offered there derivable, so that a visitor never
labours at a sequent that has no proof. **Learn** is the step-by-step
introduction for a student: the ideas of linear logic, the connectives
one at a time in plain words, what kinds of sequents cannot be proved
and why, and proof nets.

Learn holds the lessons D60 orders, each in D56's worked-example
sequence: a proof played with a reason at each inference and a
"predict the next rule" prompt, the same proof with its last steps left
to the student, then practice from the rubric; a self-explanation prompt
at the lesson's point of highest density and one hinge question at its
end whose wrong answers each name a misconception; the lesson on
sequents with no proof the erroneous example ("find the step that went
wrong"), then the refutation. The rules' schemas in the reference and
the hover explanations inside the frame (D47) come with them.

## Fixed now (the author, 2026-10-23)

- **Four sessions, each a fresh context** (the plan's guidance: a long
  horizon is split into sessions, not one context compacted):
  - **43a**, the problem store and Practice (the section below); after
    step 39, and it may run beside step 40's linlog session, which
    touches no file of the client.
  - **43b**, lessons 1 to 5 of D60 (reading a sequent, the
    multiplicatives, the additives, the units, the exponentials), with
    the reference's rule schemas they use and the frame's hover
    explanations; after 43a and step 40.
  - **43c**, lessons 6 to 9 (two sides, on step 40's two-sided surface;
    when there is no proof; proof nets, with step 39's nets with boxes
    and practice whose nets need boxes; ordinary logic, last and
    optional); after 43b.
  - **43d**, two lessons D60 does not have yet, placed where the
    session argues they belong and confirmed with the author: cut and
    its elimination (on step 41's page: a lemma that a cut makes short,
    "eliminate this cut" step by step on the derivation and the net),
    and the quantifiers (on step 42's page), each with its practice;
    after steps 41 and 42.
- **D60's order is the author's to confirm** as the content is written:
  43b sends its outline of the lessons (each lesson's worked proof, its
  faded steps, its prompt, its hinge question and its misconceptions)
  to the planning session before writing them out, and the planning
  session asks the author; 43d likewise for its two lessons.
- **Every sequent decided by linlog's command first**, every provable
  one with a session written by `linlog interact` that `proof` accepts,
  as step 38's practice (its scripts in
  `/home/tux/Projects/own/linlog-step38-screenshots/`); the evidence kept
  beside the step's screenshots. The written content under CC BY 4.0
  (D23), every reference checked against its source by reading it.
- **The education plugins** as step 38 used them: the
  `education-agent-skills` skills for the worked-example sequence and
  its fading (`worked-example-fading-designer`,
  `digital-worked-example-sequence`), the hinge questions
  (`hinge-question-designer`), the self-explanation prompts
  (`self-explanation-prompt-designer`), the erroneous example
  (`erroneous-example-designer`), the progression and cognitive load
  (`learning-progression-builder`, `cognitive-load-analyser`), and
  `frontend-design:frontend-design` for anything the lessons add to the
  page, within step 38's tokens, grammar and 44-pixel targets.
- **Content only, in the client**: a lesson needs no change of linlog;
  one that turns out to need one (a call the page lacks) says so to the
  planning session instead of making it, so that linlog keeps one
  writer. 43b and 43c may run beside step 41's linlog sessions, which
  touch no file of the client.

## The problem store and Practice (43a; the author, 2026-10-23)

- **One store of problems** in the client's content, the single source
  both Practice and Learn draw from (Learn's worked, faded and erroneous
  examples and its lessons' practice name problems of the store; no
  sequent is written twice). Each problem records its sequent, its
  calculus (the tab's mode), linlog's verdict, a difficulty on one
  scale from easy to very hard, the categories it is found under, and,
  when provable, a session written by `linlog interact` that `proof`
  accepts. The categories and the difficulty scale are the session's,
  argued with the education skills (`practice-problem-sequence-designer`,
  `learning-progression-builder`, `interleaving-unit-planner`); the
  fragment linlog detects (`seq fragment`) and the connectives a problem
  uses are among the obvious criteria, and difficulty rests on what a
  human proof needs (its size, the splits and choices on the way, the
  copies), measured on the checked proof and calibrated by hand, not on
  the search's effort alone.
- **Many problems**: at least 300 provable ones, every difficulty level
  and every fragment the page proves in represented, the hardest ones a
  real effort for someone who knows the rules; step 38's and 38b's
  practice sequents join the store. Never copied wholesale from a
  library whose licence the content's CC BY 4.0 cannot carry (LLTP is
  GPL-3.0, ILTP states no licence; the repository's rule keeps
  third-party files out).
- **Generated with intent, and chosen from the literature** (the author,
  2026-10-23): most problems come from generators written for this,
  which aim at a difficulty rather than land on one: built backwards
  from a proof whose shape is chosen (how deep, how many `⊗` splits
  with how many wrong ways to split, `&` branches, copies of a `?`
  formula, units, a two-sided `⊸L`), then disguised (formulas reordered,
  atoms renamed, invertible steps added), with linlog checking each and
  keeping those whose measured difficulty is the one aimed at; linlog's
  own families may seed some. Beside them, the insightful sequents of
  the literature, each worth a student's time for what it shows (the
  textbooks' and the survey papers' classic examples: currying,
  distributivity of `⊗` over `⊕` and its failure the other way, the
  exponential isomorphisms `!(A & B) ≡ !A ⊗ !B`, the encodings of
  intuitionistic implication, and the like), each with its source cited
  and read in the source; a single sequent stated in a paper is a fact
  of logic, not a copy of a library.
- **A difficulty that means something for a human**: the session finds a
  metric, from the literature on the difficulty of proof exercises and
  logic tutors (research sub-agents on Sonnet 5.5 at `medium`, every
  source read) and from the problems' structure, rather than from the
  search's effort, which says little about a person; candidates are the
  checked proof's size and depth, the number of points where a choice
  is needed and how many of the choices fail (a wrong `⊗` split, the
  wrong side of a `⊕`, a copy too few), the copies needed, and the
  fragment. It is validated before it ranks 300 problems: the session
  has a few sub-agents of different strengths (Haiku 5.5 and Sonnet 5.5,
  given only the rules and `linlog interact`, bounded in steps and time)
  attempt a sample across the levels, and shows that the levels order
  their success and effort; the levels' names and boundaries and the
  validation go into the report, for the author to judge.
- **Checked by the flake**: a check of the client's flake runs linlog's
  command, at the client's pinned input, over the whole store: every
  verdict is the command's, every provable problem's session is
  accepted by `proof`, and a store entry without that evidence fails the
  check. A problem whose verdict a later linlog changes fails it too.
- **Practice is a place of its own**, outside Learn's frame, reached from
  the bottom band at every width (departs from the design: D47, at the
  author's request): it offers only provable problems, filtered by the
  categories and the difficulty, and a random one (within the filters
  chosen); a problem opens on a new tab; the hint ladder is its help;
  which problems a visitor has proved may be remembered in the
  browser, as the other settings are. `frontend-design:frontend-design`
  for its look, within step 38's tokens, grammar and 44-pixel targets.
- **Learn keeps the unprovable**: the store's problems with no proof are
  Learn's (lesson 7's erroneous examples and its refutations), never
  Practice's; Learn's own practice rubric becomes the lesson's practice,
  drawn from the store.
- **Later steps add to the store**: the nets with boxes (39), the
  two-sided surface (40), cut (41) and the quantifiers (42) each add
  their problems in the session of this step that teaches them.

## How the step runs

Three sessions on Opus 5.5 at `high`, each with its goal in its launch
prompt; one fresh-context reviewer on Opus 5.5 at `high` per session,
reading the lessons against D56 and D60 and the sequents against the
command's verdicts, told to report only what breaks a stated
requirement; before-and-after screenshots at a phone's and a desktop's
width in both themes, which the author judges. Each session's tests:
every lesson's sequents and sessions checked, as the practice's are.

Deliverable: thematic jj commits in the client;
`plan/reports/43-learn-lessons.md` in linlog.
