---
paths:
  - "core/src/export/**"
  - "core/src/proofs/style.rs"
  - "core/tests/export.rs"
  - "core/tests/snapshots/**"
---

# linlog core: the exports

Loaded, beside `core.md`, when a file of the exports, the rule labels
or the export snapshots is read or changed. The text tree is the derivation
view's (`core-derivations.md`).

## Export

`export/` writes sequents and derivations as LaTeX (ebproof trees, cmll and
amssymb symbols) and Typst (curryst trees), draws them and proof
structures as SVG documents, and writes derivations as Rocq proof scripts
for NanoYalla. What the code relies on:
- **The entry points**: `latex`, `typst` and `svg` write
  `sequent(&sequent, mode, &options, &limits)`, one-sided or in
  intuitionistic mode two-sided by its reading (`Error::NotIntuitionistic`
  without one), refused past `limits.derivation_bytes` before anything is
  laid out (`export::printed`: `PER_OCCURRENCE` bytes, 32 for LaTeX and
  Typst and 128 for SVG, plus the longest atom name, per occurrence;
  `Refusal::Output { what: "sequent" }`), since a JSON sequent of a few
  hundred bytes can unfold to millions of occurrences and SVG took 310
  bytes each (F5); and each target's one `write(&derivation, &options,
  out, stop)` takes
  anything `export::Drawable` (an enum: a linear derivation, finished or
  with open goals, the two-sided view of a classical one, `TwoSided`, or
  one of LK or LJ, by `From`), as a fragment or a
  standalone document by `Options::form`; `svg` draws under a `Style` (the font
  and its advances, labels, the open goal's shape, ids per formula,
  sizes, gaps, colours) and also `net(&structure, &style, &limits,
  stop)` (`Refusal::Output { what: "drawing" }` past
  `limits.derivation_bytes`, the stop asked as the criterion asks it);
  `rocq::write` writes the lemma `Options::lemma`, or a file that starts
  with `Options::prelude`, for a linear derivation NanoYalla's script,
  refusing an open goal, Mix, affine weakening and a compact derivation
  with `Unsupported`, and for one of LK or LJ the term over `Prop`;
  `png::from_svg` and `pdf::from_svg` render a drawing. There are no
  `derivation(…) -> String` twins and no per-type `ordinary` functions:
  a `String` is an `fmt::Write` (F36).
- **`export::Styles` holds every format's options in one value** (the
  command's `--style` keys and `Settings::styles` address it): a field per
  format, and for a format the build lacks a private `Absent` that reads
  any value and writes nothing, so one settings file reads on every
  build and a misspelt format is still refused.
- **A new export option is a field, never a constant.** Every output
  has one plain-data options value with `Default`, `Clone`, `PartialEq`
  and serde behind `serialize` (`serde(default, deny_unknown_fields)`,
  so a JSON with some fields is the defaults with those changed and a
  misspelt field is an error): `TextOptions` (`proofs/fmt.rs`),
  `latex::Options`, `typst::Options`, `svg::Style`, `png::Options`,
  `pdf::Options`, `rocq::Options`, each `#[non_exhaustive]` with public
  fields, so a caller outside the crate makes one from `default()` and
  sets the fields that differ (a struct literal is refused, and a later
  field breaks nobody), in a binding or through a `#[must_use]`
  `with_*` per field, which every one of them, `export::Styles`,
  `Settings` and `svg::Font` have (a new field gets its builder; a
  non-exhaustive variant with fields gets a constructor, as
  `Advances::table`). The
  `Form` is a field of the three that have one. Presets are named
  values (`Style::dark()`, `Style::monospace()`, `Font::monospace()`).
  The defaults reproduce the output the snapshots pin, so a new field's
  default is today's behaviour.
- **One signature writes a derivation**: `latex::write`, `typst::write`,
  `svg::write`, `rocq::write` and `Derivation::write_text` take the
  derivation, the options, any `fmt::Write` and a stop closure, and
  answer an `Error` (`Refusal::Stopped { phase: Write }`, `WriteFailed`,
  `Unsupported` for Rocq,
  which refuses before it writes anything). The emitters make one
  inference in a buffer and hand it on (`notation::flush`, which asks
  the stop after each), so they hold one inference's text; the SVG tree
  keeps a few numbers per inference and lays a conclusion out again
  when it writes it (a laid-out run per inference was 50 bytes per
  character of sequent). A `String` takes the whole with a stop that
  never fires. The command writes the verdict and then the
  derivation into its output as it is made (`cli/src/io.rs`, `Output`).
  A derivation of LK or LJ (`ordinary::Derivation`) goes through the same
  `write` of every target, and has `ordinary::Derivation::write_text`
  (with `text_size` and `Display`) and `write_steps`; Rocq's `write` certifies it by the
  term of `ordinary/rocq.rs` (`core-ordinary.md`).
- **Every emitter is generic over the crate-private trait
  `proofs::style::Drawn`**, implemented by `proofs::Derivation` (in
  `style.rs`) and `ordinary::Derivation` (in `ordinary/derivation.rs`):
  the root, the premises, a rule as a number below `RULES` (`OPEN` the
  open goal's, `None` for LK/LJ), `markup(rule, labels)`, `times` (1 for
  LK/LJ), `label` (the markup and `RUN` for a run), the sequent as the
  text tree writes it (`write_sequent`) and in a `Notation`
  (`sequent(out, notation, id, aligned, marks)`), and the SVG's
  `positions` for `Style::ids`. The text tree, the steps of `<desc>`,
  `notation::walk` and the LaTeX, Typst (both layouts) and SVG trees are
  each one generic function; the public `write` takes `impl
  Into<Drawable>` and matches the enum into the generic function, since a
  public function cannot name a crate-private bound (a sealed public
  trait, as `Owner` is, would do as well; the enum needs no
  `#[expect(unnameable_types)]`). A new kind of
  derivation implements `Drawn` and adds a variant of `Drawable` with its
  `From`, and an arm in each `write`; the per-rule label tables (the text
  tree's `bars`, the SVG's laid-out labels) are indexed by `rule`, plus
  `RULES` for a run.
