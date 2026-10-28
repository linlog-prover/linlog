// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Cut elimination on proof nets: a step reduces one cut link, the
//! caller's or the strategy's, and says which and by which case;
//! [`eliminate`] takes steps by the strategy until no cut link is left or
//! a bound is reached.
//!
//! A step rewrites the net around its cut link, as the literature does
//! (Girard, *Linear logic*, TCS 50, 1987; for the generalised `?` node,
//! Accattoli, arXiv:1303.7326 §7):
//!
//! - **axiom**: a cut link between two literals, each linked by an axiom
//!   link to another literal: the two axiom links and the cut become one
//!   axiom link between those two literals;
//! - **`⊗` against `⅋`** (`tensor_par`): the cut link and the two links
//!   become two cut links, between the left premises and between the
//!   right ones;
//! - a box `!A` against a `?A⊥` vertex, by its copies: none, **weakening**:
//!   the box and the `?` vertex are erased, and every `?` vertex outside
//!   whose copies were all in the box, and every weakening that hung in
//!   it, hangs on the axiom link the erased `?` vertex hung on; one copy
//!   at the cut's depth, **dereliction**: the box is opened and its
//!   premise cut against the copy; one copy inside a box at the cut's
//!   depth, **box_into_box**: the box and the `?` vertex enter that box,
//!   the cut with them; several copies, **contraction**: the box is
//!   duplicated, and a copy gets a `?` vertex of its own cut against the
//!   duplicate, so that the next step on it is one of the two before.
//!
//! Nets have no commutative case: the rules a term's commutative cases
//! move a cut across are no part of a net.
//!
//! In MLL a step makes no vertex: it takes two out of the structure, whose
//! vertices are the forest's occurrences, so the net reached keeps the
//! ids of the net given. In MELL a contraction duplicates a box, whose
//! copies are new instances of the same occurrences, so every step
//! renumbers the vertices: the roots first, then every root's tree and
//! every cut link's two trees, depth first.
//!
//! A step is checked, not trusted: [`step`] returns a net the criterion
//! accepts, and [`eliminate`] checks every net it reaches before its next
//! step and the last one before it returns. Every net made is a proof net
//! of the same conclusion over the same forest, and sequentializes into a
//! proof the checker accepts, which the tests show on every step.

/// The reductions of a structure with boxes.
mod mell;
#[cfg(all(test, feature = "parse"))]
mod tests;

use super::{NetError, ProofStructure, VertexId, afford};
use crate::Error;
use crate::limits::{Bound, Limits, Phase, Progress, Refusal};
use crate::proofs::cut::{Case, End, Options, Strategy};
use crate::sequents::Kind;

/// A proof structure whose cuts can be eliminated: [`ProofStructure`]
/// and [`MellStructure`](super::MellStructure).
pub trait Net: Clone + crate::sealed::Net {}

impl Net for ProofStructure {}

impl Net for super::MellStructure {}

/// One step of an elimination on a net: which cut link it reduced and
/// how, in vertices of the net the step was taken on, which a drawing of
/// that net marks.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Step {
    /// The cut link reduced, from its end in a cut formula's tree.
    pub cut: (VertexId, VertexId),
    /// The case: [`Case::Axiom`], [`Case::TensorPar`],
    /// [`Case::Weakening`], [`Case::Dereliction`], [`Case::Contraction`]
    /// or [`Case::BoxIntoBox`].
    pub case: Case,
    /// Where the step acted besides the cut link, if anywhere: the copy a
    /// dereliction cut the box's premise against or a contraction gave a
    /// `?` vertex of its own, the `!` vertex of the box a box entered.
    pub above: Option<VertexId>,
}

/// A step and the net it made.
#[non_exhaustive]
#[derive(Clone, Debug)]
pub struct Reduction<N> {
    /// The net after the step, over the same forest and of the same
    /// conclusion, which the criterion accepted.
    pub net: N,
    /// What the step did.
    pub step: Step,
}

/// The outcome of an elimination on a net: the last net it reached, the
/// steps that led there and why it ended.
#[non_exhaustive]
#[derive(Clone, Debug)]
pub struct Elimination<N> {
    /// The last net reached, which the criterion accepted: without cut
    /// links when the elimination ended at [`End::CutFree`].
    pub net: N,
    /// The steps taken, in order, each on the net the step before made.
    pub steps: Vec<Step>,
    /// Why the elimination ended.
    pub end: End,
}

