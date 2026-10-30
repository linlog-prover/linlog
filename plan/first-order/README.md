# Step 42: first-order linear logic, the plan

Written by step 42's first session (2026-10-28) from `plan/42-first-order.md`,
the design note `plan/notes/api.md` (3.2, 3.14, 10.10, decisions 1, 18
and 20, which the author answered on 2026-10-09), the research notes
`plan/notes/research/fo-linear.md` and `42-first-order.md`, the reports
named in the prompt, and the sources of the last section, each read for
this plan. It says which session builds what, on which model, with which
checks, and it settles the decisions every later session builds on. Where
it decides something `api.md` left to this step, it says so; where it
departs from `api.md`, it says why.

Words: a *term* is a first-order term (`x`, `c`, `f(t, …)`), a *formula
term* an entry of the sequent's arena (`Term`); an atom is *open* when an
argument holds a bound variable and *ground* otherwise; a *binder* is a
`∀` or `∃` formula term; an *eigenvariable* is the fresh constant a `∀`
inference introduces, a *metavariable* the unknown a search puts for an
`∃` witness; a *member* is a `u32` naming an occurrence below the forest's
length and an instance past it (`api.md` 3.4); a *frame* is the list of
terms the binders above an occurrence are bound to.

## The sessions

Eleven sessions (42i and 42j added by the author on 2026-10-30, at the
review of 42a to 42d's departures; the web page, once 42i, is 42k). The
first on Fable 5.1 at `high`; every other one on
Opus 5.5 at `high` (`plan/README.md`, "Why these models and efforts",
the roles table: a step session that writes code or proofs). Every
session that changes a search, a criterion or the checker is reviewed by
the panel of `plan/26-focused-engine.md` ("The oracle"): three agents in
fresh contexts, the counterexamples on Opus 5.5 at `high`, the argument
on Fable 5.1 at `high` (the author's re-evaluation of 2026-10-18 moved
it there from Opus at `xhigh`), the integers and limits on Sonnet 5.5
at `high`; a second reference prover written outside the repository by
a fresh-context agent for the panel's counterexample runs (42d writes
the first-order one). Every session: the propositional case unchanged
(D17, below), `gate` on every code commit, `nix flake check` at its last
commit, a report section in `plan/reports/42-first-order.md`, the rules
files amended, README, the guide for AI models and the changelog true,
and a fresh-context reviewer on Opus 5.5 at `high` where the session
says so.

**42a, the plan and the data model** (Fable 5.1 at `high`; this
session). This plan. Terms, predicates and binders in the arena and its
tables, the quantifier bit, both surfaces' text and JSON (wire level 7),
the printers of every target, the views printing a quantified sequent;
every engine, the checker, interactive proving, the nets, cut
elimination and the Rocq export refusing a quantified goal with a clear
error. Checks: both target sets against `bench/targets/cut-nets.csv`
and `bench/targets/net-cut-nets.csv` with every decided row's counters
equal; the ratchet within its ceilings. Reviewer: the data model and the
decisions against the requirements, D17 and `api.md`. Leaves: ground
first-order sequents decided by every engine as propositional ones;
quantified ones refused everywhere.

**42b, the proof term and the checker** (Opus 5.5 at `high`; panel).
`Node::Forall` and `Node::Exists`, the instance table a proof owns
(`Instances`: members, frames as cons cells, the proof's own terms and
eigenvariables), the checker's two passes (the frame pass, the
bottom-up pass with instances compared by id and the eigenvariable set
flowing to the root), its faults, the oracle's two rules (`proofs/oracle.rs`),
`proofs::size`, the JSON of a proof with witnesses (level 7's proof
form), the derivation view printing instances, the LaTeX, Typst and SVG
derivations, `Proof::check` on hand-written first-order proofs and the
cut rule on a first-order cut formula. No engine yet: the proofs come
from tests and from `Interactive`'s first-order rules (42e), so this
session proves by hand what it checks. Checks: the committed proofs
(provable and faulty ones, every fault reached), the oracle against the
checker on them, D17's oracles for the checker's propositional pass
(the `check-*` journeys of the ratchet). Panel: the checker's rules,
the eigenvariable condition's argument (below) read and attacked.

