# Step 32: the web front end

## The checklist

| item | state | evidence |
|---|---|---|
| Session 32a: the plan and the bindings (session `step-32a`, 2026-10-19) | done (2026-10-19) | `plan/web/README.md`; the crate `linlog-web` and its flake checks (`zvxnzksq`, `ovuwwmtm`); the 32-bit arguments (`quznwnno`); item 3: no suspendable search needed; the reviewer's six findings answered; every code commit gated; `nix flake check` at the last commit |
| Session 32b: the client's repository and version 1's core (session `step-32b`, 2026-10-19) | done (2026-10-19) | the repository `linlog-prover.github.io` (`mvuvktkp` to `uymkopks`); version 1's core with 27 Vitest and 24 Playwright tests in its `nix flake check`; the reviewer's five findings answered; no change of linlog |
| Session 32c: the hosting, the licences and the rest of version 1 (session `step-32c`, 2026-10-19) | done (2026-10-19) | the client's commits `tlsoutzz` to `tmpzrkvn`: the Pages deployment, the CSP and its check, the licences and their check, offline and installable, About, the examples, the tour, Learn's frame, Lucide's check, the three follow-ups; 39 Playwright and 42 Vitest tests in its `nix flake check`, exit 0 at `tmpzrkvn`; the reviewer's two findings answered; no change of linlog's code |

## Session 32a: the plan and the bindings

**Outcome.** The web front end has its plan, `plan/web/README.md`: the
three sessions and which of the design's decisions each builds, version
1 and what it leaves out, the bindings' API call by call, the tab's
defaults, and, item by item of `plan/34-web.md`'s "What the bindings
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

## Session 32b: the client's repository and version 1's core

**Outcome.** The web client has its repository,
`/home/tux/Projects/own/linlog-prover.github.io` (a jj repository
colocated with git, now `linlog-prover/linlog-prover.github.io` on
GitHub, pushed by the planning session), set up as the author's sites
are, and version 1's core runs in it: a sequent typed in the field is
proved by clicking on the stage, with linlog's module in a module
worker, every action plan/web/README.md's 32b list names, and one
Playwright test per interaction at desktop and phone widths. The
client's `nix flake check` passes at its last commit; GitHub's CI passed
on the skeleton. linlog itself needed no change: the bindings of 32a
answered every call the page makes, so `../linlog-step32b` holds this
report alone.

### The checklist

| item | state | evidence |
|---|---|---|
| the repository as the author's sites (stack-spec and website-learn read first, what applies copied) | done | `mvuvktkp`: the dendritic flake with linlog as a pinned input, npm under Nix (website-learn's `node-modules.nix` and `.npmrc`), Astro 7.3.5, TypeScript 6.0.3, Solid 1.9.17 as one island, Tailwind 4.3.3 with the mockup's tokens, Biome, Vitest 5 with fast-check, Playwright 1.63 in nixpkgs' Chromium, CI pinned by SHA, the licence and headers, CLAUDE.md and `.claude/` |
| the message to planning once the skeleton's check passed | done | sent with `mvuvktkp`; planning reviewed, asked for the CI cache change (`ynkuklvx`), created the GitHub repository and pushed `main` at `ynkuklvx`; CI run 39209753641 passed |
| the module in a module Worker, request ids, cancellation by `terminate()` and a fresh worker | done | `zxmmnkol`: `src/lib/protocol.ts`, `src/worker/linlog.worker.ts`, `src/lib/api.ts` |
| version 1's core (the 32b list) | done | `smkwwtyy`, `pzptystz`, `vllkpqqv`, `vzqxtknr`; below, item by item |
| Vitest of the client's logic (the link's codec, the step tree, the worker protocol) | done | 27 tests in six files, the step tree's against linlog's real module, fast-check properties for the codec, the protocol's ordering and random proof attempts' replay |
| one Playwright test per interaction, inside the client's `nix flake check` | done | 24 tests (`tests/prover.spec.ts`, `page.spec.ts`, `phone.spec.ts`), the check `e2e` |
| the fresh-context reviewer on the worker protocol and the client's state | done, five findings, each answered | below; `vzqxtknr`, `uymkopks` |
| `nix flake check` at the client's last commit | done | exit 0 at `uymkopks` (the session's last client commit), under the machine lock, `e2e` run afresh there (24 passed) |
| any change of linlog gated | nothing to gate | no change of linlog was needed |
| this section, committed in `../linlog-step32b` | done | this commit |

