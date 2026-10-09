# A Rust developer using linlog as a library

Who I am: a Rust developer with a tool of my own that holds linear logic
formulas in its own AST. I want linlog to decide them under limits I choose,
show me the verdict and the proof, and store both. I had never used linlog.
I read only README.md in the snapshot and the published rustdoc
(https://linlog-prover.github.io/linlog/linlog/index.html), starting at the
front page and following its links. I compiled nothing: the Rust below is
what I would write, checked only against the signatures the docs show.

Overall: the API docs are complete and careful. Every item I opened has a
doc comment, most have an example, and the JSON forms are specified type by
type. The front page's "common path" example got me from text to a checked,
printed and serialized proof in a minute. Most of my lost time went to
three gaps: there is no way to build a linear sequent from my own data, the
library's default limits are not the command's, and there is no line on how
to depend on the crate.

## The tasks

| # | task | outcome |
|---|---|---|
| 1 | Add linlog to my Cargo.toml with only the features I need | done with trouble |
| 2 | Build a sequent from my own formula AST and decide it | done with trouble |
| 3 | Set a time limit, a copy bound, a memory bound and threads | done with trouble |
| 4 | Read the outcome: verdict, why unprovable/unknown, engine, cost | done |
| 5 | Get the proof, print it, walk its inferences and map them back to my formulas | done with trouble |
| 6 | Serialize the outcome and proof, read the proof back, check it again | done |
| 7 | Decide many sequents at once, each with its own time limit | done with trouble |
| 8 | Export the proof as LaTeX, SVG and PNG, and draw the proof net | done with trouble (PNG) |
| 9 | Drive a step-by-step proof from my own UI | done |
| 10 | Decide an ordinary intuitionistic formula built from my AST | done |

### 1. Add the dependency and choose features: done with trouble

What worked: the front page's "Features" section lists every feature, what
each one gates and which are on by default (`parse`, `serialize`,
`interactive`, `latex`, `typst`, `svg`, `rocq`; off: `parallel`, `png`,
`pdf`). Every item that needs a feature says so.

What cost time: neither README nor the rustdoc says how to depend on the
crate. The rustdoc says version 0.1.0, and README says "A first release" is
still planned, so it is probably not on crates.io. README's Architecture
section says the library is package `linlog` in `core/`, so I guessed a git
dependency:

```toml
[dependencies]
linlog = { git = "https://github.com/linlog-prover/linlog", default-features = false, features = ["parse", "serialize", "latex", "svg", "parallel"] }
serde_json = "1"   # the docs' examples use it; linlog does not re-export it
```

What would help: a "Using the library" paragraph in README with this
`[dependencies]` line (and later the crates.io one), the MSRV/edition, and
a note that `serde_json` is the caller's to add.

### 2. Build a sequent from my own data structures: done with trouble (closest to stuck)

What I looked for: a builder, e.g. `Sequent::push(Term) -> TermId` or
`Sequent::from_parts(terms, roots, names)`. `Sequent` has `new()` (the
empty sequent), `add(Self)` (append another sequent), `optimize()` and
read accessors (`terms`, `term`, `roots`, `atom`, …), but nothing that adds
a term, an atom or a root. I checked "List of all items" to be sure. The
ordinary-logic side has exactly what I wanted (`ordinary::Formulas::add`,
`Formulas::atom`, `ordinary::Sequent::new(formulas, left, right)`), so the
gap is only on the linear side.

That leaves two routes, both documented well enough to use.

Route A, write the text and parse it. The `Sequent` page gives the full
grammar: precedences, associativity, the identifier rule (Unicode XID), and
that `bot`, `top` and `par` are reserved. My atom names (`p.1`, `has-key`)
are not identifiers, so I rename them and keep the map:

```rust
use std::collections::HashMap;
use linlog::Sequent;

enum F { Atom(String), Not(Box<F>), Tensor(Box<F>, Box<F>), Par(Box<F>, Box<F>),
         Lolli(Box<F>, Box<F>), With(Box<F>, Box<F>), Plus(Box<F>, Box<F>),
         Bang(Box<F>), Quest(Box<F>), One, Bot, Top, Zero }

fn render(f: &F, names: &mut HashMap<String, String>, out: &mut String) {
    let bin = |a: &F, op: &str, b: &F, names: &mut HashMap<String, String>, out: &mut String| {
        out.push('('); render(a, names, out); out.push_str(op); render(b, names, out); out.push(')');
    };
    match f {
        F::Atom(n) => {
            let k = names.len();
            out.push_str(names.entry(n.clone()).or_insert_with(|| format!("x{k}")));
        }
        F::Not(a) => { out.push_str("~("); render(a, names, out); out.push(')') }
        F::Tensor(a, b) => bin(a, " * ", b, names, out),
        F::Par(a, b) => bin(a, " par ", b, names, out),
        F::Lolli(a, b) => bin(a, " -o ", b, names, out),
        F::With(a, b) => bin(a, " & ", b, names, out),
        F::Plus(a, b) => bin(a, " + ", b, names, out),
        F::Bang(a) => { out.push_str("!("); render(a, names, out); out.push(')') }
        F::Quest(a) => { out.push_str("?("); render(a, names, out); out.push(')') }
        F::One => out.push('1'), F::Bot => out.push_str("bot"),
        F::Top => out.push_str("top"), F::Zero => out.push('0'),
    }
}

fn to_sequent(hyps: &[F], goals: &[F]) -> Result<(Sequent, HashMap<String, String>), linlog::Error> {
    let mut names = HashMap::new();
    let side = |fs: &[F], names: &mut HashMap<String, String>| {
        fs.iter().map(|f| { let mut s = String::new(); render(f, names, &mut s); s })
          .collect::<Vec<_>>().join(", ")
    };
    let text = format!("{} |- {}", side(hyps, &mut names), side(goals, &mut names));
    Ok((text.parse()?, names))
}
```

Renaming makes every derivation print `x0`, `x1`, so to show my own names
I must either rename back in the printed text or check each name against
the identifier rule myself. Generating text only to have it parsed again
also feels wrong in a library whose README stresses compact data
structures.

Route B, build the JSON arena with `serde_json::json!` and deserialize it
(the `Sequent` page specifies `{"terms", "ids", "var_dict"}` exactly). This
keeps my names, but the arena is one-sided and in negation normal form, so
I would have to dualize every hypothesis and push negations to the atoms
myself, which the parser does for free. `Term::dual` only dualizes the top
node. The docs don't say whether `var_dict` takes any string, or whether a
deserialized sequent keeps its root order. The parser does not: see task 5.

What would help: a public builder on `Sequent`, mirroring
`ordinary::Formulas` (`atom(name) -> TermId`, `add(Term) -> TermId`,
`push_root(TermId)`, plus a helper that adds a hypothesis by dualizing it),
or at least a sentence on the `Sequent` page pointing to the two routes
above and naming which one to use.

### 3. Set limits: done with trouble

What worked: `search::Options` is a builder with clear, measured defaults
(`DEFAULT_COPIES`, `DEFAULT_MEMORY_LIMIT`, …), and every setter says which
engines read it. `prove_until`'s page has the deadline example I needed,
and it says what is not polled (building the forest, the final check):

```rust
use std::time::{Duration, Instant};
use linlog::{Mode, Options, prove_until};

let options = Options::default()
    .copies(None)                      // deepen while the deadline lasts, as the command does
    .memory_limit(Some(512 << 20))
    .jobs(4);                          // needs the `parallel` feature, else stays sequential
let deadline = Instant::now() + Duration::from_secs(2);
let outcome = prove_until(&sequent, Mode::CLASSICAL, &options, || Instant::now() >= deadline)?;
```

What cost time:
- README describes the command's defaults (deepen the copy bound for two
  seconds, a pool after 0.1 s). The library's defaults differ: a fixed copy
  bound of 3 and no time limit. So the same sequent can be "provable" in
  the command and "unknown: copy bound" in my tool. The `copies()` doc
  explains this ("a front end with a clock lifts it"), but I only found it
  after the mismatch had confused me. `prove` "runs to completion", so a
  Horn program whose markings grow without end (per the `Engine::Horn`
  text) runs until the 1 GiB memory bound.
