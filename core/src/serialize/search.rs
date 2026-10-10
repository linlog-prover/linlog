// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use super::proofs::Proof;
use super::sequents::Sequent as SequentForm;
use crate::fragment::{Fragment, Mode};
use crate::limits::{Limits, Space};
use crate::occurrences::{Member, OccId};
use crate::search::{
    Bias, Cadence, Disproof, Engine, Equation, Jobs, Outcome as Out, Reason, Refutation,
    StateEquation, Statistics, Unbalanced, Verdict,
};
use crate::sequents::Atom;
use crate::wire::{self, Readable};
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

impl Serialize for Mode {
    /// Serializes the mode as its name, such as `"intuitionistic"`.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.name())
    }
}

impl<'a> Deserialize<'a> for Mode {
    /// Deserializes a mode from its name, refusing an unknown one by
    /// naming the known ones.
    fn deserialize<D: Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        let name = <std::borrow::Cow<'a, str>>::deserialize(deserializer)?;
        name.parse().map_err(serde::de::Error::custom)
    }
}

/// The serialized form of a reason: its `kind`, with the bound for the
/// copy bound and for the memory limit. A kind it does not know is no
/// reason, never one of these.
#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Why {
    /// The stop condition fired.
    Stopped,
    /// The recursion limit was reached.
    RecursionLimit {
        /// The depth the limits allowed.
        depth: u32,
    },
    /// Every level up to this copy bound hit it.
    CopyBound {
        /// The bound.
        copies: u32,
    },
    /// The search held this many bytes with its memo emptied.
    MemoryLimit {
        /// The bound.
        limit_bytes: u64,
    },
    /// A structure of the search outgrew its indices.
    IndexLimit,
    /// The work limit was reached.
    WorkLimit {
        /// The bound.
        limit: u64,
    },
    /// A proof was found whose check would hold more than the memory bound.
    Unchecked {
        /// The bound.
        limit_bytes: u64,
    },
}

impl From<Reason> for Why {
    /// Converts a reason into its serialized form.
    fn from(r: Reason) -> Self {
        match r {
            Reason::Stopped => Why::Stopped,
            Reason::RecursionLimit { depth } => Why::RecursionLimit { depth },
            Reason::CopyBound { copies } => Why::CopyBound { copies },
            Reason::MemoryLimit { limit_bytes } => Why::MemoryLimit { limit_bytes },
            Reason::IndexLimit => Why::IndexLimit,
            Reason::WorkLimit { limit } => Why::WorkLimit { limit },
            Reason::Unchecked { limit_bytes } => Why::Unchecked { limit_bytes },
        }
    }
}

impl From<Why> for Reason {
    /// Converts a reason's serialized form back.
    fn from(w: Why) -> Self {
        match w {
            Why::Stopped => Reason::Stopped,
            Why::RecursionLimit { depth } => Reason::RecursionLimit { depth },
            Why::CopyBound { copies } => Reason::CopyBound { copies },
            Why::MemoryLimit { limit_bytes } => Reason::MemoryLimit { limit_bytes },
            Why::IndexLimit => Reason::IndexLimit,
            Why::WorkLimit { limit } => Reason::WorkLimit { limit },
            Why::Unchecked { limit_bytes } => Reason::Unchecked { limit_bytes },
        }
    }
}

impl Serialize for Reason {
    /// Serializes the reason tagged by its `kind`.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        Why::from(*self).serialize(serializer)
    }
}

impl<'a> Deserialize<'a> for Reason {
    /// Deserializes a reason from its `kind`, refusing one it does not
    /// know.
    fn deserialize<D: Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        Why::deserialize(deserializer).map(Reason::from)
    }
}

/// The serialized form of a refutation: its `kind`, with the counts or
/// the weights that rule a proof out. A kind it does not know is no
/// refutation, never one of these.
#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
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
            Refutation::Exhausted => WhyNot::Exhausted,
        }
    }
}

