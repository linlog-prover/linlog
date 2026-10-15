// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Problems written in other provers' syntax. Every translator prints
//! through the library's public walk over a formula (`Sequent::walk`, or
//! `Reading::walk` for an intuitionistic sequent), never by recursion, so
//! a formula of any depth takes no more stack than a literal. Every binary
//! formula is written in parentheses and every operand of `!` and `?` too,
//! so that only the rule that `!` and `?` bind tighter than every binary
//! connective matters, which every tool's grammar has, and an atom is
//! written by its
//! index (`p0`, `p1`, …), so that no name clashes with a tool's keywords
//! or lexical rules.

use crate::problems::Problem;
use anyhow::{Result, anyhow};
use linlog::occurrences::IllConnective;
use linlog::sequents::Visit;
use linlog::{Forest, OccId, Reading, Sequent, Term, TermId};
use std::fmt::Write as _;

/// How a tool spells the connectives and units.
pub struct Spelling {
    /// `⊗`
    pub tensor: &'static str,
    /// `⅋`
    pub par: &'static str,
    /// `&`
    pub with: &'static str,
    /// `⊕`
    pub plus: &'static str,
    /// `⊸`, in an intuitionistic formula.
    pub lollipop: &'static str,
    /// `1`
    pub one: &'static str,
    /// `⊥`
    pub bot: &'static str,
    /// `⊤`
    pub top: &'static str,
    /// `0`
    pub zero: &'static str,
    /// `!`, written before the parenthesised operand.
    pub bang: &'static str,
    /// `?`, written before the parenthesised operand.
    pub quest: &'static str,
    /// The text before an atom's index.
    pub atom: &'static str,
    /// The text before a negated atom's index and after it.
    pub dual: (&'static str, &'static str),
}

/// Writes the formula rooted at `root` of `sequent`, one-sided, into `out`.
pub fn one_sided(out: &mut String, sequent: &Sequent, root: TermId, s: &Spelling) {
    for visit in sequent.walk(root) {
        let _ = match visit {
            Visit::Enter(t) => match sequent.term(t) {
                Term::Atom(a) => write!(out, "{}{}", s.atom, a.index()),
                Term::DualAtom(a) => write!(out, "{}{}{}", s.dual.0, a.index(), s.dual.1),
                Term::One => write!(out, "{}", s.one),
                Term::Bot => write!(out, "{}", s.bot),
                Term::Top => write!(out, "{}", s.top),
                Term::Zero => write!(out, "{}", s.zero),
                Term::Bang(_) => write!(out, "{}(", s.bang),
                Term::Quest(_) => write!(out, "{}(", s.quest),
                Term::Tensor(..) | Term::Par(..) | Term::With(..) | Term::Plus(..) => {
                    write!(out, "(")
                }
            },
            Visit::Between(t, _) => {
                let op = match sequent.term(t) {
                    Term::Tensor(..) => s.tensor,
                    Term::Par(..) => s.par,
                    Term::With(..) => s.with,
                    Term::Plus(..) => s.plus,
                    // No stop between operands comes at a literal, a unit or
                    // a formula of one operand.
                    Term::Atom(_)
                    | Term::DualAtom(_)
                    | Term::One
                    | Term::Bot
                    | Term::Top
                    | Term::Zero
                    | Term::Bang(_)
                    | Term::Quest(_) => "",
                };
                write!(out, " {op} ")
            }
            Visit::Exit(_) => write!(out, ")"),
        };
    }
}