- There is no `timeout` option. That is reasonable since the crate has no
  clock, but a newcomer looks for one on `Options` first.
- "One thread first, then a pool" is the command's strategy, not an option.
  The `engine_for` doc hints at how to build it (ask whether the engine is
  parallel before adding a pool), but there is no example.
- `search::Options` has no serde, while the export options and
  `ViewOptions` do, so I can't load my tool's search settings from a config
  file without mapping each field myself.

What would help: a "Matching the command" paragraph on `Options` (copies
`None` plus a 2 s deadline is what `linlog prove` does), and serde on
`search::Options`.

### 4. Read the outcome: done

What worked: `Outcome` has public fields (`verdict`, `fragment`, `mode`,
`engine`, `statistics`, `net`). `Verdict` is a three-way enum. `Reason`,
`Refutation`, `Engine` and `Mode` implement `Display` as phrases, and
`Fragment::name_in(mode)` gives "IMLL" and the like. `Statistics` says
per field which engine fills it. Every `#[non_exhaustive]` is shown, so I
knew I needed the wildcard arms. Rebuilding the command's verdict line was
easy:

```rust
use linlog::{Verdict};
let head = format!("({}, {}, {} engine)",
    outcome.fragment.name_in(outcome.mode), outcome.mode, outcome.engine);
let line = match &outcome.verdict {
    Verdict::Proved(_) => format!("provable {head}"),
    Verdict::Unprovable(why) => format!("unprovable {head}: {why}"),
    Verdict::Unknown(why) => format!("unknown {head}: {why}"),
};
let s = outcome.statistics;
eprintln!("nodes {} (memo {}), splits {}, copies {}", s.nodes, s.memo_hits, s.splits, s.copies);
```

