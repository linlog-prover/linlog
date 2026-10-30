// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The `⊗` rule and Mix: the splits a factor forces, in a loop along a
//! chain of them, and the search for the free splits whose two sides
//! pass the counts.

use super::choices::Site;
use super::counts::{Split, Tally};
use super::scratch::Pooled;
use super::zone::{Cursors, Linear as _, Unrestricted as _, Zone};
use super::{
    Cuts, FORCED_PER_POLL, Found, OCCURRENCES_PER_LEAF, Run, SPLITS_PER_POLL, Searched, Step,
    Unrestricted,
};
use crate::occurrences::{OccId, Side};
use crate::proofs::{Branch, Node, NodeId};
use crate::search::Reason;
use crate::sequents::Kind;

impl<Z: Zone> Run<'_, Z> {
    /// What a factor of a `⊗` forces on its side of the split
    /// ([`Counts::forced`](super::counts::Counts::forced)). With weakening
    /// nothing but `0` forces anything, since every leaf takes any context.
    pub(super) fn forced_side(&self, factor: OccId) -> Option<Forced> {
        let forced = self.problem.counts.forced(factor);
        if self.problem.rules.affine && forced != Some(Forced::Nothing) {
            None
        } else {
            forced
        }
    }

    /// The factor of `F = A ⊗ B` that forces its split, with what it
    /// forces, the other factor, and which of the two the forcing one is.
    /// When both force, one that is closed in place (a literal, a unit)
    /// before a tensor of literals, which takes a focus of its own: a
    /// tensor of a thousand literals, nested to the left as it is read,
    /// must not pay a level of recursion per link; otherwise the left one.
    fn forced_factor(&self, f: Z::Member) -> Option<Forcing<Z::Member>> {
        let (a, b) = (self.left(f), self.right(f));
        let left = |forced| {
            Some(Forcing {
                forced,
                factor: a,
                other: b,
                side: Branch::Left,
            })
        };
        let right = |forced| {
            Some(Forcing {
                forced,
                factor: b,
                other: a,
                side: Branch::Right,
            })
        };
        match (self.forced_side(self.occ(a)), self.forced_side(self.occ(b))) {
            (Some(x), _) if x != Forced::Duals => left(x),
            (_, Some(y)) if y != Forced::Duals => right(y),
            (Some(x), _) => left(x),
            (None, Some(y)) => right(y),
            (None, None) => None,
        }
    }

    /// The `⊗` rule on `F = A ⊗ B` with context `Γ`: the forced split when
    /// a factor allows only one, else a search over the members of `Γ` for
    /// the splits whose two sides pass the counts.
    pub(super) fn split(
        &mut self,
        theta: &Unrestricted<Z>,
        gamma: &Z::Linear,
        f: Z::Member,
        budget: u32,
    ) -> Step {
        let Some(forcing) = self.forced_factor(f) else {
            return self.free_split(theta, gamma, f, budget);
        };
        let mark = self.nodes.mark();
        // A `!` factor's promotion may open binders even where no atom
        // is open, which a failure takes back with its nodes.
        let zone = Z::FIRST_ORDER.then(|| self.zone.mark());
        let mut rest = self.take_context_from(gamma);
        let mut links = self.take_links();
        let mut cursors = self.take_cursors();
        let result = self.forced_splits(
            theta,
            &mut rest,
            (f, forcing),
            (&mut links, &mut cursors),
            budget,
        );
        self.give_cursors(cursors);
        self.give_context(rest);
        let result = match result {
            Ok(Found::Proved(mut node)) => {
                for &(f, x_node, x_side) in links.iter().rev() {
                    let (left, right) = if x_side == Branch::Left {
                        (x_node, node)
                    } else {
                        (node, x_node)
                    };
                    node = self.push(Node::Tensor(f.into(), left, right));
                }
                Ok(Found::proved(node))
            }
            Ok(failed) => {
                self.nodes.release(mark);
                if let Some(zone) = zone {
                    self.zone.undo(zone);
                }
                Ok(failed)
            }
            Err(reason) => Err(reason),
        };
        self.give_links(links);
        result
    }

    /// The forced splits of `F`, given with its forcing factor as
    /// [`forced_factor`](Self::forced_factor) found it, and, as long as
    /// the factor they leave is a `⊗` with a forced split again, of that
    /// factor, in a loop: a tensor of a thousand literals costs one level
    /// of recursion and one copy of the context, not a thousand. `rest`
    /// is the context, from which every forcing factor takes its own;
    /// `links` gets every `⊗` with the proof of its forcing factor and
    /// whether that is the left one, and `cursors` remembers where the
    /// chain's lookups of duals ended. Returns the proof of the factor
    /// left at the end with what remains of the context. The chain visits
    /// no stable sequent, so it polls the stop condition itself, once
    /// every [`FORCED_PER_POLL`] splits and literals closed.
    fn forced_splits(
        &mut self,
        theta: &Unrestricted<Z>,
        rest: &mut Z::Linear,
        (mut f, first): (Z::Member, Forcing<Z::Member>),
        (links, cursors): (&mut Vec<Link<Z::Member>>, &mut Cursors),
        budget: u32,
    ) -> Step {
        // What the focuses on forcing factors along the chain cut.
        let mut cuts = Cuts::NONE;
        let mut forcing = Some(first);
        loop {
            let Some(Forcing {
                forced,
                factor: x,
                other: y,
                side: x_side,
            }) = forcing
            else {
                return Ok(self.focus(theta, rest, f, budget)?.after(cuts));
            };
            self.statistics.splits += 1;
            self.poll_forced()?;
            let x_node = match forced {
                Forced::Nothing => return Ok(Found::failed(cuts)),
                Forced::Empty => {
                    let empty = self.take_context();
                    let found = self.focus(theta, &empty, x, budget);
                    self.give_context(empty);
                    match found? {
                        Found::Proved(node) => Some(node),
                        Found::Failed(failed) => {
                            cuts = cuts.and(failed);
                            None
                        }
                    }
                }
                // The dual in `Γ` when there is one, and the axiom on the
                // two; otherwise the side stays empty and the initial rule
                // looks in `Θ`, at the price of a copy. Taking the copy
                // from `Γ` first loses nothing: the copies are the same
                // formula, so a proof that leaves this one for elsewhere
                // and copies from `Θ` here is a proof with the roles
                // swapped.
                Forced::Dual => {
                    let forest = self.problem.forest;
                    if let Some(dual) = self.zone.dual_from(forest, x, rest, cursors) {
                        rest.remove(dual);
                        Some(self.push(Node::Ax(x.into(), dual.into())))
                    } else if let Some(d) = self.zone.dual_in(forest, x, &theta.set) {
                        if budget == 0 {
                            cuts = cuts.and(Cuts::BUDGET);
                            None
                        } else {
                            let ax = self.push(Node::Ax(x.into(), d.into()));
                            Some(self.push(Node::Copy(d.into(), ax)))
                        }
                    } else {
                        None
                    }
                }
                // One dual per literal, each the first left in `Γ`: every
                // literal is proved by exactly its dual, and no dual lies in
                // `Θ`, so no other context proves the factor.
                Forced::Duals => self.literal_tensor(x, rest, cursors)?,
            };
            let Some(x_node) = x_node else {
                return Ok(Found::failed(cuts));
            };
            links.push((f, x_node, x_side));
            if self.kind(y) != Kind::Tensor {
                return Ok(self.focus(theta, rest, y, budget)?.after(cuts));
            }
            f = y;
            forcing = self.forced_factor(f);
        }
    }

    /// The proof of a tensor of positive literals from one dual per
    /// literal, each the first left in `rest`, which loses them; `None`
    /// when a dual is missing. Built in place, the axioms and the `⊗`
    /// nodes from the last occurrence back, so a tensor of any depth costs
    /// no recursion; the stop condition is polled as in the chain around
    /// it.
    fn literal_tensor(
        &mut self,
        x: Z::Member,
        rest: &mut Z::Linear,
        cursors: &mut Cursors,
    ) -> Searched {
        let forest = self.problem.forest;
        let mut duals = self.take_list();
        for leaf in forest.subtree(self.occ(x)) {
            if !forest.is_literal(leaf) {
                continue;
            }
            self.poll_forced()?;
            let leaf = self.zone.child(x, leaf);
            let Some(dual) = self.zone.dual_from(forest, leaf, rest, cursors) else {
                self.give_list(duals);
                return Ok(None);
            };
            rest.remove(dual);
            duals.push(dual);
        }
        // An occurrence's subtree follows it, so in reverse a `⊗` comes
        // after both its subformulas, the left one's proof on top.
        let mut built = self.take_links();
        for o in forest.subtree(self.occ(x)).rev() {
            let m = self.zone.child(x, o);
            let node = if forest.is_literal(o) {
                let dual = duals.pop().expect("a dual per literal");
                self.push(Node::Ax(m.into(), dual.into()))
            } else {
                self.statistics.splits += 1;
                let (_, left, _) = built.pop().expect("the left subformula's proof");
                let (_, right, _) = built.pop().expect("the right subformula's proof");
                self.push(Node::Tensor(m.into(), left, right))
            };
            built.push((m, node, Branch::Left));
        }
        let (_, node, _) = built.pop().expect("the tensor's proof");
        self.give_list(duals);
        self.give_links(built);
        Ok(Some(node))
    }

    /// Counts a forced split, a literal of a tensor closed in place or an
    /// `&` rule, and polls the stop condition once every
    /// [`FORCED_PER_POLL`] of them. No work is passed: the units two alternating searches count
    /// their slices in are what they are without this poll, and so is
    /// where each gives way.
    #[inline]
    pub(super) fn poll_forced(&mut self) -> Result<(), Reason> {
        self.forced += 1;
        if self.forced < FORCED_PER_POLL {
            return Ok(());
        }
        self.forced = 0;
        self.poll_now()
    }

    /// Polls the stop condition, passing no work: the rare branch of
    /// [`Self::poll_forced`], kept out of the chain's loop.
    #[cold]
    fn poll_now(&mut self) -> Result<(), Reason> {
        if self.stop.fired(0) {
            Err(Reason::Stopped)
        } else {
            Ok(())
        }
    }

    /// The `⊗` rule on a formula no factor of which forces its split: a
    /// search over the members of `Γ` for the splits whose two sides pass
    /// the counts. A left factor whose split is free again continues the
    /// search as a chain ([`Self::chain`]), in the same order.
    #[inline(always)]
    fn free_split(
        &mut self,
        theta: &Unrestricted<Z>,
        gamma: &Z::Linear,
        f: Z::Member,
        budget: u32,
    ) -> Step {
        let opened = self.open_split(gamma, f);
        #[cfg(feature = "parallel")]
        if self.cubes() && opened.members.len() >= 2 {
            let (a, b) = (opened.a, opened.b);
            let result = self.split_parallel(
                theta,
                &opened.members,
                (&opened.left, &opened.right),
                &opened.split,
                Join::Tensor(f, a, b),
                budget,
            );
            self.close_split(opened);
            return result;
        }
        if self.chains_on(opened.a) {
            return self.chain(theta, opened, budget);
        }
        // The parts given back one by one: moving the whole search on a
        // free split of every clause body was a tenth of a net's time.
        let Opened {
            a,
            b,
            members,
            mut left,
            mut right,
            mut split,
            ..
        } = opened;
        let result = self.search_splits(
            theta,
            &members,
            (0, 0),
            (&mut left, &mut right),
            &mut split,
            Join::Tensor(f, a, b),
            budget,
        );
        self.give_list(members);
        self.give_context(left);
        self.give_context(right);
        self.give_split(split);
        result
    }

    /// The search for the free splits of `F = A ⊗ B` over `Γ`, before its
    /// first step: the members in the order they are assigned and open in
    /// the counts, every member on the right, and, two-sided, on a
    /// hypothesis `A ⊸ B`, the goal on the consequent's side.
    #[inline(always)]
    fn open_split(&mut self, gamma: &Z::Linear, f: Z::Member) -> Opened<Z> {
        let (a, b) = (self.left(f), self.right(f));
        let mut members = self.take_list();
        gamma.list_into(&mut members);
        let mut left = self.take_context();
        let mut right = self.take_context_from(gamma);
        let mut split = self.take_split();
        split.place(self.problem.counts, self.occ(a), Branch::Left);
        split.place(self.problem.counts, self.occ(b), Branch::Right);
        let mut placed = self.take_list();
        placed.extend([a, b]);
        // Two-sided, on a hypothesis `A ⊸ B`: the goal stays with `B`, so it
        // is fixed on the consequent's side and left out of the search.
        if let Some(reading) = self.problem.reading
            && let Some((_, consequent)) = reading.implication(self.occ(f))
            && let Some(at) = members
                .iter()
                .position(|&m| reading.position(self.occ(m)) == Side::Output)
        {
            let goal = members.remove(at);
            placed.push(goal);
            if consequent == self.occ(a) {
                right.remove(goal);
                left.insert(goal);
                split.place(self.problem.counts, self.occ(goal), Branch::Left);
            } else {
                split.place(self.problem.counts, self.occ(goal), Branch::Right);
            }
        }
        self.open(&mut members, &mut split, &placed);
        self.give_list(placed);
        Opened {
            f,
            a,
            b,
            members,
            left,
            right,
            split,
        }
    }

    /// Gives the buffers of a split search back to the pools.
    fn close_split(&mut self, opened: Opened<Z>) {
        self.give_list(opened.members);
        self.give_context(opened.left);
        self.give_context(opened.right);
        self.give_split(opened.split);
    }

    /// Whether a focus on `a`, the left factor of a `⊗` whose split is
    /// searched on this thread, continues the search as a chain: a `⊗`
    /// that no factor forces, large enough to be a long one. A chain of a
    /// few links, a clause's body, recurses: the frames' bookkeeping cost
    /// a seventh of the stable sequents per second on Petri nets whose
    /// clauses have such bodies.
    fn chains_on(&self, a: Z::Member) -> bool {
        // A chain's frames would never ask a left premise for another
        // answer.
        if Z::FIRST_ORDER {
            return false;
        }
        #[cfg(feature = "parallel")]
        if self.cubes() {
            return false;
        }
        self.problem.forest.size(self.occ(a)) >= CHAIN_SIZE
            && self.kind(a) == Kind::Tensor
            && self.forced_factor(a).is_none()
    }

    /// The free splits of a chain of `⊗`, each the left factor of the one
    /// before, in a loop: each link's search is a frame on a list, the
    /// focus on the left factor of a split that the search reaches is the
    /// frame of the next link, and its result is given back to the frame
    /// below, which then searches the right premise and goes on as
    /// [`Self::search_splits`] does. The order of the steps, the nodes and
    /// the counters are those of the recursion through [`Self::focus`];
    /// only the levels of recursion are not taken, one per link, which a
    /// tensor of thousands of factors whose splits are searched used to
    /// run out of.
    fn chain(&mut self, theta: &Unrestricted<Z>, first: Opened<Z>, budget: u32) -> Step {
        let mut frames = self.take_frames();
        let walk = self.walk(first.members.len(), (0, 0));
        frames.push(Frame {
            opened: first,
            walk,
            cuts: Cuts::NONE,
            mark: 0,
        });
        let result = self.run_chain(theta, &mut frames, budget);
        while let Some(frame) = frames.pop() {
            self.close_frame(frame);
        }
        self.give_frames(frames);
        result
    }

    /// The loop of [`Self::chain`] over its frames, the last on top.
    fn run_chain(
        &mut self,
        theta: &Unrestricted<Z>,
        frames: &mut Vec<Frame<Z>>,
        budget: u32,
    ) -> Step {
        // What the left premise of the frame on top was found to be, once
        // the frame has asked for it.
        let mut given: Option<Found> = None;
        loop {
            let frame = frames.last_mut().expect("a frame until the first returns");
            match given.take() {
                Some(Found::Proved(l_node)) => {
                    let (l_node, r) = self.holding(l_node, |run| {
                        run.focus(theta, &frame.opened.right, frame.opened.b, budget)
                    });
                    match r? {
                        Found::Proved(r_node) => {
                            let node =
                                self.push(Node::Tensor(frame.opened.f.into(), l_node, r_node));
                            let frame = frames.pop().expect("the frame on top");
                            self.close_frame(frame);
                            if frames.is_empty() {
                                return Ok(Found::proved(node));
                            }
                            given = Some(Found::proved(node));
                            continue;
                        }
                        Found::Failed(failed) => {
                            self.nodes.release(frame.mark);
                            frame.cuts = frame.cuts.and(failed);
                        }
                    }
                }
                Some(Found::Failed(failed)) => frame.cuts = frame.cuts.and(failed),
                None => {}
            }
            let Opened {
                members,
                left,
                right,
                split,
                ..
            } = &mut frame.opened;
            if !self.next_split(&mut frame.walk, members, (left, right), split)? {
                let cuts = frame.cuts;
                let frame = frames.pop().expect("the frame on top");
                self.close_frame(frame);
                if frames.is_empty() {
                    return Ok(Found::failed(cuts));
                }
                given = Some(Found::failed(cuts));
                continue;
            }
            self.work += (self.problem.forest.len() / OCCURRENCES_PER_LEAF) as u64;
            frame.mark = self.nodes.mark();
            let a = frame.opened.a;
            if self.chains_on(a) {
                let opened = self.open_split(&frame.opened.left, a);
                let walk = self.walk(opened.members.len(), (0, 0));
                frames.push(Frame {
                    opened,
                    walk,
                    cuts: Cuts::NONE,
                    mark: 0,
                });
                continue;
            }
            given = Some(self.focus(theta, &frame.opened.left, a, budget)?);
        }
    }

    /// Gives the buffers of a frame of a chain back to the pools.
    fn close_frame(&mut self, frame: Frame<Z>) {
        self.give_trail(frame.walk.trail);
        self.close_split(frame.opened);
    }

    /// Puts the members a split search assigns in the order it decides
    /// them in and opens them in the split's counts: those that bear on
    /// more atoms first, then by the first atom of their rows, so that the
    /// members that bear on an atom are decided one after the other and
    /// its counts are settled early (those without a row last);
    /// interchangeable members next to each other, the lowest id last.
    /// `placed` are the members that have their side already.
    fn open(&self, members: &mut [Z::Member], split: &mut Split, placed: &[Z::Member]) {
        members.sort_unstable_by_key(|&m| {
            (
                std::cmp::Reverse(self.problem.counts.row_len(self.occ(m))),
                self.problem.counts.first_rank(self.occ(m)),
                self.class(m),
                std::cmp::Reverse(m),
            )
        });
        // Without the equation, a split can only fail the counts through a
        // member whose own interval of some atom excludes zero: with none,
        // every sum of intervals contains zero and every split passes, and
        // the counts need not know the members at all.
        let tight = |o: &Z::Member| self.problem.counts.tight(self.occ(*o));
        let inert = !self.problem.rules.equation
            && (!self.problem.rules.intervals || !placed.iter().chain(members.iter()).any(tight));
        split.set_inert(inert);
        if !inert {
            for &m in members.iter() {
                split.open(self.problem.counts, self.occ(m));
            }
        }
    }

    /// Searches the splits of a context into two sides that pass the
    /// counts: the premises of a `⊗` or the parts of a Mix, as `join`
    /// says. The members from `start` on are assigned one by one, in their
    /// order, each to the right first and then to the left, and a partial
    /// assignment is given up as soon as the counts show that no way of
    /// assigning the rest lets both sides pass. Of interchangeable members
    /// the left side takes those with the lowest ids, so that only their
    /// number varies: any other choice of as many gives the same two
    /// sequents up to a renaming. So every such split that passes the
    /// counts is reached, each once, and none that fails them. `left` and
    /// `right` are the sides and `split` their counts, with the members
    /// before `start` assigned as the bits of `prefix` say (one for the
    /// left) and the others on the right and open. Returns the node of the
    /// first split whose two sides are proved.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn search_splits(
        &mut self,
        theta: &Unrestricted<Z>,
        members: &[Z::Member],
        (start, prefix): (usize, u64),
        (left, right): (&mut Z::Linear, &mut Z::Linear),
        split: &mut Split,
        join: Join<Z::Member>,
        budget: u32,
    ) -> Step {
        if Z::FIRST_ORDER {
            return self.search_splits_framed(
                theta,
                members,
                (start, prefix),
                (left, right),
                split,
                join,
                budget,
            );
        }
        let mut walk = self.walk(members.len(), (start, prefix));
        // What the premises of the splits tried cut.
        let mut cuts = Cuts::NONE;
        let found = loop {
            match self.next_split(&mut walk, members, (left, right), split) {
                Ok(true) => {}
                Ok(false) => break Ok(None),
                Err(reason) => break Err(reason),
            }
            self.work += (self.problem.forest.len() / OCCURRENCES_PER_LEAF) as u64;
            let joined = match join {
                Join::Tensor(f, a, b) => self.premises(theta, left, right, f, a, b, budget),
                // A Mix needs two parts.
                Join::Mix if right.is_empty() => Ok(Found::NOTHING),
                Join::Mix => self.parts(theta, left, right, budget),
            };
            match joined {
                Ok(Found::Proved(node)) => break Ok(Some(node)),
                Ok(Found::Failed(failed)) => cuts = cuts.and(failed),
                Err(reason) => break Err(reason),
            }
        };
        self.give_trail(walk.trail);
        Ok(match found? {
            Some(node) => Found::proved(node),
            None => Found::failed(cuts),
        })
    }

    /// [`Self::search_splits`] in a first-order zone: the splits are the
    /// alternatives of a choice point that logs which one it tries, so
    /// that a premise's search can be resumed at it (`choices.rs`).
    #[allow(clippy::too_many_arguments)]
    fn search_splits_framed(
        &mut self,
        theta: &Unrestricted<Z>,
        members: &[Z::Member],
        (start, prefix): (usize, u64),
        (left, right): (&mut Z::Linear, &mut Z::Linear),
        split: &mut Split,
        join: Join<Z::Member>,
        budget: u32,
    ) -> Step {
        let choice = self.zone.choices().enter(Site::Splits);
        if choice.start == u32::MAX {
            self.zone.choices().fail(&choice, choice.cuts);
            return Ok(Found::failed(choice.cuts));
        }
        let mut walk = self.walk(members.len(), (start, prefix));
        let mut cuts = choice.cuts;
        let mut leaf = 0u32;
        let found = loop {
            match self.next_split(&mut walk, members, (left, right), split) {
                Ok(true) => {}
                Ok(false) => break Ok(None),
                Err(reason) => break Err(reason),
            }
            let i = leaf;
            // A split past the log's numbers is cut, never refuted.
            let Some(next) = leaf.checked_add(1) else {
                cuts = cuts.and(Cuts::BUDGET);
                break Ok(None);
            };
            leaf = next;
            if i < choice.start {
                continue;
            }
            self.zone.choices().take(&choice, i, cuts);
            self.work += (self.problem.forest.len() / OCCURRENCES_PER_LEAF) as u64;
            let joined = match join {
                Join::Tensor(f, a, b) => self.premises(theta, left, right, f, a, b, budget),
                Join::Mix if right.is_empty() => Ok(Found::NOTHING),
                Join::Mix => self.parts(theta, left, right, budget),
            };
            match joined {
                Ok(Found::Proved(node)) => break Ok(Some(node)),
                Ok(Found::Failed(failed)) => cuts = cuts.and(failed),
                Err(reason) => break Err(reason),
            }
        };
        self.give_trail(walk.trail);
        Ok(match found? {
            Some(node) => Found::proved(node),
            None => {
                self.zone.choices().fail(&choice, cuts);
                Found::failed(cuts)
            }
        })
    }

    /// [`Self::parts_fail`] in a first-order zone, as a choice point that
    /// logs its answer: a replay reads it rather than searching the parts
    /// again (`choices.rs`).
    fn parts_fail_logged(
        &mut self,
        theta: &Unrestricted<Z>,
        gamma: &Z::Linear,
        members: &[Z::Member],
        budget: u32,
    ) -> Result<bool, Reason> {
        let choice = self.zone.choices().enter(Site::Parts);
        if let Some(fail) = choice.settled {
            return Ok(fail == 1);
        }
        if choice.start == u32::MAX {
            return Ok(false);
        }
        let fail = self.parts_fail(theta, gamma, members, budget)?;
        self.zone.choices().settle(&choice, u32::from(fail));
        Ok(fail)
    }

    /// The walk of a split search over `len` members before its first
    /// step, the members before `start` assigned as the bits of `prefix`
    /// say (one for the left), the others on the right.
    fn walk(&mut self, len: usize, (start, prefix): (usize, u64)) -> Walk {
        let mut trail = self.take_trail();
        trail.extend((0..len).map(|i| {
            if i < start && prefix >> i & 1 == 1 {
                Branch::Left
            } else {
                Branch::Right
            }
        }));
        Walk {
            trail,
            next: start,
            start,
            at_leaf: false,
        }
    }

    /// Takes a split search to its next split whose two sides pass the
    /// counts, from the one it stopped at, and returns whether there is
    /// one. The members are assigned one by one, in their order, each to
    /// the right first and then to the left, and a partial assignment is
    /// given up as soon as the counts show that no way of assigning the
    /// rest lets both sides pass; of interchangeable members the left side
    /// takes those with the lowest ids, so that only their number varies.
    #[inline(always)]
    fn next_split(
        &mut self,
        walk: &mut Walk,
        members: &[Z::Member],
        (left, right): (&mut Z::Linear, &mut Z::Linear),
        split: &mut Split,
    ) -> Result<bool, Reason> {
        let rules = self.problem.rules;
        // The walk's place in locals while it moves, written back where it
        // stops.
        let (start, mut next) = (walk.start, walk.next);
        let trail = &mut walk.trail[..];
        // From a split already given, back to the last member assigned to
        // the right first.
        let mut back = std::mem::take(&mut walk.at_leaf);
        let found = 'search: loop {
            if !back {
                self.statistics.splits += 1;
                if let Err(reason) = self.poll_splits() {
                    walk.next = next;
                    return Err(reason);
                }
                if split.feasible(rules.intervals, rules.equation, rules.mix) {
                    if next < members.len() {
                        let m = members[next];
                        // On the left at once when the member before it is
                        // interchangeable and went left: the lowest ids do.
                        let side = if next > 0
                            && trail[next - 1] == Branch::Left
                            && self.zone.same(self.problem.classes, members[next - 1], m)
                        {
                            right.remove(m);
                            left.insert(m);
                            Branch::Left
                        } else {
                            Branch::Right
                        };
                        split.assign(self.problem.counts, self.occ(m), side);
                        trail[next] = side;
                        next += 1;
                        continue;
                    }
                    break true;
                }
            }
            back = false;
            // Back to the last member assigned to the right, which goes to
            // the left; those after it are open again.
            loop {
                if next == start {
                    break 'search false;
                }
                next -= 1;
                let m = members[next];
                if trail[next] == Branch::Right {
                    split.flip(self.problem.counts, self.occ(m), Branch::Left);
                    right.remove(m);
                    left.insert(m);
                    trail[next] = Branch::Left;
                    next += 1;
                    continue 'search;
                }
                split.unassign(self.problem.counts, self.occ(m), Branch::Left);
                left.remove(m);
                right.insert(m);
            }
        };
        walk.next = next;
        walk.at_leaf = found;
        Ok(found)
    }

    /// Polls the stop condition once every [`SPLITS_PER_POLL`] steps of
    /// the split searches: a search whose splits fail in focus visits no
    /// stable sequent, where the condition is polled otherwise, and may
    /// run for minutes. Inlined into the split loop, which calls it per
    /// step; the poll itself is the cold branch.
    #[inline(always)]
    fn poll_splits(&mut self) -> Result<(), Reason> {
        self.steps += 1;
        if self.steps < SPLITS_PER_POLL {
            return Ok(());
        }
        self.poll_splits_now()
    }

    /// Polls the stop condition, passing the work since the last poll:
    /// the rare branch of [`Self::poll_splits`], kept out of the split
    /// loop.
    #[cold]
    fn poll_splits_now(&mut self) -> Result<(), Reason> {
        self.steps = 0;
        let work = SPLITS_PER_POLL + std::mem::take(&mut self.work);
        if self.stop.fired(work) {
            Err(Reason::Stopped)
        } else {
            Ok(())
        }
    }

    /// Both premises of `F = A ⊗ B` for one split of the context, and the
    /// `⊗` node if both succeed. A function of its own, as
    /// [`parts`](Self::parts) is: inlined into the split search, both
    /// rules' locals would sit in its frame in a debug build, half a KiB
    /// more per level of the recursion.
    #[expect(
        clippy::too_many_arguments,
        reason = "the split search's state, passed through to the two focuses"
    )]
    fn premises(
        &mut self,
        theta: &Unrestricted<Z>,
        left: &Z::Linear,
        right: &Z::Linear,
        f: Z::Member,
        a: Z::Member,
        b: Z::Member,
        budget: u32,
    ) -> Step {
        if Z::FIRST_ORDER {
            let mut shared = right.iter().chain([b]).chain(theta.set.iter());
            return self.answers(
                &mut shared,
                |run| run.focus(theta, left, a, budget),
                |run| run.focus(theta, right, b, budget),
                |l, r| Node::Tensor(f.into(), l, r),
            );
        }
        self.two_premises(
            |run| run.focus(theta, left, a, budget),
            |run| run.focus(theta, right, b, budget),
            |l, r| Node::Tensor(f.into(), l, r),
        )
    }

    /// The Mix rule on a stable sequent no focus proves: a split into two
    /// non-empty provable parts, searched over the members after the
    /// first, which stays on the left, so that each unordered partition
    /// comes up once. In the multiplicative fragments only when the count
    /// equation admits a Mix. First, with a memo, the parts with one
    /// member less: when each of them fails hereditarily and completely,
    /// no part of this sequent's is provable, so no partition is searched
    /// and the failure is hereditary (the flag beside the result). A
    /// sequent of `n` members whose parts all fail then costs its `2ⁿ`
    /// parts `n` lookups each, where the partitions of every part cost
    /// `3ⁿ`; without a memo every part is searched again at every level,
    /// and the partitions are cheaper.
    pub(super) fn mix(
        &mut self,
        theta: &Unrestricted<Z>,
        gamma: &Z::Linear,
        members: &[Z::Member],
        tally: &Tally,
        budget: u32,
    ) -> Result<(Found, bool), Reason> {
        if members.len() < 2 || (self.problem.rules.equation && !tally.admits_mix()) {
            return Ok((Found::NOTHING, false));
        }
        let fail = self.problem.memoizes
            && if Z::FIRST_ORDER {
                self.parts_fail_logged(theta, gamma, members, budget)?
            } else {
                self.parts_fail(theta, gamma, members, budget)?
            };
        if fail {
            return Ok((Found::NOTHING, true));
        }
        let mut left = self.take_context();
        left.insert(members[0]);
        let mut right = self.take_context_from(gamma);
        right.remove(members[0]);
        let mut split = self.take_split();
        split.place(self.problem.counts, self.occ(members[0]), Branch::Left);
        let mut rest = self.take_list();
        rest.extend_from_slice(&members[1..]);
        self.open(&mut rest, &mut split, &members[..1]);
        let result = self.search_splits(
            theta,
            &rest,
            (0, 0),
            (&mut left, &mut right),
            &mut split,
            Join::Mix,
            budget,
        );
        self.give_list(rest);
        self.give_context(left);
        self.give_context(right);
        self.give_split(split);
        result.map(|found| (found, false))
    }

    /// Whether every part of `Γ` with one member less fails hereditarily
    /// and completely, at any budget and on any branch: then, as the
    /// sequent itself failed without Mix, no non-empty part of it is
    /// provable, since every proper part lies in one of them and a Mix of
    /// the whole would be of two proper parts. False at the first part
    /// that is proved, may have a provable part, or failed only within
    /// the budget or the branch: such a part, which the partitions would
    /// not all visit, must not qualify the sequent's failure (one cut by
    /// the budget kept `⊢ a, !?(s ⅋ a)` under Mix at its copy bound where
    /// the partitions refute it). Of members that are interchangeable or
    /// repeated one is left out, since the others leave a relative.
    fn parts_fail(
        &mut self,
        theta: &Unrestricted<Z>,
        gamma: &Z::Linear,
        members: &[Z::Member],
        budget: u32,
    ) -> Result<bool, Reason> {
        let mut left_out = self.take_list();
        left_out.extend_from_slice(members);
        self.one_of_each(&mut left_out);
        let mut fail = true;
        for i in 0..left_out.len() {
            let mut part = self.take_context_from(gamma);
            part.remove(left_out[i]);
            // A part is only asked whether it is provable: in a first-order
            // zone a proof's bindings are taken back, which the search of
            // the partitions must not see.
            let marks = Z::FIRST_ORDER.then(|| (self.nodes.mark(), self.zone.mark()));
            let found = self.prove_part(theta, &part, budget);
            if let Some((nodes, zone)) = marks
                && let Ok((Found::Proved(_), _)) = found
            {
                self.nodes.release(nodes);
                self.zone.undo(zone);
            }
            self.give_context(part);
            match found {
                Ok((Found::Failed(cuts), true)) if cuts == Cuts::NONE => {}
                Ok(_) => {
                    fail = false;
                    break;
                }
                Err(reason) => {
                    self.give_list(left_out);
                    return Err(reason);
                }
            }
        }
        self.give_list(left_out);
        Ok(fail)
    }

    /// Both parts of a Mix, and the Mix node if both are provable.
    fn parts(
        &mut self,
        theta: &Unrestricted<Z>,
        left: &Z::Linear,
        right: &Z::Linear,
        budget: u32,
    ) -> Step {
        if Z::FIRST_ORDER {
            let mut shared = right.iter().chain(theta.set.iter());
            return self.answers(
                &mut shared,
                |run| run.prove(theta, left, budget),
                |run| run.prove(theta, right, budget),
                Node::Mix,
            );
        }
        self.two_premises(
            |run| run.prove(theta, left, budget),
            |run| run.prove(theta, right, budget),
            Node::Mix,
        )
    }
}

