# Step 26 report: the focused engine in order

The first of the step's two sessions (Opus 5.5 at `xhigh`). It built the
reference prover and had it checked before it judged anything, took the
profile and the measurements of item 6 before any change, and did items
1 to 3, the refactoring that claims no change of the search. The second
session starts from this report: items 4 to 6, the profile at the end,
item 7.

## Outcome of the first session

- **The reference prover is in the repository** (`core/src/search/reference.rs`,
  test-only): the plain unfocused calculus, one-sided classical and
  two-sided intuitionistic, linear and affine, with and without Mix,
  contraction bounded per branch. Every engine configuration agrees with
  it on 808 generated sequents and mutants (3 260 runs of an engine
  beside a decided reference), seven sequents that need contractions in
  every mode they have, and 240 additive pairs (no proof against a
  refutation, either way), and it agrees with every verdict the families know by
  construction at the sizes it decides. It was checked before it judged:
  a fresh-context review against the calculus found no wrong answer
  (sixty sequents decided by hand, and memo on and off, budget
  monotonicity and mode consistency on about 2 800 more), and eleven
  deliberate faults in a scratch copy each make the committed tests fail.
- **The cut and dependency flags are values** a step returns (`Cuts` in a
  `Found`), each rule of a choice is written once for one thread and the
  pool (`Alternative`, `Engine::alternative`), and every engine, a pool's
  workers included, is built by `Engine::new` from one `Problem` (item 1).
