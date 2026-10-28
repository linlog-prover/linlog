// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use crate::argument_parsing::{CheckArgs, Format};
use crate::io;
use crate::limit::Deadline;
use crate::prove::{Stop, on_large_stack, stopped};
use crate::settings::{Given, time_limit};
use crate::show::{
    Show, Shown, abbreviated, derivation, every_steps, net_into, net_json, nets_exist,
};
use crate::{Status, exit_on_interrupt, interrupted};
use anyhow::{Context, Result, bail};
use linlog::{CheckError, Error, ErrorKind, Forest, Limits, Mode, Proof, Reading};
use std::time::Instant;

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
    let mode = args.mode.mode();
    let quiet = args.output.quiet;
    let show = Show::new(&args.output, &settings)?;
    let path = args.output.output.clone();
    exit_on_interrupt();
    // The file is read and parsed within the time limit, as `prove` reads
    // its sequent: a large proof file takes seconds to parse.
    let (source, limits) = (args.proof.clone(), settings.limits);
    let loaded = deadline.within(move || {
        let text = io::read(source.as_deref(), "proof", limits.memory_bytes)?;
        read_proof(&text, &limits)
    })?;
    let Some(proof) = loaded else {
        let limit = deadline.limit().expect("only a limit passes");
        let line = format!(
            "unknown: the time limit of {limit:?} was reached while the proof was read; \
             --timeout DURATION gives the check longer"
        );
        let mut out = io::Output::open(path.as_deref(), show.format.is_binary())?;
        let keep = if show.format == Format::Json {
            let checked = Checked {
                version: CHECKED_LEVEL,
                verdict: "unknown",
                mode,
                error: None,
                message: Some(line),
            };
            serde_json::to_writer(out.stream(), &checked)?;
            true
        } else {
            // An unknown is written whatever --no-verdict says, as the
            // check's own is.
            show.open(&mut out, Some(&line))?;
            show.close(&mut out, Shown::Nothing, Some(&line))?
        };
        if keep {
            out.finish()?;
        }
        return Ok(Status::Unknown);
    };
    let proof = proof?;
    if proof.goal().is_some() {
        return Err(Error::GoalProof.into());
    }
    on_large_stack(settings.limits.stack_bytes(), move || {
        let mut out = io::Output::open(path.as_deref(), show.format.is_binary())?;
        let (status, keep) = check_into(&proof, mode, &show, quiet, &deadline, &mut out)?;
        if keep {
            out.finish()?;
        }
        anyhow::Ok(status)
    })?
}

/// The wire level of `check`'s JSON: level 1 holds a verdict, a mode and
/// an error whole.
pub(crate) const CHECKED_LEVEL: u32 = 1;

/// What `check --format json` writes.
#[derive(serde::Serialize)]
pub(crate) struct Checked {
    /// The wire level.
    pub(crate) version: u32,
    /// `valid`, `invalid` or `unknown`.
    pub(crate) verdict: &'static str,
    /// The mode checked in.
    pub(crate) mode: Mode,
    /// Why the proof is not valid, as the library writes its errors.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) error: Option<Error>,
    /// The error's message with formulas for its ids, or why the check
    /// was not made.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) message: Option<String>,
}

/// Reads a proof file, its sequent within `limits.occurrences`; the
/// outcome of a sequent that was not proved is refused with what it is.
///
/// # Errors
///
/// A document that is no proof, or past the limit.
pub(crate) fn read_proof(text: &str, limits: &Limits) -> Result<Proof> {
    let mut document = serde_json::Deserializer::from_str(text);
    match linlog::wire::upgrade::<Proof, _>(&mut document, limits) {
        Ok(proof) => {
            document
                .end()
                .context("not a proof in JSON: more follows it")?;
            Ok(proof)
        }
        // Only the occurrence bound has a flag: the width of the
        // representation's indices is no setting.
        Err(error) if error.setting() == Some("limits.occurrences") => {
            bail!("{error}; raise it with --occurrence-limit")
        }
        Err(error) if error.kind() == ErrorKind::Limit => Err(error.into()),
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
            .in_mode(mode)
            .abbreviated(show.abbreviate)
            .to_string()
    };
    if format == Format::Json {
        if show.net && result.is_ok() && !quiet {
            nets_exist(proof.sequent(), mode)?;
            let why = || Stop::why(stopped(deadline));
            if net_json(None, proof, mode, show, &over, &why, out)? {
                return Ok((status, true));
            }
        }
        let (error, message) = match &result {
            Ok(()) => (None, None),
            Err(e) => (Some(Error::from(e.clone())), Some(report(e))),
        };
        let checked = Checked {
            version: CHECKED_LEVEL,
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
            Reading::of_mode(&forest, mode)
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
    let halt = every_steps(&over);
    let shown = derivation(proof, mode, None, show, halt, why, prefix.as_deref(), out)?;
    Ok((status, show.close(out, shown, line.as_deref())?))
}
