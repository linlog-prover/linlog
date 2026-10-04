// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The focused engine's tests.

use super::*;
use crate::Sequent;
use crate::search::generate::{self, Rng, Rules};
use crate::search::{Verdict, prove_goal};

/// Runs the focused engine on the roots of the forest in the fragment
/// and the mode given, two-sided in intuitionistic mode (the reading is
/// the forest's, which the front door reads itself), polling `stop`, and
/// returns the verdict with the statistics; the proof unchecked, as the
/// tests check it themselves.
fn search(
    forest: &Forest,
    fragment: Fragment,
    mode: Mode,
    reading: Option<&Reading>,
    options: &Options,
    stop: &mut dyn FnMut() -> bool,
) -> (Verdict, Statistics) {
    assert_eq!(reading.is_some(), mode.intuitionistic);
    let engine = if mode.intuitionistic {
        crate::search::Engine::TwoSided
    } else {
        crate::search::Engine::Focus
    };
    let options = options
        .clone()
        .engine(Some(engine))
        .fragment(Some(fragment))
        .check(false);
    let outcome = prove_goal(forest, forest.roots(), mode, &options, stop)
        .unwrap_or_else(|e| panic!("{}: {e}", forest.sequent()));
    (outcome.verdict, outcome.statistics)
}

/// Runs the engine on `input` under `mode` with `options`, checks the
/// proof if there is one, and returns the verdict and the statistics.
fn run(input: &str, mode: Mode, options: &Options) -> (Verdict, Statistics) {
    let s: Sequent = input.parse().unwrap_or_else(|e| panic!("{input:?}: {e}"));
    let forest = Forest::new(&s).unwrap();
    let reading = mode.intuitionistic.then(|| {
        Reading::new(&forest).unwrap_or_else(|e| panic!("{input:?}: {}", e.describe(&forest)))
    });
    let (verdict, statistics) = search(
        &forest,
        s.fragment(),
        mode,
        reading.as_ref(),
        options,
        &mut || false,
    );
    if let Verdict::Proved(proof) = &verdict {
        assert_eq!(proof.sequent(), &s);
        proof
            .check(mode)
            .unwrap_or_else(|e| panic!("{input:?}: the proof is wrong: {e}"));
    }
    (verdict, statistics)
}

/// Forty literals over atoms of their own, as the rest of a sequent.
pub(crate) fn wide_context() -> String {
    let literals: Vec<String> = (0..40).map(|i| format!("x{i}")).collect();
    literals.join(", ")
}

/// Whether `input` is provable under `mode`, panicking on `Unknown`.
fn provable(input: &str, mode: Mode) -> bool {
    match run(input, mode, &Options::default()).0 {
        Verdict::Proved(_) => true,
        Verdict::Unprovable(_) => false,
        Verdict::Unknown(reason) => panic!("{input:?}: {reason}"),
    }
}

/// The classic small sequents, in `MLL` and `MALL`, get the verdicts
/// the textbooks give them.
#[test]
fn classic_sequents() {
    let m = Mode::CLASSICAL;
    for (input, expected) in [
        ("|- ~a, a", true),
        ("a |- a", true),
        ("|- a, a", false),
        ("|-", false),
        ("a * b |- a * b", true),
        ("|- a * b, ~a par ~b", true),
        ("|- a * b, ~a, ~b", true),
        ("|- a par b, ~a, ~b", false),
        ("|- (a * b) par (~a * ~b)", false),
        ("a, a -o b, b -o c |- c", true),
        ("a -o b, b -o c |- a -o c", true),
        ("a -o b |- b -o a", false),
        ("|- a & b, ~a + ~b", true),
        ("|- a + b, ~a", true),
        ("|- a & b, ~a", false),
        ("(a & b) + (a & c) |- a & (b + c)", true),
        ("a & (b + c) |- (a & b) + (a & c)", false),
        ("a * (b + c) |- (a * b) + (a * c)", true),
        ("(a * b) + (a * c) |- a * (b + c)", true),
        ("a & b |- a", true),
        ("a |- a & b", false),
        ("a |- a + b", true),
        ("a + b |- a", false),
    ] {
        assert_eq!(provable(input, m), expected, "{input:?}");
    }
}

/// The units behave as the spec's checks say, and `⊤` closes any
/// context while `0` closes none.
#[test]
fn units() {
    let m = Mode::CLASSICAL;
    for (input, expected) in [
        ("|- 1", true),
        ("|- bot, 1", true),
        ("|- 1 * 1", true),
        ("|- bot par 1", true),
        ("|- bot par bot", false),
        ("|- bot", false),
        ("|- 1, 1", false),
        ("|- 1, a, ~a", false),
        ("|- a * 1, ~a", true),
        ("|- top", true),
        ("|- top, a", true),
        ("|- top * a, ~a, b", true),
        ("|- a * top, b", false),
        ("|- a & top, ~a", true),
        ("|- 0", false),
        ("|- 0, top", true),
        ("|- 0, top + b", true),
        ("|- 0 * a, ~a, top + b", true),
        ("|- 0 + a, ~a", true),
        ("|- 0 * a, ~a", false),
        ("0 |- a", true),
        ("a |- top", true),
    ] {
        assert_eq!(provable(input, m), expected, "{input:?}");
    }
}

/// Mix proves exactly what needs it.
#[test]
fn mix() {
    let mix = Mode::CLASSICAL.with_mix();
    for (input, without, with) in [
        ("|- a par b, ~a, ~b", false, true),
        ("a * b |- a par b", false, true),
        ("|- a, ~a, b, ~b", false, true),
        ("|- 1, a, ~a", false, true),
        ("|- 1, 1", false, true),
        ("|- a * b, ~a, ~b", true, true),
        ("|- a, b", false, false),
        ("|- a + b, ~a, c, ~c", false, true),
        ("|- (a & b) par c, ~a, ~b, ~c", false, false),
        ("|- a & b, ~a, ~b", false, false),
        ("|-", false, false),
    ] {
        assert_eq!(provable(input, Mode::CLASSICAL), without, "{input:?}");
        assert_eq!(provable(input, mix), with, "{input:?} with Mix");
    }
}

/// The memo makes no difference to the verdicts, and the counters say
/// what the search did.
#[test]
fn memo_and_statistics() {
    let (verdict, statistics) = run("|- ~a, a", Mode::CLASSICAL, &Options::default());
    assert!(verdict.proof().is_some());
    assert_eq!(statistics.nodes, 1);
    assert_eq!(statistics.memo_hits, 0);
    assert_eq!(statistics.memo_entries, 1);
    assert_eq!(statistics.splits, 0);

    // ⊢ ~a ⊕ ~b, ~a ⊕ ~b, a ⊗ b, (x ⅋ ~x) ⊗ (y ⊗ ~y) is unprovable,
    // and the two orders of choosing the `⊕` sides reach the stable
    // sequent ⊢ ~a, ~b, a ⊗ b, (x ⅋ ~x) ⊗ (y ⊗ ~y), whose refutation
    // takes a few stable sequents; the memo answers it the second time.
    let input = "|- ~a + ~b, ~a + ~b, a * b, (x par ~x) * (y * ~y)";
    let with_memo = run(input, Mode::CLASSICAL, &Options::default());
    let without = run(input, Mode::CLASSICAL, &Options::default().memo_limit(0));
    assert!(matches!(with_memo.0, Verdict::Unprovable(_)));
    assert!(matches!(without.0, Verdict::Unprovable(_)));
    assert!(with_memo.1.memo_hits >= 1);
    assert_eq!(without.1.memo_hits, 0);
    assert_eq!(without.1.memo_entries, 0);
    assert!(without.1.nodes > with_memo.1.nodes);
}

