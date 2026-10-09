# Brief: a draft of linlog's API design (`plan/notes/api.md`)

You write one of three independent drafts of the design note that step 28
of linlog's plan needs before its fixes start. Three drafters work from
three angles in fresh contexts; two judges then score the drafts against
the register of later requirements and the audit's rubric, and the
supervising session synthesises one note from the best, grafting what the
others do better. So write the best whole design you can, and be most
thorough and most concrete on your angle (below, in your prompt).

## The repository and what the design is for

linlog (`/home/tux/Projects/own/linlog`) is a linear-logic proof-search
suite in Rust: the library `core/` (package `linlog`, about 42 000 lines),
the command `cli/`, the harness `bench/`. Read `CLAUDE.md` first. Twenty-
seven steps built it; step 28 audits it once and puts it in order right
before the first release 0.1.0 (step 30). Until the release the API may
change freely (D18); after it every change is a version. The design fixes
the public surface after step 28, its data model and its wire forms, and
how each later step (29 to 38) enters it, first-order logic above all:
atoms as predicates over terms, binders in the arena, a substitution beside
the forest, a trail of bindings in the engines, witnesses in proofs, with
what stays untouched for the propositional case, which must not pay for
quantifiers (D17: the target set's counters identical, pinned CPU time
within a few percent).

The goal of the step, in the author's words: at the release, the library is
one a stranger can use from its documentation, whose types say what they
hold, whose errors are one family and whose options are data with a wire
form; the API, the data model and the wire forms are designed for the steps
that come after it (the web client, the Rocq library, MELL nets with boxes,
cut, new engines and calculi, quantifiers), so that none of them needs a
second rewrite. No quantifiers are added in step 28: the design says what
the first step of adding them is.

## What to read (search large files, never read them whole)

- `plan/28-audit-and-refactor.md`: the step's prompt (stage 2 is yours).
- `plan/README.md`, "Design decisions" (lines ~497 to ~800): D1 to D23,
  D15 to D18 and D23 above all. Do not re-litigate them.
- `plan/notes/research/design-constraints.md` (D-1 to D-12, ranked: what
  the design must decide now) and `plan/notes/research/README.md`. These
  are evidence to check against the code, not findings to copy.
- `plan/notes/requirements.md`, the register (383 KB, 254 entries `### R…`
  under sections Wire forms, Bounds and stops, Wasm portability, Data
  model, Engine interface, Proof term and checker, Errors, Options, Export,
  CLI, Harness, Docs, Other, and "Conflicts for the author"): list its
  headings with `grep -n '^##' plan/notes/requirements.md` and read the
  sections your design answers; every entry of the sections Wire forms,
  Bounds and stops, Data model, Engine interface, Proof term and checker,
  Errors, Options and Export is the design's to answer or place.