### Version 1's core, item by item

- **The sequent field** (D1, D53, D55): a new proof is a tab with the
  field focused and the placeholder `A * B |- B * A`; Enter starts it,
  Esc gives up an edit; a parse error marks its place in the text and
  says its keywords ("Unexpected * at column 5"), the full sentence on
  hover; a sequent that is not intuitionistic in that mode says "2
  formulas right of ⊢". The fragment linlog detects stands beside the
  calculus's mark, updated as the text is typed (D59).
- **The calculus's switches** (D2, D58): one cluster at the stage's top
  left, Classical or Intuitionistic, Mix, Affine, and Two-sided greyed
  with its reason; a change restarts the proof, after a warning when
  it has steps. Linear or Ordinary sits on the tab's mark (D57), Ordinary
  greyed.
- **The goals with clickable formulas** (D3, D5, D33): linlog's SVG on
  the dotted stage under a camera (zoom by buttons, keys and ctrl with
  the wheel, pan by dragging or the wheel, fit), an HTML button over
  every formula of an open goal (the drawing stays a picture), the
  active goal in the accent, the others dimmed, brought into view after
  every step; the dock above the active goal holds the navigator ("Goal
  3 (1 of 2 open)"), its close, its note and the hint.
- **The rule popover** (D9): the rules `rules` gives for the formula,
  nothing else, each with its digit; never applied on the click alone.
- **The split chooser** (D9): the formulas of the goal are clicked to
  go left, the premises listed, `split_passes` saying "Passes" or
  "Cannot close" before Apply.
- **Refusals with the library's reason** (D11, D15, D44): the step's
  error JSON worded by the page by its `details.kind` ("ax refused: no
  dual partner"), at the goal, its sentence on hover and in the live
  region; the formula's button shakes (not under reduced motion).
- **Undo per branch and redo** (D13): the step tree is read off the
  session document; undo retracts the step that opened the active goal
  and every step since in its premises, replaying the rest in the
  worker; redo puts it back until a new step; the Steps list goes back
  to any step.
- **A goal's close and all goals'** (D10, D14, D15): the search's proof
  grafted with its rule labels faint and italic; an unprovable goal says
  why with "checked" when `Disproof::check` held ("Unprovable, checked:
  atom balance"); a search that gave up says so, its reason and engine
  on hover, with a button for twice the budget; a running search shows
  "Stop the search", which terminates the worker.
- **The hint ladder** (D12): four rungs on the active goal from `hint`:
  the kind of rule next, the formula (marked), the rule and its split,
  then the page applies the steps and asks why.
- **The derivation and the finished proof's net** (D4): the Tree and Net
  tabs, Net greyed with linlog's reason or until the proof is finished;
  an intuitionistic proof's net is its essential net.
- **Every export** (D16): SVG, PNG and PDF (rendered by the module from
  Euler Math's bytes, a PDF dated by the page's clock), LaTeX, Typst,
  text, the session as JSON, the Rocq certificate of a finished proof,
  the certificate of non-derivability of a refuted goal, and the link.
- **The link** (D20): `#v=1&m=<mode>&s=<sequent>&d=<session>`, the session
  deflate-raw and base64url, written with `replaceState` as the proof
  changes, dropped past 2000 characters (the copy says to export the
  session instead); a link whose session does not read loads the
  sequent and mode with "Proof not restored".
- **Autosave** (D29) and **several proofs** (D50): every open proof in
  local storage; on a visit the last one's are set aside and offered by
  the restore bar, so nothing is overwritten before the visitor answers;
  tabs switch, close and keep each its own state, link and redo.
- **Import** (D51): a file, a paste, or a drop onto the stage, of a
  session or a sequent as linlog writes them; the panel says which it
  got, and a level it does not know is refused ("Level 9 unknown: not
  imported").
- **The keyboard path** (D17): every formula a button, Enter opens its
  rules, a digit picks one (queued if typed before the rules arrive),
  U, ⇧U, C, ⇧C, H, S, [ ], V, + − F, N, E, I, `,` and `?` for the Keys
  pane.
- **The light and dark toggle** (D26, D46): the bar's toggle pins a
  scheme, Settings offers System, Light and Dark, the choice applied
  before the first paint; every colour a `light-dark()` token.
- **Desktop and phone** (D18): at 390 pixels the bands keep one row, the
  actions by icon with their names kept for a screen reader, and the
  rule popover and split chooser become a sheet at the stage's bottom;
  `tests/phone.spec.ts` proves a sequent there and finds no sideways
  scroll.

### Decisions

- **The repository**: the default name and place of `plan/34-web.md`
  item 1. Its flake is the author's sites' scheme (flake-parts,
  import-tree, website-learn's `node-modules.nix` and `playwright.nix`,
  a `ci-cache` derivation), the Claude Code setup linlog's (its git and
  `jj new -m` guards with their `claude-hooks` check, the formatter
  hook) with website-learn's island rule adapted.
- **linlog through its input, the font too**: the package `linlog-web`
  and `cli/fonts/Euler-Math.otf` with its `OFL.txt` come from the input
  `github:linlog-prover/linlog` locked at `b509f719d767`; the dev shell
  and the sandbox link them at `.cache/`, where Vite's aliases read them,
  so the module and the font get hashed names like every asset.
- **The step tree is the session document's**: undo per branch, redo and
  the Steps list read the tree off `inferences` and `history` and name
  goals by path, never by number; a graft replays by `close` again, or
  from its own inferences where the search no longer closes it. So a
  session from a link, a file or storage undoes per branch too, and no
  separate record can drift from the document.
- **The page's settings are the tab's defaults with the work budget**:
  `{"limits": {"work": …}}`, 2²⁶ units by default, changed in Settings;
  the drawing asks `styles.svg.font_size` 30 so that a formula is a
  24-pixel target.
- **Tests**: Playwright against the built site served by
  `tests/serve.ts` as Pages serves it (no header), 24 tests, one per
  interaction; Vitest beside the code, the step tree's tests loading the
  real module in Node.

### Departures from the design

- **nixpkgs and the flake machinery follow linlog's inputs** (departs
  from the design, D41's "nixpkgs from the channel tarball"): one
  package set for the bindings and the site, every input past the
  fourteen days D43 asks, and an update of linlog moves both.
- **Solid 1.9.17 past the age gate**: the seroval fix, the version the
  author's sites run; the author accepted it on 2026-10-19 (departs from
  the design, D40's `min-release-age=14`).
- **The drawing takes the theme's colours from CSS**, not from `Style`
  (departs from the design, D26's "the drawing's Style takes the theme's
  colours"): the page overrides the SVG's paint with the tokens, so a
  theme switch needs no redraw; the exports keep linlog's own colours,
  which print.
- **Goals by `[` and `]`**, as the mockup has them, not the arrows
  (departs from the design, D17's "arrows between goals"): the arrows
  scroll and move within the page's controls.
- **The hint ladder starts on the active goal** (departs from the
  design, D12's first rung "which goal and which kind of connective"):
  the rung names the kind of rule next on the goal the student is on;
  choosing among goals is left to them.
- **The phone's bottom band stays a band** of icons, not a sheet; the
  popovers become the sheet (departs from the design, D18's "the HUD
  becomes a bottom sheet").
- **The Steps action in the bottom band**, the mockup's own departure,
  kept: D13's step list needs a home.
- **Euler Math comes from linlog's `cli/fonts/`**, the copy of CTAN's
  package the prover measures its drawings with, at the input's pinned
  commit (departs from the brief's "fetched from their upstream"):
  Figtree and Space Mono come from npm through the lock.

### What is verified

- `uymkopks` reads what each export holds (the PNG and PDF by their
  magic, LaTeX, Typst, the text, the session, both Rocq certificates by
  their contents) and checks that the hint ladder's last rung applied the
  step (the Steps list holds it).
- `nix flake check` of the client, under the machine lock on cores 8 to
  15: passed at the skeleton (`mvuvktkp`), at the CI change
  (`ynkuklvx`), before `zxmmnkol` and `smkwwtyy` were committed, at
  `vllkpqqv`, at `vzqxtknr`, and at `uymkopks`, the last (its `e2e` run
  afresh, 24 passed); one run before `vllkpqqv`
  failed in `e2e` (the keyboard test, twice with its retry), which found
  the dropped digit fixed there.
- The checks: `site`, `unit` (Vitest), `typecheck` (`astro check`),
  `lint` (Biome), `e2e` (Playwright), `no-style-attributes`,
  `licence-headers`, `actionlint`, `deadnix`, `treefmt`,
  `claude-hooks`, `playwright-version`.
- GitHub's CI on the pushed skeleton: run 39209753641 passed (planning's
  report).
- The bundle: the page's JavaScript and CSS 43 KB compressed on the
  critical path, the fonts beside; the module, 2.0 MB compressed, loads
  in the worker after the first paint (D35).
- Signing tested before every commit; every commit is signed.

### The reviewer's findings

One fresh-context reviewer (Opus 5.5, `high`, briefed with conduct.md's
rules on cores and memory, cores 8 to 15, told to report only what
breaks correctness, the design's decisions or a stated requirement) read
the worker protocol, the step tree, the link's codec and autosave, and
probed them against linlog's module in Node. It found five things, each
answered in `vzqxtknr`:

1. **A split read wrongly once a contraction had copied occurrences**
   (`|- ?(A * B), C * D`: contract, derelict twice, split, then split
   the goal `A ⊗ B, A` with `A` left, whose left premise is `[2, 2]`):
   `stepOf` counted occurrences, and a context formula with the
   subformula's occurrence made it read a split as a copy; replay then
   built another proof without a word, and an undo on one branch changed
   the other. Holds. The left side is now found by aligning the goal
   with both premises (`leftOf`), and every replayed step must make the
   premises it made, or it is taken back and its goal reported. The
   scenario is a test, failing on the old step tree, and the replay
   property now runs 150 random attempts over seven sequents, two of
   them contracting.
2. **Undo lost a graft made with twice the budget**: the replay closed
   the goal again within the settings' budget, failed, and reopened a
   branch the undo was not on. Holds. A graft the search no longer
   closes replays from its own inferences (kept in the action), so it
   stays closed, set apart as the search's no longer; a test, failing on
   the old step tree, has it.
3. **Actions overlapping a running call overwrote each other** (a close
   running while a step was applied lost the graft; two undos during a
   replay undid one step and pushed redo twice). Holds. One action that
   changes a proof runs at a time (`guard` in `state.tsx`); its buttons
   wait, and the dock offers "Stop the search". `uymkopks` tests it:
   undo pressed during a half-second search does nothing, and the
   search's verdict and the step before it stay; the test fails with the
   guard turned off (the verdict was lost).
4. **Autosave kept an older copy and piled up copies**: a proof set
   aside by one tab and worked on in another came back at its older
   state; a reload of a link's page offered the proof it had just
   opened, once more per reload. Holds. Every visit saves under a key of
   its own with the time, the copy saved last wins when visits are
   gathered, and a proof open already is taken out of the bar; the
   shelf's tests have both cases, and the link's Playwright test reloads
   and finds no bar.
5. **A worker that failed as it loaded was restarted forever** (185
   workers in 200 ms in the reviewer's probe). Holds. A worker starts
   with a call, never by itself; the protocol's tests pin that a failing
   worker is not restarted without a call, and that the next call starts
   the fresh one (the cancel test changed with the behaviour, as the
   commit says).

The reviewer's report is the text it handed back; the harness lets a
sub-agent return findings only as text and refused its writing them to
a file, so the report exists in this session as that text (extracted
verbatim from its transcript there) and as its probes under the
session's scratchpad, which reproduce findings 1, 2, 4 and 5 on the code
it reviewed. Finding 3 is the reviewer's reading of the code, not a
probe; the test that pins its answer is this session's, and fails
without the answer.

Sound by the reviewer's reading: ids never reused, so a late answer of a
stopped worker is ignored; a replay is one request; paths, not goal
numbers; undo past a split and on a finished proof; single-inference
grafts and `close_all`'s history replay alike; the link's round trip,
its 2000-character fallback, a bad document or an unknown level; import
of a session, a sequent and an unknown level.

### Left open, for 32c

- D31's hosting: the Pages deployment (`upload-pages-artifact`,
  `deploy-pages`) in the CI workflow, a `404.html`, the rustdoc staying
  at `/linlog`; GitHub's own Jekyll build of `main` failed and published
  nothing (planning), so the workflow will be the only publisher.
- D42's security baseline: the CSP as a `<meta>` tag with
  `'wasm-unsafe-eval'` and the hashes of Astro's inline scripts (the
  pre-paint theme script, the island's loader).
- D32's other half: Vite's `build.license` for the npm bundle, served
  with the package's `THIRD-PARTY-NOTICES.txt` and the font's `OFL.txt`
  (the font is served under `/_astro/` today, its licence not yet
  beside it), item 6's CC BY 4.0 on the pages that carry written
  content.
- D49 (offline and installable), D52 (About, with the footer's line on
  GitHub's logs, D36), D7 (the examples list), D8 (the tour), D6 and
  D47 (Learn and its frame), D21 and D23, and the repository's website
  field.
- Lucide's version and Solid binding checked at build time (D54): the
  package is pinned by the lock, nothing checks it further.

### Repositories and workspaces

- The client: `/home/tux/Projects/own/linlog-prover.github.io`, `main`
  on GitHub at `ynkuklvx`; the commits above it (`zxmmnkol`, `smkwwtyy`,
  `pzptystz`, `vllkpqqv`, `vzqxtknr`, `uymkopks`) are local, for planning
  to review and push. Planning's review workspace
  (`planning-review`, its empty change `wwpzpvnx`) is planning's.
- linlog: `../linlog-step32b`, this report's commit on `nqsytyrq`.

## Session 32c: the hosting, the licences and the rest of version 1

**Outcome.** The web client is ready to be published: its CI deploys
`main` to GitHub Pages once `nix flake check` passed, with a `404.html`
and nothing at `/linlog`; every page carries the Content-Security-Policy
as a `<meta>` element with `'wasm-unsafe-eval'` and the hash of every
inline script and style, which a flake check recomputes; the site serves
the licence of everything it sends (the npm bundle's notices by Vite's
`build.license`, linlog-web's `THIRD-PARTY-NOTICES.txt`, the EUPL, each
font's OFL beside it, CC BY 4.0 on the page that carries written
content), which another flake check reads; and the rest of version 1 is
built: offline and installable, About, the examples to practise on and
to watch, the tour, Learn's frame with the places of the tutorial,
practice and reference, D54's check of Lucide, and the three client
follow-ups of the review of 32b. Each has a Playwright test, 39 in all,
every one of them failing on a violation of the policy. Nothing of
linlog needed to change: `../linlog-step32c` holds this report alone.

### The checklist

| item | state | evidence |
|---|---|---|
| D31: the Pages deployment from `main` after the checks, a `404.html`, nothing at `/linlog` | done | `tlsoutzz`: `upload-pages-artifact` v5.0.0 and `deploy-pages` v5.0.1 pinned by SHA, the upload in the check job after `nix flake check`, a `deploy` job with `pages: write`, `id-token: write`; `src/pages/404.astro`; the `pages-paths` check; a Playwright test of the 404 |
| D42: the CSP as a `<meta>` element with `'wasm-unsafe-eval'` and every inline hash, with a check | done | `trqzxxqp`: Astro's `security.csp`, the pre-paint script after the policy with its hash added; the `csp` check (`modules/_checks/csp.py`), shown failing on a build with one hash removed; `tests/fixtures.ts` fails any test on a violation; a test shows the policy refusing inline code the build did not hash |
| D36: the line on GitHub's logs | done | in About (D52), `lqrzqtmy`, its Playwright test |
| D32 and item 6: the licences served and checked | done | `pwsnmrot`, `lqrzqtmy`: `/licenses/bundle.txt` (Vite's `build.license`, and Astro's notice for its inline runtime), `/licenses/THIRD-PARTY-NOTICES.txt`, `/licenses/LICENSE.txt`, `/fonts/<Family>-OFL.txt` beside each font, the CC BY line with the written content; the `licences` check; About's test fetches each |
| D49: offline and installable, the worker never on `/linlog/` | done | `ztnoqqzq`: `src/offline/sw.js` by hand, the manifest and its icons; three Playwright tests (a proof with no network, `/linlog/` untouched and uncached, Chromium's installability) |
| D52: About | done | `lqrzqtmy`; its test |
| D7, D21, D23: the examples list, their sessions, the content in the repository | done | `lqrzqtmy`: nine examples in `src/content/examples/`, seven sessions written by `linlog interact`; `src/content/examples.test.ts`; Prove and Watch tested |
| D8: the tour | done | `lqrzqtmy`: eight stops proving `A ⊗ B ⊢ B ⊗ A`; tested by keyboard under reduced motion |
| D6, D47: Learn and its frame with the tutorial's, practice's and reference's places | done | `lqrzqtmy`; tested on the desktop and at a phone's width (`nnxwqzxw`) |
| D54: Lucide checked at build time | done | `xrumwtlk`: `src/integrations/lucide.ts`, its Vitest test, a Playwright test of the pictograms |
| The review's follow-ups: a link's parse error, a cap on what `d` unpacks to, the tab's sequent in Euler Math | done | `pumurovn` (the tab's text settled in `lqrzqtmy`); a Vitest test of the cap and three Playwright tests |
| The fresh-context reviewer on the CSP and the worker's scope | done, two findings, each answered | below; `tmpzrkvn` |
| `nix flake check` at the client's last commit | done | exit 0 at `tmpzrkvn` (`aea6fcf98c92`), the session's last client commit, under the machine lock: every check, `e2e` 39 passed, `unit` 42 |
| Any change of linlog gated | done | only `plan/` changed here (this report, `plan/later.md`'s follow-ups); `nix flake check` passed at this commit (the session's output) |
| README true | done | `nnxwqzxw` |
| This section, committed in `../linlog-step32c` | done | this commit |

### What was built, item by item

- **The deployment** (D31). The check job, on `main` alone and only
  once every step before passed, builds `packages.default` into `site/`
  and uploads it; the `deploy` job, the only one given `pages: write`
  and `id-token: write`, publishes it in the `github-pages` environment,
  one deployment at a time and never cancelled midway. The upload drops
  dot-files, so nothing the site serves starts with a dot (Vite's
  default `.vite/license.md` would have been lost). The `pages-paths`
  check holds a `404.html` at the root and nothing at `/linlog`.
- **The policy** (D42, D28). Astro 7's own `security.csp` writes the
  `<meta>` element with the hashes of the inline code Astro makes; it
  writes it at the end of `<head>`, and a `<meta>` policy covers only
  what follows it, so the pre-paint theme script moved to the start of
  `<body>`, its text a constant whose hash the config adds (Astro hashes
  no `is:inline` script). `default-src 'none'`, `script-src 'self'
  'wasm-unsafe-eval'` and the hashes, `style-src 'self'` and its hash,
  `img-src`, `font-src`, `connect-src`, `worker-src` and `manifest-src`
  `'self'`, `base-uri` and `form-action` `'none'`. The worker that runs
  linlog's module gets no policy (a worker's comes from its own
  response's headers, and Pages sends none).
- **The licences** (D32, item 6). Vite's `build.license` names the npm
  packages in the bundle with their licence texts; Astro's own notice is
  added, since its island runtime is inline in the pages where no bundle
  sees it. The fonts are put under `/fonts/` by `assetFileNames` beside
  their OFL texts; Euler Math's URL is imported by the page and passed to
  the island, since Astro's client build names its own assets. The
  `licences` check reads all of it apart from the build: every run-time
  dependency at its locked version with its licence text, the crates'
  notices and the EUPL byte for byte, every font beside an OFL, Euler
  Math's that of linlog, and the CC BY link on every page marked as
  carrying written content.
- **Offline and installable** (D49). `@vite-pwa/astro` 1.2.0 supports
  Astro up to 5, so the worker is written by hand (D49's other choice):
  the build fills in the list of the site's files and a hash of their
  contents; it caches them all when it installs, answers only for them
  and for navigations to `/`, and a new deployment waits until no page
  of the old one is open, so a page never mixes two deployments' files.
  The manifest's icons are drawn from the site's icon by linlog's own
  renderer in Node, so no image library was added.
- **The written content** (D21, D23, CC BY 4.0) is Markdown in Astro's
  content collections under `src/content/`: the examples, the tour's
  stops, About and Learn's three openings. The page renders each into an
  inert template marked `data-written`, which the island clones; so the
  content is in the built HTML, where the style, policy and licence
  checks read it.
- **Practice** (D7, D21): nine examples, easiest first, two of them
  without a proof (an atom left over, and `A & B ⊢ A ⊗ B`, which only an
  exhaustive search refutes). Each was decided by linlog's command
  first, and each provable one's session was written by `linlog
  interact` with `save` and checked with `proof`; a Vitest test holds
  every example to linlog's verdict in its mode and every session to its
  sequent and a finished proof. Prove opens the sequent; Watch opens its
  proof taken back to the sequent with each step alone on the redo
  stack, so that Redo plays it one step at a time.
- **The tour** (D8): eight stops, each pointing at a part of the page
  and performing a real step on `A ⊗ B ⊢ B ⊗ A` once, on the way
  forward, so it ends on a checked proof; the caption takes the focus,
  each stop is said in the live region, and it has no motion of its own.
- **Learn** (D6, D47, D54): a frame of its own colour around the window,
  the panel beside it (below it on a phone), with Tutorial (its opening,
  the nine lessons of D60 as places, and the tour), Practice and
  Reference (the connectives as typed and printed, with their readings);
  with Learn on every action shows its label; off, none of it is on the
  page.
- **About** (D52, D36): what the page is, links to the repository, the
  documentation and the issues, linlog's version from the module, every
  licence served (each linked, and fetched by its test), and that GitHub
  logs every visitor's IP address.
- **Lucide** (D54): the build fails unless `lucide-solid` is the pinned
  version, ISC, with a `solid` export condition and a peer range the
  installed Solid meets.
- **The follow-ups**: a link whose sequent does not parse opens it in
  the field with the parse error marked, as typed; a link's session is
  inflated through a reader that stops at 512 KiB (`UNPACKED_LIMIT`),
  and one past it loads the sequent alone with "Proof not restored" and
  why; a tab shows its sequent as typed, in signs and in Euler Math
  (`a^ par b` as `a⊥ ⅋ b`), as the visual reference draws the tab. An
  intermediate version named the tab by linlog's one-sided printing; the
  typed form replaced it, since the mockup shows `A ⊗ B ⊢ B ⊗ A` there.

### Decisions

- **The deployment is a job of the CI workflow**, as the author's sites
  have it, and the artifact is built by the check job after the checks,
  so a deployment is always of what was checked.
- **Astro's `security.csp` over the sites' own hashing integration**:
  the author's sites fill a header file, which Pages does not have;
  Astro writes the `<meta>` element itself. The check recomputes
  everything in Python, as the sites' does.
- **The tab's sequent as typed**: what the visual reference shows; the
  link keeps the parseable text, since the parser does not read `a⊥`.
- **The examples are Markdown with linlog's session files beside them**,
  and the page replays them with the redo stack, so that "watch" needs
  no machinery of its own.
- **The reference holds the connectives only**: the rules' schemas in
  Euler Math (D47) come with the lessons.

### Departures from the design

- **The service worker is registered with the scope `/`** (departs from
  the design, D49's "the worker's scope is the client's own paths and
  never `/linlog/`"): the page is at `/`, and a worker's scope is a path
  prefix, so no registration includes `/` and leaves out `/linlog/`.
  What D49 asks is held by what the worker does instead: it answers only
  the site's own files and `/`, returns before anything else under
  `/linlog`, never caches a path there, and where the browser has the
  Static Routing API those requests go to the network without waking it.
  `tests/offline.spec.ts` shows the rustdoc's stand-in served from the
  network, never from the worker, and unreachable offline.
- **An example opens as a proof of its own** (departs from the design,
  D7's "replacing the current work after a warning if that work is
  unsaved"): with several proofs at once (D50) nothing needs replacing;
  an example takes the place of the page's empty proof only.
- **Learn's hover explanations are not built** (departs from the design,
  D47's "inside the frame, hover explanations are on everywhere"): the
  rule's schema, the connective's reading and the verdict's reason on
  hover belong with the lessons and the reference's schemas, a content
  session's.

### What is verified

- `nix flake check` of the client, under the machine lock: exit 0 at
  `tmpzrkvn`, the last commit. Two runs before it failed in `e2e` alone
  (at `nnxwqzxw`: the tour's, the offline test's and the several
  proofs' synchronisation, fixed in the tests at `kynzlrlk`, where the
  next run passed).
- Each check alone, in the sandbox, at `pwsnmrot`: `csp`, `licences`
  and `pages-paths` built and passed.
- Playwright, 39 tests against the built site, all passing at
  `tmpzrkvn` twice over without retries in CI's settings (78 runs), and
  114 runs of the 38 before it at `kynzlrlk`; Vitest, 42 tests.
- The `csp` check fails on a build with one hash removed from the
  policy ("index.html: inline code script-src does not allow: …").
- The bundle: 55 KB compressed on the critical path (the page, its
  stylesheet and the island's JavaScript), the fonts beside (D35).
- Signing tested before every commit; every commit is signed.

### The reviewer's findings

One fresh-context reviewer (Opus 5.5, `high`, briefed with conduct.md's
rules on cores and memory, cores 12 to 15, no nix builds, told to report
only what breaks correctness, the design's decisions or a stated
requirement) read the policy, its check and the service worker, and
probed them with two builds, a server that sends Pages' `max-age=600`,
and mutated policies loaded in Chromium. It found two things, each
answered in `tmpzrkvn`:

1. **An update could cache the old page with the new files.** The
   worker's install fetched through the browser's HTTP cache, which
   Pages lets keep a file ten minutes: a new worker installed right
   after a deployment cached the old `index.html` beside the new hashed
   files, and once active served a page whose stylesheet and island
   were gone (404, no hydration) until a later deployment. Holds: the
   probe reproduced it with two builds. Every file is now fetched with
   `cache: "reload"`. The test server sends Pages' `max-age=600` and a
   fresh `x-github-request-id` per response (as Pages does), and
   `tests/offline.spec.ts` checks that the page the worker cached is not
   the one the browser kept; the test fails on the old worker.
2. **The check passed policies Chromium enforces differently**: it kept
   the last copy of a repeated directive where browsers obey the first,
   and read `script-src` and `style-src` where `script-src-elem` or
   `style-src-elem` govern elements. Holds, though the built site wrote
   none of those forms. The check now reads a policy as a browser does;
   the reviewer's five mutated policies (a hash missing, moved to
   `style-src`, a stricter `script-src` first, a `script-src-elem`, a
   `style-src-elem`) all fail it.

Sound by the reviewer's reading and probes: one policy per page, first
in the head before any script or stylesheet, the pre-paint script first
in the body with its hash; nothing the page does blocked (the module
worker, the wasm, the font, the drawings by `innerHTML`, every download,
the worker's registration); the worker never answers or caches `/linlog`
or anything under it, with or without a query, nor `/%6Cinlog/`,
`/linlog.html` or `/linlogx` by mistake, and every subresource of a
rustdoc page passes it by. It noted that an open rustdoc tab, being the
worker's client, delays an update, which breaks nothing; and that the
worker's comment gave the wrong reason for its scope, corrected in
`tmpzrkvn`. Its first build left `.astro/.prerender/` in the client's
directory (ignored, never committed).

### Left open

- Learn's lessons (D56, D60) and its hover explanations (D47), the
  reference's rule schemas: a content session's, each sequent proved or
  refuted by the command first.
- The repository's website field pointing to the client, on the
  author's word, after the first deployment.
- The first push publishes the site; the supervisor pushes after
  review. The Cachix token is the author's to add.
- `linlog interact` prints a goal whose first hypothesis is not at
  position 0 with a stray comma (`goal 2:, 1: A ⊢ 0: A`), seen while
  writing the examples' sessions: a follow-up for linlog
  (`plan/later.md`).

### Repositories and workspaces

- The client: `/home/tux/Projects/own/linlog-prover.github.io`, `main`
  at `uymkopks`; this session's commits `tlsoutzz` to `tmpzrkvn` above
  it, local, for planning to review and push.
- linlog: `../linlog-step32c`, this report's commit on `kzmmlszx`.
