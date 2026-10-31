---
paths:
  - "web/**"
  - "modules/web.nix"
---

# linlog-web: the WebAssembly bindings

Loaded when a file of the bindings (`web/`) or their flake module is read
or changed. The plan of the web front end is `plan/web/README.md`; the
client is a repository of its own, which takes this repository's flake
as a pinned input and builds nothing of Rust.

## Layout

`web/src/calls.rs` the calls, plain Rust over JSON text, each reading
its documents and the settings and writing what the page shows;
`lib.rs` the `#[wasm_bindgen]` exports, one line each over `calls`,
named as in Rust but `export`, which JavaScript reserves (`export_as`);
`tab.rs` the tab's defaults and the overlay of the page's settings on
them; `failure.rs` the error's JSON a call throws; `build.rs` the stack.
`web/tests/calls.rs` one test per behaviour the page relies on, each
`#[wasm_bindgen_test(unsupported = test)]`, so the same test runs
natively (`cargo test`, the gate) and under Node (`web-wasm-tests`).
`about.toml` and `about.hbs` are cargo-about's configuration and template
for `THIRD-PARTY-NOTICES.txt`.

## What the code relies on

- **Nothing is kept between calls.** The page holds the session
  document and the settings and sends them with every call; a worker
  the page stops (its only cancel: the module reads no clock, and every
  call's stop is `never`) loses nothing. A handle kept in the module
  would save the read of the session per call (linear, and checked by
  `wire::upgrade` as a loaded session always is), and would have to be
  rebuilt after every stop.
- **The settings are read over the tab's** (`tab::read`): the page's
  JSON is laid over `tab::defaults()` key by key, objects merged and
  every other value replaced, then read as `Settings` with its
  `deny_unknown_fields`. So `{}` is the tab's settings, never the
  command's (whose memory bound is a gibibyte), and a misspelt key is
  refused as the command refuses it.
- **Errors are the library's JSON**, `Error`'s written form or
  `describe`'s with the session's formulas; the bindings' own refusals
  (`Failure::json`, `Failure::refused`) take the shape and a `code` the
  library uses (`json`, `unknown_name`). A new call returns `Failure`,
  never a message of its own.
- **A goal formula's `reading`** (intuitionistic mode alone, absent in
  classical mode) is `Reading::formula(o)`, the crate's two-sided printer
  for one occurrence in its position, and in intuitionistic mode `text`
  is the same (`Interactive::formula` writes under the session's
  reading): no text of an intuitionistic sequent is one-sided, and a
  typed intuitionistic sequent is read in ILL's syntax alone, so the
  one-sided text could not be read back there. It is a key of a call's
  answer, no stored document, so no wire level (`wire.rs`: a key an older
  reader may ignore raises none).
- **A typed sequent of an intuitionistic mode** (commutative; the
  Lambek calculus keeps its own reading) is read by `sequent_in`/`ill_of`
  as the settings' `input.intuitionistic` says, strict by default
  (`IllSequent::parse_within`); JSON may be the intuitionistic sequent's
  document (told by its `hypotheses` key) or a one-sided sequent read by
  `IllSequent::read_one_sided`. `read` has no mode and stays as it was;
  `read_as(input, mode, settings)` is `read` for a mode, its `text`
  two-sided, and is what the page calls for an intuitionistic proof.
  `start` and `prove` read their sequent the same way.
- **A typed sequent of a commutative classical mode is read as written**
  in `start` (`ClassicalSequent::parse_within`, `Interactive::of_classical`),
  so the session carries its record (`"written"`) from the first call and
  the page's "Two-sided" switch can turn at any point; `export` and
  `draw` draw it two-sided when `view.classical` is `"two"`, a session
  without a record (JSON) through `ClassicalSequent::read_one_sided`.
- **An intuitionistic net is drawn essential** (`draw_net` sets
  `styles.svg.essential` in an intuitionistic mode): the plain drawing of
  the one-sided net writes classical formulas.
- `web/tests/calls.rs`'s `no_intuitionistic_text_writes_a_classical_symbol`
  is the bindings' share of the property that no intuitionistic output
  has a classical symbol (the library's and the command's are their
  own); a new call that writes text of a sequent joins it.
