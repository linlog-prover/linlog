// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The journeys: small, deterministic runs of the library's hot paths
//! (searches of the target set's rows and of the families, the readers,
//! the checker, a derivation, the renderers, a batch, ordinary logic),
//! whose instruction counts under callgrind the ratchet compares with
//! committed ceilings. Each journey prepares its input first and does the
//! work it measures inside [`measured`], which callgrind's
//! `--toggle-collect` counts alone. Every search runs on one thread, so a
//! count is a function of the build and the input.

use anyhow::{Context, Result, bail};
use linlog::export::{latex, svg, typst};
use linlog::ordinary::{self, Logic, Translation};
use linlog::proofs::TextOptions;
use linlog::search::{Bias, batch};
use linlog::{Limits, Mode, Options, Proof, Sequent, Verdict, ViewOptions};
use std::fmt::Write as _;
use std::hint::black_box;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// One journey: its name, what it runs, and the run.
pub struct Journey {
    /// The name the ratchet's file and the command use.
    pub name: &'static str,
    /// What it runs, for the listing.
    pub summary: &'static str,
    /// The run: its preparation, then its measured part in [`measured`].
    pub run: fn() -> Result<()>,
}

/// Every journey, in the order of the ceilings' file.
pub static JOURNEYS: &[Journey] = &[
    Journey {
        name: "search-qbf-20-2",
        summary: "the target set's qbf/20#2, the focused engine",
        run: || search_family("qbf", 20, 2),
    },
    Journey {
        name: "search-chain-128",
        summary: "the target set's chain/128, the focused engine with many copies",
        run: || search_family("chain", 128, 0),
    },
    Journey {
        name: "search-partition-no-5",
        summary: "the target set's partition-no/5, a refutation of the focused engine",
        run: || search_family("partition-no", 5, 0),
    },
    Journey {
        name: "search-wide-m2-256",
        summary: "wide-m2/256, every literal twice, the net engine",
        run: || search_family("wide-m2", 256, 0),
    },
    Journey {
        name: "search-spec-chain",
        summary: "the reachability of the read-spec journey's chain, the Horn engine in affine mode",
        run: search_spec,
    },
    Journey {
        name: "search-chain-64-intuitionistic",
        summary: "chain/64 in intuitionistic mode, the two-sided engine",
        run: || search_intuitionistic("chain", 64),
    },
    Journey {
        name: "search-additive-14",
        summary: "additive/14, the additive path",
        run: || search_family("additive", 14, 0),
    },
    Journey {
        name: "read-text",
        summary: "the text syntax: wide-m1/2048 printed and read back",
        run: read_text,
    },
    Journey {
        name: "read-json",
        summary: "the JSON form of a sequent: wide-m1/2048",
        run: read_json,
    },
    Journey {
        name: "read-lltp",
        summary: "an LLTP problem of 2 000 clauses",
        run: read_lltp,
    },
    Journey {
        name: "read-tptp",
        summary: "a TPTP problem of 2 000 formulas",
        run: read_tptp,
    },
    Journey {
        name: "read-spec",
        summary: "a coverability problem of 2 000 rules in Mist's format",
        run: read_spec,
    },
    Journey {
        name: "check-qbf-20-2",
        summary: "the checker on the proof of qbf/20#2",
        run: || check("qbf", 20, 2),
    },
    Journey {
        name: "check-wide-m1-2048",
        summary: "the checker on the proof of wide-m1/2048",
        run: || check("wide-m1", 2048, 0),
    },
    Journey {
        name: "derivation-chain-64",
        summary: "the derivation of chain/64's proof, built and written as text",
        run: derivation,
    },
    Journey {
        name: "render-latex",
        summary: "LaTeX of the derivation of chain/64",
        run: || render(Render::Latex),
    },
    Journey {
        name: "render-typst",
        summary: "Typst of the derivation of chain/64",
        run: || render(Render::Typst),
    },
    Journey {
        name: "render-svg",
        summary: "SVG of the derivation of chain/64",
        run: || render(Render::Svg),
    },
    Journey {
        name: "batch-families",
        summary: "a batch of the families' smallest instances on one worker",
        run: batch,
    },
    Journey {
        name: "ordinary-pigeons",
        summary: "ordinary logic: a classical pigeonhole formula translated, proved, read back to LK and checked",
        run: ordinary,
    },
];

/// Runs `work` and returns its result: the part of a journey callgrind
/// counts. Never inlined, so that callgrind sees it by name.
#[inline(never)]
pub fn measured<R>(work: impl FnOnce() -> R) -> R {
    let start = Instant::now();
    let result = black_box(work());
    ELAPSED.store(
        u64::try_from(start.elapsed().as_nanos()).unwrap_or(u64::MAX),
        Ordering::Relaxed,
    );
    result
}

/// The wall-clock time of the last [`measured`] part, in nanoseconds.
static ELAPSED: AtomicU64 = AtomicU64::new(0);

