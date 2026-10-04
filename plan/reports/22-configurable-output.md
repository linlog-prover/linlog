# Step 22 report: configurable output, and no font in LaTeX and Typst

Status: done. The first session did items 1 to 6, with PNG and PDF
export and `--net` added at the author's request; the second session
(2026-10-04) did items 7 to 9.

## Outcome of the second session

- **A Typst tree of any height** (item 7). `typst::Options::layout`
  (`typst::Layout`: `auto`, `curryst`, `linlog`) chooses between curryst
  and a layout of linlog's own, which the output carries as Typst code:
  a `#context` block that lists the inferences from the root in preorder,
  `(premises, label, conclusion)` one per line, and then a fixed script
  (`typst::LAYOUT`) that measures every sequent and label, lays the tree
  out as the SVG drawing does (premises side by side, their conclusions
  centred over the conclusion, the line spanning both, the label after
  it), and places every piece in one box. Nothing is nested per level,
  so no height reaches Typst's depth limits, and no package is needed.
  The default, `auto`, keeps curryst up to `typst::CURRYST_HEIGHT`, nine
  inferences on the longest branch, and uses the own layout above, so
  every snapshot of before is unchanged. Its spacing is four new fields,
  Typst lengths with curryst's values as defaults. The snapshot
  `high.typ`, 41 inferences high, compiles with Typst 0.15.1 (this
  session, by hand, and in the `export` check), and so do
  `open-linlog.typ` (an open goal under dots in the own layout) and a
  41-high scratch tree with a Mix and an open goal.
- **A compact view** (item 8). `ViewOptions::compact` (`linlog::Compact`:
  `auto`, `always`, `never`) draws a run of one structural rule (`?w`,
  `?c`, `wk`, `!w`, `!c`) as one inference, `Inference::times` saying how
  many it stands for and its label getting a star: `?w*` in text and
  SVG, `\wn\mathrm{w}^{*}` in LaTeX, `? upright(w)^*` in Typst. The
  default, `auto`, compacts only where the whole derivation is over a
  bound, so every derivation that was shown before is shown as before;
  the command's `--compact auto|always|never` sets it, and on a terminal
  `auto` also tries the compact tree where the whole one does not fit.
  The builder merges a run in place and counts what it holds, giving up
  at the bound; the size pass returns a lower bound of the compact view
  (the inferences that are no weakening or contraction), so that an
  attempt compaction cannot rescue is refused before it starts. A Rocq
  certificate refuses a compact derivation (`Unsupported::Compact`) and
  the command never builds one for it; a session's grafts are never
  compact. Snapshots `compact.{tex,typ,svg}` pin the starred label in
  every drawn target.
- **What it does on the Petri net that motivated it**
  (`TokenRing-40-unfolded_1_1`, 65 643 clauses, run once on one core in
  a 2 GiB scope): the compact derivation has 128 inferences instead of
  65 767, the 65 640 `!w` steps drawn as one. It is still estimated at
  1.3 GB rather than 998 GiB, over the default bound of 64 MiB, so the
  command leaves it out as before (in 1.1 s, 65 MB, refused by the lower
  bound without a build): the 40 or so `⊗L` steps that decompose the
  initial marking sit *below* the run and each carries all 65 643
  formulas. Showing this net needs the translation to weaken unused `?`
  formulas at the root (below), which is not a compact view and not
  done here.
- **The smaller follow-ups** (item 9), done by a sub-agent in a jj
  workspace of its own and reviewed here: the LaTeX export writes Greek
  letters as their commands (a one-letter name bare, `\alpha`; inside
  `\mathit` in braces) and the `‿` and `·` of `lltp::read`'s names as
  `{\smallsmile}` and `{\cdotp}`, so that pdfLaTeX compiles them
  (snapshot `names.tex`); a proof net's edges and links meet a negated
  literal at its atom rather than at the middle of `A⊥`; wide axiom
  links grow with the square root of their half-width beyond
  `svg::Style::link_cap` (8 em by default), which keeps nested arcs from
  crossing (the argument is at `svg::net::height`, a unit test checks
  it); and a disconnected structure has every part but the first in the
  highlight colour, as a switching cycle has (snapshot
  `disconnected.svg`).
- **Two bugs found and fixed on the way**: the text tree panicked on any
  tree wider than 65 535 columns (a width in a format string is a `u16`
  in this Rust; the compact tree of the net above reached it), and the
  capped arc's product `cap · rx` could overflow on an enormous drawing.
