# Output formats

`linlog seq` prints a sequent one-sided in negation normal form, or
two-sided as intuitionistic linear logic reads it, converts it to JSON
(which a `.json` file or `--input-format json` reads back; with `-i` the
intuitionistic sequent's own form, whose formulas have the connectives
of ILL alone), or names its fragment:

```console
$ linlog seq print "A * B -o C |- A -o B -o C"
⊢ (A ⊗ B) ⊗ ~C, ~A ⅋ (~B ⅋ C)
$ linlog seq print -i "A * B -o C |- A -o B -o C"
(A ⊗ B) ⊸ C ⊢ A ⊸ (B ⊸ C)
$ linlog seq fragment -i "A & B |- 1"
IMALL
$ linlog seq json "A |- A"
{"version":1,"terms":[{"D":0},{"V":0}],"roots":[0,1],"atoms":["A"],"antecedents":1}
$ linlog seq json -i "A * B -o C |- A -o B -o C"
{"version":5,"terms":[{"V":0},{"V":1},{"⊗":[0,1]},{"V":2},{"⊸":[2,3]},{"⊸":[1,3]},{"⊸":[0,5]}],"atoms":["A","B","C"],"hypotheses":[4],"goal":6}
$ linlog seq fragment "A & B |- 1"
MALL
```

