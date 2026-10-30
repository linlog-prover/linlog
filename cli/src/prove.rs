// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use crate::argument_parsing::{Format, ProveArgs};
use crate::io;
use crate::limit::{Deadline, Notice};
use crate::settings::{Given, Threads, time_limit};
use crate::show::{
    Show, Shown, derivation, disproof_into, every_steps, net_into, net_json, nets_exist, note,
};
use crate::{Status, catch_interrupt, interrupted};
use anyhow::{Context, Result, bail};
use linlog::ordinary::Image;
use linlog::search::{Engine, Goal, Options, Outcome, Reason, Verdict, prove_goal, race};
use linlog::{CheckError, Error, Forest, Limits, Phase, Progress, Sequent};
use std::fmt::Write;
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
/// (`search::race`). `halt` is the command's stop condition, but for the
/// refutation after the search ([`halting`]). The single search runs on
/// the calling thread, whose stack the caller sized.
pub(crate) fn searched(
    goal: Goal<'_>,
    mode: linlog::Mode,
    options: &Options,
    limits: &linlog::Limits,
    threads: Threads,
    halt: &(dyn Fn() -> bool + Sync),
) -> Result<Outcome, linlog::Error> {
    let halt = halting(halt);
    match threads.alone {
        None => prove_goal(
            goal,
            mode,
            &options.clone().with_jobs(threads.jobs),
            limits,
            halt,
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
                halt,
            )
        }
    }
}

/// Returns the stop condition of a search from the command's, `halt`:
/// asked by the refutation after the search, Ctrl-C alone, since that
/// refutation is bounded by its work and answers a search that ran to the
/// time limit.
pub(crate) fn halting(
    halt: &(dyn Fn() -> bool + Sync),
) -> impl Fn(Progress) -> bool + Sync + Copy + '_ {
    move |progress: Progress| match progress.phase {
        Phase::Refute => interrupted(),
        _ => halt(),
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

/// Returns a search error with formulas where the library's message has
/// occurrence ids.
pub(crate) fn describe(error: Error, sequent: &Sequent) -> anyhow::Error {
    // The line is the command's; the error stays for a batch's record.
    let line = match (&error, Forest::within(sequent, &Limits::UNBOUNDED)) {
        (Error::NotIntuitionistic(_), Ok(forest)) => error.describe(&forest).to_string(),
        (Error::Check(CheckError::Refused(_)), _) => format!(
            "the search found a proof, but {error}; raise the limit with --memory-limit, or take \
             the proof unchecked with --no-check"
        ),
        _ => return error.into(),
    };
    crate::Explained::error(line, error)
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
    let (mode, read) = (args.mode.mode(), settings.input);
    let ordinary = crate::settings::ordinary(&settings, &args.logic);
    // How the sequent was written is kept for a derivation drawn so.
    let keep = settings.view.classical == linlog::ClassicalSides::Two;
    let loaded = deadline.within(move || match ordinary {
        Some(options) => input
            .image(&options, &limits)
            .map(|(forest, image)| (forest, Some(image), None)),
        None => input
            .forest(&limits, &read, mode, keep)
            .map(|(forest, writing)| (forest, None, writing)),
    })?;
    let Some(forest) = loaded else {
        let limit = deadline.limit().expect("only a limit passes");
        return unread_into(args, limit);
    };
    let (forest, image, writing) = forest?;
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
    if let Some(path) = &args.output.output {
        match std::fs::metadata(path) {
            Ok(meta) if meta.is_dir() => {
                bail!("cannot write {}: it is a directory", path.display())
            }
            Ok(meta) if !meta.is_file() => {}
            _ => drop(io::Output::open(Some(path), format.is_binary())?),
        }
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
            let net = match &outcome.verdict {
                Verdict::Proved(proof) if show.net => {
                    let over = || interrupted() || deadline.passed() || io::closed();
                    let why = || Stop::why(stopped(&deadline));
                    let found = outcome.net.as_deref();
                    net_json(found, proof, mode, &show, &over, &why, &mut out)?
                }
                _ => false,
            };
            if !net {
                serde_json::to_writer(out.stream(), &outcome)?;
            }
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
        let halt = every_steps(&over);
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
                _ => derivation(
                    proof,
                    mode,
                    writing.as_deref(),
                    &show,
                    halt,
                    why,
                    prefix.as_deref(),
                    &mut out,
                )?,
            },
            (Verdict::Unprovable(disproof), Format::Rocq, false) => {
                disproof_into(disproof, image.as_ref(), &show, prefix.as_deref(), &mut out)
            }
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
        // No bound was set, and the allocator refused: no flag gives more.
        (
            Reason::MemoryLimit {
                limit_bytes: u64::MAX,
                ..
            },
            _,
        ) => format!("{reason} {after}{at}"),
        (Reason::MemoryLimit { .. } | Reason::Unchecked { .. }, _) => {
            format!("{reason} {after}{at}; raise it with --memory-limit SIZE")
        }
        (Reason::WitnessDepth { .. }, _) => {
            format!("{reason} {after}{at}; raise it with --witness-depth N")
        }
        (Reason::Instances { .. }, _) => {
            format!("{reason} {after}{at}; raise it with --instances N")
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
            "unifications" | "bindings" => !outcome.fragment.has_quantifiers(),
            _ => false,
        };
        if !idle {
            writeln!(lines, "{}: {value}", counter.label_in(outcome.mode))
                .expect("a string takes it");
        }
    }
    write!(lines, "time: {elapsed:.2?}").expect("a string takes it");
    lines
}
