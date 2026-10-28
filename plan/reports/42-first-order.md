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
