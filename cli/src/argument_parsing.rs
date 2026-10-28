// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use clap::builder::TypedValueParser;
use clap::{Args, Parser, Subcommand, ValueEnum};
use linlog::export::rocq::Kernel;
use linlog::export::svg::MathFont;
use linlog::proofs::{Compact, cut};
use linlog::search::{Cadence, Engine, Options, Schedule};
use linlog::{Bias, Clock, Fragment, Limits, Mode};
use std::path::PathBuf;
use std::time::Duration;

/// The syntax of linear logic, for the help texts below.
macro_rules! linear_syntax {
    () => {
        "\
Sequent syntax: `A, B |- C, D` (or `⊢`), either side may be empty.
  tensor  A * B, A ⊗ B        par   A | B, A par B, A ⅋ B
  with    A & B               plus  A + B, A ⊕ B
  linear implication  A -o B, A ⊸ B
  negation  ~A, A^            exponentials  !A, ?A
  units  1, bot, ⊥, top, ⊤, 0
  quantifiers  forall x. A, ∀x. A, exists x. A, ∃x. A, forall x y. A
  predicates  p(x, f(a)): a name a quantifier binds is its variable, any
    other a constant, or a function symbol before a bracket
Binding, tightest first: A^, then ~ ! ?, then *, |, &, +, then -o, which
groups to the right, and last a quantifier, which reaches right as far as
it can."
    };
}

/// How to write a sequent, shown under the help of every command that reads
/// one.
const SYNTAX: &str = linear_syntax!();

/// How to write a sequent of linear or of ordinary logic, shown under the
/// help of the commands that read ordinary logic with `--logic`.
const ORDINARY: &str = concat!(
    linear_syntax!(),
    "

With --logic the input is ordinary logic: `a, a -> b |- b`, or a formula alone.
  and  a /\\ b, a ∧ b         or   a \\/ b, a ∨ b
  implies  a -> b, a → b      iff  a <-> b, a ↔ b
  not  ~a, ¬a                 constants  true, ⊤, false, ⊥
Binding, tightest first: ~, /\\, \\/, ->, and last <->; -> and <-> group to
the right."
);

/// Decide, check, prove step by step and export sequents of linear logic,
/// and ordinary logic through its translations
#[derive(Parser, Debug)]
#[command(version, long_about = None, after_help = ORDINARY)]
pub(crate) struct Cli {
    /// What to do.
    #[command(subcommand)]
    pub(crate) command: Command,
}

/// The commands.
#[derive(Subcommand, Debug)]
pub(crate) enum Command {
    /// Decide whether a sequent is provable, and print a proof if it is
    ///
    /// The fragment of the sequent (MLL, MALL, …) is detected and picks the
    /// engine; the first line of the output names both. The exit status is 0
    /// for provable, 1 for unprovable, 3 when the search stopped before
    /// deciding, and 2 for an error.
    ///
    /// Given several times `--file`, a directory, `--files-from`, or an
    /// input format of many sequents (`lines`, `jsonl`, `problems`; on
    /// standard input a stream, answered line by line), `prove` decides a
    /// batch. Paths are taken literally, with no glob or `~`; a directory
    /// is walked in sorted order, links followed, for the files of the
    /// input format's extension (`.p` and `.json` for `auto`). Each sequent
    /// is named by its path as given, or by its line's name or
    /// `FILE:LINE`, and gets one line in input order, `NAME: VERDICT`, as
    /// soon as it is decided; with `--format json` a JSON Lines record of
    /// `version` and `name` and the outcome's keys, or with the verdict
    /// `error` and a `message` (and the library's `error` where it caused
    /// it) for an entry that could not be decided. A drawn format writes a file per proved sequent into
    /// `--output DIR`. The mode is the flags', or a problem file's column or
    /// a record's. The exit status is the worst: an error (2) before
    /// unknown (3) before unprovable (1) before proved (0); a batch that
    /// holds no sequent is an error, and one interrupted with Ctrl-C, which
    /// ends its input, is unknown at best.
    #[command(after_help = ORDINARY)]
    Prove(ProveArgs),
    /// Check a proof read from JSON, as `prove --format json` writes it
    ///
    /// The mode comes from the flags, not from the file: pass the flags the
    /// proof was found with. The exit status is 0 for a valid proof, 1 for
    /// an invalid one, 3 when the check was given up at a limit (neither
    /// valid nor invalid), and 2 for an error. With `--format json` the
    /// verdict is `{"version", "verdict", "mode"}`, the verdict `valid`,
    /// `invalid` or `unknown`, with the library's `error` object and its
    /// `message` with formulas for the last two.
    Check(CheckArgs),
    /// Eliminate the cuts of a proof read from JSON, step by step
    ///
    /// The proof is checked in the mode the flags give, then its cuts are
    /// reduced one step at a time, each step on the cut that comes first in
    /// the proof (no cut above it), until no cut is left or a bound is
    /// reached: `--steps=1` takes a single one. Each step is printed with its
    /// case (the axiom case, a key case such as that of ⊗ and ⅋, a
    /// commutative case, weakening, dereliction, contraction, a box
    /// entering a box) and the cut formula, then the proof reached, drawn
    /// as `check` draws it; every proof reached is checked. The exit
    /// status is 0 when the elimination ended cut-free or after the steps
    /// asked for with `--steps`, 1 for a proof that is not valid, 3 when
    /// the default bound on steps, another bound or the time limit stopped
    /// it with cuts left (the proof reached is still printed), and 2 for an
    /// error. With `--format json` the record `{"version",
    /// "proof", "steps", "end"}`: the proof reached, each step's `cut`
    /// (the node of the proof it was taken on), `formula` (the occurrence
    /// cut) and `case`, and `end`, `cut_free` or `refused` with the
    /// `reason`.
    ///
    /// With `--net` the cuts are eliminated on the proof's net instead,
    /// for the sequents `--net` takes: each step reduces the first cut
    /// link (the axiom case, the key case of ⊗ and ⅋, and on boxes
    /// weakening, dereliction, contraction and a box entering a box), and
    /// the net reached is printed as text; `--format svg`, `png` or `pdf`
    /// with `--output FILE` draws every net into a file of its own, FILE-k
    /// (net-1.svg for net.svg) the net step k was taken on, with its cut
    /// link and what it acts on highlighted, and FILE the net reached (each
    /// written as it is drawn, within --derivation-limit, the steps' files
    /// within --drawings-limit together), and
    /// `--format json` writes the record `{"version", "net", "steps",
    /// "end"}`, each step's `cut` the two vertices of its cut link.
    Eliminate(EliminateArgs),
    /// Prove a sequent step by step, reading commands from standard input
    ///
    /// The session starts with the whole sequent as the one open goal; the
    /// sequent is an argument or `--file`, since standard input carries the
    /// commands. Commands, one per line, their words split as a shell
    /// splits them (quotes keep a name with blanks one word): `goals` lists
    /// the open goals with the positions of their formulas; `rules G P`
    /// lists the rules that act on formula P of goal G; `apply G P RULE
    /// [P…]` applies a rule, the further positions being the formulas that
    /// go to the left premise of a ⊗ or Mix; `cut G C [P…]` cuts goal G on
    /// the formula C, written as the mode writes formulas (quoted if it
    /// has blanks), the formulas at the positions P going to the first
    /// premise, which proves C, and the others to the second, which uses
    /// it; `undo` retracts the last step;
    /// `close [G]` lets the search close goal G, or every open goal; `show
    /// [--FORMAT] [FILE]` draws the derivation so far, open goals included,
    /// as text, or --latex, --typst, --svg, --png or --pdf; `proof
    /// [--FORMAT] [FILE]` checks the finished proof and prints or writes it,
    /// as JSON for `check` with a FILE and no format, or --rocq for a
    /// certificate; `save FILE` and `load FILE` keep and resume the session
    /// as JSON (`load` takes a session of the same sequent and mode);
    /// `help`; `quit`. A RULE is a rule's usual name or its ASCII spelling:
    /// ax, 1, bot, top, *, par, &, +1, +2, ?d, ?c, ?w, !, mix, wk (affine
    /// weakening), and in intuitionistic mode the two-sided names (-oL,
    /// -oR, *L, *R, &L1, &L2, &R, +L, +R1, +R2, 1L, 1R, 0L, topR, !L, !R,
    /// !c, !w); `rules` prints the names it applies. The exit status is 0
    /// when the session ends with a finished proof that checks and 1
    /// otherwise; a command's error is printed as `error: …` on standard
    /// output and the session goes on; 2 when the session cannot start or
    /// read its input.
    #[command(after_help = SYNTAX)]
    Interact(InteractArgs),
    /// Print a sequent, convert it to JSON, or name its fragment
    Seq {
        /// What to do with the sequent.
        #[command(subcommand)]
        command: SeqCommand,
    },
}

