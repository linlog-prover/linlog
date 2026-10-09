# Quantifier spike (draft C, section 11): report

Workspace `linlog-spike`, commits `Spike M1` (okzkzopw), `M2` (pokkvxyu), `M3` (uypxxowl);
`@` holds only the M2/M3 target CSVs. Base: `base-journeys.txt`, `spike-base.csv`. Builds and
tests in the capped scope on cores 12-15; journeys and target sets as the brief says; no G3.

## Verdicts in one table (G2: callgrind instructions, base and each milestone)

(s) marks a search journey (gate +1.0 % each, +0.5 % for their sum); ✗ marks a failed gate.

| journey | base | M1 | Δ M1 | M2 | Δ M2 | M3 | Δ M3 | M2−M1 | M3−M2 |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| search-qbf-20-2 (s) | 661558506 | 656399633 | -0.78 % | 656613066 | -0.75 % | 672338979 | +1.63 % ✗ | +0.03 % | +2.40 % |
| search-chain-128 (s) | 1216594953 | 1297286184 | +6.63 % ✗ | 1275692253 | +4.86 % ✗ | 1273590732 | +4.68 % ✗ | -1.66 % | -0.16 % |
| search-partition-no-5 (s) | 8523805 | 8648284 | +1.46 % ✗ | 8655268 | +1.54 % ✗ | 8659644 | +1.59 % ✗ | +0.08 % | +0.05 % |
| search-wide-m2-256 (s) | 274070486 | 267771715 | -2.30 % | 267771718 | -2.30 % | 267771693 | -2.30 % | +0.00 % | -0.00 % |
| search-spec-chain (s) | 40115539 | 40258292 | +0.36 % | 40284660 | +0.42 % | 40284644 | +0.42 % | +0.07 % | -0.00 % |
| search-chain-64-intuitionistic (s) | 154301282 | 164390162 | +6.54 % ✗ | 161742211 | +4.82 % ✗ | 161338146 | +4.56 % ✗ | -1.61 % | -0.25 % |
| search-additive-14 (s) | 608740834 | 600404286 | -1.37 % | 600404286 | -1.37 % | 600404285 | -1.37 % | +0.00 % | -0.00 % |
| read-text | 18548521 | 18894866 | +1.87 % | 18962472 | +2.23 % ✗ | 18898800 | +1.89 % | +0.36 % | -0.34 % |
| read-json | 12515726 | 12280396 | -1.88 % | 12280396 | -1.88 % | 12272204 | -1.95 % | +0.00 % | -0.07 % |
| read-lltp | 32976872 | 33373662 | +1.20 % | 33437684 | +1.40 % | 33349063 | +1.13 % | +0.19 % | -0.27 % |
| read-tptp | 30658823 | 31121760 | +1.51 % | 31122998 | +1.51 % | 31122677 | +1.51 % | +0.00 % | -0.00 % |
| read-spec | 22457461 | 22706161 | +1.11 % | 22707439 | +1.11 % | 22674247 | +0.97 % | +0.01 % | -0.15 % |
| check-qbf-20-2 | 135217544 | 135339839 | +0.09 % | 135341883 | +0.09 % | 135341894 | +0.09 % | +0.00 % | +0.00 % |
| check-wide-m1-2048 | 8772182 | 8792825 | +0.24 % | 8792825 | +0.24 % | 8792836 | +0.24 % | +0.00 % | +0.00 % |
| derivation-chain-64 | 17005390 | 16946345 | -0.35 % | 16946345 | -0.35 % | 17032821 | +0.16 % | +0.00 % | +0.51 % |
| render-latex | 6585706 | 6566471 | -0.29 % | 6566471 | -0.29 % | 6566471 | -0.29 % | +0.00 % | +0.00 % |
| render-typst | 5779945 | 5761836 | -0.31 % | 5761838 | -0.31 % | 5761838 | -0.31 % | +0.00 % | +0.00 % |
| render-svg | 86058037 | 86134480 | +0.09 % | 86134480 | +0.09 % | 86176727 | +0.14 % | +0.00 % | +0.05 % |
| batch-families (s) | 81563099 | 81773964 | +0.26 % | 81686566 | +0.15 % | 81701196 | +0.17 % | -0.11 % | +0.02 % |
| ordinary-pigeons (s) | 39297813 | 39274059 | -0.06 % | 39269293 | -0.07 % | 39229361 | -0.17 % | -0.01 % | -0.10 % |
| sum of (s) | 3084766317 | 3156206579 | +2.32 % ✗ | 3132119321 | +1.54 % ✗ | 3145318680 | +1.96 % ✗ | -0.76 % | +0.42 % |

