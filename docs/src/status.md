# What exists and what is planned

Built:

- A comparison with the other provers of linear logic: each built from
  its own source at a pinned version, each problem translated into its
  syntax, every run measured by BenchExec under one time and memory
  limit, the problems solved, the contradictions and the figures drawn
  from the rows (`bench/TOOLS.md`, `bench/comparison.sh`, and a CI
  workflow anyone can start).
- Parsing and printing of sequents in [the syntax](syntax.md), with the ASCII and
  Unicode spellings of every connective, and a compact JSON form.
- The syntax of first-order linear logic: quantifiers over terms,
  predicates with arguments, function symbols and constants, read,
  printed one- and two-sided and written as JSON; a sequent with
  arguments but no quantifier is decided by every engine as a
  propositional one.
- First-order proofs: `∀` and `∃` steps with eigenvariables and
  witnesses, checked with the eigenvariable condition, written as JSON,
  their derivations printed one- and two-sided (`∀R`, `∀L`, `∃R`, `∃L`,
  in the classical two-sided view too)
  and drawn in LaTeX, Typst and SVG with every instance written out.
- First-order proof search: the focused engine decides first-order
  sequents in the classical modes, Mix and affine mode included, and
  the two-sided engine in the intuitionistic modes, affine included, by
  unification with the occurs check, the eigenvariable condition kept
  by levels, and another witness asked of a premise where the first
  fails; MLL1 and MALL1 decided, with exponentials within the copy
  bound; the counts' refutations per predicate; the bounds
  `--witness-depth` and `--instances`; the unifications and bindings
  counted.
- First-order benchmarks: families with verdicts by construction (the
  Lambek calculus in Moot and Piazza's translation into MILL1, a chain
  under `!`, a witness among many in MALL1 and MILL1), the harness's
  columns of unifications and bindings, and the comparison of verdicts
  with LinearOne.
- First-order certificates: every first-order proof, classical,
  intuitionistic, affine, with Mix or with cuts, written for the Rocq
  library's first-order development and checked by Rocq.
- Detection of the fragment a sequent lives in (MLL, MLL with units, ALL,
  MALL, MELL, LL, and their intuitionistic counterparts IMLL to ILL) and
  the modes classical, affine, intuitionistic and Mix as user choices.
- Intuitionistic linear logic on the same one-sided representation: a
  sequent is written in the syntax of ILL (the hypotheses left of `⊢`,
  one goal right of it, the classical symbols refused with a hint, or read
  from the classical syntax where `--ill-syntax one-sided` asks), lowered
  once to the one-sided forest with the side of every occurrence recorded,
  written back two-sided as `Γ ⊢ A` by every output (the derivations, the
  refutations, the nets, the errors, the exports and the bindings), with
  a JSON form of its own, and proved by the two-sided focused search, by
  the embedding of IMLL into MLL proof nets, by the additive fast path, or
  by the Horn engine.
- Proofs as compact terms over subformula occurrences, an independent
  checker that decides whether a term proves its sequent (in
  intuitionistic mode also that every sequent of the proof has one goal)
  in one pass and in memory proportional to the proof,
  and a derivation view that unfolds a term into the tree of the standard
  sequent calculus, one-sided or two-sided with the rules of ILL, printed
  as text. A classical derivation is drawn two-sided on request, as its
  sequent was written: the parser records how every subformula was
  written (its side of `⊢`, and `A ⊸ B` against `A⊥ ⅋ B`), the rules are
  named by side, and a negation step stands where a negation was written. The size of a derivation is computed from the term without
  building it, and every path that builds one (the text tree, the
  exports, the search inside an interactive proof) keeps within a bound
  on that size, which is an option.
