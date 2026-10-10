// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use super::proofs::Step;
use crate::Error;
use crate::nets::{NetError, VertexId};
use crate::occurrences::ShapeError;
use crate::proofs::{CheckError, Dyadic, Fault};
use crate::search::NotTaken;
use serde::ser::{Serialize, SerializeMap, Serializer};

/// Writes a map of the given entries.
macro_rules! entries {
    ($serializer:expr) => {
        $serializer.serialize_map(Some(0))?.end()
    };
    ($serializer:expr $(; $key:literal => $value:expr)+ $(;)?) => {{
        let mut map = $serializer.serialize_map(None)?;
        $(map.serialize_entry($key, $value)?;)+
        map.end()
    }};
}

impl Serialize for Error {
    /// Writes the error as a client reads it: `code`, what a program
    /// branches on; `kind`, what it shows; `message`, the English text;
    /// `setting`, the key of the bound that refused the call, where one
    /// did; and `details`, the variant's fields.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        envelope(self, &self.to_string(), serializer)
    }
}

/// Writes `error` as [`Error`]'s form does, with `message` for its text.
pub(crate) fn envelope<S: Serializer>(
    error: &Error,
    message: &str,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let mut map = serializer.serialize_map(None)?;
    map.serialize_entry("code", error.code())?;
    map.serialize_entry("kind", &error.kind())?;
    map.serialize_entry("message", message)?;
    if let Some(setting) = error.setting() {
        map.serialize_entry("setting", setting)?;
    }
    map.serialize_entry("details", &Details(error))?;
    map.end()
}

/// The fields of an error's variant, as a map.
struct Details<'a>(&'a Error);

impl Serialize for Details<'_> {
    /// Writes the variant's fields, or the wrapped error's form.
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use Error::*;
        match self.0 {
            #[cfg(feature = "parse")]
            Parse(e) => e.serialize(s),
            #[cfg(feature = "parse")]
            Lltp { message } | Spec { message } | Tptp { message } => {
                entries!(s; "message" => message)
            }
            #[cfg(feature = "parse")]
            SeveralConjectures { second } => entries!(s; "second" => second),
            UnknownName { what, name, known } => {
                entries!(s; "what" => what; "name" => name; "known" => known)
            }
            #[cfg(feature = "parse")]
            FamilySize {
                family,
                size,
                least,
                powers_of_two,
            } => entries!(s; "family" => family; "size" => size; "least" => least;
                "powers_of_two" => powers_of_two),
            InvalidOption { key, message } => entries!(s; "key" => key; "message" => message),
            Json { form, message } => entries!(s; "form" => form; "message" => message),
            Version {
                form,
                found,
                supported,
            } => entries!(s; "form" => form; "found" => found; "supported" => supported),
            IndexOutOfBounds { space, index, len } => {
                entries!(s; "space" => space; "index" => index; "len" => len)
            }
            NotTopological {
                space,
                index,
                parent,
            } => entries!(s; "space" => space; "index" => index; "parent" => parent),
            Antecedents { antecedents, roots } => {
                entries!(s; "antecedents" => antecedents; "roots" => roots)
            }
            AtomName { name } => entries!(s; "name" => name),
            #[cfg(feature = "interactive")]
            InconsistentSession { reason } => entries!(s; "reason" => reason),
            Check(e) => e.serialize(s),
            Net(e) => e.serialize(s),
            #[cfg(feature = "interactive")]
            Step(e) => e.serialize(s),
            #[cfg(feature = "interactive")]
            OpenGoals { count } => entries!(s; "count" => count),
            NotIntuitionistic(e) => e.serialize(s),
            GoalOutputs { count } | Succedents { count } => entries!(s; "count" => count),
            FragmentMismatch { asserted, detected } => {
                entries!(s; "asserted" => asserted; "detected" => detected)
            }
            Translation { translation, logic } => {
                entries!(s; "translation" => translation; "logic" => logic)
            }
            NoEngine { fragment, mode } => entries!(s; "fragment" => fragment; "mode" => mode),
            EngineRefused { engine, because } => {
                entries!(s; "engine" => engine; "because" => because)
            }
            #[cfg(feature = "rocq")]
            Unsupported(e) => entries!(s; "missing" => e),
            Refused(refusal) => refusal.serialize(s),
            #[cfg(feature = "parallel")]
            ThreadPool { threads, message } => {
                entries!(s; "threads" => threads; "message" => message)
            }
            #[cfg(any(feature = "png", feature = "pdf"))]
            NotSvg { message } | RenderFailed { message } => entries!(s; "message" => message),
            Rejected(e) => e.serialize(s),
            ReadBack { calculus, reason } => {
                entries!(s; "calculus" => calculus; "reason" => reason)
            }
            #[cfg(feature = "interactive")]
            ForeignProof | GoalMismatch => entries!(s),
            GoalProof => entries!(s),
            #[cfg(feature = "pdf")]
            NoDate => entries!(s),
            IntuitionisticMix | WriteFailed => entries!(s),
        }
    }
}