- **`focus/mod.rs` is cut along its seams** (item 2): 3 991 lines became
  `mod.rs` (the engine, its phases and its interface, 1 638 lines),
  `split.rs` (599), `arena.rs` (350), `scratch.rs` (241), `schedule.rs`
  (559: the two searches of the default bias, out of the pool's module)
  and `tests.rs` (1 390).
- **One engine interface** (item 3): every engine implements `Decide`
  (`admits`, with the error a forced engine answers, and `decide`, which
  returns an `Answer`); `prove_goal` is the one place an answer becomes a
  `Verdict`; the options each engine reads are on its `Engine` variant and
  on every setter; the roots of a forest in any order are the sequent.
- **No search changed**: the target set's counters at the split equal
  `bench/targets/after-bias.csv` on all 99 decided rows, and every verdict
  of the 164 rows, and so do they at the end of item 3
  (`after-values.csv`), at the CPU time of the split (1.005 times on
  the decided rows over 100 ms).
- **Measured before anything changes**: the profile of the 44 rows of
  the target set over a second, and every case item 6 names.

## The checklist

| item | state | evidence |
|---|---|---|
| reference prover and its test | done | `search::reference`, 5 tests, 1.6 to 2.6 s each in a debug build; commit "Add a reference prover for the tests, and compare every engine with it" |
| its review by a fresh-context agent | done, no wrong answer; one robustness finding fixed | below, "The reference prover" |
| the families' verdicts | done, at the sizes it decides | `agrees_with_the_families` |
| deliberate faults | done, 11 of 11 make the tests fail | below |
| the profile at the start (item 5) | done | below, "The profile at the start" |
| the measurements of item 6 | done | below, "Item 6, measured before any change" |
| item 1, the flags as values, rules once, the worker in one place | done, no change of the search | commit "Return the cut and dependency flags as values, …"; counters below |
| item 2, the file along its seams | done | commit "Split the focused engine's file along its seams"; `after-split.csv` |
| item 3, one engine interface, one verdict, options documented, goal in any order | done | commit "Decide every goal through one engine interface, …"; `goal_in_any_order` |
| the target set for every commit that claims no change | done: in full at the split and at the end of item 3; at item 1 alone the families and the slow tests (44 decided rows) | `after-split.csv`, `after-values.csv`, "The target set before and after" |
| the ILTP library and the LLTP library through the batch | done, no verdict against an earlier run or a status | below, "The library runs" |
| `nix flake check` | passed, at the end of item 3 | |
| items 4, 5 (the changes), 6 (the changes), 7; the profile at the end | the second session's | "For the second session" |

## The reference prover

**What it is.** `core/src/search/reference.rs`, compiled for the tests
only, beside the checker's `proofs/oracle.rs`. Its formulas are its own
(an interned table built from the generator's `Tree`s or from a
sequent's terms), so it shares no code with the engines. One-sided, it
tries on every distinct formula of a sorted multiset every rule of the
calculus: the axiom on an atom and its dual alone, `1` alone, `⊤` with
anything, `⊥`, `⅋`, both `⊕`, `&` with both premises, `⊗` over every split
of the rest, promotion with every other formula a `?`, dereliction,
weakening and contraction of a `?`, Mix over every split into two
non-empty parts, and in affine mode the weakening of any formula.
Two-sided, the rules of ILL over atoms, `1`, `⊤`, `0`, `⊗`, `⊸`, `&`, `⊕`
and `!`, with `⊸L` keeping the goal beside `B`, and affine weakening of
hypotheses only. Contraction is the one rule that makes a sequent larger,
so each branch may contract at most a budget of times; a branch that
wanted one more is cut, and a cut failure never becomes "unprovable". A
memo keyed by the sorted multiset (and the goal) records the least budget
a proof was found at, a failure without a cut (true at every budget) and
the largest budget a cut failure was found at. The top level deepens the
budget from zero with the memo kept, as the engines do, so the claim at
every smaller budget is made too.

**What its tests assert** (`configurations` lists what is judged: the
dispatch, the net engine and the additive path forced, the focused
engine one- and two-sided under each of the three biases, and with
`parallel` the dispatch on two threads; an engine that refuses a goal
is skipped): no engine's proof where the reference refutes and no
engine's refutation where it proves, on generated classical sequents
and mutants of every rule set, linear and affine (466 cases, 428 decided
by the reference), intuitionistic ones linear and affine (342, 324),
sequents that need one or two contractions in every mode
(`engines_agree_where_contractions_are_needed`), and 120 pairs of
additive formulas classically and intuitionistically, all decided. And
the families: the provable instances proved within their copies, the
unprovable ones without exponentials refuted, the others never proved,
at `3-partition-mll-yes` and `-no` at 4, `partition-yes` at 2, `qbf` at 3
and 4 (eight instances), the four `wide` families at small sizes, `mix` at
1 to 3, `counter` and `counter-over` at 2 and 4, `growing` at 1 and 2,
`chain` at 1 to 3 and `additive` at 1 to 3. Not in reach: `3-partition`
(the Horn encodings with `&`) and `partition-no`, whose smallest instances
did not finish within the visit bound (a minute and more in a debug build).

**The review** (a fresh-context agent on Opus 5.5, my model, reading the
reference against the calculus before anything was compared with it; it
ran nothing but its own scratch tests, capped, on two cores). No rule
applied outside its side conditions, the search complete at each budget,
"unprovable" sound, the memo's three facts true, the deepening right with
the memo shared across budgets. Sixty sequents decided by hand first and
then by the reference, all equal (among them `⊢ ?~a, a ⊗ a` unknown at
budget 0 and proved at 1, `⊢ 1, 1` unprovable and provable with Mix,
`⊢ ~c ⊕ ~a, b ⊕ (c & a)` provable, `a ⊸ b ⊢ b ⊸ a` and `!a ⊸ b, a ⊢ b`
unprovable, `⊤ ⊢ 1` unprovable and provable affine); on about 2 800 more,
the same answer with the memo on and off, answers monotone in the budget,
linear proofs never refuted under Mix or weakening, every ILL proof a
proof of the lowered one-sided sequent, and no engine against the
reference. Its findings, and what was done:

1. *The visit bound did not bound the work*: the splits of a context were
   built in a list before any was counted (`3ⁿ` under Mix; 4.6 s for
   fourteen atoms). Fixed: the splits are an iterator, each split counts
   against the bound (`VISITS`, a million per decision), and the
   deepening stops once the search gave up.
2. *It refutes nothing whose search can contract*: a failing branch with
   a `?` formula contracts down to an empty budget and is cut, at every
   budget, so with exponentials it answers "proved" or "unknown". By
   design (no loop check, which the engines have and which would be a
   shared assumption); documented in the module and in `core-search.md`.
   With exponentials it therefore judges an engine's refutations only,
   and an engine's proof is the checker's to judge.
3. Documentation: its budget counts contractions per branch, one less
   than an engine's copies of a formula, so the two bounds are never
   compared; the memo's cut entries written after the search gave up are
   true only in the sense that nothing but a proof is answered then.
   Both said in the module.

**The faults** (each made in a scratch copy of the reference, the
committed tests run against the engines as they are; all eleven fail the
tests, the last run on the final reference):

| fault | tests that fail |
|---|---|
| the `⅋` rule dropped (one-sided) | classical, families, contractions |
| `⊸L` dropped (two-sided) | intuitionistic, contractions |
| Mix dropped | classical |
| `1` with any context (one-sided) | classical |
| promotion's side condition flipped | classical |
| the copy bound off by one, the cut at the old boundary (one-sided) | classical, families, contractions |
| the copy bound off by one (two-sided) | contractions |
| no cut flag at the bound (one-sided) | classical, families, contractions |
| affine weakening dropped (two-sided) | intuitionistic |
| a cut failure memoized as a refutation | classical, families, contractions |
| the axiom without its side condition (any context) | classical |

The first run of the faults found three that passed: the off-by-one of
the two-sided bound, the missing cut flag and the memoized cut failure.
The reference was asked only at the generator's own budget, which always
suffices for its provable sequents, so its claims at a smaller budget were
never exercised. Two changes made them fail: the deepening from budget
zero (every level's claim is made and judged), and the sequents that need
exactly one or two contractions, which the generators make seldom (the
two-sided off-by-one shows only there).

**What the comparison cost to find**: nothing contradicted the reference
at any point; the engines agree with it on every case above.

## The profile at the start

Taken before any change on the step's first binary (release, debug
information through `CARGO_PROFILE_RELEASE_DEBUG=true`): `linlog-bench one
--problem P --mode given --engine auto --jobs 1`, the families with
`--timeout 30` (so `mix` at 10 and 11 are their first 30 s), the LLTP
nets with `--copies 3 --timeout 5` as the target set runs them
(`DatabaseWithMutex_database20UNFOLD_5_1` with `--recursion-limit 16384`),
one run per core on cores 10 and 11 (the slower kind; `mix/9` took 17.4 s
there and 14.6 s in `after-bias.csv`), each in a scope of 8 GiB, with
`perf record -e cpu_atom/cycles/u -F 999 --call-graph lbr` from the
flake's nixpkgs (perf 7.2.8; `-m 32` when two run at once, for the
per-user `perf_event_mlock_kb`). **Why LBR**: DWARF unwinding fails on
this binary in that perf (lld places the text segment at its file offset
plus a page, which perf's libdw unwinder takes as a wrong load bias: two
thirds of the samples failed to unwind, the rest after a frame or two);
LBR call stacks are exact but 32 calls deep, so inclusive shares of outer
frames are too low, and self time is exact either way. 274 CPU seconds
over the 44 rows of the target set over a second, 27 minutes in all.
The counters of `mix/8` and `mix/9` equal `after-bias.csv`.

**Ranking by self time** (median over the 44 rows, zero where absent;
functions in the top five of at least three rows):

| function (self) | rows in top 5 | median % | max % |
|---|--:|--:|--:|
| `Engine::split` | 32 | 15.4 | 47.1 |
| `Engine::prove` | 26 | 7.2 | 37.7 |
| `Engine::search_splits` | 25 | 6.7 | 29.1 |
| memmove | 25 | 6.3 | 30.9 |
| memset | 19 | 4.9 | 35.4 |
| `Engine::dual_from` | 19 | 4.4 | 41.4 |
| hashing (foldhash, `hash_one`) | 14 | 2.2 | 26.5 |
| `hash_bytes_long` alone | 12 | 0.5 | 23.9 |
| `Vec::extend` over `Context::iter` | 11 | 3.3 | 29.0 |
| `Engine::forced_side` | 9 | 2.9 | 13.0 |
| `Split::shift` | 6 | 0.4 | 25.3 |
| `Memo::find` | 5 | 1.6 | 20.0 |
| memcmp | 5 | 1.0 | 12.8 |
| `Split::excludes` | 5 | 0.0 | 10.5 |
| `Engine::focus` | 4 | 2.1 | 8.5 |
| `Vec::extend` in `decide_with` | 4 | 1.2 | 34.1 |
| `Engine::forced_factor` | 3 | 1.6 | 9.7 |

The allocator never reaches 0.05 % in any row; parsing stays under 0.3 %.

**Who calls the library code** (summed over the rows, % of each row's
samples):

| | caller | rows ≥ 0.5 % | summed % | max % |
|---|---|--:|--:|--:|
| memmove | `OccSet::clone_from` in `Context::clone_from`, from `Engine::split` | 31 | 217 | 29.3 |
| memmove | `OccSet::clone_from` in `free_split`, from `split` | 24 | 70 | 16.1 |
| memmove | `OccSet::clone_from` in `prove_stable` | 19 | 58 | 6.2 |
| memset | `OccSet::clear` in `take_context`, from `split` | 31 | 191 | 28.1 |
| memset | `OccSet::clear` in `free_split`, from `split` | 27 | 143 | 30.2 |
| memcmp | `Memo::matches` in `Memo::find` | 25 | 130 | 12.8 |
| hashing | `OccSet::hash` in `Memo::get` | 23 | 103 | 12.6 |
| hashing | `OccSet::hash` in `prove_stable` (the loop check's hash) | 16 | 90 | 10.1 |
| hashing | `OccSet::hash` in `Memo::refuted` | 15 | 51 | 6.0 |

Inside `Engine::split` the self time is `give_cursors` inlined (up to
15 %) and `dual_in` and `literal_tensor` under `forced_splits` (up to
25 %); inside `prove`, `Context::slot` (up to 14.5 %) and the bitset's
iterator.

**Per row** (CPU seconds profiled, verdict, the top five self entries):

| row | s | verdict | top five (%) |
|---|--:|---|---|
| mix/8 | 1.9 | unprovable | `Split::shift` 18.4, `prove` 16.7, `Memo::find` 15.7, `search_splits` 12.8, `Split::excludes` 9.9 |
| mix/9 | 17.4 | unprovable | `Split::shift` 18.9, `prove` 17.1, `Memo::find` 16.6, `search_splits` 13.2, `Split::excludes` 9.0 |
| mix/10 | 29.9 | time | `Memo::find` 19.5, `Split::shift` 17.2, `prove` 16.9, `search_splits` 12.4, `Split::excludes` 10.5 |
| mix/11 | 30.0 | time | `Memo::find` 20.0, `Split::shift` 17.2, `prove` 15.9, `search_splits` 12.7, `Split::excludes` 10.4 |
| AirplaneLD-pt-0010_20_1 | 5.0 | time | `split` 47.1, `dual_from` 13.9, `forced_side` 10.8, `search_splits` 10.8, `forced_factor` 5.0 |
| DLCround_dlcro_03_b_50_1 | 5.0 | time | `hash_bytes_long` 23.9, memcmp 12.8, memmove 10.6, `prove` 9.6, memset 6.8 |
| Eratosthenes_eratosthenes-050_5_1 | 5.0 | time | `Split::shift` 25.3, `search_splits` 14.8, `split` 12.9, `Split::excludes` 8.8, `Split::flip` 5.3 |
| NeighborGrid_z_4d_3n_2m_c_2_3_20_1 | 5.0 | time | `split` 25.2, `dual_from` 14.1, `search_splits` 9.5, memmove 7.5, memset 7.2 |
| Peterson-4_50_1 | 5.0 | time | `split` 22.2, `dual_from` 10.6, `hash_bytes_long` 10.0, `prove` 9.7, memmove 7.4 |
| QuasiCertifProtocol_QCertifProtocol_10-unfold_20_1 | 5.0 | time | `split` 30.3, `search_splits` 10.5, `dual_from` 9.3, `forced_side` 7.4, extend over `Context::iter` 7.0 |
| TCPcondis_tcp15_20_1 | 5.0 | time | `split` 24.9, `dual_from` 11.7, `search_splits` 11.6, `forced_side` 8.0, `prove` 6.9 |
| TokenRing-10-unfolded_100_1 | 5.0 | time | `split` 16.3, `dual_from` 15.4, `hash_bytes_long` 12.4, `prove` 11.9, memmove 10.4 |
| DES_des_01_b_10_1 | 5.0 | time | `hash_bytes_medium` 21.6, `prove` 13.7, `focus` 8.5, `search_splits` 8.3, `split` 6.6 |
| DES_des_50_b_20_1 | 5.0 | time | `hash_bytes_long` 21.7, `prove` 14.3, memmove 7.8, `search_splits` 7.2, `focus` 6.7 |
| CloudDeployment_deploy_5_b_100_1 | 5.0 | time | memset 24.2, memmove 22.4, `hash_bytes_long` 15.8, extend over `Context::iter` 9.9, memcmp 7.4 |
| CloudReconfiguration_reconf_3_04_20_1 | 5.0 | time | `hash_bytes_long` 15.4, memmove 12.7, `split` 11.4, `prove` 10.0, memset 9.4 |
| CloudReconfiguration_reconf_3_10_100_1 | 5.0 | time | `hash_bytes_long` 17.5, memmove 13.8, `split` 11.6, memset 9.4, `prove` 8.9 |
| CloudReconfiguration_reconf_3_15_20_1 | 5.0 | time | `hash_bytes_long` 16.6, memmove 13.9, `split` 10.6, `prove` 10.5, memset 9.7 |
| CloudReconfiguration_reconf_4_01_100_1 | 4.9 | time | memmove 22.5, memset 21.3, `hash_bytes_long` 14.8, memcmp 8.2, `split` 7.7 |
| DLCround_dlcro_07_b_5_1 | 4.9 | time | memmove 19.5, memset 17.9, `hash_bytes_long` 16.0, memcmp 11.6, `prove` 7.7 |
| DLCshifumi_dlcsh_4_a_100_1 | 4.9 | time | extend in `decide_with` 34.1, sort in `one_of_each` 19.5, dedup in `one_of_each` 17.4, memmove 7.0, `hash_bytes_long` 5.3 |
| Diffusion2D_2D8_gradient_20x20_100_5_1 | 1.7 | proved | `dual_from` 29.7, `split` 20.6, `search_splits` 17.3, `Context::insert` 5.8, memmove 5.5 |
| FlexibleBarrier_flexbar_18_b_5_1 | 5.0 | time | memmove 19.2, memset 18.8, `hash_bytes_long` 13.3, memcmp 11.5, `prove` 6.8 |
| HypercubeGrid_hc3k4p4b12_5_1 | 5.0 | time | `prove` 26.0, memset 19.4, memmove 16.6, extend over `Context::iter` 10.0, `split` 6.5 |
| NeoElection_neoelection-7.unf_10_1 | 5.0 | time | memset 35.4, extend over `Context::iter` 29.0, memmove 23.1, `split` 2.8, extend in `decide_with` 1.2 |
| Angiogenesis_angiogenesis-15_20_1 | 5.0 | time | `split` 33.8, `search_splits` 11.7, `prove` 8.1, `dual_from` 7.0, `forced_side` 5.2 |
| AutoFlight_afcs_48_a_50_1 | 5.0 | time | `split` 19.1, extend in `decide_with` 17.3, memmove 9.3, sort in `one_of_each` 8.0, dedup in `one_of_each` 6.8 |
| ClientsAndServers-0010-1_100_1 | 4.9 | time | `split` 27.7, `search_splits` 10.5, `prove` 8.6, `dual_from` 7.8, `forced_side` 7.1 |
| ClientsAndServers-0020-3_50_1 | 5.0 | time | `split` 24.8, `prove` 12.3, `dual_from` 9.0, `search_splits` 8.3, extend over `Context::iter` 7.4 |
| Echo_echo-d2r9_100_1 | 5.0 | time | `split` 26.4, `dual_from` 13.4, memmove 10.7, memset 7.9, `forced_side` 7.5 |
| GPPP_G-PPP-100-1000_100_1 | 5.0 | time | `prove` 37.7, `search_splits` 29.1, `Context::insert` 11.3, extend over `Context::iter` 7.7, memmove 3.6 |
| JoinFreeModules_joinFree-10_50_1 | 5.0 | time | `prove` 32.6, `hash_bytes_medium` 15.5, `Memo::find` 10.6, `hash_one` 5.9, `search_splits` 4.9 |
| Kanban-500_50_1 | 5.0 | time | `split` 41.3, `prove` 16.2, extend over `Context::iter` 9.6, `dual_from` 7.2, memset 6.0 |
| PermAdmissibility_unf-8x8-4stageSEN-50_10_1 | 5.0 | time | `split` 17.1, extend over `Context::iter` 17.1, memset 12.3, memmove 9.5, `prove` 7.2 |
| QuasiCertifProtocol_QCertifProtocol_32-unfold_20_1 | 5.0 | time | memset 30.2, extend over `Context::iter` 25.2, `split` 14.6, memmove 7.2, `search_splits` 5.0 |
| ResAllocation_RAS-C-50_100_1 | 5.0 | time | `split` 17.6, `search_splits` 8.3, extend over `Context::iter` 7.6, `prove` 7.3, `focus` 5.8 |
| Solitaire_soli2_counter_20_1 | 5.0 | time | `split` 25.7, `search_splits` 10.8, `forced_side` 7.9, `dual_from` 7.5, extend over `Context::iter` 6.0 |
| DES_des_00_a_20_1 | 3.9 | proved | `split` 32.1, `dual_from` 13.9, `search_splits` 10.9, `forced_side` 9.3, `forced_factor` 6.3 |
| Parking_parking_2_8_5_1 | 5.0 | time | `split` 34.5, `forced_side` 13.0, `search_splits` 10.9, `dual_from` 10.1, `forced_factor` 9.7 |
| PermAdmissibility_unf-8x8-4stageSEN-05_100_1 | 4.9 | time | `split` 25.7, `dual_from` 14.9, memmove 9.6, memset 7.9, `search_splits` 7.6 |
| PhaseVariation_5-10_phaseVariation_50_1 | 5.0 | time | `dual_from` 41.4, `split` 40.0, memset 3.1, memmove 3.1, `search_splits` 2.3 |
| SimpleLoadBal_simple_lbs-15_50_1 | 4.9 | time | memmove 26.7, memset 25.6, `split` 16.5, `dual_from` 10.1, `prove` 6.0 |
| TCPcondis_tcp30_50_1 | 5.0 | time | `split` 19.5, `prove` 9.9, `search_splits` 9.3, `focus` 7.0, `Split::shift` 5.8 |
| DatabaseWithMutex_database20UNFOLD_5_1 | 5.0 | time | memmove 30.9, memset 28.7, `split` 9.0, extend in `decide_with` 6.2, `dual_from` 4.3 |

**What it says for item 5.** Step 15's three hot spots have moved: the
memo key's hashing is still there (`hash_bytes_long` in the top five of
twelve rows, up to 24 %, and hashed three times per stable sequent: the
loop check, `Memo::get`, `Memo::refuted`), but the insert's allocation is
gone (the allocator never shows), and the canonical key does not appear
by name. The largest single cost on the large nets is now **whole-width
bitset copies and clears in the `⊗` rule** (`take_context`'s clear
followed by `clone_from`, in `split` for the forced chain's context and
in `free_split` for the two sides): together 44 to 60 % of the five
widest nets (`DatabaseWithMutex` 46 672 occurrences, `NeoElection`,
`SimpleLoadBal`, `CloudDeployment`, `reconf_4_01`), and present in most
rows. Then the forced chain's dual lookups (`dual_from`, `dual_in`,
`give_cursors`), iterating a whole-width `Context` into a member list
(`Context::iter`), and on three nets `decide_with`'s filter of `Θ` and
`one_of_each`'s sort (71 % of `DLCshifumi`). Mix spends its time where
step 15 found it (the split counts and the memo).

## Item 6, measured before any change

All on the step's first binary, cores 12 to 15 in a capped scope (the
eight-core runs on 8 to 15), by a sub-agent that changed nothing.

