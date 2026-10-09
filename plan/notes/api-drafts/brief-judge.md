# Brief: judge three drafts of linlog's API design

Three drafters wrote, independently and from three angles, a draft of
`plan/notes/api.md`, the design note step 28 of linlog's plan needs before
its fixes start (repository `/home/tux/Projects/own/linlog`; read
`CLAUDE.md` first). Their common brief is
`/tmp/claude-1000/-home-tux-Projects-own-linlog/4419c622-34f5-4db8-a2bc-0b8996d2cdd4/scratchpad/design/brief-draft.md`:
read it, it says what a draft must hold (the outline of thirteen sections)
and what it is for. The drafts are `draft-a.md` (the web client and the
wire forms first), `draft-b.md` (the proof term, the checker and the Rocq
library first) and `draft-c.md` (new engines, calculi and quantifiers
first), in the same directory. Another judge, of another model, scores the
same drafts without seeing your scores; the supervising session then
synthesises one design from the best of the three, grafting what the others
do better. Your judgement is what it grafts by, so be specific.

## What you judge against

- **The register**, `plan/notes/requirements.md` (383 KB, 254 entries
  `### R…`; list them with `grep -n '^###' plan/notes/requirements.md`):
  every entry of its sections Wire forms, Bounds and stops, Data model,
  Engine interface, Proof term and checker, Errors, Options and Export is
  the design's to meet or to place, and the conflicts C1 to C3 at its end.
  Do not read the whole file; read the entries a draft's choice touches.
- **The rubric**, `plan/notes/audit-rubric.md` (the A*, S*, C-* and AIP
  criteria).
- **The audit's list for the design**, in `plan/reports/28-audit.md`, "For
  the design and the fix sessions" and "From the review" (H9, H10 with C1,
  H18 with the bounds, HD1 to HD5), and its decision list (C1 to C3, T1 to
  T7). Findings by id from `plan/reports/28-audit-findings.json` (1 MB;
  never read it whole):
  `python3 -c "import json,sys; d=json.load(open('plan/reports/28-audit-findings.json')); f={x['fid']:x for k in ('standing','refuted') for x in d[k]}; [print(json.dumps(f[i],ensure_ascii=False,indent=1)) for i in sys.argv[1:]]" F7 F19`.
- **The plan's decisions**, `plan/README.md` "Design decisions" (D1 to
  D23; D15 to D18 above all) and the research note
  `plan/notes/research/design-constraints.md` (D-1 to D-12).
- **The code.** A draft's claim about the code is worth what it is when
  checked: check a sample of each draft's claims about the present code
  (types, signatures, invariants, call sites) against `core/src/`, and
  every claim on which a draft rests a choice that differs from the other
  drafts.

## How you score

For each of the thirteen sections of the outline, and for each draft:

- a score from 1 to 5 (5: a fix session could implement it as written and
  no later step would need a breaking change; 3: right direction, gaps a
  synthesis must fill; 1: wrong or missing), and two to five lines saying
  why, citing register entries, rubric criteria, findings or code lines;
- what it does better than the other two, if anything (the graft);
- what is wrong in it: a claim the code contradicts, a choice that breaks a
  register entry, a decision of the plan or D17, a design that a named
  later step could only extend by a breaking change, speculative generality
  the plan does not name, or an inconsistency with another section of the
  same draft.

Then, across the drafts:

- **Register coverage**: the register entries of the sections above that
  no draft meets or places, and those where the drafts disagree, with
  which one is right and why.
- **The disagreements that matter**: every point where the drafts choose
  differently on a public type, a signature, a wire form or a policy, with
  your verdict and its reason (the member type, atoms and terms, the root
  order, the error family, the bounds value and the stop, the options'
  wire forms, the version policy, `Node` and the checker, the net's
  vertices and criterion, the engine interface, and whatever else you
  find).
- **The spike**: which draft's specification of the spike (section 11)
  measures D17 best, and what it should add or drop.
- **The ranking**: the drafts in order, with the one-paragraph reason, and
  the base you would synthesise from with the list of grafts in order of
  value.

Taste is not a fault: say when two choices are equally defensible and do
not count it.

## Output

Write your judgement to the file your prompt names, in Markdown, about 20
to 50 KB; your final message says in under 200 words the file, the ranking
and the three grafts that matter most.

## The rules of the machine (binding)

- **You write only your judgement file.** Never edit, create or delete
  anything in the repository or the drafts.
- **Never run `git`, and never run `jj`** (a jj command snapshots the
  working copy and tries to sign, which hangs now). Read files directly.
- You may compile and run small probes to check a claim, only like this: in
  a memory-capped scope on the cores your prompt names, with a target
  directory of your own, for example
  `systemd-run --user --scope -p MemoryMax=8G -p MemorySwapMax=0 taskset -c CORES env CARGO_BUILD_JOBS=3 RUST_TEST_THREADS=3 CARGO_TARGET_DIR=/home/tux/Projects/own/linlog/target/design-judge-X nix develop -c cargo check -p linlog`.
  No thread or job count you hand any program exceeds the cores you were
  given; nothing you run is unbounded (bound every enumeration by size,
  give every run a timeout); no run takes more than ten minutes; no
  benchmark, no `nix flake check`, no `cargo test --workspace`. A scratch
  probe goes in the scratchpad, never in the repository. Anything else you
  would want to run, ask for in your final message instead of running it.
- Nothing outward-facing: no network, no `gh`.

## Two questions the supervising session adds

Answer each in a section of its own, after the ranking.

1. **An atom representation no draft proposes.** The drafts reserve
   `Term::{Pred, DualPred}(Atom, ArgsId)` beside `Atom`/`DualAtom`, with
   `Atom` a predicate symbol, and set a fragment bit on a predicate with
   arguments so that no propositional engine pairs `p(a)` with `~p(b)`
   (`plan/notes/research/impact-quantifiers.md` §1 item 1 lists the eight
   places that pair literals by atom). The alternative: `Atom` is an
   **interned atomic formula** (a predicate symbol applied to argument
   terms, hash-consed in the sequent's atom table; nullary for every
   propositional atom), `Term` keeps `Atom`/`DualAtom` as its only literal
   variants and gains only `Forall`/`Exists`, so that a ground first-order
   sequent is a propositional sequent with structured atom names that
   every engine and the checker decide correctly as they are, and only a
   binder sets the quantifier bit. Under a binder an atom's arguments
   contain bound variables (de Bruijn indices), and the first-order engine
   and checker instantiate it through the member's frame. Judge it against
   the drafts' choice: soundness (which places could go silently wrong
   under each), D17, the wire form, the first-order engine's needs
   (candidates by predicate symbol, unification), printing, and what step
   38 would find harder. Say which you would choose and why.
2. **H9 and H10.** The drafts fix them differently (a count of the roots
   written left of `⊢`, or "the goal is the last root"). For each draft:
   does its rule refuse both witnesses of H9 and H10 (extract them from
   the findings file), what does it change for one-sided input written
   with the goal not last, for JSON sequents and LLTP files, and does it
   remove the symmetric reading of an implication that H9 rests on
   (`.claude/rules/core-forest.md`, "The choices, made deterministically")?
