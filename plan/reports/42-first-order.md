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
