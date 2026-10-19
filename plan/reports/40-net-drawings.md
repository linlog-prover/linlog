# Step 40 report: proof nets drawn with their formulas

## The checklist

| item | state | evidence |
|---|---|---|
| 1. Every formula occurrence written, by default | done | `NetDrawing::Formulas`, the default of `svg::Style::net`: every subformula occurrence is its formula's text, each `⊗`/`⅋` link joins its premises to its conclusion below them with the connective small at the joint, axiom links are arcs over the top, the conclusions at the bottom in the sequent's order; D12's layout (deterministic, integer coordinates, Euler Math widths, `textLength` on every text). Rendered and read: `net.png`, `cycle.png`, `disconnected.png`, `mix.png`, `deep.png`. |
| 2. The compact drawing as an option | done | `NetDrawing::Compact` (`svg::Style::net`, serde `"compact"`, `--style svg.net=compact`, `Style::with_net`, a line in `--help`, README and the changelog): the literals along the top, inner links as circled connectives, the conclusions written at the bottom. Snapshot `net-compact.svg`; the CLI's style test has its row. |
| 3. Every net drawing at once | done | Classical, essential (`essential.svg`, `essential-around.svg`), cyclic (`planar.svg`), Lambek (`lambek-net.svg`), with Mix (`mix.png`), all through the one `svg::net`, which `--format svg --net`, the PNG and PDF of a net and the web bindings' `draw_net` call. Ids `o<n>` (each occurrence's formula; a compact net's circle) and `l<m>-<n>` kept; no id twice (the reviewer checked every drawing by script). |
| 4. Snapshots, README, changelog | done | Snapshots in a commit of their own with the behaviour lock's net SVG and help texts, the drawing's pinned outputs and nothing else ("Redraw the net snapshots and the behaviour lock with their formulas"), each snapshot rendered with resvg and Euler Math and read (paths below); README's net paragraph and examples, the "what exists" lines and the changelog's `[Unreleased]` (Added and Changed) say what the drawing shows. |
| 5. The web client | named | What its net view and Settings take is below; the client is not changed. |
| Review | done | A fresh-context reviewer (Opus 5.5, `high`) read all fourteen rendered drawings and their SVGs against items 1–3: one finding, answered below; nothing changed for it. |
| Verification | done, one commit red by construction | `gate` on every code commit: the pinned outputs' commit (`mzstxyrl`) and the docs' (`popstnvp`) pass (exit 0, ratchet included); the drawing's commit (`wuvqoxlq`) passes clippy and fails exactly the comparisons with the pinned outputs that the next commit holds (below); `nix flake check` at the session's last commit (its exit status in the message to `planning`). |

## Outcome

A proof net drawn by linlog now shows its formulas, as Girard's paper and
the lecture notes after it draw nets. Every subformula occurrence of the
conclusions is written as its formula, at the place it has in its
conclusion's text; the literals stand along the top under the axiom
links' arcs; each `⊗` or `⅋` link joins its two premises to its
conclusion written below them, with the connective small at the joint
where the two edges meet (the `⅋` edges dashed and blue as before); and
the sequent's formulas close the drawing at the bottom in its order. A
literal that is itself a conclusion stands at the bottom too, and its
link's arc runs down to it. The essential net writes every formula as
the intuitionistic reading reads it (`a ⊸ b`, `c / b`, literals as atoms),
keeps the arrows, the green input side, the dotted line from an output
implication to its antecedent and the highlighted witness. The planar
nets of the cyclic mode and of the Lambek calculus are drawn the same
way, without crossings. The compact drawing of before is an option, now
writing the conclusions under its circles.

The rendered drawings (resvg with Euler Math, zoom 3; each SVG beside
its PNG):

- `/tmp/linlog-handoff/step40a/net.png` and `net-compact.png`: `⊢ A⊥ ⅋ B⊥, B ⊗ A`, both drawings.
- `/tmp/linlog-handoff/step40a/cycle.png`, `disconnected.png`: a switching cycle and a disconnected structure, highlighted.
- `/tmp/linlog-handoff/step40a/mix.png`, `mix-compact.png`: a net with Mix, `⊢ a, a⊥, b ⅋ b⊥`.
- `/tmp/linlog-handoff/step40a/deep.png`, `deep-compact.png`: `((a⊗b)⊗c)⊗d ⊢ a⊗(b⊗(c⊗d))`.
- `/tmp/linlog-handoff/step40a/essential.png`, `essential-around.png`, `essential-compact.png`: essential nets.
- `/tmp/linlog-handoff/step40a/planar.png`: the cyclic mode's planar net.
- `/tmp/linlog-handoff/step40a/lambek-net.png`, `lambek-compact.png`: the Lambek calculus's planar essential net.