**42c, the zone lift** (Opus 5.5 at `high`; panel, counters). The
focused engine generic over `Z: Zone` with `Ground` the one instance
(`api.md` decision 20, `core-focus.md` "Decisions"), the seam the spike
found (a handle per dual list with its mark and its `Θ` flag,
`dual_from` with its cursors, the marks' bytes charged, `fork`, `root`,
the forest passed per call), the memo and the loop check behind
`Zone::memoizes`, `#[inline]` on the shared helpers. No first-order
code. Checks: the focused target set's counters identical, the ratchet,
and **pinned CPU time on an idle machine** (`bench/baseline.sh` on the
author's night, `night-jobs`): the lift stands when the pinned time of
the focused journeys is within a few percent, else the session keeps a
duplicated fast path instead (D17's other route, decision 20's "set
aside") and says so. Panel on the counters and the argument that the
generic engine is the same search.

**42d, the first-order focused engine, one-sided** (Opus 5.5 at `high`;
panel). The `Framed` zone (members with frames), `Bindings` and the
`Trail` with levels, unification with the occurs check on the
hash-consed terms, the literal seam (candidates by predicate symbol,
a dual a member that unifies, several candidates a choice point), the
prunes' first-order forms or their switch-off (the table in `api.md`
10.10 (f)), the enumeration of a premise's answers under a shared
metavariable, the dispatch row `LL1 → Focus` (classical modes, Mix
included), the refuters' carry-over (below), the proof returned closed
under the final substitution with every unbound metavariable replaced
by a fresh constant, `Options` fields for the first-order bounds
(`DEFAULT_*`, flags, JSON keys, each with a `Reason`), and the committed
first-order reference prover (`search/reference.rs`'s sibling for the
first-order calculus, with the eigenvariable condition by freshness),
checked by a fresh-context reviewer before it judges. Checks: the
focused target set's counters identical and the ratchet (the
propositional instance pays nothing); the reference against the engine
on generated first-order sequents and the first-order families' known
verdicts (42h's generators are written early, in this session, as far
as the comparison needs them); the panel with both references.

**42e, the two-sided engine, interactive proving and the views** (Opus
5.5 at `high`; panel). The two-sided instance (the dispatch row `LL1 →
TwoSided` in intuitionistic modes), `Interactive`'s four rules
(`∀R`/`∃L` with a fresh eigenvariable, `∃R`/`∀L` with a witness the
caller gives as a flat term or leaves open as a metavariable that a
later axiom or `close` binds; `undo` undoes bindings), the session's
JSON (members as `[occurrence, frame]`, `bindings`), the two-sided
derivation view and the classical two-sided view (`ClassicalRule`'s
four rules), the exports of two-sided first-order derivations. Checks:
the two-sided target rows, the reference prover two-sided, the session
round trip. Panel: the two-sided search.

**42f, cut elimination on terms at first order** (Opus 5.5 at `high`;
reviewer). The key case `∀` against `∃`: the witness substituted for
the eigenvariable throughout the `∀` premise (the instance table and
the frames of the subproof; a subproof shared under the cut copied
first), one more pair of `proofs::cut::step`'s key match, every step
checked by 42b's checker. The nets' cut elimination refuses first-order
input (the net engine refuses it, 42a). Checks: the eliminations of the
committed first-order proofs with cuts, each step checked, the
propositional eliminations' journeys unchanged.

**42g, the Rocq development** (Opus 5.5 at `high`; a reviewer of the
statements). A second development beside the propositional one in
`rocq/` (`plan/reports/31-rocq-library.md`, item 5): first-order
formulas over a signature, the one-sided calculus with the two
quantifier rules and the eigenvariable side condition, the two-sided
intuitionistic calculus with its four, linlog's first-order proof term
as a datatype, the checker as a function with the frame pass and the
eigenvariable set, and the theorem that a term it accepts proves its
sequent; the exporter's `--kernel linlog` certificates for first-order
proofs; `modules/rocq-linlog.nix` compiling them. Representation below.
Checks: the flake's `rocq-linlog` check with the certificates of 42b's
proofs; the soundness theorems closed under the global context.

**42h, the families and LinearOne** (Opus 5.5 at `high`). The
first-order generators below, their verdicts by construction, the
harness's columns for first-order rows (the unification and binding
counters the engine reports), `bench/RESULTS.md`'s table, and the
comparison with Moot's LinearOne: linlog's MILL1 sequents written in
LinearOne's Prolog syntax by the harness, LinearOne run under SWI-Prolog
from a pinned checkout (a flake package, as `lltp` is fetched), verdicts
compared and wall times recorded, in `bench/COMPARISON.md`. Checks: no
`MISMATCH` in `summary`, every LinearOne verdict equal to linlog's.

