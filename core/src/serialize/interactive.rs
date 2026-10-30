// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use super::sequents::Sequent;
use crate::fragment::Mode;
use crate::limits::Limits;
use crate::occurrences::{Forest, Member, Side, Written};
use crate::proofs::interactive::Interactive as State;
use crate::proofs::interactive::{PartTerm, Parts};
use crate::proofs::{InfId, Inference, Named, Rule};
use crate::wire::{self, Readable};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// The serialized form of one inference: its sequent as occurrence ids,
/// and for a closed one the rule's name, the principal position and the
/// premises' indices; an open goal is its sequent alone.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Step {
    /// The sequent, as occurrence ids in ascending order.
    sequent: Vec<u32>,
    /// The rule, absent for an open goal.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    rule: Option<Named>,
    /// The position of the principal formula, absent for an axiom, Mix or
    /// an open goal.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    principal: Option<usize>,
    /// The premises, absent for a leaf.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    premises: Vec<u32>,
    /// For a cut, the occurrence of its cut formula.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    cut: Option<u32>,
}

/// The serialized form of a proof in progress: the sequent, the mode, the
/// inferences with the root first, and the goals the steps closed, in
/// order.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(super) struct Interactive {
    /// The wire level.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "wire::version"
    )]
    version: Option<u32>,
    /// The sequent being proved.
    sequent: Sequent,
    /// The mode.
    mode: Mode,
    /// The inferences.
    inferences: Vec<Step>,
    /// The steps, each the inference it closed.
    history: Vec<u32>,
    /// How the sequent was written, for a session that keeps it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    written: Option<WrittenForm>,
    /// For a sequent with quantifiers, the instances, eigenvariables and
    /// open witnesses of the steps.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    instances: Option<InstancesForm>,
}

/// The serialized form of a session's first-order part: the members past
/// the sequent's occurrences as `[occurrence, frame]`, the frames as
/// `[parent, term]` (`null` for an outermost binder's), the terms past
/// the sequent's own (`{"F": [symbol, [terms]]}`, an eigenvariable `{"E":
/// n}`, an open witness `{"M": k}`), the level each metavariable was made
/// at, how many eigenvariables were made, those that are constants, and
/// the bindings in force in the order made, each `[metavariable, term,
/// step]`.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct InstancesForm {
    /// The members past the forest.
    members: Vec<(u32, u32)>,
    /// The frames.
    frames: Vec<(Option<u32>, u32)>,
    /// The terms past the sequent's.
    terms: Vec<TermForm>,
    /// The level each metavariable was made at.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    metas: Vec<u32>,
    /// How many eigenvariables were made.
    eigens: u32,
    /// The eigenvariables that are constants.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    constants: Vec<u32>,
    /// The bindings in force, in the order made.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    bindings: Vec<(u32, u32, u32)>,
}

/// The serialized form of a term of a session's table.
#[derive(Clone, Debug, Serialize, Deserialize)]
enum TermForm {
    /// A symbol, by its index in the sequent's `symbols`, applied to
    /// terms by their ids, each before this one.
    #[serde(rename = "F")]
    App(u32, Vec<u32>),
    /// An eigenvariable by its number.
    #[serde(rename = "E")]
    Eigen(u32),
    /// An open witness, a metavariable, by its number.
    #[serde(rename = "M")]
    Meta(u32),
}

impl From<Parts> for InstancesForm {
    /// Converts the written form into its serialized one.
    fn from(parts: Parts) -> Self {
        Self {
            members: parts.members,
            frames: parts.frames,
            terms: parts
                .terms
                .into_iter()
                .map(|t| match t {
                    PartTerm::App(f, args) => TermForm::App(f, args),
                    PartTerm::Eigen(a) => TermForm::Eigen(a),
                    PartTerm::Meta(k) => TermForm::Meta(k),
                })
                .collect(),
            metas: parts.metas,
            eigens: parts.eigens,
            constants: parts.constants,
            bindings: parts.bindings,
        }
    }
}

impl From<InstancesForm> for Parts {
    /// Converts the serialized form into the written one.
    fn from(form: InstancesForm) -> Self {
        Self {
            members: form.members,
            frames: form.frames,
            terms: form
                .terms
                .into_iter()
                .map(|t| match t {
                    TermForm::App(f, args) => PartTerm::App(f, args),
                    TermForm::Eigen(a) => PartTerm::Eigen(a),
                    TermForm::Meta(k) => PartTerm::Meta(k),
                })
                .collect(),
            metas: form.metas,
            eigens: form.eigens,
            constants: form.constants,
            bindings: form.bindings,
        }
    }
}

