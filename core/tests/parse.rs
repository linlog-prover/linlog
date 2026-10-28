// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! What sequents parse to and print as, through the public API.

#![cfg(feature = "parse")]

use linlog::{Error, Limits, Order, Sequent};

/// Parses `input` as a sequent of classical linear logic and prints it back.
fn pretty(input: &str) -> String {
    match input.parse::<Sequent>() {
        Ok(s) => s.to_string(),
        Err(e) => panic!("{input:?} does not parse: {e}"),
    }
}

/// Returns whether `input` is rejected by the parser.
fn rejected(input: &str) -> bool {
    input.parse::<Sequent>().is_err()
}

/// Every constant parses in each of its spellings, dualised on the left.
#[test]
fn constants() {
    for (input, printed) in [
        ("|- 0", "⊢ 0"),
        ("|- 1", "⊢ 1"),
        ("|- bot", "⊢ ⊥"),
        ("|- ⊥", "⊢ ⊥"),
        ("|- top", "⊢ ⊤"),
        ("|- ⊤", "⊢ ⊤"),
        ("0 |-", "⊢ ⊤"),
        ("1 |-", "⊢ ⊥"),
        ("⊥ |-", "⊢ 1"),
        ("⊤ |-", "⊢ 0"),
        ("|- !(A * 1) + ?⊤", "⊢ !(A ⊗ 1) ⊕ ?⊤"),
    ] {
        assert_eq!(pretty(input), printed, "{input:?}");
    }
}

/// A sequent may have no formulas on either side, or on both.
#[test]
fn empty_sides() {
    for (input, printed) in [("|-", "⊢"), ("⊢", "⊢"), ("A |-", "⊢ ~A"), ("|- A", "⊢ A")] {
        assert_eq!(pretty(input), printed, "{input:?}");
    }
}

/// Operators bind from tightest to loosest: `^`, then `~ ! ?`, tensor, par,
/// with, plus and lollipop.
#[test]
fn precedence() {
    for (input, printed) in [
        ("|- !A^ * ?~B", "⊢ !~A ⊗ ?~B"),
        (
            "|- A -o B + C & D par E * F",
            "⊢ ~A ⅋ (B ⊕ (C & (D ⅋ (E ⊗ F))))",
        ),
        (
            "|- A * B par C & D + E -o F",
            "⊢ ((((~A ⅋ ~B) ⊗ ~C) ⊕ ~D) & ~E) ⅋ F",
        ),
    ] {
        assert_eq!(pretty(input), printed, "{input:?}");
    }
}

/// Lollipop associates to the right, every other binary operator to the left.
#[test]
fn associativity() {
    for (input, printed) in [
        ("|- A -o B -o C", "⊢ ~A ⅋ (~B ⅋ C)"),
        ("|- A * B * C", "⊢ (A ⊗ B) ⊗ C"),
        ("|- A par B par C", "⊢ (A ⅋ B) ⅋ C"),
        ("|- A & B & C", "⊢ (A & B) & C"),
        ("|- A + B + C", "⊢ (A ⊕ B) ⊕ C"),
    ] {
        assert_eq!(pretty(input), printed, "{input:?}");
    }
}

/// Every spelling of a binary operator binds and associates like the others,
/// also when spellings are mixed in one formula.
#[test]
fn operator_spellings_are_interchangeable() {
    let operators: [&[&str]; 4] = [&["*", "⊗"], &["par", "|", "⅋"], &["+", "⊕"], &["-o", "⊸"]];
    for spellings in operators {
        let expected = pretty(&format!("|- A {0} B {0} C", spellings[0]));
        for first in spellings {
            for second in spellings {
                let input = format!("|- A {first} B {second} C");
                assert_eq!(pretty(&input), expected, "{input:?}");
            }
        }
    }
}

/// A variable may occur any number of times, in any order with the others.
#[test]
fn repeated_variables() {
    for (input, printed) in [
        ("|- A, A, B", "⊢ A, A, B"),
        ("A, A -o B |- B", "⊢ ~A, A ⊗ ~B, B"),
        ("B, A |- A * B", "⊢ ~B, ~A, A ⊗ B"),
    ] {
        assert_eq!(pretty(input), printed, "{input:?}");
    }
}

