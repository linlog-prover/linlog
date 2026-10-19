// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The check of a refutation by saturation ([`Saturated`]): a set of
//! stable sequents closed under the derived rules of the focused calculus,
//! none of which stands for a stable sequent of the goal's asynchronous
//! phase that a proof needs. It shares no code with the search that
//! saturates: the rules are read off the forest here again, and every
//! rule is applied to every choice of premises from the set.
//!
//! Why it refutes. Fix the polarity of the atoms the certificate names.
//! Focusing is complete under every polarity and in every commutative
//! mode, so a provable goal has a focused proof. Its asynchronous phases
//! end in stable sequents (positive formulas and negative literals in the
//! linear zone, the subformulas of `?` in the unrestricted one); take each
//! with a negative literal under a `?` in its linear zone moved back to
//! the unrestricted zone (only a copy put it there, and the copy left it
//! there too), and with the globals below left out of it. By induction
//! over the proof, every such sequent is one a member of the set stands
//! for: the phase above it is an instance of a rule below (its focus or
//! copy, the sides of its `⊕`, the partners of its axioms, the releases
//! ending in its premises), each premise is one a member stands for, and
//! applied to those members the rule derives a sequent that stands for
//! it, since every step is monotone in what a sequent stands for (a
//! member absent from a premise that absorbs or weakens it is absorbed or
//! weakened at the step that takes it, the `&` join of two that stand
//! for equal zones stands for that zone, and the steps keep one output
//! under the reading). That sequent, its globals left out, is coherent,
//! since the proof's is, and so the closure gives a member that stands
//! for it, and for the proof's sequent. So a member stands for each
//! stable sequent of the goal's phase, which is what filling a hole
//! asks of a hole; one hole that no member fills refutes the goal.
//!
//! - *What a sequent stands for*: `linear ; unrestricted` stands for the
//!   stable sequents with exactly that linear zone and an unrestricted
//!   zone that holds `unrestricted`; where it `absorbs` (a `⊤` above), or
//!   in affine mode, for a larger linear zone too, a member in output
//!   position absorbed only where the zone has none, and weakened in
//!   never.
//! - *The normal form* of the partners: a negative literal under a `?`
//!   reaches the linear zone only by a copy, so an axiom takes it from the
//!   unrestricted zone, and from the linear zone only where the goal holds
//!   it; no copy rule is read for it. A partner outside the goal's
//!   subtrees stands in no proof of the goal, which a goal off the roots
//!   has beside it.
//! - *Coherence*: outside the subformulas of `?` no proof's sequent holds
//!   an occurrence twice, or two that a connective other than `⅋` joins
//!   (one below the other, or a `⊗`, `&` or `⊕` above both, which sends
//!   them to different premises or keeps one side), given a goal whose own
//!   members are coherent. The sorted members' neighbours are tested,
//!   since the forest numbers its occurrences in preorder, and the common
//!   ancestors of a sorted list are those of its neighbours.
//! - *The globals*: the subformulas of the `?` the goal reaches through
//!   `⅋` alone are in the unrestricted zone of every sequent of a proof,
//!   since the goal's phase decomposes the `?` before any `&`, so a set's
//!   sequents may leave them out, and a sequent derived is compared with
//!   them left out.

use super::refutation::{RefutationFault, Saturated, Stable};
use crate::fragment::Mode;
use crate::hash::HashMap;
use crate::occurrences::{Forest, OccId, Reading, Side, Sign};
use crate::sequents::Kind;

/// Why a check did not accept a certificate: it does not hold, or the
/// stop fired before the check ended.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Failed {
    /// The certificate does not hold.
    Fault(RefutationFault),
    /// The stop fired.
    Stopped,
}

impl From<RefutationFault> for Failed {
    /// The fault as a failure.
    fn from(fault: RefutationFault) -> Self {
        Failed::Fault(fault)
    }
}

/// How many units of work the check does between two polls of its stop.
const UNITS_PER_POLL: u64 = 1 << 16;

/// A sequent as the check derives it: its linear zone sorted with repeats,
/// its unrestricted zone sorted without, and whether a `⊤` above absorbs
/// any further linear member.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Zone {
    /// The linear zone.
    linear: Vec<OccId>,
    /// The unrestricted zone.
    unrestricted: Vec<OccId>,
    /// Whether it absorbs.
    absorbs: bool,
}

/// A step of a rule's term, written in postorder: the operands of a step
/// are the values of the steps before it, on a stack.
#[derive(Clone, Copy, Debug)]
enum Step {
    /// The axiom of a positive literal with the partner of a slot.
    Axiom(OccId, usize),
    /// `1`.
    One(OccId),
    /// `⊤`, absorbing.
    Top(OccId),
    /// The premise of a hole.
    Hole(usize),
    /// `⊗`.
    Tensor(OccId),
    /// `⊕` on the side given.
    Plus(OccId, OccId),
    /// `!`.
    Bang(OccId),
    /// `⅋`.
    Par(OccId),
    /// `⊥`.
    Bot(OccId),
    /// `?`.
    Quest(OccId),
    /// `&`.
    With(OccId),
    /// A copy of a `?` formula's subformula into the unrestricted zone.
    Copy(OccId),
}

/// The partner of a positive literal: a negative literal of its atom,
/// from the linear zone or, under a copy, from the unrestricted one.
#[derive(Clone, Copy, Debug)]
struct Partner {
    /// The negative literal.
    literal: OccId,
    /// Whether it comes from the unrestricted zone.
    unrestricted: bool,
}

