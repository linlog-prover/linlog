// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use crate::argument_parsing::{CheckArgs, Format, OutputArgs, ProveArgs, SequentFormat, Tree};
use crate::io;
use crate::limit::{Deadline, Notice, detached};
use crate::settings::{Given, Threads, time_limit};
use crate::style;
use crate::{Status, catch_interrupt, exit_on_interrupt, interrupted};
use anyhow::{Context, Result, anyhow, bail};
use linlog::export::Styles;
use linlog::export::{Drawable, Form, latex, pdf, png, rocq, svg, typst};
use linlog::ordinary::Image;
use linlog::proofs::{Compact, Sides, TextOptions};
use linlog::search::{Engine, Goal, Options, Outcome, Reason, Verdict, prove_goal, race};
use linlog::{
    CheckError, Criterion, Derivation, Error, ErrorKind, Forest, Limits, Mode, Proof,
    ProofStructure, Reading, Refusal, Sequent, Size, ViewOptions,
};
use std::fmt::{Display, Write};
use std::io::{IsTerminal, Write as _};
use std::thread;
use std::time::{Duration, Instant};

/// Runs `f` on a thread with a stack of `size` bytes, as
/// [`Limits::stack_bytes`] sizes it for the recursion depth, and returns
/// its result.
pub(crate) fn on_large_stack<T: Send>(size: usize, f: impl FnOnce() -> T + Send) -> Result<T> {
    thread::scope(|scope| {
        let handle = thread::Builder::new()
            .name("search".into())
            .stack_size(size)
            .spawn_scoped(scope, f)
            .with_context(|| {
                format!(
                    "cannot start a search thread with a {} MiB stack",
                    size >> 20
                )
            })?;
        Ok(handle
            .join()
            .unwrap_or_else(|panic| std::panic::resume_unwind(panic)))
    })
}

/// Runs a search with the threads `threads` gives: a pool of them from
/// the start, or the library's race, one thread first and, once
/// `threads.alone` has passed, a pool of the others beside it, the first
/// of the two to decide answering, both within one memory bound
/// (`search::race`). `halt` is the command's stop condition. The single
/// search runs on the calling thread, whose stack the caller sized.
pub(crate) fn searched(
    goal: Goal<'_>,
    mode: linlog::Mode,
    options: &Options,
    limits: &linlog::Limits,
    threads: Threads,
    halt: &(dyn Fn() -> bool + Sync),
) -> Result<Outcome, linlog::Error> {
    match threads.alone {
        None => prove_goal(
            goal,
            mode,
            &options.clone().with_jobs(threads.jobs),
            limits,
            |_| halt(),
        ),
        Some(alone) => {
            let start = Instant::now();
            race(
                goal,
                mode,
                options,
                limits,
                threads.jobs,
                |_| start.elapsed() >= alone,
                |_| halt(),
            )
        }
    }
}

/// How long a search runs before a line on standard error says that it
/// does, when standard error is a terminal.
pub(crate) const NOTICE_AFTER: Duration = Duration::from_millis(500);

/// Returns the line on standard error while a search runs long: how long
/// it may run, whether it deepens its copy bound, and how to change that.
pub(crate) fn notice_line(limit: Option<Duration>, deepens: bool) -> String {
    let deepening = if deepens {
        ", deepening the copy bound"
    } else {
        ""
    };
    match limit {
        Some(t) => format!("searching for at most {t:?}{deepening}; --timeout changes the limit"),
        None => format!("searching without a time limit{deepening}; Ctrl-C stops it"),
    }
}

/// Returns why the command's stop condition fires, if it does: Ctrl-C
/// or the deadline.
pub(crate) fn stopped(deadline: &Deadline) -> Option<Stop> {
    if interrupted() {
        Some(Stop::Interrupt)
    } else if deadline.passed() {
        deadline.limit().map(Stop::Timeout)
    } else {
        None
    }
}

/// Why the stop condition fired.
#[derive(Clone, Copy)]
pub(crate) enum Stop {
    /// The time limit passed.
    Timeout(Duration),
    /// A batch's own time limit passed.
    BatchTimeout(Duration),
    /// The user pressed Ctrl-C.
    Interrupt,
}

impl Stop {
    /// Says why a derivation or a drawing was left out or cut short.
    pub(crate) fn why(stop: Option<Self>) -> String {
        match stop {
            Some(Self::Interrupt) => "interrupted".to_owned(),
            Some(Self::Timeout(t)) => format!("the time limit of {t:?} was reached"),
            Some(Self::BatchTimeout(t)) => format!("the batch's time limit of {t:?} was reached"),
            None => "stopped".to_owned(),
        }
    }
}

/// Returns the line of a sequent the time limit passed on before it was
/// read: nothing is known of it, so the line names no fragment and no
/// engine.
pub(crate) fn unread(limit: Duration) -> String {
    format!("unknown: the time limit of {limit:?} was reached while the sequent was read")
}

/// How many inferences are built, or pieces of text written, between two
/// looks at the time limit and the Ctrl-C flag while a derivation is made.
pub(crate) const STEPS_PER_CLOCK: u32 = 256;

/// How a derivation is to be shown: what the output arguments ask for and
/// where the output goes.
pub(crate) struct Show {
    /// The format.
    pub(crate) format: Format,
    /// Whether the proof net is written rather than the derivation.
    pub(crate) net: bool,
    /// Whether the output goes to a file, which is written whole or not
    /// at all.
    file: bool,
    /// The options of every format.
    pub(crate) styles: Styles,
    /// How a derivation is shown.
    pub(crate) view: ViewOptions,
    /// The bounds on what is built and on the memory it takes.
    pub(crate) limits: Limits,
    /// When the derivation is written.
    pub(crate) tree: Tree,
    /// The columns and rows of the terminal the output goes to, if it goes
    /// to one.
    terminal: Option<(u64, u64)>,
    /// How many screens of lines a text tree may fill on a terminal.
    screens: Option<u64>,
    /// Whether the verdict line is written, and so whether a newline
    /// separates the derivation from it.
    pub(crate) verdict: bool,
    /// The most characters of a sequent in a verdict line.
    abbreviate: Option<usize>,
}

impl Show {
    /// Reads the output arguments over the settings' styles, view and
    /// limits.
    pub(crate) fn new(output: &OutputArgs, settings: &linlog::Settings) -> Result<Self> {
        let format = output.format();
        if output.net
            && !matches!(
                format,
                Format::Text | Format::Svg | Format::Png | Format::Pdf
            )
        {
            bail!("--net writes the formats text, svg, png and pdf");
        }
        let stdout = std::io::stdout();
        if format.is_binary() && output.output.is_none() && stdout.is_terminal() {
            bail!(
                "a {} is not for a terminal: write it with --output FILE",
                format.title()
            );
        }
        form(
            output.standalone,
            matches!(format, Format::Latex | Format::Typst | Format::Rocq),
        )?;
        let styles = style::read(
            &settings.styles,
            &output.style,
            Some(&output.rocq),
            format.style_key(),
            output.standalone,
        )?;
        // The size of the terminal that standard output is, and of no
        // other stream's.
        let terminal = match (&output.output, stdout.is_terminal()) {
            (None, true) => Some(
                terminal_size::terminal_size_of(stdout)
                    .map_or((80, 24), |(w, h)| (u64::from(w.0), u64::from(h.0))),
            ),
            _ => None,
        };
        Ok(Self {
            format,
            net: output.net,
            file: output.output.is_some(),
            styles,
            view: match format {
                // A certificate names every formula it weakens.
                Format::Rocq => settings.view.with_compact(Compact::Never),
                _ => settings.view,
            },
            limits: settings.limits,
            tree: output.tree,
            terminal,
            screens: output.screens.0.map(u64::from),
            verdict: !output.no_verdict,
            abbreviate: output.abbreviate.0.map(|n| n as usize),
        })
    }

