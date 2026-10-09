# Walk-through of step 30 (baseline and release 0.1.0) against `plan/notes/api.md`

Read: the design (all of it), `plan/30-baseline-release.md`, the research
note `30-release.md`, `distribution.md`, the register entries naming
step 30 (R6, R19, R41, R50, R150, R162, R194 to R197, R206, R215, R218 to
R224, R232 to R236, R252, R253), `core/Cargo.toml`, `core/src/lib.rs`,
`.claude/rules/core.md`, `bench/baseline.sh`, `.claude/rules/bench.md`, the
parser's keywords and the list of public types in `core/src`.

Step 30 is mostly process; the design touches it where the release turns
the surface into promises. The first change is therefore two or three
commits that add no behaviour and freeze what 28 produced.

## 1. The first change, sketched

**Commit 1, "Release metadata"** (manifests and crate docs, no code):

```toml
# Cargo.toml
[workspace.package]
version = "0.1.0"           # core, cli, bench: version.workspace = true (R235)
edition = "2024"
rust-version = "1.88"       # measured, not guessed (R252); not in the design
license = "EUPL-1.2"
repository = "https://github.com/linlog-prover/linlog"
homepage = "https://linlog-prover.github.io/linlog"

# core/Cargo.toml: readme, keywords (<= 5), categories ["science", "mathematics"],
# exclude = ["tests/export.rs"-style tests that read ../cli/fonts] (R232),
[package.metadata.docs.rs]
all-features = true
targets = ["x86_64-unknown-linux-gnu"]

# cli/Cargo.toml: license = "EUPL-1.2 AND OFL-1.1", own description,
# categories ["command-line-utilities"]; cli/tests/readme.rs excluded or fed a copy.
```

Plus `core/LICENSE`, `cli/LICENSE`, `cli/fonts/OFL.txt` in the package, and
in `core/src/lib.rs` the "Features" paragraph rewritten to list exactly the
ten features of `[features]` with "none is removed within 0.1" (R218), a
"Stability" section (what is API, what is not), and the `#![allow(dead_code)]`
pair narrowed (R234). `linlog_cli`'s crate doc says it carries no semver
promise (2.3, R253).

**Commit 2, "Changelog and the stability tests"**. `CHANGELOG.md` (Keep a
Changelog), the 0.1.0 section opening with a policy block that the design
supplies almost verbatim (10.2, P3, 7.1):

```text
## [0.1.0]
### Policy
- Cargo's reading of 0.y: a break is 0.(y+1).0. Closed on purpose (a new variant is a
  compile error downstream): Term, Kind, Node, Rule, Verdict, Side, Sign, Polarity,
  Position, Compact, Sides, Form, Cores, pdf::Date, ordinary::{Node, Rule, Side}.
  Planned bumps: 0.2.0 at step 34 (Cut), 0.3.0 at step 38 (Forall, Exists).
- Wire forms are promised by level, not by crate version: wire::LEVEL = 1.
- MSRV: <policy>; raised only in a 0.y bump.
- Not API: Display text, the command's output, default numbers, counter values.
### Wire forms
- level 1: <the forms of 7.3>
```

Rust and test side, all new files, no existing signature touched:

```rust
// core/tests/stability.rs  (feature serialize)
// 1. read every file of core/tests/wire/level-1/*.json (one per form, written by
//    the 0.1.0 commit) through Within::<T>::new(&Limits::default()); each must read
//    and, for write-read forms, write back byte-identical.
// 2. core/tests/wire/level-1/names.txt lists Node::TAGS, Rule::ALL names, Error::CODES,
//    Mode::NAMES, Engine::ALL names, Reason/Refutation kinds as of 0.1.0;
//    assert each list STARTS WITH the file's list (append-only).
// 3. core/closed.txt: the closed enums of the changelog; the flake's `conventions`
//    check greps `pub enum` / `pub struct` with pub fields for #[non_exhaustive] and
//    compares with the file (R50's "grep-able list", core.md names the file).
```

