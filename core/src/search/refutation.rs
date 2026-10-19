// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Why a sequent is unprovable: the refutation an exhaustive search or a
//! test of the sequent gives, and the disproof that holds it with the
//! sequent, the goal and the mode it means something in.

use crate::fragment::Mode;
use crate::occurrences::{Member, OccId, Sign};
use crate::sequents::{Atom, Sequent};
use std::fmt::{Display, Formatter, Result as FmtResult};

/// Why a sequent is unprovable. Most refutations rest on the search that
/// was exhaustive; where the literals or the connectives of the sequent
/// alone rule out a proof, the refutation says which, as the focused
/// engine checks them on every sequent it searches; and the Horn engine
/// may refute a Horn program by its state equation, whatever its search
/// found. A refutation means something only with its sequent, goal and
/// mode, which a [`Disproof`] holds; its [`Display`] writes atoms by
/// number (`#0`), the disproof's by name. Of a goal off the roots, the
/// "one-sided sequent" a count speaks of is the goal: its members, which
/// may lie below an exponential of the sequent, are counted as written.
///
/// In JSON (feature `serialize`) a refutation, as an unprovable outcome's
/// and a disproof's `refutation`, is tagged by `kind`: `{"kind": "exhausted"}`, `{"kind": "unbalanced",
/// "atom", "least", "most"}` (the atom by its index into the sequent
/// beside it), `{"kind": "one_sided", "atom", "least", "most", "copied"}`
/// (`copied` the literal `"atom"` or `"dual"`), `{"kind": "equation", "formulas", "needed", "tensors",
/// "pars", "ones", "bottoms", "mix"}` or `{"kind": "state_equation",
/// "atoms": [[atom, weight], …], "clauses": [[occurrence, weight], …],
/// "dropped": [occurrence, …]}` or `{"kind": "classical", "assignment":
/// [true, false, …]}` (a value per atom of the sequent, by index) or
/// `{"kind": "saturated", "positive": ["atom", "dual", …], "sequents":
/// [{"linear": [occurrence, …], "unrestricted": [occurrence, …],
/// "absorbs": false}, …]}`, and reads back so. A reader refuses a
/// kind it does not know: read as another, it would claim a certificate
/// it does not hold.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Refutation {
    /// Every way to prove the sequent was tried and failed.
    Exhausted,
    /// The literals of an atom cannot all meet their duals in axioms.
    Unbalanced(Unbalanced),
    /// Nor can they where copies may add literals of the atom, but only
    /// of one sign.
    OneSided(OneSided),
    /// The count equation of the multiplicatives fails.
    Equation(Equation),
    /// The goal is a Horn program whose state equation has no solution.
    StateEquation(StateEquation),
    /// The classical reading of the goal is false under an assignment.
    Classical(Classical),
    /// A forward search saturated: its sequents are closed under the
    /// rules, and the goal needs one they do not give.
    Saturated(Saturated),
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

/// The literals of an atom that cannot all meet their duals in axioms,
/// although some lie below a `?` or `!`, where copies may add any number
/// of them: every literal of the atom below one is `copied`, so copies can
/// only add that literal, and outside the exponentials it already occurs
/// between `least` and `most` times more than the other one (counted as
/// [`Unbalanced`] counts, so `least` above zero for the atom and `most`
/// below zero for its negation).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct OneSided {
    /// The atom, whose name the sequent has.
    pub atom: Atom,
    /// The least excess of positive over negative literals outside the
    /// exponentials.
    pub least: i32,
    /// The greatest excess.
    pub most: i32,
    /// The literal of the atom that every one below an exponential is.
    pub copied: Sign,
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

/// An assignment of truth values to the atoms under which the classical
/// reading of the goal is false: `!` and `?` erased, `⊗` and `&` read as
/// "and", `⅋` and `⊕` as "or", `1` and `⊤` as true, `⊥` and `0` as false,
/// `~a` as "not a", and the goal as the disjunction of its formulas. In
/// every mode a provable sequent's reading is true under every assignment
/// (each rule, Mix and weakening included, derives a tautology from
/// tautologies, and an intuitionistic sequent's one-sided form reads as
/// its implication), so the assignment proves the goal unprovable.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Classical {
    /// The value of each atom of the sequent, by index; an atom the goal
    /// does not contain has one too.
    pub assignment: Vec<bool>,
}

