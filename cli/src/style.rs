// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The options of every output format in one value, read from the JSON of
//! `--style-file` and changed by `--style KEY=VALUE`, `--lemma`,
//! `--prelude` and `--standalone`.

use crate::argument_parsing::StyleArgs;
use crate::io;
use anyhow::{Context, Result, bail};
use linlog::TextOptions;
use linlog::export::svg::Style;
use linlog::export::{Form, latex, pdf, png, rocq, typst};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The names of the formats that have options, as `--style` keys and the
/// style file name them.
const FORMATS: [&str; 7] = ["text", "latex", "typst", "svg", "png", "pdf", "rocq"];

/// The options of every output format.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Styles {
    /// The text tree's.
    pub text: TextOptions,
    /// LaTeX's.
    pub latex: latex::Options,
    /// Typst's.
    pub typst: typst::Options,
    /// SVG's, for derivations, sequents and nets, and for the drawings
    /// that PNG and PDF render.
    pub svg: Style,
    /// PNG's, beyond the SVG's.
    pub png: png::Options,
    /// PDF's, beyond the SVG's.
    pub pdf: pdf::Options,
    /// The Rocq certificate's.
    pub rocq: rocq::Options,
}

impl Styles {
    /// Reads the style file and applies the flags to it, in their order;
    /// an unprefixed key names `format`, the format of the command's
    /// output, when it has options. `standalone` makes every document
    /// format standalone.
    pub fn read(args: &StyleArgs, format: Option<&str>, standalone: bool) -> Result<Self> {
        let mut value = match &args.style_file {
            Some(path) => {
                let text = io::read(Some(path), "style file")?;
                serde_json::from_str(&text)
                    .with_context(|| format!("{} is not JSON", path.display()))?
            }
            None => Value::Object(serde_json::Map::new()),
        };
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
                         latex.{key} (text, latex, typst, svg or rocq)"
                    ),
                }
            }
            // A value that is no JSON is a string, so that `bar=|` and
            // `family=monospace` need no quotes.
            let parsed =
                serde_json::from_str(text).unwrap_or_else(|_| Value::String(text.to_owned()));
            set(&mut value, &path, parsed).with_context(|| format!("--style {setting}"))?;
        }
        let mut styles: Self =
            serde_json::from_value(value).context("the style is not one linlog knows")?;
        if let Some(lemma) = &args.lemma {
            styles.rocq.lemma = lemma.clone();
        }
        if let Some(prelude) = &args.prelude {
            styles.rocq.prelude = prelude.clone();
        }
        if standalone {
            styles.latex.form = Form::Standalone;
            styles.typst.form = Form::Standalone;
            styles.rocq.form = Form::Standalone;
        }
        Ok(styles)
    }
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
