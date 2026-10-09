# Judgement of the three drafts of `plan/notes/api.md` (judge: Fable 5.1)

Scored against the register (the sections Wire forms, Bounds and stops,
Data model, Engine interface, Proof term and checker, Errors, Options,
Export; C1 to C3), the rubric, the audit's lists (28-audit.md: the
decision list, "For the design", "From the review": H9, H10, H18, HD1 to
HD5), the plan's decisions (D1, D5 to D9, D13, D15 to D19, D23), the
research constraints D-1 to D-12, `impact-quantifiers.md` §1 and §3,
`fo-linear.md` §4 and §5, and the code of 2026-10-09. What I checked in
the code: `sequents/mod.rs` (`optimize_roots` sorts, line 205;
`optimize_terms` keeps first-occurrence order, 218–280), `term.rs`
(`Var`/`DualVar`, 12 variants), `occurrences/mod.rs` (`Forest` fields,
`MOST`, `Sign`), `reading.rs` 205–290 (the goal choice and the symmetric
implication reading), `parse/mod.rs` (every text sequent has `|-`; 412
`optimize`), `proofs/mod.rs` (`Node` over `OccId`, 16-byte assertion,
`check_within(mode, Option<u64>)`), `check.rs` 440–540 (`CheckError`,
`Problem::Memory` beside the faults), `interactive.rs` (`Refusal`,
`apply(goal, position, rule, left)`, `close_with` through `of_goal`),
`search/mod.rs` (`Stop` enum, `Task`, `Answer`, `Decide`, `dispatch`
with `expect`, `Options` private fields, `Outcome`, `Verdict`,
`Refutation`, `Reason`, `Statistics`, `stack_size`), `errors/mod.rs`
(35 variants, positional pairs), `fragment.rs` (five bits, `Mode` pub
bools), `nets/mod.rs` (`new(forest, mix)`, `NetError` over `OccId`),
`export/*` (per-type wrappers, `rocq::Options`), `batch.rs`, the
serializers (keys `ids`, `var_dict`, `proof`, `mix`, `once`), `lltp.rs`
(assembles `axioms |- conjecture`), `cli/tests/lock.rs` and `cli.rs`
(every `-i` call), `core/tests/lock.rs`. Probe (cores 6–8, capped,
`target/design-judge-f`, scratch crate on the locked serde_json 1.0.149):
`serde_json::to_string(&u64::MAX)` writes `18446744073709551615` and
`from_str::<u64>` reads it back exactly; as a `Value` it `is_u64()`.

Scores: 5 a fix session implements it as written and no later step
breaks it; 3 right direction with gaps; 1 wrong or missing. Taste is not
counted; I say where two choices are equally defensible.

## Section by section

### 1. Principles

| | A | B | C |
|---|---|---|---|
| score | 4 | 5 | 4 |

**A.** P1 to P10, each with its reason; P5 (what only a clock can
enforce is data too, `Settings::clock`) and P7 (one wire level) are the
web angle's contribution. *Wrong:* P8 rests on "u64::MAX written by
serde_json reads back as a double that no longer parses as a u64", which
the probe refutes for Rust (true of `JSON.parse` only); R247's
recommended answer is "numbers as numbers, u64::MAX meaning saturated,
nothing changes on the wire", and A's 2⁵³ sentinel is a third option the
register does not list (a decision for the author, not a principle).
P6 closes `Compact`, `Form`, `Cores`, `pdf::Date` "on purpose" where F1
lists them to mark: defensible, say why on each.

**B.** P1 (the trusted core is small, independent and specified, each
checker a total function of (term, forest, mode), refusals outside it)
and P2 (every error type that can be refused has a `Refused` variant no
fault shares) are the two principles that make S4 and S18 structural
rather than a discipline; P6 (the lowest version that represents the
value) is D-3 made precise. *Graft:* P1 and P2 as written.

**C.** P2 (one id space per kind, extended by offset: members, vertices,
cut trees, search terms) is the crisp statement of D17 for the data; P7
(engines and refuters are the crate's own plug-ins, `Decide` and
`Refute` crate-private) and P3's lint (`wildcard_enum_match_arm` denied
per module, D-11) are right. *Wrong:* nothing; P8 says "wire forms carry
a version and grow by keys" without saying when the version moves (see
7).

### 2. The public surface after step 28

| | A | B | C |
|---|---|---|---|
| score | 5 | 4 | 4 |

**A.** The module tree (`bounds`, `settings`, `wire` new; `Notation`
moved out of `export`, F82), the exact `lib.rs` re-export block, what
leaves the root, and a before→after table of 45 rows keyed by the old
name with the finding ids: the one a fix session searches. *Graft:* 2.4
whole, as the index of the synthesis. *Wrong:* `Forest::from_owned`
"unverified" (harmless); `svg::net` → `Error` loses `TooLarge` as a
return type, consistent with A's flattening (4).

**B.** Tree, re-exports, the full `#[non_exhaustive]` list with the
closed ones (B1), and a table of 30 rows that merges items per row (less
searchable, complete). Marks `Compact`, `Sides`, `Form` open where A
closes them: taste. Removes `Interactive::inferences()` (F69) and makes
`optimize` infallible (F51): right.

