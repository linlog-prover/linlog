# linlog's API after step 28: draft C (engines, calculi and quantifiers first)

Draft C of `plan/notes/api.md` (step 28, stage 2). It designs the whole
public surface, and is most detailed where steps 35 to 38 enter it: the
engine interface and its two plug-in kinds, the stop and the memory
account every engine polls and charges, the MLL engines, the ordered
calculi, the inverse method, and first-order logic in full, with the
spike that decides the one cost that is not zero by construction (D-7).

It starts from the recommended answers of C1 to C3, T1 to T7 and HD1 to
HD5 (`plan/reports/28-audit.md`); section 13 says, for each, what the
other answer would change. Claims about the code were checked against the
tree of 2026-10-09 unless marked "(unverified)". Sizes marked "probe"
were measured on replicas of the enums in a scratch crate
(`Term` with the four reserved variants 12 bytes, `Node` with `Cut`,
`Forall`, `Exists` 16 bytes, the first-order term node 12 bytes,
`ordinary::Node` with its three reserved variants 12 bytes), not on the
crate itself.

## 1. Principles

**P1. Values own what they describe; views borrow.** `Sequent`, `Forest`,
`Proof`, `ProofStructure`, `Interactive` and the ordinary values own
their data; `Reading`, `Derivation` and the printers borrow. One rule for
forests (T2): a value that outlives its call owns its forest, a call
borrows it. No `Arc` in a public type: it would cost every caller, and no
journey shows the clone.

**P2. One id space per kind, extended by offset.** Every index is a `u32`
newtype (D3). Where a later step needs more members of a space, they
*continue the numbering* into a side table: members past `forest.len()`
are instances (3.4), a proof's or a search's first-order terms continue
the sequent's (3.9), cut trees continue the forest's occurrences (34),
net vertices past the forest are instances of `?` subtrees (33). Such an
id stays one integer on the wire. *Reason:* the propositional case is
the prefix, so its values, JSON and counters do not change (D17).

**P3. The core enums are closed; what grows is open.** `Term`, `Kind`,
`Node`, `Rule`, and `Verdict`, `Side`, `Sign`, `Polarity`, `Position`
stay exhaustive: a new variant is a compile error at every match, which
is wanted, and a 0.y bump (0.2.0 at 34, 0.3.0 at 38, in the changelog's
policy line). Every other public enum, and every struct with public
fields a later step extends, is `#[non_exhaustive]` before the tag (F1,
R50). Inside the crate, matches over closed enums name their variants
(`clippy::wildcard_enum_match_arm` denied on `search/`, `export/`,
`ordinary/`, `nets/`, `sequents/`, `occurrences/`, a refusing shape
reader keeping one `#[expect]`; R248, D-11).

**P4. Errors are one family; a refusal is a kind of its own.** Every
error converts into `Error`, whose `code()` and `class()` (input fault,
refusal, defect) a program branches on, never English (section 4).

**P5. Every long call takes one stop and one bounds value**: `impl Stop`,
polled at a bounded interval of work with that work (`Progress`), and the
caller's `Limits`; what grows with the input is charged to one account
per call and refused at its bound (section 5).

**P6. Options are plain data with a wire form**: `#[non_exhaustive]`
structs with public fields, `Default`, `Clone`, `PartialEq`, serde
`(default, deny_unknown_fields)`, `#[must_use]` `const` builders and
`DEFAULT_*` constants (D15, D16, F76 to F79); runtime handles (`Pool`)
are skipped on the wire; presets are named values.

**P7. Engines and refuters are the crate's own plug-ins.** `Decide` and
`Refute` stay crate-private: the choice of engine is the library's,
measured, in one table (D7, D19). Public is what a front end needs:
`Engine`'s names, `ALL`, `parallel()`, `engine_for`, the statistics.

**P8. Wire forms carry a version and grow by keys**; no new meaning rides
on an old tag (section 7).

**P9. The propositional case does not pay for quantifiers (D17).**
First-order data lives in tables empty for propositional input,
first-order code in a monomorphisation or a path the propositional
dispatch never enters; size assertions on `Term`, `Node`, `Kind` and
`ordinary::Node` make a regression a compile error; the spike (11)
measures generics before step 38 commits to them.

## 2. The public surface after step 28

### 2.1 The module tree

```
linlog
├── sequents   Sequent, Term, TermId, Kind, Atom, Formula, Walk, Visit
│   └── (fo: reserved for step 38: FoTerm, FoTermId, FunctionId, ArgsId)
├── fragment   Fragment, Mode
├── occurrences Forest, OccId, Member, Sign, Polarity, Reading, Position,
│               IllFormula, ShapeError
├── proofs     Proof, Node, NodeId, Side, check (CheckError, Fault,
│              Dyadic), derivation (Derivation, Inference, InfId, Rule,
│              ViewOptions, Compact), size (Size), style (Labels,
│              OpenGoal), fmt (TextOptions), interactive (Interactive,
│              GoalId, Step, Refusal)
├── nets       ProofStructure, NetError, Scratch
├── search     prove, prove_until, prove_goal, engine_for, Options, Engine,
│              Bias, Schedule, Outcome, Verdict, Refutation, Reason,
│              Statistics, Pool (parallel), race (parallel), batch
├── limits     Limits, Stop, Progress, Phase, WithProgress, never
├── errors     (private module) Error, ErrorCode, ErrorClass, ParseError
├── export     latex, typst, svg, png, pdf, rocq, OutputOptions
├── ordinary   Sequent, Formulas, Node, NodeId, Logic, Translation,
│              Options, Image, Derivation, Inference, Rule
├── lltp, mist, families   (feature `parse`)
└── serialize  (private) the proxies, one per form
```

`lib.rs` re-exports at the root what the common path needs (`Sequent`,
`Term`, `TermId`, `Kind`, `Atom`, `Formula`, `Fragment`, `Mode`, `Forest`,
`OccId`, `Member`, `Reading`, `Proof`, `Node`, `NodeId`, `Rule`,
`Derivation`, `Inference`, `ViewOptions`, `Interactive`, `Step`,
`ProofStructure`, `Options`, `Engine`, `Outcome`, `Verdict`,
`Refutation`, `Reason`, `Statistics`, `Limits`, `Stop`, `Progress`,
`Error`, `ErrorCode`, `ParseError`, `prove`, `prove_until`,
`prove_goal`); the rest is reached through its module, which keeps the
front page short (C-CRATE-DOC asks for the common path there).
Becomes private: `OccSet`, `Forest::empty_set`, `root_set` (F51); the two
`Described` types give way to one `Describe` (4). `linlog_cli`'s library
is documented as the binary's internals, outside semver (R253, F203).

### 2.2 Before and after

A later session searches this table. "Removed" items have no alias
(D18, S17).

| before | after | finding |
|---|---|---|
| `Term::{Var, DualVar}`, `Kind::{Var, DualVar}`, `Sign::{Var, DualVar}` | `Term::{Atom, DualAtom}`, `Kind::{Atom, DualAtom}`, `Sign::{Plain, Dual}` (discriminants 0 and 1 kept: lists are `2·atom + sign`); `Atom` documented as a predicate symbol | F25, R251, R62 |
| `Error::InvalidVariableIndex` and the five positional index errors | `Error::IndexOutOfBounds { space: Space, index, len }`, `Error::NotTopological { space, index, parent }` | F25, F31 |
| `Error::{NetFragment, NetMode, NetGoal, EngineMode, NotAdditive, NotHorn}` | `Error::EngineRefused { engine, because: NotTaken }`; a structure's own refusal is `NetError::Fragment` | F48, R135 |
| `Error::NoEngine` (never built), `.expect` in `dispatch` | built by `dispatch` | F140 |
| `Error::Unchecked` from `prove_goal` | `Verdict::Unknown(Reason::Unchecked { limit })`; the error stays for a graft | F139, R138 |
| `Mode { pub intuitionistic, pub affine, pub mix }`, builder `affine()` | private fields; `is_intuitionistic`, `is_affine`, `has_mix`; `with_affine`, `with_mix`; `name`, `NAMES`, `FromStr`, `check` | F7, F40, R51, R241 |
| `Fragment::ALL`; `LL`: "every connective" | `Fragment::ADDITIVE` (named `ALL` on the wire); `LL`: "every propositional connective", bit 32 reserved | F56, D-4, R52 |
| roots sorted by `optimize` | written order kept; `Sequent::antecedents` | C1, F24, R53, H10 |
| `Sequent::verify_integrity` | `Sequent::check` | F83 |
| `FromStr` for `Sequent` only | plus `Sequent::parse_within(text, &Limits)` | F16 |
| `Forest::within(&s, u64)`, `from_owned(s, u64)`, `TryFrom<Sequent>` | `within(&s, &Limits)`, `from_owned(s, &Limits)`; `TryFrom` removed | F27, F63 |
| `OccSet`, `Forest::empty_set`, `root_set` public | crate-private | F51 |
| `OccId` in every member list and every `Node` operand | `Member` (3.4) | F19, R244, D-1 |
| `Proof` without a mode | `Proof::mode() -> Option<Mode>`, the optional `"mode"` key | R7 |
| `check_within(mode, Option<u64>)` | `check_with(mode, &Limits, impl Stop)` | F12, R23 |
| `derivation_size(two_sided: bool)`, `derivation_size_within`, `derivation_with(&ViewOptions, stop)`, `two_sided_derivation_with` | `derivation_size(&ViewOptions, &Limits, impl Stop)`, `derivation_with(&ViewOptions, &Limits, impl Stop)`; `ViewOptions { sides: Sides, compact }`, its bounds moved to `Limits` | F10, F62, F63 |
| `Inference`'s public fields | `#[non_exhaustive]`, accessors (`sequent() -> &[Member]`, …) | R61, R66 |
| `Rule::ALL: [Rule; 34]` | `&'static [Rule]` | D-5, R60 |
| `Interactive::apply(goal, position, rule, left)`, `goal(InfId) -> &[OccId]`, `new(&s, mode)`, `close_all -> Result<Vec<…>>` | `apply(goal, &Step)`, `goal(GoalId) -> &[Member]`, `new(&s, mode, &Limits)`, `close_all -> Vec<(GoalId, Result<Outcome, Error>)>` | F22, F69, F27, F68 |
| `ProofStructure::new(forest, mix: bool)`, `from_links`, `from_proof(&p, mix)`, `is_correct()`, `sequentialize()` | `new(forest, Mode)`, `from_links(forest, Mode, links)`, `from_proof(&p, Mode, &Limits, stop)`, `is_correct(stop)`, `sequentialize(&Limits, stop)` | F10, F12, F49, R79 |
| `search::Options` private fields and setters, `copies`, `engine`, `fragment`, `test_period`, `memory_limit`, `occurrence_limit`, `recursion_limit` | public fields and serde (6): `copy_bound`, `force`, `assume`, `test_period: Period`, `limits: Limits` | F77, F78, F79, R1 |
| `Engine::parallel` (negative match) | exhaustive; `Engine::ALL`, `name`, `FromStr`, serde both ways | F76, F141, R86 |
| `Statistics`'s hand-listed proxy | `derive(Serialize)`; `forward_copies`, `work` added | F88, F144, R243 |
| `Refutation::Unbalanced { atom, name, … }` | `{ atom: String, least, most }`; `Refutation`, `Reason` read back | F80, R3 |
| every stop `impl FnMut() -> bool` | `impl Stop` (such closures still are) | F136, R243 |
| `batch::{Options, Plan, Problem, Answer}` | `#[non_exhaustive]`, serde on `Options`, `Problem::overrides`; `Answer` → `batch::Report` | R150, R73, F56 |
| `rocq::Options::prelude: String` | `Option<String>` | F15, R140 |
| `export::*::{derivation, write, ordinary}` | one `write` and one `to_string` per target over a sealed `Drawable` | F36 |
| `ordinary::Sequent::new -> Self`, `Formulas::add` unchecked, `Translation::target() -> &str`, `Image::read_back(&Derivation)`, `Derivation::check()` | `-> Result`, checked, `-> Target`, `read_back(&Proof, &Limits, stop)`, `check(&Limits, stop)` | F13, F12, R249, D-12 |
| `Family::instance` panics | `-> Result<Instance, Error>` | F6 |
| JSON `ids`, `var_dict`, `proof`, `mix`, `memory_limit`, `once` | `roots`, `atoms`, `nodes`, `mode`, `limits.memory_bytes`, `single_use`; the old names read as aliases | T1, F26 |

