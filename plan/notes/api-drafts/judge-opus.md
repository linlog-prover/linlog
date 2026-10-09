# Judgement of the three API drafts (judge: Opus 5.5)

Drafts judged: `draft-a.md` (web client and wire forms first), `draft-b.md`
(proof term, checker, Rocq first), `draft-c.md` (engines, calculi,
quantifiers first). Judged against the register, the rubric, the audit's
design list and review (H9, H10, H18, HD1–HD5, C1–C3, T1–T7), the plan's
D15–D18, the research note's D-1–D-12, and the code at the tree of
2026-10-09.

## 0. What I checked, and how

**Claims about the present code, checked against `core/src/`**:

| claim | draft | result |
|---|---|---|
| `Error` has positional index variants, `Unchecked(CheckError)`, `Refused(Refusal)`, `InconsistentState(&str)`, no `#[non_exhaustive]` | all | true (`errors/mod.rs:20-210`); A's "35 variants" is 33, which does not matter |
| `CheckError` is a struct, `Problem::Memory` sits beside the faults, `is_refusal()` on both | A, B, C | true (`proofs/check.rs:448-536`) |
| `ViewError::{Invalid, TooLarge, Memory, TooMany, Stopped}` | A, B | true |
| `Rule::ALL: [Self; 34]` | B, C | true (`proofs/derivation.rs:136`) |
| the crate-private `search::Stop` is `Closure`/`Turn`/`Slice`/`Flags`, `fired(work)` | A, B, C | true (`search/mod.rs:46-85`) |
| `Forest::{new, within(&s, u64), from_owned}`, `TryFrom<Sequent>` | B, C | true (`occurrences/mod.rs:202-220, 387`) |
| `optimize` is fallible and sorts the roots | B | true (`sequents/mod.rs:314-317`) |
| `Options::stack_size` has an 8 MiB floor, 2 304 B a level | A | true (`search/mod.rs:1176-1187`) |
| `svg::net(&n, &style, Option<u64>) -> Result<String, TooLarge>`, `png/pdf::Options::memory`, `ViewOptions::{limit, memory, UNBOUNDED, DEFAULT_LIMIT}` | A | true |
| `Refutation::Unbalanced { atom, name, .. }`; `Statistics::memo_entries: usize` | A, C | true |
| `Interactive::{rules -> Vec<Rule>, goal(InfId) -> &[OccId], inferences(), derivation_ids}`; `ProofStructure::{new(forest, mix), from_links(.., &[(OccId, OccId)]), from_proof(&p, mix), is_correct(), sequentialize()}` | A, B, C | true |
| B's checker table (3.6): `With`'s four cases, the pointwise maximum when both absorb, `Bang` resets `any`, `Copy` checks the parent is `?`, `Weaken` refuses an output, `Mix` refused intuitionistically | B | true against `Pass::rule` (`check.rs:1156-1300`) and `Bag::unite` (`check.rs:218`, a multiset union by maximum) |
| the `DISPATCH` rows | C | C's 8.4 reproduces `search/mod.rs:503-530` row for row |
| the parallel depth is per task (`worker.depth = self.depth`) | B, C | true (`focus/parallel.rs:209, 327`); B states the per-thread counter as if it held, C designs it |
| the text syntax always has a turnstile | (Q2) | true: end of input on the left side is an error (`parse/mod.rs:323`), so every text sequent has its sides; "one-sided text" means `|- Γ` |
| `lltp::read` assembles `axioms |- conjecture` and parses it | C | true (`lltp.rs:10, 85-89`) |
| `serde_json` is not a dependency of `core` | (A3) | true: A's `wire::from_json_within`, `to_json`, `Settings::set` add one behind `serialize` |
| the bench oracle moved to `after-coverability.csv` | C | true per F181; A and B still name `after-bias.csv` |

**One probe** (rustc in the devshell, cores 9–11, capped scope): B's and
C's `Stop` trait (a blanket impl for `FnMut() -> bool`, `WithProgress<F>`
for `FnMut(Progress) -> bool`, and `impl Stop for &mut dyn Stop`) is
coherent and accepts `|| false`, `&mut closure` and `WithProgress(..)`, as
both drafts say. But `run(WithProgress(|p| p.work > 5))` fails with
E0282 ("type annotations needed"): the closure is built before any bound
on `F` is seen, so every progress closure must be written `|p: Progress|`
(B) or `|p: &Progress|` (C), unless `WithProgress` gets a constructor
function bounded by `FnMut(Progress) -> bool`. A's `impl FnMut(Progress)
-> bool` parameter infers `|p| p.work > 5` and `|_| false` (same probe).
So A's argument in 5.2 against the trait is right, and B/C need the
constructor.

## 1. Scores

| § | A | B | C |
|---|---|---|---|
| 1 Principles | 4 | 4 | 4 |
| 2 Public surface | 5 | 4 | 4 |
| 3 Data model | 3 | 4 | 4 |
| 4 Errors | 5 | 4 | 3 |
| 5 Bounds and stops | 5 | 3 | 4 |
| 6 Options | 5 | 3 | 4 |
| 7 Wire forms | 5 | 4 | 3 |
| 8 Engines, front door | 4 | 3 | 5 |
| 9 Exports | 4 | 4 | 3 |
| 10 Later steps | 4 | 4 | 5 |
| 11 Spike | 3 | 4 | 5 |
| 12 Findings | 5 | 4 | 4 |
| 13 Decisions | 4 | 4 | 4 |
| **total** | **56** | **49** | **52** |

## 2. Section by section

### §1 Principles

**A (4).** P1–P10 cover ownership (T2), one error family with a class,
one bounds value, options as data, `#[non_exhaustive]` with a closed list
and planned bumps (R50, F1), a reader-level version (F26, R6), exact
numbers in JavaScript (R247), no recursion over input, D17. P8 and P9 are
principles no other draft states. *Wrong*: nothing material; P4 says
every long call takes `&Bounds`, which the convenience entries (`prove`,
`derivation(mode)`) don't, but they are documented as the defaults.

