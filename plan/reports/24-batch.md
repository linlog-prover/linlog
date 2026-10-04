# Step 24 report: a batch mode, and LLTP input for the command

## Outcome

`linlog prove` decides many sequents in one call. It takes several
`--file` arguments, directories, lists of paths (`--files-from LIST`,
`-` for standard input, `--null` for `find -print0`), files of lines in
the text syntax, JSON Lines, the harness's problem files and LLTP
problems, and a stream on standard input that it answers line by line.
It writes one result per sequent in input order as soon as that result
and the ones before it are decided: a line of text, or a JSON Lines
record with the name first. Drawings go into a directory, and the exit
status is the worst verdict. A single `prove` now reads an LLTP file
(`.p`) and a JSON file (`.json`) by their extensions. The library has
the batch as `linlog::search::batch` (`run`, and `prove` over
`Problem`s) with one options value. It also has a thread pool kept
across searches (`search::Pool`, `Options::pool`), which a batch uses
within its sequents. Renders are bounded before usvg parses anything, by an estimate from the
drawing's glyphs, elements and arcs. A net's drawing has a bound of its
own, and the time limit and Ctrl-C reach a render. The header report is drafted
in `plan/notes/lltp-headers.md`. The batch's verdicts were compared
with single calls on the problem file and on 197 LLTP problems, and no
decided verdict differs. The batch was also timed against a shell
loop: 38 µs a sequent against 1.07 ms.

Commits, in order: "Read a sequent file by its extension: LLTP problems
and JSON"; "Keep a search's thread pool across searches"; "Decide many
sequents in one call in the library"; "Decide many sequents in one call:
a batch of files, directories, lists and streams"; "Refuse a negative or
NaN number of seconds in the harness"; "Draft the report on the wrong
LLTP headers"; "Say once that a file is no LLTP problem"; "Bound drawings and keep the
time limit and Ctrl-C over a render"; this report.

## The options, and how each front end sets them (D15, D16)

| option | library | default | the command | the web front end, another wrapper |
|---|---|---|---|---|
| each sequent's search | `batch::Options::search`, a `search::Options` | the search's defaults | the flags of a single `prove` | the search options once they have a wire form (step 28) |
| the mode of a problem without one | `batch::Options::mode`; `Problem::mode` | classical | `-i`, `-a`, `--mix`; a problem file's column, a record's `mode` | a field per problem |
| where the threads go | `batch::Options::cores`, `Cores::{Auto, Across, Within}` (serde) | `Auto` | `--cores auto\|across\|within`; `auto` is `Within` on a stream from standard input | `Within` in a browser tab (no threads without `parallel`) |
| sequents at once | `batch::Options::workers` | `DEFAULT_WORKERS` = 1 (the library does not ask the machine) | `--workers N`, by default the machine's threads | 1 |
| the batch's memory | `batch::Options::memory_limit` | `DEFAULT_MEMORY_LIMIT` = 4 GiB | `--batch-memory SIZE\|none`, by default half of `MemTotal` from `/proc/meminfo`, else 4 GiB | the library's default |
| each sequent's time | none in the library, which has no clock: the closure `run` takes polls the front end's stop | 2 s | `--timeout` | the page's own deadline |
| the batch's time | as above | none | `--batch-timeout DURATION\|none` | as above |
| a child per sequent | none | off | `--isolate` | a worker per request on the server side |
| the input's kind | none: the command's | by extension | `--input-format auto\|text\|json\|lltp\|lines\|jsonl\|problems` | the page sends sequents as JSON |
| a kept pool | `search::Options::pool(Some(Pool))` | none | set by the batch | not available without `parallel` |

`batch::Options` has no serde yet, because `search::Options` has none.
`Cores` has it. When step 28 gives the search options their wire form,
the batch's options derive it too, and the options file that the step
defers becomes possible.

**Quantifiers (D17).** The batch carries sequents and outcomes and
knows nothing of formulas, so terms change nothing here. A `Problem`
holds a `Sequent`, and a first-order sequent is one too.

## How the batch works

