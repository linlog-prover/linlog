// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Problems of the LLTP library (`github.com/meta-logic/lltp`), the
//! benchmark collection of linear logic provers: TPTP-style files of
//! `fof(name, role, formula).` clauses whose axioms are the hypotheses and
//! whose conjecture is the goal. The formulas are written as this crate's
//! parser reads them (`*`, `|`, `&`, `+`, `-o`, `!`, `?`, postfix `^`,
//! `1`, `0`, `bot`, `top`), with the same precedences, so a problem is
//! read by assembling `axioms |- conjecture` and parsing that.
//!
//! The file says nothing about the mode: the library keeps its
//! intuitionistic problems under `ILL/` and its classical ones under
//! `CLL/`, and the caller picks the mode by that.
//!
//! # Examples
//!
//! ```
//! use linlog::Limits;
//! use linlog::lltp::{Status, read};
//!
//! let text = "% Status (intuit.) : Theorem\n\
//!             fof(ax1, axiom, !(a -o b)).\n\
//!             fof(ax2, axiom, a).\n\
//!             fof(con, conjecture, b * 1).\n";
//! let problem = read(text, &Limits::default())?;
//! assert_eq!(problem.status, Some(Status::Theorem));
//! assert_eq!(problem.sequent, "!(a -o b), a |- b * 1".parse()?);
//! # Ok::<(), linlog::Error>(())
//! ```
//!
//! Needs the cargo feature `parse` (on by default).

use crate::Limits;
use crate::{Error, Sequent};

/// What stands for a `-` inside an atom name, as the library's Petri nets
/// write them (`P-start_1_1`): this crate's names cannot hold a `-`, which
/// starts the operator `-o`, and `‿` is a character that joins words in a
/// Unicode identifier.
pub const HYPHEN: char = '‿';

/// What stands for a `.` inside an atom name, as some of the library's
/// Petri nets write them (`merge.s00001061.input`): a `.` ends a clause in
/// the file and is no name character here, and `·` is one in a Unicode
/// identifier.
pub const DOT: char = '·';

/// A problem read from an LLTP file.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct Problem {
    /// The sequent of the problem: the axioms, in the order of the file,
    /// left of `⊢`, the conjecture right of it.
    pub sequent: Sequent,
    /// The provability the file's header claims, if it has a `Status`
    /// line this reader understands.
    pub status: Option<Status>,
}

/// The provability an LLTP header claims.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    /// `Theorem`: the sequent is provable.
    Theorem,
    /// `Non-Theorem` or `CounterSatisfiable`: the sequent is not provable.
    NonTheorem,
}

/// Reads an LLTP problem. Every `fof` clause with the role `axiom` or
/// `hypothesis` is a hypothesis and the one with the role `conjecture`
/// the formula on the right of `⊢`; lines from `%` on are comments. The
/// status is the first `Status (intuit.)` or `Status (linear)` comment's,
/// else the first plain `Status` comment's: the library's translations of
/// intuitionistic problems keep the classical source's status first. A
/// `-` inside an atom name becomes [`HYPHEN`], a `.` [`DOT`].
///
/// # Errors
///
/// [`Error::Lltp`] for text that is not a sequence of `fof(name, role,
/// formula).` clauses (an annotation after the formula or an empty
/// formula included) or a clause with another role,
/// [`Error::SeveralConjectures`] for a file with more than one conjecture,
/// whose meaning in linear logic no convention fixes, and
/// [`Error::Parse`] for a formula this crate's parser rejects, and the
/// parser's refusal of a sequent past `limits.occurrences`.
pub fn read(text: &str, limits: &Limits) -> Result<Problem, Error> {
    let clauses = clauses(text, |message| Error::Lltp { message })?;
    let text = format!(
        "{} |- {}",
        clauses.hypotheses().collect::<Vec<_>>().join(", "),
        clauses.conjecture()
    );
    let sequent = Sequent::parse_within(&text, limits)?;
    Ok(Problem {
        sequent,
        status: clauses.status,
    })
}