**B (4).** P1 (the trusted core is small, independent and specified,
refusals outside the verified function) and P2 (a refusal is a variant
of its own in every error type, so a `match` reading "invalid" never sees
one: S4 by construction) are the best statements of the checker's place.
*Graft*: P1's sentence on what lies outside the verified function, and
P2's structural form of S4.

**C (4).** P2, "one id space per kind, extended by offset" (members past
`forest.len()` are instances, cut trees continue the occurrences, net
vertices continue the forest), is the one principle that makes 33, 34
and 38 fit without a shape change on the wire. P3 names the lint
(`clippy::wildcard_enum_match_arm` denied per module) that enforces
R248/D-11. *Graft*: P2 and P3's lint.

### §2 The public surface

**A (5).** The fullest *before → after* table (about 45 rows: errors,
bounds constants, `Options`, the race, members, the derivation entries,
`Interactive`, nets, exports, batch, ordinary, families, `fmt::Walk`),
`lib.rs` written out, the private list, and the list of types made
`#[non_exhaustive]` including every struct-like variant (F1). *Wrong*:
misses `Fragment::ALL → ADDITIVE` (F56) and F60's `Rule` split, which B
has.

**B (4).** A good table that includes what A lacks (F60 `Rule`/`Named`,
F56, `Unprovable(Box<Disproof>)`, `optimize` infallible, the forest's
constructors folded into `new`). *Wrong*: no row for `Family::instance`
(F6) or for the batch's `Plan`/`Problem`/`Answer` beyond one line; removes
`Interactive::inferences()` without saying what a client reads instead
(the JSON, which needs `serialize`).

**C (4).** Clear table, including the forced-engine refusals merged
into `EngineRefused { engine, because: NotTaken }` and `verify_integrity
→ check` (F83). *Wrong*: the root re-exports leave out `Position`,
`GoalId` and `Criterion`-like values a front end names often (taste);
`Sign::{Plain, Dual}` against A's and B's `{Atom, Dual}` (taste, not
counted).

### §3 The data model

**A (3).** Strong on what a front end builds: the checked `Builder` with
`left`/`right` sides (R59, story 5), `Member` with `of`/`get`/
`occurrence(forest)`, `Mode` with names and `validate`, and the most
complete `Interactive` for a web client (`GoalId`, `Step`, `Applicable`
with `Needs::Split`, `GoalView`/`FormulaView`, `within`, `view`; R91,
R93, F66, F69). *Wrong*: (1) the written sides are `antecedents: u32`
with "0: one-sided input", and 3.6 reads one-sided input "by shape only",
so `|- a, top`, H10's own witness, has 0 antecedents and is read as
today and proved; A10 in 13.2 says the opposite (`⊢ a, ⊤` refused). The
type cannot tell a text `|- Γ` from a JSON sequent without the key; C's
`Option<u32>` can. (2) H9's fix is `ShapeError::MovedBottom`, "a ⊥ it
would have to move", with no rule for when the reading "moves" one; the
cause (the symmetric reading of an implication) stays. (3) `Node`'s
operands are left to "draft B" and the net's vertex type to "draft C": a
synthesis built on A must fill both. (4) A's spike puts `Kind` "the four
tags" for `Pred`/`DualPred`/`Forall`/`Exists`, which is right, but 3.2
never says it.

**B (4).** The proof term is the best specified: `Node` with members at
every operand, `Node::TAGS` for R118, `Proof { goal, mode }` with
`new_of_goal`, `with_mode`, `occurrence(m)`, `formula(m)` (B2, F89, F23,
R7); the forest's contract with `cut_pairs`, `dual`, `is_conclusion`,
`dual_literals` (D-2, F58); `VertexId` and `Criterion` (D-6, R77, R79);
`Rule` + `Named` (F60); `Disproof` holding sequent, goal, mode and
refutation (R3, R124). *Wrong*: (1) the reading's rule "the goal is the
last root, every other root reads as input" does not refuse H9's witness
(see Q2) and lets `|- top, a` be proved as `0 ⊢ a`, H10 in mirror; 12
claims both answered. (2) In a first-order owner every member is a table
entry ("`Member(i)` is entry `i`"), so a member's meaning flips between a
ground and a framed owner; C's offset rule (below `forest.len()` a member
is always the occurrence) keeps the propositional prefix. (3) No written
sides at all, so R54's "Reading records the written goal" rests on root
order alone.

