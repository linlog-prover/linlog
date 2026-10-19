// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The derived rules of the forward search, compiled from the forest before
//! it starts: one rule per formula a focus can begin on and per choice of
//! the `⊕` sides under it. A rule is the proof term of one focusing phase,
//! the focus and the asynchronous phases of what it releases, read forward:
//! its steps in postorder, with a hole where a premise goes (a stable
//! sequent derived before) and a slot where an axiom's partner, a negative
//! literal, is chosen. The goal is compiled the same way into its
//! asynchronous phase, whose holes the search fills.

use super::super::Reason;
use super::super::memory::{Charged, bytes_of};
use crate::hash::HashMap;
use crate::occurrences::{Forest, OccId, Reading, Side};
use crate::proofs::Branch;
use crate::sequents::Kind;
use std::ops::Range;

/// A step of a rule's term, in postorder: a step's operands are the
/// results of the steps before it, as on a stack.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Step {
    /// `⊢ p, x`: the axiom on the positive literal and the partner chosen
    /// for the slot.
    Axiom(OccId, u32),
    /// `⊢ 1`.
    One(OccId),
    /// `⊢ ⊤` with any context.
    Top(OccId),
    /// The premise put into the hole.
    Hole(u32),
    /// `⊗` on its two operands.
    Tensor(OccId),
    /// `⊕` on the operand, its side as given.
    Plus(OccId, Branch),
    /// `!` on its operand, which must have nothing else.
    Bang(OccId),
    /// `⅋` on its operand.
    Par(OccId),
    /// `⊥` on its operand.
    Bot(OccId),
    /// `&` on its two operands.
    With(OccId),
    /// `?`: the subformula leaves the unrestricted zone.
    Quest(OccId),
    /// A copy of the formula into the unrestricted zone.
    Copy(OccId),
}

/// The partner of a positive literal in an axiom: a negative literal of the
/// same atom, in the linear zone, or in the unrestricted one when it is the
/// subformula of a `?` (a copy above the axiom).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Partner {
    /// The negative literal.
    pub(super) x: OccId,
    /// Whether it comes from the unrestricted zone.
    pub(super) theta: bool,
}

/// A rule: ranges of the steps, holes and slots of [`Rules`].
#[derive(Clone, Debug)]
pub(super) struct Rule {
    /// Its steps.
    pub(super) steps: Range<u32>,
    /// Its holes, in the order their steps come.
    pub(super) holes: Range<u32>,
    /// Its slots, in the order their steps come.
    pub(super) slots: Range<u32>,
}

/// A hole of a rule: the occurrences the term takes from the premise put
/// there, which a premise must hold unless it may absorb or weaken them,
/// and the subformulas of the `?` the phase met before it, which the
/// premise's unrestricted zone may hold.
#[derive(Clone, Debug)]
pub(super) struct Hole {
    /// The rule it belongs to.
    pub(super) rule: u32,
    /// Its place among the rule's holes.
    pub(super) index: u32,
    /// The release of the rule it belongs to: the holes of one release
    /// share their context, the others' contexts are joined side by side.
    pub(super) group: u32,
    /// The range of its members in [`Rules::leaves`], sorted.
    pub(super) leaves: Range<u32>,
    /// The range of the subformulas of `?` in [`Rules::leaves`], sorted.
    pub(super) theta: Range<u32>,
}

/// Every rule of the search, or the goal's phase, which is one rule whose
/// holes the goal's stable sequents are.
#[derive(Debug, Default)]
pub(super) struct Rules {
    /// The rules.
    pub(super) all: Vec<Rule>,
    /// Every rule's steps.
    pub(super) steps: Vec<Step>,
    /// Every rule's holes.
    pub(super) holes: Vec<Hole>,
    /// The members of the holes, each hole's range sorted.
    pub(super) leaves: Vec<OccId>,
    /// Every rule's slots, each a range of [`Rules::partners`].
    pub(super) slots: Vec<Range<u32>>,
    /// The partners of the slots.
    pub(super) partners: Vec<Partner>,
    /// The range of [`Rules::partners`] each positive literal takes under a
    /// focus on a formula, which every rule with that focus shares.
    lists: HashMap<(OccId, OccId), Range<u32>>,
    /// The holes by their first member.
    pub(super) by_leaf: HashMap<OccId, Vec<u32>>,
    /// The holes that take no member.
    pub(super) open: Vec<u32>,
}