/// Writes the intuitionistic formula at `o` of `reading` into `out`, or
/// says which connective the spelling has no place for.
pub fn intuitionistic(out: &mut String, reading: &Reading, o: OccId, s: &Spelling) -> Result<()> {
    use IllConnective as C;
    let forest = reading.forest();
    for visit in reading.walk(o) {
        let _ = match visit {
            Visit::Enter(o) => match reading.connective(o) {
                C::Atom => {
                    let atom = (forest.atom(o)).ok_or_else(|| anyhow!("an atom without one"))?;
                    write!(out, "{}{}", s.atom, atom.index())
                }
                C::One => write!(out, "{}", s.one),
                C::Top => write!(out, "{}", s.top),
                C::Zero => write!(out, "{}", s.zero),
                C::Bang => write!(out, "{}(", s.bang),
                C::Tensor | C::With | C::Plus | C::Lollipop => write!(out, "("),
                other => return Err(anyhow!("no translation of {other:?}")),
            },
            Visit::Between(o, _) => {
                let op = match reading.connective(o) {
                    C::Tensor => s.tensor,
                    C::With => s.with,
                    C::Plus => s.plus,
                    C::Lollipop => s.lollipop,
                    other => return Err(anyhow!("no translation of {other:?}")),
                };
                write!(out, " {op} ")
            }
            Visit::Exit(_) => write!(out, ")"),
        };
    }
    Ok(())
}

/// The formulas of a classical problem, one-sided, each written by
/// [`one_sided`].
pub fn classical_formulas(problem: &Problem, s: &Spelling) -> Vec<String> {
    let sequent = &problem.sequent;
    (sequent.roots().iter())
        .map(|&root| {
            let mut out = String::new();
            one_sided(&mut out, sequent, root, s);
            out
        })
        .collect()
}

/// The hypotheses and the goal of an intuitionistic problem, each written
/// by [`intuitionistic`].
pub fn intuitionistic_formulas(problem: &Problem, s: &Spelling) -> Result<(Vec<String>, String)> {
    let forest = Forest::new(&problem.sequent)?;
    let reading = Reading::new(&forest).map_err(|e| anyhow!("no intuitionistic reading: {e}"))?;
    let write = |o: OccId| -> Result<String> {
        let mut out = String::new();
        intuitionistic(&mut out, &reading, o, s)?;
        Ok(out)
    };
    let hypotheses = reading.hypotheses().map(write).collect::<Result<_>>()?;
    Ok((hypotheses, write(reading.goal())?))
}

/// Click & coLLecT's ASCII syntax: a one-sided classical sequent `|- Γ`.
const CLICK_AND_COLLECT: Spelling = Spelling {
    tensor: "*",
    par: "|",
    with: "&",
    plus: "+",
    lollipop: "-o",
    one: "1",
    bot: "bot",
    top: "top",
    zero: "0",
    bang: "!",
    quest: "?",
    atom: "p",
    dual: ("p", "^"),
};

/// A classical problem for Click & coLLecT: `|- F1, …, Fn` on one line.
pub fn click_and_collect(problem: &Problem) -> Result<String> {
    Ok(format!(
        "|- {}\n",
        classical_formulas(problem, &CLICK_AND_COLLECT).join(", ")
    ))
}

/// The Maude prover's syntax: quoted identifiers, `o` for `⊕`, a
/// negated atom `'p ^`.
const MAUDE: Spelling = Spelling {
    tensor: "*",
    par: "|",
    with: "&",
    plus: "o",
    lollipop: "-o",
    one: "1",
    bot: "bot",
    top: "top",
    zero: "0",
    bang: "!",
    quest: "?",
    atom: "'p",
    dual: ("'p", " ^"),
};

/// The copy bound the Maude prover's own script for the LLTP library
/// gives its intuitionistic search (`utils/exec-ILLTP-flat-file.sh`).
pub const MAUDE_COPIES: u32 = 5;

/// An intuitionistic problem for the Maude prover's focused ILL system: a
/// reduction of `solve` on `[emp] Γ |~ A`, under the copy bound of the
/// prover's own script.
pub fn maude_ill(problem: &Problem) -> Result<String> {
    let (hypotheses, goal) = intuitionistic_formulas(problem, &MAUDE)?;
    let hypotheses = if hypotheses.is_empty() {
        "empMS".to_owned()
    } else {
        hypotheses.join(", ")
    };
    Ok(format!(
        "red solve({MAUDE_COPIES}, ([emp] {hypotheses} |~ {goal})) .\nq\n"
    ))
}