    /// A derivation of any size within `limits`, shown as `view` says, in
    /// `format`, wherever it goes, under `styles`: what a session writes
    /// when asked for a proof.
    pub(crate) fn session(
        format: Format,
        view: ViewOptions,
        limits: Limits,
        styles: Styles,
    ) -> Self {
        Self {
            format,
            net: false,
            file: false,
            styles,
            view,
            limits,
            tree: Tree::Always,
            terminal: None,
            screens: None,
            verdict: true,
            abbreviate: None,
        }
    }

    /// The columns and lines a text tree may take to be printed unasked,
    /// when the output is a terminal and the switch leaves it to the fit.
    pub(crate) fn fit(&self) -> Option<(u64, u64)> {
        match (self.tree, self.format, self.terminal) {
            (Tree::Auto, Format::Text, Some((columns, rows))) => Some((
                columns,
                self.screens.map_or(u64::MAX, |s| rows.saturating_mul(s)),
            )),
            _ => None,
        }
    }

    /// Writes the verdict line as a comment of the format at the start of
    /// the output, or for a binary format to standard error.
    fn verdict_line(&self, out: &mut impl Write, line: &str) -> std::fmt::Result {
        if self.format.is_binary() {
            eprintln!("{line}");
            return Ok(());
        }
        out.write_str(&note(self.format, line))
    }

    /// Whether the verdict waits for a derivation or a net to go with it:
    /// an output is made only when there is one, since a verdict alone is
    /// what the exit status says. So an SVG document is the drawing with
    /// the verdict as a comment before it or nothing (a comment alone is
    /// no XML document), and a file of any format but JSON holds a
    /// derivation or a net or is not made. JSON is always written: it is
    /// the outcome itself, with the refutation and the statistics. A
    /// binary format writes its verdict to standard error anyway.
    fn holds(&self) -> bool {
        !self.format.is_binary()
            && (self.format == Format::Svg || (self.file && self.format != Format::Json))
    }

    /// Writes the verdict line at the start of the output, unless the
    /// format holds it for its drawing, and returns what goes before the
    /// derivation or the net when one is written: a newline after a
    /// verdict line, the held verdict, or nothing.
    fn open(&self, out: &mut impl Write, line: Option<&str>) -> Result<Option<String>> {
        Ok(match line {
            Some(line) if self.format.is_binary() => {
                eprintln!("{line}");
                None
            }
            Some(line) if self.holds() => Some(format!("{}\n", note(self.format, line))),
            Some(line) => {
                self.verdict_line(out, line)?;
                self.separator().map(str::to_owned)
            }
            None => None,
        })
    }

    /// Writes a line after the derivation as a comment of the format, or
    /// to standard error where the output has no place for it: a binary
    /// format, or a held verdict whose drawing never came.
    fn after(&self, out: &mut impl Write, line: &str, drawn: bool) -> std::fmt::Result {
        if self.holds() && !drawn {
            eprintln!("{line}");
            return Ok(());
        }
        self.note(out, line)
    }

    /// Writes what became of the derivation, and returns whether the
    /// output is to be finished: rendered bytes; the line of a derivation
    /// left out; a held verdict whose derivation never came, to standard
    /// error. An output that holds its verdict ([`holds`](Self::holds)),
    /// and a binary one, is finished only with a derivation or a net in
    /// it, so without one no file is made. A derivation cut short in a
    /// file leaves no file either: the verdict and the reason go to
    /// standard error. Cut short on standard output, it stays as far as it
    /// came, with the reason after it, except in an output that holds its
    /// verdict: that is not finished, so the reason goes to standard error.
    fn close(&self, out: &mut io::Output, shown: Shown, line: Option<&str>) -> Result<bool> {
        let drawn = matches!(shown, Shown::Written | Shown::Rendered(_));
        match shown {
            Shown::Rendered(bytes) => out.stream().write_all(&bytes)?,
            Shown::LeftOut(reason) if self.holds() => eprintln!("{reason}"),
            Shown::LeftOut(reason) => self.left_out(out, &reason)?,
            Shown::Cut(reason) if self.file => {
                if let Some(line) = line.filter(|_| !self.format.is_binary()) {
                    eprintln!("{line}");
                }
                eprintln!("{reason}; the output file is not written");
                return Ok(false);
            }
            Shown::Cut(reason) if self.holds() => eprintln!("{reason}"),
            Shown::Cut(reason) => self.note(out, &reason)?,
            Shown::Written | Shown::Nothing => {}
        }
        if self.holds()
            && !drawn
            && let Some(line) = line
        {
            eprintln!("{line}");
        }
        Ok(drawn || !(self.holds() || self.format.is_binary()))
    }

    /// What goes between the verdict line and the derivation: a newline
    /// when the verdict line is written into the output.
    fn separator(&self) -> Option<&'static str> {
        (self.verdict && !self.format.is_binary()).then_some("\n")
    }

    /// Writes a line about a derivation that was left out: after the
    /// verdict when the output is a terminal, to standard error otherwise,
    /// so that a file or a pipe gets what it would get from a small proof.
    fn left_out(&self, out: &mut impl Write, line: &str) -> std::fmt::Result {
        if self.terminal.is_some() {
            self.note(out, line)
        } else {
            eprintln!("{line}");
            Ok(())
        }
    }

    /// Writes a line as a comment of the format on a line of its own,
    /// after what came before it if anything did; for a binary format, to
    /// standard error.
    fn note(&self, out: &mut impl Write, line: &str) -> std::fmt::Result {
        if self.format.is_binary() {
            eprintln!("{line}");
            return Ok(());
        }
        write!(out, "\n{}", note(self.format, line))
    }
}

/// What became of the derivation of a proof.
pub(crate) enum Shown {
    /// It was written.
    Written,
    /// It was drawn as these bytes of a binary format, for the caller to
    /// write.
    Rendered(Vec<u8>),
    /// It was left out, for the reason this line gives with the ways to
    /// get it.
    LeftOut(String),
    /// It was cut short while it was written, for the reason this line
    /// gives: what came before stays written.
    Cut(String),
    /// It was not asked for.
    Nothing,
}

/// A writer that writes a prefix before the first text it passes on, so
/// that what writes nothing leaves nothing.
pub(crate) struct Prefixed<'a, W> {
    /// Where the text goes.
    pub(crate) out: &'a mut W,
    /// The prefix, until it is written.
    pub(crate) prefix: Option<&'a str>,
}

impl<W: Write> Write for Prefixed<'_, W> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        if let Some(prefix) = self.prefix.take() {
            self.out.write_str(prefix)?;
        }
        self.out.write_str(s)
    }
}

/// A writer that passes on at most `left` characters and then fails,
/// setting `cut`.
struct Budget<'a, W> {
    /// Where the text goes.
    out: &'a mut W,
    /// The characters it still passes on.
    left: usize,
    /// Whether text was held back.
    cut: bool,
}

