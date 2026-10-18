# Step 32: the web front end

## The checklist

| item | state | evidence |
|---|---|---|
| Session 32a: the plan and the bindings (session `step-32a`, 2026-10-19) | done (2026-10-19) | `plan/web/README.md`; the crate `linlog-web` and its flake checks (`zvxnzksq`, `ovuwwmtm`); the 32-bit arguments (`quznwnno`); item 3: no suspendable search needed; the reviewer's six findings answered; every code commit gated; `nix flake check` at the last commit |
| Session 32b: the client's repository and version 1's core | not started | |
| Session 32c: hosting, licences and the rest of version 1 | not started | |

## Session 32a: the plan and the bindings

**Outcome.** The web front end has its plan, `plan/web/README.md`: the
three sessions and which of the design's decisions each builds, version
1 and what it leaves out, the bindings' API call by call, the tab's
defaults, and, item by item of `plan/32-web.md`'s "What the bindings
must offer the page", what is offered and what the library lacks. The
crate `linlog-web` (`web/`, `publish = false`) is the library compiled
for `wasm32-unknown-unknown` without `parallel` behind nineteen calls of
JSON text in and JSON, SVG or bytes out over a session, the settings and
a search's outcome. The flake builds it as the package `linlog-web` (the
module and its JavaScript from the `wasm-bindgen` CLI of the version
`Cargo.lock` pins, `wasm-opt -O3`, `LICENSE` and cargo-about's
`THIRD-PARTY-NOTICES.txt`; 5.2 MB of wasm, 2.0 MB compressed) and checks
it: `web-package`, `web-wasm-tests` (its twelve tests under Node),
`web-core-wasm32` (the library's own tests of the checker, the size
estimate, the memo and the memory account on a 32-bit target) and
`web-bindgen` (the CLI's version against the lock's). Item 3's answer:
the two searches of the default bias taking turns cost a median of 1.04
and at most 4.98 times the better search's work on the 48 rows either
decides, so no search that can be suspended is needed.

### The checklist

| item | state | evidence |
|---|---|---|
| `plan/web/README.md`: sessions, decisions per session, version 1, the API, what the library lacks | done | the file, committed with this report |
| the crate `linlog-web`, `publish = false`, JSON in, JSON or SVG out over the session, the settings and an outcome | done | `zvxnzksq` |
| built with the `wasm-bindgen` CLI at the lock's version (a check comparing them) and `wasm-opt` | done | `modules/web.nix`: the CLI picked as `wasm-bindgen-cli_0_2_127` from the lock's version, `web-bindgen`; `nix build .#linlog-web` |
| defaults that fit a tab: a work budget, memory and occurrence bounds set by a tab's memory, a smaller derivation bound | done | `web/src/tab.rs`: 2²⁶ units, 96 MiB, 1 000 000 occurrences, 4 MiB |
| the stack raised or the recursion limit lowered, with a test at the limit under wasm | done, both | an 8 MiB module stack (`web/build.rs`); the limit lowered to 384 for the host's stack; `a_search_at_the_recursion_limit_stops_there` under Node |
| the tests under wasm32, among them the checker's and the memo's at their limits | done | `web-wasm-tests`; `web-core-wasm32` (22 tests on wasm32-wasip1) |
| a sentence at each integer argument made for 64 bits | done | `quznwnno` |
| item 3 under wasm on the target set's small rows, and the answer on a suspendable search | done | below; `plan/notes/web-turns/` |
| the bindings' package and wasm tests as flake checks; `nix flake check` at the last commit | done; the last run in the message to `planning` | `web-package`, `web-wasm-tests`, `web-core-wasm32`, `web-bindgen` |
| the fresh-context reviewer on the JSON boundary and the 32-bit arguments | done, six findings, each answered | below; `ovuwwmtm` |
| every code commit gated | done | below |
| `.claude/rules/` amended, CLAUDE.md, README, the changelog | done | `web.md` (new, in CLAUDE.md's table), `flake.md`, `core-proofs.md`, `core-focus.md` |

### Decisions

- **Stateless calls.** Every call takes the session document and the
  settings and returns the new document; nothing is kept in the module.
  The page holds both anyway (the link, D20; autosave, D29; several
  proofs, D50), and cancellation is `terminate()` of the worker (D28),
  after which a handle in the module would be gone. The price is reading
  the session at every call, which `wire::upgrade` does linearly with
  the replay that checks it.
- **Errors are thrown as the library's error JSON**, `Error`'s written
  form or `describe`'s with the session's formulas, so that the page
  branches on `code`, `kind` and `details.kind` (a `StepError`'s kind and
  positions for a refused step) and words the message itself (D11, D15,
  D44). The bindings' own refusals take that shape and the library's
  codes.
- **The settings are `Settings`' JSON read over the tab's defaults**, key
  by key: `{}` is the tab's settings. Reading a page's JSON as `Settings`
  alone would give every missing key the command's default, a gibibyte
  of memory among them.
- **The tab's bounds** (`web/src/tab.rs`, each a constant with its
  reason): 96 MiB for the memory account, a little over a third of the
  256 MiB of wasm memory a tab on iOS gets (D28), since the account does
  not count the forest, the stack, the documents and the drawing; a
  million occurrences (a forest of 25 MB, which a session, a proof and a
  net each copy); a derivation of 4 MiB by the estimate, since a
  drawing's peak is several times its estimate (the size estimate's
  measurements: the SVG at 855 MB for an estimate of 130 MiB); a work
  budget of 2²⁶ units, about a second by item 3's rate; the classical
  refuter's work 2²² terms, a sixteenth of the library's.
