---
paths:
  - "cli/**"
---

# linlog-cli: the `linlog` command

Loaded when a file under `cli/` is read. The package is `linlog-cli`, the
binary `linlog` (`[[bin]]` in `cli/Cargo.toml`; `meta.mainProgram` in
`modules/workspace.nix` so that `nix run` finds it).

## Layout

- `argument_parsing.rs`: the clap tree. Doc comments are the `--help`
  text: the first paragraph is the short help (no trailing period, clap's
  style), later paragraphs the long help. The sequent syntax is one
  `SYNTAX` string shown after the help of every command that reads a
  sequent. `Cli::command().debug_assert()` in its tests catches
  inconsistent definitions.
- `lib.rs`: the package's library `linlog_cli`, which holds everything
  (public modules, so rustdoc documents them next to the core crate under
  a name of their own): dispatch, the exit status, the Ctrl-C flag, and
  the parse error with a caret under the failing character
  (`parse_error`: the line of the input the character is in; of a long
  line the `CONTEXT` characters on either side, with `…` where it is
  cut, and then the character's number in the line, as the line's number
  when the input has several; the caret's indent is written as spaces,
  never as a format width, which panics above 65 535). `main.rs` is
  one call into it and is `doc = false`: the binary is named `linlog` like
  the core crate, and documenting it would overwrite the library's docs.
- `io.rs`: input from the argument, `--file` (`-` is standard input) or
  standard input, refused when standard input is a terminal; output to
  `--output` or standard output. The text goes to the parser as it was
  read, untrimmed, so that the line and character a parse error names
  are those of the file.
