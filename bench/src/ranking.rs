// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The comparison of tools in the rows of `run --tools`, as Markdown: per
//! tool the problems solved within the time limit (the ranking), how the
//! others ended and those outside its fragment, per group of problems; for
//! linlog on one thread against each other tool what one solves and the
//! other does not; the translation times and peak memory; and every
//! problem on which two tools, or a tool and the problem's known verdict,
//! contradict each other.

use crate::summary::{Row, read_all, time};
use anyhow::{Result, bail};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::PathBuf;

/// The tool every other is set against.
const REFERENCE: &str = "linlog";

/// How many problems a list names before it says how many more there are.
const LISTED: usize = 40;

/// Whether a row decided its problem, in time or late.
fn decided(row: &Row) -> bool {
    matches!(row.get("verdict"), "proved" | "unprovable")
}

/// Whether a row decided its problem after the time limit.
fn late(row: &Row) -> bool {
    let limit: f64 = row.get("timeout_s").parse().unwrap_or(f64::INFINITY);
    decided(row) && row.get("time_ms").parse::<f64>().unwrap_or(f64::INFINITY) > limit * 1000.0
}

/// Whether a row decided its problem within the time limit and not
/// wrongly.
pub(crate) fn solved(row: &Row) -> bool {
    decided(row) && !late(row) && !wrong(row)
}

/// Whether a row's problem lies outside its tool's fragment.
fn outside(row: &Row) -> bool {
    row.get("verdict") == "refused"
}

/// The problem a row ran, the same for every tool.
fn problem(row: &Row) -> String {
    format!(
        "{}/{} ({})",
        row.get("family"),
        row.get("problem"),
        row.get("mode")
    )
}

/// The group of problems a row's belongs to in the tables.
fn group(row: &Row) -> &'static str {
    match row.get("source") {
        "lltp" if row.get("family").contains("petri-nets") => "LLTP Petri nets",
        "lltp" if row.get("mode").starts_with("intuitionistic") => "LLTP intuitionistic",
        "lltp" => "LLTP classical",
        "spec" => "coverability (qcover)",
        "family" => "generated families",
        _ => "problem files",
    }
}

/// How a run that solved nothing ended, in a word.
fn ending(row: &Row) -> &'static str {
    match (row.get("verdict"), row.get("reason")) {
        _ if wrong(row) => "wrong",
        ("refused", _) => "outside",
        ("proved" | "unprovable", _) => "late",
        (_, "timeout" | "stopped") => "time",
        (_, "out_of_memory" | "memory_limit") => "memory",
        ("error", _) => "error",
        _ => "gave up",
    }
}

/// The endings the tables count, in their order.
const ENDINGS: [&str; 7] = [
    "wrong", "late", "time", "memory", "gave up", "error", "outside",
];

/// The column `judge` sets.
const JUDGED: &str = "judged";

/// Whether a row's verdict is wrong (see [`judge`]).
fn wrong(row: &Row) -> bool {
    row.get(JUDGED) == "wrong"
}

/// Whether a row's verdict contradicts the header of an LLTP problem, whose
/// headers are known to be wrong on some files, without being wrong.
fn against_header(row: &Row) -> bool {
    row.get(JUDGED) == "header"
}

/// Marks each row's verdict: `wrong` where it refutes a problem a tool
/// proved with a checked proof, or decides a generated problem against its
/// verdict by construction, or proves without a check what the source
/// says is unprovable; `header` where it only contradicts an LLTP header,
/// which a checked proof or a refutation can show to be wrong.
fn judge(rows: &mut [Row]) {
    let checked: BTreeSet<String> = (rows.iter())
        .filter(|r| r.get("verdict") == "proved" && r.get("checked") == "ok")
        .map(problem)
        .collect();
    for row in rows.iter_mut() {
        let family = row.get("source") == "family";
        let judged = match (row.get("verdict"), row.get("expected")) {
            ("unprovable", _) if checked.contains(&problem(row)) => "wrong",
            ("unprovable", "provable") if family => "wrong",
            ("unprovable", "provable") => "header",
            ("proved", "unprovable") if row.get("checked") == "ok" => "header",
            ("proved", "unprovable") if family => "wrong",
            ("proved", "unprovable") => "header",
            _ => "",
        };
        row.set(JUDGED, judged);
    }
}

/// Prints the comparison of the tools in the files.
pub fn ranking(files: &[PathBuf]) -> Result<()> {
    let mut rows: Vec<Row> = read_all(files)?
        .into_iter()
        .filter(|row| !row.get("tool").is_empty())
        .collect();
    if rows.is_empty() {
        bail!("the files hold no row of `run --tools`");
    }
    judge(&mut rows);
    print!("{}", report(&rows));
    Ok(())
}

