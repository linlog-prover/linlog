// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! A precedence parser that keeps its own stack and writes the terms it
//! reads straight into the arena, so that neither the nesting of a formula
//! nor its length costs any of the caller's stack.
//!
//! The syntax it reads is documented on [`Sequent`], the
//! type a library user meets it on.
//!
//! The arena holds the formulas in negation normal form, one-sided: the
//! formulas left of the turnstile are negated, `A ⊸ B` is `A^⊥ ⅋ B`, and a
//! negation is pushed to the atoms. The parser writes every term as it
//! stands in the text and notes where a negation applies; one pass from
//! the last term to the first then dualises what stands under an odd
//! number of them.

use crate::Error;
use crate::errors::ParseError;
use crate::fragment::Order;
use crate::hash::HashMap;
use crate::limits::{Limits, Refusal, Space};
use crate::occurrences::Forest;
use crate::sequents::name::{RESERVED, normalized, starts_identifier};
use crate::sequents::{Atom, Sequent, Term, TermId};
use std::borrow::Cow;

/// A binary connective as it is written.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Binary {
    /// `*` or `⊗`
    Tensor,
    /// `|`, `par` or `⅋`
    Par,
    /// `&`
    With,
    /// `+` or `⊕`
    Plus,
    /// `-o` or `⊸`
    Lollipop,
    /// `\`, the left division of an ordered parse: `A \ B` is `A⊥ ⅋ B`,
    /// as `A ⊸ B` is.
    Under,
    /// `/`, the right division of an ordered parse: `B / A` is `B ⅋ A⊥`.
    Over,
}

impl Binary {
    /// Returns how tightly the connective binds: of two connectives around
    /// an operand, the one with the larger number takes it.
    const fn strength(self) -> u8 {
        match self {
            Binary::Tensor => 5,
            Binary::Par => 4,
            Binary::With => 3,
            Binary::Plus => 2,
            Binary::Lollipop | Binary::Under | Binary::Over => 1,
        }
    }

    /// Returns whether an operand between this connective on its left and
    /// `next` on its right is this one's: it binds tighter, or the two are
    /// the same connective and it associates to the left, as every one but
    /// `⊸` and `\` does.
    fn takes_before(self, next: Binary) -> bool {
        self.strength() > next.strength()
            || (self == next && !matches!(self, Binary::Lollipop | Binary::Under))
    }
}

/// What has been read and waits for the operand to its right to be
/// complete.
#[derive(Clone, Copy, Debug)]
enum Pending {
    /// `(`, which waits for its `)`.
    Open,
    /// `~`
    Dual,
    /// `!`
    Bang,
    /// `?`
    Quest,
    /// A binary connective with its left operand.
    Binary(Binary, TermId),
}

/// What the parser reads next.
enum State {
    /// Something a formula starts with.
    Operand,
    /// Something that follows the operand just read, which is the term
    /// given.
    Operator(TermId),
    /// Nothing: the sequent is complete.
    End,
}

/// A sequent in the reading: the text and the place in it, the arena so
/// far, and what waits for its right operand.
struct Parser<'a> {
    /// The text of the sequent.
    input: &'a str,
    /// The byte offset of the next character to read.
    at: usize,
    /// Whether the formulas being read stand left of the turnstile.
    left: bool,
    /// Whether nothing of this side of the turnstile has been read yet, so
    /// that the side may still turn out to have no formula.
    empty: bool,
    /// The terms read so far, each as it stands in the text.
    terms: Vec<Term>,
    /// Per term, whether the text negates it an odd number of times: by a
    /// `~` or a `^` on it, as the antecedent of a `⊸`, or as a formula
    /// left of the turnstile. The negations of the terms above it are not
    /// counted.
    negated: Vec<bool>,
    /// The formulas of the sequent, in the order of the text.
    roots: Vec<TermId>,
    /// How many formulas stand left of the turnstile, once it is passed.
    antecedents: u32,
    /// The names of the atoms, in the order they first occur.
    atoms: Vec<String>,
    /// The atom of every name read so far, by its NFC.
    names: HashMap<Cow<'a, str>, Atom>,
    /// The prefix operators, parentheses and binary connectives whose
    /// right operand is being read, the innermost last.
    pending: Vec<Pending>,
    /// How many parentheses are open.
    open: usize,
    /// How many `!` and `?` wait in `pending`: each is a term to come,
    /// counted against the bound when it is read, so that a run of them
    /// is refused before it is reduced.
    owed: u64,
    /// How many `(` and `~` wait in `pending`: they make no term, and are
    /// refused past the bound all the same, so that a run of them cannot
    /// pile up.
    marks: u64,
    /// The most terms the arena may hold: the bound, and never more than
    /// a forest indexes.
    most: u64,
    /// The bound on occurrences the caller set, if any.
    limit: Option<u64>,
    /// The order of the calculus the sequent is read for, which decides
    /// what a negation does to a product and to the antecedents.
    order: Order,
}