/// The arguments of `prove`.
#[derive(Args, Clone, Debug)]
pub(crate) struct ProveArgs {
    /// The sequent.
    #[command(flatten)]
    pub(crate) input: SequentInput,
    /// The logic.
    #[command(flatten)]
    pub(crate) mode: ModeArgs,
    /// The ordinary logic the input is in, if any.
    #[command(flatten)]
    pub(crate) logic: LogicArgs,
    /// With --logic, show the linear proof of the image instead of the
    /// proof read back as LK or LJ
    #[arg(long, requires = "logic")]
    pub(crate) linear: bool,
    /// Search in this fragment instead of the detected one (search.fragment)
    ///
    /// A sequent outside it is an error. A larger fragment than the detected
    /// one switches off the prunes that only hold in the smaller one, which
    /// is a way to compare them.
    #[arg(long, value_enum, value_name = "FRAGMENT")]
    pub(crate) fragment: Option<FragmentArg>,
    /// The engine to search with, or `auto` for the one the fragment and
    /// the mode call for (search.engine)
    ///
    /// `focus` is focused sequent search, for every classical fragment,
    /// with or without units, Mix, exponentials and weakening;
    /// `two-sided` the same for every intuitionistic fragment; `net`
    /// proof-net search, for MLL without units, with or without Mix, and
    /// in intuitionistic mode for IMLL without units by its embedding;
    /// `additive` the fast path for two additive-only formulas, in every
    /// mode; `horn` the markings of a Petri net, for Horn programs (atoms,
    /// implications between tensors of atoms, such implications under !,
    /// and one goal that is a tensor of atoms): reachability in linear
    /// mode, coverability in affine mode; `essential` proof-net search over
    /// essential nets, directed by the intuitionistic reading, for IMLL
    /// without units; `inverse` the focused inverse method, a forward
    /// search from the axioms in every mode but the ordered ones, which
    /// decides without exponentials and with them needs no copy bound, and
    /// runs only when named here.
    #[arg(long, value_name = "ENGINE", value_parser = engine_name(), default_value = "auto")]
    pub(crate) engine: Forced,
    /// The search's options, bounds and threads.
    #[command(flatten)]
    pub(crate) search: SearchArgs,
    /// Report the proof the search found without checking it (search.check)
    ///
    /// By default every proof passes the proof checker, which shares no
    /// code with the search, before anything is reported; a proof it
    /// rejects is an error (exit status 2), not a verdict.
    #[arg(long)]
    pub(crate) no_check: bool,
    /// Where and how to write the result.
    #[command(flatten)]
    pub(crate) output: OutputArgs,
    /// Also print what the search cost: the counters of the engine that
    /// ran, a line each, and the time
    ///
    /// The focused engines count stable sequents, memo use, splits and the
    /// copy bound reached, the net engine literals chosen, links tried and
    /// exact tests run, the essential-net engine the same with tests of the
    /// dominator condition, the additive one pairs of subformulas, the Horn
    /// engine markings, the inverse method the derived sequents taken up,
    /// subsumed and kept; every engine its units of work. JSON output always
    /// carries the counts; the time is printed only as text, and in a
    /// batch's JSON records as `seconds`.
    #[arg(long)]
    pub(crate) stats: bool,
    /// Read the settings from a JSON file, before the flags, which
    /// override it: an object with any of the keys `clock`, `limits`,
    /// `search`, `view`, `styles`, `batch`, `ordinary` and `input`, each an
    /// object of the fields the flags name in parentheses (`{"search":
    /// {"copies": 3}, "limits": {"memory_bytes": null}}`); a field left
    /// out keeps the command's default, and a misspelt one is an error;
    /// `ordinary` applies under --logic, which names the logic, and
    /// `batch.mode` is a batch's mode unless a mode flag is given (one
    /// sequent's mode is the flags')
    #[arg(long, value_name = "PATH")]
    pub(crate) settings: Option<PathBuf>,
    /// How a batch of many sequents runs.
    #[command(flatten)]
    pub(crate) batch: BatchArgs,
}

/// The search's options, bounds and threads, on `prove` and on a
/// session's `close`; each flag sets one key of `--settings`.
#[derive(Args, Clone, Debug)]
pub(crate) struct SearchArgs {
    /// How the focused engines pick the positive literal of every atom
    /// (search.bias)
    ///
    /// No choice changes what is provable. `rarer` mostly chains backward
    /// from the goal, `factors` forward from the hypotheses, which on Horn
    /// clauses under `!`, such as a Petri net, is often much faster but
    /// takes one copy per step on a single branch. `auto` takes `rarer`
    /// in affine mode, `factors` in linear mode without exponentials, and
    /// in linear mode with exponentials runs both searches and answers
    /// with the first that decides: alternating on one core, side by side
    /// on several.
    #[arg(long, value_enum, value_name = "BIAS", default_value_t = BiasArg::Auto)]
    pub(crate) bias: BiasArg,
    /// How often `?` formulas may be copied on one branch of the proof, or
    /// `none` for no bound (search.copies)
    ///
    /// The search tries the bounds 0, 1, … in turn. By default it goes on
    /// to the next bound until it decides or the time limit passes, and an
    /// unknown verdict says which bound it reached. With a bound, a sequent
    /// with exponentials that has no proof within it is unknown (exit
    /// status 3) unless a smaller bound already exhausted the search
    /// space, in which case it is unprovable. Sequents without exponentials
    /// are not affected.
    #[arg(long, value_name = "N", value_parser = parse_count::<u32>, default_value_t = Count(None))]
    pub(crate) copies: Count<u32>,
    /// How often `?` formulas may be copied on one branch of the forward
    /// search that `--bias auto` runs (search.forward_copies)
    ///
    /// It applies to a Horn program: clauses such as `!(a * b -o c * d)`,
    /// a marking and a goal of atoms, as a Petri net is. There a copy is
    /// one step of a chain, and a chain of n steps needs n of them. The
    /// forward search never runs within less than `--copies`; on any other
    /// sequent, and under Mix, it runs within `--copies`. Without a bound
    /// from `--copies` it has no effect: both searches deepen while the
    /// time limit lasts.
    #[arg(long, value_name = "N", default_value_t = Options::DEFAULT_FORWARD_COPIES)]
    pub(crate) forward_copies: u32,
    /// Give up after this long, such as 500ms, 10s, 2m or 1h, or `none`
    /// for no limit (clock.time_limit_ms)
    ///
    /// The verdict is then unknown (exit status 3). The time counts from
    /// the start of the command, so it covers reading and parsing the
    /// sequent as well as the search: a sequent that is not read in time
    /// is given up on like one that is not decided. Without the limit, the
    /// search runs until it decides or is interrupted with Ctrl-C, which
    /// with exponentials and no `--copies` bound can be forever. In a
    /// session it is each `close`'s limit.
    #[arg(long, value_name = "DURATION", value_parser = parse_time, default_value_t = Time(Some(DEFAULT_TIMEOUT)))]
    pub(crate) timeout: Time,
    /// The most decided sequents the search remembers at once
    /// (search.memo_limit)
    ///
    /// When the memo is full it is emptied, which costs time but not
    /// correctness. Zero switches the memo off. `--memory-limit` bounds
    /// the memo in bytes; this is the finer knob beside it.
    #[arg(long, value_name = "N", default_value_t = Options::DEFAULT_MEMO_LIMIT)]
    pub(crate) memo_limit: u32,
    /// The most work the classical refutation may do after a search that
    /// did not prove the sequent, in formulas evaluated, or 0 for none
    /// (search.classical_work)
    ///
    /// It looks for truth values of the atoms that make the sequent false
    /// read classically (`!` and `?` erased, `*` and `&` as "and", `|` and
    /// `+` as "or", `1` and `top` true, `bot` and `0` false), which no
    /// provable sequent allows: a search that was unknown is then
    /// unprovable (exit status 1), and an exhausted one gets the values as
    /// its reason. It runs after the search, past the time limit within
    /// this bound (Ctrl-C stops it), and never changes a verdict the
    /// search gave.
    #[arg(long, value_name = "N", default_value_t = Options::DEFAULT_CLASSICAL_WORK)]
    pub(crate) classical_work: u64,
    /// The most memory the search may hold: a number of bytes with a unit
    /// such as 512MiB or 4GiB, or `none` for no limit (limits.memory_bytes)
    ///
    /// Counted is the sequent as the search holds it, about 25 bytes for
    /// each subformula occurrence, and what grows with the search: what
    /// it remembers, the proofs it keeps and what every level of its
    /// recursion takes. A memo that no longer fits is emptied first,
    /// as at `--memo-limit`; when that is not enough the verdict is
    /// unknown (exit status 3). The check of the proof and the derivation
    /// built from it are under the same limit: a derivation estimated
    /// above it is left out even with `--derivation-limit none`. By
    /// default one thread and, after `--pool-after`, a pool beside it
    /// search at once, both together within the limit. A png or pdf
    /// render is estimated before it starts, its time counted as the
    /// memory the renderer fills in it, and left out above the limit (the
    /// default is a few seconds of rendering).
    #[arg(long, value_name = "SIZE", value_parser = parse_bytes, default_value_t = Bytes(Some(Limits::DEFAULT_MEMORY_BYTES)))]
    pub(crate) memory_limit: Bytes,
    /// The deepest nesting of rules on one branch before the search gives
    /// up (limits.recursion_depth)
    ///
    /// Raise it for sequents with thousands of connectives; the search runs on
    /// a thread whose stack grows with the limit.
    #[arg(long, value_name = "N", default_value_t = Limits::DEFAULT_RECURSION_DEPTH)]
    pub(crate) recursion_limit: u32,
    /// The most units of work the search may do, or `none` (limits.work)
    ///
    /// A unit is what each engine counts at its polls (a stable sequent
    /// or a split for the focused engine, a literal or a failed test for
    /// the net engines, a pair for the additive one, a marking for the
    /// Horn engine), so a limit gives the same verdict on every machine
    /// where a time limit does not; past it the verdict is unknown (exit
    /// status 3). `--stats` prints the units a search did.
    #[arg(long, value_name = "N", value_parser = parse_count::<u64>, default_value_t = Count(None))]
    pub(crate) work_limit: Count<u64>,
    /// How many threads the search may use (search.jobs)
    ///
    /// By default one thread searches first, for the time `--pool-after`
    /// gives, and then the machine's other threads join it: a small
    /// sequent is decided at once and always the same way, a hard one
    /// gets every core. A number given here is used from the
    /// start, unless `--pool-after` is given too; a number above the
    /// machine's parallelism is taken as that, with a note on standard
    /// error. More than one runs the focused engine and the net engine on
    /// that many threads; the proof found may then differ from run to run,
    /// the verdict never does.
    #[arg(short, long, value_name = "N")]
    pub(crate) jobs: Option<usize>,
    /// How long one thread searches before the other threads join it,
    /// such as 100ms; 0 starts them at once (clock.pool_after_ms)
    ///
    /// The default is 100ms when `--jobs` is not given, and 0 when it is.
    /// The single thread goes on searching beside a pool of the other
    /// threads, which starts the search afresh, and the first to decide
    /// answers; both together hold at most `--memory-limit`. Below three
    /// threads there is no single thread: two search on a pool from the
    /// start. The additive, the Horn, the essential-net and the inverse
    /// engine run on one thread and get no pool.
    #[arg(long, value_name = "DURATION", value_parser = parse_duration)]
    pub(crate) pool_after: Option<Duration>,
    /// Run the sequential engines, whose proof is a function of the input
    ///
    /// The same as `--jobs 1 --schedule turns`, and takes precedence over
    /// `--jobs`, `--pool-after` and `--schedule`.
    #[arg(long)]
    pub(crate) deterministic: bool,
    /// How often the net engine runs its exact test of correctness, and
    /// the essential-net engine its test of the dominator condition: after
    /// this many links, or `auto` (search.test_period)
    ///
    /// `auto` tests after every link on a small structure and every few
    /// links on a large one. The verdict never depends on it.
    #[arg(long, value_name = "N", value_parser = parse_cadence, default_value = "auto")]
    pub(crate) test_period: Cadence,
    /// How the two searches of `--bias auto` share one thread
    /// (search.schedule)
    #[arg(long, value_enum, value_name = "SCHEDULE", default_value_t = ScheduleArg::Auto)]
    pub(crate) schedule: ScheduleArg,
}

