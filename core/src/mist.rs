// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Coverability problems in the `.spec` format of the Mist tool
//! (`github.com/pierreganty/mist`), in which the coverability suites of
//! software verification are written (the `qcover` collection's, for
//! one): counters, rules that test and change them, an initial marking,
//! and the markings to cover. A problem is a Petri net, and is read as a
//! Horn program to decide in affine mode, where weakening lets a marking
//! that covers the goal stand for it: each counter an atom, each rule a
//! clause under `!` from the tokens it needs to those it leaves, the
//! initial marking's tokens on the left of `⊢`, and the target the goal.
//!
//! ```text
//! vars    x y
//! rules   x >= 1 -> x' = x - 1, y' = y + 2;
//! init    x = 1, y >= 0
//! target  y >= 2
//! ```
//!
//! is `!(x -o y * y), !y, x ⊢ y * y`: an initial count written `x >= k`
//! is `k` tokens and any number more, `!x`. A target of several lines is
//! covered when one of them is: each line is a clause `!(… -o goal)` to a
//! fresh atom, the goal.
//!
//! # Examples
//!
//! ```
//! use linlog::mist::{Safety, read};
//!
//! let text = "#expected result: unsafe\n\
//!             vars x y\n\
//!             rules x >= 1 -> x' = x - 1, y' = y + 2;\n\
//!             init x = 1, y >= 0\n\
//!             target y >= 2\n";
//! let problem = read(text)?;
//! assert_eq!(problem.expected, Some(Safety::Unsafe));
//! assert_eq!(problem.sequent, "!(x -o y * y), !y, x |- y * y".parse()?);
//! # Ok::<(), linlog::Error>(())
//! ```
//!
//! Needs the cargo feature `parse` (on by default).

use crate::{Error, Sequent};
use std::collections::HashMap;
use std::fmt::Write as _;

/// A problem read from a `.spec` file.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct Problem {
    /// The problem as a Horn program, for affine mode: the rules under
    /// `!`, the initial marking left of `⊢`, the target right of it.
    pub sequent: Sequent,
    /// The result the file's first line states, `#expected result: safe`
    /// or `unsafe`, if it states one.
    pub expected: Option<Safety>,
}

/// The result of a coverability problem.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Safety {
    /// No marking reachable from the initial one covers the target: the
    /// sequent is unprovable in affine mode.
    Safe,
    /// Some reachable marking covers the target: the sequent is provable
    /// in affine mode.
    Unsafe,
}

