// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The cut in a proof in progress: the user names the cut formula, which
//! joins the session's sequent as a cut formula with its dual, and the
//! goal's two premises open as goals.

use super::quantified::Quantified;
use super::{GoalId, Interactive, StepError};
use crate::Error;
#[cfg(feature = "parse")]
use crate::fragment::Mode;
use crate::limits::Limits;
use crate::occurrences::{
    ClassicalSequent, Forest, IllSequent, Member, OccId, Reading, Side, Written,
};
use crate::proofs::Rule;
use crate::proofs::derivation::{InfId, Inference};
use crate::proofs::multiset::Multiset;
use crate::sequents::Sequent;

/// A cut formula as its author wrote it: the formula, one-sided, and what
/// the syntax it was written in records of its occurrences, which a
/// session keeps for the two occurrences a cut adds, the formula on the
/// right of `⊢` in the cut's first premise and on the left in its
/// second, so that every inference of the proof prints as written.
///
/// Built from text by [`parse_within`](Self::parse_within) in the syntax
/// of the session's mode, or from a value: an [`IllSequent`] `⊢ C` (its
/// recorded sides), a [`ClassicalSequent`] `⊢ C` (how it was written) or
/// a one-sided [`Sequent`] `⊢ C` (read as its one-sided syntax writes it).
///
/// Needs the cargo feature `interactive` (on by default).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CutFormula {
    /// The formula, as the one root of a sequent of its own.
    formula: Sequent,
    /// The intuitionistic sides of its occurrences as written, the formula
    /// in output position, where it was written so.
    sides: Option<Box<[Side]>>,
    /// How its occurrences were written, the formula right of `⊢`, where it
    /// was written as classical text.
    writing: Option<Box<[Written]>>,
}

impl CutFormula {
    /// Reads a cut formula written as text in the syntax of `mode`: in an
    /// intuitionistic mode as an intuitionistic formula, strictly or
    /// one-sided as `input` says (as a goal `⊢ C` is read), and else as a
    /// classical formula, as written. A parse error points into `text`.
    ///
    /// Needs the cargo feature `parse` (on by default).
    ///
    /// # Errors
    ///
    /// [`Error::Parse`] for text that is no formula,
    /// [`Error::CutFormula`] for text of another number of formulas than
    /// one, [`Error::NotIntuitionistic`] in an intuitionistic mode for a
    /// formula that cannot be a goal, and [`Error::Refused`] past
    /// `limits.occurrences`.
    #[cfg(feature = "parse")]
    pub fn parse_within(
        text: &str,
        mode: Mode,
        input: &crate::InputOptions,
        limits: &Limits,
    ) -> Result<Self, Error> {
        /// What the text is read after: the formula as a goal.
        const TURNSTILE: &str = "|- ";
        let written = format!("{TURNSTILE}{text}");
        let shift = |e: Error| {
            let Error::Parse(mut e) = e else {
                return e;
            };
            let back = TURNSTILE.len();
            e.span = e.span.start.saturating_sub(back)..e.span.end.saturating_sub(back);
            e.span_utf16 =
                e.span_utf16.start.saturating_sub(back)..e.span_utf16.end.saturating_sub(back);
            if e.line == 1 {
                e.column = e.column.saturating_sub(back).max(1);
            }
            Error::Parse(e)
        };
        let formula = if mode.is_intuitionistic() {
            let ill = match input.intuitionistic {
                crate::IllSyntax::OneSided => {
                    let sequent = Sequent::parse_within(&written, limits).map_err(shift)?;
                    IllSequent::read_one_sided(sequent, limits)?
                }
                crate::IllSyntax::Strict => {
                    IllSequent::parse_within(&written, limits).map_err(shift)?
                }
            };
            Self::from(ill)
        } else {
            Self::from(ClassicalSequent::parse_within(&written, limits).map_err(shift)?)
        };
        match formula.formula.roots().len() {
            1 => Ok(formula),
            formulas => Err(Error::CutFormula { formulas }),
        }
    }

    /// Returns the formula, as the one root of a sequent of its own.
    pub fn sequent(&self) -> &Sequent {
        &self.formula
    }

