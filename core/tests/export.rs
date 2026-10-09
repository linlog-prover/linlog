// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The LaTeX, Typst, SVG and Rocq exports through the public API.
//! Derivations are pinned as standalone documents in `tests/snapshots/`,
//! which the flake's `export` check compiles and renders and its `rocq`
//! check runs through Rocq; run with `BLESS=1` to rewrite them after an
//! intended change, and review the diff.

#![cfg(all(
    feature = "parse",
    feature = "interactive",
    feature = "latex",
    feature = "rocq",
    feature = "svg",
    feature = "typst"
))]

use linlog::export::rocq::{self, Unsupported};
use linlog::export::svg::{self, Style};
use linlog::export::{Form, latex, typst};
use linlog::proofs::{Compact, Sides};
use linlog::proofs::{Labels, OpenGoal};
use linlog::{
    Criterion, Derivation, Forest, GoalId, Inference, Interactive, Limits, Mode, Options, Proof,
    ProofStructure, Step, VertexId, ViewOptions,
};
use linlog::{Error, Refusal};
use linlog::{Named, Rule, Sequent, Verdict, prove};
use std::collections::BTreeSet;
use std::path::Path;

/// Compares `actual` with the snapshot `name`, or writes it there when the
/// environment sets `BLESS`.
fn snapshot(name: &str, actual: &str) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/snapshots")
        .join(name);
    if std::env::var_os("BLESS").is_some() {
        std::fs::write(&path, format!("{actual}\n")).unwrap();
        return;
    }
    let expected = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{}: {e}; run with BLESS=1", path.display()));
    assert_eq!(actual, expected.trim_end_matches('\n'), "{name}");
}

/// Pins a derivation in every target as the snapshots `name.tex`,
/// `name.typ` and `name.svg`.
fn pin(name: &str, derivation: &Derivation) {
    let mut latex = latex::Options::default();
    latex.form = Form::Standalone;
    snapshot(
        &format!("{name}.tex"),
        &written(|out| latex::write(derivation, &latex, out, |_| false)),
    );
    let mut typst = typst::Options::default();
    typst.form = Form::Standalone;
    snapshot(
        &format!("{name}.typ"),
        &written(|out| typst::write(derivation, &typst, out, |_| false)),
    );
    snapshot(
        &format!("{name}.svg"),
        &written(|out| svg::write(derivation, &Style::default(), out, |_| false)),
    );
}

/// Returns a proof of `input` in `mode`.
fn proof(input: &str, mode: Mode) -> Proof {
    let sequent: Sequent = input.parse().unwrap();
    let outcome = prove(&sequent, mode, &Options::default()).unwrap();
    let Verdict::Proved(proof) = outcome.verdict else {
        panic!("{input} is provable");
    };
    *proof
}

/// Pins a derivation's certificate as the snapshot `name.v`.
fn pin_certificate(name: &str, derivation: &Derivation) {
    let options = rocq::Options::default().with_form(Form::Standalone);
    let script = certificate(|out| rocq::write(derivation, &options, out, |_| false));
    snapshot(&format!("{name}.v"), &script.unwrap());
}

/// Proves `input` in `mode` and pins its derivation, two-sided in
/// intuitionistic mode, and its certificate.
fn pin_proof(name: &str, input: &str, mode: Mode) {
    let proof = proof(input, mode);
    let derivation = if mode.is_intuitionistic() {
        proof.derivation_within(
            &ViewOptions::default().with_sides(Sides::Two),
            &Limits::default(),
            |_| false,
        )
    } else {
        proof.derivation()
    };
    let derivation = derivation.unwrap();
    pin(name, &derivation);
    pin_certificate(name, &derivation);
}

/// The default limits but no bound on a drawing's size.
fn unbounded() -> Limits {
    Limits::default().with_derivation_bytes(None)
}

/// The default limits with a drawing's size bounded by `bytes`.
fn bounded(bytes: u64) -> Limits {
    Limits::default().with_derivation_bytes(Some(bytes))
}

