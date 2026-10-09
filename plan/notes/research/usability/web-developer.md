# A web developer tries linlog from its documentation

Who I am: a web developer asked to build linlog's browser front end over
a WebAssembly build of the library. I have never used linlog. I read
README.md (snapshot `ws-research`) and the published rustdoc at
https://linlog-prover.github.io/linlog/linlog/index.html, starting at its
front page and following its links (search, Options, prove/prove_until/
prove_goal, Outcome, Verdict, Reason, Refutation, Statistics, Engine,
Bias, Mode, Fragment, Sequent, ParseError, Error, Proof, Derivation,
ViewOptions, Inference, Rule, Interactive, Refusal, Reading, Forest,
ProofStructure, export, export::svg and its functions and Style,
ordinary, Image, WriteError). I read no source code and compiled
nothing; the Rust below is what I would write, unchecked.

My working assumption for the wrapper: a `wasm-bindgen` crate of my own
(the README says `linlog-web` is planned but it does not exist yet), run
in a Web Worker, taking strings and JSON in and handing JSON or SVG
strings out.

## The tasks I would try first

1. Set the crate up for `wasm32-unknown-unknown`: features, threads, stack.
2. Prove a sequent the user types, stopped after a time limit.
3. Show a parse error under the input field, at the right place.
4. Hand the outcome to JavaScript as JSON, and errors too.
5. Let a settings panel choose engine, bias, copy bound and limits, kept as JSON.
6. Draw the proof as SVG, in light and dark themes, and make it clickable.
7. Prove step by step, the session held by the client between calls.
8. Draw the proof net of an MLL proof.
9. Decide a formula of ordinary logic and draw its LK/LJ derivation.

## 1. Set the crate up for WebAssembly: done with trouble

What worked: the front page's Features section lists every feature and
says plainly that `parallel` is "never for WebAssembly", and the
README's Architecture section says the library reads no clock and uses
threads only behind `parallel`. So the dependency line was easy:

```toml
[dependencies]
linlog = { version = "0.1", default-features = false,
           features = ["parse", "serialize", "interactive", "svg"] }
serde_json = "1"
wasm-bindgen = "0.2"
```

What cost me time:

- Nothing says which features are known to build for
  `wasm32-unknown-unknown`. Do `png` (resvg) and `pdf` (krilla) build
  there? Their docs say they render "with the fonts the caller gives",
  which sounds wasm-friendly, but I could not tell whether any
  dependency needs `getrandom`, file IO or threads. A "WebAssembly"
  paragraph on the front page (features that build, features that do
  not, how the project tests it) would settle it.
