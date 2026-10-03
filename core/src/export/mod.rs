// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Export of sequents and derivations: to LaTeX with the `ebproof` package
//! ([`latex`](crate::export::latex), feature `latex`), to Typst with the
//! `curryst` package ([`typst`](crate::export::typst), feature `typst`),
//! to SVG, with proof nets ([`svg`](crate::export::svg), feature `svg`),
//! and to Rocq as proof scripts for the NanoYalla kernel
//! ([`rocq`](crate::export::rocq), feature `rocq`).
//! Every function is a pure function of its input and of one options
//! value per target, which holds everything a user may vary and has serde
//! behind `serialize`; the output is deterministic. A derivation is
//! returned as a `String` (`derivation`) or written into any
//! [`std::fmt::Write`] with a stop condition asked between two inferences
//! (`write`), the same signature for every target and for the text tree
//! ([`Derivation::write_text`](crate::Derivation::write_text)). LaTeX,
//! Typst and Rocq come as a fragment to paste or as a standalone document
//! ([`Form`](crate::export::Form)), and SVG is always a whole document.
//!
//! All targets write formulas with the bracketing of `Display` (every
//! binary subformula of a formula in brackets) and derivations as
//! `Display` draws them: one-sided, or two-sided with the hypotheses in id
//! order when the derivation has a reading.
//! They write one inference at a time and keep their own stack, so the
//! text grows with the derivation, not with its width, and a derivation of
//! any height fits. Rules are labelled as [`Labels`](crate::Labels) says,
//! an open goal is drawn as [`OpenGoal`](crate::OpenGoal) says.

#[cfg(feature = "latex")]
pub mod latex;
/// The symbol tables and the printers the targets share.
#[cfg(any(
    feature = "latex",
    feature = "typst",
    feature = "svg",
    feature = "rocq"
))]
mod notation;
#[cfg(feature = "rocq")]
pub mod rocq;
#[cfg(feature = "svg")]
pub mod svg;
#[cfg(feature = "typst")]
pub mod typst;

/// Whether an export is a fragment to paste into a document or a document
/// of its own.
#[cfg(any(feature = "latex", feature = "typst", feature = "rocq"))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serialize", serde(rename_all = "lowercase"))]
pub enum Form {
    /// The sequent or the proof tree alone, for a document that loads the
    /// packages it needs.
    #[default]
    Fragment,
    /// A complete document that compiles on its own, cropped to its
    /// content, or a whole Rocq file with its import.
    Standalone,
}