/// What a target's `write` makes of a derivation, as a string.
fn written(write: impl FnOnce(&mut String) -> Result<(), linlog::Error>) -> String {
    let mut out = String::new();
    write(&mut out).expect("a string takes any text and nothing stops");
    out
}

/// What Rocq's `write` makes of a derivation, or why it has no
/// certificate.
fn certificate(
    write: impl FnOnce(&mut String) -> Result<(), linlog::Error>,
) -> Result<String, Unsupported> {
    let mut out = String::new();
    match write(&mut out) {
        Ok(()) => Ok(out),
        Err(linlog::Error::Unsupported(unsupported)) => Err(unsupported),
        Err(error) => panic!("{error}"),
    }
}

/// Derivations of each fragment, one-sided and two-sided, as ebproof,
/// curryst and SVG trees, and as Rocq scripts.
#[test]
fn derivations() {
    pin_proof("mll", "A * B |- B * A", Mode::CLASSICAL);
    pin_proof("mall", "A + B, 1 |- B + A", Mode::CLASSICAL);
    pin_proof("mell", "!A, !B |- !(A * A)", Mode::CLASSICAL);
    pin_proof("ill", "1, A & B, B -o C |- C", Mode::INTUITIONISTIC);
}

/// A tree higher than curryst sets is laid out by linlog's own Typst
/// code, which the `export` check compiles at 41 inferences high; the
/// own layout of a proof in progress draws its open goal under dots.
#[test]
fn typst_layout() {
    let bots = vec!["bot"; 30].join(", ");
    let input = format!("|- {bots}, 1 * (1 * (1 * (1 * (1 * (1 * (1 * (1 * (1 * (1 * 1)))))))))");
    let high = proof(&input, Mode::CLASSICAL);
    let high = high.derivation().unwrap();
    let mut options = typst::Options::default();
    options.form = Form::Standalone;
    let text = written(|out| typst::write(&high, &options, out, |_| false));
    assert!(text.contains("#context"), "past curryst's height");
    snapshot("high.typ", &text);
    let sequent: Sequent = "A, A -o B |- B".parse().unwrap();
    let mut state = Interactive::new(&sequent, Mode::INTUITIONISTIC).unwrap();
    let goals = state
        .apply(
            GoalId::new(0),
            &Step::new(1, "⊸L".parse::<Named>().unwrap()).left(&[0]),
        )
        .unwrap();
    state.apply(goals[0], &Step::new(0, Rule::Ax)).unwrap();
    options.layout = typst::Layout::Linlog;
    snapshot(
        "open-linlog.typ",
        &written(|out| typst::write(&state.derivation().unwrap(), &options, out, |_| false)),
    );
}

/// A derivation of LJ read back from the linear proof of its image is
/// drawn two-sided in every drawn target, with the ordinary connectives
/// and the rules of LJ.
#[test]
fn ordinary_derivation() {
    use linlog::ordinary::{self, Logic, Translation};
    let sequent: ordinary::Sequent = "a -> b, b -> c |- a -> c".parse().unwrap();
    let image = ordinary::translate(
        &sequent,
        Logic::Intuitionistic,
        Translation::CallByName,
        &Limits::default(),
    )
    .unwrap();
    let outcome = prove(image.sequent(), image.mode(), &Options::default()).unwrap();
    let Verdict::Proved(proof) = outcome.verdict else {
        panic!("provable");
    };
    let derivation = image
        .read_back(&proof, &Limits::default(), |_| false)
        .unwrap();
    derivation.check(&Limits::default(), |_| false).unwrap();
    let (mut tex, mut typ, mut drawing) = (String::new(), String::new(), String::new());
    let mut latex = latex::Options::default();
    latex.form = Form::Standalone;
    latex::write(&derivation, &latex, &mut tex, |_| false).unwrap();
    snapshot("ordinary.tex", &tex);
    let mut typst = typst::Options::default();
    typst.form = Form::Standalone;
    typst::write(&derivation, &typst, &mut typ, |_| false).unwrap();
    snapshot("ordinary.typ", &typ);
    svg::write(&derivation, &Style::default(), &mut drawing, |_| false).unwrap();
    snapshot("ordinary.svg", &drawing);
}

