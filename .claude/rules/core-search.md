---
paths:
  - "core/src/search/mod.rs"
  - "core/src/search/memory.rs"
  - "core/src/search/additive.rs"
  - "core/src/search/reference.rs"
---

# linlog core: proof search, its front door, its memory bound, the additive path and the reference prover

Loaded, beside `core.md`, when the search's front door, its memory
account, the additive path or the reference prover is read. The engines have files of their
own: `core-focus.md`, `core-nets.md` (the net engine) and
`core-parallel.md` (the pool).

## Proof search: the front door

`search/mod.rs` is what a front end calls: `prove(&sequent, mode,
&options)` and `prove_until(…, stop)` return `Result<Outcome, Error>`, and
`prove_goal(&forest, goal, mode, &options, stop)` decides any multiset of
occurrences of a forest, given in any order, `prove_until` being that on
the roots (the roots in any order are the roots, `is_roots`, and the
engines get them in the forest's order, so the net engine takes them and
the proof is checked): the goal's own fragment (`goal_fragment`, over the subtrees)
picks the prunes and the engine, the net engine only for the roots
(`Error::NetGoal` when forced elsewhere, since a structure's conclusions
are the forest's roots), the additive path for any two additive-only
occurrences (`additive::search_goal`), the focused engine otherwise; in
intuitionistic mode a goal must have exactly one occurrence in output
position (`Error::GoalOutputs`). **Every proof of the roots has passed
the checker when it is returned, in every build** (`Options::check`,
`DEFAULT_CHECK` true; the check is at the end of `prove_goal`, one place
for every engine, and a proof it rejects is `Error::Rejected`, an error
and never a verdict). The engines' own `debug_assert!`s on their proofs
stay, and the flake's `test-debug-assertions` check is what runs them,
since crane tests in the release profile. The harness switches the check
off to time the search alone and checks the proof itself. The proof of a
goal other than the roots
has a root that concludes the goal, so `Proof::check` rejects it; only
`Interactive` consumes such proofs, by grafting their derivation, and
`Derivation::of_goal` checks them against the goal on the way. Where
`Outcome` carries the `Verdict` (`Proved(Box<Proof>)`, `Unprovable(Refutation)`
only after an exhaustive search, which with exponentials means a
deepening level that never hit the copy bound, `Unknown(Reason)`, with
`Reason::CopyBound` when every level up to a bound hit it), the `Fragment` searched in,
the `Mode`, the `Engine` that ran, the `Statistics`, and `net`, the
`ProofStructure` the net engine found (`None` from the focused engine).
`Options` has private fields and setters (`memo_limit`, `recursion_limit`,
`engine`, `fragment`, `test_period`, `copies`, `jobs`,
`bias`, `forward_copies`, `check`, `memory_limit`, `occurrence_limit`,
and with `parallel` `pool`, the `search::Pool` of `core-parallel.md`),
the constants `DEFAULT_MEMO_LIMIT`, `DEFAULT_RECURSION_LIMIT`,
`DEFAULT_COPIES` (the library's default bound; `copies` takes an
`Option`, `None` for none), `DEFAULT_FORWARD_COPIES`, `DEFAULT_MEMORY_LIMIT` (one
gibibyte) and `DEFAULT_OCCURRENCE_LIMIT` (`Forest::DEFAULT_LIMIT`),
which the CLI shows as its defaults, `MAX_JOBS` (256: `jobs` takes more
as that many, and zero as one), and `stack_size()`,
the stack a thread needs at the recursion limit, which sizes the CLI's
search thread and the parallel pool's workers alike;
`Reason`, `Statistics`, `Engine` and `Outcome` are `#[non_exhaustive]` so
later steps add variants and fields without a breaking change.
`Statistics` has one set of counters for both engines: `nodes` is stable
sequents for `focus` and literals chosen for `net`; `memo_hits`,
`memo_entries` and `splits` are the focused engine's, `links` and `tests`
the net engine's, and the others stay zero.

- **A refutation says what the counts rule out** (`Refutation`,
  `focus::refutation`, called by `prove_goal` on every `Unprovable` of
  every engine, which construct `Refutation::Exhausted`). It builds the
  focused engine's `Counts` (fresh account, the caller's stop: a pass
  given up is `Exhausted`, which is always true of the verdict), tallies
  the goal's members, and reports the first atom by the sequent's order
  whose summed interval excludes zero (`Tally::unbalanced`, through
  `Counts::ranked`, the atoms that have rows by rank), else the count
  equation when `Rules` applies it, with the goal's `⊗`, `⅋`, `1` and
  `⊥` counted. Why the goal's sums are a refutation although the engine
  tests stable sequents only: the asynchronous phase keeps them (a `⅋`
  adds a member and a `−1` of weight, a `⊥` takes a member and a `+1`,
  a premise of `&` lies in its hull, a `?` moves a formula whose atoms
  have no rows), so every stable sequent the goal reaches fails the same
  test, under the same `Rules` the engine searched with. It runs only on
  a refutation, after the search, so no counter of a run moves; its
  time is one more `Counts` pass on a refuted sequent. Without a
  refutation from the counts (a `⊤` absorbs, weakening, exponential
  atoms) the answer is `Exhausted`, never a guess.
