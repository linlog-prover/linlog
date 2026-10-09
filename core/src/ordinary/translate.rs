// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The translations into linear logic. Each is a table of patterns: per
//! connective and per translation function (the side for the classical
//! translation, `t₀` or `t₁` for the 0/1 one), how many `!` stand on top
//! of the image, the linear connective under them, and each operand with
//! its function and the `!` added above it. The image of a sequent is
//! built from the table bottom-up over the arena, and the read-back walks
//! the forest of the image with the same table, so that every occurrence
//! is known as the ordinary formula it is the image of.

use super::{Formulas, Logic, Node, NodeId, Sequent, Side, Translation};
use crate::hash::HashMap;
use crate::sequents::{Atom, Term, TermId};
use crate::{Error, Mode};

/// The linear connective at the top of an image, under its `!`s.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Core {
    /// An atom; false in minimal logic is the atom [`FALSE`].
    Atom,
    /// A unit, as it stands right of `⊢`.
    Unit(Term),
    /// `⊗`
    Tensor,
    /// `⅋`
    Par,
    /// `&`
    With,
    /// `⊕`
    Plus,
    /// `⊸`, whose antecedent stands on the other side.
    Lollipop,
    /// No connective: the image is that of the operand, whose function is
    /// given (the classical negation).
    Pass,
}

/// An operand of a pattern: the subformula, its translation function and
/// the `!`s put above its image.
pub(super) type Operand = (NodeId, u8, u8);

/// The image of a formula under one translation function.
#[derive(Clone, Copy, Debug)]
pub(super) struct Pattern {
    /// The `!`s on top.
    pub(super) bangs: u8,
    /// The connective under them.
    pub(super) core: Core,
    /// Its operands, in order.
    pub(super) operands: [Option<Operand>; 2],
}

/// The name of the atom false is translated as in minimal logic, which no
/// atom of this crate's syntax can have, `false` being the constant there;
/// a TPTP problem can name an atom `false`, and then the atom of false
/// has as many `_` appended as make it a name of its own.
pub const FALSE: &str = "false";

/// The classical translation's function for a formula right of `⊢`.
const RIGHT: u8 = 0;
/// The classical translation's function for a formula left of `⊢`.
const LEFT: u8 = 1;
/// The 0/1 translation's negative function `t₀`.
const T0: u8 = 0;
/// The 0/1 translation's positive function `t₁`.
const T1: u8 = 1;

/// The extra nodes a translation refers to: the two implications of every
/// equivalence, and false, which `¬A` is `A → false` of.
#[derive(Clone, Debug)]
pub(super) struct Extra {
    /// Per node, the implications `A → B` and `B → A` of an equivalence.
    pub(super) iff: HashMap<NodeId, (NodeId, NodeId)>,
    /// The node of false.
    pub(super) falsity: NodeId,
}

impl Extra {
    /// Adds the extra nodes of every formula of the arena to it.
    fn add(formulas: &mut Formulas) -> Result<Self, Error> {
        let mut iff = HashMap::default();
        for n in 0..formulas.len() {
            let id = NodeId(n as u32);
            if let Node::Iff(a, b) = formulas.node(id) {
                let pair = (
                    formulas.add(Node::Implies(a, b))?,
                    formulas.add(Node::Implies(b, a))?,
                );
                iff.insert(id, pair);
            }
        }
        let falsity = formulas.add(Node::False)?;
        Ok(Self { iff, falsity })
    }
}

