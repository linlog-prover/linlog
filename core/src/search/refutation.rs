// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Why a sequent is unprovable: the refutation an exhaustive search or a
//! test of the sequent gives, and the disproof that holds it with the
//! sequent, the goal and the mode it means something in.

use crate::fragment::Mode;
use crate::occurrences::{Member, OccId};
use crate::sequents::{Atom, Sequent};
use std::fmt::{Display, Formatter, Result as FmtResult};

/// Why a sequent is unprovable. Most refutations rest on the search that
/// was exhaustive; where the literals or the connectives of the sequent
/// alone rule out a proof, the refutation says which, as the focused
/// engine checks them on every sequent it searches; and the Horn engine
/// may refute a Horn program by its state equation, whatever its search
/// found. A refutation means something only with its sequent, goal and
/// mode, which a [`Disproof`] holds; its [`Display`] writes atoms by
/// number (`#0`), the disproof's by name.
///
/// In JSON (feature `serialize`) an unprovable outcome's `refutation` is
/// `"exhausted"`, `{"unbalanced": {"atom", "least", "most"}}` (the atom
/// by its index into the sequent beside it), `{"equation": {"formulas",
/// "needed", "tensors", "pars", "ones", "bottoms", "mix"}}` or
/// `{"state_equation": {"atoms": [[atom, weight], …], "clauses":
/// [[occurrence, weight], …], "dropped": [occurrence, …]}}`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Refutation {
    /// Every way to prove the sequent was tried and failed.
    Exhausted,
    /// The literals of an atom cannot all meet their duals in axioms.
    Unbalanced(Unbalanced),
    /// The count equation of the multiplicatives fails.
    Equation(Equation),
    /// The goal is a Horn program whose state equation has no solution.
    StateEquation(StateEquation),
}

/// The literals of an atom that cannot all meet their duals in axioms:
/// whichever additive alternatives a proof takes, the positive literal
/// occurs between `least` and `most` times more than the negative one
/// (fewer, where these are negative), and never as often.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Unbalanced {
    /// The atom, whose name the sequent has.
    pub atom: Atom,
    /// The least excess of positive over negative literals.
    pub least: i32,
    /// The greatest excess.
    pub most: i32,
}

/// The count equation of the multiplicatives, which fails: a provable
/// sequent of MLL with units has exactly `tensors − pars − ones +
/// bottoms + 2` formulas, and at least that many with Mix, counted on the
/// one-sided sequent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Equation {
    /// The formulas of the sequent.
    pub formulas: u64,
    /// The formulas the equation asks for, below zero where the `⅋` and
    /// the `1` outnumber the rest.
    pub needed: i64,
    /// Its `⊗`.
    pub tensors: u64,
    /// Its `⅋`.
    pub pars: u64,
    /// Its `1`.
    pub ones: u64,
    /// Its `⊥`.
    pub bottoms: u64,
    /// Whether Mix was allowed.
    pub mix: bool,
}

/// Weights for a Horn program's Petri net, every place included, that
/// its state equation fails under (Farkas' lemma): weighting the tokens
/// of each atom and the ticket of each clause used once as given (zero
/// where not listed), no clause that can fire raises the weighted count,
/// while the goal asks it raised, so no sequence of firings reaches the
/// goal; in affine mode, where the weights are not negative, none covers
/// it either. A clause with an input that no marking reached from the
/// start holds can never fire, and is dropped from the inequalities.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct StateEquation {
    /// The atoms whose tokens weigh something, in the order of the
    /// sequent's atoms, with their weights.
    pub atoms: Vec<(Atom, i64)>,
    /// The clauses used once whose ticket weighs something, by
    /// occurrence, with their weights.
    pub clauses: Vec<(OccId, i64)>,
    /// The clauses that can never fire, by occurrence.
    pub dropped: Vec<OccId>,
}

