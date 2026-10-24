// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use crate::argument_parsing::{InputFormat, SequentInput};
use anyhow::{Context, Result, bail};
use linlog::ordinary::Image;
use linlog::wire::Within;
use linlog::{
    ClassicalSequent, Forest, IllSequent, IllSyntax, InputOptions, Limits, Mode, Order, Sequent,
    Written,
};
use serde::de::DeserializeSeed;
use std::fs;
use std::io::{self, BufRead, IsTerminal, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

/// The files under another name that this process writes outputs to
/// until they are whole, so that an interrupt can remove them.
static PARTIAL: Mutex<Vec<PathBuf>> = Mutex::new(Vec::new());

/// Returns the name an output to `path` is written under until it is
/// whole, which an interrupt removes until [`settled`] is called.
fn partial(path: &Path) -> PathBuf {
    let mut partial = path.as_os_str().to_owned();
    partial.push(format!(".{}.partial", std::process::id()));
    let partial = PathBuf::from(partial);
    let mut files = PARTIAL.lock().unwrap_or_else(PoisonError::into_inner);
    files.push(partial.clone());
    partial
}

/// Forgets a file of [`partial`] that was renamed or removed.
fn settled(partial: &Path) {
    let mut files = PARTIAL.lock().unwrap_or_else(PoisonError::into_inner);
    files.retain(|file| file != partial);
}

/// Removes every file an output of this process is being written to:
/// what an interrupt that ends the process does first.
pub(crate) fn remove_partial_files() {
    let files = PARTIAL.lock().unwrap_or_else(PoisonError::into_inner);
    for file in files.iter() {
        let _ = fs::remove_file(file);
    }
}

/// The most bytes a settings or style file may have.
const CONFIGURATION_BYTES: u64 = 16 << 20;

/// Reads all of a file, or of standard input for `None` or `-`, refusing
/// one longer than `memory` bytes (the memory limit, which nothing read
/// from it fits within). Refuses to wait on a terminal, where the user
/// most likely forgot the input.
///
/// # Errors
///
/// A file that cannot be read or is longer than `memory`, and standard
/// input when it is a terminal.
pub(crate) fn read(path: Option<&Path>, what: &str, memory: Option<u64>) -> Result<String> {
    let advice = "; raise it with --memory-limit SIZE";
    match path {
        Some(path) if path != Path::new("-") => {
            let file =
                fs::File::open(path).with_context(|| format!("cannot read {}", path.display()))?;
            capped(
                file,
                memory,
                || "it is longer than the memory limit of".to_owned(),
                advice,
            )
            .with_context(|| format!("cannot read {}", path.display()))
        }
        _ => {
            let stdin = io::stdin();
            if stdin.is_terminal() {
                let flag = if what == "sequent" {
                    ", with --file,"
                } else {
                    ""
                };
                bail!("no {what} given: pass it as an argument{flag} or on standard input");
            }
            capped(
                stdin,
                memory,
                || format!("the {what} on standard input is longer than the memory limit of"),
                advice,
            )
            .context("cannot read standard input")
        }
    }
}

/// Reads a settings or style file, of at most [`CONFIGURATION_BYTES`];
/// never standard input, which carries the sequent or a session's
/// commands.
///
/// # Errors
///
/// `-`, and a file that cannot be read or is too long.
pub(crate) fn read_configuration(path: &Path, what: &str) -> Result<String> {
    if path == Path::new("-") {
        bail!("the {what} is read from a file: standard input carries the input");
    }
    let file = fs::File::open(path).with_context(|| format!("cannot read {}", path.display()))?;
    capped(
        file,
        Some(CONFIGURATION_BYTES),
        || "it is longer than".to_owned(),
        "",
    )
    .with_context(|| format!("cannot read {}", path.display()))
}

/// Reads `input` whole as text, refusing it past `most` bytes with
/// `over`'s words, the bound, and `advice`. The length is judged before
/// the text is decoded, so that a cut inside a character past the bound
/// is not taken for text that is no UTF-8.
fn capped(
    mut input: impl Read,
    most: Option<u64>,
    over: impl FnOnce() -> String,
    advice: &str,
) -> Result<String> {
    let mut bytes = Vec::new();
    match most {
        Some(most) => {
            input.take(most.saturating_add(1)).read_to_end(&mut bytes)?;
            if bytes.len() as u64 > most {
                bail!("{} {}{advice}", over(), crate::show::bytes_text(most));
            }
        }
        None => {
            input.read_to_end(&mut bytes)?;
        }
    }
    String::from_utf8(bytes).map_err(|_| anyhow::anyhow!("it is not UTF-8 text"))
}

/// Reads `input` up to and with the next `separator` into `bytes`,
/// holding at most `most` of them: the rest of a longer line is skipped
/// without being held, and the line is answered as too long. Returns the
/// bytes read, 0 at the end of the input, and whether the line fit.
///
/// # Errors
///
/// An input that cannot be read.
pub(crate) fn until_within<R: BufRead + ?Sized>(
    input: &mut R,
    separator: u8,
    most: Option<u64>,
    bytes: &mut Vec<u8>,
) -> io::Result<(usize, bool)> {
    let Some(most) = most else {
        return Ok((input.read_until(separator, bytes)?, true));
    };
    let read = (&mut *input)
        .take(most.saturating_add(1))
        .read_until(separator, bytes)?;
    if bytes.len() as u64 <= most || bytes.last() == Some(&separator) {
        return Ok((read, true));
    }
    bytes.clear();
    loop {
        let buffer = input.fill_buf()?;
        if buffer.is_empty() {
            break;
        }
        if let Some(end) = buffer.iter().position(|&b| b == separator) {
            input.consume(end + 1);
            break;
        }
        let length = buffer.len();
        input.consume(length);
    }
    Ok((read, false))
}

/// The most bytes of one command line of a session.
pub(crate) const LINE_BYTES: u64 = 1 << 20;

/// Reads the next line of `input` without its line break, refusing one of
/// more than [`LINE_BYTES`]; `None` at the end of the input.
///
/// # Errors
///
/// An input that cannot be read, a line that is too long or no UTF-8.
pub(crate) fn line(input: &mut impl BufRead) -> Result<Option<String>> {
    let mut bytes = Vec::new();
    input
        .take(LINE_BYTES.saturating_add(1))
        .read_until(b'\n', &mut bytes)?;
    if bytes.is_empty() {
        return Ok(None);
    }
    if bytes.last() == Some(&b'\n') {
        bytes.pop();
        if bytes.last() == Some(&b'\r') {
            bytes.pop();
        }
    } else if bytes.len() as u64 > LINE_BYTES {
        bail!(
            "a line is longer than {}",
            crate::show::bytes_text(LINE_BYTES)
        );
    }
    String::from_utf8(bytes)
        .map(Some)
        .map_err(|_| anyhow::anyhow!("a line is not UTF-8 text"))
}

/// Returns `mode`, made affine for a sequent in `format` if that is a
/// coverability problem's, whose question is affine mode's.
pub(crate) fn affine_for(format: InputFormat, mode: Mode) -> Mode {
    if format == InputFormat::Spec {
        mode.with_affine()
    } else {
        mode
    }
}

/// Returns the sequent `text` holds in `format`, one of the formats of a
/// single sequent: text (a parse error points into `text`), JSON, an LLTP
/// problem or a `.spec` problem. Text, LLTP and `.spec` are refused at the
/// first occurrence past `most`, before anything of that size is built; a
/// JSON sequent is its own arena, as small as its file, and `admit` says
/// how many occurrences it unfolds to.
///
/// # Errors
///
/// Text that is no sequent in `format`, with a caret under the place for
/// the text syntax, and a sequent past `most`.
pub(crate) fn sequent_in(
    text: &str,
    format: InputFormat,
    most: u64,
    mode: Mode,
    read: &InputOptions,
) -> Result<Sequent> {
    let order = mode.order();
    // The text of an intuitionistic sequent of the commutative order is
    // read in the syntax the options name, JSON as either form.
    let intuitionistic = mode.is_intuitionistic() && order == Order::Commutative;
    let unbounded = Limits::default().with_occurrences(None);
    let within = bound(most);
    let refused = |error| refused(error, most);
    if order != Order::Commutative && matches!(format, InputFormat::Lltp | InputFormat::Spec) {
        bail!("the cyclic mode reads a sequent as text or JSON, not as an LLTP or .spec problem");
    }
    match format {
        InputFormat::Json if intuitionistic && is_ill_json(text) => {
            let mut document = serde_json::Deserializer::from_str(text);
            // Read within the bound: the reading builds the forest.
            let sequent = Within::<IllSequent>::new(&within)
                .deserialize(&mut document)
                .and_then(|sequent| document.end().map(|()| sequent));
            Ok(sequent
                .context("not an intuitionistic sequent in JSON")?
                .into_sequent())
        }
        InputFormat::Json => {
            let mut document = serde_json::Deserializer::from_str(text);
            let sequent = Within::<Sequent>::new(&unbounded)
                .deserialize(&mut document)
                .and_then(|sequent| document.end().map(|()| sequent));
            sequent.context("not a sequent in JSON")
        }
        // The library's error says that the text is no LLTP problem.
        InputFormat::Lltp => Ok(linlog::lltp::read(text, &within).map_err(refused)?.sequent),
        InputFormat::Spec => match linlog::mist::read(text, &within) {
            Ok(problem) => Ok(problem.sequent),
            Err(
                error @ linlog::Error::Refused(linlog::limits::Refusal::Occurrences {
                    occurrences,
                    ..
                }),
            ) => Err(crate::Explained::error(
                format!(
                    "the problem's tokens alone are {occurrences} subformula occurrences, more \
                     than the limit of {most}; raise it with --occurrence-limit"
                ),
                error,
            )),
            Err(e) => Err(e.into()),
        },
        _ if intuitionistic && read.intuitionistic == IllSyntax::Strict => {
            IllSequent::parse_within(text, &within)
                .map(IllSequent::into_sequent)
                .map_err(|e| match e {
                    linlog::Error::Parse(_) => crate::parse_error(text, e),
                    e => refused(e),
                })
        }
        _ => Sequent::parse_ordered(text, order, &within).map_err(|e| match e {
            linlog::Error::Parse(_) => crate::parse_error(text, e),
            e => refused(e),
        }),
    }
}

/// A sequent as it was read, with how it was written where the reading
/// kept that.
pub(crate) type AsRead = (Sequent, Option<Box<[Written]>>);

/// Reads a sequent as [`sequent_in`] does, and where `keep` is set and
/// the sequent is text of a commutative classical mode, keeps how it was
/// written, for a derivation drawn two-sided.
///
/// # Errors
///
/// As [`sequent_in`] does.
pub(crate) fn written_in(
    text: &str,
    format: InputFormat,
    most: u64,
    mode: Mode,
    read: &InputOptions,
    keep: bool,
) -> Result<AsRead> {
    let text_format = matches!(format, InputFormat::Auto | InputFormat::Text);
    if keep && text_format && !mode.is_intuitionistic() && mode.order() == Order::Commutative {
        let written = ClassicalSequent::parse_within(text, &bound(most)).map_err(|e| match e {
            linlog::Error::Parse(_) => crate::parse_error(text, e),
            e => refused(e, most),
        })?;
        let writing = written.writing().into();
        return Ok((written.into_sequent(), Some(writing)));
    }
    Ok((sequent_in(text, format, most, mode, read)?, None))
}

/// Returns whether a JSON text is an intuitionistic sequent's document,
/// which has `hypotheses` where a sequent's has `roots`.
fn is_ill_json(text: &str) -> bool {
    /// The key that tells the two forms apart.
    #[derive(serde::Deserialize)]
    struct Probe {
        /// An intuitionistic sequent's hypotheses.
        hypotheses: Option<serde::de::IgnoredAny>,
    }
    serde_json::from_str::<Probe>(text).is_ok_and(|probe| probe.hypotheses.is_some())
}

/// Returns the library's error, or for an input refused past `most`
/// occurrences the line that names the flag raising it.
pub(crate) fn refused(error: linlog::Error, most: u64) -> anyhow::Error {
    match error {
        linlog::Error::Refused(linlog::limits::Refusal::Occurrences { .. }) => {
            crate::Explained::error(
                format!(
                    "the sequent has more subformula occurrences than the limit of {most}; \
                     raise it with --occurrence-limit"
                ),
                error,
            )
        }
        error => error.into(),
    }
}

/// Returns the default limits with `most` occurrences at most.
pub(crate) fn bound(most: u64) -> Limits {
    Limits::default().with_occurrences(Some(most))
}

/// Refuses a sequent that unfolds to more than `most` occurrences, before
/// any command unfolds or prints it.
///
/// # Errors
///
/// The library's [`Refusal::Occurrences`](linlog::Refusal::Occurrences)
/// past `most`, naming the flag that raises it.
pub(crate) fn admit(sequent: Sequent, most: u64) -> Result<Sequent> {
    let occurrences = sequent.occurrences();
    if occurrences > most {
        bail!(
            "the sequent unfolds to {} subformula occurrences, more than the limit of {most}; \
             raise it with --occurrence-limit",
            if occurrences == u64::MAX {
                "more than 10¹⁹".to_owned()
            } else {
                occurrences.to_string()
            },
        );
    }
    Ok(sequent)
}

impl SequentInput {
    /// Returns the format of the one input: the flag's, or the one the
    /// file's extension names, a `.p` file being a TPTP problem when
    /// `ordinary` is set.
    pub(crate) fn format(&self, ordinary: bool) -> InputFormat {
        match self.file.first() {
            Some(path) if path != Path::new("-") => self.input_format.of(path, ordinary),
            _ => self.input_format,
        }
    }

    /// Returns the mode a sequent of this input is decided in: the flags'
    /// `mode`, made affine for a coverability problem, whose question is
    /// affine mode's whatever the flags say; no flag takes weakening
    /// away, so none contradicts it.
    pub(crate) fn mode(&self, mode: Mode) -> Mode {
        affine_for(self.format(false), mode)
    }

    /// Reads the one input from the argument, the file or standard input,
    /// and returns it with the format the flag or the file's extension
    /// names, a `.p` file being a TPTP problem when `ordinary` is set.
    fn text(&self, ordinary: bool, memory: Option<u64>) -> Result<(String, InputFormat)> {
        if self.file.len() > 1 {
            bail!("this command reads one sequent: give --file once");
        }
        let file = self.file.first().map(PathBuf::as_path);
        let format = self.format(ordinary);
        if format.is_many() {
            let name = clap::ValueEnum::to_possible_value(&format).expect("no value is skipped");
            bail!(
                "--input-format {} holds many sequents, which `prove` reads as a batch",
                name.get_name()
            );
        }
        let text = match &self.sequent {
            Some(text) => text.clone(),
            None => read(file, "sequent", memory)?,
        };
        Ok((text, format))
    }

    /// Reads the one sequent from the argument, the file or standard
    /// input, in the format the flag or the file's extension names, within
    /// `limits.occurrences` and a text within `limits.memory_bytes`.
    ///
    /// # Errors
    ///
    /// An input that cannot be read, or text that is no sequent in its
    /// format, or one past the limits.
    pub(crate) fn sequent(
        &self,
        limits: &Limits,
        read: &InputOptions,
        mode: Mode,
    ) -> Result<Sequent> {
        Ok(self.written(limits, read, mode, false)?.0)
    }

    /// Reads the one sequent as [`sequent`](Self::sequent) does, and where
    /// `keep` is set keeps how it was written ([`written_in`]).
    ///
    /// # Errors
    ///
    /// As [`sequent`](Self::sequent) does.
    pub(crate) fn written(
        &self,
        limits: &Limits,
        read: &InputOptions,
        mode: Mode,
        keep: bool,
    ) -> Result<AsRead> {
        let most = limits.occurrences.unwrap_or(u64::MAX);
        let (text, format) = self.text(false, limits.memory_bytes)?;
        if format == InputFormat::Tptp {
            bail!("--input-format tptp holds ordinary logic, which --logic reads");
        }
        let (sequent, writing) = written_in(&text, format, most, mode, read, keep)?;
        let sequent = admit(sequent, most)?;
        // A `.p` file is LLTP's unless `--logic` says otherwise, and the
        // ILTP library's problems are `.p` files too: one that reads both
        // ways gets a note, since its verdicts differ.
        if self.input_format == InputFormat::Auto
            && format == InputFormat::Lltp
            && linlog::ordinary::read_tptp(&text, &bound(most)).is_ok()
        {
            eprintln!(
                "note: the file is read as an LLTP problem of linear logic; it is also a TPTP \
                 problem of ordinary logic, which --logic reads"
            );
        }
        Ok((sequent, writing))
    }

    /// Reads one formula or sequent of ordinary logic and returns its image
    /// under the translation `options` choose, with the image's forest,
    /// within the limits as [`sequent`](Self::sequent) reads.
    ///
    /// # Errors
    ///
    /// As [`sequent`](Self::sequent) does for an ordinary sequent, and the
    /// translation's error.
    pub(crate) fn image(
        &self,
        options: &linlog::ordinary::Options,
        limits: &Limits,
    ) -> Result<(Forest, Image)> {
        let most = limits.occurrences.unwrap_or(u64::MAX);
        let (text, format) = self.text(true, limits.memory_bytes)?;
        let ordinary = crate::ordinary::sequent_in(&text, format, most)?;
        let image = crate::ordinary::image(options, &ordinary, most)?;
        let sequent = admit(image.sequent().clone(), most)?;
        Ok((Forest::from_owned(sequent, &bound(most))?, image))
    }

    /// Returns the most occurrences the flag gives the sequent.
    pub(crate) fn most(&self) -> u64 {
        self.occurrence_limit.0.unwrap_or(u64::MAX)
    }

    /// Reads the sequent and lays it out as a forest, within the limits
    /// as [`sequent`](Self::sequent) reads, with how it was written where
    /// `keep` asks it ([`written_in`]).
    ///
    /// # Errors
    ///
    /// As [`sequent`](Self::sequent) does.
    pub(crate) fn forest(
        &self,
        limits: &Limits,
        read: &InputOptions,
        mode: Mode,
        keep: bool,
    ) -> Result<(Forest, Option<Box<[Written]>>)> {
        let most = limits.occurrences.unwrap_or(u64::MAX);
        let (sequent, writing) = self.written(limits, read, mode, keep)?;
        Ok((Forest::from_owned(sequent, &bound(most))?, writing))
    }
}

/// Standard output, which takes a closed pipe as a reader that has read
/// all it wants: what is written after is dropped and nothing fails, so
/// `prove … | head -1` keeps the verdict's exit status. Whether the pipe
/// closed is [`closed`]'s.
pub(crate) struct Stdout(io::StdoutLock<'static>);

/// Set once a write to standard output met a closed pipe.
static CLOSED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Whether standard output's reader has closed the pipe.
pub(crate) fn closed() -> bool {
    CLOSED.load(std::sync::atomic::Ordering::Relaxed)
}

impl Stdout {
    /// Locks standard output.
    pub(crate) fn lock() -> Self {
        Self(io::stdout().lock())
    }

    /// Returns the result, a closed pipe taken as written.
    fn taken<T>(result: io::Result<T>, written: T) -> io::Result<T> {
        match result {
            Err(e) if e.kind() == io::ErrorKind::BrokenPipe => {
                CLOSED.store(true, std::sync::atomic::Ordering::Relaxed);
                Ok(written)
            }
            result => result,
        }
    }
}

impl Write for Stdout {
    /// Writes into standard output, or drops the bytes once its pipe is
    /// closed.
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if closed() {
            return Ok(buf.len());
        }
        Self::taken(self.0.write(buf), buf.len())
    }

    /// Flushes standard output, or nothing once its pipe is closed.
    fn flush(&mut self) -> io::Result<()> {
        if closed() {
            return Ok(());
        }
        Self::taken(self.0.flush(), ())
    }
}

/// Writes `text` and a newline to the file, which then holds all of it or
/// is as it was, or to standard output for `None`.
///
/// # Errors
///
/// The file system's error, the file then left as it was.
pub(crate) fn write(path: Option<&Path>, text: &str) -> Result<()> {
    let mut out = Output::open(path, false)?;
    out.stream().write_all(text.as_bytes())?;
    out.finish()
}

/// Where a command's output goes while it is written: standard output, or
/// a file of another name that takes the output's name once it is whole
/// (`finish`), so that the file never holds half an output. It takes text
/// as a [`std::fmt::Write`] and keeps the first error of the stream for
/// `finish`. An output dropped unfinished, after an error, writes nothing
/// more: what its buffer holds is dropped with it, so that an error found
/// before the derivation is written leaves standard output as empty as a
/// file.
pub(crate) struct Output {
    /// The stream, until it is finished.
    sink: Option<io::BufWriter<Box<dyn Write>>>,
    /// The file the output is for and the one it is written to meanwhile,
    /// for an output into a regular file.
    rename: Option<(PathBuf, PathBuf)>,
    /// The first error of the stream.
    error: Option<io::Error>,
    /// Whether the output is binary, which ends with no newline.
    binary: bool,
}

impl std::fmt::Debug for Output {
    /// Says where the output goes and how it stands; the stream is opaque.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Output")
            .field("rename", &self.rename)
            .field("error", &self.error)
            .field("binary", &self.binary)
            .finish_non_exhaustive()
    }
}