- **Rule labels are one table per convention** (`proofs/style.rs`:
  `UPRIGHT`, `SUBSCRIPT`, indexed by `Named::index` in the order of
  `Named::ALL`, plus the user's `Labels::Table` keyed by `Named`), written in a markup
  that each target sets its own way (`parts`: symbols `⊗⅋&⊕⊸!?⊤⊥01`
  and the ordinary `∧∨→¬↔`; the `~` of a two-sided classical view's
  `~L`/`~R` is text, which each target escapes, so the markup a user's
  table writes reads as it did,
  `_x`/`_{xy}` subscripts, `*` a superscript star where the target sets
  one (the mark of a run), other text upright; `latex::label`,
  `typst::label`, `svg::label`, `style::plain` for text). The upright
  table read as plain text is `Rule::name` exactly, and
  `Named::from_str(rule.name())` is the rule, for every named rule
  (`names_round_trip`, in `style.rs` and `rule.rs`): the interactive JSON
  depends on both. A new rule is a new entry in `Rule::ALL`, its named
  forms in `Named::ALL`, both tables, `name` and `from_str`.
  The labels of LK and LJ are `ordinary::Rule::markup(subscript)`
  (`Upright` the upright one, `Subscript` with `subscript`, `Off` none),
  read off one table of the two conventions (`ordinary::derivation::
  LABELS`, in the order of `ordinary::Rule::ALL`, which `rule as usize`
  indexes), both of which read as `Rule::name` in plain text
  (`labels_are_the_names`, F72); a `Labels::Table` is keyed by the
  linear rules, so it leaves them upright. A new ordinary rule is an
  entry in `ALL`, `LABELS` and `name`.
- **One table per target, one printer** (F82): `sequents::notation`
  (always compiled) has `Notation`, the symbol table (connectives,
  units, the dual's marks before and after a name, turnstile, the
  alignment mark, the atom writer `Atoms`, and
  `ordinary`, the `ordinary::Symbols` of `¬ ∧ ∨ → ↔ ⊤ ⊥`: LaTeX's
  `\lnot ` keeps its space, since `¬` is written straight before its
  operand; Typst and SVG take `Symbols::UNICODE`), and its walks over
  any `fmt::Write`: `term`, `ill`, `one_sided`, `sequent`, `reading` and
  `ordinary` (a sequent of LK or LJ two-sided over `Formulas::write`,
  lined up at the turnstile as a two-sided linear sequent is). `TEXT` is
  the table of every `Display` (`Sequent`, `Formula`, `Reading`,
  `IllFormula`, the ordinary sequent) and of the text tree, `~a` for a
  negated atom, so the exports and the text cannot drift; a new
  connective is an arm of these walks and a spelling per table. A new
  target is a new table. `Notation::sequent`
  with `marks` puts `\u{2}`/`\u{3}` around every formula, which the SVG
  layout turns into a group per formula (`Style::ids`: `i<n>-<p>` for
  position `p` of inference `n`'s sequent, the position being the one
  `Interactive::apply` takes, in the drawn order hypotheses first; in a
  derivation of LK or LJ the hypotheses are `0..left.len()` and the
  formulas right of `⊢` follow, which is also the drawn order).
- **The walk keeps its own stack** (`notation::walk`, enter and exit
  events): exits are ebproof's postfix order, enter/exit brackets
  curryst's nesting. Nothing in the emitters recurses over the tree, so
  the output is linear in the inferences (each prints its whole sequent)
  and never as wide as the tree, unlike the text renderer. The formula
  printers do not recurse either: they are loops over `sequents::fmt::Walk`,
  as `Display` is.
- **An open goal's shape is `OpenGoal`**: by default its sequent under
  vertical dots with no inference line (`\hypo{\vdots}` then
  `\infer[no rule]1{…}`; a curryst leaf that is a centred `grid` of
  `dots.v` over the sequent), bare in the text tree; `Bare`, `Mark` (a
  leaf rule labelled with the mark) and `Dashed` (ebproof's `dashed`
  style; in Typst a `grid.hline` over a grid of one column, since
  curryst 0.6.0 has one stroke per tree; a dash array in SVG; `╌` in
  text). Neither package has a per-inference dotted bar.
- **Typst symbols are Unicode characters, not names**: Typst 0.15
  removed `times.circle` and `plus.circle`, so names break across
  versions and characters do not. `&` is `class("binary", \&)`, `?` is
  `class("normal", ?)` (Typst spaces punctuation), letters in labels are
  `upright(L)` (a string in math keeps the space before it). Only the
  two-sided LaTeX tree aligns turnstiles (`&\vdash`); a one-sided
  sequent would align at its left edge, so it stays centred.
- **Limits of the packages, not of the emitters**: Typst 0.15.1 refuses
  a curryst 0.6.0 tree ("maximum show rule depth exceeded": curryst nests
  several layout elements per level) ten inferences high with a binary
  rule on the branch (nine compiles, measured on chains of `⊗` with a
  leaf on either side and on full binary trees), while chains of
  one-premise rules compile at thirteen; `typst::CURRYST_HEIGHT` is 9
  for that reason, and moves with the two versions. ebproof compiled a
  120-high tree; TeX fails with "Arithmetic overflow" on a sequent line
  wider than its largest dimension (about 5.7 m), which no layout fixes.
  Typst 0.15.1 refuses a formula nested 255 brackets deep ("maximum
  parsing depth exceeded": a chain of 256 atoms under `⅋`, every binary
  subformula being bracketed; 255 atoms compile), in either layout.
- **linlog's own Typst layout** (`typst::Layout`, `Auto` by default:
  curryst up to `CURRYST_HEIGHT`, the own layout above; `LAYOUT`): the
  output lists the inferences in preorder as `(premises, label,
  conclusion)` (`none` premises for an open goal, drawn as `open` says)
  and then a fixed script in a `#context` block that measures every
  sequent and label, lays the tree out as `svg/tree.rs` does (widths
  bottom-up over the reversed preorder with a stack, positions top-down),
  and places every piece in one `box` with `place`. Nothing is nested
  per level, which is what makes any height compile (41 in the `high`
  snapshot). Rows share a baseline: a sequent's ascent is its measured
  height less its depth, and the depth is what a strut of `3em` placed
  before it adds to the height. Typst code joins the value of every
  expression statement into the output, so a `pop()` whose value is not
  wanted is `let _ = …pop()`, and a closure cannot change a captured
  variable (loops instead of `map`). The spacing fields (`premise_gap`,
  `label_gap`, `band`, `stroke`) are `typst::Length`s, a number and a
  unit checked when made or read (`InvalidOption`, key `typst`), so an
  option's value never reaches the output as other code (F38); `import`
  and `page` stay code by design, written verbatim. The defaults are
  curryst's.
- **Snapshots**: `core/tests/export.rs` pins standalone documents in
  `core/tests/snapshots/` (`BLESS=1` rewrites them; `ordinary.*` is an
  LJ derivation read back through the public API); the flake's `export`
  check compiles every file there, by a glob, plus two CLI outputs with pdfLaTeX
  and Typst, which is what catches output that matches its snapshot but
  does not compile. The crane source keeps that directory
  (`modules/workspace.nix`), since `commonCargoSources` alone drops it.
  `typst::CURRYST` and the nixpkgs curryst in `modules/export.nix` move
  together.
