# Step 28: the audit, and the code in order for the release

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing. The step takes several
sessions: the requirements, baselines and gates; the audit; the design;
one session per area of fixes, each followed by a check round. Every
session starts from the reports before it and is reviewed before the
next. Read before you start:

- `plan/reports/17-assessment.md`: section 1 in full (1.1's stale
  claims, 1.2's refactoring list, 1.3's invariants, 1.4's tests), "The
  command, the harness and the flake as a maintainer finds them",
  section 4, 3.9, and the author's answers 4 and 5.
- `plan/README.md`: every decision, D15 to D18 and D23 above all; the
  reports of steps 18 to 27.
- `plan/later.md`: "Code audit and refactoring", "First-order linear
  logic"; `proof-search-specifications.md`, "First-order fragments".
- Every rules file under `.claude/rules/`.

## What the earlier steps left you

Twenty-seven steps by separate sessions built the library, the command
and the harness, each reviewed for its own correctness and none for the
whole; step 17 read it once as a maintainer would. The author decided on
2026-10-03 to put it in order once, here, right before the release,
rather than before the steps that add to it, so that the audit is not
paid twice (D23). Until the release the API may change freely (D18);
quantifiers will come, and the data model and the interfaces must have
their place without a second rewrite and without slowing the
propositional case (D17).

What the steps left for this one, besides section 1 of step 17's
report:

- **From steps 18 to 20**: the error family is scattered (`ShapeError`,
  `Unsupported`, `UnknownRule`, `NetError`, `ViewError`,
  `Error::Rejected`, `Error::Unchecked`, `Error::TooManyNodes`, and in
  the checker `Problem::Surplus`, a fault of the proof, beside
  `Problem::Memory`, a refusal and no verdict: a caller must never be
  able to read a refusal as "invalid"). A proof file and a session's
  state are read under the default occurrence limit whatever the caller
  asked, because `Deserialize` takes no options; the checker's pass
  takes a memory bound and no stop condition; the bounds of a search, a
  check and a view are three values a front end sets one by one.
- **From step 21**: `Verdict::Unprovable(Refutation)`, whose `Unbalanced`
  holds an atom and its name both, which the forest has;
  `Options::copies(Option<u32>)`; `Statistics::copies`, the larger of
  two searches' levels (30 under a bound of 3 on a Horn program);
  `Interactive::close_with`, which grafts any `Proof` that passes the
  checker against the goal's ids without asking whether it is a proof
  over the session's forest. In the command: the race of one thread
  and the pool is written twice (`cli/src/prove.rs`, `bench/src/run.rs`);
  its two searches each hold the whole `--memory-limit`, so a call
  without flags may hold twice it (2.05 GB measured on `SYJ206+1.016`
  in its `01` translation; half the bound each starved a wide memo, and
  one account both draw from is the question); `--jobs 2` with
  `--pool-after` runs three threads; the harness's `--pool-after` and
  `--timeout` panic on a negative or NaN value.
- **From step 22**: the left-out line of a derivation names the whole
  derivation's size even where the compact view was tried (its lower
  bound, `size::Firm`, needs a field in `ViewError`);
  `Inference::times` is a public field every constructor sets;
  `Compact` sits at the crate root beside `ViewOptions`; the Typst
  layout's spacing fields are Typst code written verbatim and unchecked;
  `RenderError` stands outside `Error` as the other export errors do;
  `rocq::Options::lemma` is not checked to be an identifier; the
  exports' `String` functions and their `write` twins could be one
  generic call.
- **From step 23**: the stale claims outside the rules files and
  CLAUDE.md (README's "returning a checked proof", the `--help` texts
  section 1.1 lists), and the lint allowances of `core/src/lib.rs`,
  whose reason is items live under one feature only, test-only items and
  a few dead ones.
- **From step 24**: `batch::Options` gets its wire form with the search
  options'; `--jobs 2` races one thread against a pool of two (three
  threads), the race written twice and its memory doubled (above).
- **From step 25**: `Image::read_back` and `Derivation::check` take no
  stop (4.5 s past the time limit on a read-back of 11.6 million
  inferences, under `--derivation-limit none` only); `ordinary::Sequent`
  and `ordinary::Derivation` have no JSON form, which the web front end
  needs; the classical certificate's import line is a constant of
  `ordinary::rocq`, an option of `rocq::Options` by D15.