/// The same input and options give the same proof, node for node.
#[test]
fn deterministic() {
    let input = "|- (a * b) + (a * c), ~a par (~b & ~c), a, ~a";
    let first = run(input, Mode::CLASSICAL.with_mix(), &Options::default());
    let second = run(input, Mode::CLASSICAL.with_mix(), &Options::default());
    assert_eq!(
        first.0.proof().unwrap().nodes(),
        second.0.proof().unwrap().nodes()
    );
    assert_eq!(first.1, second.1);
}

/// The stop condition and the recursion limit each end the search with
/// their reason, and no context is too wide to split.
#[test]
fn limits() {
    let input = "|- a * b, ~a, ~b";
    let s: Sequent = input.parse().unwrap();
    let forest = Forest::new(&s).unwrap();
    let mut calls = 0;
    let (verdict, _) = search(
        &forest,
        s.fragment(),
        Mode::CLASSICAL,
        None,
        &Options::default(),
        &mut || {
            calls += 1;
            calls >= 1
        },
    );
    assert!(matches!(verdict, Verdict::Unknown(Reason::Stopped)));

    let (verdict, _) = run(
        input,
        Mode::CLASSICAL,
        &Options::default().recursion_limit(2),
    );
    assert!(matches!(verdict, Verdict::Unknown(Reason::RecursionLimit)));
    let (verdict, _) = run(
        input,
        Mode::CLASSICAL,
        &Options::default().recursion_limit(8),
    );
    assert!(verdict.proof().is_some());

    // A `?` costs no level of recursion: three thousand of them stay
    // under the default limit.
    let hypotheses: Vec<String> = (0..3000).map(|i| format!("?a{i}")).collect();
    let many = format!("|- 1, {}", hypotheses.join(", "));
    let (verdict, _) = run(&many, Mode::CLASSICAL, &Options::default());
    assert!(verdict.proof().is_some());

    // Nor does a link of a tensor of literals, nested to the left as
    // it is read: every split is forced by the literal on the right.
    // On a thread with the stack a front end gives the search, since
    // the parser and the checker recurse to the formula's depth.
    let literals = format!(
        "|- {}, {}",
        vec!["a"; 2500].join(" * "),
        vec!["~a"; 2500].join(", ")
    );
    // The same for a tensor of tensors of literals, which is proved
    // in place from the duals of all its literals.
    let pairs = format!(
        "|- {}, {}",
        vec!["(a * b)"; 2500].join(" * "),
        vec!["~a, ~b"; 2500].join(", ")
    );
    let proved = std::thread::Builder::new()
        .stack_size(Options::default().stack_size())
        .spawn(move || {
            [literals, pairs].iter().all(|chain| {
                let (verdict, _) = run(chain, Mode::CLASSICAL, &Options::default());
                verdict.proof().is_some()
            })
        })
        .unwrap()
        .join()
        .unwrap();
    assert!(proved);

    // A free split over 126 formulas, in MALL so that the count
    // equation does not refute it first: no width is too much, and the
    // counts of `b` refute every split at once.
    let wide = format!(
        "|- ((a & a) par b) * (~a par ~b), {}",
        (0..63).map(|_| "~a, a").collect::<Vec<_>>().join(", ")
    );
    let (verdict, _) = run(&wide, Mode::CLASSICAL, &Options::default());
    assert!(matches!(verdict, Verdict::Unprovable(_)));
}

/// A sequent whose search comes back to it with other occurrences of
/// the same formulas is refuted, not left at the copy bound by its own
/// failure of the level before: a failure cut by the budget answers
/// for its own sequent alone.
#[test]
fn repeats_up_to_equal_members() {
    for input in [
        "|- ~b, (1 * a), ?(b * ((a par ~b) * ~a))",
        "a, a, b, b, !(((b * a) * a) -o b), !(((b * a) * a) -o b) |- ((b * b) * b)",
    ] {
        assert!(!provable(input, Mode::CLASSICAL), "{input:?}");
    }
    // Here the relative is reached on another branch.
    let sibling = "!(!!(b -o c) -o !c), b |- (c * ((a -o a) * c))";
    assert!(!provable(sibling, Mode::INTUITIONISTIC));
}

/// The bias never changes what is provable, only what a copy bound
/// allows: the counter with eight tokens is proved within three copies
/// a branch chaining backward, and needs seven chaining forward, where
/// it visits a fraction of the stable sequents; the default finds the
/// forward proof.
#[test]
fn bias_option() {
    let (sequent, copies) = crate::families::counter(8, false);
    let forest = Forest::new(&sequent).unwrap();
    let run = |options: &Options| {
        let fragment = sequent.fragment();
        search(
            &forest,
            fragment,
            Mode::CLASSICAL,
            None,
            options,
            &mut || false,
        )
    };
    let backward = Options::default().bias(Bias::Rarer).copies(Some(copies));
    let (verdict, slow) = run(&backward);
    assert!(verdict.proof().is_some());
    let forward = backward.clone().bias(Bias::Factors);
    let (verdict, _) = run(&forward);
    assert!(matches!(verdict, Verdict::Unknown(Reason::CopyBound(_))));
    let (verdict, fast) = run(&forward.copies(Some(7)));
    let proof = verdict.proof().expect("seven steps on one branch");
    assert_eq!(proof.check(Mode::CLASSICAL), Ok(()));
    assert!(fast.nodes * 10 < slow.nodes, "{fast:?} against {slow:?}");
    // The default runs both: the clauses are Horn, so the forward
    // search has its own bound and its proof comes first, at its cost.
    let (verdict, default) = run(&Options::default().copies(Some(copies)));
    assert_eq!(verdict.proof().unwrap().check(Mode::CLASSICAL), Ok(()));
    assert_eq!(default, fast);
}