/// The sequents a forward search kept when it saturated, which are closed
/// under the rules of the focused calculus and leave a stable sequent of
/// the goal's asynchronous phase underived: no proof of the goal exists.
/// Every rule (a focus on a positive formula, or a copy of a `?`
/// formula's subformula, with the sides of its `⊕`, the partners of its
/// axioms and the asynchronous phases of what it releases) applied to any
/// of the sequents derives one that a sequent of the set stands for, or
/// one no proof holds (a member twice, or two a `⊗`, `&` or `⊕`
/// separates, outside the subformulas of `?`), and so does Mix where the
/// mode has it; so by induction every stable sequent of a focused proof
/// is one a sequent of the set stands for, and some stable sequent of the
/// goal's phase is one none of them does. The rules are those of the
/// polarity `positive` names, under which focusing is as complete as
/// under any other. The certificate is as large as the set, which the
/// check reads whole and applies every rule to.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Saturated {
    /// The positive literal of each atom of the sequent, by index:
    /// [`Sign::Atom`] where the atom is positive, [`Sign::Dual`] where its
    /// negation is.
    pub positive: Vec<Sign>,
    /// The sequents.
    pub sequents: Vec<Stable>,
}

/// A stable sequent of a saturated set: positive formulas and negative
/// literals in its linear zone, subformulas of `?` in its unrestricted
/// one, as occurrences of the sequent's forest. It stands for the
/// sequents with exactly this linear zone and an unrestricted zone that
/// holds this one, and where it `absorbs` (a `⊤` above) or in affine mode
/// for those with a larger linear zone too, a member in output position
/// absorbed only where the zone has none and weakened in never. The
/// subformulas of the `?` the goal reaches through `⅋` alone, which every
/// sequent of a proof holds, may be left out of its unrestricted zone.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Stable {
    /// The linear zone, sorted, with repeats.
    pub linear: Vec<OccId>,
    /// The unrestricted zone, sorted, without repeats.
    pub unrestricted: Vec<OccId>,
    /// Whether a `⊤` above absorbs any further linear member.
    pub absorbs: bool,
}

impl Stable {
    /// Returns the stable sequent of the zones given.
    pub fn new(linear: Vec<OccId>, unrestricted: Vec<OccId>, absorbs: bool) -> Self {
        Self {
            linear,
            unrestricted,
            absorbs,
        }
    }
}

impl Saturated {
    /// Returns the saturated set of `sequents` under the polarity
    /// `positive`.
    pub fn new(positive: Vec<Sign>, sequents: Vec<Stable>) -> Self {
        Self { positive, sequents }
    }
}

impl Refutation {
    /// The words of the refutations, as [`Refutation::name`] gives them and
    /// the JSON form's `kind` writes them.
    pub const NAMES: &'static [&'static str] = &[
        "exhausted",
        "unbalanced",
        "one_sided",
        "equation",
        "state_equation",
        "classical",
        "saturated",
    ];

    /// Returns the refutation's word, its `kind` in the JSON form.
    pub const fn name(&self) -> &'static str {
        match self {
            Refutation::Exhausted => "exhausted",
            Refutation::Unbalanced(_) => "unbalanced",
            Refutation::OneSided(_) => "one_sided",
            Refutation::Equation(_) => "equation",
            Refutation::StateEquation(_) => "state_equation",
            Refutation::Classical(_) => "classical",
            Refutation::Saturated(_) => "saturated",
        }
    }

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
                write_unbalanced(f, &name(*atom), *least, *most, None)
            }
            Refutation::OneSided(OneSided {
                atom,
                least,
                most,
                copied,
            }) => write_unbalanced(f, &name(*atom), *least, *most, Some(*copied)),
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
            Refutation::Classical(Classical { assignment }) => {
                f.write_str("the classical reading is false")?;
                let truths: Vec<String> = (0..assignment.len())
                    .filter(|&a| assignment[a])
                    .map(|a| name(Atom::new(a as u32)))
                    .collect();
                match truths.len() {
                    _ if assignment.is_empty() => {}
                    0 => f.write_str(" where every atom is false")?,
                    n if n == assignment.len() => f.write_str(" where every atom is true")?,
                    n => write!(
                        f,
                        " where {} {} true and every other atom false",
                        list(&truths),
                        if n == 1 { "is" } else { "are" }
                    )?,
                }
                f.write_str(
                    " (! and ? erased, ⊗ and & as and, ⅋ and ⊕ as or, 1 and ⊤ true, ⊥ and 0 false), \
                     and every provable sequent reads as a tautology",
                )
            }
            Refutation::Saturated(Saturated { sequents, .. }) => write!(
                f,
                "the search saturated: {} {} closed under the rules, and the goal needs one {}",
                sequents.len(),
                if sequents.len() == 1 {
                    "sequent"
                } else {
                    "sequents"
                },
                if sequents.len() == 1 {
                    "it does not give"
                } else {
                    "they do not give"
                },
            ),
        }
    }
}

