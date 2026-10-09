# Walk-through of step 35 (MLL engines: symmetry breaking, essential nets)

Read: `plan/notes/api.md` (all but 12/11.5), `plan/35-mll-engines.md`, the research note, the 29 register
entries naming 35, `core/src/search/{net.rs,mod.rs}`, `nets/{mod,skeleton}.rs`, `serialize/nets.rs`,
`export/svg/net.rs`, `cli/src/prove.rs`, `bench/src/run.rs`, `families.rs`, and the rules files
(`core-nets.md`, `core-search.md`, `core-parallel.md`). Nothing was run.

Result: 0 blocking, 9 friction, 6 notes. The design carries step 35 without a rewrite. What costs is
that it places the *criterion* (10.7) but not three things the step needs next to it: the symmetry
argument under a directed criterion, the quadratic memory, and the routing feature as a value.

## 1. The first change, sketched

The step's first two commits, as a session would write them against the design.

**Commit 1: `Symmetries` replaces `copy_before`/`copy_after` (counter-neutral).** Nothing public.

```rust
// search/net.rs, crate-private (R103, 10.7)
struct Symmetries { group: Box<[u32]>, before: Box<[u32]>, after: Box<[u32]> }  // per literal
impl Symmetries {
    /// Groups: literals of one atom and sign under the same maximal pure tree
    /// (a root literal is a group of root literals of its atom and sign, as today).
    fn new(forest: &Forest, positions: Option<&[Position]>, prunes: NetPrunes) -> Self;
    fn admissible(&self, net: &ProofStructure, x: VertexId, y: VertexId) -> bool;   // was `ordered` twice
}
```

