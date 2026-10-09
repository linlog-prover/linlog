// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Deciding an ordinary sequent in one call: the translation, the search
//! on the image, and for a valid sequent the derivation of LK or LJ read
//! back and checked.

use super::{Derivation, Logic, Options, Sequent, Translation, translate};
use crate::Error;
use crate::limits::{Limits, Progress};
use crate::search::{self, Reason};

/// What deciding an ordinary sequent found, with the linear search's own
/// outcome on the image.
#[non_exhaustive]
#[derive(Clone, Debug)]
pub struct Outcome {
    /// The verdict, with the derivation of a valid sequent.
    pub verdict: Verdict,
    /// The logic decided.
    pub logic: Logic,
    /// The translation the image was made by.
    pub translation: Translation,
    /// The search's outcome on the image.
    pub linear: search::Outcome,
}

/// Whether an ordinary sequent is valid in its logic.
#[non_exhaustive]
#[derive(Clone, Debug)]
pub enum Verdict {
    /// The sequent is valid, and here is its derivation of LK or LJ, read
    /// back from the proof of its image and checked.
    Valid(Box<Derivation>),
    /// The image is unprovable, so the sequent is not valid; the linear
    /// outcome's disproof says why.
    NotValid,
    /// The search stopped before it could decide, for the reason given.
    Unknown(Reason),
}

/// Decides an ordinary sequent as `options` say: translates it, searches
/// the image in its mode under `search` within `limits` and until `stop`
/// fires, and for a proof reads the derivation of LK or LJ back and
/// checks it, under the same limits and stop.
///
/// # Errors
///
/// As [`translate`], [`search::prove_within`], [`Image::read_back`] and
/// [`Derivation::check`]: a proof whose derivation a bound or the stop
/// refused is that refusal, not a verdict, and a read-back that fails the
/// check is [`Error::ReadBack`], a defect.
///
/// [`Image::read_back`]: super::Image::read_back
///
/// # Examples
///
#[cfg_attr(feature = "parse", doc = "```")]
#[cfg_attr(not(feature = "parse"), doc = "```ignore")]
/// use linlog::Limits;
/// use linlog::ordinary::{self, Logic, Verdict};
///
/// let sequent: ordinary::Sequent = "|- ((a -> b) -> a) -> a".parse()?;
/// let options = ordinary::Options::default().with_logic(Logic::Classical);
/// let search = linlog::Options::default();
/// let outcome = ordinary::decide(&sequent, &options, &search, &Limits::default(), |_| false)?;
/// let Verdict::Valid(derivation) = outcome.verdict else {
///     panic!("Peirce's law holds classically");
/// };
/// assert_eq!(derivation.logic(), Logic::Classical);
/// let options = options.with_logic(Logic::Intuitionistic);
/// let outcome = ordinary::decide(&sequent, &options, &search, &Limits::default(), |_| false)?;
/// assert!(matches!(outcome.verdict, Verdict::NotValid));
/// # Ok::<(), linlog::Error>(())
/// ```
pub fn decide(
    sequent: &Sequent,
    options: &Options,
    search: &search::Options,
    limits: &Limits,
    mut stop: impl FnMut(Progress) -> bool,
) -> Result<Outcome, Error> {
    let (logic, translation) = (options.logic, options.translation());
    let image = translate(sequent, logic, translation, limits)?;
    let linear = search::prove_within(image.sequent(), image.mode(), search, limits, &mut stop)?;
    let verdict = match &linear.verdict {
        search::Verdict::Proved(proof) => {
            let derivation = image.read_back(proof, limits, &mut stop)?;
            derivation.check(limits, &mut stop)?;
            Verdict::Valid(Box::new(derivation))
        }
        search::Verdict::Unprovable(_) => Verdict::NotValid,
        search::Verdict::Unknown(reason) => Verdict::Unknown(*reason),
    };
    Ok(Outcome {
        verdict,
        logic,
        translation,
        linear,
    })
}
