// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! What the command's test files share: one way to run a command and one
//! way to write what of its output is the machine's.

use std::io::Write;
use std::process::{Command, Output, Stdio};

/// Runs the command with `stdin` as standard input and its standard
/// output and error piped, and returns its output once it exited.
pub fn output(command: &mut Command, stdin: &str) -> Output {
    let mut child = command
        // A PDF's date, so that every run writes the same bytes.
        .env("SOURCE_DATE_EPOCH", "1791158399")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    // A call that does not read its input may exit before the write: a
    // closed pipe is then no failure of the test.
    if let Err(error) = child.stdin.take().unwrap().write_all(stdin.as_bytes()) {
        assert_eq!(error.kind(), std::io::ErrorKind::BrokenPipe, "{error}");
    }
    child.wait_with_output().unwrap()
}

/// Returns the text with every time a search took, a number with its unit
/// after "after " or "time: ", and the crate's version an outcome names,
/// as `…`.
pub fn timeless(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = ["after ", "time: "]
        .iter()
        .filter_map(|prefix| rest.find(prefix).map(|found| found + prefix.len()))
        .min()
    {
        out.push_str(&rest[..at]);
        rest = &rest[at..];
        let number = rest
            .find(|c: char| !(c.is_ascii_digit() || c == '.'))
            .unwrap_or(rest.len());
        if number > 0 {
            let unit = rest[number..]
                .find(|c: char| !"nµms".contains(c))
                .map_or(rest.len(), |end| number + end);
            out.push('…');
            rest = &rest[unit..];
        }
    }
    out.push_str(rest);
    unversioned(&out)
}

/// Returns the text with the crate's version that an outcome names, in
/// `"linlog":"0.1.0"`, as `…`: a release moves it.
fn unversioned(text: &str) -> String {
    const KEY: &str = "\"linlog\":\"";
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find(KEY) {
        let (head, tail) = rest.split_at(at + KEY.len());
        out.push_str(head);
        out.push('…');
        rest = &tail[tail.find('"').unwrap_or(tail.len())..];
    }
    out.push_str(rest);
    out
}
