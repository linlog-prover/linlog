---
paths:
  - "core/src/search/parallel.rs"
  - "core/src/search/focus/parallel.rs"
  - "core/src/search/net.rs"
---

# linlog core: the parallel runtime

Loaded, beside `core.md`, when the runtime or one of the two engines on
it is read (`net::parallel` is inside `search/net.rs`).

## The parallel runtime

`search/parallel.rs` (feature `parallel`, off by default, on in the CLI,
never on wasm: it is the one place the crate needs threads) is the
runtime; `focus/parallel.rs` and `net::parallel` are the two engines on
it. `prove_goal` takes the parallel path when `Options::jobs` is above one
and the engine is `Focus`, `TwoSided` or `Net`; the additive path stays
sequential (it is linear-time in the product of the formulas' sizes and
has no or-choices worth sharing out). What the code relies on:

- **One pool per search, no global.** `Runtime::new(jobs, stack_size)`
  builds a rayon pool of `jobs` threads with stacks of
  `Limits::stack_bytes()` (the engine recurses on the worker's stack as
  it does on the caller's), which the search drops with the outcome, or
  gives back to the caller's `Pool` (below); the two places that take
  one are the net engine's in `prove_goal` and the focused engine's in
  `focus::parallel::search_goal`, which takes two for the two searches
  of the default bias with exponentials and splits `jobs` between them;
  `Error::ThreadPool` when the threads cannot start. **A search starts
  no more threads than the machine runs at once**: `prove_goal` takes
  `Options::threads()` through `parallel::threads`, the smaller of it and
  `std::thread::available_parallelism()` (which on Linux follows the
  process's CPU set and quota; where the platform does not tell, the
  options' own bound `MAX_JOBS` is all there is), and one thread left
  is the sequential path. So the tests' `jobs(4)` is two threads on a
  machine of two, and a harness row never names more threads than its
  process may run (`linlog-bench run` refuses the count). Never touch rayon's
  global pool: a library must not size or seed it, and `RAYON_NUM_THREADS`
  is read only when a builder's thread count is zero, which ours never
  is.
- **A `Pool` keeps runtimes across searches and changes no search.**
  `search::Pool` (public, cloneable, an `Arc` of the idle runtimes;
  `Options::with_pool(Some(pool))`) is the caller's, never global. Every
  runtime a search takes goes through `Lent::take(options.pool, threads,
  stack)`: an idle runtime of exactly `threads` workers (the engines
  read `Runtime::threads` for the cube count and the split tasks, so a
  larger one would change the search) and a stack at least the one
  asked (the stack bounds only overflow; the recursion limit is
  counted), else a new one. `Lent`'s `Drop` gives it back, on a
  return, an error (the first of the two-bias pair when the second
  fails to start) or a panic, always after the scopes on it ended.
  A taken runtime is removed from the idle list under its lock, so a
  runtime is held by one search at a time: two searches through one
  pool at once, or the two searches of the default bias, each run on
  their own. A rayon pool between searches holds no state (idle
  workers, empty deques, and the engines keep no thread-locals), so a
  kept runtime runs the search a fresh one runs. `MAX_KEPT` (4) bounds
  the idle list, what two searches at once with two runtimes each
  hold; the one given back longest ago is dropped past it. `Options` compares
  pools by identity (`Arc::ptr_eq`). A new place that starts a rayon
  pool takes it through `Lent::take`, never `Runtime::new`;
  `parallel::tests::searches_share_a_kept_runtime` counts the runtimes
  a pool builds (a test-only counter).
- **The stop closure is polled on the calling thread.** `Runtime::drive`
  spawns the work into the pool from an `in_place_scope` and, on the
  calling thread, waits on a channel for the result with a one
  millisecond timeout, polling the caller's closure at each timeout and
  raising the root `AtomicBool` when it fires. So `prove_goal`'s closure
  needs no `Send` and is polled about a thousand times a second, not
  once per node: a caller that read the clock every `n` polls would
  have to read it at every poll on several (the CLI and the harness read
  a flag that a timer thread raises, at every poll). A worker polls its `Flags`, the chain
  of its own cancel flag and its ancestors' up to the root, at every
  stable sequent (`prove_stable`) or literal chosen (`decide`), through
  `Stop::Flags`; the sequential engines poll the closure through
  `Stop::Closure`. rayon tasks cannot be killed, so a place that stops
  polling is a place cancellation does not reach.
- **A panic stops the rest of the search** (`RaiseOnPanic`, a guard
  that raises its flag when a panic drops it): the driver's and the
  race's loops, which call the caller's stop, raise the root flags; a
  choice's tasks and a `&` premise raise the choice's `cancel`; a net
  worker raises `found`. rayon's scope runs every task it spawned to its
  end before it lets a panic go on, so without the guard a panicking
  stop condition waited for the whole search (`mix(11)` on two threads,
  minutes; `a_panicking_stop_ends_every_worker`) and a panicking
  premise for its siblings' searches.
- **The split searches poll too** (`poll_splits`, every
  `SPLITS_PER_POLL` steps, through the same `Stop`): before, the poll
  there was tied to `Statistics::splits` being a multiple of 4 096, and a
  loop that adds two to the counter per round (its own split and a forced
  one below) from an odd value never hit one. That is how 90 one-thread
  LLTP runs of the first baseline ran past their kill, 166 more at 16
  threads, and two Petri nets were proved minutes after their limit.
  `focus::tests::stops_inside_a_split_search` and the second half of
  `focus::parallel::tests::stops` pin it on a sequent whose 2⁴² splits
  all fail in focus (affine mode, so no count cuts them).
- **Cube-and-conquer is nested fork-join at the first `LEVELS` (2)
  choices of a branch**, not a static enumeration: at a choice among
  alternatives (`decide_with`'s candidates and copies together, the two
  sides of a `⊕`, the free splits of a `⊗` with the assignment of the
  first `fixed` members per task, `fixed` giving the pool twice its
  threads in tasks and at most `MAX_FIXED` = 6 bits) an engine whose
  `or_depth` is below `LEVELS` spawns a worker per alternative but the
  first and runs the first on one more worker on its own thread
  (`choose_parallel`); workers have `or_depth + 1`. Every alternative
  runs on a worker, never on the spawning engine itself, because a
  worker's stop chain holds the choice's cancel flag and the spawning
  engine's does not: an alternative run in place would never be
  cancelled by a sibling's proof (a review measured a whole refutation
  spent that way). Below the levels a worker is the sequential engine.
  rayon's work stealing is what makes this "when the pool has idle
  workers": a spawned task nobody steals runs on the spawning thread
  after its own alternative. The `&` rule within the levels runs its
  right premise on a worker of the pool and its left one on a worker on
  its own thread (`with_parallel`), and **is a level itself** (its
  workers have `or_depth + 1`, like a choice's): a tower of nested `&`
  forks at its first `LEVELS` and runs sequentially below. When it kept
  the level, every `&` of a branch with no choice forked, so each level
  of the tower cost a scope, a worker and a copy of the branch on the
  stack and the heap: two copies of 2 900 nested `&` overflowed an
  8 MiB worker stack at a recursion limit of 3 000 on two threads (H18,
  `a_raised_limit_holds_on_the_pool`, which aborts in a release build
  only: a debug build's allowance per level hides it), and 5 000 nested
  `&` held 823 MB under a 64 MiB bound (F92; 24 MB now). The `⊗`
  premises stay sequential (the first usually fails fast). **`with_parallel` polls the engine's
  flags before it starts anything.** The asynchronous phase polls
  nowhere else (its stable sequents do), and the `&` rule on the pool,
  unlike the sequential one, starts its right premise without waiting
  for the left: a worker that was cancelled or stopped therefore went
  on to start both premises of every `&` below it, each of which did
  the same, and only their stable sequents ended them. With `n` `&` in
  one asynchronous phase that is `2ⁿ` workers. `SYJ202+1.008` in its
  cbv translation (19 KB) has such a tower: under a 5 s limit two
  threads ended after 46 s and four not within 150 s, with 578 stable
  sequents really searched and 268 million stopped at their poll; the
  first failed premise alone set it off, long before the limit.
  `a_cancelled_premise_starts_no_other` pins it on forty roots `a & b`
  (2⁴⁰ premises without the poll). A new rule that fans out on the pool
  polls before it spawns. **So does every task of a choice, before it
  builds its worker** (`choose_parallel`, `Collected::skip`): a choice
  queues a task per alternative, hundreds on a stable sequent of a
  Petri net, and a worker used to copy the branch's stack of keys, so
  the tasks the pool reached after a stop or a sibling's proof spent
  seconds before their first poll. On
  `GlobalResAllocation_galloc_res-5_100_1` on four threads a stop came
  21 s late and a proof that one thread finds in 0.14 s took 17 s;
  with the poll, 0.5 s and 0.13 s. A task skipped for an ancestor's
  flag records a stop, never a failure, since what was never searched
  cannot have failed (`a_skipped_alternative_is_no_failure`); one
  skipped for the choice's own flag records nothing, because that flag
  is raised only under the choice's lock once a proof or an error (a
  skip's stop included) is recorded. Mix stays sequential after the
  parallel alternatives failed (`last_resort`).
- **A worker reads its branch in place, and is not a copy of the
  engine** (`Spawn`, `Spawn::worker`): `Run::new` builds it, the one
  constructor of an engine, from the spawning engine's `Problem`
  (forest, reading, counts, classes, rules, account and limits), with
  the shared memo and arena by reference and fresh pools and counters,
  and then gives it the branch: `depth`, `or_depth`, and as `ancestors`
  the spawning engine's own ancestors and live stack of keys with their
  hashes, as slices (`Run::repeated` reads them before its own stack,
  `Run::above` counts them into a depth). The keys are not copied: the
  spawning engine waits at the scope while its workers run, so its
  stack cannot change under them; a worker's `ancestors` are one slice
  pair per spawn above it, at most `LEVELS` + 1. Copying it, two bitsets of the
  forest's width per stable sequent of the branch, for every task of a
  choice, was a fifth of the samples of a Petri net on four threads (with
  hashing the keys again, which a copy of the hashes had removed before).
  The ancestors are what keeps the loop check's prunes below the cube.
  **The recursion limit holds for a worker's stack**: a pool thread that
  waits at a scope runs stolen tasks on its own stack, on top of the
  waiting engine's frames, so the runtime keeps per worker the depth of
  the engine waiting there (`Runtime::waiting`, a guard that restores
  the depth it replaced; indexed by the pool's own
  `current_thread_index`, written only by the worker itself), and a
  task's worker starts at the larger of its branch's depth and its
  thread's (`Runtime::depth_here`): its recursion counts on top of the
  frames it sits on, and `Reason::RecursionLimit` comes before an
  overflow at any limit.
- **Each rule is written once; the merge of cuts is the scheduler's.**
  A choice's alternatives are one type (`focus::Alternative`: a focus on
  a member of `Γ`, a copy, a side of a `⊕`, the splits under a pattern)
  with one implementation (`Run::alternative`), which one thread runs
  in order (`Run::choose_here`) and a pool as tasks within its first
  `LEVELS` choices (`Run::choose`, `choose_parallel`); a `&` premise
  is `Run::premise` and its node `Run::both` on either. A choice's
  result is a proof if any alternative found one (a proof of one
  alternative wins over an error of another, so the pool may decide
  where one thread gives up with `RecursionLimit`), else the first error
  that is not a stop caused by cancellation (a worker's `Stopped` is
  ignored only when the choice's own `cancel` flag is raised), else a
  failure with the cuts (`focus::Cuts`) of the alternatives that ran to
  their end (`Collected::take`). **A proof carries no cuts**, on one
  thread as on the pool (`Found` is a proof or a failure with its cuts):
  a failure after a proved premise or alternative rests on its own cuts
  alone. Until step 26 one thread's proof carried those of the
  alternatives tried before it and a proved premise's cuts went into
  its sibling's failure, which was sound and less decisive at the copy
  bound. For `&`, a failed premise decides, whatever the other found.
  Success raises `cancel` at an or-node; at the `&`, **a failure of
  either premise, and the left premise's giving up**: a right premise
  that gives up (the recursion limit, the memory bound) does not cancel
  the left, whose failure still decides the rule, as on one thread, which
  searches the left first; it cancelled it before, so the pool answered
  `RecursionLimit` where one thread answered
  `Unprovable` (`a_premise_that_gives_up_cancels_no_other`). A left
  premise that gives up still cancels the right, which one thread never
  starts then: with neither cancelling on a give-up, `k` nested `&` whose
  leaves all reach the limit searched all `2ᵏ` leaves where one thread
  stops at the first (a panel's finding,
  `nested_premises_that_give_up_search_no_tree`). An error at a choice
  still cancels its siblings, so a pool may answer `RecursionLimit` where
  one thread, whose memo spared it the depth, refutes (`|- !?~c, (?(a *
  c) par ?!~c)` under Mix at a limit of 9 on four threads): the or-node
  mirror of the case the `&` lost, a follow-up. A premise
  stopped by an ancestor's flag returns `Stopped`, which gives way to
  the other's reason in the `&`'s result as it does in
  `Collected::take`. **What stays**: a premise's failure raises the flag
  only once its worker has left its nested scopes, where a waiting
  thread may have stolen a task of the sibling premise that nothing
  cancels until then (seen once: an answer that came only with the
  caller's stop, five seconds late); the stolen task polls the `&`'s
  flag, but the failed premise's continuation runs only after the stolen
  task returns, on the same stack, and a choice's failure inside the
  premise implies the premise's only in tail position, so raising the
  flag earlier needs that knowledge passed down. A worker inserts into the memo
  only what its own `prove_stable` decided, so a cancelled worker leaves
  facts and nothing half-done.
- **The shared memo is 64 shards of the sequential `Memo`** behind one
  `Mutex` each (`memo::Shared`, the key's top hash bits choosing the
  shard, the cap split among them), and `Memo::insert`'s merge under the
  shard's lock is the compare-and-swap the bound needs: a larger
  `Exhausted` budget wins, `Complete` and `Proved` win over `Exhausted`,
  a proof stays, so an entry's validity only grows whatever the
  interleaving, and two workers deciding one sequent cost duplicated
  work, never a weaker entry. A lock is held for one map operation and
  never across a recursive call. `dashmap` was not taken: the notes
  record its maintenance as thin, the shards are twenty lines, and the
  speedup table shows no contention worth a dependency. `hits` and `peak`
  are summed over the shards (`peak` is an upper bound). A shard that is
  full, by its entries or by the memory left, is emptied by itself
  under its lock; one that is empty and has no memory for its first
  chunk drops the entry (the other shards hold the memory), and a
  worker that finds the search over its bound releases every shard
  (`Shared::release`). The chunks of a shard are 64 KiB, not a
  mebibyte, so that 64 shards with one entry each are not 64 MiB.
- **The kept arena is shared behind one `Mutex<Vec<Node>>`**
  (`Kept::Shared`), and every engine of a parallel search has a pending
  stack of its own. Truncating a shared arena would be unsafe (another
  worker's nodes lie above one's mark), so nothing pending is shared: a
  `keep` takes the lock once and appends the whole segment, so a kept id
  is a position in the one arena every `Proved` entry refers to, a
  segment's premises are in it or were kept before it, and the order
  `Proof::new` needs holds across threads; the memo insert happens after
  the keep, so a hit always finds a complete subtree. A pending id means
  nothing outside its engine, so a result that leaves a worker (an
  alternative's, a `&` premise's, the root's) is kept first
  (`Run::exported`); the spawning engine wraps kept ids in pending
  nodes of its own. Failed branches therefore cost the shared arena
  nothing on the pool either.
- **Levels never overlap**: `run` deepens the copy bound on the root
  engine, which spawns nothing until its first choice and reads
  the level's cuts after every task of the level has ended (the scope waits),
  so the or-reduction over the workers is the merge above and a level is
  `Unprovable` only with no worker's failure cut.
- **The net engine's cubes are what is left of the search, split at
  its choices** (`net::parallel::search`). A cube is the links of a
  branch nobody has followed yet, and the list of cubes is kept in the
  order in which the sequential search would reach them. The root
  engine starts from the one cube without a link and, while there are
  fewer than `CUBES_PER_THREAD` (16) cubes per thread, makes a pass
  over the list that replaces every cube in place (`reset`, `seed`) by
  the branches of its next choice (`explore(Some(1), …)`: the search
  below the seed with every branch recorded and taken back at its first
  link of a literal with more than one admissible partner; the forced
  links on the way to that choice are made and stay in the cube). So
  after `d` passes the cubes are the branches of the first `d` choices
  that the tests do not reject. A branch
  that dies leaves no cube, a proof net found on the way ends
  everything, and an empty list is `Unprovable`. So nothing is searched twice: the cubes
  partition the remaining search at every moment, and a sequent whose
  links are all forced is decided by the first `explore`, which is the
  sequential search, link for link (`forced_links_are_made_once` pins
  equal statistics on `wide(64, 1)`). Before, the cubes were the
  branches of the first `d` links found by a search from the root for
  `d = 1, 2, …`; with forced links there is one branch at every depth,
  the count was never reached, and `wide-m1` at 512 pairs made
  1 + 2 + … + 1 024 = 524 800 links (9.5 s on two threads against
  33 ms on one). What it costs now where the queue stays short: the
  seed's links again per cube taken, without their tests, which is of
  the order of the `choose` the sequential search pays per node.
  Workers pull cubes from an atomic counter with one engine each, so
  the per-worker state is allocated once; a worker that finds a net
  stores it and raises the flag; `Unprovable` needs every cube to have
  ended `Ok(false)`, and any error or a real stop makes the verdict
  `Unknown`. **The order is what keeps a pool from being slower than
  one thread on a provable sequent**: the workers take cubes in the
  sequential search's order, so the cube with the proof one thread
  finds is taken no later than one thread reaches it. A first version
  kept the cubes in a queue and appended a cube's branches at its end;
  the workers then searched cubes that one thread never enters before
  the one with the proof, and Partition with the items 1, 1, 2, 4 took
  two threads 2.2 s and 253 852 literals against 1.3 s and 109 627 on
  one. **A pass is always whole.** Stopping in the middle of one, as
  soon as the count is reached, left the unsplit cubes at the end of
  the list one choice coarser than the rest, and the refutations of
  3-Partition were 3 to 4.5 % slower on two and four threads than with
  cubes of one depth; the count may therefore overshoot by a level's
  branching, as it always could.
  No state is shared beyond the flags: the structure and the scratch
  are per worker. A cube's seed must reproduce the root engine's state
  at the record: the links in order, which the structure's undo log and
  the test cadence (a multiple of the period in links) both follow.
- **A parallel run may return another proof, never another verdict**:
  every level is searched to its end by some worker with no cube
  abandoned unless a proof or an error ends it, so `Proved` and
  `Unprovable` agree with the sequential engine; only decisiveness within
  the copy bound may differ (as it does between memo and no memo), since
  the memo's contents depend on the interleaving. A parallel run may
  answer `Proved` where the sequential one answers `Unknown
  (RecursionLimit)` on another alternative. `Unknown (Stopped)` is the
  caller's stop, never a cancellation. **What a pool promises about
  time**: a stop is honoured by every worker at its next poll (the
  list under "Proof search: the front door"), the driver asks the
  caller's condition once a millisecond, and a pool costs its start
  (some tens of microseconds per thread) plus, on the net engine, the
  seeds above; it promises no speedup, and on the focused engine no
  bound on the work relative to one thread (and-parallel `&` premises
  and cubes search what one thread might have skipped). The tests
  (`focus::parallel::tests`, `net::parallel_tests`) assert exactly this
  on the generated samples with two and four threads; every proof is
  checked. For the focused engine that is `agree`: the two verdicts
  never contradict each other, and nothing is asserted about which of
  them decides within the bound (`b, ((a * 1) -o !a), !(b -o 1),
  !(1 -o ((1 * 1) -o a)), b |- (!!a * b)` with a copy bound of 2 is at
  its bound on one thread and proved on four, every time).
- **There is no portfolio of worker orders.** An option gave every
  worker a seed that ordered the candidates and copies within their
  classes; two baselines showed no gain (the second: 627 problems
  decided against 623, 10 gained and 6 lost, at the same time), and it
  was removed. The two searches of the default bias are the portfolio
  that pays: two orders that differ in the one choice that changes the
  search. Candidates and copies are ordered by id on every engine.
- **Statistics** add every worker's counters (`Statistics::add`, the
  memo's read off the shared table once), so a parallel `nodes` is the
  work done, not the work one thread would have done, and the CLI's
  pinned counts use `--deterministic`.