/// Returns the cut link of `net` a step reduces: the one at the vertex
/// `at`, either end, or the one the strategy picks, the first of the
/// net's list. A step puts the cut links it makes in the place of the one
/// it reduced, so the strategy reduces the first cut formula's cut links
/// to the end before it turns to the next one's.
fn pick<N: Net>(
    net: &N,
    at: Option<VertexId>,
    strategy: Strategy,
) -> Result<(VertexId, VertexId), NetError> {
    let cuts = net.cut_links();
    match at {
        Some(v) => cuts
            .iter()
            .copied()
            .find(|&(x, y)| x == v || y == v)
            .ok_or(NetError::NoCut { vertex: Some(v) }),
        None => match strategy {
            Strategy::Topmost => cuts
                .first()
                .copied()
                .ok_or(NetError::NoCut { vertex: None }),
        },
    }
}

/// Returns the error of the check of a net a step made: a refusal as it
/// is, any other a defect.
fn rejected(error: NetError) -> Error {
    if matches!(error, NetError::Refused { .. }) {
        return error.into();
    }
    NetError::Reduction {
        error: Box::new(error),
    }
    .into()
}

/// Returns the refusal an error is, if it is one.
fn refusal(error: &Error) -> Option<&Refusal> {
    if let Error::Refused(refusal) = error {
        return Some(refusal);
    }
    if let Error::Net(net) = error
        && let NetError::Refused { refusal } = net.as_ref()
    {
        return Some(refusal);
    }
    None
}

/// Reduces one cut link of `net`, the one at the vertex `at` (either of
/// its ends), or the one the strategy of `options` picks: checks that
/// `net` is a proof net, takes the step and checks the net it made, each
/// check asking `stop`. A step is bounded by `options.nodes`, which
/// counts a net's vertices, and by `limits.memory_bytes`.
///
/// # Errors
///
/// The criterion's [`NetError`] for a structure that is no proof net;
/// [`NetError::NoCut`] for an `at` at no cut link, or a net without one;
/// [`Refusal::Elimination`] with [`Bound::Vertices`] for a net made past
/// `options.nodes`, and the refusals of the checks and of the bound on
/// memory; [`NetError::Reduction`] where the criterion refuses the net
/// made, a defect of this crate.
pub fn step<N: Net>(
    net: &N,
    at: Option<VertexId>,
    options: &Options,
    limits: &Limits,
    mut stop: impl FnMut(Progress) -> bool,
) -> Result<Reduction<N>, Error> {
    net.check(&mut stop)?;
    let cut = pick(net, at, options.strategy)?;
    let (made, step) = net.reduce(cut, options, limits)?;
    made.check(&mut stop).map_err(rejected)?;
    Ok(Reduction { net: made, step })
}

