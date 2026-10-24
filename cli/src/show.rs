// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use crate::argument_parsing::{Format, OutputArgs, SequentFormat, Tree};
use crate::io;
use crate::limit::detached;
use crate::style;
use anyhow::{Context, Result, anyhow, bail};
use linlog::export::Styles;
use linlog::export::svg::MathFont;
use linlog::export::{Drawable, Form, latex, pdf, png, rocq, svg, typst};
use linlog::proofs::{Compact, Sides, TextOptions};
use linlog::{
    CheckError, Criterion, Derivation, Error, ErrorKind, Forest, Limits, MellStructure, Mode,
    Proof, ProofStructure, Reading, Refusal, Sequent, Size, ViewOptions,
};
use std::borrow::Cow;
use std::fmt::{Display, Write};
use std::io::{IsTerminal, Write as _};

/// How many inferences are built, or pieces of text written, between two
/// looks at the time limit and the Ctrl-C flag while a derivation is made.
pub(crate) const STEPS_PER_CLOCK: u32 = 256;

/// Returns the stop of a derivation's builder and writers: `over` asked
/// once in [`STEPS_PER_CLOCK`] calls.
pub(crate) fn every_steps(mut over: impl FnMut() -> bool) -> impl FnMut() -> bool {
    let mut steps = 0u32;
    move || {
        steps = steps.wrapping_add(1);
        steps.is_multiple_of(STEPS_PER_CLOCK) && over()
    }
}

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
    pub(crate) abbreviate: Option<usize>,
}

