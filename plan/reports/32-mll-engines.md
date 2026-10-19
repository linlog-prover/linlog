# Step 35: engines for MLL and IMLL

## The checklist

| item | state | evidence |
|---|---|---|
| Session 35a: the net engine's pruning and the routing (session `step-35a`, 2026-10-18) | done (2026-10-18) | The three prunes in `search/net.rs` (vzslkryp, nstkrpsr), each with its argument (below) and `agrees_with_every_linking` against the enumeration of every linking, with each prune off and requiring each to cut nodes: passes; deliberate faults fail it. Horn encodings: `3-partition-mll-no/5` 16 202 533 nodes to 227, `partition-no/3` 269 950 to 391; the balance prune never fires on a family. Net target set: no decided verdict changed (64 rows), the 22 Horn rows' counters fell by design; new oracle `net-after-prunes` (oopzxvzv); `net-after-routing` equal to it. Routing measured on cores 2 and 3 (4 min 32 s): the focused engines faster on every MLL and IMLL row both decide (1.07 to 32 000 times); the net engine decides only at their limits (the chain of 1 024 and 4 096 links, `wide-m1` at 4 096); so no dispatch row, a handover at `RecursionLimit` or `MemoryLimit` where the linking is forced (punyzkxx, rmpwpwsu, qoszqxns; departs from the design). Focused target set `after-routing`: no verdict changed, five rows' counters explained. Panels (Opus 5.5 counterexamples, Fable 5.1 argument, Sonnet 5.5 integers) on the prunes: not refuted, unstated steps written down (nzvlkoxp); on the routing: two defects found (memory left charged, a search without end on equal literals), both fixed and re-reviewed, not refuted. Every code commit gated; `nix flake check` at the session's last commit (below). |
| Session 35b: essential nets for IMLL (session `step-35b`, 2026-10-18) | done (2026-10-18) | `ProofStructure::is_essential` (directed acyclicity, the dominator condition, witnesses) equal to `is_correct` on every linking of generated IMLL sequents (`kvnxpwko`); the net engine's loop generic over its criterion, its search unchanged (net target set: every counter of `net-after-prunes`, its journey −0.06 %; `lrymttrx`); `Engine::Essential` with a closure of bits and an undo log charged to the memory bound, agreeing with the two-sided engine and the embedding on 442 generated sequents, every proof checked (`tqovoyop`). Measured by 35a's method on cores 2 and 3: wins only the 64-link chain (0.18 against 0.22 ms), loses every other decided row by 1.3 to 25 times, and the net engine is 2.2 and 6 times faster at the recursion limit: dispatch unchanged; it beats the net engine on Partition by 16 to 660 times. The essential drawing in the SVG export (`svg::Style::essential`, snapshots `essential.svg`, `essential-around.svg`; `tkmxmrkn`). Both target sets: no decided verdict changed. Panel: argument (Fable 5.1) and counterexamples not refuted; integers refuted once (an abort without a memory bound), fixed and the witness rerun (`uyztqwzl`). Every code commit gated; `nix flake check` run at the last commit, its exit status in the last message to `planning`. |

## Session 35a: the net engine's pruning and the routing

**Outcome.** The net engine has its three prunes: the symmetry break for
equal literals under one tree of `⊗` alone or `⅋` alone, the sound
symmetry break for equal compound subformulas (conclusions, and leaves of
one such tree), and a per-atom balance over the `⅋`-free skeleton's
components. Each has its argument (below) and a differential test against
the enumeration of every linking (`agrees_with_every_linking`), which
runs the engine with every prune and with each switched off and requires
each to cut nodes. On the Horn encodings the symmetry breaks take the
engine from seconds or the 60 s limit to milliseconds
(`3-partition-mll-no/5` 16 202 533 nodes and 56 s to 227 nodes and
0.66 ms, `partition-no/3` 269 950 nodes and 3.7 s to 391 nodes and
1.3 ms); the balance prune never fires on any family. Then the routing,
decided by the harness on the pruned engine: wherever both decide, the
focused engines are faster on every MLL and IMLL row, classical, with Mix
and intuitionistic (`wide` 1.07 to 3.6 times, the Horn encodings 1.6 to
32 000 times), and the net engine decides only where they stop, at the
recursion limit (the implication chain of 1 024 and 4 096 links) or the
memory bound (`wide-m1` with 4 096 literals). Which goal reaches a limit
depends on the limits, which the dispatch's features do not see, so the
net engine lost its dispatch row and takes a goal over where the focused
engine ends at either limit, if the goal's atoms occur once with each
sign (its linking is then forced, and the net search one pass). The net target set
keeps every decided verdict; its counters fell on the 22 Horn rows by
design, and that file is the net set's new oracle. The review of the routing found two defects, both fixed:
at a handover after the memory bound the first search's set-up charges
stayed on the account and voided the net engine's proof as unchecked;
and a goal with repeated literals went from the focused engine's quick
"unknown" to a net search without end, hence the forced-linking
condition.

### The three prunes and their arguments

All three live in `core/src/search/net.rs`: the classes in `Symmetry`
(built once per run, `Symmetry::new`), the checks in `admissible` and
`ordered`, the balance in `balanced`, called from `choose`. `Prunes`
switches each off for the tests only; a search applies all three.

**1. Equal literals under one tree.** A *literal class* is the equal
literals (atom and sign) that are conclusions (as before), or leaves of
one maximal tree of `⊗` alone or of `⅋` alone, the tree named by its top
(the anchor). A link is admissible only if the partners ascend by id
along each class. *Why exchanging two such leaves keeps correctness*, for
complete and partial linkings, with and without Mix: a `⊗` tree is a
connected acyclic subgraph of every switching, and contracting it to one
vertex, from which its leaves hang, keeps the cyclomatic number and the
components of every switching, whatever the tree's shape; every switching
of a `⅋` tree joins its root to exactly one leaf and leaves the others as
pendant paths, so the switchings of the structure join the root to each
leaf in turn, whatever the shape. An exchange of two leaves' partners is
the original linking on the reshaped tree, so it maps nets to nets and
non-nets to non-nets. *Why one linking of every orbit survives*: the
lexicographically least linking of an orbit (partners listed by literal
id) has ascending partners along every class, since consecutive
`x < x'` with partners `p > q` exchange into a linking smaller at
`min(x, q)`; this holds for all classes at once, wherever the partners
lie.

