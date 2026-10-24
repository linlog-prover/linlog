// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use crate::hash::HashMap;
use crate::limits::{Limits, Refusal};
use crate::occurrences::{IllConnective, IllSequent, Side};
use crate::sequents::{Atom, Sequent, Term, TermId, Visit, name};
use crate::wire::{self, Readable};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::borrow::Cow;

/// The serialized form of an intuitionistic formula: the connectives of
/// intuitionistic linear logic alone, so that a classical tag is refused
/// as an unknown variant. Its tags are part of the interchange format.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename = "I")]
enum Expression {
    /// An atom, by its index among the names.
    #[serde(rename = "V")]
    Atom(u32),
    /// `1`
    #[serde(rename = "1")]
    One,
    /// `0`
    #[serde(rename = "0")]
    Zero,
    /// `⊤`
    #[serde(rename = "⊤")]
    Top,
    /// `A ⊗ B`
    #[serde(rename = "⊗")]
    Tensor(u32, u32),
    /// `A ⊸ B`, the antecedent first.
    #[serde(rename = "⊸")]
    Lollipop(u32, u32),
    /// `A & B`
    #[serde(rename = "&")]
    With(u32, u32),
    /// `A ⊕ B`
    #[serde(rename = "⊕")]
    Plus(u32, u32),
    /// `!A`
    #[serde(rename = "!")]
    Bang(u32),
}

impl Expression {
    /// Returns the term of the same shape, whose bounds are the
    /// expression's: its operands and its atom.
    fn shape(self) -> Term {
        let t = TermId::new;
        match self {
            Expression::Atom(a) => Term::Atom(Atom::new(a)),
            Expression::One => Term::One,
            Expression::Zero => Term::Zero,
            Expression::Top => Term::Top,
            Expression::Tensor(k, l) | Expression::Lollipop(k, l) => Term::Tensor(t(k), t(l)),
            Expression::With(k, l) | Expression::Plus(k, l) => Term::With(t(k), t(l)),
            Expression::Bang(k) => Term::Bang(t(k)),
        }
    }

    /// Returns the operands, each with the side it stands on when the
    /// expression stands on `side`: the antecedent of `⊸` on the other.
    fn operands(self, side: Side) -> [Option<(u32, Side)>; 2] {
        match self {
            Expression::Atom(_) | Expression::One | Expression::Zero | Expression::Top => {
                [None, None]
            }
            Expression::Bang(k) => [Some((k, side)), None],
            Expression::Lollipop(k, l) => [Some((k, !side)), Some((l, side))],
            Expression::Tensor(k, l) | Expression::With(k, l) | Expression::Plus(k, l) => {
                [Some((k, side)), Some((l, side))]
            }
        }
    }

    /// Returns the one-sided term of the expression on `side`, its
    /// operands lowered to `left` and `right`.
    fn lowered(self, side: Side, left: TermId, right: TermId) -> Term {
        let output = side == Side::Output;
        match self {
            Expression::Atom(a) if output => Term::Atom(Atom::new(a)),
            Expression::Atom(a) => Term::DualAtom(Atom::new(a)),
            Expression::One if output => Term::One,
            Expression::One => Term::Bot,
            Expression::Zero if output => Term::Zero,
            Expression::Zero => Term::Top,
            Expression::Top if output => Term::Top,
            Expression::Top => Term::Zero,
            Expression::Tensor(..) if output => Term::Tensor(left, right),
            Expression::Tensor(..) => Term::Par(left, right),
            Expression::Lollipop(..) if output => Term::Par(left, right),
            Expression::Lollipop(..) => Term::Tensor(left, right),
            Expression::With(..) if output => Term::With(left, right),
            Expression::With(..) => Term::Plus(left, right),
            Expression::Plus(..) if output => Term::Plus(left, right),
            Expression::Plus(..) => Term::With(left, right),
            Expression::Bang(_) if output => Term::Bang(left),
            Expression::Bang(_) => Term::Quest(left),
        }
    }
}

/// The serialized form of an intuitionistic sequent.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Ill {
    /// The wire level.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "wire::version"
    )]
    version: Option<u32>,
    /// The arena, every operand before the term that names it.
    terms: Vec<Expression>,
    /// The atom names.
    atoms: Vec<String>,
    /// The hypotheses, as arena indices, in the order written.
    hypotheses: Vec<u32>,
    /// The goal, as an arena index.
    goal: u32,
}

