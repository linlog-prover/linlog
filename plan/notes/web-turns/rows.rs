// Writes the target set's rows with exponentials in a linear mode as JSON
// lines {name, mode, engine, copies, sequent}, from its CSV.
use std::io::Write;
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (csv, lltp) = (&args[1], &args[2]);
    let text = std::fs::read_to_string(csv).unwrap();
    let mut lines = text.lines();
    let head: Vec<&str> = lines.next().unwrap().split(',').collect();
    let col = |name: &str| head.iter().position(|h| *h == name).unwrap();
    let mut seen = std::collections::HashSet::new();
    let mut out = std::io::stdout().lock();
    for line in lines {
        let f: Vec<&str> = line.split(',').collect();
        let (source, problem, mode, fragment) = (
            f[col("source")],
            f[col("problem")],
            f[col("mode")],
            f[col("fragment")],
        );
        if !["IMELL", "MELL", "LL", "ILL"].contains(&fragment)
            || mode.contains("affine")
            || !seen.insert((problem.to_string(), mode.to_string()))
        {
            continue;
        }
        let sequent = match source {
            "family" => {
                let (name, rest) = problem.split_once('/').unwrap();
                let (size, index) = rest.split_once('#').map_or((rest, "0"), |(s, i)| (s, i));
                let family = linlog::families::find(name).unwrap();
                family
                    .instance(size.parse().unwrap(), index.parse().unwrap())
                    .unwrap()
                    .sequent
            }
            "lltp" => {
                let found = std::process::Command::new("find")
                    .args([lltp.as_str(), "-name", problem])
                    .output()
                    .unwrap();
                let path = String::from_utf8(found.stdout).unwrap();
                let path = path.lines().next().unwrap().to_string();
                let text = std::fs::read_to_string(&path).unwrap();
                linlog::lltp::read(&text, &linlog::Limits::default())
                    .unwrap()
                    .sequent
            }
            "file" => {
                let file = std::fs::read_to_string(
                    "/home/tux/Projects/own/linlog-step32a/bench/problems/slow-tests.txt",
                )
                .unwrap();
                let line = file
                    .lines()
                    .find(|l| l.split(';').next().map(str::trim) == Some(problem))
                    .unwrap();
                line.splitn(5, ';').nth(4).unwrap().trim().parse().unwrap()
            }
            other => {
                eprintln!("skip {other} {problem}");
                continue;
            }
        };
        let engine = if mode == "intuitionistic" {
            "two-sided"
        } else {
            "focus"
        };
        let copies: u32 = f[col("copies")].parse().unwrap();
        let row = serde_json::json!({"name": problem, "mode": mode, "engine": engine, "copies": copies,
            "oracle": f[col("verdict")], "oracle_engine": f[col("engine")], "sequent": sequent});
        writeln!(out, "{row}").unwrap();
    }
}