- **No font in LaTeX and Typst, a math font in SVG.** The LaTeX and Typst
  output never chooses a font, standalone or not: it is pasted into a
  document and takes that document's fonts. Only SVG names one
  (`svg::Font`, Euler Math by default). The export check runs Typst with
  `--ignore-system-fonts` (its embedded fonts serve) and resvg with
  `--skip-system-fonts` and one math font alone (Euler Math for the
  snapshots, each `--font` for the command's drawings in it, its own
  TeX Live directory), and fails on any output, so a font resvg cannot
  find, or a glyph it lacks (usvg's fallback warns), fails instead of
  falling back silently.
- **Five math fonts** (`svg::MathFont`: Euler Math 0.75, STIX Two Math
  2.12, Garamond-Math 2022-01-03, Asana Math 0.962, all OFL 1.1, and New
  Computer Modern Math 7.1.1 in its Book weight, the package's default,
  under the GUST Font License; all 1000 units per em), each setting every
  character of the table (`⅋` included: Latin Modern Math, Libertinus
  Math, the TeX Gyre fonts and Fira Math lack it, Fira Math `⊸ ⊢ ⊥ ⊤ ⋮`
  too). New Computer Modern came by the author's request: the GUST Font
  License is the LPPL 1.3c, whose clause 2 makes distributing one file of
  the package a modification, so `NewCMMath-Book-LICENSE.txt` meets
  clause 6 (the file unchanged, no support implied, where the complete
  Work is) before the GUST and LPPL texts, which must stay verbatim (no
  whitespace stripped). Its family name is `NewComputerModernMath`, which
  its Regular weight shares: give a renderer the Book file alone (the
  export check copies it apart). XITS Math is STIX's predecessor in the
  same design.
  `svg::Font` reads in JSON as a font's name (`"stix-two"`, a hand-written
  `Deserialize`: a string is a `MathFont`, a map the fields) and always
  writes the fields; `MathFont::font` is the preset, whose `family` is
  the file's family name then `serif` (Euler's keeps Neo Euler).
- **The tables are the files'**: `font.rs`'s `CHARACTERS` are the
  sorted characters and `ADVANCES` one table per `MathFont` (the order
  of `MathFont::ALL`, `column`) in their order, each the font's `hmtx`
  advance of the character (read with fontTools, `getBestCmap` and
  `hmtx`, Euler's unchanged). The characters are kept apart from the
  advances because rows of a character and four advances made the
  binary search walk wider entries: the `render-svg` journey counted
  2.1 % more, 0.4 % with the split; `every_table_is_its_font_s`
  reads each file in `cli/fonts/` with skrifa (a dev-dependency, the
  version krilla already locks) and fails on a missing glyph or an
  advance that differs, so a new font or a new character of a printer
  gets its column or row from the file, never by hand. A character
  outside the table takes `FALLBACK` (650) in every font; a viewer
  sets it in whatever font has it and `textLength` keeps the layout,
  and usvg sets it in another loaded font that has it (a warning) or
  draws the font's `.notdef`.
- **The vertical metrics stay Euler Math's for every font** (`HEIGHT`,
  `DEPTH`, `AXIS`, `SCRIPT`, `RAISE`, `LOWER`), measured against each
  file on 2026-10-22: the raised `⊥` reaches 875 (STIX), 752
  (Garamond), 805 (Asana), 879 (New Computer Modern) under `HEIGHT` 890;
  every comma is within `DEPTH` 210; the axis is 258, 250, 271 and 250
  against 250 (a fifth of a pixel at 16 px); the deepest descender of the
  table is 235, 290, 286 and 250 (Euler's own `q` 243), inside `DEPTH` plus the margin (300).
  Making them a font's would change Euler's output. One place takes a
  font's own depth: a net starts a premise edge under a written formula
  at `Advances::depth` (the deepest descender of the table: 235, 290,
  286, 250), not `DEPTH`, so that no edge runs into a `g`; Euler Math and
  a caller's font take `DEPTH`. Both drawings do, the compact one's
  literals included (`net_edges_clear_the_font_s_descenders` checks
  both): its edges once began a tenth of an em below the baseline,
  inside every font's descenders.
- **SVG** (`export/svg/`): a third table (`NOTATION`, with the atom
  letters as mathematical italic codepoints, which a math font sets as
  math italic, and `\u{1}` standing for the raised `⊥`; `PLAIN` for the
  `<title>`), and layouts of its own: `tree.rs` (a post-order pass over
  `walk`'s exits for box widths, a pre-order pass over its enters for
  positions; uniform rows of `line_height`), `net.rs` (literals in id
  order, which is left to right; connectives by height; links as
  half-ellipses whose height is proportional to their width up to
  `Style::link_cap` and to its square root beyond, so nested links never
  cross: `net::height` says why; the two drawings below). Widths are integer thousandths of an em from
  `font.rs`'s advance table of Euler Math 0.75 (a fixed fallback outside
  it); all coordinates are integers, so the output is byte-stable. The
  ordinary connectives `¬ ∧ ∨ → ↔` are in the table from the font's
  `hmtx` (`ttx -t cmap -t hmtx` of `cli/fonts/Euler-Math.otf`, which
  also gives `⊗` 668 as the table has it); a character added to a printer
  gets its advance the same way, or takes the fallback of 650.
- **No drawing is held twice** (F5): `document` takes a closure that
  writes the body into the document itself; the sequent's line is laid
  out twice by `lay` (its width for the head, then each piece written as
  it comes, `write_piece`), so no list of pieces is held; the net's
  groups are written from their five strings into the document. A
  sequent of 2.1 million occurrences peaked at 650 MB for 205 MB
  written, when the pieces, a body and the document's copy were held.
- **A style's numbers have maxima** (`Style::check`, which `svg::write`,
  `sequent` and `net` call before anything is drawn, `InvalidOption`
  naming the key): lengths `MOST_LENGTH` (a thousand ems), `label_size`
  and every advance `MOST_SIZE` (ten ems), `font_size`
  `MOST_FONT_SIZE`. Under them a coordinate grows by at most a million
  thousandths per element, so `i64` holds every drawing the memory
  holds; the products with a size (`run`'s `scale`, `head`'s pixels)
  are taken in `i128`, and `decimal` writes a negative number with one
  sign. A style of huge numbers overflowed in debug and wrote
  `width="-96823495355.-824"` in release (F4). A new number of the
  style gets its maximum in `check`. `scale` multiplies in `i64` and
  goes to `i128` only when the product passes it: the division of an
  `i128` is a call, a sixth of the SVG journey's growth when every
  width took it.