**C.** Tree with the reserved `sequents::fo`, a 31-row table that names
the JSON renames and "the old names read as aliases" (T1), `Sign::{Plain,
Dual}` (taste against A's and B's `Atom`/`Dual`). `errors` stays a
private module with `Error`, `ErrorCode` re-exported: fine. Keeps
`Error::Unchecked` for a graft (R138's wording).

### 3. The data model

| | A | B | C |
|---|---|---|---|
| score | 3 | 4 | 4 |

**A.** Strong where a front end meets it: the checked `Builder` with
`left`/`right` (R59 now, the sides built in), `Member` as every list's
element with "a member *is* its occurrence propositionally, one integer
on the wire", `Mode` with `NAMES`/`FromStr`/`validate`, `ViewOptions {
compact, sides }` (F10), `GoalId`, `Step`, `Applicable`/`Needs`,
`GoalView`/`FormulaView` (R91, R93, the web's goal view), `Criterion`,
`ordinary::decide` and `ordinary::Outcome` (R113). *Wrong:* (i)
`antecedents: u32` with "0: one-sided input" cannot carry what the
design needs: in the text syntax every sequent has `|-`
(`parse/mod.rs`, F29), so `|- a, top` (H10's witness) *is* the
zero-antecedent case; A then says both "refuses two written succedents"
and "by shape only for one-sided input", which contradict for that
input, and a JSON sequent without the key reads as zero, so a file
written from `|- a, top` reads back under a different rule than the text
(no round trip of the reading). The type must be `Option<u32>` (C). (ii)
`Node`, `Proof`'s fields, the checker and the vertex are deferred to B
and C by name, so this section is not implementable alone.

**B.** The core as a fix session needs it: `Term`/`Kind` with the
reserved variants and the 12-byte assertion; the written order with
"arena indices follow first occurrence along the roots (unverified after
hash-consing)", which `optimize_terms` 251–268 confirms (new indices in
index order); the public walk shaped for terms (`Between(TermId, u32)`,
D-10); `SequentBuilder`; the forest's three promises with `cut_pairs`,
`dual`, `is_conclusion`, `dual_literals`, `PartialEq` (D-2, F58); the
member with the rejected alternatives and the owners' `occurrence(m)`
and `formula(m)`, which A's and C's `Member::occurrence(&Forest) ->
Option<OccId>` cannot give above `forest.len()`; `Node` over `Member`
with `TAGS` (R118), the variant policy, `Proof { goal, mode }` (B2: F89
and R7 at the type, `check` against its own conclusion), the invariants;
the checker specified rule by rule (3.6, the text R117 asks for, with
the intuitionistic clauses and what lies outside the verified function);
`Rule` split with `Named { rule, side }` (F60); `Inference` and
`ViewError { Invalid, Refused { refusal, size, firm } }` (F65);
interactive proving with `Step { position, rule: Named, split: Split }`,
`StepError`, `cut` as a method at 34; `VertexId`, `Criterion::{of,
admits}` (D-6, R79, R133); `Disproof` with `check` recomputing the
conditions (R70, R71, R124). *Graft:* 3.1 to 3.3 and 3.5 to 3.11 whole.
*Wrong:* 3.4's reading rule, "the goal is the last root, which must read
as output, every other root input, never reread with another goal":
it refuses H10 but not H9 (question 2 below), and it refuses one-sided
`-i` input whose goal is not written last and old JSON files whose
sorted roots put the goal first (`A, A -o B |- A`: the goal `A` shares a
term with the hypothesis, so the sorted roots are `[0, 1, 3]` and the
last root is `A ⊗ ~B`). Replace by C's rule.

**C.** `Sequent` with `antecedents: Option<u32>` (the right type) and
the rule that the sides settle the goal and the implication's factor
(10.3's reasoning and 3.1's), `Term`/`Kind` with the predicates under
`Kind::Atom` (so every kind-based literal test stays right), the forest's
three promises with `are_duals` and "literal lists are candidates, not
partners", `Member` with `new`/`get`/`index`, `Mode` with private fields
and getters (taste against A's and B's public read-only fields), the
`QUANTIFIERS` bit, `Reading` positions stored in `Interactive`
(`Box<[Position]>`, F66), `Node` reserved variants, `Proof::mode` and
`with_instances`, `Inference` with accessors (R61, R66), `Step`/`Split`,
and 3.9's first-order tables (`FoTerm`, `ArgsId`, `FrameId`,
`Instances`), the most concrete reservation for 38. *Wrong:* (i)
`ProofStructure::new(forest, Mode)` in place of a criterion: R79 and
D-6 ask for a descriptor that 33, 35 and 36 extend in place; C's
essential criterion becomes a separate `is_essential` method, so the
"criterion" is no longer one value, and `from_links(forest, Mode,
links: &[(OccId, OccId)])` keeps `OccId` pairs while 3.7 says 33's
instance vertices "continue the numbering as a `VertexId`": a vertex
past `forest.len()` cannot be passed through that signature without a
break at 33. B's `VertexId` and `Criterion` are right. (ii) The builder
is deferred to 34 and 38 ("waits for its first callers") while R59 names
the web client and wrappers, and step 32 comes first. (iii) Whether the
parser sets `Some(0)` for `|- …` is left unclear ("absent for a sequent
written one-sided" in 7's example): it must be `Some(0)`, written, or
H10 is not refused.

### 4. Errors

| | A | B | C |
|---|---|---|---|
| score | 4 | 5 | 4 |

**A.** One flat `Error` (`ViewError`, `WriteError`, `RenderError`,
`svg::TooLarge` folded in), `Class` with six values (`Invalid`, `Limit`,
`Stopped`, `Unsupported`, `Failed`, `Defect`), `code()`, `setting()` (the
key that lifts a bound: AIP-193's "how to lift it", R129, R137),
`describe(&forest)`, `CODES`, a wire form with `details` and a
`ParseError` span in bytes, UTF-16 units and line/column (R129), a
variant table of 30 rows, codes reserved per later step. *Graft:*
`setting()`, the six classes (B's five put `ThreadPool` under "Refused",
which it is not), the code table, the UTF-16 span. *Wrong:* the S4
guarantee is procedural (`From<CheckError>` sends a refused check to
`MemoryLimit`, "a `debug_assert!`"), while `CheckError` itself keeps
`Problem::Memory` beside the faults (check.rs 465–520 today), so a
caller of `Proof::check` can still match "invalid" and see a refusal.

**B.** `ErrorKind` with `is_refusal()`, the full `Error` grouped by kind,
every specific type kept where a caller matches its data and each given
a `Refused(Refusal)` variant no fault shares (`CheckError::{Invalid,
Refused}`, `ViewError`, `WriteError`, `RenderError`, `NetError`,
`RefutationError`): S4 and S18 by construction, and `Rejected` a
`Defect`. One `Described<'a, E>` and one sealed writer per error (F48).
The wire form with `details` nested per kind. *Wrong:* no `setting`-like
hint; `ThreadPool` under the refusal group (a `Failed` kind is missing);
`Error::Parse(Vec<ParseError>)` keeps the vector where A's
`Box<ParseError>` reflects that the parser yields one error (F29's
evidence).

**C.** `ErrorCode` (an enum, snake_case on the wire) and `ErrorClass {
Input, Refusal, Defect }`; `IndexOutOfBounds { space, index, len }` and
`NotTopological` consolidate the six positional errors (F31, neat);
`UnknownName { what, name, known }` for every `FromStr` (R241, F76);
`EngineRefused { engine, because: NotTaken }` (8.6). *Wrong:* three
classes lump a stop, a bound and "unsupported" together; a front end
shows a bound reached ("raise `--memory-limit`") differently from an
unsupported fragment, and the harness's refused/unknown distinction
(R135, R213) needs the split; the mapping table is the shortest and
leaves `ShapeError`, `UnknownRule` and the ordinary errors to a line.

### 5. Bounds and stops

| | A | B | C |
|---|---|---|---|
| score | 5 | 4 | 5 |

**A.** `Bounds { memory_bytes, occurrences, derivation_bytes, work,
recursion_depth }` with `DEFAULT`, `BROWSER` (each number reasoned,
"provisional until step 32 measures it"), `UNBOUNDED`, `stack_bytes()`,
`recursion_depth_for_stack()` (R45); "a bound of 2⁵³ or more is no
bound, written `null`"; what `memory_bytes` counts stated (R29); one
account per call and per race (R18, F104); `Progress { work, held_bytes,
phase }`; the unit of work per engine and the poll cadence per call
(checker every 4 096 nodes, R23); the web client's deadline pattern
(F136); AIP-151 as the worker protocol; `Bounds::work` and `Schedule {
Auto, Turns }` (R21, R47); H18 with the test; `ReadWithin` sealed trait,
`from_json_within`, `to_json` (R27, F27); the table of every long call
(R22). *Graft:* `BROWSER`'s reasoning, `Progress::phase`, 5.5, 5.6.
*Wrong:* the stop as `impl FnMut(Progress) -> bool` and no other form
(A6) changes every existing closure `|| false` to `|_| false` at every
call site in core, cli, bench and the tests; D18 allows it and A gives
the inference argument (a closure passed to a bound infers its argument
type, a `WithProgress(|p| …)` constructor does not), so this is a
defensible taste choice, but the register's "whether a closure that
ignores the progress is still accepted is decided at step 28" is
answered "no" by A alone. `from_json_within`/`to_json`/`Settings::set`
put `serde_json` behind `serialize` in core (A3): a dependency change
(`cargo deny`), fine, but say so.

