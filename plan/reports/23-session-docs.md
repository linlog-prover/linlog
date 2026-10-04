# Step 23 report: what every session reads, short and true

Status: done, in one session (Opus 5.5 at high, unattended overnight on
2026-10-04).

## Outcome

- **The core rules are split by module.** `.claude/rules/core.md`, 2 621
  lines and 28 340 words that loaded whole for any file under `core/`,
  is now a short index (the crate's layout, a map of the public API, the
  two rules that hold for the whole crate, and a table of the module
  files) and ten files, one per module, each loaded beside the index for
  its own paths. A session that reads a file of the SVG export now loads
  5 423 words of CLAUDE.md and rules where it loaded 32 081; one that
  reads the focused engine, the largest area, 11 466. The move is word
  for word: every paragraph and bullet of the old file is in exactly one
  new file (the check is below), and the split is a commit of its own,
  so the corrections that follow are a diff of their own.
- **CLAUDE.md keeps what every session needs**: 2 142 words and 212
  lines where it had 3 741 and 392. The tour of the API, of the
  command's flags, of the harness and of the flake's modules went to the
  rules files that load with that code, where they did not already say
  it; CLAUDE.md has the project in brief, a table of the rules files,
  the commands with the verification table, the hard rules of version
  control, the conventions and the setup.
- **The stale claims** of the assessment's section 1.1 that were in the
  rules files and CLAUDE.md (twelve, counting the devshell's menu and
  the `update-deps` skill), and five more found on the way, are
  corrected against the code (the list is below).
- **README's examples run in a test**: `cli/tests/readme.rs` runs every
  command of every `console` block of `README.md` against the binary
  and compares its output with the block, so `cargo test --workspace`
  and the flake's `test` checks run them, and no step runs them by hand
  again. Of README's 65 `linlog`, `echo` and `cat` commands in 31
  blocks, 64 run: 53 are compared whole (times read as `…`), 10 by
  their shape (every number as `#`), one by its first line. The one
  left is in a block of its own that reads the LLTP library's largest
  file, and the five commands of `linlog-bench`, `pdflatex` and `rocq`
  are left to the flake checks that cover them.

## The new rules files

All under `.claude/rules/`, loaded when a file under their paths is read.
Several files may load for one path: `core.md` always loads beside a
module file, and `search/net.rs`, which holds `net::parallel`, loads both
`core-nets.md` and `core-parallel.md`.

| file | paths | what it covers | words |
|---|---|---|--:|
| `core.md` | `core/**` | the crate's layout (now with every module), a map of the public API naming the file of each entry point, the rule that nothing recurses over a formula, the doc examples' fences, and the table of the files below | 919 |
| `core-sequents.md` | `core/src/sequents/**`, `parse/**`, `fragment.rs`, `serialize/**`, `core/tests/parse.rs`, `serialize.rs` | the arena and its order, a name is an atom, negation normal form, terms, kinds, fragments and modes, the parser, the JSON form of every value | 1 692 |
| `core-forest.md` | `core/src/occurrences/**` | the occurrence forest (numbering, the bound on its size, the atom bias, literal lists), `OccSet`, the intuitionistic reading | 1 519 |
| `core-proofs.md` | `core/src/proofs/mod.rs`, `check.rs`, `oracle.rs` | proof terms, the checker: its pass, its memory bound, its integers, the zone rule, the one-succedent condition | 2 502 |
| `core-derivations.md` | `core/src/proofs/derivation.rs`, `size.rs`, `fmt.rs`, `multiset.rs`, `interactive.rs` | the derivation view, the size estimate, `ViewOptions`, the compact view, the text tree, interactive proving with its API | 3 381 |
| `core-search.md` | `core/src/search/mod.rs`, `memory.rs`, `additive.rs` | the front door, refutations, the dispatch, where every engine polls its stop, the memory bound, the additive path | 3 069 |
| `core-focus.md` | `core/src/search/focus/**`, `core/src/search/generate.rs` | the focused engine, one- and two-sided, with the memo's layout, which the memory section held before | 8 405 |
| `core-nets.md` | `core/src/nets/**`, `core/src/search/net.rs` | proof structures, the criterion, sequentialization, the net engine | 2 317 |
| `core-parallel.md` | `core/src/search/parallel.rs`, `focus/parallel.rs`, `search/net.rs` | the pool, stops, cube-and-conquer, the shared memo and arena, the net engine's cubes, what a pool promises | 2 963 |
| `core-export.md` | `core/src/export/**`, `core/src/proofs/style.rs`, `core/tests/export.rs`, `core/tests/snapshots/**` | the entry points (from CLAUDE.md), options values, the one signature, labels, notations, the packages' limits, the own Typst layout, fonts, SVG, PNG, PDF, Rocq | 2 362 |
| `core-inputs.md` | `core/src/lltp.rs`, `core/src/families.rs` | the LLTP reader and the generated families | 406 |
| `flake.md` | `flake.nix`, `flake.lock`, `modules/**` | what each flake module holds and checks (from CLAUDE.md) | 305 |