/// A link of a chain of forced splits: the `⊗`, the proof of its forcing
/// factor, and which factor that is.
type Link<M> = (M, NodeId, Branch);

/// What joins the two sides of a split of a context.
#[derive(Clone, Copy, Debug)]
pub(super) enum Join<M> {
    /// The `⊗` rule on this formula, with its left and right subformula:
    /// the sides are its premises' contexts.
    Tensor(M, M, M),
    /// The Mix rule: the sides are its parts.
    Mix,
}

/// The factor of a `⊗` that forces its split, as
/// [`forced_factor`](Run::forced_factor) returns it.
#[derive(Clone, Copy, Debug)]
struct Forcing<M> {
    /// What the factor forces on its side of the split.
    forced: Forced,
    /// The forcing factor.
    factor: M,
    /// The other factor.
    other: M,
    /// Which of the two the forcing factor is.
    side: Branch,
}

/// What a factor of a `⊗` forces on its side of the split.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Forced {
    /// No split works: the factor is `0`.
    Nothing,
    /// The empty context: the factor is `1` or `!`.
    Empty,
    /// The dual literal alone: the factor is a positive literal.
    Dual,
    /// One dual literal per literal of the factor, a tensor of positive
    /// literals.
    Duals,
}

/// The occurrences a left factor of a `⊗` must have for its free split
/// to run as the next link of a loop and not recursively: a chain of
/// tensors of that size has some dozens of links, far below the
/// recursion limit, and recursing through it is cheaper.
const CHAIN_SIZE: u32 = 256;

