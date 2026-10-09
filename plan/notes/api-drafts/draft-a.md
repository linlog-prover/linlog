# linlog's API after step 28 (draft A: the web client and the wire forms first)

One of three drafts of `plan/notes/api.md`. It designs the whole public
surface of `linlog` after step 28 and is most detailed where a front end
that is not this repository meets the library: the web client of step 32
(wasm in a worker, page-held JSON, a stop that carries progress, bounded
memory, no clock), the drivers of step 29, the promises of step 30, the
batch mode, and the command as one caller among several (D15). It starts
from the recommended answers of C1–C3, T1–T7 and HD1–HD5 (section 13
says what each other answer changes). Code claims were checked against
the tree of 2026-10-09; "(unverified)" marks the rest. A probe
(release, x86-64) measured `Error` at 112 bytes, `Outcome` 472, `Term`
12, `Node` 16, and showed that `u64::MAX` written by serde_json reads
back as a double that no longer parses as a `u64`.

A *form* is a JSON shape the library writes or reads; a *front end* is a
caller with a clock and a user (the command, `linlog-web`, the harness,
a notebook); a *long call* is a public call whose time or memory can
grow beyond linear in what the caller already holds.

## 1. Principles

**P1. What crosses a boundary is plain data with one wire form**, one
serde implementation behind `serialize`; the command, the harness and
`linlog-web` keep no name table or mirror struct. *Why*: every user of
the usability baseline rebuilt a table the command had (friction 1),
and the copies drifted (F76).

**P2. Owned values, borrowed views.** `Sequent`, `Forest`, `Proof`,
`ProofStructure`, `Interactive` own their data (T2); `Derivation<'a>`
and `Reading<'a>` borrow. *Why*: an owned value is serialized, sent to a
worker and dropped without lifetimes in the bindings.

**P3. One error family**: every public fallible call returns
`linlog::Error`, with a stable `code()`, a `class()` that tells a fault
from a refusal, and the setting that lifts a limit. *Why*: S18, F8,
F30, AIP-193; a refusal must never read as "invalid" (S4).

**P4. One bounds value and one stop on every long call.** `Bounds` holds
every resource bound the library enforces; every long call takes
`&Bounds` and `stop: impl FnMut(Progress) -> bool`, told the work done
since the last poll. The library reads no clock. *Why*: F12, F62, F136,
R22, R243; a wasm worker has no timer thread.

**P5. Options are data; presets are values** (`Bounds::BROWSER`,
`Settings::browser()`). What only a clock can enforce (the time limit,
when to add a pool) is data too, in `Settings::clock`, applied by the
front end's stop. *Why*: D15, D16, F76–F79, R141, R142, R148.

**P6. Room by marking, closed by intent.** Every public enum and
public-field struct a planned step extends is `#[non_exhaustive]`,
struct-like variants included; closed on purpose (a downstream match
must fail when they grow) are `Term`, `Kind`, `Node`, `Rule`,
`ordinary::{Node, Rule, Side}`, `Verdict`, `Side`, `Sign`, `Polarity`,
`Position`, `Compact`, `Form`, `Cores`, `pdf::Date`, their growth
planned 0.y bumps in the changelog (0.2.0 at 34, 0.3.0 at 38; R50, F1).
No match over a closed enum in the crate has a wildcard arm (R248, D-11).

**P7. A form says the lowest reader level it needs.** Every top-level
document starts with `"version": n`; a propositional cut-free value
writes 1 in every later release; a reader refuses a higher level by name
before reading on; keys are added, never renamed or retyped within a
level (7.1). *Why*: F26, R6, AIP-180, D-3; stored files and web
sessions outlive releases.

**P8. Every number on the wire is exact in JavaScript**: integers read
back are below 2⁵³, `null` is "no bound", an outgoing count that
saturates is written as 2⁵³ (7.1, R247).

**P9. Nothing recurses over input** (`core.md`): formulas, terms,
proofs, derivations, nets, sessions; a wasm stack overflow loses the
instance. **P10. The propositional case pays nothing** (D17): every
reservation is an empty table, a key skipped when empty or a
monomorphised parameter (section 11 measures the one that is not free
by construction).

## 2. The public surface after step 28

### 2.1 The module tree

```text
linlog                      the re-exports of 2.2
├── bounds        (new)     Bounds, Progress, Phase
├── settings      (new)     Settings, Clock
├── wire          (new, serialize) VERSION, SATURATED, ReadWithin, from_json_within, to_json
├── sequents                Sequent, Builder (new), Term, Kind, Atom, TermId, Formula
│   └── fmt                 Walk, Visit (public, R245), Notation (moved from export, F82)
├── fragment                Fragment, Mode
├── occurrences             Forest, OccId, Member (new), Sign, Polarity, Position,
│                           Reading, IllFormula, ShapeError        (OccSet: pub(crate), F51)
├── proofs                  Proof, Node, NodeId, Side, CheckError, Problem, Dyadic, Described
│   ├── derivation          Derivation, Inference, InfId, Rule, UnknownRule, ViewOptions, Compact, Sides
│   ├── size                Size
│   ├── style               Labels, OpenGoal, TextOptions
│   └── interactive (feat.) Interactive, GoalId, Step, StepError, Applicable, Needs,
│                           GoalView, FormulaView; Request, Response (step 32)
├── nets                    ProofStructure, Criterion (new), NetError, Scratch
├── search                  prove, prove_within, prove_goal, engine_for, race (parallel),
│   │                       Options, Schedule, Cadence, Engine, Bias, Outcome, Verdict,
│   │                       Reason, Refutation, AtomWeight, ClauseWeight, Statistics, Pool
│   └── batch               Options, Cores, Plan, Problem, Answer, Results, Cancel,
│                           prove, run, run_local
├── export                  Styles (from the command), Form, Drawable (sealed);
│                           latex, typst, svg (Style, Font, Advances), rocq, png, pdf
├── ordinary                Sequent, Formulas, Node, …, Image, Derivation, Outcome (new),
│                           Verdict (new), translate, decide (new), read_tptp
└── lltp, mist, families    (parse)
```

`errors`, `parse`, `serialize`, `hash` stay private; `wire` is the
public face of `serialize`, its documentation the schema of record
(section 7).

### 2.2 What `lib.rs` re-exports

```rust
pub use bounds::{Bounds, Phase, Progress};
pub use errors::{Class, Error};
#[cfg(feature = "parse")] pub use errors::ParseError;
pub use fragment::{Fragment, Mode};
pub use nets::{Criterion, NetError, ProofStructure};
pub use occurrences::{Forest, Member, OccId, Position, Reading, ShapeError, Sign};
pub use proofs::{CheckError, Derivation, InfId, Inference, Node, NodeId, Proof, Rule, Side, Size, ViewOptions};
#[cfg(feature = "interactive")] pub use proofs::interactive::{GoalId, Interactive, Step, StepError};
pub use search::{Bias, Engine, Options, Outcome, Reason, Refutation, Statistics, Verdict,
                 engine_for, prove, prove_goal, prove_within};
pub use sequents::{Atom, Formula, Kind, Sequent, Term, TermId};
pub use settings::{Clock, Settings};
```

No longer at the root: `Scratch`, `Polarity`, `Compact`, `Labels`,
`OpenGoal`, `TextOptions` (under their modules), `OccSet`,
`DEFAULT_MEMORY_LIMIT`, `ViewError`, `WriteError`, `Refusal` (gone).

### 2.3 What becomes private

`OccSet` and its iterator (no public call takes one, F51);
`Sequent::verify_integrity` (the readers call it);
`proofs::check::{check, check_within}` (the methods are the entry,
F51); `Forest::from_owned` (unverified that no front end needs it).
`linlog_cli` is an internal library: modules `pub(crate)` but `run`,
a sentence saying it carries no semver promise, and
`cargo-semver-checks` naming `linlog` only (R253, F203).

### 2.4 Before → after

A fix session searches this table by the old name. "→ Error" means the
type is gone and its variants are `linlog::Error`'s (4.3 maps each).

| before | after | why |
|---|---|---|
| `Error` (35 variants, positional, not marked) | `#[non_exhaustive]`, named fields, `code()`, `class()`, `setting()`, `describe()`, `Serialize` | F1 F8 F30 F31 |
| `Error::SequentParsing(Vec<ParseError>)`, `Mist(String)`, `Lltp/Tptp(String)` | `Parse(Box<ParseError>)` (one per parse), `Spec { message }`, `Lltp/Tptp { message }` | F30 F31 |
| `Error::InvalidVariableIndex`, `{Term,Node,Occurrence}IndexOutOfBounds`, `{Subterm,Premise}IndexNotDecreasing` (all `(usize, usize)`) | `AtomIndex`, `TermIndex`, `NodeIndex`, `OccurrenceIndex { index, len }`, `SubtermOrder { term, subterm }`, `PremiseOrder { node, premise }` | F25 F31 |
| `Error::Unchecked` from a search | `Verdict::Unknown(Reason::Unchecked { limit_bytes })` | F139 R138 |
| `Error::Unchecked` elsewhere, `Problem::Memory`, `ViewError::Memory`, `RenderError::Memory` (as errors) | `Error::MemoryLimit { phase, limit_bytes, needed_bytes }` | S18 F63 |
| `Error::View`, `ViewError`, `WriteError`, `RenderError`, `svg::TooLarge` | → Error (`TooLarge`, `MemoryLimit`, `TooManyInferences`, `Stopped`, `WriteFailed`, `NoCertificate`, `NotSvg`, `TooManyPixels`, `NoDate`, `RenderFailed`) | F8 F36 |
| `Error::Refused`, `proofs::Refusal` | `Error::Step`, `interactive::StepError` | A4 ("refusal" means a bound here) |
| `Error::{InconsistentState, OpenGoals, GoalOutputs, Succedents, ThreadPool}` positional | named: `InconsistentSession { reason }`, `{ count }`, `{ outputs }`, `{ threads, message }` | F31 |
| — | `Error::{Malformed, UnsupportedVersion, AtomName, SeveralConjectures, GoalProof, ForeignProof, FamilySize, InvalidOption, FeatureOff, Stopped}` | F26 HD5 HD1 F89 F23 F6 F38 F4 |
| `CheckError::describe`, `NetError::describe`, `ShapeError::describe`; two `Described`, `DescribedShape` | `Error::describe(&forest) -> proofs::Described<'_>` | F48 F50 |
| `ParseError` (`Debug`, byte span) | `+ Clone, PartialEq, Eq, std::error::Error`, `span_utf16`, `line`, `column` | F34 R129 |
| `DEFAULT_MEMORY_LIMIT` (root, `proofs`, `Options`), `png/pdf::Options::DEFAULT_MEMORY`; `Options::DEFAULT_OCCURRENCE_LIMIT`, `Forest::DEFAULT_LIMIT`; `ViewOptions::DEFAULT_LIMIT`; `Options::DEFAULT_RECURSION_LIMIT`, `Options::stack_size()` | `Bounds::{DEFAULT_MEMORY_BYTES, DEFAULT_OCCURRENCES, DEFAULT_DERIVATION_BYTES, DEFAULT_RECURSION_DEPTH}`, `Bounds::stack_bytes()`, `Bounds::recursion_depth_for_stack()` | F62 F63 R45 |
| `Options::{memory_limit, occurrence_limit, recursion_limit}`; `ViewOptions::{limit, memory, UNBOUNDED}` | fields of `Bounds`; `ViewOptions { compact, sides }` | F62 F10 |
| `Options` (private fields, setters, `job_count`) | public fields, `#[non_exhaustive]`, `with_*`, serde | F77 F78 R1 |
| `Options::test_period(Option<u32>)` | `test_period: Cadence` (`Auto`, `Every(n)`), `DEFAULT_TEST_PERIOD`, `SMALL_STRUCTURE` | F79 F81 |
| — | `Options::schedule: Schedule` | R47 |
| `prove_until(&s, mode, &o, FnMut() -> bool)` | `prove_within(&s, mode, &o, &bounds, FnMut(Progress) -> bool)` | F136 R243 |
| `prove_goal(forest, &[OccId], mode, &o, stop)`, `engine_for(forest, &[OccId], …)` | `&[Member]`; `prove_goal` takes `&bounds` too | F19 |
| `alone_first` in `cli/src/prove.rs` and `bench/src/run.rs` | `search::race` (`parallel`) | F103 F168 R85 R18 |
| `Reason::{RecursionLimit, CopyBound(u32), MemoryLimit(u64)}` | `{ depth }`, `{ copies }`, `{ limit_bytes }`; `+ WorkLimit { limit }`, `Unchecked { limit_bytes }`, `Reason::setting()` | F86 R21 R138 |
| `Refutation::Unbalanced { atom: Atom, name, .. }`; `StateEquation { weights: Vec<(String, i64)>, once }` | `Unbalanced { atom: String, .. }`; `StateEquation { weights: Vec<AtomWeight>, clauses: Vec<ClauseWeight> }` | F80 R70 T1 |
| `Statistics` (7 fields, `memo_entries: usize`, private `add`) | `+ forward_copies, work`, `memo_entries: u64`, public `add`, serde derive | F88 F144 R9 R46 |
| `Outcome { verdict, fragment, mode, engine, statistics, net }` | `+ goal, options, bounds`; `net: Option<Box<ProofStructure>>` | F89 R190 |
| `Mode { pub 3 bools }`, `affine()`, `with_mix()` | `#[non_exhaustive]`; `with_intuitionistic/with_affine/with_mix`; `name`, `NAMES`, `FromStr`, `validate` | F7 F40 R51 R241 |
| `Fragment`, `Engine`, `Bias` (no reading of names) | `FromStr` (+ `Fragment::NAMED`, `Engine::ALL`, `Bias::ALL`), serde both ways | F76 R86 |
| `Term/Kind::{Var, DualVar}`, `Sign::{Var, DualVar}` | `Term/Kind::{Atom, DualAtom}`, `Sign::{Atom, Dual}` (order kept) | F25 R251 |
| `Forest::within(&s, u64)`; `Sequent: FromStr`; `lltp::read`; `mist::read_within(text, u64)` | `Forest::within(&s, &Bounds)`; `+ Sequent::parse_within`, `Sequent::builder()`; `+ lltp::read_within`; `mist::read_within(text, &Bounds)` | F16 F27 R59 |
| `Proof::check_within(mode, Option<u64>)` | `check_within(mode, &bounds, stop)` | R23 F12 |
| `Proof::{derivation, two_sided_derivation, derivation_with, two_sided_derivation_with, derivation_size(bool), derivation_size_within(bool, Option<u64>)}` | `derivation(mode)`, `derivation_within(mode, &view, &bounds, stop)`, `derivation_size(mode, &view)`, `derivation_size_within(mode, &view, &bounds, stop)` | F10, story 6 |
| `Inference { sequent: Vec<OccId>, .. }`, `Dyadic`, `CheckError` (public fields) | `#[non_exhaustive]`, `Vec<Member>` | F1 F19 |
| `Interactive::apply(goal, position, rule, left)`, `rules -> Vec<Rule>`, `InfId` for session ids | `apply(GoalId, &Step)`, `rules -> Vec<Applicable>`, `GoalId` | F22 F69 R91 |
| `Interactive::{close, close_with}(.., view, stop)`, `close_all -> Result<Vec<(InfId, Outcome)>>`, `proof()` | `+ &bounds`; `close_all -> Vec<(GoalId, Result<Outcome, Error>)>`; `proof(&bounds, stop)`; `close_with` refuses a foreign proof | F23 F68 |
| — | `Interactive::{within, view, views}` | R27 R93 |
| `ProofStructure::{new, from_links}(forest, mix: bool, ..)`, `mix()`, `from_proof(&p, mix)`, `is_correct()`, `sequentialize()` | `(forest, Criterion, ..)`, `criterion()`, `from_proof(&p, Criterion, &bounds, stop)`, `is_correct(stop)`, `sequentialize(&bounds, stop)` | F10 F49 F12 R25 R79 |
| `NetError`, `ShapeError` | `#[non_exhaustive]`, named fields | F1 R132 |
| `{latex, typst, svg}::{ordinary, derivation, write}` per type; `{latex, typst, svg}::{sequent, two_sided}` | `write/derivation(&impl Drawable, ..)`; `sequent(&s, mode, &options, &bounds) -> Result<String, Error>` | F36 F5 |
| `svg::net(&n, &style, Option<u64>) -> Result<String, TooLarge>` | `svg::net(&n, &style, &bounds, stop) -> Result<String, Error>` | R26 |
| `png/pdf::from_svg(svg, fonts, &o)`, `Options::memory` | `from_svg(svg, fonts, &o, &bounds)`; `memory` removed | F63 |
| `rocq::Options::prelude: String`, `rocq::Unsupported` | `prelude: Option<String>`; `Unsupported` `#[non_exhaustive]` | F15 R139 R140 |
| the command's `Styles` | `export::Styles`; `+ Settings`, `Clock` | R141 R142 R148 R183 |
| `batch::Options { search, mode, cores, workers, memory_limit }`, `prove/run(problems, &o, ..)`, public-field `Plan`, `Problem`, `Answer` | `batch::Options { mode, cores, workers, total_memory_bytes }`; `prove/run(problems, &batch, &search, &bounds, ..)`, `run_local`, `Results::cancel`, `Cancel`; all `#[non_exhaustive]`, `Problem::new`, `Answer: Serialize` | F74 F77 R40 R48 R150 F164 |
| `ordinary::Sequent::new -> Self`, `Formulas::add` unchecked; `Image::read_back(&l)`, `ordinary::Derivation::check()` | `-> Result<Self, Error>`, checked; `read_back(&l, &bounds, stop)`, `check(&bounds, stop)`; `+ ordinary::decide` | F13 F12 R24 R113 R249 |
| `families::Family::instance` (panics) | `-> Result<Instance, Error>` | F6 |
| `sequents::fmt::{Walk, Visit}` crate-private, binary | public, `Between { index }`, a binder stop reserved | R245 D-10 |

