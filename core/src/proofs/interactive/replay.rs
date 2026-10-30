// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! A session rebuilt from its parts: the derivation it held, replayed
//! against the rules, as its JSON form is read back.

use super::Interactive;
use super::quantified::{Parts, Quantified};
use crate::Error;
use crate::fragment::Mode;
use crate::occurrences::{Forest, Member, OccId, Reading};
use crate::proofs::Rule;
use crate::proofs::derivation::{InfId, Inference};
use crate::proofs::multiset::Multiset;
use crate::sequents::{Eigen, FoTerm};

impl Interactive {
    /// Rebuilds a state from its parts, as deserialization does, checking
    /// that they are consistent: inference 0 concludes the sequent, every
    /// other inference is the premise of exactly one earlier inference,
    /// every closed inference's premises are what its rule on its sequent
    /// yields, and the history names distinct closed inferences whose
    /// premises were, in order, the last inferences added. A sequent with
    /// quantifiers has its first-order part (`parts`): its table read and
    /// its bindings replayed, every `∀` step's eigenvariable fresh and
    /// introduced once, every `∃` step's witness an open witness of its
    /// own or a term of the eigenvariables in its goal's scope, every
    /// axiom's literals equal instances under the bindings.
    pub(crate) fn from_parts(
        forest: Forest,
        mode: Mode,
        inferences: Vec<Inference>,
        history: Vec<InfId>,
        parts: Option<Parts>,
    ) -> Result<Self, Error> {
        let mut state = Self::from_forest(forest, mode)?;
        match (state.fo.is_some(), parts) {
            (true, Some(parts)) => {
                let q = Quantified::from_parts(&state.forest, parts, history.len())?;
                state.fo = Some(Box::new(q));
            }
            (true, None) => {}
            (false, Some(_)) => {
                return Err(Error::InconsistentSession {
                    reason: "a session of a sequent without quantifiers has instances",
                });
            }
            (false, None) => {}
        }
        let members = state.forest.len() + state.fo.as_ref().map_or(0, |q| q.table.len());
        let depth = crate::occurrences::instances::binders(&state.forest);
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
                if o.index() >= members {
                    return Err(Error::IndexOutOfBounds {
                        space: crate::limits::Space::Member,
                        index: o.index(),
                        len: members,
                    });
                }
                if o.index() < state.forest.len() && depth[o.index()] > 0 {
                    return Err(Error::InconsistentSession {
                        reason: "an occurrence under quantifiers stands without its instance",
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
                if state.fo.is_some() {
                    return Err(Error::InconsistentSession {
                        reason: "a session with quantifiers makes no cut",
                    });
                }
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
        if state.fo.is_some() {
            state.replay_binders()?;
        }
        Ok(state)
    }

    /// Checks a first-order session's `∀` and `∃` steps: each `∀` step's
    /// eigenvariable no constant and introduced by no other, each open
    /// witness opened by one step only, each witness term given free of
    /// open witnesses and of eigenvariables outside its goal's scope.
    fn replay_binders(&self) -> Result<(), Error> {
        let bad = |reason: &'static str| Error::InconsistentSession { reason };
        let q = self.fo.as_ref().expect("a first-order session");
        let (mut eigens, mut metas): (Vec<Eigen>, Vec<u32>) = (Vec::new(), Vec::new());
        for (i, inference) in self.inferences.iter().enumerate() {
            let id = InfId::new(i as u32);
            let rule = inference.rule.rule;
            if !matches!(rule, Rule::Forall | Rule::Exists) {
                continue;
            }
            let t = self
                .opened_with(id)
                .ok_or(bad("a quantifier step opens no body"))?;
            let terms = q.terms(&self.forest);
            match (rule, terms.term(t)) {
                (Rule::Forall, FoTerm::Eigen(a)) if !q.constants.contains(&a) => eigens.push(a),
                (Rule::Forall, _) => {
                    return Err(bad("a ∀ step's body is not opened with an eigenvariable"));
                }
                (_, FoTerm::Meta(k)) => metas.push(k),
                _ => {
                    let (inside, open) = q.variables(&self.forest, t);
                    let scope = self.scope(id);
                    if !open.is_empty() || !inside.iter().all(|a| scope.contains(a)) {
                        return Err(bad(
                            "an ∃ step's witness names an open witness or an eigenvariable out of its scope",
                        ));
                    }
                }
            }
        }
        eigens.sort_unstable();
        metas.sort_unstable();
        if eigens.windows(2).any(|w| w[0] == w[1]) || metas.windows(2).any(|w| w[0] == w[1]) {
            return Err(bad(
                "an eigenvariable or an open witness is opened by two steps",
            ));
        }
        Ok(())
    }

    /// Checks that a closed inference names a principal formula exactly
    /// when its rule has one and that its premises are what the rule
    /// yields on its sequent, recovering the split of a `⊗` or Mix from
    /// the left premise.
    pub(super) fn replay(&self, reading: Option<&Reading>, id: InfId) -> Result<(), Error> {
        let inference = &self.inferences[id.index()];
        let members = &inference.sequent;
        let sequent: &[OccId] = &members
            .iter()
            .map(|&m| self.occurrence(m))
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
            let mut context = Multiset::of(self.inferences[first.index()].sequent.iter().copied());
            if let Some(p) = principal {
                let left = self.forest.left(sequent[p]).unwrap_or(sequent[p]);
                if let Some(q) = &self.fo {
                    // An instance the session never made is in no premise.
                    if let Some(m) = q.find(left, q.instance(members[p]).1) {
                        context.remove(m);
                    }
                } else {
                    context.remove(Member::from(left));
                }
            }
            // Both are sorted, so each member of the context is the next
            // equal one of the sequent, the principal formula skipped.
            let mut at = 0;
            for &o in context.as_slice() {
                while at < members.len() && (members[at] < o || Some(at) == principal) {
                    at += 1;
                }
                if at == members.len() || members[at] != o {
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
        if self.fo.is_some() {
            return self.replay_framed(id, position, &left);
        }
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

    /// Checks a first-order session's closed inference, its checks on
    /// occurrences passed: an axiom's literals equal instances under the
    /// bindings, every other rule's premises the members it yields, a
    /// binder's body opened under the principal formula's frame.
    fn replay_framed(&self, id: InfId, position: usize, left: &[usize]) -> Result<(), Error> {
        let bad = |reason: &'static str| Error::InconsistentSession { reason };
        let q = self.fo.as_ref().expect("a first-order session");
        let inference = &self.inferences[id.index()];
        let members = &inference.sequent;
        let rule = inference.rule.rule;
        if rule == Rule::Ax {
            let [x, y] = members[..] else {
                unreachable!("the occurrences' check holds two literals");
            };
            return if q.same_atom(&self.forest, x, y) {
                Ok(())
            } else {
                Err(bad("an axiom's literals are not equal instances"))
            };
        }
        let body = if matches!(rule, Rule::Forall | Rule::Exists) {
            let body = self
                .body(id)
                .ok_or(bad("a quantifier step opens no body"))?;
            let (o, frame) = q.instance(members[position]);
            let opened = q.instance(body);
            let parent = opened.1.map(|f| q.table.frame_parts(f).0);
            if Some(opened.0) != self.forest.left(o) || parent != Some(frame) {
                return Err(bad(
                    "a quantifier step's body is not its binder's under its frame",
                ));
            }
            Some(body)
        } else {
            None
        };
        let expected = self.mirror(members, position, rule, left, body);
        let same = expected.is_some_and(|expected| {
            expected.len() == inference.premises.len()
                && expected
                    .iter()
                    .zip(&inference.premises)
                    .all(|(e, p)| *e == self.inferences[p.index()].sequent)
        });
        if same {
            Ok(())
        } else {
            Err(bad("an inference's premises are not what its rule yields"))
        }
    }
}