/// How the two searches of the default bias share one thread.
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ScheduleArg {
    /// In slices on two threads
    Auto,
    /// In turns on the calling thread, each turn from the search's start:
    /// the same answer and the same counts on every machine
    Turns,
}

impl From<ScheduleArg> for Schedule {
    /// Returns the library's value of the choice.
    fn from(schedule: ScheduleArg) -> Self {
        match schedule {
            ScheduleArg::Auto => Self::Auto,
            ScheduleArg::Turns => Self::Turns,
        }
    }
}

/// Parses a test period: a number of links, or `auto`.
fn parse_cadence(text: &str) -> Result<Cadence, String> {
    if text == "auto" {
        return Ok(Cadence::Auto);
    }
    text.parse()
        .map(Cadence::Every)
        .map_err(|_| format!("{text:?} is neither a number of links nor `auto`"))
}

/// How `prove` decides many sequents in one call (`prove --help` says
/// when a call is a batch and what it writes).
#[derive(Args, Clone, Debug)]
pub(crate) struct BatchArgs {
    /// Read the paths of the inputs from this file, one per line, or
    /// `-` for standard input, read as it streams
    #[arg(long, value_name = "LIST")]
    pub(crate) files_from: Option<PathBuf>,
    /// The paths of --files-from are separated by NUL, as `find -print0`
    /// writes them
    #[arg(long)]
    pub(crate) null: bool,
    /// How many sequents of a batch are decided at once, across the
    /// cores; by default as many as the machine runs threads, fewer when
    /// --batch-memory does not hold that many searches at
    /// --memory-limit each (batch.workers)
    #[arg(long, value_name = "N")]
    pub(crate) workers: Option<usize>,
    /// Where a batch's threads go (batch.cores)
    ///
    /// Across the sequents, each gets one thread for its --timeout, as with
    /// --deterministic, which answers the most sequents in a given time and
    /// gives every answer as a function of its input; a sequent that needs
    /// the pool a single call races beside one thread, or more time on one
    /// thread, stays unknown there. Within, each sequent gets the race on
    /// every thread, as a single call does. To decide what a batch across
    /// left unknown, run those sequents again with --cores within or a
    /// longer --timeout.
    #[arg(long, value_enum, value_name = "WHERE", default_value_t = CoresArg::Auto)]
    pub(crate) cores: CoresArg,
    /// The most memory all the searches of a batch hold together: a size
    /// such as 8GiB, or `none`; by default half the memory the process may
    /// use (the machine's, or its control group's limit where one is set)
    /// (batch.total_memory_bytes)
    ///
    /// Each search holds at most --memory-limit, and as many run at once
    /// as this holds; within one sequent the race of one thread and a
    /// pool is two searches, which share it.
    #[arg(long, value_name = "SIZE", value_parser = parse_bytes)]
    pub(crate) batch_memory: Option<Bytes>,
    /// Give up on the rest of a batch after this long, or `none`
    /// (clock.batch_time_limit_ms)
    ///
    /// The sequents still searched are stopped and those not begun are
    /// unknown; --timeout is each sequent's own limit.
    #[arg(long, value_name = "DURATION", value_parser = parse_time, default_value_t = Time(None))]
    pub(crate) batch_timeout: Time,
    /// Decide each sequent of a batch in a child process of its own, so
    /// that one that exhausts the memory or the stack ends only itself
    #[arg(long)]
    pub(crate) isolate: bool,
    /// The name of the one sequent a child of --isolate decides.
    #[arg(long, hide = true, value_name = "NAME", allow_hyphen_values = true)]
    pub(crate) entry_name: Option<String>,
    /// The mode of the one sequent a child of --isolate decides.
    #[arg(long, hide = true, value_name = "MODE", requires = "entry_name")]
    pub(crate) entry_mode: Option<String>,
    /// The format of the one sequent a child of --isolate decides.
    #[arg(
        long,
        hide = true,
        value_enum,
        value_name = "FORMAT",
        requires = "entry_name"
    )]
    pub(crate) entry_format: Option<InputFormat>,
    /// The file of the one sequent a child of --isolate decides; standard
    /// input when absent.
    #[arg(
        long,
        hide = true,
        value_name = "PATH",
        requires = "entry_name",
        allow_hyphen_values = true
    )]
    pub(crate) entry_file: Option<PathBuf>,
}

/// Where a batch's threads go.
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CoresArg {
    /// Across the sequents when the batch has at least as many as
    /// --workers, else within; within on a stream from standard input,
    /// whose next question may wait for an answer
    Auto,
    /// One sequent per worker, each on one thread: the sequential
    /// engines, as with --deterministic
    Across,
    /// One sequent at a time, with the threads --jobs gives
    Within,
}

/// The arguments of `interact`.
#[derive(Args, Debug)]
pub(crate) struct InteractArgs {
    /// The sequent, as an argument or with --file, since standard input
    /// carries the commands.
    #[command(flatten)]
    pub(crate) input: SequentInput,
    /// Resume a session saved with `save` instead of starting from a
    /// sequent; the mode is the file's, so the mode flags are refused
    #[arg(
        long,
        value_name = "PATH",
        conflicts_with_all = ["sequent", "file", "intuitionistic", "affine", "mix"]
    )]
    pub(crate) state: Option<PathBuf>,
    /// The logic.
    #[command(flatten)]
    pub(crate) mode: ModeArgs,
    /// The search's options, bounds and threads, for a `close`.
    #[command(flatten)]
    pub(crate) search: SearchArgs,
    /// Apply every invertible rule by itself, to the sequent at the start
    /// and to the goals each step opens: ⅋, ⊥, & and ⊤, ! when every other
    /// formula is a ?, 1, ⊗ and the axiom when their formulas are the whole
    /// goal (interactive.invertible); the command `invertible on|off`
    /// turns it on or off within the session
    #[arg(long)]
    pub(crate) invertible: bool,
    /// The largest derivation `close` grafts, by its estimated size; see
    /// `prove --derivation-limit` (limits.derivation_bytes)
    #[arg(long, value_name = "SIZE", value_parser = parse_bytes, default_value_t = Bytes(Some(Limits::DEFAULT_DERIVATION_BYTES)))]
    pub(crate) derivation_limit: Bytes,
    /// How `show` and `proof` write; a `--style` key names its format
    /// with a prefix here, `latex.labels=subscript`.
    #[command(flatten)]
    pub(crate) style: StyleArgs,
    /// The Rocq certificate's lemma and prelude, for `proof --rocq`.
    #[command(flatten)]
    pub(crate) rocq: RocqArgs,
    /// Read the settings from a JSON file, before the flags, which
    /// override it: an object with any of the keys `clock`, `limits`,
    /// `search`, `view`, `styles`, `batch`, `ordinary`, `interactive` and
    /// `input`, each an object of the fields the flags name in parentheses
    /// (`{"search": {"copies": 3}, "limits": {"memory_bytes": null}}`); a
    /// field left out keeps the command's default, and a misspelt one is
    /// an error; `ordinary` applies under --logic, which names the logic
    #[arg(long, value_name = "PATH")]
    pub(crate) settings: Option<PathBuf>,
}