**42i, the first-order search reusing its work** (Opus 5.5 at `high`;
panel; the author, 2026-10-30, before 0.2.0). What 42d switched off or
simplified, lifted in the one- and the two-sided engines: the memo and
the loop check for first-order goals, keyed by canonical instance ids
(the closing's numbering is one), proofs kept out of the memo or closed
when kept (a kept proof's members name metavariables that a later undo
unbinds: 42d's report); and a premise's next answer found by resuming
its search where it stopped, not by searching again with the failed
answers excluded. Checks: D17, both references against the engine, the families' verdicts.
Panel: the memo's keys and what a hit may reuse; the resumption's
completeness (every answer reached, none skipped, none twice where that
costs). Measured and put to the author as "The keep-or-revert
measurement" below says.

**The keep-or-revert measurement** (42i and 42j; the author,
2026-10-30: "If the improvements are marginal but the code is
disproportionately more complicated, advise me on whether to revert to
the simpler version"). Each mechanism (the memo with the loop check,
the resumed answers, the pool) is committed apart from the others and
from its tests' and fixes' commits, so that reverting it is one `jj
backout` that leaves 42d's simpler form working. Each is measured
against 42d's form on the same build (an option or a forced setting
switching the mechanism off, or the base build beside it), on 42h's
families, on the generated sequents, and on sequents chosen where the
mechanism should pay (repeated subgoals under `&` and `!` for the memo,
many failing witnesses for the resumption, wide independent premises for
the pool) as well as where it should not; for each: verdicts gained
within the default time limit, work counters, pinned time on rows of a
second or more (the timed run asked of the author first, as every timed
run is), and peak memory. Beside the gains, the cost in code: lines
added outside tests, new invariants in the rules files, the panel's
findings against it, what it adds to every later change of the engine.
The report gives both in one table per mechanism and a recommendation,
keep or revert, with its reason; the planning session puts it to the
author, who decides; a revert is a commit of its own, gated, the
references and the panel's tests still passing.

**42j, parallel first-order search** (Opus 5.5 at `high`; panel; the
author, 2026-10-30, before 0.2.0). The decision on 42i's tables
(2026-10-30, after the pinned times): both mechanisms are kept, the
resumption on by default (`chain-fo` 26 to 29 % faster, every other row
within noise); the memo costs `chain-fo` 10 to 13 % (every run agreeing,
`bench/targets/pinned-42i-memo-*.csv`), so 42j first finds that cost
and removes it, measured by pinned time against the top before it (the
timed run asked of the author first), then builds the parallel search. A first-order goal on the pool as a
propositional one is (`--jobs`, the race, the cubes), dividing only work
that shares no unbound metavariable (decision 4) or by another division
the session argues for; 42d runs it on one thread whatever `--jobs`
says. The verdicts at one, two and four threads equal to one thread's,
`--deterministic` repeatable. Checks: D17, `cargo test --workspace` (the
CLI enables `parallel`), the families. Panel: the division's soundness
and completeness, the stops. Measured at one, two and four threads and
put to the author as "The keep-or-revert measurement" above says.

**42k, the web page** (Opus 5.5 at `high`; the last session, the
author's instruction of 2026-10-23). **Also in linlog, before the page** (the author, 2026-10-30): cut
in a first-order interactive session, which 42e refused (`Interactive::cut`
with a first-order formula typed as written, plan decision 7), the
members past the forest renumbered across the rebuilt forest and every
finished proof through the checker. **First, in linlog, before the page:** `undo`
after a session is read back from its JSON must truncate as it does in a
live session (found 2026-10-30 by a side agent's note, confirmed on the
command: with a save and a load between steps, as the page makes on every
click, the fresh eigenvariable after an undo drifts `a`, `b`, `c`, `d`
and the session's table grows with every apply/undo cycle toward its term
limit; 42e left the marks out of the JSON, a departure). Either the
session's JSON carries each step's marks (wire level 7 is not released
yet) or reading a session rebuilds them from its history; a test drives
apply, save, load, undo, apply through the JSON and the bindings and
finds the same letters and the same table as a live session. Input with quantifiers and terms,
the quantifier rules in interactive proving (the student gives the
witness at `∃R` and `∀L`, the page checks the eigenvariable condition
through the library), the drawing of a first-order derivation, the
bindings' calls (`web/`) with their tests under Node, the client's lock
moved after linlog's push. No lesson or practice (43d's). Checks:
`web-wasm-tests`, the client's Playwright gate.

What waits for a later step, said here so no session builds it: the
inverse method, the net engine, the planar search, `additive`, `horn`
and `essential` at first order (below); first-order ordinary logic
(`api.md` 10.10 (j)); sorts; LLO's finer dependency; numerals in
argument position.

## D17: the propositional case unchanged

Every session compares both target sets with the oracles the prompt
names, `bench/targets/cut-nets.csv` (`bench/targets.sh LABEL`, then
`linlog-bench summary --against bench/targets/cut-nets.csv
bench/targets/LABEL.csv --counters`) and `bench/targets/net-cut-nets.csv`
(`bench/net-targets.sh LABEL`), every decided row's counters equal, and
runs the ratchet within its ceilings, never raising one for a
propositional journey; 42c measures pinned CPU time besides. A session
that changes the search on purpose (42d, 42e) still leaves every
propositional row's counters equal, since the propositional instance is
the ground instance of the same engine and a quantifier-free goal never
enters the framed code. From 42e on the pinned rows of
`bench/pinned-time.py` are each about a second or more (the author,
2026-10-30): the clock counts in 10 ms ticks, so a row of 110 ms moves
9 % by one tick and the rule of 5 % a row judged only noise there
(42c's and 42d's sums stand, carried by `mix/9` and `chain/512`).

## The decisions

### 1. The term language and its place in the arena (42a)

`api.md` 3.2 and 3.14, built as designed, with the choices this
session adds marked.

- **`Term` and `Kind` gain `Forall(TermId)` and `Exists(TermId)`**, a
  binder and its body, appended after `Quest`; the literal variants stay
  the first two and contiguous, `Kind`'s discriminants are kept (12 and
  13 are the new ones), `Term` stays 12 bytes, `Kind` 1. `dual` swaps
  them, `arity` is 1, `Kind::polarity` is `Negative` for `Forall` and
  `Positive` for `Exists`, `Kind::fragment` is the new bit. The two are
  the only new variants of `Term` and `Kind` in this step (0.2.0's
  break, listed for step 45).
