// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

use crate::argument_parsing::Format;
use crate::argument_parsing::Threads;
use crate::argument_parsing::{InteractArgs, threads};
use crate::limit::{Deadline, Notice};
use crate::prove::{
    Ended, Rendered, Show, Shown, alone_first, bytes_text, derivation, describe, notice_line,
    on_large_stack, render, stopped, unknown,
};
use crate::style;
use crate::{Status, catch_interrupt, clear_interrupt, interrupted, io};
use anyhow::{Context, Result, bail};
use linlog::export::Styles;
use linlog::export::{latex, svg, typst};
use linlog::proofs::interactive::Needs;
use linlog::search::{Engine, Options, Outcome, Verdict, engine_for, prove_goal};
use linlog::{
    Error, GoalId, Interactive, Limits, Named, Reading, Refusal, Side, Step, ViewOptions,
};
use std::fmt::Write as _;
use std::io::{BufRead, IsTerminal, Write};
use std::path::Path;
use std::time::{Duration, Instant};

/// The commands, as `help` prints them.
const HELP: &str = "\
goals               the open goals, with the positions of their formulas
rules G P           the rules that act on formula P of goal G
apply G P RULE [P…] apply a rule; further positions go to the left premise of a ⊗ or Mix
undo                retract the last step
close [G]           let the search close goal G, or every open goal
show [--FORMAT] [FILE]
                    the derivation so far, open goals included, printed or written to FILE:
                    as text, or --latex, --typst, --svg, --png or --pdf (these two need a
                    FILE); with a FILE and no format, the one its extension names
proof [--FORMAT] [FILE]
                    check the finished proof and print it or write it to FILE: as text,
                    or --json, --latex, --typst, --svg, --png, --pdf, or --rocq for a
                    certificate (--png and --pdf need a FILE);
                    with a FILE and no format, the one its extension names, else JSON
save FILE           write the session as JSON
load FILE           resume a session written by save
help                this list
quit                end the session";

/// Runs `interact`: starts or resumes a session, then reads commands from
/// standard input until `quit` or the end of input, on a thread whose stack
/// fits the recursion limit.
///
/// # Errors
///
/// A sequent or a saved session that cannot be read or has no session,
/// or a terminal on standard input; a command's error is its output and
/// the session goes on.
pub fn interact(args: &InteractArgs) -> Result<Status> {
    let state = match &args.state {
        Some(path) => load(path)?,
        None => {
            if args.input.sequent.is_none() && args.input.file.is_empty() {
                bail!(
                    "no sequent given: pass it as an argument or with --file, since standard input carries the commands"
                );
            }
            let sequent = args.input.sequent()?;
            Interactive::new(&sequent, args.mode.mode()).map_err(|e| describe(e, &sequent))?
        }
    };
    let options = Options::default()
        .with_memo_limit(args.memo_limit)
        .with_copies(args.copies.0)
        .with_bias(args.bias.into())
        .with_forward_copies(args.forward_copies);
    let limits = args.limits();
    let threads = threads(args.jobs, args.pool_after, args.deterministic);
    let options = options.with_jobs(threads.jobs);
    catch_interrupt();
    let stack_size = limits.stack_bytes();
    let styles = style::read(&args.style, None, false)?;
    let mut session = Session {
        styles,
        state,
        options,
        view: ViewOptions::default(),
        limits,
        timeout: args.timeout.0,
        threads,
        deepens: args.copies.0.is_none(),
        recursion_limit: args.recursion_limit,
    };
    on_large_stack(stack_size, move || session.run())?
}

/// What a target's `write` makes of a derivation, as a string.
fn written(write: impl FnOnce(&mut String) -> Result<(), linlog::Error>) -> Result<String> {
    let mut out = String::new();
    write(&mut out)?;
    Ok(out)
}

/// Reads a session from a JSON file.
fn load(path: &Path) -> Result<Interactive> {
    let text = io::read(Some(path), "session")?;
    serde_json::from_str(&text)
        .with_context(|| format!("{} is not a saved session", path.display()))
}