- **Snapshot differences**: new `compact.{svg,tex,typ}`, `high.typ`,
  `open-linlog.typ`, `names.tex`, `disconnected.svg`; changed `net.svg`
  and `cycle.svg` only, by the anchors at the atom (the edges, links and
  nodes of negated literals move onto the atom, the text stays, and the
  wider arcs make each view box 92 units taller). No other snapshot
  changed; `link_cap`'s default changes none.

Verified in the second session, every run in a memory-capped scope on
cores 4 to 9: see "Verification of the second session" below.

## Verification of the second session

Every cargo run in a scope of its own (`systemd-run --user --scope -p
MemoryMax=8G -p MemorySwapMax=0`, `taskset -c 4-9`, six build jobs, four
test threads), on the final tree: `cargo test --workspace` (core 159
passed and 2 ignored, export 11, CLI 19, the rest passing),
`cargo clippy --workspace --all-targets -- --deny warnings`, `cargo hack
check --each-feature -p linlog` and `--feature-powerset --depth 2`, all
clean; `BLESS=1` once for each change, the diffs read (the list above).
Typst 0.15.1 with curryst 0.6.0 (the check's, from nixpkgs) compiled
`high.typ`, `open-linlog.typ`, `compact.typ` and a scratch tree of 41
inferences with a Mix and an open goal, and gave "maximum show rule depth
exceeded" for curryst trees ten high with a binary rule. The Petri net
ran four times on one core in a 2 GiB scope (debug build, `--jobs 1`):
`--compact never` and `auto` each 1.1 s and 65 MB, left out at 64 MiB;
`always` with `--derivation-limit 1GiB` refused by the lower bound;
`always` with no bound built the 128 inferences within 71 MB and then
hit the width panic, which the fix of that commit removes (its test
draws a tree 140 000 columns wide). Not run: the text of that compact
tree written whole (about 160 MB).

## Outcome of the first session

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
- **PNG and PDF** (the author's request, below): core features `png`
  (resvg 0.47) and `pdf` (krilla 0.8.2 with krilla-svg 0.8.1), off by
  default like `parallel`, `export::png::from_svg` and
  `export::pdf::from_svg` rendering any SVG drawing with the font data
  the caller gives; the command embeds Euler Math (`cli/fonts/`, with
  its OFL text) and has `--format png` and `--format pdf` on `prove`,
  `check` and `seq print`, `proof --png FILE`/`--pdf FILE` in a session.
  Deterministic: the same drawing gives the same bytes (no system
  fonts, no date in the PDF).
- **Archival and accessible PDF** (the author's decisions, below): every
  PDF is PDF/A. PDF/A-4 (PDF 2.0) by default, PDF/A-2u (PDF 1.7) with
  `pdf::Options::compatible`, PDF/A-2a with PDF/UA-1 (PDF 1.7) with
  `pdf::Options::accessible`, tagged as one figure whose alternative text
  is the drawing's description. The library reads no clock: the date is
  `pdf::Options::date`, `RenderError::NoDate` without it; the command
  takes `SOURCE_DATE_EPOCH` or the clock. veraPDF 1.30.2 passes every
  profile, in the `export` check.
- **Accessible drawings**: `role="img"` on every SVG, its `<title>` the
  accessible name, and (`Style::description`, on by default) a `<desc>`
  that reads a derivation as numbered inferences, premises first
  (`Derivation::write_steps`), or a net as its links. The PNG carries
  title and description as iTXt and declares sRGB and its density (192
  dpi at the default scale 2, so it shows at the drawing's size).
- **An output is made only with something in it beyond the verdict.**
  Found when the author asked whether every output is well-formed: an
  unprovable sequent had left an empty `.png` or `.pdf`, and an SVG that
  was its verdict comment alone, no XML document (the latter since step
  11, and pinned by a test, which now pins the new behaviour). Then the
  author: "a file should generally only be written if there is a proof
  tree / net … whenever there is actually something that can be
  output". So a file of any format but JSON holds a derivation or a net
  or is not made, the verdict going to standard error; an SVG on
  standard output is a drawing or nothing; a derivation cut short by the
  time limit leaves no file. JSON is always written, being the outcome
  with its refutation and statistics (this session's reading of
  "something that can be output"); standard output keeps the verdict
  line of the text formats, which is the answer on a terminal. An
  earlier run's file at the path is left as it is, the exit status
  saying that no proof was found (asked, the author: "Sounds good like
  it is now"). The session's `show` writes the partial derivation to a file
  (`show part.pdf`, open goals included); the bare `show latex` is gone,
  a bare word being a file name as in `proof`.
- **`--net` replaces the formats `net` and `net-svg`** (the author:
  no aliases, the command is free until the first release): it writes
  the proof net instead of the derivation in text, svg, png or pdf.
  **The format follows the `--output` file's extension** (`.txt`,
  `.json`, `.tex`, `.typ`, `.svg`, `.png`, `.pdf`, `.v`) unless
  `--format` names one; text otherwise.
- **Ids a client can click**: with `Style::ids` every formula of a
  conclusion is a group `i<n>-<p>`, `p` being the position
  `Interactive::apply` takes; `Interactive::derivation_ids` maps a
  drawing's inference `n` to the session's goal.
- **Every rule label in a snapshot**: `every_label` pins the classical
  rules, Mix, affine weakening and the intuitionistic rules in every
  target (and the two label proofs as certificates), asserts that the
  derivations use all 33 rules, and pins LaTeX and Typst fragments
  (subscript labels, a dashed open goal) that the `export` check wraps
  in a document and compiles.
- **Bounded reports**: `check`'s verdict line abbreviates the sequent
  (`--abbreviate`, 200 characters by default, `… (N formulas)` after
  the cut), with `--quiet` too; `CheckError::describe(…).abbreviated(n)`
  cuts every formula and every formula list of an error report, which
  the command uses for `check` and for an invalid proof met while
  drawing.

Verified in this session: `cargo test --workspace` (all pass; the
snapshots are byte-identical under the defaults apart from the font
lines), `cargo clippy --workspace --all-targets -- --deny warnings`,
both `cargo hack` runs (each feature, every pair; `png` and `pdf`
included, with and without `serialize`), `cargo deny check`, the
`export` check (pdfLaTeX and Typst on every snapshot and fragment, resvg
on the SVGs, and the command's PNG and PDF through pngcheck and
poppler) and the `rocq` check (both label certificates). `nix flake
check` passed on the final tree (every check, in a capped user unit),
after the PDF profiles and the rule that a drawing format writes a whole
drawing or nothing.

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
| png | `scale` | 2 (`png::Options::DEFAULT_SCALE`) | `--style png.scale=3` |
| png | `pixels` | 2²⁶ (`DEFAULT_PIXELS`) | `--style pixels=none` |
| pdf | `embed_text` | `true` | `--style embed_text=false` |
| pdf | `compatible` | `false` (PDF/A-4) | `--style pdf.compatible=true` (PDF/A-2u) |
| pdf | `accessible` | `false` | `--style pdf.accessible=true` (PDF/A-2a + PDF/UA-1) |
| pdf | `title` | `null`: the drawing's title | `--style 'pdf.title=A proof'` |
| pdf | `language` | `en` | `--style pdf.language=de` |
| pdf | `date` | `null`: the library refuses; the command `SOURCE_DATE_EPOCH` or the clock | `--style 'pdf.date={"year":2027,…}'` |
| svg | `description` | `true` | `--style svg.description=false` |
| rocq | `form` | `fragment` | `--standalone` |
| rocq | `lemma` | `certificate` | `--lemma NAME`, `--style lemma=…` |
| rocq | `prelude` | `From NanoYalla Require Import macroll.` | `--prelude TEXT` |
| command | screens a tree may fill | 3 (`SCREENS`) | `--screens N\|none` |
| command | sequent in a verdict line | 200 characters (`ABBREVIATE`) | `--abbreviate N\|none` |
| command | verdict line | written | `--no-verdict` |
| typst | `layout` | `auto`: curryst up to `CURRYST_HEIGHT` (9), linlog's own above | `--style layout=linlog` |
| typst | `premise_gap` | `1.5em` | `--style premise_gap=2em` (own layout) |
| typst | `label_gap` | `0.2em` | `--style label_gap=0.3em` (own layout) |
| typst | `band` | `0.8em` | `--style band=1em` (own layout) |
| typst | `stroke` | `0.05em` | `--style stroke=0.5pt` (own layout) |
| svg | `link_cap` | 8000 (8 em of half-width) | `--style link_cap=4000`, `--style link_cap=null` for none |
| view | `ViewOptions::compact` | `auto` | `--compact auto\|always\|never` |

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
- **PNG and PDF take the SVG, not the derivation**: `from_svg` renders
  whatever `svg::write` or `svg::net` drew, so the drawing has one
  layout and one stop condition (the SVG's), and a front end that
  already has the SVG renders it without drawing again. The fonts are
  an argument because usvg drops a text whose font is missing without
  an error, and the web page has the font already (it would ship twice
  if the library embedded it).
- **A PDF page is the drawing at 0.75 pt per pixel**, CSS's ratio, so
  the default 16-pixel text is set at 12 points.
- **A PNG is bounded in pixels** (`pixels`, 64 million by default, 256
  MiB while drawn), refused before anything is allocated (D16).
- **The binary formats' verdict goes to standard error**, as JSON's
  read-time "unknown" did, and they are never written to a terminal.
- **A format's name prefixes a `--style` key only before a dot**:
  `text` alone is the SVG's colour field.

## Requests of the author during the step

- 2026-10-03, during the first session: "add PDF as well as PNG/JPG
  export". The step's prompt says "no new export target"; the author's
  request overrides it. A sub-agent's research (scratch build, two
  cores, 8 GB cap) found resvg for PNG and krilla with krilla-svg for
  PDF (svg2pdf is archived in krilla's favour), deterministic and pure
  Rust, and four crates outside `deny.toml`'s licences: tiny-skia and
  tiny-skia-path (BSD-3-Clause), arrayref (BSD-2-Clause), jpeg-encoder
  (IJG). The author's decisions: allow BSD-2-Clause and BSD-3-Clause
  (asked why they were denied: nothing recorded a reason, they are
  permissive like MIT); no JPEG and no JPEG XL (JPEG blurs line
  drawings and is larger than PNG there; JPEG XL has no mature pure-Rust
  encoder and pdfLaTeX, Typst and most tools do not read it); the
  library takes font data from its caller and the command embeds Euler
  Math; `--net` as a switch instead of `net-png`/`net-pdf` formats, no
  aliases for `net` and `net-svg`, and the format inferred from the
  output file's extension, `--format` overriding it.
- 2026-10-04: asked whether the outputs follow their specifications and
  whether the PDF can be archival and accessible. An audit found LaTeX,
  Typst and Rocq idiomatic and compiled, the SVG without `role="img"`,
  the PNG without colour space, density or title, the PDF neither
  archival nor accessible; a second research agent and this session's
  own tests (TeX Live 2025, Typst 0.15.1, poppler, MuPDF, Ghostscript,
  qpdf, Inkscape, veraPDF) found PDF/A-4 opened everywhere, Typst
  refusing a PDF 2.0 image by default, the TeX engines warning, and
  several archives not listing PDF/A-4 (the rules file has the list).
  The author's decisions: PDF/A-4 by default ("typst imports SVG just
  fine"), PDF/A-2u under `compatible`, accessibility as PDF/UA-1 even
  without `compatible` until krilla has PDF/UA-2, a note on the
  accessible output in the command, the web client and the doc comment,
  and the date always supplied to the library, the command using the
  clock unless `SOURCE_DATE_EPOCH` is set. Correction recorded: this
  session first told the author pdfLaTeX included a PDF 2.0 figure
  silently; it warns, as LuaLaTeX does (a cut-off log hid the line).
- `cargo deny check advisories` then flagged two crates as unmaintained
  (no vulnerability): rustybuzz (RUSTSEC-2026-0206) and ttf-parser
  (RUSTSEC-2026-0192), beneath usvg 0.47 and krilla. usvg 0.48 has moved
  to harfrust and skrifa, but krilla-svg 0.8.1, the latest, requires
  usvg 0.47 and krilla uses rustybuzz itself. `deny.toml` ignores both
  with that reason, to be dropped when krilla moves on; this was this
  session's call, which the author may reverse.

## Decisions of the second session

- **The compact view is a field of `ViewOptions`, not of each output's
  options** (the first session's note proposed the latter). Only the
  builder can avoid building the whole derivation, which is what the
  bound is about, and every output that draws a tree reads a derivation
  built under `ViewOptions`, so one field reaches the text tree, LaTeX,
  Typst, SVG, PNG and PDF alike; the web front end holds it in the same
  JSON as the bounds.
- **A run is a run of one rule**, as the step words it: consecutive
  inferences of the same structural rule, compared after the
  intuitionistic renaming. Derelictions are not structural and are not
  merged; nothing is permuted.
- **`auto` compacts only past a bound, and only under a bound in
  bytes.** Within the bounds the derivation is drawn as before, which is
  what keeps the defaults' output. The compact build is not measured
  beforehand (an exact compact size in the size pass would track runs
  across nodes and through what a `⊤` absorbs); it counts as it goes and
  gives up at the bound, and the size pass's lower bound refuses the
  hopeless cases at once. The first version tried a compact build
  without any bound when both were lifted; a core test that expects
  `TooMany` on 2⁷⁰ inferences then took the machine's memory until it
  froze and had to be reset (the run was not in a capped scope, against
  the step's rule). Now `auto` never tries without a bound, and the
  rules file says why.
- **The star**: `*` became a symbol of the label markup, a superscript
  in LaTeX and Typst and a plain star in text and SVG, so a user's table
  can use it too. A count (`?w×65640`) was not chosen: the star is the
  usual notation, and the count is in `Inference::times`, in the JSON a
  front end builds from it and in the drawing's description
  (`by !w 3 times`).
- **`CURRYST_HEIGHT` is 9**, measured with Typst 0.15.1 and curryst
  0.6.0: a tree whose longest branch has a binary rule fails at ten
  (chains of `⊗` with the leaf on either side, full binary trees), while
  one-premise chains compile at thirteen. The switch is by height alone,
  so a one-premise chain of ten to thirteen goes to the own layout though
  curryst would set it; that costs nothing but the package's look.
- **The own layout is Typst code in the output, measured by Typst**,
  not widths from linlog's advance table: the output chooses no font
  (D12), so only Typst knows the widths. Rows share a baseline by a
  measurement (the depth a strut reveals), and pieces are placed in one
  box, so nothing nests per level. The script is written after the data
  so that the inferences stream out as they are made.
- **The session has no `--compact`**: its derivation is made a rule at a
  time and its grafts are read rule by rule (`proof()` would translate a
  run wrongly), so `of_goal` forces `never`.

## Open questions, and what later steps must know

- **The Petri net's derivation stays out at the default bound**, compact
  or not (above). What would show it is a translation that weakens a
  `?` formula unused above at the root rather than where the term's `?`
  step sits, so that no sequent above carries it; the run would then be
  one inference at the bottom and the rest of the tree small. That
  changes the derivation view's shape for every proof with unused `?`
  formulas and the size pass with it, so it is a decision for a later
  step (with the size pass's exactness argument redone), not a part of
  the compact view.
- **The left-out line names the whole derivation's size** even where
  the compact view was tried; it could add the compact view's lower
  bound (`size::Firm`), which needs a field in `ViewError`: step 28.
- **A compact view of a session's drawing** is not offered: its
  inference ids are the goals' handles (`derivation_ids`), which a merge
  would renumber. If the web front end wants one, it is a pass over
  `Interactive::derivation()` with the id map carried through.
- **Quantifiers (D17)**: a first-order rule is a new `Rule` with an
  entry in both label tables; the Typst layout and the compact view read
  nothing of the rules but the label and `Rule::is_structural`, so `∀`
  and `∃` need no change there, and a structural rule of a later calculus
  says so in `is_structural`.
- **`\cdotp` is punctuation** in TeX; inside braces it is an ordinary
  symbol, which is how the LaTeX names write it.


- Step 28, from the second session: `Inference::times` is a public
  field every constructor now sets (one outside a compact view); the
  Typst layout's four spacing fields are Typst code written verbatim,
  like `import` and `page`, and not checked; `Compact` sits at the crate
  root beside `ViewOptions`.
- Step 32, from the second session: `ViewOptions` with `compact` is the
  JSON the client sends with a derivation request; `auto` already does
  what a page that must stay responsive wants. The own Typst layout
  needs nothing of the client.
- Step 28 (the API's surface): `RenderError` stands outside `Error` as
  the other export errors do; `rocq::Options::lemma` is not checked to
  be an identifier; the exports' `String` functions and their `write`
  twins could be one generic call.
- Step 32 (the web front end): the JSON of `Styles` (`cli/src/style.rs`)
  is the settings object, one key per format; the PDF needs the date
  from the browser's clock (`pdf::Options::date`), and the accessible
  PDF's note (that a screen reader reads the description, not the
  drawing) belongs next to its switch, as the author asked; `png::from_svg` and
  `pdf::from_svg` want the font bytes the page already loads;
  `Interactive::derivation_ids` with `Style::ids` maps a click to
  `(goal, position)`. The renderers are said to build for
  wasm32-unknown-unknown (resvg claims it, a wasm build of typst-pdf
  shows krilla does), but this repository has not built them for wasm.
- Later: PDF/UA-2 when krilla has it (`accessible` alone then moves to
  PDF/A-4); Inkscape warns "Couldn't parse text in PDF from UTF16" on
  krilla's PDFs of either version, which concerns krilla's text, not
  PDF 2.0, and imports them whole.
- Step 30 (the release): a binary release ships the third-party notices
  (MIT, Apache, BSD, the OFL of the embedded font); `cargo about` makes
  the list.
- Left as found, for later steps: `--stats` says "copy bound reached"
  without whose level it is (the `Statistics` has one number; step 26
  owns the schedulers); `rocq::Options::lemma` is not checked to be an
  identifier (step 28, the API's boundaries).