impl Serialize for NotTaken {
    /// Writes the reason tagged by its `kind`: `fragment` with the
    /// engine's largest and the goal's, `mode` with the mode, `goal`, or
    /// `shape`.
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            NotTaken::Fragment { decides, goal } => {
                entries!(s; "kind" => "fragment"; "decides" => decides; "goal" => goal)
            }
            NotTaken::Mode { mode } => entries!(s; "kind" => "mode"; "mode" => mode),
            NotTaken::Goal => entries!(s; "kind" => "goal"),
            NotTaken::Shape => entries!(s; "kind" => "shape"),
        }
    }
}

impl Serialize for CheckError {
    /// Writes the node at fault, its rule as the proof's form writes it,
    /// the sequents derived for its premises and the fault; or, for a
    /// check given up, the node it had come to and the refusal.
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Invalid(e) => entries!(s;
                "node" => &e.node.get();
                "rule" => &Step::from(e.rule);
                "fault" => &e.fault;
                "premises" => &e.premises),
            Self::Refused(e) => entries!(s; "node" => &e.node.get(); "refusal" => &e.refusal),
        }
    }
}

impl Serialize for Dyadic {
    /// Writes the two zones as ids and whether more is allowed.
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        entries!(s; "theta" => &self.theta; "gamma" => &self.gamma; "any" => &self.any)
    }
}

impl Serialize for Fault {
    /// Writes the fault's `kind` and its fields.
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Forbidden => entries!(s; "kind" => "forbidden"),
            Self::Shape(e) => entries!(s; "kind" => "shape"; "shape" => e),
            Self::Succedents { count } => entries!(s; "kind" => "succedents"; "count" => count),
            Self::Kind { member } => entries!(s; "kind" => "kind"; "member" => &member.get()),
            Self::NotDual => entries!(s; "kind" => "not_dual"),
            Self::Missing { premise, member } => entries!(s;
                "kind" => "missing"; "premise" => premise; "member" => &member.get()),
            Self::NotEmpty => entries!(s; "kind" => "not_empty"),
            Self::Differ => entries!(s; "kind" => "differ"),
            Self::NotUnderQuest { member } => {
                entries!(s; "kind" => "not_under_quest"; "member" => &member.get())
            }
            Self::Surplus => entries!(s; "kind" => "surplus"),
            Self::Conclusion { derived } => {
                entries!(s; "kind" => "conclusion"; "derived" => derived)
            }
        }
    }
}

impl Serialize for ShapeError {
    /// Writes the reason's `kind` and the occurrences it names.
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::NoGoal => entries!(s; "kind" => "no_goal"),
            Self::SeveralGoals { first, second } => entries!(s;
                "kind" => "several_goals"; "first" => &first.get(); "second" => &second.get()),
            Self::Formula { occurrence } => {
                entries!(s; "kind" => "formula"; "occurrence" => &occurrence.get())
            }
            Self::Succedents { count } => entries!(s; "kind" => "succedents"; "count" => count),
            Self::Hypothesis { index } => entries!(s; "kind" => "hypothesis"; "index" => index),
            Self::Undetermined { first, second } => entries!(s;
                "kind" => "undetermined"; "first" => &first.get(); "second" => &second.get()),
        }
    }
}

impl Serialize for NetError {
    /// Writes the reason's `kind` and the vertices it names.
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::NoVertex { vertex, vertices } => entries!(s;
                "kind" => "no_vertex"; "vertex" => vertex; "vertices" => vertices),
            Self::NotLiteral { vertex } => {
                entries!(s; "kind" => "not_literal"; "vertex" => &vertex.get())
            }
            Self::NotDual { x, y } => {
                entries!(s; "kind" => "not_dual"; "x" => &x.get(); "y" => &y.get())
            }
            Self::LinkedTwice { vertex } => {
                entries!(s; "kind" => "linked_twice"; "vertex" => &vertex.get())
            }
            Self::Unlinked { vertex } => {
                entries!(s; "kind" => "unlinked"; "vertex" => &vertex.get())
            }
            Self::Empty => entries!(s; "kind" => "empty"),
            Self::SwitchingCycle { cycle } => {
                entries!(s; "kind" => "switching_cycle"; "cycle" => &vertices(cycle))
            }
            Self::Disconnected { parts } => {
                let parts: Vec<Vec<u32>> = parts.iter().map(|p| vertices(p)).collect();
                entries!(s; "kind" => "disconnected"; "parts" => &parts)
            }
            Self::Fragment { fragment } => {
                entries!(s; "kind" => "fragment"; "fragment" => fragment)
            }
            Self::Mode { mode } => entries!(s; "kind" => "mode"; "mode" => mode),
            Self::Rule { node, rule } => {
                entries!(s; "kind" => "rule"; "node" => &node.get(); "rule" => rule)
            }
            Self::Refused { refusal } => entries!(s; "kind" => "refused"; "refusal" => refusal),
        }
    }
}

/// The raw ids of a list of vertices.
fn vertices(vs: &[VertexId]) -> Vec<u32> {
    vs.iter().map(|v| v.get()).collect()
}
