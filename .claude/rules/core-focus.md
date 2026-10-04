---
paths:
  - "core/src/search/focus/**"
  - "core/src/search/generate.rs"
---

# linlog core: the focused engine

Loaded, beside `core.md`, when a file of the focused engine or the
test-only generator is read. Where it polls its stop and what its
memory account counts are in `core-search.md`; the pool it runs on is
`core-parallel.md`.

## The focused engine

**Files.** `mod.rs` holds the engine (`Engine`, built only by
`Engine::new` from a `Problem`: the forest, reading, counts, classes,
rules, account and limits every engine of a search shares), its phases
and the interface the front door calls (`Focused`, `ONE_SIDED`,
`TWO_SIDED`); `split.rs` the `⊗` rule and Mix (forced chains, the split
search); `arena.rs` the proof arena; `scratch.rs` the pools of buffers
(`Pools`); `schedule.rs` the two searches of the default bias (`plan`,
`chains`, `Rule`, `turns`, and the threaded `alternate` with its baton);
`parallel.rs` the engine on a pool; `tests.rs` the tests; `bias.rs`,
`classes.rs`, `context.rs`, `counts.rs`, `memo.rs` as named below.

`search/focus/mod.rs` is the spec's MALL-Seq and MELL-Seq in one engine, for
every classical fragment up to full LL, with units, Mix, the exponentials
and affine mode as rule switches (`Rules`, from `Fragment` and `Mode`), and
the spec's two-sided engine for every intuitionistic fragment when given
the sequent's `Reading` (`Engine::TwoSided` is that configuration). Its
functions are the spec's rules: `asynchronous` (the phase `⊢ Θ ; Γ ⇑ L`),
`quest` (`?` into `Θ`), `prove` (a stable sequent), `focus` (`⊢ Θ ; Γ ⇓ F`),
`initial` (the two initial rules), `split` (the `⊗` rule), `mix`. What it
relies on:

