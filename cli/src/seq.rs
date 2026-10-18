// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! `seq print`, `seq json` and `seq fragment`: a sequent printed, written
//! as JSON, or named by its fragment.

use crate::argument_parsing::{Format, SeqCommand};
use crate::settings::{self, Given};
use crate::{Status, exit_on_interrupt, io, show, style};
use anyhow::{Result, bail};
use linlog::{Mode, Order};
use std::io::Write as _;

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
            cyclic,
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
            if logic.logic.is_some() && (*intuitionistic || *cyclic) {
                bail!(
                    "the logic decides how its image is printed: leave out --intuitionistic and --cyclic"
                );
            }
            let (sequent, mode) = match settings::ordinary(&settings, logic) {
                Some(options) => {
                    let image = input.image(&options, &settings.limits)?.1;
                    let mode = image.mode();
                    (image.sequent().clone(), mode)
                }
                None => {
                    let mode = if *cyclic {
                        Mode::CYCLIC
                    } else {
                        mode_of(*intuitionistic)
                    };
                    (input.sequent(&settings.limits, mode.order())?, mode)
                }
            };
            show::form(*standalone, Format::from(*format).has_document())?;
            let written = Format::from(*format);
            let binary = written.is_binary().then_some(written);
            let styles = style::read(
                &settings.styles,
                style,
                None,
                written.style_key(),
                *standalone,
            )?;
            let text = show::sequent_in(&sequent, mode, *format, &styles, &settings.limits)?;
            match binary {
                None => io::write(output.as_deref(), &text)?,
                Some(format) => {
                    show::for_terminal(format, output.as_deref())?;
                    // The sequent is all the output: a render the bound
                    // refuses is an error.
                    let memory = settings.limits.memory_bytes;
                    let bytes = match show::render(text, format, &styles, memory, &|| false)? {
                        show::Rendered::Bytes(bytes) => bytes,
                        show::Rendered::Refused(line) => bail!("{line}"),
                        show::Rendered::Stopped => unreachable!("nothing stops it"),
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
            cyclic,
            output,
            memory_limit,
        } => {
            let limits = io::bound(input.most()).with_memory_bytes(memory_limit.0);
            let order = if *cyclic {
                Order::Cyclic
            } else {
                Order::Commutative
            };
            let mut sequent = input.sequent(&limits, order)?;
            if *optimize {
                sequent.optimize();
            }
            io::write(output.as_deref(), &serde_json::to_string(&sequent)?)?;
        }
        SeqCommand::Fragment {
            input,
            intuitionistic,
            memory_limit,
        } => {
            let mode = mode_of(*intuitionistic);
            let limits = io::bound(input.most()).with_memory_bytes(memory_limit.0);
            let fragment = input.sequent(&limits, Order::Commutative)?.fragment();
            io::write(None, fragment.name_in(mode))?;
        }
    }
    Ok(Status::Yes)
}