Small points: `Outcome` and `Verdict` have no `Display`, so the line above
is mine to keep in sync with the command's. `Reason`'s `Display` doc gives
"the time limit was reached" as its example phrase. If that is
`Reason::Stopped`, a Ctrl-C in my tool reads as a time limit (I couldn't
tell from the docs). The "unknown" lines in README also carry the elapsed
time and the copy bound reached, which the library leaves to me (I have
`statistics.copies`, but no time, which is correct for a library with no
clock).

### 5. Get the proof, print it, map it back to my formulas: done with trouble

What worked: `proof.derivation()?.to_string()` gives the tree exactly as
README shows it, and `Derivation::two_sided` has an example for the
intuitionistic case. `Inference` (sequent as `OccId`s, `rule`, `principal`,
`premises`) and `Forest` (preorder numbering, `formula`, `parent`,
`children`, `term`) are documented with worked examples that number the
occurrences of `A, A -o B |- B`, which taught me the forest in five minutes.

```rust
let proof = outcome.verdict.proof().expect("proved");
let d = if outcome.mode.intuitionistic { proof.two_sided_derivation()? } else { proof.derivation()? };
let forest = d.forest();
for inf in d.inferences() {
    let shown: Vec<String> = match d.reading() {
        Some(r) => inf.sequent.iter().map(|&o| r.formula(o).to_string()).collect(),
        None => inf.sequent.iter().map(|&o| forest.formula(o).to_string()).collect(),
    };
    println!("{:>4} {} : {}", inf.premises.len(), inf.rule.name(), shown.join(", "));
}
```

What cost time:
- Which proof I get is not tied to the mode. `Proof::derivation()` is
  one-sided even after an intuitionistic search, so I have to branch on
  `outcome.mode` myself.