/// Without a copy bound the search goes on to the next level until it
/// decides, and says which level that was; with one it ends there.
#[test]
fn an_unbounded_search_deepens() {
    let (sequent, _) = crate::families::counter(8, false);
    let forest = Forest::new(&sequent).unwrap();
    let run = |options: &Options| {
        let fragment = sequent.fragment();
        let mode = Mode::CLASSICAL;
        search(&forest, fragment, mode, None, options, &mut || false)
    };
    let forward = Options::default().bias(Bias::Factors);
    let (verdict, statistics) = run(&forward.clone().copies(Some(3)));
    assert!(matches!(verdict, Verdict::Unknown(Reason::CopyBound(3))));
    assert_eq!(statistics.copies, 3);
    let (verdict, statistics) = run(&forward.copies(None));
    assert!(verdict.proof().is_some(), "seven steps on one branch");
    assert_eq!(statistics.copies, 7);
}

/// The two searches of the default bias on one core. In turns from
/// their start, on work that grows: here the backward search decides,
/// after the forward one used up turns and then its bound. And, with
/// threads, alternating in slices, where each search is the one it is
/// alone and none starts again.
#[test]
fn default_bias_takes_turns() {
    // The counter with eight tokens takes seven steps forward, more than
    // the forward bound here; the clauses that move a token between
    // places give the forward search markings to visit until then.
    let (mut clauses, marking, goal) = counter(8);
    for (body, head) in [
        ("a", "x"),
        ("x", "y"),
        ("y", "a"),
        ("a", "u"),
        ("u", "v"),
        ("v", "a"),
        ("b", "w"),
        ("w", "b"),
        ("x", "u"),
    ] {
        clauses.push((body.to_string(), head.to_string()));
    }
    let clauses: Vec<(&str, &str)> = clauses.iter().map(|(b, h)| (&**b, &**h)).collect();
    let text = horn(&clauses, &marking, &[goal]);
    let m = Mode::CLASSICAL;
    let options = Options::default().forward_copies(4);
    let (verdict, backward) = run(&text, m, &options.clone().bias(Bias::Rarer));
    assert!(verdict.proof().is_some());
    let (verdict, forward) = run(
        &text,
        m,
        &options.clone().bias(Bias::Factors).copies(Some(4)),
    );
    assert!(matches!(verdict, Verdict::Unknown(Reason::CopyBound(4))));

    let sequent: Sequent = text.parse().unwrap();
    let forest = Forest::new(&sequent).unwrap();
    let classes = Classes::new(&forest, None);
    let in_turns = || {
        let (first, second) = plan(&forest, forest.roots(), sequent.fragment(), m, &options);
        let second = second.expect("two searches");
        let counts = [first, second].map(|rule| Counts::new(&forest, rule.bias));
        let account = Account::new(None);
        let (result, _, statistics) = turns(
            &forest,
            forest.roots(),
            sequent.fragment(),
            m,
            None,
            &classes,
            &options,
            [
                (first, &counts[0], &account),
                (second, &counts[1], &account),
            ],
            &mut || false,
        );
        assert!(matches!(result, Ok(Some(_))));
        statistics
    };
    let both = in_turns();
    // What the turns cut short cost: less than a third of the turn
    // the forward search ends in for itself, and at most that turn and
    // the ones before it for the backward search, which then runs
    // alone.
    let cut = both.nodes - forward.nodes - backward.nodes;
    assert!(
        0 < cut && cut < 7 * forward.nodes,
        "{cut} stable sequents in the turns cut short, {forward:?}, {backward:?}"
    );
    // One thread, so the run is a function of the input.
    assert_eq!(in_turns(), both);

    #[cfg(feature = "parallel")]
    {
        let (verdict, slices) = run(&text, m, &options);
        assert!(verdict.proof().is_some());
        assert_eq!(slices.nodes, forward.nodes + backward.nodes);
        assert_eq!(slices.splits, forward.splits + backward.splits);
        assert_eq!(run(&text, m, &options).1, slices);
    }
}

/// Under a memory bound that the memo does not fit, the memo is
/// emptied whenever it reaches the bound and the search still decides,
/// in more stable sequents; under a bound that leaves the memo no room
/// the search gives up and names the bound.
#[test]
fn memory_bound() {
    let (sequent, copies) = crate::families::counter(8, false);
    let text = sequent.to_string();
    let options = Options::default().copies(Some(copies)).bias(Bias::Rarer);
    let bounded = |bytes| {
        run(
            &text,
            Mode::CLASSICAL,
            &options.clone().memory_limit(Some(bytes)),
        )
    };
    let (verdict, whole) = run(&text, Mode::CLASSICAL, &options);
    assert!(verdict.proof().is_some());
    let (verdict, tight) = bounded(8 << 10);
    assert!(verdict.proof().is_some());
    assert!(
        tight.memo_entries < whole.memo_entries && tight.nodes > whole.nodes,
        "{tight:?} within 8 KiB, {whole:?} without a bound that binds"
    );
    let (verdict, _) = bounded(1 << 10);
    assert!(matches!(
        verdict,
        Verdict::Unknown(Reason::MemoryLimit(1024))
    ));
}

/// A memo of a few entries is emptied at nearly every insertion, and
/// the kept proofs are collected as often: whatever such a search
/// proves is a proof the checker accepts (an id that a collection
/// moved and no one renamed would make it none), and its verdict never
/// contradicts the one a memo of the default size gives. A search is
/// given up after so many polls: with a memo this small some take
/// minutes.
#[test]
fn proofs_survive_collections() {
    let mut proved = 0;
    let within = |text: &str, mode: Mode, options: &Options| {
        let s: Sequent = text.parse().unwrap();
        let forest = Forest::new(&s).unwrap();
        let reading = mode.intuitionistic.then(|| Reading::new(&forest).unwrap());
        let mut polls = 0;
        let (verdict, _) = search(
            &forest,
            s.fragment(),
            mode,
            reading.as_ref(),
            options,
            &mut || {
                polls += 1;
                polls > 20_000
            },
        );
        match verdict {
            Verdict::Proved(proof) => {
                proof
                    .check(mode)
                    .unwrap_or_else(|e| panic!("{text:?}: the proof is wrong: {e}"));
                Some(true)
            }
            Verdict::Unprovable(_) => Some(false),
            Verdict::Unknown(_) => None,
        }
    };
    let mut check = |text: &str, mode: Mode, options: &Options| {
        for bias in [Bias::Rarer, Bias::Factors] {
            let options = options.clone().bias(bias);
            let whole = within(text, mode, &options);
            for limit in [1, 2, 5] {
                let small = within(text, mode, &options.clone().memo_limit(limit));
                assert!(
                    small.is_none() || whole.is_none() || small == whole,
                    "{text:?} in {mode} mode: {small:?} with a memo of {limit}, {whole:?}"
                );
                proved += u64::from(small == Some(true));
            }
        }
    };
    for (i, rules) in Rules::ALL.into_iter().enumerate() {
        let mut rng = Rng::new(700 + i as u64);
        for _ in 0..20 {
            let generate::Provable {
                mut formulas,
                copies,
            } = generate::provable(&mut rng, rules, 3, 8);
            let options = Options::default().copies(Some(copies));
            check(&generate::sequent(&formulas), mode_for(rules), &options);
            if generate::mutate(&mut rng, &mut formulas, 3) {
                check(&generate::sequent(&formulas), mode_for(rules), &options);
            }
        }
    }
    for (n, rules) in generate::IllRules::ALL.into_iter().enumerate() {
        let mut rng = Rng::new(800 + n as u64);
        for _ in 0..20 {
            let generate::Ill {
                hypotheses,
                goal,
                copies,
            } = generate::ill(&mut rng, rules, 3, 8);
            let options = Options::default().copies(Some(copies));
            let text = generate::two_sided(&hypotheses, &goal);
            check(&text, Mode::INTUITIONISTIC, &options);
        }
    }
    assert!(proved > 1000, "{proved} proofs");
}