/// Returns the pattern of the formula `id` under `function` of the
/// translation, in `logic`.
pub(super) fn pattern(
    formulas: &Formulas,
    extra: &Extra,
    translation: Translation,
    logic: Logic,
    id: NodeId,
    function: u8,
) -> Pattern {
    use Core::*;
    let node = formulas.node(id);
    let make = |bangs, core, a: Option<Operand>, b: Option<Operand>| Pattern {
        bangs,
        core,
        operands: [a, b],
    };
    let falsity = extra.falsity;
    // An equivalence's implications, read as a conjunction.
    let (forward, backward) = match node {
        Node::Iff(..) => extra.iff[&id],
        _ => (id, id),
    };
    let minimal_false = logic == Logic::Minimal && node == Node::False;
    match translation {
        Translation::Affine => {
            let (f, flip) = (function, 1 - function);
            let right = function == RIGHT;
            match node {
                Node::Atom(_) => make(0, Atom, None, None),
                Node::True => make(
                    0,
                    Unit(if right { Term::Top } else { Term::Bot }),
                    None,
                    None,
                ),
                Node::False => make(
                    0,
                    Unit(if right { Term::Bot } else { Term::Top }),
                    None,
                    None,
                ),
                Node::Not(a) => make(0, Pass, Some((a, flip, 0)), None),
                Node::And(a, b) => make(
                    0,
                    if right { With } else { Par },
                    Some((a, f, 0)),
                    Some((b, f, 0)),
                ),
                Node::Or(a, b) => make(
                    0,
                    if right { Par } else { With },
                    Some((a, f, 0)),
                    Some((b, f, 0)),
                ),
                Node::Implies(a, b) => make(
                    0,
                    if right { Par } else { With },
                    Some((a, flip, 0)),
                    Some((b, f, 0)),
                ),
                Node::Iff(..) => make(
                    0,
                    if right { With } else { Par },
                    Some((forward, f, 0)),
                    Some((backward, f, 0)),
                ),
            }
        }
        _ if minimal_false => make(0, Atom, None, None),
        Translation::CallByName => match node {
            Node::Atom(_) => make(0, Atom, None, None),
            Node::True => make(0, Unit(Term::Top), None, None),
            Node::False => make(0, Unit(Term::Zero), None, None),
            Node::Not(a) => make(0, Lollipop, Some((a, 0, 1)), Some((falsity, 0, 0))),
            Node::And(a, b) => make(0, With, Some((a, 0, 0)), Some((b, 0, 0))),
            Node::Or(a, b) => make(0, Plus, Some((a, 0, 1)), Some((b, 0, 1))),
            Node::Implies(a, b) => make(0, Lollipop, Some((a, 0, 1)), Some((b, 0, 0))),
            Node::Iff(..) => make(0, With, Some((forward, 0, 0)), Some((backward, 0, 0))),
        },
        Translation::CallByValue => match node {
            Node::Atom(_) => make(1, Atom, None, None),
            Node::True => make(0, Unit(Term::One), None, None),
            Node::False => make(0, Unit(Term::Zero), None, None),
            Node::Not(a) => make(1, Lollipop, Some((a, 0, 0)), Some((falsity, 0, 0))),
            Node::And(a, b) => make(0, Tensor, Some((a, 0, 0)), Some((b, 0, 0))),
            Node::Or(a, b) => make(0, Plus, Some((a, 0, 0)), Some((b, 0, 0))),
            Node::Implies(a, b) => make(1, Lollipop, Some((a, 0, 0)), Some((b, 0, 0))),
            Node::Iff(..) => make(0, Tensor, Some((forward, 0, 0)), Some((backward, 0, 0))),
        },
        Translation::ZeroOne if function == T0 => match node {
            Node::Atom(_) => make(0, Atom, None, None),
            Node::True => make(0, Unit(Term::Top), None, None),
            Node::False => make(0, Unit(Term::Zero), None, None),
            Node::Not(a) => make(0, Lollipop, Some((a, T1, 1)), Some((falsity, T0, 1))),
            Node::And(a, b) => make(0, With, Some((a, T0, 1)), Some((b, T0, 1))),
            Node::Or(a, b) => make(0, Plus, Some((a, T0, 1)), Some((b, T0, 1))),
            Node::Implies(a, b) => make(0, Lollipop, Some((a, T1, 1)), Some((b, T0, 1))),
            Node::Iff(..) => make(0, With, Some((forward, T0, 1)), Some((backward, T0, 1))),
        },
        Translation::ZeroOne => match node {
            Node::Atom(_) => make(0, Atom, None, None),
            Node::True => make(0, Unit(Term::One), None, None),
            Node::False => make(0, Unit(Term::Zero), None, None),
            Node::Not(a) => make(1, Lollipop, Some((a, T0, 1)), Some((falsity, T1, 0))),
            Node::And(a, b) => make(1, With, Some((a, T1, 0)), Some((b, T1, 0))),
            Node::Or(a, b) => make(0, Plus, Some((a, T1, 1)), Some((b, T1, 1))),
            Node::Implies(a, b) => make(1, Lollipop, Some((a, T0, 1)), Some((b, T1, 0))),
            Node::Iff(..) => make(1, With, Some((forward, T1, 0)), Some((backward, T1, 0))),
        },
    }
}

/// What a root formula of the image is: the ordinary formula, its
/// translation function, the `!`s above its pattern's, and its side.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Root {
    /// The ordinary formula.
    pub(super) node: NodeId,
    /// Its translation function.
    pub(super) function: u8,
    /// The `!`s on top of its image, its pattern's included.
    pub(super) bangs: u8,
    /// The side of `⊢` it stands on.
    pub(super) side: Side,
}