impl Output {
    /// Opens the output: the file at `path`, or standard output for
    /// `None`; a binary one ends without a newline.
    ///
    /// # Errors
    ///
    /// A file that cannot be made beside the path.
    pub(crate) fn open(path: Option<&Path>, binary: bool) -> Result<Self> {
        let (sink, rename): (Box<dyn Write>, _) = match path {
            None => (Box::new(Stdout::lock()), None),
            Some(path) if fs::metadata(path).is_ok_and(|m| !m.is_file()) => {
                let file = fs::File::create(path)
                    .with_context(|| format!("cannot write {}", path.display()))?;
                (Box::new(file), None)
            }
            Some(path) => {
                let partial = partial(path);
                let file = fs::File::create(&partial)
                    .inspect_err(|_| settled(&partial))
                    .with_context(|| format!("cannot write {}", path.display()))?;
                (Box::new(file), Some((path.to_owned(), partial)))
            }
        };
        Ok(Self {
            sink: Some(io::BufWriter::new(sink)),
            rename,
            error: None,
            binary,
        })
    }

    /// The stream itself, for a writer of bytes such as serde_json's; an
    /// error it meets is the writer's to report.
    pub(crate) fn stream(&mut self) -> &mut impl Write {
        self.sink.as_mut().expect("written before it is finished")
    }

