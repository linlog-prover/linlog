// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use crate::argument_parsing::{EliminateArgs, Format};
use crate::check::{CHECKED_LEVEL, Checked, read_proof};
use crate::io;
use crate::limit::Deadline;
use crate::prove::{Stop, on_large_stack, stopped};
use crate::settings::{Given, time_limit};
use crate::show::{
    Net, Rendered, Show, Shown, abbreviated, derivation, every_steps, net_of, nets_exist, note,
    render,
};
use crate::{Status, exit_on_interrupt, interrupted};
use anyhow::{Result, bail};
use linlog::proofs::cut::{self, Elimination, End};
use linlog::{CheckError, Error, Forest, Mode, Node, Proof, Reading, Refusal, Settings};
use std::fmt::Write as _;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Runs `eliminate`: reads a proof, eliminates its cuts in the mode the
/// flags give under the time limit and the bounds, and prints the steps,
/// how the elimination ended and the derivation of the proof reached, or
/// the elimination's record as JSON.
///
/// # Errors
///
/// A file that is no proof in JSON, a proof of a goal, a step the checker
/// refuses (a defect of the library), or an output that cannot be made;
/// an invalid proof is a verdict, and an elimination a bound stopped
/// prints the proof it reached.
pub(crate) fn eliminate(args: &EliminateArgs, given: &Given) -> Result<Status> {
    let start = Instant::now();
    let settings = args.settings(given)?;
    let asked = asked_steps(given, &settings);
    let deadline = Deadline::start(time_limit(&settings), start)?;
    let check = &args.check;
    let mode = check.mode.mode();
    let show = Show::new(&check.output, &settings)?;
    let path = check.output.output.clone();
    exit_on_interrupt();
    let (source, limits) = (check.proof.clone(), settings.limits);
    let loaded = deadline.within(move || {
        let text = io::read(source.as_deref(), "proof", limits.memory_bytes)?;
        read_proof(&text, &limits)
    })?;
    let Some(proof) = loaded else {
        let limit = deadline.limit().expect("only a limit passes");
        let line = format!(
            "unknown: the time limit of {limit:?} was reached while the proof was read; \
             --timeout DURATION gives the elimination longer"
        );
        let mut out = io::Output::open(path.as_deref(), show.format.is_binary())?;
        if show.format == Format::Json {
            let unknown = Checked {
                version: CHECKED_LEVEL,
                verdict: "unknown",
                mode,
                error: None,
                message: Some(line),
            };
            serde_json::to_writer(out.stream(), &unknown)?;
            out.finish()?;
        } else {
            show.open(&mut out, Some(&line))?;
            if show.close(&mut out, Shown::Nothing, Some(&line))? {
                out.finish()?;
            }
        }
        return Ok(Status::Unknown);
    };
    let proof = proof?;
    if proof.goal().is_some() {
        return Err(Error::GoalProof.into());
    }
    if show.net {
        let quiet = args.check.output.quiet;
        return on_large_stack(settings.limits.stack_bytes(), move || {
            eliminate_net(
                &proof,
                mode,
                &settings,
                &show,
                quiet,
                asked,
                &deadline,
                path.as_deref(),
            )
        })?;
    }
    on_large_stack(settings.limits.stack_bytes(), move || {
        let mut out = io::Output::open(path.as_deref(), show.format.is_binary())?;
        let quiet = args.check.output.quiet;
        let (status, keep) = eliminate_into(
            &proof, mode, &settings, &show, quiet, asked, &deadline, &mut out,
        )?;
        if keep {
            out.finish()?;
        }
        anyhow::Ok(status)
    })?
}

/// Returns whether the bound on steps is one the user asked for, on the
/// command line or by a settings file's value other than the default,
/// rather than the default that keeps a call bounded.
fn asked_steps(given: &Given, settings: &Settings) -> bool {
    given.has("steps") || settings.cut.steps != Some(cut::Options::DEFAULT_STEPS)
}

