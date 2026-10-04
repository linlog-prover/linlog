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