/// The certificate of an ordinary sequent without atoms or hypotheses
/// binds nothing, in both logics: `fun =>` is no term; and its prelude is
/// the one given, or the logic's own.
#[test]
fn ordinary_certificate_without_binders() {
    use linlog::ordinary::{self, Logic};
    let sequent: ordinary::Sequent = "|- true".parse().unwrap();
    for logic in [Logic::Classical, Logic::Intuitionistic] {
        let options = ordinary::Options::default().with_logic(logic);
        let unbounded = Limits::default();
        let outcome = ordinary::decide(&sequent, &options, &Options::default(), &unbounded, |_| {
            false
        })
        .unwrap();
        let ordinary::Verdict::Valid(derivation) = outcome.verdict else {
            panic!("{logic}: {:?}", outcome.verdict);
        };
        let mut out = String::new();
        rocq::write(&*derivation, &rocq::Options::default(), &mut out, |_| false).unwrap();
        assert!(out.contains("exact (") && !out.contains("fun =>"), "{out}");

        // Standalone, a certificate over `Prop` starts with the prelude
        // given, or by default with the excluded middle's import if it is
        // classical and with the lemma if it is of LJ.
        let standalone = rocq::Options::default().with_form(Form::Standalone);
        let start = |options: &rocq::Options| {
            let mut out = String::new();
            rocq::write(&*derivation, options, &mut out, |_| false).unwrap();
            out.lines().next().unwrap().to_owned()
        };
        let own = match logic {
            Logic::Classical => rocq::CLASSICAL_PRELUDE,
            _ => "Lemma certificate : True.",
        };
        assert_eq!(start(&standalone), own);
        let given = standalone.with_prelude(Some("From Coq Require Import Classical_Prop.".into()));
        assert_eq!(start(&given), "From Coq Require Import Classical_Prop.");
    }
}

/// A compact derivation draws a run of weakenings as one inference with a
/// starred label in every drawn target.
#[test]
fn compact_view() {
    let view = ViewOptions::default()
        .with_compact(Compact::Always)
        .with_sides(Sides::Two);
    let proof = proof("!A, !B, !C |- 1 * 1", Mode::INTUITIONISTIC);
    let derivation = proof
        .derivation_within(&view, &Limits::default(), |_| false)
        .unwrap();
    assert!(derivation.inferences().iter().any(|i| i.times() == 3));
    pin("compact", &derivation);
}

/// An open goal of a proof in progress is its sequent under vertical dots,
/// with no inference line, in every drawn target, and has no certificate.
#[test]
fn open_goal() {
    let sequent: Sequent = "A, A -o B |- B".parse().unwrap();
    let mut state = Interactive::new(&sequent, Mode::INTUITIONISTIC).unwrap();
    let goals = state
        .apply(
            GoalId::new(0),
            &Step::new(1, "⊸L".parse::<Named>().unwrap()).left(&[0]),
        )
        .unwrap();
    state.apply(goals[0], &Step::new(0, Rule::Ax)).unwrap();
    pin("open", &state.derivation().unwrap());
    pin_fragments(
        "open-dashed",
        &state.derivation().unwrap(),
        Labels::Upright,
        OpenGoal::Dashed,
    );
    let options = rocq::Options::default();
    assert_eq!(
        certificate(|out| rocq::write(&state.derivation().unwrap(), &options, out, |_| false)),
        Err(Unsupported::Open)
    );
}

/// Pins a derivation as LaTeX and Typst fragments, `name.frag.tex` and
/// `name.frag.typ`, under the labels and the open goal given: the
/// `export` check compiles them inside a document of its own.
fn pin_fragments(name: &str, derivation: &Derivation, labels: Labels, open: OpenGoal) {
    let mut latex = latex::Options::default();
    latex.labels = labels.clone();
    latex.open = open.clone();
    snapshot(
        &format!("{name}.frag.tex"),
        &written(|out| latex::write(derivation, &latex, out, |_| false)),
    );
    let mut typst = typst::Options::default();
    typst.labels = labels;
    typst.open = open;
    snapshot(
        &format!("{name}.frag.typ"),
        &written(|out| typst::write(derivation, &typst, out, |_| false)),
    );
}