/// Returns names as an English list: `a`, `a and b`, `a, b and c`.
fn list(names: &[String]) -> String {
    match names {
        [] => String::new(),
        [one] => one.clone(),
        [init @ .., last] => format!("{} and {last}", init.join(", ")),
    }
}

/// Writes why the literals of an atom named `name` cannot all meet in
/// axioms: the one outnumbers the other between `least` and `most` times,
/// outside the exponentials where some literals, all `copied`, lie below
/// one.
fn write_unbalanced(
    f: &mut Formatter<'_>,
    name: &str,
    least: i32,
    most: i32,
    copied: Option<Sign>,
) -> FmtResult {
    let (more, fewer) = if least > 0 {
        (name.to_owned(), format!("~{name}"))
    } else {
        (format!("~{name}"), name.to_owned())
    };
    let (a, b) = (least.unsigned_abs(), most.unsigned_abs());
    let (a, b) = (a.min(b), a.max(b));
    let excess = if a == b {
        format!("{a}")
    } else {
        format!("{a} to {b}")
    };
    // The copied literal is the one `copied` names, which the engine's
    // refutations make the one that outnumbers; a form read from JSON says
    // what its fields say.
    let (place, copies) = match copied {
        Some(sign) => {
            let literal = match sign {
                Sign::Atom => name.to_owned(),
                Sign::Dual => format!("~{name}"),
            };
            (
                "outside the exponentials of the one-sided sequent",
                format!(
                    ", and every literal of the atom below one, which copies add, is {literal}"
                ),
            )
        }
        None => ("in the one-sided sequent", String::new()),
    };
    write!(
        f,
        "{more} occurs {excess} more {} than {fewer} {place}{}{copies}, so they cannot all \
         meet in axioms",
        if a == 1 && b == 1 { "time" } else { "times" },
        if a == b {
            ""
        } else {
            ", whichever additive alternatives a proof takes"
        }
    )
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
/// With the feature `serialize` a disproof is the document `{"version":
/// 1, "sequent": …, "mode": …, "refutation": …, "goal": […]}`: the
/// refutation in the form [`Refutation`] gives, the goal's members only
/// for a goal off the roots. An unprovable outcome writes the same keys,
/// so it reads back as its disproof. Reading counts the sequent against
/// the limits and checks every atom and occurrence the refutation and the
/// goal name against it (`Error::IndexOutOfBounds` otherwise), not the
/// refutation itself; a refutation kind the reader does not know is
/// refused (`Error::Json`, kind malformed).
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

/// Why [`Disproof::check`] does not accept a refutation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum RefutationFault {
    /// The refutation is no certificate: an exhausted search, a count with
    /// copies or a Horn program's state equation rests on the search, and
    /// the check verifies none of them.
    #[error("a refutation of kind `{kind}` rests on the search and has no check")]
    Uncertified {
        /// The refutation's word ([`Refutation::name`]).
        kind: &'static str,
    },
    /// The invariant does not hold where the goal is: weakening discards
    /// literals and connectives, a `⊤` absorbs them, copies add them, and
    /// the count equation holds in the multiplicative fragment alone.
    #[error("the {invariant} is no invariant here: {why}")]
    NotApplicable {
        /// The invariant: `balance of the atom` or `count equation`.
        invariant: &'static str,
        /// What defeats it.
        why: &'static str,
    },
    /// The refutation's numbers are not the goal's: an atom or an
    /// occurrence the sequent lacks, or counts that differ from its own.
    #[error("the refutation's numbers are not those of the sequent")]
    Mismatch,
    /// The numbers are the goal's, and they allow a proof: the atom's
    /// excess can be zero, or the count equation holds.
    #[error("the counts allow a proof")]
    Holds,
    /// The assignment has another number of values than the sequent has
    /// atoms.
    #[error("the assignment gives {values} values for {atoms} atoms")]
    Assignment {
        /// The atoms of the sequent.
        atoms: usize,
        /// The values given.
        values: usize,
    },
    /// The assignment makes the classical reading true.
    #[error("the classical reading is true under the assignment")]
    Satisfied,
    /// A sequent of a saturated set is none of the mode: under the
    /// intuitionistic reading it has other than one member in output
    /// position, or more than one where it absorbs.
    #[error("a sequent of the saturated set is no sequent of the mode")]
    Unshaped,
    /// A rule applied to sequents of a saturated set derives one that
    /// could stand in a proof and that none of them stands for.
    #[error("the saturated set is not closed under the rules")]
    NotClosed,
    /// The sequents of a saturated set stand for every stable sequent of
    /// the goal's asynchronous phase, so they do not rule a proof out.
    #[error("the saturated set gives every stable sequent the goal needs")]
    Filled,
}