/// The serialized form of how a sequent was written: the occurrences on
/// the input side and those written as an implication, each ascending.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct WrittenForm {
    /// The occurrences whose connective was written on the input side.
    inputs: Vec<u32>,
    /// The occurrences written as an implication.
    implications: Vec<u32>,
}

impl WrittenForm {
    /// Returns the form of a record.
    fn of(writing: &[Written]) -> Self {
        let ids = |keep: fn(Written) -> bool| {
            // A record has one entry per occurrence, of which a forest has
            // fewer than `u32::MAX`.
            (0..writing.len() as u32)
                .filter(|&o| keep(writing[o as usize]))
                .collect()
        };
        Self {
            inputs: ids(|w| w.side() == Side::Input),
            implications: ids(Written::is_implication),
        }
    }

    /// Returns the record over a forest of `len` occurrences, refusing an
    /// id past it and a list out of order.
    fn record(&self, len: usize) -> Result<Box<[Written]>, crate::Error> {
        let mut writing = vec![Written::new(Side::Output, false); len];
        for (list, implication) in [(&self.inputs, false), (&self.implications, true)] {
            if !list.is_sorted_by(|a, b| a < b) {
                return Err(crate::Error::InconsistentSession {
                    reason: "a list of the record is not ascending",
                });
            }
            for &o in list {
                let Some(written) = writing.get_mut(o as usize) else {
                    return Err(crate::Error::IndexOutOfBounds {
                        space: crate::limits::Space::Occurrence,
                        index: o as usize,
                        len,
                    });
                };
                *written = if implication {
                    Written::new(written.side(), true)
                } else {
                    Written::new(Side::Input, written.is_implication())
                };
            }
        }
        Ok(writing.into_boxed_slice())
    }
}

impl From<&State> for Interactive {
    /// Converts a state into its serialized form.
    fn from(state: &State) -> Self {
        Self {
            version: Some(
                wire::level_in(Some(state.mode())).max(wire::level_of_sequent(state.sequent())),
            ),
            sequent: Sequent::from(state.sequent()),
            mode: state.mode(),
            inferences: state
                .inferences()
                .iter()
                .map(|inference| Step {
                    sequent: inference.sequent.iter().map(|o| o.get()).collect(),
                    rule: (inference.rule.rule != Rule::Open).then_some(inference.rule),
                    principal: inference.principal,
                    premises: inference.premises.iter().map(|p| p.get()).collect(),
                    cut: inference.cut.map(Member::get),
                })
                .collect(),
            history: state.history().iter().map(|id| id.get()).collect(),
            written: state.writing().map(WrittenForm::of),
            instances: state.parts().map(InstancesForm::from),
        }
    }
}

impl Interactive {
    /// Rebuilds the forest within `limits` and the state over it, checking
    /// that the inferences fit together and that every closed one is what
    /// its rule yields.
    fn within(self, limits: &Limits) -> Result<State, crate::Error> {
        let proxy = self;
        let sequent = crate::Sequent::try_from(proxy.sequent)?;
        let forest = Forest::from_owned(sequent, limits)?;
        let inferences = proxy
            .inferences
            .into_iter()
            .map(|step| Inference {
                sequent: step.sequent.into_iter().map(Member::new).collect(),
                rule: step.rule.unwrap_or_else(|| Rule::Open.into()),
                principal: step.principal,
                premises: step.premises.into_iter().map(InfId::new).collect(),
                times: 1,
                cut: step.cut.map(Member::new),
                node: None,
            })
            .collect();
        let history = proxy.history.into_iter().map(InfId::new).collect();
        let len = forest.len();
        let parts = proxy.instances.map(Parts::from);
        let mut state = State::from_parts(forest, proxy.mode, inferences, history, parts)?;
        if let Some(written) = proxy.written {
            state.keep_writing(written.record(len)?)?;
        }
        Ok(state)
    }
}

impl Serialize for State {
    /// Serializes the state as its sequent, mode, inferences and steps.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        Interactive::from(self).serialize(serializer)
    }
}

impl<'a> Deserialize<'a> for State {
    /// Deserializes a state within the default limits, as
    /// [`wire::upgrade`] does.
    fn deserialize<D: Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        Self::read(deserializer, &Limits::default())
    }
}

impl crate::sealed::Readable for State {}

impl Readable for State {
    const FORM: &'static str = "session";

    /// Reads a state and checks that it is consistent; see
    /// [`Interactive`](State).
    fn read<'de, D: Deserializer<'de>>(deserializer: D, limits: &Limits) -> Result<Self, D::Error> {
        Interactive::deserialize(deserializer)?
            .within(limits)
            .map_err(wire::fail)
    }
}
