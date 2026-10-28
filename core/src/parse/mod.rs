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
use crate::occurrences::reading::Side;
use crate::occurrences::written::Written;
use crate::sequents::fo::{self, FoTerm, FoTermId, Interner, SymbolFault};
use crate::sequents::name::{KEYWORDS, RESERVED, normalized, starts_identifier};
use crate::sequents::{Atom, Sequent, Term, TermId};
use std::borrow::Cow;

/// What a symbol used with another number of arguments than before is
/// told.
const ARITY: &str = "a symbol takes the same number of arguments everywhere: \
                     this use differs from the first";
/// What a name used as a predicate and as a function symbol is told.
const ROLE: &str = "a name is a predicate or a function symbol, not both";
/// What a keyword in term position is told.
const KEYWORD: &str = "a keyword names no variable, constant or function symbol";

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
    /// `∀x.` or `∃x.`, whose variable's name is the innermost of the
    /// parser's open scope.
    Binder(bool),
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
    /// Whether the reading records how every term was written.
    recording: bool,
    /// While recording, the terms written as `⊸`, in ascending order.
    implications: Vec<TermId>,
    /// The names of the binders whose body is being read, the innermost
    /// last: what an identifier in term position is resolved against.
    scope: Vec<&'a str>,
    /// The symbols, terms and argument lists read so far, interned; empty
    /// until the text is first-order.
    interner: Interner,
    /// Whether the text is first-order so far: an atom with arguments or a
    /// binder has been read.
    first_order: bool,
    /// Per atom, its predicate symbol and arguments, once the text is
    /// first-order; empty before, so that a propositional text pays
    /// nothing.
    predicates: Vec<(fo::Symbol, fo::Args)>,
    /// The binders read, each with its variable's name, in the order of
    /// their terms.
    binders: Vec<(TermId, String)>,
}

/// Reads `text` as a sequent for a calculus of `order`, refused past
/// `limit` terms or past `terms` first-order terms in its atoms
/// ([`Sequent::fo_size`]), and where `written` is given fills it with how every
/// occurrence of the sequent's forest was written. The one place a parser
/// is made and run: every reader calls it, so the parser is a local of
/// this function and its loop, with what the loop calls, has one caller
/// (a loop each reader inlined cost the commutative one 1.5 % to 7 % of
/// its instructions, as a shared callee lost its inlining).
fn read(
    text: &str,
    limit: Option<u64>,
    terms: Option<u64>,
    order: Order,
    written: Option<&mut Vec<Written>>,
) -> Result<Sequent, Error> {
    let mut parser = Parser::new(text, limit, order);
    parser.recording = written.is_some();
    let sequent = parser.sequent(written)?;
    if sequent.fo.atoms.is_empty() {
        return Ok(sequent);
    }
    first_order_within(sequent, terms)
}

/// Returns a first-order `sequent`, or the refusal of one whose atoms
/// unfold to more than `terms` terms. A text writes every term it holds,
/// so this bounds no more than its length does; it is the bound every
/// reader applies.
#[cold]
#[inline(never)]
fn first_order_within(sequent: Sequent, terms: Option<u64>) -> Result<Sequent, Error> {
    match terms {
        Some(limit) if sequent.fo_size() > limit => Err(Error::Refused(Refusal::Terms {
            terms: sequent.fo_size(),
            limit,
        })),
        _ => Ok(sequent),
    }
}