- **The pool against one thread at recursion limits 4 to 16**: 18 180
  sequent-mode pairs (generated classical sequents of every rule set and
  intuitionistic ones linear and affine, with mutants), 54 540 searches.
  No contradiction, no `Unknown(Stopped)` without a stop. One thread ends
  at the limit on 45 % of them at 4 and 8 % at 16. On two threads: 130
  that one thread left at the limit are decided (124 proved, 6 refuted),
  20 proved by one thread end at the limit, 16 refuted by one thread end
  at the copy bound. On four: 319 decided that one thread left at the
  limit, 18 proved at the limit, 18 refuted at the copy bound, one at
  the copy bound refuted, and the case the step names twice, one thread
  "unprovable" where the pool says "recursion limit":
  `|- !?~c, (?(a * c) par ?!~c)` under Mix, copies 3, limit 9 (one thread
  refutes after 10 stable sequents, the pool stops at the limit after
  20), and `b, (a -o a), a, a, b, ((a * b) -o a) |- ((a * a) * a)` affine,
  copies 0, limit 13 (400 against 106). One thread refuting where a pool
  stops at the copy bound: `!b |- !a` affine, copies 1, limits 8 and 9.
- **`SYJ204+1.014` in `01`**, no copy bound, 60 s: one thread proves it in
  1.98 s after 15 708 917 stable sequents (level 9); a pool of two proves
  it every time (three runs) in 5.3 to 5.4 s after 70.4 to 71.5 million.
  Before it was seen at 66 million without a proof; now 4.5 times the
  work to a proof.
- **`LCL181+1`**: in `01`, one thread deepens to 3 233 560 levels in 5 s
  (38.8 million stable sequents) without deciding; pools of two refute it
  in 3.5 to 83 ms at levels 389 to 10 987, of four in 0.6 to 1.5 ms at
  levels 10 to 67. In `cbn` and `cbv` one thread refutes it at once
  (0.12 ms, levels 3 and 5).
- **`SYN393+1` in `cbn`** (`ILLTP-SYN-cbn`): one thread, 10 s, 2 723 076
  levels, 77.6 million stable sequents, undecided; a pool of two from the
  start, five times undecided at 3.9 million levels; a pool of four, five
  times refuted in 0.8 to 4.3 ms at levels 15 to 435; the default race on
  four cores five times undecided at 2 s (about 750 000 levels: one
  thread beside a pool of three); on eight cores five times refuted in
  0.10 to 0.19 s. As the step had it.
- **`SYJ212+1.014` in `cbn`**, one thread, 10 s: the default without a
  copy bound undecided at level 5 (340 355 stable sequents); with
  `--copies 3`, at its copy bound in 1.22 s; `--bias rarer` undecided at
  level 4; `--bias factors` alone **refutes it in 1.37 s** at level 5.
  The backward search's share keeps the default from what the forward
  one does alone in a seventh of the time.
- **A worker's copy of the branch** (pool of four, `--copies 3`, 5 s,
  LBR call stacks): under `Spawn::worker` 20.0 % of the samples on
  `CloudReconfiguration_reconf_3_04_20_1` (re-hashing the keys 8.6 %,
  malloc and free 5.2 %, the copy 4.5 %; of all the run's
  `hash_bytes_long`, 55 % is the worker's), 9.5 % on
  `GlobalResAllocation_galloc_res-5_100_1`, none on `DES_des_00_a_20_1`.
  Item 1 removed the re-hashing (the hashes are copied with the keys);
  the copy itself is left for the second session.
- **The affine initial rule**: `a0, …, an |- an` affine, one thread, a
  limit of 2 s: 32.6 ms at 10 000 atoms, 283 ms at 30 000, 3.11 s at
  100 000, provable, 1.1 s past the limit (quadratic, unpolled).
- **The late cancellation at a `&`** (a stolen task of the sibling
  premise running uncancelled until the failed premise's worker has left
  its scopes) was not reproduced; the step says it was seen once.

## Items 1 to 3: what changed

**Item 1.** A step of the engine returns a `Found`: the node proving its
sequent, if any, and its `Cuts` (`exhausted`, a branch cut by the copy
budget; `dependency`, the shallowest ancestor a loop-check prune below
relied on). The invariant, stated where the type is: a step returns the
cuts of every step it ran, whatever they found. That is exactly what the
engine-wide flags held between the save and the restore in
`prove_stable`, which is why the counters could not move and did not:
`prove_stable` now reads its decision's cuts to choose its memo entry,
settles a dependency on itself, and returns the rest; `run` reads a
level's. The early return between save and restore that the assessment
named as a trap cannot exist any more. A choice's alternatives (a focus
on a member of `Γ`, a copy, a side of a `⊕`, the splits under a pattern)
are one type, `Alternative`, with one implementation,
`Engine::alternative`; one thread runs them in order (`choose_here`),
a pool as tasks (`choose_parallel`), `choose` picks. The `&` rule's
premise (`Engine::premise`) and its node (`Engine::both`) serve both.
`Engine::new` is the one constructor: it takes a `Problem` (forest,
reading, counts, classes, rules, account, limits), and a pool's worker
is that engine given the branch it continues, with the hashes of the
branch's keys copied instead of hashed again. The pools of buffers are
one value (`Pools`).

**What one thread and the pool still do differently** is the cuts of a
proof: one thread's proof carries those of the alternatives it tried
before it, the pool's carries none of its alternatives', and at a `&`
whose left premise was proved and right one failed, one thread returns
both premises' cuts and the pool the failed one's. Both are sound (a
proof makes its failed siblings' cuts irrelevant to any failure above
it). I kept each as it was, so that this session claims no change of any
search; making one thread drop them too is a change of its search (its
memo entries, its decisiveness at the bound, its counters) and goes to
the second session with a panel. It would make the two the same rule,
and should make one thread somewhat more decisive at the copy bound.

**Item 2.** Pure moves, with the visibility each needed: `arena.rs`
(the arena and its tests), `split.rs` (the `⊗` rule and Mix),
`scratch.rs` (the pools), `schedule.rs` (`plan`, `chains`, `Rule`,
`turns`, and from the pool's module `alternate`, its baton and
`merged`), `tests.rs`. Its target set run (`after-split.csv`) is the
evidence that it moved nothing.

**Item 3.** `search::Decide` is the interface (crate-private, since the
engines are): `admits(&Task)` refuses a goal with the error a forced
engine answers today, `decide(&Task, &Options, &Account, stop)` returns
an `Answer` (the proof, `None` for an exhausted search, or the reason,
with the counters and the net). `Engine::implementation` maps the public
variants to `focus::ONE_SIDED`, `focus::TWO_SIDED`, `net::Nets` and
`additive::Additive`; `dispatch` is today's choice, as a function of the
`Task`, which item 4 turns into a table. `prove_goal` builds every
`Verdict`: the four other places that did (`focus::search`,
`additive::search`, both net searches) are gone or return an `Answer`.
Options an engine does not read are documented, not refused (the
dispatch may pick an engine the caller did not name, and a front end
sets every option at once): on each `Engine` variant which it reads, and
on each setter which engines read it. The roots in any order are the
sequent: `is_roots` compares them as a multiset, the engines get the
forest's order (so a search does not depend on the order a caller gives),
and `goal_in_any_order` pins the net engine taking them, forced and by
default.

**Quantifiers (D17).** The interface keeps their place: a first-order
goal is a `Task` whose forest carries terms; what changes is inside an
engine (a trail of bindings beside the branch, the dual of a literal a
unifiable one), not `Decide`, `Answer` or the verdict. The second
session's item 7 says where the trail goes and which prunes assume ground
atoms.

## The target set before and after

| label | state | decided rows | counters against `after-bias.csv` | verdicts of all 164 rows |
|---|---|--:|---|---|
| `after-split` | the split | 99 | identical on all 99 | identical |
| `after-values` | items 1 and 3 | 99 | identical on all 99 | identical |

Beside them, at item 1 alone, the families and the slow tests of the set
(44 decided rows) were run on a separate core with identical counters.

**What the refactoring costs** (D17 asks for the pinned CPU time within
a few percent): against `after-split.csv`, taken the same day on the
same two cores, the 13 decided rows over 100 ms take 1.005 times the CPU
time (geometric mean; the clock's tick is 10 ms; the largest row,
`mix/10`, 121.7 s and 119.4 s), and the 37 rows that end at their time
limit visit 0.996 times as many stable sequents within it. The split's
own times are 0.53 of `after-bias.csv`'s, 0.87 of `after-limits.csv`'s
and 1.02 of `after-memory.csv`'s and `after-defaults.csv`'s (the same
geometric mean): what made them faster was steps 19 and 20, and nothing
moved from step 21 to this one.

## The library runs

On the release build of the end of item 3, every run by the command's
batch, detached, in a scope of 8 GiB.

**ILTP**, as step 25 ran it (`linlog prove --logic intuitionistic
--translation T` or `--logic classical`, `--file bench/iltp/Problems
--workers 2 --cores across --stats --output DIR`, the default 2 s, here
on cores 12 and 13; every proof read back, checked and written), beside
step 25's table:

| run | valid | not valid | unknown | error | step 25 | against ILTP's status |
|---|--:|--:|--:|--:|---|--:|
| intuitionistic, cbn | 68 | 28 | 177 | 1 | 68, 28, 177, 1 | 0 |
| intuitionistic, cbv | 70 | 16 | 187 | 1 | 72, 16, 185, 1 (its review 71) | 0 |
| intuitionistic, 01 | 56 | 12 | 205 | 1 | 56, 12, 205, 1 | 0 |
| classical | 155 | 1 | 116 | 2 | 155, 1, 116, 2 | — |

Every proved Theorem and refuted Non-Theorem agrees with ILTP's
intuitionistic status, no read-back failed, and the errors are step
25's (`SYN007+1.014` over the occurrence limit; classically also the
checker's memory bound on `SYJ211+1.018`). The two `cbv` Theorems fewer
are the time limit on these cores, the slowest kind (step 25 ran on 4
and 5): `SYJ205+1.014` alone is proved in 1.69 s on core 4 and not
within 2 s on core 12; and the run shared its cores with `nix flake
check`, whose builds the daemon does not pin. No search changed, as the
target set shows.

**The LLTP library through the batch by default on four cores** (`linlog
prove --file bench/lltp/ILL`, then `CLL`, on cores 8 to 11), beside
step 24's review (4 512 answers, 2 193 proved, 142 refuted): 4 512
answers, none twice, no error; **2 150 proved and 142 refuted**; no
verdict against the second baseline's classical pass
(`bench/results/2026-10-02/lltp-classical.csv`, every problem compared).
The refutations are step 24's in number; the 43 proofs fewer are the
time limit on these cores (the slower kind; the review's cores are not
recorded): of the problems left unknown here, the second baseline's
one thread proves 32, none of them in under a second (4 in 1 to 2 s, 28
in 2 to 5 s). The first run of this, beside the flake check's unpinned
builds, gave 2 148 and 142; this one ran alone.

## Decisions

- **The reference's budget counts contractions**, not dereliction-copies
  as the engines do, because contraction is the rule that makes the
  plain calculus infinite; the two bounds are never compared, only
  verdicts. It has no loop check, so it refutes nothing whose search can
  contract: a loop check is an assumption the engines share, and the
  reference is worth most where it assumes least.
- **The deepening in the reference** came from the faults: asked only at
  the generator's budget, which always suffices, its claims at smaller
  budgets were never exercised. Answering at every level from zero is
  how the engines answer too.
- **The merge of cuts on a proof stays as it was on each side** (above):
  the session claims no change of any search, and the unification is a
  change with a panel.
- **Options an engine does not read are documented, not refused.**
- **The roots in any order are handed to the engines in the forest's
  order**, so a search's counters do not depend on how a caller orders
  them; a goal that is not the roots is handed over as given.
- **The worker copies the hashes** of the branch's keys with the keys:
  same values, so no search changes, and the re-hashing was the larger
  half of `Spawn::worker`'s 20 % on a net.

## Deviations and assumptions

- The profile ran on cores 10 and 11, the slower kind, one row per core;
  the second session's profile at the end should run on the same cores
  with the same command so that the two compare.
- The reference cannot reach the smallest `3-partition` and `partition-no`
  instances, so the families' check covers the others.
