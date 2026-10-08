# Research for step 32: the web front end

Written 2026-10-08 against the snapshot of the code of that day. Sources
are cited as [Sn] and listed in section 5. "(inference)" marks my own
conclusions; everything about linlog's code names the file and item read.

## 1. The problem and the state of the art

**The problem.** A page proves sequents by clicking, with nothing
installed and nothing sent to a server (`plan/32-web.md`, Goal). So the
Rust core runs in the browser as WebAssembly, one long search must not
freeze the page, and the user must be able to stop it.

**Definitions that matter.**

- *wasm32-unknown-unknown*: the Rust target for the browser. Tier 2;
  `core` and `alloc` work fully, much of `std` returns errors, and
  `std::thread::spawn` panics [S5]. `Instant::now()` panics there too;
  `web-time` 1.1.0 (2024-03-01, MIT/Apache-2.0) replaces it with
  `performance.now()` [S20]. Panics abort by default [S5].
- *The stack*: rustc links with `-z stack-size=1048576` (1 MiB) and
  `--stack-first`; link arguments override it, and an overflow traps
  instead of corrupting static data [S6].
- *Memory*: wasm32 addresses at most 4 GiB [S17]. Memory64 is in Wasm
  3.0 (complete 2025-09-17; the web caps it at 16 GB) [S15], in Chrome
  from 133 and Firefox from 134, in no Safari [S16].
- *Threads*: `SharedArrayBuffer` needs a secure, cross-origin isolated
  document [S9], which takes the headers `Cross-Origin-Opener-Policy:
  same-origin` and `Cross-Origin-Embedder-Policy: require-corp` [S10].
  GitHub Pages cannot set them; GitHub gave no date [S11].
  `coi-serviceworker` (MIT) adds them from a service worker, reloading
  the page on the first visit, from the same origin, over HTTPS [S12].
- *Workers*: `Worker.terminate()` stops a worker "at once" [S13], the only
  hard stop of a synchronous computation without shared memory
  (inference). A task over 50 ms is a "long task"; "Don't block the main
  thread" [S14].

**The toolchain in October 2026.**

| tool | state | source |
|---|---|---|
| wasm-bindgen | 0.2.129 (2026-09-25), MIT/Apache-2.0, in the `wasm-bindgen` organisation since rustwasm was archived (2025-09); breaking changes land in 0.2.x releases (0.2.122, 0.2.123) | [S1][S2] |
| wasm-pack | 0.15.0 (2026-05-15), under `wasm-bindgen/wasm-pack` | [S3] |
| trunk | 0.21.14 (2025-05-08); 0.22.0-rc.2 is a pre-release | [S4] |
| wasm-bindgen-test | runs on Node.js by default, and in a browser or a dedicated worker with `wasm_bindgen_test_configure!` | [S38] |
| wasm-bindgen-rayon | 1.3.0 (2024-12-21), Apache-2.0; needs a pinned nightly, `-Z build-std=panic_abort,std`, `+atomics,+bulk-memory`, `--target web`, and cross-origin isolation | [S8] |
| serde-wasm-bindgen | MIT; claims "much smaller code size overhead than JSON" and faster conversion; `u64` maps to a JS number only within the safe range unless set to `bigint` | [S19] |
| tsify | 0.5.8 (2026-08-23): TypeScript types from Rust types | [S39] |
| CLI and crate | the `.wasm` schema version must equal the CLI's exactly, a known trap with nixpkgs' CLI | [S37] |
| code size | `opt-level` `"s"`/`"z"`, LTO, `wasm-opt` ("another 15-20%"), fewer panics and `format!`, `twiggy` to profile | [S18] |

*wasm32-wasip1* (Tier 2, `std`, thread spawns return an error [S7])
under a runner such as wasmtime could run linlog's own tests at 32 bits
without a browser (inference; not tried).

**UI and rendering.**

| choice | state | source |
|---|---|---|
| Leptos (Rust, fine-grained reactivity) | 0.8.22 (2026-10-05), MIT; 0.9 in beta | [S21] |
| Dioxus (Rust) | 0.7.10 (2026-07-30), MIT/Apache-2.0; 0.8.0-alpha.1 | [S22] |
| MathML Core | in Chrome since 109 (2023), and already in Gecko and WebKit (not in the source) | [S26] |
| KaTeX | 0.19.0 (2026-10-01), MIT | [S23] |
| MathJax | 4.1.3, Apache-2.0; speech output and an expression explorer for screen readers | [S24] |
| typst.ts | Apache-2.0; compiles and renders Typst in the browser through two wasm modules, a compiler and a renderer | [S25] |
| SVG accessibility | an `img` role makes child structure presentational and allows no interactive children. Graphics-ARIA's `graphics-document` (W3C Recommendation, 2018) keeps children meaningful and "may include links or other interactive widgets" | [S27] |
| offline | service workers, "to enable the creation of effective offline experiences", in secure contexts only | [S28] |