/// Returns the exit status of an elimination that ended at `end`: done (0)
/// when no cut is left, or when it took the steps the user `asked` for;
/// unknown (3) when the default bound on steps, another bound or the stop
/// ended it with cuts left.
fn status(end: &End, asked: bool) -> Status {
    match end {
        End::CutFree => Status::Yes,
        End::Refused(Refusal::Elimination {
            what: linlog::limits::Bound::Steps,
            ..
        }) if asked => Status::Yes,
        _ => Status::Unknown,
    }
}

/// Eliminates the cuts of the proof, writes the steps, the end and the
/// derivation reached into `out`, and returns the status and whether the
/// output is to be finished; `asked` says whether the bound on steps is
/// the user's.
#[expect(
    clippy::too_many_arguments,
    reason = "the command's state, as `check_into` takes it, and the elimination's"
)]
fn eliminate_into(
    proof: &Proof,
    mode: Mode,
    settings: &Settings,
    show: &Show,
    quiet: bool,
    asked: bool,
    deadline: &Deadline,
    out: &mut io::Output,
) -> Result<(Status, bool)> {
    let over = || interrupted() || deadline.passed() || io::closed();
    let result = cut::eliminate(proof, mode, &settings.cut, &settings.limits, |_| over());
    let sequent = |proof: &Proof| {
        let formulas = proof.sequent().roots().len();
        match Forest::new(proof.sequent())
            .ok()
            .filter(|_| mode.is_intuitionistic())
            .and_then(|forest| {
                Reading::of_mode(&forest, mode)
                    .ok()
                    .map(|reading| abbreviated(&reading, show.abbreviate, formulas))
            }) {
            Some(two_sided) => two_sided,
            None => abbreviated(proof.sequent(), show.abbreviate, formulas),
        }
    };
    let elimination = match result {
        Ok(elimination) => elimination,
        Err(Error::Check(error)) => {
            let (status, message) = match &error {
                CheckError::Refused(refused) => (
                    Status::Unknown,
                    format!(
                        "unknown: the proof of {} ({mode}) is not checked: {error}{}",
                        sequent(proof),
                        advice(&refused.refusal, deadline)
                    ),
                ),
                _ => (
                    Status::No,
                    format!(
                        "invalid proof of {} ({mode}): {}",
                        sequent(proof),
                        error
                            .describe(proof.forest())
                            .in_mode(mode)
                            .abbreviated(show.abbreviate)
                    ),
                ),
            };
            if show.format == Format::Json {
                let checked = Checked {
                    version: CHECKED_LEVEL,
                    verdict: if matches!(status, Status::No) {
                        "invalid"
                    } else {
                        "unknown"
                    },
                    mode,
                    error: Some(Error::Check(error)),
                    message: Some(message),
                };
                serde_json::to_writer(out.stream(), &checked)?;
                return Ok((status, true));
            }
            show.open(out, Some(&message))?;
            let keep = show.close(out, Shown::Nothing, Some(&message))?;
            return Ok((status, keep));
        }
        Err(error) => return Err(error.into()),
    };
    let status = status(&elimination.end, asked);
    if show.format == Format::Json {
        serde_json::to_writer(out.stream(), &elimination)?;
        return Ok((status, true));
    }
    let line = report(&elimination, mode, &sequent(&elimination.proof), deadline);
    let prefix = show.open(out, show.verdict.then_some(line.as_str()))?;
    if quiet {
        let keep = show.close(out, Shown::Nothing, Some(&line))?;
        return Ok((status, keep));
    }
    let why = || Stop::why(stopped(deadline));
    let shown = derivation(
        &elimination.proof,
        mode,
        None,
        show,
        every_steps(&over),
        why,
        prefix.as_deref(),
        out,
    )?;
    Ok((
        status,
        show.close(out, shown, show.verdict.then_some(line.as_str()))?,
    ))
}

