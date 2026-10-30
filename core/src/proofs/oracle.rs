// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The checker as it was first written, kept for the tests: it derives a
//! sequent for every node and keeps them all, each with a set as wide as
//! the forest, so it is quadratic in memory and easy to read. The checker
//! proper must accept and reject exactly what this one does, with the same
//! error.

use super::check::{CheckError, Dyadic, Fault};
use super::multiset::Multiset;
use super::{Branch, Node, NodeId, Proof};
#[cfg(feature = "parse")]
use crate::Sequent;
use crate::fragment::Mode;
use crate::occurrences::{Forest, Member, OccId, Reading, Side};
use crate::sequents::{Eigen, FoTerm, FoTermId, Kind};
use std::collections::{BTreeMap, BTreeSet};

/// What a subproof proves, as the checker derives it from the node's
/// premises: the dyadic sequent `⊢ Θ ; Γ`, or with `any` set `⊢ Θ ; Γ, Δ` for
/// every `Δ`, because a `⊤` leaf above absorbs whatever context it is given.
///
/// `Θ` is the least unrestricted zone the subproof needs, the occurrences it
/// copies without moving them into `Θ` itself; since weakening and
/// contraction on `Θ` are implicit, any larger zone does as well. So a proof
/// is correct when its root needs an empty `Θ` and derives the sequent's
/// formulas, and the checker never has to know the actual zone.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Derived {
    /// The unrestricted zone the subproof needs.
    pub(crate) theta: BTreeSet<Member>,
    /// The linear zone.
    pub(crate) gamma: Multiset<Member>,
    /// Whether a `⊤` above absorbs any further linear context.
    pub(crate) any: bool,
    /// The eigenvariables the witnesses and cut frames above name, less
    /// those the `∀` steps above introduce.
    pub(crate) eigens: BTreeSet<Eigen>,
}

impl Derived {
    /// Returns the sequent as ids for an error report.
    fn to_dyadic(&self) -> Dyadic {
        Dyadic {
            theta: self.theta.iter().copied().collect(),
            gamma: self.gamma.as_slice().to_vec(),
            any: self.any,
        }
    }
}

/// Checks that a proof proves its sequent: every node applies its rule to
/// what its premises derive, the mode allows the rule, and the root derives
/// exactly the sequent's formulas with nothing left in the unrestricted
/// zone; in intuitionistic mode also that the sequent has an intuitionistic
/// reading and every sequent of the proof one formula on the right of `⊢`.
/// Returns the first node that fails, in arena order, with what it needed.
pub(crate) fn check(proof: &Proof, mode: Mode) -> Result<(), CheckError> {
    let reading = if mode.intuitionistic {
        match Reading::of_mode(proof.forest(), mode) {
            Ok(reading) => Some(reading),
            Err(e) => {
                return Err(CheckError::invalid(
                    proof.root(),
                    proof.node(proof.root()),
                    vec![],
                    Fault::Shape(e),
                ));
            }
        }
    } else {
        None
    };
    let derived = derive(proof, mode, reading.as_ref())?;
    conclude(proof, mode, &derived)
}

/// Derives what every node proves, in arena order, or reports the first
/// node that misapplies its rule or uses one the mode forbids; with a
/// reading, also the first sequent that breaks the one-succedent condition.
pub(crate) fn derive(
    proof: &Proof,
    mode: Mode,
    reading: Option<&Reading>,
) -> Result<Vec<Derived>, CheckError> {
    let mut derived = Vec::with_capacity(proof.nodes().len());
    let (nodes, roots) = (proof.nodes().len(), proof.conclusion().len());
    let frames = Frames::of(proof);
    for id in proof.ids() {
        // A zone the later nodes cannot bring down to the roots: each
        // consumes two members at most.
        let room = roots + 2 * (nodes - 1 - id.index());
        let d = Step::new(proof, mode, id, &derived, reading, &frames).derive(room)?;
        derived.push(d);
    }
    Ok(derived)
}