/// What a forest's occurrences are to the search: their polarity under the
/// bias, the subformulas of `?` the goal's asynchronous phase puts into the
/// unrestricted zone on every branch, and the reading.
pub(super) struct Shape<'a> {
    /// The forest.
    pub(super) forest: &'a Forest,
    /// The sign of each atom's positive literal.
    signs: Box<[crate::occurrences::Sign]>,
    /// The intuitionistic reading, in intuitionistic mode.
    pub(super) reading: Option<&'a Reading<'a>>,
    /// Whether weakening is allowed.
    pub(super) affine: bool,
    /// The members of the goal, sorted.
    pub(super) goal: Vec<OccId>,
    /// The subformulas of the `?` that the goal reaches through `⅋` alone,
    /// sorted: in the unrestricted zone of every sequent of a proof.
    pub(super) globals: Vec<OccId>,
}

impl<'a> Shape<'a> {
    /// The shape of a goal of the forest under the atoms' signs.
    pub(super) fn new(
        forest: &'a Forest,
        goal: &[OccId],
        signs: Box<[crate::occurrences::Sign]>,
        reading: Option<&'a Reading<'a>>,
        affine: bool,
    ) -> Self {
        let mut members = goal.to_vec();
        members.sort_unstable();
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
        Self {
            forest,
            signs,
            reading,
            affine,
            goal: members,
            globals,
        }
    }

    /// Whether an occurrence is positive: `⊗ 1 ⊕ 0 !` and the literals the
    /// bias makes positive.
    pub(super) fn positive(&self, o: OccId) -> bool {
        match self.forest.kind(o) {
            Kind::Tensor | Kind::One | Kind::Plus | Kind::Zero | Kind::Bang => true,
            Kind::Atom | Kind::DualAtom => {
                let atom = self.forest.atom(o).expect("a literal has an atom");
                self.forest.sign(o) == Some(self.signs[atom.index()])
            }
            Kind::Par | Kind::Bot | Kind::With | Kind::Top | Kind::Quest => false,
        }
    }

    /// Whether an occurrence stays in a stable sequent's linear zone when an
    /// asynchronous phase meets it: a positive formula or a negative literal.
    fn stable(&self, o: OccId) -> bool {
        self.positive(o) || self.forest.is_literal(o)
    }

    /// Whether two occurrences can stand in one sequent of a proof of the
    /// goal, the goal's own members among them ([`coherent`]).
    pub(super) fn coherent(&self, x: OccId, y: OccId) -> bool {
        coherent(self.forest, x, y)
    }

    /// Whether an occurrence is in output position under the reading.
    pub(super) fn output(&self, o: OccId) -> bool {
        self.reading.is_some_and(|r| r.position(o) == Side::Output)
    }
}

/// Whether an occurrence lies inside the subformula of a `?`, whose copies
/// can repeat it.
fn copied(forest: &Forest, o: OccId) -> bool {
    let mut a = o;
    while let Some(p) = forest.parent(a) {
        if forest.kind(p) == Kind::Quest {
            return true;
        }
        a = p;
    }
    false
}

/// Whether two occurrences can stand in one sequent of a proof of a goal
/// whose own members can: outside the subformulas of `?`, where nothing is
/// repeated, two members of one sequent are distinct, neither lies below
/// the other, and the connective that joins them is a `⅋`, since a rule
/// decomposed it below them, and a `⊗`, `&` or `⊕` above both sends them to
/// different premises or keeps one side. Two joined by a `?` formula (the
/// formula twice, or it beside a member of its subformula, as a goal is
/// after a contraction and a dereliction) are let through: no stable
/// sequent holds a `?` formula, which a goal's phase decomposes first, and
/// all below its subformula may repeat. A goal of the sequent's roots
/// meets that, and so does every goal of a derivation of them.
pub(super) fn coherent(forest: &Forest, x: OccId, y: OccId) -> bool {
    let Some(w) = forest.lca(x, y) else {
        return true;
    };
    if forest.kind(w) == Kind::Quest || copied(forest, w) {
        return true;
    }
    w != x && w != y && forest.kind(w) == Kind::Par
}

