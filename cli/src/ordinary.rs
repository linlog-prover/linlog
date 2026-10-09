// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Ordinary logic on the command line: a formula or sequent of classical,
//! intuitionistic or minimal logic read under `--logic`, translated into
//! linear logic, decided there, and its proof read back as LK or LJ.

use crate::argument_parsing::{Format, InputFormat, LogicArgs, Tree};
use crate::prove::{Ended, Prefixed, Show, Shown, not_built, render, unfit, unknown};
use anyhow::{Result, anyhow, bail};
use linlog::export::{latex, rocq, svg, typst};
use linlog::ordinary::{self, Image, Logic, Translation};
use linlog::proofs::Sides;
use linlog::search::{Outcome, Verdict};
use linlog::{Error, Limits, Proof, Refusal, ViewOptions};
use std::fmt::Write;

/// Returns the ordinary sequent `text` holds in `format`: text in the
/// syntax of ordinary logic (a parse error points into `text`), or a TPTP
/// problem.
pub fn sequent_in(text: &str, format: InputFormat) -> Result<ordinary::Sequent> {
    match format {
        // The library's error says that the text is no TPTP problem.
        InputFormat::Tptp => Ok(ordinary::read_tptp(text)?.sequent),
        InputFormat::Text | InputFormat::Auto | InputFormat::Lines => {
            text.parse().map_err(|e| crate::parse_error(text, e))
        }
        _ => bail!(
            "--logic reads formulas as text, a line each with --input-format lines, or TPTP problems"
        ),
    }
}

/// Returns the image of an ordinary sequent under the translation the
/// flags choose.
pub fn image(args: &LogicArgs, sequent: &ordinary::Sequent) -> Result<Image> {
    let logic: Logic = args.logic.expect("the caller reads ordinary logic").into();
    let translation = args
        .translation
        .map_or(Translation::default_for(logic), Into::into);
    Ok(ordinary::translate(sequent, logic, translation)?)
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
/// [`derivation`](crate::prove::derivation) does for a linear one: within
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
    let styles = &show.styles;
    if let Some((columns, most)) = show.fit() {
        let (width, lines) = d.text_size(&styles.text);
        let (width, lines) = (width as u64, lines as u64);
        if width > columns || lines > most {
            let inferences = d.inferences().len() as u64;
            return Ok(Shown::LeftOut(unfit(
                inferences,
                &width.to_string(),
                lines,
                columns,
                most,
            )));
        }
    }
    if show.format.is_binary() {
        let mut drawing = String::new();
        if svg::write(&d, &styles.svg, &mut drawing, |_| halt()).is_err() {
            return Ok(stopped());
        }
        drop(d);
        let halt = std::cell::RefCell::new(halt);
        let stop = || (0..crate::prove::STEPS_PER_CLOCK).any(|_| (halt.borrow_mut())());
        return Ok(match render(drawing, show.format, styles, &stop)? {
            crate::prove::Rendered::Bytes(bytes) => Shown::Rendered(bytes),
            crate::prove::Rendered::Refused(line) => Shown::LeftOut(line),
            crate::prove::Rendered::Stopped => stopped(),
        });
    }
    let mut out = Prefixed { out, prefix };
    let written = match show.format {
        Format::Latex => latex::write(&d, &styles.latex, &mut out, |_| halt()),
        Format::Typst => typst::write(&d, &styles.typst, &mut out, |_| halt()),
        Format::Svg => svg::write(&d, &styles.svg, &mut out, |_| halt()),
        Format::Rocq => rocq::write(&d, &styles.rocq, &mut out, |_| halt()),
        Format::Text | Format::Json | Format::Png | Format::Pdf => {
            d.write_text(&styles.text, &mut out, |_| halt())
        }
    };
    match written {
        Ok(()) => Ok(Shown::Written),
        Err(Error::Refused(Refusal::Stopped { .. })) => Ok(Shown::Cut(format!(
            "the derivation is cut short: {}",
            why()
        ))),
        Err(Error::Unsupported(e)) => Err(anyhow!(e).context("no certificate")),
        Err(_) => Ok(Shown::Written),
    }
}
