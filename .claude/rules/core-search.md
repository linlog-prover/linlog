---
paths:
  - "core/src/search/mod.rs"
  - "core/src/search/memory.rs"
  - "core/src/search/work.rs"
  - "core/src/search/additive.rs"
  - "core/src/search/reference.rs"
  - "core/src/search/refutation.rs"
---

# linlog core: proof search, its front door, its memory bound, the additive path and the reference prover

Loaded, beside `core.md`, when the search's front door, its memory
account, the additive path or the reference prover is read or changed. The engines have files of their
own: `core-focus.md`, `core-nets.md` (the net engines, classical and
essential) and `core-parallel.md` (the pool).

## Proof search: the front door

`search/mod.rs` is what a front end calls: `prove(&sequent, mode,
&options)` and `prove_within(…, &limits, stop)` return `Result<Outcome,
Error>`, and `prove_goal(goal, mode, &options, &limits, stop)` decides
a `Goal` (`Copy`: the forest and the members; `Goal::conclusion(&forest)`,
or `Goal::new(&forest, &members)`, which checks the members against the
forest and is `is_conclusion` for the roots in any order) of any
multiset of a forest's occurrences, `prove_within` being that on
the roots (the roots in any order are the roots, `Forest::is_roots`,
which `Proof::new_of_goal` asks too, and the
engines get them in the forest's order, so the net engine takes them and
the proof is checked): the goal's own fragment (`goal_fragment`, over the subtrees)
picks the prunes and the engine, the net engine only for the roots
(`NotTaken::Goal` when forced elsewhere, since a structure's conclusions
are the forest's roots), the additive path for any two additive-only
occurrences (`additive::search_goal`), the focused engine otherwise; in
intuitionistic mode a goal must have exactly one occurrence in output
position (`Error::GoalOutputs`). **Every proof, of the roots or of a
goal, has passed the checker when it is returned, in every build,
unless `Options::check` is off** (`DEFAULT_CHECK` true; off, the proof
is the engine's word, `Outcome::checked` is false and the engine's
`debug_assert!` is all that runs on it; the check is in `conclude`, which
`prove_goal` calls, one place
for every engine, and a proof it rejects is `Error::Rejected`, an error
and never a verdict; a check that a bound or the stop gives up makes the
verdict `Unknown` (`unchecked`: `Reason::Unchecked { limit_bytes }` at
the memory bound, `WorkLimit` at the work bound, `Stopped`), never an
error, since a refusal is no fault of the input, and never a proof
returned unchecked). The engines' own `debug_assert!`s on their proofs
stay, and the flake's `test-debug-assertions` check is what runs them,
since crane tests in the release profile. The harness switches the check
off to time the search alone and checks the proof itself. The proof of a
goal other than the roots records the goal (`Proof::goal`, set by
`prove_goal` through `Proof::concluding`) and every proof the mode it was
found in (`Proof::mode`, a claim); the check is against the proof's own
`conclusion()`, and only `Interactive` consumes goal proofs, by grafting
their derivation. Where
`Outcome` carries the `Verdict` (`Proved(Box<Proof>)`, `Unprovable(Box<Disproof>)`,
the refutation with the sequent, the goal off the roots and the mode it
means something in (`search/refutation.rs`: `Refutation` with the
structs `Unbalanced`, `OneSided` (an atom whose literals below an
exponential all have one sign, which copies add, outnumbered outside
them the other way), `Equation`, `StateEquation`; its `Display` writes
atoms as `#0`, the disproof's by name, which the command prints)
only after an exhaustive search, which with exponentials means a
deepening level that never hit the copy bound, `Unknown(Reason)`, with
`Reason::CopyBound` when every level up to a bound hit it), the `Fragment` searched in,
the `Mode`, the `Engine` that ran, the `Statistics`, and `net`, the
`ProofStructure` the net engine found, boxed (`None` from the other
engines).
`Options` (`#[non_exhaustive]`) has public fields, each documented
with what reads it, and a `#[must_use]` `with_*` builder per field
(`engine`, `fragment`, `bias`, `copies`, `forward_copies`, `memo_limit`
a `u32`, `test_period` a `Cadence` (`Auto` or `Every(n)`), `jobs` a
`Jobs` (`Auto` or `Count(n)`, `From<usize>`), `schedule`, `check`, and
with `parallel` `pool`, the `search::Pool` of `core-parallel.md`), the
constants `DEFAULT_MEMO_LIMIT`, `DEFAULT_COPIES` (the library's default
bound; `copies` takes an `Option`, `None` for none),
`DEFAULT_FORWARD_COPIES`, which the CLI shows as its defaults, and
`MAX_JOBS` (256). **A clamp applies where a value is read**:
`Jobs::count` takes `Auto` as the machine's threads
(`available_parallelism`, one where the platform does not tell) and any
count to `1..=MAX_JOBS`; the engines read `Options::threads()` and
`memo_entries()`, never the fields' raw values. `Schedule::Turns` makes
the default bias's pair take turns on the calling thread even with
`parallel` (`schedule::turns`; `Auto` alternates in slices where a
thread starts), so its polls and answer are the same on every build;
`default_bias_takes_turns` pins that the front door's turns are the
function's. **The JSON form** (`serialize/search.rs`, `serialize::auto`):
`serde(default, deny_unknown_fields)`, `engine` and `fragment` a name or
`"auto"`, `bias` and `schedule` a name, `copies` a number or `null`,
`test_period` and `jobs` a number or `"auto"`, the pool never written;
`options_json_format` pins it. `Engine::NAMES`, `Bias::NAMES` and their
`FromStr` are the words. **The bounds are not options**: they
are `crate::Limits` (`limits.rs`, `memory_bytes`, `occurrences`,
`recursion_depth`, `derivation_bytes`, `work`), the argument every long
call takes beside its stop (`FnMut(Progress) -> bool`), whose defaults
the CLI shows as its own; `Limits::stack_bytes()` is the stack a thread
needs at the recursion depth, which sizes the CLI's search thread, the
parallel pool's workers, the default bias's second search and a batch's
workers alike. **Every poll is told the work done** (`Stop::fired(units)`,
the engines' `&mut dyn FnMut(u64) -> bool`): `prove_goal` keeps one
`Work` per search (an atomic count shared by its threads, against
`Limits::work`), counts each poll's units and asks the caller's stop
with `Progress { phase: Search, work: the poll's own units, done: every
thread's }` (so where every search runs on the calling thread, one
thread and the default bias in `Schedule::Turns`, the `work` add up to
`done` and to `Statistics::work`; the default bias's second search in
`Schedule::Auto` runs on a thread of its own, whose units reach `done`
only); past the bound the poll answers true and `prove_goal` turns the
engine's `Stopped` into `Reason::WorkLimit { limit }`. **Every thread
counts through one `Counted`**, which adds its units to the count in
batches of `Work::BATCH` (with `parallel`; alone it keeps them to its
end) and the rest when it drops, and reads the bound with its own units
(`Counted::passed`): the calling thread in `decide_goal`'s poll, which
tells the caller every poll, the one that passes the bound too; a pool's
worker in its `Stop::Flags` (`Flags::fired`, which also ends the worker
past the bound); the default bias's second thread in its `give_way`. So
the bound is late by at most a batch and a poll per thread. Two panel findings: the calling thread kept its units to the
end, and the second search, which reads the shared count, ran a whole
slice past the bound (`the_work_bound_holds_beside_the_second_search`,
which needs a release build to catch it: the calling thread's
millisecond wake-up hides it in a debug one); and `Stop::Turn` skipped
the caller at the poll that ended a turn, whose units were lost
(`a_turn_tells_the_caller_every_poll`). The driver asks the caller's
stop with 0 units each millisecond, which reports the workers' units,
and so does the calling thread while the second search has its turn.
The units an engine counted after its last poll are no work: the
additive path, which polls every 1 024 pairs, tells the stop its last
pairs when it ends, and the focused engine's last stable sequent or
split steps (under a poll's worth) go uncounted. The
units are what each engine already passed (the focused engine's slicing
work, one per literal or failed test, pair, marking or pivot; a set-up
poll none), so no poll moved and no counter of a decided run changed;
each `Engine` variant documents its unit. A search's `held_bytes` stays
zero: the default bias's two searches have accounts of their own;
`Reason`, `Statistics`, `Engine` and `Outcome` are `#[non_exhaustive]` so
later steps add variants and fields without a breaking change.
`Statistics` has one set of counters for every engine, each engine
filling those its `Engine::counters` lists with the meaning its rustdoc
gives (`nodes` is stable sequents for the focused engines, literals
chosen for the net engine, pairs for the additive path and markings for
the Horn engine; `memo_hits` and `memo_entries` the memo's of the
focused and additive engines and the markings met again and kept of the
Horn engine; `splits` the focused engines', `links` and `tests` the net
engine's, `copies` and `forward_copies` the focused engines' levels);
the counters an engine does not list stay zero; `work` is every engine's, set by
`prove_goal` from the search's `Work` (where every search runs on the
calling thread, the sum of the progress' `work`,
`the_search_counts_its_work`). `Statistics::add` is
two searches' merge (the race's), `add_run` a pool's workers'.
`Engine::counters` lists the counters each engine fills (`Counter`: the
key in the JSON form, a label, one for affine mode where the engine
counts something else there, the Horn engine's backward search, and the
meaning; `Counter::label_in(mode)` picks), which a front end shows
and `counters_are_fields` ties to the form; a new engine lists its own,
and a new counter goes into the lists of the engines that fill it.

**The race** (`search::race`, feature `parallel`): one thread first on
the calling thread and, once the caller's `add_pool` says so at one of
its polls, a pool of `threads − 1` beside it (`answer_goal` on a scoped
thread of `Limits::stack_bytes`), the first to decide raising a flag
both stops ask; below three threads, or on an engine that is not
`Engine::parallel`, one search with that many threads and `add_pool`
never asked. Both draw on one bound: the race's `Account` is the whole
and each search's is `Account::part_of` it, whose charges, shares and
forks count in the whole and stop at its bound (a part gives the whole
back what it holds when it drops); an account outside a race has no
whole, so no other search pays for it. One `Work` counts both, and the
outcome's `work` is that count. The verdict is made once both sides have
ended (`answer_goal` on each side, then `Answered::concluded` on a part
of the whole): before, each side checked its proof or built its
refutation's counts under a fresh bound of its own while the other side
still searched, so a race could hold two bounds at once. Two edge cases
moved with it, both sound: a side's proof settles the race before its
check, so a check that a bound refuses answers unknown where the other
side might once have gone on to a proof that checks (it shares the
bounds, so its check would rarely fit either); and a proof the checker
rejects is the race's error, where the other side's verdict could hide
it. A race's parts leave room for the input the whole holds
(`Account::holding`, `part_of`), so their shares are of what is left.
The command's default and the harness's `--pool-after` call it; the
copies they had are gone (F103, F104).

- **An engine's own refutation** goes in `Answer::refutation`, which
  `prove_goal` takes as it is: the Horn engine's
  `Refutation::StateEquation` (weights for every place that an exact
  check confirmed, and the clauses that can never fire, `core-horn.md`),
  the one refutation not computed from the
  counts. Every other `Unprovable` gets the counts' below. The inverse
  method's `Refutation::Saturated` is the exception: the refutation of
  last resort, taken only where the counts and the classical refuter
  find none (they are smaller and rest on simpler invariants), and
  checked first like a proof (`search/saturated.rs`, `core-inverse.md`;
  `Options::check` off skips it; a stop answers `Unknown(Stopped)`, a
  failed check `Error::Refutation`).
- **A refutation says what the counts rule out** (`Refutation`,
  `focus::counts::refutation`, called by `prove_goal` on every `Unprovable` of
  every engine that gives none of its own, which construct
  `Refutation::Exhausted`). It builds the
  focused engine's `Counts` (fresh account, the caller's stop: a pass
  given up is `Exhausted`, which is always true of the verdict), tallies
  the goal's members, and reports the first atom by the sequent's order
  whose summed interval excludes zero (`Tally::unbalanced`, through
  `Counts::ranked`, the atoms that have rows by rank), else the count
  equation when `Switches` applies it, with the goal's `⊗`, `⅋`, `1` and
  `⊥` counted. Why the goal's sums are a refutation although the engine
  tests stable sequents only: the asynchronous phase keeps them (a `⅋`
  adds a member and a `−1` of weight, a `⊥` takes a member and a `+1`,
  a premise of `&` lies in its hull, a `?` moves a formula whose atoms
  have no rows), so every stable sequent the goal reaches fails the same
  test, under the same `Switches` the engine searched with. It runs only on
  a refutation, after the search, so no counter of a run moves; its
  time is one more `Counts` pass on a refuted sequent. Without a
  refutation from the counts (a `⊤` absorbs, weakening, exponential
  atoms) the classical refuter below may give one, else the answer is
  `Exhausted`, never a guess.
- **The classical refuter** (`search/classical.rs`, `refute`, called by
  `conclude` through `classical`): after an `Unprovable` whose refutation
  is `Exhausted`, and after every `Unknown` of the engine (not one the
  check made: that sequent has a proof), it looks for an assignment
  under which the goal's classical reading is false (`!` and `?` erased,
  `⊗`/`&` and, `⅋`/`⊕` or, `1`/`⊤` true, `⊥`/`0` false, the goal the
  disjunction of its members) and answers `Unprovable` with
  `Refutation::Classical { assignment }`, a value per atom of the
  sequent, the atoms outside the goal false. The argument: every rule of
  `ll R`, for every `R`, derives a tautology from tautologies (Mix joins
  two, weakening adds a disjunct, contraction repeats one, promotion and
  dereliction read as the formula), and an intuitionistic derivation is
  a classical one of its one-sided form, which reads as the implication;
  the Rocq library proves both (`Refutations.v`, `ll_tautology`,
  `ill_ll`, `ill_tautology`), and `classical::tests::checked_proofs_read_as_tautologies`
  pins it against checked proofs of generated sequents in every mode. It
  is a backtracking over the goal's atoms by index, false before true,
  with a three-valued evaluation of the terms the goal reaches (in
  arena order, operands first, so no recursion) at each step, backing
  off where the reading is already true; one unit of work per term
  evaluated, bounded by `Options::classical_work`
  (`DEFAULT_CLASSICAL_WORK`, 2²⁶; zero switches it off; `--classical-work`
  in the command and the harness), its buffers charged to the account
  and refused where they do not fit, its polls `Phase::Refute` to the
  caller's stop. No counter of the search moves (its work is not in
  `Statistics::work`), and a decided verdict never changes; only an
  exhausted search's refutation and an undecided search's verdict do.