impl<W: Write> Write for Budget<'_, W> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        let n = s.chars().count();
        if n <= self.left {
            self.left -= n;
            return self.out.write_str(s);
        }
        let head: String = s.chars().take(self.left).collect();
        self.out.write_str(&head)?;
        (self.left, self.cut) = (0, true);
        Err(std::fmt::Error)
    }
}

/// Returns a value's text, cut after `limit` characters with `…` and
/// the number of formulas it has in all.
pub(crate) fn abbreviated(value: &impl Display, limit: Option<usize>, formulas: usize) -> String {
    let mut text = String::new();
    let Some(limit) = limit else {
        write!(text, "{value}").expect("a string takes any text");
        return text;
    };
    let mut budget = Budget {
        out: &mut text,
        left: limit,
        cut: false,
    };
    if write!(budget, "{value}").is_err() && budget.cut {
        write!(text, "… ({formulas} formulas)").expect("a string takes any text");
    }
    text
}

/// Returns a number of bytes in the largest binary unit that leaves it at
/// least one.
pub(crate) fn bytes_text(bytes: u64) -> String {
    if bytes == u64::MAX {
        return "more than 16 EiB".to_owned();
    }
    let units = ["B", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB"];
    let (mut value, mut unit) = (bytes as f64, 0);
    while value >= 1024.0 && unit + 1 < units.len() {
        (value, unit) = (value / 1024.0, unit + 1);
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {}", units[unit])
    }
}

/// Returns a count of a derivation's size as text; one that reached the
/// most its counter holds is only known to be beyond it.
pub(crate) fn count_text(count: u64) -> String {
    if count == u64::MAX {
        "more than 10¹⁹".to_owned()
    } else {
        count.to_string()
    }
}

/// Returns the line for a tree that does not fit the terminal.
pub(crate) fn unfit(inferences: u64, width: &str, lines: u64, columns: u64, most: u64) -> String {
    format!(
        "the proof tree is not shown: {} inferences, {width} columns by {} lines, for a \
         terminal of {columns} columns and at most {most} lines (--screens); print it with \
         --tree always, write it with --output FILE, or get the proof with --format json",
        count_text(inferences),
        count_text(lines)
    )
}

/// Returns the line for a derivation past the limit.
pub(crate) fn too_large(size: &Size, limit: u64, least: Option<u64>) -> String {
    // Drawn compact, the derivation is smaller, but by its lower bound
    // still over the limit.
    let compact = least
        .filter(|&least| least < size.bytes())
        .map(|least| format!(", and drawn compact at least {}", bytes_text(least)))
        .unwrap_or_default();
    format!(
        "the derivation is not written: its {} inferences with {} characters of sequents are \
         estimated at {}{compact}, over the limit of {}; --format json writes the proof itself, \
         --derivation-limit SIZE raises the limit and --derivation-limit none lifts it",
        count_text(size.inferences),
        count_text(size.characters),
        bytes_text(size.bytes()),
        bytes_text(limit)
    )
}

/// Returns the line for a derivation the view did not build: its size
/// over a bound, from `size` where it can be had, or the error's own words.
pub(crate) fn not_built(error: &Error, size: impl FnOnce() -> Option<Size>) -> String {
    let own = || format!("the derivation is not written: {error}");
    match error {
        Error::Refused(Refusal::Output {
            limit_bytes,
            least_bytes,
            ..
        }) => size().map_or_else(own, |size| too_large(&size, *limit_bytes, *least_bytes)),
        Error::Refused(Refusal::Memory { limit_bytes, .. }) => {
            size().map_or_else(own, |size| over_memory(&size, *limit_bytes))
        }
        Error::Check(CheckError::Refused(refused)) => match refused.refusal {
            Refusal::Memory { limit_bytes, .. } => proof_unread(limit_bytes),
            _ => own(),
        },
        _ => own(),
    }
}

/// Returns the line for a derivation whose proof could not be read
/// within the memory limit.
pub(crate) fn proof_unread(limit: u64) -> String {
    format!(
        "the derivation is not written: reading the proof for it takes more than the memory \
         limit of {}; --format json writes the proof itself, and --memory-limit SIZE raises \
         the limit",
        bytes_text(limit)
    )
}

/// Returns the line for a derivation past the memory limit.
pub(crate) fn over_memory(size: &Size, limit: u64) -> String {
    format!(
        "the derivation is not written: its {} inferences with {} characters of sequents are \
         estimated at {}, over the memory limit of {}; --format json writes the proof itself, \
         and --memory-limit SIZE raises the limit",
        count_text(size.inferences),
        count_text(size.characters),
        bytes_text(size.bytes()),
        bytes_text(limit)
    )
}

/// Writes the derivation of a proof into `out` as `show` asks, two-sided
/// in intuitionistic mode, after a newline: a text tree, a LaTeX or
/// Typst proof tree, an SVG document or a Rocq script; or says why it is
/// left out: a text tree that does not fit the terminal, a derivation
/// past the limit, or one that `halt` stopped, which is polled as it is
/// built and written and whose reason `why` then gives. A derivation cut
/// short by `halt` stays written as far as it came. Fails with the
/// checker's complaint, with formulas, on a proof that is none, and on a
/// derivation that has no certificate, before anything is written.
pub(crate) fn derivation(
    proof: &Proof,
    mode: Mode,
    show: &Show,
    mut halt: impl FnMut() -> bool,
    why: impl Fn() -> String,
    prefix: Option<&str>,
    out: &mut impl Write,
) -> Result<Shown> {
    if show.tree == Tree::Never {
        return Ok(Shown::Nothing);
    }
    let invalid = |e: linlog::CheckError| {
        anyhow!(
            "the proof is invalid: {}",
            e.describe(proof.forest()).abbreviated(show.abbreviate)
        )
    };
    let stopped = || Shown::LeftOut(format!("the derivation is not written: {}", why()));
    // A tree that cannot fit is known from its size alone, before
    // anything is built; the compact one may fit where it does not.
    let fit = show.fit();
    let sides = if mode.is_intuitionistic() {
        Sides::Two
    } else {
        Sides::One
    };
    let mut view = show.view.with_sides(sides);
    if let Some((columns, most)) = fit {
        let size = match proof.derivation_size(&view, &show.limits, |_| halt()) {
            Ok(size) => size,
            // No verdict on the proof: the pass was stopped or given up.
            Err(CheckError::Refused(refused))
                if matches!(refused.refusal, Refusal::Stopped { .. }) =>
            {
                return Ok(stopped());
            }
            Err(CheckError::Refused(_)) => {
                return Ok(Shown::LeftOut(proof_unread(
                    show.limits.memory_bytes.unwrap_or(0),
                )));
            }
            Err(e) => return Err(invalid(e)),
        };
        if (size.width > columns || size.lines() > most) && view.compact != Compact::Never {
            view = view.with_compact(Compact::Always);
        } else if size.width > columns || size.lines() > most {
            let width = format!("at least {}", size.width);
            return Ok(Shown::LeftOut(unfit(
                size.inferences,
                &width,
                size.lines(),
                columns,
                most,
            )));
        }
    }
    let built = proof.derivation_within(&view, &show.limits, |_| halt());
    let d = match built {
        Ok(d) => d,
        Err(Error::Check(e @ CheckError::Invalid(_))) => return Err(invalid(e)),
        Err(Error::Refused(Refusal::Stopped { .. })) => return Ok(stopped()),
        Err(Error::Check(CheckError::Refused(refused)))
            if matches!(refused.refusal, Refusal::Stopped { .. }) =>
        {
            return Ok(stopped());
        }
        Err(error) => {
            let size = || {
                proof
                    .derivation_size(&view, &Limits::default(), |_| false)
                    .ok()
            };
            return Ok(Shown::LeftOut(not_built(&error, size)));
        }
    };
    shown(d, fit, show, halt, why, prefix, out)
}

/// A derivation the command writes: linear, or read back as LK or LJ.
pub(crate) trait Written {
    /// The derivation as the exports take it.
    fn drawable(&self) -> Drawable<'_>;
    /// The widest line and the lines of its text tree.
    fn text_size(&self, options: &TextOptions) -> (usize, usize);
    /// How many inferences it has.
    fn inferences(&self) -> usize;
    /// Writes its text tree.
    fn write_text(
        &self,
        options: &TextOptions,
        out: &mut impl Write,
        stop: impl FnMut(linlog::Progress) -> bool,
    ) -> Result<(), Error>;
}

impl Written for Derivation<'_> {
    fn drawable(&self) -> Drawable<'_> {
        self.into()
    }

    fn text_size(&self, options: &TextOptions) -> (usize, usize) {
        Derivation::text_size(self, options)
    }

    fn inferences(&self) -> usize {
        Derivation::inferences(self).len()
    }

    fn write_text(
        &self,
        options: &TextOptions,
        out: &mut impl Write,
        stop: impl FnMut(linlog::Progress) -> bool,
    ) -> Result<(), Error> {
        Derivation::write_text(self, options, out, stop)
    }
}