/// The whole report on the rows.
fn report(rows: &[Row]) -> String {
    // Per problem, per tool, the first row.
    let mut problems: BTreeMap<String, BTreeMap<&str, &Row>> = BTreeMap::new();
    let mut tools: Vec<&str> = Vec::new();
    for row in rows {
        let tool = row.get("tool");
        if !tools.contains(&tool) {
            tools.push(tool);
        }
        problems
            .entry(problem(row))
            .or_default()
            .entry(tool)
            .or_insert(row);
    }
    let mut out = String::new();
    let limits: BTreeSet<String> = rows
        .iter()
        .map(|r| {
            format!(
                "{} s and {} bytes",
                r.get("timeout_s"),
                r.get("memory_limit")
            )
        })
        .collect();
    let _ = writeln!(
        out,
        "# Tools compared\n\n{} problems, {} tools, limits {}.\n",
        problems.len(),
        tools.len(),
        limits.into_iter().collect::<Vec<_>>().join("; ")
    );
    ranking_table(&mut out, &tools, &problems);
    groups_table(&mut out, &tools, &problems);
    pairs(&mut out, &tools, &problems);
    costs(&mut out, &tools, &problems);
    late_answers(&mut out, &tools, &problems);
    contradictions(&mut out, &tools, &problems);
    out
}

/// The ranking: per tool the problems solved within the limit, and how the
/// others ended.
fn ranking_table(
    out: &mut String,
    tools: &[&str],
    problems: &BTreeMap<String, BTreeMap<&str, &Row>>,
) {
    let _ = writeln!(
        out,
        "## Solved within the limit\n\n| tool | solved | proved | unprovable | {} | against the header |\n|---|---|---|---|{}---|",
        ENDINGS.join(" | "),
        "---|".repeat(ENDINGS.len())
    );
    let mut ranked: Vec<(&str, usize)> = tools
        .iter()
        .map(|&t| {
            let n = problems
                .values()
                .filter(|p| p.get(t).is_some_and(|r| solved(r)))
                .count();
            (t, n)
        })
        .collect();
    ranked.sort_by_key(|&(t, n)| (std::cmp::Reverse(n), t));
    for (tool, n) in ranked {
        let rows: Vec<&Row> = problems
            .values()
            .filter_map(|p| p.get(tool).copied())
            .collect();
        let count = |f: &dyn Fn(&Row) -> bool| rows.iter().filter(|r| f(r)).count();
        let proved = count(&|r| solved(r) && r.get("verdict") == "proved");
        let endings: Vec<String> = ENDINGS
            .iter()
            .map(|e| count(&|r| !solved(r) && ending(r) == *e).to_string())
            .collect();
        let _ = writeln!(
            out,
            "| {tool} | {n} | {proved} | {} | {} | {} |",
            n - proved,
            endings.join(" | "),
            count(&against_header),
        );
    }
    out.push_str(
        "\n`wrong`: a refutation of a problem some tool proved with a checked proof, or \
         a verdict against a generated problem's by construction, never counted as \
         solved; `late`: decided after the limit; `gave up`: the tool's own bound or \
         no answer; `outside`: not run, outside the tool's fragment or mode; `against \
         the header`: a verdict, counted as solved, that contradicts an LLTP file's \
         header, which a few of the library's files have wrong.\n\n",
    );
}

/// Per group of problems and tool, solved of those in the tool's fragment.
fn groups_table(
    out: &mut String,
    tools: &[&str],
    problems: &BTreeMap<String, BTreeMap<&str, &Row>>,
) {
    let mut groups: Vec<&str> = Vec::new();
    for p in problems.values() {
        if let Some(row) = p.values().next()
            && !groups.contains(&group(row))
        {
            groups.push(group(row));
        }
    }
    let _ = writeln!(
        out,
        "## By group: solved / in the fragment\n\n| group | problems | {} |\n|---|---|{}",
        tools.join(" | "),
        "---|".repeat(tools.len())
    );
    for g in groups {
        let of: Vec<&BTreeMap<&str, &Row>> = problems
            .values()
            .filter(|p| p.values().next().is_some_and(|r| group(r) == g))
            .collect();
        let cells: Vec<String> = tools
            .iter()
            .map(|t| {
                let rows: Vec<&Row> = of.iter().filter_map(|p| p.get(t).copied()).collect();
                let covered = rows.iter().filter(|r| !outside(r)).count();
                let n = rows.iter().filter(|r| solved(r)).count();
                if covered == 0 {
                    "–".to_owned()
                } else {
                    format!("{n} / {covered}")
                }
            })
            .collect();
        let _ = writeln!(out, "| {g} | {} | {} |", of.len(), cells.join(" | "));
    }
    out.push('\n');
}

