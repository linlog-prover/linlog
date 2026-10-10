// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use super::sequents::Sequent;
use crate::fragment::Mode;
use crate::limits::Limits;
use crate::occurrences::{Forest, Member};
use crate::proofs::interactive::Interactive as State;
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
}

impl From<&State> for Interactive {
    /// Converts a state into its serialized form.
    fn from(state: &State) -> Self {
        Self {
            version: wire::level(),
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
                })
                .collect(),
            history: state.history().iter().map(|id| id.get()).collect(),
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
            })
            .collect();
        let history = proxy.history.into_iter().map(InfId::new).collect();
        State::from_parts(forest, proxy.mode, inferences, history)
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