/// The proof's instances as the oracle reads them: every member by its
/// occurrence and its frame's terms written out, the innermost binder's
/// first, so that two instances are equal when their texts are; and the
/// faults that rest on the proof as a whole.
struct Frames {
    /// The forest's length.
    first: usize,
    /// Per member past the forest, its occurrence, its frame's terms and
    /// their texts.
    members: Vec<(OccId, Vec<FoTermId>, Vec<String>)>,
    /// Every member past the forest by its occurrence and frame's texts.
    by: BTreeMap<(OccId, Vec<String>), Member>,
    /// The terms of the sequent and the proof.
    terms: Vec<FoTerm>,
    /// Their argument lists, by list id.
    lists: BTreeMap<u32, Vec<FoTermId>>,
    /// The symbols' names.
    symbols: Vec<String>,
    /// The fault of a node, where the proof gives it one: a member under
    /// binders named bare, or a second introduction of an eigenvariable.
    faults: BTreeMap<NodeId, Fault>,
    /// The eigenvariables a `∀` step introduces; any other is a constant.
    introduced: BTreeSet<Eigen>,
}

impl Frames {
    /// Reads the instances of `proof`.
    fn of(proof: &Proof) -> Self {
        let forest = proof.forest();
        let fo = &forest.sequent().fo;
        let mut this = Self {
            first: forest.len(),
            members: vec![],
            by: BTreeMap::new(),
            terms: fo.terms.clone(),
            lists: BTreeMap::new(),
            symbols: fo.symbols.iter().map(|(n, _)| n.clone()).collect(),
            faults: BTreeMap::new(),
            introduced: BTreeSet::new(),
        };
        for (k, &(start, len)) in fo.lists.iter().enumerate() {
            let items = fo.args[start as usize..(start + len) as usize].to_vec();
            this.lists.insert(k as u32, items);
        }
        if let Some(table) = proof.instances() {
            let base = fo.lists.len() as u32;
            this.terms.extend_from_slice(table.terms());
            for (k, &(start, len)) in table.lists.iter().enumerate() {
                let items = table.args[start as usize..(start + len) as usize].to_vec();
                this.lists.insert(base + k as u32, items);
            }
            for (n, &(o, frame)) in table.members.iter().enumerate() {
                let terms = table.frame_terms(frame);
                let texts: Vec<String> = terms.iter().map(|&t| this.text(t, &[])).collect();
                this.by
                    .insert((o, texts.clone()), Member::new((this.first + n) as u32));
                this.members.push((o, terms, texts));
            }
        }
        // Bound variables above each occurrence, by walking its parents.
        let binders = |o: OccId| {
            let mut n = 0;
            let mut at = o;
            while let Some(p) = forest.parent(at) {
                if matches!(forest.kind(p), Kind::Forall | Kind::Exists) {
                    n += 1;
                }
                at = p;
            }
            n
        };
        let mut first: BTreeMap<Eigen, NodeId> = BTreeMap::new();
        for id in proof.ids() {
            let node = proof.node(id);
            if let Some(member) = node
                .members()
                .find(|m| m.index() < this.first && binders(m.occ()) > 0)
            {
                this.faults.insert(id, Fault::Instance { member });
            } else if let Node::Forall(_, a, _) = node {
                match first.get(&a) {
                    Some(_) => {
                        this.faults.insert(id, Fault::Reintroduced { eigen: a });
                    }
                    None => {
                        first.insert(a, id);
                    }
                }
            }
        }
        this.introduced = first.keys().copied().collect();
        this
    }

    /// Writes the term `t` out, every bound variable `i` as `frame[i]`.
    fn text(&self, t: FoTermId, frame: &[String]) -> String {
        match self.terms[t.index()] {
            FoTerm::Bound(i) => frame
                .get(i as usize)
                .cloned()
                .unwrap_or_else(|| format!("#{i}")),
            FoTerm::Eigen(a) => format!("#e{}", a.get()),
            FoTerm::Meta(k) => format!("#m{k}"),
            FoTerm::App(f, args) => {
                let items: Vec<String> = self.lists[&args.get()]
                    .iter()
                    .map(|&u| self.text(u, frame))
                    .collect();
                format!("{}({})", self.symbols[f.index()], items.join(","))
            }
        }
    }

