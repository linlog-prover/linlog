// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The linlog command line interface: `linlog prove`, `linlog check`,
//! `linlog interact` and `linlog seq`, as a library so that the binary is
//! one call and this documentation exists.

/// The command line arguments.
pub mod argument_parsing;
/// `prove` over many sequents.
pub mod batch;
/// The `interact` command.
pub mod interact;
/// Reading input and writing output.
pub mod io;
/// The time limit of a command.
mod limit;
/// Ordinary logic: its input, its verdict line and the derivation read
/// back.
pub mod ordinary;
/// The `prove` and `check` commands.
pub mod prove;
/// The `seq` commands.
mod seq;
/// The settings of a command, from its defaults, a file and its flags.
pub mod settings;
/// The options of every output format, from a file and flags.
pub mod style;

use anyhow::Result;
use argument_parsing::{Cli, Command};
use clap::{CommandFactory, FromArgMatches};
use linlog::Error;
use std::fmt::Write;
use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, Ordering};

/// What a command found, which decides the exit status.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    /// Provable, a valid proof, or a command that has no verdict: 0.
    Yes,
    /// Unprovable, or an invalid proof: 1.
    No,
    /// The search stopped before deciding: 3.
    Unknown,
    /// An entry of a batch was an error: 2.
    Error,
}

impl Status {
    /// Returns the worse of two verdicts of a batch: an error before
    /// unknown before unprovable before proved.
    #[must_use]
    pub fn worse(self, other: Self) -> Self {
        let rank = |s| match s {
            Status::Yes => 0,
            Status::No => 1,
            Status::Unknown => 2,
            Status::Error => 3,
        };
        if rank(other) > rank(self) {
            other
        } else {
            self
        }
    }
}

impl From<&linlog::search::Verdict> for Status {
    /// Returns the status of a verdict: proved, unprovable or unknown.
    fn from(verdict: &linlog::search::Verdict) -> Self {
        match verdict {
            linlog::search::Verdict::Proved(_) => Self::Yes,
            linlog::search::Verdict::Unprovable(_) => Self::No,
            linlog::search::Verdict::Unknown(_) => Self::Unknown,
        }
    }
}

/// The exit status of an error: bad arguments (as clap reports them),
/// unreadable input, a sequent no engine handles.
const ERROR: u8 = 2;

impl From<Status> for ExitCode {
    /// Returns the exit status for a verdict.
    fn from(s: Status) -> Self {
        ExitCode::from(match s {
            Status::Yes => 0,
            Status::No => 1,
            Status::Unknown => 3,
            Status::Error => ERROR,
        })
    }
}

/// Set by the first Ctrl-C.
static INTERRUPTED: AtomicBool = AtomicBool::new(false);

/// Whether the user pressed Ctrl-C to stop the search.
pub fn interrupted() -> bool {
    INTERRUPTED.load(Ordering::Relaxed)
}

/// Forgets a Ctrl-C that stopped a search, so that the next search runs.
pub fn clear_interrupt() {
    INTERRUPTED.store(false, Ordering::Relaxed);
}

/// Makes the first Ctrl-C stop the search, or the derivation or drawing
/// made after it, so that its verdict is unknown and the statistics still
/// print, and a second one end the program as Ctrl-C usually does, after
/// removing the files its outputs were being written to.
pub fn catch_interrupt() {
    // The handler runs on a thread of its own, once per signal, so it may
    // exit. Without it Ctrl-C ends the program, which is a fine fallback.
    let _ = ctrlc::set_handler(|| {
        if INTERRUPTED.swap(true, Ordering::Relaxed) {
            io::remove_partial_files();
            std::process::exit(130);
        }
    });
}

/// Makes Ctrl-C end the program as it usually does, after removing the
/// files its outputs were being written to: for a command that runs no
/// search.
pub fn exit_on_interrupt() {
    let _ = ctrlc::set_handler(|| {
        io::remove_partial_files();
        std::process::exit(130);
    });
}

