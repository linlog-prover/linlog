# Walk-through of step 36 (cyclic MLL and the Lambek calculus) against `plan/notes/api.md`

Read: the whole design (sections 1 to 14), `plan/36-lambek.md`, the research note
`plan/notes/research/36-lambek.md`, the register entries naming 36 (R6, R10, R50 to R53,
R55, R56, R61, R79, R88, R90, R96, R104, R121, R134, R146, R169, R176, R177, R206, R207,
R209, R216, R230, R237, R241, R246, R248), and the present `Reading`, `Forest`, parser,
`Mode`, `ProofStructure`, `Nets::admits`, `Modes`/`Feature`, `Interactive::close` and
`families::Instance`. Nothing was built or run; the claims about the planar-order walk of
10.8 (a) are checked by hand on four small sequents only (below), not proved.

What the prompt asked to be costed: the written root order. The design lands it at 28 (C1),
so at 36 it costs nothing; the cost (snapshots, the target rows whose root order changes,
`translate`'s root pairing) is booked to 28's lock commit 2 (7.5). 36 inherits `antecedents`.

## 1. The first change, sketched

Two commits, as a session would write them against the design.

**Commit 1: the order in `Mode`, with its words and the dispatch row.**

```rust
// fragment.rs (or sequents/mode.rs)
/// How a calculus orders the formulas of a sequent.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Order { #[default] Commutative, Cyclic }          // new public enum: 2.2 and 2.5 must list it

#[non_exhaustive] #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Mode { intuitionistic: bool, affine: bool, mix: bool,
                  order: Order, nonempty_antecedents: bool }     // 3.5: "order, empty_antecedents"
impl Mode {
    pub const CYCLIC: Self;        // classical + Cyclic
    pub const LAMBEK_STAR: Self;   // intuitionistic + Cyclic, empty antecedents allowed
    pub const LAMBEK: Self;        // LAMBEK_STAR + nonempty_antecedents
    pub const fn order(self) -> Order;
    pub const fn is_ordered(self) -> bool;
    pub const fn needs_antecedent(self) -> bool;                 // R4's switch
    #[must_use] pub const fn with_order(self, o: Order) -> Self;
    #[must_use] pub const fn with_nonempty_antecedents(self) -> Self;
    // NAMES gains "cyclic", "lambek-star", "lambek"; check() refuses ordered+affine, ordered+mix,
    // nonempty without ordered+intuitionistic (Error: code ordered_mode, kind unsupported)
}
// search/mod.rs
enum Modes { Any /* commutative only */, Linear, Classical, Intuitionistic, Ordered }  // take() destructures the whole Mode
Row { fragment: Fragment::MLL, modes: Modes::Ordered, feature: Feature::Any, engine: Engine::Net }
// Nets::admits: accepts Ordered; still NotTaken::{Fragment (units), Mode (affine), Goal (not roots)};
// every other Decide::admits answers NotTaken::Mode(mode) for an ordered mode
```

Wire: `"mode": "cyclic" | "lambek" | "lambek-star"` (3.5, 7.4); the writer lifts `version` to the
level 36 owns; a 0.1.0 reader refuses the word by name. Pre-release flag objects read as
commutative. Tests: round trip over `Mode::NAMES`; every `Decide` other than `Net` refuses
`CYCLIC` and `LAMBEK`; the target set's columns equal (commutative rows unchanged).

**Commit 2: planar nets (the criterion, the rank, the crossing refusal).**

```rust
#[non_exhaustive] pub struct Criterion { pub mix: bool, pub order: Order, pub nonempty: bool /* + see W4 */ }
impl Criterion { pub fn of(mode: Mode) -> Result<Self, Error>; }       // affine -> NetError::Mode
// occurrences (candidate (a) of 10.8; see W1, W3)
pub struct PlanarOrder { rank: Box<[u32]> }                             // per occurrence; identity unless a Reading is given
impl Forest { pub fn planar_order(&self, reading: Option<&Reading<'_>>) -> PlanarOrder; }
// nets
pub enum NetError { /* … */ Crossing { first: (VertexId, VertexId), second: (VertexId, VertexId) } }
impl ProofStructure {
    pub fn rank(&self, v: VertexId) -> Option<u32>;                     // None in a commutative structure
    // link() refuses a crossing with NetError::Crossing; is_correct() = switching test + planarity
}
// search/net.rs: a third constant-time rejection beside lca and skeleton: for a candidate (x, y), scan the
// k links made for u<x<v<y or x<u<y<v on ranks; symmetry breaks (copy_before/after) off when order != Commutative
```

Wire: `{"version": N, "sequent": {…}, "mix": false, "order": "cyclic", "links": [[0, 3], [1, 2]]}`
(3.11 and 7.3: the criterion's fields flattened, `order` at a level). For a Lambek net this
form cannot say which planar order applies (W1, W4).

## 2. Workarounds

Zero blocking, seven friction, five notes. Each friction item is removable by an additive change,
but each is a 28-fixed signature or an unplaced decision that 36's session would otherwise meet
in the middle of the step.

**1. [friction, the most important] Candidate (a) of 10.8 cannot serve classical `cyclic`; the
parser has no place for the order. (3.1, 10.8, decision 17, R56, R177)**
Need: in cyclic MLL the dual of `A ⊗ B` is `B⊥ ⅋ A⊥`. The parser pushes negation with
`Term::dual`, which keeps child order (`Tensor(k,l) -> Par(k,l)`, `term.rs`), at `~`, `^`, the
antecedent of `-o`/`\`/`/` and left of `⊢`. (a) recovers the planar order from D1's lowering
through a `Reading` (which dualised nodes are input position). A classical cyclic sequent has no
`Reading` (`Reading::new` fails on `|- A par B`), so there is nothing to walk: with the
order-keeping dual, `|- (a * b) -o (a * b)` is the NNF `(a⊥ ⅋ b⊥) ⅋ (a ⊗ b)`, literals
`a⊥ b⊥ a b`, whose axiom links cross, so `A -o A` is answered unprovable for a compound `A`
and `|- ~(a * b), a, b` is unprovable though Yetter's `(a⊗b)⊥ = b⊥ ⅋ a⊥` makes it provable. A
silent wrong answer against the published calculus, in the mode named `cyclic`. So (b) (the
reader dualises with reversal under an ordered parse) is forced for `cyclic`, and once it
exists it serves `lambek` too and makes the planar order the preorder (no rank array, W3
vanishes, `Inference::sequent` ascending is planar, R61 is trivial). But 3.1 fixes
`parse_within(text, &Limits)` and `FromStr`, neither of which can carry an order; and 3.1's
"roots as written" bullet is false for a Lambek parse under (b) (the antecedents come out
`An⊥ … A1⊥, C`), as is R55's "hypotheses in written order".
Why it matters: a silent wrong verdict; a late discovery rewrites 3.1's parser entry and the
lock for README's examples.
Smallest change: in 3.1 and 10.8 say that (a) is Lambek-only and that `cyclic` needs a
reversing dual in the reader; reserve the input now: `Sequent::parse_within(text, &ParseOptions)`
with `ParseOptions { limits: Limits, order: Order }` (`#[non_exhaustive]`, `FromStr` its
default), or at least name `parse_with_order` as 36's additive sibling; state in 3.1 that C1
means "as lowered" and that a two-sided `Display` goes through `Reading`.

**2. [friction] `Reading::new(forest)` has no mode or order, but 36 needs three behaviours from
it. (3.6, 10.8, R55, R246)**
(i) 3.6 says that with the sides known the antecedent of an implication is the left factor and
the symmetric reading is not tried; 10.8 says that in an ordered mode the antecedent is
whichever factor is in input position (that is what makes `B / A = B ⅋ A⊥` differ from
`A \ B = A⊥ ⅋ B`). With a mode-less `Reading::new`, `b / a, a |- b` is refused in `-i` mode
(as `b par ~a` already is: consistent, but the message must say to write `\` or `-o`) and
must be accepted in `lambek`; one constructor cannot do both. (ii) R55 and R246's refusal of an
empty antecedent is wrong in commutative ILL (`|- a` is fine). (iii) With `antecedents ==
None` (pre-release JSON, `Sequent::add`) 3.6 falls back to today's guess of the goal and the
symmetric reading; in an ordered mode the guess changes the planar order, so it must refuse.
Smallest change: `Reading::new(forest)` stays; add `Reading::of_mode(forest, mode)` (or
`within(forest, Order)`) returning `ShapeError::{EmptyAntecedent, SidesUnknown}` (the enum is
`#[non_exhaustive]`), and `Reading::division(o) -> Option<Division { antecedent, consequent,
side }>` for the printers and `Named`'s orientation. Say in 3.6 that the left-factor rule
is the commutative one.

**3. [friction] The planar rank is a second numbering outside 3.3's contract, and no signature
says who owns it. (3.3, 8.2, 3.8, 3.10, 3.11, 9.3)**
Under (a) the literals' planar order is not `OccId` order, so the net engine's crossing test,
the checker's arcs, `Interactive`'s positions, `svg::net`'s layout and the Rocq mirror (31 has
the forest fixture of 3.3, not this) all need a rank per occurrence. 10.8 writes
`Forest::planar_order(&Reading)`; `Task` has "[36] the planar order" and `Proof::check`,
`ProofStructure`, `Interactive`, `Derivation` have nothing. A foreign checker cannot recompute
what 3.3 does not specify. Cost: one `Box<[u32]>` per structure/check/session, charged to
`memory_bytes`, none for commutative modes.
Smallest change: a fourth promise in 3.3 (`planar_order` is a pure function of the forest, the
sides and the order; pinned by `core/tests/forest.rs`; computed in one ascending pass over the ids,
since a parent precedes its children, no recursion, P9); `Task { order: Option<&PlanarOrder> }`;
`ProofStructure::rank(VertexId)` public (the SVG layout needs it, W3 of 9.3); `Option<PlanarOrder>`
owned by `ProofStructure` and `Interactive`, recomputed by `check`. (Moot if W1's (b) is chosen.)

**4. [friction] `Criterion { mix, order }` cannot say L against L*, nor Lambek against cyclic.
(3.11, 8.1 step 6, R79, R246, research section 4)**
The net engine decides L* on the lowered sequent. If the restriction lives only in the checker
(R4, `Fault::EmptyAntecedent`) and the sequentialization, a net found correct under the
criterion can sequentialize only to proofs the checker refuses, and 8.1's `conclude` turns that
into `Error::Rejected` (a defect), or the engine answers `Unprovable` for a sequent with
another net. So the restriction must be inside the engine's acceptance, i.e. part of the
criterion, with its own `NetError` (`EmptyAntecedent`). Separately, the same `order: Cyclic` is
written for `cyclic` (rank = preorder) and `lambek` (rank from a `Reading`), so a stored net's
`"order": "cyclic"` is ambiguous for any sequent with a reading.
Smallest change: 3.11 lists `Criterion { mix, order, nonempty, planar_from_reading }` (all
under `#[non_exhaustive]`, flattened on the wire, at a level), `NetError::EmptyAntecedent`,
and names the net-level condition as the report's open item (Roorda 1992, Moot and Retoré
chapter 6); `Criterion::of(mode)` maps both from `Mode`.