/// A search for the free splits of a `⊗` over its context, opened: the
/// members it assigns, the two sides and their counts.
pub(super) struct Opened<Z: Zone> {
    /// The `⊗`.
    f: Z::Member,
    /// Its left factor.
    a: Z::Member,
    /// Its right factor.
    b: Z::Member,
    /// The members the search assigns, in its order.
    members: Pooled<Z::Member>,
    /// The left side.
    left: Z::Linear,
    /// The right side.
    right: Z::Linear,
    /// The counts of the two sides.
    split: Box<Split>,
}

/// Where a split search stands: the side of every member, how many are
/// assigned, from which member on it searches, and whether it stands at
/// a split it has given.
pub(super) struct Walk {
    /// Per member, its side, as far as it is assigned.
    trail: Pooled<Branch>,
    /// The members before it are assigned.
    next: usize,
    /// The first member the search assigns.
    start: usize,
    /// Whether the search stands at a split it returned.
    at_leaf: bool,
}

/// A link of a chain of free splits: its split search, what the splits
/// it tried cut, and the arena's mark when its last split's premises
/// began.
pub(super) struct Frame<Z: Zone> {
    /// The split search.
    opened: Opened<Z>,
    /// Where it stands.
    walk: Walk,
    /// What the premises of the splits tried cut.
    cuts: Cuts,
    /// The pending nodes from here on are the last split's premises'.
    mark: usize,
}