    /// Adds the eigenvariables of the term `t` to `eigens`.
    fn eigens(&self, t: FoTermId, eigens: &mut BTreeSet<Eigen>) {
        match self.terms[t.index()] {
            FoTerm::Eigen(a) => {
                eigens.insert(a);
            }
            FoTerm::App(_, args) => {
                for &u in &self.lists[&args.get()] {
                    self.eigens(u, eigens);
                }
            }
            FoTerm::Bound(_) | FoTerm::Meta(_) => {}
        }
    }

    /// Returns the occurrence `m` stands for.
    fn occ(&self, m: Member) -> OccId {
        match m.index().checked_sub(self.first) {
            None => m.occ(),
            Some(at) => self.members[at].0,
        }
    }

    /// Returns the frame of `m` written out, empty for an occurrence.
    fn frame(&self, m: Member) -> Vec<String> {
        match m.index().checked_sub(self.first) {
            None => vec![],
            Some(at) => self.members[at].2.clone(),
        }
    }

    /// Returns the member that is `o` under the frame `frame` written out.
    fn member(&self, o: OccId, frame: Vec<String>) -> Member {
        if frame.is_empty() {
            return Member::from(o);
        }
        self.by[&(o, frame)]
    }
}

/// Checks that the root derives the proof's sequent.
pub(crate) fn conclude(proof: &Proof, mode: Mode, derived: &[Derived]) -> Result<(), CheckError> {
    let root = proof.root();
    let d = &derived[root.index()];
    let frames = Frames::of(proof);
    if let Some(&eigen) = d.eigens.iter().find(|a| frames.introduced.contains(a)) {
        return Err(Step::new(proof, mode, root, derived, None, &frames)
            .fail(Fault::Eigenvariable { eigen }));
    }
    let roots = Multiset::of(proof.conclusion().into_iter().map(Member::from));
    let concludes = if d.any {
        d.gamma.is_subset(&roots)
    } else {
        d.gamma == roots
    };
    if d.theta.is_empty() && concludes {
        Ok(())
    } else {
        let frames = Frames::of(proof);
        Err(
            Step::new(proof, mode, root, derived, None, &frames).fail(Fault::Conclusion {
                derived: d.to_dyadic(),
            }),
        )
    }
}

/// One node being checked, with what its premises derived.
struct Step<'a> {
    /// The forest the occurrences index.
    forest: &'a Forest,
    /// The rules in force.
    mode: Mode,
    /// The node.
    id: NodeId,
    /// Its rule instance.
    node: Node,
    /// What the nodes before it derived.
    derived: &'a [Derived],
    /// The intuitionistic reading, in intuitionistic mode.
    reading: Option<&'a Reading<'a>>,
    /// The proof's instances.
    frames: &'a Frames,
}

impl<'a> Step<'a> {
    /// The step for node `id`, whose premises have their entries in
    /// `derived`.
    fn new(
        proof: &'a Proof,
        mode: Mode,
        id: NodeId,
        derived: &'a [Derived],
        reading: Option<&'a Reading<'a>>,
        frames: &'a Frames,
    ) -> Self {
        Self {
            forest: proof.forest(),
            mode,
            id,
            node: proof.node(id),
            derived,
            reading,
            frames,
        }
    }

    /// The occurrence a member stands for.
    fn occ(&self, m: Member) -> OccId {
        self.frames.occ(m)
    }

    /// The member of the subformula `c` of `m`'s occurrence, under `m`'s
    /// frame.
    fn sub(&self, m: Member, c: OccId) -> Member {
        self.frames.member(c, self.frames.frame(m))
    }

    /// The member of the body of `m`'s binder under its frame with the
    /// term `t` pushed.
    fn body(&self, m: Member, t: FoTermId) -> Member {
        let mut frame = vec![self.frames.text(t, &[])];
        frame.extend(self.frames.frame(m));
        self.frames.member(OccId::new(self.occ(m).get() + 1), frame)
    }

    /// The atom of the literal `m` with its frame's terms in place of its
    /// bound variables, written out, and whether it is negated.
    fn atom(&self, m: Member) -> (String, bool) {
        let o = self.occ(m);
        let f = self.forest;
        let atom = f.atom(o).unwrap();
        let negated = f.kind(o) == Kind::DualAtom;
        let Some((p, args)) = f.sequent().atom_predicate(atom) else {
            return (f.sequent().atom_name(atom).to_owned(), negated);
        };
        let frame = self.frames.frame(m);
        let items: Vec<String> = f
            .sequent()
            .arguments(args)
            .iter()
            .map(|&t| self.frames.text(t, &frame))
            .collect();
        let name = &f.sequent().symbols()[p.index()].0;
        (format!("{name}({})", items.join(",")), negated)
    }