/// Reads a `.spec` problem: the sections `vars` (the counters' names),
/// `rules` (each `guards -> updates;`, a guard `x >= k`, an update
/// `x' = x + k` or `x' = x - k`), `init` (`x = k` or `x >= k`, a counter
/// not named starting at zero) and `target` (lines of `x >= k`, a line a
/// conjunction, the lines a disjunction), in that order, and an
/// `invariants` section after them, which is skipped; from `#` to the end
/// of a line is a comment. A rule takes from each counter the most of its
/// guard and its decrement, and gives back that less the decrement plus
/// the increment.
///
/// # Errors
///
/// [`Error::Mist`] for text that is not such a problem: a section missing
/// or out of order, a counter not declared, a counter updated twice by one
/// rule or from another counter, a count that is no number below 2³², or
/// a counter named `top` or `bot`, which this crate's syntax reads as a
/// unit; [`Error::SequentParsing`] for a name its parser rejects.
pub fn read(text: &str) -> Result<Problem, Error> {
    let expected = text.lines().next().and_then(|line| {
        let line = line.trim().strip_prefix('#')?.trim();
        match line.strip_prefix("expected result:")?.trim() {
            "safe" => Some(Safety::Safe),
            "unsafe" => Some(Safety::Unsafe),
            _ => None,
        }
    });
    let tokens = tokens(text)?;
    let mut reader = Reader { tokens, at: 0 };
    reader.keyword("vars")?;
    let mut names = Vec::new();
    while !reader.at_keyword("rules") {
        let name = reader.name()?;
        if name == "top" || name == "bot" {
            return Err(error(format!(
                "a counter named `{name}`, which this crate's syntax reads as a unit"
            )));
        }
        names.push(name);
    }
    let index: HashMap<&str, usize> = names.iter().enumerate().map(|(i, &n)| (n, i)).collect();
    let counter = |name: &str| {
        index
            .get(name)
            .copied()
            .ok_or_else(|| error(format!("the counter `{name}` is not declared")))
    };
    reader.keyword("rules")?;
    let mut clauses = Vec::new();
    while !reader.at_keyword("init") {
        clauses.push(reader.rule(&names, &counter)?);
    }
    reader.keyword("init")?;
    let mut tokens = vec![0u32; names.len()];
    let mut parameters = Vec::new();
    while !reader.at_keyword("target") {
        let name = reader.name()?;
        let x = counter(name)?;
        let at_least = match reader.next()? {
            Token::Word("=") => false,
            Token::Word(">=") => true,
            token => return Err(error(format!("expected `=` or `>=` at {token}"))),
        };
        tokens[x] = reader.number()?;
        if at_least {
            parameters.push(x);
        }
        reader.comma_or_line();
    }
    reader.keyword("target")?;
    let mut targets = Vec::new();
    let mut target = Vec::new();
    while !reader.at_keyword("invariants") && reader.at < reader.tokens.len() {
        let name = reader.name()?;
        let x = counter(name)?;
        reader.word(">=")?;
        target.push((x, reader.number()?));
        if !reader.comma_or_line() {
            targets.push(merged(std::mem::take(&mut target)));
        }
    }
    if !target.is_empty() {
        targets.push(merged(target));
    }
    if targets.is_empty() {
        return Err(error("no target".to_owned()));
    }

    let mut sequent = String::new();
    for (inputs, outputs) in &clauses {
        let _ = write!(
            sequent,
            "!({} -o {}), ",
            tensor(&names, inputs),
            tensor(&names, outputs)
        );
    }
    let goal = if let [target] = &targets[..] {
        tensor(&names, target)
    } else {
        let goal = (0..)
            .map(|i| {
                if i == 0 {
                    "goal".to_owned()
                } else {
                    format!("goal_{i}")
                }
            })
            .find(|name| !index.contains_key(name.as_str()))
            .expect("some name is fresh");
        for target in &targets {
            let _ = write!(sequent, "!({} -o {goal}), ", tensor(&names, target));
        }
        goal
    };
    for &x in &parameters {
        let _ = write!(sequent, "!{}, ", names[x]);
    }
    for (x, &k) in tokens.iter().enumerate() {
        for _ in 0..k {
            let _ = write!(sequent, "{}, ", names[x]);
        }
    }
    let sequent = format!("{} |- {goal}", sequent.trim_end_matches(", "));
    Ok(Problem {
        sequent: sequent.parse()?,
        expected,
    })
}

/// The error of a text that is no `.spec` problem.
fn error(message: String) -> Error {
    Error::Mist(message)
}

/// Counts of some counters, each a counter's index and its count, sorted
/// by counter: a rule names a few of the tens of thousands of counters a
/// file may declare.
type Counts = Vec<(usize, u32)>;

/// Sorts counts by counter, a counter named twice keeping its larger
/// count, and drops zeros.
fn merged(mut counts: Counts) -> Counts {
    counts.sort_unstable();
    let mut merged: Counts = Vec::with_capacity(counts.len());
    for (x, k) in counts {
        match merged.last_mut() {
            Some((y, m)) if *y == x => *m = (*m).max(k),
            _ => merged.push((x, k)),
        }
    }
    merged.retain(|&(_, k)| k > 0);
    merged
}

/// The tensor of the counters' tokens, `1` for none.
fn tensor(names: &[&str], counts: &[(usize, u32)]) -> String {
    let factors: Vec<&str> = counts
        .iter()
        .flat_map(|&(x, k)| std::iter::repeat_n(names[x], k as usize))
        .collect();
    if factors.is_empty() {
        "1".to_owned()
    } else {
        factors.join(" * ")
    }
}

/// A token of a `.spec` file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Token<'a> {
    /// A name, a number or an operator.
    Word(&'a str),
    /// The end of a line.
    Line,
}

impl std::fmt::Display for Token<'_> {
    /// Writes the token for an error message.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Word(word) => write!(f, "`{word}`"),
            Token::Line => f.write_str("the end of a line"),
        }
    }
}