- The stack. `Options::DEFAULT_RECURSION_LIMIT` (2048) "fits the 8 MiB
  stack of a main thread", and `Options::stack_size` returns "at least a
  main thread's 8 MiB". A wasm module's stack is whatever the linker
  gives it (Rust's default for wasm32 is about 1 MiB as far as I know),
  and in the browser I cannot start a thread with a stack of my choosing.
  So with the defaults I expect a stack overflow trap on a deep search.
  The docs leave me to choose between lowering `recursion_limit` (to
  what? the docs give no bytes-per-level figure I can divide by) and
  passing `-C link-arg=-zstack-size=…` to the linker. One sentence on
  `stack_size` or `recursion_limit` about wasm would have helped,
  ideally with the measured bytes per level so I can compute a limit.
- `Options::DEFAULT_MEMORY_LIMIT` is 1 GiB, "which a laptop and a
  browser tab both have to spare". A wasm32 heap tops out at 4 GiB and
  mobile browsers often give far less. The forest is not counted under
  it (about 25 bytes per occurrence, stated on `occurrence_limit`, which
  is useful). I would lower both for mobile. I can do this, but it is
  guesswork.

## 2. Prove a typed sequent under a time limit: done with trouble

The front page's two examples and `prove_until` got me most of the way:

```rust
use linlog::{Mode, Options, Sequent, prove_until};

fn now_ms() -> f64 { js_sys::Date::now() } // or performance.now() via web_sys

pub fn prove_text(text: &str, mode: Mode, timeout_ms: f64) -> Result<linlog::Outcome, linlog::Error> {
    let sequent: Sequent = text.parse()?;
    let options = Options::default()
        .copies(None)                 // deepen while the time lasts, as the command does
        .memory_limit(Some(256 << 20))
        .recursion_limit(512);        // a guess, see task 1
    let deadline = now_ms() + timeout_ms;
    let mut polls = 0u32;
    prove_until(&sequent, mode, &options, || {
        polls += 1;
        polls % 256 == 0 && now_ms() >= deadline
    })
}
```

What worked well: `prove_until`'s documentation is exactly what a
front-end developer needs. It says the condition is the caller's
clock, where the engines poll, what is not polled (building the forest,
the proof check, single linear passes), and that a clock read every so
many polls is late by that many polls. `Mode` is three named flags with
`CLASSICAL`, `INTUITIONISTIC`, `.affine()` and `.with_mix()`.

What cost me time:

- The only `prove_until` example uses `std::time::Instant`, which panics
  on `wasm32-unknown-unknown`. The crate's whole reason for taking a stop
  closure is wasm, so a second example with a JS clock (or at least a
  note "Instant is not available in the browser; use performance.now")
  would save every web developer the same detour.
- The defaults differ from the command's, and I almost missed it. The
  README says the command deepens the copy bound until a 2 s time limit.
  The library's default is `copies = Some(3)` (`DEFAULT_COPIES`), and
  `prove` has no time limit. I found the "a front end with a clock lifts
  it (copies)" sentence only on the `DEFAULT_COPIES` constant and on
  `copies()`. Without it, my front end would answer "unknown: copy bound
  3" where the command proves `!(A -o B), !(B -o C), !(C -o D), !(D -o
  E), A |- E`. The front page should say how to get the command's
  behaviour: `copies(None)` plus a deadline in `stop`.
- The command also has `--pool-after` and parallel pools. Those are moot
  in wasm, but I had to read the `parallel` feature notes to be sure.
- Parsing and `Forest::new` take no stop condition. The README's LLTP
  example says an 86 MB file takes twelve seconds to parse. In a worker
  that is fine, since I can terminate the worker, but the docs should
  say outright that parsing cannot be interrupted, so that a single-
  threaded page caps the input size itself.
- How to interrupt from the UI. In a worker, the wasm call blocks the
  worker's event loop, so a "Stop" button cannot reach a Rust flag
  except through a `SharedArrayBuffer`, which needs cross-origin
  isolation. My choices are: a clock in `stop`, terminating the worker,
  or SAB. This is not the library's job, but one line in the planned web
  notes would help.

## 3. Show a parse error under the input: done

`Error::SequentParsing(Vec<ParseError>)`. `ParseError` has `span:
Range<usize>`, `found`, `label` and `expected`, and implements
`Display`. That is everything I need for an inline squiggle:

```rust
match text.parse::<Sequent>() {
    Err(linlog::Error::SequentParsing(errors)) => {
        for e in &errors {
            let start16 = text[..e.span.start].encode_utf16().count();
            let end16 = text[..e.span.end].encode_utf16().count();
            // send {start16, end16, message: e.to_string(), expected: e.expected}
        }
    }
    _ => { /* … */ }
}
```

Small cost: the span is in bytes, as the field doc says. JavaScript
strings index UTF-16 code units, and the syntax invites `⊗ ⅋ ⊸ ⊢`
(three UTF-8 bytes, one UTF-16 unit each), so I must convert. The doc
does say "byte range", so this is a pitfall, not a defect. It is not
clear why `SequentParsing` holds a `Vec`: can there be several errors
for one input? The README's caret display (`^ unexpected end of input`)
seems to be the command's own work and is not a library function. I
would reproduce it in the UI anyway.

## 4. Hand the outcome to JavaScript as JSON: done with trouble

What worked: `Outcome` implements `Serialize`, and its "§JSON" section
lists the keys: `verdict` ("proved"/"unprovable"/"unknown"),
`refutation`, `reason`, `fragment`, `mode`, `engine`, `statistics`, and
for a proof the `sequent` and `proof` keys, "so that the outcome reads
back as a Proof". `Refutation`'s doc lists its four JSON shapes. The
README shows two whole outcome objects, and its `"reason":
{"memory_limit": 100}` example is the clearest single sample.

```rust
#[wasm_bindgen]
pub fn prove_json(text: &str, mode_json: &str, timeout_ms: f64) -> String {
    let mode: Mode = match serde_json::from_str(mode_json) { Ok(m) => m, Err(e) => return err(e) };
    match prove_text(text, mode, timeout_ms) {
        Ok(outcome) => serde_json::to_string(&outcome).unwrap(),
        Err(e) => serde_json::json!({ "error": e.to_string() }).to_string(),
    }
}
```

What cost me time:

- The spellings are not all listed. The `Reason` tags: the doc gives
  `"stopped"` and `{"copy_bound": 3}` "such as", and the README gives
  `memory_limit`. I am guessing `"recursion_limit"` and `"index_limit"`.
  The `engine` strings: the README shows `"net"` and `"focus"`. Is
  `TwoSided` `"two-sided"`, `"two_sided"` or `"TwoSided"`? I would write a
  TypeScript union type from the docs, and the docs do not let me finish
  it. A table of every wire string per enum (or a JSON Schema / `.d.ts`)
  is what a front end wants.
- `Outcome` is "written, never read". That is fine for a one-way answer.
  But I cannot keep an outcome in IndexedDB and rebuild it in Rust, and
  `Statistics`, `Reason` and `Engine` have no `Deserialize`. A client
  that only reads JSON in JS does not mind; a Rust-side cache would.
- Errors have no JSON form: `Error` is `Display` only. I want a stable
  machine-readable code (`"fragment_mismatch"`, `"not_intuitionistic"`,
  …) so the UI can react: offer to switch mode, or highlight a
  subformula. I would match the 35 variants by hand into my own codes.
  That is doable, since the variants are documented, but it is fragile
  across versions, and `Error` is not even `#[non_exhaustive]`-marked as
  far as the page shows. For `NotIntuitionistic(ShapeError)` I could not
  tell how to get at the offending subformula to highlight it.