    /// The error for this node.
    fn fail(&self, fault: Fault) -> CheckError {
        let premises = self
            .node
            .premises()
            .map(|p| self.derived[p.index()].to_dyadic())
            .collect();
        CheckError::invalid(self.id, self.node, premises, fault)
    }

    /// What premise `p` derived.
    fn premise(&self, p: NodeId) -> Derived {
        self.derived[p.index()].clone()
    }

    /// Fails unless `o` has the kind the rule acts on.
    fn expect(&self, m: Member, kind: Kind) -> Result<(), CheckError> {
        if self.forest.kind(self.occ(m)) == kind {
            Ok(())
        } else {
            Err(self.fail(Fault::Kind { member: m }))
        }
    }

    /// Consumes one copy of `o` from the linear zone of `d`, the derived
    /// sequent of premise `premise`; a `⊤` above stands in for a missing
    /// one, but never for a second goal.
    fn take(&self, d: &mut Derived, o: Member, premise: usize) -> Result<(), CheckError> {
        if d.gamma.remove(o) {
            return Ok(());
        }
        if !d.any {
            return Err(self.fail(Fault::Missing { premise, member: o }));
        }
        // The premise's sequent holds `o` besides its zone: one goal at
        // most.
        if let Some(reading) = self.reading
            && reading.position(self.occ(o)) == Side::Output
            && self.outputs(&d.gamma) > 0
        {
            return Err(self.fail(Fault::Succedents { count: 2 }));
        }
        Ok(())
    }

    /// Counts the formulas of a linear zone in output position, in
    /// intuitionistic mode.
    fn outputs(&self, gamma: &Multiset<Member>) -> usize {
        self.reading.map_or(0, |r| {
            r.outputs(gamma.as_slice().iter().map(|&m| self.occ(m)))
        })
    }

    /// Checks the one-succedent condition on what the node derived: one
    /// output at most, and exactly one unless a `⊤` above supplies it.
    fn one_succedent(&self, d: Derived) -> Result<Derived, CheckError> {
        if self.reading.is_none() {
            return Ok(d);
        }
        let outputs = self.outputs(&d.gamma);
        if outputs > 1 || (outputs == 0 && !d.any) {
            return Err(self.fail(Fault::Succedents { count: outputs }));
        }
        Ok(d)
    }

    /// The left subformula of `m`, which must have one.
    fn left(&self, m: Member) -> Member {
        self.sub(m, self.forest.left(self.occ(m)).unwrap())
    }

    /// The right subformula of `m`, which must have one.
    fn right(&self, m: Member) -> Member {
        self.sub(m, self.forest.right(self.occ(m)).unwrap())
    }

    /// The sequent with only `o` in its linear zone.
    fn just(&self, o: Member, any: bool) -> Derived {
        Derived {
            theta: BTreeSet::new(),
            gamma: Multiset::of([o]),
            any,
            eigens: BTreeSet::new(),
        }
    }

    /// The sequent both premises of a two-premise rule give together: the
    /// unrestricted zones united, the linear zones summed, absorbing if
    /// either is.
    fn join(&self, l: Derived, r: Derived) -> Derived {
        Derived {
            theta: &l.theta | &r.theta,
            gamma: l.gamma.sum(&r.gamma),
            any: l.any || r.any,
            eigens: &l.eigens | &r.eigens,
        }
    }

    /// Derives what the node proves from what its premises derived, a
    /// linear zone of `room` members at most.
    fn derive(self, room: usize) -> Result<Derived, CheckError> {
        let d = self.rule()?;
        if d.gamma.as_slice().len() > room {
            return Err(self.fail(Fault::Surplus));
        }
        self.one_succedent(d).and_then(|d| self.antecedent(d))
    }