/// A stable sequent a rule's asynchronous phase ends in: the members the
/// phase takes from its premise, the subformulas of `?` it met, and the
/// release it belongs to (the premises of one release are joined by its
/// `&`, those of different releases side by side).
#[derive(Clone, Debug)]
struct Hole {
    /// The members taken, sorted.
    leaves: Vec<OccId>,
    /// The subformulas of `?` met, sorted.
    met: Vec<OccId>,
    /// The release.
    release: usize,
}

/// A rule: its term in postorder, its holes and the partners of its slots.
#[derive(Debug, Default)]
struct Rule {
    /// The steps.
    steps: Vec<Step>,
    /// The holes, by the index their steps name.
    holes: Vec<Hole>,
    /// The partners each slot may take, by the index the steps name.
    slots: Vec<Vec<Partner>>,
}

/// Where a rule's term starts.
#[derive(Clone, Copy)]
enum Start<'g> {
    /// A focus on a positive formula.
    Focus(OccId),
    /// A copy of a `?` formula's subformula, focused on or released.
    Copy(OccId),
    /// The goal's asynchronous phase.
    Goal(&'g [OccId]),
}

/// What is left to write of a rule's term.
enum Pending {
    /// A focus on a positive formula.
    Focus(OccId),
    /// An asynchronous phase.
    Phase(Phase),
}

/// An asynchronous phase being written: the formulas left to decompose,
/// and the stable members and `?` subformulas met.
#[derive(Clone, Default)]
struct Phase {
    /// A `⊤` met.
    top: Option<OccId>,
    /// The `⅋`, `⊥` and `?` left.
    unary: Vec<OccId>,
    /// The `&` left, decomposed after the unary ones.
    withs: Vec<OccId>,
    /// The stable members met.
    leaves: Vec<OccId>,
    /// The subformulas of `?` met.
    met: Vec<OccId>,
    /// The release.
    release: usize,
}

/// The calculus the certificate is read in: the forest under its polarity
/// in the mode, and the goal.
struct Calculus<'a> {
    /// The forest.
    forest: &'a Forest,
    /// The positive literal of each atom.
    positive: &'a [Sign],
    /// The reading, in intuitionistic mode.
    reading: Option<Reading<'a>>,
    /// Whether weakening is allowed.
    affine: bool,
    /// Whether each occurrence lies below a `?`.
    copied: Vec<bool>,
    /// Whether each occurrence lies in a goal member's subtree, which a
    /// proof of the goal holds nothing outside of.
    inside: Vec<bool>,
    /// The goal's members, sorted.
    goal: Vec<OccId>,
    /// The subformulas of the `?` the goal reaches through `⅋` alone,
    /// sorted.
    globals: Vec<OccId>,
}

impl<'a> Calculus<'a> {
    /// Whether an occurrence is positive: `⊗ 1 ⊕ 0 !` and the literals the
    /// polarity names.
    fn positive(&self, o: OccId) -> bool {
        match self.forest.kind(o) {
            Kind::Tensor | Kind::One | Kind::Plus | Kind::Zero | Kind::Bang => true,
            Kind::Atom | Kind::DualAtom => {
                self.forest.atom(o).map(|a| self.positive[a.index()]) == self.forest.sign(o)
            }
            Kind::Par | Kind::Bot | Kind::With | Kind::Top | Kind::Quest => false,
        }
    }

    /// Whether an occurrence stays in a stable sequent: positive, or a
    /// literal.
    fn stable(&self, o: OccId) -> bool {
        self.positive(o) || self.forest.is_literal(o)
    }

    /// Whether an occurrence is in output position.
    fn output(&self, o: OccId) -> bool {
        self.reading
            .as_ref()
            .is_some_and(|r| r.position(o) == Side::Output)
    }

    /// The members of a list in output position.
    fn outputs(&self, list: &[OccId]) -> usize {
        list.iter().filter(|&&o| self.output(o)).count()
    }

    /// Whether a sequent stands for larger linear zones too.
    fn weak(&self, z: &Zone) -> bool {
        z.absorbs || self.affine
    }

    /// Whether a sequent is one of the mode: under the reading one member
    /// in output position, or none where it absorbs.
    fn shaped(&self, z: &Zone) -> bool {
        self.reading.is_none() || {
            let outputs = self.outputs(&z.linear);
            outputs == 1 || (outputs == 0 && z.absorbs)
        }
    }

    /// Whether two occurrences can stand in one sequent of a proof.
    fn coherent(&self, x: OccId, y: OccId) -> bool {
        let Some(w) = self.forest.lca(x, y) else {
            return true;
        };
        self.forest.kind(w) == Kind::Quest
            || self.copied[w.index()]
            || (w != x && w != y && self.forest.kind(w) == Kind::Par)
    }

    /// Whether every two members of a sorted list can stand in one sequent
    /// of a proof.
    fn coherent_list(&self, sorted: &[OccId]) -> bool {
        sorted.windows(2).all(|w| self.coherent(w[0], w[1]))
    }

    /// Places an occurrence an asynchronous phase meets.
    fn place(&self, phase: &mut Phase, o: OccId) {
        if self.stable(o) {
            phase.leaves.push(o);
            return;
        }
        match self.forest.kind(o) {
            Kind::Top => {
                phase.top.get_or_insert(o);
            }
            Kind::With => phase.withs.push(o),
            _ => phase.unary.push(o),
        }
    }