- The outcome's `net` field (the proof net the net engine found) is not
  among the JSON keys. I would serialize it separately as a
  `ProofStructure`, which has a JSON form.
- No version marker appears in any JSON form, and nothing says whether
  the forms are stable. That matters for task 7.

## 5. A settings panel kept as JSON: stuck on the JSON part, done by hand

I wanted the panel's state, `{"engine": "focus", "bias": "rarer",
"copies": null, "memory_limit": 268435456}`, to deserialize straight
into `Options`. `Options` has private fields, builder setters and no
`Serialize`/`Deserialize`. `Engine` has `Serialize` and `Display` but no
`Deserialize` or `FromStr`. `Bias` has neither serde nor `FromStr`. So
I write my own mirror struct and my own name tables:

```rust
#[derive(serde::Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
struct Settings {
    engine: Option<String>, bias: Option<String>, fragment: Option<linlog::Fragment>,
    copies: Option<u32>, forward_copies: Option<u32>,
    memory_limit: Option<u64>, occurrence_limit: Option<u64>, check: Option<bool>,
}

fn options(s: &Settings) -> Result<Options, String> {
    let mut o = Options::default().copies(s.copies).fragment(s.fragment);
    if let Some(e) = &s.engine {
        o = o.engine(Some(match e.as_str() {
            "focus" => Engine::Focus, "net" => Engine::Net, "two-sided" => Engine::TwoSided,
            "additive" => Engine::Additive, "horn" => Engine::Horn,
            other => return Err(format!("unknown engine {other}")),
        }));
    }
    if let Some(b) = &s.bias {
        o = o.bias(match b.as_str() {
            "auto" => Bias::Auto, "rarer" => Bias::Rarer, "factors" => Bias::Factors,
            other => return Err(format!("unknown bias {other}")),
        });
    }
    // … memory_limit, occurrence_limit, forward_copies, check
    Ok(o)
}
```

That works, but it duplicates a table the command surely has. With
`#[non_exhaustive]` on `Engine`, a new engine silently falls out of my
table. Also, `copies: null` cannot mean both "no bound" and "default" in
this mirror. The export side shows how it could be: `svg::Style` and
`ViewOptions` have serde, and the README says `--style-file` "is the
form a front end keeps". I expected the same of the search options.
What would help: `Options` (and the batch options) with a documented
JSON form, and `FromStr`/serde for `Engine` and `Bias`.

