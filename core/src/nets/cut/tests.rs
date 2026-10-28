// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use super::*;
use crate::nets::{Criterion, MellStructure, Placement};
use crate::occurrences::Forest;
use crate::proofs::cut as terms;
use crate::proofs::{CutFormula, Interactive, Node, Proof};
use crate::search::generate::{self, IllRules, Rng, Rules};
use crate::search::{Engine, Verdict};
use crate::{InputOptions, Mode, Sequent, ViewOptions};
use std::collections::BTreeMap;

/// The most steps a test takes on one net before it calls the
/// elimination unfinished.
const MOST_STEPS: usize = 5_000;

/// Never stops.
fn never(_: Progress) -> bool {
    false
}

/// What the tests ask of both kinds of net.
trait Tested: Net + std::fmt::Debug {
    /// The net of a proof.
    fn of(proof: &Proof, criterion: Criterion) -> Result<Self, Error>;
    /// Its sequentialization.
    fn proof(&self) -> Proof;
}

impl Tested for ProofStructure {
    fn of(proof: &Proof, criterion: Criterion) -> Result<Self, Error> {
        Self::from_proof(proof, criterion, &Limits::default(), never)
    }

    fn proof(&self) -> Proof {
        self.sequentialize(&Limits::default(), never).unwrap()
    }
}

impl Tested for MellStructure {
    fn of(proof: &Proof, criterion: Criterion) -> Result<Self, Error> {
        Self::from_proof(proof, criterion, &Limits::default(), never)
    }

    fn proof(&self) -> Proof {
        self.sequentialize(&Limits::default(), never).unwrap()
    }
}

/// Returns a proof of the sequent `text` in `mode` with a cut on
/// `formula`, typed as the mode writes formulas, whose first premise
/// takes the members at `left`; both premises closed by the search.
fn cut_proof(text: &str, mode: Mode, formula: &str, left: &[usize]) -> Proof {
    let limits = Limits::default();
    let sequent: Sequent = text.parse().unwrap();
    let mut session = Interactive::new(&sequent, mode).unwrap();
    let goal = session.goals().next().unwrap();
    let formula =
        CutFormula::parse_within(formula, mode, &InputOptions::default(), &limits).unwrap();
    session.cut(goal, &formula, left, &limits).unwrap();
    let closed = session.close_all(
        &crate::Options::default(),
        &ViewOptions::default(),
        &limits,
        never,
    );
    assert!(
        closed
            .iter()
            .all(|(_, c)| c.as_ref().is_ok_and(|c| c.grafted.is_ok())),
        "{text}: {closed:?}"
    );
    session.proof(&limits, never).unwrap()
}

/// Proves a generated sequent within `copies` copies and a bound of polls,
/// `None` unless it found a proof.
fn proof_of(text: &str, mode: Mode, copies: u32) -> Option<(Proof, Mode)> {
    let sequent: Sequent = text.parse().unwrap();
    let engine = (!mode.is_intuitionistic()).then_some(Engine::Focus);
    let options = crate::Options::default()
        .with_copies(Some(copies))
        .with_engine(engine);
    let mut polls = 0u32;
    let outcome = crate::search::prove_within(&sequent, mode, &options, &Limits::default(), |_| {
        polls += 1;
        polls > 20_000
    })
    .ok()?;
    match outcome.verdict {
        Verdict::Proved(proof) => Some((*proof, mode)),
        Verdict::Unprovable(_) | Verdict::Unknown(_) => None,
    }
}