G1 (counters `verdict nodes splits memo_hits memo_entries links tests`, `cmp.py`): every
milestone 265/265 rows, 225 decided, **0 differences** against `spike-base.csv` and against
the committed `after-coverability.csv`.

## M1, the data model: fails G2 (largest: search-chain-128 +6.63 %)

Built (44 code files, +766 −239): 11.3's M1 in full. `Term::{Pred, DualPred, Forall,
Exists}` and `Kind::{Forall, Exists}` appended (12 and 1 bytes asserted, `Kind`'s
discriminants kept), every match extended; `sequents::fo` and 3.9's six empty tables in
`Sequent`; `Fragment::QUANTIFIERS`; `Member` as every `Node` operand (16 bytes with `Cut`,
`Forall`, `Exists`) and every member list's element (`Inference::sequent`, `Dyadic`,
`Interactive::goal`, `prove_goal`/`engine_for`, converted O(goal) at the front door;
`prove`/`prove_until` keep an internal path with no conversion); `Proof` with an empty
`Instances` table and the checker's per-proof branch, new nodes refused (`Problem::FirstOrder`,
oracle too); every `admits` refuses the bit, the dispatch answers `NoEngine`.

Choices where the draft was open: the tables one inline field (same layout, not boxed); the
`Term` variants appended after `Quest` (tags 12-15), the choice that cost; `Member::ground()`
a crate-private identity, no bound check per node; `Problem`'s single occurrences stay
`OccId`; `PartialEq` between `Member` and `OccId`; no `ordinary::Node` variants (not in
11.3); writers `unreachable!` on the new variants, printers write the predicate's name and
`∀`/`∃`; the checker's refusal of a forest with arguments left to M3.

Tests: `cargo test -p linlog` all pass (177 unit, 2 depth, 13 export, 1 lock, 11 parse, 11
serialize, 24 doc), `linlog-cli --test lock` passes, clippy `--deny warnings` clean. Lock files
untouched; test edits are type changes only (`OccId::new` → `Member::new` where a node or goal
is built; one `assert_eq!(…, [])` → `is_empty()`, ambiguous after `PartialEq<Member>`).

Cause, from disassembly of the target sets' binaries (static, no run): `Forest::atom` is
branch-free in the base (`cmpl $0x2,(tag); setb`) and in M1 becomes `cmp $0xd; ja; mov
$0x3003; bt; jae` plus three more, because `Term::atom()` now matches tags {0,1,12,13}. The
focused engine calls it per literal in its hot loops (`meets` walks every literal under every
`Θ` formula per stable sequent, which the `chain` family's copies make the bulk of its work;
also `mark_literals`, `initial`, `Counts::positive`); the functions inlining it grew (`initial`
0x977→0xb89, `mark_literals` 0x31c→0x363, `Counts::positive` 0xae→0xdc). The net and additive
journeys got cheaper (−2.30 %, −1.37 %), the checker's branch (+0.09 %, +0.24 %) and `Member`
(derivation −0.35 %) cost nothing measurable.

Under 11.5: **M1 fails G2 → bisect; the evidence points at the variants part (Term's tag
layout)**, not the `Sequent` fields, `Member` or the checker's branch. Proposed redesign: the
four literal variants first in `Term` (`Var, DualVar, Pred, DualPred`, then the rest), so
`atom()` stays one comparison (`tag < 4`); `Kind` keeps its discriminants, so `Term::kind`
stops being the identity on the tag, which only the forest's build and the readers pay.
**Not measured**: the redesign needs one more journeys run (an "M1b"), which the brief does not
name.

## M2, the zone parameter: own delta passes (largest +0.36 %, read-text); against the base fails by M1's part (chain-128 +4.86 %)

Built (11 code files, +839 −402): `search/zone/` (`Context`, `Classes` moved; traits `Zone`,
`Linear<M>`, `Unrestricted<M>`; `Ground`), `Engine<'a, Z>` and every engine type, `split`,
`scratch`, `schedule`, `parallel` generic; the front door calls `search_goal::<Ground>` only.
`Ground` holds the literal marks moved out of the engine, its methods `#[inline]` forwards of
the old code, `Mark = ()`. `mark`/`undo` at all seven places of 10.5 (e).