/// Returns the sequent of a proof as the verdict lines write it: in an
/// intuitionistic mode two-sided, abbreviated as `show` says.
fn sequent_of(proof: &Proof, mode: Mode, show: &Show) -> String {
    let formulas = proof.sequent().roots().len();
    match Forest::new(proof.sequent())
        .ok()
        .filter(|_| mode.is_intuitionistic())
        .and_then(|forest| {
            Reading::of_mode(&forest, mode)
                .ok()
                .map(|reading| abbreviated(&reading, show.abbreviate, formulas))
        }) {
        Some(two_sided) => two_sided,
        None => abbreviated(proof.sequent(), show.abbreviate, formulas),
    }
}

/// Eliminates the cuts of the proof's net and writes the steps and the
/// end, with the net reached as text, or the record as JSON, or each
/// step's net drawn into a file of its own: `FILE-k` (`net-1.svg` for
/// `net.svg`) the net step `k` was taken on, with its cut link and what
/// it acts on marked, and `FILE` the net reached.
/// Returns the status as [`eliminate_into`] does.
#[expect(
    clippy::too_many_arguments,
    reason = "the command's state, as `eliminate_into` takes it"
)]
fn eliminate_net(
    proof: &Proof,
    mode: Mode,
    settings: &Settings,
    show: &Show,
    quiet: bool,
    asked: bool,
    deadline: &Deadline,
    path: Option<&Path>,
) -> Result<Status> {
    let over = || interrupted() || deadline.passed() || io::closed();
    let drawn = matches!(show.format, Format::Svg | Format::Png | Format::Pdf);
    if !drawn {
        return eliminate_net_into(proof, mode, settings, show, quiet, asked, deadline, path);
    }
    let Some(path) = path else {
        bail!(
            "eliminate --net draws each step into a file of its own: --output FILE names them \
             FILE-1, FILE-2, … and the net reached FILE"
        );
    };
    nets_exist(proof.sequent(), mode)?;
    let why = || Stop::why(stopped(deadline));
    if let Some(status) = unchecked(proof, mode, settings, show, deadline, None)? {
        return Ok(status);
    }
    let net = match net_of(None, proof, mode, &show.limits, &over, &why)? {
        Ok(net) => net,
        Err(line) => {
            eprintln!("{line}");
            return Ok(Status::Unknown);
        }
    };
    // Each drawing with the line its file notes, or why it is left out.
    let mut drawings = Vec::new();
    let (reached, steps, end) = run(&net, mode, settings, &over, |before, step, line| {
        if !quiet {
            drawings.push((line, before.step_drawing(Some(step), show, &over)));
        }
    })?;
    let line = net_report(
        &steps,
        &end,
        &reached,
        mode,
        &sequent_of(proof, mode, show),
        deadline,
    );
    let last = line.lines().last().unwrap_or_default().to_owned();
    if !quiet {
        drawings.push((last, reached.step_drawing(None, show, &over)));
    }
    let last = drawings.len();
    for (k, (comment, drawing)) in drawings.into_iter().enumerate() {
        let file = if k + 1 == last {
            path.to_owned()
        } else {
            numbered(path, k + 1)
        };
        let drawing = match drawing {
            Ok(drawing) => drawing,
            Err(left_out) => {
                eprintln!("{}: {left_out}", file.display());
                continue;
            }
        };
        let mut out = io::Output::open(Some(&file), show.format.is_binary())?;
        if show.format == Format::Svg {
            if show.verdict {
                out.stream()
                    .write_all(note(Format::Svg, &comment).as_bytes())?;
                out.stream().write_all(b"\n")?;
            }
            out.stream().write_all(drawing.as_bytes())?;
        } else {
            match render(
                drawing,
                show.format,
                &show.styles,
                show.limits.memory_bytes,
                &over,
            )? {
                Rendered::Bytes(bytes) => out.stream().write_all(&bytes)?,
                Rendered::Refused(line) => bail!("{line}"),
                Rendered::Stopped => {
                    bail!("the drawing of {} was stopped: {}", file.display(), why())
                }
            }
        }
        out.finish()?;
    }
    if show.verdict {
        println!("{line}");
    }
    Ok(status(&end, asked))
}