**B.** `Limits { memory_bytes, occurrences, output_bytes, work }` (the
recursion limit stays `Options::recursion_limit`, with
`recursion_limit_for(stack_bytes)`), `Refusal` as the one enum every
refusing type shares, `Progress { work, nodes, bytes }` (no phase), the
`Stop` trait with the blanket impl for `FnMut() -> bool`, `WithProgress`
and `&mut dyn Stop` (the coherence claim is sound: `FnMut` is a
fundamental trait, so a local type that does not implement it may
implement `Stop`, downstream included), `forms::Within` as a
`DeserializeSeed`, the long-call table, one account for the race, H18
per thread. *Gaps:* no browser numbers, no poll cadence per call, no
schedule option (R47, F110: B says nothing of `Schedule`).

**C.** `Limits { memory, occurrences, recursion, work, derivation }`
with JSON names carrying units (`memory_bytes`), `BROWSER`,
`stack_size()`/`recursion_for_stack()`; the `Stop` trait, `WithProgress`,
`never()`; `Progress { work, done, held, phase, item }` (`item` for a
goal of `close_all` or a problem of a batch: R92 served without a second
stop) and `Phase`; the unit of work per engine with today's exact
counts; progress counted apart from the slices (R243's one subtle
requirement, named with the forced chain's poll); `Schedule { Auto,
Threads, Turns }` (F110, R47); the memory-account table per call (5.3,
R35, R36, R29); H18 the most concrete (`Runtime::depths` per rayon
thread index, a task starts from the thread's depth); the long-call
table with "a test per row fires the stop at its first poll" (R22's
Met-when). *Graft:* `Progress::item` and `done`, 5.3, `Schedule::Threads`,
H18's mechanism. *Wrong:* nothing material; A's `derivation_bytes` and
C's `derivation` are one thing under two names (F63 asks for one).

### 6. Options

| | A | B | C |
|---|---|---|---|
| score | 5 | 4 | 4 |

**A.** One convention (public fields under `#[non_exhaustive]`, named
`DEFAULT_*`, `#[must_use] with_*`, `serde(default, deny_unknown_fields)`,
no `version` on an options form, `null` = no bound, `"auto"` for an
automatic choice: F61, F76 to F79, F64); `search::Options` as a table
(field, type, default, JSON, which engine reads it, flag) with
`test_period: Cadence` and `schedule`; `jobs` on the wire in every build
(R49); `Clock { time_limit_ms, pool_after_ms, batch_time_limit_ms }` as
data (R148); `batch::Options` with `total_memory_bytes`, `run_local`
without `Send` (R48, the one draft that places it), `Results::cancel`
and `Cancel` (R40, F74); export `Styles` in the library with a
placeholder per feature that is off, so one settings file reads
everywhere (R141, R49); `Settings { clock, bounds, search, view, styles,
batch }` with `set(key, value)` and `keys()`; the flag→key table (D15,
R183). *Graft:* `Clock`, `Settings`, `run_local`, `Cancel`, the
placeholder rule, the flag table. *Wrong:* A11, "`Settings::default()` is
the command's behaviour (copies none) while `search::Options::default()`
keeps the copy bound of 3": two defaults for one field in one crate,
which C-COMMON-TRAITS and F42 (one paragraph stating the recipe) argue
against; keep one default and let the command set `copies: None`.

**B.** The key table of `search::Options` (with `limits` nested and
`refute_unknown` at 31), private fields with a builder and a getter each
(against F78's recommended public fields, serde still derivable: taste,
but it needs a getter per field), `Identifier` and `Length` as validated
option types (F38: the cleanest fix), `Styles::set`/`KEYS` with the help
text generated (F196), `TptpOptions` on `read_tptp` (D-12), presets
citing a measurement. *Gaps:* no `Clock`, no `schedule`, no non-`Send`
batch entry.

**C.** The value table (fields, defaults, set-by), `Period::BySize`,
`Options::BROWSER`, `ViewOptions::BROWSER`, `BENCHEXEC` (R151),
`OutputOptions` read by the library for `--style` (F196), the outcome
recording version and options (F77), and the decision "dispatch
thresholds are not options" (R149) with its reason. *Gaps:* as B's on
`Clock`, R48, R49.

### 7. Wire forms

| | A | B | C |
|---|---|---|---|
| score | 5 | 4 | 3 |

**A.** The policy in one place (what raises the level, what does not,
reading through a `DeserializeSeed` that records the version before a
later key fails, unknown keys per kind of form, enumerations by name,
names, numbers, text), the read-back/write-only table, every form with
an example and its reserved keys (sequent, proof, outcome, structure,
session, `Step`/`GoalView`/`Request`/`Response`, derivation, error and
progress, the options values, the batch record with `class`, the
ordinary forms, later files), the table of every string a client meets
(7.4, with closed/open/by-level), and where each form is pinned (7.5,
F90). *Graft:* the whole section as the skeleton of 7, with B's two
changes below. *Wrong:* (i) `"mix": false` → `"criterion": {"mix":
false}` moves the net's pinned form for nothing; B flattens `Criterion`
into the net so the key stays. (ii) the 2⁵³ sentinel (see 1). (iii) one
global level numbered as steps land couples unrelated forms (a session
file written at 35 says level 4 although nothing in it changed): B's
per-form "lowest version that represents the value" keeps each form's
doc self-contained; the two are otherwise equivalent and equally
defensible.

**B.** Six policy points, the forms with examples where B's choices
differ (sequent, proof with `goal` and `mode`, outcome reading back as a
`Proof` or a `Disproof`, tagged `refutation` and `reason`, `Disproof`,
statistics derived), unknown keys: data ignore, options and `Mode`
refuse (B6, with the reason), `"version"` the first key, the three lock
commits named. *Gaps:* fewer examples (session, options, ordinary,
error referenced elsewhere); no enumeration table.