    /// In the Lambek calculus without empty antecedents, checks that what
    /// the node derived holds a formula in input position.
    fn antecedent(&self, d: Derived) -> Result<Derived, CheckError> {
        let Some(reading) = self
            .reading
            .filter(|_| !self.mode.allows_empty_antecedents())
        else {
            return Ok(d);
        };
        if d.gamma
            .as_slice()
            .iter()
            .any(|&o| reading.position(self.occ(o)) == Side::Input)
        {
            Ok(d)
        } else {
            Err(self.fail(Fault::EmptyAntecedent))
        }
    }

    /// Whether the conclusion of a `⊗` at `o` over the contexts `left` and
    /// `right`, listed in occurrence order and turned to start after `o`,
    /// is the right context followed by the left one.
    fn arcs(&self, o: OccId, left: &Multiset<Member>, right: &Multiset<Member>) -> bool {
        let mut members: Vec<(OccId, bool)> = left
            .as_slice()
            .iter()
            .map(|&x| (self.occ(x), true))
            .chain(right.as_slice().iter().map(|&x| (self.occ(x), false)))
            .collect();
        members.sort_unstable();
        let turn = members.partition_point(|&(x, _)| x < o);
        members.rotate_left(turn);
        members.windows(2).all(|w| !(w[0].1 && !w[1].1))
    }