- **Where the threads go.** The user asked mid-step how a batch pairs
  with parallel search. The two levels never nest, so the threads
  never multiply. Across the sequents, each worker searches one sequent
  on one thread with the sequential engines. That is the
  `--deterministic` search, and it builds no rayon pool. The workers
  are plain `std` threads with the search's stack, as many as
  `--workers` and `--batch-memory` allow. Within a sequent there is one
  worker, the caller's thread on the large stack. It runs a single
  call's race: one thread first, then after `--pool-after` a pool of
  the other threads beside it. That pool is the kept `search::Pool`, so
  the pool's threads are started once per batch, not once per sequent.
  The default, `auto`, goes across when the batch has at least as many
  sequents as workers. It goes within when the batch has fewer, and on
  a stream from standard input, where reading ahead would block a
  program that waits for its answer. This is the arrangement that
  throughput-oriented tools use (GNU parallel, make -j, test runners
  that run one test per thread). There, many independent jobs are
  parallelised across jobs, which needs no synchronisation, and a lone
  job gets the machine. Cubes cost nothing across the sequents, and
  every answer is a function of its input. A dynamic scheme, where idle
  workers join the last hard sequents of a batch, would speed up the
  tail of a batch. It would also make each answer depend on timing. It
  is left open below.
- **The order of results** is kept by the library's iterator. A worker
  takes a problem only while it is fewer than `AHEAD` (4) per worker
  ahead of the first result not yet given out. The iterator raises the
  count of results given out as an atomic, never under the queue's
  lock. A worker blocked on a stream holds that lock, and a client that
  sends its next question only after the answer would otherwise
  deadlock.
- **Memory.** Across the sequents, each search keeps `--memory-limit`,
  so a batch's verdicts are those of single calls with the same bound.
  As many workers run as `--batch-memory` holds searches at that bound,
  and at least one, whose bound is then the batch's. Within a sequent,
  each of the two searches of the race holds at most half the batch's
  bound. The default, half the machine's memory, keeps sixteen workers
  at 1 GiB each on this machine (62 GiB) and four on a laptop of 8 GiB.
  Not counted, as in a single call: the forest (bounded by
  `--occurrence-limit`), the derivation (its own bound), a render (its
  own bound, below) and the threads' stacks.
- **Time.** Each entry starts its own `Deadline`, and its read and parse
  run under `Deadline::within`, as a single `prove` reads its one
  input. A load thread left behind when the limit passes lives on in a
  batch until its read and parse end; their cost is linear in the
  input. `--batch-timeout` stops the running searches and answers the
  entries not yet begun as unknown, without reading them. The first
  Ctrl-C stops the running searches and ends the input. The second
  exits with 130.
- **Isolation.** `--isolate` runs `linlog` once per entry. The child
  gets the command's own arguments without the batch's, and the hidden
  `--entry-name` and `--entry-mode`, and runs as a batch of one whose
  line the parent relays. A child that outlives its time limit by 5 s
  is killed. A child killed by a signal is that entry's error.

## Bounded drawings

A sub-agent built this part in a working copy of its own. I merged it
and resolved the one conflict, in `net_into`'s signature, which the
batch now calls with the stop and the reason as well.

- **The library.** `png::Options::memory` and `pdf::Options::memory`
  (`Option<u64>`, default `DEFAULT_MEMORY`, 1 GiB, `None` lifts it)
  refuse a render with `RenderError::Memory { estimate, limit }`
  before usvg parses anything. The estimate is `Measure::of(svg)`: one
  pass over the SVG text counting bytes, elements, glyphs (outside
  `<title>` and `<desc>`) and each arc's cost, times a table per output
  (PNG, PDF, PDF with text as outlines). Every sum saturates.
- **The PNG's pixel bound** is read off the root's `width` and `height`
  before the parse. A net's drawing has a bound like the derivation's:
  `svg::net(net, &style, limit)` returns `svg::TooLarge` past the limit,
  by an estimate from the structure, and the command passes
  `--derivation-limit` to it. The SVG layout's first pass asks `stop`
  after every inference.
