# Walk-through of step 37 (the focused inverse method) against `plan/notes/api.md`

Read: the design in full, `plan/37-inverse.md`, `plan/notes/research/37-inverse.md`, the 23 register entries whose steps include 37 (R9, R36, R37, R42, R43, R84, R86 to R88, R110, R111, R127, R135, R137, R147, R178, R208 to R210, R216, R231, R243, R244), and the present `search/mod.rs`, `focus/{context,classes,bias}.rs`, `occurrences/{mod,set}.rs`, `families.rs`, `cli/src/prove.rs` (statistics) and `tests/lock/json.txt`. Nothing was run. Section numbers are the design's.

Result in one line: no blocking item. Four frictions, eight notes. The design carries the engine's entry (8.3 recipe, 8.2 interface, 8.4 dispatch, 5.x bounds, 10.9) well; what is missing is at the edges of the counters, the lift list, and the wording of "complete".

## 1. The first change, sketched against the design

Step 37 is two sessions; the first change is two commits. (The first session ends with a measurement, so no `DISPATCH` row and no options knob are in them.)

### Commit 1: lift the zone types (counter-neutral, no wire change)

```rust
// core/src/search/zone/{mod.rs, context.rs, classes.rs}   (10.9; git-less move of focus/context.rs, focus/classes.rs)
pub(crate) use context::Context;           // unchanged: OccSet + extra copies + lo..hi range
pub(crate) use classes::{Classes, ClassMembers};

/// Per class, its occurrences in ascending order (a CSR over the forest): the way back
/// from a class to concrete occurrences (R84). Built by the engines that rename a derivation.
pub(crate) struct ClassMembers { start: Box<[u32]>, members: Box<[OccId]> }
impl ClassMembers {
    pub(crate) fn new(classes: &Classes) -> Self;
    pub(crate) fn of(&self, class: OccId) -> &[OccId];
    pub(crate) fn bytes(&self) -> usize;        // charged to the Account by its builder
}
```

`Classes::new(forest, reading)` keeps its signature; the focused engine's imports change from `super::{context, classes}` to `crate::search::zone`. Gate: `bench/targets.sh` columns equal `after-bias.csv` (10.9, R216).

### Commit 2: `Engine::Inverse` for the Mall/Mell fragments, forceable only

```rust
// search/mod.rs
#[non_exhaustive] pub enum Engine { Focus, Net, TwoSided, Additive, Horn, Inverse }
//   name() "inverse"; parallel() false; ALL gets it; implementation() => &inverse::INVERSE (8.3 items 1 to 5)
//   Display/FromStr/serde by name: no wire change but the string "inverse" (open set, 7.4)
pub enum Phase { /* … */ Saturate }              // 5.2, already reserved for [37]

// search/inverse/{mod.rs, rules.rs, database.rs, index.rs, read_off.rs}   (crate-private)
pub(crate) struct Inverse;
impl Decide for Inverse {
    fn admits(&self, task: &Task<'_>) -> Result<(), Error> {
        // 8.6 order: fragment, mode, goal, shape
        if !Fragment::LL.contains(task.fragment) {   // the [38] quantifier bit is outside LL: refused for free
            return Err(Error::EngineRefused { engine: Engine::Inverse,
                because: NotTaken::Fragment { decides: Fragment::LL, goal: task.fragment } });
        }
        // [36] an ordered mode: NotTaken::Mode(task.mode); every other mode, Mix and affine included, is taken
        Ok(())
    }
    fn decide(&self, task: &Task<'_>, options: &Options, limits: &Limits, account: &Account,
              stop: &mut dyn FnMut(Progress) -> bool) -> Result<Answer, Error>;
}

/// A derived (forward) sequent  Θ ; Γ →w, with the node that proves it.
struct Derived { theta: OccSet, gamma: Context, weak: bool, node: NodeId }   // = the checker's (Θ, Γ, any) (3.8)
/// One derived rule per frontier occurrence, as data (research note §2 item 1): premise
/// schemas over occurrence ids plus the Node to emit while matching.
struct Rule { principal: OccId, premises: Box<[Premise]>, emit: Emit }
```

