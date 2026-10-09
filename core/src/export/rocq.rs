// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Rocq: a derivation as a proof script that the NanoYalla kernel checks,
//! version [`NANOYALLA`], the kernel of Click & coLLecT. The script is a
//! `Lemma` whose statement is the sequent as a list of the kernel's
//! formulas, `ll [dual A; tens A (dual B); B]`, over the atoms as
//! variables of type `formula`, and whose proof applies one derived rule
//! of the kernel's `macroll` module per inference, conclusion first:
//! `tens_r_ext`, `parr_r_ext`, `oc_r_ext`, `co_r_ext` and so on, the
//! axiom as `ax_expansion`. `Qed` closes it, so Rocq accepts the file only
//! if the kernel accepts the proof.
//!
//! The kernel's sequents are lists and its rules act on a formula at a
//! given position, so the script keeps the list of every goal: the lemma
//! states the conclusion in occurrence order, each rule takes the prefix
//! before its principal formula, and a `⊗` alone, whose left premise must
//! hold what precedes the `⊗` and whose right premise what follows it,
//! gets an exchange `ex_perm_r` before it when the goal is not already in
//! that order.
//!
//! The certificate is one of classical linear logic: a two-sided
//! derivation is exported as the one-sided derivation behind it
//! (the one-sided [`Rule`] of every [`Named`](crate::proofs::Named) rule), which proves the same
//! sequent read one-sided.
//! Mix, the weakening of affine mode and an open goal have no rule in the
//! kernel, and a derivation with one is refused ([`Unsupported`]). A
//! fragment is the lemma alone, for a file that imports `macroll`; a
//! standalone file starts with [`Options::prelude`].
//!
//! An atom's name becomes a Rocq identifier: letters, digits, `_` and `'`
//! stay, any other character becomes its code point in hexadecimal between
//! underscores, a leading digit or `'` gets an underscore before it, and a
//! name that would clash with a keyword, a name of the kernel, the lemma's
//! name or another atom gets `'` appended until it is free.
//!
//! Needs the cargo feature `rocq` (on by default).
//!
//! # Examples
//!
#![cfg_attr(feature = "parse", doc = "```")]
#![cfg_attr(not(feature = "parse"), doc = "```ignore")]
//! use linlog::export::rocq::{self, Options};
//! use linlog::{Mode, Options as Search, Sequent, Verdict, prove};
//!
//! let sequent: Sequent = "A, A -o B |- B".parse()?;
//! let outcome = prove(&sequent, Mode::CLASSICAL, &Search::default())?;
//! let Verdict::Proved(proof) = &outcome.verdict else {
//!     panic!("provable");
//! };
//! let mut written = String::new();
//! rocq::write(&proof.derivation()?, &Options::default(), &mut written, |_| false)?;
//! assert_eq!(
//!     written,
//!     "Lemma certificate (A B : formula) : ll [dual A; tens A (dual B); B].
//! Proof.
//! apply (tens_r_ext [dual A]); cbn_sequent.
//! {
//!   ax_expansion.
//! }
//! {
//!   ax_expansion.
//! }
//! Qed."
//! );
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use super::notation::{Step, flush, walk};
use super::{Drawable, Form};
use crate::Error;
use crate::hash::{HashMap, HashSet};
use crate::occurrences::{Forest, OccId};
use crate::proofs::{Derivation, InfId, Rule};
use crate::sequents::{Sequent, Term, TermId, Visit, Walk};
use std::fmt::Write;
use thiserror::Error;

/// The version of NanoYalla the scripts are written for: the `nanoyalla`
/// directory of Click & coLLecT, whose `macroll` module defines the
/// derived rules the scripts apply. It builds on Rocq 9 with the standard
/// library, without Yalla.
pub const NANOYALLA: &str = "1.1.3";