`ci.md` took CLAUDE.md's paragraph on what the workflows run, `cli.md`
a bullet on the README test and the `--jobs` note, and
`claude-infra.md` lists every file and says that several may load for one
path.

The order of the old sections is kept inside each file but for one:
`core-forest.md` has the forest before the reading, which builds on it.
Two moves place a point where its code is: the paragraph "Nothing
recurses over a formula" and the doc examples' fences, which hold for
every module, are in `core.md`; the bullet on the memo's layout
(`focus/memo.rs`) left the memory bound's section for the focused
engine's, beside the memo's contract. A point two areas need lives in
one file and the other names it: the atom bias is the forest's
(`core-forest.md`) and `core-focus.md` names it; where every engine
polls is the front door's (`core-search.md`) and the engine files name
it.

## Nothing was lost

The check, run on the split commit (`Split the core rules file by
module`) and again at the end: the old file and the new ones are cut
into items, a paragraph or a bullet with its continuation lines, and
each item of the old file (the front matter and the old introduction
aside) is looked for, with its line breaks joined, among the items of
the new files.

- At the split commit: 225 items, each found in exactly one new file,
  none twice and none missing.
- At the end: 215 found in exactly one file, word for word; the other
  ten are the items this step corrected (the next section), each in one
  file.

## The claims corrected

Each was checked against the code at the step's start.

From the assessment's section 1.1:

1. CLAUDE.md, "parse, print, serialize and eventually prove": the
   suite proves; the project paragraph is rewritten.
2. CLAUDE.md, a "README roadmap": README has "What exists and what is
   planned", which CLAUDE.md now names.
3. CLAUDE.md and `core-export.md`, the source is "what
   `cleanCargoSource` keeps": `modules/workspace.nix` uses crane's
   `commonCargoSources` (now in `flake.md` and `core-export.md`).
4. CLAUDE.md, the lint allowance "while things are scaffolded": the
   reason now given is the one the assessment found (items live under
   one feature only, test-only items, a few dead ones), for the audit.
5. CLAUDE.md and `bench.md`, compare "with `after.csv` when both runs
   name a `--bias`": `bench/targets.sh` takes a label and passes no
   bias, so both now say the script's runs take the default.
6. CLAUDE.md, `bench/TARGETS.md` "compares `before`, `after-search` and
   `after`": it has five labels; CLAUDE.md now only names the file.
7. `cli.md`, the Ctrl-C handler "is installed by `prove` only":
   `catch_interrupt` is called by `prove` and `interact`.
8. `cli.md`, a new output format's places: the list now adds `render`
   for a binary format, the `Compact::Never` choice in `Show::new`, and
   the two matches with a wildcard that take a new format silently.
9. `bench.md`, "the 22-field tail" (the assessment saw 16 and 21):
   `run::TAIL` is 25.
10. `bench.md`, the reason `context_too_wide`: gone with its `Reason`
    and read only in older files; the list now has `memory_limit`,
    `index_limit` and `other`, which `bench/src/run.rs` writes.
11. `core-forest.md`, the atom bias's "only reader, through
    `polarity(o)`" against "never off `Forest::polarity`" in the same
    bullet: `Forest::polarity` has no caller outside the forest's tests;
    the engine reads `Forest::bias_under`.
12. The devshell's menu (`modules/devshell.nix`, the help line of
    `check`) and the `update-deps` skill listed the flake's checks
    without `export`, `rocq`, `bench`, `claude-hooks` and the debug
    assertions' test run.