- **`Disproof::check`** (`refutation.rs`) verifies `Classical`,
  `Unbalanced` and `Equation` against the sequent, sharing no code with
  the search: it builds a `Forest` (default limits) and evaluates over
  occurrences bottom-up (preorder, so a subtree reversed is operands
  first). `Unbalanced` is recomputed as the hull interval of the atom's
  excess with `⊤` and an exponential over the atom unbounded (`None`),
  and must equal `[least, most]` and exclude zero, outside affine mode
  (Mix keeps it); `Equation` needs the multiplicative fragment with
  units in the goal and outside affine mode, `mix` equal to the mode's,
  and every count equal. Those are the conditions under which the search
  states them (`Switches`, `intervals_run`), and the ones the Rocq
  lemmas prove. `Saturated` is checked by `search/saturated.rs`, the
  rules re-read and applied to every choice of the set's sequents
  (`core-inverse.md`, faults `Unshaped`, `NotClosed`, `Filled`, and
  `NotApplicable` and `Mismatch` as for the others), which takes as long
  as the saturation did at most. `Exhausted`, `OneSided` and `StateEquation` answer
  `RefutationFault::Uncertified` (code `uncertified`, kind unsupported);
  a refutation that does not hold answers code `invalid_refutation`.
  Sums are of at most a forest's 2³² occurrences, within `i64`.