/// The contract of the default bias: on a sequent with exponentials it
/// decides whatever the backward or the forward search decides under
/// the same options, with the same verdict, with and without the memo,
/// classically and intuitionistically.
#[test]
fn default_bias_decides_what_either_rule_does() {
    let mut compared = 0;
    let mut check = |text: &str, mode: Mode, options: &Options| {
        let both = decided(text, mode, options);
        for bias in [Bias::Rarer, Bias::Factors] {
            let one = decided(text, mode, &options.clone().bias(bias));
            assert!(
                one.is_none() || one == both,
                "{text:?} in {mode} mode: {one:?} under {bias:?}, {both:?} by default"
            );
            compared += u64::from(one.is_some());
        }
    };
    for (i, rules) in Rules::ALL.into_iter().enumerate() {
        if !rules.exponentials {
            continue;
        }
        let mut rng = Rng::new(200 + i as u64);
        for _ in 0..30 {
            let generate::Provable {
                mut formulas,
                copies,
            } = generate::provable(&mut rng, rules, 3, 8);
            let options = Options::default().copies(Some(copies));
            check(&generate::sequent(&formulas), mode_for(rules), &options);
            if generate::mutate(&mut rng, &mut formulas, 3) {
                let text = generate::sequent(&formulas);
                check(&text, mode_for(rules), &Options::default());
                // Without the memo a refutation under Mix takes seconds.
                if !rules.mix {
                    check(&text, mode_for(rules), &Options::default().memo_limit(0));
                }
            }
        }
    }
    for (n, rules) in generate::IllRules::ALL.into_iter().enumerate() {
        let mut rng = Rng::new(300 + n as u64);
        for _ in 0..30 {
            let generate::Ill {
                mut hypotheses,
                goal,
                copies,
            } = generate::ill(&mut rng, rules, 3, 8);
            let options = Options::default().copies(Some(copies));
            let text = generate::two_sided(&hypotheses, &goal);
            check(&text, Mode::INTUITIONISTIC, &options);
            hypotheses.push(goal);
            if generate::mutate(&mut rng, &mut hypotheses, 3) {
                let goal = hypotheses.pop().unwrap();
                let text = generate::two_sided(&hypotheses, &goal);
                check(&text, Mode::INTUITIONISTIC, &Options::default());
            }
        }
    }
    assert!(compared > 1000, "{compared} verdicts compared");
}

/// A split search whose splits all fail in focus visits no stable
/// sequent and still stops when asked: with weakening no count cuts
/// the 2⁴² splits of this context, and each fails at once, since no
/// member is the `~p` its left side wants.
#[test]
fn stops_inside_a_split_search() {
    let input = format!(
        "|- p * q, 0 * (~p par ~p), 0 * (~q par ~q), {}",
        wide_context()
    );
    let s: Sequent = input.parse().unwrap();
    let forest = Forest::new(&s).unwrap();
    let mut polls = 0;
    let (verdict, statistics) = search(
        &forest,
        s.fragment(),
        Mode::CLASSICAL.affine(),
        None,
        &Options::default(),
        &mut || {
            polls += 1;
            polls > 3
        },
    );
    assert!(matches!(verdict, Verdict::Unknown(Reason::Stopped)));
    assert_eq!(statistics.nodes, 1);
    assert!(statistics.splits <= 4 * SPLITS_PER_POLL, "{statistics:?}");
}

/// A chain of forced splits visits no stable sequent and still stops
/// when asked, whether its factors are literals, one split each, or
/// tensors of literals closed in place: each tensor of 5 000 factors
/// is one chain from the first stable sequent, whose poll is the
/// first, and the second comes inside the chain.
#[test]
fn stops_inside_a_forced_chain() {
    let chains = [
        format!(
            "|- {}, {}",
            vec!["a"; 5000].join(" * "),
            vec!["~a"; 5000].join(", ")
        ),
        format!(
            "|- {}, {}",
            vec!["(a * b)"; 5000].join(" * "),
            vec!["~a, ~b"; 5000].join(", ")
        ),
    ];
    for input in chains {
        let (verdict, statistics) = std::thread::Builder::new()
            .stack_size(Options::default().stack_size())
            .spawn(move || {
                let s: Sequent = input.parse().unwrap();
                let forest = Forest::new(&s).unwrap();
                let mut polls = 0;
                search(
                    &forest,
                    s.fragment(),
                    Mode::CLASSICAL,
                    None,
                    &Options::default(),
                    &mut || {
                        polls += 1;
                        polls > 1
                    },
                )
            })
            .unwrap()
            .join()
            .unwrap();
        assert!(matches!(verdict, Verdict::Unknown(Reason::Stopped)));
        assert_eq!(statistics.nodes, 1);
        assert!(statistics.splits <= FORCED_PER_POLL, "{statistics:?}");
    }
}