- Automatic proof search for every fragment, MLL to full LL and IMLL to
  ILL, with or without Mix, returning a checked proof (the checker runs
  on every proof, in every build), "unprovable" after
  an exhaustive search (the inverse method's with the set it saturated
  with, checked apart from the search) or where the sequent is false
  read classically
  (a falsifying assignment, looked for within a bound after any search
  that did not prove it), or "unknown" with the reason: a focused sequent
  engine over dyadic sequents of occurrence bitsets with a memo, counts
  that prune sequents and direct the search for the split of a `⊗`
  (contexts of any width), one representative for formulas that occur
  several times, an atom bias chosen from the sequent or by `--bias`
  (with exponentials a forward and a backward search together), a
  per-branch bound on the copies of `?` formulas that deepens
  iteratively, without end or up to a bound, and a loop check, one-sided
  or two-sided (under Mix, a sequent none of whose parts is provable is
  refuted through its parts with one member less, not its partitions;
  a tensor whose splits are searched costs one level of recursion
  however long); for
  MLL without units, where the focused engines stop at a limit or when
  asked, a proof-net engine that searches the axiom linkings with count
  checks, constant-time cycle rejections, the exact acyclicity test, a
  symmetry break for equal literals and subformulas that are conclusions
  or leaves of one tree of `⊗` alone or `⅋` alone, and a per-atom balance
  over the skeleton's components, then sequentializes the net it finds;
  for IMLL without units, when asked, the same search over essential
  nets, directed acyclicity and the dominator condition kept by a
  transitive closure of bits with an undo log;
  for two additive-only formulas a
  recursion on subformula pairs; in every commutative mode, when asked
  or in turns beside the default engine on intuitionistic goals with
  an exponential and under Mix, the focused inverse method, a forward
  saturation from the axioms by
  rules that are whole focusing phases, with subsumption over tries of
  the sequents kept, which decides without exponentials and with them
  proves without a copy bound and refutes where its saturation ends;
  and for Horn programs with clauses under
  `!` (Petri nets) a decision of the net's markings: in linear mode a
  search of the reachable markings, kept sparse and taken nearest the
  goal first, with a search backward from the goal beside it, whose
  firing sequence is the proof and whose exhaustion refutes; in affine
  mode the backward coverability algorithm, which always ends; and in
  both the net's state equation, whose refutation is a weighting of the
  atoms that no clause raises and the goal needs raised, found by a
  simplex and checked exactly. Which engine decides a goal is a table
  of rows, by fragment, mode and one more feature, each the engine
  measured fastest there or the one that decides there at all; the
  library's documentation of `Engine` has it with the measurements.
- Affine mode, where weakening is allowed, in every fragment.
- Cyclic linear logic, multiplicative and without units: the order of a
  sequent's formulas kept up to rotation, read with the dual that
  reverses products, decided by the net engine's search for planar
  proof nets (a planarity test beside the correctness criterion), and
  checked, proved step by step and drawn with no exchange.
- The Lambek calculus, with or without empty antecedents: the
  intuitionistic part of cyclic linear logic, with its two divisions,
  read two-sided in the order written, decided by a search for planar
  essential nets, and checked, proved step by step and drawn two-sided
  with the divisions' rules.
- Proof nets for MLL, with or without Mix, as a representation of their
  own: proof structures over the subformula occurrences, an independent
  correctness criterion (Danos–Regnier, decided by Yeo's deletion test on
  the coloured structure graph, with a switching cycle or the
  disconnection named when it fails), sequentialization into a checked
  proof and desequentialization of a proof into its net, a text form and
  a JSON form.
- Proof nets for MELL without units, with or without Mix, and for
  intuitionistic MELL without units (the net of the one-sided sequent,
  decided by the same criterion without Mix, which a classical proof of
  an intuitionistic sequent without units never leaves): `!`-boxes and a `?` node with any number of copies of its
  subformula, each weakening hung on an axiom link, an independent
  correctness criterion (Danos–Regnier at every box depth with the boxes
  inside contracted, with the cycle or the disconnection and the box it
  lies in named when it fails), sequentialization through the boxes into
  a checked proof and desequentialization of a proof into its net, a text
  form, a JSON form, and the drawing with every box a rectangle, in the
  command and the web bindings. The
  net of a proof is canonical only up to which axiom link each weakening
  hangs on, the link the proof's weakening rule has above it: a weakening
  left loose could not be checked, since counting the parts of a net
  accepts structures that have no proof. So two proofs that differ only
  in the order of their rules can give different nets there, and also
  where a sequent of the proof holds a formula twice, as copies of one
  `?` formula, since a rule does not say which of the two it acts on.
- Interactive proving: a proof in progress as a derivation with open
  goals, rules applied to a formula of a goal and validated (the
  connective, the mode, the context a promotion or an axiom needs, one
  goal per premise in intuitionistic mode), undo, the search closing any
  goal, every invertible rule applied by itself on request, translation
  of the finished derivation into a term the checker validates, and a
  JSON form of the session.
- Proofs with cuts: a sequent holds its cut formulas beside its
  conclusion, a proof term has the cut rule and the checker its rule in
  every commutative mode (an ordered mode refuses it), and in
  interactive proving the user names a cut formula, written as the
  mode writes formulas, whose two premises open as goals; every view
  draws the cut, the two-sided ones with the cut formula as it was
  typed, every export writes it, and a certificate in linlog's Rocq
  library states the calculus with cut. The search stays cut-free.
