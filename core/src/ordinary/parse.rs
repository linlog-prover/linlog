// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! A precedence parser for ordinary formulas and sequents that keeps its
//! own stack, in two dialects: this crate's syntax (documented on
//! [`Sequent`]) and the propositional formulas of TPTP's `fof`, which the
//! ILTP library is written in.

use super::{Formulas, Node, NodeId, Sequent};
use crate::Error;
use crate::errors::ParseError;
use crate::lltp::{Status, clauses};

/// A binary connective as it is written.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Binary {
    /// `<->`, `↔` or TPTP's `<=>`
    Iff,
    /// TPTP's `<~>`, the negated equivalence.
    Xor,
    /// `->`, `→` or TPTP's `=>`
    Implies,
    /// TPTP's `<=`, the implication from right to left.
    Reverse,
    /// `\/`, `∨` or TPTP's `|`
    Or,
    /// TPTP's `~|`
    Nor,
    /// `/\`, `∧` or TPTP's `&`
    And,
    /// TPTP's `~&`
    Nand,
}

impl Binary {
    /// Returns how tightly the connective binds.
    const fn strength(self) -> u8 {
        match self {
            Self::Iff | Self::Xor => 1,
            Self::Implies | Self::Reverse => 2,
            Self::Or | Self::Nor => 3,
            Self::And | Self::Nand => 4,
        }
    }

    /// Returns whether an operand between this connective and `next` is
    /// this one's: it binds tighter, or as tight and associates to the
    /// left, as the disjunctions and conjunctions do.
    const fn takes_before(self, next: Self) -> bool {
        self.strength() > next.strength()
            || (self.strength() == next.strength() && self.strength() >= 3)
    }
}

/// What waits for the operand to its right.
#[derive(Clone, Copy, Debug)]
enum Pending {
    /// `(`
    Open,
    /// `~` or `¬`
    Not,
    /// A binary connective with its left operand.
    Binary(Binary, NodeId),
}

/// The dialect read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Dialect {
    /// This crate's syntax, a whole sequent.
    Native,
    /// One formula of TPTP's `fof`.
    Tptp,
}

/// A text in the reading.
struct Parser<'a> {
    /// The text.
    input: &'a str,
    /// The byte offset of the next character.
    at: usize,
    /// The dialect.
    dialect: Dialect,
    /// The formulas read so far.
    formulas: &'a mut Formulas,
    /// What waits for its right operand, the innermost last.
    pending: Vec<Pending>,
    /// How many parentheses are open.
    open: usize,
}

/// What a formula's list ended with.
enum End {
    /// A comma: another formula of the same side follows.
    Comma,
    /// The turnstile.
    Turnstile,
    /// The end of the text.
    Stop,
}