/// The proofs with cuts the tests take the nets of: proofs of generated
/// sequents of unit-free MLL and MELL, classical with and without Mix and
/// intuitionistic, each cut on one of its formulas and one of the two
/// goals cut again, and proofs made here for the exponential cases.
fn proofs() -> Vec<(Proof, Mode)> {
    let mut rng = Rng::new(4103);
    let mut plain = Vec::new();
    for (mix, exponentials) in [(false, false), (true, false), (false, true), (true, true)] {
        let rules = Rules {
            units: false,
            additives: false,
            mix,
            exponentials,
        };
        let mode = if mix {
            Mode::CLASSICAL.with_mix()
        } else {
            Mode::CLASSICAL
        };
        for _ in 0..60 {
            let budget = 2 + rng.below(9);
            let provable = generate::provable(&mut rng, rules, 3, budget);
            let text = generate::sequent(&provable.formulas);
            plain.extend(proof_of(&text, mode, provable.copies));
        }
    }
    for exponentials in [false, true] {
        let rules = IllRules {
            units: false,
            additives: false,
            zero: false,
            exponentials,
        };
        for _ in 0..60 {
            let budget = 2 + rng.below(9);
            let ill = generate::ill(&mut rng, rules, 3, budget);
            let text = generate::two_sided(&ill.hypotheses, &ill.goal);
            plain.extend(proof_of(&text, Mode::INTUITIONISTIC, ill.copies));
        }
    }
    let mut proofs = crate::proofs::interactive::with_cuts(&plain, &mut rng);
    // Two copies of the box's formula: a contraction, then a dereliction;
    // a box's content using it: a box entering a box; a formula the
    // premise never uses: weakening.
    for text in [
        "!a |- a * a",
        "!a |- !a",
        "!a, b |- b",
        "!a, !a -o b |- b * a",
        "!a, !b |- !(a * b) * a",
    ] {
        let mode = Mode::INTUITIONISTIC;
        proofs.push((cut_proof(text, mode, "!a", &[0]), mode));
    }
    for mode in [Mode::CLASSICAL, Mode::CLASSICAL.with_mix()] {
        for (text, rest) in [
            ("|- ?~a, a * a", &[1][..]),
            ("|- ?~a, !a", &[1]),
            ("|- ?~a, ~b, b", &[1, 2]),
            ("|- ?~a, ?~b, !(a * b) * a", &[1, 2]),
        ] {
            proofs.push((cut_proof(text, mode, "!a", &[0]), mode));
            proofs.push((cut_proof(text, mode, "?~a", rest), mode));
        }
    }
    proofs
}

/// Returns the number of cut nodes of a proof.
fn cuts_of(proof: &Proof) -> usize {
    proof
        .nodes()
        .iter()
        .filter(|n| matches!(n, Node::Cut(..)))
        .count()
}

/// Checks that a net is a proof net and sequentializes into a proof of
/// the sequent of `proof`, cut formulas included, with its conclusion and
/// as many cuts as the net has cut links, which the checker accepts in
/// `mode`.
fn sound<N: Tested>(net: &N, proof: &Proof, mode: Mode) {
    assert_eq!(net.check(&mut never), Ok(()), "{net:?}");
    let back = net.proof();
    assert_eq!(back.check(mode), Ok(()), "{mode:?}: {net:?}");
    assert_eq!(back.sequent(), proof.sequent());
    assert_eq!(back.conclusion(), proof.conclusion());
    assert_eq!(cuts_of(&back), net.cut_links().len(), "{net:?}");
}

/// What the eliminations showed.
#[derive(Default)]
struct Seen {
    /// Proofs by mode.
    modes: BTreeMap<String, usize>,
    /// Steps by case.
    cases: BTreeMap<&'static str, usize>,
    /// Proofs with two cuts or more.
    nested: usize,
    /// Proofs with a cut on an exponential formula.
    exponential: usize,
    /// Normal forms compared with the term elimination's, without boxes
    /// and with.
    compared: [usize; 2],
}