impl Show {
    /// Reads the output arguments over the settings' styles, view and
    /// limits.
    pub(crate) fn new(output: &OutputArgs, settings: &linlog::Settings) -> Result<Self> {
        let format = output.format();
        if output.net
            && !matches!(
                format,
                Format::Text | Format::Json | Format::Svg | Format::Png | Format::Pdf
            )
        {
            bail!("--net writes the formats text, json, svg, png and pdf");
        }
        let stdout = std::io::stdout();
        for_terminal(format, output.output.as_deref())?;
        form(output.standalone, format.has_document())?;
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
    pub(crate) fn open(&self, out: &mut impl Write, line: Option<&str>) -> Result<Option<String>> {
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
    pub(crate) fn after(&self, out: &mut impl Write, line: &str, drawn: bool) -> std::fmt::Result {
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
    pub(crate) fn close(
        &self,
        out: &mut io::Output,
        shown: Shown,
        line: Option<&str>,
    ) -> Result<bool> {
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

/// Refuses a binary format bound for standard output when that is a
/// terminal.
///
/// # Errors
///
/// A PNG or PDF without `output` while standard output is a terminal.
pub(crate) fn for_terminal(format: Format, output: Option<&std::path::Path>) -> Result<()> {
    if format.is_binary() && output.is_none() && std::io::stdout().is_terminal() {
        bail!(
            "a {} is not for a terminal: write it with --output FILE",
            format.title()
        );
    }
    Ok(())
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

/// Writes the certificate of a disproof into `out` after `prefix`, for
/// `--format rocq`: that linlog's library does not derive the sequent,
/// or for an ordinary sequent (`image`) that it is not valid over `Prop`;
/// or says why there is none, a refutation that rests on the search
/// among them, which leaves the verdict as it is.
pub(crate) fn disproof_into(
    disproof: &linlog::Disproof,
    image: Option<&linlog::ordinary::Image>,
    show: &Show,
    prefix: Option<&str>,
    out: &mut impl Write,
) -> Shown {
    let mut out = Prefixed { out, prefix };
    let written = match image {
        Some(image) => rocq::write_not_valid(image, disproof, &show.styles.rocq, &mut out),
        None => rocq::write_disproof(disproof, &show.styles.rocq, &mut out),
    };
    match written {
        // The output's own error, which finishing it reports.
        Ok(()) | Err(Error::WriteFailed) => Shown::Written,
        Err(error) => Shown::LeftOut(format!("no certificate: {error}")),
    }
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
            e.describe(proof.forest())
                .in_mode(mode)
                .abbreviated(show.abbreviate)
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
        let reading = Reading::of_mode(&forest, mode)
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

/// The math fonts the PNG and PDF drawings are set in, each the file of a
/// [`MathFont`] beside its licence in `fonts/` (Euler Math's `OFL.txt`,
/// STIX Two Math's, Garamond-Math's and Asana Math's `<file>-OFL.txt`, New
/// Computer Modern Math's `NewCMMath-Book-LICENSE.txt`).
const FONTS: [(MathFont, &[u8]); 5] = [
    (MathFont::Euler, include_bytes!("../fonts/Euler-Math.otf")),
    (
        MathFont::StixTwo,
        include_bytes!("../fonts/STIXTwoMath-Regular.otf"),
    ),
    (
        MathFont::Garamond,
        include_bytes!("../fonts/Garamond-Math.otf"),
    ),
    (MathFont::Asana, include_bytes!("../fonts/Asana-Math.otf")),
    (
        MathFont::NewComputerModern,
        include_bytes!("../fonts/NewCMMath-Book.otf"),
    ),
];

/// Returns the data of the fonts a render is given: Euler Math, and every
/// other math font whose family the style names. A font the renderer is
/// not given cannot set a glyph, so a style that names one family loads
/// that font and no other.
fn fonts(styles: &Styles) -> Vec<&'static [u8]> {
    let family = &styles.svg.font.family;
    FONTS
        .iter()
        .filter(|(font, _)| *font == MathFont::Euler || family.contains(font.family()))
        .map(|&(_, data)| data)
        .collect()
}

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
/// Math font and the math font the style names ([`fonts`]), on a thread of its own while `stop` is asked every few
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
            let fonts = fonts(styles);
            detached(
                "render",
                move || png::from_svg(&svg, &fonts, &options, &limits),
                stop,
            )?
        }
        Format::Pdf => {
            let mut options = styles.pdf.clone();
            let fonts = fonts(styles);
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
                move || pdf::from_svg(&svg, &fonts, &options, &limits),
                stop,
            )?
        }
        Format::Text
        | Format::Json
        | Format::Latex
        | Format::Typst
        | Format::Svg
        | Format::Rocq => {
            unreachable!("only a binary format is rendered")
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

/// A proof's net: a proof structure of MLL, or one with boxes of MELL.
#[derive(serde::Serialize)]
#[serde(untagged)]
pub(crate) enum Net<'a> {
    /// The net of an MLL proof, or the one the net engine found.
    Mll(Cow<'a, ProofStructure>),
    /// The net with boxes of a proof with exponentials.
    Mell(MellStructure),
}

impl Display for Net<'_> {
    /// Writes the net's text form.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Mll(net) => net.fmt(f),
            Self::Mell(net) => net.fmt(f),
        }
    }
}

impl Net<'_> {
    /// Returns the net's text form as `mode` reads it: in an
    /// intuitionistic mode with the formulas two-sided.
    fn written_in(&self, mode: Mode) -> String {
        match self {
            Self::Mll(net) => net.written_in(mode).to_string(),
            Self::Mell(net) => net.written_in(mode).to_string(),
        }
    }

    /// Returns the net drawn as an SVG document, or the line that says why
    /// it is not drawn: it is estimated past the limit. In an
    /// intuitionistic mode it is drawn as its essential net, whose
    /// formulas are the intuitionistic ones.
    fn drawing(
        &self,
        show: &Show,
        mode: Mode,
        stop: &dyn Fn() -> bool,
    ) -> std::result::Result<String, String> {
        let style = show
            .styles
            .svg
            .clone()
            .with_essential(show.styles.svg.essential || mode.is_intuitionistic());
        let (drawn, size) = match self {
            Self::Mll(net) => (
                svg::net(net, &style, &show.limits, |_| stop()),
                (net.forest().len(), "occurrences", net.links().len()),
            ),
            Self::Mell(net) => (
                svg::mell_net(net, &style, &show.limits, |_| stop()),
                (net.len(), "vertices", net.links().len()),
            ),
        };
        drawn.map_err(|error| net_too_large(size, &error))
    }
}

/// Returns the line for a proof net whose drawing is past the limit, the
/// net of `size` (its parts, what they are, and its links).
fn net_too_large(size: (usize, &str, usize), error: &Error) -> String {
    let Error::Refused(Refusal::Output {
        estimate_bytes,
        limit_bytes,
        ..
    }) = *error
    else {
        return format!("the proof net is not drawn: {error}");
    };
    let (parts, what, links) = size;
    format!(
        "the proof net is not drawn: its {parts} {what} and {links} links are estimated at {}, \
         over the limit of {}; the text format writes the net itself, --derivation-limit SIZE \
         raises the limit and --derivation-limit none lifts it",
        bytes_text(estimate_bytes),
        bytes_text(limit_bytes)
    )
}

/// Returns the proof net of a proof: `found`, the net the net engine
/// found, if it ran; else for a sequent with exponentials its net with
/// boxes, otherwise its MLL net; or the line that says why it is not
/// made: a bound refused it, or `stop` fired, for the reason `why` gives.
///
/// # Errors
///
/// What the library answers that is no refusal.
pub(crate) fn net_of<'a>(
    found: Option<&'a ProofStructure>,
    proof: &Proof,
    mode: Mode,
    limits: &Limits,
    stop: &dyn Fn() -> bool,
    why: &dyn Fn() -> String,
) -> Result<std::result::Result<Net<'a>, String>> {
    if let Some(net) = found {
        return Ok(Ok(Net::Mll(Cow::Borrowed(net))));
    }
    let criterion = Criterion::of(mode)?;
    let made = if proof.sequent().fragment().has_exponentials() {
        MellStructure::from_proof(proof, criterion, limits, |_| stop()).map(Net::Mell)
    } else {
        ProofStructure::from_proof(proof, criterion, limits, |_| stop())
            .map(|net| Net::Mll(Cow::Owned(net)))
    };
    match made {
        Ok(net) => Ok(Ok(net)),
        Err(error) if error.kind() == ErrorKind::Stopped => {
            Ok(Err(format!("the proof net is not drawn: {}", why())))
        }
        Err(error) if error.is_refusal() => Ok(Err(format!("the proof net is not made: {error}"))),
        Err(error) => Err(error.into()),
    }
}