impl<'a> Parser<'a> {
    /// Returns the error for the character at byte `at`, where `expected`
    /// could have stood.
    fn unexpected(&self, at: usize, expected: &'static [&'static str]) -> Error {
        Error::from(ParseError::new(self.input, at, expected))
    }

    /// Skips white space and returns the rest of the text.
    fn rest(&mut self) -> &'a str {
        let rest = &self.input[self.at..];
        self.at += rest.len() - rest.trim_start().len();
        &self.input[self.at..]
    }

    /// Consumes `token` if the text goes on with it.
    fn eat(&mut self, token: &str) -> bool {
        let found = self.rest().starts_with(token);
        if found {
            self.at += token.len();
        }
        found
    }

    /// Consumes a negation if one starts here: `~` or `¬`, and in TPTP a
    /// `~` that does not start `~|` or `~&`.
    fn negation(&mut self) -> bool {
        match self.dialect {
            Dialect::Native => self.eat("~") || self.eat("¬"),
            Dialect::Tptp => {
                let rest = self.rest();
                !rest.starts_with("~|") && !rest.starts_with("~&") && self.eat("~")
            }
        }
    }

    /// Reads an identifier if one starts here.
    fn identifier(&mut self) -> Option<&'a str> {
        let rest = self.rest();
        let first = rest.chars().next()?;
        if !(first == '_' || unicode_ident::is_xid_start(first)) {
            return None;
        }
        let end = rest
            .find(|c| !unicode_ident::is_xid_continue(c))
            .unwrap_or(rest.len());
        self.at += end;
        Some(&rest[..end])
    }

    /// Reads one formula up to a comma, the turnstile or the end of the
    /// text outside every parenthesis, which it says.
    fn formula(&mut self) -> Result<(NodeId, End), Error> {
        loop {
            // An operand, after any prefix operators and parentheses.
            let mut operand = loop {
                if self.eat("(") {
                    self.open += 1;
                    self.pending.push(Pending::Open);
                } else if self.negation() {
                    self.pending.push(Pending::Not);
                } else {
                    break self.constant_or_atom()?;
                };
            };
            // The operators after it.
            loop {
                let at = self.at;
                let binary = match self.dialect {
                    Dialect::Native => [
                        ("<->", Binary::Iff),
                        ("↔", Binary::Iff),
                        ("->", Binary::Implies),
                        ("→", Binary::Implies),
                        ("\\/", Binary::Or),
                        ("∨", Binary::Or),
                        ("/\\", Binary::And),
                        ("∧", Binary::And),
                    ]
                    .into_iter()
                    .find(|(token, _)| self.eat(token)),
                    Dialect::Tptp => [
                        ("<=>", Binary::Iff),
                        ("<~>", Binary::Xor),
                        ("=>", Binary::Implies),
                        ("<=", Binary::Reverse),
                        ("~|", Binary::Nor),
                        ("|", Binary::Or),
                        ("~&", Binary::Nand),
                        ("&", Binary::And),
                    ]
                    .into_iter()
                    .find(|(token, _)| self.eat(token)),
                };
                if let Some((_, binary)) = binary {
                    let left = self.reduce(operand, Some(binary))?;
                    self.pending.push(Pending::Binary(binary, left));
                    break;
                }
                if self.open > 0 && self.eat(")") {
                    operand = self.reduce(operand, None)?;
                    self.pending.pop();
                    self.open -= 1;
                    continue;
                }
                let end = if self.open > 0 {
                    None
                } else if self.dialect == Dialect::Tptp {
                    self.rest().is_empty().then_some(End::Stop)
                } else if self.eat(",") {
                    Some(End::Comma)
                } else if self.eat("|-") || self.eat("⊢") {
                    Some(End::Turnstile)
                } else if self.rest().is_empty() {
                    Some(End::Stop)
                } else {
                    None
                };
                let Some(end) = end else {
                    let expected: &[&str] = if self.open > 0 {
                        &["a connective", "`)`"]
                    } else if self.dialect == Dialect::Tptp {
                        &["a connective", "the end"]
                    } else {
                        &["a connective", "`,`", "`|-`", "the end"]
                    };
                    return Err(self.unexpected(self.at.max(at), expected));
                };
                return Ok((self.reduce(operand, None)?, end));
            }
        }
    }

    /// Reads a constant or an atom.
    fn constant_or_atom(&mut self) -> Result<NodeId, Error> {
        let at = self.at;
        let native = self.dialect == Dialect::Native;
        if native && self.eat("⊤") {
            return self.formulas.add(Node::True);
        }
        if native && self.eat("⊥") {
            return self.formulas.add(Node::False);
        }
        if !native && self.eat("$true") {
            return self.formulas.add(Node::True);
        }
        if !native && self.eat("$false") {
            return self.formulas.add(Node::False);
        }
        match self.identifier() {
            Some("true") if native => self.formulas.add(Node::True),
            Some("false") if native => self.formulas.add(Node::False),
            Some(name) => self.formulas.atom(name),
            None => Err(self.unexpected(at, &["a formula"])),
        }
    }

    /// Gives `operand` to the pending operators that take it before
    /// `next` does, or to all of them up to the innermost open parenthesis
    /// for `None`, and returns the formula they make of it.
    fn reduce(&mut self, mut operand: NodeId, next: Option<Binary>) -> Result<NodeId, Error> {
        while let Some(&top) = self.pending.last() {
            let node = match top {
                Pending::Open => break,
                Pending::Not => Node::Not(operand),
                Pending::Binary(binary, left) => {
                    if next.is_some_and(|next| !binary.takes_before(next)) {
                        break;
                    }
                    let f = &mut *self.formulas;
                    match binary {
                        Binary::Iff => Node::Iff(left, operand),
                        Binary::Xor => Node::Not(f.add(Node::Iff(left, operand))?),
                        Binary::Implies => Node::Implies(left, operand),
                        Binary::Reverse => Node::Implies(operand, left),
                        Binary::Or => Node::Or(left, operand),
                        Binary::Nor => Node::Not(f.add(Node::Or(left, operand))?),
                        Binary::And => Node::And(left, operand),
                        Binary::Nand => Node::Not(f.add(Node::And(left, operand))?),
                    }
                }
            };
            self.pending.pop();
            operand = self.formulas.add(node)?;
        }
        Ok(operand)
    }
}