/// The image of an ordinary sequent under a translation: the linear
/// sequent, one-sided as every [`Sequent`](crate::Sequent) is, the mode
/// it is decided in, and what the read-back needs to map a proof of it
/// back to the ordinary formulas.
#[derive(Clone, Debug)]
pub struct Image {
    /// The linear sequent.
    pub(super) sequent: crate::Sequent,
    /// Its mode: affine classical, or intuitionistic.
    pub(super) mode: Mode,
    /// The logic.
    pub(super) logic: Logic,
    /// The translation.
    pub(super) translation: Translation,
    /// The ordinary sequent decided: the one given, with false as the goal
    /// of an intuitionistic sequent that has none.
    pub(super) ordinary: Sequent,
    /// The extra nodes, which the arena of `ordinary` holds.
    pub(super) extra: Extra,
    /// Per root of the linear sequent, in its order, what it is.
    pub(super) roots: Vec<Root>,
}

impl Image {
    /// Returns the linear sequent.
    pub fn sequent(&self) -> &crate::Sequent {
        &self.sequent
    }

    /// Returns the mode the image is decided in: classical and affine for
    /// the affine translation, intuitionistic for the others.
    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// Returns the logic.
    pub fn logic(&self) -> Logic {
        self.logic
    }

    /// Returns the translation.
    pub fn translation(&self) -> Translation {
        self.translation
    }

    /// Returns the ordinary sequent the image is of: the one translated,
    /// with false right of `⊢` where an intuitionistic or minimal sequent
    /// has nothing there.
    pub fn ordinary(&self) -> &Sequent {
        &self.ordinary
    }
}

/// Returns the image of an ordinary sequent under a translation, for
/// deciding it in `logic`: the sequent is valid exactly when the image is
/// provable in the image's [`mode`](Image::mode).
///
/// # Errors
///
/// [`Error::Translation`] when the translation does not decide the logic
/// ([`Translation::decides`]), [`Error::Succedents`] for an intuitionistic
/// or minimal sequent with more than one formula right of `⊢`, and
/// [`Refusal::Occurrences`](crate::Refusal::Occurrences) for an image larger than an arena holds.
pub fn translate(
    sequent: &Sequent,
    logic: Logic,
    translation: Translation,
) -> Result<Image, Error> {
    if !translation.decides(logic) {
        return Err(Error::Translation { translation, logic });
    }
    let mut ordinary = sequent.clone();
    let classical = logic == Logic::Classical;
    if !classical && ordinary.right.len() > 1 {
        return Err(Error::Succedents {
            count: ordinary.right.len(),
        });
    }
    let extra = Extra::add(&mut ordinary.formulas)?;
    if !classical && ordinary.right.is_empty() {
        ordinary.right.push(extra.falsity);
    }
    let formulas = &ordinary.formulas;
    let functions: u8 = match translation {
        Translation::Affine | Translation::ZeroOne => 2,
        Translation::CallByName | Translation::CallByValue => 1,
    };

    // False's atom in minimal logic, which no atom of the sequent may be
    // (a TPTP problem can name an atom `false`).
    let mut falsity = FALSE.to_owned();
    while formulas.atom_names().contains(&falsity) {
        falsity.push('_');
    }

    // Every node's image under every function, right of `⊢` and dual,
    // in index order, so that the operands' images are there: an
    // implication of an equivalence is made before the equivalence's
    // image is, from operands before both.
    let mut builder = Builder::default();
    let mut images: Vec<Option<(TermId, TermId)>> = vec![None; formulas.len() * functions as usize];
    let slot = |id: NodeId, function: u8| id.index() * functions as usize + function as usize;
    // False first, which a negation refers to wherever it stands.
    let mut order = Vec::with_capacity(formulas.len() + 1);
    order.push(extra.falsity);
    for n in 0..formulas.len() {
        let id = NodeId(n as u32);
        if let Some(&(forward, backward)) = extra.iff.get(&id) {
            order.extend([forward, backward]);
        }
        order.push(id);
    }
    for id in order {
        for function in 0..functions {
            if images[slot(id, function)].is_some() {
                continue;
            }
            let pattern = pattern(formulas, &extra, translation, logic, id, function);
            let operand =
                |(k, f, bangs): Operand, b: &mut Builder| -> Result<(TermId, TermId), Error> {
                    let (mut positive, mut negative) =
                        images[slot(k, f)].expect("an operand precedes its formula");
                    for _ in 0..bangs {
                        positive = b.add(Term::Bang(positive))?;
                        negative = b.add(Term::Quest(negative))?;
                    }
                    Ok((positive, negative))
                };
            let a = pattern.operands[0]
                .map(|o| operand(o, &mut builder))
                .transpose()?;
            let b = pattern.operands[1]
                .map(|o| operand(o, &mut builder))
                .transpose()?;
            let mut image = match pattern.core {
                Core::Atom => {
                    let name = match formulas.node(id) {
                        Node::Atom(a) => formulas.atom_name(a),
                        _ => &falsity,
                    };
                    let atom = builder.atom(name);
                    let (var, dual) = (
                        builder.add(Term::Atom(atom))?,
                        builder.add(Term::DualAtom(atom))?,
                    );
                    // The classical translation reads an atom left of `⊢`
                    // as its negation.
                    if classical && function == LEFT {
                        (dual, var)
                    } else {
                        (var, dual)
                    }
                }
                Core::Unit(unit) => (builder.add(unit)?, builder.add(unit.dual())?),
                Core::Pass => a.expect("a negation has an operand"),
                core => {
                    let ((pa, na), (pb, nb)) = (a.expect("binary"), b.expect("binary"));
                    let (positive, negative) = match core {
                        Core::Tensor => (Term::Tensor(pa, pb), Term::Par(na, nb)),
                        Core::Par => (Term::Par(pa, pb), Term::Tensor(na, nb)),
                        Core::With => (Term::With(pa, pb), Term::Plus(na, nb)),
                        Core::Plus => (Term::Plus(pa, pb), Term::With(na, nb)),
                        _ => (Term::Par(na, pb), Term::Tensor(pa, nb)),
                    };
                    (builder.add(positive)?, builder.add(negative)?)
                }
            };
            for _ in 0..pattern.bangs {
                image = (
                    builder.add(Term::Bang(image.0))?,
                    builder.add(Term::Quest(image.1))?,
                );
            }
            images[slot(id, function)] = Some(image);
        }
    }

    // The roots: classically every formula on the right of the one-sided
    // sequent, a hypothesis by the function of the left; in ILL a
    // hypothesis by its dual.
    let mut roots = Vec::with_capacity(ordinary.left.len() + ordinary.right.len());
    let mut terms = Vec::with_capacity(roots.capacity());
    for (side, list) in [(Side::Left, &ordinary.left), (Side::Right, &ordinary.right)] {
        for &node in list {
            let (function, extra_bang) = match (translation, side) {
                (Translation::Affine, Side::Left) => (LEFT, 0),
                (Translation::Affine, Side::Right) => (RIGHT, 0),
                (Translation::CallByName, Side::Left) => (0, 1),
                (Translation::ZeroOne, Side::Left) => (T0, 1),
                (Translation::ZeroOne, Side::Right) => (T1, 0),
                _ => (0, 0),
            };
            let pattern = pattern(formulas, &extra, translation, logic, node, function);
            let (mut positive, mut negative) =
                images[slot(node, function)].expect("every image is made");
            for _ in 0..extra_bang {
                positive = builder.add(Term::Bang(positive))?;
                negative = builder.add(Term::Quest(negative))?;
            }
            let dual = side == Side::Left && !classical;
            terms.push(if dual { negative } else { positive });
            roots.push(Root {
                node,
                function,
                bangs: pattern.bangs + extra_bang,
                side,
            });
        }
    }

    // Classically every image stands right of `⊢`; in ILL the hypotheses'
    // duals stand left of it, and there are fewer of them than nodes.
    let antecedents = if classical {
        0
    } else {
        ordinary.left.len() as u32
    };
    let mut linear = crate::Sequent {
        terms: builder.terms,
        roots: terms,
        atoms: builder.atoms,
        antecedents: Some(antecedents),
    };
    linear.optimize()?;
    let mode = if classical {
        Mode::CLASSICAL.with_affine()
    } else {
        Mode::INTUITIONISTIC
    };
    Ok(Image {
        sequent: linear,
        mode,
        logic,
        translation,
        ordinary,
        extra,
        roots,
    })
}

