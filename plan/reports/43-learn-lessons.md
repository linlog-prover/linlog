# Step 43: Practice and Learn, from one store of problems

## Session 43a: the problem store and Practice

### The checklist

| requirement (the section "The problem store and Practice") | status | evidence |
|---|---|---|
| One store in the client's content, the single source of Practice and Learn: each problem's sequent, calculus, linlog's verdict, difficulty, categories, a session `proof` accepts when provable | done | `src/content/problems/` (425 problems: 392 provable, 33 unprovable); `src/content.config.ts` (the schema); Learn's problems are the store's entries with a `lesson`, Practice's the provable ones (`/problems.json`) |
| At least 300 provable, every level and every fragment represented, the hardest a real effort; step 38's and 38b's practice joins | done, the hardest for the author to judge | 392 provable, by level 76, 109, 123, 84; all twelve fragments and all six calculi (`store-size.py`); all 73 of step 38's and 38b's sequents are in the store, with their lessons; the validation's agents took 24 to 46 commands on the Very hard ones |
| Generated with intent, and chosen from the literature, each source read; nothing copied from LLTP or ILTP | done | `gen/gen.py` (built backwards from a planned proof shape, then disguised), `gen/pick.py` (kept only at the level aimed at); 45 problems are stated in a work read in itself and 4 derived from one, which their source line says (`evidence/classics.md`); no library file was read |
| A difficulty that means something for a human, from the literature and the problems' structure, validated by agents of different strengths, the levels ordering their success and effort | partly: effort is ordered by the levels, success is not (every attempt succeeded; only error-free success drops, at Very hard); for the author to judge | the metric below; `research/difficulty.md`; the validation's table below (`trial/`) |
| Checked by the flake: linlog's command over the whole store | done | the client's `problems` check (`modules/_checks/problems.py`), failing on a planted wrong entry and passing on the store (Verification) |
| Practice a place of its own, outside Learn's frame, from the bottom band at every width; only provable problems; filters by category and difficulty; a random one within the filters; a problem on a new tab; the hint ladder its help; the proved ones remembered | done | `src/islands/Practice.tsx`, `src/lib/practice.ts`; Vitest `src/lib/practice.test.ts`; Playwright `tests/practice.spec.ts` and `tests/phone.spec.ts` (320 to 1920 pixels) |
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
  - **What it does not show.** Success does not separate the levels: every attempt
    succeeded within the bounds, so in this sample the levels are a scale of effort, not
    of failure. Only success without a refused step or an undo drops, at Very hard
    alone. The section asks that the levels order success and effort; this validation
    shows effort only. More agents would not change that within these bounds, and a
    tighter bound would order success by construction, so the author judges whether
    effort suffices, or whether people should try the sample.
  - **Where the choice term shows.** The proof's steps alone rank the commands as well
    (0.97 and 0.99), so the choice term adds no rank information here. It shows on
    `g291`, a problem whose proof is short (15 steps) but which needs three copies of a
    `?`-formula. Haiku spent 39 commands and 6 undos on it, after a split that left a goal
    without a copy.
  - **For the author.** The Very hard problems took the agents 24 to 46 commands; whether
    that is "a real effort for someone who knows the rules" is the author's to judge. A
    stricter bound, or people instead of agents, would test the levels' separation by
    success.

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
   done.** **Holds.** The checklist now says "partly". The report adds the error-free
   success, which drops at Very hard alone, and leaves the judgement to the author.
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
- The validation's sample had no failures. A tighter bound, or students, would test
  whether the levels also order success.
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
  - `trial/`: the validation's rules, harness, sample, logs and table.
  - `before/` and `after/`: the screenshots, listed by `ls`.
  - `store-size.py`: the store's size by level and category.