impl RefutationFault {
    /// Returns the fault's word, as an error's details write it.
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Uncertified { .. } => "uncertified",
            Self::NotApplicable { .. } => "not_applicable",
            Self::Mismatch => "mismatch",
            Self::Holds => "holds",
            Self::Assignment { .. } => "assignment",
            Self::Satisfied => "satisfied",
            Self::Unshaped => "unshaped",
            Self::NotClosed => "not_closed",
            Self::Filled => "filled",
        }
    }
}

impl Disproof {
    /// Checks the refutation against the sequent, the goal and the mode,
    /// sharing no code with the search: an assignment by evaluating the
    /// goal's classical reading, which must be false; an unbalanced atom
    /// by its excess of positive over negative literals, whose interval
    /// over the additive alternatives (`⊗` and `⅋` adding their sides, `&`
    /// and `⊕` taking the hull of theirs) must be the one stated and
    /// exclude zero, outside affine mode and with no `⊤` and no `!` or `?`
    /// over the atom in the goal; the count equation by the goal's
    /// formulas, `⊗`, `⅋`, `1` and `⊥`, which must be the ones stated and
    /// fail it, outside affine mode and in the multiplicative fragment
    /// with units, with Mix exactly where the mode allows it; a saturated
    /// set by reading the rules of the focused calculus off the forest
    /// again, under the set's polarity, and applying each to every choice
    /// of premises among its sequents, each sequent derived one that a
    /// sequent of the set stands for or that no proof holds, Mix too
    /// where the mode has it, its sequents shaped as the mode's are, and
    /// a stable sequent of the goal's phase that none stands for. Those
    /// four are certificates: every derivation keeps the invariant the
    /// first three break, and the set holds a sequent for every stable
    /// one of every proof. Time and memory are linear in the goal's
    /// occurrences for the first three; the set's check applies every
    /// rule to every choice of its sequents, which takes as long as the
    /// search that saturated did at most for a set a search gave, but has
    /// no stop here: for a set read from elsewhere it can take time
    /// exponential in the `&` under `⅋` of the goal and of the degree of
    /// a rule's holes in the set's size (a set repeating one sequent), so
    /// a caller checks such a set within a time limit of its own.
    ///
    /// # Errors
    ///
    /// [`Error::Refutation`](crate::Error::Refutation) with the
    /// [`RefutationFault`]: of kind invalid for a refutation that does not
    /// hold, unsupported for an exhausted search and the refutations that
    /// rest on it ([`RefutationFault::Uncertified`]); and the errors of
    /// [`Forest::new`](crate::Forest::new) for a sequent past the default
    /// limits.
    pub fn check(&self) -> Result<(), crate::Error> {
        let fault = |fault| crate::Error::Refutation(fault);
        let forest = crate::occurrences::Forest::new(&self.sequent)?;
        let goal: Vec<OccId> = match &self.goal {
            None => forest.roots().to_vec(),
            Some(members) => members
                .iter()
                .map(|m| {
                    m.occurrence(&forest)
                        .ok_or(fault(RefutationFault::Mismatch))
                })
                .collect::<Result<_, _>>()?,
        };
        match &self.refutation {
            Refutation::Classical(classical) => check_classical(&forest, &goal, classical),
            Refutation::Unbalanced(unbalanced) => {
                check_unbalanced(&forest, &goal, self.mode, unbalanced)
            }
            Refutation::Equation(equation) => check_equation(&forest, &goal, self.mode, equation),
            Refutation::Saturated(saturated) => {
                match super::saturated::check(&forest, &goal, self.mode, saturated, &mut |_| false)
                {
                    Ok(()) => Ok(()),
                    Err(super::saturated::Failed::Fault(fault)) => Err(fault),
                    Err(super::saturated::Failed::Stopped) => {
                        unreachable!("a check without a stop runs to its end")
                    }
                }
            }
            refutation @ (Refutation::Exhausted
            | Refutation::OneSided(_)
            | Refutation::StateEquation(_)) => Err(RefutationFault::Uncertified {
                kind: refutation.name(),
            }),
        }
        .map_err(fault)
    }
}

