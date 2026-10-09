# The audit's rubric

What step 28's audit judges the repository against, and what its fixes
and check rounds judge their own work against. Every finding cites one
criterion of this file by its identifier (`A3`, `S5`, `C-GOOD-ERR`,
`AIP-193`) or a requirement of `plan/notes/requirements.md` (`R1` …
`R254`), and carries one of three severities.

## Severities

- **must-fix**: the code gives a wrong verdict or accepts a wrong
  proof, can be made to panic, hang or exhaust memory by its input,
  breaks a bound or a contract it states, breaks a machine-checked rule,
  says something false in its documentation, or has a shape that a
  planned later step (the register) could only change by a breaking
  change after 0.1.0. A must-fix is fixed in step 28.
- **should-fix**: the code breaks a criterion at a concrete cost a fix
  removes at a proportionate one: a reader misled or slowed, a thing
  done twice that will drift, a public item that need not be, a stated
  behaviour without a test, a requirement partly met where the design
  can meet it now. Fixed in step 28 unless the fix costs more than the
  finding, which the finding then says.
- **taste**: a preference the criteria do not decide (a name among
  equally clear ones, an order of items, a style both idioms allow).
  Listed apart, never counted, and put to the author as a decision only
  where it recurs or the author's rubric names it.

A finding has a file and a line, the criterion, the severity, its
evidence (a quote, a command and its output, a failing test, a surviving
mutant of `mutants/baseline/`, a fuzz finding, a measurement) and a
proposed fix. A claim without evidence is no finding. A requirement that
a later step owns and the present code need not prepare (the register's
"Met when" names a later step's deliverable) is no finding against
today's code; that its place is missing is, under S16.

## The author's criteria (2026-10-03)

| id | criterion | severity when broken |
|---|---|---|
| A1 | **Very efficient and performant.** No needless work on a hot path (a journey of `bench/src/journeys.rs`, a row of the target set): no clone, allocation, hash or rescan per step that the data structure could avoid, no superlinear pass where a linear one does. A claim of slowness rests on a count, a profile or a time, never on reading alone; a gain of a few percent on the checker's journeys needs pinned wall-clock time beside the count (`plan/reports/28-baselines.md`, "Do the counts track time?"). | must-fix for a superlinear blow-up on input a user meets; should-fix for a measured hot spot; taste for an unmeasured micro-optimisation |
| A2 | **Idiomatic project structure and current best practice.** Modules that a reader holds (a file of thousands of lines is split along its seams), std traits where they fit (`FromStr`, `Display`, `From`, `TryFrom`, `IntoIterator`), iterators over index loops where clearer, edition 2024 idioms, `#[must_use]` where a result is easy to drop, no `unsafe` without a reason. | should-fix |
| A3 | **Concise doc comments a human understands at first read.** A function says what it does and returns, not how; a type, field, variant or module says what it is; the first sentence stands alone in rustdoc's summary. Wrong or stale docs fall under S11. | should-fix; must-fix where a public item's doc is missing (the lints enforce it) |
| A4 | **Self-documenting data structures and functions.** Types say what they hold: a newtype or an enum where a bare `u32`, `usize`, `bool` or tuple carries meaning (`Mode` where a `bool` stands for it), names that say what a thing is, an argument order neighbours share, one name for one concept across modules. | should-fix |
| A5 | **Comments only where the code cannot say it**: why, an invariant, a reference; never a restatement of the next line, a history, or a note to a reader of another repository. | should-fix where a comment misleads or repeats at length; taste otherwise |
| A6 | **Code that is pleasant to review.** One thing done once (no copied walk, table, race or verdict line), short functions with one job, control flow without flags threaded by hand, no dead code, no item live only by accident of a feature. | should-fix |

## The standing criteria (CLAUDE.md, `plan/conduct.md`, the decisions)

| id | criterion | severity when broken |
|---|---|---|
| S1 | **Bounded time and memory.** Every call that can run long takes a stop and polls it at a bounded interval of work; every allocation that grows with the input is under a bound the caller sets (the memory account, the occurrence limit, the derivation limit) or refused at it. A call without flags answers within a bounded time and memory (D16). | must-fix |
| S2 | **Every default a named option** (D15, D16): a choice a user might vary is a field of one plain-data options value with `Default`, `Clone` and serde behind `serialize`, a named constant, a flag in the command and a line of documentation; presets are values, not code paths. | must-fix for a hidden choice in what a user sees; should-fix otherwise |
| S3 | **A number soundness rests on is argued or refused.** A counter, length, index or bound of a checker, a criterion, a refutation or a reader says at its declaration why it cannot reach its limit, or reaching it is a refusal with a test that gets there; no such arithmetic wraps in any build (release included). | must-fix |
| S4 | **Verdicts are sound, and a refusal is never a verdict.** No path gives `Proved` without a proof the checker accepts or `Unprovable` without an exhaustive search or a checked refutation; a checker's refusal (memory, a stop) can never be read as "invalid"; the checker stays independent of the engines it checks. | must-fix |
| S5 | **No panic, abort or unbounded work on untrusted input.** The text and ordinary syntaxes, every JSON form, LLTP, TPTP, `.spec`, a proof or session file and every command-line value end in an error, never a panic, stack overflow or hang. | must-fix |
| S6 | **Nothing recurses over a formula** (`core.md`): a walk is a pass over the arena or the forest, or a loop over `sequents::fmt::Walk`; `core/tests/depth.rs` covers every walk the API offers. | must-fix |
| S7 | **Tests as necessary.** Each stated behaviour has one focused test; a guard soundness rests on has a test that reaches it (a surviving mutant of a checker or a refutation names a missing one); a test that pins nothing another does not is excess. | must-fix for a checker or refutation guard no test reaches; should-fix otherwise |
| S8 | **Dependencies earn their place** (D10): each serves a stated reason, is scoped to the crate and feature that use it with only the crate features used, has a licence `deny.toml` allows, and none is unused. | should-fix; must-fix for an unused or wrongly licensed one |
| S9 | **No comment names the plan**: no step, session, prompt, decision number or `plan/` path in code, doc comments or help texts; comments make sense from inside the repository alone. | must-fix (machine-checked) |
| S10 | **Licence headers**: every source file starts with the two-line header in its comment syntax; prose and configuration carry none. | must-fix (machine-checked) |
| S11 | **Every document is true.** README (its usage section is run by `cli/tests/readme.rs`, its prose is not), CLAUDE.md, the rules files, the help texts, the rustdoc and the spec say what the code does, once; no stale claim, no feature described that does not exist. | must-fix for a false claim; should-fix for a missing one |
| S12 | **Wasm stays possible** (D11): the library reads no clock, uses threads only behind `parallel`, and touches no file, process or environment. | must-fix |
| S13 | **The search is deterministic on one thread**: its counters are a function of the input and the options, so the target set is a regression oracle. | must-fix |
| S14 | **Features compile alone and in pairs**, and every feature-gated public item and module says "Needs the cargo feature"; no item is live under one feature only by accident. | should-fix |
| S15 | **The rustdoc is the manual** (`core.md`): every JSON form documented on its type, every engine on its `Engine` variant, the features in the crate docs, no public module without public items, `unreachable_pub` and `unnameable_types` clean, `cargo doc --document-private-items` builds. | should-fix; must-fix for an undocumented wire form |
| S16 | **Room for the later steps** (D17, the register): the data model, the engine interface, the errors and the wire forms leave the place each planned step needs (terms, binders, a substitution and a trail for quantifiers; boxes; cut; new engines and calculi; the web client's wire forms, stops and bounds; the Rocq library's proof term) without a second rewrite, and without slowing the propositional case (the target set's counters identical, pinned CPU time within a few percent). Cite the requirement. | must-fix where the planned step would otherwise need a breaking change after 0.1.0; should-fix otherwise |
| S17 | **No compatibility aliases before the release** (D18): an item replaced is replaced outright, not kept beside its successor. | should-fix |
| S18 | **One family of errors** with a serializable form, in which a refusal (a limit, a stop, an unsupported input) is a kind of its own and never a verdict, and each error says what went wrong in terms of the caller's input. | must-fix where a refusal can be read as a verdict; should-fix otherwise |
| S19 | **The behaviour lock holds**: the command's output, every JSON form and every snapshot change only where a finding requires it, as a commit of its own. A fix that would change them says so in its proposal. | must-fix (machine-checked by the lock) |

## The Rust API Guidelines (the library's public surface)

The checklist of rust-lang.github.io/api-guidelines, cited by its
identifier, for the crates `linlog` and, where it is a library,
`linlog_cli`. Severity: should-fix unless the row says otherwise; a
guideline broken in a way a caller cannot work around is must-fix.

| group | identifiers that apply |
|---|---|
| Naming | C-CASE (casing per RFC 430), C-CONV (`as_`, `to_`, `into_`), C-GETTER (no `get_` prefix), C-ITER (`iter`, `iter_mut`, `into_iter`), C-ITER-TY (iterator types named after their methods), C-FEATURE (feature names without placeholder words), C-WORD-ORDER (consistent word order: `ParseError`, not `ErrorParse` beside it) |
| Interoperability | C-COMMON-TRAITS (`Copy`, `Clone`, `Eq`, `PartialEq`, `Ord`, `PartialOrd`, `Hash`, `Debug`, `Display`, `Default` where they fit), C-CONV-TRAITS (`From`, `TryFrom`, `AsRef`, `AsMut`), C-COLLECT (`FromIterator`, `Extend`), C-SERDE (serde behind a feature), C-SEND-SYNC (types are `Send` and `Sync` where possible), C-GOOD-ERR (error types are meaningful, implement `std::error::Error`, `Send`, `Sync`, `Display` lowercase without trailing punctuation), C-NUM-FMT, C-RW-VALUE (functions take `R: Read` and `W: Write` by value) |
| Documentation | C-CRATE-DOC (thorough crate docs with examples), C-EXAMPLE (every public item has an example where one helps), C-QUESTION-MARK (examples use `?`, not `unwrap`), C-FAILURE (sections Errors, Panics, Safety), C-LINK (hyperlinks to related items), C-METADATA (`Cargo.toml` metadata: authors, description, license, repository, keywords, categories, documentation, homepage, readme), C-RELNOTES (release notes; for step 30), C-HIDDEN (no unhelpful implementation details in rustdoc) |
| Predictability | C-SMART-PTR, C-CONV-SPECIFIC (conversions on the most specific type), C-METHOD (functions with a clear receiver are methods), C-NO-OUT (no out-parameters), C-OVERLOAD (no surprising operator overloads), C-DEREF (only smart pointers implement `Deref`), C-CTOR (constructors are static inherent methods: `new`, `with_…`, `from_…`) |
| Flexibility | C-INTERMEDIATE (expose intermediate results to avoid duplicate work), C-CALLER-CONTROL (the caller decides where to copy and place data), C-GENERIC (generic parameters minimise assumptions), C-OBJECT (traits object-safe where they may be useful as objects) |
| Type safety | C-NEWTYPE (newtypes give static distinctions), C-CUSTOM-TYPE (arguments convey meaning through types, not `bool` or `Option`), C-BITFLAG, C-BUILDER (builders for complex values) |
| Dependability | C-VALIDATE (functions validate their arguments: must-fix where an invalid argument gives a wrong answer silently), C-DTOR-FAIL, C-DTOR-BLOCK |
| Debuggability | C-DEBUG (every public type implements `Debug`), C-DEBUG-NONEMPTY |
| Future proofing | C-SEALED (sealed traits where downstream impls are not wanted), C-STRUCT-PRIVATE (private fields), C-NEWTYPE-HIDE (newtypes hide implementation details), C-STRUCT-BOUNDS (no trait bounds on data structures that do not need them); and `#[non_exhaustive]` on every public enum and struct that a later step will extend (R50) |

Not applied: the macro guidelines (the crates export no macro) and
C-PERMISSIVE (the licence is the author's choice, EUPL-1.2). C-STABLE
(public dependencies of a stable crate are stable) is read for the
public signatures that name a dependency's type.

## Google's AIPs, for the JSON wire forms only

The JSON forms (`Sequent`, `Proof`, `Outcome` with `Verdict`,
`Refutation` and `Statistics`, `ProofStructure`, `Interactive`, the
options values, `ordinary`'s values once they have one, the batch's
lines, the errors once they have one) are a data format that the web
client, stored files and wrappers depend on. Only the AIPs' rules for a
data format apply; resources, standard methods, resource names and
pagination do not. Severity: should-fix, and must-fix where a form
cannot later be extended without breaking a stored file or a client.

| AIP | what applies |
|---|---|
| AIP-180, backwards compatibility | A field is never renamed, removed, or changed in type or meaning within a version; a new field is optional with a default that keeps the old meaning; changing a default is a breaking change; a reader states whether it refuses or ignores unknown fields, and does so consistently; every form carries a version a reader checks (R6). |
| AIP-126, enumerations | Enumerations are written by name, not by number; a value means "unspecified" only where absence is meaningful; a reader of a form that may grow states what it does with an unknown value (a client must tolerate new values: a `Refutation` or `Reason` added by a later step); names are stable across versions. |
| AIP-193, errors | An error has a machine-readable reason (a stable name of its kind), a message for a human, and structured details (the position, the limit reached, the value refused); a refusal names the bound and how to lift it; the reason, not the message, is what a client branches on. |
| AIP-140, field names | Field names are `lower_snake_case`, nouns, without prepositions, with units in the name where a number has one (`memory_limit_bytes`, `time_ms`) or the unit documented once for the form; a boolean has no `is_` prefix; abbreviations only where they are the domain's term; the same concept has the same name in every form. |
| AIP-151, long-running operations, as a pattern | A long call that a client watches (a search, a check, a batch, a rendering) has a value it can poll or receive: whether it is done, progress metadata, and at the end either the result or an error; it can be cancelled, and a cancelled call ends in a well-defined state (`Unknown` with `Reason::Stopped`), never a partial result taken for a whole one. |

## What a machine checks

These criteria are checks of `nix flake check` (and of `gate`, where
named), not a reviewer's token: a reviewer cites a check's failure as
evidence and does not re-litigate what it settles.

| criterion | check |
|---|---|
| S10 licence headers | the `conventions` check: every file with a source suffix (`.rs`, `.nix`, `.toml` but the lock and fixture files, `.sh`, `.py`, `.yml`) starts with the header |
| S9 no comment names the plan | the `conventions` check: no `plan/`, "step NN", "session" in the sense of a Claude Code session, or decision number (`D7`, `D15`) in a comment, doc comment or help text of the sources |
| spelling | the `typos` check over the sources and documents, with the project's words in `typos.toml` |
| unused dependencies | the `shear` check (`cargo shear`) |
| A3, S15 rustdoc | the `doc` check (`--deny warnings`) with the workspace's rustdoc lints in `Cargo.toml` |
| A2, A6 clippy | the `clippy` check (`--deny warnings`) with the lint groups chosen in `Cargo.toml`'s `[workspace.lints.clippy]` (the choice and the groups set aside, with their reasons, are in `plan/reports/28-audit.md`) |
| S14 features | the `features` check (cargo-hack) and `gate` |
| S19 behaviour lock | `cli/tests/lock.rs`, `core/tests/lock.rs` in the tests and `gate` |
| A1 regressions | the `ratchet` check and `gate` |
| S6 depth | `core/tests/depth.rs` |
| S8 licences | the `deny` check |
| formatting | the `treefmt` check |