/// Reads `text` as a sequent for a calculus of `order`, refused past
/// `limit` terms. The one place a parser is made and run: both readers
/// call it, so the parser is a local of this function and its loop, with
/// what the loop calls, has one caller (a loop each reader inlined cost
/// the commutative one 1.5 % to 7 % of its instructions, as a shared
/// callee lost its inlining).
fn read(text: &str, limit: Option<u64>, order: Order) -> Result<Sequent, Error> {
    Parser::new(text, limit, order).sequent()
}

impl<'a> Parser<'a> {
    /// Returns a parser at the start of `input`, for a sequent of at most
    /// `limit` terms and as many as a forest indexes.
    fn new(input: &'a str, limit: Option<u64>, order: Order) -> Self {
        let most = limit.unwrap_or(Forest::MOST).min(Forest::MOST);
        Self {
            input,
            at: 0,
            left: true,
            empty: true,
            terms: Vec::new(),
            negated: Vec::new(),
            roots: Vec::new(),
            antecedents: 0,
            atoms: Vec::new(),
            names: HashMap::default(),
            pending: Vec::new(),
            open: 0,
            owed: 0,
            marks: 0,
            most,
            limit,
            order,
        }
    }

    /// Returns the next character, or `None` at the end of the input.
    fn peek(&self) -> Option<char> {
        self.input[self.at..].chars().next()
    }