`decide` does: `Classes::new` + `ClassMembers::new` (charged, polled every 65 536 occurrences, 5.2), one forest pass for the frontier and the rules, initial sequents per class pair, the lazy OTTER loop with the database and its feature-vector index (charged by capacity, 5.4), one `stop(Progress { work, done, held_bytes, phase: Saturate, item: 0 })` per activation and per long subsumption scan, `Limits::work` giving `Err(Reason::WorkLimit { limit })`, an exceeded account `Err(Reason::MemoryLimit { limit_bytes })`. The read-off is an iterative pass (R43) that emits `Node::Ax(x.into(), y.into())`, `Tensor`, `With`, `Top`, `Bang`, `Quest`, `Copy`, `Weaken`, `Mix` over `Member::from(OccId)`, and ends in `Answer::of_arena`. A strong sequent that equals the goal is the root; a weak one that is within it is the root as the checker's `Conclusion` rule accepts it. The running example `⊢ ~A, A ⊗ ~B, B` derives `ax(0,2)` and `ax(3,4)` as initial sequents `{0,2}` and `{3,4}`, the `⊗` frontier rule at occurrence 1 (left needs member 2, right needs member 3) gives `{0,1,4}`, and the node list is exactly 7.3's `[{"ax":[0,2]},{"ax":[3,4]},{"⊗":[1,0,1]}]`.

Statistics (8.5 table): `nodes` = sequents derived, `memo_hits` = conclusions forward-subsumed, `memo_entries` = the database's peak, `work` = the activation units. No new `Refutation` (R137): a completed saturation is `Ok(None)` and `conclude` adds the count refutation or `Exhausted`.

### Wire, as an inverse run writes it (no new key beyond W1)

```json
{"version": 1, "linlog": "0.2.0", "verdict": "proved", "fragment": "MELL", "mode": "classical",
 "engine": "inverse",
 "statistics": {"nodes": 41, "memo_hits": 12, "memo_entries": 29, "splits": 0, "links": 0, "tests": 0,
                "copies": 0, "forward_copies": 0, "work": 3180},
 "sequent": {…}, "nodes": [{"ax": [5, 1]}, {"copy": [1, 0]}, …]}
{"version": 1, "verdict": "unknown", "reason": {"kind": "work_limit", "limit": 1000000}, "engine": "inverse", …}
{"version": 1, "verdict": "unprovable", "refutation": {"kind": "exhausted"}, "engine": "inverse", …}
```

Second session (not sketched): the options knobs (6.2 "[37] the inverse engine's knobs"), the dispatch `Feature` and row if a measurement earns them, the harness columns.

## 2. Workarounds

### W1. `friction`: new counters have no home that keeps the level-1 bytes (8.5, 7.1, 7.3)

Step 37 needs counters the shared set lacks: R9 names "sequents derived, kept, subsumed and probed". 8.5 answers "added: none planned" and maps three shared counters. Backward subsumptions (the entries removed, which Imogen reports as half the database) and index probes (the cost the engine is tuned by) have no field. Adding them hits three places the design leaves open:
- 7.1 says a propositional value "writes 1 in every later release and its bytes never change", and 7.3 says `statistics` writes "every counter, always". Both cannot hold once step 37 appends a field: every outcome of every engine gains a key, `core/tests/lock/json.txt` (outcome lines 6 to 10) changes, and the lock change needs a commit of its own. Smallest change: say in 8.5 that counters appended after 0.1.0 are written only when non-zero (`skip_serializing_if`), so a level-1 value of another engine keeps its bytes.
- `Statistics` is read back (7.2, "both, nested") with a derived `Deserialize`; a 0.1.0 outcome has no `derived` key, so the read fails unless the struct carries `#[serde(default)]`. 8.5 says "no hand-listed proxy" and nothing about `default`. Smallest change: put `#[serde(default)]` on the struct in 8.5.
- Without it the session must squeeze backward subsumption and probes into `splits` or `copies`, which 8.5 forbids ("never gives an existing one a new meaning").