### 2.5 Open types

`#[non_exhaustive]`, beyond today's sixteen marks: the enums `Error`,
`Class`, `Phase`, `NetError`, `ShapeError`, `StepError`,
`rocq::Unsupported`, `Schedule`, `Cadence`, `Labels`, `OpenGoal`,
`typst::Layout`, `svg::Advances`, `ordinary::{Logic, Translation}`,
`lltp::Status`, `Needs`, `Request`, `Response`; every options value
(`Mode`, `Bounds`, `Settings`, `Clock`, `search::Options`,
`batch::{Options, Plan, Problem, Answer}`, `ViewOptions`,
`TextOptions`, the export `Options`, `svg::{Style, Font}`, `Styles`,
`ordinary::Options`); the public-field structs `Progress`, `Inference`,
`ordinary::Inference`, `Dyadic`, `CheckError`, `ParseError`, `Size`,
`Criterion`, `Step`, `GoalView`, `FormulaView`, `Applicable`; and every
struct-like variant of a non-exhaustive enum.

## 3. The data model

What changes, and what a later step extends; the proof term, the
checker and the engines' internals are drafts B's and C's to detail.

### 3.1 `Sequent`

```rust
pub struct Sequent {
    terms: Vec<Term>,     // the arena, a subterm before its parents
    roots: Vec<TermId>,   // in the order written (C1)
    atoms: Vec<String>,   // distinct identifiers, by Atom
    antecedents: u32,     // roots written left of ⊢ (0: one-sided input)
    // step 38, empty propositionally: fo_terms, symbols, arities, binder names
}
const _: () = assert!(size_of::<Term>() == 12);   // F84
```

- **Written order is canonical** (C1, A): `optimize` hash-conses and
  drops unreachable terms but keeps `roots` as written; arena indices
  follow first occurrence along the roots; parse, JSON, `Display` and
  `Forest::roots()` keep it. One commit before 0.1.0 regenerates the
  snapshots whose roots were not ascending (F24).
- **The written sides are kept** (`antecedents`, A10): the one-sided
  arena cannot tell `|- a, top` from `~a |- top`, which lower alike.
  The forest and the engines ignore it; `Reading` takes the written
  succedent as the goal and refuses two written succedents (H10) and a
  `⊥` it would have to move (H9); the LLTP drivers read the clause
  roles from it (R74). It is part of equality. `Sequent::add` keeps it
  only when the added sequent has none.
- **Readers within bounds**: `Sequent::parse_within(text, &bounds)`
  hands `bounds.occurrences` to the parser's term bound, which refuses
  before allocating (F16); `FromStr` uses `Bounds::DEFAULT`. Identifiers
  are normalized to NFC (HD3, a dependency behind `parse` through
  `new-tool`); a JSON sequent's atom names must be identifiers of the
  text syntax (HD5, H19: `AtomName`).
- **A checked builder** (R59, story 5), for code without `parse`:

```rust
impl Sequent { pub fn builder() -> Builder }
impl Builder {
    pub fn atom(&mut self, name: &str) -> Result<Atom, Error>;     // interned, HD5-checked
    pub fn term(&mut self, term: Term) -> Result<TermId, Error>;   // operands exist; hash-consed
    pub fn dual(&mut self, id: TermId) -> TermId;                  // negation normal form, iterative
    pub fn left(&mut self, id: TermId) -> &mut Self;               // a written antecedent
    pub fn right(&mut self, id: TermId) -> &mut Self;              // a written succedent
    pub fn finish(self, bounds: &Bounds) -> Result<Sequent, Error>;
}
```

  Step 38 adds `predicate`, `function`, `bind`, `variable` with R136's
  checks; the propositional calls stay.

### 3.2 `Term`, `Kind`, `Atom`, `Sign`

Renamed (F25, R251): `Term::{Atom, DualAtom}`, `Kind::{Atom, DualAtom}`,
`Sign::{Atom, Dual}`, derived order kept (literal groups are indexed by
`2·atom + sign`). The JSON tags `V`, `D` stay, documented as atoms.
`Atom` is documented as a predicate symbol, nullary until step 38.

### 3.3 `Forest`: the contract

- **The numbering is a promise**: preorder over the roots in written
  order, left before right, a subtree a contiguous range, so a foreign
  checker (step 31's Rocq library, a web client) recomputes every id
  from the sequent (R69; a test pins the ids of a sequent of every
  connective).
- **`roots()` is the conclusion in written order**; ids may continue
  past it with extra trees the forest records (`cut_pairs()`, empty
  until 34; D-2); `sequent()` is the conclusion.
- **Built within a bound**: `Forest::within(&s, &bounds)` refuses more
  than `bounds.occurrences` before allocating; every reader of a form
  with a sequent builds through it (F27). About 25 bytes an occurrence,
  not counted under `memory_bytes`. `lca` costs the tree's depth, as its
  doc says (R29).
- `Forest::dual_literals(x, y)` is the one predicate for axiom partners
  (F58), where step 38 compares instances.

### 3.4 Members and identifiers

```rust
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Member(u32);     // an occurrence, or (33, 38) an instance in a table its owner keeps
impl Member {
    pub const fn of(occurrence: OccId) -> Self;
    pub const fn get(self) -> u32;
    pub fn occurrence(self, forest: &Forest) -> Option<OccId>;   // Some below forest.len()
}
const _: () = assert!(size_of::<Member>() == 4);
```

Every field and signature that lists a sequent's formulas takes `Member`
(D-1 b, R244, F19): `Inference.sequent`, `Dyadic`, `CheckError`'s
premises, `Interactive::goal`, `prove_goal`, `engine_for`,
`Outcome.goal`, `GoalView`. Propositionally a member *is* its occurrence
id, so on the wire a member is an integer and no form changes shape at
step 38. `Node`'s operands are draft B's (D-1 recommends members at the
leaves; `ax: [x, y]` is two integers either way).

`OccId`, `TermId`, `NodeId`, `InfId`, `GoalId`, `Atom`, `Member` are
`u32` on the wire. `GoalId` is new (F69): the session's numbering (root
first, then as steps opened goals), distinct from a `Derivation`'s
`InfId` (postorder).

### 3.5 `Mode` and `Fragment`

```rust
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Mode { pub intuitionistic: bool, pub affine: bool, pub mix: bool }
impl Mode {
    pub const CLASSICAL: Self; pub const INTUITIONISTIC: Self;
    pub const fn with_intuitionistic(self) -> Self;
    pub const fn with_affine(self) -> Self;        // was affine() (F40)
    pub const fn with_mix(self) -> Self;
    pub const NAMES: &'static [&'static str] =
        &["classical", "affine", "mix", "affine-mix", "intuitionistic", "intuitionistic-affine"];
    pub const fn name(self) -> &'static str;
    pub fn validate(self) -> Result<(), Error>;    // IntuitionisticMix; step 36's combinations
}
impl FromStr for Mode { type Err = Error; }        // inverts name(); the error lists NAMES
```

- Built outside the crate from the constants and builders only, so step
  36 adds `pub ordering: Ordering` without a break (R51); `Display`
  keeps its prose ("classical affine with Mix").
- **One table of words** (R241) replaces those of `cli/src/batch.rs` and
  `bench/src/problems.rs`; affine with Mix is `affine-mix` (the two
  tables wrote `mix-affine` and `affine`, F76).
- **On the wire a mode is its name** (A2, AIP-126): `"mode":
  "intuitionistic"`. An unknown word is refused by name, so step 36's
  `cyclic` and `lambek` need no key and cannot be misread as
  commutative (R10). The object of flags is read as the pre-release
  form, never written.