Where the drafted trait did not fit, and what I did: (1) `Ground` cannot hold `&Forest`: pool
workers are engines of a shorter lifetime than their spawner, so the trait is lifetime-free and
the forest is passed to each seam call; (2) the initial rules read a dual list's mark and its
"looked up in Θ" flag, which `dual_in`/`mark_literals`/`meets` do not expose: added an
associated `Duals` handle with `duals`, `marked`, `first_lookup`, so the atom and sign are
derived once as before (a first version re-derived them three times); (3) `dual_from` with its
cursors joined the seam; (4) `mark_literals` returns the bytes it allocated, which the engine
charges, so the memory account is as before; (5) `new`, `fork` (a worker's zone from its
spawner's) and `root` (a goal occurrence as a member) added; (6) `class` returns a member, the
member itself when it has no class, instead of `Option<OccId>`, which keeps `one_of_each`'s
dedup as it is; (7) **the memo and the loop check stay non-generic**: they key the ground part
(`Linear::ground() -> &Context`, `Unrestricted::ground() -> &OccSet`), gated by
`Zone::memoizes` (10.5 (f): ground sequents only), so no `Key<Z>`, and the memo's word records
are untouched; (8) `Problem` stays non-generic, the zone is handed to `Engine::new`;
`forced_side` and the counts read occurrences through `Zone::occurrence`.

Tests: as M1, all pass; `--features parallel` tests of `search::focus::parallel` (7) pass;
clippy clean with and without `parallel`.

Under 11.5: **"M2 passes"** on its own (sum of search journeys −0.76 % against M1, every
journey within +0.36 %); genericity with one instance is free, even slightly cheaper on `chain`.

## M3, the second instance: fails on its own delta (search-qbf-20-2 +2.40 % against M2)

Run because M2 left exactly 11.5's open row. Built (7 files, +479 −6): `zone/framed.rs` as
11.3 says (an `Instances` table, a `Vec<u32>` trail, `Context`/`OccSet` plus sorted framed
members, duals by predicate and equal `ArgsId`, no cursors, memo for ground members only);
`Focused::decide` sends a goal with the bit to `search_goal::<Framed>` (sequential); the
one-sided `admits` and the classical dispatch row (`LL ∪ QUANTIFIERS`) take a binder-free
goal; the checker refuses a first-order forest, as a refusal. A unit test proves `⊢ p(c),
~p(c)` and `⊢ p(c) ⊗ q(d), ~p(c), ~q(d)` through `Framed`, refutes `⊢ p(c), ~p(d)`, and finds
`engine_for` → `Focus`. Tests as M2, all pass (178 unit).

Cause, statically (M2 and M3 symbols outside `Framed`): with a second instance calling the same
non-generic helpers, LLVM stopped inlining several into `Ground`'s engine: `Context::iter` (an
out-of-line `FlatMap`), `Key::assign`, `OccSet::clone`, `Split::clone_from`,
`Forest::children` became calls; `Engine<Ground>::alternative` shrank 5361→3493 bytes.
Text (reported, not gated): base 2 338 670 B (focus+zone symbols 187 422), M1 2 360 222
(189 871), M2 2 361 022 (189 726), M3 2 505 006 (333 378, of which 140 398 name `Framed`).

