// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The rules a first-order zone adds to the engine: the identities by
//! unification, the `∃` rule in focus, and the rules of two premises that
//! ask the first for another answer where the second fails under the
//! one it gave (`framed.rs` has the argument).

use super::choices::Site;
use super::framed::Unified;
use super::zone::{Linear as _, Unrestricted as _, Zone};
use super::{Cuts, FORCED_PER_POLL, Found, Run, Step, Unrestricted};
use crate::proofs::{Node, NodeId};
use crate::search::Reason;

/// How many terms the zone's walks visit for a unit of work: a visit is a
/// lookup in a table of the terms met and a few comparisons, some sixteenth
/// of what a stable sequent's unit stands for.
const TERMS_PER_WORK: u64 = 16;

impl<Z: Zone> Run<'_, Z> {
    /// The initial rules of a first-order zone on a stable sequent, or on
    /// a positive literal in focus with its context: an axiom on two
    /// opposite literals whose terms unify, in linear mode exactly those
    /// two, or on a literal and a copy of an opposite one of `Θ`, which
    /// counts against the budget; with anything else weakened in affine
    /// mode. Each pair is an alternative, tried in the members' order,
    /// the first that unifies taken: another binding is the answer the
    /// rule around asks for where it needs one.
    pub(super) fn initial_framed(
        &mut self,
        theta: &Unrestricted<Z>,
        gamma: &Z::Linear,
        members: &[Z::Member],
        budget: u32,
    ) -> Step {
        let affine = self.problem.rules.affine;
        if !affine && members.len() > 2 {
            return Ok(Found::NOTHING);
        }
        let f = self.problem.forest;
        // Every pair tried is an alternative of a choice point that logs
        // which one it tries (`choices.rs`).
        let choice = self.zone.choices().enter(Site::Initial);
        if choice.start == u32::MAX {
            self.zone.choices().fail(&choice, choice.cuts);
            return Ok(Found::failed(choice.cuts));
        }
        let mut cuts = choice.cuts;
        let mut tried = 0u32;
        // The next pair's number, `None` past the log's numbers, where the
        // rest are cut, never refuted.
        let mut next = |cuts: &mut Cuts| {
            let i = tried;
            match tried.checked_add(1).filter(|&n| n < u32::MAX) {
                Some(n) => {
                    tried = n;
                    Some(i)
                }
                None => {
                    *cuts = cuts.and(Cuts::BUDGET);
                    None
                }
            }
        };
        for (i, &p) in members.iter().enumerate() {
            if !f.is_literal(self.occ(p)) {
                continue;
            }
            if affine || members.len() == 2 {
                for &q in &members[i + 1..] {
                    if !self.zone.opposite(f, p, q) {
                        continue;
                    }
                    let Some(n) = next(&mut cuts) else {
                        break;
                    };
                    if n < choice.start {
                        continue;
                    }
                    self.zone.choices().take(&choice, n, cuts);
                    let mark = self.zone.mark();
                    let unified = self.zone.unify(f, p, q);
                    self.poll_terms()?;
                    match unified {
                        Unified::Yes => {
                            let ax = self.push(Node::Ax(p.into(), q.into()));
                            return Ok(Found::proved(self.leftovers(gamma, &[p, q], ax)));
                        }
                        Unified::No => self.zone.undo(mark),
                        Unified::Cut => {
                            self.zone.undo(mark);
                            cuts = cuts.and(Cuts::BUDGET);
                        }
                    }
                }
            }
            if !(affine || members.len() == 1) || !self.problem.rules.exponentials {
                continue;
            }
            for d in theta.set.iter() {
                if !self.zone.opposite(f, p, d) {
                    continue;
                }
                let Some(n) = next(&mut cuts) else {
                    break;
                };
                if n < choice.start {
                    continue;
                }
                if budget == 0 {
                    // Another pair may still close the sequent without a
                    // copy.
                    cuts = cuts.and(Cuts::BUDGET);
                    continue;
                }
                self.zone.choices().take(&choice, n, cuts);
                let mark = self.zone.mark();
                let unified = self.zone.unify(f, p, d);
                self.poll_terms()?;
                match unified {
                    Unified::Yes => {
                        let ax = self.push(Node::Ax(p.into(), d.into()));
                        let copy = self.push(Node::Copy(d.into(), ax));
                        return Ok(Found::proved(self.leftovers(gamma, &[p], copy)));
                    }
                    Unified::No => self.zone.undo(mark),
                    Unified::Cut => {
                        self.zone.undo(mark);
                        cuts = cuts.and(Cuts::BUDGET);
                    }
                }
            }
        }
        self.zone.choices().fail(&choice, cuts);
        Ok(Found::failed(cuts))
    }

    /// Charges what the zone's tables and buffers grew or shrank by to the
    /// account, which the next stable sequent asks, and counts the terms
    /// the zone's walks visited as work, a unit per
    /// [`TERMS_PER_WORK`], and polls the stop condition once every
    /// [`FORCED_PER_POLL`] of them: a unification of deep terms is work
    /// that no step of the search counts, and a split search polls only
    /// every [`SPLITS_PER_POLL`](super::SPLITS_PER_POLL) of its steps.
    fn poll_terms(&mut self) -> Result<(), Reason> {
        let (before, after) = self.zone.held();
        self.scratch.resize(before, after);
        let steps = self.zone.take_steps();
        self.work += steps / TERMS_PER_WORK;
        self.forced = self.forced.saturating_add(steps);
        if self.forced < FORCED_PER_POLL {
            return Ok(());
        }
        self.forced = 0;
        if self.stop.fired(std::mem::take(&mut self.work)) {
            Err(Reason::Stopped)
        } else {
            Ok(())
        }
    }

    /// The proof of a leaf that used the members `used` of `gamma`:
    /// itself in linear mode, where they are all of it, and with the
    /// others weakened below it in affine mode.
    fn leftovers(&mut self, gamma: &Z::Linear, used: &[Z::Member], leaf: NodeId) -> NodeId {
        if !self.problem.rules.affine {
            return leaf;
        }
        let mut rest = self.take_context_from(gamma);
        for &m in used {
            if rest.contains(m) {
                rest.remove(m);
            }
        }
        let node = self.weakened(&rest, leaf);
        self.give_context(rest);
        node
    }

    /// The `∃` rule in focus on `f`: its body under a fresh metavariable,
    /// still in focus, which the identities above bind.
    pub(super) fn exists(
        &mut self,
        theta: &Unrestricted<Z>,
        gamma: &Z::Linear,
        f: Z::Member,
        budget: u32,
    ) -> Step {
        let mark = self.zone.mark();
        let Some((body, witness)) = self.zone.exists(self.problem.forest, f)? else {
            return Ok(Found::failed(Cuts::BUDGET));
        };
        match self.focus(theta, gamma, body, budget)? {
            Found::Proved(node) => Ok(Found::proved(self.push(Node::Exists(
                f.into(),
                witness,
                node,
            )))),
            failed => {
                self.zone.undo(mark);
                Ok(failed)
            }
        }
    }

    /// The `&` rule of a first-order zone: both premises on copies of the
    /// state, the left one asked for another answer where the right one
    /// fails under the one it gave.
    pub(super) fn with_framed(
        &mut self,
        theta: &Unrestricted<Z>,
        gamma: &Z::Linear,
        list: &[Z::Member],
        o: Z::Member,
        budget: u32,
    ) -> Step {
        let (a, b) = (self.left(o), self.right(o));
        let mut shared = gamma
            .iter()
            .chain(list.iter().copied())
            .chain([b])
            .chain(theta.set.iter());
        self.answers(
            &mut shared,
            |run| run.premise(theta, gamma, list, a, budget),
            |run| run.premise(theta, gamma, list, b, budget),
            |l, r| Node::With(o.into(), l, r),
        )
    }

    /// A rule of two premises in a first-order zone: `first` searches the
    /// first, `second` the second once the first is proved, `join` makes
    /// the rule's node. Where the second premise, whose members `shared`
    /// lists, has an unbound metavariable, a failure of the second asks
    /// the first for another answer: the one it gave is excluded, and the
    /// first premise's search is resumed at its last choice point
    /// (`choices.rs`), or searched again from the bindings it started
    /// from where the replay diverged, until the second is proved, the
    /// first fails, or every answer left is excluded. The rule's rounds
    /// are a choice point of their own, so that a replay of a search
    /// around it restores what its rounds before cut. The failure rests
    /// on the cuts of every search run and every one given up.
    pub(super) fn answers(
        &mut self,
        shared: &mut dyn Iterator<Item = Z::Member>,
        mut first: impl FnMut(&mut Self) -> Step,
        mut second: impl FnMut(&mut Self) -> Step,
        join: impl FnOnce(NodeId, NodeId) -> Node,
    ) -> Step {
        let forest = self.problem.forest;
        let choice = self.zone.choices().enter(Site::Answers);
        if choice.start == u32::MAX || choice.resumed {
            // Failed before, or resumed here: its premises took no
            // alternative after its last round, so no answer is left.
            self.zone.choices().fail(&choice, choice.cuts);
            return Ok(Found::failed(choice.cuts));
        }
        let mark = self.nodes.mark();
        let zone = self.zone.mark();
        let open = self.zone.open_answers(forest, shared);
        if let Err(reason) = self.poll_terms() {
            if open {
                self.zone.close_answers();
            }
            return Err(reason);
        }
        if open {
            self.zone.choices().open();
        }
        let mut cuts = choice.cuts;
        // The bytes of the answers recorded, charged while they are
        // excluded.
        let mut recorded = 0;
        // Which round this is, and whether it resumes the first premise.
        let (mut round, mut resuming) = (choice.start, false);
        self.zone.choices().take(&choice, round, cuts);
        let found = loop {
            let start = self.zone.choices().len();
            let mut l = self.first_premise(open, &mut first);
            if resuming && self.zone.choices().end_replay() {
                // The replay diverged: the first premise is searched again
                // with every failed answer excluded.
                self.nodes.release(mark);
                self.zone.undo(zone);
                self.zone.choices().truncate(start);
                l = self.first_premise(open, &mut first);
            }
            let l_node = match l {
                Ok(Found::Proved(node)) => node,
                Ok(Found::Failed(failed)) => break Ok(Found::failed(cuts.and(failed))),
                Err(reason) => break Err(reason),
            };
            let end = self.zone.choices().len();
            let (l_node, r) = self.holding(l_node, &mut second);
            match r {
                Ok(Found::Proved(r_node)) => {
                    break Ok(Found::proved(self.push(join(l_node, r_node))));
                }
                Ok(Found::Failed(failed)) => cuts = cuts.and(failed),
                Err(reason) => break Err(reason),
            }
            if open {
                let bytes = self.zone.record(forest);
                self.scratch.charge(bytes);
                recorded += bytes;
            }
            self.nodes.release(mark);
            self.zone.undo(zone);
            if !open || self.zone.exhausted(forest) {
                break Ok(Found::failed(cuts));
            }
            if let Err(reason) = self.poll_terms() {
                break Err(reason);
            }
            // The rounds are fewer than the answers recorded, each held
            // in memory; past the log's numbers the rule is cut.
            let Some(next) = round.checked_add(1).filter(|&n| n < u32::MAX) else {
                break Ok(Found::failed(cuts.and(Cuts::BUDGET)));
            };
            round = next;
            self.zone.choices().next_round(&choice, round, cuts);
            if let Err(given_up) = self.zone.choices().resume(start, end) {
                // The first premise took no alternative: its answer was
                // its only one.
                break Ok(Found::failed(cuts.and(given_up)));
            }
            resuming = true;
        };
        if open {
            self.zone.choices().close();
            self.zone.close_answers();
            self.scratch.resize(recorded, 0);
        }
        if let Ok(Found::Failed(cuts)) = found {
            self.zone.choices().fail(&choice, cuts);
        }
        found
    }

    /// Searches a rule's first premise, with its excluded answers read
    /// while it runs where the rule has any.
    fn first_premise(&mut self, open: bool, first: &mut impl FnMut(&mut Self) -> Step) -> Step {
        if open {
            self.zone.searching_first(true);
        }
        let l = first(self);
        if open {
            self.zone.searching_first(false);
        }
        l
    }
}