/// Desequentializes a proof with cuts, eliminates its net's cut links
/// step by step by the strategy, checks every net reached as [`sound`]
/// does, checks that the end is cut-free and that [`eliminate`] takes the
/// same steps to the same net, showing each to `each`, and returns the
/// normal form.
fn eliminated<N: Tested>(proof: &Proof, mode: Mode, seen: &mut Seen) -> N {
    let criterion = Criterion::of(mode).unwrap();
    let net = N::of(proof, criterion).unwrap_or_else(|e| panic!("{e} in {mode:?}: {proof:?}"));
    assert_eq!(net.cut_links().len(), cuts_of(proof));
    sound(&net, proof, mode);
    let (mut current, mut steps) = (net.clone(), Vec::new());
    while !current.cut_links().is_empty() {
        assert!(steps.len() < MOST_STEPS, "no end in {mode:?}: {net:?}");
        let reduction = step(
            &current,
            None,
            &Options::default(),
            &Limits::default(),
            never,
        )
        .unwrap_or_else(|e| panic!("{e} in {mode:?} at {current:?}"));
        sound(&reduction.net, proof, mode);
        assert_eq!(reduction.step.cut, current.cut_links()[0]);
        *seen.cases.entry(reduction.step.case.name()).or_default() += 1;
        steps.push(reduction.step);
        current = reduction.net;
    }
    let mut visited = Vec::new();
    let elimination = eliminate(
        &net,
        &Options::default(),
        &Limits::default(),
        never,
        |_, step| visited.push(*step),
    )
    .unwrap();
    assert_eq!(elimination.end, End::CutFree);
    assert_eq!(elimination.steps, steps);
    assert_eq!(visited, steps);
    assert_eq!(
        format!("{:?}", elimination.net.proof().nodes()),
        format!("{:?}", current.proof().nodes())
    );
    current
}

/// Every proof with cuts has a net with its cut links, which the
/// criterion accepts and which sequentializes into a proof with as many
/// cuts that the checker accepts in the proof's mode; every step of its
/// elimination makes such a net of the same conclusion, the elimination
/// ends without cut links as `eliminate` does in the same steps, and
/// every case of nets is met. Where the theory says the two eliminations
/// agree, the normal form is the net of the term elimination's: in MLL,
/// with or without Mix, the same links, since cut-free MLL nets are the
/// canonical form of proofs up to rule permutations; with boxes, the
/// same net up to the order of copies and the links the weakenings hang
/// on, where no occurrence has two vertices, the only nets the reading of
/// a term determines (two vertices of one occurrence leave open which a
/// rule of the term acts on).
#[test]
fn every_step_keeps_the_criterion_and_the_conclusion() {
    let mut seen = Seen::default();
    for (proof, mode) in proofs() {
        let forest = proof.forest();
        let cuts: Vec<Node> = proof
            .nodes()
            .iter()
            .copied()
            .filter(|n| matches!(n, Node::Cut(..)))
            .collect();
        seen.nested += usize::from(cuts.len() > 1);
        seen.exponential += usize::from(cuts.iter().any(|&node| {
            matches!(node, Node::Cut(a, ..)
                if matches!(forest.kind(a.occ()), Kind::Bang | Kind::Quest))
        }));
        *seen.modes.entry(format!("{mode:?}")).or_default() += 1;
        let normal = terms::eliminate(
            &proof,
            mode,
            &terms::Options::default(),
            &Limits::default(),
            never,
        )
        .unwrap();
        assert_eq!(normal.end, End::CutFree);
        let criterion = Criterion::of(mode).unwrap();
        if proof.sequent().fragment().has_nets() {
            let net: ProofStructure = eliminated(&proof, mode, &mut seen);
            let term = ProofStructure::of(&normal.proof, criterion).unwrap();
            let sorted = |net: &ProofStructure| {
                let mut links: Vec<_> = net
                    .links()
                    .iter()
                    .map(|&(x, y)| (x.min(y), x.max(y)))
                    .collect();
                links.sort_unstable();
                links
            };
            assert_eq!(sorted(&net), sorted(&term), "{mode:?}: {proof:?}");
            seen.compared[0] += 1;
        } else {
            let net: MellStructure = eliminated(&proof, mode, &mut seen);
            let term = MellStructure::of(&normal.proof, criterion).unwrap();
            let once = |net: &MellStructure| {
                let mut met = vec![false; net.forest().len()];
                net.vertices().all(|v| {
                    let o = net.occurrence(v).unwrap().index();
                    !std::mem::replace(&mut met[o], true)
                })
            };
            if once(&net) && once(&term) {
                assert!(
                    crate::nets::mell::tests::isomorphic(&net, &term),
                    "{mode:?}: {net:?}\nagainst the term's\n{term:?}"
                );
                seen.compared[1] += 1;
            }
        }
    }
    let Seen {
        modes,
        cases,
        nested,
        exponential,
        compared,
    } = seen;
    for case in [
        Case::Axiom,
        Case::TensorPar,
        Case::Weakening,
        Case::Dereliction,
        Case::Contraction,
        Case::BoxIntoBox,
    ] {
        assert!(
            cases.get(case.name()).is_some_and(|&k| k > 5),
            "{case:?}: {cases:?}"
        );
    }
    assert!(
        modes.len() == 3 && modes.values().all(|&k| k > 40),
        "{modes:?}"
    );
    assert!(
        nested > 150 && exponential > 20 && compared[0] > 150 && compared[1] > 50,
        "{nested} {exponential} {compared:?} {cases:?} {modes:?}"
    );
}