**C.** Version 1 on every form, missing reads as 1, greater refused;
renames with the old names read as aliases; numbers per R247; the
read-back table; examples for sequent (`antecedents`), proof, outcome,
structure, options. *Wrong:* (i) it never says when a version is raised:
"33's `vertices`, `boxes`, `jumps`, 34's `cuts`, 38's `substitution` are
keys to come" and "a new key is optional, skipped when empty" is D-3's
option (a) under a `version` key that never moves; a 0.1.0 reader
ignores `vertices` and reads `links` over vertex ids as occurrence ids
(impact-boxes §11 item 3), which D-3 and F26 name as the hazard. (ii)
`Mode` as an object "every key under `serde(default)`" inside data forms
that ignore unknown keys reads a cyclic mode as commutative (R10's
hazard, which B's `deny_unknown_fields` on `Mode` and A's name avoid).
(iii) `mix` → `mode` in the net form is another needless lock change.

### 8. Engines and the search's front door

| | A | B | C |
|---|---|---|---|
| score | 4 | 4 | 5 |

**A.** `prove`/`prove_within`/`prove_goal`/`engine_for`/`race`
signatures; `Decide` and `Answer` crate-private with `bounds` and the
account; `Engine::{ALL, name, parallel, FromStr}`; `Verdict` closed;
`Reason::setting()`; the dispatch as data; the registration list; a
`REFUTERS` table of functions run after `Exhausted` and, behind
`options.refute.after_unknown`, after an `Unknown` (C2 C). *Wrong:*
`race`'s stop is `Fn(Progress) -> bool + Sync` where every other call
takes `FnMut`, against A's own A6; "a `debug_assert!` in every build" is
self-contradictory (the check that promotes `Unknown` to `Unprovable`
must run in every build, as the register's C2 text intends).

**B.** `Goal<'a>` with `conclusion`, `new`, `is_conclusion`,
`is_whole_forest` (D-2's "the goal is every tree", impact-boxes item 10:
the net engine never sees a cut tree), `Verdict::Unprovable(Box<Disproof>)`,
`Reason` with `WorkLimit`, `Unchecked`; the registration list; refuters
as a crate-private `trait Refute` run under the same stop, account and
`Limits::work`; `Statistics` with `copies` fixed (F144). *Graft:* `Goal`.
*Gap:* no `Schedule`; `race` takes `widen: &AtomicBool`, fine behind
`parallel`.

**C.** The pipeline of `prove_goal` (`fragment_of`, `read`, `prepare`,
the set-up poll, `decide`, `conclude`) with "no engine builds a
`Verdict`" (S4); `Task` and `Answer` with their 36 and 38 fields
reserved; `Decide` with the contract in its doc (`Ok(None)` only after
an exhaustive search, R111; polls with its work; charges the account;
never panics on a goal it admits); `Engine` with `ALL`, `name`,
`parallel` positive and exhaustive (F141), `implementation`; the
ten-item registration list (R86); `DISPATCH` as code; `Modes`
destructuring the whole `Mode` (F140); `Feature` as one polled pass
(R88); the `Statistics` table per engine (T5, R9, `u64` for R46);
`NotTaken` (R135); `Refute` with `Counts` as the first member from 28
(moving `focus::refutation` to `search/refute.rs`); `race` with
`start_pool: FnMut(&Progress) -> bool` so the caller's clock decides
(F103, F168, R85); the names inside (F56, F113). *Graft:* the whole
section. *Note for the author:* C2's recommended text says "a table or
trait of refuters waits until a second refuter exists"; C's decision 5
overrides it with the counts refutation as the first member, which is
not speculative (the function exists) and is the cleaner place for
F128's fix; C lists it as a decision, which is right.

### 9. Exports

| | A | B | C |
|---|---|---|---|
| score | 4 | 5 | 3 |

**A.** `Drawable` sealed (F36), `write`/`derivation`/`sequent` per
target, `svg::net(.., &bounds, stop)` (R26), `from_svg(.., &bounds)`,
the SVG id grammar as contract (R163, R168), Rocq's 31 additions listed,
PNG and PDF outside the web client (R44). Sound; the sequent and net
drawings stay separate functions.

**B.** `Item<'a>` (sequent, two-sided, derivation, ordinary, net, proof
with mode, disproof) and one `write(item, &options, &limits, out, stop)`
per target, every writer estimating its output against `output_bytes`
first (F5, R26); `rocq::Options { form, lemma: Identifier, prelude:
Option<String>, kernel }`, `Kernel::Auto` by mode and the proof's rules
(B10, R139, R159 kept byte-identical), `NANOYALLA`, `LIBRARY`, `FORMAT`
(R12), the term certificate's shape (`by_check` over data, the forest
recomputed per 3.2, positions as data, R152, R54), the mirror `node (M :
Type)` over the member type (R244, R118), refutation certificates
(R153), reserved names per kernel (R155), cut and first-order as
`FORMAT` 2 and 3. *Graft:* the whole section. A target refusing an
`Item` it has no form for at run time (latex given a net) is the price of
one signature; defensible against A's compile-time `Drawable`.

**C.** One signature with `to_string`, Rocq's 31 items, the id grammar,
new options as fields. Right and short; nothing of R12, R152's shape or
the kernel rule.

### 10. How each later step enters it

| | A | B | C |
|---|---|---|---|
| score | 4 | 5 | 5 |

**A.** Steps 29 to 37 in a paragraph each with register ids, 38 in full
with "what stays untouched". Good coverage; 33 to 37 are lists of items,
not designs.

**B.** 29 to 37 with the items and the types: 33's vertex table, doors,
jumps; 34's `proofs::cut` module (`step`, `eliminate`, `Reduction`,
`Elimination`, `End::{Normal, Refused}`: a bound is never "normal",
R32, R100), `Forest::with_cuts`, the reading of a cut pair (R99); 36's
`Named` orientation, `Split::At`, `Fault::{Order, EmptyAntecedent}`
(R246's L and L*), `SequentBuilder::dual` taking the order (R56); 37's
lift of `Context`/`Classes`; 38 in full with the checker's frame pass
and eigenvariable set (fo-linear §4.2) and "what stays untouched".
*Graft:* 29 to 34.

**C.** 35 (the `Symmetries` structure R103, the pure-function pruning
state R33, the balance prune on the union-find R82, `structural_classes`
R83, `Linker<C: Correctness>` with `Switching` and `Essential`,
`Engine::Essential` forceable, the closure charged R35, ablation
`NetPrunes` R143), 36 (`Mode::order`/`empty_antecedents`, the planar
order *derived* from the reading rather than stored, with the lemma for
36's panel and a fallback: the one draft that answers R56 and research
conflict 3 with an argument; the divisions; the net engine under the
mode's criterion; the checker in planar order R121, R246), 37 (the
counter-neutral lift first, `Ok(None)` only where complete R111, the
forward derivation as `Node`s R127), 38 in full: (a) to (l), with the
prune table (R107), levels for the eigenvariable condition (fo-linear
§3.2), the undo points in the focused engine (step 26's item 7), answer
enumeration across `⊗` (impact-quantifiers finding 3), the checker's
branch per proof, views, ordinary first order (fo-embeddings §1.5), the
bounds. *Graft:* 10.2 to 10.5. *Wrong:* 10.3's "text whose lowering
negated a compound is refused in the cyclic classical mode" is a
limitation to put before the author at 36, not at 28; C says so.

### 11. The spike

| | A | B | C |
|---|---|---|---|
| score | 3 | 4 | 5 |

See "The spike" below.

### 12. Findings answered

| | A | B | C |
|---|---|---|---|
| score | 5 | 4 | 4 |

A: a table of finding→section and the register by section (every entry
of the eight sections placed). B: an inline list by id. C: an inline
list, the register ids answered, and "placed with a later step" named.
All three answer the audit's minimum list (F8, F30, F48; F26, T1; F12,
F62, F136; F76–F79; F7; F19, F25; F24, C1; F1; F22; H18).

### 13. Decisions for the author

| | A | B | C |
|---|---|---|---|
| score | 5 | 4 | 4 |

A: a table of every open decision with what the other answer changes in
this design, and eleven decisions of its own with the alternative set
aside (A1 to A11). B: the open ones in prose and B1 to B11 (B1 closed
enums, B2 a proof records its conclusion and mode, B8 cut accepted in
every commutative mode with no flag, B10 `Kernel::Auto`, B11 for 31). C:
a table and nine decisions (the sides, the derived planar order, one
integer member, public fields, refuters from 28, the progress predicate
of the race, `Engine::Essential`, `FoTerm` naming, thresholds not
options). A's is the one the author can answer line by line; B's B2,
B8, B10 and C's 2, 5, 7 must be added to it.

## Register coverage

**Entries no draft meets or places.** R63's "per-occurrence byte
assertions" on the `Forest` arrays (every draft asserts `Term`, `Node`,
`Member`; none the forest's per-occurrence bytes: one `const` on the
sum of the arrays' element sizes). R115 (the focused search hands a goal
to the net engine at the recursion limit) is named by A only, as a
later item; B and C are silent (it is a later step's; place it under
35). Everything else of the eight sections and C1 to C3 is met now or
placed by at least one draft.