/// A running session: the state, the search settings and the styles of
/// the outputs.
struct Session {
    /// How `show` and `proof` write each format.
    styles: Styles,
    /// The proof in progress.
    state: Interactive,
    /// The search settings of `close`.
    options: Options,
    /// How the derivation `close` grafts is shown.
    view: ViewOptions,
    /// The bounds of a `close`'s search and of the derivation it grafts.
    limits: Limits,
    /// How long a `close` may take.
    timeout: Option<Duration>,
    /// The threads of a `close`, and how long one searches alone.
    threads: Threads,
    /// Whether a `close` deepens the copy bound without a bound.
    deepens: bool,
    /// The recursion limit of a `close`.
    recursion_limit: u32,
}

impl Session {
    /// Reads and runs commands until `quit` or the end of input, and returns
    /// whether the proof was finished and checks.
    fn run(&mut self) -> Result<Status> {
        let stdin = std::io::stdin();
        let prompt = stdin.is_terminal();
        let mut stdout = std::io::stdout();
        let mut lines = stdin.lock().lines();
        loop {
            if prompt {
                write!(stdout, "> ")?;
                stdout.flush()?;
            }
            let Some(line) = lines.next() else {
                break;
            };
            let line = line.context("cannot read standard input")?;
            let words: Vec<&str> = line.split_whitespace().collect();
            if words.first() == Some(&"quit") {
                break;
            }
            let text = match self.command(&words) {
                Ok(text) => text,
                Err(e) => format!("error: {e:#}"),
            };
            if !text.is_empty() {
                writeln!(stdout, "{text}")?;
            }
        }
        Ok(
            if self.state.is_complete() && self.state.proof(&self.limits, |_| false).is_ok() {
                Status::Yes
            } else {
                Status::No
            },
        )
    }

