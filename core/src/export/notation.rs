// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! What the export targets share besides the printer of formulas and
//! sequents (`sequents::notation`, whose table each target fills): the
//! walk over a derivation that writes one inference at a time.

use crate::Error;
use crate::proofs::InfId;
use crate::proofs::style::Drawn;
use std::fmt::Write;

/// One step of the walk over a derivation.
#[derive(Clone, Copy)]
pub(crate) enum Step {
    /// An inference is reached, before its premises; the root is at depth 0.
    Enter(InfId, usize),
    /// An inference is left, after its premises.
    #[cfg_attr(
        not(any(feature = "typst", feature = "svg")),
        expect(
            dead_code,
            reason = "only the layouts of their own read the depth on the way up"
        )
    )]
    Exit(InfId, usize),
}

/// Walks a derivation depth-first from the root, premises in their order,
/// and hands `visit` every inference once on the way up and once on the
/// way down, until it fails. Exits alone come in postfix order. The walk
/// keeps its own stack, so a derivation of any height fits.
pub(crate) fn walk<T: Drawn, E>(
    derivation: &T,
    mut visit: impl FnMut(Step) -> Result<(), E>,
) -> Result<(), E> {
    let mut stack = vec![Step::Enter(derivation.root(), 0)];
    while let Some(step) = stack.pop() {
        visit(step)?;
        if let Step::Enter(id, depth) = step {
            stack.push(Step::Exit(id, depth));
            for &p in derivation.premises(id).iter().rev() {
                stack.push(Step::Enter(p, depth + 1));
            }
        }
    }
    Ok(())
}

/// Writes what `buffer` holds to `out`, empties it, and fails if `stop`
/// says so: the emitters make one inference at a time in a buffer and
/// hand it on, so that they hold one inference's text and can be stopped
/// between two.
pub(crate) fn flush(
    out: &mut impl Write,
    buffer: &mut String,
    stop: &mut impl FnMut() -> bool,
) -> Result<(), Error> {
    out.write_str(buffer)?;
    buffer.clear();
    if stop() {
        Err(Error::Refused(crate::limits::Refusal::Stopped {
            phase: crate::limits::Phase::Write,
        }))
    } else {
        Ok(())
    }
}