**Entries one draft misses that another meets.**
- R47/F110 (the default bias's scheme as an option): A and C have
  `Schedule`; B has none.
- R48 (a batch entry without `Send` bounds): A's `run_local`; B and C
  none.
- R49 (each feature subset the client ships is tested) and one settings
  file for every build: A's `jobs` on the wire in every build and the
  `Styles` placeholders; B and C none.
- R59 now (the builder for the web client, step 32): A and B; C defers
  to 34.
- R46 (`u64` where a count can pass 2³² on wasm32): A and C make
  `Statistics::memo_entries` `u64`; B keeps `usize` by silence.
- R79 and D-6 (a criterion value, a vertex type): A and B; C uses
  `Mode` and `OccId` pairs (a 33 break).
- R10 (a cyclic mode never read as commutative): A (a name) and B
  (`deny_unknown_fields` on `Mode`); C's object ignores unknown keys.
- R8 and R6's "a reader of the new version reads old files, an old
  reader refuses new meaning": A and B raise the version with `vertices`
  and `cuts`; C does not say so.
- R117 (the checker specified): B alone; A defers, C has the first-order
  pass only.
- R12 (the version constant tying the writer to the Rocq library): B's
  `LIBRARY` and `FORMAT`; A's `LIBRARY`; C none.
- R92's per-goal budget: all three; C's `Progress::item` lets one stop
  budget per goal without a second parameter.
- R91 (the rules `apply` would accept): A's `Applicable`/`Needs` at 28
  and `rules` filtered; B additive later; C none.
- R247: B and C follow the recommendation; A deviates (2⁵³).
- H9: C refuses it by a stated rule; A by intent (`MovedBottom`) without
  the rule; B does not refuse it.

## The disagreements that matter

1. **The member type.** All three: `Member(u32)`, one integer on the
   wire, equal to the occurrence id below `forest.len()`, an index into a
   table the owner keeps above it, in every list and every `Node` operand
   (A delegates the operands to B). Verdict: agreed. Take B's owner
   methods `occurrence(m)`/`formula(m)` on `Proof`, `Derivation`,
   `Interactive` (a member above `len()` needs the owner's table, which
   A's and C's `Member::occurrence(&Forest) -> Option<OccId>` cannot
   reach) beside `From<OccId>`.
2. **Atoms and terms.** All three reserve `Pred`/`DualPred(Atom, ArgsId)`
   beside `Atom`/`DualAtom`, `Atom` a predicate symbol, the fragment bit
   set by an argument as well as a binder (R52, D-4). Question 1 below:
   I would choose the interned atomic formula instead.
3. **The root order and the sides.** All three: C1 option A, written
   order canonical in every mode, equality structural (a cyclic sequent
   is not equal to its rotations; the rotation is the search's question).
   The sides: A `antecedents: u32` (0 = one-sided), C `Option<u32>`
   (`None` = unknown, today's reading), B none (goal = last root).
   Verdict: C's type and C's rule (question 2); the key `antecedents` is
   written whenever `Some`, including `Some(0)`, so every pinned sequent
   gains it in the version commit.
4. **The error family.** A flat with six classes and `setting()`; B
   kinds with a `Refused` variant in every specific type; C three classes
   with consolidated variants. Verdict: B's structure (S4 by
   construction), A's six classes (`Stopped` and `Failed` apart from
   `Limit` and `Unsupported`) and `setting()`, C's `IndexOutOfBounds {
   space, index, len }` and `UnknownName`.
5. **The bounds value and the stop.** Name: `Bounds` (A) against
   `Limits` (B, C): taste. Fields: recursion inside (A, C) or on
   `Options` (B): inside, since `stack_bytes()` derives from it and the
   browser preset sets it with the rest. Where it travels: a parameter
   beside `Options` (A, `prove_within(.., &bounds, stop)`) or a field of
   `search::Options` (B, C, `prove` keeps its signature): B's and C's,
   with A's `Settings` as the one value a front end holds. The stop: a
   trait with a blanket impl for `FnMut() -> bool` (B, C) against a
   closure over `Progress` alone (A): the trait; every poll site keeps
   its closure and a progress reader wraps (B9). `Progress`: C's `{ work,
   done, held, phase, item }`.
6. **The options' wire forms.** Public fields under `#[non_exhaustive]`
   (A, C, F78's fix) against private fields with builders and getters
   (B): public fields. `null` = no bound, `"auto"` = automatic (A): take.
   A mode on the wire: its name (A) against the object (B refusing
   unknown keys, C ignoring them): A's name, one table (`Mode::NAMES`,
   AIP-126), and the object read as the pre-release form; at the least
   B's `deny_unknown_fields`. The word for weakening with Mix:
   `affine-mix` (A, C) against `mix-affine` (B): taste; two of three say
   `affine-mix`.
7. **The version policy.** A: one global level, the lowest a reader
   needs, computed from the content. B: a version per form, the lowest
   that represents the value. C: version 1, raised when unsaid.
   Verdict: A or B (equally defensible; B's keeps each form's doc
   self-contained); C's must state what raises the version (a new tag, a
   new enumeration value in a form read back, a key whose presence
   changes what other keys mean), or D-3's hazard stands.