## 3. The data model

### 3.1 `Sequent`: the arena, the atoms, the written order

```rust
/// A one-sided sequent in negation normal form: root formulas over an
/// arena of shared subformulas, in the order they were written.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Sequent {
    terms: Vec<Term>,              // topological: a term names only earlier terms
    roots: Vec<TermId>,            // in written order (C1): never sorted
    atoms: Vec<String>,            // predicate symbols; distinct names
    antecedents: Option<u32>,      // roots written left of ⊢, from the first; None: written one-sided
    // step 38, private and empty for propositional input (section 3.9):
    // arities, functions, fo, args, binders
}
```

- **The canonical form is the written one (C1, option A).** `optimize`
  merges equal atoms and terms and drops unreachable ones; it never sorts
  `roots`. Parse, JSON, `add`, `Display` and `Forest::roots()` keep the
  roots as given. `Eq` and `Hash` are structural (`⊢ A, B` and `⊢ B, A`
  are two values); "the same sequent in a mode" (up to permutation, or to
  rotation cyclically) is the search's question (`is_roots`), so a
  `Sequent` means one thing in every mode (C1 rejects option C). The
  commit that stops sorting regenerates the snapshots and README blocks
  whose roots were not ascending and names them; `ordinary::translate`
  pairs roots by position.
- **The written sides are kept** (H9, H10, R54): `antecedents` counts the
  roots written left of `⊢`, which the parser lowers first; `lltp::read`
  (through the parser), `mist` and `translate` set it too. The reading then takes the goal from what
  was written: one root right of `⊢`, else `ShapeError::Succedents(n)`
  (`|- a, top` under `-i`, H10). With the sides known, an implication's
  antecedent is the factor the lowering put there (the left one under
  D1), so the second, symmetric reading that turned `(A -o bot) -o bot |-
  A` into `1 ⊸ (A ⊗ 1) ⊢ A` is not tried and the input is refused as
  `bot |- A` is (H9). `None` (a JSON sequent without the key, `add`)
  keeps today's choice.
- **A name is an atom, until step 38 makes it a name and an arity**
  (fo-linear §1): `p/0` and `p/1` are two predicates; `atom(name)` stays
  the nullary lookup.
- **Readers name their bound**: `Sequent::parse_within(text, &Limits)`
  refuses at the first term past `limits.occurrences` (F16); `FromStr` is
  that under the defaults. Atom names read from JSON are identifiers of
  the text syntax (HD5, H19), normalized to NFC on every path (HD3, H8).
- `Sequent::check()` (was `verify_integrity`, F83): indices, the order,
  at 38 scope and arities.
- A public builder (R59) waits for its first callers, cut (34) and the
  first-order families (38), since binders shape it: `sequents::Builder`,
  interning as the parser does, so a built sequent equals the parsed one.

### 3.2 `Term`, `Kind`, `Atom`

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Term {                       // closed (P3)
    Atom(Atom), DualAtom(Atom),       // was Var, DualVar
    One, Bot, Top, Zero,
    Tensor(TermId, TermId), Par(TermId, TermId),
    With(TermId, TermId), Plus(TermId, TermId),
    Bang(TermId), Quest(TermId),
    // reserved for step 38 (0.3.0), each two u32 at most:
    // Pred(Atom, ArgsId), DualPred(Atom, ArgsId), Forall(TermId), Exists(TermId)
}
const _: () = assert!(size_of::<Term>() == 12);   // with the four: 12 (probe)

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Kind {                       // closed; discriminants of today kept
    Atom, DualAtom, One, Bot, Top, Zero, Tensor, Par, With, Plus, Bang, Quest,
    // step 38 appends: Forall, Exists  (a Pred is Kind::Atom, a DualPred Kind::DualAtom)
}
const _: () = assert!(size_of::<Kind>() == 1);
```

- A literal with arguments keeps the literal kinds (`Kind::Atom`,
  `Kind::DualAtom`), so every place that reads a literal by its kind reads
  a predicate literal as one (impact-quantifiers §2.1); what tells the
  two apart is the term and the fragment's bit (3.5), never the kind.
- `Term::dual` and `Kind::dual` stay the order-keeping duals of D1
  (`(A ⊗ B)⊥ = A⊥ ⅋ B⊥`); `∀`/`∃` will swap like `!`/`?`. The ordered
  calculi of step 36 read their order through the reading (10.3), so the
  stored lowering does not change (D1 stands).
- `Term::atom()` returns the predicate of every literal (nullary or not);
  `Term::operands()` stays the *formula* children (a predicate's
  arguments are terms, not occurrences, and never reach the forest).
- `Atom(u32)`, `TermId(u32)`: unchanged newtypes (D3).

### 3.3 `Forest`: its contract

The forest is the numbering everything else names; its doc states three
promises, each pinned by a test (R69, D-2):

1. **The numbering is a public contract.** Depth-first preorder over the
   roots in written order, the left subterm before the right, a binder's
   body `o + 1`. A foreign checker (the Rocq library) may recompute it;
   a test recomputes it independently of `Forest::build` on the doc
   example and on generated sequents.
2. **`roots()` is the sequent's conclusion**, the occurrences of the
   written roots in their order; `ids()` is every occurrence. A later
   step may number trees past the conclusion's occurrences (step 34's
   cut formulas, `A` then `A⊥`, so that `dual(x) = root(A⊥) + (x −
   root(A))` for the commutative dual), which `roots()` never lists and
   `Forest::conclusion_len()` bounds; code that means "the goal is the
   sequent" compares with `roots()`, code that means "every occurrence"
   uses `ids()`. At 28 this is documentation.
3. **Literal lists are candidates, not partners.** `literals(atom,
   sign)` lists every literal occurrence of a predicate symbol and sign;
   whether two literals close an axiom is one predicate,
   `Forest::are_duals(x, y)` (same atom, opposite signs; at 38 its
   instance form lives on `Instances`), which the eight places that pair literals call
   (checker, oracle, `nets::dual`, the additive path, the net engine, the
   focused engine's `initial` and `dual_from`, the Horn engine's places;
   F58, D-4). The checker calling a forest accessor shares no engine code.

Constructors take the bounds value (`Forest::within(&s, &Limits)`,
`from_owned`); `TryFrom<Sequent>` goes, since it cannot take one (F27).
`Forest::formula(o)` prints the formula at `o`; from step 38 it names the
bound variables of the binders above `o` from the forest's parents.

### 3.4 `Member`: what a sequent's member is

```rust
/// A member of a sequent: an occurrence of the forest or, past the
/// forest's occurrences, an entry of an instance table that a proof, a
/// session or a search owns (an occurrence under bound terms, step 38;
/// a copy's instance, step 33). Below `forest.len()` a member *is* the
/// occurrence with that id.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize), serde(transparent))]
pub struct Member(u32);

impl Member {
    pub const fn new(raw: u32) -> Self;
    pub const fn get(self) -> u32;
    pub const fn index(self) -> usize;
    /// The occurrence, when the member is one of the forest's.
    pub fn occurrence(self, forest: &Forest) -> Option<OccId>;
}
impl From<OccId> for Member { /* the same number */ }
```

- **Every public list of a sequent's members is `[Member]`** (F19, R244,
  D-1): `Inference::sequent`, `Dyadic::{theta, gamma}`,
  `Interactive::goal`, `prove_goal` and `engine_for`'s goal, the session
  JSON's `sequent`, the command's goal line, and the operand of every
  `Node` variant (3.6). On the wire a member is one integer, so no form
  changes shape (P2). Ids are ordered by number, so "ascending ids with
  repeats" stays the order of a commutative sequent; under binders, ties
  between instances of one occurrence are ordered by the instance
  table's order, which is a function of the proof, not of the order a
  pool created them (impact-quantifiers §2.3).
- **Why one integer and not a pair** (fo-linear's `(OccId, frame)`): a
  pair changes the shape of `ax: [x, y]`, of the session's `sequent`
  and of every stored file; `Node` stays 16 bytes only with a one-`u32`
  operand; and the propositional case is then the identity. The table a
  member indexes is the place the frame lives (3.9).
- Engines keep `OccId` inside: a propositional engine's goal is all
  occurrences (the front door converts, O(goal)), and the first-order
  engine resolves members through its table.

### 3.5 `Mode`, `Fragment`, `Reading`

```rust
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct Mode { intuitionistic: bool, affine: bool, mix: bool }
//   step 36 adds, private: order: Order, empty_antecedents: bool

impl Mode {
    pub const CLASSICAL: Self;  pub const INTUITIONISTIC: Self;
    pub const fn is_intuitionistic(self) -> bool;  // getters
    pub const fn is_affine(self) -> bool;
    pub const fn has_mix(self) -> bool;
    #[must_use] pub const fn with_affine(self) -> Self;   // builders
    #[must_use] pub const fn with_mix(self) -> Self;
    /// The word of the mode, read back by FromStr: classical, mix,
    /// affine, affine-mix, intuitionistic, intuitionistic-affine.
    pub const fn name(self) -> &'static str;
    pub const NAMES: &'static [&'static str];
    /// Refuses a combination no calculus has (intuitionistic with Mix
    /// today; at 36 an ordered mode with weakening or Mix).
    pub fn check(self) -> Result<(), Error>;
}
impl FromStr for Mode { type Err = Error; }   // the error lists NAMES
```

- One table of words in the library replaces the command's and the
  harness's (R241); `affine-mix` names weakening with Mix, so the
  harness's `mix-affine` and the command's lossy `affine` go; `Display`
  stays the prose the command prints. `Modes::take` destructures the
  whole mode, so a field added at 36 is a compile error there (F140).
- JSON: `{"intuitionistic":…,"affine":…,"mix":…}`, every key under
  `serde(default)` (R10); a key added later is written only when not its
  default, so commutative files stay byte-identical.

```rust
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Fragment(u8);   // bits 1, 2, 4, 8, 16 used; 32 reserved
```

- The named constants `MLL` … `LL` are **propositional** (`LL`: "every
  propositional connective", D-4). Bit 32 is reserved for step 38's
  `QUANTIFIERS`, set by a binder *and* by a predicate with arguments (a
  ground `p(a)` against `~p(b)` must reach no propositional engine,
  impact-quantifiers finding 1), computed from the terms; its names
  append `1` (`MLL1`, `MLL1 with units`, `ALL1`, `MALL1`, `MELL1`, `LL1`,
  `I…` intuitionistically). Ordering is no connective class (the Lambek
  divisions are `⅋`), so no bit for 36.
- The Rust constant `Fragment::ALL` becomes `ADDITIVE` (still `ALL` on the
  wire and in `Display`), so it no longer reads as "every" (F56).
- `Fragment::has_nets()` is the one predicate of where nets exist (F28,
  R133), for `ProofStructure::new`, the net engine and the command.

`Reading<'a>` keeps borrowing its forest (P1); `Interactive` stores the
positions it computed once (`Box<[Position]>`, a byte per occurrence),
not a `Reading` (R93, F66). At 36 it gains the planar order (10.3); at 38
`∀` and `∃` keep their position in both (fo-linear §1).

### 3.6 Proofs: `Proof`, `Node`, `Rule`, `Derivation`, `Inference`, `Interactive`

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Node {                                    // closed (P3)
    Ax(Member, Member),
    Tensor(Member, NodeId, NodeId), Par(Member, NodeId),
    One(Member), Bot(Member, NodeId),
    With(Member, NodeId, NodeId), Plus(Member, Side, NodeId),
    Top(Member), Bang(Member, NodeId), Quest(Member, NodeId),
    Copy(Member, NodeId), Weaken(Member, NodeId), Mix(NodeId, NodeId),
    // reserved: Cut(Member, NodeId, NodeId)               step 34, 0.2.0
    //           Forall(Member, Eigen, NodeId)             step 38, 0.3.0
    //           Exists(Member, FoTermId, NodeId)          step 38, 0.3.0
}
const _: () = assert!(size_of::<Node>() == 16);   // with the three: 16 (probe)
impl Node { pub const NAMES: &'static [&'static str]; /* the JSON tags, R118 */ }