/// Every cut link of a net may be reduced first, not only the
/// strategy's, and the step names it from either end.
#[test]
fn any_cut_link_may_be_reduced() {
    /// Reduces the cut link at `at` of the net of `proof`, checks the net
    /// made and returns the cut link the step names.
    fn reduced<N: Tested>(proof: &Proof, mode: Mode, at: VertexId) -> (VertexId, VertexId) {
        let net = N::of(proof, Criterion::of(mode).unwrap()).unwrap();
        let r = step(
            &net,
            Some(at),
            &Options::default(),
            &Limits::default(),
            never,
        )
        .unwrap();
        sound(&r.net, proof, mode);
        r.step.cut
    }
    let mut steps = 0;
    for (proof, mode) in proofs().into_iter().step_by(3) {
        let criterion = Criterion::of(mode).unwrap();
        let mll = proof.sequent().fragment().has_nets();
        let cuts = if mll {
            ProofStructure::of(&proof, criterion)
                .unwrap()
                .cuts()
                .to_vec()
        } else {
            MellStructure::of(&proof, criterion)
                .unwrap()
                .cuts()
                .to_vec()
        };
        for (i, &(x, y)) in cuts.iter().enumerate() {
            let at = if i % 2 == 0 { x } else { y };
            let named = if mll {
                reduced::<ProofStructure>(&proof, mode, at)
            } else {
                reduced::<MellStructure>(&proof, mode, at)
            };
            assert_eq!(named, (x, y), "{mode:?}: {proof:?}");
            steps += 1;
        }
    }
    assert!(steps > 100, "{steps}");
}

/// A step names a cut link by an end, and a vertex at none, or a net
/// without one, is refused.
#[test]
fn a_step_needs_a_cut_link() {
    let proof = cut_proof("|- ?~a, a * a", Mode::CLASSICAL, "!a", &[0]);
    let net = MellStructure::of(&proof, Criterion::MLL).unwrap();
    let options = Options::default();
    let refused = step(
        &net,
        Some(VertexId::new(0)),
        &options,
        &Limits::default(),
        never,
    );
    assert!(matches!(
        refused,
        Err(Error::Net(e)) if *e == NetError::NoCut { vertex: Some(VertexId::new(0)) }
    ));
    let normal = eliminate(&net, &options, &Limits::default(), never, |_, _| {}).unwrap();
    let refused = step(&normal.net, None, &options, &Limits::default(), never);
    assert!(matches!(refused, Err(Error::Net(e)) if *e == NetError::NoCut { vertex: None }));
}