- `Fragment` keeps its opaque `u8` (five flags used; 38's `QUANTIFIERS`
  and 36's divisions take two; it can widen without a break); `NAMED`
  and `FromStr` (with or without the `I` prefix) become public; step 38
  adds `MLL1` … while `LL` stays propositional (R52).

### 3.6 `Reading`

`Reading::new` takes the one written succedent as the goal (by shape
only for one-sided input), keeps `hypotheses()` in written order (36's
ordered antecedent, R55), and gains `walk(o)`, the public two-sided
walk (R245); `ShapeError` gains `SeveralSuccedents` and `MovedBottom`
(H9, H10).

### 3.7 `Proof`

Unchanged in shape here (`forest`, `nodes: Box<[Node]>`, `Node` 16
bytes and closed); its form changes (7.3.2). Two rules:

- **A goal's proof is not a proof file**: `Outcome.goal` records the
  goal (F89) and the form writes `"goal": [members]`, which a proof
  reader refuses (`GoalProof`).
- **The mode travels where a file needs it** (R7): the proof form
  reserves `mode` (outcomes write it); `Proof` stays mode-free; step 31
  adds `ProofFile { proof, mode: Option<Mode> }` on the same form.

### 3.8 `Derivation`, `Inference`, `ViewOptions`

```rust
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ViewOptions { pub compact: Compact, pub sides: Sides }
pub enum Sides { #[default] Auto, One, Two }     // Auto: two-sided exactly in intuitionistic mode

impl Proof {
    pub fn derivation(&self, mode: Mode) -> Result<Derivation<'_>, Error>;
    pub fn derivation_within(&self, mode: Mode, view: &ViewOptions, bounds: &Bounds,
                             stop: impl FnMut(Progress) -> bool) -> Result<Derivation<'_>, Error>;
}
```

The derivation fits the mode by default (story 6); its bounds are
`derivation_bytes` and `memory_bytes`; `TooLarge` carries the compact
view's lower bound when one was tried (F65). `Inference` is
`#[non_exhaustive]`, its `sequent` members (36 keeps the written
sequence there, R61). `Derivation` gains a write-only form (R4, 7.3.7).

### 3.9 `Interactive`

```rust
pub struct GoalId(u32);                                         // F69
#[non_exhaustive] pub struct Step {
    pub position: usize, pub rule: Rule,
    pub left: Vec<usize>,          // ⊗, Mix, ⊸L: the context positions going left
    // step 38: pub witness: Option<Witness> (a term, or open)
}
impl Step { pub fn new(position: usize, rule: Rule) -> Self; pub fn with_left(self, left: impl Into<Vec<usize>>) -> Self; }
#[non_exhaustive] pub struct Applicable { pub rule: Rule, pub needs: Needs }
#[non_exhaustive] pub enum Needs { Nothing, Split }            // 38: Witness
#[non_exhaustive] pub struct GoalView { pub goal: GoalId, pub formulas: Vec<FormulaView> }
#[non_exhaustive] pub struct FormulaView {
    pub position: usize, pub member: Member,
    pub text: String,              // as Display writes it (Reading's in intuitionistic mode)
    pub side: Option<Position>,    // Input or Output under the reading
    pub rules: Vec<Applicable>,    // what apply accepts here, given the goal's context
}

impl Interactive {
    pub fn within(s: &Sequent, mode: Mode, bounds: &Bounds) -> Result<Self, Error>;   // new() is this with the defaults
    pub fn view(&self, id: GoalId) -> Result<GoalView, Error>;                           // and views()
    pub fn rules(&self, goal: GoalId, position: usize) -> Result<Vec<Applicable>, Error>;
    pub fn apply(&mut self, goal: GoalId, step: &Step) -> Result<Vec<GoalId>, Error>;
    pub fn split_passes(&self, goal: GoalId, step: &Step) -> Result<bool, Error>;
    pub fn close(&mut self, goal: GoalId, options: &Options, view: &ViewOptions, bounds: &Bounds,
                 stop: impl FnMut(Progress) -> bool) -> Result<Outcome, Error>;
    pub fn close_with(&mut self, goal: GoalId, proof: &Proof, view: &ViewOptions, bounds: &Bounds,
                      stop: impl FnMut(Progress) -> bool) -> Result<(), Error>;
    pub fn close_all(&mut self, options: &Options, view: &ViewOptions, bounds: &Bounds,
                     stop: impl FnMut(Progress) -> bool) -> Vec<(GoalId, Result<Outcome, Error>)>;
    pub fn derivation_ids(&self) -> Vec<GoalId>;   // drawn inference n is goal ids[n]
    pub fn proof(&self, bounds: &Bounds, stop: impl FnMut(Progress) -> bool) -> Result<Proof, Error>;
    // goal -> Option<&[Member]>, goals, undo -> Option<GoalId>, derivation, reading,
    // inferences, steps: as today, with GoalId for InfId
}
```

- `rules` lists only what `apply` would accept given the goal's
  context, without cloning the state; a rule needing a split says so
  instead of enumerating splits (R91, R93). The reading's positions are
  stored in the state (F66).
- `close_all` keeps every goal's result; a goal whose search errs or
  whose graft is refused stays open (F68); `bounds.work` is per goal
  (R92). `close_with` refuses a proof of another sequent or that the
  mode forbids (`ForeignProof`, F23).
- A one-sided Mix sending every formula to one side is refused
  (`StepError::EmptyPremise`, C3 A, F67). Reading a session back is
  linear (F21) and checks each history entry against the arena as it
  was (H17).
- **Driven from JSON**: step 32 adds `Request`, `Response` and
  `serve(&mut self, &Request, &Settings, stop) -> Result<Response,
  Error>` over types that exist from 28 (forms in 7.3.6). A stateless
  worker reads the page's session JSON, serves one request and posts
  the new session: a trap costs a replay, never the student's proof.
  `interact` parses its lines into `Request`s, so its messages come from
  the library (R129); a drawing returns its id map (R163).

### 3.10 `ProofStructure` and `Criterion`

```rust
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Criterion { pub mix: bool }   // 35: essential; 36: ordering
impl Criterion { pub const MLL: Self; pub const fn with_mix(self) -> Self; pub fn of(mode: Mode) -> Result<Self, Error>; }
```

`new`, `from_links` and the reader take it for `mix: bool` (F10, R79);
`from_proof` matches every `Node` (F49, R254) and takes bounds and a
stop, since step 33's instances unfold shared subproofs; `is_correct`
and `sequentialize` are long calls (R25). Step 33's vertex type is
draft C's.

### 3.11 The ordinary layer

`ordinary::Sequent::new` and `Formulas::add` are fallible and checked
(F13, R249); step 38 adds closedness and arities behind the same
signatures. One call decides (R113, the web's task 9):

```rust
pub fn decide(s: &ordinary::Sequent, options: &ordinary::Options, search: &search::Options,
              bounds: &Bounds, stop: impl FnMut(Progress) -> bool) -> Result<ordinary::Outcome, Error>;
#[non_exhaustive] pub struct Outcome {
    pub verdict: ordinary::Verdict,       // Valid(Box<Derivation>) | NotValid | Unknown(Reason)
    pub logic: Logic, pub translation: Translation,
    pub linear: search::Outcome,          // the image's outcome
}
```

`Valid` carries the LK or LJ derivation read back within `bounds` (as a
DAG where subproofs are shared, R24) and checked; a second procedure
(G4ip, 31's countermodels) plugs in behind `decide` as a table row. The
ordinary forms are born versioned with first-order keys reserved (R11,
7.3.11).

## 4. Errors

### 4.1 One family

```rust
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error { /* 4.3 */ }
impl Error {
    pub fn code(&self) -> &'static str;             // the stable reason a client branches on
    pub fn class(&self) -> Class;
    pub fn is_fault(&self) -> bool;                 // class() == Class::Invalid
    pub fn setting(&self) -> Option<&'static str>;  // the Settings key that lifts or changes it
    pub fn describe<'a>(&'a self, forest: &'a Forest) -> Described<'a>;   // formulas for ids
    pub const CODES: &'static [&'static str];
}
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Class {
    Invalid,      // the input or the request is at fault: the only verdict on it
    Limit,        // a bound refused the call; lifting it may answer
    Stopped,      // the caller's stop ended the call
    Unsupported,  // this build, engine, kernel or reader does not do it
    Failed,       // the environment failed: threads, the writer, the renderer
    Defect,       // a check of the library's own result failed: report it
}
```

- **A refusal is never a fault.** Only `Invalid` says the input is
  wrong. `From<CheckError> for Error` sends a checker that gave up for
  memory or its stop to `MemoryLimit`/`Stopped`, never to
  `InvalidProof`, which holds faults only (a `debug_assert!`). A
  search's refusal is a verdict, `Unknown(Reason)`; a proof found but
  not checkable within the bound is `Unknown(Reason::Unchecked)` (R138,
  F139).
- **Every public fallible call returns `Error`** (F8), so the front
  page's "errors are one type" is true; the payload types (`ParseError`,
  `CheckError` with `Problem` and `Dyadic`, `NetError`, `ShapeError`,
  `StepError`, `rocq::Unsupported`) stay public for matching on detail.
- **Size**: large payloads are boxed, `assert!(size_of::<Error>() <=
  64)` (112 today; the largest unboxed variant is `InvalidOption`'s two
  strings).
- `Send + Sync + 'static`, `std::error::Error`, lowercase `Display`,
  counts that agree (F33, C-GOOD-ERR). `describe(&forest)` writes
  formulas for ids (F48); `Described::abbreviated(limit)` cuts lists.
- **No public call panics on input of the right type** (R130, S5): the
  readers, `apply`, `undo` (H17), the exports (style values validated,
  F4), the families (F6); kept `expect` sites are listed in `core.md`.
  `linlog-web` installs a panic hook that posts its own
  `{"code": "panic"}` (not one of `Error::CODES`) and respawns.

### 4.2 The wire form (written only, nested)

```json
{"code": "too_many_occurrences", "class": "limit",
 "message": "the sequent unfolds to at least 67108864 subformula occurrences, more than the limit of 1000000",
 "setting": "bounds.occurrences", "details": {"occurrences": 67108864, "limit": 1000000}}
{"code": "parse", "class": "invalid", "message": "unexpected \"⊢\" at line 1, character 4",
 "details": {"span": {"start": 3, "end": 6}, "span_utf16": {"start": 3, "end": 4},
             "line": 1, "column": 4, "found": "⊢", "expected": ["a formula"]}}
```

`code` is what a client branches on (AIP-193), `class` what it shows (a
red "invalid" only for `invalid`), `setting` the key the command maps to
its flag and the web client highlights, `details` the variant's named
fields. Nested payloads carry a `kind` (`problem`, `step`, `net`,
`shape`, `certificate`); `invalid_proof`'s details are `node`, `rule`
(the node's form), `premises` (`{"theta", "gamma", "any"}`) and
`problem`. A `ParseError` gives its span in bytes, in UTF-16 units (what
a JavaScript editor indexes) and as line and column in characters,
computed when it is made (R129).

### 4.3 The variants

| today | after | code | class |
|---|---|---|---|
| `SequentParsing(Vec<ParseError>)` | `Parse(Box<ParseError>)` | `parse` | invalid |
| `Lltp`, `Mist`, `Tptp` (`String`) | `Lltp`, `Spec`, `Tptp { message }` | `lltp`, `spec`, `tptp` | invalid |
| — (HD1) | `SeveralConjectures { second }` | `several_conjectures` | invalid |
| — (serde) | `Malformed { form, message }` | `malformed` | invalid |
| — (HD5, H19) | `AtomName { name }` | `atom_name` | invalid |
| the six index errors | `AtomIndex`, `TermIndex`, `NodeIndex`, `OccurrenceIndex { index, len }`, `SubtermOrder`, `PremiseOrder` | `atom_index`, `term_index`, `node_index`, `occurrence_index`, `subterm_order`, `premise_order` | invalid |
| `NotIntuitionistic(ShapeError)`, `IntuitionisticMix`, `GoalOutputs`, `Succedents`, `FragmentMismatch`, `Translation` | same, named fields | `not_intuitionistic`, `intuitionistic_mix`, `goal_outputs`, `succedents`, `fragment_mismatch`, `translation` | invalid |
| `InvalidProof(CheckError)` (faults) | `InvalidProof(Box<CheckError>)` | `invalid_proof` | invalid |
| — (F89, F23) | `GoalProof`, `ForeignProof` | `goal_proof`, `foreign_proof` | invalid |
| `InvalidNet(NetError)` | `InvalidNet(Box<NetError>)` | `invalid_net` | invalid |
| `Refused(Refusal)` | `Step(StepError)` | `step` | invalid |
| `InconsistentState`, `OpenGoals` | `InconsistentSession { reason }`, `OpenGoals { count }` | `inconsistent_session`, `open_goals` | invalid |
| — (F6, F4, F38) | `FamilySize { family, size }`, `InvalidOption { key, message }` | `family_size`, `invalid_option` | invalid |
| `RenderError::{NoDate, Svg}` | `NoDate`, `NotSvg { message }` | `no_date`, `not_svg` | invalid |
| `TooManyOccurrences` | same | `too_many_occurrences` | limit |
| `Unchecked` (outside a search), `Problem::Memory`, `ViewError::Memory`, `RenderError::Memory` | `MemoryLimit { phase, limit_bytes, needed_bytes }` | `memory_limit` | limit |
| `ViewError::TooLarge`, `svg::TooLarge` | `TooLarge { what, estimate_bytes, limit_bytes, least_bytes }` | `too_large` | limit |
| `RenderError::TooLarge` | `TooManyPixels { pixels, limit }` | `too_many_pixels` | limit |
| `ViewError::Stopped`, `WriteError::Stopped`, `Problem::Stopped` (new) | `Stopped { phase }` | `stopped` | stopped |
| — (F26) | `UnsupportedVersion { version, supported }` | `unsupported_version` | unsupported |
| `TooManyNodes`, `ViewError::TooMany` | `TooManyNodes { nodes }`, `TooManyInferences { inferences }` | `too_many_nodes`, `too_many_inferences` | unsupported |
| `NetFragment`, `NetMode`, `NetGoal`, `EngineMode`, `NotAdditive`, `NotHorn`, `NoEngine` | same, named fields; `NoEngine` constructed by the dispatch (R135) | `net_fragment`, `net_mode`, `net_goal`, `engine_mode`, `not_additive`, `not_horn`, `no_engine` | unsupported |
| `WriteError::Unsupported` | `NoCertificate(rocq::Unsupported)` | `no_certificate` | unsupported |
| — (32) | `FeatureOff { feature }` | `feature_off` | unsupported |
| `ThreadPool`, `WriteError::Failed`, `RenderError::Failed` | `ThreadPool { threads, message }`, `WriteFailed`, `RenderFailed { message }` | `thread_pool`, `write_failed`, `render_failed` | failed |
| `Rejected`, `ReadBack` | same | `rejected`, `read_back` | defect |

`setting()`: `too_many_occurrences` → `bounds.occurrences`,
`memory_limit` → `bounds.memory_bytes`, `too_large` →
`bounds.derivation_bytes`, `too_many_pixels` → `styles.png.pixels`,
`no_date` → `styles.pdf.date`. `Phase` (shared with `Progress`):
`Read`, `Search`, `Check`, `View`, `Write`, `Render`, `Net`, `ReadBack`
(34 adds `Eliminate`, 31 `Refute`). `StepError` keeps the eleven kinds of
`Refusal` and gains `EmptyPremise` (F67); 36 adds `Exchange`, 38
`Witness`, `Eigenvariable`, `Unresolved`. The harness classifies by
`class()`: `unsupported` is its "refused" (R135), `limit` and `stopped`
"unknown", `defect` and `failed` "error". Codes reserved for later:
`cut_formula`, `not_dual_pair`, `elimination_limit` (34, R131); box kinds
in `net` (33, R132); `ordered_fragment`, `empty_antecedent`,
`ordered_mode` (36, R134); `arity`, `free_variable`, `unbound_variable`,
`term_size`, `first_order_goal` (38, R136).

## 5. Bounds and stops

### 5.1 One bounds value

```rust
/// The resources a call may use, which the library enforces.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]      // serde(default, deny_unknown_fields)
pub struct Bounds {
    pub memory_bytes: Option<u64>,       // what a call holds at once: a search's account, a
                                         // check's pass, a view's making, a render's estimate
    pub occurrences: Option<u64>,        // the most a sequent unfolds to, on every reader
    pub derivation_bytes: Option<u64>,   // a derivation's, a net drawing's, a sequent drawing's estimate
    pub work: Option<u64>,               // the most units of work of a search (5.3)
    pub recursion_depth: u32,            // the deepest recursion on any one stack (5.4)
    // 38: pub term_nodes: Option<u64>   the unfolded size of first-order terms (R250)
}
impl Bounds {
    pub const DEFAULT_MEMORY_BYTES: u64 = 1 << 30;
    pub const DEFAULT_OCCURRENCES: u64 = 50_000_000;     // above the largest LLTP problem's 27.8 M
    pub const DEFAULT_DERIVATION_BYTES: u64 = 64 << 20;
    pub const DEFAULT_RECURSION_DEPTH: u32 = 2048;
    pub const DEFAULT: Self; pub const BROWSER: Self; pub const UNBOUNDED: Self;
    pub const fn stack_bytes(&self) -> usize;
    pub const fn recursion_depth_for_stack(bytes: usize) -> u32;     // R45
    // one `with_*` per field
}
```

- **`BROWSER`** (R142), provisional until step 32 measures it in a
  worker (D16 wants a measurement; none exists): `memory_bytes` 256 MiB
  (a quarter of what a tab may grow to, leaving room for the forest, the
  view and the SVG); `occurrences` 1 000 000 (a 25 MB forest, uncounted
  above); `derivation_bytes` 4 MiB (an SVG peaks near 6.5 times the
  estimate, so 26 MB); `work` none (the client's stop keeps the
  deadline); `recursion_depth` `recursion_depth_for_stack(1 << 20)`,
  rustc's default wasm stack, about 400 with today's figure; a worker
  linked with `-zstack-size` raises it.
- **`stack_bytes`** = `recursion_depth × PER_LEVEL + RESERVE` (2 304
  bytes a level optimized, 12 288 unoptimized, measured on x86-64; 256
  KiB above the engine). The 8 MiB floor of `Options::stack_size` goes
  (it was for the derivation's builder and renderer, which no longer
  recurse, `core-derivations.md`; unverified that nothing else needs
  it; the command may keep a floor of its own). The wasm32 per-level
  figure is step 32's measurement.
- **A bound of 2⁵³ or more is no bound**: written as `null`; a reader
  refuses such a number (`invalid_option`, saying `null` lifts it).
- **What `memory_bytes` counts** is stated on the field (R29): the
  search's growing structures, the checker's pass, a view's record and
  derivation, a render's estimate, 35's closure, 37's database; not the
  forest (bounded by `occurrences`), the sequent, the proof returned,
  the net engine's linear structure, stacks, the allocator. A call's
  peak is `memory_bytes` for its largest phase plus the forest, the
  proof and a constant R18's measurement pins.
- **One account per call, and one per race** (R18): a search, its
  refutation pass and its check charge accounts of the bound one after
  the other; the two searches of `search::race` charge one account of
  the whole bound, so a call without flags holds `memory_bytes`, not
  twice it (2.05 GB on `SYJ206+1.016`). The default bias's pair keeps
  its halves (`Account::share`), on which the pinned counters depend.

### 5.2 One stop, with progress

```rust
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Progress {
    pub work: u64,         // units of work since the previous poll
    pub held_bytes: u64,   // what the call's account holds now
    pub phase: Phase,
}
```

Every long call takes `stop: impl FnMut(Progress) -> bool`: true ends a
search with `Unknown(Reason::Stopped)` and any other call with
`Error::Stopped { phase }`. No trait and no second form (A6): a
closure that ignores progress is `|_| false`, and its argument type
infers from the `FnMut(Progress)` bound, which a `Stop` trait with a
blanket impl would lose. The crate-private `search::Stop` (closure,
turn, slice, flags) stays the engines' mechanism and hands the caller
the public progress.

- **The unit of work** is a step of the call's main loop, the same on
  every build: a stable sequent, a split candidate or a forced split
  (focused engines), a literal chosen or a failed exact test (net), a
  pair (additive), a marking expanded or a pivot of the state equation
  (Horn), a node (checker, size pass), an inference (view, writers,
  read-back, ordinary checker), a link or a step (net calls). It is
  counted apart from the work that slices the default bias's two
  searches, so no counter of a decided run moves (D17). On one thread
  the progress summed over a search equals `Statistics::work` (R243's
  test).
- **Polled at a bounded interval** (R30): the engines where
  `core-search.md` lists; the checker and the size pass every 4 096
  nodes (new, R23); views and writers per inference; the net criterion
  per edge of its witness isolation; set-up passes every 65 536
  occurrences. With `work` in hand a condition never counts polls.
- **The web client's deadline**: the bindings add up `work`, read
  `performance.now()` once per 2¹⁶ units, post the progress on the same
  schedule and stop past `settings.clock.time_limit_ms`. The crate's
  example that counted polls becomes this pattern with a counter
  (F136).
- **AIP-151 as a pattern**: progress (the stop), an end (`Outcome` or
  `Error`), a cancellation (the stop, `Cancel`, a terminated worker)
  that ends in a defined state, never a partial result read as whole.
  The worker posts `{"message": "progress", …}`, then `{"message":
  "done", "outcome"}` or `{"message": "failed", "error"}`.

### 5.3 Work budget and determinism

`Bounds::work` bounds a search (`prove*`, `race`, each goal of
`close_all`, 31's refuters): past it, `Unknown(Reason::WorkLimit {
limit })` with `Statistics::work` saying what was done, so a client
sizes the next budget (R21). Views and checks after a search are bounded
by memory and size.

```rust
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Schedule { #[default] Auto, Turns }   // Auto: the default bias's pair on two threads where
                                               // `parallel` starts one, else turns; Turns: always turns
```

With `jobs: 1` and `schedule: Turns` the verdict, `Reason`,
`Statistics` and sequence of polls are a function of the input and the
options on every build, wasm included (R47, R151); `--deterministic`
sets both. `Decide::decide` stays the one entry, so a suspendable engine
can replace it (32's open question).

### 5.4 Recursion and the pool's stack (H18)

`recursion_depth` bounds nesting **on any one thread's stack**, not
along a branch: a pool worker that runs a stolen job while it waits
counts the job's depth on its own (or keeps to its branch's jobs; the
search area chooses, with a test at a raised limit on two threads). A
branch past it answers `Unknown(Reason::RecursionLimit { depth })`,
never a trap; workers are sized by `stack_bytes()`. A queued pool task
polls the stop before it starts, so a pool returns promptly (R19).

### 5.5 Reading within bounds

```rust
pub trait ReadWithin: Sized + sealed::Sealed {   // R27, F27
    fn read_within<'de, D: serde::Deserializer<'de>>(d: D, bounds: &Bounds) -> Result<Self, Error>;
}   // Sequent, Proof, ProofStructure, Interactive, ordinary::{Sequent, Derivation}; 31's ProofFile
pub fn from_json_within<T: ReadWithin>(json: &str, bounds: &Bounds) -> Result<T, Error>;
pub fn to_json<T: serde::Serialize + ?Sized>(value: &T) -> String;
```

`Deserialize` is `read_within` with `Bounds::DEFAULT`. Reading checks
the version (7.1), deserializes the private proxy (linear in the text),
counts the occurrences before any forest exists, then builds and
validates; a JSON syntax error is `Malformed` with line and column.
`check` and `interact --state` read through it (`--occurrence-limit`).

### 5.6 Every long public call (R22)

| call | bounds | stop | without them |
|---|---|---|---|
| `Sequent::parse_within`, `FromStr`, `lltp::read_within`, `mist::read_within`, the ordinary parsers | `occurrences` (before allocating) | none | linear in the text; a front end caps its bytes |
| `wire::from_json_within`, `read_within`, `Deserialize` | `occurrences`; nodes under 2³² | none | linear (a session's replay too, F21) |
| `Forest::within`, `new`; `Builder::finish`; `Family::instance` | `occurrences` | none | linear (0.43 s at 27.8 M) |
| `engine_for` | — | none | passes over the goal |
| `prove`, `prove_within`, `prove_goal`, `race` | `memory_bytes`, `work`, `recursion_depth` (+ `copies`, `memo_limit`) | every engine, set-up per 65 536 | unbounded |
| `batch::{prove, run, run_local}` | per problem; `total_memory_bytes` | each search; `Cancel` | unbounded |
| `Proof::{check, check_within, derivation_size_within}` | `memory_bytes` | per 4 096 nodes (new) | quadratic time in flat memory |
| `Proof::{derivation, derivation_within}` | `derivation_bytes`, `memory_bytes` | per inference | exponential in sharing |
| `write_text`, `export::*::write`; `rocq::{write, ordinary}` | (a bounded derivation) | per inference | linear (F3: identifiers linear) |
| `export::*::derivation -> String`, `Display` | (a bounded derivation) | none | linear |
| `{latex, typst, svg}::sequent` | `derivation_bytes` (estimated first, F5) | none | 310 bytes an occurrence |
| `svg::net` | `derivation_bytes` | per literal | linear |
| `png/pdf::from_svg` | `memory_bytes` (estimated before parsing) | not polled: renderers cannot stop | at most 8 s at 1 GiB |
| `ProofStructure::{from_links, link, unlink, is_acyclic}` | — | none | linear |
| `ProofStructure::from_proof` | `memory_bytes` (33's instances) | per node | linear in MLL |
| `ProofStructure::is_correct`, `sequentialize` | — / `memory_bytes` | per edge, per step | quadratic |
| `Interactive::{new, within}` | `occurrences` | none | linear |
| `Interactive::{goals, view, views, rules, apply, split_passes, undo, derivation, derivation_ids}` | — | none | linear in the goal or the arena (F66) |
| `Interactive::{close, close_with, close_all, proof}`, `serve` | all | yes | a search and a view |
| `ordinary::translate` | `occurrences` | none | linear |
| `Image::read_back`, `ordinary::Derivation::check`, `ordinary::decide` | `memory_bytes`, `derivation_bytes` (+ the search's) | per inference | exponential unfolded (R24) |
| 31's refuters and `Refutation::check`, 34's elimination, 37's saturation | `memory_bytes`, `work` | per step | unbounded |

## 6. Options

### 6.1 One convention

Public fields under `#[non_exhaustive]`; `Default` equal to named
`DEFAULT_*` constants; a `#[must_use]` `with_*` per field (F64), so
`Options::default().with_copies(None)` chains; the form
`serde(default, deny_unknown_fields)`: a missing key is the default, a
misspelt one `invalid_option` naming it (F61, F78). An options form has
no `version`: its version is its key set, which only grows, and an
older reader refuses a newer key by name (R6). Every field has a flag
and a documentation line (D16). On the wire `null` means "no bound" and
nothing else; an automatic choice is `"auto"` (F79).

### 6.2 `search::Options`

| field | type | default | JSON | read by | flag |
|---|---|---|---|---|---|
| `engine` | `Option<Engine>` | the dispatch | `"auto"` or a name | front door | `--engine` |
| `fragment` | `Option<Fragment>` | detected | `"auto"` or a name | front door | `--fragment` |
| `bias` | `Bias` | `Auto` | `"auto"`, `"rarer"`, `"factors"` | focused | `--bias` |
| `copies` | `Option<u32>` | `Some(3)` | number or `null` | focused | `--copies` |
| `forward_copies` | `u32` | 30 | number | focused | `--forward-copies` |
| `memo_limit` | `usize` | 2²⁰ | number | focused, additive | `--memo-limit` |
| `test_period` | `Cadence` | `Auto`: every link up to `SMALL_STRUCTURE` (200) occurrences, every `DEFAULT_TEST_PERIOD`th (4) above | `"auto"` or a number | net | `--test-period` (new, F81) |
| `jobs` | `usize` | 1, clamped to `MAX_JOBS` (256) at use | number | focused, net with `parallel` | `--jobs` |
| `schedule` | `Schedule` | `Auto` | `"auto"`, `"turns"` | default bias's pair | `--schedule` (new) |
| `check` | `bool` | true | boolean | front door | `--no-check` |
| `pool` | `Option<Pool>` (`parallel`) | none | never on the wire | parallel engines | from `--jobs` |

`jobs` is on the wire in every build and ignored without `parallel`, so
one file serves the command and the web (R49); `pool` is the runtime
handle beside the data (R116). `NET_MULTIPLICITY` becomes a public
documented constant, fixed (R149, recorded in `plan/later.md`). Later
fields, each defaulting to today's behaviour: `refute` (31, C2), the net
prunes' switches (35, R143), the inverse engine's knobs (37, R147), the
first-order bounds (38, R38), a loop check (R114).

### 6.3 `ViewOptions`, `Clock`, `batch::Options`

`ViewOptions { compact: "auto"|"always"|"never", sides:
"auto"|"one"|"two" }`, now `deny_unknown_fields` (F61).

```rust
/// What a front end with a clock applies through its stop; the library never reads it.
#[non_exhaustive] pub struct Clock {
    pub time_limit_ms: Option<u64>,        // DEFAULT_TIME_LIMIT_MS = 2000 (R148)
    pub pool_after_ms: Option<u64>,        // DEFAULT_POOL_AFTER_MS = 100: when a race adds its pool
    pub batch_time_limit_ms: Option<u64>,  // the whole batch's limit, none by default
}
#[non_exhaustive] pub struct Options {                      // batch::Options
    pub mode: Mode,                        // CLASSICAL, for a problem without its own
    pub cores: Cores,                      // Auto | Across | Within
    pub workers: usize,                    // 1, clamped to MAX_JOBS (F73)
    pub total_memory_bytes: Option<u64>,   // 4 GiB
}
#[non_exhaustive] pub struct Plan { pub workers: usize, pub search: search::Options, pub bounds: Bounds }
#[non_exhaustive] pub struct Problem { pub name: String, pub sequent: Sequent, pub mode: Option<Mode> }
#[non_exhaustive] pub struct Answer { pub name: String, pub outcome: Result<Outcome, Error> }   // the record

pub fn prove(problems: impl IntoIterator<Item = Problem, IntoIter: Send + 'static>, batch: &Options,
             search: &search::Options, bounds: &Bounds) -> Results<Answer>;
pub fn run<P: Send + 'static, R: Send + 'static>(problems: impl IntoIterator<Item = P, IntoIter: Send + 'static>,
             batch: &Options, search: &search::Options, bounds: &Bounds,
             work: impl Fn(P, &Plan, &Cancel) -> R + Send + Sync + 'static) -> Results<R>;
pub fn run_local<P, R>(problems: impl IntoIterator<Item = P>, batch: &Options, search: &search::Options,
             bounds: &Bounds, work: impl FnMut(P, &Plan, &Cancel) -> R) -> impl Iterator<Item = R>;
impl<R> Results<R> { pub fn cancel(&self); pub fn canceller(&self) -> Cancel; }   // Drop cancels too
pub struct Cancel(/* Arc<AtomicBool> */);  // is_cancelled(), cancel()
```

`run_local` (no `Send`, the caller's thread, R48) is a separate entry
because features must stay additive: `Send` bounds that appeared only
with `parallel` would break a wasm crate once anything in its graph
enabled the feature. `Results::cancel` reaches searches in flight (R40,
F74); the work closure's stop is `|_| cancel.is_cancelled() ||
deadline()`, the per-sequent limit of story 12. A problem's own
overrides (R73) are a later field of `Problem`.

### 6.4 The export options and `Styles`

The seven output values keep their fields and forms and become
`#[non_exhaustive]`. Changes: `png/pdf::Options::memory` go (renders
read `bounds.memory_bytes`, F63); `rocq::Options::prelude:
Option<String>`, `None` meaning the target's own lines (NanoYalla's
`macroll`, `Classical_Prop` for a classical ordinary certificate,
nothing for LJ: F15, R140); 31 adds `kernel` (R139); values are
validated before anything is written (`InvalidOption`): SVG lengths
under stated maxima (F4), a Rocq lemma an identifier outside the
reserved names (F38), Typst lengths by a small grammar.

```rust
#[non_exhaustive] pub struct Styles {      // R141, what --style keys address
    pub text: TextOptions, pub latex: latex::Options, pub typst: typst::Options,
    pub svg: svg::Style, pub png: png::Options, pub pdf: pdf::Options, pub rocq: rocq::Options,
}
```

Every field exists in every build; a format whose feature is off has a
private placeholder that reads any value and writes nothing, so one
settings file reads everywhere and a misspelt format is still refused.

### 6.5 `Settings`, what a front end holds

```rust
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]         // serde(default, deny_unknown_fields)
pub struct Settings {
    pub clock: Clock, pub bounds: Bounds, pub search: search::Options,
    pub view: ViewOptions, pub styles: export::Styles, pub batch: batch::Options,
}
impl Settings {
    pub fn browser() -> Self;
    pub fn set(&mut self, key: &str, value: &str) -> Result<(), Error>;   // dotted key; JSON, else a string
    pub fn keys() -> Vec<&'static str>;                                   // for help texts and errors
}
```

- **`Settings::default()` is the command's behaviour** (A11, story 1):
  `search.copies` none (a front end with a clock deepens), the clock's
  2 s and 100 ms, `Bounds::DEFAULT`. `search::Options::default()` keeps
  the copy bound of 3, since a library call without a stop must end;
  the crate's front page says so in one paragraph (F42).
- **`Settings::browser()`**: `Bounds::BROWSER`; `copies` none, `jobs` 1,
  `schedule` `Turns`; 2 s and no pool; `styles.svg` with `ids` and
  `description` on and no background.
- `set` takes `styles.svg.text`, `bounds.memory_bytes`,
  `search.copies`, …, validating the whole value (R141; needs
  `serde_json`, A3).

### 6.6 How each front end sets them (D15)

The command builds one `Settings` from its file (`--settings FILE`,
R183) and then its flags, each the spelling of a key:

| flag | key | flag | key |
|---|---|---|---|
| `--timeout` | `clock.time_limit_ms` | `--engine`, `--fragment`, `--bias` | `search.engine`, `.fragment`, `.bias` |
| `--pool-after` | `clock.pool_after_ms` | `--copies`, `--forward-copies` | `search.copies`, `.forward_copies` |
| `--batch-timeout` | `clock.batch_time_limit_ms` | `--memo-limit`, `--test-period` | `search.memo_limit`, `.test_period` |
| `--memory-limit` | `bounds.memory_bytes` | `--jobs`, `--schedule` | `search.jobs`, `.schedule` |
| `--occurrence-limit` (every reader, `check` and `interact` included) | `bounds.occurrences` | `--no-check` | `search.check` |
| `--derivation-limit` | `bounds.derivation_bytes` | `--compact` | `view.compact` |
| `--recursion-limit`, `--work-limit` (new) | `bounds.recursion_depth`, `bounds.work` | `--style K=V`, `--lemma`, `--prelude`, `--standalone` | `styles.K` |
| `--workers`, `--cores`, `--batch-memory` | `batch.workers`, `.cores`, `.total_memory_bytes` | `--deterministic` | `search.jobs = 1`, `search.schedule = turns`, `clock.pool_after_ms = null` |

The mode flags make the request's `Mode` and `batch.mode`. Whether
`--style-file` stays as a styles-only spelling is the command area's
choice. The harness maps its flags onto the same keys and records the
`Settings` of each run (R190); the web client keeps the JSON in its
storage, builds its panel from `Settings::keys()`, and sends it with
each request.

## 7. Wire forms

### 7.1 The policy

Stated once in `wire`'s documentation and `core-sequents.md`.

- **`version` is a wire level** (A1). Every top-level document starts
  with `"version": n`, the lowest level whose reader understands all of
  it; `wire::VERSION` is the highest a build reads and writes; level 1
  is 0.1.0's. The writer computes the level from the content, so a
  propositional cut-free value without boxes is level 1 in every later
  release and its bytes never change; a proof with a cut is written at
  step 34's level and a 0.1.0 reader refuses it by name. Levels are
  global, numbered as steps land, each one's additions listed under the
  changelog's "Wire forms" and in `wire`'s docs. Nested forms carry no
  `version`.
- **What raises the level**: a new tag; a new value of an enumeration in
  a form read back (a mode, a criterion); a key whose presence changes
  what other keys mean (`cuts`, `members`, `witnesses`). **What does
  not**: a key an older reader may ignore without misreading (a
  counter, an error detail, `linlog_version`, any key of a write-only
  form). No new meaning rides on an old tag through a new key
  (`impact-quantifiers.md` §3 item 8).
- **Reading**: a `version` above `wire::VERSION` is `unsupported_version`
  (both numbers named), before other keys: the writer puts `version`
  first and the reader reads the proxy through a `DeserializeSeed` that
  records the version when it meets the key, so the error names the
  version even when a later key fails. A document without `version`
  reads as level 1 and may use the pre-release names (`ids`,
  `var_dict`, `proof`, a net's `mix`, a mode as an object of flags): the
  proof files of 2026-09-30 and every form pinned today still load;
  nothing writes those names again.
- **Unknown keys**: a *data* form (sequent, proof, structure, session,
  ordinary forms, `Reason`, `Refutation`, `Statistics`) ignores them,
  which the level rule makes safe and which lets an outcome be read as a
  proof; an *options* form and a *command* (`Step`, `Request`) refuse
  them by name, since a typo there or a newer client's `witness` must
  not be applied as something else; a client of a *write-only* form
  ignores them, as each such form says (AIP-180).
- **Enumerations by name** (AIP-126), never numbers; a reader refuses an
  unknown value naming it and the known ones; a client treats the open
  enumerations of 7.4 as open sets.
- **Names** (AIP-140, T1): `lower_snake_case` nouns, units in the name
  (`limit_bytes`, `time_limit_ms`), no `is_`, one concept one key in
  every form (`sequent`, `roots`, `atoms`, `nodes`, `mode`, `goal`,
  `phase`). Renamed from the pre-release forms: `ids` → `roots`,
  `var_dict` → `atoms`, `proof` → `nodes`, `mix` → `criterion`, `once`
  → `clauses`, the reasons' bare numbers → named fields. The tags
  (`V`, `D`, `⊗`, `ax`, `⊕₁`, …) stay: short, numerous, documented.
- **Numbers** (P8, R247): integers only; ids `u32`; every integer read
  back below 2⁵³, `null` "no bound"; an outgoing count that saturates
  (`Size`, the estimates) is written as `wire::SATURATED` = 2⁵³ =
  9 007 199 254 740 992, exact in a double, meaning "2⁵³ or more", with
  `Size::exact` false (F70). `Statistics` counters, `Equation` counts and
  byte figures are clamped the same way, so the rule has no exception.
- **Text** is UTF-8; atom names are identifiers (HD5); text positions
  come in bytes, UTF-16 units and line/column (4.2).

### 7.2 Read back or written only

| form | direction | `version` | unknown keys |
|---|---|---|---|
| `Sequent` | both | when top-level | ignored |
| `Proof` (31: `ProofFile`), `ProofStructure`, `Interactive` | both | yes | ignored |
| `Mode`, `Fragment`, `Rule`, `Engine`, `Bias`, `Size`, `Reason`, `Refutation`, `Statistics` | both, nested | — | ignored |
| `Step`, `Request` (32) | both, nested | — | refused |
| `Settings` and its members, each options value alone | both | none: the key set | refused |
| `ordinary::Sequent`, `ordinary::Derivation` | both (read through the checks) | yes | ignored |
| `Outcome`, `ordinary::Outcome` (conflict 9), `Derivation` (R4), `batch::Answer` | written only | yes | client ignores |
| `GoalView`, `Applicable`, `Response`, `Error`, `Progress` | written only, nested | — | client ignores |

### 7.3 The forms

The running example is `A, A -o B |- B`: `⊢ ~A, A ⊗ ~B, B`, arena `0:
~A, 1: A, 2: ~B, 3: A ⊗ ~B, 4: B`, occurrences `0: ~A, 1: A ⊗ ~B, 2:
A, 3: ~B, 4: B`.

**7.3.1 Sequent.** `{"version": 1, "terms": [{"D": 0}, {"V": 0}, {"D":
1}, {"⊗": [1, 2]}, {"V": 1}], "roots": [0, 3, 4], "atoms": ["A", "B"],
"antecedents": 2}`. Tags: `V`/`D` an atom/dual atom by index; `"1"`,
`"⊥"`, `"⊤"`, `"0"`; `⊗ ⅋ & ⊕` two term indices; `! ?` one; a subterm
before its parents. `roots` as written (C1); `antecedents` absent when
zero. Reading checks indices and order, merges repeated names, refuses
a non-identifier name (HD5), counts the occurrences against the bound
before building anything. Before: `ids`, `var_dict`, no `version` or
`antecedents`. Reserved for 38: tags `P`, `N` (`[atom, [term ids]]`),
`∀`, `∃` (the body); keys `fo_terms` (`{"B": i}` a de Bruijn index,
`{"S": [symbol, [ids]]}`), `symbols` (`[[name, arity]]`), `arities`,
`binders` (names for printing).

**7.3.2 Proof.** `{"version": 1, "sequent": {…}, "nodes": [{"ax": [0,
2]}, {"ax": [3, 4]}, {"⊗": [1, 0, 1]}]}`. Nodes premises first, root
last: `ax [x, y]`, `⊗ [o, l, r]`, `⅋ [o, p]`, `1 o`, `⊥ [o, p]`, `& [o,
l, r]`, `⊕₁`/`⊕₂ [o, p]`, `⊤ o`, `! ? copy wk [o, p]`, `mix [l, r]`.
Level-1 keys besides: `mode` (written by outcomes, read by 31's
`ProofFile`), `goal` (refused: `goal_proof`). Reserved: `cuts` with the
tag `cut [o, l, r]` (34); `witnesses`, `members`, tags `∀ [o, e, p]`,
`∃ [o, w, p]` (38). Before: `proof`.

**7.3.3 Outcome** (written only).

```json
{"version": 1, "linlog_version": "0.1.0", "verdict": "proved",
 "fragment": "IMLL", "mode": "intuitionistic", "engine": "net",
 "statistics": {"nodes": 2, "memo_hits": 0, "memo_entries": 0, "splits": 0, "links": 2,
                "tests": 2, "copies": 0, "forward_copies": 0, "work": 2},
 "bounds": {…}, "options": {…},
 "sequent": {…}, "nodes": [{"ax": [0, 2]}, {"ax": [3, 4]}, {"⊗": [1, 0, 1]}]}
{"version": 1, "linlog_version": "0.1.0", "verdict": "unknown",
 "reason": {"kind": "memory_limit", "limit_bytes": 100}, "fragment": "LL", "mode": "classical", …}
{"version": 1, "linlog_version": "0.1.0", "verdict": "unprovable",
 "refutation": {"kind": "unbalanced", "atom": "A", "least": 1, "most": 1}, …}
```

- `verdict` (closed, D9). For `proved`, the proof's keys `sequent`,
  `nodes` and `mode`, so `linlog check` reads `prove --format json`;
  for a goal off the roots `"goal": [members]`, which a proof reader
  refuses (F89).
- `reason`, internally tagged so every kind can gain fields (F86):
  `stopped`, `recursion_limit {depth}`, `copy_bound {copies}`,
  `memory_limit {limit_bytes}`, `index_limit`, `work_limit {limit}`,
  `unchecked {limit_bytes}` (R138). Read back (R3).
- `refutation`, internally tagged: `exhausted`, `unbalanced {atom,
  least, most}`, `equation {formulas, needed, tensors, pars, ones,
  bottoms, mix}`, `state_equation {weights: [{atom, weight}], clauses:
  [{clause: occurrence, weight}]}` (the noun for `once`; what the
  payload certifies is R70's, the search area's). Read back (R3), no
  wildcard in the conversion (F87); 31 adds `classical {assignment}`.
- `statistics`: every counter, always written (R9); per engine their
  meaning is on `Statistics` (T5: the Horn engine's markings in `nodes`;
  `copies` the backward level, `forward_copies` the forward one, F144;
  `work` the progress summed).
- `bounds`, `options` (the forms of 7.3.9), `linlog_version`: what the
  verdict ran under (R190, A7). `fragment` is `name_in(mode)`. `net` is
  not written (it is `from_proof` of the proof). The counters above are
  today's; `work`'s value is illustrative.

Before: `"reason": "stopped"` or `{"copy_bound": 3}`, `"refutation":
"exhausted"` or `{"unbalanced": {…}}`, the mode as an object, `proof`,
none of the new keys.

**7.3.4 ProofStructure.** `{"version": 1, "sequent": {…}, "criterion":
{"mix": false}, "links": [[0, 2], [3, 4]]}`; links in the order made;
reading validates as `from_links` and accepts partial or incorrect
structures. Before: `"mix": false`. Later, each at a level: `boxes` and
the `?` nodes' placement (33), `cut_links` (34), `essential` (35),
`ordering` (36), `substitution` (38) (R8, R108).

**7.3.5 Interactive.** `{"version": 1, "sequent": {…}, "mode":
"intuitionistic", "inferences": [{"sequent": [0, 1, 4], "rule": "⊸L",
"principal": 1, "premises": [2, 1]}, {"sequent": [3, 4], "rule": "ax"},
{"sequent": [0, 2], "rule": "ax"}], "history": [0]}`. Inferences in the
session's order (`GoalId`s are their indices); an open goal is its
`sequent` alone; `rule` is `Rule::name` (ASCII spellings read too);
`history` the goals the steps closed. Reading is linear (F21) and checks
the history as each step left the arena (H17); an older build's file
reads, a newer one is refused by version (R5). Reserved: `cuts` (34,
R14), `bindings` and `members` (38, R97).

**7.3.6 Driving a session.** A `Step`: `{"position": 1, "rule": "⊸L",
"left": [0]}` (`left` absent when empty; 38 adds `witness`). A
`GoalView`:

```json
{"goal": 0, "formulas": [
  {"position": 0, "member": 0, "text": "A",     "side": "input",  "rules": []},
  {"position": 1, "member": 1, "text": "A ⊸ B", "side": "input",  "rules": [{"rule": "⊸L", "needs": "split"}]},
  {"position": 2, "member": 4, "text": "B",     "side": "output", "rules": []}]}
```

Requests and responses (32), internally tagged:

```json
{"request": "apply", "goal": 0, "step": {"position": 1, "rule": "⊸L", "left": [0]}}
{"response": "applied", "opened": [1, 2]}
{"request": "close", "goal": 2}
{"response": "closed", "goal": 2, "outcome": {"verdict": "proved", "goal": [0, 2], …}}
{"request": "draw"}
{"response": "drawing", "svg": "<svg …>", "goals": [2, 1, 0]}
```

Also `goals`, `goal`, `rules`, `split`, `undo`, `close_all` (`"results":
[{"goal": 1, "outcome": {…}}, {"goal": 2, "error": {…}}]`), `proof`; 34
adds `{"request": "cut", "goal", "formula", "left"}`. The bindings'
envelope: `{"session", "settings", "request"}` in, `{"version",
"session", "response"}` or `{"version", "error"}` out.

**7.3.7 Derivation** (written only, R4). `{"version": 1, "sequent": {…},
"mode": "intuitionistic", "sides": "two", "inferences": [{"sequent": [0,
2], "rule": "ax"}, {"sequent": [3, 4], "rule": "ax"}, {"sequent": [0, 1,
4], "rule": "⊸L", "principal": 1, "premises": [0, 1]}], "root": 2}`:
premises first, `times` when above 1, for a front end that draws its own
tree.

**7.3.8 Error, Progress.** 4.2; `{"work": 65536, "held_bytes": 12582912,
"phase": "search"}`.

**7.3.9 Options values.** `Settings::default()`, the export members
abbreviated:

```json
{"clock": {"time_limit_ms": 2000, "pool_after_ms": 100, "batch_time_limit_ms": null},
 "bounds": {"memory_bytes": 1073741824, "occurrences": 50000000, "derivation_bytes": 67108864,
            "work": null, "recursion_depth": 2048},
 "search": {"engine": "auto", "fragment": "auto", "bias": "auto", "copies": null, "forward_copies": 30,
            "memo_limit": 1048576, "test_period": "auto", "jobs": 1, "schedule": "auto", "check": true},
 "view": {"compact": "auto", "sides": "auto"},
 "styles": {"text": {…}, "latex": {…}, "typst": {…}, "svg": {…}, "png": {…}, "pdf": {…},
            "rocq": {"form": "fragment", "lemma": "certificate", "prelude": null}},
 "batch": {"mode": "classical", "cores": "auto", "workers": 1, "total_memory_bytes": 4294967296}}
```

Any subset is a file (`{"bounds": {"memory_bytes": 268435456}}`); each
member alone is its own form (`search::Options` is the `"search"`
object, R1).

**7.3.10 The batch's record** (written only, F164): one JSON Lines
record per problem, in input order, exactly one of an outcome and an
error, `name` second:

```json
{"version": 1, "name": "a.txt", "linlog_version": "0.1.0", "verdict": "proved", …}
{"version": 1, "name": "bad.txt", "error": {"code": "parse", "class": "invalid", …}}
{"version": 1, "name": "late.txt", "error": {"code": "stopped", "class": "stopped", "details": {"phase": "read"}}}
```

A client branches on `class`, never on prose. The command writes
`batch::Answer`'s form unchanged; `--stats`'s time is text (no clock).

**7.3.11 The ordinary forms** (R11). `ordinary::Sequent`: `{"version":
1, "formulas": [{"A": 0}, {"A": 1}, {"→": [0, 1]}], "atoms": ["a", "b"],
"left": [0], "right": [2]}` (tags `A`, `"⊤"`, `"⊥"`, `¬`, `∧`, `∨`, `→`,
`↔`), read through the checked constructor. `ordinary::Derivation`:
`{"version", "logic", "sequent", "inferences": [{"left", "right",
"rule", "principal": {"side": "right", "position": 0}, "premises"}],
"root"}`, read and checked. `ordinary::Outcome` (written only):
`{"version", "linlog_version", "verdict": "valid"|"not_valid"|"unknown",
"logic", "translation", "target", "linear": {…}, "derivation"?,
"reason"?}`. Reserved for 38 (D-4, D-12): the linear sequent's
first-order keys and tags, one term type for both arenas, per inference
`witness` or `eigenvariable`.

**7.3.12 Forms later steps add**, each starting at the level current
when it lands: `ProofFile` and the refutation file `{"version",
"sequent", "mode", "refutation"}` (31, R3, R71); `Request`/`Response`
(32); the cut-elimination session `{"version", "proof", "steps":
[{"cut", "case"}]}` (34, R15).

### 7.4 Every string a client meets (story 3)

| enumeration | values | |
|---|---|---|
| verdict | `proved`, `unprovable`, `unknown` | closed |
| `reason.kind` | `stopped`, `recursion_limit`, `copy_bound`, `memory_limit`, `index_limit`, `work_limit`, `unchecked` | open |
| `refutation.kind` | `exhausted`, `unbalanced`, `equation`, `state_equation` | open |
| `engine` | `focus`, `net`, `two-sided`, `additive`, `horn` | open |
| `bias`; `schedule`; `test_period` | `auto`, `rarer`, `factors`; `auto`, `turns`; `auto` or a number | open |
| fragment | `MLL`, `MLL with units`, `ALL`, `MALL`, `MELL`, `LL`; `I`-prefixed in an intuitionistic outcome | open (38: `MLL1`…) |
| mode | `classical`, `affine`, `mix`, `affine-mix`, `intuitionistic`, `intuitionistic-affine` | by level (36: `cyclic`, `lambek`, `lambek-star`) |
| `compact`; `sides`; `cores`; `form` | `auto`, `always`, `never`; `auto`, `one`, `two`; `auto`, `across`, `within`; `fragment`, `standalone` | closed |
| typst `layout`; `labels`; open goal; svg `advances` | `auto`, `curryst`, `linlog`; `upright`, `subscript`, `off`, `{"table"}`; `dots`, `bare`, `{"mark"}`, `dashed`; `euler`, `{"fixed"}`, `{"table"}` | open |
| rule | `Rule::ALL`'s 34 names (`ax` … `open`), ASCII spellings read | by level (34, 36, 38) |
| node tag; term tag | `ax ⊗ ⅋ 1 ⊥ & ⊕₁ ⊕₂ ⊤ ! ? copy wk mix`; `V D 1 ⊥ ⊤ 0 ⊗ ⅋ & ⊕ ! ?` | by level |
| `class`; `code`; `phase`; `needs` | 4.1; 4.3; `read` … `read_back`; `nothing`, `split` | open |
| `side` | `input`, `output` | closed |
| logic; translation; ordinary verdict | `classical`, `intuitionistic`, `minimal`; `affine`, `cbn`, `cbv`, `01`; `valid`, `not_valid`, `unknown` | open; open; closed |

`wire`'s documentation is this table and 7.3, and a test checks each row
against the code (`Engine::ALL`, `Mode::NAMES`, `Rule::ALL`,
`Error::CODES`, …). A JSON Schema or TypeScript declaration, if any, is
`linlog-web`'s (step 32), not a dependency of the library.

### 7.5 Where each form is pinned

`core/tests/lock/json.txt` pins every written form; one commit of its
own regenerates it for this section (`version`, T1's renames, the mode
by name, tagged reasons and refutations, the new outcome keys), with
README's JSON blocks and the command's lock. `core/tests/serialize.rs`
keeps what the lock cannot show (F90): each form read back, the
pre-release fixture of each read unchanged, `version: 2` refused by
name, an unknown key ignored by a data form and refused by an options
form, a saturated `Size` written as 2⁵³, a shared-subterm file past
`bounds.occurrences` refused by each reader.

## 8. Engines and the search's front door

### 8.1 The front door

```rust
pub fn prove(s: &Sequent, mode: Mode, options: &Options) -> Result<Outcome, Error>;
pub fn prove_within(s: &Sequent, mode: Mode, options: &Options, bounds: &Bounds,
                    stop: impl FnMut(Progress) -> bool) -> Result<Outcome, Error>;
pub fn prove_goal(forest: &Forest, goal: &[Member], mode: Mode, options: &Options, bounds: &Bounds,
                  stop: impl FnMut(Progress) -> bool) -> Result<Outcome, Error>;
pub fn engine_for(forest: &Forest, goal: &[Member], mode: Mode, options: &Options) -> Result<Engine, Error>;
/// One thread first; once `add_pool` says so, a pool of `threads − 1` (at least two) beside it;
/// the first to decide answers; one account of `bounds.memory_bytes` for both.
#[cfg(feature = "parallel")]
pub fn race(forest: &Forest, goal: &[Member], mode: Mode, options: &Options, bounds: &Bounds,
            threads: usize, add_pool: impl Fn() -> bool + Sync,
            stop: impl Fn(Progress) -> bool + Sync) -> Result<Outcome, Error>;
```

- `prove` is `prove_within` with `Bounds::DEFAULT` and `|_| false`; the
  copy bound of 3 makes it end (F42).
- `race` replaces the two `alone_first` (R85, F103): the single search
  runs on a thread of `bounds.stack_bytes()` and asks `add_pool` at its
  polls (the caller's flag, raised by its timer after
  `clock.pool_after_ms`: no clock in the library); the pool starts only
  if `Engine::parallel`; `threads` counts both, so `--jobs 2` runs two
  threads (F168); counters merge by `Statistics::add`. Its stop is
  `Fn + Sync` rather than `FnMut`, since two threads poll it.
- `prove_goal` is still **the one place an answer becomes a `Verdict`**:
  the refutation of an exhausted search, the refuters (8.4), the check
  of a proof of the roots (a refusal is `Unknown(Reason::Unchecked)`, a
  rejection `Error::Rejected`), `Outcome.goal`, `options` and `bounds`.

### 8.2 The engine interface (crate-private)

```rust
pub(crate) trait Decide {
    fn admits(&self, task: &Task<'_>) -> Result<(), Error>;      // fragment and mode (R135)
    fn decide(&self, task: &Task<'_>, options: &Options, bounds: &Bounds, account: &Account,
              stop: &mut Stop<'_>) -> Result<Answer, Error>;
}
pub(crate) struct Answer { result: Result<Option<Proof>, Reason>, statistics: Statistics,
                           net: Option<ProofStructure>, refutation: Option<Refutation> }
```

`Decide`, `Task` and `Answer` stay crate-private (free to change);
`decide` is the one entry, so a suspendable engine can replace it
(R47). Every `admits` checks the fragment as well as the mode and
refuses a first-order goal (`first_order_goal`) until the engine has a
first-order version; the dispatch returns `Error::NoEngine` where no
row takes a goal instead of `expect` (R135). Step 31 adds an optional
failure trace to `Answer`, off by default (R112).

### 8.3 `Engine`, `Verdict`, `Reason`, `Refutation`, `Statistics`

```rust
#[non_exhaustive] pub enum Engine { Focus, Net, TwoSided, Additive, Horn }
impl Engine {
    pub const ALL: &'static [Engine];
    pub const fn name(self) -> &'static str;    // "focus", "net", "two-sided", "additive", "horn"
    pub fn parallel(self) -> bool;              // whether it uses Options::jobs at all
}
impl FromStr for Engine { type Err = Error; }   // the command and the harness parse with it
pub enum Verdict { Proved(Box<Proof>), Unprovable(Refutation), Unknown(Reason) }   // closed (D9)
```

`Reason`, `Refutation`, `Statistics`: 2.4 and 7.3.3. `Reason::setting()`
names the key that lifts a limit (`search.copies`,
`bounds.memory_bytes`, `bounds.recursion_depth`, `bounds.work`), so the
command's advice and the web's highlighted field come from one table
(R129, R137). A counter a later engine needs is a field at the end of
`Statistics`, its CSV column appended in the same commit (R9, T5).

### 8.4 The dispatch as data; registering an engine and a refuter

`DISPATCH` stays a private table of rows (largest fragment, `Modes`,
`Feature`, engine), first match, the measurement of each row on
`Engine`'s docs (R87, D19); a routing feature is a `Feature` variant
computed in one polled linear pass (R88), an ordered mode a `Modes`
variant (R90). `NET_MULTIPLICITY` is a public documented constant
(R149).

**An engine** (R86): an `Engine` variant with its doc paragraph (options
read, fragments and modes, measurement), its `name` (`ALL` and `FromStr`
follow), arms in `implementation` and `parallel` (no default), a
`Decide` implementation, a `DISPATCH` row only where measured winning,
its counters as `Statistics` fields, its `--stats` arm, its
configuration in `search/reference.rs`. The command and the harness
keep no list: they parse with `FromStr` and list `Engine::ALL`.

**A refuter** (C2 C; 31 builds the first): a private table `REFUTERS`
of rows (fragment, modes, a function `(task, options, bounds, account,
stop) -> Option<Refutation>`). `prove_goal` runs them after an
`Unprovable` whose refutation is `Exhausted`, and, only with
`options.refute.after_unknown` (off by default, so every baseline
stays), after an `Unknown`; an answer counts only if
`Refutation::check` accepts it (a `debug_assert!` in every build, as for
proofs), and a refuter never changes a verdict the search gave. No
clock, no thread: the web client has it too.

## 9. Exports

```rust
pub trait Drawable: sealed::Sealed {}                      // F36
impl Drawable for proofs::Derivation<'_> {}
impl Drawable for ordinary::Derivation {}
// latex, typst, svg:
pub fn write(d: &impl Drawable, options: &Options, out: &mut impl fmt::Write,
             stop: impl FnMut(Progress) -> bool) -> Result<(), Error>;
pub fn derivation(d: &impl Drawable, options: &Options) -> String;
pub fn sequent(s: &Sequent, mode: Mode, options: &Options, bounds: &Bounds) -> Result<String, Error>;
// svg:
pub fn net(n: &ProofStructure, style: &Style, bounds: &Bounds, stop: impl FnMut(Progress) -> bool) -> Result<String, Error>;
// rocq: write(&Derivation, …) (NoCertificate before anything is written); ordinary(&ordinary::Derivation, …)
// png, pdf:
pub fn from_svg(svg: &str, fonts: &[&[u8]], options: &Options, bounds: &Bounds) -> Result<Vec<u8>, Error>;
```

- **One signature** writes anything drawable; the per-type wrappers and
  `ordinary` functions go (F36); a new target (Lean, R158) is a module
  of this shape and a `Styles` field. The sequent printers print
  two-sided by mode, through `Notation` (F82).
- **Options** are one value per output (D15, 6.4), validated before
  writing (F4, F38).
- **The SVG's ids are a contract** on `Style::ids` (R163, R168):
  `i<n>` a drawn inference, `i<n>-<p>` the formula at `Step`'s position
  `p`, `o<n>` a net's occurrence, `l<m>-<n>` a link; `b`, `d`, `j`, `c`
  reserved for boxes, doors, jumps and cuts. A session's drawing comes
  with its map from `n` to `GoalId`. Whether an interactive drawing's
  root becomes `graphics-document` with focusable targets is step 32's
  choice, a `Style` field defaulting to today's `img`.
- **Rocq** gains at 31, additively: `Options::kernel` (`Auto` keeps
  today's bytes, R139), the per-kernel prelude (`Option` from 28, R140),
  `write_proof(&Proof, Mode, …)` over the term (R152),
  `refutation(&Sequent, Mode, &Refutation, …)` (R153), `LIBRARY` beside
  `NANOYALLA` (R12), per-kernel `Unsupported` variants (R161).
- PNG and PDF stay out of the web client's first features (resvg and
  krilla unbuilt for wasm32; step 32 records R44; a browser prints the
  SVG).

## 10. How each later step enters it

Every item is new (a function, a type, a variant or field of a
non-exhaustive type, a key at a level) or a planned 0.y bump of a closed
enum; none changes a signature step 28 fixes.

**29, the comparison.** Drivers read the outcome's `verdict`, `engine`,
`statistics`, `linlog_version`, `bounds`, `options` (R172, R190); the
one-thread column is `--deterministic` (R151), the default column
`race` (R85), both in one account equal to BenchExec's limit (R18);
"outside the fragment" is class `unsupported` (R75, R135); translators
use `Walk`, `Reading::walk` and `antecedents` (R74, R245). Nothing new.

**30, the release.** Promised: `linlog`'s Rust API under semver (closed
enums with planned bumps, `cargo-semver-checks --all-features`, R215);
the wire forms by level, independent of the crate's version (a new level
is a changelog entry, never by itself a major bump); not the command's
output (D18); `linlog_cli` internal (R253); `wire::VERSION` 1 (R221).

**31, Rocq.** `ProofFile` (R7); `rocq::{write_proof, refutation,
Kernel, LIBRARY}`; the refutation file; `Refutation::check` sharing no
engine code (R124); the first refuter, `Options::refute`, `classical`
(C2); the Rocq `node` over the member type (R244); 0.1.0's term (R117).

**32, the web client.** `linlog-web` (a member, its wasm32 build a flake
check, R41, R239) exports JSON-in, JSON-or-SVG-out `prove`, `session`,
`check`, `decide_ordinary` over `from_json_within`, `serve` and 5.2's
stop; adds `Request`, `Response`, `serve`, `FeatureOff`; measures
`Bounds::BROWSER` and the wasm32 `PER_LEVEL` (D16); installs the panic
hook; runs the limit tests at 32 bits (R46, R198); ships `parse`,
`serialize`, `interactive`, `svg`, `latex`, `typst`, `rocq` (R49).

**33, MELL nets.** The box table and the `?` nodes' placement (C3 A:
jumps) as data, `Criterion` unchanged (R145), a vertex type (draft C),
reading arms in `from_proof` (R122, R254), box `NetError`s (R132),
`boxes` at a level (R8), SVG `b`/`d` ids and `Style` fields (R144, R168).