**2. Equal compound subformulas.** A *compound class* is the equal
compound subformulas (by a structural id computed bottom-up) that are
conclusions or leaves of one such tree; exchanging two of them is
correct by the same arguments for whole subtrees. A class is *broken*
when its copies have a *key literal*: the first literal of the first copy
that is in no literal class of two or more and inside no copy of an inner
broken class (inner classes are decided first); its offset holds in every
copy, since copies have one shape. Along the copies, the *keys* of the
key literals' partners must not descend, where a literal's key is the
first literal of its literal class, or `NONE`, above every key, inside a
copy of a broken class. *Why one linking of every orbit survives*: sort
the copies of each broken class by key, outer classes first (sorting
moves only literals inside the class's copies, all keyed `NONE`, and
carries the inner classes along, so every other class keeps its order),
then take the lexicographically least linking that exchanges within
literal classes reach; those exchanges move no key literal (it is in no
literal class) and change no key (they stay within a class, whose
members share the key). This is why the step-6 counterexample to the
spec's first-literal key (two groups whose first literals link into each
other's copies) does not apply: a key inside another broken copy is
`NONE`. A class without a key literal is not broken, which leaves its
copies unmoved and their literals keyed: in Lincoln's 3-Partition the
bins have none (all their literals are in literal classes), so the items
of equal size are keyed by their bin's `~k` class, which is what cuts the
item permutations. A class whose key literals can only take `NONE`-keyed
partners gets no chain, since its order could never reject (`wide-m2`'s
pairs, where the check cost 5 % of `search-wide-m2-256`).

**3. The balance over the skeleton's components.** No component of the
`⅋`-free skeleton (the `⊗` premise edges and the links, the structure's
union-find) may hold more unlinked literals of an atom than the atom has
unlinked pairs. *Why*: a literal never links into its own component (the
link would close a cycle every switching keeps), so each `a` of a
component needs its own `~a` outside it, and each `~a` its own `a`; the
final skeleton is a forest, so no future link serves two of them. With
two pairs or fewer, a component that breaks it holds two literals of one
sign and one of the other, which then has no admissible partner, so the
partner count of `choose` finds the dead end in the same call: the check
runs only on atoms with three pairs or more. It runs where the exact test
runs (every node up to 200 occurrences, every fourth link above), which
finds such a branch dead some links later anyway. *What it does*:
nothing on the families: the node counts of every Partition,
3-Partition and `wide` instance are equal with and without it, because
the count equation and the `⅋` boundaries keep a component to one sign of
an atom there. It is the analogue of the focused engine's split counts
in form only: what refutes the Horn encodings at a `⊗` split is a count
over whole formulas, which a net sees only as a cycle after the links. A
stronger balance needs the sets of literals joined in some switching,
which no cheap structure keeps (a follow-up, below). On every atom at
every node it cost `search-wide-m2-256` 30 % of its instructions and
`wide-m3` and `wide-m4` time; with the threshold and the cadence the
journey stands at +1.02 % of its ceiling.

**The tests.** `agrees_with_every_linking` decides more than a thousand
sequents (random balanced ones over one or two atoms, which put equal
literals under one tree; `F₁ ⊗ F₁, F₂, F₂, …` made of them, which keeps
the counts and has equal compound conclusions and leaves; and
`F, F, ~F ⊗ y, ~y ⊗ ~F` and `F ⊗ F, ~F ⊗ y, ~F, ~y` and their mutants,
whose copies link into subformulas no exchange moves), with and without
Mix, by the enumeration of every linking (every per-atom pairing, the
criterion on each), and compares the engine with all prunes (through
`prove`, its proof and net checked) and with each prune off and with
none; every prune must cut nodes on the sample. Deliberate faults made it
fail: a literal class spanning trees, keys taken inside broken copies,
the balance bound one too low. A key literal allowed inside a literal
class passed it and 40 000 more rounds in a throwaway run: that
restriction stands on the argument alone (the review below confirms it
is needed).

### The routing

**The measurement** (the harness on the pruned engine, one thread, 60 s
per run, runs under 2 s three times; a unit on cores 2 and 3 under the
machine lock, 07:46:40 to 07:51:12, 4 min 32 s, peak 838 MB; the
families of the third baseline's `engines` pass plus `wide-m1` and
`wide-m2` at 4 096, the same rows under Mix and the intuitionistic chain,
wide and curried sequents of 64 to 4 096 atoms from problem files, and
the Partition families in intuitionistic mode). Median wall times:

| rows | focused engine | net engine (pruned) |
|---|---|---|
| `wide-m1` to `wide-m4`, 8 to 32 literals | 0.05 to 0.13 ms | 0.06 to 0.28 ms |
| `wide-m1` to `wide-m4`, 256 | 2.6 to 3.5 ms | 7.9 to 9.1 ms |
| `wide-m1` to `wide-m4`, 2 048 | 166 to 199 ms | 532 to 594 ms |
| `wide-m2` at 4 096 | 0.78 s | 2.16 s |
| `wide-m1` at 4 096 | memory bound after 0.75 s | **2.13 s** |
| `wide` under Mix, 2 048 and 4 096 | 0.18 to 0.81 s; `m1`/4 096 memory bound after 0.79 s | 0.54 to 2.47 s; `m1`/4 096 **2.21 s** |
| `3-partition-mll-yes` 4 to 12, `-no` 4 to 8 | 33 to 53 µs | 0.08 to 2.8 ms |
| `partition-yes` 4 to 6, `partition-no` 3, 4 | 0.07 to 0.14 ms | 1.3 to 436 ms |
| `partition-yes` 12, 20, `partition-no` 9, 12 | 12 ms to 3.3 s | out of time at 60 s |
| the Partition table, 9 to 224 occurrences | 45 µs to 1.0 ms | 48 µs to 3.7 s |
| Partition and 3-Partition under Mix | 0.04 to 0.07 ms | 0.1 to 2.5 ms |
| intuitionistic chain, 64 and 256 links | (two-sided) faster | |
| intuitionistic chain, 1 024 and 4 096 links | (two-sided) recursion limit after 11 and 46 ms | **26 ms, 0.43 s** |
| intuitionistic wide and curried, 1 024 and 4 096 atoms | (two-sided) 5.2 to 83 ms | 49 ms to 2.2 s |
| Partition in intuitionistic mode, 3 to 12 items | (two-sided) 0.08 to 76 ms | 1.4 ms to out of time |

