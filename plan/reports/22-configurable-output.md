# Step 22 report: configurable output, and no font in LaTeX and Typst

Status: first session, in progress. This report is written as the work
goes; the sections marked *open* are what the session still owes.

## Outcome so far

- **No font in LaTeX and Typst.** The standalone LaTeX preamble no longer
  loads `eulervm`, the Typst page no longer sets `Euler Math`;
  `modules/export.nix` lost `eulervm` from its TeX Live and Typst's
  `--font-path` (Typst's embedded fonts serve; resvg keeps Euler Math for
  the SVG). The snapshots changed by exactly those two lines.
- **One options value per output**, each with `Default`, `Clone`,
  `PartialEq`, `Eq`, `Hash` and serde behind `serialize` (with
  `serde(default, deny_unknown_fields)`):
  `TextOptions` (the text tree), `latex::Options`, `typst::Options`,
  `svg::Style` (extended), `rocq::Options` (extended). `Form` became a
  field of the three that have one, and has serde.
- **One signature writes a derivation** in every output:
  `latex::write`, `typst::write`, `svg::write`, `rocq::write` and
  `Derivation::write_text`, each `(derivation, options, out: impl
  fmt::Write, stop: impl FnMut() -> bool) -> Result<(), WriteError>`.
  The emitters hold one inference's text; the SVG tree no longer keeps a
  laid-out run per inference (the 50 bytes per character of sequent of
  step 18's finding), only a few numbers, and lays each conclusion out
  again when it writes it.
- **The command writes the verdict first, then the derivation as it is
  made**, into standard output or a file (`cli/src/io.rs`, `Output`: a
  buffered stream, a file of another name renamed once whole, an
  unfinished output dropped with its buffer so that an error found
  before the derivation leaves standard output empty, as before). No
  second copy of the derivation is held. A derivation cut short by the
  time limit or Ctrl-C stays written as far as it came, followed by a
  line in the format's comment syntax that says so.
- **Rule labels are one table per convention** (`proofs/style.rs`),
  in a markup each output sets its way; the round trip
  `Rule::from_str(rule.name())` and "the upright table as plain text is
  `Rule::name`" are a test over `Rule::ALL`.
- **The command's surface**: `--style KEY=VALUE` (repeatable),
  `--style-file PATH`, `--lemma`, `--prelude`, `--no-verdict`,
  `--screens N|none`, `--abbreviate CHARS|none` on `prove` and `check`;
  `--style`, `--style-file`, `--lemma`, `--prelude` on `seq print` and
  `interact`. The session's `show` takes `--text`, `--latex`, `--typst`,
  `--svg` (the bare names still work), and `proof [--FORMAT] [FILE]`
  takes `--text`, `--json`, `--latex`, `--typst`, `--svg` and `--rocq`,
  so a finished session is certified with `proof --rocq` and `proof
  latex` no longer writes a file named `latex`.
- **Bounded reports**: `check`'s verdict line abbreviates the sequent
  (`--abbreviate`, 200 characters by default, `… (N formulas)` after
  the cut), with `--quiet` too; `CheckError::describe(…).abbreviated(n)`
  cuts every formula and every formula list of an error report, which
  the command uses for `check` and for an invalid proof met while
  drawing.

Verified in this session: `cargo test --workspace` (all pass; the
snapshots are byte-identical under the defaults after the font change),
`cargo clippy --workspace --all-targets -- --deny warnings`, `cargo hack
check --each-feature -p linlog`. Not yet run: the powerset check, the
`export` check, `nix flake check`.

## Every option, its default, and how each front end sets it

The command maps flags onto `Styles` (`cli/src/style.rs`): one object
with a key per format, `{"text": …, "latex": …, "typst": …, "svg": …,
"rocq": …}`, read from `--style-file`, then changed by every `--style
KEY=VALUE` in order (a key without a format prefix names `--format`'s
format; in `interact`, which writes several formats, the prefix is
required), then by `--lemma`, `--prelude` and `--standalone`. A value
that is not JSON is a string. The web front end holds the same JSON in
its settings and sends the section of the format it asks for; an editor
plugin or a notebook passes the library's options values, or the same
JSON through serde.

| output | field | default | command |
|---|---|---|---|
| text | `labels` | `upright` | `--style labels=subscript` |
| text | `open` | `bare` | `--style text.open=dots` (session) |
| text | `bar` | `─` (`TextOptions::BAR`) | `--style bar==` |
| text | `gap` | 3 (`TextOptions::GAP`) | `--style gap=5` |
| latex | `form` | `fragment` | `--standalone`, `--style form=standalone` |
| latex | `labels` | `upright` | `--style labels=off` |
| latex | `open` | `dots` | `--style latex.open=dashed` |
| latex | `align` | `true` | `--style align=false` |
| latex | `ebproof` | empty | `--style ebproof=center=false` |
| latex | `preamble` | `null` (the `standalone` class with amssymb, cmll, ebproof) | `--style 'preamble=\documentclass{article}…'` |
| typst | `form`, `labels`, `open` | `fragment`, `upright`, `dots` | as for LaTeX |
| typst | `import` | `#import "@preview/curryst:0.6.0": prooftree, rule` | `--style 'import=…'` |
| typst | `page` | `typst::PAGE` | `--style 'page=#set page(margin: 1cm)'` |
| svg | `font` | Euler Math (`Font::euler()`), its committed advances | `--style font.family=monospace --style 'font.advances={"fixed":600}'` |
| svg | `labels`, `open` | `upright`, `dots` | as above |
| svg | `ids` | `false` | `--style ids=true` |
| svg | `font_size` … `background` | as before (`Style::default()`) | `--style background=white` |
| rocq | `form` | `fragment` | `--standalone` |
| rocq | `lemma` | `certificate` | `--lemma NAME`, `--style lemma=…` |
| rocq | `prelude` | `From NanoYalla Require Import macroll.` | `--prelude TEXT` |
| command | screens a tree may fill | 3 (`SCREENS`) | `--screens N\|none` |
| command | sequent in a verdict line | 200 characters (`ABBREVIATE`) | `--abbreviate N\|none` |
| command | verdict line | written | `--no-verdict` |

Labels are `upright`, `subscript`, `off` or `{"table": {"⊸L": "⊸_L",
…}}` (a rule the table leaves out keeps its upright label); an open goal
is `dots`, `bare`, `dashed` or `{"mark": "?"}`.

## Decisions

- **The label markup** is text with `_x`/`_{xy}` subscripts and the
  connective characters as symbols, rather than a structure per label:
  a user writes a table entry as a string in JSON or on the command
  line, and each target already had one way to set a symbol, upright
  text and a subscript. The upright table reproduces today's labels
  byte for byte in every target.
- **`--tree` speaks for every format** (it always did in the code; the
  help said "the text tree"): `auto` differs from `always` only for the
  text tree on a terminal. The help now says so.
- **The screens, the abbreviation and the verdict line are the
  command's own flags**, not fields of a library options value: whether
  a tree fits a terminal and what goes into a verdict line are the
  command's questions (step 18's reading of D15).