pub struct Proof {
    forest: Forest,
    nodes: Box<[Node]>,
    mode: Option<Mode>,            // the mode it was found or declared in (R7)
    // step 38, private, empty for ground proofs: instances: Instances (3.9)
}
```

- **Every operand is a `Member`** (3.4). The bottom-up checker cannot
  fix the instance of a leaf from the nodes below it, a memo hit shares a
  subproof among several conclusions, and two copies of one `?`
  occurrence with different eigenvariables are one `OccId`
  (impact-quantifiers finding 2): so a first-order proof names, in each
  node, the instance it acts on, and the propositional proof names the
  occurrence, the same number.
- `Proof::new(forest, nodes, root)` keeps its signature; step 38 adds
  `Proof::with_instances(forest, nodes, root, instances)` (additive),
  which re-interns the table so that equal instances are equal members
  (the checker compares by id). No dummy argument now: the table has no
  propositional value.
- `Proof::mode() -> Option<Mode>`: set by the search (`Answer::of_arena`),
  by a reader from the optional `"mode"` key, and by `with_mode`; the
  exporters take it when the caller gives none (R7, R152).
- The checker: `check(mode)` and `check_with(mode, &Limits, impl Stop)`;
  the pass polls every 4 096 nodes (F12, R23). Section 10.5 has the
  first-order pass.
- `Rule` stays closed; `Rule::ALL: &'static [Rule]` (an array's length
  is part of its type); a test round-trips every value through
  `name`/`from_str`, the label tables are checked against `ALL.len()` at
  compile time. To come, one checklist each (`core-export.md`): `Cut`
  (34), `\L \R /L /R` (36), `∀ ∃ ∀L ∀R ∃L ∃R` (38). F60's redesign (a
  classical rule with a side, the two-sided names a display table) makes
  36's and 38's two-sided rules rows rather than variants.
- `Inference` is `#[non_exhaustive]` with private fields: `sequent() ->
  &[Member]` (ascending with repeats commutatively; in an ordered mode
  the planar order, 10.3), `rule()`, `principal() -> Option<usize>`,
  `premises() -> &[InfId]`, `times()`; step 38 adds `datum() ->
  Option<Binding>` (`Binding::Witness(FoTermId)`, `Binding::Eigen(Eigen)`).
- `Derivation::of_goal` is the graft's path; `InfId` names a derivation's
  inferences (postorder), `GoalId` the session's (top-down) (F69).

```rust
/// One step of an interactive proof: the formula acted on, the rule, and
/// what the rule needs besides.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Step { position: usize, rule: Rule, split: Split /*, 38: witness: Option<Witness> */ }
impl Step {
    pub fn new(position: usize, rule: Rule) -> Self;
    #[must_use] pub fn left(self, positions: &[usize]) -> Self;  // ⊗ and Mix: the formulas going left
}
#[non_exhaustive]
pub enum Split { None, Left(Vec<usize>) /* 36: At(usize), a cut position of an ordered goal */ }

impl Interactive {
    pub fn apply(&mut self, goal: GoalId, step: &Step) -> Result<Vec<GoalId>, Refusal>;
}
```

The command keeps its grammar `apply G P RULE [P…]` and the session JSON
its keys (F22). Step 38 adds `Step::witness`: a term over the session's
symbols, or an open metavariable that a later axiom binds and undo
unbinds, with its `Refusal`s. `close_with` refuses a proof of another
sequent (`Error::ForeignProof`) or one the session's mode forbids (F23);
`close_all` returns each goal's own result, `Progress::item` naming the
goal, so a front end budgets per goal (R92).

### 3.7 `ProofStructure`

```rust
impl ProofStructure {
    pub fn new(forest: Forest, mode: Mode) -> Result<Self, Error>;
    pub fn from_links(forest: Forest, mode: Mode, links: &[(OccId, OccId)]) -> Result<Self, Error>;
    pub fn from_proof(proof: &Proof, mode: Mode, limits: &Limits, stop: impl Stop) -> Result<Self, Error>;
    pub fn mode(&self) -> Mode;
    pub fn is_correct(&self, stop: impl Stop) -> Result<(), NetError>;
    pub fn sequentialize(&self, limits: &Limits, stop: impl Stop) -> Result<Proof, Error>;
}
```

- **The criterion is the mode** (F10, R79): Mix from `has_mix`; at 35
  `is_essential(&self, stop)` for an intuitionistic structure (R123); at
  36 planarity from the order. Affine is `NetError::Mode`, a fragment
  without nets `NetError::Fragment` (`Fragment::has_nets`).
- `from_proof` matches every `Node` variant, refusing by name a node it
  does not read (`NetError::Node`), under the limits and the stop (R254,
  F49).
- Step 33's instances continue the numbering as a `VertexId`, the MLL hot
  path unchanged (P2, D-6). `NetError` is `#[non_exhaustive]`.

### 3.8 The ordinary layer

- `ordinary::Sequent::new(formulas, left, right) -> Result<Self, Error>`
  refuses an id that is no node of the arena, `Formulas::add` an operand
  that is not an earlier node and an atom outside the names (F13, R249);
  at 38 the same two refuse a free variable and a symbol at two arities.
- `ordinary::Node` stays closed (P3), asserted at 12 bytes; reserved
  `Pred(u32, ArgsId)`, `Forall(NodeId)`, `Exists(NodeId)` (12 bytes,
  probe). `Rule`, `Inference` (private fields, `datum()` reserved),
  `Options`, `Logic`, `Translation` are `#[non_exhaustive]`.
- **One first-order term type for both layers** (D-4, impact-fo-ordinary
  item 1): `Formulas` will hold the same `fo` table as `Sequent` (3.9), so
  `translate` copies term ids and the LK/LJ checker keeps "equal formula ⇔
  equal id".
- `Translation::target() -> Target { fragment, mode }`; the docs stop
  implying that the classical path terminates on first-order input
  (fo-embeddings §1.5). `Image::read_back(&self, &Proof, limits, stop)`
  reads the proof, where witnesses live (D-12). The JSON forms of
  `ordinary::Sequent` and `Derivation` are born versioned (7).

### 3.9 First-order tables, reserved (step 38 adds them; nothing at 28)

The shapes step 38 adds, fixed here so that nothing before it takes their
place. All are private fields or new modules: adding them breaks nothing.

```rust
// sequents::fo
pub struct FoTermId(u32);   // continues past the sequent's own terms in a proof or a search (P2)
pub struct FunctionId(u32); // a function symbol, constants included: (name, arity)
pub struct ArgsId(u32);     // an argument list in a hash-consed CSR table; ArgsId::EMPTY
pub struct Eigen(u32);      // an eigenvariable of a proof
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum FoTerm {           // closed; 12 bytes (probe)
    Bound(u32),                 // de Bruijn index: innermost binder 0 (locally nameless)
    App(FunctionId, ArgsId),    // f(t…); a constant has EMPTY arguments
    Eigen(Eigen),               // only in a proof's or a search's extension
    Meta(u32),                  // a metavariable: only in a search's extension, or open in a proof (Q3)
}

// in Sequent, private:
//   arities: Vec<u32>          per atom; empty: every atom nullary
//   functions: Vec<(String, u32)>
//   fo: Vec<FoTerm>            topological, hash-consed by optimize; ground or bound only
//   args: Vec<FoTermId>, arg_start: Vec<u32>   the CSR argument lists, hash-consed
//   binders: Vec<(TermId, String)>             a bound variable's name, for printing only

// occurrences::instances
pub struct FrameId(u32);    // a list of FoTermIds, one per binder above, outermost first
pub struct Instances {      // what members past forest.len() stand for
    members: Vec<(OccId, FrameId)>,
    frames: Vec<FoTermId>, frame_start: Vec<u32>,  // hash-consed
    terms: Vec<FoTerm>,     // the extension of the sequent's fo arena (witnesses, eigenvariables)
}
```