8. **`Node`, `Proof` and the checker.** B: `Node` over `Member`, `TAGS`,
   `Proof { goal, mode }`, the checker specified; C: `Node` reserved,
   `Proof::mode`, `with_instances`; A: defers, `ProofFile { proof, mode
   }` at 31. Verdict: B (a proof records its conclusion and its claimed
   mode; `check` checks against its own conclusion; F89 and R7 at the
   type, no `ProofFile`).
9. **The net's vertices and criterion.** B: `VertexId`, `Criterion {
   mix }` with `of(mode)` and `admits(fragment, mode)`, links over
   `VertexId`, `from_proof` exhaustive and bounded, no `PartialEq`, the
   fields flattened into the net form (`"mix"` stays). A: `Criterion`,
   vertex deferred. C: `Mode` as the criterion, `OccId` pairs, `mode` key.
   Verdict: B.
10. **The engine interface.** All crate-private `Decide`; C's pipeline,
    contract, registration list and `Statistics` table; B's `Goal<'a>`;
    refuters: a trait and list from 28 (C), from 31 (B), a table of
    functions (A). Verdict: C's section with B's `Goal`; the refuter kind
    from 28 to the author (C's decision 5).
11. **The race.** A `(threads, add_pool: Fn, stop: Fn + Sync)`, B
    `(.., stop, widen: &AtomicBool)`, C `(.., start_pool: FnMut(&Progress)
    -> bool, stop: impl Stop)`. Verdict: C's (the caller's clock reads
    the progress; one stop kind), B's `AtomicBool` the simplest
    alternative.
12. **`Rule`.** B splits it into fifteen one-sided rules plus `Open` and
    `Named { rule, side }` (F60); A and C keep the 34. Verdict: B, before
    0.1.0 (38 then adds two rules, not six; 36's divisions are a side and
    an orientation).
13. **Refutations.** B: `Verdict::Unprovable(Box<Disproof>)`, the
    disproof owning the sequent, goal, mode and refutation, `check`
    recomputing the conditions; `StateEquation` payload with every place
    (R70). A, C: `Refutation` alone, the sequent and mode in a stored
    file. Verdict: B (R70, R71, R124, R3 at the type; one arena clone per
    unprovable outcome is cheap). `Unbalanced.atom` by index (B) against
    name (A, C): with the sequent beside it, the index (F80's one
    representation).
14. **The 2⁵³ rule.** A writes a saturated count as 2⁵³; B and C keep
    `u64::MAX` as R247 recommends. Verdict: B and C; A's probe claim is
    refuted, and `Number("18446744073709551615") === 2**64` is as exact a
    sentinel in JavaScript as 2⁵³ is.
15. **The export entry.** B's `Item` enum against A's and C's `Drawable`
    trait: taste; B's puts the sequent and net drawings under the same
    bounded signature (F5, R26).
16. **`ProofStructure`'s pinned key.** A `criterion`, C `mode`, B `mix`
    unchanged: B.
17. **`Interactive::rules`.** A returns `Vec<Applicable>` filtered by the
    goal's context at 28 (R91, R93); B keeps `Vec<Named>` and adds
    `applicable` later; C `Vec<Rule>`. Verdict: A's now (step 32 builds
    on it), typed with B's `Named`.