    /// Runs one command and returns what to print.
    fn command(&mut self, words: &[&str]) -> Result<String> {
        let (command, rest) = words.split_first().map_or(("", &[][..]), |(c, r)| (*c, r));
        let positions = |from: usize| -> Result<Vec<usize>> {
            rest.get(from..)
                .unwrap_or(&[])
                .iter()
                .map(|w| {
                    w.parse()
                        .with_context(|| format!("{w:?} is not a position"))
                })
                .collect()
        };
        let goal = |i: usize| -> Result<GoalId> {
            let word = rest.get(i).context("which goal? see `goals`")?;
            let id: u32 = word
                .parse()
                .with_context(|| format!("{word:?} is not a goal"))?;
            Ok(GoalId::new(id))
        };
        let position = |i: usize| -> Result<usize> {
            let word = rest.get(i).context("which formula? see `goals`")?;
            word.parse()
                .with_context(|| format!("{word:?} is not a position"))
        };
        Ok(match command {
            "" => String::new(),
            "help" => HELP.to_owned(),
            "goals" => self.goals(),
            "rules" => {
                let rules = self.state.rules(goal(0)?, position(1)?)?;
                if rules.is_empty() {
                    "no rule acts on it".to_owned()
                } else {
                    rules
                        .iter()
                        .map(|r| match r.needs {
                            Needs::Split => format!("{} (with a split)", r.rule),
                            _ => r.rule.to_string(),
                        })
                        .collect::<Vec<_>>()
                        .join("  ")
                }
            }
            "apply" => {
                let (goal, position) = (goal(0)?, position(1)?);
                let word = rest.get(2).context("which rule? see `rules`")?;
                let rule: Named = word.parse()?;
                let step = Step::new(position, rule).left(&positions(3)?);
                let opened = self.state.apply(goal, &step)?;
                self.opened(&opened)
            }
            "undo" => match self.state.undo() {
                Some(goal) => format!("reopened {}", self.goal_line(goal)),
                None => "nothing to undo".to_owned(),
            },
            "close" => {
                let goals = match rest.first() {
                    Some(_) => vec![goal(0)?],
                    None => self.state.goals().collect(),
                };
                let mut text = String::new();
                for goal in goals {
                    let (outcome, ended) = self.close(goal)?;
                    let _ = writeln!(text, "goal {}: {}", goal.get(), verdict(&outcome, &ended));
                }
                text.pop();
                if self.state.is_complete() {
                    text.push_str("\nno goal is open: `proof` checks the proof");
                }
                text
            }
            "show" => {
                let (word, path) = match rest {
                    [] => (None, None),
                    [word] if word.starts_with("--") => (Some(*word), None),
                    [path] => (None, Some(*path)),
                    [word, path] if word.starts_with("--") => (Some(*word), Some(*path)),
                    _ => bail!("show [--FORMAT] [FILE]: one format and one file at most"),
                };
                let format = match (word, path) {
                    (Some(word), _) => proof_format(word)?,
                    (None, Some(path)) => Format::of_path(Path::new(path)).unwrap_or(Format::Text),
                    (None, None) => Format::Text,
                };
                let derivation = self
                    .state
                    .derivation_within(&self.view, &self.limits, |_| false)?;
                let styles = &self.styles;
                let text = match format {
                    Format::Text => {
                        let mut text = String::new();
                        derivation.write_text(&styles.text, &mut text, |_| false)?;
                        text
                    }
                    Format::Latex => {
                        written(|out| latex::write(&derivation, &styles.latex, out, |_| false))?
                    }
                    Format::Typst => {
                        written(|out| typst::write(&derivation, &styles.typst, out, |_| false))?
                    }
                    Format::Svg | Format::Png | Format::Pdf => {
                        written(|out| svg::write(&derivation, &styles.svg, out, |_| false))?
                    }
                    Format::Json | Format::Rocq => bail!(
                        "show writes the derivation so far as --text, --latex, --typst, --svg, \
                         --png or --pdf; `save` writes the session, `proof --rocq` a finished proof"
                    ),
                };
                match (path, format.is_binary()) {
                    (None, true) => bail!("a {} needs a FILE to be written to", format.title()),
                    (None, false) => text,
                    (Some(path), true) => {
                        // A Ctrl-C of an earlier `close` must not stop it.
                        clear_interrupt();
                        match render(text, format, styles, self.limits.memory_bytes, &interrupted)?
                        {
                            Rendered::Bytes(bytes) => {
                                let mut out = io::Output::open(Some(Path::new(path)), true)?;
                                out.stream().write_all(&bytes)?;
                                out.finish()?;
                                format!("derivation so far written to {path}")
                            }
                            Rendered::Refused(line) => line,
                            Rendered::Stopped => {
                                "the derivation so far is not written: interrupted".to_owned()
                            }
                        }
                    }
                    (Some(path), false) => {
                        let mut out = io::Output::open(Some(Path::new(path)), false)?;
                        out.write_str(&text)?;
                        out.finish()?;
                        format!("derivation so far written to {path}")
                    }
                }
            }
            "proof" => {
                let (format, path) = match rest {
                    [] => (None, None),
                    [word] if word.starts_with("--") => (Some(proof_format(word)?), None),
                    [path] => (None, Some(*path)),
                    [word, path] if word.starts_with("--") => {
                        (Some(proof_format(word)?), Some(*path))
                    }
                    _ => bail!("proof [--FORMAT] [FILE]: one format and one file at most"),
                };
                let format = match (format, path) {
                    (Some(format), _) => format,
                    (None, Some(path)) => Format::of_path(Path::new(path)).unwrap_or(Format::Json),
                    (None, None) => Format::Text,
                };
                if format.is_binary() && path.is_none() {
                    bail!("a {} needs a FILE to be written to", format.title());
                }
                let proof = self.state.proof(&self.limits, |_| false)?;
                let mode = self.state.mode();
                let mut text = String::new();
                if path.is_none() {
                    text = format!("valid proof ({mode})");
                }
                if format == Format::Json {
                    let separator = if text.is_empty() { "" } else { "\n" };
                    text = format!("{text}{separator}{}", serde_json::to_string(&proof)?);
                } else {
                    let mut show =
                        Show::session(format, self.view, self.limits, self.styles.clone());
                    show.verdict = path.is_none();
                    // A Ctrl-C of an earlier `close` must not stop it.
                    clear_interrupt();
                    let stopped = || "interrupted".to_owned();
                    let prefix = (!text.is_empty()).then_some("\n");
                    match derivation(&proof, mode, &show, interrupted, stopped, prefix, &mut text)?
                    {
                        Shown::LeftOut(line) | Shown::Cut(line) => {
                            text.push('\n');
                            text.push_str(&line);
                        }
                        Shown::Rendered(bytes) => {
                            let path = path.expect("a binary format has a file");
                            let mut out = io::Output::open(Some(Path::new(path)), true)?;
                            out.stream().write_all(&bytes)?;
                            out.finish()?;
                            return Ok(format!("valid proof written to {path}"));
                        }
                        Shown::Written | Shown::Nothing => {}
                    }
                }
                match path {
                    Some(path) => {
                        io::write(Some(Path::new(path)), &text)?;
                        format!("valid proof written to {path}")
                    }
                    None => text,
                }
            }
            "save" => {
                let path = rest.first().context("save where? give a file")?;
                io::write(Some(Path::new(path)), &serde_json::to_string(&self.state)?)?;
                format!("session written to {path}")
            }
            "load" => {
                let path = rest.first().context("load what? give a file")?;
                let state = load(Path::new(path))?;
                // The exit status answers for the session's sequent and
                // mode, so a file of another question is refused, as
                // `--state` beside a sequent is.
                if state.sequent() != self.state.sequent() {
                    bail!(
                        "{path} holds a session of another sequent; `load` resumes one of this \
                         sequent, and `linlog interact --state {path}` starts that one"
                    );
                }
                if state.mode() != self.state.mode() {
                    bail!(
                        "{path} holds a session in {} mode, this one is in {} mode; \
                         `linlog interact --state {path}` starts that one",
                        state.mode(),
                        self.state.mode()
                    );
                }
                self.state = state;
                self.goals()
            }
            _ => bail!("unknown command {command:?}; `help` lists them"),
        })
    }

