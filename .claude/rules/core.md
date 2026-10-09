---
paths:
  - "core/**"
---

# linlog core: the crate, and where its rules are

Loaded when any file under `core/` is read. This file holds what is
true of the whole crate; the invariants of each module are in a file of
their own, which loads beside this one when a file of that module is
read, so a session reads the rules of the code it touches and no others.
A note for the rules goes into the file of its module, one point per
bullet; one that two modules need lives in one file and the other names
it; only what holds for the whole crate goes here.

| file | loads for | what it holds |
|---|---|---|
| `core-sequents.md` | `sequents/`, `parse/`, `fragment.rs`, `serialize/`, the parse and serialize tests | the arena and its order, negation normal form, terms, kinds, fragments and modes, the parser, the JSON forms of every value |
| `core-forest.md` | `occurrences/` | the occurrence forest (numbering, the bound on its size, literal lists), `OccSet`, the intuitionistic reading |
| `core-proofs.md` | `proofs/mod.rs`, `check.rs`, `oracle.rs` | proof terms and their invariants, the checker: its one pass, its memory bound, its integers, the one-succedent condition |
| `core-derivations.md` | `proofs/derivation.rs`, `size.rs`, `fmt.rs`, `multiset.rs`, `interactive.rs` | the derivation view, the size estimate, the bounds of `ViewOptions`, the compact view, the text tree, interactive proving |
| `core-search.md` | `search/mod.rs`, `memory.rs`, `additive.rs`, `reference.rs`, `refutation.rs` | the front door (`prove_goal`, the one engine interface `Decide`, the check of every proof, `Outcome`, `Options`, refutations, the dispatch, where every engine polls its stop), the memory bound, the additive path, the test-only reference prover |
| `core-focus.md` | `search/focus/`, `search/generate.rs` | the focused engine, one- and two-sided: the atom bias, dyadic sequents, the copy bound and the memo, the two searches of the default bias, the arena, counts, interchangeable occurrences, the split search, recursion, allocation |
| `core-horn.md` | `search/horn/` | the Horn engine: the shape it reads as a Petri net and why its refutations are sound in every mode, places, classes and dead transitions, the sparse markings, the frontier and the backward search beside it, coverability in affine mode and its trie, the state equation and its exact check, sharing memory and time, its limits, the proof read off a firing sequence |
| `core-nets.md` | `nets/`, `search/net.rs` | proof structures, the correctness criterion, sequentialization, the net engine |
| `core-parallel.md` | `search/parallel.rs`, `search/focus/parallel.rs`, `search/net.rs` | the pool, stops, cube-and-conquer, the shared memo and arena, the net engine's cubes, what a pool promises |
| `core-export.md` | `export/`, `proofs/style.rs`, the export test and snapshots | the options values, the one signature, rule labels, notations, the packages' limits, Typst's own layout, fonts, SVG, PNG, PDF, Rocq |
| `core-batch.md` | `search/batch.rs` | many sequents in one call: the order of results, the workers and the stream, how the batch's memory bound is shared |
| `core-inputs.md` | `lltp.rs`, `mist.rs`, `families.rs` | the LLTP reader, the `.spec` reader of coverability problems and the generated families |
| `core-ordinary.md` | `ordinary/` | ordinary logic: its syntax and TPTP reader, the translations as one pattern table, the read-back to LK and LJ, its checker, the certificate over `Prop` |

## Layout

