# Step 27 report: Horn programs, an engine for Petri nets, coverability, and problems from practice

Two sessions on Opus 5.5 at `xhigh`, both unattended: the first, overnight,
built the Horn engine (reachability of a Petri net's markings, the proof
read off the firing sequence) and gave it the dispatch's row for Horn
programs with a clause under `!` in linear mode; the second, by day on
2026-10-05, added coverability in affine mode, the net's state equation,
two more refutations for linear mode, the qcover suite with a reader for
its format, and the affine row. Its sections come first; the first
session's report follows under "The first session", its checklist
merged into the one below.

## Outcome of the step

- **A Horn program with a clause under `!` goes to the Horn engine in
  every mode now.** In affine mode, where a proof may weaken what a
  firing sequence leaves, the engine decides coverability by the
  backward algorithm on upward-closed sets, which always ends (Dickson's
  lemma), and its `Unprovable` is the suite's first refutation of an
  affine sequent with exponentials. On the 176 coverability problems of
  the qcover suite at 5 s it decides **162** (59 provable, 103 not) where
  the default before, the two-sided engine, decided 8; every one of the
  12 files that state a result is answered as stated, every proof is
  checked, and no verdict contradicts another engine's.
- **The state equation refutes in both modes**: a simplex beside the
  search proposes weights for the places (Farkas' lemma), an exact check
  in integers confirms that no clause that can fire raises the weighted
  count while the goal asks it raised, and the goal is unprovable with
  the new `Refutation::StateEquation`, which names the weights. With it,
  transitions that can never fire are left out, and a backward search on
  the reversed net runs beside the forward one.
- **The linear row keeps its scope.** On 7 000 random programs near the
  Horn shape (the review's design) the row now loses none of the focused
  engine's refutations in any linear mode, where the first session's
  review had found 84, 80 and 53, and decides 548 to 580 more per mode.
  On the library's 3 137 nets the engine decides 3 071 (3 026 in the
  first session), no verdict against another engine, every proof
  checked.
- **Every change of the search went through a panel**: three panels of
  three agents in two rounds, each agent trying to refute one way, the
  counterexample agents with judges of their own. No wrong verdict was
  found. The claim that no
  earlier decision is lost was refuted in both rounds (the simplex's
  memory and set-up, the release of the helpers' memory, a quadratic
  query of the trie, the proof's memory charge); each was fixed and its
  witness run again.
- **The qcover suite is fetched by the flake** (`nix build .#qcover -o
  bench/qcover`), read by `linlog::mist` and the command's
  `--input-format spec`, and run by the harness with `--spec`.
- **Left open**: 14 qcover problems at 5 s, the integer and trap
  refinements of the state equation, a sparse basis for the simplex, and
  a forward search for coverability (the follow-ups in `plan/later.md`).

## The checklist

| item | state | evidence |
|---|---|---|
| 1. reachability: count vectors, transitions by input place, a visited set, a proof the checker accepts | done (first session) | "The first session" |
| 1. the default only where it beats the forward focused search | done (first session); kept | `bench/defaults/horn-nets.csv`; 3 071 of the 3 137 nets now, `horn-nets-coverability.csv` |
| 2. coverability: the backward algorithm on upward-closed sets in affine mode | done | `core/src/search/horn/cover.rs` (`79d2c482`, the trie `216ab67c`, `126fc0e5`) |
| 2. its termination argument | done | `cover.rs`'s module documentation, `core-horn.md`, "Why the new refutations are sound" |
| 2. a panel on it | done, twice; no wrong verdict | "The panels" |
| 2. `Unprovable` in affine mode with exponentials | done | 103 qcover refutations; `decides_unbounded_nets` |
| 3. the 176 qcover instances fetched at a pinned commit, not committed | done | `modules/bench.nix` (`9028f7ff`) |
| 3. a reader for the format, parametric markings as `!p` | done | `core/src/mist.rs` (`849ddad5`, `77f9ac25`, `3d2680c7`, `dc3d6bb2`, `307d978e`), `reads_problems` |
| 3. measured with both engines at the same limit | done | "The measurement", `bench/defaults/horn-qcover.csv` |
| 4. the net-only focused follow-ups retired | done (first session) | `plan/later.md` |
| the review's condition: the row loses none of the focused engine's refutations near the Horn shape, or narrows | done: loses none | "The measurement", `bench/defaults/horn-near.csv` |
| the state equation with a Farkas certificate checked exactly, and its panel | done | `equation.rs` (`f89bda18`, `23d6c9dc`, `746f9675`), `checks_the_weights_exactly`; panels A and C |
| the decision whether affine mode moves to the new engine | done: it moves | `0b77129b`, "The decisions about the default" |
| the reference test with the new engines | done | `engines_agree_on_horn_programs` in affine mode too |
| the target set | done: the focused and net engines' 118 decided rows equal `after-horn.csv` counter for counter; the Horn rows 4 more decided, 3 with other counters, a third of the CPU | `bench/targets/after-coverability.csv` |
| the families | done: 62 proved, 38 refuted, 23 unknown, no mismatch | `linlog-bench run --all-families --timeout 5` |
| the library | done: 3 753 proved, 142 refuted, 617 unknown (the review's: 3 701, 142); the Horn engine against no header | the command's batch on `bench/lltp` |
| a rules file for the new modules, a row in `core.md`'s table | done | `core-horn.md` (coverability, the state equation, dead transitions, the backward search, memory and time), `core-inputs.md` (`mist.rs`), `core.md` |
| README for the engine and the suite | done | the Horn paragraph, the `.spec` example, the harness paragraph, the built list |
| clippy, the tests, both `cargo hack` runs, the public-items check | passed | this session's runs |
| `nix flake check` | passed: every check (the build, clippy, the tests in release and with debug assertions, the documentation, deny, the features, the exports, Rocq, the harness's families, the formatters) | on the code of `440a7907`, the last code commit; later commits are data and documents |

## The second session: coverability, the state equation, the qcover suite

### What was built

- **Coverability in affine mode** (`core/src/search/horn/cover.rs`, the
  backward search). In affine mode a proof may weaken the tokens and the
  clauses a firing sequence leaves, so the goal asks for a reachable
  marking that *covers* the target, and a clause used once is used at
  most once. The backward algorithm of Abdulla, Čerāns, Jonsson and Tsay
  keeps the minimal markings from which the target can be covered: from
  an element `m` and a transition `t` with an output on a place `m`
  marks, `max(m − out(t), 0) + in(t)`, kept unless an element already
  kept is at most it. A kept marking at most the initial one ends the
  search, and its line of parents, read upward, is the firing sequence;
  an empty queue refutes. The elements are indexed by a trie of their
  places, so the question "is some element at most this marking?" walks
  only the marking's places. The proof is the reachability proof with a
  `Weaken` for every token beyond the goal's and every clause used once
  that no firing used (`proof.rs`).
- **The state equation** (`equation.rs`), in both modes. A firing
  sequence fires each transition some number of times, so the target
  less the initial marking is a non-negative combination of the
  transitions' effects (in affine mode at least the target). Where that
  has no rational solution, Farkas' lemma gives integer weights for the
  places under which no transition raises the weighted count while the
  goal asks it raised (in affine mode the weights are not negative). A
  revised simplex in floating point (dense basis inverse, sparse columns
  of the distinct effects) proposes the weights; `certify` checks them in
  exact `i128` arithmetic with checked operations, and only a vector it
  accepts refutes, as the new `Refutation::StateEquation`, which names the
  atoms' weights in text and JSON. The simplex runs beside the search,
  its work kept to a share of the search's (a fifth in linear mode, half
  in affine mode), its set-up waiting until the search has done as much,
  and it gives its memory back whenever the search runs out of room.
- **Two more refutations for linear mode**, which the review's random
  programs needed (below): transitions with an input place that no
  reachable marking can mark are dropped before every search and the
  equation (`live` in `mod.rs`), and once the forward search has kept
  2¹⁴ markings, the same search runs on the reversed net (`both` in
  `reach.rs`), a quarter of the work, with the places no transition
  raises capped at their initial count. Its exhaustion refutes; its
  firing sequence, read backward, proves.
- **The dispatch**: the Horn row takes Horn programs with a clause under
  `!` in every mode, affine included (`Modes::Any`).
- **The qcover suite**: the flake package `qcover`
  (`github:blondimi/qcover` at its last commit, fetched, not committed),
  the reader `linlog::mist::read` and `read_within` for Mist's `.spec`
  format (rules under `!`, the initial marking left of `⊢`, a parametric
  count `x >= k` as `k` tokens and `!x`, a target of several lines as
  clauses `!(line -o goal)` to a fresh atom), the command's
  `--input-format spec` (a `.spec` file by its extension), and the
  harness's source `--spec PATH`, which runs the problems intuitionistic
  affine.

### Why the new refutations are sound

- **Affine mode is coverability.** The first session's count holds with
  weakening: a weakened input keeps the number `q` of outputs on a
  sequent, a weakened output would lower it, and the root has `q = 1`,
  so no proof of such a goal weakens an output or uses Mix, and every
  sequent has one output. The induction of `core-horn.md` then reads a
  firing sequence of the *lossy* net off the proof, where a weakening
  drops tokens (a token, a clause used once, or the rest of a clause
  whose body was partly paid, which dropped those tokens). Dropping
  tokens never enables a firing, so a lossy sequence that covers the
  target gives a plain one that covers it. Conversely the proof the
  engine builds is that plain sequence with the leftovers weakened, which
  the checker judges. So the goal is provable in affine mode exactly when
  the target is coverable, clauses used once at most once.
- **The backward search ends, and its end refutes.** No element is kept
  that is at least an earlier one (the earlier, which stays in the index,
  would cover it), and by Dickson's lemma every infinite sequence of
  count vectors has an element at least an earlier one; so finitely many
  are kept. When the queue is empty, every kept element's predecessors
  were computed and covered, so the upward closure of the elements is
  the set of markings from which the target can be covered; every kept
  element was compared with the initial marking when computed, so the
  initial marking lies outside, and the target is not coverable.
  Elements superseded by a later, smaller one are skipped, not removed:
  `max(m − out, 0) + in` is monotone in `m`, so what they would compute
  the smaller one computes.
- **The state equation.** For a firing sequence from `M₀` to `M` with
  Parikh vector `x ≥ 0`, `M − M₀ = Σ xₜ Cₜ`. Integer weights with
  `y·Cₜ ≤ 0` for every transition and `y·(M − M₀) > 0` contradict it:
  `y·(M − M₀) = Σ xₜ (y·Cₜ) ≤ 0`. In affine mode a covering `M' ≥ M`
  with `y ≥ 0` gives `y·(M' − M₀) ≥ y·(M − M₀) > 0` against the same
  bound. `certify` checks exactly these inequalities over every
  transition the search may fire, in `i128` with checked operations (a
  weight below 2⁴⁰ after the conversion, at most 2⁶³ by type, times an
  arc's weight below 2³², fewer than 2³² terms, so no sum reaches 2¹²⁷).
  The simplex, the rounding of its weights to integers, its tolerance and
  its pivot limit decide only whether a refutation is found.
- **Dead transitions.** The places some reachable marking marks lie in
  the least set holding the initial marking's places (the tokens given
  and every class place of clauses used once) that holds the outputs of
  every transition whose inputs it holds all of: by induction on the
  firing sequence. A transition with an input outside it is never
  enabled, so dropping it changes no reachable marking; every search and
  the equation, and so `certify`, see only the others.
- **The backward search in linear mode.** The reversed program swaps
  every transition's inputs and outputs and the initial and target
  markings, so its firing sequences are the original's read backward.
  Every marking a firing sequence from `M₀` to the target passes is
  reachable from `M₀`, so a place that no transition raises holds at
  most its count in `M₀` there; dropping the backward search's markings
  above those caps drops none of them, and its exhaustion means no
  firing sequence reaches the target. Its finding `M₀` is such a sequence
  read backward, which the proof builder takes in forward order and the
  checker judges.

### The panels

Every refutation is a change of the search, so each went to a panel as
in step 26: a workflow of three fresh agents, each given the commit, the
trees before and after, release binaries of both, the claim and one way
to refute it, on cores 12 to 15 in capped scopes: the counterexamples
(Opus 5.5 at `high`), the argument (Opus 5.5 at `xhigh`), the integers
and limits (`sonnet`, which `.claude/settings.json` maps to Sonnet 5.5,
at `high`). Each counterexample agent wrote its own judge, sharing no
code with the engine: the step's third judge beside the qcover results
and the families.

**First panels: A on the state equation in linear mode (`f89bda18`), B
on coverability in affine mode (`79d2c482`), side by side. No wrong
verdict; claim A3 ("no decision lost") refuted, and fixed.**

- *A, counterexamples.* A judge in Python: exhaustive reachable markings
  (bounded at 4 000), backward coverability, and an exact phase-one
  simplex in fractions for the state equation; hand-checked on 15 nets
  (one of its own expected values was wrong, the judge right). 1 350
  programs, three forms each, five modes: 43 500 verdicts, none against
  the judge; about 2 600 state-equation refutations checked against the
  exact simplex, none on an equation with a solution; no counter of a
  proof changed. Refuted claim 3: the tableau, charged to the search's
  own account at the first pop, took a reachable goal to the memory
  limit (a net of five atoms at 2 000 bytes; an idle chain of 8 000
  clauses at the default GiB, proved in 5.4 s before).
- *A, the argument.* Its own invariant first (per atom, each literal of
  a non-weakened formula meets one dual, so the target less the initial
  marking is a combination of the clauses' effects with the uses as
  coefficients), then the code line by line: claims 1 and 2 held, 750
  random comparisons, no wrong refutation, no wrap. Refuted claim 3 by
  the same mechanism (a chain of 8 190 clauses proved in 25 ms before,
  "memory limit" now; a search exhausted before, unknown now) and found
  the set-up unbounded by the budget, a latent loop on a NaN weight,
  refutations missed where the weights span more than 2²⁰, a reason
  "memory limit after 2.01 s" that includes the simplex's time, and the
  command's help still saying "in linear mode".
- *A, the integers.* Every number argued or driven: no wrap in a copy
  with overflow checks, about 60 000 fuzzed instances, 55 000
  refutations checked by a bounded search; claim 3 refuted once more at
  30 and 40 MiB; the set-up quadratic in a clause's outputs (150 000
  outputs: 3.8 s against 0.11 s), not interruptible; the NaN loop; a
  comment overstating the integers' bound.
- *B, counterexamples.* A judge with a Karp–Miller tree, exhaustive
  markings and an exact simplex; 19 hand-checked nets; 3 000 programs in
  four forms, linear and affine: 22 000 forced runs, no disagreement;
  the 12 qcover files with a result all right. The affine counter is
  exponential for the backward search (128 tokens: 6 s; linear: 2.5 ms).
- *B, the argument.* Its own invariant before the rules file; 15 600
  programs against a reference of its own (backward basis with removal,
  no class merging), no mismatch; the focused engine agreed on all 1 185
  it decided. Two documentation findings: the statistics' labels in
  affine mode, and a redundant clause in the termination argument.
- *B, the integers.* The set-up's timeout overshoot (a clause of 100 000
  outputs: 2.6 s against 22 ms), the backward search quadratic in the
  elements sharing a place, the `.spec` reader writing out a count of 20
  million tokens before any limit (617 MiB from a 100-byte file), the
  NaN loop, and the proof's replay buffers left out of the memory
  account.

What became of their findings:

| finding | by | done |
|---|---|---|
| the tableau charged to the search's account cost decisions at the memory limit | A, all three | fixed (`381ec2bb`): the simplex gives its memory back whenever the search runs out of room, and before a proof is built |
| the set-up at the first pop whatever the budget, quadratic in a clause's outputs, not polled | A argument and integers, B integers | fixed (`746f9675`, `6a348864`): the set-up waits until the search has done three times the places squared plus the arcs, effects merged in one pass, a pivot only within the budget; the dense tableau gave way to a revised simplex (`23d6c9dc`) |
| a NaN or infinite weight loops the continued fraction | A argument and integers, B integers | fixed (`746f9675`): no certificate |
| the backward search quadratic in elements sharing a place | B integers | fixed (`216ab67c`, `04aedbcf`): a trie of the elements' places |
| the `.spec` reader writes out any count | B integers | fixed (`dc3d6bb2`): the tokens are summed and checked against the occurrence limit first |
| the proof's replay buffers not charged | B integers | fixed (`13630696`) |
| statistics' labels in affine mode; the command's help | B argument, A argument | fixed (`ceb2441f`) |
| a redundant clause of the termination argument; the integers' bound overstated | B argument, A integers | fixed (`ee618eb0`, `ac5b2abe`) |
| weights spanning more than 2²⁰ (in the ratio of the largest) are missed | A argument and integers, B integers | documented in `core-horn.md`; a follow-up |
| the affine counter is exponential for the backward search | B counterexamples | a follow-up (a forward search for a covering marking beside it) |
| "the memory limit after N s" includes the simplex's run after it | A argument | documented |

**Second panel, C, on what the first panels' findings and the
measurement made of the changes** (`f89bda18..3e2369ef`: the revised
simplex, the trie, memory yielding, the set-up's wait, dead transitions,
the backward search, the reader's bound), with the binary of `79d2c482`
and the base beside it. **No wrong verdict; the claim that no earlier
decision is lost refuted again, and fixed.**

- *Counterexamples.* A judge of its own: exact reachability (bounded at
  30 000 to 300 000 markings), Karp–Miller trees forward and on the
  reversed net, integer invariants modulo small `k`, a capped backward
  search and the rational state equation by Fourier–Motzkin; 16 nets
  checked by hand. 300 programs in ten configurations on three binaries,
  3 000 in seven, 1 500 growing ones in two on two binaries: no wrong
  verdict; the 338 linear verdicts its bounded searches could not judge
  were all confirmed by an invariant, its backward search or the
  equation. Proofs found by the backward search (counters with a growing
  distractor, at 29 480 markings against 64 963 before) were checked in
  three modes. Claim D refuted: at a bound of 4 096 bytes 35 proofs the
  search found were refused by the proof's charge (none at 64 KiB or 1
  MiB), and a parity net exhausted before ended at 40 MiB because the
  release stopped at the simplex while the backward search kept its
  markings.
- *The argument.* Its own invariants for every new path to a verdict,
  then the code line by line: dead transitions, `certify` over the
  transitions that can fire, the caps and the backward search's root,
  every `Exhausted`, the reversed sequence's order and the trie's query
  all hold. Claim D refuted three ways: a simplex whose basis did not
  fit beside the search was never tried after the search gave its
  memory back; the release stopped at the simplex; the trie looked up
  every remaining place at every node (`k²` on markings of `k` places: a
  goal decided in 0.25 s before ran past 10 s). Claim C, as worded, fails
  where dead transitions are dropped. The refutation's text ("no clause
  raises the weighted count") is false of a dropped clause; and
  `give_back` claimed memory from a backward search that never started.
- *The integers.* Every number argued; a copy with overflow checks ran
  the engine's tests, all 176 qcover files and 1 500 random nets in four
  modes at two bounds without a panic; stops within 1 to 6 ms of the
  limit. Claim D refuted by the proof's charge (a chain of 6 000
  clauses at 1 MiB) and by the trie's memory (30 to 75 % more than the
  scan; two qcover problems at 1 to 1.5 MB); a 6 KB `.spec` file of one
  long name and 49 million tokens killed at 8 GiB, the reader writing the
  name once per token; one rule of 60 000 updates read in 2 s.