- Commit signing timed out once (gpg's passphrase was no longer cached);
  the work stayed on disk and was committed when signing worked again,
  each commit's state rebuilt and verified on its own.

## For the second session

In the order I would take them:

1. **Item 4, the dispatch as data**: `search::dispatch` is the function
   to turn into a table of rows (fragment, mode, feature, engine, the
   measurement), with the atom bias out of the forest (`Forest::bias`,
   `bias_under`, `signs`) into the focused engine; the ILTP images per
   translation are a workload for its measurements.
2. **Item 5's hot spots from the profile above**: the whole-width copies
   and clears of `split` and `free_split` (`take_context` clears what
   `clone_from` overwrites; the forced chain copies the whole context),
   the key hashed three times per stable sequent, `Context::iter` into
   member lists, and `decide_with`'s filter of `Θ` with `one_of_each`.
   Callgrind (through the `new-tool` skill) for the small differences.
3. **Item 6's changes**, each with its panel: the cuts of a proof on one
   thread (above), the Mix prune, the free-split loop, the Horn test on
   the goal (`chains` reads `forest.roots()`), the cancellation at a `&`
   raised where the failure is known, an error of one premise not
   cancelling the other (the two witnesses above), the copy of the
   branch per worker, the affine initial rule (quadratic, unpolled), and
   the measurements of `LCL181+1`, `SYN393+1`, `SYJ212+1.014` and
   `SYJ204+1.014` above as the before.
4. The profile at the end, as above; item 7.

The second reference the step asks for (written by a fresh-context agent
from the calculus, kept outside the repository, run by every panel's
counterexample agent) belongs to the session that changes the search.

## The commits

1. Add a reference prover for the tests, and compare every engine with it
2. Split the focused engine's file along its seams
3. Return the cut and dependency flags as values, and write each rule
   once for one thread and the pool
4. Decide every goal through one engine interface, and take a goal's
   roots in any order
5. Record the target set after the engine interface
6. Report step 26, first session (this report, the plan's Status entry,
   and how a CPU profile is taken, in `.claude/rules/bench.md`)

Nothing is pushed.

## From the review of the first session

Accepted without a fix. The commits were rebased onto "Plan: certified
refutations, the cheap ones in step 31 and the rest as later goals",
made while the session ran, without a conflict.

- **Checked**: clippy, the tests (and `cargo test -p linlog`), both
  `cargo hack` runs, `cargo deny`, `nix flake check`.
- **The target set again**, by the review on the reviewed code
  (`bench/targets.sh` on cores 2 and 3): every verdict of every row and
  the counters of every decided row equal `after-bias.csv`, and the CPU
  time is 1.009 times `after-values.csv`'s on the decided rows over
  100 ms.
- **Ten more faults** in a scratch copy of the reference, chosen by the
  review and none of the session's eleven, each make the committed tests
  fail. Five make it prove less (a `⊕` disjunct and a `&` component
  dropped, `0L` and `1L` removed, `⊤` only alone); `!R` without its side
  condition and an additive `⊗` make it prove more; the split with an
  empty left part skipped and `⊸R` without its antecedent make it prove
  less again. The review read the reference against the calculus too,
  but it is the session's model, so the faults are the weightier
  evidence.
- **The pool after item 1**, which the counters of one thread do not
  see: the 57 problems only the pool decided at step 24's review, the
  ILTP images and `CLL`, with `--cores within` (one thread raced against
  a pool, on four cores). No verdict against ILTP's statuses or the
  one-thread runs, and the pool decided 14 problems more than one thread
  per problem. Six Petri nets of the 57 were unknown at 2 s on the
  slowest cores; on the same cores, three rounds each, the binary before
  the step decided them in 14 of 18 runs and this one in 17 of 18, so it
  is the time limit, not the rewrite.
- **For the second session**: the comparison of the pool with one thread
  at recursion limits 4 to 16 ran before item 1 rewrote how a pool runs
  a choice. Repeat it on the code as it stands before the first change of
  the search, so that a later difference is the change's.

## Second session: items 4 to 7

The second session (Opus 5.5 at `xhigh`) started from the first part
and its review. It repeated the pool's comparison with one thread at
recursion limits 4 to 16 on the code as it stood, turned the dispatch
into a table with its measurements, took the hot spots of the profile
without changing a counter, made the changes of the search item 6 names,
each measured and reviewed by a panel, took the profile at the end, and
wrote item 7's note. On the author's decision it also took an external
reviewer's finding (Mix in affine mode).

### Outcome of the second session

- **The dispatch is a table** (`search::DISPATCH`: rows of a fragment,
  the modes, a feature and the engine, read from the first down), and
  `Engine`'s documentation tabulates each row with the measurement
  behind it. The atom bias is the focused engine's (`search::Bias`,
  `focus/bias.rs`); the forest has none. No row moved: the
  measurements confirm the additive row (14 to 60 times faster than the
  focused engine from depth 12 of the identity), and show the net rows
  kept for depth, not speed (below).
- **The hot spots, at identical counters**: a stable sequent hashed
  once and its zones read in place by the memo; no clear before a copy;
  `Θ`'s members by class kept along a branch; the affine initial rule in
  one pass (100 000 atoms in 73 ms instead of 3.03 s); a pool's worker
  reading its branch in place; and the linear zone's range of words, so
  that a split's setup costs the members and not the forest's width
  (NeoElection 9.7 times the stable sequents in its 5 s). The target set
  at the end of them: every counter of the 99 decided rows and every
  verdict as `after-values.csv`, at 0.93 times its CPU time.
- **The changes of the search** (item 6), each a commit that says so,
  measured, and reviewed by a panel of three: a proof carries no cuts on
  one thread either; on a pool only a failure of a `&` premise cancels
  the other; the Horn test reads the goal; under Mix a sequent whose
  parts with one member less all fail hereditarily fails without its
  partitions (`n·2ⁿ` stable sequents for `3ⁿ`); and a chain of free
  splits runs in a loop (`wide-m1` with 2 048 literals proved by the
  focused engine in 0.35 s, where it met the recursion limit); and, on
  an external reviewer's finding and the author's decision, Mix is left
  out in affine mode, where weakening makes it admissible (3 decisions
  lost at a copy bound against 1 638 gained in a panel's 696 024 runs;
  kept on the author's decision). **No panel found a wrong verdict.**
  Four found something else, each reproduced,
  fixed and pinned by a test: the `&` change's cost on nested premises
  that give up (`2ᵏ` leaves), a goal a dereliction leaves that the Horn
  test lost, two decisions the Mix prune lost (a part's cut qualifying
  the whole, and the memo off), and the stack a level of recursion
  takes since the split search is inlined.
- **At the end**, against the start of the step on the target set:
  every decided row's counters equal but the three Mix rows (`mix/10`
  10.5 million stable sequents for 1.16 billion), no verdict changed,
  0.84 times the CPU time of the decided rows over 100 ms, 1.20 times
  the stable sequents of the rows at their time limit (NeoElection 11.9
  times, CloudDeployment 2.3, TCPcondis 0.6 to 0.75 by these runs, −15 %
  paired). The default now refutes `SYJ212+1.014` in its cbn
  translation in 2.7 s, where it did not within 10 s.
- **Two references judge the engines**: the committed one and a second,
  written by a fresh agent from the calculus alone and kept outside the
  repository (`reference2`), which every panel's counterexample reviewer
  ran.

### The checklist, second session

| item | state | evidence |
|---|---|---|
| the pool against one thread at limits 4 to 16, before any change | done: no contradiction, no unasked stop, the picture of the first part | "Outcome" below; repeated after the `&` changes |
| item 4, the dispatch as data, the bias out of the forest | done, no row moved | `search::DISPATCH`, `Engine`'s docs; "Item 4" |
| item 5, the hot spots, at identical counters | done: six changes | `hot-spots.csv`; callgrind; "Item 5" |
| item 6, the Mix prune | done, panel; two findings fixed | "Under Mix, …" |
| item 6, a loop for chains of free splits | done, panel | "A chain of free splits …" |
| item 6, the Horn test on the goal's members | done, panel; one finding fixed | "The Horn test reads the goal" |
| item 6, the `&` on the pool: an error cancels the other | done, panel; one finding fixed | "On a pool, …" |
| item 6, the late cancellation at a `&` | not fixed, reasoned | "Decisions" |
| item 6, the differential run at limits 4 to 16 | done before and after | above |
| item 6, a worker's copy of the branch | done (read in place) | "Item 5" |
| item 6, `SYJ204`, `LCL181`, `SYN393`, `SYJ212` | measured before and after | "Item 6's measurements, after" |
| the cuts of a proof on one thread (left by the first part) | done, panel | "A proof carries no cuts" |
| a second reference outside the repository | done, run by every panel | "The second reference" |
| the profile at the end, beside the start | done | "The profile at the end" |
| item 7, the note for quantifiers | done | "Item 7" |
| the target set for every commit that claims no change | done in stages | "The target set before and after" |
| the ILTP library and the LLTP library through the batch | done, no contradiction | "The library runs" |
| `nix flake check` | passed, at the end | |
| an external reviewer's finding: Mix in affine mode | done on the author's decision; panel: no wrong verdict, 3 lost and 1 638 gained decisions, kept by the author | "Mix is left out in affine mode" |

### Item 4: the dispatch as data

`search::DISPATCH` is four rows, read from the first down; each names
the largest fragment it takes, its modes (`Modes`), the feature
(`Feature`) and the engine. `dispatch` is the first row that takes a
goal; the last two take every goal, by mode. The rows are today's
choice, unchanged (the dispatch tests pass as they were), and each now
has its measurement, in the table on `Engine`'s documentation (the
rustdoc is the manual) and here. Step 27's Horn engine is a row with a
`Feature` of its own (the Horn shape, `schedule::chains`), steps 35 and
37 rows of theirs.

| fragment | mode | feature | engine | measured (one thread, cores 10 and 11 unless named) |
|---|---|---|---|---|
| additives only | any | two formulas, an additive connective or unit among them | `additive` | the `additive` family (`A ⊢ A`, `A` a complete tree of `&` and `⊕` of depth d), against the focused engine forced: d 2 to 6 both under 0.3 ms (the additive path twice as fast), d 8 0.50 ms against 1.35 ms, d 10 4.8 to 9.9 against 14.8 to 19.1 ms, d 12 18 to 22 against 216 to 219 ms, d 14 97 to 191 ms against 2.7 s, d 16 0.32 s against over 20 s; classical and intuitionistic alike |
| unit-free MLL | linear, classical or intuitionistic | the sequent itself, no literal more than twice | `net` | classical: the second baseline (`bench/COMPARISON.md`, "Focus against net"): the focused engine within a factor of three of the net engine on `wide` with 8 to 1 024 literals and faster from 256 on, the net engine alone deciding `wide-m1` at 2 048 (0.64 s) within the default recursion limit; since this step's chain loop the focused engine proves that in 0.35 s, and at 4 096 it meets the memory bound. Intuitionistic, measured here on sequents of distinct atoms: wide (`a₀ ⊗ b₀, … ⊢ (bₙ ⊗ aₙ) ⊗ …`) and curried (`(a₀ ⊗ … ⊗ aₙ) ⊸ c ⊢ a₀ ⊸ (… ⊸ c)`) sequents of 1 024 and 4 096 atoms 7 to 18 times faster on the two-sided engine (wide 4 096: 0.16 s against 2.80 s), but the chain `a₀, a₀ ⊸ a₁, … ⊢ aₙ` of 1 024 and 4 096 links decided by the net engine alone (71 ms, 0.54 s), the two-sided engine meeting the recursion limit, since every link is a stable sequent and a level |
| any | intuitionistic | | `two-sided` | the general engine; where literals repeat, the Horn encodings of Partition and 3-Partition in intuitionistic mode, 10 to 10⁵ times faster than the net engine or decided where it is not within 20 s (`partition-yes` 16: 0.17 s against over 20 s; `3-partition-mll-no` 4: 0.06 ms against 2.2 s) |
| any | classical | | `focus` | the general engine; the same comparisons on the one-sided sequents in the baselines |

**Why the net rows stay.** On their feature the focused engines are as
fast or faster nearly everywhere, but they recurse once per link of a
chain of stable sequents, and the net engine is what decides such chains
within the default recursion limit; a recursion limit is an "unknown",
a factor of ten on a sequent of thousands of atoms is a fraction of a
second. The feature that would pick the faster engine is the depth a
derivation needs, which no pass computes; a focused search that hands
over to the net engine at the recursion limit would take both, and is
a follow-up, not a row.

**The bias is the focused engine's own dispatch**, documented on
`Bias::Auto` with its measurements (the factor rule without
exponentials, up to a third of the stable sequents of the rarer one; the
rarer under weakening, where the factor rule visited up to 700 times
more; both searches with exponentials, 1 520 nets against 442 and 1 576
for either alone in the second baseline). It stays in
`schedule::plan`, since it is the configuration of one engine, not a
choice between engines.

