# Step 27 report: Horn programs, an engine for Petri nets

The first of the step's two sessions (Opus 5.5 at `xhigh`), run
unattended overnight. It built the Horn engine (reachability of a Petri
net's markings, the proof read off the firing sequence), measured it
against the forward focused search on the library's 3 137 nets, gave it
the row of the dispatch the measurement supports, and had its
refutations reviewed by a panel. The second session starts from this
report and its review: coverability in affine mode and the qcover
suite.

## Outcome of the first session

- **The Horn engine exists and is the default for Horn programs with a
  clause under `!`** in linear mode, classical or intuitionistic, with or
  without Mix (`Engine::Horn`, `core/src/search/horn/`, a row of
  `search::DISPATCH` with the feature `PetriNet`; `--engine horn`). It
  reads the goal as a Petri net, searches its markings (kept once each,
  sparse, nearest the target first), builds the proof from the firing
  sequence, which the checker accepts in every mode, and refutes when the
  reachable markings are exhausted or the atom counts rule the goal out.
- **On the library's 3 137 Petri nets at 5 s it decides 3 026, where the
  forward focused search decides 1 628** (the second baseline's default
  decided 1 520): 1 400 nets only the Horn engine decides, 2 only the
  forward search, no verdict against the other, every proof checked. On
  the 1 626 both decide it takes 0.23 ms in the median against 1.2 ms and
  is faster on 1 013; it decides 2 670 nets within 10 ms, the forward
  search 1 103. By the D19 rule the row is earned; the step goes on to
  its second session.
- **Its refutations passed two panels** (three reviewers each, every one
  trying to refute them one way): none was refuted. Of what the first
  found besides, five things are fixed and committed: an expansion that
  cost every place, the command running the one-thread search a second
  time beside itself, sizes that could wrap on 32-bit targets, infinite
  nets that the atom counts refute run to the limit (the counts now come
  first), and gaps in the written argument (an independent induction
  now covers the whole shape the engine accepts); three are follow-ups.
  The second, on the counts before
  the search and the claims of no change beside it, found no wrong
  verdict either; its three findings of cost and scope are fixed.
- **Item 4 is done**: the focused engine's follow-ups that only served
  nets (the restart of a copy-bound level from the frontier, the tuning
  of the unit of work, the throughput of short clause bodies) are
  dropped in `plan/later.md`.
- **Left for the second session**: coverability in affine mode and the
  qcover suite (items 2 and 3). The suite is pinned and looked at
  (below): only 12 of its 176 files state an expected result.

## The checklist

| item | state | evidence |
|---|---|---|
| 1. reachability: markings as count vectors, transitions indexed by their input places, a visited set, a proof the checker accepts | done | `core/src/search/horn/`; `decides_the_horn_families`, `engines_agree_on_horn_programs` (every configuration against the reference on 300 random programs in three modes); every proof of the measurement checked |
| 1. the default only where it beats the forward focused search | done: the row takes Horn programs with `!` | "The measurement", `bench/defaults/horn-nets.csv` |
| 1. a panel for the refutations | done twice, not refuted | "The panels" |
| 1. the limits as refusals, each tested | done | `refuses_at_its_limits` (markings, token counts, proof nodes, memory) |
| 1. the reference test's `configurations` | done | `Engine::Horn` beside the others; Horn programs judged under a bound of polls |
| 1. the target set | done: 109 runs moved to the Horn engine (nets 53 → 85 decided); the 116 decided rows that kept their engine equal `after-panels.csv` counter for counter | `bench/targets/after-horn.csv` |
| 1. the families | done: 62 proved, 38 refuted, 23 unknown, no mismatch | `linlog-bench run --all-families --timeout 5` with the row |
| 2. coverability in affine mode | second session | |
| 3. the qcover suite, its flake package and reader | second session; the repository pinned and surveyed | "For the second session" |
| 3. the 176 instances measured with both engines | second session (affine) | |
| 4. the net-only focused follow-ups retired | done | `plan/later.md` |
| a rules file, its row in `core.md` | done | `.claude/rules/core-horn.md` |
| README for the engine | done, with the suite to come | the usage section's Horn paragraph, the built list |
| `nix flake check` | passed at `18e197d0`, the last code commit (every check, the tests in release and with debug assertions); also after `7e0b80ff` | |

## The engine

**What it takes.** A goal is a Horn program when, written one-sided with
the bodies' atoms of one sign, every member under `?` is a clause with
exactly one head, and every other member is a marking (a `⅋` of head
literals and `⊥`), a clause with exactly one head used once, or the
goal, a clause without a head, of which there is exactly one; a clause
is a tensor of body literals and `1` with at most one factor a head, a
`⅋` of head literals and `⊥` (`Program::read` in
`core/src/search/horn/mod.rs`). Two-sided that is Kanovich's !-Horn
sequent `W, Γ, !Δ ⊢ Z`: atoms, implications between tensors of atoms
used once and under `!`, and one tensor of atoms to reach. In
intuitionistic mode the reading must also put exactly the goal and the
clauses' bodies on the right of `⊢`. The test is stricter than
`schedule::chains`, which sets the forward search's bound and also takes
a goal under `?`, no goal or several goals; there the correspondence
with a net fails (`⊢ ~a, a, ~b, b` is reachable as a net and unprovable
without Mix).

**The net.** A place per atom the goal names, and one per class of
clauses used once with equal arcs, whose tokens are the class's unused
clauses: a clause used once is a transition that also takes a token of
its class, and the target holds none of them, so a firing sequence that
reaches it uses every such clause exactly once. Clauses under `?` with
equal arcs are one transition, and a clause whose inputs equal its
outputs is dropped.

**The search** (`reach.rs`). Greedy best-first over the markings: the
frontier holds successors as their parent and the transition, keyed by
the successor's distance to the target (the tokens by which the two
differ, updated per transition from the parent's), ties going to the
successors of the latest marking expanded; a successor is written out
only when taken, dropped if kept already, and otherwise kept and
expanded at once. The goal is tested when a successor is generated.
Markings are kept sparse: the marked places in order, each as its gap to
the last and its count less one in LEB128, so that a marking costs bytes
per token and the table compares bytes. The first version kept dense
`u32` vectors and stored every successor at generation; on
Philosophers-10000 (some 50 000 places) it met the 1 GiB bound after
4 096 markings, two expansions; the sparse store with successors kept
unwritten proves it in 0.14 s. The engine has no copy bound, memo limit
or recursion: a refutation is the exhaustion of a finite set of
reachable markings, and a net whose markings grow without end is
searched until the stop or the memory bound, which counts every buffer
that grows with the markings.

**The proof** (`proof.rs`), built after the search, premises first: the
goal's tensors over axioms with the tokens left, then each firing from
the last to the first (the clause's tensors over an axiom per body
literal and the proof so far above the head, wrapped in the head's `⅋`
and `⊥`; a `Copy` for a clause under `?`), then the markings' `⅋` and
`⊥` and a `Quest` per member under `?`. Which token each body literal
takes comes from a forward replay of the firings on stacks of
head-literal occurrences per place. Subtrees are walked in decreasing id
order (post-order, right factor first): nothing recurses over a formula.

**Before the search** the engine applies the atom counts that the front
door already computes for every engine's refutation (`focus::refutation`,
on a fork of the memory account): a goal whose atom cannot balance is
refuted at zero markings, where its net may be infinite (`|- b, ?~c`).
Atoms under `!` have no rows, so on a net proper the counts say nothing.