    /// The partners of a positive literal: the negative literals of its
    /// atom within the goal's subtrees, in the other position under the
    /// reading, from the unrestricted zone where they lie under a `?`, and
    /// from the linear one where they do not or the goal holds them. A
    /// literal outside the goal's subtrees stands in no sequent of a proof
    /// of the goal.
    fn partners(&self, p: OccId) -> Vec<Partner> {
        let forest = self.forest;
        let (Some(atom), Some(sign)) = (forest.atom(p), forest.sign(p)) else {
            return Vec::new();
        };
        let mut found = Vec::new();
        for &x in forest.literals(atom, !sign) {
            if !self.inside[x.index()] {
                continue;
            }
            if self.reading.is_some() && self.output(x) == self.output(p) {
                continue;
            }
            let quest = forest
                .parent(x)
                .is_some_and(|q| forest.kind(q) == Kind::Quest);
            if quest {
                found.push(Partner {
                    literal: x,
                    unrestricted: true,
                });
            }
            if !quest || self.goal.binary_search(&x).is_ok() {
                found.push(Partner {
                    literal: x,
                    unrestricted: false,
                });
            }
        }
        found
    }

    /// Writes the term of a rule from `start`, under the sides of its `⊕`
    /// given as each `⊕` and the operand taken: its steps in postorder,
    /// its holes and slots. Steps are found root first, the right operand
    /// before the left, which reversed is postorder.
    fn write(&self, start: Start<'_>, sides: &[(OccId, OccId)]) -> Rule {
        let forest = self.forest;
        let mut rule = Rule::default();
        let mut found: Vec<Step> = Vec::new();
        let mut releases = 0;
        let mut release = |phase: &mut Phase| {
            phase.release = releases;
            releases += 1;
        };
        let under_focus = |o: OccId, release: &mut dyn FnMut(&mut Phase)| {
            if self.positive(o) {
                Pending::Focus(o)
            } else {
                let mut phase = Phase::default();
                release(&mut phase);
                self.place(&mut phase, o);
                Pending::Phase(phase)
            }
        };
        let root = match start {
            Start::Focus(o) => Pending::Focus(o),
            Start::Copy(o) => {
                found.push(Step::Copy(o));
                under_focus(o, &mut release)
            }
            Start::Goal(members) => {
                let mut phase = Phase::default();
                release(&mut phase);
                for &o in members {
                    self.place(&mut phase, o);
                }
                Pending::Phase(phase)
            }
        };
        let mut stack = vec![root];
        while let Some(pending) = stack.pop() {
            match pending {
                Pending::Focus(o) => match forest.kind(o) {
                    Kind::Tensor => {
                        found.push(Step::Tensor(o));
                        let (l, r) = (forest.left(o), forest.right(o));
                        for c in [l, r].into_iter().flatten() {
                            stack.push(under_focus(c, &mut release));
                        }
                    }
                    Kind::Plus => {
                        let Some(&(_, child)) = sides.iter().find(|&&(p, _)| p == o) else {
                            unreachable!("every ⊕ under the focus has a side");
                        };
                        found.push(Step::Plus(o, child));
                        stack.push(under_focus(child, &mut release));
                    }
                    Kind::One => found.push(Step::One(o)),
                    Kind::Bang => {
                        found.push(Step::Bang(o));
                        let mut phase = Phase::default();
                        release(&mut phase);
                        if let Some(c) = forest.left(o) {
                            self.place(&mut phase, c);
                        }
                        stack.push(Pending::Phase(phase));
                    }
                    Kind::Atom | Kind::DualAtom => {
                        found.push(Step::Axiom(o, rule.slots.len()));
                        rule.slots.push(self.partners(o));
                    }
                    Kind::Zero | Kind::Par | Kind::Bot | Kind::With | Kind::Top | Kind::Quest => {
                        unreachable!("no focus on a 0 or a negative formula")
                    }
                },
                Pending::Phase(mut phase) => loop {
                    if let Some(t) = phase.top {
                        found.push(Step::Top(t));
                        break;
                    }
                    if let Some(o) = phase.unary.pop() {
                        match forest.kind(o) {
                            Kind::Par => {
                                found.push(Step::Par(o));
                                for c in forest.children(o) {
                                    self.place(&mut phase, c);
                                }
                            }
                            Kind::Bot => found.push(Step::Bot(o)),
                            Kind::Quest => {
                                found.push(Step::Quest(o));
                                phase.met.extend(forest.left(o));
                            }
                            _ => unreachable!("only ⅋, ⊥ and ? are unary"),
                        }
                        continue;
                    }
                    if let Some(o) = phase.withs.pop() {
                        found.push(Step::With(o));
                        let mut left = phase.clone();
                        left.place_child(self, forest.left(o));
                        phase.place_child(self, forest.right(o));
                        stack.push(Pending::Phase(left));
                        continue;
                    }
                    phase.leaves.sort_unstable();
                    phase.met.sort_unstable();
                    found.push(Step::Hole(rule.holes.len()));
                    rule.holes.push(Hole {
                        leaves: phase.leaves,
                        met: phase.met,
                        release: phase.release,
                    });
                    break;
                },
            }
        }
        found.reverse();
        rule.steps = found;
        rule
    }
}

impl Phase {
    /// Places an operand, where there is one.
    fn place_child(&mut self, calculus: &Calculus<'_>, o: Option<OccId>) {
        if let Some(o) = o {
            calculus.place(self, o);
        }
    }
}

/// Inserts into a sorted list, keeping repeats.
fn insert(list: &mut Vec<OccId>, o: OccId) {
    let at = list.partition_point(|&x| x <= o);
    list.insert(at, o);
}

/// Returns the sum of two sorted multisets.
fn sum(a: &[OccId], b: &[OccId]) -> Vec<OccId> {
    let mut out = Vec::with_capacity(a.len() + b.len());
    let (mut i, mut j) = (0, 0);
    while i < a.len() || j < b.len() {
        if j == b.len() || (i < a.len() && a[i] <= b[j]) {
            out.push(a[i]);
            i += 1;
        } else {
            out.push(b[j]);
            j += 1;
        }
    }
    out
}

