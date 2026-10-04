// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use crate::argument_parsing::{InputFormat, SequentInput};
use anyhow::{Context, Result, bail};
use linlog::{Forest, Sequent};
use std::fs;
use std::io::{self, IsTerminal, Read, Write};
use std::path::{Path, PathBuf};

/// Reads all of a file, or of standard input for `None` or `-`. Refuses to
/// wait on a terminal, where the user most likely forgot the input.
pub fn read(path: Option<&Path>, what: &str) -> Result<String> {
    match path {
        Some(path) if path != Path::new("-") => {
            fs::read_to_string(path).with_context(|| format!("cannot read {}", path.display()))
        }
        _ => {
            let mut stdin = io::stdin();
            if stdin.is_terminal() {
                bail!("no {what} given: pass it as an argument, with --file, or on standard input");
            }
            let mut text = String::new();
            stdin
                .read_to_string(&mut text)
                .context("cannot read standard input")?;
            Ok(text)
        }
    }
}

/// Returns the sequent `text` holds in `format`, one of the formats of a
/// single sequent: text (a parse error points into `text`), JSON or an
/// LLTP problem.
pub fn sequent_in(text: &str, format: InputFormat) -> Result<Sequent> {
    match format {
        InputFormat::Json => serde_json::from_str(text).context("not a sequent in JSON"),
        InputFormat::Lltp => Ok(linlog::lltp::read(text)
            .context("not an LLTP problem")?
            .sequent),
        _ => text.parse().map_err(|e| crate::parse_error(text, e)),
    }
}

/// Refuses a sequent that unfolds to more than `most` occurrences, before
/// any command unfolds or prints it.
pub fn admit(sequent: Sequent, most: u64) -> Result<Sequent> {
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
    /// Reads the one sequent from the argument, the file or standard
    /// input, in the format the flag or the file's extension names.
    pub fn sequent(&self) -> Result<Sequent> {
        if self.file.len() > 1 {
            bail!("this command reads one sequent: give --file once");
        }
        let file = self.file.first().map(PathBuf::as_path);
        let format = match file {
            Some(path) if path != Path::new("-") => self.input_format.of(path),
            _ => self.input_format,
        };
        if format.is_many() {
            let name = clap::ValueEnum::to_possible_value(&format).expect("no value is skipped");
            bail!(
                "--input-format {} holds many sequents, which `prove` reads as a batch",
                name.get_name()
            );
        }
        let text = match &self.sequent {
            Some(text) => text.clone(),
            None => read(file, "sequent")?,
        };
        admit(sequent_in(&text, format)?, self.most())
    }

    /// Returns the most occurrences the sequent may have.
    pub fn most(&self) -> u64 {
        self.occurrence_limit.0.unwrap_or(u64::MAX)
    }

    /// Reads the sequent and lays it out as a forest, within the limit on
    /// its occurrences.
    pub fn forest(&self) -> Result<Forest> {
        Ok(Forest::from_owned(self.sequent()?, self.most())?)
    }
}

/// Writes `text` and a newline to a file under another name beside
/// `path` and gives it that name once it is whole, so that `path` never
/// holds half an output; what is no regular file (a device, a pipe) is
/// written to as it is.
fn write_file(path: &Path, text: &str) -> io::Result<()> {
    let whole = |file: &Path| {
        let mut out = fs::File::create(file)?;
        out.write_all(text.as_bytes())?;
        out.write_all(b"\n")
    };
    if fs::metadata(path).is_ok_and(|m| !m.is_file()) {
        return whole(path);
    }
    let mut partial = path.as_os_str().to_owned();
    partial.push(format!(".{}.partial", std::process::id()));
    let partial = Path::new(&partial);
    whole(partial)
        .and_then(|()| fs::rename(partial, path))
        .inspect_err(|_| {
            let _ = fs::remove_file(partial);
        })
}

/// Writes `text` and a newline to the file, which then holds all of it or
/// is as it was, or to standard output for `None`.
pub fn write(path: Option<&Path>, text: &str) -> Result<()> {
    match path {
        Some(path) => {
            write_file(path, text).with_context(|| format!("cannot write {}", path.display()))
        }
        None => {
            let mut stdout = io::stdout().lock();
            writeln!(stdout, "{text}").context("cannot write to standard output")
        }
    }
}

/// Where a command's output goes while it is written: standard output, or
/// a file of another name that takes the output's name once it is whole
/// (`finish`), so that the file never holds half an output. It takes text
/// as a [`std::fmt::Write`] and keeps the first error of the stream for
/// `finish`. An output dropped unfinished, after an error, writes nothing
/// more: what its buffer holds is dropped with it, so that an error found
/// before the derivation is written leaves standard output as empty as a
/// file.
pub struct Output {
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

impl Output {
    /// Opens the output: the file at `path`, or standard output for
    /// `None`; a binary one ends without a newline.
    pub fn open(path: Option<&Path>, binary: bool) -> Result<Self> {
        let (sink, rename): (Box<dyn Write>, _) = match path {
            None => (Box::new(io::stdout().lock()), None),
            Some(path) if fs::metadata(path).is_ok_and(|m| !m.is_file()) => {
                let file = fs::File::create(path)
                    .with_context(|| format!("cannot write {}", path.display()))?;
                (Box::new(file), None)
            }
            Some(path) => {
                let mut partial = path.as_os_str().to_owned();
                partial.push(format!(".{}.partial", std::process::id()));
                let partial = PathBuf::from(partial);
                let file = fs::File::create(&partial)
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
    pub fn stream(&mut self) -> &mut impl Write {
        self.sink.as_mut().expect("written before it is finished")
    }

    /// Ends the output with a newline and, for a file, gives it its name;
    /// fails with the stream's first error, and then leaves a file as it
    /// was.
    pub fn finish(mut self) -> Result<()> {
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
        if done.is_err()
            && let Some((_, partial)) = &self.rename
        {
            let _ = fs::remove_file(partial);
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