    /// Applies the node's rule to what its premises derived.
    fn rule(&self) -> Result<Derived, CheckError> {
        use Node::*;
        let f = self.forest;
        if self.mode.is_ordered() && !matches!(self.node, Ax(..) | Par(..) | Tensor(..)) {
            return Err(self.fail(Fault::Forbidden));
        }
        if let Some(fault) = self.frames.faults.get(&self.id) {
            return Err(self.fail(fault.clone()));
        }
        match self.node {
            Ax(a, b) => {
                for o in [a, b] {
                    if !f.is_literal(self.occ(o)) {
                        return Err(self.fail(Fault::Kind { member: o }));
                    }
                }
                let ((x, nx), (y, ny)) = (self.atom(a), self.atom(b));
                if x != y || nx == ny {
                    return Err(self.fail(Fault::NotDual));
                }
                Ok(Derived {
                    theta: BTreeSet::new(),
                    gamma: Multiset::of([a, b]),
                    any: false,
                    eigens: BTreeSet::new(),
                })
            }
            One(o) => {
                self.expect(o, Kind::One)?;
                Ok(self.just(o, false))
            }
            Top(o) => {
                self.expect(o, Kind::Top)?;
                Ok(self.just(o, true))
            }
            Bot(o, p) => {
                self.expect(o, Kind::Bot)?;
                let mut d = self.premise(p);
                d.gamma.insert(o);
                Ok(d)
            }
            Par(o, p) => {
                self.expect(o, Kind::Par)?;
                let mut d = self.premise(p);
                self.take(&mut d, self.left(o), 0)?;
                self.take(&mut d, self.right(o), 0)?;
                d.gamma.insert(o);
                Ok(d)
            }
            Tensor(o, l, r) => {
                self.expect(o, Kind::Tensor)?;
                let (mut dl, mut dr) = (self.premise(l), self.premise(r));
                self.take(&mut dl, self.left(o), 0)?;
                self.take(&mut dr, self.right(o), 1)?;
                if self.mode.is_ordered() && !self.arcs(self.occ(o), &dl.gamma, &dr.gamma) {
                    return Err(self.fail(Fault::Order));
                }
                let mut d = self.join(dl, dr);
                d.gamma.insert(o);
                Ok(d)
            }
            With(o, l, r) => {
                self.expect(o, Kind::With)?;
                let (mut dl, mut dr) = (self.premise(l), self.premise(r));
                self.take(&mut dl, self.left(o), 0)?;
                self.take(&mut dr, self.right(o), 1)?;
                // The conclusion's context is what both premises can prove
                // it under: an absorbing premise adapts to the other one.
                let (gamma, any) = match (dl.any, dr.any) {
                    (false, false) if dl.gamma == dr.gamma => (dl.gamma, false),
                    (true, false) if dl.gamma.is_subset(&dr.gamma) => (dr.gamma, false),
                    (false, true) if dr.gamma.is_subset(&dl.gamma) => (dl.gamma, false),
                    (true, true) => (dl.gamma.union(&dr.gamma), true),
                    _ => return Err(self.fail(Fault::Differ)),
                };
                let mut gamma = gamma;
                gamma.insert(o);
                Ok(Derived {
                    theta: &dl.theta | &dr.theta,
                    gamma,
                    any,
                    eigens: &dl.eigens | &dr.eigens,
                })
            }
            Plus(o, side, p) => {
                self.expect(o, Kind::Plus)?;
                let mut d = self.premise(p);
                let chosen = match side {
                    Branch::Left => self.left(o),
                    Branch::Right => self.right(o),
                };
                self.take(&mut d, chosen, 0)?;
                d.gamma.insert(o);
                Ok(d)
            }
            Bang(o, p) => {
                self.expect(o, Kind::Bang)?;
                let mut d = self.premise(p);
                self.take(&mut d, self.left(o), 0)?;
                if !d.gamma.is_empty() {
                    return Err(self.fail(Fault::NotEmpty));
                }
                // Promotion fixes the linear zone: a ⊤ above cannot absorb
                // past it.
                Ok(Derived {
                    theta: d.theta,
                    gamma: Multiset::of([o]),
                    any: false,
                    eigens: d.eigens,
                })
            }
            Quest(o, p) => {
                self.expect(o, Kind::Quest)?;
                let mut d = self.premise(p);
                d.theta.remove(&self.left(o));
                d.gamma.insert(o);
                Ok(d)
            }
            Copy(a, p) => {
                if f.parent(self.occ(a)).map(|q| f.kind(q)) != Some(Kind::Quest) {
                    return Err(self.fail(Fault::NotUnderQuest { member: a }));
                }
                let mut d = self.premise(p);
                self.take(&mut d, a, 0)?;
                d.theta.insert(a);
                Ok(d)
            }
            Weaken(o, p) => {
                let occ = self.occ(o);
                // Weakening a `?` formula is a rule of every mode; the goal
                // is never weakened.
                if !self.mode.affine && f.kind(occ) != Kind::Quest {
                    return Err(self.fail(Fault::Forbidden));
                }
                if self
                    .reading
                    .is_some_and(|r| r.position(occ) == Side::Output)
                {
                    return Err(self.fail(Fault::Succedents { count: 0 }));
                }
                let mut d = self.premise(p);
                d.gamma.insert(o);
                Ok(d)
            }
            Mix(l, r) => {
                // Mix has no intuitionistic form: a premise would lack the
                // goal.
                if !self.mode.mix || self.mode.intuitionistic {
                    return Err(self.fail(Fault::Forbidden));
                }
                Ok(self.join(self.premise(l), self.premise(r)))
            }
            Cut(a, l, r) => {
                // The occurrence of `A⊥` in the place `a` has in `A`, from
                // the pairs themselves, under `a`'s frame.
                let o = self.occ(a);
                let Some(dual) = f
                    .cut_pairs()
                    .find(|&(root, _)| root <= o && o.get() < root.get() + f.size(root))
                    .map(|(root, dual)| OccId::new(dual.get() + (o.get() - root.get())))
                else {
                    return Err(self.fail(Fault::NotACut { member: a }));
                };
                let dual = self.sub(a, dual);
                let (mut dl, mut dr) = (self.premise(l), self.premise(r));
                self.take(&mut dl, a, 0)?;
                self.take(&mut dr, dual, 1)?;
                let mut d = self.join(dl, dr);
                // The terms the cut's frame chose come in as a witness's.
                if let Some(at) = a.index().checked_sub(self.frames.first) {
                    for &t in &self.frames.members[at].1 {
                        self.frames.eigens(t, &mut d.eigens);
                    }
                }
                Ok(d)
            }
            Forall(o, a, p) => {
                self.expect(o, Kind::Forall)?;
                let mut d = self.premise(p);
                let term = self
                    .frames
                    .terms
                    .iter()
                    .position(|&t| t == FoTerm::Eigen(a))
                    .expect("the table holds every eigenvariable's term");
                let body = self.body(o, FoTermId::new(term as u32));
                self.take(&mut d, body, 0)?;
                d.eigens.remove(&a);
                d.gamma.insert(o);
                Ok(d)
            }
            Exists(o, w, p) => {
                self.expect(o, Kind::Exists)?;
                let mut d = self.premise(p);
                self.take(&mut d, self.body(o, w), 0)?;
                self.frames.eigens(w, &mut d.eigens);
                d.gamma.insert(o);
                Ok(d)
            }
        }
    }
}