- **The interactive session's messages are not localised.** `Refusal`
  is an enum with its data, and the JSON carries the values of a
  verdict (`refutation`, `reason`, `statistics.copies`), so a front end
  that writes its own words needs nothing more from the library. Step
  21's sentences (`Refutation`'s `Display`, the command's `unknown`
  and `notice_line`) stay English.
- **An output dropped before it is finished writes nothing**: the Rocq
  refusal of a proof with Mix is found after the verdict line is
  written, and the test pins an empty standard output for it, as
  before streaming.
- **serde for the CLI**: the CLI now depends on `serde` (derive) for
  `Styles`, the version the core already locks.

## Requests of the author during the step

- 2026-10-03, during the first session: "add PDF as well as PNG/JPG
  export". The step's prompt says "no new export target"; the author's
  request overrides it. A sub-agent is researching the crates (resvg,
  svg2pdf, an encoder for JPEG), their licences against `deny.toml`,
  wasm, and the font (*open*: the decisions it needs are the author's).

## Open in this session

- Item 5: the map from a drawing's inferences to the session's goals
  (`Interactive`), and a test of `Style::ids`.
- Item 6: snapshots covering every rule label, the fragments compiled
  by the `export` check, and one dashed open goal compiled in Typst.
- The CLI test that sets every option once from the command and once
  from JSON; README's usage section; `.claude/rules/cli.md`.
- `ViewError`'s `Display` writing a saturated count as `u64::MAX`.
- The powerset check, the `export` check, `nix flake check`.
- Left as found, for later steps: `--stats` says "copy bound reached"
  without whose level it is (the `Statistics` has one number; step 26
  owns the schedulers); `rocq::Options::lemma` is not checked to be an
  identifier (step 28, the API's boundaries).