    /// Returns the formula with its constants `names` bound by a `∀`
    /// each around it, the first outermost, its occurrences' records as
    /// written with the binders' entries before them, which stand where
    /// the formula does: read again from the one-sided text, which every
    /// printer writes so that it reads back, a name that an inner binder
    /// takes written apart.
    ///
    /// # Errors
    ///
    /// [`Error::Refused`] past `limits.occurrences`.
    #[cfg(feature = "parse")]
    fn bound(&self, names: &[&str], limits: &Limits) -> Result<Self, Error> {
        let forest = Forest::within(&self.formula, &Limits::UNBOUNDED)?;
        let text = format!(
            "|- forall {}. ({})",
            names.join(" "),
            forest.formula(forest.roots()[0])
        );
        let formula = Sequent::parse_within(&text, limits)?;
        // The binders come first, each written where the formula is.
        let k = names.len();
        let sides = self.sides.as_ref().map(|sides| {
            let mut bound = vec![sides[0]; k];
            bound.extend_from_slice(sides);
            bound.into_boxed_slice()
        });
        let writing = self.writing.as_ref().map(|writing| {
            let mut bound = vec![Written::new(writing[0].side(), false); k];
            bound.extend_from_slice(writing);
            bound.into_boxed_slice()
        });
        Ok(Self {
            formula,
            sides,
            writing,
        })
    }

    /// Returns the formula as it is: without a reader of text no name is
    /// taken for an eigenvariable.
    #[cfg(not(feature = "parse"))]
    fn bound(&self, _: &[&str], _: &Limits) -> Result<Self, Error> {
        Ok(self.clone())
    }
}

impl From<IllSequent> for CutFormula {
    /// The cut formula of an intuitionistic sequent `⊢ C` and its sides
    /// as written.
    fn from(sequent: IllSequent) -> Self {
        let reading = sequent.reading();
        let sides = sequent
            .forest()
            .ids()
            .map(|o| reading.position(o))
            .collect();
        Self {
            formula: sequent.into_sequent(),
            sides: Some(sides),
            writing: None,
        }
    }
}

impl From<ClassicalSequent> for CutFormula {
    /// The cut formula of a classical sequent `⊢ C` as it was written.
    fn from(sequent: ClassicalSequent) -> Self {
        let writing = sequent.writing().into();
        Self {
            formula: sequent.into_sequent(),
            sides: None,
            writing: Some(writing),
        }
    }
}

impl From<Sequent> for CutFormula {
    /// The cut formula of a one-sided sequent `⊢ C`, read as the one-sided
    /// syntax writes it.
    fn from(formula: Sequent) -> Self {
        Self {
            formula,
            sides: None,
            writing: None,
        }
    }
}