/// The time limit of `prove` and of a session's `close` when none is
/// given: what a larger copy bound still decides against what every
/// undecided sequent then waits. Of the problems of the LLTP library that
/// a copy bound of 3 leaves undecided and a larger bound decides, nine in
/// ten are decided within a second or two, while the many that nothing
/// decides each wait the whole limit.
pub(crate) const DEFAULT_TIMEOUT: Duration = Duration::from_millis(Clock::DEFAULT_TIME_LIMIT_MS);

/// The arguments of `check`.
#[derive(Args, Debug)]
pub(crate) struct CheckArgs {
    /// The most memory the check may hold: a number of bytes with a unit
    /// such as 512MiB or 4GiB, or `none` for no limit
    ///
    /// A proof whose check would pass it is neither valid nor invalid:
    /// the verdict is unknown (exit status 3). A derivation estimated above
    /// it is left out, as past `--derivation-limit`, and so is a png or pdf
    /// render; see `prove --memory-limit` (limits.memory_bytes).
    #[arg(long, value_name = "SIZE", value_parser = parse_bytes, default_value_t = Bytes(Some(Limits::DEFAULT_MEMORY_BYTES)))]
    pub(crate) memory_limit: Bytes,
    /// Give up after this long, such as 500ms, 10s, 2m or 1h, or `none`
    /// (clock.time_limit_ms)
    ///
    /// A proof file can make its check quadratic (a large shared subproof
    /// read by many nodes); the verdict is then unknown (exit status 3).
    /// The time counts from the start of the command and covers the
    /// derivation drawn after the check.
    #[arg(long, value_name = "DURATION", value_parser = parse_time, default_value_t = Time(Some(DEFAULT_TIMEOUT)))]
    pub(crate) timeout: Time,
    /// The most units of work the check may do, or `none` (limits.work)
    ///
    /// A unit is a node of the proof or an entry of a sequent a node is
    /// handed; past the limit the verdict is unknown (exit status 3).
    #[arg(long, value_name = "N", value_parser = parse_count::<u64>, default_value_t = Count(None))]
    pub(crate) work_limit: Count<u64>,
    /// The most subformula occurrences the proof's sequent may have, or
    /// `none` (limits.occurrences); see `prove --occurrence-limit`
    #[arg(long, value_name = "N", value_parser = parse_count::<u64>, default_value_t = Count(Some(Limits::DEFAULT_OCCURRENCES)))]
    pub(crate) occurrence_limit: Count<u64>,
    /// The proof file, or standard input when absent or `-`
    #[arg(value_name = "PROOF")]
    pub(crate) proof: Option<PathBuf>,
    /// The logic the proof must hold in.
    #[command(flatten)]
    pub(crate) mode: ModeArgs,
    /// Where to write the result.
    #[command(flatten)]
    pub(crate) output: OutputArgs,
    /// Read the settings from a JSON file, before the flags, which
    /// override it: an object with any of the keys `clock`, `limits`,
    /// `search`, `view`, `styles`, `batch`, `ordinary` and `input`, each an
    /// object of the fields the flags name in parentheses (`{"search":
    /// {"copies": 3}, "limits": {"memory_bytes": null}}`); a field left
    /// out keeps the command's default, and a misspelt one is an error;
    /// `ordinary` applies under --logic, which names the logic
    #[arg(long, value_name = "PATH")]
    pub(crate) settings: Option<PathBuf>,
}

/// The arguments of `eliminate`.
#[derive(Args, Debug)]
pub(crate) struct EliminateArgs {
    /// The most steps to take, or `none` (cut.steps)
    ///
    /// Past it the proof reached is printed with the cuts it still has;
    /// the exit status is 0 at a bound given here or in a settings file,
    /// 3 at the default, which only keeps the call bounded.
    #[arg(long, value_name = "N", value_parser = parse_count::<u64>, default_value_t = Count(Some(cut::Options::DEFAULT_STEPS)))]
    pub(crate) steps: Count<u64>,
    /// The most nodes a proof a step makes may have, or with --net the most
    /// vertices of a net, or `none` (cut.nodes)
    ///
    /// The exponentials can make a proof grow beyond any practical size;
    /// a step that would pass the limit is not taken, and the proof
    /// reached is printed (exit status 3).
    #[arg(long, value_name = "N", value_parser = parse_count::<u64>, default_value_t = Count(Some(cut::Options::DEFAULT_NODES)))]
    pub(crate) node_limit: Count<u64>,
    /// The most bytes the drawings of the steps take together with --net
    /// and a drawing format, or `none` (cut.drawn_bytes)
    ///
    /// Past it the steps are taken and printed undrawn, a line on standard
    /// error says from which step on, and FILE still holds the net
    /// reached.
    #[arg(long, value_name = "SIZE", value_parser = parse_bytes, default_value_t = Bytes(Some(cut::Options::DEFAULT_DRAWN_BYTES)))]
    pub(crate) drawings_limit: Bytes,
    /// The bounds, the mode and the output, as `check` takes them.
    #[command(flatten)]
    pub(crate) check: CheckArgs,
}

/// The subcommands of `seq`.
#[derive(Subcommand, Debug)]
pub(crate) enum SeqCommand {
    /// Print a sequent one-sided, in negation normal form, or two-sided
    ///
    /// `A, A -o B |- B` prints as `⊢ ~A, A ⊗ ~B, B`: hypotheses are negated
    /// onto the right, implications become pars, and negation is pushed down
    /// to the atoms. With --intuitionistic it prints as `A, A ⊸ B ⊢ B`
    /// again, which requires an intuitionistic sequent: one formula on the
    /// right, and pars only as implications.
    #[command(after_help = ORDINARY)]
    Print {
        /// The sequent.
        #[command(flatten)]
        input: SequentInput,
        /// Print the sequent two-sided, as intuitionistic linear logic
        /// reads it
        #[arg(short, long)]
        intuitionistic: bool,
        /// Read the text as cyclic linear logic does: the dual of a
        /// product reversed, ~(A * B) as ~B par ~A, and the formulas left
        /// of ⊢ in the reverse order
        #[arg(long, conflicts_with = "intuitionistic")]
        cyclic: bool,
        /// Read the text as --cyclic does and print the sequent two-sided,
        /// as the Lambek calculus reads it: with its divisions A \ B and
        /// B / A and its hypotheses in their order
        #[arg(long, conflicts_with_all = ["intuitionistic", "cyclic"])]
        lambek: bool,
        /// The ordinary logic the input is in, if any.
        #[command(flatten)]
        logic: LogicArgs,
        /// The output format
        #[arg(long, value_enum, value_name = "FORMAT", default_value_t = SequentFormat::Text)]
        format: SequentFormat,
        /// Write a document that compiles on its own instead of a fragment
        /// to paste (latex and typst)
        #[arg(long)]
        standalone: bool,
        /// Write to this file instead of standard output
        #[arg(short, long, value_name = "PATH")]
        output: Option<PathBuf>,
        /// How the output looks.
        #[command(flatten)]
        style: StyleArgs,
        /// The most memory a png or pdf render may take, by its estimate:
        /// a number of bytes with a unit such as 512MiB or 4GiB, or `none`
        /// for no limit; see `prove --memory-limit`
        #[arg(long, value_name = "SIZE", value_parser = parse_bytes, default_value_t = Bytes(Some(Limits::DEFAULT_MEMORY_BYTES)))]
        memory_limit: Bytes,
        /// Read the settings from a JSON file, before the flags, which
        /// override it: an object with any of the keys `clock`, `limits`,
        /// `search`, `view`, `styles`, `batch`, `ordinary` and `input`, each
        /// an object of the fields the flags name in parentheses (`{"search":
        /// {"copies": 3}, "limits": {"memory_bytes": null}}`); a field left
        /// out keeps the command's default, and a misspelt one is an error;
        /// `ordinary` applies under --logic, which names the logic
        #[arg(long, value_name = "PATH")]
        settings: Option<PathBuf>,
    },
    /// Print a sequent as JSON, the form `--input-format json` reads
    #[command(after_help = SYNTAX)]
    Json {
        /// The sequent.
        #[command(flatten)]
        input: SequentInput,
        /// Merge equal subformulas, drop unreferenced ones and sort the
        /// formulas of a JSON input, as parsing text always does
        #[arg(long)]
        optimize: bool,
        /// Read the text as cyclic linear logic does: the dual of a
        /// product reversed, ~(A * B) as ~B par ~A, and the formulas left
        /// of ⊢ in the reverse order
        #[arg(long)]
        cyclic: bool,
        /// Print the intuitionistic sequent's own form: its formulas with
        /// the connectives of intuitionistic linear logic, its hypotheses
        /// and its goal, which --intuitionistic reads too
        #[arg(short, long, conflicts_with_all = ["cyclic", "optimize"])]
        intuitionistic: bool,
        /// Write to this file instead of standard output
        #[arg(short, long, value_name = "PATH")]
        output: Option<PathBuf>,
        /// The most bytes of input read: a number of bytes with a unit such
        /// as 512MiB or 4GiB, or `none` for no limit
        #[arg(long, value_name = "SIZE", value_parser = parse_bytes, default_value_t = Bytes(Some(Limits::DEFAULT_MEMORY_BYTES)))]
        memory_limit: Bytes,
    },
    /// Print the smallest fragment a sequent lives in: MLL, MLL with units,
    /// ALL, MALL, MELL or LL, or with --intuitionistic IMLL, IMLL with
    /// units, IALL, IMALL, IMELL or ILL
    #[command(after_help = SYNTAX)]
    Fragment {
        /// The sequent.
        #[command(flatten)]
        input: SequentInput,
        /// Name the intuitionistic fragment
        #[arg(short, long)]
        intuitionistic: bool,
        /// The most bytes of input read: a number of bytes with a unit such
        /// as 512MiB or 4GiB, or `none` for no limit
        #[arg(long, value_name = "SIZE", value_parser = parse_bytes, default_value_t = Bytes(Some(Limits::DEFAULT_MEMORY_BYTES)))]
        memory_limit: Bytes,
    },
}