**Its limits**, each a refusal, all four driven to their limit by
`refuses_at_its_limits`: the markings kept (`MOST_MARKINGS`, their
indices are `u32`), a token count past `u32::MAX` (`IndexLimit`), the
proof's nodes (`MOST_NODES`), and the memory bound (`MemoryLimit`).

## Why a refutation is sound

The claim: a goal `Program::read` accepts is provable in linear mode
(classical with or without Mix, or intuitionistic) exactly when its
target is reachable in the net it builds; the search answers `Ok(None)`
only after every reachable marking has been expanded.

Call a subformula an output when it is the goal or lies in a body (a
body literal, a `1`, a tensor of such), an input otherwise (markings,
heads, the clauses and the tensors on a clause's path to its head, the
`?` members), and let `q` be the outputs of a sequent. Rule by rule, in
any cut-free proof of a goal of this shape: an axiom pairs a body
literal with a head literal (`q = 1`); `1` is an output (`q = 1`); a
tensor on a clause's path to its head gives the premise of each body
factor that factor (one output) and keeps the rest; a tensor of body
factors sums its premises' outputs and adds one for itself less one per
factor; `⅋`, `⊥`, `Quest`, `Copy` and the weakening of an input keep
`q`; Mix sums. So `q − 1` counts the Mix rules and the weakened outputs
above a sequent. The root has `q = 1`: no proof of such a goal uses Mix
or weakens an output, and every sequent of it has exactly one output, so
it is an ILL proof of the two-sided reading, and ILL proofs of Horn
sequents are firing sequences (Kanovich, "The complexity of Horn
fragments of linear logic", APAL 1995). Hence reachability decides the
goal in linear mode with or without Mix, classically and
intuitionistically, and a goal with no reachable target is unprovable.
The converse is the proof the engine builds, which the checker judges.
In affine mode an output may be weakened, the target need only be
covered, and leftover tokens and unused clauses weakened: the engine
refuses affine mode (`Error::EngineMode`), coverability being the second
session's.

## The panels

Every refutation is a change of the search, so the engine's went to a
panel as step 26's did: a workflow of three fresh agents, each given the
commit, the base tree, the claim and one way to refute it, on cores 12
to 15 in capped scopes: the counterexamples (Opus 5.5 at `high`), the
argument (Opus 5.5 at `xhigh`), the integers and limits (`sonnet`, which
`.claude/settings.json` maps to Sonnet 5.5, at `high`).

**First panel, on the engine (commit `a990d47f`): not refuted.**

- *Counterexamples.* An independent judge written for the purpose,
  sharing no code with the engine: exact reachability by enumerating the
  markings, clauses used once as tickets that must all be spent, and a
  Karp–Miller tree for boundedness; checked on 39 nets decided by hand
  and the counter families. 3 200 random programs in five to seven forms
  each (two-sided, curried, one-sided, mirrored with negated bodies,
  classical, Mix, intuitionistic), 1 to 8 places, duplicates, clauses
  equal up to factor order, empty bodies and heads, identities, weights
  to 4, goals reachable by construction and random: 19 400 Horn runs
  (6 631 proved, 10 702 unprovable, 2 067 unknown), no disagreement with
  the judge; every unknown on a net the judge showed unbounded. 300
  programs of up to 404 places (multi-byte gaps and counts): no
  disagreement. 4 004 runs of the focused engines on the programs the
  Horn engine did not prove: neither ever proved what it refuted. The
  counter with 2²⁰ tokens (about a million firings) proved and checked
  in 1.5 s.
- *The argument.* The invariant stated from the code (the kept markings
  are exactly those expanded, the frontier every enabled successor not
  taken, a successor dropped only when equal to a kept one, the encoding
  canonical, the table at most half full; `Ok(None)` only with the
  frontier empty, so the kept set is closed under firing). The count
  `q − 1 = Mix + weakened outputs` checked rule by rule over the
  checker's calculus, and, without relying on Kanovich, a direct
  induction from a one-output dyadic proof to a firing sequence that
  covers everything `Program::read` accepts. Gaps in the written
  argument (the tensor of body factors, the signs, binary Mix only, the
  translation to ILL, Kanovich cited for a wider shape): the rules file
  and this report now give the argument complete.
- *Integers and limits.* Every number argued or driven to its limit in a
  scratch copy built with overflow checks: the token counts (`IndexLimit`
  exactly at the boundary, reachable by honest input: a clause adding
  5 000 tokens a firing meets it after 859 166 markings), the frontier
  key for index 0 and the largest, LEB128 at every boundary, the
  incremental distance (about 100 000 random nets fuzzed against a
  dense recomputation and an independent search, the memory account
  equal to the buffers' capacities at every step), `MOST_MARKINGS` and
  `MOST_NODES` argued (unreachable on this machine).

What it found that was no wrong verdict, and what became of it:

| finding | by | done |
|---|---|---|
| an expansion summed the target over every place: a chain of 160 000 places took 6.4 s | argument, integers | fixed (`10790a62`): 0.17 s |
| by default the command ran the one-thread search a second time on a "pool" after 100 ms: twice the CPU, twice `--memory-limit`, statistics doubled | all three | fixed (`16b93610`): no pool beside an engine that runs on one thread |
| `nodes as usize * …` in the proof's bytes and doublings in the buffers could wrap on 32-bit targets | integers | fixed (`10790a62`): checked, a refusal |
| an infinite net that the atom counts rule out runs to the limit (`|- ?~a, b`) | counterexamples, argument | fixed in `d851430f`: the counts first (below) |
| the written argument has gaps | argument | completed (`core-horn.md`, above) |
| the frontier keeps duplicates; distinct clauses used once cost `n²` bytes; the frontier's growth asks a doubling | argument, integers | follow-ups (`plan/later.md`) |

**The counts before the search.** Making the engine the default turned
the panel's lost decision into a regression: three tests of the
focused engines met sequents such as `|- b, ?~c` and `c, !c |- a`, which
the counts refute at once and the Horn engine searched to the memory
bound. `Horn::decide` now calls `focus::refutation`, the test the front
door already applies to every engine's `Unprovable`, before it searches.
That is a new path to `Unprovable`, so it had a panel of its own.

**Second panel, on the counts before the search (`d851430f`) and the
claims that `10790a62` and the split of `prove_goal` change nothing: not
refuted.**

- *Counterexamples.* The first panel's judge checked again (39 hand
  cases), a generator that adds atoms the counts can rule out (only in
  the goal, only in markings, in a goal and a body, unbalanced clauses
  used once) and balanced variants: 1 500 programs in seven forms and
  modes, each run by the final binary forced and by default and by
  `a990d47f` forced, 10 500 pairs. The final binary proves exactly the
  1 813 pairs the judge finds reachable; none of its 8 183 refutations is
  reachable; on the 3 416 pairs both binaries decide by search every
  verdict and every counter is equal; the other 6 580 are refuted by the
  counts at zero markings (1 386 of them "unknown" before). 627 runs of
  33 sequents under 19 sets of flags (sequents that are no programs,
  units, Mix, intuitionistic, invalid calls): the same verdicts, exit
  statuses and error messages, the engine's name aside. Goals off the
  roots through `interact`: right.
- *The argument.* What a refutation other than `Exhausted` asserts, from
  the code: an atom has rows only if none of its literals lies under a
  `!` or `?` anywhere in the forest, and only the contraction and
  weakening of `?` formulas copy or drop literals, so in every cut-free
  proof each literal of such an atom meets exactly one dual, in every
  additive slice, Mix (binary) included; the count equation is on only
  without exponentials, where it is the standard count. This holds of
  every proof and of goals off the roots (the flags are the forest's,
  the fragment the goal's). `Horn::decide` passes the values
  `prove_goal`'s own refutation gets. The split of `prove_goal` keeps
  the errors in their order; `10790a62` computes the same numbers.
- *Integers and limits.* No wrap and no refusal that is not "unknown";
  `target_total` and the checked products equal the old values on 64
  bits; the tally's `i32` sums cannot pass their bound under any memory
  bound (a goal list of over 8 GB would be needed).

Its findings, none a wrong verdict:

| finding | by | done |
|---|---|---|
| the counts were charged to the search's own account and never given back: near its bound the search lost decisions (a chain of 100 000 clauses at 14 MiB, proved before) | all three | fixed: the counts have a fork of the account, which goes with them; the chain is proved at 14 MiB again |
| the row read the fragment the options assert, so `--fragment mell` sent programs without `!` (even `|- a, ~a`) to the Horn engine (Partition with 12 items: 4.1 s against 14 ms) | counterexamples, argument | fixed: the feature asks for a `?` member in the goal itself (`horn::is_net`) |
| "the test costs one pass" is wrong where atoms outside every `!` sit in nested tensors: their rows are quadratic (a goal of 20 000 distinct atoms meets the 1 GiB bound, which the forced Horn engine proved in 20 ms before; the focused engines pay the same on that sequent) | all three | the rules say so; a follow-up |
| the reference test's poll bound stopped the focused configurations too | argument | fixed: it bounds only the runs that can reach the Horn engine |
| when the counts refute, `prove_goal` computes them again for the reason; a stop between the two on a large forest names "exhausted" | argument, integers | documented; the verdict is right |

## The measurement

The library's 3 137 Petri nets (`ILL/petri-nets`, every one a theorem:
the goal is the marking after a replayed firing sequence of 1 to 100
steps) at 5 s, intuitionistic, one thread, on this step's binary
(`a990d47f`), with `--engines horn,two-sided --bias factors --copies
30`: the Horn engine (which reads neither flag) and the forward focused
search as `lltp-forward.csv` ran it, the two runs of a net back to back
on one core, four streams on cores 2 to 5 taking every fourth net. 35
minutes, run detached; `bench/defaults/horn-nets.csv`.

| | Horn engine | forward focused search |
|---|--:|--:|
| decided (all proved) | **3 026** | 1 628 |
| unknown: time limit | 90 | 1 409 |
| unknown: memory limit | 21 | – |
| unknown: copy bound | – | 100 |
| decided only by this one | 1 400 | 2 |
| verdicts against the other | 0 | 0 |
| proofs the checker rejected | 0 | 0 |
| median time where both decide (1 626) | **0.23 ms** | 1.19 ms |
| faster on, where both decide | 1 013 | 613 |
| decided within 10 ms / 100 ms / 1 s | 2 670 / 2 965 / 3 014 | 1 103 / 1 359 / 1 507 |
| time of the 1 626 both decide, summed | 4.7 s | 352 s |

By the length of the firing sequence the goal was made with (the
problem's `_N_1`), nets decided by the Horn engine and by the forward
search:

| steps | nets | Horn | forward |
|--:|--:|--:|--:|
| 1 | 609 | 609 | 608 |
| 5 | 609 | 608 | 437 |
| 10 | 399 | 398 | 265 |
| 20 | 600 | 587 | 188 |
| 50 | 496 | 450 | 70 |
| 100 | 424 | 374 | 60 |

Where the Horn engine loses: the forward search alone decides
`Solitaire_soli2_10_1` (4.2 s) and `PaceMaker_5_1` (1.7 s); on 131 nets
both decide the Horn engine is more than twice as slow and above a
millisecond, on 30 more than 10 ms slower, at most 213 ms
(`NeighborGrid_z_5d_4n_1m_t_3_5_1_1`, 256 ms against 43 ms): one-step
nets of thousands of places, where reading the program and writing
markings of thousands of tokens is the cost. The 111 nets it leaves are
deep goals in large state spaces where the token distance leads the
greedy order onto plateaus (BART, Peterson, PolyORBLF, GPPP, TokenRing,
QuasiCertifProtocol at the time limit) and nets of high branching whose
frontier meets the memory bound (DNAwalker, DatabaseWithMutex,
PermAdmissibility, Echo).

**The final engine** (`16b93610`, after the panel's fixes and the
counts before the search) ran the nets again, Horn only, the same way:
the same 3 026 decided, every verdict and every counter equal, every
proof checked, at 1.03 times the time in the median (the counts' pass).

**The target set** (`bench/targets.sh after-horn`, cores 2 and 3,
against step 26's `after-panels.csv`): 109 runs moved to the Horn
engine, the counter with 8 and 16 tokens and the unreachable counter
with 8 in both modes, and 91 sampled nets, of which it decides 85 where
the focused engine decided 53, in 28 s of CPU instead of 197 (6 left: 5
at the time limit, 1 at the memory bound). The 154 runs that kept their
engine have every counter of their 116 decided rows equal, no verdict
lost, gained or contradicted, every proof checked.

**The families**: on the Horn families against the default at 10 s
(`bench/defaults/horn-families.csv`), the counter with 64 tokens proved in
0.07 ms (the default at its time limit), the unreachable counter with 64
refuted in 0.6 ms (the default at its time limit, with 32 at its copy
bound); `partition-no` (clauses used once, no `!`) two to three times
faster at the same node counts; but `partition-yes` with 12 items 2.0 s
against 15 ms, and with 16 and 20 not within 10 s where the default takes
0.2 and 0.5 s. `linlog-bench run --all-families --timeout 5` with the
row: 62 proved, 38 refuted, 23 unknown, no mismatch, every proof checked
(step 26's review: 61, 36, 26).

The qcover instances are coverability problems, affine mode, which this
session's engine refuses: they are the second session's measurement.

## The decision about the default

D19: the default moves to the new engine exactly where the numbers show
it faster or deciding more. On Horn programs with a clause under `!`
they do, in every respect the measurement has: 1 400 more nets decided
against 2 fewer, faster in the median, the sum and the tail. The row
therefore takes them, in linear mode (classical, Mix, intuitionistic):
`Row { fragment: MELL, modes: Linear, feature: PetriNet, engine: Horn }`,
before the general rows (the additive and the net rows take nothing it
takes: they need additives or no exponentials). On Horn programs
without `!` the numbers go the other way on `partition-yes`, so the
feature asks for exponentials. The step does not end with this session.

## Decided unattended

Each open choice, what was taken and what was set aside:

- **The search order: greedy best-first by the token distance**, ties
  to the latest marking expanded, as the heuristic search of
  explicit-state net checkers does (TAPAAL's verifypn searches
  reachability best-first by default). Set aside: breadth-first (the
  shortest firing sequence and the smallest proof, but a level per step
  on goals 100 steps deep), depth-first (no direction), A* (shortest
  sequences again, at the frontier's cost). The order changes neither a
  verdict without a limit nor a refutation; it is the engine's
  scheduling, not an option (as the default bias's slices are not).
- **Sparse markings and successors written only when taken.** Dense
  `u32` vectors, the first version, met the 1 GiB bound on
  Philosophers-10000 after two expansions. Set aside: hash compaction
  (bit-state hashing), which saves more but can merge two markings and
  so make a refutation unsound.
- **The shape: Kanovich's !-Horn sequent with exactly one goal**,
  stricter than `schedule::chains`. Set aside: `chains`'s whole shape
  (a goal under `?`, none or several goals), where reachability is not
  provability (`⊢ ~a, a, ~b, b`).
- **Clauses used once as transitions with a ticket place per class**, so
  the Partition encodings are programs too. Set aside: clauses under `!`
  only (simpler, but no program of the families without `!` would be
  one, and the measurement of their row needs them).
- **Mix taken, affine mode refused** in this session: the count argument
  shows Mix changes nothing on these goals; affine mode is coverability,
  the second session's, and a forward search could refute only bounded
  nets there.
- **The row takes only programs with a clause under `!`**, measured:
  Partition without `!` is the focused engines'.
- **The intuitionistic reading must agree** with the engine's outputs
  occurrence by occurrence, so that its proofs pass the one-succedent
  check; a goal whose reading differs is no program (never seen in the
  library).
- **No options of its own.** The engine reads `memory_limit` and
  `check`; it has no copy bound (a firing sequence has any length), no
  memo limit (the visited set is what makes a refutation) and no
  recursion. Set aside: a bound on the firing sequence's length mapped
  from `copies` (the library's default of 3 would have cut nearly every
  net).
- **The counters reuse `Statistics`**: `nodes` the markings reached,
  `memo_hits` those reached again, `memo_entries` those kept. Set aside:
  fields and CSV columns of their own (a harness interface change for
  three numbers that fit).
- **`Error::NotHorn`** for a forced engine on another goal, and
  `EngineMode` in affine mode, as the other engines refuse.
- **The measurement's design**: four streams, every fourth net, both
  configurations of a net back to back on one core, so that the
  comparison is paired on cores of different speed (2 and 3 fast, 4 and
  5 slower).
- **The counts before the search** (the review's lost decision, which the
  row made a regression): `focus::refutation`, the front door's own test,
  rather than a check of the net's places written for the engine (no
  second implementation of a refutation).
- **No pool beside a one-thread engine**: `search::engine_for` asked by
  the command when the pool would start, so that it costs nothing on a
  search that ends first. Set aside: making the Horn engine read `jobs`
  (nothing to share between threads that would not repeat the search).
- **The qcover suite is the second session's**: its instances are
  affine, which only the second session's engine decides; this session
  pinned the repository and read its format and expected results.
- **Not run unattended, and not needed**: `bench/baseline.sh` (the
  machine was not declared free) and any probe outside cores 2 to 15.

## Options, front ends and quantifiers

No option was added (D15, D16). The engine is chosen by the dispatch,
forced by `Options::engine(Some(Engine::Horn))`, which the command maps
from `--engine horn`, the harness from `--engines horn`, and the web
front end would hold as the engine's name, `"horn"`, in the options
value it already keeps (`Engine`'s `Display` is the name in text and
JSON). Its bounds are the existing `Options::memory_limit` (`--memory-
limit`) and the caller's stop (`--timeout`). `search::engine_for` and
`Engine::parallel` are public queries for a front end that schedules
threads, as the command does.

Quantifiers (D17): a Horn program is propositional; first-order Horn
clauses would be a different engine (resolution over terms, not a net),
and the place for them is a row of its own, beside this one, with its
own `Feature`. `Program::read` is the one place that reads the shape;
the search and the proof never see formulas but through it.

## Deviations and assumptions

- **"Markings as count vectors"** are count vectors kept sparse (the
  marked places with their counts, in LEB128), since dense vectors
  could not hold the library's widest nets; the vector is dense only for
  the marking at hand.
- **The refutations have no qcover judge yet**: the suite is affine.
  Their judges this session were the families with known verdicts
  (`counter-over`, `partition-no`), the reference prover (which proves
  where the Horn engine must not refute), and the panels' independent
  Karp–Miller judge.
- **The measurement ran by night**, on the cores the session's start
  named (2 to 5), not by day.
- **The second panel reviewed a change made after the first**: the counts
  before the search, with the claims that the expansion fix and the
  split of `prove_goal` change nothing.
- **Tests that pinned the focused engine's answers on Horn-shaped
  sequents now name that engine** (`compact_is_the_whole_with_its_runs_merged`,
  `outcome_json_format`, `prove_verdicts_and_exit_statuses`), and the
  dispatch tests pin the new row: the behaviour they pinned, the
  dispatch, was meant to change; what they test besides is unchanged.
- **`plan/README.md` is unchanged**: D8's table predates the dispatch as
  data, which `Engine`'s documentation now holds with the Horn row; the
  planning session updates the plan.

## For the second session

- **Coverability (item 2).** The engine refuses affine mode today
  (`Horn::admits`); the backward algorithm on upward-closed sets goes
  beside `reach.rs` in `search/horn/`, on the same `Program` (whose
  ticket places for clauses used once mean "at most once" in affine
  mode, the target then a lower bound). The proof of a covering marking
  is the reachability proof with the leftover tokens and unused clauses
  weakened. The count argument of `core-horn.md` extends: in affine mode
  `q − 1` counts the weakened outputs too, so with one goal none is
  weakened and the proof is an affine ILL proof.
- **The qcover suite (item 3).** `github:blondimi/qcover`, last commit
  `39d3163b99ece5771f200515b22a2d588f400aa5` (2021-03-03), `narHash`
  `sha256-MWLnMo0gdrhe0uMDx1GNZg6s9OdlcH8lupv01Qv0nwc=`, 442 MB; the
  repository is Apache-2.0 (its `LICENSE.md`), the benchmark files carry
  no statement of their own. The 176 instances are
  `stable/benchmarks/{mist (27), soter (50), wahl-kroening (46), medical
  (12), bug_tracking (41)}/**/*.spec` (and `examples/lamport/model.spec`
  besides). The format: `vars` (names), `rules` (each `guards ->
  updates;`, a guard `x >= k`, an update `x' = x+k` or `x-k`), `init`
  (`x = k`, or `x >= k`: a parametric marking, any number at least `k`,
  which the step reads as `!x` beside `k` tokens), `target` (several
  lines, each a conjunction of `x >= k`: the instance is unsafe when any
  line is coverable). **Only 12 files state an expected result** (Mist's,
  a first line `#expected result: safe` or `unsafe`: 11 safe, 1 unsafe);
  the assessment's "expected results in the files" holds for those
  alone. The other 164 need another judge: the results of the papers
  (61 unsafe and 115 safe in all, FastForward's count) or the panel's
  independent coverability check. The soter files reach 914 053 lines.
- **The Horn engine's follow-ups** are in `plan/later.md` ("Follow-ups:
  the Horn engine"): the 111 nets it leaves (the state equation,
  partial-order reduction), refutation beyond exhaustion, the frontier's
  duplicates, the clauses used once in every marking, a shortest firing
  sequence for teaching, the classical library's nets unmeasured.

## The commits

In order, each signed unless it says otherwise:

1. `a990d47f` Add the Horn engine: Horn programs decided as Petri-net
   reachability (the engine forced by `--engine horn`, its tests, the
   reference comparison, the rules file).
2. `10790a62` Keep a Horn expansion's cost to its marked places and
   check its buffers' sizes (the first panel's findings).
3. `d851430f` Make the Horn engine the default for Horn programs with a
   clause under `!` (the row, the measurement's data, the counts before
   the search, `engine_for` and `Engine::parallel`, README, the tests
   that now name the focused engine).
4. `16b93610` Add no pool beside an engine that runs on one thread (the
   first panel's finding in the command).
5. `dbd78605` Retire the focused follow-ups that only served nets, and
   list the Horn engine's (`plan/later.md`, item 4).
6. `3cf2f3ac` Count the Horn engine's rules file in CLAUDE.md.
7. `7e0b80ff` Record the target set with the Horn engine as the default
   (**unsigned**: the passphrase's cache ran out at 01:15).
8. `18e197d0` Give the Horn engine's count test an account of its own and
   its row only nets (the second panel's findings; **unsigned**).
9. This report and the second panel's follow-ups in `plan/later.md`
   (**unsigned**).

The unsigned commits are signed, once the passphrase is entered, by

```sh
jj sign -r 'main@origin..@-'
```