/// Returns the union of two sorted sets.
fn union(a: &[OccId], b: &[OccId]) -> Vec<OccId> {
    let mut out = sum(a, b);
    out.dedup();
    out
}

/// Returns the members of the sorted multiset `of` beyond those of the
/// sorted `within`, with repeats.
fn excess(of: &[OccId], within: &[OccId]) -> Vec<OccId> {
    let mut out = Vec::new();
    let mut j = 0;
    for &o in of {
        while j < within.len() && within[j] < o {
            j += 1;
        }
        if j < within.len() && within[j] == o {
            j += 1;
        } else {
            out.push(o);
        }
    }
    out
}

/// Whether the sorted multiset `small` lies within the sorted `large`.
fn within(small: &[OccId], large: &[OccId]) -> bool {
    let mut j = 0;
    for &o in small {
        while j < large.len() && large[j] < o {
            j += 1;
        }
        if j == large.len() || large[j] != o {
            return false;
        }
        j += 1;
    }
    true
}

/// Whether a list is sorted, without repeats where `set`.
fn sorted(list: &[OccId], set: bool) -> bool {
    list.windows(2)
        .all(|w| w[0] < w[1] || (!set && w[0] == w[1]))
}

impl Calculus<'_> {
    /// Takes a principal's subformula out of a premise's linear zone: it
    /// is there, or the premise absorbs it (one in output position only
    /// where the zone has none), or in affine mode it is weakened in (one
    /// in output position never).
    fn take(&self, z: &mut Zone, o: OccId) -> bool {
        if let Ok(at) = z.linear.binary_search(&o) {
            z.linear.remove(at);
            return true;
        }
        if z.absorbs {
            return !(self.output(o) && self.outputs(&z.linear) > 0);
        }
        self.affine && !self.output(o)
    }

    /// Joins the premises of a `&`, their subformulas taken: equal linear
    /// zones; one that absorbs within one that does not; two that absorb,
    /// to the larger count of each member; in affine mode either side
    /// weakened up to that, members in output position never.
    fn join(&self, l: Zone, r: Zone) -> Option<Zone> {
        let most = sum(&l.linear, &excess(&r.linear, &l.linear));
        let joined = match (l.absorbs, r.absorbs) {
            (true, true) => true,
            (false, false) => l.linear == r.linear,
            (true, false) => within(&l.linear, &r.linear),
            (false, true) => within(&r.linear, &l.linear),
        };
        if !joined {
            let weakened = |side: &Zone| {
                side.absorbs || excess(&most, &side.linear).iter().all(|&o| !self.output(o))
            };
            if !(self.affine && weakened(&l) && weakened(&r)) {
                return None;
            }
        }
        Some(Zone {
            linear: most,
            unrestricted: union(&l.unrestricted, &r.unrestricted),
            absorbs: l.absorbs && r.absorbs,
        })
    }

    /// Evaluates a rule's term over the premises of its holes and the
    /// partners of its slots, by index into `set` and into each slot's
    /// list, and returns the sequent derived, or `None` where a step's
    /// condition fails; adds to `units` a unit for each step and each
    /// member of the zone it yields.
    fn derive(
        &self,
        rule: &Rule,
        set: &[Zone],
        premises: &[usize],
        partners: &[usize],
        units: &mut u64,
    ) -> Option<Zone> {
        let forest = self.forest;
        let mut stack: Vec<Zone> = Vec::new();
        for &step in &rule.steps {
            let z = match step {
                Step::Axiom(p, slot) => {
                    let partner = rule.slots[slot][partners[slot]];
                    let mut z = Zone {
                        linear: vec![p],
                        ..Zone::default()
                    };
                    if partner.unrestricted {
                        z.unrestricted.push(partner.literal);
                    } else {
                        insert(&mut z.linear, partner.literal);
                    }
                    z
                }
                Step::One(o) => Zone {
                    linear: vec![o],
                    ..Zone::default()
                },
                Step::Top(o) => Zone {
                    linear: vec![o],
                    unrestricted: Vec::new(),
                    absorbs: true,
                },
                Step::Hole(h) => set[premises[h]].clone(),
                Step::Tensor(o) => {
                    let mut r = stack.pop()?;
                    let mut l = stack.pop()?;
                    if !self.take(&mut l, forest.left(o)?) || !self.take(&mut r, forest.right(o)?) {
                        return None;
                    }
                    let mut z = Zone {
                        linear: sum(&l.linear, &r.linear),
                        unrestricted: union(&l.unrestricted, &r.unrestricted),
                        absorbs: l.absorbs || r.absorbs,
                    };
                    insert(&mut z.linear, o);
                    z
                }
                Step::Plus(o, child) => {
                    let mut z = stack.pop()?;
                    if !self.take(&mut z, child) {
                        return None;
                    }
                    insert(&mut z.linear, o);
                    z
                }
                Step::Bang(o) => {
                    let mut z = stack.pop()?;
                    if !self.take(&mut z, forest.left(o)?) || !z.linear.is_empty() {
                        return None;
                    }
                    z.absorbs = false;
                    insert(&mut z.linear, o);
                    z
                }
                Step::Par(o) => {
                    let mut z = stack.pop()?;
                    if !self.take(&mut z, forest.left(o)?) || !self.take(&mut z, forest.right(o)?) {
                        return None;
                    }
                    insert(&mut z.linear, o);
                    z
                }
                Step::Bot(o) => {
                    let mut z = stack.pop()?;
                    insert(&mut z.linear, o);
                    z
                }
                Step::Quest(o) => {
                    let mut z = stack.pop()?;
                    let a = forest.left(o)?;
                    z.unrestricted.retain(|&x| x != a);
                    insert(&mut z.linear, o);
                    z
                }
                Step::With(o) => {
                    let mut r = stack.pop()?;
                    let mut l = stack.pop()?;
                    if !self.take(&mut l, forest.left(o)?) || !self.take(&mut r, forest.right(o)?) {
                        return None;
                    }
                    let mut z = self.join(l, r)?;
                    insert(&mut z.linear, o);
                    z
                }
                Step::Copy(o) => {
                    let mut z = stack.pop()?;
                    if !self.take(&mut z, o) {
                        return None;
                    }
                    z.unrestricted = union(&z.unrestricted, &[o]);
                    z
                }
            };
            *units += 1 + (z.linear.len() + z.unrestricted.len()) as u64;
            if !self.shaped(&z) {
                return None;
            }
            stack.push(z);
        }
        let z = stack.pop()?;
        stack.is_empty().then_some(z)
    }

    /// Calls `each` with every rule: a focus on every positive formula a
    /// stable sequent can hold (whose parent is `⅋`, `&`, `!` or none
    /// within the goal) and a copy of every subformula of a `?` but a
    /// negative literal, each under every choice of the sides of its
    /// `⊕`, a choice that reaches a `0` left out. Counts each choice
    /// looked at as work.
    fn rules(
        &self,
        work: &mut Work<'_>,
        each: &mut dyn FnMut(&Rule, &mut Work<'_>) -> Result<(), Failed>,
    ) -> Result<(), Failed> {
        let forest = self.forest;
        let mut previous = None;
        for &g in &self.goal {
            if previous.replace(g) == Some(g) {
                continue;
            }
            for o in forest.subtree(g) {
                let parent = forest.parent(o).filter(|_| o != g).map(|p| forest.kind(p));
                let mut starts = Vec::new();
                if self.positive(o)
                    && matches!(parent, None | Some(Kind::Par | Kind::With | Kind::Bang))
                {
                    starts.push(Start::Focus(o));
                }
                if parent == Some(Kind::Quest) && !(forest.is_literal(o) && !self.positive(o)) {
                    starts.push(Start::Copy(o));
                }
                for start in starts {
                    let mut open: Vec<Vec<(OccId, OccId)>> = vec![Vec::new()];
                    'plans: while let Some(sides) = open.pop() {
                        work.add(1)?;
                        let mut walk = vec![o];
                        while let Some(x) = walk.pop() {
                            match forest.kind(x) {
                                Kind::Tensor => {
                                    walk.extend(forest.children(x).filter(|&c| self.positive(c)))
                                }
                                Kind::Plus => match sides.iter().find(|&&(p, _)| p == x) {
                                    Some(&(_, child)) => {
                                        if self.positive(child) {
                                            walk.push(child);
                                        }
                                    }
                                    None => {
                                        for child in forest.children(x) {
                                            let mut more = sides.clone();
                                            more.push((x, child));
                                            open.push(more);
                                        }
                                        continue 'plans;
                                    }
                                },
                                Kind::Zero => continue 'plans,
                                _ => {}
                            }
                        }
                        let rule = self.write(start, &sides);
                        work.add(rule.steps.len() as u64)?;
                        each(&rule, work)?;
                    }
                }
            }
        }
        Ok(())
    }
}

