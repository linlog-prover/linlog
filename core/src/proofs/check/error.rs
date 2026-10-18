// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! What a check reports: the derived sequents of a failing node, the
//! fault, or the refusal that ended it without a verdict.

use crate::errors::describe::{Budget, Limit, cut, occurrence};
use crate::errors::{Described, Owner, Subject};
use crate::limits::Refusal;
use crate::occurrences::{Forest, Member, ShapeError};
use crate::proofs::{Node, NodeId};
use std::fmt::{Display, Formatter, Result as FmtResult};

/// A dyadic sequent as the checker derived it, by members: the
/// unrestricted zone `Θ`, the linear zone `Γ` with repeats, and whether a
/// `⊤` above absorbs any further linear context.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dyadic {
    /// The unrestricted zone, ascending.
    pub theta: Vec<Member>,
    /// The linear zone, ascending, with repeats.
    pub gamma: Vec<Member>,
    /// Whether the linear zone may hold anything more.
    pub any: bool,
}

impl Display for Dyadic {
    /// Writes `⊢ Θ ; Γ` with the zones as ids, omitting `Θ ;` when it is
    /// empty, and `…` after `Γ` when anything more is allowed.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.write(f, None, None)
    }
}

impl Dyadic {
    /// Writes the sequent as [`Display`] does, with formulas instead of ids
    /// when a forest is given.
    fn write(&self, f: &mut Formatter<'_>, forest: Option<&Forest>, limit: Limit) -> FmtResult {
        let list = |f: &mut Formatter<'_>, ids: &[Member]| {
            let mut budget = Budget::new(f, limit);
            for (i, &o) in ids.iter().enumerate() {
                let written =
                    std::fmt::Write::write_str(&mut budget, if i == 0 { " " } else { ", " })
                        .and_then(|()| occurrence(&mut budget, forest, o.occ()));
                if written.is_err() && budget.cut {
                    return write!(budget.f, "… ({} formulas)", ids.len());
                }
                written?;
            }
            Ok(())
        };
        f.write_str("⊢")?;
        if !self.theta.is_empty() {
            list(f, &self.theta)?;
            f.write_str(" ;")?;
        }
        list(f, &self.gamma)?;
        if self.any {
            f.write_str(if self.gamma.is_empty() {
                " …"
            } else {
                ", …"
            })?;
        }
        Ok(())
    }
}

/// Why a check of a proof ended without accepting it: the term is not a
/// proof of its conclusion, a verdict, or the check was given up without
/// one. A caller must never report a refusal as an invalid proof, which
/// the two variants keep apart.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CheckError {
    /// The term is not a proof of its conclusion in the mode.
    Invalid(Box<Invalid>),
    /// The check was given up without a verdict on the term: a bound or the
    /// caller's stop.
    Refused(Refused),
}

/// What makes a term no proof: the node at fault, what the checker had
/// derived for its premises, and what the rule required.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Invalid {
    /// The node at fault; the root when the proof concludes the wrong
    /// sequent or the sequent has no intuitionistic reading.
    pub node: NodeId,
    /// The node's rule instance.
    pub rule: Node,
    /// The sequents derived for the node's premises, in the node's order.
    pub premises: Vec<Dyadic>,
    /// What the rule required and did not get.
    pub fault: Fault,
}

/// Where a check was given up, and why.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Refused {
    /// The node the check had come to.
    pub node: NodeId,
    /// The bound that refused it, or the stop.
    pub refusal: Refusal,
}

/// What a rule required and did not get.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Fault {
    /// The mode forbids the rule: weakening of a formula that is not a `?`
    /// outside affine mode, Mix without Mix or in intuitionistic mode, and
    /// in an ordered mode every rule but the axiom, `⊗` and `⅋`.
    Forbidden,
    /// The mode is intuitionistic and the sequent has no intuitionistic
    /// reading, so no proof of it is checked.
    Shape(ShapeError),
    /// The mode is intuitionistic and a sequent of the rule has this many
    /// formulas on the right of `⊢` instead of one: the premise's linear
    /// zone after the rule, or the context a `⊤` would absorb, or a
    /// weakening of the goal.
    #[non_exhaustive]
    Succedents {
        /// How many formulas stand right of `⊢`.
        count: usize,
    },
    /// The member is not of the kind the rule acts on.
    #[non_exhaustive]
    Kind {
        /// The member the rule names.
        member: Member,
    },
    /// The two literals of an axiom are not an atom and its negation.
    NotDual,
    /// A premise, by its index in the node, lacks the member the rule
    /// consumes from it.
    #[non_exhaustive]
    Missing {
        /// Which premise.
        premise: usize,
        /// What it lacks.
        member: Member,
    },
    /// The premise of a promotion has a linear zone besides the promoted
    /// subformula.
    NotEmpty,
    /// The premises of `&` do not share their linear zone.
    Differ,
    /// In an ordered mode, the contexts of a `⊗`'s premises are not the
    /// two arcs of its conclusion's cyclic order on either side of the
    /// `⊗`: going round from it, the right premise's context must come
    /// first and the left one's after it, and joining them otherwise
    /// needs an exchange.
    Order,
    /// A copy of a member that is not the subformula of a `?`.
    #[non_exhaustive]
    NotUnderQuest {
        /// The member copied.
        member: Member,
    },
    /// The node's linear zone holds more formulas than the rest of the
    /// proof can consume: every later rule takes two at most, and the root
    /// concludes the sequent. Only a term no engine builds gets here, and
    /// over unbounded counts it would fail at the root instead.
    Surplus,
    /// The root derives this sequent, which is not the proof's conclusion:
    /// its linear zone differs from the conclusion's formulas, or its
    /// unrestricted zone holds a copied member that no `?` rule below
    /// moved there.
    #[non_exhaustive]
    Conclusion {
        /// What the root derives.
        derived: Dyadic,
    },
}