/// Intuitionistic mode: the textbook sequents of ILL, the pitfalls of
/// the spec (`0` on the left proves anything, `⊤` on the left is inert,
/// promotion needs an empty linear context) and the sequent classical
/// linear logic proves but intuitionistic linear logic does not.
#[test]
fn intuitionistic() {
    let i = Mode::INTUITIONISTIC;
    for (input, expected) in [
        ("a |- a", true),
        ("|- a -o a", true),
        ("a, a -o b |- b", true),
        ("a -o b, b -o c |- a -o c", true),
        ("a -o b |- b -o a", false),
        ("a * b |- b * a", true),
        ("a * b |- a", false),
        ("a |- a * a", false),
        ("(a * b) -o c |- a -o b -o c", true),
        ("a -o b -o c |- (a * b) -o c", true),
        ("a & b |- a", true),
        ("a & b |- a * b", false),
        ("a * b |- a & b", false),
        ("a |- a + b", true),
        ("a + b |- a", false),
        ("a + b |- b + a", true),
        ("a & (b + c) |- (a & b) + (a & c)", false),
        ("a * (b + c) |- (a * b) + (a * c)", true),
        ("(a -o b) -o a |- a", false),
        ("|- ((a -o b) -o a) -o a", false),
        ("(a -o 0) -o 0 |- a", false),
        ("|- 1", true),
        ("1 |- 1", true),
        ("a |- 1", false),
        ("1, a |- a", true),
        ("|- top", true),
        ("a |- top", true),
        ("a |- 0", false),
        // `0` on the left proves anything, `⊤` on the left is inert.
        ("0 |- a", true),
        ("0, b |- a", true),
        ("a -o 0, a |- b", true),
        ("a -o 0 |- a -o b", true),
        ("top |- a", false),
        ("top |- top", true),
        ("a, top |- a", false),
        ("top, 0 |- a", true),
        ("a -o top, a |- b", false),
        // Ambiguous roots read with the last as the goal: ⊤ ⊢ ⊤.
        ("|- 0, top", true),
        // Promotion needs an empty linear context.
        ("!a |- !a", true),
        ("a |- !a", false),
        ("!a, b |- !a", false),
        ("!a, !b |- !a", true),
        ("!a |- a * a", true),
        ("!a, !(a -o b) |- !b", true),
        ("!(a & b) |- !a * !b", true),
        ("!a * !b |- !(a & b)", true),
        ("!a, !(a -o b & c) |- b * c", true),
        ("!(a -o top), a |- b", false),
        ("!a, !(a -o 0) |- b", true),
    ] {
        assert_eq!(provable(input, i), expected, "{input:?}");
    }
    // Classical linear logic is not conservative over ILL with `0`: the
    // classical proof splits the `⊸L` with the goal on the antecedent's
    // side, which the two-sided search never does.
    let schellinx = "((a * top) & (b * top)) -o 0 |- (a -o c) + (b -o c)";
    assert!(provable(schellinx, Mode::CLASSICAL));
    assert!(!provable(schellinx, i));
    // Affine mode weakens hypotheses at the leaves, never the goal, and
    // lets promotion discard the linear context.
    for (input, expected) in [
        ("a, b |- a", true),
        ("a |- b", false),
        ("a |- 1", true),
        ("!a, b |- !a", true),
        ("a, b |- a * b", true),
        ("top |- a", false),
        ("a, top |- a", true),
        ("a & b |- a", true),
        ("!(a -o a * a), a |- a * a * a", true),
    ] {
        assert_eq!(provable(input, i.affine()), expected, "{input:?}");
    }
}

/// The classic MELL sequents: dereliction, weakening, contraction with
/// and without promotion, and what is unprovable, all within the
/// default copy bound.
#[test]
fn classic_exponentials() {
    let m = Mode::CLASSICAL;
    for (input, expected) in [
        ("!a |- a", true),
        ("!a |- 1", true),
        ("!a |- a * a", true),
        ("!a |- !a * !a", true),
        ("|- !(a -o a)", true),
        ("!a, !(a -o b) |- !b", true),
        ("!a, !a |- a", true),
        ("!a |- !!a", true),
        ("!!a |- !a", true),
        ("!a * !b |- !(a * b)", true),
        ("!a, !(a -o b), !(b -o c) |- !c", true),
        ("|- ?a, ?~a", true),
        ("!a, !~a |- 1", true),
        ("?a |- ?a par ?a", true),
        ("|- !1", true),
        ("a |- !a", false),
        ("?a |- a", false),
        ("!a |- b", false),
        ("!a, !(a -o b) |- c", false),
        ("|- ?a", false),
        ("|- ?a, ?b, ~a", true),
        ("|- ?a, b, ~a", false),
        ("|- 0, ?top", true),
        ("|- ~b, 0 par (c * (0 + a)), ?(top + b)", true),
        ("|- 0, ?a", false),
    ] {
        assert_eq!(provable(input, m), expected, "{input:?}");
    }
    // Unprovable, but every copy grows the context, so no level of the
    // bound finishes: the honest answer is unknown.
    for input in [
        "!(a * b) |- !a * !b",
        "!(a + b) |- !a + !b",
        "!(a * b) |- a",
    ] {
        assert!(
            matches!(
                run(input, m, &Options::default()).0,
                Verdict::Unknown(Reason::CopyBound(3))
            ),
            "{input:?}"
        );
    }
}

/// Full LL: the additive and the dyadic rules compose, with and
/// without Mix.
#[test]
fn full_ll() {
    for (input, expected) in [
        ("!(a & b) |- !a * !b", true),
        ("!a * !b |- !(a & b)", true),
        ("?a par ?b |- ?(a + b)", true),
        ("?(a + b) |- ?a par ?b", true),
        ("!(a & b) |- !a & !b", true),
        ("!(a & b) |- !(a * b)", true),
        ("!a & !b |- !(a & b)", false),
        ("!a, !(a -o b & c) |- b * c", true),
        ("!(a -o top) |- b", false),
        ("!(a -o top), a |- b", false),
        ("!(a -o top), a |- top", true),
        ("!(0 -o a) |- a", false),
        ("!a, !(a -o 0) |- b", true),
    ] {
        assert_eq!(provable(input, Mode::CLASSICAL), expected, "{input:?}");
    }
    // The opponent chooses the side of the `&`: unprovable, and the
    // context grows with every copy, so undecided within the bound.
    assert!(matches!(
        run(
            "!a, !(a -o b + c) |- b * c",
            Mode::CLASSICAL,
            &Options::default()
        )
        .0,
        Verdict::Unknown(Reason::CopyBound(3))
    ));
    let mix = Mode::CLASSICAL.with_mix();
    for (input, without, with) in [
        ("!a |- a, 1", false, true),
        ("!a, !b |- a * b, a", false, true),
        ("!a |- a, b", false, false),
    ] {
        assert_eq!(provable(input, Mode::CLASSICAL), without, "{input:?}");
        assert_eq!(provable(input, mix), with, "{input:?} with Mix");
    }
}

/// Affine mode proves what needs weakening and nothing more, in every
/// fragment, and decides what linear mode cannot.
#[test]
fn affine() {
    let affine = Mode::CLASSICAL.affine();
    for (input, linear, weakened) in [
        ("a |- 1", false, true),
        ("a, b |- a", false, true),
        ("a * b |- a", false, true),
        ("|- a, ~a, b", false, true),
        ("a & b |- 1", false, true),
        ("a |- 0 + 1", false, true),
        ("|- 0, a, ~a", false, true),
        ("!a, b |- a", false, true),
        ("a |- !1", false, true),
        ("a |- !b", false, false),
        ("|- a, b", false, false),
        ("|- 0", false, false),
        ("a * b |- a * b", true, true),
        ("a |- a * a", false, false),
        ("!a |- a * a", true, true),
        ("a & b |- a + b", true, true),
        ("(a * b) par c |- a, b, c", false, true),
        ("a -o b |- b", false, false),
    ] {
        assert_eq!(provable(input, Mode::CLASSICAL), linear, "{input:?}");
        assert_eq!(provable(input, affine), weakened, "{input:?} affinely");
    }
    // The context grows with every copy, so both modes give up at the
    // bound: a sequent that contains an ancestor is not redundant, and
    // affine mode is bounded like linear mode.
    let growing = "!(a -o a * a), a |- ?b";
    for mode in [Mode::CLASSICAL, affine] {
        assert!(matches!(
            run(growing, mode, &Options::default()).0,
            Verdict::Unknown(Reason::CopyBound(3))
        ));
    }
    // A sequent proved only through a larger one above it.
    for input in [
        "|- ?(a par ~a)",
        "|- ?!1",
        "!(a * ~a) |-",
        "|- ?(a par ~a), b",
    ] {
        assert!(provable(input, affine), "{input:?}");
    }
    // Weakening goes below a promotion, never above it.
    let (verdict, _) = run("b |- !(a -o a)", affine, &Options::default());
    let proof = verdict.proof().unwrap();
    let Node::Weaken(_, below) = proof.node(proof.root()) else {
        panic!("the leftover is weakened at the root");
    };
    assert!(matches!(proof.node(below), Node::Bang(..)));
}