What worked: each setter's doc says which engines read it (for example
"Only the focused engine reads it"), so I can grey out irrelevant
controls per engine. `engine_for` tells me before a search which engine
will run. That is a nice touch for showing "will use: Horn engine".

## 6. Draw the proof as SVG, themed and clickable: done

This is the best-documented part for my purposes.

```rust
use linlog::export::svg::{self, Style};
use linlog::{ViewOptions, Verdict};

let Verdict::Proved(proof) = &outcome.verdict else { /* … */ };
let view = ViewOptions::default().limit(Some(4 << 20));
let derivation = if mode.intuitionistic {
    proof.two_sided_derivation_with(&view, || stop())?
} else {
    proof.derivation_with(&view, || stop())?
};
let mut style = if dark { Style::dark() } else { Style::default() };
style.ids = true;              // i<n>-<p> groups for clicks
style.background = None;       // transparent, the page paints
let svg_text = svg::derivation(&derivation, &style);
```

What worked well: the `svg` module says the layout is computed with
Euler Math's advances from a committed table, so the output is
deterministic and needs no font at run time. Every `<text>` carries
`textLength`, so a page without the font keeps the layout. The element
ids are documented: `i<n>`, `i<n>-<p>`, and `o<n>`/`l<m>-<n>` for nets.
`Style::dark()` exists. `Style` has serde, so a theme is a JSON object.
`description` gives a `<desc>` for screen readers, and
`Derivation::write_steps` gives the same text for an ARIA live region.
`ViewOptions` explains why a derivation can be exponentially larger
than its proof and how to bound it.

Small costs:

- Colours are "CSS colours". I wanted `currentColor` or `var(--fg)`, so
  that the page's theme switch needs no redraw. As far as I know both
  are valid in inline SVG, but the doc does not say whether the strings
  are copied verbatim or validated.
- The font is requested and not embedded. I need to ship Euler Math as
  a web font myself. The README says so; a pointer to where the font
  file comes from, and its licence, would complete it.
- `svg::derivation` returns a `String` with no size limit or stop,
  while `svg::net` takes a `limit` and `svg::write` takes a stop. I
  understood the limit is enforced when the `Derivation` is built
  (`ViewOptions`), but I had to read three pages to work that out.

## 7. Prove step by step, state held by the client: done with trouble

The README says `Interactive`'s "JSON form is what a web client will
hold between requests", and the type's documentation delivers: a
complete example, the JSON shape (`sequent`, `mode`, `inferences`,
`history`), the guarantee that reading replays every inference, and
`derivation_ids`, which maps an inference `n` of the SVG (`i<n>`) back
to the goal id that `apply` and `close` take. That last method is the
piece a clickable proof tree needs, and it is there.

```rust
#[wasm_bindgen]
pub fn step(state_json: &str, goal: u32, position: usize, rule: &str, left: &[usize]) -> String {
    let mut state: Interactive = serde_json::from_str(state_json).unwrap();
    let rule: Rule = match rule.parse() { Ok(r) => r, Err(e) => return err(e) };
    let goal = InfId::new(goal);
    match state.apply(goal, position, rule, left) {
        Ok(opened) => {
            let opened: Vec<u32> = opened.iter().map(|g| g.get()).collect(); // InfId has no serde
            json!({ "state": state, "opened": opened }).to_string()
        }
        Err(refusal) => json!({ "refused": refusal.to_string() }).to_string(),
    }
}
```

What cost me time:

- `InfId` from a number. The client sends back an id it got from a
  click (`i<n>` → `derivation_ids()[n]`) or from a goal list. The
  `Interactive` page never shows this. `InfId`'s own page (one more
  click) has `new`, `get` and `index`, but no serde, so the goals that
  `apply` opens need a manual mapping to numbers before they go out as
  JSON. A line on `Interactive` ("goal ids travel as `InfId::get`") would
  have saved the detour.
- `Refusal` has `Display` but no JSON form. I wanted to highlight the
  offending position (`Split { position }`, `NotQuest { position }`), so
  I would match the eleven variants myself. The answers of `rules()`
  (`Vec<Rule>`) do serialize, since `Rule` has serde.
