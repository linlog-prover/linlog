// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use super::proofs::Proof;
use crate::fragment::{Fragment, Mode};
use crate::occurrences::Member;
use crate::search::{
    Bias, Cadence, Engine, Equation, Jobs, Outcome as Out, Reason, Refutation, StateEquation,
    Statistics, Unbalanced, Verdict,
};
use crate::sequents::Sequent;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

impl Serialize for Fragment {
    /// Serializes the fragment as its name, such as `"MALL"`.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.name())
    }
}

impl<'a> Deserialize<'a> for Fragment {
    /// Deserializes a fragment from its name, giving the named fragment,
    /// which contains every fragment of that name.
    fn deserialize<D: Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        let name = String::deserialize(deserializer)?;
        name.parse().map_err(serde::de::Error::custom)
    }
}

/// The serialized form of a mode: its three flags by name.
#[derive(Serialize, Deserialize)]
#[serde(remote = "Mode")]
struct ModeDef {
    /// Intuitionistic rather than classical.
    intuitionistic: bool,
    /// Weakening allowed.
    affine: bool,
    /// Mix allowed.
    mix: bool,
}

impl Serialize for Mode {
    /// Serializes the mode as an object of its three flags.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        ModeDef::serialize(self, serializer)
    }
}

impl<'a> Deserialize<'a> for Mode {
    /// Deserializes a mode from an object of its three flags.
    fn deserialize<D: Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        ModeDef::deserialize(deserializer)
    }
}

/// The serialized form of a reason: a tag, with the bound for the copy
/// bound and for the memory limit.
#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum Why {
    /// The stop condition fired.
    Stopped,
    /// The recursion limit was reached.
    RecursionLimit,
    /// Every level up to this copy bound hit it.
    CopyBound(u32),
    /// The search held this many bytes with its memo emptied.
    MemoryLimit(u64),
    /// A structure of the search outgrew its indices.
    IndexLimit,
}

impl From<Reason> for Why {
    /// Converts a reason into its serialized form.
    fn from(r: Reason) -> Self {
        match r {
            Reason::Stopped => Why::Stopped,
            Reason::RecursionLimit => Why::RecursionLimit,
            Reason::CopyBound(n) => Why::CopyBound(n),
            Reason::MemoryLimit(bytes) => Why::MemoryLimit(bytes),
            Reason::IndexLimit => Why::IndexLimit,
        }
    }
}

/// The serialized form of a refutation: a tag, with the counts or the
/// weights that rule a proof out.
#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum WhyNot {
    /// The search was exhaustive.
    Exhausted,
    /// An atom's literals cannot pair up.
    Unbalanced {
        /// The atom's index into the sequent's atoms.
        atom: u32,
        /// The least excess of positive over negative literals.
        least: i32,
        /// The greatest.
        most: i32,
    },
    /// The count equation fails.
    Equation {
        /// The formulas.
        formulas: u64,
        /// The formulas the equation asks for.
        needed: i64,
        /// The `⊗`.
        tensors: u64,
        /// The `⅋`.
        pars: u64,
        /// The `1`.
        ones: u64,
        /// The `⊥`.
        bottoms: u64,
        /// Whether Mix was allowed.
        mix: bool,
    },
    /// The state equation of the Petri net has no solution.
    StateEquation {
        /// The atoms that weigh something, by index, with their weights.
        atoms: Vec<(u32, i64)>,
        /// The clauses used once that weigh something, by occurrence, with
        /// their weights.
        clauses: Vec<(u32, i64)>,
        /// The clauses that can never fire, by occurrence.
        dropped: Vec<u32>,
    },
}

impl From<&Refutation> for WhyNot {
    /// Converts a refutation into its serialized form.
    fn from(r: &Refutation) -> Self {
        match r {
            &Refutation::Unbalanced(Unbalanced { atom, least, most }) => WhyNot::Unbalanced {
                atom: atom.get(),
                least,
                most,
            },
            &Refutation::Equation(Equation {
                formulas,
                needed,
                tensors,
                pars,
                ones,
                bottoms,
                mix,
            }) => WhyNot::Equation {
                formulas,
                needed,
                tensors,
                pars,
                ones,
                bottoms,
                mix,
            },
            Refutation::StateEquation(StateEquation {
                atoms,
                clauses,
                dropped,
            }) => WhyNot::StateEquation {
                atoms: atoms.iter().map(|&(a, w)| (a.get(), w)).collect(),
                clauses: clauses.iter().map(|&(o, w)| (o.get(), w)).collect(),
                dropped: dropped.iter().map(|o| o.get()).collect(),
            },
            _ => WhyNot::Exhausted,
        }
    }
}

/// The keys an unprovable outcome adds for its disproof beside its
/// `mode` and `refutation`: the sequent, and the goal when it is not the
/// roots.
#[derive(Serialize)]
struct DisproofKeys<'a> {
    /// The sequent.
    sequent: &'a Sequent,
    /// The goal refuted, absent for the roots.
    #[serde(skip_serializing_if = "Option::is_none")]
    goal: Option<&'a [Member]>,
}