- `interact.rs`: `interact`, a line-based session over `Interactive`:
  the state comes from the sequent argument or `--state FILE` (a session
  `save` wrote; the mode is then the file's), the commands from standard
  input (`goals`, `rules`, `apply`, `undo`, `close`, `show [--FORMAT]
  [FILE]`, `proof [--FORMAT] [FILE]`, `save`, `load`, `help`, `quit`;
  `HELP` is the list; `show` prints or writes the partial derivation,
  open goals included, as text, LaTeX, Typst, SVG, PNG or PDF, `proof`
  the checked proof in any format, `--rocq` the certificate; PNG and
  PDF go into a FILE only; a format is a word with dashes so that it
  never reads as a file name, and a FILE without one is written in the
  format its extension names, JSON for `proof`, text for `show`),
  every command's output or `error: …` goes to standard output and the
  session goes on,
  and the whole loop runs inside `on_large_stack` so that `close` and the
  derivation drawing have the stack `prove` has. `goal_line` prints a goal
  with the position of every formula, two-sided under the reading.
- `style.rs`: `Styles`, the options value of every format (the
  library's `TextOptions`, `latex::Options`, `typst::Options`,
  `svg::Style`, `png::Options`, `pdf::Options`, `rocq::Options`) under a
  key per format, read from `--style-file` and changed by `--style
  KEY=VALUE` (a key is dotted; a format's name is a prefix only before a
  dot, so `--style text=navy` is SVG's colour under `--format svg`; a
  value that is no JSON is a string), then `--lemma`, `--prelude`,
  `--standalone`. `interact` and every command with several formats need
  the prefix. A new option is a field of the library's value, never a
  flag of its own here.
- `prove.rs`: `prove` and `check`, and `--net`'s refusal of
  sequents outside unit-free MLL and of affine mode (`nets_exist`), before
  the search runs; in intuitionistic mode the net printed is the one of
  the one-sided sequent. The format is `--format`, else the one the
  `--output` file's extension names (`Format::of_path`), else text
  (`OutputArgs::format`); `--net` writes the net in text, svg, png or
  pdf instead of the derivation. PNG and PDF (`Format::is_binary`) are
  the SVG drawing rendered by `render` with the Euler Math font the
  command embeds (`FONT`, `cli/fonts/`, its OFL beside it; the crane
  source keeps that directory), never written to a terminal, their
  verdict and notes on standard error and their output without a
  closing newline. A PDF's date is `SOURCE_DATE_EPOCH` when it is set,
  else the clock (`made`); the tests set it, so their PDFs are byte for
  byte the same. An accessible PDF (`--style pdf.accessible=true`)
  prints a note on standard error that a screen reader reads the
  drawing's description, not the drawing. `--copies N|none` (`Bound`, default `none`,
  on `prove` and `interact`) is `Options::copies`: by default the
  focused engine's deepening goes on until it decides or the time limit
  passes, and with a number it ends there with `unknown … the copy
  bound of N was reached after …`, exit status 3 like every other
  unknown. The library's own default stays a bound of 3, since a library
  call without a stop condition must end; the command has a time limit. `--bias auto|rarer|factors`
  (`BiasArg`, on `prove` and `interact`) is `Options::bias`: how the
  focused engines pick each atom's positive literal, never what is
  provable; `auto` on a sequent with exponentials runs a search under
  each rule, and `--forward-copies` (default
  `Options::DEFAULT_FORWARD_COPIES`, on both commands too) is
  `Options::forward_copies`, the forward search's own copy bound on
  Horn programs. A test that pins a copy bound's message sets both
  bounds, or names a bias. `--stats` prints `copy bound reached`
  (`Statistics::copies`) where the fragment has exponentials. `--net` prints the net the
  net engine found (`Outcome::net`) and otherwise the net read off the
  proof (`net_into`); `--stats` prints the counters of the engine that
  ran (`statistics`, one arm per engine with its own counters). The
  source formats (`latex`, `typst`, `svg`, `rocq`) write the verdict line
  and the statistics as comments of the target (`note`; an XML comment
  turns every `-` into `‐`, since it cannot hold `--` and the advice
  names flags) and the derivation through the library's `write` under
  the format's options from `Styles`; `--standalone` makes the LaTeX,
  Typst or Rocq output a document and is refused, exit 2, for the
  others (`form`: an SVG is always a whole document). **An output is made only with
  something in it beyond the verdict** (`Show::holds`, `Show::open`,
  `Show::close`), since the exit status gives the verdict: a file of
  any format but JSON holds a derivation or a net, or is not made, the
  verdict and any note then on standard error; an SVG, on standard
  output too, is the drawing with its verdict comment before it or
  nothing (a comment alone is no XML document); PNG and PDF always put
  the verdict on standard error. JSON is always written: it is the
  outcome, refutation and statistics included. The held verdict is the
  derivation's prefix (`derivation(…, prefix, out)`). A derivation cut
  short by the time limit leaves no file; on standard output it stays
  as far as it came, with the reason after it. An earlier run's file at
  the path is left as it is. In intuitionistic mode the two-sided derivation
  goes to Rocq and is certified one-sided; a proof with Mix or affine
  weakening is exit 2 with the library's `Unsupported` message, after
  the search, and standard output stays empty. `seq print --format`
  (`SequentFormat`, with `svg`, `png`, `pdf`) prints through
  `sequent_in`, which `sequent_text` wraps. There
  is no `net` subcommand: the CLI's nets are those of proofs, which
  `prove` and `check` draw.

## Invariants

- **Exit status**: 0 proved, valid or done; 1 unprovable or invalid; 2 an
  error, the same status clap uses for bad arguments; 3 unknown. Scripts
  depend on it, and `cli/tests/cli.rs` pins it.
- **The search runs on its own thread** (`on_large_stack`) with the stack
  core's `Options::stack_size` computes for the recursion limit: twice
  the engine's measured cost per level (4 KiB unoptimized, 1 KiB
  optimized), at least 8 MiB, because the derivation builder and its
  renderer, which also recurse to the proof's height, run on the same
  thread. A 1000-level `⊗` chain renders on it without overflow in a
  debug build. The parallel search sizes its pool's threads by the same
  function, so the CLI computes no stack size of its own.