/// Returns the file of step `k`'s drawing: `FILE-k` with the output's
/// extension.
fn numbered(path: &Path, k: usize) -> PathBuf {
    let stem = path
        .file_stem()
        .map_or_else(String::new, |s| s.to_string_lossy().into_owned());
    let name = match path.extension() {
        Some(ext) => format!("{stem}-{k}.{}", ext.to_string_lossy()),
        None => format!("{stem}-{k}"),
    };
    path.with_file_name(name)
}

/// Eliminates the cuts of the proof's net and writes the steps, the end
/// and the net reached as text, or the record as JSON, into the output.
#[expect(
    clippy::too_many_arguments,
    reason = "the command's state, as `eliminate_into` takes it"
)]
fn eliminate_net_into(
    proof: &Proof,
    mode: Mode,
    settings: &Settings,
    show: &Show,
    quiet: bool,
    asked: bool,
    deadline: &Deadline,
    path: Option<&Path>,
) -> Result<Status> {
    let over = || interrupted() || deadline.passed() || io::closed();
    nets_exist(proof.sequent(), mode)?;
    let mut out = io::Output::open(path, false)?;
    if let Some(status) = unchecked(proof, mode, settings, show, deadline, Some(&mut out))? {
        out.finish()?;
        return Ok(status);
    }
    let why = || Stop::why(stopped(deadline));
    let net = match net_of(None, proof, mode, &show.limits, &over, &why)? {
        Ok(net) => net,
        Err(line) => {
            eprintln!("{line}");
            return Ok(Status::Unknown);
        }
    };
    if show.format == Format::Json {
        let status = match &net {
            Net::Mll(net) => {
                let e = linlog::nets::cut::eliminate(
                    net.as_ref(),
                    &settings.cut,
                    &settings.limits,
                    |_| over(),
                    |_, _| {},
                )?;
                serde_json::to_writer(out.stream(), &e)?;
                status(&e.end, asked)
            }
            Net::Mell(net) => {
                let e = linlog::nets::cut::eliminate(
                    net,
                    &settings.cut,
                    &settings.limits,
                    |_| over(),
                    |_, _| {},
                )?;
                serde_json::to_writer(out.stream(), &e)?;
                status(&e.end, asked)
            }
        };
        out.finish()?;
        return Ok(status);
    }
    let (reached, steps, end) = run(&net, mode, settings, &over, |_, _, _| {})?;
    let line = net_report(
        &steps,
        &end,
        &reached,
        mode,
        &sequent_of(proof, mode, show),
        deadline,
    );
    let prefix = show.open(&mut out, show.verdict.then_some(line.as_str()))?;
    if quiet {
        if show.close(&mut out, Shown::Nothing, Some(&line))? {
            out.finish()?;
        }
        return Ok(status(&end, asked));
    }
    let text = format!("{}{}", prefix.unwrap_or_default(), reached.written_in(mode));
    out.write_str(&text)?;
    if show.close(
        &mut out,
        Shown::Written,
        show.verdict.then_some(line.as_str()),
    )? {
        out.finish()?;
    }
    Ok(status(&end, asked))
}

/// Eliminates the cut links of a net by the settings, giving `each` the
/// net every step was taken on, the step and its line, and returns the
/// net reached, each step's line and how it ended.
fn run<'n>(
    net: &Net<'n>,
    mode: Mode,
    settings: &Settings,
    over: &dyn Fn() -> bool,
    mut each: impl FnMut(&Net<'_>, &linlog::nets::cut::Step, String),
) -> Result<(Net<'n>, Vec<String>, End)> {
    use std::borrow::Cow;
    let mut lines = Vec::new();
    let mut seen = |before: &Net<'_>, step: &linlog::nets::cut::Step| {
        let line = format!(
            "{}. {} on the cut of {}",
            lines.len() + 1,
            step.case.name(),
            before.formula(step.cut.0, mode)
        );
        each(before, step, line.clone());
        lines.push(line);
    };
    let (reached, end) = match net {
        Net::Mll(net) => {
            let e = linlog::nets::cut::eliminate(
                net.as_ref(),
                &settings.cut,
                &settings.limits,
                |_| over(),
                |before, step| seen(&Net::Mll(Cow::Borrowed(before)), step),
            )?;
            (Net::Mll(Cow::Owned(e.net)), e.end)
        }
        Net::Mell(net) => {
            let e = linlog::nets::cut::eliminate(
                net,
                &settings.cut,
                &settings.limits,
                |_| over(),
                |before, step| seen(&Net::Mell(before.clone()), step),
            )?;
            (Net::Mell(e.net), e.end)
        }
    };
    Ok((reached, lines, end))
}