Built in one index-order pass: with 3.3 promise 1 (parents before children) the tree id is
`tree[o] = if pure(parent(o), kind(o)) { tree[parent] } else { o }`, no recursion (P9). With the groups
being today's chains of root literals, `nodes`, `links`, `tests` of every row equal before and after
(R206's gate). Gate it before commit 2 changes any counter.

**Commit 2: the leaf break and its option.**

```rust
// search::Options gains (6.2, R143):
pub net_prunes: NetPrunes,                 // default: all on; wire "net_prunes": {"leaves": true, ...}
#[non_exhaustive] #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NetPrunes { pub leaves: bool, pub balance: bool, pub compounds: bool }
impl NetPrunes { pub const ALL: Self; pub const NONE: Self; #[must_use] pub const fn without_leaves(self) -> Self; /* ... */ }
// Statistics (non_exhaustive) appended, only if the ablation needs them (8.5, R9):
pub leaf_pruned: u64, pub balance_pruned: u64          // JSON "statistics" keys, CSV columns at the tail
```

```json
{"search": {"engine": "auto", "...": "...", "net_prunes": {"leaves": true, "balance": true, "compounds": true}}}
```

The balance prune reads per-component, per-atom counts kept in `Skeleton` (R82):

```rust
// nets/skeleton.rs (pub(crate) now; the engine is in search::)
pub(crate) fn find(&self, x: u32) -> u32;                  // or ProofStructure::component(VertexId)
pub(crate) fn unlinked(&self, root: u32, atom: Atom, sign: Sign) -> u32;   // payload restored by undo
```

**Commit 3 (routing):** `Feature::NoEqualLeaves` (`task.roots && no_equal_leaves(task.forest)`, same pass
as `Symmetries::new`); the row is decided by the harness; `Engine::Net`'s doc table gets the measurement.

```rust
#[non_exhaustive] pub struct Features { pub multiplicity: u32, pub equal_leaves: bool }   // see item 9
pub fn features(goal: Goal<'_>) -> Features;
```

**Commit 4+ (essential criterion):** `net::Linker<C: Correctness>`, crate-private; `Switching` is today's
loop. `Essential` keeps the closure.

```rust
pub(crate) trait Correctness {
    fn link(&mut self, x: VertexId, y: VertexId) -> bool;     // false: the directed cycle is closed (O(1) from the closure)
    fn unlink(&mut self);                                      // pops the undo log, in stack order (R35, R33)
    fn complete(&mut self, net: &ProofStructure, scratch: &mut Scratch) -> bool;   // Yeo | dominator condition
}
struct Essential { positions: Box<[Position]>, closure: Bits /* n rows of n/64 words */, log: Vec<..> }
```

Public surface (the part item 1 discusses):

```rust
impl ProofStructure {
    pub fn is_essential(&self, stop: impl FnMut(Progress) -> bool) -> Result<(), NetError>;   // positions from Reading::new(self.forest()) (R81)
}
// NetError (non_exhaustive) gains: DirectedCycle(Vec<VertexId>), NoDominator { link: VertexId, input: VertexId }
pub enum Engine { Focus, Net, TwoSided, Additive, Horn, Essential }          // "essential" on the wire, parallel() == false
pub enum NotTaken { ... }   // Essential::admits: Mode(m) for classical/affine/Mix, Fragment{decides: MLL,..}, Goal
```

`Essential::decide` charges `n * ceil(n/64) * 8` bytes to the `Account` before allocating; past the bound
it answers `Reason::MemoryLimit { limit_bytes }`. It sequentializes the net found
(`ProofStructure::sequentialize(&Limits, stop)`) and returns `Answer { net: Some(net), .. }` (R128).
Drawing: `svg::Style` gains `net: ..` fields (`arrows`, `polarity`, `dominators`), defaults reproduce
today's snapshots; `svg::net(&net, &style, &limits, stop)` reads `Reading::new(net.forest())`.

## 2. Workarounds

**1. `Criterion` gains `essential`, and `ProofStructure::is_essential` exists beside it (3.11 comment
`[35] essential`; 7.3 "[35] the essential criterion"; 10.7). Friction.**
Need: R123 wants "is this embedding-found net also an essential net of its reading", a question about a
structure whose own criterion is the switching one. That is `is_essential`, independent of `criterion()`.
The design also puts a field on `Criterion`, which (a) duplicates the question under `is_correct`, (b)
makes `{mix: true, essential: true}` a value that must be refused, (c) is a new value of an enumeration in
a read-back form, so it raises the wire level of the net form (7.1), and (d) cannot be chosen from a mode:
`Criterion::of(mode)` takes only the mode, and intuitionistic mode must keep giving the switching
criterion (the embedding, pinned). Nothing in the search needs the structure to carry it: the engine's
criterion is the type parameter of `Linker<C>`. Smallest change: remove `[35] essential` from `Criterion`
and from the net form in 7.3; step 35 adds `is_essential` only. No new level, no invalid combination.

**2. `is_correct(&self, stop)` and `is_essential` carry no `&Limits` while 5.6 lists `memory_bytes` for
`is_correct` and calls it "quadratic" (3.11 vs 5.6). Note.** The criterion functions must be linear in
memory (topological order plus a dominator tree, Lengauer–Tarjan or the iterative Cooper–Harvey–Kennedy)
for the signature to be right; the n² closure is a search-only structure. State that in 3.11, or `&Limits`
must be added to `is_correct` now (a later parameter is a break).

**3. `Symmetries` "built once per run in one forest pass" (10.7) is keyed by the forest alone; under the
essential criterion that key is unsound. Friction (silent wrong `Unprovable` if the structure is shared
as drawn).**
Swapping two equal literals of one pure `⅋` tree is an automorphism of the *undirected* switching
structure. The essential structure is directed: in `a ⊸ a ⊸ b` (`~a ⅋ (~a ⅋ ~b..)`) the two `~a` sit at
different implication links, so the swap is not an automorphism, and the lexicographic representative
of an orbit may be a non-essential linking while a swapped one is essential. Even in the embedding the
positions differ inside one pure tree whenever an implication's `⅋` is in it. Need: groups keyed by
`(atom, sign, tree, position)`, and for `Essential` pure-`⊗` trees plus `⅋` nodes that are not
implications only. Smallest change: 10.7 says `Symmetries::new(task, criterion)` (forest and the
`Reading`'s positions) and `Correctness` owns the key; the panel (prompt) reads the orbit argument per
criterion. `Task.reading` is already there (8.2), so no signature moves.

**4. The balance prune's table is memory outside every bound (5.1 "What memory_bytes counts" lists only
the closure for 35; R82, R34). Friction.**
Counts per component root and atom are `components x atoms` words: a Partition sequent is small, but a
`wide` sequent of 10^5 literals with 10^4 atoms is gigabytes, uncounted ("the net engine's linear
structure" is the excluded class, and this one is not linear). Smallest change: add "[35] the balance
table" to 5.1/5.4's list; it is allocated sparsely (only atoms of multiplicity above 1) or the prune is
switched off past a size, and `NetPrunes::balance` defaults to what the measurement keeps. Also the
accessor: `Skeleton` is `pub(super)` in `nets`, the engine is in `search::net`; R82's "component id"
needs a `pub(crate)` accessor (nothing public; 3.11 lists `same_component` only).

**5. The closure matrix under `Limits::UNBOUNDED` or `memory_bytes: None` (5.1, R130). Friction.**
`Account::charge` can refuse only against a bound. With none, n up to 50 million occurrences asks for
`n²/8` bytes: a capacity overflow or an allocation abort on input of the right type, which P-rules
(4.1 "no public call panics") forbid, and in wasm loses the instance. Smallest change: say that the
closure uses `try_reserve` and answers `Reason::IndexLimit` (5.1's representation caps, already in
`Reason`) when it fails or overflows, bound or not. Also the dispatch must not route a large IMLL to
`Essential` and turn a decided row into `Unknown(MemoryLimit)`: its row needs a size feature (8.4
`Feature` may read the occurrence count).

**6. `Statistics.tests` for the essential engine is "closure updates" (8.5) while R9 forbids giving a
counter a new meaning. Note.** Fine for a new engine variant, but if essential is "the net engine under a
criterion" instead (10.7 leaves it open), `tests` has two meanings in one engine and `bench/targets.sh`
columns mix them. Decide: a forceable `Engine::Essential` variant (also the only way a test or the harness
can force "net with the switching criterion" against "essential" on one intuitionistic sequent, which the
measurement needs). Counters appended are always written, so each changes the pinned outcome JSON of
every engine: the behaviour-lock commit of its own.

**7. `net_prunes` is a permanent Settings key for a measurement knob (6.2, 10.7). Note.** The options
form's key set only grows (7.1/6.1) and an older reader refuses a newer key, while the step's own prompt
says a prune that wins nowhere is deleted. `NetPrunes` is new at 35 (after the release), so it may be shaped
now: `#[non_exhaustive]`, private bits, wire form a list of switched-off names read by name, and a removed
name still read (documented as a no-op). `Settings::default()` JSON gains the key: README and lock block.

**8. `NET_MULTIPLICITY` becomes "a public documented constant" (6.2 last bullet). Friction (a breaking
change after 0.1.0).** The step may replace `FewEqualLiterals` by `NoEqualLeaves` (prompt, fixed item 2;
8.4 "a row the measurement does not earn is deleted"). A public constant that the library then removes is
a break the design created. Smallest change: keep it private; the number and its measurement live in the
`Engine` doc table (as today), and the harness gets the value through item 9.

**9. R204 is not met: the harness needs the routing feature's *value*, not the engine (10.1). Friction.**
`bench/src/run.rs:825` recomputes `multiplicity` from `Forest` on its own (drift already in the code;
R204: "computed through the library's own function"). 10.1 gives "a per-problem routing query beside
`engine_for`", which returns an `Engine`; a CSV column per feature value needs the value. Smallest
change: a public `search::Features` (`#[non_exhaustive]`, values) and `search::features(Goal) -> Features`
which `Feature::of` itself calls, placed at 35 (not 29); the harness column and `summary` group by it.

**10. `Row` has one `Feature` (8.4); the research's three candidates include "both". Note.** First-match
rows give OR, not AND. Make it `features: &'static [Feature]` (conjunction), a private change, so a
combination is a row and not a new variant per pair. The honest AND is likely (equal leaves and a size
bound, item 5).

**11. R115 (hand-over at the recursion limit) is "a decision of `conclude`" (8.4/10.7) but 8.1's `conclude`
has no such branch. Friction.** Needed answers: `Err(RecursionLimit)` from `Focus` retried with `Net` if it
`admits` (and the row for the mode exists); which `Outcome.engine` (the one that decided); which
`Statistics` (`Statistics::add` merges workers of one engine, and `nodes` means stable sequents in one and
literals in the other, so not a sum); `engine_for` cannot predict it (document); one account across both;
`race` (`Engine::parallel` is the first engine's). Smallest change: a paragraph in 8.1 step 6, and
`Outcome` (non-exhaustive) gains `abandoned: Vec<(Engine, Reason)>` or nothing but a documented rule.

**12. The essential drawing's selector and its failure modes are unplaced (9.1, 9.3, 10.7, R166, R175).
Friction.** `svg::net` takes only a `ProofStructure` and `Style`. The net an engine found in
intuitionistic mode has the switching criterion (item 1), so the essential layer cannot be chosen by the
criterion; it must be a `Style` field (`net: Plain | Essential`, plus `arrows`, `polarity`, `dominators`).
The drawing then needs `Reading::new` (fallible: `ShapeError`) and `is_essential` (dominator layer) and
errors for a net that is neither. Say so in 9.1: the estimate (`limits.derivation_bytes`) covers the extra
layers, the failures are `Error::Net`/`Error::Shape`, and the CLI's `--net` (a bool, R175) becomes
`--style svg.net=essential` plus a text form, with no new flag. `o<n>` ids stay; a dominator edge needs a
prefix reserved beside `b d j c` (9.3), e.g. `e`.

**13. `net::parallel` is written against the concrete `Engine` (cubes, `seed`, `reset`, `explore`).
Note.** Keep `Essential` sequential (`Engine::parallel() == false`, R105's decision) and let the cube
code name `Linker<Switching>`; then no generics reach the pool. A closure matrix per worker would also
multiply the memory by the threads.

**14. `answer()` does `.expect("...")` on `sequentialize()`, which after 3.11 takes `&Limits` and a stop
and can refuse. Note.** The refusal maps to `Reason::Stopped`/`MemoryLimit` (a net found and lost to a
bound), not to a panic; the same line serves both engines.

**15. The net engine's target set (R206) is not created anywhere in the design. Friction.** 3.11's gate
("the net engine's rows of the target set keep `links` and `tests`") assumes it, but `bench/targets.sh`
lists only focused rows. Commit 1 above is only "counter-neutral" if a net CSV exists before it, and
commit 2 changes the counters on purpose (a new oracle, its own commit). Place it: step 28 or 30
creates `bench/targets-net.sh` and its CSV; or step 33 (the first to change `ProofStructure`).

## 3. Register entries naming step 35

| entry | status | where / what is missing |
|---|---|---|
| R9 counters | met | 8.5 table, "appended, never a new meaning"; see item 6 |
| R33 pure function of links | met | 10.7 (kept in `link`/`unlink`) |
| R34 linear work in the polled loop | met | 10.7; balance table memory not (item 4) |
| R35 closure charged to the account | met | 5.1, 5.4, 10.7; unbounded case missing (item 5) |
| R42 no clock/thread/OS randomness | met | 10.4, P9 (fixed-seed hash in 10.7) |
| R50 non_exhaustive | met | 2.5 (`Criterion`, `NetError`, `Style`, `Engine`, `Statistics`, `Options`); `NetPrunes` is new at 35 |
| R79 calculus descriptor | met | 3.11, but see item 1 |
| R81 polarization of a net | met | 10.7 (recompute `Reading::new(net.forest())`; the engine copies positions from `Task.reading` via `position(o)`, no self-reference) |
| R82 skeleton aggregates | placed | 10.7; the accessor and the table's memory not (item 4) |
| R83 structural class | met | 10.7 `Forest::structural_classes()` |
| R86 adding an engine | met | 8.3 steps (1) to (10) |
| R87 dispatch data with measurement | met | 8.4 (doc table test) |
| R88 new feature is a variant | met | 8.4; conjunction missing (item 10) |
| R103 one symmetry structure | met | 10.7; keying under a directed criterion not (item 3) |
| R105 loop reusable with a second criterion | met | 10.7 `Linker<C>`; parallel decision to the step (item 13) |
| R123 criterion independent of search | met | 10.7, `NetError::{DirectedCycle, NoDominator}`; `is_essential` signature absent from 3.11 (items 1, 2) |
| R128 proof by sequentializing | met | 3.11, 8.2 (`Answer.net`) |
| R135 refusal | met | 8.6 (`NotTaken::{Mode, Fragment, Goal}`); `NetError::Mode` is named in 3.11 but not in its enum |
| R143 ablation | met | 6.2 `net_prunes`; item 7 |
| R144 drawing through `Style` | met | 9.3, 2.5 |
| R166 drawing of an essential net | placed, not met | 10.7 says "Style fields"; selector, estimate, failures not (item 12) |
| R175 command writes the essential net | not placed | the command's area; the flag shape follows from item 12 |
| R203 test-only linking enumerator | not in the design | belongs to the step's tests |
| R204 routing feature per problem | not met | 10.1 gives an engine query, not the value (item 9) |
| R205 unit-free IMLL families | fits | `Family` is a non-exhaustive static table; no change |
| R206 net target set | not placed | item 15 |
| R209 differential setup | met | 8.3 step (8); the pruned variant is `net_prunes` off, the oracle R203 |
| R216 target set as oracle | met | 11, 8.5 (columns appended); needs item 15 for the net |
| R229 doc table carries the measurement | met | 8.3 (1), (10); 8.4 |
| R115 hand-over (26, 27, later) | placed | 8.4, 10.7; item 11 |

## 4. What fits well

- `Forest`'s numbering contract (3.3 promise 1) makes the pure-tree pass one index-order loop with no
  recursion, and `VertexId(i) == OccId(i)` leaves the hot loop and the cubes' `(OccId, OccId)` pairs
  untouched.
- `Decide`/`Task`/`Answer` take a new engine with no change to the front door (8.2, 8.3); `Engine`
  non-exhaustive, `Statistics` appended counters, `NetError` and `Style` non-exhaustive carry every
  addition of the step without a break; `Task` is crate-private, so its extra fields (positions, groups)
  cost nothing.
- `Linker<C>` as a monomorphised parameter satisfies D17 (the switching instance is today's loop), and
  `Limits`/`Account`/`Reason::MemoryLimit` already have the shape the closure matrix needs.