    /// Ends the output with a newline and, for a file, gives it its name;
    /// fails with the stream's first error, and then leaves a file as it
    /// was.
    ///
    /// # Errors
    ///
    /// The stream's first error, or the file system's when it renames.
    pub(crate) fn finish(mut self) -> Result<()> {
        let mut sink = self.sink.take().expect("finished once");
        let done = match self.error.take() {
            Some(error) => Err(error),
            None if self.binary => sink.flush(),
            None => sink.write_all(b"\n").and_then(|()| sink.flush()),
        };
        let what = match &self.rename {
            Some((path, _)) => format!("cannot write {}", path.display()),
            None => "cannot write to standard output".to_owned(),
        };
        let done = done.and_then(|()| match &self.rename {
            Some((path, partial)) => fs::rename(partial, path),
            None => Ok(()),
        });
        if let Some((_, partial)) = self.rename.take() {
            if done.is_err() {
                let _ = fs::remove_file(&partial);
            }
            settled(&partial);
        }
        done.context(what)
    }
}

impl Drop for Output {
    /// Drops what an output that was never finished still buffers, and
    /// removes its file.
    fn drop(&mut self) {
        if let Some(sink) = self.sink.take() {
            let _ = sink.into_parts();
        }
        if let Some((_, partial)) = &self.rename {
            let _ = fs::remove_file(partial);
            settled(partial);
        }
    }
}

impl std::fmt::Write for Output {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        if self.error.is_some() {
            return Err(std::fmt::Error);
        }
        self.stream().write_all(s.as_bytes()).map_err(|e| {
            self.error = Some(e);
            std::fmt::Error
        })
    }
}