- **From step 26**: the engine interface (`search::Decide`, `Answer`,
  one place that builds a `Verdict`) and the dispatch as data
  (`search::DISPATCH`) are the shape the API keeps; its report's item 7
  says where a trail of bindings would go in the focused engine and
  which prunes assume ground atoms, the input of `plan/notes/api.md`'s
  first-order plan. Its other follow-ups (a sparse memo key, a pool's
  choice whose error cancels its siblings) are in `plan/later.md`.
- **From step 27**: the Horn engine reuses `Statistics` (`nodes` the
  markings reached, `memo_hits` and `memo_entries` the markings reached
  again and kept), so the meaning of a counter depends on the engine;
  `Refutation::StateEquation` names weights, and `Refutation` now lists
  every JSON form; `search::engine_for` and `Engine::parallel` are public
  queries for a front end that schedules threads; `linlog::mist` reads
  Mist's `.spec` format, beside `lltp`; `Error::NotHorn` beside the other
  refusals of a forced engine.

## Goal

At the release, the library is one a stranger can use from its
documentation, whose types say what they hold, whose errors are one
family and whose options are data with a wire form; the command, the
harness, the flake and every document say and do the same thing once;
the code meets the author's rubric, agreed by reviewers that looked for
its faults independently; and the API, the data model and the wire
forms are designed for the steps that come after it (the web client,
the Rocq library, MELL nets with boxes, cut, new engines and calculi,
quantifiers), so that none of them needs a second rewrite.

## How the step runs (the author, 2026-10-05)

The author asked for this step to be thorough and is ready to spend
more of the quota on it. Its design follows the advice of the day
(`plan/README.md`, "Why these models and efforts", 2026-10-05): fan out
agents where the work is reading and judging, keep one implementer per
area where the work is tightly coupled, give every session a check it
can run, have a fresh context refute every result, put independence in
methods and in models of more than one kind, and measure before any
change made for speed. Each stage below is a session of its own (`--name
step-28`, `step-28b`, …), reviewed by the planning session before the
next. A workflow may be larger than the default size guideline: this
prompt asks for the scale it names, and a run logs what a bound drops.

## Stage 0: requirements, baselines and gates (one session, Opus 5.5 at `high`)

What the audit and the fixes are judged against, and what proves that a
fix changed nothing it should not.

1. **The register of later requirements**,
   `plan/notes/requirements.md`: what each later step needs of the
   library, read from prompts 29 to 38, `plan/later.md` and D15 to D22,
   one agent per prompt (a workflow; Sonnet 5.5 at `high`), merged and
   deduplicated. For example the web client's wasm build, no clock,
   bounded memory and a stop on every long call, and a wire form for
   every options value, outcome, proof, derivation, session and error;
   the Rocq library's proof term and checker; the place of a rule added
   by cut; the proof structures that MELL boxes extend; an engine and a
   dispatch row per new calculus; terms, binders and a trail for
   quantifiers. Each requirement names the step, the item of the API it
   concerns and how a later session would know it is met.
2. **A behaviour lock** (characterization tests, before anything is
   refactored): the command's output in every format and its exit status
   on a fixed corpus (sequents of every fragment and mode, ordinary
   logic, `.p` and `.spec` files, errors), and every JSON form
   (`Sequent`, `Proof`, `Outcome`, `ProofStructure`, `Interactive`,
   `Refutation`), pinned as committed fixtures with a test that compares
   them. A later change to one of them is a commit of its own that says
   why. The target set, the families, ILTP, qcover and the LLTP batch's
   verdicts are the search's lock, as before.
3. **Performance as a measurement first**, as Anthropic's performance
   sprint of 2026-09 did it: a fixed set of small, deterministic
   journeys (rows of the target set, families, readers, the checker, a
   derivation, a render, a batch), their instruction counts under
   callgrind, and a check, before any number is trusted, that lowering
   each count lowers its wall-clock time on a pinned core. Then a
   ratchet: the counts as committed ceilings, a flake check that fails
   when one is passed by more than its tolerance, and a command that
   lowers a ceiling whenever a count goes down, never up without a
   commit that says why.
