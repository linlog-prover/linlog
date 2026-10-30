# Step 42: first-order linear logic

The reports of step 42's sessions, one section each, under the plan
`plan/first-order/README.md`, which the first session wrote.

## Session 42a: the plan and the data model

Fable 5.1 at `high`, unattended, 2026-10-28, in the jj workspace
`../linlog-step42a` on `kkwnxqss` (linlog's `main` at the start; the
main checkout moved on by one plan commit meanwhile, which the
supervisor merges). Three commits beside the report's: the plan, the
data model, and the command's behaviour lock re-blessed because the
help texts name the quantifiers.

### Outcome

The step has its plan, `plan/first-order/README.md`: nine sessions (42a
to 42i) with their scope, model, effort, checks and panels, and the
decisions every later session builds on, each with its sources. The
data model is built: terms, predicates and the binders `∀` and `∃` in
the arena and in the intuitionistic surface, both parsers, every
printer, the JSON forms at wire level 7, and a refusal at every entry
that takes propositional sequents alone. A sequent with arguments but
no quantifier is propositional and decided by every engine as before.
The propositional case is unchanged: both target sets equal their
oracles on every decided row (145 focused rows and 64 net rows alike,
no verdict differs), and every journey of the ratchet is within its
ceiling, none raised.

### What was built