/// Returns the lines that say what an elimination on a net did: a line
/// per step, then how it ended.
fn net_report(
    steps: &[String],
    end: &End,
    reached: &Net<'_>,
    mode: Mode,
    sequent: &str,
    deadline: &Deadline,
) -> String {
    let mut lines = steps.to_vec();
    let count = counted(steps.len(), "step", "steps");
    lines.push(match end {
        End::CutFree => format!("cut-free proof net of {sequent} ({mode}) after {count}"),
        end => {
            let End::Refused(refusal) = end else {
                unreachable!("an elimination ends cut-free or refused")
            };
            format!(
                "proof net of {sequent} ({mode}) with {} left after {count}: {refusal}{}",
                counted(reached.cuts(), "cut link", "cut links"),
                advice(refusal, deadline)
            )
        }
    });
    lines.join("\n")
}

/// Checks the proof as the elimination on terms does before its first
/// step and writes what it says when it is no proof in the mode, or its
/// check is given up, into `out`, or to standard error without one;
/// returns the status then, `None` for a proof that checks.
fn unchecked(
    proof: &Proof,
    mode: Mode,
    settings: &Settings,
    show: &Show,
    deadline: &Deadline,
    out: Option<&mut io::Output>,
) -> Result<Option<Status>> {
    let over = || interrupted() || deadline.passed() || io::closed();
    let Err(error) = proof.check_within(mode, &settings.limits, |_| over()) else {
        return Ok(None);
    };
    let (status, message) = match &error {
        CheckError::Refused(refused) => (
            Status::Unknown,
            format!(
                "unknown: the proof of {} ({mode}) is not checked: {error}{}",
                sequent_of(proof, mode, show),
                advice(&refused.refusal, deadline)
            ),
        ),
        _ => (
            Status::No,
            format!(
                "invalid proof of {} ({mode}): {}",
                sequent_of(proof, mode, show),
                error
                    .describe(proof.forest())
                    .in_mode(mode)
                    .abbreviated(show.abbreviate)
            ),
        ),
    };
    match out {
        Some(out) if show.format == Format::Json => {
            let checked = Checked {
                version: CHECKED_LEVEL,
                verdict: if matches!(status, Status::No) {
                    "invalid"
                } else {
                    "unknown"
                },
                mode,
                error: Some(Error::Check(error)),
                message: Some(message),
            };
            serde_json::to_writer(out.stream(), &checked)?;
        }
        Some(out) => {
            show.open(out, Some(&message))?;
            show.close(out, Shown::Nothing, Some(&message))?;
        }
        None => eprintln!("{message}"),
    }
    Ok(Some(status))
}

/// Returns `count` and the noun for one or for many.
fn counted(count: usize, one: &str, many: &str) -> String {
    format!("{count} {}", if count == 1 { one } else { many })
}

