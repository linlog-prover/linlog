// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The comparison of two runs' CSV files as `linlog-bench summary`
//! prints it, end to end on a pair of small files.

use std::path::Path;
use std::process::Command;

/// The columns the comparison reads; the others are empty.
const HEADER: &str = "family,problem,mode,engine_requested,jobs,test_period,run,verdict,\
                      expected,reason,time_ms,timeout_s,checked,nodes,memo_hits,memo_entries,splits";

/// The earlier run: `f/a` proved as expected, `f/b` refuted, `f/c` at its
/// time limit.
const BEFORE: &str = "f,f/a,classical,auto,1,,0,proved,proved,,2.0,5,ok,10,1,4,0
f,f/b,classical,auto,1,,0,unprovable,,,4.0,5,,5,0,2,1
f,f/c,classical,auto,1,,0,unknown,,timeout,5000.0,5,,99,0,9,0
";

/// The later run: `f/a` refuted against its expected verdict, `f/b`
/// refuted in one stable sequent more, `f/c` proved, each in half the
/// time.
const AFTER: &str = "f,f/a,classical,auto,1,,0,unprovable,proved,,1.0,5,,10,1,4,0
f,f/b,classical,auto,1,,0,unprovable,,,2.0,5,,6,0,2,1
f,f/c,classical,auto,1,,0,proved,,,3.0,5,ok,7,0,3,0
";

/// Writes a CSV file of the rows given under the header.
fn write(path: &Path, rows: &str) {
    std::fs::write(path, format!("{HEADER}\n{rows}")).unwrap();
}

/// Runs `linlog-bench summary` with the arguments, and returns its
/// standard output once it succeeded.
fn summary(args: &[&Path]) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_linlog-bench"))
        .arg("summary")
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

/// `summary --against` with `--counters` prints the verdict that differs,
/// the counter that differs on a problem decided alike, what each file
/// decides and the other does not, the groups and the problems, every
/// table of it; `summary --before` prints the same comparison of each
/// file with the file of its name in the earlier directory, without the
/// counters' rows and the reverse list.
#[test]
fn the_comparison_prints_what_differs() {
    let dir = std::env::temp_dir().join(format!("linlog-compare-{}", std::process::id()));
    let earlier = dir.join("old");
    std::fs::create_dir_all(&earlier).unwrap();
    let (before, after) = (dir.join("before.csv"), dir.join("after.csv"));
    write(&before, BEFORE);
    write(&after, AFTER);
    write(&earlier.join("runs.csv"), BEFORE);
    write(&dir.join("runs.csv"), AFTER);
    let against = summary(&[
        Path::new("--against"),
        &before,
        &after,
        Path::new("--counters"),
    ]);
    let since = summary(&[Path::new("--before"), &earlier, &dir.join("runs.csv")]);
    std::fs::remove_dir_all(&dir).unwrap();
    assert_eq!(against, AGAINST);
    assert_eq!(since, SINCE);
}

/// What `summary --against before.csv after.csv --counters` prints.
const AGAINST: &str = r#"## after against before

### Verdicts that differ

| file | family | configuration | problem | after | before |
|---|---|---|---|--:|--:|
|  | f | classical auto j1 | f/a | 1.0 ms ✗ | 2.0 ms ✓ |

### Counters that differ

Of the 1 problems both decide alike on one thread, 1 differ in nodes, splits, memo_hits, memo_entries, links, tests.

| file | family | configuration | problem | counter | after | before |
|---|---|---|---|---|--:|--:|
|  | f | classical auto j1 | f/b | nodes | 6 | 5 |

### Decided by after, not by before

| file | family | configuration | problem | after | before |
|---|---|---|---|--:|--:|
|  | f | classical auto j1 | f/c | 3.0 ms ✓ | > 5 s |

### Decided by before, not by after

None.

### By group

Problems in both; decided (late: after the time limit); decided by one and not the other; where the undecided ended (`time` the time limit or `killed` past it, `bound` the copy bound, `depth` the recursion limit, `memory` the memory limit, `index` an index past its width, `wide` a context too wide to split, `crash` out of memory); the median ratio of before's time to after's on the problems both decide within the limit (how many); and the problems only before has.

| file | family | configuration | problems | after decided | before decided | only after | only before | after ended | before ended | before/after | new |
|---|---|---|--:|--:|--:|--:|--:|---|---|--:|--:|
|  | f | classical auto j1 | 3 | 3 | 2 | 1 | 0 | – | 1 time | 2.00× (2) | 0 |

### Generated and listed problems

| file | family | configuration | problem | after | before | before/after |
|---|---|---|---|--:|--:|--:|
|  | f | classical auto j1 | f/a | 1.0 ms ✗ | 2.0 ms ✓ | 2.00× |
|  | f | classical auto j1 | f/b | 2.0 ms ✗ | 4.0 ms ✗ | 2.00× |
|  | f | classical auto j1 | f/c | 3.0 ms ✓ | > 5 s |  |

"#;

/// What `summary --before old runs.csv` prints.
const SINCE: &str = r#"### Verdicts that differ

| file | family | configuration | problem | before | after |
|---|---|---|---|--:|--:|
| runs | f | classical auto j1 | f/a | 2.0 ms ✓ | 1.0 ms ✗ |

### Counters that differ

Of the 1 problems both decide alike on one thread, 1 differ in nodes, splits, memo_hits, memo_entries, links, tests.

### Decided by before, not by after

None.

### By group

Problems in both; decided (late: after the time limit); decided by one and not the other; where the undecided ended (`time` the time limit or `killed` past it, `bound` the copy bound, `depth` the recursion limit, `memory` the memory limit, `index` an index past its width, `wide` a context too wide to split, `crash` out of memory); the median ratio of after's time to before's on the problems both decide within the limit (how many); and the problems only after has.

| file | family | configuration | problems | before decided | after decided | only before | only after | before ended | after ended | after/before | new |
|---|---|---|--:|--:|--:|--:|--:|---|---|--:|--:|
| runs | f | classical auto j1 | 3 | 2 | 3 | 0 | 1 | 1 time | – | 0.50× (2) | 0 |

### Generated and listed problems

| file | family | configuration | problem | before | after | after/before |
|---|---|---|---|--:|--:|--:|
| runs | f | classical auto j1 | f/a | 2.0 ms ✓ | 1.0 ms ✗ | 0.50× |
| runs | f | classical auto j1 | f/b | 4.0 ms ✗ | 2.0 ms ✗ | 0.50× |
| runs | f | classical auto j1 | f/c | > 5 s | 3.0 ms ✓ |  |

"#;