/// The clauses of a TPTP-style file: its text with names made readable,
/// the ranges of its formulas in that text by role, and its status.
pub(crate) struct Clauses {
    /// The file's text without comments, a `-` inside a name as
    /// [`HYPHEN`] and a `.` as [`DOT`].
    code: String,
    /// The formulas of the clauses with the role `axiom` or `hypothesis`.
    hypotheses: Vec<std::ops::Range<usize>>,
    /// The formula of the one clause with the role `conjecture`.
    conjecture: std::ops::Range<usize>,
    /// The provability the header claims.
    pub(crate) status: Option<Status>,
}

impl Clauses {
    /// Returns the hypotheses' formulas, in the order of the file.
    pub(crate) fn hypotheses(&self) -> impl Iterator<Item = &str> {
        self.hypotheses.iter().map(|r| &self.code[r.clone()])
    }

    /// Returns the conjecture's formula.
    pub(crate) fn conjecture(&self) -> &str {
        &self.code[self.conjecture.clone()]
    }
}

/// Splits a file of `fof(name, role, formula).` clauses as [`read`]
/// describes, with every error of the file's form made by `error`: every
/// formula is not empty, and there is exactly one conjecture.
pub(crate) fn clauses(text: &str, error: fn(String) -> Error) -> Result<Clauses, Error> {
    // The status of a line for the logic of the problem, and of a plain one.
    let (mut status, mut plain) = (None, None);
    let mut code = String::with_capacity(text.len());
    for line in text.lines() {
        let (before, comment) = line.split_once('%').unwrap_or((line, ""));
        if let Some((key, word)) = comment
            .split_once("Status")
            .and_then(|(_, r)| r.rsplit_once(':'))
        {
            let word = word.trim();
            let value = if word.starts_with("Theorem") {
                Some(Status::Theorem)
            } else if word.starts_with("Non-Theorem") || word.starts_with("CounterSatisfiable") {
                Some(Status::NonTheorem)
            } else {
                None
            };
            let slot = if key.contains("(intuit") || key.contains("(linear") {
                &mut status
            } else {
                &mut plain
            };
            if slot.is_none() {
                *slot = value;
            }
        }
        let chars: Vec<char> = before.chars().collect();
        for (i, &c) in chars.iter().enumerate() {
            let name = |j: usize| {
                chars
                    .get(j)
                    .is_some_and(|&c| c.is_ascii_alphanumeric() || c == '_')
            };
            // A `-` or `.` between two characters of a name belongs to the
            // name, unless the `-` starts the operator `-o`.
            let inner = i > 0 && name(i - 1) && name(i + 1);
            code.push(match c {
                '-' if inner && !(chars[i + 1] == 'o' && !name(i + 2)) => HYPHEN,
                '.' if inner => DOT,
                c => c,
            });
        }
        code.push('\n');
    }

    let mut hypotheses = Vec::new();
    let mut conjecture = None;
    let mut rest = code.trim_start();
    while !rest.is_empty() {
        let body = rest
            .strip_prefix("fof")
            .map(str::trim_start)
            .and_then(|r| r.strip_prefix('('))
            .ok_or_else(|| error(format!("expected `fof(` at `{}`", excerpt(rest))))?;
        let (name, body) = body
            .split_once(',')
            .ok_or_else(|| error(format!("a clause without a role at `{}`", excerpt(rest))))?;
        let name = name.trim();
        let (role, body) = body
            .split_once(',')
            .ok_or_else(|| error(format!("clause `{name}` has no formula")))?;
        // The formula runs to the parenthesis that closes `fof(`; a comma
        // outside its parentheses starts an annotation.
        let mut depth = 1usize;
        let end = body
            .char_indices()
            .find(|&(_, c)| {
                match c {
                    '(' => depth += 1,
                    ')' => depth -= 1,
                    _ => {}
                }
                depth == 0 || (depth == 1 && c == ',')
            })
            .ok_or_else(|| error(format!("clause `{name}` is not closed")))?;
        if end.1 == ',' {
            return Err(error(format!(
                "clause `{name}` has an annotation, which this reader does not take"
            )));
        }
        let end = end.0;
        let formula = body[..end].trim();
        if formula.is_empty() {
            return Err(error(format!("clause `{name}` has an empty formula")));
        }
        let start = formula.as_ptr() as usize - code.as_ptr() as usize;
        let range = start..start + formula.len();
        match role.trim() {
            "axiom" | "hypothesis" => hypotheses.push(range),
            "conjecture" if conjecture.is_some() => {
                return Err(Error::SeveralConjectures {
                    second: name.to_owned(),
                });
            }
            "conjecture" => conjecture = Some(range),
            other => {
                return Err(error(format!(
                    "clause `{name}` has the role `{other}`, not axiom, hypothesis or conjecture"
                )));
            }
        }
        rest = body[end + 1..]
            .trim_start()
            .strip_prefix('.')
            .ok_or_else(|| error(format!("clause `{name}` does not end with `).`")))?
            .trim_start();
    }
    let Some(conjecture) = conjecture else {
        return Err(error("no conjecture".to_owned()));
    };
    Ok(Clauses {
        code,
        hypotheses,
        conjecture,
        status: status.or(plain),
    })
}