/// Returns the lines that say what the elimination did: a numbered line
/// per step with its case and the cut formula, then how it ended.
fn report(elimination: &Elimination, mode: Mode, sequent: &str, deadline: &Deadline) -> String {
    let proof = &elimination.proof;
    let reading = Reading::of_mode(proof.forest(), mode)
        .ok()
        .filter(|_| mode.is_intuitionistic());
    let mut lines: Vec<String> = elimination
        .steps
        .iter()
        .enumerate()
        .map(|(k, step)| {
            let formula = match &reading {
                Some(reading) => reading.formula(proof.occurrence(step.formula)).to_string(),
                None => proof.formula(step.formula).to_string(),
            };
            format!("{}. {} on the cut of {formula}", k + 1, step.case.name())
        })
        .collect();
    let steps = counted(elimination.steps.len(), "step", "steps");
    lines.push(match &elimination.end {
        End::CutFree => format!("cut-free proof of {sequent} ({mode}) after {steps}"),
        end => {
            let End::Refused(refusal) = end else {
                unreachable!("an elimination ends cut-free or refused")
            };
            let cuts = proof
                .nodes()
                .iter()
                .filter(|n| matches!(n, Node::Cut(..)))
                .count();
            format!(
                "proof of {sequent} ({mode}) with {} left after {steps}: {refusal}{}",
                counted(cuts, "cut", "cuts"),
                advice(refusal, deadline)
            )
        }
    });
    lines.join("\n")
}

/// Returns how to lift the bound that refused the elimination, or why
/// the stop ended it.
fn advice(refusal: &Refusal, deadline: &Deadline) -> String {
    match refusal.setting() {
        Some("cut.steps") => "; --steps N allows more".to_owned(),
        Some("cut.nodes") => "; raise it with --node-limit N".to_owned(),
        Some("limits.memory_bytes") => "; raise it with --memory-limit SIZE".to_owned(),
        Some("limits.work") => "; raise it with --work-limit N".to_owned(),
        _ => match stopped(deadline) {
            Some(Stop::Timeout(t)) => format!(
                ": the time limit of {t:?} was reached; --timeout DURATION gives the \
                 elimination longer"
            ),
            _ => String::new(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::argument_parsing::{Cli, Command};
    use clap::{CommandFactory, FromArgMatches};

    /// The bound on steps is the user's when `--steps` is given or a file
    /// sets another value; reaching the default with cuts left is unknown,
    /// reaching the user's is done, and so is no cut left; every other
    /// bound is unknown.
    #[test]
    fn the_default_bound_on_steps_is_unknown() {
        let asked = |line: &[&str]| {
            let matches = Cli::command().try_get_matches_from(line).unwrap();
            let Command::Eliminate(args) = Cli::from_arg_matches(&matches).unwrap().command else {
                unreachable!("an eliminate command")
            };
            let given = Given::of(&matches);
            asked_steps(&given, &args.settings(&given).unwrap())
        };
        assert!(!asked(&["linlog", "eliminate", "p.json"]));
        assert!(asked(&["linlog", "eliminate", "--steps=1", "p.json"]));
        assert!(asked(&[
            "linlog",
            "eliminate",
            "--steps",
            "10000",
            "p.json"
        ]));
        // The ends of a proof with cuts at one step and at a bound on
        // nodes the first step passes.
        let limits = linlog::Limits::default();
        let mode = Mode::CLASSICAL;
        let sequent: linlog::Sequent = "A, A -o B |- B".parse().unwrap();
        let mut session = linlog::Interactive::new(&sequent, mode).unwrap();
        let goal = session.goals().next().unwrap();
        let formula = linlog::CutFormula::parse_within(
            "A -o B",
            mode,
            &linlog::InputOptions::default(),
            &limits,
        )
        .unwrap();
        session.cut(goal, &formula, &[1], &limits).unwrap();
        let options = linlog::Options::default();
        let view = linlog::ViewOptions::default();
        session.close_all(&options, &view, &limits, |_| false);
        let proof = session.proof(&limits, |_| false).unwrap();
        let end = |options: cut::Options| {
            cut::eliminate(&proof, mode, &options, &limits, |_| false)
                .unwrap()
                .end
        };
        let steps = end(cut::Options::default().with_steps(Some(1)));
        assert_eq!(status(&steps, false), Status::Unknown);
        assert_eq!(status(&steps, true), Status::Yes);
        assert_eq!(status(&end(cut::Options::default()), false), Status::Yes);
        let nodes = end(cut::Options::default().with_nodes(Some(1)));
        assert_eq!(status(&nodes, true), Status::Unknown);
    }
}
