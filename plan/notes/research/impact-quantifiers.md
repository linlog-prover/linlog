# What first-order quantifiers would change in the current code

Written 2026-10-08 for the design of step 28, stage 2 (`plan/notes/api.md`),
from the snapshot of the repository; every line number is the snapshot's,
and every place below was read in the code, not only in the rules files.
The logic, the techniques and the literature are in `38-first-order.md`
(cited "38 §n"), and the reports of steps 18, 20, 22, 24, 25, 26 (item 7)
and 27 already say where quantifiers go in their own parts; this note
cites them rather than repeating them. Paths are relative to the
repository root.

Two labels:

- **open now**: cheap at step 28 and costly after it, because it fixes a
  public signature or field, a wire form, or a contract that steps 29 to 37
  build on before step 38 arrives;
- **at 38**: local and cheap at the feature's step. **At 38, not local**
  marks a cost that no decision now lowers but that the spike of stage 2,
  item 4, should measure.

"(inference)" marks a conclusion of this note's own.

## 1. What the earlier notes do not say

1. **Ground predicates break the propositional engines even when there is
   no binder.** If `Atom` names a predicate symbol, as 38 §3 has it
   (`Pred(Atom, Args)`), then `p(a)` and `~p(b)` are two literals of one atom
   with opposite signs. Every place that pairs literals by atom would accept
   them as an axiom:
   - the checker (`core/src/proofs/check.rs:1167`, and `oracle.rs:271`);
   - `ProofStructure::dual` (`core/src/nets/mod.rs:280-283`);
   - the additive path (`core/src/search/additive.rs:191`);
   - the net engine's partner lists (`core/src/search/net.rs:437`);
   - the focused engine's `initial`, `dual_in` and `dual_from`
     (`core/src/search/focus/mod.rs:1588-1647`, `focus/split.rs:19-40`);
   - the Horn engine's places (`core/src/search/horn/mod.rs:405-412`).

   A first-order sequent without quantifiers sets no quantifier flag. So the
   fragment bit that step 26 proposes (report, item 7) must be set by a
   predicate argument as well as by a binder, or else ground atomic formulas
   must be interned as atoms of their own (inference). The checker is what
   every proof is trusted on. Step 38's prompt has the data model come "first
   and by itself", so that first commit must make `Proof::check` refuse a
   sequent with arguments until `Ax` compares instances.

2. **The leaves of a proof term do not fix their instances.** The checker
   works out each node's sequent from its premises alone, bottom-up
   (`check.rs:1-20`, `Pass::rule` at 1156), and a memo hit shares one
   subproof among several conclusions (`core-proofs.md`). Under a binder,
   `Ax(x, y)` names two occurrences, not two instances. The instance comes
   from the `∀` and `∃` nodes below the leaf, which the pass has not reached
   yet, and a shared subproof can sit under different instantiations.
   - Without exponentials, a schematic leaf with deferred equations would
     work (inference).
   - With `?` copies it does not. Two copies of one occurrence with different
     eigenvariables are a single `OccId` in a `Bag` (`check.rs:118-131`).

   So `Ax` and `Copy` must name *members* (an occurrence together with its
   instance), or the proof must carry a table that does. `Node` stays at 16
   bytes only if a member is one `u32`: an id into a table that the `Proof`
   owns (inference).

3. **One answer per goal is not enough across a `⊗`.** Step 26's item 7 lets
   bindings cross the premises of a `⊗` and undoes them "only when the split
   fails". But the engine returns one proof per goal (`Found::Proved`,
   `focus/mod.rs:458-510`; the memo's `Entry::Proved`, `focus/memo.rs:85-90`)
   and never asks a proved premise for another one.

   Example: `p(a) & p(b), q(b) ⊢ ∃x.(p(x) ⊗ q(x))` is provable with `x = b`.
   The only split that passes the counts gives the left premise
   `p(a) & p(b) ⊢ p(X)`. Its first proof binds `X = a`, the right premise
   `q(b) ⊢ q(a)` then fails, and the split is lost.

   Completeness needs one of two things (inference):
   - the left premise's answers enumerated, by success continuations or by
     answer lists in the memo;
   - witnesses chosen at the `∃` from a finite set of candidates. This keeps
     the one-answer control, but the set is finite only without function
     symbols.

   38 §2 states the problem for the `&` premises; the `⊗` and Mix premises
   have it too.

4. **The text syntax has room, but not for every spelling.** These are all
   errors today (`core/src/parse/mod.rs:304-309, 330-373`):
   - a `(` after an identifier;
   - `[`, `]` and `:`;
   - a `,` inside brackets.

   `core/tests/parse.rs:163, 170` pins `(A, B)` and `∀` as errors. Predicates
   and quantifiers can therefore come in without changing any text that
   parses now. Some spellings still collide:
   - `forall` and `exists` are atom names today (`|- forall` parses), so they
     can be keywords only in context;
   - a `.` after the bound variable collides with the LLTP and TPTP readers,
     which end a clause at `.` (`core/src/lltp.rs:38-43`, `clauses` from 125);
   - TPTP's own `![X]:` and `?[X]:` collide in spelling with linlog's `!` and
     `?`, but `![` and `?[` are errors today, so they are free (inference).

5. **Equality up to renaming (α-equivalence) versus the names the user
   wrote.** `optimize` merges equal `Term` values (`sequents/mod.rs:254-268`).
   - If a binder's display name lives in the `Term` (`Forall(name, body)`),
     then `∀x.p(x)` and `∀y.p(y)` stay two terms and two classes of
     interchangeable occurrences (`focus/classes.rs:31-49`).
   - If it does not, a merge prints one of the two formulas with the other's
     name.

   Either choice is consistent. It belongs in `api.md` together with the
   hash-consing argument of 38 §2.