**The atom bias out of the forest** (item 9 of the assessment): `Bias`
is `search::Bias` (still `linlog::Bias` at the root), `focus/bias.rs`
computes the signs per rule, and the forest lost `bias`, `bias_under`,
`polarity`, `positive_literals` and `negative_literals`: a literal's
polarity is read off the engine's `Counts`, as the rules file had
always asked. `Kind::polarity`, the fixed polarity of a connective,
stays.

### Item 5: the hot spots, at identical counters

From the profile at the start (the first part's ranking), in the order
taken; every one a commit that claims no change of the search, and the
target set at the last of them (`hot-spots.csv`, the head of these
commits, against `after-values.csv`): every counter of the 99 decided
rows equal, every verdict equal, the CPU time of the decided rows over
100 ms 0.93 times, the rows that end at their time limit 1.11 times the
stable sequents within it (geometric means). The bias's commit was run
alone as well (`forest-bias.csv`: identical, 0.98 times).

| change | where the profile had it | measured |
|---|---|---|
| a stable sequent hashed once; the memo reads both zones in place (`memo::Zones`), so a lookup copies no key and the canonical key no `Θ` | `OccSet::hash` in `Memo::get`, in the loop check and in `Memo::refuted` (12 to 26 % on nets), the key's copy in `prove_stable` | instructions inside `Engine::run` (callgrind, explicit bias, one search): `IBM319_5_1` −10.1 %, `Railroad_railroad-010-pt_20_1` −6.3 %, `GPPP_G-PPP-1-1000_50_1` −7.9 %, `qbf/20#2` −3.8 %, `chain/128` −3.3 %, `mix/7` −2.6 %, with the next three changes |
| no clear before a copy (`take_context_from`, `take_context_any`) | `OccSet::clear` in `take_context` before a `clone_from`, from `split` (memset up to 28 %) | the widest net of the callgrind rows, `Echo_echo-d5r3_5_1`, −74.9 % with the others |
| `Θ`'s members by class kept from the stable sequent before (`ByClass`), so the copies are no sort per stable sequent | `decide_with`'s filter of `Θ` and `one_of_each`'s sort (71 % of `DLCshifumi_dlcsh_4_a_100_1`) | `DLCshifumi` 1.84 times the stable sequents in its 5 s (target set) |
| the affine initial rule in one pass over the members (`mark_literals` keeps each list's first member; a dual's lookup in `Θ` once per list) | step 25's review: quadratic and unpolled | `a₀, …, aₙ ⊢ aₙ` affine: 10 000 atoms 58 → 26 ms, 100 000 atoms 3.03 s → 73 ms (wall, the command, `--quiet`) |
| a pool's worker reads its branch in place (`Engine::ancestors`) | `Spawn::worker` 20 % on a net on four threads | no counter of one thread can move; the pool's tests and the fuzz at small limits as before |
| the linear zone's range of words (`Context::lo..hi`), one copy over the union of two ranges, a narrow forest's zone always whole | the free split's setup on wide nets: members listed from the whole bitset, copied, cleared (86 % of `NeoElection_neoelection-7.unf_10_1`) | stable sequents in 5 s, paired on one core: NeoElection 3 355 → 32 940, `DatabaseWithMutex_database20UNFOLD_5_1` 1.7 times, `CloudReconfiguration_reconf_3_04_20_1` 1.15 times, `Echo_echo-d5r3_5_1` unchanged (a first version that cleared and then copied cost it 40 %) |

**Callgrind** (adopted through the `new-tool` skill as the flake's
`valgrind.out`, recorded in `.claude/rules/bench.md` beside perf): it
counts a `memset` above 2 KiB per byte (glibc's `rep stosb`), so a
change that moves bytes from a copy to a clear reads thousands of times
its cost there; the zone's range was judged by time and throughput for
that reason.

**Not taken**: the memo's key is still both zones at the forest's width
(the hash, the comparison and the record of a stable sequent are the
width's), the sparse key the rules file names as the next gain; and
AirplaneLD's time, which is the split search itself (204 million split
steps in 5 s at 47 stable sequents, the counts having no rows for the
exponential atoms of its clauses).

### Item 6: the changes of the search, each with its panel

Every change is a commit that says it changes the search, measured on
the target set and reviewed by a panel run as a workflow: three agents
in fresh contexts, each given the diff, the claim, the changed and the
base tree, and one way to refute it: the counterexamples (Opus 5.5 at
`high`: both references and the generators against the changed engine,
more and larger sequents than the committed tests, and the base tree
beside it), the argument (Opus 5.5 at `xhigh`: the invariant stated from
the code first, then every site checked, the claim read last), and the
integers and limits (Sonnet 5 at `high`: every number the change rests
on driven to its limit). Each ran on named cores in capped scopes. A
change stands when none of the three refutes it; four panels found a
cost or a lost decision that was no wrong verdict, each fixed and
pinned by a test (below). Six panels, eighteen reviewers, more than
four million engine runs on each tree between them.

**The second reference** (`reference2`, in the scratchpad, not in the
repository): written by a fresh Opus 5.5 agent from the calculus alone,
reading README's syntax, the sequent rules, the generator and the
families but none of the engines or the committed reference. A
standalone program with its own parser (the generator's ASCII and the
library's printed syntax), every rule on every formula, splits as count
vectors, contraction budgeted per branch and deepened, a memo that
keeps a cut failure from ever answering "unprovable", a work bound of
four million. Checked before it judged: seventy sequents decided by
hand, all equal; 54 family instances, none wrong; 10 596 generated
sequents against the engines with no contradiction (7 382 proved and
2 380 refuted by both); 30 000 self-consistency checks across modes.
Every panel's counterexample reviewer ran it beside the committed one,
and the two references never contradicted each other.

#### A proof carries no cuts

*The change* (commit "Let a proof carry no cuts, on one thread as on
the pool"): `Found` is an enum, a proof or a failure with its `Cuts`, so
a proof carries none by construction; a failure after a proved premise
or alternative rests on its own cuts alone. Before, one thread's proof
carried the cuts of the alternatives tried before it, and a proved
premise's cuts went into its sibling's failure, which kept failures
`Exhausted` or out of the memo that are complete facts.

*Measured*: the target set's 99 decided rows keep every counter
(`proof-cuts.csv` against `hot-spots.csv`), no verdict moved; the
effect is on generated sequents with exponentials at their copy bound.

*Panel: not refuted.* Counterexamples: 584 256 engine runs per tree on
12 172 generated sequent-mode cases (every rule set, linear, affine and
Mix, intuitionistic linear and affine, copy bounds 0 to 4, memo default,
0 and 2, every bias, one thread and pools of 2 and 4), both references
on every case: no contradiction anywhere, no rejected proof; on one
thread 31 runs newly decided, every one from "copy bound" to
"unprovable", and the 24 sequents newly refuted were proved by neither
reference. The argument: the invariant derived first (a failure claims
no proof within the budget and outside the loop check's prunes at or
below its dependency), every site checked; sound, and the claim's
argument incomplete: what makes it sound is that the failed premise was
searched under the same branch stack and budget as the rule's
conclusion, and that what a proved sibling leaves in the memo carries
no dependency (now in `core-focus.md`); a shadow build that computed the
old cuts beside the new never found them weaker, and of the 3 673 stable
sequents newly recorded complete, `reference2` proved none. It also
noted that the pool's workers below the parallel levels change too.
Integers: nothing the diff computes; driven `copies` to `u32::MAX − 1`
and none, memo limits 0 to 2, recursion limits 0 to 2 048: nothing.

#### On a pool, a premise of `&` that gives up cancels only as one thread would

*The change* (commits "Cancel the other premise of a & on the pool only
when one fails" and, after the panel, "Cancel the right premise of a &
on the pool when the left one gives up"): a failure of either premise
cancels the other; a left premise that gives up cancels the right,
which one thread never starts; a right premise that gives up leaves the
left to run, whose failure decides the rule as on one thread. Before,
any result but a proof cancelled the other, so the pool answered the
recursion limit where one thread refuted.

*Measured*: the pool against one thread at recursion limits 4 to 16
(the first part's test, after the refinement): 18 180 pairs, 54 540
searches, no contradiction, no unasked stop; on two threads no
"unprovable" turned "recursion limit", on four threads 2, both without a
`&` (a choice's error cancelling its siblings, the mirror case left
open). `a_premise_that_gives_up_cancels_no_other` pins the witness;
`nested_premises_that_give_up_search_no_tree` pins the panel's finding.

*Panel: not refuted, one finding fixed.* Counterexamples: about 2.8
million engine runs per tree on 11 279 cases, at the default limit and
at limits 4 to 16, both references: no contradiction, no rejected proof,
no unasked stop, one thread unchanged run for run (272 919 runs); the
pool decided 130 cases one thread decided and the old pool did not
("recursion limit" to "unprovable" 130 and 130 on two and four threads,
against 5 and 2 before). The argument: sound, and the claimed cost
understated: with premises that give up cancelling nothing, `k` nested
`&` whose leaves all reach the limit searched all `2ᵏ` leaves on the
pool (262 144 stable sequents at k = 18, one thread one). Fixed by the
asymmetric rule above: 33 stable sequents at k = 16, the test failing
without it (65 536). Integers: nothing.

#### The Horn test reads the goal

*The change* (commits "Test the Horn shape on the goal's members, not on
the forest's roots" and, after the panel, "Take a clause used once as a
step of a Horn program"): `schedule::chains` reads the goal; on the
roots nothing changes. A clause used once (in the linear zone) counts
as a step of the program, as a dereliction leaves one there.

*Panel: not refuted, one finding fixed.* Counterexamples: 494 523
engine runs per tree, goals on and off the roots (sub-multisets,
derelictions, decompositions), Horn programs by hand and random, both
references on 21 822 goal sequents: no contradiction; on the roots
every verdict and counter equal. Off the roots 529 runs decided only
after, and 310 only before: goals that a dereliction leaves (`b ⊸ c` in
the linear zone is no marking and no goal), whose forward bound fell
back to `copies`; `!(a⊸b), b⊸c, c⊸d, !(d⊸e), !(e⊸f), !(f⊸g), a ⊢ g` was
proved before and stopped at the copy bound after. Fixed by taking a
clause used once; the witness is in `the_horn_test_reads_the_goal`.
The argument: a bound cannot flip a verdict (the levels up to `n` of a
search within `m > n` are those within `n`); `chains` reads only the
goal's members, in any order, repeats included (4 000 random goals);
under a stop a forward search that runs on keeps its share of the work.
Integers: `forward_copies` 0 and `u32::MAX` and the rest: nothing. *An
incident*: this panel's counterexample reviewer stopped its own run with
`pkill -f` by the test binary's name, which other panels' binaries
shared, and killed the `&` panel's runs at 18:19; that reviewer redid
them under other names. Later briefs forbid killing by name.

#### Under Mix, a sequent none of whose parts is provable fails by its parts

*The change* (commits "Refute a Mix sequent by its parts with one member
less when none has a provable part" and, after the panel, "Let the parts
of a Mix decide a sequent only as complete facts, and only with a
memo"): a stable sequent fails *hereditarily* when no non-empty
sub-multiset of its linear zone is provable with its unrestricted zone;
that holds when it has one member and fails, or when every rule but Mix
failed on it and every part with one member less fails hereditarily
(every proper part lies in one of them, and a Mix is of two proper
parts). `mix` decides those parts first (one per class of
interchangeable or repeated members), and where each fails hereditarily
and completely, the sequent fails without its partitions. The flag is a
value `prove_stable` returns, never in `Found`; the memo keeps it as a
complete failure of a new kind (`Failure::Hereditary`).

*Measured*: the `mix` family's stable sequents fall from about `3ⁿ`
towards `n·2ⁿ` (`mix(7)` 114 675 against 1 586 132, `mix(8)` 524 273
against 14 316 140, `mix(10)` 10.5 million against 1.16 billion); their
time hardly moves (`mix(8)` 1.39 s against 1.43 s), since the focus on
`~aᵢ ⊗ ~bᵢ` searches the splits of the other members, which no count
cuts under `⊕ 0`, and those splits are now the `3ⁿ` (34.6 million
against 51.7 million at 8); `mix(11)` stays undecided within 300 s.

*Panel: two reviewers found a lost decision, both fixed; no wrong
verdict.* The argument: the hereditary claim's induction holds (with
the loop check: the parts are searched with the sequent on the stack, so
their claims bound every part by the same smallest proof size; repeated
and interchangeable members map sub-multisets onto sub-multisets), every
site checked; but the claim "no verdict changes" was false: the parts'
cuts were summed into the sequent's failure, including parts the
partitions never visit (the first member stays on the left), and
`|- a, !?(s par a)` under Mix, refuted by the base in two stable
sequents, stayed at the copy bound (457 stable sequents at 3 copies,
3.8 million at 6; the command spent its whole 2 s on it). The integers:
with the memo off the parts are searched again at every level, and
`|- ~b, ~c, b, (bot + ~b), ?(c * b)` under Mix with one copy needed two
levels of recursion more than the partitions (proved at a limit of 9
before, at the limit after). Both reproduced here, fixed (a part decides
only when its failure is complete; the parts are tried only where the
engine memoizes), and pinned by `the_parts_of_a_mix_decide_only_as_facts`,
which fails on the code before the fix.
The counterexamples reviewer, on the code before the fix: about 144 000
engine runs per tree over 3 500 Mix-heavy cases (every classical rule
set with Mix, linear and affine, copy bounds 0 to 4, memo default, 0, 1
and 2, one thread and pools, the families `mix(1)` to `mix(6)`,
duplicated and interchangeable members, provable parts), both
references: no contradiction, no rejected proof; 18 cases decided only
before, by the same summed cuts (`|- ?a * d, ?a * d, ?a * d` affine
with Mix, `|- ?a * (d + 0), ?a * (d + 0), ?a * (d + 0), ~a` with Mix:
refuted again after the fix, at 209 and 641 stable sequents against the
base's 130 and 440), and 22 % more stable sequents on the provable runs.
Measured here after the fix on 960 generated Mix sequents of each kind:
10 % more stable sequents without exponentials and 3 % with them (one
more decided), against 27 times fewer on the mix family; the parts are
tried everywhere a memo is, and that cost is the price. One witness
stays: `|- (a + 0), ?~b` affine with Mix at one copy and a memo of *one*
entry is refuted by the base and left at the copy bound by the change
(with two entries and more, both refute it): the parts' entries evict
the others, the interplay of a tiny memo with decisiveness at the bound
that the rules file states and the tests allow.

#### A chain of free splits runs in a loop

*The change* (commits "Search a chain of free splits in a loop, a frame
per link" and "Keep the common free split as fast as before the chain
loop"): the split search became resumable (`next_split` on a `Walk`),
and where the left factor of a `⊗` whose split is searched is itself
such a `⊗` of at least `CHAIN_SIZE` (256) occurrences, the focus on it
is the next frame of a loop (`split::chain`) instead of a level of
recursion; its result goes back to the frame below, which searches its
right premise and resumes. The order of the steps is the recursion's,
so every counter of a search that does not reach the limit is.

*Measured*: `wide-m1` with the focused engine forced, at the default
limit: 512 literals 25 ms, 2 048 proved in 0.35 s (the recursion limit
before; the net engine took 0.64 s in the second baseline), 4 096 at
the memory bound (a frame keeps its split's counts, as the recursion's
stack did). The target set: every counter of the 96 rows decided before
and after equal; but the restructured split cost the nets with short
clause bodies throughput (TCPcondis_tcp15 −28 %), which the size
threshold, the walk's position in locals, the parts given back one by
one, the zone's range skipped on narrow forests and the inlined split
search won back to −15 % on that row and within a few percent elsewhere
(paired runs on one core). The SYJ208 problems of the target set that
end at the recursion limit still do: their depth is not a chain of
large tensors.

*Panel: not refuted.* Counterexamples: about 141 000 runs per tree
(generated sequents, also padded into chains of 150 links over 1 000
occurrences, the wide family with k up to 1 500 and m 1 to 4 and
crossed unprovable variants, at limits down to 50, and affine wide
sequents stopped at a counted poll so that the step order shows): equal
counters on 114 421 one-thread runs, 4 203 runs decided that the base
left at the recursion limit, none the other way, no contradiction with
either reference. The argument: every path out of `run_chain` checked
against `premises` and `search_splits` (marks, holds, releases, cuts,
where work and splits count); a variant with `CHAIN_SIZE` 1 and the
range tracked everywhere ran 117 536 chains and 805 323 frames with the
range invariant asserted after every mutation, identical to the base on
every line not at the limit. Its one finding, the stack: a level of the
remaining recursion takes 1.1 KiB in release and 5.6 KiB in debug since
the split search is inlined (0.9 and 4.8 before), so `PER_LEVEL` now
allows twice that (2.25 KiB, 12 KiB; commit "Allow a level of recursion
the stack a panel measured since the split search was inlined").
Integers: `CHAIN_SIZE` around 256 identical statistics; the recursion
depth needed for `wide(k, 1)` the same plateau for k from 300 to 2 400;
the zone fuzzed against a multiset at widths across both boundaries.

#### Mix is left out in affine mode

*The change* (commit "Leave Mix out of the focused search in affine
mode", from an external reviewer's read-through, made on the author's
decision): `Rules::new` sets `mix` only without weakening. With
weakening Mix is admissible branch for branch: the other part, whose
members sit in a stable zone and are never decomposed, rides along the
proof of one part through every rule (both premises of a `&`, either
side of a `⊗`, which nothing forces in affine mode) and is weakened at
its leaves, with no copy added or moved. So a proof at a level stays
one, and "unprovable" stays sound. The argument does not give that no
answer moves to "unknown" (the memo's state differs without the Mix
search's entries), which the reviewer had asked to be shown; the author
decided on the measurement: 15 360 generated runs (every rule set, copy
bounds 0 to 5, memo default and 2), no decision lost, 25 gained, 26
times fewer stable sequents; the reviewer's own measurement reproduces
(four pairs of the Mix family 32 265 against 46 537 stable sequents,
five 663 521 against 804 721). `affine_mode_leaves_mix_out` pins it.
The target set has no affine row. `Mode`, the checker, the interactive
rules and the net engine are untouched.

*Panel: no wrong verdict; three lost decisions, kept on the author's
decision.* The argument reviewer checked the carry argument site by
site (two refinements: a carried proof may drop a copy that a negative
literal of the carried part makes unconsumed, never gain one; and it
may repeat an ancestor, which the loop check prunes while the subproof
at the repeat proves the ancestor within its budget), and ran 696 024
runs (22 912 generated sequents and mutants, every rule set, bounds 0
to 5, memo default, 2 and 0): 3 decisions lost, 1 638 gained, 1 074
searches finished that the old one could not within 200 000 stable
sequents. The loss at the library's default: `|- ?(~c par ~c), ?(~b *
~b), !?(?b * c)` affine with Mix at three copies, proved before (its
proof has no Mix node and four copies on a branch, the memo's proofs
reused below their budget) and at the copy bound after, proved at four
by both and by the command's default; reproduced here. The other two:
refutations at bound 2 under a memo of two entries, decided at 3. Every
Mix proof the old search found came after a failure cut by the budget,
never after a complete one, as the argument predicts. The author kept
the change on these numbers; the commit message, the rules note and
this report carry them. The counterexamples reviewer (5.6 million runs
of the change, 3.3 million of the base, both references on every case)
found the same: no wrong verdict, no rejected proof; at one thread the
change's affine-with-Mix equals its affine run for run; 1 998 runs
decided only after (1 163 from the copy bound to unprovable, 835 from
its time stop), and 10 runs decided only before, two sequents at two
copies under a memo of two or three entries, proved by the change at
three copies or with the default memo. Integers: nothing the diff
computes.

#### Item 6's measurements, after

On the head's binary, cores 12 to 15 as before, the same commands:

| case | at the start of the step | after |
|---|---|---|
| `SYJ204+1.014` in `01`, no copy bound, 60 s | one thread proves it in 1.98 s, 15 708 917 stable sequents, level 9; a pool of two in 5.3 to 5.4 s, 70.4 to 71.5 million | one thread the same, counter for counter; a pool of two 5.42 to 5.45 s, 70.8 to 71.2 million |
| `LCL181+1` in `01`, 5 s | one thread 3 233 560 levels undecided; pools of two refute it in 3.5 to 83 ms, of four in 0.6 to 1.5 ms | one thread 3 161 298 levels undecided; pools of two 0.6 to 4.2 ms (levels 28 to 544), of four 0.6 to 1.3 ms |
| `SYN393+1` in `cbn` | one thread 10 s undecided (2 723 076 levels); pool of two undecided; pool of four refutes in 0.8 to 4.3 ms; the default on four cores 0 of 5 within 2 s, on eight 5 of 5 | one thread 2 675 335 levels undecided; pool of two undecided; pool of four refutes in 0.8 to 11.7 ms; `--jobs 4 --pool-after 100ms` 1 of 5 within 2 s (1.62 s), the default on four cores 0 of 5; on eight cores (2 to 5 and 12 to 15) refuted every time in 0.10 to 0.17 s (before 0.10 to 0.19 s) |
| `SYJ212+1.014` in `cbn`, 10 s | the default without a bound undecided at level 5; `--bias factors` refutes in 1.37 s at level 5 | **the default refutes it in 2.74 s at level 4**; `--bias factors` in 0.96 s at level 4 (a proof carries no cuts, so the forward search's levels are decided one earlier) |
| `a₀, …, aₙ ⊢ aₙ` affine | 3.03 s at 100 000 atoms | 73 ms |

What these are: `LCL181+1` and `SYN393+1` are where the loop check meets
the memo's cut entries, termination on dyadic sequents, which step 25
measured and this step was not to change (`plan/later.md`); the
`SYJ204` pool of two still does 4.5 times one thread's work, which is
the order the pool's tasks take and the memo's interleaving, not a
change of this step.

### The target set before and after

On cores 2 and 3, `bench/targets.sh`, each against the one before and
against `after-values.csv` (the end of the first part):

| label | state | decided rows' counters | verdicts | CPU time, decided rows over 100 ms | stable sequents of the rows at their time limit (38, without `mix/11`) |
|---|---|---|---|---|---|
| `forest-bias` | the bias out of the forest | all 99 equal | equal | 0.98 | 1.04 |
| `hot-spots` | the five hot spots before the range | all 99 equal | equal | 0.93 | 1.11 |
| `proof-cuts` | a proof carries no cuts | all 99 equal | equal | 0.98 against `hot-spots` | 1.00 |
| `after-changes` | the `&`, the Horn test, the Mix parts, the chain loop, the zone's range | 96 equal, the three Mix rows changed | equal | 1.02 against `proof-cuts` | 0.96 |
| `after-fast` | the common free split restored | 96 equal (99 against `after-changes`) | equal | 0.99 against `proof-cuts` | 1.01 |
| `after-panels` | the panels' fixes | 96 equal, the three Mix rows changed | equal | **0.84** against `after-values` | **1.20** against `after-values` |

The Mix rows: `mix/8` 524 273 stable sequents (14 316 140 before) in
1.22 s (1.37), `mix/9` 2 359 279 (129 009 092) in 10.2 s (12.7),
`mix/10` 10 485 741 (1 161 737 180) in 90 s (119); `mix/11` stays at
its 300 s limit. Rows at their time limit, against `after-values`:
NeoElection 11.9 times the stable sequents, CloudDeployment 2.3,
DLCshifumi and DLCround_dlcro_07 1.8, DatabaseWithMutex 1.5; the short
clause bodies of TCPcondis 0.6 to 0.75 and NeighborGrid 0.68 by these
runs, whose throughput by day moves by a fifth between runs (paired
runs on one core put TCPcondis at −15 % and the rest within a few
percent). The affine change touches no row: the set has none.

### The library runs

**ILTP** (`linlog prove --logic intuitionistic --translation T` and
`--logic classical`, `--file bench/iltp/Problems --workers 2 --cores
across --stats --output DIR`, the default 2 s, cores 4 and 5 as step 25
ran it; every proof read back and checked), on the head before the
panels' last fixes:

| run | valid | not valid | unknown | error | step 25 | first part |
|---|--:|--:|--:|--:|---|---|
| intuitionistic, cbn | 68 | 28 | 177 | 1 | 68, 28, 177, 1 | the same |
| intuitionistic, cbv | 71 | 16 | 186 | 1 | 72, 16, 185, 1 (its review 71) | 70 (slower cores) |
| intuitionistic, 01 | 56 | 12 | 205 | 1 | 56, 12, 205, 1 | the same |
| classical | 155 | 1 | 116 | 2 | 155, 1, 116, 2 | the same |

No read-back failed; every proved Theorem and refuted Non-Theorem
agrees with ILTP's intuitionistic status; the errors are step 25's.

**The LLTP library through the batch by default on four cores** (`linlog
prove --file bench/lltp/ILL`, then `CLL`, cores 2 to 5): 4 512 answers,
none twice, no error; **2 156 proved and 142 refuted**; no verdict
against the second baseline's classical pass or the first part's run
(2 150 and 142 on cores 8 to 11): 8 decided more (two BridgeAndVehicles
nets, four PolyORBLF nets, DLCshifumi and SimpleLoadBal), 2 fewer
(HexagonalGrid, HouseConstruction), at the time limit. Step 24's review
had 2 193 and 142 on cores not recorded.

### The profile at the end, beside the start

The same command, rows and perf as the start (`linlog-bench one
--problem P --mode given --engine auto --jobs 1`, the families with
`--timeout 30`, the nets with `--copies 3 --timeout 5`,
`DatabaseWithMutex` with `--recursion-limit 16384`; `perf record -e
cpu_atom/cycles/u -F 999 --call-graph lbr`, one row per core), on the
head's binary with debug information (`307357353592`, before the
affine change, which no row touches), on cores 4 and 5, the kind of
the start's 10 and 11. Read as text, by function and by kind of work.

**By kind of work** (self time; the median over the 44 rows counts a
row without that work as zero):

| kind of work (self) | median % start | max % start | median % end | max % end |
|---|--:|--:|--:|--:|
| memset (clears of zones) | 4.9 | 35.4 | 0.6 | 2.1 |
| memmove (copies of zones) | 6.3 | 30.9 | 3.2 | 25.8 |
| hashing | 2.2 | 26.5 | 0.9 | 31.1 |
| memcmp (comparisons of keys) | 1.0 | 12.8 | 0.8 | 27.1 |
| member lists (Context::iter) | 4.8 | 36.1 | 1.6 | 7.3 |
| Context::canonical_from | 0.0 | 0.0 | 2.2 | 38.0 |
| duals in forced chains | 4.4 | 41.4 | 6.8 | 42.5 |
| split counts (`Split`) | 1.8 | 46.6 | 2.1 | 77.3 |

The clears and the member lists are gone from the wide nets (the
zone's range), the copies halved; `canonical_from` shows as a function
of its own at the end only because the start inlined it. What the wide
nets spend now is the memo's key: hashing and comparing both zones at
the forest's width (`CloudDeployment_deploy_5_b_100_1` 31 % and 27 %,
`DLCround_dlcro_07_b_5_1` 23 % and 23 %), and the canonical key, built
for every stable sequent of a forest with two equal formulas
(`HypercubeGrid_hc3k4p4b12_5_1` 38 %, `JoinFreeModules_joinFree-10_50_1`
28 %). The functions themselves cannot be set side by side by name: the
inlining folded `split`, `focus` and `prove` into `focus_on` and
`prove_stable`.

**Per row** (CPU seconds of the run, its verdict, its stable sequents,
which for a row at its time limit is its throughput; the top three at
the end by self time):

| row | start: CPU s, verdict, stable sequents | end: CPU s, verdict, stable sequents | top three at the end (%) |
|---|---|---|---|
| AirplaneLD-pt-0010_20_1 | 5.0, unknown, 47 | 5.0, unknown, 47 | focus_on 50.8, dual_from 13.5, search_splits 12.5 |
| CloudReconfiguration_reconf_3_15_20_1 | 5.0, unknown, 3484270 | 5.0, unknown, 5020857 | focus_on 21.1, hashing 14.5, dual_from 11.0 |
| CloudReconfiguration_reconf_4_01_100_1 | 4.9, unknown, 2078949 | 5.0, unknown, 4520510 | hashing 18.7, focus_on 17.6, memcmp 15.7 |
| DLCround_dlcro_07_b_5_1 | 4.9, unknown, 1841157 | 5.0, unknown, 4129291 | memcmp 23.2, hashing 23.0, focus_on 12.8 |
| DLCshifumi_dlcsh_4_a_100_1 | 4.9, unknown, 149426 | 5.0, unknown, 308451 | prove_stable 62.5, memmove 10.7, hashing 5.6 |
| Diffusion2D_2D8_gradient_20x20_100_5_1 | 1.7, proved, 21036 | 1.6, proved, 21036 | dual_from 30.1, search_splits 23.8, focus_on 21.9 |
| FlexibleBarrier_flexbar_18_b_5_1 | 5.0, unknown, 1350347 | 5.0, unknown, 2937369 | memcmp 23.7, hashing 16.9, focus_on 13.8 |
| HypercubeGrid_hc3k4p4b12_5_1 | 5.0, unknown, 46112 | 5.0, unknown, 85722 | context::Context::canonical_from 38.0, prove_stable 13.7, focus_on 9.4 |
| NeoElection_neoelection-7.unf_10_1 | 5.0, unknown, 2551 | 5.0, unknown, 43349 | focus_on 31.0, prove_stable 21.1, search_splits 8.6 |
| Angiogenesis_angiogenesis-15_20_1 | 5.0, unknown, 463401 | 5.0, unknown, 473479 | focus_on 42.5, search_splits 10.5, dual_from 7.1 |
| AutoFlight_afcs_48_a_50_1 | 5.0, unknown, 482371 | 5.0, unknown, 728319 | focus_on 28.7, prove_stable 22.0, search_splits 12.4 |
| ClientsAndServers-0010-1_100_1 | 4.9, unknown, 485099 | 5.0, unknown, 434536 | chain 23.2, focus_on 13.9, memmove 12.0 |
| ClientsAndServers-0020-3_50_1 | 5.0, unknown, 361030 | 5.0, unknown, 330664 | chain 22.5, memmove 13.3, focus_on 11.6 |
| Echo_echo-d2r9_100_1 | 5.0, unknown, 363802 | 5.0, unknown, 425920 | focus_on 35.5, dual_from 11.8, search_splits 8.8 |
| GPPP_G-PPP-100-1000_100_1 | 5.0, unknown, 31550 | 5.0, unknown, 31131 | search_splits 36.9, context::Context::canonical_from 25.2, context::Context::insert 13.9 |
| JoinFreeModules_joinFree-10_50_1 | 5.0, unknown, 10920576 | 5.0, unknown, 12396024 | context::Context::canonical_from 27.9, hashing 16.9, memo::Memo::find 9.3 |
| Kanban-500_50_1 | 5.0, unknown, 95369 | 5.0, unknown, 127002 | focus_on 34.4, context::Context::canonical_from 16.1, chain 10.4 |
| PermAdmissibility_unf-8x8-4stageSEN-50_10_1 | 5.0, unknown, 103903 | 5.0, unknown, 155524 | chain 24.5, memmove 16.9, focus_on 6.2 |
| QuasiCertifProtocol_QCertifProtocol_32-unfold_20_1 | 5.0, unknown, 4 | 5.0, unknown, 4 | focus_on 45.9, dual_from 10.6, search_splits 9.9 |
| ResAllocation_RAS-C-50_100_1 | 5.0, unknown, 2795352 | 5.0, unknown, 3210105 | focus_on 34.2, search_splits 8.6, context::Context::canonical_from 6.8 |
| Solitaire_soli2_counter_20_1 | 5.0, unknown, 657143 | 5.0, unknown, 670967 | focus_on 36.0, search_splits 11.0, dual_from 8.0 |
| family-mix-8-0 | 1.9, unprovable, 14316140 | 1.5, unprovable, 524273 | counts::Split::shift 38.7, counts::Split::excludes 17.3, counts::Split::flip 16.7 |
| DES_des_00_a_20_1 | 3.9, proved, 718236 | 4.0, proved, 718236 | focus_on 36.8, search_splits 12.7, dual_from 12.0 |
| Parking_parking_2_8_5_1 | 5.0, unknown, 314121 | 5.0, unknown, 306530 | focus_on 44.6, search_splits 15.6, forced_side 8.6 |
| PermAdmissibility_unf-8x8-4stageSEN-05_100_1 | 4.9, unknown, 300920 | 5.0, unknown, 349191 | focus_on 32.7, dual_from 17.4, search_splits 9.8 |
| PhaseVariation_5-10_phaseVariation_50_1 | 5.0, unknown, 74243 | 5.0, unknown, 77374 | focus_on 46.4, dual_from 42.5, search_splits 2.3 |
| SimpleLoadBal_simple_lbs-15_50_1 | 4.9, unknown, 159887 | 5.0, unknown, 267517 | focus_on 27.4, memmove 25.8, dual_from 15.0 |
| TCPcondis_tcp30_50_1 | 5.0, unknown, 5223062 | 5.0, unknown, 4892162 | focus_on 36.0, search_splits 9.4, context::Context::canonical_from 5.8 |
| DatabaseWithMutex_database20UNFOLD_5_1 | 5.0, unknown, 68224 | 5.0, unknown, 188443 | focus_on 27.3, prove_stable 21.3, dual_from 11.9 |
| family-mix-9-0 | 17.4, unprovable, 129009092 | 12.5, unprovable, 2359279 | counts::Split::shift 40.4, counts::Split::excludes 18.4, counts::Split::flip 16.7 |
| family-mix-10-0 | 29.9, unknown, 215449068 | 30.0, unknown, 4590493 | counts::Split::shift 38.7, counts::Split::excludes 21.3, counts::Split::flip 16.2 |
| family-mix-11-0 | 30.0, unknown, 216832317 | 30.0, unknown, 4590107 | counts::Split::shift 39.0, counts::Split::excludes 21.6, counts::Split::flip 16.2 |
| DLCround_dlcro_03_b_50_1 | 5.0, unknown, 5862462 | 5.0, unknown, 8961601 | hashing 23.5, memcmp 17.0, focus_on 14.5 |
| Eratosthenes_eratosthenes-050_5_1 | 5.0, unknown, 409691 | 5.0, unknown, 398606 | counts::Split::shift 25.9, focus_on 17.3, search_splits 13.4 |
| NeighborGrid_z_4d_3n_2m_c_2_3_20_1 | 5.0, unknown, 115239 | 5.0, unknown, 135417 | focus_on 34.7, dual_from 16.4, search_splits 12.1 |
| Peterson-4_50_1 | 5.0, unknown, 5466873 | 5.0, unknown, 6004157 | focus_on 30.9, dual_from 11.2, prove_stable 9.3 |
| QuasiCertifProtocol_QCertifProtocol_10-unfold_20_1 | 5.0, unknown, 2730 | 5.0, unknown, 2730 | focus_on 45.5, search_splits 11.4, dual_from 10.0 |
| TCPcondis_tcp15_20_1 | 5.0, unknown, 3406796 | 5.0, unknown, 2926215 | focus_on 32.0, dual_from 11.4, search_splits 10.9 |
| TokenRing-10-unfolded_100_1 | 5.0, unknown, 3118149 | 5.0, unknown, 3780602 | focus_on 24.3, dual_from 18.2, prove_stable 12.1 |
| DES_des_01_b_10_1 | 5.0, unknown, 26326640 | 5.0, unknown, 29014215 | focus_on 20.4, hashing 16.0, search_splits 8.9 |
| DES_des_50_b_20_1 | 5.0, unknown, 20698728 | 5.0, unknown, 25772278 | focus_on 19.2, hashing 16.2, memcmp 8.2 |
| CloudDeployment_deploy_5_b_100_1 | 5.0, unknown, 763006 | 5.0, unknown, 2318943 | hashing 31.1, memcmp 27.1, focus_on 12.3 |
| CloudReconfiguration_reconf_3_04_20_1 | 5.0, unknown, 3301474 | 5.0, unknown, 5045901 | focus_on 21.1, hashing 13.6, dual_from 11.4 |
| CloudReconfiguration_reconf_3_10_100_1 | 5.0, unknown, 3657333 | 5.0, unknown, 5310523 | focus_on 22.8, hashing 14.7, memcmp 9.9 |

At the end the rows visit more stable sequents in their time than at
the start on the same kind of core, by up to 17 times (NeoElection) and
2 to 3 times on the widest nets (DatabaseWithMutex, CloudDeployment,
DLCround, FlexibleBarrier, DLCshifumi), and fewer on four rows with
short clause bodies (ClientsAndServers −8 and −10 %, TCPcondis −6 and
−14 %), where the chain's frames and the resumable split search cost
what the paired runs measured. `mix/8` and `mix/9` take 1.5 and 12.5 s
against 1.9 and 17.4 s; `mix/10` and `mix/11` still reach their 30 s.

### Item 7: ready for quantifiers

For the design note of step 28 (`plan/notes/api.md`). The interface
keeps their place, as the first part said: a first-order goal is a
`Task` whose forest carries terms, `Decide`, `Answer` and the verdict do
not change, and a row of `DISPATCH` with a fragment bit for the
quantifiers sends it to the engines that take it. What changes is inside
the focused engine.

**Where a trail of bindings goes.** Beside the branch, on `Engine`: a
stack of the variable bindings made since the search began (`trail`)
with a mark per choice point, undone by truncation exactly where the
arena's pending nodes are released, since a binding, like a pending
node, belongs to the branch that made it: `prove_stable` on a failure,
the four places where a first premise was proved and the second failed
(`premises`, `both`, `parts`, the forced chain in `split`), each
alternative of `choose_here` that fails, and a frame of `split::chain`
whose split failed. Bindings cross the premises of a `⊗` (the left
premise's unifier constrains the right one), so they are not undone
between them, only when the split fails; on a pool, a worker starts from
the spawning engine's trail as it reads its branch in place now
(`Engine::ancestors`), and a binding made in one alternative never
reaches another. A copy from `Θ` of a formula under a quantifier takes
fresh variables per copy: an occurrence id no longer names the formula
of a zone, so the zones become (occurrence, instance) pairs, or the
instances are renamed apart in a table beside the forest.

**Which prunes assume ground atoms**, and what each needs:

- *The initial rules and the duals* (`initial`, `dual_in`, `dual_from`,
  `mark_literals`, `meets`): a literal meets "its dual" by the forest's
  list of the other sign of the same atom. With terms the list is that
  of the predicate, and a dual is a member that unifies; the first one
  found is no longer the only one worth trying.
- *Forced splits* (`Forced::Dual`, `Forced::Duals`, `literal_tensor`,
  the cursors): a positive literal factor "takes exactly its dual, the
  first one", sound because every dual is the same formula. With terms
  the candidates unify differently, and the forced split becomes a
  choice among them; "the dual in `Γ` before a copy from `Θ`" rests on
  the same identity.
- *Interchangeable occurrences* (`Classes`, the canonical keys, one of
  each kind, the canonical splits): equal terms are interchangeable; with
  variables two occurrences of one term differ once their variables are
  bound differently, so a class holds only ground subterms, or the class
  is of the instance.
- *The memo and the loop check*: keys are the zones' occurrence ids, a
  sequent determined by them only when nothing is bound. A first-order
  key adds the bindings of the zones' variables (or the memo stays off
  on a branch with open bindings), and the loop check compares instances.
  `Exhausted`, `Complete` and the new `Hereditary` keep their meaning
  per instance.
- *The counts* (`Counts`, the interval check, `Refutation::Unbalanced`)
  are per atom; per predicate they stay sound (a literal pairs only with
  the same predicate) and become weaker; the count equation counts
  connectives and is unaffected; the atom bias is per predicate.
- *The net engine* links literals pairwise; with terms a linking also
  needs one unifier for all its links, which the correctness criterion
  does not see; the additive path takes no quantifier.

### Decisions

- **The dispatch's rows did not move.** The measurement shows the net
  rows kept for depth, not speed; a row that hands a focused search
  over to the net engine at the recursion limit is the better feature
  and a follow-up, not a guess this step makes.
- **The bias stays the focused engine's own choice**, in
  `schedule::plan`, with its measurements on `Bias::Auto`: it configures
  one engine and chooses none.
- **`Found` became an enum**, so that a proof cannot carry cuts by
  construction; the change of the search this made is the one item 1's
  report left open, and it was given its panel.
- **The `&` rule's cancellation follows one thread exactly** rather than
  "cancel on failure only", which the step's wording suggested: the
  panel showed the latter searches every leaf of nested premises that
  give up.
- **The Mix parts are tried wherever the engine memoizes**, at 3 to 10 %
  more stable sequents on generated Mix sequents, for the `3ⁿ` to `n·2ⁿ`
  of sequents none of whose parts is provable; only complete failures of
  the parts decide, as the panel's two findings required.
- **The chain loop starts at a left factor of 256 occurrences**: the
  frames cost nets with short clause bodies up to a fifth of their
  throughput, and the loop is for chains of hundreds of links.
- **The zone's range is skipped on forests of eight words or fewer**,
  and copied over the union of two ranges: both measured.
- **The late cancellation at a `&`** (a stolen task of the sibling
  premise running on until the failed premise's worker leaves its
  scopes) is not fixed: the failed premise's continuation runs only when
  the stolen task returns, on the same stack, and a choice's failure
  inside it means the premise's only in tail position, so raising the
  flag earlier needs that knowledge passed down; it was not reproduced
  by the first session either. Said in `core-parallel.md`.
- **`LCL181+1`, `SYN393+1`, `SYJ212+1.014` and `SYJ204+1.014`** were
  measurements, as item 6 says; termination on dyadic sequents is not
  this step's, and none of this step's changes aimed at them (their
  numbers after are below).

- **Mix in affine mode is left out of the search**, on the author's
  decision twice: first on the argument and a measurement of no loss,
  then again on the panel's larger measurement of 3 decisions lost at a
  copy bound against 1 638 gained, no wrong verdict either time.

### Verification

- Every commit: clippy with `--deny warnings` and `cargo test
  --workspace` in a capped scope; both `cargo hack` checks (each
  feature, every pair; the split search touches `parallel`'s gates) at
  the end; the rustdoc builds without a warning (the dispatch table and
  `Bias::Auto`'s documentation).
- The target set at every stage (above), the reference tests, the
  pool's differential run at limits 4 to 16 before and after, six
  panels, ILTP and LLTP through the batch, the profile at the end.
- `nix flake check` passes on the final tree (every check: build,
  clippy, test, test-debug-assertions, doc, deny, features, export,
  rocq, bench, deadnix, actionlint, treefmt, claude-hooks), run last,
  after the panels' timed runs.

### Deviations and assumptions

- **The time-limited rows by day** move by about a fifth between runs of
  the target set on cores 2 and 3 while other work runs; paired runs of
  two binaries on one core judged each regression and gain here.
- **The library runs** ran on cores 2 to 5 and 4 and 5 (the first part:
  8 to 11 and 12 and 13), beside panels on other cores; their verdict
  counts differ from the first part's only at the time limit.
- **The profile at the end** ran on cores 4 and 5, the same kind as the
  start's 10 and 11 (both 4.0 GHz), since the panels held 10 and 11.
- **The second reference** reads and parses text and is a binary of its
  own; the step asked for one "sharing no code with the committed one",
  and it shares none with the crate either.
- **A reviewer ran one command unscoped** (the second reference's
  author, a three-line batch of the library) and **one killed processes
  by name** (the Horn panel's, `pkill -f` on a test binary shared with
  the `&` panel's runs, which that panel redid); briefs since forbid it.
- **Commits waited about fifteen minutes** for the signing passphrase,
  which had outlived gpg's two-hour cache (the edits stayed on disk and
  were committed in their units once it was entered); a loop keeps the
  cache warm now.

### Open questions and follow-ups

- **A sparse memo key**: both zones at the forest's width are hashed,
  compared and stored per stable sequent; the zone's range made the
  rest proportional to the members, and the key is what is left (the
  rules file names it).
- **A focused search that hands over to the net engine at the
  recursion limit**, the feature the dispatch's net rows stand in for.
- **A choice's error still cancels its siblings on a pool**, the mirror
  of the `&` case: a pool may answer the recursion limit where one
  thread refutes (`|- !?~c, (?(a * c) par ?!~c)` with Mix at a limit of 9
  on four threads).
- **The late cancellation at a `&`** (above, in the decisions).
- **The Mix parts' cost**: 3 to 10 % more stable sequents on generated
  Mix sequents; trying them only on sequents of more than a few members
  would take most of the gain and little of the cost (not measured).
- **The split search under weakening** is monotone (`plan/later.md`, with
  the Horn bound under affine with Mix, both from the reviewer's note).
- **TCPcondis_tcp15_20_1** keeps 15 % less throughput since the split
  search became resumable.
- **`mix(11)`**: its stable sequents are `n·2ⁿ` now, its splits still
  `3ⁿ` (the negative tensors' splits under `⊕ 0`, which no count cuts).

### The commits, second session

1. Keep the atom bias in the focused engine, not in the forest
2. Hash a stable sequent once, and let the memo read its zones in place
3. Find a stable sequent's initial pairs and its copies from Θ in linear time
4. Dispatch by a table of rows, with the measurement behind each
5. Let a pool's worker read its branch in place instead of copying it
6. Let a proof carry no cuts, on one thread as on the pool
7. Cancel the other premise of a & on the pool only when one fails
8. Test the Horn shape on the goal's members, not on the forest's roots
9. Refute a Mix sequent by its parts with one member less when none has a provable part
10. Search a chain of free splits in a loop, a frame per link
11. Count the search's instructions with callgrind, as the bench rules say
12. Keep the range of words a linear zone occupies, and copy and clear only that
13. Keep the common free split as fast as before the chain loop
14. Record the target set at each stage of the focused engine's second pass
15. Cancel the right premise of a & on the pool when the left one gives up
16. Say that a pool never contradicts one thread, not that it decides alike
17. Say in README how the engine for a goal is chosen, and what Mix and long tensors cost now
18. Take a clause used once as a step of a Horn program
19. Let the parts of a Mix decide a sequent only as complete facts, and only with a memo
20. Record the target set after the panels' fixes
21. Leave Mix out of the focused search in affine mode
22. Allow a level of recursion the stack a panel measured since the split search was inlined
23. Report step 26, second session (this part, the plan's status entry, the second reference in the search rules)

Nothing is pushed.

## From the review of the second session

Accepted without a fix to the code. The prompts were amended (below).

- **Checked**: clippy, the tests (and `cargo test -p linlog`), both
  `cargo hack` runs, `cargo deny`, `nix flake check`.
- **The target set again**, by the review on the head (cores 2 and 3):
  every verdict and every counter of the decided rows equal
  `after-panels.csv`, so the last two commits (Mix left out in affine
  mode, which no row has, and the stack per level) move none, at 0.92
  times its CPU time; against `after-values.csv` only the three Mix rows
  differ, at 0.80 times.
- **The families** through the harness (`linlog-bench run
  --all-families --timeout 5`): no mismatch and no error (61 proved, 36
  refuted, 26 unknown).
- **The LLTP library** through the batch by default on the slowest four
  cores: 4 512 answers, 2 194 proved and 142 refuted, against step 24's
  review 2 193 and 142, with no verdict against it. Five nets decided
  only now (DLCshifumi, Diffusion2D, two PolyORBLF, SimpleLoadBal), four
  problems only then, all at the 2 s limit: paired on one core the
  two-sided engine's time moves both ways by up to 8 % on the ILTP images
  (`SYJ203+1.008` in `cbv` 1.90 to 2.03 s, `SYJ204+1.014` in `01` 2.10 to
  1.98 s, `SYJ203+1.008` in `01` unchanged).
- **Inputs at the new code's limits**: the Mix parts' recursion counts
  against the recursion limit (10 000 copies of `(a * ~a) + 0` under Mix
  meet the default limit as an unknown, and with the limit raised are
  refuted at a depth of 10 000 without a fault, the thread sized by the
  new per-level estimate); the time limit holds in the parts' search; in
  affine mode with Mix, 1 000 such members are refuted at once where the
  binary before the step reached its time limit (the splits of
  interchangeable members stay quadratic, and polled).
- **The panels' third member ran Sonnet 5.** Claude Code resolves the
  workflow alias `sonnet` to `claude-sonnet-5`, not the API's Sonnet
  5.5 the prompt named: 1 046 of the agents' requests carry that id. The
  prompts now name the alias and what it resolves to.
- **Kept**: step 26's second reference lives in that session's scratch
  directory under `/tmp`, which a reboot clears. It judges proofs and
  refutations without `!` only, as the committed one does, so step 27's
  prompt names other judges for the Horn engines' refutations.