**34, cut.** `Node::Cut(OccId, NodeId, NodeId)`, `Rule::Cut` (0.2.0);
extra trees and the dual by offset (D-2, R57); `cuts` and `cut` in the
proof and session forms at one level (R13, R14); `Request::Cut` (R95);
`proofs::cut::{step, eliminate}` with bounds and stop,
`Phase::Eliminate`, its form (R15, R32, R100); R131's codes; `cut_links`
(R80, R101). The search stays cut-free.

**35, MLL engines.** Prune switches (R143), counters (R9),
`Criterion::essential`, `is_essential(&reading, stop)` (R123), an
`Engine` variant if it earns a row, its closure charged (R35).

**36, Lambek.** `Mode::ordering` and its names at one level (R51, R241,
R246); the ordered antecedent (R55); the reversed dual in
`Builder::dual` (R56); `Criterion::ordering` (R104); a `Modes` row and
an engine (R90); the divisions (a bump of `Rule`, R61);
`StepError::Exchange` (R96); R134's codes.

**37, the inverse method.** `Engine::Inverse`, a row only if earned, its
database charged (R36), polled per saturation step (R37), its limits as
`Reason` variants (R111), counters, knobs per R147.

### 10.1 Step 38, first-order logic in full

**The first step of adding quantifiers is the data model alone**: the
variants, arenas, tables, parser, printers and forms below, with every
engine and the checker refusing a first-order goal (`first_order_goal`,
`unsupported`) and the target set's columns and the journeys unchanged.
Search, the checker's rules and the views follow.