/// The copy bound of one search, the backward one here: a level that
/// hit its bound never answers `Unprovable`, a level that did not
/// answers it, a failure recorded at a smaller remaining budget is not
/// reused at a larger one, and the bound is per branch.
#[test]
fn copy_bound() {
    let m = Mode::CLASSICAL;
    let with = |copies| Options::default().bias(Bias::Rarer).copies(Some(copies));
    // ⊢ ?~a, a needs one copy: bound 0 is hit, bound 1 proves.
    assert!(matches!(
        run("!a |- a", m, &with(0)).0,
        Verdict::Unknown(Reason::CopyBound(0))
    ));
    assert!(run("!a |- a", m, &with(1)).0.proof().is_some());
    // ⊢ ?~a, a ⊗ a: the stable sequent ⊢ ~a ; a fails at level 0 for
    // lack of budget and must be searched again at level 1.
    assert!(run("!a |- a * a", m, &with(1)).0.proof().is_some());
    // The clause and one `~a` are copied on every branch to an `a`:
    // bound 2, however many branches there are; and the two branches
    // of ⊢ ?~a, (a ⊗ a) ⊗ a take one copy each.
    assert!(matches!(
        run("!a, !(a -o a -o a -o b) |- b", m, &with(1)).0,
        Verdict::Unknown(Reason::CopyBound(1))
    ));
    assert!(
        run("!a, !(a -o a -o a -o b) |- b", m, &with(2))
            .0
            .proof()
            .is_some()
    );
    assert!(run("!a |- (a * a) * a", m, &with(1)).0.proof().is_some());
    // Unprovable, decided at a level that never hit the bound: after
    // one copy of ~a nothing is left to copy. (⊢ ?~a, b is refuted by
    // the balance of b before any copy.)
    assert!(matches!(
        run("!a |- b", m, &with(0)).0,
        Verdict::Unprovable(_)
    ));
    assert!(matches!(
        run("!a |- ?b", m, &with(1)).0,
        Verdict::Unknown(Reason::CopyBound(1))
    ));
    assert!(matches!(
        run("!a |- ?b", m, &with(2)).0,
        Verdict::Unprovable(_)
    ));
    // The loop check decides a sequent whose copies repeat a stable
    // sequent, without a bound to hit.
    assert!(matches!(
        run("!(a -o a), a |- b", m, &with(1)).0,
        Verdict::Unprovable(_)
    ));
    // A growing context is never decided within a bound.
    for copies in [0, 2, 5] {
        assert!(matches!(
            run("!(a -o a * a), a |- ?b", m, &with(copies)).0,
            Verdict::Unknown(Reason::CopyBound(c)) if c == copies
        ));
    }
    assert_eq!(
        Reason::CopyBound(3).to_string(),
        "the copy bound of 3 was reached"
    );
}

/// The memo makes no difference to a MELL verdict, with and without
/// the copy bound binding, and the entries survive across levels.
#[test]
fn memo_across_levels() {
    for input in [
        "!a, !(a -o b), !(b -o c) |- c * c * c",
        "!(a -o a * a), a |- b",
        "!a, !(a -o b) |- c",
        "!(a & b) |- !a * !b",
    ] {
        for copies in [1, 3] {
            let options = Options::default().copies(Some(copies));
            let (with, stats) = run(input, Mode::CLASSICAL, &options);
            let (without, _) = run(input, Mode::CLASSICAL, &options.clone().memo_limit(0));
            assert_eq!(
                std::mem::discriminant(&with),
                std::mem::discriminant(&without),
                "{input:?} at {copies} copies: {with:?} with the memo, {without:?} without"
            );
            if copies == 3 && input.contains("c * c * c") {
                assert!(with.proof().is_some());
                assert!(stats.memo_hits > 0, "{input:?}: the memo was used");
            }
        }
    }
}

/// The dyadic proofs the engine finds unfold into the standard
/// derivations, with dereliction, contraction, weakening and promotion
/// where they belong.
#[test]
fn exponential_derivations() {
    let render = |input: &str, mode: Mode| {
        let (verdict, _) = run(input, mode, &Options::default());
        verdict.proof().unwrap().derivation().unwrap().to_string()
    };
    assert_eq!(
        render("!A |- A", Mode::CLASSICAL),
        ["─────── ax", "⊢ ~A, A", "──────── ?d", "⊢ ?~A, A"].join("\n")
    );
    assert_eq!(
        render("!A |- 1", Mode::CLASSICAL),
        ["  ─── 1", "  ⊢ 1", "──────── ?w", "⊢ ?~A, 1"].join("\n")
    );
    assert_eq!(
        render("!A |- A * A", Mode::CLASSICAL),
        [
            "─────── ax    ─────── ax",
            "⊢ ~A, A       ⊢ ~A, A",
            "──────── ?d   ──────── ?d",
            "⊢ ?~A, A      ⊢ ?~A, A",
            "────────────────────── ⊗",
            "  ⊢ ?~A, ?~A, A ⊗ A",
            "  ───────────────── ?c",
            "    ⊢ ?~A, A ⊗ A",
        ]
        .join("\n")
    );
    assert_eq!(
        render("!A, !(A -o B) |- !B", Mode::CLASSICAL),
        [
            "─────── ax",
            "⊢ ~A, A",
            "──────── ?d   ─────── ax",
            "⊢ ?~A, A      ⊢ ~B, B",
            "───────────────────── ⊗",
            "  ⊢ ?~A, A ⊗ ~B, B",
            " ─────────────────── ?d",
            " ⊢ ?~A, ?(A ⊗ ~B), B",
            " ──────────────────── !",
            " ⊢ ?~A, ?(A ⊗ ~B), !B",
        ]
        .join("\n")
    );
    assert_eq!(
        render("A, B |- A", Mode::CLASSICAL.affine()),
        ["  ─────── ax", "  ⊢ ~A, A", "─────────── wk", "⊢ ~A, ~B, A",].join("\n")
    );
}

/// The mode a generated sequent is proved in.
fn mode_for(rules: Rules) -> Mode {
    if rules.mix {
        Mode::CLASSICAL.with_mix()
    } else {
        Mode::CLASSICAL
    }
}