impl Interactive {
    /// Cuts an open goal `Γ, Δ` on the formula `A` that `formula` names:
    /// `A` and its dual join the session's sequent as a cut formula
    /// ([`Sequent::cuts`]), and the goal's premises `Γ, A` and `Δ, A⊥`
    /// open as goals, in that order, the formulas at the positions `left`
    /// going to the first, the others to the second. Intuitionistically
    /// that is `Γ ⊢ A` and `Δ, A ⊢ C`: the goal's goal goes to the second
    /// premise. The occurrences of `A` and `A⊥` are recorded as `formula`
    /// was written, `A` right of `⊢` and `A⊥` left of it, so that the
    /// two-sided views print them as written. [`undo`](Self::undo) takes
    /// the cut back, its formula with it. A cut is refused in an ordered
    /// mode, whose dual turns a formula round. In a session with
    /// quantifiers `A` is a closed formula of the sequent's symbols, which
    /// may have quantifiers of its own, and the goals' instances keep
    /// their terms and eigenvariables; a cut formula with a quantifier
    /// makes a session without one a session with quantifiers.
    ///
    /// # Errors
    ///
    /// [`Error::Step`] with [`StepError::NoGoal`] for a goal that is not
    /// open, [`StepError::Mode`] in an ordered mode, [`StepError::Split`]
    /// for a position past the goal or named twice, and
    /// [`StepError::Succedents`] in intuitionistic mode where a premise
    /// would not have one goal (the formulas going left must all be
    /// hypotheses), [`StepError::CutSymbol`] for a cut formula naming a
    /// symbol the sequent has with another arity or role, or in a session
    /// with quantifiers one it has not (an eigenvariable's letter among
    /// them); [`Error::CutFormula`] for a formula of another number
    /// of formulas than one; [`Error::NotIntuitionistic`] in
    /// intuitionistic mode for a one-sided formula that cannot be a goal;
    /// [`Error::Refused`] when the sequent with the cut formula has more
    /// occurrences than `limits.occurrences`. Nothing changes then.
    pub fn cut(
        &mut self,
        goal: GoalId,
        formula: &CutFormula,
        left: &[usize],
        limits: &Limits,
    ) -> Result<[GoalId; 2], Error> {
        let sequent = self.goal(goal)?.to_vec();
        if self.mode.is_ordered() {
            return Err(StepError::Mode {
                rule: Rule::Cut.into(),
                mode: self.mode,
            }
            .into());
        }
        let mut used = vec![false; sequent.len()];
        for &p in left {
            if p >= sequent.len() || std::mem::replace(&mut used[p], true) {
                return Err(StepError::Split { position: p }.into());
            }
        }
        let named = if self.forest.sequent().is_first_order() || formula.formula.is_first_order() {
            self.cut_symbols(goal, &formula.formula)?
        } else {
            Vec::new()
        };
        // A cut formula naming eigenvariables is held closed, bound by a
        // `∀` per eigenvariable, and cut on its body's instance.
        let bound;
        let formula = if named.is_empty() {
            formula
        } else {
            let names: Vec<&str> = named.iter().map(|(name, _)| name.as_str()).collect();
            bound = formula.bound(&names, limits)?;
            &bound
        };
        let depth = named.len() as u32;
        let mut extended = self.forest.sequent().clone();
        extended.add_cut(formula.formula.clone())?;
        let forest = Forest::within(&extended, limits)?;
        // The new trees follow every occurrence the session had, a forest
        // of fewer than `u32::MAX`.
        let a = OccId::new(self.forest.len() as u32);
        let size = forest.size(a) as usize;
        let dual = OccId::new(a.get() + forest.size(a));
        debug_assert_eq!(forest.cut_pairs().last(), Some((a, dual)));
        let reading = match &self.reading {
            Some((positions, goal)) => {
                let sides = match &formula.sides {
                    Some(sides) => sides.clone(),
                    None => {
                        let ill = IllSequent::read_one_sided(formula.formula.clone(), limits)?;
                        let reading = ill.reading();
                        ill.forest().ids().map(|o| reading.position(o)).collect()
                    }
                };
                let mut extended = positions.to_vec();
                extended.extend_from_slice(&sides);
                extended.extend(sides.iter().map(|&side| !side));
                debug_assert!(
                    Reading::of_mode(&forest, self.mode)
                        .is_ok_and(|r| forest.ids().all(|o| r.position(o) == extended[o.index()])),
                    "the sides recorded as written are the reading's"
                );
                Some((extended.into_boxed_slice(), *goal))
            }
            None => None,
        };
        let writing = match &self.writing {
            None => None,
            Some(writing) => Some({
                let record = match &formula.writing {
                    Some(record) => record.clone(),
                    // Of fewer occurrences than the forest just built.
                    None => crate::occurrences::written::one_sided(&Forest::within(
                        &formula.formula,
                        &Limits::UNBOUNDED,
                    )?),
                };
                let mut extended = writing.to_vec();
                extended.extend_from_slice(&record);
                // Written left of `⊢`, every occurrence takes the other side.
                extended.extend(
                    record
                        .iter()
                        .map(|w| Written::new(!w.side(), w.is_implication())),
                );
                extended.into_boxed_slice()
            }),
        };
        debug_assert_eq!(forest.len(), self.forest.len() + 2 * size);
        // The members past the forest move past the cut formulas' trees.
        let (from, to) = (self.forest.len() as u32, forest.len() as u32);
        let shift = |m: Member| {
            if m.get() < from {
                m
            } else {
                Member::new(m.get() - from + to)
            }
        };
        let premises = {
            let positions = reading.as_ref().map(|(p, _)| &**p);
            let shifted: Vec<Member> = sequent.iter().map(|&m| shift(m)).collect();
            // An occurrence of the new forest, the cut formulas' among
            // them, or a member moved past it.
            let occurrence = |m: Member| {
                if m.get() < to {
                    m.occ()
                } else {
                    self.occurrence(Member::new(m.get() - to + from))
                }
            };
            // The bodies' occurrences stand for their instances here.
            cut_premises(
                positions,
                &shifted,
                occurrence,
                Member::new(a.get() + depth),
                Member::new(dual.get() + depth),
                &used,
            )?
        };
        // The bodies' occurrences and the instances the cut is on.
        let mut instances: Vec<(Member, Member)> = Vec::new();
        match self.fo.as_mut() {
            Some(q) => {
                let mark = q.mark();
                q.rebase(&self.forest, &forest);
                for inference in &mut self.inferences {
                    for m in &mut inference.sequent {
                        *m = shift(*m);
                    }
                }
                if !named.is_empty() {
                    let mut frame = None;
                    for &(_, e) in &named {
                        let t = q.eigen_term(e);
                        frame = Some(q.frame(frame, t));
                    }
                    for o in [a, dual] {
                        let o = OccId::new(o.get() + depth);
                        instances.push((Member::from(o), q.member(o, frame)));
                    }
                    // Fewer steps than inferences, which an `InfId` numbers.
                    q.made(self.history.len() as u32, mark);
                }
            }
            // A cut formula with a quantifier makes the session one with
            // quantifiers; it had no member past its forest.
            None if forest.has_binders() => {
                let mut q = Quantified::new(&forest);
                q.limit = limits.terms;
                self.fo = Some(Box::new(q));
            }
            None => {}
        }
        self.forest = forest;
        self.reading = reading;
        self.writing = writing;
        let premises = premises.map(|premise| {
            let mut premise = premise.into_members();
            if !instances.is_empty() {
                for m in &mut premise {
                    if let Some(&(_, instance)) = instances.iter().find(|(o, _)| o == m) {
                        *m = instance;
                    }
                }
                premise.sort_unstable();
            }
            premise
        });
        let cut = match instances.first() {
            Some(&(_, instance)) => {
                self.show();
                instance
            }
            None => Member::from(a),
        };
        let base = self.inferences.len() as u32;
        let ids = [InfId::new(base), InfId::new(base + 1)];
        for premise in premises {
            self.inferences.push(Inference {
                sequent: premise,
                rule: Rule::Open.into(),
                principal: None,
                premises: vec![],
                times: 1,
                cut: None,
                node: None,
            });
        }
        let inference = &mut self.inferences[goal.index()];
        inference.rule = Rule::Cut.into();
        inference.principal = None;
        inference.premises = ids.to_vec();
        inference.cut = Some(cut);
        self.history.push(goal.inf());
        Ok(ids.map(GoalId::of))
    }