/// A linear arena under construction, every term made once.
#[derive(Default)]
struct Builder {
    /// The terms, each after its subterms.
    terms: Vec<Term>,
    /// The index of every term made.
    ids: HashMap<Term, TermId>,
    /// The atom names.
    atoms: Vec<String>,
    /// The index of every atom name.
    names: HashMap<String, Atom>,
}

impl Builder {
    /// Returns the id of `term`, adding it unless it is there, or fails
    /// when the arena is full.
    fn add(&mut self, term: Term) -> Result<TermId, Error> {
        if let Some(&id) = self.ids.get(&term) {
            return Ok(id);
        }
        let most = crate::Forest::MOST;
        if self.terms.len() as u64 >= most {
            return Err(Error::Refused(crate::limits::Refusal::Occurrences {
                occurrences: most.saturating_add(1),
                limit: most,
            }));
        }
        let id = TermId::new(self.terms.len() as u32);
        self.terms.push(term);
        self.ids.insert(term, id);
        Ok(id)
    }

    /// Returns the atom called `name`.
    fn atom(&mut self, name: &str) -> Atom {
        if let Some(&atom) = self.names.get(name) {
            return atom;
        }
        let atom = Atom::new(self.atoms.len() as u32);
        self.atoms.push(name.to_owned());
        self.names.insert(name.to_owned(), atom);
        atom
    }
}
