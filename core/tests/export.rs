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
use linlog::{
    Compact, Derivation, Forest, InfId, Interactive, Mode, OccId, Options, Proof, ProofStructure,
    ViewOptions,
};
use linlog::{Labels, OpenGoal, Reading, Rule, Sequent, Verdict, prove};
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
    let latex = latex::Options {
        form: Form::Standalone,
        ..latex::Options::default()
    };
    snapshot(
        &format!("{name}.tex"),
        &latex::derivation(derivation, &latex),
    );
    let typst = typst::Options {
        form: Form::Standalone,
        ..typst::Options::default()
    };
    snapshot(
        &format!("{name}.typ"),
        &typst::derivation(derivation, &typst),
    );
    snapshot(
        &format!("{name}.svg"),
        &svg::derivation(derivation, &Style::default()),
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
    let options = rocq::Options {
        form: Form::Standalone,
        ..rocq::Options::default()
    };
    let script = rocq::derivation(derivation, &options);
    snapshot(&format!("{name}.v"), &script.unwrap());
}

/// Proves `input` in `mode` and pins its derivation, two-sided in
/// intuitionistic mode, and its certificate.
fn pin_proof(name: &str, input: &str, mode: Mode) {
    let proof = proof(input, mode);
    let derivation = if mode.intuitionistic {
        proof.two_sided_derivation()
    } else {
        proof.derivation()
    };
    let derivation = derivation.unwrap();
    pin(name, &derivation);
    pin_certificate(name, &derivation);
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
    let options = typst::Options {
        form: Form::Standalone,
        ..typst::Options::default()
    };
    let written = typst::derivation(&high, &options);
    assert!(written.contains("#context"), "past curryst's height");
    snapshot("high.typ", &written);
    let sequent: Sequent = "A, A -o B |- B".parse().unwrap();
    let mut state = Interactive::new(&sequent, Mode::INTUITIONISTIC).unwrap();
    let goals = state.apply(InfId::new(0), 1, Rule::ImpLeft, &[0]).unwrap();
    state.apply(goals[0], 0, Rule::Ax, &[]).unwrap();
    let options = typst::Options {
        layout: typst::Layout::Linlog,
        ..options
    };
    snapshot(
        "open-linlog.typ",
        &typst::derivation(&state.derivation(), &options),
    );
}

/// A compact derivation draws a run of weakenings as one inference with a
/// starred label in every drawn target.
#[test]
fn compact_view() {
    let view = ViewOptions::default().compact(Compact::Always);
    let proof = proof("!A, !B, !C |- 1 * 1", Mode::INTUITIONISTIC);
    let derivation = proof.two_sided_derivation_with(&view, || false).unwrap();
    assert!(derivation.inferences().iter().any(|i| i.times == 3));
    pin("compact", &derivation);
}

/// An open goal of a proof in progress is its sequent under vertical dots,
/// with no inference line, in every drawn target, and has no certificate.
#[test]
fn open_goal() {
    let sequent: Sequent = "A, A -o B |- B".parse().unwrap();
    let mut state = Interactive::new(&sequent, Mode::INTUITIONISTIC).unwrap();
    let goals = state.apply(InfId::new(0), 1, Rule::ImpLeft, &[0]).unwrap();
    state.apply(goals[0], 0, Rule::Ax, &[]).unwrap();
    pin("open", &state.derivation());
    pin_fragments(
        "open-dashed",
        &state.derivation(),
        Labels::Upright,
        OpenGoal::Dashed,
    );
    let options = rocq::Options::default();
    assert_eq!(
        rocq::derivation(&state.derivation(), &options),
        Err(Unsupported::Open)
    );
}