    /// Returns the error for the character at byte `at`, or for the end of
    /// the input there, where `expected` could have stood.
    fn unexpected(&self, at: usize, expected: &'static [&'static str]) -> Error {
        Error::from(ParseError::new(self.input, at, expected))
    }

    /// What can start a formula here: on an empty left side the turnstile
    /// too.
    const fn operands(&self) -> &'static [&'static str] {
        if self.empty && self.left {
            &["a formula", "`|-`"]
        } else {
            &["a formula"]
        }
    }

    /// What can follow an operand here: a connective, and inside a
    /// parenthesis its closing one, outside every parenthesis a comma and
    /// the turnstile on the left side or the end on the right side.
    const fn operators(&self) -> &'static [&'static str] {
        if self.open > 0 {
            &["a connective", "`)`"]
        } else if self.left {
            &["a connective", "`,`", "`|-`"]
        } else {
            &["a connective", "`,`", "the end"]
        }
    }

    /// Appends a term to the arena and returns its index, or fails when
    /// the arena is full.
    fn push(&mut self, term: Term) -> Result<TermId, Error> {
        let index = self.terms.len() as u64;
        if index.saturating_add(self.owed) >= self.most {
            return Err(self.past_bound());
        }
        self.terms.push(term);
        self.negated.push(false);
        Ok(TermId::new(index as u32))
    }

    /// Notes one more negation of a term.
    fn negate(&mut self, term: TermId) {
        self.negated[term.index()] ^= true;
    }

    /// Reads the rest of the identifier that starts at byte `start` and
    /// whose first character has been read, and returns it whole.
    fn identifier(&mut self, start: usize) -> &'a str {
        let rest = &self.input[self.at..];
        let end = rest
            .find(|c| !unicode_ident::is_xid_continue(c))
            .unwrap_or(rest.len());
        self.at += end;
        &self.input[start..self.at]
    }

    /// Returns the atom called `name` in NFC, a new one if the name is.
    /// A name read before is one lookup; only a new spelling is composed.
    fn atom(&mut self, name: &'a str) -> Atom {
        if let Some(&atom) = self.names.get(name) {
            return atom;
        }
        let key = normalized(name);
        if let Cow::Owned(composed) = &key
            && let Some(&atom) = self.names.get(composed.as_str())
        {
            return atom;
        }
        let atom = Atom::new(self.atoms.len() as u32);
        self.atoms.push(key.clone().into_owned());
        self.names.insert(key, atom);
        atom
    }

    /// Gives `operand` to the pending operators that take it before the
    /// connective `next` does, or to all of them up to the innermost open
    /// parenthesis for `None`, the innermost first, and returns the term
    /// they make of it.
    fn reduce(&mut self, mut operand: TermId, next: Option<Binary>) -> Result<TermId, Error> {
        while let Some(&top) = self.pending.last() {
            let term = match top {
                Pending::Open => break,
                Pending::Dual => {
                    self.negate(operand);
                    self.pending.pop();
                    self.marks -= 1;
                    continue;
                }
                // Each pays the term it owed.
                Pending::Bang => {
                    self.owed -= 1;
                    Term::Bang(operand)
                }
                Pending::Quest => {
                    self.owed -= 1;
                    Term::Quest(operand)
                }
                Pending::Binary(connective, left) => {
                    if next.is_some_and(|next| !connective.takes_before(next)) {
                        break;
                    }
                    match connective {
                        Binary::Tensor => Term::Tensor(left, operand),
                        Binary::Par => Term::Par(left, operand),
                        Binary::With => Term::With(left, operand),
                        Binary::Plus => Term::Plus(left, operand),
                        Binary::Lollipop | Binary::Under => {
                            self.negate(left);
                            Term::Par(left, operand)
                        }
                        Binary::Over => {
                            self.negate(operand);
                            Term::Par(left, operand)
                        }
                    }
                }
            };
            self.pending.pop();
            operand = self.push(term)?;
        }
        Ok(operand)
    }

    /// Completes a formula of the sequent, outside every parenthesis, with
    /// its last operand.
    fn root(&mut self, operand: TermId) -> Result<(), Error> {
        let root = self.reduce(operand, None)?;
        if self.left {
            self.negate(root);
        }
        self.roots.push(root);
        Ok(())
    }

    /// Passes the turnstile: what follows is the right side, of which
    /// nothing has been read.
    fn turnstile(&mut self) -> State {
        // Every formula is a term, of which there are fewer than `u32::MAX`.
        self.antecedents = self.roots.len() as u32;
        self.left = false;
        self.empty = true;
        State::Operand
    }

    /// Returns the refusal of a text with more terms than the bound.
    /// Every term of the text is an occurrence of its own, so the text is
    /// refused at the first term past it.
    fn past_bound(&self) -> Error {
        Error::Refused(match self.limit {
            Some(limit) if limit < Forest::MOST => Refusal::Occurrences {
                occurrences: limit.saturating_add(1),
                limit,
            },
            _ => Refusal::Index {
                what: Space::Occurrence,
                count: Forest::MOST.saturating_add(1),
                most: Forest::MOST,
            },
        })
    }

    /// Notes an operator that stands before its operand, or an open
    /// parenthesis. Two `~` in a row cancel, so a run of them waits as
    /// one at most; a `!` or `?` is counted as the term it will be, and a
    /// `(` or `~` as a mark, of which no more than the bound may wait.
    fn prefix(&mut self, operator: Pending) -> Result<State, Error> {
        self.empty = false;
        match operator {
            Pending::Dual if matches!(self.pending.last(), Some(Pending::Dual)) => {
                self.pending.pop();
                self.marks -= 1;
                return Ok(State::Operand);
            }
            Pending::Bang | Pending::Quest => {
                self.owed += 1;
                if (self.terms.len() as u64).saturating_add(self.owed) > self.most {
                    return Err(self.past_bound());
                }
            }
            Pending::Open | Pending::Dual => {
                self.marks += 1;
                if self.marks > self.most {
                    return Err(self.past_bound());
                }
            }
            Pending::Binary(..) => {}
        }
        self.pending.push(operator);
        Ok(State::Operand)
    }

    /// Reads what a formula starts with: a prefix operator, an open
    /// parenthesis, a constant or a variable. An empty left side may have
    /// the turnstile there and an empty right side the end of the input.
    fn operand(&mut self) -> Result<State, Error> {
        let start = self.at;
        let Some(c) = self.peek() else {
            return if self.empty && !self.left {
                Ok(State::End)
            } else {
                Err(self.unexpected(start, self.operands()))
            };
        };
        self.at += c.len_utf8();
        let term = match c {
            '~' => return self.prefix(Pending::Dual),
            '!' => return self.prefix(Pending::Bang),
            '?' => return self.prefix(Pending::Quest),
            '(' => {
                self.open += 1;
                return self.prefix(Pending::Open);
            }
            '0' => Term::Zero,
            '1' => Term::One,
            '⊥' => Term::Bot,
            '⊤' => Term::Top,
            '⊢' if self.empty && self.left => return Ok(self.turnstile()),
            '|' if self.empty && self.left => {
                // Only `|-` can start a sequent with `|`, so what is wrong
                // is the character after it.
                if self.peek() != Some('-') {
                    return Err(self.unexpected(self.at, &["the `-` of `|-`"]));
                }
                self.at += 1;
                return Ok(self.turnstile());
            }
            _ if starts_identifier(c) => match self.identifier(start) {
                "bot" => Term::Bot,
                "top" => Term::Top,
                // The connective, where only a formula can stand.
                "par" => {
                    let error = ParseError::spanning(self.input, start..self.at, self.operands());
                    return Err(error.into());
                }
                word if RESERVED.contains(&word) => {
                    let error = ParseError::reserved(self.input, start..self.at, self.operands());
                    return Err(error.into());
                }
                name => Term::Atom(self.atom(name)),
            },
            _ => return Err(self.unexpected(start, self.operands())),
        };
        self.empty = false;
        Ok(State::Operator(self.push(term)?))
    }

    /// Reads what follows the operand just read: a postfix `^`, a binary
    /// connective, a closing parenthesis, or outside every parenthesis a
    /// comma, the turnstile on the left side and the end of the input on
    /// the right side.
    fn operator(&mut self, operand: TermId) -> Result<State, Error> {
        let start = self.at;
        let outermost = self.open == 0;
        let Some(c) = self.peek() else {
            if self.left || !outermost {
                return Err(self.unexpected(start, self.operators()));
            }
            self.root(operand)?;
            return Ok(State::End);
        };
        self.at += c.len_utf8();
        let connective = match c {
            '^' => {
                self.negate(operand);
                return Ok(State::Operator(operand));
            }
            '*' | '⊗' => Binary::Tensor,
            '⅋' => Binary::Par,
            '&' => Binary::With,
            '+' | '⊕' => Binary::Plus,
            '⊸' => Binary::Lollipop,
            '-' => {
                // Only `-o` starts with `-`, so what is wrong is the
                // character after it.
                if self.peek() != Some('o') {
                    return Err(self.unexpected(self.at, &["the `o` of `-o`"]));
                }
                self.at += 1;
                Binary::Lollipop
            }
            // The divisions of the Lambek calculus, in an ordered parse.
            '\\' if self.order != Order::Commutative => Binary::Under,
            '/' if self.order != Order::Commutative => Binary::Over,
            '|' if self.left && outermost && self.peek() == Some('-') => {
                self.at += 1;
                self.root(operand)?;
                return Ok(self.turnstile());
            }
            // Where no turnstile can stand, `|-` is a `|` before a `-`,
            // which starts no formula and is reported as that.
            '|' => Binary::Par,
            '⊢' if self.left && outermost => {
                self.root(operand)?;
                return Ok(self.turnstile());
            }
            ',' if outermost => {
                self.root(operand)?;
                return Ok(State::Operand);
            }
            ')' if !outermost => {
                let inner = self.reduce(operand, None)?;
                // What stopped the reduction is the open parenthesis.
                self.pending.pop();
                self.open -= 1;
                self.marks -= 1;
                return Ok(State::Operator(inner));
            }
            _ if starts_identifier(c) && self.identifier(start) == "par" => Binary::Par,
            _ => return Err(self.unexpected(start, self.operators())),
        };
        let left = self.reduce(operand, Some(connective))?;
        self.pending.push(Pending::Binary(connective, left));
        Ok(State::Operand)
    }

    /// Reads the whole input as a sequent.
    fn sequent(mut self) -> Result<Sequent, Error> {
        let mut state = State::Operand;
        loop {
            let blank = &self.input[self.at..];
            self.at += blank.len() - blank.trim_start().len();
            state = match state {
                State::Operand => self.operand()?,
                State::Operator(operand) => self.operator(operand)?,
                State::End => return self.finish(),
            };
        }
    }

    /// Dualises every term that stands under an odd number of negations,
    /// reversing the operands of a product when `ORDERED`.
    fn dualise<const ORDERED: bool>(&mut self) {
        // A term comes after its subterms, so one pass from the last term
        // sees every negation of a term before it reaches the term.
        for n in (0..self.terms.len()).rev() {
            if self.negated[n] {
                let mut term = self.terms[n].dual();
                if ORDERED {
                    term = match term {
                        Term::Tensor(l, r) => Term::Tensor(r, l),
                        Term::Par(l, r) => Term::Par(r, l),
                        other => other,
                    };
                }
                self.terms[n] = term;
                for k in term.subterms() {
                    self.negated[k.index()] ^= true;
                }
            }
        }
    }

    /// Applies the negations noted and returns the sequent, optimized. In
    /// the cyclic order the dual of a product reverses it, `(A ⊗ B)⊥ =
    /// B⊥ ⅋ A⊥`, and the antecedents `A₁, …, Aₙ ⊢` become `Aₙ⊥, …, A₁⊥`,
    /// the dual of their product.
    fn finish(mut self) -> Result<Sequent, Error> {
        if self.order == Order::Commutative {
            self.dualise::<false>();
        } else {
            self.dualise::<true>();
            self.roots[..self.antecedents as usize].reverse();
        }
        let mut sequent = Sequent {
            terms: self.terms,
            roots: self.roots,
            atoms: self.atoms,
            antecedents: Some(self.antecedents),
        };
        sequent.optimize();
        Ok(sequent)
    }
}

