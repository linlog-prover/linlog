# Walk-through of step 29 (the comparison) against `plan/notes/api.md`

Sources read: `api.md` in full (sections 11.5 and 12 are placeholders),
`plan/29-comparison.md`, `plan/notes/research/29-comparison.md`, the
register entries naming 29 (R2, R18, R74, R75, R85, R151, R172, R184 to
R193, R217, and R245, R190 which also name 32/38), `core/src/sequents/fmt.rs`,
`core/src/lltp.rs`, `core/src/occurrences/reading.rs`, `bench/src/run.rs`.
Step 29 is mostly `bench/`, `modules/` and CI. Its dependence on the library
is narrow: the translators, the verdict reading, the memory bound and the
race. That is where the findings are. No finding is `blocking`; the design
holds. Seven are `friction` (2.1 to 2.6, 2.8), six are `note`.

## 1. The first change, sketched

Two commits in `bench/`, then the flake and CI. The library is not touched.

**Commit A: a translator layer over the public walk.**
`bench/src/tools/{mod,syntax,cover}.rs`, no external tool yet.

```rust
// bench/src/tools/syntax.rs: one table per tool; the printer is generic.
use linlog::sequents::fmt::{Visit, Walk};      // public by R245 (2.1, 2.4 last rows)
use linlog::{Error, Forest, Kind, Limits, Mode, Reading, Sequent};

pub struct Syntax {
    pub name: fn(&str) -> String,             // atom names: HYPHEN/DOT back, Prolog case rules
    pub op: fn(Kind) -> Option<Op>,           // None: the tool lacks the connective
    pub lolli: Option<Op>,                    // two-sided ILL tools only
    pub sequent: Layout,                      // OneSided(roots) | TwoSided("=>", ",") | SingleFormula(par)
}
pub enum Op { Infix(&'static str), Prefix(&'static str), Postfix(&'static str), Word(&'static str) }
pub enum Untranslatable { Connective(Kind), Quantifier, StructuredAtom, Mode(Mode), Limit(Error) }

pub fn print(s: &Sequent, mode: Mode, syn: &Syntax, limits: &Limits, out: &mut String)
    -> Result<(), Untranslatable>
{
    let forest = Forest::within(s, limits).map_err(Untranslatable::Limit)?;
    if mode.is_intuitionistic() {
        let r = Reading::new(&forest)?;                    // R54's reading, from s.antecedents()
        for h in r.hypotheses() { emit(r.walk(h), &r, syn, out)?; }   // ILL stops (R245)
        emit(r.walk(r.goal()), &r, syn, out)
    } else {
        let k = s.antecedents().unwrap_or(0) as usize;     // the written sides (3.1, R74)
        for (i, &root) in s.roots().iter().enumerate() {   // written order (C1)
            let dualized = i < k;                           // a left root is stored as A-perp
            for v in Walk::over(s, root) { /* Enter / Between(t, index) / Exit; kind = s.term(t).kind() */ }
        }
    }
}
```

`cover.rs` is the driver's admission table:

```rust
pub struct Features { pub fragment: Fragment, pub mode: Mode, pub kinds: u16 /* a pass over s.terms() */,
                      pub structured_atoms: bool /* see 2.3 */ }
pub enum Coverage { Covered, Outside(&'static str) }       // "outside" is the CSV's `refused`, reason "fragment"
```

Tests of the commit (R245, R74): each printer on a formula nested 100 000
deep on a 256 KiB thread; a translated Petri net re-read by `lltp::read` is
equal as a `Sequent` (C1 and `antecedents` make `Eq` mean "the same written
problem"); each translator on the families (known verdicts, F6's
`Family::instance -> Result`).

**Commit B: tool rows in the CSV and a driver.**
`HEADER` gets columns appended at the end of the tail, as `bench.md` requires:
`tool,tool_version,translate_ms,peak_kb,raw_status,settings`. A `Driver` enum
(`Linlog(Column)`, `External(Tool)`) with `command(&Problem) -> Vec<String>`,
`verdict(exit, stdout) -> Verdict3 { Proved, Refuted, Unknown, Refused(reason), Error }`
and the tool's own limits. The linlog columns are one process each, measured
whole by BenchExec:

```text
linlog prove --file P.p --input-format lltp [-i] --deterministic --timeout none \
             --memory-limit <bexec - margin> --format json            # one thread (main column)
linlog prove --file P.p --input-format lltp [-i] --timeout none --memory-limit <...> --format json   # default (race)
```

The reader takes `version`, `verdict`, `engine`, `statistics`, `reason.kind`,
`refutation.kind` from the outcome JSON (7.3); a nonzero exit with an `Error`
document is classed by `kind` (4.1: `unsupported` = refused, `limit`/`stopped`
= unknown, `defect`/`failed` = error). Where the harness calls the library
itself (the `one` child for the main table, R85):

```rust
let forest = Forest::within(&s, &limits)?;
let outcome = linlog::search::race(Goal::conclusion(&forest), mode, &options, &limits,
                                   threads, |p| clock.pool_due(p), |p| expired.load(Relaxed))?;
```

Linlog's configuration for the page and for the authors' message is
`Settings` as JSON (7.3), written once per column into the `settings` field.

## 2. Workarounds

### 2.1 The public walk has no signature (friction)
*Need.* Commit A prints through `Walk`, `Visit` and `Reading::walk` (R245).
*Design.* 2.1 lists `fmt: Walk, Visit (public, R245)`; 2.4 gives only
`Visit::Between(TermId, index)`; 3.6 says "`Reading::walk(o)`, the two-sided
walk, is public". No type is shown, and today's `Walk<T, F>` is generic over
the id type with a closure for operands, takes the bracket flag, and
`Visit::{Enter, Between, Exit}` carry `T` only.
*Why.* A public shape is frozen at 0.1.0 (D17, D18) and R245 asks for
n-ary children, a binder stop and a no-recursion test; a session cannot
choose this by itself. Left open, the harness writes its own explicit-stack
walk and the "no copy of the library's walk" condition fails.
*Smallest change.* Fix in 2.4/3.6: `Walk::over(&Sequent, TermId)` and
`Reading::walk(OccId)` (concrete, no closure), `Visit<T>` `#[non_exhaustive]`
(P3 already implies it) with `Enter(T, nested)`, `Between(T, index: u32)`,
`Exit(T, nested)`; a leaf yields `Enter` alone; documented that a unary
operator yields `Enter, <operand>, Exit` and a binary one `Enter, a, Between(.., 1), b, Exit`.
38's binder is an `Enter` of `Kind::Forall` with one operand, so no new stop.

### 2.2 `Reading::walk` does not say which ILL connective a stop is (friction)
*Need.* Sympli, Maude ILL and llprover take `Γ ⊢ C` with `⊸ ⊗ & ⊕ ! 1 0 ⊤`.
*Design.* 3.6 and 3.10 keep `Reading::{position, implication, formula}`;
`IllFormula` prints linlog's syntax only. The stored term of an input-side
`⊗` is a `⅋`, of a `⊕` an `&`; a `⅋` is `⊸` only if `implication(o)` says so.
The `(kind, position)` to ILL connective table is nowhere public.
*Why.* Without it every ILL translator repeats `Reading`'s case analysis. The
copy drifts silently when 36 changes the reading (divisions), and R245's
"intuitionistic walk" is unmet.
*Smallest change.* `Reading::walk` yields `Visit<OccId>` plus a method
`Reading::connective(o) -> Ill`, with `#[non_exhaustive] enum Ill { Atom(Atom), One, Zero, Top,
Bang, Tensor, With, Plus, Lolli }` (the connectives of the intuitionistic reading).
It is the same table `IllFormula`'s `Display` already uses.