- **Atoms as predicates over terms.** `Atom` names a predicate symbol;
  `Term::Pred(Atom, ArgsId)` and `DualPred(Atom, ArgsId)` hold two `u32`
  within the 12 bytes, `ArgsId` indexing an argument list; a
  propositional literal stays `Term::Atom`, so no propositional arena or
  file changes. `Fragment::QUANTIFIERS` is set by an argument as well as
  a binder, in `Sequent::fragment`'s one pass (R52).
- **Binders in the arena.** `Term::Forall(TermId)`, `Exists(TermId)`
  (unary, dual to each other, `∀` negative, `∃` positive); the body
  locally nameless over a second arena `fo_terms` of `Bound(u32)` (de
  Bruijn) and `App(Symbol, ArgsId)` with a `symbols` table of `(name,
  arity)`, hash-consed, so `∀x.p(x)` and `∀y.p(y)` are one term; names in
  a side table for printing; one term type for `Sequent` and
  `ordinary::Formulas` (D-4). `Walk` gains a binder stop and visits
  arguments with `Between { index }`; nothing recurses over a term
  (R43); `Bounds::term_nodes` bounds the unfolded size on every reader
  (R250).
- **The forest unchanged**: a binder is a unary occurrence, literal
  groups by predicate and sign, and `dual_literals` is the one place
  instances are compared (F58).