- **Locally nameless** (fo-linear §5.1): bound variables are indices, so
  α-equal closed formulas are one `TermId` and `optimize` hash-conses
  `∀x.p(x)` and `∀y.p(y)` with no renaming; the name table is for
  printing only, the first name winning a merge, a clash primed. Closed
  input: an identifier in term position that no binder binds is a
  constant (fo-linear §7 Q1, the author's to confirm).
- **Instantiation never rewrites a formula**: the instance of an
  occurrence is its term read through its frame. The sequent's tables are
  read-only during a search; a search's terms extend them by offset and
  are truncated by its trail (10.5).
- **Bounds**: `Sequent::fo_size()`, the unfolded size of the first-order
  terms, saturating like `occurrences()`, and `Limits::terms` (a field
  added at 38, `Limits` being `#[non_exhaustive]`) refusing a larger
  input at every reader (R250); no term operation walks a shared term
  once per path.

## 4. Errors

```rust
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error { /* one variant per kind; named fields only (F31) */ }

impl Error {
    /// The stable name of the kind, which a program branches on.
    pub fn code(&self) -> ErrorCode;
    /// A fault of the input, a refusal (a bound, a stop, something not
    /// supported: no verdict), or a defect of this crate.
    pub fn class(&self) -> ErrorClass;
    pub fn is_refusal(&self) -> bool { self.class() == ErrorClass::Refusal }
    /// The message with formulas instead of ids, given the forest the ids are of.
    pub fn describe<'a>(&'a self, forest: &'a Forest) -> Describe<'a>;
}
#[non_exhaustive] #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ErrorCode { /* snake_case on the wire: sequent_parsing, too_many_occurrences, … */ }
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ErrorClass { Input, Refusal, Defect }
```

- **One family** (F8, F30, F48, S18): each error type stays precise
  where a function can only fail that way (C-GOOD-ERR) and converts into
  `Error`, whose variants have named fields, a lowercase `Display`
  without a full stop, `Send + Sync`, and `source()` to the inner type;
  one `Describe` adaptor over one writer per type replaces the two
  `Described` types and `NetError`'s messages written twice.
- **A refusal is typed, never a verdict** (S4): a checker's refusal is
  `Error::Unchecked`, class `Refusal`; a found proof whose check is
  refused is no error but `Unknown(Reason::Unchecked { limit })` (F139,
  R138); size and memory `ViewError`s are refusals; `Rejected` and
  `ReadBack` are defects.
- **Wire form** (R129, AIP-193), written only: `{"code":
  "engine_refused", "class": "refusal", "message": "…", "details":
  {"engine": "net", "because": {"fragment": {"decides": "MLL", "goal":
  "MALL"}}}}`; `details` holds the fields by name, a `ParseError`'s byte
  span with `line` and `column` (UTF-16 units, as a browser counts).

| type today | into `Error` | class |
|---|---|---|
| `ParseError` (gains `Error`, `Clone`, `PartialEq`, F34) | `SequentParsing { errors }` | Input |
| `Error::{Lltp, Mist, Tptp}(String)` | the same, each with its reader's `Problem` value instead of a string | Input |
| `CheckError` | `InvalidProof` / `Unchecked` (`is_refusal`) | Input / Refusal |
| `NetError` | `InvalidNet`; its `Fragment` and `Mode` cases | Input; Refusal |
| `ShapeError` | `NotIntuitionistic` | Input |
| `ViewError` | `InvalidProof` for `Invalid`, else `View` | Refusal |
| `Refusal` (interactive) | `Refused` | Input |
| `WriteError`, `RenderError`, `rocq::Unsupported`, `svg::TooLarge` | `Write`, `Render`, `Unsupported`, `TooLarge` | Refusal (stopped, unsupported, too large), Defect (`fmt::Error`) |
| `UnknownRule`, and every `FromStr` of a name (`Engine`, `Mode`, `Bias`, `Fragment`, `Logic`, `Translation`) | `UnknownName { what, name, known }` | Input |
| the six refusals of a forced engine | `EngineRefused { engine, because: NotTaken }` (8.6) | Refusal |
| `NoEngine` | built by the dispatch (8.4) | Refusal |
| a stop outside the search | `Stopped { phase }` | Refusal |
| `TooManyOccurrences`, `TooManyNodes`, the index errors | named fields; `Space::{Atom, Term, Node, Occurrence, Member}` names which index | Input |

## 5. Bounds and stops

### 5.1 One bounds value

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize), serde(default, deny_unknown_fields))]
pub struct Limits {
    /// The most bytes a call may hold at once; None for no bound. JSON `memory_bytes`.
    pub memory: Option<u64>,        // DEFAULT_MEMORY: 1 GiB
    /// The most subformula occurrences an input may unfold to. JSON `occurrences`.
    pub occurrences: Option<u64>,   // DEFAULT_OCCURRENCES: 50 000 000; None: what a forest indexes
    /// The deepest recursion of the focused engine. JSON `recursion`.
    pub recursion: u32,             // DEFAULT_RECURSION: 2 048
    /// The most units of work (5.2); None for no bound. JSON `work`.
    pub work: Option<u64>,          // None
    /// The most bytes a derivation is estimated at and still built. JSON `derivation_bytes`.
    pub derivation: Option<u64>,    // DEFAULT_DERIVATION: 64 MiB
    // step 38: terms: Option<u64>, the unfolded size of first-order terms (R250)
}
impl Limits {
    /// A browser tab's: 256 MiB, 2 million occurrences, the recursion a
    /// 1 MiB stack holds, 16 MiB of derivation (measured at step 32).
    pub const BROWSER: Self;
    /// The stack a thread needs at `recursion`, and its inverse (R45).
    pub const fn stack_size(&self) -> usize;
    pub const fn recursion_for_stack(bytes: usize) -> u32;
}
```

Every long call takes `&Limits` (F62, R22): the search reads it from
`Options::limits`, every other call as an argument. The command builds
it once from `--memory-limit`, `--occurrence-limit`, `--recursion-limit`,
`--work-limit` and `--derivation-limit`, the web client holds it as JSON
(`Limits::BROWSER`), and a third wrapper sets one value (R142, D15, D16).
`DEFAULT_MEMORY_LIMIT` (three copies today) is one constant,
`Limits::DEFAULT_MEMORY` (F63).

### 5.2 One stop, with progress

```rust
/// What a long call polls: true stops it. Polled at a bounded interval of
/// work, never by a count of polls.
pub trait Stop {
    fn poll(&mut self, progress: &Progress) -> bool;
}
impl<F: FnMut() -> bool> Stop for F { /* ignores the progress */ }
/// A closure that reads the progress.
pub struct WithProgress<F>(pub F);
impl<F: FnMut(&Progress) -> bool> Stop for WithProgress<F> {}
impl Stop for &mut dyn Stop {}
/// The stop that never fires.
pub fn never() -> impl Stop;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct Progress {
    /// Units of work since the last poll.
    pub work: u64,
    /// Units of work since the call began; on one thread it ends as `Statistics::work`.
    pub done: u64,
    /// Bytes the call's account holds now.
    pub held: u64,
    /// What is running.
    pub phase: Phase,
    /// The item of a call that runs several: a goal of `close_all`, a problem of a batch.
    pub item: u32,
}
#[non_exhaustive]
pub enum Phase { SetUp, Search, Refute, Check, Derivation, Export, Read, Net }
```

- Closures `FnMut() -> bool` keep working (no stop of the command, the
  harness or the tests changes); a front end that reads progress wraps
  its closure in `WithProgress` or implements `Stop` itself. The blanket
  impl, `WithProgress`, the impl for `&mut dyn Stop` and a downstream
  `impl Stop for Mine` coexist under coherence (probe). The trait is
  object safe; entries take `impl Stop` and pass `&mut dyn Stop` inward.
- **The unit of work** is one elementary step, about a member moved and
  its counts tested in a split search; each engine documents its own on
  its `Engine` variant (focused: 16 per stable sequent plus its width in
  128-occurrence words, one per split step or forced split, as
  `Stop::fired` counts today; net: one per literal chosen and per failed
  exact test, as it passes today; additive: one per pair; Horn: the
  places of a marking expanded; the checker: one per node plus its
  members). `limits.work` ends a search with
  `Reason::WorkLimit(n)`, deterministic on one thread and the same on
  wasm (R21); a wasm caller reads its clock every so many units (R243).
- **Progress is counted apart from the slices** of the default bias's two
  searches (`Poll::fired(slice, progress)` inside, the engines' `Stop`
  enum renamed `Poll`), so no counter of a decided run moves: the forced
  chain's poll, which passes no slice on purpose, passes its progress.
- **The default bias's scheme** is an option, `Options::schedule:
  Schedule::{Auto, Threads, Turns}`, so one input gives one set of
  counters on the command and on wasm (`Turns`; F110, R47).

### 5.3 The memory account

`search::memory::Account` stays crate-private (P7); what is public is the
bound. One account per call, charged by capacity where anything grows
with the input, refused at the bound with the call's own refusal:

| call | what charges it | refusal |
|---|---|---|
| a search (`prove_goal`) | memo, arena, pools, counts, classes, additive memo, Horn markings and simplex; from step 35 the essential closure matrix, from 37 the inverse database and index, from 38 bindings, trail, search terms, instance tables | `Reason::MemoryLimit` |
| the race of one thread and a pool (`search::race`, R85) | **one account both draw from** (F104, R18): a search that cannot get memory empties its own memo first, as today, so the sum stays under the bound; which keeps the larger memo is first come, first served, measured on the comparison set before the race ships (half each starved a wide memo) | `Reason::MemoryLimit` |
| a batch | each worker a search's bound, as many workers as the batch's bound holds (unchanged, `core-batch.md`) | per problem |
| the check, the size pass, a derivation | the pass's tables and states (unchanged), the derivation's estimate | `Unchecked` / `ViewError::Memory` |
| `from_proof`, `sequentialize`, `Image::read_back`, `ordinary::Derivation::check` | the structure built, the unfolded derivation | `Error::View` / `NetError` refusal |
| readers | not an account: `Limits::occurrences` (and at 38 `terms`) refuse before anything of that size is built | `TooManyOccurrences` |

What no account counts is stated on `Limits::memory`: the forest and the
sequent (bounded by `occurrences`), the proof returned, thread stacks,
the allocator's overhead (R29). The net engine's structure and scratch
are linear in the forest and stay uncounted; the essential engine's
quadratic matrix is counted (R35).

**The pool's stack (H18).** A thread that waits at a scope runs stolen
jobs on top of its own frames, so the recursion a thread's stack holds is
not the task's depth. The bound holds when the depth is the thread's:
the runtime keeps one depth per worker (`Runtime::depths`, indexed by
rayon's thread index, a runtime field and no thread-local), and a task
starts from the depth of the thread that runs it. `Reason::RecursionLimit`
then comes before an overflow at any `recursion`, and
`Limits::stack_size()` sizes every worker for it.

### 5.4 Every long public call

| call | stop | bound | refusal |
|---|---|---|---|
| `prove`, `prove_until`, `prove_goal` | `impl Stop` (`prove`: `never()`) | `Options::limits` | `Verdict::Unknown(Reason)` |
| `engine_for` | none: linear passes | `limits.occurrences` | `Error` |
| `search::race` (parallel) | `impl Stop` | one account | `Unknown` |
| `batch::run`, `batch::prove` | a batch stop polled by every worker; `Results::cancel` and `Drop` cancel (R40, F74) | the batch's bound | `Unknown(Stopped)` per problem |
| `Interactive::close`, `close_with`, `close_all` | `impl Stop`, `Progress::item` per goal | `Options::limits` | per goal |
| `Proof::check_with`, `derivation_size`, `derivation_with` | every 4 096 nodes | `limits.memory`, `limits.derivation` | `Unchecked`, `ViewError` |
| `ProofStructure::from_proof`, `is_correct`, `sequentialize` | per node, per round, per stage | `limits.memory` | `NetError::Stopped`, `Error::View` |
| `export::*::write` | per inference (as today) | the derivation was bounded | `WriteError::Stopped` |
| `svg::net`, `svg::sequent`, `svg::two_sided` | per link, per piece (F5) | a byte bound | `TooLarge` |
| `png::from_svg`, `pdf::from_svg` | none | the render's own bound, now on any SVG (F37) | `RenderError` |
| `Sequent::parse_within`, `from_json_within` of every form, `lltp`, `mist`, `ordinary::read_tptp` | none: linear in the input | `limits.occurrences` | `TooManyOccurrences` |
| `Interactive::from_json_within` | none: made linear (F21) | `limits.occurrences` | `Error` |
| `ordinary::translate`, `Image::read_back`, `ordinary::Derivation::check` | read-back and check: per inference | `Limits` | `Error::Stopped`, `View` |
| `families::Family::instance` | none | its sizes | `Error::FamilySize` (F6) |

A test per row fires the stop at its first poll (R22, R30); the table
lives in the crate docs, and `core-search.md`'s "Not polled" list shrinks
to `Forest::within` and single linear passes.

## 6. Options

Every value follows P6. Fields, defaults and wire keys:

| value | fields (default) | set by |
|---|---|---|
| `Limits` | 5.1 | `--memory-limit`, `--occurrence-limit`, `--recursion-limit`, `--work-limit`, `--derivation-limit`; the web client's settings; `Limits::BROWSER` |
| `search::Options` | `limits` (`Limits::default()`), `memo_limit` (2²⁰), `force: Option<Engine>` (`None`: the dispatch), `assume: Option<Fragment>` (`None`: detected), `copy_bound: Option<u32>` (`Some(3)`), `forward_copies: u32` (30), `bias: Bias` (`Auto`), `schedule: Schedule` (`Auto`), `test_period: Period` (`BySize`), `jobs: u32` (1, more than `MAX_JOBS` read as that many), `check: bool` (true), `pool: Option<Pool>` (not on the wire) | flags of `prove` and `interact` from one shared struct (F194), `--options FILE` (R183), the web client's JSON |
| `batch::Options` | `search: search::Options`, `workers`, `cores: Cores`, `memory: Option<u64>`; `Problem` gains `overrides: Option<search::Options>` (R73) | the batch flags |
| `ViewOptions` | `sides: Sides` (`OneSided`), `compact: Compact` (`Auto`) | `--two-sided`, `--compact` |
| `export::OutputOptions` (R141) | `text: TextOptions`, `latex`, `typst`, `svg: svg::Style`, `png`, `pdf`, `rocq` | `--style KEY=VALUE` and `--style-file`, one dotted key per field, read by the library (F196) |
| `rocq::Options` | `form`, `lemma` (checked to be an identifier, F38), `prelude: Option<String>` (`None`: the kernel's import, F15, R140), step 31's `kernel: Kernel` (`Auto`) | `--lemma`, `--prelude`, `--style rocq.*` |
| `ordinary::Options` | `translation`, step 38's TPTP options belong to `read_tptp`'s own value (D-12) | `--translation` |

- **Constants**: every default is a `DEFAULT_*` of its type with a flag
  and a doc line (D16), including the command's time limit,
  `search::Options::DEFAULT_TIME_LIMIT: Duration = 2 s`, data the front
  ends apply through their own stop (R148), and the net engine's test
  period (`Period::BySize`: every link up to 200 occurrences, every
  fourth above; a `--test-period` flag, R149).
- **Presets** are named values: `Limits::BROWSER`,
  `search::Options::BROWSER` (its limits, no copy bound, `Schedule::Turns`,
  one job), `ViewOptions::BROWSER`, `svg::Style::dark()`, and
  `Options::BENCHEXEC` for step 29's single-thread column (R151).
- **What an outcome records**: the written `Outcome` gains the crate
  version and the options it ran under, as optional keys (F77).
- **Dispatch thresholds are not options** (R149, decided here): the net
  row's `NET_MULTIPLICITY` is part of a measured row, documented on
  `Engine` beside its measurement; a caller who wants another engine
  forces it (`force`), which is the knob D16 asks for. A threshold option
  would let a front end move a row the library's measurement placed.

## 7. Wire forms

**Policy** (R6, F26, AIP-180, P8). Every form read back (`Sequent`,
`Proof`, `ProofStructure`, `Interactive`, `Refutation`, `Reason`, the
options values, the ordinary forms) and the written `Outcome` and
`Error` carry `"version": 1`; missing reads as 1, a greater one is
refused by name. A new key is optional, skipped when empty, so a
propositional, cut-free, commutative file is byte-identical whatever
later steps add; no new meaning rides on an old tag (a predicate literal
is the tag `P`, never `{"V": a}` beside an `args` key). Data forms ignore
unknown keys, options forms refuse them, each form's doc says which; an
unknown tag fails with the tag in the message. `core/tests/serialize.rs`
keeps one file per form from before each step that changes it and reads
it unchanged.

**Names** (T1, AIP-140): `ids` → `roots`, `var_dict` → `atoms`, `proof`
→ `nodes`, `memory_limit` → `limits.memory_bytes`, `once` →
`single_use`, `mix` (nets) → `mode`; enumerations by name (AIP-126);
`Engine`, `Bias`, `Schedule`, `Fragment`, `Mode` words read and written
by one table each. The pre-release names are read as aliases, so a file
of 2026-09-30 loads; only the new names are written. The tags `V`, `D`
and the rule tags stay (R251).

**Numbers** (R247): integers are JSON numbers; counts that may pass 2⁵³
(`Size`, `Statistics`) are exact below it and saturate at `u64::MAX`,
which a reader treats as "at least"; `Size::exact` is false whenever a
count saturated (F70). No string-encoded integers.

**Read back or write-only**: `Outcome` and `Error` are written only (R3);
`Refutation` and `Reason` read back, so a stored refutation can be
checked (31) — a stored refutation names its sequent and mode
(`{"version":1,"sequent":…,"mode":…,"refutation":…}`).

Examples (propositional output stays as it is but for `version` and T1):

```json
{"version":1,"terms":[{"D":0},{"D":1},{"⅋":[0,1]},{"V":1},{"V":0},{"⊗":[3,4]}],"roots":[2,5],"atoms":["A","B"],"antecedents":1}
```
`A ⊗ B |- B ⊗ A`: the arena unchanged, `antecedents` the written sides
(absent for a sequent written one-sided).

```json
{"version":1,"sequent":{…},"mode":{"intuitionistic":false,"affine":false,"mix":false},
 "nodes":[{"ax":[2,4]},{"ax":[1,5]},{"⊗":[3,0,1]},{"⅋":[0,2]}]}