/// What a user may vary in a certificate.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serialize", serde(default, deny_unknown_fields))]
pub struct Options {
    /// The lemma alone, or a whole file that starts with the prelude.
    pub form: Form,
    /// The name of the lemma.
    pub lemma: Identifier,
    /// The lines a standalone file starts with, before the lemma, or
    /// `None` for the target's own: the import of NanoYalla's `macroll`
    /// for a linear certificate ([`NANOYALLA_PRELUDE`]), of the standard
    /// library's excluded middle for a classical one over `Prop`
    /// ([`CLASSICAL_PRELUDE`]), nothing for one of LJ.
    pub prelude: Option<String>,
}

impl Default for Options {
    /// Returns the lemma `certificate` alone, and each target's own
    /// prelude for a whole file.
    fn default() -> Self {
        Self {
            form: Form::Fragment,
            lemma: Identifier::new("certificate").expect("an identifier"),
            prelude: None,
        }
    }
}

impl Options {
    /// Returns the options with the form given.
    #[must_use]
    pub fn with_form(self, form: Form) -> Self {
        Self { form, ..self }
    }

    /// Returns the options with the lemma given.
    #[must_use]
    pub fn with_lemma(self, lemma: Identifier) -> Self {
        Self { lemma, ..self }
    }

    /// Returns the options with the prelude given, `None` for each
    /// target's own.
    #[must_use]
    pub fn with_prelude(self, prelude: Option<String>) -> Self {
        Self { prelude, ..self }
    }
}

/// The prelude of a linear certificate: the import of NanoYalla's
/// `macroll`, which brings the kernel's `ll`, its formulas and its derived
/// rules into scope.
pub const NANOYALLA_PRELUDE: &str = "From NanoYalla Require Import macroll.";

/// The prelude of a classical certificate over `Prop`: the import of the
/// standard library's excluded middle, for `NNPP`.
pub const CLASSICAL_PRELUDE: &str = "From Stdlib Require Import Classical_Prop.";

/// A Rocq identifier, which a lemma's name must be: an ASCII letter or
/// `_` and then letters, digits, `_` and `'`, and no keyword of Rocq.
/// The names a kernel uses are its writer's to avoid, so a name stays an
/// identifier whatever kernel a later release adds.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Identifier(Box<str>);

impl Identifier {
    /// Returns the identifier `name` is.
    ///
    /// # Errors
    ///
    /// [`Error::InvalidOption`] for a name that is no identifier or a
    /// keyword.
    pub fn new(name: &str) -> Result<Self, Error> {
        let mut chars = name.chars();
        let lexical = chars
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
            && chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '\'');
        let why = if !lexical {
            "is no Rocq identifier: an ASCII letter or _, then letters, digits, _ and '"
        } else if KEYWORDS.contains(&name) {
            "is a keyword of Rocq"
        } else {
            return Ok(Self(name.into()));
        };
        Err(Error::InvalidOption {
            key: "rocq.lemma",
            message: format!("`{name}` {why}"),
        })
    }

    /// Returns the identifier's text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::str::FromStr for Identifier {
    type Err = Error;

    /// Reads an identifier as [`new`](Self::new) does.
    fn from_str(name: &str) -> Result<Self, Error> {
        Self::new(name)
    }
}