/// Reads `text` as a sequent of the commutative calculus within `limits`,
/// as [`Sequent::parse_within`] does, and returns it with how every
/// occurrence of its forest was written, by occurrence id.
pub(crate) fn written(text: &str, limits: &Limits) -> Result<(Sequent, Vec<Written>), Error> {
    let mut writing = Vec::new();
    let sequent = read(
        text,
        limits.occurrences,
        limits.terms,
        Order::Commutative,
        Some(&mut writing),
    )?;
    Ok((sequent, writing))
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
            recording: false,
            implications: Vec::new(),
            scope: Vec::new(),
            interner: Interner::new(),
            first_order: false,
            predicates: Vec::new(),
            binders: Vec::new(),
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

    /// Returns the atom called `name` in NFC, a new one if the name is:
    /// one lookup, which inserts the name where it is new.
    fn atom(&mut self, name: &'a str) -> Atom {
        use std::collections::hash_map::Entry;
        let next = Atom::new(self.atoms.len() as u32);
        match self.names.entry(normalized(name)) {
            Entry::Occupied(known) => *known.get(),
            Entry::Vacant(new) => {
                self.atoms.push(new.key().clone().into_owned());
                new.insert(next);
                next
            }
        }
    }

    /// Gives `atom`, read as `name`, its predicate in a first-order text
    /// where it is new: the nullary symbol of its name.
    #[cold]
    #[inline(never)]
    fn nullary(&mut self, atom: Atom, name: &'a str) -> Result<(), Error> {
        if atom.index() < self.predicates.len() {
            return Ok(());
        }
        let start = self.at - name.len();
        let key = self.atoms[atom.index()].clone();
        let symbol = self.symbol(&key, 0, true, start..self.at)?;
        let empty = self.interner.list(&[]);
        self.predicates.push((symbol, empty));
        Ok(())
    }

    /// Makes the text first-order, if it was not: every atom read so far
    /// gets its predicate, the nullary symbol of its name.
    #[cold]
    #[inline(never)]
    fn enter_first_order(&mut self, span: std::ops::Range<usize>) -> Result<(), Error> {
        if std::mem::replace(&mut self.first_order, true) || self.atoms.is_empty() {
            return Ok(());
        }
        let empty = self.interner.list(&[]);
        for a in 0..self.atoms.len() {
            let name = std::mem::take(&mut self.atoms[a]);
            let symbol = self.symbol(&name, 0, true, span.clone());
            self.atoms[a] = name;
            self.predicates.push((symbol?, empty));
        }
        Ok(())
    }

    /// Returns the symbol called `name` of `arity` in the role `predicate`
    /// says, interned, or the error at `span` for a name known with
    /// another arity or in the other role.
    #[cold]
    #[inline(never)]
    fn symbol(
        &mut self,
        name: &str,
        arity: u32,
        predicate: bool,
        span: std::ops::Range<usize>,
    ) -> Result<fo::Symbol, Error> {
        self.interner
            .symbol(name, arity, predicate)
            .map_err(|fault| {
                let hint = match fault {
                    SymbolFault::Arity { .. } => ARITY,
                    SymbolFault::Role | SymbolFault::Repeated => ROLE,
                };
                ParseError::spanning(self.input, span, &[])
                    .with_hint(hint)
                    .into()
            })
    }

    /// Reads the variables and the dot of a binder whose keyword ends at
    /// the current place, `x y.` of `forall x y.`, and opens a binder per
    /// variable, the first outermost.
    #[cold]
    #[inline(never)]
    fn binder(&mut self, universal: bool) -> Result<State, Error> {
        let keyword = self.at;
        // Before the variables open their scope, so that every atom read
        // so far gets its predicate.
        self.enter_first_order(keyword..self.at)?;
        let mut names = 0;
        loop {
            self.skip_blank();
            let start = self.at;
            let Some(c) = self.peek() else {
                return Err(self.unexpected(start, &["a variable name"]));
            };
            if c == '.' && names > 0 {
                self.at += 1;
                break;
            }
            if !starts_identifier(c) {
                let expected: &'static [&'static str] = if names > 0 {
                    &["a variable name", "`.`"]
                } else {
                    &["a variable name"]
                };
                return Err(self.unexpected(start, expected));
            }
            self.at += c.len_utf8();
            let name = self.identifier(start);
            if KEYWORDS.contains(&name) || RESERVED.contains(&name) {
                let error = ParseError::spanning(self.input, start..self.at, &["a variable name"]);
                return Err(error.with_hint(KEYWORD).into());
            }
            self.scope.push(name);
            names += 1;
        }
        for _ in 0..names {
            self.prefix(Pending::Binder(universal))?;
        }
        Ok(State::Operand)
    }

    /// Skips the whitespace at the current place.
    fn skip_blank(&mut self) {
        let blank = &self.input[self.at..];
        self.at += blank.len() - blank.trim_start().len();
    }

    /// Reads the argument list of the predicate `name`, whose `(` has been
    /// read, up to its `)`, and returns the predicate's atom: an
    /// identifier bound by an open binder is that variable, any other a
    /// constant or, before a `(`, a function symbol. Nested applications
    /// are read on an explicit stack: no term costs the caller's stack.
    #[cold]
    #[inline(never)]
    fn predicate(&mut self, name: &'a str, start: usize) -> Result<Atom, Error> {
        self.enter_first_order(start..self.at)?;
        // The applications being read, each its symbol, where it starts
        // and its arguments so far; the predicate's own list is the first.
        let mut open: Vec<(Cow<'a, str>, usize, Vec<FoTermId>)> =
            vec![(normalized(name), start, Vec::new())];
        loop {
            // A term.
            self.skip_blank();
            let at = self.at;
            match self.peek() {
                Some(c) if starts_identifier(c) => {
                    self.at += c.len_utf8();
                }
                _ => return Err(self.unexpected(at, &["a term"])),
            }
            let word = self.identifier(at);
            if KEYWORDS.contains(&word) || RESERVED.contains(&word) {
                let error = ParseError::spanning(self.input, at..self.at, &["a term"]);
                return Err(error.with_hint(KEYWORD).into());
            }
            let word = normalized(word);
            self.skip_blank();
            if self.peek() == Some('(') {
                self.at += 1;
                open.push((word, at, Vec::new()));
                continue;
            }
            let mut term = match self.scope.iter().rev().position(|&v| *v == *word) {
                Some(index) => FoTerm::Bound(index as u32),
                None => {
                    let symbol = self.symbol(&word, 0, false, at..self.at)?;
                    FoTerm::App(symbol, self.interner.list(&[]))
                }
            };
            // The term closes every application whose last argument it is.
            loop {
                let id = self.interner.term(term);
                let list = &mut open.last_mut().expect("the predicate's list is open").2;
                list.push(id);
                self.skip_blank();
                let at = self.at;
                match self.peek() {
                    Some(',') => {
                        self.at += 1;
                        break;
                    }
                    Some(')') => self.at += 1,
                    _ => return Err(self.unexpected(at, &["`,`", "`)`"])),
                }
                let (symbol, from, items) = open.pop().expect("an application was open");
                let arity = u32::try_from(items.len()).map_err(|_| self.past_bound())?;
                let args = self.interner.list(&items);
                if open.is_empty() {
                    let symbol = self.symbol(&symbol, arity, true, from..self.at)?;
                    let key = self.interner.tables.atom_key(symbol, args);
                    let atom = match self.names.get(key.as_str()) {
                        Some(&atom) => atom,
                        None => {
                            let atom = Atom::new(self.atoms.len() as u32);
                            self.atoms.push(key.clone());
                            self.names.insert(Cow::Owned(key), atom);
                            self.predicates.push((symbol, args));
                            atom
                        }
                    };
                    return Ok(atom);
                }
                let symbol = self.symbol(&symbol, arity, false, from..self.at)?;
                term = FoTerm::App(symbol, args);
            }
        }
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
                Pending::Binder(universal) => {
                    // A binder binds loosest: a connective to its right is
                    // inside its scope.
                    if next.is_some() {
                        break;
                    }
                    self.close_binder(universal, operand)
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
                            if self.recording {
                                // The index the term is pushed at below.
                                self.implications.push(TermId::new(self.terms.len() as u32));
                            }
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

    /// Closes the innermost binder over `operand`, its variable's name
    /// recorded for the term about to be pushed, and returns that term.
    #[cold]
    #[inline(never)]
    fn close_binder(&mut self, universal: bool, operand: TermId) -> Term {
        self.owed -= 1;
        let name = self.scope.pop().expect("a binder has its variable");
        self.binders
            .push((TermId::new(self.terms.len() as u32), name.to_owned()));
        if universal {
            Term::Forall(operand)
        } else {
            Term::Exists(operand)
        }
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
            Pending::Bang | Pending::Quest | Pending::Binder(_) => {
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
            '∀' => return self.binder(true),
            '∃' => return self.binder(false),
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
                "forall" => return self.binder(true),
                "exists" => return self.binder(false),
                name if self.input.as_bytes().get(self.at) == Some(&b'(') => {
                    self.at += 1;
                    Term::Atom(self.predicate(name, start)?)
                }
                name => {
                    let atom = self.atom(name);
                    if self.first_order {
                        self.nullary(atom, name)?;
                    }
                    Term::Atom(atom)
                }
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

    /// Reads the whole input as a sequent, and where `written` is given
    /// records into it how every occurrence was written.
    fn sequent(mut self, written: Option<&mut Vec<Written>>) -> Result<Sequent, Error> {
        let mut state = State::Operand;
        loop {
            let blank = &self.input[self.at..];
            self.at += blank.len() - blank.trim_start().len();
            state = match state {
                State::Operand => self.operand()?,
                State::Operator(operand) => self.operator(operand)?,
                State::End => return self.finish(written),
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

    /// Returns how every occurrence of the sequent's forest was written,
    /// by occurrence id, once the negations are applied: a term the
    /// negations dualised stands on the input side, which is what
    /// `negated` then holds of it. Before `optimize` every term has one
    /// parent, so the terms in preorder from the roots in order are the
    /// forest's occurrences in its numbering.
    fn written(&self) -> Vec<Written> {
        let mut implication = vec![false; self.terms.len()];
        for &term in &self.implications {
            implication[term.index()] = true;
        }
        let mut writing = Vec::with_capacity(self.terms.len());
        let mut stack = Vec::new();
        for &root in &self.roots {
            stack.push(root);
            while let Some(term) = stack.pop() {
                let side = if self.negated[term.index()] {
                    Side::Input
                } else {
                    Side::Output
                };
                writing.push(Written::new(side, implication[term.index()]));
                let (left, right) = self.terms[term.index()].operands();
                stack.extend(right);
                stack.extend(left);
            }
        }
        writing
    }

    /// Applies the negations noted and returns the sequent, optimized,
    /// and where `written` is given records into it how every occurrence
    /// was written. In the cyclic order the dual of a product reverses it,
    /// `(A ⊗ B)⊥ = B⊥ ⅋ A⊥`, and the antecedents `A₁, …, Aₙ ⊢` become
    /// `Aₙ⊥, …, A₁⊥`, the dual of their product.
    fn finish(mut self, written: Option<&mut Vec<Written>>) -> Result<Sequent, Error> {
        if self.order == Order::Commutative {
            self.dualise::<false>();
        } else {
            self.dualise::<true>();
            self.roots[..self.antecedents as usize].reverse();
        }
        if let Some(written) = written {
            *written = self.written();
        }
        let mut fo = self.interner.tables;
        fo.atoms = self.predicates;
        fo.binders = self.binders;
        let mut sequent = Sequent {
            terms: self.terms,
            roots: self.roots,
            atoms: self.atoms,
            antecedents: Some(self.antecedents),
            cuts: vec![],
            fo,
        };
        debug_assert!(sequent.check().is_ok(), "{:?}", sequent.check());
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
        read(
            text,
            limits.occurrences,
            limits.terms,
            Order::Commutative,
            None,
        )
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
        read(text, limits.occurrences, limits.terms, order, None)
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
/// `par`, `|` as a connective, `?`, `⊥`, `bot`), with its hint, if one
/// starts before byte `before`, else `None`. The symbols are whole tokens
/// wherever they stand, so a scan finds them without the parser's state,
/// cutting the text into tokens where the parser does: `|-` is the
/// turnstile and `-o` the implication, each one token, and an identifier
/// is read whole from where one starts, so that `bottom` and the `obot`
/// of `-obot`'s neighbour `|-obot` are atoms and the `bot` of `-obot` is
/// the constant. The text is scanned whole, so that a token is read as
/// the parser reads it whatever follows the place.
pub(crate) fn classical(text: &str, before: usize) -> Option<ParseError> {
    let mut chars = text.char_indices().peekable();
    while let Some((at, c)) = chars.next() {
        if at >= before {
            return None;
        }
        let mut end = at + c.len_utf8();
        let mut next_is = |d: char| {
            let is = chars.peek().is_some_and(|&(_, next)| next == d);
            if is {
                chars.next();
            }
            is
        };
        let hint = match c {
            '~' | '^' => NEGATION,
            '⅋' => PAR,
            // The turnstile and the implication, read as single tokens.
            '|' if next_is('-') => continue,
            '-' if next_is('o') => continue,
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
                .sequent(None)
                .unwrap()
                .terms()
                .len(),
            5
        );
        assert!(matches!(
            Parser::new(input, Some(4), Order::Commutative).sequent(None),
            Err(Error::Refused(Refusal::Occurrences {
                occurrences: 5,
                limit: 4
            }))
        ));
    }
}