**Commit 3, "Release checks"**: a flake check `package` running
`cargo package --workspace --offline --list` / the publish dry run (R236); a
wasm32 check (`cargo check --target wasm32-unknown-unknown -p linlog
--no-default-features --features parse,serialize,interactive,latex,typst,svg,rocq`,
R41); an MSRV check on the `rust-version` toolchain (R252); in `ci.yml` an
online job `cargo semver-checks check-release -p linlog --all-features`
that is skipped while crates.io has no `linlog` (R215); `release.yml` on a
tag (R233); `CITATION.cff`, `rocq/rocq-linlog.opam` draft (R162),
README install section (R224). The baseline (item 1 of the prompt) is
harness work: the design changes only what it runs against (item 11 below).

## 2. Workarounds

1. **friction. R206's net-engine target set does not exist, but 3.11 gates on it.**
   3.11: "the net engine's rows of the target set keep `links` and `tests`
   on one thread, taken before the retype" (the `VertexId` retype, step 28).
   `bench/targets.sh` is the focused engine's set only; R206 ("Not met")
   says 28 *or 30* creates the net set, and the step 30 prompt does not
   mention it. Created at 30 it is taken after 28's retype, so it is no
   "before"; the spike (11.2) measured the focused set and the journeys only.
   Smallest change: 3.11 and section 11 say who captures `links`/`tests`
   (jobs 1, pinned cores, from the `engines` and `period` problems of
   `baseline.sh`) before the retype commit, as a CSV in `bench/targets/`;
   step 30 promotes it to the R206 set and names it in CLAUDE.md's table.

2. **friction. The wire promise has no mechanical form after release (7.1, 7.5).**
   `cargo-semver-checks` sees only Rust (R215); the design's tests (7.4,
   7.5) check the `wire` table against the code and read the *pre-release*
   fixtures. Nothing freezes what 0.1.0 itself writes, nor that `Node::TAGS`,
   `Error::CODES`, `Mode::NAMES`, `Rule::ALL` stay append-only. Smallest
   change: 7.5 gets a fourth item: step 30 commits level-1 golden files and
   the `names.txt` lists (sketch above); every later release reads them.

3. **friction. Added fields break old files unless the rule says `#[serde(default)]` (7.1, 8.5, 3.12).**
   `Statistics` is "`derive(Serialize, Deserialize)`, no proxy" (8.5), the
   refutation payloads and reason variants "can gain fields" (F86). Adding
   a field to a `#[non_exhaustive]` struct is a Rust-compatible change, yet
   a 0.2.0 reader of a 0.1.0 outcome fails on the missing key unless every
   such field is `serde(default)`. R6 requires "a reader of the new version
   reads old files". Smallest change: one sentence in 7.1 (every serde
   struct of a read-back form is `serde(default)` at container level, a field
   added later defaults) plus item 2's golden files as the test.