- **Bound variables are de Bruijn indices** (locally nameless): the
  innermost binder is `0`. A closed formula term is then α-canonical and
  `optimize` hash-conses `∀x.p(x)` and `∀y.p(y)` into one term with no
  renaming. The bound names are kept for printing only, in a side table
  from the binder's `TermId` to its name (`Sequent::binder_name`); when
  two binders merge the first name wins, and a printer writes a name
  that a symbol or an enclosing binder takes with `_` and a number after
  it (`x_1`; `api.md` 3.14 said "primed", but `′` is no identifier
  character, so the text would not read back).
- **First-order terms live in a second arena of the sequent**,
  `FoTerm`: `Bound(u32)` and `App(Symbol, Args)`, 12 bytes, topological
  and hash-consed; `Args` indexes a hash-consed table of argument lists
  (a flat vector with starts and lengths), `Args::EMPTY` the empty list
  of a constant. `FoTerm` gains `Eigen(u32)` in 42b and `Meta(u32)` in
  42d, each when its first use exists (no dead variant), in a proof's or
  a search's extension of the arena only: the sequent's arena holds
  bound and ground terms alone, and is read-only during a search.
- **A symbol is a name with one arity** (departs from the design: this
  session's choice, where `fo-linear.md` §1 proposed Prolog's `p/0` and
  `p/1` as two symbols and `api.md` 3.14 keyed a symbol by name, arity
  and kind):
  the symbol table holds `(name, arity)`, a name's first use fixes its
  arity, a use at another arity is refused by every reader, and a name
  used as a predicate and as a function symbol is refused (one
  namespace, `api.md` 3.14). Why: `p` and `p(a)` in one sequent is a
  mistake more often than a design, the Rocq signature and LinearOne's
  export want one arity per name, and the wire's `symbols` then needs
  no kind. A symbol's name is an atom name (an identifier in NFC that no
  keyword or reserved word takes), so a sequent written as text reads
  back as itself.
- **An atom is an interned atomic formula** (decision 1 of `api.md`):
  the atom table's key stays the canonical text (`p(f(a), b)`, an open
  atom `p(#0)` with its bound variables as indices), so that
  `atom_name` and every caller keyed by names stay right; beside it, a
  table parallel to the atoms holds each atom's symbol and argument list,
  empty exactly when every atom is nullary and there is no binder (then
  the sequent is propositional, its tables cost nothing and its JSON
  stays level 1 byte for byte). Ground first-order input is
  propositional by construction: `⊢ ~p(a), p(a)` is decided by every
  engine today, as a sequent with structured atom names, and its
  fragment is `MLL`.
- **The quantifier bit**: `Fragment::QUANTIFIERS` (32), set by a binder
  alone; the names `MLL1`, `MLL1 with units`, `ALL1`, `MALL1`, `MELL1`,
  `LL1` (and `IMLL1`, … in an intuitionistic mode), read back by
  `FromStr`; `has_nets` is false with the bit. Every `admits` refuses
  the bit by the fragment it decides (`LL` does not contain `LL1`), and
  the dispatch finds no row, so the front door answers `Error::NoEngine`
  ("no engine for MLL1 in classical mode yet") and a forced engine
  `Error::EngineRefused`; the checker, `Interactive`, the proof
  structures, cut elimination and the Rocq export refuse a forest with a
  binder by name (`Error::Quantifiers`, kind unsupported, naming the
  entry; the checker's own `CheckError::Quantifiers`, no verdict,
  departs from the design's `Fault::Instance` of `api.md` 10.10: a
  refusal is no fault of the term, and a caller must never report it as
  one). 42b to 42g lift each refusal as they build.
- **Bounds** (departs from the design, by deferral): `api.md` 3.14's
  `Limits::terms` and `Sequent::fo_size` are 42b's, with the instance
  table they bound; until then every reader of first-order terms counts
  the terms the atoms unfold to against `limits.occurrences` (the JSON's
  terms are shared, so a short document could name an atom of
  exponential length), and the text, linear in its terms, needs no more.