/// Writes the proof net of a proof into `out` as `show` asks: as text, or
/// drawn as an SVG document, a PNG image or a PDF document; or says why
/// it is left out: it is not made ([`net_of`]), its drawing is estimated
/// past the derivation limit, a bound of the render refused it, or `stop`
/// fired while it was rendered, for the reason `why` gives. `found` is
/// the net the net engine found, if it ran.
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
    let net = match net_of(found, proof, mode, &show.limits, stop, why)? {
        Ok(net) => net,
        Err(line) => return Ok(Shown::LeftOut(line)),
    };
    let separator = prefix.unwrap_or("");
    if !matches!(show.format, Format::Svg | Format::Png | Format::Pdf) {
        write!(out, "{separator}{}", net.written_in(mode))?;
        return Ok(Shown::Written);
    }
    let drawing = match net.drawing(show, mode, stop) {
        Ok(drawing) => drawing,
        Err(line) => return Ok(Shown::LeftOut(line)),
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

/// Writes the proof net of a proof as its JSON document into `out`, and
/// returns whether it did; the line that says why it is not made goes to
/// standard error, and the caller writes the outcome instead, since JSON
/// is always written.
///
/// # Errors
///
/// What [`net_of`] fails with, and an output that cannot be written.
pub(crate) fn net_json(
    found: Option<&ProofStructure>,
    proof: &Proof,
    mode: Mode,
    show: &Show,
    stop: &dyn Fn() -> bool,
    why: &dyn Fn() -> String,
    out: &mut io::Output,
) -> Result<bool> {
    match net_of(found, proof, mode, &show.limits, stop, why)? {
        Ok(net) => {
            serde_json::to_writer(out.stream(), &net)?;
            Ok(true)
        }
        Err(line) => {
            eprintln!("{line}");
            Ok(false)
        }
    }
}

/// Fails unless proof nets exist for the sequent in the mode: unit-free
/// MLL, linear, with or without Mix, classical or intuitionistic (where
/// the net is the one of the one-sided sequent); with exponentials, nets
/// with boxes of unit-free MELL, classical, with or without Mix.
pub(crate) fn nets_exist(sequent: &Sequent, mode: Mode) -> Result<()> {
    if mode.is_affine() {
        bail!("proof nets exist in linear mode only, with or without --mix, not in {mode} mode");
    }
    let fragment = sequent.fragment();
    if fragment.has_exponentials() {
        return Ok(MellStructure::exist(fragment, mode)?);
    }
    Ok(linlog::nets::exist(fragment, mode)?)
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