impl Written for linlog::ordinary::Derivation {
    fn drawable(&self) -> Drawable<'_> {
        self.into()
    }

    fn text_size(&self, options: &TextOptions) -> (usize, usize) {
        Self::text_size(self, options)
    }

    fn inferences(&self) -> usize {
        Self::inferences(self).len()
    }

    fn write_text(
        &self,
        options: &TextOptions,
        out: &mut impl Write,
        stop: impl FnMut(linlog::Progress) -> bool,
    ) -> Result<(), Error> {
        Self::write_text(self, options, out, stop)
    }
}

/// A string that takes text up to `most` bytes and refuses the rest: a
/// drawing too large to render within the memory bound is never held
/// whole.
struct Bounded {
    /// The text taken.
    text: String,
    /// The most bytes it takes.
    most: usize,
    /// Whether text was refused.
    over: bool,
}

impl Write for Bounded {
    /// Takes the text, or refuses it past the bound.
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        if self.text.len().saturating_add(s.len()) > self.most {
            self.over = true;
            return Err(std::fmt::Error);
        }
        self.text.push_str(s);
        Ok(())
    }
}

/// Writes a derivation that is built in the format `show` names, and
/// returns what became of it: left out where it does not fit the terminal
/// (`fit`), rendered for PNG and PDF from an SVG held within the memory
/// bound, else written into `out` after `prefix`, under `halt`; `why`
/// says what stopped it.
pub(crate) fn shown(
    d: impl Written,
    fit: Option<(u64, u64)>,
    show: &Show,
    mut halt: impl FnMut() -> bool,
    why: impl Fn() -> String,
    prefix: Option<&str>,
    out: &mut impl Write,
) -> Result<Shown> {
    let stopped = || Shown::LeftOut(format!("the derivation is not written: {}", why()));
    let styles = &show.styles;
    if let Some((columns, most)) = fit {
        let (width, lines) = d.text_size(&styles.text);
        let (width, lines) = (width as u64, lines as u64);
        if width > columns || lines > most {
            return Ok(Shown::LeftOut(unfit(
                d.inferences() as u64,
                &width.to_string(),
                lines,
                columns,
                most,
            )));
        }
    }
    if show.format.is_binary() {
        let memory = show.limits.memory_bytes;
        let mut drawing = Bounded {
            text: String::new(),
            most: memory.map_or(usize::MAX, |m| usize::try_from(m).unwrap_or(usize::MAX)),
            over: false,
        };
        let written = svg::write(d.drawable(), &styles.svg, &mut drawing, |_| halt());
        if drawing.over {
            return Ok(Shown::LeftOut(format!(
                "the drawing is not rendered: its SVG passes the memory limit of {}; \
                 --memory-limit SIZE raises the limit, and the svg format writes the drawing \
                 without rendering it",
                bytes_text(memory.unwrap_or(0))
            )));
        }
        if written.is_err() {
            return Ok(stopped());
        }
        drop(d);
        // `halt` looks at its condition once in `STEPS_PER_CLOCK` calls,
        // and the render's wait asks every few milliseconds.
        let halt = std::cell::RefCell::new(halt);
        let stop = || (0..STEPS_PER_CLOCK).any(|_| (halt.borrow_mut())());
        return Ok(
            match render(drawing.text, show.format, styles, memory, &stop)? {
                Rendered::Bytes(bytes) => Shown::Rendered(bytes),
                Rendered::Refused(line) => Shown::LeftOut(line),
                Rendered::Stopped => stopped(),
            },
        );
    }
    let mut out = Prefixed { out, prefix };
    let written = match show.format {
        Format::Latex => latex::write(d.drawable(), &styles.latex, &mut out, |_| halt()),
        Format::Typst => typst::write(d.drawable(), &styles.typst, &mut out, |_| halt()),
        Format::Svg => svg::write(d.drawable(), &styles.svg, &mut out, |_| halt()),
        Format::Rocq => rocq::write(d.drawable(), &styles.rocq, &mut out, |_| halt()),
        Format::Text | Format::Json | Format::Png | Format::Pdf => {
            d.write_text(&styles.text, &mut out, |_| halt())
        }
    };
    match written {
        Ok(()) => Ok(Shown::Written),
        Err(Error::Refused(Refusal::Stopped { .. })) => Ok(Shown::Cut(format!(
            "the derivation is cut short: {}",
            why()
        ))),
        Err(Error::Unsupported(e)) => Err(anyhow!(e).context("no certificate")),
        // The output's own error, which finishing it reports.
        Err(_) => Ok(Shown::Written),
    }
}

/// Returns a sequent in a format, one-sided, or two-sided in
/// intuitionistic mode when it has an intuitionistic reading, under the
/// format's options; for PNG and PDF, the SVG document to render. A
/// LaTeX, Typst or SVG text estimated past `limits.derivation_bytes` is
/// refused before anything is laid out.
///
/// # Errors
///
/// A sequent without an intuitionistic reading in intuitionistic mode,
/// described with its formulas, and a text past the bound.
pub(crate) fn sequent_in(
    sequent: &Sequent,
    mode: Mode,
    format: SequentFormat,
    styles: &Styles,
    limits: &Limits,
) -> Result<String> {
    // The sequent was admitted when it was read; the reading's error is
    // described with formulas before the library prints it.
    let limits = &limits.with_occurrences(None);
    let text = if mode.is_intuitionistic() {
        let forest = Forest::within(sequent, limits)?;
        let reading = Reading::new(&forest)
            .map_err(|e| anyhow!("not an intuitionistic sequent: {}", e.describe(&forest)))?;
        reading.to_string()
    } else {
        sequent.to_string()
    };
    match format {
        SequentFormat::Text => Ok(text),
        SequentFormat::Latex => latex::sequent(sequent, mode, &styles.latex, limits),
        SequentFormat::Typst => typst::sequent(sequent, mode, &styles.typst, limits),
        SequentFormat::Svg | SequentFormat::Png | SequentFormat::Pdf => {
            svg::sequent(sequent, mode, &styles.svg, limits)
        }
    }
    .map_err(|e| match e {
        Error::Refused(Refusal::Output { .. }) => {
            anyhow!("{e}; raise the limit with a settings file's limits.derivation_bytes")
        }
        e => e.into(),
    })
}