/// Every rule label of every target is in a snapshot: the classical rules
/// on one sequent, Mix and affine weakening on their own, the
/// intuitionistic rules on one sequent; the intuitionistic ones also as
/// fragments with subscript labels.
#[test]
fn every_label() {
    let classical = "|- ((((~A | ~B) | (A * B)) & 1) & ((bot | 1) & top)) & \
                     (((1 + 0) & (0 + 1)) & ((!1 & ?1) & ((?A | 1) & (?~A | (A * A)))))";
    let intuitionistic = "|- (((A -o ((A -o B) -o B)) & ((A * B) -o (B * A))) & \
                          (((A & B) -o A) & ((A & B) -o B))) & \
                          ((((A + B) -o (B + A)) & (1 -o 1)) & \
                          (((0 -o A) & top) & ((!A -o (A * A)) & ((!A -o !A) & (!A -o 1)))))";
    let mut rules = BTreeSet::new();
    for (name, input, mode) in [
        ("labels", classical, Mode::CLASSICAL),
        ("mix", "A, B |- A, B", Mode::CLASSICAL.with_mix()),
        ("affine", "A, B |- A", Mode::CLASSICAL.with_affine()),
        ("labels_ill", intuitionistic, Mode::INTUITIONISTIC),
    ] {
        let proof = proof(input, mode);
        let derivation = if mode.is_intuitionistic() {
            proof.derivation_within(
                &ViewOptions::default().with_sides(Sides::Two),
                &Limits::default(),
                |_| false,
            )
        } else {
            proof.derivation()
        };
        let derivation = derivation.unwrap();
        rules.extend(derivation.inferences().iter().map(Inference::rule));
        pin(name, &derivation);
        if name.starts_with("labels") {
            pin_certificate(name, &derivation);
        }
        if name == "labels_ill" {
            pin_fragments(name, &derivation, Labels::Subscript, OpenGoal::Dots);
        }
    }
    let missing: Vec<Named> = Named::ALL
        .into_iter()
        .filter(|r| r.rule != Rule::Open && !rules.contains(r))
        .collect();
    assert!(missing.is_empty(), "no snapshot has {missing:?}");
}