/// Splits a file into names (letters, digits and `_`), numbers, the
/// operators `>=`, `->`, `=`, `'`, `+`, `-`, `,` and `;`, and ends of
/// lines, comments dropped.
fn tokens(text: &str) -> Result<Vec<Token<'_>>, Error> {
    let mut tokens = Vec::new();
    for line in text.lines() {
        let line = line.split_once('#').map_or(line, |(code, _)| code);
        let mut at = 0;
        while at < line.len() {
            let rest = &line[at..];
            let c = rest.chars().next().expect("not at the end");
            let length = if c.is_whitespace() {
                at += c.len_utf8();
                continue;
            } else if c.is_ascii_alphanumeric() || c == '_' {
                rest.find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                    .unwrap_or(rest.len())
            } else if rest.starts_with(">=") || rest.starts_with("->") {
                2
            } else if "=+-,;'".contains(c) {
                1
            } else {
                return Err(error(format!("unexpected `{c}`")));
            };
            tokens.push(Token::Word(&rest[..length]));
            at += length;
        }
        tokens.push(Token::Line);
    }
    Ok(tokens)
}

/// What reads the tokens: the tokens and the next one's index.
struct Reader<'a> {
    /// The tokens.
    tokens: Vec<Token<'a>>,
    /// The next token's index.
    at: usize,
}

impl<'a> Reader<'a> {
    /// Skips ends of lines.
    fn skip_lines(&mut self) {
        while self.tokens.get(self.at) == Some(&Token::Line) {
            self.at += 1;
        }
    }