- **Atom bias** (`focus/bias.rs`, `signs(forest, rule)`, a function of
  the sequent alone, so a run stays deterministic; `Counts::new_until`
  reads it once per search and the engine reads polarities off
  `Counts::positive`, so that the option reaches every place a literal's
  polarity matters, `literal_tensor` included; it lived in the forest
  until the forest's only reader was this engine).
  Focusing is
  complete for every assignment of polarities to atoms, so the bias is
  chosen for speed and can never change what is provable. Without an
  exponential in the sequent: per atom, the literal that is more often a
  direct factor of a `⊗` is positive, each occurrence weighted by ½ per
  `&` or `⊕` above it (a proof takes one side of a choice, so the two
  heads `~d` of a clause `(… ⊗ ~d) ⊕ (… ⊗ ~d)` count as one); a `⊗`
  with a positive literal factor has its split forced. On a tie, and
  whenever the sequent has a `!` or `?`, the old rule: the literal with
  fewer occurrences is positive, a tie makes `Var` positive, so an atom
  with one sign only has all its literals negative. The engine takes
  the old rule in affine mode as well (`schedule::plan`): nothing forces a
  split there, so the factors have nothing to say, and a review measured
  up to 700 times the stable sequents on generated affine sequents with
  the factor rule. With exponentials the bias decides the shape of the
  focused proofs, hence the copies a branch needs, and neither rule
  wins: the counter family chains forward under the factor rule and
  needs `n − 1` copies on its one branch where the rarer-literal rule
  needs `log₂ n`, and at a bound of `n − 1` the factor rule decides the
  counter with 16 tokens in 19 stable sequents instead of 473 232. So
  `Bias::Auto` is the rarer-literal rule there, and the engine's
  default runs a search under each ("The default bias with exponentials
  is two searches", below). `Options::bias` names the rules:
  `Bias::Rarer` and `Bias::Factors` for any sequent. Rules tried on the
  exponential-free targets and not taken: `Var` always (as good on the
  families written two-sided, where it is forward chaining, but it
  depends on how the atoms happen to be written and loses the gains on
  Partition and the wide sequents), `DualVar` always (30 to 300 times
  more stable sequents on the Horn families), the factor count without
  the ½ (the heads of the 3-Partition clauses outvote the goal: 317 138
  stable sequents against 923 at bins of four). Measured, old rule and
  new (stable sequents, splits): unsolvable 3-Partition with bins of
  five 3 373 and 41 160 against 971 and 36 072; QBF 20 #2 105 667 and
  1 037 858 against 60 883 and 97 394; Partition with seven items 179
  and 6 008 against 178 and 2 078; `wide-m3` at 30 91 and 768 against 31
  and 688.
- **Two-sided is one constraint.** Every rule of the two-sided focused
  calculus is a rule of this engine on the lowered sequent (`⊸R` and `⊗L`
  are `⅋`, `⊸L` is a `⊗` in input position, `!L` is `quest` plus a copy,
  and so on), and starting from one output-shaped root every rule keeps
  exactly one output on each premise by itself, except the split of a
  hypothesis `A ⊸ B`, where the goal must go with the consequent `B⊥`.
  So `split`, in its search for the free splits, fixes the one output
  member of `Γ` on the consequent's side (`Reading::implication`) and
  assigns the rest; the forced splits need no change (the dual of an output positive
  literal is a hypothesis in `Γ` or `Θ`, the dual of an input positive
  literal is the goal itself or nothing, `1` and `!` are output-only, a
  `0` factor fails), `Θ` holds only input occurrences, a leaf's `weakened`
  never sees an output (debug-asserted), promotion needs `Γ` empty as
  before, and the count prunes are necessary conditions on the lowered
  sequent, hence sound. Mix is refused before the engine runs. The memo,
  the copy budget, the loop check and the pools are indifferent to
  positions: the key `(Θ, Γ)` determines the two-sided sequent. A
  fresh-context review compared the engine with an independent unfocused
  two-sided prover on about 60 000 sequents over every ILL connective,
  linear and affine, with no disagreement.

- **Dyadic sequents.** `Θ`, the unrestricted zone, is an `OccSet` of the
  subformulas of the `?` formulas decomposed on the branch; it only grows
  along a branch, is shared by every premise, and a `?A` whose `A` is
  already there changes nothing. `Γ`, the linear zone, is a `Context`
  (`focus/context.rs`): a bitset plus a sorted list of the extra copies of
  occurrences present more than once, empty until a copy repeats an
  occurrence (a copied `~a ⅋ ~a` releases the same `~a` twice), which is
  the one allocation on the hot path. Member lists (`gamma.iter()`) carry
  repeats, and a split search assigns positions, so it reads a member's
  side off its own trail, never off `contains`. A `Context` keeps the
  range of its bitset's words that may hold a member (`lo..hi`, widened
  by an insertion, kept by a removal), so that `clear`, `iter`, `len` and
  `is_empty` cost the range and `clone_from` the union of both ranges,
  one copy (the source's words outside its range are empty, so they
  clear the target's); a zone of at most `NARROW` (8) words keeps every
  word in range, since there the bookkeeping cost more than it saved; equality and the hash are of the members, never
  of the range, since the memo and the loop check compare zones by
  them. On a net of tens of thousands of occurrences a stable sequent's
  members lie in a few words: the free splits of
  `NeoElection_neoelection-7.unf_10_1` listed, copied and cleared the
  whole width per split (86 % of its time); with the range it visits ten
  times the stable sequents in its 5 s. The memo's key still holds both
  zones at the forest's width (a sparse key is the follow-up).
- **Stable sequents only.** The asynchronous phase runs to completion (`⅋`
  opens, `⊥` drops, `⊤` closes with a `Top` node and the pending `⅋`/`⊥`
  nodes wrapped around it, `&` branches on copies of the state, `?` moves
  its subformula into `Θ` under a `Quest` node); what reaches `prove` is
  `Θ` plus a `Γ` of positive formulas and negative literals, and only those
  are memoized. `search_goal` starts from any multiset of occurrences with
  an empty `Θ` (an interactive prover's open goal); `search` starts from
  the roots.
- **The copy budget.** A copy (rule D2: focus on a `Θ` member, which stays
  there, under a `Copy` node) costs one unit of a per-branch budget passed
  down the calls; so does the initial rule `⊢ Θ, p⊥ ; · ⇓ p`, emitted as
  `Copy(p⊥)` above `Ax(p, p⊥)`, so that the bound counts every `?d` of the
  derivation. `run` deepens the budget from 0 to the search's bound
  (`Options::copies`; the forward search of the default bias may have
  a larger one, below), or without end where `copies` is `None`
  (`Options::copy_bound` is then `u32::MAX`, which the inclusive range
  reaches without a wrap and no search reaches at all: every level
  visits a stable sequent). `Statistics::copies` is the budget of the
  last level begun, of two searches the larger (`Statistics::add` takes
  the maximum), which is how far an unbounded search got when its stop
  fired. The library's default keeps `DEFAULT_COPIES` (3): `prove` has
  no stop condition, and a search without a bound ends only when it
  decides; the command's default is `None` under a time limit. A
  level whose search skipped a copy for lack of budget returns
  `Cuts::exhausted`; `Unprovable` is answered only by a level whose
  result has it clear, and `Reason::CopyBound` when every level's has it.
  Each stable sequent reads the cuts its decision returned, so the memo
  entry says whether *that* subtree was cut. Without exponentials there
  is one level with budget 0 and nothing is cut.
- **Memo contract with the bound** (`focus/memo.rs`). The key is both
  zones. `Proved(NodeId)` is a fact at any budget (a proof is a proof; one
  found with more copies than the current level allows is still returned,
  so the bound limits the search, not the proof: `Proved` at level `k`
  does not mean a proof with at most `k` copies per branch, and a
  reported minimum would have to be budget-aware).
  `Failed(Complete)` (the subtree was explored to the end without hitting
  the budget, and without a prune that depends on an ancestor, below) is a
  fact at any budget, since more budget adds nothing that was not tried.
  `Failed(Exhausted(r))`, cut by the budget with `r` copies left, applies
  only when at most `r` are left now (`Memo::get`), and its hit sets
  `Cuts::BUDGET`; a later entry only raises `r`, and `Complete` or `Proved`
  replace it. Entries survive across levels; that is where the
  re-exploration of deepening is recovered. Never memoize across forests.
- **The memo's layout** (`focus/memo.rs`): an entry is a record of
  words in a chunk (the key's hash, the entry packed into a word, the
  place of the linear zone's extra copies, both zones' words), found
  through an index of record numbers with linear probing, at most half
  full. Chunks have a fixed number of records, a power of two, about a
  mebibyte, or a sixteenth of the room there is under a small bound.
  So an entry costs no allocation and is counted with its chunk,
  `clear` resets a count and zeroes the index, keeping the memory for
  the entries to come, and `release` or a drop frees some hundreds of
  blocks. The table is never iterated and never deletes, which is what
  makes the index this simple. Per entry: 24 bytes of header, eight for
  the index, and the two zones as bitsets of the forest's width, which
  dominate on a forest of thousands of occurrences (a sparse form of
  the zones is the next thing to gain, and a follow-up).
- **Complete failures are keyed up to interchangeable members of `Γ`**
  (`Context::canonical_from`: every member replaced by its class, the
  canonical key; `Θ` stays as it is). By the lemma on interchangeable
  occurrences a `Complete` failure holds for every sequent with that
  canonical key, so it is recorded there and `Memo::refuted` reads it
  for any of them. A proof names occurrences, so `Proved` stays under
  the sequent's own key; renaming a proof on a hit was not taken, since
  an occurrence of the proof may lie below a member of `Γ` and below a
  member of `Θ` at once, and which replacement applies depends on the
  path it came by. **`Exhausted` stays under the sequent's own key
  too, and must**: it is as true of a relative as of the sequent (a
  replacement keeps a proof's shape and copies), but shared it defeats
  the deepening. A sequent whose search reaches a relative of itself
  one copy lower was answered by its own entry of the level before, so
  it was cut again and recorded `Exhausted` one higher, at every level,
  where the search of the relative itself would have ended in a repeat
  and a complete failure; a review found 167 generated sequents that the
  engine refuted before and that stayed at the copy bound, and putting
  the canonical keys on the branch stack as well cured only those whose
  relative lies below them (`repeats_up_to_equal_members` pins one of
  each kind). So among relatives only facts that no budget qualifies
  are shared, and the loop check compares the sequents' own keys, as it
  always did. One table holds everything: a key that is not canonical
  holds a proof or an `Exhausted`, a canonical key may also hold a
  `Complete`, which is all a relative reads there; when the two keys
  are equal, or no two occurrences of the forest are interchangeable
  (`Classes::distinct`, which skips the canonical key altogether), the
  one `get` of before. Keying `Θ` by class too would merge more (a `?`
  below interchangeable members adds different ids) and is not done: it
  would cost a pass over `Θ` per stable sequent. Measured (stable
  sequents, memo entries): the unsolvable 3-Partition with bins of four
  4 761 and 509 before, 2 991 and 296 after; nothing on the families
  with exponentials, whose failures within a level are mostly cut ones
  (the counter with 16 tokens 473 232 and 1 049 either way).
- **The loop check** uses the branch stack of stable sequents (`stack`,
  live up to `stack_len`, entries reused), on with exponentials only: a
  stable sequent equal to an ancestor is pruned, because a smallest proof
  of the ancestor never passes through it. Such a failure is a fact about
  the branch, not the sequent: `Cuts::dependency` carries the shallowest
  ancestor depth a prune below relied on, a failure that carries a
  dependency on an ancestor is not memoized, and the dependency is
  discharged at that ancestor, whose own failure is genuine (a proof of
  the repeat would be a proof of the ancestor). Order in `prove_stable`: a
  `Proved` or `Complete` memo entry answers first; then the stack; then an
  `Exhausted` entry, so that a repeated sequent is pruned rather than
  reported as cut by the budget. Pruned branches are never cut by the
  budget.
  Every stack entry has its hash beside it (`hashes`), compared before
  the sequents: the ancestors of a branch mostly share `Θ` and the set
  of `Γ`, so a comparison of sequents ran over both bitsets before it
  met the difference (the `growing` family at a bound of 1 024 took
  530 ms of CPU for 392 961 stable sequents, 260 ms with the hashes).
- **The default bias with exponentials is two searches** (`focus::plan`,
  `search_goal`). `Bias::Auto` on a goal with exponentials in linear
  mode, classical or intuitionistic (the fragment searched and the
  forest both have a `!` or `?`, the mode is not affine), is decided by
  the *backward* search, `Bias::Rarer` within `Options::copies`, which
  is what `Auto` was alone before, and by the *forward* search,
  `Bias::Factors`, each an unchanged search of the engine (`Rule`: a
  bias and a copy bound) with a memo, an arena, a branch stack and
  counts of its own. The first to decide answers. What the code relies
  on, and what it promises:
  - **The contract, never less than the backward search.** With no stop
    firing, `Auto` answers `Proved` or `Unprovable` wherever
    `Bias::Rarer` does under the same options, and wherever
    `Bias::Factors` does, with the same verdict. The argument is an
    identity, not an estimate: each of the two is the explicit search
    itself. Where they alternate in slices, a search is never
    restarted, only made to wait, so its run is the explicit one
    counter for counter (`default_bias_takes_turns` pins the sum);
    where they take turns from their start, every turn begins with a
    fresh `Engine`, memo and arena, so a turn is a prefix of the
    explicit run and the turn that is not cut *is* that run. `Auto`
    ends only on a decided result or when both searches ended. The
    forward search's levels up to `copies` are those of `Bias::Factors`
    under the same options (the deepening is level by level, so a
    larger bound continues the same run).
    `default_bias_decides_what_either_rule_does` pins it on generated
    sequents, classical and intuitionistic, with and without the memo.
    On a pool the same holds up to the pool's own caveat (decisiveness
    within the bound depends on the interleaving).
  - **Nothing is shared between the two searches but the verdict**, and
    the forest, the reading and the `Classes`, which are not the
    search's. A proof and a complete failure are facts under either
    bias, since provability does not depend on it, and sharing them
    would be sound; it is not done because it would break the identity
    above: an entry from the other search changes which entries this one
    makes, and with them its decisiveness at the bound (the engine's own
    "the memo can change decisiveness within the bound", which a review
    of the first pass saw on real cases when complete failures were
    shared among relatives). A failure cut by the budget is a statement
    about one rule's search space under one budget, and the loop check
    about one branch of one search: neither means anything to the
    other search. The same reason keeps a search's own memo out of its
    next turn where turns restart it. The price is memory: two memos of
    at most `Options::memo_limit` entries each where the searches run
    at once, and for the same reason each search has half of
    `Options::memory_limit` (`Account::share`): one search's memory
    must not decide what the other may keep. So under a memory bound
    the contract reads "wherever `Bias::Rarer` does under the same
    options with half the memory".
  - **`Unprovable` keeps its meaning**: a level of either search that
    ended without a cut, which refutes the sequent because focusing is
    complete for every bias. `Unknown` needs both searches to have ended
    undecided; its reason is the backward search's, and a copy bound is
    reported as `CopyBound(Options::copies)`, which is true of both
    searches (the forward one was cut at every level up to its own
    bound, which is at least that).
  - **Without a copy bound** (`Options::copies(None)`, the command's
    default) both searches deepen until one decides or the stop fires,
    and the forward bound has no effect (the larger of no bound and 30
    is none). The identity above holds as it is: each search is the
    explicit one without a bound, whose levels up to any `n` are those
    of the same search under `Some(n)`, so with no stop firing the
    unbounded default decides whatever the default under any bound
    decides, and whatever either explicit search decides. The price is
    the backward search's share: on a sequent where the forward one
    used to end at its bound and hand over the core, it keeps a third of
    the work (on one core) until the stop.
  - **The forward bound** is `Options::copies`, and the larger of that
    and `Options::forward_copies` (`DEFAULT_FORWARD_COPIES`) where the
    goal is a Horn program (`chains`, which reads the goal's members: a
    goal off the roots, as an interactive close hands over, gets the
    bound by its own shape, and a clause used once, as a dereliction leaves
    one in the linear zone, is a step of the program like a copy;
    `the_horn_test_reads_the_goal`) and the mode has no Mix. A
    program: every member of the goal under a `?` is a clause, a tensor
    of body literals, all of one sign throughout the goal, with at most
    one factor a head instead, a literal of the other sign or a `⅋` of
    such (what `!(a ⊗ b ⊸ c ⊗ d)` lowers to); every other member is a
    marking, a `⅋` of head literals, a goal, a tensor of body literals,
    or a clause used once; `1` and `⊥` stand for an empty body, goal or
    head. Off the roots the bound moves an answer both ways between
    decided and "unknown" (a panel measured 529 runs decided only with
    the goal's test and 310 only with the roots', before the clause used
    once was taken), and under a stop a forward search that runs on
    keeps its share of the work, so an "unknown" by the copy bound may
    become one by the time limit. A
    Petri net with a marking to reach is exactly that. Why a bound of its own: a forward chain takes
    one copy per step on one branch, where the same derivation
    backward takes as many as its tree is deep, so no multiple of
    `copies` converts one into the other. Why only on Horn clauses: a
    copy of a clause rewrites `Γ` and opens no branch for further
    copies, so a level costs the markings reachable within it, which
    the memo holds once each; on arbitrary formulas a deeper bound
    multiplies the search by the copies' alternatives per level, and
    with the bound applied everywhere the generated tests no longer
    finished. The test is on the whole goal and on the signs because
    a first version that looked only at the shape of the formulas under
    `?` let through `(c ⊸ c), !((c ⊸ b) ⊸ c), 1 ⊢ 1 ⊸ 1 ⊗ c`, which
    answered "unknown" in 0.02 s before and took sevenfold per copy
    with the bound (a review's finding). What remains is the price of
    the bound on a real program whose markings grow: `!(a ⊸ a ⊗ b),
    !(1 ⊸ b), !(b ⊸ a ⊗ b), !(a ⊗ a ⊗ b ⊸ a), a ⊢ 1` answers
    "unknown" after 0.6 s where the backward search alone took 0.02 s,
    and of a review's 2 000 random programs 45 took over a second to
    an "unknown" that took under 0.1 s before, against 471 that are
    decided now and were not (3 and 453 at a bound of 10) (the LLTP translations of intuitionistic problems that
    `--copies 10` decides are decided by the bound, under either bias,
    and stay the user's `--copies`). Why not under Mix: every stable
    sequent a chain leaves unproved is tried in every partition, and a
    chain that grows them (`⊢ !?(~a ⅋ c)`) did not finish at a bound of
    10. `Options::copies` keeps its documented meaning for the backward
    search and for either bias named explicitly; with `forward_copies`
    at 0 the forward search runs within `copies` everywhere.
  - **When the two rules agree on every atom** the forward search is the
    backward one continued, and runs alone (its levels up to `copies`
    are the backward search's).
  - **The unit of work** is the engine's own, since the crate has no
    clock: a step of a split search is one, a stable sequent
    `NODE_WORK` plus what grows with its size (the forest's width for
    the zones, the members, the copies and their comparisons with the
    members in `meets`), a split whose premises are tried the forest's
    width again. The engine adds these up in `Engine::work` and hands
    them to the stop condition at its two polls (`Stop::fired(work)`);
    `Stop::Closure` and `Stop::Flags` ignore them, so nothing changes
    for a search that runs alone. So a run is a function of the input.
    The unit follows the time only roughly: what a stable sequent and a
    split step cost varies by two orders of magnitude between problems
    with the sizes of `Γ` and `Θ` (a forced chain's lookups of duals
    are not counted at all), so in seconds one search's share can be
    several times the other's. Counting a poll as the unit made the
    backward search's share a hundred times too long on Petri nets (a
    poll in a split search is 4 096 steps), and a flat cost per stable
    sequent starved a net's backward proof.
  - **On one core with threads** (`focus::schedule::alternate`, feature
    `parallel`, whatever `Options::jobs` says below two): the forward
    search runs on the calling thread and the backward one on a scoped
    thread of `Options::stack_size()`, and a `Baton` lets one of them
    run at a time: a search gives way after a slice of work
    (`Stop::Slice`, `SLICE`; the backward search gets `BACKWARD_SHARE`
    = 2 slices' worth) and waits for its turn. Nothing is restarted and
    nothing depends on the scheduler, so the statistics are the two
    searches' own, added up, and a function of the input. A search that
    decides stops the other at the end of its slice; one that ended
    undecided leaves the other to run on, and the calling thread then
    wakes once a millisecond. The caller's stop is not `Send`, so it
    lives on the calling thread, and it is polled once for every poll
    of either search: at the forward search's own polls, and for the
    backward search's (counted in `Baton::polls`) within a millisecond
    of each, since the calling thread wakes that often while the
    backward search has its turn (`Baton::pass_polling`) and after the
    forward one has ended (`Baton::caught_up`). It used to wait for the
    whole slice on the condition variable, and a slice is counted in
    work, not in time. A condition that counts its polls or reads a
    clock every `n` of them therefore sees what it sees of one search; polled only once per
    wake-up, it was a second late (a review's finding). The backward
    search reads `Baton::halt` at every poll. `StopOnPanic` stops it
    when the caller's condition panics. `Ended`, dropped on return and
    on a panic, hands the baton on so that nobody waits for a thread
    that is gone. What it costs against the better rule alone, in units
    of work `W`: `1.5 W` when the backward search decides and `3 W`
    when the forward one does, plus a slice. Why the backward search
    gets twice the work: it is what the default was before, so under a
    time limit everything it decides alone in two thirds of the limit
    stays decided; at equal shares two nets that it proves in 2.1 s and
    1.9 s (`NeighborGrid_z_2d_3n_1m_t_1_2_10_1`, `UtahNoC_5_1`
    classically) were not proved within 5 s, the forward search's units
    being slower there.
  - **On one thread without threads** (`focus::schedule::turns`, the fallback when
    the feature is off or the thread cannot start; `FIRST_TURN`,
    `TURN_GROWTH`, `Stop::Turn`): round `i` gives the forward search
    `FIRST_TURN · 4^i` units and the backward one twice that, each
    turn from the search's start; a search that ended undecided takes
    no further turn, and the other then runs without one. `Statistics`
    adds up every turn (the memo's entries are the most of one turn).
    The turns that end early are a geometric series, so the run takes
    less than `5 W` whichever search decides after `W` (its own cut
    turns, under a third of its last, and the other's up to that
    round); a search that decides within its first turn costs what it
    costs alone, plus one first turn of the forward search when it is
    the backward one. This scheme was the first built and measured: on
    the LLTP sample it decided what the alternating one does, at three
    times the time of the better rule in the sum and up to twenty times
    on single nets, and it lost eight of the 1 890 LLTP rows the first
    baseline decided to the 5 s limit, which is why threads are used
    where they exist.
  - **On a pool** (`focus::parallel::search_goal`, `search::parallel::
    race`, `Rule::search_on`) the two searches run side by side, each
    with its own shared memo and arena, the forward one on a pool of
    `jobs / 2` threads and the backward one on a pool of the rest, and
    a decided result raises the other's root flag. Two pools and not
    one, because a pool thread that waits at a scope runs stolen tasks:
    on one pool a thread of the search that has just decided can be
    deep inside a task of the other, which nothing stops, and the
    verdict waits for it. A pool of one thread runs the sequential
    engine (`Rule::search_on` leaves `runtime` unset). The merge
    (`merged`, shared with `alternate`): a verdict of either; else
    `Stopped` when either was stopped, which without a verdict can only
    be the caller's stop; else the backward search's reason.
- **The spec's affine prune is wrong and is not implemented.** It prunes a
  stable sequent that *contains* an ancestor as a multiset, arguing that
  weakening shortens the proof; but weakening turns a proof of the smaller
  sequent into one of the larger, never the reverse, and `⊢ ?(a ⅋ ~a)` is
  provable only through `⊢ a ⅋ ~a ; a, ~a`, which contains the root. A
  review found 426 wrong `Unprovable` verdicts in 5 200 random affine
  sequents with it. The same holds with the zones equal. So affine mode
  is not a decision procedure here: it runs the bounded, loop-checked
  search of linear mode with weakening, and answers `CopyBound` like it.
  (The prune in the other direction, a sequent *contained in* an
  ancestor, is sound but useless: it is the useful branch.)
- **Affine mode** has no relaxed rules in the term: a leaf (`Ax`, `One`,
  `Bang`) weakens every leftover member of `Γ` below itself (`weakened`,
  one `Weaken` per copy); weakening never goes above a promotion. Nothing
  but `0` forces a split in affine mode (`forced_side`), every dual pair
  or literal with its dual in `Θ` closes a stable sequent (`initial`, which
  goes on to the next pair when the budget refuses a copy), `1` and `!`
  are candidates with any context, a `0` is not fatal (it is weakened at a
  leaf), and the interval check and the count equation are off
  (`Rules::intervals`, `Rules::equation`): weakening discards any
  imbalance.
- **The rules with `Θ`.** D1 candidates first (`⊗`, `⊕`; `1` and `!` only
  when alone), then the copies from `Θ`: a member with an unconsumed copy
  in `Γ` is skipped (a second copy cannot help before the first is used,
  and the two are the same formula), those with a literal whose dual is a
  member first (`meets`), then by id; a negative `Θ` member is copied and
  released. `!A` in focus needs `Γ` empty and releases `A` into an empty
  `Γ` under a `Bang`. A positive-literal factor of a `⊗` takes its dual
  from `Γ` when there is one and otherwise leaves its side empty for the
  `Θ` initial rule; the dual in `Γ` first loses no proof, since the copies
  are the same formula and a proof that spends this one elsewhere and
  copies here is the same proof with the roles swapped, but the swap moves
  a copy to another branch, so a sequent may need one level more than its
  best proof's copies per branch (`⊢ ?~p, ?p, ~p, p ⊗ ⊥` is proved at
  bound 2, not 1).
- **Memo validity without exponentials** is unconditional, as before: cut-
  free provability of a set of occurrences depends on the set alone, and
  every entry is `Proved` or `Complete`. When the table is full it is
  cleared (`Options::memo_limit`; zero switches it off) and the kept
  arena collected; a `Proved` id never dangles, because the entries
  that named the dropped nodes are gone with the table.
- **The proof arena has two parts** (`Arena`): a node is *pending* in the
  engine's own stack (`push`, an id with the `PENDING` bit) until the
  stable sequent it helps to prove is proved and memoized, when
  `prove_stable` *keeps* the nodes pushed since its `mark` (`keep` moves
  them to the kept arena, premises renamed, and returns the root's kept
  id); a failed step *releases* them (`release`, a truncation). The
  release points are `prove_stable` on a failure and the four places
  where a first premise is proved and the second fails (`premises`, the
  forced split, `both` for `with`, `parts`); every other failure pushes
  nothing. The
  argument that a release is safe: node ids travel only upwards as return
  values, so nothing outside the failed call holds an id pushed after its
  mark, and a memo entry holds kept ids only. The argument for `keep`:
  the nodes pending above a `prove_stable`'s mark were pushed by its own
  decision, which succeeded, so they rest on each other and on kept nodes
  (memo hits) alone, never on a pending node below the mark; that is
  debug-asserted in `append`. So the kept arena holds the proofs of
  memoized stable sequents and the final proof, and nothing of a failed
  branch: before this, one stable sequent of a Petri net pushed 110 MB of
  nodes a second for left premises whose splits then failed, and nine
  LLTP runs aborted at 16 GiB. With the memo off (`memoizes` false),
  nothing is kept before the root, so the pending stack is the partial
  proof alone. Either part holds at most 2³¹ nodes (`Arena::most`):
  `keep` answers `Reason::IndexLimit` beyond, and a `push` beyond drops
  the node and marks the arena (`overflowed`), after which every `keep`
  fails, so no proof resting on the missing node gets out (every proof
  that leaves an engine passes a `keep`: the memo's, the root's in
  `Rule::search`, a worker's in `exported`). A proof's node order is
  the order of keeping, which `core/tests/serialize.rs` pins on one
  small proof.
- **The kept arena is collected when an engine's own memo is emptied**
  (`Arena::collect`, from `Engine::remember` and `relieve`): the proofs
  of entries the memo dropped were never reclaimed, and on `qbf/48#0`
  they grew by 15 MB a second until the machine's memory was gone. A
  collection keeps what the pending nodes, the ids *held* and the root
  it is given rest on (one pass down marks, since a premise has a
  smaller id; one pass up moves the nodes that stay and renames their
  premises), and renames those three in place. What it relies on: **a
  kept id lives in exactly four kinds of place**, a memo entry (gone
  when the collection runs), a premise of a pending node, the result a
  call is about to return (the root given), and a local of a rule that
  holds the proof of its first premise while it searches the second.
  The last are `with` (and `both`), `premises` and `parts`, which put the id on the
  arena's `held` stack around the second search (`hold`, `unhold`) and
  read it back, since it may have moved; the forced split's `links`
  hold pending ids only (a forcing factor's proof ends in a node pushed
  by the chain or by `focus_on`), and `decompose`, `focus_on` and
  `split` wrap a result into a pending node before any other call. **A
  new rule that keeps an id across a call that can reach
  `prove_stable` must hold it**, or a collection in between leaves it
  pointing at another node: the proof is then wrong, which the checker
  catches (`Error::Rejected`), never a verdict. `proofs_survive_collections`
  runs generated sequents under memos of one, two and five entries, so
  that nearly every insertion collects, and checks every proof. A
  collection gives memory back when three quarters of the allocation
  are free, down to twice what stays (shrinking to fit made the next
  `keep` double the vector again, a copy of the arena per node). The
  shared arena of a pool is not collected: its workers hold ids nobody
  could rename, so there the kept proofs count toward the bound until
  the search ends.
- **A `0` is fatal only without a `⊤`.** The spec calls a `0` in a stable
  sequent fatal, but `⊢ 0, ⊤ ⊕ b` is provable through the `⊕`; the
  immediate failure applies only when no member has a `⊤` below it
  (`Tally::absorbs`), and not in affine mode. The other immediate tests: a
  dual pair succeeds; a literal-only sequent fails without Mix and with an
  empty `Θ`; an unbalanced sequent fails.
- **Counts** (`focus/counts.rs`): per occurrence a sparse row of intervals
  per atom (literals `±1`, `⊗`/`⅋` sum, `&`/`⊕` hull, units nothing), an
  `absorbs` flag (a `⊤` at or below it: the row is meaningless and any set
  containing the occurrence passes), and a `weight` `t − p − #1 + #⊥`. The
  interval check is sound in every fragment without exponentials (proof by
  induction on the rules, with `⊤` covered by the flag and `0` as `(0, 0)`).
  With exponentials, an atom with a literal below any `?` or `!` in the
  problem gets no row entries anywhere (its copies and discards break the
  balance; `Θ` members are not in the tally, and they contain only such
  atoms), and a `⊤` below any `?` or `!` switches the check off altogether
  (`absorbs_from_copies`: a copy of it absorbs any imbalance). The hull
  for `&` is the spec's choice; the intersection would be sound too and
  stronger, and is a follow-up. The count equation
  `c = t − p − #1 + #⊥ + 2` (`≥` with Mix, and `>` for a Mix to be worth
  trying) is only sound without additives, additive units or
  exponentials and without weakening, and `Rules::equation` switches it on
  for exactly those cases; `⊢ a ⊕ b, ~a` is the counterexample the spec
  names. A `Tally` keeps a set's sums incrementally; a `Split` keeps those
  of the two sides of a split in the making and what the members not yet
  assigned can still add (below). **How the rows are laid out and what
  they cost** (`Counts::new_until`): one pass from the last occurrence
  down writes each row straight after the others into three flat
  arrays, a binary node's by merging its children's, which are written
  already (`Rows::merge`; `bound[n − 1 − o]` is where the row of `o`
  starts), so there is no vector per occurrence. An entry's atom is its
  *rank among the atoms that have rows*, the non-exponential ones in
  their own order, so a `Tally` and a `Split` are as wide as those are
  many: a Petri net has tens of thousands of atoms and none with a
  row, and a `Split` of six arrays over all of them per level of
  recursion was what four Philosophers nets ran out of memory on. The
  ranks are monotone, so `first_atom` orders members as before. The
  rows together can still be quadratic in the forest (a nest of `⊗`
  and `⅋` over distinct atoms has a row as long as its subtree at
  every level): that is what a row is, so the set-up charges them to
  the account as they grow and answers `MemoryLimit`, polls by the
  entries merged and not only by the occurrences, and answers
  `IndexLimit` past 2³² entries. The atoms below a `!` or `?` are
  found in one pass that remembers where the outermost one ends; a
  walk of every exponential's subtree was quadratic in a tower of
  them (2.2 s for `!` nested 80 000 deep, before the first poll).
- **Interchangeable occurrences** (`focus/classes.rs`, `Classes`): two
  occurrences of the same term, and under a reading in the same
  position, share a class, named by its first occurrence. The lemma
  everything below rests on: *a sequent stays provable, by a proof of
  the same shape (hence with the same copies on every branch), when
  members of `Γ` are replaced by interchangeable occurrences one for
  one, and `Θ` by any set of occurrences with the same classes.* By
  induction on the proof: every rule reads off an occurrence its kind,
  its atom and sign, its subformulas and (two-sided) its position and
  whether it is an implication; equal terms give equal kinds, atoms and
  subterms, and the position below an occurrence is a function of its
  term and its own position (the reading's choice of the antecedent is
  made per term), so the premises of the rule on the replaced occurrence
  are again replacements of the premises. No rule looks at an
  occurrence's id, its parent or its root. What was checked and is *not*
  part of the definition: the zone (a replacement keeps each member in
  its zone, and `Θ` counts as a set of classes because `?` adds a formula
  that may already be there under another id); the branch stack (the
  loop check prunes a sequent equal to an ancestor, which stays sound
  when fewer proofs are searched: of the proofs with canonical choices a
  smallest one has no repeat either, since the choices are a function of
  the sequent they are made in and a subproof of a canonical proof is
  canonical); the copy bookkeeping (the rule that skips a `Θ` member
  with an unconsumed copy in `Γ` is by id and skips less than the same
  rule by class would, and by class it is the same normal form). The
  count rows, weights and `absorbs` are functions of the term and the
  problem's exponential atoms, so interchangeable occurrences have equal
  counts.
- **One of each kind** (`one_of_each`): of interchangeable focus
  candidates, and of interchangeable members of `Θ` to copy (among those
  the unconsumed-copy rule leaves), only the lowest id is tried; the
  focus on another leaves the same sequent up to a replacement. A
  repeated occurrence of `Γ` is one candidate for the same reason.
- **Canonical splits**: in `search_splits` the left side takes, of each
  class, the members with the lowest ids, so only the number taken
  varies (`C(n, k)` splits of `n` equal hypotheses become one per `k`).
  Sound and complete by the lemma: a split with another choice of as
  many gives the same two premises up to a replacement. In the order of
  `Engine::open` a class is a run with descending ids, so the rule is
  "a member goes left at once when the one before it is of its class
  and went left", which the trail decides; on the pool the patterns
  that break it are not spawned (`split_parallel`), so the chunks still
  partition the splits searched. Under Mix the first member, which is
  fixed on the left, has the lowest id of all, which agrees with the
  rule; a partition both of whose parts hold members of that first
  member's class still comes up twice, which costs time only. Measured (stable sequents, splits): the unsolvable 3-Partition
  with bins of four 1 834 321 and 13 015 869 before, 4 761 and 54 193
  after; the counter with 8 tokens 2 316 421 and 4 644 336 before,
  14 228 and 42 105 after, and with 16 tokens, which no run had
  finished, 473 232 and 2 004 517.
- **Focus candidates.** Every `⊗` and `⊕` of a stable sequent; `1` and `!`
  only when alone (they need an empty context; any, in affine mode); never
  a literal (a positive literal in focus succeeds only in the initial
  cases). Order: `1` and `!`, a `⊗` with a forced split, `⊕`, a `⊗` whose
  split is enumerated; ascending ids within a class; then the copies.
  This order is what makes the run deterministic, with the memo, which is
  only looked up, never iterated.
- **Forced splits.** A factor that is a positive literal takes exactly its
  dual from the context, and the first dual occurrence when there are
  several (they are the same formula, so the residues are equal
  multisets), or nothing when the context has none; a factor `1` or `!`
  takes the empty context; a factor `0` fails the candidate, not the
  sequent. `⊤`, `⊥` and negative literals force nothing: `⊢ ⊥ ⊗ b, a, ~a,
  ~b` needs `{a, ~a}` on the `⊥` side.
- **A factor that is a tensor of positive literals forces its side too**
  (`Forced::Duals`, `Counts::literal_tensor`): one dual per literal, each
  the first left in `Γ`, and the candidate fails when one is missing;
  the factor's proof is then built in place (`literal_tensor`: the
  axioms and the `⊗` nodes from the last occurrence back), with no focus
  on it, so a chain of such tensors costs no recursion either. When both
  factors force, the one closed in place goes first
  (`forced_factor`): `a ⊗ b ⊗ c` is nested to the left, and taking the
  tensor on the left as the forcing factor cost a focus per link (2 500
  tokens ran into the recursion limit; `limits` pins the chain).
  The argument: in focus the factor is decomposed by `⊗` rules down to
  its literals, each of which stays in focus and closes by an initial
  rule alone, that is on exactly its dual, from `Γ` or by a copy from
  `Θ`; the rule applies only to tensors none of whose literals has a dual
  directly under a `?` anywhere in the forest (the only way a literal
  gets into `Θ`), so every dual comes from `Γ` and the factor's side is
  one dual per literal and nothing else. Which occurrences is immaterial
  by the lemma on interchangeable occurrences. Without that proviso the
  rule would be the single literal's "dual from `Γ` first" applied per
  literal, which is complete but moves copies between branches, and
  would leave fewer proofs within a copy bound than the search of every
  split finds; so with a dual in `Θ` the split stays searched. Two-sided
  it needs no change, for the reason the single literal needs none: the
  premise it forces is the only classically provable one. Affine mode
  forces nothing, as before. This is what a Horn clause's body is when
  its atoms are positive, so `(b ⊗ t) ⊗ ~d` costs no split search at
  all. Measured (splits, the stable sequents unchanged): the unsolvable
  3-Partition with bins of five 36 072 before and 3 074 after, Partition
  with seven items 2 078 and 766, the unsolvable one with five 11 433
  and 2 106; of the sampled LLTP nets, `RwMutex_rwmutex-r2000w10_1_1` is
  proved in 1.1 s and `Diffusion2D_2D8_gradient_40x40_50_5_1` in 0.27 s
  where 4.9 s were needed before it.
- **`split_passes`** is the count test of a split as a function (the
  engine's `Rules::new` and a `Split` with every member placed), for the
  interactive state's helper. It is `Split::feasible`, the very test the
  engine's search ends on, so the two cannot drift apart.
- **Free splits are searched, not enumerated** (`search_splits`, for `⊗`
  and Mix alike, `Join` saying which). The statement: the splits whose
  premises are searched are exactly those whose two sides pass the counts
  (the interval check, and the equation where it is on), each once; a
  split that fails the counts is never visited, and no other is skipped.
  The members are assigned one at a time, in a fixed order, to the right
  first and then to the left (a depth-first search with an explicit
  trail, so a context of any width costs no recursion and no mask: there
  is no width limit, and `Reason::ContextTooWide` is gone). A partial
  assignment is cut when `Split::feasible` fails, which is a necessary
  condition for some completion to pass, per side and per atom on its
  own: with `lo`/`hi` the side's sums so far and `below`/`above` the sums
  of the negative parts of the open members' `lo` and of the positive
  parts of their `hi`, a side without an absorbing member needs
  `lo + below ≤ 0 ≤ hi + above` for every atom (any completion adds a
  subset of the open rows, and a subset's sum is bounded by those parts),
  unless an open member absorbs and can still join it, each open
  absorber serving one side (`needy ≤ open_absorbers`); the equation
  likewise with the slack `c − weight − 2` of a side and the open
  members' contributions `1 − weight`. With no member open the bounds
  are the sides' own sums, so the test at a leaf is the old `sides_pass`
  exactly; the argument for the cut is that it only removes subtrees
  whose every leaf fails that test. `Split::bad` counts the excluded
  atoms incrementally (`update` compares before and after, both sides,
  on the atoms of the member's row), so a step costs the member's row.
  The order (`Engine::open`): longer rows first (a compound member
  bears on several atoms; once the compounds are placed the literals of
  an atom are settled by its counts), then by the row's first atom, so
  that an atom's literals are neighbours, then descending id, so that
  the lowest ids change sides fastest, as they did in the Gray-code
  enumeration this replaces; members without a row (units, exponential
  atoms only) last, where only the equation can cut. The order changes
  which proof is found first and nothing else. `Statistics::splits`
  counts the steps of these searches (one feasibility test each) and the
  forced splits. Where no prune can cut (`Split::set_inert`, decided in
  `Engine::open`: the equation off and no member, placed or open, with a
  row entry that excludes zero by itself, which is every split under
  weakening and every Mix of formulas like `(a ⊗ b) ⊕ 0`), `feasible` is
  true whatever the assignment, so the counts are left alone and the
  members not even opened; the steps and their count are the same.
  `Tally::clear` and `Split::clear` zero the atoms their members touched
  (`touched`), not every atom of the sequent: a Petri net has thousands
  of atoms, none with a row, and cleared 24 KB per stable sequent.
  Measured on the first baseline's instances: Partition
  with six items 946 564 520 splits before and 3 337 after at the same
  94 stable sequents, the unsolvable one with four items 8 192 777 and
  6 845, QBF over 12 variables 9 389 062 and 37 980.
- **Mix** is tried last on a stable sequent, after the copies, with the
  first member fixed on the left so each partition comes up once, the
  trivial partition skipped (a leaf with an empty right side), the
  partitions searched by `search_splits` like the splits of a `⊗`, and
  each part decided by `prove` with the same `Θ` and budget, so the memo
  shares parts between partitions.
- **Hereditary failures** (`Failure::Hereditary`, `parts_fail`,
  `prove_part`): under Mix a stable sequent `P` fails *hereditarily*
  when no non-empty sub-multiset of its `Γ` is provable with its `Θ`.
  That holds when `P` has one member and fails, or when every rule but
  Mix failed on `P` and every `P \ {x}` fails hereditarily: every proper
  part lies in some `P \ {x}`, and a Mix of `P` is of two proper parts.
  So before searching partitions `mix` decides the parts with one member
  less (one per class of interchangeable or repeated members, since the
  others leave relatives) and, if each fails hereditarily, fails without
  a partition, `n` lookups for each of `2ⁿ` parts where the partitions of
  every part cost `3ⁿ`; the first part proved or not hereditary sends it
  to the partitions as before. The flag travels as a value
  (`prove_stable` returns it, `decide` takes it as an out-parameter that
  only `mix` and a one-member failure set), never in `Found`, so no
  other rule's failure can carry it. A hereditary failure is memoized as
  a complete one that says so, under the canonical key (a replacement
  maps parts to parts), and with the cuts of every part's failure; a
  failure of a part that a cut or a dependency qualifies is still
  hereditary for its caller under those cuts, and the cluster argument
  of the loop check carries over: of all provable parts of `P` take one
  with a smallest proof; it has no repeat of `P` or of a sequent between,
  else a part would have a smaller proof. Measured: `mix(7)` (14 members)
  114 675 stable sequents against 1 586 132, `mix(8)` 524 273 against
  14 316 140; the time hardly moves (1.39 s against 1.43 s at 8), since
  the focus on `~aᵢ ⊗ ~bᵢ` searches `2ᵐ` splits of the other members,
  which no count cuts under `⊕ 0`, and those splits are now the `3ⁿ`
  (34.6 million at 8 against 51.7 million before).
- **Recursion.** `prove`, `focus` and `asynchronous` count one level each;
  `Options::recursion_limit` stops the search with
  `Reason::RecursionLimit`. Two chains cost no level per link, since the
  LLTP Petri nets have them by the thousand: the `?` rules of an
  asynchronous phase are applied in `decompose`'s loop on a growing copy
  of `Θ` (a net's transitions are `!` hypotheses, and one level per `?`
  stopped every net with more than about two thousand of them before its
  first stable sequent, which is what 929 of the first baseline's 983
  recursion-limit rows were), and a chain of forced splits runs in
  `forced_splits`' loop on one context that every forcing factor takes
  its own from, the `⊗` nodes built afterwards from `links` in the order
  the recursion pushed them (a marking is a tensor of thousands of
  literals). A positive literal that forces a split is closed in place
  (`Ax`, or `Ax` under `Copy` from `Θ`, exactly what `initial` does on
  one or two members), and a dual literal is looked up through the
  forest's list of that literal's occurrences, in id order
  as before, not by a pass over the zone: in a chain by `dual_from`,
  which starts where the chain's last lookup of that literal ended
  (`Cursors`, one position per list, reset through the list of those
  that moved; a chain's context only loses members, so what a lookup
  passed over is gone for the rest of the chain), and elsewhere, for
  the unrestricted zone, by `dual_in` from the head. From the head
  every time, a chain over a marking of thousands of equal tokens was
  quadratic in them. A nested chain (below a `!` that a forced split
  promotes) has another context and takes cursors of its own from the
  pool. None of this changes a counter
  on the sequential targets; what changes is which sequents reach the
  limit. **A chain of free splits** (a `⊗` whose left factor is a `⊗`
  that no factor forces, and so on) runs in a loop too (`split::chain`,
  on the thread that searches it; the pool's first levels keep their
  tasks): each link's split search is a resumable `Walk` in a `Frame`,
  the left premise of a split it reaches is the next frame, and the
  result goes back to the frame below, which searches the right premise
  and resumes; `search_splits` is the same `next_split` loop with
  `premises` in place of the next frame, so the steps, nodes and
  counters are those of the recursion, and only the levels are not taken.
  Only a left factor of `CHAIN_SIZE` (256) occurrences or more starts a
  frame: the frames' bookkeeping, and the split search made resumable
  at all, cost Petri nets whose clause bodies are tensors of a few
  factors up to a fifth of their stable sequents per second, so a short
  chain recurses as before (with `next_split`'s place in locals and the
  search's parts given back one by one, which won most of it back;
  `TCPcondis_tcp15_20_1` still visits 15 % fewer in its 5 s, the price
  of the loop, against the wide nets' gains of the zone's range).
  `wide-m1` at 2 048 literals, which met the recursion limit, is proved
  in 0.35 s; at 4 096 the counts of a split per link (a frame keeps its
  own, as the recursion kept them on the stack) reach the memory bound.
  Measured stack per level, before the loop, on a chain of tensors whose
  splits are searched, which was the deepest set of frames (`focus`,
  `split`, `free_split`, `search_splits`, `premises`; the other cycles
  take three levels for some ten frames): 3.6 KiB in debug builds, 0.9 KiB in
  release, with the split's counts boxed in their pool (1.5 KiB with
  them in the frame, which overflowed the stack `Options::stack_size`
  gives at a raised limit; it now allows twice the measured). So the
  default of 2048 fits an 8 MiB main-thread stack, in a debug build only
  just; it stays, since of
  the 24 sampled problems that ended at the limit only two (ILLTP-SYJ
  problems whose search is that deep) still do.
- **No allocation per node once warm**: sets, contexts, keys, member lists,
  tallies, split counts, trails and the links of forced chains come from
  pools on the engine (`take_*`/`give_*`); a leaked buffer on an error
  path only costs an allocation later. A memo insertion copies the key
  into the memo's own chunk, and a repeated occurrence grows a
  context's extra list, which is the one allocation left per stable
  sequent. Two derived `clone_from`s used to allocate behind this
  sentence's back, which a heap profile showed (three million
  allocations in five seconds of an ILLTP problem): `OccSet`'s, which
  every copy of a zone goes through, and `Key`'s, on the branch stack;
  `OccSet` has its own now, and the stack copies with `Key::assign`.
  A derived `Clone` on a type that owns a buffer never reuses it.
  Every pool buffer is charged to the search's account when it is
  made, the lists when they are given back (`Pooled`), through the
  engine's `scratch`, which releases the charge when the engine goes:
  a pool's workers come and go by the thousand. The copies are ordered by an unstable
  sort on the id and one pass that asks `meets` once per formula: a
  stable `sort_by_key` allocated its buffer for every stable sequent
  with more than twenty copies and called `meets` at every comparison
  (58 % of the samples on the chain of 256 clauses). `meets` itself
  reads marks: `mark_literals` stamps, once per stable sequent, the
  lists of the literals among its members (`Engine::present`, one
  stamp per list of the forest, `Engine::stamp` the current one, a
  `u64` that cannot wrap in any run), and a formula of `Θ` meets a
  member when a literal below it has its dual's list stamped. Comparing
  every literal below every formula with every member was their
  product: a quarter of a second per stable sequent on
  `GPPP_G-PPP-1000-10_10_1` (a marking of thousands of tokens under
  clauses of thousands of literals), between two polls. The order of
  the copies and the work counted for a slice are what they were.
- **The memo can change decisiveness within the bound, never a verdict.**
  An `Exhausted` entry is a fact about the sequent alone, the loop check
  about the branch, so a run with the memo may answer `Unknown` where a
  memo-free run answers `Unprovable` (or the reverse); the generated tests
  assert only that the two never contradict.
- **The cuts are values a step returns** (`Found`: a proof, or a
  failure with its `Cuts`: `exhausted`, a branch cut by the copy budget,
  and `dependency`, the shallowest ancestor a loop-check prune below
  relied on). A failure carries the cuts of every failed step it rests
  on; a proof carries none, since it is a fact at every budget and on
  every branch, and a failure of a rule whose other premise was proved
  rests on its own premise alone (until step 26, one thread's proofs
  carried the cuts of the alternatives tried before them, as the
  engine-wide flags these values replaced did; dropping them made more
  failures complete and was a change of the search with its panel). Why
  that is sound (the panel's argument, sharper than the first one): a
  failed premise was searched under the same branch stack and the same
  budget as the rule's conclusion, so every proof through the rule
  contains a proof of that premise under the same conditions and its
  cuts are all the rule's failure rests on; what a proved sibling's
  search leaves for later is a memo entry, and an entry is a proof or a
  failure free of dependencies, the stack is popped back before the
  sibling runs, and the minimality argument discharges a dependency per
  rule instance. It changes the pool's workers below the parallel
  levels too, which run the same rules.
  `prove_stable` reads its decision's cuts to choose the memo entry
  (`Exhausted`, `Complete`, or none under a dependency), settles a
  dependency on itself and returns the rest; `run` reads a level's. A
  new rule adds the cuts of every failed step it runs to its failure
  (`Found::after`, `Cuts::and`): a cut dropped there is a wrong
  `Unprovable`, which the reference tests (`search::reference`) may
  catch and the counters do not. The pool merges them the same way
  (`core-parallel.md`).
- **Every proof passes the checker**: in `prove_goal` for a proof of the
  roots, in every build; `debug_assert!` in `search` besides, and
  every test that gets a proof calls `check`. The test-only generator
  `search/generate.rs` builds random provable sequents (and mutants of
  them) for every combination of units, additives, Mix and exponentials,
  and reports the most derelictions on one branch of the proof it read
  the sequent off, which bounds the copies the engine needs; a new rule
  set extends it rather than writing new positives by hand.