/// Whether every two members of a list sorted in the forest's order can
/// stand in one sequent of a proof: those consecutive are, the common
/// ancestors of a list sorted in preorder being those of its neighbours.
pub(super) fn coherent_list(forest: &Forest, sorted: &[OccId]) -> bool {
    sorted.windows(2).all(|w| coherent(forest, w[0], w[1]))
}

/// A subtree of a rule's term still to be written.
enum Pending {
    /// The focus on a positive formula.
    Focus(OccId),
    /// An asynchronous phase.
    Async(Phase),
    /// A copy of the subformula of a `?` and the phase it begins.
    Copy(OccId),
}

/// The state of an asynchronous phase: the negative formulas left to
/// decompose, the stable members met, and the subformulas of `?` moved into
/// the unrestricted zone.
#[derive(Clone, Default)]
struct Phase {
    /// A `⊤` met, which closes the phase.
    top: Option<OccId>,
    /// The `⅋`, `⊥` and `?` left.
    unary: Vec<OccId>,
    /// The `&` left, decomposed last.
    withs: Vec<OccId>,
    /// The stable members met.
    leaves: Vec<OccId>,
    /// The subformulas of `?` met.
    theta: Vec<OccId>,
    /// The release the phase belongs to, among the rule's: the premises of
    /// one release share their context.
    group: u32,
}

impl Phase {
    /// The bytes its lists hold.
    fn bytes(&self) -> usize {
        bytes_of(&self.unary)
            + bytes_of(&self.withs)
            + bytes_of(&self.leaves)
            + bytes_of(&self.theta)
    }

    /// Places an occurrence the phase meets.
    fn place(&mut self, shape: &Shape<'_>, o: OccId) {
        if shape.stable(o) {
            self.leaves.push(o);
            return;
        }
        match shape.forest.kind(o) {
            Kind::Top => {
                self.top.get_or_insert(o);
            }
            Kind::With => self.withs.push(o),
            _ => self.unary.push(o),
        }
    }
}

/// Writes rules into a [`Rules`], charging what it allocates.
struct Writer<'s, 'a, 'c> {
    /// The shape of the goal.
    shape: &'s Shape<'a>,
    /// The rules written.
    rules: Rules,
    /// The steps of the rule being written, in reverse.
    reversed: Vec<Step>,
    /// Its holes, in reverse, as their members, `?` subformulas and
    /// release.
    holes: Vec<(Vec<OccId>, Vec<OccId>, u32)>,
    /// Its slots, in reverse, as the positive literal and the root.
    slots: Vec<(OccId, OccId)>,
    /// The charge for what is allocated, which holds the rules' tables as
    /// long as the search runs.
    charged: &'s mut Charged<'c>,
    /// The bytes charged for the rules' tables.
    held: usize,
    /// The steps written since the last poll.
    unpolled: u64,
    /// The releases of the rule being written.
    groups: u32,
    /// The bytes the rule being written holds beside its steps: its holes'
    /// lists and the phases waiting on the stack.
    pending: usize,
    /// The bytes of the lists in [`Rules::by_leaf`], which the table's own
    /// capacity does not count.
    inner: usize,
}

/// How many steps of the rules are written between two polls of the stop,
/// each a unit of work.
const STEPS_PER_POLL: u64 = 1 << 12;