impl Sequent {
    /// Parses a two-sided sequent such as `A, B |- A * B` into a one-sided
    /// one, optimized, refusing it at the first term past
    /// `limits.occurrences`, before anything of that size is built. The
    /// nesting of its formulas may be of any depth.
    ///
    /// Needs the cargo feature `parse` (on by default).
    ///
    /// # Errors
    ///
    /// [`Error::Parse`] for text that is no sequent, [`Refusal::Occurrences`]
    /// past the bound, and [`Refusal::Index`] for more terms than a forest
    /// indexes, whatever the bound.
    pub fn parse_within(text: &str, limits: &Limits) -> Result<Self, Error> {
        read(text, limits.occurrences, Order::Commutative)
    }

    /// Parses a sequent as [`parse_within`](Self::parse_within) does, for
    /// a calculus that keeps `order`: in [`Order::Cyclic`] the dual of a
    /// product reverses it, `~(A * B)` reading as `~B par ~A`, and the
    /// formulas left of the turnstile come in the reverse order, each
    /// negated, so that `A, B |- C` is `|- ~B, ~A, C`, the cyclic sequent
    /// whose proofs are those of the two-sided one. The formulas then
    /// stand in the order the cyclic calculus reads, which is the
    /// occurrence forest's. An ordered parse also reads the two divisions
    /// of the Lambek calculus, `A \ B` as `A -o B` (associating to the
    /// right) and `B / A` as `B par ~A` (to the left), both binding as
    /// loosely as `-o`. In [`Order::Commutative`] it is
    /// [`parse_within`](Self::parse_within).
    ///
    /// Needs the cargo feature `parse` (on by default).
    ///
    /// # Errors
    ///
    /// Those of [`parse_within`](Self::parse_within).
    ///
    /// # Examples
    ///
    /// ```
    /// use linlog::{Limits, Order, Sequent};
    ///
    /// let cyclic = Sequent::parse_ordered("a * b |- a * b", Order::Cyclic, &Limits::default())?;
    /// assert_eq!(cyclic.to_string(), "⊢ ~b ⅋ ~a, a ⊗ b");
    /// let commutative: Sequent = "a * b |- a * b".parse()?;
    /// assert_eq!(commutative.to_string(), "⊢ ~a ⅋ ~b, a ⊗ b");
    /// # Ok::<(), linlog::Error>(())
    /// ```
    pub fn parse_ordered(text: &str, order: Order, limits: &Limits) -> Result<Self, Error> {
        read(text, limits.occurrences, order)
    }
}