**5. [friction] "Switched by the criterion's `order`" puts a branch in the hot loop. (10.8, 10.7,
P10, D17, R206)**
10.7 makes the engine `Linker<C: Correctness>` so the commutative instance pays nothing; 10.8
makes planarity a runtime switch per candidate inside `choose`/`next_partner`. D17 asks for
monomorphisation, and the gate is the target set's `links`/`tests` and pinned time.
Smallest change: 10.8 states planarity as a second type parameter (or a `Planar<C>` wrapper of
`Correctness`) selected once in `decide`, with `Commutative` the empty instance; the R206 gate
is then "instructions of the net journeys unchanged".

**6. [friction] The net engine takes only the roots, so `Interactive::close` has no engine in an
ordered session. (3.10, 3.11, 8.4, 8.6, R90, R96)**
`Nets::admits` refuses `!task.roots` (`NotTaken::Goal`), a structure's vertices are the whole
forest (`VertexId(i) == OccId(i)`), and 10.8 gives every other engine `NotTaken::Mode`. An
open goal of a Lambek session is a sub-sequence of members, so `close` and `close_all` (public in
3.10, used by the web client) would answer `EngineRefused` for every goal but the initial one.
R96 does not ask for `close`, but 3.10 promises it for every session.
Smallest change: either say in 3.10 that `close` in an ordered mode closes only the initial
goal (documented `EngineRefused { Net, Goal }`), or reserve in 3.11 a goal-subset vertex table
(`ProofStructure::of_goal(Goal, Criterion)`, `VertexId` through a table, `None` for the whole
forest), which 34's `is_whole_forest` and 33's instances already half-need.