**Existing implementations.**

| tool | what | language, where the logic runs | licence, release |
|---|---|---|---|
| Click & coLLecT [S30][S31] | interactive LL prover for teaching, inspired by Logitext; LaTeX and Coq (NanoYalla) export | OCaml backend (opium); `auto_prove_sequent.ml` in the backend, so search is likely server-side (inference) | LGPL-2.1; no releases tagged |
| The Incredible Proof Machine [S32] | graphical natural-deduction proofs, incredible.pm | Haskell core compiled to JS by GHCJS, plain JS UI, all in the browser | MIT; no releases tagged |
| llprover (Tamura, 1997) [S33] | cut-free two-sided LL prover | CGI, server-side; probably offline now | not stated |
| jsCoq [S34] | Coq in the browser, "no servers" | js_of_ocaml | (not checked) |
| lean4web [S35] | Lean 4 editor | Lean on the server, in Bubblewrap (not in the source) | (not checked) |
| Z3's JS package [S36] | SMT in wasm | needs threads, so COOP/COEP; points GitHub Pages users to the service-worker workaround, not for complex apps | (not checked) |

None runs a linear-logic search in the browser with a checked proof
term; Click & coLLecT needs its server (inference from [S31]).

## 2. What the step needs

**Architecture** (recommended). A dedicated worker holds the wasm
instance and the live `Interactive`; the page keeps the state's JSON
after every step and shows the SVG the worker returns. Why: the engine
runs synchronously, so it cannot share the main thread [S14]; a panic
aborts [S5] and an overflow traps [S6], leaving the instance unusable
(inference); `terminate()` is the only hard stop [S13]. With the JSON on
the page, a crash or a stop costs a new worker and one replay
(`TryFrom<Interactive> for State` in `serialize/interactive.rs` calls
`from_parts`, linear in inferences times sequent length), never the
student's proof.

**Cancellation and progress without threads.** Three tools: (a) a work
budget counted in the stop closure; (b) a deadline from
`performance.now()` read in the bindings, keeping the clock out of the
core (D11) [S20]; (c) `terminate()` and respawn as "force stop".
Progress is a `postMessage` from the stop closure; a busy worker can
send but not receive (inference). A cooperative stop button needs the
search to return to the event loop between slices, since there is no
shared flag without `SharedArrayBuffer` [S9]: a second reason for the
suspendable engine of the prompt's question 3 (inference).

**Threads: no**, as `plan/32-web.md` fixes. They need nightly and
`build-std` [S8] (the repo pins `stable`) and headers Pages lacks [S11].
The cost is the default bias's two searches in restarting turns
(`search/focus/schedule.rs`, `turns`, `TURN_GROWTH` = 4,
`BACKWARD_SHARE` = 2), up to five times the better search.

**Stack.** At the default recursion limit (`Options::DEFAULT_RECURSION_LIMIT`
= 2048) `Options::stack_size()` computes 2048 × 2304 B ≈ 4.5 MiB in
release and returns its floor of 8 MiB (`search/mod.rs`), over rustc's
1 MiB [S6]. Raise it with `-C link-arg=-zstack-size=…` in `linlog-web`'s
build, measure the per-level figure under wasm (it was measured on
x86-64), and test a search at the limit inside wasm (inference).

**Memory.** wasm32 caps at 4 GiB [S17] and a tab has less. As far as I
know linear memory never shrinks, so a worker keeps its peak until
terminated (inference; not verified). Recommend tab defaults such as
`memory_limit` 256 MiB, `occurrence_limit` about 10⁶, `ViewOptions::limit`
4 MiB (inference; to be measured), and a fresh worker after a
`MemoryLimit`. Memory64 is no way out while Safari lacks it [S16].

**Bundle size.** Features `parse`, `serialize`, `interactive`, `svg` and
the text exports (`latex`, `typst`, `rocq`); not `png` or `pdf` (resvg,
krilla) at first, since a browser prints SVG (inference). Then
`opt-level`, LTO, `wasm-opt`, `twiggy` [S18]. No size of linlog's
`.wasm` is known yet.

