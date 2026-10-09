# linlog's API after step 28: the design

The public surface of the `linlog` crate after step 28, its data model
and its wire forms, and how each later step (29 to 38) enters it,
first-order logic above all. Step 28's fix areas implement what this note
marks **[28]**; what it marks **[N]** is the place step N fills, fixed
now so that step N needs no breaking change after 0.1.0. Written by step
28's design stage (2026-10-09) from three drafts and two independent
judgements (`plan/notes/api-drafts/`), walked through against every
later step (section 12), measured by a spike (section 11) and reviewed
in a fresh context (end of section 12). The author answered the open
questions (C1 to C3, T1 to T7, HD1 to HD5 of `plan/reports/28-audit.md`)
and every decision this design adds on 2026-10-09 (section 14): the
recommended answers, with a converter beside the one wire level
(decision 4) and link-time optimisation now (T3).

Words: a *form* is a JSON shape the library writes or reads; a *front
end* is a caller with a clock and a user (the command, `linlog-web`, the
harness, a notebook); a *long call* is a public call whose time or
memory can grow beyond linear in what the caller already holds. Register
entries are `R…` (`plan/notes/requirements.md`), audit findings `F…` and
`H…` (`plan/reports/28-audit-findings.json`), rubric criteria `A…`,
`S…`, `C-…` (`plan/notes/audit-rubric.md`), research constraints `D-1` to
`D-12` (`plan/notes/research/design-constraints.md`).

## 1. Principles

**P1. Owned values, borrowed views** (T2). `Sequent`, `Forest`, `Proof`,
`Disproof`, `ProofStructure` and `Interactive` own their data;
`Derivation<'a>`, `Reading<'a>` and `Goal<'a>` borrow. One rule for
forests: a value that outlives its call owns its forest, a call borrows
one. No `Arc` in a public type. *Why*: an owned value is serialized,
sent to a worker and dropped without lifetimes in the bindings, and no
journey shows the clone (recorded in `core.md` with T2's answer).

**P2. One id space per kind, extended by offset.** Every index is a
`u32` newtype (D3). Where a later step needs more members of a space,
they continue its numbering into a side table: a member past
`forest.len()` is an instance (38), a cut tree's occurrences follow the
conclusion's (34), a net's vertices past the forest are instances of `?`
subtrees (33), a proof's first-order terms continue the sequent's (38).
Such an id stays one integer on the wire. *Why*: the propositional case
is the prefix, so its values, files and counters never change (D17).

**P3. The core enums are closed; what grows is open.** `Term`, `Kind`,
`Node`, `Rule`, `Verdict`, `Branch`, `Sign`, `Polarity`, `Side` and
`ordinary::{Node, Rule, Side}` stay exhaustive: a new variant is meant to
be a compile error at every downstream `match`, and is a planned 0.y
bump written in the changelog (0.2.0 at 34, 0.3.0 at 38). Every other
public enum, every struct with public fields a later step extends, and
every struct-like variant of a marked enum is `#[non_exhaustive]` (F1,
R50); a variant of a marked enum that carries data has named fields, so a
later step can add one (a `NetError` witness gains the box it lies in at
33; walk-through 33). Two exceptions, each for a reason: a newtype
variant that wraps a whole `#[non_exhaustive]` error or refutation type
of the crate, which grows inside (`Error::Check(CheckError)`,
`Error::Parse(Box<ParseError>)`, `CheckError::Invalid(Box<Invalid>)`,
`Fault::Shape(ShapeError)`, `Refutation::Unbalanced(Unbalanced)`); and
an option's value written as one JSON scalar (`Jobs::Count(n)`,
`Cadence::Every(n)`), which cannot gain a field without a new form.
Every other positional variant becomes named at 28 (4.1 lists them). The derives a public type has are promised with it
(`Copy`, `Eq`, `Hash` on `Limits`, `Mode`, `Statistics`, `Progress`):
dropping one later is a break. Inside the crate a `match` over a closed enum names its variants:
`clippy::wildcard_enum_match_arm` is denied on `search/`, `export/`,
`ordinary/`, `nets/`, `sequents/`, `occurrences/` and `proofs/`, with one
`#[expect]` per shape reader that refuses (R248, D-11).