/// The serialized form of the statistics: its counters by name.
#[derive(Serialize)]
#[serde(remote = "Statistics")]
struct StatisticsDef {
    /// Stable sequents visited.
    nodes: u64,
    /// Visits answered from the memo.
    memo_hits: u64,
    /// The peak size of the memo.
    memo_entries: usize,
    /// Context splits examined.
    splits: u64,
    /// Axiom links the net engine tried.
    links: u64,
    /// Exact acyclicity tests the net engine ran.
    tests: u64,
    /// The largest copy bound the deepening reached.
    copies: u32,
}

/// The serialized form of an outcome: the verdict as a word, the reason for
/// `unknown`, how the search ran, and for `proved` the proof's sequent and
/// nodes as keys of the outcome itself, so that the outcome reads as a
/// proof file too.
#[derive(Serialize)]
struct Outcome<'a> {
    /// `proved`, `unprovable` or `unknown`.
    verdict: &'static str,
    /// Why the search could not decide.
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<Why>,
    /// Why the sequent is unprovable.
    #[serde(skip_serializing_if = "Option::is_none")]
    refutation: Option<WhyNot>,
    /// The fragment searched in, by its name in the mode.
    fragment: &'static str,
    /// The mode searched in.
    mode: Mode,
    /// The engine that ran.
    engine: Engine,
    /// What the search cost.
    #[serde(with = "StatisticsDef")]
    statistics: Statistics,
    /// The proof, for `proved`.
    #[serde(flatten)]
    proof: Option<Proof>,
    /// What the disproof adds, for `unprovable`.
    #[serde(flatten)]
    disproof: Option<DisproofKeys<'a>>,
}

impl Serialize for Out {
    /// Serializes the outcome as its verdict, the fragment, mode and engine
    /// of the search, the statistics, and the proof if there is one.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let (verdict, reason, refutation, proof, disproof) = match &self.verdict {
            Verdict::Proved(p) => ("proved", None, None, Some(Proof::from(&**p)), None),
            Verdict::Unprovable(d) => {
                let keys = DisproofKeys {
                    sequent: d.sequent(),
                    goal: d.goal(),
                };
                let why = WhyNot::from(d.refutation());
                ("unprovable", None, Some(why), None, Some(keys))
            }
            Verdict::Unknown(r) => ("unknown", Some(Why::from(*r)), None, None, None),
        };
        Outcome {
            verdict,
            reason,
            refutation,
            fragment: self.fragment.name_in(self.mode),
            mode: self.mode,
            engine: self.engine,
            statistics: self.statistics,
            proof,
            disproof,
        }
        .serialize(serializer)
    }
}

impl Serialize for Engine {
    /// Serializes the engine as its name, such as `"focus"`.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'a> Deserialize<'a> for Engine {
    /// Deserializes an engine from its name.
    fn deserialize<D: Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}

impl Serialize for Bias {
    /// Serializes the rule as its name, such as `"rarer"`.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'a> Deserialize<'a> for Bias {
    /// Deserializes a rule from its name.
    fn deserialize<D: Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}

/// A value written `"auto"` or as a number.
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum AutoOrNumber {
    /// A number.
    Number(u64),
    /// A word, of which `"auto"` is the one read.
    Word(String),
}

impl AutoOrNumber {
    /// Returns the number, or `None` for `"auto"`, or the error for any
    /// other word.
    fn number<E: serde::de::Error>(self) -> Result<Option<u64>, E> {
        match self {
            Self::Number(n) => Ok(Some(n)),
            Self::Word(word) if word == "auto" => Ok(None),
            Self::Word(word) => Err(E::custom(format!(
                "expected a number or \"auto\", found {word:?}"
            ))),
        }
    }
}

impl Serialize for Jobs {
    /// Serializes the threads as `"auto"` or their number.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Auto => serializer.serialize_str("auto"),
            Self::Count(n) => serializer.serialize_u64(u64::try_from(*n).unwrap_or(u64::MAX)),
        }
    }
}

impl<'a> Deserialize<'a> for Jobs {
    /// Deserializes the threads from `"auto"` or a number.
    fn deserialize<D: Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(match AutoOrNumber::deserialize(deserializer)?.number()? {
            None => Self::Auto,
            Some(n) => Self::Count(usize::try_from(n).unwrap_or(usize::MAX)),
        })
    }
}

impl Serialize for Cadence {
    /// Serializes the cadence as `"auto"` or the number of links.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Auto => serializer.serialize_str("auto"),
            Self::Every(n) => serializer.serialize_u32(*n),
        }
    }
}

impl<'a> Deserialize<'a> for Cadence {
    /// Deserializes the cadence from `"auto"` or a number of links.
    fn deserialize<D: Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(match AutoOrNumber::deserialize(deserializer)?.number()? {
            None => Self::Auto,
            Some(n) => Self::Every(u32::try_from(n).map_err(serde::de::Error::custom)?),
        })
    }
}