- **One interface every engine implements** (`Decide`, crate-private):
  `admits(&task)` refuses a goal with the error a forced engine answers
  (`NetFragment`, `NetMode`, `NetGoal`, `EngineMode`, `NotAdditive`,
  `NotHorn`), and
  `decide(&task, &options, &account, stop)` returns an `Answer`: the
  proof (`Ok(Some)`), an exhausted search (`Ok(None)`) or the reason it
  stopped, the counters, and the net the net engine found.
  `Engine::implementation` maps each public variant to its
  implementation (`focus::ONE_SIDED`, `focus::TWO_SIDED`, `net::Nets`,
  `additive::Additive`, `horn::Horn`); a `Task` is what they are handed (forest, goal,
  fragment, mode, reading, whether it is the roots). An engine that keeps
  its proof as nodes of an arena answers through `Answer::of_arena`.
  **`prove_goal` is the one place an answer becomes a `Verdict`**: the
  refutation of an exhausted search, the check of a proof of the roots
  (and, with the check off, a `debug_assert!` of it), whatever engine
  ran. Which options each engine reads is said on its `Engine` variant
  and on each setter of `Options`; an option an engine does not read is
  documented there, never refused, since the dispatch may pick an engine
  the caller did not name. A new engine is a variant, an implementation
  of `Decide`, a line in `Engine::implementation`, and where it is the
  default a row in `DISPATCH`.
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
  connective or unit (`TwoFormulas`) to `additive`; unit-free MLL in a
  linear mode, the roots with no literal more than `NET_MULTIPLICITY`
  (2) times (`FewEqualLiterals`) to `net`; the rest by mode.
- **Why the net row is kept** (measured at step 26 on one thread): on
  unit-free MLL with few equal literals the focused engines are as fast
  as the net engine or faster (the classical `wide` sequents from 8 to
  1 024 literals within a factor of three; intuitionistic wide and
  curried sequents of 1 024 and 4 096 atoms 7 to 18 times faster), but
  they recurse once per link of a chain of stable sequents, and only the
  net engine decides `wide-m1` at 2 048 (0.64 s) and the chain
  `a₀, a₀ ⊸ a₁, … ⊢ aₙ` at 1 024 and 4 096 links (71 ms, 0.54 s) within
  the default recursion limit. The feature that would pick the faster
  engine is the depth a derivation needs, which no pass computes; a
  focused search that hands over to the net engine at the recursion
  limit is the follow-up. Multiplicity is the proxy for where the net
  engine loses: equal literals are interchangeable partners, and the
  linking search pays a permutation's worth of nodes for every wrong
  choice among them (Horn encodings: Partition and 3-Partition 10 to
  10⁵ times slower, intuitionistic and classical alike, `bench/RESULTS.md`
  and the step-26 run).