`sequents` (the arena, its terms, the formula walk `fmt`, the one
printer `notation` and the check of atom names `name`), `parse`,
`serialize`, `fragment`, `errors` (the one `Error`, `describe` and the
parse error), `limits` (`Limits`, `Progress`, `Refusal`), `settings`
(`Clock`, `Settings`, what a front end holds),
`wire` (the wire level, `upgrade`, `Within`, behind `serialize`),
`occurrences` (forest, sets, and the intuitionistic `reading`), `proofs`
(terms in `mod.rs`, `check`, the checker's first implementation `oracle`
for tests only, `derivation`, `size`, the text tree `fmt`, the rule
labels `style`, the crate-private `multiset`, and `interactive` behind
the feature of that name), `search` (the front door in `mod.rs`, the
memory account in `memory`, the focused engine in `focus/` with
`arena`, `bias`, `classes`, `context`, `counts`, `memo`, `schedule`, `scratch`,
`split`, `tests` and `parallel`, the net engine in
`net`, the Horn engine in `horn/` with `reach`, `cover`, `equation`, `proof` and `tests`, the runtime of the pool in `parallel` behind the feature of that
name, the additive path in `additive`, the test-only `generate` with its
classical and intuitionistic proof generators and the test-only
`reference` prover), `nets` (structures and
the criterion's front door in `mod.rs`, the graph and the Yeo test in
`graph`, the union-find in `skeleton`, `sequentialize`), `export` (the
walk over a derivation in `notation`, and `latex`, `typst`, `svg` with
`font`, `tree` and `net`, `png`, `pdf` and `rocq` behind the features
of those names),
`ordinary` (the arena and syntax in `mod.rs`, `parse` behind the
feature of that name, `translate`, `derivation` with the read-back and
its checker, `rocq` behind that feature), and `lltp`, `mist` and
`families` behind `parse`. `lib.rs` re-exports the
public types, so users write `linlog::Sequent`, `linlog::Proof`,
`linlog::prove`, and so on. `hash` is crate-private.

## The public API at a glance

Each entry point is described in the file of its module:
- sequents: `"…".parse::<Sequent>()`, `Display`, serde, `fragment()`,
  `occurrences()`; `Fragment`, `Mode` (`core-sequents.md`);
- `Forest::new(&sequent)` and `Forest::within(&sequent, &limits)`
  (`Refusal::Occurrences` past the limit), `Reading::new(&forest)`
  or a `ShapeError` (`core-forest.md`);
- `Proof::new(forest, nodes, root)`, `check(mode)`,
  `check_within(mode, &limits, stop)`, and `CheckError::{Invalid, Refused}`
  (`core-proofs.md`);