- **The command lets the refutation run past its deadline**
  (`cli/src/prove.rs`, `halting`): its stop answers a `Phase::Refute`
  poll with Ctrl-C alone, so a search the time limit ended is still
  refuted, within the work bound; the harness's child does the same.
- **One interface every engine implements** (`Decide`, crate-private):
  `admits(&task)` refuses a goal with the error a forced engine answers,
  `Error::EngineRefused { engine, because: NotTaken }`, checking in this
  order its largest fragment (`NotTaken::Fragment { decides, goal }`:
  focused `LL`, net `MLL`, additive `ADDITIVE`, Horn `MELL`, inverse `LL`), its modes
  (`Mode`), whether the goal is the roots (`Goal`, the net engine) and
  the goal's shape (`Shape`: not two formulas, no Horn program);
  `NotTaken::explained(engine)` is the message, in each engine's words
  where the behaviour lock pins them (`proof nets exist for MLL without
  units only, not for …`); code `engine_refused`, kind `unsupported`,
  and the harness reads the kind, not the variant, as `refused`; and
  `decide(&task, &options, &account, stop)` returns an `Answer`: the
  proof (`Ok(Some)`), an exhausted search (`Ok(None)`) or the reason it
  stopped, the counters, and the net the net engine found.
  `engine_for` answers which engine `prove_goal` would run on a goal
  (the passes before the search: the fragment, the reading, the
  dispatch, `admits`), and `Engine::parallel` whether that engine uses
  `Options::jobs` at all (the additive and the Horn engine do not); the
  command asks both before it adds a pool beside a single thread.
  `prove_goal` and `engine_for` share `fragment_of`, `read` and
  `prepare`, in the order of the errors a call answers.
  `Engine::implementation` maps each public variant to its
  implementation (`focus::ONE_SIDED`, `focus::TWO_SIDED`, `net::Nets`,
  `additive::Additive`, `horn::Horn`, `essential::Essentials`, `inverse::Inverse`); a `Task` is what they are handed (forest, goal,
  fragment, mode, reading, whether it is the roots, and the cell of the
  goal's Horn program: `horn::Program::of` reads it once per task, for
  the dispatch's feature, `admits` and the search alike, so a new reader
  of the program goes through it). An engine that keeps
  its proof as nodes of an arena answers through `Answer::of_arena`.
  **`conclude` (through `prove_goal`) is the one place an answer becomes a `Verdict`**: the
  refutation of an exhausted search, the check of a proof of the roots
  (and, with the check off, a `debug_assert!` of it), whatever engine
  ran. Which options each engine reads is said on its `Engine` variant
  and on each setter of `Options`; an option an engine does not read is
  documented there, never refused, since the dispatch may pick an engine
  the caller did not name. A new engine is a variant, an implementation
  of `Decide`, a line in `Engine::implementation`, and where it is the
  default a row in `DISPATCH`; its modes are tested in its own `admits`
  and written in its row's `Modes`, not read from one `Engine::modes()`
  as the design has it (a departure the step's report records).
- **The dispatch is a table** (`DISPATCH`, plan decision D8 as D19
  keeps it): rows of a largest fragment, the modes (`Modes`), a feature
  (`Feature`) and the engine, read from the first down (`dispatch`); the
  last two rows take every goal, intuitionistic to `two_sided` and
  classical to `focus`. The measurement behind each row is on `Engine`'s
  documentation ("Which engine decides a goal"), the rustdoc being the
  manual, and in `plan/reports/26-focused-engine.md`. A new engine's row
  goes where it wins and names its feature as a variant of `Feature`
  (step 27's Horn programs, for one); the order of the rows is the
  priority. Rows now: two formulas of the additive fragment with a
  connective or unit (`TwoFormulas`) to `additive`; a Horn program with a
  clause under `!` in any mode (`PetriNet`: the fragment has
  exponentials and `horn::is_net` finds a program with a `?` member in
  the goal itself, whatever fragment the options assert) to `horn`; the
  rest by mode. The first row is the Lambek calculus's: unit-free MLL in
  an ordered intuitionistic mode (`Modes::Lambek`) to `essential`, the
  planar essential search; the second the cyclic mode's (`Modes::Cyclic`)
  to `net`, the planar search; every other row's `Modes` takes
  commutative modes only (`Any` is every commutative mode).
- **An ordered mode at the front door** (`prepare`, `read`): `Mode::check`
  first (`OrderedMode`), then a fragment beyond unit-free MLL is
  `OrderedFragment` (before the dispatch, so that it is named and not
  `NoEngine`), and an engine other than the mode's planar one (`net` in
  the cyclic mode, `essential` in the Lambek calculus), forced, is
  `EngineRefused` with `NotTaken::Mode`. The reading is
  `Reading::of_mode`, which refuses a Lambek sequent without a
  hypothesis (`NotIntuitionistic`, `ShapeError::EmptyAntecedent`). No
  handover from the net engine to itself (`answer_goal` asks `engine !=
  Engine::Net`), none in the Lambek calculus (`Nets::admits` refuses
  it), and a race runs one search in an ordered mode. The classical refuter and the counts' refutations stay
  sound there (cyclic provability implies the commutative one); the
  planar search's own unprovable answer is `Exhausted`.
- **Why the Horn row takes affine mode too** (measured at step 27's
  second session): on qcover's 176 coverability problems at 5 s it
  decides 162 where the two-sided engine, the default before, decides 8
  (no verdict against another); on 10 464 random affine Horn programs it
  decides every one in at most 0.2 ms, the focused engines 9 578 at
  1 s; the focused engines seldom refute an affine goal with `!` (2 of
  qcover's 176), their copy bound deepening without end.
- **Why the Horn row takes only programs with `!`** (measured at step
  27): on the library's nets the Horn engine decides 3 026 nets against 1 628 (1 400 only by the Horn engine, 2 only by the forward search, no verdict against the other), in 0.23 ms against 1.2 ms in the median of the 1 626 both decide, faster on 1 013 of them; 2 670 within 10 ms against 1 103, but on Horn
  programs without exponentials (the Partition encodings, clauses used
  once) the focused engine's counts win by up to a hundredfold
  (`partition-yes` with 12 items 15 ms against 2.0 s, with 16 and 20
  items 0.2 and 0.5 s against over 10 s), while on `partition-no` the
  Horn engine is two to three times faster at the same node counts. The
  row's feature is the fragment's exponentials and the shape, nothing
  finer.
- **The net engine has no row; it takes over at a limit**
  (`handed_over`, in `answer_goal`): no engine forced, the dispatch's
  engine ended at `RecursionLimit` or `MemoryLimit`, the net engine
  admits the goal (unit-free MLL, the roots, no weakening), and every
  atom occurs once with each sign (`one_linking`). Then
  `Account::rewind` gives back what the first search left charged, and
  the net engine decides with the same options, limits, stop and work;
  the outcome names it and carries its counters (in a race the other
  search's are added), `work` both engines' units. Why each part:
  - *No row*: wherever both engines decide, the focused ones are faster
    (measured on the pruned net engine, `plan/reports/35-mll-engines.md`);
    the net engine decides only where they stop at a limit (long
    implication chains, some sequents of thousands of distinct atoms),
    and which goal reaches a limit depends on the limits, which no
    feature of the dispatch sees. The old row (no literal more than
    twice, `NET_MULTIPLICITY`) took those cases too but cost 2.8 to 26
    times elsewhere.
  - *`rewind`*: the focused engines charge their set-up's tables
    (`Classes`, `Counts`) and a memo's reserve and never release them,
    so without it the net engine's proof is checked under a spent
    account and comes back `Unchecked` (`wide(8192, 1)` at the default
    limits did).
  - *`one_linking`*: the net search is exponential on equal literals;
    without it a goal the focused engine gave up quickly at its limit
    became a search without end. With every atom once per sign the
    linking is forced: one link per pair, no backtracking, quadratic.
  - `IndexLimit` is not handed over (nothing measured there); a forced
    engine never is.
- **The essential-net engine has no row and takes no handover**
  (`Engine::Essential`, `essential::Essentials`): measured on the
  intuitionistic rows that placed the net engine, it beat the two-sided
  engine only on the 64-link chain (0.18 ms against 0.22 ms), which no
  feature of the dispatch singles out, and at the recursion limit the
  net engine is 2.2 to 6 times faster than it. It admits intuitionistic
  mode without weakening, unit-free IMLL and the roots
  (`NotTaken::Mode`, `Fragment`, `Goal`), is not `Engine::parallel`
  (the cubes are the classical criterion's), reads
  `Options::test_period` as the net engine does, and counts `nodes`,
  `links` and `tests` (dominator tests) with the net engine's units. It
  was last in `Engine::ALL` until the inverse method, since `Engine`'s
  order is its declaration order.
- **The inverse method has no row and takes no handover**
  (`Engine::Inverse`, `inverse::Inverse`, `core-inverse.md`): a forward
  saturation in every commutative mode (`NotTaken::Mode` in the ordered
  ones, `NotTaken::Fragment` beyond `LL`), not `Engine::parallel`,
  reading `bias` and `check`; it counts `nodes` (sequents taken up),
  `memo_hits` (sequents subsumed) and `memo_entries` (sequents kept).
  Its `Unprovable` comes from a saturation that ended and carries the
  counts' or the classical refutation where they find one, and else the
  set it saturated with, checked (`Refutation::Saturated`). It is last
  in `Engine::ALL`.
- **Intuitionistic mode** first computes the `Reading`
  (`Error::NotIntuitionistic`, whose message has ids; the CLI describes it
  with formulas) and refuses Mix (`Error::IntuitionisticMix`: a Mix premise
  would have no goal); then the same rows, with `two_sided` in place of
  `focus`, and `net` on unit-free IMLL by the embedding (below) where
  the two-sided engine stops at a limit.
  `Options::engine` forces an engine; `Engine::Net` on a fragment outside
  unit-free MLL, asserted or detected, or in affine mode,
  `Engine::Essential` outside unit-free IMLL or in classical or affine
  mode, `Focus` in
  intuitionistic mode and `TwoSided` in classical mode, `Additive` on
  anything but two additive-only formulas and `Horn` on a goal that is no
  Horn program are `Error::EngineRefused` (each engine's
  `Decide::admits`). A new engine's `Engine` variant has a
  `Display` that is its name in text and JSON, and a value of `--engine`
  in the CLI (`.claude/rules/cli.md`).
- **IMLL by embedding.** In intuitionistic mode the net engine runs on the
  one-sided sequent unchanged and its proof is returned as it is: every
  cut-free MLL proof of a sequent with one output-shaped root keeps
  exactly one output on every sequent (an all-input MLL sequent without
  units is unprovable, since every leaf has an output, so the split of a
  hypothesis `A ⊸ B` can never take the goal to the antecedent's side),
  hence any sequentialization of a classical net of an IMLL sequent passes
  the intuitionistic checker and no essential-net condition is needed for
  the verdict. With `1` the lowered sequent has units and goes two-sided.
- `Options::fragment` asserts a fragment: a sequent outside it is
  `Error::FragmentMismatch`, and the search runs in the asserted fragment,
  which switches off the prunes that only hold in the smaller one and
  picks the engine (`--fragment mall` on an MLL input runs `focus`).
- The crate has no clock (D11): a time limit is a closure the caller gives
  `prove_within`, and it answers `Unknown (Reason::Stopped)`. **Where it
  is polled**, which is every place a search can spend time without
  reaching another of them:
  - *The focused engine*: once per stable sequent (`prove_stable`); once
    every `SPLITS_PER_POLL` (4096) steps of its searches for the splits
    of a `⊗` or a Mix (`poll_splits`, on a counter of its own,
    `Run::steps`: a split search whose splits fail in focus visits no
    stable sequent and can run for minutes); once every
    `FORCED_PER_POLL` (4096) forced splits and literals of tensors
    closed in place (`poll_forced`, counter `Run::forced`: a chain of
    forced splits visits no stable sequent either, and a marking of a
    Petri net is a tensor of thousands of literals), the `&` rules
    counted on the same counter (`with`: `⊢ a₁ & b₁, …, aₙ & bₙ, ⊤`
    with `⊤` taken last has `2ⁿ` branches closed by `⊤` and no stable
    sequent; the fuzzer's `ordinary_decide` found such images that ran
    tens of seconds past a time limit of one,
    `with_rules_poll_the_stop`); and on a pool at every `&`
    (`with_parallel`, below). An `&` rule is work (`poll_with`: a step
    and the copy of a zone of the forest's width, the stable sequent's
    width cost), so `Limits::work` bounds such a proof and the default
    bias's two searches share it (`with_rules_are_work_and_their_proof_is_polled`);
    it asks the memory bound too, as a stable sequent does
    (`a_proof_of_with_rules_keeps_to_the_memory_bound`: 26 pairs peaked
    at 4.6 GiB under the default 1 GiB, now end at it in a second). The
    first two and the `&` rule's poll pass the work
    done since the last poll (`Stop::fired(work)`), which the two
    searches of the default bias slice by (`Stop::Slice`, `Stop::Turn`)
    and the caller's progress reports;
    the chain's poll passes none, so that the slices of those two
    searches are what they were before it existed and the counters of a
    decided run did not move.
  - *The net engine*: once per literal chosen (`decide`) and once per
    exact test that fails (`explore`): a run of failures chooses no
    literal.
  - *The set-up*, on a forest of `SET_UP_POLL` (65 536) occurrences or
    more (`set_up_stopped`): in `prove_goal` once the fragment, the
    reading and the dispatch are done, in `focus::prepare` (both
    `search_goal`s' set-up) after the classes and after the plan, and inside
    `Counts::new_until` every 65 536 occurrences visited, which is the
    longest pass. On the library's largest problem (`SYJ212+1.020` in
    its cbv translation, 27.8 million occurrences) the first poll comes
    after 0.12 s and no two are more than 0.2 s apart; without them the
    first came after 1.27 s. A smaller forest gets none of these polls:
    a pass takes under a millisecond there, and a condition that counts
    its polls (a test, a front end that counts work) sees the engine's
    own and no others.
  - *Not polled*: `Forest::new` and `Forest::within` (0.43 s on that
    problem; a caller with a deadline builds the forest itself, as the
    CLI does, and calls `prove_goal`), a single linear pass over the
    forest, and the collection of the kept arena
    when a memo is emptied (one pass over the kept nodes, milliseconds
    at a million of them). Freeing a full memo is no longer among
    them: its entries are records in chunks ("The memory bound",
    below), so emptying one resets a count and dropping one frees a few
    hundred blocks. When every key was two allocations, a memo at its
    cap of 2²⁰ entries took 0.15 to 0.55 s to free, which was what a
    stop was late by and a fifth of the time of a memo-bound search;
    now a stop on `qbf/40#1` with its memo full comes 14 to 21 ms
    after the limit on the machine's three kinds of core.
  - *The check at the end of `prove_goal` and the size pass of a
    derivation take the caller's stop* (`check_within`, every 4 096
    nodes or 65 536 units of its work, `core-proofs.md`), though they
    are short on the engines' proofs: on the largest the engines find in
    the LLTP library (`SYJ202+1.005` in its cbv translation, 566 490
    inferences) the check takes 22 ms and `Proof::derivation_size`, the
    same pass with an observer, 39 ms; on the largest Petri nets proved
    3 to 6 ms and 5 to 8 ms. A proof file, read from anywhere, can make
    a check quadratic, which is what the polls by work are for.
  A condition must be cheap, because it is asked at every poll, and it
  must not ration its own work by counting polls: polls come millions
  of times a second on a small problem and 30 ms apart on a forest of
  millions of occurrences, so "look at the clock every 1 024 polls" was
  exact on the first and half a minute late on the second. The CLI and
  the harness read a flag that a timer thread raises. The crate docs in
  `lib.rs` show the common path (parse, fragment, prove, derivation, JSON)
  as a doc test; keep it the shortest correct program when the API moves.
  The focused engine recurses on the caller's stack, bounded by
  `Limits::recursion_depth`; a caller that raises it runs the
  search on a thread with a larger stack (`Limits::stack_bytes`). The net
  engine and its sequentialization keep stacks of their own.

## The memory bound

`Limits::memory_bytes` (`DEFAULT_MEMORY_BYTES`, one gibibyte; `None`
lifts it) bounds what a search holds, and `search/memory.rs` is how:
an `Account` (the bound and an atomic count of bytes) that everything
which grows charges where it allocates, by the capacity allocated and
not by what is in use. `prove_goal` makes one per search.

- **What counts**: the focused memo (its chunks, its index, its table of
  unrestricted zones), the kept arena and every engine's pending
  stack, the branch stack of keys, the failures that wait (the lists
  by their capacity, `Waiting::grown`, charged after each close, and a
  key per entry, as many as ever waited at once; `relieve` gives both
  back; charging the keys alone left some 190 bytes an entry
  uncounted, a panel's and a review's finding: a search under 4 MiB
  peaked 9.9 MiB above its start), every pool buffer (sets, contexts,
  keys, tallies, split counts, cursors when made; lists, trails and
  links by what they had grown to when last given back), the `Counts`
  (rows and per-occurrence arrays) and the `Classes` of the set-up,
  and the additive path's memo and arena; **the input**, the forest
  with its sequent (`Forest::held_bytes`, about 25 bytes an
  occurrence), charged once by `prove_goal` and by `race` to the
  whole, and each search's intuitionistic reading
  (`Reading::held_bytes`, a byte an occurrence), and the check and the
  refutation after a search, which get what the bound leaves
  (`Account::left`; a refused check names the option's bound). **Why
  the input** (departs from the design, which left the forest and the
  sequent to the caller): the comparison gave linlog a
  `--memory-limit` equal to its cgroup's, and the default's race on
  `SYJ206+1.020` and `SYJ212+1.020` was killed there although its
  account stayed within the bound, since the process held the input
  beside it, 0.6 to 0.8 GiB on those files (a diagnosis by an
  instrumented copy of the account against the resident memory: the
  resident memory above the input stayed under the account's peak on
  every run). Since a search's set-up and memo are linear in the
  forest too, an input near `Limits::occurrences` leaves nothing for
  the search under the default bound; no problem of the library above
  a million occurrences was decided in the second baseline, where the
  input is 27 MiB. `Account::share` divides what the bound leaves
  beside what the account holds, so the default bias's two searches
  outside a race (whose shares have no whole) are bounded with the
  input too. **What does not**: under `--logic` the command's `Image`
  (`ordinary/translate.rs`, the ordinary sequent and the linear one
  beside the forest, linear in the input), the goal's occurrences a
  search keeps for its verdict, the proof returned beyond what the
  arena held (the focused engine keeps its root's proof in place where
  nothing is kept yet, `Arena::keep_all`, and `Proof::build` compacts
  the arena's vector in place, both polling every 2¹⁶ nodes, so a
  proof costs its nodes once plus four bytes a node while it is built;
  before, the root's keep and the build each copied it, unpolled: a
  second past the time limit on 24 pairs of `&` rules closed by `⊤`),
  a `Tally`'s and a `Split`'s `touched` lists (bounded by the atoms
  with rows: an atom is listed when a row finds its sums at zero;
  listed at every row, they grew with the copies of a formula once
  copied members had rows, 142 MiB under a bound of 16 MiB on a
  panel's witness), a context's extra list, the table a collection
  uses while it runs (four bytes a kept node), the stacks of the
  threads, the net engine (a structure and a scratch linear in the
  forest, per thread), and the allocator's own overhead. Measured when
  the input went uncounted, the process's peak was the count plus what
  it took to hold the input: R8 (`SYJ202+1.008` in cbv) under 256 MiB
  ended by the bound at a peak of 253 MiB, under 16 MiB at 20 MiB.
- **The order of answers** when memory runs short: a memo that has no
  room for a new key is emptied and the kept arena collected
  (`Run::remember`; the same as at `memo_limit`, which stays as the
  finer knob: it is what the pinned counters depend on, and a table
  that fits the cache can beat one that fits the memory); a search
  that finds itself over the bound at a stable sequent or an `&` rule
  empties the memo, collects, and if that is not enough gives the
  memo's memory back (`Run::relieve`); `Unknown(Reason::MemoryLimit(bytes))` when
  what is left, the branch's own buffers and proofs as allocated, is
  still over, or when an empty memo cannot have its first chunk. The
  value in the reason is the option's, whatever share a search had
  (`Reason::as_set`). `relieve` does not squeeze the arena to fit: the
  next `keep` would double it again, and a search at its bound would
  copy its arena at every node (seen: 170 stable sequents a second
  where there were 400 000).
- **The memo leaves an eighth of the bound free** (`Account::spares`):
  it would otherwise take every byte, and the first growth of anything
  that cannot be emptied would cost the whole memo, at every node.
- **`Reason::IndexLimit`** is what a structure answers when it outgrows
  its `u32` indices, which only a search without a memory bound can:
  the arena at 2³¹ nodes, the counts' rows at 2³² entries, a forest of
  2³¹ occurrences or more for the counts (their balances are `i32`
  sums over a subtree), the additive path's arena at 2³² nodes.
- **`Limits::occurrences`** (`DEFAULT_OCCURRENCES`, fifty
  million) is the bound on the input: `prove` and `prove_within` build
  their forest with `Forest::within`, the command checks
  `Sequent::occurrences()` when it reads a sequent, before any command
  unfolds or prints it. `Forest::new`, `Interactive::new` and a plain
  `Deserialize` (a proof file, a session's state, a net) have the
  default; `Forest::within`, `Interactive::within` and the wire's
  readers (`wire::upgrade`, `wire::Within`) take the caller's limits.
  The command still reads proof files and sessions with plain serde,
  under the default whatever a flag says, until its area moves it to
  `upgrade`.
- **On a pool** the workers share the account, each engine's own
  buffers are released when it goes (`Charged`), and the shared arena
  only grows. A pool therefore reaches the bound sooner than one
  thread on a search that proves much and keeps it.

## The additive fast path

`search/additive.rs` decides a sequent of exactly two additive-only
formulas by a recursion on pairs of subformula occurrences, one below each
root, memoized on the pair: `⊤` closes, `&` on either side needs both
subformulas against the other, two dual literals are an axiom, `⊕` on
either side tries one subformula at a time, and nothing else proves
anything. `&` is invertible and goes first; **which `⊕` to decompose is a
real choice** (a `&` below the other formula's `⊕` may need both sides of
this one: `⊢ ~c ⊕ ~a, b ⊕ (c & a)`), so both formulas' `⊕` are tried and
the memo is what bounds the work by `|A|·|B|`; a first version that
returned after the first formula's `⊕` was caught by the review. The
procedure is the same in every mode: additive rules keep one output by
themselves, and neither weakening nor Mix can help a two-formula sequent
(a proof of one formula alone ends in `⊤` leaves, which absorb the other).
`Statistics::nodes` is pairs visited, `memo_hits` and `memo_entries` the
memo's. Its memo and its arena are charged to the search's account at
every pair (`settle`): over the bound the memo goes, and the arena
alone over the bound is `MemoryLimit` (the arena is append-only here:
nothing collects it, since a pair's proof is one node and the memo is
what bounds the pairs). The memo holds at most `Options::memo_limit` pairs and is emptied
when full, like the focused memo (zero switches it off); the product
bound on the time then no longer holds in theory, but the identity of
depth 16 (7.9 million pairs without a cap) is decided with the default
limit of 2²⁰ in 11.7 million visits instead of 10.7 million, in less
time (a table that fits the cache) and in 0.1 GB instead of 0.46 GB.
What took gigabytes on such a proof was not the search but the checker's
first implementation, which kept a `Θ` bitset of the forest's width for
every node (262 141 nodes of 32 KB at depth 16, 7.6 GB): the first
baseline's "8 GB of memo" was this, measured by the peak before and
after the check. The checker no longer keeps one ("The checker").

## The reference prover

`search/reference.rs` (test-only) decides a small sequent by the plain
rules of the unfocused calculus, one-sided classical and two-sided
intuitionistic, linear and affine, with and without Mix, every rule on
every formula, with contraction bounded per branch and the bound deepened
from zero, so that every budget's answer is given; it shares no code with
the engines (its formulas are its own, read from the generator's `Tree`s
or a sequent's terms). Its tests compare every engine configuration
(the dispatch, each engine forced, the focused engines under each bias,
and with `parallel` the dispatch on two threads) on generated sequents,
their mutants and a few that need contractions, and the families'
verdicts at the sizes it decides; what they assert is what the contract
allows: never a proof against a refutation, either way.

- **It cuts every failing sequent whose search can contract**: one with a
  `?` member (or a `!` hypothesis) contracts to an empty budget and is
  cut, so with exponentials it refutes only where the failure lies on a
  premise without one (`⊢ 0 ⊗ ?a`). It judges an engine's "unprovable";
  an engine's proof is the checker's.
- **Its budget counts contractions per branch**, one less than an
  engine's copies of a formula; the two are never compared.
- **`VISITS`** (a million sequents and splits per decision) bounds its
  work; a split enumeration is lazy and each split counts, since Mix and
  `⊗` have `2ⁿ` splits of `n` formulas.
- **What was checked before it judged**: a fresh-context review against
  the calculus (sixty sequents decided by hand, no disagreement; memo on
  and off, budget monotonicity and mode consistency on about 2 800 more),
  and eleven deliberate faults in a scratch copy (a rule dropped, a side
  condition flipped, a copy bound off by one, a cut flag dropped, a cut
  failure memoized as a refutation), each of which makes the committed
  tests fail. A fault in the bound only shows on sequents that need
  contractions, which the generators make seldom: hence
  `engines_agree_where_contractions_are_needed`, and the deepening, which
  asks the reference at every budget below the one given.
- A new engine joins `configurations` in its tests. The inverse method
  runs there within `INVERSE_POLLS` polls (its saturation need not end
  with exponentials); its own differential test gives it more and
  asserts it decides every sequent without exponentials.
- **A change of the search is judged against a second reference too**,
  kept outside the repository and written by a fresh agent from the
  calculus alone (its own parser of the text syntax, every rule on every
  formula, contraction budgeted per branch). None is maintained: a step
  that changes the search writes or asks for its own, so that the
  judgment's independence does not rest on one file.

## Decisions

The author's answers for the release (`plan/notes/api.md` §14), which
the fixes implement and later rounds judge against. Where a bullet above
still describes code that a decision changes, the decision holds, and
the commit that lands it rewrites that bullet.

- **`Verdict::Unprovable` carries a `Disproof`**: the sequent, the goal,
  the mode and the refutation. `Verdict` is closed, and a refutation's
  checker and certificate need its sequent and mode.
- **Refuters run after the search, never during it**: after an
  `Unprovable` that has no certificate, and after an `Unknown`. A
  refuter never changes a verdict the search gave. The design had
  refuters after an `Unknown` only behind `refute_unknown`, off by
  default; the author decided on 2026-10-18 that the classical refuter
  answers an `Unknown` by default (R2 of
  `plan/notes/research/comparison-gaps.md`), its bound `classical_work`
  the only switch, so `refute_unknown` does not exist. Baselines keep
  every decided verdict; rows that were unknown may become unprovable.
- **`search::Options::default()` keeps the copy bound of 3, and
  `Settings::default()` is the command's behaviour** (no copy bound, a
  clock). A library call without a stop must end; a front end with a
  clock deepens.
- **Dispatch thresholds are not options**: a row's threshold is a
  private constant beside its measurement, and `engine` is the knob. A
  front end must not move a row the library's measurement placed.
- **`Statistics` keeps its shared counters**, each documented per
  engine: an engine adds a field only where none fits, and never gives a
  counter it fills a new meaning. Every baseline and the CSV columns read
  them by name.
- **An outcome names the crate's version, not its options**: the front
  end records the settings it ran under, once.
