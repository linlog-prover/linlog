# Scope: teaching and interaction

Written 2026-10-09 for the question of how linlog's scope could grow
beyond or beside its saved plans. I read `plan/later.md`, the research
notes' `README.md`, `32-web.md`, `usability-baseline.md` and
`refutations.md` first. Sources outside the repository are [E1] to
[E24] in the last section; repository files are named by path.
"(inference)" marks my own conclusions.

What is already planned for teaching: step 32 is "`linlog interact`
with a mouse", with goals as click targets, rules, undo, close, the
drawing and the exports (`plan/32-web.md`, "What is fixed now", 4). The
research note for 32 lists a `graphics-document` root and MathJax speech
as later accessibility work (`32-web.md`, Accessibility; Formulas).
Steps 33 to 36 add the objects a course shows: MELL nets with boxes,
cut elimination one step at a time with each step drawn
(`plan/34-cut.md`), the drawing of essential nets "which teaching wants"
(`plan/35-mll-engines.md`), Lambek. D21 makes research and teaching
equal aims (`plan/README.md`). Nothing below repeats those.

## 1. The field as it stands

**Linear-logic trainers.** Click & coLLecT (Etienne Callies and Olivier
Laurent, TLLA 2021 [E1]) is the reference point. On its site the user
clicks connectives to apply rules, double-clicks `?` for contraction,
can switch on "auto-reverse", which "applies systematically reversible
rules to generated sequents", introduces cuts, eliminates or commutes
them, and asks an auto-prover per sequent: a red turnstile "means that
the sequent is known not to be provable", an orange one is a time-out
[E2]. It exports Coq (NanoYalla) (not in the source), LaTeX, PDF, PNG and text, and shares a
proof as a URL [E2]; the state is LZMA-compressed JSON, and the README
credits Logitext as its inspiration [E3]. "Export to proof-nets" is
listed as a future direction [E4]. Its logic runs in an OCaml HTTP
server (`plan/notes/research/29-comparison.md`). For proof nets the
nearest tool is Satoshi Matsuoka's Proof Net Calculator, a Scala
library with a simple GUI that checks MLL structures by several criteria
(Danos–Regnier, de Naurois–Mogbil, a linear-time one) and reduces cuts
[E20]. I found no web tool in which a student draws a linking and the
criterion answers (inference from the searches run). Frank Pfenning's
CMU course 15-816 is the standard set of course notes on linear logic
for computer science (sequent calculus, focusing, linear logic
programming) [E22].