### 2.3 Ground first-order atoms are propositional to every driver (friction; silent wrong answer after 38)
*Need.* A translator prints `Sequent::atom_name(a)`; a driver decides `Covered`
from `Sequent::fragment()` (R75).
*Design.* Decision 1 (3.2): an atom is an interned atomic formula;
`atom_name(a)` is "the name of the atom's predicate symbol"; ground
first-order input has fragment `MLL`, "which is true of its logic"; the crate's
own printers use a crate-private atom writer.
*Why.* After 38, `p(a)` and `p(b)` are two atoms with one `atom_name`. A
translator written at 29 prints both as `p` and sends the tool a different
problem, and `fragment()` shows nothing, so no driver guard fires. The
comparison would record a tool's answer to the wrong question. The shipped
translators outlive 29.
*Smallest change.* Place at 28: `Sequent::atom_arity(Atom) -> u32` (0 always
now) and `Sequent::has_structured_atoms() -> bool`, and make the one atom
writer public as `Sequent::atom_label(Atom) -> impl Display` (the harness
renames the result). Document that `atom_name` is not an identity on atoms.
`Untranslatable::StructuredAtom` is the refusal.

### 2.4 The memory bound is not the process's bound (friction)
*Need.* R18: every tool under the same limit, linlog inside the cgroup that
BenchExec sets; "stays at or under `--memory-limit` plus a stated constant".
*Design.* 5.1 and 5.4: one account for the race (good), but not counted: the
forest (about 25 bytes an occurrence, up to `occurrences` = 50 000 000), the
sequent, the proof returned, stacks, the allocator. A call's peak is
`memory_bytes` for its largest phase plus those. No public number gives the
uncounted part.
*Why.* Without it the harness chooses a margin by trial, and a linlog row
killed by the cgroup reads as a linlog failure where another tool is "merely
slow" (R18's own words). The step also has to show the 14 large nets stay inside.
*Smallest change.* Public `Forest::BYTES_PER_OCCURRENCE` (the constant R63's
assertion already needs) and `Sequent::heap_bytes()`; and an additive
`Statistics::peak_held_bytes`, so the harness can print the accounted peak
beside BenchExec's `peak_kb` and state the constant measured.

### 2.5 `Settings` cannot name "the command's default" (friction)
*Need.* The second linlog column is "its default", which must be what the
command does with no flags (R85, R172), and R190 needs the configuration
recorded.
*Design.* 6.5 says `Settings::default()` is the command's behaviour, yet 6.2
and the JSON of 7.3 give `jobs` default `1`, and 5.4 says "below three threads
there is no race". Today's command, with no `--jobs`, takes every thread the
machine runs after 100 ms (`argument_parsing.rs` 195-208, `cli.md` 281-284).
`race` takes `threads` as an argument, so it is not in any settings value.
*Why.* The stored `settings` of the default column would show `jobs: 1`,
which is the one-thread column. A reader of the page could not reproduce the
column from it. A change of the command's thread rule would also not show.
*Smallest change.* `jobs: Jobs` with `"auto" | n` (P7 already says automatic is
`"auto"`), `Settings::default()` `auto`, `search::Options::default()` 1 (the
same split as `copies`, decision 12); the front end resolves `auto` to a count
once and `--deterministic` stays `jobs = 1`. The outcome's `statistics` or a
`threads` key then records the resolved count.

### 2.6 An outcome does not say whether its proof was checked (friction)
*Need.* R192 and the matrix claim "every proof checked"; rows made with
`--no-check` must be distinguishable. Under BenchExec the CLI JSON is the only
record.
*Design.* 7.3's `Outcome` and 8.1 (`options.check`; off, a `debug_assert!`)
carry no flag; `Proof` does not record it either.
*Why.* A published `proved` row from `--no-check` is indistinguishable in the
data from a checked one, and a comparison claim rests on that.
*Smallest change.* `Outcome::checked: bool` and a key `"checked"` (additive,
written whenever `verdict` is `proved`). It also lets the CSV's `checked` column
be read from the library instead of derived by the harness.