/// Checks that `classical` falsifies the classical reading of the goal.
fn check_classical(
    forest: &crate::occurrences::Forest,
    goal: &[OccId],
    classical: &Classical,
) -> Result<(), RefutationFault> {
    use crate::sequents::Kind::*;
    let (atoms, values) = (
        forest.sequent().atom_names().len(),
        classical.assignment.len(),
    );
    if atoms != values {
        return Err(RefutationFault::Assignment { atoms, values });
    }
    let value = |o: OccId| {
        let atom = forest.atom(o).map(|a| classical.assignment[a.index()]);
        atom.unwrap_or(false)
    };
    let mut truth = vec![false; forest.len()];
    for &member in goal {
        // Preorder: a subtree's occurrences follow its root, so its
        // operands are evaluated first from the end.
        for o in forest.subtree(member).rev() {
            let side = |side: Option<OccId>| side.is_some_and(|s| truth[s.index()]);
            let (l, r) = (side(forest.left(o)), side(forest.right(o)));
            truth[o.index()] = match forest.kind(o) {
                Atom => value(o),
                DualAtom => !value(o),
                One | Top => true,
                Bot | Zero => false,
                Tensor | With => l && r,
                Par | Plus => l || r,
                Bang | Quest => l,
            };
        }
        if truth[member.index()] {
            return Err(RefutationFault::Satisfied);
        }
    }
    Ok(())
}

/// Checks that the atom of `unbalanced` has the stated excess interval in
/// the goal, which excludes zero, where the balance is an invariant.
fn check_unbalanced(
    forest: &crate::occurrences::Forest,
    goal: &[OccId],
    mode: Mode,
    unbalanced: &Unbalanced,
) -> Result<(), RefutationFault> {
    use crate::sequents::Kind::*;
    const INVARIANT: &str = "balance of the atom";
    if mode.affine {
        return Err(RefutationFault::NotApplicable {
            invariant: INVARIANT,
            why: "weakening discards literals",
        });
    }
    let atom = unbalanced.atom;
    if atom.index() >= forest.sequent().atom_names().len() {
        return Err(RefutationFault::Mismatch);
    }
    // Per occurrence the least and the greatest excess, `None` where a
    // `⊤` absorbs or copies of the atom's literals can add any number:
    // sums of ±1 over the goal's members, each a subtree of the forest's
    // fewer than 2³² occurrences: a goal past 2³¹ members would be needed
    // to leave an `i64`, which no memory holds.
    let mut span: Vec<Option<(i64, i64)>> = vec![None; forest.len()];
    let (mut least, mut most) = (0, 0);
    for &member in goal {
        for o in forest.subtree(member).rev() {
            let side = |side: Option<OccId>| side.and_then(|s| span[s.index()]);
            let (l, r) = (side(forest.left(o)), side(forest.right(o)));
            let literal = |e| {
                Some(if forest.atom(o) == Some(atom) {
                    (e, e)
                } else {
                    (0, 0)
                })
            };
            span[o.index()] = match forest.kind(o) {
                Atom => literal(1),
                DualAtom => literal(-1),
                One | Bot | Zero => Some((0, 0)),
                Top => None,
                Tensor | Par => l.zip(r).map(|((a, b), (c, d))| (a + c, b + d)),
                With | Plus => l.zip(r).map(|((a, b), (c, d))| (a.min(c), b.max(d))),
                Bang | Quest => l.filter(|&s| s == (0, 0)),
            };
        }
        let Some((lo, hi)) = span[member.index()] else {
            return Err(RefutationFault::NotApplicable {
                invariant: INVARIANT,
                why: "a ⊤ absorbs, or copies of the atom's literals add any number",
            });
        };
        (least, most) = (least + lo, most + hi);
    }
    if (least, most) != (i64::from(unbalanced.least), i64::from(unbalanced.most)) {
        Err(RefutationFault::Mismatch)
    } else if least <= 0 && 0 <= most {
        Err(RefutationFault::Holds)
    } else {
        Ok(())
    }
}