**7. [friction] `Goal` and rotations. (3.4, 8.1, 8.2, 10.8)**
10.8 says `prove_goal` "accepts the roots in a cyclic mode" up to rotation, but `Goal::new(forest,
&[Member])` decides `is_conclusion()` without a mode, and `Task.goal` is documented "in any
order" (`is_roots` sorts today). In an ordered mode the slice is a cyclic sequence and a
non-rotation permutation is a different goal that needs exchange.
Smallest change: 8.1 defines a goal's members as a sequence; `is_conclusion()` is "a rotation of
`roots()`" (the multiset test stays for commutative modes, applied in `prove_goal`, which has the
mode); a permutation that is no rotation in an ordered mode is `Error::GoalMismatch`.

**8. [note] `Mode` details. (3.5, 2.2, 2.5, 4.4, R10, R50, R51, R176)**
`empty_antecedents: bool` with default `false` reads as "no empty antecedents" for commutative
ILL, where they are allowed, and gives two equal-meaning `Mode` values (`Eq`, `Hash`); name it
`nonempty_antecedents` (default false = unrestricted) and let `check` refuse it outside
ordered intuitionistic. `Order` is a new public enum absent from the re-exports (2.2) and from
2.5's lists. The builders and getters for 36's fields (`order()`, `with_order`, the flag's pair)
are not listed, and the command's flags (R176) need them. Codes `ordered_fragment`,
`empty_antecedent`, `ordered_mode` are reserved (4.4) without variant, kind or fields; give them
rows: `unsupported` for the fragment and the mode, `invalid`/`malformed` for the antecedent.

**9. [note] Derivation and session forms. (3.9, 3.10, 7.3, 10.8, R61, R96)**
3.9 comments `Inference::sequent()` "the written sequence when ordered", 10.8 says "planar order";
they differ under (a) and agree under (b). `Needs::{Nothing, Split}` cannot tell a UI whether to
ask for a subset (`Split::Left(Vec)`) or a cut point (`Split::At(usize)`): add `Needs::Cut`
(non-exhaustive, additive). `Named` gains an orientation field (3.9 already says so) and
`FromStr`/`name` four new strings (`\L`, `/L`, `\R`, `/R`, backslash escaped in JSON). The
meaning of the `sequent` arrays in a session and a derivation changes from ascending to a sequence;
both forms are already read under the `mode` word, so no key is needed, but the reader's
validation (H17) differs by mode and should be written down. `StepError::Exchange` appears
only in 10.8, not in 3.10's list.

**10. [note] The builder's `dual`. (3.1, 10.8, decision 16)**
10.8 says "the builder's `dual` takes the order (3.1)", but 3.1 gives `dual(TermId)` and the builder
may ship at 32 or 34, before 36. Reserve `Builder::order(Order)` (a setter before the first
`dual`) or `dual_in(Order, TermId)`; adding a parameter later is a break.

**11. [note] The checker's specification has no ordered clauses. (3.8, 9.2, R117, R121, R246)**
The rules table of 3.8 is the text 31's Rocq function translates; 36's `Order` and
`EmptyAntecedent` are prose. 36 writes the ordered clauses (axiom on adjacent literals in
rank order, `⅋` operands adjacent and in order, the `⊗` premises the two arcs around the
principal, the root equal up to rotation, R4) into the same table and `oracle.rs`, and the Rocq
`params` gains `order` with a new `FORMAT`. `Kernel::Auto` (9.2) must answer
`Unsupported::Mode` for an ordered mode, not pick the library kernel that has no order.

**12. [note] Every `is_intuitionistic()` site treats `lambek` as ILL and silently drops the
order. (3.5, 3.8, 3.10, 8.4)**
`Modes::take` destructures the whole `Mode` (F140), but the checker, `Interactive`, the derivation
(`Sides::Auto`), the exports and the command read single getters. A site that forgets
`is_ordered()` accepts a non-planar proof as ILL, a wrong "valid". Add one `Mode::is_ordered()`
guard at the entry of `check`, `Interactive::within`, `ProofStructure::from_proof` and every
exporter (refusing, until its ordered clause exists), and a test that lists the public
entries and feeds each `Mode::CYCLIC` and `Mode::LAMBEK`.

## 3. Register entries

| entry | met or placed | where / what is missing |
|---|---|---|
| R6 (levels) | placed | 7.1: mode words and `order` raise the level; commutative values stay level 1 |
| R10 (mode in JSON) | met | 3.5, 7.4: words, no key; old object read without `version` |
| R50 (non_exhaustive) | partly | `Order` and the new `Criterion`/`Named` fields: `Order` is not listed (W8) |
| R51 (place in `Mode`) | met | 3.5, 10.8; builders/getters unlisted (W8) |
| R52 (division class) | met | 3.5: no bit, divisions are `⅋` with a negated factor; printing them as `\` `/` needs the `Reading` with an order (W2) |
| R53 (written order) | met | 3.1 (C1), 10.8: cyclic sequents are not equal to rotations, `roots()` the written rotation; but under (b) not "as written" for Lambek (W1) |
| R55 (ordered antecedent, empty refused) | not placed | 3.6 has no order parameter on `Reading` (W2) |
| R56 (reversing dual) | placed, undecided | 10.8 and decision 17; (a) does not cover `cyclic` (W1); parser/builder inputs missing (W1, W10) |
| R61 (inference keeps order) | placed | 3.9, 10.8; comment inconsistent (W9) |
| R79 (calculus descriptor) | placed | 3.11 `Criterion`; L/L* and the Lambek/cyclic split missing (W4) |
| R88 (new mode dimension) | met | 8.4 `Modes` destructures the mode; one variant, one row |
| R90 (ordered routed to planar, refused elsewhere) | met | 8.4, 8.6 `NotTaken::Mode`; `Interactive::close` gap (W6) |
| R96 (interactive ordered) | placed | `Split::At`, `StepError::Exchange` (10.8); `Needs` and `close` gaps (W9, W6) |
| R104 (planar linking) | placed | `NetError::Crossing`, criterion `order`, symmetry breaks off (10.8); hot-loop form (W5) |
| R121 (checker over sequences) | placed | 3.8 prose only (W11); rank ownership (W3) |
| R134 (named errors) | placed, thin | codes reserved in 4.4 without variants (W8) |
| R146 (planar engine, same options) | met | 8.3, 8.5: same `Options`, appended counters, `Progress` unit of work |
| R169 (exports) | partly | `rocq::Unsupported` (9.2); notation symbols and planar-arc layout need `ProofStructure::rank` (W3) |
| R176 (command flags) | placed | 6.6 "mode flags make the request's Mode"; flag names are the command area's |
| R177 (parser: divisions, order-keeping) | not placed | no parser input for the order, `\` and `/` not in 3.1 (W1) |
| R206, R216 (counter-exact target set) | placed | 3.11's gate before the retype; ordered rows add their own counters; gate form (W5) |
| R207 (ordered problems, mode words) | placed | `families::Instance` already carries `mode`; `Mode::NAMES` (R241) serves the harness |
| R209 (reference prover, differential) | placed | 8.3 item 8; a cyclic reference prover in `search/reference.rs` is 36's |
| R230 (room recorded) | met | this design, 10.8 |
| R237 (input of the MILL embedding) | met | C1 plus `Mode`; W1 decides which lowering the embedding reads |
| R241 (mode names) | met | 3.5: one table, `cyclic`, `lambek`, `lambek-star` |
| R246 (restriction on every sequent, L*) | partly | `Fault::EmptyAntecedent` and the mode flag placed; the net-level condition and the engine's completeness are not (W4) |
| R248 (matches name variants) | met | P3: new `Fault`, `NetError`, `Order` arms are compile errors |

## 4. What fits well

- C1 at 28 makes the prompt's "fixed now" item free at 36; `antecedents` gives `Reading` the
  sides without a second type; `Forest` already owns its sequent, so no signature changes for it.
- `Mode` private fields plus names on the wire absorb `cyclic`, `lambek`, `lambek-star` with no
  key and no misreading; `Modes::take` destructuring the whole mode and `NotTaken::Mode` make
  "ordered reaches no engine but the planar one" a compile-time and test-time fact.
- `Rule` as `(rule, side)` with `Named` non_exhaustive takes the four divisions without a
  `Rule` variant; `Node` needs nothing; `Criterion`, `NetError`, `Fault`, `Split`, `Statistics`
  all take 36's additions without a break.