/// How many characters of the input a parse error shows on either side of
/// the place where parsing failed.
const CONTEXT: usize = 60;

/// Returns a parse error as a message that points at the place in the input
/// where parsing failed: the line of the input the place is in, with a
/// caret under the character there. Of a long line only the part around
/// the place is shown, with `…` where it is cut, and the message then
/// says which character of the line it is, counting from 1; for an input
/// of several lines it says which line as well. It ends with what could
/// have stood there, as the parser says.
pub fn parse_error(input: &str, error: Error) -> anyhow::Error {
    let Error::Parse(parsed) = &error else {
        return error.into();
    };
    let errors = std::slice::from_ref(parsed.as_ref());
    if input.trim().is_empty() {
        return anyhow::Error::msg("the input is empty; the empty sequent is written |-");
    }
    let mut message = String::from("cannot parse the sequent");
    for e in errors {
        // The end of the input is shown after its last visible character,
        // and a place that is none of this input's as its nearest.
        let mut at = match e.found {
            Some(_) => e.span.start.min(input.len()),
            None => input.trim_end().len(),
        };
        while !input.is_char_boundary(at) {
            at -= 1;
        }
        let (before, after) = input.split_at(at);
        let line = before.rfind('\n').map_or(0, |end| end + 1);
        let before = &before[line..];
        let after = &after[..after.find('\n').unwrap_or(after.len())];
        // The part shown starts at most `CONTEXT` characters before the
        // place and ends at most as many after its start.
        let from = before
            .char_indices()
            .rev()
            .nth(CONTEXT - 1)
            .map_or(0, |(start, _)| start);
        let to = after
            .char_indices()
            .nth(CONTEXT)
            .map_or(after.len(), |(end, _)| end);
        // A tab or another character without a width of one would move
        // the caret off its place.
        let shown = |part: &str| -> String {
            part.chars()
                .map(|c| {
                    if c.is_whitespace() || c.is_control() {
                        ' '
                    } else {
                        c
                    }
                })
                .collect()
        };
        let cut = |is_cut: bool| if is_cut { "…" } else { "" };
        let indent = usize::from(from > 0) + before[from..].chars().count();
        let found = match &e.found {
            Some(token) => format!("unexpected {token:?}"),
            None => "unexpected end of input".into(),
        };
        write!(
            message,
            "\n  {}{}{}{}\n  {}^ {found}",
            cut(from > 0),
            shown(&before[from..]),
            shown(&after[..to]),
            cut(to < after.len()),
            " ".repeat(indent),
        )
        .unwrap();
        let character = before.chars().count() + 1;
        if input.trim_end().contains('\n') {
            let number = input[..line].matches('\n').count() + 1;
            write!(message, " at line {number}, character {character}").unwrap();
        } else if from > 0 {
            write!(message, " at character {character}").unwrap();
        }
        if let Some((last, rest)) = e.expected.split_last() {
            message.push_str(", expected ");
            if !rest.is_empty() {
                write!(message, "{} or ", rest.join(", ")).unwrap();
            }
            message.push_str(last);
        }
    }
    anyhow::Error::msg(message)
}

/// Runs the command the arguments name, with the flags given on the
/// command line.
fn run(cli: &Cli, given: &settings::Given) -> Result<Status> {
    match &cli.command {
        Command::Prove(args) => prove::prove(args, given),
        Command::Check(args) => prove::check(args, given),
        Command::Interact(args) => interact::interact(args, given),
        Command::Seq { command } => seq::run(command, given),
    }
}

/// Parses the command line arguments, runs the command, and returns the
/// exit status of its verdict, or 2 after printing an error.
pub fn main() -> ExitCode {
    let matches = Cli::command().get_matches();
    let cli = Cli::from_arg_matches(&matches).unwrap_or_else(|e| e.exit());
    match run(&cli, &settings::Given::of(&matches)) {
        Ok(status) => status.into(),
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::from(ERROR)
        }
    }
}