### W2. `friction`: the command and the harness have no machine-readable counter labels (8.3 item 9, 8.5, R137)

`cli/src/prove.rs:1195` prints counters with one `match` per engine and a `_` arm of focus labels ("stable sequents visited"). `Engine` is `#[non_exhaustive]`, so a downstream crate is forced to keep the `_`, and the 8.3 claim "the compiler names 2 to 4 sites" does not reach the command: an inverse run would print "stable sequents … memo entries … splits examined", wrong for the engine (R137), silently. 8.3 item 9 only says the flag lists derive from `ALL`. Smallest change: a public `Engine::counters(self) -> &'static [Counter]` (or `Statistics::labelled(engine)`) giving key, label and meaning per engine, which the command and the CSV header read, with a test that every `Engine::ALL` has one (the same test style as 7.4's table check).

### W3. `friction`: "complete" is worded so that a finished MELL saturation reads as "gave up" (8.2, 10.9 second bullet, R37, R111)

10.9: "`Ok(None)` only from a saturation that completed where it is complete (MALL); with exponentials it gives up with a `Reason` (R111)". 8.2's contract says an incomplete engine "gives up with a `Reason`, never with `Ok(None)`". Read literally, a MELL saturation that terminates (many unprovable inputs do: `Θ` is a set and the multiplicities do not grow on a cycle-free input) must be reported `Unknown`, and there is no `Reason` for it. That discards a correct refutation. R37 and the research note (§4, "Non-saturation") state the right rule: `Ok(None)` iff the saturation completed and no rule application was cut by a bound. The risk is a silent loss of `Unprovable` verdicts, never a wrong one. Smallest change: restate 8.2 and 10.9 as that rule; R111's "admits refusing the incomplete case when forced" is dropped for this engine (the prompt wants it forceable on exponentials). A later multiplicity bound is a new `Reason` variant, which `Reason` (`#[non_exhaustive]`, internally tagged) already allows.

### W4. `friction`: the lift list in 10.9 is shorter than what the engine reads (10.9 first bullet, R84)

- The atom polarity: the note and 6.2 reuse `Options::bias` for the inverse engine, but `focus/bias.rs::signs` is `pub(super)`; 10.9 lifts only `Context` and `Classes`. Add `bias` (and the polarity half of `Counts`, if the engine reads it) to the lift.
- The kept-proof arena: the database's sequents each hold a `NodeId`, backward subsumption kills nodes, and only `focus/arena.rs` has `collect`. Nothing in 10.9 or 5.4 places a shared arena; the session either lifts `Arena` too or writes a second collector (D7 forbids two engines of one algorithm, not shared data, but this is a second arena).
- R84 as 10.9 words it, "`Classes` gains the reverse map", makes the focused engine build and `account.charge(classes.bytes())` (`focus/mod.rs:210,231`, `parallel.rs:86`) a table it never reads. That moves the focused engine's charged memory, so "counter-neutral" fails for the memory-limit tests. Smallest change: say it is a separate type built by the renaming engines (`ClassMembers` above).
- `search/zone/` is also where spike M2 put the `Zone` trait (11.2). 11.5 is empty; if M2 passes, step 38 generifies `Context`; name the module so the lift is not moved twice.

### W5. `note`: the proof read-off cannot keep "sharing is the DAG" at class level (10.9 last bullet, R84, R127)

10.9 and R127 say the forward derivation becomes `Node` terms with sharing as the DAG and "no change in `check.rs`". But the engine's interchange of equal occurrences (the reason `Classes` is lifted) means a derived sequent holds classes with multiplicities; a `Node` names concrete members, and a shared subproof used at two places needs two different assignments. The rename pass therefore unshares (the same reason `core-focus.md` keeps `Proved` under the sequent's own key). The design is not wrong (the checker stays as it is, P4), but 10.9 should say that the read-off is a top-down rename with memo on `(node, assignment)`, charged to `limits.memory_bytes` and refused with `Reason::MemoryLimit`, and in affine mode where it emits `Weaken` (one per leftover, at the use site, not below a shared sequent). Otherwise the session will find it in the panel.