impl std::str::FromStr for Sequent {
    type Err = Error;

    /// Parses a sequent of ordinary logic, or one formula to prove, in
    /// the syntax [`Sequent`] documents.
    fn from_str(s: &str) -> Result<Self, Error> {
        let mut formulas = Formulas::default();
        let mut parser = Parser {
            input: s,
            at: 0,
            dialect: Dialect::Native,
            formulas: &mut formulas,
            pending: Vec::new(),
            open: 0,
        };
        let (mut left, mut right) = (Vec::new(), Vec::new());
        let mut turnstile = false;
        // An empty side: the turnstile first, or nothing after it.
        if parser.eat("|-") || parser.eat("⊢") {
            turnstile = true;
        }
        if !(turnstile && parser.rest().is_empty()) {
            loop {
                let (formula, end) = parser.formula()?;
                if turnstile { &mut right } else { &mut left }.push(formula);
                match end {
                    End::Comma => {}
                    End::Turnstile if !turnstile => {
                        turnstile = true;
                        if parser.rest().is_empty() {
                            break;
                        }
                    }
                    End::Turnstile => {
                        // The turnstile's last character: `⊢` is three bytes.
                        let last = parser.input[..parser.at]
                            .char_indices()
                            .next_back()
                            .map_or(0, |(at, _)| at);
                        return Err(parser.unexpected(last, &["a connective", "`,`", "the end"]));
                    }
                    End::Stop => break,
                }
            }
        }
        if !turnstile {
            // One formula, or a list, to prove.
            std::mem::swap(&mut left, &mut right);
        }
        Sequent::new(formulas, left, right)
    }
}

/// A problem of ordinary logic read from a TPTP file.
///
/// Needs the cargo feature `parse` (on by default).
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct Problem {
    /// The axioms left of `⊢`, the conjecture right of it.
    pub sequent: Sequent,
    /// The provability the file's header claims: its `Status (intuit.)`
    /// line, else its plain `Status` line.
    pub status: Option<Status>,
}