    /// Returns the member a cut names and its dual's: a cut formula's root
    /// and its dual's, or, for a cut on an instance, the member of the body
    /// below a cut formula's `∀`s under a frame of as many terms and the
    /// member of the dual's body under the same frame; `None` for any other
    /// member.
    pub(super) fn cut_pair(&self, cut: Member) -> Option<(Member, Member)> {
        let (o, frame) = match &self.fo {
            Some(q) => q.instance(cut),
            None => (cut.occurrence(&self.forest)?, None),
        };
        let root = self.forest.root(o);
        let (_, dual) = self.forest.cut_pairs().find(|&(a, _)| a == root)?;
        let Some(frame) = frame else {
            return (o == root).then_some((cut, Member::from(dual)));
        };
        let q = self.fo.as_ref()?;
        // A frame is as long as its occurrence's binders.
        let depth = q.frame_terms(Some(frame)).len() as u32;
        let binders = (root.get()..root.get() + depth)
            .all(|b| self.forest.kind(OccId::new(b)) == crate::sequents::Kind::Forall);
        (o.get() == root.get() + depth && binders)
            .then(|| q.find(OccId::new(dual.get() + depth), Some(frame)))
            .flatten()
            .map(|d| (cut, d))
    }

    /// Returns the eigenvariables a cut formula on `goal` names, by the
    /// letters the session writes them with, each with its name, in the
    /// order of the formula's symbols. Refuses one the goal is not in the
    /// scope of, as a witness's; a symbol the sequent has with another
    /// arity or role, which joining the two would rename apart; and in a
    /// session with quantifiers any other symbol the sequent has not: the
    /// eigenvariables take the letters no symbol takes, so a new symbol
    /// could take one's letter.
    fn cut_symbols(
        &self,
        goal: GoalId,
        formula: &Sequent,
    ) -> Result<Vec<(String, crate::sequents::Eigen)>, StepError> {
        let ours: crate::hash::HashMap<&str, (u32, bool)> =
            symbol_roles(self.forest.sequent()).into_iter().collect();
        // An eigenvariable's letter reads as a constant: a nullary symbol
        // that is no predicate. The text is read without the session, so
        // only a reader of text has the letters.
        let letters: crate::hash::HashMap<String, crate::sequents::Eigen> = match &self.fo {
            Some(q) if cfg!(feature = "parse") => crate::occurrences::instances::eigen_names(
                &self.forest.sequent().fo,
                &q.shown.eigenvariables(),
            )
            .into_iter()
            .map(|(e, name)| (name, e))
            .collect(),
            _ => crate::hash::HashMap::default(),
        };
        let scope = if letters.is_empty() {
            Vec::new()
        } else {
            self.scope(goal.inf())
        };
        let mut named = Vec::new();
        for (name, role) in symbol_roles(formula) {
            let known = ours.get(name);
            if known.is_none()
                && role == (0, false)
                && let Some(&e) = letters.get(name)
            {
                if !scope.contains(&e) {
                    return Err(StepError::Eigenvariable {
                        name: name.to_owned(),
                    });
                }
                named.push((name.to_owned(), e));
            } else if known.is_some_and(|&r| r != role) || (known.is_none() && self.fo.is_some()) {
                return Err(StepError::CutSymbol {
                    name: name.to_owned(),
                });
            }
        }
        Ok(named)
    }