**C (4).** `Sequent { antecedents: Option<u32> }` (`None` only for a JSON
sequent without the key) is the one representation of the written sides
that settles H9 and H10 (Q2). `Member` by offset (P2), `Term`/`Kind`
with size assertions, `Mode` with private fields and getters, and the
first-order tables of 3.9 typed in full (`FoTerm`, `FunctionId`,
`ArgsId`, `Eigen`, `FrameId`, `Instances`). *Wrong*: (1) "a `Pred` is
`Kind::Atom`" (3.2): every site that reads a literal by its kind then
reads `p(a)` as an atom with no compile error, which is exactly the
silent pairing impact-quantifiers §1 item 1 warns of; A and B give the
reserved variants kinds of their own. (2) `ProofStructure::new(forest,
Mode)` with links over `OccId`: step 33's vertices past the forest
cannot be named by `from_links(&[(OccId, OccId)])` without a new
signature, against D-6 and R77; R79's "Met when" asks for a net
descriptor value, not the mode. (3) The written sides' JSON (7) is
"absent for a sequent written one-sided", which collapses `Some(0)` (a
text `|- Γ`) into `None` and brings H10 back through a JSON round trip;
it must be absent only for `None`. (4) The builder waits for 34 (R59
allows it; A's is the better graft).

### §4 Errors

**A (5).** Every variant mapped with its code and class (4.3), six
classes (`Invalid`, `Limit`, `Stopped`, `Unsupported`, `Failed`,
`Defect`) so a client and the harness tell "refused" from "unknown"
(R135, R138), `setting()` naming the key that lifts a limit (R129,
R137: the web client highlights it, the command maps it to its flag),
`describe(&forest)` (F48), boxed payloads with a size assertion, the
wire form with `span_utf16` and line/column, the panic hook's reserved
code (R130). *Wrong*: flattening `ViewError`, `WriteError`,
`RenderError` and `Refusal` into `Error` drops C-GOOD-ERR's precise
return types; equally defensible against B, not counted.

**B (4).** `ErrorKind` with `Malformed`/`Invalid`/`Unsupported`/
`Refused`/`Defect`, specific types kept and converting losslessly, and
above all `CheckError::{Invalid(Box<Invalid>), Refused(Refused)}` and
`Fault` without `Memory`, which removes today's hazard (a `Problem::Memory`
beside `Problem::Surplus`) by type, not by a predicate. *Graft*: that
split. *Wrong*: `Error::Parse(Vec<ParseError>)` keeps the `Vec` though
the parser yields one error (`core-sequents.md`); no `setting()`.

**C (3).** Good parts: `EngineRefused { engine, because: NotTaken }`
(R135, F140) and `UnknownName { what, name, known }` for every `FromStr`.
*Wrong*: `ErrorClass { Input, Refusal, Defect }` puts "this engine does
not decide MALL" and "the memory bound was hit" and "stopped" in one
class, which is what the harness must tell apart (refused vs unknown,
R135) and what the command's exit status 2 vs 3 rests on (R138); and
`CheckError` keeps its struct with `is_refusal`, so a caller matching
`problem` still meets the refusal beside the faults (S4).

### §5 Bounds and stops