/// Where a sequent comes from.
#[derive(Args, Clone, Debug)]
pub(crate) struct SequentInput {
    /// The sequent, such as "A, A -o B |- B"; read from --file when absent,
    /// or from standard input by every command but `interact`
    #[arg(value_name = "SEQUENT", conflicts_with = "file")]
    pub(crate) sequent: Option<String>,
    /// Read the sequent from this file, or `-` for standard input (but in
    /// `interact`, whose commands come from there)
    ///
    /// The file's kind is its extension's unless --input-format says
    /// otherwise: `.p` an LLTP problem (under --logic, where a command
    /// takes it, a TPTP problem of ordinary logic), `.spec` a
    /// coverability problem, `.json` a JSON sequent, anything else the
    /// text syntax, which may span lines. `prove` takes the flag more than
    /// once, and a directory, as a batch.
    #[arg(short, long, value_name = "PATH", action = clap::ArgAction::Append)]
    pub(crate) file: Vec<PathBuf>,
    /// How to read the input, rather than by the file's extension
    ///
    /// `lines`, `jsonl` and `problems` hold many sequents, which `prove`
    /// decides as a batch.
    #[arg(long, value_enum, value_name = "FORMAT", default_value_t = InputFormat::Auto)]
    pub(crate) input_format: InputFormat,
    /// The most subformula occurrences the sequent may have, or `none`
    /// (limits.occurrences)
    ///
    /// A sequent in JSON can share subformulas, so a small file may stand
    /// for a sequent of any size; one beyond the limit is refused before
    /// it is unfolded. Occurrences cost some 25 bytes each before any
    /// search, and no limit lets through more than about four billion. A
    /// text nested deeper than the limit (more `(` or `~` open at once) is
    /// refused the same way.
    #[arg(long, value_name = "N", value_parser = parse_count::<u64>, default_value_t = Count(Some(Limits::DEFAULT_OCCURRENCES)))]
    pub(crate) occurrence_limit: Count<u64>,
    /// How the text of an intuitionistic sequent is read: in its own
    /// syntax alone, or in the classical one (input.intuitionistic)
    ///
    /// With --intuitionistic a sequent is written in the syntax of
    /// intuitionistic linear logic, A -o B, A |- B, and a symbol of the
    /// classical one (~, ^, par, |, ?, bot) is refused with what to write
    /// instead; `one-sided` reads the classical syntax as the one-sided
    /// sequent it stands for, ~(A * ~B), A |- B as A ⊸ B, A ⊢ B.
    #[arg(long, value_enum, value_name = "SYNTAX", default_value_t = IllSyntaxArg::Strict)]
    pub(crate) ill_syntax: IllSyntaxArg,
}

/// How the text of an intuitionistic sequent is read.
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum IllSyntaxArg {
    /// The syntax of intuitionistic linear logic alone
    Strict,
    /// The classical syntax, read as the one-sided sequent it stands for
    OneSided,
}

impl From<IllSyntaxArg> for linlog::IllSyntax {
    /// The library's syntax of the flag's.
    fn from(syntax: IllSyntaxArg) -> Self {
        match syntax {
            IllSyntaxArg::Strict => Self::Strict,
            IllSyntaxArg::OneSided => Self::OneSided,
        }
    }
}

/// How an input is read.
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum InputFormat {
    /// By the file's extension: `.p` lltp (tptp under --logic, where a
    /// command takes it), `.spec` spec, `.json` json, else text
    Auto,
    /// One sequent in the text syntax, which may span lines
    Text,
    /// One sequent in JSON, as `seq json` writes it
    Json,
    /// A problem of the LLTP library: its axioms ⊢ its conjecture; the file
    /// does not say whether it is intuitionistic, the flags do
    Lltp,
    /// A coverability problem in the .spec format of the Mist tool: its
    /// rules under !, its initial marking ⊢ its target, a question of
    /// affine mode, in which it is decided without --affine
    Spec,
    /// One sequent per line in the text syntax, as `NAME: SEQUENT` or
    /// `SEQUENT`; blank lines and everything from `#` on are skipped
    Lines,
    /// One JSON value per line: a sequent as `seq json` writes it, or
    /// {"name": …, "mode": …, "sequent": …} with the sequent in JSON or as
    /// text and the mode named as in `problems`; blank lines are skipped
    Jsonl,
    /// A problem of ordinary propositional logic in TPTP syntax, as the
    /// ILTP library writes them: its axioms ⊢ its conjecture (with --logic,
    /// where a `.p` file is one)
    Tptp,
    /// The benchmark harness's problem files: `NAME; MODE; EXPECTED;
    /// COPIES; SEQUENT` per line, MODE one of classical, mix, affine,
    /// affine-mix, intuitionistic, intuitionistic-affine, cyclic, lambek
    /// or lambek-star (whose sequents are read as --cyclic reads them);
    /// EXPECTED and COPIES are not read
    Problems,
}

impl InputFormat {
    /// Returns the format of a file: this one, or for `Auto` the one its
    /// extension names, a `.p` file being a TPTP problem of ordinary logic
    /// when `ordinary` is set and an LLTP problem otherwise.
    #[must_use]
    pub(crate) fn of(self, path: &std::path::Path, ordinary: bool) -> Self {
        if self != Self::Auto {
            return self;
        }
        match path.extension().and_then(|e| e.to_str()) {
            Some("p") if ordinary => Self::Tptp,
            Some("p") => Self::Lltp,
            Some("spec") => Self::Spec,
            Some("json") => Self::Json,
            _ => Self::Text,
        }
    }

    /// Whether an input in the format holds many sequents.
    pub(crate) fn is_many(self) -> bool {
        matches!(self, Self::Lines | Self::Jsonl | Self::Problems)
    }
}

/// The ordinary logic an input is in, and the translation that decides
/// it.
#[derive(Args, Clone, Debug)]
pub(crate) struct LogicArgs {
    /// Read the input as ordinary logic, through its translation into
    /// linear logic
    ///
    /// The input is a formula or a sequent of propositional logic in the
    /// syntax below (a `.p` file a TPTP problem, as the ILTP library writes
    /// them). `prove` decides its image and reads a proof back as a
    /// derivation of LK (classical) or LJ (intuitionistic, minimal), which
    /// is checked by the rules of the logic before it is shown; classical
    /// logic is decided in affine MALL, with no copy bound, intuitionistic
    /// and minimal logic in ILL, under the time limit, so they may be
    /// answered unknown, and the JSON output is the search's outcome on the
    /// image. `seq print` prints the image.
    #[arg(long, value_enum, value_name = "LOGIC")]
    pub(crate) logic: Option<LogicArg>,
    /// The translation into linear logic: affine for classical logic, cbn
    /// (by default), cbv or 01 for intuitionistic and minimal logic
    /// (ordinary.translation)
    #[arg(long, value_enum, value_name = "TRANSLATION", requires = "logic")]
    pub(crate) translation: Option<TranslationArg>,
}

/// An ordinary logic.
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LogicArg {
    /// Classical propositional logic, derivations in LK
    Classical,
    /// Intuitionistic propositional logic, derivations in LJ
    Intuitionistic,
    /// Minimal logic: intuitionistic without ex falso, false an atom
    Minimal,
}

impl From<LogicArg> for linlog::ordinary::Logic {
    fn from(logic: LogicArg) -> Self {
        match logic {
            LogicArg::Classical => Self::Classical,
            LogicArg::Intuitionistic => Self::Intuitionistic,
            LogicArg::Minimal => Self::Minimal,
        }
    }
}