So the focused engines win every row both decide, by 1.07 times
(`partition-table/1-1`, `wide-m1/8`) to 32 000 times
(`partition-table/2-2-2-2-2-2-9-1`), and the net engine decides three
rows they do not, each at a limit of theirs. *The routing feature*: the
prompt's candidate, "no two equal literals under one pure tree", is not
it (the net engine loses on sequents of distinct atoms too), and neither
is multiplicity, width or size: `wide-m1` and `wide-m2` at 4 096 are the
same sequent up to the sharing of atoms, and the focused engine proves
the second in 0.78 s; the chain and the curried sequent have the same
size and multiplicity. What separates the engines is the focused
engine's limit itself. So `DISPATCH` lost the net row (and with it
`Feature::FewEqualLiterals`, `NET_MULTIPLICITY` and `Modes::Linear`), and
`answer_goal` hands a goal over (`handed_over`): no engine forced, the
first engine ended at `RecursionLimit` or `MemoryLimit`, and the net
engine admits the task (unit-free MLL, the roots, no weakening), and
every atom occurs once with each sign (`one_linking`, added after the
review below); the net engine then decides it with the same options,
limits, stop and work count, after what the first search left charged is
given back (`Account::rewind`, also after the review), the outcome names
it, its counters replace the first search's, and `work` counts both, in
mixed units. The cost of a handover is the first search's
time to its limit: 11 and 46 ms on the chains, 0.75 s on `wide-m1` at
4 096 (2.9 s in all against 2.1 s forced). The `Engine` documentation's
table carries the measurement on the two general rows and in the
paragraph on the handover (`the_documented_dispatch_is_the_dispatch`
checks the table against `DISPATCH`).

*Departure from the plan's wording*: item 2 asks for a feature row in
the dispatch table; the measurement shows no static feature, so the
net engine's place is a handover at a limit, documented beside the
table rather than as a row of it (departs from the design).

*What changed for a user*: MLL and IMLL sequents are now proved by the
focused engines, so the verdict line names `focus` or `two-sided`, the
derivations are theirs (on README's examples the same derivations; in
JSON the axioms of some proofs are written in the other order), and
`--net` reads the net off the proof instead of the search; `--engine net`
gives the net engine's own search, statistics and net as before. The
behaviour locks (`cli/tests/lock`, `core/tests/lock/json.txt`) were
reblessed in the routing commit, README's examples and prose updated, the
tests that pinned the net engine's own output now force it (the `--stats`
example, `outcome_json_format`, the pool's shared runtime, the net
engine's two journeys), and `embedding_agrees_with_the_two_sided_engine`
forces it on its old sample (no literal more than twice), since on
generated IMLL sequents with literals three times or more the net engine
took 212 s of a debug build.

### Options

