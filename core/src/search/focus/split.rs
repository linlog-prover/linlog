// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The `⊗` rule and Mix: the splits a factor forces, in a loop along a
//! chain of them, and the search for the free splits whose two sides
//! pass the counts.

use super::context::Context;
use super::counts::{Split, Tally};
use super::{
    Cuts, Engine, FORCED_PER_POLL, Found, OCCURRENCES_PER_LEAF, SPLITS_PER_POLL, Search, Step,
};
use crate::occurrences::{OccId, OccSet, Position};
use crate::proofs::{Node, NodeId, Side};
use crate::search::Reason;
use crate::sequents::Kind;

impl Engine<'_> {
    /// The first occurrence of the literal dual to `literal` in `rest`, the
    /// context of a chain of forced splits, which is what [`Self::dual_in`]
    /// finds there, read from where the chain's last lookup of that
    /// literal ended: the context of a chain only loses members, so the
    /// occurrences passed over once are in it no more. A chain's lookups
    /// of one literal together cost its list once and not once each,
    /// which on a marking of thousands of equal tokens was most of a
    /// search's time.
    fn dual_from(&self, literal: OccId, rest: &Context, cursors: &mut Cursors) -> Option<OccId> {
        let f = self.forest;
        let (atom, sign) = (f.atom(literal)?, f.sign(literal)?);
        let duals = f.literals(atom, !sign);
        let list = 2 * atom.index() + (!sign) as usize;
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
        match self.forest.kind(factor) {
            Kind::Zero => Some(Forced::Nothing),
            _ if self.rules.affine => None,
            Kind::One | Kind::Bang => Some(Forced::Empty),
            Kind::Var | Kind::DualVar if self.counts.positive(self.forest, factor) => {
                Some(Forced::Dual)
            }
            Kind::Tensor if self.counts.literal_tensor(factor) => Some(Forced::Duals),
            _ => None,
        }
    }

    /// The factor of `F = A ⊗ B` that forces its split, with what it
    /// forces, the other factor, and whether the forcing one is the left.
    /// When both force, one that is closed in place (a literal, a unit)
    /// before a tensor of literals, which takes a focus of its own: a
    /// tensor of a thousand literals, nested to the left as it is read,
    /// must not pay a level of recursion per link; otherwise the left one.
    fn forced_factor(&self, f: OccId) -> Option<(Forced, OccId, OccId, bool)> {
        let (a, b) = (self.forest.left(f).unwrap(), self.forest.right(f).unwrap());
        [(a, b, true), (b, a, false)]
            .into_iter()
            .filter_map(|(x, y, left)| Some((self.forced_side(x)?, x, y, left)))
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
            Ok(Found {
                node: Some(mut node),
                cuts,
            }) => {
                for &(f, x_node, x_is_left) in links.iter().rev() {
                    let (left, right) = if x_is_left {
                        (x_node, node)
                    } else {
                        (node, x_node)
                    };
                    node = self.push(Node::Tensor(f, left, right));
                }
                Ok(Found {
                    node: Some(node),
                    cuts,
                })
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
        (links, cursors): (&mut Vec<(OccId, NodeId, bool)>, &mut Cursors),
        budget: u32,
    ) -> Step {
        // What the focuses on forcing factors along the chain cut.
        let mut cuts = Cuts::NONE;
        loop {
            let Some((forced, x, y, x_is_left)) = self.forced_factor(f) else {
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
                    let found = found?;
                    cuts = cuts.and(found.cuts);
                    found.node
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
                        Some(self.push(Node::Ax(x, dual)))
                    } else if let Some(d) = self.dual_in(x, |d| theta.contains(d)) {
                        if budget == 0 {
                            cuts = cuts.and(Cuts::BUDGET);
                            None
                        } else {
                            let ax = self.push(Node::Ax(x, d));
                            Some(self.push(Node::Copy(d, ax)))
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
            links.push((f, x_node, x_is_left));
            if self.forest.kind(y) != Kind::Tensor {
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
    fn literal_tensor(&mut self, x: OccId, rest: &mut Context, cursors: &mut Cursors) -> Search {
        let mut duals = self.take_list();
        for leaf in self.forest.subtree(x) {
            if !self.forest.is_literal(leaf) {
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
        for o in self.forest.subtree(x).rev() {
            let node = if self.forest.is_literal(o) {
                let dual = duals.pop().expect("a dual per literal");
                self.push(Node::Ax(o, dual))
            } else {
                self.statistics.splits += 1;
                let (_, left, _) = built.pop().expect("the left subformula's proof");
                let (_, right, _) = built.pop().expect("the right subformula's proof");
                self.push(Node::Tensor(o, left, right))
            };
            built.push((o, node, true));
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
    fn poll_forced(&mut self) -> Result<(), Reason> {
        self.forced += 1;
        if self.forced < FORCED_PER_POLL {
            return Ok(());
        }
        self.forced = 0;
        if self.stop.fired(0) {
            Err(Reason::Stopped)
        } else {
            Ok(())
        }
    }

    /// The `⊗` rule on a formula no factor of which forces its split: a
    /// search over the members of `Γ` for the splits whose two sides pass
    /// the counts.
    fn free_split(&mut self, theta: &OccSet, gamma: &Context, f: OccId, budget: u32) -> Step {
        let (a, b) = (self.forest.left(f).unwrap(), self.forest.right(f).unwrap());
        let mut members = self.take_list();
        members.extend(gamma.iter());
        let mut left = self.take_context();
        let mut right = self.take_context_from(gamma);
        let mut split = self.take_split();
        split.place(self.counts, a, Side::Left);
        split.place(self.counts, b, Side::Right);
        let mut placed = self.take_list();
        placed.extend([a, b]);
        // Two-sided, on a hypothesis `A ⊸ B`: the goal stays with `B`, so it
        // is fixed on the consequent's side and left out of the search.
        if let Some(reading) = self.reading
            && let Some((_, consequent)) = reading.implication(f)
            && let Some(at) = members
                .iter()
                .position(|&m| reading.position(m) == Position::Output)
        {
            let goal = members.remove(at);
            placed.push(goal);
            if consequent == a {
                right.remove(goal);
                left.insert(goal);
                split.place(self.counts, goal, Side::Left);
            } else {
                split.place(self.counts, goal, Side::Right);
            }
        }
        self.open(&mut members, &mut split, &placed);
        self.give_list(placed);
        let join = Join::Tensor(f, a, b);

        #[cfg(feature = "parallel")]
        let result = if self.cubes() && members.len() >= 2 {
            self.split_parallel(theta, &members, (&left, &right), &split, join, budget)
        } else {
            self.search_splits(
                theta,
                &members,
                (0, 0),
                (&mut left, &mut right),
                &mut split,
                join,
                budget,
            )
        };
        #[cfg(not(feature = "parallel"))]
        let result = self.search_splits(
            theta,
            &members,
            (0, 0),
            (&mut left, &mut right),
            &mut split,
            join,
            budget,
        );
        self.give_list(members);
        self.give_context(left);
        self.give_context(right);
        self.give_split(split);
        result
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
                std::cmp::Reverse(self.counts.row_len(m)),
                self.counts.first_atom(m),
                self.classes.of(m),
                std::cmp::Reverse(m),
            )
        });
        // Without the equation, a split can only fail the counts through a
        // member whose own interval of some atom excludes zero: with none,
        // every sum of intervals contains zero and every split passes, and
        // the counts need not know the members at all.
        let tight = |o: &OccId| self.counts.tight(*o);
        let inert = !self.rules.equation
            && (!self.rules.intervals || !placed.iter().chain(members.iter()).any(tight));
        split.set_inert(inert);
        if !inert {
            for &m in members.iter() {
                split.open(self.counts, m);
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
        let rules = self.rules;
        let mut trail = self.take_trail();
        trail.extend((0..members.len()).map(|i| {
            if i < start && prefix >> i & 1 == 1 {
                Side::Left
            } else {
                Side::Right
            }
        }));
        // The members before `next` are assigned, as the trail says.
        let mut next = start;
        // What the premises of the splits tried cut.
        let mut cuts = Cuts::NONE;
        let found = 'search: loop {
            self.statistics.splits += 1;
            self.poll_splits()?;
            if split.feasible(rules.intervals, rules.equation, rules.mix) {
                if next < members.len() {
                    let m = members[next];
                    // On the left at once when the member before it is
                    // interchangeable and went left: the lowest ids do.
                    let side = if next > 0
                        && trail[next - 1] == Side::Left
                        && self.classes.same(members[next - 1], m)
                    {
                        right.remove(m);
                        left.insert(m);
                        Side::Left
                    } else {
                        Side::Right
                    };
                    split.assign(self.counts, m, side);
                    trail[next] = side;
                    next += 1;
                    continue;
                }
                self.work += (self.forest.len() / OCCURRENCES_PER_LEAF) as u64;
                let joined = match join {
                    Join::Tensor(f, a, b) => self.premises(theta, left, right, f, a, b, budget)?,
                    // A Mix needs two parts.
                    Join::Mix if right.is_empty() => Found::NOTHING,
                    Join::Mix => self.parts(theta, left, right, budget)?,
                };
                cuts = cuts.and(joined.cuts);
                if joined.node.is_some() {
                    break joined.node;
                }
            }
            // Back to the last member assigned to the right, which goes to
            // the left; those after it are open again.
            loop {
                if next == start {
                    break 'search None;
                }
                next -= 1;
                let m = members[next];
                if trail[next] == Side::Right {
                    split.flip(self.counts, m, Side::Left);
                    right.remove(m);
                    left.insert(m);
                    trail[next] = Side::Left;
                    next += 1;
                    continue 'search;
                }
                split.unassign(self.counts, m, Side::Left);
                left.remove(m);
                right.insert(m);
            }
        };
        self.give_trail(trail);
        Ok(Found { node: found, cuts })
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
        let Some(l_node) = l.node else {
            return Ok(l);
        };
        let held = self.nodes.hold(l_node);
        let r = self.focus(theta, right, b, budget)?.after(l.cuts);
        let l_node = self.nodes.unhold(held);
        let Some(r_node) = r.node else {
            self.nodes.release(mark);
            return Ok(r);
        };
        Ok(Found {
            node: Some(self.push(Node::Tensor(f, l_node, r_node))),
            cuts: r.cuts,
        })
    }

    /// The Mix rule on a stable sequent no focus proves: a split into two
    /// non-empty provable parts, searched over the members after the
    /// first, which stays on the left, so that each unordered partition
    /// comes up once. In the multiplicative fragments only when the count
    /// equation admits a Mix.
    pub(super) fn mix(
        &mut self,
        theta: &OccSet,
        gamma: &Context,
        members: &[OccId],
        tally: &Tally,
        budget: u32,
    ) -> Step {
        if members.len() < 2 || (self.rules.equation && !tally.admits_mix()) {
            return Ok(Found::NOTHING);
        }
        let mut left = self.take_context();
        left.insert(members[0]);
        let mut right = self.take_context_from(gamma);
        right.remove(members[0]);
        let mut split = self.take_split();
        split.place(self.counts, members[0], Side::Left);
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

    /// Both parts of a Mix, and the Mix node if both are provable.
    fn parts(&mut self, theta: &OccSet, left: &Context, right: &Context, budget: u32) -> Step {
        let mark = self.nodes.mark();
        let l = self.prove(theta, left, budget)?;
        let Some(l_node) = l.node else {
            return Ok(l);
        };
        let held = self.nodes.hold(l_node);
        let r = self.prove(theta, right, budget)?.after(l.cuts);
        let l_node = self.nodes.unhold(held);
        let Some(r_node) = r.node else {
            self.nodes.release(mark);
            return Ok(r);
        };
        Ok(Found {
            node: Some(self.push(Node::Mix(l_node, r_node))),
            cuts: r.cuts,
        })
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
