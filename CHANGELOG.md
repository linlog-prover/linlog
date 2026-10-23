# Changelog

Every release of the crates `linlog` (the library) and `linlog-cli` (the
`linlog` command), newest first. The form is that of
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the
versions follow [Semantic Versioning](https://semver.org/) as Cargo reads
it for versions below 1.0: a change that breaks a caller raises the
second number (0.1.x to 0.2.0), anything else the third.

## [Unreleased]

### Added

- Proof nets of unit-free MELL with `!`-boxes, in classical mode with or
  without Mix: `MellStructure`, its vertices instances of the forest's
  occurrences, a generalised `?` vertex with any number of copies of its
  subformula, each weakening (a `?` without copies) hung on an axiom
  link; built from its `Parts` (`Placed` vertices, links, weakenings),
  validated, or desequentialized from a checked proof
  (`MellStructure::from_proof`); the criterion `is_correct`
  (Danos–Regnier at every box depth, boxes contracted, `?` vertices and
  their auxiliary doors switched over their wires) and `sequentialize`
  into a proof the checker accepts; `MellStructure::exist` says where
  they exist. New `NetError` variants: `Placement` (with `Placement`,
  the fault), `Unattached`, `InBox` (a failure inside a box) and
  `NotMell`, in JSON the kinds `placement`, `unattached`, `in_box` and
  `not_mell`. The command, the bindings and the drawing do not take them
  yet.
- The bindings' `view` (and every call that answers with a view) gives
  each goal formula, in intuitionistic mode, its `reading`: the formula
  as `Reading::formula` writes it in its position, a hypothesis `A ⊸ B`
  where `text` has the one-sided `A ⊗ ~B`; the key is absent in
  classical mode. An added key of a call's answer, no wire level.

- The focused inverse method, `Engine::Inverse` (`--engine inverse`, JSON
  `"inverse"`), in every commutative mode: a forward search that derives
  stable sequents from the axioms upward by rules compiled from the
  sequent's focusing phases, keeps a derived sequent unless a kept one
  subsumes it, and proves the goal once every stable sequent of its
  asynchronous phase is derived. It decides without exponentials, needs
  no copy bound with them, and answers `Unprovable` where its saturation
  ends, with the set it saturated with as the refutation (below); its
  rules, sequents and indices count under the memory bound. Its counters
  are `nodes` (sequents taken up), `memo_hits` (sequents subsumed) and
  `memo_entries` (sequents kept). The dispatch gives it a turn (under
  "Changed").
- The refutation by a saturated set, `Refutation::Saturated` with
  `search::Saturated` (the atoms' polarity and the sequents) and
  `search::Stable` (a sequent's linear and unrestricted zones and whether
  it absorbs), in JSON `{"kind": "saturated", "positive": […],
  "sequents": [{"linear": […], "unrestricted": […], "absorbs": …}, …]}`,
  which raises the wire level to 3 for a disproof or an unprovable
  outcome that holds it. `Disproof::check` verifies it apart from the
  search: the rules of the focused calculus read off the sequent again
  and applied to every choice of the set's sequents, each sequent derived
  one the set stands for or one no proof holds, Mix too where the mode
  has it, and a stable sequent of the goal's phase the set does not
  give; the faults `RefutationFault::Unshaped`, `NotClosed` and `Filled`
  say what fails. The search checks it before answering, as it checks a
  proof, unless `Options::check` is off; the counts' and the classical
  refutations are preferred where they apply. The Rocq export refuses it
  (`export::rocq::Unsupported::Saturated`).
- The Rocq library `rocq-linlog` under `rocq/` (logical path `Linlog`,
  Rocq 9 with its standard library and nothing else): formulas in
  negation normal form over an abstract atom type, the one-sided cut-free
  calculus of classical linear logic as a plain inductive with binary Mix
  and general weakening as parameters, linlog's proof term as a datatype,
  a checker as a function and the theorem that a term the checker accepts
  proves its sequent, closed under the global context. The flake builds
  it as a package and compiles the certificates under `rocq/tests/`
  against it.
- In the Rocq library, the theorems for Mix and for affine logic
  (`soundness_mix`, `soundness_affine`), and the two-sided calculus of
  intuitionistic linear logic and its affine variant (`ill`, over
  `iformula` named as in Yalla's `nanoill.v`), with two-sided proof terms,
  their checker and the theorem `soundness_ill`, each closed under the
  global context.
- A second Rocq kernel in the exporter, linlog's own library:
  `rocq::Kernel` (`NanoYalla`, `Linlog`), `rocq::Options::kernel` (JSON
  `styles.rocq.kernel`, `"nanoyalla"` or `"linlog"`) and the command's
  `--kernel`, with the version `rocq::LINLOG` and the preludes
  `LINLOG_PRELUDE` and `LINLOG_TWO_SIDED_PRELUDE`. A certificate for it
  states the sequent in the mode's calculus, two-sided for an
  intuitionistic derivation, and proves it from the proof term; Mix and
  affine weakening have one. The flake's `rocq-linlog` check compiles
  the exporter's certificates for it.
- In the Rocq library, the theorems `mix_to_ll` and `mix_to_classical`
  (`Mix.v`): a derivation with binary Mix gives one of `?(⊥⊗⊥), Γ`
  without Mix and without cut. Beside the library, under `rocq/bridges/`,
  the equivalences of `ll classical` with NanoYalla's `nanoll.ll` and of
  `ill intuitionistic` with Yalla's `nanoill.ill`, which the flake's new
  `rocq-bridges` check compiles against the two kernels' files as its
  inputs pin them (Yalla is a new input), each closed under the global
  context.
- In the Rocq library, the refutations (`Refutations.v`): every
  derivable sequent of `ll R`, for every `R`, reads as a tautology
  classically (`ll_tautology`); without weakening an atom's excess over
  the additive alternatives contains zero (`balance`) and a
  multiplicative sequent with units has the formulas the count equation
  asks for, at least that many with Mix (`count_equation`); and the
  two-sided calculus embeds in the one-sided one (`ill_ll`), so that
  each gives `~ ll R Γ` or `~ ill R Γ C` from an assignment, an atom or
  nothing, tested by computation (`classical_refutation`,
  `unbalanced_refutation`, `equation_refutation` and their two-sided
  forms), all closed under the global context.
- A classical refuter after the search: where a search did not prove its
  goal, an assignment of truth values to the atoms under which the goal
  read classically is false (`!` and `?` erased, `⊗` and `&` as "and",
  `⅋` and `⊕` as "or", `1` and `⊤` true, `⊥` and `0` false), which no
  provable sequent has in any mode, makes an unknown verdict unprovable
  and gives an exhausted search a certificate: the refutation
  `Refutation::Classical` (`search::Classical`, its `assignment` a value
  per atom), bounded by `search::Options::classical_work` (JSON
  `classical_work`, default `Options::DEFAULT_CLASSICAL_WORK`, 2²⁶ terms
  evaluated, zero for none) and the memory bound, and the command's
  `--classical-work N`, which the harness takes too. No verdict a search
  gave changes, nor its counters.
- `Disproof::check`, which verifies a classical assignment, an
  unbalanced atom and the count equation against the sequent, the goal
  and the mode, sharing no code with the search, and its error
  `Error::Refutation` with `search::RefutationFault` (codes
  `invalid_refutation`, kind invalid, and `uncertified`, kind
  unsupported, for the refutations that rest on the search). The
  harness writes its result in the `checked` column of an unprovable
  row.

- Certificates of non-derivability in the exporter, for linlog's
  library: `rocq::write_disproof` writes, for a disproof whose refutation
  `Disproof::check` accepts, the lemma `~ ll R Γ` (or `~ ill R Γ C` in
  intuitionistic mode) proved by the library's refutation theorem by
  computation, with the prelude `rocq::LINLOG_REFUTATION_PRELUDE`, and
  `rocq::write_not_valid` the lemma `~ (forall a b : Prop, F)` for an
  ordinary sequent whose image an assignment refutes, by instantiating
  its atoms; `rocq::Unsupported` gains `Disproof` (NanoYalla named) and
  `NotValid`. The command's `--format rocq` writes them for an
  unprovable or not valid verdict, and says on standard error where
  there is none.
- The WebAssembly bindings `linlog-web` (`web/`, not published): the
  library for a web page, compiled for `wasm32-unknown-unknown` without
  `parallel`, behind calls of JSON in and JSON, SVG or bytes out over a
  session, the settings (read over a browser tab's defaults:
  `linlog_web::tab`) and a search's outcome. The flake builds the module
  with the `wasm-bindgen` CLI of the version `Cargo.lock` pins and
  `wasm-opt`, with the notices of the crates inside it, as the package
  `linlog-web`, and runs its tests under Node (`web-wasm-tests`).
- The flake check `web-core-wasm32` runs the library's tests of the
  proof checker, the size estimate, the memo and the memory account on a
  32-bit target, `wasm32-wasip1` under wasmtime.

- The cyclic mode: cyclic linear logic, multiplicative and without
  units, where a one-sided sequent is a sequence of formulas read up to
  rotation and nothing else is exchanged. `Order` (`Commutative`,
  `Cyclic`), `Mode::CYCLIC` with `Mode::order` and `Mode::is_ordered`,
  the mode word `cyclic`; `Mode::check` refuses an ordered mode with
  weakening or Mix (`Error::OrderedMode`, code `ordered_mode`), and the search a sequent beyond unit-free MLL in
  it (`Error::OrderedFragment`, `ordered_fragment`). The command's
  `--cyclic` (on `prove`, `check`, `interact`, `seq print` and `seq
  json`), and `cyclic` as a batch entry's mode.
- `Sequent::parse_ordered`, which reads a sequent for an order: in the
  cyclic one the dual of a product reverses it (`~(A * B)` is `~B par
  ~A`) and the formulas left of `|-` come reversed, each negated.
- The cyclic checker: `Proof::check(Mode::CYCLIC)` accepts the axiom,
  `⊗` and `⅋` only (`Fault::Forbidden` otherwise) and a `⊗` only when its
  premises' contexts are the two arcs on either side of it
  (`Fault::Order`).
- Planar proof nets: `Criterion::order` and `Criterion::CYCLIC`, under
  which `is_correct` asks besides that no two links cross in the cyclic
  order of the literals; `ProofStructure::is_planar` decides that alone,
  with the witness `NetError::Crossing`. The net engine decides the
  cyclic mode, its default there, by a linking search that admits a
  link only when it crosses no link made and the literals between its
  ends are balanced; it decides goals off the roots in that mode.
- Step-by-step proving in the cyclic mode: a `⊗` splits its goal into
  the two arcs on either side of it, at a cut point (`Step::at`,
  `Split::At`) or by positions that make them, any other split being
  `StepError::Exchange`.
- `rocq::Unsupported::Ordered`: the Rocq exporter refuses a derivation
  or a disproof of the cyclic mode, whose calculi are commutative.
- The Lambek calculus, intuitionistic and in the cyclic order, without
  empty antecedents (`Mode::LAMBEK`, the mode word `lambek`) or with them
  (`Mode::with_empty_antecedents`, `lambek-star`), and
  `Mode::allows_empty_antecedents`. The command's `--lambek` and
  `--empty-antecedents` (on `prove`, `check` and `interact`) and `seq
  print --lambek`.
- The divisions `A \ B` and `B / A` in the ordered parse, as `A -o B` and
  `B par ~A`.
- The ordered reading of the Lambek calculus: `Reading::of_mode`,
  `Reading::is_ordered`, `IllConnective::Under` and `Over`, and
  `ShapeError::EmptyAntecedent` for a sequent without a hypothesis where
  the mode needs one; derivations, LaTeX, Typst and SVG print its
  sequents two-sided in their order with the divisions.
- The Lambek checker: `Proof::check(Mode::LAMBEK)` is the cyclic check
  under the ordered reading with one output and, without empty
  antecedents, an input in every sequent (`Fault::EmptyAntecedent`).
- The rules of the divisions, `\L`, `\R`, `/L` and `/R`: `Named::division`
  and the closed enum `Division` (`Under`, `Over`); a step that would
  leave a premise without a hypothesis is `StepError::EmptyAntecedent`.
- The essential engine decides the Lambek calculus, its default and only
  engine there, by a search for planar essential nets that, without
  empty antecedents, asks the body of every implication but the goal
  for a hypothesis of its own; it decides goals off the roots in the
  mode. The net engine refuses the mode.
- `svg::Style::net` (`svg::NetDrawing`, `--style svg.net=compact`, JSON
  `styles.svg.net`, `"formulas"` by default or `"compact"`): the compact
  drawing of a proof net writes only its literals and its conclusions,
  every inner `⊗` and `⅋` a small circle as before, for a net too wide
  to write every subformula, and now writes the conclusions under the
  edges hanging from its bottom layer.
- Four more math fonts for the drawings beside Euler Math, the default:
  `svg::MathFont` (`Euler`, `StixTwo`, `Garamond`, `Asana`,
  `NewComputerModern`; JSON `"euler"`, `"stix-two"`, `"garamond"`,
  `"asana"`, `"new-computer-modern"`) names one, and `MathFont::font` is
  its `svg::Font`, laid out by the new `svg::Advances` variants
  `StixTwo`, `Garamond`, `Asana` and `NewComputerModern`, each the
  advances of the font's file (STIX Two Math 2.12, Garamond-Math
  2022-01-03, Asana Math 0.962, New Computer Modern Math 7.1.1 Book) as
  Euler Math's are. A net drawn with its formulas starts its edges below
  the font's deepest descender. `svg::Font` reads
  in JSON as a font's name too (`styles.svg.font` `"stix-two"`), so a
  front end chooses a font by name; the command's `--font NAME` sets it,
  embeds the four fonts with their licences beside Euler Math
  (`cli/fonts/`), and renders a PNG or PDF with the font the drawing
  names. Output with the default font is unchanged.

### Wire forms

- Level 2: the refutation kind `classical`, `{"kind": "classical",
  "assignment": [true, false, …]}`, the modes `"cyclic"`, `"lambek"`
  and `"lambek-star"`, the rule names `"\\L"`, `"\\R"`, `"/L"` and
  `"/R"`, the faults and shape errors `"empty_antecedent"`, and a proof
  structure's `"order": "cyclic"`. A disproof or an unprovable outcome
  that holds the refutation, and a document of the cyclic mode or
  order, are written at level 2; every other document stays at level 1,
  byte for byte.
- Level 3: the refutation kind `saturated`, `{"kind": "saturated",
  "positive": ["atom" | "dual", …], "sequents": [{"linear": […],
  "unrestricted": […], "absorbs": …}, …]}`; a disproof or an unprovable
  outcome that holds it is written at level 3.
- The essential-net engine for IMLL without units, `Engine::Essential`
  (`--engine essential`, JSON `"essential"`): the net engine's linking
  search over the essential net the intuitionistic reading directs,
  keeping directed acyclicity by a transitive closure of bits with an
  undo log, counted under the memory bound, and testing the dominator
  condition; it runs only when forced. `ProofStructure::is_essential`
  decides that criterion independently of any search, with the
  witnesses `NetError::DirectedCycle` and `NetError::NoDominator` (and
  `NetError::NotIntuitionistic` for a sequent without a reading), and
  `ProofStructure::reading` gives a structure's reading.
- The drawing of an essential net in the SVG export: `svg::Style`'s
  `essential` (`--style svg.essential=true`, off by default) draws the
  net of an intuitionistic sequent with every edge an arrow from the
  hypotheses toward the goal, the input side in the colour `input`, the
  nodes as `⊸` and `⊗`, each implication joined to its antecedent by a
  dotted line in the colour `dominator`, and the essential criterion's
  witness highlighted.

- `Proof::onto(forest, goal)` carries a proof over to a goal of another
  forest whose formulas are the proof's conclusions up to their order,
  every occurrence renamed to the one in its place, refusing with the new
  `Error::SequentMismatch` (code `sequent_mismatch`, `details.proof` and
  `details.goal` the formulas only on each side) when they differ;
  `Interactive::fill(goal, &proof, …)` grafts such a proof, another
  session's, as one step, checked in the session's mode as `close_with`
  checks; the bindings' `fill(session, goal, source, settings)` takes the
  source as a finished session.

### Changed

- linlog has a logo, the word with its i drawn as `⊸` stood on end,
  under CC BY 4.0: the README (and so crates.io), the library's rustdoc
  (its logo and favicon) and the Rocq library's new README show it, by
  URL from linlog-prover.github.io, which serves the files.
- The bindings' settings take a string where the tab's settings hold
  an object, such as a math font's name for `styles.svg.font`; an array
  there is still refused.
- Two rows of the dispatch give the inverse method turns beside the
  default engine: on an intuitionistic goal with an exponential (beside
  the two-sided engine) and on a goal under Mix without weakening
  (beside the focused engine), the engine searches alone for 2²⁴ of its
  units of work, then the inverse method takes a turn each time the
  engine's own work has doubled, from its start, of at most a thirty-second
  of that work in its own units, on the same thread, inside a poll of
  the engine, which goes on where it stopped unless the turn decided:
  the turns together do at most a sixteenth of the engine's work. Only on a search of one
  thread with no engine forced (a race's single side takes it); the
  outcome names the engine that answered, and `Statistics::work` counts
  both.
- A compact proof net's premise edges begin below each literal at the
  deepest descender of the drawing's math font, as the drawing with
  formulas does, instead of a tenth of an em below its baseline, inside
  every font's descenders: 210 thousandths of an em for Euler Math, 235,
  290, 286 and 250 for STIX Two Math, Garamond-Math, Asana Math and New
  Computer Modern Math.
- A proof net is drawn as the literature draws one: every subformula
  occurrence of its conclusions written as its formula, where it stands
  in its conclusion, each `⊗` or `⅋` link joining its two premises to
  its conclusion written below them with the connective small at the
  joint, the literals along the top under the axiom links' arcs, and the
  sequent's formulas at the bottom in its order, a literal among them
  reached by its link's arc. An essential net writes every formula as
  the reading reads it. The ids `o<n>` (now each occurrence's formula)
  and `l<m>-<n>` are kept. This is `--format svg --net`, the PNG and PDF
  of a net and the web bindings' `draw_net`.

- In a label table of the exports, `\` and `/` are symbols, set as the
  divisions are: LaTeX `\backslash` and `/`, Typst `backslash` and
  `slash`.

- The command refutes by the classical reading by default: a sequent
  whose search ended unknown (at the time limit, a copy bound or another
  limit) and whose classical reading an assignment falsifies is
  unprovable (exit status 1); `--classical-work 0` gives the search's
  word alone. The refutation runs after the search, past the time limit,
  within its bound of work, and Ctrl-C stops it.

- With no kernel named, `--format rocq` writes a proof of Mix, affine or
  intuitionistic logic for linlog's library, as the mode chooses; a
  classical proof is written for NanoYalla exactly as before, and
  `--kernel nanoyalla` keeps the NanoYalla certificate of an
  intuitionistic proof and the refusal of Mix and affine weakening.
- Unit-free MLL and IMLL go to the focused engines by default, which
  measured faster than the net engine on every sequent both decide
  (`wide` sequents 1.07 to 3.6 times, the Horn encodings of Partition
  and 3-Partition up to 32 000 times); the net engine takes a sequent
  whose atoms occur once with each sign over where they stop at their
  recursion limit or memory bound (a chain
  of thousands of implications, `wide` with 4 096 literals). Before, MLL
  with no literal more than twice went to the net engine.
- The net engine breaks the symmetry of equal literals and equal
  subformulas that are conclusions or leaves of one tree of `⊗` alone
  or `⅋` alone, and checks each atom's balance over its skeleton's
  components: Lincoln's MLL 3-Partition with bins of five is refuted in
  227 nodes instead of 16 million, Matsuoka's Partition with three
  items in 391 instead of 269 950.

## [0.1.0] - 2026-10-18

The first release.

### What is promised

- **The Rust API of `linlog`** follows semantic versioning from this
  release on. Every public enum and every struct with public fields
  that may grow is `#[non_exhaustive]`, so a new variant or field is no
  break. The enums of the calculus are closed on purpose, so that a
  `match` downstream stops compiling when the calculus grows instead of
  falling into a wildcard arm: `Term`, `Kind`, `Node`, `Rule`,
  `search::Verdict`, `Branch`, `Sign`, `occurrences::Polarity`, `Side`,
  `ordinary::{Node, Rule, Side}` and `sequents::Visit`. Cut and, later,
  the quantifiers will add variants to them, each in a release that
  raises the second number.
- **The cargo features** of `linlog` are `parse`, `serialize`,
  `interactive`, `latex`, `typst`, `svg` and `rocq` (on by default) and
  `parallel`, `png` and `pdf` (off). None is removed within 0.1.
- **The JSON forms** of the data (sequents, proofs, disproofs, proof
  structures, sessions, outcomes) are promised by their wire level, not
  by the crate's version: every such document carries `"version": n`, a
  reader reads every level up to its own and refuses a later one by
  name, and a new level is a line of this file under "Wire forms",
  never by itself a break of the crates. This release writes and reads
  level 1. The options and settings values carry no version and refuse
  a key they do not know, so a later release reads this one's settings,
  and a settings file that uses a later key is refused by name here.
- **The minimum supported Rust version** is 1.92 (`rust-version`), for
  both crates with every feature; raising it is a new 0.y version and a
  line of this file.
- **The command**: within 0.1 its subcommands, flags and exit statuses
  are only added to, and its JSON output (`--format json`) follows the
  wire levels, so a 0.1.x release may write a new level (a new kind of
  refutation, say); its text for people (verdict lines, explanations,
  help) may change in any release. The library
  `linlog_cli` inside the package `linlog-cli` is the binary's and no
  API.
- **Not promised**: the text of `Display` and of error messages (their
  `code` is), the defaults' numbers (`Limits::DEFAULT_*`, the clock's
  defaults), the engines' counters, which engine decides a sequent, and
  the time it takes.

### Wire forms

- Level 1: the sequent, the proof, the disproof, the proof structure,
  the interactive session and the outcome (written only), each
  described on its type and in the module `wire`. The repository keeps
  documents of these forms as this release writes them, which every
  later release must read.

### Added

- Sequents of classical and intuitionistic linear logic, with or without
  units, exponentials, Mix and weakening (affine mode), in a text syntax
  with ASCII and Unicode spellings and a compact JSON form; detection of
  the fragment, from MLL to LL and IMLL to ILL.
- Proof search for every fragment, returning a proof the independent
  checker has validated, "unprovable" after an exhaustive search (with
  the reason where the sequent's counts tell), or "unknown" with the
  bound or limit that ended it: a focused sequent engine, a proof-net
  engine for MLL, a recursion for additive sequents and an engine for
  Horn programs under `!` (Petri nets), chosen by fragment and mode; a
  thread pool behind the feature `parallel`; stop conditions, memory
  and occurrence limits honoured everywhere a search spends time or
  memory.
- Proofs as terms over subformula occurrences, their checker, and the
  derivation view of the one-sided and two-sided sequent calculus.
- Proof nets for MLL with or without Mix: the Danos–Regnier criterion,
  sequentialization and desequentialization.
- Interactive proving: rules applied to the goals of a derivation in
  progress, undo, and the search closing a goal.
- Exports of sequents, derivations and proof nets to LaTeX, Typst and
  SVG, rendered as PNG and PDF behind the features `png` and `pdf`, and
  of finished proofs to Rocq scripts for the NanoYalla kernel.
- Ordinary classical, intuitionistic and minimal propositional logic,
  decided through its translations into linear logic, with the proof
  read back as a checked derivation of LK or LJ, and a reader for TPTP
  problems.
- Readers for the LLTP library's problems and for coverability problems
  in the `.spec` format, and generated problem families with known
  verdicts.
- The command `linlog`: `prove` (one sequent, or many from files,
  directories and standard input), `check`, `interact` and
  `seq print|json|fragment`, with a time limit of two seconds by
  default, Ctrl-C, JSON output and every export. Its exit status is 0
  for proved or valid, 1 for unprovable or invalid, 2 for an error, 3
  for unknown and 130 at a second Ctrl-C.

[Unreleased]: https://github.com/linlog-prover/linlog/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/linlog-prover/linlog/releases/tag/v0.1.0