No option was added. The prunes are part of the engine's search, as the
conclusion classes were, and have no setting a user would vary
(`Prunes` exists for the tests only); the handover follows the dispatch's
rule that thresholds are not options, and `Options::engine` remains the
knob for every front end (`--engine` in the CLI, `engine` in a settings
file and in the web front end's JSON). Quantifiers (D17): nothing here
fixes an interface; the net engine stays propositional.

### The reviews

The panel of step 26, each member in a fresh context with the diff, the
claim and the code but not this session's reasoning, under the machine
rules (named cores, capped scopes, bounded runs); twice: on the prunes
(`vzslkryp`) and on the routing (`punyzkxx`, then the fix `rmpwpwsu`).

*The prunes.* **Counterexamples** (Opus 5.5, `high`): wrote its own
reference (every linking, every Danos–Regnier switching, its own
union-find, no code shared with `nets/`) beside the committed
sequent-calculus reference prover, and compared about 457 000 sequent
and mode pairs (about 228 000 provable) in 14 engine configurations (the
eight subsets of the prunes, two test periods, `prove` forced to the net
engine at three periods with every proof checked, two threads), from
balanced sequents, copies of compound formulas as conclusions and as
leaves of `⊗` and `⅋` trees, nested copies, doubled provable sequents,
mutants, and small Partition and 3-Partition instances: not refuted;
the two references never disagreed; its three deliberate faults
(balance bound one too low, a strict key order, leaf classes across
connectives) were each caught by thousands of mismatches. **The
argument** (Fable 5.1, `high`): stated the invariant from the code
before reading the argument and proved it (the exchanges for `⊗` and
`⅋` trees, literal and compound leaves, with and without Mix; the
orbit theorem with both kinds of constraint across nested classes; the
completeness of pruning partial linkings; the balance and its
threshold; the cubes): not refuted. It named six steps the written
argument left unsaid (the exchange as the linking on a reshaped tree,
sorting one class keeping the others' order, the key's offset holding in
every copy, why partial linkings are pruned without loss, why outer
classes go first, why a vacuous chain may go) and confirmed that the
restriction on key literals, which the test does not catch, is needed.
*Answered*: all six are now in `Symmetry`'s documentation and in
`core-nets.md` (`nzvlkoxp`). **Integers and limits** (Sonnet 5.5,
`high`): every number (ids, the sentinel, anchors, shapes, offsets,
keys, tallies) driven to its edge (forests to 65 534 occurrences, key
literals as the last occurrence, roots all literals, empty and
one-occurrence sequents, atoms with exactly three pairs, 3 000-atom
chains, 17 904 random sequents against the focused engine): not
refuted. *Answered*: the forest's bound is now stated on
`Symmetry::new`; the tables' memory, uncharged as the structure is, is
now on `Engine::Net`'s documentation; the per-node cost of the balance
was already cut by the cadence (`nstkrpsr`). Its aside, the focused
engine overflowing a 2 MB debug test thread on a 200-link chain, is a
follow-up below.

*The routing.* **The argument** (Fable 5.1, `high`): refuted. At a
handover after `MemoryLimit` reached in the focused engine's set-up,
the set-up's charges (`Classes`, `Counts`, charged and never released)
stayed on the account, the net engine proved the sequent, and the
proof's check ran under the spent account: `Unknown(Unchecked)` where
`--engine net` proves (`wide(256, 1)` under 1 MiB). **Integers and
limits** (Sonnet 5.5, `high`): refuted, the same defect, also at the
default limits (`wide(8192, 1)`: `Unknown(Unchecked)` after 12.1 s) and
for `wide(4096, 1)` under 4 to 256 MiB; everything else held (the work
bound overshot by at most one poll and a proof never returned past it,
the stop closure across both searches, recursion limits 0 to 5 and
chains to 20 000 links, the guards: affine, forced engines, units,
open goals). *Answered*: fixed in `rmpwpwsu`: `Account::rewind` gives
back everything charged since the first search began before the net
engine runs (it returned without a proof, so it holds nothing); the
reviewer's case is now part of `the_net_engine_takes_over_at_a_limit`,
which fails without the fix (`Unknown(Unchecked { limit_bytes: 1048576 })`)
and passes with it, and `wide(8192, 1)` at the default limits is proved
by the net engine in 12.1 s. The argument's other points, each
answered in `rmpwpwsu`: the `Engine` table's "3 to 83 ms" is 5 to 83 ms
and its "30 000" is 32 000; the handover's times now name the first
search's time too; "no static feature" overstated it: the honest claim,
now in `core-search.md`, is that which goal reaches a limit depends on
the limits, which no feature of the dispatch sees, and that the removed
row would have taken both measured cases at a cost (2.8 times on
`wide-m2` at 4 096, 10 to 26 times on intuitionistic wide and curried
sequents) above the handover's; README's "a sequent of thousands of
literals" became "some sequents of thousands of distinct atoms";
`Statistics::work` now says it mixes units after a handover; that a race
adds the other search's counters is said in `handed_over` and the rules;
`IndexLimit` is not handed over (no row measured there), said in the
rules. **Counterexamples** (Opus 5.5, `high`): no wrong verdict, panic
or verdict after a stop in about 73 000 runs over generated sequents
(classical with and without Mix and units, intuitionistic, affine)
under 13 settings of the recursion limit and the memory bound, 20 759
of them handovers, every verdict against the reference prover; chains,
`wide`, forced engines, 1 023 open goals, stops at every poll, every
work bound, `jobs` 2, a pool and `race` on two threads, and the CLI's
flags. It reproduced the memory defect at `punyzkxx` (615 generated
cases) and found none at `rmpwpwsu`. It refuted claim (3), "no hang":
the handover took any unit-free MLL goal at a limit, also one with an
equal-literal core, where the net engine is exponential: a 2 100-link
chain beside a sequent of repeated `a` and `b` (its witness) went from
the focused engine's `RecursionLimit` after 0.9 s to a net search still
running at the 20 s stop, and without a stop practically for ever.
*Answered* in `qoszqxns`: the handover also requires every atom to
occur once with each sign (`one_linking`), so the linking is forced and
the net search one pass; every measured handover has that shape, and
both witnesses now end as before the routing (`unknown … the recursion
limit of 2048 was reached after 975.49ms`, and after 2.48 ms at a limit
of 1); `the_net_engine_takes_over_at_a_limit` pins an equal-literal goal
that stays with the two-sided engine at its limit. Its third point,
that `work` mixes two engines' units after a handover and the verdict
line names only the last reason, is documented (`Statistics::work`) and
left as is.

*The fixes, reviewed again* (the argument and the counterexamples
resumed on `rmpwpwsu` and `qoszqxns`): not refuted. **The argument**
(Fable 5.1): `rewind` is right on every path (the single search, a
race's parts, which another thread's charges cannot disturb, and the
`share` and `fork` accounts, which give back their own at their drop);
`one_linking` with the count check means every literal has exactly one
dual, so `choose` forces every link, the symmetry and balance prunes are
idle, and a failed test unwinds without alternatives; every case that
failed before now returns the net engine's checked proof. Three points
on the wording, answered in `tvstokwp`: `Limits::work` now says it mixes
both engines' units after a handover, `rewind`'s doc names a memo's
reserve, and "one pass" became "one link per pair, no backtracking,
quadratic" (the scan of the atoms per link). Its fourth point, that
`wide(8192, 1)` at the default limits was its own estimate: the integers
reviewer ran it (12.1 s, `Unknown(Unchecked)`), and after the fix the
command proves it with the net engine in 12.1 s. **Counterexamples**
(Opus 5.5): no wrong verdict, no handover where none is due, no hang,
no panic in 52 040 runs under the 13 settings (4 987 handovers, 22 094
verdicts against the reference prover, every run capped by a 3 s stop
and none reaching it); both witnesses stay with the first engine
(`long.txt` at the recursion limit after 0.87 s, as forced). By design
the condition passes up goals with a repeated literal that the net
engine decides quickly at the artificially small limits of the runs
(depths 1 to 9: of 9 835 such cases the forced net engine decided 6 937
within 50 ms and left 2 877 undecided at 3 s); at the default limits the
focused engines decide such small goals themselves. The stops, work
bounds, parallel paths and the CLI were not re-run on `qoszqxns`, which
only adds a condition to the handover.

### The target sets

Both on cores 2 and 3 under the machine lock. *The net engine's set*
after the prunes (`net-after-prunes`, about 5 minutes on core 2)
against `net-after-efficiency`: no decided verdict changed (64 rows);
the counters of the 42 rows of `wide` and `3-partition-mll-yes` are
equal, and those of the 22 Horn rows changed, by design, all downward:
`3-partition-mll-no/5` 16 202 533 nodes, 38 945 748 links to 227 and
425; `3-partition-mll-no/4` at test period 16 49 883 557 nodes to
1 012; `partition-yes/4` 235 737 to 485; the Partition table
`2-3-2-1` 344 946 to 692, `1-1-1-5` 560 182 to 840. It is the net set's
new oracle (`oopzxvzv`). After the routing (`net-after-routing`, the
same 5 minutes): every counter of `net-after-prunes` on all 64 rows, so
the balance's cadence and the routing left the net search as it was.
*The focused engine's set* after the routing (`after-routing`, about
6 minutes on cores 2 and 3, the LLTP library linked into the workspace
from the main checkout's store path) against `after-efficiency`: no
decided verdict changed (135 rows); counters differ on five: 
`partition-table/1-1`, which the dispatch now gives the focused engine
(7 stable sequents where the net engine chose 15 literals), and the
chain family's four, the one memo entry and hit that `after-generations`
recorded before this step. No row of it was decided by a handover
(no `net` engine where none was asked), so the two fixes after it
(`rmpwpwsu`, `qoszqxns`), which only change handovers, leave it as it
is; the net set forces its engine and has no handover.

### Commits

Each code commit passed the gate (`gate: passed`, on cores 4 to 7 in a
capped scope); all signed (the first two by `planning` after the
session committed them with signing off).

| change | subject |
|---|---|
| `vzslkryp` | Break the net engine's leaf and compound symmetries and balance its components |
| `nstkrpsr` | Check the net engine's balance only where its exact test runs |
| `oopzxvzv` | Make the net target set after the prunes its oracle |
| `punyzkxx` | Route unit-free MLL to the focused engines, the net engine at their limits |
| `nzvlkoxp` | State what the net engine's symmetry argument rests on |
| `qlrwqlyw` | Add both target sets after the routing |
| `rmpwpwsu` | Give back what the first search held before the net engine takes over |
| `qoszqxns` | Hand over to the net engine only where the linking is forced |
| `tvstokwp` | Say where a handover's work and its forced linking cost |
| the last | the rules trimmed to invariants and reasons, and this report |

### What is left open, and what 35b inherits

- **A stronger balance.** The skeleton-component balance never fires on
  the families. A per-atom Hall condition over the sets of literals
  joined in some switching (each such set pairwise unlinkable) would see
  more, at the cost of a properly coloured reachability per node; worth
  it only if the net engine ever earns a row back.
- **The focused engine's memory bound on `wide-m1` at 4 096**: it stops
  after two stable sequents (0.75 s), where `wide-m2` at the same size is
  proved in 0.78 s. Why the first takes more memory is a question for
  the focused engine's follow-ups; if it is fixed, the handover's
  `MemoryLimit` case loses its one measured row.
- **The handover's cost** is the first search's time to its limit; a
  cheap predictor of depth (the derivation height a chain of links
  needs) would skip it, and none is known.
- **The net engine on generated IMLL sequents with repeated literals**
  can take seconds in a debug build (the 212 s of the embedding test);
  the release harness did not meet one, since the dispatch no longer
  sends it such sequents.
- **The focused engine on a debug test thread**: the review of the
  prunes saw `Engine::Focus` overflow a 2 MB test thread in a debug build
  on the chain `|- x0, (~x0 * x1), …, ~x199`; the net engine proves it.
  Not touched here (the focused engine's follow-ups).
- **The symmetry tables are not charged** to `Limits::memory_bytes`, as
  the net engine's structure and scratch are not (about 24 bytes per
  occurrence more, documented on `Engine::Net`).
- **35b inherits** a net engine that is exact and much faster on Horn
  encodings but the default nowhere except at the focused engines'
  limits, and the measurement method: IMLL rows of essential nets must
  beat the two-sided engine, which decides the intuitionistic wide and
  curried sequents of 4 096 atoms in 82 ms and the Partition rows in
  0.08 to 76 ms, and loses only the chain at the recursion limit. The
  problem files of this measurement (`imll.txt` and `mix.txt`) are made
  by a short script recorded in the session's scratchpad; the chains,
  wide and curried sequents are those of step 26's report.

## Session 35b: essential nets for IMLL

**Outcome.** linlog decides essential nets: `ProofStructure::is_essential`
reads the structure of an intuitionistic sequent as Lamarche's polarized
structure (the reading directs every edge from the hypotheses toward the
goal) and decides it by directed acyclicity and the dominator condition,
with witnesses; on every linking of generated IMLL sequents it gives the
Danos–Regnier verdict. The essential-net engine (`Engine::Essential`,
`--engine essential`) is the net engine's linking search under that
criterion, kept incrementally by a transitive closure of bits with an
undo log, charged to the memory bound. It agrees with the two-sided
engine and with the embedding on generated sequents, every proof through
the checker and every net the net of its proof. Measured by 35a's method
it wins one row against the two-sided engine (the 64-link chain, 0.18 ms
against 0.22 ms) and loses every other, and at the two-sided engine's
recursion limit the net engine, which takes those goals over, is 2.2 and
6 times faster than it; so the dispatch is unchanged and the engine is
one a user names. It is far better than the net engine on the Horn
encodings (Partition 16 to 660 times faster), where its directed
rejections cut what the switchings see only after the links. The drawing
of an essential net is in the SVG export (`svg::Style::essential`,
`--style svg.essential=true`), with two snapshots. To make room for the
engine without a second copy of the search, the net engine's loop became
generic over a correctness criterion; the classical search is unchanged
(every counter of its target set, its journey's instructions).

The panel of step 26 reviewed the criterion, the generic loop and the
engine: the argument (Fable 5.1) and the counterexamples (Opus 5.5)
refuted nothing; the integers and limits (Sonnet 5.5) found an abort
without a memory bound, now fixed (`uyztqwzl`).

### The criterion

`nets/essential.rs`. The edges: an output occurrence that is no root to
its parent, an input connective to each of its input premises, and every
link from its input literal (`~a`, always input under the reading) to its
output literal (`a`); the antecedent of an output `⅋` (`A ⊸ B`, stored
`A⊥ ⅋ B`) has none. Only an input `⅋` (a hypothesis `A ⊗ B`) has two
edges out, and in a complete linking only the goal has none, so in an
acyclic one every occurrence reaches the goal and no connectedness or
count condition is needed. A complete linking is an essential net when
no directed cycle exists (an iterative depth-first search) and every
output `⅋` but the goal dominates its antecedent (a breadth-first search
from the antecedent that never enters the implication must not reach the
goal); the goal's own implication is exempt, every path to the goal
ending there. The witnesses are `NetError::DirectedCycle` (each vertex
followed by its successor) and `NetError::NoDominator` (the implication
and the path around it), with `NetError::NotIntuitionistic` for a
sequent without a reading; all three describe themselves with formulas
and have a JSON form (`directed_cycle`, `no_dominator`,
`not_intuitionistic`). The memory is linear, `stop` asked once per
implication. `ProofStructure::reading` is the structure's reading (R81:
recomputed per call, since a structure owns its forest and a reading
borrows one).

*Why it is the Danos–Regnier verdict*: Lamarche's theorem (an IMLL
structure sequentializes as an IMLL proof iff its essential net is
acyclic and dominated), with the fact the embedding already rests on
(every sequentialization of a classical net of an IMLL sequent is
intuitionistic). The test `agrees_with_the_switchings` checks the
equality on every linking (up to 120 per sequent) of 400 generated IMLL
sequents and their mutants, the dominator condition against an
enumeration of every simple path from the antecedent, and every witness
(a cycle follows edges and closes; a path starts at the antecedent,
follows edges, avoids the implication and ends at the goal); its
thresholds require more than 300 nets, 300 cycles and 100 failures of
the dominator condition among the structures. `witnesses` pins one of
each kind and the description of a path.