**Wire format.** JSON strings through `serde_json`, the forms
`core/tests/serialize.rs` pins: one format for CLI, files and web,
testable without a browser. serde-wasm-bindgen is smaller and faster
[S19], but the payloads are kilobytes (inference).

**Client technology** (deferred by the step). Recommend TypeScript over
the worker, no framework or a small one. The logic, layout and drawing
are Rust already (`export::svg`), so the UI is a sequent box, goal
lists, a rule menu and buttons. A Rust framework adds a second wasm
module on the main thread, and Leptos and Dioxus both have breaking
majors in pre-release [S21][S22]. TS also fits D22's split, which a Rust
client strains (the caveat in `plan/32-web.md`). Against it: the author
reads Rust (inference).

**Formulas.** Derivations and nets as linlog's SVG (D12); goals and menus
as HTML text in Euler Math from the library's printer. MathML Core [S26]
is optional; KaTeX and MathJax add nothing the SVG lacks, though
MathJax's speech [S24] is a later candidate; typst.ts [S25] would
preview the Typst export at the cost of two wasm modules (inference).

**Accessibility.** `export/svg/mod.rs` writes `role="img"` with `<title>`
and an optional `<desc>` (`Style::description`). An `img` may not have
interactive children [S27], yet the targets `i<n>-<p>` are children, so
an interactive drawing needs a `graphics-document` root and focusable
targets (inference from [S27]), and each goal formula an HTML button.

**Offline.** A service worker caching page, wasm and font [S28]; optional
at first, and the slot `coi-serviceworker` would take [S12].

## 3. What linlog's library must offer

**Data model.** `Sequent` (`sequents/mod.rs`) reads from text
(`FromStr`) and JSON (`serialize/sequents.rs`, `{terms, ids,
var_dict}`). `OccId`, `InfId` and `TermId` are `u32`, safe in JS.

- *Missing*: `ParseError::span` (`errors/parse.rs`) is a byte range, and
  JS strings index UTF-16 units, so a non-ASCII sequent (`⊗`, `⊢`)
  underlines the wrong place. Add a char- or UTF-16 span to the wire
  form, or document that bytes must be converted.
- *Missing*: a JSON form of `ordinary::Sequent` and
  `ordinary::Derivation` (follow-up 1 of `plan/reports/25-ordinary-logic.md`).
- *Missing for D17*: terms and binders do not exist yet. Keep the JSON
  forms' tags open (an object per term already allows new tags).

**Occurrence bound on entry.** `Interactive::new`
(`proofs/interactive.rs`) and every `Deserialize` use `Forest::new`,
that is `Forest::DEFAULT_LIMIT` = 50 million occurrences, and "no way to
pass another" (`.claude/rules/core-search.md`, the memory bound). A tab
needs a smaller bound on the sequent and on a loaded state.
**Needed**: `Interactive::within(&sequent, mode, limit)`, and a
`from_json_within(json, limit)` (or a `DeserializeSeed`) for states,
proofs and nets. Until then, the bindings must check
`Sequent::occurrences()` before building anything.