```
A proof: `mode` optional (R7); every operand a member (one integer).
At 34 a cut-free proof stays byte-identical and a proof with cuts adds
`"cuts":[t…]` and the tag `cut`; at 38 a first-order proof adds the tags
`∀` `[m, a, p]` and `∃` `[m, w, p]` and the keys `instances` (`[[occ,
frame]…]`), `frames`, `fo_terms`, `eigenvariables`.

```json
{"version":1,"verdict":"unknown","reason":{"work_limit":{"units":1000000}},"fragment":"MALL",
 "mode":{…},"engine":"focus","statistics":{"nodes":81,"memo_hits":3,"memo_entries":40,"splits":512,
 "links":0,"tests":0,"copies":0,"forward_copies":0,"work":1000004},"linlog":"0.1.0","options":{…}}
```
An outcome: reasons with data are objects (`{"copy_bound":{"bound":3}}`,
`{"memory_limit":{"bytes":1073741824}}`), data-free ones strings (F86);
an outcome of a goal off the roots carries `"goal":[m…]` and is refused
by `check` (F89).

```json
{"version":1,"sequent":{…},"mode":{"intuitionistic":false,"affine":false,"mix":true},"links":[[0,2],[3,4]]}
```
A proof structure: `mode` in place of `mix` (`mix` read as an alias);
33's `vertices`, `boxes`, `jumps`, 34's `cuts`, 38's `substitution` are
keys to come.

```json
{"version":1,"limits":{"memory_bytes":268435456,"occurrences":2000000,"recursion":400},
 "force":null,"copy_bound":null,"bias":"auto","schedule":"turns","jobs":1,"check":true}
```
Search options (keys absent take their defaults; an unknown key is an
error). The session form keeps its keys, `version` added; its members are
integers, and step 38 adds `instances` and `bindings`.

## 8. Engines and the search's front door

### 8.1 The pipeline, and the one place a verdict is built

`prove_goal(forest, goal: &[Member], mode, &options, impl Stop) ->
Result<Outcome, Error>` runs, in this order, each a function of its own,
`engine_for` sharing steps 1 to 3:

1. **`fragment_of`**: the members are occurrences of the forest (step 38
   adds an entry that takes the caller's instance table, additive), the
   goal's fragment detected or the asserted one checked
   (`FragmentMismatch`).
2. **`read`**: `Mode::check`, then in intuitionistic mode the reading
   (`NotIntuitionistic`), from step 36 the planar order in an ordered mode.
3. **`prepare`**: the `Task`; the engine forced (`options.force`) or
   dispatched (8.4, `NoEngine` when no row takes the goal); its
   `admits` (8.6).
4. **The set-up poll** on a large forest.
5. **`decide`**, under one account (5.3) and the caller's stop.
6. **`conclude`**, the one place an answer becomes a `Verdict`:
   - `Ok(Some(proof))`: a proof of the roots is checked in every build
     (`options.check`; off, a `debug_assert!`); a rejected one is
     `Error::Rejected` (a defect, never a verdict), a refused check
     `Unknown(Reason::Unchecked { limit })` (F139), else `Proved`.
   - `Ok(None)`: `Unprovable` with the engine's own refutation, else the
     first refuter's (8.7), else `Exhausted`.
   - `Err(reason)`: `Unknown(reason)`, unless a refuter may decide it
     (8.7, off by default).
7. The `Outcome`: verdict, fragment, mode, engine, statistics, the net,
   and for a goal off the roots the goal itself (F89).

Every proof of the roots has passed the checker when it is returned;
every `Unprovable` rests on an exhaustive search or a refutation; every
refusal is `Unknown` or an `Error` of class `Refusal`. No engine builds a
`Verdict` (S4).

### 8.2 The interface every engine implements (crate-private)

```rust
/// A goal as an engine is handed it.
pub(crate) struct Task<'a> {
    pub(crate) forest: &'a Forest,
    pub(crate) goal: &'a [OccId],        // the roots in the forest's order when `roots`
    pub(crate) fragment: Fragment,
    pub(crate) mode: Mode,
    pub(crate) reading: Option<&'a Reading<'a>>,
    pub(crate) roots: bool,
    // step 36: the planar order, in an ordered mode
    // step 38: instances: Option<&'a Instances>, None for a propositional goal
}

/// What an engine's search ended with.
pub(crate) struct Answer {
    pub(crate) result: Result<Option<Proof>, Reason>,
    pub(crate) statistics: Statistics,
    pub(crate) net: Option<ProofStructure>,
    pub(crate) refutation: Option<Refutation>,   // the engine's own, sound by its argument
    // step 31 (R112): trace, an opt-in failure record, off by default
}

pub(crate) trait Decide: Sync {
    /// Refuses a goal the engine does not decide, as a forced engine is
    /// refused: its largest fragment first, then its modes, the goal, the shape.
    fn admits(&self, task: &Task<'_>) -> Result<(), Error>;
    /// Decides the goal. `Ok(None)` only when the search was exhaustive
    /// for this goal in its fragment and mode (an incomplete engine gives
    /// up with a `Reason`, never with `Ok(None)`, R111); polls `stop` at a
    /// bounded interval of work, passing that work; charges `account` for
    /// everything that grows with the input; returns a proof whose
    /// premises precede their conclusions; never panics on a goal it admits.
    fn decide(&self, task: &Task<'_>, options: &Options, account: &Account,
              stop: &mut dyn Stop) -> Result<Answer, Error>;
}
```

`Decide::decide` stays the single entry so that a resumable (explicit
stack) focused engine can replace it without a change of the front door
(R47). `Answer::of_arena(forest, mode, (result, nodes, statistics))`
builds the `Proof` (with its mode) for an engine that keeps an arena; at
38 `of_instances` passes the table. A first-order goal enters through
the same `Task` and `Answer` (R109): the forest carries the terms, the
task its instances, and the verdict is built as above.

### 8.3 `Engine`: the registration list

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Engine { Focus, Net, TwoSided, Additive, Horn }

impl Engine {
    /// Every engine, in the order of declaration.
    pub const ALL: &'static [Engine];
    /// The name in text, in JSON and on the command line: an open set.
    pub const fn name(self) -> &'static str;          // exhaustive match
    /// Whether it searches on several threads when `Options::jobs` asks:
    /// the command adds a pool beside one thread only for these.
    pub const fn parallel(self) -> bool;               // exhaustive match, positive (F141)
    fn implementation(self) -> &'static dyn Decide;    // exhaustive match
}
impl Display for Engine {}  impl FromStr for Engine {}  // serde both ways, by name
```

**Adding an engine** (R86) touches this list and nothing in
`prove_goal`, `Task` or `Answer`; the compiler names 2 to 4:
(1) the variant and its doc: what it decides, the options it reads and
those it ignores (documented, never refused, R147), its unit of work, its
counters, its measured row; (2) `name`; (3) `parallel` (a sequential
engine answers `false` and says it runs on the calling thread whatever
`jobs` says, R110); (4) `implementation`; (5) `ALL`, which a test checks
by an exhaustive match mapping each variant to its index; (6) its module
with `impl Decide`; (7) a `DISPATCH` row only where a measurement shows
it winning (D19), else it stays forceable; (8)
`reference::configurations` and a differential test from the start
(R209); (9) README's console blocks (`cli/tests/readme.rs`); the
command's `--engine` and the harness's `--engines` derive from `ALL`, and
the JSON `engine` string is documented as an open set (AIP-126);
(10) its rules file and `core-search.md`.

### 8.4 The dispatch as data

```rust
struct Row { fragment: Fragment, modes: Modes, feature: Feature, engine: Engine }
const DISPATCH: &[Row] = &[
    Row { fragment: Fragment::ADDITIVE, modes: Modes::Any,     feature: Feature::TwoFormulas,      engine: Engine::Additive },
    Row { fragment: Fragment::MLL,      modes: Modes::Linear,  feature: Feature::FewEqualLiterals, engine: Engine::Net },
    Row { fragment: Fragment::MELL,     modes: Modes::Any,     feature: Feature::PetriNet,         engine: Engine::Horn },
    Row { fragment: Fragment::LL,       modes: Modes::Intuitionistic, feature: Feature::Any,       engine: Engine::TwoSided },
    Row { fragment: Fragment::LL,       modes: Modes::Classical,      feature: Feature::Any,       engine: Engine::Focus },
];
fn dispatch(task: &Task<'_>) -> Result<Engine, Error>;   // the first row that takes it, else NoEngine
```

- **Rows are first-match, in priority order**, each measurement in
  `Engine`'s doc table; a test reads that table from the source and
  checks it lists `DISPATCH`'s rows in order (R87). A row the measurement
  does not earn is deleted; its engine stays forceable (D19).
- **`Modes`** destructures the whole `Mode` (F140). Every row of today,
  `Modes::Any` included, takes commutative modes only, so the ordered
  modes of 36 reach no engine but by a row of their own (R90).
- **`Feature`**: one linear pass over the task, linear memory, no clock,
  polled on a large forest (R88); a new one is a variant and an arm of
  `Feature::of`. Coming: 35's pure-tree feature, 37's "many hypotheses,
  small goal", each only with a row it earns. `NET_MULTIPLICITY` stays a
  private constant of its row (6).
- **No row takes a fragment it does not name**: at 38 a goal with the
  quantifier bit reaches only the two rows that name `LL1`, and every
  other engine's `admits` refuses it (R135).

### 8.5 `Statistics`, per engine (T5)

```rust
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
#[cfg_attr(feature = "serialize", derive(Serialize))]   // no hand-listed proxy (F88)
pub struct Statistics {
    pub nodes: u64, pub memo_hits: u64, pub memo_entries: u64,  // u64: 32-bit targets (R46)
    pub splits: u64, pub links: u64, pub tests: u64,
    pub copies: u32,          // the level the search bounded by `copy_bound` began last
    pub forward_copies: u32,  // the forward search's level, under `Bias::Auto` (F144)
    pub work: u64,            // the units of work; on one thread, the sum of every Progress::work
    // appended by later steps, each with its line in this table and a CSV column at the tail's end
}
```

T5's recommended answer: the shared counters stay, since every baseline
and `bench/targets.sh` read them by name; each is documented per engine
on the type and in the JSON form; an engine adds a field only where no
shared counter fits, and never gives an existing one a new meaning on an
engine that already fills it (R9).

| field | focus, two-sided | net (35: essential) | additive | horn | inverse (37) | first-order focus (38) |
|---|---|---|---|---|---|---|
| `nodes` | stable sequents | literals chosen | pairs | markings reached | sequents derived | stable sequents |
| `memo_hits` | memo hits | 0 | memo hits | markings met again | conclusions subsumed | ground memo hits |
| `memo_entries` | the memo's peak | 0 | the memo's peak | markings kept | the database's peak | ground memo's peak |
| `splits` | split steps | 0 | 0 | 0 | 0 | split steps |
| `links`, `tests` | 0 | links tried, exact tests (35: closure updates in `tests`) | 0 | 0 | 0 | 0 |
| `copies`, `forward_copies` | levels | 0 | 0 | 0 | 0 | levels |
| added | | 35: per-prune counts if the harness's ablation needs them | | | 37: none planned | 38: `unifications`, `bindings` |

### 8.6 What a forced engine refuses

```rust
Error::EngineRefused { engine: Engine, because: NotTaken }
#[non_exhaustive]
pub enum NotTaken {
    Fragment { decides: Fragment, goal: Fragment },  // beyond its largest fragment
    Mode(Mode),                                       // not in its modes
    Goal,                                             // not the roots (the net engine)
    Shape,                                            // not two additive formulas, not a Horn program
}
```

Every `admits` checks, in this order, its largest fragment (focused:
`LL`, at 38 `LL1`; net: `MLL`; additive: `ADDITIVE`; Horn: `MELL`), its
modes, the goal, the shape (F140, R135). Nothing is refused today that
was not before (every detected fragment lies in `LL`); the messages keep
the words the behaviour lock pins (`Display` writes per engine and
reason). The harness classifies by `ErrorClass::Refusal` instead of a
list of variants (R135).

### 8.7 Refuters: the second plug-in kind (C2, option C)

```rust
pub(crate) trait Refute: Sync {
    /// Whether the refuter can say anything of the goal in its fragment and mode.
    fn applies(&self, task: &Task<'_>) -> bool;
    /// A refutation of the goal within its own bounds, or none; never a guess.
    fn refute(&self, task: &Task<'_>, options: &Options, account: &Account,
              stop: &mut dyn Stop) -> Option<Refutation>;
}
pub(crate) enum Refuter { Counts /* step 31: Classical; later: Kripke, Phase */ }
const REFUTERS: &[Refuter] = &[Refuter::Counts];
```

- The count refutation that `prove_goal` computes today
  (`focus::refutation`) **is** the first refuter: the kind exists from
  step 28 with one member, moved to `search/refute.rs`, sharing the
  focused engine's `Counts` as today (the independence of a *checker* is
  step 31's `Refutation::check`, in `search/refutation.rs`, which shares
  no engine code, R124).