/// Returns the form `--standalone` asks for, which only the LaTeX, Typst
/// and Rocq formats (`exported`) have: the others have one form, an SVG
/// always being a document, so the flag would change nothing.
///
/// # Errors
///
/// `--standalone` beside a format that has one form.
pub(crate) fn form(standalone: bool, exported: bool) -> Result<Form> {
    match (standalone, exported) {
        (false, _) => Ok(Form::Fragment),
        (true, true) => Ok(Form::Standalone),
        (true, false) => bail!("--standalone needs --format latex, typst or rocq"),
    }
}

/// Returns a line or lines of text as the format writes them next to its
/// output: as they are, or as LaTeX, Typst, XML or Rocq comments.
pub(crate) fn note(format: Format, text: &str) -> String {
    let comment = |line: &str| match format {
        Format::Latex => format!("% {line}"),
        Format::Typst => format!("// {line}"),
        // Rocq nests comments and reads string literals inside them, so a
        // `(*`, a `*)` or an odd number of `"` would end the comment early
        // or never, and the text after it would be read as vernacular.
        Format::Rocq => format!(
            "(* {} *)",
            line.replace("(*", "( *")
                .replace("*)", "* )")
                .replace('"', "\"\"")
        ),
        // An XML comment cannot hold `--`, which flag names bring.
        Format::Svg => format!("<!-- {} -->", line.replace('-', "\u{2010}")),
        Format::Text | Format::Json | Format::Png | Format::Pdf => line.to_owned(),
    };
    let lines: Vec<String> = text.lines().map(comment).collect();
    lines.join("\n")
}

/// Returns a search error with formulas where the library's message has
/// occurrence ids.
pub(crate) fn describe(error: Error, sequent: &Sequent) -> anyhow::Error {
    match (&error, Forest::within(sequent, &Limits::UNBOUNDED)) {
        (Error::NotIntuitionistic(_), Ok(forest)) => anyhow!("{}", error.describe(&forest)),
        (Error::Check(CheckError::Refused(_)), _) => anyhow!(
            "the search found a proof, but {error}; raise the limit with --memory-limit, or take \
             the proof unchecked with --no-check"
        ),
        _ => error.into(),
    }
}

/// The Euler Math font, which the PNG and PDF drawings are set in; its
/// licence is `fonts/OFL.txt`.
const FONT: &[u8] = include_bytes!("../fonts/Euler-Math.otf");

/// What became of a drawing handed to a renderer.
pub(crate) enum Rendered {
    /// The bytes of the image or the document.
    Bytes(Vec<u8>),
    /// A bound refused it before the render began, for the reason this
    /// line gives with the flag that raises the bound.
    Refused(String),
    /// The stop condition fired before the render ended.
    Stopped,
}

/// Renders an SVG document in a binary format, PNG or PDF, with the Euler
/// Math font and whatever font the style names that the renderer knows
/// of, on a thread of its own while `stop` is asked every few
/// milliseconds: the renderers cannot be stopped from inside. Returns the
/// bytes, the line of a bound that refused the drawing before it was
/// parsed, or that `stop` fired; the thread of a stopped render is left
/// to finish, within the memory bound that its estimate kept, and what
/// it makes is dropped. `memory` is the render's bound, the command's
/// `--memory-limit`.
pub(crate) fn render(
    svg: String,
    format: Format,
    styles: &Styles,
    memory: Option<u64>,
    stop: &dyn Fn() -> bool,
) -> Result<Rendered> {
    let limits = Limits::default().with_memory_bytes(memory);
    let rendered = match format {
        Format::Png => {
            let options = styles.png.clone();
            detached(
                "render",
                move || png::from_svg(&svg, &[FONT], &options, &limits),
                stop,
            )?
        }
        _ => {
            let mut options = styles.pdf.clone();
            if options.date.is_none() {
                options.date = Some(made()?);
            }
            if options.accessible {
                eprintln!(
                    "note: the accessible PDF tags the drawing as one figure; a screen reader \
                     reads its description, a numbered reading of the proof, not the drawing, \
                     and a long proof may still be hard to follow"
                );
            }
            detached(
                "render",
                move || pdf::from_svg(&svg, &[FONT], &options, &limits),
                stop,
            )?
        }
    };
    Ok(match rendered {
        None => Rendered::Stopped,
        Some(Ok(bytes)) => Rendered::Bytes(bytes),
        Some(Err(Error::Refused(Refusal::Memory {
            needed_bytes: Some(estimate),
            limit_bytes: limit,
            ..
        }))) => Rendered::Refused(format!(
            "the drawing is not rendered: the {} is estimated to take {}, over the memory limit \
             of {}; --memory-limit SIZE raises the limit, and the svg format writes the drawing \
             without rendering it",
            format.title(),
            bytes_text(estimate),
            bytes_text(limit)
        )),
        Some(Err(Error::Refused(Refusal::Pixels { pixels, limit, .. }))) => {
            Rendered::Refused(format!(
                "the drawing is not rendered: the image would have {pixels} pixels, over the limit \
             of {limit}; --style png.pixels=N raises the limit, and the svg format writes the \
             drawing without rendering it"
            ))
        }
        Some(Err(error)) => return Err(error.into()),
    })
}

/// Returns the date a document is made: `SOURCE_DATE_EPOCH`, the seconds
/// since the Unix epoch that reproducible builds set, or else the clock.
fn made() -> Result<pdf::Date> {
    let seconds = match std::env::var("SOURCE_DATE_EPOCH") {
        Ok(text) => text
            .trim()
            .parse()
            .with_context(|| format!("SOURCE_DATE_EPOCH is {text:?}, not a number of seconds"))?,
        Err(_) => std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |since| since.as_secs()),
    };
    Ok(pdf::Date::from_unix(seconds))
}

/// Returns the line for a proof net whose drawing is past the limit.
fn net_too_large(net: &ProofStructure, error: &Error) -> String {
    let Error::Refused(Refusal::Output {
        estimate_bytes,
        limit_bytes,
        ..
    }) = *error
    else {
        return format!("the proof net is not drawn: {error}");
    };
    format!(
        "the proof net is not drawn: its {} occurrences and {} links are estimated at {}, over \
         the limit of {}; the text format writes the net itself, --derivation-limit SIZE raises \
         the limit and --derivation-limit none lifts it",
        net.forest().len(),
        net.links().len(),
        bytes_text(estimate_bytes),
        bytes_text(limit_bytes)
    )
}