**Proof term and checker.** `Proof` (`proofs/mod.rs`) has `check`,
`check_within(mode, memory)` and JSON (`serialize/proofs.rs`). It is
enough for the web. On 32 bits, the checker's `MOST = (u32::MAX - 1)
as usize` (`proofs/check.rs`) is the one place written for a 32-bit
`usize`. Each of the others needs its sentence and its test at the
limit under wasm32, as the step prompt says.

**Interactive proving.** `Interactive` has `new`, `goals`, `goal`,
`reading`, `rules`, `apply(goal, position, rule, left)`, `split_passes`,
`undo`, `close(goal, &Options, &ViewOptions, stop)`, `close_all`,
`derivation`, `derivation_ids` and `proof`, and its JSON is pinned. What
is missing or in the way:

- `Refusal` (the reason `apply` and `rules` refuse) has `Display` but no
  wire form. Give it a stable code plus a message, so the page can grey
  out or explain without parsing English.
- No *view* of a goal: the client needs, for each open goal, each
  formula's text (Unicode and ASCII), its side under `Reading`, and the
  rules `rules()` offers. Today it would make one call per position
  across the boundary. **Needed**: one serializable `GoalView` (or
  `Interactive::view()`), produced by the walk in `sequents::fmt`.
- The click mapping: SVG ids `i<n>-<p>` count the *derivation's*
  inferences, and `derivation_ids()` maps them back to the state's
  `InfId`. Return the map together with the SVG, or emit state ids in
  the SVG, so the client cannot mix the two numberings up.
- `close` runs the check and builds the derivation unpolled
  (`core-search.md`, "Not polled"). That is fine at today's sizes, but
  `ViewOptions` must be the tab's, with `Size::height` asked before
  building, as the prompt says.

**Engine interface and dispatch.** The front door is `prove`,
`prove_until`, `prove_goal` and `engine_for` (`search/mod.rs`), with the
crate-private `Decide` behind them. The engines poll a `Stop`
(`search/mod.rs`, `Stop::fired(work)`), but the public stop is
`FnMut() -> bool` with no argument.

- **In the way**: the rule says a condition "must not ration its own
  work by counting polls" (`core-search.md`). Without a timer thread,
  a wasm caller can only read `performance.now()` at every poll, a call
  across the JS boundary millions of times a second (inference), or
  count polls, which the rule forbids. **Needed**: a public stop that
  receives the work done since the last poll (the `work` that
  `Stop::fired` already has), such as a trait `Stop { fn poll(&mut self,
  progress: Progress) -> bool }` with `Progress { work, nodes }`, kept
  for `FnMut() -> bool` by a blanket impl. Then the bindings can read
  the clock every N units of *work* and post progress on the same
  schedule.
  `Statistics` exists only in the final `Outcome`, so a live readout
  needs the counters in `Progress` too.
- A suspendable search (return between slices, resume later) would give
  cooperative stopping and fair turns on one thread; it is the prompt's
  question 3. Leave room for a `Search` value with `step(budget) ->
  Option<Outcome>` (inference).
- `Options::stack_size()` uses x86-64 figures (`PER_LEVEL`): add a wasm
  figure, or document that the link argument must match
  `recursion_limit`.

**JSON wire forms.** Present: `Sequent`, `Proof`, `ProofStructure`,
`Interactive`, `Outcome` (written only), `Fragment`, `Mode`, `Rule`,
`ViewOptions`, `Size`, and the export options (`svg::Style`, `font.rs`,
`latex.rs`, `typst.rs`, `rocq.rs`, `proofs/fmt.rs`, `proofs/style.rs`).

- **Missing**: `search::Options` has no serde (`search/mod.rs`, private
  fields, no derive), though D15 and assessment 3.14 name it first. Give
  it a `default`, `deny_unknown_fields` form like the export options,
  with `jobs` and `pool` absent without `parallel`.
- **Missing**: `Error` (`errors/mod.rs`, 35 `#[error]` messages) has no wire form.
  It needs a code, a message, and the data a client needs (the span of
  a parse error, the limit hit).
- **Missing**: the settings object `Styles` lives in the CLI
  (`cli/src/style.rs`). Report 22 names it as the web's settings, so move
  it into the library or into `linlog-web`, one key per format.
- **Numbers beyond 2⁵³**: `JSON.parse` rounds integers above 2⁵³
  (inference; standard IEEE-754 doubles). Affected: `Size`, whose counts
  saturate at `u64::MAX` (`proofs/size.rs`); `Refutation::Equation`'s
  `needed: i128` (`serialize/search.rs`, `WhyNot`); `memory_limit`
  bytes; `Statistics` counters. Decide one rule: a saturated value goes
  as `null` or a string, or every wire number is documented below 2⁵³.
- `Outcome` cannot be read back. The page only shows outcomes, so that
  is acceptable, but say so on the type.

**Options.** `search::Options` (`memo_limit`, `recursion_limit`,
`engine`, `fragment`, `test_period`, `copies`, `bias`, `forward_copies`,
`check`, `memory_limit`, `occurrence_limit`, `jobs`, `pool`) and
`ViewOptions` (`limit`, `memory`, `compact`) need tab presets as named
values (D15), such as `Options::browser()` and `ViewOptions::browser()`,
each constant with its measurement (D16).

## 4. Risks, open questions, and what the prompt should add

**Risks.**

- wasm-bindgen breaks within 0.2.x [S2], and its CLI must match the
  lock file exactly [S37]. Pin the CLI in the flake from `Cargo.lock`'s
  version and fail the flake check on a mismatch.
- A trap (panic or stack overflow) loses the instance [S5][S6]. Without
  the page-held JSON, the student loses the proof.
- The restarting turns may make some teaching sequents slow. Measure
  them on the target set's small rows, as the prompt asks.
- 32-bit integer arguments were written for 64 bits (prompt, item 2).
  An unwrapped `usize` product in a bound check is a silent wrong
  answer, not a crash (inference).