- `derivation()`, `derivation_within(&view, &limits, stop)` (the view's
  `sides`, one, two or by the proof's mode), `derivation_size(&view, …)`,
  `write_text`;
  `Interactive` (`core-derivations.md`);
- `prove`, `prove_within`, `prove_goal` with `Options` and `Limits`, returning an
  `Outcome` with a `Verdict` (`Proved`, `Unprovable` with a `Disproof`,
  `Unknown` with a `Reason`) and `Statistics`; `Options::engine` forces
  an `Engine`, whose variants describe the engines of the crate-private
  `search::focus`, `search::net`, `search::additive` and `search::horn`,
  `Options::pool` names a `search::Pool` kept across searches
  (`core-search.md`, `core-focus.md`, `core-nets.md`, `core-horn.md`, with `parallel`
  `core-parallel.md`);
- `ProofStructure` over `VertexId`s under a `Criterion`: `new`,
  `from_links`, `link`/`unlink`, `is_correct(stop)`,
  `sequentialize(&limits, stop)`, `from_proof(&proof, criterion,
  &limits, stop)` (`core-nets.md`);
- `export::latex`, `typst`, `svg`, `png`, `pdf`, `rocq`, each with one
  options value and `write(&derivation, &options, out, stop)`
  (`core-export.md`);
- `search::batch::run` and `prove`, many sequents under one options
  value (`core-batch.md`);
- `lltp::read`, `mist::read` and `families` (`core-inputs.md`);
- `ordinary`: `"…".parse::<ordinary::Sequent>()`, `read_tptp`,
  `translate(&sequent, logic, translation)` to an `Image`,
  `Image::read_back(&proof, &limits, stop)` to a `Derivation` of LK or
  LJ, `Derivation::check(&limits, stop)`, `ordinary::decide` all of it
  in one call, `export::rocq::ordinary` (`core-ordinary.md`).

## Crate-wide rules

**The rustdoc is the library's manual**, published from `main`. What a
user of the crate needs and the code keeps private is said on a public
item: the text syntax on `Sequent` and the ordinary one on
`ordinary::Sequent` (the parsers are private), every JSON
form on its type (`Sequent`, `Proof`, `Outcome`, `ProofStructure`,
`Interactive`; `serialize` is private), each engine on its `Engine`
variant (the engine modules are private), the features in the crate
docs, and "Needs the cargo feature" on every feature-gated public module
and item (`doc(cfg)` is unstable on the pinned toolchain). A change to
any of these changes those docs in the same commit. No public module is
left without public items, and `RUSTFLAGS="-W unreachable_pub -W
unnameable_types" cargo check -p linlog --all-features` stays clean: a
`pub` item is reachable from the root, and a type in a public signature
can be named.

**One error family** (`errors/mod.rs`, `limits.rs`): every public
fallible call answers `Error` or a specific type that converts into it
without loss (`CheckError`, `NetError`, `ShapeError`, `StepError`,
`ParseError`, `rocq::Unsupported`). `Error::kind()` (`ErrorKind`:
malformed, invalid, unsupported, limit, stopped, failed, defect) is one
`match` without a wildcard, so a new variant must say what it is;
`code()` is the stable reason a program branches on, listed in
`Error::CODES` (a new code goes there); `setting()` names the settings
key whose bound refused a call. Every refusal is
`Error::Refused(limits::Refusal)`, or the refusal variant of a specific
type (`CheckError::Refused`, `NetError::Refused`), so nothing that a
bound or the caller's stop ended can be read as a fault. Every variant
that carries data has named fields, but those wrapping a whole
`#[non_exhaustive]` error type of the crate (`Check`, `Net`, `Parse`,
`Rejected`, `NotIntuitionistic`, `Unsupported`); `size_of::<Error>()`
stays within 64 bytes (a `const` assertion), larger payloads boxed, and
`Error` is `Send + Sync + 'static` (another).

- **One `describe`** (`errors/describe.rs`): `describe(&owner)` on
  `Error`, `CheckError`, `NetError` and `ShapeError` returns the one
  `Described`, which prints the message with formulas for ids, read
  from an `Owner` (a `Forest`, `Proof`, `Derivation`, `ProofStructure`
  or `Interactive`), with `abbreviated(limit)`, which every writer
  honours for every formula and every list it writes (`errors::describe::
  {Budget, cut}`: a fault's member, a net's vertices, a shape's
  occurrences; a fault's formula left whole printed 134 MB from a proof
  file of 594 bytes). Each type writes itself
  once, `write(f, Option<&Forest>, …)`, which its `Display` calls with
  `None`, so the two cannot drift; `Error::write` adds the prefix of a
  wrapping variant, the same words its `#[error]` has. `Owner` is not
  sealed: a sealed supertrait is unnameable, which the crate's lint
  check forbids, so a later method gets a default body.
- **No `source()`**: a wrapping variant's message holds the inner
  error's, and the command and the harness print errors with anyhow's
  `{:#}`, which would print a source a second time.
- **The written form** (`serialize/errors.rs`, written only):
  `{"code", "kind", "message", "setting" (only where a bound refused),
  "details"}`, `details` the variant's named fields or the wrapped
  type's own form (`ParseError` derived; `CheckError` with the rule in
  the proof's node form; `Fault`, `ShapeError` and `NetError` each with
  a `kind`; `StepError` and `Refusal` tagged `kind`); a `Described`
  writes the same with the message in formulas. A new variant adds its
  arm to `Details`, which has no wildcard.