/// Writes the proof net of a proof into `out` as `show` asks: as text, or
/// drawn as an SVG document, a PNG image or a PDF document; or says why
/// the drawing is left out: it is estimated past the derivation limit, a
/// bound of the render refused it, or `stop` fired while it was rendered,
/// for the reason `why` gives. `found` is the net the net engine found,
/// if it ran.
#[allow(clippy::too_many_arguments)]
pub(crate) fn net_into(
    found: Option<&ProofStructure>,
    proof: &Proof,
    mode: Mode,
    show: &Show,
    stop: &dyn Fn() -> bool,
    why: &dyn Fn() -> String,
    prefix: Option<&str>,
    out: &mut io::Output,
) -> Result<Shown> {
    let made;
    let net = match found {
        Some(net) => net,
        None => {
            let criterion = Criterion::of(mode)?;
            match ProofStructure::from_proof(proof, criterion, &show.limits, |_| stop()) {
                Ok(net) => made = net,
                Err(error) if error.kind() == ErrorKind::Stopped => {
                    return Ok(Shown::LeftOut(format!(
                        "the proof net is not drawn: {}",
                        why()
                    )));
                }
                Err(error) if error.is_refusal() => {
                    return Ok(Shown::LeftOut(format!(
                        "the proof net is not made: {error}"
                    )));
                }
                Err(error) => return Err(error.into()),
            }
            &made
        }
    };
    let separator = prefix.unwrap_or("");
    if !matches!(show.format, Format::Svg | Format::Png | Format::Pdf) {
        write!(out, "{separator}{net}")?;
        return Ok(Shown::Written);
    }
    let drawing = match svg::net(net, &show.styles.svg, &show.limits, |_| stop()) {
        Ok(drawing) => drawing,
        Err(error) => return Ok(Shown::LeftOut(net_too_large(net, &error))),
    };
    if show.format == Format::Svg {
        write!(out, "{separator}{drawing}")?;
        return Ok(Shown::Written);
    }
    Ok(
        match render(
            drawing,
            show.format,
            &show.styles,
            show.limits.memory_bytes,
            stop,
        )? {
            Rendered::Bytes(bytes) => Shown::Rendered(bytes),
            Rendered::Refused(line) => Shown::LeftOut(line),
            Rendered::Stopped => Shown::LeftOut(format!("the proof net is not drawn: {}", why())),
        },
    )
}

/// Fails unless proof nets exist for the sequent in the mode: unit-free
/// MLL, linear, with or without Mix, classical or intuitionistic (where
/// the net is the one of the one-sided sequent).
pub(crate) fn nets_exist(sequent: &Sequent, mode: Mode) -> Result<()> {
    if mode.is_affine() {
        bail!("proof nets exist in linear mode only, with or without --mix, not in {mode} mode");
    }
    Ok(linlog::nets::exist(sequent.fragment(), mode)?)
}

/// What `prove` answers when the time limit passed before the sequent was
/// read: the verdict is unknown, and the line names no fragment and no
/// engine, since nothing is known of the sequent. The line is the output,
/// as a comment of the format; JSON has no form for it, so there it goes
/// to standard error.
fn unread_into(args: &ProveArgs, limit: Duration) -> Result<Status> {
    let line = unread(limit);
    // A file holds a derivation or nothing, and JSON has no form for this.
    match (args.output.format(), &args.output.output) {
        (Format::Json, _) | (_, Some(_)) => eprintln!("{line}"),
        (format, None) if format.is_binary() => eprintln!("{line}"),
        (format, None) => io::write(None, &note(format, &line))?,
    }
    Ok(Status::Unknown)
}

/// Runs `prove`: reads the sequent, searches on a large stack, and prints
/// the verdict line, the derivation or the proof net and the statistics,
/// or the outcome as JSON. A time limit counts from here: the sequent is
/// read, parsed and laid out as a forest under it, on a thread the
/// command stops waiting for when the limit passes.
///
/// # Errors
///
/// An input that cannot be read, a sequent the search refuses (an engine
/// or a fragment that does not apply), an output that cannot be made, or
/// a proof the checker rejects.
pub(crate) fn prove(args: &ProveArgs, given: &Given) -> Result<Status> {
    let start = Instant::now();
    let settings = args.settings(given)?;
    if crate::batch::is_batch(args) {
        return crate::batch::run(args, &settings, start);
    }
    ordinary_mode(args)?;
    let deadline = Deadline::start(time_limit(&settings), start)?;
    let input = args.input.clone();
    let limits = settings.limits;
    let ordinary = crate::settings::ordinary(&settings, &args.logic);
    let loaded = deadline.within(move || match ordinary {
        Some(options) => input
            .image(&options, &limits)
            .map(|(forest, image)| (forest, Some(image))),
        None => input.forest(&limits).map(|forest| (forest, None)),
    })?;
    let Some(forest) = loaded else {
        let limit = deadline.limit().expect("only a limit passes");
        return unread_into(args, limit);
    };
    let (forest, image) = forest?;
    let sequent = forest.sequent();
    let mode = image
        .as_ref()
        .map_or_else(|| args.input.mode(args.mode.mode()), Image::mode);
    if args.output.net {
        nets_exist(sequent, mode)?;
    }
    let options = &settings.search;
    let limits = settings.limits;
    let threads = Threads::of(&settings);
    let deepens = options.copies.is_none() && sequent.fragment().has_exponentials();
    let format = args.output.format();
    let quiet = args.output.quiet;
    let show = Show::new(&args.output, &settings)?;
    // An output file that cannot be made is found before the search, which
    // it would lose: an output dropped unfinished leaves no file, and the
    // file at the path stays as it was. Only a file, or a path that does
    // not exist yet: a pipe opened and closed would send its reader the end.
    if let Some(path) = &args.output.output
        && std::fs::metadata(path).map_or(true, |m| m.is_file())
    {
        drop(io::Output::open(Some(path), format.is_binary())?);
    }
    catch_interrupt();

    let status = on_large_stack(limits.stack_bytes(), || {
        let start = Instant::now();
        let notice = Notice::start(NOTICE_AFTER, notice_line(deadline.limit(), deepens));
        // Both conditions are flags, so every poll asks both.
        let halt = || interrupted() || deadline.passed();
        let outcome = searched(
            Goal::conclusion(&forest),
            mode,
            options,
            &limits,
            threads,
            &halt,
        )
        .map_err(|e| describe(e, sequent))?;
        let stop = stopped(&deadline);
        drop(notice);
        let elapsed = start.elapsed();
        let status = Status::from(&outcome.verdict);
        let mut out = io::Output::open(args.output.output.as_deref(), format.is_binary())?;
        if format == Format::Json {
            serde_json::to_writer(out.stream(), &outcome)?;
            out.finish()?;
            return Ok(status);
        }
        // The verdict first, then the derivation as it is made.
        let ended = Ended { stop, elapsed };
        let line = show.verdict.then(|| match &image {
            Some(image) => crate::ordinary::verdict_line(&outcome, image, &ended),
            None => verdict_line(&outcome, options.fragment.is_some(), &ended),
        });
        let prefix = show.open(&mut out, line.as_deref())?;
        // The time limit and Ctrl-C hold for the derivation as for the
        // search.
        // A reader that closed the pipe wants no more of the derivation.
        let over = || interrupted() || deadline.passed() || io::closed();
        let mut steps = 0u32;
        let halt = || {
            steps = steps.wrapping_add(1);
            steps.is_multiple_of(STEPS_PER_CLOCK) && over()
        };
        let why = || Stop::why(stopped(&deadline));
        let shown = match (&outcome.verdict, format, quiet) {
            (Verdict::Proved(proof), _, false) if show.net => {
                let found = outcome.net.as_deref();
                net_into(
                    found,
                    proof,
                    mode,
                    &show,
                    &over,
                    &why,
                    prefix.as_deref(),
                    &mut out,
                )?
            }
            (Verdict::Proved(_), _, false) if over() => {
                Shown::LeftOut(format!("the derivation is not written: {}", why()))
            }
            (Verdict::Proved(proof), _, false) => match (&image, args.linear) {
                (Some(image), false) => crate::ordinary::derivation(
                    image,
                    proof,
                    &show,
                    halt,
                    why,
                    prefix.as_deref(),
                    &mut out,
                )?,
                _ => derivation(proof, mode, &show, halt, why, prefix.as_deref(), &mut out)?,
            },
            _ => Shown::Nothing,
        };
        let drawn = matches!(shown, Shown::Written | Shown::Rendered(_));
        let keep = show.close(&mut out, shown, line.as_deref())?;
        if args.stats {
            show.after(&mut out, &statistics(&outcome, elapsed), drawn)?;
        }
        if keep {
            out.finish()?;
        }
        anyhow::Ok(status)
    })??;
    Ok(status)
}