- **The time limit is a flag, and it counts from the start of the
  command** (`limit.rs`: `Deadline`). `Deadline::start(limit, start)`
  starts a thread that sleeps until the limit has passed and raises an
  `AtomicBool`; the stop closure of `prove` and of a session's `close`
  is `interrupted() || deadline.passed()`, two loads, asked at every
  poll. No poll looks at a clock: the closure used to look every 1 024
  polls on one thread, which is exact where the engine polls millions of
  times a second and half a minute late where a poll takes 30 ms (a
  forest of millions of occurrences, a Petri net whose stable sequents
  are large; `--timeout 1s` on the library's largest file ended the
  search after 32.8 s). Dropping the `Deadline` ends its thread, so a
  session starts one per `close`. `prove` counts the limit from its
  first line: the sequent is read, parsed and laid out as a forest by
  `Deadline::within`, on a thread of a main thread's stack that the
  command stops waiting for when the limit passes (the parser cannot be
  stopped from inside; the thread ends with the process), and the answer
  is then `unknown: the time limit of 1s was reached while the sequent
  was read` (`unread`: the output in every format but JSON, where it
  goes to standard error and standard output stays empty), exit status
  3. Without a limit the load runs on the main thread as before. The
  search is `prove_goal` on that forest's roots, which is what
  `prove_until` does after building the forest itself. A session's
  sequent is read outside any limit: its `--timeout` is a `close`'s.
  The CLI knows why the search stopped (`Stop`), so the verdict line
  says "the time limit of 10s was reached" or "interrupted" instead of
  `Reason::Stopped`'s generic phrase. A session's `close` records its
  `Stop` the same way. `cli/tests/cli.rs::timeout` pins
  the line and the status on one thread, on a pool and during the read.