impl<'s, 'a, 'c> Writer<'s, 'a, 'c> {
    /// Charges the tables' growth, and answers the memory bound.
    fn settle(&mut self) -> Result<(), Reason> {
        let r = &self.rules;
        let held = bytes_of(&r.all)
            + bytes_of(&r.steps)
            + bytes_of(&r.holes)
            + bytes_of(&r.leaves)
            + bytes_of(&r.slots)
            + bytes_of(&r.partners)
            + r.by_leaf.capacity() * (size_of::<(OccId, Vec<u32>)>() + 1)
            + r.lists.capacity() * (size_of::<((OccId, OccId), Range<u32>)>() + 1)
            + bytes_of(&r.open)
            + bytes_of(&self.reversed)
            + bytes_of(&self.holes)
            + self.pending
            + self.inner;
        self.charged.resize(self.held, held);
        self.held = held;
        if self.charged.account().over() {
            return Err(Reason::MemoryLimit {
                limit_bytes: self.charged.account().limit(),
            });
        }
        Ok(())
    }

    /// Counts a step written, and every [`STEPS_PER_POLL`] steps charges
    /// the tables and what the rule being written holds, and polls the
    /// stop with them as work.
    fn tick(&mut self, stop: &mut dyn FnMut(u64) -> bool) -> Result<(), Reason> {
        self.tick_by(1, stop)
    }

    /// Counts `units` of work (steps written, literals scanned for
    /// partners, members copied into the tables), charging and polling as
    /// [`Writer::tick`] does.
    fn tick_by(&mut self, units: u64, stop: &mut dyn FnMut(u64) -> bool) -> Result<(), Reason> {
        self.unpolled += units;
        if self.unpolled < STEPS_PER_POLL {
            return Ok(());
        }
        self.settle()?;
        if stop(std::mem::take(&mut self.unpolled)) {
            return Err(Reason::Stopped);
        }
        Ok(())
    }

    /// Writes the term below `root` under the choice of `⊕` sides given,
    /// and records it as a rule unless a slot has no partner; `anchor` is
    /// the formula the focus began on, which an axiom's partner must be
    /// able to stand beside.
    fn write(
        &mut self,
        root: Pending,
        sides: &[(OccId, Branch)],
        anchor: OccId,
        stop: &mut dyn FnMut(u64) -> bool,
    ) -> Result<(), Reason> {
        let forest = self.shape.forest;
        self.reversed.clear();
        self.holes.clear();
        self.slots.clear();
        self.pending = 0;
        self.groups = 0;
        let mut stack = Vec::new();
        self.push(&mut stack, root);
        while let Some(pending) = stack.pop() {
            self.tick(stop)?;
            match pending {
                Pending::Copy(o) => {
                    self.reversed.push(Step::Copy(o));
                    let next = self.under_focus(o);
                    self.push(&mut stack, next);
                }
                Pending::Focus(o) => match forest.kind(o) {
                    Kind::Tensor => {
                        self.reversed.push(Step::Tensor(o));
                        let (l, r) = (forest.left(o).expect("⊗"), forest.right(o).expect("⊗"));
                        // Reversed postorder: the right operand first.
                        let (l, r) = (self.under_focus(l), self.under_focus(r));
                        self.push(&mut stack, l);
                        self.push(&mut stack, r);
                    }
                    Kind::Plus => {
                        let side = sides
                            .iter()
                            .find(|&&(p, _)| p == o)
                            .map(|&(_, side)| side)
                            .expect("every ⊕ under the focus has a side");
                        self.reversed.push(Step::Plus(o, side));
                        let child = match side {
                            Branch::Left => forest.left(o),
                            Branch::Right => forest.right(o),
                        }
                        .expect("⊕");
                        let next = self.under_focus(child);
                        self.push(&mut stack, next);
                    }
                    Kind::One => self.reversed.push(Step::One(o)),
                    Kind::Bang => {
                        self.reversed.push(Step::Bang(o));
                        let mut phase = self.release();
                        phase.place(self.shape, forest.left(o).expect("!"));
                        self.push(&mut stack, Pending::Async(phase));
                    }
                    Kind::Atom | Kind::DualAtom => {
                        let slot = self.slots.len() as u32;
                        self.slots.push((o, anchor));
                        self.reversed.push(Step::Axiom(o, slot));
                    }
                    Kind::Zero | Kind::Par | Kind::Bot | Kind::With | Kind::Top | Kind::Quest => {
                        unreachable!("a plan never focuses on a 0 or a negative formula")
                    }
                },
                Pending::Async(mut phase) => {
                    self.pending -= phase.bytes();
                    loop {
                        if let Some(t) = phase.top {
                            self.reversed.push(Step::Top(t));
                            break;
                        }
                        if let Some(o) = phase.unary.pop() {
                            match forest.kind(o) {
                                Kind::Par => {
                                    self.reversed.push(Step::Par(o));
                                    for c in forest.children(o) {
                                        phase.place(self.shape, c);
                                    }
                                }
                                Kind::Bot => self.reversed.push(Step::Bot(o)),
                                Kind::Quest => {
                                    self.reversed.push(Step::Quest(o));
                                    phase.theta.push(forest.left(o).expect("?"));
                                }
                                _ => unreachable!("only ⅋, ⊥ and ? are unary in a phase"),
                            }
                            self.tick(stop)?;
                            continue;
                        }
                        if let Some(o) = phase.withs.pop() {
                            self.reversed.push(Step::With(o));
                            let mut left = phase.clone();
                            left.place(self.shape, forest.left(o).expect("&"));
                            phase.place(self.shape, forest.right(o).expect("&"));
                            // Reversed postorder: the right premise first,
                            // the left one after everything it pushes.
                            self.push(&mut stack, Pending::Async(left));
                            self.tick(stop)?;
                            continue;
                        }
                        let index = self.holes.len() as u32;
                        phase.leaves.sort_unstable();
                        phase.theta.sort_unstable();
                        self.pending += bytes_of(&phase.leaves) + bytes_of(&phase.theta);
                        self.holes.push((phase.leaves, phase.theta, phase.group));
                        self.reversed.push(Step::Hole(index));
                        break;
                    }
                }
            }
        }
        self.record(stop)?;
        self.settle()
    }