    /// Runs the search on a goal, on one thread first as `prove` does,
    /// stopped by the time limit or Ctrl-C, and returns its outcome with
    /// how it ended.
    fn close(&mut self, goal: GoalId) -> Result<(Outcome, Ended)> {
        clear_interrupt();
        let start = Instant::now();
        let deadline = Deadline::start(self.timeout, start)?;
        let notice = Notice::start(
            crate::prove::NOTICE_AFTER,
            notice_line(self.timeout, self.deepens),
        );
        let halt = || interrupted() || deadline.passed();
        let goal_sequent = self
            .state
            .goal(goal)?
            .iter()
            .map(|&m| self.state.occurrence(m))
            .collect::<Vec<_>>();
        let (forest, mode) = (self.state.forest(), self.state.mode());
        let parallel =
            || engine_for(forest, &goal_sequent, mode, &self.options).is_ok_and(Engine::parallel);
        let limits = &self.limits;
        let searched = alone_first(
            &self.options,
            limits.stack_bytes(),
            self.threads,
            &halt,
            parallel,
            |options, halt| prove_goal(forest, &goal_sequent, mode, options, limits, |_| halt()),
        );
        let closed = searched.and_then(|outcome| {
            if let Verdict::Proved(proof) = &outcome.verdict {
                self.state
                    .close_with(goal, proof, &self.view, &self.limits, |_| halt())?;
            }
            Ok(outcome)
        });
        drop(notice);
        let ended = Ended {
            stop: stopped(&deadline),
            elapsed: start.elapsed(),
            recursion_limit: self.recursion_limit,
        };
        match closed {
            Ok(outcome) => Ok((outcome, ended)),
            Err(Error::Refused(Refusal::Output {
                estimate_bytes,
                limit_bytes,
                ..
            })) => bail!(
                "the search proved the goal, but the derivation to graft is too large: it is \
                 estimated at {}, over the limit of {}; the goal stays open (--derivation-limit \
                 raises the limit)",
                bytes_text(estimate_bytes),
                bytes_text(limit_bytes)
            ),
            Err(Error::Refused(Refusal::Stopped { .. })) => bail!(
                "the search proved the goal, but its derivation was not grafted before the \
                 time limit or the interrupt; the goal stays open"
            ),
            Err(error) => Err(error.into()),
        }
    }

