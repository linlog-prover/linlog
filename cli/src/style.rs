// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The options of every output format in one value, read from the JSON of
//! `--style-file` and changed by `--style KEY=VALUE`, `--lemma`,
//! `--prelude`, `--kernel`, `--font` and `--standalone`.

use crate::argument_parsing::{RocqArgs, StyleArgs};
use crate::io;
use anyhow::{Context, Result, bail};
use linlog::export::svg::MathFont;
use linlog::export::{Form, Styles};
use serde_json::Value;

/// The names of the formats that have options, as `--style` keys and the
/// style file name them.
const FORMATS: [&str; 7] = ["text", "latex", "typst", "svg", "png", "pdf", "rocq"];

/// Reads the style file over `base` and applies the flags to it, in their
/// order; an unprefixed key names `format`, the format of the command's
/// output, when it has options. `standalone` makes every document format standalone.
///
/// # Errors
///
/// A style file that cannot be read or holds no styles, a key that names
/// no option, or a value the option cannot take.
pub(crate) fn read(
    base: &Styles,
    args: &StyleArgs,
    rocq: Option<&RocqArgs>,
    format: Option<&str>,
    standalone: bool,
) -> Result<Styles> {
    let mut value = serde_json::to_value(base)?;
    if let Some(path) = &args.style_file {
        let text = io::read_configuration(path, "style file")?;
        let file = serde_json::from_str(&text)
            .with_context(|| format!("{} is not JSON", path.display()))?;
        crate::settings::over(&mut value, file);
    }
    for setting in &args.style {
        let Some((key, text)) = setting.split_once('=') else {
            bail!("--style {setting}: give KEY=VALUE");
        };
        let mut path: Vec<&str> = key.split('.').collect();
        // A format's name is a prefix only before a dot: `text` alone
        // is the colour field of SVG's style.
        if path.len() == 1 || !FORMATS.contains(&path[0]) {
            match format {
                Some(format) => path.insert(0, format),
                None => bail!(
                    "--style {setting}: name the format the key is of, as in \
                     latex.{key} (text, latex, typst, svg, png, pdf or rocq)"
                ),
            }
        }
        // A command without the Rocq format takes none of its options.
        if path[0] == "rocq" && rocq.is_none() {
            bail!("--style {setting}: this command writes no Rocq certificate");
        }
        // A value that is no JSON is a string, so that `bar=|` and
        // `family=monospace` need no quotes.
        let parsed = serde_json::from_str(text).unwrap_or_else(|_| Value::String(text.to_owned()));
        set(&mut value, &path, parsed).with_context(|| format!("--style {setting}"))?;
    }
    let mut styles: Styles =
        serde_json::from_value(value).context("the style is not one linlog knows")?;
    if let Some(lemma) = rocq.and_then(|r| r.lemma.as_ref()) {
        styles.rocq.lemma = lemma.parse()?;
    }
    if let Some(prelude) = rocq.and_then(|r| r.prelude.as_ref()) {
        styles.rocq.prelude = Some(prelude.clone());
    }
    if let Some(kernel) = rocq.and_then(|r| r.kernel) {
        styles.rocq.kernel = Some(kernel.into());
    }
    if let Some(font) = args.font {
        styles.svg.font = MathFont::from(font).font();
    }
    if standalone {
        styles.latex.form = Form::Standalone;
        styles.typst.form = Form::Standalone;
        styles.rocq.form = Form::Standalone;
    }
    // A fragment has no prelude, so the flag would do nothing there.
    if rocq.is_some_and(|r| r.prelude.is_some()) && !matches!(styles.rocq.form, Form::Standalone) {
        bail!(
            "--prelude goes into a standalone certificate: add --standalone, or --style \
             rocq.form=standalone"
        );
    }
    Ok(styles)
}

/// Sets the field at `path` of a JSON object to `new`, making the objects
/// on the way.
fn set(value: &mut Value, path: &[&str], new: Value) -> Result<()> {
    let mut at = value;
    for (i, key) in path.iter().enumerate() {
        let Value::Object(object) = at else {
            bail!("{} is not an object", path[..i].join("."));
        };
        at = object
            .entry(key.to_string())
            .or_insert_with(|| Value::Object(serde_json::Map::new()));
    }
    *at = new;
    Ok(())
}