/// Runs the named journey once, or `repeat` times printing the wall-clock
/// time of each measured part in nanoseconds on a line of its own.
pub fn run(name: &str, repeat: Option<u32>) -> Result<()> {
    let Some(journey) = JOURNEYS.iter().find(|j| j.name == name) else {
        bail!("no journey {name:?}; `linlog-bench journeys` lists them");
    };
    match repeat {
        None => (journey.run)(),
        Some(n) => {
            for _ in 0..n {
                (journey.run)()?;
                println!("{}", ELAPSED.load(Ordering::Relaxed));
            }
            Ok(())
        }
    }
}

/// The options of a search of the target set: one thread, the problem's
/// copy bound or the default, no check (a journey of its own). With
/// exponentials the default bias runs two searches in turns, whose waiting
/// thread wakes on a clock, so a count would depend on the time: such a
/// journey names the rarer literal's search alone.
fn options(sequent: &Sequent, copies: Option<u32>) -> Options {
    let bias = if sequent.fragment().has_exponentials() {
        Bias::Rarer
    } else {
        Bias::Auto
    };
    Options::default()
        .with_jobs(1)
        .with_bias(bias)
        .with_copies(Some(copies.unwrap_or(Options::DEFAULT_COPIES)))
        .with_check(false)
}

/// An instance of a family.
fn instance(family: &str, size: u32, index: u32) -> Result<linlog::families::Instance> {
    let family = linlog::families::find(family).with_context(|| format!("no family {family}"))?;
    Ok(family.instance(size, index))
}

/// Decides an instance of a family as the target set does.
fn search_family(family: &str, size: u32, index: u32) -> Result<()> {
    let problem = instance(family, size, index)?;
    let options = options(&problem.sequent, problem.copies);
    let outcome = measured(|| linlog::prove(&problem.sequent, problem.mode, &options))?;
    decided(&outcome.verdict, problem.provable)
}

/// Decides an instance of a family in intuitionistic mode.
fn search_intuitionistic(family: &str, size: u32) -> Result<()> {
    let problem = instance(family, size, 0)?;
    let mut mode = Mode::INTUITIONISTIC;
    if problem.mode.is_affine() {
        mode = mode.with_affine();
    }
    if problem.mode.has_mix() {
        mode = mode.with_mix();
    }
    let options = options(&problem.sequent, problem.copies);
    let outcome = measured(|| linlog::prove(&problem.sequent, mode, &options))?;
    decided(&outcome.verdict, problem.provable)
}

/// Fails unless the verdict is the one expected: a journey that stops
/// deciding measures something else.
fn decided(verdict: &Verdict, provable: bool) -> Result<()> {
    match verdict {
        Verdict::Proved(_) if provable => Ok(()),
        Verdict::Unprovable(_) if !provable => Ok(()),
        other => bail!(
            "expected {}, got {other:?}",
            if provable { "a proof" } else { "a refutation" }
        ),
    }
}

/// A proof of a provable instance of a family.
fn proof(family: &str, size: u32, index: u32) -> Result<Proof> {
    let problem = instance(family, size, index)?;
    match linlog::prove(
        &problem.sequent,
        problem.mode,
        &options(&problem.sequent, problem.copies),
    )?
    .verdict
    {
        Verdict::Proved(proof) => Ok(*proof),
        other => bail!("{family}/{size}: {other:?}"),
    }
}

/// The sequent the readers' text and JSON journeys read.
fn wide() -> Result<Sequent> {
    Ok(instance("wide-m1", 2048, 0)?.sequent)
}

/// Reads a sequent's text.
fn read_text() -> Result<()> {
    let text = wide()?.to_string();
    measured(|| text.parse::<Sequent>())?;
    Ok(())
}

/// Reads a sequent's JSON form.
fn read_json() -> Result<()> {
    let json = serde_json::to_string(&wide()?)?;
    measured(|| serde_json::from_str::<Sequent>(&json))?;
    Ok(())
}

/// The number of clauses, formulas or rules of the readers' problems.
const LINES: usize = 2000;

/// Reads an LLTP problem of a chain of clauses.
fn read_lltp() -> Result<()> {
    let mut text = String::from("% Status : Theorem\n");
    for i in 0..LINES {
        writeln!(
            text,
            "fof(c{i}, axiom, !(p{i} -o (p{} * (q{i} | ~q{i})))).",
            i + 1
        )?;
    }
    writeln!(
        text,
        "fof(start, axiom, p0).\nfof(goal, conjecture, p{LINES})."
    )?;
    measured(|| linlog::lltp::read(&text, &Limits::default()))?;
    Ok(())
}

/// Reads a TPTP problem of a chain of implications.
fn read_tptp() -> Result<()> {
    let mut text = String::from("% Status : Theorem\n");
    for i in 0..LINES {
        writeln!(text, "fof(a{i}, axiom, (p{i} => (p{} & ~ ~ q{i}))).", i + 1)?;
    }
    writeln!(text, "fof(c, conjecture, (p0 => p{LINES})).")?;
    measured(|| ordinary::read_tptp(&text))?;
    Ok(())
}