/// The work of a check, polled against its stop.
struct Work<'s> {
    /// The units since the last poll.
    unpolled: u64,
    /// The stop, told the units at each poll.
    stop: &'s mut dyn FnMut(u64) -> bool,
}

impl Work<'_> {
    /// Counts units of work, polling the stop once enough are done.
    fn add(&mut self, units: u64) -> Result<(), Failed> {
        self.unpolled = self.unpolled.saturating_add(units);
        if self.unpolled >= UNITS_PER_POLL && (self.stop)(std::mem::take(&mut self.unpolled)) {
            return Err(Failed::Stopped);
        }
        Ok(())
    }
}

/// The certificate's sequents, indexed by what the check asks of them.
struct Set {
    /// The sequents.
    zones: Vec<Zone>,
    /// The sequents that stand for their linear zone alone, by it.
    exact: HashMap<Vec<OccId>, Vec<usize>>,
    /// Those, by every member of their linear zone.
    holding: HashMap<OccId, Vec<usize>>,
    /// Those with an empty linear zone.
    exact_empty: Vec<usize>,
    /// The sequents that stand for larger linear zones too.
    weak: Vec<usize>,
    /// Those, by the first member of their linear zone.
    weak_first: HashMap<OccId, Vec<usize>>,
    /// Those with an empty linear zone.
    weak_empty: Vec<usize>,
}

impl Set {
    /// Indexes the sequents.
    fn new(calculus: &Calculus<'_>, zones: Vec<Zone>) -> Self {
        let mut set = Set {
            zones: Vec::new(),
            exact: HashMap::default(),
            holding: HashMap::default(),
            exact_empty: Vec::new(),
            weak: Vec::new(),
            weak_first: HashMap::default(),
            weak_empty: Vec::new(),
        };
        for (i, z) in zones.iter().enumerate() {
            if calculus.weak(z) {
                set.weak.push(i);
                match z.linear.first() {
                    Some(&o) => set.weak_first.entry(o).or_default().push(i),
                    None => set.weak_empty.push(i),
                }
            } else {
                set.exact.entry(z.linear.clone()).or_default().push(i);
                let mut members = z.linear.clone();
                members.dedup();
                for o in members {
                    set.holding.entry(o).or_default().push(i);
                }
                if z.linear.is_empty() {
                    set.exact_empty.push(i);
                }
            }
        }
        set.zones = zones;
        set
    }