/// A translation of ordinary logic into linear logic.
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TranslationArg {
    /// Classical logic into affine MALL: negation normal form, ∧ as &, ∨
    /// as ⅋
    Affine,
    /// Girard's call-by-name translation into ILL: A → B as !A ⊸ B, A ∨ B
    /// as !A ⊕ !B, hypotheses under !
    Cbn,
    /// Girard's call-by-value translation into ILL: atoms as !a, A → B as
    /// !(A ⊸ B), ∧ as ⊗
    Cbv,
    /// Liang and Miller's 0/1 translation into ILL, as the LLTP library
    /// states it
    #[value(name = "01")]
    ZeroOne,
}

impl From<TranslationArg> for linlog::ordinary::Translation {
    fn from(translation: TranslationArg) -> Self {
        match translation {
            TranslationArg::Affine => Self::Affine,
            TranslationArg::Cbn => Self::CallByName,
            TranslationArg::Cbv => Self::CallByValue,
            TranslationArg::ZeroOne => Self::ZeroOne,
        }
    }
}

/// The logic a sequent is proved in.
#[derive(Args, Clone, Debug)]
pub(crate) struct ModeArgs {
    /// Intuitionistic linear logic: hypotheses left of ⊢ and one goal
    /// right of it, written in its own syntax (--ill-syntax), and
    /// two-sided derivations
    #[arg(short, long)]
    pub(crate) intuitionistic: bool,
    /// Affine logic: a hypothesis may go unused (weakening)
    #[arg(short, long)]
    pub(crate) affine: bool,
    /// Allow the Mix rule, which proves ⊢ Γ, Δ from ⊢ Γ and ⊢ Δ
    #[arg(long)]
    pub(crate) mix: bool,
    /// Cyclic linear logic: the order of the formulas counts, up to
    /// rotation, with no exchange (unit-free MLL); the sequent's text is
    /// read with the dual of a product reversed, ~(A * B) as ~B par ~A,
    /// and the formulas left of ⊢ in the reverse order
    #[arg(long, conflicts_with = "intuitionistic")]
    pub(crate) cyclic: bool,
    /// The Lambek calculus: intuitionistic and in the order written, with
    /// the divisions A \ B and B / A beside A * B (unit-free), every sequent
    /// with a formula left of ⊢; the text is read as --cyclic reads it
    #[arg(long, conflicts_with_all = ["intuitionistic", "cyclic"])]
    pub(crate) lambek: bool,
    /// With --lambek, allow sequents with no formula left of ⊢ (the
    /// calculus L*)
    #[arg(long, requires = "lambek")]
    pub(crate) empty_antecedents: bool,
}

impl ModeArgs {
    /// Returns the mode the flags ask for.
    pub(crate) fn mode(&self) -> Mode {
        let mut mode = if self.intuitionistic {
            Mode::INTUITIONISTIC
        } else if self.cyclic {
            Mode::CYCLIC
        } else if self.lambek {
            Mode::LAMBEK
        } else {
            Mode::CLASSICAL
        };
        if self.empty_antecedents {
            mode = mode.with_empty_antecedents();
        }
        if self.affine {
            mode = mode.with_affine();
        }
        if self.mix {
            mode = mode.with_mix();
        }
        mode
    }
}

/// Where and how a command writes its result.
#[derive(Args, Clone, Debug)]
pub(crate) struct OutputArgs {
    /// The output format; by default the one the extension of the
    /// `--output` file names (.txt, .json, .tex, .typ, .svg, .png, .pdf,
    /// .v), and text otherwise
    #[arg(long, value_enum, value_name = "FORMAT")]
    pub(crate) format: Option<Format>,
    /// Write to this file instead of standard output; the file is made
    /// only with a derivation or a net in it (or in JSON, always), and
    /// otherwise the verdict goes to standard error
    #[arg(short, long, value_name = "PATH")]
    pub(crate) output: Option<PathBuf>,
    /// Write the proof net of the proof instead of its derivation, for MLL
    /// without units, with or without Mix, and with boxes for MELL without
    /// units, classical with or without Mix or intuitionistic (the net of
    /// the one-sided sequent): as text (the sequent, the
    /// copies of every ?, its axiom links as pairs of literals with their
    /// vertex numbers, and the verdict of the correctness criterion), as
    /// json (the net's document in place of the outcome), or drawn as svg,
    /// png or pdf (the formula trees with the axiom links as arcs over the
    /// literals, every !-box a rectangle around its contents)
    #[arg(long)]
    pub(crate) net: bool,
    /// Print only the verdict line, not the derivation; with --output,
    /// on standard error, since a file holds a derivation or nothing
    #[arg(short, long)]
    pub(crate) quiet: bool,
    /// Write a document that compiles on its own instead of a fragment to
    /// paste (latex, typst and rocq)
    #[arg(long)]
    pub(crate) standalone: bool,
    /// Leave out the verdict line, so that the output is the derivation
    /// alone; the exit status still gives the verdict
    #[arg(long)]
    pub(crate) no_verdict: bool,
    /// When to write the derivation, in every format
    ///
    /// By default the text tree is printed on a terminal only if it fits:
    /// no line wider than the terminal and no more lines than
    /// `--screens` screens. Otherwise one line says how large the tree is
    /// and how to get it. Into a file or a pipe, and in every other
    /// format, the derivation is written whatever its size, up to
    /// `--derivation-limit`; `never` writes the verdict alone.
    #[arg(long, value_enum, value_name = "WHEN", default_value_t = Tree::Auto)]
    pub(crate) tree: Tree,
    /// How many screens of lines a text tree may fill and still be
    /// printed on a terminal by `--tree auto`, or `none` for any number
    #[arg(long, value_name = "N", value_parser = parse_count::<u32>, default_value_t = Count(Some(SCREENS)))]
    pub(crate) screens: Count<u32>,
    /// The most characters of a sequent in a verdict line, and of a list
    /// of formulas in an error report, or `none` for no limit; a longer
    /// one is cut with `…` and the number of its formulas
    #[arg(long, value_name = "CHARS", value_parser = parse_count::<u32>, default_value_t = Count(Some(ABBREVIATE)))]
    pub(crate) abbreviate: Count<u32>,
    /// How the output looks.
    #[command(flatten)]
    pub(crate) style: StyleArgs,
    /// The Rocq certificate's lemma and prelude.
    #[command(flatten)]
    pub(crate) rocq: RocqArgs,
    /// The largest derivation to build, by its estimated size: a number
    /// of bytes with a unit such as 64MiB or 2GiB, or `none` for no limit
    /// (limits.derivation_bytes)
    ///
    /// A derivation writes out the whole sequent at every inference and
    /// repeats every subproof that the proof shares, so it can be larger
    /// than the proof by any factor. A derivation estimated above the
    /// limit is not built, in any format: the verdict is reported without
    /// it, with a line that says how large it is, and the exit status is
    /// the verdict's. `--format json` writes the proof itself at any size.
    /// The drawing of a proof net (`--net`) is estimated before it is
    /// drawn and left out above the same limit.
    #[arg(long, value_name = "SIZE", value_parser = parse_bytes, default_value_t = Bytes(Some(Limits::DEFAULT_DERIVATION_BYTES)))]
    pub(crate) derivation_limit: Bytes,
    /// When a run of one structural rule, such as the weakenings of every
    /// unused `?` formula, is drawn as one inference labelled with a
    /// star (`?w*`), in every format but rocq (view.compact)
    #[arg(long, value_enum, value_name = "WHEN", default_value_t = CompactArg::Auto)]
    pub(crate) compact: CompactArg,
    /// How a derivation of classical linear logic is drawn: one-sided, or
    /// two-sided as the sequent was written, in every format but rocq
    /// (view.classical)
    ///
    /// Two-sided, every sequent has its formulas on the sides of `⊢` they
    /// were written on, the rules are named by side (`⊗L`, `⅋R`, `⊸L`,
    /// …), and where a negation was written a step `~L` or `~R` moves the
    /// formula across `⊢`. A sequent given as JSON, LLTP or .spec is
    /// written one-sided, its antecedents on the left. Derivations of
    /// intuitionistic linear logic are two-sided always, and those of the
    /// cyclic mode one-sided.
    #[arg(long, value_enum, value_name = "SIDES", default_value_t = ClassicalSidesArg::One)]
    pub(crate) classical_sides: ClassicalSidesArg,
}

/// How a derivation of classical linear logic is drawn.
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ClassicalSidesArg {
    /// One-sided, every formula right of `⊢`
    One,
    /// Two-sided, as the sequent was written
    Two,
}

impl From<ClassicalSidesArg> for linlog::ClassicalSides {
    /// Returns the library's value of the switch.
    fn from(sides: ClassicalSidesArg) -> Self {
        match sides {
            ClassicalSidesArg::One => Self::One,
            ClassicalSidesArg::Two => Self::Two,
        }
    }
}

/// When a derivation draws a run of one structural rule as one inference.
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CompactArg {
    /// Where the whole derivation would pass `--derivation-limit` or, as
    /// a text tree on a terminal, not fit it
    Auto,
    /// Always
    Always,
    /// Never: every rule its own inference
    Never,
}