impl From<WhyNot> for Refutation {
    /// Converts a refutation's serialized form back; its indices are the
    /// disproof's to check against its sequent.
    fn from(w: WhyNot) -> Self {
        match w {
            WhyNot::Exhausted => Refutation::Exhausted,
            WhyNot::Unbalanced { atom, least, most } => Refutation::Unbalanced(Unbalanced {
                atom: Atom::new(atom),
                least,
                most,
            }),
            WhyNot::Equation {
                formulas,
                needed,
                tensors,
                pars,
                ones,
                bottoms,
                mix,
            } => Refutation::Equation(Equation {
                formulas,
                needed,
                tensors,
                pars,
                ones,
                bottoms,
                mix,
            }),
            WhyNot::StateEquation {
                atoms,
                clauses,
                dropped,
            } => Refutation::StateEquation(StateEquation {
                atoms: atoms.into_iter().map(|(a, w)| (Atom::new(a), w)).collect(),
                clauses: clauses
                    .into_iter()
                    .map(|(o, w)| (OccId::new(o), w))
                    .collect(),
                dropped: dropped.into_iter().map(OccId::new).collect(),
            }),
        }
    }
}

impl Serialize for Refutation {
    /// Serializes the refutation tagged by its `kind`.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        WhyNot::from(self).serialize(serializer)
    }
}

impl<'a> Deserialize<'a> for Refutation {
    /// Deserializes a refutation from its `kind`, refusing one it does not
    /// know.
    fn deserialize<D: Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        WhyNot::deserialize(deserializer).map(Refutation::from)
    }
}

/// The serialized form of a disproof: its level at the top of a
/// document, the sequent, the mode, the refutation and the goal where it
/// is not the roots. An unprovable outcome holds the same keys, so it
/// reads back as its disproof.
#[derive(Serialize, Deserialize)]
struct DisproofForm {
    /// The wire level, written only at the top of a document.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "wire::version"
    )]
    version: Option<u32>,
    /// The sequent.
    sequent: SequentForm,
    /// The mode it is refuted in.
    mode: Mode,
    /// Why it is unprovable.
    refutation: WhyNot,
    /// The goal refuted, absent for the roots.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    goal: Option<Vec<u32>>,
}

impl DisproofForm {
    /// Rebuilds the disproof the form holds, its sequent within `limits`,
    /// every atom and occurrence it names checked against the sequent; the
    /// refutation itself is not checked.
    fn within(self, limits: &Limits) -> Result<Disproof, crate::Error> {
        let sequent = self.sequent.within(limits)?;
        let atoms = sequent.atom_names().len();
        let occurrences = usize::try_from(sequent.occurrences()).unwrap_or(usize::MAX);
        let atom = |atom: Atom| match atom.index() {
            index if index < atoms => Ok(()),
            index => Err(crate::Error::IndexOutOfBounds {
                space: Space::Atom,
                index,
                len: atoms,
            }),
        };
        let occurrence = |index: usize| match index {
            index if index < occurrences => Ok(()),
            index => Err(crate::Error::IndexOutOfBounds {
                space: Space::Occurrence,
                index,
                len: occurrences,
            }),
        };
        let refutation = Refutation::from(self.refutation);
        match &refutation {
            Refutation::Unbalanced(unbalanced) => atom(unbalanced.atom)?,
            Refutation::StateEquation(equation) => {
                for &(a, _) in &equation.atoms {
                    atom(a)?;
                }
                for &(o, _) in &equation.clauses {
                    occurrence(o.index())?;
                }
                for o in &equation.dropped {
                    occurrence(o.index())?;
                }
            }
            Refutation::Exhausted | Refutation::Equation(_) => {}
        }
        let disproof = Disproof::new(sequent, self.mode, refutation);
        Ok(match self.goal {
            None => disproof,
            Some(goal) => {
                for &m in &goal {
                    occurrence(m as usize)?;
                }
                disproof.of_goal(goal.into_iter().map(Member::new).collect())
            }
        })
    }
}

impl Serialize for Disproof {
    /// Serializes the disproof as a document.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        DisproofForm {
            version: wire::level(),
            sequent: SequentForm::from(self.sequent()),
            mode: self.mode(),
            refutation: WhyNot::from(self.refutation()),
            goal: self
                .goal()
                .map(|goal| goal.iter().map(|m| m.get()).collect()),
        }
        .serialize(serializer)
    }
}

impl<'a> Deserialize<'a> for Disproof {
    /// Deserializes a disproof within the default limits, as
    /// [`wire::upgrade`] does.
    fn deserialize<D: Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        Self::read(deserializer, &Limits::default())
    }
}

impl crate::sealed::Readable for Disproof {}

impl Readable for Disproof {
    const FORM: &'static str = "disproof";