## Decisions

- **Each subformula is drawn where it stands in its conclusion's text.**
  The layout is one pass up the forest and one down. A connective's text
  is placed with its own main connective under the link's joint, and the
  subtree of each premise is placed as far to that side of the joint as
  the premise stands in the text (half the connective, a space, and a
  bracket when the premise is binary). In a one-sided net the subtrees'
  boxes then coincide with the texts, so every subformula is exactly
  where it is in its conclusion and the drawing is as wide as the
  sequent's line. The same rule holds for the essential net's formulas,
  whose written order can differ from the structure's (the Lambek
  calculus writes the operands of an input product the other way round);
  there the boxes are only kept apart, not aligned. Why this rule: since
  the subtrees of a link's two premises lie on either side of its joint,
  and every node is below its whole subtree, no edge crosses a formula,
  not even a long edge from a deep conclusion up to a literal. A layout
  that centred each text under its premises puts the edge to a literal
  through its sibling's bracketed text from depth three on (worked
  through for `a ⊗ (b ⊗ (c ⊗ d))` before writing the code).
- **The literals stay on one row along the top**, as D12 and the old
  drawing have them, so the arcs and their non-crossing argument
  (`net::height`) are unchanged; a connective is one layer below the
  lower of its premises, and the conclusions on the bottom layer.
- **A literal that is a conclusion stands at the bottom**, its link's
  path running down to it (`M… V… A… V…`, one path, the same id), so all
  the conclusions are at the bottom in order and no occurrence is written
  twice; this holds in both drawings.
- **Rows** are `line_height + node_radius + HEIGHT + DEPTH` apart (2.98 em
  by default), the joint `1.5 · node_radius` above its text's top, the
  edges stopping `node_radius` from it. `line_height` and `node_radius`
  keep their fields; their documentation says what they mean in this
  drawing. No new spacing field: the two existing ones place everything.
- **An essential net's joint shows `⊸` for every implication, the
  divisions included**, and `⊗` for a product. The first rendering wrote
  `\` and `/` at the joints of the Lambek net, where they read as pieces
  of the edges; the conclusion's text below says which division it is.
- **The order at the bottom is the structure's, one-sided** (the title's
  order). In the Lambek calculus the hypotheses therefore stand reversed
  against the two-sided sequent (`a \ b, a, c / b, c` for
  `c / b, a, a \ b ⊢ c`), as the old drawing's literals already did: the
  planar layout needs the cyclic one-sided order.
- **The drawing with formulas has no edges hanging below the
  conclusions**: the conclusion is the text right under its joint. The
  essential net's hypotheses and goal are told apart by the colour, and
  the compact drawing keeps its hanging edges with their arrows.
- **Compact is the old drawing with the conclusions written**: the
  literals packed `literal_gap` apart, each circle halfway between its
  premises' anchors, and the conclusions spaced by their texts and
  written under the hanging edges. Its compound conclusions' texts carry
  no id, since the circle has `o<n>`.
- **The size bound counts every written formula**: a formula is written
  once per subformula, so the drawing with formulas grows with the
  conclusions' size times their depth (quadratically for a chain). The
  estimate counts each text bottom-up, with a `<text>` per piece, two
  more per raised `⊥` where the reading does not write the atom alone.
  `net_limit` now checks that the estimate is at least the drawing and
  within four times of it, for a small net and a chain twelve deep, in
  each drawing. A first version counted the raised `⊥` in the essential
  net too, which is 4.15 times the drawing on the small net: fixed by
  telling the estimate whether the reading draws it.
- **Commits**: the code with its tests (`wuvqoxlq`); the pinned outputs
  on their own, the snapshots and the behaviour lock's files, which the
  drawing changes together (`mzstxyrl`; a first run had the lock in a
  commit of its own after the docs, which left two red commits, so it
  was folded into the snapshots' commit); the docs (`popstnvp`); the
  report. The supervisor moved the stack onto `main`'s head
  `trppoyyspmku`, which adds plan files only. **Every code commit green
  is impossible here**: the step asks for the snapshots in a commit of
  their own, and the drawing changes them, so the commit of the drawing
  alone fails their comparison. Gated at `wuvqoxlq`, clippy passes and
  `cargo test --workspace --no-fail-fast` fails exactly the four
  snapshot tests of `core/tests/export.rs` (`nets`, `cyclic`, `lambek`,
  `essential_nets`, at the snapshot comparison) and `behaviour_lock`
  (`format-net-svg` and four help texts), every file of which
  `mzstxyrl` holds; the gate passes at `mzstxyrl` and at `popstnvp`, and
  the code of the three is the same.
- **Two tests changed where their pinned behaviour was meant to
  change**: `svg_structure` counts 13 texts, 6 paths and no circle for
  `⊢ A⊥ ⅋ B⊥, B ⊗ A` by default and 13, 8, 2 compact (was 8, 8, 2), and
  the command's `svg_formats` looks for the root's `<g id="o0">` instead
  of `<circle id="o0"`.

## Options (D15, D16, D17)

- `svg::Style::net: NetDrawing`, `Formulas` (the default) or `Compact`,
  `#[non_exhaustive]` with serde `"formulas"`/`"compact"`, built with
  `Style::with_net`. The command: `--style svg.net=compact` or
  `{"svg": {"net": "compact"}}` in a style or settings file; the web
  front end: `styles.svg.net` in the settings JSON it passes to
  `draw_net`, `export` and the renderers. A public item changed only by
  addition (the field, its builder, the enum).
