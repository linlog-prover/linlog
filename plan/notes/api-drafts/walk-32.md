# Walk-through of step 32 (the web client) against `plan/notes/api.md`

Read: the design in full; `plan/32-web.md`; `research/32-web.md` and
`research/usability/web-developer.md`; the 58 register entries that name
step 32; `api-drafts/draft-a.md` 3.9 and 7.3.6 (the only place where
`GoalView`, `Request` and `Response` have a shape); `core/src/proofs/
interactive.rs`, `cli/src/interact.rs`, `core/src/export/svg/`. Nothing was
built or run. Result: 0 blocking, 9 friction (items 1 to 9), 9 note (items 10 to 18).

## 1. The first change, sketched

Two commits, in this order (the second needs the first's wasm check).

**Commit 1, "Add linlog-web and its wasm32 check".** Workspace member
`linlog-web` (`cdylib`), `wasm32-unknown-unknown` in `rust-toolchain.toml`,
a flake check that builds `linlog` with `--no-default-features --features
parse,serialize,interactive,latex,typst,svg,rocq` and `linlog-web` for the
target, and runs `wasm-bindgen-test` (R41, R198, R239), `wasm-bindgen-cli`
pinned to `Cargo.lock`. Exports, all JSON or SVG strings:

```rust
// linlog-web/src/lib.rs
#[wasm_bindgen(start)] fn start() { std::panic::set_hook(Box::new(post_panic)); }  // {"code":"panic"}
#[wasm_bindgen] pub fn defaults() -> String;                // Settings::browser() as JSON, wire::LEVEL, the name tables
#[wasm_bindgen] pub fn parse(text: &str, settings: &str) -> String;   // Sequent::parse_within -> sequent form | Error form
#[wasm_bindgen] pub fn prove(text: &str, mode: &str, settings: &str, on_progress: &Function) -> String;
#[wasm_bindgen] pub fn serve(envelope: &str, on_progress: &Function) -> String;  // session requests
#[wasm_bindgen] pub fn check(form: &str, settings: &str, on_progress: &Function) -> String;   // Proof or Disproof
```

`prove` is: `Settings` read (plain `Deserialize`, `deny_unknown_fields`);
`Sequent::parse_within(text, &settings.limits)`; `mode.parse::<Mode>()`;
`prove_within(&s, mode, &settings.search, &settings.limits, |p| clock.poll(p))`;
`serde_json::to_string(&outcome)`. The clock is the client's (5.2):

```rust
struct Clock { limit_ms: Option<u64>, start: f64, next: u64 }
impl Clock { fn poll(&mut self, p: Progress, post: &Function) -> bool {
    if p.done < self.next { return false }
    self.next = p.done + (1 << 16);
    post.call1(&JsValue::NULL, &to_json(&p));                    // {"work","done","held_bytes","phase","item"}
    self.limit_ms.is_some_and(|l| performance_now() - self.start >= l as f64) } }
```

`serve` reads `{"settings": {...}, "session": {...}, "request": {...}}` in
two stages (settings first, then the session through `Within::<Interactive>
::new(&settings.limits)`, 5.5), calls `session.serve(&request, &settings,
stop)` and writes `{"version": 1, "session": {...}, "response": {...}}`
or `{"version": 1, "error": {"code", "kind", "message", "setting"?,
"details"}}` (4.3). The page keeps the session JSON, so `terminate()` plus a
new worker recovers from any trap.

**Commit 2, "Add Request, Response and serve" (core, `interactive` +
`serialize`).** Types from 3.10, 7.2, 7.3 and draft-a 7.3.6, renamed to the
design (`Limits`, `Named`, `GoalId`, `Member`):

```rust
#[non_exhaustive] pub struct GoalView { pub goal: GoalId, pub formulas: Vec<FormulaView> }
#[non_exhaustive] pub struct FormulaView { pub position: usize, pub member: Member, pub text: String,
                                            pub side: Option<Position>, pub rules: Vec<Applicable> }
impl Interactive { pub fn view(&self, id: GoalId, text: &TextOptions) -> Result<GoalView, StepError>;
                   pub fn serve(&mut self, r: &Request, s: &Settings, stop: impl FnMut(Progress) -> bool)
                       -> Result<Response, Error>; }
#[non_exhaustive] #[serde(tag = "request", rename_all = "snake_case", deny_unknown_fields)]
pub enum Request { Goals, Goal { goal }, Rules { goal, position }, Split { goal, step }, Apply { goal, step },
                   Undo, Close { goal }, CloseAll, Proof, Draw, Export { format } }
#[non_exhaustive] #[serde(tag = "response", rename_all = "snake_case")]
pub enum Response { Goals { goals: Vec<GoalView> }, Applied { opened: Vec<GoalId> }, Closed { goal, outcome },
                    Results { results: Vec<GoalResult> }, Drawing { svg: String, goals: Vec<GoalId> },
                    Text { format, text: String }, Proof { proof }, Undone { goal: Option<GoalId> } }
```

JSON (wire level 1; `Request` refuses unknown keys, `Response` is written
only, 7.2):

```json
{"request": "apply", "goal": 0, "step": {"position": 1, "rule": "⊸L", "left": [0]}}
{"response": "applied", "opened": [1, 2]}
{"response": "drawing", "svg": "<svg …>", "goals": [2, 1, 0]}
{"response": "goals", "goals": [{"goal": 1, "formulas": [
   {"position": 0, "member": 0, "text": "A", "side": "input", "rules": []},
   {"position": 1, "member": 1, "text": "A ⊸ B", "side": "input", "rules": [{"rule": "⊸L", "needs": "split"}]}]}]}
```

`Request::Draw` forces `styles.svg.ids = true` before it calls
`export::svg::write(&session.derivation(), …)`; the page binds a click on
`i<n>-<p>` to `goals[n]` and sends `rules` and then `apply`. Not in the
first change: nets (`ProofStructure::from_proof` then `svg::net`), ordinary
logic, PNG and PDF.

## 2. Workarounds

1. **friction. A missing key in a tab reads as the native default.** The
   need: the browser preset `Settings::browser()` (6.5) must hold whatever
   the page stored. Stands in the way: 6.1 and 7.1 (a missing key is
   `Default`) with 5.1 (`Limits::default()` is 1 GiB, 50 million
   occurrences, depth 2048, 6.5 makes `Settings::default()` the command's).
   A page that stores only its overrides, or a settings blob from before a
   new limit exists (`Limits::terms` at 38, the closure matrix at 35), reads
   the unsafe native value for every key it lacks, silently. Fix: a seeded
   reader `wire::Over<'a, T>(&'a T)` that reads a partial JSON on top of a
   base (`Settings::browser()`), in `wire` beside `Within`; or
   `Default for Limits` is `BROWSER` under `cfg(target_pointer_width =
   "32")`. 5.5 (`Within`) is the place; the seed is additive.
2. **friction. Convenience calls are native-sized and nothing marks them.**
   `Interactive::new`, `FromStr for Sequent`, `Forest::new`, `prove`,
   `Proof::derivation` and plain `Deserialize` ("`Within` at
   `Limits::default()`", 5.5) all take the native limits. A single
   `serde_json::from_str::<Proof>` in `linlog-web` lets a few hundred bytes
   unfold to 1.25 GB (R27). Fix: a `clippy.toml` `disallowed-methods` list
   for `linlog-web`, listed in `web.md`; the design's 5.6 table should mark
   the convenience calls as "native defaults, not for a tab".
3. **friction (signature, decide before 0.1.0). `Interactive::derivation`
   is infallible, unbounded and has no view.** 3.10 lists it "unchanged";
   `Proof::derivation` is `Result` and `derivation_within(&ViewOptions,
   &Limits, stop)` (3.9). The drawing of a session therefore bypasses
   `limits.derivation_bytes` (R28), `export::*::write` takes no `&Limits`
   (9.1, 5.6 "a bounded derivation"), and a pasted 500 000-occurrence
   sequent (inside `BROWSER.occurrences`, 1 000 000) is drawn as one
   100 MB SVG. `ViewOptions::{compact, sides}` cannot be applied to a
   session either, and `derivation_ids` (3.10, "drawn inference n is goal
   ids[n]") is valid for the one uncompacted derivation only. Adding
   `Result` later is a break. Fix: `Interactive::derivation(&self) ->
   Result<Derivation<'_>, Error>` (defaults) and `derivation_within(&view,
   &limits, stop)`; `derivation_ids` documented per view, or
   `Derivation::origins() -> &[GoalId]` instead.
4. **friction (signature). `close` drops a proved goal whose graft is
   refused.** 3.10, `close -> Result<Outcome, Error>`; a graft past
   `limits.derivation_bytes` (4 MiB in a tab) is `Error::Refused(Output)` and
   the checked proof is lost, as `cli/src/interact.rs` shows today (it
   builds "the search proved the goal, but the derivation to graft is too
   large" by calling `prove_goal` and `close_with` itself). The tab hits
   this bound often (a DAG proof, a small bound). The web can copy the
   CLI's two calls, which makes `close` pointless for both. Fix: `close`
   returns `Result<Closed, Error>` with `Closed { outcome: Outcome,
   grafted: Result<(), Refusal> }` (the goal stays open, the proof is
   kept for export).
5. **friction. A stateless worker is quadratic over a session.** 3.10 and
   7.3 ("a stateless worker reads the page's session, serves one request
   and posts the new session"); the session JSON lists every inference's
   whole sequent, so a click costs a read with a replay (F21), a
   `Forest::within` and a write of N inferences times n members: 100 steps
   on 10 000 occurrences is about 5 MB of JSON each way (R93's "10 000-
   occurrence states"). The research recommended the reverse: a live
   `Interactive` in the worker, the JSON on the page as the recovery.
   Fix: do not pin statelessness in 3.10; `Response` never embeds the
   session; `Request::Session` returns it on demand (after a step, or
   every k steps); `serve(&mut self)` already serves both. A line in 3.10.
6. **friction. `GoalView` has no signature, and what it needs is not in
   the design.** `view(&self, id, &TextOptions, &ViewOptions)`: the
   formula text must follow `settings.styles.text` (ASCII or Unicode) and
   `view.sides`, else the goal list and the drawing disagree (classical
   mode with `Sides::Two`: `side` is `None` in the list, two-sided in the
   SVG; draft-a's `view(&self, id)` has neither). Cost: each formula's
   `rules` is "what `apply` accepts given the goal's context" (3.10), so a
   naive `view` is O(goal x rules) per position and O(goal²) for
   `Promotion` ("every other formula is `?`"): the contract needs "O(goal)
   per view; the context facts computed once" and a 10 000-occurrence
   test (R93). 3.10 and 5.6 (which lists no `view`).
7. **friction. `rules` hides why a rule does not apply.** 3.10 `rules ->
   Vec<Applicable>` lists the accepted ones only; R91's "met when" asks
   each rule with `Result<(), Refusal>`. A student who clicks `A ⊗ B` on the
   wrong side wants "⊗R needs the formula on the right of ⊢". The only
   way is `apply` on a clone per candidate (what R91 forbids). Fix, all
   additive: `Applicable` gains `refused: Option<StepError>` and `rules`
   an `all: bool` flag, or `Interactive::explain(goal, position) ->
   Vec<(Named, Result<Needs, StepError>)>`.
8. **friction. The recursion depth is a data knob that traps a tab.** 5.1
   (`recursion_depth`, `BROWSER`, `stack_bytes`) with 4.2 and `Reason::
   setting()` (8.1): `RecursionLimit` tells the page that
   `limits.recursion_depth` lifts it, and a panel raises it above what the
   1 MiB link argument holds: a stack-overflow trap, the instance lost (R45's
   "a stop before an overflow" holds only below the stack). The per-level
   figure is a `const` of the library that step 32 must make `cfg(target_
   arch = "wasm32")`; the design gives only the optimized x86-64 number
   (2 304 B), and `core-search.md` has 12 288 B in debug, so a
   `wasm-bindgen-test` run in the dev profile traps at `BROWSER`'s depth.
   Fix: `Limits::within_stack(self, bytes) -> Self` (clamps), used by
   `linlog-web` at the boundary; `stack_bytes` and `recursion_depth_for_
   stack` take `cfg!(debug_assertions)` into account; `Reason::setting()`
   answers `None` where the front end says the stack is fixed (or the
   page greys the key).
9. **friction. The settings panel and the TypeScript types have no source
   of truth.** 6.6 and 7.3: "the web client builds its panel from the
   form". The form is the default values only; the panel needs per field
   its type, range, enum names, the line of documentation (D16) and "read
   by" (6.2 has the column, as prose). 7.4 says a `.d.ts` is `linlog-web`'s,
   and the library has no schema derive, so about thirty types are
   mirrored by hand (`Settings`, `Outcome`, `Error`, `GoalView`, `Request`,
   `Response`, `Statistics`, `Reason`), drifting at every step 33 to 38.
   `Bias`, `Schedule`, `Cadence`, `Compact`, `Sides` have no `ALL`/`NAMES`
   in the design, only `Engine::ALL`, `Mode::NAMES`, `Rule::ALL`. Also
   `Settings` (6.5) has no `ordinary` member for `ordinary::Options` that
   `ordinary::decide` takes. Fix: `Settings::FIELDS: &[Field { key, kind,
   default, read_by, doc }]` (one table for the CLI's `--help`, the harness
   and the panel; a test that it covers every key of `Settings::default()`
   JSON); `NAMES` for every closed word list; an optional `schema`
   feature (schemars) or the pinned fixtures as the contract for the mirror
   in `linlog-web`; `Settings.ordinary`.
10. **note. `memo_limit: usize` is a 32-bit hazard on the wire.** 6.2 (the
    table), R46: one settings file is read by the command and the tab (R49),
    and `4294967296` is a valid number natively and an
    `invalid_option` on wasm32. The memo's slots are `u32` anyway. Fix:
    `u32`, or `u64` clamped where read like `jobs`. `Step.position: usize`
    is harmless.
11. **note. The 32 types are specified by a superseded draft.** 3.10 and 7.3
    send the reader to `api-drafts/draft-a.md` 7.3.6, which names `Bounds`,
    `Rule` for `Named`, a `Progress` without `done` and `item`, and a
    `view(&self, id)` without options. `Request`, `Response`, `GoalView`,
    `serve` are in neither the module tree (2.1) nor the re-exports
    (2.2); `serve` calls `export::svg`, `latex`, `typst` and `rocq` from
    `proofs::interactive`, which inverts the layering (`export` depends on
    `proofs`) and needs `FeatureOff` arms. Fix: one module `session` (features
    `interactive` + `serialize`) in 2.1, the request and response table in
    7.3, not the draft.
12. **note. The clock cadence of 2^16 units (5.2) is not a time bound.** A
    unit is "a stable sequent plus its width in 128-occurrence words" for
    the focused engines but "a failed exact test" for the net engine
    (O(n) each): on a 1 000-occurrence net 2^16 units are seconds, and the
    2 s limit is late by that much. Fix: the unit table states a cost per
    unit (the exact test charged per edge), and `linlog-web` reads the
    clock whenever `p.work >= 1024`; measure in the first session (D16).
13. **note. SVG ids are not namespaced** (9.3, R163). Two inline drawings in
    one document (two `i0`; derivation and net on one page; an undo
    preview) collide, and `getElementById` returns the first. Fix, additive:
    `Style::id_prefix: String` (default empty, so every snapshot stands);
    document that `b`, `d`, `j`, `c` stay reserved. The root-role field
    (`graphics-document`, 9.3 last sentence) is also step 32's, as the design
    says, and additive.
14. **note. Errors are described with a forest the caller does not hold.**
    `Error::describe(&forest)` (4.1) and `Error::form(&self, Option<&Forest>)`
    (4.3: the only place `form` appears) — two names for one job; `prove_
    within` builds its forest inside, so `NotIntuitionistic(ShapeError)` can
    be described only if `linlog-web` builds a second `Forest::within` on the
    error path. No source span exists for an occurrence (hash-consed), so
    a `ShapeError` cannot underline the text; the page shows the subformula
    as text. Fix: one name, and `ShapeError` carries the subformula's text.
15. **note. The envelope cannot be one derive.** `{settings, session,
    request}`: a `#[derive(Deserialize)]` reads `session` with the plain
    impl (item 2). `linlog-web` reads in two stages with `RawValue` and
    `Within::<Interactive>::new(&settings.limits)`. A recipe in `wire`'s docs
    and a test.
16. **note. 3.1 and 10.4 disagree about the builder.** 3.1: "[32 or 34,
    whichever first needs it]"; 10.4 lists it as added by 32. Step 32's
    first version parses text and reads JSON; nothing calls the builder.
    Decision 16 stands; 10.4 should say "34 or the client's first formula
    editor".
17. **note. Three names for one idea.** `Side` (the ⊕ choice) and `Position`
    (input or output) are different types; the wire key `side` holds a
    `Position` (7.4: `side | input, output`), `Named.side` and
    `FormulaView.side` too, and `Step.position` is an index. A TypeScript
    reader sees `side: "input"` and `position: 1`. Rename `Named.side` and
    `FormulaView.side` to `position` of the type, or the Rust type `Side`
    to `Branch`, now: a field rename after 0.1.0 is a break.
18. **note. "A suspendable engine can replace `Decide::decide` without a
    change of the front door" (5.3, 8.2) over-claims.** A search that
    returns between slices needs a value to resume, which `prove_goal ->
    Result<Outcome, Error>` and `close` cannot hand back. It is an additive
    function later (`prove_resumable -> Search`), so nothing breaks; the
    page uses `terminate()`, and the sentence should say so. Related: the
    NFC dependency of HD3 (3.1) adds to the `.wasm` (a size budget, R49),
    to be measured with `twiggy`.

## 3. Register entries naming step 32

Met or placed (section): R1 (6.2, 7.3), R2 (3.12, 7.3, 8.1), R3 (3.12,
7.3), R4 (3.9, 7.3; item 3 for the compact view), R5 (3.10, 7.3), R6
(7.1), R11 (3.13, 7.3), R247 (7.1), R21 (5.3), R22 (5.6), R23 (3.8), R24
(3.13), R25 and R26 (3.11, 9.1, boxes at 33), R27 (5.5; item 2), R29
(5.1, 3.3), R30 (5.2; item 12), R243 (5.2), R250 (3.14), R41 (10.4), R42
(10.4, P9, by convention; the grep or lint of its "met when" is not
placed), R47 (5.3), R116 (6.2), R129 (4; item 14), R130 (4.1, the panic
hook), R141 (6.4), R142 (5.1, 6.5; item 1), R148 (6.3), R183 (6.6), R241
(3.5), R244 (3.4), R245 (3.6, 10.1), R92 and R94 (3.10), R93 (3.10;
items 5 and 6), R163 (9.3; item 13), R44 (9.1, to be recorded by 32).

Partly: R28 (item 3: the drawing of a session is unbounded), R45 (5.1,
5.4; item 8), R46 (8.5 makes `Statistics` `u64`, 10.4 the tests; item 10),
R49 (6.2, 6.4; `Settings.batch` and `jobs` are in the panel), R91 (3.10;
item 7), R190 (7.3 and decision 13 put the options with the front end, not
in the outcome: R190's "met when" is changed, not met), R239 (10.4 names
the member and the check, not the flake package `.#linlog-web`).

Not placed: R164 (a student-built net needs link and unlink requests on a
`ProofStructure` JSON; 10.4 lists no net request, though every library call
exists), R171 (the Typst export of a proof deeper than about eleven
inferences, 9.x is silent; the page's Typst export fails when compiled),
R198 (core's tests under wasm: only the limit tests of R46 are in 10.4),
R226 (the `web.md` rules file, `claude-infra.md`).

## 4. What fits well

- `Member`, `GoalId` and the offset rule (3.4, P2): the page's ids are
  integers, `rules`/`apply` positions and `i<n>-<p>` stay one scheme at 33 to
  38, and nothing the client stores changes.
- `Limits` and `Progress` (5.1, 5.2) are exactly what a wasm caller needs: a
  deterministic work count, `held_bytes` and `phase` for the progress
  message, `Schedule::Turns` for the same counters on wasm, and the stop as a
  closure with no clock in the library.
- Error `code`/`kind`/`setting`/`details` (4.3), the version-first levels
  with refused unknown keys on commands (7.1), and `Settings`/`Styles` in the
  library (6.5): the page needs no name table of its own for modes, engines
  or rules.