impl Invalid {
    /// Returns the fault at `node`, with the premises derived for it.
    pub(crate) fn new(node: NodeId, rule: Node, premises: Vec<Dyadic>, fault: Fault) -> Self {
        Self {
            node,
            rule,
            premises,
            fault,
        }
    }
}

impl CheckError {
    /// Returns the node at fault, or the node the check had come to when
    /// it was refused.
    pub fn node(&self) -> NodeId {
        match self {
            Self::Invalid(invalid) => invalid.node,
            Self::Refused(refused) => refused.node,
        }
    }

    /// Returns the fault, or `None` for a refusal.
    pub fn fault(&self) -> Option<&Fault> {
        match self {
            Self::Invalid(invalid) => Some(&invalid.fault),
            Self::Refused(_) => None,
        }
    }

    /// Returns the error of a fault at `node`.
    pub(crate) fn invalid(node: NodeId, rule: Node, premises: Vec<Dyadic>, fault: Fault) -> Self {
        Self::Invalid(Box::new(Invalid::new(node, rule, premises, fault)))
    }
}

impl Display for CheckError {
    /// Writes the node, its premises and the problem, as in `node 2 (⊗ on 1
    /// from 0, 1) with premises ⊢ 0, 3 and ⊢ 3, 4: premise 0 lacks
    /// occurrence 2`.
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        self.write(f, None, None)
    }
}

impl CheckError {
    /// Returns the error for display with formulas instead of occurrence
    /// ids, read from `owner`, the proof that failed or its forest; the
    /// nodes keep their ids. For instance `node 2 (⊗ on A ⊗ ~B from 0, 1)
    /// with premises ⊢ ~A, ~B and ⊢ ~B, B: premise 0 lacks A`.
    pub fn describe<'a>(&'a self, owner: &'a impl Owner) -> Described<'a> {
        Described::new(Subject::Check(self), owner)
    }

    /// Writes the error as [`Display`] does, with formulas instead of ids
    /// when a forest is given.
    pub(crate) fn write(
        &self,
        f: &mut Formatter<'_>,
        forest: Option<&Forest>,
        limit: Limit,
    ) -> FmtResult {
        let invalid = match self {
            Self::Invalid(invalid) => invalid,
            Self::Refused(refused) => {
                return write!(
                    f,
                    "the check was given up at node {}, without a verdict on the proof: {}",
                    refused.node.get(),
                    refused.refusal
                );
            }
        };
        invalid.write(f, forest, limit)
    }
}

impl Invalid {
    /// Writes the fault as [`CheckError`]'s `Display` does, with formulas
    /// instead of ids when a forest is given.
    fn write(&self, f: &mut Formatter<'_>, forest: Option<&Forest>, limit: Limit) -> FmtResult {
        write!(f, "node {} ({}", self.node.get(), self.rule.name())?;
        for (i, o) in self.rule.members().enumerate() {
            f.write_str(if i == 0 { " on " } else { ", " })?;
            cut(f, forest, o.occ(), limit)?;
        }
        for (i, p) in self.rule.premises().enumerate() {
            write!(f, "{}{}", if i == 0 { " from " } else { ", " }, p.get())?;
        }
        f.write_str(")")?;
        for (i, p) in self.premises.iter().enumerate() {
            f.write_str(if i == 0 { " with premises " } else { " and " })?;
            p.write(f, forest, limit)?;
        }
        f.write_str(": ")?;
        use Fault::*;
        match &self.fault {
            Forbidden => f.write_str("the mode forbids the rule"),
            Shape(e) => {
                f.write_str("not an intuitionistic sequent: ")?;
                e.write(f, forest, limit)
            }
            Succedents { count } => write!(
                f,
                "a sequent of the rule has {count} formulas on the right of ⊢ instead of one"
            ),
            Kind { member: o } => {
                if forest.is_none() {
                    f.write_str("occurrence ")?;
                }
                cut(f, forest, (*o).occ(), limit)?;
                f.write_str(" is not what the rule acts on")
            }
            NotDual => f.write_str("the literals are not an atom and its negation"),
            Missing { premise, member: o } => {
                write!(f, "premise {premise} lacks ")?;
                if forest.is_none() {
                    f.write_str("occurrence ")?;
                }
                cut(f, forest, (*o).occ(), limit)
            }
            NotEmpty => f.write_str("the linear zone is not empty"),
            Differ => f.write_str("the premises differ"),
            Order => f.write_str("the premises' contexts are not the arcs on either side of the ⊗"),
            NotUnderQuest { member: o } => {
                if forest.is_none() {
                    f.write_str("occurrence ")?;
                }
                cut(f, forest, (*o).occ(), limit)?;
                f.write_str(" is not under a ?")
            }
            Surplus => f.write_str(
                "the linear zone holds more formulas than the rest of the proof can consume",
            ),
            Conclusion { derived: d } => {
                f.write_str("the proof concludes ")?;
                d.write(f, forest, limit)?;
                f.write_str(", not the sequent")
            }
        }
    }
}

impl std::error::Error for CheckError {}