- **The command.** It sets the render bound from `--memory-limit` where
  the style keeps the library's default, so `--style pdf.memory=…` still
  wins. `seq print` has `--memory-limit` now. A refused render is a
  left-out line that names the flag; in `seq print`, whose whole output
  the drawing is, it is an error. A render runs on a detached thread
  that the command polls every 5 ms (`limit::detached`, `render` in
  `prove.rs`), so the time limit and the first Ctrl-C reach it, and the
  drawing is then left out with the reason. The second Ctrl-C removes
  every `.PID.partial` file the process has open (a registry in
  `io.rs`) before it exits with 130. `check` and `seq print` now exit
  with 130 at the first Ctrl-C, after the same clean-up.
- **The measure follows the cost.** A glyph costs 1 000 to 2 000 bytes
  and an element 4 400 to 4 700. An arc's cost is not its width but
  where it starts: usvg computes in single precision and strokes an arc
  that starts millions of units out in many pieces, about 11 ms an arc
  at 8·10⁶ units and up to 35 ms further out, against 1.4 ms near the
  origin. Each coefficient is the larger of the measured memory per
  unit and the measured time per unit, converted at 128 MiB/s, plus a
  quarter. The estimates sit 1.3 to 9 times above the measured peaks.
  The measurements were taken on core 6 with derivations of short names
  (`a0`…`a200`), derivations of 20-character names and nets of 500 to
  6 000 links.
- **What the default promises**: a render admitted under 1 GiB holds at
  most 1 GiB and runs for at most about 8 s on core 6, by construction.
  The largest admitted inputs took 4.5 s. A render thread that a batch
  stops waiting for therefore lives on for at most that long, within
  the bound.
- **The review's inputs again**, each in a scope of 3 GiB on core 6.
  The 400-atom tensor as a PDF was refused at an estimate of 5.9 GiB
  (0.28 s, 59 MiB), and as a PNG at 4.67 billion pixels before the
  parse. The net of 6 000 links (`--net --mix`) was refused as a PDF at
  17.0 GiB and as a PNG at 434 billion pixels (0.98 s, 34 MiB, the
  search included). No file was made. By hand: a PDF of `a0`…`a100`
  under `--timeout 1s` was left out at 1.02 s; one Ctrl-C during the
  render left it out as interrupted; two exited with 130 and removed
  the partial file.
- **Left open from this part.** Drawing each link of a net in its own
  coordinates (a `translate` per arc) would avoid usvg's precision
  fault and admit much larger nets, but it changes the net snapshots.
  The net estimate overshoots nested nets four to nine times. The
  second stroking pass charged to a PNG's arcs is not measured. An
  accessible PDF of a large derivation fails krilla's PDF/A-2a check
  (`TooLongString`, the alternative text) with exit status 2, which was
  so before. The coefficients were measured with the default style.

## Measurements

All runs were in memory-capped scopes on cores 4 to 9, with
`--timeout 2s`.

**The batch against single calls.** The two "orders of cores" are
taken here as the two ways a batch spends its cores. Across is
`--cores across --workers 4` against single calls with `--deterministic`,
four at a time. Within is `--cores within --workers 1 --jobs 4` against
single calls with `--jobs 4`, one at a time.

| input | cores | equal | differ | batch exit / worst single |
|---|---|---|---|---|
| `slow-tests.txt` (15 problems) | across | 15 | 0 | 1 / 1 |
| `slow-tests.txt` | within | 15 | 0 | 1 / 1 |
| LLTP sample, ILL (196: every 23rd `.p` file) | across | 195 | 1 | 3 / 3 |
| LLTP sample, ILL | within | 196 | 0 | 3 / 3 |
| LLTP sample, CLL (1) | both | 1 | 0 | 1 / 1 |

Within, 80 problems were unknown on both sides by the time limit and 5
by the recursion limit. The one difference is
`petri-nets/MCC/Diffusion2D_2D8_gradient_20x20_100_5_1`. Across, the
batch ran out of time at a copy bound of 4, while the single call proved
it in 1.3 to 2.3 s. Four hard nets ran side by side, so the problem sits
at the edge of the limit, not at a different search. `--isolate`
printed the same bytes as the run in one process.