### 2.7 Names of reasons and refutations are only on the wire (note)
*Need.* The CSV `reason` column and the contradiction list (R2, R187) print
`reason.kind` and `refutation.kind`.
*Design.* 7.4 lists the strings and says a test checks "each row against the
code", but gives `name()` and `NAMES` only for `Engine`, `Mode`, `Rule`, `Node`.
*Why.* The harness would serialize to JSON to read a tag, or match on the open
enums with a wildcard that mislabels the next variant.
*Smallest change.* `Reason::name()`, `Refutation::name()`, `Verdict::name()`
and `NAMES`, used by the serializer, so the wire and the CSV cannot differ.

### 2.8 Ground truth: the design promises a check the step cannot have yet (friction)
*Need.* "Contradictions between tools listed", decided by a checked proof or a
countermodel (research 2.5; R187).
*Design.* 10.1 says "Proved and unprovable outcomes are checkable ground
truth (a `Proof`, a `Disproof`)". But `Disproof::check` is [31] (3.12, 10.3),
after 29, and `Refutation::Exhausted` (most focused-engine refusals) has no
certificate even then.
*Why.* A row where linlog says unprovable and a tool says proved is exactly
the contradiction the page must settle, and only linlog's `proved` side is
checkable. The sentence in 10.1 would be published as stronger than it is.
*Smallest change.* Correct 10.1: only `Proof` is checkable at 29; an
`Unprovable` is "unchecked" in the contradiction list unless its refutation
is one of the three recomputable counts. Optionally order `Disproof::check`
for those three before 29. No signature changes.

### 2.9 The outcome names a version, not a commit or options (note)
*Need.* R190: tool versions, linlog's commit, commands, limits recorded by the run.
*Design.* Decision 13 and 7.3: `"linlog": "0.1.0"`, no options in the outcome;
the front end records its settings (placed: 6.6 last paragraph).
*Why.* Before 0.1.0 is tagged many commits carry the same crate version, so the
page's `tool_version` for linlog cannot come from the outcome. R190's Met-when
(version and options in `prove --format json`) is deliberately not met.
*Smallest change.* None to the library: the harness takes the commit from the
flake revision and writes `env.json`; `linlog --version` printing the commit is
the command area's (not placed in the design). Mark R190 "met with deviation".

### 2.10 The race merges statistics two ways (note)
*Need.* R85: the harness row for the default column equals the command's.
*Design.* 5.4: "the statistics are the deciding search's"; 8.5:
`Statistics::add` "the race and a batch merge with it". Today's `alone_first`
sums `nodes`, `splits` and the rest of both searches.
*Why.* The default column's counters change meaning at 28 (decider only), so
older baselines' default rows are not comparable by counters, and 5.4 and 8.5
disagree.
*Smallest change.* Say in 5.4 that `add` is the batch's and the race's
documented rule is "the decider's"; the comparison compares verdicts and
times for the default column, never its counters.

### 2.11 The reading's new refusals can drop LLTP ILL rows (note)
*Need.* The set is the LLTP library's intuitionistic problems.
*Design.* 3.6: with the sides known every antecedent must read as input, no
symmetric reading inside implications, `Succedents` otherwise. `lltp::read`
now sets `antecedents`.
*Why.* A problem the 17-assessment run decided could become `refused` for
linlog and then be "outside the fragment" for nobody. The design says the
verdicts of the pinned witnesses only.
*Smallest change.* None. Step 28's lock commit counts the LLTP ILL verdicts
before and after on the 1 342 problems, and 29 starts from that number.
(C1's root-order change also moves the target set's counters; 3.1 handles it.)

### 2.12 The time limit's start is unspecified (note)
*Need.* R184: BenchExec times the whole process; linlog must not stop itself
earlier by a clock that starts late or early.
*Design.* 6.3's `Clock::time_limit_ms` is "applied by the front end's stop";
the instant it counts from is not stated (the command counts from its start,
the harness from `loaded`). The flag spelling for "no limit" with
`--timeout` is not shown.
*Smallest change.* One sentence on `Clock`: counted from the front end's
start, `null` = none; the CLI row in 6.6 gets `--timeout none`.