- **The substitution beside the forest.** An instance is `(OccId,
  FrameId)`, a frame the terms the enclosing binders are bound to,
  hash-consed in a search arena; a `Member` at or above `forest.len()`
  indexes the instance table of its proof or session. The sequent's
  arenas are read-only during search.
- **The trail in the engines.** One iterative unifier (`search::unify`)
  with bindings in the search arena and a trail (a mark per choice
  point, undo by truncation), an occurs check, eigenvariable levels; the
  focused engine generic over its zone (`Engine<Z: Zone>`, section 11);
  only ground stable sequents memoized; everything charged and polled
  (R39, R106, R107, R109).
- **Witnesses in proofs.** `Node::Forall(OccId, Eigen, NodeId)` and
  `Exists(OccId, Witness, NodeId)` in 16 bytes, a witness arena owned by
  the `Proof`, the checker's frame pass and eigenvariable set only when
  the forest has binders (R125, R126); the forms' `witnesses`,
  `members`, `bindings`, `Step::witness`, `Needs::Witness` at one level
  (R16, R17, R97); `Reason` variants for term size and instances (R38).
- **Untouched for the propositional case**: `Term` 12 bytes, `Kind` one,
  `Node` 16, `OccId` and `Member` one `u32`, the forest's arrays,
  `OccSet`, the memo keys, `Decide`, `Answer`, `Verdict`, the dispatch's
  rows (one added), every propositional form's bytes, the target set's
  counters (D17).