- **The defaults of a call without flags**, on `prove` and on a
  session's `close` alike: no copy bound, the time limit
  `DEFAULT_TIMEOUT` (2 s; `--timeout DURATION|none`, `Time`), and one
  thread first (`threads` in `argument_parsing.rs` makes `Threads`:
  without `--jobs` every thread the machine runs at once after
  `DEFAULT_POOL_AFTER`, 100 ms; `--jobs N` from the start unless
  `--pool-after` is given; `--deterministic` one thread throughout).
  `alone_first` in `prove.rs` runs the search on a thread of its own
  and waits for it up to `Threads::alone`; if it has not decided, a pool
  of the other threads (`jobs − 1`, at least two: a pool of one would be
  the same search again) searches beside it, each within the whole of
  `--memory-limit` (a halved bound starved the memo of a wide sequent:
  `SYJ212+1.013` in `cbn`, refuted in 0.52 s by the restart's pool, was
  not refuted in 5 s by the race's with a quarter of the bound per
  search), and the first to decide raises a flag that stops the
  other (its stop is the command's or that flag); the outcome adds the
  other's counters (`copies` and `memo_entries` by the maximum). The
  single thread is not stopped when the pool starts, because a pool can
  search worse than one thread: on two cores, restarting the search on
  the pool at the switch lost three LLTP problems that one thread proves
  in two seconds (`SYJ204+1.014` in the `01` translation: 15.7 million
  stable sequents on one thread, none decided in 66 million on a pool of
  two). A session's `close` runs the same race over `prove_goal` on the
  goal's occurrences and grafts the proof with `Interactive::close_with`
  under the command's own stop, so the switch never stops a graft. The
  sequential engines are what a pinned output (a test's `--stats`
  counts, a proof compared across runs) needs, since a parallel run's counts add every thread's
  and its proof is the first found; a test that pins such output names
  `--deterministic`, and one whose verdict could depend on the
  machine's speed names `--copies` or `--timeout`. The CLI enables
  core's `parallel` feature in `cli/Cargo.toml`.
- **A wait is never silent on a terminal** (`limit.rs`: `Notice`): when
  standard error is a terminal and the search runs longer than
  `NOTICE_AFTER` (half a second), a thread writes one line there (how
  long the search may run, whether it deepens its bound, the flag that
  changes it, `notice_line`) and takes it back with `\r` and a clear of
  the line when the search ends; the notice is dropped, and its thread
  joined, before anything else is written. Nothing is written into a
  pipe or a file.
- **What "unknown" says** (`unknown`, shared by `verdict_line` and the
  session's `verdict`, with `Ended`: the command's `Stop`, the time the
  search took and the recursion limit): for every `Reason` the bound or
  limit with its value, after how long (except at the time limit, which
  says it), at which copy bound where the engine deepened one (the
  fragment has exponentials, the engine is `focus` or `two-sided`), and
  the flag to try. The tests compare such lines through `timeless` in
  `cli/tests/cli.rs`, which writes every time after "after " as `…`.
  An "unprovable" prints the library's `Refutation`.
- **Every proof reported has passed the checker**: the library checks it
  before `prove_until` returns (`Options::check`), so `--quiet` and
  `--format json` are checked like the drawn formats; `--no-check`
  switches it off. A proof the checker rejects is `Error::Rejected`, exit
  status 2: a defect to report, not a verdict.
- **A derivation is made by `derivation(proof, mode, &show, halt, why,
  out)` and nowhere else** (`prove.rs`; `Show` is what the output
  arguments ask for and where the output goes, `Shown` what became of
  the derivation: `Written` into `out`, `Rendered(bytes)` for a binary
  format, `LeftOut(line)` or `Nothing`). The output is `io::Output`: the
  verdict line goes in first and the derivation after it as the library
  writes it, so no second copy is held; a file is written under another
  name and renamed when whole, and an output dropped unfinished (an
  error) drops its buffer, so that a refused certificate leaves standard
  output empty. `--derivation-limit`
  (`Limit`, default the library's `ViewOptions::DEFAULT_LIMIT`, `none`
  lifts it) is `ViewOptions::limit`, on `prove`, `check` and `interact`.
  A derivation past it is not built: the verdict line stands, the exit
  status is the verdict's, and one line says how large the derivation is
  and names `--format json` and the flag (`too_large`). That line follows
  the verdict when the output is a terminal and goes to standard error
  otherwise (`Show::left_out`), so that a file or a pipe holds what it
  would hold for a small proof minus the tree. In a session the line is
  the command's output, and a `close` whose graft is refused leaves the
  goal open.
- **A text tree that does not fit the terminal is not printed** (`--tree
  auto`, the default; `Show::fit`): when standard output is a terminal,
  there is no `--output` and the format is `text`, the tree is printed
  only if its widest line fits the terminal's columns and its lines
  `--screens` (`SCREENS`, 3, or `none`) times the terminal's rows. Otherwise the verdict is
  followed by one line with the tree's inferences, columns and lines and
  the ways to get it (`unfit`: `--tree always`, `--output FILE`,
  `--format json`). The decision is made twice: from the proof's `Size`
  before anything is built (its width is a lower bound, its lines are
  exact), and from `Derivation::text_size` once the derivation exists,
  which is exact. `--tree always` prints whatever the size, under the
  limit; `--tree never` leaves the derivation out of every format
  without a line; into a file or a pipe `auto` is `always`, so a script
  gets the same output whatever the terminal. `--quiet` is what it was.
  The terminal's size is that of standard output alone:
  `terminal_size::terminal_size_of(stdout)`, never the crate's
  `terminal_size()`, which falls back to standard error and standard
  input and would report a size while the output goes to a pipe; 80 by
  24 where the terminal does not say.
- **The time limit and Ctrl-C hold after the search** (`prove`): the same
  two flags are polled every `STEPS_PER_CLOCK` (256) inferences
  built and pieces of text written (`halt`, which every exporter's
  `write` asks between two inferences). A derivation stopped while it is
  built is left out with the reason (`why`); one stopped while it is
  written stays written as far as it came, followed by a line in the
  format's comment syntax that says it is cut short. The verdict stands.
- **Shortened lines**: `check`'s verdict line abbreviates the sequent
  and its error report every formula list (`--abbreviate`, `ABBREVIATE`
  200 characters, `none`), through `abbreviated` and the library's
  `Described::abbreviated`. `--no-verdict` leaves the verdict line out.
- **The two limits on size** are the library's, with its defaults.
  `--memory-limit SIZE|none` (`Limit`, default
  `Options::DEFAULT_MEMORY_LIMIT`; on `prove` and `interact`) is
  `Options::memory_limit`: a search that passes it answers `unknown …
  the memory limit of 1 GiB was reached; raise it with --memory-limit`,
  exit status 3. `--occurrence-limit N|none` (`Most`, default
  `Forest::DEFAULT_LIMIT`) is on `SequentInput`, so on every command
  that reads a sequent: `SequentInput::sequent` compares
  `Sequent::occurrences()` with it and refuses with exit status 2
  before anything unfolds, prints or lays out the sequent (a JSON
  sequent of 427 bytes can stand for 67 million occurrences);
  `SequentInput::forest` then builds the forest within the same limit,
  and `sequent_in` and `describe`, which see a sequent that was
  admitted, build theirs without one. A session's state and a proof
  file are read by serde, which takes no options: their sequents are
  under the default whatever the flag says.
- **Ctrl-C** (`ctrlc`, whose handler runs on a thread of its own once per
  signal): the first sets a flag the search polls, so the outcome is
  unknown and `--stats` still prints; the second exits with 130. The
  handler is installed by `prove` only.
- **JSON output is core's `Outcome` serialization**, unchanged; `check`
  reads it as a `Proof` because the proof's keys are flattened into it.
  The time is not in the JSON (core has no clock, and the output stays
  reproducible); `--stats` prints it as text.
- `check` takes the mode from its flags, never from the file's `mode` key.
- **`interact` needs the sequent as an argument or `--file`**: standard
  input carries the commands, so the fallback to standard input that
  `SequentInput` gives the other commands is refused with a message. Its
  exit status is 0 only when the session ends with a finished proof that
  checks (`Interactive::proof`), 1 otherwise; a refused command is not an
  error of the session. Rule names on the command line are what
  `Rule::from_str` accepts: the usual spellings and ASCII ones (`*`,
  `par`, `+1`, `-oL`, `&L1`, …). `close` clears the Ctrl-C flag first
  (`clear_interrupt`), since a stopped search must not stop the next one.
- **Intuitionistic mode is a matter of presentation in the CLI**: the
  verdict line and the JSON name the fragment through
  `Fragment::name_in(mode)` (`IMLL`, `ILL`, …); `prove -i` and `check -i`
  print the two-sided derivation (`Proof::two_sided_derivation`) and
  `check -i` the sequent two-sided (`sequent_text`); `seq print -i` and
  `seq fragment -i` do the same for a bare sequent. A sequent with no
  intuitionistic reading is exit 2 with core's `ShapeError` described with
  formulas (`describe` in `prove.rs`, which rebuilds the forest for that);
  a classical proof file checked with `-i` is an invalid proof, exit 1.

## Extension points

- **An engine**: a variant of `EngineArg` with a doc comment (its `--help`
  line) and its arm in `From<EngineArg> for Option<Engine>`. The verdict
  line prints `Engine`'s `Display`; `statistics` in `prove.rs` gets an arm
  only if the engine has counters of its own, as the net and additive
  engines have (`two-sided` shares the focus engine's).
- **An `interact` command**: an arm in `Session::command`, a line in
  `HELP`, and the session test in `cli/tests/cli.rs`, which pins the
  exact output of a scripted session.
- **An output format** (`lean`, say): a variant of `Format` (its
  extension in `Format::of_path`, its key in `style_key` and `Styles`),
  its arm in `derivation`'s `match show.format` calling the library's
  `write`, and in `note` for its comment syntax, so that a source file
  still compiles with the verdict in it. A format with a document form
  takes `--standalone` (`form` lists which formats have one); one that
  draws nets gets an arm in `net_into` and `--net`'s list in
  `Show::new`; a binary one is `is_binary`.
- **A new `Reason`**: its arm in `unknown` (`prove.rs`), which turns a
  generic phrase into advice (`RecursionLimit`, `CopyBound` and
  `MemoryLimit` name the flag to raise); the default arm prints
  `Reason`'s `Display` with the time, as for `IndexLimit`, which no flag
  raises. A new `Refutation` needs nothing here: its `Display` is the
  line.
- Stay out of `core`'s way: no clap types or exit statuses in `core`, and
  the CLI never re-implements what `core` computes (fragment names, the
  mode's words, the JSON form).