/// Checks that the goal has the formulas and connectives `equation`
/// states, which fail the count equation, where it is an invariant.
fn check_equation(
    forest: &crate::occurrences::Forest,
    goal: &[OccId],
    mode: Mode,
    equation: &Equation,
) -> Result<(), RefutationFault> {
    use crate::sequents::Kind::*;
    const INVARIANT: &str = "count equation";
    if mode.affine {
        return Err(RefutationFault::NotApplicable {
            invariant: INVARIANT,
            why: "weakening adds formulas",
        });
    }
    let (mut tensors, mut pars, mut ones, mut bottoms) = (0u64, 0u64, 0u64, 0u64);
    for &member in goal {
        for o in forest.subtree(member) {
            match forest.kind(o) {
                Tensor => tensors += 1,
                Par => pars += 1,
                One => ones += 1,
                Bot => bottoms += 1,
                Atom | DualAtom => {}
                Top | Zero | With | Plus | Bang | Quest => {
                    return Err(RefutationFault::NotApplicable {
                        invariant: INVARIANT,
                        why: "it holds in the multiplicative fragment with units alone",
                    });
                }
            }
        }
    }
    // Counts over the goal's members, each a subtree of fewer than 2³²
    // occurrences: far within an `i64` for any goal a memory holds.
    let needed = tensors as i64 - pars as i64 - ones as i64 + bottoms as i64 + 2;
    let formulas = goal.len() as u64;
    let stated = Equation {
        formulas,
        needed,
        tensors,
        pars,
        ones,
        bottoms,
        mix: mode.mix,
    };
    if *equation != stated {
        return Err(RefutationFault::Mismatch);
    }
    let holds = if mode.mix {
        formulas as i64 >= needed
    } else {
        formulas as i64 == needed
    };
    if holds {
        Err(RefutationFault::Holds)
    } else {
        Ok(())
    }
}

#[cfg(all(test, feature = "parse"))]
mod tests {
    use super::*;
    use crate::search::{Options, Verdict, prove};

    /// Returns the disproof the default search gives `input` in `mode`.
    fn refuted(input: &str, mode: Mode) -> Disproof {
        let sequent: Sequent = input.parse().unwrap();
        match prove(&sequent, mode, &Options::default()).unwrap().verdict {
            Verdict::Unprovable(disproof) => *disproof,
            other => panic!("{input}: {other:?}"),
        }
    }

    /// Returns the disproof of `input` in `mode` for `refutation`.
    fn claimed(input: &str, mode: Mode, refutation: Refutation) -> Disproof {
        Disproof::new(input.parse().unwrap(), mode, refutation)
    }

    /// Returns the fault the check finds, `None` where it accepts.
    fn fault(disproof: &Disproof) -> Option<RefutationFault> {
        match disproof.check() {
            Ok(()) => None,
            Err(crate::Error::Refutation(fault)) => Some(fault),
            Err(other) => panic!("{other:?}"),
        }
    }