- **The text syntax** (both surfaces): `forall x. A`, `exists x. A`,
  `∀x. A`, `∃x. A`, several variables `forall x y. A` (nested binders),
  the dot required; a quantifier binds loosest, its scope running to
  the right as far as it can (to a `)`, a `,`, the turnstile or the
  end), so `forall x. p(x) -o q(x)` is `∀x.(p(x) ⊸ q(x))`; a negated
  quantifier is read as the dual binder over the negated body. A
  predicate is an identifier followed at once by `(`; inside an
  argument list a term is an identifier bound by an enclosing binder (a
  variable), else a constant, or an identifier applied to terms; an
  identifier alone is `p/0`, today's atom. No capitalisation convention
  (every present `A |- A` would otherwise read with free variables),
  closed input (an unbound identifier in term position is a constant;
  the alternative, Prolog's implicit closure, is set aside as `api.md`
  3.14 proposed), no numerals in argument position (refused, so the
  syntax can grow by that token later), shadowing allowed, `forall` and
  `exists` refused as names anywhere. The intuitionistic strict surface
  reads the same binders (`parse::classical` needs nothing); the
  one-sided lowering gives a binder's body its binder's side.
- **The printers**: `∀x. A` and `∃x. A` in text, `\forall x.\,A` in
  LaTeX, `forall x. A` in Typst, the text form in SVG; a bound variable
  by its binder's name, `p(t, …)` through the one atom writer with a
  term walk inside the atom's stop. The Rocq export answers
  `Unsupported::Quantifiers` until 42g.
- **The wire, level 7**: a sequent whose tables are not empty writes
  `"symbols": [["p", 1], ["a", 0]]` (names with arities), `"fo":
  [{"B": 0}, {"F": [1, []]}]` (a bound variable; an application, the
  symbol and its arguments as `fo` indices, each before its term), a
  structured atom in `"atoms"` as `{"P": [symbol, [fo ids]]}` (every
  atom, once one has arguments), a binder as `{"∀": body}` or
  `{"∃": body}`, and `"binders": ["x", …]`, the variables' names in the
  order of the binders (beside the terms, not inside them, so that a
  term stays three words and a propositional document reads as fast as
  before); the intuitionistic sequent's form the same keys and tags. A document that holds such a sequent is
  written at level 7 (`wire::TERMS`), a sequent without them at its
  level today; `wire::LEVEL` is 7, and a level-7 document pinned in
  `core/tests/wire/level-7.txt`. Level 7 is raised once more by 42b's
  proof forms (`{"∀": [m, a, p]}`, `{"∃": [m, w, p]}`, the instance
  table) and 42e's session form, each at level 7 still: a level is a
  release's, not a session's, and nothing of level 7 is released before
  the step ends.

### 2. The proof term, witnesses and eigenvariables (42b)