    /// Pushes a subtree to write, counting the lists of a phase among what
    /// the rule being written holds until it is popped.
    fn push(&mut self, stack: &mut Vec<Pending>, pending: Pending) {
        if let Pending::Async(phase) = &pending {
            self.pending += phase.bytes();
        }
        stack.push(pending);
    }

    /// A phase of a release of its own.
    fn release(&mut self) -> Phase {
        // A rule of 2³² releases has as many steps, which `record` refuses
        // before the numbers are read.
        self.groups = self.groups.wrapping_add(1);
        Phase {
            group: self.groups.wrapping_sub(1),
            ..Phase::default()
        }
    }

    /// The subtree of an operand of a positive connective under focus: the
    /// focus goes on into a positive one, and a negative one is released.
    fn under_focus(&mut self, o: OccId) -> Pending {
        if self.shape.positive(o) {
            Pending::Focus(o)
        } else {
            let mut phase = self.release();
            phase.place(self.shape, o);
            Pending::Async(phase)
        }
    }

    /// Records the rule written, numbering its holes and slots in the order
    /// of its steps, unless a slot has no partner; refuses a rule that
    /// takes a table past what a `u32` indexes, which only a search without
    /// a memory bound reaches.
    fn record(&mut self, stop: &mut dyn FnMut(u64) -> bool) -> Result<(), Reason> {
        let base_slot = self.rules.slots.len();
        // Slots, in step order: the reverse of the written order. Rules
        // that focus on the same formula under other `⊕` sides share their
        // literals' lists.
        let slots = std::mem::take(&mut self.slots);
        for &(p, anchor) in slots.iter().rev() {
            let range = match self.rules.lists.get(&(p, anchor)) {
                Some(range) => range.clone(),
                None => {
                    let start = self.rules.partners.len() as u32;
                    let (found, scanned) = partners(self.shape, p, anchor);
                    self.rules.partners.extend(found);
                    let range = start..self.rules.partners.len() as u32;
                    self.rules.lists.insert((p, anchor), range.clone());
                    self.tick_by(scanned, stop)?;
                    range
                }
            };
            if range.is_empty() {
                self.rules.slots.truncate(base_slot);
                self.holes.clear();
                self.pending = 0;
                return Ok(());
            }
            self.rules.slots.push(range);
        }
        let rule = self.rules.all.len() as u32;
        let (holes, slots) = (self.holes.len() as u32, slots.len() as u32);
        let hole_base = self.rules.holes.len() as u32;
        // The holes leave the rule being written as they are copied into
        // the tables, which are charged and polled as they grow: the goal's
        // phase can hold tens of millions of members.
        let written = std::mem::take(&mut self.holes);
        self.pending += bytes_of(&written);
        for (index, (leaves, theta, group)) in written.into_iter().rev().enumerate() {
            let start = self.rules.leaves.len() as u32;
            self.rules.leaves.extend_from_slice(&leaves);
            let mid = self.rules.leaves.len() as u32;
            self.rules.leaves.extend_from_slice(&theta);
            let end = self.rules.leaves.len() as u32;
            let global = self.rules.holes.len() as u32;
            match leaves.first() {
                Some(&first) => {
                    let list = self.rules.by_leaf.entry(first).or_default();
                    let before = bytes_of(list);
                    list.push(global);
                    self.inner += bytes_of(list) - before;
                }
                None => self.rules.open.push(global),
            }
            self.rules.holes.push(Hole {
                rule,
                index: index as u32,
                group,
                leaves: start..mid,
                theta: mid..end,
            });
            self.pending -= bytes_of(&leaves) + bytes_of(&theta);
            self.tick_by(1 + (leaves.len() + theta.len()) as u64, stop)?;
        }
        self.pending = 0;
        let step_base = self.rules.steps.len() as u32;
        for i in (0..self.reversed.len()).rev() {
            let step = self.reversed[i];
            self.rules.steps.push(match step {
                Step::Hole(h) => Step::Hole(holes - 1 - h),
                Step::Axiom(p, s) => Step::Axiom(p, slots - 1 - s),
                step => step,
            });
            self.tick(stop)?;
        }
        self.rules.all.push(Rule {
            steps: step_base..self.rules.steps.len() as u32,
            holes: hole_base..hole_base + holes,
            slots: base_slot as u32..base_slot as u32 + slots,
        });
        // Every index into the tables is a `u32`, and every range over them
        // was taken from their lengths: a table past `u32::MAX` would make
        // them wrap, so the rule that takes one there is refused before any
        // is read.
        let most = u32::MAX as usize;
        let r = &self.rules;
        if [
            r.all.len(),
            r.steps.len(),
            r.holes.len(),
            r.leaves.len(),
            r.slots.len(),
            r.partners.len(),
        ]
        .into_iter()
        .any(|len| len > most)
        {
            return Err(Reason::IndexLimit);
        }
        Ok(())
    }
}

