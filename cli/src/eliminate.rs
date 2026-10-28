// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use crate::argument_parsing::{EliminateArgs, Format};
use crate::check::{CHECKED_LEVEL, Checked, read_proof};
use crate::io;
use crate::limit::Deadline;
use crate::prove::{Stop, on_large_stack, stopped};
use crate::settings::{Given, time_limit};
use crate::show::{Show, Shown, abbreviated, derivation, every_steps};
use crate::{Status, exit_on_interrupt, interrupted};
use anyhow::Result;
use linlog::proofs::cut::{self, Elimination, End};
use linlog::{CheckError, Error, Forest, Mode, Node, Proof, Reading, Refusal, Settings};
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
    on_large_stack(settings.limits.stack_bytes(), move || {
        let mut out = io::Output::open(path.as_deref(), show.format.is_binary())?;
        let quiet = args.check.output.quiet;
        let (status, keep) =
            eliminate_into(&proof, mode, &settings, &show, quiet, &deadline, &mut out)?;
        if keep {
            out.finish()?;
        }
        anyhow::Ok(status)
    })?
}

/// Eliminates the cuts of the proof, writes the steps, the end and the
/// derivation reached into `out`, and returns the status and whether the
/// output is to be finished.
fn eliminate_into(
    proof: &Proof,
    mode: Mode,
    settings: &Settings,
    show: &Show,
    quiet: bool,
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
    let status = match &elimination.end {
        End::CutFree
        | End::Refused(Refusal::Elimination {
            what: linlog::limits::Bound::Steps,
            ..
        }) => Status::Yes,
        _ => Status::Unknown,
    };
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

/// Returns `count` and the noun for one or for many.
fn counted(count: usize, one: &str, many: &str) -> String {
    format!("{count} {}", if count == 1 { one } else { many })
}

/// Returns the lines that say what the elimination did: a line per step
/// with its case and the cut formula, then how it ended.
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
            format!(
                "step {}: {} on the cut of {formula}",
                k + 1,
                step.case.name()
            )
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