### 2.13 Admission needs finer facts than `Fragment`, and the engine table is prose (note)
*Need.* Which tool covers which problem: `⊤` apart from `0`, `!` apart from
`?`, `1` apart from `⊥` (research 3); the matrix row "fragments and modes per
engine" (R217).
*Design.* 3.5 keeps five bits; 8.3/8.6 hold each engine's fragments and modes
as a rustdoc table and a crate-private `admits`.
*Why.* The driver does its own pass over `terms()` (works: `Kind` is public
and closed, so a new kind is a compile error), and the matrix row is typed
by hand and can drift.
*Smallest change.* Additive: `Sequent::kinds() -> KindSet`, and
`Engine::decides() -> (Fragment, &'static [&'static str])` read by `admits` and
the docs. Neither blocks.

## 3. Register entries naming step 29

| entry | status |
|---|---|
| R2 outcome JSON complete, refutation named | met, 7.3 (`refutation.kind`, `Disproof`); the goal-proof half by 3.7 (`goal`). Name strings: 2.7 |
| R18 one memory bound, equal for every tool | partly: one race account (5.4), uncounted part stated (5.1) but not exposed: 2.4 |
| R74 recover the written structure | met: `antecedents` (3.1), written order (C1), `lltp::read` sets it. Names: `HYPHEN`/`DOT` unchanged. Structured atoms: 2.3 |
| R75 fragment and mode exposed, "outside" distinct | met for linlog (`Fragment::NAMED`, `Mode::NAMES`, `ErrorKind::Unsupported`, 8.6); finer kinds: 2.13 |
| R85 the race lives once | met, 5.4 (`search::race`); `Settings` cannot name its default: 2.5; statistics: 2.10 |
| R151 one thread first-class | met, 5.3 and 6.6 (`--deterministic`: jobs 1, `Turns`, no pool); the harness maps to the same keys |
| R172 one `linlog prove` call suffices | placed in 6.6 and 7.3; needs 2.5, 2.6, 2.12. Mode from the directory stays the harness's (`-i` flag) |
| R184 timed like the others | harness and CLI; the library offers the whole-process route; 2.12 |
| R185 tools are rows | harness only; nothing in the library; `tool_version` for linlog: 2.9 |
| R186 wall, CPU, peak memory | harness (BenchExec); linlog's accounted peak: 2.4 |
| R187 disagreements listed | harness; ground truth: 2.8 |
| R188 deterministic shards | harness; `Problem::name` and written order make a stable list |
| R189 CI job | not the library's; the CLI and the harness build with `parallel` (needs the race, 5.4) |
| R190 environment record | partly: `Settings` JSON (7.3) and the version; commit and options not in the outcome (decision 13): 2.9 |
| R191 plots from the CSV | harness; `Reason::name`: 2.7 |
| R192 the checking is stated | partly: check on by default (8.1); not visible in the outcome: 2.6 |
| R193 qcover in the set | `mist::read` takes `&Limits` (5.5), sides by its own count (3.1); other tools' `.spec` translators are the harness's |
| R217 linlog's claims stated once from the code | partly: `Fragment::NAMED`, `Mode::NAMES`, `Engine::ALL`, `wire` docs; per-engine fragments only as prose: 2.13 |
| R245 public walk and ILL walk | placed in name only: 2.1, 2.2 |

## 4. What fits well

- `Sequent::antecedents` with the written order (3.1, 3.6) is exactly what the
  translators and the contradiction list needed: LLTP's axioms and
  conjecture come back without a second parser, and `Eq` on `Sequent` makes
  the "re-reads to the same problem" test one assertion.
- The error kinds (4.1), `Settings`, `--deterministic` and `search::race`
  (5.4, 6.6) give the harness one vocabulary for "refused / unknown / error"
  and one place for the default's behaviour; BenchExec can run the command as is.
- Closed `Kind` and `Term` mean a translator's `match` without a wildcard
  turns each new connective at 34 and 38 into a compile error instead of a
  silently wrong problem sent to another tool.
