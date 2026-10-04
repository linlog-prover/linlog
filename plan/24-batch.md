# Step 24: a batch mode, and LLTP input for the command

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing. Read before you start:

- `plan/later.md`: "A batch mode for the CLI" (its six headings are the
  requirement), and under "Follow-ups: the benchmarks" the wrong
  headers.
- `plan/reports/17-assessment.md`: 2.2, 3.12, and the author's answers 1
  and 9.
- `plan/reports/19-time-limits.md`, `20-memory-and-boundaries.md`,
  `21-defaults.md`, `22-configurable-output.md`, `23-session-docs.md`.
- `plan/README.md`: D15, D16.
- `.claude/rules/cli.md`, `core-inputs.md`, `core-export.md` (the
  renderers) and `bench.md`, which load with the files they are about.
- `core/src/lltp.rs`, `cli/**`, `cli/tests/readme.rs`,
  `bench/src/problems.rs`, `bench/src/run.rs`.

## What the earlier steps left you

Step 22 drew proofs as PNG and PDF, and its review found that nothing
bounds the drawing once a derivation is admitted. `png::from_svg` and
`pdf::from_svg` hand the whole SVG to usvg, whose parse sets every glyph
as a path, some 80 bytes per byte of SVG, before any bound is compared;
krilla's PDF takes 90 to 145 bytes per byte in all, the more the denser
the text. So a derivation the default `ViewOptions` admits took over
4 GiB as a PDF and was killed (`a0 ⊗ … ⊗ a399 ⊢ a0 ⊗ … ⊗ a399`, an SVG
of 52 MB), and one of 13 MB took 1.2 GB and 5.9 s, three times the
default time limit, which no poll reaches inside the two crates. usvg
also strokes every wide arc of a net to bound it: a net of 6 000 links
(`--mix`, 4.4 MB of SVG) took 17 s as a PDF and 17 s to be refused as a
PNG. The PNG's pixel bound is compared only after the parse, the net's
drawing (`svg::net`, `--net`) has no bound of its own at all, and the
SVG layout's first pass over the inferences polls nothing (0.67 s on a
tree of 2 400 inferences of 1 200 atoms, with the limit lifted). A
batch that writes drawings into a directory meets all of it per
sequent, in a process that lives on.

Step 23 split the rules by module: what a later session must know of
the batch goes into the rules file of the module that holds it (a new
module of the core crate gets a file of its own, with its paths, and a
row in `core.md`'s table; the command's side goes into `cli.md`). It
also made README's examples a test (`cli/tests/readme.rs`): every
`console` block runs against the binary, so the examples of the batch
and of LLTP input go into README and are checked there, a block whose
output is the machine's is marked as the test's doc comment says, and
a directory of drawings is checked by its files' kinds.

The default of a single call is a race (step 21): one thread, then a
pool beside it, each within the whole `--memory-limit`, so one sequent
may hold twice the bound; the race is written twice, in the command and
in the harness, which step 28 takes up.

## Goal

`linlog prove` decides many sequents in one call: a file of them, several
files, a directory, a stream on standard input answered line by line, in
the command's own syntax, in the harness's problem files and in the LLTP
library's. It is what research use looks like, what an editor or a
script talks to, and what every later measurement by day runs through.

## What to build

1. **The batch as a library notion** (D15): problems in, results out,
   as iterators, with one options value; the command is its first
   caller. The batch takes the command's flags; an options file waits for
   the options' wire form, which step 28 gives every options value.