impl From<&IllSequent> for Ill {
    /// Writes the sequent's formulas as its reading reads them, each
    /// formula of a one-sided term on one side once.
    fn from(sequent: &IllSequent) -> Self {
        let reading = sequent.reading();
        let forest = sequent.forest();
        let mut terms = Vec::new();
        let mut index = HashMap::<(TermId, Side), u32>::default();
        // Each formula once, however many one-sided terms read as it.
        let mut written = HashMap::<Expression, u32>::default();
        // Each formula after its operands, in the order written.
        for &root in forest.roots() {
            for visit in reading.walk(root) {
                // A formula without operands is entered and never left.
                let o = match visit {
                    Visit::Exit(o) => o,
                    Visit::Enter(o) if forest.left(o).is_none() => o,
                    Visit::Enter(_) | Visit::Between(..) => continue,
                };
                let key = (forest.term(o), reading.position(o));
                if index.contains_key(&key) {
                    continue;
                }
                let at = |k: Option<_>| {
                    let k = k.expect("a connective has its operands");
                    index[&(forest.term(k), reading.position(k))]
                };
                let (l, r) = reading.operands(o);
                use IllConnective as C;
                let expression = match reading.connective(o) {
                    C::Atom => Expression::Atom(forest.atom(o).expect("an atom").get()),
                    C::One => Expression::One,
                    C::Zero => Expression::Zero,
                    C::Top => Expression::Top,
                    C::Bang => Expression::Bang(at(l)),
                    C::Tensor => Expression::Tensor(at(l), at(r)),
                    C::With => Expression::With(at(l), at(r)),
                    C::Plus => Expression::Plus(at(l), at(r)),
                    C::Lollipop | C::Under | C::Over => Expression::Lollipop(at(l), at(r)),
                };
                // Fewer terms than the forest has occurrences, which fit a `u32`.
                let n = *written.entry(expression).or_insert_with(|| {
                    terms.push(expression);
                    terms.len() as u32 - 1
                });
                index.insert(key, n);
            }
        }
        let at = |o| index[&(forest.term(o), reading.position(o))];
        Self {
            version: None,
            terms,
            atoms: forest.sequent().atom_names().to_vec(),
            hypotheses: reading.hypotheses().map(at).collect(),
            goal: at(reading.goal()),
        }
    }
}

impl Ill {
    /// Returns the sequent the form holds, lowered within the limits:
    /// every formula on the side it stands on, each term on a side once.
    fn within(mut self, limits: &Limits) -> Result<IllSequent, crate::Error> {
        for name in &mut self.atoms {
            if let Cow::Owned(composed) = name::normalized(name) {
                *name = composed;
            }
            name::check(name)?;
        }
        let len = self.terms.len();
        // Each term lowers to two at most, numbered by a `u32`.
        let most = u64::from(u32::MAX / 2);
        if len as u64 > most {
            return Err(crate::Error::Refused(Refusal::Index {
                what: crate::limits::Space::Term,
                count: len as u64,
                most,
            }));
        }
        for (n, e) in self.terms.iter().enumerate() {
            e.shape().check_bounds(self.atoms.len() as u32, n as u32)?;
        }
        let roots = self.hypotheses.iter().chain([&self.goal]);
        if let Some(&n) = roots.clone().find(|&&n| n as usize >= len) {
            return Err(crate::Error::IndexOutOfBounds {
                space: crate::limits::Space::Term,
                index: n as usize,
                len,
            });
        }
        // Which sides each term is needed on, parents before operands.
        let bit = |side: Side| if side == Side::Input { 1u8 } else { 2 };
        let mut needed = vec![0u8; len];
        for &h in &self.hypotheses {
            needed[h as usize] |= bit(Side::Input);
        }
        needed[self.goal as usize] |= bit(Side::Output);
        for n in (0..len).rev() {
            for side in [Side::Input, Side::Output] {
                if needed[n] & bit(side) != 0 {
                    for (k, s) in self.terms[n].operands(side).into_iter().flatten() {
                        needed[k as usize] |= bit(s);
                    }
                }
            }
        }
        // The one-sided terms, operands first; at most two per term.
        let mut lowered = vec![[TermId::new(0); 2]; len];
        let mut terms = Vec::new();
        // At most two terms each, fewer than `u32::MAX` in all (above).
        for n in 0..len {
            for side in [Side::Input, Side::Output] {
                if needed[n] & bit(side) == 0 {
                    continue;
                }
                let [l, r] = self.terms[n]
                    .operands(side)
                    .map(|k| k.map_or(TermId::new(0), |(k, s)| lowered[k as usize][s as usize]));
                lowered[n][side as usize] = TermId::new(terms.len() as u32);
                terms.push(self.terms[n].lowered(side, l, r));
            }
        }
        let mut roots: Vec<TermId> = self
            .hypotheses
            .iter()
            .map(|&h| lowered[h as usize][Side::Input as usize])
            .collect();
        roots.push(lowered[self.goal as usize][Side::Output as usize]);
        let mut sequent = Sequent {
            terms,
            roots,
            atoms: self.atoms,
            // Fewer hypotheses than `u32::MAX`: each is a lowered term.
            antecedents: Some(self.hypotheses.len() as u32),
        };
        sequent.merge_atoms();
        let occurrences = sequent.occurrences();
        match limits.occurrences {
            Some(limit) if occurrences > limit => {
                return Err(crate::Error::Refused(Refusal::Occurrences {
                    occurrences,
                    limit,
                }));
            }
            _ => {}
        }
        IllSequent::lowered(sequent, limits)
    }
}

impl Serialize for IllSequent {
    /// Serializes the sequent as a document: its level, its arena of
    /// intuitionistic formulas, the atom names, the hypotheses and the
    /// goal.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut proxy = Ill::from(self);
        proxy.version = Some(wire::ILL);
        proxy.serialize(serializer)
    }
}

impl<'a> Deserialize<'a> for IllSequent {
    /// Deserializes a sequent within the default limits, as
    /// [`wire::upgrade`] does.
    fn deserialize<D: Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        Self::read(deserializer, &Limits::default())
    }
}

impl crate::sealed::Readable for IllSequent {}

impl Readable for IllSequent {
    const FORM: &'static str = "intuitionistic sequent";

    /// Reads a sequent, checks its arena's bounds and order and the atom
    /// names, and lowers it within the limits.
    fn read<'de, D: Deserializer<'de>>(deserializer: D, limits: &Limits) -> Result<Self, D::Error> {
        let proxy = Ill::deserialize(deserializer)?;
        proxy.within(limits).map_err(wire::fail)
    }
}
