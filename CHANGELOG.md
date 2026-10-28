# Changelog

Every release of the crates `linlog` (the library) and `linlog-cli` (the
`linlog` command), newest first. The form is that of
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the
versions follow [Semantic Versioning](https://semver.org/) as Cargo reads
it for versions below 1.0: a change that breaks a caller raises the
second number (0.1.x to 0.2.0), anything else the third.

## [Unreleased]

### Added

- `IllSequent`, a sequent of intuitionistic linear logic as written: its
  hypotheses in the order written and exactly one goal, every formula
  built from `⊗`, `⊸`, `&`, `⊕`, `!`, `1`, `⊤`, `0` and atoms. It is
  built by `IllSequent::parse_within` (and `FromStr`), a strict parser
  that refuses the symbols of classical linear logic (`~`, `^`, `⅋`,
  `par`, `|`, `?`, `⊥`, `bot`) with a hint of what to write instead
  (`ParseError::hint`, a new field), by its JSON form, and by
  `IllSequent::read_one_sided`, the explicit intuitionistic reading of a
  one-sided sequent. It lowers once to the one-sided forest the engines
  take and records every occurrence's side there; `IllSequent::reading`
  is the `Reading` over those sides, equal to `Reading::new`'s.
- The intuitionistic sequent's JSON form, `{"version": 5, "terms": […],
  "atoms": […], "hypotheses": […], "goal": n}`, whose terms are the
  connectives of intuitionistic linear logic alone (a classical tag is
  refused), at wire level 5 (below).
- `InputOptions` (`Settings::input`, in JSON `"input"`), how a front end
  reads its input: `intuitionistic`, an `IllSyntax`, `strict` by default
  (`InputOptions::DEFAULT_INTUITIONISTIC`) or `one-sided`.
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
  `not_mell`.
- A MELL structure's text form (`Display`: each `?` vertex's copies and
  their boxes, a weakening's link, the links, the verdict) and JSON form
  (`Serialize`, `Deserialize`, `wire::Readable`), read and validated as
  `MellStructure::from_parts` is, within the limits.
- `svg::mell_net` draws a proof net with boxes in both drawings, every
  `!`-box a rectangle around its contents with its `!` and an auxiliary
  door per `?` vertex outside with a copy inside on its bottom border,
  nested boxes nested, the ids `o<n>` (the only vertex of occurrence
  `n`), `v<k>` (vertex `k` of an occurrence with several), `b<k>` and
  `l<x>-<y>`.
- A net's look, `svg::Style::net_look` (`svg::NetLook`, in JSON its
  fields or a preset's name): `premises` (`Premises::Dashed`, the
  default, or `Plain`: the edges into a `⅋` or `?` node and the wires
  drawn as every other edge), `conclusions` (`Conclusions::Raised`, the
  default, or `Bottom`: a `!` conclusion with auxiliary doors on the
  bottom row too, under an edge down from its door) and `why_not`
  (`WhyNot::Node`, the default, or `Rules`: a `?` node drawn as a
  dereliction `?d` under each copy, inside its box, binary contractions
  `?c` and a weakening `?w`, each copy's wire through doors of its
  own), and the presets `NetLook::LINLOG` (the default, the drawing as
  before) and `NetLook::TEXTBOOK` (all three the textbooks' way), named
  by `NetPreset`. The command's `--style svg.net_look=textbook` and
  `svg.net_look.premises=plain`, `…conclusions=bottom` and
  `…why_not=rules`, and the bindings' `styles.svg.net_look`. New types
  only; no enumeration gained a variant.
- The command's `--net` gives a MELL proof (classical, with or without
  Mix) its net with boxes in `prove` and `check`, as text, SVG, PNG and
  PDF, and `--net --format json` writes the net's document, of an MLL
  proof too, in place of the outcome when there is a proof.
- The bindings' `draw_net` draws a finished MELL proof's net with boxes,
  and `view` gives no `net` refusal for it.
- Proof nets with boxes for intuitionistic MELL without units: the net
  of the one-sided sequent, decided by the criterion without Mix
  (`MellStructure::exist` admits intuitionistic mode, affine modes still
  not), sequentialized into a proof the checker accepts in intuitionistic
  mode; `MellStructure::reading` gives a structure's reading. The
  command's `prove --net` and `check --net` and the bindings' `draw_net`
  and `view` give an intuitionistic MELL proof its net, and with
  `svg.essential` `svg::mell_net` draws it as an essential net with
  boxes: the reading's formulas, the input side in the colour `input`,
  every edge, wire and link an arrow toward the goal, a hypothesis `!A`'s
  rules `!d`, `!c` and `!w` in the textbook look. No enumeration gained
  a variant.
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
- The invertible rules applied without being asked, in every mode:
  `Interactive::invert(goal, &limits)` and `invert_all(&limits)` apply
  each rule invertible in the session's mode (`⅋`, `⊥`, `&`, `⊤`; `!`
  when every other formula is a `?`; `1`, `⊗` and the axiom when their
  formulas are the whole goal) until none applies, each application an
  ordinary step that `undo` retracts, the steps' inferences bounded by
  `limits.memory_bytes` and returning an `Inverted` (the steps taken,
  the goals left open, the refusal that stopped it if one did);
  `Interactive::is_invertible(sequent, &step)` says whether a step is
  one of them. The options value `proofs::interactive::Options` (its
  field `invertible`, off by default) is the new `Settings::interactive`
  (`{"interactive": {"invertible": true}}`). `linlog interact` takes
  `--invertible` and the command `invertible on|off`, and its `undo`
  retracts a step together with the invertible steps that followed it.
  The web bindings apply them in `start` and `apply` when the setting
  is on (`inverted` in the answer), add the call `invert_all`, list in
  the view the inferences an invertible rule closed (`invertible`), and
  say of a hint whether its rule is invertible.
- `ClassicalSequent`, a sequent of classical linear logic as written: the
  one-sided forest the engines take and, per occurrence, a `Written`
  (its `side` of `⊢` and whether it `is_implication`), recorded where the
  text is read (`ClassicalSequent::parse_within` and `FromStr`, the
  parser extended by a record it fills) and never inferred, or the
  record of a one-sided sequent as its syntax writes it
  (`ClassicalSequent::read_one_sided`). It prints back as written, up
  to brackets and double negations (`Display`, `formula`), and says
  where a negation was written (`is_negated`).
- `TwoSided`, the two-sided view of a classical derivation, built from a
  one-sided `Derivation` and the record (`TwoSided::new`, refused past
  `limits.derivation_bytes`): every sequent split by the side of `⊢` its
  formulas were written on, every rule named two-sided
  (`ClassicalRule`: `⊗L`, `⊗R`, `⅋L`, `⅋R`, `⊸L`, `⊸R`, `1L`, `1R`,
  `⊥L`, `⊥R`, `&L₁`, `&L₂`, `&R`, `⊕L`, `⊕R₁`, `⊕R₂`, `⊤R`, `0L`, `!L`,
  `!R`, `!c`, `!w`, `?L`, `?R`, `?c`, `?w`, `mix`, `wk`), and a negation
  step `~L` or `~R` inserted where a negation was written. It draws as a
  text tree (`Display`, `write_text`) and through `Drawable::Classical`
  (a new variant) in LaTeX, Typst and SVG; Rocq certifies the one-sided
  derivation it shows.
- `ViewOptions::classical`, a `ClassicalSides` (`one`, the default
  `ViewOptions::DEFAULT_CLASSICAL`, or `two`; in JSON `{"view":
  {"classical": "two"}}`): whether a front end that holds a classical
  sequent as written draws its derivation two-sided. The command's
  `--classical-sides one|two` (on `prove`, a batch included, and
  `check`) sets it.
- `Interactive::of_classical` starts a session of a classical sequent as
  written and keeps the record (`Interactive::writing`), so that its
  derivation can be drawn two-sided at any point of the proof; a
  session's JSON carries it as `"written": {"inputs": […],
  "implications": […]}`. The web bindings' `start` keeps it for a typed
  sequent of a commutative classical mode, and `export` and `draw` draw
  such a session two-sided when the settings' `view.classical` is
  `"two"`.

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
- Level 4: the proof structure with boxes, `{"version": 4, "sequent":
  {…}, "mix": false, "vertices": [[occurrence, parent, box], …],
  "links": [[x, y], …], "weakenings": [[q, l], …]}`, whose links name
  vertices; it is always written at level 4, and the proof structure's
  reader refuses a document with `vertices`.
- Level 5: the intuitionistic sequent, `{"version": 5, "terms": […],
  "atoms": […], "hypotheses": […], "goal": n}` with the terms `V`, `1`,
  `0`, `⊤`, `⊗`, `⊸`, `&`, `⊕` and `!`; it is always written at level 5.
- Level 6: the cut. A sequent's cut formulas, `"cuts": [a, …]`, the term
  of each cut formula `A` (its dual is in the arena, which the reader
  finds again), and the proof's tag `{"cut": [a, l, r]}`; a session's
  cut inference names its formula, `"cut": a`. A document whose sequent
  holds a cut formula (a proof with a cut, a session with one, an
  outcome of a goal of one) is written at level 6; every other document
  stays as it was.
- A session's optional key `written`, how its sequent was written, raises
  no level: a reader that ignores it reads the session as it is, and
  draws it one-sided.
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
- Proofs with cuts. A sequent holds its cut formulas beside its
  conclusion (`Sequent::cuts`, each a formula `A` and its dual `A⊥`,
  added by `Sequent::add_cut`), which its forest numbers after the
  conclusion, `A` then `A⊥` (`Forest::cut_pairs`, `has_cuts`,
  `is_conclusion`, `dual`, `is_cut_formula`); the reading reads `A` as
  output and `A⊥` as input (`ShapeError::CutFormula`, kind
  `cut_formula`, where it cannot). The proof term `Node::Cut(a, l, r)`
  (tag `cut`) and the checker's rule: the first premise has `a`, an
  occurrence of a cut formula `A`, and the second its dual, in every
  commutative mode, intuitionistically whichever premise proves it;
  `Fault::NotACut` (kind `not_a_cut`) for a cut on another member, and
  `Fault::Forbidden` in an ordered mode. `Rule::Cut` and
  `ClassicalRule::Cut` name it in every view and export (`cut`), and
  `Inference::cut` gives a cut inference's formula. A net is not built
  of a proof with a cut (`NetError::Rule`), the read-back to LK and LJ
  refuses it, and NanoYalla's certificate too (`Unsupported::Cut`): the
  kernel by default is then linlog's library, whose calculi have the cut
  as an optional rule.
- Cut elimination on proof terms, `proofs::cut`: `step(&proof, mode,
  at, &options, &limits, stop)` reduces the cut at `at`, or the one the
  strategy picks, checks the proof it makes in `mode` and returns it with
  the `Step` (`cut`, the cut formula's occurrence `formula`, its `Case`
  and the node `above` it acted at); `eliminate` takes steps until no
  cut is left or a bound is reached and returns the `Elimination`: the
  last proof reached, which the checker accepted, the steps, and the
  `End` (`CutFree`, or `Refused` with the refusal). Every proof made lives
  over the same forest and cuts only occurrences of the cut formulas'
  trees. The cases (`Case`): `Axiom`, `TensorPar`, `OneBot`, `WithPlus`,
  `Weakening` (also affine weakening, which erases the other premise),
  `Dereliction`, `Contraction`, `BoxIntoBox` and `Commutative`; the
  strategy `Strategy::Topmost`, the cut with no cut above it. Every mode
  with the cut: classical, affine, with Mix, intuitionistic (every
  proof made stays intuitionistic).
- `cut::Options` (`Settings::cut`, in JSON `"cut"`): `strategy`, `steps`
  (`DEFAULT_STEPS`, 10 000) and `nodes`, the most nodes of a proof a
  step makes (`DEFAULT_NODES`, 100 000); `Refusal::Elimination` with its
  `Bound` (`steps` or `nodes`), code `elimination_limit`, settings keys
  `cut.steps` and `cut.nodes`; `Phase::Eliminate`; `Error::NoCut` (code
  `no_cut`, kind unsupported) for a step asked of a node that is no cut
  or of a cut-free proof, and `Error::Reduction` (code
  `reduction_rejected`, kind defect) were the checker ever to refuse a
  proof a step made. `Proof::is_cut_free`.
- The record of an elimination, written only: `{"version", "proof",
  "steps": [{"cut", "formula", "case", "above"}], "end"}`, `end`
  `cut_free` or `refused` with the `reason` in an error's form, at wire
  level 6.
- `linlog eliminate PROOF`: checks a proof file in the flags' mode and
  eliminates its cuts, printing each step's case and cut formula and the
  derivation of the proof reached, or the record with `--format json`;
  `--steps N` (one step with `--steps 1`) and `--node-limit N` bound it.
- The search decides a goal of a forest with cut formulas on a cut-free
  forest of the goal's formulas, so no engine sees a cut formula and
  every proof found is cut-free, carried back to the goal and checked
  there.
- The cut in interactive proving: `CutFormula`, a cut formula as typed
  (`CutFormula::parse_within(text, mode, &input, &limits)` reads it in
  the syntax of the mode, or `From` an `IllSequent`, a
  `ClassicalSequent` or a `Sequent` `⊢ C`), and
  `Interactive::cut(goal, &formula, left, &limits)`, which adds the cut
  formula to the session's sequent and opens the premises `Γ, A` and
  `Δ, A⊥`, the occurrences of `A` and `A⊥` recorded as typed (right and
  left of `⊢`, intuitionistic sides as written); `undo` takes a cut back
  with its formula; `Error::CutFormula` (code `cut_formula`) for text of
  another number of formulas than one. The command's `interact` has
  `cut G C [P…]`, and the bindings' `cut(session, goal, formula, left,
  settings)`.

### Changed

- The command reads the text of an intuitionistic sequent (`-i`, and a
  batch's intuitionistic entries) in the syntax of intuitionistic linear
  logic alone: a `~`, `^`, `par`, `|`, `?` or `bot` is refused with a
  line saying what to write instead. `--ill-syntax one-sided` (the
  settings key `input.intuitionistic`, on `prove`, `interact` and `seq`)
  reads the classical syntax as before, `~(A * ~B), A |- B` as `A ⊸ B, A
  ⊢ B`. A JSON input in intuitionistic mode may be the intuitionistic
  sequent's form as well as the one-sided one, and `seq json -i` writes
  the former. The bindings' `start` and `prove` read a typed sequent of an
  intuitionistic mode the same way, by the settings' `input`, and the new
  call `read_as(input, mode, settings)` reads one for a mode.
- Every output of an intuitionistic sequent writes it as it was written,
  two-sided, with no classical symbol: the refutations' phrases (an atom
  occurring positively more often than negatively, the count equation
  over `⊗` and `⊸` on either side, the classical reading with `⊸` as
  implication), the checker's reports through `describe` (the sequents of
  a failed node two-sided, the rules by their two-sided names) for a
  proof, derivation or session in an intuitionistic mode and wherever
  `Described::in_mode` (new) names one, `Error::SequentMismatch`'s
  formulas, `Interactive::formula`, `Derivation::formula` of a two-sided
  derivation and `Proof::formula` of a proof that records an
  intuitionistic mode, a net's text through `ProofStructure::written_in`
  and `MellStructure::written_in` (new), and an essential net's SVG title
  and description. The command and the bindings draw the net of an
  intuitionistic proof as its essential net, and the bindings' goal
  formulas have `text` as `reading` writes it.

- A net with boxes in linlog's look keeps the wires that run level
  between the same two rows apart, those bound for different `?`
  vertices included, half the node radius at least, the rows drawn as
  far apart as that needs, as the textbook look already did; before,
  two such wires could share one line.
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
- The closed enums of the calculus gain the cut, the planned break of
  0.2.0: `Node::Cut` and `Rule::Cut` (`Rule::ALL` now has 17 rules and
  `Named::ALL` 35), so a `match` on either names the new variant.
- The Rocq library's calculi have the cut as an optional rule: `rules`
  gains the field `cut` and `irules` the field `icut`. A certificate that
  states its calculus by a record literal (a one-sided proof with Mix or
  affine weakening, and its refutations) now writes `cut := false`, and
  one that 0.1.0 wrote without it no longer compiles; a certificate that
  names `classical`, `intuitionistic` or `affine` is unchanged.

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