/// A classical problem for the Maude prover's focused LL system: a search
/// for a proof of `|~ [emp] Γ`.
pub fn maude_cll(problem: &Problem) -> Result<String> {
    Ok(format!(
        "search [1] prove(|~ [emp] {}) =>* proved .\nq\n",
        classical_formulas(problem, &MAUDE).join(", ")
    ))
}

/// llprover's syntax: `+` for `⅋`, `/\\` and `\\/` for the additives,
/// `0` for `⊥` and `bot` for `0`, as Girard wrote them first.
const LLPROVER: Spelling = Spelling {
    tensor: "*",
    par: "+",
    with: "/\\",
    plus: "\\/",
    lollipop: "->",
    one: "1",
    bot: "0",
    top: "top",
    zero: "bot",
    bang: "!",
    quest: "?",
    atom: "p",
    dual: ("~(p", ")"),
};

/// A classical problem for llprover: its classical system, then the
/// one-sided sequent `[] --> Γ`.
pub fn llprover_cll(problem: &Problem) -> Result<String> {
    let formulas = classical_formulas(problem, &LLPROVER);
    let right = if formulas.is_empty() {
        "[]".to_owned()
    } else {
        formulas.join(", ")
    };
    Ok(format!("cll(full).\n[] --> {right}.\n"))
}

/// An intuitionistic problem for llprover: its intuitionistic system, then
/// `Γ --> A`.
pub fn llprover_ill(problem: &Problem) -> Result<String> {
    let (hypotheses, goal) = intuitionistic_formulas(problem, &LLPROVER)?;
    let hypotheses = if hypotheses.is_empty() {
        "[]".to_owned()
    } else {
        hypotheses.join(", ")
    };
    Ok(format!("ill(full).\n{hypotheses} --> {goal}.\n"))
}

/// linTAP's syntax: `@` for `⅋`, `0` for `⊥`, a negated atom `(~p)`.
const LINTAP: Spelling = Spelling {
    tensor: "*",
    par: "@",
    with: "",
    plus: "",
    lollipop: "@>",
    one: "1",
    bot: "0",
    top: "",
    zero: "",
    bang: "!",
    quest: "?",
    atom: "p",
    dual: ("(~p", ")"),
};

/// A classical problem for linTAP: the one formula `F1 @ (F2 @ …)`, the
/// `⅋` of the sequent's formulas. linTAP reads a `!` it meets in a formula
/// proved one-sided as an atom, so a sequent with one is refused.
pub fn lintap(problem: &Problem) -> Result<String> {
    // The parser and the readers keep no term that no root reaches.
    if (problem.sequent.terms().iter()).any(|t| matches!(t, Term::Bang(_))) {
        return Err(anyhow!("linTAP reads a `!` proved one-sided as an atom"));
    }
    let formulas = classical_formulas(problem, &LINTAP);
    // The `⅋` of no formulas is `⊥`, which linTAP writes `0`.
    let Some(last) = formulas.len().checked_sub(1) else {
        return Ok("0.\n".to_owned());
    };
    let mut out = String::new();
    for (i, formula) in formulas.iter().enumerate() {
        if i < last {
            let _ = write!(out, "({formula} @ ");
        } else {
            out.push_str(formula);
        }
    }
    out.push_str(&")".repeat(last));
    out.push_str(".\n");
    Ok(out)
}

/// An intuitionistic multiplicative problem for LinearOne: `[Γ] => A.`,
/// an atom `at(p, [])`, `p(A, B)` for `⊗` and `impl(A, B)` for `⊸`.
pub fn linearone(problem: &Problem) -> Result<String> {
    let forest = Forest::new(&problem.sequent)?;
    let reading = Reading::new(&forest).map_err(|e| anyhow!("no intuitionistic reading: {e}"))?;
    let write = |o: OccId| -> Result<String> {
        let mut out = String::new();
        for visit in reading.walk(o) {
            match visit {
                Visit::Enter(o) => match reading.connective(o) {
                    IllConnective::Atom => {
                        let atom =
                            (forest.atom(o)).ok_or_else(|| anyhow!("an atom without one"))?;
                        let _ = write!(out, "at(p{},[])", atom.index());
                    }
                    IllConnective::Tensor => out.push_str("p("),
                    IllConnective::Lollipop => out.push_str("impl("),
                    other => return Err(anyhow!("LinearOne has no {other:?}")),
                },
                Visit::Between(..) => out.push(','),
                Visit::Exit(_) => out.push(')'),
            }
        }
        Ok(out)
    };
    let hypotheses = reading
        .hypotheses()
        .map(write)
        .collect::<Result<Vec<_>>>()?;
    Ok(format!(
        "[{}] => {}.\n",
        hypotheses.join(","),
        write(reading.goal())?
    ))
}

