# linlog's API after step 28: the design (draft B)

Draft B of `plan/notes/api.md`, written from the angle of the proof term,
the checker and the Rocq library. It designs the whole surface, and is
most detailed where the trusted core is: `Proof`, `Node`, the forest's
contract, the member type, the checker and its specification, the
derivation view, interactive proving as a partial derivation, the Rocq
library's mirror of the term, cut, MELL nets, certified refutations and
witnesses. It starts from the recommended answers of C1 to C3, T1 to T7
and HD1 to HD5 (section 13 says what another answer changes). Claims
about the code were checked against the tree of 2026-10-09; a claim not
checked says "(unverified)". Rust is edition 2024; every signature is
the public one after step 28 unless it says which step adds it.

## 1. Principles

**P1. The trusted core is small, independent and specified.** The term,
the forest's numbering and the checker are what a proof is trusted on
and what the Rocq library (31) mirrors. The checker, the refutation
checker and the net criterion share no code with any engine; each is a
total function of (term, forest, mode), stated rule by rule (3.6) so a
foreign checker recomputes it. Refusals that exist because Rust's
resources are finite (memory, a stop, an index width) lie outside that
function and are never verdicts.

**P2. A refusal is a kind of its own, in every error.** Every error has
an `ErrorKind`; `Refused` (a bound or the stop) and `Unsupported`
(outside what linlog, the engine or the kernel does) are no verdict.
An error type that can be refused has a variant `Refused(Refusal)` no
fault shares, so a `match` that reads "invalid" never sees one (S4,
S18; today's `Problem::Memory` beside `Problem::Surplus` is the hazard).

**P3. Every long call takes the bounds and the stop**: one plain-data
`Limits` (memory, occurrences, output size, work) and one `Stop` told
the work done since the last poll; readers of JSON take the limits
through a `DeserializeSeed` (S1, D11, F12, F62, F136).

**P4. Options are data**: one value per feature, `Default`, `Clone`,
`PartialEq`, serde with `default, deny_unknown_fields`,
`#[non_exhaustive]`, named `DEFAULT_*` constants, presets as values
(D15, D16).

**P5. Types say what they hold, and grow without breaking.** A member of
a sequent is a `Member`, a net vertex a `VertexId`, a session's goal a
`GoalId`, never a bare `OccId` standing for one. What a named step
extends is `#[non_exhaustive]`, except the closed core enums `Term`,
`Kind`, `Node`, `Rule`, `Verdict`, `Side`, `Sign`, `Position`, whose new
variants are meant as compile errors downstream and come with planned
0.y bumps (0.2.0 at step 34, 0.3.0 at step 38; R50, F1; decision B1).

**P6. Wire forms are versioned and additive.** Every form read back
carries `"version"`; a writer writes the lowest version that represents
the value, so propositional, cut-free values stay version 1 and
byte-identical; a reader refuses a newer version by name (D-3, F26,
AIP-180).

**P7. Owned values, borrowed views.** `Proof`, `ProofStructure`,
`Interactive` own their `Forest` (T2); `Derivation`, `Reading`, `Goal`,
`Described` borrow.

**P8. The propositional case pays nothing (D17).** Later steps add empty
tables, additive constructors, reserved keys or monomorphised
parameters; the size assertions (`Term` 12 bytes, `Node` 16) and the
target set's counters gate them, and the spike (11) measures the one
change whose cost is not zero by construction.

## 2. The public surface after step 28

### 2.1 The module tree

```
linlog
├── sequents      Sequent, SequentBuilder, Term, Kind, Atom, TermId, Formula, Walk, Visit
├── fragment      Fragment, Mode
├── occurrences   Forest, OccId, Member, Sign, Polarity, Position, Reading, IllFormula, ShapeError
├── proofs        Proof, Node, NodeId, Side
│   ├── check     CheckError, Invalid, Fault, Refused, Dyadic        (the checker; functions private)
│   ├── derivation Derivation, Inference, InfId, Rule, Named, ViewOptions, Compact, Sides, ViewError
│   ├── size      Size
│   ├── style     Labels, OpenGoal, WriteError, TextOptions
│   └── interactive (feature) Interactive, GoalId, Step, Split, StepError
├── refutation    Refutation, Unbalanced, Equation, StateEquation, Disproof, RefutationError   (new)
├── nets          ProofStructure, VertexId, Criterion, NetError, Scratch
├── search        prove, prove_until, prove_goal, engine_for, Goal, Options, Outcome, Verdict,
│   │             Reason, Statistics, Engine, Bias, Pool (feature parallel), race (parallel)
│   └── batch     run, prove, Options, Plan, Problem, Answer, Cores, Results
├── limits        Limits, Refusal, Stop, Progress, WithProgress                    (new)
├── forms         Within (feature serialize): reading any form within Limits     (new)
├── export        Form, Styles, RenderError; latex, typst, svg, png, pdf, rocq
├── ordinary      Sequent, Formulas, Node, Image, Derivation, Inference, Rule, Logic, Translation, Options, …
├── lltp, mist, families   (feature parse)
└── (private)     errors, hash, parse, serialize, occurrences::set
```

`lib.rs` re-exports, at the root: `Sequent, SequentBuilder, Term, Kind,
Atom, TermId, Formula`; `Fragment, Mode`; `Forest, OccId, Member, Sign,
Position, Reading, ShapeError`; `Proof, Node, NodeId, Side, CheckError,
Derivation, Inference, InfId, Rule, Named, ViewOptions, Compact, Size`;
`Interactive, GoalId, Step, StepError` (feature `interactive`);
`Refutation, Disproof`; `ProofStructure, VertexId, Criterion, NetError`;
`prove, prove_until, prove_goal, Goal, Options, Outcome, Verdict,
Reason, Statistics, Engine, Bias`; `Limits, Refusal, Stop, Progress`;
`Error, ErrorKind, Described`; `ParseError` (feature `parse`).

No longer public: `OccSet` and its `Iter`, `Forest::empty_set` and
`root_set` (F51: no public function takes a set, and its width rules
would become API the zone types must keep); `proofs::check::check` and
`check_within` as free functions (F51; the methods on `Proof` stay);
`Sequent::verify_integrity` (F51: every public way to a `Sequent` keeps
the invariant); `linlog::Scratch` at the root (it stays as
`nets::Scratch`; a second structure of step 33 or 34 would make the root
name ambiguous); `DEFAULT_MEMORY_LIMIT` at the root (one constant,
`Limits::DEFAULT_MEMORY_BYTES`). `nets::Described`, `proofs::Described`
and `DescribedShape` become the one `Described<'a, E>` (F48).

**Marked `#[non_exhaustive]` at 28** (F1, R50), with each struct-like
variant of a marked enum: `Error`, `ErrorKind`, `ParseError`,
`ShapeError`, `CheckError`, `Invalid`, `Refused`, `Fault`, `Dyadic`,
`ViewError`, `WriteError`, `RenderError`, `NetError`, `StepError`,
`RefutationError`, `Refusal`, `Progress`, `Limits`, `Mode`, `Criterion`,
`Inference`, `Size`, `Named`, `Step`, `Split`, `Compact`, `Sides`,
`Form`, `Item`, `Outcome`, `Reason`, `Statistics`, `Engine`, `Bias`,
`Refutation` and its payloads, every options value (`search`, `batch`,
`ViewOptions`, `TextOptions`, `latex`, `typst`, `svg::Style`, `png`,
`pdf`, `rocq` and `Kernel`, `Styles`), `Labels`, `OpenGoal`,
`lltp::Status`, `batch::{Plan, Problem, Answer, Cores}`,
`ordinary::{Options, Logic, Translation, Inference, Procedure}`. Closed
on purpose, each new variant a planned bump: `Term`, `Kind`, `Node`,
`Rule`, `Verdict`, `Side`, `Sign`, `Position`, and `ordinary::{Node,
Rule}` like their linear twins.

### 2.2 Before and after

Every public item renamed, moved, merged, retyped or removed; additions
are in sections 3 to 9.

| before | after | why |
|---|---|---|
| `Term`/`Kind::{Var, DualVar}`, `Sign::{Var, DualVar}` | `Term`/`Kind::{Atom, DualAtom}`, `Sign::{Atom, Dual}`, derived order kept (`2·atom + sign`) | F25, R251 |
| `Error::InvalidVariableIndex`, `TermIndexOutOfBounds`, `SubtermIndexNotDecreasing`, `NodeIndexOutOfBounds`, `PremiseIndexNotDecreasing`, `OccurrenceIndexOutOfBounds` (positional) | `AtomOutOfBounds`, `TermOutOfBounds`, `SubtermNotBefore`, `NodeOutOfBounds`, `PremiseNotBefore`, `MemberOutOfBounds`, named fields | F25, F31 |
| `Error::{TooManyOccurrences, TooManyNodes}` | `Error::Refused(Refusal::{Occurrences, Index})` | P2, F30 |
| `Error::{InvalidProof, Unchecked}(CheckError)`, `Rejected(Box<CheckError>)`, `InvalidNet`, `Refused(interactive::Refusal)`, `InconsistentState(&str)` | `Error::Check(CheckError)`, `Rejected(Box<check::Invalid>)` (a `Defect`), `Net`, `Step(StepError)`, `State { reason }` | S18, F31 |
| `Error::{SequentParsing, Lltp, Mist, Tptp}` | `Error::Parse(Vec<ParseError>)`, `Error::Read { format, reason }` | F31 |
| `CheckError { node, rule, premises, problem }`, `is_refusal`; `Problem` | `CheckError::{Invalid(Box<Invalid>), Refused(Refused)}`; `Fault` without `Memory` | P2, F56 |
| `interactive::Refusal` | `StepError` | P2 |
| `Dyadic`, `Inference::sequent`, `Node`'s operands, `Node::occurrences()` over `OccId` | over `Member`; `Node::members()`, `Node::TAGS` | F19, R118 |
| `Proof::{derivation_with(&view, stop), two_sided_derivation(_with), derivation_size(two_sided), derivation_size_within, check_within(mode, memory)}` | `derivation_with(&view, &limits, stop)` (`view.sides`), `derivation_size(&view, &limits, stop)`, `check_within(mode, &limits, stop)` | F10, F12, F62 |
| `ViewOptions { limit, memory, compact }`, `DEFAULT_LIMIT`, `UNBOUNDED` | `ViewOptions { compact, sides }`; bounds in `Limits` | F62, F63 |
| `DEFAULT_MEMORY_LIMIT` (root, `search::Options`, `batch::Options`), `png`/`pdf::Options::DEFAULT_MEMORY`, `Options::{memory_limit, occurrence_limit}`, `DEFAULT_OCCURRENCE_LIMIT` | `Limits::{DEFAULT_MEMORY_BYTES, DEFAULT_OCCURRENCES}`, `Options::limits(Limits)` | F62, F63, F79 |
| `Rule` (34 variants), `Rule::ALL: [Rule; 34]`, `classical()`, `intuitionistic()` | `Rule` (15 one-sided rules and `Open`), `ALL: &'static [Rule]`, `Named { rule, side }` | F60, R61 |
| `Derivation::two_sided(proof, view, stop)` | `Derivation::new(proof, &view, &limits, stop)` | F10 |
| `Interactive::{apply(goal, position, rule, left), new(&sequent, mode), close_with(..), close_all(..) -> Result<Vec<..>>}`, `InfId` for session ids, `inferences()` | `apply(goal, &Step)`, `new(.., &limits)`, `close_with` refusing a foreign forest or mode, `close_all -> Vec<(GoalId, Result<..>)>`, `GoalId`; `inferences()` removed | F22, F27, F23, F68, F69 |
| `ProofStructure::{new(forest, mix), from_links(forest, mix, ..), from_proof(proof, mix), mix(), is_correct(), sequentialize()}`, `partner`, `links`, `link`, `unlink`, `unlinked`, `same_component` over `OccId` | `new(forest, &Criterion)`, `from_links(.., &Criterion, ..)`, `from_proof(&proof, &Criterion, &limits, stop)`, `criterion()`, `is_correct(&limits, stop)`, `sequentialize(&limits, stop)`; the rest over `VertexId` | F10, F12, R79, D-6 |
| `NetError` (exhaustive, `OccId`) | non-exhaustive, `VertexId`, `Refused(Refusal)` | F1, R132 |
| `Mode { pub … }`, `Mode::affine()` | non-exhaustive `Mode`, `with_affine()`, `name`, `NAMES`, `FromStr` | F7, F40, R241 |
| `Fragment::ALL` | `Fragment::ADDITIVE` (named `ALL`) | F56 |
| `Verdict::Unprovable(Refutation)`; `Refutation::{Unbalanced { atom, name, .. }, Equation {..}, StateEquation { weights, once }}` | `Unprovable(Box<Disproof>)`; payload structs, no `name`, every place's weight | R3, R124, F80, R70 |
| `Reason::{RecursionLimit, CopyBound(u32), MemoryLimit(u64)}` | `RecursionLimit { limit }`, `CopyBound { bound }`, `MemoryLimit { bytes }`, new `WorkLimit`, `Unchecked` | F86, R138 |
| `prove_goal(forest, &[OccId], ..)`, `engine_for(forest, &[OccId], ..)`; every `impl FnMut() -> bool` stop | `prove_goal(Goal, ..)`, `engine_for(Goal, ..)`; `impl Stop` | F19, F136 |
| `search::Options` without serde, `Engine` written only, `Bias` without serde | serde; `FromStr` for `Engine`, `Bias`, `Mode`, `Fragment`; `Engine::ALL` | F76, F77, R1 |
| `batch::Options::plan(within: bool)`, public batch fields | `plan(Cores)`, non-exhaustive, serde on `Options` | F10, R150 |
| `latex`/`typst`/`svg::{sequent, two_sided, derivation, ordinary, write}`, `svg::net` (`TooLarge`), `rocq::{derivation, write, ordinary}` | one `write(Item, &options, &limits, out, stop)` per target; `TooLarge` is `Refusal::Output` | F36, F5, R26, R152 |
| `rocq::Options { form, lemma: String, prelude: String }`, `rocq::Unsupported` (exhaustive) | `{ form, lemma: Identifier, prelude: Option<String>, kernel }`, `Unsupported::{Open, Rule, Mode, Refutation}` | R139, R140, F38, F15 |
| `png`/`pdf::from_svg(svg, fonts, &options)` | `+ &limits` | F62 |
| the command's `Styles` | `export::Styles` | R141 |
| `ordinary::Sequent::new -> Self`; `Image::read_back(&linear)`; `ordinary::Derivation::check()` | `-> Result`; `read_back(&proof, &view, &limits, stop)`; `check(&limits, stop)` | F13, F12, R24 |
| `Sequent::optimize() -> Result` sorting roots; `Forest::{new(&s), within, from_owned}`, `TryFrom<Sequent>` | `optimize()` infallible, written order; `Forest::new(&s, &limits)`, readers through `forms::Within`; `Sequent::parse_within` | C1, F24, F27, F16 |

## 3. The data model

### 3.1 Sequents, terms and atoms

```rust
/// An atom: a predicate symbol, by its index in a sequent's atom table
/// (arity 0 until step 38).
pub struct Atom(u32);
pub struct TermId(u32);
/// A node of a formula in negation normal form. Closed (P5).
pub enum Term {
    Atom(Atom), DualAtom(Atom),                         // were Var, DualVar (F25)
    One, Bot, Top, Zero,
    Tensor(TermId, TermId), Par(TermId, TermId), With(TermId, TermId), Plus(TermId, TermId),
    Bang(TermId), Quest(TermId),
    // 38: Pred(Atom, Args), DualPred(Atom, Args), Forall(TermId), Exists(TermId)
}
const _: () = assert!(size_of::<Term>() == 12);          // new (F84, R63)
#[repr(u8)] pub enum Kind { Atom, DualAtom, One, Bot, Top, Zero, Tensor, Par, With, Plus, Bang, Quest }
```

`Sequent` keeps private fields; step 38 adds, empty until then, a
first-order term arena with an argument table, the symbol table
`(name, arity)`, the predicate arities and the binder names (for
printing only), all private, so additive.

**The canonical form is the written one (C1, answer A).** `optimize`
hash-conses, merges atoms by name and drops unreachable terms, and no
longer sorts the roots (it becomes infallible); arena indices follow the
first occurrence along the roots, as the parser assigns them
(unverified after hash-consing: the fix checks it). Equality
and `Hash` compare the written root list; a cyclic sequent (36) is not
equal to its rotations. The one commit that stops sorting regenerates
the snapshots and README blocks whose roots were not ascending, names
them as its exemption from the behaviour lock, and lists the target
rows whose root order changed.

**The walk is public** (R245, D-10), shaped for terms:

```rust
#[non_exhaustive]
pub enum Visit { Enter(TermId), Between(TermId, u32 /* before operand index ≥ 1 */), Exit(TermId) }
impl Sequent { pub fn walk(&self, root: TermId) -> Walk<'_> }      // Walk: Iterator<Item = Visit>
```

A binder is a unary node (its `Enter` is the binder stop); the operand
index makes argument lists fit; 38 adds `walk_args` and maybe
`Visit::Bound`. Every printer stays a loop over it; `core/tests/depth.rs`
runs it at 100 000 levels.

**A checked builder** (R59, R58) for cut, the families, substitution and
the web client:

```rust
impl SequentBuilder {
    pub fn new() -> Self;
    pub fn atom(&mut self, name: &str) -> Atom;                    // interns by name
    pub fn term(&mut self, term: Term) -> Result<TermId, Error>;   // checks indices, hash-conses
    pub fn dual(&mut self, term: TermId) -> TermId;                 // NNF dual, child order kept
    pub fn root(&mut self, term: TermId) -> Result<(), Error>;
    pub fn build(self) -> Sequent;                                  // equal to the parsed one
}
```

### 3.2 The forest's contract

The rustdoc of `Forest` states three things as public contract, each
with a test, because the Rocq checker, stored proofs and sessions and
the SVG ids rest on them (R69, D-2):

1. **The numbering is a promise a foreign checker recomputes**: a
   depth-first preorder, the conclusion's roots in written order (C1),
   the left subterm before the right, a unary operand at `o + 1`, so
   `subtree(o) == o .. o + size(o)`; a pure function of the sequent and
   the recorded extra trees. A fixture (`core/tests/forest.rs`) pins
   `id → (kind, parent, atom)` for a dozen sequents; the Rocq test suite
   reads the same file.
2. **Conclusion, trees and arena are three things.** `roots()` is the
   conclusion. Ids may continue past its trees with **extra trees** the
   forest records: step 34's cut pairs, `A` then `A⊥`, in the order
   added. `sequent()` is the conclusion, whose arena may hold terms its
   roots do not reach. `len()` and `ids()` cover every tree.
3. **A cut pair's dual is by offset**: NNF duality keeps the shape and
   the child order, so `dual(x) = root(A⊥) + (x − root(A))`. This holds
   for the commutative dual only; in an ordered mode (36) cut is refused
   until a step asks for it, and then the dual is a table.

```rust
impl Forest {
    pub fn new(sequent: &Sequent, limits: &Limits) -> Result<Self, Error>;   // was new + within
    pub fn cut_pairs(&self) -> &[(OccId, OccId)];        // new: empty until 34
    pub fn dual(&self, o: OccId) -> Option<OccId>;       // new: Some under a cut tree
    pub fn is_conclusion(&self, o: OccId) -> bool;       // new: under a root of the conclusion
    pub fn dual_literals(&self, x: OccId, y: OccId) -> bool;   // new (F58): the axiom-partner test
    // unchanged: roots, sequent, len, ids, term, kind, formula, parent, root, size, depth,
    // atom, sign, left, right, children, subtree, is_below, lca, literals, all_literals
}
impl PartialEq for Forest   // new: equal sequents and extra trees, hence equal numbering (D5)
```

`within`, `from_owned` and `TryFrom<Sequent>` fold into `new` (the
readers use `forms::Within`). `dual_literals` is the one place the eight
axiom-partner sites of F58 call; at 38 it compares instances and until
then refuses a literal with arguments. The search's `is_roots` becomes
"the goal is every tree" (`Goal::is_whole_forest`), which the net engine
requires, so a cut tree never reaches it (impact-boxes item 10).

### 3.3 The member of a sequent

Under `?` copies with quantifiers (38) one occurrence has several
instances, and the bottom-up checker cannot recover a leaf's instance
from the nodes below it (impact-quantifiers finding 2), nor a `⊗`'s
principal when two instances of it share a zone. So every node operand
and every listed sequent names a **member** (D-1 option b, R244, F19):

```rust
/// A member of a sequent of its owner (a proof, a derivation, a session,
/// a goal). In a ground owner the members are the occurrences:
/// `Member(i)` is `OccId(i)`. A first-order owner (38) has an interned
/// table of instances `(OccId, FrameId)`, and `Member(i)` is entry `i`.
pub struct Member(u32);
impl From<OccId> for Member      // the occurrence as a member of a ground owner
```

- **One integer on the wire**, now and at 38: no JSON shape changes.
- **Every field and signature that lists members takes `Member`**:
  `Node`'s operands, `Dyadic`, `Inference::sequent`,
  `Interactive::goal`, `Goal`, `Fault`'s payloads, and the crate-private
  `Drawn::sequent` and `Notation::sequent`. `Forest`, `Reading` and
  `ProofStructure::vertex` keep `OccId`: they speak of occurrences.
- **Owners map members back**: `occurrence(m) -> OccId` and
  `formula(m) -> impl Display + '_` on `Proof`, `Derivation`,
  `Interactive`; the identity today, a table lookup and an instance
  printed through its frame at 38. Callers through them never change.
- **The table is interned**, so zones stay multisets of `u32` compared by
  id and `&`'s "equal zones" stays a table equality; a frame is a
  hash-consed list of what the binders above the occurrence are bound
  to. Rejected: a pair in every operand (breaks the 16-byte `Node` and
  every JSON shape) and a parallel `frames` list (two lists that can
  disagree).
- **The zones the engines instantiate** (D-7): the ground zone is today's
  `Context`; 38's framed zone is a bitset over empty-frame occurrences
  plus a sorted vector of framed members; both implement a crate-private
  `trait Zone`, the focused engine `Engine<Z: Zone>` monomorphised at
  `Context` if the spike accepts it (11), a duplicated module if not.

### 3.4 Mode, fragment, reading

```rust
#[non_exhaustive]
pub struct Mode { pub intuitionistic: bool, pub affine: bool, pub mix: bool }   // read-only outside
impl Mode {
    pub const CLASSICAL: Self; pub const INTUITIONISTIC: Self;
    #[must_use] pub const fn with_affine(self) -> Self;      // was affine() (F40, F64)
    #[must_use] pub const fn with_mix(self) -> Self;
    pub const fn name(self) -> &'static str;                 // R241
    pub const NAMES: &'static [(&'static str, Mode)];
}
impl FromStr for Mode                                        // inverse of name
```

Words: `classical`, `mix`, `affine`, `mix-affine`, `intuitionistic`,
`intuitionistic-affine` (B7); `Display` stays prose. Step 36 adds `pub
order: Order` and `validate()`; the JSON object refuses unknown keys, so
a 0.1.0 reader refuses `"order"` instead of reading a cyclic mode as
commutative (R10).

`Fragment` keeps its `u8` with three free bits: 36 may take one for the
divisions, 38 takes `FIRST_ORDER`, set by a predicate argument as well
as a binder (names `MLL1`, …, `IMLL1`, …; the named constants stay
propositional). `Fragment::ALL` becomes `ADDITIVE`, its name still
`ALL` (F56); `FromStr` reads what `Display` and `name_in` write.

`Reading` keeps its borrowed form and accessors, with two changes. The
goal is read from what was written (C1, H9, H10): with the roots in
written order the goal is the last root, which must read as output, and
every other root must read as input; anything else is refused
(`ShapeError`, whose message says to write the goal last), never reread
with another goal. This settles R54's
`⊤`/`0` ambiguity at its source and is what the two-sided Rocq statement
states; its effect on the lock is the library area's to measure.
`Interactive` and `Derivation` keep the positions they computed and hand
out a `Reading<'_>` in O(1) (F66, R93). Step 34 reads a cut pair's `A`
as output and `A⊥` as input, choosing the goal among the conclusion's
roots only (R99).

### 3.5 The proof term

```rust
/// One rule instance of the dyadic calculus `⊢ Θ ; Γ`. Closed (P5): a new
/// variant is a 0.y bump and a constructor of the Rocq mirror.
pub enum Node {
    Ax(Member, Member),           Tensor(Member, NodeId, NodeId), Par(Member, NodeId),
    One(Member),                  Bot(Member, NodeId),            With(Member, NodeId, NodeId),
    Plus(Member, Side, NodeId),   Top(Member),                    Bang(Member, NodeId),
    Quest(Member, NodeId),        Copy(Member, NodeId),           Weaken(Member, NodeId),
    Mix(NodeId, NodeId),
    // 34: Cut(Member, NodeId, NodeId)       the A root of a cut pair; left Γ, A, right Δ, A⊥
    // 38: Forall(Member, Eigen, NodeId)     Exists(Member, WitnessId, NodeId)
}
const _: () = assert!(size_of::<Node>() == 16);
impl Node {
    pub const TAGS: &'static [&'static str];     // wire tags in variant order (R118)
    pub const fn principal(self) -> Option<Member>;
    pub fn members(self) -> impl Iterator<Item = Member>;    // was occurrences()
    pub fn premises(self) -> impl Iterator<Item = NodeId>;
    pub const fn name(self) -> &'static str;
}
```

A probe compiled `Node` with the three reserved variants (16 bytes) and
`Term` with its four (12 bytes). **The variant policy**: `Node` stays
exhaustive; each addition is a 0.y bump in the changelog, a wire tag, a
`Fault` arm, an oracle arm, a `Rule`, a constructor of the Rocq `node`,
and the test of R118 (`Node::TAGS` against `rocq/theories/Tags.v`)
fails until all exist. Every `match` on `Node` in the crate names its
variants (R248, D-11), `from_proof` and the exporters included.

```rust
/// A proof: nodes over a forest it owns, premises before conclusions, the
/// root last, and the conclusion it proves (the sequent, or a goal).
pub struct Proof { forest: Forest, nodes: Box<[Node]>, goal: Option<Box<[Member]>>, mode: Option<Mode> }
impl Proof {
    pub fn new(forest: Forest, nodes: Vec<Node>, root: NodeId) -> Result<Self, Error>;
    pub fn new_of_goal(forest: Forest, goal: &[Member], nodes: Vec<Node>, root: NodeId) -> Result<Self, Error>;
    #[must_use] pub fn with_mode(self, mode: Mode) -> Self;
    pub fn goal(&self) -> Option<&[Member]>;        // None: a proof of the sequent
    pub fn mode(&self) -> Option<Mode>;             // the mode it is meant for (R7)
    pub fn occurrence(&self, m: Member) -> OccId;
    pub fn formula(&self, m: Member) -> impl Display + '_;
    pub fn has_cuts(&self) -> bool;
    pub fn check(&self, mode: Mode) -> Result<(), CheckError>;
    pub fn check_within(&self, mode: Mode, limits: &Limits, stop: impl Stop) -> Result<(), CheckError>;
    pub fn derivation(&self) -> Result<Derivation<'_>, ViewError>;
    pub fn derivation_with(&self, view: &ViewOptions, limits: &Limits, stop: impl Stop) -> Result<Derivation<'_>, ViewError>;
    pub fn derivation_size(&self, view: &ViewOptions, limits: &Limits, stop: impl Stop) -> Result<Size, CheckError>;
    // unchanged: forest, sequent, nodes, node, ids, root
}
```

- **A proof records its conclusion** (B2, F89, F23). `prove_goal` off the
  conclusion sets `goal`; `check` checks a term against its own
  conclusion, so a goal proof is checkable and is never "a proof of the
  sequent that the checker rejects". `goal().is_none()` is the question
  `linlog check` and the Rocq writer ask (`Error::GoalProof` otherwise).
- **`mode` is a claim, not a check**: the search's mode or a file's;
  `check` takes its mode explicitly, and so does the Rocq writer (R152).
- **Invariants** unchanged, stated on the type: premises precede, root
  last, `new` keeps what the root reaches and renumbers, 1 to 2³² − 1
  nodes (`Refusal::Index`), operands in range (`MemberOutOfBounds`),
  `new` does not check.
- **Cut formulas go through the forest** (D-2): 34 builds the forest with
  its cut pairs first, and `Proof::new` keeps its signature. D-5's other
  option, side tables in `Proof::new` now, passes empty arguments in
  every call for 0.1.0; 38's `Proof::with_instances` is additive (R249).
- **Witnesses out of line** (R125): at 38 the proof owns a witness arena,
  an eigenvariable count and the instance table; `Forall` and `Exists`
  name entries by `u32`; an engine's proof is closed under its final
  substitution; `Answer::of_arena` passes the tables.

### 3.6 The checker, specified

The API: `Proof::check(mode)` and `check_within(mode, &limits, stop)`;
the pass `check::examine` stays crate-private with its observers. New at
28: the stop (polled per node, R23), the limits (memory, work), the
error type, and this specification, which goes into `core-proofs.md`
and the rustdoc of `proofs::check` as the text the Rocq function is
written against (R117).

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
pub enum Fault {
    Forbidden, Shape(ShapeError), Succedents(usize), Kind(Member), NotDual,
    Missing { premise: usize, member: Member }, NotEmpty, Differ, NotUnderQuest(Member),
    Surplus, Conclusion(Dyadic),
    // 34: NotACut(Member), Ordered   36: Order, EmptyAntecedent   38: Instance, Eigenvariable, Frames
}
```

**The rules.** A state is `(Θ, Γ, any)`: `Θ` the least set of
unrestricted members the subproof needs, `Γ` a multiset, `any` whether a
`⊤` above absorbs further linear context. `take(S, x)` removes one `x`
from `Γ`, or succeeds absorbed if `x` is absent and `any`, else
`Missing`; `put(S, x)` adds one; `S ⊎ T = (Θ ∪ Θ', Γ + Γ', any ∨ any')`;
`l`, `r` are the children of the principal (in a framed owner, the
members of the children under its frame).

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
| `Cut(a, p, q)` (34) | S, T | `a` the `A` root of a cut pair | `take(S, a) ⊎ take(T, dual(a))` |
| root | S | `Θ = ∅` and `Γ` = the conclusion, or `Γ` within it if absorbing (`Conclusion`) | |

Intuitionistic mode adds, under the reading (`Shape` without one): (R1)
every derived `Γ` holds at most one output member, exactly one unless
absorbing; (R2) `take` absorbs an absent output only if the premise has
no output; (R3) `Weaken` never weakens an output; Mix is `Forbidden`;
failures are `Succedents`. Step 36 adds (R4) for Lambek: every derived
sequent keeps an input (`EmptyAntecedent`), and the ordered checks
(`Order`); 38 adds `Ax`'s instance comparison, the frames of
`Forall`/`Exists` and the eigenvariable set flowing up (fo-linear §4.2).

**Outside the verified function.** `Surplus` exists because Rust's
counters are finite; over `nat` such a zone fails at the root, so the
verdicts agree (a lemma at 31, or the report says why not). `Refused`
has no counterpart; soundness needs "accepted implies derivable" only.
`oracle.rs` (test-only) is kept, gets every new arm, and
`agrees_with_the_first_implementation` covers each new node kind: it is
the text the Rocq function translates (R117, R120).

**The integers** (S3) rest on the same four facts: fewer than 2³² nodes,
two premises at most per node (`Cut` too, R31), fewer than 2³² − 1
occurrences, zones within `Bag::MOST` after `within`. A framed owner
adds members, not premises; its table is bounded by the nodes and
charged to the same memory bound (R126).

### 3.7 Rules, the derivation view, inferences

```rust
/// A rule of the one-sided standard calculus. Closed (P5).
pub enum Rule { Ax, Tensor, Par, One, Bot, With, PlusLeft, PlusRight, Top,
    Promotion, Dereliction, Contraction, Weakening, Mix, AffineWeakening, Open }
impl Rule {
    pub const ALL: &'static [Rule];
    pub const fn name(self) -> &'static str;
    pub const fn is_structural(self) -> bool;
    pub const fn premises(self) -> u8;            // new: replace the `matches!` sites (impact-boxes 7)
    pub const fn has_principal(self) -> bool;     // new
}
/// A rule as a derivation names it: the one-sided rule and, two-sided,
/// the side of ⊢ its principal stands on (⊸L is ⊗ on the input side).
#[non_exhaustive] pub struct Named { pub rule: Rule, pub side: Option<Position> }
impl Named { pub const fn name(self) -> &'static str; }       // ⊸L, ⊗R, &L₁, !c, …
impl FromStr for Named; impl From<Rule> for Named;
```

F60's split: the 18 two-sided variants are `(rule, side)`, consumers
lose their `unreachable!` arms, 38 adds two `Rule`s, not six, and 36's
divisions are a side plus an orientation, a field of `Named`. Label
tables index `(rule, side)`; `names_round_trip` covers every `Named`.
Every form writes `Named::name`, so none changes.

```rust
#[non_exhaustive]
pub struct Inference {
    pub sequent: Vec<Member>,      // ascending with repeats; the written sequence in an ordered mode (36)
    pub rule: Rule,
    pub principal: Option<usize>,  // a position in `sequent`; None for ax, Mix and Cut
    pub premises: Vec<InfId>,
    pub times: u32,                // a run of a structural rule (compact view)
    // 38: pub datum: Option<Datum>   the witness or the eigenvariable
}
impl<'a> Derivation<'a> {
    pub fn new(proof: &'a Proof, view: &ViewOptions, limits: &Limits, stop: impl Stop) -> Result<Self, ViewError>;
    pub fn named(&self, id: InfId) -> Named;                  // the two-sided name under the reading
    pub fn occurrence(&self, m: Member) -> OccId;
    pub fn formula(&self, m: Member) -> impl Display + '_;
    // unchanged: forest, reading, inferences, inference, root, write_text, text_size, write_steps
}
#[non_exhaustive] pub struct ViewOptions { pub compact: Compact, pub sides: Sides }
pub enum Sides { One, Two }                 // Two: under the intuitionistic reading; Sides::of(mode)
#[non_exhaustive]
pub enum ViewError { Invalid(Box<Invalid>), Refused { refusal: Refusal, size: Option<Size>, firm: Option<Size> } }
```

`ViewError` can no longer hold a refused check under `Invalid`; its
`TooLarge`, `Memory`, `TooMany` and `Stopped` are refusals
(`Output`, `Memory`, `Index`, `Stopped`) with the size, and `firm` is the
compact view's lower bound when it was tried (F65). `Size` is
`#[non_exhaustive]`; a saturated count is `u64::MAX` with `exact: false`
(F70, R247). A derivation is written, never read (R4): `{"version": 1,
"sequent": …, "inferences": [{"sequent": [0, 1], "rule": "ax",
"premises": [], "times": 1}, …]}`, the session's inference objects; a
client sends the proof and the view options to have it rebuilt.

### 3.8 Interactive proving: a partial derivation the checker completes

D13 stands: the state is a derivation of the standard calculus over the
session's forest, open goals as leaves, and a completed state translates
into a term the checker validates (`proof()` checks always, in the
session's mode). What changes is that **nothing enters the state that
the checker has not seen in the session's mode over the session's
forest** (F23): `close_with` refuses a proof over another forest
(`proof.forest() != self.forest()`: `Error::ForeignProof`), a proof
whose recorded conclusion is not the goal (as multisets:
`Error::GoalMismatch`), and checks it in `self.mode()`, not in
`mode.affine()`. F23's two witnesses (a `[Top(4)]` of another forest, a
`Weaken` in a linear session) become tests of these errors.

```rust
pub struct GoalId(u32);                      // a session's own inference ids (F69)
/// A step: the rule on the formula at `position` of a goal, with its split.
#[non_exhaustive]
pub struct Step { pub position: usize, pub rule: Named, pub split: Split }
impl Step {
    pub fn new(position: usize, rule: impl Into<Named>) -> Self;
    #[must_use] pub fn left(self, positions: &[usize]) -> Self;
    // 38: #[must_use] pub fn witness(self, w: Witness) -> Self      a term, or Witness::Open
}
#[non_exhaustive] pub enum Split { None, Left(Vec<usize>) /* 36: At(usize) */ }

impl Interactive {
    pub fn new(sequent: &Sequent, mode: Mode, limits: &Limits) -> Result<Self, Error>;
    pub fn goals(&self) -> impl Iterator<Item = GoalId> + '_;
    pub fn goal(&self, id: GoalId) -> Option<&[Member]>;
    pub fn rules(&self, goal: GoalId, position: usize) -> Result<Vec<Named>, StepError>;
    pub fn apply(&mut self, goal: GoalId, step: &Step) -> Result<Vec<GoalId>, StepError>;
    pub fn split_passes(&self, goal: GoalId, step: &Step) -> Result<bool, StepError>;
    pub fn undo(&mut self) -> Option<GoalId>;
    pub fn close(&mut self, goal: GoalId, options: &Options, view: &ViewOptions, stop: impl Stop) -> Result<Outcome, Error>;
    pub fn close_with(&mut self, goal: GoalId, proof: &Proof, view: &ViewOptions, limits: &Limits, stop: impl Stop) -> Result<(), Error>;
    pub fn close_all(&mut self, options: &Options, view: &ViewOptions, stop: impl Stop) -> Vec<(GoalId, Result<Outcome, Error>)>;
    pub fn derivation_ids(&self) -> Vec<GoalId>;      // a drawing's InfId → the GoalId apply takes
    pub fn occurrence(&self, m: Member) -> OccId;
    pub fn formula(&self, m: Member) -> impl Display + '_;
    // unchanged: forest, sequent, mode, reading (now O(1)), derivation, proof, is_complete, steps
    // removed: inferences() (F69: its premises were session ids typed InfId; goal, derivation, JSON serve)
}
#[non_exhaustive]
pub enum StepError { NoGoal(GoalId), NoFormula { position: usize, len: usize }, Rule { rule: Named, position: usize },
    Mode { rule: Named, mode: Mode }, NotAlone { rule: Named, position: usize }, NoDual { position: usize },
    NotQuest { position: usize }, Split { position: usize }, NoSplit { rule: Named }, Succedents(usize),
    Output { position: usize }, EmptySide }
```

- `close_all` gives every goal its own result (F68, R92); the grafts made
  stay. A per-goal budget is `Limits::work` in the options, or a stop
  counting `Progress::work`.
- A Mix whose split sends every formula to one side is
  `StepError::EmptySide` (F67, R94): with C3's answer A nothing closes an
  empty goal.
- The filtered list a mouse client shows (R91), `applicable(goal,
  position) -> Vec<Named>`, is additive; `split_passes` takes the `Step`,
  so 36's interval split needs no new signature.
- The reader (`from_parts`) adds H17's bound: a history entry lies below
  the arena's length at the time of its step, so `undo` cannot index past
  it; the replay becomes linear (F21).
- **Cut (34)** is a method, not a `Step`: `cut(goal, formula, split) ->
  Result<[GoalId; 2], StepError>` appends a cut pair to the forest (every
  id stays) and opens `Γ, A` and `Δ, A⊥`; `undo` keeps the trees, which
  `is_whole_forest` makes harmless.
- **Witnesses (38)** ride on `Step::witness` (F22): a term, or
  `Witness::Open` that a later axiom resolves; `undo` undoes bindings;
  `StepError` gains `Witness`, `Eigenvariable`, `Unresolved`.
- **JSON** keeps its keys plus `"version"`: `{"version": 1, "sequent":
  …, "mode": …, "inferences": [{"sequent": [0, 1, 4], "rule": "⊸L",
  "principal": 1, "premises": [1, 2]}, {"sequent": [3, 4]}], "history":
  [0]}`; 34 adds `"cuts"` (version 2), 38 `"instances"` and a per-inference
  `"witness"` (version 3). `Step` has serde (`{"position": 1, "rule": "⊗",
  "left": [0]}`) for the web client.

### 3.9 Proof structures: vertices, criteria, boxes

```rust
/// A vertex: in MLL the occurrence itself (vertex i is occurrence i, no
/// table); in MELL (33) an instance, a collector, a door or a box node.
pub struct VertexId(u32);
/// The rules a structure is checked under: the one value 33, 35 and 36
/// extend in place of `mix: bool` (R79).
#[non_exhaustive]
pub struct Criterion { pub mix: bool /* 35: essential; 36: order */ }
impl Criterion {
    pub const MLL: Self;
    #[must_use] pub const fn with_mix(self) -> Self;
    pub fn of(mode: Mode) -> Result<Self, Error>;                         // NetMode for affine
    pub fn admits(self, fragment: Fragment, mode: Mode) -> Result<(), Error>; // R133, F28: the one predicate
}
impl ProofStructure {
    pub fn new(forest: Forest, criterion: &Criterion) -> Result<Self, Error>;
    pub fn from_links(forest: Forest, criterion: &Criterion, links: &[(VertexId, VertexId)]) -> Result<Self, Error>;
    pub fn from_proof(proof: &Proof, criterion: &Criterion, limits: &Limits, stop: impl Stop) -> Result<Self, Error>;
    pub fn criterion(&self) -> Criterion;
    pub fn vertex(&self, o: OccId) -> Option<VertexId>;     // None when o has several instances
    pub fn occurrence(&self, v: VertexId) -> OccId;
    pub fn is_correct(&self, limits: &Limits, stop: impl Stop) -> Result<(), NetError>;
    pub fn sequentialize(&self, limits: &Limits, stop: impl Stop) -> Result<Proof, Error>;
    // over VertexId now: partner, links, unlinked, link, unlink, same_component
    // unchanged: forest, sequent, is_complete, scratch, is_acyclic
    // 33: boxes, box_of, depth, jumps, kind;  34: cuts, cut(x, y), the erased set
}
#[non_exhaustive]
pub enum NetError { NoVertex { vertex: u32, vertices: u32 }, NotLiteral(VertexId), NotDual(VertexId, VertexId),
    LinkedTwice(VertexId), Unlinked(VertexId), Empty, SwitchingCycle(Vec<VertexId>),
    Disconnected(Vec<Vec<VertexId>>), Refused(Refusal) }
    // 33: LinkAcrossBoxes, JumpAcrossBoxes, JumpToNeighbour, MissingJump, DoorWithoutPremise, and the
    //     box a cycle lies in; 34: CutNotDual; 35: DirectedCycle, NoDominator; 36: Crossing
```

- **The MLL hot path keeps its cost**: `VertexId(i) == OccId(i)`, no
  table; `partner`, the CSR graph and the skeleton index as today, and
  the net engine keeps `link_unchecked`, `same_component`, `is_acyclic`.
  Gate: a counter-exact target list for the net engine (`links`, `tests`
  on one thread, impact-boxes item 9), taken before the retype.
- **`from_proof` is exhaustive and bounded** (F49, R254, R25): `Ax` a
  link, `Tensor`/`Par`/`Mix` nothing, every other arm
  `Error::NetFragment` naming the rule, until 33 (exponentials, `Weaken`)
  and 34 (`Cut`) replace them. Desequentializing a MELL term unfolds
  shared subproofs per path, so its vertices are charged to
  `memory_bytes` and the stop polled per node from the first signature.
- **One criterion value**; boxes are a property of the structure, not of
  the criterion (R145). Its fields serialize flattened into the net, so
  `"mix": false` stays where it is.
- **MELL nets are equal up to isomorphism modulo jumps** (mell-nets-spec
  §9), so `ProofStructure` has no `PartialEq`.
- **JSON** (R8): `{"version": 1, "sequent": …, "mix": false, "links":
  [[0, 2], [3, 4]]}`; 33 writes `"vertices": [[occ, parent, box], …]`,
  `"boxes"`, `"jumps"` and links over vertex ids at version 2 when there
  is an exponential; 34 `"cuts"` and the erased set at version 3.

### 3.10 Refutations and disproofs

A refutation means something only with its sequent, goal and mode, and
some refutations are certificates a checker sharing no code with the
engines re-verifies (R124; refutations.md groups A and E). So an
`Unprovable` verdict carries a `Disproof` that holds them, as a `Proof`
holds its forest; it holds the `Sequent` (an arena linear in the input
text), not the forest, which `check` rebuilds within its limits.

```rust
#[non_exhaustive]
pub enum Refutation {
    Exhausted,                              // no certificate
    Unbalanced(Unbalanced),
    Equation(Equation),
    StateEquation(StateEquation),
    // 31 (C2): Classical(Assignment); later: Saturated (37), countermodels (R72)
}
#[non_exhaustive] pub struct Unbalanced { pub atom: Atom, pub least: i32, pub most: i32 }   // name from the sequent (F80)
#[non_exhaustive] pub struct Equation { pub formulas: u64, pub tensors: u64, pub pars: u64, pub ones: u64, pub bottoms: u64, pub mix: bool }
/// Farkas weights of a Horn program's Petri net, every place included (R70).
#[non_exhaustive] pub struct StateEquation {
    pub atoms: Vec<(Atom, i64)>,      // each atom's place
    pub clauses: Vec<(OccId, i64)>,   // each clause used once, a place of its own
    pub dropped: Vec<OccId>,          // dead clauses: an input outside the closed set
}
pub struct Disproof { sequent: Sequent, goal: Option<Box<[Member]>>, mode: Mode, refutation: Refutation }
impl Disproof {
    pub fn new(sequent: Sequent, mode: Mode, refutation: Refutation) -> Self;
    pub fn check(&self, limits: &Limits, stop: impl Stop) -> Result<(), RefutationError>;
    // sequent, goal, mode, refutation; Display writes it in words, atoms by name
}
#[non_exhaustive]
pub enum RefutationError { Invalid { reason: &'static str }, Uncertified, Refused(Refusal) }
```

- **The conditions travel with the check, not the value** (R70, R71).
  `Disproof::check` recomputes from the sequent and mode whether the kind
  applies (intervals: no weakening, no `⊤` under an exponential, no row
  for an atom under one; the equation: no additives, additive units,
  exponentials or weakening; the state equation: a Horn program,
  nonnegative weights in affine mode), then the certificate itself, by
  functions written again in `refutation.rs` (literals ±1, `⊗`/`⅋` sum,
  `&`/`⊕` hull), never by calling `focus::Counts` or `horn::equation`:
  the language-neutral statement 31's lemmas mirror. Outside its
  conditions a refutation is `Invalid`; `Exhausted` is `Uncertified`.
- **`StateEquation`'s payload changes before 0.1.0** so the checker
  verifies the closure, the dropped transitions and `y·C ≤ 0`,
  `y·(M − M₀) > 0` itself; `once` goes.
- **Checked where it can be**: `prove_goal` checks every refutation it
  returns in debug builds and tests; a refuter turns `Unknown` into
  `Unprovable` only with one `check` accepts, in every build (C2).
- **No refutation rests on a bound** (R71): `Unprovable` comes only from
  a level that met none. A bound proved sufficient, a loop check (R114)
  and a failure trace (R112) are additive kinds and options.
- `prove_goal` builds the `Disproof` (one clone of the sequent arena per
  unprovable outcome, B3); `Outcome` stays write-only (R3).

### 3.11 The ordinary layer

`ordinary::Sequent::new` and `Formulas::add` become fallible (F13, R249)
and refuse an operand that is not an earlier node or an atom outside the
names (at 38 also a free variable, a symbol at two arities).
`Image::read_back(&self, proof: &Proof, view, limits, stop)` reads back
through the linear derivation built within `output_bytes`
(`Compact::Never`), so step 25's 6 GiB unfoldings are refused before
they are made; `ordinary::Derivation::check(&limits, stop)` polls per
inference (F12, R24). Deciding is one call (R113):
`ordinary::decide(&sequent, &ordinary::Options, &search::Options, stop)
-> Result<ordinary::Outcome, Error>` translates, proves the image and
keeps both for the read-back; a second procedure (G4ip with
countermodels, R72) is a variant of `ordinary::Procedure` in its
options. JSON (R11, F53), versioned from birth: `ordinary::Sequent` is
`{"version": 1, "formulas": {"nodes": […], "atoms": […]}, "left": […],
"right": […]}`, read through `Within`; `ordinary::Derivation` is written
only, `{"version": 1, "logic": …, "formulas": …, "inferences": [{"left",
"right", "rule", "principal", "premises"}]}`; `Image` is rebuilt by
`translate`. The ordinary terms share the linear arena's first-order term
type (D-4), so the translation copies term ids and the LK/LJ checker
keeps "equal formula iff equal id"; 38 adds its tags and a
per-inference `"witness"` at version 2.

## 4. Errors

### 4.1 One family

`linlog::Error` is the family: every public fallible call returns it or a
specific error type that converts into it without loss, and every one
answers `kind()` and `code()` from one table. Specific types stay where a
caller matches their data (the checker's premises, a net's cycle, a parse
span, a refused step), as `std` does; the front page names them and says
each converts (F8).

```rust
#[non_exhaustive]
pub enum ErrorKind {
    Malformed,     // the input is not what it says: no sequent, indices that do not fit, a newer version
    Invalid,       // a verdict: the proof, net, step or certificate is not what it claims
    Unsupported,   // no verdict: outside what linlog, the engine or the kernel does
    Refused,       // no verdict: a bound (set or default) or the caller's stop
    Defect,        // linlog's own: an engine's proof the checker rejects, a read-back that is no LK/LJ
}
impl ErrorKind { pub const fn is_refusal(self) -> bool; /* Unsupported | Refused */ pub const fn name(self) -> &'static str; }

#[non_exhaustive]
pub enum Error {
    // Malformed
    Parse(Vec<ParseError>), Read { format: InputFormat, reason: String },   // LLTP, .spec, TPTP (HD1)
    AtomOutOfBounds { atom: u32, atoms: u32 }, TermOutOfBounds { term: u32, terms: u32 },
    SubtermNotBefore { term: u32, subterm: u32 }, NodeOutOfBounds { node: u32, nodes: u32 },
    PremiseNotBefore { node: u32, premise: u32 }, MemberOutOfBounds { member: u32, members: u32 },
    AtomName { name: String } /* HD5 */, State { reason: &'static str },
    Version { form: &'static str, found: u32, supported: u32 }, Setting { key: String, reason: String },
    // Invalid, or the inner kind
    Check(CheckError), Net(NetError), Step(StepError), Refutation(RefutationError), ForeignProof, GoalMismatch,
    // Unsupported
    NotIntuitionistic(ShapeError), IntuitionisticMix, GoalOutputs { outputs: usize }, NetFragment(Fragment),
    NetMode(Mode), NetGoal, EngineMode { engine: Engine, mode: Mode }, NotAdditive { fragment: Fragment, roots: usize },
    NotHorn, NoEngine { fragment: Fragment, mode: Mode }, FragmentMismatch { asserted: Fragment, detected: Fragment },
    Translation { translation: Translation, logic: Logic }, Succedents { count: usize }, OpenGoals { open: usize }, GoalProof,
    // Refused, or the inner kind
    Refused(Refusal), View(ViewError), Write(WriteError), Render(RenderError), ThreadPool { threads: usize, reason: String },
    // Defect
    Rejected(Box<check::Invalid>), ReadBack { calculus: &'static str, reason: String },
}
impl Error {
    pub fn kind(&self) -> ErrorKind;          // a wrapped error's own kind
    pub fn code(&self) -> &'static str;       // the stable wire reason
    pub fn describe<'a>(&'a self, forest: &'a Forest) -> Described<'a, Error>;
}
```

- **A refusal never reads as invalid** (S4, S18): `kind()` is one `match`
  without a wildcard; every error type that can be refused keeps a
  variant of its own for it. `Error::Rejected` is a `Defect`, never a
  verdict on the sequent; a found proof whose check is refused for memory
  is no error but `Unknown(Reason::Unchecked)` (R138, F139), so the
  command exits 3; `Error::Check` remains for `Interactive::proof` and
  `close_with`, which are no search.
- **Named fields** (F31). **Every type** is `Clone`, `Debug`, `Display`,
  `std::error::Error`, `Send`, `Sync`; `ParseError` gains `Clone`,
  `PartialEq`, `Error` and `expected` (F34, F29), its span in bytes with
  `utf16_span(&self, text)` for a JavaScript editor (R129).
- **No public call panics on input of the right type** (R130, S5); the
  kept `expect` sites are invariants of the crate's own values, listed in
  `core.md`. A client or the harness classifies refusals by `kind()`,
  never by a list of variants (R135).
- **One `describe`** (F48): `Described<'a, E>` borrows the error and the
  forest, prints formulas for ids and has `abbreviated(limit)`; each
  error writes itself through one sealed writer `write(f,
  Option<&Forest>, limit)`, so `Display` and `describe` cannot drift.

### 4.2 The wire form of an error

Errors are written, never read: `impl Serialize for Error` and
`Error::form(&self, Option<&Forest>) -> impl Serialize + '_` (messages
with formulas), from the one `match`:

```json
{"code": "invalid_proof", "kind": "invalid",
 "message": "node 2 (⊗ on 1 from 0, 1) with premises ⊢ 0, 3 and ⊢ 3, 4: premise 0 lacks occurrence 2",
 "details": {"node": 2, "rule": {"⊗": [1, 0, 1]}, "fault": {"kind": "missing", "premise": 0, "member": 2},
             "premises": [{"theta": [], "gamma": [0, 3], "any": false}, {"theta": [], "gamma": [3, 4], "any": false}]}}
```

`code` is what a client branches on (AIP-193), `message` the English
text, `details` named fields with units in their names (`limit_bytes`).
The codes (`parse`, `atom_out_of_bounds`, …, `invalid_proof`,
`switching_cycle`, `stopped`, `memory_limit`, `too_many_occurrences`,
`output_too_large`, `work_limit`, `index_limit`, …) are a table in the
rustdoc of `Error`, stable from 0.1.0; a client accepts unknown codes and
falls back on `kind` (AIP-126).

### 4.3 How each error type maps in

| type | after | kind |
|---|---|---|
| `ParseError` | `Error::Parse`; `#[non_exhaustive]`, `expected` | Malformed |
| `CheckError`, `Problem` | `Invalid(Box<Invalid>)` / `Refused(Refused)`; `Fault` without `Memory` | Invalid / Refused |
| `ViewError` | `Invalid(Box<Invalid>)` / `Refused { refusal, size, firm }` | Invalid / Refused |
| `NetError` | `VertexId` witnesses, `Refused(Refusal)` | Invalid; Malformed for `NoVertex`, `NotLiteral`, `LinkedTwice`; Refused |
| `ShapeError` | `#[non_exhaustive]`, in `Error::NotIntuitionistic` | Unsupported |
| `interactive::Refusal` | `StepError` | Invalid |
| `WriteError` | `{ Failed, Unsupported(rocq::Unsupported), Refused(Refusal) }` (`Stopped` is `Refusal::Stopped`) | Defect / Unsupported / Refused |
| `rocq::Unsupported` | `#[non_exhaustive] { Open, Rule { kernel, rule }, Mode { kernel, mode }, Refutation { kernel } }` | Unsupported |
| `svg::TooLarge` | `Refusal::Output { estimate, limit }` | Refused |
| `RenderError` | `{ Svg, TooLarge { pixels, limit }, Refused(Refusal), NoDate, Failed }` | Malformed / Refused / Unsupported / Defect |
| `UnknownRule` | the `FromStr` error of `Named` (the name, non-exhaustive) | Malformed |
| new `RefutationError` | `Invalid` / `Uncertified` / `Refused` | Invalid / Unsupported / Refused |

## 5. Bounds and stops

### 5.1 One bounds value

```rust
/// What a call may take. Every long call and every reader takes one.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]   // serde(default, deny_unknown_fields)
pub struct Limits {
    pub memory_bytes: Option<u64>,   // held beyond the input: memo, arena, checker states, records, tables, renders
    pub occurrences: Option<u64>,    // a forest built, a text or file read
    pub output_bytes: Option<u64>,   // a derivation, drawing or certificate, estimated before it is made
    pub work: Option<u64>,           // Progress::work summed: a deterministic, clock-free budget
    // 38: pub term_size: Option<u64>   the unfolded size of first-order terms (R250)
}
impl Limits {
    pub const DEFAULT_MEMORY_BYTES: u64 = 1 << 30;
    pub const DEFAULT_OCCURRENCES: u64 = 50_000_000;
    pub const DEFAULT_OUTPUT_BYTES: u64 = 64 << 20;
    pub const NONE: Self;
    pub fn browser() -> Self;                      // a tab's preset, measured at step 32 (R142)
    #[must_use] pub const fn with_memory_bytes(self, bytes: Option<u64>) -> Self;   // one per field
}
#[non_exhaustive]
pub enum Refusal { Stopped, Memory { limit: u64 }, Occurrences { occurrences: u64, limit: u64 },
    Output { estimate: u64, limit: u64 }, Work { limit: u64 }, Index { what: Indexed, count: u64, most: u64 } }
```

`None` means no bound in every field (F79); the representation's caps
remain (`Forest::MOST`, 2³² − 1 nodes) as `Refusal::Index`. One value
replaces `Options::memory_limit` and `occurrence_limit`,
`ViewOptions::{limit, memory}`, `check_within`'s `memory` and
`png`/`pdf::Options::memory` (F62, F63). `search::Options` holds one, so
`prove` keeps its signature; every other long call takes `&Limits`. The
field's rustdoc says what `memory_bytes` does not count: the input, the
stacks, the allocator, the net engine's structure, linear in the forest
(R29).

### 5.2 One stop, with progress

```rust
#[non_exhaustive]
pub struct Progress {
    pub work: u64,    // units since the last poll, the unit of Limits::work
    pub nodes: u64,   // so far: stable sequents, literals chosen, markings, nodes checked
    pub bytes: u64,   // held now, as the account counts
}
pub trait Stop { fn poll(&mut self, progress: Progress) -> bool; }
impl<F: FnMut() -> bool> Stop for F { … }                    // today's closures keep working (B9)
pub struct WithProgress<F>(pub F);
impl<F: FnMut(Progress) -> bool> Stop for WithProgress<F> { … }
impl Stop for &mut dyn Stop { … }
```

A probe compiled these three impls together without a coherence
conflict, and a function taking `impl Stop` accepted a closure, `&mut` a
closure, a `WithProgress` and a `&mut dyn Stop`; the crate-private
`search::Stop` enum of today is renamed (`Poll`). The unit of work is
each poll site's natural step (a stable sequent, a literal chosen, a
marking, a checked node, an inference written), documented per engine on
`Engine` (R30); past `Limits::work` a search is `Unknown(Reason::
WorkLimit)`, any other call `Refusal::Work`. The slices that alternate
the default bias's two searches (`Stop::Slice`, `Turn`, crate-private)
stay apart from `Progress::work`, so a decided run's counters do not move
(R243). The front page's example that counts polls (F136) becomes a
`Limits::work` example.

### 5.3 Readers within limits

`Deserialize` takes no options, so `forms::Within<'a, T>` implements
`DeserializeSeed` for `Sequent`, `Proof`, `Disproof`, `ProofStructure`,
`Interactive`, `ordinary::Sequent` and the options values, building
forests within `occurrences` and tables within `memory_bytes`, and
checking atom names (R27, F27, H19, HD5); the plain `Deserialize` is
`Within` at `Limits::default()`. Text readers take `&Limits` as well:
`Sequent::parse_within` counts occurrences while it reads (F16), and
`lltp::read`, `mist::read`, `ordinary::read_tptp`. All are linear (a
session's replay after F21), so none needs a stop.

### 5.4 Every long public call

| call | stop, polled per | bounds |
|---|---|---|
| `prove`, `prove_until`, `prove_goal`, `search::race` | stable sequent, split batch, literal, marking, set-up pass | `Limits`; copies, recursion, memo |
| `Interactive::close`, `close_all`, `close_with` | the search's, then node checked and inference grafted | `Limits` |
| `Proof::check_within` | node | memory, work |
| `Proof::derivation_with`, `derivation_size`, `Derivation::new` | node, inference | memory, output |
| `Disproof::check` | pass | memory, work, occurrences |
| `ProofStructure::from_proof`, `is_correct`, `sequentialize` | node, deletion round, stage | memory |
| every export `write`, `Derivation::write_text` | inference, node, link | output |
| `png`/`pdf::from_svg` | none: refused before usvg parses | memory (the estimate) |
| `Image::read_back`, `ordinary::Derivation::check`, `ordinary::decide` | inference; the search's | output, memory |
| `batch::run`, `batch::prove` | per problem, plus a cancel handle on `Results` (R40) | per problem, the batch's share |
| elimination (34), saturation (37), refuters (C2) | step, given clause, assignment | work, memory, output |
| `Forest::new`, `parse_within`, the readers, `translate`, `Interactive::new`, `Reading::new` | none: linear | occurrences, text length |

No other public call is above linear in its input; the crate docs carry
this table (R22).

### 5.5 One account, and the pool's stack

The race of one thread and the pool (R85) moves into the library,
`search::race(goal, mode, &options, stop, widen: &AtomicBool)` behind
`parallel` (the caller raises `widen` when its own timer fires: no
clock), and both searches draw on one `memory::Account`, so a call holds
`memory_bytes` once (F104, R18). The pool's stack (H18): the recursion
limit bounds the frames on a thread, not the depth in the proof. The
counter is per thread, so a job a waiting thread steals counts on top of
what the thread holds, `Options::stack_size()` holds under stealing, and
a stolen job past it answers `RecursionLimit` for its branch: only
`Unknown` can result. Every pool task polls before it starts (R19).
`Options::recursion_limit_for(stack_bytes)` inverts `stack_size()` for a
given stack (wasm's 1 MiB, R45); counts that can pass 2³² are `u64` in
every public type, so wasm32 changes no answer (R46).

## 6. Options

Plain data: `#[non_exhaustive]`, `Default`, `Clone`, `PartialEq`, serde
with `default, deny_unknown_fields`; callers write `default()` plus
assignments or `#[must_use]` `with_*` builders (F64); every default a
named constant with a flag and a doc line (D16).

**`search::Options`** (R1, F77), private fields with a builder and a
getter each; the JSON keys are the fields:

| key | default | flag |
|---|---|---|
| `engine` | `null` (the dispatch), else a name | `--engine` |
| `fragment` | `null` (detected), else a name | `--fragment` |
| `bias` | `"auto"` (`"rarer"`, `"factors"`) | `--bias` |
| `copies` | 3 (`null`: no bound) | `--copies` |
| `forward_copies` | 30 | `--forward-copies` |
| `memo_limit` | 1 048 576 | `--memo-limit` |
| `recursion_limit` | 2048 | `--recursion-limit` |
| `test_period` | `null` (every link to 200 occurrences, every 4th above, as `DEFAULT_TEST_*`) | `--test-period` (new, F81) |
| `jobs` | 1 | `--jobs` |
| `check` | `true` | `--no-check` |
| `limits` | `Limits::default()` | `--memory-limit`, `--occurrence-limit`, `--derivation-limit`, `--work-limit` (new) |
| `refute_unknown` (31, C2) | `false` | `--refute-unknown` |
| `pool` | not written: a runtime handle | `--jobs`, `--pool-after` |

`Engine`, `Bias`, `Fragment` and `Mode` read their names back (F76,
R241), so no front end keeps a name table. `Options::DEFAULT_TIME_LIMIT:
Duration = 2 s` is data each front end applies through its stop (R148).
Steps 37 and 38 add their knobs as fields (R38). The command builds one
`search::Options` and one `Styles` from its flags, or reads them from
`--options FILE` and `--style-file` (R183); a batch record may carry a
problem's own options; the web client holds both as JSON. `Limits`'
form: `{"memory_bytes": 1073741824, "occurrences": 50000000,
"output_bytes": 67108864, "work": null}`.

**`batch::Options`** (serde): `search`, `workers`, `cores`, the stream
flag; the batch divides `limits.memory_bytes` among workers as
`core-batch.md` says; `batch::Problem` gains a problem's own options
(R73); `plan` takes the resolved `Cores` (F10).

**Export options** stay one per target, `#[non_exhaustive]`; strings that
become code are validated types: `rocq`'s `lemma: Identifier` (F38),
Typst's spacing fields a `Length` instead of verbatim code.
**`export::Styles`** (R141) moves into the library: `{ text, latex,
typst, svg, png, pdf, rocq }` with `set(key, value)` for `--style
KEY=VALUE` and `Styles::KEYS`, from which the help text is generated
(F196); the web client holds one `Styles` JSON. **`ordinary::Options`**
keeps the translation's choices; TPTP's go to `read_tptp(text,
&TptpOptions, &limits)` (D-12). **Presets** are values
(`Limits::browser()`, `svg::Style::{dark, monospace}()`), each citing
its measurement (D15, R142).

## 7. Wire forms

### 7.1 The policy

1. **A version on every form read back** (`Sequent`, `Proof`, `Disproof`,
   `ProofStructure`, `Interactive`, `ordinary::Sequent`) and on the
   written `Outcome`, `Derivation`, `ordinary::Derivation`: `"version"`,
   the first key, an integer; missing reads as 1; a nested form carries
   none and is read under its container's.
2. **The lowest version that represents the value**: version `n` only
   when a reader of `n − 1` would misread or drop something. Cut-free
   proofs, MLL nets and propositional sequents stay version 1 and
   byte-identical forever. A newer version is refused with
   `Error::Version { form, found, supported }`.
3. **Unknown keys**: a data form ignores them (the version gates meaning,
   and an outcome read as a proof must pass its other keys); options
   values refuse them, naming the key; so does `Mode`, whose meaning a
   key could change. No first-order, net, cut or ordered meaning rides on
   an old tag through a new key: it comes with a new tag or version.
4. **Unknown values**: a data form's tags are refused by name; the open
   enumerations a client receives (`reason.kind`, `refutation.kind`,
   `engine`, an error's `code`) are documented as open (AIP-126).
5. **Names** (T1, answer A, in the versions' commit): `ids` → `roots`,
   `var_dict` → `atoms`, `proof` → `nodes`, `memory_limit` →
   `memory_bytes`, `once` gone; the tags `V` and `D` stay, documented as
   an atom and its dual. Keys are snake_case nouns, units in the name.
6. **Numbers** (R247): JSON numbers, exact below 2⁵³. Only `Size`
   reaches beyond, saturating at `u64::MAX` with `exact: false`; every
   other count is bounded far below by the occurrence and node limits,
   and each form's rustdoc says so.

### 7.2 The forms

**Sequent** (v1). `"A |- A"`:
`{"version": 1, "terms": [{"D": 0}, {"V": 0}], "roots": [0, 1], "atoms": ["A"]}`.
A unit is its symbol, a connective `{"⊗": [l, r]}` or `{"!": i}`, an atom
`{"V": a}` and its dual `{"D": a}`, each naming earlier terms; `roots`
in written order (C1); atom names identifiers of the syntax (HD5). 38
adds tags `P`/`N`, `∀`, `∃` and keys `fo_terms`, `symbols`, `arities`,
`binders` at v2.

**Proof** (v1):

```json
{"version": 1, "sequent": {"terms": [{"D": 0}, {"V": 0}], "roots": [0, 1], "atoms": ["A"]},
 "nodes": [{"ax": [0, 1]}], "mode": {"intuitionistic": false, "affine": false, "mix": false}}
```

Tags `ax ⊗ ⅋ 1 ⊥ & ⊕₁ ⊕₂ ⊤ ! ? copy wk mix`, members then premise
indices, premises first, the root last; optional `"goal"` (F89) and
`"mode"` (R7). 34: `"cuts"` (a value in `Sequent`'s form whose roots are
the cut formulas, atoms merged by name, appended in order) and `{"cut":
[a, l, r]}`, v2. 38: `{"∀": [m, e, p]}`, `{"∃": [m, w, p]}`,
`"instances"`, `"frames"`, `"witnesses"`, v3.

**Outcome** (written, v1): `verdict`, `fragment` (its name in the mode),
`mode`, `engine`, `statistics`, and per verdict the proof's keys, the
disproof's keys (`sequent`, `goal`, `refutation`) or `reason`. A proved
outcome reads back as a `Proof`, an unprovable one as a `Disproof`, and
`linlog check` checks either (F208, R3):

```json
{"version": 1, "verdict": "unprovable", "fragment": "MLL", "mode": {…}, "engine": "net",
 "statistics": {"nodes": 0, "memo_hits": 0, "memo_entries": 0, "splits": 0, "links": 0, "tests": 0, "copies": 0, "work": 0},
 "sequent": {…}, "refutation": {"kind": "equation", "formulas": 3, "tensors": 0, "pars": 1, "ones": 0, "bottoms": 0, "mix": false, "needed": 1}}
```

**Refutation** and **Reason** are internally tagged, so every kind can
gain fields (F86): `{"kind": "unbalanced", "atom": 0, "least": 1,
"most": 1}` (the atom by index, F80), `{"kind": "state_equation",
"atoms": [[0, 2]], "clauses": [[7, 1]], "dropped": [12]}`, `{"kind":
"copy_bound", "bound": 3}`, `{"kind": "memory_limit", "limit_bytes":
1073741824}`. Their serde is derived on the types, with no proxy ending
in a wildcard (F87, R3). **Disproof** (v1): `version`, `sequent`, `goal`,
`mode`, `refutation`. **Statistics**: `nodes, memo_hits, memo_entries,
splits, links, tests, copies, work`, derived on the type (F88), the
meaning per engine a table on it (T5). **ProofStructure** (v1) and
**Interactive** (v1): 3.9, 3.8. **Mode**: an object, unknown keys
refused, 36's `"order"` written only when not commutative.
**Fragment**, **Engine**, **Bias**, **Named**: names. **Options
values**: no version, defaults for missing keys, unknown keys refused,
`null` for no bound. **Error**: 4.2. **Derivation**: 3.7.
**Ordinary**: 3.11.

| read back | written only |
|---|---|
| Sequent, Proof, Disproof, ProofStructure, Interactive, ordinary::Sequent (`Deserialize` or `forms::Within`); the options values, `Limits`, `Styles`; nested Mode, Fragment, Engine, Bias, Named, Refutation, Reason | Outcome (reads as a Proof or a Disproof), Statistics, Size, Derivation, ordinary::Derivation, Error |

**What changes in the pinned forms**: three commits of the library area,
each its own change of the behaviour lock: (1) versions, T1's renames,
the outcome's `sequent` for unprovable verdicts; (2) the written root
order (C1); (3) the tagged `refutation` and `reason` and
`StateEquation`'s payload. Nothing else in this design moves a pinned
form.

## 8. Engines and the search's front door

```rust
pub fn prove(sequent: &Sequent, mode: Mode, options: &Options) -> Result<Outcome, Error>;
pub fn prove_until(sequent: &Sequent, mode: Mode, options: &Options, stop: impl Stop) -> Result<Outcome, Error>;
pub fn prove_goal(goal: Goal<'_>, mode: Mode, options: &Options, stop: impl Stop) -> Result<Outcome, Error>;
pub fn engine_for(goal: Goal<'_>, mode: Mode, options: &Options) -> Result<Engine, Error>;
/// Members of a forest: its conclusion, or a goal a session left open.
pub struct Goal<'a> { /* private */ }
impl<'a> Goal<'a> {
    pub fn conclusion(forest: &'a Forest) -> Self;
    pub fn new(forest: &'a Forest, members: &'a [Member]) -> Result<Self, Error>;
    pub fn is_conclusion(&self) -> bool;
    pub fn is_whole_forest(&self) -> bool;     // every tree, cut trees included: what the net engine takes
    // 38: framed(forest, &instances, members)
}
#[non_exhaustive] pub struct Outcome { pub verdict: Verdict, pub fragment: Fragment, pub mode: Mode,
    pub engine: Engine, pub statistics: Statistics, pub net: Option<ProofStructure> }
pub enum Verdict { Proved(Box<Proof>), Unprovable(Box<Disproof>), Unknown(Reason) }      // closed (D9)
#[non_exhaustive] pub enum Reason { Stopped, RecursionLimit { limit: u32 }, CopyBound { bound: u32 },
    MemoryLimit { bytes: u64 }, WorkLimit { work: u64 }, IndexLimit, Unchecked { bytes: u64 } }
```

- **`prove_goal` stays the one place an answer becomes a verdict**; every
  proof it returns has passed the checker against its own conclusion
  (goals off the roots included now that a proof records its goal); a
  refused check is `Unknown(Reason::Unchecked)`, a rejection
  `Error::Rejected`.
- **The engine interface stays crate-private** (`Decide`, `Task`,
  `Answer`, `of_arena`), as step 26 shaped it; `Task` carries members and
  `whole_forest`, at 38 the instance table. A first-order goal enters
  through the same `Task` (R109); engines without a first-order form
  refuse the bit with a typed error.
- **Registering an engine is one list** (R86) in `core-search.md`: the
  variant (with `name`, `FromStr`, `Engine::ALL`), the `Decide`
  implementation and its `implementation` arm, its arm in
  `Engine::parallel` (an exhaustive `match` now, F141), the options it
  reads in its docs, a `DISPATCH` row where it is the default (its
  measurement in the `Engine` rustdoc table, R87), its configuration in
  `reference.rs`. The command and the harness parse names with `FromStr`.
- **The dispatch is data** (`DISPATCH`: largest fragment, `Modes`,
  `Feature`, engine, first match). Its thresholds (`NET_MULTIPLICITY`)
  belong to the measured rows, not to options; `Options::engine`
  overrides a row (R149). 36 adds `Modes::Ordered` and keeps
  `Modes::Any` off the ordered modes (R90); 35, 37, 38 add rows.
- **Refuters, the second plug-in kind** (C2, answer C): a crate-private
  `trait Refute` and a list that `prove_goal` runs after an exhaustive
  search whose refutation is `Exhausted`, and, with `refute_unknown` (off
  by default), after an `Unknown`, under the same stop, account and
  `Limits::work`. A refuter never changes a verdict the search gave, and
  turns `Unknown` into `Unprovable` only with a refutation
  `Disproof::check` accepts. The first is 31's classical assignment (a
  bounded DPLL over the erased formula, fixed order); G4ip countermodels
  plug in the same way (R72, R113).
- **`Statistics`** keeps the shared counters (T5), gains `work`, and
  `copies` reports the level of the search that decided (F144).

## 9. Exports

### 9.1 One entry per target

```rust
#[non_exhaustive]
pub enum Item<'a> {
    Sequent(&'a Sequent), TwoSided(&'a Reading<'a>), Derivation(&'a Derivation<'a>),
    Ordinary(&'a ordinary::Derivation), Net(&'a ProofStructure),          // svg
    Proof { proof: &'a Proof, mode: Mode }, Disproof(&'a Disproof),         // rocq
}
// in latex, typst, svg, rocq:
pub fn write(item: Item<'_>, options: &Options, limits: &Limits, out: &mut impl fmt::Write, stop: impl Stop)
    -> Result<(), WriteError>;
```

One function per target replaces the `String` functions and their
`write` twins (F36). A target refuses an item it has no form for, and
every writer estimates its output and refuses past `output_bytes` before
it writes, which bounds the sequent drawings of F5 and the net drawing
(R26); the stop is asked per inference, node or link. `png`/`pdf::
from_svg` take `&Limits` too. Emitters stay generic over the
crate-private `Drawn` and one loop over `Visit`. A new item (33's boxes,
34's elimination, 35's essential net) is a variant of `Item`.

### 9.2 The Rocq export and the library of step 31

```rust
#[non_exhaustive]                     // serde(default, deny_unknown_fields)
pub struct Options { pub form: Form, pub lemma: Identifier, pub prelude: Option<String>, pub kernel: Kernel }
#[non_exhaustive]
pub enum Kernel { #[default] Auto, NanoYalla /* 31: Linlog */ }
pub const NANOYALLA: &str = "1.1.3";
pub const LIBRARY: &str = "0.1.0";   // the rocq-linlog release the certificates are written for (R12)
pub const FORMAT: u32 = 1;           // the certificate format: node set, member encoding, numbering
```

`lemma` is checked to be a Rocq identifier that no kernel name takes
(F38); `prelude: None` is the chosen kernel's own import (R140), for the
ordinary certificate too (F15).

- **`Kernel::Auto`** is `NanoYalla` at 28 (the library does not exist
  yet); step 31 adds `Linlog` and makes `Auto` write NanoYalla's script
  exactly as today for a proof in classical mode without Mix and affine
  weakening that uses only NanoYalla's rules, and the library's term
  certificate otherwise (R139, B10). The classical `.v` snapshots
  (`ll`, `mll`, `mall`, `mell`, `labels`) stay byte-identical under
  `Auto`; `ill.v` and `labels_ill.v` are written with `NanoYalla` forced,
  which certifies an intuitionistic proof as the classical proof of its
  one-sided sequent, as today (R159). `NanoYalla` refuses the rest
  (`Unsupported::Rule { kernel, rule }` for Mix, `AffineWeakening`,
  `Cut`, `Forall`, `Exists`; `Mode` for an ordered mode). The script
  builds the derivation the command builds today (`Sides::of(mode)`,
  `Compact::Never`) within the limits, so `Unsupported::Compact` goes;
  `Open` stays refused (R157 can come as an `Item::Session`); a goal proof
  is `Error::GoalProof`.
- **The term certificate** (kernel `Linlog`, R152) states the sequent
  over the library's plain inductive (`ll p [formulas]`, `p` the record
  `{mix; affine}`; intuitionistic, `ill a [hypotheses] goal` from the
  `Reading`, R54), schematic in the atoms by a substitution lemma, and
  proves it `by_check` over data: the formulas in written order (the Rocq
  side recomputes the forest by 3.2's contract, R69), the nodes in arena
  order as a list of `node nat` (one line each, shared nodes once:
  linear in the proof, polled per node), and in intuitionistic mode the
  positions as data, which the Rocq checker verifies locally (each
  position agrees with its parent's by the grammar; the goal is the last
  root) instead of mirroring the reading's choices. It begins `(* linlog
  0.1.0, rocq-linlog 0.1.0, certificate format 1 *)` and `Check
  Linlog.Certificate.format_1.`, so a library without that format fails
  on that line by name: what ties the writer to the library (R12). A
  flake check compares `LIBRARY` with `rocq/rocq-linlog.opam`.
- **The mirror** (R117, R118, D20): `node (M : Type)`, one constructor
  per `Node` variant in `Node::TAGS` order, the operand of type `M`
  (`nat`, an occurrence, in the propositional development; an index into
  the certificate's instance table in 38's development beside it, R68),
  premises as list indices; `Linlog/Tags.v` lists the constructors for
  R118's test. `check` is a fold over the nodes with a map from index to
  state, a translation of `oracle.rs` (3.6): `Θ` a sorted list, `Γ` a
  sorted list with counts, `any` a bool. One soundness theorem per
  calculus, in `Prop`, no `Admitted`, `Print Assumptions` closed (B11).
- **Refutation certificates** (R153): `Item::Disproof` states `~ ll p
  [formulas]`, proved from the kind's lemma and a computation over its
  numbers (`Unbalanced` the atom, `Equation` nothing, `StateEquation` the
  weights, 31's `Classical` the assignment); `Exhausted`, and every
  refutation on NanoYalla, is `Unsupported::Refutation`.
- **Reserved names per kernel** (R155): the crate-private
  `identifiers(atoms, &reserved)` takes the kernel's set, a hash set
  (F3); the new statement printer is one loop over `Visit` (R156).
- **Cut** (34): `params.cut`, a `Cut` constructor, `FORMAT` 2; the
  eliminated proof certifies with `cut := false`. **First order** (38):
  the second development, `FORMAT` 3 (R161, R242).

### 9.3 Drawings and their ids

`svg::Style::ids` documents the id grammar as public contract (R163):
`i<n>` an inference of the drawing, `i<n>-<p>` the formula at position
`p` (the position `Step` takes; `derivation_ids` maps `n` to a
`GoalId`), `o<n>` a net vertex, `l<m>-<n>` a link by vertex ids, and
`b`, `d`, `j`, `c` reserved for boxes, doors, jumps and cut links.
`svg::Style` is `#[non_exhaustive]` and gains box and essential-net
fields at 33 and 35 with defaults that keep every snapshot (R144, R166).

## 10. How each later step enters

Each step names what it adds; only 34 and 38 break a public item, and
only the closed enums, as planned (P5, B1).

**29, comparison.** Translators use the public `Sequent::walk` and
`Reading` (R245); drivers read `Sequent::fragment` and `Mode::name`
(R75); linlog's default column runs `search::race`, its one-thread column
`jobs(1)`, under one `memory_bytes` (R85, R151, R18); proved and
unprovable outcomes are checkable ground truth. LLTP's written roles
(R74) are a field of `lltp::Problem` (non-exhaustive); the routing
feature per problem (R204) an additive query beside `engine_for`.

**30, release.** The changelog's policy line names the closed enums and
the bumps (0.2.0 at 34, 0.3.0 at 38) and has a heading for wire-form
changes (R6, R221); `rocq::LIBRARY` equals the opam draft (R162).

**31, Rocq.** Adds `Kernel::Linlog`'s writer, `rocq/` with the mirror of
3.6 and 9.2, refutation certificates, the classical-assignment refuter
and `Refutation::Classical`, `refute_unknown`, and the tests R118 and
R199 (the Rocq checker against the Rust one on mutants). `node (M :
Type)` leaves room for 38; `params` is a record 34 extends. Nothing
breaks: `Kernel`, `Unsupported`, `Refutation` are non-exhaustive.

**32, web.** `linlog-web` passes JSON both ways (the session, `Step`,
`Outcome`, `Error`, `Styles`, `Limits`), builds with
`Limits::browser()`, stops with a `WithProgress` that reads
`performance.now()` every N units of work, reads every form through
`forms::Within`, and binds clicks to 9.3's ids.

**33, MELL nets.** Adds the vertex table (instances, collectors, doors,
box nodes), `BoxId`, `boxes`, `box_of`, `depth`, `jumps`, the
generalized `?` node (n ≥ 0 premises), one door per (`?`-instance, box),
jumps for weakening and `⊥` (C3, A), the criterion per depth,
sequentialization through boxes, `from_proof`'s exponential arms,
`NetError`'s box variants, v2 of the net form, `Style`'s box fields.
`Criterion::admits` takes MELL and still refuses affine mode (R145);
forcing the net engine on MELL still refuses (R89). Nothing breaks: the
API speaks `VertexId` from 28 and no `Node` is needed.

**34, cut.** Adds `Node::Cut` and `Rule::Cut` (0.2.0), `Fault::{NotACut,
Ordered}`, the checker's and oracle's arm (3.6), `Forest::with_cuts(&sequent,
&cuts, &limits)` (`cut_pairs` and `dual` exist from 28), v2 of the proof
form, the reading of a cut pair, `Interactive::cut`, and elimination:

```rust
pub mod cut {   // in proofs
    #[non_exhaustive] pub struct Options { pub strategy: Strategy }       // serde, default
    #[non_exhaustive] pub enum Strategy { #[default] LowestFirst }        // principal cases first
    pub fn step(proof: &Proof, at: Option<NodeId>, options: &Options, limits: &Limits, stop: impl Stop) -> Result<Reduction, Error>;
    pub fn eliminate(proof: &Proof, options: &Options, limits: &Limits, stop: impl Stop) -> Elimination;
    #[non_exhaustive] pub struct Reduction { pub proof: Proof, pub cut: NodeId, pub case: Case }
    #[non_exhaustive] pub struct Elimination { pub proof: Proof, pub steps: Vec<(NodeId, Case)>, pub end: End }
    #[non_exhaustive] pub enum End { Normal, Refused(Refusal) }    // a bound is never "normal" or "invalid"
}
```

Every intermediate proof lives over the same extended forest and passes
the checker; a shared subproof is duplicated by reference; `work` counts
steps, `memory_bytes` the arena (R32, R100). Its written form (R15):
`{"version": 1, "steps": [{"cut": 4, "case": "axiom"}, …], "end": …,
"proof": …}`. Net elimination (R101) is a step function with cut links
(`cut(x, y)`, `cuts()`), an erased set, `NetError::CutNotDual`, v3 of
the net form, and an `Item` to draw it. The search stays cut-free and the
net engine never sees a cut tree. Rocq: `FORMAT` 2.

**35, MLL engines.** `Criterion` gains the essential criterion,
`NetError` `DirectedCycle` and `NoDominator`, `Options` a non-exhaustive
value of the net prunes' switches (R143), `Statistics` counters, an
engine or criterion with its row, an essential-net drawing; a net's
positions come from `Reading::new(net.forest())` (R81).

**36, Lambek and cyclic MLL.** `Mode::order`, `Order`, its words,
`validate`; `Modes::Ordered` and `EngineMode` for every other engine
(R90); the planar criterion; `Named`'s orientation for the divisions;
`Split::At`; `Fault::{Order, EmptyAntecedent}` (R4); the division bit;
the parser's `\` and `/`; `SequentBuilder::dual` taking the order (the
cyclic dual reverses a product, R56); the ordered reading's sequence of
hypotheses. The written order is canonical from 28, so no stored id
moves.

**37, inverse method.** `Engine::Inverse` documenting the options it
reads (R147); the `Zone` trait with `Context` and `Classes` lifted to
`search/` in a counter-neutral commit first (D-7); the database charged
to the account (R36); saturation's `Reason` or `Refutation` (R111);
counters. Its forward derivation converts into today's `Node`s (R127).

### 10.38 First-order linear logic, in full

**The first step of adding quantifiers** is one commit of the data model
alone, behind the spike's verdict: the `Term` and `Kind` variants, the
empty arenas, the fragment bit, unary binders in the forest, and
`FIRST_ORDER` refused everywhere: `dual_literals` refuses a literal with
arguments, `Proof::check` and the oracle a forest with arguments
(`Fault::Instance`) until `Ax` compares instances, every engine's
`admits` the bit (`Error::EngineFragment { engine, fragment }`, R136), so
no engine searches a predicate as an atom. The target set's counters are
identical after it; everything below lands behind that refusal.

*Atoms as predicates over terms.* `Term::{Pred, DualPred}(Atom, Args)`,
`Args` an index into a CSR table of argument lists over a second,
hash-consed arena (`FoTerm::Bound(u32)`, a de Bruijn index;
`FoTerm::App(Symbol, Args)`), symbols `(name, arity)`, predicate arities
beside the atoms. A literal group `literals(atom, sign)` holds
candidates; `dual_literals` compares instances. Ground `p(a)` against
`~p(b)` sets `FIRST_ORDER` with no binder present.

*Binders in the arena.* `Term::{Forall, Exists}(TermId)`, locally
nameless: bound variables nameless (`∀x.p(x)` and `∀y.p(y)` hash-cons to
one term), display names in a side table. NNF swaps them; `∀` is
negative, `∃` positive; the forest numbers a binder as a unary
occurrence. `Forest::formula(o)` prints under the names of the binders
above `o`; nothing recurses over a term (R43); `Limits::term_size`
bounds unfolded terms at every reader and printer (R250); the reading
extends to `∀` and `∃` (R65).

*The substitution beside the forest.* The sequent's arenas stay
read-only; a search arena holds metavariables by level (the eigenvariable
condition a comparison of levels), bindings and frames. A framed zone's
member is `(OccId, FrameId)` (3.3). Memo keys only on ground stable
sequents (fo-linear §5.3); a failure may be shared up to a renaming of
eigenvariables, a proof may not.

*The trail of bindings.* One iterative unifier with an occurs check and
an undo trail, a mark per choice point (R106), beside the focused
engine's branch where pending arena nodes are released, charged and
polled (R39). Premises of `⊗`, Mix and `&` share metavariables, so a
first-order engine enumerates a premise's answers or picks witnesses
from a finite set, and the pool does not treat `&` premises sharing one
as independent (R107). The propositional engine keeps one answer.

*Witnesses in proofs.* `Node::Forall(Member, Eigen, NodeId)` and
`Node::Exists(Member, WitnessId, NodeId)` (0.3.0); the proof owns the
witness arena, the eigenvariable count and the instance table, closed
under the final substitution (`Proof::with_instances`). The checker
interns members, compares instances by hash-consed id through their
frames (never unfolding a term), and checks the eigenvariable condition
(each introduced by one `Forall`, in no root, used only by witnesses
above it) with a set flowing up with the state: `Exists` adds its
witness's, `Forall` removes its own, the root ends empty (fo-linear
§4.2, for 38's panel). `Inference` gains `datum`, `Rule` `Forall` and
`Exists` (two-sided names through `Named`), `Size` counts substituted
characters, `Step` takes a witness or leaves it open. JSON: v3 of the
proof, v2 of the sequent and session; a test per form reads a pre-step
file unchanged. The outputs print through `Notation`, one loop over
`Visit` with term stops (R170); Rocq's first-order development is
`FORMAT` 3; the ordinary layer gets quantifier rows over the same term
type. A first-order net with one unifier on its links (R108) is an
additive table and key, later.

*What stays untouched for the propositional case*: `Term` (12 bytes),
`Kind`, `Node` (16 bytes), `OccId`, `Member` (the occurrence itself), the
forest's arrays and literal CSR, `OccSet`, `Context`, the memo keys, the
checker's `Bag` of `u32`, every propositional JSON form byte for byte,
the dispatch rows, `Mode`, the count equation, the reading; the focused
engine monomorphised at `Z = Context` with no frame, level or
unification; the checker's ground pass with no member table. The target
set's counters stay identical and pinned CPU time within 2 % (11).

## 11. The spike

What the term data model costs the propositional case, measured in a
throw-away jj workspace from the tree of 2026-10-09, built only far
enough to compile, pass the tests and run the measurements, in three
cumulative parts so that a rejection names its part:

1. **Terms.** `Term` gains `Pred(Atom, Args)`, `DualPred(Atom, Args)`,
   `Forall(TermId)`, `Exists(TermId)`; `Kind` the four tags; `Sequent`
   an empty first-order arena and `arities`; `Fragment::FIRST_ORDER` set
   by `Sequent::fragment`; every `match` over `Term` and `Kind` names
   the new variants in a refusing arm (no wildcard), `optimize_terms` and
   `map_subterms` included; `size_of::<Term>() == 12` asserted. Parser
   and JSON untouched.
2. **Members and the term.** `Member` replaces `OccId` in `Node`'s
   operands, the checker's `Zone` and `Bag` keys, `Dyadic`, `Inference`;
   `Node` gains `Cut`, `Forall`, `Exists` with refusing checker arms; the
   checker's pass is generic over a crate-private `Space` with the ground
   instance (identity, no table).
3. **The zone parameter** (D-7). `Context` and `Classes` lifted to
   `search/`, then the focused engine as `Engine<Z: Zone>` at
   `Z = Context`, with the memo's `Key<Z>`.

**Measured** on the same cores before and after, against a reference
built from the same tree: `bench/targets.sh` (`verdict`, `nodes`,
`splits`, `memo_hits`, `memo_entries` against the oracle
`bench/TARGETS.md` names), the twenty journeys under callgrind
(`ratchet`), pinned CPU time of the target rows over a second; the
command's `.text` size is recorded, not gated. **Accepted** when every
decided row's counters are identical, every journey within the
ratchet's 2 % of the reference (`read-*` for part 1, `check-*` and
`derivation-chain-64` for part 2, `search-*` for part 3), and pinned CPU
time within 2 %. **Rejected** part by part, with the fallback: part 1,
the variants wait for 38 behind a literal-access trait; part 2, the
ground checker pass stays as is and 38 adds a framed pass beside it;
part 3, `focus` stays monomorphic and 38 duplicates the module for the
framed zone (D17's duplicated fast path). `api.md` records which.

## 12. Findings answered

F1 (P5, 2.2, 3); F2 (4.1: `NetMode`'s message names the modes nets
exist in); F3 (9.2); F5 (9.1); F7 (3.4); F8 (4.1); F10 (3.7, 3.9, 6);
F12 (5.4); F13 (3.11); F15 (9.2); F16 (5.3); F18 (7.2: every serde type
documents its form); F19 (3.3); F21 (3.8, 5.3); F22 (3.8); F23 (3.8);
F24 (3.1); F25 (3.1); F26 (7.1); F27 (5.3); F28 (3.9); F29, F34 (4.1);
F30 (4.1, 4.2); F31 (4.1); F36 (9.1); F38 (9.2); F40, F64 (3.4, 6); F48
(4.1); F49 (3.9); F51 (2.1); F53 (3.11); F56 (2.2; the engine-internal
names are the search area's); F58 (3.2); F60 (3.7); F61 (7.1); F62, F63
(5.1); F65 (3.7); F66 (3.4); F67, F68, F69 (3.8); F70 (3.7); F76, F77,
F78 (6); F79 (5.1); F80 (3.10, 7.2); F81 (6); F84 (3.1); F85 (3.5); F86,
F87, F88 (7.2); F89 (3.5, 7.2); F98, F139 (4.1); F104 (5.5); F136
(5.2); F141, F144 (8); F208 (7.2); H2, H3, H4, H5, H8, H16 (13, HD1 to
HD4); H9, H10 (3.4); H17 (3.8); H18 (5.5); H19 (5.3, HD5). The tests
the audit asks for (F11, F17, F55, F57, F71, F93 to F98) and the
search's internal findings are the fix areas'; this design moves nothing
they pin.

## 13. Decisions for the author

**The open ones, on their recommended answers**, and what the other
answer changes here:

- **C1**, written root order now: A (3.1). B (at 36) keeps the sort, the
  contract says "sorted", 36 breaks every stored id, and the reading
  cannot read the written succedent.
- **C2**, refuters in the library, Unknown decided only by option: C
  (8). A drops `Refutation::Classical`, the refuter list and
  `refute_unknown`, and 31's exporter searches; B drops `refute_unknown`.
- **C3**, no nullary Mix: A (3.8, 10). B adds a zero-premise Mix node (a
  bump, a Rocq constructor, a checker arm) and makes `EmptySide`
  closable.
- **T1**, rename the keys with the versions: yes (7.1); "keep" keeps
  `ids`, `var_dict`, `proof` in 7.2. **T2**, owned forests: yes (P7);
  `Arc<Forest>` would let `Disproof` hold the forest, nothing else
  changes. **T3** to **T7**: no effect (T5's shared counters assumed).
- **HD1** refuse several conjectures; **HD2** a `.spec` file is affine;
  **HD3** NFC in the readers; **HD4** `load` refuses another sequent or
  mode; **HD5** refuse a JSON atom name that is no identifier. Each
  other answer changes only its reader.

**The ones this design adds:**

- **B1** closed core enums with planned bumps (P5), against
  `#[non_exhaustive]` on `Node`, `Rule`, `Term`, `Kind`, which makes 34
  and 38 non-breaking but forces consumers into wildcard arms that
  silently mishandle a cut or a binder. Recommended: closed (R50, F1).
- **B2** a proof records its conclusion and its claimed mode (3.5),
  against a separate goal-proof type and the mode only in the outcome.
- **B3** `Verdict::Unprovable(Box<Disproof>)`, a sequent clone per
  unprovable outcome and its `sequent` key in the outcome, against
  `Unprovable(Refutation)` with callers building a `Disproof`.
- **B4** `CheckError` as fault or refusal variants, against today's
  struct with `is_refusal`.
- **B5** one `Limits`, with `ViewOptions`, the renders and the checker
  losing their own bounds, against keeping them and a presets holder.
- **B6** data forms ignore unknown keys under versions, options and
  `Mode` refuse them, against refusing everywhere (the outcome would then
  nest its proof, another lock change).
- **B7** `mix-affine` for Mix with affine, against `affine-mix`.
- **B8** the checker accepts `Cut` in every commutative mode with no
  flag ("cut-free" is `has_cuts`), against a `cut` flag in `Mode` (R119's
  other wording).
- **B9** closures stay stops, progress through `WithProgress`, against
  every stop taking `Progress`.
- **B10** `Kernel::Auto` by the mode and the proof's rules, against the
  mode alone (which sends a classical cut proof to NanoYalla's refusal).
- **B11** for step 31: `Prop`, the forest recomputed, schematic lemmas,
  positions as data, `Surplus` a lemma; only "the Rocq side recomputes
  the numbering" is fixed by this design.