/// linlog on one thread against each other tool on the problems both
/// cover: solved by both, by one alone, by neither, and the problems the
/// other solves and linlog does not.
fn pairs(out: &mut String, tools: &[&str], problems: &BTreeMap<String, BTreeMap<&str, &Row>>) {
    if !tools.contains(&REFERENCE) {
        return;
    }
    let _ = writeln!(
        out,
        "## {REFERENCE} against each tool, on the problems both cover\n\n\
         | tool | both cover | both solve | {REFERENCE} alone | the tool alone | neither |\n\
         |---|---|---|---|---|---|"
    );
    let mut lost: Vec<(&str, Vec<&String>)> = Vec::new();
    for &tool in tools.iter().filter(|&&t| t != REFERENCE) {
        let mut cells = [0usize; 5];
        let mut theirs = Vec::new();
        for (name, p) in problems {
            let (Some(a), Some(b)) = (p.get(REFERENCE), p.get(tool)) else {
                continue;
            };
            if outside(a) || outside(b) {
                continue;
            }
            cells[0] += 1;
            match (solved(a), solved(b)) {
                (true, true) => cells[1] += 1,
                (true, false) => cells[2] += 1,
                (false, true) => {
                    cells[3] += 1;
                    theirs.push(name);
                }
                (false, false) => cells[4] += 1,
            }
        }
        let cells: Vec<String> = cells.iter().map(usize::to_string).collect();
        let _ = writeln!(out, "| {tool} | {} |", cells.join(" | "));
        lost.push((tool, theirs));
    }
    out.push('\n');
    for (tool, theirs) in lost.into_iter().filter(|(_, l)| !l.is_empty()) {
        let _ = writeln!(
            out,
            "Solved by {tool} and not by {REFERENCE} ({}):\n",
            theirs.len()
        );
        for name in theirs.iter().take(LISTED) {
            let _ = writeln!(out, "- {name}");
        }
        if theirs.len() > LISTED {
            let _ = writeln!(out, "- and {} more", theirs.len() - LISTED);
        }
        out.push('\n');
    }
}

/// Every answer that came after the limit and before the run was ended:
/// what a longer limit would have solved, as far as the margin before the
/// kill shows it.
fn late_answers(
    out: &mut String,
    tools: &[&str],
    problems: &BTreeMap<String, BTreeMap<&str, &Row>>,
) {
    let mut rows: Vec<(&String, &str, &Row)> = Vec::new();
    for (name, p) in problems {
        for &tool in tools {
            if let Some(row) = p.get(tool).filter(|r| late(r)) {
                rows.push((name, tool, row));
            }
        }
    }
    if rows.is_empty() {
        out.push_str("## Answered after the limit\n\nNone.\n\n");
        return;
    }
    let _ = writeln!(
        out,
        "## Answered after the limit\n\n| problem | tool | verdict | time |\n|---|---|---|---|"
    );
    for (name, tool, row) in rows {
        let ms: f64 = row.get("time_ms").parse().unwrap_or(0.0);
        let _ = writeln!(
            out,
            "| {name} | {tool} | {} | {} |",
            row.get("verdict"),
            time(ms)
        );
    }
    out.push('\n');
}

/// Per tool the time its translation took and the peak memory of its
/// solved runs.
fn costs(out: &mut String, tools: &[&str], problems: &BTreeMap<String, BTreeMap<&str, &Row>>) {
    let _ = writeln!(
        out,
        "## Translation and memory\n\n\
         | tool | translated | translation, median | translation, most | peak memory of solved runs, median | most |\n\
         |---|---|---|---|---|---|"
    );
    for &tool in tools {
        let rows: Vec<&Row> = problems
            .values()
            .filter_map(|p| p.get(tool).copied())
            .filter(|r| !outside(r))
            .collect();
        let mut translate: Vec<f64> = rows
            .iter()
            .filter_map(|r| r.get("translate_ms").parse().ok())
            .collect();
        let mut peak: Vec<f64> = rows
            .iter()
            .filter(|r| solved(r))
            .filter_map(|r| r.get("peak_bytes").parse().ok())
            .collect();
        let median = |v: &mut Vec<f64>| {
            v.sort_by(f64::total_cmp);
            (v.get(v.len() / 2).copied(), v.last().copied())
        };
        let (t_median, t_most) = median(&mut translate);
        let (p_median, p_most) = median(&mut peak);
        let ms = |v: Option<f64>| v.map_or("–".to_owned(), time);
        let mib =
            |v: Option<f64>| v.map_or("–".to_owned(), |b| format!("{:.1} MiB", b / 1_048_576.0));
        let _ = writeln!(
            out,
            "| {tool} | {} | {} | {} | {} | {} |",
            translate.len(),
            ms(t_median),
            ms(t_most),
            mib(p_median),
            mib(p_most)
        );
    }
    out.push_str(
        "\nlinlog reads LLTP and `.spec` files itself and the rest as JSON, which the \
         harness writes; the translation time is the harness's, outside the measured run.\n\n",
    );
}