/// A name is one atom however its accents are encoded: `é` composed and
/// as `e` with a combining acute is the composed name (NFC).
#[test]
fn names_are_composed() {
    let sequent: Sequent = "e\u{301} |- \u{e9}".parse().unwrap();
    assert_eq!(sequent, "\u{e9} |- \u{e9}".parse().unwrap());
    assert_eq!(sequent.atom_names(), ["\u{e9}"]);
}

/// Repeated formulas are all kept, each with its own subformulas.
#[test]
fn repeated_formulas() {
    for (input, printed) in [
        ("|- A, A, B, C, C", "⊢ A, A, B, C, C"),
        ("|- ⊤, ⊤, ⊥, ⊥", "⊢ ⊤, ⊤, ⊥, ⊥"),
        ("|- A, A, B * B", "⊢ A, A, B ⊗ B"),
    ] {
        assert_eq!(pretty(input), printed, "{input:?}");
    }
}

/// A word that merely starts like a constant is a variable or an error.
#[test]
fn constant_prefixes() {
    assert_eq!(pretty("|- bottom, topx"), "⊢ bottom, topx");
    assert!(rejected("|- 0a"));
    assert!(rejected("|- 10"));
    assert!(rejected("|- ⊥A"));
}

/// Printing a parsed sequent gives text that parses back to the same sequent.
#[test]
fn printed_sequents_parse_back() {
    for input in [
        "|-",
        "A |- A",
        "A * B, C par D |- A & B, C + D",
        "!A, ?B |- ~A, B^",
        "A -o B -o C, (A -o B) -o C |- (A * B) + (0 & top)",
        "0, 1, bot, top |- 0, 1, ⊥, ⊤",
        "A, A -o B |- B, B * ~A",
    ] {
        let printed = pretty(input);
        assert_eq!(pretty(&printed), printed, "{input:?}");
    }
}

/// An error names the first character that cannot go on a sequent, by its
/// byte offset, or the end of the input; of `|-` and `-o` cut short that
/// is the character after the first.
/// The cyclic order's parse reverses what a negation dualises, products
/// and the antecedents, and leaves every other formula as the commutative
/// parse reads it; its sequents print back as themselves.
#[test]
fn cyclic_duals_reverse() {
    let cyclic = |input: &str| {
        Sequent::parse_ordered(input, Order::Cyclic, &Limits::default())
            .unwrap_or_else(|e| panic!("{input:?}: {e}"))
    };
    for (input, printed) in [
        ("|- ~(a * b), a, b", "⊢ ~b ⅋ ~a, a, b"),
        ("|- (a par b)^, b, a", "⊢ ~b ⊗ ~a, b, a"),
        ("a, a -o b |- b", "⊢ ~b ⊗ a, ~a, b"),
        ("a -o b, a |- b", "⊢ ~a, ~b ⊗ a, b"),
        ("a, b, c |- (a * b) * c", "⊢ ~c, ~b, ~a, (a ⊗ b) ⊗ c"),
        ("|- a * (b par c), ~c", "⊢ a ⊗ (b ⅋ c), ~c"),
    ] {
        let sequent = cyclic(input);
        assert_eq!(sequent.to_string(), printed, "{input}");
        assert_eq!(sequent.antecedents(), cyclic(input).antecedents());
        assert_eq!(cyclic(printed).to_string(), printed);
    }
    assert_eq!(cyclic("a, b |- c").antecedents(), Some(2));
    assert_eq!(pretty("a, b |- (a * b) * c"), "⊢ ~a, ~b, (a ⊗ b) ⊗ c");
    assert_eq!(
        Sequent::parse_ordered("a * b |- c", Order::Commutative, &Limits::default()).unwrap(),
        "a * b |- c".parse::<Sequent>().unwrap()
    );
}