impl std::fmt::Display for Identifier {
    /// Writes the identifier.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(feature = "serialize")]
impl serde::Serialize for Identifier {
    /// Serializes the identifier as its text.
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

#[cfg(feature = "serialize")]
impl<'a> serde::Deserialize<'a> for Identifier {
    /// Deserializes an identifier from its text, refusing what
    /// [`new`](Self::new) refuses.
    fn deserialize<D: serde::Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        let name = <std::borrow::Cow<'a, str>>::deserialize(deserializer)?;
        Self::new(&name).map_err(serde::de::Error::custom)
    }
}

/// Why a derivation has no certificate: it uses a rule the kernel lacks.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Error)]
#[cfg_attr(
    feature = "serialize",
    derive(serde::Serialize),
    serde(rename_all = "snake_case")
)]
pub enum Unsupported {
    /// The derivation has an open goal: it is a proof in progress.
    #[error("the proof is not finished: an open goal has no certificate")]
    Open,
    /// The derivation uses Mix.
    #[error("NanoYalla has no Mix rule, so a proof with Mix has no certificate")]
    Mix,
    /// The derivation weakens a formula that is not a `?` formula.
    #[error(
        "NanoYalla weakens ? formulas only, so a proof with the weakening of affine mode has no certificate"
    )]
    AffineWeakening,
    /// The derivation draws a run of a structural rule as one inference,
    /// which names none of the formulas after the first.
    #[error(
        "a compact derivation names one formula of a run of structural rules, so it has no certificate: build the derivation with every rule"
    )]
    Compact,
}

/// Rocq's keywords, which no atom and no lemma may take.
pub(crate) const KEYWORDS: &[&str] = &[
    "_",
    "Axiom",
    "CoFixpoint",
    "Definition",
    "Fixpoint",
    "Hypothesis",
    "IF",
    "Lemma",
    "Parameter",
    "Proof",
    "Prop",
    "Qed",
    "SProp",
    "Set",
    "Theorem",
    "Type",
    "Variable",
    "as",
    "at",
    "by",
    "cofix",
    "discriminated",
    "else",
    "end",
    "exists",
    "exists2",
    "fix",
    "for",
    "forall",
    "fun",
    "if",
    "in",
    "lazymatch",
    "let",
    "match",
    "multimatch",
    "return",
    "then",
    "using",
    "where",
    "with",
];

/// The names of the kernel a script uses, which no atom may take.
const KERNEL: &[&str] = &[
    "Atom",
    "app",
    "aplus",
    "apply",
    "awith",
    "ax_expansion",
    "ax_r_ext",
    "bidual",
    "bot",
    "bot_r_ext",
    "cbn_sequent",
    "co_r_ext",
    "cons",
    "covar",
    "de_r_ext",
    "dual",
    "ex_perm_r",
    "ex_transpL",
    "formula",
    "list",
    "ll",
    "map",
    "nat",
    "nil",
    "O",
    "oc",
    "oc_r_ext",
    "one",
    "one_r_ext",
    "parr",
    "parr_r_ext",
    "permL_of_perm",
    "plus_r1_ext",
    "plus_r2_ext",
    "S",
    "tens",
    "tens_r_ext",
    "top",
    "top_r_ext",
    "var",
    "with_r_ext",
    "wk_r_ext",
    "wn",
    "zero",
];

/// Returns an atom's name as a Rocq identifier, before the check for
/// clashes.
fn identifier(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    if name.starts_with(|c: char| c.is_ascii_digit() || c == '\'') {
        out.push('_');
    }
    for c in name.chars() {
        if c.is_ascii_alphanumeric() || c == '_' || c == '\'' {
            out.push(c);
        } else {
            write!(out, "_{:x}_", c as u32).unwrap();
        }
    }
    if out.is_empty() {
        out.push('_');
    }
    out
}

/// Returns the identifiers of atoms with these names, in their order,
/// each free of clashes with the keywords, the kernel, `lemma` and the
/// others: the first clash of an identifier gets `'` appended, the later
/// ones `'2`, `'3` and so on.
pub(crate) fn identifiers(atoms: &[String], lemma: &str) -> Vec<String> {
    let mut names: Vec<String> = Vec::with_capacity(atoms.len());
    // The names taken so far, so that a clash costs a lookup: comparing
    // with every earlier name was quadratic in the atoms, seconds before
    // the first poll on a dictionary of a hundred thousand.
    let mut taken: HashSet<String> = HashSet::default();
    // Per identifier, the suffixes given to its clashes so far: names
    // that all map to one identifier (`ä` and `_e4_`) each took a prime
    // more than the last, quadratic in their number.
    let mut clashes: HashMap<String, u32> = HashMap::default();
    for name in atoms {
        let base = identifier(name);
        let free = |id: &str, taken: &HashSet<String>| {
            !KEYWORDS.contains(&id) && !KERNEL.contains(&id) && id != lemma && !taken.contains(id)
        };
        let mut id = base.clone();
        if !free(&id, &taken) {
            let suffix = clashes.entry(base.clone()).or_insert(0);
            loop {
                *suffix += 1;
                id = match *suffix {
                    1 => format!("{base}'"),
                    n => format!("{base}'{n}"),
                };
                if free(&id, &taken) {
                    break;
                }
            }
        }
        taken.insert(id.clone());
        names.push(id);
    }
    names
}