    /// Takes back the cut formula of the cut that [`undo`](Self::undo)
    /// retracts: the last one added, since the steps after it were
    /// retracted before it.
    pub(super) fn uncut(&mut self) {
        let mut sequent = self.forest.sequent().clone();
        sequent.pop_cut();
        // A forest of fewer occurrences than the one it had.
        let forest = Forest::within(&sequent, &Limits::UNBOUNDED).expect("a smaller forest");
        if !forest.has_binders() {
            self.fo = None;
        }
        if let Some(q) = self.fo.as_mut() {
            q.rebase(&self.forest, &forest);
            // The members past the forest move back over the cut formula's
            // trees, which no inference left names.
            let (from, to) = (self.forest.len() as u32, forest.len() as u32);
            for inference in &mut self.inferences {
                for m in &mut inference.sequent {
                    if m.get() >= from {
                        *m = Member::new(m.get() - from + to);
                    }
                }
            }
        }
        self.forest = forest;
        let len = self.forest.len();
        if let Some((positions, goal)) = &mut self.reading {
            *positions = positions[..len].into();
            debug_assert!(goal.index() < len);
        }
        if let Some(writing) = &mut self.writing {
            *writing = writing[..len].into();
        }
    }

    /// Checks that a cut read back names the cut formula `a` of its own
    /// pair and that its premises are what the cut yields on its sequent,
    /// the split recovered from the first premise.
    #[cfg(feature = "serialize")]
    pub(super) fn replay_cut(&self, id: InfId) -> Result<(), Error> {
        let inconsistent = |reason| Error::InconsistentSession { reason };
        let inference = &self.inferences[id.index()];
        let sequent = &inference.sequent;
        let Some((a, dual)) = inference.cut.and_then(|a| self.cut_pair(a)) else {
            return Err(inconsistent(
                "a cut names no cut formula, nor the instance of one's body that a session cuts on",
            ));
        };
        // A cut on an instance holds the goal's eigenvariables, as a
        // given witness does.
        if let Some(q) = self.fo.as_ref()
            && let (_, Some(frame)) = q.instance(a)
        {
            let scope = self.scope(id);
            let terms = q.terms(&self.forest);
            let held = q.frame_terms(Some(frame)).into_iter().all(|t| {
                matches!(terms.term(t), crate::sequents::FoTerm::Eigen(e) if scope.contains(&e))
            });
            if !held {
                return Err(inconsistent(
                    "a cut's instance holds a term that is no eigenvariable in its goal's scope",
                ));
            }
        }
        if self.mode.is_ordered() {
            return Err(inconsistent("an ordered mode has no cut"));
        }
        let &[first, _] = inference.premises.as_slice() else {
            return Err(inconsistent("a cut has two premises"));
        };
        let mut context = Multiset::of(self.inferences[first.index()].sequent.iter().copied());
        if !context.remove(a) {
            return Err(inconsistent("a cut's first premise lacks its cut formula"));
        }
        // Both are sorted, so each member of the context is the next equal
        // one of the sequent.
        let mut used = vec![false; sequent.len()];
        let mut at = 0;
        for &o in context.as_slice() {
            while at < sequent.len() && sequent[at] < o {
                at += 1;
            }
            if at == sequent.len() || sequent[at] != o {
                return Err(inconsistent(
                    "a premise holds a formula its conclusion lacks",
                ));
            }
            used[at] = true;
            at += 1;
        }
        let positions = self.reading.as_ref().map(|(p, _)| &**p);
        let expected = cut_premises(positions, sequent, |m| self.occurrence(m), a, dual, &used)?;
        let same = expected.iter().zip(&inference.premises).all(|(e, p)| {
            e.as_slice()
                .iter()
                .copied()
                .eq(self.inferences[p.index()].sequent.iter().copied())
        });
        if same {
            Ok(())
        } else {
            Err(inconsistent(
                "an inference's premises are not what its rule yields",
            ))
        }
    }
}