/// The ordered parse reads the Lambek calculus's two divisions: `A \ B`
/// as `A -o B` and `B / A` as `B par ~A`, `\` to the right and `/` to
/// the left; the commutative parse has neither.
#[test]
fn divisions() {
    let cyclic = |input: &str| {
        Sequent::parse_ordered(input, Order::Cyclic, &Limits::default())
            .unwrap_or_else(|e| panic!("{input:?}: {e}"))
            .to_string()
    };
    for (input, printed) in [
        ("a, a \\ b |- b", "⊢ ~b ⊗ a, ~a, b"),
        ("b / a, a |- b", "⊢ ~a, a ⊗ ~b, b"),
        ("|- a \\ b \\ c", "⊢ ~a ⅋ (~b ⅋ c)"),
        ("|- c / b / a", "⊢ (c ⅋ ~b) ⅋ ~a"),
        ("|- a \\ b / c", "⊢ ~a ⅋ (b ⅋ ~c)"),
    ] {
        assert_eq!(cyclic(input), printed, "{input}");
    }
    assert_eq!(cyclic("a \\ b |- c"), cyclic("a -o b |- c"));
    for input in ["b / a |- b", "a \\ b |- b"] {
        assert!(input.parse::<Sequent>().is_err(), "{input}");
    }
}

#[test]
fn error_positions() {
    for (input, at, found) in [
        ("", 0, None),
        ("A", 1, None),
        ("|- (A", 5, None),
        ("|- A,", 5, None),
        ("|- A * )", 7, Some(")")),
        ("|- A B", 5, Some("B")),
        ("|- A parx B", 5, Some("p")),
        ("|- (A, B)", 5, Some(",")),
        ("A, |- B", 3, Some("|")),
        ("A |- B |- C", 8, Some("-")),
        ("(A |- A)", 4, Some("-")),
        ("| A |- B", 1, Some(" ")),
        ("A - B |- C", 3, Some(" ")),
        ("A -", 3, None),
        // A binder without its variable ends at the end of the input.
        ("|- é * ∀", 11, None),
        // The Unicode turnstile out of place, and a `)` that closes
        // nothing; `⊢` is three bytes.
        ("A, ⊢ B", 3, Some("⊢")),
        ("A ⊢ ⊢ B", 6, Some("⊢")),
        ("A ⊢ B ⊢ C", 8, Some("⊢")),
        ("(A ⊢ A)", 3, Some("⊢")),
        ("|- A)", 4, Some(")")),
        // A keyword where a formula starts is the whole word.
        ("|- par par par", 3, Some("par")),
        ("|- A * forall", 13, None),
    ] {
        let Err(Error::Parse(error)) = input.parse::<Sequent>() else {
            panic!("{input:?} is no parse error");
        };
        assert_eq!(
            (error.span.start, error.found.as_deref()),
            (at, found),
            "{input:?}"
        );
    }
}

/// An error gives its place as a line and a character, and in UTF-16
/// code units, and says what could have stood there: a sequent without a
/// turnstile ends where `|-` was due.
#[test]
fn error_places_and_expectations() {
    let error = |input: &str| match input.parse::<Sequent>() {
        Err(Error::Parse(error)) => *error,
        other => panic!("{input:?}: {other:?}"),
    };
    let e = error("|- é,\n 𝔸 * ⊢");
    assert_eq!((e.span.clone(), e.span_utf16.clone()), (15..18, 12..13));
    assert_eq!((e.line, e.column), (2, 6));
    assert_eq!(e.expected, ["a formula"]);
    assert_eq!(
        e.to_string(),
        "unexpected \"⊢\" at line 2, character 6, expected a formula"
    );
    let e = error("A * B");
    assert_eq!(
        (e.found, e.expected),
        (None, &["a connective", "`,`", "`|-`"][..])
    );
    // Inside a parenthesis only a connective or its end can follow.
    let e = error("|- (A * B");
    assert_eq!((e.found, e.expected), (None, &["a connective", "`)`"][..]));
    // A quantifier's keyword names no atom: its variables must follow.
    let e = error("exists |- é");
    assert_eq!(
        (&e.span, &e.span_utf16, e.reserved),
        (&(7..8), &(7..8), false)
    );
    assert_eq!(
        e.to_string(),
        "unexpected \"|\" at line 1, character 8, expected a variable name"
    );
}