/// Returns the start of `text`, for an error message.
fn excerpt(text: &str) -> &str {
    let end = text.char_indices().nth(24).map_or(text.len(), |(i, _)| i);
    &text[..end]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Clauses over several lines, comments between and after them, a
    /// `hypothesis` role, uppercase atoms, names with `-` and `.`, postfix
    /// negation and every unit read into the sequent the parser gives for
    /// the same formulas; the intuitionistic status wins over the plain
    /// one before it, and malformed files are refused.
    #[test]
    fn reads_problems() {
        let text = "%---\n% Status   : Theorem\n% Status (intuit.) : Non-Theorem\n%---\n\
                    fof(ax1, axiom, !( (P-a_1) -o\n  (Q.b * Q.b) ) ).   % a transition\n\
                    fof(h, hypothesis, bot | top^).\n\
                    fof(con, conjecture, (Q.b * Q.b) + 0 & 1).\n";
        let problem = read(text, &Limits::default()).unwrap();
        assert_eq!(problem.status, Some(Status::NonTheorem));
        let expected: Sequent = "!(P‿a_1 -o Q·b * Q·b), bot | top^ |- (Q·b * Q·b) + 0 & 1"
            .parse()
            .unwrap();
        assert_eq!(problem.sequent, expected);

        for bad in [
            "fof(a, axiom, a).",
            "fof(a, axiom, a). fof(c, conjecture, a",
            "fof(c, conjecture, a) fof(d, axiom, b).",
            "fof(c, definition, a).",
            "fof(c, conjecture, a, unknown).",
            "cnf(c, conjecture, a).",
            "fof(h, axiom, bot). fof(c, conjecture, ).",
            "fof(h, axiom, ). fof(c, conjecture, a).",
        ] {
            assert!(
                matches!(read(bad, &Limits::default()), Err(Error::Lltp { .. })),
                "{bad:?}"
            );
        }
    }

    /// A second conjecture is refused by name: joined right of `⊢`, the
    /// two would be read as their par, and `⊢ a, ~a` is provable where
    /// neither `⊢ a` nor `⊢ ~a` is.
    #[test]
    fn refuses_several_conjectures() {
        let text = "fof(c1, conjecture, a). fof(c2, conjecture, a^).";
        assert!(matches!(
            read(text, &Limits::default()),
            Err(Error::SeveralConjectures { second }) if second == "c2"
        ));
    }
}