/// Returns every symbol of a sequent in the order of its table, with its
/// arity and whether it is a predicate: of a sequent without first-order
/// terms, its atoms as nullary predicates.
fn symbol_roles(sequent: &Sequent) -> Vec<(&str, (u32, bool))> {
    let fo = &sequent.fo;
    let mut roles: Vec<(&str, (u32, bool))> = fo
        .symbols
        .iter()
        .map(|(name, arity)| (name.as_str(), (*arity, false)))
        .collect();
    for &(p, _) in &fo.atoms {
        roles[p.index()].1.1 = true;
    }
    if fo.atoms.is_empty() {
        roles.extend(
            sequent
                .atom_names()
                .iter()
                .map(|name| (name.as_str(), (0, true))),
        );
    }
    roles
}

/// Returns the premises of a cut on `a`, whose dual is `dual`, of the goal
/// `sequent`, the formulas marked in `left` going to the first: in
/// intuitionistic mode, under `positions` of each member's occurrence,
/// each with one goal.
fn cut_premises(
    positions: Option<&[Side]>,
    sequent: &[Member],
    occurrence: impl Fn(Member) -> OccId,
    a: Member,
    dual: Member,
    left: &[bool],
) -> Result<[Multiset<Member>; 2], StepError> {
    let (mut first, mut second) = (Multiset::new(), Multiset::new());
    for (&o, &goes_left) in sequent.iter().zip(left) {
        if goes_left {
            first.insert(o);
        } else {
            second.insert(o);
        }
    }
    first.insert(a);
    second.insert(dual);
    if let Some(positions) = positions {
        for premise in [&first, &second] {
            let outputs = premise
                .as_slice()
                .iter()
                .filter(|&&m| positions[occurrence(m).index()] == Side::Output)
                .count();
            if outputs != 1 {
                return Err(StepError::Succedents { count: outputs });
            }
        }
    }
    Ok([first, second])
}