/// Eliminates the cut links of `net` by the strategy of `options`, step by
/// step, until none is left or a bound of `options` or `limits` or the
/// stop ends it: the net reached is then returned with its cut links and
/// the refusal that ended it. Every net reached is checked before the
/// next step is taken on it, and the last before it is returned; `each`
/// is given every step once the net it made has passed its check, with
/// the net it was taken on, which is how a caller draws every step. The
/// stop is asked between steps and by the checks.
///
/// Every strategy ends: in MLL a step takes two vertices out of the net,
/// and nets with boxes are strongly normalising (Girard, *Linear logic*,
/// 1987, §III.4, for the nets of MELL), so the bound on steps bounds time,
/// not termination.
///
/// # Errors
///
/// The criterion's [`NetError`] for a structure that is no proof net, or
/// its refusal of the first check; [`NetError::Reduction`] where the
/// criterion refuses a net a step made, a defect of this crate.
pub fn eliminate<N: Net>(
    net: &N,
    options: &Options,
    limits: &Limits,
    mut stop: impl FnMut(Progress) -> bool,
    mut each: impl FnMut(&N, &Step),
) -> Result<Elimination<N>, Error> {
    let mut steps: Vec<Step> = Vec::new();
    // The net reached last and the one before it, which is the last one
    // checked while the latest waits for its check.
    let mut current = net.clone();
    let mut checked: Option<N> = None;
    loop {
        let ended = |net: N, steps: Vec<Step>, refusal: Refusal| Elimination {
            net,
            steps,
            end: End::Refused(refusal),
        };
        match current.check(&mut stop) {
            Ok(()) => {}
            Err(NetError::Refused { refusal }) => {
                let Some(checked) = checked else {
                    return Err(NetError::Refused { refusal }.into());
                };
                // The latest net is unchecked: the one before it is the
                // net reached.
                steps.pop();
                return Ok(ended(checked, steps, refusal));
            }
            Err(invalid) if checked.is_none() => return Err(invalid.into()),
            Err(invalid) => return Err(rejected(invalid)),
        }
        if let (Some(before), Some(last)) = (&checked, steps.last()) {
            each(before, last);
        }
        let Ok(cut) = pick(&current, None, options.strategy) else {
            return Ok(Elimination {
                net: current,
                steps,
                end: End::CutFree,
            });
        };
        if let Some(limit) = options.steps
            && steps.len() as u64 >= limit
        {
            let refusal = Refusal::Elimination {
                what: Bound::Steps,
                limit,
            };
            return Ok(ended(current, steps, refusal));
        }
        if stop(Progress::new(Phase::Eliminate, 1, steps.len() as u64)) {
            let refusal = Refusal::Stopped {
                phase: Phase::Eliminate,
            };
            return Ok(ended(current, steps, refusal));
        }
        match current.reduce(cut, options, limits) {
            Ok((made, step)) => {
                steps.push(step);
                checked = Some(std::mem::replace(&mut current, made));
            }
            Err(error) => match refusal(&error) {
                Some(refusal) => {
                    let refusal = refusal.clone();
                    return Ok(ended(current, steps, refusal));
                }
                None => return Err(error),
            },
        }
    }
}

/// Fails with [`Refusal::Elimination`] when a net of `vertices` passes
/// the bound of `options`.
fn within(vertices: usize, options: &Options) -> Result<(), Error> {
    match options.nodes {
        Some(limit) if vertices as u64 > limit => Err(Error::Refused(Refusal::Elimination {
            what: Bound::Vertices,
            limit,
        })),
        _ => Ok(()),
    }
}

impl crate::sealed::Net for ProofStructure {
    fn cut_links(&self) -> &[(VertexId, VertexId)] {
        self.cuts()
    }

    fn check(&self, stop: &mut dyn FnMut(Progress) -> bool) -> Result<(), NetError> {
        self.is_correct(stop)
    }

    fn reduce(
        &self,
        (x, y): (VertexId, VertexId),
        options: &Options,
        limits: &Limits,
    ) -> Result<(Self, Step), Error> {
        let forest = self.forest();
        let at = self
            .cuts()
            .iter()
            .position(|&cut| cut == (x, y))
            .expect("a cut link of the net");
        let mut cuts = self.cuts().to_vec();
        let mut links = self.links().to_vec();
        let case = match forest.kind(x.occ()) {
            Kind::Atom | Kind::DualAtom => {
                // A proof net links every literal, and never a cut's two
                // ends to each other.
                let (px, py) = (
                    self.partner(x).expect("a proof net links every literal"),
                    self.partner(y).expect("a proof net links every literal"),
                );
                let ends = |&(u, v): &(VertexId, VertexId)| [x, y].contains(&u) || [x, y].contains(&v);
                let first = links.iter().position(ends).expect("the literal's link");
                links[first] = (px, py);
                links.retain(|link| !ends(link));
                cuts.remove(at);
                Case::Axiom
            }
            Kind::Tensor | Kind::Par => {
                let child = |v: VertexId, left: bool| {
                    let o = v.occ();
                    VertexId::of(if left { forest.left(o) } else { forest.right(o) }.unwrap())
                };
                cuts.splice(
                    at..=at,
                    [
                        (child(x, true), child(y, true)),
                        (child(x, false), child(y, false)),
                    ],
                );
                Case::TensorPar
            }
            Kind::One
            | Kind::Bot
            | Kind::Top
            | Kind::Zero
            | Kind::With
            | Kind::Plus
            | Kind::Bang
            | Kind::Quest => unreachable!("a proof structure is of unit-free MLL"),
        };
        within(self.vertices().count() - 2, options)?;
        afford(Self::bytes(forest), limits)?;
        let made = Self::with_cuts(forest.clone(), self.criterion(), &cuts, &links)?;
        let step = Step {
            cut: (x, y),
            case,
            above: None,
        };
        Ok((made, step))
    }
}