### The search

*The loop.* The net engine's `Linker` became `Linker<'a, C: Correctness>`
(R105): the loop keeps the choice, the frames, the cadence, the stop,
the counters and the cubes, and the criterion answers `mate`, `admits`
(its constant-time rejections), `link` (fallible, for a bound), `unlink`,
`test` and `balanced`. `Switching` holds the classical structure and its
exact test, its methods the code the loop had inline, in the same order.
The ratchet's net journey stood at 260 221 535 instructions against its
ceiling of 260 393 363 (-0.07 %) at that commit, and the net target set
kept every counter. The cubes stay `Linker<Switching>`'s: the essential
engine runs on one thread (`Engine::parallel` false), since it measured
no row worth a pool.

*The criterion of the search.* `Essential` keeps the reflexive transitive
closure of the directed graph as rows of bits (`n·⌈n/64⌉` words, row `u`
what `u` reaches). A link from `i` to `o` ORs row `o` into every row that
reaches `i` (exact for one new edge; row `o` is never written, since `o`
does not reach `i`), logging every changed word with its old value after
a mark; an unlink restores them in reverse. Rejected at once: a link
whose output literal reaches its input literal (a directed cycle, kept
by every completion; so the graph stays acyclic), and a link whose input
literal lies in the antecedent of an implication `l` by input premises
alone while its output literal reaches the goal and not `l` (a path from
the antecedent around `l`, kept by every completion). At the cadence of
the net engine's exact test and on every complete linking, the dominator
test: for each implication `l` but the goal, with antecedent `s`, it
fails when `s` reaches no `l` but reaches the goal or something `l`
reaches, or when some input `⅋` between `s` and `l` has a premise that
reaches no `l` but reaches the goal or something `l` reaches. Sound on
partial linkings, since what `l` reaches (and the goal) can never reach
`l` again in an acyclic completion, where it reaches the goal; exact on
complete ones, by the lemma that every path from `s` to the goal passes
`l` iff `s` reaches `l` and every successor of a vertex between them
reaches `l` (only an input `⅋` has two successors). The complete linking
that passes is sequentialized through the classical structure, as the
net engine's is; a `debug_assert!` checks it with `is_essential`.

*The symmetry breaks* are the net engine's, unchanged, which the API
notes (10.7, "the symmetry groups are the criterion's") warned against:
an exchange of two equal leaves of one `⅋` tree is no automorphism of
the directed graph (in `a ⊸ a ⊸ b` the two `~a` sit at different
implications, one bound by each). Why each still holds for essential
nets: the orbit argument of 35a needs of the criterion only that every
exchange maps correct complete linkings to correct ones and incorrect to
incorrect, and that the canonical net's partial linkings pass every
rejection. *Leaf classes* (equal literals that are conclusions or leaves
of one pure tree) and *compound classes* (equal subformulas there) are
built from the forest alone; an exchange keeps the sequent, so it keeps
its reading, its one output root and every polarity (equal literals
have one sign, hence one side), and maps a complete linking to a
complete linking of the same intuitionistic sequent; there the
essential verdict is the Danos–Regnier one (the criterion's section),
which the exchange preserves, so it maps essential nets to essential
nets. The canonical linking of an essential net's orbit is therefore an
essential net, and its partial linkings pass the cycle and binder
rejections and the dominator test, since each rejects only partial
linkings with no correct completion. Which implication binds which
literal changes under an exchange, but no rejection is asked to be
invariant, only sound. *The balance prune* is not used: the criterion
keeps no skeleton (`balanced` is true). Checked beyond the argument: the
argument reviewer re-derived it and ran the engine with every prune
against the enumeration of every linking of 42 872 small sequents (no
net lost), and the counterexamples reviewer compared the engine's
verdict with "some linking is a net" on 5.8 million linkings.

*Memory* (R35): the closure and the criterion's linear tables are
charged to the search's account before the matrix is allocated, in
checked arithmetic (a matrix past a 32-bit `usize` or past the bound is
`Reason::MemoryLimit`); the log grows by chunks, each charged before it
is reserved, and a refusal in the middle of a link restores the words it
changed and stops the search. `the_closure_is_charged` pins the unknown
at 2 000 bytes and the proof at 1 MiB; `unlinking_restores_the_closure`
compares the closure after every unlink with a copy taken before the
link.

*The tests* (`search/essential.rs`): `agrees_with_the_two_sided_engine`
decides generated IMLL sequents and their mutants with no literal more
than three times (a linking search is exponential in a literal's copies:
on one generated sequent with three atoms some fifteen times each, both
net engines ran 20 s in release without an answer where the two-sided
engine proved it at once) with the essential, the two-sided and, where
no literal is there more than twice, the net engine; every proof
checked, the net of every essential proof read back from the proof
(`from_proof` gives its links, R128) and accepted by `is_essential`.
On the committed seed it compares 442 sequents (253 provable), 337 of them with the net engine too. `refusals` pins the refusals (classical and affine mode, units).
The reference prover's comparison (`search/reference.rs`) runs the
engine forced among its configurations.

### The measurement

By 35a's method: the harness at the drawing's commit (release), one
thread, cores 2 and 3 under the machine lock, 60 s per run, runs under
2 s three times (median), 35a's problem file of chains, wide and curried
sequents of 64 to 4 096 atoms, and Partition in intuitionistic mode;
11:28:44 to 11:33:14, one unit, peak 425 MB.

| row | two-sided | net | essential | essential / two-sided | essential / net |
|---|--:|--:|--:|--:|--:|
| chain 64 | 0.219 ms | 0.255 ms | **0.182 ms** | 0.83 | 0.71 |
| wide 64 | 0.108 ms | 0.556 ms | 0.398 ms | 3.7 | 0.72 |
| curried 64 | 0.090 ms | 0.256 ms | 0.204 ms | 2.3 | 0.80 |
| chain 256 | 1.74 ms | 1.74 ms | 2.38 ms | 1.4 | 1.4 |
| wide 256 | 0.792 ms | 7.47 ms | 5.17 ms | 6.5 | 0.69 |
| curried 256 | 0.768 ms | 2.83 ms | 2.31 ms | 3.0 | 0.82 |
| chain 1 024 | recursion limit after 11.6 ms | 25.3 ms | 54.4 ms | | 2.2 |
| wide 1 024 | 6.18 ms | 125 ms | 90.2 ms | 15 | 0.72 |
| curried 1 024 | 5.71 ms | 46.1 ms | 41.8 ms | 7.3 | 0.91 |
| chain 4 096 | recursion limit after 45.9 ms | 0.41 s | 2.47 s | | 6.0 |
| wide 4 096 | 81.6 ms | 2.10 s | 2.06 s | 25 | 0.98 |
| curried 4 096 | 83.1 ms | 0.79 s | 1.43 s | 17 | 1.8 |
| `partition-yes/4` | 0.083 ms | 1.70 ms | 0.105 ms | 1.3 | 0.06 |
| `partition-yes/5` | 0.104 ms | 27.1 ms | 0.185 ms | 1.8 | 0.007 |
| `partition-yes/6` | 0.150 ms | 451 ms | 0.681 ms | 4.5 | 0.0015 |
| `partition-yes/12` | 12.8 ms | out of time | out of time | | |
| `partition-no/3` | 0.072 ms | 1.33 ms | 0.072 ms | 1.0 | 0.05 |
| `partition-no/4` | 0.141 ms | 8.88 ms | 0.192 ms | 1.4 | 0.02 |
| `partition-no/9` | 75.4 ms | out of time | out of time | | |

*The decision* (item 3 of the plan): the dispatch is unchanged. The one
row the essential engine wins is the smallest chain, by 37 µs; the chain
of 256 links, the wide and curried sequents of the same 64 atoms and
everything larger it loses, so no feature of the dispatch (fragment,
mode, a size, the forced linking) singles the row out without taking
rows it loses. The handover at the two-sided engine's limits stays the
net engine's, which is 2.2 and 6 times faster there. *Why it loses*: on a
forced linking the closure costs a column scan of `n` rows and an OR of
`n/64` words into every row that reaches the input literal, per link,
which is quadratic per link on a long chain, where the net engine's
Yeo test runs every fourth link; the two-sided engine needs no linking
at all on these shapes. *Why it beats the net engine on Partition*: a
directed cycle or a path around an implication shows on the closure as
soon as the link that makes it is tried (`partition-yes/6`: 0.68 ms
against 0.45 s), where a switching cycle needs the exact test and often
more links.

### The drawing

`svg::Style::essential` (off by default, so every drawing before it is
byte for byte the same; the snapshots `net.svg`, `cycle.svg` and
`disconnected.svg` did not change) draws a structure whose sequent has
an intuitionistic reading as its essential net, on the proof net's
layout: every edge an arrowhead halfway along it, from the hypotheses
toward the goal (the hypotheses' edges below enter upward, the goal's
leaves downward, each link's arrowhead at the top of its arc toward its
output literal), the input occurrences, their edges and their literals
in the colour `Style::input` (`#009e73`), the nodes as the reading's
connectives (`⊸` for an output `⅋` and an input `⊗`, `⊗` for the
others), every literal as its atom, and each implication joined to its
antecedent by a dotted line in `Style::dominator` (`#cc79a7`): the edge
the essential net lacks, which the implication must dominate. A
structure that is no essential net has `is_essential`'s witness in the
highlight colour (the cycle, or the path and the implication's dotted
line). A sequent without a reading is drawn as without the option, so
one settings file serves both kinds of nets. Snapshots:
`essential.svg` (`a ⊸ b, b ⊸ c ⊢ a ⊸ c`) and `essential-around.svg`
(`⊢ (a ⊸ b) ⊗ (b ⊸ a)` linked across, its path highlighted).