- `conclude` runs them **after** the search, never during: after
  `Ok(None)` to name why (an engine's own refutation wins: the Horn
  engine returns the counts' it computed, F128); after `Unknown`, only
  when `options.refute_unknown` (step 31, `false` by default, so every
  baseline's verdicts stay), and then only a refutation that
  `Refutation::check` accepts turns `Unknown` into `Unprovable`. A
  refuter never changes `Proved` nor a search's `Unprovable` (research
  conflict 8).
- Under the caller's stop and a fresh account of the same bound; its own
  bounds (atoms, work) are `Options` fields with `DEFAULT_*` constants and
  flags (C2). It runs on the calling thread, no clock: wasm-safe.

### 8.8 `engine_for`, `Engine::parallel`, the race

`engine_for(forest, goal, mode, &options) -> Result<Engine, Error>` costs
what `prove_goal` costs before it searches; with `Engine::parallel` it is
what a front end asks before it adds threads. The race of one thread and
a pool lives once, in the library, behind `parallel` (R85, F103):

```rust
#[cfg(feature = "parallel")]
pub fn race(forest: &Forest, goal: &[Member], mode: Mode, options: &Options,
            start_pool: impl FnMut(&Progress) -> bool, stop: impl Stop) -> Result<Outcome, Error>;
```

`start_pool` is polled with the single thread's progress, so the caller
decides with its clock and the library reads none; never more threads
than `jobs` (F168): the pool has `jobs − 1` workers, and below three jobs
there is no race; both searches
draw on one account (5.3); the first decisive answer wins and cancels the
other; the statistics are those of the search that answered. The command
and the harness call it (one place, R85).

### 8.9 Names inside (F56, F113)

The engines' structs are named by role, not `Engine`: `focus::Focused`
(the `Decide` value) and `focus::Run` (a run), `net::Linker`,
`additive::Pairs`, `horn::Horn`; `focus::Rules` → `Switches`,
`schedule::Rule` → `Plan`, `check::Problem` → `Fault` (with
`CheckError::fault`), `batch::Answer` → `batch::Report`; the result alias `focus::Search` → `Searched`.

## 9. Exports

- **One signature per target** (F36): `export::<target>::write(&impl
  Drawable, &Options, out: impl fmt::Write, stop: impl Stop) ->
  Result<(), WriteError>` over a sealed trait `Drawable` that
  `proofs::Derivation` and `ordinary::Derivation` implement (the
  crate-private `Drawn` made a sealed public trait, so the wrappers and
  the `String` twins go); `to_string(&impl Drawable, &Options) ->
  String` the one-line form. LK/LJ gain what they lacked by construction.
- **Rocq** (step 31): `rocq::write_proof(&Proof, Option<Mode>, &Options,
  out, stop)`, `Options::kernel: Kernel { Auto, NanoYalla, Linlog }`
  (`Auto` byte-identical to today), `Unsupported` `#[non_exhaustive]` and
  per kernel, with `Cut`, `Cyclic`, `Quantifiers` refusals reserved
  (R139, R161); `rocq::refutation(…)` for certified refutations (R153).
- **SVG ids are a contract** (D-6, R163): `o<n>` an occurrence (a vertex
  from 33), `i<n>-<p>` an inference's formula, `l<m>-<n>` a link;
  prefixes `b`, `d`, `j`, `c` reserved for boxes, doors, jumps, cuts.
- New options are fields (D15): the essential net's arrows and polarity
  colours (35), the planar arcs (36), quantifier spellings and whether
  witnesses show (38) are `svg::Style`, `TextOptions`, `latex::Options`
  fields with today's output as default; `Style` and every options value
  are `#[non_exhaustive]`.

## 10. How each later step enters it

### 10.1 Steps 29 to 34

- **29, the comparison**: `search::race` (8.8), `Options::BENCHEXEC`
  (R151), the public walk (`sequents::Walk`, public at 28 with
  `Visit::{Enter, Between(t, index), Exit}` over an operands function of
  any arity, so a binder is the `Enter`/`Exit` of a unary node and a
  first-order term walk is the same type over `FoTermId`s; `Reading::walk`
  the two-sided one; R245, D-10); LLTP's axioms and conjecture are
  `Sequent::antecedents` (R74).
- **30, the release**: the changelog's policy line names the closed
  enums and the bumps; `core.md` lists every public enum and struct with
  public fields and its choice (R50).
- **31, the Rocq library**: the Rocq `node` takes the member type as a
  parameter (R244), its constructors compared with `Node::NAMES` (R118),
  `Cut`, `Forall`, `Exists` planned (R125); certified refutations through
  `Refutation`'s read-back, `Refutation::check`, `Refuter::Classical` and
  `refute_unknown` (8.7), `rocq::write_proof` and `Kernel` (9); no
  nullary Mix (C3).
- **32, the web client**: `Limits::BROWSER`, `WithProgress`,
  `Schedule::Turns`, the `_within` readers, the error form; the
  derivation's JSON is decided as "the proof plus `ViewOptions` is the
  form, a client rebuilds the derivation" (R4).
- **33, MELL nets**: `ProofStructure::new(forest, Mode)`, a `VertexId`
  continuing the forest's ids (P2), boxes, doors, jumps, `from_proof`
  reading the exponential nodes; forcing the net engine on MELL still
  refuses (R89).
- **34, cut**: cut trees after the conclusion (3.3), `Node::Cut(Member,
  NodeId, NodeId)`, `Rule::Cut`, the `cut` tag and `cuts` key; `roots()`
  stays the conclusion, and the dispatch's features and counts are made
  to read the conclusion's occurrences, counter-neutral (R98); cut refused
  in an ordered mode (R57); `Interactive::cut` a method of its own (R95);
  elimination under `Limits` and a stop (R32).

### 10.2 Step 35: the MLL engines

- **One structure for the symmetry breaks** (R103): the net engine's
  chains of equal literal conclusions (`copy_before`, `copy_after`)
  become `Symmetries`, built once per run in one forest pass: groups of
  literals of one atom and sign "under the same maximal pure tree" (a
  conclusion is its own tree), so the leaf break adds groups and no code
  path; the admissibility test stays in one place. All pruning state is a
  pure function of the links, kept in `link`/`unlink` (R33), and every
  new rule's work per candidate is O(1) or O(atoms touched) inside the
  polled loop (R34).
- **The balance prune** reads per-component, per-atom counts that the
  skeleton's union-find keeps and its undo log restores (R82); the
  structure exposes a component id crate-privately.
- **Equal compound conclusions** need structural classes valid for any
  sequent, not only an optimized one: a crate-private
  `Forest::structural_classes()` (fixed-seed hash, O(n), R83), which the
  inverse method's classes reuse (37).
- **The routing feature** is a `Feature` variant (the pure-tree property,
  the same pass) and a row that replaces or narrows `FewEqualLiterals`
  only where the harness shows it winning; else the row stays as it is.
- **Essential nets are a criterion, not a second engine** (D7, R105):
  the net engine's loop becomes `net::Linker<C: Correctness>`, crate-
  private, with `Switching` (today's Yeo test, unchanged code at
  `C = Switching`, its `nodes`, `links`, `tests` equal before and after)
  and `Essential` (directed acyclicity by an incremental transitive
  closure with an undo log paired with `link`/`unlink`, the dominator
  condition at a complete linking). `Engine::Essential` (`"essential"`)
  is the forceable variant the measurement needs; its row exists only if
  it beats the two-sided engine on IMLL; `parallel()` is `false` until its
  cubes are shown to replay the closure per worker. The closure matrix
  (n² bits) is charged to the account and refused with
  `Reason::MemoryLimit` (R35): the net engine starts taking its account.
  The proof is the sequentialization of the net found (R128).
  `ProofStructure::is_essential(stop)` is the criterion independent of
  any search (R123), with `NetError` witnesses (a directed cycle, a
  missing dominator).
- **Ablation** (R143): `Options::net_prunes: NetPrunes { leaves,
  balance, compounds }`, all on, `#[non_exhaustive]`, documented "every
  setting is sound; for measurement", with a flag in the harness and the
  command.
- Essential-net drawing: `svg::Style` fields (arrows, polarity colours,
  dominators), today's net drawing as default (R144, R166).

### 10.3 Step 36: cyclic MLL and the Lambek calculus

- **Mode** gains, as private fields with builders (additive, `Mode` being
  `#[non_exhaustive]`): `order: Order` (`Commutative` by default,
  `Cyclic`) and `empty_antecedents: bool` (false: the calculus L; true:
  L\*, D16). `Mode::CYCLIC` (classical, cyclic), `Mode::LAMBEK`
  (intuitionistic, cyclic), `LAMBEK.with_empty_antecedents()`; words
  `cyclic`, `lambek`, `lambek-star`; `Mode::check` refuses an ordered
  mode with weakening or Mix (R134). JSON: `"order":"cyclic"` and
  `"empty_antecedents":true` written only when not their defaults (R10).
- **The written order is already canonical** (C1, done at 28). Equality
  stays structural; "the same cyclic sequent" is a rotation, which
  `prove_goal` accepts as the roots in a cyclic mode (`is_roots` up to
  rotation); `Forest::roots()` is the written rotation (R53).
- **The order of the literals is derived, not stored** (this draft's
  answer to R56 and research conflict 3). D1's lowering keeps operand
  order under negation (`(A ⊗ B)⊥` is stored `A⊥ ⅋ B⊥`, the cyclic
  negation being `B⊥ ⅋ A⊥`), and a hash-consed arena cannot tell a
  dualized term from one written so. But in a two-sided Lambek sequent an
  occurrence is in input position exactly when the lowering dualized it
  (the left side and each antecedent flip both, and Lambek input has no
  `~`), so the cyclic form is D1's read in another order:
  `Forest::planar_order(&Reading)` ranks the literals by a walk over the
  input roots in reverse written order, then the goal, visiting an
  input-position binary node's operands right first (`A, B ⊢ A ⊗ B`,
  stored `⊢ ~A, ~B, A ⊗ B`, reads `~B, ~A, A, B`: two non-crossing
  links). One-sided cyclic input has no reading: its order is the
  preorder, and text whose lowering negated a compound is refused in the
  cyclic classical mode (a private flag of `Sequent` the parser sets).
  Step 36's panel checks the lemma; the fallback is a cyclic lowering
  chosen in the reader (`ParseOptions::lowering`), still one meaning per
  `Sequent`. `Term::dual`, the JSON, every stored proof and commutative
  snapshot stay either way.