- **Intuitionistic mode** first computes the `Reading`
  (`Error::NotIntuitionistic`, whose message has ids; the CLI describes it
  with formulas) and refuses Mix (`Error::IntuitionisticMix`: a Mix premise
  would have no goal); then the same rows, with `two_sided` in place of
  `focus`, and `net` on unit-free IMLL by the embedding (below).
  `Options::engine` forces an engine; `Engine::Net` on a fragment outside
  unit-free MLL, asserted or detected, is `Error::NetFragment`, and in
  affine mode `Error::NetMode`; `Focus` in intuitionistic mode and
  `TwoSided` in classical mode are `Error::EngineMode`; `Additive` on
  anything but two additive-only formulas is `Error::NotAdditive`; `Horn`
  on a goal that is no Horn program is `Error::NotHorn`, and in affine
  mode `Error::EngineMode` (each engine's `Decide::admits`). A new engine's `Engine` variant has a
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
  `prove_until`, and it answers `Unknown (Reason::Stopped)`. **Where it
  is polled**, which is every place a search can spend time without
  reaching another of them:
  - *The focused engine*: once per stable sequent (`prove_stable`); once
    every `SPLITS_PER_POLL` (4096) steps of its searches for the splits
    of a `⊗` or a Mix (`poll_splits`, on a counter of its own,
    `Engine::steps`: a split search whose splits fail in focus visits no
    stable sequent and can run for minutes); once every
    `FORCED_PER_POLL` (4096) forced splits and literals of tensors
    closed in place (`poll_forced`, counter `Engine::forced`: a chain of
    forced splits visits no stable sequent either, and a marking of a
    Petri net is a tensor of thousands of literals); and on a pool at
    every `&` (`with_parallel`, below). The first two pass the work
    done since the last poll (`Stop::fired(work)`), which only the two
    searches of the default bias count (`Stop::Slice`, `Stop::Turn`);
    the chain's poll passes none, so that the slices of those two
    searches are what they were before it existed and the counters of a
    decided run did not move.
  - *The net engine*: once per literal chosen (`decide`) and once per
    exact test that fails (`explore`): a run of failures chooses no
    literal.
  - *The set-up*, on a forest of `SET_UP_POLL` (65 536) occurrences or
    more (`set_up_stopped`): in `prove_goal` once the fragment, the
    reading and the dispatch are done, in `focus::search_goal` (and the
    pool's) after the classes and after the plan, and inside
    `Counts::new_until` every 65 536 occurrences visited, which is the
    longest pass. On the library's largest problem (`SYJ212+1.020` in
    its cbv translation, 27.8 million occurrences) the first poll comes
    after 0.12 s and no two are more than 0.2 s apart; without them the
    first came after 1.27 s. A smaller forest gets none of these polls:
    a pass takes under a millisecond there, and a condition that counts
    its polls (a test, a front end that counts work) sees the engine's
    own and no others.
  - *Not polled*: `Forest::new` (0.43 s on that problem; a caller with
    a deadline builds the forest itself, as the CLI does, and calls
    `prove_goal`), a single pass over the forest, the check of the
    proof at the end of `prove_goal` and the size pass of a derivation
    (below), `sequentialize`, and the collection of the kept arena
    when a memo is emptied (one pass over the kept nodes, milliseconds
    at a million of them). Freeing a full memo is no longer among
    them: its entries are records in chunks ("The memory bound",
    below), so emptying one resets a count and dropping one frees a few
    hundred blocks. When every key was two allocations, a memo at its
    cap of 2²⁰ entries took 0.15 to 0.55 s to free, which was what a
    stop was late by and a fifth of the time of a memo-bound search;
    now a stop on `qbf/40#1` with its memo full comes 14 to 21 ms
    after the limit on the machine's three kinds of core.
  - *The check and the size pass are not polled because they are
    short*: on the largest proof the engines find in the LLTP library
    (`SYJ202+1.005` in its cbv translation, 566 490 inferences) the
    check takes 22 ms and `Proof::derivation_size`, the same pass with
    an observer, 39 ms; on the largest Petri nets proved 3 to 6 ms and
    5 to 8 ms. Poll them when a proof a hundred times that size is in
    reach.
  A condition must be cheap, because it is asked at every poll, and it
  must not ration its own work by counting polls: polls come millions
  of times a second on a small problem and 30 ms apart on a forest of
  millions of occurrences, so "look at the clock every 1 024 polls" was
  exact on the first and half a minute late on the second. The CLI and
  the harness read a flag that a timer thread raises. The crate docs in
  `lib.rs` show the common path (parse, fragment, prove, derivation, JSON)
  as a doc test; keep it the shortest correct program when the API moves.
  The focused engine recurses on the caller's stack, bounded by
  `Options::recursion_limit`; a caller that raises the limit runs the
  search on a thread with a larger stack (`Options::stack_size`). The net
  engine and its sequentialization keep stacks of their own.

## The memory bound

`Options::memory_limit` (`DEFAULT_MEMORY_LIMIT`, one gibibyte; `None`
lifts it) bounds what a search holds, and `search/memory.rs` is how:
an `Account` (the bound and an atomic count of bytes) that everything
which grows charges where it allocates, by the capacity allocated and
not by what is in use. `prove_goal` makes one per search.

- **What counts**: the focused memo (its chunks, its index, the extra
  copies), the kept arena and every engine's pending stack, the branch
  stack of keys, every pool buffer (sets, contexts, keys, tallies,
  split counts, cursors when made; lists, trails and links by what they
  had grown to when last given back), the `Counts` (rows and
  per-occurrence arrays) and the `Classes` of the set-up, and the
  additive path's memo and arena. **What does not**: the forest and the
  sequent (the caller's; `Options::occurrence_limit` bounds them,
  below), the proof returned, a `Tally`'s and a `Split`'s `touched`
  lists (bounded by the rows of a sequent's members), a context's
  extra list, the table a collection uses while it runs (four bytes a
  kept node), the stacks of the threads, the net engine (a structure
  and a scratch linear in the forest, per thread), and the allocator's
  own overhead. Measured, the process's peak is the count plus what it
  takes to hold the input: R8 (`SYJ202+1.008` in cbv) under 256 MiB
  ends by the bound at a peak of 253 MiB, under 16 MiB at 20 MiB.
- **The order of answers** when memory runs short: a memo that has no
  room for a new key is emptied and the kept arena collected
  (`Engine::remember`; the same as at `memo_limit`, which stays as the
  finer knob: it is what the pinned counters depend on, and a table
  that fits the cache can beat one that fits the memory); a search
  that finds itself over the bound at a stable sequent empties the
  memo, collects, and if that is not enough gives the memo's memory
  back (`Engine::relieve`); `Unknown(Reason::MemoryLimit(bytes))` when
  what is left, the branch's own buffers and proofs as allocated, is
  still over, or when an empty memo cannot have its first chunk. The
  value in the reason is the option's, whatever share a search had
  (`focus::reason`). `relieve` does not squeeze the arena to fit: the
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
- **`Options::occurrence_limit`** (`Forest::DEFAULT_LIMIT`, fifty
  million) is the bound on the input: `prove` and `prove_until` build
  their forest with `Forest::within`, the command checks
  `Sequent::occurrences()` when it reads a sequent, before any command
  unfolds or prints it. `Forest::new`, and with it `Interactive::new`
  and every deserializer (a proof file, a session's state, a net), has
  the default and no way to pass another, since `Deserialize` takes no
  options: a proof file whose sequent has more occurrences is refused
  whatever a flag says.
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

- **It refutes nothing whose search can contract**: a failing branch with
  a `?` formula (or a `!` hypothesis) contracts to an empty budget and is
  cut. With exponentials it judges an engine's "unprovable" only; an
  engine's proof is the checker's.
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
- A new engine joins `configurations` in its tests.
- **A second reference, kept outside the repository**, judged every
  change of the search at step 26: written by a fresh agent from the
  calculus alone (its own parser of the text syntax, every rule on every
  formula, contraction budgeted per branch), it agreed with this one on
  every sequent both decided. It lives in that session's scratchpad and
  is not maintained; a step that changes the search writes or asks for
  its own, so that the panel's independence does not rest on this one
  file.