- `plan/notes/audit-rubric.md`: the criteria (A*, S*, the Rust API
  Guidelines' C-* identifiers, AIP-126/140/151/180/193 for the JSON forms).
- `plan/reports/28-audit.md`: "Outcome", "The decision list" (C1 to C3,
  T1 to T7), "For the design and the fix sessions", "From the review"
  (H1 to H21, HD1 to HD5). The findings file
  `plan/reports/28-audit-findings.json` is about 1 MB: never read it whole.
  Extract findings by id with a script, for example
  `python3 -c "import json,sys; d=json.load(open('plan/reports/28-audit-findings.json')); f={x['fid']:x for k in ('standing','refuted') for x in d[k]}; [print(json.dumps(f[i],ensure_ascii=False,indent=1)) for i in sys.argv[1:]]" F7 F19`.
  The design must answer at least: the error family (F8, F30, F48), the
  versioned wire forms and their key names (F26, T1), one bounds value with
  a stop for every long call (F12, F62, F136), the options values' wire
  forms (F76 to F79), `Mode` as a type of its own (F7), members and atoms
  named for quantifiers (F19, F25), the written order of the roots (F24,
  C1, with H9 and H10), `#[non_exhaustive]` (F1), the place of a witness in
  `Interactive::apply` (F22), and H18 (the pool's stack) with the bounds.
  Search the findings for others that touch a public type (`area` is
  `library`; the titles are short) and answer those your angle meets.
- `.claude/rules/core.md` and the `core-*.md` rules of the modules you
  design (they hold invariants the code does not show).
- The research notes of your angle under `plan/notes/research/` (named in
  your prompt) and the later steps' prompts `plan/29-*.md` to `plan/38-*.md`.
- The code: `core/src/lib.rs` (the re-exports are the public surface),
  then the modules. Check every claim you make about the code against it.

## The decisions open for the author (start from the recommended answers)

The author has answered none yet. Design on the recommended answer of each
of C1 to C3 (register conflicts), T1 to T7 (taste) and HD1 to HD5 (from the
held-back audit), all in `plan/reports/28-audit.md`, and say for each one
whose other answer would change your design, how.

## What the draft holds

Write it to the file named in your prompt, in Markdown, about 40 to 90 KB:
complete enough that a fix session can implement it and a judge can score
it, with no padding. Use this outline so the drafts can be compared section
by section:

1. **Principles**: the few rules the surface follows (ownership, errors,
   bounds, options, extensibility, versioning), each with its reason.
2. **The public surface after step 28**: the module tree, what `lib.rs`
   re-exports, what becomes private; a table *before → after* of every
   renamed, moved, merged or removed public item (a later session
   searches it).
3. **The data model**: `Sequent` (arena, atom table, the written order of
   the roots, the reserved first-order tables), `Term`/`Kind`, `Atom`,
   `Forest` (its contract: numbering, roots, extra trees), the sequent
   member type, `Mode`, `Fragment`, `Reading`; `Proof`, `Node`, `Rule`,
   `Derivation`, `Inference`, `Interactive`; `ProofStructure`; the
   ordinary layer's values. Rust signatures for every type a later step
   extends, with size assertions where size matters.
4. **Errors**: one family, kinds or codes, refusal against fault (a caller
   must never be able to read a refusal as "invalid"), the serializable
   form, how each existing error type maps into it.
5. **Bounds and stops**: one bounds value, the stop with progress, the
   memory account, the `_within` readers; a table of every long public
   call with its stop and bound.
6. **Options**: every options value, its fields and defaults, its wire
   form, how the command, the batch mode and the web client set it (D15,
   D16); presets.
7. **Wire forms**: the version policy, every JSON form's schema (key
   names, enumerations, numbers above 2⁵³, unknown keys), with an example
   of each changed form; what is read back and what is write-only.
8. **Engines and the search's front door**: `Decide`, `Answer`,
   `Verdict`, `Refutation`, `Statistics`, the dispatch as data, how an
   engine and a refuter are registered.
9. **Exports**: the entry signature, the options values, the writers.
10. **How each later step enters it**: a short section per step 29 to 38,
    naming the items it adds and confirming no earlier item breaks;
    first-order logic in full (atoms as predicates over terms, binders in
    the arena, the substitution beside the forest, the trail of bindings
    in the engines, witnesses in proofs) and what stays untouched for the
    propositional case.
11. **The spike**: the data-model change for terms that stage 2's spike
    should build (in a throw-away jj workspace) to measure what it costs
    the propositional case on the target set's counters and stage 0's
    journeys (instruction counts): what exactly to change, how far, and
    what result accepts or rejects it.
12. **Findings answered**: a table of every finding id you answer and
    where.
13. **Decisions for the author**: the open ones (with your provisional
    answer) and any your design adds, each with its alternatives.

Be concrete: Rust signatures, JSON examples, names. Where you choose
between options, pick one and give the reason in a sentence or two; list
an alternative you set aside only where a judge would ask. Mark
"(unverified)" any claim about the code you did not check. Prefer what is
idiomatic and current best practice in Rust (2024 edition) and in the
standards the rubric cites. Do not design beyond what the plan names
(no speculative generality); do design so that every named later step
fits without a breaking change after 0.1.0.

## The rules of the machine (binding)

- **You write only your draft file** (in the scratchpad path your prompt
  names). Never edit, create or delete anything in the repository.
- **Never run `git`, and never run `jj`** (not even `jj st` or `jj log`:
  a jj command snapshots the working copy and tries to sign, which hangs
  now). Read files directly; history is not needed.
- You may compile and run small probes to check a claim (a size
  assertion, a signature), only like this: in a memory-capped scope on
  the cores your prompt names, with a target directory of your own, for
  example
  `systemd-run --user --scope -p MemoryMax=8G -p MemorySwapMax=0 taskset -c CORES env CARGO_BUILD_JOBS=3 RUST_TEST_THREADS=3 CARGO_TARGET_DIR=/home/tux/Projects/own/linlog/target/design-X nix develop -c cargo check -p linlog`
  No thread or job count you hand any program exceeds the cores you were
  given; nothing you run is unbounded (bound every enumeration by size,
  give every run a timeout); no run takes more than ten minutes; no
  benchmark, no `nix flake check`, no `cargo test --workspace` (a focused
  `cargo test -p linlog NAME` is fine). A scratch Rust probe goes in the
  scratchpad, never in the repository. Anything else you would want to
  run, ask for in your final message instead of running it.
- Nothing outward-facing: no network calls beyond reading local files, no
  `gh`.

## Your final message

Report in under 300 words: the file you wrote, its size, the three design
choices you consider most important and least obvious, and what you could
not verify.