`--format latex` and `--format typst` write the derivation of `prove` and
`check` as a proof tree to paste into a paper or slides: LaTeX for the
[ebproof](https://ctan.org/pkg/ebproof) package with the connectives of
[cmll](https://ctan.org/pkg/cmll) and amssymb (turnstiles aligned in
two-sided trees), Typst for the
[curryst](https://typst.app/universe/package/curryst) package. The verdict
line becomes a comment. `seq print` takes the same formats for a sequent:

```console
$ linlog prove -i --format latex "A, A -o B |- B"
% provable (IMLL, intuitionistic, two-sided engine)
\begin{prooftree}
\infer0[$\mathrm{ax}$]{A &\vdash A}
\infer0[$\mathrm{ax}$]{B &\vdash B}
\infer2[$\multimap\mathrm{L}$]{A, A \multimap B &\vdash B}
\end{prooftree}
$ linlog prove --format typst "A & B |- A + B"
// provable (ALL, classical, additive engine)
#prooftree(
  rule(
    name: $⊕_1$,
    rule(
      name: $⊕_1$,
      rule(name: $"ax"$, $⊢ A^⊥, A$),
      $⊢ A^⊥, A ⊕ B$,
    ),
    $⊢ A^⊥ ⊕ B^⊥, A ⊕ B$,
  ),
)
$ linlog seq print -i --format latex "A * B -o C |- A -o B -o C"
$(A \otimes B) \multimap C \vdash A \multimap (B \multimap C)$
$ linlog seq print --format typst "A * B -o C |- A -o B -o C"
$⊢ (A ⊗ B) ⊗ C^⊥, A^⊥ ⅋ (B^⊥ ⅋ C)$
```

`--standalone` writes a document that compiles on its own instead, cropped
to the tree. Neither form chooses a font: the output takes the fonts of
the document it goes into. `pdflatex` needs the ebproof, cmll, amsfonts
and standalone packages, and `typst compile` fetches curryst 0.6.0 on
first use. Typst refuses a curryst tree with binary rules more than nine
inferences high, so a higher tree is written in a layout of linlog's own
instead: a `#context` block that lists the inferences and the Typst code
that measures and places them, which needs no package and sets a tree of
any height. `--style layout=linlog` asks for it at any height,
`layout=curryst` for curryst always, and `premise_gap`, `label_gap`,
`band` and `stroke` set its spacing, each a number and one of the units
`pt`, `mm`, `cm`, `in` and `em`:

```console
$ linlog prove --format typst --style layout=linlog "A & B |- A + B" | head -12
// provable (ALL, classical, additive engine)
#context {
  let open = "dots"
  let gap = (1.5em).to-absolute()
  let name-gap = (0.2em).to-absolute()
  let band = (0.8em).to-absolute()
  let stroke = 0.05em
  let dots = $dots.v$
  let nodes = (
    (1, $⊕_1$, $⊢ A^⊥ ⊕ B^⊥, A ⊕ B$),
    (1, $⊕_1$, $⊢ A^⊥, A ⊕ B$),
    (0, $"ax"$, $⊢ A^⊥, A$),
```

```console
$ linlog prove --format latex --standalone --output proof.tex "!A |- A * !A"
$ pdflatex proof.tex
```

Names of more than one letter are set in italics (`\mathit{foo}`,
`italic("foo")`), with the characters LaTeX or Typst treat specially
escaped; LaTeX writes Greek letters as their commands (`\alpha`) and the
`‿` and `·` of the names read from the LLTP library as `\smallsmile` and
`\cdotp`, so that pdfLaTeX compiles them. In `interact`, `show --latex` and `show --typst` draw the
derivation so far, an open goal as its sequent under vertical dots.

`--format svg` draws the derivation as an SVG image instead, and with
`--net` the proof net of the proof, for the sequents `--net` takes, as
the literature draws nets: every subformula occurrence of the
conclusions written as its formula, where it stands in its conclusion;
the literals along the top, every axiom link an arc over the two it
joins; each `⊗` or `⅋` link joining its two premises to its conclusion
written below them, the connective small at the joint (the premise
edges of a `⅋` dashed and blue, since a switching keeps one of them);
and the sequent's formulas at the bottom in its order, a literal among
them reached by its link's arc. `--style svg.net=compact` writes only the
literals and the conclusions, every inner `⊗` and `⅋` a small circle,
for a net too wide to write every subformula. The verdict becomes an
XML comment. `--style svg.essential=true` draws the net of an
intuitionistic sequent as its essential net: every formula written as
the reading reads it (a hypothesis `A ⊸ B`, a literal as its atom), every
edge an arrow from the hypotheses toward the goal, the input side in
green (`svg.input`), the links as `⊸` and `⊗`, each implication of the
goal's side joined to its antecedent by a dotted line (`svg.dominator`),
and a directed cycle or a path around an implication highlighted. A net
with boxes is drawn the same way, every `!`-box a rectangle around what
lies in it, nested boxes nested: its `!` and an auxiliary door for every
`?` outside it with a copy inside stand on its bottom border, and the
wires from the copies, dashed and blue like a `⅋`'s premise edges since a
switching keeps one wire of a `?`, leave the box through its doors; with
`svg.essential=true` an intuitionistic net with boxes is drawn as its
essential net too, a hypothesis `!A` (the conclusion `?~A`) and its
wires in green and led up into its copies, its rules those of `!`, and a
switching cycle highlighted, which is what decides it.
`--style svg.net_look=textbook` draws a net as the textbooks do instead:
every edge plain, every conclusion on the bottom row, a `!` with
auxiliary doors reached by an edge down from its door, and each `?` node
as the nodes of the rules, a dereliction `?d` under every copy (inside
the copy's box), each copy's wire leaving its boxes through doors of its
own, binary contractions `?c` joining them below, and a weakening `?w`.
Each of the three is a key of its own: `svg.net_look.premises=plain`,
`svg.net_look.conclusions=bottom` and `svg.net_look.why_not=rules`, and
`svg.net_look=linlog` is the default. `seq print --format svg` and `show --svg`
in `interact` draw a sequent and the derivation so far. The drawings ask for
the Euler Math font without embedding it; every text is stretched to the
width Euler Math gives it, so a viewer without the font keeps the layout.
`--font` sets and lays out a drawing in another math font the command
carries instead: `stix-two` (STIX Two Math), `garamond` (Garamond-Math),
`asana` (Asana Math) or `new-computer-modern` (New Computer Modern Math),
or `euler`, the default (`--style svg.font=NAME`
and a style file's `"font": "NAME"` say the same).
The text stays selectable, and `--standalone` is refused, since an SVG is
always a document.

```console
$ linlog prove -i --format svg --output proof.svg "1, A & B, B -o C |- C"
$ linlog prove --net --output net.svg "A * B |- B * A"
$ linlog prove --net --style svg.net=compact --output compact.svg "A * B |- B * A"
$ linlog prove --net --output boxes.svg "!a, !c |- !(a * a) * ?c"
$ linlog prove --net --style svg.net_look=textbook --output textbook.svg "!a, !b, !(c * d) |- !((a * a) * (d * c))"
$ linlog prove -i --net --style svg.essential=true --output essential.svg "A -o B, B -o C |- A -o C"
$ linlog prove -i --net --style svg.essential=true --output essential-boxes.svg "!a, !(a -o b) |- !(b * a)"
$ linlog seq print --format svg "A |- A"
<svg xmlns="http://www.w3.org/2000/svg" role="img" width="70.272" height="27.2" viewBox="0 0 4392 1700" font-family="'Euler Math', 'Neo Euler', serif" font-size="1000" fill="black">
<title>⊢ A⊥, A</title>
<g>
<text x="300" y="1190" textLength="1801" lengthAdjust="spacing">⊢ 𝐴</text>
<text x="2101" y="790" textLength="611" lengthAdjust="spacing" font-size="700">⊥</text>
<text x="2712" y="1190" textLength="1380" lengthAdjust="spacing">, 𝐴</text>
</g>
</svg>
$ linlog seq print --format svg --font stix-two "A |- A"
<svg xmlns="http://www.w3.org/2000/svg" role="img" width="62.416" height="27.2" viewBox="0 0 3901 1700" font-family="'STIX Two Math', serif" font-size="1000" fill="black">
<title>⊢ A⊥, A</title>
<g>
<text x="300" y="1190" textLength="1636" lengthAdjust="spacing">⊢ 𝐴</text>
<text x="1936" y="790" textLength="504" lengthAdjust="spacing" font-size="700">⊥</text>
<text x="2440" y="1190" textLength="1161" lengthAdjust="spacing">, 𝐴</text>
</g>
</svg>
```

`--format png` and `--format pdf` render the same drawings as a PNG image
(at twice the drawing's size, with its title and description) or a PDF
page (its text selectable), in the math font `--font` names, which the
command carries (Euler Math by default);
with `--output` the extension names the format, which `--format`
overrides, and the verdict goes to standard error. Neither is written to
a terminal. A file is made only with a derivation or a net in it (or in
JSON, always): for an unprovable sequent the verdict goes to standard
error and no file is made, and an SVG on standard output is a drawing
or nothing. The PDF is an archival PDF/A-4 document (PDF 2.0);
`--style pdf.compatible=true` makes it PDF/A-2u (PDF 1.7) for tools and
archives that take nothing newer, and `--style pdf.accessible=true` an
accessible PDF/UA-1 document (PDF/A-2a), whose drawing a screen reader
reads as a numbered list of the proof's inferences. The document's date
is `SOURCE_DATE_EPOCH` when it is set, so a build can reproduce it byte
for byte, and the current time otherwise. Every drawing carries that
reading too, as the SVG's description.

```console
$ linlog prove -i --output proof.pdf "1, A & B, B -o C |- C"
provable (IMALL, intuitionistic, two-sided engine)
```

Everything an output shows can be changed. `--style KEY=VALUE` sets one
option of the format's options (`labels` upright, subscript or off, the
shape of an open goal, LaTeX's turnstile alignment, ebproof options and
preamble, Typst's import and page, the SVG's font, sizes and colours, a
PNG's scale, the text tree's bar and gap; `--help` lists them), with a
prefix such as `svg.` for another format; `--style-file` reads them all
as JSON, one object per format, which is the form a front end keeps;
`--lemma` and `--prelude` name the Rocq lemma and its file's first lines;
`--no-verdict` writes the derivation alone.

```console
$ linlog prove -i --format latex --style labels=subscript --style align=false "A, A -o B |- B"
% provable (IMLL, intuitionistic, two-sided engine)
\begin{prooftree}
\infer0[$\mathrm{ax}$]{A \vdash A}
\infer0[$\mathrm{ax}$]{B \vdash B}
\infer2[$\multimap_{\mathrm{L}}$]{A, A \multimap B \vdash B}
\end{prooftree}
$ linlog prove --style gap=6 --style bar="=" "A * B |- A * B"
provable (MLL, classical, focus engine)
======= ax      ======= ax
⊢ ~A, A         ⊢ ~B, B
======================= ⊗
    ⊢ ~A, ~B, A ⊗ B
    ================ ⅋
    ⊢ ~A ⅋ ~B, A ⊗ B
$ echo '{"svg": {"background": "white"}, "png": {"scale": 3}}' > style.json
$ linlog prove --net --style-file style.json --output net.png "A * B |- B * A"
provable (MLL, classical, focus engine)
```

The first two draw these:

![The derivation of 1, A & B, B ⊸ C ⊢ C](https://raw.githubusercontent.com/linlog-prover/linlog/main/core/tests/snapshots/ill.svg)
![The proof net of ⊢ A⊥ ⅋ B⊥, B ⊗ A](https://raw.githubusercontent.com/linlog-prover/linlog/main/core/tests/snapshots/net.svg)

and the net with boxes, its two copies of `?a⊥` in the box of
`!(a ⊗ a)`:

![The proof net of ⊢ ?a⊥, ?c⊥, !(a ⊗ a) ⊗ ?c](https://raw.githubusercontent.com/linlog-prover/linlog/main/core/tests/snapshots/net-boxes.svg)

and a net in the textbooks' look, its two copies of `a⊥` derelicted in
the box and contracted below it, `?b⊥` weakened:

![The proof net of ⊢ ?a⊥, ?b⊥, ?(c⊥ ⅋ d⊥), !((a ⊗ a) ⊗ (d ⊗ c)) as the textbooks draw it](https://raw.githubusercontent.com/linlog-prover/linlog/main/core/tests/snapshots/net-promotion-textbook.svg)

`--settings FILE` reads every setting the flags set from one JSON file,
in the form a front end keeps: an object with any of `clock`, `limits`,
`search`, `view`, `styles`, `batch` and `ordinary`, each holding the
fields the flags name in their `--help` (`--copies` sets
`search.copies`, `--timeout` `clock.time_limit_ms`, `--memory-limit`
`limits.memory_bytes`). A field the file leaves out keeps the command's
default, and a flag given beside the file overrides it. `batch.mode` is
the mode of a batch's entries that name none, as the mode flags set it;
`ordinary` applies under `--logic`, which names the logic: the file's
`ordinary.logic` is not read (the whole settings a front end keeps always
hold one, which would make every input ordinary logic), and its
`ordinary.translation` is the translation unless `--translation` names
one.

```console
$ echo '{"search": {"copies": 0, "engine": "focus"}}' > settings.json
$ linlog prove --settings settings.json "|- ?(~a * ~a), a * a * a * a"
unknown (MELL, classical, focus engine): the copy bound of 0 was reached after 300.39µs; raise it with --copies N, or lift it with --copies none to deepen it while the time limit lasts
$ linlog prove --settings settings.json --copies 3 "|- ?(~a * ~a), a * a * a * a"
unprovable (MELL, classical, focus engine): the search was exhaustive
```