**P4. The trusted core is small, independent and specified.** The term,
the forest's numbering, the checker, the net criterion and the
refutation checker share no code with any engine; each is a total
function of its inputs, stated rule by rule (3.8), so that a foreign
checker (step 31's Rocq library) recomputes it. What exists only because
Rust's resources are finite (memory, a stop, an index width) lies outside
that function and is never a verdict.

**P5. One error family; a refusal is never a fault.** Every public
fallible call returns `linlog::Error` or a specific error type that
converts into it without loss; every error has a `kind()`, a stable
`code()` and, where a bound refused it, the `setting()` that lifts it.
Every specific type that can be refused carries the refusal as a variant
of its own (`Refused(Refusal)`), so a `match` that reads "invalid" never
sees one (S4, S18, F8, F30).

**P6. One bounds value and one stop on every long call.** `Limits` holds
every resource bound the library enforces; every long call takes
`&Limits` and `stop: impl FnMut(Progress) -> bool`, told the work done
since the last poll. The library reads no clock (D11). The convenience
calls (`prove`, `Proof::check`, `derivation`) are those with
`Limits::default()` and a stop that never fires.

**P7. Options are data; presets are values** (D15, D16). One plain-data
value per feature: public fields under `#[non_exhaustive]`, `Default`
equal to named `DEFAULT_*` constants, a `#[must_use]` `with_*` per field,
serde with `default` and `deny_unknown_fields`; `null` on the wire means
"no bound" and nothing else, an automatic choice is `"auto"` (F79).
What only a clock enforces (the time limit, when to add a pool) is data
too, in `Clock`, applied by the front end's stop. `Settings` is the one
value a front end holds.

**P8. A form says the lowest reader level it needs** (F26, R6, AIP-180,
D-3). Every top-level document starts with `"version": n`; a
propositional, cut-free value writes 1 in every later release; a reader
refuses a higher level by name before it reads on; a key is added, never
renamed or retyped within a level, and no new meaning rides on an old tag
through a new key (section 7).

**P9. Nothing recurses over input** (`core.md`): formulas, first-order
terms, proofs, derivations, nets, sessions. A wasm stack overflow loses
the instance.

**P10. The propositional case pays nothing** (D17). Every reservation is
an empty table, a key skipped when empty, an additive constructor or a
monomorphised parameter; the size assertions (`Term` 12 bytes, `Node`
16, `Kind` 1, `Member` 4, `ordinary::Node` 12) make a regression a
compile error, and the one change whose cost is not zero by construction
is measured (section 11).

## 2. The public surface after step 28

### 2.1 The module tree

```text
linlog                      the re-exports of 2.2
├── limits      (new)       Limits, Progress, Phase, Refusal
├── settings    (new)       Settings, Clock
├── wire        (new, serialize) Within, upgrade, LEVEL: the forms' schema
├── sequents                Sequent, Term, TermId, Kind, Atom, Formula
│   └── fmt                 Walk, Visit (public, R245)
├── fragment                Fragment, Mode
├── occurrences             Forest, OccId, Member (new), Sign, Polarity, Side (was Position),
│                           Reading, IllFormula, ShapeError          (OccSet: pub(crate), F51)
├── proofs                  Proof, Node, NodeId, Branch (was Side)
│   ├── check               CheckError, Invalid, Fault, Dyadic       (the pass stays private)
│   ├── derivation          Derivation, Inference, InfId, Rule, Named, ViewOptions, Compact, Sides
│   ├── size                Size
│   ├── style               Labels, OpenGoal, TextOptions
│   └── interactive (feat.) Interactive, GoalId, Step, Split, StepError, Applicable, Needs
├── refutation  (new)       Refutation, Unbalanced, Equation, StateEquation, Disproof
├── nets                    ProofStructure, VertexId (new), Criterion (new), NetError, Scratch
├── search                  prove, prove_within, prove_goal, engine_for, Goal, race (parallel),
│   │                       Options, Jobs, Schedule, Cadence, Engine, Bias, Outcome, Verdict,
│   │                       Reason, Statistics, Pool (parallel)
│   └── batch               Options, Cores, Plan, Problem, Answer, Results, Cancel, prove, run, run_local
├── export                  Styles (from the command), Drawable (sealed);
│                           latex, typst, svg (Style, …), rocq (Kernel, …), png, pdf
├── ordinary                Sequent, Formulas, Node, …, Image, Derivation, Outcome (new),
│                           translate, decide (new), read_tptp
├── lltp, mist, families    (feature parse)
└── (private)               errors (Error, ErrorKind, ParseError re-exported), hash, parse, serialize
```

`errors` stays private and its types are re-exported; `serialize` stays
private, and `wire` (feature `serialize`) is its public face: the
bounded reader `Within` (5.5), the converter `upgrade` (7.1), `LEVEL`,
and the documentation of every form, the schema of record (7).

### 2.2 What `lib.rs` re-exports [28]

```rust
pub use errors::{Error, ErrorKind};
#[cfg(feature = "parse")] pub use errors::ParseError;
pub use fragment::{Fragment, Mode};
pub use limits::{Limits, Phase, Progress, Refusal};
pub use nets::{Criterion, NetError, ProofStructure, VertexId};
pub use occurrences::{Forest, Member, OccId, Reading, ShapeError, Side, Sign};
pub use proofs::{Branch, CheckError, Derivation, InfId, Inference, Named, Node, NodeId, Proof, Rule,
                 Size, ViewOptions};
#[cfg(feature = "interactive")] pub use proofs::interactive::{GoalId, Interactive, Step, StepError};
pub use refutation::{Disproof, Refutation};
pub use search::{Bias, Engine, Goal, Options, Outcome, Reason, Statistics, Verdict,
                 engine_for, prove, prove_goal, prove_within};
pub use sequents::{Atom, Formula, Kind, Sequent, Term, TermId};
pub use settings::{Clock, Settings};
```

No longer at the root: `Scratch` (stays `nets::Scratch`: a second
structure of 33 or 34 would make the name ambiguous), `Polarity`,
`Compact`, `Labels`, `OpenGoal`, `TextOptions` (under their modules),
`OccSet`, `DEFAULT_MEMORY_LIMIT` (one constant, `Limits::DEFAULT_MEMORY_BYTES`),
`ViewError`, `WriteError`, `Refusal` of the session (renamed
`StepError`; `limits::Refusal` is the bound's refusal).

### 2.3 What becomes private [28]

`OccSet` and its iterator, `Forest::{empty_set, root_set}` (no public
call takes a set, and its width rules would become API the zone types
must keep; F51); the free functions `proofs::check::{check,
check_within}` (the methods on `Proof` are the entry); `Sequent::
verify_integrity` (every public way to a `Sequent` keeps the invariant;
`Sequent::check` is its crate-private successor, F83); `Forest::
from_owned` unless a caller outside the crate needs it (the fix area
checks); `TryFrom<Sequent> for Forest` (it cannot take a bound, F27).
`linlog_cli` is an internal library: its modules `pub(crate)` but `run`,
a sentence saying it carries no semver promise, `cargo-semver-checks`
naming `linlog` only (R253, F203).

### 2.4 Before and after

A later session searches this table by the old name. "Removed" items
have no alias (D18, no compatibility aliases before 0.1.0). All rows are
[28] unless marked.

| before | after | why |
|---|---|---|
| `proofs::Side` (the `⊕` rule's choice), `occurrences::Position` (input or output) | `Branch`, `Side` (the side of `⊢`): `Named.side` and the wire's `side` hold a `Side`, `Step.position` an index (walk-through 32) | D18 |
| `Term::{Var, DualVar}`, `Kind::{Var, DualVar}`, `Sign::{Var, DualVar}` | `Term::{Atom, DualAtom}`, `Kind::{Atom, DualAtom}`, `Sign::{Atom, Dual}`, discriminants kept (literal groups are `2·atom + sign`); the JSON tags `V`, `D` stay | F25, R251 |
| `Atom` "a propositional variable" | `Atom` an atomic formula, nullary until step 38 (3.2) | D-4, decision 1 (14) |
| `Sequent::optimize` sorting the roots, fallible | written order kept (C1); infallible on a sequent that passed `check` | F24, C1 |
| — | `Sequent::antecedents() -> Option<u32>`, the roots written left of `⊢` | H9, H10, R54, R74 |
| `Sequent::verify_integrity` | crate-private `Sequent::check` | F51, F83 |
| `FromStr for Sequent` only | `+ Sequent::parse_within(text, &Limits)` | F16 |
| `Forest::within(&s, u64)`, `from_owned(s, u64)`, `TryFrom<Sequent>` | `Forest::within(&s, &Limits)`; `new(&s)` its defaults; `TryFrom` removed | F27, F63 |
| `OccSet`, `Forest::{empty_set, root_set}` public | crate-private | F51 |
| `OccId` in `Node`'s operands, `Dyadic`, `Inference::sequent`, `Interactive::goal`, `prove_goal`, `engine_for` | `Member` (3.4); `Node::occurrences()` → `Node::members()` | F19, R244, D-1 |
| `Mode { pub intuitionistic, pub affine, pub mix }`, `Mode::affine()` | private fields, `is_intuitionistic()`, `is_affine()`, `has_mix()`, `with_affine()`, `with_mix()`, `name()`, `NAMES`, `FromStr`, `check()` | F7, F40, R51, R241 |
| `Fragment::ALL` (the additive class) | `Fragment::ADDITIVE` (its name stays `ALL`); `FromStr`, `NAMED` | F56, F76 |
| `Proof` without a conclusion or mode | `Proof::{goal(), mode()}`, `new_of_goal`, `with_mode` | F89, F23, R7 |
| `Proof::check_within(mode, Option<u64>)` | `check_within(mode, &Limits, stop)` | F12, R23 |
| `CheckError { node, rule, premises, problem }`, `is_refusal()`; `Problem` with `Memory` | `CheckError::{Invalid(Box<Invalid>), Refused(Refused)}`; `Fault` (was `Problem`) without `Memory` | S4, F56 |
| `Proof::{derivation_with(&view, stop), two_sided_derivation(_with), derivation_size(bool), derivation_size_within(bool, Option<u64>)}` | `derivation()`, `derivation_within(&view, &Limits, stop)`, `derivation_size(&view, &Limits, stop)`; the sides are `view.sides` | F10, F62 |
| `ViewOptions { limit, memory, compact }`, `DEFAULT_LIMIT`, `UNBOUNDED` | `ViewOptions { compact, sides }`; the bounds in `Limits` | F62, F63 |
| `ViewError`, `WriteError`, `RenderError`, `svg::TooLarge` | `Error` variants (`InvalidProof`, `Refused`, `WriteFailed`, `RenderFailed`, …) | F8, F36, S18 |
| `Rule` (34 variants), `Rule::ALL: [Rule; 34]`, `classical()`, `intuitionistic()` | `Rule` (15 one-sided rules and `Open`), `ALL: &'static [Rule]`, `Named { rule, side }` with the two-sided names | F60, R61, D-5 |
| `Inference`'s public fields | `#[non_exhaustive]`, private fields, accessors (`sequent() -> &[Member]`, `rule() -> Named`, …) | F1, R61, R66 |
| `Interactive::apply(goal, position, rule, left)`, `goal(InfId) -> &[OccId]`, `rules -> Vec<Rule>`, `new(&s, mode)`, `close_all -> Result<Vec<…>>`, `inferences()` | `apply(GoalId, &Step)`, `goal(GoalId) -> Option<&[Member]>`, `rules -> Vec<Applicable>`, `within(&s, mode, &Limits)` (`new` its defaults), `close_all -> Vec<(GoalId, Result<Outcome, Error>)>`; `inferences()` removed | F22, F27, F66, F68, F69, R91 |
| `Interactive::close_with` grafting any proof that checks against the goal | refuses a proof of another forest, of another goal, or that the session's mode forbids | F23 |
| `proofs::Refusal` (a step refused) | `interactive::StepError` | P5 |
| `ProofStructure::{new(forest, mix), from_links(forest, mix, …), from_proof(&p, mix), mix(), is_correct(), sequentialize()}`, links over `OccId` | `new(forest, Criterion)`, `from_links(…, Criterion, &[(VertexId, VertexId)])`, `from_proof(&p, Criterion, &Limits, stop)`, `criterion()`, `is_correct(stop)`, `sequentialize(&Limits, stop)`; links over `VertexId` | F10, F12, F49, R79, D-6 |
| `NetError` (exhaustive, `OccId`) | `#[non_exhaustive]`, `VertexId`, `Refused(Refusal)` | F1, R132 |
| `Verdict::Unprovable(Refutation)`; `Refutation::{Unbalanced { atom, name, … }, Equation {…}, StateEquation { weights, once }}` | `Unprovable(Box<Disproof>)`; payload structs, `Unbalanced { atom }` (the name from the sequent), `StateEquation`'s weights for every place | R3, R70, R124, F80 |
| `Reason::{RecursionLimit, CopyBound(u32), MemoryLimit(u64)}` | `RecursionLimit { depth }`, `CopyBound { copies }`, `MemoryLimit { limit_bytes }`, `+ WorkLimit { limit }`, `Unchecked { limit_bytes }`, `Reason::setting()` | F86, R21, R138 |
| `Error::Unchecked` from a search | `Verdict::Unknown(Reason::Unchecked { limit_bytes })` | F139, R138 |
| `prove_until(&s, mode, &o, FnMut() -> bool)`, `prove_goal(forest, &[OccId], mode, &o, stop)`, `engine_for(forest, &[OccId], …)` | `prove_within(&s, mode, &o, &Limits, stop)`, `prove_goal(Goal, mode, &o, &Limits, stop)`, `engine_for(Goal, mode, &o)` | F19, F136, F62 |
| every `impl FnMut() -> bool` stop | `impl FnMut(Progress) -> bool` | F136, R243 |
| `search::Options` (private fields, setters, `memory_limit`, `occurrence_limit`, `recursion_limit`, `test_period(Option<u32>)`) | public fields, serde; the bounds in `Limits`; `test_period: Cadence`; `+ schedule: Schedule` | F77, F78, F79, F81, R1, R47 |
| `Engine` written only, `Bias` without serde, `Engine::parallel` a negative match | `Engine::ALL`, `name`, `FromStr`, serde both ways; `parallel` an exhaustive positive match | F76, F141, R86 |
| `Statistics`' hand-listed proxy, `memo_entries: usize` | `derive(Serialize)`, every counter `u64`, `+ forward_copies, work` | F88, F144, R9, R46 |
| `alone_first` in `cli/src/prove.rs` and `bench/src/run.rs` | `search::race` (feature `parallel`) | F103, F168, R85, R18 |
| `Error` (35 positional variants, not marked) | `#[non_exhaustive]`, named fields, `kind()`, `code()`, `setting()`, `describe()`, `Serialize` (section 4) | F1, F8, F30, F31 |
| the six index errors (`InvalidVariableIndex`, …) | `IndexOutOfBounds { space, index, len }`, `NotTopological { space, index, parent }` | F25, F31 |
| `NetFragment`, `NetMode`, `NetGoal`, `EngineMode`, `NotAdditive`, `NotHorn` | `EngineRefused { engine, because: NotTaken }`; a structure's own refusal is `NetError` | F48, R135, F140 |
| `UnknownRule`, and the name tables of the command and the harness | `UnknownName { what, name, known }` from every `FromStr` | F76, R241 |
| `CheckError::describe`, `NetError::describe`, `ShapeError::describe`; two `Described`, `DescribedShape` | `Error::describe(&forest) -> Described<'_>`, one writer per type | F48, F50 |
| `ParseError` (`Debug`, byte span) | `+ Clone, PartialEq, Eq, std::error::Error`, `span_utf16`, `line`, `column`, `expected` | F29, F34, R129 |
| `DEFAULT_MEMORY_LIMIT` (three copies), `png`/`pdf::Options::memory`, `Options::DEFAULT_OCCURRENCE_LIMIT`, `Forest::DEFAULT_LIMIT`, `ViewOptions::DEFAULT_LIMIT`, `Options::{DEFAULT_RECURSION_LIMIT, stack_size()}` | `Limits::{DEFAULT_MEMORY_BYTES, DEFAULT_OCCURRENCES, DEFAULT_DERIVATION_BYTES, DEFAULT_RECURSION_DEPTH}`, `Limits::{stack_bytes(), recursion_depth_for_stack()}` | F62, F63, R45 |
| `{latex, typst, svg}::{derivation, write, ordinary}` per type, `{latex, typst, svg}::{sequent, two_sided}`, `svg::net(&n, &style, Option<u64>)` | one `write(&impl Drawable, &options, out, stop)` per target; `sequent(&s, &options, &Limits)`; `svg::net(&n, &style, &Limits, stop)` | F36, F5, R26 |
| `png`/`pdf::from_svg(svg, fonts, &options)` | `+ &Limits` | F63 |
| `rocq::Options { form, lemma: String, prelude: String }`, `rocq::Unsupported` (exhaustive) | `lemma` checked to be a Rocq identifier, `prelude: Option<String>`, `Unsupported` `#[non_exhaustive]`; [31] `kernel` | F15, F38, R139, R140 |
| the command's `Styles` | `export::Styles`; `+ Settings`, `Clock` | R141, R142, R148, R183 |
| `batch::{Options, Plan, Problem, Answer}` with public fields, `plan(within: bool)` | `#[non_exhaustive]`, serde on `Options`, `plan(Cores)`, `Results::cancel`, `Cancel`, `run_local` | F10, F73, F74, R40, R48, R150 |
| `ordinary::Sequent::new -> Self`, `Formulas::add` unchecked, `Translation::target() -> &str`, `Image::read_back(&Derivation)`, `ordinary::Derivation::check()` | `-> Result<Self, Error>`, checked, `-> Target`, `read_back(&Proof, &Limits, stop)`, `check(&Limits, stop)`; `+ ordinary::decide` | F12, F13, R24, R113, R249, D-12 |
| `families::Family::instance` (panics) | `-> Result<Instance, Error>` | F6 |
| `sequents::fmt::{Walk, Visit}` crate-private, binary `Between` | public, `Visit::Between(TermId, index)` | R245, D-10 |
| JSON keys `ids`, `var_dict`, `proof`, `memory_limit`, `once`; the mode as an object; `"reason": {"copy_bound": 3}` | `roots`, `atoms`, `nodes`, `limit_bytes`, removed with `StateEquation`'s new payload; the mode by name; tagged reasons and refutations; no reader of the old names, the fixtures regenerated (7.1, 7.5) | T1, F26, F86, D18 |

### 2.5 Open and closed types

`#[non_exhaustive]` [28], beyond today's sixteen marks: the enums
`Error`, `ErrorKind`, `Refusal`, `Phase`, `NetError`, `ShapeError`,
`StepError`, `Fault`, `Split`, `Needs`, `NotTaken`, `Space`,
`rocq::Unsupported`, `Schedule`, `Cadence`, `Labels`, `OpenGoal`,
`typst::Layout`, `svg::Advances`, `ordinary::{Logic, Translation}`,
`lltp::Status`; every options value (`Mode`, `Limits`, `Settings`,
`Clock`, `search::Options`, `batch::{Options, Plan, Problem, Answer}`,
`ViewOptions`, `TextOptions`, each export's `Options`, `svg::Style`,
`Styles`, `ordinary::Options`, `Criterion`); the structs with public
fields `Progress`, `Dyadic`, `Invalid`, `Refused`, `Size`, `Step`,
`Applicable`, `Unbalanced`, `Equation`, `StateEquation`, `ParseError`,
`Statistics`, `Outcome`; every struct-like variant of a marked enum.
The option-valued enums are options (D15) and open too: `Compact`,
`Sides`, `Form` (31 reserves `Form::Check`), `Cores`, `Jobs`, `pdf::Date`; and
the public types the list would otherwise miss: `svg::Font`,
`mist::Safety`, `ordinary::{Target, Outcome}`, `rocq::Identifier`,
`typst::Length`, `Order` (36). Closed on purpose (P3): `Term`, `Kind`,
`Node`, `Rule`, `Verdict`, `Branch`, `Sign`, `Polarity`, `Side`,
`ordinary::{Node, Rule, Side}`, and `sequents::fmt::Visit`; `Rule` gains `Cut`
at 34 and `Forall`, `Exists` at 38, `Node` gains `Cut` at 34 and
`Forall`, `Exists` at 38, `Term` and `Kind` gain `Forall`, `Exists` at
38. `Engine` and `Reason` are open enumerations whose JSON strings a
client treats as an open set (AIP-126).

## 3. The data model

### 3.1 `Sequent`

```rust
/// A one-sided sequent in negation normal form: root formulas over an
/// arena of shared subformulas, in the order they were written.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Sequent {
    terms: Vec<Term>,            // topological: a term names only earlier terms
    roots: Vec<TermId>,          // in written order (C1): never sorted
    atoms: Vec<String>,          // the atom table: one distinct key per atom (3.2)
    antecedents: Option<u32>,    // the roots written left of ⊢, which come first; None: not known
    // [34], private and empty without a cut: cuts: Vec<TermId>, the cut formulas, numbered
    //   by the forest after the roots
    // [38], private and empty for propositional input: 3.14's tables
}
const _: () = assert!(size_of::<Term>() == 12);
```

- **The canonical form is the written one** [28] (C1, answer A).
  `optimize` merges equal atoms and terms and drops unreachable ones; it
  never sorts the roots. Parse, JSON, `add`, `Display` and
  `Forest::roots()` keep the roots as given; arena indices follow the
  first occurrence along the roots. `Eq` and `Hash` are structural (`⊢ A,
  B` and `⊢ B, A` are two values; a cyclic sequent is not equal to its
  rotations, which are the search's question at 36), so a `Sequent` means
  one thing in every mode (C1 rejects the mode-dependent order). The one
  commit that stops sorting regenerates the snapshots and README blocks
  whose roots were not ascending, names them as its exemption from the
  behaviour lock, lists the target rows whose root order changed (their
  counters may move: such a commit is a change of the search's input, run
  through `bench/targets.sh` and recorded as a new oracle), and updates
  `ordinary::translate`'s root pairing to pair by position.
- **The written sides are kept** [28] (H9, H10, R54, R74). The parser
  sets `antecedents` to `Some(k)` for every text it reads (`Some(0)` for
  `|- Γ`: every text sequent has a turnstile), `lltp::read` and
  `mist::read` through the parser or their own count, `ordinary::
  translate` from the ordinary sequent's sides. `None` means the sides are
  not known: a JSON sequent without the key, `Sequent::add`, a sequent
  built in code that names no sides. The forest and the engines
  ignore it; the reading takes the goal from it (3.6). It is part of
  equality. A JSON sequent writes the key whenever it is `Some`, `0`
  included, so a round trip keeps the rule (both judges caught a draft
  that dropped `Some(0)`).
- **The cut formulas belong to the sequent** [34] (walk-through 34): a
  sequent with cuts is its conclusion (the roots) and its cut formulas
  (`cuts`, each `A` of a pair `A`, `A⊥`), so `Proof`, `Disproof`, `Goal`,
  `Interactive` and the wire carry them through the `Sequent` they hold,
  with no parallel parameter; `Forest::new` numbers them after the
  conclusion (3.3), `forest.sequent()` is that value and `roots()` the
  conclusion. The field is private and empty until 34, and a cut-free
  sequent writes no key.
- **Readers name their bound** [28]: `Sequent::parse_within(text,
  &Limits)` refuses at the first term past `limits.occurrences`, before
  allocating (F16); `FromStr` is that under the defaults. Identifiers are
  normalized to NFC on every path (HD3, a dependency behind `parse`
  through `new-tool`); a JSON sequent's atom names must be identifiers of
  the text syntax that are no keyword (`par`, `top`, `bot`) and no word
  reserved for later steps (`forall`, `exists`), which a text reader
  would read otherwise (HD5, H19: `Error::AtomName`; walk-through 30).
  Every reader refuses the reserved words from 28: `parse_within` and
  `FromStr` as identifiers, with a message that names the word as
  reserved (`|- forall` reads as an atom today), and the JSON, LLTP and
  `.spec` readers as atom names (`AtomName`); `∀` and `∃` are refused
  already. So 38's binders are new tokens, never a new meaning of text
  that reads today; no fixture and no README example has such an atom,
  so the lock does not move. The text syntax grows by new tokens only;
  the reserved words are listed on `Sequent`. [36] adds an ordered parse as a sibling of
  `parse_within` that takes the order, since the dual of a product
  reverses its operands there (10.8); `FromStr` stays the commutative
  reader.
- **A checked builder** (R59) [32 or 34, whichever first needs it]:
  `Sequent::builder() -> Builder` with `atom(name)`, `term(Term)` (indices
  checked, hash-consed as the parser does, so a built sequent equals the
  parsed one), `dual(TermId)` (iterative NNF), `left(TermId)`,
  `right(TermId)` (the written sides), `finish(&Limits)`. Additive, so it
  waits for its first caller; 38 adds `predicate`, `function`, `bind`.

### 3.2 `Term`, `Kind`, `Atom`: an atom is an atomic formula

```rust
/// An atom: an atomic formula, by its index in its sequent's atom table.
/// Every atom is nullary (a propositional variable) until first-order
/// logic gives atoms arguments.
pub struct Atom(u32);
pub struct TermId(u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Term {                         // closed (P3)
    Atom(Atom), DualAtom(Atom),         // were Var, DualVar (F25, R251)
    One, Bot, Top, Zero,
    Tensor(TermId, TermId), Par(TermId, TermId), With(TermId, TermId), Plus(TermId, TermId),
    Bang(TermId), Quest(TermId),
    // [38] Forall(TermId), Exists(TermId): a binder and its body, locally nameless
}
#[repr(u8)]
pub enum Kind { Atom, DualAtom, One, Bot, Top, Zero, Tensor, Par, With, Plus, Bang, Quest }
//   [38] appends Forall, Exists; today's discriminants kept
const _: () = assert!(size_of::<Kind>() == 1);
```

**The decision** (section 14, decision 1, answered by the author on
2026-10-09).
An `Atom` is an *interned atomic formula*: a predicate symbol applied to
argument terms, hash-consed in the sequent's atom table, nullary for
every propositional atom. `Term` keeps `Atom`/`DualAtom` as its only
literal variants and gains only the two binders at 38; only a binder sets
the quantifier bit. This departs from the research's recommendation
(D-4, R52, R62, R16: `Term::{Pred, DualPred}(Atom, ArgsId)` beside the
atoms, `Atom` a predicate symbol, and a fragment bit set by any argument
so that no propositional engine pairs `p(a)` with `~p(b)`), which all
three drafts followed, and both judges chose it over them. The reasons:

- **Soundness by construction, not by guard.** Eight places pair literals
  by atom (the checker's `Ax`, the oracle, `nets::dual`, the additive
  path, the net engine's partner lists, the focused engine's `initial`,
  `dual_in` and `dual_from`, the Horn engine's places;
  `impact-quantifiers.md` §1 item 1). Under the drafts' design each is
  safe only while the bit reaches it; one missed guard, a forced engine
  or a wildcard arm accepts `p(a)` against `~p(b)` as an axiom, silently.
  With interned atoms the eight places compare atom ids, which for ground
  atoms is exactly syntactic equality of atomic formulas: a ground
  first-order sequent is a propositional sequent with structured atom
  names, which every engine, the checker and the counts decide correctly
  as they are. Open atoms exist only under a binder, whose kinds are new
  variants, so every exhaustive `match` fails to compile until it
  decides.
- **D17**: two `Term` variants instead of four, no new literal kind on a
  hot path, the literal lists still `2·atom + sign`. The spike measured
  the drafts' four-variant superset (section 11), which bounds this.
- **Practice**: ground first-order problems (parametrised Petri nets,
  planning) are decided by every engine the day the data model lands.
- **Its costs, at 38**: an interning map for atoms by `(symbol,
  arguments)` in `optimize`, the parser and `add`; a symbol → atoms index
  for the first-order engine's candidates (a CSR over the atom table, per
  atom, built only when a symbol has an arity above 0); fragment names
  for ground first-order input stay propositional (`MLL`, which is true of
  its logic). And **where construction stops**: an open atom can become
  any ground atom of its symbol, so the counts, the count equation and
  the bias count a symbol that has an open atom per symbol, over all its
  atoms, through the index; only a symbol whose atoms are all ground
  keeps today's count per atom (10.10 (f)). Counting `p(a)` alone in
  `∀x.p(x) ⊢ p(a)` would refute a provable sequent as `Unbalanced`: this
  is a guard, like the pairing sites' binder bit below, not a consequence
  of the representation. On the wire a structured atom table is a new
  level (7.1).

What step 28 fixes for it [28] (walk-throughs 29 and 38): `Atom`'s
documentation as above; **the atom table holds one distinct key per
atom**, and `Sequent::atom_name(a)` is that key (the name of a nullary
atom; at 38 the canonical text of a ground atom, `p(f(a), b)`, and of an
open one, `p(#0)`), so that every caller that uses names as keys today
(the Rocq writer's identifiers, the harness, a translator) stays right
when atoms gain arguments; `Sequent::atom_count()` for the uses of
`atom_names().len()`; `Sequent::atom(name)` the lookup by key. Step 38
adds `atom_symbol(a)` and `atom_arguments(a)`. `Fragment` documents the
reserved quantifier bit (no constant until 38). `Term::operands()` stays
the *formula* children; an atom's arguments never reach the forest.

**Open atoms** (walk-through 38). Under a binder an atom's arguments hold
bound variables, and two literals of one open atom with opposite signs
(`∃x.p(x) ⊢ ∀x.p(x)`) are no axiom: there the pairing sites are guarded
by the binder bit, as the drafts' design guarded every predicate. The
guard is exact (an open atom exists only under a binder, `Sequent::check`
refuses a loose bound variable at a root), it is set by one kind in one
pass, and step 38's first commit tests it by forcing every engine and the
checker on such a sequent and expecting the refusal;
`Forest::dual_literals` asserts in debug builds that both atoms are
closed. Ground atoms need no guard at the pairing sites; at the counts
a symbol with an open atom is counted per symbol (above).

**The literal variants come first** [28] (the spike, 11.5): `Term`'s and
`Kind`'s literal variants are the first two, contiguous, and a variant is
added after the compound ones, never between, so `Kind::is_literal` and
`Term::atom()` stay one comparison on the focused engine's hot path; a
test of `Kind`'s `repr(u8)` discriminants pins it. Appending `Pred` and
`DualPred` after `Quest` cost the focused engine's journeys 6.6 %.

### 3.3 `Forest`: its contract [28]

The forest is the numbering everything else names; its doc states three
promises, each pinned by a test (R69, D-2):

1. **The numbering is a public contract.** Depth-first preorder over the
   roots in written order, the left subterm before the right, a unary
   operand (and at 38 a binder's body) at `o + 1`, so `subtree(o) == o ..
   o + size(o)`; a pure function of the sequent (and at 34 of its
   recorded extra trees). A foreign checker recomputes it: a fixture
   (`core/tests/fixtures/forest.txt`, one occurrence a line: id, kind,
   parent, atom) pins the numbering of a dozen sequents, every connective
   included; a Rust test checks it against `Forest`, and step 31's Rocq
   test suite reads the same data file (walk-through 31).
2. **`roots()` is the conclusion; `ids()` is every occurrence.** A later
   step numbers trees past the conclusion's occurrences (34's cut pairs,
   `A` then `A⊥`, so that `dual(x) = root(A⊥) + (x − root(A))` for the
   commutative dual; in an ordered mode cut is refused until a step asks
   for it, and then the dual is a table). Code that means "the goal is the
   sequent" compares with `roots()`, code that means "every occurrence"
   uses `ids()`. At 28 this is documentation; 34 adds `cut_pairs()`,
   `dual()` and `is_conclusion()`.
3. **Literal lists are candidates; one predicate pairs them.**
   `literals(atom, sign)` lists the literal occurrences of an atom and
   sign; whether two literals close an axiom is `Forest::dual_literals(x,
   y)` (same atom, opposite signs), which the eight pairing places call
   (F58). The checker calling a forest accessor shares no engine code. At
   38 an instance comparison through frames sits beside it (3.14).

Constructors: `Forest::new(&s)` (the defaults) and `Forest::within(&s,
&Limits)`, which refuses more than `limits.occurrences` before
allocating (F27); every reader of a form with a sequent builds through
it. About 25 bytes an occurrence, outside `limits.memory_bytes` (its
doc says so, R29): the public constant `Forest::BYTES_PER_OCCURRENCE`
is that figure, which a `const` assertion on the per-occurrence arrays'
element sizes guards (R63) and a front end adds to the memory bound to
size a process (R18; walk-through 29). `Forest::formula(o)` prints the
formula at `o`; from 38 it names the bound variables of the binders
above `o` from the parents. `lca` costs the tree's depth, as its doc
says.

### 3.4 `Member`: what a sequent's member is [28]

```rust
/// A member of a sequent of its owner (a proof, a derivation, a session,
/// a goal): below `forest.len()` the occurrence with that id; past it, an
/// entry of an instance table the owner keeps (an occurrence under bound
/// terms, from 38). A net's vertices are `VertexId`s, not members (3.11).
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize), serde(transparent))]
pub struct Member(u32);
impl Member {
    pub const fn new(raw: u32) -> Self;
    pub const fn get(self) -> u32;
    pub const fn index(self) -> usize;
    /// The occurrence, when the member is one of the forest's own.
    pub fn occurrence(self, forest: &Forest) -> Option<OccId>;
}
impl From<OccId> for Member {}
const _: () = assert!(size_of::<Member>() == 4);
```

- **Every public list of a sequent's members is `[Member]`** (F19,
  R244, D-1): `Node`'s operands (3.7), `Dyadic`, `Inference::sequent()`,
  `Interactive::goal`, `Goal`, `Fault`'s payloads, the session form's
  `sequent`, the command's goal line, and the crate-private `Drawn::
  sequent` and `Notation::sequent`. `Forest`, `Reading` and
  `ProofStructure::vertex` keep `OccId`: they speak of occurrences.
- **By offset, not by owner** (P2): a stored id below `forest.len()`
  means the occurrence in every owner, so the ground case needs no table
  and every propositional form is byte-identical forever; under binders,
  interning keeps one member per instance, and ties between instances of
  one occurrence are ordered by the table's order, a function of the
  proof, never of the order a pool created them.
- **Owners map members back**: `occurrence(m) -> OccId` and `formula(m)
  -> impl Display + '_` on `Proof`, `Derivation` and `Interactive`, the
  identity and `Forest::formula` today, a table lookup and an instance
  printed through its frame at 38, so a caller written against 0.1.0
  stays right (`Member::occurrence(&Forest)` cannot reach an owner's
  table).
- **Why one integer and not a pair** (fo-linear's `(OccId, frame)`): a
  pair changes the shape of `ax: [x, y]`, of the session's `sequent` and
  of every stored file, and `Node` stays 16 bytes only with a one-`u32`
  operand; the table a member indexes is where the frame lives (3.14).
- Engines keep `OccId` inside: a propositional goal is all occurrences
  (the front door converts, O(goal)).
- **Across owners** [38] (walk-through 38): two framed members of
  different owners are equal when their occurrences and resolved frames
  are, which the crate's `Instances::import` decides; `Disproof` gets a
  private instance table with an additive `with_instances`, as `Proof`
  does, and an outcome of a framed goal reports the bindings it made.
  [34] adds `dual(m)` to the owners' methods.

### 3.5 `Mode` and `Fragment` [28]

```rust
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct Mode { intuitionistic: bool, affine: bool, mix: bool }
//   [36] adds, private: order: Order, nonempty_antecedents: bool (L rather than L*;
//   false, unrestricted, by default, so a commutative mode has one value)
impl Mode {
    pub const CLASSICAL: Self; pub const INTUITIONISTIC: Self;
    pub const fn is_intuitionistic(self) -> bool; pub const fn is_affine(self) -> bool;
    pub const fn has_mix(self) -> bool;
    #[must_use] pub const fn with_affine(self) -> Self; #[must_use] pub const fn with_mix(self) -> Self;
    /// The word read back by `FromStr`: classical, affine, mix, affine-mix,
    /// intuitionistic, intuitionistic-affine.
    pub const fn name(self) -> &'static str;
    pub const NAMES: &'static [&'static str];
    /// Refuses a combination no calculus has: intuitionistic with Mix today,
    /// at 36 an ordered mode with weakening or Mix.
    pub fn check(self) -> Result<(), Error>;
}
impl FromStr for Mode { type Err = Error; }        // UnknownName, listing NAMES
```

- Private fields, getters and builders, so step 36 adds a field without a
  break and a combination is checked where it is built (R51).
- **One table of words** (R241) replaces those of `cli/src/batch.rs` and
  `bench/src/problems.rs`; weakening with Mix is `affine-mix` (the
  harness wrote `mix-affine` and the command's batch `affine`; no
  committed baseline holds the word). `Display` stays the prose the
  command prints ("classical affine with Mix"). `Modes::take` in the
  dispatch destructures the whole mode, so a field added at 36 is a
  compile error there (F140).
- **On the wire a mode is its name** (AIP-126): `"mode":
  "intuitionistic"`. An unknown word is refused naming the known ones, so
  36's `cyclic`, `lambek` and `lambek-star` need no key and cannot be
  read as commutative (R10). The pre-release object of flags is not
  read (7.1).

```rust
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Fragment(u8);   // bits 1, 2, 4, 8, 16 used; 32 reserved for QUANTIFIERS [38]
```

- The named constants `MLL` … `LL` are documented as propositional
  ("every propositional connective" for `LL`; a ground first-order
  sequent is propositional, 3.2); the Rust constant
  `Fragment::ALL` becomes `ADDITIVE` (its name stays `ALL` on the wire
  and in `Display`, F56); `NAMED` and `FromStr` (with or without the
  `I` prefix) are public (F76). `Fragment::has_nets()` is the one
  predicate of where a proof structure exists (F28, R133), which
  `ProofStructure` and the command ask; each engine's `admits` keeps its
  own largest fragment (8.6), since at 33 structures take MELL while the
  net engine stays at MLL (walk-through 33).
- [38] `QUANTIFIERS`, set by a binder in `Sequent::fragment`'s one pass
  (3.2); its names append `1` (`MLL1`, …, `IMLL1` intuitionistically).
  Ordering is no connective class, so 36 takes no bit unless its
  divisions need one.

### 3.6 `Reading`: the goal is what was written [28]

`Reading<'a>` keeps borrowing its forest (P1) and its accessors. The
choice of the goal and of an implication's antecedent changes (H9, H10,
R54; both judges, on a check of the three drafts' rules against the two
witnesses, found only this one refuses both):

- **With the sides known** (`antecedents == Some(k)`): the roots after
  the first `k` are the written succedents; intuitionistic mode needs
  exactly one, the goal (`ShapeError::Succedents { count }` otherwise, which
  refuses H10's `|- a, top` and its mirror `|- top, a`); every
  antecedent must read as input; and an implication's antecedent is the
  factor the lowering put there (the left one, D1), so the symmetric
  reading that turned `(A -o bot) -o bot |- A` into `1 ⊸ (A ⊗ 1) ⊢ A` is
  not tried and the input is refused as `bot |- A` is (H9).
- **With the sides unknown** (`None`; decision 21): the reading answers
  only where it is the one reading. The goal is the one root that can be
  it, every other root reading as input; where two roots can each be the
  goal (only formulas of `⊤` and `0` stand on either side, as in `⊢ 0,
  ⊤`) the reading refuses, `ShapeError::Undetermined { first, second }`,
  with a message that says to give the sides (`antecedents`, or the text
  form). An implication's antecedent is the left factor, as with the
  sides known: the JSON keeps its operands in the order written (`optimize`
  does not reorder them), so there is no symmetric reading anywhere, and
  H9's one-sided form is refused from JSON as from text. `⊢ ⊤, a` has one
  reading, `0 ⊢ a`, since `a` cannot be a hypothesis, and is answered:
  H10's question, two succedents, needs the sides, and a JSON sequent
  that asks it writes `"antecedents": 0` and is refused as the text is.
- **What it changes for the user**: an intuitionistic sequent is written
  two-sided. Text written one-sided with several roots (`|- ~A, B`,
  read today as `A ⊢ B`) is refused with a message that says to write
  `A |- B`; `B par ~A` in a two-sided `-i` input is refused, as the
  reading's own error text already says (`⅋ only as A ⊸ B, that is ~A ⅋
  B`); the lock's `error-not-intuitionistic` call (`|- ?A, !~A`) gets
  the succedent message. The commit names these in README and in its
  lock change. A JSON sequent without the key is read by decision 21's
  rule.
- Both witnesses run on the binary of 2026-10-09 (session `step-28c`):
  `prove -i '|- top, a'` and `prove -i '(A -o bot) -o bot |- A'` both
  answer `provable`, exit 0; they are the tests of this change, as text
  and as JSON without `antecedents` (H9's refused by the factor rule,
  `⊢ ⊤, a` answered as `0 ⊢ a`, and with `"antecedents": 0` refused), with
  `⊢ 0, ⊤` without sides refused as undetermined. Neither the reading nor
  anything after it guesses: `Interactive`, `Derivation`, the checker,
  the oracle and the Horn engine take the reading the front door or the
  session made and choose nothing of their own.
- **The rule is written down** (walk-through 31): the position grammar
  of `core-forest.md` (output: `⊗ ⊕ & ! 1 ⊤ 0`, atoms, `A ⊸ B` stored `A⊥ ⅋
  B`; input: the duals; the flip only at an implication's antecedent),
  the goal's choice and the implication's factor move into `Reading`'s
  rustdoc as the normative text, in the style of 3.8's table, which step
  31's Rocq certificate checks positions against. The left-factor rule
  is the commutative one; [36] adds `Reading::of_mode(forest, mode)`,
  under which an ordered mode takes the antecedent from whichever factor
  is in input position and refuses `None` sides and an empty antecedent
  (L).
- `Interactive` and `Derivation` keep the positions they computed (a
  `Box<[Side]>`, a byte per occurrence) and hand out a `Reading<'_>`
  in O(1) (F66, R93).
- **The public walks** (R245, D-10; walk-through 29), shapes fixed at 28
  since a translator outside the crate builds on them:

  ```rust
  /// A stop of a formula's walk, in the order it is written.
  pub enum Visit<T> { Enter(T), Between(T, u32), Exit(T) }   // closed: a binder is an Enter
  impl Sequent { pub fn walk(&self, root: TermId) -> Walk<'_, TermId>; }   // Iterator<Item = Visit<TermId>>
  impl Reading<'_> {
      pub fn walk(&self, o: OccId) -> Walk<'_, OccId>;
      /// The intuitionistic connective an occurrence is under its position.
      pub fn connective(&self, o: OccId) -> IllConnective;   // Atom, One, Zero, Top, Bang, Tensor, With, Plus, Lolli
  }
  ```

  A leaf yields `Enter` alone; a unary node `Enter`, its operand, `Exit`;
  a binary one `Enter`, the first, `Between(t, 1)`, the second, `Exit`;
  [38] a binder is an `Enter` of its kind with one operand, and an atom's
  arguments are walked by a term walk of the same shape over `FoTermId`.
  `IllConnective` (`#[non_exhaustive]`) is the table `IllFormula`'s
  `Display` uses, so no translator copies the reading's case analysis.
  `core/tests/depth.rs` runs both at 100 000 levels. [34] reads a cut pair's `A` as output and `A⊥` as input,
  choosing the goal among the conclusion's roots only (R99). [36] adds
  the planar order (10.8). [38] `∀` and `∃` keep their position.

### 3.7 `Proof` and `Node` [28]

```rust
/// One rule instance of the dyadic calculus `⊢ Θ ; Γ`. Closed (P3): a new
/// variant is a 0.y bump and a constructor of the Rocq mirror.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Node {
    Ax(Member, Member),
    Tensor(Member, NodeId, NodeId), Par(Member, NodeId),
    One(Member), Bot(Member, NodeId),
    With(Member, NodeId, NodeId), Plus(Member, Branch, NodeId),
    Top(Member), Bang(Member, NodeId), Quest(Member, NodeId),
    Copy(Member, NodeId), Weaken(Member, NodeId), Mix(NodeId, NodeId),
    // [34] Cut(Member, NodeId, NodeId)    the A root of a cut pair; left Γ, A, right Δ, A⊥
    // [38] Forall(Member, Eigen, NodeId), Exists(Member, FoTermId, NodeId)
}
const _: () = assert!(size_of::<Node>() == 16);
impl Node {
    pub const NAMES: &'static [&'static str];       // one per variant, in order: what Rocq's constructors match (R118)
    pub const TAGS: &'static [&'static str];        // the wire tags (`⊕₁` and `⊕₂` are one variant)
    pub const fn principal(self) -> Option<Member>;
    pub fn members(self) -> impl Iterator<Item = Member>;   // was occurrences()
    pub fn premises(self) -> impl Iterator<Item = NodeId>;
    pub const fn name(self) -> &'static str;
}

/// A proof: nodes over a forest it owns, premises before conclusions, the
/// root last, and the conclusion it proves (the sequent, or a goal).
pub struct Proof { forest: Forest, nodes: Box<[Node]>, goal: Option<Box<[Member]>>, mode: Option<Mode> }
impl Proof {
    pub fn new(forest: Forest, nodes: Vec<Node>, root: NodeId) -> Result<Self, Error>;
    pub fn new_of_goal(forest: Forest, goal: &[Member], nodes: Vec<Node>, root: NodeId) -> Result<Self, Error>;
    #[must_use] pub fn with_mode(self, mode: Mode) -> Self;
    pub fn goal(&self) -> Option<&[Member]>;     // None: a proof of the sequent
    pub fn mode(&self) -> Option<Mode>;          // the mode it is meant for: a claim, not a check (R7)
    pub fn occurrence(&self, m: Member) -> OccId;
    pub fn formula(&self, m: Member) -> impl Display + '_;
    pub fn check(&self, mode: Mode) -> Result<(), CheckError>;
    pub fn check_within(&self, mode: Mode, limits: &Limits, stop: impl FnMut(Progress) -> bool)
        -> Result<(), CheckError>;
    // the views: 3.9; unchanged: forest, sequent, nodes, node, ids, root
}
```

- **Every operand is a member** (3.4): under `?` copies with quantifiers
  one occurrence has several instances, and the bottom-up checker can
  recover neither a leaf's instance nor a `⊗`'s principal from the nodes
  below it (impact-quantifiers finding 2).
- **A proof records its conclusion and its claimed mode** (F89, F23, R7).
  `prove_goal` off the conclusion sets `goal`; `check` checks a term
  against its own conclusion, so a goal proof is checkable and never "a
  proof of the sequent the checker rejects". `goal().is_none()` is what
  `linlog check`, `from_proof` and the Rocq writer require
  (`Error::GoalProof` otherwise, kind `unsupported`: a goal proof is no
  wrong proof, only not one of the sequent, so `linlog check` exits 2 on
  it, not 1, and the harness counts it as refused, never as invalid). The search sets `mode`; a reader takes the optional `mode`
  key; `check` and the exporters still take a mode explicitly.
- **Cut formulas go through the forest** (D-2): 34 builds the forest
  with its cut pairs first, and `Proof::new` keeps its signature;
  **witnesses out of line** (R125): 38's `Proof::with_instances(forest,
  nodes, root, instances)` is additive and owns the instance table, the
  frames and the proof-local term extension (3.14). No dummy argument
  now: those tables have no propositional value (D-5's other option,
  side tables in `new` from 28, set aside).
- **The variant policy** (R118): a new `Node` is a 0.y bump, a wire tag,
  its `Fault` arms, its oracle arm, a `Rule`, a constructor of the Rocq
  `node`, a row of 3.8's table; the test that compares `Node::NAMES` with
  the Rocq constructors (and the branch of `Plus` separately) fails until
  all exist. [34] `principal()` of a `Cut` is `None`, as for `Mix`; its
  `members()` yield the cut formula's occurrence. Every `match` on `Node`
  names its variants, `from_proof` and the exporters included.
- **Invariants**, unchanged and stated on the type: premises precede,
  root last, `new` keeps what the root reaches and renumbers, 1 to 2³² − 1
  nodes (`Refusal::Index`), operands in range (`IndexOutOfBounds`),
  `new` does not check.

### 3.8 The checker, specified [28]

`Proof::check(mode)` and `check_within(mode, &Limits, stop)`; the pass
(`check::examine`) stays crate-private with its observers. New at 28:
the stop, polled every 4 096 nodes with the nodes passed as work (F12,
R23); the limits (`memory_bytes`, `work`); the error type; and this
specification, which goes into `core-proofs.md` and the rustdoc of
`proofs::check` as the text step 31's Rocq function is written against
(R117).

```rust
#[non_exhaustive]
pub enum CheckError {
    Invalid(Box<Invalid>),     // the term is not a proof of its conclusion in the mode: a verdict
    Refused(Refused),          // given up without a verdict: a bound or the stop (no premises)
}
#[non_exhaustive] pub struct Invalid { pub node: NodeId, pub rule: Node, pub premises: Vec<Dyadic>, pub fault: Fault }
#[non_exhaustive] pub struct Refused { pub node: NodeId, pub refusal: Refusal }
#[non_exhaustive] pub struct Dyadic { pub theta: Vec<Member>, pub gamma: Vec<Member>, pub any: bool }
#[non_exhaustive]
pub enum Fault {   // was Problem, without Memory; data in named fields (P3)
    Forbidden, Shape(ShapeError), Succedents { count: usize }, Kind { member: Member }, NotDual,
    Missing { premise: usize, member: Member }, NotEmpty, Differ, NotUnderQuest { member: Member },
    Surplus, Conclusion { derived: Dyadic },
    // [34] NotACut { member }  [36] Order, EmptyAntecedent  [38] Instance, Eigenvariable, Witness
}
```

**The rules.** A state is `(Θ, Γ, any)`: `Θ` the least set of
unrestricted members the subproof needs, `Γ` a multiset, `any` whether a
`⊤` above absorbs further linear context. `take(S, x)` removes one `x`
from `Γ`, or succeeds absorbed if `x` is absent and `any`, else
`Missing`; `put(S, x)` adds one; `S ⊎ T = (Θ ∪ Θ', Γ + Γ', any ∨ any')`;
`l`, `r` are the children of the principal (at 38, in a framed owner,
the members of the children under its frame).

| node | premises | requires | derives |
|---|---|---|---|
| `Ax(x, y)` | | `dual_literals(x, y)` (`Kind`, `NotDual`) | `(∅, {x, y}, no)` |
| `One(o)` / `Top(o)` | | kind `1` / `⊤` | `(∅, {o}, no)` / `(∅, {o}, yes)` |
| `Bot(o, p)` | S | kind `⊥` | `put(S, o)` |
| `Par(o, p)` | S | kind `⅋` | `put(take(take(S, l), r), o)` |
| `Tensor(o, p, q)` | S, T | kind `⊗` | `put(take(S, l) ⊎ take(T, r), o)` |
| `With(o, p, q)` | S, T | kind `&`; after `take(S, l)`, `take(T, r)`: equal `Γ` if neither absorbs; the absorbing `Γ` within the exact one (result exact); both absorbing: pointwise maximum, absorbing; else `Differ` | `put((Θ ∪ Θ', Γ, any), o)` |
| `Plus(o, s, p)` | S | kind `⊕` | `put(take(S, child s), o)` |
| `Bang(o, p)` | S | kind `!`; `take(S, l)` leaves `Γ = ∅` (`NotEmpty`) | `(Θ, {o}, no)` |
| `Quest(o, p)` | S | kind `?` | `put((Θ ∖ {l}, Γ, any), o)` |
| `Copy(a, p)` | S | `a`'s parent is a `?` (`NotUnderQuest`) | `(Θ ∪ {a}, Γ of take(S, a), any)` |
| `Weaken(o, p)` | S | affine, or `o` a `?` (`Forbidden`) | `put(S, o)` |
| `Mix(p, q)` | S, T | Mix, not intuitionistic (`Forbidden`) | `S ⊎ T` |
| [34] `Cut(a, p, q)` | S, T | `a` an occurrence of the `A` tree of a cut pair (`dual(a)` defined; elimination moves cuts below the root) | `take(S, a) ⊎ take(T, dual(a))` |
| root | S | `Θ = ∅` and `Γ` = the conclusion, or `Γ` within it if absorbing (`Conclusion`) | |

Intuitionistic mode adds, under the reading (`Shape` without one): (R1)
every derived `Γ` holds at most one output member, exactly one unless
absorbing; (R2) `take` absorbs an absent output only if the premise has
no output; (R3) `Weaken` never weakens an output; Mix is `Forbidden`;
failures are `Succedents`. [36] adds (R4) for Lambek: every derived
sequent keeps an input (`EmptyAntecedent`), and the ordered checks
(`Order`); [38] `Ax`'s instance comparison, the frames of `Forall` and
`Exists`, and the eigenvariable set flowing up with the state (an
`Exists` adds its witness's, a `Forall` removes its own, the root's is
empty, each eigenvariable introduced by one `Forall` node, counted by
node since a memo hit shares a subproof; fo-linear §4.2, for 38's
panel). When a proof has no instance table the pass is today's, the fast
path a branch per proof, not per node; with one, the checker interns
instances in an overlay of its own over the proof's table (P4: the table
is the engines' product, read only), and the eigenvariable set flows as a
persistent structure, not copied per state (walk-through 38). [36]: every
public entry that takes a mode (the checker, `Interactive::within`,
`ProofStructure::from_proof`, the exporters) refuses an ordered mode
until its ordered rule exists, through one `Mode::is_ordered()` guard and
a test that feeds each entry the ordered modes.

**Outside the verified function** (P4). `Surplus` exists because Rust's
counters are finite; over `nat` such a zone fails at the root, so the
verdicts agree (a lemma at 31, or 31's report says why not). `Refused`
has no counterpart; soundness needs only "accepted implies derivable".
`oracle.rs` (test-only) is kept, gets every new arm, and
`agrees_with_the_first_implementation` covers each new node kind: it is
the text the Rocq function translates (R117, R120).

**The integers** (S3) rest on the same four facts as today
(`core-proofs.md`): fewer than 2³² nodes, two premises at most per node
(`Cut` too, R31), fewer than 2³² − 1 occurrences, zones within
`Bag::MOST` after `within`. A framed owner adds members, not premises;
its table is bounded by the nodes and charged to the same memory bound
(R126).

### 3.9 `Rule`, `Named`, `Derivation`, `Inference`, `ViewOptions` [28]

```rust
/// A rule of the one-sided standard calculus. Closed (P3).
pub enum Rule { Ax, Tensor, Par, One, Bot, With, PlusLeft, PlusRight, Top,
    Promotion, Dereliction, Contraction, Weakening, Mix, AffineWeakening, Open }
    // [34] Cut   [38] Forall, Exists
impl Rule {
    pub const ALL: &'static [Rule];
    pub const fn name(self) -> &'static str;
    pub const fn is_structural(self) -> bool;
    pub const fn premises(self) -> u8;         // replaces the `matches!` sites (impact-boxes 7)
    pub const fn has_principal(self) -> bool;
}
/// A rule as a derivation names it: the one-sided rule and, two-sided,
/// the side of ⊢ its principal stands on (⊸L is ⊗ on the input side).
#[non_exhaustive] pub struct Named { pub rule: Rule, pub side: Option<Side> }
//   [36] adds the orientation of a division
impl Named {
    pub const fn new(rule: Rule, side: Option<Side>) -> Self;  // a downstream crate's (rule, side)
    pub const fn name(self) -> &'static str;                  // ⊸L, ⊗R, &L₁, !c, …
}
impl FromStr for Named {}  impl From<Rule> for Named {}
```

`Rule` as a classical rule and a position (F60, the step's own item): the
eighteen two-sided variants become `(rule, side)`, consumers lose their
`unreachable!` arms, 38 adds two rules instead of six, and 36's divisions
are a side and an orientation. Label tables index `(rule, side)`; a test
round-trips every `Named` through `name` and `from_str`. Every form
writes `Named::name`, so none changes. `Rule` and `Named` live in a file
of their own (`proofs/rule.rs`).

```rust
#[non_exhaustive] #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ViewOptions { pub compact: Compact, pub sides: Sides }
pub enum Sides { #[default] Auto, One, Two }   // Auto: two-sided exactly when the proof's mode is intuitionistic

impl Proof {
    pub fn derivation(&self) -> Result<Derivation<'_>, Error>;     // the defaults
    pub fn derivation_within(&self, view: &ViewOptions, limits: &Limits,
                             stop: impl FnMut(Progress) -> bool) -> Result<Derivation<'_>, Error>;
    pub fn derivation_size(&self, view: &ViewOptions, limits: &Limits,
                           stop: impl FnMut(Progress) -> bool) -> Result<Size, Error>;
}
#[non_exhaustive]
pub struct Inference { /* private */ }
impl Inference {
    pub fn sequent(&self) -> &[Member];   // ascending with repeats; [36] the written sequence when ordered
    pub fn rule(&self) -> Named;
    pub fn principal(&self) -> Option<usize>;   // a position in `sequent`; None for ax, Mix, Cut
    pub fn premises(&self) -> &[InfId];
    pub fn times(&self) -> u32;           // a run of one structural rule, in the compact view
    // [38] datum() -> Option<Binding>: the witness or the eigenvariable
}
```

- A derivation's bounds are `limits.derivation_bytes` and
  `limits.memory_bytes`; a derivation past the first is
  `Refusal::Output { estimate_bytes, limit_bytes, least_bytes }`, the last
  the compact view's lower bound when it was tried (F65); `Size` is
  `#[non_exhaustive]`, a saturated count is `u64::MAX` with `exact: false`
  (F70, R247).
- `Derivation` gains `occurrence(m)`, `formula(m)` (3.4) and a written-only
  form (R4, 7.3): a front end that draws its own tree reads it; a client
  that wants it rebuilt sends the proof and the view options.
- `InfId` numbers a derivation's inferences (postorder); `GoalId` a
  session's (3.10): two numberings, two types (F69).

### 3.10 `Interactive`: a partial derivation the checker completes [28]

D13 stands: the state is a derivation of the standard calculus over the
session's forest, open goals as leaves, and a finished state translates
into a term the checker validates in the session's mode. **Nothing enters
the state that the checker has not seen over the session's forest in the
session's mode** (F23): `close_with` refuses a proof of another forest
(`Error::ForeignProof`), a proof whose recorded conclusion is not the
goal as a multiset (`Error::GoalMismatch`), and checks it in
`self.mode()`. F23's two witnesses (a `[Top(4)]` of another forest, a
`Weaken` in a linear session) become tests of these errors.

```rust
pub struct GoalId(u32);                             // the session's numbering (F69)
#[non_exhaustive] #[derive(Clone, Debug, PartialEq, Eq)]    // serde: {"position", "rule", "left"?}
pub struct Step { pub position: usize, pub rule: Named, pub split: Split }
impl Step {
    pub fn new(position: usize, rule: impl Into<Named>) -> Self;
    #[must_use] pub fn left(self, positions: &[usize]) -> Self;   // ⊗, Mix, ⊸L: the context going left
    // [38] #[must_use] pub fn witness(self, w: Witness) -> Self  a term, or Witness::Open
}
#[non_exhaustive] pub enum Split { None, Left { positions: Vec<usize> } /* [36] At { position: usize }: an ordered goal's cut point */ }
#[non_exhaustive] pub struct Applicable { pub rule: Named, pub needs: Needs }
#[non_exhaustive] pub enum Needs { Nothing, Split /* [38] Witness */ }

impl Interactive {
    pub fn new(s: &Sequent, mode: Mode) -> Result<Self, Error>;               // the defaults
    pub fn within(s: &Sequent, mode: Mode, limits: &Limits) -> Result<Self, Error>;
    pub fn goals(&self) -> impl Iterator<Item = GoalId> + '_;
    pub fn goal(&self, id: GoalId) -> Option<&[Member]>;
    /// What `apply` would accept at the position, given the goal's context (R91, R93).
    pub fn rules(&self, goal: GoalId, position: usize) -> Result<Vec<Applicable>, StepError>;
    pub fn apply(&mut self, goal: GoalId, step: &Step) -> Result<Vec<GoalId>, StepError>;
    pub fn split_passes(&self, goal: GoalId, step: &Step) -> Result<bool, StepError>;
    pub fn undo(&mut self) -> Option<GoalId>;
    pub fn close(&mut self, goal: GoalId, options: &Options, view: &ViewOptions, limits: &Limits,
                 stop: impl FnMut(Progress) -> bool) -> Result<Closed, Error>;
    pub fn close_with(&mut self, goal: GoalId, proof: &Proof, view: &ViewOptions, limits: &Limits,
                      stop: impl FnMut(Progress) -> bool) -> Result<(), Error>;
    pub fn close_all(&mut self, options: &Options, view: &ViewOptions, limits: &Limits,
                     stop: impl FnMut(Progress) -> bool) -> Vec<(GoalId, Result<Closed, Error>)>;
    pub fn proof(&self, limits: &Limits, stop: impl FnMut(Progress) -> bool) -> Result<Proof, Error>;
    pub fn derivation(&self) -> Result<Derivation<'_>, Error>;          // the defaults
    pub fn derivation_within(&self, view: &ViewOptions, limits: &Limits,
                             stop: impl FnMut(Progress) -> bool) -> Result<Derivation<'_>, Error>;
    pub fn derivation_ids(&self) -> Vec<GoalId>;     // a drawing's InfId n is goal ids[n]
    pub fn occurrence(&self, m: Member) -> OccId;
    pub fn formula(&self, m: Member) -> impl Display + '_;
    // unchanged: forest, sequent, mode, reading (O(1)), is_complete, steps
}
/// What a `close` did: the search's outcome, and whether its proof was grafted.
#[non_exhaustive] pub struct Closed { pub outcome: Outcome, pub grafted: Result<(), Refusal> }
```

- **A session's drawing is bounded like a proof's** (walk-through 32):
  `derivation` is fallible and `derivation_within` takes the view and the
  limits, so a pasted large sequent cannot escape
  `limits.derivation_bytes`; `derivation_ids` is documented for the
  uncompacted view. **A proved goal is never lost**: a graft refused for
  its size leaves the goal open and keeps the checked proof in
  `Closed::outcome` for export (today the command calls `prove_goal` and
  `close_with` itself for that).

- `close_all` gives every goal its own result and keeps the grafts made
  (F68, R92); `limits.work` is a budget per goal, and `Progress` tells
  the stop which phase runs. A Mix that sends every formula to one side is
  `StepError::EmptyPremise` (F67, C3's answer A: nothing closes an empty
  goal).
- Reading a session back is linear (F21) and checks each history entry
  against the arena as that step left it, so `undo` cannot index past it
  (H17). The JSON form keeps its keys plus `version` (7.3).
- [32] adds, in a module `session` (features `interactive` and
  `serialize`), `GoalView`/`FormulaView` (a goal's formulas with their
  text under the `TextOptions` and sides under the `ViewOptions`, and their
  `Applicable` rules, O(goal) per view), `Request`, `Response` and `serve`
  (forms in 7.3). A worker keeps a live `Interactive` and the page holds
  the session's JSON as its recovery: a stateless worker would read and
  write the whole session per click, quadratic over a session
  (walk-through 32). `rules` with the refused rules and why
  (`explain`) is additive then. [34] adds `cut(goal, formula: &Sequent, split) ->
  Result<[GoalId; 2], Error>` (a cut is a method, not a `Step`: it appends
  a cut pair to the forest; the formula is a one-root sequent, merged by
  atom name; the forest's growth is bounded by the session's
  `occurrences`, and an intuitionistic cut is validated before the state
  changes). [38] `Step::witness`, bindings that
  `undo` undoes, `StepError::{Witness, Eigenvariable, Unresolved}`.

### 3.11 `ProofStructure`: vertices and the criterion [28]

```rust
/// A vertex: in MLL the occurrence itself (vertex i is occurrence i, no
/// table); in MELL (33) an instance, a collector, a door or a box node.
pub struct VertexId(u32);
/// The rules a structure is checked under, the one value 33, 35 and 36
/// extend in place of `mix: bool` (R79).
#[non_exhaustive] #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Criterion { pub mix: bool /* [36] order, and L against L*, flattened on the wire */ }
impl Criterion {
    pub const MLL: Self; #[must_use] pub const fn with_mix(self) -> Self;
    pub fn of(mode: Mode) -> Result<Self, NetError>;  // NetError::Mode: affine mode has no nets
}
impl ProofStructure {
    pub fn new(forest: Forest, criterion: Criterion) -> Result<Self, Error>;
    pub fn from_links(forest: Forest, criterion: Criterion, links: &[(VertexId, VertexId)]) -> Result<Self, Error>;
    pub fn from_proof(proof: &Proof, criterion: Criterion, limits: &Limits,
                      stop: impl FnMut(Progress) -> bool) -> Result<Self, Error>;
    pub fn criterion(&self) -> Criterion;
    pub fn vertex(&self, o: OccId) -> Option<VertexId>;   // None when o has several instances (33)
    pub fn occurrence(&self, v: VertexId) -> OccId;
    pub fn is_correct(&self, stop: impl FnMut(Progress) -> bool) -> Result<(), NetError>;   // linear in memory
    pub fn sequentialize(&self, limits: &Limits, stop: impl FnMut(Progress) -> bool) -> Result<Proof, Error>;
    // over VertexId now: partner, links, unlinked, link, unlink, same_component
    // [33] boxes, box_of, depth, jumps;  [34] cut links, the erased set
}
#[non_exhaustive]
pub enum NetError {     // every variant with named fields, each #[non_exhaustive] (P3)
    // the structure is malformed (kind malformed)
    NoVertex { vertex: u32, vertices: u32 }, NotLiteral { vertex: VertexId },
    NotDual { x: VertexId, y: VertexId }, LinkedTwice { vertex: VertexId },
    // the criterion fails (kind invalid)
    Unlinked { vertex: VertexId }, Empty, SwitchingCycle { cycle: Vec<VertexId> },
    Disconnected { parts: Vec<Vec<VertexId>> },
    // no structure exists here (kind unsupported)
    Fragment { fragment: Fragment }, Mode { mode: Mode },
    Refused { refusal: Refusal },
}
    // [33] LinkAcrossBoxes, JumpAcrossBoxes, MissingJump, …, and `within: Option<BoxId>` on the
    //      cycle and the parts; [34] CutNotDual; [35] DirectedCycle, NoDominator; [36] Crossing
```

- **The MLL hot path keeps its cost**: `VertexId(i) == OccId(i)`, no
  table; the net engine keeps its unchecked link and its union-find. The
  identity holds exactly when a structure has no vertex table; a
  structure with one (33) numbers its own vertices, an occurrence under a
  weakened `?` having none and one under a copied `?` several:
  `vertex(o)` is `Some` for exactly one instance and 33 adds
  `instances(o)` (walk-through 33). **The gate needs an instrument
  first** (walk-throughs 30, 33, 35): the target set runs the focused
  engine only, so area 3.1 commits, before the retype, a net-engine list
  (`bench/targets.sh`'s rows or a script beside it, one thread, pinned
  cores) of the `engines` and `period` problems with `links` and `tests`,
  which the retype must keep (R206, step 30 promotes it).
- `is_correct` and [35] `is_essential` are linear in memory (the
  essential criterion's dominators by an iterative algorithm), so they
  take a stop and no limits; the n² closure is the essential *search*'s
  (10.7). `sequentialize` and `from_proof` take `&Limits`.
- **`from_proof` is exhaustive and bounded** (F49, R254, R25): a goal proof
  is refused (`Error::GoalProof`); `Ax` a link,
  `Tensor`/`Par`/`Mix` nothing, every other arm refused by name until 33
  (exponentials, `Weaken`) and 34 (`Cut`) read them; its vertices charged
  to `limits.memory_bytes` and the stop polled per node from the first
  signature, since unfolding a MELL term's shared subproofs per path is
  exponential.
- One criterion value; boxes are a property of the structure, not of the
  criterion (R145). The essential criterion is no field of it (walk-through
  35): it is a question of the structure, `is_essential`, and the search's
  `Linker<C>` parameter; a field would duplicate it, admit
  `{mix, essential}` and raise the wire level. Its fields serialize flattened into the net, so the
  pinned `"mix": false` stays (F10, R79). `ProofStructure` has no
  `PartialEq` (MELL nets are equal up to isomorphism modulo jumps).

### 3.12 Refutations and disproofs [28]

A refutation means something only with its sequent, goal and mode, and
some are certificates a checker sharing no code with the engines
re-verifies (R124). So `Verdict::Unprovable` carries a `Disproof` that
holds them, as a `Proof` holds its forest; `Verdict` is closed (D9), so
this is decided now or never without a break.

```rust
#[non_exhaustive]
pub enum Refutation {
    Exhausted,                       // an exhaustive search, no certificate
    Unbalanced(Unbalanced),
    Equation(Equation),
    StateEquation(StateEquation),
    // [31] Classical(Assignment) (C2); later countermodels (R72)
}
#[non_exhaustive] pub struct Unbalanced { pub atom: Atom, pub least: i32, pub most: i32 }   // the name from the sequent (F80)
#[non_exhaustive] pub struct Equation { pub formulas: u64, pub needed: u64, pub tensors: u64, pub pars: u64,
                                        pub ones: u64, pub bottoms: u64, pub mix: bool }
/// Farkas weights of a Horn program's Petri net, every place included (R70).
#[non_exhaustive] pub struct StateEquation { pub atoms: Vec<(Atom, i64)>, pub clauses: Vec<(OccId, i64)>,
                                              pub dropped: Vec<OccId> }
pub struct Disproof { sequent: Sequent, goal: Option<Box<[Member]>>, mode: Mode, refutation: Refutation }
impl Disproof {
    pub fn new(sequent: Sequent, mode: Mode, refutation: Refutation) -> Self;
    pub fn sequent(&self) -> &Sequent;  pub fn goal(&self) -> Option<&[Member]>;
    pub fn mode(&self) -> Mode;  pub fn refutation(&self) -> &Refutation;
    // [31] pub fn check(&self, limits: &Limits, stop) -> Result<(), Error>   (R124)
}
```

- `prove_goal` builds the `Disproof`: one clone of the sequent's arena per
  unprovable outcome, against a search.
- **A refutation's kind is an open enumeration** (walk-through 31): a
  reader that does not know a kind reads it as `Exhausted`, "no
  certificate this reader can check", which keeps the verdict and loses
  only the certificate; so a new kind (31's `classical`) raises no wire
  level and a propositional outcome stays level 1. [31] adds
  `Refutation::applies(&Sequent, goal, Mode)`, the one function the
  checker and the writer call, and the refutation's faults as an error of
  their own (`invalid_refutation`). Its `Display` writes the
  refutation in words, atoms by name.
- **`StateEquation`'s payload changes before 0.1.0** (R70) so that step
  31's checker verifies the closure, the dropped transitions and `y·C ≤ 0`,
  `y·(M − M₀) > 0` itself; `once` goes.
- **The conditions travel with the check, not the value** (R70, R71): step
  31's `Disproof::check` recomputes from the sequent and mode whether the
  kind applies, then the certificate, by functions written again in
  `refutation.rs`, never by calling the engines' counts. No refutation
  rests on a bound (R71): `Unprovable` comes only from a level that met
  none.

### 3.13 The ordinary layer [28]

- `ordinary::Sequent::new` and `Formulas::add` are fallible and refuse an
  operand that is not an earlier node and an atom outside the names (F13,
  R249); [38] also a free variable and a symbol at two arities.
  `ordinary::Node` stays closed, asserted at 12 bytes; its `Atom(u32)`
  indexes the same interned atom table as `Sequent`'s (3.2), so [38] adds
  `Forall(NodeId)`, `Exists(NodeId)` only, `translate` copies atom ids
  and "equal formula ⇔ equal id" holds (walk-through 38).
  `Rule`, `Inference` (private fields, [38] `datum()`), `Options`,
  `Logic`, `Translation` are `#[non_exhaustive]`.
- **One first-order term type for both layers** [38] (D-4,
  impact-fo-ordinary item 1): `Formulas` holds the same tables as
  `Sequent` (3.14), so `translate` copies term ids and the LK/LJ checker
  keeps "equal formula ⇔ equal id".
- `Translation::target() -> Target { fragment, mode }`; the docs stop
  implying the classical path terminates on first-order input
  (fo-embeddings §1.5). `Image::read_back(&self, &Proof, &Limits, stop)`
  reads the proof, where witnesses live (D-12), within
  `limits.derivation_bytes`, so step 25's unfoldings of gigabytes are
  refused before they are made; `ordinary::Derivation::check(&Limits,
  stop)` polls per inference (F12, R24).
- **One call decides** (R113): `ordinary::decide(&sequent,
  &ordinary::Options, &search::Options, &Limits, stop) -> Result<ordinary::
  Outcome, Error>`, an `Outcome { verdict: Valid(Box<Derivation>) |
  NotValid | Unknown(Reason), logic, translation, linear: search::Outcome }`;
  a second procedure (G4ip with countermodels, R72) plugs in behind it.
  TPTP's options go to `read_tptp(text, &TptpOptions, &Limits)`, not to
  `ordinary::Options` (D-12).
- The ordinary forms are born versioned (R11, F53): 7.3. `ordinary::
  Outcome` is `#[non_exhaustive]`; [31] adds the countermodel of a "not
  valid" (an assignment over the ordinary atoms, which `Image` maps back),
  and the certificate R154 asks for is the one over the ordinary formula
  in `Prop`, not one over the image (walk-through 31).

### 3.14 The first-order tables, reserved [38]

The shapes step 38 adds, fixed here so that nothing before it takes their
place. All are private fields or new modules: adding them breaks nothing.

```rust
// sequents::fo
pub struct SymbolId(u32);   // a predicate or function symbol: (name, arity, kind); constants are arity 0
pub struct FoTermId(u32);   // continues past the sequent's own terms in a proof or a search (P2)
pub struct ArgsId(u32);     // an argument list in a hash-consed CSR table; ArgsId::EMPTY
pub struct Eigen(u32);      // an eigenvariable of a proof
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum FoTerm {           // closed; 12 bytes
    Bound(u32),                 // de Bruijn index: the innermost binder is 0 (locally nameless)
    App(SymbolId, ArgsId),      // f(t…); a constant has EMPTY arguments
    Eigen(Eigen),               // only in a proof's or a search's extension
    Meta(u32),                  // a metavariable: only in a search's extension, or open in a proof (fo-linear Q3)
}
// in Sequent, private: symbols: Vec<(String, u32)>; per atom (SymbolId, ArgsId), interned;
//   fo: Vec<FoTerm> (topological, hash-consed, ground or bound only); args (CSR, hash-consed);
//   binders: Vec<(TermId, String)> (a bound variable's name, for printing only)

// occurrences::instances
pub struct FrameId(u32);    // a hash-consed cons cell (parent: Option<FrameId>, term: FoTermId)
pub struct Instances {      // what members past forest.len() stand for
    members: Vec<(OccId, FrameId)>,
    frames: Vec<(Option<FrameId>, FoTermId)>,    // hash-consed; quadratic flat lists avoided (R43)
    terms: Vec<FoTerm>,     // the extension of the sequent's fo arena (witnesses, eigenvariables)
    args: …, symbols: …,    // the extensions a witness or a fresh constant needs
    names: …,               // display names of eigenvariables and metavariables
}
// Instances::atom_of(member) -> (SymbolId, ArgsId): the substituted atom, interned, mapped
// back to the sequent's Atom when it equals one
```

- **Frames are cons cells**, not flat lists (walk-through 38): a tower of
  `D` binders proved level by level would otherwise hold `D²/2` entries.
  A reader refuses an instance table larger than the proof's nodes allow
  and entries no node names.
- **One namespace of symbols**: a name used as a predicate and as a
  function is refused, so the wire's `symbols` need no kind.
- **The tables are empty exactly when every atom is nullary and there is
  no binder**, an invariant `optimize` and `Sequent::check` keep, so a
  sequent equal to a propositional one writes level 1.

- **Locally nameless** (fo-linear §5.1): bound variables are indices, so
  α-equal closed formulas are one `TermId` and `optimize` hash-conses
  `∀x.p(x)` and `∀y.p(y)` with no renaming; the name table is for printing
  only, the first name winning a merge, a clash primed. Closed input: an
  identifier in term position that no binder binds is a constant
  (fo-linear §7 Q1, the author's to confirm at 38).
- **Instantiation never rewrites a formula**: the instance of an
  occurrence is its atom read through its frame. The sequent's tables are
  read-only during a search; a search's terms extend them by offset and
  are truncated by its trail (10.10).
- **Bounds**: `Sequent::fo_size()`, the unfolded size of the first-order
  terms, saturating like `occurrences()`, and `Limits::terms` (a field
  added at 38, `Limits` being `#[non_exhaustive]`) refusing a larger input
  at every reader (R250); no term operation walks a shared term once per
  path, and the checker compares instances by hash-consed id, never by
  unfolding.

## 4. Errors [28]

### 4.1 One family

```rust
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error { /* 4.4; named fields, or one wrapped error type (P3, F31) */ }
impl Error {
    /// What kind of failure this is; a wrapped error answers its own.
    pub fn kind(&self) -> ErrorKind;
    /// The stable reason a program branches on, never the English text.
    pub fn code(&self) -> &'static str;
    /// The settings key that lifts or changes the bound that refused the call.
    pub fn setting(&self) -> Option<&'static str>;
    /// The message with formulas for ids, given the owner of the ids (a
    /// `Forest`, `Proof`, `Derivation`, `Interactive`, `ProofStructure`):
    /// members and vertices past the forest need their owner's table.
    pub fn describe<'a>(&'a self, owner: &'a impl Owner) -> Described<'a>;   // Owner: sealed
    pub const CODES: &'static [&'static str];
}
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ErrorKind {
    Malformed,    // the input is not what it says: no sequent, indices that do not fit, a newer version
    Invalid,      // a verdict: the proof, net, step or certificate is not what it claims
    Unsupported,  // no verdict: outside what this build, engine, kernel or reader does
    Limit,        // no verdict: a bound (set or default) refused the call; lifting it may answer
    Stopped,      // no verdict: the caller's stop ended the call
    Failed,       // the environment failed: threads, the writer, the renderer
    Defect,       // a check of the library's own result failed: report it
}
impl ErrorKind { pub const fn is_refusal(self) -> bool; /* Unsupported | Limit | Stopped */ }
```

- **A refusal never reads as a fault** (S4, S18). Only `Invalid` says the
  claim is wrong; `Malformed` says the input is no well-formed value
  (the command's exit status 2, where `Invalid` from `check` is 1). Every
  specific error type that can be refused carries the refusal as a
  variant of its own (`CheckError::Refused`, `NetError::Refused`), whose
  payload is a `limits::Refusal` (5.1), so `kind()` is one `match`
  without a wildcard. A search's refusal is a verdict, `Unknown(Reason)`;
  a proof found but not checkable within the bound is
  `Unknown(Reason::Unchecked { limit_bytes })` (R138, F139).
  `Error::Rejected` (an engine's proof the checker rejects) and
  `ReadBack` are `Defect`, never a verdict on the sequent.
- **Specific types stay where a caller matches their data** (C-GOOD-ERR):
  `ParseError` (a span), `CheckError` (the node, the premises, the fault),
  `NetError` (a cycle's vertices), `ShapeError` (the subformula),
  `StepError` (why a step was refused), `rocq::Unsupported` (what a kernel
  lacks). Each converts into `Error` without loss, and the front page
  names them and says so (F8). Their positional variants become named
  (P3): `Fault::{Succedents { count }, Kind { member }, NotUnderQuest {
  member }, Conclusion { derived }}` (3.8), [34] `NotACut { member }`;
  `ShapeError::{SeveralGoals { first, second }, Formula { occurrence },
  Succedents { count }}` and the new `Undetermined { first, second }`
  (3.6); `StepError::{NoGoal { goal }, Succedents {
  count }}`; `NotTaken::Mode { mode }` (8.6); `Split::Left { positions }`
  (3.10), [36] `At { position }`; [38] `Witness::Term { term }`. The types that held only refusals or a
  failed writer fold into `Error`: `ViewError`, `WriteError`,
  `RenderError`, `svg::TooLarge` go.
- `Send + Sync + 'static`, `std::error::Error` with `source()` to the
  inner type, a lowercase `Display` without a full stop, counts that agree
  (F33). Large payloads are boxed and `size_of::<Error>()` asserted at most
  64 bytes (112 today).
- **One `describe`** (F48, F50; walk-throughs 32, 33, 38): `Described<'a>`
  borrows the error and its owner, prints formulas for ids, has
  `abbreviated(limit)` and is `Serialize` (the wire form with formulas);
  an error raised inside a call that builds its own forest (`prove_within`'s
  `ShapeError`) carries the subformula's text, since the caller holds no
  forest to describe it with; each type
  writes itself through one sealed writer `write(f, Option<&Forest>,
  limit)`, so `Display` and `describe` cannot drift.
- **No public call panics on input of the right type** (R130, S5): the
  readers, `apply`, `undo` (H17), the exports (style values validated,
  F4), the families (F6); the `expect` sites kept are invariants of the
  crate's own values, listed in `core.md`. `linlog-web` installs a panic
  hook that posts its own `{"code": "panic"}` (not one of `CODES`) and
  respawns the worker.
- **The messages the behaviour lock pins keep their words** unless a
  finding changes them; such a change is in the lock commit that says
  why.
- **The command's exit statuses by kind** (walk-through 30): a verdict
  keeps 0, 1 or 3; an error is 2, except `Invalid` from `check` (1) and
  a search's refusals, which are verdicts (`Unknown`, 3). The command area
  writes the table into `cli.md` and the help, one place.

### 4.2 The refusal

```rust
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refusal {
    Stopped { phase: Phase },
    Memory { phase: Phase, limit_bytes: u64, needed_bytes: Option<u64> },
    Occurrences { occurrences: u64, limit: u64 },
    Output { what: &'static str, estimate_bytes: u64, limit_bytes: u64, least_bytes: Option<u64> },
    Work { limit: u64 },
    Pixels { pixels: u64, limit: u64 },
    Index { what: Space, count: u64, most: u64 },   // a representation's cap: 2³² − 1 nodes, Forest::MOST
}
```

`Error::Refused(Refusal)` is the one variant every refused call outside a
search returns, its kind `Stopped` for `Stopped` and `Limit` otherwise,
its `setting()` the key of 6.6 (`limits.memory_bytes`,
`limits.occurrences`, `limits.derivation_bytes`, `limits.work`,
`styles.png.pixels`); `Index` names no setting, since no flag lifts a
representation's width.

### 4.3 The wire form (written only, nested)

```json
{"code": "too_many_occurrences", "kind": "limit",
 "message": "the sequent unfolds to at least 67108864 subformula occurrences, more than the limit of 1000000",
 "setting": "limits.occurrences", "details": {"occurrences": 67108864, "limit": 1000000}}
{"code": "parse", "kind": "malformed", "message": "unexpected \"⊢\" at line 1, character 4",
 "details": {"span": {"start": 3, "end": 6}, "span_utf16": {"start": 3, "end": 4},
             "line": 1, "column": 4, "found": "⊢", "expected": ["a formula"]}}
{"code": "invalid_proof", "kind": "invalid", "message": "node 2 (⊗ on 1 from 0, 1) …",
 "details": {"node": 2, "rule": {"⊗": [1, 0, 1]}, "fault": {"kind": "missing", "premise": 0, "member": 2},
             "premises": [{"theta": [], "gamma": [0, 3], "any": false}, {"theta": [], "gamma": [3, 4], "any": false}]}}
```

`code` is what a client branches on (AIP-193), `kind` what it shows (a
red "invalid" only for `invalid`), `setting` the key the command maps to
its flag and a web client highlights, `details` the variant's named fields
with units in their names. A `ParseError` gives its span in bytes, in
UTF-16 units (what a JavaScript editor indexes) and as line and column in
characters, computed when it is made (R129). `describe(&owner)`
serializes the messages with formulas. The codes are a
table in the rustdoc of `Error`, stable from 0.1.0; a client accepts an
unknown code and falls back on `kind`.

### 4.4 The variants

| today | after | code | kind |
|---|---|---|---|
| `SequentParsing(Vec<ParseError>)` | `Parse(Box<ParseError>)` (the parser yields one) | `parse` | malformed |
| `Lltp`, `Mist`, `Tptp` (`String`) | `Lltp`, `Spec`, `Tptp { message }` | `lltp`, `spec`, `tptp` | malformed |
| — (HD1) | `SeveralConjectures { second }` | `several_conjectures` | malformed |
| — (serde) | `Json { form, message }` | `json` | malformed |
| — (F26) | `Version { form, found, supported }` | `unsupported_version` | unsupported |
| — (HD5, H19) | `AtomName { name }` | `atom_name` | malformed |
| the six index errors | `IndexOutOfBounds { space: Space, index, len }`, `NotTopological { space, index, parent }` (`Space::{Atom, Term, Node, Occurrence, Member, Vertex}`) | `index_out_of_bounds`, `not_topological` | malformed |
| `InconsistentState`, `OpenGoals` | `InconsistentSession { reason }`, `OpenGoals { count }` | `inconsistent_session`, `open_goals` | malformed, invalid |
| — (F6, F4, F38) | `FamilySize { family, size }`, `InvalidOption { key, message }` | `family_size`, `invalid_option` | malformed |
| `UnknownRule`; every name table outside the library | `UnknownName { what, name, known }` | `unknown_name` | malformed |
| `InvalidProof(CheckError)`, `Unchecked(CheckError)` | `Check(CheckError)`: its kind is the inner one | `invalid_proof`, or the refusal's | invalid, limit, stopped |
| — (F89) | `GoalProof` (a goal proof where the sequent's is needed: `linlog check`, `from_proof`, the Rocq writer) | `goal_proof` | unsupported |
| — (F23) | `ForeignProof`, `GoalMismatch` | `foreign_proof`, `goal_mismatch` | invalid |
| `InvalidNet(NetError)` | `Net(Box<NetError>)` | `invalid_net`, or the refusal's | invalid, limit, stopped |
| `Refused(proofs::Refusal)` | `Step(StepError)` | `step` | invalid |
| `NotIntuitionistic(ShapeError)`, `IntuitionisticMix`, `GoalOutputs`, `Succedents`, `FragmentMismatch`, `Translation` | the same, named fields | `not_intuitionistic`, `intuitionistic_mix`, `goal_outputs`, `succedents`, `fragment_mismatch`, `translation` | unsupported |
| `NetFragment`, `NetMode`, `NetGoal`, `EngineMode`, `NotAdditive`, `NotHorn` | `EngineRefused { engine, because: NotTaken }` (8.6); the messages keep the lock's words | `engine_refused` | unsupported |
| `NoEngine` (never built) | built by the dispatch | `no_engine` | unsupported |
| `TooManyOccurrences`, `TooManyNodes`, `ViewError::{TooLarge, Memory, TooMany, Stopped}`, `Problem::Memory` outside a search, `RenderError::{TooLarge, Memory}`, `svg::TooLarge`, `WriteError::Stopped` | `Refused(Refusal)` (4.2) | `too_many_occurrences`, `memory_limit`, `output_too_large`, `too_many_pixels`, `work_limit`, `index_limit`, `stopped` | limit, stopped |
| `WriteError::Unsupported` | `Unsupported(rocq::Unsupported)` | `no_certificate` | unsupported |
| — (32) | `FeatureOff { feature }` | `feature_off` | unsupported |
| `ThreadPool`, `WriteError::Failed`, `RenderError::{Failed, Svg, NoDate}` | `ThreadPool { threads, message }`, `WriteFailed`, `RenderFailed { message }`, `NotSvg { message }`, `NoDate` | `thread_pool`, `write_failed`, `render_failed`, `not_svg`, `no_date` | failed, malformed |
| `Rejected`, `ReadBack` | the same, named fields | `rejected`, `read_back` | defect |

Codes reserved for later, each with its variant when its step lands:
`cut_formula`, `not_dual_pair`, `elimination_limit` (34, R131); the box
kinds of `NetError` (33, R132); `ordered_fragment`, `empty_antecedent`,
`ordered_mode` (36, R134); `arity`, `free_variable`, `unbound_variable`,
`term_size`, `first_order_goal` (38, R136). The harness classifies by
`kind()`: `unsupported` is its "refused" (R135), `limit` and `stopped`
"unknown", `defect` and `failed` "error".

## 5. Bounds and stops [28]

### 5.1 One bounds value

```rust
/// The resources a call may use, which the library enforces.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]      // serde(default, deny_unknown_fields)
pub struct Limits {
    /// What a call holds at once beyond its input: a search's account, a check's pass,
    /// a derivation's making, a render's estimate. None: no bound.
    pub memory_bytes: Option<u64>,
    /// The most subformula occurrences a sequent may unfold to, on every reader.
    pub occurrences: Option<u64>,
    /// The estimated size of a derivation, a net's or a sequent's drawing, made only below it.
    pub derivation_bytes: Option<u64>,
    /// The most units of work a search may do (5.3). None: no bound.
    pub work: Option<u64>,
    /// The deepest recursion on any one stack (5.4).
    pub recursion_depth: u32,
    // [38] terms: Option<u64>, the unfolded size of first-order terms (R250)
}
impl Limits {
    pub const DEFAULT_MEMORY_BYTES: u64 = 1 << 30;
    pub const DEFAULT_OCCURRENCES: u64 = 50_000_000;     // above the largest LLTP problem's 27.8 million
    pub const DEFAULT_DERIVATION_BYTES: u64 = 64 << 20;
    pub const DEFAULT_RECURSION_DEPTH: u32 = 2048;
    pub const UNBOUNDED: Self;                           // every Option None
    // [32] BROWSER: a tab's, measured in a worker
    pub const fn stack_bytes(&self) -> usize;            // what a thread needs at recursion_depth
    pub const fn recursion_depth_for_stack(bytes: usize) -> u32;   // its inverse (R45)
    // a #[must_use] with_* per field
}
```

- **One value replaces three** (F62, F63): `search::Options::{memory_limit,
  occurrence_limit, recursion_limit}`, `ViewOptions::{limit, memory}`,
  `check_within`'s `memory`, `png`/`pdf::Options::memory`, and the three
  copies of `DEFAULT_MEMORY_LIMIT`. `None` means no bound in every
  optional field (F79); `recursion_depth` is always a number, since every
  thread's stack is finite and sized from it (5.4); the representation's caps (`Forest::MOST`, 2³² − 1 nodes) remain
  and answer `Refusal::Index`. Every long call takes `&Limits`; the
  command builds it once from `--memory-limit`, `--occurrence-limit`,
  `--derivation-limit`, `--work-limit` (new) and `--recursion-limit`; the
  web client holds it as JSON.
- **`BROWSER`** (R142) [32], additive: a preset with no caller before the
  web client ships nothing until step 32 measures it (walk-through 30), so
  0.1.0 promises no browser numbers. Its starting point: `memory_bytes` 256 MiB (a quarter of
  what a tab may grow to, leaving room for the forest, the view and the
  SVG), `occurrences` 1 000 000 (a 25 MB forest, uncounted above),
  `derivation_bytes` 4 MiB (an SVG peaks near 6.5 times the estimate),
  `work` none (the client's stop keeps the deadline), `recursion_depth`
  `recursion_depth_for_stack(1 << 20)`, rustc's default wasm stack. Step
  32 measures each and records it (D16), with `Limits::within_stack(bytes)`
  clamping a stored depth to what a fixed stack holds, since a raised
  depth past it traps the instance (walk-through 32).
- **`stack_bytes`** = `recursion_depth × PER_LEVEL + RESERVE` (2 304
  bytes a level optimized, measured on x86-64). The 8 MiB floor of
  `Options::stack_size` was for the derivation's builder and renderer,
  which no longer recurse (`core-derivations.md`); the command may keep a
  floor of its own. The wasm32 figure is step 32's.
- **What `memory_bytes` counts** is stated on the field (R29): the
  search's growing structures, the checker's pass, a view's record and
  derivation, a render's estimate, [35] the closure matrix, [37] the
  database; not the forest (bounded by `occurrences`), the sequent, the
  proof returned, the net engine's linear structure, thread stacks, the
  allocator. A call's peak is `memory_bytes` for its largest phase plus
  the forest (`Forest::BYTES_PER_OCCURRENCE` an occurrence) and the proof.
- **A table above linear in the input is reserved fallibly** (walk-through
  35): with no bound (`memory_bytes: None`) the account cannot refuse, so
  every structure whose size is above linear in the input ([35] the
  closure matrix and the balance table, [37] the database) is allocated
  with `try_reserve` and answers `Reason::IndexLimit` when it cannot be
  had or its size overflows, bound or not: no public call aborts on input
  of the right type (R130).

### 5.2 One stop, with progress

```rust
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Progress {
    pub work: u64,         // units of work since the previous poll, the unit of Limits::work
    pub done: u64,         // units since the call began; on one thread a search's ends as Statistics::work
    pub held_bytes: u64,   // what the call's account holds now
    pub phase: Phase,
    pub item: u32,         // the item of a call that runs several: a goal of close_all, a problem of a batch
}
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Phase { #[default] Read, Search, Refute, Check, View, Write, Render, Net, ReadBack }
//   [34] Eliminate  [37] Saturate
```

Every long call takes `stop: impl FnMut(Progress) -> bool`; true ends a
search with `Unknown(Reason::Stopped)` and any other call with
`Refusal::Stopped { phase }`.

- **A closure over the progress, no trait** (decided against a `Stop`
  trait with a blanket impl for `FnMut() -> bool`): one form; a closure
  that ignores the progress is `|_| false`, and its argument type is
  inferred from the bound. A probe of the judges showed the trait's
  wrapper for progress closures fails to infer (`WithProgress(|p| …)` is
  E0282 without an annotation). The engines pass `&mut dyn FnMut(Progress)
  -> bool` inward; the crate-private `search::Stop` (closure, turn,
  slice, flags), renamed `Poll`, stays the engines' mechanism and hands
  the caller the public progress. The race's stop is `Fn(Progress) -> bool
  + Sync`, since two threads poll it.
- **The unit of work** is a step of the call's main loop, the same on
  every build, documented per engine on its `Engine` variant with its cost
  (a front end reads its clock every so many units, so a unit whose cost
  grows with the input is charged by that size: walk-through 32) (R30): a
  stable sequent plus its width in 128-occurrence words, a split
  candidate or a forced split (focused engines); a literal chosen or a
  failed exact test (net); a pair (additive); a marking expanded or a
  pivot of the state equation (Horn); a node (checker, size pass); an
  inference (views, writers, read-back, ordinary checker); a link or a
  step (net calls). It is counted apart from the work that slices the
  default bias's two searches, so no counter of a decided run moves
  (D17). On one thread the progress summed over a search equals
  `Statistics::work` (R243's test).
- **Polled at a bounded interval** (R30): the engines where
  `core-search.md` lists; the checker and the size pass every 4 096 nodes
  (new, R23); views and writers per inference; the net criterion per
  edge of its witness isolation; set-up passes every 65 536 occurrences.
  With `work` in hand a condition never counts polls (the front page's
  example that does, F136, becomes a `Limits::work` example).
- **The web client's deadline** (32): the bindings add up `work`, read
  `performance.now()` once per 2¹⁶ units, post the progress on the same
  schedule and stop past `settings.clock.time_limit_ms`.
- **AIP-151 as a pattern**: progress (the stop), an end (`Outcome` or
  `Error`), a cancellation (the stop, `Cancel`, a terminated worker) that
  ends in a defined state, never a partial result read as whole.
- The change of every poll site to the progress stop touches every
  engine's hot poll: it is a commit of its own, measured on the target
  set's counters and the journeys like the spike (section 11), and it is
  the search area's (3.2). **Until it lands, a shim**: the library area
  (3.1) gives every public call its `stop: impl FnMut(Progress) -> bool`,
  and the front door hands the engines, whose polls still call a
  `FnMut() -> bool` (`Stop::Closure`), the crate-private
  `search::without_progress(&mut stop)`, which calls the caller's stop
  with `Progress { phase: Phase::Search, ..Progress::default() }`: no
  work, no bytes. The polls the library area writes itself (the checker,
  the size pass, the views, the writers, the net calls, the set-up
  passes) pass their progress from the first commit. The measured commit
  removes the shim; `Statistics::work` and R243's test that the progress
  summed on one thread equals it come after it (7.5's commit (4)).

### 5.3 Work budget and determinism

`Limits::work` bounds a search (`prove*`, `race`, each goal of
`close_all`, [31] the refuters): past it, `Unknown(Reason::WorkLimit {
limit })` with `Statistics::work` saying what was done, so a client sizes
the next budget (R21). Views and checks after a search are bounded by
memory and size, and refuse past `work` too.

```rust
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Schedule { #[default] Auto, Turns }   // Auto: the default bias's pair on two threads where
                                               // `parallel` starts one, else turns; Turns: always turns
```

With `jobs: 1` and `schedule: Turns` the verdict, `Reason`, `Statistics`
and the sequence of polls are a function of the input and the options on
every build, wasm included (R47, R151); `--deterministic` sets both.
`Decide::decide` stays the one entry; a search that suspends and resumes
would be an additive function returning a value to resume, not a change
of `prove_goal` (walk-through 32), and a web worker stops by terminating.

### 5.4 The memory account, the race and the pool's stack

`search::memory::Account` stays crate-private; what is public is the
bound. One account per call, charged by capacity where anything grows
with the input, refused at the bound with the call's own refusal:

| call | what charges it | refusal |
|---|---|---|
| a search (`prove_goal`) | memo, arena, pools, counts, classes, additive memo, Horn markings and simplex; [35] the essential closure matrix, [37] the inverse database and index, [38] bindings, trail, search terms, instance tables | `Reason::MemoryLimit` |
| `search::race` | **one account both searches draw from** (F104, R18): a search that cannot get memory empties its own memo first, as today; the default bias's pair inside each keeps its halves (`Account::share`), on which the pinned counters depend | `Reason::MemoryLimit` |
| a batch | each worker a search's bound, as many workers as the batch's bound holds (`core-batch.md`) | per problem |
| the check, the size pass, a derivation | the pass's tables and states, the derivation's estimate | `CheckError::Refused`, `Refusal::Memory` |
| `from_proof`, `sequentialize`, `Image::read_back`, `ordinary::Derivation::check` | the structure built, the unfolded derivation | `Refusal::Memory` |
| readers | not an account: `occurrences` (and at 38 `terms`) refuse before anything of that size is built | `Refusal::Occurrences` |

**The race** (R85, F103, F168) lives once, in the library:

```rust
/// Decides the goal on at most `threads` threads, the calling one counted.
/// From three: one thread first and, once `add_pool` says so, a pool of
/// `threads − 1` beside it; the first to decide answers and stops the
/// other; one account for both. Two: a pool of two from the start. One,
/// or an engine that searches on one thread: the calling thread alone.
/// Below three threads `add_pool` is never asked.
#[cfg(feature = "parallel")]
pub fn race(goal: Goal<'_>, mode: Mode, options: &Options, limits: &Limits, threads: usize,
            add_pool: impl FnMut(Progress) -> bool, stop: impl Fn(Progress) -> bool + Sync)
    -> Result<Outcome, Error>;
```

`add_pool` is asked at the single search's polls with its progress, so
the caller decides with its own clock (`clock.pool_after_ms`) and the
library reads none. `threads` counts every thread, so `--jobs 2`
runs two, not three (F168). A race needs a pool of at least two beside
the single search, since a pool of one is the single search again, so
below three threads there is none: two threads search on a pool of two
from the start, which runs the default bias's pair side by side (5.3),
and one thread searches alone, the pair in turns. In a race the
statistics add both searches' counters (`Statistics::add`, `copies` and `memo_entries` by the
maximum), as the command's race does today (walk-through 29); the
comparison compares the default column's verdicts and times, never its
counters. The command and the
harness call it; neither keeps a copy.

**The pool's stack** (H18). A thread that waits at a scope runs stolen
jobs on top of its own frames, so the recursion a thread's stack holds is
not a task's depth. The bound holds when the depth is the thread's: the
runtime keeps one depth per worker (indexed by the pool's thread index, a
runtime field, no thread-local), and a task starts from the depth of the
thread that runs it, so a stolen job counts on top of the thief's.
`Reason::RecursionLimit` then comes before an overflow at any
`recursion_depth`, and every worker is sized by `stack_bytes()`; a
queued task polls the stop before it starts, so a pool returns promptly
(R19). The search area writes it with a test at a raised limit on two
threads.

### 5.5 Reading within limits

`Deserialize` takes no options, so a reader that must hold a bound is a
`DeserializeSeed`, format-agnostic (no `serde_json` in the library):

```rust
/// Reads a form within limits: forests within `occurrences`, tables within
/// `memory_bytes`, atom names checked, the version checked first.
pub struct Within<'a, T> { limits: &'a Limits, … }
impl<'a, T> Within<'a, T> { pub fn new(limits: &'a Limits) -> Self; }
impl<'de, T: sealed::Readable> DeserializeSeed<'de> for Within<'_, T> { type Value = T; }
```

For `Sequent`, `Proof`, `Disproof`, `ProofStructure`, `Interactive`,
`ordinary::Sequent` and the options values (R27, F27, H19, HD5); the
plain `Deserialize` is `Within` at `Limits::default()`. Reading checks the
version (7.1), deserializes the private proxy (linear in the text),
counts the occurrences before any forest exists, then builds and
validates. Text readers take `&Limits` too: `Sequent::parse_within`,
`lltp::read`, `mist::read`, `ordinary::read_tptp`, `Interactive::within`.
All are linear (a session's replay after F21), so none needs a stop.
`check` and `interact --state` read through `Within`, under
`--occurrence-limit`.

### 5.6 Every long public call (R22)

| call | limits | stop | without them |
|---|---|---|---|
| `Sequent::parse_within`, `FromStr`, `lltp::read`, `mist::read`, the ordinary parsers, `read_tptp` | `occurrences` (before allocating) | none | linear in the text |
| `Within` and `Deserialize` of every form | `occurrences`; nodes under 2³² | none | linear (a session's replay too, F21) |
| `Forest::within`, `new`; `Family::instance` | `occurrences` | none | linear (0.43 s at 27.8 million) |
| `engine_for` | none | none | passes over the goal |
| `prove`, `prove_within`, `prove_goal`, `race` | `memory_bytes`, `work`, `recursion_depth` (+ `copies`, `memo_limit`) | every engine; set-up every 65 536 occurrences | unbounded |
| `batch::{prove, run, run_local}` | per problem; the batch's total | each search; `Cancel` | unbounded |
| `Proof::{check, check_within}`, `derivation_size` | `memory_bytes`, `work` | every 4 096 nodes (new) | quadratic time in flat memory |
| `Proof::{derivation, derivation_within}` | `derivation_bytes`, `memory_bytes` | per inference | exponential in sharing |
| `Derivation::write_text`, `export::*::write` | (a bounded derivation) | per inference | linear |
| `{latex, typst, svg}::sequent` | `derivation_bytes` (estimated first, F5) | none | about 310 bytes an occurrence |
| `svg::net` | `derivation_bytes` | per literal | linear |
| `png`/`pdf::from_svg` | `memory_bytes` (estimated before parsing) | none: renderers cannot stop | at most about 8 s at 1 GiB |
| `ProofStructure::{from_links, link, unlink}` | none | none | linear |
| `ProofStructure::from_proof` | `memory_bytes` (33's instances) | per node | linear in MLL |
| `ProofStructure::{is_correct, sequentialize}` | `memory_bytes` | per edge, per step | quadratic |
| `Interactive::{new, within}` | `occurrences` | none | linear |
| `Interactive::{goals, rules, apply, split_passes, undo, derivation, derivation_ids}` | none | none | linear in the goal or the arena (F66) |
| `Interactive::{close, close_with, close_all, proof}` | all | yes | a search, a check and a view |
| `ordinary::translate` | `occurrences` | none | linear |
| `Image::read_back`, `ordinary::Derivation::check`, `ordinary::decide` | `memory_bytes`, `derivation_bytes` (+ the search's) | per inference | exponential unfolded (R24) |
| [31] refuters, `Disproof::check`; [34] elimination; [37] saturation | `memory_bytes`, `work` | per step | unbounded |

A test per row fires the stop at its first poll, or reaches the bound
(R22, R30); the table lives in the crate docs, and `core-search.md`'s
"Not polled" list shrinks to `Forest::within` and single linear passes.

## 6. Options [28]

### 6.1 One convention

Public fields under `#[non_exhaustive]` (F78: four shapes today);
`Default` equal to named `DEFAULT_*` constants; a `#[must_use]` `with_*`
per field (F64), so `Options::default().with_copies(None)` chains; serde
with `default` and `deny_unknown_fields`: a missing key is the default, a
misspelt one `invalid_option` naming it (F61). An options form has no
`version`: its version is its key set, which only grows, and an older
reader refuses a newer key by name (R6). On the wire `null` is "no
bound" and nothing else; an automatic choice is `"auto"` (F79). Every
field has a flag and a line of documentation (D16). Clamps (`jobs` above
`MAX_JOBS`) apply where the value is read, with the note the command
prints today.

### 6.2 `search::Options`

| field | type | default | JSON | read by | flag |
|---|---|---|---|---|---|
| `engine` | `Option<Engine>` | the dispatch | `"auto"` or a name | front door | `--engine` |
| `fragment` | `Option<Fragment>` | detected | `"auto"` or a name | front door | `--fragment` |
| `bias` | `Bias` | `Auto` | `"auto"`, `"rarer"`, `"factors"` | focused | `--bias` |
| `copies` | `Option<u32>` | `Some(3)` | a number or `null` | focused | `--copies` |
| `forward_copies` | `u32` | 30 | a number | focused | `--forward-copies` |
| `memo_limit` | `u32` | 2²⁰ | a number | focused, additive | `--memo-limit` |
| `test_period` | `Cadence` | `Auto`: every link up to `SMALL_STRUCTURE` (200) occurrences, every `DEFAULT_TEST_PERIOD`th (4) above | `"auto"` or a number | net | `--test-period` (new, F81) |
| `jobs` | `Jobs` | `Count(1)`; `Settings::default()` has `Auto`, every thread the machine runs, resolved by the front end once (walk-through 29); a count clamped to `MAX_JOBS` (256) at use | `"auto"` or a number | focused, net with `parallel` | `--jobs` |
| `schedule` | `Schedule` | `Auto` | `"auto"`, `"turns"` | the default bias's pair | `--schedule` (new) |
| `check` | `bool` | true | a boolean | front door | `--no-check` |
| `pool` | `Option<Pool>` (`parallel`) | none | never on the wire | parallel engines | from `--jobs` |

- `Jobs { Auto, Count(usize) }` and `Cadence { Auto, Every(u32) }` are
  written `"auto"` or the number (P3's second exception).
- `jobs` is on the wire in every build and ignored without `parallel`, so
  one file serves the command and the web client (R49); `pool` is the
  runtime handle beside the data (R116).
- **Dispatch thresholds are not options** (R149, decided): the net row's
  `NET_MULTIPLICITY` is part of a measured row, a private constant whose
  value and measurement are in `Engine`'s doc table (a public constant
  would be a break when step 35 replaces its feature; walk-through 35;
  [35] `search::features(Goal)` gives the harness the features' values); a caller who wants another
  engine sets `engine`, which is the knob D16 asks for. A threshold option
  would let a front end move a row the library's measurement placed.
- Later fields, each defaulting to today's behaviour: [31] `refute_unknown`
  (C2), [35] `net_prunes: NetPrunes { leaves, balance, compounds }` (R143,
  "every setting is sound; for measurement"), [37] the inverse engine's
  knobs (R147), [38] the first-order bounds (a witness depth, instances
  per branch, R38), a loop check (R114).
- `search::Options::default()` keeps the copy bound of 3, since a library
  call without a stop must end (F42); `Settings::default()` is the
  command's behaviour (6.5).

### 6.3 `ViewOptions`, `Clock`, `batch::Options`

`ViewOptions { compact: "auto"|"always"|"never", sides:
"auto"|"one"|"two" }`, `deny_unknown_fields` (F61).

```rust
/// What a front end with a clock applies through its stop, counted from the
/// front end's start; the library never reads it. `null`: no limit.
#[non_exhaustive] pub struct Clock {
    pub time_limit_ms: Option<u64>,        // DEFAULT_TIME_LIMIT_MS = 2000 (R148)
    pub pool_after_ms: Option<u64>,        // DEFAULT_POOL_AFTER_MS = 100: when a race adds its pool
    pub batch_time_limit_ms: Option<u64>,  // the whole batch's limit, none by default
}
#[non_exhaustive] pub struct Options {     // batch::Options, serde
    pub mode: Mode,                        // CLASSICAL, for a problem without its own
    pub cores: Cores,                      // Auto | Across | Within
    pub workers: usize,                    // 1, clamped to MAX_JOBS (F73)
    pub total_memory_bytes: Option<u64>,   // 4 GiB
}
#[non_exhaustive] pub struct Plan { pub workers: usize, pub search: search::Options, pub limits: Limits }
#[non_exhaustive] pub struct Problem { pub name: String, pub sequent: Sequent, pub mode: Option<Mode> }
                  // [later] overrides: Option<(search::Options, Limits)> (R73)
#[non_exhaustive] pub struct Answer { pub name: String, pub outcome: Result<Outcome, Error> }

pub fn prove(problems: impl IntoIterator<Item = Problem, IntoIter: Send + 'static>, batch: &Options,
             search: &search::Options, limits: &Limits) -> Results<Answer>;
pub fn run<P: Send + 'static, R: Send + 'static>(problems: impl IntoIterator<Item = P, IntoIter: Send + 'static>,
             batch: &Options, search: &search::Options, limits: &Limits,
             work: impl Fn(P, &Plan, &Cancel) -> R + Send + Sync + 'static) -> Results<R>;
pub fn run_local<P, R>(problems: impl IntoIterator<Item = P>, batch: &Options, search: &search::Options,
             limits: &Limits, work: impl FnMut(P, &Plan, &Cancel) -> R) -> impl Iterator<Item = R>;
impl<R> Results<R> { pub fn cancel(&self); pub fn canceller(&self) -> Cancel; }   // Drop cancels too
pub struct Cancel(/* Arc<AtomicBool> */);  // is_cancelled(), cancel()
```

`run_local` (no `Send`, the caller's thread, R48) is a separate entry
because features must stay additive: `Send` bounds that appeared only
with `parallel` would break a wasm crate once anything in its graph
enabled the feature. `Results::cancel` reaches searches in flight (R40,
F74); a work closure's stop is `|_| cancel.is_cancelled() ||
deadline()`.

### 6.4 The export options and `Styles`

The seven output values keep their fields and forms and become
`#[non_exhaustive]`. Changes: `png`/`pdf::Options::memory` go (renders
read `limits.memory_bytes`, F63); `rocq::Options::prelude:
Option<String>`, `None` meaning the target's own lines (NanoYalla's
`macroll`, `Classical_Prop` for a classical ordinary certificate,
nothing for LJ: F15, R140); [31] `kernel: Kernel` (R139); strings that
become code are validated types, checked before anything is written
(`InvalidOption`): Rocq's `lemma: Identifier` (an identifier outside the
reserved names, F38), Typst's spacing fields a `Length` of a small
grammar instead of verbatim code, SVG lengths under stated maxima (F4).

```rust
#[non_exhaustive] pub struct Styles {      // R141, what --style keys address
    pub text: TextOptions, pub latex: latex::Options, pub typst: typst::Options,
    pub svg: svg::Style, pub png: png::Options, pub pdf: pdf::Options, pub rocq: rocq::Options,
}
```

Every field exists in every build; a format whose feature is off has a
private placeholder that reads any value and writes nothing, so one
settings file reads everywhere and a misspelt format is still refused.

### 6.5 `Settings`: what a front end holds

```rust
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]         // serde(default, deny_unknown_fields)
pub struct Settings {
    pub clock: Clock, pub limits: Limits, pub search: search::Options,
    pub view: ViewOptions, pub styles: export::Styles, pub batch: batch::Options,
    pub ordinary: ordinary::Options,
}
// [32] Settings::browser(), and reading a partial settings file over a base
//      (`wire::Over`), so a key the page lacks takes the browser's value, not the native one
```

- **`Settings::default()` is the command's behaviour** (F42): `search.
  copies` none (a front end with a clock deepens until its time limit),
  the clock's 2 s and 100 ms, `Limits::default()`. `search::Options::
  default()` keeps the copy bound of 3, and the crate's front page says
  in one paragraph why the two differ.
- [32] **`Settings::browser()`**: `Limits::BROWSER`; `copies` none, `jobs`
  1, `schedule` `Turns`; 2 s and no pool; `styles.svg` with `ids` and
  `description` on and no background. A field table (`Settings::FIELDS`:
  key, kind, default, reader, doc) for the web client's panel and the
  command's help is additive then.
- The dotted keys a command line sets (`--style svg.text=navy`,
  `limits.memory_bytes`) are read by the front end against the settings'
  JSON form, as the command's `style.rs` does today; the library adds no
  JSON dependency for it.

### 6.6 How each front end sets them (D15)

The command builds one `Settings` from `--settings FILE` (R183) and then
its flags, each the spelling of a key:

| flag | key | flag | key |
|---|---|---|---|
| `--timeout` | `clock.time_limit_ms` | `--engine`, `--fragment`, `--bias` | `search.engine`, `.fragment`, `.bias` |
| `--pool-after` | `clock.pool_after_ms` | `--copies`, `--forward-copies` | `search.copies`, `.forward_copies` |
| `--batch-timeout` | `clock.batch_time_limit_ms` | `--memo-limit`, `--test-period` | `search.memo_limit`, `.test_period` |
| `--memory-limit` | `limits.memory_bytes` | `--jobs`, `--schedule` | `search.jobs`, `.schedule` |
| `--occurrence-limit` (every reader, `check` and `interact` included) | `limits.occurrences` | `--no-check` | `search.check` |
| `--derivation-limit` | `limits.derivation_bytes` | `--compact` | `view.compact` |
| `--recursion-limit`, `--work-limit` (new) | `limits.recursion_depth`, `limits.work` | `--style K=V`, `--lemma`, `--prelude`, `--standalone` | `styles.K` |
| `--workers`, `--cores`, `--batch-memory` | `batch.workers`, `.cores`, `.total_memory_bytes` | `--deterministic` | `search.jobs = 1`, `search.schedule = turns`, `clock.pool_after_ms = null` |

The mode flags make the request's `Mode` and `batch.mode`. Whether
`--style-file` stays as a styles-only spelling is the command area's
choice. The harness maps its flags onto the same keys and records its
settings beside each run (R190); the web client keeps the JSON in its
storage, builds its panel from the form, and sends it with each request;
a third wrapper (a notebook, an editor plugin) holds the same value.

## 7. Wire forms [28]

### 7.1 The policy

Stated once in the documentation of the public module `wire` (feature
`serialize`: `Within`, `LEVEL`, and the schema of every form) and in
`core-sequents.md`.

- **`version` is a wire level.** Every top-level document starts with
  `"version": n`, the lowest level whose reader understands all of it;
  `wire::LEVEL` is the highest a build reads and writes; level 1 is
  0.1.0's. The writer computes the level from the content, so a
  propositional, cut-free, commutative value is level 1 in every later
  release and its bytes never change; a proof with a cut is written at
  step 34's level and a 0.1.0 reader refuses it by name. Levels are
  global, numbered as steps land, each one's additions listed under the
  changelog's "Wire forms" and in `wire`'s docs; nested forms carry no
  `version`. (Set aside: a version per form, which both judges found
  equally defensible; one number is what a client stores, and an outcome
  read as a proof then carries one.)
- **What raises the level**: a new tag; a new value of a closed
  enumeration in a form read back (a mode, a criterion); a key whose
  presence changes what other keys mean (`cuts`, an instance table, a
  structured atom table); a structure over a fragment that 0.1.0 refused
  (a MELL net with no box key yet, walk-through 33). **What does not**: a
  key an older reader may ignore without misreading (a counter, an error
  detail, any key of a written-only form); a new value of an open
  enumeration whose meaning an older reader can fall back on (a
  refutation's kind reads as `exhausted`, 3.12).
- **Every key a value of a form read back may lack is
  `serde(default)`**: the optional keys and every key added after 0.1.0,
  so a field added later reads from an older file (walk-throughs 30, 37);
  `Statistics` is `serde(default)` whole. The keys every level-1 value
  has (a sequent's `terms`, `roots` and `atoms`, a proof's `sequent` and
  `nodes`, …, each form's list in `wire`'s docs) are required, so a
  document of another shape is refused naming the key (`Error::Json`),
  never read as an empty value. A counter or key appended after 0.1.0 is
  written only when it is not its default, so a value that does not use
  it keeps its bytes (`statistics` included: walk-throughs 37, 38). No new
  meaning rides on an old tag through a new key (`impact-quantifiers.md`
  §3 item 8).
- **Reading**: a `version` above `wire::LEVEL` is `unsupported_version`
  naming both numbers, before the other keys: the writer puts `version`
  first and the reader records it when it meets the key, so the error
  names the version even when a later key fails. A document without
  `version` reads as level 1, in the level's names only. The pre-release
  names (`ids`, `var_dict`, `proof`, a mode as an object of flags) are
  not read: nothing is kept beside its successor before the release
  (D18), and a second grammar in every reader would be fuzz surface for
  ever. Commit (1) of 7.5 regenerates every fixture that holds them; a
  file in them is refused naming the key it lacks (the next item); nothing
  writes them again (R6).
- **Older levels are converted** [28] (decision 4, the author's):
  `wire::upgrade` is the one place that knows the forms of released
  levels. It reads a document of any level from 1 to `wire::LEVEL` and
  returns the value of the current release, which the writer then writes
  in the current form (at the lowest level that holds it, P8). A level
  that changes a form (a key renamed or retyped, a construct represented
  anew) adds one step from its predecessor; a document that no step can
  carry is `unsupported_version` saying why. Every reader goes through
  it, `Within` and the plain `Deserialize` included, so a reader keeps
  one grammar, the current one. The steps are typed and format-agnostic
  like `Within`: a level that changes a form keeps the previous level's
  private proxy for it and a conversion from it, so the library needs no
  JSON value type. At level 1 it is the identity, built in commit (1) of
  7.5 or the library area's other wire-form commits, with one test (7.5).
  It converts between released levels only, never from the pre-release
  names (the previous item), so it is no alias (D18). The command reads
  every form through it; a command that rewrites a stored file comes with
  the first level above 1, when there is something to convert.

  ```rust
  /// Reads a document of any released level and returns it as a value of
  /// the current release: an older level through each level's step in turn.
  pub fn upgrade<'de, T: sealed::Readable, D: Deserializer<'de>>(document: D, limits: &Limits)
      -> Result<T, Error>;
  ```
- **Unknown keys**: a *data* form (sequent, proof, disproof, structure,
  session, ordinary forms, `Reason`, `Refutation`, `Statistics`) ignores
  them, which the level rule makes safe and which lets an outcome be read
  as a proof or a disproof; an *options* form and a *command* (`Step`,
  [32] `Request`) refuse them by name, since a typo there or a newer
  client's `witness` must not be applied as something else; a client of a
  *written-only* form ignores them, as each such form says (AIP-180).
- **Enumerations by name** (AIP-126), never numbers; a reader refuses an
  unknown value naming it and the known ones; a client treats the open
  enumerations of 7.4 as open sets.
- **Names** (AIP-140, T1): `lower_snake_case` nouns, units in the name
  (`limit_bytes`, `time_limit_ms`), no `is_`, one concept one key in every
  form (`sequent`, `roots`, `atoms`, `nodes`, `mode`, `goal`, `phase`).
  Renamed from the pre-release forms: `ids` → `roots`, `var_dict` →
  `atoms`, `proof` → `nodes`, `memory_limit` → `limit_bytes`, `once` gone
  with `StateEquation`'s payload, the reasons' bare numbers → named
  fields. The tags (`V`, `D`, `⊗`, `ax`, `⊕₁`, …) stay: short, numerous,
  documented (`V` and `D` as an atom and its dual).
- **Numbers** (R247): integers only; ids `u32`. Every integer read back is
  exact in JavaScript (below 2⁵³): a bound that large is written `null`,
  and a reader refuses a larger one, saying `null` lifts it. A written-only
  count that saturates (`Size`, the estimates, a counter) is written as
  `u64::MAX` and means "at least"; `Size::exact` is false then (F70).
  serde_json reads `u64::MAX` back exactly in Rust (the Fable judge's
  probe), and JavaScript reads it as 2⁶⁴, a value no exact count takes.
- **Text** is UTF-8; atom names are identifiers that are no keyword
  (HD5, 3.1); text positions come in bytes, UTF-16 units and line and
  column (4.3). The text syntax is a form too: it grows by new tokens
  only, never by a new meaning of text that reads today, which is why
  every reader refuses the words reserved for later steps from 0.1.0
  (3.1).

### 7.2 Read back or written only

| form | direction | `version` | unknown keys |
|---|---|---|---|
| `Sequent` | both | when top-level | ignored |
| `Proof`, `Disproof`, `ProofStructure`, `Interactive` | both | yes | ignored |
| `Mode`, `Fragment`, `Named`, `Engine`, `Bias`, `Reason`, `Refutation`, `Statistics`, `Size` | both, nested | — | ignored |
| `Step`, [32] `Request` | both, nested | — | refused |
| `Settings`, `Limits` and each options value alone | both | none: the key set | refused |
| `ordinary::Sequent`, `ordinary::Derivation` | both (read through the checks) | yes | ignored |
| `Outcome` (reads back as a `Proof` or a `Disproof`), `ordinary::Outcome`, `Derivation` (R4), `batch::Answer` | written only | yes | client ignores |
| `Applicable`, [32] `GoalView`, `Response`; `Error`, `Progress` | written only, nested | — | client ignores |

### 7.3 The forms

The running example is `A, A -o B |- B`: `⊢ ~A, A ⊗ ~B, B`, arena `0:
~A, 1: A, 2: ~B, 3: A ⊗ ~B, 4: B`, occurrences `0: ~A, 1: A ⊗ ~B, 2: A,
3: ~B, 4: B`.

**Sequent.** `{"version": 1, "terms": [{"D": 0}, {"V": 0}, {"D": 1},
{"⊗": [1, 2]}, {"V": 1}], "roots": [0, 3, 4], "atoms": ["A", "B"],
"antecedents": 2}`. Tags: `V`/`D` an atom/its dual by index; `"1"`,
`"⊥"`, `"⊤"`, `"0"`; `⊗ ⅋ & ⊕` two term indices; `! ?` one; a subterm
before its parents. `roots` as written (C1); `antecedents` written
whenever known, `0` included. Reading checks indices and order, merges
repeated names, refuses a non-identifier name (HD5), counts the
occurrences against the bound before building anything. Before: `ids`,
`var_dict`, no `version`, no `antecedents`. [38], at a level of its own:
an `atoms` entry is a string (nullary) or `[symbol, [term ids]]`; tags
`∀`, `∃` (the body); keys `fo_terms` (`{"B": i}` a de Bruijn index,
`{"S": [symbol, [ids]]}`), `symbols` (`[[name, arity]]`), `binders`
(names for printing).

**Proof.** `{"version": 1, "sequent": {…}, "nodes": [{"ax": [0, 2]},
{"ax": [3, 4]}, {"⊗": [1, 0, 1]}], "mode": "classical"}`. Nodes premises
first, root last: `ax [x, y]`, `⊗ [o, l, r]`, `⅋ [o, p]`, `1 o`, `⊥ [o,
p]`, `& [o, l, r]`, `⊕₁`/`⊕₂ [o, p]`, `⊤ o`, `! ? copy wk [o, p]`, `mix
[l, r]`; every operand a member (one integer). Optional `mode` (R7) and
`goal` (a goal proof, F89). [34] `cuts` (a value in `Sequent`'s form
whose roots are the cut formulas, appended in order) and the tag `cut [a,
l, r]`; [38] `instances`, `frames`, `fo_terms`, `eigenvariables`, tags `∀
[m, e, p]`, `∃ [m, w, p]`. Before: `proof`.

**Disproof.** `{"version": 1, "sequent": {…}, "mode": "classical",
"refutation": {"kind": "equation", "formulas": 3, "needed": 1, "tensors":
0, "pars": 1, "ones": 0, "bottoms": 0, "mix": false}}`, optional `goal`.

**Outcome** (written only):

```json
{"version": 1, "linlog": "0.1.0", "verdict": "proved", "fragment": "IMLL", "mode": "intuitionistic",
 "engine": "net",
 "statistics": {"nodes": 2, "memo_hits": 0, "memo_entries": 0, "splits": 0, "links": 2,
                "tests": 2, "copies": 0, "forward_copies": 0, "work": 2},
 "sequent": {…}, "nodes": [{"ax": [0, 2]}, {"ax": [3, 4]}, {"⊗": [1, 0, 1]}]}
{"version": 1, "linlog": "0.1.0", "verdict": "unknown",
 "reason": {"kind": "memory_limit", "limit_bytes": 100}, "fragment": "LL", "mode": "classical", …}
{"version": 1, "linlog": "0.1.0", "verdict": "unprovable", "sequent": {…},
 "refutation": {"kind": "unbalanced", "atom": 0, "least": 1, "most": 1}, …}
```

- `verdict` closed (D9). For `proved`, the proof's keys (`sequent`,
  `nodes`, `mode`, and `goal` off the roots), so `linlog check` reads
  `prove --format json`; for `unprovable` the disproof's keys, so the
  outcome reads back as a `Disproof` (31's `linlog check` of a refutation).
- `reason` internally tagged so that every kind can gain fields (F86):
  `stopped`, `recursion_limit {depth}`, `copy_bound {copies}`,
  `memory_limit {limit_bytes}`, `index_limit`, `work_limit {limit}`,
  `unchecked {limit_bytes}` (R138). `refutation` likewise: `exhausted`,
  `unbalanced {atom, least, most}` (the atom by index into the sequent
  beside it), `equation {…}`, `state_equation {atoms: [[atom, weight]],
  clauses: [[occurrence, weight]], dropped: [occurrences]}`; [31]
  `classical {assignment}`. Derived on the types, no proxy ending in a
  wildcard (F87, F88).
- `statistics`: every counter of 0.1.0, always written (R9); a counter
  appended later only when not zero; their meaning per engine is on
  `Statistics` (8.5) and their labels on `Engine::counters()`.
- `checked`: whether the proof of a `proved` outcome passed the checker
  (false only under `--no-check`), so a stored row says it (walk-through
  29).
- `linlog`: the crate's version, so a stored outcome says what produced
  it (F77, R190); README's test and the behaviour lock read it as `…`.
  The options and limits a verdict ran under are the front end's to
  record beside it (the harness's CSV columns, the web client's settings);
  the outcome does not repeat them.
- Before: `"reason": "stopped"` or `{"copy_bound": 3}`, `"refutation":
  "exhausted"` or `{"unbalanced": {…}}`, the mode as an object, `proof`,
  none of the new keys.

**ProofStructure.** `{"version": 1, "sequent": {…}, "mix": false,
"links": [[0, 2], [3, 4]]}`: the criterion's fields flattened, so the
pinned `mix` stays; links over vertex ids in the order made; reading
validates as `from_links` and accepts partial or incorrect structures.
Later, each at a level: [33] `vertices`, `boxes`, `jumps`; [34] cut links
and the erased set; [35] the essential criterion; [36] `order`; [38] a
substitution (R8, R108).

**Interactive.** `{"version": 1, "sequent": {…}, "mode":
"intuitionistic", "inferences": [{"sequent": [0, 1, 4], "rule": "⊸L",
"principal": 1, "premises": [2, 1]}, {"sequent": [3, 4], "rule": "ax"},
{"sequent": [0, 2], "rule": "ax"}], "history": [0]}`. Inferences in the
session's order (`GoalId`s are their indices); an open goal is its
`sequent` alone; `rule` is `Named::name` (ASCII spellings read too);
`history` the goals the steps closed. Reading is linear (F21) and checks
the history as each step left the arena (H17). [34] `cuts` (R14); [38]
`instances`, `bindings` and a per-inference `witness` (R97). A `Step`:
`{"position": 1, "rule": "⊸L", "left": [0]}` (`left` absent when empty;
[38] `witness`, a flat term in `fo_terms`' shape, or `"open"`). [32]'s
`GoalView`, `Request` and `Response` are internally tagged
(`{"request": "apply", "goal": 0, "step": {…}}`, `{"response":
"applied", "opened": [1, 2]}`), their table written by step 32 in this
section.

**Derivation** (written only, R4). `{"version": 1, "sequent": {…},
"mode": "intuitionistic", "sides": "two", "inferences": [{"sequent": [0,
2], "rule": "ax"}, {"sequent": [3, 4], "rule": "ax"}, {"sequent": [0, 1,
4], "rule": "⊸L", "principal": 1, "premises": [0, 1]}], "root": 2}`:
premises first, `times` when above 1, for a front end that draws its own
tree.

**Error, Progress.** 4.3; `{"work": 65536, "done": 1048576,
"held_bytes": 12582912, "phase": "search", "item": 0}`.

**Settings** (`Settings::default()`, the export members abbreviated):

```json
{"clock": {"time_limit_ms": 2000, "pool_after_ms": 100, "batch_time_limit_ms": null},
 "limits": {"memory_bytes": 1073741824, "occurrences": 50000000, "derivation_bytes": 67108864,
            "work": null, "recursion_depth": 2048},
 "search": {"engine": "auto", "fragment": "auto", "bias": "auto", "copies": null, "forward_copies": 30,
            "memo_limit": 1048576, "test_period": "auto", "jobs": "auto", "schedule": "auto", "check": true},
 "view": {"compact": "auto", "sides": "auto"},
 "styles": {"text": {…}, "latex": {…}, "typst": {…}, "svg": {…}, "png": {…}, "pdf": {…},
            "rocq": {"form": "fragment", "lemma": "certificate", "prelude": null}},
 "batch": {"mode": "classical", "cores": "auto", "workers": 1, "total_memory_bytes": 4294967296},
 "ordinary": {"translation": "auto"}}
```

Any subset is a file (`{"limits": {"memory_bytes": 268435456}}`); each
member alone is its own form (`search::Options` is the `"search"` object,
R1).

**The batch's record** (written only, F164): one JSON Lines record per
problem, in input order, exactly one of an outcome and an error, `name`
second: `{"version": 1, "name": "a.txt", "linlog": "0.1.0", "verdict":
"proved", …}`, `{"version": 1, "name": "bad.txt", "error": {"code":
"parse", "kind": "malformed", …}}`.

**The ordinary forms** (R11, F53). `ordinary::Sequent`: `{"version": 1,
"formulas": [{"A": 0}, {"A": 1}, {"→": [0, 1]}], "atoms": ["a", "b"],
"left": [0], "right": [2]}` (tags `A`, `"⊤"`, `"⊥"`, `¬`, `∧`, `∨`, `→`,
`↔`), read through the checked constructor. `ordinary::Derivation`:
`{"version", "logic", "sequent", "inferences": [{"left", "right", "rule",
"principal": {"side", "position"}, "premises"}], "root"}`, read and
checked. `ordinary::Outcome` (written only): `{"version", "linlog",
"verdict": "valid"|"not_valid"|"unknown", "logic", "translation",
"target", "linear": {…}, "derivation"?, "reason"?}`. [38] the first-order
keys and tags (one term type for both arenas), per inference `witness` or
`eigenvariable`.

**Forms later steps add**, each at the level current when it lands: the
refutation file is the `Disproof` form (31, R3, R71); `Request`/`Response`
(32); the cut-elimination record `{"version", "proof", "steps": [{"cut",
"case"}], "end"}` (34, R15).

### 7.4 Every string a client meets

| enumeration | values | |
|---|---|---|
| verdict | `proved`, `unprovable`, `unknown` | closed |
| `reason.kind` | `stopped`, `recursion_limit`, `copy_bound`, `memory_limit`, `index_limit`, `work_limit`, `unchecked` | open |
| `refutation.kind` | `exhausted`, `unbalanced`, `equation`, `state_equation` | open |
| `engine` | `focus`, `net`, `two-sided`, `additive`, `horn` | open |
| `bias`; `schedule`; `test_period` | `auto`, `rarer`, `factors`; `auto`, `turns`; `auto` or a number | open |
| fragment | `MLL`, `MLL with units`, `ALL`, `MALL`, `MELL`, `LL`; `I`-prefixed in an intuitionistic outcome | open (38: `MLL1` …) |
| mode | `classical`, `affine`, `mix`, `affine-mix`, `intuitionistic`, `intuitionistic-affine` | by level (36: `cyclic`, `lambek`, `lambek-star`) |
| `compact`; `sides`; `cores`; `form` | `auto`, `always`, `never`; `auto`, `one`, `two`; `auto`, `across`, `within`; `fragment`, `standalone` | closed |
| typst `layout`; `labels`; open goal; svg `advances` | as today | open |
| rule | `Named::name` of every `(rule, side)` (`ax`, `⊗`, `⊸L`, …), ASCII spellings read | by level (34, 36, 38) |
| node tag; term tag | `ax ⊗ ⅋ 1 ⊥ & ⊕₁ ⊕₂ ⊤ ! ? copy wk mix`; `V D 1 ⊥ ⊤ 0 ⊗ ⅋ & ⊕ ! ?` | by level |
| `kind`; `code`; `phase`; `needs` | 4.1; 4.4; 5.2; `nothing`, `split` | open |
| `side` | `input`, `output` | closed |
| logic; translation; ordinary verdict | `classical`, `intuitionistic`, `minimal`; `affine`, `cbn`, `cbv`, `01`; `valid`, `not_valid`, `unknown` | open; open; closed |

`wire`'s documentation is this table and 7.3, and a test checks each row
against the code (`Engine::ALL`, `Mode::NAMES`, `Rule::ALL`, `Node::TAGS`,
`Error::CODES`, …). A JSON Schema or TypeScript declaration, if any, is
`linlog-web`'s (32), not a dependency of the library.

### 7.5 What changes in the pinned forms

`core/tests/lock/json.txt`, the command's lock and README's JSON blocks
change in four commits, each its own change of the behaviour lock that
says why. Three are the library area's (3.1): (1) `version`, `linlog`,
`checked`, T1's renames, the mode by name, the tagged reasons and
refutations, and every fixture regenerated in the new names (7.1); (2)
the written root order and `antecedents` (C1, H9, H10), with the
intuitionistic refusals of 3.6; (3) the disproof's keys in an unprovable
outcome and `StateEquation`'s payload. The fourth is the search area's
(3.2), since the engines count what it adds only there: (4) `work` and
`forward_copies` in `statistics` and the command's `--stats` lines, with
`copies` the level of the search that decided (F144). The commits that
count them come first and keep the counts private, each with the target
set's counters equal: the progress stop at every poll site (5.2) counts
`work`, F144's commit splits the forward search's level from `copies`;
then one commit adds both fields to `Statistics` and its form. Until (4),
the outcome's `statistics` are today's seven counters.
`core/tests/serialize.rs` keeps what the lock cannot show (F90): each
form read back, a file in the pre-release names refused naming the
missing key (7.1), `version: 2` refused by name, an unknown key ignored
by a data form and refused by an options form, `wire::upgrade` the
identity at level 1 (each form's level-1 document read through it equals
the same read through `Within` and writes back byte for byte), a
saturated `Size`
written as `u64::MAX`, a shared-subterm file past `limits.occurrences`
refused by every reader. Nothing else in this design moves a pinned
form. [30] commits the
level-1 files 0.1.0 writes as golden files every later release reads, and
the name lists (`Node::TAGS`, `Error::CODES`, `Mode::NAMES`,
`Rule::ALL`'s names) as append-only lists a test compares (walk-through
30).

## 8. Engines and the search's front door [28]

### 8.1 The front door, and the one place a verdict is built

```rust
pub fn prove(s: &Sequent, mode: Mode, options: &Options) -> Result<Outcome, Error>;
pub fn prove_within(s: &Sequent, mode: Mode, options: &Options, limits: &Limits,
                    stop: impl FnMut(Progress) -> bool) -> Result<Outcome, Error>;
pub fn prove_goal(goal: Goal<'_>, mode: Mode, options: &Options, limits: &Limits,
                  stop: impl FnMut(Progress) -> bool) -> Result<Outcome, Error>;
pub fn engine_for(goal: Goal<'_>, mode: Mode, options: &Options) -> Result<Engine, Error>;

/// Members of a forest that a search decides: its conclusion, or a goal a
/// session left open.
pub struct Goal<'a> { /* private: the forest, the members, whether it is the conclusion */ }
impl<'a> Goal<'a> {
    pub fn conclusion(forest: &'a Forest) -> Self;
    pub fn new(forest: &'a Forest, members: &'a [Member]) -> Result<Self, Error>;   // members checked
    pub fn is_conclusion(&self) -> bool;
    // [34] is_whole_forest(): every tree, cut trees included, which the net engine takes
    // [38] framed(forest, &instances, members)
}

#[non_exhaustive] pub struct Outcome { pub verdict: Verdict, pub fragment: Fragment, pub mode: Mode,
    pub engine: Engine, pub statistics: Statistics, pub net: Option<Box<ProofStructure>>,
    pub checked: bool }   // the proof passed the checker: false only with `options.check` off (7.3)
pub enum Verdict { Proved(Box<Proof>), Unprovable(Box<Disproof>), Unknown(Reason) }   // closed (D9)
#[non_exhaustive] pub enum Reason { Stopped, RecursionLimit { depth: u32 }, CopyBound { copies: u32 },
    MemoryLimit { limit_bytes: u64 }, WorkLimit { limit: u64 }, IndexLimit, Unchecked { limit_bytes: u64 } }
impl Reason { pub fn setting(&self) -> Option<&'static str>; }   // the key that lifts it (R129, R137)
// Verdict, Reason and Refutation have `name()` and `NAMES`, which the serializer writes, so the
// harness's columns and the wire cannot differ (walk-through 29); so have Bias, Schedule, Cadence,
// Compact, Sides and every other word list.
```

[36] A goal's members are a sequence: in an ordered mode `is_conclusion`
means a rotation of the roots, decided by `prove_goal`, which has the
mode, and a permutation that is no rotation is `Error::GoalMismatch`; in
the commutative modes they are a multiset as today.

`prove` is `prove_within` with `Limits::default()` and `|_| false`; the
copy bound of 3 makes it end (F42). `prove_goal` runs, in this order,
each a function of its own, `engine_for` sharing the first three:

1. **`fragment_of`**: the goal's fragment detected, or the asserted one
   checked (`FragmentMismatch`).
2. **`read`**: `Mode::check`, then in intuitionistic mode the reading
   (`NotIntuitionistic`), [36] the planar order in an ordered mode.
3. **`prepare`**: the `Task`; the engine forced (`options.engine`) or
   dispatched (8.4; `NoEngine` when no row takes the goal, never an
   `expect`, F140); its `admits` (8.6).
4. **The set-up poll** on a large forest.
5. **`decide`**, under one account (5.4) and the caller's stop.
6. **`conclude`**, the one place an answer becomes a `Verdict`:
   - `Ok(Some(proof))`: a proof of the conclusion is checked in every
     build (`options.check`; off, a `debug_assert!`), a goal proof against
     its goal; a rejected one is `Error::Rejected` (a defect, never a
     verdict), a refused check `Unknown(Reason::Unchecked)` (F139), else
     `Proved` with the proof's `goal` and `mode` set.
   - `Ok(None)`: `Unprovable` with a `Disproof` holding the engine's own
     refutation, else the count refutation (`focus::refutation`), else
     `Exhausted`; [31] the refuters after it (8.7).
   - `Err(reason)`: `Unknown(reason)`; [31] a refuter may decide it only
     with `refute_unknown` (off by default).
7. **The `Outcome`**: verdict, fragment, mode, engine, statistics, the net.

Every proof of the conclusion has passed the checker when it is returned;
every `Unprovable` rests on an exhaustive search or a refutation; every
refusal is `Unknown` or an `Error` whose kind is a refusal. No engine
builds a `Verdict` (S4). `prove_goal` takes `&Limits`; `engine_for`
costs what `prove_goal` costs before it searches, and with
`Engine::parallel` it is what a front end asks before it adds threads.

### 8.2 The interface every engine implements (crate-private)

```rust
/// A goal as an engine is handed it.
pub(crate) struct Task<'a> {
    pub(crate) forest: &'a Forest,
    pub(crate) goal: &'a [OccId],         // the roots in the forest's order when `roots`
    pub(crate) fragment: Fragment,
    pub(crate) mode: Mode,
    pub(crate) reading: Option<&'a Reading<'a>>,
    pub(crate) roots: bool,
    // [36] the planar order, in an ordered mode
    // [38] instances: Option<&'a Instances>, None for a propositional goal
}
/// What an engine's search ended with.
pub(crate) struct Answer {
    pub(crate) result: Result<Option<Proof>, Reason>,
    pub(crate) statistics: Statistics,
    pub(crate) net: Option<ProofStructure>,
    pub(crate) refutation: Option<Refutation>,   // the engine's own, sound by its argument
    // [31] trace: an opt-in failure record, off by default (R112)
}
pub(crate) trait Decide: Sync {
    /// Refuses a goal the engine does not decide, as a forced engine is
    /// refused: its largest fragment first, then its modes, the goal, the shape.
    fn admits(&self, task: &Task<'_>) -> Result<(), Error>;
    /// Decides the goal. `Ok(None)` only when the search was exhaustive for
    /// this goal in its fragment and mode (an incomplete engine gives up with
    /// a `Reason`, never with `Ok(None)`, R111); polls `stop` at a bounded
    /// interval of work, passing that work; charges `account` for everything
    /// that grows with the input; returns a proof whose premises precede their
    /// conclusions; never panics on a goal it admits.
    fn decide(&self, task: &Task<'_>, options: &Options, limits: &Limits, account: &Account,
              stop: &mut dyn FnMut(Progress) -> bool) -> Result<Answer, Error>;
}
```

`Decide::decide` stays the single entry so that a resumable engine can
replace it without a change of the front door (R47). `Answer::of_arena`
builds the `Proof` (with its goal and mode) for an engine that keeps an
arena; [38] `of_instances` passes the table. A first-order goal enters
through the same `Task` and `Answer` (R109). `Decide`, `Task` and
`Answer` stay crate-private: the choice of engine is the library's,
measured, in one table (D7, D19).

### 8.3 `Engine`: the registration list

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Engine { Focus, Net, TwoSided, Additive, Horn }
impl Engine {
    pub const ALL: &'static [Engine];
    /// The name in text, in JSON and on the command line: an open set.
    pub const fn name(self) -> &'static str;          // exhaustive match
    /// Whether it searches on several threads when `Options::jobs` asks: the
    /// race adds a pool beside one thread only for these.
    pub const fn parallel(self) -> bool;               // exhaustive match, positive (F141)
    /// Its counters: key, label and meaning, which the command's `--stats`
    /// and the harness's header read (walk-through 37).
    pub const fn counters(self) -> &'static [Counter];
    fn implementation(self) -> &'static dyn Decide;    // exhaustive match
}
impl Display for Engine {}  impl FromStr for Engine {}  // serde both ways, by name
```

**Adding an engine** (R86) touches this list and nothing in `prove_goal`,
`Task` or `Answer`; the compiler names 2 to 4: (1) the variant and its
doc (what it decides, the options it reads and those it ignores,
documented, never refused, R147; its unit of work; its counters; its
measured row); (2) `name`; (3) `parallel` (a sequential engine answers
`false` and runs on the calling thread whatever `jobs` says, R110); (4)
`implementation` and the crate-private `modes()`, which `admits` and the
dispatch row both read, so a mode is not written twice; (5) `ALL`, which a
test checks by an exhaustive `match` mapping each variant to its index, and
`counters()`; (6) its module with `impl Decide`; (7)
a `DISPATCH` row only where a measurement shows it winning (D19), else it
stays forceable; (8) `reference::configurations` and a differential test
from the start (R209); (9) README's console blocks; the command's
`--engine` and the harness's `--engines` derive from `ALL`; (10) its rules
file and `core-search.md`.

### 8.4 The dispatch as data

```rust
struct Row { fragment: Fragment, modes: Modes, feature: Feature, engine: Engine }
const DISPATCH: &[Row] = &[
    Row { fragment: Fragment::ADDITIVE, modes: Modes::Any,            feature: Feature::TwoFormulas,      engine: Engine::Additive },
    Row { fragment: Fragment::MLL,      modes: Modes::Linear,         feature: Feature::FewEqualLiterals, engine: Engine::Net },
    Row { fragment: Fragment::MELL,     modes: Modes::Any,            feature: Feature::PetriNet,         engine: Engine::Horn },
    Row { fragment: Fragment::LL,       modes: Modes::Intuitionistic, feature: Feature::Any,              engine: Engine::TwoSided },
    Row { fragment: Fragment::LL,       modes: Modes::Classical,      feature: Feature::Any,              engine: Engine::Focus },
];
```

- **Rows are first-match, in priority order**, each measurement in
  `Engine`'s doc table; a test checks that the table lists `DISPATCH`'s
  rows in order (R87). A row the measurement does not earn is deleted;
  its engine stays forceable (D19).
- **`Modes`** destructures the whole `Mode` (F140). Every row of today,
  `Modes::Any` included, takes commutative modes only, so [36]'s ordered
  modes reach no engine but by a row of their own (R90).
- **`Feature`**: one linear pass over the task, linear memory, no clock,
  polled on a large forest (R88); a new one is a variant and an arm of
  `Feature::of`. Coming: [35] the pure-tree feature, [37] "many
  hypotheses, small goal" (which may read the written sides for routing,
  never for meaning; walk-through 37), each only with a row it earns.
  [35] also places R115 here: a focused search at its recursion limit
  handing a goal to the net engine is a decision of `conclude`, not of an
  engine, whose `Outcome.engine` and statistics are the deciding engine's
  and which `engine_for` documents it cannot predict.
- **No row takes a fragment it does not name**: [38] a goal with the
  quantifier bit reaches only the rows that name `LL1`, and every other
  engine's `admits` refuses it (R135).

### 8.5 `Statistics`, per engine (T5)

```rust
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize), serde(default))]   // no proxy (F88)
pub struct Statistics {
    pub nodes: u64, pub memo_hits: u64, pub memo_entries: u64,  // u64 on 32-bit targets too (R46)
    pub splits: u64, pub links: u64, pub tests: u64,
    pub copies: u32,          // the level of the search that decided, under `copies`
    pub forward_copies: u32,  // the forward search's level, under `Bias::Auto` (F144); area 3.2
    pub work: u64,            // the units of work; on one thread, the sum of every Progress::work; area 3.2
    // appended by later steps, each with its line in this table and a CSV column at the end
}
impl Statistics { pub fn add(&mut self, other: &Self); }   // the race and a batch merge with it
```

T5's recommended answer: the shared counters stay, since every baseline
and `bench/targets.sh` read them by name; each is documented per engine
on the type and in the JSON form; an engine adds a field only where no
shared counter fits, and never gives an existing one a new meaning on an
engine that already fills it (R9).

| field | focus, two-sided | net ([35] essential) | additive | horn | [37] inverse | [38] first-order focus |
|---|---|---|---|---|---|---|
| `nodes` | stable sequents | literals chosen | pairs | markings reached | sequents derived | stable sequents |
| `memo_hits` | memo hits | 0 | memo hits | markings met again | conclusions subsumed | ground memo hits |
| `memo_entries` | the memo's peak | 0 | the memo's peak | markings kept | the database's peak | the ground memo's peak |
| `splits` | split steps | 0 | 0 | 0 | 0 | split steps |
| `links`, `tests` | 0 | links tried, exact tests ([35] closure updates in `tests`) | 0 | 0 | 0 | 0 |
| `copies`, `forward_copies` | levels | 0 | 0 | 0 | 0 | levels |
| added | | [35] per-prune counts if the harness's ablation needs them | | | none planned | `unifications`, `bindings` |

### 8.6 What a forced engine refuses

```rust
Error::EngineRefused { engine: Engine, because: NotTaken }
#[non_exhaustive]
pub enum NotTaken {
    Fragment { decides: Fragment, goal: Fragment },  // beyond its largest fragment
    Mode { mode: Mode },                              // not in its modes
    Goal,                                             // not the roots (the net engine)
    Shape,                                            // not two additive formulas, not a Horn program
}
```

Every `admits` checks, in this order, its largest fragment (focused:
`LL`, [38] `LL1`; net: `MLL`; additive: `ADDITIVE`; Horn: `MELL`), its
modes, the goal, the shape (F140, R135). Nothing is refused that was not
before (every detected fragment lies in `LL`); `Display` writes per engine
and reason the words the behaviour lock pins. The harness classifies by
`ErrorKind::Unsupported` instead of a list of variants (R135).

### 8.7 Refuters, the second plug-in kind [31]

C2's answer: a refuter is a crate-private kind beside the engines, which
`conclude` runs **after** the search, never during: after an `Unprovable`
whose refutation is `Exhausted`, to attach a checkable one, and, with
`options.refute_unknown` (off by default, so every baseline keeps its
verdicts), after an `Unknown`, turning it into `Unprovable` only with a
refutation `Disproof::check` accepts. A refuter never changes a verdict
the search gave (research conflict 8). It runs under the caller's stop and
a fresh account of the same bound, on the calling thread, with its own
bounds as `Options` fields with `DEFAULT_*` constants and flags; no clock,
so the web client has it too.

```rust
pub(crate) trait Refute: Sync {
    fn applies(&self, task: &Task<'_>) -> bool;
    fn refute(&self, task: &Task<'_>, options: &Options, limits: &Limits, account: &Account,
              stop: &mut dyn FnMut(Progress) -> bool) -> Option<Refutation>;
}
```

The kind arrives with its first member beyond the counts, step 31's
classical assignment (a bounded DPLL over the erased formula, fixed
order), as C2's text has it; the count refutation of today stays where
`conclude` calls it, and becomes the list's first row then (section 14,
decision 6). G4ip countermodels (R72, R113) plug in the same way.

### 8.8 Names inside (F56, F113)

The engines' structs are named by role, not `Engine`: `focus::Focused`
(the `Decide` value) and `focus::Run` (a run), `net::Linker`,
`additive::Pairs`, `horn::Horn`; `focus::Rules` → `Switches`,
`schedule::Rule` → `Plan`, `check::Problem` → `Fault`, `batch::Answer`
stays the record's name (it is public); the result alias `focus::Search`
→ `Searched`.

## 9. Exports [28]

### 9.1 One entry per target

```rust
pub trait Drawable: sealed::Sealed {}                      // F36
impl Drawable for proofs::Derivation<'_> {}
impl Drawable for ordinary::Derivation {}
// in latex, typst, svg:
pub fn write(d: &impl Drawable, options: &Options, out: &mut impl fmt::Write,
             stop: impl FnMut(Progress) -> bool) -> Result<(), Error>;
pub fn sequent(s: &Sequent, mode: Mode, options: &Options, limits: &Limits) -> Result<String, Error>;
// in svg:
pub fn net(n: &ProofStructure, style: &Style, limits: &Limits,
           stop: impl FnMut(Progress) -> bool) -> Result<String, Error>;
// in png, pdf:
pub fn from_svg(svg: &str, fonts: &[&[u8]], options: &Options, limits: &Limits) -> Result<Vec<u8>, Error>;
```

- **One signature writes anything drawable**; a `String` is an
  `fmt::Write`, so the per-type wrappers, the `ordinary` functions and the
  `String` twins go (F36); a new target (Lean, R158) is a module of this
  shape and a `Styles` field. The sealed trait keeps a target from being
  handed what it cannot draw at compile time. The sequent printers print
  two-sided by mode, through `Notation` (F82); the estimate is checked
  against `limits.derivation_bytes` before anything is written (F5, R26).
- **Options** are one value per output (D15, 6.4), validated before
  writing (F4, F38).
- PNG and PDF stay out of the web client's first features (resvg and
  krilla unbuilt for wasm32; step 32 records R44; a browser prints the
  SVG).

### 9.2 The Rocq export and the library of step 31

```rust
#[non_exhaustive]                     // serde(default, deny_unknown_fields)
pub struct Options { pub form: Form, pub lemma: Identifier, pub prelude: Option<String> /* [31] kernel: Kernel */ }
// [31] #[non_exhaustive] pub enum Kernel { #[default] Auto, NanoYalla, Linlog }
pub const NANOYALLA: &str = "1.1.3";
// [31] pub const LIBRARY: &str;  pub const FORMAT: u32;   the rocq-linlog release and the certificate format (R12)
pub fn write(d: &proofs::Derivation<'_>, options: &Options, out: &mut impl fmt::Write,
             stop: impl FnMut(Progress) -> bool) -> Result<(), Error>;   // NanoYalla's script, as today
```

- **At 28** the Rocq writer keeps its output byte for byte (the snapshots
  `ll`, `mll`, `mall`, `mell`, `labels`, `ill`, `labels_ill` stay);
  `lemma` is checked for its lexical form (F38; the names a kernel
  reserves are checked when that kernel writes, since a stored lemma must
  not become invalid when a kernel is added: walk-through 31), the atoms
  are named by their keys (3.2), `prelude: None` is the target's own import,
  for the ordinary certificate too (F15, R140); `Unsupported` is
  `#[non_exhaustive]` and refuses Mix, affine weakening and an open goal
  by name, before anything is written.
- **[31] `Kernel::Auto`** writes NanoYalla's script exactly as today for a
  proof in classical mode without Mix and affine weakening, and the
  library's term certificate otherwise (R139, R159); `NanoYalla` refuses
  the rest (`Unsupported::Rule { kernel, rule }` for Mix,
  `AffineWeakening`, [34] `Cut`, [38] `Forall`, `Exists`; `Mode` for an
  ordered mode); a goal proof is `Error::GoalProof`.
- **[31] The term certificate** (kernel `Linlog`, R152): `rocq::
  write_proof(&Proof, Mode, &Options, out, stop)` states the sequent over
  the library's inductive (`ll p [formulas]`, `p` the record `{mix;
  affine}`; intuitionistic, `ill a [hypotheses] goal` from the `Reading`,
  R54), schematic in the atoms, and proves it by computation over data:
  the formulas in written order (the Rocq side recomputes the forest by
  3.3's contract, R69), the nodes in arena order as a list of `node nat`
  (one line each, shared nodes once: linear in the proof, polled per
  node), and in intuitionistic mode the positions as data, which the Rocq
  checker verifies locally (each position agrees with its parent's by the
  grammar; the goal is the written succedent) instead of mirroring the
  reading's choices. It begins with a comment naming linlog's version, the
  library's and the certificate format, and `Check
  Linlog.Certificate.format_1.`, so a library without that format fails
  on that line by name: that ties the writer to the library (R12); a flake
  check compares `LIBRARY` with the opam file.
- **[31] The mirror** (R117, R118, D20): `node (M : Type)`, one
  constructor per `Node` variant in `Node::TAGS` order, the operand of type
  `M` (`nat`, an occurrence, propositionally; an index into the
  certificate's instance table in 38's development beside it, R68),
  premises as list indices; `check` a fold over the nodes translating
  `oracle.rs` (3.8). One soundness theorem per calculus, in `Prop` (the
  author's question 1 of step 31), no `Admitted`, `Print Assumptions`
  closed.
- **[31] Refutation certificates** (R153): `rocq::write_disproof(&Disproof,
  …)` states `~ ll p [formulas]`, proved from the kind's lemma and a
  computation over its numbers; `Exhausted`, and every refutation under
  NanoYalla, is `Unsupported::Refutation`. Reserved names per kernel
  (R155) through the crate-private `identifiers(atoms, &reserved)` (a hash
  set, F3); the statement printer is one loop over `Visit` (R156).
- [34] `params.cut`, a `Cut` constructor, a new `FORMAT`; the eliminated
  proof certifies with `cut := false`. [38] the second development, a new
  `FORMAT` (R161, R242).

### 9.3 Drawings and their ids

`svg::Style::ids` documents the id grammar as a public contract (R163,
R168): `i<n>` a drawn inference, `i<n>-<p>` the formula at `Step`'s
position `p` (`Interactive::derivation_ids` maps `n` to a `GoalId`),
`o<n>` a net's vertex (an occurrence in MLL), `l<m>-<n>` a link by vertex
ids; prefixes `b`, `d`, `j`, `c` reserved for boxes, doors, jumps and cut
links. `svg::Style` is `#[non_exhaustive]` and gains box and
essential-net fields at 33 and 35 with defaults that keep every snapshot
(R144, R166); whether an interactive drawing's root becomes
`graphics-document` with focusable targets is step 32's choice, a `Style`
field defaulting to today's `img`.

## 10. How each later step enters it

Every item below is new (a function, a type, a variant or field of a
non-exhaustive type, a key at a level) or a planned 0.y bump of a closed
enum; none changes a signature step 28 fixes. Section 12 has what the
walk-through of each step found.

### 10.1 Step 29: the comparison

Drivers read the outcome's `verdict`, `engine`, `statistics` and `linlog`
(R172, R190); the one-thread column is `--deterministic` (`jobs` 1,
`Schedule::Turns`, R151), the default column `race` (R85), both under one
account equal to BenchExec's memory limit (R18); "outside the fragment"
is `ErrorKind::Unsupported` (R75, R135); translators use the public
`sequents::fmt::Walk`, `Reading::walk` and `Sequent::antecedents` (LLTP's
axioms and conjecture are the written sides, R74, R245); `Sequent::
fragment` and `Mode::name` tell a driver whether a tool covers a problem
(R75). A per-problem routing query (R204) is an additive function beside
`engine_for`. A `proved` outcome is checkable ground truth at 29 (its
`Proof`, with `checked`); an `unprovable` one is a `Disproof`, checkable
from step 31's `Disproof::check` and then only for the kinds it certifies,
so the comparison lists a contradiction against linlog's `unprovable` as
unchecked until then (walk-through 29).

### 10.2 Step 30: the release

The changelog's policy line names the closed enums and the planned bumps
(0.2.0 at 34, 0.3.0 at 38) and has a heading for wire-level changes (R6,
R221); `cargo-semver-checks --all-features` on `linlog` only (R215,
R253), which passes from 0.1.1 since 0.1.0 has no baseline, and the rule
that the first breaking commit bumps the version in the same change;
R41's wasm32 build of the library as a flake check, since the published
crate must not pin a design the web client would undo (walk-through 30);
the golden level-1 files and the append-only name lists (7.5); the wire forms are promised by level, independently of the crate's
version (a new level is a changelog entry, never by itself a major bump);
the command's output is not "the API" (D18); `core.md` lists every public
enum and every struct with public fields with its choice (R50);
`wire::LEVEL` is 1.

### 10.3 Step 31: the Rocq library

`Kernel::Linlog`'s writer (`rocq::write_proof`, `write_disproof`,
`LIBRARY`, `FORMAT`), `rocq/` with the mirror of 3.8 and 9.2,
`Disproof::check` sharing no engine code (R124, in `refutation.rs`), the
first refuter (the classical assignment) with `Refutation::Classical` and
`refute_unknown` (C2, 8.7), R154's classical "not valid" certificate for
an ordinary sequent through the same refuter and `write_disproof` over
the image, the tests R118 (`Node::TAGS` against the Rocq constructors)
and R199 (the Rocq checker against the Rust one on mutants). `node (M :
Type)` leaves room for 38; `params` is a record 34 extends. No nullary
Mix (C3). Nothing breaks: `Kernel`, `Unsupported`, `Refutation` are
non-exhaustive, `Disproof` exists from 28. Step 31 takes the proof term
as step 28 leaves it.

### 10.4 Step 32: the web client

`linlog-web` (a workspace member, its wasm32 build a flake check, R41)
exports JSON-in, JSON-or-SVG-out calls over `wire::Within`: prove a
sequent, drive a session, check a proof or a disproof, decide an ordinary
sequent. It adds `Request`, `Response`, `Interactive::serve` and
`GoalView`/`FormulaView` in the module `session` (3.10), `Error::FeatureOff`,
the panic hook (4.1), `Limits::BROWSER` and `Settings::browser()` with
`wire::Over` (5.1, 6.5), and the builder only if the client gets a formula
editor (3.1); measures the presets and the wasm32 per-level stack figure
(D16); keeps a live session in its worker, the page holding the session's
JSON as recovery (3.10); stops with a progress closure that reads
`performance.now()` every so many units of work (5.2); runs the limit tests
at 32 bits (R46); ships `parse`, `serialize`, `interactive`, `svg`,
`latex`, `typst`, `rocq` (R49) and binds clicks to 9.3's ids (R163). The
derivation's form is "the proof plus `ViewOptions`", rebuilt by the
client's call, or the written-only `Derivation` form (R4).

### 10.5 Step 33: MELL nets with boxes

The vertex table past the forest (instances of `?` subtrees, collectors,
doors, box nodes) behind `VertexId`, a `BoxId`, `boxes`, `box_of`,
`depth`, `jumps`; the generalized `?` node, one door per (`?`-instance,
box); jumps for weakening and `⊥` chosen canonically from the term (C3's
answer A); the criterion per box depth reachable through `is_correct`,
allocation-free, the CSR graph with variable arity (R102);
sequentialization through boxes; `from_proof`'s exponential arms (R122,
R254); `NetError`'s box variants with witnesses naming the box (R132);
the net form's `vertices`, `boxes`, `jumps` at a level (R8); `svg::Style`'s
box fields and the `b`/`d` ids, with a layout order that may permute the
roots for drawing only (R144, R165, R168). `Criterion` still refuses
affine mode (R145), forcing the net engine on MELL still refuses (R89).
Nothing breaks: the API speaks `VertexId` from 28, and no `Node` is added.

### 10.6 Step 34: cut

`Node::Cut(Member, NodeId, NodeId)` and `Rule::Cut` (0.2.0);
`Fault::NotACut`; the checker's and the oracle's arm (3.8);
the sequent's `cuts` (3.1), which `Forest::new` numbers after the
conclusion, `cut_pairs()`, `dual()` (total on a pair's `A` tree, its
inverse on the `A⊥` tree) and `is_conclusion()` (3.3); `Goal` and `Task`
name the trees a goal's members lie in, so the dispatch's features, the
bias and the counts read the goal's occurrences, a cut-free conclusion
keeping its fast path (R98; walk-through 34);
the proof form's `cuts` and `cut` tag at a level (R13); the reading of a
cut pair (R99); `Goal::is_whole_forest`, so the net engine never sees a
cut tree and the engines see the same goal (R98; the dispatch's features
and counts read the conclusion's occurrences, counter-neutral);
`Interactive::cut` (R95) and the session form's `cuts` (R14); cut refused
in an ordered mode (R57); and elimination:

```rust
pub mod cut {   // in proofs
    #[non_exhaustive] pub struct Options { pub strategy: Strategy }       // serde, default
    #[non_exhaustive] pub enum Strategy { #[default] LowestFirst }
    pub fn step(proof: &Proof, at: Option<NodeId>, options: &Options, limits: &Limits,
                stop: impl FnMut(Progress) -> bool) -> Result<Reduction, Error>;
    pub fn eliminate(proof: &Proof, options: &Options, limits: &Limits,
                     stop: impl FnMut(Progress) -> bool) -> Elimination;
    #[non_exhaustive] pub struct Reduction { pub proof: Proof, pub cut: NodeId, pub case: Case }
    #[non_exhaustive] pub struct Elimination { pub proof: Proof, pub steps: Vec<(NodeId, Case)>, pub end: End }
    #[non_exhaustive] pub enum End { Normal, Refused(Refusal) }    // a bound is never "normal" or "invalid"
}
```

Every intermediate proof lives over the same extended forest and passes
the checker; `work` counts steps, `memory_bytes` the arena (R32, R100);
`Phase::Eliminate`; R131's codes; its written form (R15). Net elimination
(R101) is a step function with cut links and an erased set,
`NetError::CutNotDual`, a level of the net form, and an SVG drawing of
each step (R167). The search stays cut-free. Rocq: `params.cut`, a new
`FORMAT`.

### 10.7 Step 35: the MLL engines

- **One structure for the symmetry breaks** (R103): the net engine's
  chains of equal literal conclusions become `Symmetries`, built once per
  run in one forest pass (groups of literals of one atom and sign under
  the same maximal pure tree), so the leaf break adds groups and no code
  path; all pruning state is a pure function of the links, kept in
  `link`/`unlink` (R33), every new rule's work per candidate O(1) or
  O(atoms touched) inside the polled loop (R34).
- **The balance prune** reads per-component, per-atom counts that the
  skeleton's union-find keeps and its undo log restores (R82).
- **Equal compound conclusions** need structural classes valid for any
  sequent: a crate-private `Forest::structural_classes()` (fixed-seed
  hash, O(n), R83), which 37's classes reuse.
- **Essential nets are a criterion, not a second algorithm** (D7, R105):
  the net engine's loop becomes `net::Linker<C: Correctness>`,
  crate-private, `Switching` today's test (its `nodes`, `links`, `tests`
  equal before and after) and `Essential` (directed acyclicity by an
  incremental transitive closure with an undo log paired with
  `link`/`unlink`, the dominator condition at a complete linking).
  `ProofStructure::is_essential` is the criterion independent of any
  search (R123, linear in memory), with `NetError::{DirectedCycle,
  NoDominator}`, and no field of `Criterion` (3.11); a net's positions come from
  `Reading::new(net.forest())` (R81). Whether the essential search is a
  forceable `Engine` variant or the net engine under the essential
  criterion is the step's measurement (D19; a variant is the only way the
  harness can force one against the other, and keeps `tests` one meaning
  per engine); its closure matrix (n² bits) is charged to the account,
  reserved fallibly (5.1) and refused with `Reason::MemoryLimit` or
  `IndexLimit` (R35), and its row needs a size feature so that a large
  IMLL goal is not routed into a refusal.
- **The symmetry groups are the criterion's** (walk-through 35): swapping
  two equal literals of one pure `⅋` tree is an automorphism of the
  undirected switching graph, not of the directed essential structure
  (in `a ⊸ a ⊸ b` the two `~a` sit at different implication links), so
  `Symmetries::new(task, criterion)` takes the reading's positions and each
  `Correctness` owns its key; the panel reads the orbit argument per
  criterion. The balance table is charged and reserved fallibly (5.1).
- **Ablation** (R143): `search::Options::net_prunes`, all on, every
  setting sound, its wire form a list of the prunes switched off, read by
  name, a removed name still read as a no-op, since a prune that wins
  nowhere is deleted. `search::features(Goal) -> Features` gives the
  harness the routing features' values (R204). `NetError` and the
  drawing: `svg::Style` selects the essential layer, `e` is reserved for
  dominator edges beside `b`, `d`, `j`, `c`. **R115** (a focused search at its recursion limit handing
  a goal to the net engine) is a decision of `conclude` (8.4), measured
  here. Essential-net drawing: `svg::Style` fields, today's drawing as
  default (R144, R166).

### 10.8 Step 36: cyclic MLL and the Lambek calculus

- **`Mode`** gains private fields with builders: `order: Order`
  (`Commutative` by default, `Cyclic`) and `empty_antecedents: bool`
  (false: the calculus L; true: L\*, D16); `Mode::CYCLIC`,
  `Mode::LAMBEK`, the words `cyclic`, `lambek`, `lambek-star` at a level;
  `Mode::check` refuses an ordered mode with weakening or Mix (R134).
- **The written order is already canonical** (C1, done at 28).
  "The same cyclic sequent" is a rotation, which `prove_goal` accepts as
  the roots in a cyclic mode; `Forest::roots()` is the written rotation
  (R53).
- **The dual of a product in an ordered mode** (R56, research conflict
  3). Classical cyclic MLL has no reading to recover an order from, so
  its reader must dualise with reversal (`(A ⊗ B)⊥ = B⊥ ⅋ A⊥`): with D1's
  order-keeping dual, `|- ~(a * b), a, b` would come out unprovable in
  the mode named `cyclic`, a wrong answer (walk-through 36). So **the
  ordered parse** (3.1's sibling of `parse_within`, taking the order)
  lowers with a reversing dual, the planar order is then the preorder of
  the forest (no second numbering), and `Inference::sequent()` ascending is
  planar. For the Lambek calculus a second way exists, the planar order
  derived from D1's lowering through the reading (draft C's lemma,
  untested); the panel chooses, and the reversing parse serves both. The
  commutative `Term::dual`, the JSON, every stored proof and every
  commutative snapshot stay; under the ordered parse "the roots as
  written" means as lowered (the antecedents reversed), which the
  two-sided printer reads back through the reading. The builder takes the
  order before its first `dual` (3.1).
- **Divisions**: `A \ B` parses to `A⊥ ⅋ B`, `B / A` to `B ⅋ A⊥`; in an
  ordered mode the reading takes the antecedent from whichever factor is
  in input position (that is how `\` and `/` differ; L has no units); the
  printers write them back; `Named` gains the orientation.
- **The engine** is the net engine with planarity as a third rejection
  (one engine per algorithm), selected once by a type parameter beside
  `Linker<C>`'s criterion, `Commutative` the empty instance, so the
  commutative engine pays nothing (D17); `Criterion` gains `order` and the
  L restriction (both flattened on the wire, at a level) with
  `NetError::EmptyAntecedent`, since a net found under a criterion without
  it could sequentialize only to proofs the checker refuses; the
  symmetry breaks are off under order; a `Modes::Ordered` row; every other
  engine refuses an ordered mode with `NotTaken::Mode` (R90, R104). Units,
  Mix and weakening in an ordered mode are refused with named errors
  (R134).
- **The checker** reads zones in the planar order: one sort per `⊗` to
  test that the premises' contexts are the two arcs, the axiom on adjacent
  literals, and in L the fourth clause R4 (`Fault::EmptyAntecedent`), the
  oracle alike (R121, R246). `Inference::sequent()` is in planar order in
  an ordered mode (R61); `Split::At { position }` splits an ordered goal
  (R96); `StepError::Exchange` refuses what needs exchange; NanoYalla has
  no cyclic certificate (`rocq::Unsupported`); the exports draw planar
  arcs and no exchange (R169).

### 10.9 Step 37: the focused inverse method

- **First commit, counter-neutral**: `Context`, `Classes`, the atom bias
  (`bias::signs`) and the arena with its collector move from
  `search/focus/` to `search/zone/` (shared types: D7 forbids two engines
  of one algorithm, not shared data), `bench/targets.sh`'s columns equal
  before and after; `search/zone/` is also where step 38's `Zone` trait
  lives (section 11), so nothing moves twice. The reverse map from a class
  to its occurrences (R84) is a type of its own, built by the engines that
  rename (`ClassMembers`), so the focused engine's charged memory does not
  move; one notion of class serves 35 and 37 (walk-through 37).
- **D-7's pinned time** (G3, decision 20) is the lift's first measurement,
  on an idle machine: the spike's generic zone with its second instance
  (M2d and M3i, rebuilt on the lifted `search/zone/` from
  `spike-report.md`, which names their abandoned commits) against the
  lift, the target set's rows over a second and the search journeys
  pinned to one core, five runs each. Within a few percent, 38 makes the
  engine generic; past it, 38 takes D17's duplicated fast path (11.4's
  last row).
- `Engine::Inverse` (`"inverse"`), `parallel()` false, its fragments and
  modes in a table on the variant (R135, R147); the database and its index
  charged by capacity (R36); the loop polls at every given clause, in
  every long subsumption scan and index rebuild, and in the set-up on a
  large forest (R37).
- `Ok(None)` exactly when the saturation completed and no rule
  application was cut by a bound, in every fragment (a terminating MELL
  saturation is a refutation, not a give-up); a bound that cut it is a
  `Reason` (R111; walk-through 37). An exhausted saturation gets its
  refutation from `conclude` (R137); no `Refutation::Saturated`, which a
  checker without the database could not verify. Since `prove` must end
  without a stop, the inverse engine's options (`InverseOptions`, nested
  in `search::Options`) carry a `DEFAULT_*` bound of their own with its
  `Reason`, flag and JSON key (R147).
- The forward derivation becomes today's `Node` terms (sharing is the
  DAG, a subsuming smaller sequent a `Weaken` below, a `Θ` use a `Quest`
  and `Copy`; the weak flag is the checker's `any`), checked by the front
  door like any proof (R127). Equal occurrences interchanged by the
  classes mean the read-off is a top-down renaming with a memo on (node,
  assignment), charged to the account, unsharing where a subproof serves
  two assignments, and in affine mode a `Weaken` at the use site. No
  `DISPATCH` row unless a measurement earns one.

### 10.10 Step 38: first-order linear logic, in full

**The first step of adding quantifiers** is the data model alone, with
its refusal everywhere: `Term::{Forall, Exists}` and `Kind::{Forall,
Exists}` (0.3.0), the tables of 3.14 (empty for propositional input), the
structured atom table and its interning (3.2), the `QUANTIFIERS` bit set
by a binder, and in the same commit every engine's `admits` refusing the
bit (`EngineRefused`, R136) and `Proof::check` refusing a forest with
binders (`Fault::Instance`) until the frame pass exists (R135: "the data
model comes first and by itself"). Ground first-order input is accepted
from that commit on, by every engine, as propositional input with
structured atoms (3.2). The target set's counters are identical after
it; each later commit of 38 makes one layer accept binders.

- **(a) Atoms as predicates over terms.** An atom is `(symbol, arguments)`,
  interned, its key the canonical text (3.2); symbols `(name, arity)` in
  one namespace; the term arena `fo` (`FoTerm::Bound`, `App`, constants
  with empty arguments), argument lists hash-consed in a CSR table, all
  topological; `Sequent::check` verifies indices, arities and scope in one
  ascending pass (per term its largest loose de Bruijn index, 0 at every
  root). The literal lists stay by atom and sign; a symbol → atoms index
  gives the first-order engine its candidates; `Sequent::{atom_symbol,
  atom_arguments, atom_is_open, symbols, fo_term, binder_name}` are the
  public accessors, all iterative.
- **(b) Binders in the arena.** `Term::{Forall, Exists}(TermId)`,
  `Kind::Forall` negative, `Kind::Exists` positive, `dual` swapping them,
  locally nameless (3.14); a binder is a unary occurrence with its body at
  `o + 1`, so the forest's numbering and arrays stay, plus a binder-depth
  array only when the sequent has binders. The parser's binder entry on
  its one stack resolves names against the open binders; 100 000 nested
  binders and terms parse, print, unify and check on 256 KiB (R43). The
  syntax is the author's (fo-linear §7 Q1); the arena does not depend on
  it.
- **(c) The member as an occurrence and a frame.** A member past
  `forest.len()` is `(OccId, FrameId)` in an `Instances` table of a proof,
  a session or a search (3.14); a frame lists the terms the binders above
  the occurrence are bound to, hash-consed, so equal instances are equal
  members. fo-linear-44 (`!∀x.p(x) ⊢ p(a) ⊗ p(b)`: frames `[a]` and `[b]`
  of one body occurrence in one `Θ`) is two members, fo-linear-62's three
  copies of one clause three; no signature changes.
- **(d) The substitution beside the forest.** The sequent's tables are
  read-only; a search keeps beside the forest `Bindings` (per metavariable
  an optional term and a level), a `Trail` (the variables bound and the
  levels lowered, a mark per choice point; undo pops to the mark and
  truncates the search's term extension), and counters that number fresh
  variables per worker, never by address (R39). The eigenvariable
  condition is kept by levels (fo-linear §3.2): a variable carries the
  level of its creation, an eigenvariable one more than its branch's;
  binding a metavariable to a term with a younger eigenvariable fails; the
  occurs check and the lowering ride on one iterative walk that visits
  each shared node once (R106, R250).
- **(e) The trail in the focused engine** (step 26's report, item 7): a
  mark at a choice, undone by truncation exactly where the arena's
  pending nodes are released: `prove_stable` on a failure; the four places
  where a first premise was proved and the second failed (`premises`,
  `both`, `parts`, the forced chain in `split`); each failing alternative
  of `choose_here`; a frame of `split::chain` whose split failed.
  Bindings cross the premises of `⊗` and Mix, undone only when the rule
  fails, and a failure on the right must ask the left for **another
  answer**: one answer per goal loses `p(a) & p(b), q(b) ⊢ ∃x.(p(x) ⊗
  q(x))` (impact-quantifiers finding 3), so the framed engine enumerates
  a premise's answers (a resumable premise search, or witnesses from a
  finite candidate set without function symbols); the mechanism is 38's
  and its panel's, and the propositional `Found` stays. A pool's worker
  starts from its spawner's trail; no binding reaches another
  alternative; premises sharing an unbound metavariable run in order on
  one thread. **Generic** (the spike's answer on counts, 11.5; pinned time
at 37's lift, decision 20): the engine is
  generic over `Z: Zone` with the seam the spike found (a handle per dual
  list with its mark and its `Θ` flag, `dual_from` with its cursors, the
  marks' bytes charged, `fork` for a pool's worker, `root`, the forest
  passed per call), the memo and the loop check keyed by the ground part
  behind `Zone::memoizes`, and `#[inline]` on the hot helpers both
  instances share.
- **(f) The prunes that assume ground atoms**, each given a first-order
  form or switched off for a goal with the bit, the propositional path
  unchanged (R107):

  | prune | first-order form |
  |---|---|
  | `initial`, `dual_in`, `dual_from`, `mark_literals`, `meets` | the literal seam (one internal interface, R109): candidates by predicate symbol through the index, a dual is a member that unifies; several candidates are a choice point |
  | forced splits (`Forced::Dual`, `Duals`, `literal_tensor`, cursors) | a choice among unifiable duals; forced only when a candidate is unique and ground |
  | interchangeable occurrences (`Classes`, canonical keys and splits) | classes of ground members only; a framed member is its own class |
  | the memo and the loop check | ground stable sequents only, keyed by instance; a `Complete` failure may be shared under a variant key later, `Proved` never |
  | counts, the count equation, the bias | per atom as today for a symbol whose atoms are all ground; per symbol, over all its atoms, for a symbol with any open atom: sound, weaker. Per atom there is unsound: in `∀x.p(x) ⊢ p(a)` the open `p(#0)` becomes `p(a)`, and a count of `p(a)` alone refutes a provable sequent. The count refutation is per symbol there, or skipped; `Refutation::Unbalanced` names an atom, so a per-symbol refutation is a kind of 38's (additive, 3.12) |
  | and-parallel `&`, the cubes | only premises sharing no unbound metavariable |
  | the net engine, the additive path, the Horn engine | refuse the bit (a first-order Horn engine is another engine, step 27's report) |

- **(g) The dispatch.** The rows of the focused engines name `LL1`; no
  other row contains the bit; `NoEngine` for none.
- **(h) Witnesses and eigenvariables in proofs.** `Node::Forall(Member,
  Eigen, NodeId)` and `Node::Exists(Member, FoTermId, NodeId)`, 16 bytes;
  `Proof::with_instances` owns the table (3.14: the members, the frames as
  cons cells, the proof-local extensions of terms, arguments and symbols,
  `atom_of(member)`, the display names: witnesses over the sequent's
  symbols and the proof's eigenvariables; an open metavariable allowed or replaced by
  a fresh constant, fo-linear Q3, the author's). The proof an engine
  returns is closed under its final substitution (R125). The checker:
  3.8's rules over members, `Ax` comparing the two instances by id,
  `Exists(m, w, p)` consuming the body member `(o + 1, F·w)` of the
  premise and concluding `m = (o, F)`, `Forall` the same with its
  eigenvariable, the eigenvariable set flowing up; new faults `Instance`,
  `Eigenvariable`, `Witness`; the oracle gets the same rules, and a
  first-order reference prover is written for the panel (R126).
- **(i) Views, sessions, exports.** The derivation prints instances
  through the frames and counts substituted characters in its size (R66);
  `Inference::datum` holds a `Binding` (the witness or the eigenvariable);
  the session holds an `Instances` table and bindings, `Step::witness`
  takes `Witness::{Open, Term { term: TermBuf }}`, a flat topological term in
  `fo_terms`' shape (text is the front end's, since `interactive` has no
  parser), or opens a metavariable that `close` or a later axiom binds,
  and `undo` undoes bindings (R97); the printers name bound variables from the side table,
  print `p(t, …)` through the one atom writer and terms through a term
  walk inside the atom's stop (R170); `rocq` answers
  `Unsupported::Quantifiers` until the library has binders (R161).
- **(j) Ordinary first-order logic.** `ordinary::Formulas` holds the same
  `fo` table, symbols and interned atoms as `Sequent` (one term type and
  one atom notion, 3.13), so `translate` copies atom and term ids;
  `ordinary::Node::{Forall, Exists}`;
  the pattern table gains its rows without touching the propositional ones
  (R67); the classical image puts `?` on every `∃` (fo-embeddings §1.5),
  so `Translation::target()` is `affine LL` and `Unknown` reaches the
  classical path; `Image::read_back` reads the proof's witnesses;
  `ordinary::Inference::datum`; the Rocq certificate is over a domain `D`
  with an inhabitant.
- **(k) Bounds and reasons.** `Limits::terms` on every reader (R250);
  first-order search bounds as `Options` fields with `DEFAULT_*`, flags
  and JSON keys, each reaching a `Reason` of its own; `Unprovable` only
  from a level that met none of them (R38). MLL1 and MALL1 are decided by
  the bounded search alone.
- **(l) What stays untouched for the propositional case** (D17): the
  occurrence numbering and every stored id; the sizes of `Term` (12),
  `Node` (16), `Kind` (1), `Member` (4), `ordinary::Node` (12); every
  propositional JSON byte; `Decide`, `Answer`, `Verdict`, `Task`'s
  propositional fields; `Mode`; the forest's arrays (the binder-depth
  array is empty without binders); `Context`, `OccSet`, the memo's
  records, `Classes`, `Counts` at their propositional instance; the
  checker's pass on a proof without instances; the target set's counters
  and, within a few percent, its pinned CPU time (rerun at every commit of
  38 that touches `search/` or `proofs/check.rs`).

## 11. The spike: what quantifiers cost the propositional case

### 11.1 What it answers

Two questions D17 leaves open, and nothing else:

- **Q1, the data model.** Do the reserved variants, `Member` as every
  node's operand and every member list's element, the empty first-order
  tables in `Sequent` and the quantifier bit cost the propositional case
  anything? Expected nothing (the size assertions say the layout is the
  same), but wider matches, larger `Sequent` clones and new arms in the
  checker's hot `match` are what a count shows and a reading does not.
- **Q2, the zone parameter (D-7).** Does a focused engine generic over its
  zone, monomorphised at today's `Context`, cost the propositional
  instance? Monomorphisation should give today's code; what it does not
  settle is the trait's hooks on the hot path (the trail's marks, the
  literal seam). D-7 concerns crate-private code only, so no public
  signature depends on its answer; the answer decides how step 38 builds
  its engine (generic, or D17's duplicated fast path).

It does not measure first-order speed, nor build unification, the
parser, the JSON, the instance checker or the session.

### 11.2 How it was run

In a jj workspace of its own (`spike`, at `../linlog-spike`) on the tree
of 2026-10-09, thrown away afterwards, nothing merged; by an agent (Opus
5.5 at `xhigh`) from draft C's specification (`plan/notes/api-drafts/
draft-c.md` §11), on cores 12 to 15. The base is the same tree
unmodified, built in the same directory with the same toolchain before
the spike's first edit: the journeys' instruction counts under callgrind
(the ratchet's twenty journeys) and the target set (`bench/targets.sh`,
pinned cores 2 and 3), whose counters equal the committed oracle
`after-coverability.csv` on all 225 decided rows. Each milestone is
measured against that base.

- **M1, the data model** (Q1): the drafts' four reserved `Term` variants
  (`Pred`, `DualPred`, `Forall`, `Exists`, a superset of section 3.2's
  two), `Kind::{Forall, Exists}`, `Node::{Cut, Forall, Exists}`, every
  exhaustive `match` extended (the readers refusing the new terms, the
  checker refusing the new nodes, every engine's `admits` refusing the
  bit), the size assertions, the empty first-order fields in `Sequent`
  (in its derived `Eq` and `Hash` and in the clone the forest keeps),
  `Member` as every node's operand and every member list's element, and
  the checker's branch per proof.
- **M2, the zone parameter, one instance** (Q2): `Context` and `Classes`
  lifted to `search/zone/`, the focused engine generic over `Z: Zone`
  with the trait of draft C §11.3 (members, the two zones, the literal
  seam, the trail's `mark`/`undo` called at 10.10 (e)'s places), `Ground`
  the only instance the front door calls, its methods `#[inline]`
  forwards to today's code.
- **M3, a second instance in the binary** (Q2's second half): a stub of
  real code (`Framed`: members through an instance table, duals matched
  by predicate and argument list, no unification, a memo for ground
  members only), reached from the dispatch only for the quantifier bit,
  with a unit test proving two ground first-order sequents through it.
- After the first results, **M1d**, **M2d**, **M1b** and **M3i** (11.5):
  the design's own data model, the zone on it, the drafts' variants
  reordered, and the second instance with its shared helpers inlined.

### 11.3 The gates

- **G1, exactness** (must hold, never traded): the target set's columns
  `verdict`, `nodes`, `splits`, `memo_hits`, `memo_entries` (and
  `links`, `tests`) of every decided row equal to the base's. A difference
  is a bug of the spike, fixed and measured again, never a cost.
- **G2, instructions** (the main gate; counts are deterministic and
  independent of the machine's load): each search journey (`search-*`,
  `batch-families`, `ordinary-pigeons`) at most **+1.0 %**, every other
  journey at most +2.0 % (the ratchet's tolerance), the sum of the search
  journeys at most **+0.5 %**. By part: `read-*` for the variants,
  `check-*` and `derivation-chain-64` for `Member` and the checker's
  branch, `search-*` for the zone.
- **G3, time**: not gated in the spike (the machine ran other agents);
  the target set's own CPU times are recorded as indicative, with the
  load. Step 37's lift runs it first for the zone (10.9, decision 20),
  and step 38 gates pinned time on its own commits (10.10 (l)).

### 11.4 What the results decide

| result | decision |
|---|---|
| M1 passes | the data model of 3.2 to 3.14 stands; step 28 lands its parts (`Member`, the renames, the assertions, the docs of `Fragment` and `Atom`); the variants come at 34 and 38 |
| M1 fails G1 or G2 | bisect its parts (variants, `Sequent` fields, `Member`, the checker's branch); the failing part is redesigned before 38 (an empty table behind one `Box` instead of several `Vec`s; binders' kinds in a side array of the forest instead of `Kind`), measured again, and recorded |
| M2 passes | **D-7 adopted** on counts: step 38 makes the focused engine generic over `Z: Zone`, `Ground` the propositional instance, after step 37's lift, whose pinned-time run (G3) confirms it or sends 38 to the next row (decision 20) |
| M2 fails | **D17's duplicated fast path**: the propositional engine stays as it is; step 38's framed engine is a sibling module that shares the zones, counts, bias, arena, scratch and classes but not the hot loop, the one exception to D7, recorded under D17's name |

### 11.5 The results

Run on 2026-10-09 (the spike's agent from 07:31; M1d, M2d, M1b and M3i
after a pause, M3i's counts and both last target sets by the session
from 10:35). The full report, with every journey and the disassembly
behind the first finding, is `plan/notes/api-drafts/spike-report.md`;
the numbers below are instruction counts under callgrind against the
base, each journey of the ratchet's twenty, and the target set's five
counters on its 225 decided rows.

| milestone | what it is | G1 | worst search journey | search journeys' sum | worst other journey | G2 |
|---|---|---|---|--:|---|---|
| M1 | the drafts' data model: `Term::{Pred, DualPred, Forall, Exists}` appended | exact | `search-chain-128` +6.63 % | +2.32 % | `read-text` +1.87 % | fails |
| M1b | M1 with the four literal variants first | exact | `search-partition-no-5` +0.16 % | −0.75 % | `read-text` +2.00 % | passes, `read-text` at the limit |
| **M1d** | **this design's data model**: `Term` gains only `Forall`, `Exists` (3.2); `Member`, the empty tables, the bit, the `Node` variants, the checker's branch as in M1 | exact | `batch-families` +0.03 % | −0.94 % | `read-tptp` +1.45 % | **passes** |
| M2 | the focused engine generic over `Z: Zone` on M1 | exact | `search-chain-128` +4.86 % (M1's part) | +1.54 % | `read-text` +2.23 % | fails through M1; its own delta −0.76 % |
| **M2d** | the same on M1d | exact | `search-partition-no-5` +0.23 % | −1.33 % | `read-tptp` +1.61 % | **passes** |
| M3 | a second, first-order instance (`Framed`) in the binary, on M2 | exact | `search-chain-128` +4.68 % | +1.96 % | `read-text` +1.89 % | fails; its own delta `search-qbf-20-2` +2.40 % |
| **M3i** | M3 on M2d, the shared hot helpers `#[inline]` | exact (223 rows decided in both) | `search-qbf-20-2` +0.94 % | −0.53 % | `read-tptp` +1.54 % | **passes**; against M2d alone up to +2.19 % on three journeys |

What it decides (11.4):

- **The data model of 3.2 to 3.14 stands** (M1d passes): `Member` as
  every operand, the empty first-order tables, the quantifier bit and the
  checker's branch per proof cost nothing measurable (the checker's
  journeys −0.24 %), and the two binder variants appended to `Term` cost
  nothing.
- **Decision 1 has a measured cost on its alternative.** The drafts'
  `Pred` and `DualPred` appended after `Quest` cost 6.6 % on the focused
  engine's journeys: `Term::atom()` then matches the tags {0, 1, 12, 13},
  which turns the forest's literal test from one comparison into a bit
  test with branches, and the engine calls it per literal in its hot loops
  (`meets`, `mark_literals`, `initial`, the counts). The same variants
  placed first (M1b) cost nothing, so the alternative is viable with that
  order; the interned atom (M1d) needs no such care.
- **A rule the spike adds to 3.2** [28]: the literal variants of `Term`
  and `Kind` are the first, contiguous, and a variant is added after the
  compound ones, never between; `Kind::is_literal` and `Term::atom()` stay
  one comparison, which a test of `Kind`'s discriminants (it is
  `repr(u8)`) pins. A size assertion cannot catch it, and the target
  set's times cannot either; the journeys did.
- **D-7 is adopted on instruction counts** (M2d and M3i pass G1 and G2),
  provisional on pinned time (G3 at step 37's lift, decision 20): step 38
  makes the focused engine generic over `Z: Zone`, `Ground` the
  propositional instance, after step 37's lift. One instance is free (M2 against M1 −0.76 %, M2d against M1d
  −0.39 %); a second instance costs through the shared helpers LLVM stops
  inlining into the first (`Context::iter`, `Key::assign`,
  `OccSet::clone`, `Split::clone_from`), not through its own code, and
  `#[inline]` on those helpers brings it within the gates (M3i).
  Against M2d alone the second instance still costs up to 2.2 % on three
  journeys (`qbf`, `wide`, `additive`), within the gates only because
  M1d's and M2d's layouts gained as much; step 38 measures its own
  commits against the gates (10.10 (l)), and its first remedy past them
  is the inlining of whatever helper its symbol diff shows outlined, then
  a `Framed` that shares none of them.
- **The zone's seam is wider than drafted** (M2), which 10.10 (e) takes:
  a handle per dual list (its mark and its "looked up in `Θ`" flag),
  `dual_from` with its cursors, the marks' bytes charged by the engine,
  `fork` for a pool's worker, `root`, and the forest passed per call (the
  trait has no lifetime, since a pool's workers outlive no spawner's
  borrow). **The memo and the loop check stay ground**: they key the
  ground part of a zone behind `Zone::memoizes`, so there is no `Key<Z>`.
- `admits`' scan for binders runs only when the bit is set, and the
  checker's refusal of a first-order forest lands with the first
  first-order search, as a refusal.
- **Code size**, reported, not gated: the release binary's text 2 338 670
  bytes in the base, 2 360 222 with M1, 2 361 022 with M2, 2 505 006 with
  M3 (of which 140 398 bytes name `Framed`).
- **Time**, indicative only (G3 not run; the machine ran other work): the
  target set's three rows over a second took 90.47 s of CPU in the base
  and between 90.45 s and 92.93 s in M1 to M2d and M1b (load 0.3 to 2.0);
  M3i's took 108.67 s, every `mix` row 12 to 21 % slower alike, with the
  load at 24.7 from another workflow's builds on the shared cores. A count
  the load cannot change decided it: `mix/8` under callgrind takes
  29 542 018 712 instructions in the base, 29 554 079 268 in M2d
  (+0.04 %) and 29 438 722 626 in M3i (−0.35 %), with equal counters. The
  zone's decision rests on the counts until G3 runs at step 37's lift
  (10.9): a count does not see the instruction cache, which a second
  monomorphised engine strains (M3's 166 KB more text, 7 %).

## 12. The walk-through

Ten agents (Sonnet 5.5 at `high`), one per step 29 to 38, each sketched
its step's first change against this note as committed in ab0b27e5 and
reported where it had to work around it (`plan/notes/api-drafts/walk-NN.md`).
Two blocking items, 70 friction items, 59 notes in all; every item is
answered below, most by a change of the sections above (marked with the
walk-through's number there), the rest by the step that owns it. Nothing
was set aside without a reason.

| step | blocking, friction, notes | what changed in this note | left to the step, with the reason |
|---|---|---|---|
| 29 | 0, 7, 6 | the public walks' signatures and `Reading::connective` (3.6); distinct atom keys (3.2); `Forest::BYTES_PER_OCCURRENCE` (3.3); `jobs: "auto"` (6.2); `checked` in the outcome (7.3); `name()`/`NAMES` on the verdict, reason and refutation (8.1); the race's statistics as today's sum (5.4); 10.1's checkable ground truth corrected; the clock's start (6.3) | R190's commit in the outcome (the harness records it; the crate version alone is in the outcome, decision 13); a peak held-bytes counter and `Engine::decides()` (additive) |
| 30 | 0, 6, 6 | the net-engine list before the retype (3.11); `serde(default)` on read-back forms (7.1); keywords and reserved words refused as atom names, the syntax's growth rule (3.1, 7.1); option-valued enums open and the missing types listed (2.5); promised derives (P3); `BROWSER` and the browser settings deferred to 32 (5.1, 6.5); exit statuses by kind (4.1); golden files and append-only name lists at 30 (7.5) | the closed core enums are a decision for the author (14.2, decision 18); R41's wasm32 check moves to 30 (10.2's list); the bump rule and the skipped semver check before the first publication are 30's text |
| 31 | 0, 9, 4 | `Node::NAMES` beside `TAGS` (3.7); the lemma's lexical check only, reserved names per kernel (9.2); the forest fixture as a data file (3.3); the position grammar normative in `Reading`'s docs (3.6); a refutation kind as an open enumeration (3.12, 7.1); `Form` open (2.5); `ordinary::Outcome` open, R154's certificate over the ordinary formula (3.13) | `Kernel::resolve` and one `rocq::certify` entry, `refute: Refute { Off, Unprovable, Always }` in place of a bool, the refuter's "gave up" answer, `Refutation::applies` and its fault type, the certificate's own compatibility rule: all additive at 31 and its own to shape |
| 32 | 0, 9, 9 | `Interactive::derivation` fallible and bounded, `close` returning `Closed` (3.10); a live worker, not a stateless one (3.10); `memo_limit: u32` (6.2); `Settings.ordinary` (6.5); the renames `Branch` and `Side` (2.4); `describe` through the owner (4.1); the unit of work's cost stated (5.2); the suspendable sentence corrected (5.3) | `wire::Over`, `Limits::within_stack`, `GoalView`'s signature with the text and view options, `explain`, `Settings::FIELDS`, an SVG id prefix, a `clippy.toml` list of the native-sized convenience calls for `linlog-web`: additive at 32 |
| 33 | 1, 5, 3 | **blocking**: every data-carrying `NetError` variant has named fields, so 33 adds the box (P3, 3.11); `NetError::{Fragment, Mode}` and its kind per variant (3.11); `has_nets()` for structures only (3.5); the vertex numbering rule (3.11); `describe` through the owner (4.1); a goal proof refused by `from_proof` (3.11); a structure over a refused fragment raises the level (7.1) | the staged `NetBuilder`, `instances(o)`, `isomorphic`: additive at 33 |
| 34 | 0, 6, 5 | the cut formulas reserved in `Sequent` itself, carried by every owner (3.1); the checker's `Cut` on any occurrence of the `A` tree (3.8); `Cut`'s principal `None` (3.7); `Goal` and `Task` naming the goal's trees (10.6); `Interactive::cut`'s formula and refusal (3.10) | `cut::step`/`eliminate` taking the mode and `eliminate` returning `Result`, the record's meaning, net cut links as a list of their own (`cut_links`, `erased`), `Proof::cut` composing two goal proofs, `is_cut_free`: 34's to write, all within the shapes fixed here |
| 35 | 0, 9, 6 | no essential field in `Criterion` (3.11); `is_correct` linear in memory (3.11); the symmetry groups keyed per criterion, an unsoundness otherwise (10.7); fallible reservation of superlinear tables (5.1); `NET_MULTIPLICITY` private (6.2); the essential search as a forceable variant (10.7); R115's rules (8.4) | the prune switches' wire form, `features()`, `Row`'s conjunction of features, the essential drawing's `Style` field: 35's, additive |
| 36 | 0, 7, 5 | classical cyclic needs a reversing dual in the reader: the ordered parse beside `parse_within` (3.1, 10.8, decision 19); planarity a type parameter (10.8); `Criterion` gains the order and the L restriction (10.8); `nonempty_antecedents` (3.5); `Reading::of_mode` (3.6); the `is_ordered()` guard at every entry (3.8); goals as sequences and rotations (8.1) | whether an ordered session's `close` closes only the initial goal or a goal-subset structure exists, `Needs::Cut`, the ordered clauses in 3.8's table: 36's, with its panel |
| 37 | 0, 4, 8 | counters appended later written only when not zero, `serde(default)` on `Statistics` (7.1, 8.5); `Engine::counters()` (8.3); the "complete" rule restated, no `Saturated` (10.9, 3.12); the lift list with the bias and the arena, `ClassMembers` (10.9); the read-off as a renaming (10.9); a default bound in `InverseOptions` (10.9); `Engine::modes()` (8.3); one notion of class (10.9) | the dispatch row's feature (it may read the written sides for routing, 8.4) |
| 38 | 1, 8, 7 | **blocking**: one distinct key per atom, `atom_name` the key (3.2); open atoms guarded by the binder bit, stated and tested (3.2); `Instances`' extensions, `atom_of`, the checker's own overlay (3.14, 3.8); frames as cons cells (3.14); `describe` through the owner (4.1); members across owners (3.4); `Witness` and `Binding` (10.10); appended counters (7.1); the ordinary layer's atoms the same interned table (3.13); one namespace of symbols, the tables' emptiness invariant (3.14) | `Unbalanced` per symbol for open atoms, a first-order net criterion, the fragment names' spelling with two bits left: 38's, additive |

**What fits well**, in the walkers' words: the member as one integer by
offset (31, 34, 38), the proof that records its goal and mode (31, 34),
`Disproof` (29, 31), the closed core enums with planned bumps (30, 34),
the wire level (29, 30, 32), the error family with a refusal variant in
every type (29, 32), `Settings` and `Clock` (29, 32), the engine
registration list and the dispatch as data (35, 37), and the first-order
plan's split into a data-model commit and layers (38).

**The fresh review** (Fable 5.1 at `high`,
`plan/notes/api-drafts/review-fable.md`) found nothing blocking and ten
items, each answered above: (1) the fourth lock commit, the stop's shim
and `checked` (7.5, 5.2, 8.1, 8.5); (2) the race at two threads and
below (5.4); (3) no reader of the pre-release names, required level-1
keys (7.1, 2.4, 3.1, 3.5, 14.1); (4) the reserved words refused by
every reader (3.1, 7.1); (5) the counts per symbol for a symbol with an
open atom (3.2, 10.10 (f), decision 1); (6) D-7 provisional on pinned
time at 37's lift (10.9, 10.10 (e), 11.3 to 11.5, decision 20); (7)
named fields and P3's two exceptions (P3, 3.6, 3.8, 3.10, 4.1, 6.2, 8.6,
10.8, 10.10 (i)); (8) the mismatches (2.1, 2.2, 2.4, 2.5, 3.9, 5.1, 5.2,
8.1, 8.5); (9) `GoalProof` of kind `unsupported` (3.7, 4.4); (10) the
tests of H9 and H10, which the supervisor's check of the answer turned
into decision 21: with the sides unknown the reading answers only where
it is the one reading (3.6).

## 13. Findings and requirements answered

| finding | where |
|---|---|
| F1 `#[non_exhaustive]`; F84, F85 sizes and wildcards | P3, 2.5, 3.2 |
| F4, F38 option values unchecked; F5 sequent drawing unbounded | 6.4, 9.1 |
| F6 families panic | 2.4 |
| F7 `Mode`; F40, F64 builder names, `must_use` | 3.5, 6.1 |
| F8, F30, F31, F33, F34, F48, F50 the error family | 4 |
| F10 bools for a mode and a criterion | 3.9, 3.11 |
| F12, F21 stops and linear readers on long calls; R22 | 5.6 |
| F13 ordinary constructors | 3.13 |
| F15 Rocq prelude | 6.4, 9.2 |
| F16, F27 readers before or without the bound | 3.1, 3.3, 5.5 |
| F18, F61 forms undocumented, unknown keys three ways | 7.1, 7.2 |
| F19 the member type | 3.4 |
| F22 `apply`'s arguments; F67, C3 an empty Mix premise | 3.10 |
| F23 `close_with` grafts a foreign proof | 3.10 |
| F24, C1, H9, H10 the written order and sides | 3.1, 3.6 |
| F25 atoms called variables | 3.2 |
| F26, T1 versions and key names | 7.1 |
| F28 where nets exist | 3.5 |
| F36 the export wrappers; F82 `Notation` | 9.1 |
| F42 the command's recipe, two defaults | 6.2, 6.5 |
| F49 `from_proof`'s wildcard | 3.11 |
| F51 needless `pub`; F203, R253 `linlog_cli` | 2.3 |
| F53 the ordinary forms | 3.13, 7.3 |
| F56, F113 names that differ between neighbours | 2.4, 8.8 |
| F58 axiom partners in eight places | 3.3 |
| F60 `Rule` as a rule and a side | 3.9 |
| F62, F63 three bounds values, several names | 5.1 |
| F65, F70 the lower bound in the refusal, a saturated `Size` | 3.9, 7.1 |
| F66 the reading recomputed; F68, R92 `close_all`; F69 two numberings | 3.6, 3.10, 3.9 |
| F73, F74, R40 the batch's workers and cancel | 6.3 |
| F76 to F79 names and forms of the options | 3.5, 6, 8.3 |
| F80, F86, F87, F88 refutation and reason forms, the statistics' proxy | 3.12, 7.3, 8.5 |
| F81 the test period, the time limit | 6.2, 6.3 |
| F83 `verify_integrity` | 2.3 |
| F89 an outcome of a goal | 3.7, 7.3 |
| F90 the serialize tests pinning twice | 7.5 |
| F103, F104, F168, R85, R18 the race and its memory | 5.4 |
| F110, R47 the default bias's scheme | 5.3 |
| F136, R243, R21 the stop and the work budget | 5.2, 5.3 |
| F139, R138 a proof found and not checked | 4.1, 8.1 |
| F140, F141 `NoEngine`, `Engine::parallel` | 8.1, 8.3, 8.6 |
| F144, T5 statistics | 8.5 |
| F164 the batch's records | 7.3 |
| H8, HD3 identifiers by code point | 3.1 |
| H17 a crafted session history | 3.10 |
| H18 the pool's stack | 5.4 |
| H19, HD5 atom names; HD1 several conjectures | 3.1, 4.4 |
| H2 to H5, H16, HD2, HD4 the readers' and `interact`'s behaviour | 14 (the readers and the command areas implement the answers) |

The search's internal findings, the tests the audit asks for (F11, F17,
F55, F57, F71, F93 to F98) and the command's and the harness's findings
are the fix areas'; this design moves nothing they pin.

**The register.** Every entry of the sections Wire forms, Bounds and
stops, Data model, Engine interface, Proof term and checker, Errors,
Options and Export is met at 28 or placed with its step, by section:
R1 6.2, 7.3 · R2, R3 3.12, 7.3, 8.1 · R4 3.9, 7.3 · R5 3.10, 7.3 · R6
7.1 · R7 3.7 · R8 7.3, 10.5 · R9 8.5 · R10 3.5 · R11 3.13, 7.3 · R12
9.2 · R13, R14 7.3, 10.6 · R15 10.6 · R16, R17 3.14, 7.3, 10.10 · R247
7.1 · R18 5.4 · R19 5.4 · R20 8.7, 10.3 · R21 5.3 · R22, R23 5.6, 3.8
· R24 3.13 · R25 3.11 · R26 9.1 · R27 5.5 · R28, R29 5.1, 3.3 · R30
5.2 · R31, R32 3.8, 10.6 · R33 to R35 10.7 · R36, R37 10.9 · R38, R39,
R250 3.14, 10.10 · R40 6.3 · R243 5.2 · R41, R42 10.4, P9 · R43 P9 ·
R44 9.1 · R45 5.1, 5.4 · R46 8.5, 10.4 · R47 5.3 · R48 6.3 · R49 6.2,
6.4 · R50 2.5 · R51 3.5 · R52 3.5, 3.2 (amended: decision 1) · R53, R54
3.1, 3.6 · R55, R56 10.8 · R57, R58 3.3, 10.6 · R59 3.1 · R60, R61 3.9,
10.6, 10.8 · R62 to R67 3.2, 3.14, 10.10 · R68, R69 3.3, 9.2 · R70,
R71 3.12 · R72 8.7 · R73 6.3 · R74, R75 3.1, 10.1 · R76 to R78 10.5 ·
R79 3.11 · R80, R81 10.6, 10.7 · R82 to R84 10.7, 10.9 · R241 3.5 ·
R244 3.4 · R245 3.6, 10.1 · R248 P3 · R249 3.13 · R251 3.2 · R85 5.4 ·
R86 to R88 8.3, 8.4 · R89, R90 10.5, 10.8 · R91, R93, R94 3.10 · R95
to R97 3.10, 10.6, 10.8, 10.10 · R98 to R105 10.5 to 10.8 · R106 to
R109 10.10, 8.2 · R110, R111 8.3, 10.9 · R112 8.2 · R113 3.13 · R114
6.2 · R115 8.4, 10.7 · R116 6.2 · R117 to R128, R246, R254 3.7, 3.8,
3.11, 10 · R129 to R136 4 · R137 8.1 · R138 4.1 · R139, R140, R141 6.4,
9.2 · R142 5.1, 6.5 · R143 to R147 6.2, 10.7, 10.9 · R148 6.3 · R149 6.2
· R150 6.3 · R151 5.3 · R152 to R171, R242 9, 10 · R183 6.6 · R190 7.3.
What section 12's walk-through found missing is answered there.

## 14. Decisions, answered by the author

Answered by the author on 2026-10-09, each by the author's own choice:
the recommended answer to every question and decision below, except
decision 4, taken with a converter added, and T3, changed to link-time
optimisation now. Each row keeps what the other answer would have
changed. The answers are rules under `.claude/rules/`, which the fixes
implement and later rounds judge against.

### 14.1 The open questions, and what another answer would have changed

| # | answered by the author, 2026-10-09 | what the other answer would have changed |
|---|---|---|
| C1 | the written order canonical at 28 | B (at 36): `optimize` sorts until 36, which then breaks every stored id as a level of its own; `antecedents` lands anyway (H9, H10 need it). C (mode-dependent): ruled out by 3.1, a `Sequent` would mean two things. |
| C2 | refuters in the library, after the search, `refute_unknown` off | A (inside the Rocq exporter): no refuter kind, no `Refutation::Classical`, the classical search in `rocq`; no front end shows the assignment. B (after an Unprovable only): no `refute_unknown`. |
| C3 | no nullary Mix | B: a `Node` variant and its tag before 31 (cheaper before 0.1.0 than at a bump), a checker arm, a Rocq constructor; `StepError::EmptyPremise` becomes closable. |
| T1 | rename the keys with the version; no reader keeps the old names (D18, the author's standing rule) | keep `ids`, `var_dict`, `proof`; only `version`, the tagged reasons and the new keys change, and fewer fixtures change in commit (1). |
| T2 | owned forests | `Arc<Forest>` inside `Proof`, `Disproof`, `ProofStructure`, `Interactive`: no wire change, cheaper clones in the bindings; signatures keep their shape. |
| T3 | **link-time optimisation now**, not after a measurement (the author's change of the recommendation): `lto = "fat"` and `codegen-units = 1` in `[profile.release]`, a commit of its own early in area 3.1 that re-records the ratchet's ceilings, saying why (the profile moves every count), and notes the gate's longer release build. The spike's counts (11.5) were taken without it, so the gates of 37 and 38 measure under it | the efficiency area measures it on the journeys and pinned time and adopts it only if the gain holds |
| T4, T6, T7 | as the audit recommends | nothing here (R130's no-panic rule stands either way). |
| T5 | shared counters, documented per engine | per-engine counters: `statistics` an object per engine, the CSV columns change, a level of the outcome. |
| HD1 | refuse several conjectures | conjoin them; no `SeveralConjectures`. |
| HD2 | a `.spec` file is affine | refuse it without `--affine`: an error of kind `unsupported`. |
| HD3 | NFC when reading | code-point identity, stated in the syntax's docs; no dependency. |
| HD4 | `load` refuses another sequent or mode | the command's concern only. |
| HD5 | refuse a non-identifier atom name | print such names quoted; no `AtomName`. |

### 14.2 The decisions this design adds

| # | decision, answered by the author, 2026-10-09 | set aside | why |
|---|---|---|---|
| 1 | **An atom is an interned atomic formula** (3.2): `Term` gains only `Forall`/`Exists` at 38, ground first-order input is propositional by construction; where a symbol has an open atom, the pairing sites are guarded by the binder bit and the counts go per symbol (3.2, 10.10 (f)), guards and not construction | the research's `Term::{Pred, DualPred}(Atom, ArgsId)` with a fragment bit set by any argument (D-4, R52, R62, R16, all three drafts) | eight pairing sites correct for ground atoms by construction instead of by guard; fewer variants; ground problems decided at once (both judges chose it); measured free (the spike's M1d), where the alternative costs 6.6 % unless its literal variants come first (M1, M1b). Amends R52, R62, R16 and D-4's wording |
| 2 | **The written sides decide the intuitionistic goal** (3.1, 3.6): `antecedents`, one written succedent, no symmetric reading when the sides are known | "the goal is the last root" (refuses neither H9 nor `|- top, a`); refusing every ambiguous `⊤`/`0` input | the only rule that refuses both witnesses; costs one-sided intuitionistic text with several roots, which is refused with advice |
| 3 | **The stop is a closure over `Progress`**, every `|| false` becoming `|_| false` (5.2); the author wants a progress value for front ends and left its form to this recommendation | a `Stop` trait with a blanket impl for `FnMut() -> bool` and a wrapper | one form, inference works (the trait's wrapper does not infer, a judge's probe); D18 allows the break |
| 4 | **One global wire level** (7.1), **and a converter** (the author's addition): from the first version bump on, linlog converts a document of an older released level to the current one where it can, through one entry, `wire::upgrade`, the identity at level 1, each later level adding its step | a version per form; no converter, an older document read only as it was written | one number a client stores; an outcome read as a proof carries one; a stored file outlives the release that wrote it |
| 5 | **A mode on the wire is its name** (3.5) | the object of flags with keys added at 36 | 36's modes need no key and cannot be misread as commutative (R10) |
| 6 | **The refuter kind arrives at 31** with its first new member (8.7) | the kind from 28 with the counts as its first member (draft C) | C2's text; no second member before 31 |
| 7 | **`Verdict::Unprovable(Box<Disproof>)`** (3.12) | `Unprovable(Refutation)`, callers building a disproof | `Verdict` is closed: now or a bump; 31's checker and certificate need the sequent and mode with the refutation; one arena clone per unprovable outcome |
| 8 | **A proof records its goal and claimed mode** (3.7) | a separate goal-proof type; the mode only in the outcome | F89, F23 and R7 at the type; `check` checks a term against its own conclusion |
| 9 | **`Rule` is a one-sided rule and `Named` adds the side** (3.9) | the 34 variants | 38 adds two rules instead of six, 36's divisions are a field, the step's own item (F60) |
| 10 | **Seven error kinds**, a refusal variant in every specific type (4) | three classes; one flat enum without specific types | the harness and the exit statuses need malformed, invalid, unsupported, limit, stopped, failed and defect apart (R135, R138) |
| 11 | **`Limits` is an argument of every long call** (5.1), `Settings` holds it beside `search::Options` | a field of `search::Options` | one rule for every long call (a check's bounds are not a search's options) |
| 12 | **`Settings::default()` is the command's behaviour** (no copy bound, 2 s), `search::Options::default()` keeps the copy bound of 3 (6.5) | one default | a library call without a stop must end; a front end with a clock deepens |
| 13 | **An outcome names the crate's version, not its options** (7.3) | the options and limits in every outcome | a batch's records stay short; the front end records its settings once |
| 14 | **Written-only counts saturate at `u64::MAX`, read-back integers stay below 2⁵³** (7.1) | 2⁵³ as the saturation value | R247's recommended form; JavaScript reads `u64::MAX` as 2⁶⁴, which no exact count takes |
| 15 | **Dispatch thresholds are not options** (6.2) | an option per row's threshold (R149 read literally) | a front end must not move a row the library's measurement placed; `engine` is the knob |
| 16 | **The builder and the browser presets wait for their first caller** (3.1, 5.1, 6.5), additive | the builder at 28 (two drafts); `BROWSER` at 28 (draft A) | nothing at 28 calls them, and 0.1.0 should promise no unmeasured numbers; adding them later breaks nothing |
| 17 | **The Lambek calculus's planar order is 36's panel's** (10.8): the reversing parse, or the order derived from D1's lowering through the reading | a reversing dual in every mode | D1 stands either way; no stored form changes |
| 18 | **`Term`, `Kind`, `Node`, `Rule` stay closed** (P3), each new variant a planned 0.y bump (0.2.0 at 34, 0.3.0 at 38) | `#[non_exhaustive]` on them, which makes 34 and 38 additive for downstream crates but forces wildcard arms there that silently mishandle a cut or a binder | a downstream `match` should fail to compile when the calculus grows; inside the crate the lint keeps them exhaustive either way (walk-through 30 asked for the author's word) |
| 19 | **Classical cyclic MLL is read with a reversing dual** by an ordered parse beside `parse_within` (3.1, 10.8) | the order derived from D1's lowering for every ordered mode | a classical cyclic sequent has no reading to derive an order from; with the order-keeping dual `|- ~(a * b), a, b` would be unprovable in cyclic mode, a wrong answer (walk-through 36) |
| 20 | **The focused engine becomes generic over its zone at 38** (D-7; 10.10 (e), 11.5): adopted on instruction counts, provisional on pinned time, which step 37's lift measures first (G3, on an idle machine; 10.9) | D17's duplicated fast path, a sibling module for the framed zone, which 38 takes if G3 fails | instruction counts: one instance free, a second within the gates once the shared helpers are inlined (M2d, M3i). D-7 and D17 ask for pinned time within a few percent too, and callgrind does not see the instruction cache a second monomorphised engine strains (M3 adds 166 KB, 7 % of the text); nothing public rests on it |
| 21 | **With the sides unknown, the reading answers only where it is the one reading** (3.6): a JSON sequent without `antecedents`, or one built in code without sides, has as its goal the one root that can be it, and an implication's antecedent is the left factor as with the sides known; two roots that can each be the goal (formulas of `⊤` and `0`) are refused (`ShapeError::Undetermined`) with a message that asks for the sides | today's guess (the last root that can be the goal, the symmetric reading), which keeps H9's and H10's wrong answers reachable through JSON and the library; refusing every intuitionistic sequent without sides, which guesses nothing either and costs nothing found, but refuses a hand-written JSON sequent that has one reading | closes H9 and H10 on every input, not only text. Its cost, checked: no lock entry, fixture, test or README block moves beyond commit (2) of 7.5, since every `-i` input there is text, and the JSON that `check -i` reads in the tests is written by `prove -i` from text, with the key; LLTP, the families and `ordinary::translate` give the sides; `Sequent::add` has no caller outside its own tests; `Interactive`, `Derivation`, the checker, the oracle and the Horn engine choose nothing of their own |