/// Sympli's syntax: `#` for `⊤`, an operator between spaces, since an
/// atom may contain `*`, `+`, `!` and `#`.
const SYMPLI: Spelling = Spelling {
    tensor: "*",
    par: "",
    with: "&",
    plus: "+",
    lollipop: "-o",
    one: "1",
    bot: "",
    top: "#",
    zero: "0",
    bang: "! ",
    quest: "",
    atom: "p",
    dual: ("", ""),
};

/// An intuitionistic problem for Sympli, which proves a formula without
/// hypotheses: `Γ ⊢ A` as `(⊗Γ) ⊸ A`, with its log of the proof's
/// validation on.
pub fn sympli(problem: &Problem) -> Result<String> {
    let (hypotheses, goal) = intuitionistic_formulas(problem, &SYMPLI)?;
    let formula = if hypotheses.is_empty() {
        goal
    } else {
        format!("(({})) -o {goal}", hypotheses.join(") * ("))
    };
    Ok(format!("%log validation : highlevel.\n%prove {formula}.\n"))
}

/// lolli's syntax, its Unicode spelling.
const LOLLI: Spelling = Spelling {
    tensor: "⊗",
    par: "⅋",
    with: "&",
    plus: "⊕",
    lollipop: "⊸",
    one: "1",
    bot: "⊥",
    top: "⊤",
    zero: "0",
    bang: "!",
    quest: "?",
    atom: "p",
    dual: ("p", "^"),
};

/// The longest argument the kernel passes to a program (`MAX_ARG_STRLEN`).
const ARGUMENT: usize = 128 * 1024;

/// A classical problem for lolli: `⊢ Γ`, which its command takes as one
/// argument, so a sequent longer than an argument can be is refused.
pub fn lolli(problem: &Problem) -> Result<String> {
    let text = format!("⊢ {}", classical_formulas(problem, &LOLLI).join(", "));
    if text.len() >= ARGUMENT {
        return Err(anyhow!(
            "lolli takes the sequent as one argument, at most {ARGUMENT} bytes"
        ));
    }
    Ok(text)
}

/// APLL's syntax: `|` for `⅋`, a negated atom `p^`.
const APLL: Spelling = Spelling {
    tensor: "*",
    par: "|",
    with: "&",
    plus: "+",
    lollipop: "-o",
    one: "1",
    bot: "bot",
    top: "top",
    zero: "0",
    bang: "!",
    quest: "?",
    atom: "p",
    dual: ("p", "^"),
};

/// A classical problem for APLL: `|- Γ`.
pub fn apll_ll(problem: &Problem) -> Result<String> {
    Ok(format!(
        "|- {}\n",
        classical_formulas(problem, &APLL).join(", ")
    ))
}