/// Neither the nesting of a formula nor the length of a chain costs any
/// stack: formulas 50 000 deep parse on a thread with a small one, to a
/// sequent of as many subformula occurrences as the text has.
#[test]
fn depth_costs_no_stack() {
    /// How many connectives each formula has: the stack of 256 KiB holds
    /// about five bytes for each, less than a call frame even where one
    /// frame serves two of them.
    const N: usize = 50_000;
    let work = || {
        let chain = |operator: &str| format!("|- {}a", format!("a {operator} ").repeat(N));
        for (shape, input, occurrences) in [
            (
                "nested to the right",
                format!("|- {}a{}", "a * (".repeat(N), ")".repeat(N)),
                2 * N + 1,
            ),
            (
                "nested to the left",
                format!("|- {}a{}", "(".repeat(N), " * a)".repeat(N)),
                2 * N + 1,
            ),
            ("a chain of tensors", chain("*"), 2 * N + 1),
            ("a chain of lollipops", chain("-o"), 2 * N + 1),
            ("prefix operators", format!("|- {}a", "!".repeat(N)), N + 1),
            ("postfix operators", format!("|- a{}", "^".repeat(N)), 1),
            (
                "on the left of the turnstile",
                format!("{}a{} |-", "!(a -o ".repeat(N), ")".repeat(N)),
                3 * N + 1,
            ),
        ] {
            let sequent: Sequent = input
                .parse()
                .unwrap_or_else(|e| panic!("{shape} does not parse: {e}"));
            assert_eq!(sequent.occurrences(), occurrences as u64, "{shape}");
        }
    };
    std::thread::Builder::new()
        .stack_size(256 * 1024)
        .spawn(work)
        .unwrap()
        .join()
        .unwrap();
}

/// Quantifiers and predicates: `forall x. A` and `exists x. A` (`∀`, `∃`,
/// several variables at once) bind loosest and to the right, a predicate
/// takes its arguments in brackets, an identifier bound by an enclosing
/// binder is its variable and any other a constant, a negation swaps the
/// binders, and every sequent prints as text that reads back as itself.
#[test]
fn quantifiers_and_predicates() {
    for (input, printed) in [
        ("forall x. p(x) |- p(a)", "⊢ ∃x. ~p(x), p(a)"),
        ("|- exists x. p(x) -o q(x)", "⊢ ∃x. (~p(x) ⅋ q(x))"),
        (
            "∀x. p(x) * q |- ∃y. p(y) * q",
            "⊢ ∃x. (~p(x) ⅋ ~q), ∃y. (p(y) ⊗ q)",
        ),
        (
            "|- ∀x y. ~r(x, f(y)) ⅋ r(x, f(y))",
            "⊢ ∀x. (∀y. (~r(x, f(y)) ⅋ r(x, f(y))))",
        ),
        ("|- (forall x. p(x)) * q", "⊢ (∀x. p(x)) ⊗ q"),
        (
            "|- ~forall x. p(x), (exists y. p(y))^",
            "⊢ ∃x. ~p(x), ∀y. ~p(y)",
        ),
        ("|- !forall x. p(x, c)", "⊢ !(∀x. p(x, c))"),
        // A bound name that a constant or an enclosing binder takes is
        // written apart, so that the text reads back as the formula.
        ("forall x. p(x) |- p(x)", "⊢ ∃x_1. ~p(x_1), p(x)"),
        ("|- forall x. forall x. p(x)", "⊢ ∀x. (∀x_1. p(x_1))"),
        // Two binders over one shape are one term, whatever their names.
        (
            "forall x. p(x), forall y. p(y) |- q",
            "⊢ ∃x. ~p(x), ∃x. ~p(x), q",
        ),
        // Atoms read before the first binder or predicate get their
        // predicates then.
        ("q |- forall x. p(x)", "⊢ ~q, ∀x. p(x)"),
        ("a * forall x. b, c |- d(e)", "⊢ ~a ⅋ (∃x. ~b), ~c, d(e)"),
        // A shadowed name is written apart from its enclosing binder's.
        (
            "|- forall x. forall x. forall x. r(x)",
            "⊢ ∀x. (∀x_1. (∀x_2. r(x_2)))",
        ),
    ] {
        let s: Sequent = input.parse().unwrap_or_else(|e| panic!("{input:?}: {e}"));
        assert_eq!(s.to_string(), printed, "{input:?}");
        let back: Sequent = printed
            .parse()
            .unwrap_or_else(|e| panic!("{printed:?}: {e}"));
        assert_eq!(back.to_string(), printed, "{input:?} read back");
        assert_eq!(
            (
                back.terms(),
                back.roots(),
                back.atom_names(),
                back.symbols()
            ),
            (s.terms(), s.roots(), s.atom_names(), s.symbols()),
            "{input:?} read back"
        );
        assert!(s.fragment().has_quantifiers(), "{input:?}");
    }
    let s: Sequent = "forall x. p(x), forall y. p(y) |- q".parse().unwrap();
    assert_eq!(s.terms().len(), 3);
    assert_eq!(s.fragment().to_string(), "MLL1");
    // Ground atoms are propositional: no binder, no quantifier bit, and
    // an atom's key is its text.
    let ground: Sequent = "p(a, f(b)) |- p(a, f(b))".parse().unwrap();
    assert_eq!(ground.fragment().to_string(), "MLL");
    assert!(ground.is_first_order());
    assert_eq!(ground.atom_names(), ["p(a, f(b))"]);
    assert_eq!(ground.symbols().len(), 4);
    let open: Sequent = "|- forall x. p(x, a)".parse().unwrap();
    assert_eq!(open.atom_names(), ["p(#0, a)"]);
    assert!(open.atom_is_open(linlog::Atom::new(0)));
    assert!(!ground.atom_is_open(linlog::Atom::new(0)));
    assert_eq!(open.binder_name(linlog::TermId::new(1)), Some("x"));
}