/// Refuses a mode flag beside `--logic`, whose translation decides the
/// mode.
pub(crate) fn ordinary_mode(args: &ProveArgs) -> Result<()> {
    let mode = &args.mode;
    if args.logic.logic.is_some() && (mode.intuitionistic || mode.affine || mode.mix) {
        bail!("the logic's translation decides the mode: leave out -i, -a and --mix");
    }
    Ok(())
}

/// How a search ended, as far as the command knows it beyond the outcome.
pub(crate) struct Ended {
    /// Why the command's stop condition fired, if it did.
    pub(crate) stop: Option<Stop>,
    /// How long the search took.
    pub(crate) elapsed: Duration,
}

/// Returns the first line of the text output: the verdict, where the
/// search ran, and for an undecided sequent why.
pub(crate) fn verdict_line(outcome: &Outcome, asserted: bool, ended: &Ended) -> String {
    let context = format!(
        "{}{}, {}, {} engine",
        outcome.fragment.name_in(outcome.mode),
        if asserted { " as asserted" } else { "" },
        outcome.mode,
        outcome.engine
    );
    match &outcome.verdict {
        Verdict::Proved(_) => format!("provable ({context})"),
        Verdict::Unprovable(refutation) => format!("unprovable ({context}): {refutation}"),
        Verdict::Unknown(reason) => {
            format!("unknown ({context}): {}", unknown(*reason, outcome, ended))
        }
    }
}

/// Returns why a search did not decide, for each way it can end: the
/// bound or limit it reached, after how long, at which copy bound when it
/// deepened one, and the flag that changes it.
pub(crate) fn unknown(reason: Reason, outcome: &Outcome, ended: &Ended) -> String {
    let deepened = outcome.fragment.has_exponentials()
        && matches!(outcome.engine, Engine::Focus | Engine::TwoSided);
    let at = if deepened {
        format!(" at a copy bound of {}", outcome.statistics.copies)
    } else {
        String::new()
    };
    let after = format!("after {:.2?}", ended.elapsed);
    match (reason, ended.stop) {
        (Reason::Stopped, Some(Stop::Timeout(t))) => {
            format!(
                "the time limit of {t:?} was reached{at}; --timeout DURATION gives the search longer"
            )
        }
        (Reason::Stopped, Some(Stop::BatchTimeout(t))) => format!(
            "the batch's time limit of {t:?} was reached{at}; --batch-timeout DURATION gives the \
             batch longer"
        ),
        (Reason::Stopped, Some(Stop::Interrupt)) => format!("interrupted {after}{at}"),
        (Reason::WorkLimit { .. }, _) => {
            format!("{reason} {after}{at}; raise it with --work-limit N")
        }
        (Reason::CopyBound { .. }, _) => format!(
            "{reason} {after}; raise it with --copies N, or lift it with --copies none to deepen \
             it while the time limit lasts"
        ),
        (Reason::RecursionLimit { depth, .. }, _) => format!(
            "the recursion limit of {depth} was reached {after}{at}; raise it with \
             --recursion-limit N"
        ),
        (Reason::MemoryLimit { .. } | Reason::Unchecked { .. }, _) => {
            format!("{reason} {after}{at}; raise it with --memory-limit SIZE")
        }
        // Any other reason, which no flag changes.
        _ => format!("{reason} {after}{at}"),
    }
}

/// Returns the statistics as text, one line per counter the engine that
/// ran keeps (`Engine::counters`, with the library's labels), then the
/// time. The copy bound reached is left out where the fragment has no
/// exponentials, and the forward search's where it did not run.
pub(crate) fn statistics(outcome: &Outcome, elapsed: Duration) -> String {
    // The counters by their keys in the statistics' JSON form.
    let values = serde_json::to_value(outcome.statistics).unwrap_or_default();
    let mut lines = String::new();
    for counter in outcome.engine.counters() {
        let value = values
            .get(counter.key)
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
        let idle = match counter.key {
            "copies" => !outcome.fragment.has_exponentials(),
            "forward_copies" => value == 0,
            _ => false,
        };
        if !idle {
            writeln!(lines, "{}: {value}", counter.label).expect("a string takes it");
        }
    }
    write!(lines, "time: {elapsed:.2?}").expect("a string takes it");
    lines
}

/// Runs `check`: reads a proof, checks it in the mode the flags give
/// under the time limit and the bounds, and prints the verdict and the
/// derivation, or the verdict as JSON.
///
/// # Errors
///
/// A file that is no proof in JSON, a proof of a goal, or an output that
/// cannot be made; an invalid proof is a verdict, and a check given up at
/// a limit is unknown, neither an error.
pub(crate) fn check(args: &CheckArgs, given: &Given) -> Result<Status> {
    let start = Instant::now();
    let settings = args.settings(given)?;
    let deadline = Deadline::start(time_limit(&settings), start)?;
    let text = io::read(args.proof.as_deref(), "proof", settings.limits.memory_bytes)?;
    let proof = read_proof(&text, &settings.limits)?;
    drop(text);
    if proof.goal().is_some() {
        return Err(Error::GoalProof.into());
    }
    let mode = args.mode.mode();
    let quiet = args.output.quiet;
    let show = Show::new(&args.output, &settings)?;
    let path = args.output.output.clone();
    exit_on_interrupt();
    on_large_stack(settings.limits.stack_bytes(), move || {
        let mut out = io::Output::open(path.as_deref(), show.format.is_binary())?;
        let (status, keep) = check_into(&proof, mode, &show, quiet, &deadline, &mut out)?;
        if keep {
            out.finish()?;
        }
        anyhow::Ok(status)
    })?
}