impl From<CompactArg> for Compact {
    /// Returns the library's value of the switch.
    fn from(compact: CompactArg) -> Self {
        match compact {
            CompactArg::Auto => Self::Auto,
            CompactArg::Always => Self::Always,
            CompactArg::Never => Self::Never,
        }
    }
}

/// When a command writes the derivation of a proof.
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Tree {
    /// The text tree on a terminal if it fits, everything else always
    Auto,
    /// Whatever its size, up to `--derivation-limit`
    Always,
    /// Not at all
    Never,
}

/// How many screens of lines a text tree may fill on a terminal by
/// default.
pub(crate) const SCREENS: u32 = 3;

/// The most characters of a sequent in a verdict line by default: two
/// or three lines of a terminal.
pub(crate) const ABBREVIATE: u32 = 200;

/// How the outputs look: one options value per format, read from a file
/// and changed by flags.
#[derive(Args, Clone, Debug, Default)]
pub(crate) struct StyleArgs {
    /// Set an option of an output format, such as `labels=subscript`,
    /// `latex.open=bare` or `svg.font.family=monospace` (repeatable)
    /// (styles.FORMAT.KEY)
    ///
    /// KEY is a field of the format's options, dotted into nested ones,
    /// as `--style-file` holds them; without a prefix `text.`, `latex.`,
    /// `typst.`, `svg.`, `png.`, `pdf.` or `rocq.` it names the format
    /// `--format` gives.
    /// VALUE is JSON, or else a string: `gap=5`, `ids=true`,
    /// `open={"mark":"?"}`, `labels.table.⊸L=⊸_L`. The fields:
    /// text: labels, open, bar, gap; latex: form, labels, open, align,
    /// ebproof, preamble; typst: form, labels, open, import, page,
    /// layout (auto, curryst or linlog: curryst for a tree up to nine
    /// inferences high, linlog's own code above), premise_gap, label_gap,
    /// band, stroke (lengths for linlog's own layout: a number and pt,
    /// mm, cm, in or em); svg:
    /// font (family, advances), labels, open, ids, description, font_size,
    /// label_size, line_height, premise_gap, literal_gap, label_gap,
    /// margin, stroke_width, link_height, link_cap, node_radius, text,
    /// line, par, link, highlight, background, essential, input,
    /// dominator, net (formulas: a proof net with every subformula
    /// written, or compact: only its literals and conclusions written,
    /// its inner links circles), net_look (a net's look: textbook or
    /// linlog, the default, or its fields premises: dashed or plain,
    /// the edges into a ⅋ or ? node; conclusions: raised or bottom, a !
    /// with auxiliary doors written a row above the ? nodes or on the
    /// bottom row; why_not: node or rules, a ? node one node or the
    /// rules' dereliction, contraction and weakening nodes); png: scale, pixels; pdf:
    /// embed_text, compatible, accessible, title, language, date; rocq:
    /// form, lemma, prelude, kernel (where the command writes Rocq). `null` lifts a
    /// bound (`pixels=null`). Labels
    /// are upright, subscript, off or {"table":{RULE:LABEL}}; an open goal
    /// is dots, bare, dashed or {"mark":TEXT}
    #[arg(long = "style", value_name = "KEY=VALUE")]
    pub(crate) style: Vec<String>,
    /// Read the options of the output formats from a JSON file: an object
    /// with a key per format (text, latex, typst, svg, png, pdf, and rocq
    /// where the command writes Rocq), each holding the fields `--style` names; a field left out keeps its
    /// default (styles)
    #[arg(long, value_name = "PATH")]
    pub(crate) style_file: Option<PathBuf>,
    /// The math font a drawing (SVG, PNG, PDF) is set and laid out in:
    /// euler, stix-two, garamond, asana or new-computer-modern; by default
    /// Euler Math
    /// (styles.svg.font)
    #[arg(long, value_enum, value_name = "FONT")]
    pub(crate) font: Option<FontArg>,
}

/// A math font the command embeds, for its drawings.
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FontArg {
    /// Euler Math 0.75, upright, after Hermann Zapf's Euler for the AMS
    Euler,
    /// STIX Two Math 2.12, in the style of Times
    StixTwo,
    /// Garamond-Math 2022-01-03, matching EB Garamond
    Garamond,
    /// Asana Math 0.962, in the style of Palatino
    Asana,
    /// New Computer Modern Math 7.1.1 (Book), the Computer Modern of TeX
    NewComputerModern,
}

impl From<FontArg> for MathFont {
    /// Returns the library's math font.
    fn from(font: FontArg) -> Self {
        match font {
            FontArg::Euler => Self::Euler,
            FontArg::StixTwo => Self::StixTwo,
            FontArg::Garamond => Self::Garamond,
            FontArg::Asana => Self::Asana,
            FontArg::NewComputerModern => Self::NewComputerModern,
        }
    }
}

/// The Rocq certificate's flags, on the commands that write one.
#[derive(Args, Clone, Debug, Default)]
pub(crate) struct RocqArgs {
    /// The Rocq kernel a certificate of linear logic is written for; by
    /// default the mode's: nanoyalla for classical linear logic, linlog
    /// for Mix, affine and intuitionistic logic (styles.rocq.kernel)
    #[arg(long, value_enum, value_name = "KERNEL")]
    pub(crate) kernel: Option<KernelArg>,
    /// The name of the lemma of a Rocq certificate (styles.rocq.lemma)
    #[arg(long, value_name = "NAME")]
    pub(crate) lemma: Option<String>,
    /// The lines a standalone Rocq file starts with, before the lemma, in
    /// place of the certificate's own import (styles.rocq.prelude)
    #[arg(long, value_name = "TEXT")]
    pub(crate) prelude: Option<String>,
}

/// The Rocq kernel a certificate is written for.
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum KernelArg {
    /// NanoYalla 1.1.3, the kernel of Click & coLLecT: classical linear
    /// logic one-sided, a proof script checked rule by rule
    Nanoyalla,
    /// linlog's own Rocq library (rocq/ of the repository, the package
    /// rocq-linlog): every mode in its own calculus, intuitionistic
    /// logic two-sided, a proof term checked by the library's checker
    Linlog,
}

impl From<KernelArg> for Kernel {
    /// Returns the library's kernel.
    fn from(kernel: KernelArg) -> Self {
        match kernel {
            KernelArg::Nanoyalla => Self::NanoYalla,
            KernelArg::Linlog => Self::Linlog,
        }
    }
}

/// A number of bytes, or none for no limit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Bytes(pub Option<u64>);

impl std::fmt::Display for Bytes {
    /// Writes the bytes as a size flag reads them, in the largest unit
    /// that divides them.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            None => f.write_str("none"),
            Some(bytes) => {
                let (mut value, mut unit) = (bytes, 0);
                while unit + 1 < UNITS.len() && value >= 1024 && value.is_multiple_of(1024) {
                    (value, unit) = (value / 1024, unit + 1);
                }
                write!(f, "{value}{}", UNITS[unit])
            }
        }
    }
}

/// A number, or none for no bound.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Count<T>(pub Option<T>);

impl<T: std::fmt::Display> std::fmt::Display for Count<T> {
    /// Writes the number as a flag reads it.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            None => f.write_str("none"),
            Some(n) => write!(f, "{n}"),
        }
    }
}

/// Parses a number, or `none`.
fn parse_count<T: std::str::FromStr>(text: &str) -> Result<Count<T>, String> {
    if text == "none" {
        return Ok(Count(None));
    }
    text.parse()
        .map(|n| Count(Some(n)))
        .map_err(|_| format!("{text:?} is neither a number nor `none`"))
}

/// A time limit, or none.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Time(pub Option<Duration>);

impl std::fmt::Display for Time {
    /// Writes the limit as `--timeout` reads it.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            None => f.write_str("none"),
            Some(t) => write!(f, "{t:?}"),
        }
    }
}

/// Parses a time limit: a duration, or `none`.
fn parse_time(text: &str) -> Result<Time, String> {
    if text == "none" {
        return Ok(Time(None));
    }
    parse_duration(text).map(|t| Time(Some(t)))
}

/// The units of a size, each 1024 of the one before.
const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];

/// Parses a size such as `64MiB`, `2GiB` or `1000000` (bytes), or `none`.
fn parse_bytes(text: &str) -> Result<Bytes, String> {
    if text == "none" {
        return Ok(Bytes(None));
    }
    let split = text
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(text.len());
    let (number, unit) = text.split_at(split);
    let unit = unit.trim();
    let power = UNITS
        .iter()
        .position(|u| u.eq_ignore_ascii_case(unit) || (unit.is_empty() && *u == "B"))
        .ok_or_else(|| format!("unknown unit {unit:?}; use B, KiB, MiB, GiB or TiB, or `none`"))?;
    let number: u64 = number
        .parse()
        .map_err(|_| format!("{text:?} is not a size such as 64MiB or 2GiB, or `none`"))?;
    Ok(Bytes(Some(
        number.saturating_mul(1024u64.saturating_pow(power as u32)),
    )))
}

