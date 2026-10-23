// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use super::sequents::Sequent;
use crate::fragment::Order;
use crate::limits::Limits;
use crate::nets::{Criterion, MellStructure, Parts, Placed, ProofStructure, VertexId};
use crate::occurrences::{Forest, OccId};
use crate::wire::{self, Readable};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// The serialized form of a proof structure: its level, its sequent, the
/// criterion's fields (whether Mix is allowed), and its links as pairs of
/// vertex ids in the order they were made.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Net {
    /// The wire level.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "wire::version"
    )]
    version: Option<u32>,
    /// The sequent.
    sequent: Sequent,
    /// Whether Mix is allowed.
    mix: bool,
    /// The order the criterion keeps, written where it is not the
    /// commutative one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    order: Option<OrderForm>,
    /// The axiom links.
    links: Vec<(u32, u32)>,
    /// The vertices of a structure with boxes, which this form does not
    /// read: its links name those vertices, not occurrences.
    #[serde(default, skip_serializing)]
    vertices: Option<serde::de::IgnoredAny>,
}

/// An order other than the commutative one, by its name.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum OrderForm {
    /// The commutative order, which a writer leaves out.
    Commutative,
    /// The cyclic order.
    Cyclic,
}

impl From<OrderForm> for Order {
    /// The order the name names.
    fn from(form: OrderForm) -> Self {
        match form {
            OrderForm::Commutative => Order::Commutative,
            OrderForm::Cyclic => Order::Cyclic,
        }
    }
}

impl Serialize for ProofStructure {
    /// Serializes the structure as a document: its level, sequent,
    /// criterion and links.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let order = match self.criterion().order {
            Order::Commutative => None,
            Order::Cyclic => Some(OrderForm::Cyclic),
        };
        Net {
            // The cyclic order is level 2's.
            version: Some(if order.is_some() { 2 } else { wire::BASE }),
            sequent: Sequent::from(self.sequent()),
            mix: self.criterion().mix,
            order,
            vertices: None,
            links: self
                .links()
                .iter()
                .map(|&(x, y)| (x.get(), y.get()))
                .collect(),
        }
        .serialize(serializer)
    }
}

impl<'a> Deserialize<'a> for ProofStructure {
    /// Deserializes a structure within the default limits, as
    /// [`wire::upgrade`] does.
    fn deserialize<D: Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        Self::read(deserializer, &Limits::default())
    }
}

impl crate::sealed::Readable for ProofStructure {}

impl Readable for ProofStructure {
    const FORM: &'static str = "proof structure";

    /// Reads a structure, rebuilding the forest of its sequent within the
    /// limits and validating the links as [`ProofStructure::from_links`]
    /// does; whether it is a proof net is for
    /// [`ProofStructure::is_correct`].
    fn read<'de, D: Deserializer<'de>>(deserializer: D, limits: &Limits) -> Result<Self, D::Error> {
        let net = Net::deserialize(deserializer)?;
        if net.vertices.is_some() {
            return Err(serde::de::Error::custom(
                "a proof structure with boxes, whose links name its vertices: read it as one",
            ));
        }
        let build = || {
            let sequent = crate::Sequent::try_from(net.sequent)?;
            let forest = Forest::from_owned(sequent, limits)?;
            let links: Vec<(VertexId, VertexId)> = net
                .links
                .into_iter()
                .map(|(x, y)| (VertexId::new(x), VertexId::new(y)))
                .collect();
            let criterion = Criterion {
                mix: net.mix,
                order: net.order.map_or(Order::Commutative, Order::from),
            };
            ProofStructure::from_links(forest, criterion, &links)
        };
        build().map_err(wire::fail)
    }
}

/// The serialized form of a proof structure with boxes: its level, its
/// sequent, whether Mix is allowed, the vertices after the roots as
/// `[occurrence, parent, box]` (`box` the `!` vertex of the innermost box
/// around the vertex, or `null` outside every box), the axiom links and
/// the weakenings (a `?` vertex without copies and the literal it hangs
/// on), as vertex ids.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Boxed {
    /// The wire level.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "wire::version"
    )]
    version: Option<u32>,
    /// The sequent.
    sequent: Sequent,
    /// Whether Mix is allowed.
    mix: bool,
    /// The vertices after the roots.
    vertices: Vec<(u32, u32, Option<u32>)>,
    /// The axiom links.
    links: Vec<(u32, u32)>,
    /// The weakenings.
    weakenings: Vec<(u32, u32)>,
}

impl Serialize for MellStructure {
    /// Serializes the structure as a document: its level, sequent,
    /// criterion, vertices, links and weakenings.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let parts = self.parts();
        let pair = |&(x, y): &(VertexId, VertexId)| (x.get(), y.get());
        Boxed {
            version: Some(wire::BOXES),
            sequent: Sequent::from(self.forest().sequent()),
            mix: self.criterion().mix,
            vertices: parts
                .vertices
                .iter()
                .map(|p| {
                    (
                        p.occurrence.get(),
                        p.parent.get(),
                        p.inside.map(VertexId::get),
                    )
                })
                .collect(),
            links: parts.links.iter().map(pair).collect(),
            weakenings: parts.weakenings.iter().map(pair).collect(),
        }
        .serialize(serializer)
    }
}

impl<'a> Deserialize<'a> for MellStructure {
    /// Deserializes a structure within the default limits, as
    /// [`wire::upgrade`] does.
    fn deserialize<D: Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        Self::read(deserializer, &Limits::default())
    }
}

impl crate::sealed::Readable for MellStructure {}

impl Readable for MellStructure {
    const FORM: &'static str = "proof structure with boxes";

    /// Reads a structure, rebuilding the forest of its sequent within the
    /// limits and validating its vertices, links and weakenings as
    /// [`MellStructure::from_parts`] does, its graph within
    /// `limits.memory_bytes`; whether it is a proof net is for
    /// [`MellStructure::is_correct`].
    fn read<'de, D: Deserializer<'de>>(deserializer: D, limits: &Limits) -> Result<Self, D::Error> {
        let boxed = Boxed::deserialize(deserializer)?;
        let build = || {
            let sequent = crate::Sequent::try_from(boxed.sequent)?;
            let forest = Forest::from_owned(sequent, limits)?;
            let pair = |&(x, y): &(u32, u32)| (VertexId::new(x), VertexId::new(y));
            let parts = Parts {
                vertices: boxed
                    .vertices
                    .iter()
                    .map(|&(o, p, b)| {
                        Placed::new(OccId::new(o), VertexId::new(p), b.map(VertexId::new))
                    })
                    .collect(),
                links: boxed.links.iter().map(pair).collect(),
                weakenings: boxed.weakenings.iter().map(pair).collect(),
            };
            let criterion = Criterion {
                mix: boxed.mix,
                order: Order::Commutative,
            };
            MellStructure::from_parts(forest, criterion, &parts, limits)
        };
        build().map_err(wire::fail)
    }
}