### Options (D15, D16, D17)

- `Engine::Essential` is a value of `Options::engine`: `--engine
  essential` in the command, `"essential"` in a settings file, in the web
  front end's JSON and in the harness's `--engines`. Its default is not
  to run (the dispatch gives it nothing), which is the safe one.
- `Options::test_period` now sets the cadence of the essential engine's
  dominator test too (`--test-period`); its default is the net engine's.
- The bound on its closure is `Limits::memory_bytes` (`--memory-limit`),
  whose default (1 GiB) holds the closure of about 90 000 occurrences;
  past it the verdict is unknown, before anything is allocated.
- `svg::Style::essential`, `input` and `dominator` (`--style
  svg.essential=true`, `svg.input`, `svg.dominator`; `styles.svg.…` in a
  settings file and the web front end's JSON), with `with_*` builders;
  defaults off, `#009e73`, `#cc79a7`, the same in `Style::dark()`.
- Quantifiers (D17): nothing here fixes an interface for them; the
  essential net is propositional like the proof net.

### The target sets

Both under the machine lock, on cores 2 and 3, the LLTP library linked
into the workspace from the main checkout's store path. *Net set*
(`net-after-essential`, 11:34:07 to 11:34:42) against
`net-after-prunes`: no verdict differs, and 0 of the 64 rows both decide
differ in `nodes`, `links` or `tests`. *Focused set* (`after-essential`,
11:35:14 to 11:43:02) against `after-efficiency`: no verdict differs;
counters differ on 5 of 135 rows, exactly the five 35a's report explains
(`partition-table/1-1` and the chain family's four); against
`after-routing`, 0 of 140. Committed in `ynuokwvo` with their entry in
`bench/TARGETS.md`; neither is a new oracle.

### The reviews

The panel of step 26, each member in a fresh context with the diffs, the
claims and a frozen copy of the tree but not this session's reasoning,
under the machine rules (cores 4 to 7, capped scopes, bounded runs), on
`kvnxpwko`, `lrymttrx` and `tqovoyop` (the criterion, the generic loop,
the engine).

**The argument** (Fable 5.1, `high`): wrote the invariants from the code
before reading the argument, then checked it at every site: not refuted.
It confirmed that the criterion is Lamarche's for this polarization (an
edge from an antecedent to its implication would change nothing: either
it closes a cycle or the path around it is a dominator failure), that a
balanced intuitionistic sequent always meets the count equation
(`c = t − p + 2g` for `g` output roots), the soundness of both
rejections and the dominator test and its exactness, the closure's
update and undo (also when the log's reservation fails mid-link, which
it drove), the transfer of the symmetry breaks, and that the generic
loop is the classical search line for line. Experiments: on every
complete linking (692 388) of every balanced sequent of 42 872 small
ones `is_essential` equals `is_correct`, and the engine at three
cadences, the net engine and the two-sided engine each give the
enumeration's verdict. *Answered*: its four unsaid steps (the reading's
`expect`, `is_complete` resting on the balance, the count equation
never rejecting in intuitionistic mode, `oriented` feeding the binder)
are in `core-nets.md`; its wording nit was in the argument file only.