- **An atom writer is a variant of `sequents::notation::Atoms`**, a
  closed set matched in one generic `write`, never a function pointer:
  through `&mut dyn Write` every character of a name was a virtual
  call, and the Typst journey counted 24 % more than with the writer
  compiled for its `String`. A new target's writer is a variant there,
  behind its feature, and a generic function in its module.
- **`escape` is the one place text enters a drawing**: it writes what
  XML 1.0 cannot carry (a control character but tab, line feed and
  carriage return, U+FFFE, U+FFFF) as `�`, whatever string a style or
  a label table holds (F39: a control character made the `<desc>`
  invalid XML, so PNG and PDF failed on a proved sequent). Atom names
  need nothing of their own: they are identifiers, which hold no
  control character, so none can pose as the layout's markers
  (`\u{1}` to `\u{5}`).
- **PNG and PDF render the SVG** (`export/png.rs` with resvg and the
  png encoder, `export/pdf.rs` with krilla and krilla-svg, features
  `png` and `pdf`, `export::parse` and `export::texts` shared):
  `from_svg(svg, fonts, &options)` takes the SVG text any drawing gives
  and the data of font files, and nothing else, so the bytes are a
  function of the arguments. What keeps them so: resvg without its
  default features (no `system-fonts`, no `memmap-fonts`: fontdb is
  built without file access), krilla's document id is a hash of the
  bytes, and **the crate reads no clock**: a PDF's date is
  `pdf::Options::date`, and without one `from_svg` answers
  `Error::NoDate` (every PDF/A part requires a date; the command
  takes `SOURCE_DATE_EPOCH` or the clock, `pdf::Date::from_unix`).
  Calling `load_system_fonts`, or turning those resvg features on, ends
  it. A text whose font is missing is dropped by usvg without an error,
  which is why the fonts are an argument and the command embeds the five
  math fonts (`cli/fonts/`, each with its licence, `show::FONTS`), giving a
  render Euler Math and the one whose family the style names, so the
  default render loads what it always did. `Costs::base` (32 MiB)
  covers the font data, at most two files of 0.4 and 0.8 MB. krilla-svg switches krilla's
  default features on, so none can be turned off there.