- **Divisions**: `A \ B` parses to `A⊥ ⅋ B`, `B / A` to `B ⅋ A⊥`. In an
  ordered mode the reading takes the antecedent from whichever factor is
  in input position (both orientations: that is how `\` and `/` differ;
  L has no units, so nothing is ambiguous), unlike 3.1's commutative rule;
  the printers write them back, commutative output byte-identical.
- **The engine is the net engine** with planarity as a third
  constant-time rejection, switched by the mode as `ProofStructure`'s
  criterion is (no new `Engine`: one engine per algorithm); the symmetry
  breaks are off under order; a row `Row { fragment: MLL, modes:
  Modes::Ordered, … engine: Net }`, and every other engine refuses an
  ordered mode with `NotTaken::Mode` (R90). Units, Mix and weakening in
  an ordered mode are refused with named errors (R134).
- **The checker** reads zones in the planar order: one sort per `⊗` to
  test that the premises' contexts are the two arcs, the axiom on
  adjacent literals, and in L the fourth clause R4 (every derived sequent
  has an input occurrence, `Fault::EmptyAntecedent`), the oracle alike
  (R121, R246). `Inference::sequent()` is in planar order in an ordered
  mode; `Split::At(position)` splits an ordered goal at a cut position
  (R96); `Refusal::Exchange` refuses what needs exchange. NanoYalla has
  no cyclic certificate (`rocq::Unsupported::Cyclic`).

### 10.4 Step 37: the focused inverse method

- **First commit, counter-neutral**: `Context` and `Classes` move from
  `search/focus/` to `search/zone/` (shared types; D7 forbids two engines
  of one algorithm, not shared data), `bench/targets.sh` columns equal
  before and after. `Classes` gains the reverse map from a class to its
  occurrences (R84), for renaming a forward derivation back to ids.
- `Engine::Inverse` (`"inverse"`), `parallel()` `false`; `admits` states
  its fragments and modes in a table on the variant (R135).
- The database and its index are charged by capacity (R36); the loop
  polls at every given clause, in every long subsumption scan and index
  rebuild, and in the set-up on a large forest (R37), passing work.
- `Ok(None)` only from a saturation that completed in a fragment where
  it is complete (MALL); with exponentials it gives up with a `Reason`
  (`Stopped`, `MemoryLimit`, `IndexLimit`, or a `SaturationLimit` if a
  multiplicity bound is added, with its option and flag, R111). An
  exhausted saturation gets its refutation from `conclude` (8.1), no new
  `Refutation` unless the engine names one (R137).
- The forward derivation becomes today's `Node` terms (sharing is the
  DAG, a subsuming smaller sequent a `Weaken` below, a `Θ` use a `Quest`
  and `Copy`; the weak flag is the checker's `any`), checked by the
  front door like any proof (R127).
- `Statistics`: the shared counters (8.5); no field unless the harness
  needs one. No `DISPATCH` row unless a measurement earns one; else
  `--engine inverse` only, as the prompt allows.

### 10.5 Step 38: first-order linear logic, in full

**The first step of adding them** is the data model alone, with its
refusal everywhere: the reserved `Term`, `Kind`, `Node` variants, the
tables of 3.9 (empty for propositional input), the `QUANTIFIERS` bit, and
in the same commit every engine's `admits` refusing the bit and
`Proof::check` refusing a forest with arguments (`Fault::FirstOrder`)
until the axiom compares instances (R135: "the data model comes first and
by itself"). Each later commit of 38 makes one layer accept it.

**(a) Atoms as predicates over terms.** `Term::{Pred, DualPred}(Atom,
ArgsId)` with the literal kinds; an atom is `(name, arity)`; the term
arena `fo` (`FoTerm::Bound`, `FoTerm::App`, constants with empty
arguments), argument lists hash-consed in a CSR table, all topological;
`Sequent::check` verifies indices, arities and scope in one ascending
pass (per term its largest loose de Bruijn index, 0 at every root). The
literal lists are by predicate and sign: candidates (3.3).

**(b) Binders in the arena.** `Term::{Forall, Exists}(TermId)`,
`Kind::Forall` negative, `Kind::Exists` positive, `dual` swapping them,
locally nameless (3.9); a binder is a unary occurrence with its body at
`o + 1`, so the forest's numbering and arrays stay, plus a binder-depth
array only when the sequent has quantifiers. The parser's binder entry
on its one stack resolves names against the open binders; 100 000 nested
binders and terms parse, print, unify and check on 256 KiB (R43). The
syntax is the author's (fo-linear §7 Q1); the arena does not depend on it.

**(c) The member as an occurrence and a frame.** A member past
`forest.len()` is `(OccId, FrameId)` in an `Instances` table (3.9) of a
proof, a session or a search; a frame lists the terms the binders above
the occurrence are bound to, hash-consed, so equal instances are equal
members. fo-linear-44 (`!∀x.p(x) ⊢ p(a) ⊗ p(b)`: frames `[a]` and `[b]` of
one body occurrence in one `Θ`) is two members, fo-linear-62's three
copies of one clause three; no signature changes.

**(d) The substitution beside the forest.** The sequent's tables are
read-only; a search keeps beside the forest `Bindings` (per metavariable
an `Option<FoTermId>` and a level), a `Trail` (the variables bound and
the levels lowered, a mark per choice point; undo pops to the mark and
truncates the search's term extension), and counters that number fresh
variables, per worker on a pool, never by address (R39). The
eigenvariable condition is kept by **levels** (fo-linear §3.2: LLV with
an integer): a variable carries the level of its creation, an
eigenvariable one more than its branch's; binding a metavariable to a
term with a younger eigenvariable fails; the occurs check and the
lowering ride on one iterative walk that visits each shared node once
(R106, R250).

**(e) The trail in the focused engine** (step 26's item 7): a mark at a
choice, undone by truncation exactly where the arena's pending nodes are
released: `prove_stable` on a failure; the four places where a first
premise was proved and the second failed (`premises`, `both`, `parts`,
the forced chain in `split`); each failing alternative of `choose_here`;
a frame of `split::chain` whose split failed. Bindings cross the premises
of `⊗` and Mix, undone only when the rule fails, and a failure on the
right must ask the left for **another answer**: one answer per goal
(`Found::Proved`, `Entry::Proved`) loses `p(a) & p(b), q(b) ⊢
∃x.(p(x) ⊗ q(x))` (impact-quantifiers finding 3), so the framed engine
enumerates a premise's answers (a resumable premise search, or witnesses
from a finite candidate set without function symbols); the mechanism is
38's and its panel's, and the propositional `Found` stays. A pool's
worker starts from its spawner's trail; no binding reaches another
alternative; premises sharing an unbound metavariable run in order on
one thread. Generic or duplicated: the spike's answer (11).

**(f) The prunes that assume ground atoms**, each given a first-order
form or switched off for a goal with the bit, the propositional path
unchanged (R107):

| prune | first-order form |
|---|---|
| `initial`, `dual_in`, `dual_from`, `mark_literals`, `meets` | the literal seam (made one internal interface at 28, R109): candidates by predicate symbol, a dual is a member that unifies; several candidates are a choice point |
| forced splits (`Forced::Dual`, `Duals`, `literal_tensor`, cursors) | a choice among unifiable duals; forced only when a candidate is unique and ground |
| interchangeable occurrences (`Classes`, canonical keys, one of each kind, canonical splits) | classes of ground members only; a framed member is its own class |
| the memo and the loop check | ground stable sequents only, keyed by instance; a `Complete` failure may be shared under a variant key later, `Proved` never |
| counts, the count equation, the bias | per predicate symbol: sound, weaker; `Refutation::Unbalanced` names a predicate |
| and-parallel `&`, the cubes | only premises sharing no unbound metavariable |
| the net engine, the additive path, the Horn engine | refuse the bit (first-order Horn is another engine, step 27's report) |

**(g) The dispatch.** The two last rows name `LL1` (the focused
engines); no other row contains the bit; `NoEngine` for none.

**(h) Witnesses and eigenvariables in proofs.** `Node::Forall(Member,
Eigen, NodeId)` and `Node::Exists(Member, FoTermId, NodeId)`, 16 bytes;
`Proof::with_instances` owns the table: the members, the frames, the
proof-local term extension (witnesses over the sequent's symbols and the
proof's eigenvariables; an open metavariable allowed or replaced by a
fresh constant, fo-linear Q3, the author's). The proof an engine returns
is closed under its final substitution (R125). JSON: the tags `∀ [m, a,
p]`, `∃ [m, w, p]` and the keys of 7 (R17); an older reader fails on the
tags.

**The checker** (R126): when the proof has no instances, today's pass,
unchanged (the fast path is a branch per proof, not per node). Otherwise
the same bottom-up pass over members: `Ax` compares the two instances by
id (eigenvariables and open metavariables as constants), `&` compares
zones by member, `Exists(m, w, p)` consumes the body member `(o + 1,
F·w)` of the premise and concludes `m = (o, F)`, `Forall(m, a, p)` the
same with `a`; and the eigenvariable condition as a set of eigenvariables
flowing up with the state: an `Exists` adds its witness's, a `Forall`
removes its own, the root's is empty, and each eigenvariable is
introduced by one `Forall` node (counted by node, since a memo hit shares
a subproof; fo-linear §4.2, the claim for 38's panel). The set is
charged to the pass's bound. New faults: `Witness`, `Eigenvariable`,
`Instance` (`Fault` is `#[non_exhaustive]`). The oracle gets the same
rules, a first-order reference prover is written for the panel.

**(i) Views, sessions, exports.** The derivation prints instances through
the frames and counts substituted characters in its size (R66);
`Inference::datum` holds the witness or the eigenvariable; the session
holds an `Instances` table and `Bindings`, `Step::witness` gives a term
or opens a metavariable that `close` or a later axiom binds, undo undoes
bindings (R97); the printers name bound variables from the side table;
`rocq` answers `Unsupported::Quantifiers` until the Rocq library has
binders (R161).

**(j) Ordinary first-order logic.** `ordinary::Formulas` holds the same
`fo` table and symbol tables as `Sequent` (one term type, 3.8), so
`translate` copies term ids; `ordinary::Node::{Pred, Forall, Exists}`;
the pattern table gains its rows without touching the propositional ones
(R67), the classical image puts `?` on every `∃` (fo-embeddings §1.5),
so `Translation::target()` is `affine LL` and `Unknown` reaches the
classical path; `Image::read_back` reads the proof's witnesses;
`ordinary::Inference::datum`; the Rocq certificate is over a domain `D`
with an inhabitant (impact-fo-ordinary §0.2).

**(k) Bounds and reasons.** `Limits::terms` on every reader (R250);
first-order search bounds as `Options` fields with `DEFAULT_*`, flags and
JSON lines (a witness depth, instances per branch, deepened), each
reaching a `Reason` of its own; `Unprovable` only from a level that met
none of them, with exponentials the copy bound as today (R38). MLL1 and
MALL1 are decided by the bounded search alone.

**(l) What stays untouched for the propositional case** (D17): the
occurrence numbering and every stored id; the sizes of `Term` (12),
`Node` (16), `Kind` (1), `ordinary::Node` (12); every propositional JSON
byte; `Decide`, `Answer`, `Verdict`, `Task`'s propositional fields;
`Mode`; the forest's arrays (the binder depth array is empty without
quantifiers); `Context`, `OccSet`, the memo's records, `Classes`,
`Counts` at their propositional instance; the checker's pass on a proof
without instances; the target set's counters and, within a few percent,
its pinned CPU time (the spike's gates, 11.4, rerun at every 38 commit
that touches `search/` or `proofs/check.rs`).

## 11. The spike

### 11.1 What it answers, and what not

Two questions D17 leaves open, and nothing else:

- **Q1, the data model.** Do the reserved variants (`Term::{Pred,
  DualPred, Forall, Exists}`, `Kind::{Forall, Exists}`, `Node::{Cut,
  Forall, Exists}`), `Member` as every node's operand and every member
  list's element, the empty first-order tables in `Sequent` and `Proof`,
  and the `QUANTIFIERS` bit cost the propositional case anything?
  Expected: nothing; the size assertions say the layout is the same, but
  wider matches, larger `Sequent` clones and new arms in the checker's
  hot `match` are what a count shows and a reading does not.
- **Q2, the zone parameter (D-7).** Does a focused engine generic over
  its zone, with a second, first-order instance compiled into the same
  binary and reachable from the dispatch, cost the propositional
  instance? Monomorphisation at `Ground` should give the code of today;
  what it does not settle is the trait's hooks on the hot path (the trail
  marks, the literal seam) and the second instance's code beside the
  first (fo-linear §5.5).

It does not measure first-order speed, nor build unification, the
parser, the JSON, the instance checker or the session.