**A (5).** `Bounds` with `memory_bytes`, `occurrences`,
`derivation_bytes`, `work`, `recursion_depth`, `BROWSER` with the
reasoning behind each figure (R142, pending 32's measurement),
`stack_bytes` and its inverse (R45), what `memory_bytes` does not count
(R29), one account per call and one for the race while the default
bias's pair keeps its halves (R18, so the pinned counters stay), the
stop as `impl FnMut(Progress) -> bool` (the probe confirms its argument),
the unit of work per engine counted apart from the slices (R243),
`Schedule::{Auto, Turns}` (R47), `ReadWithin` (R27), and the long-call
table with a "without them" column (R22). *Wrong*: H18 is left open ("or
keeps to its branch's jobs; the search area chooses").

**B (3).** `Limits` and the `Refusal` enum are sound; the `Within` seed
is format-agnostic. *Wrong*: no scheme option for the default bias (R47
absent, so a work-budget stop behaves differently with and without
`parallel`); `search::race(goal, mode, &options, stop, widen:
&AtomicBool)` has two threads polling `stop: impl Stop` with no `Send`/
`Sync` bound; `WithProgress(|p| ..)` needs an annotation (probe); the
recursion bound stays in `search::Options` beside `Limits`.

**C (4).** `Limits` with `recursion`, the richest `Progress` (`done`,
`item` naming the goal of `close_all` or the problem of a batch, `phase`),
`Schedule::{Auto, Threads, Turns}`, the account table per call including
35's closure and 37's database (R35, R36), and H18 designed:
`Runtime::depths` per rayon worker, a task starting from its thread's
depth. *Graft*: the H18 design and the account table. *Wrong*: the same
`WithProgress` annotation problem; `race`'s `stop: impl Stop` polled from
two threads without `Sync`.

### §6 Options

**A (5).** Every field with its type, default, JSON, reader and flag;
`"auto"` for an automatic choice and `null` only for "no bound" (F79's
fix on the wire); `test_period: Cadence` with its constants (F81);
`Clock` (time limit, pool delay, batch limit) as data the front end
applies (R148, no clock in the library); `batch::Options`, `Plan`,
`Problem`, `Answer`, `Results::cancel`, `Cancel` and `run_local` without
`Send` (R40, R48, F74); `Styles` (R141); `Settings` with the flag → key
table (D15, R183) and the explanation of the two defaults (F42).
*Wrong*: `Settings::set` and `from_json_within` add `serde_json` to the
library (A3 says so; acceptable behind `serialize`).

**B (3).** *Wrong*: the options form uses `null` for "the dispatch"
(`engine`), "the default policy" (`test_period`) and "no bound"
(`copies`), which is F79's ambiguity carried onto the wire; no
`schedule`; `batch::Options` in one paragraph. *Graft*: validated types
for strings that become code (`rocq::Identifier`, Typst `Length`; F38,
F4).

**C (4).** Compact and right on presets (`Limits::BROWSER`,
`search::Options::BROWSER`, `BENCHEXEC` for R151) and on dispatch
thresholds not being options (R149, argued). *Wrong*: `force: null` and
`copy_bound: null` repeat B's F79 ambiguity; `batch::Options::memory` vs
`Limits` per problem left loose.

### §7 Wire forms

**A (5).** The most implementable policy: a global wire level as
`version`, written as the lowest a reader needs, with a list of what
raises it and what does not; unknown keys by class of form (data ignore,
options and commands refuse, write-only: client ignores); pre-release
names read when `version` is absent, so the files of 2026-09-30 load
(R6's "Met when"); enumerations by name with the table of every string a
client meets (7.4, story 3); an example of each changed form; where each
is pinned. *Wrong*: (1) `mix → "criterion": {"mix": …}` changes the
pinned net JSON, which F10's fix and R79's "Met when" keep unchanged (B
flattens `Criterion` so `"mix"` stays); (2) a saturated count written as
2⁵³ departs from R247's and F26's recommended `u64::MAX` (defensible:
nothing a JS client echoes back can then fail to parse; see the
disagreements).

**B (4).** Per-form version, lowest that represents the value; `Mode`
an object that refuses unknown keys (R10 safe); outcome reads back as a
`Proof` or a `Disproof`; the lock changes listed as three commits.
*Wrong*: (1) T1's renames with no reading of the old names, so the
pinned files of 2026-09-30 no longer load (R6); (2) a per-form version
on an outcome that is read as a proof means an outcome-only addition
raises a number the proof reader refuses (A's argument for one level);
(3) `Unbalanced`'s `atom` becomes an index on the wire, where F80's fix
keeps the name.

**C (3).** *Wrong*: (1) no rule for when a version is raised ("a new key
is optional, skipped when empty" only); (2) `Mode` stays an object with
every key `serde(default)` inside data forms that ignore unknown keys,
so a 0.1.0 reader reads a cyclic proof's `{"…", "order": "cyclic"}` as
commutative, which R6 ("fails with a message that names the tag") and
R10 forbid; (3) the `antecedents` key is absent at zero (see §3); (4) the
net form's `mix → mode` changes the pinned JSON (R79). Reads pre-release
names as aliases (R6 met).

### §8 Engines and the front door

**A (4).** Signatures over `&[Member]`, `race` specified best (threads
count both, so `--jobs 2` runs two, F168; `add_pool` asked at polls, no
clock; one account; `Fn + Sync` stop because two threads poll it),
`Decide::admits` refusing the first-order bit (R135), the registration
recipe, `REFUTERS` with `Refutation::check` gating an `Unknown` (C2).

**B (3).** `Goal<'a>` as a validated value with `is_whole_forest` (what
the net engine takes once cut trees exist, R98) is a good idea. *Wrong*:
less detail on dispatch and registration than A or C; the race as
above.

**C (5).** `prove_goal` as seven named stages, `Decide`'s contract in
its doc comments (`Ok(None)` only when exhaustive, polls with work,
charges the account, never panics on an admitted goal; R111), the
ten-item registration list with the four the compiler forces (R86,
F141), the dispatch matching today's table exactly, `NotTaken`, the
per-engine `Statistics` table through step 38 (T5, R9), refuters with
`conclude`'s rule (an engine's own refutation wins, F128). *Wrong*:
making the count refutation a refuter at 28 departs from C2's text
("waits until a second refuter exists"); C records it as decision 5.

### §9 Exports

**A (4).** The sealed `Drawable` (F36's own fix), one signature per
target with bounds and stop, the SVG id contract (R163, R168), Rocq's
additions at 31 additive, PNG/PDF outside the first wasm features (R44).

**B (4).** *Wrong*: one `write(Item, ..)` per target over an `Item` enum
that a target refuses at run time (`latex::write(Item::Net(..))`
compiles), weaker than F36's `Drawable`. *Graft*: 9.2, the Rocq design:
`Kernel::Auto` keeping NanoYalla's bytes (R139, R159), `LIBRARY` and
`FORMAT` constants and the certificate's first line tying writer to
library (R12), `node (M : Type)` with constructors in `Node::TAGS` order
(R118, R244), positions as certificate data checked locally (R54),
refutation certificates (R153), reserved names per kernel (R155).

**C (3).** `Drawable` and the id contract; Rocq in three lines.

### §10 How each later step enters

**A (4).** Every step 29–38 in a paragraph; 10.1 lists first-order logic
and what stays untouched for the propositional case (D17).

**B (4).** Each step; the `cut` module's signatures for 34 (R32, R100,
R15); 38's checker with the eigenvariable set flowing up (R126).

**C (5).** 35–38 in depth: `Symmetries` (R103), `Linker<C:
Correctness>` (R105), `NetPrunes` (R143), the planar order derived from
D1's lowering with a fallback (R56, provisional, for 36's panel), the
inverse method's conversion to `Node`s (R127), the first-order prune
table (R107), the trail's exact places in the focused engine, and the
loss of answers across `⊗` premises (impact-quantifiers finding 3).
*Wrong*: settles choices the steps own (`Engine::Essential`, cyclic as
the net engine's mode); acceptable as provisional.

### §11 The spike

**A (3).** Three layers, measured after each. *Wrong*: the framed zone
lives in a test only, so the second monomorphisation, the one cost
fo-linear §5.5 says genericity does not settle, is never in the binary;
compares with `after-bias.csv` (F181: the oracle moved); no base built the
same day in the same environment; no instruction-count gate tighter
than the ratchet.

**B (4).** Three cumulative parts with a fallback for each (part 1
behind a literal-access trait, part 2 a framed checker pass beside the
ground one, part 3 a duplicated module), and the checker measured on
the `check-*` journeys. *Wrong*: no second instance; same oracle issue.

**C (5).** See section 5 below.

### §12 Findings answered

**A (5)**: a table by finding and the register by section. **B (4)**,
**C (4)**: lists; C adds the register entries placed with later steps.
B's and A's claims for H9/H10 overstate (Q2).

### §13 Decisions for the author

All three (4): each states what the other answer of C1–C3, T1–T7, HD1–HD5
would change, and adds its own (A1–A11, B1–B11, C 1–8). A's A1 (one
level), A6 (the stop's form) and A10 (the sides) and B's B2–B4 are the
consequential ones.

## 3. Register coverage

A's register map cites all 174 entries of the eight sections; B leaves
59 uncited, C 59 (counted by script, ranges expanded). Citation is not
treatment. **Met or placed by no draft in substance** (cited only by
A's range lines):

- R102 (the Danos–Regnier test at each box depth reachable through
  `is_correct`, allocation-free, CSR with variable arity), R165 (the box
  drawing needs a layout order that permutes roots), R167 (drawing cut
  links and each elimination step): step 33/34 internals; the synthesis
  should place each in 10's step lines.
- R115 (the focused search handing a goal to the net engine at the
  recursion limit): no draft says where in the front door or `Decide`.
- R154 (a classical "not valid" certificate for an ordinary sequent):
  C2's consumer; place it with 31's refuter.
- R171 (Typst deep proofs): an export detail, place with the Typst
  writer.

**Entries where the drafts disagree, with the right one**:

| entry | A | B | C | right |
|---|---|---|---|---|
| R6 (old fixtures load) | old names read without `version` | not read | aliases | A, C |
| R10 / R6 (Mode gains a key) | a name; old object read | object, unknown key refused | object, `default`, unknown ignored | A or B; C misreads a cyclic mode |
| R47 (one scheme with and without threads) | `Schedule` | absent | `Schedule` | A, C |
| R54, H9, H10 | `u32` sides, inconsistent | goal last | `Option<u32>` sides, no symmetric reading | C (Q2) |
| R59 (builder) | at 28, with sides | at 28 | at 34 | A (R59 allows C's) |
| R77, R79, D-6 (net vertex, descriptor) | `Criterion`, vertex deferred | `VertexId` + `Criterion`, `"mix"` kept | `Mode`, `OccId` links | B |
| R79 / F10 (net JSON unchanged) | `criterion` key | `mix` kept | `mode` key | B |
| R135, R138 (refused vs unknown) | six classes | five kinds | three classes | A, B |
| R148 (time limit) | `Clock` data with a wire form | `Duration` constant | `Duration` constant | all meet; A also gives the web its key |
| R243 (progress to the stop; closures kept?) | closures dropped | kept, `WithProgress` | kept, `WithProgress` | A (probe), or B/C with a bounded constructor |
| R244 (member) | offset | table per FO owner | offset | A, C |
| R247 (numbers beyond 2⁵³) | 2⁵³ | `u64::MAX` | `u64::MAX` | taste; follow R247 unless a read-back form carries such a count |
| R20 / C2 (refuter table) | at 31 | at 31 | at 28 | A, B follow C2's text; C's reason is fair |

## 4. The disagreements that matter

1. **The member type.** All `Member(u32)`, one integer on the wire, in
   `Inference.sequent`, `Dyadic`, `Interactive::goal`, `prove_goal`,
   `engine_for`. *Node's operands*: B and C members everywhere, A defers.
   *Meaning in a first-order owner*: A and C by offset (below
   `forest.len()` the occurrence, past it an instance), B all table
   entries. **Verdict: members in every `Node` operand, by offset (C's
   P2)**: a stored id below `forest.len()` means the occurrence in every
   owner, the ground case needs no table, and interning keeps one member
   per instance because an occurrence under no binder has only the empty
   frame.
2. **Atoms and terms.** All reserve `Term::{Pred, DualPred}(Atom,
   ArgsId)`, `Forall`, `Exists`, `Atom` a predicate symbol, the fragment
   bit set by an argument. *Kind of a predicate literal*: A and B a kind
   of its own; C `Kind::Atom`. **Verdict: a kind of its own** at least,
   so every exhaustive match is a compile error where C's choice lets the
   eight pairing sites read `p(a)` as `p` silently; better still the
   alternative of Q1.
3. **The root order.** All written order now (C1 A). Agreement.
4. **The written sides.** A `u32`, B none (goal last), C `Option<u32>`.
   **Verdict: C**, with its JSON form fixed (absent only for `None`); Q2.
5. **The error family.** A: one flat `Error`, `Class` (6), `code() ->
   &'static str`, `setting()`. B: specific types kept and converting,
   `ErrorKind` (5), `CheckError::{Invalid, Refused}`. C: specific types,
   `ErrorClass` (3), `ErrorCode` enum. **Verdict: A's classes and
   `setting()` and wire form, B's `CheckError` split** (and its rule that
   every type that can be refused carries the refusal as a variant of its
   own); flat against nested is taste. C's three classes lose the
   distinction R135 and R138 need.
6. **The bounds value and its place.** A: `Bounds`, a separate argument
   of every long call, including `prove_within`. B, C: `Limits`, a field
   of `search::Options` and an argument elsewhere. Equally defensible; B/C
   save an argument and let R73's per-problem overrides carry bounds; A
   keeps one rule ("every long call takes `&Bounds`"). Not counted. The
   recursion bound belongs in it (A, C), since R45's stack sizing is a
   bound.
7. **The stop.** A: `impl FnMut(Progress) -> bool`, no trait. B, C: `trait
   Stop`, blanket for `FnMut() -> bool`, `WithProgress`. **Verdict: A**:
   one form, inference works (probe), R243 lets step 28 drop the plain
   closure (D18). If the synthesis keeps the trait, `WithProgress` needs
   `fn with_progress<F: FnMut(Progress) -> bool>(f: F) ->
   WithProgress<F>`.
8. **The options' wire forms.** A: `"auto"` for automatic, `null` only for
   no bound. B, C: `null` for both. **Verdict: A** (F79).
9. **The version policy.** A: one global level. B: per form, lowest. C:
   per form, no bump rule. **Verdict: A**, for the outcome-read-as-proof
   case and because one number is what a web client stores; B's
   per-form rule is the fallback, C's is incomplete.
10. **`Mode` on the wire.** A a name, B an object refusing unknown keys, C
    an object ignoring them. **Verdict: A or B; not C.**
11. **`Node` and the checker.** Only B specifies the checker (the rule
    table, the intuitionistic conditions, what lies outside the verified
    function) and the variant policy (a new `Node` is a bump, a tag, a
    `Fault` arm, an oracle arm, a `Rule`, a Rocq constructor, R118's
    test). **Verdict: B.**
12. **A proof's conclusion.** A, C: `Outcome.goal`, a goal proof refused
    by the proof reader. B: `Proof { goal, mode }`, checked against its
    own conclusion. **Verdict: B** (F89, F23, and a checkable goal proof
    for `close_with`), keeping A's `GoalProof` refusal where a file must
    prove the sequent.
13. **`Verdict::Unprovable`'s payload.** A, C: `Refutation`. B:
    `Box<Disproof>` holding sequent, goal and mode. `Verdict` is closed
    (D9), so this is decided now or never without a break. **Verdict: B**,
    narrowly: R124's checker and R153's writer need the sequent and mode
    with the refutation, and the cost is one arena clone per unprovable
    outcome.
14. **The net's vertices and criterion.** **Verdict: B** (`VertexId`
    distinct in the API with `VertexId(i) == OccId(i)` in MLL, `Criterion`
    flattened so `"mix"` stays; D-6, R77, R79, F10).
15. **`Rule`.** B splits it (F60: 15 one-sided rules and `Open`, `Named {
    rule, side }`, names unchanged on the wire); A and C keep 34 variants.
    **Verdict: B**: 34's and 38's bumps add one and two variants, not
    two and six, and consumers lose their `unreachable!` arms.
16. **Exports.** `Drawable` (A, C) against `Item` (B). **Verdict:
    `Drawable`**, plus B's Rocq design.
17. **`Step`'s split.** A `left: Vec<usize>`; B, C `Split::{None, Left}`
    with 36's `At`. **Verdict: B/C**, a closed set of split kinds is what
    36 extends.
18. **The engine interface.** All crate-private `Decide`. **Verdict: C's
    contract and registration list**; B's `Goal<'a>` is a reasonable
    addition, taste.
19. **The race.** **Verdict: A's signature** (threads, `add_pool`, `Fn +
    Sync` stop), C's per-worker depths for H18 beside it.
20. **`Settings`** (A only). Not speculative: D15's "third wrapper gets
    the same surface", R141 and R183 call for it; `Settings::set` may wait
    if `serde_json` in the library is unwanted.

## 5. The spike

**C's specification measures D17 best.** It names the two questions
(Q1 the data model, Q2 the zone parameter), builds three milestones each
measured against the unmodified tree built the same day in the same
environment (M1 the data model with `Member` in `Node` and the checker's
per-proof branch; M2 the focused engine generic over `Zone` at `Ground`
with the trail's `mark`/`undo` called at the real places; M3 a second
instance, `Framed`, compiled into the binary and reached from the
dispatch only for the bit), and gates them in order of trust: G1 the
target set's counters exact against the base and the oracle F181 names;
G2 instruction counts (search journeys +1.0 % each, their sum +0.5 %);
G3 pinned time (geometric mean ≤ 1.02, no row above 1.05 unless its base
spreads as much). Its decision table maps each outcome to what `api.md`
records, including "M2 passes, M3 fails" (the second instance's code, not
genericity), which neither A nor B can see.

**Add**: (1) A's last sentence: the change of every poll site to the
progress stop is its own measured commit (it touches every engine's hot
poll); (2) B's per-part fallback for the checker (a framed pass beside
the ground one) as the M1 fallback for the checker's branch; (3) if Q1's
alternative is adopted, M1 changes to `Term::{Forall, Exists}` only and a
structured atom table, which shrinks its matches; (4) record `.text` per
milestone and the focused engine's share (C reports it; keep it
ungated). **Drop**: G3's repeated journey medians if the target rows
already gate time, to keep the spike within one night's budget; C's
"cores 2 and 3" is fine only if nothing else timed runs (the machine is
shared by day).

## 6. The ranking

1. **A (56).** The surface a fix session implements at step 28 (the
   table of changes, errors, bounds, options, wire forms, batch,
   interactive for a client) is the most complete and the most
   implementable as written, and its choices on the contested public
   points (stop, `"auto"` vs `null`, one wire level, `Mode` by name,
   old names read, six error classes) are the right ones. Its faults
   are concentrated and graftable: the written sides, H9, the deferred
   `Node` operands and vertex type, and a spike that leaves the second
   instance out.
2. **C (52).** The best engine interface, the best later-step entries
   and the best spike, and the only correct H9/H10 rule; weaker on the
   wire policy (no bump rule, `Mode` misread), errors (three classes) and
   nets (`Mode` as criterion, `OccId` links), and it lets predicate
   literals share `Kind::Atom`.
3. **B (49).** The trusted core (proof term, checker spec, `CheckError`
   split, `Proof { goal, mode }`, `Disproof`, `VertexId`, `Named`, Rocq)
   is the best of the three and must be grafted whole; as a base it
   misses R47, leaves H9 unfixed, drops the old names (R6), and keeps
   F79's `null` ambiguity.

**Base: A.** Grafts, in order of value:

1. **C's written sides** (3.1, 3.5): `antecedents: Option<u32>`, exactly
   one written succedent under `-i`, and with the sides known the left
   factor is the antecedent (no symmetric reading); JSON writes the key
   whenever it is `Some`. Replaces A's 3.1 and 3.6.
2. **B's trusted core**: 3.5 (`Node` with `Member` operands, `TAGS`, the
   variant policy, `Proof { goal, mode }`), 3.6 (the checker's
   specification, `CheckError::{Invalid, Refused}`, `Fault`), and B4/P2
   into A's §4.
3. **C's spike** (§11), with the additions of section 5 above.
4. **C's §8** (the pipeline, `Decide`'s contract, the registration list,
   `NotTaken`, the per-engine statistics table) and **C's H18 design**
   (`Runtime::depths`), into A's §8 and 5.4.
5. **B's nets** (3.9): `VertexId`, `Criterion` flattened into `"mix"`,
   `from_proof` exhaustive and bounded.
6. **B's Rocq** (9.2) into A's §9.
7. **B's `Rule`/`Named`** (3.7, F60) and B's/C's `Split`.
8. **C's first-order tables and offset principle** (P2, 3.9, 10.2–10.5)
   into A's 10.1, with the kind question settled by Q1.
9. **B's `Disproof`** as `Unprovable`'s payload (B3).
10. C's `EngineRefused`/`UnknownName` and P3's lint; B's validated
    `Identifier`/`Length` option types.

## 7. Question 1: atoms as interned atomic formulas

**The alternative.** `Atom` indexes an interned atomic formula (a
predicate symbol with its argument list, hash-consed; nullary for every
propositional atom). `Term` keeps `Atom`/`DualAtom` as its only literals
and gains only `Forall`/`Exists`; only a binder sets the quantifier bit.
Under a binder an atom's arguments contain de Bruijn indices, and the
first-order engine and checker instantiate it through the member's
frame. impact-quantifiers §1 item 1 already names it as the other way out
("or else ground atomic formulas must be interned as atoms of their own").

**Soundness: where each can go wrong silently.**
- *Drafts' choice.* The eight places that pair literals by atom (the
  checker's `Ax` at `check.rs:1167`, the oracle, `nets::dual`, the
  additive path, the net engine's partner lists, the focused engine's
  `initial`/`dual_in`/`dual_from`, the Horn engine's places) all pair `p`
  with `~p` whatever the arguments. Each is safe only if the bit reaches
  it: an engine's `admits`, the checker's refusal of a forest with
  arguments, `dual_literals` refusing them. One missed site is a wrong
  "provable", with no compile error, and under C's `Kind::Atom` for `Pred`
  not even a kind tells them apart. Count refutations and classes keyed by
  atom are sound but say less.
- *Alternative.* Those eight places compare atom ids, which for ground
  atoms is exactly syntactic equality of atomic formulas: a ground
  first-order sequent is decided and checked correctly by every engine as
  it is, counts and Horn places included (stronger, even). Open atoms
  exist only under a binder, whose `Kind::Forall`/`Exists` are new
  variants, so every exhaustive match (R248) fails to compile until it
  decides; the remaining hazard is a site that compares two open atoms by
  id (`p(#0)` under two different binders is one id), which only
  first-order code meets, behind the bit. The surface that can go silently
  wrong shrinks from eight propositional sites to the first-order
  engine's own instance comparison.

**D17.** Fewer `Term` variants (two, not four), no new literal kind in
the engines' hot `match`es, literal lists still indexed by `2·atom +
sign`. The atom table grows a symbol and an argument list per entry,
which no hot path reads. Free by construction, more clearly than the
drafts'.

**The wire form.** Propositional `atoms: ["A", "B"]` and the tags `V`/`D`
are unchanged. A first-order atom table must not ride on the old key: if
the arguments were a parallel key (`atom_args`), a 0.1.0 reader would
ignore it and merge `p(a)` and `p(b)` by name ("a name is an atom",
`core-sequents.md`), a silent misreading. So a structured atom table is a
new level (A's policy) or version (B's), refused by an old reader before
it reads; A's level rule covers it as written ("a key whose presence
changes what other keys mean"). The drafts' `P`/`N` tags fail on an old
reader by tag, without depending on the version gate.

**What the first-order engine needs.** Candidates by predicate symbol:
under the alternative the forest's literal lists are by atomic formula,
so the engine needs a symbol → atoms index (a CSR built once per forest
when the bit is set; no propositional cost). Unification compares
`(atom, frame)` pairs: same symbol, arguments unified through the
frames; identical in both designs.

**Printing.** Printers print atom names from the table. Under the
alternative a name becomes `symbol(args)`, and an open atom needs the
binder names of its context; the walk carries them either way. The
public `Sequent::atom_name(a) -> &str`, `atom_names()`, `Refutation::
Unbalanced`'s name and the HD5 identifier check assume a name is a
string: step 28 should make the accessors say "the predicate symbol's
name" (and add an `atom_args` accessor at 38) so nothing breaks.

**What step 38 would find harder.** The symbol index above; merging
atoms by `(symbol, args)` in `optimize`, `add` and deserialization;
fragment names (a ground first-order sequent is named `MLL`, which is
true of its logic); the counts' prune must drop to per-symbol only for
open atoms. What it would find easier: the first commit of 38 can admit
ground first-order input everywhere instead of refusing it, and the
checker needs instance comparison only for framed members.

**What I would choose: the alternative**, with three things fixed at
step 28: `Atom` documented as "an atomic formula, nullary until step 38"
(R62's "predicate symbol with an arity" becomes the atom's symbol), the
atom-name accessors defined as the symbol's name, and the wire rule that
a structured atom table raises the level. The spike's M1 then reserves
two `Term` variants, not four. The deciding reason is soundness: it turns
the silent pairing of eight propositional sites into compile errors at
the binder kinds.

## 8. Question 2: H9 and H10

The witnesses (from the findings file): **H9**, `(A -o bot) -o bot |- A`
under `-i`, lowered to `⊢ (~A ⅋ ⊥) ⊗ 1, A`; the reading cannot read the
hypothesis with its left factor as antecedent (`A ⊸ ⊥` needs a ⊥ ILL
lacks), so it takes the symmetric reading `1 ⊸ (A ⊗ 1)` and proves
`1 ⊸ (A ⊗ 1) ⊢ A`. **H10**, `|- a, top` under `-i`, proved as `0 ⊢ a`;
the same for an LLTP file with conjectures `a` and `top`. The rule
(`core-forest.md`, "The choices, made deterministically") takes as goal
the root that can only be output, else the last root that can be output,
and inside an implication the left factor when that works, the right one
otherwise.

**A** (`antecedents: u32`, 0 meaning one-sided; one written succedent;
`MovedBottom`):
- *H10*: `|- a, top` has 0 antecedents, which A calls one-sided input and
  reads "by shape only", that is as today: **not refused** by 3.1/3.6 as
  written, refused only under A10's reading. Internally inconsistent.
- *H9*: refused by `MovedBottom` if that rule is made precise (it is not:
  nothing says how the reading knows a ⊥ was "moved"); the symmetric
  reading stays for every other case.
- *One-sided text with the goal not last* (`|- B, ~A`): read by shape, as
  today. *JSON*: no key and 0 are the same, so old and new JSON behave
  alike, whichever way the inconsistency is resolved. *LLTP*: assembled
  as text, so the sides are kept (R74).

**B** (no sides; the goal is the last root, every other root must read
as input):
- *H10*: `|- a, top` → goal `top`, and `a` cannot be input: **refused**.
  But `|- top, a` → goal `a`, `top` read as the hypothesis `0`: proved as
  `0 ⊢ a`, H10's fault with the roots swapped. Two written succedents
  are not refused "whatever they are built from" (H10's fix).
- *H9*: the goal `A` is last and the hypothesis reads as input through the
  symmetric reading exactly as today: **not refused**. B does not remove
  the symmetric reading.
- *One-sided text with the goal not last*: `|- B, ~A` is refused (today
  `A ⊢ B`). *JSON*: a stored sequent from before 0.1.0 has sorted roots,
  so under `-i` its goal may not be last: refused, or, for `⊤`/`0`-built
  roots, read with another goal. *LLTP*: the conjecture is last: fine.

**C** (`antecedents: Option<u32>`, `Some` for every text and LLTP input;
exactly one root right of `⊢`, else `Succedents(n)`; with the sides known
the antecedent of an implication is the factor the lowering put there):
- *H10*: `|- a, top` is `Some(0)` with two succedents: **refused**; so is
  `|- top, a`.
- *H9*: with the sides known the symmetric reading is not tried; the left
  factor reading fails on the ⊥ as `bot |- A` does: **refused**. This
  removes the symmetric reading for every sequent whose sides are known,
  which is what H9 rests on.
- *One-sided text with several roots* (`|- ~A, B`): refused, since two
  roots stand right of `⊢` (today `A ⊢ B`). C does not say so; it is
  what H10's fix asks ("two written succedents are refused") and README's
  only one-sided `-i` example (`|- A par B`, one root) is unaffected. An
  explicit `B par ~A` in a two-sided `-i` input is refused too (today read
  as `A ⊸ B`), matching the reading's own error text ("⅋ only as A ⊸ B,
  that is ~A ⅋ B"). *JSON*: a sequent without the key (`None`) keeps
  today's reading, so H9 and H10 survive for stored JSON; new JSON must
  write `"antecedents": 0` for `Some(0)`, which C's 7 gets wrong. *LLTP*:
  sides kept through the parser.

**Verdict**: only C's rule refuses both witnesses and removes the
symmetric reading; it needs its JSON form fixed and its behaviour change
for `|- Γ` under `-i` stated in the commit and README. A's `u32` should
become C's `Option<u32>`; B's goal-last rule should not be used.

## 9. What I would want run (not run)

- `linlog prove -i -q '|- top, a'` and `'(A -o bot) -o bot |- A'` on the
  current binary, to pin the two mirrors of H9/H10 that B's rule leaves
  as tests for the fix session.
- A size probe of `Error` after boxing the large payloads (A's 64-byte
  assertion) once the variant list is final.