- Positions and two-sided display. `goal(id)` returns the goal's
  occurrences "in any order" for `prove_goal`, while the `Inference`
  sequent is "in ascending order". Positions in `apply` are positions in
  the goal's sequent. For an intuitionistic goal I show hypotheses left
  and the goal right, using `reading().position(o)`, but I must keep the
  original position numbers on the buttons. The README's `interact`
  transcript (`goal 0: 0: A, 1: A ⊸ B ⊢ 2: B`) showed me the convention,
  which helped.
- `close` returns an `Outcome` whose proof is of the goal only, and
  `prove_goal` warns that `Proof::check` rejects it. But `Outcome`'s JSON
  says the outcome "reads back as a Proof". If I send a close result to
  the client as JSON, it looks like a proof file that will not check. I
  would strip `sequent`/`proof` from it. The docs should say this on
  `Outcome` or `close`.
- Persistence. A session saved in `localStorage` today must load after
  the library is updated. Nothing says whether the session JSON is
  stable, carries a version, or how an old or unknown rule tag is
  reported. I would add my own `{"v": 1, …}` envelope.
- `close` takes `&Options`, which brings back task 5's missing serde.

## 8. Draw the proof net: done

`Outcome::net` is `Some` when the net engine found the proof. Otherwise
`ProofStructure::from_proof(&proof, mode.mix)` desequentializes any MLL
proof, and `svg::net(&net, &style, Some(limit))` draws it. The doc says
what each colour and dash means, that a switching cycle is highlighted,
and gives the element ids. `ProofStructure` has a JSON form that
accepts partial and incorrect structures, which would allow a "link the
axioms yourself" exercise mode later. Error cases (units, additives) are
documented on `new` and in the README (`proof nets exist for MLL without
units only`). One unknown: `Fragment` has no `has_units` method that I
saw, so my UI greys the "net" button by trying `from_proof` and
catching the error. That is fine.

## 9. Ordinary logic with an LK/LJ drawing: done with trouble

The `ordinary` module page has a complete example: parse, `translate`,
`prove(image.sequent(), image.mode(), …)`, `linear_derivation`,
`read_back`, `check`. `svg::ordinary` draws it.

```rust
let s: linlog::ordinary::Sequent = text.parse()?;
let image = translate(&s, Logic::Classical, Translation::default_for(Logic::Classical))?;
let outcome = prove_until(image.sequent(), image.mode(), &options, stop)?;
// … proved → read_back → svg::ordinary(&lk, &style, &mut out, stop)
```

Costs: `svg::ordinary` only writes into a `Write`, while
`svg::derivation` returns a `String`, so the two drawing calls are
shaped differently. The outcome JSON speaks of the linear image
("proved", fragment "MALL", engine), while the command says "valid
(classical logic by the affine translation …)". There is no
library-side JSON for an ordinary verdict, so I compose one myself.
`Translation::default_for` I took from the `ordinary::Options` line.
There is also an `ordinary::Options`, which clashes by name with
`search::Options`, and I did not find a single `decide(ordinary_sequent,
options, stop)` entry point.

## Summary of what would have helped most

1. A "WebAssembly" section on the front page: the features that build
   for wasm32, the stack size needed (bytes per recursion level), and a
   `prove_until` example with a JS clock instead of `Instant`.
2. The search options as a JSON form (serde on `Options`), with
   `FromStr`/serde on `Engine` and `Bias`. Today a front end keeps its
   own name tables.
3. A complete list of wire strings (Reason tags, Engine names), ideally
   as a schema or `.d.ts`, and a stated stability and versioning rule
   for the session JSON.
4. Machine-readable errors and refusals (a code plus the positions),
   next to the `Display` text.
5. A sentence on the front page on how to match the command's
   behaviour (`copies(None)` plus a deadline).

What worked well: `prove_until`'s account of polling; the `Outcome`,
`Proof`, `Sequent`, `ProofStructure` and `Interactive` JSON sections;
the SVG module (deterministic layout, documented element ids, dark
style, screen-reader text, `derivation_ids` for clicks); the per-option
"which engine reads it" notes; and the README's examples, which double
as test fixtures I can compare my front end against.