18. **H18.** B and C: the depth is per thread (a stolen job counts on top
    of the thief's), so only `Unknown(RecursionLimit)` can result; A
    leaves the mechanism to the search area. Verdict: C's wording.
19. **`Fragment::ALL` → `ADDITIVE`** (B, C; F56): take; A is silent.
20. **`Sign`**: `{Atom, Dual}` (A, B) against `{Plain, Dual}` (C): taste.

## The spike

C's specification measures D17 best. It is the only one that compiles a
second instance of the generic engine into the binary and reaches it from
the dispatch (M3: `Framed`, a stub of real code with a unit test that
proves `⊢ p(c), ~p(c)` through it), which is exactly what fo-linear §5.5
says monomorphisation does not settle (code size, the instruction cache)
and what D-7 wants decided. A's layer 2 keeps "a stub framed zone in a
test only, so the binary holds the propositional instance alone", which
measures the trait's hooks but not the second instance; B's three parts
measure the `Space` parameter of the checker (good, A and C lack it) but
no second instance either. C's gates are the right shape: G1 exactness
on the five columns against the base run of the same day and the
committed oracle (`after-coverability.csv`, F181: A names
`after-bias.csv`, which F181 shows is no longer the oracle), G2 the
twenty journeys with +1.0 % per search journey and +0.5 % on their sum
(tighter than the ratchet's 2 %, as a design gate should be), G3 pinned
time with a geometric mean of 1.02 and interleaved runs, the text size
reported, and a decision table including "M2 passes, M3 fails" (outline
the cold paths, measure again). Both base and milestone built in one
environment (bench.md's 0.9 % between the shell's and the flake's
builds).

What C should add: B's attribution of journeys to parts (`read-*` for the
variants, `check-*` and `derivation-chain-64` for the member and the
checker's branch, `search-*` for the zone), which C has only as "bisect
its four parts" when M1 fails; A's rule that the change of every poll
site to the progress stop is measured in a commit of its own (it moves
no counter, but it touches the hot loop); and B's `Space` parameter of
the checker's pass in M1, since D-5 recommends the frame pass "running
only when the forest has binders" and a generic pass is one way to keep
the ground pass byte-identical. What C should drop: nothing; the
prescription of cores 2 and 3 belongs to the machine rules, not the
design.

## The ranking

**B, then A, then C.** B gets right the decisions whose wrong answer
costs most after 0.1.0 (design-constraints §1: D-1 to D-5 fail silently
or in files that cannot be recalled): the member and the owners' tables,
the forest's contract, the proof recording its conclusion and mode, the
checker specified for the Rocq mirror, the vertex and criterion, the
disproof, the error family with a refusal variant in every type, the
`Rule` split, and the version per form; its one wrong choice, the
goal-last reading, is a paragraph and C has the replacement. A is the
most complete and the most searchable document (the before→after table,
the wire forms with every example and string, the options with their
flags, the bounds with their reasons, the decisions table) and the only
one that designs for the web client's worker and settings; its wrongs
are a type (`u32` for the sides), a refuted probe behind one rule, and
two defaults for one field. C is the authority on the engines, steps 35
to 38 and the spike, and its sides rule is the one that refuses both H9
and H10; but its wire policy leaves D-3's hazard open, its net API would
break at 33, its errors are coarser, and it defers the builder past the
web client. Scores by section: A 56, B 56, C 54 of 65; the tie is broken
by where the risk lies.

**The base to synthesise from.** Take A's outline and tables as the
skeleton (sections 1, 2, 5, 6, 7, 12, 13: the fix sessions and the author
search those), and paste B's sections 3, 4 and 9 and C's sections 8 and
11 in whole, merging 10 from B (29 to 34) and C (35 to 38). Grafts in
order of value:

1. C's sides rule into B's 3.4 and A's 3.1: `antecedents: Option<u32>`
   set by the parser for every text (`Some(0)` included), by `lltp` and
   `translate`; with the sides known the goal is the one root right of
   `⊢` (else `ShapeError::Succedents(n)`) and an implication's antecedent
   is the factor the lowering put there (the symmetric reading dropped);
   `None` keeps today's choice (question 2).
2. B's data model (3.1 to 3.3, 3.5 to 3.11) and errors (4) as the core
   text, with A's `Builder::left/right`, A's `Class::{Stopped, Failed}`
   and `setting()`, C's `IndexOutOfBounds { space }`.
3. A's wire forms (7) as the text, with B's per-form version, B's
   flattened `Criterion` (`"mix"` stays), `u64::MAX` per R247, and the
   mode's name (A).
4. C's engines (8), 10.2 to 10.5 and the spike (11), with B's `Goal<'a>`
   and A's separate measurement of the progress stop.
5. B's exports (9) with A's SVG id grammar and PNG/PDF note.
6. A's bounds and options (5, 6): `Bounds::BROWSER`'s reasoning, 5.5,
   5.6, `Clock`, `Settings`, `run_local`, `Cancel`, the placeholders, the
   flag table; C's `Progress { done, item, phase }`, `Schedule::Threads`,
   5.3's account table, H18's mechanism; the stop as B's and C's trait.
   Drop A11 (one default for `copies`).
7. A's `Applicable`/`Needs`/`GoalView`/`FormulaView` and the
   `Request`/`Response` forms (7.3.6), typed with B's `Named`.
8. B's `Rule`/`Named` split everywhere a rule is named (`Step`,
   `Inference`, the label tables).
9. C's 3.9 (the first-order tables) as the named reservation, and C's
   `Fragment` paragraph adjusted to question 1's answer if taken.
10. A's 12 and 13 as the format, with B2, B8, B10, B11 and C's decisions
    2, 5 and 7 added to the author's table.

## Question 1: an atom as an interned atomic formula

**The drafts' choice.** `Term::{Pred, DualPred}(Atom, ArgsId)` beside
`Atom`/`DualAtom`, `Atom` a predicate symbol `(name, arity)`, and
`Fragment::QUANTIFIERS` set by an argument as well as a binder, so that
a ground `p(a)` against `~p(b)` reaches no propositional engine
(impact-quantifiers §1 item 1; R52; D-4). Its soundness rests on guards:
every engine's `admits` refusing the bit, `Proof::check` and the oracle
refusing a forest with arguments until `Ax` compares instances, and the
eight pairing sites (`check.rs:1167`, `oracle.rs:271`, `nets/mod.rs:280`,
`additive.rs:191`, `net.rs:437`, `focus/mod.rs:1588–1647`,
`focus/split.rs:19–40`, `horn/mod.rs:405–412`) routed through one
`dual_literals` that refuses arguments. The failure mode is silent: a
forced engine, a JSON proof fed to the checker, a wildcard arm over
`Term` (D-11's thirty sites) or one site left on `atom(a) == atom(b)`
accepts `p(a)` against `~p(b)` as an axiom. The bit "computed from the
terms, not from `Kind`" is one more pass that exists only to close that
hole.

**The alternative.** `Atom` is an interned atomic formula: a predicate
symbol applied to argument terms, hash-consed in the sequent's atom
table, nullary for every propositional atom; `Term` keeps `Atom`/`DualAtom`
as its only literal variants and gains `Forall(TermId)`/`Exists(TermId)`;
only a binder sets the quantifier bit; under a binder an atom's arguments
hold de Bruijn indices, and the first-order engine and checker read an
occurrence's atom through the member's frame.

*Soundness.* A ground first-order sequent is then a propositional sequent
whose atoms have structured names: `p(a)` and `p(b)` are two indices of
the atom table, so every one of the eight sites is correct as it stands,
by construction and not by guard, and D-11's wildcards cannot pair them.
One guard remains, the binder bit, set where `Sequent::fragment` already
reads `Kind::Forall|Exists` in its one pass; it protects the engines and
the checker until the frame pass exists, exactly as the drafts' bit does,
but it is one bit set by one kind, not a property computed from the
terms. The checker's rules for binders are the same as the drafts' (`Ax`
compares instances through frames by hash-consed id, fo-linear §4.2).
Nothing goes silently wrong on ground input anywhere; on binder input the
same one guard as before. Where the alternative could go wrong: an atom
with a loose de Bruijn index at a root, from JSON or a builder; `Sequent::check`
refuses it with the scope check both designs need (R136).

*D17.* `Term` gains two variants, not four; `Kind` gains two; no new
literal kind, so `is_literal`, `Kind::sign`, `Term::atom()`,
`optimize_atoms`, `merge_atoms`, `offset` and `map_subterms` (the sites
F85 and D-11 list for literals) are untouched; the forest's literal CSR
by `2·atom + sign` is exactly right for the propositional engines on
ground first-order input, and the `Counts`, `Classes` and bias are per
ground atomic formula, which is the sound and complete propositional
treatment. The atom table of a propositional sequent stays `Vec<String>`
in effect (an entry with a symbol and no arguments); the structured
entries and their interning map are empty tables. The spike's Q1 has
less to measure.

*The wire form.* The tags `V` and `D` serve every atom; no `P`/`N` tags.
An `atoms` entry is a string (nullary) or `[symbol, [argument ids]]`,
with `fo_terms`, `symbols` and `binders` as the drafts have them; a
propositional file is byte-identical; a ground first-order file fails on
a 0.1.0 reader at the entry's type, named by the version check. HD5's
identifier rule applies to the nullary entries and the symbol names.

*The first-order engine's needs.* Candidates by predicate symbol need a
second index, symbol → atoms (a CSR over the atom table, per atom not
per occurrence, built when the sequent has a symbol of arity above 0);
the literal seam then iterates the symbol's atoms and their lists and
unifies `args(atom)` read through the two frames, which is what the
drafts' seam does with `literals(symbol, sign)` and `ArgsId`. Counts and
bias "per predicate symbol" (C's prune table) aggregate the symbol's
atoms through the same index. `Unbalanced` names an atom, which is now a
ground formula or an open one: finer, still sound.

*Printing.* An atom prints its symbol and arguments through a term walk
inside the atom's stop (D-10's "a term walk inside a predicate's
`Enter`"), bound names from the binder side table of the binders above
`o` (impact-quantifiers §3 item 5), as in the drafts.

*What step 38 would find harder.* (i) Interning: a `Pred` term is
hash-consed by `optimize_terms` for free, whereas the atom table needs
its own interning by `(symbol, args)` in `optimize` and in the parser
and `add`'s merge becomes structural (today it is by name): a map and a
pass, after the argument arena is canonical. (ii) `Sequent::add` and
`merge_atoms` offset and merge the argument arena and the symbol table,
as the drafts also must. (iii) A ground first-order sequent is reported
as `MLL`, `MALL`, … (propositional) and routed to the propositional
engines, the Horn engine included: a change of vocabulary R52 and the
first-order families (R211) must state, and a gain for practice
(parametrised Petri net places decide today). (iv) The Rocq export of a
ground first-order proof names atoms that are formulas: the 31 kernel
takes atoms as indices (R69) and the per-kernel escaping (R155) handles
the name. Nothing in the frame pass, the trail, the memo keys or the
witness arena changes.

*What the drafts' choice does better.* Hash-consing of predicate terms
for free; `Term::atom()` returning the symbol suits per-symbol counts
without an index; one notion of `Atom` that D-4 and R52 already
describe; the register's letter (R52 "set by a predicate with arguments
as well as by a binder") is met rather than amended.

*Verdict.* I would choose the interned atomic formula. The register's
R52 clause exists to remedy finding 1, and impact-quantifiers §1 item 1
itself offers this remedy ("or else ground atomic formulas must be
interned as atoms of their own"); it removes the hazard at the source
instead of guarding eight sites and thirty wildcards, costs the
propositional case less (two variants, no new literal kind), makes
ground first-order problems decidable by every engine on the day the
data model lands, and asks of 38 an interning map and a symbol index.
It amends R52, R62 and R16 (no `P`/`N` tags), and D-4's "Atom is a
predicate symbol" becomes "an atom is an atomic formula; its symbol has
an arity". It is a decision for the author, since it departs from the
research's recommendation, and the spike's M1 should be built on it if
taken.

## Question 2: H9 and H10

The witnesses (28-audit-findings.json): **H9** `linlog prove -i
'(A -o bot) -o bot |- A'` is proved as `1 ⊸ (A ⊗ 1) ⊢ A`. The hypothesis
lowers to `(~A ⅋ ⊥) ⊗ 1` (`parse/mod.rs`: `A ⊸ B` on the left is `A ⊗
B⊥`, the antecedent left); `Reading::new` (reading.rs 250–266) tries the
left factor as the antecedent (output position), which fails since `⊥` is
input-only (line 208), then the right factor `1`, which works: the
symmetric reading of an implication (core-forest.md, "The choices, made
deterministically"). **H10** `linlog prove -i '|- a, top'` is proved as
`0 ⊢ a`: the goal is the one root that can only be output (reading.rs
222–228), which is `a`, and `⊤`, which can stand on either side, is read
as the hypothesis `0`; the second written succedent moved left. Both inputs are text, and every text sequent has
`|-`, so `|- a, top` is a two-sided sequent with an empty left side; the
same arena arises from `~a |- top` (A's own example), which is why the
arena cannot refuse it and the sides must be kept.

**A.** Rule: the one written succedent is the goal; two written
succedents refused (`SeveralSuccedents`); a `⊥` the reading "would have
to move" refused (`MovedBottom`); by shape for one-sided input. *H10:*
refused only if `|- a, top` counts as two written succedents; with
`antecedents: u32` and "0: one-sided input" A's text says both; the
intent is clear, the type is not. *H9:* the intent is right but the
mechanism is unstated: `MovedBottom` names the symptom, and the cause is
the symmetric factor choice, which A never says it drops (so `b ⅋ ~a`
spelled by hand would still read as `a ⊸ b`, while H9's shape is
refused by a special case). *One-sided input with the goal not last:*
unaffected by shape (today's rule) if "one-sided" means `antecedents ==
0`, in which case H10 is not refused; refused as two succedents
otherwise. *JSON:* a sequent without the key reads as zero antecedents,
so the file of `|- a, top` and the text take different rules, and a
sequent written back omits the key when zero: no round trip of the
reading. *LLTP:* the reader parses `axioms |- conjecture`, so the sides
are known and HD1 already refuses a second conjecture. *The symmetric
reading:* not removed by any stated rule.

**B.** Rule: the goal is the last root, which must read as output, every
other root input, never reread with another goal. *H10:* refused (`⊤`
last is the goal, `a` must be input, cannot). *H9:* **not refused**: the
goal `A` is last and reads as output; the hypothesis must read as input,
and `(~A ⅋ ⊥) ⊗ 1` does read as input through the symmetric factor
choice (`⊗` in input position: `l_in && r_out` with `1` as the flipped
antecedent, reading.rs 250–258), so B still proves `1 ⊸ (A ⊗ 1) ⊢ A`.
B's 12 lists H9 as answered in 3.4; it is not. *One-sided input with the
goal not last:* refused (`|- B, ~A` under `-i` reads today as `A ⊢ B`;
under B the last root `~A` must be output and cannot): a regression no
lock case covers (`cli/tests/cli.rs` 453 and 526 and `lock.rs` 328 use
one-sided `-i` only for inputs that are refused anyway). *JSON:* the rule
applies to every stored file; old files whose sorted roots put a shared
goal first (`A, A -o B |- A`: the goal `A` has the small id of the
hypothesis's atom, so the sorted roots end with `A ⊗ ~B`) are refused,
although B's reader accepts the pre-release form. *LLTP:* fine (the
conjecture is last). *The symmetric reading:* kept.

**C.** Rule: `antecedents: Option<u32>` set by the parser (and `lltp`,
`translate`); with the sides known, the goal is the one root right of
`⊢` (`ShapeError::Succedents(n)` otherwise) and an implication's
antecedent is the factor the lowering put there, the left one under D1,
so the symmetric reading is not tried; `None` keeps today's choice.
*H10:* refused (two roots right of `⊢`), provided the parser sets
`Some(0)` for `|- …`; C's 7 says "absent for a sequent written
one-sided", which must mean JSON-born, not text: the synthesis says
`Some(n)` for every text and writes the key whenever `Some`. *H9:*
refused: the left factor `~A ⅋ ⊥` must be the antecedent and cannot be
output. *One-sided input with the goal not last:* `|- ~a, b` under `-i`
becomes two written succedents and is refused, with a message to write
hypotheses left of `⊢`; the audit's H10 fix ("two written succedents are
refused whatever they are built from") accepts this, and no lock case
has a valid one-sided `-i` input; D1's "an ILL sequent is a one-sided
sequent of a particular shape" stays true of the arena, and the one-sided
spelling remains readable through JSON (`None`) and `Sequent::add`.
A hand-written symmetric spelling in two-sided text (`|- b par ~a`) is
refused too: ILL users write `-o`, and the lock has no such case. *JSON:*
without the key, today's reading; with it, the text's rule; the writer
emits the key, so a file round-trips. *LLTP:* sides known, the
conjecture the goal; HD1 refuses a second one. *The symmetric reading:*
removed exactly where the sides are known, kept where they are not:
right, since for parsed two-sided text the lowering fixes the antecedent
and the symmetric reading can only succeed where the proper one fails,
which is the H9 class; for one-sided JSON the user gave a one-sided
sequent and any reading of it is an interpretation, not a rereading of
what was written.

**Verdict.** C's rule is the one to take, with `Some(0)` made explicit
and the `antecedents` key written whenever present (a change of every
pinned sequent form, in the version commit). A intends the same and
needs C's type and rule; B's rule must be replaced, since it leaves H9 as
a wrong verdict and refuses inputs that are fine today.