**Counterexamples** (Opus 5.5, `high`): wrote its own two-sided IMLL
prover from the calculus (24 sequents decided by hand first) beside the
committed reference, and compared 160 819 small sequents with both and
6 559 larger ones with the two-sided engine, the essential engine at
cadences 1, 2, 4 and auto; checked `is_essential` against `is_correct`
on 5 816 139 complete linkings; checked every rejection of the
incremental parts on 8.9 million partial linkings (6.3 million
constant-time rejections and 1.7 million dominator rejections, each
sound), the closure against a recomputation at every state, memory
bounds on 1 500 sequents at 16 bounds, 5 335 stops and 2 800 work
bounds: not refuted. Of three planted faults its comparison caught two;
the committed tests caught one of them, not the other (the dominator
test without the input `⅋`s). The third (escapes into what the
implication reaches dropped) only weakens pruning and no verdict test
can see it. *Answered* in `uyztqwzl`: both witnesses it gave are now
`the_review_s_witnesses` (`a -o (a * a) |- (a -o a) * a` unprovable,
`|- b -o (a -o (b * a))` provable); the pruning-only fault stands on the
argument, which two reviewers confirmed (a follow-up below).

**Integers and limits** (Sonnet 5.5, `high`): every index, cast and
charge bounded and driven: 470 400 memory-bound runs with the log
growing mid-search, 396 refusals mid-link rolled back exactly, 1.9
million runs with the stop firing at every poll, 960 000 decided runs at
eight cadences against the two-sided engine, the drawing's arithmetic
over 37 500 styles: refuted once. Without a memory bound
(`--memory-limit none`, or `Limits::memory_bytes: None`) a 987-byte JSON
sequent of 33.5 million occurrences made the closure ask for 140 TB and
the process abort (`memory allocation of 140737484161024 bytes failed`),
reproduced here. *Answered* in `uyztqwzl`: the closure and the log are
reserved fallibly, a refusal is `MemoryLimit`, which without a bound now
reads "the memory the allocator would give was used up" with no flag to
raise (the witness ends so after 0.33 s). Its notes: the set-up did not
poll the stop (now every 1 024 rows); `is_essential` reported 25 bytes
per vertex where it can hold about 60 (`ESSENTIAL_BYTES` now counts the
doubling stack and queue); `limit_bytes` in the command's output was the
account's remainder (the library's `prove_within` reports the bound
given, `the_closure_is_charged`; the command's path is the other
engines' too, a follow-up); and a 1 s stop overshot by 5 s on a sequent
of 65 000 `a` and `~a`, which is the shared loop's `choose`, quadratic
in an atom's literals before the node's poll, the net engine
overshooting the same input by 23 s (a follow-up, the loop's).

The fix itself is not a change of the search or the criterion (the
allocation, a poll, a constant, a message), so it was not reviewed
again; its witness was rerun.

### Commits

| change | subject |
|---|---|
| `kvnxpwko` | Decide essential nets independently of any search |
| `lrymttrx` | Let the net engine's linking search take its criterion as a parameter |
| `zkxmklzx` | Raise render-svg's ceiling by 2.5 %: the heap layout the essential-net engine's code leaves |
| `tqovoyop` | Add the essential-net engine for IMLL |
| `tkmxmrkn` | Draw essential nets in the SVG export |
| `owsqvswo` | Say what the essential-net engine measured |
| `ynuokwvo` | Add both target sets after the essential-net engine |
| `uyztqwzl` | Answer the panel on the essential-net engine |
| `zxywzsvo` | Restore render-svg's ceiling |
| `qrmxzuqp` | Link the essential-net module's documentation to the reading by its path |
| the last | the rules and this report |

Every code commit passed the gate (`gate: passed`, in the workspace
`../linlog-step35b-gate`, on cores 4 to 7 in a capped scope):
`kvnxpwko`, `lrymttrx`, `tqovoyop` (first refused by the ratchet on
`render-svg`, +2.53 %, which `zkxmklzx` answers: per function under
callgrind every count of the SVG writer equal to the instruction, the
difference glibc's allocator, 460 calls of `malloc_consolidate` against
156; then passed), `tkmxmrkn`, `owsqvswo`, `uyztqwzl` and `qrmxzuqp`
(the first `nix flake check` at the report's commit failed in the
private documentation, an intra-doc link of the essential-net module
that rustdoc did not resolve; the gate builds no documentation). The ceiling
commits and the target sets are no code. Signed up to `ynuokwvo`; the
passphrase's cache ran out after it, so `uyztqwzl`, `zxywzsvo`,
`qrmxzuqp` and the last were committed with signing off, for `planning` to sign. `nix flake
check` runs at the last commit, which this report is in; its exit status
is in the last message to `planning`.

### What is left open

- **A row for the essential engine**, if one is wanted: it would need a
  cheaper closure. The matrix over every occurrence is what costs on
  long chains; the paths that matter run between literals, so a closure
  over the input literals alone (about a quarter of the occurrences, a
  sixteenth of the bits) with the fixed parts precomputed would cut the
  per-link work by about that factor, and a column index (the ancestors
  of each literal) would replace the scan of every row. Not done: no row
  asked for it, and the two-sided engine leads by 2 to 25 times.
- **The essential engine on a pool**: the cubes are the classical
  criterion's (`net::parallel` on `Linker<Switching>`); with a closure
  per worker (the spec's plan) it would run there unchanged in form.
- **Repeated literals**: both linking searches are exponential in a
  literal's copies where the two-sided engine's counts decide at once; a
  balance over the essential net (what one input literal's chain can
  reach) might prune there, unexplored.
- **R123's comparison on the libraries**: `is_essential` and the engine
  were not run over the ILTP and LLTP problems that are unit-free IMLL;
  that run was not named in this step and was not asked for.
- **The shared loop's poll latency**: `choose` counts every unlinked
  literal's admissible partners before a node's poll, quadratic in one
  atom's literals (the net engine overshot a 1 s stop by 23 s on a
  sequent of 65 000 `a` and `~a`, the essential engine by 5 s). Bounding
  the count by the stop, or polling inside it, is the loop's follow-up.
- **The pruning-only part of the dominator test** (escapes into what the
  implication reaches) has no test that would fail without it; a test of
  `dominated` on a partial linking whose only escape is such a vertex
  would pin it.
- **The command's `limit_bytes`** after a memory limit reached is the
  account's remainder where the library reports the bound given; it is
  the other engines' path too.
- **The drawing's description** is the net's text form, which states the
  Danos–Regnier verdict (the same one); a wording in the essential
  criterion's terms would need a text form per criterion.