    /// Lists the open goals, or says that none is.
    fn goals(&self) -> String {
        let goals: Vec<GoalId> = self.state.goals().collect();
        if goals.is_empty() {
            return "no goal is open: `proof` checks the proof".to_owned();
        }
        goals
            .iter()
            .map(|&g| self.goal_line(g))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Lists goals just opened, or says that the step closed its goal.
    fn opened(&self, goals: &[GoalId]) -> String {
        if goals.is_empty() {
            return if self.state.is_complete() {
                "closed; no goal is open: `proof` checks the proof".to_owned()
            } else {
                "closed".to_owned()
            };
        }
        goals
            .iter()
            .map(|&g| format!("opened {}", self.goal_line(g)))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Returns `goal G: ` and the goal's sequent with the position of every
    /// formula, two-sided in intuitionistic mode.
    fn goal_line(&self, goal: GoalId) -> String {
        let sequent: Vec<_> = (self.state.goal(goal).unwrap_or(&[]).iter())
            .map(|&m| self.state.occurrence(m))
            .collect();
        let forest = self.state.forest();
        let reading = self.state.reading();
        let mut line = format!("goal {}:", goal.get());
        let formula = |o| match &reading {
            Some(reading) => reading.formula(o).to_string(),
            None => forest.formula(o).to_string(),
        };
        let side = |o, side| {
            reading
                .as_ref()
                .is_none_or(|r: &Reading| r.position(o) == side)
        };
        for (i, &o) in sequent.iter().enumerate() {
            if side(o, Side::Input) && reading.is_some() {
                let _ = write!(
                    line,
                    "{}{i}: {}",
                    if i == 0 { " " } else { ", " },
                    formula(o)
                );
            }
        }
        line.push_str(" ⊢");
        let mut first = true;
        for (i, &o) in sequent.iter().enumerate() {
            if reading.is_none() || side(o, Side::Output) {
                let _ = write!(
                    line,
                    "{}{i}: {}",
                    if first { " " } else { ", " },
                    formula(o)
                );
                first = false;
            }
        }
        line
    }
}

/// Returns the format a word of `proof` names, `--latex` and so on.
fn proof_format(word: &str) -> Result<Format> {
    Ok(match word {
        "--text" => Format::Text,
        "--json" => Format::Json,
        "--latex" => Format::Latex,
        "--typst" => Format::Typst,
        "--svg" => Format::Svg,
        "--png" => Format::Png,
        "--pdf" => Format::Pdf,
        "--rocq" => Format::Rocq,
        _ => bail!(
            "proof {word}? the formats are --text, --json, --latex, --typst, --svg, --png, \
             --pdf and --rocq"
        ),
    })
}

/// Returns the verdict of a `close` as one line.
fn verdict(outcome: &Outcome, ended: &Ended) -> String {
    let context = format!(
        "{}, {}, {} engine",
        outcome.fragment.name_in(outcome.mode),
        outcome.mode,
        outcome.engine
    );
    match &outcome.verdict {
        Verdict::Proved(_) => format!("proved ({context})"),
        Verdict::Unprovable(refutation) => format!("unprovable ({context}): {refutation}"),
        Verdict::Unknown(reason) => {
            format!("unknown ({context}): {}", unknown(*reason, outcome, ended))
        }
    }
}