4. **friction. The text syntax is a form with no policy, and HD5 lets keywords through (3.1, 7.1).**
   Today `par`, `top`, `bot` are keywords (`core/src/parse/mod.rs`); an
   atom named `par` read from JSON passes "is an identifier" (HD5) and
   prints as text that reparses as a connective. Step 36 adds `\` and `/`,
   step 38 `forall`, `exists`, `p(a)` and the author's binder syntax; a
   0.1.0 file with an atom `forall` then changes meaning or fails. R221
   makes the changelog list the surface, and the syntax is not in 7.
   Smallest change: HD5's check (`Error::AtomName`) also refuses the
   keywords and the reserved `forall`, `exists` from 0.1.0 (a refusal now
   costs nothing, a refusal later is a break); 7.1 gets "the text syntax
   grows by new tokens only; the reserved words are listed on `Sequent`".

5. **friction. R41 (wasm32 check) names step 30; the design places it at 32 only (10.2, 10.4).**
   The risk R41 guards ("the published crate does not pin a design the web
   client must undo") is exactly 0.1.0's. The code is clean today (threads
   only behind `parallel`; `batch` runs on the caller's thread without it),
   so the check is cheap. Smallest change: 10.2 lists R41's flake check;
   10.4 keeps the bindings and the 32-bit tests (R46).

6. **friction. Two closed-versus-open choices ship in 0.1.0 without the author's answer (P3, 2.5, 14).**
   (a) P3 closes `Term`, `Kind`, `Node`, `Rule` and plans 0.2.0 and 0.3.0;
   the research (open question 2) leaves it to the author and 14 does not
   list it, yet the changelog's policy line must state it, and the brief's
   aim is no break after 0.1.0. `#[non_exhaustive]` affects downstream
   crates only; inside, P3's `wildcard_enum_match_arm` deny keeps the
   compile errors. Smallest change: add it to 14.2 as a decision with the
   alternative. (b) 2.5 closes *option-valued* enums (`Cores`, `Form`,
   `Compact`, `Sides`, `pdf::Date`) with P3's logic-enum reasoning, but
   D15 says options grow: `Form` is the Rocq output form (R157's open-goal
   export, R158's further targets are plausible new values), `Cores` a
   scheduling policy. A new value is a 0.y bump for no compile-time gain.
   Smallest change: mark these `#[non_exhaustive]`; keep closed only the
   logic enums and `Side`, `Sign`, `Polarity`, `Position`, `Verdict`.

7. **note. 2.5's list is not yet complete (R50: "a grep-able list names every public enum and struct").**
   Public types in `core/src` that 2.5 neither marks nor closes:
   `export::svg::Font` (public fields, not `#[non_exhaustive]` today),
   `mist::Safety` (twin of `lltp::Status`, which is marked),
   `ordinary::Target` (3.13: `{ fragment, mode }`), `ordinary::Outcome`,
   `rocq::Identifier`, typst's `Length`, `Described`, `Within`, the public
   `Walk`/`Visit` (R245). Derived traits are API too and P3 lists none:
   `Statistics`, `Limits`, `Mode`, `Progress` are `Copy`, `Error` is `Eq`;
   a later non-`Copy` counter or non-`Eq` payload is a break. Smallest
   change: step 30 generates the list from `core/closed.txt` (item 2) and
   the design adds one sentence naming the derives that are promised.

8. **note. Provisional values ship in the first release (5.1, 6.5, 3.5).**
   `Limits::BROWSER` and `Settings::browser()` have no caller before step
   32 and their numbers are "provisional until step 32 measures them".
   Decision 16 defers the builder for that reason; the same reasoning
   defers these (both additive). `Fragment::QUANTIFIERS` "reserved" as a
   `pub const` promises a bit nothing sets; documenting bit 32 as reserved
   in the type's doc needs no item. The changelog says default numbers
   (`Limits::DEFAULT_*`, `Clock` defaults) are not promised.

9. **note. The command's surface is undecided between R221 and 10.2.**
   R221 wants the changelog to state "commands and statuses"; 10.2 says the
   command's output is not the API (D18), and `cargo-semver-checks` names
   `linlog` only, so `linlog-cli`'s flags, exit statuses and
   `--format json` have no stated promise. 4.1 adds seven `ErrorKind`s but
   not their exit statuses (today 0 proved, 1 unprovable or invalid, 2
   error, 3 unknown; where `Unsupported`, `Stopped`, `Failed`, `Defect`
   land is the command area's). Smallest change: a table `ErrorKind` to
   exit status in 4.1, and one line in 10.2: subcommands, flags, statuses
   and the JSON levels are `linlog-cli`'s promise under its own version,
   the human text is not.

10. **note. R215 cannot be "met" at 30, and the process after 0.1.0 is unstated (10.2).**
    There is no baseline until the author publishes, so the job exists but
    skips; R215's "passes on main" holds from 0.1.1. Once published, a
    breaking commit (34's `Node::Cut`) fails the job on main until the
    manifests read 0.2.0, so the rule is: the first breaking commit bumps
    the version in the same change. Also `--all-features` builds the rustdoc
    JSON of resvg and krilla (heavy, online only), and `parse`/`serialize`
    items under `--default-features` would be unchecked. Smallest change:
    10.2 states the bump rule and the skip.

11. **note. The third baseline compares against a second one taken before 28's search-input and poll changes.**
    (a) C1 (3.1) stops sorting roots: counters of rows with non-ascending
    roots move, so "comparable" (R194) holds for columns, not always for
    values; the target-set oracle covers 225 rows, not the LLTP passes.
    (b) 5.4 gives `race` one memory account, which changes what the
    `lltp-default` pass measures against the pool-with-queued-tasks case
    the prompt asks about. (c) the progress polls (5.2) were gated on
    instructions, time was not (G3). (d) `baseline.sh` writes
    `--pool-after 0.1` (seconds) where the key is `clock.pool_after_ms`
    100: R196's tie to the constants needs the unit conversion stated in
    6.6. Smallest change: step 28's last commit records a targets run, the
    step 30 report attributes differences to it, and 6.6 gets a unit column.

12. **note. Stamp and register items the design leaves open.** `Outcome.linlog`
    is `CARGO_PKG_VERSION` and reads `0.1.0` in every pre-tag build (the
    manifests already say so, R197), so stored outcomes of development
    builds cannot be told from the release; the baseline's `RESULTS.md`
    names the commit instead. `LIBRARY` of the Rocq export (9.2) is
    defined at 31, so the draft opam file's version at 30 (R162) has no
    value to equal; use the crate's 0.1.0 and let 31 decide. The `parse`
    feature gates syntax, LLTP, `.spec`, families and TPTP together plus
    (HD3) a normalisation dependency; a lean text-only build for 32 is an
    additive new feature that `parse` implies, so no decision is needed
    before the tag, but 3.1 should say `parse` keeps implying all of them.

## 3. Register entries naming step 30

| entry | status |
|---|---|
| R6 compatibility policy | placed: 7.1, 7.5; mechanical freeze and `serde(default)` missing (items 2, 3) |
| R19 pool returns promptly | placed: 5.4 (queued tasks poll before they start, a test at two threads) |
| R41 wasm32 check | not placed at 30 (item 5) |
| R50 non_exhaustive or closed, listed | placed: P3, 2.5; list incomplete, option enums arguable (items 6, 7) |
| R150 batch structs | met: 6.3, 2.5 |
| R162 opam draft | not placed (item 12); `LIBRARY` is 31's |
| R194 CSV comparable | placed: 8.5 appends columns at the end (`work`, `forward_copies`); mode words unchanged except `affine-mix`, which no baseline holds (checked: only `bench/src/problems.rs` writes `mix-affine`) |
| R195, R197 | harness work, untouched by the design |
| R196 new-defaults pass | met by 6.3 constants and 6.6, unit column missing (item 11d) |
| R206 net target set | not placed; 3.11 relies on it (item 1) |
| R215 semver check | placed: 2.3, 10.2; process missing (item 10) |
| R218 metadata, ten features frozen | not placed; no conflict (design adds no feature) |
| R219, R222, R223, R224, R233, R235, R236 | not placed; no conflict |
| R220 docs.rs all features | not placed; `wire` (feature `serialize`) is the schema of record, so all-features is required, which R220 already asks |
| R221 changelog | placed: 10.2 (policy line, wire heading); commands and statuses open (item 9) |
| R232 packaging outside the crate | not placed; the new `core/tests/forest.rs` fixture is read by 31's `rocq/` tests from outside, which is fine since `rocq/` is not packaged |
| R234 dead-code allowance | not placed; reserved crate-private items in the design are comments, not code |
| R252 MSRV | not placed; nothing in the design's signatures exceeds 1.88 (associated-type bounds need 1.79) |
| R253 `linlog_cli` | placed: 2.3 |

## 4. What fits well

Section 2.3 and 10.2 give the release almost its whole policy text (closed
list, planned bumps, levels independent of crate version, semver-checks on
`linlog` only), so the changelog is a transcription. `wire::LEVEL`, the
`Node::TAGS`/`Error::CODES`/`Mode::NAMES`/`Engine::ALL` tables and the
non-exhaustive-by-default rule (P3, 2.5) make the stability tests of item 2
a few dozen lines. The session-and-sequent-free API (public `Limits`,
`Progress`, `race`, constants for the defaults) lets the baseline harness
derive the `lltp-default` flags from the library, as R196 wants.
