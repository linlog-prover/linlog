// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! A session rebuilt from its parts: the derivation it held, replayed
//! against the rules, as its JSON form is read back.

use super::Interactive;
use crate::Error;
use crate::fragment::Mode;
use crate::occurrences::{Forest, Member, OccId, Reading};
use crate::proofs::Rule;
use crate::proofs::derivation::{InfId, Inference};
use crate::proofs::multiset::Multiset;

impl Interactive {
    /// Rebuilds a state from its parts, as deserialization does, checking
    /// that they are consistent: inference 0 concludes the sequent, every
    /// other inference is the premise of exactly one earlier inference,
    /// every closed inference's premises are what its rule on its sequent
    /// yields, and the history names distinct closed inferences whose
    /// premises were, in order, the last inferences added.
    pub(crate) fn from_parts(
        forest: Forest,
        mode: Mode,
        inferences: Vec<Inference>,
        history: Vec<InfId>,
    ) -> Result<Self, Error> {
        let mut state = Self::from_forest(forest, mode)?;
        let Some(root) = inferences.first() else {
            return Err(Error::InconsistentSession {
                reason: "there is no inference",
            });
        };
        if !root
            .sequent
            .iter()
            .map(|m| m.occ())
            .eq(state.forest.roots().iter().copied())
        {
            return Err(Error::InconsistentSession {
                reason: "inference 0 does not conclude the sequent",
            });
        }
        let n = inferences.len();
        let mut parent = vec![None; n];
        for (i, inference) in inferences.iter().enumerate() {
            for &o in &inference.sequent {
                if o.index() >= state.forest.len() {
                    return Err(Error::IndexOutOfBounds {
                        space: crate::limits::Space::Occurrence,
                        index: o.index(),
                        len: state.forest.len(),
                    });
                }
            }
            if !inference.sequent.is_sorted() {
                return Err(Error::InconsistentSession {
                    reason: "a sequent is not in ascending order",
                });
            }
            for &p in &inference.premises {
                if p.index() <= i || p.index() >= n || parent[p.index()].is_some() {
                    return Err(Error::InconsistentSession {
                        reason: "a premise must be a later inference and the premise of one inference only",
                    });
                }
                parent[p.index()] = Some(i);
            }
        }
        if parent.iter().skip(1).any(Option::is_none) {
            return Err(Error::InconsistentSession {
                reason: "an inference other than the root is nobody's premise",
            });
        }
        state.inferences = inferences;
        let reading = state.reading();
        for id in 0..n {
            let inference = &state.inferences[id];
            if inference.rule.rule == Rule::Open {
                if inference.principal.is_some() || !inference.premises.is_empty() {
                    return Err(Error::InconsistentSession {
                        reason: "an open goal has no principal formula and no premise",
                    });
                }
                continue;
            }
            if (inference.rule.rule == Rule::Cut) != inference.cut.is_some() {
                return Err(Error::InconsistentSession {
                    reason: "a cut names its cut formula, every other rule none",
                });
            }
            if inference.rule.rule == Rule::Cut {
                state.replay_cut(InfId::new(id as u32))?;
            } else {
                state.replay(reading.as_ref(), InfId::new(id as u32))?;
            }
        }
        // Seen from the last step back, the inferences a step added are
        // those of its subtree that exist at that point, and they are the
        // suffix of the arena then.
        let mut len = n;
        let mut seen = vec![false; n];
        for &goal in history.iter().rev() {
            // Below `len`, the arena as this step left it: an entry inside
            // a later step's subtree would make `undo` index past it.
            let closed = goal.index() < len
                && state.inferences[goal.index()].rule.rule != Rule::Open
                && !std::mem::replace(&mut seen[goal.index()], true);
            if !closed {
                return Err(Error::InconsistentSession {
                    reason: "the history names an inference that is not a closed one, or names one twice",
                });
            }
            // The inferences this step added: those of its subtree below
            // `len`. A premise has a larger index than its conclusion, so
            // the walk enters no inference a later step added, and every
            // inference is walked by one step only.
            let (mut added, mut lowest) = (0, len);
            let mut walk = state.inferences[goal.index()].premises.clone();
            while let Some(id) = walk.pop() {
                if id.index() >= len {
                    continue;
                }
                added += 1;
                lowest = lowest.min(id.index());
                walk.extend(state.inferences[id.index()].premises.iter().copied());
            }
            // Each inference is the premise of one other only, so the walk
            // counts each once: `added` of them below `len` and none below
            // `start` is exactly the suffix `start..len`.
            let start = len - added;
            if lowest < start {
                return Err(Error::InconsistentSession {
                    reason: "the history does not match the order of the inferences",
                });
            }
            len = start;
        }
        // The cut formulas are those of the cuts, in the order they were
        // made, so that `undo` takes back the last one with its cut.
        let made = state
            .history
            .iter()
            .chain(&history)
            .filter_map(|id| state.inferences[id.index()].cut);
        if !made
            .map(Member::occ)
            .eq(state.forest.cut_pairs().map(|(a, _)| a))
        {
            return Err(Error::InconsistentSession {
                reason: "the cut formulas are not those of the cuts in the order made",
            });
        }
        state.history = history;
        Ok(state)
    }