/// A bound ends an elimination with a net the criterion accepts that
/// still has its cut links: the steps, the vertices (one below the
/// largest net the elimination of a contraction reaches) and the stop.
#[test]
fn a_bound_stops_with_a_net() {
    let proof = cut_proof("|- ?~a, !(a * a) * a", Mode::CLASSICAL, "!a", &[0]);
    let net = MellStructure::of(&proof, Criterion::MLL).unwrap();
    let mut largest = 0;
    let full = eliminate(
        &net,
        &Options::default(),
        &Limits::default(),
        never,
        |net, _| {
            largest = largest.max(net.len());
        },
    )
    .unwrap();
    assert_eq!(full.end, End::CutFree);
    let largest = largest.max(full.net.len()) as u64;
    let options = Options::default().with_steps(Some(1));
    let one = eliminate(&net, &options, &Limits::default(), never, |_, _| {}).unwrap();
    assert_eq!(
        one.end,
        End::Refused(Refusal::Elimination {
            what: Bound::Steps,
            limit: 1
        })
    );
    assert_eq!(one.steps.len(), 1);
    let options = Options::default().with_nodes(Some(largest - 1));
    let few = eliminate(&net, &options, &Limits::default(), never, |_, _| {}).unwrap();
    assert_eq!(
        few.end,
        End::Refused(Refusal::Elimination {
            what: Bound::Vertices,
            limit: largest - 1
        })
    );
    for ended in [&one, &few] {
        assert_ne!(ended.net.cuts(), []);
        sound(&ended.net, &proof, Mode::CLASSICAL);
    }
    let mut stopped_with_cuts = 0;
    for polls in 1..40 {
        let mut asked = 0;
        let stopped = eliminate(
            &net,
            &Options::default(),
            &Limits::default(),
            |_| {
                asked += 1;
                asked > polls
            },
            |_, _| {},
        );
        match stopped {
            Ok(e) if e.end == End::CutFree => {}
            Ok(e) => {
                assert!(matches!(e.end, End::Refused(Refusal::Stopped { .. })));
                assert_ne!(e.net.cuts(), []);
                sound(&e.net, &proof, Mode::CLASSICAL);
                stopped_with_cuts += 1;
            }
            Err(Error::Net(e)) => assert!(matches!(*e, NetError::Refused { .. })),
            Err(e) => panic!("{e}"),
        }
    }
    assert!(stopped_with_cuts > 3, "{stopped_with_cuts}");
}

/// A cut link joins an occurrence of a cut formula's tree to its dual,
/// below which lies no other cut link, and nothing else; the vertices
/// above no cut link are none of the structure, and a cut against an
/// axiom between its own two ends is a switching cycle.
#[test]
fn cut_links_are_checked() {
    let mut sequent: Sequent = "|- a, ~a".parse().unwrap();
    sequent.add_cut("|- a par ~a".parse().unwrap()).unwrap();
    // 0 a, 1 ~a, 2 a ⅋ ~a, 3 a, 4 ~a, 5 ~a ⊗ a, 6 ~a, 7 a.
    let forest = Forest::new(&sequent).unwrap();
    let v = VertexId::new;
    let built = |cuts: &[(u32, u32)], links: &[(u32, u32)]| {
        let pairs =
            |list: &[(u32, u32)]| list.iter().map(|&(x, y)| (v(x), v(y))).collect::<Vec<_>>();
        ProofStructure::with_cuts(forest.clone(), Criterion::MLL, &pairs(cuts), &pairs(links))
    };
    let net = built(&[(5, 2)], &[(3, 4), (0, 6), (1, 7)]).unwrap();
    assert_eq!(net.cuts(), [(v(2), v(5))]);
    assert_eq!(net.is_correct(never), Ok(()));
    let cut = |vertex: u32| {
        format!(
            "{:?}",
            Err::<(), Error>(
                NetError::Placement {
                    vertex: v(vertex),
                    fault: Placement::Cut,
                }
                .into()
            )
        )
    };
    let error = |built: Result<ProofStructure, Error>| format!("{:?}", built.map(|_| ()));
    assert_eq!(error(built(&[(2, 6)], &[])), cut(2));
    assert_eq!(error(built(&[(0, 1)], &[])), cut(0));
    assert_eq!(error(built(&[(2, 5), (3, 6)], &[])), cut(3));
    assert!(matches!(
        built(&[], &[(1, 3)]),
        Err(Error::Net(e)) if matches!(*e, NetError::Placement { fault: Placement::Uncut, .. })
    ));
    let net = built(&[(3, 6), (4, 7)], &[(0, 1), (3, 6), (4, 7)]).unwrap();
    assert!(matches!(
        net.is_correct(never),
        Err(NetError::SwitchingCycle { .. })
    ));
    assert!(!net.is_acyclic(&mut net.scratch()));
}