**Sequent-calculus trainers for ordinary logic.** Logitext derives a
sequent by clicking connectives; its server is Haskell and Ur/Web over
Coq (BSD-3-Clause) [E5]. A "purely client-side reimplementation" covers
LK without cut (LJ is a separate implementation it links to) and keeps the derivation in the URL [E6]. The
Sequent Calculus Trainer of Ehle, Hundeshagen and Lange (Kassel,
ThEdu'17) gives compiler-like feedback on wrong rule applications and a
"traffic-light-like system" on every open sequent, green valid, red
invalid, yellow unknown, using an automated prover and Z3; a red light
after a step tells the student the step lost the proof, and a hint mode
names the next rule [E7]. Joomy Korkut's Proof Tree Builder (Princeton,
ThEdu'22) applies rules upward in the browser for sequent calculus and
Hoare logic, with basic automation [E12].

**Graphical proofs and games.** Joachim Breitner's Incredible Proof
Machine (ITP 2016) has students drag rule blocks and wire them into port
graphs, an ND-like but non-linear representation; tasks are grouped
into sessions, each session may show only a subset of a logic's rules
"for an educational progression", and the logics and tasks are files an
educator edits [E8]. A workshop with 13 high-school students covered
the propositional part in 14 hours and scored well on a standard
usability questionnaire [E8]; the core is Haskell compiled to JavaScript
(MIT) [E9]. The Lean Game Server (Alexander Bentkamp, Jon Eugster)
hosts level-based games after Kevin Buzzard's and Mohammad Pedramfar's
Natural Number Game, with Lean on the server in a sandbox [E19]. Chris
Martens's Ceptre (AIIDE 2015) builds game mechanics on linear logic and
draws a correspondence between gameplay and proof search [E21].

**Course platforms and generated exercises.** Carnap (Graham
Leach-Krouse, Jake Ehrlich) checks natural deduction in many textbook
systems, LK and LJ among them, grades problem sets automatically, and
offers a JSON API for custom interfaces [E11]; over a decade more than
45 000 students used it [E10]. Iltis (Geck, Zeume and others) composes
feedback generators per task type into strategies an instructor tunes,
from much feedback for beginners to almost none for exam practice
[E14], and plugs into Moodle through LTI [E15]. ProofBuddy (Karsten,
Jacobsen, Eiken, Nestmann, Villadsen) wraps a server-side Isabelle and
records fine-grained interaction data for research [E13]. On generation:
Ahmed, Gulwani and Karkare (IJCAI 2013) generate both solutions and new
natural-deduction problems "with specified solution characteristics"
by forward and backward search over a precomputed proof graph [E16];
Lodder, Heeren and Jeuring's Logax generates Hilbert-style proofs and
gives hints that still work when a student strays from the generated
proof (SIGCSE 2017, IJAIED 2021) [E17]; Automata Tutor v3 grades
constructions and generates new instances for several problem types
[E18].

**Accessibility.** The accessible HTML of *forall x: Calgary* codes
proofs as tables with a column for the subproof level, so that "screen
readers should also announce if a subproof has just been closed" [E23].
MathJax's explorer and the Speech Rule Engine produce speech and
keyboard navigation for formulas [E25]. WAI-ARIA Graphics'
`graphics-document` role keeps an SVG's children meaningful and
interactive [E24]. I found no screen-reader-usable sequent-calculus or
proof-net tool (inference from the searches run).

**Where linlog stands.** linlog already has what the trainers above
borrow from a server: a checked proof term, three-valued verdicts, a
search from any goal, a net criterion with a witness
(`NetError::SwitchingCycle`, `.claude/rules/core-nets.md`), a stable
JSON state, and SVG with click ids. Running all of it in the browser
(step 32) is a combination none of the tools above has
(`32-web.md`, Existing implementations).

## 2. Candidate extensions

### education-1: a provability light on every open goal

*What.* After each `apply`, the client runs a short, bounded search on
each new goal and shows proved, unprovable or unknown, as the Trainer's
traffic light [E7] and Click & coLLecT's turnstile colours [E2] do. In
the classical reading a red goal can carry the counter-assignment, and
in fragments where `refutations.md` has a certificate, the red is
certified.
*Value.* The feedback the Trainer's authors found decisive for the
"semantic level": the student learns at once that a split lost the
proof [E7].
*Cost.* Small in the library: `close` already searches a goal; needed
are a per-goal budget (later.md lists one as "the client's wrapping"),
results cached per goal, and the work in the worker of step 32.
*Relation.* Extends step 32 and the certified refutations.
*Leave room now.* A goal status as data on `Interactive` (or beside
it), not a side effect of `close`; `Outcome`/`Refutation` serializable
per goal; `Stop::poll(progress)` (README §2) so several small searches
share one budget.

### education-2: proof-net exercises, a linking drawn and checked

*What.* The student clicks pairs of dual literals on the drawn formula
trees; linlog checks the linking, colours the switching cycle or the
disconnected parts, and sequentializes a correct net into a derivation.
MLL first, MELL boxes after step 33.
*Value.* The usability baseline's logician was stuck exactly here: "no
command checks a linking a student proposes" (story 9). No web tool does
it (section 1).
*Cost.* Moderate: a partial-linking state, `check` reading a
`ProofStructure` JSON, the witness mapped to SVG ids. No new theory for
MLL; boxes need 33's data model.
*Relation.* Extends steps 5 and 11 and usability story 9; sits beside 33
and 35 (essential nets as a drawing).
*Leave room now.* `ProofStructure` with a version and a constructor
from a linking given as pairs of `OccId`; `NetError` variants carrying
ids, not text; the SVG id scheme (`o<n>`, `l<m>-<n>`) fixed as a
contract that a click on a literal and a coloured edge both use.

### education-3: hints and auto-reverse from the engines

*What.* A "next step" taken from a proof the search finds for the goal,
given in layers: the formula to work on, then the rule, then the split,
then the step applied, as Logax's hints [E17]. An option applies the
invertible rules automatically, as Click & coLLecT's auto-reverse [E2];
the focused engine's phases name exactly those (inference).
*Value.* Turns the trainer from a checker into a tutor; the Trainer has
a hint mode for valid sequents [E7].
*Cost.* Small to moderate. A hint must survive the student's own path,
so it is recomputed per goal; a hint for a `⊗` split needs the split the
proof chose, mapped to the goal's occurrences.
*Relation.* Extends step 32's interactive view (and step 9).
*Leave room now.* `close` able to return the proof without grafting it
(a peek), and a hint as a value: goal, position, `Rule`, split.
`rules` should expose invertibility per rule and mode.

### education-4: course packs, exercise sets with checked answers

*What.* A file format for a course: a list of sequents, each with its
mode and fragment, the rules allowed (a subset, as the Incredible Proof
Machine's sessions [E8]), whether search may close goals, and the
expected verdict. linlog grades a submitted proof or state by the
checker, and writes the sheet and the solution sheet through the LaTeX
and Typst exports.
*Value.* What Carnap and Iltis give an instructor [E10][E14], for linear
logic: answers checked by the same checker as the search's proofs.
*Cost.* Moderate: a format with a version, a rule restriction in
`Interactive`, and a grading entry beside `batch::run`.
*Relation.* New; sits beside step 24 (batch) and step 32.
*Leave room now.* A rule set as a value (`Rule::ALL` and a subset type)
that `Interactive::rules` and `apply` filter by; the `Proof` and
`Interactive` JSON versioned and readable after a release (README §2,
wire forms).

### education-5: generated exercises with certified properties

*What.* Seeded generators of sequents with chosen properties: fragment,
size, provable or not, minimal proof height, a required rule (a `⊗`
split that must be right, a `!` that must be promoted), and a distinct
shape per student. Each candidate is decided by the engines and kept
only with a checked proof or a certified refutation. Ahmed et al. search
backward from proofs to problems with given solution characteristics
[E16]; Automata Tutor generates instances [E18].
*Value.* Fresh exercises per student and per exam, with no wrong answer
key.
*Cost.* Moderate; proof statistics (height, rules used) on `Proof`, a
canonical form to deduplicate (the hash-consed arena gives one), and
"unknown" excluded.
*Relation.* New; beside `families.rs` (seeded SplitMix64 generators with
a `provable` field, `.claude/rules/core-inputs.md`) and education-4.
*Leave room now.* `families::Instance` as a public, documented type a
caller can build; proof statistics derived from the term, not from
engine counters.

### education-6: interactive LK and LJ through the embeddings

*What.* Logitext's interaction [E5][E6] for classical, intuitionistic
and minimal logic: the student applies LK or LJ rules; each goal's light
and the final check go through step 25's translation and read-back.
*Value.* LK and LJ are what Carnap's courses and Logitext's users
prove in [E11][E6]; one tool for both lets a course show the
translation, the same goal in LJ and in ILL (inference).
*Cost.* Real: an interactive state over `ordinary::Derivation`, and the
JSON form of `ordinary::Sequent` and `ordinary::Derivation` that step
25's report lists as missing (`32-web.md`, Data model).
*Relation.* New; beside steps 25 and 32.
*Leave room now.* `Interactive`'s operations (goals, positions, rules,
apply, undo, proof) described as a trait or a generic over a calculus,
so a second calculus is a second implementation rather than a copy.

### education-7: accessible derivations and nets

*What.* Every drawing has a structural twin: a derivation as a nested
outline (conclusion, rule, premises), a net as a list of links and
nodes, each formula with a spoken form ("A tensor B"), keyboard focus
on every click target, and an SVG root of role `graphics-document`
[E24], as the textbook's accessible proofs encode the subproof level
for screen readers [E23].
*Value.* Blind and low-vision students can do the same exercises; no
tool in section 1 offers that for sequents or nets.
*Cost.* Small in the library (a printer), most of it in the client.
*Relation.* Extends step 32; `32-web.md` names `graphics-document` and
MathJax speech as later work.
*Leave room now.* The public formula walk (`sequents::fmt::Walk`,
README §2) able to drive a spoken printer; a derivation outline as a
JSON form alongside the SVG; `Style` with a description per element,
not only per drawing.

### education-8: shareable and embeddable proofs

*What.* A proof state compressed into the page's URL, as Click &
coLLecT [E3] and the Logitext reimplementation [E6] do; an embed of one
exercise in a course page or textbook, read only or live.
*Value.* Students hand in a link; a lecturer puts a live example in the
notes.
*Cost.* Small: the state's JSON exists; compression and a size check in
the client.
*Relation.* Extends step 32 (the client repository).
*Leave room now.* A compact, deterministic, versioned state form (the
same state always the same string), and `from_json_within` with a bound
(README §2) since a URL is untrusted input.

### education-9: resource puzzles over the Horn engine

*What.* A play mode for Horn problems: places as tokens, clauses as
transitions the player fires, the goal a marking to reach or cover. The
Horn engine says whether the level is solvable, gives the firing
sequence as the solution, and reads it back as a proof. Levels in
sessions, as the Incredible Proof Machine's [E8].
*Value.* Shows the resource reading of linear logic without syntax;
Ceptre draws the same link between play and proof search [E21].
*Cost.* Moderate: a step API over the engine's net view, a drawing of a
Petri net (D12 asks for a layout the object suggests; a net has none
fixed: inference), level files.
*Relation.* New; beside step 27.
*Leave room now.* The Horn engine's places, transitions and firing
sequence as public data, and the proof read off a firing sequence
callable on a sequence the player gives.

## 3. Considered and rejected

- **User-defined logics, as the Incredible Proof Machine's logic files
  [E8].** linlog's checker is the trusted base for a fixed calculus
  (D6); a rule restriction (education-4) gives the progression without
  that.
- **Graphical natural deduction in the Incredible Proof Machine's
  style.** A third representation beside derivations and nets, which no
  plan, checker or export of linlog reads.
- **A server with accounts and a grade book (Carnap, Iltis, ProofBuddy
  [E10][E14][E13]).** Step 32's goal is that nothing is sent to a
  server; LTI needs a tool provider on a server (inference). A course
  pack graded locally (education-4) covers the core.
- **Finite phase countermodels shown to students.** `refutations.md`
  found no tool that searches them; the classical assignment in
  education-1 is the part within reach.
- **A Lean or Rocq game in the Natural Number Game's style.** The Lean
  target is deferred until after step 31 (`plan/later.md`), and the game
  server runs Lean on a server [E19].
- **MALL nets as an exercise.** Dropped in `plan/later.md`: "a display
  feature without a use".
- **A language-model tutor.** Its hints could not be checked by the
  checker, which every other answer here passes (inference).

## 4. Sources

- [E1] E. Callies, O. Laurent (2021). "Click and coLLecT: An Interactive
  Linear Logic Prover". TLLA 2021. https://hal-lirmm.ccsd.cnrs.fr/lirmm-03271501
- [E2] Click & coLLecT, site and help. https://click-and-collect.linear-logic.org/ (read 2026-10-09)
- [E3] Click & coLLecT repository, README. https://github.com/ComputerAidedLL/click-and-collect
- [E4] Click & coLLecT wiki. https://github.com/etiennecallies/click-and-collect/wiki
- [E5] ezyang, Logitext repository (BSD-3-Clause). https://github.com/ezyang/logitext
- [E6] Hedonistic Learning, "a purely client-side reimplementation of
  Logitext". https://www.hedonisticlearning.com/logic/description.html
- [E7] A. Ehle, N. Hundeshagen, M. Lange (2018). "The Sequent Calculus
  Trainer with Automated Reasoning – Helping Students to Find Proofs".
  ThEdu'17, EPTCS 267, 19–37. https://arxiv.org/abs/1803.01467
- [E8] J. Breitner (2016). "Visual Theorem Proving with the Incredible
  Proof Machine". ITP 2016, LNCS 9807, 123–139.
  https://www.joachim-breitner.de/publications/Incredible_ITP2016_preprint.pdf
- [E9] J. Breitner and contributors, The Incredible Proof Machine
  repository (MIT). https://github.com/nomeata/incredible
- [E10] G. Leach-Krouse (2026). "Carnap Ten Years Later: Lessons Learned
  and Next Steps". arXiv:2607.07722, accepted at TEAL. https://arxiv.org/abs/2607.07722
- [E11] G. Leach-Krouse, J. Ehrlich, "About Carnap". https://carnap.io/about
- [E12] J. Korkut (2023). "A Proof Tree Builder for Sequent Calculus and
  Hoare Logic". ThEdu'22, EPTCS 375, 54–62. https://arxiv.org/abs/2303.05865
- [E13] N. Karsten, F. K. Jacobsen, K. J. Eiken, U. Nestmann, J. Villadsen
  (2023). "ProofBuddy: A Proof Assistant for Learning and Monitoring".
  TFPIE 2023, EPTCS 382. https://arxiv.org/abs/2308.06970
- [E14] G. Geck, C. Quenkert, M. Schmellenkamp, J. Schmidt, F. Tschirbs,
  F. Vehlken, T. Zeume (2021). "Iltis: Teaching Logic in the Web".
  arXiv:2105.05763. https://arxiv.org/abs/2105.05763
- [E15] Iltis, site. https://iltis.rub.de/
- [E16] U. Z. Ahmed, S. Gulwani, A. Karkare (2013). "Automatically
  Generating Problems and Solutions for Natural Deduction". IJCAI 2013,
  1968–1975. https://www.ijcai.org/Proceedings/13/Papers/291.pdf
- [E17] J. Lodder, B. Heeren, J. Jeuring (2017). "Generating Hints and
  Feedback for Hilbert-style Axiomatic Proofs". SIGCSE 2017, 387–392;
  with W. Neijenhuis (2021), "Generation and Use of Hints and Feedback
  in a Hilbert-Style Axiomatic Proof Tutor", IJAIED 31(1), 99–133.
  https://cs.ou.nl/members/bastiaan/AxiomaticProofs.html
- [E18] L. D'Antoni, M. Helfrich, J. Kretinsky, E. Ramneantu,
  M. Weininger (2020). "Automata Tutor v3". arXiv:2005.01419.
  https://arxiv.org/abs/2005.01419
- [E19] A. Bentkamp, J. Eugster and contributors, lean4game README.
  https://github.com/leanprover-community/lean4game
- [E20] S. Matsuoka, Proof Net Calculator (v0.0.12, page updated
  2023-09-21). https://staff.aist.go.jp/s-matsuoka/PNCalculator/index.html
- [E21] C. Martens (2015). "Ceptre: A Language for Modeling Generative
  Interactive Systems". AIIDE 11(1), 51–57.
  https://ojs.aaai.org/index.php/AIIDE/article/view/12784
- [E22] F. Pfenning, 15-816 Linear Logic, CMU (spring 2012 and earlier).
  https://www.cs.cmu.edu/~fp/courses/15816-s12/
- [E23] *forall x: Calgary*, Appendix "Notes on accessibility".
  https://forallx.openlogicproject.org/html/A4.html
- [E24] W3C (2018). WAI-ARIA Graphics Module 1.0, Recommendation.
  https://www.w3.org/TR/graphics-aria-1.0/
- [E25] MathJax, "Accessibility Extensions".
  https://docs.mathjax.org/en/latest/basic/a11y-extensions.html

Sources checked 2026-10-09: 25 checked, 2 corrected, 0 removed, 1 claims marked.