/// Writes the formula at `id` in the kernel's constructors, in brackets if
/// `argument` is set and it is an application.
fn term(out: &mut String, sequent: &Sequent, names: &[String], id: TermId, argument: bool) {
    use Term::*;
    for visit in Walk::new(id, argument, |k| sequent.term(k).operands()) {
        match visit {
            Visit::Enter(k, argument) => {
                let t = sequent.term(k);
                if argument && !matches!(t, Atom(_) | One | Bot | Top | Zero) {
                    out.push('(');
                }
                match t {
                    Atom(a) => out.push_str(&names[a.index()]),
                    DualAtom(a) => {
                        out.push_str("dual ");
                        out.push_str(&names[a.index()]);
                        if argument {
                            out.push(')');
                        }
                    }
                    One => out.push_str("one"),
                    Bot => out.push_str("bot"),
                    Top => out.push_str("top"),
                    Zero => out.push_str("zero"),
                    Tensor(..) => out.push_str("tens "),
                    Par(..) => out.push_str("parr "),
                    With(..) => out.push_str("awith "),
                    Plus(..) => out.push_str("aplus "),
                    Bang(_) => out.push_str("oc "),
                    Quest(_) => out.push_str("wn "),
                }
            }
            Visit::Between(_) => out.push(' '),
            // What has subformulas is an application.
            Visit::Exit(_, argument) => {
                if argument {
                    out.push(')');
                }
            }
        }
    }
}

/// Writes occurrences as a list of the kernel's formulas, `[A; dual B]`.
fn list(out: &mut String, forest: &Forest, names: &[String], occurrences: &[OccId]) {
    out.push('[');
    for (i, &o) in occurrences.iter().enumerate() {
        if i > 0 {
            out.push_str("; ");
        }
        term(out, forest.sequent(), names, forest.term(o), false);
    }
    out.push(']');
}

/// The script in progress.
struct Script<'a> {
    /// The derivation being written.
    derivation: &'a Derivation<'a>,
    /// The forest its occurrences index.
    forest: &'a Forest,
    /// The identifier of every atom.
    names: Vec<String>,
    /// The text so far.
    out: String,
    /// The list of formulas Rocq shows as the goal of each inference not
    /// yet written, set when its conclusion is written.
    goals: Vec<Vec<OccId>>,
    /// Whether an inference is a premise of a binary rule, whose script
    /// goes into braces of its own.
    braced: Vec<bool>,
    /// How many braces are open.
    depth: usize,
}