/// The partners a positive literal under focus may take: the negative
/// literals of its atom that can stand beside the formula the focus began
/// on, from the unrestricted zone where they are the subformula of a `?`
/// (and from the linear zone too where the goal holds them), with the
/// opposite position under a reading; and the literals scanned, as work.
fn partners(shape: &Shape<'_>, p: OccId, anchor: OccId) -> (Vec<Partner>, u64) {
    let forest = shape.forest;
    let (atom, sign) = (
        forest.atom(p).expect("a literal"),
        forest.sign(p).expect("a literal"),
    );
    let mut found = Vec::new();
    for &x in forest.literals(atom, !sign) {
        if shape.reading.is_some() && shape.output(x) == shape.output(p) {
            continue;
        }
        if !shape.coherent(anchor, x) {
            continue;
        }
        let quest = forest
            .parent(x)
            .is_some_and(|q| forest.kind(q) == Kind::Quest);
        if quest {
            found.push(Partner { x, theta: true });
        }
        if !quest || shape.goal.binary_search(&x).is_ok() {
            found.push(Partner { x, theta: false });
        }
    }
    (found, forest.literals(atom, !sign).len() as u64)
}

/// Calls `each` with every way of choosing the `⊕` sides under a focus on
/// `root`, a list of the `⊕` reached and the side taken, until it fails; a
/// choice that reaches a `0` is left out, since no focus on it succeeds.
/// `each` is called with `None` for every choice looked at, whole or not,
/// as a unit of work: choices that a `0` ends can be exponentially many.
fn plans(
    shape: &Shape<'_>,
    root: OccId,
    mut each: impl FnMut(Option<&[(OccId, Branch)]>) -> Result<(), Reason>,
) -> Result<(), Reason> {
    let forest = shape.forest;
    let mut open = vec![Vec::new()];
    'plans: while let Some(sides) = open.pop() {
        each(None)?;
        let mut walk = vec![root];
        while let Some(o) = walk.pop() {
            match forest.kind(o) {
                Kind::Tensor => walk.extend(forest.children(o).filter(|&c| shape.positive(c))),
                Kind::Plus => match sides.iter().find(|&&(p, _)| p == o) {
                    Some(&(_, side)) => {
                        let child = match side {
                            Branch::Left => forest.left(o),
                            Branch::Right => forest.right(o),
                        }
                        .expect("⊕");
                        if shape.positive(child) {
                            walk.push(child);
                        }
                    }
                    None => {
                        for side in [Branch::Right, Branch::Left] {
                            let mut more = sides.clone();
                            more.push((o, side));
                            open.push(more);
                        }
                        continue 'plans;
                    }
                },
                Kind::Zero => continue 'plans,
                _ => {}
            }
        }
        each(Some(&sides))?;
    }
    Ok(())
}