- Scope creep: a web application grows without end (3.14's own risk).

**Open questions.**

1. TS or a Rust framework for the client: the author's choice, given §2.
2. Is a suspendable focused engine needed? This depends on the measured
   slowdown and on whether cooperative stop matters more than
   `terminate()`.
3. The tab defaults: memory, occurrences, view limit, work budget, and
   a time limit read in the bindings.
4. Does the `Stop` change land at step 28 (the API audit) or here? It
   touches every engine's poll sites.
5. Can a 4 MiB+ stack be set per worker, or only per module at link
   time? I found no source that says (unverified).
6. Is the drawing's accessibility (a `graphics-document` root,
   focusable targets) a library option on `Style`, or post-processing in
   the client?
7. Should the client support ordinary logic in its first version? It
   needs the missing JSON forms.

**What the step's prompt should add.**

- The worker-plus-page-JSON architecture as the default, and
  `terminate()`-and-replay as the recovery from any trap.
- A required test: a search at `recursion_limit` under wasm32, and the
  checker and memo limit tests run at 32 bits (wasm-bindgen-test [S38];
  the wasip1 runner [S7] is an option for the core's own suite).
- The flake check pins `wasm-bindgen-cli` to `Cargo.lock` [S37].
- The library items of §3 as a checklist for steps 28 and 32: serde for
  `search::Options`, a wire form for `Error` and `Refusal`, a goal view,
  the stop with progress, the occurrence bound on entry, a rule for
  numbers above 2⁵³, UTF-16 spans, tab presets, and `Styles` in the
  library.
- A size budget for the `.wasm`, measured with `twiggy` [S18], and the
  features the client ships.
- The accessibility question of §2 before the SVG's ids are frozen.

## 5. Sources

- [S1] Crichton, A. (2025). "Sunsetting the rustwasm GitHub org". Inside
  Rust blog, 2025-07-21.
  https://blog.rust-lang.org/inside-rust/2025/07/21/sunsetting-the-rustwasm-github-org
- [S2] wasm-bindgen contributors (2026). Releases, 0.2.118 to 0.2.129;
  docs.rs page of 0.2.129 (MIT/Apache-2.0). GitHub / docs.rs, read
  2026-10-08. https://github.com/wasm-bindgen/wasm-bindgen/releases ,
  https://docs.rs/crate/wasm-bindgen/latest
- [S3] wasm-pack contributors (2026). Releases (0.15.0). GitHub.
  https://github.com/wasm-bindgen/wasm-pack/releases
- [S4] trunk contributors (2025). Releases (0.21.14). GitHub.
  https://github.com/trunk-rs/trunk/releases
- [S5] The Rust project (2026). "wasm32-unknown-unknown". The rustc book,
  platform support.
  https://doc.rust-lang.org/rustc/platform-support/wasm32-unknown-unknown.html
- [S6] The Rust project. `rustc_target/src/spec/base/wasm.rs` (the stack
  arguments). Compiler docs.
  https://doc.rust-lang.org/stable/nightly-rustc/src/rustc_target/spec/base/wasm.rs.html
- [S7] The Rust project (2026). "wasm32-wasip1". The rustc book.
  https://doc.rust-lang.org/rustc/platform-support/wasm32-wasip1.html
- [S8] RReverser and contributors (2024). wasm-bindgen-rayon
  1.3.0. docs.rs. https://docs.rs/crate/wasm-bindgen-rayon/latest
- [S9] MDN contributors. "SharedArrayBuffer", security requirements. MDN
  Web Docs.
  https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/SharedArrayBuffer
- [S10] Kitamura, E. (2020, updated 2022). "Making your website
  'cross-origin isolated' using COOP and COEP". web.dev.
  https://web.dev/articles/coop-coep
- [S11] GitHub community (2022–). "Allow setting COOP and COEP headers in
  Github Pages", discussion #13309. GitHub.
  https://github.com/orgs/community/discussions/13309
- [S12] Zuidhof, G. coi-serviceworker (MIT). GitHub.
  https://github.com/gzuidhof/coi-serviceworker
- [S13] MDN contributors. "Worker: terminate() method". MDN Web Docs.
  https://developer.mozilla.org/en-US/docs/Web/API/Worker/terminate
- [S14] Wagner, J. and Kenny, B. (2022, updated 2024). "Optimize long
  tasks". web.dev. https://web.dev/articles/optimize-long-tasks
- [S15] Rossberg, A. (2025). "Wasm 3.0 Completed". webassembly.org news,
  2025-09-17. https://webassembly.org/news/2025-09-17-wasm-3.0/
- [S16] caniuse.com. "WebAssembly Memory64". Read 2026-10-08.
  https://caniuse.com/wf-wasm-memory64
- [S17] Haas, A., Kummerow, J. and Zakai, A. (2020). "Up to 4GB of memory
  in WebAssembly". V8 blog, 2020-05-14. https://v8.dev/blog/4gb-wasm-memory
- [S18] Rust and WebAssembly Working Group. "Shrinking .wasm Code Size".
  The Rust and WebAssembly book (no longer maintained).
  https://rustwasm.github.io/docs/book/reference/code-size.html
- [S19] RReverser. serde-wasm-bindgen (MIT). GitHub.
  https://github.com/RReverser/serde-wasm-bindgen
- [S20] web-time 1.1.0 (2024), MIT/Apache-2.0. docs.rs.
  https://docs.rs/crate/web-time/latest
- [S21] Leptos contributors (2026). leptos 0.8.22, 2026-10-05; LICENSE
  (MIT). docs.rs / GitHub. https://docs.rs/crate/leptos/latest ,
  https://github.com/leptos-rs/leptos
- [S22] DioxusLabs (2026). dioxus 0.7.10, 2026-07-30, MIT/Apache-2.0.
  docs.rs. https://docs.rs/crate/dioxus/latest
- [S23] KaTeX contributors (2026). Releases (v0.19.0, 2026-10-01);
  repository (MIT). https://github.com/KaTeX/KaTeX/releases ,
  https://github.com/KaTeX/KaTeX
- [S24] MathJax contributors. Repository (Apache-2.0, accessibility);
  npm versions (4.1.3) via jsDelivr. https://github.com/mathjax/MathJax ,
  https://data.jsdelivr.com/v1/packages/npm/mathjax
- [S25] Myriad-Dreamin and contributors. typst.ts (Apache-2.0). GitHub.
  https://github.com/Myriad-Dreamin/typst.ts
- [S26] Igalia (2023). "Igalia brings MathML back to Chromium", 2023-01-10.
  https://igalia.com/2023/01/10/Igalia-Brings-MathML-Back-to-Chromium.html
- [S27] W3C (2018). WAI-ARIA Graphics Module 1.0, W3C Recommendation,
  2018-10-02. https://www.w3.org/TR/graphics-aria-1.0/
- [S28] MDN contributors. "Service Worker API". MDN Web Docs.
  https://developer.mozilla.org/en-US/docs/Web/API/Service_Worker_API
- [S30] Callies, E. and Laurent, O. (2021). "Click and coLLecT: An
  Interactive Linear Logic Prover". TLLA 2021. HAL lirmm-03271501.
  https://hal-lirmm.ccsd.cnrs.fr/lirmm-03271501
- [S31] Callies, E. and contributors. click-and-collect (LGPL-2.1).
  GitHub. https://github.com/etiennecallies/click-and-collect
- [S32] Breitner, J. and contributors. The Incredible Proof Machine (MIT).
  GitHub. https://github.com/nomeata/incredible
- [S33] Tamura, N. (1997). Announcement of a linear logic prover (CGI).
  TYPES mailing list archive.
  https://www.engineering.upenn.edu/~sweirich/types/archive/1997-98/msg00036.html
- [S34] Gallego Arias, E. J., Pin, B. and Jouvelot, P. (2016). "jsCoq:
  Towards Hybrid Theorem Proving Interfaces". UITP 2016. HAL
  hal-01425752. https://hal.archives-ouvertes.fr/hal-01425752
- [S35] leanprover-community. lean4web. GitHub.
  https://github.com/leanprover-community/lean4web
- [S36] Z3 contributors. "z3-solver" published README. GitHub.
  https://github.com/Z3Prover/z3/blob/master/src/api/js/PUBLISHED_README.md
- [S37] NixOS Discourse (2024). "Rust wasm-bindgen schema version".
  https://discourse.nixos.org/t/rust-wasm-bindgen-schema-version/56574
- [S38] wasm-bindgen contributors. "Testing in headless browsers". The
  wasm-bindgen guide.
  https://wasm-bindgen.github.io/wasm-bindgen/wasm-bindgen-test/browsers.html
- [S39] madonoharu and contributors. tsify 0.5.8 (2026-08-23). docs.rs.
  https://docs.rs/crate/tsify/latest

Sources checked 2026-10-08: 38 checked, 1 corrected, 0 removed, 2 claims marked.
