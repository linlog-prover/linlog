// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use super::sequents::Sequent;
use crate::fragment::Mode;
use crate::limits::Limits;
use crate::occurrences::{Forest, Member};
use crate::proofs::{Branch, Named, Node, NodeId, Proof as Prf};
use crate::wire::{self, Readable};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// The serialized form of a proof node: the rule's tag with its occurrence
/// ids and premise node indices. The tags are part of the interchange
/// format, so renaming one breaks it.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(rename = "N")]
pub(super) enum Step {
    /// The axiom on two literals.
    #[serde(rename = "ax")]
    Ax(u32, u32),
    /// `⊗` on an occurrence, from the left and the right premise.
    #[serde(rename = "⊗")]
    Tensor(u32, u32, u32),
    /// `⅋` on an occurrence, from a premise.
    #[serde(rename = "⅋")]
    Par(u32, u32),
    /// `1` on an occurrence.
    #[serde(rename = "1")]
    One(u32),
    /// `⊥` on an occurrence, from a premise.
    #[serde(rename = "⊥")]
    Bot(u32, u32),
    /// `&` on an occurrence, from the left and the right premise.
    #[serde(rename = "&")]
    With(u32, u32, u32),
    /// `⊕₁` on an occurrence, from a premise.
    #[serde(rename = "⊕₁")]
    PlusLeft(u32, u32),
    /// `⊕₂` on an occurrence, from a premise.
    #[serde(rename = "⊕₂")]
    PlusRight(u32, u32),
    /// `⊤` on an occurrence.
    #[serde(rename = "⊤")]
    Top(u32),
    /// Promotion on an occurrence, from a premise.
    #[serde(rename = "!")]
    Bang(u32, u32),
    /// The `?` step on an occurrence, from a premise.
    #[serde(rename = "?")]
    Quest(u32, u32),
    /// A copy of an occurrence, from a premise.
    #[serde(rename = "copy")]
    Copy(u32, u32),
    /// Weakening of an occurrence, from a premise.
    #[serde(rename = "wk")]
    Weaken(u32, u32),
    /// Mix, from two premises.
    #[serde(rename = "mix")]
    Mix(u32, u32),
}

/// The serialized form of a proof: its level at the top of a document,
/// its sequent and its nodes, premises before conclusions, the root last,
/// the mode it is meant for and the goal it concludes, where recorded.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(super) struct Proof {
    /// The wire level, written only at the top of a document.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "wire::version"
    )]
    version: Option<u32>,
    /// The sequent proved.
    sequent: Sequent,
    /// The nodes.
    nodes: Vec<Step>,
    /// The mode the proof is meant for, where recorded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    mode: Option<Mode>,
    /// The goal the root concludes, absent for the sequent's roots.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    goal: Option<Vec<u32>>,
}

impl From<Node> for Step {
    /// Converts a node into its serialized form.
    fn from(n: Node) -> Self {
        use Node::*;
        use Step as S;
        let (o, n_) = (|o: Member| o.get(), |p: NodeId| p.get());
        match n {
            Ax(a, b) => S::Ax(o(a), o(b)),
            Tensor(x, l, r) => S::Tensor(o(x), n_(l), n_(r)),
            Par(x, p) => S::Par(o(x), n_(p)),
            One(x) => S::One(o(x)),
            Bot(x, p) => S::Bot(o(x), n_(p)),
            With(x, l, r) => S::With(o(x), n_(l), n_(r)),
            Plus(x, Branch::Left, p) => S::PlusLeft(o(x), n_(p)),
            Plus(x, Branch::Right, p) => S::PlusRight(o(x), n_(p)),
            Top(x) => S::Top(o(x)),
            Bang(x, p) => S::Bang(o(x), n_(p)),
            Quest(x, p) => S::Quest(o(x), n_(p)),
            Copy(x, p) => S::Copy(o(x), n_(p)),
            Weaken(x, p) => S::Weaken(o(x), n_(p)),
            Mix(l, r) => S::Mix(n_(l), n_(r)),
        }
    }
}