/// Compiles the rules of the search: a focus on every positive formula a
/// stable sequent can hold, and a copy of every subformula of a `?` but the
/// negative literals (which a partner takes from the unrestricted zone),
/// each under every choice of its `⊕` sides. Charges the tables to
/// `charged`, which holds them while the search runs, and counts a step as
/// a unit of work for `stop`.
pub(super) fn compile(
    shape: &Shape<'_>,
    charged: &mut Charged<'_>,
    stop: &mut dyn FnMut(u64) -> bool,
) -> Result<Rules, Reason> {
    let forest = shape.forest;
    let mut writer = Writer {
        shape,
        rules: Rules::default(),
        reversed: Vec::new(),
        holes: Vec::new(),
        slots: Vec::new(),
        charged,
        held: 0,
        unpolled: 0,
        groups: 0,
        pending: 0,
        inner: 0,
    };
    let mut previous = None;
    for &g in &shape.goal {
        if previous.replace(g) == Some(g) {
            continue;
        }
        for o in forest.subtree(g) {
            let parent = forest.parent(o).filter(|_| o != g).map(|p| forest.kind(p));
            let focus = shape.positive(o)
                && matches!(parent, None | Some(Kind::Par | Kind::With | Kind::Bang));
            if focus {
                plans(shape, o, |sides| match sides {
                    Some(sides) => writer.write(Pending::Focus(o), sides, o, stop),
                    None => writer.tick(stop),
                })?;
            }
            let copied =
                parent == Some(Kind::Quest) && !(forest.is_literal(o) && !shape.positive(o));
            if copied {
                plans(shape, o, |sides| match sides {
                    Some(sides) => writer.write(Pending::Copy(o), sides, o, stop),
                    None => writer.tick(stop),
                })?;
            }
        }
    }
    writer.settle()?;
    Ok(writer.rules)
}

/// Compiles the goal's asynchronous phase as one rule without slots, whose
/// holes are the stable sequents the goal decomposes into, charging it to
/// `charged` and counting a step as a unit of work for `stop`.
pub(super) fn goal(
    shape: &Shape<'_>,
    charged: &mut Charged<'_>,
    stop: &mut dyn FnMut(u64) -> bool,
) -> Result<Rules, Reason> {
    let mut writer = Writer {
        shape,
        rules: Rules::default(),
        reversed: Vec::new(),
        holes: Vec::new(),
        slots: Vec::new(),
        charged,
        held: 0,
        unpolled: 0,
        groups: 0,
        pending: 0,
        inner: 0,
    };
    let mut phase = writer.release();
    for &o in &shape.goal {
        phase.place(shape, o);
    }
    let anchor = shape.goal.first().copied().unwrap_or(OccId::new(0));
    writer.write(Pending::Async(phase), &[], anchor, stop)?;
    Ok(writer.rules)
}