**Nothing recurses over a formula.** A sequent read from JSON can be nested
as deep as it is long, and a recursion per level ends the process where
an error is owed. So every walk is a pass over the arena or over the
forest in index order (`fragment`, `optimize`, `Forest`, `Reading::new`,
the engines' `Classes` and `Counts`), or, where a formula is written, a
loop over `sequents::fmt::Walk`: the stops of a formula in the order it is
written (`Enter` a subformula, `Between` the two subformulas of a binary
one, `Exit` a compound one), from a stack of its own whose first sixteen
steps are inline, so that an ordinary formula costs no allocation. The
printer of every target and of `Display` (`sequents::notation`:
`Notation::term` and `Notation::ill`, `core-export.md`) and the Rocq
printer are each one `match` over those stops, and a new printer is
another: never a function that calls itself. `core/tests/depth.rs` runs every walk the public API
offers on formulas nested 100 000 deep, on a thread with a stack of
256 KiB.

**No wildcard arm over the calculus's enums** in `sequents`,
`occurrences`, `nets`, `export` and `ordinary`: each denies clippy's
`wildcard_enum_match_arm` at its top, so a variant a later step adds to
`Term`, `Kind`, `Node`, `Rule` or `Named` is a compile error at every
arm that must decide it, never a fall into an existing branch (F85: a
binder kept its old index in `optimize`, a quantifier rule reached
NanoYalla's output). A test's catch-all arm takes an `#[expect]` with
its reason.

Doc examples that parse are fenced with `cfg_attr(feature = "parse", doc =
"```")` and an `ignore` fence otherwise, so `cargo test --no-default-features`
passes; copy that pattern for a new example.

## Decisions

The author's answers for the release (`plan/notes/api.md` §14), which
the fixes implement and later rounds judge against. Where a bullet above
still describes code that a decision changes, the decision holds, and
the commit that lands it rewrites that bullet.

- **The calculus's enums stay exhaustive** (`Term`, `Kind`, `Node`,
  `Rule`, `Verdict`, `Branch`, `Sign`, `Polarity`, `Side` and
  `ordinary::{Node, Rule, Side}`); every other public enum, every
  options value and every struct with public fields that a later step
  extends is `#[non_exhaustive]`, and so is every struct-like variant of
  a marked enum (`Error`, `Refusal`, `Fault`, `StepError`, `ShapeError`,
  `NetError`, `Split`, `Advances`): only the mark makes a downstream
  pattern write `..` and refuses its struct expression, so that a later
  step adds a field without a break (named fields alone do not: a
  pattern that names every field stops compiling). A data-carrying
  variant has named fields, unless it wraps a whole `#[non_exhaustive]`
  error or refutation type or is an option's value written as one JSON
  scalar. A new variant of a closed enum is a planned 0.y bump, so a
  downstream `match` fails to compile when the calculus grows instead of
  falling into a wildcard arm. A front end therefore never builds an
  error of the library: it asks the call that does (`nets::exist`,
  `Interactive::goal`).
- **One error family**: every public fallible call returns `Error` or a
  specific type that converts into it without loss. `ErrorKind` has seven
  kinds: malformed, invalid, unsupported, limit, stopped, failed and
  defect. Every specific type that can be refused carries the refusal as
  a variant of its own. Only `invalid` says a claim is wrong, so a
  refusal never reads as a fault, and the harness and the exit statuses
  tell the kinds apart.
- **Every long call takes `&Limits` and `stop: impl FnMut(Progress) ->
  bool`**, with no `Stop` trait; a closure that ignores the progress is
  `|_| false`. One rule serves every long call, and a front end without
  a clock reads its deadline from the work done; a trait's wrapper for
  progress closures does not infer.
- **Owned values, borrowed views**: `Sequent`, `Forest`, `Proof`,
  `Disproof`, `ProofStructure` and `Interactive` own their data;
  `Derivation`, `Reading` and `Goal` borrow; no `Arc` in a public type.
  An owned value is serialized and sent to a worker without lifetimes in
  the bindings, and no journey shows the clone.
- **An item without a caller waits for one** (the checked builder,
  `Limits::BROWSER`, `Settings::browser()`): it is additive later, and
  the first release promises no unmeasured numbers.
- **Lints**: clippy's `unwrap_used`, `expect_used` and `panic` stay off,
  since the code trusts its own invariants and the fuzzers decide whether
  input panics. `pedantic` is cleared in the area each site falls in,
  each lint turned on in the commit that fixes its last site; a cast
  goes through `try_from` or an `#[expect]` that gives the reason.