"Euler everywhere" and "SVG text, say" were no longer in `core.md`
(step 22 corrected them), and CLAUDE.md's "`nix flake check`, which runs
all of the above" became true with step 18's `test-debug-assertions`.

Found on the way:

13. `core-focus.md` and `core-parallel.md`: "as the CLI does every 1 024
    [polls] on one thread" and "a caller that reads the clock every `n`
    polls (the CLI does, on one thread) must read it every poll on
    several (`polls_per_clock` in the CLI)". The command reads a flag
    that a timer thread raises (`cli/src/limit.rs`), and
    `polls_per_clock` exists nowhere.
14. `core-search.md`, "Whether `net` stays the default anywhere else is
    for the second baseline's `engines` runs to say": the second
    baseline said it (`bench/COMPARISON.md`, "Focus against net"); the
    bullet now says what it found and where the dispatch's change is
    planned (`plan/later.md`).
15. `core-nets.md`, the leaf symmetry break "is the follow-up the plan's
    step 14 should measure": it is a follow-up in `plan/later.md`.
16. `core.md`'s layout omitted `errors`, `oracle`, `size`, `style`,
    `memory`, `parallel`, `classes`, `context`, `png`, `pdf`, the SVG's
    modules, `lltp` and `families`.
17. `core-derivations.md`, "Step 5 reads axiom links off the `ax`
    inferences": `ProofStructure::from_proof` reads them off the `Ax`
    nodes. `core-proofs.md`, "the same terms serve step 8": they serve
    intuitionistic mode.

Added where only CLAUDE.md said it: the `Interactive` API
(`core-derivations.md`), the exports' entry points (`core-export.md`),
the `--jobs` note (`cli.md`), and a map of the public API (`core.md`).

## README's examples as a test

`cli/tests/readme.rs` reads `README.md` when it is built
(`include_str!`), takes every block fenced as `console`, and runs each
`$ ` line through `sh -c` in a directory of the block's own under
`CARGO_TARGET_TMPDIR`, with the directory of the binary under test first
on `PATH`, so pipes (`| linlog check`, `| head -12`) and redirections
(`> verdict.txt`, `cat verdict.txt`, `echo … > style.json`) work as they
read. Standard output and standard error go to one pipe, in the order
the command wrote them, and are compared with the lines that follow the
command; lines starting with `> ` are its standard input (the
`interact` transcript). Every time after "after " or "time: " is read as
`…`.

A comment on the line before a block says how much of it is the
machine's, invisible on GitHub:

- `<!-- readme-check: machine -->` compares the shape, every run of
  digits as `#`: the blocks with a copy bound reached under a time limit
  (512, 18, 12) and a thread count (`-j 10000`'s note). Four blocks.
- `<!-- readme-check: terminal -->` compares the first line, the
  verdict: the block that shows what a terminal 30 columns wide gets.
- `<!-- readme-check: skip, … -->`: the block that reads
  `SYJ212+1.020.txt`, 86 MB of the LLTP library, which I split from the
  `--timeout` block before it so that the first command of that block
  is still checked.

A file a command writes must exist when the command names it with
`--output`, and every file of a known kind in the block's directory
must be of that kind: PDF by `%PDF-`, PNG by its signature, SVG as an
`<svg>` element, LaTeX by `\end{prooftree}`, Rocq by `Qed.`, JSON by
parsing. Its bytes are not compared, and `SOURCE_DATE_EPOCH` is not set.
The commands of `linlog-bench`, `pdflatex` and `rocq` are skipped by
name (`OTHER_PROGRAMS`); a command of any other program fails the test,
so a new kind of example is a decision. `shared.json`, which README
reads and does not make, is written by `fixtures`, and its length is
asserted to be README's 427 bytes. One README example gained
`--deterministic` (`--engine focus --stats --quiet`), whose counts are
pinned. `modules/workspace.nix` now keeps `README.md` in the crane
source.

Checked by mutation before the commit: a changed line of a `seq print`
output and an SVG written to `proof.png` each failed the test with the
README line, the expected and the actual output. A run takes 5.4 s
(debug build): the four examples that end at the two-second default time
limit or the one-second limit dominate.

## The words a session loads

What loads when a session reads one file, CLAUDE.md and the rules files
whose paths match, before (at the step's base, `trtmxyyk`) and after.
The global `~/.claude/CLAUDE.md` and the memory index load in both and
are left out.

| file read | before | after | rules files after |
|---|--:|--:|---|
| `core/src/sequents/mod.rs` | 32 081 | 4 753 | `core-sequents.md`, `core.md` |
| `core/src/parse/mod.rs` | 32 081 | 4 753 | `core-sequents.md`, `core.md` |
| `core/src/serialize/proofs.rs` | 32 081 | 4 753 | `core-sequents.md`, `core.md` |
| `core/src/occurrences/mod.rs` | 32 081 | 4 580 | `core-forest.md`, `core.md` |
| `core/src/proofs/check.rs` | 32 081 | 5 563 | `core-proofs.md`, `core.md` |
| `core/src/proofs/derivation.rs` | 32 081 | 6 442 | `core-derivations.md`, `core.md` |
| `core/src/proofs/interactive.rs` | 32 081 | 6 442 | `core-derivations.md`, `core.md` |
| `core/src/search/mod.rs` | 32 081 | 6 130 | `core-search.md`, `core.md` |
| `core/src/search/additive.rs` | 32 081 | 6 130 | `core-search.md`, `core.md` |
| `core/src/search/focus/mod.rs` | 32 081 | 11 466 | `core-focus.md`, `core.md` |
| `core/src/search/focus/parallel.rs` | 32 081 | 14 429 | `core-focus.md`, `core-parallel.md`, `core.md` |
| `core/src/search/net.rs` | 32 081 | 8 341 | `core-nets.md`, `core-parallel.md`, `core.md` |
| `core/src/nets/graph.rs` | 32 081 | 5 378 | `core-nets.md`, `core.md` |
| `core/src/export/svg/mod.rs` | 32 081 | 5 423 | `core-export.md`, `core.md` |
| `core/src/families.rs` | 32 081 | 3 467 | `core-inputs.md`, `core.md` |
| `core/src/lib.rs` | 32 081 | 3 061 | `core.md` |
| `cli/src/prove.rs` | 7 315 | 6 043 | `cli.md` |
| `bench/src/run.rs` | 8 126 | 6 551 | `bench.md` |
| `modules/checks.nix` | 3 741 | 2 447 | `flake.md` |
| `.github/workflows/ci.yml` | 4 000 | 2 462 | `ci.md` |
| any other file (README, plan) | 3 741 | 2 142 | none |

Words are `wc -w`'s. The first file of a module costs this once per
session; a second file of another module adds that module's file.

## Decided unattended

- **Flat names, `core-*.md`, not a directory `rules/core/`.** Claude
  Code reads rules files from subdirectories too, but the flat names
  match the existing files and leave no doubt about discovery.
- **An index that loads for every core file.** The alternative, no
  catch-all, would leave `lib.rs`, `errors/` and `hash.rs` without rules
  and the crate-wide rules without a home; at 919 words it costs little.
- **Ten module files.** The focused engine (8 405 words) could be cut
  again (the default bias's two searches are 190 lines of it), but the
  prompt names it as one area, and a session on the engine needs both.
- **`search/net.rs` loads the parallel rules**, since `net::parallel`
  is inside it; the alternative, a path the rules cannot express, would
  leave the cubes' rules unread there.
- **`machine` compares the shape of every line, not only the verdict
  line.** The prompt asks for the verdict line's shape; the other lines
  of those blocks are fixed text (the second verdict line of the
  `-j 10000` example) and comparing them costs nothing. If a machine
  makes one of them flaky, narrowing `shape` to the first line is a
  one-line change.
- **Commands run through `sh`, with both streams merged.** The
  alternative, parsing pipes and redirections in the test, would be a
  shell of its own; `sh` is in the nix sandbox and on every machine.
- **Other programs are skipped by name, not by marking their blocks**,
  because one block mixes `linlog` (checked) with `pdflatex` (skipped).
- **The fixture is generated, not a file**, so no fixture directory
  joins the crane source, and its size checks README's sentence.
- **CLAUDE.md is 212 lines**, over its maintainer note's "well under
  200": what remains is the commands, the verification table, the jj
  rules and the conventions, which every session needs, and the table
  of rules files. The flake's modules and the CI details went to
  `flake.md` and `ci.md` to get there.
- **The devshell's menu and the `update-deps` skill** are not rules
  files, but their lists of checks were stale claims that sessions read,
  so they are corrected in the same commit (a help string, no
  behaviour).
- **README's `--engine focus --stats --quiet` example gained
  `--deterministic`**: its counts would be a pool's if the first thread
  ran past 100 ms, and the prompt asks pinned blocks to name what they
  need. No other example needed a flag.

No library feature was added, so D15, D16 and D17 have nothing to apply
to; the check has no options beyond its markers.

## Verification

Every run in a capped scope or unit (8 GiB, cores 4 to 9,
`CARGO_BUILD_JOBS=6`, `RUST_TEST_THREADS=4`):

- `cargo clippy --workspace --all-targets -- --deny warnings`: clean.
- `cargo test --workspace`: every test passes, 160 in the core crate's
  unit tests (2 ignored, as before), and `readme_examples` in 5.45 s.
- `nix flake check --max-jobs 2 --cores 6` (the nix daemon builds, so
  the cores are its options, not the scope's): exit 0, every check of
  `x86_64-linux`, with `readme_examples` passing in the `test` and the
  `test-debug-assertions` builds inside the sandbox (5.15 s each).
- The coverage of the old rules file, as above; the README test's
  failure path by two mutations, as above.

No `cargo hack` run: no `cfg` or feature changed.

## Left open

- **The prompts' boilerplate** ("How to work on this step") tells a
  session to record notes in `.claude/rules/core.md`. With the split it
  should say "in the rules file of the module" (the index says so too);
  that text lives in `plan/README.md` and the prompts, which the review
  finishes.
- **The stale claims outside the rules files and CLAUDE.md** stay for
  step 28, as the step says: README's "returning a checked proof" and
  the `--help` texts the assessment lists were not re-checked here,
  except where the README test now pins an example.
- **The README check assumes a machine of four threads or more**: with
  fewer, `-j 4` prints a note that README does not show, and the
  `-j 10000` note says "thread" for one. CI's runner has four.
- **The `bench` check fails on a `MISMATCH` only**, as the assessment
  found; `flake.md` says what it fails on, and the fix belongs to the
  harness's audit.

## Commits

Five commits, all signed (the passphrase's cache held throughout, and
`/tmp/linlog-step23-unsigned` never appeared): "Run README's examples
in a test", "Split the core rules file by module", "Correct the stale
claims of the rules files", "Keep in CLAUDE.md what every session
needs", and this report. No commit needs `jj sign -r
'main@origin..@-'` in the morning; it would do no harm.

## From the review (2026-10-04)

Accepted as it is. The planning session read the report, the split, the
corrections, the new CLAUDE.md and the README test, and ran clippy, the
tests with and without `parallel`, both `cargo hack` runs, `cargo deny`
and `nix flake check`, all passing.

- **Nothing lost, checked independently**: the old file cut into its
  212 paragraphs and bullets, every one is in exactly one file at the
  split commit (the title and the introduction aside), and at the end
  all but the ten this step corrected, which are the ten the report
  lists.
- **The corrections**, spot-checked against the code: `bias_under` is
  the engine's one reader of the bias, `close_with` and
  `derivation_ids` are the session's API as written, `run::TAIL` is 25,
  and `bench/COMPARISON.md` says what `core-search.md` now cites.
- **The paths**: every file of the core crate loads its module's file
  besides the index, except `lib.rs`, `errors/`, `hash.rs`,
  `Cargo.toml` and `tests/depth.rs`, for which the index's crate-wide
  rules are the right ones.
- **The README test is not timing-bound**: run with the release binary,
  every example compared whole decides in 20 ms or less; the three that
  run into a time limit (2 s, 2 s, 1 s) are the blocks marked `machine`.
- **Fixed in the review** (the report's first open item): `conduct.md`
  told every session to record notes in `.claude/rules/core.md`; it now
  names the module's rules file. Prompt 31 and four passages of
  `plan/later.md` pointed at sections that moved, and now name
  `core-proofs.md`, `core-export.md`, `core-focus.md` and
  `core-parallel.md`. Earlier prompts and Status entries keep the name
  they were written with.
- **The unattended run** took 25 minutes; the passphrase's cache held,
  so every commit is signed, and nothing waited on an answer.
- Not run: the library sweep and the target set, since no code but a
  test and a help string changed.