/// A coverability problem of a chain of transitions, each moving the one
/// token on: the last place is reachable.
fn spec() -> Result<String> {
    let mut text = String::from("vars\n");
    for i in 0..=LINES {
        write!(text, " v{i}")?;
    }
    text.push_str("\n\nrules\n");
    for i in 0..LINES {
        writeln!(
            text,
            "v{i} >= 1 -> v{i}' = v{i}-1, v{}' = v{}+1;",
            i + 1,
            i + 1
        )?;
    }
    text.push_str("\ninit\nv0=1");
    for i in 1..=LINES {
        write!(text, ", v{i}=0")?;
    }
    writeln!(text, "\n\ntarget\nv{LINES} >= 1")?;
    Ok(text)
}

/// Reads a coverability problem.
fn read_spec() -> Result<()> {
    let text = spec()?;
    measured(|| linlog::mist::read(&text, &Limits::default()))?;
    Ok(())
}

/// Decides a coverability problem as the harness does, intuitionistic
/// affine.
fn search_spec() -> Result<()> {
    let problem = linlog::mist::read(&spec()?, &Limits::default())?;
    let mode = Mode::INTUITIONISTIC.with_affine();
    let options = options(&problem.sequent, None);
    let outcome = measured(|| linlog::prove(&problem.sequent, mode, &options))?;
    decided(&outcome.verdict, true)
}

/// Checks the proof of a provable instance of a family.
fn check(family: &str, size: u32, index: u32) -> Result<()> {
    let proof = proof(family, size, index)?;
    measured(|| proof.check(Mode::CLASSICAL))?;
    Ok(())
}

/// Builds a derivation and writes it as text.
fn derivation() -> Result<()> {
    let proof = proof("chain", 64, 0)?;
    measured(|| -> Result<usize> {
        let derivation =
            proof.derivation_within(&ViewOptions::default(), &Limits::default(), |_| false)?;
        let mut text = String::new();
        derivation.write_text(&TextOptions::default(), &mut text, |_| false)?;
        Ok(text.len())
    })?;
    Ok(())
}

/// The renderers a journey measures.
enum Render {
    /// `export::latex`.
    Latex,
    /// `export::typst`.
    Typst,
    /// `export::svg`.
    Svg,
}

/// Renders the derivation of a proof.
fn render(format: Render) -> Result<()> {
    let proof = proof("chain", 64, 0)?;
    let derivation = proof.derivation()?;
    let length = measured(|| match format {
        Render::Latex => latex::derivation(&derivation, &latex::Options::default()).len(),
        Render::Typst => typst::derivation(&derivation, &typst::Options::default()).len(),
        Render::Svg => svg::derivation(&derivation, &svg::Style::default()).len(),
    });
    black_box(length);
    Ok(())
}

/// Decides the smallest instance of every family as one batch.
fn batch() -> Result<()> {
    let problems: Vec<batch::Problem> = linlog::families::FAMILIES
        .iter()
        .map(|family| {
            let problem = family.instance(family.sizes[0], 0);
            batch::Problem {
                name: problem.name,
                sequent: problem.sequent,
                mode: Some(problem.mode),
            }
        })
        .collect();
    let options = batch::Options {
        cores: batch::Cores::Across,
        ..batch::Options::default()
    };
    // The rarer literal's search alone, as `options` takes it with
    // exponentials, for every problem.
    let search = Options::default()
        .with_jobs(1)
        .with_bias(Bias::Rarer)
        .with_check(false);
    let decided =
        measured(|| batch::prove(problems, &options, &search, &Limits::default()).count());
    black_box(decided);
    Ok(())
}

/// Decides a classical pigeonhole formula through its translation, reads
/// the proof back to LK and checks it.
fn ordinary() -> Result<()> {
    // Three pigeons in two holes: some hole holds two of them.
    let p = |i: u32, h: u32| format!("p{i}{h}");
    let choices: Vec<String> = (0..3)
        .map(|i| format!("({} \\/ {})", p(i, 0), p(i, 1)))
        .collect();
    let mut text = choices.join(", ");
    text.push_str(" |- ");
    let mut clashes = Vec::new();
    for h in 0..2 {
        for i in 0..3 {
            for j in i + 1..3 {
                clashes.push(format!("({} /\\ {})", p(i, h), p(j, h)));
            }
        }
    }
    text.push_str(&clashes.join(" \\/ "));
    let sequent: ordinary::Sequent = text.parse()?;
    measured(|| -> Result<()> {
        let image = ordinary::translate(&sequent, Logic::Classical, Translation::Affine)?;
        let outcome = linlog::prove(
            image.sequent(),
            image.mode(),
            &options(image.sequent(), None),
        )?;
        let Verdict::Proved(proof) = outcome.verdict else {
            bail!("the pigeonhole formula is valid: {:?}", outcome.verdict);
        };
        let linear =
            image.linear_derivation(&proof, &ViewOptions::default(), &Limits::default(), |_| {
                false
            })?;
        image.read_back(&linear)?.check()?;
        Ok(())
    })
}