4. **Evidence that no reviewer supplies**, each adopted through the
   `new-tool` skill: mutation testing (`cargo-mutants` or its current
   equivalent) of the checker, the readers, the search's front door,
   the Horn engine's refutations and the ordinary layer's checker, the
   surviving mutants listed; and a fuzz target for every reader of
   untrusted input (the text and ordinary syntaxes, every JSON form,
   LLTP, TPTP, `.spec`, a proof to check), run for a bounded time each
   on named cores, every crash or bound passed recorded. The web client
   will feed these readers text from a browser.
5. **The gate**: one command (a devshell entry) that runs what every
   commit of this step must pass (clippy, the tests, both `cargo hack`
   runs, the behaviour lock, the ratchet), and that every later session
   runs before each commit; a session may also set it as its `/goal`
   condition so that it does not stop before the gate passes.

`plan/reports/28-baselines.md` holds the register's summary, the
journeys and their validation, the mutation and fuzz results, and the
gate. The session ends there.

## Stage 1: the audit (one session, Opus 5.5 at `high`; a workflow)

1. **The rubric**, `plan/notes/audit-rubric.md`: every criterion, each
   marked must-fix, should-fix or taste. The author's, of 2026-10-03:
   very efficient and performant; an idiomatic project structure and
   code following current best practice; concise doc comments a human
   understands at first read; self-documenting data structures and
   functions; comments only where the code cannot say it; altogether,
   code that is pleasant to review. The standing ones, from CLAUDE.md,
   `plan/conduct.md` and the decisions: bounded time and memory; every
   default a named option (D15, D16); a number soundness rests on argued
   at its declaration or refused at its limit; tests as necessary;
   dependencies that earn their place; no recursion over a formula; no
   comment naming the plan; licence headers; README true. And two
   external standards, each criterion cited by its identifier: the Rust
   API Guidelines checklist (naming, interoperability, documentation,
   predictability, flexibility, type safety, dependability,
   debuggability, future proofing) for the library; and, for the JSON
   wire forms only, the parts of Google's API Improvement Proposals that
   concern a data format rather than a network service (AIP-180 for
   backwards compatibility, AIP-126 for enumerations, AIP-193 for
   errors, AIP-140 for field names, and AIP-151 as a pattern for a long
   call the client can watch and cancel). AIPs are written for
   resource-oriented network APIs: their resources, methods and
   pagination do not apply here.
2. **What a machine can check is a check, not a reviewer's token**: the
   clippy lint groups chosen (and the ones not chosen, with the reason),
   rustdoc's lints, unused dependencies, a spell check, the greps for
   comments that name the plan and for licence headers, all in `nix
   flake check`, so that they never regress.
3. **The audit as a workflow** (the author's choice, 2026-10-03 and
   2026-10-05). Each reviewer reads the whole repository through one
   lens against the rubric, the register of later requirements and stage
   0's evidence:

   | lens | model, effort |
   |---|---|
   | soundness of the checker, proof terms, nets and their criterion, and the integers they rest on | Fable 5.1, `high` |
   | soundness of every refutation (the focused engine's prunes, the counts, the Horn engine's equation, coverability and searches, the ordinary layer) | Fable 5.1, `high` |
   | readiness for the later steps, against the register | Fable 5.1, `high` |
   | the API, the data model and the wire forms, against the Rust API Guidelines and the AIPs named above | Opus 5.5, `xhigh` |
   | performance, from stage 0's measurements (profiles read as text, never drawn) | Opus 5.5, `high` |
   | untrusted input and resource bounds, from the fuzz results | Opus 5.5, `high` |
   | idiom, structure, reuse and simplification | Opus 5.5, `high` |
   | doc comments, names, comments | Opus 5.5, `medium` |
   | tests: missing for a stated behaviour or a surviving mutant, or in excess | Sonnet 5.5, `high` |
   | coherence of README, CLAUDE.md, the rules, the help texts with the code | Sonnet 5.5, `high` |

   Round one is independent: a finding has a file and a line, the
   criterion (a rubric entry, a guideline's identifier or a
   requirement), a severity, its evidence (a quote, a failing test, a
   surviving mutant, a measurement) and a proposed fix, as a JSON
   schema. A reviewer reports what breaks a must-fix or should-fix
   criterion or a requirement; a reviewer asked for faults finds some
   where the work is sound, so taste is listed apart and not counted.
   Round two is a cross-examination: each finding goes to two reviewers
   other than its author and of another model where one is in the
   panel, who try to refute it with evidence; it falls when both do.
   Then a completeness critic (Fable 5.1 at `high`) asks what was not
   read, which lens had no finding where one was due, and which claim
   rests on no evidence; its gaps are a further round, and rounds stop
   when one adds no confirmed finding (two at most). Duplicates are
   merged by code, near-duplicates by one agent. Matters of taste go to
   the author as a short decision list. In a workflow name Sonnet 5.5 as
   `sonnet`, which `.claude/settings.json` maps to `claude-sonnet-5-5`;
   no agent runs at `max`. An agent that runs programs gets the rules of
   `plan/conduct.md` in its prompt (named cores, a memory-capped scope,
   bounded runs). A run may use up to about sixty agents.