6. **"Nothing recurses" and the size bound apply to terms as well.**
   - Unification, the occurs check, substitution, comparison of instances and
     the printing of terms must all be iterative (`core.md`, "Nothing recurses
     over a formula").
   - A DAG of argument terms read from JSON unfolds exponentially, as a DAG of
     formulas does (`sequents/mod.rs:145-173`). `Sequent::occurrences` counts
     formula occurrences only, so printing and the size estimate
     (`proofs/size.rs:193-229`) need a bound on unfolded terms of their own.
   - The checker must compare the instances of witnesses by their hash-consed
     id and never unfold them. Otherwise a hostile proof file passes
     `check_within`'s memory bound by time instead.

## 2. The places

Each entry gives the place, what it assumes, what the feature needs there,
and the recommendation.

### 2.1 The data model: types, layout, sizes, indices

- `core/src/sequents/term.rs:27-47` **`Atom`**, documented as "a propositional
  variable": an index into the names. *Needs*: a predicate symbol with its
  arity, or an atomic formula (finding 1). **Open now**: write the contract
  down (item 7); the rest at 38.
- `term.rs:52-77` **`Term`**: twelve variants, at most two `u32`, so 12 bytes,
  with no size assertion. *Needs*: `Pred`/`DualPred` (an atom and an id into
  an argument table) and `Forall`/`Exists` (a body, perhaps a name), still 12
  bytes (38 §3). **Open now**: add
  `const _: () = assert!(size_of::<Term>() == 12);` beside `Node`'s
  (`proofs/mod.rs:158`), the guard D17 asks for (item 10). The variants come
  at 38.
- `term.rs:80-107` **`Kind`** (`repr(u8)`, arity 0, 1 or 2 at 130-137,
  `is_literal` at 140). *Needs*: `Forall` and `Exists` of arity 1. Keeping
  predicate literals as `Var`/`DualVar` kinds leaves every place that reads a
  literal by its kind as it is. **At 38** (decided in `api.md`).
- `term.rs:202-209` **`operands`/`subterms`**: formula children only, which
  the forest numbers and `finish` dualises (`parse/mod.rs:402`). *Needs*:
  arguments kept out of them, so that they are not occurrences. **At 38**,
  with the contract in `api.md`.
- `core/src/sequents/mod.rs:62-71` **`Sequent { terms, roots, atoms }`**
  (`pub(crate)`). *Needs*:
  - a symbol table (predicates and functions with their arities);
  - an argument arena, kept apart from `terms`, so that no propositional
    `TermId` moves;
  - name hints for bound variables.

  **At 38** (the fields are private; the accessors are additive).
- `sequents/mod.rs:149-173` **`sizes`, `occurrences`**: they count formula
  occurrences, and arguments are not occurrences. *Needs*: a bound on unfolded
  terms (finding 6). **At 38.**
- `sequents/mod.rs:176-201` **`verify_integrity`**: the bounds of atoms and
  subterms. *Needs*:
  - symbol bounds and arities;
  - the order of the argument arena;
  - the scope of every de Bruijn index (bound variables written as a count of
    enclosing binders). This takes one ascending pass that keeps, per term,
    the largest index left unbound (inference).

  **At 38.**
- `sequents/mod.rs:281-309, 326-356` **`optimize_atoms`, `merge_atoms`** ("a
  name is an atom"). `let (Var(a) | DualVar(a)) = *e else { continue }` (289)
  and `other => other` (344) skip a new literal variant silently. **Open now**
  for the wildcards (item 6); merging symbols by name and arity at 38.
- `sequents/mod.rs:361-379` **`Sequent::add`**, through `Term::offset`
  (`term.rs:227-234`, `other => other.map_subterms(…)`), shifts the atom of
  `Var`/`DualVar` only. *Needs*: the symbols and the argument arena shifted
  too. **Open now** for the wildcard; the rest at 38.
- `core/src/occurrences/mod.rs:18-39, 195` **`OccId(u32)`, `NONE`,
  `Forest::MOST`**: numbered in preorder. Binders are unary occurrences and
  arguments are none, so a propositional sequent keeps its ids. **None.**
- `occurrences/mod.rs:43-62` **`Sign { Var, DualVar }`**, named after the
  literal kinds. **Open now** (the names, item 9).
- `occurrences/mod.rs:154-180, 374-378, 443-465` **`Forest`**, with its
  literal lists in CSR layout by atom and sign. With predicates a group holds
  literals with different arguments: candidates for unification, not
  partners. *Needs*: the doc to say "of one predicate symbol"; an array of
  binder depths only when there are quantifiers (per occurrence a `u32` at
  most, as `core-forest.md` asks). **Open now** for the contract (item 7); the
  rest at 38.
- `occurrences/mod.rs:279-281` **`Forest::formula(o)`** prints
  `sequent.formula(term(o))`, that is, the term alone. With bound variables
  written as indices, an occurrence below a binder prints with a dangling
  index. *Needs*: the names of the binders above `o`, from its parents, or
  the member's instance. **Open now** for the contract (item 5).
- `core/src/occurrences/set.rs:24-27` **`OccSet`**, public: a sequent as a
  set of ids. Under `?` copies with instances, a member is not an id.
  **Open now**, through items 1 and 3.
- `core/src/occurrences/reading.rs:197-279` **`Reading::new`**, an
  exhaustive match at 206-213: a position per occurrence. A quantifier keeps
  its position and can stand on either side. **At 38.**
- `core/src/fragment.rs:13-42, 95-125, 160-174, 197-215` **`Fragment`**:
  - a `u8` with five bits used;
  - `LL` is documented as "every connective" (41);
  - `name` and `name_in` map every value to one of six names;
  - `Kind::fragment` gives literals the empty fragment;
  - `Sequent::fragment` reads kinds only.

  *Needs*: a bit set by arguments and by binders, read from the terms and
  not from the kinds, and names for it (`MLL1`, …). **Open now**: document
  that the named constants are propositional (item 7). The bit comes at 38.
- `fragment.rs:220-229` **`Mode`**: three public fields, which 38 leaves as
  they are (38 §3). **None.**
- `core/src/errors/mod.rs:19-210` **`Error`**: no `#[non_exhaustive]`, and
  `InvalidVariableIndex` (21-24) names an atom index. *Needs* variants for:
  - a symbol index out of range;
  - an arity mismatch;
  - a bound index out of scope in JSON;
  - the term bound.

  **Open now** (items 9 and 10).
- The proof term and the engines' types are in 2.4 and 2.5.

### 2.2 Exhaustive matches and visitors over formula, node and rule kinds

**Exhaustive matches.** The compiler lists these when a variant is added,
so they are mechanical work **at 38**:

- over `Term`:
  - `term.rs` `kind` (147-163), `dual` (176-192), `operands` (202-209);
  - `sequents/fmt.rs` `fmt_term` (145-156);
  - the dualisation in `parse/mod.rs` `finish` (395-414);
  - `serialize/sequents.rs:61-104`;
  - `proofs/size.rs` `weights` (205-216);
  - `export/notation.rs` `term` (64-78);
  - `export/rocq.rs` `term` (263-279);
- over `Kind`:
  - `fragment.rs` `Kind::fragment`;
  - `occurrences/mod.rs` `polarity` (102-109);
  - `reading.rs` `Reading::new` (206-213) and `fmt_formula` (361-370);
  - `notation.rs` `ill` (106-116);
  - `focus/counts.rs` `Counts::new` (278-321);
  - `proofs/interactive.rs` `rules` (501-513) and `expand` (606-620);
- over `Node`:
  - `proofs/mod.rs` `principal`, `premises`, `name`, `map_premises`
    (163-230, 463-478);
  - `check.rs` `rule` (1160-1304) and `oracle.rs`;
  - `derivation.rs` `Build::unfold` (1211-1285);
  - `size.rs` `Measure`;
  - `serialize/proofs.rs:68-120`;
- over `Rule`:
  - `derivation.rs` `name`, `from_str`;
  - `rocq.rs` `inference` (386-469).

**Wildcard arms that a new variant falls into without a compiler error.**
**Open now** (item 6). For each arm, what a `∀`, an `∃` or a `Pred` does
there:

| place | arm | what goes wrong |
|---|---|---|
| `term.rs:169` `Term::atom` | `_ => None` | a `Pred` literal is not a literal; `Forest::build`'s `unwrap` (`occurrences/mod.rs:449`) panics |
| `term.rs:222` `map_subterms` | `leaf => leaf` | a binder's body keeps its old index in `optimize_terms` (`sequents/mod.rs:261`): a corrupt arena |
| `term.rs:232` `offset` | `other => …` | `Sequent::add` leaves a `Pred`'s atom unshifted |
| `sequents/mod.rs:289, 344` | `let … else`, `other => other` | the atoms of a `Pred` are not merged: "a name is an atom" breaks |
| `occurrences/mod.rs:96` `Kind::sign` | `_ => None` | a new literal kind has no sign |
| `focus/mod.rs:865` `decompose` | `_ =>` "positive or literal" | a `∀` goes into the stable sequent undecomposed |
| `focus/mod.rs:878-881` | `_ => Node::Bot(o, node)` | a binder applied in the loop is recorded as a `⊥` node |
| `focus/mod.rs:1708` `focus_class` | `_ => 2` | an `∃` is ranked with `⊕` |
| `focus/mod.rs:1769` `focus_on` | `_ =>` release | an `∃` in focus is released as negative, with no witness |
| `derivation.rs:230-250, 253-278` `classical`, `intuitionistic` | `rule => rule`, `(rule, _) => rule` | `∀L` maps to nothing, and a `∀` on a hypothesis keeps its one-sided name |
| `interactive.rs:1054, 1095` `term` | `_ => Node::Top`, `_ => Node::Mix` | an unhandled rule becomes a `⊤` or Mix node |
| `nets/graph.rs:147` `Graph::new` | `_ => to.push(NONE)` | a quantifier node is taken for a leaf |
| `export/svg/net.rs:291` | `_ => ("⊗", AXIS)` | a quantifier link is drawn as `⊗` |
| `sequents/fmt.rs:162`, `notation.rs:85, 123`, `reading.rs:375` (between two operands) | `_ => " ⊕ "` | harmless for unary binders; a new binary connective (step 36) prints as `⊕` |

- Loud and acceptable: the `unreachable!` arms at `focus/mod.rs:1309`,
  `interactive.rs:619, 727, 1072, 1098` and `nets/graph.rs:463`.
- Refusals by default, which are right: the shape readers at
  `horn/mod.rs:400, 436-438`, `focus/schedule.rs:249-250`, `split.rs:56-62`
  and `size.rs:413`.

**The visitor.** `core/src/sequents/fmt.rs:16-125`: `Walk` and `Visit` are
`pub(crate)`; `operands` gives at most two children; the stops are `Enter`,
`Between` and `Exit`. A binder is the `Enter`/`Exit` of a unary node. A
printer must push the binder's name at `Enter`, pop it at `Exit`, and run a
term walk inside a predicate's `Enter`. Printing from an occurrence inside a
formula needs the names of the binders above it (finding 6, item 5). The
printers built on it:

- `Display` of `Sequent` and `Formula` (`fmt.rs:137-200`);
- `Reading::fmt_formula` (`reading.rs:355-385`);
- `Notation::term` and `Notation::ill` (`notation.rs:60-134`);
- the Rocq `term` (`rocq.rs:256-296`).

`size.rs` `weights` (193-229) is a pass over the arena. **Open now** (item 5).

### 2.3 Invariants: occurrence sets, memo keys, hashing, orderings

- **The arena in topological order** (`core-sequents.md`; `verify_integrity`)
  must hold for the argument arena as well, and scope is a new invariant.
  **At 38.**
- **The parser's order of pushes is a format contract** (`core-sequents.md`,
  "The arena comes out in the order …"). An argument arena of its own keeps
  every propositional `TermId` and `OccId`, so every stored proof stays
  valid. **At 38**, with the decision in `api.md`.
- **"A name is an atom"** (`merge_atoms`). *Needs*: the namespaces of
  predicate, function and variable names, and a rule for one name used with
  two arities. **At 38.**
- **Hash-consing and α** (finding 5); the fixed seed of `core/src/hash.rs`
  serves the term tables too. Fresh eigenvariables and metavariables must be
  numbered by a counter, per worker on a pool, never by address, so that a
  run on one thread stays a function of its input, as the target set's
  columns require. **At 38.**
- **The preorder numbering**, a contract per research README §2, is
  untouched by binders. **None.**
- **The order of members.** `Inference.sequent` is "ascending ids"
  (`derivation.rs:545`), `Dyadic` likewise (`check.rs:349-356`), and so is
  `multiset.rs`. With instances, ties need an order that does not depend on
  when the arena created them (a parallel search creates them in any order),
  for example the structural order of the instance's terms (inference).
  **At 38.**
- **Zones.**
  - `Θ` is an `OccSet`;
  - `Context { set: OccSet, extra: Vec<(OccId, u32)> }` counts copies per id
    (`focus/context.rs:19-31`);
  - `Key { theta, gamma }` (`memo.rs:33-38`) is stored as fixed-width records
    (`memo.rs:167-175`; step 20's report: "the record's variable part … is
    where instances go").

  **Item 3** (open now); the rest **at 38, not local**.
- **Interchangeable occurrences** rest on "nothing a proof does tells them
  apart" (`classes.rs:4-10`) and are keyed by `TermId` (35-46). With bound
  variables written as indices, `p(#0)` under two different binders is one
  term, so a class needs the binder context or the instance, or is formed
  for ground subterms only (26, item 7). **At 38.**
- **The memo's facts** (`Failure::Complete`, `Hereditary`) hold per key.
  With metavariables, only ground stable sequents are memoized (38 §2). The
  loop check compares hashes, then keys (`focus/mod.rs:1152-1165`). **At 38.**
- **The counts** are per atom (`counts.rs:16-45`, the exponential atoms at
  209-227). Per predicate symbol they stay sound and get weaker.
  `Refutation::Unbalanced { atom, name }` then names a predicate. **At 38.**
- **The bias** is per atom (`bias.rs:13-60`) and becomes per predicate.
  **At 38.**
- **The checker's integers** (`check.rs:22-25`): every new counter (the set
  of open eigenvariables, the instance tables) saturates or refuses. **At 38.**
- **The occurrence bound** (`Forest::DEFAULT_LIMIT`, `MOST`) gets a sister
  bound on unfolded terms (finding 6). **At 38.**

### 2.4 The proof term and the checker

- `core/src/proofs/mod.rs:116-158` **`Node`**: the rule, an occurrence and at
  most two `NodeId`s, asserted at 16 bytes. *Needs*:
  - `Forall(o, eigen, p)` and `Exists(o, witness, p)`, which fit in 16 bytes;
  - `Ax` and `Copy` operands that are members (finding 2).

  **Open now**: the type of the operand and the table (item 2).
- `proofs/mod.rs:308-369` **`Proof { forest, nodes }`, `Proof::new(forest,
  nodes, root)`**: there is no witness or instance table, and `new` refuses
  any operand at `forest.len()` or above (336-340). A second constructor is
  additive, so `new` can stay for ground proofs (inference). **Open now** for
  the decision (item 2); the code at 38.
- `core/src/proofs/check.rs:63-111, 118-255, 264-286` **`Zone`
  (`HashSet<OccId>`), `Bag` (`HashMap<OccId, u32>`), `State`**, and
  `table_bytes(most, size_of::<OccId>())`: members as ids, with memory charged
  by the size of an entry. *Needs*: member keys (step 18's report). **At 38**,
  shaped by items 1 and 2.
- `check.rs:349-356, 447-460, 463-511` **`Dyadic { theta: Vec<OccId>,
  gamma: Vec<OccId>, any }` and `CheckError`**, all fields public, carried by
  `CheckError.premises` and `Problem::Conclusion`. **Open now** (items 1 and
  10). `Problem` is `#[non_exhaustive]`, so `Eigenvariable` and `Witness`
  refusals are additive.
- `check.rs:730-782, 970-987` **`examine(goal: &[OccId])`, `conclude`**: the
  goal is a list of ids, and the root's members carry no instance. **At 38.**
- `check.rs:808-849` **`Observer::weight(o: OccId)`, `Facts.shared:
  &[OccId]`**: per occurrence (the size and derivation observers, step 18's
  report). **At 38.**
- `check.rs:1161-1170` **`Ax`**: a literal, equal atoms, opposite signs.
  *Needs*: equality of instances, with eigenvariables as constants (38 §3);
  the hazard of finding 1. **At 38**, with the refusal in the data-model
  commit.
- `check.rs:1204-1229` **`&`**: `dl.gamma == dr.gamma` becomes equality by
  instance. **At 38.**
- `check.rs:1062-1068, 1266-1279` **`quest`, `Copy`**: an unrestricted member
  is the child of a `?`. *Needs*: `Θ` members as (child, instance). **At 38.**
- **The eigenvariable condition** is a new component of the state (38 §3: a
  set of unbound eigenvariables that flows up, counted by node because memo
  hits share subproofs). **At 38**, reviewed by the panel.
- `core/src/proofs/oracle.rs:261-272` is the checker's first implementation,
  kept for tests, and mirrors all of the above. **At 38.**
- `core/src/proofs/derivation.rs:61-175, 136` **`Rule`**: 33 rules and
  `Open`, with `ALL: [Self; 34]` a fixed-size public array. The label tables
  `[&str; Rule::ALL.len()]` are indexed by `rule as usize`
  (`style.rs:46-77, 222`). *Needs*: `∀ ∃ ∀L ∀R ∃L ∃R`, plus 34's `Cut` and
  36's divisions. **Open now**, minor: `ALL` as `&'static [Rule]` (item 10).
  The variants at 38.
- `derivation.rs:541-560` **`Inference { sequent: Vec<OccId>, rule,
  principal, premises, times }`**, all fields public: members as ids, and no
  witness. **Open now** (items 1 and 10).
- `derivation.rs:678-690, 1187-1290` **`of_goal(…, goal: &[OccId], …)`,
  `Build::unfold`**: top-down over a `Multiset` of ids, so instances can be
  applied on the way up. That is easier than in the checker. **At 38.**
- `core/src/proofs/size.rs:28-49, 193-229, 320-323` **`Size`** (fields
  public, `characters` "exact") and `weights` per term. The width of an
  instance depends on its witness. *Needs*: a weight per member, or
  `exact = false` under quantifiers; and the bound on unfolded terms. **At 38.**
- `core/src/proofs/interactive.rs`:
  - `goal(id) -> Option<&[OccId]>` (460-465);
  - `rules` (497-534);
  - `apply(goal, position, rule, left: &[usize])` (538-588): no witness and
    no metavariable, and the state holds no substitution;
  - `expand` (591 on);
  - `close` and `close_with` (816-852) hand id goals to `prove_goal` and
    `Derivation::of_goal`;
  - `term` (1006-1100), with the wildcards of 2.2;
  - `Refusal` is `#[non_exhaustive]` (37-104), so a refusal of a witness is
    additive.

  **Open now** for `apply`'s argument (item 4); the rest at 38.

### 2.5 The engines and the dispatch

- `core/src/search/mod.rs:229-323` **`prove_goal(forest, goal: &[OccId], …)`,
  `engine_for`**: public, with goals as ids. An open interactive goal under
  binders needs its instances. **Open now** (item 1).
- `search/mod.rs:327-340, 613-621` **`fragment_of`, `goal_fragment`**: the
  kinds of the subtree. **At 38**, with item 7's bit.
- `search/mod.rs:408-423` **`Task`**: unchanged per step 26, with the forest
  carrying the terms; members beyond ids need an optional instance list
  (38 §3). **At 38.**
- `search/mod.rs:444-463` **`Answer::of_arena`** calls `Proof::new(forest,
  nodes, root)` and passes the witness arena at 38. `Decide` (466-486) stays
  as it is (step 26). **At 38.**
- `search/mod.rs:489-533` **the dispatch**: the rows take a goal by
  `Fragment::contains`, and `.expect("the last two rows take every goal")`
  (495) panics on a fragment that no row's fragment contains. *Needs*: rows
  for the bit (focus, two-sided), or `NoEngine`. **At 38.**
- `search/mod.rs:582-641` **`Feature::FewEqualLiterals`** counts by atom.
  **At 38.**
- **Admission**:
  - `Focused::admits` checks the mode only (`focus/mod.rs:133-148`);
  - `Horn::admits` checks the shape only (`horn/mod.rs:38-43`);
  - the net and the additive engine check `Fragment::contains` (`net.rs:50-61`,
    `additive.rs:39-47`).

  `--engine focus` or `--engine horn` forced onto a first-order goal reaches
  an engine that conflates instances (finding 1). **Open now**: every
  `admits` checks the fragment, which changes nothing today (item 7).
- `search/mod.rs:259-268` with `focus/mod.rs:268-321` **the refutation of an
  exhaustive search**: counts per atom (sound per predicate) and
  `Refutation::Equation`, which counts connectives (unaffected, step 26).
  **At 38.**
- `search/mod.rs:664-756, 851-886, 1415-1491` **`Engine`, `Options`,
  `Reason`, `Statistics`**: `#[non_exhaustive]` or private fields. A bound on
  witness depth (D16), a term-depth reason and an `unifications` counter are
  additive. **None now.**
- **Focused engine internals**, each **at 38, not local** together, and the
  spike's subject:
  - `decompose` (`focus/mod.rs:824-893`) gains `∀` (a fresh eigenvariable, a
    trail mark);
  - `focus_on` (1723-1782) gains `∃`;
  - `initial`, `dual_in`, `mark_literals` and `meets` (1579-1694), and
    `split.rs` `dual_from` and `forced_side` (19-64), become unification with
    choice points (step 26, item 7);
  - the counts, classes, memo keys, `Context` and the bias as in 2.3;
  - the and-parallel `&` and the cubes assume independent premises
    (`focus/parallel.rs:1-22`, `LEVELS` at 41);
  - the answer enumeration of finding 3.
- **The net engine and nets.**
  - Partners by atom (`net.rs:433-445`), balance per atom (159, 231-240).
  - `ProofStructure::new` refuses by fragment (`nets/mod.rs:200-206`), which
    is right once the bit exists.
  - `dual` compares atoms (280-283).
  - `from_proof` reads `Ax` only (243-256).
  - `NetError` is exhaustive (45-74).

  MLL1 nets need quantifier links, eigenvariable jumps and one unifier for
  every link (step 26). **At 38** (a later sub-step); `NetError`
  `#[non_exhaustive]` is **open now** (item 10).
- **The additive path** refuses through `ALL.contains` once the bit exists;
  its dual compares atoms (`additive.rs:191`). **At 38.**
- **The Horn engine**: places by atom (`horn/mod.rs:405-412`,
  `horn/proof.rs:155`). First-order Horn is another engine (step 27's report).
  **At 38**: `admits` refuses.
- **The test generators**: `generate.rs` and `reference.rs` are propositional
  (`Tree` with `u8` atoms). Step 38's prompt asks for a second reference
  prover. **At 38.**

### 2.6 The JSON wire forms and their stability

- `core/src/serialize/sequents.rs:9-59` **the sequent**: tags `V D 1 ⊥ ⊤ 0 ⊗
  ⅋ & ⊕ ! ?`; the proxy `{terms, ids, var_dict}` has no
  `deny_unknown_fields`. An old reader ignores unknown keys and refuses
  unknown tags.
  - 38 §3's additions (tags `P`, `N`, `∀`, `∃`; keys `fo_terms`, `symbols`,
    `variables` with defaults, left out when empty) are safe because every
    first-order file then carries a new tag (inference).
  - A design that kept `{"V": a}` and put the arguments under a key beside
    it would be read by an older linlog, or a web client of step 32, as a
    propositional sequent, silently.

  **Open now**: the rule (item 8). `var_dict`, `V` and `D` stay on the wire,
  documented as atoms (a rename there is a format break). **None.**
- `serialize/proofs.rs:12-66` **the proof**: step tags, `{sequent, proof}`.
  `∀ [o, a, p]`, `∃ [o, w, p]` and a `witnesses` key are additive, and member
  ids as integers keep `ax: [x, y]` as it is (items 1 and 2).
  `core/tests/serialize.rs:223` pins an unknown tag (`cut`) as refused.
  **At 38.**
- `serialize/interactive.rs:14-43` **the interactive state**:
  `Step.sequent: Vec<u32>`. If a member stays one integer, the shape stays.
  If it becomes `[id, frame]`, an existing key changes shape. A witness per
  inference is an additive key. **Open now** (item 1).
- `serialize/nets.rs` **the proof structure**: `links` as pairs of
  occurrence ids, which an MLL1 net (no copies) keeps. **None.**
- `serialize/search.rs:9-43, 104-151` **fragments and refutations**: a
  fragment is its name, out of six (`NAMED`, with the `I` prefix stripped).
  New names must keep "lossy towards larger", and an old reader refuses them,
  which is right. The refutation's `atom` key then holds a predicate name; a
  new counter needs its proxy line. **At 38.**
- **The options forms** of the exports, `TextOptions`, `svg::Style` and
  `rocq::Options` have `serde(default, deny_unknown_fields)`. New fields for
  quantifier notation are additive for new readers, and old readers refuse
  new files, as intended. **None.**
- **No version field anywhere.** Step 38 needs none if item 8's rule holds,
  since an old reader fails on the new tags; a version still gives a better
  error (research README §2). **At 38**, or as that README decides.

### 2.7 The exports

- `core/src/export/notation.rs:20-55` **`Notation`** (crate-private) has no
  spelling for `∀` or `∃` and no term syntax (`atom: fn(&mut String, &str)`
  writes a name, 52). The constants per target are at `latex.rs:67`,
  `typst.rs:79` and `svg/mod.rs:213, 234`. **At 38.**
- `notation.rs:60-134, 151-199`: `term` and `ill` walk with `Walk`;
  `sequent(…, &[OccId], …)` takes members as ids, as do `Drawn::sequent` and
  `positions` (`proofs/style.rs:146-210, 218-283`). **Items 1 and 5.**
- `proofs/style.rs:46-77, 93`: the label tables, and the set of symbol
  characters, which lacks `∀ ∃`. Labels are per rule, so "∃ t" on an
  inference line needs a label per inference (`Drawn::label`, 207-215) and a
  "show witnesses" field (D15). **At 38.**
- `core/src/export/svg/font.rs:79`: 208 glyph advances, none for `∀ ∃` (650
  by default under Euler). `svg/net.rs:291` has the wildcard of 2.2. The
  `i<n>-<p>` ids are positions, unchanged. **At 38.**
- `core/src/export/rocq.rs`:
  - `Unsupported` (108-128) is exhaustive. **Open now** (item 10). It needs a
    `Quantifiers` variant until step 31's library has them.
  - `write` refuses at 550-557.
  - `identifiers` (242) and `term` (256-296) declare atoms as formulas; a
    predicate needs `Parameter p : term -> formula` and a type of terms.
    **At 38**, and step 31's kernel.
- `latex::Options` (212-231), `typst::Options` (146), `svg::Style` (58 on)
  and `TextOptions` (`proofs/fmt.rs:18-31`) have public fields and no
  `#[non_exhaustive]`. Quantifier spelling and the display of witnesses are
  new fields. **Open now** (item 10).
- `proofs/fmt.rs:170-205`: the text tree prints `forest.formula` and
  `reading.formula`. **Item 5.**
- The Typst layout and the compact view read only labels and
  `is_structural` (step 22's report). **None.**

### 2.8 The command: parser, flags, outputs

- `core/src/parse/mod.rs` (finding 4):
  - `operand` (272-313) reads identifiers as atoms (304-308);
  - `operator` (319-378) refuses a `(` after an operand (373);
  - `Pending` (64-75) needs a binder entry;
  - `finish` (395-414) passes the negation to `subterms`, which reach binder
    bodies but not arguments;
  - `Parser::most` (123, 166-178) bounds formula terms only.

  **At 38.**
- `sequents/mod.rs:15-61` and `core/src/lib.rs:71-91`: the syntax and the
  JSON as documented. **At 38.**
- `core/src/lltp.rs:38-43, 125`: a `.` ends a clause, and names map `-` to
  `‿` and `.` to `·`. This constrains the syntax (finding 4). **At 38.**
- `cli/src/argument_parsing.rs:1129-1158` **`FragmentArg`** has six values.
  **At 38.**
- `cli/src/interact.rs`:
  - the grammar `apply G P RULE [P…]` (25-31; the numbers after the rule are
    the split, 197-201) has no token for a witness;
  - `goal_line` (439-475) prints members through `forest.formula` and
    `reading.formula`.

  **Open now** (items 4, 1 and 5).
- `cli/src/lib.rs:296` (`seq fragment`), `cli/src/prove.rs:953-955` (the net
  drawing refuses outside MLL, which the bit keeps right), and the messages
  about the copy bound (`prove.rs:1019, 1161, 1234`); README's examples
  through `cli/tests/readme.rs`. **At 38.**
- `core/src/ordinary/`: propositional; `read_tptp` refuses first-order TPTP.
  Step 25's report puts `∀` and `∃` into `ordinary::Node` later. **At 38 or
  later.**
- Atom names from JSON print unescaped (`sequents/fmt.rs:146`), so a name
  `p(a)` would read back as a predicate after 38 (inference; minor). **At 38.**

### 2.9 The harness and the families

- `bench/src/run.rs:32-36, 339-354, 825-835`: the CSV header has `fragment`
  and `multiplicity`; resuming refuses a file with other columns;
  multiplicity is counted per atom and then means per predicate symbol (say
  so). **At 38.**
- `core/src/families.rs:525-545`: generators build text and parse it, with
  verdicts by construction; `Family` and `Instance` are
  `#[non_exhaustive]`. First-order families per 38 §2. **At 38.**
- `bench/src/problems.rs`: LLTP, ILTP's 274 propositional problems, qcover.
  ILTP's first-order problems need first-order ordinary logic. **Later.**
- `bench/targets.sh` and `bench/TARGETS.md` are D17's gate (equal counters,
  pinned CPU time within a few percent), which the spike runs. **None now.**

### 2.10 The tests and the docs

- `core/tests/parse.rs:154-184` pins `∀` (170) and `(A, B)` (163) as errors;
  the `∀` row changes with the syntax. **At 38.**
- `core/tests/serialize.rs` pins every form (25-48, 104-175, 237-266,
  269-320, 361-400). Add, per form, a test that reads a file from before the
  step unchanged (research README §2). **At 38**; the rule behind it is item
  8.
- `core/tests/depth.rs` (100 000 levels on a 256 KiB stack) gains binder
  towers and terms nested 100 000 deep (`f(f(…))`) for every walk, for
  unification and for the checker. **At 38.**
- `core/tests/export.rs` (the snapshots) and `cli/tests/readme.rs`. **At 38.**
- The rules files:
  - `core-sequents.md` (terms, kinds, fragments, parsing, serialization);
  - `core-forest.md` (literal lists by atom);
  - `core-proofs.md`, `core-derivations.md`;
  - `core-focus.md` (every prune of 2.3), `core-search.md` (dispatch, admission);
  - `core-export.md`;
  - `core.md` ("nothing recurses" for terms; the term bound).

  **At 38**, each in the commit that changes its area.

## 3. The ten "open now" items that would cost most to leave, ranked

1. **Decide what a member is, and use one type for it in the public API.**
   Today `OccId` stands for a member in:
   - `Inference.sequent` (`derivation.rs:546`);
   - `Dyadic.theta` and `Dyadic.gamma` (`check.rs:349-356`);
   - `Interactive::goal` (`interactive.rs:460`);
   - `prove_goal` and `engine_for` (`search/mod.rs:229, 314`);
   - `Drawn::sequent` and `Notation::sequent`;
   - the interactive JSON's `sequent` (`serialize/interactive.rs:17`);
   - the CLI's `goal_line`.

   Under `?` copies with instances an occurrence id names no member. One
   option (inference) is a `Member(u32)` newtype, distinct from `OccId`, that
   equals the occurrence id for a member without an instance and indexes the
   proof's instance table above `forest.len()`. It keeps every wire form's
   shape and costs nothing at run time.
   - *Now*: a mechanical retype, or field docs plus accessors.
   - *Later*: a breaking change on fields that the web client (32), step 33's
     instance members, step 34's extra trees and step 37's sequents all
     build on before 38.

2. **Make the proof term's leaves name members, and give `Proof` the tables
   for them** (`proofs/mod.rs:116-158, 322-369`). The bottom-up checker and
   shared subproofs need `Ax` and `Copy` to fix their instances (finding 2).
   `Node` stays 16 bytes only with a one-`u32` member.
   - *Now*: write the operand's meaning and the table into `api.md`, and tell
     step 31 to parameterize the occurrence argument of its Rocq `node`
     (research README §4, item 2).
   - *Later*: 31's datatype and reflective checker, which mirror `check.rs`,
     are redone.

3. **Make the engine generic over its zone member before steps 33 and 37
   write zone code** (`focus/context.rs:19-31`, `memo.rs:33-38, 167-175`,
   `classes.rs:31-49`, `Θ` as `OccSet`).
   - *Now*: the spike measures the generic form against the target set (D17),
     and `api.md` names the zone and frame types (research README, conflict
     6).
   - *Later*: 37's lifted `Context` and `Classes` and 33's instances, written
     for ids, are retrofitted, and the propositional fast path is re-measured
     each time.

4. **Give an interactive step an argument value with room for a witness or
   an open metavariable** (`interactive.rs:538-588`; the CLI's
   `apply G P RULE [P…]`, `interact.rs:25-31, 197-201`).
   - *Now*: a value such as `{position, rule, split}` that can take a
     `witness` field later, and a CLI token that a position cannot be.
   - *Later*: a breaking change to the protocol that step 32's web client is
     built on, and to the saved sessions.

5. **Make the formula walk public with a stop for binders and a walk over
   terms, and state that `Forest::formula(o)` prints with the names of the
   binders above `o`** (`sequents/fmt.rs:16-125`,
   `occurrences/mod.rs:279-281`, `reading.rs:355-385`, `notation.rs:60-199`,
   `rocq.rs:256-296`).
   - *Now*: one stop that no printer uses yet.
   - *Later*: every printer written in the meantime is reworked: 29's
     translators for eight provers, 31's Rocq formulas, 32's goal view, 36's
     printer.

6. **Replace the wildcard arms over `Term`, `Kind`, `Node` and `Rule` with
   exhaustive ones**, and add `clippy::wildcard_enum_match_arm` with allows
   on the shape readers that refuse (2.2's table, about 20 sites).
   - *Now*: edits with no change of behaviour (the target set's columns
     stay equal).
   - *Later*: silent wrong results instead of compiler errors. Each of the
     three steps that add variants (34, 36, 38) pays again: a corrupt arena
     from `map_subterms`, a `∀` recorded as `⊥`, an `∃` released without a
     witness.

7. **Write the fragment and atom contract, and make every engine's
   admission check the fragment** (`fragment.rs:13-42, 160-174`, `Forest`'s
   literal lists, `Focused::admits` at `focus/mod.rs:133`, `Horn::admits` at
   `horn/mod.rs:38`, the dispatch's `expect` at `search/mod.rs:495`).
   - The first-order bit is set by predicate arguments as well as by binders
     (finding 1).
   - `LL` and the other named constants stay propositional.
   - `Atom` names a predicate symbol, and a literal group holds candidates.
   - *Now*: docs and two one-line checks, neither of which refuses anything
     today.
   - *Later*: a window in which a forced engine or the checker accepts
     `p(a)` against `~p(b)`, and a meaning of `LL` that steps 29 and 32 will
     have published.

8. **State the rule for the wire forms: no first-order meaning rides on an
   old tag through a new key.** The data proxies ignore unknown keys
   (`serialize/sequents.rs:51-59`, `proofs.rs:60-66`, `interactive.rs:33-43`,
   `nets.rs`), so the rule is what makes every first-order file fail on an
   older reader instead of being misread.
   - *Now*: a paragraph in `api.md`, then one test per form that reads a
     file from before the step.
   - *Later*: files and web sessions in the wild misread by older versions,
     which no fix can recall.

9. **Rename the propositional "variable" vocabulary of the public API
   before 0.1.0**: `Term::Var` and `DualVar`, `Kind::Var` and `DualVar`,
   `Sign::Var` and `DualVar`, `Error::InvalidVariableIndex`, and the
   syntax docs' "variables". Keep `var_dict`, `V` and `D` on the wire,
   documented as atoms.
   - *Now*: free (D18; no aliases before the release).
   - *Later*: a major change, or two meanings of "variable" once terms have
     variables, metavariables and eigenvariables.

10. **Mark or encapsulate the public types that step 38 extends**:
    - `#[non_exhaustive]` on `Error`, `export::rocq::Unsupported` (which needs
      `Quantifiers`) and `NetError`;
    - `#[non_exhaustive]` or accessors on `Dyadic`, `Inference`,
      `CheckError`, `Size`, `latex::Options`, `typst::Options`, `svg::Style`
      and `TextOptions`;
    - `Rule::ALL` as a slice;
    - a size assertion on `Term` (12 bytes, D17).
    - *Now*: attributes and a builder style for the options (research README
      §2).
    - *Later*: every addition breaks callers. Marking a type
      `#[non_exhaustive]` after 0.1.0 is itself a breaking change.

## 4. What stays as it is for the propositional case

These need nothing from the design beyond keeping them:

- the occurrence numbering and every stored id, as long as arguments get an
  arena of their own;
- the sizes of `Term` and `Node`;
- the JSON of every propositional file;
- `Decide`, `Answer`, `Verdict` and the dispatch's rows (one more row);
- `Mode`;
- the count equation;
- the positions of the intuitionistic reading;
- the target set's counters (D17's gate).