- **The PDF is always PDF/A** (the author's decision): PDF/A-4 (PDF 2.0)
  by default, PDF/A-2u (PDF 1.7) with `compatible`, PDF/A-2a with
  PDF/UA-1 (PDF 1.7) with `accessible`, whatever `compatible` says,
  until krilla has PDF/UA-2; then `accessible` alone moves to PDF/A-4
  with PDF/UA-2 and no caller changes. krilla validates on `finish`, the
  `export` check validates with veraPDF. The accessible document is
  tagged by hand around `draw_svg`, which tags nothing: one `Figure`
  whose alternative text is the drawing's `<desc>`, a one-entry outline
  (PDF/UA-1 in krilla requires one), the title and language in the
  metadata. Facts behind the default (2026-10): Typst 0.15 refuses a
  PDF 2.0 image without `--pdf-standard 2.0` (it takes the SVG), pdfTeX
  and LuaTeX warn about the version, Chromium shows no title from an
  XMP-only file, and the UK National Archives, KOST-CECO, ETH Library,
  the USPTO and the Bundesarchiv (without consultation) do not list
  PDF/A-4: that is what `compatible` is for.
- **A drawing is accessible SVG**: `role="img"` on the root, `<title>`
  its accessible name, and with `Style::description` a `<desc>` that
  reads it in order (`Derivation::write_steps`: numbered inferences,
  premises first; a net's text form), which the PNG carries as its
  `Description` and the accessible PDF as its figure's alternative
  text. No ids on the two, so that inlining several drawings in one
  page cannot collide (SVG-AAM takes them from the elements).
- **The PNG declares itself**: sRGB, a density of 96 dpi times the
  scale (so a viewer shows it at the drawing's size), `Title` and
  `Description` as iTXt. Its pixel bound (`png::Options::pixels`) is
  compared before the parse with the `width`/`height` the root declares
  (`declared_size`, plain or `px` numbers, rounded up as usvg does) and
  again after it, for a size the text did not give.
- **A render is bounded before usvg parses anything**
  (`from_svg(svg, fonts, &options, &limits)` within `limits.memory_bytes`,
  default 1 GiB, one bound under one name, F63; `Refusal::Memory { phase:
  Render }`,
  and `Refusal::Pixels` past the pixel bound): usvg sets
  every glyph as a path and strokes every arc to bound it inside one
  call, so nothing can be compared or polled once it runs. The estimate
  is `Measure::of` (bytes, elements, glyphs outside `<title>`/`<desc>`,
  and each arc's time by `arc_micros`, all read off the text in one
  pass) times a `Costs` per output (`PNG`, `PDF`, `PDF_OUTLINES`; the
  PNG adds its pixels and strokes arcs twice). Every coefficient is the
  larger of the memory and the time measured per unit, time counted at
  `BYTES_PER_MICROSECOND` (128 MiB/s), a quarter more: so the bound
  bounds memory and time at once, 1 GiB being at most 8 s of render on a
  slow core (measured on core 6 of the author's desktop; the measured
  peaks and times are 1.3 to 9 times below the estimate). The doc
  comments on `Costs` and `arc_micros` say what was measured; a new
  renderer version or an output with other elements is measured again
  the same way (peak RSS with `/usr/bin/time -v`, one core, capped).
  **Only what the drawings have is rendered**: `Measure::of` refuses
  (`Error::NotSvg`) an element outside `ELEMENTS` (`svg`, `title`,
  `desc`, `g`, `text`, `path`, `rect`, `circle`), any `href` and a
  document type, since `from_svg` takes any text and the estimate
  knows the cost of those alone (F37: six levels of `<use>` in 1 287
  bytes took 227 MB and 4.3 s under 64 MiB; entities expand likewise);
  `arcs` reads a `d` in either quote, and a tag ends at its first `>`
  outside a quoted value (`tag_end`: `<desc class=">"/>` hid the
  `<use>` after it), a `<title>` or `<desc>` at its end tag with any
  white space before the `>` (`through_end_tag`: `</desc >` did too). A new element of the SVG writer
  joins `ELEMENTS` with its cost measured.
- **usvg's cost of an arc is a precision fault, not its size**: it
  computes in `f32`, and an arc starting millions of units out (a net
  wider than a few thousand literals) is stroked in many pieces, up to
  35 ms each (50 to 200 million units), against microseconds near the
  origin; past a billion units it is cheap again. `arc_micros` is a line
  in the start's distance capped at 35 ms plus 1.4 ms for a wide arc. A
  link drawn in coordinates of its own (a `translate` on each arc) would
  avoid the fault; the drawing does not do it yet.
- **A net is drawn with its formulas by default** (`Style::net`,
  `NetDrawing::Formulas`; `Compact` the other value, `--style
  svg.net=compact`): every occurrence is its formula's text (`o<n>` on
  it), a connective's text placed with its own main connective under
  its link's joint, and each premise's subtree as far to its side of
  the joint as the premise stands in the text (half the connective, a
  space, and a bracket when the premise is binary). So a one-sided
  net's every subformula is exactly where it is in its conclusion's
  text, and since the subtrees of a link's two premises lie on either
  side of its joint and a layer is below all of its subtree, no edge
  crosses a formula, a long edge up to a literal included: that is the
  invariant the separation keeps, and a change of it (a gap that is not
  at least the text's) can put an edge through a formula, which no test
  but looking at the rendering catches. Rows are `line_height +
  node_radius + HEIGHT + DEPTH` apart; the joint is `1.5 · node_radius`
  above its text's top and the edges stop `node_radius` from it (the
  `⅋` glyph at `label_size` nearly fills that). A literal that is a
  conclusion stands on the bottom row and its link's path runs down to
  it (`M… V top A… V…`), so the conclusions are all at the bottom in
  the roots' order with no id written twice. Compact keeps the packing
  `literal_gap` apart and each circle halfway between its premises'
  anchors, and spaces the conclusions by their texts, written under the
  edges hanging from the bottom layer (no id on a compound
  conclusion's text: its circle has `o<n>`). The order at the bottom is
  the structure's, one-sided: in the Lambek calculus its hypotheses
  come reversed against the two-sided sequent, as the planar layout
  needs.
- **Cut links and the steps of an elimination** (`svg::net_step`,
  `svg::mell_net_step`, and `net`/`mell_net` for a net with cut links):
  a cut link is a bar `2·node_radius` under its two formulas, joined to
  them from below (Girard's drawing), id `c<x>-<y>` with its end in a cut
  formula's tree first, the order `nets::cut::Step::cut` names it in.
  Outside every box the two ends stand at the bottom after the
  conclusions, side by side (the MLL drawer's `tops`, its dead
  occurrences, those of a cut formula above every cut link, skipped);
  inside a box they are layout children of the box after its premise and
  boxed copies (keys `n + 2i`), on their own rows, which the box's `!`
  stands a row below. The drawing grows by the bar's room only when a
  cut link stands at the bottom. A step's marks (`Marks`, built from the
  step and the net it was taken on) are the cut link, the axiom links at
  its ends (axiom), the premise edges of its ends (`tensor_par`), the box
  of its `!` end (every exponential case), the box entered
  (`box_into_box`) and the wire to the copy acted on (dereliction,
  contraction), all in the highlight group. A net with cut links is
  drawn under the reading with `style.essential` as a cut-free one is
  (the cut formula's tree an output one, its dual's an input one, the
  cut bar joining them): otherwise an intuitionistic elimination's steps
  would write the one-sided net's classical symbols, which no
  intuitionistic output does (`an_intuitionistic_elimination_is_drawn_in_its_syntax`).
  Its highlight is still `is_essential`'s verdict, which for cut links
  is the switching criterion's. `cut_elimination_is_drawn` checks
  every step of three eliminations in both looks and both drawings, and
  pins `net-cut.svg` and `net-cut-boxes.svg`.
- **A net with boxes** (`svg::mell_net`, `export/svg/mell.rs`) is the
  multiplicative layout over a *layout tree*: a vertex's premises, a `!`
  vertex's box (node `n + b`), a `?` vertex's copies at its own depth,
  and a box's premise and its *boxed copies* (copies in the box of a `?`
  outside it), those of a `?` earlier in the formula trees' pre-order to
  the premise's left. So a box's contents are one subtree, its extent
  plus `node_radius` its rectangle, and the separation argument above
  keeps every premise edge out of every box and formula. Compact, that
  needs a `⊗`/`⅋` circle in the gap between its two subtrees' extents,
  not halfway between their anchors as the MLL compact drawing has it:
  there a box of one subtree reached over the circle and the edge to the
  other premise rose through it (the review's finding,
  `edges_keep_out_of_boxes`); and the copies of a `?` with several at its
  own depth on the row above it in both drawings. What it cannot
  keep: a door's wire goes from the box to a `?` vertex in another part
  of the trees, so it may cross edges (and in a crowded net a formula);
  it runs straight down from the door, along `2·node_radius` above its
  target's joint row, then into it, which keeps it off joints and
  circles. Rows are a longest path over what a vertex depends on (its
  layout children, a `!`'s box contents, the boxes a `?`'s wires leave
  last), so a `?` is always a row below its doors; roots go to the
  bottom, except a `!` with auxiliary doors, which stays at its row (its
  `?`s must be below it); the copies of a `?` with several at its own
  depth go to the row above it. A box's lid is its contents'
  arcs plus `node_radius`, a link over boxes of its depth rises by the
  highest lid it spans (a sparse table of the depth's boxes, which lie
  apart), so no arc enters a box it does not belong to. The bottom border
  is broken `node_radius` around the `!` joint and each door. Weakenings
  stand alone (the literal they hang on is in the description), so the
  doors drawn are those of copies (departs from `auxiliary_doors`, which
  counts a weakening's too). Ids: `o<n>` on the only vertex of occurrence
  `n`, `v<k>` where an occurrence has several, `l<x>-<y>` on links by
  vertex, `b<k>` on a box's outline; a switching cycle's pairs (a wire is
  the pair of its `?` and the copy or contracted box it comes from) are
  highlighted, a disconnection is not. Boxes take the `line` colour,
  wires and `?` edges the `par` colour and dash (under
  `Premises::Dashed`). The estimate counts per vertex (an occurrence under a `?` may
  have none or many), per door and per box, and the text form.
- **A net's look is `Style::net_look`** (`svg::NetLook`, three fields,
  each a closed enum; `NetLook::LINLOG` the default, unchanged from
  before it existed, and `NetLook::TEXTBOOK` the preset, both constants
  of the type and `NetPreset` their names, which JSON reads in place of
  the fields as `Font` reads a `MathFont`'s). Each choice follows a figure
  read in the source itself (Laurent's proof-net slides as used in
  Faggian's LMFI 2025 course, slides 42 to 51; Straßburger, *Proof Nets
  and the Identity of Proofs*, INRIA RR 6013, Fig. 10 on p. 45; Guerrieri,
  Manara, Pellissier, Tortora de Falco and Vaux Auclair, TLLA 2021,
  Figs. 1 to 4; Chouquet and Vaux Auclair, LMCS 17:4, Fig. 1):
  - `premises`: `Dashed` (the edges into a `⅋` or `?` and the wires in
    the `par` colour and dashed) is linlog's own, no source does it;
    `Plain` draws them as every other edge, as all four do (Laurent 42
    and 43, Straßburger Fig. 10, TLLA Figs. 1 and 2, Chouquet Fig. 1).
    Plain drops the dashed group altogether, so a plain drawing has no
    `stroke-dasharray` (a highlighted switched edge is not dashed
    either).
  - `conclusions`: `Raised` (a `!` with auxiliary doors written under its
    door, a row above the `?` nodes its wires reach) is linlog's own;
    `Bottom` writes it on the bottom row under an edge down from its door,
    as Chouquet Fig. 1 (the `!` on the border, its wire down beside the
    `?`s' to one line), Laurent 43 (`!A` level with `?Γ`) and Straßburger
    Fig. 10 (`!A, ?B₁ … ?Bₙ` on one row) do; TLLA Fig. 2 ends its
    conclusions at several heights. Only the formulas drawing changes:
    the compact one and every MLL net have their conclusions at the
    bottom already (`net_looks` asserts the compact drawing equal).
  - `why_not`: `Node` (one `?` with every copy and wire a premise, a
    weakening one without) is TLLA Fig. 1 (a `?` vertex of any arity) and
    Chouquet Fig. 1 (one `?` collecting the copies' wires); `Rules` is
    Laurent 42 and Straßburger Fig. 10: `?d` under each copy, binary
    `?c`, `?w`. It is a faithful drawing of the generalised node because
    that node is the class of nets equal up to the associativity and
    commutativity of contraction and its commutation with box borders;
    the drawing picks one member: every dereliction at its copy (in the
    copy's box, on the row below it, which the layout reserves: a box's
    `!` is a row lower for a copy in it), every contraction at the `?`'s
    own depth, so a box has a door per copy, not per `?` (`Doors::of`
    keyed by copy), and the contractions a left comb in a column over
    the `?`'s formula, the copies at its depth first and then the wires,
    each nearest the column first. The doors have no symbol and break no
    border: a wire crosses it, as in Straßburger Fig. 10 and Chouquet
    Fig. 1 (Laurent's `?p` door node is not drawn). A lone wire is the
    conclusion itself and has no node (in the compact drawing no circle,
    so no element has that vertex's id). Edges into a `?c` are its
    switched premises (dashed under `Dashed`); a copy's edge into its
    `?d` and a `?`'s edge down to its formula are not.
- **The rules' rows and wires**: a `?` vertex takes `levels` rows of
  nodes (one for the derelictions at its depth, or with wires alone a
  landing row the first wire comes down through; one per contraction
  past the first premise; one for a weakening), all above its formula
  when written, all but the last when its circle is the last node; its
  layer adds them, so every node is below every box its wires leave.
  Nodes stand at the joint height of their row in the formulas drawing,
  so every wire's leg runs between two rows like the default's (level
  with a node it would pass through other `?`s' nodes on that row: the
  first attempt did). A side premise comes in along twice the radius
  above its contraction; the first wire, with wires alone, along twice
  the radius above the landing row and down the column, so the two
  premises of the first contraction never share a leg. Legs of several
  wires between the same two rows are staggered (`legs`, at most the
  radius apart, within the room below the circles above), the one whose
  door is further along its direction higher, so that where spans
  overlap one wire's way down does not cross the other's leg; and the
  rows are drawn apart enough (`line_height` raised, counted from each
  `?`'s premises before any height is fixed) that the most wires sharing
  a band stand half the radius apart at least: the uniform compact rows
  left six wires 60 units apart against a 40-unit stroke, one bar to the
  eye (the review's finding; `net_looks` measures six). The default
  look staggers the same way: its wires run along twice the radius above
  their `?`'s row, or above the row of the box whose door they go into
  (a wire from an inner box's door to an outer box's door), so a band is
  a row, and the rows are spaced by the most door wires ending on one
  row; before, two wires bound for different `?`s on one row shared a
  leg and read as one (`net-promotion.svg`, `!a, !b, !(c * d) ⊢ !((a ⊗
  a) ⊗ (d ⊗ c))`), and so did door-to-door wires (`!a, !b ⊢ !(!a ⊗ !b ⊗
  !a ⊗ !b)`, the integers panel's witness; `net_looks` measures both, in
  both looks where they apply). With formulas the default's legs keep
  within `line_height − 2.5·radius − stroke` above the band, below the
  text of the row above (the joint is 1.5 radii above its text's top);
  the rules' room there, `pitch − 3·radius`, reached into that text when
  first reused. What no test catches: a
  leg through another formula or a wire across an edge (accepted, as
  for the default's wires); look at the renderings.
- **The estimate under the rules** adds `2 · RULE` per copy (its `?d` and
  the `?c` it goes into) and `RULE` per `?` (its last node or weakening),
  `RULE` being a node's label, circle and edge; the default's slack
  (about 3.2 times the drawing on `net-promotion`) leaves little room, so
  `CONNECTIVE` per node broke the within-four-times bound.
- **An essential net is the proof net's layout with the reading drawn
  in** (`Style::essential`, off by default; `input` and `dominator` its
  colours): every edge an arrowhead halfway along it (half the node
  radius long, in the edge's group), the input occurrences, their texts
  and their edges in `input`, every formula written as the reading
  writes it (`Notation::ill`: literals as their atoms, `A ⊸ B`, `A \ B`),
  the links `⊸` (an output `⅋`, an input `⊗`, a division too: a `\` or
  `/` at a joint reads as a piece of an edge, and the text below tells
  the side) or `⊗`, the antecedent's edge dotted in `dominator` (no edge
  of the
  net: the implication must dominate it), each link's arrowhead at the
  top of its arc toward its output literal, and in the compact drawing
  the hypotheses' hanging edges pointing up and the goal's down (the
  drawing with its formulas has no hanging edges: the colours tell the
  hypotheses from the goal). The verdict drawn is
  `is_essential`'s (a directed cycle, or the path around an implication
  plus that implication's dotted line, highlighted); the title is the
  sequent two-sided and the description the net's text form under the
  reading (`written_in`), so an essential drawing has no classical
  symbol; the front ends set `essential` in an intuitionistic mode, since
  the plain drawing writes the one-sided formulas. A sequent without an intuitionistic
  reading is drawn as without the option, so one settings file serves
  classical and intuitionistic nets. The estimate adds an arrowhead's
  bytes per edge, root and link and a document's for the two extra
  groups.
- **An intuitionistic net with boxes under `Style::essential`** is the
  MELL layout with the reading drawn in as above (`svg::mell_net` passes
  the structure's reading): the reading's formulas (a hypothesis `!A` for
  the input `?A⊥`, written with `!` at its nodes and doors), input
  vertices, their circles, texts and edges, every wire and every node of
  the rules in `input`, an arrowhead on every edge toward the goal (into
  a `?`'s copies upward, along each wire toward its copy: on a wire's
  longer of its way down from the door and its leg, under the rules on
  its way down from the door), the edge into an output implication's
  antecedent dotted in `dominator`, the rules' labels `!d`, `!c`, `!w`;
  `dashed` is off, as in the MLL essential drawing. The highlight is the
  switching cycle of `is_correct`, which decides the structure: there is
  no essential criterion with boxes. The estimate adds an arrowhead's
  bytes per vertex, root, door, link and rule node, the input colour per
  text piece, no raised `⊥` pieces (the reading writes atoms plain), and
  the two groups by their size: a flat `DOCUMENT` for them broke the
  four-times bound of the textbook look, whose default estimate is
  already 3.75 times its drawing (`essential_nets`, five snapshots
  `essential-boxes*`).
- **A net's drawing has a size bound like a derivation's**: `svg::net`
  estimates its bytes from the structure (`net::estimate`: every written
  formula's text, counted bottom-up from its subformulas' with a
  `PIECE` per `<text>` (a negated literal's raised `⊥` adds two where
  the reading does not write the atom alone), fixed bytes per
  connective, conclusion and link, and the description counted by
  writing the net's text form into a counter that gives up past the
  limit) and refuses past `limit`; the estimate is at least the drawing
  (`net_limit` checks it within four times, on a small net and a chain
  twelve deep, in each drawing). A formula is written once per
  subformula, so the drawing with formulas grows with the conclusions'
  size times their depth, quadratically for a chain: the bound refuses
  it where the compact drawing still fits.
- The SVG tree asks `stop` after laying out each inference in its first
  pass too (before the head is written), not only while it writes.
- **Dependency versions**: krilla-svg pins usvg 0.47, so resvg stays at
  0.47 with it: one usvg tree serves both, and `deny.toml` ignores the
  unmaintained rustybuzz and ttf-parser beneath them until krilla moves
  on.
- **A superscript or subscript is its own `<text>`**, never a `<tspan>`
  with `dy`: resvg (which Typst uses to draw SVG images) spreads
  `textLength` wrongly across such a tspan, while every renderer agrees
  on separate positioned texts. Spaces separate pieces instead of
  starting or ending one. The font has no `₁`/`₂`, so rule names'
  subscripts are lowered digits.
- The text bounds are `font::HEIGHT` (a raised `⊥`) above and
  `font::DEPTH` (a comma) below every baseline; the layouts reserve
  them for every line, and the structural test in `core/tests/export.rs`
  checks every element against the view box with the same bounds.
- **Rocq's two kernels** (`export/rocq/mod.rs`, `rocq::Kernel`):
  `Options::kernel` names one, `None` (the default) lets the mode
  choose in `linear`: a recorded mode that is intuitionistic, affine or
  Mix goes to linlog's library (`Linlog`), a classical one to NanoYalla
  unless the derivation uses Mix or affine `wk` (a proof checked in
  another mode than it claims), and a proof that records no mode
  (`Derivation::mode`, set from `Proof::mode` and by `Interactive`) goes
  to linlog's library when its derivation is two-sided or uses those
  rules; a derivation with a cut goes to linlog's library whatever its
  mode, and an explicit `NanoYalla` refuses it (`Unsupported::Cut`:
  NanoYalla's cut is an axiom or Yalla's theorem, no rule of its
  kernel). So a classical proof with no kernel named is written for
  NanoYalla exactly as before, which the NanoYalla snapshots pin (they
  name the kernel, since by default `ill.v` and `labels_ill.v` would go
  to the library), and an explicit `NanoYalla` keeps every refusal.
- **linlog's kernel** (`export/rocq/linlog.rs`, the library `rocq/`,
  `rocq.md`): a one-sided derivation is written as the library's dyadic
  `proof`, a two-sided one as its `iproof`, under a lemma over `ll` or
  `ill` with `(atom := string)` and the atoms as Rocq strings (`"`
  doubled), so no atom is renamed. The one-sided term moves every `?`
  formula into the unrestricted zone with a `Quest` where it first
  appears (the root's members, and a premise's members that are children
  of the rule's principal formula): then a dereliction is `Copy`, a
  contraction and a `?` weakening write nothing, a promotion's `?`
  context is in the zone as `Bang` requires, and an affine `wk` of a `?`
  formula writes nothing either. The checker accepts that term although
  it is not the Rust proof's own term (the Rocq checker accepts every
  term that keeps its zones consistent, `rocq.md`). The statement's
  calculus is what the mode allows and what the derivation uses together
  (`linlog::Calculus`, `cut` among its fields, set by a cut inference),
  so a certificate never claims less than the proof needs (a proof with
  a cut states the calculus with `cut := true`, through `soundness` with
  the record literal); a cut's premises get a `Quest` for a `?` cut
  formula (the first) or a `?` dual (the second), as a principal
  formula's children do; one-sided the cut is `Cut A`, `A` as the first
  premise holds it, two-sided `ICut A`, `A` as the premise that proves it
  holds it, the premises in the derivation's order (the library's arm
  takes either order); `soundness_classical`, `soundness_mix`, `soundness_affine`
  or `soundness` with the record for Mix and weakening together. The
  two-sided term writes each formula by its occurrence's position under
  the reading (`Reading::position`): a `top` in output position is `⊤`
  (`itop`), in input position `0` (`izero`), and so on; `⊤R` names its
  context and `0L` its context and goal from the inference's sequent,
  which the derivation view has instantiated. The lemma's goal is the
  reading's goal (by id for roots built from `⊤` and `0` alone). Open
  goals and compact runs are refused as for NanoYalla; nothing else is.
  The core tests pin this kernel's certificates in
  `core/tests/snapshots/linlog/`, which the flake's `rocq-linlog` check
  compiles, and `rocq::LINLOG` is asserted equal to the opam file's
  version.
- **Certificates of non-derivability** (`rocq::write_disproof`,
  `linlog::disproof`): for linlog's library whatever the mode
  (`Unsupported::Disproof` where the options name NanoYalla), the lemma
  `~ ll (atom := string) R [Γ]`, or `~ ill … R [Γ] C` in intuitionistic
  mode with the reading's goal as for proofs, `R` the mode's rules alone
  (a disproof records no rules used), proved by `apply
  (THEOREM …). vm_compute. reflexivity.` with `classical_refutation _
  (fun a => existsb (String.eqb a) [true atoms])`,
  `unbalanced_refutation String.eqb "a" _` or `equation_refutation _`
  (`i…` two-sided). A saturated set (`Refutation::Saturated`) is refused
  first (`Unsupported::Saturated`): the library has no theorem for it,
  though `Disproof::check` accepts it. It runs `Disproof::check` first
  and returns its error, so a refutation that rests on the search
  (`RefutationFault::Uncertified`) or that does not hold has no
  certificate: the library's tests and the Rust check accept the same
  refutations under the same conditions, and a certificate is written
  exactly when it compiles. The formula printers (`atom`, `formula`,
  `iformula`) are free functions over the forest that the proof writer
  calls too. The goal of a disproof off the roots is its members. The
  command writes one for `--format rocq` on an unprovable verdict
  (`disproof_into` in `cli/src/show.rs`), and a line `no certificate:
  …` on standard error where there is none, the verdict unchanged.
  Snapshots `core/tests/snapshots/linlog/refuted_*.v`; the
  `rocq-linlog` check compiles them and four the command writes.
- **Not valid over `Prop`** (`rocq::write_not_valid`,
  `ordinary::rocq::not_valid`): from an ordinary sequent's image and its
  disproof with `Refutation::Classical`, the atoms valued by name in the
  image (an atom it lacks is false) and the ordinary sequent evaluated
  over its arena in order (each node after its subformulas): only an
  assignment that makes every hypothesis true and every formula right of
  `⊢` false is written, as `~ (forall a b : Prop, Γ -> Δ)` with `Γ` and
  `Δ` as the positive certificate states them, proved by `intro H.
  specialize (H True False …). tauto.`, in LK and LJ alike; otherwise
  `Unsupported::NotValid`. Snapshots `not_valid_lk.v` and
  `not_valid_lj.v` (compiled by the `rocq` check, beside two the
  command writes).
- **NanoYalla** (`export/rocq/mod.rs`): the kernel is NanoYalla `NANOYALLA`
  (Click & coLLecT's `nanoyalla/`: `nanoll.v` is the trusted `ll`
  inductive over list sequents, `macroll.v` the derived rules), and the
  script relies on its `_ext` lemmas exactly as stated there: every rule
  takes the list `l1` of formulas before its principal one and infers the
  rest by unification, `oc_r_ext l1 (A) l2` needs both contexts without
  their `?`, `tens_r_ext l1 A B l2 : ll (l1 ++ A :: nil) -> ll (B :: l2)
  -> ll (l1 ++ tens A B :: l2)` needs the left premise's context before
  the `⊗` and the right one's after it, `ax_expansion` closes `[dual A;
  A]` and `[A; dual A]` for any formula `A` (atoms are `formula`
  binders of the lemma, so the lemma is schematic), and `ex_perm_r p l`
  proves the goal whose position `i` holds `l[p[i]]` from `ll l`. So the
  exporter tracks the goal list of every inference (`Script::goals`, set
  when the conclusion is written; the root is the sequent in id order)
  and emits one `ex_perm_r` only before a `⊗` whose goal is not already
  split around it; every other rule acts in place, and a contraction
  leaves its two copies adjacent. Equal ids are equal formulas, so the
  first matching position serves for a repeated occurrence. The
  certificate is classical: a two-sided derivation goes through the
  one-sided `Rule` of every `Named` rule, and the checked sequent is the
  one-sided one. Mix,
  affine `wk` and `Rule::Open` are refused before anything is written
  (`Unsupported`), since the kernel has no such rule; a derivation or a
  disproof of an ordered mode is refused by both kernels
  (`Unsupported::Ordered`): their calculi are commutative, and a
  commutative certificate would state another sequent's derivability
  (a cyclic proof is a commutative one, but the certificate would not say
  the order was kept). The cyclic snapshots (`cyclic.tex`, `.typ`,
  `.svg`, the planar net `planar.svg`) are drawn as any other: the view
  has the cyclic order, and the net SVG puts the conclusions in their
  order with links above, so a planar net is drawn without a crossing.
  The Lambek snapshots (`lambek.tex`, `.typ`, `.svg`, and the planar
  essential net `lambek-net.svg`) are two-sided with the divisions:
  every notation has `under` and `over` (LaTeX `\mathbin{\backslash}`
  and `\mathbin{/}`, Typst `class("binary", backslash)` and `slash`,
  since a bare `/` is a fraction there, text and SVG `\` and `/`), and
  `\` and `/` are label symbols (`\L`, `/R`), set `\backslash` and `/`
  in LaTeX, `backslash` and `slash` in Typst; a user's label with either
  character sets it as a symbol now;
  atom names are
  escaped to identifiers and made distinct from `RESERVED` (keywords and
  every kernel name a script mentions), the lemma's name and each other,
  in one pass: the first clash of an identifier gets `'`, the later ones
  `'2`, `'3` (a counter per identifier; escaping is not injective, `ä`
  and `_e4_` are one identifier, and a prime more per clash made 2¹⁴
  such names take two minutes, F3).
  `Options` (D15: `form`, `lemma`, `prelude`, non-exhaustive with
  `with_*`) is the configuration; no other choice is a constant.
  `lemma` is an `Identifier`, checked for its lexical form (an ASCII
  letter or `_`, then letters, digits, `_`, `'`) and against Rocq's
  `KEYWORDS` when it is made or read (`Error::InvalidOption`, key
  `rocq.lemma`, F38); the names a kernel uses (`KERNEL`) only keep atoms
  apart, so a lemma's name stays valid whatever kernel a later release
  adds. `prelude` is `None` for each certificate's own
  (`NANOYALLA_PRELUDE` for a linear one, `CLASSICAL_PRELUDE` for a
  classical one over `Prop`, nothing for LJ) and a given text for every
  kind (F15). The snapshots' `.v` files are compiled by the
  flake's `rocq` check against the kernel built from the `nanoyalla`
  input; the kernel needs Rocq 9 with `rocq-stdlib` (its `From Coq
  Require Import Lia`, deprecated but accepted) and nothing of Yalla.

## Quantifier rules (step 42b)

- `Rule::Forall` and `Rule::Exists` follow `Cut` in `Rule::ALL` (the
  one-sided rules keep `index() == rule as usize`, which `Named::index`
  now takes directly), and `Named::ALL` gains them and `∀R`, `∃L`, `∃R`,
  `∀L` before `open`; the label tables follow. The LaTeX and Typst label
  symbols map `∀` and `∃`. `every_label` pins the first-order
  derivations `first_order_swap` (one-sided) and `first_order_swap_ill`
  (two-sided), read from `core/tests/first-order/`.
- The Rocq export still refuses a sequent with binders; its arms for the
  quantifier rules are unreachable behind that guard.

