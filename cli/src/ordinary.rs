// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Ordinary logic on the command line: a formula or sequent of classical,
//! intuitionistic or minimal logic read under `--logic`, translated into
//! linear logic, decided there, and its proof read back as LK or LJ.

use crate::argument_parsing::{InputFormat, Tree};
use crate::io::{bound, refused};
use crate::prove::{Ended, unknown};
use crate::show::{Show, Shown, not_built};
use anyhow::{Result, anyhow, bail};
use linlog::ordinary::{self, Image};
use linlog::proofs::Sides;
use linlog::search::{Outcome, Verdict};
use linlog::{Error, Limits, Proof, Refusal, ViewOptions};
use std::fmt::Write;

/// Returns the ordinary sequent `text` holds in `format`: text in the
/// syntax of ordinary logic (a parse error points into `text`), or a TPTP
/// problem, refused at the first subformula past `most` occurrences.
///
/// # Errors
///
/// Text that is no ordinary sequent, with a caret under the place, no
/// TPTP problem, or one past `most`.
pub(crate) fn sequent_in(text: &str, format: InputFormat, most: u64) -> Result<ordinary::Sequent> {
    let within = bound(most);
    match format {
        // The library's error says that the text is no TPTP problem.
        InputFormat::Tptp => Ok(ordinary::read_tptp(text, &within)
            .map_err(|e| refused(e, most))?
            .sequent),
        InputFormat::Text | InputFormat::Auto | InputFormat::Lines => {
            ordinary::Sequent::parse_within(text, &within).map_err(|e| match e {
                Error::Parse(_) => crate::parse_error(text, e),
                e => refused(e, most),
            })
        }
        _ => bail!(
            "--logic reads formulas as text, a line each with --input-format lines, or TPTP problems"
        ),
    }
}

/// Returns the image of an ordinary sequent in the logic, under the
/// translation, that `options` give, refused past `most` occurrences.
///
/// # Errors
///
/// The translation's error: one that does not decide the logic, more
/// than one formula right of `⊢` in intuitionistic or minimal logic, or
/// an image past `most`.
pub(crate) fn image(
    options: &ordinary::Options,
    sequent: &ordinary::Sequent,
    most: u64,
) -> Result<Image> {
    ordinary::translate(sequent, options.logic, options.translation(), &bound(most))
        .map_err(|e| refused(e, most))
}

/// Returns the first line of the text output for an ordinary sequent:
/// the verdict, the logic and the translation, the engine, and for an
/// undecided sequent why.
pub(crate) fn verdict_line(outcome: &Outcome, image: &Image, ended: &Ended) -> String {
    let translation = image.translation();
    let context = format!(
        "{} logic by the {translation} translation into {}, {} engine",
        image.logic(),
        translation.target(),
        outcome.engine
    );
    match &outcome.verdict {
        Verdict::Proved(_) => format!("valid ({context})"),
        Verdict::Unprovable(refutation) => {
            format!("not valid ({context}): the image is unprovable: {refutation}")
        }
        Verdict::Unknown(reason) => {
            format!("unknown ({context}): {}", unknown(*reason, outcome, ended))
        }
    }
}

/// Makes the derivation of LK or LJ a proof of the image reads back as
/// and writes it in the format `show` names, as
/// [`derivation`](crate::show::derivation) does for a linear one: within
/// the view's bounds, the time limit and Ctrl-C, fitted to a terminal.
/// The read-back is checked before it is written; one that fails the
/// check is an error, a defect to report.
pub(crate) fn derivation(
    image: &Image,
    proof: &Proof,
    show: &Show,
    mut halt: impl FnMut() -> bool,
    why: impl Fn() -> String,
    prefix: Option<&str>,
    out: &mut impl Write,
) -> Result<Shown> {
    if show.tree == Tree::Never {
        return Ok(Shown::Nothing);
    }
    let stopped = || Shown::LeftOut(format!("the derivation is not written: {}", why()));
    let d = match image.read_back(proof, &show.limits, |_| halt()) {
        Ok(d) => d,
        Err(Error::Refused(Refusal::Stopped { .. })) => return Ok(stopped()),
        Err(error) if error.is_refusal() => {
            let sides = if image.mode().is_intuitionistic() {
                Sides::Two
            } else {
                Sides::One
            };
            let view = ViewOptions::default().with_sides(sides);
            let size = || {
                proof
                    .derivation_size(&view, &Limits::default(), |_| false)
                    .ok()
            };
            return Ok(Shown::LeftOut(not_built(&error, size)));
        }
        Err(error @ Error::ReadBack { .. }) => return Err(error.into()),
        Err(error) => return Err(anyhow!(error).context("the proof cannot be unfolded")),
    };
    match d.check(&show.limits, |_| halt()) {
        Err(Error::Refused(Refusal::Stopped { .. })) => return Ok(stopped()),
        checked => checked?,
    }
    crate::show::shown(d, show.fit(), show, halt, why, prefix, out)
}