| finding | by | done |
|---|---|---|
| the release gave back the simplex's tableau and stopped there | C argument, counterexamples, integers | fixed (`a182ad1f`): both give back |
| a simplex without room beside the search never tried again | C argument | fixed (`a182ad1f`): it waits for `finish` |
| the trie's query `k²` in the marking's places | C argument | fixed (`126fc0e5`): the smaller of the children and the places |
| the refutation's text names every clause, the check only those that can fire | C argument | fixed (`cd52211a`) |
| `give_back` on a backward search that never started | C argument | fixed (`a182ad1f`) |
| the proof's charge refuses proofs at tight bounds | C counterexamples, integers | the charge is the memory held now; the replay's second copy of its pairs is gone (`c4aee499`): a 10-bit counter needs under 262 KB of bound, against 303 KB with the copy and 197 KB under the first session's charge, which left the tokens out |
| the trie takes 30 to 75 % more memory than the scan | C integers | accepted: at the default bound no decision moves; the speed is the trie's point |
| a `.spec` name written once per token | C integers | fixed (`307d978e`): the sequent is built as an arena, a token one shared literal |
| a rule's updates read quadratically | C integers | fixed (`307d978e`): merged as sorted lists, 60 000 updates in 0.06 s |
| claims C and D as worded: counters move where dead transitions are dropped, and where the equation refutes later | C argument, counterexamples | the claims were too wide; the counters are equal on programs without dead transitions (1 978 of 1 978) and wherever the search decides alone (330 of 330) |
| the trie's table under-charged by about 14 % (hashbrown's buckets) | C integers | a follow-up |
| at 4 096 bytes the checker gives up checking a proof (an error, exit 2) | C counterexamples | unchanged since before the session; a follow-up |

