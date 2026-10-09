// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

/// The written form of errors.
mod errors;

pub(crate) use errors::envelope;

/// Serde for an optional choice, written `"auto"` for `None` and as the
/// value's name otherwise.
pub(crate) mod auto {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::fmt::Display;
    use std::str::FromStr;

    /// Writes `"auto"` or the value's name.
    pub(crate) fn serialize<T: Display, S: Serializer>(
        value: &Option<T>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match value {
            None => serializer.serialize_str("auto"),
            Some(value) => serializer.collect_str(value),
        }
    }

    /// Reads `"auto"` as `None` and any other name as the value it names.
    pub(crate) fn deserialize<'a, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
    where
        T: FromStr<Err: Display>,
        D: Deserializer<'a>,
    {
        let name = String::deserialize(deserializer)?;
        if name == "auto" {
            return Ok(None);
        }
        name.parse().map(Some).map_err(serde::de::Error::custom)
    }
}
/// Serde for an optional bound, read back exactly by JavaScript: a bound
/// past 2⁵³ is written `null`, no bound, which it is in effect, and read
/// as an error that says `null` lifts it.
pub(crate) mod exact {
    use serde::{Deserialize, Deserializer, Serializer};

    /// The largest integer JavaScript reads exactly.
    const MOST: u64 = 1 << 53;

    /// Writes the bound, or `null` for none or one past 2⁵³.
    pub(crate) fn serialize<S: Serializer>(
        value: &Option<u64>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match *value {
            Some(bound) if bound <= MOST => serializer.serialize_u64(bound),
            _ => serializer.serialize_none(),
        }
    }

    /// Reads a bound up to 2⁵³, or `null` for none.
    pub(crate) fn deserialize<'a, D: Deserializer<'a>>(
        deserializer: D,
    ) -> Result<Option<u64>, D::Error> {
        match Option::<u64>::deserialize(deserializer)? {
            Some(bound) if bound > MOST => Err(serde::de::Error::custom(format!(
                "the bound {bound} is past 2^53, which JavaScript cannot read exactly; null lifts it"
            ))),
            bound => Ok(bound),
        }
    }
}
/// Serde support for `Interactive`: the sequent, the mode, the inferences
/// and the steps.
#[cfg(feature = "interactive")]
mod interactive;
/// Serde support for `ProofStructure`: the sequent and the links.
mod nets;
/// Serde support for `Proof`, through a proxy with short tags.
mod proofs;
/// Serde support for `Fragment`, `Mode` and the outcome of a search.
mod search;
/// Serde support for `Sequent`, through a proxy with short tags.
mod sequents;