**What a shell loop pays** (cores 4 and 5):

| input | method | wall | per sequent |
|---|---|---|---|
| 2 000 small sequents | `while read` loop of `prove --deterministic` | 2.14 s | 1.07 ms |
| 2 000 small sequents | `xargs -P 2` | 1.27 s | 0.63 ms |
| 2 000 small sequents | batch, `--cores across --workers 2` | 0.076 s | 38 µs |
| 2 000 small sequents | batch, `--cores within --workers 1 --jobs 2` | 0.127 s | 64 µs |
| `slow-tests.txt` | `while read` loop | 28 ms | 1.9 ms |
| `slow-tests.txt` | batch, across | 8 ms | 0.5 ms |

The batch is 28 times cheaper than the loop on small sequents. The time
limit's thread and the load thread per entry are included in its
38 µs. On the LLTP sample, where the search dominates, the batch takes
the single calls' time (44.7 s against 43.6 s across, 166 s against
169 s within).

**A stream.** A program asked five questions over pipes, waiting for
each answer before it sent the next one. Each answer came 0.2 to 1.5 ms
after its question, and the exit status after standard input closed was
2, from the one malformed line.

## Verification

All of these pass on the final tree: `cargo clippy --workspace
--all-targets -- --deny warnings`, `cargo test --workspace` (README's
examples with the new ones among them, and `batch_inputs_and_exit_status`
in `cli/tests/cli.rs`), `cargo hack check --each-feature -p linlog`,
`cargo hack check --feature-powerset --depth 2 -p linlog`, and `nix
flake check`. The first run of `nix flake check` failed on two doc links
of the batch module, which are fixed in its commit. The harness's
change only adds argument parsing. The flake's `bench` check runs the
harness, but the all-families verdict run was not made. The target set
was not run, since no search changed: the kept pool reuses a runtime
only when its thread count is exactly the one asked for.

## Decisions

- **Input formats are a flag or an extension, never the text.**
  `--input-format` replaces `--json-input` outright (no aliases before
  the release). `lines`, `jsonl` and `problems` are formats of many
  sequents, which the commands of one sequent refuse. A file of the
  text format is still one sequent, which may span lines. A directory
  takes the files of its format's extension: `.p` and `.json` for
  `auto`, `.txt` for `text`, `lines` and `problems`, and `.jsonl` for
  `jsonl`. This keeps out the library's README, scripts and Maude
  result files.
- **A line's name** comes before a `:`, which no sequent holds. Without
  a name, the entry is called `FILE:LINE`. A file is named by its path
  as given, and a drawn file mirrors that path under `--output DIR`
  with the format's extension added (`X.p.svg`). The translations of
  one LLTP problem therefore never overwrite each other.
- **A problem file's `copies` and `expected` columns are not read**:
  the flags hold for the whole batch, so that its results are those of
  single calls with the same flags. The mode comes from the column, as
  the step says.
- **The output**: stdout gets text lines, or JSON Lines without a
  directory. With `--output DIR`, stdout keeps the text lines, and each
  proved sequent's derivation goes into the directory, or with
  `--format json` each outcome does. A derivation that is left out adds
  an indented line under its sequent's line, and so do `--stats`'
  counters. JSON with `--stats` adds `seconds`.
- **The isolated child is the command itself**, given its own
  arguments minus the batch's. Building the child's arguments from the
  parsed values would need every flag written out again, which a new
  flag would silently miss. The filter needs a new batch flag added to
  its list, which `cli.md` says.
- **The harness's seconds** (`--timeout`, `--pool-after`, `--grace`,
  `--load-limit`) refuse a negative number or NaN. Step 21's review
  assigned this to this step.

## Deviations and assumptions

- "Both orders of cores" was read as across and within, the two ways a
  batch spends its cores, not as cores 4 to 7 against 7 to 4.