4. **`plan/reports/28-audit.md`**: the findings by area and severity,
   each with its evidence, the decision list, and what the run cost
   (agents, and tokens by model). The session ends there, and the author
   answers the decision list; the answers become rules under
   `.claude/rules/`, which later rounds judge against.

## Stage 2: the design (one session, Opus 5.5 at `xhigh`)

`plan/notes/api.md`: the public surface after this step, its data model
and its wire forms, and how each later step enters it, first-order
logic above all (atoms as predicates over terms, binders in the arena,
a substitution beside the forest, a trail of bindings in the engines,
witnesses in proofs), with what stays untouched for the propositional
case.

1. **Three drafts from different angles**, each in a fresh context with
   the register, the audit and the rubric (Opus 5.5 at `xhigh`): the web
   client and the wire forms first; the proof term, the checker and the
   Rocq library first; new engines, calculi and quantifiers first.
2. **Judged and synthesised**: two judges score each draft against the
   register and the rubric, independently (Fable 5.1 at `high`, Opus 5.5
   at `high`), and the session synthesises from the best, grafting what
   the others do better.
3. **Walked through**: for each later step, an agent (Sonnet 5.5 at
   `high`) sketches its first change against the design and reports
   where it would have to work around it; the design answers each
   report or the register records why not.
4. **A spike for quantifiers**, in a jj workspace of its own and thrown
   away: the data-model change the design proposes for terms, built far
   enough to measure what it costs the propositional case on the target
   set and stage 0's journeys (D17: the propositional case must not
   pay).

The note ends with the decisions it needs from the author, and the
session with them; the author signs the design off before stage 3.

## Stage 3: the fixes (one implementer session per area, in this order)

Each session takes the rubric, the register, the decision list, the
design and its area's findings. Fixes are sequential in the one working
copy, by one implementer per area, since the parts of one refactoring
share too much context to split among agents; agents read, review and
measure beside it. Every commit passes the gate; the behaviour lock
holds unless a finding requires a change, which is a commit of its own;
before an area is closed, a reviewer in a fresh context (Fable 5.1 at
`high` for soundness, Opus 5.5 at `high` otherwise) reads its diff
against its findings and the design and reports only what breaks a
criterion or a requirement.

1. **The library's API, data model and wire forms** (Opus 5.5 at
   `xhigh`; a change of the checker is reviewed by the panel, as a
   change of the search is), as the design says. Then: what is `pub` and
   need not be; names that differ between neighbours (the `Engine`
   structs beside the enum, the types called `Rule` or `Rules`);
   `#[non_exhaustive]` and builders applied evenly; `Mode` where a
   `bool` stands for it; one rule for taking or cloning a forest; one
   numbering of inferences or a map between the two; options with a
   wire form for every options value (the command, the batch mode and
   the web client are then callers of the same values, D15), each JSON
   form with a version and documented schema; one family of errors with
   a serializable form; one printer of formulas and of two-sided
   sequents over `Notation`; `Rule` as a classical rule and a position,
   in a file of its own; `proofs/interactive.rs` along its seams; the
   lint allowances of `core/src/lib.rs` removed; the invariants of
   section 1.3 as types, assertions or tests where one sentence of code
   does it.
2. **The search** (Opus 5.5 at `xhigh`): the audit's findings under
   `core/src/search/`. A commit that claims no change of the search
   keeps the target set's counters; one that changes it is reviewed by
   the panel step 26's prompt describes, its argument read on Fable 5.1
   at `high` again.
