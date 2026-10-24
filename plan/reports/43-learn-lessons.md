# Step 43: Practice and Learn, from one store of problems

## Session 43a: the problem store and Practice

### The checklist

| requirement (the section "The problem store and Practice") | status | evidence |
|---|---|---|
| One store in the client's content, the single source of Practice and Learn: each problem's sequent, calculus, linlog's verdict, difficulty, categories, a session `proof` accepts when provable | done | `src/content/problems/` (425 problems: 392 provable, 33 unprovable); `src/content.config.ts` (the schema); Learn's problems are the store's entries with a `lesson`, Practice's the provable ones (`/problems.json`) |
| At least 300 provable, every level and every fragment represented, the hardest a real effort; step 38's and 38b's practice joins | done, the hardest for the author to judge | 392 provable, by level 76, 109, 123, 84; all twelve fragments and all six calculi (`store-size.py`); all 73 of step 38's and 38b's sequents are in the store, with their lessons; the validation's agents took 24 to 46 commands on the Very hard ones |
| Generated with intent, and chosen from the literature, each source read; nothing copied from LLTP or ILTP | done | `gen/gen.py` (built backwards from a planned proof shape, then disguised), `gen/pick.py` (kept only at the level aimed at); 45 problems are stated in a work read in itself and 4 derived from one, which their source line says (`evidence/classics.md`); no library file was read |
| A difficulty that means something for a human, from the literature and the problems' structure, validated by agents of different strengths, the levels ordering their success and effort | done for effort and first-attempt success, strictly at every level; impossible here for plain success between Easy, Moderate and Hard, confirmed by a third round and a fourth with a weaker solver, each declared in advance: 32 of 32 attempts solved at each of the three levels (below, "What cannot be shown") | Rounds 2 and 3 pooled, bound 30 commands and 6 minutes, 20 attempts per level: solved 20, 20, 20, 9; first-attempt success 19, 15, 11, 6; mean commands 4.2, 7.8, 15.2, 20.5. Round 1 (60 commands) ordered effort. The levels' names and boundaries are for the author to judge. `trial/`, `trial2/`, `trial3/`, `trial4/` |
| Checked by the flake: linlog's command over the whole store | done | the client's `problems` check (`modules/_checks/problems.py`), failing on a planted wrong entry and passing on the store (Verification) |
| Practice a place of its own, outside Learn's frame, from the bottom band at every width; only provable problems; filters by category and difficulty; a random one within the filters; a problem on a new tab; the hint ladder its help; the proved ones remembered | done, a test per behaviour | `src/islands/Practice.tsx`, `src/lib/practice.ts`. Its own place outside Learn's frame, from the band, offering the store's provable problems alone, key P: Playwright `practice.spec.ts` "Practice opens from the bottom band, outside Learn's frame, with the store's provable problems alone" and Vitest `practice.test.ts` "offers every provable problem of the store and none of its unprovable ones". At every width: "Practice is in the bottom band at every desktop width" (641 to 1920 pixels) and `phone.spec.ts` "Practice is in the bottom band at a phone's width, and a problem opens from it" (320, 360, 412). Filters by category and difficulty: "the filters show the problems of the chosen categories and levels, kept for the next visit" and Vitest "filters by category, any chosen within a group and every group at once, and by level". A random problem within the filters: "a random problem is one the filters show, and opens on a new tab beside the work open" and Vitest "picks a random problem among those the filters pass, and none from none". A problem on a new tab, the proved ones remembered: "a problem opens on a new tab, and once the visitor proves it, it is marked, also on the next visit" and "a problem the search closes is not marked proved". Closed by Escape or a press outside: `dismiss.spec.ts`. Its controls' sizes: `targets.spec.ts`. Learn's practice from the store: `learn.spec.ts` "a lesson's problem opens at once, and Watch plays its proof one step at a time" and `problems.test.ts` (the lessons' mix, level 3 per lesson); Learn keeps the unprovable: the schema's refinement "an unprovable problem is Learn's" |
| Learn keeps the unprovable; its practice rubric becomes the lessons' practice, from the store | done | the schema refuses an unprovable problem without a lesson; Learn's frame holds the tutorial (each lesson with its practice) and the reference |
| The education skills and `frontend-design` used | done | below (Decisions) |
| Before and after screenshots, both widths, both themes | done | `/home/tux/Projects/own/linlog-step43a-screenshots/before/` and `after/`, the same 24 names |
| A fresh-context reviewer, each finding answered | done | below (The review) |
| The client's `nix flake check` at its last commit; `jj st` clean in both workspaces | done | Verification |

### Outcome

The client has one store of 425 problems in `src/content/problems/`, which the page's
Practice and Learn both read. Each problem records its sequent, its calculus, linlog's
verdict, a level of difficulty, the categories it is found under, the fragment linlog
detects, and, when provable, a session that `linlog interact` wrote and `proof` accepts.
Of the 392 provable problems, 311 come from generators written for a target difficulty
(19 of them variants for weakening and Mix), 32 are step 38's own, and 49 cite a work
read in itself: 45 stated there (34 new classics and 11 of step 38's), 4 derived from it
and said so. The flake's new `problems` check runs linlog's command, at the client's
pinned input, over the whole store. It fails on a verdict or fragment that is not the
command's, on a provable problem without a session `proof` accepts, and on an
unprovable one with a session.

Practice is a place of its own, a sheet over the stage that the bottom band opens at
every width (key P). It offers the 392 provable problems, filtered by their connectives,
their logic, their source and their level, or one at random within the filters. A
problem opens as a proof on a new tab, and the problems the visitor has proved are
marked, also on the next visit. Learn's frame now holds the tutorial and the reference.
Its practice moved under the tutorial's lessons and is drawn from the store, so the
unprovable problems stay Learn's alone.

### The difficulty: the metric, its sources, its levels

- **What is measured.** Each problem is measured on the proof linlog's own hints give,
  the proof whose session the store keeps, never on the search's effort (`gen/measure.mjs`,
  through the pinned bindings). Three numbers come from that proof:
  - The proof's steps, the inferences a visitor clicks.
  - At every goal of the proof where no invertible rule applies (a focused visitor
    applies those first, the hint ladder's first rung), every move a visitor could make
    there: every rule on every formula, every split of the context, at most 1024 splits
    sampled for a context of more than ten formulas. linlog's search decides whether the
    goal stays provable after each move.
  - Of the moves that fail: shallow ones, which linlog refutes by an atom's count (a
    visitor who counts atoms sees them), and deep ones.
- **The score.** The bits of choice at a goal are log2(moves / sound moves); the bits
  left after counting are log2((moves − shallow) / sound moves). The score is
  `steps + 2 × (bits left after counting) + ½ × (bits a count settles)`, summed over the
  proof.
- **The levels.**

  | level | name | score |
  |---|---|---|
  | 1 | Easy | under 6 |
  | 2 | Moderate | 6 to under 14 |
  | 3 | Hard | 14 to under 30 |
  | 4 | Very hard | 30 and above |

  An unprovable problem has no proof to measure, so it keeps the level step 38 set by
  hand (1 to 3). A variant built from a measured problem takes its source's score plus
  one step (an unused hypothesis), or the sum of its two halves' plus one (two proofs
  under Mix, whose split the halves' disjoint atoms settle): in affine mode no wrong move
  is refuted by an atom's count, so the measure itself would count every one as deep.
  The score counts steps too, so a long proof with no hard choice can be Hard: 28 of the
  generated Hard problems have no choice left after counting atoms.
