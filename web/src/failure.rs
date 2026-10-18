// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Why a call answered no value: the library's error in its written form,
//! which the page words itself.

use linlog::{Error, Owner};
use std::fmt::{self, Display, Formatter};

/// A failed call: the JSON of the error, `{"code", "kind", "message",
/// "setting", "details"}` as the library writes an [`Error`], so that the
/// page branches on `code` and `kind` and never on the message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Failure(pub String);

impl Failure {
    /// Returns the failure of an error of the library whose formulas
    /// `owner` writes.
    pub fn described(error: &Error, owner: &impl Owner) -> Self {
        Self(serde_json::to_string(&error.describe(owner)).unwrap_or_else(|_| written(error)))
    }

    /// Returns the failure of a document that is no JSON or not of the
    /// form asked for, in the library's form of such an error.
    pub fn json(error: serde_json::Error) -> Self {
        let message = format!("not the JSON asked for: {error}");
        Self(
            serde_json::json!({
                "code": "json",
                "kind": "malformed",
                "message": message,
                "details": {"line": error.line(), "column": error.column()},
            })
            .to_string(),
        )
    }

    /// Returns the failure of a request the bindings refuse before the
    /// library sees it, with the code and kind the library would give it.
    pub fn refused(code: &str, kind: &str, message: &str) -> Self {
        Self(serde_json::json!({"code": code, "kind": kind, "message": message}).to_string())
    }
}

impl From<Error> for Failure {
    /// The error's written form.
    fn from(error: Error) -> Self {
        Self(written(&error))
    }
}

impl Display for Failure {
    /// Writes the JSON.
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Returns an error's written form.
fn written(error: &Error) -> String {
    serde_json::to_string(error).unwrap_or_else(|_| {
        serde_json::json!({"code": error.code(), "message": error.to_string()}).to_string()
    })
}