/// The output formats of `prove` and `check`.
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Format {
    /// The verdict on one line, then the derivation as a text tree
    Text,
    /// JSON: prove's outcome (verdict, fragment, mode, engine, statistics,
    /// and the proof, which `check` reads), one record a line in a batch;
    /// check's verdict and error
    Json,
    /// The verdict as a comment, then the derivation as a LaTeX proof tree
    /// of the ebproof package, with the connectives of cmll and amssymb
    Latex,
    /// The verdict as a comment, then the derivation as a Typst proof tree:
    /// with the curryst package for a low tree, in linlog's own layout for
    /// a high one (`--style typst.layout`)
    Typst,
    /// The verdict as an XML comment, then the derivation drawn as an SVG
    /// document, set in the Euler Math font unless `--font` names another
    Svg,
    /// The derivation drawn as a PNG image, the verdict on standard error;
    /// written to a file or a pipe, never to a terminal
    Png,
    /// The derivation drawn as a PDF document, its text selectable, the
    /// verdict on standard error; written to a file or a pipe, never to a
    /// terminal
    Pdf,
    /// The verdict as a comment, then the derivation as a Rocq
    /// certificate for the kernel `--kernel` names or the mode chooses:
    /// for NanoYalla 1.1.3, the kernel of Click & coLLecT (the nanoyalla
    /// directory of github.com/ComputerAidedLL/click-and-collect, built
    /// with Rocq 9 and its standard library, no Yalla needed), a lemma
    /// stating the sequent one-sided, proved rule by rule, which refuses
    /// Mix and the weakening of affine mode; for linlog's own library, a
    /// lemma stating the sequent in the mode's calculus, two-sided for
    /// intuitionistic logic, proved from the proof term by the library's
    /// checker; closed by Qed, `--standalone` adds the imports
    /// (`--prelude`), and `--lemma` names the lemma
    Rocq,
}

impl Format {
    /// Returns the format a file's extension names, if it names one.
    pub(crate) fn of_path(path: &std::path::Path) -> Option<Self> {
        let extension = path.extension()?.to_str()?.to_ascii_lowercase();
        Self::value_variants()
            .iter()
            .copied()
            .find(|format| format.extension() == extension)
    }

    /// Returns the extension of a file in the format.
    pub(crate) fn extension(self) -> &'static str {
        match self {
            Self::Text => "txt",
            Self::Json => "json",
            Self::Latex => "tex",
            Self::Typst => "typ",
            Self::Svg => "svg",
            Self::Png => "png",
            Self::Pdf => "pdf",
            Self::Rocq => "v",
        }
    }

    /// Returns the name the format's options have in the styles, if it
    /// has options.
    pub(crate) fn style_key(self) -> Option<&'static str> {
        Some(match self {
            Self::Text => "text",
            Self::Json => return None,
            Self::Latex => "latex",
            Self::Typst => "typst",
            Self::Svg => "svg",
            Self::Png => "png",
            Self::Pdf => "pdf",
            Self::Rocq => "rocq",
        })
    }

    /// Whether the format is binary, so that it goes to a file or a pipe
    /// and its verdict to standard error.
    pub(crate) fn is_binary(self) -> bool {
        matches!(self, Self::Png | Self::Pdf)
    }

    /// Whether the format has a document form, which `--standalone` asks
    /// for (an SVG is always a whole document).
    pub(crate) fn has_document(self) -> bool {
        matches!(self, Self::Latex | Self::Typst | Self::Rocq)
    }

    /// Returns the format's name as a message writes it.
    pub(crate) fn title(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Json => "JSON",
            Self::Latex => "LaTeX",
            Self::Typst => "Typst",
            Self::Svg => "SVG",
            Self::Png => "PNG",
            Self::Pdf => "PDF",
            Self::Rocq => "Rocq",
        }
    }
}

impl OutputArgs {
    /// Returns the format the output is written in: `--format`, else the
    /// one the output file's extension names, else text.
    pub(crate) fn format(&self) -> Format {
        self.format
            .or_else(|| self.output.as_deref().and_then(Format::of_path))
            .unwrap_or(Format::Text)
    }
}

/// The output formats of `seq print`.
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SequentFormat {
    /// Unicode text
    Text,
    /// LaTeX math, with the connectives of cmll and amssymb
    Latex,
    /// Typst math
    Typst,
    /// An SVG document of one line, set in the Euler Math font
    Svg,
    /// A PNG image of the SVG document's line
    Png,
    /// A PDF document of the SVG document's line
    Pdf,
}

impl From<SequentFormat> for Format {
    /// Returns the output format of the same name.
    fn from(format: SequentFormat) -> Self {
        match format {
            SequentFormat::Text => Self::Text,
            SequentFormat::Latex => Self::Latex,
            SequentFormat::Typst => Self::Typst,
            SequentFormat::Svg => Self::Svg,
            SequentFormat::Png => Self::Png,
            SequentFormat::Pdf => Self::Pdf,
        }
    }
}

/// The fragments `--fragment` names.
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FragmentArg {
    /// Multiplicatives: ⊗ ⅋
    Mll,
    /// Multiplicatives and their units: ⊗ ⅋ 1 ⊥
    MllUnits,
    /// Additives only: & ⊕ ⊤ 0
    All,
    /// Multiplicatives and additives with units
    Mall,
    /// Multiplicatives and exponentials with units
    Mell,
    /// Every connective
    Ll,
}

impl From<FragmentArg> for Fragment {
    /// Returns the fragment the name stands for.
    fn from(f: FragmentArg) -> Self {
        match f {
            FragmentArg::Mll => Fragment::MLL,
            FragmentArg::MllUnits => Fragment::MLL_WITH_UNITS,
            FragmentArg::All => Fragment::ADDITIVE,
            FragmentArg::Mall => Fragment::MALL,
            FragmentArg::Mell => Fragment::MELL,
            FragmentArg::Ll => Fragment::LL,
        }
    }
}

/// The engine `--engine` forces, or `None` for the automatic choice.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Forced(pub Option<Engine>);

/// Reads an engine's name, as the library spells it, or `auto`.
fn engine_name() -> impl TypedValueParser<Value = Forced> {
    clap::builder::PossibleValuesParser::new(
        std::iter::once("auto").chain(Engine::NAMES.iter().copied()),
    )
    .map(|name| Forced(name.parse().ok()))
}

/// The rules `--bias` names.
#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BiasArg {
    /// `rarer` in affine mode, `factors` in linear mode without
    /// exponentials, and in linear mode with exponentials both, each in a
    /// search of its own
    Auto,
    /// The literal with fewer occurrences is positive
    Rarer,
    /// The literal that is more often a direct factor of a tensor is
    /// positive, so that more splits are forced
    Factors,
}

impl From<BiasArg> for Bias {
    /// Returns the rule the name stands for.
    fn from(b: BiasArg) -> Self {
        match b {
            BiasArg::Auto => Bias::Auto,
            BiasArg::Rarer => Bias::Rarer,
            BiasArg::Factors => Bias::Factors,
        }
    }
}

/// Parses a duration such as `500ms`, `10s`, `1.5m` or `1h`; a bare number
/// is seconds.
fn parse_duration(text: &str) -> Result<Duration, String> {
    let split = text
        .find(|c: char| !(c.is_ascii_digit() || c == '.'))
        .unwrap_or(text.len());
    let (number, unit) = text.split_at(split);
    let seconds_per_unit = match unit.trim() {
        "ms" => 0.001,
        "" | "s" => 1.0,
        "m" | "min" => 60.0,
        "h" => 3600.0,
        _ => return Err(format!("unknown unit {unit:?}; use ms, s, m or h")),
    };
    let number: f64 = number
        .parse()
        .map_err(|_| format!("{text:?} is not a duration such as 500ms, 10s, 2m or 1h"))?;
    Duration::try_from_secs_f64(number * seconds_per_unit).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    /// The argument definitions are consistent, as clap checks them.
    #[test]
    fn arguments_are_consistent() {
        Cli::command().debug_assert();
    }

    /// The `--style` help names every field of every format's options,
    /// as the settings' JSON form has them.
    #[test]
    fn the_style_help_names_every_option() {
        let command = Cli::command();
        let prove = command.find_subcommand("prove").unwrap();
        let style = prove
            .get_arguments()
            .find(|a| a.get_id() == "style")
            .unwrap();
        let help = style.get_long_help().unwrap().to_string();
        let styles = serde_json::to_value(linlog::export::Styles::default()).unwrap();
        for (format, fields) in styles.as_object().unwrap() {
            assert!(help.contains(&format!("{format}:")), "{format}");
            for field in fields.as_object().unwrap().keys() {
                assert!(help.contains(field.as_str()), "{format}.{field}");
            }
        }
    }

    /// Durations take a unit, seconds by default.
    #[test]
    fn durations() {
        for (text, millis) in [
            ("500ms", 500),
            ("10s", 10_000),
            ("1.5m", 90_000),
            ("1h", 3_600_000),
            ("2", 2000),
        ] {
            assert_eq!(parse_duration(text), Ok(Duration::from_millis(millis)));
        }
        for text in ["", "s", "10 days", "-1s", "1e3s"] {
            assert!(parse_duration(text).is_err(), "{text:?}");
        }
    }
}
