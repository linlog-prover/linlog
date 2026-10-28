// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The settings of a browser tab: the command's defaults where they fit a
//! tab, and smaller bounds where they do not, and how a page's settings
//! are read over them.

use crate::failure::Failure;
use linlog::export::Styles;
use linlog::search::{Jobs, Options};
use linlog::{Clock, Limits, Settings};
use serde_json::Value;

/// The stack the module is linked with (`build.rs`), in bytes.
pub const STACK_BYTES: usize = parse(env!("LINLOG_WEB_STACK_BYTES"));

/// The memory bound of a call: 96 MiB. A tab on iOS gets about 256 MiB of
/// WebAssembly memory in all; what the bound does not count (the forest,
/// the stack, the documents in and out, the drawing) has the rest.
pub const MEMORY_BYTES: u64 = 96 << 20;

/// The bound on a sequent's occurrences: a million, whose forest takes
/// about 25 MB, and a session, a proof and a net each hold a copy of it.
pub const OCCURRENCES: u64 = 1_000_000;

/// The bound on a derivation's estimated size: 4 MiB, a sixteenth of the
/// command's. A drawing's peak is several times its estimate, and a tree
/// of that size is far wider than a screen.
pub const DERIVATION_BYTES: u64 = 4 << 20;

/// The work a call may do: the page's budget, since the module reads no
/// clock and a search ends at its work bound or not at all. About a
/// second of search: the backward search did 68 million units a second
/// in Node on a laptop's slower cores, the forward one fewer or more.
pub const WORK: u64 = 1 << 26;

/// The work the classical refuter may do after a search that did not
/// prove its goal, in its own terms: a sixteenth of the library's default.
pub const CLASSICAL_WORK: u64 = Options::DEFAULT_CLASSICAL_WORK >> 4;

/// The deepest recursion of a search: 384 levels. The engine's frames are
/// on the module's stack ([`STACK_BYTES`]) and on the host's own, whose
/// size a page cannot set (Node's and a browser's are about 1 MiB), and
/// the host's runs out first: on the path whose levels are the largest, a
/// Mix of pairs, the optimized module recursed to 768 levels and not to
/// 1 024 in Node, so the limit is half the deepest that held.
pub const RECURSION_DEPTH: u32 = 384;

/// The bound on the bytes the drawings of an elimination's steps take
/// together: 16 MiB, four drawings at the tab's bound on one
/// ([`DERIVATION_BYTES`]), as the command's is four at its own. The page
/// holds every step's drawing, and the command's 256 MiB is past a tab's
/// whole memory bound ([`MEMORY_BYTES`]).
pub const DRAWN_BYTES: u64 = 4 * DERIVATION_BYTES;

/// Returns the tab's settings: the command's but for the bounds above, one
/// thread, no copy bound (the work bound ends a search), no clock, and
/// drawings with an id on every formula and a description for a screen
/// reader.
pub fn defaults() -> Settings {
    let limits = Limits::default()
        .with_memory_bytes(Some(MEMORY_BYTES))
        .with_occurrences(Some(OCCURRENCES))
        .with_derivation_bytes(Some(DERIVATION_BYTES))
        .with_work(Some(WORK))
        .with_recursion_depth(RECURSION_DEPTH);
    let search = Options::default()
        .with_copies(None)
        .with_jobs(Jobs::Count(1))
        .with_classical_work(CLASSICAL_WORK);
    let clock = Clock::default()
        .with_time_limit_ms(None)
        .with_pool_after_ms(None);
    let mut styles = Styles::default();
    styles.svg = styles.svg.with_ids(true).with_description(true);
    let cut = linlog::proofs::cut::Options::default().with_drawn_bytes(Some(DRAWN_BYTES));
    Settings::default()
        .with_cut(cut)
        .with_clock(clock)
        .with_limits(limits)
        .with_search(search)
        .with_styles(styles)
}

/// Reads a page's settings over the tab's: every key the page gives
/// replaces the tab's value there, an object key by key, and every other
/// keeps the tab's. An empty text is the tab's settings.
///
/// # Errors
///
/// A text that is no JSON, and a key or value the settings refuse.
pub fn read(json: &str) -> Result<Settings, Failure> {
    if json.trim().is_empty() {
        return Ok(defaults());
    }
    let given: Value = serde_json::from_str(json).map_err(Failure::json)?;
    let mut merged = serde_json::to_value(defaults()).map_err(Failure::json)?;
    overlay(&mut merged, given, "the settings")?;
    serde_json::from_value(merged).map_err(Failure::json)
}

/// Writes `given` over `base`: objects key by key, every other value
/// whole. Where `base` is an object, `given` must be one or a string (a
/// name the option reads, as a math font's for `styles.svg.font`): the
/// settings' reader would take an array for the fields in order and the
/// command's defaults for the rest.
fn overlay(base: &mut Value, given: Value, at: &str) -> Result<(), Failure> {
    match (base, given) {
        (Value::Object(base), Value::Object(given)) => {
            for (key, value) in given {
                match base.get_mut(&key) {
                    Some(slot) => overlay(slot, value, &format!("`{key}`"))?,
                    None => {
                        base.insert(key, value);
                    }
                }
            }
            Ok(())
        }
        (Value::Object(_), given) if !given.is_string() => Err(Failure::refused(
            "json",
            "malformed",
            &format!("{at} must be a JSON object"),
        )),
        (base, given) => {
            *base = given;
            Ok(())
        }
    }
}

/// Reads a decimal number at compile time.
const fn parse(text: &str) -> usize {
    let bytes = text.as_bytes();
    let mut value = 0;
    let mut i = 0;
    while i < bytes.len() {
        value = value * 10 + (bytes[i] - b'0') as usize;
        i += 1;
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;
    use linlog::export::svg::MathFont;

    /// A page's key replaces the tab's value and leaves its siblings, and
    /// a misspelt key, or a value that is no object where one belongs, is
    /// refused.
    #[test]
    fn a_page_s_key_is_read_over_the_tab_s() {
        let settings = read(r#"{"limits": {"work": 5}}"#).unwrap();
        assert_eq!(settings.limits.work, Some(5));
        assert_eq!(settings.limits.memory_bytes, Some(MEMORY_BYTES));
        assert_eq!(read("").unwrap(), defaults());
        assert!(read(r#"{"limits": {"budget": 5}}"#).is_err());
        // An array is no object, though serde would read one as a struct.
        for array in ["[]", r#"{"limits": []}"#, r#"{"limits": [null]}"#] {
            assert!(read(array).is_err(), "{array}");
        }
    }

    /// A math font is chosen by its name over the tab's font, and a name
    /// no option reads is refused.
    #[test]
    fn a_font_is_chosen_by_its_name() {
        let settings = read(r#"{"styles": {"svg": {"font": "stix-two"}}}"#).unwrap();
        assert_eq!(settings.styles.svg.font, MathFont::StixTwo.font());
        assert!(settings.styles.svg.ids);
        assert!(read(r#"{"styles": {"svg": {"font": "stix"}}}"#).is_err());
        assert!(read(r#"{"limits": "none"}"#).is_err());
    }
}