2. **Input**, every form chosen by a flag or an extension and never
   guessed from a line's text (`A` is an atom and a file name alike): a
   file stays one sequent, which may span lines as today; lines of
   sequents with comments and optional names, in the text syntax or as
   JSON Lines (what `seq json` writes, or a record with a name and a
   mode beside the sequent); several `--file` arguments; a directory;
   a list of paths (`--files-from LIST`, as `tar -T` and `rsync
   --files-from` have it, `-` for standard input, read as it streams,
   and a NUL-separated form for `find -print0`); the harness's problem
   files, which are `.txt` too and so need their flag; LLTP files,
   which `lltp::read` reads and the command cannot take today, also for
   a single `prove`. What to settle and say in the help: paths relative
   to the current directory, as `tar` and `git --pathspec-from-file`
   take them, and literal (no glob, no `~`); a file's kind by its
   extension (`.p` LLTP, `.json` a JSON sequent, else text) unless a
   flag says otherwise; the mode from the flags, a problem file's
   column or a record, never from a directory's name (an LLTP file does
   not say whether it is intuitionistic; the harness reads that off the
   `ILL` directory); a directory walked in sorted order, symlinks
   followed without loops; every entry named by its path as given,
   since LLTP repeats a file name across its translations
   (`SYJ212+1.020.p` is in `01`, `cbn` and `cbv`), so neither results
   nor drawn files overwrite each other; a missing or unreadable file
   that entry's error; the read and parse of each entry under its time
   limit, as `prove` reads its one input today (the largest LLTP file
   is 86 MB, read whole before the occurrence limit can apply).
3. **Output**: one result per sequent in input order, as it is decided,
   as a line of text or a JSON Lines record with what `--format json`
   and `--stats` carry; for the drawing formats a directory. A malformed
   line is that line's error. The exit status is the worst verdict, by
   the order error, unknown, unprovable, proved.
4. **Limits**: the time limit per sequent and one for the whole batch;
   the memory bound of step 20 per sequent, which is what lets one
   process hold a batch, and one for the batch as a whole, since workers
   side by side add up (sixteen at the default of 1 GiB, each perhaps
   racing two searches, would be up to 32 GiB): say how the workers
   share it; `--isolate` for a child per sequent where that is not
   enough.
5. **Cores** go across the sequents by default, one sequent per worker
   on the sequential engines; within one sequent when the batch is short
   or the user says so (D16: the default is sensible and both are
   flags). A pool is kept across sequents rather than built per call.
6. **A stream**: standard input read line by line and each answer
   flushed, so that the same command serves a program that asks, waits
   and asks again.
7. **Bounded drawings**, before the batch writes any (D16): a bound on
   what a render takes, compared before usvg parses anything, in the
   library (a field of `png::Options` and `pdf::Options` with a
   `RenderError` for what it refuses, which the command sets from
   `--memory-limit`), from a measure that follows the cost: the glyphs
   and the arcs of the drawing rather than its bytes, measured as above
   on derivations of short and of long atom names and on nets; the
   PNG's pixel bound read off the SVG's size before the parse; a bound
   on a net's drawing like the derivation's; and the time limit over a
   render. A thread the command stops waiting for, as `Deadline::within`
   does for the read, ends with the process in a single call but lives
   on with its memory in a batch, so the bound in bytes must also keep
   a render short; say what it promises in seconds. Today the first
   Ctrl-C does not reach a render, and the second exits with 130 and
   leaves the output's `FILE.PID.partial` behind, which nothing removes.
8. **The draft of the header report.** With LLTP input in the command,
   write `plan/notes/lltp-headers.md`: the 28 files whose headers
   contradict them, each with linlog's verdict, the checked proof as
   JSON or the classical countermodel, the bound it needed, and where
   the Maude prover's own result file agrees. The author sends it, not
   a session; the release step reminds the author of it.

## Constraints

- A batch's results are those of the single calls, verdict for verdict.
- The harness keeps its child per run: it measures, and a measurement
  wants isolation.
- No engine change.

## Verification

The checks of CLAUDE.md's table and `nix flake check`. The batch against
single calls on `bench/problems/slow-tests.txt` and on a sample of two
hundred LLTP problems, in both orders of cores; a timing of what a shell
loop pays against the batch (named here: two pinned cores, under ten
minutes, detached). For item 7, the review's inputs again (the 400-atom
tensor as PDF and PNG under the defaults, the net of 6 000 links with
`--net`), each in a memory-capped scope: refused at once, or drawn
within the bounds and the time limit. README's examples pass with the
new ones among them.

## Deliverables

- Thematic jj commits.
- `plan/notes/lltp-headers.md`.
- README's usage with the batch and LLTP input, and the rules files of
  the modules touched.
- `plan/reports/24-batch.md`: the options and how each front end sets
  them, the timing, decisions, deviations, open questions.