/// What is no first-order sequent is refused at its place with a hint: a
/// symbol with another number of arguments than before, a name used as a
/// predicate and as a function symbol, a keyword in a binder's or a term's
/// place, a binder without its dot, a numeral in term position.
#[test]
fn first_order_errors() {
    for (input, at, found, hint) in [
        (
            "p(a), p |- p(a)",
            6,
            Some("p"),
            "a symbol takes the same number",
        ),
        (
            "p(a) |- p(a, b)",
            8,
            Some("p(a, b)"),
            "a symbol takes the same number",
        ),
        (
            "p(f(a)) |- f(p)",
            13,
            Some("p"),
            "a symbol takes the same number",
        ),
        (
            "p(f(a)), f |- q",
            9,
            Some("f"),
            "a symbol takes the same number",
        ),
        (
            "f(a) |- q(f(b))",
            10,
            Some("f(b)"),
            "a name is a predicate or a function",
        ),
        (
            "|- forall top. p(x)",
            10,
            Some("top"),
            "a keyword names no variable",
        ),
        ("|- p(par)", 5, Some("par"), "a keyword names no variable"),
        // Without the dot, `p` is one more variable.
        ("|- forall x p(x)", 13, Some("("), ""),
        ("|- p(0)", 5, Some("0"), ""),
        ("|- p(a, )", 8, Some(")"), ""),
        ("|- p(a", 6, None, ""),
    ] {
        let Err(Error::Parse(error)) = input.parse::<Sequent>() else {
            panic!("{input:?} is no parse error");
        };
        assert_eq!(
            (error.span.start, error.found.as_deref()),
            (at, found),
            "{input:?}"
        );
        if hint.is_empty() {
            assert_eq!(error.hint, None, "{input:?}");
        } else {
            assert!(
                error.hint.is_some_and(|h| h.starts_with(hint)),
                "{input:?}: {:?}",
                error.hint
            );
        }
    }
    // The strict intuitionistic syntax reads the same binders.
    let ill: linlog::IllSequent = "forall x. p(x) -o q(x), p(a) |- exists y. q(y)"
        .parse()
        .unwrap();
    assert_eq!(ill.to_string(), "∀x. (p(x) ⊸ q(x)), p(a) ⊢ ∃y. q(y)");
}
