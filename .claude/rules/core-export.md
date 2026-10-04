---
paths:
  - "core/src/export/**"
  - "core/src/proofs/style.rs"
  - "core/tests/export.rs"
  - "core/tests/snapshots/**"
---

# linlog core: the exports

Loaded, beside `core.md`, when a file of the exports, the rule labels
or the export snapshots is read. The text tree is the derivation
view's (`core-derivations.md`).

## Export

`export/` writes sequents and derivations as LaTeX (ebproof trees, cmll and
amssymb symbols) and Typst (curryst trees), draws them and proof
structures as SVG documents, and writes derivations as Rocq proof scripts
for NanoYalla. What the code relies on:
- **A new export option is a field, never a constant.** Every output
  has one plain-data options value with `Default`, `Clone`, `PartialEq`
  and serde behind `serialize` (`serde(default, deny_unknown_fields)`,
  so a JSON with some fields is the defaults with those changed and a
  misspelt field is an error): `TextOptions` (`proofs/fmt.rs`),
  `latex::Options`, `typst::Options`, `svg::Style`, `rocq::Options`; the
  `Form` is a field of the three that have one. Presets are named
  values (`Style::dark()`, `Style::monospace()`, `Font::monospace()`).
  The defaults reproduce the output the snapshots pin, so a new field's
  default is today's behaviour.
- **One signature writes a derivation**: `latex::write`, `typst::write`,
  `svg::write`, `rocq::write` and `Derivation::write_text` take the
  derivation, the options, any `fmt::Write` and a stop closure, and
  answer `WriteError` (`Stopped`, `Failed`, `Unsupported` for Rocq,
  which refuses before it writes anything). The emitters make one
  inference in a buffer and hand it on (`notation::flush`, which asks
  the stop after each), so they hold one inference's text; the SVG tree
  keeps a few numbers per inference and lays a conclusion out again
  when it writes it (a laid-out run per inference was 50 bytes per
  character of sequent). `derivation(…) -> String` is the same with a
  string and no stop. The command writes the verdict and then the
  derivation into its output as it is made (`cli/src/io.rs`, `Output`).
- **Rule labels are one table per convention** (`proofs/style.rs`:
  `UPRIGHT`, `SUBSCRIPT`, indexed by `rule as usize` in the order of
  `Rule::ALL`, plus the user's `Labels::Table`), written in a markup
  that each target sets its own way (`parts`: symbols `⊗⅋&⊕⊸!?⊤⊥01`,
  `_x`/`_{xy}` subscripts, `*` a superscript star where the target sets
  one (the mark of a run), other text upright; `latex::label`,
  `typst::label`, `svg::label`, `style::plain` for text). The upright
  table read as plain text is `Rule::name` exactly, and
  `Rule::from_str(rule.name())` is the rule, for every rule
  (`names_round_trip`): the interactive JSON depends on both. A new rule
  is a new entry in `Rule::ALL`, both tables, `name` and `from_str`.
- **One table per target, one printer.** `notation::Notation` is the
  symbol table (connectives, units, dual mark, turnstile, the alignment
  mark, the atom escaper); `Notation::term` and `Notation::ill` are the
  bracketing of `Sequent`'s and `Reading`'s `Display` over it, and must
  stay in step with them. A new target is a new table. `Notation::sequent`
  with `marks` puts `\u{2}`/`\u{3}` around every formula, which the SVG
  layout turns into a group per formula (`Style::ids`: `i<n>-<p>` for
  position `p` of inference `n`'s sequent, the position being the one
  `Interactive::apply` takes, in the drawn order hypotheses first).
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
  `label_gap`, `band`, `stroke`) are Typst lengths written verbatim, as
  `import` and `page` are; their defaults are curryst's.
- **Snapshots**: `core/tests/export.rs` pins standalone documents in
  `core/tests/snapshots/` (`BLESS=1` rewrites them); the flake's `export`
  check compiles exactly those files plus two CLI outputs with pdfLaTeX
  and Typst, which is what catches output that matches its snapshot but
  does not compile. The crane source keeps that directory
  (`modules/workspace.nix`), since `commonCargoSources` alone drops it.
  `typst::CURRYST` and the nixpkgs curryst in `modules/export.nix` move
  together.
- **No font in LaTeX and Typst, Euler in SVG.** The LaTeX and Typst
  output never chooses a font, standalone or not: it is pasted into a
  document and takes that document's fonts. Only SVG names one
  (`svg::Font`, Euler Math by default). The export check runs Typst with
  `--ignore-system-fonts` (its embedded fonts serve) and resvg with
  `--skip-system-fonts` and Euler Math alone, and fails on any output,
  so a font resvg cannot find fails instead of falling back silently.
- **SVG** (`export/svg/`): a third table (`NOTATION`, with the atom
  letters as mathematical italic codepoints, which a math font sets as
  math italic, and `\u{1}` standing for the raised `⊥`; `PLAIN` for the
  `<title>`), and layouts of its own: `tree.rs` (a post-order pass over
  `walk`'s exits for box widths, a pre-order pass over its enters for
  positions; uniform rows of `line_height`), `net.rs` (literals in id
  order, which is left to right; connectives by height; links as
  half-ellipses whose height is proportional to their width up to
  `Style::link_cap` and to its square root beyond, so nested links never
  cross: `net::height` says why). Widths are integer thousandths of an em from
  `font.rs`'s advance table of Euler Math 0.75 (a fixed fallback outside
  it); all coordinates are integers, so the output is byte-stable.
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
  `RenderError::NoDate` (every PDF/A part requires a date; the command
  takes `SOURCE_DATE_EPOCH` or the clock, `pdf::Date::from_unix`).
  Calling `load_system_fonts`, or turning those resvg features on, ends
  it. A text whose font is missing is dropped by usvg without an error,
  which is why the fonts are an argument and the command embeds Euler
  Math (`cli/fonts/`, with its OFL). krilla-svg switches krilla's
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
  `Description` as iTXt; a pixel bound (`png::Options::pixels`)
  refuses before the image is allocated, which is up to eight bytes a
  pixel, but after the SVG is parsed. **Nothing bounds the renderers
  yet**: usvg's parse takes some 80 bytes per byte of SVG (it sets every
  glyph as a path), krilla's PDF 90 to 145, so a derivation the default
  `ViewOptions` admits (an SVG of 52 MB) took over 4 GiB as a PDF; and
  usvg strokes every wide arc of a net to bound it, 17 s for a net of
  6 000 links (4.4 MB of SVG), which no stop reaches.
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
- **Rocq** (`export/rocq.rs`): the kernel is NanoYalla `NANOYALLA`
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
  certificate is classical: a two-sided derivation goes through
  `Rule::classical`, and the checked sequent is the one-sided one. Mix,
  affine `wk` and `Rule::Open` are refused before anything is written
  (`Unsupported`), since the kernel has no such rule; atom names are
  escaped to identifiers and made distinct from `RESERVED` (keywords and
  every kernel name a script mentions), the lemma's name and each other.
  `Options` (D15: `lemma`, `prelude`) is the configuration; no other
  choice is a constant. The snapshots' `.v` files are compiled by the
  flake's `rocq` check against the kernel built from the `nanoyalla`
  input; the kernel needs Rocq 9 with `rocq-stdlib` (its `From Coq
  Require Import Lia`, deprecated but accepted) and nothing of Yalla.