- **Sources** (`research/difficulty.md`, a research agent on Sonnet 5.5 at `medium`; every
  source opened and quoted with its page).
  - Proof size as the primary predictor. Ahmed, Gulwani and Karkare (IJCAI 2013) generate
    natural deduction problems by the shape of their proof and report a solver's success
    falling with its steps. Mendes, Marcos and Terrematte (arXiv 2603.07322) define
    proving complexity on minimal proofs, and say a formula's size is a weak proxy.
    Strannegård et al., as Mayn and van Deemter (arXiv 2204.12197) summarise them, found
    proof length the strongest correlate of people's accuracy; that is a secondary report.
  - The choice points. The complexity results say where proof search must guess: MLL is
    NP-complete (Kanovich, LICS 1992, its tensor's split), without exponentials the logic
    is PSPACE-complete, with them undecidable (Lincoln, Mitchell, Scedrov and Shankar
    1992). That these choices make a problem hard for a person is this session's
    hypothesis from those results, which the validation tests. No source studies human
    difficulty in linear logic.
  - The validation by solvers of different strengths follows TPTP's rating: a problem is
    rated by the fraction of systems that fail on it (Sutcliffe, Fuchs and Suttner,
    TR-ARP-11-00, pp. 6–7). Following Mayn and van Deemter, success and attempts are the
    measures.
  - Calibrated by hand. Among the weightings tried, the score's ordered step 38's
    hand-graded examples best: 499 concordant pairs to 80 (`gen/existing-measured.jsonl`).
    The boundaries put step 38's grades 1, 2 and 3 roughly at Easy, Moderate and Hard.
- **The validation** (`trial/`, before the store was final; the sample is the store's):
  - **The sample.** Twelve problems drawn by a fixed seed, three per level, at each level
    one classical, one intuitionistic and one of a third fragment (`trial/sample.json`).
  - **The agents.** Each of Haiku 5.5 and Sonnet 5.5, at `medium`, attempted each
    problem, one agent per problem and model (24 runs). Each was given `trial/rules.md`
    (the rules of both calculi and the commands) and `trial/play.py` alone, which runs
    `linlog interact` with only `goals`, `rules`, `apply`, `undo` and `show`, no search
    and no hint.
  - **The bounds.** At most 60 commands and 15 minutes per attempt, each command on
    cores 8 to 11 under a 2 GB cap with a 60-second limit. No attempt reached either
    bound.

  | level | model | solved | mean commands | mean refused steps | mean seconds |
  |---|---|---|---|---|---|
  | 1 Easy | Haiku | 3/3 | 4.7 | 0.7 | 11 |
  | 1 Easy | Sonnet | 3/3 | 4.7 | 0.0 | 11 |
  | 2 Moderate | Haiku | 3/3 | 9.7 | 0.3 | 26 |
  | 2 Moderate | Sonnet | 3/3 | 10.7 | 0.0 | 27 |
  | 3 Hard | Haiku | 3/3 | 13.3 | 0.0 | 48 |
  | 3 Hard | Sonnet | 3/3 | 13.7 | 0.0 | 54 |
  | 4 Very hard | Haiku | 3/3 | 36.3 | 1.7 | 106 |
  | 4 Very hard | Sonnet | 3/3 | 31.0 | 1.0 | 65 |

  Solved with no refused step and no undo (first-attempt success, Mayn and van Deemter's
  measure): Easy 2/3 and 3/3, Moderate 2/3 and 3/3, Hard 3/3 and 3/3, Very hard 0/3 and
  1/3 (Haiku, then Sonnet).

  - **What it shows.** Effort is ordered by the levels for both models: the mean
    commands and time grow at every step from Easy to Very hard. Across the twelve
    problems, the score's rank correlation (Spearman) with the commands is 0.97 for
    Haiku and 0.96 for Sonnet; with the time, 0.97 and 0.92.
  - **What the first round does not show.** Success does not separate the levels:
    every attempt succeeded within the bounds. Only success without a refused step or an
    undo drops, at Very hard alone.
- **The second round** (`trial2/`). Before any attempt, `trial2/PLAN.md` fixed the bound
  at 30 commands and 6 minutes, half the first round's commands: TPTP rates a problem by
  the systems that fail on it within a realistic resource limit. It also fixed a fresh
  sample, 4 problems per level (seed 4344, none of the first round's, two classical and
  two intuitionistic per level), and what would refute the levels. The same harness and
  the same models ran, 32 attempts.

  | level | model | solved | mean commands | mean refused steps | mean seconds |
  |---|---|---|---|---|---|
  | 1 Easy | Haiku | 4/4 | 5.0 | 0.0 | 12 |
  | 1 Easy | Sonnet | 4/4 | 5.2 | 0.5 | 12 |
  | 2 Moderate | Haiku | 4/4 | 7.5 | 0.0 | 22 |
  | 2 Moderate | Sonnet | 4/4 | 8.0 | 0.8 | 20 |
  | 3 Hard | Haiku | 4/4 | 13.0 | 0.2 | 46 |
  | 3 Hard | Sonnet | 4/4 | 15.8 | 0.5 | 49 |
  | 4 Very hard | Haiku | 2/4 | 17.5 | 0.2 | 90 |
  | 4 Very hard | Sonnet | 1/4 | 23.8 | 4.2 | 75 |

  Both models together:

  | level | solved | solved with no refused step and no undo | mean commands | mean seconds |
  |---|---|---|---|---|
  | 1 Easy | 8/8 | 7/8 | 5.1 | 12 |
  | 2 Moderate | 8/8 | 6/8 | 7.8 | 21 |
  | 3 Hard | 8/8 | 5/8 | 14.4 | 48 |
  | 4 Very hard | 3/8 | 2/8 | 20.6 | 82 |

  - Effort rises at every level, as in the first round.
  - First-attempt success (Mayn and van Deemter's measure) falls at every level.
  - Success within the bound never rises: it is equal from Easy to Hard and falls at
    Very hard.
  - So the levels order both success and effort, in this sense. First-attempt success
    and effort are strictly ordered: each level is below the next. Plain success within
    the bound is ordered without being strict: the problems up to Hard are within both
    models' reach (8/8 each, a ceiling), and it separates Very hard from the rest.
    Separating Easy, Moderate and Hard by plain success would need a bound chosen to
    make them fail. This session did not choose one after seeing the data.
  - The failures were real dead ends or overruns. Sonnet followed a hypothesis that
    cannot give `C` on `g299` and undid eight steps. Haiku and Sonnet found their routes
    through the copies of a `!`-hypothesis too long for the bound on `g263`.
  - A bound counts steps, so a very long proof fails it by length too. `g299`'s stored
    proof has 49 steps, though no attempt needs the stored proof (Haiku proved `g250`,
    whose stored proof has 70 steps, in 28 commands). That is part of what Very hard
    means here: long and full of choices.
  - Sonnet did not do better than Haiku. The levels are about the problems; the two
    models' strengths, as a pair of solvers, did not separate in this task.
  - **Where the choice term shows.** The proof's steps alone rank the commands as well
    (0.97 and 0.99), so the choice term adds no rank information here. It shows on
    `g291`, a problem whose proof is short (15 steps) but which needs three copies of a
    `?`-formula. Haiku spent 39 commands and 6 undos on it, after a split that left a goal
    without a copy.
  - **For the author.** The Very hard problems took the agents 24 to 46 commands; whether
    that is "a real effort for someone who knows the rules" is the author's to judge. A
    stricter bound, or people instead of agents, would test the levels' separation by
    success.

### What cannot be shown

The section asks that the levels order the agents' success. Plain success, solved or not
within the bound, does not separate Easy, Moderate and Hard with these two agents. A third
round and a fourth with a weaker solver, each declared before it ran in order to test
exactly this, confirm it.

- **The third round** (`trial3/`, `trial3/PLAN.md` written before any attempt): the same
  bound as the second round (30 commands, 6 minutes), the same harness, rules and
  agents, and 6 fresh problems per level (seed 4345, none of the earlier rounds'), 48
  attempts. Its declared test was that plain success, pooled with round 2, falls
  between each pair of neighbouring levels. It did not.

  Rounds 2 and 3 pooled (20 attempts per level):

  | level | solved | first-attempt success | mean commands | mean seconds |
  |---|---|---|---|---|
  | 1 Easy | 20/20 | 19/20 | 4.2 | 10 |
  | 2 Moderate | 20/20 | 15/20 | 7.8 | 21 |
  | 3 Hard | 20/20 | 11/20 | 15.2 | 45 |
  | 4 Very hard | 9/20 | 6/20 | 20.5 | 73 |

- **The fourth round** (`trial4/`, `trial4/PLAN.md` written before any attempt): a
  weaker solver, Haiku 5.5 at `low` effort, under the same bound and harness, on 6 fresh
  problems per level (seed 4346), 24 attempts. The brief named `medium`; a solver at `low`
  departs from it, to test the strengths the goal asks for. Its declared test, plain
  success falling between neighbouring levels, did not hold either: solved 6/6, 6/6,
  6/6, 4/6, mean commands 5.0, 7.8, 17.3, 19.3.
- Across all four rounds and three solver strengths (Haiku at `low` and `medium`,
  Sonnet at `medium`), every Easy, Moderate and Hard problem was proved: 32 of 32
  attempts at each of the three levels. Success there is at its ceiling, and neither a
  larger sample nor a weaker solver moved it. With these agents, plain success cannot
  order the three lower levels.
- Only a bound tightened after seeing the data would separate them, set between the
  commands Moderate and Hard took (7.8 and 15.2 on average). Such a bound would show the
  bound chosen, not the levels, so none was used. Weaker solvers would fail earlier, but
  the brief names the two agents, at `medium`.
- What the levels do order:
  - Effort, strictly at every level, in all three rounds.
  - First-attempt success, strictly at every level: 19, 15, 11 and 6 of 20. A refused
    step or an undo counts as a failure, the measure Mayn and van Deemter validate a
    difficulty estimate against.
  - Plain success, which never rises, and separates Very hard (9/20) from the rest.
- Students instead of agents would test plain success for people; that is the author's
  to decide.

### Decisions

- **The store is Markdown, one file per problem**, as step 38's examples were (D23):
  `src/content/problems/<id>.md` with its frontmatter, and a provable problem's
  `<id>.session.json` beside it.
  - Generated problems have no title: Practice shows the sequent itself.
  - Learn's problems keep their titles, explanations, lessons and order. Step 38's
    examples moved there unchanged but for the level, the categories, the fragment and,
    where a work states them, their source.
- **Practice's problems are a JSON file the page fetches when Practice first opens**
  (`src/pages/problems.json.ts`), not the island's props. 392 problems would have added
  about 74 KB to the page's first load, which is 120 KB today. The service worker caches
  the file with the rest, so Practice works offline.
- **The categories** come from the connectives a problem uses (`multiplicatives`,
  `additives`, `units`, `exponentials`, D60's lessons 2 to 5), its logic (`classical`,
  `intuitionistic`, and `structural` for weakening or Mix), and `literature` for a problem
  with its source.
  - The filter combines them as `practice-problem-sequence-designer` and
    `interleaving-unit-planner` argue: within a group, any chosen category (a visitor picks
    the topics to practise); across the groups, all at once.
  - With no filter, the random pick draws from every category, which interleaves them
    (Rohrer; Taylor and Rohrer, as the skill cites them).
  - The fragment linlog detects is shown on each problem and recorded in the store.
    Twelve fragment chips would have crowded the filter, so the fragment is not one.
- **The levels**: four qualitatively different stages, as `learning-progression-builder`
  asks.
  - Easy: every choice forced or settled by counting atoms.
  - Moderate: a few choices, mostly settled by counting.
  - Hard: longer proofs, or choices a count does not settle.
  - Very hard: long proofs with several interacting choices or copies (a generated one
    needs at least 4 bits of choice a count does not settle).
  - `cognitive-load-analyser`'s element interactivity (Sweller 1994) is why several open
    splits and copies count more than steps. That is the theory's motivation, not
    evidence.
- **The generator** (`gen/gen.py`) works in three phases.
  - **The plan.** It draws a plan for the level aimed at: how many context splits
    (`⊗R`, `⊸L`), branches (`&R`, `⊕L`), side choices beside a distractor (`⊕R`, `&L`),
    copies of a `!`-hypothesis, promotions, units, and invertible steps.
  - **The proof.** It grows a proof from its conclusion, whose formulas start as holes
    that the rules applied to them fill. It closes every goal by axioms, an identity, or a
    tensor of what is left.
  - **The disguise.** It reorders the hypotheses and renames the atoms onto a small
    alphabet: the fewer letters, the more splits look plausible.

  Of 10,000 candidates, linlog proved all and measured each (`gen/meas.jsonl`).
  `gen/pick.py` kept only those whose measured level is the level aimed at, then
  balanced the levels across the twelve fragments. It also refused, among others, very
  hard ones without real choices (fewer than 4 bits left after counting).
  - Nineteen variants add the affine and Mix calculi: an unused hypothesis, and two
    proofs side by side under one `⅋`, one of them with both in affine Mix.
  - Eight generated problems repeated another's sequent in the other calculus and were
    dropped. The store writes no sequent twice in one calculus (a test). Step 38's
    `tensor-par` and `mix-net` share a sequent on purpose, in two calculi.
- **The sessions** are written as step 38's were. `linlog interact` applies the steps
  linlog's own hints give, then `save` and `proof` exit 0 for every one
  (`evidence/interact.txt`), so Watch plays them step by step.
- **The classics** (`evidence/classics.md`) are read in two works.
  - The Handbook of Linear Logic (the LLHandbook project's draft of 6 March 2026,
    §1.3.3, pp. 16–18, and §1.7.3, p. 49): read by this session from the PDF's text.
  - Wikipedia's "Linear logic": its "Remarkable formulas" and "Extending
    classical/intuitionistic logic", read in the article's wikitext.
  - The research agent's other sources (the Stanford Encyclopedia, nLab) agree. Its PDFs
    (Girard, Wadler, Troelstra) could not be read, so nothing is taken from them.
  - The K, W and S combinators and modus ponens under Girard's translation are this
    session's derivation from Wikipedia's sentence that "intuitionistic implication is
    encoded as !A ⊸ B"; each one's source line begins "Derived here from" and names what
    the encoding is applied to.
  - A single sequent stated in a work is a fact of logic. No problem library was opened.
- **Learn's lessons from the second keep two provable problems at level 3 or above**,
  38b's requirement in the new scale. Under the metric, lessons 4 and 8 had none, so
  four problems of the store joined them: two of the units (`g213`, `g214`) and two for
  proof nets (`g173`, `g206`). A near miss for lesson 8 (`near-net`, refuted by an atom's
  count) keeps the lesson's mix. Two unprovable classics of the Handbook (`⊥ ⊗ ⊥` and
  `⊥ & ⊥`) joined lesson 4.
- **The flake check** runs one batch: `linlog prove --input-format jsonl --format json
  --deterministic --timeout 20s` over the whole store. It compares each verdict, fragment
  and sequent, then resumes each session with `interact --state … proof`. It takes about
  a second.
- **The band.** Practice sits beside Learn, with the label "Practice" and a dumbbell.
  With Learn on, the band packs its actions tighter so that it stays one row at 1280
  pixels.
- `frontend-design` for the look: one memorable element, the level as a stair of four
  steps with its name beside it, in the accent; everything else in step 38's grammar.
  - The chips are labels in the 44-pixel grammar; a chosen one is shown by its outline,
    weight and a check mark, never colour alone.
  - The list is dense rows rather than cards, the sequent in Euler Math as each row's
    subject.
  - No new colour token: one size token, `--size-practice`.

### Judgement calls

- **On a phone, Export moved to More's sheet** to give Practice a place in the band. Seven
  44-pixel buttons fit 320 pixels, eight would not. Export is the action least used
  while proving on a phone. (Departs from step 38's phone band, not from a decision of
  the design.)
- **A problem opens on a new tab** through the page's existing action: it replaces only
  an empty tab, as Learn's problems always did. Work open stays open (a test).
- **"Proved" is remembered** under `linlog-practice`, with the filters. A problem counts
  as proved when the tab Practice opened it on finishes its proof with no goal closed by
  the search (Close, Close all); a step the hint ladder's last rung applies counts, since
  the visitor asked for it and took it. The privacy page lists the key; its legal ground is the article it
  already cites.
- **The Practice place is a menu**, in the page's terms: Escape and a press outside close
  it, as every sheet does, and its tests sit with them (`dismiss.spec.ts`).
- **`unknown` is not in the store**: every problem's verdict is proved or unprovable.
  `!(A + B) |- !A + !B` (Wikipedia's converse) stayed out, since linlog gave no verdict
  within the budget. `1 |- bot` with Mix stayed out too: the Handbook's `mix₀` is not
  linlog's Mix, which refutes it.
- **Tests changed on purpose**:
  - Learn's frame test: two rubrics, the tutorial first.
  - The example test opens Learn rather than Practice.
  - The difficulty's wording: a level's name rather than "Difficulty 2 of 3".
  - 38b's hard problems per lesson: "level 3 or above" rather than "difficulty 3".
  - The targets walk measures Practice, and the chips as targets.

### Departures from the design

- The validation's fourth round ran Haiku 5.5 at `low` effort, where the brief names
  `medium`, to try a weaker solver (above, "What cannot be shown").

- Practice is a place of its own outside Learn's frame (D47), at the author's request, as
  the section says. Learn's frame keeps two of D47's three rubrics, the tutorial and the
  reference. Each lesson's practice sits under its lesson, as D60 orders. (departs from
  the design: D47, at the author's request)
- The phone's band (step 38) gives Export's place to Practice (above).

### The review

One round, a fresh-context reviewer on Opus 5.5 at `high`, told to report only what breaks
a stated requirement (`/home/tux/Projects/own/linlog-step43a-screenshots/review.md`).
It confirmed:
- the store check (exit 0), and that its `proof` step refuses an unfinished session;
- step 38's 73 examples moved intact;
- that only provable problems reach Practice;
- the Handbook's and Wikipedia's citations, against the works;
- the validation's figures, from the logs;
- that the trial agents used only the rules and the harness.

Its findings, each answered:
1. **The 18 variants aimed at no level, and several were misranked** (an affine identity
   was Hard; Mix ones were Very hard with no choice left after counting). Also, 28
   generated Hard problems have no choice left after counting, though the report said
   Hard means such choices. **Holds.**
   - In affine mode no wrong move is refuted by an atom's count, so the measure counted
     every one as deep.
   - Each variant now takes its linear source's score plus one step, and a Mix one the
     sum of its halves' scores plus one (client `uuozkosr`); nine moved down a level.
   - The level descriptions now say what the score does: it counts steps too, so a long
     proof can be Hard.
2. **The validation does not show success ordered by the levels, yet the checklist said
   done.** **Holds** for the first round. A second round followed, its bound and sample
   declared before any attempt (`trial2/PLAN.md`). In it success never rises from level
   to level and first-attempt success falls at every level (above).
3. **The four sequents of Girard's translation are not in the cited section.** **Holds.**
   Their source lines now begin "Derived here from" and name what the encoding is applied
   to. The count is now 45 stated and 4 derived.
4. **"Proved" recorded proofs the visitor did not make** (Close all, Watch). **Holds.**
   Only the tab Practice opened counts, and only with no goal closed by the search. One
   test proves a problem by the visitor's own steps; another shows that Close all does
   not mark one.
5. **`near-net` was missing from the evidence.** **Holds.** It is added to
   `evidence/store-final.jsonl`, with linlog's refutation.

Borderline: there was no problem in affine Mix. Answered by one (`g319`), so every
calculus the page proves in has problems.

### Verification

- **The client's `nix flake check`** passed (exit status 0) at its last commit,
  `uuozkosr`, under the machine lock. It ran on cores 8 to 11 in a capped scope; the log
  is `evidence/flake-check.log`. Its checks: `site`, `unit`, `typecheck`, `lint`, `e2e`,
  the new `problems`, `csp`, `licences`, `no-style-attributes`, `pages-paths`,
  `licence-headers`, `actionlint`, `deadnix`, `treefmt`, `claude-hooks` and
  `playwright-version`.
- **The `problems` check.** It fails on two planted entries (`A * B |- A` filed as
  provable, and a provable problem without a session): "2 of 427 problems fail:
  planted-nosession: provable, but no session proves it; planted-wrong: the store says
  provable, linlog says unprovable", exit 1. With them removed it passes: "all 425
  problems are as linlog's command decides them; 392 provable, each with a session that
  `proof` accepts", exit 0.
- **The tests.**
  - Vitest: 481 passed. They include the store's tests, one per problem, and Practice's
    logic, with the real offering.
  - Playwright: 87 passed before the review's changes; the review's Practice and phone
    tests (10) passed after them; the flake's `e2e` ran them all at `uuozkosr`.
  - Biome and `astro check`: 0 errors.
- **The sessions.** Every new session was written by `linlog interact` with `proof` exit 0
  (`evidence/interact.txt`).
- **Signing.** A signature was tested before each commit, and every commit is signed.
- **The screenshots.** `before/` and `after/` hold the same 24 names: `01-band`,
  `02-practice`, `03-practice-filtered`, `04-problem-opened`, `05-learn` and
  `06-learn-lesson`, each at a phone's and a desktop's width in both themes.
  `shots.mjs` beside them is the script. Before, the scenes show Learn's practice rubric;
  after, Practice's place.
- **The client's commits**, on `uwxqplov` (`main`):

  | change | what |
  |---|---|
  | `xoooyxpw` | Practice as a place of its own, from one store of problems |
  | `svsnvvqm` | the problems that cite a source filed under the literature; only the lessons numbered |
  | `uuozkosr` | the review's answers: variants levelled from their sources, derived classics said so, proved means the visitor's proof |

### What is left open

- Later steps add their problems to the store in the sessions that teach them (39's nets
  with boxes, 40's two-sided surface, 41's cut, 42's quantifiers). Each new problem is
  measured as above before it is filed, and checked by the flake.
- The validation is by agents; students would test the levels for people, and Moderate
  and Hard are told apart by effort and first-attempt success, not by plain success.
- The level of an unprovable problem is a hand grade; a measure for "how hard to see
  why not" (lesson 7's erroneous examples) is 43c's.

### Repositories and workspaces

- The client: `/home/tux/Projects/own/linlog-prover.github.io-43a`, a jj workspace on
  the client's `uwxqplov`, its commits listed under Verification; nothing pushed.
- linlog: `../linlog-step43a` on `zpwypsws`, this report's commit; no code changed.
- The evidence: `/home/tux/Projects/own/linlog-step43a-screenshots/`.
  - `gen/`: the generators, the measure, the picking, the store's build, the classics.
  - `evidence/`: each problem's measures, the interact exits, the classics' passages.
  - `research/`: the research agents' notes.
  - `trial/` to `trial4/`: the validation's four rounds, each with its rules, harness, sample, logs and table (`trial2/PLAN.md` to `trial4/PLAN.md` written before their rounds ran).
  - `before/` and `after/`: the screenshots, listed by `ls`.
  - `store-size.py`: the store's size by level and category.

## Session 43b: lessons 1 to 5

### The checklist

| requirement (the brief and the section "Goal") | status | evidence |
|---|---|---|
| The outline of lessons 1 to 5 sent to `planning` before the lessons were written, and the author's answer | done: confirmed as sent | sent 2026-10-24 10:09 (`/home/tux/Projects/own/linlog-step43b-screenshots/outline.md`); `planning` relayed at 10:11: "The author confirmed the outline of lessons 1 to 5 as you sent it at 10:09: the order, the worked and faded proofs, the hinge questions, and the two new store problems (`extra`, `copy-drop`)" |
| Lessons 1 to 5 in D56's sequence: a worked proof played with a reason at each inference and "predict the next rule" before each reveal, the same proof with its last steps left to the student, practice from the store, a self-explanation prompt at the densest step, a hinge question whose wrong answers each name a misconception | done | `src/content/lessons/1-reading.md` to `5-exponentials.md`, `LessonView` in `src/islands/Learn.tsx`; Playwright `tests/lessons.spec.ts` (7 tests); Vitest `src/content/lessons.test.ts` (14 tests) |
| Lesson 2's erroneous example, the legal split that leads nowhere | done | `detour` in `2-multiplicatives.md`, played on the stage with linlog's verdict ("Cannot close") and taken back by the next step; `lessons.test.ts` holds that `split_passes` refuses it and that linlog applies it; Playwright "the dead end is a legal split played on the stage, which cannot close, then taken back" |
| The hinge's answer decides between the next lesson and the faded proof again | done | Playwright "the hinge question names a wrong answer's misconception and offers the faded proof, the right one the next lesson" |
| The reference's rule schemas the lessons use | done | `src/content/rules/` (13 rules), drawn by `Rules` into `texts/reference.md`; Playwright "the reference draws every rule of the lessons as a schema, with its two-sided names" |
| The frame's hover explanations (D47): a rule its schema, a connective its reading, a verdict its reason | done | Playwright "with Learn on, a rule of the popover shows its schema and a formula its reading; off, neither"; a verdict's reason is the note's and the status's tip, as before |
| Every lesson's sequent a store problem with linlog's verdict and session; the store's flake check passing | done | three new problems (`extra`, `copy-drop`, `bare-beside-bang`), `evidence/new-problems.txt`, `evidence/interact.txt`; `check-problems`: "all 428 problems are as linlog's command decides them; 393 provable, each with a session that `proof` accepts" |
| The education skills and `frontend-design` used | done | below, "Decisions" |
| One fresh-context reviewer, one round, each finding answered | done | below, "The review"; `/home/tux/Projects/own/linlog-step43b-screenshots/review.md` |
| Before-and-after screenshots of each lesson at a phone's and a desktop's width in both themes | done | `/home/tux/Projects/own/linlog-step43b-screenshots/before/` (24) and `after/` (92) |
| The client's `nix flake check` at its last commit; `jj st` clean in both workspaces | done | exit status 0 at `qmlqvzzn` (19116070); below, "Verification" |

### Outcome

Learn's tutorial has its first five lessons written, in D60's order, as the author
confirmed them from the outline: reading a sequent, the multiplicatives, the additives,
the units, the exponentials. Each opens from the tutorial's list as a lesson of its own:
a short opening in plain words; a worked proof of the store played on the stage one
inference at a time, the visitor predicting each rule among the lesson's rules before the
page plays it and writes its reason beside the inference; a self-explanation prompt after
the densest step, with a model answer to compare; the same proof with its first steps
played and the rest left to the visitor; the lesson's practice from the store; and a
hinge question whose right answer offers the next lesson and whose wrong answers each say
the misconception they come from and offer the faded proof again. Lesson 2 also shows the
legal split that leads nowhere: the page plays it, its two dead premises drawn with linlog's verdict "Cannot close", and the next step takes it back and plays the right split. Lessons
6 to 9 keep their practice until 43c writes them.

The reference draws the thirteen one-sided rules the lessons use as schemas, each with
what it does, its two-sided names and the lesson that teaches it, cited from the Handbook
of linear logic's Table 1.4. With Learn on, a rule in the stage's popover shows its schema
on hover, and a formula its connective's name and reading; with Learn off, neither.

Three problems joined the store (428 problems, 393 provable), each decided by linlog's
command and checked by the flake's `problems` check. On the way the touch-target walk,
extended by the lesson's states, found that the stage's zoom-to-floor fell short on a
phone (a formula's button 39 pixels wide), which is fixed at its cause.

### The lessons

| lesson | worked proof (store) | faded: played, then the visitor's | self-explanation (after step) | hinge | misconceptions named |
|---|---|---|---|---|---|
| 1 Reading a sequent | `identity`, `A ⊢ A`: ax | none played; the one step | why `⊢ A⊥, A` is an axiom and `⊢ A, A` is not (1) | which goal the axiom closes by itself | the axiom pairs a formula with itself; an unused formula may be left over; any atom closes with any negated atom |
| 2 The multiplicatives | `commute`, `A ⊗ B ⊢ B ⊗ A`: ⅋, ⊗, ax, ax; the dead end played before ⊗ and taken back | ⅋; then ⊗, ax, ax | where the left premise's formulas come from (2, the ⊗) | which split of `(A ⊗ B) ⊗ C` can pass | a factor needs one formula beside it, however many atoms it has; a premise can do without its atoms' duals |
| 3 The additives | `distributivity`, `A ⊗ (B ⊕ C) ⊢ (A ⊗ B) ⊕ (A ⊗ C)`: ⅋, &, ⊕₁, ⊕₂, ⊗, ⊗, four ax | ⅋, &; then the choices, splits, axioms | why each premise of `&` keeps the whole context (2, the &) | after `&` on `B & C`, what the premises hold | `&` splits as `⊗`; `&` chooses as `⊕`; `&` puts its parts side by side as `⅋` |
| 4 The units | `units-together`, `(A ⊗ 1) & (1 ⊗ A) ⊢ A ⊗ (1 ⊕ 0)`: ⊕₁, ⅋, ⊥, ⊗, ax, ⊕₁, 1 | ⊕₁, ⅋, ⊥; then the split onward | why `1`'s premise holds nothing else (4, the ⊗) | which goal has a proof | `1` is true and closes beside an axiom; `0` can be left over; the empty goal is proved |
| 5 The exponentials | `copy-drop` (new), `!A, !B ⊢ !A ⊗ A`: ?w, ?c, ⊗, !, ?d, ?d, ax, ax | ?w, ?c, ⊗; then the promotion onward | why `!` needs every other formula under `?` (4, the !) | which has a proof | `!` on any formula one has; `?` gives copies as `!` does; any hypothesis may be copied |

Each lesson's practice is its store problems (lesson 1: 6, lesson 2: 9, lesson 3: 9,
lesson 4: 15, lesson 5: 13), easiest first, provable and unprovable mixed.

### Decisions

- **The lessons are content, one Markdown file each** (`src/content/lessons/`, D23): the
  opening is the body; the front matter names the worked proof (a store problem of the
  same lesson), one rule and reason per step of its session, the rules to predict among,
  how many steps the faded proof plays, the self-explanation prompt and its model answer,
  the hinge question, and lesson 2's dead end. `lessons.test.ts` holds each lesson's
  steps to its session's history, rule by rule, so a session rewritten in another order
  fails the test rather than misplacing the reasons.
- **The worked proof is the session played by Redo** on a tab of its own (`prover.worked`,
  new in `state.tsx`): the session replayed to the lesson's place, the rest on the redo
  stack. The faded proof is the same call with the rest left to the visitor. Both reuse
  the lesson's tab while it holds the problem's sequent, so a lesson opens one tab, not
  one per press. If the visitor works on the tab meanwhile, the next prediction replays it
  to where the lesson stands.
- **The reason stands on the stage beside its inference** (D22), never in Learn's panel:
  a note in Learn's colour on the rule label's row, past the drawing's right edge so that
  it covers no goal; under the drawing where the stage leaves no room there; above the
  stage's controls on a phone. The panel says only whether the prediction was right. This
  is the lessons' one memorable element (`frontend-design`); everything else keeps step
  38's grammar, tokens and 44-pixel targets.
- **The prediction is a row of the lesson's rules as buttons**, with "Show the step" for a
  visitor who does not want to guess (the expertise-reversal guard of
  `worked-example-fading-designer`). Every press plays the step; a wrong guess is answered
  with the right rule, never a refusal (`digital-worked-example-sequence`: immediate,
  specific feedback).
- **The faded proof leaves each lesson's new choice to the visitor** (the split, the side of
  `⊕`, the promotion), not only the closing axioms. `worked-example-fading-designer` fades
  from the end and keeps the conceptual step supported longest across several completion
  problems; D56 has one faded proof, and the visitor has just watched the same proof, so
  the one faded proof starts before the lesson's choice.
- **The self-explanation prompt is free text kept nowhere, then a model answer**
  (`self-explanation-prompt-designer`: generation, not recognition), placed after the step
  D60 names, which is each lesson's densest: the axiom, the split, the `&`, the split
  around `1`, the promotion. It appears once its step is played and stays.
- **The hinge questions** (`hinge-question-designer`): each wrong answer is the right
  answer under one misconception, worded as that idea ("This answer comes from the idea
  that …"), and why it fails. Where an option is a sequent, it is a store problem of the
  lesson, and the test holds that it is provable exactly when it is the right answer.
  Lessons 2 and 3 ask about a split and about premises, which linlog decided
  (`evidence/hinge-premises.txt`).
- **The order** (`learning-progression-builder`): D60's, confirmed by the author. Each
  lesson needs the one before: lesson 2's split needs lesson 1's axiom; the additives'
  practice uses `⊗`; the units are the multiplicatives' and the additives' units; the
  exponentials' copies go through `⊗` and `?w` meets `1` in the practice.
- **Cognitive load** (`cognitive-load-analyser`): the four parts of a lesson stand in
  order along a rule in Learn's colour; the worked proof shows one prompt at a time;
  the reason is at the inference, where the eye already is (split attention); the
  opening is a paragraph or two.
- **The rules' schemas are content too** (`src/content/rules/`), drawn by the island into
  the reference's text and into the popover's tip from the same files: the Handbook's
  Table 1.4, one-sided (read in the Handbook's own PDF, fetched 2026-10-24, p. 22), with
  the axiom on atoms as linlog applies it (§1.3.2, p. 14, says atoms suffice), and the
  two-sided names from linlog's `ClassicalRule` table. A two-sided name in the popover
  (an intuitionistic tab) shows the one-sided rule it names.
- **The hover readings** are `READINGS` in `words.ts`, which a test holds to the reference
  table's name and reading columns, so the two cannot drift apart.
- **`yaml` became a dev dependency** at the version the lock already held (2.9.1, under
  Astro): the lessons' front matter nests, and the test reads it.

### Judgement calls

- **A third new store problem, `bare-beside-bang`** (`!A, B ⊢ !A ⊗ !B`, unprovable,
  level 2 by hand), beside the two the outline named. Lesson 5's worked proof
  (`copy-drop`) must be a store problem of lesson 5, which made the lesson's ninth
  provable problem; with three unprovable ones, the store's test that a lesson's practice
  is never three alike in a row cannot hold (at most eight provable problems fit between
  three unprovable ones). A fourth unprovable problem was needed. This one shows the
  lesson's stuck point, promotion's context. The test was not loosened.
- **Lesson 1's worked proof has one inference** (`A ⊢ A`, ax), so its faded proof leaves
  that one step to the visitor; the outline said so and the author confirmed it.
- **A wrong prediction still plays the step**, with the right rule named: the worked
  proof is the page's, and a refusal there would make the visitor guess until right,
  which the outline did not ask for.
- **The reason is placed under the drawing** when the drawing's right edge leaves no
  room on the stage (lesson 3's `&` on a desktop), and above the stage's controls on a
  phone, where it may cover the root's row but never the active goal.
- **The hinge's misconception is worded as the idea behind the answer** ("This answer
  comes from the idea that …"), so that it names a belief, not a fault of the visitor.
- **A verdict's reason stays a tip with Learn off** (review finding 3, below).
- **The floor's fix** is outside the lessons, but the walk that measures the lessons'
  states failed on it, and a failing check is fixed at its cause.

### Departures from the design

- None from D21, D22, D47, D56 or D60. The reference gives each one-sided schema with its
  two-sided names (D47: "in the one-sided and the two-sided names"); the two-sided schemas
  themselves come with lesson 6 (43c).

### The review

One round, a fresh-context reviewer on Opus 5.5 at `high`, briefed with D21, D22, D47,
D56, D60, the confirmed outline and the machine's rules (linlog's command alone, capped,
on cores 10 and 11), told to report only what breaks a stated requirement
(`/home/tux/Projects/own/linlog-step43b-screenshots/review.md`). It confirmed every
verdict and session the lessons use, that each lesson's steps follow its session and
each reason fits its goal, the hinges' splits and premises, the dead end, every part D56
and D60 ask for, the schemas against Table 1.4 and the two-sided names against linlog's
table, and that the new hovers are off with Learn off. Its findings, each answered:

1. **"Linear logic drops nothing that is not under `?`" is false**: `⊤` closes a goal
   whatever else is in it (`linlog prove "|- ~A, A, B, top"`: provable). **Holds.** Lesson
   1's hinge, lesson 4's model answer and `extra`'s explanation now say that only `⊤` or
   `?w` disposes of a formula, and lesson 4's answer gives the true reason: the rule `1`
   has the conclusion `⊢ 1` (client `qmlqvzzn`).
2. **Lesson 2's hinge named a misconception that does not lead to its option**: in
   "`A⊥` with `A ⊗ B`; `B⊥, C⊥` with `C`" the factor `C` takes two formulas. **Holds.** The
   misconception is now "a factor of `⊗` needs one formula beside it, however many atoms it
   has", which gives `A ⊗ B` one dual and `C` the rest. The confirmed outline had the old
   wording; the fix keeps the option and the answer.
3. **A verdict's reason shows on hover with Learn off**, against D47's "with Learn off
   none of this". **Does not hold.** D15 gives every visitor the verdict's reason in one
   sentence, and the note's and the status's tips carried it before Learn's frame
   existed (2026-10-19); D47's sentence is about what the frame adds. Taking the reason
   away from the expert's page would break D15.
4. **`mix` and `wk` showed an empty tip with Learn on**: no rule file covers them.
   **Holds.** A rule the reference has no schema of now shows no tip. The schemas of Mix
   and weakening are not lessons 1 to 5's.
5. **Departures from the confirmed outline, not yet named.**
   - a. Lesson 1's choices lacked "no rule". **Holds**: added.
   - b. A third new problem. **Holds**: named above (judgement calls) and to `planning`.
   - c. The dead end only selected the split in the chooser, where the outline has the
     page play it, show it fail and take it back. **Holds**: the page now plays it, its
     premises drawn with linlog's verdict, and the next step takes it back
     (Playwright "the dead end is a legal split played on the stage, which cannot close,
     then taken back").
   - d. The outline said lesson 2 has eleven store problems; it has nine. **Holds**: the
     outline's count was wrong, the lesson's practice was not; this report says nine.

### Verification

- **The client's `nix flake check`** passed, exit status 0, at its last commit `qmlqvzzn`
  (19116070), under the machine lock on cores 10 and 11
  (`/home/tux/Projects/own/linlog-step43b-screenshots/evidence/flake-check.log`):
  `check-problems` "all 428 problems are as linlog's command decides them; 393 provable,
  each with a session that `proof` accepts", Vitest 504 passed, Playwright 103 passed. It
  passed before the review too, at `nnkrvpzp`.
- **Vitest** (504): `lessons.test.ts` holds each lesson to its session, rule by rule, the
  faded and prompt steps within it, the dead end refused by `split_passes` and applied by
  linlog, one right hinge answer and each wrong one with a misconception, an option's
  problem provable exactly when it is right, a rule file for every rule a worked proof
  uses, and the hover readings equal to the reference table's; `words.test.ts` the
  readings.
- **Playwright** (103): `tests/lessons.spec.ts`, seven tests, one per interaction (the
  worked proof with its predictions and the reason beside the inference, the dead end, the
  self-explanation prompt, the faded proof finished by the visitor, the hinge's two
  outcomes, the hover schema and reading with Learn on and off, the reference's
  schemas); the touch-target walk now includes a lesson's five states. Four tests changed
  on purpose, because lessons 1 to 5 now open as lessons: the practice tests open a lesson
  by its button (`learn.spec.ts`, `phone.spec.ts`, `engines.spec.ts`).
- **The walk failed before the floor's fix**: "lesson's worked proof: button.fb "B,
  formula 2 of goal 3" 39×55" at a phone's width; it passes after (client `snmtlozz`).
- **The sequents**: the three new problems by `linlog prove` and `seq fragment`
  (`evidence/new-problems.txt`), `copy-drop`'s session by `linlog interact` from linlog's
  hints, `proof` exit 0, and again resumed from the file (`evidence/interact.txt`); the
  hinges' premises (`evidence/hinge-premises.txt`); `copy-drop` measured by step 43a's
  measure at score 21.1, Hard (`scripts/worked-measured.jsonl`).
- **Signing**: a signature was tested before each commit; every commit is signed.
- **The screenshots**: `before/` (24: the five lessons and the reference, each at a
  phone's and a desktop's width in both themes) and `after/` (92: the same 24 names, and
  for each lesson its worked proof in progress, its faded proof and its hinge answered,
  lesson 2's dead end, and a rule's schema on hover), taken by `shots.mjs` from the
  built site at munpltsm and at `qmlqvzzn`.
- **The client's commits**, above `munpltsm`:

  | change | what |
  |---|---|
  | `snsvoswm` | the problems lessons 1 and 5 need, filed in the store |
  | `snmtlozz` | the zoom-to-floor solved per button, the padding apart |
  | `tmmtqlrt` | `yaml` as a dev dependency, at the version the lock held |
  | `nnkrvpzp` | lessons 1 to 5, the rules' schemas, Learn's hover explanations |
  | `qmlqvzzn` | the review's answers |

### What is left open

- Lessons 6 to 9 (43c): two sides with the two-sided schemas, when there is no proof,
  proof nets, ordinary logic.
- The schemas of Mix and weakening, the structural rules of the modes, when a lesson
  teaches them.
- A lesson's progress is not remembered across visits: the outline asked for none.

### Repositories and workspaces

- The client: `/home/tux/Projects/own/linlog-prover.github.io-43b`, the jj workspace
  `step43b` on `munpltsm`, its commits above; nothing pushed.
- linlog: `../linlog-step43b` on `nvlzxlln`, this report's commit; no code changed.
- The evidence: `/home/tux/Projects/own/linlog-step43b-screenshots/` (`outline.md`,
  `review.md`, `before/`, `after/`, `evidence/`, `scripts/`, `refs/` the Handbook read,
  `shots.mjs`, `run-shots.sh`).

## Session 43c: lessons 6 to 9

### The checklist

| requirement (the brief and the section "Goal") | status | evidence |
|---|---|---|
| The outline of lessons 6 to 9 sent to `planning` before the lessons were written, and the author's answer | done: confirmed, with two changes | sent 2026-10-24 11:28 (`/home/tux/Projects/own/linlog-step43c-screenshots/outline.md`); `planning` relayed at 11:31: confirmed as outlined, lesson 9 a regular lesson rather than optional, and lesson 7 to show "unknown" happening live at a small budget (`outline-answer.txt` beside it); a second message asked that every lesson problem be decided at the page's default budget |
| Lessons 6 to 9 in D56's sequence: a worked proof played with a reason at each inference and "predict the next rule" before each reveal, the faded proof, practice from the store, a self-explanation prompt at the densest step, a hinge question whose wrong answers each name a misconception | done | `src/content/lessons/6-two-sides.md` to `9-ordinary.md`; Vitest `src/content/lessons.test.ts`, Playwright `tests/lessons.spec.ts` |
| Lesson 7: the erroneous example, "find the step that went wrong", then the refutation; the three verdicts and "unknown" | done | the attempt (`7-no-proof.attempt.json`), "No proof, and why" (three problems asked of linlog), "When the search gives up" (live, at 20 units); Playwright "the attempt plays its steps, …", "a problem asked of linlog …", "a search with a small budget answers unknown, …" |
| Lesson 8: the net of a finished MLL proof, the switching condition, the essential net, nets with boxes and practice whose nets need boxes | done | "See its net" and "More nets" (an essential net, a box in a box, an intuitionistic box); `bang-bang`, `g164` and `box-door` in its practice; Playwright "a lesson's finished proof shows its net, and its gallery draws nets with boxes" |
| The reference's rule schemas the lessons use and the frame's hover explanations | done | the two-sided rules of intuitionistic linear logic, `src/content/rules/14-ill-ax.md` to `32-ill-contraction-l.md`, drawn in the reference's new section; an intuitionistic proof's popover shows them on hover; Playwright "the reference draws every rule …" and "in an intuitionistic proof, a rule of the popover shows its two-sided schema"; Vitest "draws each two-sided rule as the one-sided rule it names …" |
| Every lesson's sequent a store problem with linlog's verdict and session; the store's flake check passing | done | nine new problems and six moved into lessons (below); `evidence/new-problems.txt`, `evidence/*.interact.txt`; `check-problems`: "all 436 problems are as linlog's command decides them; 397 provable, each with a session that `proof` accepts" |
| Every lesson problem decided at the page's default budget | done | `evidence/budget-after.txt`: at most 16 387 of 2²⁶ units; Vitest "decides every problem of the lessons well inside the page's budget"; `tripling` replaced (below) |
| The education skills and `frontend-design` used | done | below, "Decisions" |
| One fresh-context reviewer, one round, each finding answered | done: two findings, both held and answered | below, "The review"; `/home/tux/Projects/own/linlog-step43c-screenshots/review.md` |
| Before-and-after screenshots of each lesson at a phone's and a desktop's width in both themes | done | `/home/tux/Projects/own/linlog-step43c-screenshots/before/` (20) and `after/` (92) |
| The client's `nix flake check` at its last commit; `jj st` clean in both workspaces | done | exit status 0 at `lltvnnlv` (19676123); below, "Verification" |

### Outcome

Learn's tutorial now has all nine lessons of D60, in its order, as the author confirmed
them from the outline (lesson 9 a regular lesson, as the author asked). Lessons 6 to 9
follow lessons 1 to 5: a worked proof of the store played on the stage one inference at a
time, the visitor predicting each rule before the page plays it and writes its reason
beside the inference; a self-explanation prompt with a model answer; the same proof with
its first steps played and the rest left to the visitor; the lesson's practice; and a
hinge question whose wrong answers each name their misconception.

- **Lesson 6, two sides**, teaches intuitionistic mode on step 40's two-sided surface: one
  formula on the right, the rules in pairs, `⊸L` and its split. The reference draws the
  nineteen two-sided rules of intuitionistic linear logic as schemas, each with the
  one-sided rule it is, and in an intuitionistic proof the popover shows a rule's
  two-sided schema on hover.
- **Lesson 7, when there is no proof**, opens with the erroneous example: the page plays an
  attempt at a provable sequent step by step, each step with its author's plausible
  reason, until it is stuck; the visitor finds the step that went wrong (each wrong answer
  names its misconception); linlog's verdict on the goal that step left, "Unprovable,
  checked: atom balance", stands beside it. The worked proof then plays the right step.
  Three sequents with no proof are asked of linlog on the stage, one per reason (atom
  balance, a classical countermodel, an exhausted search), and a sequent searched with a
  budget of 20 units answers "Unknown: gave up", which the retry's doublings turn into a
  proof at 160 units, as the author asked.
- **Lesson 8, proof nets**, plays weak distributivity with reasons that say what each step
  is in the net, then turns the stage to the finished proof's net; "More nets" draws an
  essential net, a box inside a box, and an intuitionistic net with a box.
- **Lesson 9, ordinary logic**, proves the images of ordinary sequents under linlog's
  translations, as `linlog seq print --logic …` prints them: the page has no ordinary mode.

Nine problems joined the store and one left it (436 problems, 397 provable), each decided
by linlog's command and checked by the flake's `problems` check. On the way, a step's reason
beside a rule whose name has a subscript (`⊕₁`, `&L₂`) turned out never to show, lessons 1
to 5's included; it is fixed at its cause.

### The lessons

| lesson | worked proof (store) | faded: played, then the visitor's | self-explanation (after step) | hinge | misconceptions named |
|---|---|---|---|---|---|
| 6 Two sides | `composition`, `A ⊸ B, B ⊸ C ⊢ A ⊸ C`: ⊸R, ⊸L, ax, ⊸L, ax, ax | ⊸R; then both ⊸L splits and the axioms | which one-sided rule `⊸L` is, and why its split works as `⊗`'s (2, the first ⊸L) | in `A ⊸ B, A, C ⊢ B ⊗ C`, which hypotheses go to ⊸L's left premise | ⊸L's left premise takes every other hypothesis; using `A ⊸ B` leaves `A` in place, as modus ponens does |
| 7 When there is no proof | `wrong-choice` (new), `(A & B) ⊗ C ⊢ C ⊗ (B ⊕ D)`: ⅋, ⊕₂, ⊗, ax, ⊕₁, ax, after the attempt ⅋, ⊕₁, ⊗, ax, ⊕₁ | ⅋; then the choice onward | count the atoms after the attempt's `⊕₁` and the proof's `⊕₂` (2, the ⊕₂) | why `A ⊗ B ⊢ A` has no proof | a sequent with no proof is classically false; "no proof found" means the search gave up; a wrong step of an attempt means no proof |
| 8 Proof nets | `weak-distributivity`, `(A ⅋ B) ⊗ C ⊢ A ⅋ (B ⊗ C)`: ⅋, ⅋, ⊗, ax, ⊗, ax, ax | ⅋, ⅋; then both splits and the axioms | where the split's choice shows in the net, and why every switching leaves a tree (3, the first ⊗) | which sequent has a proof net | linking every atom to its dual makes a net; a net may fall apart into pieces; a net read the other way up is a net |
| 9 Ordinary logic | `girard-modus`, `!(!A ⊸ B), !A ⊢ B`, modus ponens translated: !L, ⊸L, !R, ax, !L, ax | !L; then the rest | why the translation needs both `!` (2, the ⊸L) | why linlog proves Peirce's law classically and refutes it intuitionistically | a formula is valid whatever the logic; translating into linear logic loses valid formulas; the search gave up |

The attempt's wrong answers name their own misconceptions: the first step is to blame; a
split is to blame whenever an attempt gets stuck after one; a branch that closes can be
where it went wrong; the wrong step is the one where the attempt got stuck.

Each lesson's practice is its store problems, easiest first, provable and unprovable mixed:
lesson 6 has 14, lesson 7 has 10, lesson 8 has 15, lesson 9 has 7.

### Decisions

- **The lessons keep 43b's form**: one Markdown file each, the front matter naming the
  worked proof, a rule and reason per step of its session, the rules to predict among,
  the faded steps, the prompt and the hinge; `lessons.test.ts` holds the steps to the
  session's history, as for lessons 1 to 5.
- **Three optional parts carry what lessons 7 and 8 need**, each in the front matter, so
  that the content stays data:
  - `attempt`, the erroneous example: steps with the attempt's own reasons and, for every
    step but the wrong one, the misconception that would blame it. Its session is
    `lessons/7-no-proof.attempt.json`, an unfinished session that `linlog interact` wrote
    (`evidence/attempt.interact.txt`). The page plays it by Redo on the lesson's tab, as it
    plays the worked proof; the question is answered as the hinge is; the right answer
    offers "Ask linlog about the goal step 2 left", which takes the attempt back to just
    after that step and runs the search on the goal it left. `lessons.test.ts` holds that the
    wrong step is the first after which an open goal has no proof, by the module's search
    within the page's budget, and that the attempt ends stuck.
  - `gallery`, problems of the lesson shown on the stage: asked of linlog (`ask`: the search
    on the sequent itself, as Close runs it, so the page checks the refutation) or replayed
    with their net drawn (`net`). The test holds that each is a problem of the lesson, the
    asked ones unprovable, the drawn ones with a net.
  - `unknown`, a sequent outside the store searched with a budget too small. The page opens
    it and closes its goal with that budget; the note's "Try N units" doubles it. The test
    holds the text to the module: unknown at 20, 40 and 80 units, proved at 160 and at the
    page's budget. The sequent, `(A ⊕ B) ⊗ (A ⊕ B) ⊢ (A ⊗ A) ⊕ (A ⊗ B) ⊕ (B ⊗ A) ⊕ (B ⊗ B)`,
    was chosen for a search long enough to stop and short enough to decide in three
    doublings, and a proof a student can see: one of the four cases fits each choice.
    `linlog prove --deterministic --work-limit 20`, `40`, `80`: unknown; `160`: provable
    (`evidence/new-problems.txt`).
  - `net`: the finished worked proof offers "See its net".
- **The two-sided schemas are rules of their own** (`rules/14-ill-*.md`), not a second
  drawing of the one-sided ones: the Handbook's Table 1.5 (§1.7.2.1, p. 48, read in its
  text) in linlog's names (`Rule::name` in `core/src/proofs/rule.rs`: `!w` and `!c` for the
  Handbook's `(wL)` and `(cL)`), without the cut, the quantifiers and the exchange, each with
  `oneSided`, the one-sided rule it is. The test holds each to a one-sided rule that lists
  its name among its two-sided names, so the two tables cannot drift apart. `ruleNamed`
  prefers a two-sided schema in an intuitionistic proof, whose drawing is two-sided; a
  classical proof drawn two-sided keeps the one-sided schema, since the intuitionistic
  rules allow one formula on the right and the classical ones do not.
- **The new problems**:
  - lesson 6: `used-up`, the hinge's sequent (from the review), and the Handbook's formula `((((0 ⊸ 1) ⊸ 1) ⊸ 0) ⊸ 0) ⊸ 1` (§1.7.2.1, p. 49),
    unprovable in intuitionistic mode and provable in classical mode, the Handbook's
    example of what one formula on the right forbids (`handbook-intuitionistic`,
    `handbook-classical`);
  - lesson 7: `wrong-choice`, the attempt's and the worked proof's sequent, and
    `odd-machine` (below);
  - lesson 8: `box-door`, `!(A ⊸ B), A ⊢ !B`, a box whose door would not be under `?`;
  - lesson 9: Peirce's law under both translations (`peirce-intuitionistic`, unprovable;
    `peirce-classical`, provable in affine mode, unprovable without weakening) and double
    negation under the call-by-name one (`double-negation-translated`, unprovable).
  - The provable ones were measured by 43a's measure (`handbook-classical` 15.0, Hard;
    `wrong-choice` 10.7, Moderate; `peirce-classical` 13.2, Moderate); the unprovable ones
    were levelled by hand. Their sessions come from linlog's own hints, `proof` exit 0.
  - Into lessons: `girard-modus`, `girard-k`, `girard-w` and `girard-s` (lesson 9),
    `bang-bang` and `g164` (lesson 8), each now with an explanation; `g203` and the girard
    problems had none.
- **The education skills**:
  - `erroneous-example-designer`: one realistic error (the wrong half of `&`, chosen
    early), which shows only later; the visitor finds it, says why, and fixes it in the
    faded proof; it follows correct examples.
  - `hinge-question-designer`: each wrong answer is the right answer under one
    misconception, the options parallel.
  - `worked-example-fading-designer` and `digital-worked-example-sequence`: one step at a
    time, the reason at the inference, the faded proof starting before the lesson's own
    choice, as in 43b.
  - `self-explanation-prompt-designer`: why-questions at the densest step, answerable from
    the lesson, free text then a model answer.
  - `learning-progression-builder`: the prerequisites support D60's order. 6 needs 2 (the
    split) and 5 (`!`); 7 needs the atom counts and every connective's classical reading,
    and as an erroneous example it follows correct examples; 8's essential nets need 6 and
    its boxes 5; 9 needs 5, 6 and 7.
  - `cognitive-load-analyser`: one prompt at a time, the reasons beside the inference, the
    new parts in the lesson's own column of parts.
  - `frontend-design`: the new parts use 43b's grammar alone: the parts' rule in Learn's
    colour, the hinge's options for the attempt's question, the practice cards for the
    gallery, no new token; the touch-target walk measures them.

### Judgement calls

- **`tripling` left the store, replaced by `odd-machine`**, at `planning`'s second message
  (every lesson problem decided at the page's default budget; one that could come back
  unknown replaced, not marked). The page's budget counts units of work, not time, so a slow
  machine gives the same verdict; but the page decides `tripling` only after 16 778 456 of
  its 2²⁶ (67 108 864) units, by the inverse method's saturation, and the command without
  `--deterministic` answers unknown within 20 seconds. Every other lesson problem needs at
  most 298 units. `odd-machine`, `!(a ⊸ a ⊗ a ⊗ a), a ⊢ a ⊗ a`, makes `tripling`'s point (each
  use adds two `a`, so the count stays odd) and is decided in 16 387 units by the Horn
  engine, with and without `--deterministic` (`evidence/budget.txt`, `budget-after.txt`). An
  unprovable problem must belong to a lesson, so `tripling` was removed rather than moved.
- **Lesson 8 takes `bang-bang` (`!A ⊢ !!A`) where the outline named `g203`**: `g203`'s `!` is
  on the left, `?` one-sided, so its net has a contraction but no box. `bang-bang`'s two
  promotions draw a box inside a box (the Playwright test counts two).
- **A fifth unprovable problem in lesson 8, `box-door`**: with three problems whose nets need
  boxes, the lesson's practice would have had three provable problems in a row, which the
  store's test refuses; `!(A ⊗ B) ⊢ !A ⊗ !B`, the first candidate, is unknown to the command,
  so `!(A ⊸ B), A ⊢ !B`, decided unprovable, took its place. The test was not loosened.
- **Two fixes outside the lessons**, each at its cause:
  - A step's reason beside a rule whose name has a subscript (`⊕₁`, `⊕₂`, `&L₁`) never
    showed: linlog draws such a label as a group, and the stage looked for a text. Lesson 3's
    `⊕` steps had the same fault since 43b. The attempt's Playwright test now pins it.
  - `close` rounds its budget to whole units: a retry or the lesson's small budget is a
    fraction or a multiple of the one Settings holds, which a visitor may set to any number.
- **"Ask linlog" runs the search on the problem's sequent as Close does**, not Close all, so
  the page checks the refutation and says so ("Unprovable, checked: …").
- **The attempt's options show each step's reason** under its rule (`cognitive-load-analyser`:
  without it, the visitor must find each step on the stage to judge it, a split of
  attention).
- **The live unknown's sentence names the outcome**: "at 160 units linlog proves it". The
  command proves it from 120 units; the page's retry doubles 20, 40, 80, 160, so 160 is what
  the visitor sees, and the test pins those four answers.

### Departures from the design

- None from D21, D22, D47, D56 or D60. Lesson 7 has, besides D56's erroneous example and
  refutation, a worked proof and a faded proof as the other lessons do: the goal asks the
  worked-example sequence of every lesson, and the attempt's sequent is provable, so its
  right proof is the worked one.

### The review

One round, a fresh-context reviewer on Opus 5.5 at `high`, briefed with D21, D22, D47,
D56, D60, the confirmed outline and the author's changes, the known departures, and the
machine's rules (linlog's command alone, `--deterministic`, capped, on cores 10 and 11),
told to report only what breaks a stated requirement
(`/home/tux/Projects/own/linlog-step43c-screenshots/review.md`). It confirmed:
- the verdicts of all 45 problems of lessons 6 to 9, each explanation against linlog's
  reason, and the most work any needs (16 387 units);
- the live unknown (unknown at 20, 40 and 80 units, provable at 120 and 160, outside the
  store);
- every reason, model answer and hinge line of the four lessons;
- the translations against `seq print`;
- the Handbook problem against p. 49, and the nineteen schemas against Table 1.5 and
  linlog's names;
- every part D56 and D60 ask for, and lesson 9 marked optional nowhere;
- the three known departures as sound, and that the tests hold what they claim.

Its findings, each answered:
1. **Lesson 7 stated the atom-pairing reason as a law of every proof**, in its opening,
   its model answer and its hinge's right answer; `⊤` and `?w` drop formulas (`A ⊢ ⊤` and
   `!A ⊢ 1` are provable). **Holds.** All three now say where it holds, a goal with no `⊤`
   and no `?`, which the lesson's sequents are (client `lltvnnlv`).