/// Returns proofs to test a pass over proofs on, each with the mode it was
/// found in: those the engines find of generated sequents, classical and
/// intuitionistic, linear and affine, and of the smallest instance of
/// every family, and with the feature `interactive` proofs with cuts of
/// the generated sequents.
#[cfg(feature = "parse")]
pub(crate) fn proofs() -> Vec<(Proof, Mode)> {
    use crate::search::generate::{self, IllRules, Rng, Rules};
    use crate::search::{Options, Verdict, prove_within};
    let classical = Mode::CLASSICAL;
    let mut cases: Vec<(String, Mode, Options)> = vec![];
    for (k, rules) in Rules::ALL.into_iter().enumerate() {
        let mut rng = Rng::new(900 + k as u64);
        for _ in 0..25 {
            let budget = 2 + rng.below(9);
            let provable = generate::provable(&mut rng, rules, 3, budget);
            let mode = if rules.mix {
                classical.with_mix()
            } else {
                classical
            };
            let options = Options::default().with_copies(Some(provable.copies));
            let text = generate::sequent(&provable.formulas);
            cases.push((text.clone(), mode, options.clone()));
            cases.push((text, mode.with_affine(), options));
        }
    }
    for (k, rules) in IllRules::ALL.into_iter().enumerate() {
        let mut rng = Rng::new(950 + k as u64);
        for _ in 0..25 {
            let budget = 2 + rng.below(9);
            let ill = generate::ill(&mut rng, rules, 3, budget);
            let options = Options::default().with_copies(Some(ill.copies));
            let text = generate::two_sided(&ill.hypotheses, &ill.goal);
            cases.push((text.clone(), Mode::INTUITIONISTIC, options.clone()));
            cases.push((text, Mode::INTUITIONISTIC.with_affine(), options));
        }
    }
    let mut proofs: Vec<(Proof, Mode)> = vec![];
    for (text, mode, options) in cases {
        let sequent: Sequent = text.parse().unwrap();
        let mut polls = 0;
        let outcome = prove_within(&sequent, mode, &options, &crate::Limits::default(), |_| {
            polls += 1;
            polls > 20_000
        });
        if let Ok(Verdict::Proved(proof)) = outcome.map(|o| o.verdict) {
            proofs.push((*proof, mode));
        }
    }
    // Proofs with cuts of the generated sequents, kept for the end.
    #[cfg(feature = "interactive")]
    let cuts = crate::proofs::interactive::with_cuts(&proofs, &mut Rng::new(41));
    for family in crate::families::FAMILIES {
        let instance = family.instance(family.sizes[0], 0).unwrap();
        let options = match instance.copies {
            Some(copies) => Options::default().with_copies(Some(copies)),
            None => Options::default(),
        };
        let mut polls = 0;
        let outcome = prove_within(
            &instance.sequent,
            instance.mode,
            &options,
            &crate::Limits::default(),
            |_| {
                polls += 1;
                polls > 20_000
            },
        );
        if let Ok(Verdict::Proved(proof)) = outcome.map(|o| o.verdict) {
            proofs.push((*proof, instance.mode));
        }
    }
    let mut rng = Rng::new(36);
    for k in 0..60 {
        let mode = if k % 2 == 0 {
            Mode::LAMBEK
        } else {
            Mode::LAMBEK.with_empty_antecedents()
        };
        let budget = 2 + rng.below(8);
        let (hypotheses, goal) = generate::lambek(&mut rng, 3, budget, k % 2 == 1);
        let text = generate::lambek_text(&hypotheses, &goal);
        let sequent =
            Sequent::parse_ordered(&text, crate::Order::Cyclic, &crate::Limits::default()).unwrap();
        if let Ok(Verdict::Proved(proof)) =
            crate::search::prove(&sequent, mode, &Options::default()).map(|o| o.verdict)
        {
            proofs.push((*proof, mode));
        }
    }
    #[cfg(feature = "interactive")]
    proofs.extend(cuts);
    proofs
}
