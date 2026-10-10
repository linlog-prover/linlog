// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The `⊗` rule and Mix: the splits a factor forces, in a loop along a
//! chain of them, and the search for the free splits whose two sides
//! pass the counts.

use super::context::Context;
use super::counts::{Split, Tally};
use super::scratch::Pooled;
use super::{
    Cuts, FORCED_PER_POLL, Found, OCCURRENCES_PER_LEAF, Run, SPLITS_PER_POLL, Searched, Step,
};
use crate::occurrences::{Forest, Member, OccId, OccSet, Side};
use crate::proofs::{Branch, Node, NodeId};
use crate::search::Reason;
use crate::sequents::Kind;

impl Run<'_> {
    /// The first occurrence of the literal dual to `literal` in `rest`, the
    /// context of a chain of forced splits, which is what [`Self::dual_in`]
    /// finds there, read from where the chain's last lookup of that
    /// literal ended: the context of a chain only loses members, so the
    /// occurrences passed over once are in it no more. A chain's lookups
    /// of one literal together cost its list once and not once each,
    /// which on a marking of thousands of equal tokens was most of a
    /// search's time.
    fn dual_from(&self, literal: OccId, rest: &Context, cursors: &mut Cursors) -> Option<OccId> {
        let f = self.problem.forest;
        let (atom, sign) = (f.atom(literal)?, f.sign(literal)?);
        let duals = f.literals(atom, !sign);
        let list = Forest::list(atom, !sign);
        let start = cursors.passed[list] as usize;
        let found = duals[start..].iter().position(|&d| rest.contains(d));
        let passed = found.map_or(duals.len(), |at| start + at);
        if passed != start {
            if start == 0 {
                cursors.moved.push(list as u32);
            }
            // A list is a part of the forest's occurrences, which a `u32`
            // indexes.
            cursors.passed[list] = passed as u32;
        }
        found.map(|at| duals[start + at])
    }

    /// What a factor of a `⊗` forces on its side of the split: nothing at
    /// all for `0`, the empty context for `1` and `!`, the dual literal
    /// alone for a positive literal, and one dual per literal for a tensor
    /// of positive literals whose duals lie in the linear zone only. With
    /// weakening nothing but `0` forces anything, since every leaf takes
    /// any context.
    pub(super) fn forced_side(&self, factor: OccId) -> Option<Forced> {
        match self.problem.forest.kind(factor) {
            Kind::Zero => Some(Forced::Nothing),
            _ if self.problem.rules.affine => None,
            Kind::One | Kind::Bang => Some(Forced::Empty),
            Kind::Atom | Kind::DualAtom
                if self.problem.counts.positive(self.problem.forest, factor) =>
            {
                Some(Forced::Dual)
            }
            Kind::Tensor if self.problem.counts.literal_tensor(factor) => Some(Forced::Duals),
            _ => None,
        }
    }

    /// The factor of `F = A ⊗ B` that forces its split, with what it
    /// forces, the other factor, and which of the two the forcing one is.
    /// When both force, one that is closed in place (a literal, a unit)
    /// before a tensor of literals, which takes a focus of its own: a
    /// tensor of a thousand literals, nested to the left as it is read,
    /// must not pay a level of recursion per link; otherwise the left one.
    fn forced_factor(&self, f: OccId) -> Option<(Forced, OccId, OccId, Branch)> {
        let (a, b) = (
            self.problem.forest.left(f).unwrap(),
            self.problem.forest.right(f).unwrap(),
        );
        [(a, b, Branch::Left), (b, a, Branch::Right)]
            .into_iter()
            .filter_map(|(x, y, side)| Some((self.forced_side(x)?, x, y, side)))
            .min_by_key(|&(forced, ..)| forced == Forced::Duals)
    }

    /// The `⊗` rule on `F = A ⊗ B` with context `Γ`: the forced split when
    /// a factor allows only one, else a search over the members of `Γ` for
    /// the splits whose two sides pass the counts.
    pub(super) fn split(&mut self, theta: &OccSet, gamma: &Context, f: OccId, budget: u32) -> Step {
        if self.forced_factor(f).is_none() {
            return self.free_split(theta, gamma, f, budget);
        }
        let mark = self.nodes.mark();
        let mut rest = self.take_context_from(gamma);
        let mut links = self.take_links();
        let mut cursors = self.take_cursors();
        let result = self.forced_splits(theta, &mut rest, f, (&mut links, &mut cursors), budget);
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
                    node = self.push(Node::Tensor(Member::from(f), left, right));
                }
                Ok(Found::proved(node))
            }
            Ok(failed) => {
                self.nodes.release(mark);
                Ok(failed)
            }
            Err(reason) => Err(reason),
        };
        self.give_links(links);
        result
    }

    /// The forced splits of `F` and, as long as the factor they leave is a
    /// `⊗` with a forced split again, of that factor, in a loop: a tensor
    /// of a thousand literals costs one level of recursion and one copy of
    /// the context, not a thousand. `rest` is the context, from which
    /// every forcing factor takes its own; `links` gets every `⊗` with the
    /// proof of its forcing factor and whether that is the left one, and
    /// `cursors` remembers where the chain's lookups of duals ended.
    /// Returns the proof of the factor left at the end with what remains
    /// of the context. The chain visits no stable sequent, so it polls
    /// the stop condition itself, once every [`FORCED_PER_POLL`] splits
    /// and literals closed.
    fn forced_splits(
        &mut self,
        theta: &OccSet,
        rest: &mut Context,
        mut f: OccId,
        (links, cursors): (&mut Vec<(OccId, NodeId, Branch)>, &mut Cursors),
        budget: u32,
    ) -> Step {
        // What the focuses on forcing factors along the chain cut.
        let mut cuts = Cuts::NONE;
        loop {
            let Some((forced, x, y, x_side)) = self.forced_factor(f) else {
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
                    if let Some(dual) = self.dual_from(x, rest, cursors) {
                        rest.remove(dual);
                        Some(self.push(Node::Ax(x.into(), dual.into())))
                    } else if let Some(d) = self.dual_in(x, |d| theta.contains(d)) {
                        if budget == 0 {
                            cuts = cuts.and(Cuts::BUDGET);
                            None
                        } else {
                            let ax = self.push(Node::Ax(x.into(), d.into()));
                            Some(self.push(Node::Copy(Member::from(d), ax)))
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
            if self.problem.forest.kind(y) != Kind::Tensor {
                return Ok(self.focus(theta, rest, y, budget)?.after(cuts));
            }
            f = y;
        }
    }

    /// The proof of a tensor of positive literals from one dual per
    /// literal, each the first left in `rest`, which loses them; `None`
    /// when a dual is missing. Built in place, the axioms and the `⊗`
    /// nodes from the last occurrence back, so a tensor of any depth costs
    /// no recursion; the stop condition is polled as in the chain around
    /// it.
    fn literal_tensor(&mut self, x: OccId, rest: &mut Context, cursors: &mut Cursors) -> Searched {
        let mut duals = self.take_list();
        for leaf in self.problem.forest.subtree(x) {
            if !self.problem.forest.is_literal(leaf) {
                continue;
            }
            self.poll_forced()?;
            let Some(dual) = self.dual_from(leaf, rest, cursors) else {
                self.give_list(duals);
                return Ok(None);
            };
            rest.remove(dual);
            duals.push(dual);
        }
        // An occurrence's subtree follows it, so in reverse a `⊗` comes
        // after both its subformulas, the left one's proof on top.
        let mut built = self.take_links();
        for o in self.problem.forest.subtree(x).rev() {
            let node = if self.problem.forest.is_literal(o) {
                let dual = duals.pop().expect("a dual per literal");
                self.push(Node::Ax(o.into(), dual.into()))
            } else {
                self.statistics.splits += 1;
                let (_, left, _) = built.pop().expect("the left subformula's proof");
                let (_, right, _) = built.pop().expect("the right subformula's proof");
                self.push(Node::Tensor(Member::from(o), left, right))
            };
            built.push((o, node, Branch::Left));
        }
        let (_, node, _) = built.pop().expect("the tensor's proof");
        self.give_list(duals);
        self.give_links(built);
        Ok(Some(node))
    }

    /// Counts a forced split, or a literal of a tensor closed in place,
    /// and polls the stop condition once every [`FORCED_PER_POLL`] of
    /// them. No work is passed: the units two alternating searches count
    /// their slices in are what they are without this poll, and so is
    /// where each gives way.
    #[inline]
    fn poll_forced(&mut self) -> Result<(), Reason> {
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
    fn free_split(&mut self, theta: &OccSet, gamma: &Context, f: OccId, budget: u32) -> Step {
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
    fn open_split(&mut self, gamma: &Context, f: OccId) -> Opened {
        let (a, b) = (
            self.problem.forest.left(f).unwrap(),
            self.problem.forest.right(f).unwrap(),
        );
        let mut members = self.take_list();
        members.extend(gamma.iter());
        let mut left = self.take_context();
        let mut right = self.take_context_from(gamma);
        let mut split = self.take_split();
        split.place(self.problem.counts, a, Branch::Left);
        split.place(self.problem.counts, b, Branch::Right);
        let mut placed = self.take_list();
        placed.extend([a, b]);
        // Two-sided, on a hypothesis `A ⊸ B`: the goal stays with `B`, so it
        // is fixed on the consequent's side and left out of the search.
        if let Some(reading) = self.problem.reading
            && let Some((_, consequent)) = reading.implication(f)
            && let Some(at) = members
                .iter()
                .position(|&m| reading.position(m) == Side::Output)
        {
            let goal = members.remove(at);
            placed.push(goal);
            if consequent == a {
                right.remove(goal);
                left.insert(goal);
                split.place(self.problem.counts, goal, Branch::Left);
            } else {
                split.place(self.problem.counts, goal, Branch::Right);
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
    fn close_split(&mut self, opened: Opened) {
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
    fn chains_on(&self, a: OccId) -> bool {
        #[cfg(feature = "parallel")]
        if self.cubes() {
            return false;
        }
        self.problem.forest.size(a) >= CHAIN_SIZE
            && self.problem.forest.kind(a) == Kind::Tensor
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
    fn chain(&mut self, theta: &OccSet, first: Opened, budget: u32) -> Step {
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
    fn run_chain(&mut self, theta: &OccSet, frames: &mut Vec<Frame>, budget: u32) -> Step {
        // What the left premise of the frame on top was found to be, once
        // the frame has asked for it.
        let mut given: Option<Found> = None;
        loop {
            let frame = frames.last_mut().expect("a frame until the first returns");
            match given.take() {
                Some(Found::Proved(l_node)) => {
                    let held = self.nodes.hold(l_node);
                    let r = self.focus(theta, &frame.opened.right, frame.opened.b, budget)?;
                    let l_node = self.nodes.unhold(held);
                    match r {
                        Found::Proved(r_node) => {
                            let node = self.push(Node::Tensor(
                                Member::from(frame.opened.f),
                                l_node,
                                r_node,
                            ));
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
    fn close_frame(&mut self, frame: Frame) {
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
    fn open(&self, members: &mut [OccId], split: &mut Split, placed: &[OccId]) {
        members.sort_unstable_by_key(|&m| {
            (
                std::cmp::Reverse(self.problem.counts.row_len(m)),
                self.problem.counts.first_rank(m),
                self.problem.classes.of(m),
                std::cmp::Reverse(m),
            )
        });
        // Without the equation, a split can only fail the counts through a
        // member whose own interval of some atom excludes zero: with none,
        // every sum of intervals contains zero and every split passes, and
        // the counts need not know the members at all.
        let tight = |o: &OccId| self.problem.counts.tight(*o);
        let inert = !self.problem.rules.equation
            && (!self.problem.rules.intervals || !placed.iter().chain(members.iter()).any(tight));
        split.set_inert(inert);
        if !inert {
            for &m in members.iter() {
                split.open(self.problem.counts, m);
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
        theta: &OccSet,
        members: &[OccId],
        (start, prefix): (usize, u64),
        (left, right): (&mut Context, &mut Context),
        split: &mut Split,
        join: Join,
        budget: u32,
    ) -> Step {
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
        members: &[OccId],
        (left, right): (&mut Context, &mut Context),
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
                            && self.problem.classes.same(members[next - 1], m)
                        {
                            right.remove(m);
                            left.insert(m);
                            Branch::Left
                        } else {
                            Branch::Right
                        };
                        split.assign(self.problem.counts, m, side);
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
                    split.flip(self.problem.counts, m, Branch::Left);
                    right.remove(m);
                    left.insert(m);
                    trail[next] = Branch::Left;
                    next += 1;
                    continue 'search;
                }
                split.unassign(self.problem.counts, m, Branch::Left);
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
    /// run for minutes.
    fn poll_splits(&mut self) -> Result<(), Reason> {
        self.steps += 1;
        if self.steps < SPLITS_PER_POLL {
            return Ok(());
        }
        self.steps = 0;
        let work = SPLITS_PER_POLL + std::mem::take(&mut self.work);
        if self.stop.fired(work) {
            Err(Reason::Stopped)
        } else {
            Ok(())
        }
    }

    /// Both premises of `F = A ⊗ B` for one split of the context, and the
    /// `⊗` node if both succeed.
    #[allow(clippy::too_many_arguments)]
    fn premises(
        &mut self,
        theta: &OccSet,
        left: &Context,
        right: &Context,
        f: OccId,
        a: OccId,
        b: OccId,
        budget: u32,
    ) -> Step {
        let mark = self.nodes.mark();
        let l = self.focus(theta, left, a, budget)?;
        let Found::Proved(l_node) = l else {
            return Ok(l);
        };
        let held = self.nodes.hold(l_node);
        let r = self.focus(theta, right, b, budget)?;
        let l_node = self.nodes.unhold(held);
        let Found::Proved(r_node) = r else {
            self.nodes.release(mark);
            return Ok(r);
        };
        Ok(Found::proved(self.push(Node::Tensor(
            Member::from(f),
            l_node,
            r_node,
        ))))
    }

    /// The Mix rule on a stable sequent no focus proves: a split into two
    /// non-empty provable parts, searched over the members after the
    /// first, which stays on the left, so that each unordered partition
    /// comes up once. In the multiplicative fragments only when the count
    /// equation admits a Mix. First, with a memo, the parts with one
    /// member less: when each of them fails hereditarily and completely,
    /// no part of this sequent's is provable, so no partition is searched
    /// and `hereditary` is set. A sequent of `n` members whose parts all
    /// fail then costs its `2ⁿ` parts `n` lookups each, where the
    /// partitions of every part cost `3ⁿ`; without a memo every part is
    /// searched again at every level, and the partitions are cheaper.
    pub(super) fn mix(
        &mut self,
        theta: &OccSet,
        gamma: &Context,
        members: &[OccId],
        tally: &Tally,
        budget: u32,
        hereditary: &mut bool,
    ) -> Step {
        if members.len() < 2 || (self.problem.rules.equation && !tally.admits_mix()) {
            return Ok(Found::NOTHING);
        }
        if self.problem.memoizes && self.parts_fail(theta, gamma, members, budget)? {
            *hereditary = true;
            return Ok(Found::NOTHING);
        }
        let mut left = self.take_context();
        left.insert(members[0]);
        let mut right = self.take_context_from(gamma);
        right.remove(members[0]);
        let mut split = self.take_split();
        split.place(self.problem.counts, members[0], Branch::Left);
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
        result
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
        theta: &OccSet,
        gamma: &Context,
        members: &[OccId],
        budget: u32,
    ) -> Result<bool, Reason> {
        let mut left_out = self.take_list();
        left_out.extend_from_slice(members);
        self.one_of_each(&mut left_out);
        let mut fail = true;
        for i in 0..left_out.len() {
            let mut part = self.take_context_from(gamma);
            part.remove(left_out[i]);
            let found = self.prove_part(theta, &part, budget);
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
    fn parts(&mut self, theta: &OccSet, left: &Context, right: &Context, budget: u32) -> Step {
        let mark = self.nodes.mark();
        let l = self.prove(theta, left, budget)?;
        let Found::Proved(l_node) = l else {
            return Ok(l);
        };
        let held = self.nodes.hold(l_node);
        let r = self.prove(theta, right, budget)?;
        let l_node = self.nodes.unhold(held);
        let Found::Proved(r_node) = r else {
            self.nodes.release(mark);
            return Ok(r);
        };
        Ok(Found::proved(self.push(Node::Mix(l_node, r_node))))
    }
}

/// How far a chain of forced splits has read the forest's lists of the
/// occurrences of each literal in its search for duals.
pub(super) struct Cursors {
    /// Per list, twice the atom plus the sign as the forest numbers them:
    /// how many occurrences at its head are no longer in the chain's
    /// context.
    pub(super) passed: Vec<u32>,
    /// The lists whose entry is not zero.
    pub(super) moved: Vec<u32>,
}

/// What joins the two sides of a split of a context.
#[derive(Clone, Copy, Debug)]
pub(super) enum Join {
    /// The `⊗` rule on this formula, with its left and right subformula:
    /// the sides are its premises' contexts.
    Tensor(OccId, OccId, OccId),
    /// The Mix rule: the sides are its parts.
    Mix,
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
pub(super) struct Opened {
    /// The `⊗`.
    f: OccId,
    /// Its left factor.
    a: OccId,
    /// Its right factor.
    b: OccId,
    /// The members the search assigns, in its order.
    members: Pooled<OccId>,
    /// The left side.
    left: Context,
    /// The right side.
    right: Context,
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
pub(super) struct Frame {
    /// The split search.
    opened: Opened,
    /// Where it stands.
    walk: Walk,
    /// What the premises of the splits tried cut.
    cuts: Cuts,
    /// The pending nodes from here on are the last split's premises'.
    mark: usize,
}