- The LLTP sample is 197 files, every 23rd of the 4 512, not 200.
- `xargs` on this machine's path is uutils' 0.10.0, which ignores `-P`.
  The parallel single calls used GNU findutils' `xargs`.

## Open questions and follow-ups

- **The header report found more than 28 wrong headers.** 24 of the 28
  come from one bug in the library's translator: its grammar declares
  no precedence, so `~a => b` is read as `~(a => b)`. Another 21 files
  say "Theorem" and have classical countermodels, though linlog and
  Maude leave them undecided. In all, 101 of the 362 ILTP/KLE
  originals are misread. The report has a section on them, which the
  author can keep or cut. A classical sweep of every translated file
  would list all of them. The release step reminds the author to send
  it.
- **The tail of a batch across the cores**: when fewer sequents than
  workers remain, the idle cores could join them (`Within` for the
  rest). The answers would then depend on timing. Worth it if
  measurements of large libraries show a long tail.
- `--jobs 2` still races one thread against a pool of two, three
  threads in all (step 21's review). The race is written twice, in the
  command and in the harness, and step 28 takes it up together with
  its doubled memory in a single call.
- `--isolate`'s argument filter does not see a short flag cluster that
  ends in `f` (`-qf FILE`). Such a cluster is passed to the child
  unchanged, which then reads the file as well.

## From the review (2026-10-04)

Accepted, with one fix of the batch's memory default. The planning
session first finished what the author had asked for while this step
ran: the rustdoc shows the whole public API ("Show the whole public API
in the docs: the syntax, the JSON forms, the engines and the
features"), then read the report and the batch, ran the checks and the
library through the command.

- **The default batch was killed in a control group** (fixed: "Size a
  batch's default memory by its control group's limit as well as the
  machine's"). `--batch-memory` took half of `/proc/meminfo`'s
  `MemTotal`, which a container, a systemd scope or a CI runner does
  not lower. The sixteen heaviest problems of the library as one default
  batch in a scope of 8 GiB, on this machine's 62 GB and sixteen
  threads, ran sixteen workers and were killed for memory with no line
  written, where every single call fits. The default now takes the
  least `memory.max` of the process's cgroups (v2, or v1's limit) when
  that is less: the same batch ends in 8.3 s at 6.2 GB with all sixteen
  answers.
- **The header report holds**: every one of its 40 countermodels makes
  every formula of its file false under the classical reading it
  states, checked by an evaluator of the files' JSON that shares no
  code with the step's script, with no atom left unassigned; the nine
  proofs it attaches pass `linlog check -i`.
- **By hand**, in capped scopes: entries named `../b.txt`, an absolute
  path, `../../evil` and `/abs/x` all wrote inside `--output DIR`; a
  malformed line was that line's error and the exit status the worst;
  a program asking over pipes got each answer as soon as it was decided
  (0.1 to 1.6 ms beyond the search); `--isolate` gave the same lines; a
  first Ctrl-C answered the running entries as interrupted and ended the
  input, leaving the others without a line, as the rules say; step 22's
  drawings were refused before any parse (the 400-atom PDF at an
  estimate of 5.9 GiB in 0.29 s, the 6 000-link net at 16.3 GiB, the
  100-atom PNG by its pixels) and the smaller ones drawn.
- **The library through the batch**, by default on four cores: the
  `ILL` directory in 19 minutes and `CLL` at once, 4 512 answers, none
  twice, no error, and no verdict against step 22's single calls. 2 193
  proved and 142 refuted, against 2 239 and 152: 57 problems were
  decided only by the single calls, whose default races a pool beside
  one thread where the batch gives each sequent one thread (`LCL181+1`,
  the `SYJ208+1` refutations in `cbv`, forty Petri nets near the limit),
  and one only by the batch (`SYJ204+1.014` in `cbv`). README now says
  what the cores across the sequents give up. The single calls had
  taken eight cores for 37 minutes.
- Also fixed: the help of `seq json` named `--json-input`, which this
  step replaced.
- Not run: the target set, since the sequential engines did not change.