    /// Reads a disproof: rebuilds its sequent within the limits and checks
    /// every index of its refutation and goal, not the refutation.
    fn read<'de, D: Deserializer<'de>>(deserializer: D, limits: &Limits) -> Result<Self, D::Error> {
        DisproofForm::deserialize(deserializer)?
            .within(limits)
            .map_err(wire::fail)
    }
}

/// The keys an unprovable outcome adds for its disproof beside its
/// `mode` and `refutation`: the sequent, and the goal when it is not the
/// roots.
#[derive(Serialize)]
struct DisproofKeys<'a> {
    /// The sequent.
    sequent: SequentForm,
    /// The goal refuted, absent for the roots.
    #[serde(skip_serializing_if = "Option::is_none")]
    goal: Option<&'a [Member]>,
}

/// The serialized form of an outcome: the verdict as a word, the reason for
/// `unknown`, how the search ran, and for `proved` the proof's sequent and
/// nodes as keys of the outcome itself, so that the outcome reads as a
/// proof file too.
#[derive(Serialize)]
struct Outcome<'a> {
    /// The wire level.
    version: u32,
    /// The crate's version, which wrote the outcome.
    linlog: &'static str,
    /// `proved`, `unprovable` or `unknown`.
    verdict: &'static str,
    /// Whether the proof passed the checker, for `proved`.
    #[serde(skip_serializing_if = "Option::is_none")]
    checked: Option<bool>,
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
        let verdict = self.verdict.name();
        let (reason, refutation, proof, disproof) = match &self.verdict {
            Verdict::Proved(p) => (None, None, Some(Proof::keys(p)), None),
            Verdict::Unprovable(d) => {
                let keys = DisproofKeys {
                    sequent: SequentForm::from(d.sequent()),
                    goal: d.goal(),
                };
                let why = WhyNot::from(d.refutation());
                (None, Some(why), None, Some(keys))
            }
            Verdict::Unknown(r) => (Some(Why::from(*r)), None, None, None),
        };
        Outcome {
            version: crate::wire::LEVEL,
            linlog: env!("CARGO_PKG_VERSION"),
            verdict,
            checked: proof.as_ref().map(|_| self.checked),
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Every reason reads back as itself, and a kind no reason has is
    /// refused.
    #[test]
    fn reasons_read_back() {
        for reason in [
            Reason::Stopped,
            Reason::RecursionLimit { depth: 2048 },
            Reason::CopyBound { copies: 3 },
            Reason::MemoryLimit {
                limit_bytes: 1 << 30,
            },
            Reason::IndexLimit,
            Reason::WorkLimit { limit: 1 << 20 },
            Reason::Unchecked {
                limit_bytes: 1 << 30,
            },
        ] {
            let json = serde_json::to_string(&reason).unwrap();
            assert_eq!(
                serde_json::from_str::<Reason>(&json).unwrap(),
                reason,
                "{json}"
            );
        }
        assert_eq!(
            serde_json::to_string(&Reason::CopyBound { copies: 3 }).unwrap(),
            r#"{"kind":"copy_bound","copies":3}"#
        );
        let unknown = serde_json::from_str::<Reason>(r#"{"kind":"tired"}"#);
        assert!(
            unknown
                .unwrap_err()
                .to_string()
                .contains("unknown variant `tired`")
        );
    }

    /// Every refutation reads back as itself, and a kind no refutation
    /// has is refused, never read as `exhausted`.
    #[test]
    fn refutations_read_back() {
        let (a, o) = (Atom::new, OccId::new);
        for refutation in [
            Refutation::Exhausted,
            Refutation::Unbalanced(Unbalanced {
                atom: a(1),
                least: 1,
                most: 2,
            }),
            Refutation::Equation(Equation {
                formulas: 2,
                needed: 3,
                tensors: 1,
                pars: 0,
                ones: 0,
                bottoms: 0,
                mix: false,
            }),
            Refutation::StateEquation(StateEquation {
                atoms: vec![(a(0), 2), (a(2), -1)],
                clauses: vec![(o(3), 1)],
                dropped: vec![o(5)],
            }),
        ] {
            let json = serde_json::to_string(&refutation).unwrap();
            let back = serde_json::from_str::<Refutation>(&json).unwrap();
            assert_eq!(back, refutation, "{json}");
        }
        let unknown = serde_json::from_str::<Refutation>(r#"{"kind":"tableau"}"#);
        let message = unknown.unwrap_err().to_string();
        assert!(message.contains("unknown variant `tableau`"), "{message}");
    }
}