- **`Step` and `Applicable` have no wire form in the library**: `StepForm`
  (`{"position", "rule", "left"}`) and `Offered` (`{"rule", "split"}`)
  are the bindings', the rule by its `Named` name. When the library
  gives them one (at level 1), the bindings take it.
- **A hint is the first steps of the goal's proof**: `hint` runs
  `prove_goal` on the goal (the session untouched) and reads the
  derivation of that proof alone (never the session's, which a large tree
  would have refused), from its root up to and including the first
  inference that is not structural (`first_steps`): a contraction at the
  root comes with the rule that uses both copies, since a search on the
  contracted goal may weaken the copy again, and hints that stop at the
  contraction contract and weaken in turn forever (a reviewer's finding).
  Each step is the principal's position, and for `⊗` the left premise's
  members less the left subformula, for Mix the left premise's members,
  matched to positions of its goal; the goal of a later step is the one
  the step before opened, whose members are the premise's in the same
  order. `following_the_hints_proves_a_sequent` pins it.
- **`close_all`'s budget is the searches' work**: each goal is closed
  with what the searches before it left of `limits.work` as its own, so
  the library itself says `work_limit` where the budget ran out (a stop
  that fires would say `stopped`, which the page cannot tell from a
  cancel). What a search spent is its `statistics.work`; the polls are
  no measure, since work between the last poll and the end is not
  reported. The check of each proof has the same remainder as its own
  bound, and the refuter its `classical_work`.
- **The integers at the boundary**: a goal and a position come from
  JavaScript as numbers (`f64`), which `calls::index` takes only as an
  integer from 0 to `u32::MAX`: as a `u32` parameter wasm-bindgen would
  take any number modulo 2³² (`2 ** 32 + 3` as 3, `1.5` as 1, `NaN` as
  0). A `u32` widens to `usize` on every target. A position inside a
  step's JSON is a `usize` read by serde_json, which refuses a number past
  `usize::MAX` on wasm32 rather than wrapping it
  (`a_position_past_32_bits_is_refused`). Every count the library writes
  is a `u64`, exact in JavaScript to 2⁵³ (`wire`'s rule).
- **The settings' overlay refuses a non-object where the tab has an
  object, but for a string**: serde reads an array as a struct's fields
  in order and fills the rest from `Default`, the command's values, so
  `[]` would have been the command's settings, a gibibyte of memory and
  a recursion depth the host's stack does not hold. A string is a name
  an option reads in place of its fields (`styles.svg.font`:
  `"stix-two"`, `a_font_is_chosen_by_its_name`), or is refused by serde.
- **`render` takes one font's file**: the page passes the file of the
  font its export's `styles.svg.font` names, and no other font sets the
  drawing's text (`an_export_is_set_in_the_font_the_settings_name`).
- **The stack**: `build.rs` links the module with an 8 MiB stack
  (`-zstack-size`, read back as `tab::STACK_BYTES`), but the host's own
  stack, which no page can set (about 1 MiB in Node and in a browser),
  runs out first: each wasm frame is a native frame too. Measured in Node
  on the deepest levels, a Mix of pairs: the optimized module reached
  768 levels and not 1 024, the debug one 512 and not 768. Hence
  `tab::RECURSION_DEPTH` 384, half the optimized depth, and
  `a_search_at_the_recursion_limit_stops_there` runs the three paths
  with the largest levels (copies of a `!`, nested `&`, a Mix of pairs)
  at the limit under Node, in the debug build the check uses. A change
  that grows a level's frames shows there first.
- **The invertible rules** (`interactive.invertible`, off in the tab's
  settings as in the library's): `start` applies them to the root and
  `apply` to the goals its step opened, `opened` then the goals they
  left open and `inverted` how many steps they took; `invert_all`
  applies them whatever the settings say, which is what turning the
  toggle on mid-session needs. A run is bounded by the smaller of
  `limits.memory_bytes` and `limits.derivation_bytes` (`inverting`), so
  that it never makes a session the page could not draw. The view's
  `invertible` lists every closed inference whose rule is invertible on
  its conclusion, whoever took it (the session does not record who):
  the page's undo with the toggle on goes past them to the nearest step
  that is not one. A replay must run with the setting off, or every
  replayed step would apply the invertible rules again and the recorded
  ones would find their goals closed. A hint's `invertible` is the
  library's judgement of its last step, for the ladder's first rung.
- **`unsafe_code` stays forbidden**: the lint does not see the unsafe
  code `#[wasm_bindgen]` generates, so the workspace's `forbid` holds for
  the bindings too.

- **Nets**: `view`'s `net` and `draw_net` gate on `nets_exist`, which
  takes `MellStructure::exist` for a sequent with exponentials, so a
  finished MELL proof, classical or intuitionistic, draws its net with
  boxes (`svg::mell_net`, ids `o<n>`, `v<k>`, `b<k>`, `l<x>-<y>`; with
  `styles.svg.essential` an intuitionistic one as its essential net with
  boxes, the same ids); an affine one is refused with `no_nets`.
- **A net's look is `styles.svg.net_look`**, read over the tab's like
  every setting: the name `"textbook"` or `"linlog"` (a string, which the
  overlay lets through where the tab has an object, as for the font), or
  an object of `premises`, `conclusions` and `why_not`, each left out
  keeping linlog's (`a_proof_with_exponentials_draws_its_net_with_boxes`
  checks both forms). The tab's default is linlog's look; nothing in
  `tab::defaults` sets it. Under the rules a compact `?` vertex whose
  one premise is a wire has no circle, so no element carries its id.

- **`cut(session, goal, formula, left, settings)`** reads the cut
  formula's text with `CutFormula::parse_within` in the session's mode
  and the settings' `input`, `left` a JSON array of positions, and
  answers as `apply` does (the invertible steps after it under the
  setting); `no_intuitionistic_text_writes_a_classical_symbol` cuts every
  session on its goal before closing it.

- **A session's cuts replay from the view's `cuts`**: each cut's
  inference and its formula as typed in the mode's syntax (the reading's
  text in an intuitionistic mode, `Interactive::formula_as_written` in a
  classical session that keeps its record, else one-sided), which `cut`
  reads again; `start` on the session's sequent leaves its cut formulas
  out (`Interactive::within`), so the replayed cut adds the same terms
  and the same occurrences (`a_cut_opens_two_goals`).
- **Cut elimination** (`cuts`, `cut_step`, `eliminate`; `net_cuts`,
  `net_cut_step`, `net_eliminate`) takes the finished session and the
  proof or net the page holds, `""` for the session's own: the session
  gives the mode, and for a classical proof drawn as written its
  record, which every proof of the elimination shares since a step
  keeps the forest (`proof_in` refuses a proof of another sequent with
  `foreign_proof`, as the record is indexed by its occurrences). Undo is
  the page's: it keeps the proof or net before each step, and a step of
  `eliminate` is taken again by `eliminate` with `cut.steps` one fewer,
  the strategy being deterministic.
- **A cut's ids in a derivation's drawing** come from
  `Inference::node`, the proof node an inference shows, matched against
  the cut's node (`Drawn::showing`); in a two-sided classical drawing
  through `TwoSided::shows`, a negation step showing no node. A shared
  cut node is several inferences. Node ids do not carry over from a
  proof to the next, so the ids of a step are those of the drawing of
  the proof it was taken on.
- **The net of an elimination** follows the command's `net_of`
  (`Net::of`): with boxes for exponentials and where `ProofStructure`
  refuses with `Placement::Cut`, which a session's own proof never
  makes (each cut adds a cut formula) but a proof an elimination on
  terms made may: hence the net calls take a proof's document too. In
  an intuitionistic mode a net with cut links is drawn under the
  reading as `draw_net` draws a cut-free one (the library draws a cut
  link's two trees as an output and an input one), so no step writes a
  classical symbol: `no_intuitionistic_text_writes_a_classical_symbol`
  checks every step's drawing on the net too. A MELL step renumbers the vertices, so a net's ids are those of the net
  the step was taken on, and the vertex ids of `id` follow
  `svg::mell_net`'s rule (`o<n>` for an occurrence's only vertex, `v<k>`
  otherwise, `b<k>` for the box a box entered).
- **The drawings of `eliminate` and `net_eliminate`** are bounded by
  `cut.drawn_bytes` together (`Drawings`), the command's rule: none
  after the first that would pass it, `undrawn` naming that step. A
  drawing its own bound refuses is `null` with `refused`, and the steps
  go on drawn; the drawing of the proof or net reached is the view's,
  outside the bound, as the command writes `FILE` anyway.

- **First-order sessions**: a step's `witness` is text the session
  reads (`Interactive::witness`, the session's letters for its
  eigenvariables), left out for an open one; `rules` writes `"witness":
  true` only where a rule takes one, so propositional answers are as
  before. A goal formula's `reading` is the session's text (the reading
  of the occurrence alone would print an instance's open formula,
  `p(#0)`), and a cut's formula of the elimination calls an instance's
  text (`member_in`, `Proof::formula`).
- **The view's `opened`** lists every closed quantifier step with the
  term it took, as taken (`Interactive::opened_term` reads the table,
  not the shown one: an open witness an axiom filled is still `null`),
  because the page rebuilds each step from the session and replays: a
  witness replayed as the term that filled it would make other members
  than the open one did. Eigenvariables are numbered in the order the
  `∀` steps are taken, so a replay that leaves one out renumbers the
  later ones; the page maps a recorded witness's letters by the `∀`
  inferences' entries here. The view's `forest` (a session with
  quantifiers alone) lets the page read a member's occurrence off the
  session's `instances`, since a replay renumbers the members past the
  forest and only the occurrences stay.
- **`hint` at first order** closes the goal in a clone of the session
  (`Interactive::close`, which takes a goal of instances through its
  written sequent; `Goal::new` refuses members past the forest) and
  reads the steps off the clone's derivation above the goal
  (`read_steps`, as `first_steps` does for a propositional proof). An
  `∃R`/`∀L` step carries the witness the search chose
  (`Interactive::opened_term`) only where the session reads it: a
  witness nothing constrained is a constant the search made in the copy,
  so it stays open (`following_first_order_hints_proves_a_sequent`).
- **The tab's `limits.terms` is its occurrences** (`tab::TERMS`): the
  library's fifty million would pass the tab's memory before a session's
  table refused a step. `search.witness_depth` and `search.instances`
  are the library's.

## The build (`modules/web.nix`)

- The `wasm-bindgen` CLI is nixpkgs' `wasm-bindgen-cli_X_Y_Z` for the
  version of the `wasm-bindgen` crate in `Cargo.lock`, which the
  manifest pins with `=` (and `wasm-bindgen-test` with it, which pins
  the crate exactly): a module the CLI of another version writes does
  not load. `web-bindgen` compares the CLI's `--version` with the lock.
  A bump of the crate needs the nixpkgs in `flake.lock` to have its CLI;
  otherwise the flake fails at evaluation, naming it.
- The package is `cargo build --release` for wasm32, `wasm-bindgen
  --target web`, then `wasm-opt -O3`, with `LICENSE` and
  `THIRD-PARTY-NOTICES.txt` (cargo-about, `--frozen`, the target's
  crates only, from `about.toml`'s accepted licences, which are
  `deny.toml`'s). A crate whose licence is not listed fails the package.
- `web-wasm-tests` runs `cargo test --target wasm32-unknown-unknown`
  with `wasm-bindgen-test-runner` in Node, which only finds
  `#[wasm_bindgen_test]` functions; `web-core-wasm32` runs the library's
  own unit tests of the checker, the size estimate, the memo and the
  memory account on `wasm32-wasip1` under wasmtime, since plain
  `#[test]` functions have no runner on the web target. On WASI a panic
  aborts the run: a failing test there is found by running each alone.
- Locally: `nix develop` has the CLI, `wasm-opt`, Node and wasmtime;
  `CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER=wasm-bindgen-test-runner
  cargo test -p linlog-web --target wasm32-unknown-unknown` and
  `CARGO_TARGET_WASM32_WASIP1_RUNNER="wasmtime run --" cargo test -p
  linlog --lib --target wasm32-wasip1 -- proofs::check` run the two.

## Extension points

- A new call: a function of `calls` over `&str` returning
  `Result<String, Failure>`, one export in `lib.rs`, a row in
  `plan/web/README.md`'s table, and a test in `web/tests/calls.rs` if
  the page relies on a behaviour of it.
- A new default for a tab: a constant of `tab` with the reason, set in
  `tab::defaults()`, and the table of `plan/web/README.md`.