    /// Checks that a closed inference names a principal formula exactly
    /// when its rule has one and that its premises are what the rule
    /// yields on its sequent, recovering the split of a `⊗` or Mix from
    /// the left premise.
    pub(super) fn replay(&self, reading: Option<&Reading>, id: InfId) -> Result<(), Error> {
        let inference = &self.inferences[id.index()];
        let sequent: &[OccId] = &inference
            .sequent
            .iter()
            .map(|m| m.occ())
            .collect::<Vec<_>>();
        let rule = inference.rule;
        let classical = rule.rule;
        let has_principal = classical.has_principal();
        if inference.principal.is_some() != has_principal {
            return Err(Error::InconsistentSession {
                reason: "a rule other than the axiom and Mix names its principal formula, those two none",
            });
        }
        let principal = inference.principal;
        if principal.is_some_and(|p| p >= sequent.len()) {
            return Err(Error::InconsistentSession {
                reason: "a principal position is out of range",
            });
        }
        // The formulas the left premise took from the context, as positions
        // in the sequent.
        let mut left = Vec::new();
        if matches!(classical, Rule::Tensor | Rule::Mix) {
            let Some(&first) = inference.premises.first() else {
                return Err(Error::InconsistentSession {
                    reason: "a split has no premise",
                });
            };
            let mut context = Multiset::of(
                self.inferences[first.index()]
                    .sequent
                    .iter()
                    .map(|m| m.occ()),
            );
            if let Some(p) = principal {
                context.remove(self.forest.left(sequent[p]).unwrap_or(sequent[p]));
            }
            // Both are sorted, so each member of the context is the next
            // equal one of the sequent, the principal formula skipped.
            let mut at = 0;
            for &o in context.as_slice() {
                while at < sequent.len() && (sequent[at] < o || Some(at) == principal) {
                    at += 1;
                }
                if at == sequent.len() || sequent[at] != o {
                    return Err(Error::InconsistentSession {
                        reason: "a premise holds a formula its conclusion lacks",
                    });
                }
                left.push(at);
                at += 1;
            }
        }
        let position = match principal {
            Some(p) => p,
            // Mix: the first formula of the left premise stands for the
            // position; the axiom has position 0, its sequent being the
            // two literals.
            None if classical == Rule::Mix => {
                if left.is_empty() {
                    return Err(Error::InconsistentSession {
                        reason: "a Mix has an empty left premise",
                    });
                }
                left.remove(0)
            }
            None => 0,
        };
        let expected = self.expand(reading, sequent, position, rule, &left)?;
        let same = expected.len() == inference.premises.len()
            && expected.iter().zip(&inference.premises).all(|(e, p)| {
                e.as_slice()
                    .iter()
                    .copied()
                    .map(Member::from)
                    .eq(self.inferences[p.index()].sequent.iter().copied())
            });
        if same {
            Ok(())
        } else {
            Err(Error::InconsistentSession {
                reason: "an inference's premises are not what its rule yields",
            })
        }
    }
}
