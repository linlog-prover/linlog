// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! `seq print`, `seq json` and `seq fragment`: a sequent printed, written
//! as JSON, or named by its fragment.

use crate::argument_parsing::{Format, SeqCommand, SequentFormat};
use crate::settings::{self, Given};
use crate::{Status, exit_on_interrupt, io, prove, style};
use anyhow::{Result, bail};
use linlog::Mode;
use std::io::{IsTerminal, Write as _};

/// Returns the mode of `seq`'s `--intuitionistic` flag.
const fn mode_of(intuitionistic: bool) -> Mode {
    if intuitionistic {
        Mode::INTUITIONISTIC
    } else {
        Mode::CLASSICAL
    }
}

/// Runs a `seq` command.
///
/// # Errors
///
/// An input that cannot be read or is no sequent, a settings file or a
/// style that cannot be read, an output that cannot be made, and a
/// drawing the memory bound refuses.
pub(crate) fn run(command: &SeqCommand, given: &Given) -> Result<Status> {
    match command {
        SeqCommand::Print {
            input,
            intuitionistic,
            logic,
            format,
            standalone,
            output,
            style,
            memory_limit,
            settings,
        } => {
            exit_on_interrupt();
            let settings = settings::of_print(
                settings.as_deref(),
                memory_limit.0,
                input.occurrence_limit.0,
                given,
            )?;
            if logic.logic.is_some() && *intuitionistic {
                bail!("the logic decides how its image is printed: leave out --intuitionistic");
            }
            let most = settings::most(&settings);
            let (sequent, mode) = match settings::ordinary(&settings, logic) {
                Some(options) => {
                    let image = input.image(&options, most)?.1;
                    let mode = image.mode();
                    (image.sequent().clone(), mode)
                }
                None => (input.sequent(most)?, mode_of(*intuitionistic)),
            };
            prove::form(
                *standalone,
                matches!(format, SequentFormat::Latex | SequentFormat::Typst),
            )?;
            let (key, binary) = match format {
                SequentFormat::Text => ("text", None),
                SequentFormat::Latex => ("latex", None),
                SequentFormat::Typst => ("typst", None),
                SequentFormat::Svg => ("svg", None),
                SequentFormat::Png => ("png", Some(Format::Png)),
                SequentFormat::Pdf => ("pdf", Some(Format::Pdf)),
            };
            let styles = style::read(&settings.styles, style, Some(key), *standalone)?;
            let text = prove::sequent_in(&sequent, mode, *format, &styles)?;
            match binary {
                None => io::write(output.as_deref(), &text)?,
                Some(format) => {
                    if output.is_none() && std::io::stdout().is_terminal() {
                        bail!(
                            "a {} is not for a terminal: write it with --output FILE",
                            format.title()
                        );
                    }
                    // The sequent is all the output: a render the bound
                    // refuses is an error.
                    let memory = settings.limits.memory_bytes;
                    let bytes = match prove::render(text, format, &styles, memory, &|| false)? {
                        prove::Rendered::Bytes(bytes) => bytes,
                        prove::Rendered::Refused(line) => bail!("{line}"),
                        prove::Rendered::Stopped => unreachable!("nothing stops it"),
                    };
                    let mut out = io::Output::open(output.as_deref(), true)?;
                    out.stream().write_all(&bytes)?;
                    out.finish()?;
                }
            }
        }
        SeqCommand::Json {
            input,
            optimize,
            output,
        } => {
            let mut sequent = input.sequent(input.most())?;
            if *optimize {
                sequent.optimize();
            }
            io::write(output.as_deref(), &serde_json::to_string(&sequent)?)?;
        }
        SeqCommand::Fragment {
            input,
            intuitionistic,
        } => {
            let mode = mode_of(*intuitionistic);
            let fragment = input.sequent(input.most())?.fragment();
            io::write(None, fragment.name_in(mode))?;
        }
    }
    Ok(Status::Yes)
}
