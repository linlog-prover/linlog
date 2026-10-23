# Step 43: Learn's lessons, and the practice that goes with them

You are working on linlog's web client, in the client's repository
(`/home/tux/Projects/own/linlog-prover.github.io`), whose CLAUDE.md and
`.claude/rules/` apply throughout; in this repository CLAUDE.md applies
to anything you read or change. The step is content, written into the
place step 34 made for it and step 38 filled with practice; it takes
three sessions. This prompt was written on 2026-10-23 at the author's
word. It says what is fixed.

Read `plan/web/design.md` (D1 to D60, fixed: D21, D23, D47, D56 and D60
above all), the mockup (`plan/web/mockup/` and its README),
`plan/web/README.md`, the reports `plan/reports/34-web.md`,
`38-web-polish.md` (its "Practice" and the references of item 7) and
`38b-zoom-fonts-dismiss.md` (the harder practice, the stage's fonts),
and `plan/later.md`'s "Learn's content".

## Goal

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

- **Three sessions, each a fresh context** (the plan's guidance: a long
  horizon is split into sessions, not one context compacted):
  - **43a**, lessons 1 to 5 of D60 (reading a sequent, the
    multiplicatives, the additives, the units, the exponentials), with
    the reference's rule schemas they use and the frame's hover
    explanations; after steps 39 and 40.
  - **43b**, lessons 6 to 9 (two sides, on step 40's two-sided surface;
    when there is no proof; proof nets, with step 39's nets with boxes
    and practice whose nets need boxes; ordinary logic, last and
    optional); after 43a.
  - **43c**, two lessons D60 does not have yet, placed where the
    session argues they belong and confirmed with the author: cut and
    its elimination (on step 41's page: a lemma that a cut makes short,
    "eliminate this cut" step by step on the derivation and the net),
    and the quantifiers (on step 42's page), each with its practice;
    after steps 41 and 42.
- **D60's order is the author's to confirm** as the content is written:
  43a sends its outline of the lessons (each lesson's worked proof, its
  faded steps, its prompt, its hinge question and its misconceptions)
  to the planning session before writing them out, and the planning
  session asks the author; 43c likewise for its two lessons.
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
  writer. 43a and 43b may run beside step 41's linlog sessions, which
  touch no file of the client.

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