/// An intuitionistic problem for APLL: `Γ |- A`.
pub fn apll_ill(problem: &Problem) -> Result<String> {
    let (hypotheses, goal) = intuitionistic_formulas(problem, &APLL)?;
    Ok(format!("{} |- {goal}\n", hypotheses.join(", ")))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A problem of the sequent in the mode.
    fn problem(text: &str, mode: linlog::Mode) -> Problem {
        Problem {
            sequent: text.parse().unwrap(),
            mode,
            expected: None,
            copies: None,
        }
    }

    /// Each translator writes a small sequent as its tool reads it.
    #[test]
    fn translations() {
        let classical = problem(
            "!a, b * c |- ?~a par (1 & top), 0 + bot",
            linlog::Mode::CLASSICAL,
        );
        assert_eq!(
            click_and_collect(&classical).unwrap(),
            "|- ?(p0^), (p1^ | p2^), (?(p0^) | (1 & top)), (0 + bot)\n"
        );
        assert_eq!(
            maude_cll(&classical).unwrap(),
            "search [1] prove(|~ [emp] ?('p0 ^), ('p1 ^ | 'p2 ^), (?('p0 ^) | (1 & top)), \
             (0 o bot)) =>* proved .\nq\n"
        );
        let intuitionistic = problem(
            "!a, a -o (b & 1) |- !(b + top) * 0",
            linlog::Mode::INTUITIONISTIC,
        );
        assert_eq!(
            maude_ill(&intuitionistic).unwrap(),
            "red solve(5, ([emp] !('p0), ('p0 -o ('p1 & 1)) |~ (!(('p1 o top)) * 0))) .\nq\n"
        );
        assert_eq!(
            llprover_cll(&classical).unwrap(),
            "cll(full).\n[] --> ?(~(p0)), (~(p1) + ~(p2)), (?(~(p0)) + (1 /\\ top)), (bot \\/ 0).\n"
        );
        assert_eq!(
            llprover_ill(&intuitionistic).unwrap(),
            "ill(full).\n!(p0), (p0 -> (p1 /\\ 1)) --> (!((p1 \\/ top)) * bot).\n"
        );
        let mll = problem("a * b, ~c |- b * a, ?~c", linlog::Mode::CLASSICAL);
        assert_eq!(
            lintap(&mll).unwrap(),
            "(((~p0) @ (~p1)) @ (p2 @ ((p1 * p0) @ ?((~p2))))).\n"
        );
        assert_eq!(
            lintap(&problem("|-", linlog::Mode::CLASSICAL)).unwrap(),
            "0.\n"
        );
        let bang = problem("?~a |- !a", linlog::Mode::CLASSICAL);
        assert!(lintap(&bang).is_err(), "a ! is refused");
        let mill = problem("a -o b, a * c |- b * c", linlog::Mode::INTUITIONISTIC);
        assert_eq!(
            linearone(&mill).unwrap(),
            "[impl(at(p0,[]),at(p1,[])),p(at(p0,[]),at(p2,[]))] => p(at(p1,[]),at(p2,[])).\n"
        );
        assert_eq!(
            sympli(&intuitionistic).unwrap(),
            "%log validation : highlevel.\n%prove ((! (p0)) * ((p0 -o (p1 & 1)))) -o (! ((p1 + #)) * 0).\n"
        );
        assert_eq!(
            lolli(&classical).unwrap(),
            "⊢ ?(p0^), (p1^ ⅋ p2^), (?(p0^) ⅋ (1 & ⊤)), (0 ⊕ ⊥)"
        );
        assert_eq!(
            apll_ll(&classical).unwrap(),
            "|- ?(p0^), (p1^ | p2^), (?(p0^) | (1 & top)), (0 + bot)\n"
        );
        assert_eq!(
            apll_ill(&intuitionistic).unwrap(),
            "!(p0), (p0 -o (p1 & 1)) |- (!((p1 + top)) * 0)\n"
        );
        let goal = problem("|- a -o a", linlog::Mode::INTUITIONISTIC);
        assert_eq!(
            maude_ill(&goal).unwrap(),
            "red solve(5, ([emp] empMS |~ ('p0 -o 'p0))) .\nq\n"
        );
    }

    /// A formula nested 100 000 deep is translated on a stack of 256 KiB.
    #[test]
    fn deep_formulas_need_no_stack() {
        let depth = 100_000;
        let text = format!("|- {}a{}", "!(a * ".repeat(depth), ")".repeat(depth));
        let deep = problem(&text, linlog::Mode::CLASSICAL);
        let translated = std::thread::Builder::new()
            .stack_size(256 * 1024)
            .spawn(move || click_and_collect(&deep).unwrap().len())
            .unwrap()
            .join()
            .unwrap();
        assert!(translated > 8 * depth);
    }
}