### 11.2 Where, and against what

A jj workspace of its own on the stage-2 tree (`jj workspace add`, by
the design session; thrown away, nothing merged), its own target
directory, `--release` builds of `linlog-cli` and `linlog-bench`. The
base is the same tree unmodified, built in the same environment (the
devshell's; bench.md: the shell's and the flake's builds differ by up to
0.9 % on `render-svg`, so a comparison takes both counts from one).
Each milestone is a commit of the workspace measured against the base,
not against the milestone before; the deltas between milestones are
reported for attribution.

### 11.3 What it builds: three milestones

**M1, the data model** (Q1; about 600 changed lines, mostly mechanical):
the reserved variants appended to `Term`, `Kind` (today's discriminants
kept) and `Node`, every exhaustive match extended (`dual` swaps the
binders, `operands` gives a binder its body and a predicate none,
`Kind::polarity` and `Kind::fragment` as in 10.5 (b), the readers
refusing the new terms, the checker refusing the new nodes with a stub
fault), the assertions of 3.2; the empty first-order fields of 3.9 in
`Sequent`, in its derived `Eq` and `Hash` and in the clone the forest
keeps, and `Sequent::fragment` setting `QUANTIFIERS`; `Member` as every
node's operand and every member list's element (3.4), converted at the
boundaries; `Proof` with an empty instance table and the checker's
branch per proof; every engine's `admits` refusing the bit.

**M2, the zone parameter, one instance** (Q2's first half; about 1 500
lines touched in `search/focus/`): `Context` and `Classes` lifted to
`search/zone/` (step 37's move), and the focused engine made generic:

```rust
/// What the focused engine's zones hold and how their literals meet.
pub(crate) trait Zone: Sized {
    /// A member of a zone: `OccId` for `Ground`.
    type Member: Copy + Eq + Ord + Hash + Debug + Send + Sync;
    /// The linear zone, a multiset: `Context` for `Ground`.
    type Linear: Linear<Self::Member>;
    /// The unrestricted zone, a set: `OccSet` for `Ground`.
    type Unrestricted: Unrestricted<Self::Member>;
    /// A point of the trail to undo to: `()` for `Ground`.
    type Mark: Copy;

    fn occurrence(&self, m: Self::Member) -> OccId;
    /// The member of a subformula of a member's occurrence (its frame inherited).
    fn child(&self, m: Self::Member, child: OccId) -> Self::Member;
    /// The literal seam (R109): the candidates in a zone that close an
    /// axiom with a literal member, first one first; the marks of
    /// `mark_literals` and `meets`; the forced dual of a positive literal factor.
    fn dual_in(&self, literal: Self::Member, within: impl Fn(Self::Member) -> bool) -> Option<Self::Member>;
    fn mark_literals(&mut self, members: &[Self::Member]);
    fn meets(&self, formula: Self::Member) -> bool;
    /// Interchangeable members: `Classes::of` for `Ground`, none for a framed one.
    fn class(&self, classes: &Classes, m: Self::Member) -> Option<OccId>;
    /// Whether a stable sequent may be memoized: always for `Ground`.
    fn memoizes(&self, theta: &Self::Unrestricted, gamma: &Self::Linear) -> bool;
    /// The trail: a mark at a choice, undone where pending nodes are released.
    fn mark(&self) -> Self::Mark;
    fn undo(&mut self, mark: Self::Mark);
}
```

`focus::Run<'a, Z>`, `Problem<'a, Z>`, `Key<Z>`, the memo's records,
`split.rs`, `parallel.rs` and `schedule.rs` take `Z`; every literal
lookup of today (`initial`, `dual_in`, `dual_from` and its cursors,
`mark_literals`, `meets`, `forced_side`, `literal_tensor`) goes through
the seam, and `mark`/`undo` are called at the places of 10.5 (e) on
every path, as 38 will call them. `Ground` holds what the seam reads
today (the forest, and the literal lists' marks and stamp, moved from the
run); its methods are `#[inline]` forwards to today's code, its `undo`
does nothing; the front door calls `search_goal::<Ground>` only.

**M3, the second instance** (Q2's second half; about 400 new lines):
`Framed`, a stub of real code: members resolved through a per-search
`Instances` table; `Linear` a `Context` of plain members plus a sorted
`Vec<(Member, u32)>` of framed ones, `Unrestricted` an `OccSet` plus a
sorted vector, the trail a `Vec<u32>` with marks; duals by predicate from
the forest's lists, matched by their instances' argument ids (ground
matching: no unification, a binder in the goal refused); the memo for
ground members only. `Focused::decide` sends a goal with `QUANTIFIERS`
to `search_goal::<Framed>`, so the second monomorphisation is in the
binary and no propositional input reaches it; a unit test builds `⊢
p(c), ~p(c)` and `⊢ p(c) ⊗ q(d), ~p(c), ~q(d)` by hand and proves both
through it, so the instance runs.

### 11.4 The gates

Measured on a quiet machine as stage 3's timed runs are (cores 2 and 3,
nothing heavy beside, no timer due):

- **G1, exactness** (must hold, never traded): `bench/targets.sh spike-mN`
  has the columns `verdict`, `nodes`, `splits`, `memo_hits`,
  `memo_entries` of every decided row equal to the base's run of the
  same day and to the committed oracle (`after-coverability.csv`, the
  one bench.md now names, F181) where both decide. A difference is a bug
  of the spike, fixed and measured again, never a cost.
- **G2, instructions** (the main gate; counts are deterministic): the
  ratchet's twenty journeys under callgrind, base and milestone in one
  build environment. Each search journey (`search-*`, `batch-families`,
  `ordinary-pigeons`) at most **+1.0 %**, every other journey at most
  +2.0 % (the ratchet's tolerance), and the sum of the search journeys at
  most **+0.5 %**.
- **G3, time** (D17's "within a few percent"): the target set's rows that
  take a second or more in the base, three runs each, base and milestone
  interleaved on core 2: the geometric mean of the per-row median ratios
  at most **1.02**, no row above 1.05 unless its own base runs spread as
  much; and the journeys' `--repeat 20` medians within +2 % (counts do
  not always track time: bench.md's four journeys).
- **Reported, not gated**: the release binary's text size and the focused
  engine's share of it, base and milestone, which is what M3 adds.

### 11.5 What the results decide

| result | decision recorded in `api.md` |
|---|---|
| M1 passes | The data model of 3.2 to 3.9 stands; area 1 lands its step-28 parts (`Member`, the renames, the assertions, the docs of `Fragment`); the variants come at 34 and 38 as planned. |
| M1 fails G2 or G3 | Bisect its four parts (variants, `Sequent` fields, `Member`, the checker's branch); the failing part is redesigned before 38 (an empty table behind one `Box` instead of five `Vec`s; binders' kinds in a side array of the forest instead of `Kind`), measured again, and recorded. |
| M2 and M3 pass | **D-7 adopted**: step 38 makes the focused engine generic over `Z: Zone` with this trait, `Ground` the propositional instance, after step 37's lift. |
| M2 passes, M3 fails | The cost is the second instance's code, not genericity: its entry `#[inline(never)]` and its cold paths outlined, measured once more; if it still fails, the fallback below with the generic zones kept. |
| M2 fails | **D17's duplicated fast path**: the propositional engine stays as it is; step 38's framed engine is a sibling module that shares the zones, counts, bias, arena, scratch and classes but not the hot loop, the one exception to D7, recorded under D17's name. |

The numbers, per journey and per row, go beside the decision; step 38
reruns G1 and G2 on every commit that touches `search/` or
`proofs/check.rs` (10.5 (l)).

## 12. Findings answered

F1 (P3, 2.2); F5, F37 (5.4); F6 (2.2, 5.4); F7, F40 (3.5); F8, F30, F31,
F34, F48 (4); F10 (3.6, 3.7); F12, F21 (5.4); F13 (3.8); F15, F38 (6);
F16 (3.1); F18, F86, F87, F88 (7, 8.5); F19 (3.4); F22, F23 (3.6); F24
(3.1); F25 (3.2); F26 (7); F27 (3.3, 5.4); F28 (3.5); F36 (9); F49 (3.7);
F51 (2.1); F56, F113 (8.9); F58 (3.3); F60, F61 (3.6, 7); F62, F63
(5.1); F66 (3.5); F68, F69 (3.6); F70 (7); F74 (5.4); F76 to F79 (6,
2.2); F80 (2.2); F84, F85 (3.2, P3); F89 (7, 8.1); F103, F104, F168
(8.8, 5.3); F110, F136 (5.2); F128 (8.7); F139 (8.1, 4); F140, F141
(8.3, 8.4, 8.6); F144 (8.5); H8, H9, H10, H19 (3.1); H18 (5.3); HD1 to
HD5 (13).

Requirements answered beyond those: R9, R10, R21, R29, R33 to R39, R43,
R45, R47, R51 to R57, R59, R61 to R67, R73, R74, R79, R81 to R84, R86 to
R90, R96, R97, R103 to R111, R118 to R128, R135, R137, R138, R142 to
R151, R241, R243 to R246, R248 to R251, R254. Placed with a later step
rather than met at 28: R20, R72, R112 (31), R76 to R78 (33), R80, R95,
R100, R101 (34), R113, R114.

## 13. Decisions for the author

**The open ones, on their recommended answers:**

| # | provisional | what the other answer changes here |
|---|---|---|
| C1 | written order now | B: `optimize` sorts until 36, which then breaks every stored id as a version; `antecedents` lands anyway (H10). C: ruled out by 3.1 and 10.3. |
| C2 | refuters after the search, `refute_unknown` off | A: the kind keeps one member and the classical search goes into `rocq`; B: no `refute_unknown`. 8.7's place stays. |
| C3 | no nullary Mix | B adds a closed `Node` variant before 31 and a checker rule. |
| T1 | rename keys with the version | 7's aliases go. |
| T2 | owned forests | `Arc<Forest>` inside `Proof`, `ProofStructure`, `Interactive`; signatures keep their shape. |
| T5 | shared counters, documented per engine | Per-engine counters change the CSV columns; 8.5 becomes per-engine structs. |
| HD1 | refuse several conjectures | Conjoining changes the readers only. |
| HD3 | NFC identifiers | Code-point identity: 3.1's sentence goes. |
| HD5 | refuse non-identifier atom names | The printers quote them instead. |
| T3, T4, T6, T7, HD2, HD4 | as the audit recommends | No effect on this design. |

**Decisions this draft adds:**

1. **The written sides** (`Sequent::antecedents`, a new optional JSON
   key): set by the parser, `lltp` and `translate`, read by the reading.
   Alternative: refuse every intuitionistic sequent with two roots that
   can be output (no data, but a JSON or LLTP sequent with a `⊤`-built
   hypothesis would then be refused too).
2. **The planar order is derived, D1's lowering stays** (10.3).
   Alternative: a reversing dual in every mode (one stored form, but D1
   re-litigated and every printed hypothesis `A ⊸ B` becoming `~B ⊗ A`),
   or a cyclic lowering chosen at reading time (kept as the fallback).
3. **A member is one integer** past the forest into a table (3.4), against
   fo-linear's pair. Alternative: `(OccId, FrameId)` everywhere, a shape
   change of every stored form.
4. **Options are public fields** (`#[non_exhaustive]`, `Default`, serde,
   `#[must_use]` builders), `search::Options` included, its clamps applied
   where read. Alternative: private fields and setters for all (today's
   search options), which needs a proxy per form for serde.
5. **The refuters are a kind from 28**, with the counts as the first
   member. Alternative (C2's text): wait for a second refuter.
6. **The race takes a progress predicate** (`start_pool`), so the library
   reads no clock. Alternative: a `Duration` behind `parallel`.
7. **`Engine::Essential` is a variant; cyclic mode is the net engine's.**
   An essential search is forced for its measurement; the planar search is
   the net engine under the mode's criterion, as `ProofStructure`'s is.
   Alternative: `Engine::Planar` too, or the essential criterion chosen
   inside `Nets::decide` from the reading.
8. **First-order terms are `FoTerm`/`FoTermId`**, as the specification
   names them; formula nodes stay `Term` (the JSON key `terms` and every
   public signature that names a formula id). Alternative: rename formula nodes now and give
   `Term` to first-order terms, the cleaner vocabulary at the price of a
   rename of every public signature that names a formula id.
9. **Dispatch thresholds are not options** (6). Alternative: an option
   per row's threshold (`NET_MULTIPLICITY`), as R149 reads D16 literally.