2. **Lesson 6's hinge sequent, `A ⊸ B, A, C ⊢ B ⊗ C`, was not a store problem** (borderline;
   43b's hinges in lessons 2 and 3 set the precedent). **Holds** against the goal's wording.
   It joined the store as `used-up` in lesson 6, its session from linlog's hints, `proof`
   exit 0 (`evidence/used-up.interact.txt`), level 2 by 43a's measure (7.3).

### Verification

- **The client's `nix flake check`** passed, exit status 0, at its last commit `lltvnnlv`
  (19676123), under the machine lock on cores 10 and 11
  (`/home/tux/Projects/own/linlog-step43c-screenshots/evidence/flake-check.log`):
  - `check-problems`: "all 436 problems are as linlog's command decides them; 397
    provable, each with a session that `proof` accepts";
  - Vitest 527 passed, Playwright 109 passed.
  - It passed before the review too, at `vzxqvlos` (`evidence/flake-check-1.log`).
- **Vitest** (527): `lessons.test.ts` now holds the nine lessons in D60's order and each
  worked proof to its session, with a two-sided schema for every rule of an intuitionistic
  proof. Its new tests:
  - the attempt's one wrong step is the first after which an open goal has no proof, by
    the module's search;
  - the live unknown answers unknown at 20, 40 and 80 units and proves its sequent at 160
    and at the page's budget, its text naming 160;
  - a gallery's problems are the lesson's, the asked ones unprovable, the drawn ones with
    a net, and lesson 8's worked proof has a net;
  - every lesson problem is decided within a hundredth of the page's budget, as the store
    says;
  - each two-sided rule is the one-sided rule it names.