/// Every problem on which a tool's decided verdict contradicts another's
/// or the known one, with every tool's verdict.
fn contradictions(
    out: &mut String,
    tools: &[&str],
    problems: &BTreeMap<String, BTreeMap<&str, &Row>>,
) {
    let _ = writeln!(
        out,
        "## Contradictions\n\nA problem on which one tool proves what another refutes, or \
         a tool decides against the known verdict. A proof is checked where the row \
         says so (linlog's command checks every proof it reports, Sympli validates \
         its own); no tool's `unprovable` is checked, linlog's included.\n"
    );
    let mut any = false;
    for (name, p) in problems {
        let verdicts: BTreeSet<&str> = p
            .values()
            .filter(|r| decided(r))
            .map(|r| r.get("verdict"))
            .collect();
        if verdicts.len() < 2 && !p.values().any(|r| wrong(r) || against_header(r)) {
            continue;
        }
        if !any {
            let _ = writeln!(
                out,
                "| problem | expected | {} |\n|---|---|{}",
                tools.join(" | "),
                "---|".repeat(tools.len())
            );
            any = true;
        }
        let expected = p.values().next().map_or("", |r| r.get("expected"));
        let cells: Vec<String> = tools
            .iter()
            .map(|t| match p.get(t) {
                Some(r) if decided(r) => {
                    let checked = if r.get("checked") == "ok" {
                        " (checked)"
                    } else {
                        " (unchecked)"
                    };
                    format!("{}{checked}", r.get("verdict"))
                }
                Some(r) => ending(r).to_owned(),
                None => "–".to_owned(),
            })
            .collect();
        let _ = writeln!(out, "| {name} | {expected} | {} |", cells.join(" | "));
    }
    if !any {
        out.push_str("None.\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::summary::tests::row;

    /// A row of `run --tools`.
    fn run(tool: &str, problem: &str, verdict: &str, time_ms: &str) -> Row {
        row(&[
            ("source", "family"),
            ("family", "f"),
            ("problem", problem),
            ("mode", "classical"),
            ("tool", tool),
            ("verdict", verdict),
            ("time_ms", time_ms),
            ("timeout_s", "10"),
            ("expected", if problem == "p3" { "unprovable" } else { "" }),
            (
                "checked",
                if tool == "linlog" && verdict == "proved" {
                    "ok"
                } else {
                    ""
                },
            ),
        ])
    }

    /// The ranking counts per tool; a refutation of a problem with a checked
    /// proof and an unchecked proof of a generated problem's refutable
    /// sequent are wrong and not solved, a late answer is not solved; the
    /// pair table names what the other tool alone solves; and a planted
    /// disagreement is listed with both verdicts, against the known one as
    /// well.
    #[test]
    fn tools_ranked_and_contradictions_listed() {
        let mut rows = [
            run("linlog", "p1", "proved", "5"),
            run("other", "p1", "unprovable", "7"),
            run("linlog", "p2", "unknown", "10000"),
            run("other", "p2", "proved", "30"),
            run("linlog", "p3", "unprovable", "1"),
            run("other", "p3", "proved", "20000"),
        ];
        judge(&mut rows);
        let text = report(&rows);
        assert!(text.contains("| linlog | 2 | 1 | 1 | 0 | 0 |"), "{text}");
        assert!(text.contains("| other | 1 | 1 | 0 | 2 | 0 |"), "{text}");
        assert!(text.contains("| other | 3 | 0 | 2 | 1 | 0 |"), "{text}");
        assert!(
            text.contains("Solved by other and not by linlog (1):\n\n- f/p2 (classical)"),
            "{text}"
        );
        assert!(
            text.contains("| f/p1 (classical) |  | proved (checked) | unprovable (unchecked) |"),
            "{text}"
        );
        assert!(
            text.contains(
                "| f/p3 (classical) | unprovable | unprovable (unchecked) | proved (unchecked) |"
            ),
            "{text}"
        );
        assert!(!text.contains("| f/p2"), "{text}");
        assert!(
            text.contains("| f/p3 (classical) | other | proved | 20.00 s |"),
            "{text}"
        );
    }
}