### W6. `note`: the dispatch feature has no stated input (8.4 "[37] many hypotheses, small goal", R88, 3.1)

`Feature::of(&Task)` must tell "many hypotheses, small goal". Classically the forest does not know the sides; `Sequent::antecedents()` does (3.1, `Some(k)` for every text, `None` for pre-release JSON and built sequents). But 3.1 says "the forest and the engines ignore it", and the sequent's meaning must not depend on how it was written. Choose one: the feature reads `task.forest.sequent().antecedents()` when `task.roots` (3.1's sentence relaxed to "the dispatch may read it for routing"), falling back to the reading's positions intuitionistically and to "not met" classically when `None`. Without it the row has to be a root-polarity heuristic. Verdicts are unaffected either way.

### W7. `note`: the default `search::Options` does not make a forced inverse call end (6.2, F42, 5.3)

F42/8.1: "a library call without a stop must end; the copy bound of 3 makes it end". The inverse engine ignores `copies`; `prove` is `Limits::default()` (`work: None`) and `|_| false`, so a non-saturating MELL goal ends only at the 1 GiB account (a long time, and `MemoryLimit`). The research note proposes no multiplicity bound in session one. Either `Options::default()` carries a `DEFAULT_*` bound for the inverse engine with its own `Reason` (D16, R147: a named constant and a flag), or 6.2 notes that the guarantee holds through memory only. The knob shape follows the `NetPrunes` precedent (6.2): a nested `inverse: InverseOptions` with `Default`, `with_*`, and the fields appended as a new key set (an options form has no `version`).

### W8. `note`: each engine's modes are written twice (8.3 item 1, 8.4 `Modes`, 8.6)

A new engine's `admits` repeats its mode test beside its `DISPATCH` row's `Modes`. `Modes::take` destructures the whole `Mode` so that 36's field is a compile error (F140); an `admits` that matches modes by hand does not get that. Smallest change: `Engine::modes(self) -> Modes` (exhaustive match) that `admits` calls, so a new `Mode` field is caught in one place; the Inverse takes `Modes::Any` minus ordered.

### W9. `note`: `Refutation::Saturated` is reserved but not needed (3.12 comment "later Saturated (37)", R137)

3.12 reserves a `Saturated` for 37; R137 says no new `Refutation` unless the engine can name its own. A checker that shares no engine code (P4, R124) cannot verify "the database is closed" without the database, so `Saturated` would carry only a count, which `Exhausted` plus `Statistics` already gives. Remove the reservation from the comment, or state what the certificate is. Not needed for the first change.

### W10. `note`: three small signatures the sketch assumed (3.4, 8.2)

- Every `Node` constructor in an engine takes `Member`; the engine writes `x.into()` at each emit site (3.4 "engines keep `OccId` inside"). Fine; the doc of `Member` could say `From<OccId>` is the one conversion engines use.
- `Answer::of_arena`'s new signature is not given; 8.2 says it sets goal and mode. For a goal off the roots (an `Interactive::close` on the inverse engine) it needs `Task` or the goal as an argument. State it.
- `Classes` is by term and position over `forest.ids()`, cut trees included (34); the frontier pass and the initial sequents must be restricted to `task.goal`'s subtrees. 8.2's `Task` carries that; a sentence in 10.6 ("the engines read the conclusion's or the goal's occurrences") would make it a promise.