/// Returns proofs with cuts made of `proofs`: per proof of a commutative
/// mode, its sequent's session cut on one of its formulas `C` (the others
/// going to the premise that proves `C`, or intuitionistically `C ⊢ C`
/// as the first premise where `C` is a hypothesis), then one of the two
/// goals cut again on one of its formulas, every goal closed by the
/// search; each with its mode. The second cut's formula may be the first
/// cut's.
#[cfg(all(test, feature = "parse"))]
pub(crate) fn with_cuts(
    proofs: &[(crate::Proof, Mode)],
    rng: &mut crate::search::generate::Rng,
) -> Vec<(crate::Proof, Mode)> {
    let limits = Limits::default();
    let options = crate::Options::default();
    let view = crate::ViewOptions::default();
    let mut made = Vec::new();
    for (proof, mode) in proofs {
        if mode.is_ordered() || proof.goal().is_some() {
            continue;
        }
        let Ok(mut session) = Interactive::new(proof.sequent(), *mode) else {
            continue;
        };
        let mut goal = session.goals().next().expect("the root is open");
        for _ in 0..2 {
            let members = session.goal(goal).expect("open").to_vec();
            let p = rng.below(members.len());
            let o = members[p].occ();
            let (text, left): (String, Vec<usize>) = match session.reading() {
                Some(reading) if reading.position(o) == Side::Input => {
                    (reading.formula(o).to_string(), vec![p])
                }
                reading => {
                    let text = match reading {
                        Some(reading) => reading.formula(o).to_string(),
                        None => session.forest().formula(o).to_string(),
                    };
                    (text, (0..members.len()).filter(|&q| q != p).collect())
                }
            };
            let formula =
                CutFormula::parse_within(&text, *mode, &crate::InputOptions::default(), &limits)
                    .expect("a formula as the session prints it");
            let premises = session
                .cut(goal, &formula, &left, &limits)
                .expect("a cut that leaves one goal per premise");
            goal = premises[rng.below(2)];
        }
        let mut polls = 0;
        let closed = session.close_all(&options, &view, &limits, |_| {
            polls += 1;
            polls > 20_000
        });
        if closed.iter().all(|(_, c)| {
            c.as_ref()
                .is_ok_and(|c| c.grafted.is_ok() && c.outcome.verdict.proof().is_some())
        }) && let Ok(proof) = session.proof(&limits, |_| false)
        {
            made.push((proof, *mode));
        }
    }
    made
}

#[cfg(all(test, feature = "parse"))]
mod tests {
    use super::*;
    use crate::proofs::{Named, Node};
    use crate::{InputOptions, Options, TwoSided, ViewOptions};

    /// Reads a cut formula of `mode` with the default input options.
    fn formula(text: &str, mode: Mode) -> Result<CutFormula, Error> {
        CutFormula::parse_within(text, mode, &InputOptions::default(), &Limits::default())
    }

    /// A cut opens its two premises as goals, the cut formula right of `⊢`
    /// in the first and left of it in the second, as typed; the finished
    /// proof has the cut and checks, the two-sided view names it `cut`,
    /// the session writes its formula as typed, a session started on its
    /// sequent starts from the conclusion and the same cut made again
    /// gives the same sequent, and `undo` takes the cut back with its
    /// formula.
    #[test]
    fn a_cut_opens_two_goals_and_undo_takes_it_back() {
        let limits = Limits::default();
        let written: ClassicalSequent = "A, A -o B |- B".parse().unwrap();
        let mut session = Interactive::of_classical(&written, Mode::CLASSICAL, &limits).unwrap();
        let before = state(&session);
        let root = session.goals().next().unwrap();
        let [first, second] = session
            .cut(
                root,
                &formula("A -o B", Mode::CLASSICAL).unwrap(),
                &[1],
                &limits,
            )
            .unwrap();
        let shown = |g| -> Vec<String> {
            let goal = session.goal(g).unwrap();
            goal.iter()
                .map(|&m| session.forest().formula(m.occ()).to_string())
                .collect()
        };
        assert_eq!(shown(first), ["A ⊗ ~B", "~A ⅋ B"]);
        assert_eq!(shown(second), ["~A", "B", "A ⊗ ~B"]);
        session.close_all(
            &Options::default(),
            &ViewOptions::default(),
            &limits,
            |_| false,
        );
        let proof = session.proof(&limits, |_| false).unwrap();
        assert_eq!(
            proof
                .nodes()
                .iter()
                .filter(|n| matches!(n, Node::Cut(..)))
                .count(),
            1
        );
        assert_eq!(proof.check(Mode::CLASSICAL), Ok(()));
        let derivation = session.derivation().unwrap();
        let cut = derivation.inference(derivation.root());
        assert_eq!(cut.rule(), Named::from(Rule::Cut));
        let view = TwoSided::new(&derivation, session.writing().unwrap(), &limits).unwrap();
        let root = view.root();
        assert_eq!(view.rule(root).name(), "cut");
        let premises: Vec<String> = view
            .premises(root)
            .iter()
            .map(|&p| view.sequent(p).to_string())
            .collect();
        assert_eq!(premises, ["A ⊸ B ⊢ A ⊸ B", "A, A ⊸ B ⊢ B"]);
        let typed = session
            .formula_as_written(cut.cut().unwrap())
            .unwrap()
            .to_string();
        assert_eq!(typed, "A ⊸ B");
        let mut again = Interactive::within(session.sequent(), Mode::CLASSICAL, &limits).unwrap();
        assert!(!again.forest().has_cuts());
        assert_eq!(state(&again), before);
        let root = again.goals().next().unwrap();
        let typed = formula(&typed, Mode::CLASSICAL).unwrap();
        again.cut(root, &typed, &[1], &limits).unwrap();
        assert_eq!(again.sequent(), session.sequent());
        while session.undo().is_some() {}
        assert_eq!(state(&session), before);
        assert!(!session.forest().has_cuts());
    }