impl From<Step> for Node {
    /// Converts a serialized step back into a node.
    fn from(s: Step) -> Self {
        use Node as N;
        use Step::*;
        let (o, n) = (Member::new, NodeId::new);
        match s {
            Ax(a, b) => N::Ax(o(a), o(b)),
            Tensor(x, l, r) => N::Tensor(o(x), n(l), n(r)),
            Par(x, p) => N::Par(o(x), n(p)),
            One(x) => N::One(o(x)),
            Bot(x, p) => N::Bot(o(x), n(p)),
            With(x, l, r) => N::With(o(x), n(l), n(r)),
            PlusLeft(x, p) => N::Plus(o(x), Branch::Left, n(p)),
            PlusRight(x, p) => N::Plus(o(x), Branch::Right, n(p)),
            Top(x) => N::Top(o(x)),
            Bang(x, p) => N::Bang(o(x), n(p)),
            Quest(x, p) => N::Quest(o(x), n(p)),
            Copy(x, p) => N::Copy(o(x), n(p)),
            Weaken(x, p) => N::Weaken(o(x), n(p)),
            Mix(l, r) => N::Mix(n(l), n(r)),
        }
    }
}

impl Proof {
    /// The proof's keys as an outcome holds them, the outcome's `version`
    /// and `mode` its own: the sequent, the nodes and the goal.
    pub(super) fn keys(p: &Prf) -> Proof {
        Proof {
            version: None,
            sequent: Sequent::from(p.sequent()),
            nodes: p.nodes().iter().copied().map(Step::from).collect(),
            mode: None,
            goal: p.goal().map(|goal| goal.iter().map(|m| m.get()).collect()),
        }
    }

    /// The proof as a document: its level, its keys and its mode.
    fn document(p: &Prf) -> Proof {
        Proof {
            version: wire::level(),
            mode: p.mode(),
            ..Proof::keys(p)
        }
    }

    /// Rebuilds the proof the form holds, its forest within `limits`; the
    /// bounds and the order of its nodes are checked, not the proof.
    fn within(self, limits: &Limits) -> Result<Prf, crate::Error> {
        let sequent = crate::Sequent::try_from(self.sequent)?;
        let forest = Forest::from_owned(sequent, limits)?;
        let nodes: Vec<Node> = self.nodes.into_iter().map(Node::from).collect();
        // The root is the last node, whose index must be a node id.
        let Ok(root) = u32::try_from(nodes.len().saturating_sub(1)) else {
            return Err(crate::Error::Refused(crate::limits::Refusal::Index {
                what: crate::limits::Space::Node,
                count: nodes.len() as u64,
                most: u64::from(u32::MAX),
            }));
        };
        let root = NodeId::new(root);
        let proof = match self.goal {
            None => Prf::new(forest, nodes, root)?,
            Some(goal) => {
                let goal: Vec<Member> = goal.into_iter().map(Member::new).collect();
                Prf::new_of_goal(forest, &goal, nodes, root)?
            }
        };
        Ok(match self.mode {
            Some(mode) => proof.with_mode(mode),
            None => proof,
        })
    }
}

impl serde::Serialize for Prf {
    /// Serializes the proof as a document.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        Proof::document(self).serialize(serializer)
    }
}

impl<'a> serde::Deserialize<'a> for Prf {
    /// Deserializes a proof within the default limits, as
    /// [`wire::upgrade`] does.
    fn deserialize<D: Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        Self::read(deserializer, &Limits::default())
    }
}

impl crate::sealed::Readable for Prf {}

impl Readable for Prf {
    const FORM: &'static str = "proof";

    /// Reads a proof: rebuilds its forest within the limits and checks the
    /// bounds and the order of its nodes, not the proof.
    fn read<'de, D: Deserializer<'de>>(deserializer: D, limits: &Limits) -> Result<Self, D::Error> {
        Proof::deserialize(deserializer)?
            .within(limits)
            .map_err(wire::fail)
    }
}

impl serde::Serialize for Named {
    /// Serializes the rule as its name.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.name())
    }
}

impl<'a> serde::Deserialize<'a> for Named {
    /// Deserializes a rule from its name, as `Named::from_str` reads it.
    fn deserialize<D: Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        let name = <std::borrow::Cow<'a, str>>::deserialize(deserializer)?;
        name.parse().map_err(serde::de::Error::custom)
    }
}