- `Node::Forall(Member, Eigen, NodeId)` and `Node::Exists(Member,
  Witness, NodeId)`, 16 bytes: the binder's member, the eigenvariable
  (a `u32` id, named by index, never bound in the term) or the witness
  (an index into the proof's term extension), the premise. The premise's
  body member is `(o + 1, F·a)` or `(o + 1, F·w)` where `(o, F)` is the
  conclusion's; a proof whose forest has no binder has no instance table
  and pays nothing.
- **The instance table** (`Instances`, `api.md` 3.14): members past
  `forest.len()` as `(OccId, FrameId)`, frames as hash-consed cons
  cells (a tower of `D` binders holds `D` cells, not `D²/2`), the
  proof's extension of the term arena (witnesses over the sequent's
  symbols and the proof's eigenvariables), the display names of
  eigenvariables (`a`, `b`, … primed past a clash with the sequent's
  symbols). `Proof::with_instances` owns it; the propositional
  constructors are unchanged.
- **Witnesses are recorded, never recovered**, and a proof is closed: an
  engine returns its proof under the final substitution, and a
  metavariable still unbound (a witness nothing constrained) is replaced
  by a fresh constant outside the signature, as Lincoln and Shankar
  instantiate; so no `Meta` ever stands in a proof, and the checker has
  no open witness to treat (`fo-linear.md` §7 Q3, decided here). An
  interactive witness left open is a metavariable of the session, bound
  by a later axiom or `close`, and closed the same way when the session's
  proof is taken.
- **The wire**: `{"∀": [m, a, p]}`, `{"∃": [m, w, p]}`, the proof's
  `"instances"` (members as `[occurrence, frame]`, frames as `[parent,
  term]`, the extension's terms as `fo` terms plus `{"E": k}`), level 7.

### 3. The checker's eigenvariable condition (42b, the panel)

The two passes of `fo-linear.md` §4.2, specified for the build:

- **The frame pass** (top-down, reverse node order): the root's members
  carry the empty frame; `Forall(m, a, p)` gives the body member of `p`
  the frame of `m` extended by `a`, `Exists(m, w, p)` by `w`; every other
  rule passes frames on unchanged; a node reached from two parents (a
  shared subproof) must receive equal frames, else a fault. Frames are
  interned per check in an overlay over the proof's table, so the
  bottom-up pass compares members by id.
- **The bottom-up pass** is today's with members for occurrences: `Ax`
  needs its two instances equal by id with eigenvariables as constants;
  `&` equal zones by instance; `⊗`, Mix, `!`, `⊤` and `any` unchanged;
  `Forall` consumes the body member and concludes the binder's;
  `Exists` the same with its witness.
- **The eigenvariable condition**: a proof is valid only if every
  eigenvariable `a` is introduced by exactly one `Forall` node `N`
  (counted by node, since a shared subproof unfolds `N` once per parent
  with one `a`), occurs in no root and in no cut formula (both closed by
  construction), and occurs only in witnesses of `Exists` nodes inside
  `N`'s subtree. Implemented as a set flowing from the leaves to the
  root with the state, persistent, never copied per node: `Exists` adds
  the eigenvariables of its witness, `Forall` removes its own, and the
  set at the root must be empty.
- **Why it is the condition** (the argument the panel reads, derived
  here from the rules of `fo-linear.md` §2.1): the condition of the
  calculus is that `a` occurs free nowhere in `N`'s conclusion. Every
  member of that conclusion, visible or absorbed by a `⊤` above, has a
  frame built by the nodes between the root and `N`, so `a` can enter it
  only through a witness chosen at an `Exists` outside `N`'s subtree;
  that witness's `a` flows up a path that never passes `N`, reaches the
  root, and the check fails. Conversely a derivation that meets the
  calculus's condition has every `a` in witnesses inside `N`'s subtree
  after renaming each `∀` step's eigenvariable apart, which the checker
  demands as its form (one `a` per node). The absorbed context of a `⊤`
  is covered because frames, not formulas, carry `a`, which answers the
  difficulty `plan/reports/31-rocq-library.md` item 5 saw for the
  one-sided checker. Two subtleties the panel attacks: a shared subproof
  under two `Forall` nodes with one eigenvariable (refused: one
  introduction per node), and an `Exists` below `N` whose witness names
  `a` (refused: `a` reaches the root).

### 4. Which engines gain quantifiers, which refuse them

- **The focused engine, one- and two-sided, gains them** (42d, 42e):
  backward search with a trail (`plan/reports/26-focused-engine.md`
  item 7 says where the marks go), unification with the occurs check at
  the identities, the eigenvariable condition kept by **levels** (a
  level per metavariable and per eigenvariable, the eigenvariable's one
  more than its branch's; binding a metavariable to a term with a
  younger eigenvariable fails; the lowering of older variables trailed),
  which `fo-linear.md` §3.2 derives from Nadathur, Jayaraman and Kwon's
  universe index and whose soundness and completeness are those of
  Lincoln and Shankar's LLV (`h(T)` over every metavariable of the
  branch; an older variable is exactly one LLV would have put under
  `h`). The memo and the loop check key ground stable sequents only; the
  `&` premises and the parallel cubes split only premises sharing no
  unbound metavariable; a failing right premise of `⊗` asks the left
  for another answer. (42d built the search with the memo and the loop
check off for first-order goals, another answer by searching again with
the failed ones excluded, and no pool; the author accepted these for
the moment on 2026-10-30 and asked for all three before 0.2.0: 42i and
42j.) MLL1 and MALL1 are decided by the bounded search
  alone (MLL1 is NP-complete and MALL1 NEXPTIME-complete, Lincoln and
  Shankar's theorems 3.5 and 3.4, the hardness Lincoln and Scedrov's);
  with exponentials the copy bound and the three-valued answer carry
  over.
- **The inverse method refuses them in this step.** It extends to first
  order by lifting: every sequent carries a substitution, unification at
  the initial sequents (each renamed apart), at every binary rule and
  in contraction, parameters substituted only by parameters, subsumption
  as instantiation filtered by Tammet's tests, and a forest of
  substitution trees as the index (Chaudhuri's thesis, §5.3 to §5.5,
  read for this plan). `plan/reports/37-inverse.md` says where the terms
  go (the zone's members, the tries' keys). That is a step of its own
  with its own panel; its row keeps `LL` and refuses `LL1` by fragment.
- **The net engine, `essential`, `additive`, `horn` and the planar
  search refuse them**: a first-order net needs one unifier for all its
  links and the eigenvariable condition as a box-like region or jumps
  (Girard's first-order nets; LinearOne's criterion), which the
  correctness criteria do not see; the additive path and the Horn
  program are propositional shapes; a first-order Horn engine is a row
  of its own (`plan/reports/27-horn.md`). Each refuses by its fragment
  as today.

### 5. The refuters (42d)

- **The unbalanced atom and the one-sided atom carry over per symbol**:
  a symbol all of whose atoms are ground keeps today's count per atom
  (sound: a ground literal pairs only with its equal); a symbol with an
  open atom is counted per symbol over all its atoms through the symbol
  index, sound and weaker (`∀x.p(x) ⊢ p(a)` counted per atom would be
  refuted). A per-symbol refutation names a symbol, not an atom, so it
  is a kind of its own (`unbalanced_symbol`, `one_sided_symbol`), at
  level 7; the propositional kinds and bytes are untouched.
- **The count equation carries over**: it counts connectives, a binder
  counted as its body (`plan/reports/31-rocq-library.md`, "Quantifiers").
- **The classical refuter refuses a quantified goal**: its reading is a
  finite assignment, and a first-order countermodel needs a domain; a
  new kind of a later step.
- **The saturated set refuses a quantified goal**: with quantifiers a
  saturation is a semi-decision (MELL1 is undecidable), and the inverse
  method does not run at first order in this step.
- Every refusal is by the fragment the refuter admits; `Disproof::check`
  takes the new kinds in 42d.

### 6. The ordered modes

The planar search stays the engine of cyclic MLL and the Lambek
calculus. Moot and Piazza's embedding of the Lambek calculus into
first-order MILL (a formula at a pair of string positions, `‖A/B‖^{x,y}
= ∀z.(‖B‖^{y,z} ⊸ ‖A‖^{x,z})`, `‖B\A‖^{y,z} = ∀x.(‖B‖^{x,y} ⊸
‖A‖^{x,z})`, `‖A•B‖^{x,z} = ∃y.(‖A‖^{x,y} ⊗ ‖B‖^{y,z})`, a sequent
`A₁, …, Aₙ ⊢ B` as `‖A₁‖^{0,1}, …, ‖Aₙ‖^{n−1,n} ⊢ ‖B‖^{0,n}`; the
clauses as Moot restates them in 2014, the 2001 paper itself not
fetched for this plan) was weighed against it: the embedding decides
the same sequents through the first-order engine, but the planar search
is built, measured and gives planar nets and derivations the page
draws, while the embedded proof is a first-order derivation whose
reading back as a Lambek derivation is work of its own. So the
embedding is not an engine: it is a **generator** (42h): the Lambek
family's sequents translated to MILL1, with the planar search's verdict
as the oracle, a differential test of the first-order engine on
problems with known answers and a bridge to LinearOne's own grammar
examples. The ordered modes refuse a quantified goal by `OrderedFragment`.

### 7. Cut at first order

Terms, yes; nets, no. The cut rule reaches first order in 42b (a cut
formula is a closed term of the arena, `push_cuts` dualises a binder
with the rest, the checker's arm takes members, `Interactive::cut` types
it as written) and cut elimination on terms in 42f (the `∀`/`∃` key
case by substitution of the witness for the eigenvariable, well defined
because an eigenvariable is introduced once and used inside its node's
subtree only; a shared subproof under the cut is copied before it is
substituted, which grows the proof (corrected 2026-10-30 at a heads-up
box in 42f's pane: the plan said no step adds a node), so each copy is
charged against `cut::Options`' nodes bound before it is built, and the
bounded elimination stays bounded by that charge). The nets' cut elimination refuses first-order input with the
net engine.

### 8. The Rocq development (42g)

A second development beside the propositional one, as
`plan/reports/31-rocq-library.md` item 5 and "Quantifiers" lay it out,
with the representation chosen here: terms over a signature as an
inductive with de Bruijn indices for bound variables (linlog's own
representation, so the exporter's translation is a transcription) and
eigenvariables as free constants named by naturals; the one-sided
calculus with the `∀` rule's side condition as freshness against the
whole conclusion (the absorbed context explicit, which the exporter
gives it from the checker's frames) and the two-sided intuitionistic
calculus with its four rules; the proof term with the witness and the
eigenvariable named; an eigenvariable no `∀` step introduces is a
constant outside the signature (42b's checker, the author's acceptance
of 2026-10-30: domains are not empty, so `∀x.p(x) ⊢ ∃y.p(y)` is
provable), which the calculus must admit too; the checker as a function
mirroring 42b's two passes; the theorem that an accepted term proves its sequent. Yalla and
NanoYalla are propositional (their formula types read for this plan),
so no bridge is possible and none is planned; Laurent's anti-locally-
nameless representation (CPP 2021, two kinds of variables, the `∀`
premise at the canonical eigenvariable `0`) is the alternative set
aside: it removes the freshness condition at the cost of a
representation the exporter would have to shift into.

### 9. The families and LinearOne (42h)

Three generators with verdicts by construction:

- `lambek-fo`: Moot and Piazza's translation of the Lambek family's
  sequents (decision 6), provable exactly when the planar search proves
  the original (the embedding is sound and complete for the Lambek
  calculus; this plan relies on Moot's restatement and 42h cites the
  theorem from the 2001 paper before the family is committed).
- `chain-fo`: `p(0), !∀x.(p(x) ⊸ p(s(x))) ⊢ p(sⁿ(0))`, provable with
  `n` copies, and an unprovable variant: MELL1 under the copy bound, the
  first-order Horn shape. (Corrected 2026-10-30 at a heads-up box in
  42h's pane: the variant `⊢ p(sⁿ⁺¹(0))` this plan named is provable with
  `n + 1` copies; 42h chooses one unprovable by a stated invariant, its
  expected verdict what linlog can answer, unprovable by a refuter or
  unknown at the copy bound.)
- `witness-fo`: `p(a₁) & … & p(aₙ), q(aₖ) ⊢ ∃x.(p(x) ⊗ q(x))` and its
  unprovable variant with `q(b)`: MALL1, the premise that must be asked
  for another answer (`api.md` 10.10 (e)), at sizes that measure the
  enumeration.

LinearOne (Moot, SWI-Prolog, MILL1 by proof nets with unification and
an eigenvariable check on the `∀` link; input `at(p, [X, Y])`,
`impl(A, B)`, `p(A, B)` for the tensor, `forall(X, F)`, `exists(X, F)`;
no units, additives or exponentials) is the prover to compare with on
`lambek-fo` and on the MILL1 part of `witness-fo` (`⊸`, `⊗`, `∃` only):
the harness writes its syntax, runs it from a pinned checkout and
records verdicts and wall time; llprover is not compared (its
contraction cap of three).

## Sources

Read for this plan on 2026-10-28, by this session or by its research
sub-agents (Sonnet 5.5), whose quotes this session relied on:

- `plan/42-first-order.md`; `plan/later.md` ("First-order linear
  logic"); `proof-search-specifications.md` ("First-order fragments" and
  its erratum on MALL1); `plan/notes/api.md` (3.2, 3.3, 3.8, 3.14,
  10.10, 11, 14.2); `plan/notes/research/fo-linear.md` and
  `42-first-order.md`; `plan/reports/17-assessment.md` (3.9, answer 4);
  the reports of steps 26 (item 7), 28, 31 (item 5, "Quantifiers"), 37
  ("Where quantifiers go"), 40 and 41 (each "Options, front ends and
  quantifiers"); `plan/README.md` (D1, D5, D6, D17, "Why these models
  and efforts"); `plan/26-focused-engine.md` ("The oracle");
  `.claude/rules/core.md`, `core-sequents.md`, `core-forest.md`,
  `core-focus.md`, `core-proofs.md`, `core-search.md`, `rocq.md`.
- Kaustuv Chaudhuri, *The Focused Inverse Method for Linear Logic*, PhD
  thesis, CMU 2006 (chaudhuri.info/papers/chaudhuri06thesis.pdf), §2.1.4
  (the rules), §5.3 to §5.5 (lifting, unification, subsumption, the
  index); Chaudhuri and Pfenning, *A Focusing Inverse Method Theorem
  Prover for First-Order Linear Logic*, CADE 2005.
- Patrick Lincoln and Natarajan Shankar, *Proof search in first-order
  linear logic and other cut-free sequent calculi*, LICS 1994
  (csl.sri.com/papers/lics94/lics94.pdf): Theorem 3.4 (MALL1
  NEXPTIME-complete, the hardness by Lincoln and Scedrov, TCS 135, 1994,
  not fetched), Theorem 3.5 (MLL1 NP-complete), the calculi LLG, LLV and
  LLO, §2 to §4.
- Richard Moot, *Extended Lambek calculi and first-order linear logic*,
  2014 (arXiv:1405.6678, §2.1) and *Hybrid type-logical grammars,
  first-order linear logic and the descriptive inadequacy of lambda
  grammars*, 2013 (arXiv:1305.6238, §4 and §5), for the translation of
  Moot and Piazza, *Linguistic Applications of First Order
  Intuitionistic Linear Logic*, JoLLI 10, 2001 (not fetched: Springer
  and the archive refused the request).
- Richard Moot, LinearOne (github.com/RichardMoot/LinearOne): `README`,
  `mill1.pl`, `translations.pl`, `lambek_grammar.pl`, `ll1_grammar.pl`.
- Olivier Laurent, Yalla (github.com/olaure01/yalla), its formula types
  and `microyalla/`; *An Anti-Locally-Nameless Approach to Formalizing
  Quantifiers*, CPP 2021 (hal-03096253), the abstract and the `∀` rule.
- Through `fo-linear.md`'s citations, not fetched again: Nadathur,
  Jayaraman and Kwon 1995 (the universe index); Hughes 2018 (unification
  nets); de Bruijn 1972; Charguéraud 2012 (locally nameless).