### W11. `note`: two sources for "classes" (10.7 and 10.9)

10.7 says 35's crate-private `Forest::structural_classes()` is reused by 37's classes (R83); 10.9 says 37 lifts the existing `Classes` (by `TermId` and position). Hash-consing already makes `TermId` equality structural. Pick one definition before 35 and 37 both land, or 37's `Classes` sits beside a second class notion.

### W12. `note`: the harness and families are outside the design but not blocked (R208, R210)

`families::Family` is a `#[non_exhaustive]` struct table with `instance -> Result<Instance, Error>` (2.4), so a theory-heavy family (R210) is added without a break. R208 (`--only-undecided FILE`) is a harness flag; `bench.md` owns it. `Statistics`' appended columns follow W1.

## 3. Register entries naming step 37

| entry | met or placed | where / what is missing |
|---|---|---|
| R9 counters in `Statistics` | partly | 8.5 maps three shared counters; derived/kept/probed not placed; byte and `serde(default)` rule missing (W1) |
| R36 database charged | placed | 5.1, 5.4 table, 10.9 |
| R37 stop in every step | placed | 5.2 (`Phase::Saturate`), 5.6 last row, 10.9; `Ok(None)` wording needs W3 |
| R42 no clock or thread | placed | P-rules, 5.2; fixed-seed hasher |
| R43 no recursion | placed | P9; read-off is iterative, its test in `depth.rs` is the step's |
| R84 class to occurrences | placed in 10.9, but see W4 (charging) and W5 (unsharing) |
| R86 one list per engine | met | 8.3, `Engine::ALL`, 8.3 (9) for the lists; command labels W2 |
| R87 dispatch as data | met | 8.4 (a deleted row if unearned) |
| R88 new feature is a variant | met, input unplaced | 8.4 lists "[37] many hypotheses, small goal"; its input W6 |
| R110 parallel false, batch memory | met | 8.3 (3), 5.4 table |
| R111 saturated vs gave up | contradicts | 8.2 and 10.9 wording (W3); "admits refuses the incomplete case" is not wanted |
| R127 forward derivation to `Node` | placed | 10.9, 3.7, 3.8; unsharing W5 |
| R135 states what it admits | met | 8.6 and the `Engine` table; quantifier bit refused by `Fragment::LL.contains` |
| R137 command output right for the engine | not placed | counter labels W2; the unknown advice follows `Reason::setting()` (4.2) |
| R147 options read and ignored | placed, shape open | 6.2 "[37] knobs"; W7 |
| R178 `--engine inverse` | met | 8.3 (9) derives from `ALL` |
| R208 undecided rows | not in the design | harness, `bench.md` (W12) |
| R209 differential tests | met | 8.3 (8) |
| R210 families | met | 2.4 `Family` is a table |
| R216 target set | met | 10.9 first bullet; 11 |
| R231 docs in the same commit | met | 8.3 (10) |
| R243 progress stop | met | 5.2 |
| R244 one member type | met | 3.4 |

## 4. What fits well

- The checker's `(Θ, Γ, any)` of 3.8 is Chaudhuri's weak-flag calculus rule for rule, so the forward derivation needs no new `Node`, no recorded context and no change to the checker; 3.7's closed `Node` costs step 37 nothing.
- 8.3 (the recipe) and 8.2 (`Task`, `Answer`, `Decide`) fit a new engine with no change to `prove_goal`; `Fragment::LL`'s exclusion of the quantifier bit makes 38's refusal automatic; `Phase::Saturate`, `Limits::work`, `Reason::WorkLimit` and the account table already place the bounds.
- The public forest numbering (3.3, preorder, `subtree(o) == o..o+size(o)`, `root(o)`) gives the feature-vector index its per-root ranges without a new accessor, and `OccSet` becoming crate-private (2.3) leaves room for the sparse key the memo's follow-up wants.