    /// What a test compares sessions by: the sequent, the forest's size
    /// and the goals.
    fn state(session: &Interactive) -> String {
        let goals: Vec<_> = session
            .goals()
            .map(|g| session.goal(g).unwrap().to_vec())
            .collect();
        format!(
            "{:?} {} {goals:?}",
            session.sequent(),
            session.forest().len()
        )
    }

    /// A cut is refused, and nothing changes, for a goal that is not
    /// open, in an ordered mode, for a split past the goal or naming a
    /// formula twice, where a premise would have two goals or none, and
    /// for text of another number of formulas than one; a parse error
    /// points into the text typed.
    #[test]
    fn refusals() {
        let limits = Limits::default();
        let classical = formula("A", Mode::CLASSICAL).unwrap();
        let sequent: Sequent = "A |- A".parse().unwrap();
        let mut session = Interactive::new(&sequent, Mode::INTUITIONISTIC).unwrap();
        let root = session.goals().next().unwrap();
        let ill = formula("A", Mode::INTUITIONISTIC).unwrap();
        let step = |e: Result<[GoalId; 2], Error>| match e {
            Err(Error::Step(e)) => e,
            other => panic!("{other:?}"),
        };
        // The goal A going left with the cut formula: ⊢ A, A has two goals.
        assert_eq!(
            step(session.cut(root, &ill, &[1], &limits)),
            StepError::Succedents { count: 2 }
        );
        assert_eq!(
            step(session.cut(root, &ill, &[2], &limits)),
            StepError::Split { position: 2 }
        );
        assert_eq!(
            step(session.cut(root, &ill, &[0, 0], &limits)),
            StepError::Split { position: 0 }
        );
        assert!(!session.forest().has_cuts());
        let [first, _] = session.cut(root, &ill, &[0], &limits).unwrap();
        assert!(matches!(
            step(session.cut(root, &ill, &[], &limits)),
            StepError::NoGoal { .. }
        ));
        // A classical formula typed in intuitionistic mode is refused by
        // the strict syntax; one-sided, ~A cannot be a goal.
        assert!(matches!(
            formula("A par B", Mode::INTUITIONISTIC),
            Err(Error::Parse(_))
        ));
        let one_sided = CutFormula::from("|- ~A".parse::<Sequent>().unwrap());
        assert!(matches!(
            session.cut(first, &one_sided, &[], &limits),
            Err(Error::NotIntuitionistic(_))
        ));
        assert!(matches!(
            formula("A, B", Mode::CLASSICAL),
            Err(Error::CutFormula { formulas: 2 })
        ));
        let Err(Error::Parse(e)) = formula("A * * B", Mode::CLASSICAL) else {
            panic!("no formula");
        };
        assert_eq!((e.span, e.column), (4..5, 5));
        let cyclic: Sequent = "|- A, ~A".parse().unwrap();
        let mut ordered = Interactive::new(&cyclic, Mode::CYCLIC).unwrap();
        let root = ordered.goals().next().unwrap();
        assert!(matches!(
            step(ordered.cut(root, &classical, &[], &limits)),
            StepError::Mode { .. }
        ));
    }
}