- Cut elimination on proof terms, in every mode that has the cut
  (classical, affine, with Mix, intuitionistic): one step at a time, on
  the cut the caller names or the one with no cut above it, each step
  named by its case and checked, and to the end within a bound on steps
  and on the proof's size, which stops with the proof reached; at first
  order too, with the key case of `∀` and `∃` substituting the witness
  for the eigenvariable.
- Cut elimination on proof nets, with and without boxes (classical, with
  Mix, intuitionistic): a proof's cuts become cut links of its net, which
  the correctness criteria accept and sequentialization gives back as
  cuts; one step at a time on the cut link the caller names or the first
  one, named by its case (the axiom case, ⊗ against ⅋, and for boxes
  weakening, dereliction, contraction and a box entering a box), every
  net reached checked by the criterion, and every step drawn with the
  cut link and what the step acts on highlighted.
- Export of sequents and derivations, finished or in progress, to LaTeX
  (ebproof proof trees) and Typst (curryst proof trees, and trees of any
  height in a layout of linlog's own), as fragments or standalone
  documents that choose no font, and drawings of sequents,
  derivations and proof nets as SVG (every subformula of a net written,
  or compact; a net with boxes with every `!`-box a rectangle; in
  linlog's look or the textbooks', with plain edges, the conclusions on
  the bottom row and a `?` node as the dereliction, contraction and
  weakening nodes of its rules, each of the three an option of its own), laid out with the character widths
  of Euler Math, STIX Two Math, Garamond-Math, Asana Math or New Computer
  Modern Math (or a font of the caller's), a switching cycle or the
  parts of a disconnected structure highlighted, intuitionistic ones
  also as essential nets (directed edges, polarities, the dominator
  condition), with a reading for
  screen readers, and
  rendered as PNG and as archival (PDF/A-4, PDF/A-2u) or accessible
  (PDF/UA-1) PDF.
- Output configured through the library: one options value per format,
  with serde, for rule labels (one table per convention, or the user's),
  the shape of an open goal, alignment, preambles, the SVG's font, sizes
  and colours, and the certificate's names; the command sets them with
  `--style` and `--style-file`, and writes a derivation as it is made.
- Proof certificates: a finished proof as a Rocq script for the NanoYalla
  kernel, a lemma proved rule by rule and closed by `Qed`, for every
  classical fragment and for intuitionistic proofs as the classical
  proofs they are, with a flake check that runs Rocq on them.
- A Rocq library of linlog's own (`rocq/`, the package `rocq-linlog`,
  on Rocq 9 and its standard library alone): the one-sided calculus of
  classical linear logic as a plain inductive, with Mix and general
  weakening as parameters, and the two-sided calculus of intuitionistic
  linear logic and its affine variant, each with the cut as an optional
  rule; linlog's proof terms as
  datatypes, one-sided and two-sided, checkers as functions, and for
  every mode the theorem that a term the checker accepts proves its
  sequent, closed under the global context; a certificate is the
  sequent, the term and one computation. Beside it a first-order
  development: terms with de Bruijn indices and eigenvariables, both
  calculi with the quantifier rules and the eigenvariable condition,
  the first-order proof terms, their checkers in the two passes of
  linlog's own, and the same theorems in every mode. `--format rocq`
  writes them as its second kernel (`--kernel linlog`, the default for
  Mix, affine, intuitionistic and first-order proofs). The flake builds the library and compiles the
  certificates under `rocq/tests/` and those the command writes against
  it. The library proves that a derivation with Mix gives one of
  `?(⊥⊗⊥), Γ` without Mix and without cut, and, in modules under
  `rocq/bridges/` that the flake checks against the kernels' own pinned
  files, that its classical calculus derives exactly the sequents
  NanoYalla's derives and its intuitionistic calculus exactly those of
  Yalla's standalone `nanoill.v`.
- Certified refutations: the library proves that every derivable
  sequent, in every mode, is a tautology read classically, that without
  weakening an atom's literals balance and a multiplicative sequent
  meets the count equation, so that `--format rocq` writes, for an
  unprovable sequent with such a refutation, the lemma that the calculus
  does not derive it, checked by computation; the library's
  `Disproof::check` verifies the same refutations without the search, and
  an ordinary sequent that is not valid gets the lemma over `Prop` by
  instantiating its atoms.
- Parallel search, behind the library's `parallel` feature and on by
  default in the command: the focus engine runs the choices nearest the
  root on a thread pool, cube-and-conquer style, with the `&` premises in
  parallel and one memo shared by every thread; the net engine splits its
  search into cubes at its first choices for the pool; a stop condition
  reaches every thread; a pool has at most as many threads as the machine
  runs at once; the sequential engines stay one flag away.
- Time limits that hold: a stop condition is asked wherever a search can
  spend time (stable sequents, split searches, chains of forced splits,
  the set-up on a large sequent, every thread of a pool), and the
  command's `--timeout` counts from its start, reading and parsing
  included, and is kept to within a fraction of a second.
- Limits on memory and on the input: a search holds at most
  `--memory-limit` bytes (its memo is emptied first, then the answer is
  "unknown" with the reason), a sequent unfolds to at most
  `--occurrence-limit` occurrences, and no walk over a formula, a net
  or a derivation recurses on the input's depth.
- Defaults a newcomer can use, each of them an option: the copy bound
  deepens while a time limit of two seconds lasts, one thread searches
  for a tenth of a second before the other cores join it, an "unknown"
  says which bound or limit ended the search, after how long and at which
  copy bound, with the flag that changes it, and an "unprovable" says why
  where the counts of the sequent tell (an atom whose literals cannot
  pair up, the count equation that fails) or an assignment falsifies it.
- The `linlog` command: `prove`, `check`, `interact` and `seq`, with time
  limits, Ctrl-C, statistics, JSON output, proof nets, LaTeX, Typst,
  SVG and Rocq output, and `--jobs`, `--pool-after` and `--deterministic`
  for the search; every setting a flag sets read from one file with
  `--settings` as well, the form a front end keeps; `check` under a time
  limit and limits of work and memory, unknown where one ends it;
  a proof tree is printed on a terminal where it fits, a run of one
  structural rule is drawn as one inference where the whole tree would
  not fit or pass `--derivation-limit`, and a derivation past the limit
  either way is left out with a line that says so.
- Many sequents in one call: files, directories, lists of paths and
  streams on standard input, in the text syntax, as JSON Lines, in the
  harness's problem files and in the LLTP library's format, one answer
  per sequent in input order, with a time limit per sequent and for the
  batch, a memory bound for the batch, and a child process per sequent
  on request.
- Ordinary classical, intuitionistic and minimal propositional logic
  decided through their embeddings into linear logic: a syntax of its
  own and a reader for TPTP problems, the translations by name (classical
  logic into affine MALL, Girard's call-by-name and call-by-value
  translations and the 0/1 translation into ILL, minimal logic with false
  an atom) with the image printable, the linear proof read back as a
  derivation of LK or LJ that a checker of its own validates and every
  output draws, and a certificate over `Prop` for Rocq.
- Benchmarks: readers for the problems of the LLTP library and for
  coverability problems in the `.spec` format of the Mist tool, generated
  families with known verdicts (the hard families of the literature and
  the cases where one engine is known to be slow), and `linlog-bench`,
  which runs them with a time limit per run, writes CSV and summarises it,
  with a script that takes a whole baseline on an idle machine and one
  that runs the focused engine's target set by day, and a comparison of
  two baselines problem by problem.
- Performance work on the focused engine, measured by three baselines,
  one before it, one after it and one at the first release.
- Releases: the crates `linlog` and `linlog-cli` on crates.io, with a
  changelog that states what each version promises, and a workflow that
  publishes a version tag.
- WebAssembly bindings for a web page, the unpublished crate
  `linlog-web` in `web/`: the library compiled for
  `wasm32-unknown-unknown` behind calls of JSON in and JSON or SVG out
  (a sequent read, an interactive proof step by step, a goal closed by
  the search within a budget of work, a hint, every export), with
  defaults that fit a browser tab, built by the flake as a package of
  the module and its JavaScript, and tested under Node; the library's
  checker and memo are tested on a 32-bit target there too.

Planned, in roughly this order:

- Certified refutations beyond the cheap ones and the saturated sets:
  countermodels of the other logics, failure certificates of the
  backward search.
- A web page for proving step by step in the browser, on those bindings.
- First-order linear logic beyond what is built: the memo and the
  parallel search for first-order goals, the times against LinearOne,
  and first-order proving on the web page.

linlog is very much inspired by [Click & coLLecT](https://www.click-and-collect.linear-logic.org).
It aims to support more use cases on a modern, more efficient base.
Feature requests and contributions are welcome. linlog is licensed
under the EUPL; see
[Licence](https://github.com/linlog-prover/linlog/blob/main/README.md#licence).