impl Script<'_> {
    /// Writes a tactic line at the current indentation.
    fn line(&mut self, text: &str) {
        for _ in 0..self.depth {
            self.out.push_str("  ");
        }
        self.out.push_str(text);
        self.out.push('\n');
    }

    /// Writes a rule's tactic, `apply (<rule> <arguments>); cbn_sequent.`,
    /// without the `cbn_sequent` when the rule leaves no goal.
    fn apply(&mut self, rule: &str, arguments: &[String], leaf: bool) {
        let mut text = format!("apply ({rule}");
        for a in arguments {
            text.push(' ');
            text.push_str(a);
        }
        text.push(')');
        text.push_str(if leaf { "." } else { "; cbn_sequent." });
        self.line(&text);
    }

    /// Returns occurrences as a list of the kernel's formulas.
    fn list(&self, occurrences: &[OccId]) -> String {
        let mut out = String::new();
        list(&mut out, self.forest, &self.names, occurrences);
        out
    }

    /// Writes the tactic of the inference `id`, whose goal is `goal`, and
    /// sets the goals of its premises.
    fn inference(&mut self, id: InfId, goal: Vec<OccId>) {
        let (derivation, forest) = (self.derivation, self.forest);
        let inference = derivation.inference(id);
        let rule = inference.rule.rule;
        if rule == Rule::Ax {
            self.line("ax_expansion.");
            return;
        }
        let o = inference.sequent[inference.principal.unwrap()];
        let at = goal.iter().position(|&x| x == o.occ()).unwrap();
        let (before, after) = (&goal[..at], &goal[at + 1..]);
        let prefix = self.list(before);
        let (left, right) = (forest.left(o.occ()), forest.right(o.occ()));
        // The goal of the premise when the rule replaces the principal
        // formula by `with`.
        let replaced = |with: &[OccId]| {
            let mut up = Vec::with_capacity(goal.len() + 1);
            up.extend_from_slice(before);
            up.extend_from_slice(with);
            up.extend_from_slice(after);
            up
        };
        let premise = |script: &mut Self, up: Vec<OccId>| {
            script.goals[inference.premises[0].index()] = up;
        };
        match rule {
            Rule::One => self.line("apply one_r_ext."),
            Rule::Top => self.apply("top_r_ext", &[prefix], true),
            Rule::Bot => {
                self.apply("bot_r_ext", &[prefix], false);
                premise(self, replaced(&[]));
            }
            Rule::Par => {
                self.apply("parr_r_ext", &[prefix], false);
                premise(self, replaced(&[left.unwrap(), right.unwrap()]));
            }
            Rule::PlusLeft => {
                self.apply("plus_r1_ext", &[prefix], false);
                premise(self, replaced(&[left.unwrap()]));
            }
            Rule::PlusRight => {
                self.apply("plus_r2_ext", &[prefix], false);
                premise(self, replaced(&[right.unwrap()]));
            }
            Rule::Dereliction => {
                self.apply("de_r_ext", &[prefix], false);
                premise(self, replaced(&[left.unwrap()]));
            }
            Rule::Weakening => {
                self.apply("wk_r_ext", &[prefix], false);
                premise(self, replaced(&[]));
            }
            Rule::Contraction => {
                self.apply("co_r_ext", &[prefix], false);
                premise(self, replaced(&[o.occ(), o.occ()]));
            }
            Rule::Promotion => {
                // The context is `?` formulas, which the rule takes
                // without their `?`.
                let under: Vec<OccId> = before
                    .iter()
                    .chain(after)
                    .map(|&q| forest.left(q).unwrap())
                    .collect();
                let (l1, l2) = under.split_at(before.len());
                let mut a = String::from("(");
                term(
                    &mut a,
                    forest.sequent(),
                    &self.names,
                    forest.term(left.unwrap()),
                    false,
                );
                a.push(')');
                let arguments = [self.list(l1), a, self.list(l2)];
                self.apply("oc_r_ext", &arguments, false);
                premise(self, replaced(&[left.unwrap()]));
            }
            Rule::With => {
                self.apply("with_r_ext", &[prefix], false);
                let (l, r) = (inference.premises[0], inference.premises[1]);
                self.goals[l.index()] = replaced(&[left.unwrap()]);
                self.goals[r.index()] = replaced(&[right.unwrap()]);
                self.braced[l.index()] = true;
                self.braced[r.index()] = true;
            }
            Rule::Tensor => self.tensor(id, goal),
            Rule::Ax | Rule::Mix | Rule::AffineWeakening | Rule::Open => {
                unreachable!("refused before anything is written")
            }
        }
    }

    /// Writes the tactics of a `⊗`: the exchange that puts the left
    /// premise's context before the `⊗` and the right premise's after it,
    /// when the goal is not in that order already, then the rule.
    fn tensor(&mut self, id: InfId, goal: Vec<OccId>) {
        let (derivation, forest) = (self.derivation, self.forest);
        let inference = derivation.inference(id);
        let o = inference.sequent[inference.principal.unwrap()];
        let (a, b) = (
            forest.left(o.occ()).unwrap(),
            forest.right(o.occ()).unwrap(),
        );
        let (l, r) = (inference.premises[0], inference.premises[1]);
        // The left context, consumed as the goal's formulas are assigned.
        let mut left: Vec<OccId> = derivation
            .inference(l)
            .sequent
            .iter()
            .map(|m| m.occ())
            .collect();
        left.remove(left.iter().position(|&x| x == a).unwrap());
        let (mut before, mut after) = (Vec::new(), Vec::new());
        let mut principal = false;
        for &x in &goal {
            if x == o.occ() && !principal {
                principal = true;
            } else if let Some(i) = left.iter().position(|&y| y == x) {
                left.remove(i);
                before.push(x);
            } else {
                after.push(x);
            }
        }
        let mut target = before.clone();
        target.push(o.occ());
        target.extend_from_slice(&after);
        if target != goal {
            // `ex_perm_r p l` proves `l` permuted so that position `i`
            // holds `l[p[i]]`, from `l`.
            let mut used = vec![false; target.len()];
            let p: Vec<String> = goal
                .iter()
                .map(|&x| {
                    let j = (0..target.len())
                        .find(|&j| !used[j] && target[j] == x)
                        .unwrap();
                    used[j] = true;
                    j.to_string()
                })
                .collect();
            let arguments = [format!("[{}]", p.join("; ")), self.list(&target)];
            self.apply("ex_perm_r", &arguments, true);
        }
        let prefix = self.list(&before);
        self.apply("tens_r_ext", &[prefix], false);
        before.push(a);
        after.insert(0, b);
        self.goals[l.index()] = before;
        self.goals[r.index()] = after;
        self.braced[l.index()] = true;
        self.braced[r.index()] = true;
    }
}