/// Pins a derivation as LaTeX and Typst fragments, `name.frag.tex` and
/// `name.frag.typ`, under the labels and the open goal given: the
/// `export` check compiles them inside a document of its own.
fn pin_fragments(name: &str, derivation: &Derivation, labels: Labels, open: OpenGoal) {
    let latex = latex::Options {
        labels: labels.clone(),
        open: open.clone(),
        ..latex::Options::default()
    };
    snapshot(
        &format!("{name}.frag.tex"),
        &latex::derivation(derivation, &latex),
    );
    let typst = typst::Options {
        labels,
        open,
        ..typst::Options::default()
    };
    snapshot(
        &format!("{name}.frag.typ"),
        &typst::derivation(derivation, &typst),
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
        ("affine", "A, B |- A", Mode::CLASSICAL.affine()),
        ("labels_ill", intuitionistic, Mode::INTUITIONISTIC),
    ] {
        let proof = proof(input, mode);
        let derivation = if mode.intuitionistic {
            proof.two_sided_derivation()
        } else {
            proof.derivation()
        };
        let derivation = derivation.unwrap();
        rules.extend(derivation.inferences().iter().map(|i| i.rule));
        pin(name, &derivation);
        if name.starts_with("labels") {
            pin_certificate(name, &derivation);
        }
        if name == "labels_ill" {
            pin_fragments(name, &derivation, Labels::Subscript, OpenGoal::Dots);
        }
    }
    let missing: Vec<Rule> = Rule::ALL
        .into_iter()
        .filter(|r| *r != Rule::Open && !rules.contains(r))
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
    let goals = state.apply(InfId::new(0), 1, Rule::ImpLeft, &[0]).unwrap();
    let style = Style {
        ids: true,
        ..Style::default()
    };
    let drawing = svg::derivation(&state.derivation(), &style);
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
    state.apply(ids[drawn], 0, Rule::Ax, &[]).unwrap();
}

/// A proof of full linear logic with a contraction is pinned as a
/// certificate; the lemma's name and the prelude are options; a proof
/// with Mix or with the weakening of affine mode has no certificate.
#[test]
fn certificates() {
    let ll = proof("!(A & B) |- !A * !B", Mode::CLASSICAL);
    let derivation = ll.derivation().unwrap();
    pin_certificate("ll", &derivation);
    let options = rocq::Options {
        form: Form::Standalone,
        lemma: "bang_with".to_owned(),
        prelude: "Require Import kernel.".to_owned(),
    };
    let script = rocq::derivation(&derivation, &options).unwrap();
    assert!(script.starts_with("Require Import kernel.\n\nLemma bang_with (A B : formula) : ll ["));
    assert!(script.contains("apply (co_r_ext []); cbn_sequent.\n"));
    assert!(script.ends_with("\nQed."));

    let options = rocq::Options::default();
    let mix = proof("A, B |- A, B", Mode::CLASSICAL.with_mix());
    assert_eq!(
        rocq::derivation(&mix.derivation().unwrap(), &options),
        Err(Unsupported::Mix)
    );
    let affine = proof("A, B |- A", Mode::CLASSICAL.affine());
    assert_eq!(
        rocq::derivation(&affine.derivation().unwrap(), &options),
        Err(Unsupported::AffineWeakening)
    );
}

/// Sequents print as math, one-sided or two-sided, with longer names in
/// italic and escaped; Greek names and those of the LLTP library are
/// pinned as a document that pdfLaTeX compiles.
#[test]
fn sequents() {
    let sequent: Sequent = "α, Γ -o P‿a·b |- P‿a·b * Γα".parse().unwrap();
    let options = latex::Options {
        form: Form::Standalone,
        ..latex::Options::default()
    };
    snapshot("names.tex", &latex::sequent(&sequent, &options));

    let sequent: Sequent = "x_1 * foo, !A |- ?B & 1, B".parse().unwrap();
    assert_eq!(
        latex::sequent(&sequent, &latex::Options::default()),
        r"$\vdash \mathit{x\_1}^\bot \parr \mathit{foo}^\bot, \wn A^\bot, B, \wn B \with \mathbf{1}$"
    );
    assert_eq!(
        typst::sequent(&sequent, &typst::Options::default()),
        r#"$⊢ italic("x_1")^⊥ ⅋ italic("foo")^⊥, class("normal", ?)A^⊥, B, class("normal", ?)B class("binary", \&) bold(1)$"#
    );

    let sequent: Sequent = "!A, A -o (B + top) |- B & 0".parse().unwrap();
    let forest = Forest::new(&sequent).unwrap();
    let reading = Reading::new(&forest).unwrap();
    assert_eq!(
        latex::two_sided(&reading, &latex::Options::default()),
        r"$\oc A, A \multimap (B \oplus \top) \vdash B \with 0$"
    );
    assert_eq!(
        typst::two_sided(&reading, &typst::Options::default()),
        r#"$!A, A ⊸ (B ⊕ ⊤) ⊢ B class("binary", \&) 0$"#
    );
    assert!(
        svg::two_sided(&reading, &Style::default())
            .contains("<title>!A, A ⊸ (B ⊕ ⊤) ⊢ B &amp; 0</title>")
    );
}