/// The verdict of `input` under `mode` with `options`, as a three-way
/// value: provable, unprovable, or undecided within the copy bound.
fn decided(input: &str, mode: Mode, options: &Options) -> Option<bool> {
    match run(input, mode, options).0 {
        Verdict::Proved(_) => Some(true),
        Verdict::Unprovable(_) => Some(false),
        Verdict::Unknown(Reason::CopyBound(_)) => None,
        Verdict::Unknown(reason) => panic!("{input:?}: {reason}"),
    }
}

/// Proves `samples` generated sequents of up to `budget` rules per rule
/// set with or without exponentials, each within the derelictions of
/// its proof, and decides one mutant of each with and without the memo.
/// Returns how many sequents and mutants were decided, how many mutants
/// were provable, how many were undecided within the copy bound, and
/// the most stable sequents one search visited.
fn generated(samples: u64, budget: usize, exponentials: bool) -> (u64, u64, u64, u64, u64) {
    let (mut sequents, mut mutants, mut provable_mutants, mut undecided, mut most_nodes) =
        (0, 0, 0, 0, 0);
    for (i, rules) in Rules::ALL.into_iter().enumerate() {
        if rules.exponentials != exponentials {
            continue;
        }
        let mode = mode_for(rules);
        let mut rng = Rng::new(i as u64);
        for _ in 0..samples {
            let budget = 2 + rng.below(budget - 1);
            let generate::Provable {
                mut formulas,
                copies,
            } = generate::provable(&mut rng, rules, 3, budget);
            let text = generate::sequent(&formulas);
            let options = Options::default().copies(Some(copies));
            let (verdict, statistics) = run(&text, mode, &options);
            assert!(
                verdict.proof().is_some(),
                "{text:?} is provable in {mode} mode within {copies} copies, but the engine \
                 says {verdict:?}"
            );
            // A linear proof is an affine proof.
            let (affine, _) = run(&text, mode.affine(), &options);
            assert!(
                affine.proof().is_some(),
                "{text:?} is provable in {} mode within {copies} copies, but the engine \
                 says {affine:?}",
                mode.affine()
            );
            sequents += 1;
            most_nodes = most_nodes.max(statistics.nodes);
            if generate::mutate(&mut rng, &mut formulas, 3) {
                let text = generate::sequent(&formulas);
                let with_memo = decided(&text, mode, &Options::default());
                let without = decided(&text, mode, &Options::default().memo_limit(0));
                // The memo never contradicts the memo-free search; it
                // may decide where the other is undecided within the
                // bound and the other way round, since an entry cut by
                // the budget is a fact about the sequent alone while
                // the loop check is a fact about the branch.
                assert!(
                    with_memo.is_none() || without.is_none() || with_memo == without,
                    "{text:?} in {mode} mode: {with_memo:?} with the memo, {without:?} without"
                );
                mutants += 1;
                provable_mutants += u64::from(with_memo == Some(true));
                undecided += u64::from(with_memo.is_none());
            }
        }
    }
    (sequents, mutants, provable_mutants, undecided, most_nodes)
}

/// Every generated provable sequent is proved with a checked proof
/// within the derelictions of its proof, in every fragment with and
/// without Mix, and its mutant is decided the same way with and
/// without the memo.
#[test]
fn generated_sequents() {
    let (sequents, mutants, _, undecided, _) = generated(40, 10, false);
    assert_eq!(sequents, 320);
    assert!(mutants > 200, "{mutants} mutants");
    assert_eq!(undecided, 0);
    let (sequents, mutants, _, undecided, _) = generated(40, 10, true);
    assert_eq!(sequents, 320);
    assert!(mutants > 200, "{mutants} mutants");
    assert!(
        undecided < mutants / 4,
        "{undecided} of {mutants} mutants undecided"
    );
}

/// Proves `samples` generated intuitionistic sequents of up to `budget`
/// rules per rule set, each within the derelictions of its proof, in
/// linear and affine mode; compares the two-sided verdict on each with
/// the classical engine's on the same one-sided sequent, which must
/// agree without `0`; and on a mutant of each. Returns how many
/// sequents and mutants were compared and how many mutants the classical
/// engine proved that the two-sided one refuted, which only `0` allows.
fn generated_ill(samples: u64, budget: usize) -> (u64, u64, u64) {
    use crate::search::generate::IllRules;
    let i = Mode::INTUITIONISTIC;
    let (mut sequents, mut mutants, mut non_conservative) = (0, 0, 0);
    for (n, rules) in IllRules::ALL.into_iter().enumerate() {
        let mut rng = Rng::new(100 + n as u64);
        for _ in 0..samples {
            let budget = 2 + rng.below(budget - 1);
            let generate::Ill {
                mut hypotheses,
                goal,
                copies,
            } = generate::ill(&mut rng, rules, 3, budget);
            let text = generate::two_sided(&hypotheses, &goal);
            let options = Options::default().copies(Some(copies));
            let (verdict, _) = run(&text, i, &options);
            assert!(
                verdict.proof().is_some(),
                "{text:?} is provable in ILL within {copies} copies, but the engine says \
                 {verdict:?}"
            );
            let (affine, _) = run(&text, i.affine(), &options);
            assert!(affine.proof().is_some(), "{text:?} affine: {affine:?}");
            sequents += 1;
            // The classical engine on the same one-sided sequent.
            let classical = decided(&text, Mode::CLASSICAL, &options);
            assert!(
                classical != Some(false),
                "{text:?} is provable in ILL, so classically too, but the engine says \
                 {classical:?}"
            );
            hypotheses.push(goal);
            if generate::mutate(&mut rng, &mut hypotheses, 3) {
                let goal = hypotheses.pop().unwrap();
                let text = generate::two_sided(&hypotheses, &goal);
                let two_sided = decided(&text, i, &Options::default());
                let classical = decided(&text, Mode::CLASSICAL, &Options::default());
                match (two_sided, classical) {
                    (Some(true), Some(false)) => panic!("{text:?}: provable in ILL only"),
                    (Some(false), Some(true)) => {
                        assert!(rules.zero, "{text:?}: provable classically only");
                        non_conservative += 1;
                    }
                    _ => {}
                }
                mutants += 1;
            }
        }
    }
    (sequents, mutants, non_conservative)
}

/// Every generated intuitionistic sequent is proved two-sided with a
/// checked proof within the derelictions of its proof, in every
/// intuitionistic fragment, and the classical engine agrees on the
/// mutants except where `0` makes classical linear logic prove more.
#[test]
fn generated_intuitionistic_sequents() {
    let (sequents, mutants, non_conservative) = generated_ill(40, 10);
    assert_eq!(sequents, 480);
    assert!(mutants > 300, "{mutants} mutants");
    assert!(
        non_conservative < mutants / 10,
        "{non_conservative} of {mutants}"
    );
}