/// Writes the certificate of a derivation into `out`, in the options'
/// form, one inference at a time, and asks `stop` after each; a
/// derivation without a certificate is refused before anything is
/// written. A linear derivation is a lemma with its proof script for
/// NanoYalla, checked one-sided. One of LK or LJ, which must have passed
/// [`check`](crate::ordinary::Derivation::check), is the lemma
/// `options.lemma` stating the ordinary sequent over `Prop`, every atom a
/// proposition bound by `forall`, the hypotheses as premises and the
/// formulas right of `⊢` as their disjunction (`False` for none), proved
/// by a term made rule by rule; it needs no library, but a classical one
/// uses the excluded middle of the standard library (`NNPP`), which a
/// standalone file imports instead of `options.prelude`. A `String` takes
/// the whole.
///
/// # Errors
///
/// [`Error::Unsupported`] for a derivation without a certificate (an
/// open goal, Mix, affine weakening, a compact run), [`Error::GoalProof`]
/// for one of a goal off the roots, [`Refusal::Stopped`](crate::Refusal::Stopped)
/// when `stop` fired and [`Error::WriteFailed`] when `out` refused the
/// text.
pub fn write<'a>(
    derivation: impl Into<Drawable<'a>>,
    options: &Options,
    out: &mut impl Write,
    stop: impl FnMut(crate::limits::Progress) -> bool,
) -> Result<(), Error> {
    let stop = crate::limits::counting(stop, crate::limits::Phase::Write);
    match derivation.into() {
        Drawable::Linear(derivation) => linear(derivation, options, out, stop),
        Drawable::Ordinary(derivation) => {
            crate::ordinary::rocq::write(derivation, options, out, stop)
        }
    }
}