/// Proof nets are pinned as SVG: a net whose links cross, a structure
/// whose switching cycle is highlighted, and one whose second part is.
#[test]
fn nets() {
    let net = ProofStructure::from_proof(&proof("A * B |- B * A", Mode::CLASSICAL), false);
    snapshot("net.svg", &svg::net(&net.unwrap(), &Style::default()));

    let forest = Forest::new(&"|- A * ~A".parse().unwrap()).unwrap();
    let links = [(OccId::new(1), OccId::new(2))];
    let cyclic = ProofStructure::from_links(forest, false, &links).unwrap();
    assert!(cyclic.is_correct().is_err());
    snapshot("cycle.svg", &svg::net(&cyclic, &Style::default()));

    // ⊢ A ⅋ B, ~A, ~B: the part of `B` and `~B` hangs off the right premise.
    let forest = Forest::new(&"|- A par B, ~A, ~B".parse().unwrap()).unwrap();
    let links = [
        (OccId::new(1), OccId::new(3)),
        (OccId::new(2), OccId::new(4)),
    ];
    let parted = ProofStructure::from_links(forest, false, &links).unwrap();
    let drawing = svg::net(&parted, &Style::default());
    let highlight = format!(r#"stroke="{}""#, Style::default().highlight);
    assert!(drawing.contains(&highlight), "{drawing}");
    snapshot("disconnected.svg", &drawing);
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
    let drawn = svg::derivation(&two_sided.two_sided_derivation().unwrap(), &style);
    assert_eq!(structure(&drawn), [11, 5, 0]);

    let sequent: Sequent = "A, A -o B |- B".parse().unwrap();
    let mut state = Interactive::new(&sequent, Mode::INTUITIONISTIC).unwrap();
    let goals = state.apply(InfId::new(0), 1, Rule::ImpLeft, &[0]).unwrap();
    state.apply(goals[0], 0, Rule::Ax, &[]).unwrap();
    assert_eq!(
        structure(&svg::derivation(&state.derivation(), &style)),
        [6, 2, 0]
    );

    let net = ProofStructure::from_proof(&proof("A * B |- B * A", Mode::CLASSICAL), false);
    assert_eq!(structure(&svg::net(&net.unwrap(), &style)), [8, 8, 2]);

    let sequent: Sequent = "x_1 * foo |- A".parse().unwrap();
    assert_eq!(structure(&svg::sequent(&sequent, &style)), [5, 0, 0]);
}

/// PNG and PDF render a drawing with the font given: the PNG carries the
/// drawing's title, every PDF profile passes krilla's own validation, the
/// accessible one is tagged, and a PDF without a date is refused.
#[cfg(all(feature = "png", feature = "pdf"))]
#[test]
fn renders() {
    use linlog::export::{RenderError, pdf, png};
    let font =
        std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("../cli/fonts/Euler-Math.otf"))
            .unwrap();
    let derivation = proof("A, A -o B |- B", Mode::INTUITIONISTIC);
    let drawing = svg::derivation(
        &derivation.two_sided_derivation().unwrap(),
        &Style::default(),
    );
    let image = png::from_svg(&drawing, &[&font], &png::Options::default()).unwrap();
    assert!(image.starts_with(b"\x89PNG"));
    assert!(image.windows(5).any(|w| w == b"Title"));
    let date = Some(pdf::Date::from_unix(1_791_158_399));
    for (compatible, accessible, version) in [
        (false, false, "%PDF-2.0"),
        (true, false, "%PDF-1.7"),
        (false, true, "%PDF-1.7"),
    ] {
        let options = pdf::Options {
            compatible,
            accessible,
            date,
            ..pdf::Options::default()
        };
        let document = pdf::from_svg(&drawing, &[&font], &options).unwrap();
        assert!(
            document.starts_with(version.as_bytes()),
            "{compatible} {accessible}"
        );
        let root = b"/StructTreeRoot";
        let tagged = document.windows(root.len()).any(|w| w == root);
        assert_eq!(tagged, accessible);
    }
    assert_eq!(
        pdf::from_svg(&drawing, &[&font], &pdf::Options::default()),
        Err(RenderError::NoDate)
    );
}