- **The arena** (`core/src/sequents/term.rs`, `fo.rs`, `mod.rs`): `Term`
  and `Kind` gain `Forall(TermId)` and `Exists(TermId)` after `Quest`
  (discriminants 12 and 13, the literal variants first; `Term` stays 12
  bytes, `Kind` 1), `dual` swaps them, `arity` is 1, `Kind::polarity`
  is negative for `∀` and positive for `∃`, `Kind::fragment` is the
  new bit `Fragment::QUANTIFIERS` (32) with the names `MLL1` to `LL1`
  and `IMLL1` to `ILL1`. The first-order tables `FirstOrder` hang on the
  sequent (`fo`), empty exactly when every atom is nullary and there is
  no binder: `symbols` as `(name, arity)`, a term arena of `FoTerm`
  (`Bound(n)`, a de Bruijn index, and `App(Symbol, Args)`), the argument
  lists as a flat vector with starts and lengths, each distinct list
  once, `atoms` parallel to the atom table with each atom's predicate
  and list, and `binders`, the variables' names by binder term, for
  printing. The atom table's key stays the canonical text (`p(f(a),
  b)`, an open atom `p(#0)`), so every caller keyed by names is
  unchanged. `Sequent::check` checks the tables (indices, arities, one
  role per name, topological terms) and the scope (a root or cut formula
  needs no binder above it); `optimize`, `merge_atoms`,
  `renumber_atoms`, `add`, `add_cut` and `pop_cut` keep the tables in
  step. New public items: `Sequent::{is_first_order, atom_predicate,
  atom_is_open, symbols, fo_terms, arguments, binder_name}`,
  `Forest::has_binders`, `Fragment::{QUANTIFIERS, MLL1, …,
  has_quantifiers}`, `Kind::is_binder`, `IllConnective::{Forall,
  Exists}`, the module `sequents::fo` (`FoTerm`, `Symbol`, `FoTermId`,
  `Args`, `SymbolFault`), the errors `Error::{Quantifiers, Symbol,
  LooseVariable, Predicates}`, `CheckError::Quantifiers`,
  `Unsupported::Quantifiers`, `Space::Symbol`.
- **The parser** (`core/src/parse/mod.rs`): `forall x. A`, `exists x.
  A`, `∀x. A`, `∃x. A` and `forall x y. A`, a binder opened per
  variable as an owed term on the one stack (`Pending::Binder`), which
  reduces at the end of its scope and breaks the reduction when a
  connective follows (so a quantifier binds loosest and reaches right);
  a predicate's arguments on an explicit stack of open applications
  (`Parser::predicate`), an identifier bound by an enclosing binder its
  variable, any other a constant or a function symbol, every symbol
  interned through `fo::Interner` with one arity and one role per name;
  the text becomes first-order at its first binder or predicate, when
  every atom read so far gets its nullary predicate
  (`enter_first_order`), and a nullary atom read after gets one as it is
  read. The first-order paths are out of line (`#[cold]`), so the
  propositional reader is the one function it was. `forall` and
  `exists` are no longer refused as reserved words: `ParseError::reserved`
  is never set (the field stays, so a stored error reads), and its
  constructor is gone.
- **The printers** (`core/src/sequents/notation.rs`): one atom writer
  for every target (`write_atom`, a predicate's arguments through
  `FirstOrder::write`, iterative), a binder as `∀x. ` with the body
  bracketed when binary and the binder bracketed inside another formula,
  so that the text reads back as the formula; a bound name that a symbol
  or an enclosing binder takes is written with `_` and a number after it
  (`x_1`), through a scope made at the first binder and dropped only
  then (`Slot`, `close`), so a propositional print pays no drop. LaTeX
  writes `\forall x.\,`, Typst `forall x. `, the SVG the text form; the
  Rocq export refuses by `Unsupported::Quantifiers`.
- **The intuitionistic surface**: the strict parser reads the same
  binders (its scan for classical tokens needs nothing), the reading
  gives a binder's body its side as `!`'s, `IllConnective::Forall` is an
  output `∀` or an input `∃` (and the reverse), the two-sided classical
  view writes the binder as written.
- **The wire** (`core/src/serialize/sequents.rs`, `ill.rs`, `wire.rs`):
  level 7, `wire::TERMS`, written whenever the tables or the binders'
  names are not empty: `symbols` (names with arities), `fo` (`{"B": n}`,
  `{"F": [symbol, [terms]]}`), every atom as `{"P": [symbol, [terms]]}`
  once one has arguments, the terms `{"∀": body}` and `{"∃": body}`,
  and `binders`, the variables' names in the order of the terms, beside
  the terms and not inside them so that a term stays three words and a
  propositional document reads as before; the intuitionistic document
  the same keys and tags, at the larger of level 5 and the sequent's.
  Reading interns equal lists as the parser does, checks the tables
  before any key is built, counts the terms every atom unfolds to
  against `limits.occurrences` (shared terms could name a key of
  exponential length from a short document), and reads a document of
  names and binders alone as names with its binders. The names of the
  atoms read as a plain list until the first predicate (`Atoms`), so the
  propositional path is byte for byte and instruction for instruction
  what it was. `core/tests/wire/level-7.txt` pins a one-sided, a ground
  and an intuitionistic document; `the_seventh_level_reads` reads them
  back and writes the same bytes.
- **The refusals**: no engine's `admits` contains the bit, so the
  dispatch answers `Error::NoEngine` ("no engine for MLL1 in classical
  mode yet") and a forced engine `Error::EngineRefused` by its fragment;
  the ordered modes `OrderedFragment`; the checker
  `CheckError::Quantifiers` (no verdict), interactive proving at its
  start, its JSON reader and its cut, and cut elimination
  `Error::Quantifiers` naming the entry; the proof structures by
  `has_nets`; the Rocq export `Unsupported::Quantifiers`; the
  refutations' checks `RefutationFault::NotApplicable`. The command's `check` reports the
  checker's refusal as an error (exit status 2), never as an invalid
  proof. The
  `unreachable!` arms on `Kind::Forall | Kind::Exists` in the engines,
  the nets and the refuters rest on those guards.
- **Tests**: `core/tests/parse.rs` (`quantifiers_and_predicates`: the
  syntax, the printed text read back, the keys, the fragment;
  `first_order_errors`: every refusal of the first-order syntax with its
  place and hint), `core/tests/serialize.rs` (`first_order_round_trip`:
  the one-sided and the intuitionistic form, the pinned bytes, the
  documents a file cannot hold, the exponential key refused),
  `core/tests/stability.rs` (level 7), `core/tests/first_order.rs`
  (every engine's refusal in every mode, the checker's, the session's
  on every path, the structures' and the refutation's; the join of two
  sequents' tables; a ground cyclic net at level 7).
- **Docs**: README ("First-order sequents", the syntax paragraph, the
  built and planned lists), the guide for AI models (`reference.md`'s
  syntax table and example, `SKILL.md`'s sentence), the changelog, the
  command's help text (the lock re-blessed for it), `.claude/rules/`
  (`core-sequents.md`, `core-forest.md`, `core.md`).

### The decisions, and where they depart from the design

The plan's "The decisions" settles what the goal names. Three depart
from `plan/notes/api.md` and say so there: a symbol has one arity per
name (where the research note proposed Prolog's `p/0` and `p/1` and
the design keyed a symbol by name, arity and kind), the checker's
refusal is `CheckError::Quantifiers` rather than a `Fault::Instance`
(a refusal is no fault of the term, and a caller must never report it
as one), and `Limits::terms` with `fo_size` are deferred to 42b with
the instance table they bound (the readers count unfolded terms against
`limits.occurrences` meanwhile); a fourth, lesser one: a clashing bound
name is written with `_` and a number, not primed, since `′` is no
identifier character. The arity decision was taken on the supervisor's
note of 2026-10-28: the parser's refusal spans the use that differs and
says so in a static hint ("a symbol takes the same number of arguments
everywhere: this use differs from the first"); it does not name the
first use's place, since `ParseError` carries one span and a static
hint, which `first_order_errors` pins.

### The reviewer's round

A fresh-context reviewer (Opus 5.5 at `high`) read the data model and
the plan against the requirements, D17 and `api.md`, and reported seven
breaks and two departures, each answered here; every break was fixed
and tested, and the reviewer confirmed D17 on the target sets itself.

1. *A nullary atom before the first binder left the predicate table
   misaligned* (`q |- forall x. p(x)` failed a debug assertion and could
   index out of bounds in release): held. `binder` now enters the
   first-order state before it opens the variables' scope; pinned by
   `quantifiers_and_predicates`.
2. *The JSON reader unfolded shared terms into an atom's key before any
   limit*: held. `Tables::read` counts the terms every atom unfolds to
   against `limits.occurrences` before building a key, and refuses with
   `Refusal::Occurrences`; pinned by a 40-level document of 2⁴¹ leaves
   in `first_order_round_trip`. `Limits::terms` stays 42b's, said in the
   plan.
3. *An atom's argument ids were not range-checked*: held.
   `FirstOrder::check` checks every entry of `args` against the terms
   first; pinned.
4. *A document with a binder and no symbols could not round-trip*:
   held. Such a document now reads as names with its binders and writes
   back as it was read (`Tables::of` writes names where no atom is a
   predicate; `level_of_sequent` is 7 whenever the tables or the names
   are not empty); pinned.
5. *Interactive proving let binders through its JSON reader and its
   cut, and `add_cut` left the tables behind*: held. `from_forest`,
   which the reader uses, and `cut` refuse a binder by
   `Error::Quantifiers`, and `add_cut` joins the formula's tables as
   `add` does; pinned by `a_session_refuses_a_quantifier_on_every_path`.
6. *`Sequent::add` interned every symbol as a function symbol and so
   renamed a propositional side's atom away from its pair*: held. The
   join gives each symbol its role from its side's atoms; pinned by
   `adding_sequents_joins_their_tables`.
7. *A cyclic net document of a ground first-order sequent kept level 2*:
   held; it takes the larger of 2 and the sequent's level; pinned.
8. *Departures not marked*: held; marked in the plan (above).
9. *The printer's scope scanned every enclosing name and every symbol
   per binder, cubic on deep shadowing*: held in part. The scope keeps a
   count per open name and the symbols as a set, so a binder costs a
   hash lookup, and a shadowed name gets a number (`x_2`), not one more
   character per level.

### The propositional case (D17)

- `bench/targets/42a.csv` against `bench/targets/cut-nets.csv` and
  `bench/targets/net-42a.csv` against `bench/targets/net-cut-nets.csv`
  (`linlog-bench summary --against … --counters`): no verdict differs,
  and of the 145 focused and 64 net rows both decide alike, none differs
  in `nodes`, `splits`, `memo_hits`, `memo_entries`, `links` or
  `tests`. The runs were made before the reviewer's fixes, which touch
  no engine, and the gate's search journeys after them are within 1.8 %
  of their ceilings as they were.
- The ratchet first failed on six journeys (`read-text` +7.2 %,
  `read-json` +10.4 % with the last-level misses up fifteenfold,
  `read-lltp` +3.0 %, `derivation-chain-64` +10.4 %, `render-latex`
  +7.7 %, `render-typst` +9.2 %), from inlining the propositional path
  lost: a bigger `Expression` (binder names inside the terms), an
  untagged atom form that serde buffers, the atom writer out of line,
  the parser's lookup split in two, a scope dropped per print. Each was
  fixed at its cause (the report's "What was built" says how, and
  `core-sequents.md` records what the counts rest on), by profiling
  both trees under callgrind function by function; no ceiling was
  raised. The gate's last run: `read-text` +1.58 %, `read-json` +1.38 %,
  `read-lltp` +0.46 %, `derivation-chain-64` +1.64 %, `render-latex`
  +1.63 %, `render-typst` +1.95 %, every search journey as before.

### What is not verified

- Pinned CPU time was not measured (the plan gives it to 42c's lift on
  an idle machine); the instruction counts above are the evidence.
- The `Hash` of `Term` is one write (its kind in the top byte over the
  second index's top bits, which an arena past 2²⁴ terms fills): a
  collision there costs a probe, never a wrong answer, and the memo and
  the arena still key by equality.
- The LLTP reader maps a first-order parse error's span and hint into
  the file as it maps the others; no LLTP problem is first-order, so no
  test exercises a binder there beyond `parse_errors_are_placed_in_the_file`.

### Checklist

- [x] `plan/first-order/README.md`: the sessions in order with scope,
  model, effort, checks and panels; the decisions settled; every source
  cited (`plan/first-order/README.md`, "Sources").
- [x] Terms, predicates, `∀` and `∃` in the arena and both surfaces'
  parsers, the printers, the JSON at level 7, with tests and a round
  trip (`cargo test -p linlog`: 362 unit tests and every test file
  pass, `first_order_round_trip` and `the_seventh_level_reads` among
  them).
- [x] Every engine refuses a quantified goal with a clear error
  (`linlog prove "forall x. p(x) |- p(a)"`: "no engine for MLL1 in
  classical mode yet", exit 2; `every_engine_refuses_a_quantified_goal`).
- [x] Both target sets equal their oracles on every decided row; the
  ratchet within its ceilings, none raised (the gate's table above).
- [x] README, the guide (its tests pass: `cargo test -p linlog-cli`)
  and the changelog true for what a user can type.
- [x] The reviewer's one round, each finding answered above.
- [x] Every code commit gated (the data-model commit with the lock's
  commit that follows it, since the help texts changed); `nix flake
  check` at the last commit: see the last message to the supervisor
  for its exit status.
- [x] `jj st` clean, the session's commits in `jj log`.
- [x] This section, and the next.
- [x] The last message to `linlog-orchestrator`.

### What the next sessions must know

- **The tables are the arena's companion, not the forest's.** A binder
  is a unary occurrence with its body at `o + 1` and nothing else in the
  forest knows it; an atom's arguments never reach the forest. 42b's
  instance table (members past `forest.len()` as `(OccId, FrameId)`)
  and 42d's `Bindings` extend `FirstOrder::terms` by offset and must
  truncate on undo; the sequent's tables are read-only after reading.
- **`fo.atoms` is parallel to the atom table or empty**, and `fo.atoms`
  empty with `fo.binders` not is a state only a JSON document of names
  and binders produces (never the parser): `write_atom` and
  `atom_predicate` take it as nullary atoms. `is_first_order()` is
  "the tables or the binders are not empty"; `fragment().has_quantifiers()`
  is "a binder exists"; `Forest::has_binders` scans the kinds.
- **An atom's key is its canonical text** and `atom_name` is that key:
  `p(f(a), b)`, an open atom `p(#0)`. 42b's `Instances::atom_of` maps a
  substituted atom back to the sequent's `Atom` where the key is equal.
  The symbol order is the order of first use in the text, arguments
  before their predicate (`c` before `p` in `p(x, c)`): the JSON's
  `symbols` follow it, and a sequent read back is equal to the parsed
  one in its tables too (`first_order_round_trip`).
- **The ratchet is tight.** `read-text`, `read-json`, the derivation and
  the render journeys are within 0.1 % to 0.6 % of their 2 % margin.
  Anything added to the parser's hot loop, the printers' `Enter`/`Exit`
  arms, `Term`'s `Hash` or the JSON proxies' propositional path is
  measured by journey (`linlog-bench ratchet --check --only NAME`) and
  profiled against the base build (`valgrind --tool=callgrind
  --toggle-collect="*journeys::measured*"` on both binaries, then
  `callgrind_annotate` function by function) before it is called done;
  `core-sequents.md` lists what the counts rest on. The three walks of
  `Notation` must keep their first-order paths out of line.
- **The guards are the soundness of the `unreachable!` arms**: every
  `Kind::Forall | Kind::Exists => unreachable!` in `search/`, `nets/`,
  `proofs/interactive/`, the refuters and the exports is reached only
  through an entry that refuses a binder first (the fragment in
  `admits`, `has_binders` in the checker, interactive proving, cut
  elimination and the Rocq export, `has_nets` in the structures,
  `NotApplicable` in `Disproof::check`). A session that lifts a guard
  gives every arm behind it its rule in the same commit.
- **The syntax is settled** (the plan, decision 1): one arity per name,
  one role per name, closed formulas, no numerals in term position,
  keywords no names; the parse errors of the first-order syntax are
  `ParseError`s spanning the use with a static hint (`ARITY`, `ROLE`,
  `KEYWORD` in `parse/mod.rs`). `ParseError::reserved` is never set.
- **0.2.0's break** (step 45): `Term::{Forall, Exists}`, `Kind::{Forall,
  Exists}`, `IllConnective::{Forall, Exists}` (non-exhaustive already),
  `CheckError::Quantifiers`, `Unsupported::Quantifiers` (non-exhaustive
  already), `Space::Symbol`; and the new types of `sequents::fo`.
- **The checker's eigenvariable condition and the proof term are
  settled in the plan** (decisions 2 and 3) and not built: 42b builds
  them under the panel, with the argument the plan gives to be attacked
  on a shared subproof under two `Forall` nodes and on an `Exists` below
  the `Forall` whose witness names the eigenvariable.

## Session 42b: the proof term and the checker

Opus 5.5 at `high`, unattended, 2026-10-28/29, in the jj workspace
`../linlog-step42b` on `yztusxvs` (linlog's `main`). The usage limit cut
the session once, after the code, the gates and the target sets; the
panel's first round died on it unreported and ran again afresh.

### Outcome

The checker checks first-order proofs. `Node::Forall(member, eigen,
premise)` and `Node::Exists(member, witness, premise)` act on members
that are occurrences under binders with frames of terms, numbered by
the proof's instance table (`Instances`), which `Proof::with_instances`
checks and keeps canonical and closed. The checker's two passes (the
frame pass, then the bottom-up pass comparing instances by number and
carrying the eigenvariables to the root) decide the eigenvariable
condition; the first implementation (`oracle.rs`) has first-order rules
of its own and agrees on every committed proof and every mutant. The
size estimate, the derivation (one-sided and two-sided: `∀R`, `∃L`,
`∃R`, `∀L`) and its LaTeX, Typst and SVG drawings write every instance
with its terms; the proof's JSON at level 7 adds the steps and
`instances`; `Limits::terms` with `Refusal::Terms` bounds first-order
terms on every reader, with the command's `--term-limit`. Sixteen
proofs written by hand are pinned under `core/tests/first-order/`: nine
accepted, seven refused, every new fault reached. The propositional
case is unchanged in its counters and within every ceiling.

### What was built

- **The proof term** (`proofs/mod.rs`, `occurrences/instances.rs`):
  `Node::Forall`/`Exists` (16 bytes still), `Eigen`, `FoTerm::Eigen`,
  `Instances` and `FrameId` with a builder (`new`, `eigen`, `apply`,
  `frame`, `member`) and readers; `Proof::with_instances`,
  `Proof::instances`, `occurrence` and `formula` reading the table;
  `Proof::new` on a forest with binders goes through it. The table is
  made canonical (equal terms, lists, frames and members one entry) and
  closed (every member a rule derives from a named one), so every member
  a check, a view or an error names has a number.
- **The checker** (`check/frames.rs`, `check/pass.rs`, `check/zones.rs`):
  the zones hold members; the pass is generic over `Instancing`
  (`Ground`, the propositional instance, compiled as before; `Framed`).
  New faults `Fault::Instance`, `Eigenvariable`, `Reintroduced`.
  `CheckError::Quantifiers` (42a's, unreleased) is gone with its guard.
- **The oracle**: members by their written-out frames, atoms compared by
  their written instances, its own eigenvariable sets.
- **The views**: `Rule::Forall`/`Exists` and their two-sided names and
  labels; `Build`, `Record` and `Measure` generic like the pass;
  `Notation::sequent` sends a sequent with instances out of line;
  `Notation::instance` writes an instance through a scope holding its
  frame's terms; eigenvariables named `a`, `b`, … past the symbols.
- **The wire**: `{"∀": [m, a, p]}`, `{"∃": [m, w, p]}`, `"instances":
  {"members", "frames", "terms"}` (`{"E": k}` an eigenvariable), level 7.
- **Bounds**: `Limits::terms` (`DEFAULT_TERMS` 50 000 000) bounds
  `Sequent::fo_size` in the parser, the JSON sequent reader (42a's count
  against `limits.occurrences` replaced) and a proof's table; the
  table's ids refused past `u32` at construction; the hash-consed
  numbers are `u64`.
- **Cut at first order**: a cut on a cut formula with binders and on an
  instance of one (`cut`, `cut_ill`, `cut_instance`); `push_cuts` names
  a dual binder's variable.
- **Docs**: README (first-order proofs, `--term-limit`, the built and
  planned lists), the guide's reference, the changelog, the rules files
  (`core-proofs.md`, `core-derivations.md`, `core-sequents.md`,
  `core-export.md`).

### Departures from the design

- **The frame pass derives no frame** (departs from the design): a node
  names its members with their frames, so the pass resolves them, finds
  the bodies and the faults; "a shared node receives equal frames" is
  the bottom-up comparison by number (`shared_apart`, `Missing`).
- **A cut on an instance brings its frame's eigenvariables** (departs
  from the design, an addition): without it `cut_frame` passed.
- **An eigenvariable no `∀` step introduces is a constant outside the
  signature** (departs from the design's "every eigenvariable is
  introduced by exactly one `Forall` node"): decision 2's fresh constant
  had no other way into the table (the counterexamples finding, below).
- Eigenvariables named with `_` and a number past a clash, not primed.
- The parser's arity refusal still names the second use only (the small
  finding of 42a's review, not taken: `ParseError` has one span).

### The panel's findings and answers

On the frozen tree `vmpysoru`, each reviewer in a fresh context with its
own scratch directory, under the machine's rules.

- **The argument (Fable 5.1, `high`)**: no break. It derived the
  invariant from the code first, then found the plan's argument resting
  on frames "built by the nodes between the root and N", which the code
  does not need and does not give (any leaf may name a framed member);
  the argument the code rests on (consumption of framed members) was
  stated nowhere. Answer: recorded in `core-proofs.md`. Its 18 attacks
  and 18 mutants were judged as the calculus or the plan's stricter
  form judges them. A misleading message for a witness out of range
  ("a root formula is term …") is fixed.
- **The counterexamples (Opus 5.5, `high`)**: no soundness bug in
  45 531 mutants of the committed proofs, about 610 000 random terms and
  14.7 million of their mutants against a brute-force MALL1 decider and
  a derivation re-checker, and 3.3 million fuzzed documents. Two
  findings, both held: (1) a panic on a JSON proof whose cut formula has
  a loose bound variable: the reader checked the scope before the cuts
  joined; now it checks after them (`a_cut_formula_with_a_loose_variable_is_refused`);
  (2) a fresh constant could not be a witness, so `∀x.p(x) ⊢ ∃y.p(y)`
  had no proof term: an eigenvariable no `∀` step introduces is now such
  a constant, and the committed `unintroduced` is accepted (checker and
  oracle alike).
- **The integers and limits (Sonnet 5.5, `high`)**: no wrap, no
  soundness break; three resource holes, all held: (1) the frame pass
  kept every witness's eigenvariables per term and per node, quadratic
  and outside the pass's bounds and stop (a 2.6 MB file took 4 GB): the
  walk moved into the bottom-up pass as counted, polled work
  (`the_walk_of_witnesses_is_counted_work`); (2) an error wrote a frame
  term in full, exponential for shared terms: cut at 4 096 characters
  (`an_error_cuts_a_frame_term`); (3) frames canonicalized by walking
  each chain anew, quadratic in a tower, and an axiom's frame read
  uncounted: each frame made once, the frame lengths counted as work.
  Its requests to run the command on its witnesses were covered by the
  library tests above.

### The propositional case (D17)

- Target sets at `vmpysoru` against `bench/targets/42a.csv` and
  `bench/targets/net-42a.csv` (`linlog-bench summary --against …
  --counters`): no verdict differs; of the 145 focused and 64 net rows
  both decide alike, none differs in a counter. The later commits touch
  the command and the first-order paths only.
- The ratchet, none raised: `check-qbf-20-2` +0.61 %, `check-wide`
  +0.17 %, `render-latex` +0.90 %, `render-typst` +1.09 %, `render-svg`
  +0.89 %, `derivation-chain-64` +1.98 %. It first failed (`check-*` up
  to +3.5 %, renders up to +28 %); the causes, found by callgrind against
  the base build, were the eigenvariable set in `State` (moved to a side
  table only the framed pass has), the formula walk instantiated twice
  (the instance printer writes through its own writer type), the sequent
  writer's shape (instances out of line), `Named::index`'s search and the
  labels' symbol scan (direct paths), and a branch per weight (one vector
  by member).

### What is not verified

- Pinned CPU time (42c's, on an idle machine); the evidence is the
  instruction counts.
- The command was not run on the panel's largest witnesses; the library
  tests reach the same bounds.

### Checklist

- [x] `Node::Forall`/`Exists`, the instance table, the two passes and
  the eigenvariable condition, their faults (`first_order.rs` tests).
- [x] Written proofs, every fault reached, a shared subproof under two
  `∀` steps and `∃` steps below the `∀` steps they name.
- [x] The oracle's rules agreeing; the cut rule at first order; the
  size; the term limits with refusals reached.
- [x] JSON read back equal; the derivation, LaTeX, Typst and SVG
  printing instances.
- [x] The panel's round, each finding answered above.
- [x] Both target sets, every decided row's counters equal; the ratchet
  within its ceilings, none raised.
- [x] Every code commit gated; `nix flake check` at the last commit: see
  the last message to the supervisor for its exit status.
- [x] This section, and the next.
- [x] The last message to `linlog-orchestrator`.

### What the next sessions must know

- The rules files hold the invariants: `core-proofs.md` ("First-order
  proofs": the table canonical and closed at construction, the frame
  pass, the eigenvariable flow, why it meets the condition, what is
  stricter than the calculus), `core-derivations.md` (instances in the
  views), `core-sequents.md` (`Limits::terms`, `push_cuts`' names),
  `core-export.md` (the quantifier rules' labels).
- **A fresh constant is an eigenvariable no `∀` step introduces**: 42d
  closes an unbound metavariable with one, and needs no symbols'
  extension for it.
- **The derivation journey has 0.02 % of margin** (`derivation-chain-64`
  +1.98 % against 2 %): 42a's +1.62 % plus 0.34 % of this session's,
  which callgrind with line tables places in bounds-checked slice
  indexing inside the inlined derivation-text path, in no first-order
  code. The next change to the derivation's propositional path must be
  measured against the base build first.
- `CheckError::Quantifiers` no longer exists; interactive proving, cut
  elimination, the classical two-sided view, the nets and the Rocq
  export still refuse a sequent with binders, their quantifier arms
  unreachable behind those guards.
- The JSON reader numbers argument lists apart from the parser, so a
  parsed and a read first-order sequent are unequal by `==` (a
  follow-up; the tests compare texts).
- 0.2.0's break adds: `Node::{Forall, Exists}`, `Rule::{Forall,
  Exists}`, `FoTerm::Eigen`, `Fault::{Instance, Eigenvariable,
  Reintroduced}`, `Refusal::Terms`; additions: `Instances`, `FrameId`,
  `Eigen`, `Proof::{with_instances, instances}`, `Sequent::fo_size`,
  `Limits::{terms, DEFAULT_TERMS, with_terms}`; removed (unreleased):
  `CheckError::Quantifiers`.

## Session 42c: the focused engine generic over its zone

Opus 5.5 at `high`, unattended, 2026-10-29, in the jj workspace
`../linlog-step42c` on `qlqzolyl` (linlog's `main`). The usage limit
cut the session once, during the panel; the counterexamples reviewer
died on it and ran again afresh.

### Outcome

The focused engine, one- and two-sided, is generic over its zone:
`Run<'a, Z: Zone>`, with `Ground` the one instance, whose members are
the forest's occurrences and whose methods are the engine's former code
inlined. No first-order code. The search is the same search: both
target sets equal their oracles on every decided row (145 focused and 64
net rows, no counter and no verdict differs), the search journeys'
instruction counts are within −0.75 % and +0.34 % of the base build's
(summed −0.33 %), no ceiling raised. **The pinned time confirms the
lift**: measured side by side with the base build, the focused engine's
rows summed take 0.9968 of the base's CPU time, no row more than 1.5 %
above; decision 20 stands, and D17's duplicated fast path is not taken.

### What was built

- **`core/src/search/focus/zone.rs`**: the trait `Zone` (members, the
  two zone types, the literal seam, the classes, the memo's gate) and
  the zone traits `Linear` and `Unrestricted`, implemented by `Context`
  and `OccSet` as `#[inline]` forwards; `Cursors` (moved from
  `split.rs`); `Ground` with the literal lists' marks (`Marks`,
  `lists`, `stamp`, moved from the engine).
- **The seam, as the plan names it**: a handle per dual list
  (`Zone::duals`) with its mark (`marked`) and its `Θ` flag
  (`first_lookup`); `dual_in`; `dual_from` with the chains' cursors;
  `mark_literals` charging its marks' bytes to the engine's `scratch`;
  `meets`; `class` and `same`; `fork` for a pool's worker (behind
  `parallel`); `root` for a goal's occurrence; `child` for a
  subformula's member; the forest passed to every call (the trait has
  no lifetime); `#[inline]` on every forward. The memo, the loop check
  and the waiting failures key occurrences only (`Linear::ground`,
  `Unrestricted::ground`) behind `Zone::memoizes`; a sequent a zone does
  not memoize is decided in `prove_unkeyed`, which `Ground` never
  reaches. There is no `Key<Z>`.
- **Generic over `Z`**: `Run`, `Alternative`, `Unrestricted` (the
  interned `Θ`), `ByClass`, `Pools`, `Join`, `Forcing`, `Opened`,
  `Frame`, `Spawn`, `search_goal`, `Plan::search`, `Plan::search_on`,
  `turns`, `alternate`; the front door and the pool name
  `search_goal::<Ground>`. Everything keyed by occurrences (the counts,
  the splits' counts, the reading, kinds and sizes) reads `Run::occ`.
- **The pinned-time script** `bench/pinned-time.py` (below) and its
  results; the rules files (`core-focus.md`: the seam, `Ground` as the
  old code, the decision; `bench.md`: the script), CLAUDE.md's commands.
- Nothing public changed (`search_goal` became private to the module,
  where its only callers are); no enum gained a variant, so 0.2.0's
  list gains nothing from this session. README, the guide and the
  changelog describe nothing that changed.

### The pinned time

Run on 2026-10-29 from 05:39 to 05:42 as the user unit
`linlog-step42c-pinned` under the machine lock: the base build
(`0f807b61`, `qlqzolyl`) and the lifted build (`af09b399`, `wtozsvrq`),
each a release `linlog-bench` copied to `target/pinned/`, every row
taken by both at once, base on core 2 and lift on core 3 for the first
half of the runs and the cores swapped for the second, five runs per
row; per run the `cpu_ms` of `linlog-bench run` or the measured part of
a journey. The rule, set before the run: the lift stands when the
focused rows summed are within 3 % of the base and no focused row's
median is more than 5 % above.

| row | base ms | lift ms | ratio | range of the runs' ratios |
|---|--:|--:|--:|---|
| search-qbf-20-2 | 26.42 | 26.60 | 1.0067 | 0.986–1.018 |
| search-chain-128 | 34.97 | 33.72 | 0.9642 | 0.945–0.991 |
| search-partition-no-5 | 0.410 | 0.407 | 0.9927 | 0.946–1.020 |
| search-chain-64-intuitionistic | 4.50 | 4.34 | 0.9631 | 0.947–0.971 |
| chain/256 | 280 | 270 | 0.9643 | 0.964–1.000 |
| growing/1024 | 110 | 110 | 1.0000 | 1.000–1.000 |
| qbf/24#1 | 110 | 110 | 1.0000 | 1.000–1.000 |
| mix/8, the focused engine forced | 680 | 690 | 1.0147 | 0.986–1.029 |
| mix/9, the focused engine forced | 5850 | 5900 | 1.0085 | 0.988–1.019 |
| chain/256 two-sided, forced | 280 | 270 | 0.9643 | 0.929–1.000 |
| chain/512 two-sided, forced | 2330 | 2260 | 0.9700 | 0.962–0.996 |
| **the focused rows summed** | 9706 | 9675 | **0.9968** | 0.981–1.003 |
| the other rows summed (the other engines' journeys, the batch, ordinary logic, the target set's two rows over a second) | 8786 | 8766 | 0.9977 | 0.997–1.007 |

`bench/targets/pinned-42c.csv` holds every row with its spreads (the
largest minus the smallest run over the median, at most 8 % on the
sub-millisecond journeys and 3.7 % on the rows of a tenth of a second
and more, where `cpu_ms`'s 10 ms ticks are most of it), its header
naming both commits and the load (1.74 to 1.99);
`bench/targets/pinned-42c.runs.csv` every run. Every pair agreed on
verdict and nodes but `Peterson-4_50_1`'s, which ends at its time limit.
How to run it again or resume it: `bench.md`, "Pinned time of two
builds"; a stopped unit (`systemctl --user stop linlog-step42c-pinned`)
resumes from its runs file when started again with the same command.

### The propositional case (D17)

- `bench/targets/42c.csv` against `bench/targets/42b.csv` and
  `bench/targets/net-42c.csv` against `bench/targets/net-42b.csv`
  (`linlog-bench summary --against … --counters`): no verdict differs;
  of the 145 focused and 64 net rows both decide alike, none differs in
  `nodes`, `splits`, `memo_hits`, `memo_entries`, `links` or `tests`.
- The ratchet at the lift (the gate): within every ceiling, none raised.
  Against the base build counted the same way: `search-qbf-20-2`
  −0.05 %, `search-chain-128` −0.75 %, `search-partition-no-5`
  +0.34 %, `search-chain-64-intuitionistic` −0.73 %, `batch-families`
  −0.01 %, `ordinary-pigeons` −0.07 %, the net, Horn and additive
  journeys and every checker, derivation and render journey equal or
  within 0.04 %; the search journeys summed −0.33 % (api.md 11's bounds:
  +1.0 % each, +0.5 % summed). `derivation-chain-64` is unchanged.

### Departures from the design

- **The rows of the pinned time** (departs from api.md 10.9's list, on
  the supervisor's note of 2026-10-29): the target set's rows over a
  second are now the Horn engine's two (`Peterson-4_50_1` at its time
  limit, `JoinFreeModules_joinFree-10_50_1` at its memory limit), since
  Mix went to the inverse method; they cannot show what the lift costs.
  They are kept as controls, and the focused engine is measured on
  rows it decides of a tenth of a second or more: `chain/256`,
  `growing/1024`, `qbf/24#1` one-sided, `mix/8` and `mix/9` with the
  focused engine forced (classical with Mix), `chain/256` and
  `chain/512` intuitionistic with the two-sided engine forced, beside
  the search journeys.
- **Not on an idle machine** (the brief's method, which replaces the
  plan's `baseline.sh` on the author's night): another session's nix
  builds ran on the other cores, which it could not pin (its message of
  05:40); both builds ran at the same minute on two performance cores,
  so the load weighed on both alike, and the spreads are as above.
- **`zone.rs` lives in `search/focus/`**, not in `search/zone/` (api.md
  10.9): the shared types never moved there at step 37 (the inverse
  method has its own), so the trait sits beside the engine that is its
  one user.
- **No trail in the trait**: the plan's 42c names the seam without the
  trail's `mark` and `undo`, which the spike's M2 placed; they are 42d's,
  with the `Bindings` and the `Trail` they would act on, at the places
  api.md 10.10 (e) lists.
- **`mark_literals` charges through the engine's `Charged`** rather than
  returning its bytes for the caller to charge: the same charge at the
  same point, one call less.
- **Two hooks the spike did not have**, so that `Ground` keeps its pools
  and charges exactly: `Zone::word_lists` (the pool the memo's key words
  come from, the member lists' own for `Ground`) and `Zone::listed` (the
  words as members where none repeats).

### The panel's findings and answers

On the frozen tree `af09b399` (the workspace `../linlog-step42c-panel`),
each reviewer in a fresh context with its own scratch directory, on
cores 4 and 5, 6 to 9, 10 and 11, after the pinned measurement.

- **The argument (Fable 5.1, `high`)**: no break. It stated the
  invariant from the code first (every trait call the identity lift of
  the code it replaced, at the same point, with no side effect more:
  identities for `root`, `occurrence`, `child`; the inherent methods
  behind every forward; `new` and `fork` the old fresh marks; the
  initial rules' `first_lookup` written only where the old code wrote
  `tried`; the words from the member lists' pool), checked every hunk
  against the base, and ran both builds: 72 problems under 13
  configurations (biases, forced engines, memory limits of 200 KiB to
  1 MiB, a copy bound, no memo), the problem files' 164 records and
  `run --all-families` in four configurations, every decided record
  equal field by field, the proofs included, and 72 of 72 verdicts
  equal on two threads. The only differences were runs the clock
  stopped in both builds. Nothing to answer.
- **The integers and limits (Sonnet 5.5, `high`)**: no break, no wrap.
  Its table covers the marks' stamp, the lists' index and count, the
  marks' and the cursors' charges, the cursors' `u32` casts, the pools'
  charged bytes, `budget - 1`, the `REPEATED` bit of the key words, the
  members' order and the pool's patterns; it ran both builds with
  overflow checks and debug assertions on and the account's every
  charge and release hashed, on 11 000 random sequents, under memory
  limits that empty the memo, on chains of forced splits of 10 000
  atoms, 100 000 copies of one atom, and 4 million atoms under `?` (192
  MB of marks), and found the memory-limit threshold of a proof at the
  same byte in both builds. Nothing to answer.
- **The counterexamples (Opus 5.5, `high`)**: no break. Its first run
  died on the usage limit unreported; a fresh one ran on the same frozen
  tree with a new scratch directory. Both builds through the command,
  deterministic, under work limits: 0 differences in 99 232 run pairs
  (byte-equal output, the proof and every counter included) over its
  own random sequents (classical forced onto the focused engine in every
  mode with and without Mix, intuitionistic onto the two-sided one) and
  the repository's generator (11 516 classical and 12 220 intuitionistic
  sequents, provable ones and mutants), each also under memory limits
  of 40 to 64 KiB that empty the memo, a memo of three entries, both
  biases and another copy bound; `run --all-families` with the focused
  engines forced, with and without a memory limit, every decided row's
  counters equal, the memory-limit rows too (the only differing rows
  were `mix` runs stopped by the clock and then refuted by the classical
  refutation; `mix/9` decided by the search has equal counters); 3 000
  sequents and the families on four threads with no contradiction. On
  the lift, the committed tests that compare the engines with the
  reference pass, widened fivefold in a scratch copy for the classical
  one (the intuitionistic one did not finish within its cap, so no
  result); 35 488 classical and 28 072 intuitionistic judgements of the
  engines against the committed reference, and 18 500 sequents against
  a reference of its own (a dyadic, unfocused, exhaustive prover with
  a copy bound, sharing no code), no contradiction. Nothing to answer.

### What is not verified

- The pinned time was measured with another session's builds on the
  other cores, not on an idle machine; both builds shared every minute
  of it, so the ratios hold, the absolute times less.
- `cpu_ms` counts in 10 ms ticks, so the rows of a tenth of a second
  resolve 1 in 11 to 28; the journeys and the rows of seconds carry the
  comparison.
- The flake's ratchet notes the cache model's misses on
  `check-qbf-20-2` moved by more than 5 %: a checker journey whose
  instruction count is exactly the base's (the lift does not touch the
  checker); not a failure, and not pursued.
- No second instance exists, so what a second instance costs `Ground`
  through the shared helpers (api.md 11.5, M3) is 42d's to measure.

### Checklist

- [x] The focused engine generic over `Z: Zone`, `Ground` its one
  instance, the seam of the plan's 42c, no first-order code
  (`wtozsvrq`).
- [x] Both target sets equal their oracles on every decided row
  (`42c` against `42b`, `net-42c` against `net-42b`).
- [x] The ratchet within its ceilings, none raised; the search journeys
  within api.md 11's bounds against the base build.
- [x] The pinned time, side by side on cores 2 and 3, the cores swapped
  halfway, five runs per row, in a resumable user unit; committed as
  `bench/targets/pinned-42c.csv` with its runs; the decision: the lift
  stands.
- [x] The panel's round on a frozen tree, each finding answered above
  (none held).
- [x] Every code commit gated (the lift's commit; the measurement's
  commit adds a script and data only); `nix flake check` at the last
  commit: see the last message to the supervisor for its exit status.
- [x] The rules files (`core-focus.md`, `bench.md`) and CLAUDE.md
  amended; README, the guide and the changelog need nothing.
- [x] `jj st` clean, the session's commits in `jj log`.
- [x] This section, and the next.
- [x] The last message to `linlog-orchestrator`.

### What the next sessions must know

- **42d adds `Framed` as a second `impl Zone`** in `focus/zone.rs`
  beside `Ground`, and the dispatch's choice of it by the quantifier
  bit at `search_goal::<Z>`'s callers (`Focused::decide` and the
  pool's `search_goal`). The trait has what `Ground` needed and what the
  spike found; the trail (`mark`, `undo` at api.md 10.10 (e)'s places),
  the enumeration of a premise's answers and the bindings are 42d's to
  add, with `Ground`'s versions empty and `#[inline]`.
- **`Ground` is the old code and must stay so**: its pools, charges and
  orders are those of the base engine (`core-focus.md`, "`Ground` must
  stay the old code"), which is why `Zone::word_lists` hands `Ground`
  the member lists' own pool and `Zone::listed` gives the words as its
  members. A second instance takes its own word pool (a `Pools` field
  that `Ground` never reads would be dead today, so it is not there).
- **`prove_unkeyed` is the path of a sequent the zone does not
  memoize**: no memo, no loop check, no waiting failure. It is the
  plan's "the memo and the loop check key ground stable sequents only";
  whether the loop check must see a framed sequent (copies under
  exponentials) is 42d's question, with its panel.
- **Measure the second instance against `Ground` alone**, by the
  journeys' counts against the lift (`af09b399` is the base to count)
  and by pinned time (`bench/pinned-time.py`, the focused rows it
  lists); the spike's M3 found the cost in shared helpers outlined
  (`Context::iter`, `Key::assign`, `OccSet::clone`, `Split::clone_from`),
  and its remedy `#[inline]` on them.
- **The target set no longer has focused rows over a second**: Mix goes
  to the inverse method, `chain/256` is the longest focused row at
  0.28 s. A pinned comparison forces the focused engines onto `mix` and
  the two-sided one onto `chain` (`bench.md`).

## Session 42d: the first-order focused engine

Opus 5.5 at `high`, unattended, 2026-10-29, in the jj workspace
`../linlog-step42d` on `mksrqnul` (linlog's `main`). The author's stop
from 11:05 paused the session; it resumed at 13:46 from its checklist.

### Outcome

The focused engine decides first-order sequents in the classical modes,
with Mix and in affine mode: a second zone, `Framed`, beside `Ground`,
with unification and the occurs check on shared terms, the
eigenvariable condition kept by levels, a trail undone where the arena
releases nodes, the identities found by unification through the zone's
literal seam, a premise asked for another answer where the other fails,
and the proof closed (every witness a term, an unbound metavariable a
fresh constant) and accepted by the checker. MLL1 and MALL1 are decided;
with exponentials the copy bound and the three verdicts carry over. The
counts carry over per predicate and refute by kinds of their own; the
first-order bounds `witness_depth` and `instances` are options with
flags, keys and reasons. A committed reference prover (unfocused,
freshness) agrees with the engine on generated sequents, its reviewer
found nothing, and the panel found three resource holes, all fixed. The
propositional case pays nothing measurable: both target sets equal
their oracles on every decided row, the ratchet is within every
ceiling with none raised (the search journeys +0.18 % to +0.42 % of the
base's instructions), and the pinned time of the focused rows summed is
1.0103 of the base's, within the rule set before it.

### The rule of the pinned time, set before the run (14:49)

The propositional engine pays nothing (D17) when, measured side by side
with `bench/pinned-time.py` (the base build `mksrqnul`, the change's build
on top, each row five times, cores 2 and 3 swapped halfway), the focused
rows summed take at most 3 % more CPU time than the base's and no focused
row's median is more than 5 % above the base's: 42c's rule, kept so that
the two measurements read alike. A row of a tenth of a second resolves
1 in 10 to 28 in `cpu_ms`'s ticks, so a row above 5 % there is looked at
with the journeys' instruction counts before it is called a regression.
Past the rule, the remedy is the spike's: inline what the symbol diff
shows outlined, and measure again.

### What was built

- **The `Framed` zone** (`core/src/search/focus/framed.rs`), the second
  `impl Zone` beside `Ground`: a member past the forest's length is an
  occurrence of a *span*, the instance of a binder's body (one id per
  occurrence of the body's subtree, the body's frame for all), so
  `child` is arithmetic and `occurrence` a binary search over the spans;
  frames are cons cells of terms; a term is an id of the sequent's arena
  read under a frame, or a variable of the search (a metavariable, bound
  to such a term, or an eigenvariable), never built during the search;
  unification (`unify_pairs`, `bind`) is iterative, each pair of shared
  terms once, with the occurs check and the levels in one walk; the
  trail holds the bindings and lowerings, a `Mark` where every table
  stands, `undo` pops and truncates. The linear zone is a sorted list of
  members with repeats, the unrestricted one a sorted set.
- **The rules a first-order zone adds** (`first_order.rs`): the
  identities by unification (`initial_framed`: a pair of opposite
  literals of one predicate whose terms unify, or a literal and a copy of
  an opposite one of `Θ`, each tried in turn, affine weakening of the
  rest), `∃` in focus (a fresh metavariable for the body, still in
  focus), `∀` in the asynchronous phase (`decompose`, a fresh
  eigenvariable, the `∀` node built on the way back), and `answers`, the
  rule of two premises that asks the first for another answer (below),
  used by `⊗`'s premises, Mix's parts and `&`.
- **The generic engine's first-order paths** are behind `if
  Z::FIRST_ORDER`, a constant, each inside a match's existing fallback
  arm so that `Ground`'s decision trees are what they were; the zone is
  undone where the arena releases nodes (`decompose`, `exists`,
  `prove_unkeyed`, `answers`, each identity tried).
- **The proof returned closed** (`Zone::close`, from `Plan::search` on a
  proof): the instance table built from the spans and frames under the
  final bindings, a metavariable left unbound a fresh constant (an
  eigenvariable number past every one the search made, which no `∀`
  step introduces, 42b's constant outside the signature);
  `Finished::instances` carries it, and `Answer::of_arena` builds the
  proof with `Proof::with_instances`. The front door checks every proof
  as before.
- **The dispatch** (`search::DISPATCH`): the two classical rows take
  `Fragment::LL1`; `Focused::admits` takes `LL1` one-sided and `LL`
  two-sided; `Focused::decide` runs a first-order goal on one thread
  (`search_goal::<Framed>`). The Mix row's inverse-method turn is refused
  by the inverse method's fragment, so a first-order goal takes none.
- **The prunes** (api.md 10.10 (f)): the counts, the count equation and
  the bias per predicate where a predicate has an open atom
  (`counts::Predicates`, `bias.rs`), per atom otherwise; a quantifier's
  row and weight are its body's; the forced split of a literal of such a
  predicate, and of a `!` factor where any atom is open, switched off;
  the classes kept for members outside every binder; the memo, the loop
  check and the chains of free splits switched off for a first-order goal
  (below).
- **The refuters** (decision 5): `Refutation::UnbalancedSymbol` and
  `OneSidedSymbol` (JSON `unbalanced_symbol`, `one_sided_symbol`, the
  predicate by its symbol index, checked against the sequent's symbols
  when read), produced by `counts::refutation` for a group of an open
  predicate; `Disproof::check` checks `UnbalancedSymbol`, `Unbalanced` on
  a quantified sequent where its predicate has no open atom, and
  `Equation` with a quantifier counted as its body (`check_balance` is
  the shared count); `Classical` and `Saturated` stay `NotApplicable`
  with a quantifier; the classical refuter answers nothing for a forest
  with a binder; the Rocq disproof writer refuses a quantified sequent.
- **The bounds** (D16): `Options::witness_depth`
  (`DEFAULT_WITNESS_DEPTH` 64, `with_witness_depth`, JSON
  `witness_depth`, `--witness-depth N`, `search.witness_depth`,
  `Reason::WitnessDepth`) and `Options::instances` (`DEFAULT_INSTANCES`
  2¹⁶, `with_instances`, JSON `instances`, `--instances N`,
  `search.instances`, `Reason::Instances`), each cutting a branch like
  the copy budget, the command's line naming its flag; member ids past
  `u32` answer `Reason::IndexLimit`, tested through a test-only lowered
  limit.
- **The reference prover** (`core/src/search/reference_fo.rs`,
  test-only): one-sided classical first-order linear logic, linear and
  affine, with and without Mix, unfocused and exhaustive, unknowns bound
  by unification with the occurs check, the eigenvariable condition by
  freshness on the closed derivation, the goals on a stack so that a
  failure backtracks into every choice made before, the invertible rules
  first; with its generator of random first-order sequents and the
  comparison `the_focused_engine_agrees`.
- **Tests**: `core/tests/first_order.rs`
  (`the_focused_engine_takes_a_quantified_goal_and_no_other`,
  `the_focused_engine_decides_first_order_sequents`,
  `the_first_order_families_have_their_verdicts`,
  `the_first_order_bounds_stop_the_search`,
  `the_counts_refute_first_order_sequents`), `focus::tests::
  a_first_order_search_past_its_member_ids_stops`, the reference's two
  tests; the options' JSON, the reasons' and refutations' names updated
  where their lists grew.
- **Docs**: README ("First-order sequents", the built and planned
  lists), the guide for AI models (`SKILL.md`'s syntax sentence,
  `reference.md`'s first-order paragraph, refutation kinds, reasons and
  flags), the changelog, the command's help (the behaviour lock
  re-blessed in a commit of its own), `.claude/rules/core-focus.md`
  ("The first-order zone") and `core-search.md` (the refuters with
  quantifiers, the `LL1` rows, the bounds).

### Departures from the design

- **The memo and the loop check are off for a first-order goal,
  ground stable sequents too** (departs from the design: the plan's
  decision 4 and api.md 10.10 (f) key the ground stable sequents). A
  proof of a ground stable sequent holds members whose metavariables a
  later undo of a rule around it unbinds, so only failures could be
  kept; and after the first `∃` step the members of a sequent are
  rarely all outside binders, so a ground key would seldom match. The
  supervisor's note asked whether an unprovable first-order sequent with
  `!` must then run out the clock: it does, but no memo would end it
  either: `p(a), !∀x.(p(x) ⊸ p(s(x))) ⊢ p(b)` meets a new instance
  `p(sⁿ(a))` at every level, which keys by instance would never repeat
  (MELL1 is undecidable; a saturation argument is a later step's). An
  instance-keyed memo needs canonical instance ids, which the plan lists
  with the variant keys for later. Without exponentials the search is
  finite and decides.
- **Another answer by exclusion, not by a resumable search** (the design
  left the mechanism to this session and its panel): the first premise
  is searched again with every recorded failed answer excluded at the
  bindings, rather than resumed; a re-search costs the first premise's
  search again, and the mechanism needs no continuation in the engine.
- **`FoTerm` gains no `Meta`** (departs from the plan's decision 1, which
  gave it `Meta(u32)` in this session): the search's terms are a private
  type (`TRef`, `Var`), and a proof never holds a metavariable, so no
  public enum gains a variant for it.
- **No pool for a first-order goal** (departs from the design's "the
  `&` premises and the parallel cubes split only premises sharing no
  unbound metavariable"): a first-order goal runs on one thread,
  whatever `--jobs` says; the default bias's two searches still
  alternate.
- **`instances` bounds the instances held at once**, not per branch
  (api.md 6.2's "instances per branch"): the spans the search holds,
  which is what the tables cost and what the trail truncates.
- **The witness depth is measured when the binding is made**, on the
  term as its bindings then resolve it; a later binding of a
  metavariable inside it is measured on its own.
- **One counter of eigenvariables for the levels**, not one per branch:
  an eigenvariable made on another branch reaches a metavariable only
  through older metavariables, whose levels refuse it, so the levels a
  branch would count apart give the same refusals.
- **A `!` factor forces no split where an atom is open** (an addition to
  the table of api.md 10.10 (f)): a promotion may bind a metavariable
  the other factor holds, which the forced chain would never ask for
  another answer.

### The closed enums' new variants (for step 45)

No variant of `Term`, `Kind`, `Node`, `Rule` or `FoTerm`. New variants
of non-exhaustive enums: `Reason::WitnessDepth`, `Reason::Instances`,
`Refutation::UnbalancedSymbol`, `Refutation::OneSidedSymbol`. Additions:
the structs `UnbalancedSymbol` and `OneSidedSymbol`, the fields
`Options::witness_depth` and `Options::instances` (`Options` is
non-exhaustive), `Options::{DEFAULT_WITNESS_DEPTH, DEFAULT_INSTANCES,
with_witness_depth, with_instances}`.

### The propositional case (D17)

- **Target sets**: `bench/targets/42d.csv` against `bench/targets/42c.csv`
  and `bench/targets/net-42d.csv` against `bench/targets/net-42c.csv`
  (`linlog-bench summary --against … --counters`): no verdict differs; of
  the 145 focused and 64 net rows both decide alike, none differs in
  `nodes`, `splits`, `memo_hits`, `memo_entries`, `links` or `tests`.
- **The ratchet** (the gate, none raised): within every ceiling. Counted
  against the base build (`mksrqnul`, which holds 42c's lift) the search
  journeys are +0.18 % to +0.42 % (`search-qbf-20-2` +0.25 %,
  `search-chain-128` +0.23 %, `search-partition-no-5` +0.18 %,
  `search-chain-64-intuitionistic` +0.24 %, `search-spec-chain`
  +0.42 %, `search-wide-m2-256` and `search-additive-14` equal), every
  other journey within 0.2 %. The first gate failed: `search-chain-128`
  +2.60 % over its ceiling, `search-chain-64-intuitionistic` +2.58 %,
  `search-spec-chain` +2.04 %. Callgrind against the base, function by
  function, found the causes and each was fixed there: `Ground::dual_in`
  left out of line once it took the set rather than a closure (80
  million instructions on the chain; `#[inline(always)]`), the `!`
  switch-off and the guarded match arms changing `Ground`'s decision
  trees in `forced_factor`, `decompose`, `decide_with` and `focus_on`
  (the `!` switch-off moved into the counts, the first-order arms into
  the fallback arms), and the counts' atom mapping outlined in their
  set-up pass (two `#[inline(always)]` functions, and a row copy of its
  own for a quantifier).
- **Pinned time** (the rule above, set at 14:49), run 15:13 to 15:15 as
  the user unit `linlog-step42d-pinned` under the machine lock, the base
  build (`mksrqnul`) on core 2 and the change's on core 3 for half the
  runs and swapped for the rest, five runs per row, the load 2.25 to
  2.92 (the author's browser on other cores; the only other session
  busy then, asked, kept off cores 2 and 3):

| row | base ms | change ms | ratio | range of the runs' ratios |
|---|--:|--:|--:|---|
| search-qbf-20-2 | 27.61 | 27.28 | 0.9881 | 0.980–1.020 |
| search-chain-128 | 33.86 | 34.31 | 1.0133 | 0.961–1.020 |
| search-partition-no-5 | 0.425 | 0.421 | 0.9906 | 0.940–1.002 |
| search-chain-64-intuitionistic | 4.40 | 4.41 | 1.0041 | 0.985–1.028 |
| chain/256 | 270 | 280 | 1.0370 | 1.000–1.039 |
| growing/1024 | 110 | 110 | 1.0000 | 1.000–1.100 |
| qbf/24#1 | 110 | 110 | 1.0000 | 1.000–1.091 |
| mix/8, the focused engine forced | 690 | 700 | 1.0145 | 1.014–1.030 |
| mix/9, the focused engine forced | 5870 | 5950 | 1.0136 | 0.997–1.024 |
| chain/256 two-sided, forced | 270 | 280 | 1.0370 | 0.964–1.037 |
| chain/512 two-sided, forced | 2300 | 2290 | 0.9957 | 0.987–1.018 |
| **the focused rows summed** | 9686 | 9786 | **1.0103** | 0.997–1.022 |
| the other rows summed | 8867 | 8847 | 0.9978 | 0.996–1.003 |

  The rule holds: the focused rows summed are 1.03 % above the base,
  within 3 %, and no focused row's median is more than 5 % above; the
  two rows at 1.037 are one 10 ms tick of 270, and their journeys'
  counts are +0.23 % and +0.24 %. `bench/targets/pinned-42d.csv` holds
  every row with its spreads, `bench/targets/pinned-42d.runs.csv` every
  run.

### The reference's reviewer

A fresh-context reviewer (Opus 5.5 at `high`) read
`core/src/search/reference_fo.rs` against the calculus before it judged
the engine, with its own scratch copy on cores 4 to 11. **No finding.**
What it ran: 34 sequents whose verdicts it derived by hand (the
eigenvariable condition through a chain of unknowns, the drinker formula
linear and affine, a witness shared by the premises of `&`, an `∃` that
waits for a `⊕` and a `&`, Mix, the exponentials, the units); about
31 000 runs against a concrete prover of its own sharing no code (every
rule on every formula, a fresh constant per `∀`, ground witnesses over
the sequent's symbols, the eigenvariables in scope and one constant
more), at formula depths 3 and 4, in the linear, Mix, affine and
Mix-with-affine modes at copies 0 to 2: the reference never answered
`Unprovable` where the other found a proof, and every proof the
reference claimed the other derived too; 600 propositional sequents
against `reference::sequent`, no contradiction; and a reading of the
opening of binders, the undo on every failing path, the occurs check,
the freshness check (sound, and complete since a most general unifier
fails it only if every solution does), the invertible rules and the
contraction flag. Its one note, no finding: like the propositional
reference, `sequent` ignores `mode.order`, which no caller sets; left as
it is, as the propositional one is.

The committed comparison (`reference_fo::tests::the_focused_engine_agrees`,
360 generated sequents in three modes, both the dispatch and the engine
forced) passes; widened 25-fold in a scratch copy (9 000 sequents, 27 000
cases, release build) it found no disagreement: the reference decided
25 921, proved 11 381, and the engine proved 11 290 of those within its
work bound (the rest unknown), against none.

### The panel's findings and answers

On the frozen tree `080c7adf` (the workspace `../linlog-step42d-panel`),
each reviewer in a fresh context with its own scratch directory, on
cores 4 and 5, 6 to 9, 10 and 11, after the pinned measurement.

- **The argument (Fable 5.1, `high`)**: no break. It stated the
  invariants from the code first (unification as most general unifiers
  on shared terms with the occurs check, acyclic bindings; the trail
  undone with the nodes at the same points and nothing read past a mark
  after an undo; the levels sound against the checker's global
  condition and complete because a refused binding always puts an
  eigenvariable into the conclusion of its own `∀` step, the other order
  reached as another alternative; the answer enumeration sound by
  lifting a proof under an instance to one under the pattern, complete
  because a refusal happens only at bindings and the intermediate tuples
  of a proof are more general than its final one, and ending by
  `exhausted`; the closing's fresh constants past every surviving
  eigenvariable; the prunes kept and the switch-offs complete), checked
  each line by line, and ran about 130 sequents of its own in every
  classical mode, every proof re-read by `linlog check`. Two
  observations, no finding: (1) a forced chain whose `!` factor fails
  released its nodes but did not undo the zone, harmless while no atom
  is open (no binding happens) but against the stated invariant:
  answered, the chain undoes the zone with its nodes now; (2) the witness
  depth also bounds a closed input term a witness takes: as specified,
  the README shows it.
- **The counterexamples (Opus 5.5, `high`)**: no break. A second
  reference of its own in Python sharing no code (ground witnesses over
  the sequent's names, closed under its function symbols to depth 1 or
  2, freshness literal, unprovable only where complete); 8 539 generated
  sequents in the classical, Mix, affine and affine Mix modes, about
  270 000 engine decisions under ten option sets (copy bounds 0 to 3 and
  none, both biases, the focused engine forced, `-j 4`, memory limits of
  8 KiB to 256 KiB, tight instances and witness depths): never a proof
  against a refutation from either reference, never a proof under one
  option set and a refutation under another, and the two references
  never contradicted each other (about 23 000 cases decided by both);
  the committed comparison widened to 144 000 cases, no contradiction;
  the families (the witness family proved up to 89 alternatives and at
  200 and 1 000, refuted with `q(b)` at every size; the chain proved
  within `n` copies and never within `n − 1`, up to 65, and at 66 to 70
  unknown at the default witness depth, as specified); 150
  graph-reachability sequents judged against a breadth-first search;
  about 60 000 returned proofs read back by `linlog check`, all valid;
  every other engine and the ordered modes refuse `LL1`. Its two notes
  (the copy bound counts uses, not contractions; `-j 4` on three cores
  prints a note) are no findings.
- **The integers and limits (Sonnet 5.5, `high`)**: three findings, all
  held, each reproduced before it was answered.
  1. *A witness nested deeper than about 180 000 applications around an
     eigenvariable aborted the command by a stack overflow*
     (`|- forall z. (~p(f(…f(z)…)) par (exists y. p(y)))` at 200 000,
     with `--witness-depth 4294967295`), in 42b's `Canonical::term`,
     which recursed per argument list and which the engine's closed
     proofs now feed: reproduced (exit by `stack overflow, aborting`).
     Fixed at the cause: the walk has a stack of its own, arguments
     first; 200 000 and 1 000 000 deep now prove and check;
     `a_deep_witness_closes_without_recursion` pins it at 100 000.
  2. *The time limit overshot in proportion to the terms' depth*: a
     split search unifying terms 20 000 deep at every step ran 48 s past
     a 1 s limit, since a split search polls every 4 096 steps and a
     unification was no step: reproduced at 4 000 deep (8.7 s against
     1 s). Fixed: the zone counts the terms its walks visit
     (`Framed::steps`), and the engine counts them as work, one unit per
     sixteen, and polls on them every 4 096 (`Run::poll_terms`, after
     each unification and each step of `answers`); both witnesses now end
     at the limit (0.98 s, 1.00 s); `a_deep_unification_is_counted_work`
     pins it deterministically under a bound on work.
  3. *The zone's memory was not charged to the bound*: a search held
     882 MiB under `--memory-limit 16MiB`: reproduced. Fixed in part:
     the zone's tables and its walks' retained buffers are charged to the
     engine's account as they grow (`Zone::held`, in the same poll), and
     the witness now ends `unknown` at the memory limit after 0.4 s. The
     walks' temporary sets in `equal`, `fits`, `instance_of`, `pattern`
     and `open_answers` stay uncharged; each is as large as the terms it
     walks, so bounded by the input's terms and the witness depth, and
     freed at the walk's end.
  Its note 4 (the witness depth measured as each binding is made, so a
  proof may hold a deeper witness built by several bindings) is the
  specified behaviour (`Options::witness_depth`'s doc, the departures
  above). Every other integer held at its limit, driven there with
  overflow checks on: member ids and `member_limit` (13 607 refusals in
  a sweep, no wrong verdict), frames below `u32::MAX`, variable ids, the
  eigenvariable numbers and the fresh constants past them (a forged
  clash rejected by the checker), the witness depth at and past it, the
  instances at 0 to 13 and `u32::MAX`, the `child` arithmetic, the casts
  of the patterns' counts, the counts' `i32` and the predicate groups,
  the JSON reader's symbol index of a refutation, and the iterative walks
  on terms nested up to 5 000 000 deep.

The fixes are the commit after the frozen tree; they touch the first-order
zone and 42b's canonicalization only (the forced chain's undo is behind
`Z::FIRST_ORDER`), and the gate's ratchet counts them.

### What is not verified

- The pinned time was measured on the build before the panel's fixes;
  those touch the first-order zone and the proof's canonicalization, and
  the gate's ratchet counts every propositional journey exactly as before
  them, so the measurement stands for the last commit by the counts, not
  by a second run.
- No problem library is first-order: the evidence is the generated
  sequents, the two references, the families and the panel's runs. The
  families as `linlog::families` entries, their harness columns and the
  comparison with LinearOne are 42h's.
- The walks' temporary sets are not charged to the memory bound (the
  panel's third finding, fixed in part; above).
- An unprovable first-order sequent with `!` that the counts do not
  refute runs to the time limit (by default) or the copy bound; MELL1 is
  undecidable, and no memo of this session would end it.

### Checklist

- [x] `Framed` as the second `impl Zone`, `Bindings` and the trail with
  levels, unification with the occurs check on the shared terms, the
  literal seam, a premise's answers enumerated under a shared
  metavariable (`first_order.rs`, `framed.rs`); shown by the command on
  first-order sequents in the classical modes, Mix included, every proof
  checked (the session's last messages).
- [x] The prunes' first-order forms or their switch-off (above; the
  memo off, a departure).
- [x] The dispatch row `LL1 → Focus` in the classical modes, Mix
  included.
- [x] The refuters' carry-over (decision 5): the per-predicate kinds,
  their checks and JSON; the classical refuter's refusal.
- [x] The proof returned closed, an unbound metavariable a fresh
  constant.
- [x] `Options::witness_depth` and `Options::instances` with
  `DEFAULT_*`, flags, JSON keys and reasons; each bound's stop tested by
  an input that reaches it (`the_first_order_bounds_stop_the_search`),
  and the member ids' refusal
  (`a_first_order_search_past_its_member_ids_stops`).
- [x] The first-order reference prover committed, its reviewer's round
  (no finding), the reference against the engine on generated sequents
  with no disagreement.
- [x] Both target sets against `42c.csv` and `net-42c.csv`, every
  decided row's counters equal; the ratchet within its ceilings, none
  raised; the pinned time with its rule set before, in the user unit
  `linlog-step42d-pinned` on cores 2 and 3, resumable.
- [x] The panel's round on a frozen tree, each finding answered above.
- [x] README, the guide (its tests pass) and the changelog true.
- [x] Every code commit gated (the engine's with the lock's that follows
  it, the reference's, the panel's fixes'); `nix flake check` at the
  last commit: see the last message to the supervisor for its exit
  status.
- [x] `jj st` clean, the session's commits in `jj log`.
- [x] This section, and the next.
- [x] The last message to `linlog-orchestrator`.

### What the next sessions must know

- **`.claude/rules/core-focus.md`, "The first-order zone"**, holds the
  invariants: members as spans, terms read through frames and never
  built, levels by one counter, the trail undone where nodes are
  released, another answer by excluded patterns, the switch-offs, term
  work polled and the zone's tables charged. `core-search.md` holds the
  refuters with quantifiers, the `LL1` rows and the bounds.
- **42e (the two-sided engine)** reuses `Framed` as it is: the reading
  is per occurrence, so a span member's position is its occurrence's;
  `TwoSided`'s `admits` takes `LL` today and must take `LL1` with the
  intuitionistic rows; the two-sided `⊸L` split runs through `premises`,
  hence through `answers`. `Interactive`'s open witness is a metavariable
  of a session's own zone; `close` hands a goal to `search_goal`, which
  must then choose `Framed` by the goal's fragment as `Focused::decide`
  does.
- **The memo is off for a first-order goal.** A later step that keys
  ground or instance sequents needs canonical instance ids (the
  closing's numbering is one, per proof) and must keep proofs out of the
  memo or close them when kept: a kept proof's members name
  metavariables that a later undo unbinds.
- **`Ground` must stay the old code**: every first-order case sits
  inside a match's existing fallback arm behind `Z::FIRST_ORDER`, and a
  trait method `Ground` implements on a hot path is
  `#[inline(always)]` where its signature changed (`dual_in`); a guarded
  arm of its own cost `search-qbf-20-2` 0.4 % and an outlined `dual_in`
  `search-chain-128` 2.6 % (profiled with callgrind against the base,
  function by function: `bench.md`, 42a's note).
- **0.2.0's list** (step 45) gains no variant of `Term`, `Kind`, `Node`,
  `Rule` or `FoTerm`; the new variants are of non-exhaustive enums
  (`Reason`, `Refutation`), listed above.
- **The witness depth is per binding**: a proof may hold a deeper
  witness built by several bindings, as `Options::witness_depth` says;
  the panel measured the chain family needing exactly `n − 1`.

## Session 42e: the two-sided engine, interactive proving and the views

Opus 5.5 at `high`, unattended, 2026-10-30, in the jj workspace
`../linlog-step42e` on `wqqxusyk` (linlog's `main`), with two
sub-agents in workspaces of their own (`../linlog-step42e-classical`,
`../linlog-step42e-reference`) whose commits were rebased into the
session's chain and gated there.

### Outcome

The two-sided focused engine decides first-order sequents in the
intuitionistic modes, affine included: the dispatch's intuitionistic
rows and `TwoSided`'s `admits` take `LL1`, and the `Framed` zone runs
under the intuitionistic reading unchanged, since the reading is read
per occurrence and `⊸L`'s goal placement compares occurrences. Every
proof it returns passes the checker; a two-sided first-order reference
prover agrees with it (no disagreement in 6 000 generated decisions).
Interactive proving takes quantifiers: `∀R` and `∃L` open a formula
with a fresh eigenvariable, `∃R` and `∀L` with a witness given as a
term or left open as a metavariable (`$1`, `$2`, …) that a later axiom
binds by unification or `close` by the search; `undo` unbinds what its
step bound; the session's JSON keeps its instances and bindings and
reads back as itself; the command's `interact` takes a witness after
the rule's name. The classical two-sided view draws first-order
derivations with `∀L`, `∀R`, `∃L`, `∃R`; the intuitionistic two-sided
view and the LaTeX, Typst and SVG exports printed instances already
(42b), and print a session's open witnesses too. The propositional
case pays nothing: both target sets equal their oracles on every
decided row, the ratchet is within every ceiling with none raised.
The panel found faults in the interactive code
(none in the engine's verdicts or the checker): `close` on ground and
empty goals, unbounded terms and tables, exponential and quadratic walks,
forged documents accepted, a stop not polled under `--instances 1`, a
graft that did not read back; all held, all fixed and pinned, and a
fresh-context reviewer read the fixes. The pinned time waits for the
author's yes (below).

### What was built

- **The two-sided engine at first order** (`uplrqror`): `Focused::admits`
  takes `Fragment::LL1` one- and two-sided, the two intuitionistic rows
  of `search::DISPATCH` take `LL1` (their documented fragment `any,
  quantifiers too`); the exponential row's turn of the inverse method is
  refused by the inverse method's fragment, so a first-order goal takes
  none. Nothing in the engine changed: `Focused::decide` already chose
  `Framed` by the quantifier bit, the reading is per occurrence
  (`Reading::position`, `Reading::implication` take occurrences, and a
  span member's occurrence is its body's), and `open_split` places the
  goal of a `⊸L` by comparing occurrences. Test:
  `the_two_sided_engine_decides_first_order_sequents` (thirteen
  sequents, linear and affine, every proof checked).
- **The two-sided first-order reference** (`loltzmnn`, the sub-agent
  `linlog-step42e-reference`): `reference_fo::intuitionistic`, ILL1 by its
  own two-sided rules (the invertible ones applied first, each justified
  in a comment by the inversion lemma), unknowns bound by unification
  with the occurs check, the eigenvariable condition by freshness on the
  closed derivation, contraction of `!` hypotheses bounded per branch
  and "unprovable" only where the bound was never hit; a generator of
  random two-sided first-order sequents (trees for the reference, text
  for the engine, never read back into the reference);
  `decides_the_intuitionistic_textbook_cases` (23 cases) and
  `the_two_sided_engine_agrees` (400 decisions, dispatch and forced).
  Twelve deliberate faults put into a scratch copy, one at a time: eleven
  failed a test at once, the `!R` side condition after two cases were
  added for it, the twelfth (no undo after a failed axiom) has no
  observable effect. A scratch run of 3 000 sequents in two modes (6 000
  decisions, 1 408 sequents with a quantifier): no disagreement, every
  engine proof checked.
- **The classical two-sided view at first order** (`mywnluvl`, the
  sub-agent `linlog-step42e-classical`): `ClassicalRule::{ForallLeft,
  ForallRight, ExistsLeft, ExistsRight}` (`Rule::Forall` on the input
  side is `∃L`, on the output side `∀R`; `Rule::Exists` `∀L` or `∃R`),
  `exposed` giving a binder's body, the exposed member found under the
  principal's frame (a binder's body under the frame pushed), the
  record read through `Derivation::occurrence`, the counting pass on
  `Weights::of_members`, an instance written through the derivation's
  framing (`Notation::classical` takes members); the label tables of
  every target. Tests: `a_first_order_view_reads_as_its_term` (eight
  sequents, every rule and negation steps), `rules_are_labelled_in_declaration_order`,
  a pinned tree, and the snapshots `first_order_classical.{tex,typ,svg}`.
- **Interactive proving with quantifiers** (`knnknqqs`,
  `core/src/proofs/interactive/`): `quantified.rs`, a session's own
  instance table (members, frames, closed terms and eigenvariables
  hash-consed; metavariables as `FoTerm::Meta`), levels and the trail by
  step number, unification with the occurs check, the shown table (bound
  metavariables replaced) that the views and the proof read;
  `witness.rs`, the witness read from text (`Interactive::witness`) and
  built from `TermNode`s, held to the goal's scope; `expand.rs`, the four
  rules in `apply`, the axiom by unification, member premises
  (`framed_premises`, `mirror`), `undo` unbinding; `invert.rs`, `∀`
  invertible and the automatic axiom closing only equal instances;
  `terms.rs`, `Node::Forall` and `Node::Exists` in the session's proof,
  open witnesses closed by fresh constants; `replay.rs` and
  `serialize/interactive.rs`, the session's JSON with `instances`;
  `handover.rs`, `close` on a first-order goal. The command's `interact`
  takes `apply G P RULE [TERM]`, lists `∀L (with a witness or none)`,
  prints goals with their instances, and closes such a goal through
  `Interactive::close`. Tests (`core/tests/first_order.rs`):
  `a_session_proves_with_the_quantifier_rules`,
  `a_session_keeps_the_eigenvariable_condition`,
  `the_search_closes_a_session_goal`,
  `a_session_holds_deep_witnesses_and_bounds_its_table` (`olorpnkz`), and
  the refusals of `a_session_refuses_a_quantifier_on_every_path`.
- **The pinned rows** (`yutzwtzr`, `bench/pinned-time.py`): every focused
  and two-sided row runs about a second or more, as the plan's D17 now
  asks: the four search journeys repeated (`journey NAME --repeat N`,
  their measured parts summed: `search-qbf-20-2` ×36,
  `search-chain-128` ×30, `search-partition-no-5` ×2 400,
  `search-chain-64-intuitionistic` ×200, each about 1.0 to 1.2 s on
  core 2), `chain/512` (3.0 s, against `chain/256`'s 0.27 s),
  `qbf/34#1` (1.5 s, against `qbf/24#1`'s 0.11 s), `mix/9` forced onto
  the focused engine (5.9 s), `chain/384` and `chain/512` two-sided
  forced (1.15 s and 2.3 s, against `chain/256`'s 0.27 s). `growing` has
  no row (its size is a copy bound, and its search takes 0.14 s at 2 048,
  4 096 and 16 384 alike) and `mix/8` (0.69 s) none, `mix/9` covering the
  family. The sizes were chosen from one run each on core 2 while the
  learning website's session loaded the machine (load 25 to 28): CPU
  times good to some tens of percent, enough to choose a size.

### Decisions, and where they depart from the design

- **`close` writes a first-order goal as a sequent of its own**
  (`handover.rs`) (departs from the design: 42d's note had the session's
  open witness "a metavariable of a session's own zone" and `close`
  handing the goal to `search_goal` with `Framed`). The engine's zone
  starts from the sequent's roots and builds no term, while a session's
  goal holds instances whose frames hold terms the sequent's arena lacks
  (`f(a)` over an eigenvariable). So the goal becomes one formula: its
  formulas joined by `⅋` (the hypotheses first, so that the reading
  takes each as an implication's antecedent), under a prefix that binds
  the open witnesses by `∃` and the eigenvariables made after the first
  of them by `∀` in the order made, so that the search's eigenvariable
  condition is the session's levels; older eigenvariables and constants
  are constant symbols named as the session writes them. The search's
  proof opens that prefix and splits that spine first; its derivation is
  carried back with those steps dropped, every member mapped by its
  offset and frame, the prefix's eigenvariables the session's, every
  other eigenvariable fresh (a constant where no `∀` step introduces
  it), and the prefix's witnesses bound. Nothing in the engine changed.
  The outcome `close` returns is the search's on that sequent.
- **The library reads a witness's text** (`Interactive::witness`)
  (departs from the design: `api.md` 10.10 (i) left the text to the
  front end, "since `interactive` has no parser"): a closed term is
  names, brackets and commas, so a reader of a dozen lines needs no
  `parse` feature, and the command and the web page (42k) read alike,
  with the session's eigenvariable names. `Witness::Term` takes the flat
  `TermNode` list the design named.
- **`undo` truncates to its step's mark**, as 42a's note asked of the
  instance table: every step that makes entries records where the
  table, the metavariables, the eigenvariables and the constants stood,
  and its undo goes back there (the panel's fix: the first build left
  undone entries behind, which made the table unbounded under apply/undo
  cycles and kept `close`'s constants past its undo). The marks are no
  part of the JSON (departs from the design, which named no such key): a
  session read back undoes its earlier steps without truncating, their
  entries left unused; the steps after the read truncate.
- **A given witness is held to its goal's scope**: the eigenvariables of
  the `∀` steps on the path from the root, and the constants. An
  eigenvariable of a sibling branch is refused, though it would be sound
  as a constant there: the checker demands every occurrence of an
  eigenvariable inside its `∀` step's subtree. An open witness is held
  by levels, as in the engine.
- **A cut is refused in a first-order session** (departs from the plan's
  decision 7, "`Interactive::cut` types it as written", which no session
  built): the cut rebuilds the session's forest, and a first-order
  session's members past the forest would be numbered anew.
  `Error::Quantifiers` with "a cut in interactive proving".
- **An open witness is written `$1`, `$2`, …** (the plan named no form;
  the supervisor asked for one no reader takes as a formula): `?1` reads
  as the formula `?(1)`; no reader of either surface takes a `$`
  (pinned in `a_session_proves_with_the_quantifier_rules`), nor is it a
  name renamed apart (`x_1`). One function (`instances::meta_name`) names
  it for every printer; LaTeX escapes it, Typst writes it as a string.
- **`FoTerm::Meta(u32)`** is added now (the plan gave it to 42d, which
  needed none): a session's derivation shows open witnesses, through the
  same framing as a proof's. A proof's table refuses one
  (`instances::canonical`); a session's proof closes them first.
- **The command's goal lines** print a member's instance, and their
  separator follows the first formula printed on each side: a goal whose
  first hypothesis stood at position 1 printed `goal 2:, 1: !a` before,
  which README showed; it prints `goal 2: 1: !a` now. A pre-existing
  fault, fixed because every first-order goal with an instance after a
  bare occurrence met it.
- **`close` on a first-order session runs on one thread** in the command
  (no race of `searched`), as the first-order engine does.

### The closed enums' new variants (for step 45)

`FoTerm::Meta` (`FoTerm` is exhaustive: 0.2.0's break). New variants of
non-exhaustive enums: `StepError::{NoWitness, Witness, Eigenvariable}`,
`Needs::Witness`, `ClassicalRule::{ForallLeft, ForallRight, ExistsLeft,
ExistsRight}`. Additions: the types `Witness` and `TermNode`
(`proofs::interactive`), the field `Step::witness` (`Step` is
non-exhaustive), `Step::with_witness`, `Interactive::witness`.

### The propositional case (D17)

- **Target sets** against `bench/targets/42d.csv` and
  `bench/targets/net-42d.csv` (`linlog-bench summary --against …
  --counters`), three times: at `olorpnkz` (before the panel), at
  `vpxwqwsr` (the last commit that touches the search; files
  `42e.csv`, `net-42e.csv`) and at `trrlqvvm`, **the session's last code
  commit, which the D17 evidence covers** (files `42e-last.csv`,
  `net-42e-last.csv`). Each time no verdict differs, and of the 145
  focused and 64 net rows both decide alike, none differs in `nodes`,
  `splits`, `memo_hits`, `memo_entries`, `links` or `tests`.
- **The ratchet** (every gate, none raised; the last at `trrlqvvm`):
  within every ceiling; `bench/ceilings.csv` is unchanged over the
  session.
  `derivation-chain-64`, the journey with no margin, counted +1.98 % of
  its ceiling at the engine's commit (the base's +1.98 %), +1.93 % at the
  classical view's, +1.97 % at the interactive commit's; `render-svg`
  +0.89 %, `check-qbf-20-2` +0.61 %, `search-chain-128` −0.21 %, as at the
  base. The derivation view's and the exports' propositional paths gained
  only the classical view's member-aware writer, whose count moved
  `derivation-chain-64` by −0.05 %.

### The panel's findings and answers

On the frozen tree `olorpnkz` (the workspace `../linlog-step42e-panel`),
each member in a fresh context with its own scratch directory: the
argument on Fable 5.1 at `high` (cores 4 and 5), the counterexamples on
Opus 5.5 at `high` (6 to 9), the integers and limits on Sonnet 5.5 at
`high` (10 and 11). Every finding was reproduced before it was answered,
and every one below held; the fixes are the commits `vrlsyrnp` ("Fix what
the panel found in first-order sessions") and `vpxwqwsr` ("Fix what the
panel's counterexamples found"), each gated, every witness rerun against
them.

- **`close` on a goal without a binder or an open witness failed with
  `GoalMismatch`** (all three members; the counterexamples' fuzz met it
  about 2 800 times, the argument's 2 583 of 9 280 closes): the written
  sequent of such a goal has no binder, so its proof has no instance
  table, and `carry_back` required one. Fixed: an empty table stands in.
  Reproduced with `forall x. p(x) |- p(c)` after `∀L c`, and classically
  with `|- exists x. ~p(x), p(c)`; both now proved and checked.
- **`close` on the empty goal of a classical mode panicked** (argument,
  counterexamples): `|- (exists x. p(x)) * bot` after `⊗` and `⊥`.
  Fixed: the written sequent has no formula, and the search refutes it
  by the count equation, as the propositional `close` does.
- **`undo` of a `close` kept the constants it made** (argument,
  counterexamples) and, more broadly, **a session's table grew without
  bound and each step cost the whole table** (integers: 20 000 apply/undo
  cycles 47 s, quadratic). Fixed at the cause: every step records a mark
  and its undo truncates to it (above, departs from my first design);
  the cycles now take 0.13 s in a debug build, and
  `undo_drops_what_its_step_made` pins three closes undone reading as
  before.
- **Terms of shared halves unfolded without bound** (integers): a 1 KB
  session file whose frame term doubles 24 to 60 times made `goals`
  print gigabytes and `close` run out of memory past `--timeout`, and
  the same through a `TermNode` list. Fixed: every term of a session
  unfolds to at most `limits.terms` (`Quantified::limit`, counted by
  `show` over the shown table), checked at every step, at `close`'s graft
  and at reading; the files are now refused at once ("unfold to at least
  67108863 terms, more than the limit of 50000000; raise it with
  --term-limit"); `a_session_refuses_what_no_step_could_make` pins a
  witness of 2⁴⁰.
- **Unification and the instance comparison were exponential on shared
  terms** (integers: two 28-deep DAGs 3.6 s and doubling). Fixed: both
  skip a pair met before, and the comparison reads the shown table.
- **Reading was quadratic** in a chain of bindings (80 000: 11.9 s) and
  in `∃` steps (100 000: 38 s) (integers). Fixed: a binding's term may not
  be a bound metavariable at its top (no step makes one), and scopes are
  read by intervals of one walk of the tree; the chain is refused at
  once, the 100 000 steps load in 1.4 s.
- **A forged binding step and a forged metavariable level were accepted**
  (integers; the level also counterexamples): the first made `undo` give
  a state no session reaches, the second let an axiom bind an
  eigenvariable the condition forbids (the checker still refused the
  proof). Fixed: a binding must be made by a step that closes a goal (an
  axiom, `1`, `⊤`, or a graft), and an open witness's level may admit no
  eigenvariable a `∀` step made at its step or later; both forgeries are
  refused and pinned.
- **An exhausted eigenvariable counter made `close` report
  `GoalMismatch`** (integers, low). Fixed: a refusal (`index_limit`). Its
  note that `∀R` at the exhausted counter says "the witness is no term
  here" is left: a live session truncates its eigenvariables on undo, so
  only a document claiming 2³² of them reaches it.
- **`--instances 1` ignored the time and work limits** (counterexamples):
  each copy level was cut at once by the instances bound and reached no
  stable sequent, the only place the stop was polled, so the deepening
  ran past a million levels a second (in the one-sided engine since 42d;
  the two-sided path reaches it now). Fixed: each level of a first-order
  search is work and polls (behind `Z::FIRST_ORDER`, so `Ground` is
  unchanged); now "the time limit of 1s was reached at a copy bound of
  3084287", and `the_first_order_bounds_stop_the_search` pins the work
  bound in both modes.
- **A session `close` had grafted did not read back** (counterexamples, 3
  in its fuzz): `!0, (!0 -o ((forall x. r(x, x)) -o (p(b) * q))) |- …`.
  The graft unfolded the search's proof one-sided (`Derivation::new`),
  whose structural steps the reader's one-succedent check refused.
  Fixed: the graft is `Derivation::of_goal` of the written sequent's
  root, two-sided as the propositional `close` grafts; the file now reads
  back and its proof checks.
- **A text-started session failed its own `load`** (counterexamples,
  lesser): a parsed and a JSON-read first-order sequent are unequal by
  `==` (their argument lists numbered apart, 42b's follow-up), and
  `interact`'s `load` compared them so. Fixed there: `load` compares
  their JSON forms; the `==` itself stays a follow-up.
- **Observation, no finding** (argument): `close` binds an open witness
  its goal leaves unconstrained, so a witness shared with a sibling goal
  may be committed so that the sibling becomes unprovable; as designed,
  recoverable by `undo`.

What held, by the members' counts: the argument derived the two-sided
zone's invariants (one output per sequent consumed only by a leaf, the
goal of `⊸L` fixed on the consequent's side on the free and the forced
splits, the levels and the trail unchanged by the reading) and the
session's condition from the code, and ran 3 360 engine cases against
the reference (no contradiction) and 19 000 random session walks
(242 000 steps, 166 000 undos, 9 760 closes; every one of 38 239
finished proofs accepted). The counterexamples ran 823 200 engine runs
over 39 200 cases under many option sets and a second reference of its
own (`ref2.py`, ground witnesses) on 2 300 sequents × 2 modes × 13 flag
sets: no contradiction, 33 661 proofs re-checked by `linlog check`; and
30 771 fuzzed sessions (300 000 steps applied, 486 000 JSON round trips,
145 000 axioms judged by its own unifier, 24 900 witnesses by its own
scope; 11 000 finished proofs checked, 21 000 `close` goals against its
reference). The integers drove every counter and id of the session and
its JSON to its limit (4 700 mutated documents, no panic or overflow),
the eigenvariable and metavariable numbers at `u32::MAX`, deep terms of
10⁵ to 10⁶ levels through every walk (no stack overflow), and `close`
against the time limit and the interrupt (stopped on time).

### The fix review

The panel had read the tree before its fixes, so a fresh-context
reviewer (`linlog-step42e-fixreview`, Opus 5.5 at `high`, cores 4 to 11)
read the fix round, `vrlsyrnp` and `vpxwqwsr`, on a workspace at
`vpxwqwsr`, against the panel's findings and the eigenvariable condition
with open witnesses, and reran the counterexamples member's generators
and session fuzzer at its seeds and sizes on that tree, the supervisor's
request. **No correctness break.** It found each fix at its finding's
cause; truncation on undo safe (ids only grow, undo is last in first out
with the trail popped first, the maps keep exactly the ids below the
mark, a step failing midway goes back to its own mark, a session read
back has no marks for its old steps); the reader's new checks accepting
what sessions make (a single-rule graft rooted at `0L` binding a fresh
constant reads back and checks); the per-level poll changing no verdict
(the CLI driver's summaries byte for byte the member's, `--instances 1`
included). Its reruns: 30 771 fuzzed sessions at the member's seeds (and
the same again replacing the session by its read-back JSON at random,
74 266 reloads), about 1.4 million steps tried, 297 560 applied, 11 406
finished proofs checked in process and 9 000 through `linlog check`, all
valid, 487 049 round trips and 163 000 whole-JSON comparisons after
`undo` with no mismatch, 21 315 intuitionistic closes against the
member's second reference without a contradiction, the 2 764
`GoalMismatch` closes and 3 failed reads of the member's runs now none,
21 491 empty goals closed without a panic; the widened two-sided
comparison 42 160 cases, 885 360 runs, its tallies identical to the
member's.

One lesser finding, held and fixed (`trrlqvvm`, "Hold a session to its
term limit as its reader does"): `interact` started a session by
`Interactive::new`, ignoring `--term-limit`, and the reader counted the
table's entries against the limit where the live session counted only
its terms' unfolded sizes, so under `--term-limit 4` a session's own file
was refused. Now `interact` starts it by `Interactive::within` with the
command's limits, and the live session counts its entries beside its
terms (`Quantified::largest`), one measure for both; pinned in
`a_session_refuses_what_no_step_could_make`. Its witness (`|- exists x.
p(x), ~p(a), q(g(b))`, `--term-limit 4`) refuses the deep witness at
once, and a session within the limit reads back under it.

### The pinned time: pending

The rows of `bench/pinned-time.py` were changed first (above) so that
every focused and two-sided row runs about a second or more. The rule,
set before any run and sent to the supervisor with the request:
the propositional engine pays nothing when the focused rows summed take
at most 3 % more CPU time than the base's and no focused row's median is
more than 5 % above the base's, a row above 5 % looked at with the
journeys' instruction counts first (42c's and 42d's rule). The run waits
for the author's yes through the supervisor, which had not come when
everything else was done; nothing timed was run. Its command, from
`../linlog-step42e`, after building `linlog-bench` in release at
`wqqxusyk` (base) and at the session's last code commit `trrlqvvm` (lift)
into `target/pinned/base/` and `target/pinned/lift/`, under the machine
lock, the other sessions told to keep off cores 2 and 3:

```sh
systemd-run --user --unit=linlog-step42e-pinned --collect --same-dir \
  -p MemoryMax=8G -p MemorySwapMax=0 --setenv=PATH="$PATH" \
  flock /tmp/linlog-machine.lock python3 bench/pinned-time.py run \
  target/pinned/base/linlog-bench target/pinned/lift/linlog-bench \
  bench/targets/pinned-42e.runs.csv
python3 bench/pinned-time.py summary bench/targets/pinned-42e.runs.csv bench/targets/pinned-42e.csv
```

Cores 2 and 3, five runs per row, about 4 to 5 minutes. What the counts
say meanwhile: the ratchet's propositional journeys are where the base
had them at every gate, and the only search change of the session past
the dispatch (a poll per copy level) is behind `Z::FIRST_ORDER`, which
`Ground` compiles away.

### What is not verified

- No problem library is first-order and intuitionistic: the evidence for
  the two-sided engine is the generated sequents against the two-sided
  reference, the committed tests and the panel's runs.
- `close` on a first-order goal answers what the search answers on the
  written sequent; where a goal's open witness is shared with other
  goals, the binding it chooses may leave another goal unprovable, which
  `undo` takes back (as a later axiom's binding may).
- The web bindings (`web/`) do not yet offer the quantifier rules or a
  witness; that is 42k's.
- The message of `Error::Quantifiers` still ends "a sequent with ∀ or ∃ is
  read, printed and written as JSON, and nothing else", false since 42d
  (the search decides, the checker checks, sessions prove); left as it is
  (a follow-up: it is pinned by tests of the nets and the Rocq export).
- `cargo clippy -p linlog --all-targets` alone (without `--workspace`)
  warns of an import `core/src/search/focus/tests.rs` uses only with
  `parallel`, at `main` already (both sub-agents found it); the gate and
  the flake check run with the workspace's features and pass. A
  follow-up.

### Checklist

- [x] The two-sided instance of the first-order engine: the dispatch row
  `LL1 → TwoSided` in the intuitionistic modes, affine included; shown
  by the command on first-order intuitionistic sequents, every proof
  checked by `linlog check` (the session's messages).
- [x] `Interactive`'s four quantifier rules, a witness given and one left
  open, bound by a later axiom or by `close`; `undo` undoing bindings;
  the session's JSON round trip; the command's `interact` doing the same,
  every finished proof checked.
- [x] The two-sided derivation view, the classical two-sided view and the
  LaTeX, Typst and SVG exports printing first-order derivations with
  their instances (and a session's open witnesses).
- [x] Both target sets against `42d.csv` and `net-42d.csv`, every decided
  row's counters equal, at the session's last code commit; the ratchet
  within its ceilings at every gate, none raised.
- [x] `bench/pinned-time.py`'s rows of about a second or more; the rule set
  before any run and the request sent; [ ] the timed run itself: pending
  the author's yes, its command above.
- [x] The panel's round on a frozen tree (`olorpnkz`), each finding
  answered above; the fixes reviewed by a fresh-context reviewer
  (below).
- [x] README, the guide (its tests pass) and the changelog true.
- [x] Every code commit gated (the interactive commit with the lock's
  that follows it); `nix flake check` at the last commit: see the last
  message to the supervisor for its exit status.
- [x] `jj st` clean, the session's commits in `jj log`.
- [x] This section, and the next.
- [x] `.claude/rules/` amended (`core-derivations.md`, `core-focus.md`,
  `core-proofs.md`, `core-search.md`, `core-export.md`, `core.md`,
  `cli.md`, `bench.md`).
- [x] The last message to `linlog-orchestrator`.

### What the next sessions must know

- **`.claude/rules/core-derivations.md`, "First-order sessions"**, holds
  the session's invariants: its own hash-consed table, truncated by
  `undo` to its step's mark, the trail by step number, the term limit
  measured as the reader measures it, levels as the engine's and the scope for a
  given witness, the shown table that every reader reads, `$1` for an
  open witness, the axiom by unification and the automatic axiom on
  equal instances only, `close` by the written sequent, the JSON's checks,
  no cut. `core-search.md` holds the `LL1` rows and the two-sided
  reference; `core-derivations.md` the classical view's members.
- **42f (cut elimination)**: a session's proof is an ordinary proof with
  instances; a first-order session makes no cut, so 42f's proofs with
  cuts come from tests and files.
- **42g (Rocq)**: a session's proof may hold eigenvariables no `∀` step
  introduces (its open witnesses closed, and the search's own constants
  that `close` carried back): the checker's constants outside the
  signature, which the certificates must admit.
- **42i (the memo, resumed answers)**: the two-sided engine at first
  order is the one-sided engine's `Framed` under a reading; whatever 42i
  changes there it changes for both, and `the_two_sided_engine_agrees`
  joins `the_focused_engine_agrees` as its check.
- **42k (the web page)**: `Interactive::witness` reads a witness from text
  with the session's eigenvariable letters, `Needs::Witness` says where
  one may be given, `StepError::Eigenvariable` names the eigenvariable
  the condition refuses, `$1` is an open witness's name everywhere, and
  the session's JSON carries `instances`; `close` of a first-order goal
  runs on one thread, through `Interactive::close`. The bindings' `hint`
  (`web/src/calls.rs`) searches a goal by `Goal::new` on its members,
  which refuses a goal of instances (`Error::IndexOutOfBounds`): 42k
  gives it the session's written sequent, as `close` does.
- 0.2.0's list gains `FoTerm::Meta`; the rest are variants of
  non-exhaustive enums and additions (above).

## Session 42f: cut elimination at first order

Opus 5.5 at `high`, unattended, 2026-10-30, in the jj workspace
`../linlog-step42f` on `qvpqzppo` (linlog's `main`); the reviewer in a
workspace of its own, `../linlog-step42f-review`, at the frozen code
commit `wpuvlukz`.

### Outcome

Cut elimination on proof terms works on first-order proofs, in every
mode that has the cut (classical, with Mix, affine, intuitionistic,
intuitionistic affine), one step at a time and to the end, every step's
proof accepted by the unchanged checker. Every case of 41b's reads
members (instances) through the proof's instance table, and the key case
of `∀` against `∃` (`Case::ForallExists`) cuts the body's instance at the
witness and substitutes the witness for the eigenvariable throughout the
`∀` premise, in a copy where something else still reads it. A `∀` step
that a step remakes while the proof still reads the original gets an
eigenvariable of its own, so that every eigenvariable stays introduced by
one node, as the checker demands. Copies are charged against the bound on
nodes before they are made (the supervisor's requirement). `linlog
eliminate` takes first-order proofs, each step's line writing the
instance it cut; the nets' elimination still refuses them. The
propositional case is unchanged: both target sets equal their oracles on
every decided row, the ratchet within its ceilings with none raised.

### What was built

- **`core/src/proofs/cut/framed.rs`** (new): `Table`, the step's copy of
  the proof's instance table with lookups by occurrence and frame, by a
  frame's parent and term and by eigenvariable, and fresh eigenvariables
  past every one the table or a node names; `Rewrite::substitute`, the
  substitution of terms for eigenvariables in a subproof (frames,
  witnesses, a `∀` step's own eigenvariable when renamed), which copies
  only the nodes whose subproof names the map's domain and counts them
  first; `Rewrite::afford`, the charge of a copy against `cut.nodes` and
  `limits.memory_bytes`; `Rewrite::apart`, the renaming apart of twin
  `∀` steps after a step; `Rewrite::compact`, the made proof's table
  holding only what its nodes name.
- **`core/src/proofs/cut/mod.rs`**: every helper on members (`occ`,
  `beside`, `left`, `right`, `child`, `dual`, `body`), the zones
  `Record` keeps as members, `above` with the rules `∀` and `∃` (the
  affine erasure's specialization), the key case pair, `reduce`
  fallible (a copy's refusal), the quantifier refusal of `step` and
  `eliminate` gone. `Proof::formula` writes an instance with its frame's
  terms (it indexed past the forest before).
- **The command** (`cli/src/eliminate.rs`): each step's formula is
  written from the proof the step was taken on (`cut_formula`, in
  `eliminate`'s callback), since an instance's number means nothing in
  the proof reached.
- **Tests**: six proof files written by hand
  (`core/tests/first-order/cut_shared{,_ill}.json`, a `∀x.∀y` subproof
  that two cuts share after a commutation over `&`, whose inner `∀` step
  the substitution's copy renames; `cut_tensor{,_ill}.json`, a cut on the
  instance `p(a) ⊗ q(a)` of an eigenvariable introduced below the cut;
  `cut_deep{,_ill}.json`, the eigenvariable in the witness `f(g(b))` two
  rules up the `∀` premise), beside 42b's `cut`, `cut_ill` and
  `cut_instance`. `the_cuts_are_eliminated` takes every one of them step
  by step in its mode and every mode beside it (28 eliminations), each
  step checked, the last cut-free, and asserts the renaming happened;
  `the_written_cuts_are_eliminated` reads the files through the wire and
  eliminates them through the public API, every proof seen checked, and
  the proof structures refusing them; `first_order_cuts_are_eliminated`
  takes the search's proofs of 600 generated classical sequents and ten
  written intuitionistic ones, cuts each root against its identity
  expanded rule by rule, and eliminates 413 proofs (4 338 steps, 385 key
  cases of `∀` and `∃`, classical, Mix, affine, intuitionistic and
  intuitionistic affine), reducing every cut on the way first too;
  `copies_are_charged_against_the_bounds` (below).

### The bound on copies (departs from the design)

The plan's decision 7 says "no step adds a node, so the bounded
elimination stays bounded". At first order that is wrong: the key case
copies the `∀` premise where another cut still reads it, and the
renaming apart copies a `∀` step's premise, so a proof whose shared
subproof sits under several `∀`/`∃` cuts grows with every key case. How
large a copy can get: `substitute` copies at most the nodes its top
reaches, so a key case adds at most the `∀` premise's nodes and the
chain below the cut; a round of `apart` adds at most the renamed step's
premise and the chain below it, and the rounds are at most the twins the
step made (each round removes one and adds none). Why the bound holds:
every copy is counted before it is built (`substitute`'s first pass
reads only) and refused when the arena, the proof's nodes and the
step's, would pass `cut.nodes`, or `limits.memory_bytes` at 16 bytes a
node (`afford`); the proof made has at most the arena's nodes, so it is
within the bound, and the elimination ends there with the last checked
proof and `Refusal::Elimination { what: Nodes }`, as the propositional
bound ends it. The charge is on the arena, conservative: a step whose
copy replaces a subproof nothing else reads may be refused though the
proof it would make is smaller. The memory bound is charged too, but the
checker's pass needs far more than 16 bytes a node, so in practice it
refuses first. `copies_are_charged_against_the_bounds` pins it: a
`∀x.∀y.∀z` subproof that eight cuts share (a goal of three levels of
`&`) grows the proof by copies alone, and under a bound between its
nodes and the most the unbounded elimination reaches, the elimination
stops with a checked proof, cuts left, the bound's refusal, at a key
case of `∀` and `∃`.

### Decisions, and where they depart from the design

- **Renaming apart after the step, not inside each case** (`apart`):
  every case that remakes a `∀` step (the key case's copy, a commutation
  over a shared `∀`, the axiom case's rename path, the exponentials'
  generic arm, the affine erasure's specialization) can leave a twin
  while the original is still read; one pass after the step finds them
  by reachability and renames the later twin (or the earlier, where it
  lies above the later), so no case needs to know whether its original
  survives. A twin that nothing reads any more is dropped and keeps its
  eigenvariable, so the names a reader sees change only where a copy
  must differ from a surviving original.
- **The checker is the judge, unchanged**: every proof a step makes is
  checked (as in 41b), so a defect of the substitution would surface as
  `Error::Reduction`, never as a wrong proof.
- **`Proof::formula` writes instances** (an addition to a public
  method's behaviour: it indexed past the forest on an instance before,
  a panic), each term cut after 4 096 characters as an error's are.
- **The record's `formula`** stays a member of the proof the step was
  taken on, as `cut` and `above` are its nodes; a front end that draws a
  step has that proof from `eliminate`'s callback.
- **The propositional path is the old one**: a proof without a table
  takes `Member::occ` and the forest's navigation; no elimination journey
  is in the ratchet, and the search does not read `proofs::cut`.

### The closed enums' new variants (for step 45)

None of a closed enum: `Case` is non-exhaustive, and gains
`Case::ForallExists` (`forall_exists` on the wire, "the key case of ∀
and ∃"). No variant of `Node`, `Rule`, `Term`, `Kind` or `FoTerm` was
added. Changed by addition: `Proof::formula` writes instances.
`Error::Quantifiers` is no longer returned by cut elimination (its doc
names the one entry left, a cut in a first-order session).

### The propositional case (D17)

- **Target sets** at `wpuvlukz`, the session's code commit, against
  `bench/targets/42e-last.csv` and `bench/targets/net-42e-last.csv`
  (`linlog-bench summary --against … --counters`): no verdict differs;
  of the 145 focused problems both decide alike on one thread, 0 differ
  in `nodes`, `splits`, `memo_hits`, `memo_entries`, `links` or `tests`;
  of the 64 net problems, 0. The files are `bench/targets/42f.csv` and
  `bench/targets/net-42f.csv`; every later commit touches documents
  only.
- **The ratchet** (the gate at `wpuvlukz`): every journey within its
  ceiling, `bench/ceilings.csv` unchanged; `derivation-chain-64` +1.97 %
  (42e's +1.97 %), `check-qbf-20-2` +0.61 %, `search-chain-128` −0.21 %,
  as at the base.

### The reviewer's round

`linlog-step42f-review` (Opus 5.5 at `high`, fresh context, cores 4 to
11, its own workspace at `wpuvlukz` and scratch directory), briefed with
the claim (every step on a checked proof gives a checked proof of the
same conclusion or a bound's refusal, never `Error::Reduction` or a
panic) and four criteria: the substitution against the eigenvariable
condition (no capture, frames rebuilt right, a shared subproof copied),
every eigenvariable introduced by one node after every step, the other
cases on members in every mode, and the bounds (the supervisor's: every
copy charged before it is built), told to report only what breaks one
of them with a witness. **No finding.** What it ran: about 18 000
accepted proofs and 566 000 steps from search proofs of generated and
hand-picked sequents, cut against identities at the root and at random
inner nodes, an identity shared by two cuts, two cut formulas, and
proofs hash-consed into DAGs, in every mode each passes, random walks
at random cuts of up to 60 steps besides the strategy, every step's
proof checked with its sequent and conclusion (about 69 900 key cases
of `∀` and `∃`, 300 rounds of `apart`, 34 of them renaming more than
one eigenvariable; 0 failures); a hand-built proof reaching `apart`'s
branch that renames the earlier twin (a `∀` step read by an outer cut
and by a cut inside its other premise, `⊢ ?∀x.(p(x) ⅋ ~p(x))` with the
cut formula `⊥` in Mix mode), every cut at every stage to depth 8 and
the command; 6 787 propositional proofs with cuts through the base's
and this commit's `linlog eliminate --format json`, byte-identical
outputs and exit statuses; `--node-limit` 5 to 30 and none on every
committed file, each ending cut-free or refused with a checked proof.
Two notes, neither a finding, answered here: its generator could not
build cuts for some deep `∀x₁…∀xₖ` formulas (a defect of its scratch
table, so deep chains were covered only by the repository's tests and
the generated proofs where the cut was built); and the instance table's
growth inside a step is not charged to `memory_bytes`. **Answer**: the
table grows by at most its own size per substitution and the rounds are
bounded by the node budget, which the node charge already holds; no
change.

### What is not verified

- The memory bound's charge of copies is not reached by a test: the
  checker's pass refuses first at any memory limit low enough (above).
- No pinned-time run: no search changed (the brief's).
- The web bindings call the same `step` and `eliminate`, so the page's
  elimination now takes first-order proofs too; their drawing of a
  first-order step is 42k's to test.

### Checklist

- [x] The key case of `∀` and `∃` and the other cases on proofs with
  instances, the eliminations of committed first-order proofs with cuts
  (a cut on a first-order formula, one on an instance, a shared subproof
  under a `∀`/`∃` cut, an eigenvariable deep in a witness), every step
  checked, the last cut-free.
- [x] `linlog eliminate` on first-order proofs, classical, Mix, affine
  and intuitionistic, its steps writing instances, each intermediate
  proof (`--steps=k --format json`) accepted by `linlog check`; `--net`
  refusing first-order input (exit 2, "proof nets exist for MLL without
  units only, not for MLL1").
- [x] Copies charged before they are made, pinned by a test (the
  supervisor's requirement).
- [x] Both target sets against `42e-last` and `net-42e-last`, every
  decided row's counters equal; the ratchet within its ceilings, none
  raised.
- [x] The reviewer's round, each finding answered above.
- [x] README, the guide (its tests pass) and the changelog true.
- [x] Every code commit gated; `nix flake check` at the last commit: see
  the last message to the supervisor for its exit status.
- [x] `.claude/rules/core-proofs.md` and `cli.md` amended.
- [x] This section, and the next.
- [x] The last message to `linlog-orchestrator`.

### What the next sessions must know

- **`.claude/rules/core-proofs.md`, "Cut elimination"**, holds the
  first-order invariants: members everywhere, the substitution and why
  nothing is captured, the renaming apart after every step, copies
  charged before they are made, the table compacted, `Step::formula` a
  member of the proof the step was taken on.
- **42g (Rocq)**: the eliminated proofs are ordinary checked proofs with
  instances; an eigenvariable a substitution brings in from a witness is
  either one a `∀` step below the cut introduces or a constant no step
  introduces, which the certificates must admit as 42b's checker does.
- **42k (the web page)**: the bindings' `cut_step` and `eliminate` now
  take first-order proofs (nothing in `web/` changed); a step's
  `formula` is a member of the proof drawn before the step, whose
  derivation writes the instance; `Case::ForallExists` is
  `forall_exists` on the wire, and its step has no `above`.
- **A proof made by a step may name eigenvariables past the input's**:
  the renamed twins take fresh numbers, so the letters a derivation
  gives eigenvariables can shift between steps where a copy was renamed.