- Mapping back to my AST: the parser sorts the root formulas. I learnt
  this from `Sequent::optimize` ("sorts the root formulas"), `FromStr`
  ("optimized") and, most plainly, from the `Reading` page ("the arena
  keeps its roots sorted by term, not in the order they were written").
  So `forest.roots()[i]` is not my i-th formula, and nothing maps a parsed
  root back to its input position. I would match roots by printing them
  and comparing strings.
- `Derivation`'s `Display` gives the default text tree. To get the
  `TextOptions` (bar, gap, labels) I need `write_text`, which wants a stop
  closure even when I have none (`|| false`, as the examples show).

What would help: keeping the written order of the roots, or a map from
input position to root; and `Outcome::derivation()` that picks one-sided or
two-sided by the outcome's mode.

### 6. Serialize and read back: done

What worked: the JSON forms are specified on each type. The `Outcome` doc
says an outcome "is written, never read" and "reads back as a `Proof`",
which is exactly what I need to store results and re-check them later:

```rust
let stored = serde_json::to_string(&outcome)?;
// later
let value: serde_json::Value = serde_json::from_str(&stored)?;
let mode: linlog::Mode = serde_json::from_value(value["mode"].clone())?;
if value["verdict"] == "proved" {
    let proof: linlog::Proof = serde_json::from_value(value)?;
    proof.check(mode)?;   // independent of the search
}
```

Small points: `Statistics`, `Verdict`, `Reason` and `Refutation` are not
`Serialize` on their own, so a record of mine that holds only the
statistics can't derive `Serialize`; I have to keep the whole `Outcome`.
The `Proof` page's `Deserialize` doc names `Prf::check`, which does not
exist (a typo for `Proof::check`).

### 7. Many sequents, each with a time limit: done with trouble

What worked: the `search::batch` page has a short example with
`batch::prove`, and `Options::plan` explains how workers and memory are
shared.

What cost time: `batch::prove` takes no stop condition. The doc says "a
front end that needs a time limit or a stop of its own calls `run`", but
`run` has no example. Its `work` closure gets `(P, &Options)`, which I take
to be the search options, but not the batch's default mode, so I capture
that myself. `DEFAULT_WORKERS` is 1 ("a front end sets it"), and without
the `parallel` feature everything runs on my thread, which I only saw on
the module page.

```rust
use std::time::{Duration, Instant};
use linlog::search::batch::{self, Cores, Problem};
use linlog::{Mode, Options, prove_until};

let opts = batch::Options {
    search: Options::default().copies(None),
    mode: Mode::CLASSICAL,
    cores: Cores::Across,
    workers: std::thread::available_parallelism().map_or(1, |n| n.get()),
    memory_limit: Some(4 << 30),
};
let default_mode = opts.mode;
let results = batch::run(problems, &opts, move |p: Problem, search: &Options| {
    let deadline = Instant::now() + Duration::from_secs(5);
    let outcome = prove_until(&p.sequent, p.mode.unwrap_or(default_mode), search,
                              || Instant::now() >= deadline);
    (p.name, outcome)
});
for (name, outcome) in results { /* in input order, as decided */ }
```

What would help: an example of `run` with a per-sequent deadline, which is
probably the most common reason to use `run`, and a `timeout`-like closure
factory in the batch options.

### 8. Export: done with trouble (PNG)

What worked: the `export` module page explains the shared shape of every
target (`derivation` returns a `String`, `write` streams with a stop). The
LaTeX page has a full example and explains how atom names are escaped. Each
options struct is serde and `Default`.

```rust
use linlog::export::{Form, latex, svg};
let d = proof.two_sided_derivation()?;
let tex = latex::derivation(&d, &latex::Options { form: Form::Standalone, ..Default::default() });
let drawing = svg::derivation(&d, &svg::Style::default());
if let Some(net) = &outcome.net {
    let net_svg = svg::net(net, &svg::Style::default(), None)?;
}
// feature "png"
let png = linlog::export::png::from_svg(&drawing, &[EULER_MATH_OTF], &linlog::export::png::Options::default())?;
```

What cost time: `png::from_svg` and `pdf::from_svg` take the font file
bytes. README says the command "carries" Euler Math, but the library
doesn't expose those bytes, and no page says where to get the font or
under what licence, so `EULER_MATH_OTF` above is a placeholder I must fill
myself. `outcome.net` is only set when the net engine ran. For a proof
from the focus engine I would need `ProofStructure::from_proof`, which I
found on the `ProofStructure` page, not near `svg::net`.

What would help: on `png`/`pdf`, one line naming the font file and where
to get it (or a `font` feature that embeds it), and a pointer from
`svg::net` to `ProofStructure::from_proof`.

### 9. Step-by-step proving from my UI: done

The `Interactive` page's example (start, `rules`, `apply` with the split
positions, `proof`) worked as written in my head on the first try.
`derivation_ids` explains how to map an SVG's `i<n>` ids back to goals,
and `split_passes` is a nice cheap check for a UI. The JSON form for
keeping a session between requests is specified. Nothing to add.

### 10. Ordinary intuitionistic logic from my AST: done

The `ordinary` module's example goes from a sequent to `translate`,
`prove`, `linear_derivation`, `read_back` and `check` in one block.
`Formulas::add`/`atom` and `ordinary::Sequent::new` let me build
the sequent from my AST directly, which made the missing builder on the
linear side (task 2) stand out. (I didn't build this one to the end, but
the signatures fit.)

## What worked well

- Every public item has a real doc comment, and most pages have an example
  that runs from parsing to the result.
- The front page's "common path" is the right first example: parse, check
  the fragment, prove, print, serialize, read back, check again.
- The `Engine` page's dispatch table says which engine decides what and
  why, with measurements. (Its "measured" column is one very wide cell,
  hard to read in the browser.)
- Error and refusal variants say what was wrong and what to change.
  `CheckError::is_refusal` makes "neither valid nor invalid" explicit.
- The JSON forms are specified type by type, with how they are checked on
  reading.

## What would have saved the most time

1. A builder for linear `Sequent` (task 2), like `ordinary::Formulas`.
2. A "Using the library" paragraph in README: the dependency line, the
   features, `serde_json`, and how the library's defaults differ from the
   command's (copy bound 3 and no time limit, against deepening for 2 s).
3. Root order preserved, or mapped, after parsing (task 5).
4. An example of `batch::run` with a per-sequent deadline (task 7).
5. Where the PNG/PDF font comes from (task 8).
6. Minor: `Prf::check` typo on `Proof`; `Sequent::verify_integrity`'s doc
   ("Check whether the internal data structure is correct") is the one doc
   that is not in the crate's style and does not say what it returns.