impl Refutation {
    /// Writes the refutation as a phrase, each atom by its name in
    /// `names`, or by its number without them.
    fn write(&self, f: &mut Formatter<'_>, names: Option<&[String]>) -> FmtResult {
        let name = |atom: Atom| match names.and_then(|names| names.get(atom.index())) {
            Some(name) => name.clone(),
            None => format!("#{}", atom.get()),
        };
        match self {
            Refutation::Exhausted => f.write_str("the search was exhaustive"),
            Refutation::Unbalanced(Unbalanced { atom, least, most }) => {
                let name = name(*atom);
                let (more, fewer) = if *least > 0 {
                    (name.clone(), format!("~{name}"))
                } else {
                    (format!("~{name}"), name)
                };
                let (a, b) = (least.unsigned_abs(), most.unsigned_abs());
                let (a, b) = (a.min(b), a.max(b));
                let excess = if a == b {
                    format!("{a}")
                } else {
                    format!("{a} to {b}")
                };
                write!(
                    f,
                    "{more} occurs {excess} more {} than {fewer} in the one-sided sequent{}, so \
                     they cannot all meet in axioms",
                    if a == 1 && b == 1 { "time" } else { "times" },
                    if a == b {
                        ""
                    } else {
                        ", whichever additive alternatives a proof takes"
                    }
                )
            }
            Refutation::Equation(Equation {
                formulas,
                needed,
                tensors,
                pars,
                ones,
                bottoms,
                mix,
            }) => {
                // A count below zero with the minus sign of the expression.
                let needed = if *needed < 0 {
                    format!("−{}", needed.unsigned_abs())
                } else {
                    needed.to_string()
                };
                write!(
                    f,
                    "the count equation fails: a provable one-sided sequent of MLL has {}#⊗ − \
                     #⅋ − #1 + #⊥ + 2 formulas, here {tensors} − {pars} − {ones} + {bottoms} + 2 = \
                     {needed}, and this one has {formulas}",
                    if *mix { "at least " } else { "exactly " },
                )
            }
            Refutation::StateEquation(StateEquation { atoms, clauses, .. }) => {
                f.write_str(
                    "the state equation of the Petri net has no solution, so no firing of its \
                     clauses yields the goal's atoms: weighting",
                )?;
                let once = !clauses.is_empty();
                for (i, &(atom, weight)) in atoms.iter().enumerate() {
                    let sign = if weight < 0 { "−" } else { "" };
                    let between = match i {
                        0 => " each",
                        _ if i + 1 == atoms.len() && !once => " and",
                        _ => ",",
                    };
                    write!(
                        f,
                        "{between} {} by {sign}{}",
                        name(atom),
                        weight.unsigned_abs()
                    )?;
                }
                if once {
                    let between = if atoms.is_empty() { "" } else { " and" };
                    f.write_str(between)?;
                    f.write_str(" the clauses used once by weights of their own")?;
                }
                f.write_str(
                    ", no clause that can fire raises the weighted count of the atoms, and the goal \
                     asks it raised",
                )
            }
        }
    }
}

impl Display for Refutation {
    /// Writes the refutation as a phrase, such as `the search was
    /// exhaustive`, with every atom by its number, as in `#0`.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.write(f, None)
    }
}

/// A refutation with what it refutes: the sequent, the goal within it if
/// another than the sequent's roots, and the mode. A search's
/// [`Verdict::Unprovable`](super::Verdict::Unprovable) carries one, as a
/// [`Verdict::Proved`](super::Verdict::Proved) carries a proof.
///
/// # JSON
///
/// With the feature `serialize` an unprovable outcome writes the
/// disproof's keys: `sequent`, `mode`, `refutation` (in the form
/// [`Refutation`] gives) and `goal` (the members, only for a goal off the
/// roots).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Disproof {
    /// The sequent.
    sequent: Sequent,
    /// The goal refuted, `None` for the sequent's roots.
    goal: Option<Box<[Member]>>,
    /// The mode it is refuted in.
    mode: Mode,
    /// Why it is unprovable.
    refutation: Refutation,
}

impl Disproof {
    /// Returns the disproof of a sequent in a mode.
    pub fn new(sequent: Sequent, mode: Mode, refutation: Refutation) -> Self {
        Self {
            sequent,
            goal: None,
            mode,
            refutation,
        }
    }

    /// Returns the disproof as one of a goal within the sequent.
    #[must_use]
    pub(crate) fn of_goal(self, goal: Box<[Member]>) -> Self {
        Self {
            goal: Some(goal),
            ..self
        }
    }

    /// Returns the sequent.
    pub fn sequent(&self) -> &Sequent {
        &self.sequent
    }

    /// Returns the goal refuted, `None` for the sequent's roots.
    pub fn goal(&self) -> Option<&[Member]> {
        self.goal.as_deref()
    }

    /// Returns the mode it is refuted in.
    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// Returns why the goal is unprovable.
    pub fn refutation(&self) -> &Refutation {
        &self.refutation
    }
}

impl Display for Disproof {
    /// Writes the refutation as a phrase, with every atom by its name.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.refutation.write(f, Some(self.sequent.atom_names()))
    }
}