/// With ids per formula, the drawing of a proof in progress names every
/// formula of an open goal by its inference and position, and the
/// session's map turns the inference into the goal `apply` takes.
#[test]
fn ids_name_goals() {
    let sequent: Sequent = "A, A -o B |- B".parse().unwrap();
    let mut state = Interactive::new(&sequent, Mode::INTUITIONISTIC).unwrap();
    let goals = state
        .apply(
            GoalId::new(0),
            &Step::new(1, "⊸L".parse::<Named>().unwrap()).left(&[0]),
        )
        .unwrap();
    let mut style = Style::default();
    style.ids = true;
    let drawing = written(|out| svg::write(&state.derivation().unwrap(), &style, out, |_| false));
    let ids = state.derivation_ids();
    let drawn = ids.iter().position(|&id| id == goals[1]).unwrap();
    // The open goal `B ⊢ B` has the hypothesis at position 0.
    assert!(
        drawing.contains(&format!(r#"<g id="i{drawn}-0">"#)),
        "{drawing}"
    );
    assert!(
        drawing.contains(&format!(r#"<g id="i{drawn}-1">"#)),
        "{drawing}"
    );
    state.apply(ids[drawn], &Step::new(0, Rule::Ax)).unwrap();
}

/// A proof of full linear logic with a contraction is pinned as a
/// certificate; the lemma's name and the prelude are options; a proof
/// with Mix or with the weakening of affine mode has no certificate.
#[test]
fn certificates() {
    let ll = proof("!(A & B) |- !A * !B", Mode::CLASSICAL);
    let derivation = ll.derivation().unwrap();
    pin_certificate("ll", &derivation);
    let options = rocq::Options::default()
        .with_form(Form::Standalone)
        .with_lemma("bang_with".parse().unwrap())
        .with_prelude(Some("Require Import kernel.".to_owned()));
    // A lemma's name is an identifier and no keyword of Rocq.
    for name in ["two words", "1st", "fun", "_", ""] {
        let refused = name.parse::<rocq::Identifier>();
        assert!(
            matches!(
                &refused,
                Err(linlog::Error::InvalidOption {
                    key: "rocq.lemma",
                    ..
                })
            ),
            "{name:?}: {refused:?}"
        );
    }
    let script = certificate(|out| rocq::write(&derivation, &options, out, |_| false)).unwrap();
    assert!(script.starts_with("Require Import kernel.\n\nLemma bang_with (A B : formula) : ll ["));
    assert!(script.contains("apply (co_r_ext []); cbn_sequent.\n"));
    assert!(script.ends_with("\nQed."));

    let options = rocq::Options::default();
    let mix = proof("A, B |- A, B", Mode::CLASSICAL.with_mix());
    assert_eq!(
        certificate(|out| rocq::write(&mix.derivation().unwrap(), &options, out, |_| false)),
        Err(Unsupported::Mix)
    );
    let affine = proof("A, B |- A", Mode::CLASSICAL.with_affine());
    assert_eq!(
        certificate(|out| rocq::write(&affine.derivation().unwrap(), &options, out, |_| false)),
        Err(Unsupported::AffineWeakening)
    );
}

/// Sequents print as math, one-sided or two-sided, with longer names in
/// italic and escaped; Greek names and those of the LLTP library are
/// pinned as a document that pdfLaTeX compiles.
#[test]
fn sequents() {
    let sequent: Sequent = "α, Γ -o P‿a·b |- P‿a·b * Γα".parse().unwrap();
    let mut options = latex::Options::default();
    options.form = Form::Standalone;
    let (classical, unbounded) = (Mode::CLASSICAL, Limits::default());
    let latex = |sequent, options| latex::sequent(sequent, classical, options, &unbounded).unwrap();
    snapshot("names.tex", &latex(&sequent, &options));

    let sequent: Sequent = "x_1 * foo, !A |- ?B & 1, B".parse().unwrap();
    assert_eq!(
        latex(&sequent, &latex::Options::default()),
        r"$\vdash \mathit{x\_1}^\bot \parr \mathit{foo}^\bot, \wn A^\bot, \wn B \with \mathbf{1}, B$"
    );
    assert_eq!(
        typst::sequent(&sequent, classical, &typst::Options::default(), &unbounded).unwrap(),
        r#"$⊢ italic("x_1")^⊥ ⅋ italic("foo")^⊥, class("normal", ?)A^⊥, class("normal", ?)B class("binary", \&) bold(1), B$"#
    );

    let sequent: Sequent = "!A, A -o (B + top) |- B & 0".parse().unwrap();
    let i = Mode::INTUITIONISTIC;
    assert_eq!(
        latex::sequent(&sequent, i, &latex::Options::default(), &unbounded).unwrap(),
        r"$\oc A, A \multimap (B \oplus \top) \vdash B \with 0$"
    );
    assert_eq!(
        typst::sequent(&sequent, i, &typst::Options::default(), &unbounded).unwrap(),
        r#"$!A, A ⊸ (B ⊕ ⊤) ⊢ B class("binary", \&) 0$"#
    );
    assert!(
        svg::sequent(&sequent, i, &Style::default(), &unbounded)
            .unwrap()
            .contains("<title>!A, A ⊸ (B ⊕ ⊤) ⊢ B &amp; 0</title>")
    );
    // A sequent's text past the bound is refused before it is laid out,
    // and one without an intuitionistic reading in intuitionistic mode.
    let tight = Limits::default().with_derivation_bytes(Some(100));
    assert!(matches!(
        svg::sequent(&sequent, classical, &Style::default(), &tight),
        Err(linlog::Error::Refused(linlog::Refusal::Output {
            what: "sequent",
            ..
        }))
    ));
    let classical_only: Sequent = "|- a, b".parse().unwrap();
    assert!(matches!(
        latex::sequent(&classical_only, i, &latex::Options::default(), &unbounded),
        Err(linlog::Error::NotIntuitionistic(_))
    ));
}

/// Proof nets are pinned as SVG: a net whose links cross, a structure
/// whose switching cycle is highlighted, and one whose second part is.
#[test]
fn nets() {
    let net = ProofStructure::from_proof(
        &proof("A * B |- B * A", Mode::CLASSICAL),
        Criterion::MLL,
        &Limits::default(),
        |_| false,
    );
    snapshot(
        "net.svg",
        &svg::net(&net.unwrap(), &Style::default(), &unbounded(), |_| false).unwrap(),
    );

    let forest = Forest::new(&"|- A * ~A".parse().unwrap()).unwrap();
    let links = [(VertexId::new(1), VertexId::new(2))];
    let cyclic = ProofStructure::from_links(forest, Criterion::MLL, &links).unwrap();
    assert!(cyclic.is_correct(|_| false).is_err());
    snapshot(
        "cycle.svg",
        &svg::net(&cyclic, &Style::default(), &unbounded(), |_| false).unwrap(),
    );

    // ⊢ A ⅋ B, ~A, ~B: the part of `B` and `~B` hangs off the right premise.
    let forest = Forest::new(&"|- A par B, ~A, ~B".parse().unwrap()).unwrap();
    let links = [
        (VertexId::new(1), VertexId::new(3)),
        (VertexId::new(2), VertexId::new(4)),
    ];
    let parted = ProofStructure::from_links(forest, Criterion::MLL, &links).unwrap();
    let drawing = svg::net(&parted, &Style::default(), &unbounded(), |_| false).unwrap();
    let highlight = format!(r#"stroke="{}""#, Style::default().highlight);
    assert!(drawing.contains(&highlight), "{drawing}");
    snapshot("disconnected.svg", &drawing);
}

/// A net's drawing estimated past its limit is not drawn; the estimate is
/// at least the drawing's bytes and within a few times of them.
#[test]
fn net_limit() {
    let style = Style::default();
    let net = ProofStructure::from_proof(
        &proof("A * B |- B * A", Mode::CLASSICAL),
        Criterion::MLL,
        &Limits::default(),
        |_| false,
    );
    let net = net.unwrap();
    let drawing = svg::net(&net, &style, &unbounded(), |_| false).unwrap();
    let bytes = drawing.len() as u64;
    assert!(matches!(
        svg::net(&net, &style, &bounded(bytes), |_| false),
        Err(Error::Refused(Refusal::Output { estimate_bytes, limit_bytes, .. }))
            if estimate_bytes > bytes && limit_bytes == bytes
    ));
    assert_eq!(
        svg::net(&net, &style, &bounded(4 * bytes), |_| false),
        Ok(drawing)
    );
}

/// The SVG layout asks its stop condition in its first pass over the
/// inferences, before anything is written.
#[test]
fn svg_stops_in_its_layout() {
    let proof = proof("A * B |- B * A", Mode::CLASSICAL);
    let derivation = proof.derivation().unwrap();
    let mut out = String::new();
    let written = svg::write(&derivation, &Style::default(), &mut out, |_| true);
    assert!(matches!(
        written,
        Err(Error::Refused(Refusal::Stopped { .. }))
    ));
    assert_eq!(out, "");
}

/// Parses an SVG document, checks that every text, circle and path lies
/// inside its view box, and returns the number of `<text>`, `<path>` and
/// `<circle>` elements.
fn structure(document: &str) -> [usize; 3] {
    let document = roxmltree::Document::parse(document).expect("well-formed XML");
    let root = document.root_element();
    let view: Vec<i64> = root
        .attribute("viewBox")
        .unwrap()
        .split(' ')
        .map(|v| v.parse().unwrap())
        .collect();
    let inside = |x: i64, y: i64| (0..=view[2]).contains(&x) && (0..=view[3]).contains(&y);
    let mut counts = [0; 3];
    for e in document.descendants().filter(|e| e.is_element()) {
        let number = |name: &str| e.attribute(name).unwrap().parse::<i64>().unwrap();
        match e.tag_name().name() {
            "text" => {
                counts[0] += 1;
                let size = e
                    .attribute("font-size")
                    .map_or(1000, |_| number("font-size"));
                let (x, y) = (number("x"), number("y"));
                assert!(inside(x, y - size * 890 / 1000), "{e:?}");
                assert!(
                    inside(x + number("textLength"), y + size * 210 / 1000),
                    "{e:?}"
                );
            }
            "circle" => {
                counts[2] += 1;
                let (x, y, r) = (number("cx"), number("cy"), number("r"));
                assert!(inside(x - r, y - r) && inside(x + r, y + r), "{e:?}");
            }
            "path" => {
                counts[1] += 1;
                let d = e.attribute("d").unwrap();
                let (mut x, mut y) = (0, 0);
                // The commands, each a letter followed by its numbers.
                let mut rest = d;
                while let Some(letter) = rest.chars().next() {
                    let end = rest[1..]
                        .find(|c: char| c.is_ascii_uppercase())
                        .map_or(rest.len(), |i| i + 1);
                    let n: Vec<i64> = rest[1..end]
                        .split(' ')
                        .map(|v| v.parse().unwrap())
                        .collect();
                    match letter {
                        'M' | 'L' => (x, y) = (n[0], n[1]),
                        'H' => x = n[0],
                        'V' => y = n[0],
                        'A' => {
                            assert!(inside(x + n[0], y - n[1]), "{e:?}");
                            (x, y) = (n[5], n[6]);
                        }
                        _ => panic!("{d}"),
                    }
                    assert!(inside(x, y), "{e:?}");
                    rest = &rest[end..];
                }
            }
            _ => {}
        }
    }
    counts
}

/// The documents are well-formed and draw inside their view box, with one
/// text per conclusion and rule name and one line per inference, and in a
/// net one circle per connective, one path per premise edge, conclusion
/// and link, and one text per literal and connective; the raised `⊥` of
/// a negated literal and the subscript of `&L₂` are texts of their own.
#[test]
fn svg_structure() {
    let style = Style::default();
    let two_sided = proof("1, A & B, B -o C |- C", Mode::INTUITIONISTIC);
    let drawn = written(|out| {
        svg::write(
            &two_sided
                .derivation_within(
                    &ViewOptions::default().with_sides(Sides::Two),
                    &Limits::default(),
                    |_| false,
                )
                .unwrap(),
            &style,
            out,
            |_| false,
        )
    });
    assert_eq!(structure(&drawn), [11, 5, 0]);

    let sequent: Sequent = "A, A -o B |- B".parse().unwrap();
    let mut state = Interactive::new(&sequent, Mode::INTUITIONISTIC).unwrap();
    let goals = state
        .apply(
            GoalId::new(0),
            &Step::new(1, "⊸L".parse::<Named>().unwrap()).left(&[0]),
        )
        .unwrap();
    state.apply(goals[0], &Step::new(0, Rule::Ax)).unwrap();
    assert_eq!(
        structure(&written(|out| svg::write(
            &state.derivation().unwrap(),
            &style,
            out,
            |_| false
        ))),
        [6, 2, 0]
    );

    let net = ProofStructure::from_proof(
        &proof("A * B |- B * A", Mode::CLASSICAL),
        Criterion::MLL,
        &Limits::default(),
        |_| false,
    );
    assert_eq!(
        structure(&svg::net(&net.unwrap(), &style, &unbounded(), |_| false).unwrap()),
        [8, 8, 2]
    );

    let sequent: Sequent = "x_1 * foo |- A".parse().unwrap();
    let drawn = svg::sequent(&sequent, Mode::CLASSICAL, &style, &Limits::default()).unwrap();
    assert_eq!(structure(&drawn), [5, 0, 0]);
}

/// PNG and PDF render a drawing with the font given: the PNG carries the
/// drawing's title, every PDF profile passes krilla's own validation, the
/// accessible one is tagged, and a PDF without a date is refused.
#[cfg(all(feature = "png", feature = "pdf"))]
#[test]
fn renders() {
    use linlog::export::{pdf, png};
    let font =
        std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("../cli/fonts/Euler-Math.otf"))
            .unwrap();
    let derivation = proof("A, A -o B |- B", Mode::INTUITIONISTIC);
    let drawing = written(|out| {
        svg::write(
            &derivation
                .derivation_within(
                    &ViewOptions::default().with_sides(Sides::Two),
                    &Limits::default(),
                    |_| false,
                )
                .unwrap(),
            &Style::default(),
            out,
            |_| false,
        )
    });
    let image = png::from_svg(
        &drawing,
        &[&font],
        &png::Options::default(),
        &Limits::default(),
    )
    .unwrap();
    assert!(image.starts_with(b"\x89PNG"));
    assert!(image.windows(5).any(|w| w == b"Title"));
    let date = Some(pdf::Date::from_unix(1_791_158_399));
    for (compatible, accessible, version) in [
        (false, false, "%PDF-2.0"),
        (true, false, "%PDF-1.7"),
        (false, true, "%PDF-1.7"),
    ] {
        let mut options = pdf::Options::default();
        options.compatible = compatible;
        options.accessible = accessible;
        options.date = date;
        let document = pdf::from_svg(&drawing, &[&font], &options, &Limits::default()).unwrap();
        assert!(
            document.starts_with(version.as_bytes()),
            "{compatible} {accessible}"
        );
        let root = b"/StructTreeRoot";
        let tagged = document.windows(root.len()).any(|w| w == root);
        assert_eq!(tagged, accessible);
    }
    assert_eq!(
        pdf::from_svg(
            &drawing,
            &[&font],
            &pdf::Options::default(),
            &Limits::default()
        ),
        Err(Error::NoDate)
    );
}

/// The bounds of a render are compared before the SVG is parsed: a text
/// of many glyphs that is no document passes neither the memory bound
/// nor, with the size its root declares, the pixel bound, and each
/// refusal says so rather than that the document does not read.
#[cfg(all(feature = "png", feature = "pdf"))]
#[test]
fn render_bounds_come_first() {
    use linlog::export::{pdf, png};
    let glyphs = format!("<svg><text>{}</text>", "x".repeat(1 << 16));
    let limits = Limits::default().with_memory_bytes(Some(64 << 20));
    assert!(matches!(
        png::from_svg(&glyphs, &[], &png::Options::default(), &limits),
        Err(Error::Refused(Refusal::Memory {
            limit_bytes: 67_108_864,
            needed_bytes: Some(estimate),
            ..
        })) if estimate > 1000 << 16
    ));
    let mut pdf = pdf::Options::default();
    pdf.date = Some(pdf::Date::from_unix(0));
    assert!(matches!(
        pdf::from_svg(&glyphs, &[], &pdf, &limits),
        Err(Error::Refused(Refusal::Memory { .. }))
    ));
    let wide = r#"<svg width="100000" height="1000.5px"><text>"#;
    assert!(matches!(
        png::from_svg(wide, &[], &png::Options::default(), &Limits::default()),
        Err(Error::Refused(Refusal::Pixels {
            pixels: 400_400_000,
            limit: png::Options::DEFAULT_PIXELS,
            ..
        }))
    ));
    // What no drawing of linlog has is refused before the estimate, which
    // could not see its cost: six levels of seven `<use>` over a `<rect>`
    // took 227 MB under a bound of 64 MiB.
    for svg in [
        r##"<svg xmlns="http://www.w3.org/2000/svg"><rect id="r" width="9" height="9"/><use href="#r"/></svg>"##,
        r#"<svg><g><image width="9" height="9"/></g></svg>"#,
        r#"<svg><path xlink:href="x" d="M0 0"/></svg>"#,
        r#"<!DOCTYPE svg [<!ENTITY a "aa">]><svg><text>&a;</text></svg>"#,
        // An empty `<title/>` hid what followed it from the measure.
        r#"<svg><title/><use href="x"/></svg>"#,
        r#"<svg><desc/><image width="9" height="9"/></svg>"#,
    ] {
        assert!(
            matches!(
                png::from_svg(svg, &[], &png::Options::default(), &limits),
                Err(Error::NotSvg { .. })
            ),
            "{svg}"
        );
        assert!(
            matches!(
                pdf::from_svg(svg, &[], &pdf, &limits),
                Err(Error::NotSvg { .. })
            ),
            "{svg}"
        );
    }
}
