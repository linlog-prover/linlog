// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! A session rebuilt from its parts: the derivation it held, replayed
//! against the rules, as its JSON form is read back.

use super::Interactive;
use super::quantified::{Parts, Quantified, prefix_max};
use crate::Error;
use crate::fragment::Mode;
use crate::hash::HashMap;
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
        limits: &crate::Limits,
    ) -> Result<Self, Error> {
        let mut state = Self::from_forest(forest, mode)?;
        let marks = parts.as_ref().map(|parts| parts.marks.clone());
        match (state.fo.is_some(), parts) {
            (true, Some(parts)) => {
                let q = Quantified::from_parts(&state.forest, parts, history.len(), limits.terms)?;
                state.fo = Some(Box::new(q));
            }
            (true, None) => {
                if let Some(q) = state.fo.as_mut() {
                    q.limit = limits.terms;
                }
            }
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
        // Per step, the inferences it added: `start..len`.
        let mut added_by: Vec<(usize, usize)> = Vec::with_capacity(history.len());
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
            added_by.push((start, len));
            len = start;
        }
        added_by.reverse();
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
            state.replay_binders(&added_by)?;
        }
        if let (Some(q), Some(marks)) = (state.fo.as_mut(), marks) {
            q.read_marks(&marks, state.history.len())?;
            // The members past the forest that the inferences before each
            // step name: what the step's mark must keep.
            let occurrences = state.forest.len();
            let named = prefix_max(state.inferences.iter().map(|inference| {
                inference
                    .sequent
                    .iter()
                    .map(|m| (m.index() + 1).saturating_sub(occurrences))
                    .max()
                    .unwrap_or(0)
            }));
            for (step, members) in q.marked_members() {
                if named[added_by[step as usize].0] > members {
                    return Err(Error::InconsistentSession {
                        reason: "a mark is out of order, past the table, or drops what a step before it made",
                    });
                }
            }
        }
        Ok(state)
    }

    /// Checks a first-order session's `∀` and `∃` steps: each `∀` step's
    /// eigenvariable no constant and introduced by no other, each open
    /// witness opened by one step only, each witness term given free of
    /// open witnesses and of eigenvariables outside its goal's scope.
    fn replay_binders(&self, added_by: &[(usize, usize)]) -> Result<(), Error> {
        let bad = |reason: &'static str| Error::InconsistentSession { reason };
        let q = self.fo.as_ref().expect("a first-order session");
        let n = self.inferences.len();
        // The step that closed each inference: its own, or the graft's
        // that added it.
        let mut step_of = vec![None; n];
        for (h, (&goal, &(start, end))) in self.history.iter().zip(added_by).enumerate() {
            step_of[goal.index()] = Some(h);
            for slot in &mut step_of[start..end] {
                *slot = Some(h);
            }
        }
        // Each inference's place in the tree: entered and left in a walk
        // from the root, so that an inference lies below another exactly
        // when its interval holds the other's.
        let (mut enter, mut exit) = (vec![0usize; n], vec![0usize; n]);
        let mut clock = 0;
        let mut walk = vec![(InfId::new(0), false)];
        while let Some((id, done)) = walk.pop() {
            clock += 1;
            if done {
                exit[id.index()] = clock;
            } else {
                enter[id.index()] = clock;
                walk.push((id, true));
                walk.extend(
                    self.inferences[id.index()]
                        .premises
                        .iter()
                        .map(|&p| (p, false)),
                );
            }
        }
        let (mut eigens, mut metas): (Vec<Eigen>, Vec<u32>) = (Vec::new(), Vec::new());
        let mut introduced: HashMap<Eigen, InfId> = HashMap::default();
        let mut opened: Vec<(u32, usize)> = Vec::new();
        // The smallest eigenvariable a `∀` step made at each step or later.
        let mut later = vec![u32::MAX; self.history.len() + 1];
        let mut witnessed = Vec::new();
        for (i, inference) in self.inferences.iter().enumerate() {
            let id = InfId::new(i as u32);
            let rule = inference.rule.rule;
            if !matches!(rule, Rule::Forall | Rule::Exists) {
                continue;
            }
            let t = self
                .opened_with(id)
                .ok_or(bad("a quantifier step opens no body"))?;
            let step = step_of[i].ok_or(bad("a closed inference no step closed"))?;
            let terms = q.terms(&self.forest);
            match (rule, terms.term(t)) {
                (Rule::Forall, FoTerm::Eigen(a)) if !q.constants.contains(&a) => {
                    eigens.push(a);
                    introduced.insert(a, id);
                    later[step] = later[step].min(a.get());
                }
                (Rule::Forall, _) => {
                    return Err(bad("a ∀ step's body is not opened with an eigenvariable"));
                }
                (_, FoTerm::Meta(k)) => {
                    metas.push(k);
                    opened.push((k, step));
                }
                _ => witnessed.push((id, t)),
            }
        }
        for h in (0..self.history.len()).rev() {
            later[h] = later[h].min(later[h + 1]);
        }
        // An open witness admits no eigenvariable made at its step or later.
        if opened
            .iter()
            .any(|&(k, step)| q.metas[k as usize].made > later[step])
        {
            return Err(bad(
                "an open witness's level admits an eigenvariable made after it",
            ));
        }
        for (id, t) in witnessed {
            let (inside, open) = q.variables(&self.forest, t);
            let in_scope = |a: &Eigen| {
                q.constants.contains(a)
                    || introduced.get(a).is_some_and(|&by| {
                        enter[by.index()] < enter[id.index()] && exit[id.index()] < exit[by.index()]
                    })
            };
            if !open.is_empty() || !inside.iter().all(in_scope) {
                return Err(bad(
                    "an ∃ step's witness names an open witness or an eigenvariable out of its scope",
                ));
            }
        }
        // A binding is made by an axiom, a `1` or `⊤` closing its goal, or a
        // graft of the search, never by a step that only opens goals.
        for (_, _, step) in q.bindings() {
            let h = step as usize;
            let goal = &self.inferences[self.history[h].index()];
            let (start, end) = added_by[h];
            // A graft added closed inferences; a rule's own premises are
            // open until a later step closes them.
            let closes = matches!(goal.rule.rule, Rule::Ax | Rule::One | Rule::Top)
                || (start..end)
                    .any(|i| step_of[i] == Some(h) && self.inferences[i].rule.rule != Rule::Open);
            if !closes {
                return Err(bad("a binding is made by a step that closes no goal"));
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