### The measurement

Every run below ran detached, in capped scopes, four streams on cores 2
to 5 taking every fourth problem, on the session's last engines: qcover,
the library's nets and the LLTP batch on `307d978e`; the near-Horn
programs and the families on `6a348864`; the target set on `440a7907`.
The commits between them change speed and memory, not what is decided at
these bounds: the trie's walk (its answers are the same), the release of
both helpers and the simplex's retry (memory at the bound), the reader's
arena (the same formulas), the proof's charge (tight bounds) and the
place of the dead-transition pass (a few percent of time). The focused engines' rows come from
the session's first runs (`--engines horn,two-sided --bias factors
--copies 30` and `--engines auto --copies none`), since no commit of the
session touches them. The data are in `bench/defaults/` (below).

**The qcover suite** (176 problems, intuitionistic affine, 5 s):

| | Horn engine | two-sided, factors, 30 copies | the default before (two-sided, deepening) |
|---|--:|--:|--:|
| decided | **162** | 9 | 8 |
| provable (unsafe) | 59 | 7 | 7 |
| unprovable (safe) | 103 | 2 | 1 |
| unknown | 14 (time) | 167 (163 time, 4 copy bound) | 168 (time) |
| decided only by this one | 153 | 0 | 0 |
| verdicts against another | 0 | 0 | 0 |
| median time where both decide | 0.14 ms (9) | 54.7 ms | 60.0 ms (8) |
| decided within 10 ms / 100 ms / 1 s | 97 / 146 / 155 | | |

By suite, decided by the Horn engine: Mist 26 of 27, Soter 47 of 50,
Wahl–Kroening 46 of 46, bug tracking 32 of 41, medical 11 of 12. The 12
files that state a result are all answered as stated (11 safe, 1
unsafe), every proof checked. The literature counts 61 unsafe and 115
safe instances; the engine proves 59 and refutes 103. Of the 102
refutations of the run before the last, run again one by one, the backward search's exhaustion
gave 65 (all 32 of bug tracking, all 10 medical, 13 of Mist, 8 of Soter,
2 of Wahl–Kroening) and the state equation 37 (28 of Soter, 9 of Mist);
which of the two comes first is a matter of their shares of the time.
The 14 left: 9 bug-tracking
nets of 754 places and 27 370 rules, three Soter nets (the two `ring`
nets of tens of thousands of places, where the simplex's basis does not
fit, and `howait`), Mist's `extendedread-write` and one medical net; the
follow-ups in `plan/later.md` name what would reach them.

**Random programs near the Horn shape** (the review's design, a
generator of the session's: 7 000 programs of one to four atoms and one
to four clauses used once or under `!`, a marking, a goal reached by a
replayed firing sequence or random, mutations at the shape's edge; 1 s):

| | classical | Mix | intuitionistic | affine | intuitionistic affine |
|---|--:|--:|--:|--:|--:|
| programs the Horn engine takes (the row's in linear mode; every Horn program in affine mode) | 4 318 | 4 318 | 4 318 | 5 232 | 5 232 |
| decided by it | 4 303 | 4 303 | 4 303 | **5 232** | **5 232** |
| decided by the focused engine forced | 3 748 | 3 723 | 3 755 | 4 610 | 4 968 |
| the focused engine decides, the Horn engine not | **0** | **0** | **0** | 0 | 0 |
| the Horn engine decides, the focused engine not | 555 | 580 | 548 | 622 | 264 |
| verdicts against each other | 0 | 0 | 0 | 0 | 0 |
| median time where both decide | 0.033 ms | 0.032 ms | 0.033 ms | 0.031 ms | 0.031 ms |
| … the focused engine's | 0.109 ms | 0.127 ms | 0.107 ms | 0.038 ms | 0.039 ms |
| the Horn engine over a millisecond slower / the focused | 6 / 12 | 3 / 169 | 6 / 11 | 0 / 263 | 0 / 204 |

The first session's review had found 84, 80 and 53 programs the focused
engine refutes and the row left unknown. With the state equation alone
the session's first run of these programs left 10, 10 and 6 (the
equation is rationally feasible for them, and they grow without end);
dead transitions and the backward search refute every one of them, and
the final run leaves none.

**The library's 3 137 Petri nets** with the Horn engine at 5 s
(`bench/defaults/horn-nets-coverability.csv`, beside the first session's
`horn-nets.csv`):

| | first session | this session |
|---|--:|--:|
| decided (all proved) | 3 026 | **3 071** |
| unknown: time / memory limit | 90 / 21 | 60 / 6 |
| decided only by this one | 1 | 46 |
| decided by the forward focused search alone | 2 | 1 (`PaceMaker_5_1`) |
| verdicts against the forward search, proofs rejected | 0, 0 | 0, 0 |
| decided within 10 ms / 100 ms / 1 s | 2 670 / 2 965 / 3 014 | 2 635 / 2 994 / 3 064 |
| time of the 3 025 both decide, summed | 56.3 s | 43.8 s |

The 46 gained are mostly the backward search's (GPPP, BridgeAndVehicles,
DatabaseWithMutex, ResAllocation, Echo: a `BridgeAndVehicles` net of
717 390 markings forward is proved in 3 818); the one lost,
`PolyORBLF_PolyORB-LF-S02-J04-T06-unfolded_50_1`, was proved at 4.95 s
before and now runs out of time with the backward search's and the
equation's shares taken. Of the 3 025 both decide, 2 935 have equal
counters; the other 90 are the dropped dead transitions' new order among
successors at equal distance and the backward search's markings. The
times of two runs on different days do not compare (the medians, 0.50
and 0.62 ms, moved with the machine); paired on one core, 40 nets of 1
to 500 ms took 5.7 ms in the median before the session and 5.7 to 5.9
ms now, in 0.81 s and 0.50 s together (the pass for dead transitions
moved from every reading of a goal to the search, `440a7907`, after the
first pairing showed 4 to 6 % in the median).

**The families**, `linlog-bench run --all-families --timeout 5` with the
final engine: 62 proved, 38 refuted, 23 unknown, no mismatch, as in the
first session's review; the counter and the unreachable counter on the
Horn engine at every size in under 0.1 ms, Partition with clauses used
once still on the focused engine (forced on the Horn engine: 5 of 9
`partition-yes`, 5 of 7 `partition-no` at 10 s, against 7 and 5).

**The LLTP library** through the command's batch with its defaults (2 s,
four workers on cores 2 to 5): 3 753 proved, 142 refuted, 617 unknown of
4 512, against 3 701 and 142 at the first session's review; the Horn
engine proves 3 071 of them. Of the verdicts, 28 disagree with the
library's headers, all from the two-sided and net engines, which this
session did not touch (earlier reports traced such disagreements to the
headers, KLE013, KLE065 and SYN001 among them), none from the Horn
engine.

**The target set** (`bench/targets.sh after-coverability`, cores 2 and
3) against the first session's `after-horn.csv`: the same 265 runs on
the same engines; no verdict lost or contradicted, every proof checked.
The 118 rows the focused and net engines decide have every counter
equal. The Horn engine's 109 runs decide 107 where they decided 103
(`Echo_echo-d2r9_100_1`, `GPPP_G-PPP-100-1000_100_1`,
`Solitaire_soli2_counter_20_1`, `PermAdmissibility_unf-8x8-4stageSEN-05_100_1`
now within the limit), in 8.9 s of CPU against 27.8 s; 3 of the 103
reached their verdicts by other numbers of markings (`TokenRing-10`,
two `CloudReconfiguration` nets: the backward search's start).

### The decisions about the default (D19)

- **Affine mode moves to the Horn engine** (`0b77129b`): the row's modes
  are now `Modes::Any`. On qcover it decides 162 problems where the
  default before decided 8, with no verdict against it and faster where
  both decide; on the random affine programs it decides all 10 464, the
  focused engines 9 578, and is never more than a millisecond slower.
  The focused engines seldom refute an affine goal with `!` at all: their
  copy bound deepens without end.
- **The linear row keeps its scope.** The review's condition was that
  the row lose none of the focused engine's refutations on programs near
  the Horn shape, or narrow. On the session's 7 000 programs it loses
  none in any linear mode and gains 548 to 580 per mode; the library's
  nets keep their verdicts (below). The row still takes only programs
  with a clause under `!`: nothing in this session's numbers concerns
  programs without one, and the first session's Partition measurement
  stands.

### Decided unattended

- **Floating point proposes, integers decide.** The simplex runs in
  `f64` and its weights are rounded to integers and checked exactly; set
  aside: an exact rational simplex (big integers or `i128` fractions,
  slower and still needing a bound) and an LP crate (`microlp`, the
  maintained pure-Rust one, reads a clock and neither polls the caller's
  stop nor charges the memory account). The check is the only part soundness rests on.
- **A revised simplex with a dense basis inverse and sparse columns**,
  first the steepest column, Bland's rule after 64 pivots without
  progress. Set aside: the dense tableau (it was the first version; a
  pivot cost the rows times the transitions, 20 ms on the bug-tracking
  nets), a sparse LU (the right next step for nets of tens of thousands
  of places, a follow-up).
- **The simplex beside the search, by work, not before it**: most nets
  of practice are decided by the search in microseconds, and a set-up
  costs the places squared. Its shares (a fifth in linear mode, half in
  affine mode) were set on a sample of 50 library nets, where the first
  setting (half in both) made the median 1.6 times slower, and on the
  qcover problems that only the equation refutes.
- **The backward algorithm, smallest elements first, superseded elements
  skipped rather than removed**, indexed by a trie of places. Set aside:
  a scan by first place (the first version, quadratic where elements
  share a control place), removal of superseded elements (more
  bookkeeping, no change in what is computed), Karp–Miller trees (they
  decide coverability too, but blow up on the nets of verification, and
  give no refutation that a checker of this kind reads).
- **Dead transitions and a backward reachability search** for the
  linear row's refutations, rather than narrowing the row: both are
  standard, both are cheap where they do not help (a pass over the arcs;
  a search started only after 2¹⁴ markings), and together with the
  equation they refute every program the review and this session found
  the row losing. Set aside: trap constraints and the integer state
  equation (stronger, but no program here needed them; follow-ups), and
  narrowing the row by a feature (none separates the lost programs).
- **The `.spec` reader builds the sequent directly** (an arena, every
  token of a counter one shared literal), first by text for the parser,
  until a panel showed a short file asking for gigabytes. A target of
  several lines is a fresh atom `goal` with a clause per line, which in
  affine mode is covered exactly when a line is; set aside: one problem
  per line (the instance's verdict would be a disjunction the harness
  does not know) and `⊕` (no Horn program).
- **`.spec` problems run intuitionistic affine** in the harness, as the
  LLTP nets run intuitionistically; the command leaves the mode to the
  flags, as for LLTP files (README shows `-i --affine`).
- **`Refutation::StateEquation` names the atoms' weights** and says
  whether clauses used once weigh something, in text and JSON; the
  weights of the class places are not named (a class has no name of its
  own). Set aside: reporting the equation's refutation as `Exhausted`
  (untrue) and naming each class by its first clause (long, and the
  weights of the atoms are what a reader checks).
- **Tests**: the engine's tests pin the refutations of unbounded nets in
  every mode, the exactness of `certify`, every new limit as a refusal
  (the backward search's counts, its elements, the token count of
  `.spec` files), and the reference comparison runs every engine on the
  random Horn programs in affine mode too. The limit tests that pinned
  the memory bound use nets that nothing refutes (a parity net that
  grows both ways), since the equation, dead transitions or the
  backward search now refute the old ones.

### Options, front ends and quantifiers

No option was added. The engine's new parts are its scheduling, as the
first session's search order is: they change how soon a verdict comes,
never which, and the shares and thresholds are named constants with the
reasons beside them (`LINEAR_ENTRIES_PER_UNIT`, `AFFINE_ENTRIES_PER_UNIT`,
`BACKWARD_AFTER`, `BACKWARD_SHARE`, `STALLED`). The bounds are the
existing `Options::memory_limit` and the caller's stop. The `.spec`
format is a value of the command's `--input-format` (and a `.spec` file's
extension), and the library's `mist::read_within` takes the occurrence
limit that `--occurrence-limit` sets; a web front end would read a
`.spec` text through the same function and choose affine mode in the
options value it already holds. The harness's `--spec PATH` is a source
beside `--lltp`. The new refutation is part of `Outcome`'s JSON.

Quantifiers (D17): unchanged from the first session; coverability and
the state equation are propositional, and first-order Horn clauses would
be a row of their own.

### Deviations and assumptions

- **"Expected results in the files"** holds for 12 of the 176 (11 safe,
  1 unsafe), as the first session found; the other judges of the
  refutations were the panels' independent checks (two Karp–Miller
  trees, three exhaustive searches, three exact simplexes in fractions,
  each agent's own), the families with known verdicts, and the focused
  engines where they decide.
- **"The near-Horn programs of the review"**: the review's own 7 000
  programs are not in the repository; the session generated 7 000 by the
  review's description (`plan/reports/27-horn.md`, "From the review of
  the first session") with a script of its own, kept outside the
  repository. Its numbers are in the same range as the review's (84,
  80, 53 lost before the state equation; this generator's first run, with
  the equation, 10, 10, 6).
- **The panels ran twice**, as the first session's did: once on the two
  changes the step names (the state equation, coverability), and once on
  what their findings and the measurement made of them (the revised
  simplex, the trie, memory yielding, dead transitions, the backward
  search, the reader's bound).
- **Two more refutations than the step names** (dead transitions, the
  backward search): the review asked that the row lose none of the
  focused engine's refutations, and the state equation alone left 26.
- **The proof's memory charge rose** (`13630696`): the replay's second
  copy of its pairs, its tokens and the clauses fired were not counted,
  so at a tight bound a goal the search solves may now end at the bound
  in the proof, where before it held more than the bound allowed (a
  chain of 6 000 clauses at 1 MiB; the second panel's integers agent
  measured the boundary at 5 577 clauses against 8 737). The bound is
  now the memory held.
- **The trie costs memory**: 30 to 75 % more than the scan by first
  place on qcover's nets, so at a bound of 1 to 1.5 MB two problems that
  the first panel's binary decided end at the bound (the second panel's
  integers agent); at the default bound none.
- **Counters move where dead transitions are dropped**: the order among
  successors at the same distance follows the transitions' indices, so
  90 of the library's nets reached their (equal) verdicts by other
  numbers of markings; in affine mode the dropped transitions add no
  backward elements.
- **`Refutation::StateEquation` is a public variant**, beside the counts'
  refutations; `Refutation`'s documentation now lists every JSON form.
- **`plan/README.md` is unchanged**, as in the first session: D8's note
  still says the Horn row is linear mode only, which `Engine`'s
  documentation and `core-search.md` now correct; the planning session
  updates the plan.

### What is left open

In `plan/later.md`, "Follow-ups: the Horn engine":

- **14 qcover problems at 5 s**: qcover's own algorithm prunes every
  backward element whose rational state equation from the initial
  marking has no solution (an LP per element), and a forward search for
  a covering marking would prove what the backward search reaches
  slowly (the affine counter of 128 tokens: 6 s).
- **Refutations no part reaches**: nets growing both ways whose equation
  has a rational solution and whose goal fails for a reason of integers
  or order (the parity net of `refuses_at_its_limits`); the integer
  state equation and trap constraints would.
- **Certificates whose weights span more than 2²⁰**, which the
  floating-point vertex loses; and a sparse basis for nets of tens of
  thousands of places.
- **The shares and thresholds** (`LINEAR_ENTRIES_PER_UNIT`,
  `AFFINE_ENTRIES_PER_UNIT`, `BACKWARD_AFTER`, `BACKWARD_SHARE`) were set
  on a sample by day; the next baseline should look at them on an idle
  machine.
- **Memory at tight bounds**: the trie's extra memory, its table's
  charge, and the checker that gives up at a few kilobytes.
- **The 66 library nets** left at 5 s, as the first session listed them.

### The commits

In order; the first 26 are signed, the rest unsigned (the passphrase's
cache ran out at 11:02):

1. `9028f7ff` Fetch the qcover coverability suite in the flake.
2. `849ddad5` Read coverability problems in Mist's .spec format (the
   reader, the command's `--input-format spec`, the harness's `--spec`).
3. `f89bda18` Refute Horn programs whose state equation has no solution.
4. `79d2c482` Decide Horn programs in affine mode by backward
   coverability.
5. `77f9ac25` Keep a .spec problem's rules sparse (the first qcover run
   met 114 GB on the largest file).
6. `3d2680c7` Read a .spec line that starts with a comma as the previous
   line's continuation (41 bug-tracking files).
7. `ceb2441f` Name the Horn engine's backward counts in the command's
   statistics.
8. `23d6c9dc` Solve the state equation by a revised simplex.
9. `216ab67c` Index the backward search's elements by a trie of their
   places.
10. `04aedbcf` Count the trie's lookups in the backward search's work.
11. `746f9675` Set up the state equation only once the search has done
    as much (panel A, B).
12. `dc3d6bb2` Refuse a .spec problem whose tokens pass the occurrence
    limit before writing it out (panel B).
13. `13630696` Charge the Horn proof's replay buffers to the memory
    account (panel B).
14. `ee618eb0` Say why the backward search ends without the removal it
    does not do (panel B).
15. `381ec2bb` Give the Horn search the simplex's memory whenever it runs
    out of room (panel A).
16. `3e2369ef` Refute what the state equation leaves: drop dead
    transitions, and search backward beside the forward search.
17. `ac5b2abe` Say what bounds the state equation's integer weights
    (panel A).
18. `0b77129b` Send Horn programs with a clause under ! to the Horn
    engine in affine mode too.
19. `6a348864` Keep the simplex and the backward search out of the way of
    nets decided quickly.
20. `0d276790` Say in README that affine Horn programs no longer meet the
    focused engines.
21. `b39dbd90` List coverability and the state equation in the rules
    index.
22. `d06cc806` Name the qcover package among CLAUDE.md's commands.
23. `357b0245` Describe every refutation and its JSON form on Refutation.
24. `126fc0e5` Walk the backward search's trie by a node's children or
    the marking's places, whichever are fewer (panel C).
25. `a182ad1f` Free both helpers' memory when the Horn search runs out of
    room, and let a simplex without room try after the search (panel C).
26. `cd52211a` Say that the state equation's weights count only the
    clauses that can fire (panel C; with the engine table's qcover
    numbers).
27. `307d978e` Build a .spec problem's sequent as an arena, every token
    of a counter one shared literal (panel C; **unsigned**).
28. `c4aee499` Let the Horn proof read each firing's pairs where the
    replay wrote them (panel C; **unsigned**).
29. `440a7907` Drop dead transitions in the search only, not in every
    reading of a goal (**unsigned**).
30. `9f1b2ec2` Record the Horn engine's measurements (**unsigned**).
31. This report and the follow-ups in `plan/later.md` (**unsigned**).

### From the review of the second session

Accepted without a fix to the code.

- **Signed**: the five commits made after the passphrase's cache ran
  out, at the review with the author present.
- **Checked**: clippy, the tests (and `cargo test -p linlog`), both
  `cargo hack` runs, `cargo deny`, `nix flake check`.
- **The target set again** (cores 2 and 3): every row, verdict and
  counter equal to `after-coverability.csv`, the Horn rows included.
- **The families**: no mismatch (62 proved, 38 refuted, 23 unknown).
- **qcover, run again** (`-i -a`, 5 s, two of the slower cores): 59
  proved, 102 refuted, 15 unknown; the 12 files that state a result all
  answered as stated; the proofs within the literature's 61 unsafe, the
  refutations within its 115 safe. The largest file (18 MB, 914 053
  lines) is read and searched within the 2 s limit at 250 MB, and
  printed in 0.34 s.
- **The LLTP library** by default on four cores: 3 750 proved and 142
  refuted (the session's 3 753 and 142), no contradiction, no net
  refuted; against the first session's review 52 decided more and 3
  fewer, `SYJ` problems of the two-sided engine at the 2 s limit.
- **The review's own 7 000 programs near the Horn shape**, the very
  sequents of the first review, on the new binary against the focused
  engines forced, at 1 s: no contradiction and no error in any of five
  modes. In linear mode the Horn engine now decides 267, 272 and 298
  that the focused engine does not, and loses 1, 1 and 0 where it lost
  84, 80 and 53; in affine mode it decides all 3 751 the row takes,
  1 383 and 762 more than the focused engine, and loses none.
- **The one loss is the follow-up the report names**:
  `!((c * d) -o (d * 1)), !(a -o 1), (a -o (b * d)), !((d * d) -o 1),
  !a |- b`, intuitionistic or classical. It is unreachable by parity (once
  the clause used once fires, `d` stays odd: no clause adds a `d`, only
  pairs are removed, and the clause that takes a single `d` needs a `c`
  that never comes), the rational state equation has a solution (half a
  firing), and `!a` makes the markings grow, so neither search ends;
  the focused engine refutes it, the Horn engine meets the memory bound
  at 10 s. The session's own generator met no such program, so its
  "loses none" holds for its programs; for the review's it is 1 of
  3 751 against 267 gained. The integer state equation (modulo small
  `k`, as panel C's judge used) is the follow-up that refutes it.
- **Prompt 28 is finished** (the items steps 22 to 27 left it, the qcover
  run among its checks), and D8's note says the Horn row takes every
  mode.

## The first session

The first session's report as it was written, with the review that
followed it.

### Outcome of the first session

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

### The engine

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

### Why a refutation is sound

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

### The panels

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

### The measurement

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

### The decision about the default

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

### Decided unattended

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

### Options, front ends and quantifiers

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

### Deviations and assumptions

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

### For the second session

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

### The commits

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

### From the review of the first session

Accepted without a fix to the code; the second session gets one more
requirement (prompt 27, "From the review of the first session").

- **Signed**: the three commits made after the passphrase's cache ran out
  were signed at the review, with the author present.
- **Checked**: clippy, the tests (and `cargo test -p linlog`), both
  `cargo hack` runs, `cargo deny`, `nix flake check`. The panels ran
  Sonnet 5.5 as their third member (226 requests with its id, no
  catalog warning).
- **The target set again**, by the review (cores 2 and 3): every counter
  and verdict equal to `after-horn.csv`; against `after-panels.csv` the
  109 runs that changed engine and nothing else.
- **The families**: no mismatch (62 proved, 38 refuted, 23 unknown, 12
  instances on the Horn engine).
- **The LLTP library** through the batch by default on the slowest four
  cores: 3 701 proved and 142 refuted, against 2 194 and 142 at step
  26's review; 1 508 decided only now, one only then (`SYJ201+1.005` in
  `cbn`, on the two-sided engine, at the time limit); no contradiction,
  no net refuted; 6.3 minutes where it took about 20.
- **The bounds**: on `BART-030-unf_50_1` the time limit holds at 2.00 s;
  on `DNAwalker_dnawalk-18_20_1` and `DatabaseWithMutex_database40UNFOLD_50_1`
  the memory bound of 1 GiB holds (0.6 and 0.84 GB resident), and at
  4 GiB, 2.4 GB.
- **The refutations at the shape's edge**: 7 000 random programs near
  the Horn shape (one to four atoms and clauses, used once and under
  `!`, goals reached by a replayed firing sequence or random, mutations
  with `&`, `⊕`, `!` in a body or head or goal, `1`, `!a` beside them,
  nested `!`, clauses equal on both sides), at 1 s, by default and with
  the focused engine forced, intuitionistic, classical and with Mix: no
  contradiction and no error in 42 000 runs. But of the 3 751 the row
  sends to the Horn engine, 84, 80 and 53 that the focused engine
  refutes stay unknown, against 1 or 2 gained: nets whose markings grow
  without end while the goal is unreachable (`!d, !((c * d * d) -o 1)
  |- (c * c)`, where nothing makes `c`). The library cannot show it,
  since every net there is reachable. The second session closes it, the
  state equation with a Farkas certificate checked exactly being the
  canonical way, or the row narrows.

## Unsigned commits

The commits after the passphrase's cache ran out (at 11:02, the 27th to
the 31st above: `307d978e`, `c4aee499`, `440a7907`, `9f1b2ec2` and this
report's) are unsigned; once the passphrase is entered, one command signs
them all:

```sh
jj sign -r 'main@origin..@-'
```