    /// The next word, ends of lines skipped, or the error of a file that
    /// ends.
    fn next(&mut self) -> Result<Token<'a>, Error> {
        self.skip_lines();
        let token = *self
            .tokens
            .get(self.at)
            .ok_or_else(|| error("the file ends early".to_owned()))?;
        self.at += 1;
        Ok(token)
    }

    /// Whether the next word, ends of lines skipped, is `keyword`.
    fn at_keyword(&mut self, keyword: &str) -> bool {
        self.skip_lines();
        self.tokens.get(self.at) == Some(&Token::Word(keyword))
    }

    /// Reads the word `word`.
    fn word(&mut self, word: &str) -> Result<(), Error> {
        match self.next()? {
            Token::Word(w) if w == word => Ok(()),
            token => Err(error(format!("expected `{word}` at {token}"))),
        }
    }

    /// Reads the section's keyword `keyword`.
    fn keyword(&mut self, keyword: &str) -> Result<(), Error> {
        self.word(keyword)
            .map_err(|_| error(format!("expected the section `{keyword}`")))
    }

    /// Reads a name.
    fn name(&mut self) -> Result<&'a str, Error> {
        match self.next()? {
            Token::Word(w) if w.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_') => Ok(w),
            token => Err(error(format!("expected a name at {token}"))),
        }
    }

    /// Reads a number below 2³².
    fn number(&mut self) -> Result<u32, Error> {
        match self.next()? {
            Token::Word(w) => w
                .parse()
                .map_err(|_| error(format!("expected a number below 2³² at `{w}`"))),
            token => Err(error(format!("expected a number at {token}"))),
        }
    }

    /// Reads a comma if one comes next, on this line or first on a later
    /// one, and returns whether one did or the line goes on: a comma at the
    /// end of a line or at the start of the next joins the two, as
    /// qcover's files write long initial markings.
    fn comma_or_line(&mut self) -> bool {
        let mut at = self.at;
        while self.tokens.get(at) == Some(&Token::Line) {
            at += 1;
        }
        match self.tokens.get(at) {
            Some(Token::Word(",")) => {
                self.at = at + 1;
                true
            }
            Some(Token::Word(_)) => at == self.at,
            _ => false,
        }
    }

    /// Reads a rule, `guards -> updates;`, as the tokens it takes from
    /// each counter and those it gives.
    fn rule(
        &mut self,
        names: &[&str],
        counter: &dyn Fn(&str) -> Result<usize, Error>,
    ) -> Result<(Counts, Counts), Error> {
        let mut guards = Vec::new();
        let mut changes: Vec<(usize, i64)> = Vec::new();
        if !matches!(self.tokens.get(self.at), Some(Token::Word("->"))) {
            loop {
                let x = counter(self.name()?)?;
                self.word(">=")?;
                guards.push((x, self.number()?));
                match self.next()? {
                    Token::Word(",") => {}
                    Token::Word("->") => break,
                    token => return Err(error(format!("expected `,` or `->` at {token}"))),
                }
            }
        } else {
            self.at += 1;
        }
        loop {
            if let Ok(Token::Word(";")) = self.peek() {
                self.at += 1;
                break;
            }
            let name = self.name()?;
            let x = counter(name)?;
            self.word("'")?;
            self.word("=")?;
            if self.name()? != name {
                return Err(error(format!("`{name}'` is updated from another counter")));
            }
            let sign = match self.next()? {
                Token::Word("+") => 1,
                Token::Word("-") => -1,
                token => return Err(error(format!("expected `+` or `-` at {token}"))),
            };
            let k = i64::from(self.number()?);
            if changes.iter().any(|&(y, _)| y == x) {
                return Err(error(format!("a rule updates `{name}` twice")));
            }
            changes.push((x, sign * k));
            match self.next()? {
                Token::Word(",") => {}
                Token::Word(";") => break,
                token => return Err(error(format!("expected `,` or `;` at {token}"))),
            }
        }
        let guards = merged(guards);
        let mut counters: Vec<usize> = guards.iter().map(|&(x, _)| x).collect();
        counters.extend(changes.iter().map(|&(x, _)| x));
        counters.sort_unstable();
        counters.dedup();
        let (mut inputs, mut outputs) = (Vec::new(), Vec::new());
        for x in counters {
            let guard = guards.iter().find(|&&(y, _)| y == x).map_or(0, |&(_, k)| k);
            let change = changes
                .iter()
                .find(|&&(y, _)| y == x)
                .map_or(0, |&(_, c)| c);
            let taken = i64::from(guard).max(-change);
            let given = u32::try_from(taken + change)
                .map_err(|_| error(format!("a rule gives `{}` 2³² tokens or more", names[x])))?;
            inputs.push((x, u32::try_from(taken).expect("a guard or a decrement")));
            outputs.push((x, given));
        }
        inputs.retain(|&(_, k)| k > 0);
        outputs.retain(|&(_, k)| k > 0);
        Ok((inputs, outputs))
    }

    /// The next word, ends of lines skipped, without reading it.
    fn peek(&mut self) -> Result<Token<'a>, Error> {
        let token = self.next()?;
        self.at -= 1;
        Ok(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Rules over several lines, guards above a decrement, a decrement
    /// without a guard, a rule without guards, comments, parameters, lines
    /// joined by a comma at the end of one or the start of the next, a
    /// target of two lines and a counter left out of `init`, and malformed
    /// files refused.
    #[test]
    fn reads_problems() {
        let text = "# a comment\n\
                    vars\n  a b c\n\
                    rules\n  a >= 2 ->\n    a' = a - 1,\n    b' = b + 1;\n\
                    \x20 b >= 1 -> c' = c - 1;   # takes a c it does not test\n\
                    \x20 -> a' = a + 1;\n\
                    init\n  a = 1\n  , b >= 2\n\
                    target\n  c >= 1,\n  b >= 1\n  a >= 3\n\
                    invariants\n  a = 1, b = 1\n";
        let problem = read(text).unwrap();
        assert_eq!(problem.expected, None);
        let expected: Sequent = "!(a * a -o a * b), !(b * c -o b), !(1 -o a), \
                                 !(b * c -o goal), !(a * a * a -o goal), !b, a, b, b |- goal"
            .parse()
            .unwrap();
        assert_eq!(problem.sequent, expected);

        for bad in [
            "rules init target a >= 1",
            "vars a rules init a = 1",
            "vars a rules a >= 1 -> b' = b + 1; init target a >= 1",
            "vars a b rules a >= 1 -> a' = b + 1; init target a >= 1",
            "vars a rules a >= 1 -> a' = a + 1, a' = a - 1; init target a >= 1",
            "vars a rules init a = 4294967296 target a >= 1",
            "vars top rules init target top >= 1",
        ] {
            assert!(matches!(read(bad), Err(Error::Mist(_))), "{bad:?}");
        }
    }
}
