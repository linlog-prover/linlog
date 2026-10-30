// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The memo and the loop check of a first-order search, which key a
//! stable sequent by its instance (`framed/key.rs` has the keys).
//!
//! Only failures are recorded: a proof's members name spans and
//! metavariables that an undo of a rule around it takes back, so a proof
//! kept for later would name what is gone. A failure is a fact about its
//! sequent where its search ran with no answer excluded (an excluded
//! answer may be all that failed it), rested on no prune of an ancestor
//! (the dependency the propositional engine lets wait is dropped here),
//! and bound only what the sequent's own terms reach: its metavariables
//! and the ones its search made. A complete failure then holds for every
//! sequent with its canonical key; one cut by a bound only for the
//! instance itself, at its own budget, as the propositional engine keeps
//! such failures under the sequent's own key so that deepening closes.

use super::memo::{Entry, Failure, Key, Zones};
use super::zone::{Linear as _, Zone};
use super::{Cuts, Found, NO_DEPENDENCY, Run, Unrestricted};
use crate::occurrences::OccId;
use crate::search::Reason;

impl<Z: Zone> Run<'_, Z> {
    /// The decision of a first-order stable sequent under its keys: a
    /// complete failure recorded for its canonical key or one cut at its
    /// own key answers it, a repeat of an ancestor is pruned, else it is
    /// decided and a failure recorded as the module says. `None` where it
    /// has no keys.
    pub(super) fn prove_keyed(
        &mut self,
        theta: &Unrestricted<Z>,
        gamma: &Z::Linear,
        budget: u32,
    ) -> Option<Result<(Found, bool), Reason>> {
        let mut own = self.take_words();
        let mut canonical = self.take_words();
        let forest = self.problem.forest;
        let decided = self
            .zone
            .keys(forest, &theta.set, gamma, &mut own, &mut canonical)
            .then(|| self.prove_by_keys(theta, gamma, (&own, &canonical), budget));
        self.give_words(own);
        self.give_words(canonical);
        decided
    }

    /// [`Self::prove_keyed`] with the keys written.
    fn prove_by_keys(
        &mut self,
        theta: &Unrestricted<Z>,
        gamma: &Z::Linear,
        (own, canonical): (&[OccId], &[OccId]),
        budget: u32,
    ) -> Result<(Found, bool), Reason> {
        // The unrestricted zone is among the members' words.
        let own = Zones {
            theta: 0,
            members: own,
        };
        let canonical = Zones {
            theta: 0,
            members: canonical,
        };
        let (own_hash, canonical_hash) = (own.hash(), canonical.hash());
        if let Some(failure) = self.memo.refuted(canonical, canonical_hash) {
            return Ok((Found::NOTHING, failure == Failure::Hereditary));
        }
        // Only failures cut by a bound lie under the own key.
        let cut = self.memo.get(own, own_hash, budget).is_some();
        if self.problem.rules.stack
            && let Some(depth) = self.repeated(own, own_hash)
        {
            // A depth of the branch, which is as deep as the recursion
            // limit allows.
            return Ok((Found::failed(Cuts::repeat(depth as u32)), false));
        }
        if cut {
            return Ok((Found::failed(Cuts::BUDGET), false));
        }
        if self.problem.rules.stack {
            self.push_stack(own, own_hash);
        }
        let nodes = self.nodes.mark();
        let zone = self.zone.mark();
        let mut members = self.take_list();
        gamma.list_into(&mut members);
        let result = self.decide(theta, gamma, &members, budget);
        self.give_list(members);
        let own_depth = if self.problem.rules.stack {
            self.stack_len -= 1;
            (self.above() + self.stack_len) as u32
        } else {
            NO_DEPENDENCY
        };
        let (found, hereditary) = result?;
        let Found::Failed(cuts) = found else {
            return Ok((found, false));
        };
        self.nodes.release(nodes);
        self.zone.undo(zone);
        // A prune of this very sequent is settled with it.
        let dependency = if cuts.dependency >= own_depth {
            NO_DEPENDENCY
        } else {
            cuts.dependency
        };
        if dependency == NO_DEPENDENCY && !self.zone.excluding() {
            if cuts.exhausted {
                self.remember(own, own_hash, Entry::Failed(Failure::Exhausted(budget)))?;
            } else {
                let complete = if hereditary {
                    Failure::Hereditary
                } else {
                    Failure::Complete
                };
                self.remember(canonical, canonical_hash, Entry::Failed(complete))?;
            }
        }
        let cuts = Cuts {
            exhausted: cuts.exhausted,
            dependency,
        };
        Ok((Found::failed(cuts), hereditary))
    }

    /// Pushes a stable sequent on the branch stack, as the propositional
    /// engine does, into the entry's own buffer.
    fn push_stack(&mut self, key: Zones<'_>, hash: u64) {
        if self.stack_len == self.stack.len() {
            self.scratch
                .charge(size_of::<Key>() + size_of::<u64>() + size_of::<usize>());
            self.stack.push(Key::default());
            self.hashes.push(0);
            self.marks.push(0);
        }
        let pushed = &mut self.stack[self.stack_len];
        pushed.assign(key);
        pushed.members.settle(&mut self.scratch);
        self.hashes[self.stack_len] = hash;
        self.marks[self.stack_len] = self.waiting.len();
        self.stack_len += 1;
    }
}