- **The stack**: raised to 8 MiB with `-zstack-size` (`web/build.rs`,
  read back as `tab::STACK_BYTES`) and the recursion limit lowered too,
  because the module's stack is not what runs out. Each wasm frame is
  also a frame on the host's own stack, about 1 MiB in Node and in a
  browser and not settable by a page. Measured in Node (debug and
  optimized builds, three paths): copies of a `!` held 2 048 levels
  optimized and failed at 3 072; nested `&` was cheap; a Mix of pairs,
  the largest levels, held 768 optimized and failed at 1 024, held 512
  in debug and failed at 768 ("RangeError: Maximum call stack size
  exceeded", a clean failure the worker reports). The limit is 384, half
  the deepest the optimized module held, the factor the library's own
  stack estimate keeps (`Limits::PER_LEVEL`). The test at the limit runs
  the three paths at 384 under Node in the debug build.
- **The library's own limit tests run on wasm32-wasip1 under wasmtime**,
  not on `wasm32-unknown-unknown`: `wasm-bindgen-test-runner` only runs
  `#[wasm_bindgen_test]` functions, and the library's unit tests are
  plain `#[test]`s. Both targets are wasm32 with the same data layout
  (a 32-bit `usize`, hashbrown's 8-byte control group), so the tests
  read the arithmetic and the layout a browser runs, and the library's
  tests stay as they are.
- **Item 3's engine is forced** (departs from the design): the design
  measures "the target set's small rows", but the default dispatch now
  sends every decided row with exponentials either to the Horn engine
  (91 of them) or to the focused engine where both biases give every
  atom the same literal and one search runs (`chain`), so no decided row
  of the set runs the turns by default. The measurement forces the
  engine the two searches belong to (`focus` classically, `two-sided`
  intuitionistically) on every row with exponentials in a linear mode.
- **The hint is the first steps of the goal's proof** (D12): `hint`
  runs the search on the goal, the session untouched, and reads the
  derivation of that proof alone from its root up to and including the
  first rule that is not structural, each step as `apply` takes it on
  the goal the one before opened (the principal's position, and for a
  `⊗` or Mix the left premise's formulas as positions). The page shows
  the last of them at the rung "which rule" and applies them all at the
  bottom-out rung. A library call for the first steps of a goal's proof
  would serve other front ends; it is listed in `plan/web/README.md` as
  what the library lacks.
- **The notices** of the crates inside the module are cargo-about's
  (D32), from `web/about.toml`'s accepted licences, which are
  `deny.toml`'s, `--frozen` in the package build; the bindings' crate
  itself is left out and the package carries `LICENSE`.

### Item 3: the two searches of the default bias under wasm

**Method.** The release package (`nix build .#linlog-web`: the module
after `wasm-opt`) in Node 24, on cores 12 to 15, one fixed set for all
runs, under the machine lock, as the unit `step32a-turns`
(2026-10-19, about ten minutes). The rows: every row of the target set
`bench/targets/after-efficiency.csv` with exponentials (fragments MELL,
LL, IMELL, ILL) in a linear mode that is not affine, 127 of them,
their sequents generated from the library's families and read from the
LLTP library (`plan/notes/web-turns/rows.rs`). For each row, three
searches with the row's copy bound and the tab's recursion limit, the
engine forced as above, the classical refuter off: the backward search
alone (`bias: rarer`) and the forward search alone (`bias: factors`, with
the forward bound of 30 copies where the row is a Horn program, the
bound the default bias gives it there), each within W = 2²⁴ units of
work, and the default bias, the two in turns, within 8 W. Each search
ran twice and the second time was kept (`plan/notes/web-turns/turns.mjs`,
times by `performance.now()` around the call; the data
`plan/notes/web-turns/turns.jsonl`, read by `analyze.py`). "Small" is
what the question needs: a row on which the better explicit search
decides within W.

**Numbers.**

- 48 of the 127 rows are small (45 Horn programs in intuitionistic
  mode, `chain/16`, `chain/64`, `chain/128`); the better search is the
  forward one on 47 of them. The default bias decides all 48, with the
  oracle's verdicts (46 proved, 2 unprovable); no verdict of any run
  differs from the oracle's.
- **Work**: the default bias against the better search, a median of
  1.00 and at most 4.98 (`Echo_echo-d5r3_5_1`), within the bound the
  scheme promises (less than five times the better search's work).
- **Time under wasm**: a median of 1.04; over the 48 rows 1 457 ms against
  663 ms, 2.20 times; above five times on two rows,
  `PermAdmissibility_unf-8x8-4stageSEN-50_10_1` (19.4 times, 303 ms
  against 16 ms, 2.95 times the work) and `SwimmingPool-4_10_1` (5.1
  times, 10 ms against 2 ms). There the time exceeds the work: every turn
  starts its search afresh (a new run, memo and arena), and on a large
  sequent that set-up is what the turns repeat. The median better search
  takes 1.9 ms, the longest default 303 ms.
- Of the 79 rows neither explicit search decides within W, the default
  decides 12 within 8 W; the others end at the work bound (52), the copy
  bound (20 and 2) or the recursion limit (5).
- The backward search did 68 million units of work a second (the median
  over its runs above 50 ms), which sets the tab's work budget.

**Answer: no search that can be suspended is needed.** In work the turns
cost what the scheme promises, at most five times the better search, and
in the median nothing. In time they cost 2.2 times the better search
over the small rows, and the worst row stays a third of a second. A
search that can be suspended (an explicit stack in the focused engine,
so that the two alternate in slices, as they do on threads) would save
the restarts. It would turn the worst row's 19 times into its work
ratio of about three, on a call that takes a third of a second in a tab,
and is not worth a step of its own for the page. What the numbers do
show is the restart's set-up cost on large sequents, a follow-up for the
focused engine (below).

### What is verified

- **The gate** (`taskset -c 8-15 nix develop -c gate` in a capped unit,
  in the workspace `../linlog-step32a-gate` at each commit): passed at
  every code commit, `lonrknzo` (the core tests' build without
  `parallel`), `quznwnno` (the 32-bit arguments; restarted once after
  the formatter's change to its test, which was folded in before the
  commit was gated), `zvxnzksq` (the bindings) and `ovuwwmtm` (the
  review's answers), each ending `gate: passed`, exit 0. The ratchet's
  largest rise was +1.80 % (`batch-families`), within its ceilings; no
  search changed.
- **The bindings' tests**: twelve, natively (`cargo test -p
  linlog-web`) and under Node (`wasm-bindgen-test-runner`, the debug
  build), all passing at `ovuwwmtm`; clippy clean natively and for
  wasm32 (`--no-deps`).
- **The library's tests on 32 bits**: the 22 tests of `proofs::check`,
  `proofs::size`, `search::focus::memo` and `search::memory` on
  wasm32-wasip1 under wasmtime pass after `quznwnno`; before it, two of
  them failed there (`holds_no_more_than_its_bound`,
  `refuses_its_tables_before_it_judges`, each naming the tables' bytes
  for 64-bit pointers), and the library's unit tests did not build
  without `parallel` until `lonrknzo`. The same 22 pass natively.
- **The layout fact the sentences rest on**: hashbrown's table of 16
  buckets of `u32` takes 96 bytes on x86-64 and 88 on wasm32-wasip1 (a
  counting allocator under wasmtime), so its control group is 8 bytes
  there and `table_bytes` counts 8 above it.
- **The flake**: a first `nix flake check` (under the lock, at
  `vnrsunuo`) failed at `typos` alone (the settings test's misspelt key)
  and built every other check, the new ones among them
  (`web-wasm-tests`: 10 passed in the sandbox; `web-core-wasm32`: 22
  passed; `web-package`; `web-bindgen`); the key changed in
  `ovuwwmtm`. The run at the report commit, the session's last, is in
  the message to `planning`.
- **The package**: `nix build .#linlog-web` under the lock gives
  `linlog_web.js`, `linlog_web_bg.wasm` (5.2 MB, 2.0 MB with gzip -9),
  their type declarations, `LICENSE` and `THIRD-PARTY-NOTICES.txt` (33
  licence groups over the module's crates).
- Signing tested before every commit; every commit is signed.

### The reviewer's findings

One fresh-context reviewer (Opus 5.5, `high`, briefed with the rules on
cores and memory, cores 8 to 11, and told to report only what breaks
correctness or a stated requirement) read the JSON boundary and every
integer argument made for 32 bits, and probed them: 1 871 mutated
sessions and outcomes through every call (no panic natively, no abort
under wasm32), malformed steps, modes, formats, SVG, fonts, nested JSON
and formulas 200 000 deep (each the error JSON), the view's map against
the drawing, the 22 library tests on wasm32-wasip1, positions in a
step's JSON past 32 bits in Node, and each sentence of `quznwnno`. It
found six things, each answered in `ovuwwmtm`:

1. **Following the hints never finished a proof that contracts a `?`
   formula** (`!A |- A * A`): the hint was the root inference of the
   found proof, a contraction, and on the contracted goal the search's
   root was the weakening of the copy, so the two alternated forever.
   Holds. A hint is now the steps from the root up to and including the
   first that is not structural; `following_the_hints_proves_a_sequent`
   follows hints to a finished proof on four sequents, two of the
   reviewer's among them.
2. **A JSON array in the settings brought back the command's
   defaults** (`[]`, `{"limits": []}`): serde reads an array as a
   struct's fields in order and `Default` fills the rest, so the
   overlay's "a key replaces the default" let a gibibyte and a recursion
   depth of 2 048 through. Holds. The overlay refuses a value that is not
   an object where the tab's settings have one; the unit test reads the
   three cases.
3. **`hint` threw where its doc said `step: null`**, on a session whose
   whole derivation passed `derivation_bytes`, since it built the
   session's derivation to read one inference. Holds. It reads the goal's
   proof's derivation alone now, and a refusal of that is `null`.
4. **`close_all`'s budget ended with `stopped`, not `work_limit`**, since
   the budget was a stop that fired. Holds: the page branches on the
   kind, and `stopped` is a cancel's. Each goal is now closed with what
   the searches before it left of the budget as its own `limits.work`;
   the library reports `work_limit`. What a search spent is its
   `statistics.work` (a count of the polls missed the work after the
   last poll: a Horn goal's polls said 4 units where its check alone
   took more than 6).
5. **Goal and position numbers from JavaScript wrapped**: a `u32`
   parameter of wasm-bindgen takes any number modulo 2³², so `2 ** 32 +
   3` was goal 3. Holds. The exports take a number and `calls::index`
   refuses one that is not an integer from 0 to `u32::MAX`
   (`a_number_from_the_page_is_an_index_or_refused`).
6. **`closing_stops_at_the_work_budget` did not test the shared
   budget its doc named**: its session had one goal. Holds.
   `closing_all_goals_shares_one_budget` splits a goal in two and closes
   both with the first's work and half the second's: the first proved,
   the second `work_limit`. The focused engine searches there, since the
   Horn engine closes those goals in a few units, fewer than the check
   of the proof.

Sound by the reviewer's reading: every 32-bit sentence of `quznwnno`
(the tables below 2³⁶, `Bag::len` saturating above `Bag::MOST` on 32
bits, `State::entries` widened, the builder's capacity at most
`Derivation::MOST`, the memo's products bounded by a 4 GiB address
space), the positions inside a step's JSON, `{}` being the tab's
settings, and the view's `inferences` against the drawing's ids.

### Left open

- **The restart's set-up cost** on large sequents under the turns
  (19.4 times the better search's time on
  `PermAdmissibility_unf-8x8-4stageSEN-50_10_1` for 2.95 times its
  work): a turn could keep the set-up its search made (the classes, the
  counts) and drop only what the search grew. Unassigned; it matters
  only without threads.
- **The library's clippy under wasm32** finds sites that are right on
  both widths but that clippy reads per target:
  `absurd_extreme_comparisons` at `core/src/ordinary/mod.rs:452`
  (`Formulas::MOST` is `usize::MAX` on 32 bits) and three
  `unnecessary_box_returns` in `core/src/proofs/check/pass.rs`, whose
  `State` is smaller there. No check runs clippy for wasm32 (the
  bindings' own code passes it, `--no-deps`); a `#[cfg_attr]` or a check
  of its own is for a later session.
- **The library's whole unit test suite on 32 bits**: `web-core-wasm32`
  runs the checker's, the size estimate's, the memo's and the memory
  account's. Running the rest there (many use threads, which WASI lacks)
  would find what else assumes 64 bits; not done.
- **A first step of a goal's proof** as a library call (for the hint),
  **`Step` and `Applicable` as wire forms**, **ordinary logic's JSON
  forms**: listed in `plan/web/README.md`, each additive at level 1.

### Workspaces

`../linlog-step32a` (this session's, at the report commit) and
`../linlog-step32a-gate` (where each code commit was gated). Both can be
removed once the commits are on `main`: nothing in them is untracked
but the build trees.