/// Reads a proof file, its sequent within `limits.occurrences`; the
/// outcome of a sequent that was not proved is refused with what it is.
///
/// # Errors
///
/// A document that is no proof, or past the limit.
fn read_proof(text: &str, limits: &Limits) -> Result<Proof> {
    let mut document = serde_json::Deserializer::from_str(text);
    match linlog::wire::upgrade::<Proof, _>(&mut document, limits) {
        Ok(proof) => {
            document
                .end()
                .context("not a proof in JSON: more follows it")?;
            Ok(proof)
        }
        Err(error) if error.kind() == ErrorKind::Limit => {
            bail!("{error}; raise it with --occurrence-limit")
        }
        Err(error) => {
            // An outcome without a proof is a file the user may well hand
            // over; say what it is rather than which key it lacks.
            /// The one key of an outcome looked for; serde skips the
            /// rest without holding it.
            #[derive(serde::Deserialize)]
            struct Outcome {
                /// The outcome's verdict.
                verdict: Option<String>,
            }
            let verdict = serde_json::from_str::<Outcome>(text)
                .ok()
                .and_then(|outcome| outcome.verdict);
            if let Some(verdict) = verdict.filter(|v| v != "proved") {
                bail!(
                    "the file is the outcome of a sequent the search found {verdict}: it holds no proof to check"
                );
            }
            Err(error.into())
        }
    }
}

/// Checks the proof, writes the verdict and the derivation into `out`,
/// and returns the verdict's status and whether the output is to be
/// finished ([`Show::close`]); the time limit and a closed standard
/// output stop the check and the derivation after it (Ctrl-C ends the
/// command: `check` installs `exit_on_interrupt`).
fn check_into(
    proof: &Proof,
    mode: Mode,
    show: &Show,
    quiet: bool,
    deadline: &Deadline,
    out: &mut io::Output,
) -> Result<(Status, bool)> {
    let format = show.format;
    let over = || interrupted() || deadline.passed() || io::closed();
    let result = proof.check_within(mode, &show.limits, |_| over());
    let status = match &result {
        Ok(()) => Status::Yes,
        Err(CheckError::Refused(_)) => Status::Unknown,
        Err(_) => Status::No,
    };
    let report = |e: &linlog::CheckError| {
        e.describe(proof.forest())
            .abbreviated(show.abbreviate)
            .to_string()
    };
    if format == Format::Json {
        /// What `check --format json` writes.
        #[derive(serde::Serialize)]
        struct Checked {
            /// The wire level.
            version: u32,
            /// `valid`, `invalid` or `unknown`.
            verdict: &'static str,
            /// The mode checked in.
            mode: Mode,
            /// Why the proof is not valid, as the library writes its
            /// errors.
            #[serde(skip_serializing_if = "Option::is_none")]
            error: Option<Error>,
            /// The error's message with formulas for its ids.
            #[serde(skip_serializing_if = "Option::is_none")]
            message: Option<String>,
        }
        let (error, message) = match &result {
            Ok(()) => (None, None),
            Err(e) => (Some(Error::from(e.clone())), Some(report(e))),
        };
        let checked = Checked {
            version: linlog::wire::LEVEL,
            verdict: match status {
                Status::Yes => "valid",
                Status::Unknown => "unknown",
                _ => "invalid",
            },
            mode,
            error,
            message,
        };
        serde_json::to_writer(out.stream(), &checked)?;
        return Ok((status, true));
    }
    // A sequent with no intuitionistic reading is an invalid proof in
    // intuitionistic mode, printed one-sided.
    let formulas = proof.sequent().roots().len();
    let sequent = match Forest::new(proof.sequent())
        .ok()
        .filter(|_| mode.is_intuitionistic())
        .and_then(|forest| {
            Reading::new(&forest)
                .ok()
                .map(|reading| abbreviated(&reading, show.abbreviate, formulas))
        }) {
        Some(two_sided) => two_sided,
        None => abbreviated(proof.sequent(), show.abbreviate, formulas),
    };
    let line = match &result {
        Ok(()) => format!("valid proof of {sequent} ({mode})"),
        Err(e @ CheckError::Refused(refused)) => {
            let advice = match refused.refusal.setting() {
                Some("limits.memory_bytes") => "; raise it with --memory-limit SIZE".to_owned(),
                Some("limits.work") => "; raise it with --work-limit N".to_owned(),
                _ => match stopped(deadline) {
                    Some(Stop::Timeout(t)) => format!(
                        ": the time limit of {t:?} was reached; --timeout DURATION gives the \
                         check longer"
                    ),
                    _ => String::new(),
                },
            };
            format!("unknown: the proof of {sequent} ({mode}) is not checked: {e}{advice}")
        }
        Err(e) => format!("invalid proof of {sequent} ({mode}): {}", report(e)),
    };
    let line = (show.verdict || result.is_err()).then_some(line);
    let prefix = show.open(out, line.as_deref())?;
    if result.is_err() || quiet {
        let keep = show.close(out, Shown::Nothing, line.as_deref())?;
        return Ok((status, keep));
    }
    let why = || Stop::why(stopped(deadline));
    if show.net {
        nets_exist(proof.sequent(), mode)?;
        let shown = net_into(None, proof, mode, show, &over, &why, prefix.as_deref(), out)?;
        return Ok((status, show.close(out, shown, line.as_deref())?));
    }
    let mut steps = 0u32;
    let halt = || {
        steps = steps.wrapping_add(1);
        steps.is_multiple_of(STEPS_PER_CLOCK) && over()
    };
    let shown = derivation(proof, mode, show, halt, why, prefix.as_deref(), out)?;
    Ok((status, show.close(out, shown, line.as_deref())?))
}

#[cfg(test)]
mod tests {
    use super::{Format, Show, Shown, derivation, note};
    use crate::argument_parsing::{Cli, Command};
    use crate::settings::Given;
    use clap::Parser;
    use linlog::{Mode, Options, Sequent};

    /// A derivation the stop ends while it is built is left out with the
    /// stop's reason, never as a check given up without a verdict.
    #[test]
    fn a_stopped_derivation_says_why() {
        let line = ["linlog", "check", "--format", "latex", "--output", "x.tex"];
        let Command::Check(check) = Cli::try_parse_from(line).unwrap().command else {
            unreachable!("a check command")
        };
        let settings = check.settings(&Given::default()).unwrap();
        let show = Show::new(&check.output, &settings).unwrap();
        // Past the nodes the checker passes between two polls of its stop.
        let atoms: Vec<String> = (0..1000).map(|i| format!("A{i}")).collect();
        let duals: Vec<String> = atoms.iter().map(|a| format!("~{a}")).collect();
        let text = format!("|- {}, {}", atoms.join(" * "), duals.join(", "));
        let sequent: Sequent = text.parse().unwrap();
        let outcome = linlog::prove(&sequent, Mode::CLASSICAL, &Options::default()).unwrap();
        let linlog::search::Verdict::Proved(proof) = outcome.verdict else {
            panic!("the sequent is provable")
        };
        let mut out = String::new();
        let shown = derivation(
            &proof,
            Mode::CLASSICAL,
            &show,
            || true,
            || "the time limit of 1s was reached".to_owned(),
            None,
            &mut out,
        )
        .unwrap();
        let Shown::LeftOut(line) = shown else {
            panic!("the derivation is left out")
        };
        assert_eq!(
            line,
            "the derivation is not written: the time limit of 1s was reached"
        );
    }

    /// A Rocq comment stays one comment whatever its text holds: no `(*`
    /// opens a nested one, no `*)` closes it, every `"` is paired.
    #[test]
    fn rocq_notes_stay_comments() {
        assert_eq!(
            note(Format::Rocq, r#"a *) Lemma x : True. (* b (*) " c"#),
            r#"(* a * ) Lemma x : True. ( * b ( * ) "" c *)"#
        );
    }
}