/// Writes the NanoYalla certificate of a linear derivation as [`write`]
/// does.
fn linear(
    derivation: &Derivation,
    options: &Options,
    out: &mut impl Write,
    mut stop: impl FnMut() -> bool,
) -> Result<(), Error> {
    if derivation.is_of_goal() {
        return Err(Error::GoalProof);
    }
    for inference in derivation.inferences() {
        if inference.times > 1 {
            return Err(Unsupported::Compact.into());
        }
        match inference.rule.rule {
            Rule::Open => return Err(Unsupported::Open.into()),
            Rule::Mix => return Err(Unsupported::Mix.into()),
            Rule::AffineWeakening => return Err(Unsupported::AffineWeakening.into()),
            Rule::Ax
            | Rule::Tensor
            | Rule::Par
            | Rule::One
            | Rule::Bot
            | Rule::With
            | Rule::PlusLeft
            | Rule::PlusRight
            | Rule::Top
            | Rule::Promotion
            | Rule::Dereliction
            | Rule::Contraction
            | Rule::Weakening => {}
        }
    }
    let forest = derivation.forest();
    let names = identifiers(forest.sequent().atom_names(), options.lemma.as_str());
    let count = derivation.inferences().len();
    let mut script = Script {
        derivation,
        forest,
        names,
        out: String::new(),
        goals: vec![Vec::new(); count],
        braced: vec![false; count],
        depth: 0,
    };
    let root = derivation.root();
    script.goals[root.index()] = derivation
        .inference(root)
        .sequent
        .iter()
        .map(|m| m.occ())
        .collect();

    if options.form == Form::Standalone {
        let prelude = options.prelude.as_deref().unwrap_or(NANOYALLA_PRELUDE);
        write!(out, "{}\n\n", prelude.trim_end())?;
    }
    write!(script.out, "Lemma {}", options.lemma)?;
    if !script.names.is_empty() {
        write!(script.out, " ({} : formula)", script.names.join(" "))?;
    }
    script.out.push_str(" : ll ");
    let conclusion = script.list(&script.goals[root.index()]);
    script.out.push_str(&conclusion);
    script.out.push_str(".\nProof.\n");
    walk(derivation, |step| {
        match step {
            Step::Enter(id, _) => {
                if script.braced[id.index()] {
                    script.line("{");
                    script.depth += 1;
                }
                let goal = std::mem::take(&mut script.goals[id.index()]);
                script.inference(id, goal);
            }
            Step::Exit(id, _) => {
                if script.braced[id.index()] {
                    script.depth -= 1;
                    script.line("}");
                }
            }
        }
        flush(out, &mut script.out, &mut stop)
    })?;
    out.write_str("Qed.")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Letters, digits, `_` and `'` stay; other characters become their
    /// code points; a clash with a keyword, the kernel, the lemma or
    /// another atom gets `'` appended.
    #[test]
    fn names_become_identifiers() {
        for (name, expected) in [
            ("A", "A"),
            ("x_1", "x_1"),
            ("p'", "p'"),
            ("1a", "_1a"),
            ("α", "_3b1_"),
            ("a b", "a_20_b"),
            ("", "_"),
        ] {
            assert_eq!(identifier(name), expected, "{name:?}");
        }
        let atoms: Vec<String> = ["tens", "with", "certificate", "α", "_3b1_", "_"]
            .map(String::from)
            .into();
        assert_eq!(
            identifiers(&atoms, "certificate"),
            ["tens'", "with'", "certificate'", "_3b1_", "_3b1_'", "_'"]
        );
        // Names that all map to one identifier take one suffix each.
        let atoms: Vec<String> = ["ää", "ä_e4_", "_e4_ä", "_e4__e4_"]
            .map(String::from)
            .into();
        assert_eq!(
            identifiers(&atoms, "certificate"),
            ["_e4__e4_", "_e4__e4_'", "_e4__e4_'2", "_e4__e4_'3"]
        );
    }
}