/// The same on a larger sample of larger proofs, without exponentials:
/// with them, a few sequents of a sample this size take minutes at the
/// bound their derelictions give. Run it in release mode and read the
/// numbers it prints.
#[test]
#[ignore = "a larger sample; run with --release -- --ignored --nocapture"]
fn generated_large_sample() {
    let start = std::time::Instant::now();
    let (sequents, mutants, provable_mutants, _, most_nodes) = generated(500, 24, false);
    println!(
        "{sequents} generated sequents proved, {mutants} mutants decided consistently \
         ({provable_mutants} of them provable), at most {most_nodes} stable sequents per \
         search, in {:.2?}",
        start.elapsed()
    );
}

/// Encodes a Horn program with reusable clauses, as the ILLTP library
/// states Petri-net reachability: every clause `body ⊸ head` (products
/// of atoms) under a `!`, the initial marking as hypotheses, the goal
/// marking as the conclusion.
fn horn(clauses: &[(&str, &str)], marking: &[&str], goal: &[&str]) -> String {
    let mut hypotheses: Vec<String> = clauses
        .iter()
        .map(|(body, head)| format!("!({body} -o {head})"))
        .collect();
    hypotheses.extend(marking.iter().map(|a| (*a).to_string()));
    format!("{} |- {}", hypotheses.join(", "), goal.join(" * "))
}

/// The counter program: `n` tokens `a`, two `a` make a `b`, two `b` a
/// `c`, and so on up the alphabet, with `n` a power of two.
fn counter(n: usize) -> (Vec<(String, String)>, Vec<&'static str>, &'static str) {
    let levels = n.trailing_zeros() as usize;
    let names = ["a", "b", "c", "d", "e", "f"];
    let clauses = (0..levels)
        .map(|i| {
            (
                format!("{} * {}", names[i], names[i]),
                names[i + 1].to_string(),
            )
        })
        .collect();
    (clauses, vec!["a"; n], names[levels])
}

/// Horn problems over reusable clauses: a chain of implications takes
/// one copy per clause on one branch, the counter program reaches its
/// goal within the copies its firings take and not below, an
/// unreachable marking is undecided within the bound in either mode,
/// and affine mode reaches a goal that leaves a token over.
#[test]
fn horn_programs() {
    let m = Mode::CLASSICAL;
    let chain: Vec<(String, String)> = (0..6)
        .map(|i| (format!("x{i}"), format!("x{}", i + 1)))
        .collect();
    let chain: Vec<(&str, &str)> = chain.iter().map(|(b, h)| (&**b, &**h)).collect();
    let text = horn(&chain, &["x0"], &["x6"]);
    assert!(
        run(&text, m, &Options::default().copies(Some(6)))
            .0
            .proof()
            .is_some()
    );
    // Five copies are too few for either search alone; the default's
    // forward search has a bound of its own on Horn clauses.
    for bias in [Bias::Rarer, Bias::Factors] {
        assert!(matches!(
            run(&text, m, &Options::default().bias(bias).copies(Some(5))).0,
            Verdict::Unknown(Reason::CopyBound(5))
        ));
    }
    assert!(
        run(&text, m, &Options::default().copies(Some(5)))
            .0
            .proof()
            .is_some()
    );
    assert!(matches!(
        run(
            &text,
            m,
            &Options::default().copies(Some(5)).forward_copies(0)
        )
        .0,
        Verdict::Unknown(Reason::CopyBound(5))
    ));

    let (clauses, marking, goal) = counter(4);
    let clauses: Vec<(&str, &str)> = clauses.iter().map(|(b, h)| (&**b, &**h)).collect();
    assert!(provable(&horn(&clauses, &marking, &[goal]), m));
    // A goal no firing reaches: the backward search is cut at its
    // bound, the forward one runs out of markings within its own.
    assert!(matches!(
        run(
            &horn(&clauses, &marking, &[goal, "a"]),
            m,
            &Options::default().bias(Bias::Rarer)
        )
        .0,
        Verdict::Unknown(Reason::CopyBound(3))
    ));
    assert!(matches!(
        run(
            &horn(&clauses, &marking, &[goal, "a"]),
            m,
            &Options::default()
        )
        .0,
        Verdict::Unprovable(_)
    ));
    assert!(matches!(
        run(
            &horn(&clauses, &marking, &[goal, "a"]),
            m.affine(),
            &Options::default()
        )
        .0,
        Verdict::Unknown(Reason::CopyBound(3))
    ));
    let mut five = marking.clone();
    five.push("a");
    assert!(matches!(
        run(&horn(&clauses, &five, &[goal]), m, &Options::default()).0,
        Verdict::Unprovable(_)
    ));
    assert!(provable(&horn(&clauses, &five, &[goal]), m.affine()));
}

/// A 3-Partition instance with a solution is proved: the first bin
/// choices work out, so the search is short.
#[test]
fn three_partition_solved() {
    let yes = crate::families::three_partition(&[1, 2, 3, 1, 2, 3], 2, 6).to_string();
    assert!(provable(&yes, Mode::CLASSICAL), "{yes}");
}

/// The Horn test reads the goal, not the forest's roots: a goal of the
/// clause, the marking and the goal of a program is one, inside a sequent
/// whose other hypothesis, a `&`, is none, so the forward search gets
/// its own bound there and not on the whole sequent.
#[test]
fn the_horn_test_reads_the_goal() {
    let sequent: Sequent = "!(a -o b), a, c & d |- b".parse().unwrap();
    let forest = Forest::new(&sequent).unwrap();
    let roots = forest.roots();
    let program: Vec<OccId> = roots
        .iter()
        .copied()
        .filter(|&r| forest.kind(r) != Kind::Plus)
        .collect();
    assert_eq!(program.len(), 3);
    assert!(!schedule::chains(&forest, roots));
    assert!(schedule::chains(&forest, &program));
    let options = Options::default().forward_copies(30);
    let forward = |goal: &[OccId]| {
        plan(&forest, goal, sequent.fragment(), Mode::CLASSICAL, &options)
            .0
            .copies
    };
    assert_eq!(forward(roots), Options::DEFAULT_COPIES);
    assert_eq!(forward(&program), 30);
}

/// Under Mix, a sequent none of whose parts is provable is refuted by its
/// parts with one member less: the twelve members of `mix(6)` cost at
/// most `12 · 2¹²` stable sequents, where searching the partitions of
/// every part visited 175 100, a third of `3¹²`.
#[test]
fn unprovable_parts_cost_no_partitions() {
    let sequent = crate::families::mix(6);
    let forest = Forest::new(&sequent).unwrap();
    let (verdict, statistics) = search(
        &forest,
        sequent.fragment(),
        Mode::CLASSICAL.with_mix(),
        None,
        &Options::default(),
        &mut || false,
    );
    assert!(matches!(verdict, Verdict::Unprovable(_)), "{verdict:?}");
    assert!(statistics.nodes <= 12 << 12, "{statistics:?}");
}