3. **Efficiency** (Opus 5.5 at `xhigh`), once the structure has settled,
   as the performance sprint ran: one journey at a time, its hot spot
   from a profile read as text, a test that pins the behaviour before
   the change, the change, the instruction count and the pinned
   wall-clock time after, and the ratchet's ceiling lowered. The targets
   are not where it stops; a gain that costs more code than it is worth
   (the sprint's example: nine hundred lines for two milliseconds) goes
   to the author as a decision, not into the tree.
4. **The command, the harness, the flake and the documents** (Opus 5.5
   at `high`): `prove` and `interact` share their search flags, the
   building of `Options`, the stop closure and the verdict line; a new
   output format is one place; the help texts that are wrong are right;
   `interact`'s words are read with quoting, its ASCII rule names are in
   its help, and `--file -` is refused; a closed pipe keeps the verdict;
   `--output` into a missing directory fails before the search; exit
   statuses beyond 0 to 3 are documented or removed; a session's sequent
   is read under the limit as `prove`'s is. Step 21's leftovers above.
   Tests for the stated behaviours that have none (`--timeout`, Ctrl-C,
   `--jobs`, `--bias`, `--memo-limit`, `--output`). The harness: tests
   for `summary` and the comparison, the `bench` check failing on an
   `error` or `crash` row, one configuration label, a table of the
   counters in `summary`, a late verdict kept apart, the columns read by
   name, `baseline.sh`'s hard-coded second baseline as parameters,
   `reruns.txt` and the repeated experiments retired or regenerated. The
   flake and CI: `cargo doc --document-private-items` builds, the
   fragments the command prints by default compile in the `export`
   check, the declared systems are those CI builds, the constants that
   name pinned versions are checked against the pins. Every stale claim
   of section 1.1 that step 23 did not correct, and the spec's two wrong
   statements moved to its errata.

## Stage 4: the check rounds

After each fix session, a workflow of the same lenses one effort level
lower: each finding of that area goes back to the lens that raised it,
which answers fixed, partly fixed, not fixed or regressed, with
evidence; and the lenses read the diff since the audit, and only it,
for new findings. Stage 0's evidence is taken again: the mutants that
survive are no more than before, every fuzz target runs its time
without a crash, the ratchet holds. The step is done when no confirmed
must-fix or should-fix finding is open, the decision list is answered,
and the register shows every later requirement met or recorded with its
reason, after at most three rounds; a last reader in a fresh context
(Fable 5.1 at `high`) then reads the rustdoc's front page and README as
a stranger would and reports what it could not use. What remains goes to
`plan/later.md` with its reason.

## Constraints

- The command's behaviour, every JSON form and every snapshot stay as
  they are unless a finding requires a change; each such change is a
  commit of its own and listed in the report.
- A commit that claims no change of the search keeps `nodes`, `splits`,
  `memo_hits` and `memo_entries` of every decided row of
  `bench/targets.sh` identical, and pinned CPU time on its rows over a
  second within two percent.
- The CSV columns of the baselines stay readable.
- Dependencies only where they earn their place (D10).
- No quantifiers are added: the note says what the first step of adding
  them is. The Rocq library (step 31) comes after this step and takes
  the proof term as it leaves it.

## Verification

The gate of stage 0 before every commit that touches code (clippy, the
tests, both `cargo hack` runs, the behaviour lock, the ratchet);
`bench/targets.sh` after any commit that touches
`search/` (its Horn rows included); the qcover suite through the harness
(`--spec bench/qcover`, intuitionistic affine) after any commit that
touches `search/horn/`; `linlog-bench run --all-families --timeout 5`; `summary
--before` on the committed baselines reproducing `bench/COMPARISON.md`;
`nix flake check`; `nix build .#doc`, and the rustdoc front page read as
a stranger would read it.

## Deliverables

- Thematic jj commits, each of which builds and passes the gate alone.
- `plan/notes/requirements.md`, `plan/notes/audit-rubric.md`,
  `plan/notes/api.md`.
- The behaviour lock's fixtures and test, the journeys, the ratchet and
  its flake check, the mutation and fuzz setup, the gate.
- `plan/reports/28-baselines.md` (stage 0), `plan/reports/28-audit.md`
  (stage 1), and `plan/reports/28-audit-and-refactor.md`: the surface
  before and after, what was renamed or removed (a table a later session
  can search), the design and the first-order plan in a page, the
  efficiency journeys before and after, the check rounds, decisions,
  deviations, open questions, what steps 29 to 31 must know, and the
  register with each requirement's state.