Under 11.5: **"M2 passes, M3 fails"**. The table's remedy (`#[inline(never)]` on the second
instance's entry, cold paths outlined) does not address this mechanism: the cost is shared
helpers outlined, so the remedy to try is `#[inline]` on the hot helpers both instances call
(`Context::iter`/`insert`/`remove`/`Hash`, `OccSet::clone`, `Key::assign`, `Zones::hash`), or a
`Framed` that does not share them. Not measured (a run the brief does not name).

## Indicative times (target set, `cpu_ms` of run 0; G3 not run)

Rows of ≥ 300 ms in the base (resolution 10 ms): mix/8 1060 / 1070 / 1060 / 1060; mix/9 9220
/ 9320 / 9130 / 9150; mix/10 80190 / 80190 / 80260 / 80420; chain/256 320 / 320 / 310 / 310
(base / M1 / M2 / M3). Rows over 1 s (`cmp.py`): 90470 → 90580 (+0.12 %), 90450 (−0.02 %),
90630 (+0.18 %). Load (`uptime`, start → end): M1 0.31 → 1.24, M2 0.95 → 1.95, M3 0.99 → 1.04;
during M1's and M2's runs my builds and tests ran on cores 12-15. The `chain` rows the journeys
flag are too short here to show a few percent.

## What the design should say

1. **Term's tag layout is part of the hot path**: the literal variants contiguous at the
   start. The journeys caught it; the target set's `cpu_ms` cannot, nor a size assertion.
2. **The literal seam is wider than drafted**: a dual-list handle (mark and Θ-lookup flag),
   `dual_from` with cursors, a charge for the marks, `fork` for pool workers, `root`, and a
   forest passed per call (no lifetime on the trait).
3. **The memo and loop check stay ground**: key `Linear::ground()`/`Unrestricted::ground()`
   behind `Zone::memoizes`; drop `Key<Z>` from 11.3.
4. **One zone instance is free; a second costs through shared helpers**, not through its own
   code. 11.5's remedy row should name inlining of shared hot helpers.
5. Smaller: `admits`'s `has_binders` scan (O(terms) per classical prove in M3) should run only
   when the bit is set; the checker's refusal of a first-order forest belongs with the first
   first-order search, as a refusal.

Not run: G3; an M1b with the literal variants first; M3 with the helpers `#[inline]`; the
parallel tests of the whole crate (only `search::focus::parallel`'s); `cargo test --workspace`,
`cargo hack`, `nix flake check`, as the brief says.

## After the pause (written by the session that ran the spike)

The agent built and committed M1d, M2d and M1b and measured their
journeys and the target sets of M1d and M2d before the pause; it had
prepared M3i's tree. The session then committed M3i, ran its tests (core
178, the lock, the parallel tests: all pass), resumed M1b's target set
with M1b's own binary, and measured M3i's journeys and target set, and a
count of `mix/8` under callgrind for the base, M2d and M3i. The numbers
and what they decide are in `plan/notes/api.md`, section 11.5:

| milestone | G1 | worst search journey | search sum | worst other |
|---|---|---|--:|---|
| M1d | exact | `batch-families` +0.03 % | −0.94 % | `read-tptp` +1.45 % |
| M2d | exact | `search-partition-no-5` +0.23 % | −1.33 % | `read-tptp` +1.61 % |
| M1b | exact | `search-partition-no-5` +0.16 % | −0.75 % | `read-text` +2.00 % |
| M3i | exact (223) | `search-qbf-20-2` +0.94 % | −0.53 % | `read-tptp` +1.54 % |

`mix/8` under callgrind: base 29 542 018 712, M2d 29 554 079 268, M3i
29 438 722 626 instructions, equal counters.

The spike's commits were abandoned and its workspace forgotten and
removed once these numbers were recorded (jj's operation log keeps them):
M1 aa32c537, M2 ff095ff9, M3 bfeb4da5, M1d bac29a3e, M2d cddad557, M1b
38107ce8, M3i cacd233e.