/// Reads a problem of propositional logic in TPTP's `fof` syntax, as the
/// ILTP library writes them: every clause with the role `axiom` or
/// `hypothesis` is a hypothesis, the one `conjecture` the formula right
/// of `⊢`. A formula is built with `~`, `&`, `|`, `=>`, `<=`, `<=>`, `<~>`,
/// `~|`, `~&`, `$true` and `$false`, which bind in that order from `~`,
/// the tightest, to the equivalences (TPTP asks for parentheses where
/// this matters, and reads `~` as tight as here). The file's statuses are
/// read as [`lltp::read`](crate::lltp::read) reads them.
///
/// Needs the cargo feature `parse` (on by default).
///
/// # Errors
///
/// [`Error::Tptp`] for a file that is not a sequence of such clauses,
/// [`Error::SeveralConjectures`] for a file with more than one conjecture
/// (TPTP asks for each to be proved, which right of `⊢` would read as
/// their disjunction), and [`Error::Parse`] for a formula that is
/// not one.
pub fn read_tptp(text: &str) -> Result<Problem, Error> {
    let clauses = clauses(text, |message| Error::Tptp { message })?;
    let mut formulas = Formulas::default();
    let mut read = |text: &str| {
        let mut parser = Parser {
            input: text,
            at: 0,
            dialect: Dialect::Tptp,
            formulas: &mut formulas,
            pending: Vec::new(),
            open: 0,
        };
        parser.formula().map(|(formula, _)| formula)
    };
    let left = clauses
        .hypotheses()
        .map(&mut read)
        .collect::<Result<_, _>>()?;
    let right = vec![read(clauses.conjecture())?];
    Ok(Problem {
        sequent: Sequent::new(formulas, left, right)?,
        status: clauses.status,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Both spellings of every connective, the precedences and the
    /// associativities read as documented; a text without a turnstile is
    /// a formula to prove, and the linear connectives are refused.
    #[test]
    fn reads_the_syntax() {
        let read = |text: &str| text.parse::<Sequent>().unwrap().to_string();
        assert_eq!(
            read("a /\\ b \\/ ~c -> d <-> e"),
            "⊢ (((a ∧ b) ∨ ¬c) → d) ↔ e"
        );
        assert_eq!(
            read("a ∧ b ∨ ¬c → d ↔ e"),
            read("a /\\ b \\/ ~c -> d <-> e")
        );
        assert_eq!(read("a -> b -> c"), "⊢ a → (b → c)");
        assert_eq!(read("a /\\ b /\\ c"), "⊢ (a ∧ b) ∧ c");
        assert_eq!(read("true, ⊥ |- false, ⊤"), "⊤, ⊥ ⊢ ⊥, ⊤");
        assert_eq!(read("|-"), "⊢");
        assert_eq!(read("a |-"), "a ⊢");
        assert_eq!(read("~~(a)"), "⊢ ¬¬a");
        for bad in [
            "a & b",
            "a | b",
            "a -o b",
            "a |- b |- c",
            "a ⊢ b ⊢ c",
            "(a",
            "a)",
            "a b",
            "",
        ] {
            assert!(bad.parse::<Sequent>().is_err(), "{bad:?}");
        }
    }

    /// TPTP's connectives, its constants, the clauses' roles and the
    /// status, with `~` as tight as its operand.
    #[test]
    fn reads_tptp() {
        let text = "% Status (intuit.) : Non-Theorem\n\
                    fof(a, axiom, ~ p => (q <= r)).\n\
                    fof(b, hypothesis, (p <~> q) & (p ~| $true) & (q ~& $false)).\n\
                    fof(c, conjecture, ~ ~ p <=> p).\n";
        let problem = read_tptp(text).unwrap();
        assert_eq!(problem.status, Some(Status::NonTheorem));
        assert_eq!(
            problem.sequent.to_string(),
            "¬p → (r → q), (¬(p ↔ q) ∧ ¬(p ∨ ⊤)) ∧ ¬(q ∧ ⊥) ⊢ ¬¬p ↔ p"
        );
        assert!(matches!(
            read_tptp("fof(c, axiom, p)."),
            Err(Error::Tptp { .. })
        ));
        assert!(read_tptp("fof(c, conjecture, p -> q).").is_err());
    }

    /// A second conjecture is refused by name: right of `⊢` the two would
    /// be their disjunction, and `p ⊢ p, q` is valid where `p ⊢ q` is not.
    #[test]
    fn refuses_several_conjectures() {
        let text = "fof(a, axiom, p). fof(c1, conjecture, p). fof(c2, conjecture, q).";
        assert!(matches!(
            read_tptp(text),
            Err(Error::SeveralConjectures { second }) if second == "c2"
        ));
    }
}