/// What intuitionistic linear logic writes instead of a symbol of the
/// classical syntax.
const NEGATION: &str = "intuitionistic linear logic has no negation: write a negated formula \
                        on the other side of |- (A |- B, not |- ~A, B), and ~A par B as A -o B";
/// See [`NEGATION`].
const PAR: &str = "intuitionistic linear logic has no par: write ~A par B as A -o B";
/// See [`NEGATION`].
const QUEST: &str = "intuitionistic linear logic has no ?: write ?~A right of |- as !A left of it";
/// See [`NEGATION`].
const BOT: &str = "intuitionistic linear logic has no bot: write bot right of |- as 1 left of it";

/// Returns the error for the first symbol of `text` that classical linear
/// logic has and intuitionistic linear logic does not (`~`, `^`, `⅋`,
/// `par`, `|` as a connective, `?`, `⊥`, `bot`), with its hint, or `None`
/// if it has none. The symbols are whole tokens wherever they stand, so a
/// scan of the characters and identifiers finds them without the parser:
/// `|` followed by `-` is the turnstile, and an identifier is skipped
/// whole, so that `bottom` is an atom.
pub(crate) fn classical(text: &str) -> Option<ParseError> {
    let mut chars = text.char_indices().peekable();
    while let Some((at, c)) = chars.next() {
        let mut end = at + c.len_utf8();
        let hint = match c {
            '~' | '^' => NEGATION,
            '⅋' => PAR,
            '|' if chars.peek().is_some_and(|&(_, next)| next == '-') => continue,
            '|' => PAR,
            '?' => QUEST,
            '⊥' => BOT,
            _ if starts_identifier(c) => {
                while let Some(&(next, d)) = chars.peek() {
                    if !unicode_ident::is_xid_continue(d) {
                        break;
                    }
                    end = next + d.len_utf8();
                    chars.next();
                }
                match &text[at..end] {
                    "par" => PAR,
                    "bot" => BOT,
                    _ => continue,
                }
            }
            _ => continue,
        };
        return Some(ParseError::spanning(text, at..end, &[]).with_hint(hint));
    }
    None
}

impl std::str::FromStr for Sequent {
    type Err = Error;

    /// Parses a sequent as [`Sequent::parse_within`] does, within the
    /// default [`Limits`].
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse_within(s, &Limits::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A sequent of more terms than the arena may hold is refused, and one
    /// that fills it is read.
    #[test]
    fn refuses_more_terms_than_the_arena_holds() {
        let input = "A |- !A * B";
        assert_eq!(
            Parser::new(input, Some(5), Order::Commutative)
                .sequent()
                .unwrap()
                .terms()
                .len(),
            5
        );
        assert!(matches!(
            Parser::new(input, Some(4), Order::Commutative).sequent(),
            Err(Error::Refused(Refusal::Occurrences {
                occurrences: 5,
                limit: 4
            }))
        ));
    }
}