- The quantifiers (D17): a quantifier link has one premise; in this
  layout it would be a joint straight above its conclusion's connective
  with the premise's subtree above it, the separation rule taking one
  side only. Nothing of it is built.

## What the web client should take

- **Settings**: a choice "Net drawing: formulas / compact" mapped to
  `styles.svg.net` (`"formulas"` by default), in `drawSettings` and in
  the export settings, so the net view and the downloaded SVG, PNG and
  PDF agree.
- **The net view** is larger now: as wide as the sequent's line and
  about three ems per layer (the old drawing was 1.5 ems per layer and
  as wide as the literals). `fitView` fits a fresh drawing; a wider
  default zoom-out for deep nets may be wanted.
- **Colours**: the page's CSS paints every `text` with its ink colour
  and every `path`, `circle` and stroked group with its ink stroke
  (`.drawing svg text`, `.drawing svg path`), which removes the
  essential net's green input side, now on whole formulas. This predates
  the step; the page should exempt the `fill`/`stroke` the drawing sets
  for `svg.input`, `svg.dominator` and `svg.highlight`, as it does for
  the `⅋` colour.
- Nothing in the page reads the net's ids today (`decorate` returns
  early for a net), so nothing of it breaks.

## The review

A fresh-context reviewer (Opus 5.5 at `high`), briefed with the machine
rules and told to report only what breaks a stated requirement, read the
fourteen rendered PNGs and their SVGs (and zoomed crops of three) against
items 1–3, the ids and the highlights.

- Requirement 1 (the default drawing), 2 (compact), 4 (ids, none twice)
  and 5 (highlights): hold, checked by geometry on the long diagonals of
  `deep` and `planar` and the drops in `disconnected` and `planar`; the
  `⅋` joint's edges stop at the glyph's outline (zoom 12).
- Finding: in the compact essential drawings the `⊸` inside the circle
  of an input occurrence is black while its circle, edge and conclusion
  are green (`essential-compact.svg`, `lambek-compact.svg`). **Answer:
  not changed.** The compact drawing is the old one by item 2 ("the
  inner links as today's circled connectives"), whose symbols were black
  in green circles before this step (the old `essential.svg`); the
  polarity is on the circle, which is the occurrence's element. The
  drawing with formulas colours the joint's connective with its text.

## What is left open

- The client's CSS overriding the drawing's colours (above).
- The drawing with formulas grows with depth; the bound refuses a deep
  net's drawing where the compact one still fits. The command does not
  fall back to the compact drawing by itself; a user asks for it.
- usvg's precision fault on arcs far from the origin (`core-export.md`)
  applies as before; the drawing with formulas is wider, so it starts at
  fewer literals.