    /// The check accepts the search's assignments, balances and equations,
    /// and rejects every one that does not hold where the claim is: a
    /// satisfying or short assignment, numbers not the goal's or that
    /// allow a proof, a mode or connectives that defeat the invariant, and
    /// the refutations that rest on the search.
    #[test]
    fn a_wrong_refutation_is_rejected() {
        let classical = Mode::CLASSICAL;
        let assignment = |values: &[bool]| {
            Refutation::Classical(Classical {
                assignment: values.to_vec(),
            })
        };
        let found = refuted("|- a & b, ~a", classical);
        assert_eq!(fault(&found), None);
        assert_eq!(fault(&refuted("a, a -o b |- b * a", classical)), None);
        let wrong = claimed("|- a & b, ~a", classical, assignment(&[true, true]));
        assert_eq!(fault(&wrong), Some(RefutationFault::Satisfied));
        let short = claimed("|- a & b, ~a", classical, assignment(&[true]));
        assert_eq!(
            fault(&short),
            Some(RefutationFault::Assignment {
                atoms: 2,
                values: 1
            })
        );
        // In every mode the reading holds, intuitionistic too.
        let ill = refuted("a + b |- a", Mode::INTUITIONISTIC);
        assert!(matches!(ill.refutation(), Refutation::Classical(_)));
        assert_eq!(fault(&ill), None);

        let unbalanced = |atom: u32, least: i32, most: i32| {
            Refutation::Unbalanced(Unbalanced {
                atom: Atom::new(atom),
                least,
                most,
            })
        };
        let found = refuted("|- (a * a) + (a * a * a), ~a", classical);
        assert_eq!(found.refutation(), &unbalanced(0, 1, 2));
        assert_eq!(fault(&found), None);
        let mix = refuted("a |- b", classical.with_mix());
        assert!(matches!(mix.refutation(), Refutation::Unbalanced(_)));
        assert_eq!(fault(&mix), None);
        for (input, mode, refutation, expected) in [
            (
                "|- a, ~a",
                classical,
                unbalanced(0, 0, 0),
                RefutationFault::Holds,
            ),
            (
                "|- a, ~a",
                classical,
                unbalanced(0, 1, 1),
                RefutationFault::Mismatch,
            ),
            (
                "|- a, a",
                classical,
                unbalanced(1, 2, 2),
                RefutationFault::Mismatch,
            ),
            (
                "|- a + ~a, a",
                classical,
                unbalanced(0, 1, 1),
                RefutationFault::Mismatch,
            ),
            (
                "|- a + ~a, a",
                classical,
                unbalanced(0, 0, 2),
                RefutationFault::Holds,
            ),
        ] {
            assert_eq!(
                fault(&claimed(input, mode, refutation)),
                Some(expected),
                "{input}"
            );
        }
        for (input, mode) in [
            ("|- a, a", classical.with_affine()),
            ("|- a, a, top", classical),
            ("|- ?a, a", classical),
        ] {
            assert!(
                matches!(
                    fault(&claimed(input, mode, unbalanced(0, 2, 2))),
                    Some(RefutationFault::NotApplicable { .. })
                ),
                "{input}"
            );
        }

        let found = refuted("|- a par b, ~a, ~b", classical);
        let Refutation::Equation(equation) = *found.refutation() else {
            panic!("{found:?}");
        };
        assert_eq!(fault(&found), None);
        let stated = |input: &str, mode: Mode, equation: Equation| {
            fault(&claimed(input, mode, Refutation::Equation(equation)))
        };
        let with_mix = Equation {
            mix: true,
            ..equation
        };
        assert_eq!(
            stated("|- a par b, ~a, ~b", classical, with_mix),
            Some(RefutationFault::Mismatch)
        );
        assert_eq!(
            stated("|- a par b, ~a, ~b", classical.with_mix(), with_mix),
            Some(RefutationFault::Holds)
        );
        let holds = Equation {
            formulas: 2,
            needed: 2,
            tensors: 0,
            pars: 0,
            ones: 0,
            bottoms: 0,
            mix: false,
        };
        assert_eq!(
            stated("|- a, ~a", classical, holds),
            Some(RefutationFault::Holds)
        );
        for (input, mode) in [
            ("|- a par b, ~a, ~b", classical.with_affine()),
            ("|- a & b, ~a, ~b", classical),
            ("|- ?(a par b), ~a, ~b", classical),
        ] {
            assert!(
                matches!(
                    stated(input, mode, equation),
                    Some(RefutationFault::NotApplicable { .. })
                ),
                "{input}"
            );
        }

        let exhausted = claimed("|- a & b, ~a", classical, Refutation::Exhausted);
        let error = exhausted.check().unwrap_err();
        assert_eq!(
            (error.code(), error.kind()),
            ("uncertified", crate::ErrorKind::Unsupported)
        );
        let error = wrong.check().unwrap_err();
        assert_eq!(
            (error.code(), error.kind()),
            ("invalid_refutation", crate::ErrorKind::Invalid)
        );
    }
}