## 11. The spike

In a throw-away jj workspace, never merged; three layers, measured after
each, so a cost is attributed to its cause.

1. **The data, inert.** `Term` gains `Pred`, `DualPred`, `Forall`,
   `Exists` and `Kind` the four tags, `assert!(size_of::<Term>() ==
   12)`; `Sequent` empty `fo_terms`, `symbols` and argument lists;
   `Fragment::QUANTIFIERS`; `Node` gains `Forall`, `Exists` within 16
   bytes and `Proof` an empty witness arena; `Member` replaces `OccId`
   in the signatures of 3.4; every match lists the new variants (engines
   refuse them in `admits`, printers write them, the parser never makes
   them). *Expected*: no counter changes by construction; instruction
   counts within the ratchet's tolerance.
2. **The zone parameter.** `search::focus::Engine<Z: Zone>`, a
   crate-private `trait Zone` with exactly what the engine does with a
   context (insert, remove, contains, iterate, the memo key's bytes, the
   classes' lookup), implemented by `Context`; `Key<Z>` and `Classes`
   follow; a stub framed zone in a test only, so the genericity is real
   while the binary holds the propositional instance alone. The one
   layer whose cost is not zero by construction (D-7).
3. **The measurement.** `bench/targets.sh spike-a` against
   `after-bias.csv`: `verdict`, `nodes`, `splits`, `memo_hits`,
   `memo_entries` identical on every decided row; pinned CPU time on
   rows over a second within 2 % (step 28's constraint), a row outside
   rerun on the same core; `ratchet` on stage 0's journeys within
   tolerance; the release `.text` size beside the baseline's.

**Accept** layer 2 when all hold: step 38 instantiates the generic
engine with a framed zone, and `api.md` names the trait. **Reject** it
when a counter differs (the spike is wrong: find why) or the time is
over 2 % after the rerun: step 38 then duplicates the engine (`focus`
concrete, `focus_fo` generic) as D17 allows, and `api.md` says so. Step
28 lands the free parts of layer 1 that it needs now (`Member`, the size
assertion, the exhaustive matches) and leaves the variants to 38. The
change of every poll site to the progress stop (5.2) is measured the
same way in its own commit of stage 3.

## 12. Findings answered

| finding | where |
|---|---|
| F1 non_exhaustive; F84, F85 Term size, wildcards | 2.5, P6, 3.1 |
| F4, F38 option values unchecked; F5 sequent drawing unbounded | 6.4, 9, 5.6 |
| F6 families panic | 4.3 |
| F7 Mode; F40, F64 builder names, `must_use` | 3.5, 6.1 |
| F8, F30, F31, F33, F34, F48, F50 the error family | 4 |
| F10 bools for a mode and a criterion | 3.8, 3.10 |
| F12 no stop on long calls; R22 | 5.6 |
| F13 ordinary constructors | 3.11 |
| F15 Rocq prelude | 6.4 |
| F16, F27 readers before or without the bound | 3.1, 5.5 |
| F18, F61 forms undocumented, unknown keys three ways | 7.1, 7.2 |
| F19 member type | 3.4 |
| F21, H17 reading a session back | 3.9, 7.3.5 |
| F22 `apply`'s arguments; F67, C3 empty Mix premise | 3.9 |
| F23 `close_with` grafts a foreign proof | 3.9 |
| F24, C1, H9, H10 written order and sides | 3.1, 3.6 |
| F25 atoms called variables | 3.2 |
| F26, T1 versions and key names | 7.1 |
| F36 export wrappers; F82 Notation | 9 |
| F42 the command's recipe | 6.5 |
| F49 `from_proof`'s wildcard | 3.10 |
| F51 needless `pub`; F203, R253 `linlog_cli` | 2.3 |
| F53 ordinary forms | 3.11, 7.3.11 |
| F58 axiom partners in eight places | 3.3 |
| F62, F63 three bounds values, several names | 5.1 |
| F65, F70 lower bound in the error, saturated `Size` | 3.8, 7.1 |
| F66 reading recomputed; F68, R92 `close_all`; F69 two numberings | 3.9, 3.4 |
| F73, F74, R40 batch workers and cancel | 6.3 |
| F76–F79 names and form of the options | 3.5, 6, 8.3 |
| F80, F86, F87, F88 refutation and reason forms, statistics proxy | 7.3.3, 8.3 |
| F81 test period, time limit | 6.2, 6.3 |
| F89 outcome of a goal | 3.7, 7.3.3 |
| F90 `serialize.rs` pinning twice | 7.5 |
| F103, F168, R85, R18 the race | 8.1, 5.1 |
| F136, R243, R21 the stop and the work budget | 5.2, 5.3 |
| F139, R138 a proof found and not checked | 4.1, 8.1 |
| F144, T5 statistics | 7.3.3, 8.3 |
| F164, F170 batch records, `check`'s JSON | 7.3.10, 4.2 |
| H18 the pool's stack | 5.4 |
| H19, HD5 atom names; HD1; HD3 | 3.1, 4.3 |

**The register by section** (the room is made; "later" steps build it):
R1 6.2, 7.3.9 · R2 R3 7.3.3, 8.3 · R4 7.3.7 · R5 3.9, 7.3.5 · R6 7.1 ·
R7 3.7 · R8 7.3.4 · R9 8.3 · R10 3.5 · R11 3.11, 7.3.11 · R12 9 · R13
R14 7.3.2, 7.3.5 · R15 7.3.12 · R16 R17 7.3.1, 7.3.2, 10.1 · R247 7.1 ·
R18 5.1 · R19 5.4 · R20 8.4 · R21 5.3 · R22 R23 5.6 · R24 3.11 ·
R25 3.10 · R26 9 · R27 5.5 · R28 R29 5.1, 3.3 · R30 5.2 · R31 R32 10
(34) · R33–R35 10 (35) · R36 R37 10 (37) · R38 R39 R250 10.1 · R40 6.3
· R243 5.2 · R41 R42 10 (32), P9 · R43 P9, 10.1 · R44 9 · R45 5.1, 5.4
· R46 10 (32) · R47 5.3 · R48 6.3 · R49 6.2 · R50 2.5 · R51 3.5 · R52
3.5, 10.1 · R53 R54 3.1, 3.6 · R55 R56 10 (36) · R57 R58 3.3, 10 (34) ·
R59 3.1 · R60 R61 3.8, 10 · R62–R67 10.1 · R68 9 · R69 3.3 · R70 R71
7.3.3 · R72 8.4 · R73 6.3 · R74 R75 10 (29) · R76–R84 10 (33, 35, 37) ·
R241 3.5 · R244 3.4 · R245 3.6 · R248 P6 · R249 3.11 · R251 3.2 · R85
8.1 · R86–R88 8.4 · R89 R90 10 · R91 R93 R94 3.9 · R95–R97 3.9, 10 ·
R98–R105 10 · R106–R109 10.1, 11 · R110 R111 10 (37) · R112 8.2 · R113
3.11 · R114 6.2 · R115 8.4 · R116 6.2 · R117–R128 R246 R254 3.7, 3.10,
10 (draft B details the checker) · R129–R136 4 · R137 8.3 · R138 4.1 ·
R139 R140 R141 6.4, 9 · R142 5.1, 6.5 · R143–R147 10 · R148 6.3 · R149
6.2 · R150 6.3 · R151 5.3 · R152–R171 R242 9, 10 · R183 6.6 · R190
7.3.3 · R215 R220 R221 10 (30).

## 13. Decisions for the author

### 13.1 The open ones, on their recommended answers

| # | provisional | what the other answer changes here |
|---|---|---|
| C1 | written order canonical at 28 | B: roots sorted until 36, and the written sides a flag per root rather than a count (sorting mixes them; H9, H10 still need them); 36's change a level of its own. C: a `Sequent`'s meaning depends on how it was built, which this design has no place for. |
| C2 | refuters in the library, after an `Unknown` behind an option | A: no `REFUTERS` or `Options::refute`; the classical refutation is the exporter's and no outcome carries it. |
| C3 | no nullary Mix; `StepError::EmptyPremise` | B: a `Node` variant and tag at 28 (cheaper before 0.1.0 than at a bump); the empty goal closable. |
| T1 | rename keys with the version | keep `ids`, `var_dict`, `proof`, `mix`, `once`; only `version`, the tagged reasons and the new keys change; 7.1's aliases go. |
| T2 | owned forests | `Arc<Forest>`: no wire change, cheaper clones in the bindings. |
| T5 | shared counters, documented per engine | per-engine counters: `statistics` an object per engine, the CSV changes, a level of the outcome. |
| T3, T4, T6, T7 | as recommended | nothing here (R130's no-panic rule stands either way). |
| HD1 | refuse several conjectures | conjoin; no `several_conjectures`. |
| HD2 | a `.spec` file is affine | refuse without `--affine`: an `invalid` error. |
| HD3 | NFC when reading | code-point identity, stated in the syntax's docs. |
| HD4 | `load` refuses another question | the command's concern only. |
| HD5 | refuse non-identifier atom names | print them quoted; no `atom_name`. |

### 13.2 What this draft adds

| # | choice | alternative set aside |
|---|---|---|
| A1 | one global wire level as `version`, the lowest a reader needs | a version per form bumped on any change: 0.1.0 would refuse an unchanged propositional proof of 0.3.0, and an outcome read as a proof would carry two versions |
| A2 | a mode on the wire is its name | the object with `serde(default)` and an `ordering` key (R10), a level of its own |
| A3 | `serde_json` in the library behind `serialize` (`from_json_within`, `to_json`, line and column, `Settings::set`, R141) | format-agnostic `ReadWithin` only; the command and `linlog-web` each keep the dotted keys |
| A4 | `Bounds` beside `search::Options`; `Settings` with `Clock` the one value a front end holds | `Options::bounds` as a field: one argument fewer, but a check's bounds in the search's options |
| A5 | the error family flattened (`ViewError`, `WriteError`, `RenderError`, `svg::TooLarge` gone; `Refusal` → `StepError`), six classes | sub-types kept as return types with `From` and a `code()` each |
| A6 | the stop is `impl FnMut(Progress) -> bool`, no trait, no `FnMut() -> bool` form | a sealed trait with a blanket impl for `FnMut() -> bool` and a wrapper for progress closures |
| A7 | every outcome names `linlog_version`, `bounds`, `options` | only on request: shorter README blocks, stored outcomes untraceable (the version string changes README at each release unless the readme test masks it) |
| A8 | a saturated count is written as 2⁵³ | a decimal string for `Size` (R247's second option) |
| A9 | `Request`, `Response`, `serve` at 32, their types at 28 | at 28, so `interact` speaks the protocol at once |
| A10 | the written sides as `antecedents` (H9, H10, R54, R74) | refuse every intuitionistic input with a goal ambiguous among `⊤`/`0` roots, which refuses `~a ⊢ ⊤` with `⊢ a, ⊤` |
| A11 | `Settings::default()` is the command's behaviour; `search::Options::default()` keeps the copy bound of 3 | one default, the library's then deepening without end under `prove` |