    /// The sequents that can go into a hole: those that stand for their
    /// zone alone and hold the hole's members, and every one that stands
    /// for larger zones. Any other fails at the step that takes the member
    /// it lacks. Counts a unit for each sequent looked at and each member
    /// compared.
    fn candidates(&self, hole: &Hole, work: &mut Work<'_>) -> Result<Vec<usize>, Failed> {
        let mut out: Vec<usize> = match hole.leaves.first() {
            None => self.exact.values().flatten().copied().collect(),
            Some(o) => {
                let holding = self.holding.get(o).map_or(&[][..], Vec::as_slice);
                let compared: usize = holding.iter().map(|&i| self.zones[i].linear.len()).sum();
                work.add((holding.len() + compared) as u64)?;
                holding
                    .iter()
                    .copied()
                    .filter(|&i| within(&hole.leaves, &self.zones[i].linear))
                    .collect()
            }
        };
        out.extend(&self.weak);
        work.add(out.len() as u64)?;
        Ok(out)
    }

    /// Whether a sequent of the set stands for every sequent `c` stands
    /// for: one that stands for its linear zone alone, for `c`'s with an
    /// unrestricted zone within `c`'s, where `c` does not stand for more;
    /// one that stands for larger zones, for `c`'s with both zones within
    /// `c`'s (its members in output position, which `c`'s linear zone
    /// holds at most one of, are those of `c` or absorbed, since the
    /// set's sequents are shaped).
    fn subsumes(
        &self,
        calculus: &Calculus<'_>,
        c: &Zone,
        work: &mut Work<'_>,
    ) -> Result<bool, Failed> {
        if !calculus.weak(c)
            && let Some(list) = self.exact.get(&c.linear)
        {
            for &i in list {
                work.add(1 + self.zones[i].unrestricted.len() as u64)?;
                if within(&self.zones[i].unrestricted, &c.unrestricted) {
                    return Ok(true);
                }
            }
        }
        let mut firsts = c.linear.clone();
        firsts.dedup();
        let lists = firsts
            .iter()
            .filter_map(|o| self.weak_first.get(o))
            .chain([&self.weak_empty]);
        for list in lists {
            for &i in list {
                let s = &self.zones[i];
                work.add(1 + (s.linear.len() + s.unrestricted.len()) as u64)?;
                if within(&s.linear, &c.linear) && within(&s.unrestricted, &c.unrestricted) {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
}

/// Checks a refutation by saturation of the goal `goal` of the forest in
/// `mode` (the module's documentation says why it refutes), polling `stop`
/// with its units of work: the certificate names a positive literal for
/// each atom and occurrences of the forest in sorted zones, its sequents
/// are sequents of the mode, every rule applied to every choice of its
/// premises among them and of its axioms' partners derives a sequent that
/// cannot stand in a proof or that one of them stands for, so does Mix
/// where the mode has it, and some stable sequent of the goal's phase is
/// one none of them stands for.
pub(crate) fn check(
    forest: &Forest,
    goal: &[OccId],
    mode: Mode,
    saturated: &Saturated,
    stop: &mut dyn FnMut(u64) -> bool,
) -> Result<(), Failed> {
    const INVARIANT: &str = "saturation";
    let not_applicable = |why| {
        Failed::Fault(RefutationFault::NotApplicable {
            invariant: INVARIANT,
            why,
        })
    };
    if mode.is_ordered() {
        return Err(not_applicable(
            "its rules are those of the commutative modes",
        ));
    }
    if mode.intuitionistic && mode.has_mix() {
        return Err(not_applicable("intuitionistic mode has no Mix"));
    }
    if saturated.positive.len() != forest.sequent().atom_names().len() {
        return Err(RefutationFault::Mismatch.into());
    }
    let len = forest.len();
    let ours =
        |list: &[OccId], set: bool| sorted(list, set) && list.iter().all(|o| o.index() < len);
    if !saturated
        .sequents
        .iter()
        .all(|s| ours(&s.linear, false) && ours(&s.unrestricted, true))
    {
        return Err(RefutationFault::Mismatch.into());
    }
    let reading = if mode.intuitionistic {
        Some(
            Reading::of_mode(forest, mode)
                .map_err(|_| not_applicable("the sequent has no intuitionistic reading"))?,
        )
    } else {
        None
    };
    let mut copied = vec![false; len];
    for i in 0..len {
        let o = OccId::new(i as u32);
        if let Some(p) = forest.parent(o) {
            copied[i] = copied[p.index()] || forest.kind(p) == Kind::Quest;
        }
    }
    let mut members = goal.to_vec();
    members.sort_unstable();
    let mut inside = vec![false; len];
    for &g in goal {
        for o in forest.subtree(g) {
            inside[o.index()] = true;
        }
    }
    let mut globals = Vec::new();
    let mut walk = members.clone();
    while let Some(o) = walk.pop() {
        match forest.kind(o) {
            Kind::Par => walk.extend(forest.children(o)),
            Kind::Quest => globals.extend(forest.left(o)),
            _ => {}
        }
    }
    globals.sort_unstable();
    globals.dedup();
    let calculus = Calculus {
        forest,
        positive: &saturated.positive,
        reading,
        affine: mode.is_affine(),
        copied,
        inside,
        goal: members,
        globals,
    };
    if !calculus.coherent_list(&calculus.goal) {
        return Err(not_applicable(
            "two members of the goal can stand in no sequent of a proof together",
        ));
    }
    let zones: Vec<Zone> = saturated
        .sequents
        .iter()
        .map(|s: &Stable| Zone {
            linear: s.linear.clone(),
            unrestricted: s.unrestricted.clone(),
            absorbs: s.absorbs,
        })
        .collect();
    if !zones.iter().all(|z| calculus.shaped(z)) {
        return Err(RefutationFault::Unshaped.into());
    }
    let set = Set::new(&calculus, zones);
    let mut work = Work { unpolled: 0, stop };
    // A sequent derived, its globals left out: it cannot stand in a proof,
    // or a sequent of the set stands for it.
    let closed = |mut c: Zone, work: &mut Work<'_>| -> Result<(), Failed> {
        c.unrestricted
            .retain(|o| calculus.globals.binary_search(o).is_err());
        work.add((c.linear.len() + c.unrestricted.len()) as u64)?;
        let all = sum(&c.linear, &c.unrestricted);
        if !calculus.coherent_list(&all) || set.subsumes(&calculus, &c, work)? {
            return Ok(());
        }
        Err(RefutationFault::NotClosed.into())
    };
    calculus.rules(&mut work, &mut |rule, work| {
        let lists = rule
            .holes
            .iter()
            .map(|h| set.candidates(h, work))
            .collect::<Result<Vec<Vec<usize>>, Failed>>()?;
        if lists.iter().any(Vec::is_empty) || rule.slots.iter().any(Vec::is_empty) {
            return Ok(());
        }
        // What each candidate brings beside the hole's members, for the
        // premises of one release, whose `&` steps join them: two that
        // stand for their zone alone must bring the same.
        let contexts: Vec<Vec<Vec<OccId>>> = rule
            .holes
            .iter()
            .zip(&lists)
            .map(|(h, list)| {
                list.iter()
                    .map(|&i| excess(&set.zones[i].linear, &h.leaves))
                    .collect()
            })
            .collect();
        let compared: usize = lists
            .iter()
            .flatten()
            .map(|&i| set.zones[i].linear.len())
            .sum();
        work.add(compared as u64)?;
        let fits = |at: &[usize], j: usize| {
            let p = lists[j][at[j]];
            calculus.weak(&set.zones[p])
                || (0..j).all(|i| {
                    let q = lists[i][at[i]];
                    rule.holes[i].release != rule.holes[j].release
                        || calculus.weak(&set.zones[q])
                        || contexts[i][at[i]] == contexts[j][at[j]]
                })
        };
        let holes = rule.holes.len();
        let mut at = vec![0usize; holes];
        let mut premises = vec![0usize; holes];
        let mut j = 0;
        loop {
            if j == holes {
                for (p, (&a, list)) in premises.iter_mut().zip(at.iter().zip(&lists)) {
                    *p = list[a];
                }
                let mut partners = vec![0usize; rule.slots.len()];
                loop {
                    let mut units = 0;
                    let derived =
                        calculus.derive(rule, &set.zones, &premises, &partners, &mut units);
                    work.add(units)?;
                    if let Some(c) = derived {
                        closed(c, work)?;
                    }
                    let mut more = false;
                    for (k, p) in partners.iter_mut().enumerate() {
                        *p += 1;
                        if *p < rule.slots[k].len() {
                            more = true;
                            break;
                        }
                        *p = 0;
                    }
                    if !more {
                        break;
                    }
                }
                if holes == 0 {
                    return Ok(());
                }
                j = holes - 1;
                at[j] += 1;
            }
            while at[j] < lists[j].len() && !fits(&at, j) {
                work.add(1 + j as u64)?;
                at[j] += 1;
            }
            if at[j] < lists[j].len() {
                j += 1;
            } else if j == 0 {
                return Ok(());
            } else {
                at[j] = 0;
                j -= 1;
                at[j] += 1;
            }
        }
    })?;
    if mode.has_mix() && !calculus.affine {
        for i in 0..set.zones.len() {
            for k in i..set.zones.len() {
                let (l, r) = (&set.zones[i], &set.zones[k]);
                closed(
                    Zone {
                        linear: sum(&l.linear, &r.linear),
                        unrestricted: union(&l.unrestricted, &r.unrestricted),
                        absorbs: l.absorbs || r.absorbs,
                    },
                    &mut work,
                )?;
            }
        }
    }
    // The goal's phase: some stable sequent of it is one no sequent of
    // the set stands for, which a proof would need.
    let phase = calculus.write(Start::Goal(&calculus.goal), &[]);
    let fills = |s: &Zone, hole: &Hole| {
        within(&s.unrestricted, &hole.met)
            && within(&s.linear, &hole.leaves)
            && (calculus.weak(s) || s.linear.len() == hole.leaves.len())
            && (s.absorbs
                || excess(&hole.leaves, &s.linear)
                    .iter()
                    .all(|&o| !calculus.output(o)))
    };
    let mut open = false;
    for hole in &phase.holes {
        let mut filled = false;
        for s in &set.zones {
            work.add(1 + (s.linear.len() + s.unrestricted.len()) as u64)?;
            if fills(s, hole) {
                filled = true;
                break;
            }
        }
        if !filled {
            open = true;
            break;
        }
    }
    if !open {
        return Err(RefutationFault::Filled.into());
    }
    Ok(())
}

#[cfg(all(test, feature = "parse"))]
mod tests {
    use super::{Failed, check};
    use crate::occurrences::{Forest, OccId, Sign};
    use crate::search::{
        Engine, Options, Refutation, RefutationFault, Saturated, Stable, Verdict, prove,
    };
    use crate::{Mode, Sequent};

    /// The forest of a sequent and the saturated set the inverse method
    /// refutes it with in `mode`.
    fn saturated(text: &str, mode: Mode) -> (Forest, Saturated) {
        let sequent: Sequent = text.parse().unwrap();
        let options = Options::default().with_engine(Some(Engine::Inverse));
        let outcome = prove(&sequent, mode, &options).unwrap();
        let Verdict::Unprovable(disproof) = outcome.verdict else {
            panic!("{text} in {mode}: {:?}", outcome.verdict);
        };
        let Refutation::Saturated(set) = disproof.refutation().clone() else {
            panic!("{text} in {mode}: {disproof}");
        };
        (Forest::new(&sequent).unwrap(), set)
    }

    /// Checks a set as the refutation of a sequent's roots in `mode`.
    fn checked(forest: &Forest, mode: Mode, set: &Saturated) -> Result<(), Failed> {
        check(forest, forest.roots(), mode, set, &mut |_| false)
    }

    /// A goal off the roots: a literal of the atom outside the goal's
    /// subtrees is no partner, and the engine's set passes (the panel's
    /// witness, refused as not closed while the check took every literal
    /// of the forest).
    #[test]
    fn a_goal_off_the_roots_is_refuted() {
        let sequent: Sequent = "|- ((?a par ?~a) par (b * ~b)) * (a * ~a)".parse().unwrap();
        let forest = Forest::new(&sequent).unwrap();
        let members = [crate::Member::new(1)];
        let goal = crate::search::Goal::new(&forest, &members).unwrap();
        let options = Options::default().with_engine(Some(Engine::Inverse));
        let outcome = crate::search::prove_goal(
            goal,
            Mode::CLASSICAL,
            &options,
            &crate::Limits::default(),
            |_| false,
        )
        .unwrap();
        let Verdict::Unprovable(disproof) = outcome.verdict else {
            panic!("{:?}", outcome.verdict);
        };
        assert!(matches!(disproof.refutation(), Refutation::Saturated(_)));
        assert_eq!(disproof.check(), Ok(()));
    }

    /// The sets the inverse method refutes with pass in every commutative
    /// mode.
    #[test]
    fn the_engines_sets_pass() {
        for (text, mode) in [
            ("|- ?~a * ?~a, !a", Mode::CLASSICAL),
            ("|- ?~a * ?~a, !a", Mode::CLASSICAL.with_mix()),
            ("|- ?~a * ?~a, !a", Mode::CLASSICAL.with_affine()),
            ("|- ?(~a * ~a), !a", Mode::CLASSICAL),
            ("|- a, ~a, !b, ?~b", Mode::CLASSICAL),
            ("!(a -o b), a |- b * b", Mode::INTUITIONISTIC),
            ("!(a -o b), !(b -o c), a |- c * c", Mode::INTUITIONISTIC),
            ("!(a -o b), a |- b * b", Mode::INTUITIONISTIC.with_affine()),
        ] {
            let (forest, set) = saturated(text, mode);
            assert_eq!(checked(&forest, mode, &set), Ok(()), "{text} in {mode}");
        }
    }

    /// A set is refused for each condition of the check it breaks: a
    /// polarity for another number of atoms, a member past the forest or a
    /// zone out of order, a sequent of another shape than the mode's, a
    /// rule (one without premises among them) or Mix whose sequent derived
    /// none of the set stands for, a set that gives every stable sequent
    /// of the goal's phase, and an ordered mode.
    #[test]
    fn a_wrong_set_is_refused() {
        let fault = |f| Err(Failed::Fault(f));
        let intuitionistic = Mode::INTUITIONISTIC;
        let (forest, set) = saturated("!(a -o b), a |- b * b", intuitionistic);
        let refused = |set: Saturated| checked(&forest, intuitionistic, &set);
        let with = |change: &dyn Fn(&mut Saturated)| {
            let mut set = set.clone();
            change(&mut set);
            set
        };
        let past = OccId::new(forest.len() as u32);
        assert_eq!(
            refused(with(&|s| s.positive.push(Sign::Atom))),
            fault(RefutationFault::Mismatch)
        );
        assert_eq!(
            refused(with(&|s| s.sequents[0].linear.push(past))),
            fault(RefutationFault::Mismatch)
        );
        assert_eq!(
            refused(with(&|s| {
                let z = &mut s.sequents[0].unrestricted;
                z.extend([OccId::new(1), OccId::new(0)]);
            })),
            fault(RefutationFault::Mismatch)
        );
        // Two members in output position: the goal's `b ⊗ b` twice.
        let goal = forest.roots()[forest.roots().len() - 1];
        assert_eq!(
            refused(with(&|s| s.sequents.push(Stable::new(
                vec![goal, goal],
                Vec::new(),
                false
            )))),
            fault(RefutationFault::Unshaped)
        );
        // No sequent at all: the axioms, which have no premises, derive
        // ones none stands for.
        assert_eq!(
            refused(with(&|s| s.sequents.clear())),
            fault(RefutationFault::NotClosed)
        );
        // Each sequent the search kept left out: one at least is needed.
        assert!((0..set.sequents.len()).any(|i| refused(with(&|s| {
            s.sequents.remove(i);
        })) == fault(RefutationFault::NotClosed)));
        // One sequent that absorbs everything: closed, and it gives the
        // goal every stable sequent it needs.
        assert_eq!(
            refused(with(
                &|s| s.sequents = vec![Stable::new(Vec::new(), Vec::new(), true)]
            )),
            fault(RefutationFault::Filled)
        );
        assert_eq!(
            checked(&forest, Mode::CYCLIC, &set),
            fault(RefutationFault::NotApplicable {
                invariant: "saturation",
                why: "its rules are those of the commutative modes",
            })
        );
        // The set of `⊢ a, ~a` and `⊢ !b ; ~b` is closed under the rules,
        // but not under Mix, which joins them into a sequent of the goal's.
        let (forest, set) = saturated("|- a, ~a, !b, ?~b", Mode::CLASSICAL);
        assert_eq!(
            checked(&forest, Mode::CLASSICAL.with_mix(), &set),
            fault(RefutationFault::NotClosed)
        );
    }
}