- **Playwright** (109). New tests in `tests/lessons.spec.ts`:
  - the two-sided schema on hover;
  - the attempt, its question and linlog's answer on the goal it left;
  - a problem asked of linlog;
  - the search giving up at 20 units and proving at 160 after three retries;
  - the net of the worked proof and a box inside a box.

  The touch-target walk now measures lesson 7's attempt, its question and the live
  unknown, at every width. Three tests changed on purpose, because lessons 6 to 9 are now
  written:
  - the reference test counts two lists of schemas;
  - `learn.spec.ts` opens lesson 6 by its button;
  - `engines.spec.ts` counts nine written lessons.
- **The sequents**:
  - every new problem by `linlog prove --deterministic` and `seq fragment`, the attempt's
    goals, lesson 6's hinge premises, the live unknown by `--work-limit`, and the
    translations by `seq print` (`evidence/new-problems.txt`);
  - the sessions by `linlog interact` from linlog's hints, `proof` exit 0
    (`evidence/*.interact.txt`; the attempt's exit 1, as an unfinished session's);
  - every lesson problem at the page's budget by the page's module (`evidence/budget.txt`
    before the replacement, `budget-after.txt` after; `scripts/budget.mjs`).
- **Signing**: a signature was tested before each commit; every commit is signed.
- **The screenshots**: `before/` (20: the four lessons and the reference, each at a phone's
  and a desktop's width in both themes; before, lessons 6 to 8 were their practice alone
  and lesson 9 "to come", so its pictures show the tutorial's list) and `after/` (92: the
  same 20 names, and for each lesson its worked proof in progress, its faded proof and its
  hinge answered; lesson 7's attempt with its question, a problem asked of linlog and the
  search giving up; lesson 8's net and a box in a box; a two-sided schema on hover). They
  were taken by `shots.mjs` from the site built at `rsxtpzrz` and at `lltvnnlv`.
- **The client's commits**, above `rsxtpzrz` (the client's `main` when the session began,
  after a rebase from `qmlqvzzn` at `planning`'s word):

  | change | what |
  |---|---|
  | `mtmtnkun` | the problems lessons 6 to 9 need, filed in the store |
  | `qypomyyv` | a lesson's reason placed beside a rule label drawn with a subscript |
  | `tvypowqr` | a search's scaled budget rounded to whole units |
  | `vnlrstuz` | `tripling` replaced by `odd-machine`; `bang-bang` into lesson 8 for `g203` |
  | `vzxqvlos` | lessons 6 to 9, the two-sided rules' schemas |
  | `lltvnnlv` | the review's answers |

### What is left open

- The command without `--deterministic` answers `tripling` unknown within 20 seconds,
  where `--deterministic` (and the page) decide it by the inverse method; the race does not
  give the inverse method its turn there. It is linlog's dispatch, not the lessons', and
  `tripling` is no longer in the store.
- The page shows ordinary logic only through its images; lesson 9 can say "type it here"
  once the page has an ordinary mode (D57).
- The before-and-after pictures are for the author to judge.
- A lesson's progress is not remembered across visits, as for lessons 1 to 5.

### Repositories and workspaces

- The client: `/home/tux/Projects/own/linlog-prover.github.io-43c`, the jj workspace
  `step43c` on `rsxtpzrz`, its commits above; nothing pushed.
- linlog: `../linlog-step43c` on `zuqxrmzx`, this report's commit; no code changed.
- The evidence: `/home/tux/Projects/own/linlog-step43c-screenshots/` (`outline.md`,
  `outline-answer.txt`, `review.md`, `before/`, `after/`, `evidence/`, `scripts/`,
  `shots.mjs`, `run-shots.sh`, `linlog-cli` the command at the client's pinned input).
